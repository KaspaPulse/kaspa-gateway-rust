use super::*;

pub(super) fn normalized_hashes(value: &Value) -> Vec<Value> {
    sequence(value).into_iter().filter(|entry| !entry.is_null()).map(|entry| json!({
        "stage":field(entry,&["stage"]),"network":field(entry,&["network"]),"runtime_role":field(entry,&["runtime_role","runtimeRole"]),
        "bridge_instance_id":field(entry,&["bridge_instance_id","bridgeInstanceId"]),"expected_sha256":field(entry,&["expected_sha256","expectedSha256"]),
        "result_sha256":field(entry,&["result_sha256","resultSha256"]),"result_source":field(entry,&["result_source","resultSource"]),
        "windows_clipboard_sha256":field(entry,&["windows_clipboard_sha256","windowsClipboardSha256"]),"raw_file":field(entry,&["raw_file","rawFile"])
    })).collect()
}
pub(super) fn normalized_pids(value: &Value) -> EvidenceResult<Vec<Value>> {
    let mut result = Vec::new();
    for entry in sequence(value) {
        if let Some(pid) = integer(property(entry, &["pid"]))? {
            result.push(json!({"network":field(entry,&["network"]),"runtime_role":field(entry,&["runtime_role","runtimeRole"]),
                "bridge_instance_id":field(entry,&["bridge_instance_id","bridgeInstanceId"]),"pid":pid}));
        }
    }
    Ok(result)
}
pub(super) fn normalized_ports(value: &Value) -> EvidenceResult<Vec<Value>> {
    let mut result = Vec::new();
    for entry in sequence(value) {
        if let Some(port) = integer(property(entry, &["port"]))? {
            let host = field(entry, &["host"]);
            result.push(json!({"network":field(entry,&["network"]),"runtime_role":field(entry,&["runtime_role","runtimeRole"]),
                "host":host,"port":port,"purpose":field(entry,&["purpose"]),"endpoint":format!("{host}:{port}")}));
        }
    }
    Ok(result)
}
fn matches_fields(actual: &Value, expected: &Value, fields: &[&str]) -> bool {
    fields
        .iter()
        .all(|key| text(&actual[*key]).eq_ignore_ascii_case(&text(&expected[*key])))
}
fn recovery_file_strings(value: &Value) -> Vec<String> {
    // PowerShell's parenthesized property pipeline casts an absent/null property
    // to one empty string; an explicitly empty array remains empty.
    if value.is_null() {
        vec![String::new()]
    } else {
        sequence(value).into_iter().map(text).collect()
    }
}
fn resolve_executable(repository: &Path, artifact: &Path, recorded: &str) -> PathBuf {
    let recorded = if recorded.trim().is_empty() {
        repository
            .join("target")
            .join("kgw-zero-touch-e2e")
            .join("debug")
            .join("kaspa-gateway-desktop.exe")
    } else {
        PathBuf::from(recorded)
    };
    if recorded.is_file() {
        return recorded;
    }
    let exported = artifact.join("export").join("kaspa-gateway-desktop.exe");
    if exported.is_file() {
        exported
    } else {
        recorded
    }
}

