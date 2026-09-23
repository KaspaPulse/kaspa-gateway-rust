use flate2::read::GzDecoder;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use tar::Archive;
use tempfile::{TempDir, tempdir};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const REPOSITORY: &str = "KaspaPulse/kaspa-gateway-rust";
const ARCHIVE_ROOT: &str = "ksss-consumer-runtime-v1";
const RUNTIME_MODULES: &[&str] = &[
    "bundle_verify",
    "canonical",
    "classify_changes",
    "consumer_api",
    "knowledge",
    "resolver",
    "runtime_verify",
    "schema_registry",
];
static PY_RUNTIME_LOCK: Mutex<()> = Mutex::new(());

struct RuntimeBundle {
    _temp: TempDir,
    root: PathBuf,
    manifest: Value,
}
fn ksss_dir(root: &Path) -> PathBuf {
    root.join(".security").join("ksss")
}

fn trace_stage(stage: &str) {
    if std::env::var_os("KGW_KSSS_TRACE").is_some() {
        eprintln!("KSSS_STAGE={stage}");
    }
}

fn load_json(path: &Path, label: &str) -> Result<Value, String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| format!("{label}_MISSING_OR_UNSAFE"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label}_MISSING_OR_UNSAFE"));
    }
    let bytes = fs::read(path).map_err(|_| format!("{label}_INVALID_JSON"))?;
    let value: Value =
        serde_json::from_slice(&bytes).map_err(|_| format!("{label}_INVALID_JSON"))?;
    if !value.is_object() {
        return Err(format!("{label}_INVALID"));
    }
    Ok(value)
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("MISSING_OR_INVALID:{key}"))
}

fn integer(value: &Value, key: &str) -> Result<i64, String> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("MISSING_OR_INVALID:{key}"))
}
fn array_strings(value: &Value, key: &str) -> Result<Vec<String>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("MISSING_OR_INVALID:{key}"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("MISSING_OR_INVALID:{key}"))
        })
        .collect()
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| format!("UNSAFE_OR_MISSING_FILE:{}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("UNSAFE_OR_MISSING_FILE:{}", path.display()));
    }
    let mut file = File::open(path).map_err(|error| format!("SHA256_OPEN_FAILED:{error}"))?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("SHA256_READ_FAILED:{error}"))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn load_policy(root: &Path) -> Result<Value, String> {
    load_json(&ksss_dir(root).join("trust-policy.json"), "TRUST_POLICY")
}

fn evidence_dir(root: &Path, policy: &Value) -> Result<PathBuf, String> {
    let release = string(policy, "ksss_release")?;
    if release.is_empty() || release.contains('/') || release.contains('\\') {
        return Err("TRUST_POLICY_RELEASE_INVALID".to_owned());
    }
    let path = ksss_dir(root).join("trust").join("evidence").join(release);
    if !path.is_dir() {
        return Err("RUNTIME_EVIDENCE_DIRECTORY_MISSING".to_owned());
    }
    Ok(path)
}

fn runtime_artifact_path(root: &Path, policy: &Value) -> Result<PathBuf, String> {
    let name = string(policy, "runtime_artifact_filename")?;
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        return Err("RUNTIME_ARTIFACT_FILENAME_INVALID".to_owned());
    }
    Ok(evidence_dir(root, policy)?.join(name))
}

fn verify_artifact_pin(root: &Path) -> Result<(Value, PathBuf), String> {
    let policy = load_policy(root)?;
    if integer(&policy, "runtime_sequence")? < integer(&policy, "minimum_allowed_runtime_sequence")?
    {
        return Err("RUNTIME_SEQUENCE_BELOW_FLOOR".to_owned());
    }
    let artifact = runtime_artifact_path(root, &policy)?;
    let expected = string(&policy, "runtime_artifact_sha256")?;
    if expected.len() != 64 || sha256_file(&artifact)? != expected {
        return Err("RUNTIME_ARTIFACT_SHA256_MISMATCH".to_owned());
    }
    Ok((policy, artifact))
}
fn safe_relative_member(raw: &str) -> Result<PathBuf, String> {
    let prefix = format!("{ARCHIVE_ROOT}/");
    let Some(relative) = raw.strip_prefix(&prefix) else {
        return Err("RUNTIME_ARCHIVE_ROOT_INVALID".to_owned());
    };
    if relative.is_empty() || relative.starts_with("./") || relative.contains('\\') {
        return Err("RUNTIME_ARCHIVE_PATH_INVALID".to_owned());
    }
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("RUNTIME_ARCHIVE_PATH_INVALID".to_owned());
    }
    Ok(path.to_path_buf())
}

fn extract_runtime(root: &Path) -> Result<RuntimeBundle, String> {
    trace_stage("extract_runtime:start");
    let (policy, artifact) = verify_artifact_pin(root)?;
    trace_stage("extract_runtime:pin_verified");
    let temp = tempdir().map_err(|error| format!("RUNTIME_TEMP_DIR_FAILED:{error}"))?;
    let file =
        File::open(&artifact).map_err(|error| format!("RUNTIME_ARCHIVE_OPEN_FAILED:{error}"))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let mut seen = BTreeSet::new();

    for entry in archive
        .entries()
        .map_err(|error| format!("RUNTIME_ARCHIVE_INVALID:{error}"))?
    {
        let mut entry = entry.map_err(|error| format!("RUNTIME_ARCHIVE_INVALID:{error}"))?;
        if !entry.header().entry_type().is_file() {
            return Err("RUNTIME_ARCHIVE_NON_REGULAR_MEMBER".to_owned());
        }
        let raw = String::from_utf8(entry.path_bytes().into_owned())
            .map_err(|_| "RUNTIME_ARCHIVE_PATH_INVALID".to_owned())?;
        let relative = safe_relative_member(&raw)?;
        let key = relative.to_string_lossy().replace('\\', "/");
        if !seen.insert(key) {
            return Err("RUNTIME_ARCHIVE_DUPLICATE_MEMBER".to_owned());
        }
        let target = temp.path().join(ARCHIVE_ROOT).join(&relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("RUNTIME_ARCHIVE_CREATE_DIR_FAILED:{error}"))?;
        }
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|error| format!("RUNTIME_ARCHIVE_MEMBER_CREATE_FAILED:{error}"))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("RUNTIME_ARCHIVE_MEMBER_COPY_FAILED:{error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644))
                .map_err(|error| format!("RUNTIME_ARCHIVE_PERMISSIONS_FAILED:{error}"))?;
        }
    }

    trace_stage("extract_runtime:archive_extracted");
    let runtime_root = temp.path().join(ARCHIVE_ROOT);
    let manifest = load_json(
        &runtime_root.join("runtime-manifest.json"),
        "RUNTIME_MANIFEST",
    )?;
    for key in [
        "ksss_release",
        "ksss_source_sha",
        "runtime_sequence",
        "runtime_files_digest",
        "policy_bundle_digest",
    ] {
        if manifest.get(key) != policy.get(key) {
            return Err(format!("RUNTIME_POLICY_BINDING_MISMATCH:{key}"));
        }
    }
    if manifest.get("network_required_for_execution") != Some(&Value::Bool(false)) {
        return Err("RUNTIME_NETWORK_REQUIREMENT_INVALID".to_owned());
    }
    if manifest.get("external_ksss_checkout_required") != Some(&Value::Bool(false)) {
        return Err("RUNTIME_EXTERNAL_CHECKOUT_INVALID".to_owned());
    }
    trace_stage("extract_runtime:done");
    Ok(RuntimeBundle {
        _temp: temp,
        root: runtime_root,
        manifest,
    })
}

fn py_failure(label: &str, error: PyErr) -> String {
    format!("{label}:{error}")
}

