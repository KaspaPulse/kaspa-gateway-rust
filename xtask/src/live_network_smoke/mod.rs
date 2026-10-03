//! Native Windows smoke orchestration for the existing same-executable parent.
mod runner;
mod tcp;

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::Duration;

type Result<T> = std::result::Result<T, String>;
const NO_PROGRESS_WARNING: &str = "No chain counter changed during the short observation window. This is informational during initial block download; use an extended synchronization test for production readiness.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet10,
}
impl Network {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet10 => "testnet10",
        }
    }
    pub fn port(self) -> u16 {
        match self {
            Self::Mainnet => 16110,
            Self::Testnet10 => 16210,
        }
    }
    fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            "testnet10" => Ok(Self::Testnet10),
            _ => Err(format!(
                "Unsupported smoke network: {value}; only mainnet and testnet10 are permitted"
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub repository: PathBuf,
    pub networks: Vec<Network>,
    pub ready_timeout: Duration,
    pub observation: Duration,
    pub skip_build: bool,
    pub plan_only: bool,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Options> {
    let mut args = args.into_iter();
    let mut root = None;
    let mut networks = None;
    let mut ready = None;
    let mut observation = None;
    let mut skip_build = false;
    let mut plan_only = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repository" if root.is_none() => {
                root = Some(PathBuf::from(
                    args.next().ok_or("--repository requires a path")?,
                ))
            }
            "--networks" if networks.is_none() => {
                let value = args
                    .next()
                    .ok_or("--networks requires a comma-separated list")?;
                let values = value
                    .split(',')
                    .map(|item| Network::parse(item.trim()))
                    .collect::<Result<Vec<_>>>()?;
                if values.is_empty() {
                    return Err("At least one network is required".to_owned());
                }
                networks = Some(values);
            }
            "--ready-timeout-seconds" if ready.is_none() => {
                let value = args
                    .next()
                    .ok_or("--ready-timeout-seconds requires an integer")?
                    .parse::<u64>()
                    .map_err(|_| "Invalid readiness timeout".to_owned())?;
                if !(30..=900).contains(&value) {
                    return Err("Readiness timeout must be in 30..=900 seconds".to_owned());
                }
                ready = Some(Duration::from_secs(value));
            }
            "--observation-seconds" if observation.is_none() => {
                let value = args
                    .next()
                    .ok_or("--observation-seconds requires an integer")?
                    .parse::<u64>()
                    .map_err(|_| "Invalid observation interval".to_owned())?;
                if !(15..=600).contains(&value) {
                    return Err("Observation interval must be in 15..=600 seconds".to_owned());
                }
                observation = Some(Duration::from_secs(value));
            }
            "--skip-build" if !skip_build => skip_build = true,
            "--plan-only" if !plan_only => plan_only = true,
            _ => return Err(format!("Unknown or repeated live-smoke option: {arg}")),
        }
    }
    Ok(Options {
        repository: root.unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("workspace parent")
                .to_owned()
        }),
        networks: networks.unwrap_or_else(|| vec![Network::Mainnet, Network::Testnet10]),
        ready_timeout: ready.unwrap_or(Duration::from_secs(240)),
        observation: observation.unwrap_or(Duration::from_secs(45)),
        skip_build,
        plan_only,
    })
}

fn normalized_network(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .flat_map(char::to_lowercase)
        .collect()
}

#[derive(Debug, Clone)]
struct Snapshot {
    server_version: String,
    is_synced: bool,
    peers: u32,
    daa: u64,
    blocks: u64,
    headers: u64,
}
impl Snapshot {
    fn parse(value: &Value, network: Network) -> Result<Self> {
        let actual = value
            .get("network")
            .and_then(Value::as_str)
            .ok_or("Probe result is missing network identity")?;
        if normalized_network(actual) != network.name() {
            return Err(format!(
                "network mismatch: expected={};actual={actual}",
                network.name()
            ));
        }
        let integer = |name: &str| {
            value
                .get(name)
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("Probe {name} must be an unsigned integer"))
        };
        let peers = integer("peerCount")?;
        if peers > i32::MAX as u64 {
            return Err("Probe peerCount exceeds the original signed32 contract".to_owned());
        }
        Ok(Self {
            server_version: value
                .get("serverVersion")
                .and_then(Value::as_str)
                .ok_or("Probe serverVersion must be a string")?
                .to_owned(),
            is_synced: value
                .get("isSynced")
                .and_then(Value::as_bool)
                .ok_or("Probe isSynced must be a boolean")?,
            peers: peers as u32,
            daa: integer("virtualDaaScore")?,
            blocks: integer("blockCount")?,
            headers: integer("headerCount")?,
        })
    }
}

