use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::kgw_real_owner_runtime::KgwRealOwnerRuntime;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KgwNetwork {
    Mainnet,
    Testnet10,
    Testnet12,
}

impl KgwNetwork {
    pub fn parse(value: &str) -> Result<Self, KgwServiceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" | "testnet10" => Ok(Self::Testnet10),
            "testnet12" | "tn12" => Ok(Self::Testnet12),
            _ => Err(KgwServiceError::UnsupportedNetwork(value.to_string())),
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

    pub fn rpc_endpoint(self) -> &'static str {
        match self {
            Self::Mainnet => "127.0.0.1:16110",
            Self::Testnet10 => "127.0.0.1:16210",
            Self::Testnet12 => "127.0.0.1:16310",
        }
    }

    pub fn stratum_listen(self) -> &'static str {
        match self {
            Self::Mainnet => "0.0.0.0:5555",
            Self::Testnet10 => "0.0.0.0:15555",
            Self::Testnet12 => "0.0.0.0:25555",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspadNodeKind {
    Disable,
    IntegratedInProc,
    IntegratedAsDaemon,
    IntegratedAsPassiveSync,
    Remote,
}

impl KaspadNodeKind {
    pub fn parse(value: &str) -> Result<Self, KgwServiceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disable" | "disabled" | "stop" => Ok(Self::Disable),
            "integrated-inproc" | "inproc" | "inprocess" => Ok(Self::IntegratedInProc),
            "integrated-as-daemon" | "daemon" => Ok(Self::IntegratedAsDaemon),
            "integrated-as-passive-sync" | "passive-sync" => Ok(Self::IntegratedAsPassiveSync),
            "remote" | "remote-rpc" => Ok(Self::Remote),
            _ => Err(KgwServiceError::UnsupportedNodeKind(value.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::IntegratedInProc => "integrated-inproc",
            Self::IntegratedAsDaemon => "integrated-as-daemon",
            Self::IntegratedAsPassiveSync => "integrated-as-passive-sync",
            Self::Remote => "remote",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeNodeKind {
    Disable,
    OfficialExternalNode,
    OfficialInProcessNode,
}

impl BridgeNodeKind {
    pub fn parse(value: &str) -> Result<Self, KgwServiceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "disable" | "disabled" | "stop" => Ok(Self::Disable),
            "official-external-node" | "external" | "external-node" => {
                Ok(Self::OfficialExternalNode)
            }
            "official-inprocess-node" | "inproc" | "inprocess" | "inprocess-node" => {
                Ok(Self::OfficialInProcessNode)
            }
            _ => Err(KgwServiceError::UnsupportedBridgeKind(value.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::OfficialExternalNode => "official-external-node",
            Self::OfficialInProcessNode => "official-inprocess-node",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSettings {
    pub network: KgwNetwork,
    pub node_kind: KaspadNodeKind,
    pub bridge_kind: BridgeNodeKind,
    pub rpc_endpoint: String,
    pub stratum_listen: String,
    pub app_dir_name: String,
    pub enable_utxo_index: bool,
    pub archival: bool,

    pub bridge_internal_cpu_miner: bool,
    pub bridge_internal_cpu_miner_address: Option<String>,
    pub bridge_internal_cpu_miner_threads: Option<u16>,
    pub bridge_internal_cpu_miner_throttle_ms: Option<u64>,
    pub bridge_internal_cpu_miner_template_poll_ms: Option<u64>,
}

fn kgw_service_runtime_appdir_root_string() -> String {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("rusty-kaspa")
            .to_string_lossy()
            .to_string()
    } else {
        std::env::temp_dir()
            .join("rusty-kaspa")
            .to_string_lossy()
            .to_string()
    }
}
impl NodeSettings {
    pub fn from_strings(
        network: String,
        node_kind: String,
        bridge_kind: String,
    ) -> Result<Self, KgwServiceError> {
        let network = KgwNetwork::parse(&network)?;
        let node_kind = KaspadNodeKind::parse(&node_kind)?;
        let bridge_kind = BridgeNodeKind::parse(&bridge_kind)?;

        if bridge_kind == BridgeNodeKind::OfficialInProcessNode
            && node_kind != KaspadNodeKind::IntegratedInProc
        {
            return Err(KgwServiceError::InProcessBridgeRequiresInProcessNode);
        }

        Ok(Self {
            network,
            node_kind,
            bridge_kind,
            rpc_endpoint: network.rpc_endpoint().to_string(),
            stratum_listen: network.stratum_listen().to_string(),
            app_dir_name: kgw_service_runtime_appdir_root_string(),
            enable_utxo_index: true,
            archival: false,

            bridge_internal_cpu_miner: false,
            bridge_internal_cpu_miner_address: None,
            bridge_internal_cpu_miner_threads: None,
            bridge_internal_cpu_miner_throttle_ms: None,
            bridge_internal_cpu_miner_template_poll_ms: None,
        })
    }

    pub fn command_preview(&self) -> String {
        let mut parts = vec!["kaspad".to_string()];

        match self.network {
            KgwNetwork::Mainnet => {}
            KgwNetwork::Testnet10 => {
                parts.push("--testnet".to_string());
                parts.push("--netsuffix=10".to_string());
            }
            KgwNetwork::Testnet12 => {
                parts.push("--testnet".to_string());
                parts.push("--netsuffix=12".to_string());
            }
        }

        if self.enable_utxo_index {
            parts.push("--utxoindex".to_string());
        }

        if self.archival {
            parts.push("--archival".to_string());
        }

        parts.push(format!("--rpclisten={}", self.rpc_endpoint));
        parts.push(format!("--appdir={}", self.app_dir_name));
        parts.join(" ")
    }

    pub fn bridge_command_preview(&self) -> String {
        [
            "kaspa-stratum-bridge".to_string(),
            format!("--network={}", self.network.as_str()),
            format!("--branch={}", self.network.branch()),
            format!("--node-kind={}", self.bridge_kind.as_str()),
            format!("--kaspa-rpc={}", self.rpc_endpoint),
            format!("--stratum-listen={}", self.stratum_listen),
        ]
        .into_iter()
        .chain(if self.bridge_internal_cpu_miner {
            vec![
                "--internal-cpu-miner".to_string(),
                self.bridge_internal_cpu_miner_address
                    .as_ref()
                    .map(|address| format!("--internal-cpu-miner-address={}", address))
                    .unwrap_or_default(),
                self.bridge_internal_cpu_miner_threads
                    .map(|threads| format!("--internal-cpu-miner-threads={}", threads))
                    .unwrap_or_default(),
                self.bridge_internal_cpu_miner_throttle_ms
                    .map(|value| format!("--internal-cpu-miner-throttle-ms={}", value))
                    .unwrap_or_default(),
                self.bridge_internal_cpu_miner_template_poll_ms
                    .map(|value| format!("--internal-cpu-miner-template-poll-ms={}", value))
                    .unwrap_or_default(),
            ]
        } else {
            Vec::new()
        })
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KaspadServiceEvents {
    Disable {
        network: KgwNetwork,
    },
    StartInternalInProc {
        settings: NodeSettings,
        network: KgwNetwork,
    },
    StartInternalAsDaemon {
        settings: NodeSettings,
        network: KgwNetwork,
    },
    StartInternalAsPassiveSync {
        settings: NodeSettings,
        network: KgwNetwork,
    },
    StartRemoteConnection {
        settings: NodeSettings,
        network: KgwNetwork,
    },
    StartBridgeExternalNode {
        settings: NodeSettings,
        network: KgwNetwork,
    },
    StartBridgeInProcessNode {
        settings: NodeSettings,
        network: KgwNetwork,
    },
    Stdout {
        network: KgwNetwork,
        line: String,
    },
    Exit,
}

impl KaspadServiceEvents {
    pub fn from_node_settings(settings: &NodeSettings) -> Result<Vec<Self>, KgwServiceError> {
        let mut events = Vec::new();

        let node_event = match settings.node_kind {
            KaspadNodeKind::Disable => Self::Disable {
                network: settings.network,
            },
            KaspadNodeKind::IntegratedInProc => Self::StartInternalInProc {
                settings: settings.clone(),
                network: settings.network,
            },
            KaspadNodeKind::IntegratedAsDaemon => Self::StartInternalAsDaemon {
                settings: settings.clone(),
                network: settings.network,
            },
            KaspadNodeKind::IntegratedAsPassiveSync => Self::StartInternalAsPassiveSync {
                settings: settings.clone(),
                network: settings.network,
            },
            KaspadNodeKind::Remote => Self::StartRemoteConnection {
                settings: settings.clone(),
                network: settings.network,
            },
        };

        events.push(node_event);

        match settings.bridge_kind {
            BridgeNodeKind::Disable => {}
            BridgeNodeKind::OfficialExternalNode => {
                events.push(Self::StartBridgeExternalNode {
                    settings: settings.clone(),
                    network: settings.network,
                });
            }
            BridgeNodeKind::OfficialInProcessNode => {
                if settings.node_kind != KaspadNodeKind::IntegratedInProc {
                    return Err(KgwServiceError::InProcessBridgeRequiresInProcessNode);
                }

                events.push(Self::StartBridgeInProcessNode {
                    settings: settings.clone(),
                    network: settings.network,
                });
            }
        }

        Ok(events)
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Disable { .. } => "Disable",
            Self::StartInternalInProc { .. } => "StartInternalInProc",
            Self::StartInternalAsDaemon { .. } => "StartInternalAsDaemon",
            Self::StartInternalAsPassiveSync { .. } => "StartInternalAsPassiveSync",
            Self::StartRemoteConnection { .. } => "StartRemoteConnection",
            Self::StartBridgeExternalNode { .. } => "StartBridgeExternalNode",
            Self::StartBridgeInProcessNode { .. } => "StartBridgeInProcessNode",
            Self::Stdout { .. } => "Stdout",
            Self::Exit => "Exit",
        }
    }

    pub fn network(&self) -> Option<KgwNetwork> {
        match self {
            Self::Disable { network }
            | Self::StartInternalInProc { network, .. }
            | Self::StartInternalAsDaemon { network, .. }
            | Self::StartInternalAsPassiveSync { network, .. }
            | Self::StartRemoteConnection { network, .. }
            | Self::StartBridgeExternalNode { network, .. }
            | Self::StartBridgeInProcessNode { network, .. }
            | Self::Stdout { network, .. } => Some(*network),
            Self::Exit => None,
        }
    }

    pub fn to_log_line(&self) -> String {
        match self.network() {
            Some(network) => format!(
                "event={};network={};branch={}",
                self.kind(),
                network.as_str(),
                network.branch()
            ),
            None => format!("event={}", self.kind()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSlotStatus {
    pub network: KgwNetwork,
    pub branch: &'static str,
    pub node_kind: KaspadNodeKind,
    pub bridge_kind: BridgeNodeKind,
    pub node_running: bool,
    pub bridge_running: bool,
    pub rpc_attached: bool,
    pub last_event: String,
    pub last_error: Option<String>,
    pub logs: Vec<String>,
}

impl RuntimeSlotStatus {
    pub fn new(network: KgwNetwork) -> Self {
        Self {
            network,
            branch: network.branch(),
            node_kind: KaspadNodeKind::Disable,
            bridge_kind: BridgeNodeKind::Disable,
            node_running: false,
            bridge_running: false,
            rpc_attached: false,
            last_event: "not-started".to_string(),
            last_error: None,
            logs: Vec::new(),
        }
    }

    pub fn to_log_line(&self) -> String {
        format!(
            "network={};branch={};node_kind={};bridge_kind={};node_running={};bridge_running={};rpc_attached={};last_event={};last_error={}",
            self.network.as_str(),
            self.branch,
            self.node_kind.as_str(),
            self.bridge_kind.as_str(),
            self.node_running,
            self.bridge_running,
            self.rpc_attached,
            self.last_event,
            self.last_error.clone().unwrap_or_else(|| "none".to_string())
        )
    }
}

#[derive(Debug, Error)]
pub enum KgwServiceError {
    #[error("unsupported network: {0}")]
    UnsupportedNetwork(String),

    #[error("unsupported node kind: {0}")]
    UnsupportedNodeKind(String),

    #[error("unsupported bridge kind: {0}")]
    UnsupportedBridgeKind(String),

    #[error("in-process bridge requires integrated in-process node")]
    InProcessBridgeRequiresInProcessNode,

    #[error("service controller channel is closed")]
    ChannelClosed,

    #[error("service controller lock failed")]
    LockFailed,
}

#[derive(Debug)]
struct ControllerState {
    real_owner_runtime: KgwRealOwnerRuntime,
    slots: HashMap<KgwNetwork, RuntimeSlotStatus>,
    logs: VecDeque<String>,
    exit_requested: bool,
}

impl ControllerState {
    fn new() -> Self {
        let mut slots = HashMap::new();
        for network in [
            KgwNetwork::Mainnet,
            KgwNetwork::Testnet10,
            KgwNetwork::Testnet12,
        ] {
            slots.insert(network, RuntimeSlotStatus::new(network));
        }

        Self {
            real_owner_runtime: KgwRealOwnerRuntime::new(),
            slots,
            logs: VecDeque::new(),
            exit_requested: false,
        }
    }

    fn push_log(&mut self, network: Option<KgwNetwork>, line: impl Into<String>) {
        let line = format!("[{}] {}", timestamp_ms(), line.into());

        if self.logs.len() >= 512 {
            self.logs.pop_back();
        }
        self.logs.push_front(line.clone());

        if let Some(network) = network {
            if let Some(slot) = self.slots.get_mut(&network) {
                if slot.logs.len() >= 128 {
                    slot.logs.remove(0);
                }
                slot.logs.push(line);
            }
        }
    }

    fn slot_mut(&mut self, network: KgwNetwork) -> &mut RuntimeSlotStatus {
        self.slots
            .entry(network)
            .or_insert_with(|| RuntimeSlotStatus::new(network))
    }
}

pub struct KgwServiceController {
    sender: Mutex<Sender<KaspadServiceEvents>>,
    state: Arc<Mutex<ControllerState>>,
}

impl KgwServiceController {
    pub fn spawn() -> Arc<Self> {
        let (sender, receiver) = mpsc::channel::<KaspadServiceEvents>();
        let state = Arc::new(Mutex::new(ControllerState::new()));
        let worker_state = state.clone();

        thread::Builder::new()
            .name("kgw-kgw-service-controller".to_string())
            .spawn(move || {
                Self::event_loop(receiver, worker_state);
            })
            .expect("failed to spawn kgw service controller");

        Arc::new(Self {
            sender: Mutex::new(sender),
            state,
        })
    }

    pub fn apply_node_settings(&self, settings: NodeSettings) -> Result<String, KgwServiceError> {
        let events = KaspadServiceEvents::from_node_settings(&settings)?;

        for event in events {
            self.send_event(event)?;
        }

        Ok(format!(
            "apply_node_settings accepted;network={};node_kind={};bridge_kind={};branch={};runtime_appdir={};command_preview={};bridge_command_preview={}",
            settings.network.as_str(),
            settings.node_kind.as_str(),
            settings.bridge_kind.as_str(),
            settings.network.branch(),
            settings.app_dir_name,
            settings.command_preview(),
            settings.bridge_command_preview()
        ))
    }

    pub fn disable_network(&self, network: KgwNetwork) -> Result<String, KgwServiceError> {
        self.send_event(KaspadServiceEvents::Disable { network })?;

        Ok(format!(
            "disable accepted;network={};branch={}",
            network.as_str(),
            network.branch()
        ))
    }
    pub fn status(&self, network: Option<KgwNetwork>) -> Result<String, KgwServiceError> {
        let state = self.state.lock().map_err(|_| KgwServiceError::LockFailed)?;

        if let Some(network) = network {
            let slot = state
                .slots
                .get(&network)
                .cloned()
                .unwrap_or_else(|| RuntimeSlotStatus::new(network));

            return Ok(slot.to_log_line());
        }

        let mut lines = Vec::new();

        for network in [
            KgwNetwork::Mainnet,
            KgwNetwork::Testnet10,
            KgwNetwork::Testnet12,
        ] {
            let slot = state
                .slots
                .get(&network)
                .cloned()
                .unwrap_or_else(|| RuntimeSlotStatus::new(network));

            lines.push(slot.to_log_line());
        }

        Ok(lines.join("\n"))
    }

    pub fn logs(&self, network: Option<KgwNetwork>) -> Result<String, KgwServiceError> {
        let state = self.state.lock().map_err(|_| KgwServiceError::LockFailed)?;

        if let Some(network) = network {
            let slot = state
                .slots
                .get(&network)
                .cloned()
                .unwrap_or_else(|| RuntimeSlotStatus::new(network));

            return Ok(slot.logs.join("\n"));
        }

        Ok(state.logs.iter().cloned().collect::<Vec<_>>().join("\n"))
    }

    fn send_event(&self, event: KaspadServiceEvents) -> Result<(), KgwServiceError> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| KgwServiceError::LockFailed)?;
        sender
            .send(event)
            .map_err(|_| KgwServiceError::ChannelClosed)
    }

    fn event_loop(receiver: Receiver<KaspadServiceEvents>, state: Arc<Mutex<ControllerState>>) {
        while let Ok(event) = receiver.recv() {
            let should_exit = Self::handle_event(event, &state);
            if should_exit {
                break;
            }
        }
    }

    fn handle_event(event: KaspadServiceEvents, state: &Arc<Mutex<ControllerState>>) -> bool {
        let mut state = match state.lock() {
            Ok(state) => state,
            Err(_) => return true,
        };

        let network = event.network();
        state.push_log(network, format!("received {}", event.to_log_line()));

        match event {
            KaspadServiceEvents::Disable { network } => {
                Self::stop_all_services_for_network(&mut state, network);
                let slot = state.slot_mut(network);
                slot.node_kind = KaspadNodeKind::Disable;
                slot.bridge_kind = BridgeNodeKind::Disable;
                slot.last_event = "Disable".to_string();
                false
            }

            KaspadServiceEvents::StartInternalInProc { settings, network } => {
                Self::stop_all_services_for_network(&mut state, network);
                Self::handle_network_change(&mut state, network);
                Self::start_internal_inproc(&mut state, settings);
                false
            }

            KaspadServiceEvents::StartInternalAsDaemon { settings, network } => {
                Self::stop_all_services_for_network(&mut state, network);
                Self::handle_network_change(&mut state, network);
                Self::start_internal_as_daemon(&mut state, settings);
                false
            }

            KaspadServiceEvents::StartInternalAsPassiveSync { settings, network } => {
                Self::stop_all_services_for_network(&mut state, network);
                Self::handle_network_change(&mut state, network);
                Self::start_internal_as_passive_sync(&mut state, settings);
                false
            }

            KaspadServiceEvents::StartRemoteConnection { settings, network } => {
                Self::stop_all_services_for_network(&mut state, network);
                Self::handle_network_change(&mut state, network);
                Self::start_remote_connection(&mut state, settings);
                false
            }

            KaspadServiceEvents::StartBridgeExternalNode { settings, network } => {
                Self::handle_network_change(&mut state, network);
                Self::start_bridge_external_node(&mut state, settings);
                false
            }

            KaspadServiceEvents::StartBridgeInProcessNode { settings, network } => {
                Self::handle_network_change(&mut state, network);
                Self::start_bridge_inprocess_node(&mut state, settings);
                false
            }

            KaspadServiceEvents::Stdout { network, line } => {
                state.push_log(Some(network), format!("stdout {}", line));
                let slot = state.slot_mut(network);
                slot.last_event = "Stdout".to_string();
                false
            }

            KaspadServiceEvents::Exit => {
                state.exit_requested = true;
                state.push_log(None, "Exit requested");
                true
            }
        }
    }

    fn stop_all_services_for_network(state: &mut ControllerState, network: KgwNetwork) {
        state.push_log(
            Some(network),
            "kgw real owner step: stop_all_services_for_network",
        );

        match state.real_owner_runtime.stop_network(network) {
            Ok(runtime_status) => {
                state.push_log(
                    Some(network),
                    format!("real-owner-runtime-stop {}", runtime_status.to_log_line()),
                );
            }
            Err(error) => {
                state.push_log(
                    Some(network),
                    format!("real-owner-runtime-stop-error {}", error),
                );
            }
        }

        let slot = state.slot_mut(network);
        slot.node_running = false;
        slot.bridge_running = false;
        slot.rpc_attached = false;
        slot.last_event = "stop_all_services".to_string();
    }
    fn handle_network_change(state: &mut ControllerState, network: KgwNetwork) {
        state.push_log(
            Some(network),
            format!(
                "kgw step: handle_network_change network={} branch={}",
                network.as_str(),
                network.branch()
            ),
        );
    }

    fn start_internal_inproc(state: &mut ControllerState, settings: NodeSettings) {
        let network = settings.network;

        state.push_log(
            Some(network),
            "kgw real owner step: StartInternalInProc -> owner runtime session",
        );

        match state.real_owner_runtime.start_node_owner_session(&settings) {
            Ok(runtime_status) => {
                state.push_log(
                    Some(network),
                    format!("real-owner-runtime {}", runtime_status.to_log_line()),
                );

                let slot = state.slot_mut(network);
                slot.node_kind = settings.node_kind;
                slot.bridge_kind = settings.bridge_kind;
                slot.node_running = runtime_status.official_core_running;
                slot.bridge_running = runtime_status.bridge_owner_active;
                slot.rpc_attached = runtime_status.official_core_running;
                slot.last_event = "StartInternalInProc".to_string();
                slot.last_error = None;
            }
            Err(error) => {
                let error_text = error.to_string();
                state.push_log(
                    Some(network),
                    format!("real-owner-runtime-error {error_text}"),
                );

                let slot = state.slot_mut(network);
                slot.node_running = false;
                slot.bridge_running = false;
                slot.rpc_attached = false;
                slot.last_event = "StartInternalInProcFailed".to_string();
                slot.last_error = Some(error_text);
            }
        }
    }
    fn start_internal_as_daemon(state: &mut ControllerState, settings: NodeSettings) {
        let network = settings.network;

        state.push_log(
            Some(network),
            "kgw real owner step: StartInternalAsDaemon is handled by same owner runtime session; no external daemon process is used",
        );

        match state.real_owner_runtime.start_node_owner_session(&settings) {
            Ok(runtime_status) => {
                let slot = state.slot_mut(network);
                slot.node_kind = settings.node_kind;
                slot.bridge_kind = settings.bridge_kind;
                slot.node_running = runtime_status.official_core_running;
                slot.bridge_running = runtime_status.bridge_owner_active;
                slot.rpc_attached = runtime_status.official_core_running;
                slot.last_event = "StartInternalAsDaemon".to_string();
                slot.last_error = None;
            }
            Err(error) => {
                let slot = state.slot_mut(network);
                slot.node_running = false;
                slot.bridge_running = false;
                slot.rpc_attached = false;
                slot.last_event = "StartInternalAsDaemonFailed".to_string();
                slot.last_error = Some(error.to_string());
            }
        }
    }
    fn start_internal_as_passive_sync(state: &mut ControllerState, settings: NodeSettings) {
        let network = settings.network;

        state.push_log(
            Some(network),
            "kgw real owner step: StartInternalAsPassiveSync is handled by owner runtime session",
        );

        match state.real_owner_runtime.start_node_owner_session(&settings) {
            Ok(runtime_status) => {
                let slot = state.slot_mut(network);
                slot.node_kind = settings.node_kind;
                slot.bridge_kind = settings.bridge_kind;
                slot.node_running = runtime_status.official_core_running;
                slot.bridge_running = runtime_status.bridge_owner_active;
                slot.rpc_attached = runtime_status.official_core_running;
                slot.last_event = "StartInternalAsPassiveSync".to_string();
                slot.last_error = None;
            }
            Err(error) => {
                let slot = state.slot_mut(network);
                slot.node_running = false;
                slot.bridge_running = false;
                slot.rpc_attached = false;
                slot.last_event = "StartInternalAsPassiveSyncFailed".to_string();
                slot.last_error = Some(error.to_string());
            }
        }
    }
    fn start_remote_connection(state: &mut ControllerState, settings: NodeSettings) {
        let network = settings.network;

        state.push_log(
            Some(network),
            format!(
                "kgw step: StartRemoteConnection endpoint={}",
                settings.rpc_endpoint
            ),
        );

        let slot = state.slot_mut(network);
        slot.node_kind = settings.node_kind;
        slot.bridge_kind = settings.bridge_kind;
        slot.node_running = false;
        slot.rpc_attached = true;
        slot.last_event = "StartRemoteConnection".to_string();
        slot.last_error = None;
    }

    fn start_bridge_external_node(state: &mut ControllerState, settings: NodeSettings) {
        let network = settings.network;

        state.push_log(
            Some(network),
            format!(
                "kgw bridge step: StartBridgeExternalNode rpc={} stratum={} branch={}",
                settings.rpc_endpoint,
                settings.stratum_listen,
                network.branch()
            ),
        );

        let slot = state.slot_mut(network);
        slot.bridge_kind = BridgeNodeKind::OfficialExternalNode;
        slot.bridge_running = true;
        slot.last_event = "StartBridgeExternalNode".to_string();
        slot.last_error = None;
    }

    fn start_bridge_inprocess_node(state: &mut ControllerState, settings: NodeSettings) {
        let network = settings.network;

        state.push_log(
            Some(network),
            format!(
                "kgw bridge step: StartBridgeInProcessNode rpc={} stratum={} branch={}",
                settings.rpc_endpoint,
                settings.stratum_listen,
                network.branch()
            ),
        );

        let slot = state.slot_mut(network);
        slot.bridge_kind = BridgeNodeKind::OfficialInProcessNode;
        slot.bridge_running = true;
        slot.last_event = "StartBridgeInProcessNode".to_string();
        slot.last_error = None;
    }
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

pub fn exact_kgw_service_controller_summary_v1() -> &'static str {
    "Exact KGW controller flow: NodeSettings -> KaspadServiceEvents::from_node_settings -> service_events sender -> controller event loop -> handle_event lifecycle. mainnet/testnet10 use master; testnet12 uses tn12."
}
