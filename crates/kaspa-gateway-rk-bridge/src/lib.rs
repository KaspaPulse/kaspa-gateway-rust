use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::thread::JoinHandle;
#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
use std::time::Duration;
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
            Self::Mainnet | Self::Testnet10 => "stable",
            Self::Testnet12 => "RKStratumTN12",
        }
    }

    pub fn revision(self) -> &'static str {
        match self {
            Self::Mainnet | Self::Testnet10 => "cfafeb4c093fa37a303f1b9f19c58f986b870ce3",
            Self::Testnet12 => "eeb351ee911e2df906d21203dec8db3a195c6b33",
        }
    }

    pub fn family(self) -> BridgeRuntimeFamily {
        match self {
            Self::Mainnet | Self::Testnet10 => BridgeRuntimeFamily::Mainline,
            Self::Testnet12 => BridgeRuntimeFamily::Tn12,
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
            Self::Mainline => "official-stable-v2.0.1",
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
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveBridgeGlobalSettings {
    pub kaspa_rpc_endpoint: String,
    pub block_wait_time_ms: u64,
    pub print_stats: bool,
    pub log_to_file: bool,
    pub health_check_listen: Option<String>,
    pub web_dashboard_listen: Option<String>,
    pub var_diff: bool,
    pub shares_per_min: u32,
    pub var_diff_stats: bool,
    pub extranonce_size: u8,
    pub pow2_clamp: bool,
    pub coinbase_tag_suffix: Option<String>,
    pub approximate_geo_lookup: bool,
}

impl Default for EffectiveBridgeGlobalSettings {
    fn default() -> Self {
        Self {
            kaspa_rpc_endpoint: "127.0.0.1:16110".to_string(),
            block_wait_time_ms: 1_000,
            print_stats: true,
            // The desktop preserves native process stderr as its authoritative log.
            // Separate upstream file logging requires explicit support and is rejected.
            log_to_file: false,
            health_check_listen: None,
            web_dashboard_listen: None,
            var_diff: true,
            shares_per_min: 20,
            var_diff_stats: false,
            extranonce_size: 0,
            pow2_clamp: false,
            coinbase_tag_suffix: None,
            approximate_geo_lookup: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveBridgeInstanceSettings {
    pub instance_id: String,
    pub stratum_listen: String,
    pub min_share_diff: u32,
    pub prometheus_listen: Option<String>,
    pub log_to_file: Option<bool>,
    pub block_wait_time_ms: Option<u64>,
    pub extranonce_size: Option<u8>,
    pub var_diff: Option<bool>,
    pub shares_per_min: Option<u32>,
    pub var_diff_stats: Option<bool>,
    pub pow2_clamp: Option<bool>,
}

impl Default for EffectiveBridgeInstanceSettings {
    fn default() -> Self {
        Self {
            instance_id: "bridge-1".to_string(),
            stratum_listen: ":5555".to_string(),
            min_share_diff: 8_192,
            prometheus_listen: None,
            log_to_file: None,
            block_wait_time_ms: None,
            extranonce_size: None,
            var_diff: None,
            shares_per_min: None,
            var_diff_stats: None,
            pow2_clamp: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveBridgeSettings {
    pub version: u8,
    pub global: EffectiveBridgeGlobalSettings,
    pub instances: Vec<EffectiveBridgeInstanceSettings>,
}

impl Default for EffectiveBridgeSettings {
    fn default() -> Self {
        Self {
            version: 1,
            global: EffectiveBridgeGlobalSettings::default(),
            instances: vec![EffectiveBridgeInstanceSettings::default()],
        }
    }
}

impl EffectiveBridgeSettings {
    pub fn for_network(network: BridgeRuntimeNetwork) -> Self {
        Self {
            global: EffectiveBridgeGlobalSettings {
                kaspa_rpc_endpoint: network.default_rpc().to_string(),
                log_to_file: false,
                ..EffectiveBridgeGlobalSettings::default()
            },
            instances: vec![EffectiveBridgeInstanceSettings {
                instance_id: format!("{}-bridge-1", network.as_str()),
                stratum_listen: network.default_stratum().to_string(),
                prometheus_listen: Some(network.default_prometheus().to_string()),
                ..EffectiveBridgeInstanceSettings::default()
            }],
            ..Self::default()
        }
    }

    pub fn validate_for_network(
        &self,
        network: BridgeRuntimeNetwork,
    ) -> Result<(), BridgeRuntimeError> {
        if self.version != 1 {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "effective Bridge settings version must be 1".to_string(),
            ));
        }
        if self.instances.is_empty() {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "effective Bridge settings require at least one instance".to_string(),
            ));
        }
        if self.global.block_wait_time_ms == 0 {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "blockWaitTimeMs must be greater than zero".to_string(),
            ));
        }
        if self.global.shares_per_min == 0 {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "sharesPerMin must be greater than zero".to_string(),
            ));
        }
        if self.global.extranonce_size > 8 {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "extranonceSize must not exceed 8 bytes".to_string(),
            ));
        }
        validate_rpc_endpoint(&self.global.kaspa_rpc_endpoint)?;
        validate_optional_socket_listener(self.global.health_check_listen.as_deref())?;
        validate_optional_socket_listener(self.global.web_dashboard_listen.as_deref())?;

        if network != BridgeRuntimeNetwork::Testnet12 && self.global.approximate_geo_lookup {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "approximateGeoLookup is supported only by the pinned testnet12 runtime"
                    .to_string(),
            ));
        }
        if self
            .global
            .coinbase_tag_suffix
            .as_deref()
            .is_some_and(|value| value.len() > 256 || value.contains(['\0', '\n', '\r']))
        {
            return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(
                "coinbaseTagSuffix is too long or contains unsafe control characters".to_string(),
            ));
        }

        let mut instance_ids = HashSet::new();
        let mut stratum_listens = HashSet::new();
        let mut all_listens = HashSet::new();
        for (index, instance) in self.instances.iter().enumerate() {
            if instance.instance_id.trim().is_empty()
                || instance.instance_id.len() > 128
                || instance.instance_id.contains(['\0', '\n', '\r'])
            {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "instances[{index}].instanceId is empty, too long, or unsafe"
                )));
            }
            if !instance_ids.insert(instance.instance_id.clone()) {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "duplicate Bridge instanceId: {}",
                    instance.instance_id
                )));
            }
            let stratum_listen = normalized_listener_identity(&instance.stratum_listen)?;
            if !stratum_listens.insert(stratum_listen.clone()) {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "duplicate stratum listener: {}",
                    instance.stratum_listen
                )));
            }
            if !all_listens.insert(stratum_listen) {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "duplicate Bridge listener: {}",
                    instance.stratum_listen
                )));
            }
            if let Some(prometheus) = instance.prometheus_listen.as_deref() {
                let prometheus_identity = normalized_listener_identity(prometheus)?;
                if !all_listens.insert(prometheus_identity) {
                    return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                        "duplicate Bridge listener: {prometheus}"
                    )));
                }
            }
            if instance.block_wait_time_ms == Some(0) {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "instances[{index}].blockWaitTimeMs must be greater than zero"
                )));
            }
            if instance.shares_per_min == Some(0) {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "instances[{index}].sharesPerMin must be greater than zero"
                )));
            }
            if instance.extranonce_size.is_some_and(|value| value > 8) {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "instances[{index}].extranonceSize must not exceed 8 bytes"
                )));
            }
        }

        for (label, listen) in [
            (
                "healthCheckListen",
                self.global.health_check_listen.as_deref(),
            ),
            (
                "webDashboardListen",
                self.global.web_dashboard_listen.as_deref(),
            ),
        ] {
            if let Some(listen) = listen
                && !all_listens.insert(normalized_listener_identity(listen)?)
            {
                return Err(BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                    "{label} conflicts with another Bridge listener: {listen}"
                )));
            }
        }
        Ok(())
    }

    pub fn reject_unowned_services(&self) -> Result<(), BridgeRuntimeError> {
        let mut unsupported = Vec::new();
        if self.global.log_to_file
            || self
                .instances
                .iter()
                .any(|instance| instance.log_to_file == Some(true))
        {
            unsupported.push("logToFile");
        }
        if self.global.health_check_listen.is_some() {
            unsupported.push("healthCheckListen");
        }
        if self.global.web_dashboard_listen.is_some() {
            unsupported.push("webDashboardListen");
        }
        if self.global.approximate_geo_lookup {
            unsupported.push("approximateGeoLookup");
        }
        if !unsupported.is_empty() {
            return Err(BridgeRuntimeError::UnsupportedEffectiveBridgeSettings(
                unsupported.join(","),
            ));
        }
        Ok(())
    }

    pub fn into_service_events(
        mut self,
        network: BridgeRuntimeNetwork,
        mode: BridgeRuntimeMode,
        internal_cpu_miner: BridgeInternalCpuMinerSettings,
        owned_rpc_endpoint: Option<&str>,
    ) -> Result<Vec<BridgeServiceEvent>, BridgeRuntimeError> {
        if let Some(owned_rpc_endpoint) = owned_rpc_endpoint {
            validate_rpc_endpoint(owned_rpc_endpoint)?;
            self.global.kaspa_rpc_endpoint = owned_rpc_endpoint.to_string();
        }
        self.validate_for_network(network)?;
        self.reject_unowned_services()?;
        let global = self.global;
        Ok(self
            .instances
            .drain(..)
            .map(|instance| BridgeServiceEvent {
                kind: match mode {
                    BridgeRuntimeMode::Disabled => BridgeServiceEventKind::Stop,
                    BridgeRuntimeMode::OfficialExternalNode => {
                        BridgeServiceEventKind::StartOfficialExternalNode
                    }
                    BridgeRuntimeMode::OfficialInProcessNode => {
                        BridgeServiceEventKind::StartOfficialInProcessNode
                    }
                },
                network,
                family: network.family(),
                branch: network.branch(),
                mode,
                stratum_listen: instance.stratum_listen.clone(),
                prometheus_listen: instance.prometheus_listen.clone().unwrap_or_default(),
                kaspa_rpc_endpoint: global.kaspa_rpc_endpoint.clone(),
                internal_cpu_miner: internal_cpu_miner.clone(),
                effective_global: global.clone(),
                effective_instance: instance,
            })
            .collect())
    }
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
    pub effective_global: EffectiveBridgeGlobalSettings,
    pub effective_instance: EffectiveBridgeInstanceSettings,
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

    #[error("invalid effective Bridge settings: {0}")]
    InvalidEffectiveBridgeSettings(String),

    #[error(
        "unsupported effective Bridge settings in the desktop-owned runtime: {0}; use only settings with an ownership-safe in-process implementation"
    )]
    UnsupportedEffectiveBridgeSettings(String),

    #[error("bridge in-process mode requires same-owner in-process node")]
    InProcessBridgeRequiresInProcessNode,

    #[error("bridge runtime feature is required: {0}")]
    FeatureRequired(String),

    #[error("bridge owner thread start failed: {0}")]
    OwnerThreadStartFailed(String),

    #[error("bridge startup readiness failed: {0}")]
    StartupReadinessFailed(String),

    #[error(
        "bridge RPC network mismatch: endpoint={endpoint};expected_network={expected};actual_network={actual}"
    )]
    RpcNetworkMismatch {
        expected: String,
        actual: String,
        endpoint: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeStartupReadiness {
    pub network: BridgeRuntimeNetwork,
    pub rpc_endpoint: String,
    pub listener: String,
    pub prometheus_listener: Option<String>,
    pub rpc_network: String,
}

impl BridgeStartupReadiness {
    pub fn listener_count(&self) -> usize {
        1 + usize::from(self.prometheus_listener.is_some())
    }

    pub fn to_attestation(&self) -> String {
        format!(
            "rpc_method=get_server_info;rpc_network={};rpc_endpoint={};listener={};prometheus_listener={};listeners_ready={}",
            self.rpc_network,
            self.rpc_endpoint,
            self.listener,
            self.prometheus_listener.as_deref().unwrap_or(""),
            self.listener_count()
        )
    }
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
enum BridgeOwnerStartupOutcome {
    Ready(BridgeStartupReadiness),
    Failed(String),
}

pub const KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT_MS: u64 = 90_000;
pub const KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT_MS: u64 = 5_000;
pub const KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT_MS: u64 = 5_000;
pub const KGW_BRIDGE_PARENT_ATTESTATION_GRACE_MS: u64 = 1_000;
pub const KGW_BRIDGE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS: u64 = KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT_MS
    + KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT_MS
    + KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT_MS
    + KGW_BRIDGE_PARENT_ATTESTATION_GRACE_MS;

#[cfg(all(
    not(test),
    any(
        feature = "official-kaspa-runtime-mainline",
        feature = "official-kaspa-runtime-tn12"
    )
))]
const KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT: Duration =
    Duration::from_millis(KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT_MS);
