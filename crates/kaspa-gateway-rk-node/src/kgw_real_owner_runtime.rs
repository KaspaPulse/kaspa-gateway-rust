use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::JoinHandle;
#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::kgw_service_controller::{BridgeNodeKind, KaspadNodeKind, KgwNetwork, NodeSettings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KgwRuntimeStartPolicy {
    Disabled,
    FeatureRequired,
    OwnerThreadStarted,
    OfficialCoreRunning,
}

impl KgwRuntimeStartPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::FeatureRequired => "feature-required",
            Self::OwnerThreadStarted => "owner-thread-started",
            Self::OfficialCoreRunning => "official-core-running",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KgwRuntimeFeatureStatus {
    pub network: KgwNetwork,
    pub branch: &'static str,
    pub required_runtime_feature: &'static str,
    pub runtime_feature_enabled: bool,
    pub can_start_in_owner: bool,
}

impl KgwRuntimeFeatureStatus {
    pub fn for_network(network: KgwNetwork) -> Self {
        let (required_runtime_feature, runtime_feature_enabled) = match network {
            KgwNetwork::Mainnet | KgwNetwork::Testnet10 => (
                "official-kaspa-runtime-mainline",
                cfg!(feature = "official-kaspa-runtime-mainline"),
            ),
            KgwNetwork::Testnet12 => (
                "official-kaspa-runtime-tn12",
                cfg!(feature = "official-kaspa-runtime-tn12"),
            ),
        };

        Self {
            network,
            branch: network.branch(),
            required_runtime_feature,
            runtime_feature_enabled,
            can_start_in_owner: runtime_feature_enabled,
        }
    }

    pub fn to_log_line(&self) -> String {
        format!(
            "network={};branch={};required_runtime_feature={};runtime_feature_enabled={};can_start_in_owner={}",
            self.network.as_str(),
            self.branch,
            self.required_runtime_feature,
            self.runtime_feature_enabled,
            self.can_start_in_owner
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KgwRuntimeSessionStatus {
    pub network: KgwNetwork,
    pub branch: &'static str,
    pub node_kind: KaspadNodeKind,
    pub bridge_kind: BridgeNodeKind,
    pub owner_thread_alive: bool,
    pub official_core_running: bool,
    pub bridge_owner_active: bool,
    pub runtime_requested: bool,
    pub start_policy: KgwRuntimeStartPolicy,
    pub feature_status: KgwRuntimeFeatureStatus,
    pub last_message: String,
    pub logs: Vec<String>,
}

impl KgwRuntimeSessionStatus {
    pub fn to_log_line(&self) -> String {
        format!(
            "network={};branch={};node_kind={};bridge_kind={};owner_thread_alive={};official_core_running={};bridge_owner_active={};runtime_requested={};start_policy={};features=[{}];last_message={}",
            self.network.as_str(),
            self.branch,
            self.node_kind.as_str(),
            self.bridge_kind.as_str(),
            self.owner_thread_alive,
            self.official_core_running,
            self.bridge_owner_active,
            self.runtime_requested,
            self.start_policy.as_str(),
            self.feature_status.to_log_line(),
            self.last_message
        )
    }
}

#[derive(Debug, Error)]
pub enum KgwRealOwnerError {
    #[error("runtime owner lock failed")]
    LockFailed,

    #[error("unsupported bridge/node combination")]
    UnsupportedCombination,

    #[error("runtime feature is required: {0}")]
    FeatureRequired(String),

    #[error("invalid RPC listen endpoint: {0}")]
    InvalidRpcEndpoint(String),

    #[error("invalid P2P listen endpoint: {0}")]
    InvalidP2pListen(String),

    #[error("failed to start owner thread: {0}")]
    OwnerThreadStartFailed(String),

    #[error("node RPC readiness failed: {0}")]
    NodeRpcReadinessFailed(String),

    #[error("bridge startup readiness failed: {0}")]
    BridgeStartupReadinessFailed(String),

    #[error(
        "node RPC network mismatch: endpoint={endpoint};expected_network={expected};actual_network={actual}"
    )]
    NodeRpcNetworkMismatch {
        expected: String,
        actual: String,
        endpoint: String,
    },

    #[error("official in-process Rusty Kaspa core is already claimed in this desktop process: {0}")]
    OfficialCoreAlreadyClaimed(String),
}

#[derive(Debug)]
struct RuntimeSession {
    status: KgwRuntimeSessionStatus,
    owner_thread: Option<JoinHandle<()>>,
    core_terminal_outcome: Option<std::sync::mpsc::Receiver<String>>,
    core_terminal_error: Option<String>,
    bridge_handle: Option<kaspa_gateway_rk_bridge::BridgeOwnerRuntimeHandle>,
}

impl RuntimeSession {
    fn new(network: KgwNetwork) -> Self {
        let feature_status = KgwRuntimeFeatureStatus::for_network(network);

        Self {
            status: KgwRuntimeSessionStatus {
                network,
                branch: network.branch(),
                node_kind: KaspadNodeKind::Disable,
                bridge_kind: BridgeNodeKind::Disable,
                owner_thread_alive: false,
                official_core_running: false,
                bridge_owner_active: false,
                runtime_requested: false,
                start_policy: KgwRuntimeStartPolicy::Disabled,
                feature_status,
                last_message: "not-started".to_string(),
                logs: Vec::new(),
            },
            owner_thread: None,
            core_terminal_outcome: None,
            core_terminal_error: None,
            bridge_handle: None,
        }
    }

