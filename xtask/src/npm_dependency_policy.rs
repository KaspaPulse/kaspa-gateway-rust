use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use time::{Date, Month, OffsetDateTime};

const POLICY_PATH: &str = "docs/security/npm-dependency-policy.json";

#[derive(Debug)]
pub struct GateError {
    pub code: i32,
    pub message: String,
}

impl GateError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: format!("KGW npm dependency policy gate FAILED: {}", message.into()),
        }
    }

    fn failure(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: format!("KGW npm dependency policy gate FAILED: {}", message.into()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Deprecation {
    name: String,
    version: String,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>, root: &Path) -> Result<String, GateError> {
    let mut workspace = None;
    let mut ci_log = None;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--workspace" => {
                workspace = Some(
                    args.next()
                        .ok_or_else(|| GateError::usage("--workspace is required"))?,
                );
            }
            "--ci-log" => {
                ci_log = Some(
                    args.next()
                        .ok_or_else(|| GateError::usage("--ci-log is required"))?,
                );
            }
            _ => return Err(GateError::usage(format!("Unknown argument: {argument}"))),
        }
    }

    let workspace = workspace.ok_or_else(|| GateError::usage("--workspace is required"))?;
    let ci_log = ci_log.ok_or_else(|| GateError::usage("--ci-log is required"))?;
    run(root, &workspace, Path::new(&ci_log))
}
fn run(root: &Path, workspace: &str, ci_log: &Path) -> Result<String, GateError> {
    let policy = read_json(&root.join(POLICY_PATH))
        .map_err(|error| GateError::failure(format!("invalid policy JSON: {error}")))?;
    if policy.get("schema_version").and_then(Value::as_i64) != Some(1) {
        return Err(GateError::usage(format!(
            "unsupported schema {}",
            policy
                .get("schema_version")
                .map(Value::to_string)
                .unwrap_or_else(|| "MISSING".to_owned())
        )));
    }

    let workspace_policy = policy
        .get("workspaces")
        .and_then(Value::as_object)
        .and_then(|workspaces| workspaces.get(workspace))
        .ok_or_else(|| GateError::usage(format!("unknown workspace {workspace}")))?;

    let workspace_relative = workspace_policy
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| GateError::failure(format!("workspace {workspace} is missing path")))?;
    let workspace_dir = root.join(workspace_relative);
    let audit_output_path = workspace_dir.join("npm-audit.json");

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let audit = Command::new(npm)
        .args(["audit", "--json", "--audit-level=low"])
        .current_dir(&workspace_dir)
        .output()
        .map_err(|error| GateError::failure(format!("npm audit execution failed: {error}")))?;

    fs::write(&audit_output_path, &audit.stdout)
        .map_err(|error| GateError::failure(format!("cannot write npm audit output: {error}")))?;

    let status = audit.status.code().unwrap_or(-1);
    if status != 0 && status != 1 {
        let stderr = String::from_utf8_lossy(&audit.stderr);
        return Err(GateError::failure(format!(
            "npm audit operational status {status}.{}",
            if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(" {}", stderr.trim())
            }
        )));
    }

    let audit_json: Value = serde_json::from_slice(&audit.stdout)
        .map_err(|error| GateError::failure(format!("invalid npm audit JSON: {error}")))?;
    let lock = read_json(&workspace_dir.join("package-lock.json"))
        .map_err(|error| GateError::failure(format!("invalid package-lock.json: {error}")))?;

    let install_log_path = if ci_log.is_absolute() {
        ci_log.to_path_buf()
    } else {
        root.join(ci_log)
    };
    if !install_log_path.is_file() {
        return Err(GateError::failure(format!(
            "install log missing: {}",
            ci_log.display()
        )));
    }
    let install_log = fs::read_to_string(&install_log_path)
        .map_err(|error| GateError::failure(format!("install log read failed: {error}")))?;

    let failures = validate_policy_snapshot(
        workspace_policy,
        &audit_json,
        &lock,
        &install_log,
        OffsetDateTime::now_utc(),
    );

    if !failures.is_empty() {
        let mut message = format!("for {workspace}:");
        for failure in failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        return Err(GateError::failure(message));
    }

    fs::remove_file(&audit_output_path)
        .map_err(|error| GateError::failure(format!("cannot remove npm audit output: {error}")))?;

    let counts = audit_json
        .pointer("/metadata/vulnerabilities")
        .and_then(Value::as_object)
        .ok_or_else(|| GateError::failure("npm audit JSON is missing metadata.vulnerabilities."))?;
    let deprecations = extract_deprecations(&install_log);
    Ok(format!(
        "KGW npm dependency policy gate PASSED: workspace={workspace}; critical={}; high={}; moderate={}; accepted_low={}; accepted_deprecations={}; review_until={}",
        count(counts, "critical"),
        count(counts, "high"),
        count(counts, "moderate"),
        count(counts, "low"),
        deprecations.len(),
        workspace_policy
            .get("review_until")
            .and_then(Value::as_str)
            .unwrap_or("NONE")
    ))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn count(map: &Map<String, Value>, key: &str) -> i64 {
    map.get(key).and_then(Value::as_i64).unwrap_or(0)
}
fn validate_policy_snapshot(
    workspace_policy: &Value,
    audit: &Value,
    lock: &Value,
    install_log: &str,
    now: OffsetDateTime,
) -> Vec<String> {
    let mut failures = Vec::new();
    let allowed = workspace_policy
        .get("allowed_vulnerabilities")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let allowed_deprecations = workspace_policy
        .get("allowed_deprecations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let allowed_by_name: BTreeMap<String, Value> = allowed
        .iter()
        .filter_map(|item| {
            item.get("name")
                .and_then(Value::as_str)
                .map(|name| (name.to_owned(), item.clone()))
        })
        .collect();

    if !allowed.is_empty() || !allowed_deprecations.is_empty() {
        match workspace_policy
            .get("review_until")
            .and_then(Value::as_str)
            .and_then(parse_review_expiry)
        {
            None => failures.push(
                "Accepted npm dependency risk is missing a valid review_until date.".to_owned(),
            ),
            Some(expiry) if now > expiry => {
                let review_until = workspace_policy
                    .get("review_until")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                failures.push(format!(
                    "Accepted npm dependency risk review expired on {review_until}."
                ));
            }
            Some(_) => {}
        }
    }

    let packages = lock
        .get("packages")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(required_versions) = workspace_policy
        .get("required_lock_versions")
        .and_then(Value::as_object)
    {
        for (lock_path, expected) in required_versions {
            let expected = expected.as_str().unwrap_or_default();
            let actual = packages
                .get(lock_path)
                .and_then(|item| item.get("version"))
                .and_then(Value::as_str);
            if actual != Some(expected) {
                failures.push(format!(
                    "Lockfile drift: {lock_path} expected {expected}, found {}.",
                    actual.unwrap_or("MISSING")
                ));
            }
        }
    }

    let vulnerabilities = audit
        .get("vulnerabilities")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    for (name, finding) in &vulnerabilities {
        let severity = finding
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let Some(expected) = allowed_by_name.get(name) else {
            failures.push(format!(
                "Unapproved npm vulnerability: {name} ({severity})."
            ));
            continue;
        };

        let expected_severity = expected
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if severity != expected_severity {
            failures.push(format!(
                "Severity drift for {name}: expected {expected_severity}, found {severity}."
            ));
        }

        let (via_packages, advisory_urls) = via_profile(finding.get("via"));
        let expected_via = string_array(expected.get("via_packages"));
        let expected_urls = string_array(expected.get("advisory_urls"));

        if via_packages != expected_via {
            failures.push(format!(
                "Dependency-path drift for {name}: via package set changed."
            ));
        }
        if advisory_urls != expected_urls {
            failures.push(format!(
                "Advisory identity drift for {name}: advisory URL set changed."
            ));
        }
    }

    for expected in &allowed {
        if let Some(name) = expected.get("name").and_then(Value::as_str)
            && !vulnerabilities.contains_key(name)
        {
            failures.push(format!(
                "Stale npm vulnerability exception: {name} is no longer reported."
            ));
        }
    }

    match audit
        .pointer("/metadata/vulnerabilities")
        .and_then(Value::as_object)
    {
        None => failures.push("npm audit JSON is missing metadata.vulnerabilities.".to_owned()),
        Some(metadata) => {
            for severity in ["critical", "high", "moderate"] {
                let value = count(metadata, severity);
                if value != 0 {
                    failures.push(format!(
                        "npm audit reports {value} {severity} vulnerabilities."
                    ));
                }
            }
            let expected_low = allowed
                .iter()
                .filter(|item| item.get("severity").and_then(Value::as_str) == Some("low"))
                .count() as i64;
            let actual_low = count(metadata, "low");
            if actual_low != expected_low {
                failures.push(format!(
                    "npm low-vulnerability count drift: expected {expected_low}, found {actual_low}."
                ));
            }
        }
    }

    let actual_deprecations: BTreeSet<String> = extract_deprecations(install_log)
        .into_iter()
        .map(|item| deprecation_key(&item))
        .collect();
    let expected_deprecations: BTreeSet<String> = allowed_deprecations
        .iter()
        .filter_map(deprecation_from_json)
        .map(|item| deprecation_key(&item))
        .collect();

    if actual_deprecations != expected_deprecations {
        failures.push(format!(
            "npm deprecation warning set drift: expected [{}], found [{}].",
            expected_deprecations
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", "),
            actual_deprecations
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    failures
}
fn via_profile(via: Option<&Value>) -> (Vec<String>, Vec<String>) {
    let mut via_packages = Vec::new();
    let mut advisory_urls = Vec::new();

    if let Some(items) = via.and_then(Value::as_array) {
        for item in items {
            if let Some(package) = item.as_str() {
                via_packages.push(package.to_owned());
            } else if let Some(url) = item.get("url").and_then(Value::as_str) {
                advisory_urls.push(url.to_owned());
            }
        }
    }

    via_packages.sort();
    advisory_urls.sort();
    (via_packages, advisory_urls)
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    let mut result: Vec<String> = value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect();
    result.sort();
    result
}

fn extract_deprecations(text: &str) -> Vec<Deprecation> {
    const PREFIX: &str = "npm warn deprecated ";
    let mut found = Vec::new();

    for line in text.lines() {
        let Some(rest) = line.strip_prefix(PREFIX) else {
            continue;
        };
        let Some(colon) = rest.find(':') else {
            continue;
        };
        let package_and_version = &rest[..colon];
        let Some((name, version)) = package_and_version.rsplit_once('@') else {
            continue;
        };
        if !name.is_empty() && !version.is_empty() {
            found.push(Deprecation {
                name: name.to_owned(),
                version: version.to_owned(),
            });
        }
    }

    found
}

fn deprecation_key(item: &Deprecation) -> String {
    format!("{}@{}", item.name, item.version)
}

fn deprecation_from_json(value: &Value) -> Option<Deprecation> {
    Some(Deprecation {
        name: value.get("name")?.as_str()?.to_owned(),
        version: value.get("version")?.as_str()?.to_owned(),
    })
}

fn parse_review_expiry(value: &str) -> Option<OffsetDateTime> {
    let mut parts = value.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    let month = Month::try_from(month).ok()?;
    let date = Date::from_calendar_date(year, month, day).ok()?;
    Some(date.with_hms(23, 59, 59).ok()?.assume_utc())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn now(value: &str) -> OffsetDateTime {
        match value {
            "2026-09-10T17:00:00Z" => Date::from_calendar_date(2026, Month::September, 10)
                .unwrap()
                .with_hms(17, 0, 0)
                .unwrap()
                .assume_utc(),
            "2026-10-11T00:00:00Z" => Date::from_calendar_date(2026, Month::October, 11)
                .unwrap()
                .with_hms(0, 0, 0)
                .unwrap()
                .assume_utc(),
            _ => panic!("unsupported test timestamp: {value}"),
        }
    }

    fn base_policy() -> Value {
        json!({
            "review_until": "2026-10-10",
            "allowed_vulnerabilities": [
                {
                    "name": "diff",
                    "severity": "low",
                    "via_packages": [],
                    "advisory_urls": ["https://github.com/advisories/GHSA-73rr-hh4g-fpgx"]
                },
                {
                    "name": "mocha",
                    "severity": "low",
                    "via_packages": ["diff"],
                    "advisory_urls": []
                },
                {
                    "name": "@wdio/mocha-framework",
                    "severity": "low",
                    "via_packages": ["mocha"],
                    "advisory_urls": []
                }
            ],
            "allowed_deprecations": [
                {"name": "glob", "version": "10.5.0"},
                {"name": "whatwg-encoding", "version": "3.1.1"}
            ],
            "required_lock_versions": {
                "node_modules/@wdio/mocha-framework": "9.31.7",
                "node_modules/mocha": "11.8.0",
                "node_modules/mocha/node_modules/diff": "7.0.0",
                "node_modules/js-yaml": "4.3.2",
                "node_modules/glob": "10.5.0",
                "node_modules/whatwg-encoding": "3.1.1"
            }
        })
    }

    fn base_audit() -> Value {
        json!({
            "vulnerabilities": {
                "diff": {
                    "severity": "low",
                    "via": [{"url": "https://github.com/advisories/GHSA-73rr-hh4g-fpgx"}]
                },
                "mocha": {"severity": "low", "via": ["diff"]},
                "@wdio/mocha-framework": {"severity": "low", "via": ["mocha"]}
            },
            "metadata": {
                "vulnerabilities": {
                    "info": 0,
                    "low": 3,
                    "moderate": 0,
                    "high": 0,
                    "critical": 0,
                    "total": 3
                }
            }
        })
    }

    fn base_lock() -> Value {
        json!({
            "packages": {
                "node_modules/@wdio/mocha-framework": {"version": "9.31.7"},
                "node_modules/mocha": {"version": "11.8.0"},
                "node_modules/mocha/node_modules/diff": {"version": "7.0.0"},
                "node_modules/js-yaml": {"version": "4.3.2"},
                "node_modules/glob": {"version": "10.5.0"},
                "node_modules/whatwg-encoding": {"version": "3.1.1"}
            }
        })
    }

    fn base_log() -> &'static str {
        "npm warn deprecated glob@10.5.0: upstream constraint\n\
npm warn deprecated whatwg-encoding@3.1.1: upstream constraint"
    }

    fn validate(policy: &Value, audit: &Value, lock: &Value, log: &str, at: &str) -> Vec<String> {
        validate_policy_snapshot(policy, audit, lock, log, now(at))
    }

    #[test]
    fn baseline_accepted_risk_snapshot_passes() {
        assert!(
            validate(
                &base_policy(),
                &base_audit(),
                &base_lock(),
                base_log(),
                "2026-09-10T17:00:00Z"
            )
            .is_empty()
        );
    }

    #[test]
    fn new_high_vulnerability_fails_closed() {
        let mut audit = base_audit();
        audit["vulnerabilities"]["injected-high"] = json!({"severity": "high", "via": []});
        audit["metadata"]["vulnerabilities"]["high"] = json!(1);
        audit["metadata"]["vulnerabilities"]["total"] = json!(4);
        let failures = validate(
            &base_policy(),
            &audit,
            &base_lock(),
            base_log(),
            "2026-09-10T17:00:00Z",
        );
        assert!(
            failures
                .iter()
                .any(|item| item.contains("Unapproved npm vulnerability: injected-high"))
        );
    }

    #[test]
    fn new_low_vulnerability_fails_closed() {
        let mut audit = base_audit();
        audit["vulnerabilities"]["injected-low"] = json!({"severity": "low", "via": []});
        audit["metadata"]["vulnerabilities"]["low"] = json!(4);
        audit["metadata"]["vulnerabilities"]["total"] = json!(4);
        let failures = validate(
            &base_policy(),
            &audit,
            &base_lock(),
            base_log(),
            "2026-09-10T17:00:00Z",
        );
        assert!(
            failures
                .iter()
                .any(|item| item.contains("Unapproved npm vulnerability: injected-low"))
        );
    }
    #[test]
    fn expired_exception_review_fails_closed() {
        let failures = validate(
            &base_policy(),
            &base_audit(),
            &base_lock(),
            base_log(),
            "2026-10-11T00:00:00Z",
        );
        assert!(failures.iter().any(|item| item.contains("review expired")));
    }

    #[test]
    fn new_deprecation_warning_fails_closed() {
        let log = format!(
            "{}\nnpm warn deprecated surprise-package@1.0.0: new warning",
            base_log()
        );
        let failures = validate(
            &base_policy(),
            &base_audit(),
            &base_lock(),
            &log,
            "2026-09-10T17:00:00Z",
        );
        assert!(
            failures
                .iter()
                .any(|item| item.contains("deprecation warning set drift"))
        );
    }

    #[test]
    fn lockfile_drift_fails_closed() {
        let mut lock = base_lock();
        lock["packages"]["node_modules/js-yaml"]["version"] = json!("4.3.1");
        let failures = validate(
            &base_policy(),
            &base_audit(),
            &lock,
            base_log(),
            "2026-09-10T17:00:00Z",
        );
        assert!(
            failures
                .iter()
                .any(|item| item.contains("node_modules/js-yaml expected 4.3.2"))
        );
    }

    #[test]
    fn stale_vulnerability_exception_fails_closed() {
        let mut audit = base_audit();
        audit["vulnerabilities"]
            .as_object_mut()
            .unwrap()
            .remove("diff");
        audit["metadata"]["vulnerabilities"]["low"] = json!(2);
        audit["metadata"]["vulnerabilities"]["total"] = json!(2);
        let failures = validate(
            &base_policy(),
            &audit,
            &base_lock(),
            base_log(),
            "2026-09-10T17:00:00Z",
        );
        assert!(
            failures
                .iter()
                .any(|item| item.contains("Stale npm vulnerability exception: diff"))
        );
    }

    #[test]
    fn scoped_deprecation_names_use_last_at_separator() {
        let found =
            extract_deprecations("npm warn deprecated @scope/pkg@1.2.3: old\nnot a deprecation");
        assert_eq!(
            found,
            vec![Deprecation {
                name: "@scope/pkg".to_owned(),
                version: "1.2.3".to_owned(),
            }]
        );
    }
}