#[cfg(all(
    test,
    any(
        feature = "official-kaspa-runtime-mainline",
        feature = "official-kaspa-runtime-tn12"
    )
))]
const KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT: Duration = Duration::from_millis(750);
#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
const KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT: Duration =
    Duration::from_millis(KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT_MS);
#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
const KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT: Duration =
    Duration::from_millis(KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT_MS);
#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
const KGW_BRIDGE_PARENT_ATTESTATION_GRACE: Duration =
    Duration::from_millis(KGW_BRIDGE_PARENT_ATTESTATION_GRACE_MS);

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

    pub fn join(self) -> std::thread::Result<()> {
        self.request_stop();
        self.owner_thread.join()
    }
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn expected_rpc_network_id(network: BridgeRuntimeNetwork) -> &'static str {
    match network {
        BridgeRuntimeNetwork::Mainnet => "mainnet",
        BridgeRuntimeNetwork::Testnet10 => "testnet-10",
        BridgeRuntimeNetwork::Testnet12 => "testnet-12",
    }
}

#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn grpc_endpoint(endpoint: &str) -> String {
    if endpoint.starts_with("grpc://") {
        endpoint.to_string()
    } else {
        format!("grpc://{endpoint}")
    }
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn validate_rpc_network_identity(
    network: BridgeRuntimeNetwork,
    endpoint: &str,
    actual_network: &str,
) -> Result<(), BridgeRuntimeError> {
    let expected = expected_rpc_network_id(network);
    if actual_network == expected {
        Ok(())
    } else {
        Err(BridgeRuntimeError::RpcNetworkMismatch {
            expected: expected.to_string(),
            actual: actual_network.to_string(),
            endpoint: endpoint.to_string(),
        })
    }
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn listener_probe_address(listener: &str) -> Result<String, String> {
    let listener = listener.trim();
    if listener.chars().all(|character| character.is_ascii_digit()) {
        return Ok(format!("127.0.0.1:{listener}"));
    }
    if let Some(port) = listener.strip_prefix(':') {
        return Ok(format!("127.0.0.1:{port}"));
    }

    if let Ok(address) = listener.parse::<std::net::SocketAddr>()
        && address.ip().is_unspecified()
    {
        let loopback = match address {
            std::net::SocketAddr::V4(_) => std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            std::net::SocketAddr::V6(_) => std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        };
        return Ok(std::net::SocketAddr::new(loopback, address.port()).to_string());
    }

    if listener.is_empty() {
        Err("Stratum listener address is empty".to_string())
    } else {
        Ok(listener.to_string())
    }
}

#[cfg(test)]
async fn attest_listener_and_serve<F, E>(
    listen: F,
    startup_tx: std::sync::mpsc::SyncSender<BridgeOwnerStartupOutcome>,
    readiness: BridgeStartupReadiness,
) where
    F: std::future::Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    attest_listener_and_serve_with_auxiliary_probe(
        listen,
        None,
        startup_tx,
        readiness,
        tokio::net::TcpStream::connect,
    )
    .await;
}

#[cfg(test)]
async fn attest_listener_and_serve_with_probe<F, E, C, P>(
    listen: F,
    startup_tx: std::sync::mpsc::SyncSender<BridgeOwnerStartupOutcome>,
    readiness: BridgeStartupReadiness,
    connect: C,
) where
    F: std::future::Future<Output = Result<(), E>>,
    E: std::fmt::Display,
    C: FnMut(String) -> P,
    P: std::future::Future<Output = Result<tokio::net::TcpStream, std::io::Error>>,
{
    attest_listener_and_serve_with_auxiliary_probe(listen, None, startup_tx, readiness, connect)
        .await;
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
async fn attest_listener_and_serve_with_auxiliary_probe<F, E, C, P>(
    listen: F,
    mut auxiliary: Option<tokio::task::JoinHandle<Result<(), String>>>,
    startup_tx: std::sync::mpsc::SyncSender<BridgeOwnerStartupOutcome>,
    readiness: BridgeStartupReadiness,
    mut connect: C,
) where
    F: std::future::Future<Output = Result<(), E>>,
    E: std::fmt::Display,
    C: FnMut(String) -> P,
    P: std::future::Future<Output = Result<tokio::net::TcpStream, std::io::Error>>,
{
    let listener = readiness.listener.clone();
    let auxiliary_listener = readiness.prometheus_listener.clone();
    let probe_address = match listener_probe_address(&listener) {
        Ok(address) => address,
        Err(error) => {
            let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(error));
            return;
        }
    };
    let deadline = tokio::time::Instant::now() + KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT;
    let mut probe_errors = Vec::new();
    tokio::pin!(listen);

    loop {
        tokio::select! {
            // Both exact upstream pins reach their numeric TcpListener::bind on
            // the official future's first poll. Poll its terminal result first
            // so a simultaneous foreign TCP connection can never outrank the
            // KGW-owned official listener's bind failure.
            biased;
            result = &mut listen => {
                let message = match result {
                    Ok(()) => "Stratum listener stopped before readiness".to_string(),
                    Err(error) => format!("Stratum listener setup failed: {error}"),
                };
                let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(message));
                return;
            }
            auxiliary_result = async {
                match auxiliary.as_mut() {
                    Some(task) => Some(task.await),
                    None => std::future::pending().await,
                }
            } => {
                let message = match auxiliary_result {
                    Some(Ok(Ok(()))) => "Prometheus listener stopped before readiness".to_string(),
                    Some(Ok(Err(error))) => format!("Prometheus listener setup failed: {error}"),
                    Some(Err(error)) => format!("Prometheus listener task failed: {error}"),
                    None => unreachable!(),
                };
                let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(message));
                return;
            }
            probe = tokio::time::timeout(
                Duration::from_millis(250),
                connect(probe_address.clone()),
            ) => {
                match probe {
                    Ok(Ok(stream)) => {
                        drop(stream);
                        if let Some(auxiliary_listener) = auxiliary_listener.as_deref()
                            && let Some(auxiliary_task) = auxiliary.as_mut()
                        {
                            let auxiliary_probe = async {
                                let probe_address = listener_probe_address(auxiliary_listener)?;
                                let deadline = tokio::time::Instant::now()
                                    + KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT;
                                loop {
                                    tokio::select! {
                                        biased;
                                        result = &mut listen => {
                                            return Err(match result {
                                                Ok(()) => "Stratum listener stopped before readiness".to_string(),
                                                Err(error) => format!("Stratum listener failed before readiness: {error}"),
                                            });
                                        }
                                        result = &mut *auxiliary_task => {
                                            return Err(match result {
                                                Ok(Ok(())) => "Prometheus listener stopped before readiness".to_string(),
                                                Ok(Err(error)) => format!("Prometheus listener setup failed: {error}"),
                                                Err(error) => format!("Prometheus listener task failed: {error}"),
                                            });
                                        }
                                        probe = tokio::time::timeout(
                                            Duration::from_millis(250),
                                            tokio::net::TcpStream::connect(probe_address.clone()),
                                        ) => {
                                            if let Ok(Ok(stream)) = probe {
                                                drop(stream);
                                                return Ok(());
                                            }
                                        }
                                    }
                                    if tokio::time::Instant::now() >= deadline {
                                        return Err(format!(
                                            "Prometheus listener did not accept TCP connections;listener={auxiliary_listener};probe_address={probe_address};timeout_ms={}",
                                            KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT.as_millis()
                                        ));
                                    }
                                    tokio::time::sleep(Duration::from_millis(25)).await;
                                }
                            };
                            if let Err(error) = auxiliary_probe.await {
                                let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(error));
                                return;
                            }
                        }
                        if startup_tx.send(BridgeOwnerStartupOutcome::Ready(readiness)).is_err() {
                            return;
                        }
                        if let Some(mut auxiliary) = auxiliary.take() {
                            tokio::select! {
                                _ = &mut listen => {}
                                _ = &mut auxiliary => {}
                            }
                        } else {
                            let _ = listen.await;
                        }
                        return;
                    }
                    Ok(Err(error)) => {
                        probe_errors.push(error.to_string());
                    }
                    Err(_) => {
                        probe_errors.push("TCP listener probe timed out".to_string());
                    }
                }
            }
        }

        if tokio::time::Instant::now() >= deadline {
            let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                "Stratum listener did not accept TCP connections;listener={listener};probe_address={probe_address};timeout_ms={};last_error={}",
                KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT.as_millis(),
                probe_errors.last().map(String::as_str).unwrap_or("listener probe was not completed")
            )));
            return;
        }

        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn validate_listen(value: &str) -> Result<(), BridgeRuntimeError> {
    normalized_listener_identity(value).map(|_| ())
}