    fn push_log(&mut self, message: impl Into<String>) {
        let line = format!("[{}] {}", timestamp_ms(), message.into());

        if self.status.logs.len() >= 128 {
            self.status.logs.remove(0);
        }

        self.status.logs.push(line);
    }
}

#[derive(Clone, Debug)]
pub struct KgwRealOwnerRuntime {
    inner: Arc<Mutex<HashMap<KgwNetwork, RuntimeSession>>>,
}

#[cfg(all(
    not(test),
    any(
        feature = "official-kaspa-runtime-mainline",
        feature = "official-kaspa-runtime-tn12"
    )
))]
const KGW_NODE_READINESS_TIMEOUT: Duration = Duration::from_secs(90);
#[cfg(all(
    test,
    any(
        feature = "official-kaspa-runtime-mainline",
        feature = "official-kaspa-runtime-tn12"
    )
))]
const KGW_NODE_READINESS_TIMEOUT: Duration = Duration::from_millis(750);
#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
const KGW_NODE_READINESS_RETRY_INTERVAL: Duration = Duration::from_millis(250);

impl Default for KgwRealOwnerRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl KgwRealOwnerRuntime {
    pub fn new() -> Self {
        let mut sessions = HashMap::new();

        for network in [
            KgwNetwork::Mainnet,
            KgwNetwork::Testnet10,
            KgwNetwork::Testnet12,
        ] {
            sessions.insert(network, RuntimeSession::new(network));
        }

        Self {
            inner: Arc::new(Mutex::new(sessions)),
        }
    }

    pub fn feature_status(
        &self,
        network: KgwNetwork,
    ) -> Result<KgwRuntimeFeatureStatus, KgwRealOwnerError> {
        let sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;

        Ok(sessions
            .get(&network)
            .map(|session| session.status.feature_status.clone())
            .unwrap_or_else(|| KgwRuntimeFeatureStatus::for_network(network)))
    }

    pub fn status(
        &self,
        network: KgwNetwork,
    ) -> Result<KgwRuntimeSessionStatus, KgwRealOwnerError> {
        let sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;

        Ok(sessions
            .get(&network)
            .map(|session| session.status.clone())
            .unwrap_or_else(|| RuntimeSession::new(network).status))
    }

    pub fn logs(&self, network: KgwNetwork) -> Result<String, KgwRealOwnerError> {
        let status = self.status(network)?;
        Ok(status.logs.join("\n"))
    }

