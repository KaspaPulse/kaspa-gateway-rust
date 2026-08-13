use serde::{Deserialize, Serialize};
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
    pub rpc_network: String,
}

impl BridgeStartupReadiness {
    pub fn to_attestation(&self) -> String {
        format!(
            "rpc_method=get_server_info;rpc_network={};rpc_endpoint={};listener={};listeners_ready=1",
            self.rpc_network, self.rpc_endpoint, self.listener
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

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
async fn attest_listener_and_serve<F, E>(
    listen: F,
    startup_tx: std::sync::mpsc::SyncSender<BridgeOwnerStartupOutcome>,
    readiness: BridgeStartupReadiness,
) where
    F: std::future::Future<Output = Result<(), E>>,
    E: std::fmt::Display,
{
    attest_listener_and_serve_with_probe(listen, startup_tx, readiness, |address| {
        tokio::net::TcpStream::connect(address)
    })
    .await;
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
async fn attest_listener_and_serve_with_probe<F, E, C, P>(
    listen: F,
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
            probe = tokio::time::timeout(
                Duration::from_millis(250),
                connect(probe_address.clone()),
            ) => {
                match probe {
                    Ok(Ok(stream)) => {
                        drop(stream);
                        if startup_tx.send(BridgeOwnerStartupOutcome::Ready(readiness)).is_err() {
                            return;
                        }
                        let _ = listen.await;
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
                        None,
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
                    rpc_network: rpc_info.network_id.to_string(),
                };
                let bridge_config = kaspa_stratum_bridge_mainline::StratumServerBridgeConfig {
                    instance_id: format!("{}-bridge-1", event_for_thread.network.as_str()),
                    stratum_port: event_for_thread.stratum_listen.clone(),
                    kaspad_address: event_for_thread.kaspa_rpc_endpoint.clone(),
                    prom_port: event_for_thread.prometheus_listen.clone(),
                    print_stats: true,
                    log_to_file: false,
                    health_check_port: String::new(),
                    block_wait_time: Duration::from_millis(1000),
                    min_share_diff: 8192,
                    var_diff: true,
                    shares_per_min: 20,
                    var_diff_stats: false,
                    extranonce_size: 0,
                    pow2_clamp: false,
                    coinbase_tag_suffix: None,
                };
                let listen = kaspa_stratum_bridge_mainline::listen_and_serve_with_shutdown(
                    bridge_config,
                    std::sync::Arc::clone(&kaspa_api),
                    Some(std::sync::Arc::clone(&kaspa_api)),
                    thread_shutdown,
                );
                attest_listener_and_serve(listen, startup_tx, readiness).await;
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
                        None,
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

                let bridge_config = kaspa_stratum_bridge_tn12::StratumServerBridgeConfig {
                    instance_id: format!("{}-bridge-1", event_for_thread.network.as_str()),
                    stratum_port: event_for_thread.stratum_listen.clone(),
                    kaspad_address: event_for_thread.kaspa_rpc_endpoint.clone(),
                    prom_port: event_for_thread.prometheus_listen.clone(),
                    print_stats: true,
                    log_to_file: false,
                    health_check_port: String::new(),
                    block_wait_time: Duration::from_millis(1000),
                    min_share_diff: 8192,
                    var_diff: true,
                    shares_per_min: 20,
                    var_diff_stats: false,
                    extranonce_size: 0,
                    pow2_clamp: false,
                    coinbase_tag_suffix: None,
                };
                let listen = kaspa_stratum_bridge_tn12::listen_and_serve_with_shutdown(
                    bridge_config,
                    std::sync::Arc::clone(&kaspa_api),
                    Some(std::sync::Arc::clone(&kaspa_api)),
                    thread_shutdown,
                );
                attest_listener_and_serve(listen, startup_tx, readiness).await;
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
            rpc_network: "testnet-10".to_string(),
        };

        let attestation = readiness.to_attestation();
        assert!(attestation.contains("rpc_method=get_server_info"));
        assert!(attestation.contains("rpc_network=testnet-10"));
        assert!(attestation.contains("listener=127.0.0.1:15555"));
        assert!(attestation.contains("listeners_ready=1"));
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