pub fn check(repository: &Path, artifact: &Path) -> EvidenceResult<Value> {
    let result = read_json(&artifact.join("zero-touch-result.json"))?;
    let mut errors = Vec::<String>::new();
    if result.is_null() {
        return Ok(
            json!({"passed":false,"errors":["Missing zero-touch-result.json."],"result":null,"evidence":null}),
        );
    }
    if !truthy(property(&result, &["completed"])) {
        errors.push("zero-touch-result.json completed is not true.".to_owned());
    }
    if !truthy(property(&result, &["success"])) {
        errors.push("zero-touch-result.json success is not true.".to_owned());
    }
    if decimal_i32(property(&result, &["exit_code"]))? != Some(0) {
        errors.push("zero-touch-result.json exit_code is not 0.".to_owned());
    }
    let wdio = reports::wdio(artifact)?;
    if !truthy(&wdio["passed"]) {
        append_errors(&mut errors, &wdio, "errors");
    }
    let policy = reports::policy(artifact)?;
    if !truthy(&policy["passed"]) {
        append_errors(&mut errors, &policy, "errors");
    }
    let commit = identity::commit(repository)?;
    let artifact_commit = field(&result, &["git_commit"]);
    if !artifact_commit.eq_ignore_ascii_case(&commit) {
        errors.push(format!(
            "Artifact git commit '{artifact_commit}' does not match current commit '{commit}'."
        ));
    }
    let diff = identity::source_diff(repository)?;
    let artifact_diff = field(&result, &["source_diff_sha256"]);
    if !artifact_diff.eq_ignore_ascii_case(&diff) {
        errors.push(format!(
            "Artifact source diff SHA256 '{artifact_diff}' does not match current '{diff}'."
        ));
    }
    let mut executable = field(&result, &["app_binary"]);
    if executable.trim().is_empty() {
        executable = field(
            &read_json(&artifact.join("zero-touch-script-summary.json"))?,
            &["app_binary"],
        );
    }
    let binary = resolve_executable(repository, artifact, &executable);
    if !binary.is_file() {
        errors.push(format!(
            "E2E desktop executable is missing: {}",
            binary.display()
        ));
    } else {
        let actual = identity::hash_file(&binary)?;
        let expected = field(&result, &["executable_sha256"]);
        if !actual.eq_ignore_ascii_case(&expected) {
            errors.push(format!(
                "Artifact executable SHA256 '{expected}' does not match current '{actual}'."
            ));
        }
    }
    let recorded = sequence(property(&result, &["passed_stages"]))
        .into_iter()
        .map(text)
        .collect::<Vec<_>>();
    for stage in stage::required_stages()? {
        if !recorded
            .iter()
            .any(|name| name.eq_ignore_ascii_case(stage.name))
        {
            errors.push(format!(
                "Artifact did not record required passed stage '{}'.",
                stage.name
            ));
        }
    }
    let evidence = stage::summary(artifact)?;
    if !truthy(&evidence["passed"]) {
        append_errors(&mut errors, &evidence, "validation_errors");
    }
    let recorded_hashes = normalized_hashes(property(&result, &["clipboard_hashes"]));
    for expected in normalized_hashes(&evidence["clipboard_hashes"]) {
        let keys = [
            "stage",
            "network",
            "runtime_role",
            "bridge_instance_id",
            "expected_sha256",
            "result_sha256",
            "windows_clipboard_sha256",
        ];
        if !recorded_hashes
            .iter()
            .any(|actual| matches_fields(actual, &expected, &keys))
        {
            errors.push(format!(
                "Artifact result did not record validated clipboard hash evidence for '{}'.",
                text(&expected["stage"])
            ));
        }
    }
    let recorded_pids = normalized_pids(property(&result, &["process_ids"]))?;
    for expected in normalized_pids(&evidence["process_ids"])? {
        if !recorded_pids.iter().any(|actual| {
            matches_fields(
                actual,
                &expected,
                &["network", "runtime_role", "bridge_instance_id", "pid"],
            )
        }) {
            errors.push(format!(
                "Artifact result did not record validated PID evidence for '{}/{}/{}'.",
                text(&expected["runtime_role"]),
                text(&expected["network"]),
                text(&expected["pid"])
            ));
        }
    }
    let recorded_ports = normalized_ports(property(&result, &["ports"]))?;
    for expected in normalized_ports(&evidence["ports"])? {
        if !recorded_ports.iter().any(|actual| {
            matches_fields(
                actual,
                &expected,
                &["network", "runtime_role", "host", "port"],
            )
        }) {
            errors.push(format!(
                "Artifact result did not record validated port evidence for '{}/{}/{}:{}'.",
                text(&expected["runtime_role"]),
                text(&expected["network"]),
                text(&expected["host"]),
                text(&expected["port"])
            ));
        }
    }
    if truthy(property(&result, &["recovered_from_completed_wdio_run"])) {
        let failure = field(&result, &["original_failure_stage"]);
        if !failure.eq_ignore_ascii_case("Zero-touch result writing") {
            errors.push(format!("Recovered artifact original_failure_stage was '{failure}', expected 'Zero-touch result writing'."));
        }
        if field(&result, &["recovery_timestamp"]).trim().is_empty() {
            errors.push("Recovered artifact did not record recovery_timestamp.".to_owned());
        }
        let files = recovery_file_strings(property(&result, &["recovery_evidence_files"]));
        if files.is_empty() {
            errors.push("Recovered artifact did not record recovery_evidence_files.".to_owned());
        }
        for path in files {
            if !Path::new(&path).is_file() {
                errors.push(format!(
                    "Recovered artifact references missing recovery evidence file: {path}"
                ));
            }
        }
    }
    Ok(
        json!({"passed":errors.is_empty(),"errors":errors,"result":result,"evidence":evidence,"wdio":wdio,"testnet13_policy":policy}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absent_recovery_files_preserve_null_pipeline_conversion() {
        assert_eq!(recovery_file_strings(&Value::Null), vec![String::new()]);
        assert!(recovery_file_strings(&json!([])).is_empty());
        assert_eq!(
            recovery_file_strings(&json!([null, "file"])),
            vec![String::new(), "file".to_owned()]
        );
    }

    #[test]
    fn evidence_record_aliases_normalize_without_losing_large_pids() {
        let records = normalized_pids(
            &json!([{ "runtimeRole":"node","pid":"9007199254740993"},null,{"pid":"bad"}]),
        )
        .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["pid"].as_i64(), Some(9007199254740993));
        assert_eq!(records[0]["runtime_role"], "node");
    }
    #[test]
    fn portable_export_is_used_only_when_recorded_binary_is_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact = temp.path().join("artifact");
        let export = artifact.join("export");
        fs::create_dir_all(&export).expect("export dir");
        let exported = export.join("kaspa-gateway-desktop.exe");
        fs::write(&exported, b"exported").expect("exported binary");
        let missing = temp.path().join("missing.exe");
        assert_eq!(
            resolve_executable(temp.path(), &artifact, &missing.to_string_lossy()),
            exported
        );

        let recorded = temp.path().join("recorded.exe");
        fs::write(&recorded, b"recorded").expect("recorded binary");
        assert_eq!(
            resolve_executable(temp.path(), &artifact, &recorded.to_string_lossy()),
            recorded
        );
    }

    #[test]
    fn hash_matching_requires_every_identity_field() {
        let expected = json!({"stage":"Mainnet Node","expected_sha256":"ab","runtime_role":"node"});
        assert!(matches_fields(
            &json!({"stage":"mainnet node","expected_sha256":"AB","runtime_role":"node"}),
            &expected,
            &["stage", "expected_sha256", "runtime_role"]
        ));
        assert!(!matches_fields(
            &json!({"stage":"mainnet node","expected_sha256":"AB","runtime_role":"bridge"}),
            &expected,
            &["stage", "expected_sha256", "runtime_role"]
        ));
    }
}
