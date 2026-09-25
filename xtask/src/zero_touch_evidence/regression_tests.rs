//! Durable regression fixtures for the evidence protocol, never runtime proof.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    marker: String,
}
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let marker = format!(
            "{}-{nonce}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        );
        let root = std::env::temp_dir().join(format!("kgw-evidence-regression-{marker}"));
        fs::create_dir(&root).unwrap();
        fs::write(root.join(".fixture-owner"), marker.as_bytes()).unwrap();
        Self { root, marker }
    }
    fn json(&self, relative: &str, value: &Value) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }
    fn bytes(&self, relative: &str, value: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, value).unwrap();
    }
    fn case(&self, stage: &stage::Stage, pid: i32) {
        let prefix = format!("cases/{}", stage.slug);
        let raw = "Synthetic regression evidence, not a native process log.\n";
        let sha = identity::hash_text(raw);
        self.json(
            &format!("{prefix}/{}", stage.status_file),
            &json!({"pid":pid}),
        );
        self.json(&format!("{prefix}/runtime-logs.json"), &json!({"entries":[{"network":stage.network,"runtime_role":stage.role,"raw_text":raw}]}));
        self.json(
            &format!("{prefix}/process-tree.json"),
            &json!([{"ProcessId":pid}]),
        );
        self.json(
            &format!("{prefix}/port-state.json"),
            &json!(
                stage
                    .ports
                    .iter()
                    .map(|port| json!({"LocalPort":port}))
                    .collect::<Vec<_>>()
            ),
        );
        self.json(&format!("{prefix}/clipboard-capture.json"), &json!({"event":{"network":stage.network,"runtime_role":stage.role,"bridge_instance_id":"fixture","expected_sha256":sha,"result_sha256":sha,"result_source":"fixture"},"clipboard":{"actual_sha256":sha}}));
        self.bytes(&format!("{prefix}/clipboard.raw.txt"), raw.as_bytes());
        // The legacy evidence contract checks file presence, not image decoding.
        self.bytes(
            &format!("{prefix}/current-screenshot.png"),
            b"synthetic presence fixture",
        );
    }
    fn policy(&self) {
        self.json("cases/testnet13-policy/testnet13-policy.json", &json!({"nodeEnabled":false,"bridgeEnabled":false,"startBlocked":{"ok":false,"error":"experimental opt-in policy"},"status":"running=false"}));
        for name in ["process-tree.json", "current-dom-state.json"] {
            self.json(&format!("cases/testnet13-policy/{name}"), &json!({}));
        }
        for name in ["current-screenshot.png", "current-page-source.html"] {
            self.bytes(
                &format!("cases/testnet13-policy/{name}"),
                b"synthetic fixture",
            );
        }
    }
    fn wdio(&self) {
        let names = [
            "Mainnet Node regression copies only real child raw lines",
            "Testnet10 Node copies isolated Testnet10 child raw lines",
            "Mainnet Bridge copies only bridge child raw lines",
            "Testnet10 Bridge copies isolated Testnet10 bridge child raw lines",
            "Testnet13 stays disabled by default and policy blocks zero-touch launch",
        ];
        self.json("json/wdio-0-0.json",&json!({"specs":["synthetic.e2e.js"],"state":{"passed":5,"failed":0,"skipped":0},"suites":[{"tests":names.iter().map(|name| json!({"name":name,"state":"passed"})).collect::<Vec<_>>()}]}));
        self.bytes("junit/wdio-0-0.xml",br#"<testsuites><testsuite tests="5" failures="0" errors="0" skipped="0"/></testsuites>"#);
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if fs::read(self.root.join(".fixture-owner")).ok().as_deref()
            == Some(self.marker.as_bytes())
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
fn errors_contain(report: &Value, message: &str) -> bool {
    sequence(property(report, &["errors", "validation_errors"]))
        .iter()
        .any(|error| text(error).contains(message))
}
#[test]
fn stage_fixture_accepts_only_complete_consistent_saved_records() {
    let fixture = Fixture::new();
    let stage = stage::required_stages().unwrap().remove(0);
    fixture.case(&stage, 731);
    let report = stage::stage_evidence(&fixture.root, &stage).unwrap();
    assert_eq!(report["passed"], true);
    assert_eq!(report["process_id"], 731);
    assert_eq!(report["ports"].as_array().unwrap().len(), 2);
    assert_eq!(
        report["clipboard_hash"]["expected_sha256"],
        report["clipboard_hash"]["windows_clipboard_sha256"]
    );
}
#[test]
fn missing_pid_and_wrong_network_do_not_inherit_previous_success() {
    let fixture = Fixture::new();
    let stage = stage::required_stages().unwrap().remove(0);
    fixture.case(&stage, 731);
    fixture.json("cases/mainnet-node/node-owner-status.json", &json!({}));
    fixture.json(
        "cases/mainnet-node/runtime-logs.json",
        &json!({"entries":[{"network":"testnet10","runtime_role":"node","raw_text":"fixture"}]}),
    );
    let report = stage::stage_evidence(&fixture.root, &stage).unwrap();
    assert_eq!(report["passed"], false);
    assert!(errors_contain(&report, "real child PID"));
    assert!(errors_contain(&report, "used network 'testnet10'"));
}
#[test]
fn raw_file_tampering_and_transport_wrappers_are_independent_rejections() {
    let fixture = Fixture::new();
    let stage = stage::required_stages().unwrap().remove(0);
    fixture.case(&stage, 731);
    fixture.bytes(
        "cases/mainnet-node/clipboard.raw.txt",
        b"[KGW_CHILD_STDOUT] tampered",
    );
    let report = stage::stage_evidence(&fixture.root, &stage).unwrap();
    assert_eq!(report["passed"], false);
    assert!(errors_contain(&report, "SHA256 did not match"));
    assert!(errors_contain(&report, "contained transport wrapper text"));
}
#[test]
fn bridge_selected_port_override_still_requires_matching_tcp_evidence() {
    let fixture = Fixture::new();
    let stage = stage::required_stages().unwrap().remove(2);
    fixture.case(&stage, 733);
    fixture.json(
        "cases/mainnet-bridge/bridge-runtime-selection.json",
        &json!({"runtime_bridge_port":20556}),
    );
    let report = stage::stage_evidence(&fixture.root, &stage).unwrap();
    assert_eq!(report["passed"], false);
    assert_eq!(report["ports"][0]["port"], 20556);
    assert!(errors_contain(&report, "port 20556"));
}
#[test]
fn summary_requires_observation_file_and_every_stage() {
    let fixture = Fixture::new();
    for (index, stage) in stage::required_stages().unwrap().iter().enumerate() {
        fixture.case(stage, 731 + index as i32);
    }
    let missing = stage::summary(&fixture.root).unwrap();
    assert_eq!(missing["passed"], false);
    assert_eq!(missing["passed_stages"].as_array().unwrap().len(), 4);
    assert!(errors_contain(&missing, "Missing zero-touch-observations"));
    fixture.json(
        "zero-touch-observations.json",
        &json!({"pids":[],"ports":[]}),
    );
    let present = stage::summary(&fixture.root).unwrap();
    assert_eq!(present["passed"], true);
    assert_eq!(present["process_ids"].as_array().unwrap().len(), 4);
    assert_eq!(present["clipboard_hashes"].as_array().unwrap().len(), 4);
}
#[test]
fn testnet13_evidence_rejects_enablement_or_running_status() {
    let fixture = Fixture::new();
    fixture.policy();
    assert_eq!(reports::policy(&fixture.root).unwrap()["passed"], true);
    fixture.json("cases/testnet13-policy/testnet13-policy.json",&json!({"nodeEnabled":true,"bridgeEnabled":true,"startBlocked":{"ok":true,"error":"unexpected"},"status":"running=true;pid=31"}));
    let report = reports::policy(&fixture.root).unwrap();
    assert_eq!(report["passed"], false);
    assert_eq!(report["errors"].as_array().unwrap().len(), 5);
}
#[test]
fn wdio_requires_exact_counters_and_expected_test_names() {
    let fixture = Fixture::new();
    fixture.wdio();
    assert_eq!(reports::wdio(&fixture.root).unwrap()["passed"], true);
    let mut value = read_json(&fixture.root.join("json/wdio-0-0.json")).unwrap();
    value["state"]["skipped"] = json!(1);
    value["suites"][0]["tests"][0]["name"] = json!("unexpected test");
    fixture.json("json/wdio-0-0.json", &value);
    let report = reports::wdio(&fixture.root).unwrap();
    assert_eq!(report["passed"], false);
    assert!(errors_contain(&report, "skipped=1"));
    assert!(errors_contain(&report, "did not contain expected test"));
}
#[test]
fn duplicate_json_reports_cannot_satisfy_single_spec_contract() {
    let fixture = Fixture::new();
    fixture.wdio();
    let report = read_json(&fixture.root.join("json/wdio-0-0.json")).unwrap();
    fixture.json("json/duplicate.json", &report);
    let result = reports::wdio(&fixture.root).unwrap();
    assert_eq!(result["passed"], false);
    assert_eq!(result["spec_count"], 2);
}
#[test]
fn malformed_xml_and_forbidden_dtd_fail_closed() {
    let fixture = Fixture::new();
    fixture.wdio();
    for content in [
        "<testsuites><testsuite tests=\"5\" failures=\"0\" errors=\"0\" skipped=\"0\"/></wrong>",
        "<!DOCTYPE x [<!ENTITY x 'expanded'>]><x>&x;</x>",
    ] {
        fixture.bytes("junit/wdio-0-0.xml", content.as_bytes());
        let report = reports::wdio(&fixture.root).unwrap();
        assert_eq!(report["passed"], false);
        assert_eq!(report["junit_tests"], 0);
        assert!(errors_contain(&report, "Invalid WebdriverIO JUnit"));
    }
}
#[test]
fn unicode_text_decoding_preserves_valid_bom_but_rejects_broken_bytes() {
    let fixture = Fixture::new();
    fixture.bytes("utf8.txt", b"\xef\xbb\xbfhello");
    fixture.bytes("utf16.txt", b"\xff\xfeh\0i\0");
    fixture.bytes("odd.txt", b"\xff\xfeh");
    fixture.bytes("broken.txt", b"\xffhello");
    assert_eq!(read_text(&fixture.root.join("utf8.txt")).unwrap(), "hello");
    assert_eq!(read_text(&fixture.root.join("utf16.txt")).unwrap(), "hi");
    assert!(read_text(&fixture.root.join("odd.txt")).is_err());
    assert!(read_text(&fixture.root.join("broken.txt")).is_err());
}
#[test]
fn oversized_saved_evidence_is_rejected_before_allocation() {
    let fixture = Fixture::new();
    let path = fixture.root.join("oversized.json");
    let file = fs::File::create(&path).unwrap();
    file.set_len(MAX_EVIDENCE_BYTES + 1).unwrap();
    drop(file);
    assert!(read_json(&path).unwrap_err().contains("exceeds"));
}
