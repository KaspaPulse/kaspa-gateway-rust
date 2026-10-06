use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const TASK_ID: &str =
    "KASPA_GATEWAY_REAL_RUNTIME_ACCEPTANCE_EXPLORER_AND_RELEASE_ADMISSION_HARDENING_V1";

const REQUIRED_LOCAL_GATES: &[&str] = &[
    "SOURCE_INTEGRITY",
    "CARGO_FMT",
    "CLIPPY",
    "UNIT_TESTS",
    "PROPERTY_TESTS",
    "STATE_MODEL_TESTS",
    "TRANSACTION_SEMANTIC_TESTS",
    "DATABASE_MIGRATION_TESTS",
    "DATABASE_STORAGE_FAULT_MATRIX",
    "NODE_BRIDGE_HERMETIC_MATRIX",
    "EXPLORER_HERMETIC_MATRIX",
    "FAULT_INJECTION",
    "LOG_ISOLATION",
    "UI_TRUTH",
    "CONCURRENCY_RACE_MATRIX",
    "LARGE_DATA_MATRIX",
    "NATIVE_DEV_E2E",
    "REAL_NODE_BRIDGE_CANARY",
    "REAL_EXPLORER_CANARY",
    "PRODUCTION_RELEASE_BUILD",
    "NSIS_INSTALL",
    "INSTALLED_BINARY_E2E",
    "UPGRADE_ACCEPTANCE",
    "UNINSTALL_REINSTALL",
    "LIFECYCLE_STRESS",
    "FLAKY_DETECTION",
    "SECURITY",
    "LANGUAGE_POLICY",
    "SBOM_PRECHECK",
    "FINAL_LOCAL_RECEIPT",
];

const WINDOWS_NSIS: &str = "KaspaGateway-windows-x64-nsis.exe";
const WINDOWS_RAW_EXE: &str = "kaspa-gateway-desktop-windows-x64.exe";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    version: String,
    repository: PathBuf,
    evidence: PathBuf,
}

