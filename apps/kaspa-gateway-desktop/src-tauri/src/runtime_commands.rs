use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealNodeRuntimeSettings {
    pub network: String,
    pub node_kind: String,
    pub bridge_kind: String,
    pub enable_utxo_index: bool,
    pub archival: bool,
}

impl Default for RealNodeRuntimeSettings {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            node_kind: "integrated-inproc".to_string(),
            bridge_kind: "official-inprocess-node".to_string(),
            enable_utxo_index: true,
            archival: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealBridgeRuntimeSettings {
    pub network: String,
    pub bridge_kind: String,
    pub bridge1_enabled: bool,
    pub bridge2_enabled: bool,
    pub bridge1_port: u16,
    pub bridge2_port: u16,
    pub extranonce: u8,
}

impl Default for RealBridgeRuntimeSettings {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            bridge_kind: "official-inprocess-node".to_string(),
            bridge1_enabled: true,
            bridge2_enabled: false,
            bridge1_port: 5555,
            bridge2_port: 5556,
            extranonce: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealNodeRuntimeStatus {
    pub network: String,
    pub branch: String,
    pub node_kind: String,
    pub bridge_kind: String,
    pub running: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealBridgeRuntimeStatus {
    pub network: String,
    pub branch: String,
    pub bridge_kind: String,
    pub running: bool,
    pub message: String,
}

fn normalize_network(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "mainnet" => "mainnet".to_string(),
        "testnet" | "testnet10" | "testnet-10" => "testnet10".to_string(),
        "testnet12" | "tn12" | "testnet-12" => "testnet12".to_string(),
        "simnet" => "simnet".to_string(),
        "devnet" => "devnet".to_string(),
        _ => "mainnet".to_string(),
    }
}

fn branch_for_network(network: &str) -> &'static str {
    match normalize_network(network).as_str() {
        "testnet12" => "tn12",
        _ => "master",
    }
}

fn rpc_for_network(network: &str) -> &'static str {
    match normalize_network(network).as_str() {
        "mainnet" => "127.0.0.1:16110",
        "testnet10" => "127.0.0.1:16210",
        "testnet12" => "127.0.0.1:16310",
        "simnet" => "127.0.0.1:16410",
        "devnet" => "127.0.0.1:16510",
        _ => "127.0.0.1:16110",
    }
}

fn stratum_for_network(network: &str) -> &'static str {
    match normalize_network(network).as_str() {
        "mainnet" => "0.0.0.0:5555",
        "testnet10" => "0.0.0.0:15555",
        "testnet12" => "0.0.0.0:25555",
        "simnet" => "0.0.0.0:35555",
        "devnet" => "0.0.0.0:45555",
        _ => "0.0.0.0:5555",
    }
}

fn validate_network(value: &str) -> Result<(), String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "mainnet" | "testnet" | "testnet10" | "testnet-10" | "testnet12" | "tn12"
        | "testnet-12" | "simnet" | "devnet" => Ok(()),
        other => Err(format!("Unsupported network: {other}")),
    }
}

fn validate_node_kind(value: &str) -> Result<(), String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "disable"
        | "disabled"
        | "integrated-inproc"
        | "integrated-as-daemon"
        | "integrated-as-passive-sync"
        | "remote" => Ok(()),
        other => Err(format!("Unsupported KGW node kind: {other}")),
    }
}

fn validate_bridge_kind(value: &str) -> Result<(), String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "disable" | "disabled" | "official-external-node" | "official-inprocess-node" => Ok(()),
        other => Err(format!("Unsupported KGW bridge kind: {other}")),
    }
}

fn validate_port(label: &str, value: u16) -> Result<(), String> {
    if value == 0 {
        return Err(format!("{label} port must be greater than zero."));
    }

    Ok(())
}

fn normalized_node_kind(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "disabled" => "disable".to_string(),
        "" => "integrated-inproc".to_string(),
        other => other.to_string(),
    }
}

fn normalized_bridge_kind(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "disabled" => "disable".to_string(),
        "" => "official-inprocess-node".to_string(),
        other => other.to_string(),
    }
}

fn node_command_preview(settings: &RealNodeRuntimeSettings) -> String {
    let network = normalize_network(&settings.network);
    let mut parts = vec!["kgw-owner:kaspad".to_string()];

    match network.as_str() {
        "testnet10" => {
            parts.push("--testnet".to_string());
            parts.push("--netsuffix=10".to_string());
        }
        "testnet12" => {
            parts.push("--testnet".to_string());
            parts.push("--netsuffix=12".to_string());
        }
        _ => {}
    }

    if settings.enable_utxo_index {
        parts.push("--utxoindex".to_string());
    }

    if settings.archival {
        parts.push("--archival".to_string());
    }

    parts.push(format!("--rpclisten={}", rpc_for_network(&network)));
    parts.push(format!("--appdir=kaspa-gateway-{network}"));

    parts.join(" ")
}

fn bridge_command_preview(settings: &RealBridgeRuntimeSettings) -> String {
    let network = normalize_network(&settings.network);

    [
        "kgw-owner:kaspa-stratum-bridge".to_string(),
        format!("--network={network}"),
        format!("--branch={}", branch_for_network(&network)),
        format!(
            "--node-kind={}",
            normalized_bridge_kind(&settings.bridge_kind)
        ),
        format!("--kaspa-rpc={}", rpc_for_network(&network)),
        format!("--stratum-listen={}", stratum_for_network(&network)),
    ]
    .join(" ")
}