    pub fn attest_node_rpc_readiness(
        &self,
        settings: &NodeSettings,
    ) -> Result<String, KgwRealOwnerError> {
        let mut core_terminal_outcome = {
            let mut sessions = self
                .inner
                .lock()
                .map_err(|_| KgwRealOwnerError::LockFailed)?;
            let session = sessions.get(&settings.network).ok_or_else(|| {
                KgwRealOwnerError::NodeRpcReadinessFailed(format!(
                    "network={};endpoint={};owner session is missing",
                    settings.network.as_str(),
                    settings.rpc_endpoint
                ))
            })?;
            if let Some(error) = session.core_terminal_error.clone() {
                return Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
                    "network={};endpoint={};terminal_error={error}",
                    settings.network.as_str(),
                    settings.rpc_endpoint
                )));
            }
            sessions
                .get_mut(&settings.network)
                .and_then(|session| session.core_terminal_outcome.take())
                .ok_or_else(|| {
                    KgwRealOwnerError::NodeRpcReadinessFailed(format!(
                        "network={};endpoint={};owner core thread is not active",
                        settings.network.as_str(),
                        settings.rpc_endpoint
                    ))
                })?
        };

        let readiness = match settings.network {
            KgwNetwork::Mainnet | KgwNetwork::Testnet10 => {
                attest_mainline_node_rpc_readiness(settings, &mut core_terminal_outcome)
            }
            KgwNetwork::Testnet12 => {
                attest_tn12_node_rpc_readiness(settings, &mut core_terminal_outcome)
            }
        };

        let mut sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;
        let session = sessions
            .entry(settings.network)
            .or_insert_with(|| RuntimeSession::new(settings.network));
        session.core_terminal_error = readiness.as_ref().err().map(ToString::to_string);
        session.core_terminal_outcome = Some(core_terminal_outcome);

        match &readiness {
            Ok(evidence) => {
                session.status.start_policy = KgwRuntimeStartPolicy::OfficialCoreRunning;
                session.status.owner_thread_alive = session
                    .owner_thread
                    .as_ref()
                    .is_some_and(|thread| !thread.is_finished());
                session.status.official_core_running = session.status.owner_thread_alive;
                session.status.last_message = format!(
                    "official Rusty Kaspa core passed RPC readiness;network={};{}",
                    settings.network.as_str(),
                    evidence
                );
            }
            Err(error) => {
                session.status.start_policy = KgwRuntimeStartPolicy::OwnerThreadStarted;
                session.status.owner_thread_alive = session
                    .owner_thread
                    .as_ref()
                    .is_some_and(|thread| !thread.is_finished());
                session.status.official_core_running = false;
                session.status.last_message = error.to_string();
            }
        }

        let message = session.status.last_message.clone();
        session.push_log(message);
        readiness
    }

    pub fn start_node_owner_session(
        &self,
        settings: &NodeSettings,
    ) -> Result<KgwRuntimeSessionStatus, KgwRealOwnerError> {
        if settings.node_kind == KaspadNodeKind::Disable {
            return self.stop_network(settings.network);
        }

        if settings.bridge_kind == BridgeNodeKind::OfficialInProcessNode
            && settings.node_kind != KaspadNodeKind::IntegratedInProc
        {
            return Err(KgwRealOwnerError::UnsupportedCombination);
        }

        {
            let mut sessions = self
                .inner
                .lock()
                .map_err(|_| KgwRealOwnerError::LockFailed)?;

            let session = sessions
                .entry(settings.network)
                .or_insert_with(|| RuntimeSession::new(settings.network));

            session.status.feature_status = KgwRuntimeFeatureStatus::for_network(settings.network);
            session.status.runtime_requested = true;
            session.status.node_kind = settings.node_kind;
            session.status.bridge_kind = settings.bridge_kind;

            if !session.status.feature_status.can_start_in_owner {
                session.status.start_policy = KgwRuntimeStartPolicy::FeatureRequired;
                session.status.owner_thread_alive = false;
                session.status.official_core_running = false;
                session.status.bridge_owner_active = false;
                session.status.last_message = format!(
                    "owner runtime is ready; enable feature {} for {}; no fallback layer is allowed",
                    session.status.feature_status.required_runtime_feature,
                    settings.network.as_str()
                );
                let msg = session.status.last_message.clone();
                session.push_log(msg);
                return Ok(session.status.clone());
            }

            kgw_claim_single_inproc_official_core(settings.network)?;

            if session.owner_thread.is_none() {
                let (owner_thread, core_terminal_outcome) =
                    spawn_official_core_thread(settings.clone())?;
                session.owner_thread = Some(owner_thread);
                session.core_terminal_outcome = Some(core_terminal_outcome);
                session.core_terminal_error = None;
            }

            session.status.start_policy = KgwRuntimeStartPolicy::OwnerThreadStarted;
            session.status.owner_thread_alive = true;
            session.status.official_core_running = false;
            session.status.bridge_owner_active = false;
            session.status.last_message = format!(
                "official Rusty Kaspa core owner thread started; RPC readiness pending;network={};endpoint={}",
                settings.network.as_str(),
                settings.rpc_endpoint
            );
            let message = session.status.last_message.clone();
            session.push_log(message);
        }

        let node_readiness = self.attest_node_rpc_readiness(settings)?;

        let bridge_start = start_bridge_owner_if_requested(settings);
        let mut sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;
        let session = sessions
            .entry(settings.network)
            .or_insert_with(|| RuntimeSession::new(settings.network));

        match bridge_start {
            Ok(bridge_handle) => {
                session.bridge_handle = bridge_handle;
                session.status.bridge_owner_active = session.bridge_handle.is_some();
            }
            Err(error) => {
                session.bridge_handle = None;
                session.status.bridge_owner_active = false;
                session.push_log(format!("bridge-owner-start-error {}", error));
                session.status.start_policy = KgwRuntimeStartPolicy::OwnerThreadStarted;
                session.status.official_core_running = false;
                session.status.last_message = format!(
                    "bridge readiness failed after node RPC readiness;network={};node_readiness={};error={}",
                    settings.network.as_str(),
                    node_readiness,
                    error
                );
                return Err(KgwRealOwnerError::BridgeStartupReadinessFailed(error));
            }
        }

        session.status.start_policy = KgwRuntimeStartPolicy::OfficialCoreRunning;
        session.status.owner_thread_alive = session
            .owner_thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished());
        session.status.official_core_running = session.status.owner_thread_alive;
        session.status.last_message = format!(
            "official Rusty Kaspa core and bridge owner accepted inside KGW owner; network={};branch={};node_kind={};bridge_kind={};node_readiness={};same_db_path=true;exclusive_node_owner_per_network=true;no_binary_path_fallback=true;no_bridge_binary_fallback=true;no_process_owner_fallback=true",
            settings.network.as_str(),
            settings.network.branch(),
            settings.node_kind.as_str(),
            settings.bridge_kind.as_str(),
            node_readiness
        );
        let msg = session.status.last_message.clone();
        session.push_log(msg);

        Ok(session.status.clone())
    }

    pub fn stop_network(
        &self,
        network: KgwNetwork,
    ) -> Result<KgwRuntimeSessionStatus, KgwRealOwnerError> {
        let mut sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;

        let session = sessions
            .entry(network)
            .or_insert_with(|| RuntimeSession::new(network));

        if let Some(bridge_handle) = session.bridge_handle.take() {
            bridge_handle.request_stop();
        }

        session.owner_thread = None;
        session.core_terminal_outcome = None;
        session.core_terminal_error = None;
        session.status.node_kind = KaspadNodeKind::Disable;
        session.status.bridge_kind = BridgeNodeKind::Disable;
        session.status.owner_thread_alive = false;
        session.status.official_core_running = false;
        session.status.bridge_owner_active = false;
        session.status.runtime_requested = false;
        session.status.start_policy = KgwRuntimeStartPolicy::Disabled;
        session.status.last_message =
            "KGW owner runtime session disabled through owner state".to_string();
        let msg = session.status.last_message.clone();
        session.push_log(msg);

        Ok(session.status.clone())
    }
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn expected_rpc_network_id(network: KgwNetwork) -> &'static str {
    match network {
        KgwNetwork::Mainnet => "mainnet",
        KgwNetwork::Testnet10 => "testnet-10",
        KgwNetwork::Testnet12 => "testnet-12",
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
    network: KgwNetwork,
    endpoint: &str,
    actual_network: &str,
) -> Result<(), KgwRealOwnerError> {
    let expected = expected_rpc_network_id(network);
    if actual_network == expected {
        Ok(())
    } else {
        Err(KgwRealOwnerError::NodeRpcNetworkMismatch {
            expected: expected.to_string(),
            actual: actual_network.to_string(),
            endpoint: endpoint.to_string(),
        })
    }
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn attest_mainline_node_rpc_readiness(
    settings: &NodeSettings,
    core_terminal_outcome: &mut std::sync::mpsc::Receiver<String>,
) -> Result<String, KgwRealOwnerError> {
    use kaspa_rpc_core_mainline::api::rpc::RpcApi;

    let endpoint = settings.rpc_endpoint.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| KgwRealOwnerError::OwnerThreadStartFailed(error.to_string()))?;

    runtime.block_on(async move {
        let deadline = tokio::time::Instant::now() + KGW_NODE_READINESS_TIMEOUT;
        let mut last_error = "RPC endpoint has not responded yet".to_string();

        loop {
            fail_if_core_terminated(core_terminal_outcome, settings)?;

            if tokio::time::Instant::now() >= deadline {
                return Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
                    "network={};endpoint={};timeout_ms={};last_error={}",
                    settings.network.as_str(),
                    endpoint,
                    KGW_NODE_READINESS_TIMEOUT.as_millis(),
                    last_error
                )));
            }

            let attempt = tokio::time::timeout(Duration::from_secs(3), async {
                let client = kaspa_grpc_client_mainline::GrpcClient::connect(grpc_endpoint(&endpoint))
                    .await
                    .map_err(|error| error.to_string())?;
                client
                    .get_server_info()
                    .await
                    .map_err(|error| error.to_string())
            })
            .await;

            match attempt {
                Ok(Ok(info)) => {
                    let actual_network = info.network_id.to_string();
                    validate_rpc_network_identity(settings.network, &endpoint, &actual_network)?;

                    return Ok(format!(
                        "rpc_method=get_server_info;rpc_network={};rpc_endpoint={};server_version={}",
                        actual_network, endpoint, info.server_version
                    ));
                }
                Ok(Err(error)) => last_error = error,
                Err(_) => last_error = "RPC connection attempt timed out".to_string(),
            }

            tokio::time::sleep(KGW_NODE_READINESS_RETRY_INTERVAL).await;
        }
    })
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
fn attest_mainline_node_rpc_readiness(
    settings: &NodeSettings,
    _core_terminal_outcome: &mut std::sync::mpsc::Receiver<String>,
) -> Result<String, KgwRealOwnerError> {
    Err(KgwRealOwnerError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-mainline",
        settings.network.as_str()
    )))
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn attest_tn12_node_rpc_readiness(
    settings: &NodeSettings,
    core_terminal_outcome: &mut std::sync::mpsc::Receiver<String>,
) -> Result<String, KgwRealOwnerError> {
    use kaspa_rpc_core_tn12::api::rpc::RpcApi;

    let endpoint = settings.rpc_endpoint.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| KgwRealOwnerError::OwnerThreadStartFailed(error.to_string()))?;

    runtime.block_on(async move {
        let deadline = tokio::time::Instant::now() + KGW_NODE_READINESS_TIMEOUT;
        let mut last_error = "RPC endpoint has not responded yet".to_string();

        loop {
            fail_if_core_terminated(core_terminal_outcome, settings)?;

            if tokio::time::Instant::now() >= deadline {
                return Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
                    "network={};endpoint={};timeout_ms={};last_error={}",
                    settings.network.as_str(),
                    endpoint,
                    KGW_NODE_READINESS_TIMEOUT.as_millis(),
                    last_error
                )));
            }

            let attempt = tokio::time::timeout(Duration::from_secs(3), async {
                let client = kaspa_grpc_client_tn12::GrpcClient::connect(grpc_endpoint(&endpoint))
                    .await
                    .map_err(|error| error.to_string())?;
                client
                    .get_server_info()
                    .await
                    .map_err(|error| error.to_string())
            })
            .await;

            match attempt {
                Ok(Ok(info)) => {
                    let actual_network = info.network_id.to_string();
                    validate_rpc_network_identity(settings.network, &endpoint, &actual_network)?;

                    return Ok(format!(
                        "rpc_method=get_server_info;rpc_network={};rpc_endpoint={};server_version={}",
                        actual_network, endpoint, info.server_version
                    ));
                }
                Ok(Err(error)) => last_error = error,
                Err(_) => last_error = "RPC connection attempt timed out".to_string(),
            }

            tokio::time::sleep(KGW_NODE_READINESS_RETRY_INTERVAL).await;
        }
    })
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
fn attest_tn12_node_rpc_readiness(
    settings: &NodeSettings,
    _core_terminal_outcome: &mut std::sync::mpsc::Receiver<String>,
) -> Result<String, KgwRealOwnerError> {
    Err(KgwRealOwnerError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-tn12",
        settings.network.as_str()
    )))
}

