use super::*;
use std::fs;

const NOW: &str = "2026-09-20T15:00:00+00:00";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repository root")
        .to_path_buf()
}

fn copy_file(source: &Path, destination: &Path) {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::copy(source, destination).unwrap();
}

fn temp_policy_root(files: &[&str]) -> TempDir {
    let root = repo_root();
    let temp = tempdir().unwrap();
    for relative in files {
        copy_file(
            &ksss_dir(&root).join(relative),
            &ksss_dir(temp.path()).join(relative),
        );
    }
    temp
}

fn signed_bundle() -> RuntimeBundle {
    extract_runtime(&repo_root()).expect("signed runtime")
}

#[test]
fn test_offline_self_contained_check() {
    let result = check(&repo_root(), NOW).expect("offline KSSS check");
    assert_eq!(result["adoption_status"], "PASS");
    assert_eq!(result["external_ksss_checkout_required"], false);
    assert_eq!(result["application_runtime_status"], "NOT_VERIFIED");
    assert_eq!(result["release_qualification"], "NOT_VERIFIED");
}

#[test]
fn test_mixed_changes_preserve_every_official_trigger() {
    let bundle = signed_bundle();
    let kinds = with_api(&bundle, |py, api| {
        classify(
            py,
            api,
            &BTreeSet::from([
                "crates/kaspa-gateway-runtime/src/lib.rs".to_owned(),
                "Cargo.lock".to_owned(),
                ".github/workflows/ci.yml".to_owned(),
                ".security/ksss/trust-policy.json".to_owned(),
            ]),
        )
    })
    .unwrap();
    assert_eq!(
        kinds,
        vec![
            "CODE_ONLY_CHANGE",
            "DEPENDENCY_CHANGE",
            "POLICY_CHANGE",
            "WORKFLOW_CHANGE",
        ]
    );
}
#[test]
fn test_risk_floor_cannot_be_lowered_by_profile() {
    let temp = temp_policy_root(&[
        "repository-policy.yaml",
        "applicability-v1.json",
        "local-strengthening.json",
    ]);
    let path = ksss_dir(temp.path()).join("repository-policy.yaml");
    let mut policy = load_json(&path, "POLICY").unwrap();
    policy["audit_profile"] = json!("light");
    policy["controls"]["additional_required"] = json!([]);
    write_json(&path, &policy).unwrap();

    let bundle = signed_bundle();
    let error = with_api(&bundle, |py, api| resolve(py, api, temp.path(), NOW)).unwrap_err();
    assert!(error.contains("RISK_FLOOR_WEAKENED"), "{error}");
}

#[test]
fn test_signed_archive_pin_rejects_modified_bytes() {
    let temp = temp_policy_root(&["trust-policy.json"]);
    let policy = load_policy(temp.path()).unwrap();
    let evidence = ksss_dir(temp.path())
        .join("trust/evidence")
        .join(string(&policy, "ksss_release").unwrap());
    fs::create_dir_all(&evidence).unwrap();
    fs::write(
        evidence.join(string(&policy, "runtime_artifact_filename").unwrap()),
        b"modified",
    )
    .unwrap();
    let error = verify_artifact_pin(temp.path()).unwrap_err();
    assert!(error.contains("SHA256_MISMATCH"), "{error}");
}

#[test]
fn test_runtime_rollback_floor_rejected() {
    let temp = temp_policy_root(&["trust-policy.json"]);
    let path = ksss_dir(temp.path()).join("trust-policy.json");
    let mut policy = load_json(&path, "POLICY").unwrap();
    policy["runtime_sequence"] = json!(1);
    write_json(&path, &policy).unwrap();
    let error = verify_artifact_pin(temp.path()).unwrap_err();
    assert!(error.contains("BELOW_FLOOR"), "{error}");
}