fn json_to_py<'py>(py: Python<'py>, value: &Value) -> Result<Bound<'py, PyAny>, String> {
    let module = py
        .import("json")
        .map_err(|error| py_failure("PY_JSON_IMPORT", error))?;
    let payload =
        serde_json::to_string(value).map_err(|error| format!("JSON_SERIALIZE_FAILED:{error}"))?;
    module
        .call_method1("loads", (payload,))
        .map_err(|error| py_failure("PY_JSON_LOADS", error))
}

fn py_to_json(value: &Bound<'_, PyAny>) -> Result<Value, String> {
    let py = value.py();
    let module = py
        .import("json")
        .map_err(|error| py_failure("PY_JSON_IMPORT", error))?;
    let payload: String = module
        .call_method1("dumps", (value,))
        .map_err(|error| py_failure("PY_JSON_DUMPS", error))?
        .extract()
        .map_err(|error| py_failure("PY_JSON_EXTRACT", error))?;
    serde_json::from_str(&payload).map_err(|error| format!("PY_JSON_RESULT_INVALID:{error}"))
}

fn with_api<T>(
    bundle: &RuntimeBundle,
    callback: impl for<'py> FnOnce(Python<'py>, &Bound<'py, PyModule>) -> Result<T, String>,
) -> Result<T, String> {
    let _runtime_guard = PY_RUNTIME_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    trace_stage("with_api:before_attach");
    Python::attach(|py| {
        trace_stage("with_api:attached");
        let sys = py
            .import("sys")
            .map_err(|error| py_failure("PY_SYS_IMPORT", error))?;
        let modules = sys
            .getattr("modules")
            .map_err(|error| py_failure("PY_SYS_MODULES", error))?;
        for name in RUNTIME_MODULES {
            let present = modules
                .contains(*name)
                .map_err(|error| py_failure("RUNTIME_MODULE_COLLISION_CHECK", error))?;
            if present {
                return Err(format!("RUNTIME_MODULE_NAME_COLLISION:{name}"));
            }
        }

        let scripts = bundle.root.join("scripts");
        let scripts_text = scripts.to_string_lossy().into_owned();
        let path_list = sys
            .getattr("path")
            .map_err(|error| py_failure("PY_SYS_PATH", error))?;
        path_list
            .call_method1("insert", (0, scripts_text.as_str()))
            .map_err(|error| py_failure("PY_SYS_PATH_INSERT", error))?;

        trace_stage("with_api:path_inserted");
        let result = (|| {
            let pathlib = py
                .import("pathlib")
                .map_err(|error| py_failure("PY_PATHLIB_IMPORT", error))?;
            let runtime_path = pathlib
                .getattr("Path")
                .and_then(|class| class.call1((bundle.root.to_string_lossy().as_ref(),)))
                .map_err(|error| py_failure("PY_RUNTIME_PATH", error))?;
            trace_stage("with_api:before_runtime_verify_import");
            let verifier = py
                .import("runtime_verify")
                .map_err(|error| py_failure("RUNTIME_SELF_VERIFIER_IMPORT", error))?;
            trace_stage("with_api:runtime_verify_imported");
            let verification = verifier
                .getattr("verify_runtime")
                .and_then(|function| function.call1((runtime_path,)))
                .map_err(|error| py_failure("RUNTIME_SELF_VERIFY_FAILED", error))?;
            trace_stage("with_api:runtime_verify_called");
            let verification = py_to_json(&verification)?;
            if verification.get("status").and_then(Value::as_str) != Some("PASS") {
                return Err("RUNTIME_SELF_VERIFY_NOT_PASS".to_owned());
            }

            trace_stage("with_api:runtime_verify_pass");
            let api = py
                .import("consumer_api")
                .map_err(|error| py_failure("RUNTIME_API_IMPORT", error))?;
            let version: String = api
                .getattr("RUNTIME_API_VERSION")
                .and_then(|value| value.extract())
                .map_err(|error| py_failure("RUNTIME_API_VERSION_READ", error))?;
            if version != "1.0.0" {
                return Err("RUNTIME_API_VERSION_MISMATCH".to_owned());
            }
            trace_stage("with_api:consumer_api_ready");
            callback(py, &api)
        })();

        let _ = path_list.call_method1("remove", (scripts_text.as_str(),));
        for name in RUNTIME_MODULES {
            let _ = modules.call_method1("pop", (*name, py.None()));
        }
        result
    })
}

fn api_noarg_json(api: &Bound<'_, PyModule>, name: &str) -> Result<Value, String> {
    let result = api
        .getattr(name)
        .and_then(|function| function.call0())
        .map_err(|error| py_failure(&format!("RUNTIME_API_CALL:{name}"), error))?;
    py_to_json(&result)
}

fn api_one_json(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    name: &str,
    argument: &Value,
) -> Result<Value, String> {
    let argument = json_to_py(py, argument)?;
    let result = api
        .getattr(name)
        .and_then(|function| function.call1((argument,)))
        .map_err(|error| py_failure(&format!("RUNTIME_API_CALL:{name}"), error))?;
    py_to_json(&result)
}

fn api_one_string_json(
    api: &Bound<'_, PyModule>,
    name: &str,
    value: &str,
) -> Result<Value, String> {
    let result = api
        .getattr(name)
        .and_then(|function| function.call1((value,)))
        .map_err(|error| py_failure(&format!("RUNTIME_API_CALL:{name}"), error))?;
    py_to_json(&result)
}
fn api_kwargs_json(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    name: &str,
    kwargs: &[(&str, Value)],
) -> Result<Value, String> {
    let dictionary = PyDict::new(py);
    for (key, value) in kwargs {
        dictionary
            .set_item(*key, json_to_py(py, value)?)
            .map_err(|error| py_failure("RUNTIME_API_KWARG", error))?;
    }
    let result = api
        .getattr(name)
        .and_then(|function| function.call((), Some(&dictionary)))
        .map_err(|error| py_failure(&format!("RUNTIME_API_CALL:{name}"), error))?;
    py_to_json(&result)
}

fn api_record_reusable(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    record: &Value,
    context: &Value,
    active_change_types: &Value,
) -> Result<bool, String> {
    let kwargs = PyDict::new(py);
    kwargs
        .set_item("context", json_to_py(py, context)?)
        .map_err(|error| py_failure("RUNTIME_API_KWARG", error))?;
    kwargs
        .set_item("active_change_types", json_to_py(py, active_change_types)?)
        .map_err(|error| py_failure("RUNTIME_API_KWARG", error))?;
    let record = json_to_py(py, record)?;
    api.getattr("record_reusable")
        .and_then(|function| function.call((record,), Some(&kwargs)))
        .and_then(|result| result.extract::<bool>())
        .map_err(|error| py_failure("RUNTIME_API_CALL:record_reusable", error))
}

fn api_assert_valid(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    document: &Value,
) -> Result<Value, String> {
    api_one_json(py, api, "assert_valid_document", document)
}
fn api_canonical_sha256(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    document: &Value,
) -> Result<String, String> {
    let argument = json_to_py(py, document)?;
    api.getattr("canonical_sha256")
        .and_then(|function| function.call1((argument,)))
        .and_then(|result| result.extract::<String>())
        .map_err(|error| py_failure("RUNTIME_API_CALL:canonical_sha256", error))
}

fn api_classify_paths(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    paths: &[String],
) -> Result<Vec<String>, String> {
    let list = PyList::new(py, paths).map_err(|error| py_failure("PY_LIST_CREATE", error))?;
    api.getattr("classify_paths")
        .and_then(|function| function.call1((list,)))
        .and_then(|result| result.extract::<Vec<String>>())
        .map_err(|error| py_failure("RUNTIME_API_CALL:classify_paths", error))
}