#[cfg(any(
    test,
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn fail_if_core_terminated(
    core_terminal_outcome: &mut std::sync::mpsc::Receiver<String>,
    settings: &NodeSettings,
) -> Result<(), KgwRealOwnerError> {
    match core_terminal_outcome.try_recv() {
        Ok(error) => Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
            "network={};endpoint={};terminal_error={error}",
            settings.network.as_str(),
            settings.rpc_endpoint
        ))),
        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
            Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
                "network={};endpoint={};terminal_error=official core outcome channel disconnected",
                settings.network.as_str(),
                settings.rpc_endpoint
            )))
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => Ok(()),
    }
}

static KGW_SINGLE_INPROC_OFFICIAL_CORE_CLAIM: OnceLock<Mutex<Option<KgwNetwork>>> = OnceLock::new();

fn kgw_claim_single_inproc_official_core(network: KgwNetwork) -> Result<(), KgwRealOwnerError> {
    let claim = KGW_SINGLE_INPROC_OFFICIAL_CORE_CLAIM.get_or_init(|| Mutex::new(None));
    let mut guard = claim.lock().map_err(|_| KgwRealOwnerError::LockFailed)?;

    match *guard {
        Some(existing) if existing != network => {
            Err(KgwRealOwnerError::OfficialCoreAlreadyClaimed(format!(
                "already_started_network={};requested_network={};reason=Rusty Kaspa initializes a process-global logger; starting a second in-process network causes SetLoggerError and can terminate the desktop process; use one in-process network per desktop process or run other networks through an isolated process owner",
                existing.as_str(),
                network.as_str()
            )))
        }
        Some(_) => Ok(()),
        None => {
            *guard = Some(network);
            Ok(())
        }
    }
}