#[derive(Debug, Serialize)]
struct VersionContract {
    requested: String,
    package_json: String,
    tauri_config: String,
    cargo_package: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GateReceipt {
    status: String,
    evidence: String,
}

#[derive(Debug, Deserialize)]
struct LocalAdmissionEvidence {
    schema_version: u32,
    task_id: String,
    requested_version: String,
    head: String,
    tree: String,
    state_generation: u64,
    state_snapshot: String,
    artifact_source_head: String,
    windows_artifact_root: String,
    allowed_post_artifact_paths: Vec<String>,
    gates: BTreeMap<String, GateReceipt>,
}

#[derive(Debug, Serialize)]
struct WindowsArtifactReport {
    spdx_version: String,
    package_count: usize,
    desktop_version_matches: usize,
    nsis_sha256: String,
    raw_exe_sha256: String,
    installed_exe_sha256: String,
    requested_commit_sha: String,
    installer_smoke: String,
    uninstall_smoke: String,
    authenticode_status: String,
}

#[derive(Debug, Serialize)]
struct AdmissionReport {
    schema_version: u32,
    mode: &'static str,
    requested_version: String,
    head: String,
    tree: String,
    version_contract: VersionContract,
    required_gates: &'static [&'static str],
    verified_gates: Vec<String>,
    evidence_path: String,
    evidence_generation: u64,
    artifact_source_head: String,
    post_artifact_changes: Vec<String>,
    windows_artifact: WindowsArtifactReport,
    local_release_admission: &'static str,
    push_allowed: bool,
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<String, String> {
    let options = parse_args(args, default_root)?;
    match run_local(&options) {
        Ok(report) => serde_json::to_string_pretty(&report)
            .map_err(|error| format!("release-admission: serialize report: {error}")),
        Err(error) => Err(format!(
            "LOCAL_RELEASE_ADMISSION=FAIL; PUSH_ALLOWED=NO; {error}"
        )),
    }
}

fn run_local(options: &Options) -> Result<AdmissionReport, String> {
    let contract = verify_version_contract(&options.repository, &options.version)?;
    let (head, tree) = git_identity(&options.repository)?;
    verify_source_integrity(&options.repository)?;

    let evidence_path = resolve_inside_repository(
        &options.repository,
        &options.evidence,
        "local admission evidence",
    )?;
    let evidence: LocalAdmissionEvidence = serde_json::from_slice(
        &fs::read(&evidence_path)
            .map_err(|error| format!("read {}: {error}", evidence_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", evidence_path.display()))?;

    verify_evidence_identity(&evidence, &options.version, &head, &tree)?;
    let verified_gates = verify_gate_receipts(&evidence.gates)?;

    let state_snapshot_path = resolve_inside_repository(
        &options.repository,
        Path::new(&evidence.state_snapshot),
        "state snapshot",
    )?;
    let state_snapshot: Value = serde_json::from_slice(
        &fs::read(&state_snapshot_path)
            .map_err(|error| format!("read {}: {error}", state_snapshot_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", state_snapshot_path.display()))?;
    verify_state_snapshot(&state_snapshot, evidence.state_generation, &head, &tree)?;

    let post_artifact_changes = verify_post_artifact_changes(
        &options.repository,
        &evidence.artifact_source_head,
        &evidence.allowed_post_artifact_paths,
    )?;

    let artifact_root = resolve_inside_repository(
        &options.repository,
        Path::new(&evidence.windows_artifact_root),
        "Windows artifact root",
    )?;
    let windows_artifact = verify_windows_artifact(
        &artifact_root,
        &options.version,
        &evidence.artifact_source_head,
    )?;

    Ok(AdmissionReport {
        schema_version: 2,
        mode: "local",
        requested_version: options.version.clone(),
        head,
        tree,
        version_contract: contract,
        required_gates: REQUIRED_LOCAL_GATES,
        verified_gates,
        evidence_path: evidence_path.display().to_string(),
        evidence_generation: evidence.state_generation,
        artifact_source_head: evidence.artifact_source_head,
        post_artifact_changes,
        windows_artifact,
        local_release_admission: "PASS",
        push_allowed: true,
    })
}

fn parse_args(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<Options, String> {
    match args.next().as_deref() {
        Some("local") => {}
        Some(other) => {
            return Err(format!(
                "release-admission: unsupported mode {other:?}; expected local"
            ));
        }
        None => return Err("release-admission: missing mode; expected local".to_owned()),
    }

    let mut version = None;
    let mut repository = None;
    let mut evidence = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--version" if version.is_none() => {
                version = Some(next_value(args, "--version")?);
            }
            "--repository" if repository.is_none() => {
                repository = Some(PathBuf::from(next_value(args, "--repository")?));
            }
            "--evidence" if evidence.is_none() => {
                evidence = Some(PathBuf::from(next_value(args, "--evidence")?));
            }
            _ => {
                return Err(format!(
                    "release-admission: unknown or repeated argument {flag}"
                ));
            }
        }
    }

    let version = version.ok_or_else(|| "release-admission: --version is required".to_owned())?;
    validate_release_version(&version)?;

    let repository = repository.unwrap_or_else(|| default_root.to_path_buf());
    let repository = repository.canonicalize().map_err(|error| {
        format!(
            "release-admission: invalid repository {}: {error}",
            repository.display()
        )
    })?;
    if !repository.join("Cargo.toml").is_file() {
        return Err(format!(
            "release-admission: repository has no Cargo.toml: {}",
            repository.display()
        ));
    }

    let evidence = evidence.unwrap_or_else(|| {
        PathBuf::from("artifacts")
            .join("release-admission")
            .join("FINAL_LOCAL_RECEIPT.json")
    });

    Ok(Options {
        version,
        repository,
        evidence,
    })
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("release-admission: {flag} requires a value"))
}

fn validate_release_version(value: &str) -> Result<(), String> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.bytes().all(|byte| byte.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
        })
    {
        return Err(format!(
            "release-admission: version must be a stable X.Y.Z numeric version; got {value:?}"
        ));
    }
    Ok(())
}

fn json_version(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("release-admission: read {}: {error}", path.display()))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("release-admission: parse {}: {error}", path.display()))?;
    value
        .get("version")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "release-admission: {} has no string version",
                path.display()
            )
        })
}