fn api_validate_json_schema(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    document: &Value,
    schema: &Value,
) -> Result<Vec<String>, String> {
    let registry = api
        .getattr("schema_registry")
        .map_err(|error| py_failure("RUNTIME_SCHEMA_REGISTRY", error))?;
    let document = json_to_py(py, document)?;
    let schema = json_to_py(py, schema)?;
    registry
        .getattr("validate_json_schema")
        .and_then(|function| function.call1((document, schema)))
        .and_then(|result| result.extract::<Vec<String>>())
        .map_err(|error| py_failure("RUNTIME_API_CALL:validate_json_schema", error))
}

struct TrustStructure {
    policy: Value,
    directory: PathBuf,
}

fn pointer_str<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("MISSING_OR_INVALID:{pointer}"))
}

fn pointer_i64(value: &Value, pointer: &str) -> Result<i64, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("MISSING_OR_INVALID:{pointer}"))
}

fn parse_rfc3339(value: &str, label: &str) -> Result<OffsetDateTime, String> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| format!("{label}_INVALID"))
}

fn parse_sha256sums(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let text = fs::read_to_string(path).map_err(|_| "SHA256SUMS_MISSING_OR_UNSAFE".to_owned())?;
    let mut entries = BTreeMap::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((digest, raw_name)) = line.split_once(char::is_whitespace) else {
            return Err("SHA256SUMS_LINE_INVALID".to_owned());
        };
        let name = raw_name.trim_start().trim_start_matches('*');
        if digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || name.contains('/')
            || name.contains('\\')
            || entries.contains_key(name)
        {
            return Err("SHA256SUMS_PATH_INVALID".to_owned());
        }
        entries.insert(name.to_owned(), digest.to_owned());
    }
    Ok(entries)
}
fn verify_evidence_manifest(directory: &Path) -> Result<BTreeMap<String, String>, String> {
    let entries = parse_sha256sums(&directory.join("SHA256SUMS"))?;
    let mut actual = BTreeSet::new();
    for item in fs::read_dir(directory).map_err(|_| "EVIDENCE_DIRECTORY_MISSING".to_owned())? {
        let item = item.map_err(|_| "EVIDENCE_DIRECTORY_READ_FAILED".to_owned())?;
        let name = item.file_name().to_string_lossy().into_owned();
        if name == "SHA256SUMS" {
            continue;
        }
        if item
            .file_type()
            .map_err(|_| "EVIDENCE_FILE_TYPE_FAILED".to_owned())?
            .is_file()
        {
            actual.insert(name);
        }
    }
    if actual != entries.keys().cloned().collect() {
        return Err("EVIDENCE_FILE_SET_MISMATCH".to_owned());
    }
    for (name, expected) in &entries {
        if sha256_file(&directory.join(name))? != *expected {
            return Err(format!("EVIDENCE_DIGEST_MISMATCH:{name}"));
        }
    }
    Ok(entries)
}

fn verify_structure(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    root: &Path,
    explicit_time: &str,
) -> Result<TrustStructure, String> {
    let policy = load_policy(root)?;
    let directory = evidence_dir(root, &policy)?;
    verify_evidence_manifest(&directory)?;

    let trust_root_path = directory.join("ksss-trust-root-v1.json");
    let trust_root = load_json(&trust_root_path, "TRUST_ROOT")?;
    if sha256_file(&trust_root_path)? != string(&policy, "trust_root_file_sha256")? {
        return Err("TRUST_ROOT_FILE_DIGEST_MISMATCH".to_owned());
    }
    if api_canonical_sha256(py, api, &trust_root)?
        != string(&policy, "trust_root_canonical_sha256")?
    {
        return Err("TRUST_ROOT_CANONICAL_DIGEST_MISMATCH".to_owned());
    }
    let now = parse_rfc3339(explicit_time, "EXPLICIT_TIME")?;
    let valid_from = parse_rfc3339(string(&trust_root, "valid_from")?, "TRUST_ROOT_VALID_FROM")?;
    let expires_at = parse_rfc3339(string(&trust_root, "expires_at")?, "TRUST_ROOT_EXPIRES_AT")?;
    if now < valid_from {
        return Err("TRUST_ROOT_NOT_YET_VALID".to_owned());
    }
    if now >= expires_at {
        return Err("TRUST_ROOT_EXPIRED".to_owned());
    }
    for key in ["trusted_issuer", "trusted_signer_or_workflow_identity"] {
        if trust_root.get(key) != policy.get(key) {
            return Err(format!("TRUST_ROOT_{}_MISMATCH", key.to_ascii_uppercase()));
        }
    }
    if integer(&trust_root, "minimum_allowed_ksss_sequence")?
        > integer(&policy, "expected_sequence")?
    {
        return Err("KSSS_SEQUENCE_BELOW_ROOT_FLOOR".to_owned());
    }

    let release = string(&policy, "ksss_release")?;
    let identity = load_json(
        &directory.join(format!("ksss-release-identity-{release}.json")),
        "RELEASE_IDENTITY",
    )?;
    let metadata = load_json(
        &directory.join(format!("ksss-trusted-release-{release}.json")),
        "RELEASE_METADATA",
    )?;
    let release_statement = load_json(
        &directory.join(format!("ksss-release-attestation-{release}.intoto.json")),
        "RELEASE_STATEMENT",
    )?;
    let runtime_statement = load_json(
        &directory.join(format!("ksss-consumer-runtime-{release}.intoto.json")),
        "RUNTIME_STATEMENT",
    )?;
    let expected_source = string(&policy, "ksss_source_sha")?;
    let expected_bundle = string(&policy, "policy_bundle_digest")?;
    if string(&identity, "KSSS_RELEASE")? != release
        || string(&identity, "SOURCE_SHA")? != expected_source
    {
        return Err("RELEASE_IDENTITY_MISMATCH".to_owned());
    }
    if string(&identity, "POLICY_BUNDLE_DIGEST")? != expected_bundle {
        return Err("POLICY_BUNDLE_DIGEST_MISMATCH".to_owned());
    }
    if string(&metadata, "release")? != release
        || string(&metadata, "source_sha")? != expected_source
    {
        return Err("TRUSTED_RELEASE_IDENTITY_MISMATCH".to_owned());
    }
    if integer(&metadata, "sequence")? != integer(&policy, "expected_sequence")? {
        return Err("TRUSTED_RELEASE_SEQUENCE_MISMATCH".to_owned());
    }
    if string(&metadata, "policy_bundle_digest")? != expected_bundle {
        return Err("TRUSTED_RELEASE_BUNDLE_MISMATCH".to_owned());
    }
    if pointer_str(&release_statement, "/subject/0/digest/sha256")? != expected_bundle {
        return Err("RELEASE_STATEMENT_SUBJECT_MISMATCH".to_owned());
    }
    if pointer_str(&release_statement, "/predicate/sourceSha")? != expected_source {
        return Err("RELEASE_STATEMENT_SOURCE_MISMATCH".to_owned());
    }

    let artifact = runtime_artifact_path(root, &policy)?;
    let runtime_sha = sha256_file(&artifact)?;
    if runtime_sha != string(&policy, "runtime_artifact_sha256")? {
        return Err("RUNTIME_ARTIFACT_SHA256_MISMATCH".to_owned());
    }
    if pointer_str(&runtime_statement, "/subject/0/digest/sha256")? != runtime_sha {
        return Err("RUNTIME_STATEMENT_SUBJECT_MISMATCH".to_owned());
    }
    if pointer_str(&runtime_statement, "/predicate/ksssRelease")? != release
        || pointer_str(&runtime_statement, "/predicate/sourceSha")? != expected_source
    {
        return Err("RUNTIME_STATEMENT_IDENTITY_MISMATCH".to_owned());
    }
    if pointer_i64(&runtime_statement, "/predicate/runtimeSequence")?
        != integer(&policy, "runtime_sequence")?
    {
        return Err("RUNTIME_SEQUENCE_MISMATCH".to_owned());
    }
    if pointer_str(&runtime_statement, "/predicate/runtimeFilesDigest")?
        != string(&policy, "runtime_files_digest")?
    {
        return Err("RUNTIME_FILES_DIGEST_MISMATCH".to_owned());
    }
    if runtime_statement.pointer("/predicate/networkRequiredForExecution")
        != Some(&Value::Bool(false))
    {
        return Err("RUNTIME_NETWORK_CONTRACT_INVALID".to_owned());
    }
    if runtime_statement.pointer("/predicate/externalKsssCheckoutRequired")
        != Some(&Value::Bool(false))
    {
        return Err("RUNTIME_EXTERNAL_CHECKOUT_CONTRACT_INVALID".to_owned());
    }

    Ok(TrustStructure { policy, directory })
}