#[allow(dead_code)]
fn kgw_runtime_appdir_root_string() -> String {
    kgw_owner_safe_runtime_appdir_root()
        .to_string_lossy()
        .to_string()
}
fn spawn_official_core_thread(
    settings: NodeSettings,
) -> Result<(JoinHandle<()>, std::sync::mpsc::Receiver<String>), KgwRealOwnerError> {
    match settings.network {
        KgwNetwork::Mainnet | KgwNetwork::Testnet10 => spawn_mainline_core_thread(settings),
        KgwNetwork::Testnet12 => spawn_tn12_core_thread(settings),
    }
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn spawn_mainline_core_thread(
    settings: NodeSettings,
) -> Result<(JoinHandle<()>, std::sync::mpsc::Receiver<String>), KgwRealOwnerError> {
    let mut args = kaspad_lib_mainline::args::Args {
        appdir: Some(settings.app_dir_name.clone()),
        utxoindex: settings.enable_utxo_index,
        archival: settings.archival,
        yes: true,
        disable_upnp: true,
        log_level: "INFO".to_string(),
        ..Default::default()
    };

    if settings.network == KgwNetwork::Testnet10 {
        args.testnet = true;
        args.testnet_suffix = 10;
    }

    args.listen = settings
        .p2p_listen
        .as_deref()
        .map(|listen| {
            listen
                .parse::<kaspa_utils_mainline::networking::ContextualNetAddress>()
                .map_err(|error| KgwRealOwnerError::InvalidP2pListen(error.to_string()))
        })
        .transpose()?;

    args.rpclisten = Some(
        settings
            .rpc_endpoint
            .parse::<kaspa_utils_mainline::networking::ContextualNetAddress>()
            .map_err(|error| KgwRealOwnerError::InvalidRpcEndpoint(error.to_string()))?,
    );

    kgw_apply_embedded_fd_limits_mainline(&mut args);
    let fd_total_budget = kgw_embedded_core_fd_budget(
        kaspa_utils_mainline::fd_budget::limit(),
        args.rpc_max_clients as i32,
        args.inbound_limit as i32,
        args.outbound_target as i32,
    );

    let network = settings.network;
    let (terminal_tx, terminal_rx) = std::sync::mpsc::sync_channel(1);

    let owner_thread = std::thread::Builder::new()
        .name(format!("kaspad-{}", network.as_str()))
        .spawn(move || {
            let outcome = kgw_run_official_core_with_panic_boundary(network, move || {
                let (core, rpc_core_service) =
                    kaspad_lib_mainline::daemon::create_core(args, fd_total_budget);

                let _keep_rpc_core_service_alive = rpc_core_service;
                core.run();
            });
            let _ = terminal_tx.send(outcome);
        })
        .map_err(|error| KgwRealOwnerError::OwnerThreadStartFailed(error.to_string()))?;

    Ok((owner_thread, terminal_rx))
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
fn spawn_mainline_core_thread(
    settings: NodeSettings,
) -> Result<(JoinHandle<()>, std::sync::mpsc::Receiver<String>), KgwRealOwnerError> {
    Err(KgwRealOwnerError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-mainline",
        settings.network.as_str()
    )))
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn spawn_tn12_core_thread(
    settings: NodeSettings,
) -> Result<(JoinHandle<()>, std::sync::mpsc::Receiver<String>), KgwRealOwnerError> {
    let mut args = kaspad_lib_tn12::args::Args {
        appdir: Some(settings.app_dir_name.clone()),
        utxoindex: settings.enable_utxo_index,
        archival: settings.archival,
        yes: true,
        disable_upnp: true,
        log_level: "INFO".to_string(),
        ..Default::default()
    };
    args.testnet = true;
    args.testnet_suffix = match settings.network {
        KgwNetwork::Testnet10 => 10,
        KgwNetwork::Testnet12 => 12,
        KgwNetwork::Mainnet => args.testnet_suffix,
    };

    args.listen = settings
        .p2p_listen
        .as_deref()
        .map(|listen| {
            listen
                .parse::<kaspa_utils_tn12::networking::ContextualNetAddress>()
                .map_err(|error| KgwRealOwnerError::InvalidP2pListen(error.to_string()))
        })
        .transpose()?;

    args.rpclisten = Some(
        settings
            .rpc_endpoint
            .parse::<kaspa_utils_tn12::networking::ContextualNetAddress>()
            .map_err(|error| KgwRealOwnerError::InvalidRpcEndpoint(error.to_string()))?,
    );

    kgw_apply_embedded_fd_limits_tn12(&mut args);
    let fd_total_budget = kgw_embedded_core_fd_budget(
        kaspa_utils_tn12::fd_budget::limit(),
        args.rpc_max_clients as i32,
        args.inbound_limit as i32,
        args.outbound_target as i32,
    );

    let network = settings.network;
    let (terminal_tx, terminal_rx) = std::sync::mpsc::sync_channel(1);

    let owner_thread = std::thread::Builder::new()
        .name(format!("kaspad-{}", network.as_str()))
        .spawn(move || {
            let outcome = kgw_run_official_core_with_panic_boundary(network, move || {
                let (core, rpc_core_service) =
                    kaspad_lib_tn12::daemon::create_core(args, fd_total_budget);

                let _keep_rpc_core_service_alive = rpc_core_service;
                core.run();
            });
            let _ = terminal_tx.send(outcome);
        })
        .map_err(|error| KgwRealOwnerError::OwnerThreadStartFailed(error.to_string()))?;

    Ok((owner_thread, terminal_rx))
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
fn spawn_tn12_core_thread(
    settings: NodeSettings,
) -> Result<(JoinHandle<()>, std::sync::mpsc::Receiver<String>), KgwRealOwnerError> {
    Err(KgwRealOwnerError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-tn12",
        settings.network.as_str()
    )))
}