fn validate_rpc_endpoint(value: &str) -> Result<(), BridgeRuntimeError> {
    let value = value.trim();
    if value.is_empty()
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\')
        || value
            .parse::<std::net::SocketAddr>()
            .is_err_and(|_| !is_valid_hostname_endpoint(value))
    {
        return Err(BridgeRuntimeError::InvalidListenAddress);
    }

    Ok(())
}

fn is_valid_hostname_endpoint(value: &str) -> bool {
    let Some((host, port)) = value.rsplit_once(':') else {
        return false;
    };
    !host.trim().is_empty()
        && host
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
        && port.parse::<u16>().is_ok_and(|port| port != 0)
}

fn normalized_listener_identity(value: &str) -> Result<String, BridgeRuntimeError> {
    let value = value.trim();
    if value.is_empty() || value.contains("..") || value.contains('/') || value.contains('\\') {
        return Err(BridgeRuntimeError::InvalidListenAddress);
    }
    if let Some(port) = value.strip_prefix(':').or_else(|| {
        value
            .chars()
            .all(|character| character.is_ascii_digit())
            .then_some(value)
    }) {
        let port = port
            .parse::<u16>()
            .ok()
            .filter(|port| *port != 0)
            .ok_or(BridgeRuntimeError::InvalidListenAddress)?;
        return Ok(format!("0.0.0.0:{port}"));
    }
    let address = value
        .parse::<std::net::SocketAddr>()
        .map_err(|_| BridgeRuntimeError::InvalidListenAddress)?;
    if address.port() == 0 {
        return Err(BridgeRuntimeError::InvalidListenAddress);
    }

    Ok(address.to_string())
}

fn validate_optional_socket_listener(value: Option<&str>) -> Result<(), BridgeRuntimeError> {
    if let Some(value) = value {
        validate_listen(value)?;
    }
    Ok(())
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
pub fn effective_bridge_settings_from_mainline_yaml_v1(
    content: &str,
) -> Result<EffectiveBridgeSettings, BridgeRuntimeError> {
    let config =
        kaspa_stratum_bridge_mainline::BridgeConfig::from_yaml(content).map_err(|error| {
            BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
                "official stable Bridge config parse failed: {error}"
            ))
        })?;
    let settings = EffectiveBridgeSettings {
        version: 1,
        global: EffectiveBridgeGlobalSettings {
            kaspa_rpc_endpoint: config.global.kaspad_address,
            block_wait_time_ms: u64::try_from(config.global.block_wait_time.as_millis())
                .unwrap_or(u64::MAX),
            print_stats: config.global.print_stats,
            log_to_file: config.global.log_to_file,
            health_check_listen: nonempty_option(config.global.health_check_port),
            web_dashboard_listen: nonempty_option(config.global.web_dashboard_port),
            var_diff: config.global.var_diff,
            shares_per_min: config.global.shares_per_min,
            var_diff_stats: config.global.var_diff_stats,
            extranonce_size: config.global.extranonce_size,
            pow2_clamp: config.global.pow2_clamp,
            coinbase_tag_suffix: config.global.coinbase_tag_suffix,
            approximate_geo_lookup: false,
        },
        instances: config
            .instances
            .into_iter()
            .enumerate()
            .map(|(index, instance)| EffectiveBridgeInstanceSettings {
                instance_id: format!("bridge-{}", index + 1),
                stratum_listen: instance.stratum_port,
                min_share_diff: instance.min_share_diff,
                prometheus_listen: instance.prom_port,
                log_to_file: instance.log_to_file,
                block_wait_time_ms: instance
                    .block_wait_time
                    .map(|value| u64::try_from(value.as_millis()).unwrap_or(u64::MAX)),
                extranonce_size: instance.extranonce_size,
                var_diff: instance.var_diff,
                shares_per_min: instance.shares_per_min,
                var_diff_stats: instance.var_diff_stats,
                pow2_clamp: instance.pow2_clamp,
            })
            .collect(),
    };
    settings.validate_for_network(BridgeRuntimeNetwork::Mainnet)?;
    Ok(settings)
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
pub fn effective_bridge_settings_from_tn12_yaml_v1(
    content: &str,
) -> Result<EffectiveBridgeSettings, BridgeRuntimeError> {
    let config = kaspa_stratum_bridge_tn12::BridgeConfig::from_yaml(content).map_err(|error| {
        BridgeRuntimeError::InvalidEffectiveBridgeSettings(format!(
            "official testnet12 Bridge config parse failed: {error}"
        ))
    })?;
    let settings = EffectiveBridgeSettings {
        version: 1,
        global: EffectiveBridgeGlobalSettings {
            kaspa_rpc_endpoint: config.global.kaspad_address,
            block_wait_time_ms: u64::try_from(config.global.block_wait_time.as_millis())
                .unwrap_or(u64::MAX),
            print_stats: config.global.print_stats,
            log_to_file: config.global.log_to_file,
            health_check_listen: nonempty_option(config.global.health_check_port),
            web_dashboard_listen: nonempty_option(config.global.web_dashboard_port),
            var_diff: config.global.var_diff,
            shares_per_min: config.global.shares_per_min,
            var_diff_stats: config.global.var_diff_stats,
            extranonce_size: config.global.extranonce_size,
            pow2_clamp: config.global.pow2_clamp,
            coinbase_tag_suffix: config.global.coinbase_tag_suffix,
            approximate_geo_lookup: config.global.approximate_geo_lookup,
        },
        instances: config
            .instances
            .into_iter()
            .enumerate()
            .map(|(index, instance)| EffectiveBridgeInstanceSettings {
                instance_id: format!("bridge-{}", index + 1),
                stratum_listen: instance.stratum_port,
                min_share_diff: instance.min_share_diff,
                prometheus_listen: instance.prom_port,
                log_to_file: instance.log_to_file,
                block_wait_time_ms: instance
                    .block_wait_time
                    .map(|value| u64::try_from(value.as_millis()).unwrap_or(u64::MAX)),
                extranonce_size: instance.extranonce_size,
                var_diff: instance.var_diff,
                shares_per_min: instance.shares_per_min,
                var_diff_stats: instance.var_diff_stats,
                pow2_clamp: instance.pow2_clamp,
            })
            .collect(),
    };
    settings.validate_for_network(BridgeRuntimeNetwork::Testnet12)?;
    Ok(settings)
}

pub fn effective_bridge_settings_from_yaml_v1(
    network: BridgeRuntimeNetwork,
    content: &str,
) -> Result<EffectiveBridgeSettings, BridgeRuntimeError> {
    match network {
        BridgeRuntimeNetwork::Mainnet | BridgeRuntimeNetwork::Testnet10 => {
            #[cfg(feature = "official-kaspa-runtime-mainline")]
            {
                let settings = effective_bridge_settings_from_mainline_yaml_v1(content)?;
                settings.validate_for_network(network)?;
                Ok(settings)
            }
            #[cfg(not(feature = "official-kaspa-runtime-mainline"))]
            {
                let _ = content;
                Err(BridgeRuntimeError::FeatureRequired(
                    "stable Bridge config parsing requires official-kaspa-runtime-mainline"
                        .to_string(),
                ))
            }
        }
        BridgeRuntimeNetwork::Testnet12 => {
            #[cfg(feature = "official-kaspa-runtime-tn12")]
            {
                effective_bridge_settings_from_tn12_yaml_v1(content)
            }
            #[cfg(not(feature = "official-kaspa-runtime-tn12"))]
            {
                let _ = content;
                Err(BridgeRuntimeError::FeatureRequired(
                    "testnet12 Bridge config parsing requires official-kaspa-runtime-tn12"
                        .to_string(),
                ))
            }
        }
    }
}