fn cosign_verify(
    cosign: &OsStr,
    statement: &Path,
    bundle: &Path,
    trusted_root: &Path,
    identity: &str,
    issuer: &str,
) -> Result<(), String> {
    let output = Command::new(cosign)
        .arg("verify-blob")
        .arg(statement)
        .arg("--bundle")
        .arg(bundle)
        .arg("--certificate-identity")
        .arg(identity)
        .arg("--certificate-oidc-issuer")
        .arg(issuer)
        .arg("--trusted-root")
        .arg(trusted_root)
        .output()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                "COSIGN_NOT_AVAILABLE".to_owned()
            } else {
                format!("COSIGN_EXEC_FAILED:{error}")
            }
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail = stderr
            .chars()
            .rev()
            .take(400)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        return Err(format!("COSIGN_VERIFY_FAILED:{tail}"));
    }
    Ok(())
}

fn trust_verify(
    bundle: &RuntimeBundle,
    root: &Path,
    explicit_time: &str,
    require_cryptographic: bool,
    cosign: &OsStr,
) -> Result<Value, String> {
    with_api(bundle, |py, api| {
        let structural = verify_structure(py, api, root, explicit_time)?;
        let policy = &structural.policy;
        let release = string(policy, "ksss_release")?;
        let mut crypto = "NOT_REQUESTED";

        if require_cryptographic {
            let trusted_root = structural
                .directory
                .join(string(policy, "sigstore_trusted_root_filename")?);
            if sha256_file(&trusted_root)? != string(policy, "sigstore_trusted_root_sha256")? {
                return Err("SIGSTORE_TRUSTED_ROOT_DIGEST_MISMATCH".to_owned());
            }
            let identity = string(policy, "trusted_signer_or_workflow_identity")?;
            let issuer = string(policy, "trusted_issuer")?;
            cosign_verify(
                cosign,
                &structural
                    .directory
                    .join(format!("ksss-release-attestation-{release}.intoto.json")),
                &structural
                    .directory
                    .join(format!("ksss-release-attestation-{release}.sigstore.json")),
                &trusted_root,
                identity,
                issuer,
            )?;
            cosign_verify(
                cosign,
                &structural
                    .directory
                    .join(format!("ksss-consumer-runtime-{release}.intoto.json")),
                &structural
                    .directory
                    .join(format!("ksss-consumer-runtime-{release}.sigstore.json")),
                &trusted_root,
                identity,
                issuer,
            )?;
            crypto = "PASS";
        }

        Ok(json!({
            "status": "PASS",
            "repository": REPOSITORY,
            "ksssRelease": string(policy, "ksss_release")?,
            "ksssSourceSha": string(policy, "ksss_source_sha")?,
            "policyBundleDigest": string(policy, "policy_bundle_digest")?,
            "runtimeArtifactSha256": string(policy, "runtime_artifact_sha256")?,
            "runtimeFilesDigest": string(policy, "runtime_files_digest")?,
            "runtimeSequence": integer(policy, "runtime_sequence")?,
            "trustRoot": string(policy, "trust_root_version")?,
            "structuralVerification": "PASS",
            "cryptographicVerification": crypto,
            "runtimeCryptographicVerification": crypto,
            "policyEnforcementAuthorized": require_cryptographic && crypto == "PASS",
            "PRODUCTION_CHANGE": false,
            "DEPLOYMENT": false,
            "CLOUDFLARE_CHANGE": false,
            "D1_MUTATION": false,
            "USER_DATA_ACCESS": false
        }))
    })
}

fn git_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("GIT_EXEC_FAILED:{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "GIT_COMMAND_FAILED:{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

fn nul_paths(bytes: &[u8]) -> Result<Vec<String>, String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| String::from_utf8(part.to_vec()).map_err(|_| "GIT_PATH_NOT_UTF8".to_owned()))
        .collect()
}

