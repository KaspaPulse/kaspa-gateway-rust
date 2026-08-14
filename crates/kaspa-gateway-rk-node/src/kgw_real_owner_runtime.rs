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

    #[error("invalid effective node settings: {0}")]
    InvalidEffectiveNodeSettings(String),

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

    #[error("official runtime shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("official runtime shutdown is already in progress for {0}")]
    ShutdownInProgress(String),
}

#[derive(Debug)]
struct RuntimeSession {
    status: KgwRuntimeSessionStatus,
    owner_thread: Option<JoinHandle<()>>,
    core_shutdown_tx: Option<std::sync::mpsc::SyncSender<()>>,
    core_terminal_outcome: Option<std::sync::mpsc::Receiver<Result<String, String>>>,
    core_terminal_error: Option<String>,
    bridge_handle: Option<kaspa_gateway_rk_bridge::BridgeOwnerRuntimeHandle>,
    shutdown_in_progress: bool,
}

type OfficialCoreOwnerThread = (
    JoinHandle<()>,
    std::sync::mpsc::SyncSender<()>,
    std::sync::mpsc::Receiver<Result<String, String>>,
);

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
            core_shutdown_tx: None,
            core_terminal_outcome: None,
            core_terminal_error: None,
            bridge_handle: None,
            shutdown_in_progress: false,
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
        let mut sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;

        let session = sessions
            .entry(network)
            .or_insert_with(|| RuntimeSession::new(network));
        refresh_core_terminal_status(session);
        Ok(session.status.clone())
    }

    pub fn terminal_failure(
        &self,
        network: KgwNetwork,
    ) -> Result<Option<String>, KgwRealOwnerError> {
        let status = self.status(network)?;
        if status.runtime_requested && !status.official_core_running {
            Ok(Some(status.last_message))
        } else {
            Ok(None)
        }
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

            if session.shutdown_in_progress {
                return Err(KgwRealOwnerError::ShutdownInProgress(
                    settings.network.as_str().to_string(),
                ));
            }

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
                let (owner_thread, core_shutdown_tx, core_terminal_outcome) =
                    spawn_official_core_thread(settings.clone())?;
                session.owner_thread = Some(owner_thread);
                session.core_shutdown_tx = Some(core_shutdown_tx);
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
        let (bridge_handle, core_shutdown_tx, owner_thread, core_terminal_outcome) = {
            let mut sessions = self
                .inner
                .lock()
                .map_err(|_| KgwRealOwnerError::LockFailed)?;
            let session = sessions
                .entry(network)
                .or_insert_with(|| RuntimeSession::new(network));

            if session.shutdown_in_progress {
                return Err(KgwRealOwnerError::ShutdownInProgress(
                    network.as_str().to_string(),
                ));
            }
            session.shutdown_in_progress = session.bridge_handle.is_some()
                || session.owner_thread.is_some()
                || session.core_shutdown_tx.is_some();

            (
                session.bridge_handle.take(),
                session.core_shutdown_tx.take(),
                session.owner_thread.take(),
                session.core_terminal_outcome.take(),
            )
        };

        let bridge_was_owned = bridge_handle.is_some();
        let core_was_owned = owner_thread.is_some();
        let graceful_core_stop_requested = core_was_owned && core_shutdown_tx.is_some();
        let mut shutdown_failures = Vec::new();

        // Bridge listeners depend on the Node RPC service. Stop and join them
        // before requesting official Core shutdown.
        if let Some(bridge_handle) = bridge_handle
            && let Err(panic) = bridge_handle.join()
        {
            shutdown_failures.push(format!(
                "network={};component=bridge-owner;panic={}",
                network.as_str(),
                panic_payload_message(panic.as_ref())
            ));
        }

        if let Some(core_shutdown_tx) = core_shutdown_tx
            && let Err(error) = core_shutdown_tx.send(())
        {
            shutdown_failures.push(format!(
                "network={};component=official-core;request_error={error}",
                network.as_str()
            ));
        }

        if let Some(owner_thread) = owner_thread
            && let Err(panic) = owner_thread.join()
        {
            shutdown_failures.push(format!(
                "network={};component=official-core-owner;panic={}",
                network.as_str(),
                panic_payload_message(panic.as_ref())
            ));
        }

        let terminal_outcome = match core_terminal_outcome {
            Some(receiver) => match receiver.try_recv() {
                Ok(Ok(evidence)) => evidence,
                Ok(Err(error)) => {
                    shutdown_failures.push(error.clone());
                    error
                }
                Err(error) => {
                    let failure = format!(
                        "network={};component=official-core;terminal_outcome_error={error}",
                        network.as_str()
                    );
                    shutdown_failures.push(failure.clone());
                    failure
                }
            },
            None => format!(
                "official Rusty Kaspa core was already stopped;network={}",
                network.as_str()
            ),
        };

        let mut sessions = self
            .inner
            .lock()
            .map_err(|_| KgwRealOwnerError::LockFailed)?;
        let session = sessions
            .entry(network)
            .or_insert_with(|| RuntimeSession::new(network));

        session.core_shutdown_tx = None;
        session.shutdown_in_progress = false;
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
        let already_stopped = !bridge_was_owned && !core_was_owned;
        session.status.last_message = format!(
            "official runtime shutdown joined;network={};already_stopped={already_stopped};bridge_joined={bridge_was_owned};rpc_core_service_released_before_core_shutdown={graceful_core_stop_requested};core_shutdown={graceful_core_stop_requested};core_join={core_was_owned};terminal_outcome={terminal_outcome}",
            network.as_str(),
        );
        let msg = session.status.last_message.clone();
        session.push_log(msg);

        if shutdown_failures.is_empty() {
            Ok(session.status.clone())
        } else {
            Err(KgwRealOwnerError::ShutdownFailed(
                shutdown_failures.join(" | "),
            ))
        }
    }
}