fn validate_node_settings(settings: &RealNodeRuntimeSettings) -> Result<(), String> {
    validate_network(&settings.network)?;
    validate_node_kind(&settings.node_kind)?;
    validate_bridge_kind(&settings.bridge_kind)?;

    let node_kind = normalized_node_kind(&settings.node_kind);
    let bridge_kind = normalized_bridge_kind(&settings.bridge_kind);

    if bridge_kind == "official-inprocess-node" && node_kind != "integrated-inproc" {
        return Err("KGW in-process bridge requires integrated-inproc node kind.".to_string());
    }

    Ok(())
}

fn validate_bridge_settings(settings: &RealBridgeRuntimeSettings) -> Result<(), String> {
    validate_network(&settings.network)?;
    validate_bridge_kind(&settings.bridge_kind)?;
    validate_port("Bridge 1", settings.bridge1_port)?;
    validate_port("Bridge 2", settings.bridge2_port)?;

    if settings.extranonce > 16 {
        return Err("Extranonce must be between 0 and 16.".to_string());
    }

    if settings.bridge1_enabled
        && settings.bridge2_enabled
        && settings.bridge1_port == settings.bridge2_port
    {
        return Err(
            "Bridge 1 and Bridge 2 cannot use the same port while both are enabled.".to_string(),
        );
    }

    Ok(())
}

#[tauri::command]
pub fn real_node_default_runtime_settings() -> Result<RealNodeRuntimeSettings, String> {
    Ok(RealNodeRuntimeSettings::default())
}

#[tauri::command]
pub fn real_bridge_default_runtime_settings() -> Result<RealBridgeRuntimeSettings, String> {
    Ok(RealBridgeRuntimeSettings::default())
}

#[tauri::command]
pub fn real_node_runtime_command_preview(
    settings: RealNodeRuntimeSettings,
) -> Result<String, String> {
    validate_node_settings(&settings)?;

    Ok(node_command_preview(&settings))
}

#[tauri::command]
pub fn real_bridge_runtime_command_preview(
    settings: RealBridgeRuntimeSettings,
) -> Result<String, String> {
    validate_bridge_settings(&settings)?;

    Ok(bridge_command_preview(&settings))
}

#[tauri::command]
pub fn real_node_runtime_status(network: Option<String>) -> Result<RealNodeRuntimeStatus, String> {
    let network = normalize_network(network.as_deref().unwrap_or("mainnet"));
    validate_network(&network)?;

    Ok(RealNodeRuntimeStatus {
        branch: branch_for_network(&network).to_string(),
        network,
        node_kind: "kgw-controller-owned".to_string(),
        bridge_kind: "kgw-controller-owned".to_string(),
        running: false,
        message: "Status is owned by KGW service controller; no executable path is used."
            .to_string(),
    })
}

#[tauri::command]
pub fn real_bridge_runtime_status(
    network: Option<String>,
) -> Result<RealBridgeRuntimeStatus, String> {
    let network = normalize_network(network.as_deref().unwrap_or("mainnet"));
    validate_network(&network)?;

    Ok(RealBridgeRuntimeStatus {
        branch: branch_for_network(&network).to_string(),
        network,
        bridge_kind: "kgw-controller-owned".to_string(),
        running: false,
        message: "Bridge status is owned by KGW service controller; no external bridge executable path is used."
            .to_string(),
    })
}

#[tauri::command]
pub fn real_node_runtime_apply_settings(
    settings: RealNodeRuntimeSettings,
) -> Result<String, String> {
    validate_node_settings(&settings)?;

    Ok(format!(
        "kgw apply_node_settings only;network={};branch={};node_kind={};bridge_kind={};preview={}",
        normalize_network(&settings.network),
        branch_for_network(&settings.network),
        normalized_node_kind(&settings.node_kind),
        normalized_bridge_kind(&settings.bridge_kind),
        node_command_preview(&settings)
    ))
}

#[tauri::command]
pub fn real_bridge_runtime_apply_settings(
    settings: RealBridgeRuntimeSettings,
) -> Result<String, String> {
    validate_bridge_settings(&settings)?;

    Ok(format!(
        "kgw bridge settings only;network={};branch={};bridge_kind={};preview={}",
        normalize_network(&settings.network),
        branch_for_network(&settings.network),
        normalized_bridge_kind(&settings.bridge_kind),
        bridge_command_preview(&settings)
    ))
}

#[tauri::command]
pub fn tail_process_log(network: Option<String>) -> Result<String, String> {
    let network = normalize_network(network.as_deref().unwrap_or("mainnet"));
    Ok(format!(
        "KGW owner logs are exposed by kgw_kgw_runtime_logs_v1; network={}; branch={}; no process log file is tailed.",
        network,
        branch_for_network(&network)
    ))
}
#[tauri::command]
pub fn real_node_runtime_report(network: Option<String>) -> Result<RealNodeRuntimeStatus, String> {
    real_node_runtime_status(network)
}
#[tauri::command]
pub fn real_bridge_runtime_report(
    network: Option<String>,
) -> Result<RealBridgeRuntimeStatus, String> {
    real_bridge_runtime_status(network)
}