#[allow(dead_code)]
fn kgw_embedded_core_fd_budget(
    fd_limit: i32,
    rpc_max_clients: i32,
    inbound_limit: i32,
    outbound_target: i32,
) -> i32 {
    let usable_limit = fd_limit.max(128);
    let owner_reserved = rpc_max_clients
        .saturating_add(inbound_limit)
        .saturating_add(outbound_target);
    let safety_margin = 96;

    usable_limit
        .saturating_sub(owner_reserved)
        .saturating_sub(safety_margin)
        .clamp(64, 256)
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn kgw_apply_embedded_fd_limits_mainline(args: &mut kaspad_lib_mainline::args::Args) {
    args.rpc_max_clients = args.rpc_max_clients.min(16);
    args.inbound_limit = args.inbound_limit.min(32);
    args.outbound_target = args.outbound_target.min(8);
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn kgw_apply_embedded_fd_limits_tn12(args: &mut kaspad_lib_tn12::args::Args) {
    args.rpc_max_clients = args.rpc_max_clients.min(16);
    args.inbound_limit = args.inbound_limit.min(32);
    args.outbound_target = args.outbound_target.min(8);
}

#[allow(dead_code)]
fn kgw_run_official_core_with_panic_boundary<F>(network: KgwNetwork, run_core: F) -> String
where
    F: FnOnce(),
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_core)) {
        Ok(()) => format!(
            "official Rusty Kaspa core returned;network={}",
            network.as_str()
        ),
        Err(payload) => format!(
            "official Rusty Kaspa core panicked;network={};panic={}",
            network.as_str(),
            panic_payload_message(payload.as_ref())
        ),
    }
}