#[derive(Clone)]
struct Profile {
    network: Network,
    port: u16,
    app_dir: PathBuf,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl Profile {
    fn endpoint(&self) -> String {
        format!("grpc://127.0.0.1:{}", self.port)
    }
}

fn round_tenth(value: f64) -> f64 {
    (value * 10.0).round_ties_even() / 10.0
}

fn successful_result(
    profile: &Profile,
    first: &Snapshot,
    second: &Snapshot,
    relaunch: &Snapshot,
    duration: f64,
) -> Value {
    let daa = second.daa > first.daa;
    let blocks = second.blocks > first.blocks;
    let headers = second.headers > first.headers;
    let progress = daa || blocks || headers;
    let mut result = json!({
        "Network":profile.network.name(),"Success":second.peers>0,"ProcessStayedAlive":true,"RpcReady":true,
        "RpcEndpoint":profile.endpoint(),"ServerVersion":second.server_version,"PeerCount":second.peers,"IsSynced":second.is_synced,
        "FirstVirtualDaaScore":first.daa,"SecondVirtualDaaScore":second.daa,
        "FirstBlockCount":first.blocks,"SecondBlockCount":second.blocks,
        "FirstHeaderCount":first.headers,"SecondHeaderCount":second.headers,
        "DaaProgressed":daa,"BlockProgressed":blocks,"HeaderProgressed":headers,"ChainProgressObserved":progress,
        "ParentLossCleanup":true,"RelaunchAfterParentLoss":true,"RelaunchPeerCount":relaunch.peers,
        "ObservationWarning":if progress{None}else{Some(NO_PROGRESS_WARNING)},
        "FullSyncRequiredForProduction":!second.is_synced,"DurationSeconds":round_tenth(duration),
        "StdoutLog":profile.stdout.to_string_lossy(),"StderrLog":profile.stderr.to_string_lossy(),
    });
    if second.peers == 0 {
        result["Failure"] = json!("RPC opened, but no connected peers were reported.");
    }
    result
}
fn failed_result(profile: &Profile, alive: bool, duration: f64, error: &str) -> Value {
    json!({"Network":profile.network.name(),"Success":false,"ProcessStayedAlive":alive,"RpcReady":false,
        "RpcEndpoint":profile.endpoint(),"Failure":error,"DurationSeconds":round_tenth(duration),
        "StdoutLog":profile.stdout.to_string_lossy(),"StderrLog":profile.stderr.to_string_lossy()})
}
fn report(networks: &[Network], preflight: Value, results: &[Value], generated: &str) -> Value {
    json!({"SchemaVersion":1,"GeneratedAt":generated,"TestKind":"short-live-network-smoke",
        "StableRuntime":"rusty-kaspa v2.1.0","StableRuntimeCommit":"01b532e8b553523216471682649693af92f0fd16",
        "TestedNetworks":networks.iter().map(|network|network.name()).collect::<Vec<_>>(),
        "ExperimentalTestnet13Started":false,"Preflight":preflight,"Results":results,
        "Passed":!results.is_empty() && results.len()==networks.len() && results.iter().zip(networks).all(|(result,network)|result.get("Success").and_then(Value::as_bool)==Some(true) && result.get("Network").and_then(Value::as_str)==Some(network.name()))})
}

pub fn run(options: Options) -> Result<()> {
    runner::run(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile() -> Profile {
        Profile {
            network: Network::Testnet10,
            port: 16210,
            app_dir: "data/testnet10".into(),
            stdout: "node.stdout.log".into(),
            stderr: "node.stderr.log".into(),
        }
    }
    fn snapshot() -> Snapshot {
        Snapshot {
            server_version: "2.1.0".to_owned(),
            is_synced: false,
            peers: 2,
            daa: 9007199254740993,
            blocks: 10,
            headers: 20,
        }
    }
    #[test]
    fn defaults_and_network_boundaries_match_the_existing_tool() {
        let options = parse(Vec::new()).unwrap();
        assert_eq!(options.networks, vec![Network::Mainnet, Network::Testnet10]);
        assert_eq!(options.ready_timeout, Duration::from_secs(240));
        assert_eq!(options.observation, Duration::from_secs(45));
        assert_eq!(Network::Mainnet.port(), 16110);
        assert_eq!(Network::Testnet10.port(), 16210);
        for name in ["testnet13", "devnet", "simnet", "mainnet;command", ""] {
            assert!(Network::parse(name).is_err());
        }
    }
    #[test]
    fn cli_rejects_invalid_ranges_and_arbitrary_executables() {
        for args in [
            vec!["--ready-timeout-seconds", "29"],
            vec!["--ready-timeout-seconds", "901"],
            vec!["--observation-seconds", "14"],
            vec!["--observation-seconds", "601"],
            vec!["--executable", "foreign"],
            vec!["--networks", "testnet13"],
            vec!["--networks", "mainnet,,testnet10"],
            vec!["--skip-build", "--skip-build"],
        ] {
            assert!(parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn no_counter_progress_is_informational_not_a_false_sync_claim() {
        let value = snapshot();
        let result = successful_result(&profile(), &value, &value, &value, 45.0);
        assert_eq!(result["Success"], true);
        assert_eq!(result["ChainProgressObserved"], false);
        assert_eq!(result["IsSynced"], false);
        assert_eq!(result["FullSyncRequiredForProduction"], true);
        assert_eq!(result["ObservationWarning"], NO_PROGRESS_WARNING);
    }
    #[test]
    fn peer_count_not_large_chain_counters_controls_health() {
        let first = snapshot();
        let mut second = first.clone();
        second.daa += 1;
        second.peers = 0;
        let result = successful_result(&profile(), &first, &second, &first, 45.05);
        assert_eq!(result["Success"], false);
        assert_eq!(result["DaaProgressed"], true);
        assert_eq!(
            result["FirstVirtualDaaScore"].as_u64(),
            Some(9007199254740993)
        );
        assert_eq!(
            result["SecondVirtualDaaScore"].as_u64(),
            Some(9007199254740994)
        );
        assert_eq!(
            result["Failure"],
            "RPC opened, but no connected peers were reported."
        );
    }
    #[test]
    fn probes_reject_missing_or_wrong_network_and_inexact_numeric_types() {
        let base = json!({"network":"testnet-10","serverVersion":"2.1.0","peerCount":1,"isSynced":false,"virtualDaaScore":1,"blockCount":2,"headerCount":3});
        assert!(Snapshot::parse(&base, Network::Testnet10).is_ok());
        assert!(Snapshot::parse(&base, Network::Mainnet).is_err());
        for (field, value) in [
            ("peerCount", json!(2147483648_u64)),
            ("virtualDaaScore", json!(1.5)),
            ("blockCount", Value::Null),
            ("isSynced", json!("false")),
        ] {
            let mut changed = base.clone();
            changed[field] = value;
            assert!(Snapshot::parse(&changed, Network::Testnet10).is_err());
        }
    }
    #[test]
    fn failed_and_incomplete_report_cannot_claim_success() {
        let value = failed_result(&profile(), true, 1.25, "fixture failure");
        assert_eq!(value["RpcReady"], false);
        assert_eq!(value["Success"], false);
        assert_eq!(
            report(&[Network::Testnet10], json!({}), &[value], "fixture-time")["Passed"],
            false
        );
        assert_eq!(report(&[], json!({}), &[], "fixture-time")["Passed"], false);
    }
    #[test]
    fn partially_completed_network_matrix_never_reports_passed() {
        let snapshot = snapshot();
        let first = successful_result(&profile(), &snapshot, &snapshot, &snapshot, 1.0);
        assert_eq!(
            report(
                &[Network::Mainnet, Network::Testnet10],
                json!({}),
                &[first],
                "fixture-time"
            )["Passed"],
            false
        );
    }
}