#[test]
fn test_archive_paths_fail_closed() {
    for raw in [
        "../bad",
        "ksss-consumer-runtime-v1/../bad",
        "ksss-consumer-runtime-v1/",
    ] {
        assert!(safe_relative_member(raw).is_err(), "{raw}");
    }
}
#[test]
fn test_learning_rejects_stale_and_wrong_context() {
    let root = repo_root();
    let record = load_json(
        &ksss_dir(&root).join("learning/known-failures/KGW-OWNER-TERMINAL-PID.json"),
        "RECORD",
    )
    .unwrap();
    let bundle = signed_bundle();
    with_api(&bundle, |py, api| {
        let matched = api_one_kwargs_json(
            py,
            api,
            "lookup_known_failures",
            &json!([record.clone()]),
            &[
                (
                    "fingerprint",
                    record.get("failure_fingerprint").cloned().unwrap(),
                ),
                ("context", record.get("applies_when").cloned().unwrap()),
                ("active_change_types", json!([])),
            ],
        )?;
        assert_eq!(matched.as_array().unwrap().len(), 1);

        let mut wrong = record.get("applies_when").cloned().unwrap();
        wrong["platform"] = json!("linux");
        let no_match = api_one_kwargs_json(
            py,
            api,
            "lookup_known_failures",
            &json!([record.clone()]),
            &[
                (
                    "fingerprint",
                    record.get("failure_fingerprint").cloned().unwrap(),
                ),
                ("context", wrong),
                ("active_change_types", json!([])),
            ],
        )?;
        assert!(no_match.as_array().unwrap().is_empty());
        assert!(!api_record_reusable(
            py,
            api,
            &record,
            record.get("applies_when").unwrap(),
            &json!(["CODE_ONLY_CHANGE"]),
        )?);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_reference_parity_rejects_different_signed_runtime() {
    let temp = temp_policy_root(&["trust-policy.json", "reference-parity.json"]);
    let path = ksss_dir(temp.path()).join("reference-parity.json");
    let mut proof = load_json(&path, "PARITY").unwrap();
    proof["runtime_artifact_sha256"] = json!("f".repeat(64));
    write_json(&path, &proof).unwrap();
    let bundle = signed_bundle();
    let error = reference_parity(temp.path(), &bundle.manifest).unwrap_err();
    assert!(error.contains("PARITY_IDENTITY"), "{error}");
}

#[test]
fn test_threat_model_rejects_missing_surface_inventory() {
    let root = repo_root();
    let mut threat = load_json(&ksss_dir(&root).join("threat-model.json"), "THREAT").unwrap();
    threat.as_object_mut().unwrap().remove("surfaces");
    let bundle = signed_bundle();
    let errors = with_api(&bundle, |py, api| {
        let schema = api_load_document(api, "schemas/threat-model.schema.json")?;
        api_validate_json_schema(py, api, &threat, &schema)
    })
    .unwrap();
    assert!(!errors.is_empty());
}
#[test]
fn test_knowledge_lookup_reads_existing_evidence_without_reexecution() {
    let root = repo_root();
    let record = load_json(
        &ksss_dir(&root).join("learning/known-failures/KGW-OWNER-TERMINAL-PID.json"),
        "RECORD",
    )
    .unwrap();
    let temp = tempdir().unwrap();
    let query = temp.path().join("query.json");
    write_json(
        &query,
        &json!({
            "context": record["applies_when"],
            "failure_fingerprint": record["failure_fingerprint"]
        }),
    )
    .unwrap();
    let result = knowledge_lookup(&root, &query).unwrap();
    assert_eq!(result["PREVIOUS_KNOWLEDGE_SEARCH"], "PASS");
    assert_eq!(result["match_status"], "MATCH");
    assert_eq!(result["historical_tests_reexecuted"], false);
}

#[test]
fn test_diag_order() {
    let bundle = signed_bundle();
    with_api(&bundle, |py, api| {
        let wrong = api_one_json(
            py,
            api,
            "validate_diagnosis_sequence",
            &json!(["DIAGNOSIS"]),
        );
        assert!(wrong.is_err());
        let ok = api_one_kwargs_json(
            py,
            api,
            "validate_diagnosis_sequence",
            &json!([
                "CONTAIN_OR_RESTORE",
                "PREVIOUS_KNOWLEDGE_SEARCH=PASS",
                "DIAGNOSIS"
            ]),
            &[("high_severity_production_incident", json!(true))],
        )?;
        assert_eq!(ok, json!("PASS"));
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_learning_cannot_close_without_protection() {
    let bundle = signed_bundle();
    let error = with_api(&bundle, |py, api| {
        api_kwargs_json(
            py,
            api,
            "learning_closeout",
            &[
                ("service_status", json!("RESOLVED")),
                ("learning_status", json!("CLOSED")),
                ("durable_protections", json!([])),
            ],
        )
    })
    .unwrap_err();
    assert!(!error.is_empty());
}

struct ReceiptFixture {
    _temp: TempDir,
    root: PathBuf,
    artifact: PathBuf,
    evidence: PathBuf,
    receipt_path: PathBuf,
    receipt: Value,
    current: Value,
}

impl ReceiptFixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let root = temp.path().to_path_buf();
        let artifact = root.join("candidate.exe");
        fs::write(&artifact, b"SYNTHETIC TEST ARTIFACT").unwrap();
        let evidence = root.join("evidence");
        fs::create_dir_all(&evidence).unwrap();
        let proof = evidence.join("proof.json");
        fs::write(&proof, b"{\"synthetic_contract_test_only\":true}").unwrap();

        let repository = repo_root();
        let contract = load_json(
            &ksss_dir(&repository).join("runtime-contract.json"),
            "RUNTIME_CONTRACT",
        )
        .unwrap();
        let current = json!({
            "source_sha": "a".repeat(40),
            "tree_hash": "b".repeat(64),
            "worktree_state": {"fixture": "synthetic"}
        });
        let required_checks = array_strings(&contract, "required_checks").unwrap();
        let cases = array_strings(&contract, "cases")
            .unwrap()
            .into_iter()
            .map(|case_id| {
                let checks: Map<String, Value> = required_checks
                    .iter()
                    .map(|name| {
                        (
                            name.clone(),
                            json!({
                                "status": "PASS",
                                "evidence_path": "proof.json",
                                "evidence_sha256": sha256_file(&proof).unwrap()
                            }),
                        )
                    })
                    .collect();
                json!({
                    "case_id": case_id,
                    "owner_identity": {
                        "pid": 1234,
                        "start_time": 123456,
                        "executable": "C:/fixture/candidate.exe"
                    },
                    "checks": checks
                })
            })
            .collect::<Vec<_>>();
        let receipt = json!({
            "schema": "kgw-runtime-qualification-receipt-v1",
            "source_sha": current["source_sha"],
            "tree_hash": current["tree_hash"],
            "worktree_state": current["worktree_state"],
            "artifact_sha256": sha256_file(&artifact).unwrap(),
            "environment_fingerprint": "c".repeat(64),
            "host_os": "windows",
            "test_kind": "real-packaged-application",
            "cases": cases
        });
        let receipt_path = root.join("receipt.json");
        write_json(&receipt_path, &receipt).unwrap();
        Self {
            _temp: temp,
            root: repository,
            artifact,
            evidence,
            receipt_path,
            receipt,
            current,
        }
    }

    fn sync(&self) {
        write_json(&self.receipt_path, &self.receipt).unwrap();
    }

    fn verify(&self) -> Result<Value, String> {
        self.sync();
        release_check_with_identity(
            &self.root,
            &self.receipt_path,
            &self.artifact,
            &self.evidence,
            &"c".repeat(64),
            self.current.clone(),
        )
    }
}
#[test]
fn test_complete_synthetic_receipt_is_only_receipt_validation() {
    let fixture = ReceiptFixture::new();
    let result = fixture.verify().unwrap();
    assert_eq!(result["receipt_validation"], "PASS");
    assert_eq!(result["application_executed_by_this_command"], false);
}

#[test]
fn test_missing_case_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["cases"].as_array_mut().unwrap().pop();
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("CASES_INCOMPLETE"), "{error}");
}

#[test]
fn test_duplicate_case_rejected() {
    let mut fixture = ReceiptFixture::new();
    let cases = fixture.receipt["cases"].as_array_mut().unwrap();
    let first = cases[0].clone();
    let last = cases.len() - 1;
    cases[last] = first;
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("CASES_INCOMPLETE"), "{error}");
}

#[test]
fn test_ci_success_cannot_replace_runtime_proof() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["test_kind"] = json!("unit-tests");
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("REAL_PACKAGED_WINDOWS"), "{error}");
}

