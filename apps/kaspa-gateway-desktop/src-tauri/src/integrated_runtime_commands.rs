use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static KGW_CONTROLLER: OnceLock<Arc<kaspa_gateway_rk_node::KgwServiceController>> = OnceLock::new();
static KGW_RAW_PROCESS_LOG_SEQUENCE_V1: AtomicU64 = AtomicU64::new(1);
const KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1: usize = 4096;
pub(crate) const KGW_NODE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS_V1: u64 = 90_000;
pub(crate) const KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1: u64 = 100_000;
pub(crate) const KGW_BRIDGE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1: u64 =
    kaspa_gateway_rk_bridge::KGW_BRIDGE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS + 9_000;
const _: () = assert!(
    KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1
        > KGW_NODE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS_V1
);

fn controller() -> &'static Arc<kaspa_gateway_rk_node::KgwServiceController> {
    KGW_CONTROLLER.get_or_init(kaspa_gateway_rk_node::KgwServiceController::spawn)
}

#[derive(Debug)]
struct KgwParallelSelfWorker {
    role: String,
    network: String,
    appdir: String,

    node_mode: String,
    child: Child,
    reader_handles: Vec<std::thread::JoinHandle<()>>,
    // Keeps the registry-owned buffer alive with the runtime owner; insertion
    // remains exclusive to `kgw_worker_spawn_reader`.
    _raw_logs: KgwRawProcessLogBufferV1,
    started_ms: u128,
    exit_logged: bool,
    readiness_evidence: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KgwStartupControlMessageV1 {
    pub(crate) version: u8,
    pub(crate) outcome: String,
    pub(crate) runtime_role: String,
    pub(crate) network: String,
    pub(crate) evidence: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwRuntimeRawLogEntryV1 {
    pub sequence: u64,
    pub network: String,
    pub source: String,
    pub runtime_role: String,
    pub bridge_instance_id: Option<String>,
    pub stream: String,
    pub received_ms: u64,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwRuntimeDiagnosticRecordV1 {
    pub diagnostic_event: String,
    pub network: String,
    pub source: String,
    pub runtime_role: String,
    pub bridge_instance_id: Option<String>,
    pub received_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwRuntimeLogsReportV1 {
    pub version: String,
    pub network: Option<String>,
    pub runtime_role: Option<String>,
    pub bridge_instance_id: Option<String>,
    pub source: String,
    pub buffer_limit: usize,
    pub entries: Vec<KgwRuntimeRawLogEntryV1>,
    pub diagnostics: Vec<KgwRuntimeDiagnosticRecordV1>,
}

static KGW_PARALLEL_SELF_WORKERS: OnceLock<Mutex<HashMap<String, KgwParallelSelfWorker>>> =
    OnceLock::new();
type KgwRawProcessLogBufferV1 = Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>;
static KGW_RAW_PROCESS_LOG_BUFFERS_V1: OnceLock<Mutex<HashMap<String, KgwRawProcessLogBufferV1>>> =
    OnceLock::new();

fn kgw_parallel_self_workers() -> &'static Mutex<HashMap<String, KgwParallelSelfWorker>> {
    KGW_PARALLEL_SELF_WORKERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn kgw_raw_process_log_buffers_v1() -> &'static Mutex<HashMap<String, KgwRawProcessLogBufferV1>> {
    KGW_RAW_PROCESS_LOG_BUFFERS_V1.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
fn kgw_worker_uses_test_command() -> bool {
    std::env::var_os("KGW_TEST_SELF_WORKER_COMMAND").is_some()
        || std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_COMMAND").is_some()
        || std::env::var_os("KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND").is_some()
        || std::env::var_os("KGW_TEST_SELF_WORKER_MISSING_COMMAND").is_some()
}

#[cfg(not(test))]
fn kgw_worker_uses_test_command() -> bool {
    false
}

fn kgw_worker_now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn kgw_worker_now_ms_u64() -> u64 {
    u64::try_from(kgw_worker_now_ms()).unwrap_or(u64::MAX)
}

fn kgw_worker_next_raw_log_sequence_v1() -> u64 {
    KGW_RAW_PROCESS_LOG_SEQUENCE_V1.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
thread_local! {
    static KGW_START_TRACE_TEST_ENABLED_V1: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) struct KgwStartTraceTestGuardV1 {
    previous: bool,
}

#[cfg(test)]
impl Drop for KgwStartTraceTestGuardV1 {
    fn drop(&mut self) {
        KGW_START_TRACE_TEST_ENABLED_V1.with(|enabled| enabled.set(self.previous));
    }
}

#[cfg(test)]
pub(crate) fn kgw_start_trace_test_enable_v1() -> KgwStartTraceTestGuardV1 {
    let previous = KGW_START_TRACE_TEST_ENABLED_V1.with(|enabled| {
        let previous = enabled.get();
        enabled.set(true);
        previous
    });

    KgwStartTraceTestGuardV1 { previous }
}

pub(crate) fn kgw_start_trace_enabled_v1() -> bool {
    #[cfg(test)]
    if KGW_START_TRACE_TEST_ENABLED_V1.with(|enabled| enabled.get()) {
        return true;
    }

    match std::env::var("KGW_START_TRACE") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn kgw_start_trace_json_escape_v1(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

fn kgw_start_trace_clean_field_v1(value: &str, fallback: &str) -> String {
    let clean = value
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        return fallback.to_string();
    }

    clean.chars().take(160).collect()
}

fn kgw_start_trace_safe_details_v1(value: &str) -> String {
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("private")
        || lowered.contains("mnemonic")
        || lowered.contains("wallet")
        || lowered.contains("address")
    {
        return "{\"redacted\":true,\"reason\":\"sensitive-field\"}".to_string();
    }

    let clean = value
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();

    clean.chars().take(1200).collect()
}

pub(crate) fn kgw_start_trace_format_v1(
    source: &str,
    stage: &str,
    network: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
) -> String {
    let source = kgw_start_trace_clean_field_v1(source, "native");
    let stage = kgw_start_trace_clean_field_v1(stage, "unknown");
    let network = kgw_start_trace_clean_field_v1(network, "unknown");
    let action = kgw_start_trace_clean_field_v1(action, "unknown");
    let result = kgw_start_trace_clean_field_v1(result, "unknown");
    let details = details.map(kgw_start_trace_safe_details_v1);

    let mut line = format!(
        "[KGW_START_TRACE] {{\"timestamp\":{},\"source\":\"{}\",\"stage\":\"{}\",\"network\":\"{}\",\"action\":\"{}\",\"result\":\"{}\"",
        kgw_worker_now_ms(),
        kgw_start_trace_json_escape_v1(&source),
        kgw_start_trace_json_escape_v1(&stage),
        kgw_start_trace_json_escape_v1(&network),
        kgw_start_trace_json_escape_v1(&action),
        kgw_start_trace_json_escape_v1(&result)
    );

    if let Some(details) = details {
        line.push_str(",\"details\":\"");
        line.push_str(&kgw_start_trace_json_escape_v1(&details));
        line.push('"');
    }

    line.push('}');
    line
}

#[cfg(test)]
static KGW_START_TRACE_TEST_LINES_V1: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn kgw_start_trace_test_take_lines_v1() -> Vec<String> {
    let sink = KGW_START_TRACE_TEST_LINES_V1.get_or_init(|| Mutex::new(Vec::new()));
    match sink.lock() {
        Ok(mut guard) => std::mem::take(&mut *guard),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn kgw_start_trace_emit_v1(
    source: &str,
    stage: &str,
    network: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
) {
    if !kgw_start_trace_enabled_v1() {
        return;
    }

    let line = kgw_start_trace_format_v1(source, stage, network, action, result, details);
    eprintln!("{line}");

    #[cfg(test)]
    {
        let sink = KGW_START_TRACE_TEST_LINES_V1.get_or_init(|| Mutex::new(Vec::new()));
        if let Ok(mut guard) = sink.lock() {
            guard.push(line);
        }
    }
}

fn kgw_worker_key(role: &str, network: &str) -> String {
    format!(
        "{}:{}",
        role.trim().to_ascii_lowercase(),
        network.trim().to_ascii_lowercase()
    )
}

fn kgw_worker_replace_raw_log_buffer_v1(
    role: &str,
    network: &str,
) -> Result<KgwRawProcessLogBufferV1, String> {
    let buffer = Arc::new(Mutex::new(VecDeque::new()));
    let mut buffers = kgw_raw_process_log_buffers_v1()
        .lock()
        .map_err(|_| "raw process log buffer registry lock failed".to_string())?;
    buffers.insert(kgw_worker_key(role, network), Arc::clone(&buffer));
    Ok(buffer)
}

fn kgw_worker_clean_optional_identifier_v1(value: Option<String>) -> Option<String> {
    let clean = value?
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '\0' && *ch != '\r' && *ch != '\n')
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        None
    } else {
        Some(clean.chars().take(128).collect())
    }
}

fn kgw_worker_push_raw_log(
    logs: &Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>,
    entry: KgwRuntimeRawLogEntryV1,
) {
    if let Ok(mut guard) = logs.lock() {
        if guard.len() >= KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1 {
            guard.pop_front();
        }

        guard.push_back(entry);
    }
}

fn kgw_worker_raw_log_entry_v1(
    role: &str,
    network: &str,
    stream: &str,
    line: String,
) -> KgwRuntimeRawLogEntryV1 {
    KgwRuntimeRawLogEntryV1 {
        sequence: kgw_worker_next_raw_log_sequence_v1(),
        network: network.to_string(),
        source: "self-worker".to_string(),
        runtime_role: role.to_string(),
        // The official Bridge library exposes process-wide output and no
        // structural listener attribution. Keep this field unset rather than
        // inventing per-instance provenance from the active UI selection.
        bridge_instance_id: None,
        stream: stream.to_string(),
        received_ms: kgw_worker_now_ms_u64(),
        raw_text: line,
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests; unused in the library test target.
pub(crate) fn kgw_raw_log_entry_for_test_v1(
    sequence: u64,
    network: &str,
    runtime_role: &str,
    bridge_instance_id: Option<&str>,
    stream: &str,
    raw_text: &str,
) -> KgwRuntimeRawLogEntryV1 {
    KgwRuntimeRawLogEntryV1 {
        sequence,
        network: network.to_string(),
        source: "self-worker".to_string(),
        runtime_role: runtime_role.to_string(),
        bridge_instance_id: bridge_instance_id.map(str::to_string),
        stream: stream.to_string(),
        received_ms: 1,
        raw_text: raw_text.to_string(),
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests; unused in the library test target.
pub(crate) fn kgw_raw_log_text_from_entries_for_test_v1(
    mut entries: Vec<KgwRuntimeRawLogEntryV1>,
) -> String {
    entries.sort_by_key(|entry| entry.sequence);
    entries
        .into_iter()
        .map(|entry| entry.raw_text)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn kgw_worker_safe_child_line_v1(line: &str) -> String {
    let clean = line
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();

    let mut redacted = Vec::new();
    let mut redact_next = false;

    for token in clean.split_whitespace() {
        let lower = token.to_ascii_lowercase();

        if redact_next {
            redacted.push("[redacted-sensitive-value]".to_string());
            redact_next = false;
            continue;
        }

        if matches!(
            lower.as_str(),
            "--internal-cpu-miner-address"
                | "--wallet"
                | "--wallet-address"
                | "--private-key"
                | "--token"
                | "--secret"
                | "--mnemonic"
        ) {
            redacted.push(token.to_string());
            redact_next = true;
            continue;
        }

        if lower.starts_with("kaspa:")
            || lower.starts_with("kaspatest:")
            || lower.starts_with("kaspadev:")
            || lower.starts_with("kaspareg:")
        {
            redacted.push("[redacted-wallet-address]".to_string());
            continue;
        }

        if let Some((key, _value)) = token.split_once('=') {
            let key_lower = key.to_ascii_lowercase();
            if key_lower.contains("token")
                || key_lower.contains("secret")
                || key_lower.contains("private")
                || key_lower.contains("mnemonic")
                || key_lower == "wallet"
                || key_lower == "wallet_address"
                || key_lower == "wallet-address"
                || key_lower == "internal_cpu_miner_address"
                || key_lower == "internal-cpu-miner-address"
            {
                redacted.push(format!("{key}=[redacted-sensitive-value]"));
                continue;
            }
        }

        redacted.push(token.to_string());
    }

    if redacted.is_empty() {
        String::new()
    } else {
        redacted.join(" ")
    }
}

pub(crate) fn kgw_worker_format_child_mirror_line_v1(
    role: &str,
    network: &str,
    bridge_instance_id: Option<&str>,
    stream: &str,
    child_pid: u32,
    line: &str,
) -> String {
    let prefix = if stream == "stdout" {
        "[KGW_CHILD_STDOUT]"
    } else {
        "[KGW_CHILD_STDERR]"
    };
    let safe_line = kgw_worker_safe_child_line_v1(line);
    let bridge_instance_json = bridge_instance_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("\"{}\"", kgw_start_trace_json_escape_v1(value.trim())))
        .unwrap_or_else(|| "null".to_string());

    format!(
        "{} {{\"timestamp\":{},\"eventKind\":\"diagnostic_transport_record\",\"network\":\"{}\",\"runtimeRole\":\"{}\",\"bridgeInstanceId\":{},\"pid\":{},\"stage\":\"diagnostic_transport.child.{}\",\"stream\":\"{}\",\"action\":\"start\",\"result\":\"line\",\"safeText\":\"{}\"}}",
        prefix,
        kgw_worker_now_ms(),
        kgw_start_trace_json_escape_v1(network),
        kgw_start_trace_json_escape_v1(role),
        bridge_instance_json,
        child_pid,
        kgw_start_trace_json_escape_v1(stream),
        kgw_start_trace_json_escape_v1(stream),
        kgw_start_trace_json_escape_v1(&safe_line)
    )
}

fn kgw_start_trace_capture_test_line_v1(_line: &str) {
    #[cfg(test)]
    {
        let sink = KGW_START_TRACE_TEST_LINES_V1.get_or_init(|| Mutex::new(Vec::new()));
        if let Ok(mut guard) = sink.lock() {
            guard.push(_line.to_string());
        }
    }
}

fn kgw_worker_argument_names_for_trace_v1(command: &Command) -> Vec<String> {
    let mut names = Vec::new();
    let mut previous_flag: Option<String> = None;

    for arg in command.get_args() {
        let text = arg.to_string_lossy().to_string();

        if text.starts_with("--") {
            previous_flag = Some(text.clone());
            names.push(text);
            continue;
        }

        let placeholder = match previous_flag.as_deref() {
            Some("--kgw-self-worker") => "<runtime-role>",
            Some("--network") => "<network>",
            Some("--appdir") => "<database-directory>",
            Some("--rpc") => "<rpc-endpoint>",
            Some("--listen") => "<p2p-listen>",
            Some("--stratum") => "<stratum-endpoint>",
            Some("--node-mode") => "<node-mode>",
            Some("--bridge-config") => "<bridge-config-path>",
            Some("--bridge-instance-listen") => "<bridge-instance-listen>",
            Some("--internal-cpu-miner-address") => "<redacted-wallet-address>",
            Some("--internal-cpu-miner-threads") => "<thread-count>",
            Some("--internal-cpu-miner-throttle-ms") => "<throttle-ms>",
            Some("--internal-cpu-miner-template-poll-ms") => "<template-poll-ms>",
            Some("--exact") => "<test-name>",
            Some(flag) if flag.starts_with("--") => "<argument>",
            _ => "<argument>",
        };

        names.push(placeholder.to_string());
        previous_flag = None;
    }

    names
}

fn kgw_worker_endpoint_port_v1(endpoint: &str) -> Option<u16> {
    endpoint
        .rsplit(':')
        .next()
        .and_then(|value| value.trim().parse::<u16>().ok())
}

fn kgw_worker_node_mode_for_trace_v1(role: &str, bridge_node_mode: &str) -> String {
    if role == "node" {
        "same-exe-self-worker".to_string()
    } else {
        bridge_node_mode.to_string()
    }
}

fn kgw_worker_spawn_plan_details_v1(
    role: &str,
    network: &str,
    node_mode: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
    command: &Command,
    uses_test_command: bool,
    has_bridge_config_path: bool,
) -> String {
    let executable_path = command.get_program().to_string_lossy().to_string();
    let working_directory = command
        .get_current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|path| path.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    serde_json::json!({
        "runtimeRole": role,
        "network": network,
        "nodeMode": node_mode,
        "sameExecutable": true,
        "externalKaspadExe": false,
        "usesTestCommand": uses_test_command,
        "hasBridgeConfigPath": has_bridge_config_path,
        "executablePath": executable_path,
        "workingDirectory": working_directory,
        "argumentNames": kgw_worker_argument_names_for_trace_v1(command),
        "databaseDirectory": settings.app_dir_name,
        "rpcPort": kgw_worker_endpoint_port_v1(&settings.rpc_endpoint),
        "stratumPort": kgw_worker_endpoint_port_v1(&settings.stratum_listen),
    })
    .to_string()
}

fn kgw_worker_apply_current_dir_v1(command: &mut Command) {
    if let Ok(current_dir) = std::env::current_dir() {
        command.current_dir(current_dir);
    }
}

fn kgw_worker_startup_control_path_v1(role: &str, network: &str) -> std::path::PathBuf {
    static STARTUP_CONTROL_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let nonce = kgw_worker_now_ms();
    let sequence = STARTUP_CONTROL_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir()
        .join("KaspaGateway")
        .join("startup-control")
        .join(format!(
            "{}-{}-{}-{nonce}-{sequence}.json",
            role,
            network,
            std::process::id()
        ))
}

fn kgw_worker_read_startup_control_v1(
    path: &std::path::Path,
) -> Result<KgwStartupControlMessageV1, String> {
    let payload = std::fs::read(path)
        .map_err(|error| format!("read startup control failed {}: {error}", path.display()))?;
    serde_json::from_slice(&payload)
        .map_err(|error| format!("parse startup control failed {}: {error}", path.display()))
}

pub(crate) fn kgw_worker_validate_startup_attestation_v1(
    message: KgwStartupControlMessageV1,
    role: &str,
    network: &str,
) -> Result<String, String> {
    if message.version != 1 {
        return Err(format!(
            "startup attestation protocol version mismatch;expected=1;actual={}",
            message.version
        ));
    }

    if message.runtime_role != role || message.network != network {
        return Err(format!(
            "startup attestation identity mismatch;expected_role={};actual_role={};expected_network={};actual_network={}",
            role, message.runtime_role, network, message.network
        ));
    }

    match message.outcome.as_str() {
        "READY" => message
            .evidence
            .filter(|evidence| !evidence.trim().is_empty())
            .ok_or_else(|| {
                format!(
                    "startup attestation READY is missing evidence;role={};network={}",
                    role, network
                )
            }),
        "FAILED" => Err(format!(
            "role startup failed;role={};network={};error={}",
            role,
            network,
            message
                .error
                .unwrap_or_else(|| "self-worker reported FAILED without an error".to_string())
        )),
        other => Err(format!(
            "startup attestation outcome is invalid;role={};network={};outcome={other}",
            role, network
        )),
    }
}

fn kgw_worker_remove_startup_control_v1(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(path.with_extension("tmp"));
}

fn kgw_worker_wait_for_startup_attestation_v1(
    child: &mut Child,
    path: &std::path::Path,
    role: &str,
    network: &str,
    logs: &Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>,
) -> Result<String, String> {
    let timeout_ms = std::env::var("KGW_TEST_STARTUP_ATTESTATION_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(match role {
            "bridge" => KGW_BRIDGE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1,
            _ => KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1,
        });
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

    loop {
        if path.is_file() {
            let message = kgw_worker_read_startup_control_v1(path);
            kgw_worker_remove_startup_control_v1(path);
            return kgw_worker_validate_startup_attestation_v1(message?, role, network);
        }

        if let Some(exit_status) = child.try_wait().map_err(|error| error.to_string())? {
            kgw_worker_remove_startup_control_v1(path);
            let captured = logs
                .lock()
                .map(|guard| {
                    guard
                        .iter()
                        .map(|entry| kgw_worker_safe_child_line_v1(&entry.raw_text))
                        .collect::<Vec<_>>()
                        .join(" | ")
                })
                .unwrap_or_else(|_| "worker log lock failed".to_string());
            let exit_code = exit_status
                .code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string());
            return Err(format!(
                "self-worker exited before role readiness;role={};network={};status={};exit_code={};logs={}",
                role, network, exit_status, exit_code, captured
            ));
        }

        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            kgw_worker_remove_startup_control_v1(path);
            return Err(format!(
                "role startup attestation timed out;role={};network={};timeout_ms={timeout_ms}",
                role, network
            ));
        }

        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn kgw_worker_spawn_reader<R>(
    role: String,
    network: String,
    bridge_instance_id: Option<String>,
    stream: String,
    child_pid: u32,
    reader: R,
    logs: Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>,
) -> std::thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let buffered = BufReader::new(reader);

        for line in buffered.lines() {
            match line {
                Ok(line) => {
                    if kgw_start_trace_enabled_v1() {
                        let mirror = kgw_worker_format_child_mirror_line_v1(
                            &role,
                            &network,
                            bridge_instance_id.as_deref(),
                            &stream,
                            child_pid,
                            &line,
                        );
                        eprintln!("{mirror}");
                        kgw_start_trace_capture_test_line_v1(&mirror);
                    }

                    let entry = kgw_worker_raw_log_entry_v1(&role, &network, &stream, line);
                    kgw_worker_push_raw_log(&logs, entry);
                }
                Err(_error) => {
                    kgw_start_trace_emit_v1(
                        "native",
                        "native.diagnostic_transport_reader_error",
                        &network,
                        "read-child-line",
                        "error",
                        Some(&format!(
                            "{{\"eventKind\":\"diagnostic_transport_record\",\"runtimeRole\":\"{}\",\"stream\":\"{}\",\"pid\":{},\"error\":\"{}\"}}",
                            kgw_start_trace_json_escape_v1(&role),
                            kgw_start_trace_json_escape_v1(&stream),
                            child_pid,
                            kgw_start_trace_json_escape_v1(&_error.to_string())
                        )),
                    );
                    break;
                }
            }
        }
    })
}

fn kgw_worker_join_readers_v1(reader_handles: Vec<std::thread::JoinHandle<()>>) {
    for reader_handle in reader_handles {
        let _ = reader_handle.join();
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_capture_raw_pipe_for_test_v1<R>(
    role: &str,
    network: &str,
    stream: &str,
    reader: R,
    logs: KgwRawProcessLogBufferV1,
) -> std::thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    kgw_worker_spawn_reader(
        role.to_string(),
        network.to_string(),
        None,
        stream.to_string(),
        1,
        reader,
        logs,
    )
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_empty_raw_log_buffer_for_test_v1() -> KgwRawProcessLogBufferV1 {
    Arc::new(Mutex::new(VecDeque::new()))
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_raw_log_buffer_entries_for_test_v1(
    logs: &KgwRawProcessLogBufferV1,
) -> Vec<KgwRuntimeRawLogEntryV1> {
    logs.lock()
        .map(|entries| entries.iter().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_clear_raw_process_log_buffers_for_test_v1() {
    if let Ok(mut buffers) = kgw_raw_process_log_buffers_v1().lock() {
        buffers.clear();
    }
}

fn kgw_worker_command(
    role: &str,
    network: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
    startup_control_path: &std::path::Path,
) -> Result<Command, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(exe);
    kgw_worker_apply_current_dir_v1(&mut command);

    #[cfg(test)]
    if let Some(path) = std::env::var_os("KGW_TEST_SELF_WORKER_MISSING_COMMAND") {
        let mut missing = Command::new(path);
        kgw_worker_apply_current_dir_v1(&mut missing);
        missing
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(missing);
    }

    #[cfg(test)]
    if let Some(fail_line) = std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_COMMAND") {
        command
            .arg("--exact")
            .arg("integrated_runtime_commands::kgw_test_self_worker_fail")
            .arg("--nocapture")
            .env("KGW_TEST_SELF_WORKER_FAIL_CHILD", "1")
            .env("KGW_TEST_SELF_WORKER_FAIL_STDERR", fail_line)
            .env("KGW_TEST_SELF_WORKER_ROLE", role)
            .env("KGW_TEST_SELF_WORKER_NETWORK", network)
            .env("KGW_TEST_SELF_WORKER_CONTROL_PATH", startup_control_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(command);
    }

    #[cfg(test)]
    if let Some(fail_line) = std::env::var_os("KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND") {
        command
            .arg("--exact")
            .arg("integrated_runtime_commands::kgw_test_self_worker_delayed_fail")
            .arg("--nocapture")
            .env("KGW_TEST_SELF_WORKER_DELAYED_FAIL_CHILD", "1")
            .env("KGW_TEST_SELF_WORKER_DELAYED_FAIL_STDERR", fail_line)
            .env("KGW_TEST_SELF_WORKER_ROLE", role)
            .env("KGW_TEST_SELF_WORKER_NETWORK", network)
            .env("KGW_TEST_SELF_WORKER_CONTROL_PATH", startup_control_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(command);
    }

    #[cfg(test)]
    if std::env::var_os("KGW_TEST_SELF_WORKER_COMMAND").is_some() {
        command
            .arg("--exact")
            .arg("integrated_runtime_commands::kgw_test_self_worker_hold")
            .arg("--nocapture")
            .env("KGW_TEST_SELF_WORKER_CHILD", "1")
            .env("KGW_TEST_SELF_WORKER_ROLE", role)
            .env("KGW_TEST_SELF_WORKER_NETWORK", network)
            .env("KGW_TEST_SELF_WORKER_CONTROL_PATH", startup_control_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(command);
    }

    command
        .arg("--kgw-self-worker")
        .arg(role)
        .arg("--network")
        .arg(network)
        .arg("--appdir")
        .arg(&settings.app_dir_name)
        .arg("--rpc")
        .arg(&settings.rpc_endpoint)
        .args(
            settings
                .p2p_listen
                .as_deref()
                .filter(|_| role == "node")
                .into_iter()
                .flat_map(|listen| ["--listen", listen]),
        )
        .arg("--startup-control-path")
        .arg(startup_control_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    Ok(command)
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests; unused in the library test target.
pub(crate) fn kgw_worker_node_command_args_for_test_v1(
    role: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
) -> Result<Vec<String>, String> {
    let normalized_role = role.trim().to_ascii_lowercase();
    let network = settings.network.as_str().to_string();
    let control_path = std::env::temp_dir()
        .join("KaspaGateway")
        .join("startup-control")
        .join("kgw-startup-control-args-test.json");
    let mut command = kgw_worker_command(&normalized_role, &network, settings, &control_path)?;

    if normalized_role == "node" {
        if settings.enable_utxo_index && !kgw_worker_uses_test_command() {
            command.arg("--utxoindex");
        }

        if settings.archival && !kgw_worker_uses_test_command() {
            command.arg("--archival");
        }
    }

    Ok(command
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect())
}

fn kgw_worker_role_from_request(
    runtime_role: Option<&str>,
    settings: &kaspa_gateway_rk_node::NodeSettings,
) -> String {
    if let Some(role) = runtime_role {
        let role = role.trim().to_ascii_lowercase();
        if role == "node" || role == "bridge" {
            return role;
        }
    }

    if settings.bridge_kind != kaspa_gateway_rk_node::BridgeNodeKind::Disable
        && settings.node_kind == kaspa_gateway_rk_node::KaspadNodeKind::Remote
    {
        return "bridge".to_string();
    }

    "node".to_string()
}

fn kgw_validate_network_start_policy(
    network: kaspa_gateway_rk_node::KgwNetwork,
    experimental_network_opt_in: Option<bool>,
) -> Result<(), String> {
    if network == kaspa_gateway_rk_node::KgwNetwork::Testnet12
        && experimental_network_opt_in != Some(true)
    {
        return Err(
            "start_blocked=true;start_allowed=false;network=testnet12;block_reason=experimental-network-opt-in-required;message=Testnet 12 is experimental and disabled by default. Enable it explicitly in this network tab before starting."
                .to_string(),
        );
    }

    Ok(())
}

fn kgw_worker_start(
    role: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
    bridge_instance_id: Option<String>,
    bridge_structured_instances: Option<String>,
    bridge_config_path: Option<String>,
    bridge_instance_listens_override: Option<Vec<String>>,
) -> Result<String, String> {
    let network = settings.network.as_str().to_string();
    let role = role.trim().to_ascii_lowercase();
    let key = kgw_worker_key(&role, &network);
    let bridge_node_mode =
        if settings.bridge_kind == kaspa_gateway_rk_node::BridgeNodeKind::OfficialInProcessNode {
            "inprocess"
        } else {
            "external"
        };
    let trace_node_mode = kgw_worker_node_mode_for_trace_v1(&role, bridge_node_mode);
    // Listener selection is management metadata only. Upstream Bridge logging
    // is process-wide and offers no structural record-to-listener attribution.
    let _ = bridge_instance_id;
    let has_bridge_config_path = bridge_config_path
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());

    kgw_start_trace_emit_v1(
        "native",
        "native.runtime_owner_check_started",
        &network,
        "start",
        "checking",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\"}}",
            role, trace_node_mode
        )),
    );

    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    if role == "bridge" && bridge_node_mode == "inprocess" {
        let node_key = kgw_worker_key("node", &network);
        let mut remove_stale_node = false;

        if let Some(existing) = workers.get_mut(&node_key) {
            let running = existing
                .child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_none();

            if running {
                kgw_start_trace_emit_v1(
                    "native",
                    "native.runtime_owner_checked",
                    &network,
                    "start",
                    "blocked",
                    Some(&format!(
                        "{{\"runtimeRole\":\"bridge\",\"blockReason\":\"node-tab-owner-running\",\"existingPid\":{}}}",
                        existing.child.id()
                    )),
                );
                return Err(format!(
                    "start_blocked=true;start_allowed=false;runtime_role=bridge;network={};node_mode=inprocess;block_reason=node-tab-owner-running;message=Cannot start bridge in-process because the same-network node is already running. Stop the node first or use external bridge mode.;node_pid={};appdir={}",
                    network,
                    existing.child.id(),
                    existing.appdir
                ));
            }

            let _ = existing.child.wait();
            remove_stale_node = true;
        }

        if remove_stale_node {
            workers.remove(&node_key);
        }
    }

    if role == "node" {
        let bridge_key = kgw_worker_key("bridge", &network);
        let mut remove_stale_bridge = false;

        if let Some(existing) = workers.get_mut(&bridge_key) {
            let running = existing
                .child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_none();

            if running && existing.node_mode == "inprocess" {
                kgw_start_trace_emit_v1(
                    "native",
                    "native.runtime_owner_checked",
                    &network,
                    "start",
                    "blocked",
                    Some(&format!(
                        "{{\"runtimeRole\":\"node\",\"blockReason\":\"bridge-inprocess-owner-running\",\"existingPid\":{}}}",
                        existing.child.id()
                    )),
                );
                return Err(format!(
                    "start_blocked=true;start_allowed=false;runtime_role=node;network={};node_mode=inprocess;block_reason=bridge-inprocess-owner-running;message=Cannot start node because bridge in-process owns this network. Stop the bridge first.;bridge_pid={};appdir={}",
                    network,
                    existing.child.id(),
                    existing.appdir
                ));
            }

            if !running {
                let _ = existing.child.wait();
                remove_stale_bridge = true;
            }
        }

        if remove_stale_bridge {
            workers.remove(&bridge_key);
        }
    }

    if let Some(existing) = workers.get_mut(&key) {
        if existing
            .child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_none()
        {
            kgw_start_trace_emit_v1(
                "native",
                "native.runtime_owner_checked",
                &network,
                "start",
                "blocked",
                Some(&format!(
                    "{{\"runtimeRole\":\"{}\",\"blockReason\":\"duplicate-owner\",\"existingPid\":{},\"nodeMode\":\"{}\"}}",
                    existing.role,
                    existing.child.id(),
                    existing.node_mode
                )),
            );
            return Err(format!(
                "start_blocked=true;start_allowed=false;block_reason=duplicate-owner;runtime_role={};network={};pid={};appdir={};node_mode={};readiness=READY;readiness_evidence={};message=Process owner already exists for this network and role.",
                existing.role,
                existing.network,
                existing.child.id(),
                existing.appdir,
                existing.node_mode,
                existing.readiness_evidence
            ));
        }

        let _ = existing.child.wait();
        workers.remove(&key);
    }

    kgw_start_trace_emit_v1(
        "native",
        "native.runtime_owner_checked",
        &network,
        "start",
        "available",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\"}}",
            role, trace_node_mode
        )),
    );

    let startup_control_path = kgw_worker_startup_control_path_v1(&role, &network);
    if let Some(parent) = startup_control_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create startup control directory failed {}: {error}",
                parent.display()
            )
        })?;
    }
    kgw_worker_remove_startup_control_v1(&startup_control_path);
    let mut command = kgw_worker_command(&role, &network, settings, &startup_control_path)?;

    if role == "bridge" {
        // KGW_BRIDGE_NORMAL_LOG_DEFAULT_R130
        // Normal bridge runs must not default to verbose protocol DEBUG logs.
        // Developers can still opt in explicitly by setting RUST_LOG before launch.
        let bridge_rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        command.env("RUST_LOG", bridge_rust_log);
    }

    let test_command = kgw_worker_uses_test_command();

    if role == "node" {
        if settings.enable_utxo_index && !test_command {
            command.arg("--utxoindex");
        }

        if settings.archival && !test_command {
            command.arg("--archival");
        }
    } else if !test_command {
        command.arg("--stratum").arg(&settings.stratum_listen);
        command.arg("--node-mode").arg(bridge_node_mode);

        // KGW_BRIDGE_DUAL_CLI_CONFIG_REAL_RUNNER_R122
        // Config route wins over UI/CLI instances to avoid mixing two instance sources.
        if let Some(config_path) = bridge_config_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            command.arg("--bridge-config").arg(config_path);
        } else {
            let bridge_instance_listens = bridge_instance_listens_override.unwrap_or_else(|| {
                kgw_bridge_structured_instance_listens_r120(bridge_structured_instances.as_deref())
            });

            for listen in &bridge_instance_listens {
                command.arg("--bridge-instance-listen").arg(listen);
            }
        }

        if bridge_node_mode == "inprocess" {
            if settings.enable_utxo_index {
                command.arg("--utxoindex");
            }

            if settings.archival {
                command.arg("--archival");
            }
        }

        if settings.bridge_internal_cpu_miner {
            command.arg("--internal-cpu-miner");

            if let Some(address) = &settings.bridge_internal_cpu_miner_address {
                command.arg("--internal-cpu-miner-address").arg(address);
            }

            if let Some(threads) = settings.bridge_internal_cpu_miner_threads {
                command
                    .arg("--internal-cpu-miner-threads")
                    .arg(threads.to_string());
            }

            if let Some(throttle_ms) = settings.bridge_internal_cpu_miner_throttle_ms {
                command
                    .arg("--internal-cpu-miner-throttle-ms")
                    .arg(throttle_ms.to_string());
            }

            if let Some(template_poll_ms) = settings.bridge_internal_cpu_miner_template_poll_ms {
                command
                    .arg("--internal-cpu-miner-template-poll-ms")
                    .arg(template_poll_ms.to_string());
            }
        }
    }

    let spawn_plan_details = kgw_worker_spawn_plan_details_v1(
        &role,
        &network,
        &trace_node_mode,
        settings,
        &command,
        test_command,
        has_bridge_config_path,
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.spawn_plan_created",
        &network,
        "start",
        "ok",
        Some(&spawn_plan_details),
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.spawn_attempted",
        &network,
        "start",
        "attempting",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\"}}",
            role, trace_node_mode
        )),
    );

    // Raw buffers are registered immediately before spawn and retained
    // independently of live child ownership. Genuine official output emitted
    // before startup failure or process exit remains queryable through logs IPC.
    let logs = kgw_worker_replace_raw_log_buffer_v1(&role, &network)?;
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            kgw_start_trace_emit_v1(
                "native",
                "native.spawn_failed",
                &network,
                "start",
                "error",
                Some(&format!(
                    "{{\"runtimeRole\":\"{}\",\"errorKind\":\"{:?}\"}}",
                    role,
                    error.kind()
                )),
            );
            return Err(format!(
                "spawn_failed=true;runtime_role={};network={};source=self-worker;error={}",
                role, network, error
            ));
        }
    };
    let pid = child.id();