fn fingerprint(root: &Path, names: &[String]) -> Result<String, String> {
    let mut names = names.to_vec();
    names.sort();
    names.dedup();
    let mut digest = Sha256::new();
    for name in names {
        digest.update(name.replace('\\', "/").as_bytes());
        digest.update([0]);
        let path = root.join(&name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let target = fs::read_link(&path)
                    .map_err(|error| format!("IDENTITY_SYMLINK_READ_FAILED:{error}"))?;
                digest.update(target.to_string_lossy().as_bytes());
            }
            Ok(metadata) if metadata.is_file() => {
                let bytes = fs::read(&path)
                    .map_err(|error| format!("IDENTITY_FILE_READ_FAILED:{error}"))?;
                digest.update(Sha256::digest(bytes));
            }
            _ => digest.update(b"<missing-or-non-file>"),
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn identity(root: &Path) -> Result<Value, String> {
    let tracked = nul_paths(&git_bytes(root, &["ls-files", "-z"])?)?;
    let untracked = nul_paths(&git_bytes(
        root,
        &["ls-files", "--others", "--exclude-standard", "-z"],
    )?)?;
    let source_sha = String::from_utf8(git_bytes(root, &["rev-parse", "HEAD"])?)
        .map_err(|_| "GIT_HEAD_NOT_UTF8".to_owned())?
        .trim()
        .to_owned();
    let git_tree = String::from_utf8(git_bytes(root, &["rev-parse", "HEAD^{tree}"])?)
        .map_err(|_| "GIT_TREE_NOT_UTF8".to_owned())?
        .trim()
        .to_owned();

    let staged = git_bytes(root, &["diff", "--cached", "--binary"])?;
    let unstaged = git_bytes(root, &["diff", "--binary"])?;
    let submodules = git_bytes(root, &["submodule", "status", "--recursive"])?;

    Ok(json!({
        "source_sha": source_sha,
        "git_tree": git_tree,
        "tree_hash": fingerprint(root, &tracked)?,
        "worktree_state": {
            "staged_fingerprint": sha256_bytes(&staged),
            "unstaged_fingerprint": sha256_bytes(&unstaged),
            "untracked_fingerprint": fingerprint(root, &untracked)?,
            "submodule_fingerprint": sha256_bytes(&submodules),
        }
    }))
}

fn classify(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    paths: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let mut kinds = BTreeSet::new();
    for path in paths {
        for kind in api_classify_paths(py, api, std::slice::from_ref(path))? {
            kinds.insert(kind);
        }
    }
    Ok(kinds.into_iter().collect())
}
fn resolve(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    root: &Path,
    explicit_time: &str,
) -> Result<Value, String> {
    let policy = load_json(
        &ksss_dir(root).join("repository-policy.yaml"),
        "REPOSITORY_POLICY",
    )?;
    let applicability = load_json(
        &ksss_dir(root).join("applicability-v1.json"),
        "APPLICABILITY",
    )?;
    let strengthening = load_json(
        &ksss_dir(root).join("local-strengthening.json"),
        "LOCAL_STRENGTHENING",
    )?;
    api_assert_valid(py, api, &policy)?;
    api_assert_valid(py, api, &strengthening)?;

    if string(&policy, "repository")? != REPOSITORY
        || string(&applicability, "repository")? != REPOSITORY
    {
        return Err("REPOSITORY_IDENTITY_MISMATCH".to_owned());
    }

    let catalog = api_noarg_json(api, "control_catalog")?;
    let controls = catalog
        .get("controls")
        .and_then(Value::as_array)
        .ok_or_else(|| "CONTROL_CATALOG_INVALID".to_owned())?;
    let mut catalog_by_id = BTreeMap::new();
    for item in controls {
        let id = string(item, "id")?.to_owned();
        catalog_by_id.insert(id, item.clone());
    }

    let required: BTreeSet<String> = array_strings(&applicability, "required_controls")?
        .into_iter()
        .collect();
    if required.is_empty() || required.iter().any(|id| !catalog_by_id.contains_key(id)) {
        return Err("APPLICABILITY_UNKNOWN_OR_EMPTY".to_owned());
    }

    let risk_required: BTreeSet<String> = policy
        .pointer("/controls/additional_required")
        .and_then(Value::as_array)
        .ok_or_else(|| "RISK_REQUIRED_CONTROLS_INVALID".to_owned())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| "RISK_REQUIRED_CONTROLS_INVALID".to_owned())
        })
        .collect::<Result<_, _>>()?;
    if !required.is_subset(&risk_required) {
        return Err("RISK_FLOOR_WEAKENED".to_owned());
    }
    let mut base_controls = Map::new();
    for (id, item) in &catalog_by_id {
        base_controls.insert(
            id.clone(),
            json!({
                "required": false,
                "exception_allowed": item.get("exception_allowed").cloned().unwrap_or(Value::Null),
                "parameters": {}
            }),
        );
    }
    let base = json!({"repository": REPOSITORY, "controls": base_controls});

    let audit_profile = string(&policy, "audit_profile")?;
    let profile = api_one_string_json(api, "profile", audit_profile)?;
    let profile_required: BTreeSet<String> = array_strings(&profile, "required_controls")?
        .into_iter()
        .collect();
    let repository_required: Vec<String> =
        required.intersection(&profile_required).cloned().collect();

    let effective = api_kwargs_json(
        py,
        api,
        "resolve_effective_policy",
        &[
            ("base_policy", base),
            (
                "risk_classification",
                json!({
                    "classification": policy.get("risk").cloned().unwrap_or(Value::Null),
                    "required_controls": risk_required.iter().cloned().collect::<Vec<_>>()
                }),
            ),
            (
                "applicability",
                json!({"required_controls": required.iter().cloned().collect::<Vec<_>>()}),
            ),
            (
                "repository_profile",
                json!({"profile": audit_profile, "required_controls": repository_required}),
            ),
            ("local_strengthening", strengthening),
            ("exceptions", json!([])),
            ("explicit_time", Value::String(explicit_time.to_owned())),
            ("resolver_version", Value::String("1.0.0".to_owned())),
        ],
    )?;
    let resolved: BTreeSet<String> = array_strings(&effective, "required_controls")?
        .into_iter()
        .collect();
    if resolved != required {
        return Err("REQUIRED_CONTROL_RESOLUTION_DRIFT".to_owned());
    }
    Ok(effective)
}
fn adoption(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    root: &Path,
    adopted_at: &str,
) -> Result<Value, String> {
    let trust = load_policy(root)?;
    let policy = load_json(
        &ksss_dir(root).join("repository-policy.yaml"),
        "REPOSITORY_POLICY",
    )?;
    let risk = policy.get("risk").cloned().unwrap_or(Value::Null);
    let applicability = load_json(
        &ksss_dir(root).join("applicability-v1.json"),
        "APPLICABILITY",
    )?;
    let value = json!({
        "schema_family": "ksss-adoption",
        "schema_version": "1.0.0",
        "repository": REPOSITORY,
        "ksss_release": string(&trust, "ksss_release")?,
        "ksss_source_sha": string(&trust, "ksss_source_sha")?,
        "policy_bundle_digest": string(&trust, "policy_bundle_digest")?,
        "repository_profile": string(&policy, "audit_profile")?,
        "risk_classification_digest": api_canonical_sha256(py, api, &risk)?,
        "applicability_digest": api_canonical_sha256(py, api, &applicability)?,
        "adopted_at": adopted_at,
    });
    api_assert_valid(py, api, &value)?;
    Ok(value)
}

fn reference_parity(root: &Path, manifest: &Value) -> Result<Value, String> {
    let proof = load_json(
        &ksss_dir(root).join("reference-parity.json"),
        "REFERENCE_PARITY",
    )?;
    let trust = load_policy(root)?;
    for key in ["ksss_release", "ksss_source_sha", "runtime_artifact_sha256"] {
        if proof.get(key) != trust.get(key) {
            return Err("REFERENCE_PARITY_IDENTITY_MISMATCH".to_owned());
        }
    }
    let rows = proof
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "REFERENCE_PARITY_INCOMPLETE".to_owned())?;
    let unique: BTreeSet<&str> = rows
        .iter()
        .filter_map(|row| row.get("path").and_then(Value::as_str))
        .collect();
    if proof.get("status").and_then(Value::as_str) != Some("PASS")
        || proof
            .get("mismatches")
            .and_then(Value::as_array)
            .is_none_or(|v| !v.is_empty())
        || proof.get("runtime_files_digest") != manifest.get("runtime_files_digest")
        || proof.get("compared_file_count").and_then(Value::as_u64) != Some(34)
        || proof.get("byte_equal_count").and_then(Value::as_u64) != Some(34)
        || rows.len() != 34
        || unique.len() != 34
    {
        return Err("REFERENCE_PARITY_INCOMPLETE".to_owned());
    }

    let files = manifest
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| "RUNTIME_MANIFEST_FILES_INVALID".to_owned())?;
    for row in rows {
        let path = string(row, "path")?;
        let Some(expected) = files.get(path).and_then(Value::as_str) else {
            return Err("REFERENCE_PARITY_FILE_MISMATCH".to_owned());
        };
        if row.get("byte_equal") != Some(&Value::Bool(true))
            || ["manifest_sha256", "runtime_sha256", "source_sha256"]
                .iter()
                .any(|key| row.get(*key).and_then(Value::as_str) != Some(expected))
        {
            return Err("REFERENCE_PARITY_FILE_MISMATCH".to_owned());
        }
    }
    Ok(json!({
        "status": "PASS",
        "compared_files": 34,
        "source_comparison_reused": true
    }))
}
fn learning(py: Python<'_>, api: &Bound<'_, PyModule>, root: &Path) -> Result<Value, String> {
    let index = load_json(
        &ksss_dir(root).join("learning/index.json"),
        "LEARNING_INDEX",
    )?;
    let mut total = 0_usize;
    for (family, field) in [
        ("known-failure", "known_failures"),
        ("known-good-path", "known_good_paths"),
    ] {
        let paths = array_strings(&index, field)?;
        for relative in paths {
            let record = load_json(&ksss_dir(root).join(&relative), "LEARNING_RECORD")?;
            let family_result = api_assert_valid(py, api, &record)?;
            let actual_family = family_result
                .as_array()
                .and_then(|items| items.first())
                .and_then(Value::as_str)
                .unwrap_or_default();
            if actual_family != family {
                return Err("LEARNING_FAMILY_MISMATCH".to_owned());
            }
            for reference in array_strings(&record, "evidence_refs")? {
                if let Some(relative) = reference.strip_prefix("repo:")
                    && !root.join(relative).is_file()
                {
                    return Err(format!("LEARNING_EVIDENCE_MISSING:{reference}"));
                }
            }
            let invalidated_by = record
                .get("invalidated_by")
                .and_then(Value::as_array)
                .ok_or_else(|| "LEARNING_INVALIDATORS_MISSING".to_owned())?;
            if invalidated_by.is_empty() {
                return Err("STALE_LEARNING_RECORD_ACCEPTED".to_owned());
            }
            let first_change = Value::Array(vec![invalidated_by[0].clone()]);
            if api_record_reusable(
                py,
                api,
                &record,
                record.get("applies_when").unwrap_or(&Value::Null),
                &first_change,
            )? {
                return Err("STALE_LEARNING_RECORD_ACCEPTED".to_owned());
            }
            total += 1;
        }
    }
    if array_strings(&index, "known_failures")?.is_empty()
        || array_strings(&index, "known_good_paths")?.is_empty()
    {
        return Err("LEARNING_INVENTORY_EMPTY".to_owned());
    }
    let _ = api_one_json(
        py,
        api,
        "validate_diagnosis_sequence",
        &json!(["PREVIOUS_KNOWLEDGE_SEARCH=PASS", "DIAGNOSIS"]),
    )?;
    if total == 0 {
        return Err("LEARNING_INVENTORY_EMPTY".to_owned());
    }
    Ok(json!({
        "known_failures": array_strings(&index, "known_failures")?.len(),
        "known_good_paths": array_strings(&index, "known_good_paths")?.len(),
        "historical_tests_reexecuted": false,
        "record_validation": "PASS"
    }))
}

