use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeRuntimeNetwork {
    Mainnet,
    Testnet10,
    Testnet12,
}

impl BridgeRuntimeNetwork {
    pub fn parse(value: &str) -> Result<Self, BridgeRuntimeError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" | "testnet10" => Ok(Self::Testnet10),
            "tn12" | "testnet12" => Ok(Self::Testnet12),
            _ => Err(BridgeRuntimeError::UnsupportedNetwork(value.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet10 => "testnet10",
            Self::Testnet12 => "testnet12",
        }
    }

        pub fn branch(self) -> &'static str {
        match self {
            Self::Mainnet => "master",
            Self::Testnet10 | Self::Testnet12 => "RKStratumTN12",
        }
    }

        pub fn revision(self) -> &'static str {
        match self {
            Self::Mainnet => "4969c6c31bec6a1a31e17c2e8a43532e17834240",
            Self::Testnet10 | Self::Testnet12 => "f197b5ca00a92825f9962db1b3ed0f5d00455464",
        }
    }

        pub fn family(self) -> BridgeRuntimeFamily {
        match self {
            Self::Mainnet => BridgeRuntimeFamily::Mainline,
            Self::Testnet10 | Self::Testnet12 => BridgeRuntimeFamily::Tn12,
        }
    }

    pub fn default_rpc(self) -> &'static str {
        match self {
            Self::Mainnet => "127.0.0.1:16110",
            Self::Testnet10 => "127.0.0.1:16210",
            Self::Testnet12 => "127.0.0.1:16310",
        }
    }

    pub fn default_stratum(self) -> &'static str {
        match self {
            Self::Mainnet => "0.0.0.0:5555",
            Self::Testnet10 => "0.0.0.0:15555",
            Self::Testnet12 => "0.0.0.0:25555",
        }
    }

    pub fn default_prometheus(self) -> &'static str {
        match self {
            Self::Mainnet => "127.0.0.1:2114",
            Self::Testnet10 => "127.0.0.1:12114",
            Self::Testnet12 => "127.0.0.1:22114",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeRuntimeFamily {
    Mainline,
    Tn12,
}

impl BridgeRuntimeFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainline => "mainline-master",
            Self::Tn12 => "tn12-only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeRuntimeMode {
    Disabled,
    OfficialExternalNode,
    OfficialInProcessNode,
}