    kgw_start_trace_emit_v1(
        "native",
        "native.spawn_succeeded",
        &network,
        "start",
        "ok",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{}}}",
            role, trace_node_mode, pid
        )),
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.child_pid_recorded",
        &network,
        "start",
        "ok",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{}}}",
            role, trace_node_mode, pid
        )),
    );

    let mut reader_handles = Vec::with_capacity(2);
    if let Some(stdout) = child.stdout.take() {
        reader_handles.push(kgw_worker_spawn_reader(
            role.clone(),
            network.clone(),
            None,
            "stdout".to_string(),
            pid,
            stdout,
            Arc::clone(&logs),
        ));
        kgw_start_trace_emit_v1(
            "native",
            "native.stdout_reader_attached",
            &network,
            "start",
            "attached",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    } else {
        kgw_start_trace_emit_v1(
            "native",
            "native.stdout_reader_attached",
            &network,
            "start",
            "missing",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    }

    if let Some(stderr) = child.stderr.take() {
        reader_handles.push(kgw_worker_spawn_reader(
            role.clone(),
            network.clone(),
            None,
            "stderr".to_string(),
            pid,
            stderr,
            Arc::clone(&logs),
        ));
        kgw_start_trace_emit_v1(
            "native",
            "native.stderr_reader_attached",
            &network,
            "start",
            "attached",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    } else {
        kgw_start_trace_emit_v1(
            "native",
            "native.stderr_reader_attached",
            &network,
            "start",
            "missing",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    }

    let readiness_evidence = match kgw_worker_wait_for_startup_attestation_v1(
        &mut child,
        &startup_control_path,
        &role,
        &network,
        &logs,
    ) {
        Ok(evidence) => evidence,
        Err(error) => {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
                let _ = child.wait();
            }
            kgw_worker_join_readers_v1(reader_handles);
            kgw_worker_remove_startup_control_v1(&startup_control_path);
            kgw_start_trace_emit_v1(
                "native",
                "native.startup_response_returned",
                &network,
                "start",
                "error",
                Some(&format!(
                    "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{},\"reason\":\"role-startup-attestation-failed\",\"error\":\"{}\"}}",
                    role,
                    trace_node_mode,
                    pid,
                    kgw_start_trace_json_escape_v1(&error)
                )),
            );
            return Err(error);
        }
    };

    if let Some(exit_status) = child.try_wait().map_err(|error| error.to_string())? {
        kgw_worker_join_readers_v1(reader_handles);
        kgw_start_trace_emit_v1(
            "native",
            "native.startup_response_returned",
            &network,
            "start",
            "error",
            Some(&format!(
                "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{},\"reason\":\"self-worker-exited-during-startup\",\"status\":\"{}\"}}",
                role, trace_node_mode, pid, exit_status
            )),
        );
        let captured = logs
            .lock()
            .map(|guard| {
                guard
                    .iter()
                    .map(|entry| kgw_worker_safe_child_line_v1(&entry.raw_text))
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .unwrap_or_else(|_| "worker log lock failed".to_string());

        // `ExitStatus` display text differs by platform. Keep it for diagnostics,
        // but expose a stable numeric field for callers and cross-platform tests.
        let exit_code = exit_status
            .code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string());

        return Err(format!(
            "self-worker exited during startup;role={};network={};pid={};status={};exit_code={};logs={}",
            role, network, pid, exit_status, exit_code, captured
        ));
    }

    let stored_node_mode = if role == "bridge" {
        bridge_node_mode.to_string()
    } else {
        trace_node_mode.clone()
    };

    workers.insert(
        key,
        KgwParallelSelfWorker {
            role: role.clone(),
            network: network.clone(),
            appdir: settings.app_dir_name.clone(),
            node_mode: stored_node_mode.clone(),
            child,
            reader_handles,
            _raw_logs: logs,
            started_ms: kgw_worker_now_ms(),
            exit_logged: false,
            readiness_evidence: readiness_evidence.clone(),
        },
    );

    Ok(format!(
        "parallel-owned-self-worker started;role={};network={};pid={};owner=self-worker;runtime_state=running;readiness=READY;readiness_evidence={};same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;appdir={};rpc={};stratum={};node_mode={};same_db_path=true;exclusive_node_owner_per_network=true",
        role,
        network,
        pid,
        readiness_evidence,
        settings.app_dir_name,
        settings.rpc_endpoint,
        settings.stratum_listen,
        stored_node_mode
    ))
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_raw_process_log_buffer_count_for_test_v1() -> usize {
    kgw_raw_process_log_buffers_v1()
        .lock()
        .map(|buffers| buffers.len())
        .unwrap_or_default()
}

fn kgw_worker_stop(network: &str, runtime_role: Option<&str>) -> Result<Option<String>, String> {
    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let wanted_role = runtime_role.map(|role| role.trim().to_ascii_lowercase());
    let wanted_network = network.trim().to_ascii_lowercase();

    let keys = workers
        .iter()
        .filter_map(|(key, worker)| {
            let role_match = wanted_role
                .as_ref()
                .map(|role| role == &worker.role)
                .unwrap_or(true);

            if worker.network == wanted_network && role_match {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if keys.is_empty() {
        return Ok(None);
    }

    let mut lines = Vec::new();

    for key in keys {
        if let Some(mut worker) = workers.remove(&key) {
            let pid = worker.child.id();
            let _ = worker.child.kill();
            let _ = worker.child.wait();
            kgw_worker_join_readers_v1(worker.reader_handles);

            lines.push(format!(
                "parallel-owned-self-worker stopped;role={};network={};pid={};appdir={};node_mode={}",
                worker.role, worker.network, pid, worker.appdir, worker.node_mode
            ));
        }
    }

    Ok(Some(lines.join("\n")))
}

#[tauri::command]
pub fn kgw_shutdown_all_runtime_workers_v1() -> Result<String, String> {
    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let keys = workers.keys().cloned().collect::<Vec<_>>();

    if keys.is_empty() {
        return Ok("parallel-owned-self-worker shutdown-all;stopped=0".to_string());
    }

    let mut stopped = Vec::new();

    for key in keys {
        if let Some(mut worker) = workers.remove(&key) {
            let pid = worker.child.id();
            let role = worker.role.clone();
            let network = worker.network.clone();

            let _ = worker.child.kill();
            let _ = worker.child.wait();
            kgw_worker_join_readers_v1(worker.reader_handles);

            stopped.push(format!(
                "parallel-owned-self-worker shutdown-all;role={};network={};pid={}",
                role, network, pid
            ));
        }
    }

    Ok(stopped.join("\n"))
}

fn kgw_worker_status(
    network: Option<&str>,
    runtime_role: Option<&str>,
) -> Result<Option<String>, String> {
    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());

    let mut lines = Vec::new();

    for worker in workers.values_mut() {
        if let Some(ref value) = wanted_network
            && &worker.network != value
        {
            continue;
        }

        if let Some(ref value) = wanted_role
            && &worker.role != value
        {
            continue;
        }

        let exit_status = worker.child.try_wait().map_err(|error| error.to_string())?;
        let running = exit_status.is_none();

        if let Some(status) = exit_status
            && !worker.exit_logged
        {
            kgw_start_trace_emit_v1(
                "native",
                "native.diagnostic_transport_process_exit",
                &worker.network,
                "status",
                "observed",
                Some(&format!(
                    "{{\"eventKind\":\"diagnostic_transport_record\",\"runtimeRole\":\"{}\",\"pid\":{},\"status\":\"{}\"}}",
                    kgw_start_trace_json_escape_v1(&worker.role),
                    worker.child.id(),
                    kgw_start_trace_json_escape_v1(&status.to_string())
                )),
            );
            worker.exit_logged = true;
        }

        lines.push(format!(
            "parallel-owned-self-worker status;role={};network={};pid={};running={};readiness={};readiness_evidence={};same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;appdir={};started_ms={};node_mode={}",
            worker.role,
            worker.network,
            worker.child.id(),
            running,
            if running { "READY" } else { "FAILED" },
            worker.readiness_evidence,
            worker.appdir,
            worker.started_ms,
            worker.node_mode
        ));
    }

    if lines.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lines.join("\n")))
    }
}

fn kgw_worker_logs(
    network: Option<&str>,
    runtime_role: Option<&str>,
    _bridge_instance_id: Option<&str>,
) -> Result<Option<KgwRuntimeLogsReportV1>, String> {
    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());

    let buffers = kgw_raw_process_log_buffers_v1()
        .lock()
        .map_err(|_| "raw process log buffer registry lock failed".to_string())?;
    let mut matched_buffer = false;
    let mut lines = Vec::new();

    for (key, logs) in buffers.iter() {
        let (role, network) = key.split_once(':').unwrap_or(("", ""));
        if wanted_network
            .as_deref()
            .is_some_and(|value| network != value)
            || wanted_role.as_deref().is_some_and(|value| role != value)
        {
            continue;
        }
        matched_buffer = true;
        if let Ok(logs) = logs.lock() {
            lines.extend(logs.iter().cloned());
        }
    }

    if !matched_buffer {
        return Ok(None);
    }

    lines.sort_by_key(|entry| entry.sequence);

    Ok(Some(KgwRuntimeLogsReportV1 {
        version: "kgw_runtime_logs_v1".to_string(),
        network: wanted_network,
        runtime_role: wanted_role,
        // IPC compatibility field only. Official Bridge records are emitted by
        // a process-wide logger and cannot be attributed to one listener.
        bridge_instance_id: None,
        source: "self-worker".to_string(),
        buffer_limit: KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1,
        entries: lines,
        diagnostics: Vec::new(),
    }))
}

fn kgw_worker_clear_logs(
    network: Option<&str>,
    runtime_role: Option<&str>,
    _bridge_instance_id: Option<&str>,
) -> Result<bool, String> {
    let buffers = kgw_raw_process_log_buffers_v1()
        .lock()
        .map_err(|_| "raw process log buffer registry lock failed".to_string())?;

    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());
    let mut matched_buffer = false;

    for (key, logs) in buffers.iter() {
        let (role, network) = key.split_once(':').unwrap_or(("", ""));
        if wanted_network
            .as_deref()
            .is_some_and(|value| network != value)
            || wanted_role.as_deref().is_some_and(|value| role != value)
        {
            continue;
        }

        matched_buffer = true;

        if let Ok(mut logs) = logs.lock() {
            logs.clear();
        }
    }

    Ok(matched_buffer)
}

fn kgw_empty_runtime_logs_report_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    source: &str,
    diagnostics: Vec<KgwRuntimeDiagnosticRecordV1>,
) -> KgwRuntimeLogsReportV1 {
    KgwRuntimeLogsReportV1 {
        version: "kgw_runtime_logs_v1".to_string(),
        network,
        runtime_role,
        bridge_instance_id,
        source: source.to_string(),
        buffer_limit: KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1,
        entries: Vec::new(),
        diagnostics,
    }
}