fn panic_payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else {
        "non-string panic payload".to_string()
    }
}
fn start_bridge_owner_if_requested(
    settings: &NodeSettings,
) -> Result<Option<kaspa_gateway_rk_bridge::BridgeOwnerRuntimeHandle>, String> {
    if settings.bridge_kind == BridgeNodeKind::Disable {
        return Ok(None);
    }

    let bridge_mode = match settings.bridge_kind {
        BridgeNodeKind::Disable => kaspa_gateway_rk_bridge::BridgeRuntimeMode::Disabled,
        BridgeNodeKind::OfficialExternalNode => {
            kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialExternalNode
        }
        BridgeNodeKind::OfficialInProcessNode => {
            kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialInProcessNode
        }
    };

    let bridge_settings = kaspa_gateway_rk_bridge::BridgeRuntimeSettings {
        network: settings.network.as_str().to_string(),
        mode: bridge_mode,
        stratum_listen: Some(settings.stratum_listen.clone()),
        prometheus_listen: Some(default_prometheus_for_network(settings.network).to_string()),
        kaspa_rpc_endpoint: Some(settings.rpc_endpoint.clone()),
        internal_cpu_miner: kaspa_gateway_rk_bridge::BridgeInternalCpuMinerSettings {
            enabled: settings.bridge_internal_cpu_miner,
            address: settings.bridge_internal_cpu_miner_address.clone(),
            threads: settings.bridge_internal_cpu_miner_threads,
            throttle_ms: settings.bridge_internal_cpu_miner_throttle_ms,
            template_poll_ms: settings.bridge_internal_cpu_miner_template_poll_ms,
        },
        explicit_runtime_opt_in: true,
    };

    let bridge_event =
        kaspa_gateway_rk_bridge::bridge_service_event_from_settings_v1(bridge_settings)
            .map_err(|error| error.to_string())?;

    kaspa_gateway_rk_bridge::start_official_bridge_owner_thread_ready_v1(bridge_event)
        .map(|(handle, _readiness)| Some(handle))
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
fn kgw_owner_safe_runtime_appdir_root() -> std::path::PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("KaspaGateway")
            .join("nodes")
    } else {
        std::env::temp_dir().join("KaspaGateway").join("nodes")
    }
}