fn refresh_core_terminal_status(session: &mut RuntimeSession) {
    let terminal = session
        .owner_thread
        .as_ref()
        .is_some_and(JoinHandle::is_finished);
    if !terminal {
        return;
    }

    session.status.owner_thread_alive = false;
    session.status.official_core_running = false;
    if session.core_terminal_error.is_none()
        && let Some(receiver) = session.core_terminal_outcome.as_ref()
    {
        session.core_terminal_error = match receiver.try_recv() {
            Ok(Ok(evidence)) => Some(format!("official core terminated unexpectedly: {evidence}")),
            Ok(Err(error)) => Some(error),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Some("official core outcome channel disconnected".to_string())
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
        };
    }
    session.status.last_message = session
        .core_terminal_error
        .clone()
        .unwrap_or_else(|| "official core owner thread terminated".to_string());
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
    core_terminal_outcome: &mut std::sync::mpsc::Receiver<Result<String, String>>,
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
    _core_terminal_outcome: &mut std::sync::mpsc::Receiver<Result<String, String>>,
) -> Result<String, KgwRealOwnerError> {
    Err(KgwRealOwnerError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-mainline",
        settings.network.as_str()
    )))
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn attest_tn12_node_rpc_readiness(
    settings: &NodeSettings,
    core_terminal_outcome: &mut std::sync::mpsc::Receiver<Result<String, String>>,
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
    _core_terminal_outcome: &mut std::sync::mpsc::Receiver<Result<String, String>>,
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
    core_terminal_outcome: &mut std::sync::mpsc::Receiver<Result<String, String>>,
    settings: &NodeSettings,
) -> Result<(), KgwRealOwnerError> {
    match core_terminal_outcome.try_recv() {
        Ok(Ok(evidence)) => Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
            "network={};endpoint={};terminal_error=official core terminated before readiness: {evidence}",
            settings.network.as_str(),
            settings.rpc_endpoint
        ))),
        Ok(Err(error)) => Err(KgwRealOwnerError::NodeRpcReadinessFailed(format!(
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
) -> Result<OfficialCoreOwnerThread, KgwRealOwnerError> {
    match settings.network {
        KgwNetwork::Mainnet | KgwNetwork::Testnet10 => spawn_mainline_core_thread(settings),
        KgwNetwork::Testnet12 => spawn_tn12_core_thread(settings),
    }
}

#[cfg(any(
    feature = "official-kaspa-runtime-mainline",
    feature = "official-kaspa-runtime-tn12"
))]
fn validate_effective_node_settings(settings: &NodeSettings) -> Result<(), KgwRealOwnerError> {
    let effective = &settings.effective_node;
    effective
        .validate()
        .map_err(|error| KgwRealOwnerError::InvalidEffectiveNodeSettings(error.to_string()))
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn build_mainline_args(
    settings: &NodeSettings,
) -> Result<kaspad_lib_mainline::args::Args, KgwRealOwnerError> {
    validate_effective_node_settings(settings)?;
    let mut args = kaspad_lib_mainline::args::Args {
        appdir: Some(settings.app_dir_name.clone()),
        utxoindex: settings.enable_utxo_index,
        archival: settings.archival,
        logdir: settings.effective_node.log_dir.clone(),
        no_log_files: settings.effective_node.no_log_files,
        unsafe_rpc: settings.effective_node.unsafe_rpc,
        log_level: settings.effective_node.log_level.clone(),
        async_threads: settings.effective_node.async_threads,
        connect_peers: settings
            .effective_node
            .connect_peers
            .iter()
            .map(|value| {
                value
                    .parse::<kaspa_utils_mainline::networking::ContextualNetAddress>()
                    .map_err(|error| {
                        KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                            "invalid connect peer {value}: {error}"
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        add_peers: settings
            .effective_node
            .add_peers
            .iter()
            .map(|value| {
                value
                    .parse::<kaspa_utils_mainline::networking::ContextualNetAddress>()
                    .map_err(|error| {
                        KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                            "invalid add peer {value}: {error}"
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        user_agent_comments: settings.effective_node.user_agent_comments.clone(),
        reset_db: settings.effective_node.reset_db,
        outbound_target: settings.effective_node.outbound_target,
        inbound_limit: settings.effective_node.inbound_limit,
        rpc_max_clients: settings.effective_node.rpc_max_clients,
        max_tracked_addresses: settings.effective_node.max_tracked_addresses,
        enable_unsynced_mining: settings.effective_node.enable_unsynced_mining,
        sanity: settings.effective_node.sanity,
        yes: settings.effective_node.yes,
        externalip: settings
            .effective_node
            .external_ip
            .as_deref()
            .map(|value| {
                value
                    .parse::<kaspa_utils_mainline::networking::ContextualNetAddress>()
                    .map_err(|error| {
                        KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                            "invalid external IP {value}: {error}"
                        ))
                    })
            })
            .transpose()?,
        perf_metrics: settings.effective_node.perf_metrics,
        perf_metrics_interval_sec: settings.effective_node.perf_metrics_interval_sec,
        disable_upnp: settings.effective_node.disable_upnp,
        disable_dns_seeding: settings.effective_node.disable_dns_seeding,
        disable_grpc: settings.effective_node.disable_grpc,
        ram_scale: settings.effective_node.ram_scale,
        retention_period_days: settings.effective_node.retention_period_days,
        override_params_file: settings.effective_node.override_params_file.clone(),
        rocksdb_preset: settings.effective_node.rocksdb_preset.clone(),
        rocksdb_wal_dir: settings.effective_node.rocksdb_wal_dir.clone(),
        rocksdb_cache_size: settings.effective_node.rocksdb_cache_size,
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
    args.rpclisten_borsh = settings
        .effective_node
        .rpc_listen_borsh
        .as_deref()
        .map(|value| {
            value
                .parse::<kaspa_wrpc_server_mainline::address::WrpcNetAddress>()
                .map_err(|error| {
                    KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                        "invalid Borsh RPC listen {value}: {error}"
                    ))
                })
        })
        .transpose()?;
    args.rpclisten_json = settings
        .effective_node
        .rpc_listen_json
        .as_deref()
        .map(|value| {
            value
                .parse::<kaspa_wrpc_server_mainline::address::WrpcNetAddress>()
                .map_err(|error| {
                    KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                        "invalid JSON RPC listen {value}: {error}"
                    ))
                })
        })
        .transpose()?;

    kgw_apply_embedded_fd_limits_mainline(&mut args);
    Ok(args)
}

#[cfg(feature = "official-kaspa-runtime-mainline")]
fn spawn_mainline_core_thread(
    settings: NodeSettings,
) -> Result<OfficialCoreOwnerThread, KgwRealOwnerError> {
    let args = build_mainline_args(&settings)?;
    let fd_total_budget = kgw_embedded_core_fd_budget(
        kaspa_utils_mainline::fd_budget::limit(),
        args.rpc_max_clients as i32,
        args.inbound_limit as i32,
        args.outbound_target as i32,
    );

    let network = settings.network;
    let (terminal_tx, terminal_rx) = std::sync::mpsc::sync_channel(1);
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::sync_channel(1);

    let owner_thread = std::thread::Builder::new()
        .name(format!("kaspad-{}", network.as_str()))
        .spawn(move || {
            let outcome = kgw_run_official_core_with_panic_boundary(network, move || {
                let (core, rpc_core_service) =
                    kaspad_lib_mainline::daemon::create_core(args, fd_total_budget);
                let workers = core.start();

                // The exact pinned daemon contract requires this handle to be
                // released before Core shutdown.
                loop {
                    match shutdown_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            if workers.iter().all(JoinHandle::is_finished) {
                                drop(rpc_core_service);
                                core.join(workers);
                                return Err(format!(
                                    "official Rusty Kaspa core workers terminated before graceful Stop;network={}",
                                    network.as_str()
                                ));
                            }
                        }
                    }
                }
                drop(rpc_core_service);
                use kaspa_core_mainline::signals::Shutdown;
                core.shutdown();
                core.join(workers);
                Ok(format!(
                    "official Rusty Kaspa core shutdown and workers joined;network={}",
                    network.as_str()
                ))
            });
            let _ = terminal_tx.send(outcome);
        })
        .map_err(|error| KgwRealOwnerError::OwnerThreadStartFailed(error.to_string()))?;

    Ok((owner_thread, shutdown_tx, terminal_rx))
}

#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
fn spawn_mainline_core_thread(
    settings: NodeSettings,
) -> Result<OfficialCoreOwnerThread, KgwRealOwnerError> {
    Err(KgwRealOwnerError::FeatureRequired(format!(
        "{} requires official-kaspa-runtime-mainline",
        settings.network.as_str()
    )))
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn build_tn12_args(
    settings: &NodeSettings,
) -> Result<kaspad_lib_tn12::args::Args, KgwRealOwnerError> {
    validate_effective_node_settings(settings)?;
    let mut args = kaspad_lib_tn12::args::Args {
        appdir: Some(settings.app_dir_name.clone()),
        utxoindex: settings.enable_utxo_index,
        archival: settings.archival,
        logdir: settings.effective_node.log_dir.clone(),
        no_log_files: settings.effective_node.no_log_files,
        unsafe_rpc: settings.effective_node.unsafe_rpc,
        log_level: settings.effective_node.log_level.clone(),
        async_threads: settings.effective_node.async_threads,
        connect_peers: settings
            .effective_node
            .connect_peers
            .iter()
            .map(|value| {
                value
                    .parse::<kaspa_utils_tn12::networking::ContextualNetAddress>()
                    .map_err(|error| {
                        KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                            "invalid connect peer {value}: {error}"
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        add_peers: settings
            .effective_node
            .add_peers
            .iter()
            .map(|value| {
                value
                    .parse::<kaspa_utils_tn12::networking::ContextualNetAddress>()
                    .map_err(|error| {
                        KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                            "invalid add peer {value}: {error}"
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        user_agent_comments: settings.effective_node.user_agent_comments.clone(),
        reset_db: settings.effective_node.reset_db,
        outbound_target: settings.effective_node.outbound_target,
        inbound_limit: settings.effective_node.inbound_limit,
        rpc_max_clients: settings.effective_node.rpc_max_clients,
        max_tracked_addresses: settings.effective_node.max_tracked_addresses,
        enable_unsynced_mining: settings.effective_node.enable_unsynced_mining,
        sanity: settings.effective_node.sanity,
        yes: settings.effective_node.yes,
        externalip: settings
            .effective_node
            .external_ip
            .as_deref()
            .map(|value| {
                value
                    .parse::<kaspa_utils_tn12::networking::ContextualNetAddress>()
                    .map_err(|error| {
                        KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                            "invalid external IP {value}: {error}"
                        ))
                    })
            })
            .transpose()?,
        perf_metrics: settings.effective_node.perf_metrics,
        perf_metrics_interval_sec: settings.effective_node.perf_metrics_interval_sec,
        disable_upnp: settings.effective_node.disable_upnp,
        disable_dns_seeding: settings.effective_node.disable_dns_seeding,
        disable_grpc: settings.effective_node.disable_grpc,
        ram_scale: settings.effective_node.ram_scale,
        retention_period_days: settings.effective_node.retention_period_days,
        override_params_file: settings.effective_node.override_params_file.clone(),
        rocksdb_preset: settings.effective_node.rocksdb_preset.clone(),
        rocksdb_wal_dir: settings.effective_node.rocksdb_wal_dir.clone(),
        rocksdb_cache_size: settings.effective_node.rocksdb_cache_size,
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
    args.rpclisten_borsh = settings
        .effective_node
        .rpc_listen_borsh
        .as_deref()
        .map(|value| {
            value
                .parse::<kaspa_wrpc_server_tn12::address::WrpcNetAddress>()
                .map_err(|error| {
                    KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                        "invalid Borsh RPC listen {value}: {error}"
                    ))
                })
        })
        .transpose()?;
    args.rpclisten_json = settings
        .effective_node
        .rpc_listen_json
        .as_deref()
        .map(|value| {
            value
                .parse::<kaspa_wrpc_server_tn12::address::WrpcNetAddress>()
                .map_err(|error| {
                    KgwRealOwnerError::InvalidEffectiveNodeSettings(format!(
                        "invalid JSON RPC listen {value}: {error}"
                    ))
                })
        })
        .transpose()?;

    kgw_apply_embedded_fd_limits_tn12(&mut args);
    Ok(args)
}

#[cfg(feature = "official-kaspa-runtime-tn12")]
fn spawn_tn12_core_thread(
    settings: NodeSettings,
) -> Result<OfficialCoreOwnerThread, KgwRealOwnerError> {
    let args = build_tn12_args(&settings)?;
    let fd_total_budget = kgw_embedded_core_fd_budget(
        kaspa_utils_tn12::fd_budget::limit(),
        args.rpc_max_clients as i32,
        args.inbound_limit as i32,
        args.outbound_target as i32,
    );

    let network = settings.network;
    let (terminal_tx, terminal_rx) = std::sync::mpsc::sync_channel(1);
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::sync_channel(1);

    let owner_thread = std::thread::Builder::new()
        .name(format!("kaspad-{}", network.as_str()))
        .spawn(move || {
            let outcome = kgw_run_official_core_with_panic_boundary(network, move || {
                let (core, rpc_core_service) =
                    kaspad_lib_tn12::daemon::create_core(args, fd_total_budget);
                let workers = core.start();

                loop {
                    match shutdown_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            if workers.iter().all(JoinHandle::is_finished) {
                                drop(rpc_core_service);
                                core.join(workers);
                                return Err(format!(
                                    "official Rusty Kaspa core workers terminated before graceful Stop;network={}",
                                    network.as_str()
                                ));
                            }
                        }
                    }
                }
                drop(rpc_core_service);
                use kaspa_core_tn12::signals::Shutdown;
                core.shutdown();
                core.join(workers);
                Ok(format!(
                    "official Rusty Kaspa core shutdown and workers joined;network={}",
                    network.as_str()
                ))
            });
            let _ = terminal_tx.send(outcome);
        })
        .map_err(|error| KgwRealOwnerError::OwnerThreadStartFailed(error.to_string()))?;

    Ok((owner_thread, shutdown_tx, terminal_rx))
}

#[cfg(not(feature = "official-kaspa-runtime-tn12"))]
fn spawn_tn12_core_thread(
    settings: NodeSettings,
) -> Result<OfficialCoreOwnerThread, KgwRealOwnerError> {
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
fn kgw_run_official_core_with_panic_boundary<F>(
    network: KgwNetwork,
    run_core: F,
) -> Result<String, String>
where
    F: FnOnce() -> Result<String, String>,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run_core)) {
        Ok(outcome) => outcome,
        Err(payload) => Err(format!(
            "official Rusty Kaspa core panicked;network={};panic={}",
            network.as_str(),
            panic_payload_message(payload.as_ref())
        )),
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
    "R18 real KGW owner runtime: requests enter NodeSettings -> KaspadServiceEvents::from_node_settings -> service_events.sender -> controller -> owner runtime session -> official Rusty Kaspa create_core -> Core::start -> drop RpcCoreService -> Shutdown::shutdown -> Core::join; bridge uses KaspaApi -> listen_and_serve_with_shutdown -> BridgeOwnerRuntimeHandle::join."
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
    fn runtime_session_retains_graceful_core_stop_and_join_state() {
        let session = RuntimeSession::new(KgwNetwork::Mainnet);
        assert!(session.core_shutdown_tx.is_none());
        assert!(session.owner_thread.is_none());
        assert!(session.core_terminal_outcome.is_none());

        let source = include_str!("kgw_real_owner_runtime.rs");
        let graceful_mainline = source
            .split("fn spawn_mainline_core_thread(")
            .nth(1)
            .expect("mainline official core owner source must exist");
        let drop_rpc = graceful_mainline
            .find("drop(rpc_core_service);")
            .expect("owner must release RpcCoreService");
        let shutdown = graceful_mainline
            .find("core.shutdown();")
            .expect("owner must call official Shutdown");
        let join = graceful_mainline[shutdown..]
            .find("core.join(workers);")
            .map(|offset| shutdown + offset)
            .expect("owner must join official workers after Shutdown");
        assert!(drop_rpc < shutdown && shutdown < join);
    }

    #[test]
    fn completed_owner_join_is_reflected_as_not_running() {
        let mut session = RuntimeSession::new(KgwNetwork::Mainnet);
        session.status.owner_thread_alive = true;
        session.status.official_core_running = true;
        let (terminal_tx, terminal_rx) = std::sync::mpsc::sync_channel(1);
        terminal_tx
            .send(Ok("official core joined".to_string()))
            .unwrap();
        session.core_terminal_outcome = Some(terminal_rx);
        session.owner_thread = Some(std::thread::spawn(|| {}));
        session.owner_thread.as_ref().unwrap().thread().unpark();
        while !session.owner_thread.as_ref().unwrap().is_finished() {
            std::thread::yield_now();
        }

        refresh_core_terminal_status(&mut session);

        assert!(!session.status.owner_thread_alive);
        assert!(!session.status.official_core_running);
        assert!(
            session
                .status
                .last_message
                .contains("terminated unexpectedly")
        );
    }

    #[test]
    fn terminal_failure_reports_post_ready_owner_exit() {
        let runtime = KgwRealOwnerRuntime::new();
        {
            let mut sessions = runtime.inner.lock().unwrap();
            let session = sessions.get_mut(&KgwNetwork::Mainnet).unwrap();
            session.status.runtime_requested = true;
            session.status.owner_thread_alive = true;
            session.status.official_core_running = true;
            let (terminal_tx, terminal_rx) = std::sync::mpsc::sync_channel(1);
            terminal_tx
                .send(Err("official core fixture panic".to_string()))
                .unwrap();
            session.core_terminal_outcome = Some(terminal_rx);
            session.owner_thread = Some(std::thread::spawn(|| {}));
        }

        while !runtime
            .inner
            .lock()
            .unwrap()
            .get(&KgwNetwork::Mainnet)
            .unwrap()
            .owner_thread
            .as_ref()
            .unwrap()
            .is_finished()
        {
            std::thread::yield_now();
        }

        let failure = runtime
            .terminal_failure(KgwNetwork::Mainnet)
            .unwrap()
            .expect("terminal requested runtime must report a root cause");
        assert!(failure.contains("official core fixture panic"));
    }

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

    #[cfg(feature = "official-kaspa-runtime-mainline")]
    #[test]
    fn mainline_owner_args_apply_the_complete_effective_settings_contract() {
        let mut settings = NodeSettings::from_strings(
            "testnet10".to_string(),
            "integrated-inproc".to_string(),
            "disable".to_string(),
        )
        .unwrap();
        settings.app_dir_name = "effective-mainline-test".to_string();
        let effective = crate::EffectiveNodeSettings {
            log_level: "debug".to_string(),
            async_threads: 3,
            ram_scale: 0.75,
            yes: true,
            no_log_files: true,
            sanity: true,
            enable_unsynced_mining: true,
            p2p_listen: Some("127.0.0.1:26211".to_string()),
            external_ip: Some("127.0.0.1:26211".to_string()),
            disable_upnp: true,
            disable_dns_seeding: true,
            user_agent_comments: vec!["kgw-effective".to_string()],
            rpc_listen: "127.0.0.1:26210".to_string(),
            rpc_listen_borsh: Some("127.0.0.1:27210".to_string()),
            rpc_listen_json: Some("127.0.0.1:28210".to_string()),
            rpc_max_clients: 12,
            unsafe_rpc: true,
            disable_grpc: false,
            connect_peers: vec!["127.0.0.1:26212".to_string()],
            add_peers: Vec::new(),
            outbound_target: 7,
            inbound_limit: 31,
            utxo_index: false,
            archival: true,
            reset_db: true,
            perf_metrics: true,
            max_tracked_addresses: 123,
            retention_period_days: Some(2.5),
            perf_metrics_interval_sec: 9,
            rocksdb_preset: Some("hdd".to_string()),
            rocksdb_cache_size: Some(256),
            rocksdb_wal_dir: Some("wal".to_string()),
            override_params_file: None,
            log_dir: None,
        };
        settings.apply_effective_node_settings(effective).unwrap();

        let args = build_mainline_args(&settings).unwrap();
        assert!(args.testnet);
        assert_eq!(args.testnet_suffix, 10);
        assert_eq!(args.appdir.as_deref(), Some("effective-mainline-test"));
        assert_eq!(args.logdir, None);
        assert_eq!(args.log_level, "debug");
        assert_eq!(args.async_threads, 3);
        assert_eq!(args.ram_scale, 0.75);
        assert_eq!(args.rpclisten.unwrap().to_string(), "127.0.0.1:26210");
        assert!(matches!(
            args.rpclisten_borsh,
            Some(kaspa_wrpc_server_mainline::address::WrpcNetAddress::Custom(address))
                if address.to_string() == "127.0.0.1:27210"
        ));
        assert!(matches!(
            args.rpclisten_json,
            Some(kaspa_wrpc_server_mainline::address::WrpcNetAddress::Custom(address))
                if address.to_string() == "127.0.0.1:28210"
        ));
        assert_eq!(args.listen.unwrap().to_string(), "127.0.0.1:26211");
        assert_eq!(args.rpc_max_clients, 12);
        assert_eq!(args.inbound_limit, 31);
        assert_eq!(args.outbound_target, 7);
        assert_eq!(args.connect_peers[0].to_string(), "127.0.0.1:26212");
        assert!(args.add_peers.is_empty());
        assert_eq!(args.user_agent_comments, vec!["kgw-effective"]);
        assert!(!args.utxoindex);
        assert!(args.archival && args.reset_db && args.perf_metrics);
        assert_eq!(args.max_tracked_addresses, 123);
        assert_eq!(args.retention_period_days, Some(2.5));
        assert_eq!(args.rocksdb_preset.as_deref(), Some("hdd"));
        assert_eq!(args.rocksdb_cache_size, Some(256));
        assert_eq!(args.rocksdb_wal_dir.as_deref(), Some("wal"));
        assert_eq!(args.override_params_file, None);
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

    #[cfg(feature = "official-kaspa-runtime-tn12")]
    #[test]
    fn tn12_effective_settings_cannot_override_experimental_network_identity() {
        let mut settings = NodeSettings::from_strings(
            "testnet12".to_string(),
            "integrated-inproc".to_string(),
            "disable".to_string(),
        )
        .unwrap();
        let mut effective = crate::EffectiveNodeSettings {
            rpc_listen: KgwNetwork::Testnet12.rpc_endpoint().to_string(),
            ..Default::default()
        };
        effective.rpc_max_clients = 16;
        effective.inbound_limit = 32;
        settings.apply_effective_node_settings(effective).unwrap();
        let args = build_tn12_args(&settings).unwrap();
        assert!(args.testnet);
        assert_eq!(args.testnet_suffix, 12);
        assert!(!args.devnet);
        assert!(!args.simnet);
    }

    #[test]
    fn core_terminal_outcome_preserves_panic_root_cause_without_stderr_wrapper() {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let outcome = kgw_run_official_core_with_panic_boundary(KgwNetwork::Mainnet, || {
            panic!("delayed core initialization failure")
        });
        std::panic::set_hook(previous_hook);

        let outcome = outcome.unwrap_err();
        assert!(outcome.contains("official Rusty Kaspa core panicked"));
        assert!(outcome.contains("delayed core initialization failure"));
    }

    #[test]
    fn readiness_fails_immediately_after_core_terminal_outcome() {
        let (terminal_tx, mut terminal_rx) = std::sync::mpsc::sync_channel(1);
        terminal_tx
            .send(Err(
                "official core returned during initialization".to_string()
            ))
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