fn kgw_inprocess_diagnostic_logs_report_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    controller_logs: String,
) -> KgwRuntimeLogsReportV1 {
    let line_count = controller_logs.lines().count();
    let diagnostics = if line_count == 0 {
        Vec::new()
    } else {
        vec![KgwRuntimeDiagnosticRecordV1 {
            diagnostic_event: "kgw_diagnostic_transport_record_v1".to_string(),
            network: network.clone().unwrap_or_else(|| "unknown".to_string()),
            source: "in-process-controller".to_string(),
            runtime_role: runtime_role
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            bridge_instance_id: bridge_instance_id.clone(),
            received_ms: kgw_worker_now_ms_u64(),
            message: format!(
                "Controller diagnostics are available separately; no child stdout/stderr raw output is available for this component. diagnosticLineCount={line_count}"
            ),
        }]
    };

    kgw_empty_runtime_logs_report_v1(
        network,
        runtime_role,
        bridge_instance_id,
        "in-process-controller",
        diagnostics,
    )
}
fn parse_network(
    network: Option<String>,
) -> Result<Option<kaspa_gateway_rk_node::KgwNetwork>, String> {
    match network {
        Some(value) => kaspa_gateway_rk_node::KgwNetwork::parse(&value)
            .map(Some)
            .map_err(|error| error.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn kgw_runtime_owner_summary_v1() -> Result<String, String> {
    Ok(kaspa_gateway_rk_node::exact_kgw_service_controller_summary_v1().to_string())
}

#[tauri::command]
pub fn kgw_runtime_owner_status_v1(
    network: Option<String>,
    runtime_role: Option<String>,
) -> Result<String, String> {
    if let Some(worker_status) = kgw_worker_status(network.as_deref(), runtime_role.as_deref())? {
        return Ok(worker_status);
    }

    let requested_role = runtime_role
        .as_deref()
        .map(|role| role.trim().to_ascii_lowercase());

    if requested_role.as_deref() == Some("bridge") {
        let network_label = network
            .as_deref()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "all".to_string());

        return Ok(format!(
            "parallel-owned-self-worker status;role=bridge;network={};running=false;same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;node_mode=none;message=no bridge worker status yet",
            network_label
        ));
    }

    let network = parse_network(network)?;
    controller()
        .status(network)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_kgw_runtime_logs_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
) -> Result<KgwRuntimeLogsReportV1, String> {
    if let Some(worker_logs) = kgw_worker_logs(
        network.as_deref(),
        runtime_role.as_deref(),
        bridge_instance_id.as_deref(),
    )? {
        return Ok(worker_logs);
    }

    let bridge_instance_id = kgw_worker_clean_optional_identifier_v1(bridge_instance_id);
    let runtime_role = runtime_role
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let network_label = network
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let parsed_network = parse_network(network)?;
    let controller_logs = controller()
        .logs(parsed_network)
        .map_err(|error| error.to_string())?;

    Ok(kgw_inprocess_diagnostic_logs_report_v1(
        network_label,
        runtime_role,
        bridge_instance_id,
        controller_logs,
    ))
}

#[tauri::command]
pub fn kgw_kgw_runtime_clear_logs_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
) -> Result<KgwRuntimeLogsReportV1, String> {
    let _ = kgw_worker_clear_logs(
        network.as_deref(),
        runtime_role.as_deref(),
        bridge_instance_id.as_deref(),
    )?;

    let bridge_instance_id = kgw_worker_clean_optional_identifier_v1(bridge_instance_id);
    let runtime_role = runtime_role
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let network_label = network
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    Ok(kgw_empty_runtime_logs_report_v1(
        network_label,
        runtime_role,
        bridge_instance_id,
        "self-worker",
        Vec::new(),
    ))
}

fn kgw_safe_runtime_appdir_root() -> std::path::PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("KaspaGateway")
            .join("nodes")
    } else {
        std::env::temp_dir().join("KaspaGateway").join("nodes")
    }
}