#[allow(dead_code)]
fn kgw_owner_safe_runtime_appdir(network: KgwNetwork) -> String {
    kgw_owner_safe_runtime_appdir_root()
        .join(network.as_str())
        .to_string_lossy()
        .to_string()
}
fn default_prometheus_for_network(network: KgwNetwork) -> &'static str {
    match network {
        KgwNetwork::Mainnet => "127.0.0.1:2114",
        KgwNetwork::Testnet10 => "127.0.0.1:12114",
        KgwNetwork::Testnet12 => "127.0.0.1:22114",
    }
}

pub fn real_owner_runtime_summary_v1() -> &'static str {
    "R18 real KGW owner runtime: requests enter NodeSettings -> KaspadServiceEvents::from_node_settings -> service_events.sender -> controller -> owner runtime session -> official Rusty Kaspa create_core -> core.run -> bridge KaspaApi -> listen_and_serve_with_shutdown."
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod kgw_runtime_fd_budget_tests {
    use super::*;

    #[test]
    fn embedded_fd_budget_fits_windows_default_limit() {
        let fd_budget = kgw_embedded_core_fd_budget(512, 16, 32, 8);

        assert!(fd_budget >= 64);
        assert!(fd_budget <= 256);
        assert!(fd_budget + 16 + 32 + 8 + 96 <= 512);
    }

    #[test]
    fn embedded_fd_budget_removes_legacy_1024_floor_behavior() {
        let fd_budget = kgw_embedded_core_fd_budget(512, 128, 128, 8);

        assert!(fd_budget < 512);
        assert_ne!(fd_budget, 1024);
    }

    #[test]
    fn rpc_readiness_requires_exact_network_identity() {
        assert_eq!(expected_rpc_network_id(KgwNetwork::Mainnet), "mainnet");
        assert_eq!(expected_rpc_network_id(KgwNetwork::Testnet10), "testnet-10");
        assert_eq!(expected_rpc_network_id(KgwNetwork::Testnet12), "testnet-12");
    }

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    #[test]
    fn mainline_owner_args_honor_isolated_p2p_listener() {
        let listen = "127.0.0.1:26111";
        let parsed = listen
            .parse::<kaspa_utils_mainline::networking::ContextualNetAddress>()
            .unwrap();
        assert_eq!(parsed.to_string(), listen);
    }

    #[cfg(feature = "official-kaspa-runtime-tn12")]
    #[test]
    fn tn12_owner_args_honor_isolated_p2p_listener() {
        let listen = "127.0.0.1:26311";
        let parsed = listen
            .parse::<kaspa_utils_tn12::networking::ContextualNetAddress>()
            .unwrap();
        assert_eq!(parsed.to_string(), listen);
    }

    #[test]
    fn core_terminal_outcome_preserves_panic_root_cause_without_stderr_wrapper() {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = kgw_run_official_core_with_panic_boundary(KgwNetwork::Mainnet, || {
            panic!("delayed core initialization failure")
        });
        std::panic::set_hook(previous_hook);

        assert!(outcome.contains("official Rusty Kaspa core panicked"));
        assert!(outcome.contains("delayed core initialization failure"));
    }

    #[test]
    fn readiness_fails_immediately_after_core_terminal_outcome() {
        let (terminal_tx, mut terminal_rx) = std::sync::mpsc::sync_channel(1);
        terminal_tx
            .send("official core returned during initialization".to_string())
            .unwrap();
        let settings = NodeSettings::from_strings(
            "mainnet".to_string(),
            "integrated-inproc".to_string(),
            "disable".to_string(),
        )
        .unwrap();

        let error = fail_if_core_terminated(&mut terminal_rx, &settings).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("official core returned during initialization")
        );
    }

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    #[test]
    fn unreachable_rpc_endpoint_produces_failed_readiness() {
        let unavailable = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = unavailable.local_addr().unwrap().to_string();
        drop(unavailable);
        let (_terminal_tx, mut terminal_rx) = std::sync::mpsc::sync_channel(1);
        let mut settings = NodeSettings::from_strings(
            "mainnet".to_string(),
            "integrated-inproc".to_string(),
            "disable".to_string(),
        )
        .unwrap();
        settings.rpc_endpoint = endpoint.clone();

        let error = attest_mainline_node_rpc_readiness(&settings, &mut terminal_rx).unwrap_err();
        let error = error.to_string();
        assert!(error.contains("node RPC readiness failed"));
        assert!(error.contains(&endpoint));
        assert!(error.contains("timeout_ms=750"));
    }

    #[test]
    fn wrong_rpc_network_produces_typed_mismatch() {
        let error = validate_rpc_network_identity(
            KgwNetwork::Testnet10,
            "127.0.0.1:16210",
            expected_rpc_network_id(KgwNetwork::Mainnet),
        )
        .unwrap_err();
        let text = error.to_string();
        assert!(text.contains("expected_network=testnet-10"));
        assert!(text.contains("actual_network=mainnet"));
    }
}