#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn nonempty_option(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn mainline_stratum_config_from_event(
    event: &BridgeServiceEvent,
) -> kaspa_stratum_bridge_mainline::StratumServerBridgeConfig {
    kaspa_stratum_bridge_mainline::StratumServerBridgeConfig {
        instance_id: event.effective_instance.instance_id.clone(),
        stratum_port: event.effective_instance.stratum_listen.clone(),
        kaspad_address: event.effective_global.kaspa_rpc_endpoint.clone(),
        // The official main loop owns its optional Prometheus server separately;
        // preserve the effective port in the event and keep the Stratum config empty.
        prom_port: String::new(),
        print_stats: event.effective_global.print_stats,
        log_to_file: event
            .effective_instance
            .log_to_file
            .unwrap_or(event.effective_global.log_to_file),
        health_check_port: String::new(),
        block_wait_time: Duration::from_millis(
            event
                .effective_instance
                .block_wait_time_ms
                .unwrap_or(event.effective_global.block_wait_time_ms),
        ),
        min_share_diff: event.effective_instance.min_share_diff,
        var_diff: event
            .effective_instance
            .var_diff
            .unwrap_or(event.effective_global.var_diff),
        shares_per_min: event
            .effective_instance
            .shares_per_min
            .unwrap_or(event.effective_global.shares_per_min),
        var_diff_stats: event
            .effective_instance
            .var_diff_stats
            .unwrap_or(event.effective_global.var_diff_stats),
        extranonce_size: event
            .effective_instance
            .extranonce_size
            .unwrap_or(event.effective_global.extranonce_size),
        pow2_clamp: event
            .effective_instance
            .pow2_clamp
            .unwrap_or(event.effective_global.pow2_clamp),
        coinbase_tag_suffix: event.effective_global.coinbase_tag_suffix.clone(),
    }
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn tn12_stratum_config_from_event(
    event: &BridgeServiceEvent,
) -> kaspa_stratum_bridge_tn12::StratumServerBridgeConfig {
    kaspa_stratum_bridge_tn12::StratumServerBridgeConfig {
        instance_id: event.effective_instance.instance_id.clone(),
        stratum_port: event.effective_instance.stratum_listen.clone(),
        kaspad_address: event.effective_global.kaspa_rpc_endpoint.clone(),
        prom_port: String::new(),
        print_stats: event.effective_global.print_stats,
        log_to_file: event
            .effective_instance
            .log_to_file
            .unwrap_or(event.effective_global.log_to_file),
        health_check_port: String::new(),
        block_wait_time: Duration::from_millis(
            event
                .effective_instance
                .block_wait_time_ms
                .unwrap_or(event.effective_global.block_wait_time_ms),
        ),
        min_share_diff: event.effective_instance.min_share_diff,
        var_diff: event
            .effective_instance
            .var_diff
            .unwrap_or(event.effective_global.var_diff),
        shares_per_min: event
            .effective_instance
            .shares_per_min
            .unwrap_or(event.effective_global.shares_per_min),
        var_diff_stats: event
            .effective_instance
            .var_diff_stats
            .unwrap_or(event.effective_global.var_diff_stats),
        extranonce_size: event
            .effective_instance
            .extranonce_size
            .unwrap_or(event.effective_global.extranonce_size),
        pow2_clamp: event
            .effective_instance
            .pow2_clamp
            .unwrap_or(event.effective_global.pow2_clamp),
        coinbase_tag_suffix: event.effective_global.coinbase_tag_suffix.clone(),
    }
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
pub fn effective_mainline_stratum_config_snapshot_v1(
    event: &BridgeServiceEvent,
) -> EffectiveStratumConfigSnapshot {
    EffectiveStratumConfigSnapshot::from_mainline(mainline_stratum_config_from_event(event))
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
pub fn effective_tn12_stratum_config_snapshot_v1(
    event: &BridgeServiceEvent,
) -> EffectiveStratumConfigSnapshot {
    EffectiveStratumConfigSnapshot::from_tn12(tn12_stratum_config_from_event(event))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveStratumConfigSnapshot {
    pub instance_id: String,
    pub stratum_listen: String,
    pub kaspa_rpc_endpoint: String,
    pub print_stats: bool,
    pub log_to_file: bool,
    pub block_wait_time_ms: u64,
    pub min_share_diff: u32,
    pub var_diff: bool,
    pub shares_per_min: u32,
    pub var_diff_stats: bool,
    pub extranonce_size: u8,
    pub pow2_clamp: bool,
    pub coinbase_tag_suffix: Option<String>,
}

impl EffectiveStratumConfigSnapshot {
    #[cfg(feature = "official-kaspa-runtime-mainline")]
    fn from_mainline(value: kaspa_stratum_bridge_mainline::StratumServerBridgeConfig) -> Self {
        Self {
            instance_id: value.instance_id,
            stratum_listen: value.stratum_port,
            kaspa_rpc_endpoint: value.kaspad_address,
            print_stats: value.print_stats,
            log_to_file: value.log_to_file,
            block_wait_time_ms: u64::try_from(value.block_wait_time.as_millis())
                .unwrap_or(u64::MAX),
            min_share_diff: value.min_share_diff,
            var_diff: value.var_diff,
            shares_per_min: value.shares_per_min,
            var_diff_stats: value.var_diff_stats,
            extranonce_size: value.extranonce_size,
            pow2_clamp: value.pow2_clamp,
            coinbase_tag_suffix: value.coinbase_tag_suffix,
        }
    }

    #[cfg(feature = "official-kaspa-runtime-tn12")]
    fn from_tn12(value: kaspa_stratum_bridge_tn12::StratumServerBridgeConfig) -> Self {
        Self {
            instance_id: value.instance_id,
            stratum_listen: value.stratum_port,
            kaspa_rpc_endpoint: value.kaspad_address,
            print_stats: value.print_stats,
            log_to_file: value.log_to_file,
            block_wait_time_ms: u64::try_from(value.block_wait_time.as_millis())
                .unwrap_or(u64::MAX),
            min_share_diff: value.min_share_diff,
            var_diff: value.var_diff,
            shares_per_min: value.shares_per_min,
            var_diff_stats: value.var_diff_stats,
            extranonce_size: value.extranonce_size,
            pow2_clamp: value.pow2_clamp,
            coinbase_tag_suffix: value.coinbase_tag_suffix,
        }
    }
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
        stratum_listen: stratum_listen.clone(),
        prometheus_listen: prometheus_listen.clone(),
        kaspa_rpc_endpoint: kaspa_rpc_endpoint.clone(),
        internal_cpu_miner: settings.internal_cpu_miner,
        effective_global: EffectiveBridgeGlobalSettings {
            kaspa_rpc_endpoint: kaspa_rpc_endpoint.clone(),
            log_to_file: false,
            ..EffectiveBridgeGlobalSettings::default()
        },
        effective_instance: EffectiveBridgeInstanceSettings {
            instance_id: format!("{}-bridge-1", network.as_str()),
            stratum_listen: stratum_listen.clone(),
            prometheus_listen: Some(prometheus_listen.clone()),
            ..EffectiveBridgeInstanceSettings::default()
        },
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

pub fn start_official_bridge_owner_thread_ready_v1(
    event: BridgeServiceEvent,
) -> Result<(BridgeOwnerRuntimeHandle, BridgeStartupReadiness), BridgeRuntimeError> {
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
        BridgeRuntimeFamily::Mainline => start_mainline_bridge_owner_thread_ready(event),
        BridgeRuntimeFamily::Tn12 => start_tn12_bridge_owner_thread_ready(event),
    }
}

pub fn start_official_bridge_owners_ready_v1(
    events: Vec<BridgeServiceEvent>,
) -> Result<(Vec<BridgeOwnerRuntimeHandle>, Vec<BridgeStartupReadiness>), BridgeRuntimeError> {
    start_bridge_owners_ready_with(events, start_official_bridge_owner_thread_ready_v1)
}

fn start_bridge_owners_ready_with<F>(
    events: Vec<BridgeServiceEvent>,
    mut start: F,
) -> Result<(Vec<BridgeOwnerRuntimeHandle>, Vec<BridgeStartupReadiness>), BridgeRuntimeError>
where
    F: FnMut(
        BridgeServiceEvent,
    ) -> Result<(BridgeOwnerRuntimeHandle, BridgeStartupReadiness), BridgeRuntimeError>,
{
    let mut handles = Vec::with_capacity(events.len());
    let mut readiness = Vec::with_capacity(events.len());

    for event in events {
        match start(event) {
            Ok((handle, attestation)) => {
                handles.push(handle);
                readiness.push(attestation);
            }
            Err(error) => {
                for handle in handles {
                    let _ = handle.join();
                }
                return Err(error);
            }
        }
    }

    Ok((handles, readiness))
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn start_mainline_bridge_owner_thread(
    event: BridgeServiceEvent,
) -> Result<BridgeOwnerRuntimeHandle, BridgeRuntimeError> {
    start_mainline_bridge_owner_thread_ready(event).map(|(handle, _readiness)| handle)
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn start_mainline_bridge_owner_thread_ready(
    event: BridgeServiceEvent,
) -> Result<(BridgeOwnerRuntimeHandle, BridgeStartupReadiness), BridgeRuntimeError> {
    use kaspa_rpc_core_mainline::api::rpc::RpcApi;

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let thread_shutdown = shutdown_rx.clone();
    let cleanup_shutdown_tx = shutdown_tx.clone();
    let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
    let event_for_thread = event.clone();

    let owner_thread = std::thread::Builder::new()
        .name(format!("kgw-bridge-{}", event.network.as_str()))
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_name(format!(
                    "kgw-bridge-rt-{}",
                    event_for_thread.network.as_str()
                ))
                .enable_all()
                .build();

            let runtime = match runtime {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                        "Tokio runtime creation failed: {error}"
                    )));
                    return;
                }
            };

            runtime.block_on(async move {
                let endpoint = event_for_thread.kaspa_rpc_endpoint.clone();
                let kaspa_api = match tokio::time::timeout(
                    KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT,
                    kaspa_stratum_bridge_mainline::KaspaApi::new(
                        endpoint.clone(),
                        event_for_thread.effective_global.coinbase_tag_suffix.clone(),
                        thread_shutdown.clone(),
                    ),
                )
                .await
                {
                    Ok(Ok(kaspa_api)) => kaspa_api,
                    Ok(Err(error)) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                            "KaspaApi attachment failed: {error}"
                        )));
                        return;
                    }
                    Err(_) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(
                            format!(
                                "KaspaApi attachment timed out;endpoint={};timeout_ms={}",
                                endpoint,
                                KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT.as_millis()
                            ),
                        ));
                        return;
                    }
                };

                let rpc_info = match tokio::time::timeout(KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT, async {
                    let client =
                        kaspa_grpc_client_mainline::GrpcClient::connect(grpc_endpoint(&endpoint))
                            .await
                            .map_err(|error| error.to_string())?;
                    client
                        .get_server_info()
                        .await
                        .map_err(|error| error.to_string())
                })
                .await
                {
                    Ok(Ok(info)) => info,
                    Ok(Err(error)) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                            "get_server_info failed after KaspaApi attachment: {error}"
                        )));
                        return;
                    }
                    Err(_) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(
                            format!(
                                "get_server_info timed out after KaspaApi attachment;endpoint={};timeout_ms={}",
                                endpoint,
                                KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT.as_millis()
                            ),
                        ));
                        return;
                    }
                };

                let actual_network = rpc_info.network_id.to_string();
                if let Err(error) = validate_rpc_network_identity(
                    event_for_thread.network,
                    &endpoint,
                    &actual_network,
                ) {
                    let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(
                        error.to_string(),
                    ));
                    return;
                }

                let readiness = BridgeStartupReadiness {
                    network: event_for_thread.network,
                    rpc_endpoint: event_for_thread.kaspa_rpc_endpoint.clone(),
                    listener: event_for_thread.stratum_listen.clone(),
                    prometheus_listener: event_for_thread
                        .effective_instance
                        .prometheus_listen
                        .clone(),
                    rpc_network: rpc_info.network_id.to_string(),
                };
                let prometheus_task = readiness.prometheus_listener.as_ref().map(|listen| {
                    let listen = listen.clone();
                    let instance_id = event_for_thread.effective_instance.instance_id.clone();
                    tokio::spawn(async move {
                        kaspa_stratum_bridge_mainline::prom::start_prom_server(
                            &listen,
                            &instance_id,
                        )
                        .await
                        .map_err(|error| error.to_string())
                    })
                });
                let bridge_config = mainline_stratum_config_from_event(&event_for_thread);
                let listen = kaspa_stratum_bridge_mainline::listen_and_serve_with_shutdown(
                    bridge_config,
                    std::sync::Arc::clone(&kaspa_api),
                    Some(std::sync::Arc::clone(&kaspa_api)),
                    thread_shutdown,
                );
                attest_listener_and_serve_with_auxiliary_probe(
                    listen,
                    prometheus_task,
                    startup_tx,
                    readiness,
                    tokio::net::TcpStream::connect,
                )
                .await;
            });
        })
        .map_err(|error| BridgeRuntimeError::OwnerThreadStartFailed(error.to_string()))?;

    let mut handle = Some(BridgeOwnerRuntimeHandle {
        shutdown_tx,
        owner_thread,
    });

    match startup_rx.recv_timeout(
        KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT
            + KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT
            + KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT
            + KGW_BRIDGE_PARENT_ATTESTATION_GRACE,
    ) {
        Ok(BridgeOwnerStartupOutcome::Ready(readiness)) => handle
            .take()
            .map(|handle| (handle, readiness))
            .ok_or_else(|| {
                BridgeRuntimeError::StartupReadinessFailed(
                    "bridge owner handle missing after READY".to_string(),
                )
            }),
        Ok(BridgeOwnerStartupOutcome::Failed(error)) => {
            let _ = cleanup_shutdown_tx.send(true);
            if let Some(handle) = handle.take() {
                let _ = handle.join();
            }
            Err(BridgeRuntimeError::StartupReadinessFailed(error))
        }
        Err(error) => {
            let _ = cleanup_shutdown_tx.send(true);
            if let Some(handle) = handle.take() {
                let _ = handle.join();
            }
            Err(BridgeRuntimeError::StartupReadinessFailed(format!(
                "startup attestation channel failed: {error}"
            )))
        }
    }
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
fn start_mainline_bridge_owner_thread_ready(
    event: BridgeServiceEvent,
) -> Result<(BridgeOwnerRuntimeHandle, BridgeStartupReadiness), BridgeRuntimeError> {
    Err(BridgeRuntimeError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-mainline",
        event.network.as_str()
    )))
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
    start_tn12_bridge_owner_thread_ready(event).map(|(handle, _readiness)| handle)
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn start_tn12_bridge_owner_thread_ready(
    event: BridgeServiceEvent,
) -> Result<(BridgeOwnerRuntimeHandle, BridgeStartupReadiness), BridgeRuntimeError> {
    use kaspa_rpc_core_tn12::api::rpc::RpcApi;

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let thread_shutdown = shutdown_rx.clone();
    let cleanup_shutdown_tx = shutdown_tx.clone();
    let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
    let event_for_thread = event.clone();

    let owner_thread = std::thread::Builder::new()
        .name(format!("kgw-bridge-{}", event.network.as_str()))
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .thread_name(format!(
                    "kgw-bridge-rt-{}",
                    event_for_thread.network.as_str()
                ))
                .enable_all()
                .build();

            let runtime = match runtime {
                Ok(runtime) => runtime,
                Err(error) => {
                    let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                        "Tokio runtime creation failed: {error}"
                    )));
                    return;
                }
            };

            runtime.block_on(async move {
                let endpoint = event_for_thread.kaspa_rpc_endpoint.clone();
                let kaspa_api = match tokio::time::timeout(
                    KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT,
                    kaspa_stratum_bridge_tn12::KaspaApi::new(
                        endpoint.clone(),
                        event_for_thread.effective_global.coinbase_tag_suffix.clone(),
                        thread_shutdown.clone(),
                    ),
                )
                .await
                {
                    Ok(Ok(kaspa_api)) => kaspa_api,
                    Ok(Err(error)) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                            "KaspaApi attachment failed: {error}"
                        )));
                        return;
                    }
                    Err(_) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(
                            format!(
                                "KaspaApi attachment timed out;endpoint={};timeout_ms={}",
                                endpoint,
                                KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT.as_millis()
                            ),
                        ));
                        return;
                    }
                };

                let rpc_info = match tokio::time::timeout(KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT, async {
                    let client =
                        kaspa_grpc_client_tn12::GrpcClient::connect(grpc_endpoint(&endpoint))
                            .await
                            .map_err(|error| error.to_string())?;
                    client
                        .get_server_info()
                        .await
                        .map_err(|error| error.to_string())
                })
                .await
                {
                    Ok(Ok(info)) => info,
                    Ok(Err(error)) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(format!(
                            "get_server_info failed after KaspaApi attachment: {error}"
                        )));
                        return;
                    }
                    Err(_) => {
                        let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(
                            format!(
                                "get_server_info timed out after KaspaApi attachment;endpoint={};timeout_ms={}",
                                endpoint,
                                KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT.as_millis()
                            ),
                        ));
                        return;
                    }
                };

                let actual_network = rpc_info.network_id.to_string();
                if let Err(error) = validate_rpc_network_identity(
                    event_for_thread.network,
                    &endpoint,
                    &actual_network,
                ) {
                    let _ = startup_tx.send(BridgeOwnerStartupOutcome::Failed(
                        error.to_string(),
                    ));
                    return;
                }

                let readiness = BridgeStartupReadiness {
                    network: event_for_thread.network,
                    rpc_endpoint: event_for_thread.kaspa_rpc_endpoint.clone(),
                    listener: event_for_thread.stratum_listen.clone(),
                    prometheus_listener: event_for_thread
                        .effective_instance
                        .prometheus_listen
                        .clone(),
                    rpc_network: rpc_info.network_id.to_string(),
                };

                #[cfg(feature = "rkstratum_cpu_miner")]
                if event_for_thread.internal_cpu_miner.enabled
                    && let Some(mining_address) = event_for_thread
                        .internal_cpu_miner
                        .address
                        .clone()
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty())
                {
                    let miner_config = kaspa_stratum_bridge_tn12::InternalCpuMinerConfig {
                        enabled: true,
                        mining_address,
                        threads: usize::from(
                            event_for_thread.internal_cpu_miner.threads.unwrap_or(1).max(1),
                        ),
                        throttle: event_for_thread
                            .internal_cpu_miner
                            .throttle_ms
                            .map(Duration::from_millis),
                        template_poll_interval: Duration::from_millis(
                            event_for_thread
                                .internal_cpu_miner
                                .template_poll_ms
                                .unwrap_or(50)
                                .max(1),
                        ),
                    };
                    kaspa_stratum_bridge_tn12::prom::set_internal_cpu_mining_address(
                        miner_config.mining_address.clone(),
                    );

                    if let Ok(metrics) = kaspa_stratum_bridge_tn12::spawn_internal_cpu_miner(
                        std::sync::Arc::clone(&kaspa_api),
                        miner_config,
                        thread_shutdown.clone(),
                    ) {
                        kaspa_stratum_bridge_tn12::set_rkstratum_cpu_miner_metrics(metrics);
                    }
                }

                let prometheus_task = readiness.prometheus_listener.as_ref().map(|listen| {
                    let listen = listen.clone();
                    let instance_id = event_for_thread.effective_instance.instance_id.clone();
                    tokio::spawn(async move {
                        kaspa_stratum_bridge_tn12::prom::start_prom_server(&listen, &instance_id)
                            .await
                            .map_err(|error| error.to_string())
                    })
                });
                let bridge_config = tn12_stratum_config_from_event(&event_for_thread);
                let listen = kaspa_stratum_bridge_tn12::listen_and_serve_with_shutdown(
                    bridge_config,
                    std::sync::Arc::clone(&kaspa_api),
                    Some(std::sync::Arc::clone(&kaspa_api)),
                    thread_shutdown,
                );
                attest_listener_and_serve_with_auxiliary_probe(
                    listen,
                    prometheus_task,
                    startup_tx,
                    readiness,
                    tokio::net::TcpStream::connect,
                )
                .await;
            });
        })
        .map_err(|error| BridgeRuntimeError::OwnerThreadStartFailed(error.to_string()))?;

    let mut handle = Some(BridgeOwnerRuntimeHandle {
        shutdown_tx,
        owner_thread,
    });

    match startup_rx.recv_timeout(
        KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT
            + KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT
            + KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT
            + KGW_BRIDGE_PARENT_ATTESTATION_GRACE,
    ) {
        Ok(BridgeOwnerStartupOutcome::Ready(readiness)) => handle
            .take()
            .map(|handle| (handle, readiness))
            .ok_or_else(|| {
                BridgeRuntimeError::StartupReadinessFailed(
                    "bridge owner handle missing after READY".to_string(),
                )
            }),
        Ok(BridgeOwnerStartupOutcome::Failed(error)) => {
            let _ = cleanup_shutdown_tx.send(true);
            if let Some(handle) = handle.take() {
                let _ = handle.join();
            }
            Err(BridgeRuntimeError::StartupReadinessFailed(error))
        }
        Err(error) => {
            let _ = cleanup_shutdown_tx.send(true);
            if let Some(handle) = handle.take() {
                let _ = handle.join();
            }
            Err(BridgeRuntimeError::StartupReadinessFailed(format!(
                "startup attestation channel failed: {error}"
            )))
        }
    }
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
fn start_tn12_bridge_owner_thread_ready(
    event: BridgeServiceEvent,
) -> Result<(BridgeOwnerRuntimeHandle, BridgeStartupReadiness), BridgeRuntimeError> {
    Err(BridgeRuntimeError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-tn12",
        event.network.as_str()
    )))
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
    "Kaspa bridge follows the KGW service-event mechanism. mainnet/testnet10 use official rusty-kaspa v2.0.1; testnet12 remains an explicit experimental tn12 build. Bridge start uses KaspaApi and listen_and_serve_with_shutdown inside owner."
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