fn api_load_document(api: &Bound<'_, PyModule>, relative: &str) -> Result<Value, String> {
    api_one_string_json(api, "load_document", relative)
}

fn api_one_kwargs_json(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    name: &str,
    argument: &Value,
    kwargs: &[(&str, Value)],
) -> Result<Value, String> {
    let dictionary = PyDict::new(py);
    for (key, value) in kwargs {
        dictionary
            .set_item(*key, json_to_py(py, value)?)
            .map_err(|error| py_failure("RUNTIME_API_KWARG", error))?;
    }
    let argument = json_to_py(py, argument)?;
    let result = api
        .getattr(name)
        .and_then(|function| function.call((argument,), Some(&dictionary)))
        .map_err(|error| py_failure(&format!("RUNTIME_API_CALL:{name}"), error))?;
    py_to_json(&result)
}

fn check_in_context(
    py: Python<'_>,
    api: &Bound<'_, PyModule>,
    bundle: &RuntimeBundle,
    root: &Path,
    explicit_time: &str,
) -> Result<Value, String> {
    verify_structure(py, api, root, explicit_time)?;

    let threat = load_json(&ksss_dir(root).join("threat-model.json"), "THREAT_MODEL")?;
    let threat_schema = api_load_document(api, "schemas/threat-model.schema.json")?;
    let errors = api_validate_json_schema(py, api, &threat, &threat_schema)?;
    if !errors.is_empty() || string(&threat, "repository")? != REPOSITORY {
        return Err(format!("THREAT_MODEL_INVALID:{}", errors.join(";")));
    }

    let parity = reference_parity(root, &bundle.manifest)?;
    let saved = load_json(&ksss_dir(root).join("ksss-adoption.json"), "KSSS_ADOPTION")?;
    let adopted_at = string(&saved, "adopted_at")?;
    if saved != adoption(py, api, root, adopted_at)? {
        return Err("ADOPTION_SNAPSHOT_DRIFT".to_owned());
    }
    let saved_effective = load_json(
        &ksss_dir(root).join("effective-policy.json"),
        "EFFECTIVE_POLICY",
    )?;
    if saved_effective != resolve(py, api, root, adopted_at)? {
        return Err("EFFECTIVE_POLICY_SNAPSHOT_DRIFT".to_owned());
    }
    let _ = resolve(py, api, root, explicit_time)?;
    let states = load_json(&ksss_dir(root).join("control-state.json"), "CONTROL_STATE")?;
    let applicability = load_json(
        &ksss_dir(root).join("applicability-v1.json"),
        "APPLICABILITY",
    )?;
    let required: BTreeSet<String> = array_strings(&applicability, "required_controls")?
        .into_iter()
        .collect();
    let state_controls = states
        .get("controls")
        .and_then(Value::as_object)
        .ok_or_else(|| "CONTROL_STATE_INCOMPLETE".to_owned())?;
    if required.iter().any(|id| !state_controls.contains_key(id)) {
        return Err("CONTROL_STATE_INCOMPLETE".to_owned());
    }
    if string(&states, "application_runtime_status")? != "NOT_VERIFIED" {
        return Err("RUNTIME_CLAIM_REQUIRES_SEPARATE_RECEIPT".to_owned());
    }

    Ok(json!({
        "adoption_status": "PASS",
        "ksss_release": string(&bundle.manifest, "ksss_release")?,
        "network_required_for_execution": false,
        "external_ksss_checkout_required": false,
        "reference_parity": parity,
        "learning": learning(py, api, root)?,
        "application_runtime_status": "NOT_VERIFIED",
        "release_qualification": "NOT_VERIFIED"
    }))
}

fn check(root: &Path, explicit_time: &str) -> Result<Value, String> {
    let bundle = extract_runtime(root)?;
    with_api(&bundle, |py, api| {
        check_in_context(py, api, &bundle, root, explicit_time)
    })
}

fn changed_paths(root: &Path, base: &str) -> Result<BTreeSet<String>, String> {
    if base.len() != 40
        || !base
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("BASE_MUST_BE_EXACT_COMMIT".to_owned());
    }
    let mut paths = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only", "-z", base, "HEAD", "--"],
        vec!["diff", "--name-only", "-z"],
        vec!["diff", "--cached", "--name-only", "-z"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        for path in nul_paths(&git_bytes(root, &args)?)? {
            paths.insert(path);
        }
    }
    Ok(paths)
}

fn resolve_base(root: &Path, candidate: Option<&str>) -> Result<String, String> {
    let valid = |value: &str| {
        value.len() == 40
            && value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    };
    match candidate {
        Some(value) if valid(value) && value != "0000000000000000000000000000000000000000" => {
            Ok(value.to_owned())
        }
        Some("0000000000000000000000000000000000000000") | None => {
            let parent = String::from_utf8(git_bytes(root, &["rev-parse", "HEAD^"])?)
                .map_err(|_| "GIT_PARENT_NOT_UTF8".to_owned())?
                .trim()
                .to_owned();
            if valid(&parent) {
                Ok(parent)
            } else {
                Err("BASE_MUST_BE_EXACT_COMMIT".to_owned())
            }
        }
        Some(_) => Err("BASE_MUST_BE_EXACT_COMMIT".to_owned()),
    }
}

fn evaluate(root: &Path, base: &str, explicit_time: &str) -> Result<Value, String> {
    let bundle = extract_runtime(root)?;
    with_api(&bundle, |py, api| {
        let _ = check_in_context(py, api, &bundle, root, explicit_time)?;
        let paths = changed_paths(root, base)?;
        let types = classify(py, api, &paths)?;
        let type_set: BTreeSet<&str> = types.iter().map(String::as_str).collect();
        let catalog = api_noarg_json(api, "control_catalog")?;
        let controls = catalog
            .get("controls")
            .and_then(Value::as_array)
            .ok_or_else(|| "CONTROL_CATALOG_INVALID".to_owned())?;
        let mut invalidated = Vec::new();
        for item in controls {
            let invalidators = item
                .pointer("/freshness/invalidated_by")
                .and_then(Value::as_array)
                .ok_or_else(|| "CONTROL_CATALOG_INVALID".to_owned())?;
            if invalidators
                .iter()
                .filter_map(Value::as_str)
                .any(|kind| type_set.contains(kind))
            {
                invalidated.push(string(item, "id")?.to_owned());
            }
        }
        invalidated.sort();
        Ok(json!({
            "status": "PASS",
            "identity": identity(root)?,
            "change_types": types,
            "invalidated_controls": invalidated,
            "changed_paths": paths.into_iter().collect::<Vec<_>>(),
            "automatic_test_execution": false
        }))
    })
}
fn load_learning_records(root: &Path, index: &Value, field: &str) -> Result<Vec<Value>, String> {
    array_strings(index, field)?
        .into_iter()
        .map(|relative| load_json(&ksss_dir(root).join(relative), "LEARNING_RECORD"))
        .collect()
}