fn kgw_network_runtime_appdir(network: kaspa_gateway_rk_node::KgwNetwork) -> String {
    kgw_safe_runtime_appdir_root()
        .join(network.as_str())
        .to_string_lossy()
        .to_string()
}

fn kgw_safe_runtime_appdir(value: String) -> String {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return kgw_safe_runtime_appdir_root()
            .join("default")
            .to_string_lossy()
            .to_string();
    }

    let input = std::path::PathBuf::from(trimmed);

    if input.is_absolute() {
        return input.to_string_lossy().to_string();
    }

    let safe_name = trimmed.replace(['\\', '/', ':'], "_");

    kgw_safe_runtime_appdir_root()
        .join(safe_name)
        .to_string_lossy()
        .to_string()
}
fn kgw_command_preview_find_cli_value(command_preview: &str, flag: &str) -> Option<String> {
    let mut parts = command_preview.split_whitespace().peekable();

    while let Some(part) = parts.next() {
        if let Some(value) = part.strip_prefix(&format!("{flag}=")) {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }

        if part == flag
            && let Some(value) = parts.peek()
        {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn kgw_command_preview_has_cli_flag(command_preview: &str, flag: &str) -> bool {
    command_preview
        .split_whitespace()
        .any(|part| part == flag || part.starts_with(&format!("{flag}=")))
}

fn kgw_bridge_preview_parse_u16(value: Option<String>) -> Option<u16> {
    value
        .and_then(|item| item.trim().parse::<u16>().ok())
        .filter(|item| *item > 0)
}

fn kgw_bridge_preview_parse_u64(value: Option<String>) -> Option<u64> {
    value
        .and_then(|item| item.trim().parse::<u64>().ok())
        .filter(|item| *item > 0)
}
fn kgw_command_preview_normalize_listen(value: String) -> String {
    let trimmed = value.trim().to_string();

    if trimmed.starts_with(':') {
        format!("0.0.0.0{}", trimmed)
    } else {
        trimmed
    }
}

// KGW_BRIDGE_DUAL_CLI_CONFIG_REAL_RUNNER_R122
// KGW_BRIDGE_PREVIEW_ALL_INSTANCES_R123
fn kgw_bridge_preview_instance_clause_listen_r123(instance_clause: &str) -> Option<String> {
    let clean = instance_clause.trim();

    if clean.is_empty() {
        return None;
    }

    for raw_part in clean.split(',') {
        let part = raw_part.trim();
        let Some((raw_key, raw_value)) = part.split_once('=').or_else(|| part.split_once(':'))
        else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_start_matches('-')
            .replace('-', "_")
            .to_ascii_lowercase();

        if !matches!(
            key.as_str(),
            "port" | "stratum" | "stratum_port" | "stratum_listen" | "listen"
        ) {
            continue;
        }

        let value = raw_value.trim().trim_matches('"').trim_matches('\'');

        if let Some(port) = kgw_bridge_normalize_instance_port_r110f(value) {
            return Some(kgw_command_preview_normalize_listen(port));
        }

        let tail = value.rsplit(':').next().unwrap_or(value).trim();
        if let Some(port) = kgw_bridge_normalize_instance_port_r110f(tail) {
            return Some(kgw_command_preview_normalize_listen(port));
        }
    }

    None
}

// KGW_BRIDGE_PREVIEW_ALL_INSTANCES_R123
fn kgw_bridge_command_preview_instance_listens_r123(command_preview: Option<&str>) -> Vec<String> {
    let Some(preview) = command_preview
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Vec::new();
    };

    let parts = preview.split_whitespace().collect::<Vec<_>>();
    let mut listens = Vec::<String>::new();
    let mut index = 0usize;

    while index < parts.len() {
        let part = parts[index];

        let value = if let Some(value) = part.strip_prefix("--instance=") {
            Some(value)
        } else if part == "--instance" {
            index += 1;
            parts.get(index).copied()
        } else {
            None
        };

        if let Some(instance_clause) = value
            && let Some(listen) = kgw_bridge_preview_instance_clause_listen_r123(instance_clause)
            && !listens.iter().any(|existing| existing == &listen)
        {
            listens.push(listen);
        }

        index += 1;
    }

    listens
}

fn kgw_bridge_config_path_from_preview_r122(command_preview: Option<&str>) -> Option<String> {
    let preview = command_preview?.trim();

    if preview.is_empty() {
        return None;
    }

    kgw_command_preview_find_cli_value(preview, "--config")
        .or_else(|| kgw_command_preview_find_cli_value(preview, "--bridge-config"))
        .or_else(|| kgw_command_preview_find_cli_value(preview, "--config-path"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn kgw_apply_command_preview_overrides(
    settings: &mut kaspa_gateway_rk_node::NodeSettings,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
) {
    if let Some(node_preview) = node_command_preview {
        if let Some(value) = kgw_command_preview_find_cli_value(&node_preview, "--listen") {
            settings.p2p_listen = Some(kgw_command_preview_normalize_listen(value));
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&node_preview, "--rpclisten") {
            settings.rpc_endpoint = kgw_command_preview_normalize_listen(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&node_preview, "--appdir") {
            settings.app_dir_name = kgw_safe_runtime_appdir(value);
        }

        settings.enable_utxo_index = kgw_command_preview_has_cli_flag(&node_preview, "--utxoindex");

        settings.archival = kgw_command_preview_has_cli_flag(&node_preview, "--archival");
    }

    if let Some(bridge_preview) = bridge_command_preview {
        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--kaspad-address")
            .or_else(|| kgw_command_preview_find_cli_value(&bridge_preview, "--kaspa-rpc"))
        {
            settings.rpc_endpoint = kgw_command_preview_normalize_listen(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--stratum-listen")
            .or_else(|| kgw_command_preview_find_cli_value(&bridge_preview, "--stratum-port"))
        {
            settings.stratum_listen = kgw_command_preview_normalize_listen(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--appdir")
            && !value.trim().is_empty()
        {
            settings.app_dir_name = kgw_safe_runtime_appdir(value);
        }

        // KGW_BRIDGE_INPROCESS_SAME_DB_OWNER_V7_INTEGRATED
        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--node-mode") {
            match value.trim().to_ascii_lowercase().as_str() {
                "inprocess" | "inproc" | "official-inprocess-node" => {
                    settings.node_kind = kaspa_gateway_rk_node::KaspadNodeKind::IntegratedInProc;
                    settings.bridge_kind =
                        kaspa_gateway_rk_node::BridgeNodeKind::OfficialInProcessNode;
                    settings.enable_utxo_index =
                        kgw_command_preview_has_cli_flag(&bridge_preview, "--utxoindex");
                    settings.archival =
                        kgw_command_preview_has_cli_flag(&bridge_preview, "--archival");
                }
                "external" | "external-node" | "official-external-node" => {
                    settings.node_kind = kaspa_gateway_rk_node::KaspadNodeKind::Remote;
                    settings.bridge_kind =
                        kaspa_gateway_rk_node::BridgeNodeKind::OfficialExternalNode;
                }
                _ => {}
            }
        }

        settings.bridge_internal_cpu_miner =
            kgw_command_preview_has_cli_flag(&bridge_preview, "--internal-cpu-miner");
        settings.bridge_internal_cpu_miner_address =
            kgw_command_preview_find_cli_value(&bridge_preview, "--internal-cpu-miner-address");
        settings.bridge_internal_cpu_miner_threads = kgw_bridge_preview_parse_u16(
            kgw_command_preview_find_cli_value(&bridge_preview, "--internal-cpu-miner-threads"),
        );
        settings.bridge_internal_cpu_miner_throttle_ms = kgw_bridge_preview_parse_u64(
            kgw_command_preview_find_cli_value(&bridge_preview, "--internal-cpu-miner-throttle-ms"),
        );
        settings.bridge_internal_cpu_miner_template_poll_ms =
            kgw_bridge_preview_parse_u64(kgw_command_preview_find_cli_value(
                &bridge_preview,
                "--internal-cpu-miner-template-poll-ms",
            ));
    }
}

// KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
fn kgw_bridge_instance_value_r110f(serialized: &str, wanted_key: &str) -> Option<String> {
    let wanted = wanted_key.trim().to_ascii_lowercase();

    for raw_part in serialized.split(',') {
        let part = raw_part.trim();
        let Some((raw_key, raw_value)) = part.split_once('=') else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_start_matches('-')
            .replace('-', "_")
            .to_ascii_lowercase();
        let value = raw_value.trim();

        if value.is_empty() {
            continue;
        }

        let matched = match wanted.as_str() {
            "port" => matches!(
                key.as_str(),
                "port" | "stratum" | "stratum_port" | "stratum_listen" | "listen"
            ),
            _ => key == wanted,
        };

        if matched {
            return Some(value.to_string());
        }
    }

    None
}

// KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
fn kgw_bridge_normalize_instance_port_r110f(value: &str) -> Option<String> {
    let clean = value.trim().trim_start_matches(':').trim();

    if clean.is_empty() {
        return None;
    }

    if !clean.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let Ok(port) = clean.parse::<u16>() else {
        return None;
    };

    if port == 0 {
        return None;
    }

    Some(format!(":{port}"))
}

// KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
// KGW_BRIDGE_DUAL_CLI_REAL_RUNNER_R120
fn kgw_bridge_normalize_instance_listen_r120(value: &str) -> Option<String> {
    let clean = value.trim().trim_matches('"').trim_matches('\'');

    if clean.is_empty() {
        return None;
    }

    if let Some(port) = kgw_bridge_normalize_instance_port_r110f(clean) {
        return Some(port);
    }

    let tail = clean.rsplit(':').next().unwrap_or(clean).trim();
    kgw_bridge_normalize_instance_port_r110f(tail)
}

// KGW_BRIDGE_DUAL_CLI_REAL_RUNNER_R120
fn kgw_bridge_instance_value_r120(serialized: &str, wanted_key: &str) -> Option<String> {
    let wanted = wanted_key.trim().replace('-', "_").to_ascii_lowercase();

    let normalized = serialized
        .replace(['{', '}', '[', ']', '"'], "")
        .replace("\\r", ",")
        .replace("\\n", ",");

    for raw_part in normalized.split([',', ';']) {
        let part = raw_part.trim();
        let Some((raw_key, raw_value)) = part.split_once('=').or_else(|| part.split_once(':'))
        else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_start_matches('-')
            .replace('-', "_")
            .to_ascii_lowercase();

        let value = raw_value.trim().trim_matches('\'');

        if value.is_empty() {
            continue;
        }

        let matched = match wanted.as_str() {
            "port" => matches!(
                key.as_str(),
                "port"
                    | "stratum"
                    | "stratum_port"
                    | "stratumport"
                    | "stratum_listen"
                    | "stratumlisten"
                    | "listen"
            ),
            _ => key == wanted,
        };

        if matched {
            return Some(value.to_string());
        }
    }

    None
}

// KGW_BRIDGE_DUAL_CLI_REAL_RUNNER_R120
fn kgw_bridge_structured_instance_listens_r120(serialized: Option<&str>) -> Vec<String> {
    let Some(raw) = serialized.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };

    let mut listens = Vec::<String>::new();

    for object_like in raw.split('{').skip(1) {
        let chunk = object_like.split('}').next().unwrap_or(object_like);

        if let Some(listen) = kgw_bridge_instance_value_r120(chunk, "port")
            .as_deref()
            .and_then(kgw_bridge_normalize_instance_listen_r120)
            && !listens.iter().any(|existing| existing == &listen)
        {
            listens.push(listen);
        }
    }

    if listens.is_empty()
        && let Some(listen) = kgw_bridge_instance_value_r120(raw, "port")
            .as_deref()
            .and_then(kgw_bridge_normalize_instance_listen_r120)
    {
        listens.push(listen);
    }

    listens
}

fn kgw_apply_bridge_active_instance_runtime_overrides_r110f(
    settings: &mut kaspa_gateway_rk_node::NodeSettings,
    bridge_active_instance_id: Option<String>,
    bridge_active_instance: Option<String>,
    bridge_active_instance_port: Option<String>,
    bridge_structured_instances: Option<String>,
) {
    let _ = bridge_active_instance_id;

    let structured_ports =
        kgw_bridge_structured_instance_listens_r120(bridge_structured_instances.as_deref());

    let explicit_port = bridge_active_instance_port
        .as_deref()
        .and_then(kgw_bridge_normalize_instance_port_r110f);

    let serialized_port = match bridge_active_instance.as_deref() {
        Some(serialized) => kgw_bridge_instance_value_r110f(serialized, "port")
            .as_deref()
            .and_then(kgw_bridge_normalize_instance_port_r110f),
        None => None,
    };

    if let Some(port) = explicit_port
        .or(serialized_port)
        .or_else(|| structured_ports.first().cloned())
    {
        settings.stratum_listen = kgw_command_preview_normalize_listen(port);
    }
}

#[allow(clippy::too_many_arguments)] // Stable Tauri IPC contract; grouping would break frontend argument names.
#[tauri::command]
pub fn kgw_kgw_apply_node_settings_v1(
    network: String,
    node_kind: String,
    bridge_kind: String,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
    runtime_role: Option<String>,
    bridge_active_instance_id: Option<String>,
    bridge_active_instance: Option<String>,
    bridge_active_instance_port: Option<String>,
    bridge_structured_instances: Option<String>,
    experimental_network_opt_in: Option<bool>,
) -> Result<String, String> {
    kgw_start_trace_emit_v1(
        "native",
        "native.tauri_command_entered",
        &network,
        "start",
        "entered",
        Some("{\"commandName\":\"kgw_kgw_apply_node_settings_v1\"}"),
    );
    kgw_start_trace_emit_v1(
        "native",
        "native.payload_received_and_validated",
        &network,
        "start",
        "received",
        Some(&format!(
            "{{\"hasNodeCommandPreview\":{},\"hasBridgeCommandPreview\":{},\"runtimeRole\":\"{}\",\"hasBridgeActiveInstance\":{},\"hasBridgeStructuredInstances\":{},\"experimentalOptIn\":{}}}",
            node_command_preview
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            bridge_command_preview
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            runtime_role.as_deref().unwrap_or(""),
            bridge_active_instance
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            bridge_structured_instances
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            experimental_network_opt_in == Some(true)
        )),
    );

    let mut settings =
        match kaspa_gateway_rk_node::NodeSettings::from_strings(network, node_kind, bridge_kind) {
            Ok(settings) => {
                kgw_start_trace_emit_v1(
                    "native",
                    "native.payload_received_and_validated",
                    settings.network.as_str(),
                    "start",
                    "validated",
                    Some("{\"schema\":\"node-settings-v1\"}"),
                );
                settings
            }
            Err(error) => {
                let error_text = error.to_string();
                kgw_start_trace_emit_v1(
                    "native",
                    "native.payload_received_and_validated",
                    "unknown",
                    "start",
                    "error",
                    Some("{\"schema\":\"node-settings-v1\"}"),
                );
                kgw_start_trace_emit_v1(
                    "native",
                    "native.startup_response_returned",
                    "unknown",
                    "start",
                    "error",
                    Some("{\"reason\":\"payload-validation-failed\"}"),
                );
                return Err(error_text);
            }
        };

    kgw_start_trace_emit_v1(
        "native",
        "native.network_normalized",
        settings.network.as_str(),
        "start",
        "ok",
        Some(&format!(
            "{{\"normalizedNetwork\":\"{}\"}}",
            settings.network.as_str()
        )),
    );

    let experimental_network = settings.network == kaspa_gateway_rk_node::KgwNetwork::Testnet12;
    let experimental_allowed = !experimental_network || experimental_network_opt_in == Some(true);
    kgw_start_trace_emit_v1(
        "native",
        "native.experimental_opt_in_evaluated",
        settings.network.as_str(),
        "start",
        if experimental_allowed {
            "allowed"
        } else {
            "blocked"
        },
        Some(&format!(
            "{{\"experimentalNetwork\":{},\"explicitOptIn\":{}}}",
            experimental_network,
            experimental_network_opt_in == Some(true)
        )),
    );

    if let Err(error) =
        kgw_validate_network_start_policy(settings.network, experimental_network_opt_in)
    {
        kgw_start_trace_emit_v1(
            "native",
            "native.startup_response_returned",
            settings.network.as_str(),
            "start",
            "error",
            Some("{\"reason\":\"experimental-network-policy\"}"),
        );
        return Err(error);
    }

    let bridge_config_path_for_worker =
        kgw_bridge_config_path_from_preview_r122(bridge_command_preview.as_deref());

    let bridge_instance_listens_from_preview =
        kgw_bridge_command_preview_instance_listens_r123(bridge_command_preview.as_deref());

    let bridge_instance_listens_override = if bridge_instance_listens_from_preview.is_empty() {
        None
    } else {
        Some(bridge_instance_listens_from_preview)
    };

    kgw_apply_command_preview_overrides(
        &mut settings,
        node_command_preview,
        bridge_command_preview,
    );
    settings.app_dir_name = kgw_network_runtime_appdir(settings.network);

    let bridge_active_instance_id_for_worker = bridge_active_instance_id.clone();
    let bridge_structured_instances_for_worker = bridge_structured_instances.clone();

    kgw_apply_bridge_active_instance_runtime_overrides_r110f(
        &mut settings,
        bridge_active_instance_id,
        bridge_active_instance,
        bridge_active_instance_port,
        bridge_structured_instances,
    );

    let role = kgw_worker_role_from_request(runtime_role.as_deref(), &settings);

    let result = kgw_worker_start(
        &role,
        &settings,
        bridge_active_instance_id_for_worker,
        bridge_structured_instances_for_worker,
        bridge_config_path_for_worker,
        bridge_instance_listens_override,
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.startup_response_returned",
        settings.network.as_str(),
        "start",
        if result.is_ok() { "ok" } else { "error" },
        Some(&format!("{{\"runtimeRole\":\"{}\"}}", role)),
    );

    result
}

#[tauri::command]
pub fn kgw_kgw_disable_network_v1(
    network: String,
    runtime_role: Option<String>,
) -> Result<String, String> {
    if let Some(stopped) = kgw_worker_stop(&network, runtime_role.as_deref())? {
        return Ok(stopped);
    }

    let network =
        kaspa_gateway_rk_node::KgwNetwork::parse(&network).map_err(|error| error.to_string())?;

    controller()
        .disable_network(network)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_runtime_owner_plan_v1(network: Option<String>) -> Result<String, String> {
    let network = network.unwrap_or_else(|| "mainnet".to_string());

    kaspa_gateway_rk_node::runtime_owner_plan_for_network_v1(Some(network))
        .map(|plan| plan.to_log_line())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_all_parallel_node_bridge_plans_v1() -> Result<String, String> {
    kaspa_gateway_rk_node::all_parallel_runtime_plans_log_v1().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_kgw_node_bridge_service_plan_v1(
    network: String,
    node_mode: String,
    bridge_mode: String,
    explicit_runtime_opt_in: bool,
) -> Result<String, String> {
    let node_mode = kaspa_gateway_rk_node::KaspaNodeRuntimeMode::parse(&node_mode)
        .map_err(|error| error.to_string())?;

    let bridge_mode = kaspa_gateway_rk_node::KaspaBridgeRuntimeMode::parse(&bridge_mode)
        .map_err(|error| error.to_string())?;

    let settings = kaspa_gateway_rk_node::KaspaRuntimeSettings {
        network,
        node_mode,
        bridge_mode,
        explicit_runtime_opt_in,
        ..kaspa_gateway_rk_node::KaspaRuntimeSettings::default()
    };

    let plan = kaspa_gateway_rk_node::build_official_kaspa_runtime_plan_v1(settings)
        .map_err(|error| error.to_string())?;

    Ok(plan.to_log_line())
}

#[tauri::command]
pub fn kgw_kgw_real_owner_summary_v1() -> Result<String, String> {
    Ok(kaspa_gateway_rk_node::real_owner_runtime_summary_v1().to_string())
}

#[tauri::command]
pub fn kgw_kgw_real_owner_feature_status_v1(network: String) -> Result<String, String> {
    let network =
        kaspa_gateway_rk_node::KgwNetwork::parse(&network).map_err(|error| error.to_string())?;

    let feature_status = kaspa_gateway_rk_node::KgwRuntimeFeatureStatus::for_network(network);

    Ok(feature_status.to_log_line())
}

#[tauri::command]
pub fn kgw_kgw_smoke_start_network_v1(network: String, bridge: bool) -> Result<String, String> {
    let bridge_kind = if bridge {
        "official-inprocess-node"
    } else {
        "disable"
    };

    let settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        network.clone(),
        "integrated-inproc".to_string(),
        bridge_kind.to_string(),
    )
    .map_err(|error| error.to_string())?;

    let accepted = controller()
        .apply_node_settings(settings)
        .map_err(|error| error.to_string())?;

    let status = controller()
        .status(Some(
            kaspa_gateway_rk_node::KgwNetwork::parse(&network)
                .map_err(|error| error.to_string())?,
        ))
        .map_err(|error| error.to_string())?;

    let logs = controller()
        .logs(Some(
            kaspa_gateway_rk_node::KgwNetwork::parse(&network)
                .map_err(|error| error.to_string())?,
        ))
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "smoke_start_network accepted\n{}\nstatus={}\nlogs=\n{}",
        accepted, status, logs
    ))
}

#[tauri::command]
pub fn kgw_kgw_smoke_stop_network_v1(network: String) -> Result<String, String> {
    let parsed_network =
        kaspa_gateway_rk_node::KgwNetwork::parse(&network).map_err(|error| error.to_string())?;

    let accepted = controller()
        .disable_network(parsed_network)
        .map_err(|error| error.to_string())?;

    let status = controller()
        .status(Some(parsed_network))
        .map_err(|error| error.to_string())?;

    let logs = controller()
        .logs(Some(parsed_network))
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "smoke_stop_network accepted\n{}\nstatus={}\nlogs=\n{}",
        accepted, status, logs
    ))
}

#[cfg(test)]
#[test]
fn kgw_test_self_worker_hold() {
    if std::env::var_os("KGW_TEST_SELF_WORKER_CHILD").is_none() {
        return;
    }

    let role = std::env::var("KGW_TEST_SELF_WORKER_ROLE").unwrap_or_else(|_| "node".to_string());
    let network =
        std::env::var("KGW_TEST_SELF_WORKER_NETWORK").unwrap_or_else(|_| "mainnet".to_string());

    let stdout = std::env::var("KGW_TEST_SELF_WORKER_STDOUT")
        .unwrap_or_else(|_| format!("test-self-worker stdout role={role} network={network}"));
    let stderr = std::env::var("KGW_TEST_SELF_WORKER_STDERR")
        .unwrap_or_else(|_| format!("test-self-worker stderr role={role} network={network}"));

    println!("{stdout}");
    eprintln!("{stderr}");

    let control_path = std::env::var("KGW_TEST_SELF_WORKER_CONTROL_PATH")
        .expect("test self-worker control path must be set");
    let delay_ms = std::env::var("KGW_TEST_SELF_WORKER_READY_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_default();
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    let evidence = format!(
        "{{\"version\":1,\"outcome\":\"READY\",\"runtimeRole\":\"{}\",\"network\":\"{}\",\"evidence\":\"test-role-ready\",\"error\":null}}",
        role, network
    );
    std::fs::write(control_path, evidence).expect("test self-worker must publish READY");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

#[cfg(test)]
#[test]
fn kgw_test_self_worker_fail() {
    if std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_CHILD").is_none() {
        return;
    }

    let role = std::env::var("KGW_TEST_SELF_WORKER_ROLE").unwrap_or_else(|_| "node".to_string());
    let network =
        std::env::var("KGW_TEST_SELF_WORKER_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    let line = std::env::var("KGW_TEST_SELF_WORKER_FAIL_STDERR").unwrap_or_else(|_| {
        format!("test-self-worker forced failure role={role} network={network}")
    });

    println!("test-self-worker failing stdout role={role} network={network}");
    eprintln!("{line}");
    std::process::exit(1);
}

#[cfg(test)]
#[test]
fn kgw_test_self_worker_delayed_fail() {
    if std::env::var_os("KGW_TEST_SELF_WORKER_DELAYED_FAIL_CHILD").is_none() {
        return;
    }

    let role = std::env::var("KGW_TEST_SELF_WORKER_ROLE").unwrap_or_else(|_| "node".to_string());
    let network =
        std::env::var("KGW_TEST_SELF_WORKER_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    let line = std::env::var("KGW_TEST_SELF_WORKER_DELAYED_FAIL_STDERR")
        .unwrap_or_else(|_| "delayed startup failure".to_string());
    let control_path = std::env::var("KGW_TEST_SELF_WORKER_CONTROL_PATH")
        .expect("test self-worker control path must be set");

    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let message = serde_json::json!({
        "version": 1,
        "outcome": "FAILED",
        "runtimeRole": role,
        "network": network,
        "evidence": null,
        "error": line,
    });
    std::fs::write(control_path, message.to_string())
        .expect("test self-worker must publish FAILED");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