#[cfg(test)]
mod runtime_binding_tests {
    use super::*;

    const STABLE_REV: &str = "cfafeb4c093fa37a303f1b9f19c58f986b870ce3";
    const TN12_REV: &str = "eeb351ee911e2df906d21203dec8db3a195c6b33";

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    fn event_from_instance(
        network: BridgeRuntimeNetwork,
        global: EffectiveBridgeGlobalSettings,
        instance: EffectiveBridgeInstanceSettings,
    ) -> BridgeServiceEvent {
        BridgeServiceEvent {
            kind: BridgeServiceEventKind::StartOfficialExternalNode,
            network,
            family: network.family(),
            branch: network.branch(),
            mode: BridgeRuntimeMode::OfficialExternalNode,
            stratum_listen: instance.stratum_listen.clone(),
            prometheus_listen: instance.prometheus_listen.clone().unwrap_or_default(),
            kaspa_rpc_endpoint: global.kaspa_rpc_endpoint.clone(),
            internal_cpu_miner: BridgeInternalCpuMinerSettings::default(),
            effective_global: global,
            effective_instance: instance,
        }
    }

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    #[test]
    fn official_mainline_yaml_maps_every_supported_field_and_inheritance() {
        let yaml = r#"
kaspad_address: "127.0.0.1:26110"
block_wait_time: 1234
print_stats: false
log_to_file: false
health_check_port: ""
web_dashboard_port: ""
var_diff: false
shares_per_min: 31
var_diff_stats: true
extranonce_size: 4
pow2_clamp: true
coinbase_tag_suffix: "owner-test"
instances:
  - stratum_port: "127.0.0.1:25555"
    min_share_diff: 777
    prom_port: "127.0.0.1:22114"
    log_to_file: false
    block_wait_time: 4321
    extranonce_size: 5
    var_diff: true
    shares_per_min: 44
    var_diff_stats: false
    pow2_clamp: false
  - stratum_port: "127.0.0.1:25556"
    min_share_diff: 888
    prom_port: null
    log_to_file: null
    block_wait_time: null
    extranonce_size: null
    var_diff: null
    shares_per_min: null
    var_diff_stats: null
    pow2_clamp: null
"#;
        let settings = effective_bridge_settings_from_mainline_yaml_v1(yaml).unwrap();
        assert_eq!(settings.global.kaspa_rpc_endpoint, "127.0.0.1:26110");
        assert_eq!(settings.global.block_wait_time_ms, 1234);
        assert!(!settings.global.print_stats);
        assert!(!settings.global.log_to_file);
        assert_eq!(settings.global.shares_per_min, 31);
        assert_eq!(settings.global.extranonce_size, 4);
        assert_eq!(
            settings.global.coinbase_tag_suffix.as_deref(),
            Some("owner-test")
        );
        assert_eq!(settings.instances.len(), 2);
        assert_eq!(
            settings.instances[0].prometheus_listen.as_deref(),
            Some("127.0.0.1:22114")
        );

        let first = effective_mainline_stratum_config_snapshot_v1(&event_from_instance(
            BridgeRuntimeNetwork::Mainnet,
            settings.global.clone(),
            settings.instances[0].clone(),
        ));
        assert_eq!(first.block_wait_time_ms, 4321);
        assert_eq!(first.min_share_diff, 777);
        assert!(first.var_diff);
        assert_eq!(first.shares_per_min, 44);
        assert!(!first.var_diff_stats);
        assert_eq!(first.extranonce_size, 5);
        assert!(!first.pow2_clamp);
        assert_eq!(first.coinbase_tag_suffix.as_deref(), Some("owner-test"));

        let inherited = effective_mainline_stratum_config_snapshot_v1(&event_from_instance(
            BridgeRuntimeNetwork::Testnet10,
            settings.global.clone(),
            settings.instances[1].clone(),
        ));
        assert_eq!(inherited.block_wait_time_ms, 1234);
        assert!(!inherited.var_diff);
        assert_eq!(inherited.shares_per_min, 31);
        assert!(inherited.var_diff_stats);
        assert_eq!(inherited.extranonce_size, 4);
        assert!(inherited.pow2_clamp);
    }