fn load_global_records(directory: &Path) -> Result<Vec<Value>, String> {
    let mut paths = Vec::new();
    if directory.is_dir() {
        for item in
            fs::read_dir(directory).map_err(|_| "GLOBAL_KNOWLEDGE_READ_FAILED".to_owned())?
        {
            let path = item
                .map_err(|_| "GLOBAL_KNOWLEDGE_READ_FAILED".to_owned())?
                .path();
            if path.extension() == Some(OsStr::new("json")) {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| load_json(&path, "GLOBAL_KNOWLEDGE"))
        .collect()
}

fn knowledge_lookup(root: &Path, context_path: &Path) -> Result<Value, String> {
    let query = load_json(context_path, "KNOWLEDGE_QUERY")?;
    let index = load_json(
        &ksss_dir(root).join("learning/index.json"),
        "LEARNING_INDEX",
    )?;
    let bundle = extract_runtime(root)?;

    with_api(&bundle, |py, api| {
        let mut failures = load_learning_records(root, &index, "known_failures")?;
        let mut good = load_learning_records(root, &index, "known_good_paths")?;
        failures.extend(load_global_records(
            &bundle.root.join("learning/global-known-failures"),
        )?);
        good.extend(load_global_records(
            &bundle.root.join("learning/global-known-good-paths"),
        )?);
        for record in failures.iter().chain(&good) {
            api_assert_valid(py, api, record)?;
        }
        let context = query
            .get("context")
            .cloned()
            .ok_or_else(|| "KNOWLEDGE_CONTEXT_REQUIRED".to_owned())?;
        let changes = query
            .get("active_change_types")
            .cloned()
            .unwrap_or_else(|| json!([]));

        let matched_failures = if let Some(fingerprint) = query.get("failure_fingerprint") {
            api_one_kwargs_json(
                py,
                api,
                "lookup_known_failures",
                &Value::Array(failures.clone()),
                &[
                    ("fingerprint", fingerprint.clone()),
                    ("context", context.clone()),
                    ("active_change_types", changes.clone()),
                ],
            )?
        } else {
            json!([])
        };
        let matched_good =
            if let Some(operation_class) = query.get("operation_class").and_then(Value::as_str) {
                api_one_kwargs_json(
                    py,
                    api,
                    "lookup_known_good_paths",
                    &Value::Array(good.clone()),
                    &[
                        ("operation_class", Value::String(operation_class.to_owned())),
                        ("context", context),
                        ("active_change_types", changes),
                    ],
                )?
            } else {
                json!([])
            };

        let failure_rows = matched_failures.as_array().cloned().unwrap_or_default();
        let good_rows = matched_good.as_array().cloned().unwrap_or_default();
        let known_failures: Vec<Value> = failure_rows
            .iter()
            .map(|record| {
                json!({
                    "record_id": record.get("record_id").cloned().unwrap_or(Value::Null),
                    "recommended_action": record.get("recommended_action").cloned().unwrap_or(Value::Null),
                    "evidence_refs": record.get("evidence_refs").cloned().unwrap_or(Value::Null)
                })
            })
            .collect();
        let known_good_paths: Vec<Value> = good_rows
            .iter()
            .map(|record| record.get("record_id").cloned().unwrap_or(Value::Null))
            .collect();
        Ok(json!({
            "PREVIOUS_KNOWLEDGE_SEARCH": "PASS",
            "match_status": if known_failures.is_empty() && known_good_paths.is_empty() {
                "NO_VALID_MATCH"
            } else {
                "MATCH"
            },
            "known_failures": known_failures,
            "known_good_paths": known_good_paths,
            "historical_tests_reexecuted": false
        }))
    })
}

fn json_truthy(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => {
            value.as_i64().is_some_and(|number| number != 0)
                || value.as_u64().is_some_and(|number| number != 0)
                || value.as_f64().is_some_and(|number| number != 0.0)
        }
        Some(Value::String(value)) => !value.is_empty(),
        Some(Value::Array(value)) => !value.is_empty(),
        Some(Value::Object(value)) => !value.is_empty(),
    }
}

fn safe_evidence_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if relative.is_empty() {
        return Err("RUNTIME_EVIDENCE_PATH_INVALID".to_owned());
    }
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("RUNTIME_EVIDENCE_PATH_INVALID".to_owned());
    }
    let candidate = root.join(relative_path);
    let metadata =
        fs::symlink_metadata(&candidate).map_err(|_| "RUNTIME_EVIDENCE_PATH_INVALID".to_owned())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("RUNTIME_EVIDENCE_PATH_INVALID".to_owned());
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|_| "RUNTIME_EVIDENCE_PATH_INVALID".to_owned())?;
    let canonical = candidate
        .canonicalize()
        .map_err(|_| "RUNTIME_EVIDENCE_PATH_INVALID".to_owned())?;
    if !canonical.starts_with(&canonical_root) {
        return Err("RUNTIME_EVIDENCE_PATH_INVALID".to_owned());
    }
    Ok(canonical)
}

fn release_check_with_identity(
    root: &Path,
    receipt_path: &Path,
    artifact: &Path,
    evidence_root: &Path,
    expected_environment: &str,
    current: Value,
) -> Result<Value, String> {
    let receipt = load_json(receipt_path, "RUNTIME_RECEIPT")?;
    if string(&receipt, "schema")? != "kgw-runtime-qualification-receipt-v1" {
        return Err("RUNTIME_RECEIPT_SCHEMA_INVALID".to_owned());
    }
    if string(&receipt, "environment_fingerprint")? != expected_environment {
        return Err("RUNTIME_ENVIRONMENT_MISMATCH".to_owned());
    }
    for field in ["source_sha", "tree_hash", "worktree_state"] {
        if receipt.get(field) != current.get(field) {
            return Err(format!("RUNTIME_RECEIPT_IDENTITY_MISMATCH:{field}"));
        }
    }
    if string(&receipt, "artifact_sha256")? != sha256_file(artifact)? {
        return Err("RUNTIME_ARTIFACT_MISMATCH".to_owned());
    }
    let environment = string(&receipt, "environment_fingerprint")?;
    if environment.len() != 64
        || !environment
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("RUNTIME_ENVIRONMENT_IDENTITY_MISSING".to_owned());
    }
    if string(&receipt, "host_os")? != "windows"
        || string(&receipt, "test_kind")? != "real-packaged-application"
    {
        return Err("REAL_PACKAGED_WINDOWS_EVIDENCE_REQUIRED".to_owned());
    }

    let contract = load_json(
        &ksss_dir(root).join("runtime-contract.json"),
        "RUNTIME_CONTRACT",
    )?;
    let required_cases: BTreeSet<String> = array_strings(&contract, "cases")?.into_iter().collect();
    let cases = receipt
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| "RUNTIME_CASES_INCOMPLETE".to_owned())?;
    let actual_cases: BTreeSet<String> = cases
        .iter()
        .filter_map(|case| case.get("case_id").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect();
    if cases.len() != required_cases.len() || actual_cases != required_cases {
        return Err("RUNTIME_CASES_INCOMPLETE".to_owned());
    }
    let required_checks = array_strings(&contract, "required_checks")?;

    for case in cases {
        let case_id = string(case, "case_id")?;
        let owner = case
            .get("owner_identity")
            .and_then(Value::as_object)
            .ok_or_else(|| "EXACT_OWNER_IDENTITY_MISSING".to_owned())?;
        let pid = owner.get("pid").and_then(Value::as_i64).unwrap_or(0);
        if pid <= 0 {
            return Err("EXACT_OWNER_PID_MISSING".to_owned());
        }
        if !json_truthy(owner.get("start_time")) || !json_truthy(owner.get("executable")) {
            return Err("EXACT_OWNER_IDENTITY_MISSING".to_owned());
        }
        let checks = case
            .get("checks")
            .and_then(Value::as_object)
            .ok_or_else(|| "RUNTIME_CHECK_NOT_PASS".to_owned())?;
        for name in &required_checks {
            let item = checks
                .get(name)
                .and_then(Value::as_object)
                .ok_or_else(|| format!("RUNTIME_CHECK_NOT_PASS:{case_id}:{name}"))?;
            if item.get("status").and_then(Value::as_str) != Some("PASS") {
                return Err(format!("RUNTIME_CHECK_NOT_PASS:{case_id}:{name}"));
            }
            let relative = item
                .get("evidence_path")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let path = safe_evidence_path(evidence_root, relative)?;
            let expected = item
                .get("evidence_sha256")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if expected != sha256_file(&path)? {
                return Err("RUNTIME_EVIDENCE_DIGEST_MISMATCH".to_owned());
            }
        }
    }

    Ok(json!({
        "receipt_validation": "PASS",
        "application_executed_by_this_command": false,
        "artifact_sha256": string(&receipt, "artifact_sha256")?,
        "cases": cases.len()
    }))
}

