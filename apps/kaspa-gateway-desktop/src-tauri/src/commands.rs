// ============================================================================
// KGW_OWNERSHIP_COMMANDS_GENERAL_ADAPTER_ONLY
// General command adapter file.
// Do not add transaction fetch/sync ownership here.
// Transaction orchestration belongs to crates/kaspa-gateway-runtime/src/transaction_sync.rs
// ============================================================================

use crate::app_logger;
use kaspa_gateway_api::{AddressNameRecord, ApiClientConfig, KaspaApiClient};
use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::{AppInfo, KaspaAddress};
use kaspa_gateway_db::{AddressRecord, DatabaseManager, DatabasePaths};
use kaspa_gateway_node::{NodeCapabilityManager, NodeManagerConfig};
use kaspa_gateway_runtime::runtime_check_default;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
pub struct DesktopRuntimeState {
    kgw_node_running: Mutex<bool>,
    kgw_bridge_running: Mutex<bool>,
    integrated_bridge_running: Mutex<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopAppInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopRuntimeReport {
    pub lines: Vec<String>,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopAddressRecord {
    pub address: String,
    pub name: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopUiSection {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopDashboardReport {
    pub app: DesktopAppInfo,
    pub runtime: DesktopRuntimeReport,
    pub api_network_url: String,
    pub node_capabilities: String,
    pub saved_addresses_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopExplorerBalanceReport {
    pub address: String,
    pub masked_address: String,
    pub balance_sompi: u64,
    pub balance_kas: f64,
    pub api_url: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopNetworkAnalyticsReport {
    pub hashrate: Option<Value>,
    pub network: Option<Value>,
    pub blockdag: Option<Value>,
    pub coin_supply: Option<Value>,
    pub halving: Option<Value>,
    pub block_reward: Option<Value>,
    pub max_hashrate: Option<Value>,
    pub summary_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopNodeSettings {
    pub network: String,
    pub rpc_host: String,
    pub rpc_port: u16,
    pub utxo_index: bool,
    pub auto_start: bool,
    pub auto_restart: bool,
}

impl Default for DesktopNodeSettings {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            rpc_host: "127.0.0.1".to_string(),
            rpc_port: 16110,
            utxo_index: true,
            auto_start: false,
            auto_restart: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopNodeSettingsReport {
    pub settings: DesktopNodeSettings,
    pub command_preview: String,
    pub capability_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopProcessReport {
    pub status: String,
    pub message: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DesktopBridgeMode {
    IntegratedBridge,
    OfficialExternalNode,
    RemoteEndpoint,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopBridgeSettings {
    pub mode: DesktopBridgeMode,

    pub bridge_host: String,
    pub bridge1_enabled: bool,
    pub bridge1_port: u16,
    pub bridge2_enabled: bool,
    pub bridge2_port: u16,
    pub node_rpc_host: String,
    pub node_rpc_port: u16,
    pub extra_args: String,
}

impl Default for DesktopBridgeSettings {
    fn default() -> Self {
        Self {
            mode: DesktopBridgeMode::IntegratedBridge,
            bridge_host: "127.0.0.1".to_string(),
            bridge1_enabled: true,
            bridge1_port: 5555,
            bridge2_enabled: false,
            bridge2_port: 5556,
            node_rpc_host: "127.0.0.1".to_string(),
            node_rpc_port: 16110,
            extra_args: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopBridgeReport {
    pub settings: DesktopBridgeSettings,
    pub command_preview: String,
    pub capability_status: String,
    pub status: String,
}

#[tauri::command]
pub fn desktop_ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub fn app_info() -> DesktopAppInfo {
    let info = AppInfo::default();

    DesktopAppInfo {
        name: info.name,
        version: info.version,
    }
}

#[tauri::command]
pub fn runtime_check() -> Result<DesktopRuntimeReport, String> {
    let report = runtime_check_default().map_err(|error| error.to_string())?;

    Ok(DesktopRuntimeReport {
        lines: report.to_lines(),
        healthy: report.is_healthy(),
    })
}

#[tauri::command]
pub fn api_network_url() -> Result<String, String> {
    let client =
        KaspaApiClient::new(ApiClientConfig::default()).map_err(|error| error.to_string())?;

    client
        .network_info_url()
        .map(|url| url.to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn settings_fetch_address_names() -> Result<Vec<AddressNameRecord>, String> {
    let client =
        KaspaApiClient::new(ApiClientConfig::default()).map_err(|error| error.to_string())?;

    client
        .fetch_address_names()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn node_capabilities() -> Result<String, String> {
    let config = NodeManagerConfig::default();

    NodeCapabilityManager::inspect(&config)
        .map(|capabilities| capabilities.describe())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn dashboard_report() -> Result<DesktopDashboardReport, String> {
    let app = app_info();
    let runtime = runtime_check()?;
    let api_network_url = api_network_url()?;
    let node_capabilities = node_capabilities()?;
    let saved_addresses_count = list_addresses()?.len();

    Ok(DesktopDashboardReport {
        app,
        runtime,
        api_network_url,
        node_capabilities,
        saved_addresses_count,
    })
}

#[tauri::command]
pub fn ui_sections() -> Vec<DesktopUiSection> {
    vec![
        DesktopUiSection {
            id: "dashboard".to_string(),
            title: "Dashboard".to_string(),
            description: "Runtime overview, node status, API readiness, and quick health checks.".to_string(),
        },
        DesktopUiSection {
            id: "node".to_string(),
            title: "Kaspa Node".to_string(),
            description: "Node settings, command preview, version status, logs, and lifecycle controls.".to_string(),
        },
        DesktopUiSection {
            id: "bridge".to_string(),
            title: "Kaspa Bridge".to_string(),
            description: "Integrated bridge capability, compatibility bridge options, bridge instances, and logs.".to_string(),
        },
        DesktopUiSection {
            id: "explorer".to_string(),
            title: "Explorer".to_string(),
            description: "Fetch balances, transactions, and network data for selected Kaspa addresses.".to_string(),
        },
        DesktopUiSection {
            id: "top-addresses".to_string(),
            title: "Top Addresses".to_string(),
            description: "Rich list, address names, rank search, and known address analysis.".to_string(),
        },
        DesktopUiSection {
            id: "addresses".to_string(),
            title: "Manage Addresses".to_string(),
            description: "Save, list, import, export, and manage known Kaspa addresses.".to_string(),
        },
        DesktopUiSection {
            id: "analytics".to_string(),
            title: "Analytics".to_string(),
            description: "Network hashrate, difficulty, BlockDAG, supply, halving, and block reward.".to_string(),
        },
        DesktopUiSection {
            id: "settings".to_string(),
            title: "Settings".to_string(),
            description: "Main settings, external APIs, displayed tabs, currencies, languages, and file paths.".to_string(),
        },
        DesktopUiSection {
            id: "logs".to_string(),
            title: "Logs".to_string(),
            description: "Live log, severity filtering, search, copy, and diagnostics.".to_string(),
        },
        DesktopUiSection {
            id: "export".to_string(),
            title: "Export".to_string(),
            description: "Save reports and address data as CSV, HTML, or PDF.".to_string(),
        },
        DesktopUiSection {
            id: "about".to_string(),
            title: "About".to_string(),
            description: "Version, project information, donation address, and credits.".to_string(),
        },
    ]
}

#[tauri::command]
pub fn list_addresses() -> Result<Vec<DesktopAddressRecord>, String> {
    crate::db_state::with_database_manager("addresses.list", |manager| {
        let repository = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        let records = repository.list().map_err(|error| error.to_string())?;

        Ok(records
            .into_iter()
            .map(|record| DesktopAddressRecord {
                address: record.address,
                name: record.name,
                network: record.network,
            })
            .collect())
    })
}

#[tauri::command]
pub fn add_address(address: String, name: String) -> Result<String, String> {
    app_logger::log_info(
        "addresses",
        &format!("add_address requested address={}", address),
    );

    let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;
    let clean_name = sanitize_optional_label(&name);

    crate::db_state::with_database_manager("addresses.add", |manager| {
        let repository = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        let record = AddressRecord::new(parsed.as_str(), clean_name, "mainnet")
            .map_err(|error| error.to_string())?;

        repository
            .upsert(&record)
            .map_err(|error| error.to_string())?;

        Ok("Address saved successfully.".to_string())
    })
}

#[tauri::command]
pub fn delete_address(address: String) -> Result<String, String> {
    app_logger::log_info(
        "addresses",
        &format!("delete_address requested address={}", address),
    );

    let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;
    let canonical_address = parsed.as_str().to_string();

    crate::db_state::with_database_manager("addresses.delete_with_transactions", |manager| {
        let addresses_repository = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        let deleted_address = addresses_repository
            .delete(&canonical_address)
            .map_err(|error| error.to_string())?;

        let transactions_repository = manager
            .transactions_repository()
            .map_err(|error| error.to_string())?;

        let deleted_transactions = transactions_repository
            .delete_for_address(&canonical_address)
            .map_err(|error| error.to_string())?;

        app_logger::log_info(
            "addresses",
            &format!(
                "delete_address cleanup ok address={} deleted_address={} deleted_transactions={}",
                canonical_address, deleted_address, deleted_transactions
            ),
        );

        Ok(format!(
            "Address deleted successfully. Deleted cached transactions: {}.",
            deleted_transactions
        ))
    })
}

#[tauri::command]
pub fn rename_address(address: String, name: String) -> Result<String, String> {
    app_logger::log_info(
        "addresses",
        &format!("rename_address requested address={}", address),
    );

    let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;
    let clean_name = sanitize_optional_label(&name);

    crate::db_state::with_database_manager("addresses.rename", |manager| {
        let repository = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        let record = AddressRecord::new(parsed.as_str(), clean_name, "mainnet")
            .map_err(|error| error.to_string())?;

        repository
            .upsert(&record)
            .map_err(|error| error.to_string())?;

        Ok("Address renamed successfully.".to_string())
    })
}

#[tauri::command]
pub async fn explorer_fetch_balance(
    address: String,
) -> Result<DesktopExplorerBalanceReport, String> {
    let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;

    let client =
        KaspaApiClient::new(ApiClientConfig::default()).map_err(|error| error.to_string())?;

    let api_url = client
        .address_balance_url(parsed.as_str())
        .map_err(|error| error.to_string())?
        .to_string();

    let balance_sompi = client
        .fetch_address_balance_sompi(parsed.as_str())
        .await
        .map_err(|error| error.to_string())?;

    let balance_kas = balance_sompi as f64 / 100_000_000.0;

    Ok(DesktopExplorerBalanceReport {
        address: parsed.as_str().to_string(),
        masked_address: parsed.masked(),
        balance_sompi,
        balance_kas,
        api_url,
        status: "Fetch completed.".to_string(),
    })
}

async fn fetch_optional_json(client: &KaspaApiClient, path: &str) -> Option<Value> {
    client.get_json::<Value>(path).await.ok()
}

fn format_status_line(label: &str, value: &Option<Value>) -> String {
    match value {
        Some(value) => format!("{label}=ok:{value}"),
        None => format!("{label}=unavailable"),
    }
}
#[tauri::command]
pub async fn network_analytics_report() -> Result<DesktopNetworkAnalyticsReport, String> {
    let api_config = ApiClientConfig::default();
    let endpoints = api_config.endpoints.clone();

    let client = KaspaApiClient::new(api_config).map_err(|error| error.to_string())?;

    let hashrate = fetch_optional_json(&client, &endpoints.hashrate).await;
    let network = fetch_optional_json(&client, &endpoints.network).await;
    let blockdag = fetch_optional_json(&client, &endpoints.blockdag_info).await;
    let coin_supply = fetch_optional_json(&client, &endpoints.coinsupply).await;
    let halving = fetch_optional_json(&client, &endpoints.halving).await;
    let block_reward = fetch_optional_json(&client, &endpoints.blockreward).await;
    let max_hashrate = fetch_optional_json(&client, &endpoints.max_hashrate).await;

    let summary_lines = vec![
        format_status_line("hashrate", &hashrate),
        format_status_line("network", &network),
        format_status_line("blockdag", &blockdag),
        format_status_line("coinsupply", &coin_supply),
        format_status_line("halving", &halving),
        format_status_line("block_reward", &block_reward),
        format_status_line("max_hashrate", &max_hashrate),
    ];

    Ok(DesktopNetworkAnalyticsReport {
        hashrate,
        network,
        blockdag,
        coin_supply,
        halving,
        block_reward,
        max_hashrate,
        summary_lines,
    })
}

#[tauri::command]
pub fn load_node_settings() -> Result<DesktopNodeSettingsReport, String> {
    let settings = load_node_settings_from_db()?;
    build_node_settings_report(settings)
}

#[tauri::command]
pub fn save_node_settings(
    settings: DesktopNodeSettings,
) -> Result<DesktopNodeSettingsReport, String> {
    validate_node_settings(&settings)?;

    let manager = database_manager()?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repository
        .set(
            "node.settings.json",
            &serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    build_node_settings_report(settings)
}

#[tauri::command]
pub fn node_command_preview(
    settings: DesktopNodeSettings,
) -> Result<DesktopNodeSettingsReport, String> {
    validate_node_settings(&settings)?;
    build_node_settings_report(settings)
}

#[tauri::command]
pub fn node_start(
    state: State<'_, DesktopRuntimeState>,
    settings: DesktopNodeSettings,
) -> Result<DesktopProcessReport, String> {
    validate_node_settings(&settings)?;

    let mut running = state
        .kgw_node_running
        .lock()
        .map_err(|_| "Failed to lock KGW node state.".to_string())?;

    *running = true;

    Ok(DesktopProcessReport {
        status: "accepted".to_string(),
        message: format!(
            "KGW NodeSettings accepted; network={}; flow=NodeSettings -> KaspadServiceEvents::from_node_settings -> service_events.sender -> controller",
            normalize_desktop_network(&settings.network)
        ),
        pid: None,
    })
}
#[tauri::command]
pub fn node_stop(state: State<'_, DesktopRuntimeState>) -> Result<DesktopProcessReport, String> {
    let mut running = state
        .kgw_node_running
        .lock()
        .map_err(|_| "Failed to lock KGW node state.".to_string())?;

    *running = false;

    Ok(DesktopProcessReport {
        status: "stopped".to_string(),
        message: "KGW Disable event accepted for node owner.".to_string(),
        pid: None,
    })
}
#[tauri::command]
pub fn node_status(state: State<'_, DesktopRuntimeState>) -> Result<DesktopProcessReport, String> {
    let running = state
        .kgw_node_running
        .lock()
        .map_err(|_| "Failed to lock KGW node state.".to_string())?;

    Ok(DesktopProcessReport {
        status: if *running { "accepted" } else { "stopped" }.to_string(),
        message: "Node lifecycle is owned by KGW controller; no KGW owner path is used."
            .to_string(),
        pid: None,
    })
}
#[tauri::command]
pub fn load_bridge_settings(
    state: State<'_, DesktopRuntimeState>,
) -> Result<DesktopBridgeReport, String> {
    let settings = load_bridge_settings_from_db()?;
    build_bridge_report(settings, &state)
}

#[tauri::command]
pub fn save_bridge_settings(
    state: State<'_, DesktopRuntimeState>,
    settings: DesktopBridgeSettings,
) -> Result<DesktopBridgeReport, String> {
    validate_bridge_settings(&settings)?;

    let manager = database_manager()?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repository
        .set(
            "bridge.settings.json",
            &serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    build_bridge_report(settings, &state)
}

#[tauri::command]
pub fn bridge_command_preview(
    state: State<'_, DesktopRuntimeState>,
    settings: DesktopBridgeSettings,
) -> Result<DesktopBridgeReport, String> {
    validate_bridge_settings(&settings)?;
    build_bridge_report(settings, &state)
}

#[tauri::command]
pub fn bridge_start(
    state: State<'_, DesktopRuntimeState>,
    settings: DesktopBridgeSettings,
) -> Result<DesktopProcessReport, String> {
    validate_bridge_settings(&settings)?;

    let mut running = state
        .kgw_bridge_running
        .lock()
        .map_err(|_| "Failed to lock KGW bridge state.".to_string())?;

    *running = settings.mode != DesktopBridgeMode::Disabled;

    Ok(DesktopProcessReport {
        status: if *running { "accepted" } else { "disabled" }.to_string(),
        message:
            "KGW bridge settings accepted; bridge lifecycle is owned by the service controller."
                .to_string(),
        pid: None,
    })
}
#[tauri::command]
pub fn bridge_stop(state: State<'_, DesktopRuntimeState>) -> Result<DesktopProcessReport, String> {
    {
        let mut integrated = state
            .integrated_bridge_running
            .lock()
            .map_err(|_| "Failed to lock integrated bridge state.".to_string())?;
        *integrated = false;
    }

    let mut running = state
        .kgw_bridge_running
        .lock()
        .map_err(|_| "Failed to lock KGW bridge state.".to_string())?;

    *running = false;

    Ok(DesktopProcessReport {
        status: "stopped".to_string(),
        message: "KGW bridge Disable event accepted.".to_string(),
        pid: None,
    })
}
#[tauri::command]
pub fn bridge_status(
    state: State<'_, DesktopRuntimeState>,
) -> Result<DesktopProcessReport, String> {
    let running = state
        .kgw_bridge_running
        .lock()
        .map_err(|_| "Failed to lock KGW bridge state.".to_string())?;

    Ok(DesktopProcessReport {
        status: if *running { "accepted" } else { "stopped" }.to_string(),
        message: "Bridge lifecycle is owned by KGW controller; no KGW bridge path is used."
            .to_string(),
        pid: None,
    })
}
fn bridge_status_from_state(
    state: &State<'_, DesktopRuntimeState>,
) -> Result<DesktopProcessReport, String> {
    let running = state
        .kgw_bridge_running
        .lock()
        .map_err(|_| "Failed to lock KGW bridge state.".to_string())?;

    Ok(DesktopProcessReport {
        status: if *running { "accepted" } else { "stopped" }.to_string(),
        message: "Bridge status is delegated to KGW service controller.".to_string(),
        pid: None,
    })
}
fn load_node_settings_from_db() -> Result<DesktopNodeSettings, String> {
    let manager = database_manager()?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    match repository
        .get("node.settings.json")
        .map_err(|error| error.to_string())?
    {
        Some(value) => {
            serde_json::from_str::<DesktopNodeSettings>(&value).map_err(|error| error.to_string())
        }
        None => Ok(DesktopNodeSettings::default()),
    }
}

fn normalize_desktop_network(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "mainnet" => "mainnet".to_string(),
        "testnet" | "testnet10" | "testnet-10" => "testnet10".to_string(),
        "testnet12" | "tn12" | "testnet-12" => "testnet12".to_string(),
        "simnet" => "simnet".to_string(),
        "devnet" => "devnet".to_string(),
        _ => "mainnet".to_string(),
    }
}

fn branch_for_desktop_network(network: &str) -> &'static str {
    match normalize_desktop_network(network).as_str() {
        "testnet12" => "tn12",
        _ => "master",
    }
}

fn node_rpc_endpoint(settings: &DesktopNodeSettings) -> String {
    format!("{}:{}", settings.rpc_host, settings.rpc_port)
}

fn build_node_settings_report(
    settings: DesktopNodeSettings,
) -> Result<DesktopNodeSettingsReport, String> {
    validate_node_settings(&settings)?;

    let network = normalize_desktop_network(&settings.network);
    let mut preview = vec!["kgw-owner:kaspad".to_string()];

    match network.as_str() {
        "testnet10" => {
            preview.push("--testnet".to_string());
            preview.push("--netsuffix=10".to_string());
        }
        "testnet12" => {
            preview.push("--testnet".to_string());
            preview.push("--netsuffix=12".to_string());
        }
        _ => {}
    }

    if settings.utxo_index {
        preview.push("--utxoindex".to_string());
    }

    preview.push(format!("--rpclisten={}", node_rpc_endpoint(&settings)));
    preview.push(format!("--appdir=kaspa-gateway-{network}"));

    Ok(DesktopNodeSettingsReport {
        settings,
        command_preview: preview.join(" "),
        capability_status: format!(
            "KGW owner flow only; branch={}; no KGW owner path; no KGW owner.",
            branch_for_desktop_network(&network)
        ),
    })
}

fn validate_node_settings(settings: &DesktopNodeSettings) -> Result<(), String> {
    match normalize_desktop_network(&settings.network).as_str() {
        "mainnet" | "testnet10" | "testnet12" | "simnet" | "devnet" => {}
        other => return Err(format!("Unsupported network selected: {other}")),
    }

    validate_host(&settings.rpc_host)?;

    if settings.rpc_port == 0 {
        return Err("RPC port cannot be zero.".to_string());
    }

    Ok(())
}
fn load_bridge_settings_from_db() -> Result<DesktopBridgeSettings, String> {
    let manager = database_manager()?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    match repository
        .get("bridge.settings.json")
        .map_err(|error| error.to_string())?
    {
        Some(value) => {
            serde_json::from_str::<DesktopBridgeSettings>(&value).map_err(|error| error.to_string())
        }
        None => Ok(DesktopBridgeSettings::default()),
    }
}

fn build_bridge_report(
    settings: DesktopBridgeSettings,
    state: &State<'_, DesktopRuntimeState>,
) -> Result<DesktopBridgeReport, String> {
    validate_bridge_settings(&settings)?;

    let mode = match settings.mode {
        DesktopBridgeMode::IntegratedBridge => "official-inprocess-node",
        DesktopBridgeMode::OfficialExternalNode => "official-external-node",
        DesktopBridgeMode::RemoteEndpoint => "official-external-node",
        DesktopBridgeMode::Disabled => "disable",
    };

    let command_preview = format!(
        "kgw-owner:kaspa-stratum-bridge --network=mainnet --branch=master --node-kind={} --kaspa-rpc={}:{} --stratum-listen={}:{}",
        mode,
        settings.node_rpc_host,
        settings.node_rpc_port,
        settings.bridge_host,
        settings.bridge1_port
    );

    let status = bridge_status_from_state(state)?.message;

    Ok(DesktopBridgeReport {
        settings,
        command_preview,
        capability_status:
            "Bridge lifecycle is owned by KGW service controller; no KGW bridge path.".to_string(),
        status,
    })
}

fn validate_bridge_settings(settings: &DesktopBridgeSettings) -> Result<(), String> {
    validate_host(&settings.bridge_host)?;
    validate_host(&settings.node_rpc_host)?;

    if settings.bridge1_port == 0 {
        return Err("Bridge 1 port cannot be zero.".to_string());
    }

    if settings.bridge2_enabled && settings.bridge2_port == 0 {
        return Err("Bridge 2 port cannot be zero.".to_string());
    }

    if settings.node_rpc_port == 0 {
        return Err("Node RPC port cannot be zero.".to_string());
    }

    if settings.bridge1_enabled
        && settings.bridge2_enabled
        && settings.bridge1_port == settings.bridge2_port
    {
        return Err("Bridge 1 and Bridge 2 cannot use the same port.".to_string());
    }

    if settings.extra_args.contains('\0')
        || settings.extra_args.contains('\n')
        || settings.extra_args.contains('\r')
        || settings.extra_args.contains("&&")
        || settings.extra_args.contains("||")
        || settings.extra_args.contains(';')
        || settings.extra_args.contains('|')
    {
        return Err("Unsafe bridge argument text rejected.".to_string());
    }

    Ok(())
}
fn validate_host(host: &str) -> Result<(), String> {
    if host.trim().is_empty() {
        return Err("Host cannot be empty.".to_string());
    }

    if host.contains('\0')
        || host.contains('/')
        || host.contains('\\')
        || host.contains('?')
        || host.contains('#')
        || host.contains('@')
        || host.contains(' ')
        || host.contains('\n')
        || host.contains('\r')
    {
        return Err("Host contains unsafe characters.".to_string());
    }

    Ok(())
}

fn sanitize_optional_label(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !ch.is_control())
        .take(120)
        .collect()
}
fn database_manager() -> Result<DatabaseManager, String> {
    let root = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("databases");

    let paths = DatabasePaths::new(root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths);

    manager
        .initialize_all()
        .map_err(|error| error.to_string())?;

    Ok(manager)
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    let allowed = [
        "https://github.com/KaspaPulse",
        "https://kaspa.stream/addresses/kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g",
        "https://x.com/KaspaPulse",
    ];

    if !allowed.iter().any(|candidate| *candidate == url) {
        return Err(format!("External URL is not allowed: {}", url));
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|error| format!("Failed to open external URL on Windows: {}", error))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|error| format!("Failed to open external URL on macOS: {}", error))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|error| format!("Failed to open external URL on Linux: {}", error))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err("Opening external URLs is not supported on this platform".to_string())
}