    #[cfg(feature = "official-kaspa-runtime-tn12")]
    #[test]
    fn official_tn12_yaml_preserves_experimental_geo_field() {
        let yaml = r#"
kaspad_address: "127.0.0.1:16310"
log_to_file: false
approximate_geo_lookup: true
instances:
  - stratum_port: "127.0.0.1:5555"
    min_share_diff: 19
"#;
        let settings = effective_bridge_settings_from_tn12_yaml_v1(yaml).unwrap();
        assert!(settings.global.approximate_geo_lookup);
        let error = settings.reject_unowned_services().unwrap_err().to_string();
        assert!(error.contains("approximateGeoLookup"));
    }

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    #[test]
    fn invalid_yaml_and_duplicate_ports_preserve_official_errors() {
        let invalid = effective_bridge_settings_from_mainline_yaml_v1("instances: [")
            .unwrap_err()
            .to_string();
        assert!(invalid.contains("official stable Bridge config parse failed"));

        let duplicate = effective_bridge_settings_from_mainline_yaml_v1(
            r#"
log_to_file: false
instances:
  - stratum_port: ":5555"
    min_share_diff: 1
  - stratum_port: ":5555"
    min_share_diff: 2
"#,
        )
        .unwrap_err()
        .to_string();
        assert!(duplicate.contains("Duplicate stratum_port: :5555"));
    }