fn release_check(
    root: &Path,
    receipt_path: &Path,
    artifact: &Path,
    evidence_root: &Path,
    expected_environment: &str,
) -> Result<Value, String> {
    release_check_with_identity(
        root,
        receipt_path,
        artifact,
        evidence_root,
        expected_environment,
        identity(root)?,
    )
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("OUTPUT_CREATE_DIR_FAILED:{error}"))?;
    }
    let mut payload = serde_json::to_string_pretty(value)
        .map_err(|error| format!("OUTPUT_JSON_FAILED:{error}"))?;
    payload.push('\n');
    fs::write(path, payload).map_err(|error| format!("OUTPUT_WRITE_FAILED:{error}"))
}

fn materialize(root: &Path, explicit_time: &str) -> Result<Value, String> {
    let bundle = extract_runtime(root)?;
    with_api(&bundle, |py, api| {
        verify_structure(py, api, root, explicit_time)?;
        let adoption_value = adoption(py, api, root, explicit_time)?;
        let effective = resolve(py, api, root, explicit_time)?;
        write_json(&ksss_dir(root).join("ksss-adoption.json"), &adoption_value)?;
        write_json(&ksss_dir(root).join("effective-policy.json"), &effective)?;
        Ok(json!({"materialization": "PASS"}))
    })
}

fn now_rfc3339() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("TIME_FORMAT_FAILED:{error}"))
}

struct CliOptions {
    values: BTreeMap<String, String>,
    flags: BTreeSet<String>,
}

fn parse_options(args: &mut impl Iterator<Item = String>) -> Result<CliOptions, String> {
    let mut values = BTreeMap::new();
    let mut flags = BTreeSet::new();
    while let Some(flag) = args.next() {
        if !flag.starts_with("--") {
            return Err(format!("unexpected positional argument: {flag}"));
        }
        if flag == "--require-cryptographic" {
            if !flags.insert(flag.clone()) {
                return Err(format!("duplicate option: {flag}"));
            }
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        if values.insert(flag.clone(), value).is_some() {
            return Err(format!("duplicate option: {flag}"));
        }
    }
    Ok(CliOptions { values, flags })
}

fn ensure_allowed(
    options: &CliOptions,
    value_flags: &[&str],
    boolean_flags: &[&str],
) -> Result<(), String> {
    for key in options.values.keys() {
        if !value_flags.contains(&key.as_str()) {
            return Err(format!("unknown option: {key}"));
        }
    }
    for key in &options.flags {
        if !boolean_flags.contains(&key.as_str()) {
            return Err(format!("unknown flag: {key}"));
        }
    }
    Ok(())
}

fn required(options: &CliOptions, name: &str) -> Result<String, String> {
    options
        .values
        .get(name)
        .cloned()
        .ok_or_else(|| format!("missing required option: {name}"))
}

fn emit(value: &Value, output: Option<&str>) -> Result<(), String> {
    if let Some(path) = output {
        write_json(Path::new(path), value)?;
    }
    let payload = serde_json::to_string_pretty(value)
        .map_err(|error| format!("OUTPUT_JSON_FAILED:{error}"))?;
    println!("{payload}");
    Ok(())
}

pub fn run_cli(
    args: &mut impl Iterator<Item = String>,
    repository_root: &Path,
) -> Result<(), String> {
    let action = args
        .next()
        .ok_or_else(|| "missing KSSS action".to_owned())?;
    let options = parse_options(args)?;
    let result = match action.as_str() {
        "check" => {
            ensure_allowed(&options, &["--explicit-time", "--output"], &[])?;
            let explicit_time = options
                .values
                .get("--explicit-time")
                .cloned()
                .unwrap_or(now_rfc3339()?);
            check(repository_root, &explicit_time)?
        }
        "trust-verify" => {
            ensure_allowed(
                &options,
                &["--explicit-time", "--cosign", "--output", "--repo"],
                &["--require-cryptographic"],
            )?;
            let explicit_time = options
                .values
                .get("--explicit-time")
                .cloned()
                .unwrap_or(now_rfc3339()?);
            let repo = options
                .values
                .get("--repo")
                .map(PathBuf::from)
                .unwrap_or_else(|| repository_root.to_path_buf());
            let cosign = options
                .values
                .get("--cosign")
                .cloned()
                .unwrap_or_else(|| "cosign".to_owned());
            let bundle = extract_runtime(&repo)?;
            trust_verify(
                &bundle,
                &repo,
                &explicit_time,
                options.flags.contains("--require-cryptographic"),
                OsStr::new(&cosign),
            )?
        }

        "evaluate" => {
            ensure_allowed(&options, &["--base", "--explicit-time", "--output"], &[])?;
            let base = resolve_base(
                repository_root,
                options.values.get("--base").map(String::as_str),
            )?;
            let explicit_time = options
                .values
                .get("--explicit-time")
                .cloned()
                .unwrap_or(now_rfc3339()?);
            evaluate(repository_root, &base, &explicit_time)?
        }
        "knowledge" => {
            ensure_allowed(&options, &["--context", "--output"], &[])?;
            knowledge_lookup(
                repository_root,
                Path::new(&required(&options, "--context")?),
            )?
        }
        "release-check" => {
            ensure_allowed(
                &options,
                &[
                    "--receipt",
                    "--artifact",
                    "--evidence-root",
                    "--environment-fingerprint",
                    "--output",
                ],
                &[],
            )?;
            release_check(
                repository_root,
                Path::new(&required(&options, "--receipt")?),
                Path::new(&required(&options, "--artifact")?),
                Path::new(&required(&options, "--evidence-root")?),
                &required(&options, "--environment-fingerprint")?,
            )?
        }
        "materialize" => {
            ensure_allowed(&options, &["--explicit-time", "--output"], &[])?;
            let explicit_time = options
                .values
                .get("--explicit-time")
                .cloned()
                .unwrap_or(now_rfc3339()?);
            materialize(repository_root, &explicit_time)?
        }
        _ => return Err(format!("unknown KSSS action: {action}")),
    };
    emit(&result, options.values.get("--output").map(String::as_str))
}

#[cfg(test)]
#[path = "ksss_tests.rs"]
mod tests;