fn cargo_package_version(path: &Path) -> Result<String, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("release-admission: read {}: {error}", path.display()))?;
    let mut in_package = false;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "version" {
            continue;
        }
        let value = value.trim();
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Ok(value[1..value.len() - 1].to_owned());
        }
        return Err(format!(
            "release-admission: {} package version is not a quoted literal",
            path.display()
        ));
    }
    Err(format!(
        "release-admission: {} has no [package] version",
        path.display()
    ))
}

fn verify_version_contract(root: &Path, requested: &str) -> Result<VersionContract, String> {
    let package_json = json_version(&root.join("apps/kaspa-gateway-desktop/package.json"))?;
    let tauri_config =
        json_version(&root.join("apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json"))?;
    let cargo_package =
        cargo_package_version(&root.join("apps/kaspa-gateway-desktop/src-tauri/Cargo.toml"))?;

    for (label, actual) in [
        ("package.json", package_json.as_str()),
        ("tauri.conf.json", tauri_config.as_str()),
        ("Cargo.toml", cargo_package.as_str()),
    ] {
        if actual != requested {
            return Err(format!(
                "version mismatch: requested={requested}; {label}={actual}"
            ));
        }
    }

    Ok(VersionContract {
        requested: requested.to_owned(),
        package_json,
        tauri_config,
        cargo_package,
    })
}

fn verify_source_integrity(root: &Path) -> Result<(), String> {
    git_success(
        root,
        &["diff", "--quiet", "--exit-code"],
        "unstaged content diff",
    )?;
    git_success(
        root,
        &["diff", "--cached", "--quiet", "--exit-code"],
        "staged content diff",
    )?;
    let untracked = git_stdout(root, &["ls-files", "--others", "--exclude-standard"])?;
    if !untracked.trim().is_empty() {
        return Err(format!(
            "source integrity failed: untracked non-ignored files exist: {}",
            untracked.replace('\n', ", ")
        ));
    }
    Ok(())
}

fn verify_evidence_identity(
    evidence: &LocalAdmissionEvidence,
    requested_version: &str,
    head: &str,
    tree: &str,
) -> Result<(), String> {
    if evidence.schema_version != 2 {
        return Err(format!(
            "local evidence schema mismatch: expected=2 actual={}",
            evidence.schema_version
        ));
    }
    if evidence.task_id != TASK_ID {
        return Err(format!(
            "local evidence task mismatch: expected={TASK_ID} actual={}",
            evidence.task_id
        ));
    }
    if evidence.requested_version != requested_version {
        return Err(format!(
            "local evidence version mismatch: expected={requested_version} actual={}",
            evidence.requested_version
        ));
    }
    if evidence.head != head || evidence.tree != tree {
        return Err(format!(
            "local evidence source mismatch: expected head/tree={head}/{tree}; evidence={}/{}",
            evidence.head, evidence.tree
        ));
    }
    if evidence.artifact_source_head.len() != 40
        || !evidence
            .artifact_source_head
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("local evidence artifact_source_head is not a 40-hex commit".to_owned());
    }
    Ok(())
}