#[test]
fn test_stale_source_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["source_sha"] = json!("d".repeat(40));
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("IDENTITY_MISMATCH"), "{error}");
}

#[test]
fn test_different_artifact_rejected() {
    let fixture = ReceiptFixture::new();
    fs::write(&fixture.artifact, b"DIFFERENT").unwrap();
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("ARTIFACT_MISMATCH"), "{error}");
}
#[test]
fn test_different_environment_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["environment_fingerprint"] = json!("d".repeat(64));
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("ENVIRONMENT_MISMATCH"), "{error}");
}

#[test]
fn test_missing_owner_identity_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["cases"][0]["owner_identity"]
        .as_object_mut()
        .unwrap()
        .remove("start_time");
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("OWNER_IDENTITY"), "{error}");
}

#[test]
fn test_failed_check_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["cases"][0]["checks"]["ready_owned_process"]["status"] = json!("FAIL");
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("CHECK_NOT_PASS"), "{error}");
}

#[test]
fn test_missing_evidence_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["cases"][0]["checks"]["ready_owned_process"]["evidence_path"] =
        json!("missing.json");
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("EVIDENCE_PATH"), "{error}");
}

#[test]
fn test_evidence_tampering_rejected() {
    let fixture = ReceiptFixture::new();
    fs::write(fixture.evidence.join("proof.json"), b"changed").unwrap();
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("EVIDENCE_DIGEST"), "{error}");
}

#[test]
fn test_evidence_path_escape_rejected() {
    let mut fixture = ReceiptFixture::new();
    fixture.receipt["cases"][0]["checks"]["ready_owned_process"]["evidence_path"] =
        json!("../candidate.exe");
    let error = fixture.verify().unwrap_err();
    assert!(error.contains("EVIDENCE_PATH"), "{error}");
}
