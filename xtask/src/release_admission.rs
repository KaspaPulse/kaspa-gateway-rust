use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    version: String,
    repository: PathBuf,
}

#[derive(Debug, Serialize)]
struct VersionContract {
    requested: String,
    package_json: String,
    tauri_config: String,
    cargo_package: String,
}

#[derive(Debug, Serialize)]
struct FoundationReport {
    schema_version: u32,
    mode: &'static str,
    requested_version: String,
    head: String,
    tree: String,
    version_contract: VersionContract,
    required_gates: &'static [&'static str],
    implemented_orchestrator_gates: Vec<&'static str>,
    missing_orchestrator_gates: Vec<&'static str>,
    local_release_admission: &'static str,
    push_allowed: bool,
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<String, String> {
    let options = parse_args(args, default_root)?;
    let contract = verify_version_contract(&options.repository, &options.version)?;
    let (head, tree) = git_identity(&options.repository)?;

    let implemented = vec!["SOURCE_INTEGRITY"];
    let missing = REQUIRED_LOCAL_GATES
        .iter()
        .copied()
        .filter(|gate| !implemented.contains(gate))
        .collect::<Vec<_>>();

    let report = FoundationReport {
        schema_version: 1,
        mode: "local",
        requested_version: options.version,
        head,
        tree,
        version_contract: contract,
        required_gates: REQUIRED_LOCAL_GATES,
        implemented_orchestrator_gates: implemented,
        missing_orchestrator_gates: missing.clone(),
        local_release_admission: "FAIL",
        push_allowed: false,
    };

    let rendered = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("release-admission: serialize report: {error}"))?;

    if missing.is_empty() {
        return Ok(rendered);
    }

    Err(format!(
        "LOCAL_RELEASE_ADMISSION=FAIL; PUSH_ALLOWED=NO; canonical orchestrator is fail-closed until these gates have executable runners: {}\n{rendered}",
        missing.join(",")
    ))
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
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--version" if version.is_none() => {
                version = Some(next_value(args, "--version")?);
            }
            "--repository" if repository.is_none() => {
                repository = Some(PathBuf::from(next_value(args, "--repository")?));
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

    Ok(Options {
        version,
        repository,
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
                "LOCAL_RELEASE_ADMISSION=FAIL; PUSH_ALLOWED=NO; release-admission version mismatch: requested={requested}; {label}={actual}"
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

fn git_identity(root: &Path) -> Result<(String, String), String> {
    Ok((
        git_stdout(root, &["rev-parse", "HEAD"])?,
        git_stdout(root, &["rev-parse", "HEAD^{tree}"])?,
    ))
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

    #[test]
    fn parser_requires_local_mode_and_version() {
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
    fn foundation_is_fail_closed_until_every_runner_exists() {
        assert!(REQUIRED_LOCAL_GATES.contains(&"INSTALLED_BINARY_E2E"));
        assert!(REQUIRED_LOCAL_GATES.contains(&"REAL_EXPLORER_CANARY"));
        assert!(REQUIRED_LOCAL_GATES.contains(&"LIFECYCLE_STRESS"));
        assert!(REQUIRED_LOCAL_GATES.contains(&"FINAL_LOCAL_RECEIPT"));
    }
}