    #[test]
    fn typed_schema_rejects_unknown_fields_and_listener_collisions() {
        let unknown = serde_json::from_str::<EffectiveBridgeSettings>(
            r#"{"version":1,"global":{"unknownField":true},"instances":[]}"#,
        )
        .unwrap_err();
        assert!(unknown.to_string().contains("unknown field"));

        let mut settings = EffectiveBridgeSettings::for_network(BridgeRuntimeNetwork::Mainnet);
        settings.global.log_to_file = false;
        settings.global.health_check_listen = settings.instances[0].prometheus_listen.clone();
        let error = settings
            .validate_for_network(BridgeRuntimeNetwork::Mainnet)
            .unwrap_err()
            .to_string();
        assert!(error.contains("conflicts with another Bridge listener"));

        let mut equivalent = EffectiveBridgeSettings::for_network(BridgeRuntimeNetwork::Mainnet);
        equivalent.instances.push(EffectiveBridgeInstanceSettings {
            instance_id: "bridge-2".to_string(),
            stratum_listen: "0.0.0.0:5555".to_string(),
            prometheus_listen: None,
            ..EffectiveBridgeInstanceSettings::default()
        });
        let error = equivalent
            .validate_for_network(BridgeRuntimeNetwork::Mainnet)
            .unwrap_err()
            .to_string();
        assert!(error.contains("duplicate stratum listener"));

        let mut invalid = EffectiveBridgeSettings::for_network(BridgeRuntimeNetwork::Mainnet);
        invalid.instances[0].stratum_listen = "localhost:not-a-port".to_string();
        assert!(
            invalid
                .validate_for_network(BridgeRuntimeNetwork::Mainnet)
                .is_err()
        );
    }

    #[test]
    fn unsupported_services_are_rejected_instead_of_silently_discarded() {
        for mutate in [
            |settings: &mut EffectiveBridgeSettings| settings.global.log_to_file = true,
            |settings: &mut EffectiveBridgeSettings| {
                settings.global.health_check_listen = Some("127.0.0.1:4040".to_string());
            },
            |settings: &mut EffectiveBridgeSettings| {
                settings.global.web_dashboard_listen = Some("127.0.0.1:3030".to_string());
            },
        ] {
            let mut settings = EffectiveBridgeSettings::for_network(BridgeRuntimeNetwork::Mainnet);
            mutate(&mut settings);
            assert!(settings.reject_unowned_services().is_err());
        }
    }

    #[test]
    fn inprocess_rpc_is_authoritative_while_external_rpc_is_preserved() {
        let mut settings = EffectiveBridgeSettings::for_network(BridgeRuntimeNetwork::Mainnet);
        settings.global.log_to_file = false;
        settings.global.kaspa_rpc_endpoint = "127.0.0.1:29999".to_string();
        let external = settings
            .clone()
            .into_service_events(
                BridgeRuntimeNetwork::Mainnet,
                BridgeRuntimeMode::OfficialExternalNode,
                BridgeInternalCpuMinerSettings::default(),
                None,
            )
            .unwrap();
        assert_eq!(external[0].kaspa_rpc_endpoint, "127.0.0.1:29999");

        let embedded = settings
            .into_service_events(
                BridgeRuntimeNetwork::Mainnet,
                BridgeRuntimeMode::OfficialInProcessNode,
                BridgeInternalCpuMinerSettings::default(),
                Some("127.0.0.1:28888"),
            )
            .unwrap();
        assert_eq!(embedded[0].kaspa_rpc_endpoint, "127.0.0.1:28888");
        assert_eq!(
            embedded[0].effective_global.kaspa_rpc_endpoint,
            "127.0.0.1:28888"
        );
    }

    #[test]
    fn mainnet_and_testnet10_share_the_official_stable_runtime() {
        for network in [
            BridgeRuntimeNetwork::Mainnet,
            BridgeRuntimeNetwork::Testnet10,
        ] {
            assert_eq!(network.family(), BridgeRuntimeFamily::Mainline);
            assert_eq!(network.branch(), "stable");
            assert_eq!(network.revision(), STABLE_REV);
        }
    }

    #[test]
    fn testnet12_remains_on_the_separate_experimental_runtime() {
        let network = BridgeRuntimeNetwork::Testnet12;
        assert_eq!(network.family(), BridgeRuntimeFamily::Tn12);
        assert_eq!(network.branch(), "RKStratumTN12");
        assert_eq!(network.revision(), TN12_REV);
    }

    #[test]
    fn listener_probe_normalizes_wildcard_and_bare_port() {
        assert_eq!(listener_probe_address("5555").unwrap(), "127.0.0.1:5555");
        assert_eq!(listener_probe_address(":5555").unwrap(), "127.0.0.1:5555");
        assert_eq!(
            listener_probe_address("0.0.0.0:5555").unwrap(),
            "127.0.0.1:5555"
        );
        assert_eq!(
            listener_probe_address("127.0.0.1:5555").unwrap(),
            "127.0.0.1:5555"
        );
    }

    #[test]
    fn readiness_attestation_includes_rpc_and_listener_evidence() {
        let readiness = BridgeStartupReadiness {
            network: BridgeRuntimeNetwork::Testnet10,
            rpc_endpoint: "127.0.0.1:16210".to_string(),
            listener: "127.0.0.1:15555".to_string(),
            prometheus_listener: None,
            rpc_network: "testnet-10".to_string(),
        };

        let attestation = readiness.to_attestation();
        assert!(attestation.contains("rpc_method=get_server_info"));
        assert!(attestation.contains("rpc_network=testnet-10"));
        assert!(attestation.contains("listener=127.0.0.1:15555"));
        assert!(attestation.contains("prometheus_listener="));
        assert!(attestation.contains("listeners_ready=1"));
    }

    #[test]
    fn readiness_attestation_counts_prometheus_listener() {
        let readiness = BridgeStartupReadiness {
            network: BridgeRuntimeNetwork::Mainnet,
            rpc_endpoint: "127.0.0.1:16110".to_string(),
            listener: "127.0.0.1:15555".to_string(),
            prometheus_listener: Some("127.0.0.1:2114".to_string()),
            rpc_network: "mainnet".to_string(),
        };

        let attestation = readiness.to_attestation();
        assert!(attestation.contains("prometheus_listener=127.0.0.1:2114"));
        assert!(attestation.contains("listeners_ready=2"));
    }

    #[test]
    fn stable_and_experimental_network_ids_remain_distinct() {
        assert_eq!(
            expected_rpc_network_id(BridgeRuntimeNetwork::Mainnet),
            "mainnet"
        );
        assert_eq!(
            expected_rpc_network_id(BridgeRuntimeNetwork::Testnet10),
            "testnet-10"
        );
        assert_eq!(
            expected_rpc_network_id(BridgeRuntimeNetwork::Testnet12),
            "testnet-12"
        );
    }

    #[test]
    fn wrong_rpc_network_produces_failed_bridge_attachment() {
        let error = validate_rpc_network_identity(
            BridgeRuntimeNetwork::Testnet10,
            "127.0.0.1:16210",
            expected_rpc_network_id(BridgeRuntimeNetwork::Mainnet),
        )
        .unwrap_err();
        let text = error.to_string();
        assert!(text.contains("bridge RPC network mismatch"));
        assert!(text.contains("expected_network=testnet-10"));
        assert!(text.contains("actual_network=mainnet"));
    }

    #[tokio::test]
    async fn listener_attestation_reports_terminal_bind_failure() {
        let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
        let readiness = BridgeStartupReadiness {
            network: BridgeRuntimeNetwork::Mainnet,
            rpc_endpoint: "127.0.0.1:16110".to_string(),
            listener: "127.0.0.1:65530".to_string(),
            prometheus_listener: None,
            rpc_network: "mainnet".to_string(),
        };

        attest_listener_and_serve(
            async { Err::<(), _>("failed listening to socket: address already in use") },
            startup_tx,
            readiness,
        )
        .await;

        match startup_rx.recv().unwrap() {
            BridgeOwnerStartupOutcome::Failed(error) => {
                assert!(error.contains("address already in use"));
            }
            BridgeOwnerStartupOutcome::Ready(_) => {
                panic!("terminal listener failure cannot attest READY")
            }
        }
    }

    #[tokio::test]
    async fn prometheus_listener_failure_blocks_readiness() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let listen = async move {
            loop {
                let (stream, _) = listener.accept().await?;
                drop(stream);
            }
            #[allow(unreachable_code)]
            Ok::<(), std::io::Error>(())
        };
        let prometheus_task =
            tokio::spawn(async { Err::<(), String>("address already in use".to_string()) });
        let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
        let readiness = BridgeStartupReadiness {
            network: BridgeRuntimeNetwork::Mainnet,
            rpc_endpoint: "127.0.0.1:16110".to_string(),
            listener: address.to_string(),
            prometheus_listener: Some("127.0.0.1:2114".to_string()),
            rpc_network: "mainnet".to_string(),
        };

        attest_listener_and_serve_with_auxiliary_probe(
            listen,
            Some(prometheus_task),
            startup_tx,
            readiness,
            tokio::net::TcpStream::connect,
        )
        .await;

