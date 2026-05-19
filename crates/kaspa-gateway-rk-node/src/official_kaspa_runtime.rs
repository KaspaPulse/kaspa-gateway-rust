use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspaRuntimeNetwork {
    Mainnet,
    Testnet10,
    Testnet12,
}

impl KaspaRuntimeNetwork {
    pub fn parse(value: &str) -> Result<Self, KaspaRuntimeError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" | "testnet10" => Ok(Self::Testnet10),
            "testnet12" | "tn12" => Ok(Self::Testnet12),
            _ => Err(KaspaRuntimeError::UnsupportedNetwork(value.to_string())),
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

    pub fn family(self) -> KaspaRuntimeFamily {
        match self {
            Self::Mainnet => KaspaRuntimeFamily::Mainline,
            Self::Testnet10 | Self::Testnet12 => KaspaRuntimeFamily::Tn12,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspaRuntimeFamily {
    Mainline,
    Tn12,
}

impl KaspaRuntimeFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mainline => "mainline-master",
            Self::Tn12 => "tn12-only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspaNodeRuntimeMode {
    Disabled,
    RemoteRpc,
    IntegratedInProcess,
    IntegratedAsDaemon,
}

impl KaspaNodeRuntimeMode {
    pub fn parse(value: &str) -> Result<Self, KaspaRuntimeError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "remote" | "remote-rpc" => Ok(Self::RemoteRpc),
            "inproc" | "inprocess" | "integrated-inprocess" => Ok(Self::IntegratedInProcess),
            "daemon" | "integrated-daemon" | "integrated-as-daemon" => Ok(Self::IntegratedAsDaemon),
            _ => Err(KaspaRuntimeError::UnsupportedNodeMode(value.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::RemoteRpc => "remote-rpc",
            Self::IntegratedInProcess => "integrated-inprocess",
            Self::IntegratedAsDaemon => "integrated-as-daemon",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspaBridgeRuntimeMode {
    Disabled,
    OfficialExternalNode,
    OfficialInProcessNode,
}

impl KaspaBridgeRuntimeMode {
    pub fn parse(value: &str) -> Result<Self, KaspaRuntimeError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "external" | "external-node" | "official-external-node" => {
                Ok(Self::OfficialExternalNode)
            }
            "inproc" | "inprocess" | "inprocess-node" | "official-inprocess-node" => {
                Ok(Self::OfficialInProcessNode)
            }
            _ => Err(KaspaRuntimeError::UnsupportedBridgeMode(value.to_string())),
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
pub enum KaspaRuntimeServiceEventKind {
    StopAll,
    StartNodeInProc,
    StartNodeDaemon,
    ConnectRemoteRpc,
    StartBridgeExternalNode,
    StartBridgeInProcessNode,
    Stdout,
    Status,
}

impl KaspaRuntimeServiceEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StopAll => "stop-all",
            Self::StartNodeInProc => "start-node-inproc",
            Self::StartNodeDaemon => "start-node-daemon",
            Self::ConnectRemoteRpc => "connect-remote-rpc",
            Self::StartBridgeExternalNode => "start-bridge-external-node",
            Self::StartBridgeInProcessNode => "start-bridge-inprocess-node",
            Self::Stdout => "stdout",
            Self::Status => "status",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspaRuntimeStep {
    ConvertSettingsToRuntimeDecision,
    SelectOfficialBranchByNetwork,
    ConvertDecisionToServiceEvents,
    ConvertNodeConfigToKaspadArgs,
    ComputeFileDescriptorBudget,
    CreateCoreWithRuntime,
    SpawnNamedKaspadThread,
    StoreRpcCoreService,
    AttachRpcToApplicationServices,
    StartOfficialBridgeFromServiceEvent,
    RelayNodeStdoutToLogs,
    RelayBridgeStdoutToLogs,
    PublishRuntimeStatus,
    ShutdownBridgeOwner,
    ShutdownNodeCore,
    JoinKaspadThread,
}

impl KaspaRuntimeStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConvertSettingsToRuntimeDecision => "convert-settings-to-runtime-decision",
            Self::SelectOfficialBranchByNetwork => "select-official-branch-by-network",
            Self::ConvertDecisionToServiceEvents => "convert-decision-to-service-events",
            Self::ConvertNodeConfigToKaspadArgs => "convert-node-config-to-kaspad-args",
            Self::ComputeFileDescriptorBudget => "compute-file-descriptor-budget",
            Self::CreateCoreWithRuntime => "create-core-with-runtime",
            Self::SpawnNamedKaspadThread => "spawn-named-kaspad-thread",
            Self::StoreRpcCoreService => "store-rpc-core-service",
            Self::AttachRpcToApplicationServices => "attach-rpc-to-application-services",
            Self::StartOfficialBridgeFromServiceEvent => "start-official-bridge-from-service-event",
            Self::RelayNodeStdoutToLogs => "relay-node-stdout-to-logs",
            Self::RelayBridgeStdoutToLogs => "relay-bridge-stdout-to-logs",
            Self::PublishRuntimeStatus => "publish-runtime-status",
            Self::ShutdownBridgeOwner => "shutdown-bridge-owner",
            Self::ShutdownNodeCore => "shutdown-node-core",
            Self::JoinKaspadThread => "join-kaspad-thread",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaspaRuntimeSettings {
    pub network: String,
    pub node_mode: KaspaNodeRuntimeMode,
    pub bridge_mode: KaspaBridgeRuntimeMode,
    pub app_dir_name: Option<String>,
    pub stratum_listen: Option<String>,
    pub prometheus_listen: Option<String>,
    pub remote_rpc_endpoint: Option<String>,
    pub enable_utxo_index: bool,
    pub archival: bool,
    pub explicit_runtime_opt_in: bool,
}

impl Default for KaspaRuntimeSettings {
    fn default() -> Self {
        Self {
            network: "mainnet".to_string(),
            node_mode: KaspaNodeRuntimeMode::Disabled,
            bridge_mode: KaspaBridgeRuntimeMode::Disabled,
            app_dir_name: None,
            stratum_listen: None,
            prometheus_listen: None,
            remote_rpc_endpoint: None,
            enable_utxo_index: true,
            archival: false,
            explicit_runtime_opt_in: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaspaRuntimeServiceEvent {
    pub kind: KaspaRuntimeServiceEventKind,
    pub network: KaspaRuntimeNetwork,
    pub family: KaspaRuntimeFamily,
    pub branch: &'static str,
    pub node_mode: KaspaNodeRuntimeMode,
    pub bridge_mode: KaspaBridgeRuntimeMode,
    pub rpc_endpoint: String,
    pub stratum_listen: String,
}

impl KaspaRuntimeServiceEvent {
    pub fn to_log_line(&self) -> String {
        format!(
            "event={};network={};family={};branch={};node_mode={};bridge_mode={};rpc_endpoint={};stratum_listen={}",
            self.kind.as_str(),
            self.network.as_str(),
            self.family.as_str(),
            self.branch,
            self.node_mode.as_str(),
            self.bridge_mode.as_str(),
            self.rpc_endpoint,
            self.stratum_listen
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaspaRuntimePlan {
    pub owner_crate: &'static str,
    pub official_repository: &'static str,
    pub node_package: &'static str,
    pub bridge_package: &'static str,
    pub network: KaspaRuntimeNetwork,
    pub family: KaspaRuntimeFamily,
    pub branch: &'static str,
    pub node_mode: KaspaNodeRuntimeMode,
    pub bridge_mode: KaspaBridgeRuntimeMode,
    pub service_events: Vec<KaspaRuntimeServiceEvent>,
    pub node_feature_expected: &'static str,
    pub bridge_feature_expected: &'static str,
    pub official_node_feature_enabled: bool,
    pub official_bridge_feature_expected: bool,
    pub starts_now: bool,
    pub runs_parallel_safe: bool,
    pub uses_local_clone: bool,
    pub uses_downloaded_exe: bool,
    pub frontend_owns_start: bool,
    pub tauri_owns_runtime: bool,
    pub steps: Vec<KaspaRuntimeStep>,
    pub decision: String,
}

impl KaspaRuntimePlan {
    pub fn to_log_line(&self) -> String {
        let steps = self
            .steps
            .iter()
            .map(|step| step.as_str())
            .collect::<Vec<_>>()
            .join(",");

        let events = self
            .service_events
            .iter()
            .map(|event| event.to_log_line())
            .collect::<Vec<_>>()
            .join("|");

        format!(
            "owner={};repo={};node_package={};bridge_package={};network={};family={};branch={};node_mode={};bridge_mode={};events={};node_feature_expected={};bridge_feature_expected={};official_node_feature={};official_bridge_expected={};starts_now={};parallel_safe={};local_clone={};downloaded_exe={};frontend_start={};tauri_runtime={};steps={};decision={}",
            self.owner_crate,
            self.official_repository,
            self.node_package,
            self.bridge_package,
            self.network.as_str(),
            self.family.as_str(),
            self.branch,
            self.node_mode.as_str(),
            self.bridge_mode.as_str(),
            events,
            self.node_feature_expected,
            self.bridge_feature_expected,
            self.official_node_feature_enabled,
            self.official_bridge_feature_expected,
            self.starts_now,
            self.runs_parallel_safe,
            self.uses_local_clone,
            self.uses_downloaded_exe,
            self.frontend_owns_start,
            self.tauri_owns_runtime,
            steps,
            self.decision
        )
    }
}

#[derive(Debug, Error)]
pub enum KaspaRuntimeError {
    #[error("unsupported network: {0}")]
    UnsupportedNetwork(String),

    #[error("unsupported node mode: {0}")]
    UnsupportedNodeMode(String),

    #[error("unsupported bridge mode: {0}")]
    UnsupportedBridgeMode(String),

    #[error("invalid app dir name")]
    InvalidAppDirName,

    #[error("invalid listen address")]
    InvalidListenAddress,

    #[error("in-process bridge requires in-process node mode")]
    InProcessBridgeRequiresInProcessNode,
}

fn validate_safe_value(value: &str) -> Result<(), KaspaRuntimeError> {
    if value.trim().is_empty()
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(KaspaRuntimeError::InvalidAppDirName);
    }

    Ok(())
}

fn validate_listen(value: &str) -> Result<(), KaspaRuntimeError> {
    if value.trim().is_empty()
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(KaspaRuntimeError::InvalidListenAddress);
    }

    Ok(())
}

pub fn runtime_service_events_from_settings_v1(
    settings: &KaspaRuntimeSettings,
) -> Result<Vec<KaspaRuntimeServiceEvent>, KaspaRuntimeError> {
    let network = KaspaRuntimeNetwork::parse(&settings.network)?;

    if let Some(app_dir) = &settings.app_dir_name {
        validate_safe_value(app_dir)?;
    }

    let rpc_endpoint = settings
        .remote_rpc_endpoint
        .clone()
        .unwrap_or_else(|| network.default_rpc().to_string());

    let stratum_listen = settings
        .stratum_listen
        .clone()
        .unwrap_or_else(|| network.default_stratum().to_string());

    validate_listen(&rpc_endpoint)?;
    validate_listen(&stratum_listen)?;

    if settings.bridge_mode == KaspaBridgeRuntimeMode::OfficialInProcessNode
        && settings.node_mode != KaspaNodeRuntimeMode::IntegratedInProcess
    {
        return Err(KaspaRuntimeError::InProcessBridgeRequiresInProcessNode);
    }

    let mut events = Vec::new();

    let node_event_kind = match settings.node_mode {
        KaspaNodeRuntimeMode::Disabled => KaspaRuntimeServiceEventKind::StopAll,
        KaspaNodeRuntimeMode::RemoteRpc => KaspaRuntimeServiceEventKind::ConnectRemoteRpc,
        KaspaNodeRuntimeMode::IntegratedInProcess => KaspaRuntimeServiceEventKind::StartNodeInProc,
        KaspaNodeRuntimeMode::IntegratedAsDaemon => KaspaRuntimeServiceEventKind::StartNodeDaemon,
    };

    events.push(KaspaRuntimeServiceEvent {
        kind: node_event_kind,
        network,
        family: network.family(),
        branch: network.branch(),
        node_mode: settings.node_mode,
        bridge_mode: settings.bridge_mode,
        rpc_endpoint: rpc_endpoint.clone(),
        stratum_listen: stratum_listen.clone(),
    });

    let bridge_event_kind = match settings.bridge_mode {
        KaspaBridgeRuntimeMode::Disabled => None,
        KaspaBridgeRuntimeMode::OfficialExternalNode => {
            Some(KaspaRuntimeServiceEventKind::StartBridgeExternalNode)
        }
        KaspaBridgeRuntimeMode::OfficialInProcessNode => {
            Some(KaspaRuntimeServiceEventKind::StartBridgeInProcessNode)
        }
    };

    if let Some(kind) = bridge_event_kind {
        events.push(KaspaRuntimeServiceEvent {
            kind,
            network,
            family: network.family(),
            branch: network.branch(),
            node_mode: settings.node_mode,
            bridge_mode: settings.bridge_mode,
            rpc_endpoint,
            stratum_listen,
        });
    }

    Ok(events)
}

pub fn build_official_kaspa_runtime_plan_v1(
    settings: KaspaRuntimeSettings,
) -> Result<KaspaRuntimePlan, KaspaRuntimeError> {
    let network = KaspaRuntimeNetwork::parse(&settings.network)?;
    let events = runtime_service_events_from_settings_v1(&settings)?;

    let node_feature_expected = match network.family() {
        KaspaRuntimeFamily::Mainline => "official-kaspa-runtime-mainline",
        KaspaRuntimeFamily::Tn12 => "official-kaspa-runtime-tn12",
    };

    let bridge_feature_expected = match network.family() {
        KaspaRuntimeFamily::Mainline => "official-kaspa-runtime-mainline",
        KaspaRuntimeFamily::Tn12 => "official-kaspa-runtime-tn12",
    };

    let official_node_feature_enabled = match network.family() {
        KaspaRuntimeFamily::Mainline => cfg!(feature = "official-kaspa-runtime-mainline"),
        KaspaRuntimeFamily::Tn12 => cfg!(feature = "official-kaspa-runtime-tn12"),
    };

    let decision = if !settings.explicit_runtime_opt_in {
        "plan only; explicit opt-in is false; no node or bridge is started"
    } else {
        "KGW service-event mechanism selected; real start remains owner-gated"
    };

    Ok(KaspaRuntimePlan {
        owner_crate: "kaspa-gateway-rk-node",
        official_repository: "https://github.com/kaspanet/rusty-kaspa.git",
        node_package: "kaspad_lib",
        bridge_package: "kaspa-stratum-bridge",
        network,
        family: network.family(),
        branch: network.branch(),
        node_mode: settings.node_mode,
        bridge_mode: settings.bridge_mode,
        service_events: events,
        node_feature_expected,
        bridge_feature_expected,
        official_node_feature_enabled,
        official_bridge_feature_expected: settings.bridge_mode != KaspaBridgeRuntimeMode::Disabled,
        starts_now: false,
        runs_parallel_safe: true,
        uses_local_clone: false,
        uses_downloaded_exe: false,
        frontend_owns_start: false,
        tauri_owns_runtime: false,
        steps: vec![
            KaspaRuntimeStep::ConvertSettingsToRuntimeDecision,
            KaspaRuntimeStep::SelectOfficialBranchByNetwork,
            KaspaRuntimeStep::ConvertDecisionToServiceEvents,
            KaspaRuntimeStep::ConvertNodeConfigToKaspadArgs,
            KaspaRuntimeStep::ComputeFileDescriptorBudget,
            KaspaRuntimeStep::CreateCoreWithRuntime,
            KaspaRuntimeStep::SpawnNamedKaspadThread,
            KaspaRuntimeStep::StoreRpcCoreService,
            KaspaRuntimeStep::AttachRpcToApplicationServices,
            KaspaRuntimeStep::StartOfficialBridgeFromServiceEvent,
            KaspaRuntimeStep::RelayNodeStdoutToLogs,
            KaspaRuntimeStep::RelayBridgeStdoutToLogs,
            KaspaRuntimeStep::PublishRuntimeStatus,
            KaspaRuntimeStep::ShutdownBridgeOwner,
            KaspaRuntimeStep::ShutdownNodeCore,
            KaspaRuntimeStep::JoinKaspadThread,
        ],
        decision: decision.to_string(),
    })
}

pub fn all_parallel_runtime_plans_v1() -> Result<Vec<KaspaRuntimePlan>, KaspaRuntimeError> {
    ["mainnet", "testnet10", "testnet12"]
        .into_iter()
        .map(|network| {
            build_official_kaspa_runtime_plan_v1(KaspaRuntimeSettings {
                network: network.to_string(),
                node_mode: KaspaNodeRuntimeMode::IntegratedInProcess,
                bridge_mode: KaspaBridgeRuntimeMode::OfficialInProcessNode,
                ..KaspaRuntimeSettings::default()
            })
        })
        .collect()
}

pub fn official_kaspa_runtime_summary_v1() -> &'static str {
    "KGW mechanism applied to all node networks and bridges in parallel: settings -> runtime decision -> service events -> owner plan/status. mainnet/testnet10 use official master branch; testnet12 uses official tn12 branch. No local clone, no downloaded exe, no frontend-owned runtime start."
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
pub fn official_node_mainline_dependency_marker_v1() -> &'static str {
    std::any::type_name::<kaspad_lib_mainline::args::Args>()
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
pub fn official_node_mainline_dependency_marker_v1() -> &'static str {
    "official-kaspa-runtime-mainline feature disabled"
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
pub fn official_node_tn12_dependency_marker_v1() -> &'static str {
    std::any::type_name::<kaspad_lib_tn12::args::Args>()
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
pub fn official_node_tn12_dependency_marker_v1() -> &'static str {
    "official-kaspa-runtime-tn12 feature disabled"
}

impl fmt::Display for KaspaNodeRuntimeMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