fn verify_gate_receipts(gates: &BTreeMap<String, GateReceipt>) -> Result<Vec<String>, String> {
    let required = REQUIRED_LOCAL_GATES
        .iter()
        .map(|gate| (*gate).to_owned())
        .collect::<BTreeSet<_>>();
    let actual = gates.keys().cloned().collect::<BTreeSet<_>>();

    let missing = required.difference(&actual).cloned().collect::<Vec<_>>();
    let extra = actual.difference(&required).cloned().collect::<Vec<_>>();
    if !missing.is_empty() || !extra.is_empty() {
        return Err(format!(
            "gate receipt set mismatch: missing=[{}] extra=[{}]",
            missing.join(","),
            extra.join(",")
        ));
    }

    for gate in REQUIRED_LOCAL_GATES {
        let receipt = gates
            .get(*gate)
            .ok_or_else(|| format!("missing gate receipt {gate}"))?;
        if receipt.status != "PASS" {
            return Err(format!(
                "gate {gate} is not PASS: status={:?}",
                receipt.status
            ));
        }
        if receipt.evidence.trim().is_empty() {
            return Err(format!("gate {gate} has empty evidence description"));
        }
    }

    Ok(REQUIRED_LOCAL_GATES
        .iter()
        .map(|gate| (*gate).to_owned())
        .collect())
}

fn verify_state_snapshot(
    state: &Value,
    expected_generation: u64,
    head: &str,
    tree: &str,
) -> Result<(), String> {
    require_state_str(state, "task_id", TASK_ID)?;
    require_state_str(state, "head", head)?;
    require_state_str(state, "tree", tree)?;
    let generation = state
        .get("generation")
        .and_then(Value::as_u64)
        .ok_or_else(|| "state snapshot generation missing".to_owned())?;
    if generation != expected_generation {
        return Err(format!(
            "state snapshot generation mismatch: expected={expected_generation} actual={generation}"
        ));
    }

    for (field, expected) in [
        ("owned_programming_implementation", "100_PERCENT_RUST"),
        ("zero_touch_e2e", "PASS_5_OF_5"),
        (
            "explorer_transaction_semantic_correctness",
            "PASS_TARGETED_FIXTURES",
        ),
        (
            "explorer_hermetic_http_faults",
            "PASS_429_404_MALFORMED_CURSOR_RETRY_BOUNDS",
        ),
        ("explorer_storage_fault_matrix", "PASS"),
        ("explorer_large_data_1m", "PASS"),
        ("lifecycle_stress_native_event_r3", "VERIFIED_SUCCESS"),
        ("real_node_bridge_canary", "PASS"),
        ("real_explorer_canary", "PASS"),
        ("explorer_ui_responsiveness", "PASS"),
        ("op26_local_production_rc_r3", "VERIFIED_SUCCESS"),
        ("op27_installed_nsis_e2e_r3", "VERIFIED_SUCCESS"),
        ("op28_upgrade_acceptance", "VERIFIED_SUCCESS"),
        ("op29_uninstall_reinstall", "VERIFIED_SUCCESS"),
        ("flaky_release_tests_zero_proof", "VERIFIED_SUCCESS"),
        ("op31_final_security_and_sbom_precheck", "VERIFIED_SUCCESS"),
    ] {
        require_state_str(state, field, expected)?;
    }

    let invalidated = state
        .get("op31_prior_runtime_acceptance_invalidated")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            "state snapshot op31_prior_runtime_acceptance_invalidated missing".to_owned()
        })?;
    if invalidated {
        return Err("state snapshot says prior runtime acceptance was invalidated".to_owned());
    }

    Ok(())
}

fn require_state_str(state: &Value, field: &str, expected: &str) -> Result<(), String> {
    let actual = state
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("state snapshot missing string field {field}"))?;
    if actual != expected {
        return Err(format!(
            "state snapshot field mismatch: {field} expected={expected:?} actual={actual:?}"
        ));
    }
    Ok(())
}

