use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static KGW_CONTROLLER: OnceLock<Arc<kaspa_gateway_rk_node::KgwServiceController>> = OnceLock::new();

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
    logs: Arc<Mutex<VecDeque<String>>>,
    started_ms: u128,
}

static KGW_PARALLEL_SELF_WORKERS: OnceLock<Mutex<HashMap<String, KgwParallelSelfWorker>>> =
    OnceLock::new();

fn kgw_parallel_self_workers() -> &'static Mutex<HashMap<String, KgwParallelSelfWorker>> {
    KGW_PARALLEL_SELF_WORKERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn kgw_worker_now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn kgw_worker_key(role: &str, network: &str) -> String {
    format!(
        "{}:{}",
        role.trim().to_ascii_lowercase(),
        network.trim().to_ascii_lowercase()
    )
}

fn kgw_worker_push_log(logs: &Arc<Mutex<VecDeque<String>>>, line: impl Into<String>) {
    let line = line.into().trim_end_matches('\r').to_string();

    if line.trim().is_empty() {
        return;
    }

    if let Ok(mut guard) = logs.lock() {
        if guard.len() >= 4096 {
            guard.pop_front();
        }

        guard.push_back(line);
    }
}

fn kgw_worker_spawn_reader<R>(_label: String, reader: R, logs: Arc<Mutex<VecDeque<String>>>)
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let buffered = BufReader::new(reader);

        for line in buffered.lines() {
            match line {
                Ok(line) => kgw_worker_push_log(&logs, line),
                Err(_error) => {
                    break;
                }
            }
        }
    });
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