impl BridgeRuntimeMode {
    pub fn parse(value: &str) -> Result<Self, BridgeRuntimeError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "external" | "external-node" | "official-external-node" => {
                Ok(Self::OfficialExternalNode)
            }
            "inproc" | "inprocess" | "inprocess-node" | "official-inprocess-node" => {
                Ok(Self::OfficialInProcessNode)
            }
            _ => Err(BridgeRuntimeError::UnsupportedBridgeMode(value.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::OfficialExternalNode => "official-external-node",
            Self::OfficialInProcessNode => "official-inprocess-node",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeServiceEventKind {
    Stop,
    StartOfficialExternalNode,
    StartOfficialInProcessNode,
    Stdout,
    Status,
}

impl BridgeServiceEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::StartOfficialExternalNode => "start-official-external-node",
            Self::StartOfficialInProcessNode => "start-official-inprocess-node",
            Self::Stdout => "stdout",
            Self::Status => "status",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeRuntimeStep {
    ConvertBridgeSettingsToDecision,
    SelectOfficialBranchByNetwork,
    ConvertDecisionToServiceEvent,
    ValidateNetworkPortsAndMode,
    AttachToOfficialKaspaNodeRpc,
    StartOfficialBridgeWithExternalNode,
    StartOfficialBridgeWithInProcessNode,
    RelayBridgeStdoutToLogs,
    PublishBridgeStatus,
    StopBridgeOwner,
}

impl BridgeRuntimeStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConvertBridgeSettingsToDecision => "convert-bridge-settings-to-decision",
            Self::SelectOfficialBranchByNetwork => "select-official-branch-by-network",
            Self::ConvertDecisionToServiceEvent => "convert-decision-to-service-event",
            Self::ValidateNetworkPortsAndMode => "validate-network-ports-and-mode",
            Self::AttachToOfficialKaspaNodeRpc => "attach-to-official-kaspa-node-rpc",
            Self::StartOfficialBridgeWithExternalNode => "start-official-bridge-with-external-node",
            Self::StartOfficialBridgeWithInProcessNode => {
                "start-official-bridge-with-inprocess-node"
            }
            Self::RelayBridgeStdoutToLogs => "relay-bridge-stdout-to-logs",
            Self::PublishBridgeStatus => "publish-bridge-status",
            Self::StopBridgeOwner => "stop-bridge-owner",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BridgeInternalCpuMinerSettings {
    pub enabled: bool,
    pub address: Option<String>,
    pub threads: Option<u16>,
    pub throttle_ms: Option<u64>,
    pub template_poll_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeRuntimeSettings {
    pub network: String,
    pub mode: BridgeRuntimeMode,
    pub stratum_listen: Option<String>,
    pub prometheus_listen: Option<String>,
    pub kaspa_rpc_endpoint: Option<String>,
    pub internal_cpu_miner: BridgeInternalCpuMinerSettings,
    pub explicit_runtime_opt_in: bool,
}

impl Default for BridgeRuntimeSettings {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            mode: BridgeRuntimeMode::Disabled,
            stratum_listen: None,
            prometheus_listen: None,
            kaspa_rpc_endpoint: None,
            internal_cpu_miner: BridgeInternalCpuMinerSettings::default(),
            explicit_runtime_opt_in: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeServiceEvent {
    pub kind: BridgeServiceEventKind,
    pub network: BridgeRuntimeNetwork,
    pub family: BridgeRuntimeFamily,
    pub branch: &'static str,
    pub mode: BridgeRuntimeMode,
    pub stratum_listen: String,
    pub prometheus_listen: String,
    pub kaspa_rpc_endpoint: String,
    pub internal_cpu_miner: BridgeInternalCpuMinerSettings,
}

impl BridgeServiceEvent {
    pub fn to_log_line(&self) -> String {
        format!(
            "event={};network={};family={};branch={};mode={};stratum_listen={};prometheus_listen={};kaspa_rpc_endpoint={};internal_cpu_miner_enabled={};internal_cpu_miner_address={}",
            self.kind.as_str(),
            self.network.as_str(),
            self.family.as_str(),
            self.branch,
            self.mode.as_str(),
            self.stratum_listen,
            self.prometheus_listen,
            self.kaspa_rpc_endpoint,
            self.internal_cpu_miner.enabled,
            self.internal_cpu_miner.address.as_deref().unwrap_or("")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeRuntimePlan {
    pub owner_crate: &'static str,
    pub official_repository: &'static str,
    pub official_package: &'static str,
    pub network: BridgeRuntimeNetwork,
    pub family: BridgeRuntimeFamily,
    pub branch: &'static str,
    pub mode: BridgeRuntimeMode,
    pub service_event: BridgeServiceEvent,
    pub feature_expected: &'static str,
    pub feature_enabled: bool,
    pub starts_now: bool,
    pub runs_parallel_safe: bool,
    pub uses_local_clone: bool,
    pub uses_packaged_binary: bool,
    pub frontend_owns_start: bool,
    pub steps: Vec<BridgeRuntimeStep>,
    pub decision: String,
}

impl BridgeRuntimePlan {
    pub fn to_log_line(&self) -> String {
        let steps = self
            .steps
            .iter()
            .map(|step| step.as_str())
            .collect::<Vec<_>>()
            .join(",");

        format!(
            "owner={};repo={};package={};network={};family={};branch={};mode={};event={};feature_expected={};feature_enabled={};starts_now={};parallel_safe={};local_clone={};packaged_binary={};frontend_start={};steps={};decision={}",
            self.owner_crate,
            self.official_repository,
            self.official_package,
            self.network.as_str(),
            self.family.as_str(),
            self.branch,
            self.mode.as_str(),
            self.service_event.to_log_line(),
            self.feature_expected,
            self.feature_enabled,
            self.starts_now,
            self.runs_parallel_safe,
            self.uses_local_clone,
            self.uses_packaged_binary,
            self.frontend_owns_start,
            steps,
            self.decision
        )
    }
}

#[derive(Debug, Error)]
pub enum BridgeRuntimeError {
    #[error("unsupported network: {0}")]
    UnsupportedNetwork(String),

    #[error("unsupported bridge mode: {0}")]
    UnsupportedBridgeMode(String),

    #[error("invalid listen address")]
    InvalidListenAddress,

    #[error("bridge in-process mode requires same-owner in-process node")]
    InProcessBridgeRequiresInProcessNode,

    #[error("bridge runtime feature is required: {0}")]
    FeatureRequired(String),

    #[error("bridge owner thread start failed: {0}")]
    OwnerThreadStartFailed(String),
}

pub struct BridgeOwnerRuntimeHandle {
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    owner_thread: JoinHandle<()>,
}

impl std::fmt::Debug for BridgeOwnerRuntimeHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BridgeOwnerRuntimeHandle")
            .field("owner_thread_finished", &self.owner_thread.is_finished())
            .finish()
    }
}

impl BridgeOwnerRuntimeHandle {
    pub fn request_stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    pub fn is_finished(&self) -> bool {
        self.owner_thread.is_finished()
    }
}

fn validate_listen(value: &str) -> Result<(), BridgeRuntimeError> {
    if value.trim().is_empty()
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(BridgeRuntimeError::InvalidListenAddress);
    }

    Ok(())
}

pub fn bridge_service_event_from_settings_v1(
    settings: BridgeRuntimeSettings,
) -> Result<BridgeServiceEvent, BridgeRuntimeError> {
    let network = BridgeRuntimeNetwork::parse(&settings.network)?;

    let stratum_listen = settings
        .stratum_listen
        .unwrap_or_else(|| network.default_stratum().to_string());

    let prometheus_listen = settings
        .prometheus_listen
        .unwrap_or_else(|| network.default_prometheus().to_string());

    let kaspa_rpc_endpoint = settings
        .kaspa_rpc_endpoint
        .unwrap_or_else(|| network.default_rpc().to_string());

    validate_listen(&stratum_listen)?;
    validate_listen(&prometheus_listen)?;
    validate_listen(&kaspa_rpc_endpoint)?;

    let kind = match settings.mode {
        BridgeRuntimeMode::Disabled => BridgeServiceEventKind::Stop,
        BridgeRuntimeMode::OfficialExternalNode => {
            BridgeServiceEventKind::StartOfficialExternalNode
        }
        BridgeRuntimeMode::OfficialInProcessNode => {
            BridgeServiceEventKind::StartOfficialInProcessNode
        }
    };

    Ok(BridgeServiceEvent {
        kind,
        network,
        family: network.family(),
        branch: network.branch(),
        mode: settings.mode,
        stratum_listen,
        prometheus_listen,
        kaspa_rpc_endpoint,
        internal_cpu_miner: settings.internal_cpu_miner,
    })
}

pub fn official_bridge_plan_from_settings_v1(
    settings: BridgeRuntimeSettings,
    node_is_inprocess_owner: bool,
) -> Result<BridgeRuntimePlan, BridgeRuntimeError> {
    if settings.mode == BridgeRuntimeMode::OfficialInProcessNode && !node_is_inprocess_owner {
        return Err(BridgeRuntimeError::InProcessBridgeRequiresInProcessNode);
    }

    let explicit_runtime_opt_in = settings.explicit_runtime_opt_in;
    let event = bridge_service_event_from_settings_v1(settings)?;
    let network = event.network;

    let feature_expected = match network.family() {
        BridgeRuntimeFamily::Mainline => "official-kaspa-runtime-mainline",
        BridgeRuntimeFamily::Tn12 => "official-kaspa-runtime-tn12",
    };

    let feature_enabled = match network.family() {
        BridgeRuntimeFamily::Mainline => cfg!(feature = "official-kaspa-runtime-mainline"),
        BridgeRuntimeFamily::Tn12 => cfg!(feature = "official-kaspa-runtime-tn12"),
    };

    let starts_now =
        explicit_runtime_opt_in && event.mode != BridgeRuntimeMode::Disabled && feature_enabled;

    let decision = if !explicit_runtime_opt_in {
        "bridge plan only; explicit opt-in is false; no bridge is started"
    } else {
        match event.mode {
            BridgeRuntimeMode::Disabled => "bridge disabled by settings",
            BridgeRuntimeMode::OfficialExternalNode => {
                "official bridge external-node mode selected; owner can start listener"
            }
            BridgeRuntimeMode::OfficialInProcessNode => {
                "official bridge in-process-node mode selected; owner can start listener against owner RPC"
            }
        }
    };

    Ok(BridgeRuntimePlan {
        owner_crate: "kaspa-gateway-rk-bridge",
        official_repository: "https://github.com/kaspanet/rusty-kaspa.git",
        official_package: "kaspa-stratum-bridge",
        network,
        family: network.family(),
        branch: network.branch(),
        mode: event.mode,
        service_event: event,
        feature_expected,
        feature_enabled,
        starts_now,
        runs_parallel_safe: true,
        uses_local_clone: false,
        uses_packaged_binary: false,
        frontend_owns_start: false,
        steps: vec![
            BridgeRuntimeStep::ConvertBridgeSettingsToDecision,
            BridgeRuntimeStep::SelectOfficialBranchByNetwork,
            BridgeRuntimeStep::ConvertDecisionToServiceEvent,
            BridgeRuntimeStep::ValidateNetworkPortsAndMode,
            BridgeRuntimeStep::AttachToOfficialKaspaNodeRpc,
            BridgeRuntimeStep::StartOfficialBridgeWithExternalNode,
            BridgeRuntimeStep::StartOfficialBridgeWithInProcessNode,
            BridgeRuntimeStep::RelayBridgeStdoutToLogs,
            BridgeRuntimeStep::PublishBridgeStatus,
            BridgeRuntimeStep::StopBridgeOwner,
        ],
        decision: decision.to_string(),
    })
}

pub fn start_official_bridge_owner_thread_v1(
    event: BridgeServiceEvent,
) -> Result<BridgeOwnerRuntimeHandle, BridgeRuntimeError> {
    match event.kind {
        BridgeServiceEventKind::Stop
        | BridgeServiceEventKind::Stdout
        | BridgeServiceEventKind::Status => {
            return Err(BridgeRuntimeError::UnsupportedBridgeMode(
                event.kind.as_str().to_string(),
            ));
        }
        BridgeServiceEventKind::StartOfficialExternalNode
        | BridgeServiceEventKind::StartOfficialInProcessNode => {}
    }

    match event.family {
        BridgeRuntimeFamily::Mainline => start_mainline_bridge_owner_thread(event),
        BridgeRuntimeFamily::Tn12 => start_tn12_bridge_owner_thread(event),
    }
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn start_mainline_bridge_owner_thread(
    event: BridgeServiceEvent,
) -> Result<BridgeOwnerRuntimeHandle, BridgeRuntimeError> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let thread_shutdown = shutdown_rx.clone();

    let owner_thread = std::thread::Builder::new()
        .name(format!("kgw-bridge-{}", event.network.as_str()))
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_name(format!("kgw-bridge-rt-{}", event.network.as_str()))
                .enable_all()
                .build();

            let Ok(runtime) = runtime else {
                return;
            };

            runtime.block_on(async move {
                let kaspa_api = kaspa_stratum_bridge_mainline::KaspaApi::new(
                    event.kaspa_rpc_endpoint.clone(),
                    None,
                    thread_shutdown.clone(),
                )
                .await;

                let kaspa_api = match kaspa_api {
                    Ok(kaspa_api) => kaspa_api,
                    Err(_error) => {
                        return;
                    }
                };
                let bridge_config = kaspa_stratum_bridge_mainline::StratumServerBridgeConfig {
                    instance_id: format!("{}-bridge-1", event.network.as_str()),
                    stratum_port: event.stratum_listen.clone(),
                    kaspad_address: event.kaspa_rpc_endpoint.clone(),
                    prom_port: event.prometheus_listen.clone(),
                    print_stats: true,
                    log_to_file: false,
                    health_check_port: String::new(),
                    block_wait_time: std::time::Duration::from_millis(1000),
                    min_share_diff: 8192,
                    var_diff: true,
                    shares_per_min: 20,
                    var_diff_stats: false,
                    extranonce_size: 0,
                    pow2_clamp: false,
                    coinbase_tag_suffix: None,
                };
                match kaspa_stratum_bridge_mainline::listen_and_serve_with_shutdown(
                    bridge_config,
                    std::sync::Arc::clone(&kaspa_api),
                    Some(std::sync::Arc::clone(&kaspa_api)),
                    thread_shutdown,
                )
                .await
                {
                    Ok(()) => {}
                    Err(_error) => {}
                }
            });
        })
        .map_err(|error| BridgeRuntimeError::OwnerThreadStartFailed(error.to_string()))?;

    Ok(BridgeOwnerRuntimeHandle {
        shutdown_tx,
        owner_thread,
    })
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
fn start_mainline_bridge_owner_thread(
    event: BridgeServiceEvent,
) -> Result<BridgeOwnerRuntimeHandle, BridgeRuntimeError> {
    Err(BridgeRuntimeError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-mainline",
        event.network.as_str()
    )))
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn start_tn12_bridge_owner_thread(
    event: BridgeServiceEvent,
) -> Result<BridgeOwnerRuntimeHandle, BridgeRuntimeError> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let thread_shutdown = shutdown_rx.clone();

    let owner_thread = std::thread::Builder::new()
        .name(format!("kgw-bridge-{}", event.network.as_str()))
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_name(format!("kgw-bridge-rt-{}", event.network.as_str()))
                .enable_all()
                .build();

            let Ok(runtime) = runtime else {
                return;
            };

            runtime.block_on(async move {
                let kaspa_api = kaspa_stratum_bridge_tn12::KaspaApi::new(
                    event.kaspa_rpc_endpoint.clone(),
                    None,
                    thread_shutdown.clone(),
                )
                .await;

                let kaspa_api = match kaspa_api {
                    Ok(kaspa_api) => kaspa_api,
                    Err(_error) => {
                        return;
                    }
                };

                // KGW_BRIDGE_OWNER_INTERNAL_CPU_MINER_REAL_RKSTRATUMTN12_V4
                // Existing owner edit only.
                // Real RKStratumTN12 API verified from bridge/src/rkstratum_cpu_miner.rs:
                // - InternalCpuMinerConfig
                // - spawn_internal_cpu_miner(Arc<KaspaApi>, InternalCpuMinerConfig, watch::Receiver<bool>)
                // CPU miner is feature-gated and must never block Stratum server startup when disabled.
                #[cfg(feature = "rkstratum_cpu_miner")]
                {
                    if event.internal_cpu_miner.enabled {
                        let maybe_mining_address = event
                            .internal_cpu_miner
                            .address
                            .clone()
                            .map(|value| value.trim().to_string())
                            .filter(|value| !value.is_empty());

                        let Some(mining_address) = maybe_mining_address else {
                            // Missing mining address disables the optional internal CPU miner only.
                            // It must not prevent the official Stratum listener from starting.
                            return;
                        };

                        let threads: usize =
                            usize::from(event.internal_cpu_miner.threads.unwrap_or(1).max(1));
                        let throttle = event
                            .internal_cpu_miner
                            .throttle_ms
                            .map(std::time::Duration::from_millis);
                        let template_poll_interval = std::time::Duration::from_millis(
                            event
                                .internal_cpu_miner
                                .template_poll_ms
                                .unwrap_or(50)
                                .max(1),
                        );

                        let miner_config = kaspa_stratum_bridge_tn12::InternalCpuMinerConfig {
                            enabled: true,
                            mining_address,
                            threads,
                            throttle,
                            template_poll_interval,
                        };
                        kaspa_stratum_bridge_tn12::prom::set_internal_cpu_mining_address(
                            miner_config.mining_address.clone(),
                        );

                        match kaspa_stratum_bridge_tn12::spawn_internal_cpu_miner(
                            std::sync::Arc::clone(&kaspa_api),
                            miner_config,
                            thread_shutdown.clone(),
                        ) {
                            Ok(metrics) => {
                                kaspa_stratum_bridge_tn12::set_rkstratum_cpu_miner_metrics(metrics);
                            }
                            Err(_error) => {}
                        }
                    } else {
                    }
                }

                #[cfg(not(feature = "rkstratum_cpu_miner"))]
                {
                    if event.internal_cpu_miner.enabled {
                    } else {
                    }
                }

                let bridge_config = kaspa_stratum_bridge_tn12::StratumServerBridgeConfig {
                    instance_id: format!("{}-bridge-1", event.network.as_str()),
                    stratum_port: event.stratum_listen.clone(),
                    kaspad_address: event.kaspa_rpc_endpoint.clone(),
                    prom_port: event.prometheus_listen.clone(),
                    print_stats: true,
                    log_to_file: false,
                    health_check_port: String::new(),
                    block_wait_time: std::time::Duration::from_millis(1000),
                    min_share_diff: 8192,
                    var_diff: true,
                    shares_per_min: 20,
                    var_diff_stats: false,
                    extranonce_size: 0,
                    pow2_clamp: false,
                    coinbase_tag_suffix: None,
                };
                match kaspa_stratum_bridge_tn12::listen_and_serve_with_shutdown(
                    bridge_config,
                    std::sync::Arc::clone(&kaspa_api),
                    Some(std::sync::Arc::clone(&kaspa_api)),
                    thread_shutdown,
                )
                .await
                {
                    Ok(()) => {}
                    Err(_error) => {}
                }
            });
        })
        .map_err(|error| BridgeRuntimeError::OwnerThreadStartFailed(error.to_string()))?;