fn verify_post_artifact_changes(
    root: &Path,
    artifact_source_head: &str,
    allowed_paths: &[String],
) -> Result<Vec<String>, String> {
    git_stdout(
        root,
        &[
            "cat-file",
            "-e",
            &format!("{artifact_source_head}^{{commit}}"),
        ],
    )?;

    let range = format!("{artifact_source_head}..HEAD");
    let actual_text = git_stdout(root, &["diff", "--name-only", &range])?;
    let actual = actual_text
        .lines()
        .map(|line| line.trim().replace('\\', "/"))
        .filter(|line| !line.is_empty())
        .collect::<BTreeSet<_>>();
    let allowed = allowed_paths
        .iter()
        .map(|path| path.trim().replace('\\', "/"))
        .filter(|path| !path.is_empty())
        .collect::<BTreeSet<_>>();

    if actual != allowed {
        let unexpected = actual.difference(&allowed).cloned().collect::<Vec<_>>();
        let missing = allowed.difference(&actual).cloned().collect::<Vec<_>>();
        return Err(format!(
            "post-artifact change-set mismatch: unexpected=[{}] receipt_only=[{}]",
            unexpected.join(","),
            missing.join(",")
        ));
    }

    for path in &actual {
        if !is_non_product_post_artifact_path(path) {
            return Err(format!(
                "product-source change after qualified artifact is forbidden: {path}"
            ));
        }
    }

    Ok(actual.into_iter().collect())
}

fn is_non_product_post_artifact_path(path: &str) -> bool {
    path == "SECURITY_ADVISORIES.md"
        || path.starts_with("docs/security/")
        || path.starts_with("e2e/")
        || path == "xtask/Cargo.toml"
        || path.starts_with("xtask/src/")
}

fn verify_windows_artifact(
    artifact_root: &Path,
    requested_version: &str,
    artifact_source_head: &str,
) -> Result<WindowsArtifactReport, String> {
    let sbom_path = artifact_root.join("WINDOWS_SBOM.spdx.json");
    let sbom: Value = serde_json::from_slice(
        &fs::read(&sbom_path).map_err(|error| format!("read {}: {error}", sbom_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", sbom_path.display()))?;

    let spdx_version = sbom
        .get("spdxVersion")
        .and_then(Value::as_str)
        .ok_or_else(|| "Windows SBOM missing spdxVersion".to_owned())?;
    if spdx_version != "SPDX-2.3" {
        return Err(format!("Windows SBOM SPDX mismatch: {spdx_version}"));
    }
    let packages = sbom
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "Windows SBOM missing packages array".to_owned())?;
    if packages.len() != 2385 {
        return Err(format!(
            "Windows SBOM package count drift: expected=2385 actual={}",
            packages.len()
        ));
    }
    let desktop_version_matches = packages
        .iter()
        .filter(|package| {
            package.get("name").and_then(Value::as_str) == Some("kaspa-gateway-desktop")
                && package.get("versionInfo").and_then(Value::as_str) == Some(requested_version)
        })
        .count();
    if desktop_version_matches == 0 {
        return Err(format!(
            "Windows SBOM missing kaspa-gateway-desktop {requested_version}"
        ));
    }

    let sums_path = artifact_root.join("SHA256SUMS");
    let sums_text = fs::read_to_string(&sums_path)
        .map_err(|error| format!("read {}: {error}", sums_path.display()))?;
    let mut declared = BTreeMap::<String, String>::new();
    for line in sums_text.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let digest = parts
            .next()
            .ok_or_else(|| format!("malformed SHA256SUMS line: {line}"))?;
        let name = parts
            .next()
            .ok_or_else(|| format!("malformed SHA256SUMS line: {line}"))?;
        if parts.next().is_some()
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!("malformed SHA256SUMS line: {line}"));
        }
        if declared
            .insert(name.to_owned(), digest.to_ascii_lowercase())
            .is_some()
        {
            return Err(format!("duplicate SHA256SUMS entry: {name}"));
        }
    }
    let expected_names = [WINDOWS_NSIS, WINDOWS_RAW_EXE]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let declared_names = declared.keys().cloned().collect::<BTreeSet<_>>();
    if declared_names != expected_names {
        return Err(format!(
            "SHA256SUMS exact entry set mismatch: {:?}",
            declared_names
        ));
    }

    let nsis_sha256 = sha256_file(&artifact_root.join(WINDOWS_NSIS))?;
    let raw_exe_sha256 = sha256_file(&artifact_root.join(WINDOWS_RAW_EXE))?;
    if declared.get(WINDOWS_NSIS) != Some(&nsis_sha256) {
        return Err("NSIS SHA256SUMS binding mismatch".to_owned());
    }
    if declared.get(WINDOWS_RAW_EXE) != Some(&raw_exe_sha256) {
        return Err("raw EXE SHA256SUMS binding mismatch".to_owned());
    }

    let smoke_path = artifact_root.join("WINDOWS_INSTALLER_SMOKE.txt");
    let smoke_text = fs::read_to_string(&smoke_path)
        .map_err(|error| format!("read {}: {error}", smoke_path.display()))?;
    let mut smoke = BTreeMap::<String, String>::new();
    for line in smoke_text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            smoke.insert(key.to_owned(), value.to_owned());
        }
    }
    let requested_commit_sha = smoke_value(&smoke, "REQUESTED_COMMIT_SHA")?;
    if requested_commit_sha != artifact_source_head {
        return Err(format!(
            "Windows artifact source mismatch: expected={artifact_source_head} actual={requested_commit_sha}"
        ));
    }
    let installer_smoke = smoke_value(&smoke, "WINDOWS_INSTALLER_SMOKE")?;
    let uninstall_smoke = smoke_value(&smoke, "WINDOWS_UNINSTALL_SMOKE")?;
    if installer_smoke != "PASS" || uninstall_smoke != "PASS" {
        return Err(format!(
            "Windows installer smoke failed: install={installer_smoke} uninstall={uninstall_smoke}"
        ));
    }
    let installed_exe_sha256 = smoke_value(&smoke, "WINDOWS_INSTALLED_EXE_SHA256")?;
    if installed_exe_sha256.len() != 64
        || !installed_exe_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("WINDOWS_INSTALLED_EXE_SHA256 is not a 64-hex digest".to_owned());
    }
    let authenticode_status = smoke
        .get("WINDOWS_AUTHENTICODE_STATUS")
        .cloned()
        .unwrap_or_else(|| "UNKNOWN".to_owned());

    Ok(WindowsArtifactReport {
        spdx_version: spdx_version.to_owned(),
        package_count: packages.len(),
        desktop_version_matches,
        nsis_sha256,
        raw_exe_sha256,
        installed_exe_sha256,
        requested_commit_sha,
        installer_smoke,
        uninstall_smoke,
        authenticode_status,
    })
}

