//! Read-only observations from the exact embedded runtime family.
//! Process/startup ownership is deliberately separate from RPC and mining activity.
use crate::{BridgeRuntimeFamily, BridgeRuntimeNetwork};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::time::{Duration, Instant};

pub const OBSERVATION_PREFIX: &str = "KGW_RUNTIME_OBSERVATION_V1 ";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeObservation {
    pub version: u8,
    pub role: String,
    pub network: String,
    pub worker_pid: u32,
    pub rpc_ready: bool,
    pub synced: Option<bool>,
    pub virtual_daa_score: Option<u64>,
    pub cpu_enabled: bool,
    pub cpu_hashes_tried: Option<u64>,
    pub cpu_blocks_submitted: Option<u64>,
    // Upstream increments blocks_accepted only after a BLUE-in-DAG confirmation.
    pub cpu_blocks_confirmed_blue: Option<u64>,
    pub cpu_hashrate_hs: Option<f64>,
    pub error: Option<String>,
}
type CpuReader = Arc<dyn Fn() -> (u64, u64, u64) + Send + Sync>;
static CPU_READERS: OnceLock<Mutex<HashMap<&'static str, CpuReader>>> = OnceLock::new();
pub(crate) fn set_cpu_reader(network: BridgeRuntimeNetwork, reader: CpuReader) {
    if let Ok(mut readers) = CPU_READERS.get_or_init(Default::default).lock() {
        readers.insert(network.as_str(), reader);
    }
}
fn cpu_counts(network: BridgeRuntimeNetwork) -> Option<(u64, u64, u64)> {
    let reader = CPU_READERS
        .get()?
        .lock()
        .ok()?
        .get(network.as_str())
        .cloned()?;
    Some(reader())
}
pub struct RuntimeObserver {
    stop: mpsc::Sender<()>,
    owner: Option<std::thread::JoinHandle<()>>,
}
impl Drop for RuntimeObserver {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(owner) = self.owner.take() {
            let _ = owner.join();
        }
    }
}
pub fn start_runtime_observer(
    network: BridgeRuntimeNetwork,
    role: &str,
    endpoint: &str,
) -> Result<RuntimeObserver, String> {
    if !matches!(role, "node" | "bridge") {
        return Err("Invalid observation role".into());
    }
    let (stop, receiver) = mpsc::channel();
    let role = role.to_owned();
    let endpoint = endpoint.to_owned();
    let owner = std::thread::Builder::new()
        .name(format!("kgw-observer-{}", network.as_str()))
        .spawn(move || {
            let mut previous: Option<(Instant, u64)> = None;
            loop {
                if receiver.try_recv().is_ok() {
                    break;
                }
                // A short-lived current-thread runtime also drops all RPC tasks on timeout.
                let rpc = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime.block_on(async {
                        tokio::time::timeout(
                            Duration::from_secs(2),
                            observe_rpc(network, &endpoint),
                        )
                        .await
                        .map_err(|_| "RPC observation timed out".to_string())?
                    }),
                    Err(error) => Err(error.to_string()),
                };
                let now = Instant::now();
                let counts = (role == "bridge").then(|| cpu_counts(network)).flatten();
                let rate = counts.and_then(|(hashes, _, _)| {
                    previous.and_then(|(at, before)| {
                        let seconds = now.duration_since(at).as_secs_f64();
                        (seconds > 0.0 && hashes >= before)
                            .then(|| (hashes - before) as f64 / seconds)
                    })
                });
                previous = counts.map(|(hashes, _, _)| (now, hashes));
                let observation = RuntimeObservation {
                    version: 1,
                    role: role.clone(),
                    network: network.as_str().into(),
                    worker_pid: std::process::id(),
                    rpc_ready: rpc.is_ok(),
                    synced: rpc.as_ref().ok().map(|info| info.0),
                    virtual_daa_score: rpc.as_ref().ok().map(|info| info.1),
                    cpu_enabled: counts.is_some(),
                    cpu_hashes_tried: counts.map(|c| c.0),
                    cpu_blocks_submitted: counts.map(|c| c.1),
                    cpu_blocks_confirmed_blue: counts.map(|c| c.2),
                    cpu_hashrate_hs: rate,
                    error: rpc.err(),
                };
                if let Ok(json) = serde_json::to_string(&observation) {
                    // This is real native worker output; the parent retains it unchanged.
                    eprintln!("{OBSERVATION_PREFIX}{json}");
                }
                match receiver.recv_timeout(Duration::from_secs(2)) {
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    _ => break,
                }
            }
        })
        .map_err(|error| error.to_string())?;
    Ok(RuntimeObserver {
        stop,
        owner: Some(owner),
    })
}
async fn observe_rpc(network: BridgeRuntimeNetwork, endpoint: &str) -> Result<(bool, u64), String> {
    match network.family() {
        BridgeRuntimeFamily::Mainline => observe_mainline(network, endpoint).await,
        BridgeRuntimeFamily::Tn13 => observe_tn13(network, endpoint).await,
    }
}
#[cfg(feature = "official-kaspa-runtime-mainline")]
async fn observe_mainline(
    network: BridgeRuntimeNetwork,
    endpoint: &str,
) -> Result<(bool, u64), String> {
    use kaspa_rpc_core_mainline::api::rpc::RpcApi;
    let endpoint = if endpoint.starts_with("grpc://") {
        endpoint.to_owned()
    } else {
        format!("grpc://{endpoint}")
    };
    let client = kaspa_grpc_client_mainline::GrpcClient::connect(endpoint)
        .await
        .map_err(|e| e.to_string())?;
    let response = tokio::time::timeout(Duration::from_secs(1), client.get_server_info()).await;
    let _ = tokio::time::timeout(Duration::from_millis(500), client.disconnect()).await;
    let info = response
        .map_err(|_| "get_server_info timed out".to_string())?
        .map_err(|e| e.to_string())?;
    let expected = match network {
        BridgeRuntimeNetwork::Mainnet => "mainnet",
        BridgeRuntimeNetwork::Testnet10 => "testnet-10",
        BridgeRuntimeNetwork::Testnet13 => "testnet-13",
    };
    let actual = info.network_id.to_string();
    if actual != expected {
        return Err(format!(
            "RPC network mismatch: expected={expected}; actual={actual}"
        ));
    }
    Ok((info.is_synced, info.virtual_daa_score))
}
#[cfg(not(feature = "official-kaspa-runtime-mainline"))]
async fn observe_mainline(
    _network: BridgeRuntimeNetwork,
    _endpoint: &str,
) -> Result<(bool, u64), String> {
    Err("official-kaspa-runtime-mainline is not compiled".into())
}
#[cfg(feature = "official-kaspa-runtime-tn13")]
async fn observe_tn13(
    network: BridgeRuntimeNetwork,
    endpoint: &str,
) -> Result<(bool, u64), String> {
    use kaspa_rpc_core_tn13::api::rpc::RpcApi;
    let endpoint = if endpoint.starts_with("grpc://") {
        endpoint.to_owned()
    } else {
        format!("grpc://{endpoint}")
    };
    let client = kaspa_grpc_client_tn13::GrpcClient::connect(endpoint)
        .await
        .map_err(|e| e.to_string())?;
    let response = tokio::time::timeout(Duration::from_secs(1), client.get_server_info()).await;
    let _ = tokio::time::timeout(Duration::from_millis(500), client.disconnect()).await;
    let info = response
        .map_err(|_| "get_server_info timed out".to_string())?
        .map_err(|e| e.to_string())?;
    let expected = match network {
        BridgeRuntimeNetwork::Mainnet => "mainnet",
        BridgeRuntimeNetwork::Testnet10 => "testnet-10",
        BridgeRuntimeNetwork::Testnet13 => "testnet-13",
    };
    let actual = info.network_id.to_string();
    if actual != expected {
        return Err(format!(
            "RPC network mismatch: expected={expected}; actual={actual}"
        ));
    }
    Ok((info.is_synced, info.virtual_daa_score))
}
#[cfg(not(feature = "official-kaspa-runtime-tn13"))]
async fn observe_tn13(
    _network: BridgeRuntimeNetwork,
    _endpoint: &str,
) -> Result<(bool, u64), String> {
    Err("official-kaspa-runtime-tn13 is not compiled".into())
}