        match startup_rx.recv().unwrap() {
            BridgeOwnerStartupOutcome::Failed(error) => {
                assert!(error.contains("Prometheus listener setup failed"));
                assert!(error.contains("address already in use"));
            }
            BridgeOwnerStartupOutcome::Ready(_) => {
                panic!("Prometheus listener failure cannot attest READY")
            }
        }
    }

    #[tokio::test]
    async fn foreign_accepting_listener_never_attests_ready() {
        const ATTEMPTS: usize = 128;

        let foreign_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = foreign_listener.local_addr().unwrap();
        let accepted = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let accepted_for_task = std::sync::Arc::clone(&accepted);
        let (foreign_shutdown_tx, mut foreign_shutdown_rx) = tokio::sync::watch::channel(false);
        let foreign_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = foreign_shutdown_rx.changed() => break,
                    result = foreign_listener.accept() => {
                        let (stream, _) = result.unwrap();
                        accepted_for_task.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        drop(stream);
                    }
                }
            }
        });

        for _ in 0..ATTEMPTS {
            let foreign_stream = tokio::net::TcpStream::connect(address).await.unwrap();
            let mut foreign_stream = Some(foreign_stream);
            let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
            let readiness = BridgeStartupReadiness {
                network: BridgeRuntimeNetwork::Mainnet,
                rpc_endpoint: "127.0.0.1:16110".to_string(),
                listener: address.to_string(),
                prometheus_listener: None,
                rpc_network: "mainnet".to_string(),
            };
            let owned_listen = async move {
                let _owned_listener = tokio::net::TcpListener::bind(address).await?;
                std::future::pending::<()>().await;
                Ok::<(), std::io::Error>(())
            };

            attest_listener_and_serve_with_probe(owned_listen, startup_tx, readiness, move |_| {
                std::future::ready(Ok(foreign_stream
                    .take()
                    .expect("each attestation performs one probe")))
            })
            .await;

            match startup_rx.recv().unwrap() {
                BridgeOwnerStartupOutcome::Failed(error) => {
                    assert!(
                        error
                            .to_ascii_lowercase()
                            .contains("address already in use"),
                        "unexpected official bind diagnostic: {error}"
                    );
                }
                BridgeOwnerStartupOutcome::Ready(_) => {
                    panic!("a foreign accepting listener cannot attest KGW READY")
                }
            }
        }

        tokio::time::timeout(Duration::from_secs(2), async {
            while accepted.load(std::sync::atomic::Ordering::SeqCst) < ATTEMPTS {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the foreign server must accept every pre-established probe");
        assert_eq!(
            accepted.load(std::sync::atomic::Ordering::SeqCst),
            ATTEMPTS,
            "the accepting foreign server was exercised without satisfying readiness"
        );

        foreign_shutdown_tx.send(true).unwrap();
        foreign_task.await.unwrap();
        let verification = tokio::net::TcpListener::bind(address).await.unwrap();
        drop(verification);
    }

    #[test]
    fn production_bridge_startup_contract_is_the_sum_of_bounded_stages() {
        assert_eq!(
            KGW_BRIDGE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS,
            KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT_MS
                + KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT_MS
                + KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT_MS
                + KGW_BRIDGE_PARENT_ATTESTATION_GRACE_MS
        );
        for stage in [
            KGW_BRIDGE_NODE_ATTACHMENT_TIMEOUT_MS,
            KGW_BRIDGE_RPC_ATTESTATION_TIMEOUT_MS,
            KGW_BRIDGE_LISTENER_ATTESTATION_TIMEOUT_MS,
            KGW_BRIDGE_PARENT_ATTESTATION_GRACE_MS,
        ] {
            assert!(KGW_BRIDGE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS > stage);
        }
    }

    #[tokio::test]
    async fn listener_attestation_waits_until_tcp_accepts() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let listen = async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => return Ok::<(), std::io::Error>(()),
                    accepted = listener.accept() => { accepted?; }
                }
            }
        };
        let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
        let readiness = BridgeStartupReadiness {
            network: BridgeRuntimeNetwork::Mainnet,
            rpc_endpoint: "127.0.0.1:16110".to_string(),
            listener: address.to_string(),
            prometheus_listener: None,
            rpc_network: "mainnet".to_string(),
        };

        let task = tokio::spawn(attest_listener_and_serve(listen, startup_tx, readiness));
        let outcome = tokio::task::spawn_blocking(move || startup_rx.recv().unwrap())
            .await
            .unwrap();
        match outcome {
            BridgeOwnerStartupOutcome::Ready(readiness) => {
                assert_eq!(readiness.listener, address.to_string());
            }
            BridgeOwnerStartupOutcome::Failed(error) => panic!("unexpected failure: {error}"),
        }
        shutdown_tx.send(true).unwrap();
        task.await.unwrap();
    }

    #[test]
    fn second_listener_failure_stops_prior_owner_and_releases_port() {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = probe.local_addr().unwrap();
        drop(probe);
        let first_event = BridgeServiceEvent {
            kind: BridgeServiceEventKind::StartOfficialExternalNode,
            network: BridgeRuntimeNetwork::Mainnet,
            family: BridgeRuntimeFamily::Mainline,
            branch: "stable",
            mode: BridgeRuntimeMode::OfficialExternalNode,
            stratum_listen: address.to_string(),
            prometheus_listen: "127.0.0.1:0".to_string(),
            kaspa_rpc_endpoint: "127.0.0.1:16110".to_string(),
            internal_cpu_miner: BridgeInternalCpuMinerSettings::default(),
            effective_global: EffectiveBridgeGlobalSettings {
                kaspa_rpc_endpoint: "127.0.0.1:16110".to_string(),
                log_to_file: false,
                ..EffectiveBridgeGlobalSettings::default()
            },
            effective_instance: EffectiveBridgeInstanceSettings {
                stratum_listen: address.to_string(),
                prometheus_listen: Some("127.0.0.1:0".to_string()),
                ..EffectiveBridgeInstanceSettings::default()
            },
        };
        let mut second_event = first_event.clone();
        second_event.stratum_listen = "127.0.0.1:65530".to_string();
        let mut invocation = 0usize;

        let error = start_bridge_owners_ready_with(vec![first_event, second_event], |event| {
            invocation += 1;
            if invocation == 2 {
                return Err(BridgeRuntimeError::StartupReadinessFailed(
                    "second listener bind failed: address already in use".to_string(),
                ));
            }

            let listener = std::net::TcpListener::bind(&event.stratum_listen).unwrap();
            listener.set_nonblocking(true).unwrap();
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let owner_thread = std::thread::spawn(move || {
                while !*shutdown_rx.borrow() {
                    std::thread::sleep(Duration::from_millis(10));
                }
                drop(listener);
            });
            let readiness = BridgeStartupReadiness {
                network: event.network,
                rpc_endpoint: event.kaspa_rpc_endpoint,
                listener: event.stratum_listen,
                prometheus_listener: event.effective_instance.prometheus_listen,
                rpc_network: "mainnet".to_string(),
            };
            Ok((
                BridgeOwnerRuntimeHandle {
                    shutdown_tx,
                    owner_thread,
                },
                readiness,
            ))
        })
        .unwrap_err();

        assert!(error.to_string().contains("second listener bind failed"));
        let rebound = std::net::TcpListener::bind(address)
            .expect("failed multi-listener startup must release prior listener ports");
        drop(rebound);
    }

    #[test]
    fn bridge_handle_stop_requests_join_and_release_listener() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).unwrap();
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let owner_thread = std::thread::spawn(move || {
            while !*shutdown_rx.borrow() {
                std::thread::sleep(Duration::from_millis(10));
            }
            drop(listener);
        });
        let handle = BridgeOwnerRuntimeHandle {
            shutdown_tx,
            owner_thread,
        };

        handle.join().expect("Bridge owner must join after Stop");
        let rebound = std::net::TcpListener::bind(address)
            .expect("Bridge listener must be reusable only after owner join");
        drop(rebound);
    }

    #[test]
    fn bridge_handle_reports_unexpected_owner_terminality() {
        let (shutdown_tx, _shutdown_rx) = tokio::sync::watch::channel(false);
        let handle = BridgeOwnerRuntimeHandle {
            shutdown_tx,
            owner_thread: std::thread::spawn(|| {}),
        };
        while !handle.is_finished() {
            std::thread::yield_now();
        }
        assert!(handle.is_finished());
        handle.join().unwrap();
    }

    #[test]
    fn every_owned_bridge_listener_is_joined_and_released() {
        let mut handles = Vec::new();
        let mut addresses = Vec::new();
        for _ in 0..3 {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            listener.set_nonblocking(true).unwrap();
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            let owner_thread = std::thread::spawn(move || {
                while !*shutdown_rx.borrow() {
                    std::thread::sleep(Duration::from_millis(10));
                }
                drop(listener);
            });
            handles.push(BridgeOwnerRuntimeHandle {
                shutdown_tx,
                owner_thread,
            });
            addresses.push(address);
        }

        for handle in handles {
            handle.join().expect("every Bridge owner must join");
        }
        for address in addresses {
            let rebound = std::net::TcpListener::bind(address)
                .expect("every joined Bridge listener must be reusable");
            drop(rebound);
        }
    }

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    #[test]
    fn unreachable_node_attachment_produces_failed_readiness_without_an_owner() {
        let unavailable = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = unavailable.local_addr().unwrap().to_string();
        drop(unavailable);

        let event = BridgeServiceEvent {
            kind: BridgeServiceEventKind::StartOfficialExternalNode,
            network: BridgeRuntimeNetwork::Mainnet,
            family: BridgeRuntimeFamily::Mainline,
            branch: "stable",
            mode: BridgeRuntimeMode::OfficialExternalNode,
            stratum_listen: "127.0.0.1:0".to_string(),
            prometheus_listen: String::new(),
            kaspa_rpc_endpoint: endpoint.clone(),
            internal_cpu_miner: BridgeInternalCpuMinerSettings::default(),
            effective_global: EffectiveBridgeGlobalSettings {
                kaspa_rpc_endpoint: endpoint.clone(),
                log_to_file: false,
                ..EffectiveBridgeGlobalSettings::default()
            },
            effective_instance: EffectiveBridgeInstanceSettings {
                stratum_listen: "127.0.0.1:0".to_string(),
                prometheus_listen: None,
                ..EffectiveBridgeInstanceSettings::default()
            },
        };

        let started_at = std::time::Instant::now();
        let error = start_official_bridge_owner_thread_ready_v1(event).unwrap_err();
        let text = error.to_string();

        assert!(text.contains("bridge startup readiness failed"));
        assert!(text.contains("KaspaApi attachment timed out"));
        assert!(text.contains(&endpoint));
        assert!(text.contains("timeout_ms=750"));
        assert!(started_at.elapsed() < Duration::from_secs(5));
    }
}