fn smoke_value(smoke: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    smoke
        .get(key)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| format!("Windows installer smoke marker missing {key}"))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn resolve_inside_repository(root: &Path, path: &Path, label: &str) -> Result<PathBuf, String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize repository {}: {error}", root.display()))?;
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("{label} {} is unavailable: {error}", candidate.display()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "{label} escapes repository: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn git_identity(root: &Path) -> Result<(String, String), String> {
    Ok((
        git_stdout(root, &["rev-parse", "HEAD"])?,
        git_stdout(root, &["rev-parse", "HEAD^{tree}"])?,
    ))
}

fn git_success(root: &Path, args: &[&str], label: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("launch git for {label}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "source integrity failed ({label}); git {} exit={:?}; stderr={}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

fn git_stdout(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("release-admission: launch git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "release-admission: git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(version: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(root.path().join("apps/kaspa-gateway-desktop/src-tauri")).expect("dirs");
        fs::write(root.path().join("Cargo.toml"), "[workspace]\nmembers=[]\n").expect("workspace");
        fs::write(
            root.path().join("apps/kaspa-gateway-desktop/package.json"),
            format!(r#"{{"version":"{version}"}}"#),
        )
        .expect("package");
        fs::write(
            root.path()
                .join("apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json"),
            format!(r#"{{"version":"{version}"}}"#),
        )
        .expect("tauri");
        fs::write(
            root.path()
                .join("apps/kaspa-gateway-desktop/src-tauri/Cargo.toml"),
            format!("[package]\nname=\"desktop\"\nversion=\"{version}\"\n"),
        )
        .expect("cargo");
        root
    }

    fn complete_gate_map() -> BTreeMap<String, GateReceipt> {
        REQUIRED_LOCAL_GATES
            .iter()
            .map(|gate| {
                (
                    (*gate).to_owned(),
                    GateReceipt {
                        status: "PASS".to_owned(),
                        evidence: format!("evidence for {gate}"),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn parser_requires_local_mode_version_and_defaults_evidence() {
        let root = fixture("0.1.5");
        let mut good = vec![
            "local".to_owned(),
            "--version".to_owned(),
            "0.1.5".to_owned(),
            "--repository".to_owned(),
            root.path().to_string_lossy().into_owned(),
        ]
        .into_iter();
        let options = parse_args(&mut good, root.path()).expect("parse");
        assert_eq!(options.version, "0.1.5");
        assert_eq!(
            options.evidence,
            PathBuf::from("artifacts")
                .join("release-admission")
                .join("FINAL_LOCAL_RECEIPT.json")
        );

        let mut missing = vec!["local".to_owned()].into_iter();
        assert!(parse_args(&mut missing, root.path()).is_err());

        let mut wrong_mode = vec![
            "remote".to_owned(),
            "--version".to_owned(),
            "0.1.5".to_owned(),
        ]
        .into_iter();
        assert!(parse_args(&mut wrong_mode, root.path()).is_err());
    }

    #[test]
    fn stable_version_parser_is_fail_closed() {
        for good in ["0.1.5", "1.0.0", "12.34.56"] {
            validate_release_version(good).expect(good);
        }
        for bad in ["", "0.1", "v0.1.5", "0.1.5-rc.1", "01.1.5", "0.01.5"] {
            assert!(validate_release_version(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn version_contract_requires_all_three_surfaces() {
        let root = fixture("0.1.5");
        let value = verify_version_contract(root.path(), "0.1.5").expect("contract");
        assert_eq!(value.package_json, "0.1.5");
        assert_eq!(value.tauri_config, "0.1.5");
        assert_eq!(value.cargo_package, "0.1.5");

        fs::write(
            root.path().join("apps/kaspa-gateway-desktop/package.json"),
            r#"{"version":"0.1.4"}"#,
        )
        .expect("mismatch");
        assert!(verify_version_contract(root.path(), "0.1.5").is_err());
    }

    #[test]
    fn gate_receipt_requires_exact_complete_pass_set() {
        let gates = complete_gate_map();
        assert_eq!(
            verify_gate_receipts(&gates).expect("complete gates").len(),
            REQUIRED_LOCAL_GATES.len()
        );

        let mut missing = gates.clone();
        missing.remove("REAL_EXPLORER_CANARY");
        assert!(verify_gate_receipts(&missing).is_err());

        let mut failed = gates.clone();
        failed.get_mut("SECURITY").unwrap().status = "FAIL".to_owned();
        assert!(verify_gate_receipts(&failed).is_err());

        let mut empty_evidence = gates;
        empty_evidence
            .get_mut("FINAL_LOCAL_RECEIPT")
            .unwrap()
            .evidence
            .clear();
        assert!(verify_gate_receipts(&empty_evidence).is_err());
    }

    #[test]
    fn post_artifact_allowlist_rejects_product_source() {
        for good in [
            "SECURITY_ADVISORIES.md",
            "docs/security/npm-dependency-policy.json",
            "e2e/package-lock.json",
            "xtask/Cargo.toml",
            "xtask/src/release_admission.rs",
        ] {
            assert!(is_non_product_post_artifact_path(good), "{good}");
        }
        for bad in [
            "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
            "crates/kaspa-gateway-runtime/src/lib.rs",
            "apps/kaspa-gateway-desktop/frontend/src/main.js",
        ] {
            assert!(!is_non_product_post_artifact_path(bad), "{bad}");
        }
    }
}