fn kgw_worker_start(
    role: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
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
                return Ok(format!(
                    "start_blocked=true;start_allowed=false;runtime_role=bridge;network={};node_mode=inprocess;block_reason=node-tab-owner-running;message=Cannot start bridge in-process because the same-network node is already running. Stop the node first or use external bridge mode.;node_pid={};appdir={}",
                    network,
                    existing.child.id(),
                    existing.appdir
                ));
            }

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
                return Ok(format!(
                    "start_blocked=true;start_allowed=false;runtime_role=node;network={};node_mode=inprocess;block_reason=bridge-inprocess-owner-running;message=Cannot start node because bridge in-process owns this network. Stop the bridge first.;bridge_pid={};appdir={}",
                    network,
                    existing.child.id(),
                    existing.appdir
                ));
            }

            if !running {
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
            return Ok(format!(
                "parallel-owned-self-worker already running;role={};network={};pid={};appdir={};node_mode={}",
                existing.role,
                existing.network,
                existing.child.id(),
                existing.appdir,
                existing.node_mode
            ));
        }

        workers.remove(&key);
    }

    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let logs = Arc::new(Mutex::new(VecDeque::new()));

    let mut command = Command::new(exe);

    command
        .arg("--kgw-self-worker")
        .arg(&role)
        .arg("--network")
        .arg(&network)
        .arg("--appdir")
        .arg(&settings.app_dir_name)
        .arg("--rpc")
        .arg(&settings.rpc_endpoint)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    if role == "bridge" {
        let bridge_rust_log = std::env::var("RUST_LOG")
            .unwrap_or_else(|_| "info,kaspa_stratum_bridge=debug".to_string());
        command.env("RUST_LOG", bridge_rust_log);
    }

    if role == "node" {
        if settings.enable_utxo_index {
            command.arg("--utxoindex");
        }

        if settings.archival {
            command.arg("--archival");
        }
    } else {
        command.arg("--stratum").arg(&settings.stratum_listen);
        command.arg("--node-mode").arg(bridge_node_mode);

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

    let mut child = command.spawn().map_err(|error| error.to_string())?;

    if let Some(stdout) = child.stdout.take() {
        kgw_worker_spawn_reader("stdout".to_string(), stdout, Arc::clone(&logs));
    }

    if let Some(stderr) = child.stderr.take() {
        kgw_worker_spawn_reader("stderr".to_string(), stderr, Arc::clone(&logs));
    }

    let pid = child.id();
    let stored_node_mode = if role == "bridge" {
        bridge_node_mode.to_string()
    } else {
        "node".to_string()
    };

    workers.insert(
        key,
        KgwParallelSelfWorker {
            role: role.clone(),
            network: network.clone(),
            appdir: settings.app_dir_name.clone(),
            node_mode: stored_node_mode.clone(),
            child,
            logs,
            started_ms: kgw_worker_now_ms(),
        },
    );

    Ok(format!(
        "parallel-owned-self-worker started;role={};network={};pid={};same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;appdir={};rpc={};stratum={};node_mode={};same_db_path=true;exclusive_node_owner_per_network=true",
        role,
        network,
        pid,
        settings.app_dir_name,
        settings.rpc_endpoint,
        settings.stratum_listen,
        stored_node_mode
    ))
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

            lines.push(format!(
                "parallel-owned-self-worker stopped;role={};network={};pid={};appdir={};node_mode={}",
                worker.role, worker.network, pid, worker.appdir, worker.node_mode
            ));
        }
    }

    Ok(Some(lines.join("\n")))
}

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
        if let Some(ref value) = wanted_network {
            if &worker.network != value {
                continue;
            }
        }

        if let Some(ref value) = wanted_role {
            if &worker.role != value {
                continue;
            }
        }

        let running = worker
            .child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_none();

        lines.push(format!(
            "parallel-owned-self-worker status;role={};network={};pid={};running={};same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;appdir={};started_ms={};node_mode={}",
            worker.role,
            worker.network,
            worker.child.id(),
            running,
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
) -> Result<Option<String>, String> {
    let workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());

    let mut matched_worker = false;
    let mut lines = Vec::new();

    for worker in workers.values() {
        if let Some(ref value) = wanted_network {
            if &worker.network != value {
                continue;
            }
        }

        if let Some(ref value) = wanted_role {
            if &worker.role != value {
                continue;
            }
        }

        matched_worker = true;

        if let Ok(logs) = worker.logs.lock() {
            lines.extend(logs.iter().cloned());
        }
    }

    if lines.is_empty() {
        if matched_worker {
            Ok(Some(String::new()))
        } else {
            Ok(None)
        }
    } else {
        Ok(Some(lines.join("\n")))
    }
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
) -> Result<String, String> {
    if let Some(worker_logs) = kgw_worker_logs(network.as_deref(), runtime_role.as_deref())? {
        return Ok(worker_logs);
    }
    let network = parse_network(network)?;
    controller()
        .logs(network)
        .map_err(|error| error.to_string())
}

fn kgw_safe_runtime_appdir_root() -> std::path::PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data).join("rusty-kaspa")
    } else {
        std::env::temp_dir().join("rusty-kaspa")
    }
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

    let safe_name = trimmed
        .replace('\\', "_")
        .replace('/', "_")
        .replace(':', "_");

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

        if part == flag {
            if let Some(value) = parts.peek() {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(value.to_string());
                }
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

fn kgw_apply_command_preview_overrides(
    settings: &mut kaspa_gateway_rk_node::NodeSettings,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
) {
    if let Some(node_preview) = node_command_preview {
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

        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--appdir") {
            if !value.trim().is_empty() {
                settings.app_dir_name = kgw_safe_runtime_appdir(value);
            }
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

#[tauri::command]
pub fn kgw_kgw_apply_node_settings_v1(
    network: String,
    node_kind: String,
    bridge_kind: String,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
    runtime_role: Option<String>,
) -> Result<String, String> {
    let mut settings =
        kaspa_gateway_rk_node::NodeSettings::from_strings(network, node_kind, bridge_kind)
            .map_err(|error| error.to_string())?;

    kgw_apply_command_preview_overrides(
        &mut settings,
        node_command_preview,
        bridge_command_preview,
    );

    let role = kgw_worker_role_from_request(runtime_role.as_deref(), &settings);

    kgw_worker_start(&role, &settings)
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
