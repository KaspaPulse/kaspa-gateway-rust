use super::*;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Stage {
    pub name: &'static str,
    pub slug: &'static str,
    pub network: &'static str,
    pub role: &'static str,
    pub status_file: &'static str,
    pub ports: Vec<i32>,
}
impl Stage {
    pub fn to_json(&self) -> Value {
        json!({"Name":self.name,"Slug":self.slug,"Network":self.network,
            "RuntimeRole":self.role,"OwnerStatusFile":self.status_file,"RequiredPorts":self.ports})
    }
}
fn env_port(name: &str, default: i32) -> EvidenceResult<i32> {
    let raw = std::env::var(name).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(default);
    }
    let value = raw
        .trim()
        .parse::<i32>()
        .ok()
        .filter(|port| (1024..=65535).contains(port));
    value.ok_or_else(|| format!("{name} must be an integer TCP port in 1024..65535; got '{raw}'."))
}
pub fn required_stages() -> EvidenceResult<Vec<Stage>> {
    let mr = env_port("KGW_E2E_MAINNET_RPC_PORT", 16110)?;
    let mp = env_port("KGW_E2E_MAINNET_P2P_PORT", 16111)?;
    let mb = env_port("KGW_E2E_MAINNET_BRIDGE_PORT", 5556)?;
    let tr = env_port("KGW_E2E_TESTNET10_RPC_PORT", 16210)?;
    let tp = env_port("KGW_E2E_TESTNET10_P2P_PORT", 16211)?;
    let tb = env_port("KGW_E2E_TESTNET10_BRIDGE_PORT", 5656)?;
    Ok(vec![
        Stage {
            name: "Mainnet Node",
            slug: "mainnet-node",
            network: "mainnet",
            role: "node",
            status_file: "node-owner-status.json",
            ports: vec![mr, mp],
        },
        Stage {
            name: "Testnet10 Node",
            slug: "testnet10-node",
            network: "testnet10",
            role: "node",
            status_file: "node-owner-status.json",
            ports: vec![tr, tp],
        },
        Stage {
            name: "Mainnet Bridge",
            slug: "mainnet-bridge",
            network: "mainnet",
            role: "bridge",
            status_file: "bridge-owner-status.json",
            ports: vec![mb],
        },
        Stage {
            name: "Testnet10 Bridge",
            slug: "testnet10-bridge",
            network: "testnet10",
            role: "bridge",
            status_file: "bridge-owner-status.json",
            ports: vec![tb],
        },
    ])
}
pub fn has_transport_wrapper(raw: &str) -> bool {
    let raw = raw.to_lowercase();
    [
        "kgw_raw_process_log_v1",
        "[kgw_child_stdout]",
        "[kgw_child_stderr]",
        "diagnostic_transport_record",
        ";source=self-worker;",
        ";runtime_role=",
        ";received_ms=",
    ]
    .iter()
    .any(|marker| raw.contains(marker))
}
fn status_pid(status: &Value) -> EvidenceResult<Option<i32>> {
    if let Some(pid) = decimal_i32(property(status, &["pid"]))? {
        return Ok(Some(pid));
    }
    let re = Regex::new(r"(?i)(^|;)pid=(\d+)(;|$)").map_err(|error| error.to_string())?;
    if let Some(found) = re.captures(&field(status, &["status"])) {
        return decimal_i32(&json!(&found[2]));
    }
    Ok(None)
}
fn has_port(state: &Value, port: i32) -> EvidenceResult<bool> {
    for entry in sequence(state) {
        for names in [
            ["LocalPort", "localPort", "local_port"],
            ["RemotePort", "remotePort", "remote_port"],
        ] {
            if decimal_i32(property(entry, &names))? == Some(port) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
pub fn stage_evidence(artifact: &Path, stage: &Stage) -> EvidenceResult<Value> {
    let name = stage.name;
    let case_dir = artifact.join("cases").join(stage.slug);
    let owner_path = case_dir.join(stage.status_file);
    let logs_path = case_dir.join("runtime-logs.json");
    let clipboard_path = case_dir.join("clipboard-capture.json");
    let raw_path = case_dir.join("clipboard.raw.txt");
    let process_path = case_dir.join("process-tree.json");
    let port_path = case_dir.join("port-state.json");
    let screenshot_path = case_dir.join("current-screenshot.png");
    let selection_path = case_dir.join("bridge-runtime-selection.json");
    let mut errors = Vec::<String>::new();
    if !case_dir.is_dir() {
        errors.push(format!(
            "Missing case directory for {name}: {}",
            case_dir.display()
        ));
    }
    let owner = read_json(&owner_path)?;
    let logs = read_json(&logs_path)?;
    let clipboard_capture = read_json(&clipboard_path)?;
    let processes = read_json(&process_path)?;
    let ports_state = read_json(&port_path)?;
    let selection = read_json(&selection_path)?;
    let pid = status_pid(&owner)?;
    if let Some(pid) = pid.filter(|pid| *pid > 0) {
        let mut found = false;
        for process in sequence(&processes) {
            if decimal_i32(property(process, &["ProcessId", "processId", "process_id"]))?
                == Some(pid)
            {
                found = true;
                break;
            }
        }
        if !found {
            errors.push(format!(
                "{name} PID {pid} was not present in process-tree.json."
            ));
        }
    } else {
        errors.push(format!(
            "{name} did not record a real child PID in {}.",
            stage.status_file
        ));
    }
    let mut required_ports = stage.ports.clone();
    if stage.role == "bridge" {
        let selected = decimal_i32(property(
            &selection,
            &[
                "runtimeBridgePort",
                "runtime_bridge_port",
                "listeningBridgePort",
                "listening_bridge_port",
                "bridgePort",
                "bridge_port",
            ],
        ))?;
        if let Some(port) = selected.filter(|port| *port > 0) {
            required_ports = vec![port];
        }
    }
    let mut stage_ports = Vec::new();
    for port in required_ports {
        stage_ports.push(json!({"network":stage.network,"runtime_role":stage.role,"host":"127.0.0.1","port":port}));
        if !has_port(&ports_state, port)? {
            errors.push(format!(
                "{name} did not capture TCP port evidence for port {port}."
            ));
        }
    }
    if !screenshot_path.is_file() {
        errors.push(format!("{name} did not capture a screenshot."));
    }
    let entries = sequence(property(&logs, &["entries"]));
    if entries.is_empty() || (entries.len() == 1 && entries[0].is_null()) {
        errors.push(format!(
            "{name} runtime-logs.json did not contain raw entries."
        ));
    }
    for entry in entries.into_iter().filter(|entry| !entry.is_null()) {
        let network = field(entry, &["network"]);
        let role = field(entry, &["runtimeRole", "runtime_role"]);
        let raw = field(entry, &["rawText", "raw_text"]);
        if !network.eq_ignore_ascii_case(stage.network) {
            errors.push(format!(
                "{name} runtime log entry used network '{network}'."
            ));
            break;
        }
        if !role.eq_ignore_ascii_case(stage.role) {
            errors.push(format!("{name} runtime log entry used role '{role}'."));
            break;
        }
        if has_transport_wrapper(&raw) {
            errors.push(format!(
                "{name} runtime log entry contained transport wrapper text."
            ));
            break;
        }
    }
    let mut clipboard_hash = Value::Null;
    let mut instance = String::new();
    if clipboard_capture.is_null() {
        errors.push(format!("{name} did not write clipboard-capture.json."));
    } else {
        let event = property(&clipboard_capture, &["event"]);
        let clipboard = property(&clipboard_capture, &["clipboard"]);
        let network = field(event, &["network"]);
        let role = field(event, &["runtimeRole", "runtime_role"]);
        instance = field(event, &["bridgeInstanceId", "bridge_instance_id"]);
        let expected = field(event, &["expectedSha256", "expected_sha256"]);
        let result = field(event, &["resultSha256", "result_sha256"]);
        let source = field(event, &["resultSource", "result_source", "status"]);
        let actual = field(clipboard, &["sha256", "actual_sha256"]);
        if !network.eq_ignore_ascii_case(stage.network) {
            errors.push(format!(
                "{name} clipboard event network '{network}' did not match {}.",
                stage.network
            ));
        }
        if !role.eq_ignore_ascii_case(stage.role) {
            errors.push(format!(
                "{name} clipboard event role '{role}' did not match {}.",
                stage.role
            ));
        }
        if stage.role == "bridge" && instance.trim().is_empty() {
            errors.push(format!(
                "{name} clipboard event did not include a bridge instance."
            ));
        }
        if expected.trim().is_empty() || result.trim().is_empty() || actual.trim().is_empty() {
            errors.push(format!("{name} clipboard hashes were incomplete."));
        } else if !expected.eq_ignore_ascii_case(&result) || !expected.eq_ignore_ascii_case(&actual)
        {
            errors.push(format!(
                "{name} clipboard expected/native/Windows SHA256 values did not match."
            ));
        }
        if raw_path.is_file() {
            let raw = read_text(&raw_path)?;
            if !actual.trim().is_empty() && !identity::hash_text(&raw).eq_ignore_ascii_case(&actual)
            {
                errors.push(format!(
                    "{name} clipboard.raw.txt SHA256 did not match clipboard-capture.json."
                ));
            }
            if has_transport_wrapper(&raw) {
                errors.push(format!(
                    "{name} clipboard.raw.txt contained transport wrapper text."
                ));
            }
        } else {
            errors.push(format!("{name} did not write clipboard.raw.txt."));
        }
        clipboard_hash = json!({"stage":name,"network":stage.network,"runtime_role":stage.role,"bridge_instance_id":instance,
            "expected_sha256":expected,"result_sha256":result,"result_source":source,"windows_clipboard_sha256":actual,"raw_file":raw_path});
    }
    Ok(
        json!({"name":name,"slug":stage.slug,"network":stage.network,"runtime_role":stage.role,"bridge_instance_id":instance,
        "passed":errors.is_empty(),"errors":errors,"warnings":[],"process_id":pid,"ports":stage_ports,"clipboard_hash":clipboard_hash,
        "evidence_files":{"owner_status":owner_path,"runtime_logs":logs_path,"clipboard_capture":clipboard_path,"clipboard_raw":raw_path,
            "process_tree":process_path,"port_state":port_path,"screenshot":screenshot_path}}),
    )
}
pub fn summary(artifact: &Path) -> EvidenceResult<Value> {
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut reports = Vec::new();
    let mut hashes = Vec::new();
    let mut pids = Vec::new();
    let mut ports = Vec::new();
    let observations = read_json(&artifact.join("zero-touch-observations.json"))?;
    if observations.is_null() {
        errors.push("Missing zero-touch-observations.json.".to_owned());
    } else {
        for entry in sequence(property(&observations, &["pids"])) {
            if let Some(pid) = integer(property(entry, &["pid"]))? {
                pids.push(json!({"network":field(entry,&["network"]),"runtime_role":field(entry,&["runtimeRole","runtime_role"]),"pid":pid}));
            }
        }
        for entry in sequence(property(&observations, &["ports"])) {
            if let Some(port) = integer(property(entry, &["port"]))? {
                ports.push(json!({"network":field(entry,&["network"]),"runtime_role":field(entry,&["runtimeRole","runtime_role"]),
                    "host":field(entry,&["host"]),"port":port,"purpose":field(entry,&["purpose"])}));
            }
        }
    }
    for stage in required_stages()? {
        let report = stage_evidence(artifact, &stage)?;
        if !truthy(&report["passed"]) {
            append_errors(&mut errors, &report, "errors");
        }
        append_errors(&mut warnings, &report, "warnings");
        if !report["clipboard_hash"].is_null() {
            hashes.push(report["clipboard_hash"].clone());
        }
        if let Some(pid) = integer(&report["process_id"])?.filter(|pid| *pid > 0) {
            pids.push(json!({"network":report["network"],"runtime_role":report["runtime_role"],"bridge_instance_id":report["bridge_instance_id"],"pid":pid}));
        }
        ports.extend(sequence(&report["ports"]).into_iter().cloned());
        reports.push(report);
    }
    let passed = reports
        .iter()
        .filter(|report| truthy(&report["passed"]))
        .map(|report| report["name"].clone())
        .collect::<Vec<_>>();
    let failed = reports
        .iter()
        .find(|report| !truthy(&report["passed"]))
        .map(|report| report["name"].clone())
        .unwrap_or(Value::Null);
    Ok(
        json!({"passed":errors.is_empty(),"validation_errors":errors,"warnings":warnings,"passed_stages":passed,"failed_stage":failed,
        "clipboard_hashes":hashes,"process_ids":pids,"ports":ports,"stages":reports}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_transport_markers_are_rejected_case_insensitively() {
        assert!(has_transport_wrapper("[KGW_CHILD_STDERR] wrapped"));
        assert!(has_transport_wrapper(";RUNTIME_ROLE=node"));
        assert!(!has_transport_wrapper(
            "2026-09-25 05:00:00 native node message"
        ));
    }
    #[test]
    fn status_pid_prefers_explicit_pid_and_requires_status_boundaries() {
        assert_eq!(
            status_pid(&json!({"pid":"73","status":"pid=99;"})).unwrap(),
            Some(73)
        );
        assert_eq!(
            status_pid(&json!({"status":"running=true;PID=99;"})).unwrap(),
            Some(99)
        );
        assert_eq!(status_pid(&json!({"status":"otherpid=99;"})).unwrap(), None);
        assert!(status_pid(&json!({"pid":"2147483648"})).is_err());
    }
    #[test]
    fn port_aliases_preserve_local_or_remote_evidence_contract() {
        assert!(has_port(&json!([{ "remote_port":"16110"}]), 16110).unwrap());
        assert!(!has_port(&json!([{ "localPort":16111}]), 16110).unwrap());
    }
}