    Ok(BridgeOwnerRuntimeHandle {
        shutdown_tx,
        owner_thread,
    })
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
fn start_tn12_bridge_owner_thread(
    event: BridgeServiceEvent,
) -> Result<BridgeOwnerRuntimeHandle, BridgeRuntimeError> {
    Err(BridgeRuntimeError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-tn12",
        event.network.as_str()
    )))
}

pub fn all_parallel_bridge_plans_v1() -> Result<Vec<BridgeRuntimePlan>, BridgeRuntimeError> {
    ["mainnet", "testnet10", "testnet12"]
        .into_iter()
        .map(|network| {
            official_bridge_plan_from_settings_v1(
                BridgeRuntimeSettings {
                    network: network.to_string(),
                    mode: BridgeRuntimeMode::OfficialExternalNode,
                    explicit_runtime_opt_in: true,
                    ..BridgeRuntimeSettings::default()
                },
                false,
            )
        })
        .collect()
}

pub fn official_kaspa_bridge_summary_v1() -> &'static str {
    "Kaspa bridge follows the KGW service-event mechanism. mainnet/testnet10 use official rusty-kaspa master branch; testnet12 uses official rusty-kaspa tn12 branch. Bridge start uses KaspaApi and listen_and_serve_with_shutdown inside owner."
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
pub fn official_bridge_mainline_dependency_marker_v1() -> &'static str {
    "official-kaspa-runtime-mainline dependency selected"
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
pub fn official_bridge_mainline_dependency_marker_v1() -> &'static str {
    "official-kaspa-runtime-mainline feature disabled"
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
pub fn official_bridge_tn12_dependency_marker_v1() -> &'static str {
    "official-kaspa-runtime-tn12 dependency selected"
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
pub fn official_bridge_tn12_dependency_marker_v1() -> &'static str {
    "official-kaspa-runtime-tn12 feature disabled"
}



