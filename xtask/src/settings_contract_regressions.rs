use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize)]
struct Invocation {
    function: &'static str,
    args: Vec<Value>,
}

#[derive(Debug)]
enum Check {
    Exact(Value),
    Subset(Value),
    HasKeys(Vec<&'static str>),
    MissingKeys(Vec<&'static str>),
    Contains(Vec<&'static str>),
    NotContains(Vec<&'static str>),
}

#[derive(Debug)]
struct Assertion {
    case: &'static str,
    invocation: Invocation,
    check: Check,
}
fn patch(base: &Value, updates: &[(&str, Value)]) -> Value {
    let mut value = base.clone();
    let object = value
        .as_object_mut()
        .expect("settings regression fixture base must be an object");
    for (key, replacement) in updates {
        object.insert((*key).to_owned(), replacement.clone());
    }
    value
}

fn node_values() -> Value {
    json!({
        "rpcListenEnabled": true, "rpcListenHost": "127.0.0.1", "rpcListenPort": "16110",
        "rpcBorshEnabled": false, "rpcBorshHost": "invalid host", "rpcBorshPort": "",
        "rpcJsonEnabled": false, "listenEnabled": false, "externalIpEnabled": false,
        "connectEnabled": false, "addPeerEnabled": false, "noLogFiles": true,
        "perfMetrics": true, "asyncThreads": "16", "ramScale": "1",
        "rpcMaxClients": "16", "outPeers": "8", "maxInPeers": "32", "logLevel": "info"
    })
}

fn bridge_values() -> Value {
    json!({
        "nodeMode": "external", "kaspadAddress": "127.0.0.1:16110",
        "stratumPort": ":5555", "promPort": ":2112", "minShareDiff": "8192",
        "sharesPerMin": "30", "extranonceSize": "0", "blockWaitTime": "50ms",
        "inprocessRpcListen": "127.0.0.1:16110", "inprocessRamScale": "1",
        "inprocessAsyncThreads": "16", "inprocessOutpeers": "8", "inprocessMaxInpeers": "32"
    })
}
fn push(
    assertions: &mut Vec<Assertion>,
    case: &'static str,
    function: &'static str,
    args: Vec<Value>,
    check: Check,
) {
    assertions.push(Assertion {
        case,
        invocation: Invocation { function, args },
        check,
    });
}

fn subset(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(actual), Value::Object(expected)) => expected.iter().all(|(key, value)| {
            actual
                .get(key)
                .is_some_and(|candidate| subset(candidate, value))
        }),
        _ => actual == expected,
    }
}

fn verify(check: &Check, actual: &Value) -> Result<(), String> {
    match check {
        Check::Exact(expected) if actual != expected => {
            Err(format!("expected exact {expected}, got {actual}"))
        }
        Check::Subset(expected) if !subset(actual, expected) => {
            Err(format!("expected subset {expected}, got {actual}"))
        }
        Check::HasKeys(keys) => {
            let object = actual
                .as_object()
                .ok_or_else(|| format!("expected object, got {actual}"))?;
            let missing = keys
                .iter()
                .filter(|key| !object.contains_key(**key))
                .collect::<Vec<_>>();
            if missing.is_empty() {
                Ok(())
            } else {
                Err(format!("missing keys {missing:?} in {actual}"))
            }
        }
        Check::MissingKeys(keys) => {
            let object = actual
                .as_object()
                .ok_or_else(|| format!("expected object, got {actual}"))?;
            let present = keys
                .iter()
                .filter(|key| object.contains_key(**key))
                .collect::<Vec<_>>();
            if present.is_empty() {
                Ok(())
            } else {
                Err(format!("unexpected keys {present:?} in {actual}"))
            }
        }
        Check::Contains(parts) => {
            let text = actual
                .as_str()
                .ok_or_else(|| format!("expected string, got {actual}"))?;
            let missing = parts
                .iter()
                .filter(|part| !text.contains(**part))
                .collect::<Vec<_>>();
            if missing.is_empty() {
                Ok(())
            } else {
                Err(format!("missing text {missing:?} in {text:?}"))
            }
        }
        Check::NotContains(parts) => {
            let text = actual
                .as_str()
                .ok_or_else(|| format!("expected string, got {actual}"))?;
            let present = parts
                .iter()
                .filter(|part| text.contains(**part))
                .collect::<Vec<_>>();
            if present.is_empty() {
                Ok(())
            } else {
                Err(format!("unexpected text {present:?} in {text:?}"))
            }
        }
        _ => Ok(()),
    }
}

fn build_assertions() -> Vec<Assertion> {
    let mut a = Vec::new();
    let node = node_values();
    let bridge = bridge_values();

    push(
        &mut a,
        "default form uses valid KGW settings; blank optional values are inactive",
        "validateNodeForm",
        vec![node.clone(), json!({}), json!("mainnet")],
        Check::Exact(json!({})),
    );

    let endpoint_rows = [
        ("rpcBorshEnabled", "rpcBorshHost", "rpcBorshPort"),
        ("rpcJsonEnabled", "rpcJsonHost", "rpcJsonPort"),
        ("listenEnabled", "listenHost", "listenPort"),
        ("externalIpEnabled", "externalIpHost", "externalIpPort"),
        ("connectEnabled", "connectHost", "connectPort"),
        ("addPeerEnabled", "addPeerHost", "addPeerPort"),
    ];
    for (parent, host, port) in endpoint_rows {
        let case = match parent {
            "rpcBorshEnabled" => "rpcBorshEnabled controls both children and validation",
            "rpcJsonEnabled" => "rpcJsonEnabled controls both children and validation",
            "listenEnabled" => "listenEnabled controls both children and validation",
            "externalIpEnabled" => "externalIpEnabled controls both children and validation",
            "connectEnabled" => "connectEnabled controls both children and validation",
            _ => "addPeerEnabled controls both children and validation",
        };
        let disabled = patch(&node, &[(host, json!("")), (port, json!(""))]);
        push(
            &mut a,
            case,
            "nodeFieldEnabled",
            vec![json!(host), disabled.clone(), json!({})],
            Check::Exact(json!(false)),
        );
        push(
            &mut a,
            case,
            "nodeFieldEnabled",
            vec![json!(port), disabled.clone(), json!({})],
            Check::Exact(json!(false)),
        );
        push(
            &mut a,
            case,
            "validateNodeForm",
            vec![disabled.clone(), json!({}), json!("mainnet")],
            Check::Exact(json!({})),
        );
        let enabled = patch(&disabled, &[(parent, json!(true))]);
        push(
            &mut a,
            case,
            "nodeFieldEnabled",
            vec![json!(host), enabled.clone(), json!({})],
            Check::Exact(json!(true)),
        );
        push(
            &mut a,
            case,
            "validateNodeForm",
            vec![enabled.clone(), json!({}), json!("mainnet")],
            Check::HasKeys(vec![host, port]),
        );
        let valid = patch(
            &enabled,
            &[(host, json!("127.0.0.1")), (port, json!("19100"))],
        );
        push(
            &mut a,
            case,
            "validateNodeForm",
            vec![valid, json!({}), json!("mainnet")],
            Check::Exact(json!({})),
        );
    }
    for name in [
        "appDir",
        "configFile",
        "overrideParamsFile",
        "testnet",
        "netsuffix",
        "noGrpc",
        "rpcListenEnabled",
    ] {
        let case = match name {
            "appDir" => "appDir stays managed even when an inclusion option is true",
            "configFile" => "configFile stays managed even when an inclusion option is true",
            "overrideParamsFile" => {
                "overrideParamsFile stays managed even when an inclusion option is true"
            }
            "testnet" => "testnet stays managed even when an inclusion option is true",
            "netsuffix" => "netsuffix stays managed even when an inclusion option is true",
            "noGrpc" => "noGrpc stays managed even when an inclusion option is true",
            _ => "rpcListenEnabled stays managed even when an inclusion option is true",
        };
        let mut options = Map::new();
        options.insert(name.to_owned(), Value::Bool(true));
        push(
            &mut a,
            case,
            "nodeFieldEnabled",
            vec![json!(name), node.clone(), Value::Object(options)],
            Check::Exact(json!(false)),
        );
    }

    push(
        &mut a,
        "explicit optional empty value must be corrected before Save/Start",
        "validateNodeForm",
        vec![node.clone(), json!({"uaComment": true}), json!("mainnet")],
        Check::HasKeys(vec!["uaComment"]),
    );
    push(
        &mut a,
        "explicit optional empty value must be corrected before Save/Start",
        "validateNodeForm",
        vec![
            patch(&node, &[("uaComment", json!("desktop"))]),
            json!({"uaComment": true}),
            json!("mainnet"),
        ],
        Check::MissingKeys(vec!["uaComment"]),
    );
    let disabled_drafts = patch(
        &node,
        &[
            ("logDir", json!("")),
            ("perfMetrics", json!(false)),
            ("perfMetricsInterval", json!("")),
            ("rocksDbPreset", json!("default")),
        ],
    );
    for (field, options) in [
        ("logDir", json!({"logDir": true})),
        ("perfMetricsInterval", json!({"perfMetricsInterval": true})),
        ("rocksDbCacheSize", json!({"rocksDbCacheSize": true})),
    ] {
        push(
            &mut a,
            "disabled parent suppresses a stored logging/performance/cache draft",
            "nodeFieldEnabled",
            vec![json!(field), disabled_drafts.clone(), options],
            Check::Exact(json!(false)),
        );
    }

    for (key, bad, case) in [
        ("asyncThreads", "0", "asyncThreads rejects 0"),
        ("ramScale", "NaN", "ramScale rejects NaN"),
        ("ramScale", "11", "ramScale rejects 11"),
        ("rpcMaxClients", "17", "rpcMaxClients rejects 17"),
        ("outPeers", "9", "outPeers rejects 9"),
        ("maxInPeers", "33", "maxInPeers rejects 33"),
        ("asyncThreads", "1.2", "asyncThreads rejects 1.2"),
    ] {
        push(
            &mut a,
            case,
            "validateNodeForm",
            vec![
                patch(&node, &[(key, json!(bad))]),
                json!({}),
                json!("mainnet"),
            ],
            Check::HasKeys(vec![key]),
        );
    }
    let host_case = "IP and hostname validation covers IPv4, IPv6 and invalid input";
    for host in [
        "127.0.0.1",
        "::1",
        "[2001:db8::1]",
        "node.example",
        "localhost",
    ] {
        push(
            &mut a,
            host_case,
            "isHost",
            vec![json!(host)],
            Check::Exact(json!(true)),
        );
    }
    for host in ["300.1.2.3", "127.0.0", "-bad", "bad host", "a/b", "::::"] {
        push(
            &mut a,
            host_case,
            "isHost",
            vec![json!(host)],
            Check::Exact(json!(false)),
        );
    }
    push(
        &mut a,
        host_case,
        "endpoint",
        vec![json!("::1"), json!(16110)],
        Check::Exact(json!("[::1]:16110")),
    );
    push(
        &mut a,
        host_case,
        "isPort",
        vec![json!("65535")],
        Check::Exact(json!(true)),
    );
    push(
        &mut a,
        host_case,
        "isPort",
        vec![json!("65536")],
        Check::Exact(json!(false)),
    );
    push(
        &mut a,
        host_case,
        "isPort",
        vec![json!("0")],
        Check::Exact(json!(false)),
    );

    let overlap = "wildcard/localhost aliases conflict but different exact interfaces do not";
    push(
        &mut a,
        overlap,
        "listenersOverlap",
        vec![
            json!({"host":"0.0.0.0","port":"16110"}),
            json!({"host":"127.0.0.1","port":"16110"}),
        ],
        Check::Exact(json!(true)),
    );
    push(
        &mut a,
        overlap,
        "listenersOverlap",
        vec![
            json!({"host":"localhost","port":16110}),
            json!({"host":"127.0.0.1","port":16110}),
        ],
        Check::Exact(json!(true)),
    );
    push(
        &mut a,
        overlap,
        "listenersOverlap",
        vec![
            json!({"host":"192.168.1.1","port":16110}),
            json!({"host":"192.168.1.2","port":16110}),
        ],
        Check::Exact(json!(false)),
    );
    push(
        &mut a,
        overlap,
        "validateNodeForm",
        vec![
            patch(
                &node,
                &[
                    ("listenEnabled", json!(true)),
                    ("listenHost", json!("0.0.0.0")),
                    ("listenPort", json!("16110")),
                ],
            ),
            json!({}),
            json!("mainnet"),
        ],
        Check::HasKeys(vec!["listenPort"]),
    );

    let rpc_case = "RPC stays loopback unless unsafe RPC is selected";
    let unsafe_host = patch(&node, &[("rpcListenHost", json!("0.0.0.0"))]);
    push(
        &mut a,
        rpc_case,
        "validateNodeForm",
        vec![unsafe_host.clone(), json!({}), json!("mainnet")],
        Check::HasKeys(vec!["rpcListenHost"]),
    );
    push(
        &mut a,
        rpc_case,
        "validateNodeForm",
        vec![
            patch(&unsafe_host, &[("unsafeRpc", json!(true))]),
            json!({}),
            json!("mainnet"),
        ],
        Check::MissingKeys(vec!["rpcListenHost"]),
    );

    let network_case = "network profiles are independent and unsynced mining is test-only";
    let tn = patch(
        &node,
        &[
            ("rpcListenPort", json!("16210")),
            ("enableUnsyncedMining", json!(true)),
        ],
    );
    push(
        &mut a,
        network_case,
        "validateNodeForm",
        vec![tn.clone(), json!({}), json!("testnet10")],
        Check::Exact(json!({})),
    );
    push(
        &mut a,
        network_case,
        "validateNodeForm",
        vec![tn, json!({}), json!("mainnet")],
        Check::HasKeys(vec!["enableUnsyncedMining"]),
    );

    let runtime = "enabled profile never implies a running process or synchronized network";
    push(
        &mut a,
        runtime,
        "runtimePresentation",
        vec![json!({"enabled":true,"running":false})],
        Check::Subset(json!({"profile":"Enabled","process":"Stopped","network":"Not connected"})),
    );
    push(
        &mut a,
        runtime,
        "runtimePresentation",
        vec![json!({"running":true})],
        Check::Subset(json!({"network":"Synchronization not reported"})),
    );
    push(
        &mut a,
        runtime,
        "runtimePresentation",
        vec![json!({"running":true,"synced":false})],
        Check::Subset(json!({"network":"Not synchronized"})),
    );
    push(
        &mut a,
        runtime,
        "runtimePresentation",
        vec![json!({"error":"exit 1"})],
        Check::Subset(json!({"process":"Failed"})),
    );

    push(
        &mut a,
        "Bridge defaults validate without starting an owner",
        "validateBridgeForm",
        vec![bridge.clone(), json!({}), json!("mainnet")],
        Check::Exact(json!({})),
    );

    let external = patch(
        &bridge,
        &[
            ("inprocessRpcListen", json!("bad")),
            ("inprocessRamScale", json!("bad")),
        ],
    );
    push(
        &mut a,
        "Bridge external mode excludes all stored in-process values",
        "bridgeFieldEnabled",
        vec![json!("inprocessRpcListen"), external.clone(), json!({})],
        Check::Exact(json!(false)),
    );
    push(
        &mut a,
        "Bridge external mode excludes all stored in-process values",
        "validateBridgeForm",
        vec![external, json!({}), json!("mainnet")],
        Check::Exact(json!({})),
    );

    let inprocess_case = "Bridge in-process mode validates its node endpoint and CPU count";
    let inprocess = patch(&bridge, &[("nodeMode", json!("inprocess"))]);
    push(
        &mut a,
        inprocess_case,
        "validateBridgeForm",
        vec![inprocess.clone(), json!({}), json!("mainnet")],
        Check::Exact(json!({})),
    );
    push(
        &mut a,
        inprocess_case,
        "validateBridgeForm",
        vec![
            patch(
                &inprocess,
                &[
                    ("inprocessRpcListen", json!("")),
                    ("inprocessAsyncThreads", json!("0")),
                ],
            ),
            json!({}),
            json!("mainnet"),
        ],
        Check::HasKeys(vec!["inprocessRpcListen", "inprocessAsyncThreads"]),
    );
    for (field, case) in [
        (
            "kaspadAddress",
            "Bridge required kaspadAddress rejects an empty value",
        ),
        (
            "stratumPort",
            "Bridge required stratumPort rejects an empty value",
        ),
        (
            "minShareDiff",
            "Bridge required minShareDiff rejects an empty value",
        ),
        (
            "blockWaitTime",
            "Bridge required blockWaitTime rejects an empty value",
        ),
    ] {
        push(
            &mut a,
            case,
            "validateBridgeForm",
            vec![
                patch(&bridge, &[(field, json!(""))]),
                json!({}),
                json!("mainnet"),
            ],
            Check::HasKeys(vec![field]),
        );
    }

    let optional = "Bridge optional arguments require values only when included";
    push(
        &mut a,
        optional,
        "validateBridgeForm",
        vec![
            bridge.clone(),
            json!({"coinbaseTagSuffix":true}),
            json!("mainnet"),
        ],
        Check::HasKeys(vec!["coinbaseTagSuffix"]),
    );
    push(
        &mut a,
        optional,
        "validateBridgeForm",
        vec![
            bridge.clone(),
            json!({"coinbaseTagSuffix":false}),
            json!("mainnet"),
        ],
        Check::Exact(json!({})),
    );

    let config_case =
        "Bridge config mode suppresses conflicting inline values but validates its path";
    let config = patch(
        &bridge,
        &[
            ("config", json!("C:\\KGW\\bridge.yaml")),
            ("kaspadAddress", json!("")),
        ],
    );
    push(
        &mut a,
        config_case,
        "bridgeFieldEnabled",
        vec![
            json!("kaspadAddress"),
            config.clone(),
            json!({"config":true}),
        ],
        Check::Exact(json!(false)),
    );
    push(
        &mut a,
        config_case,
        "validateBridgeForm",
        vec![config.clone(), json!({"config":true}), json!("mainnet")],
        Check::Exact(json!({})),
    );
    push(
        &mut a,
        config_case,
        "validateBridgeForm",
        vec![
            patch(&config, &[("config", json!("relative.yaml"))]),
            json!({"config":true}),
            json!("mainnet"),
        ],
        Check::HasKeys(vec!["config"]),
    );
    for name in [
        "testnet",
        "appdir",
        "inprocessAppdirMirror",
        "inprocessNetworkArgs",
        "inprocessConfigfile",
        "inprocessOverrideParamsFile",
        "inprocessDevnet",
        "inprocessSimnet",
        "healthCheckPort",
        "webDashboardPort",
        "logToFile",
        "approxGeoLookup",
    ] {
        let mut options = Map::new();
        options.insert(name.to_owned(), Value::Bool(true));
        push(
            &mut a,
            "Bridge managed values never become editable effective overrides",
            "bridgeFieldEnabled",
            vec![json!(name), bridge.clone(), Value::Object(options)],
            Check::Exact(json!(false)),
        );
    }

    let peers = patch(
        &bridge,
        &[
            ("nodeMode", json!("inprocess")),
            ("inprocessRpcListen", json!("0.0.0.0:16110")),
            ("inprocessConnect", json!("127.0.0.1:16111")),
            ("inprocessAddPeer", json!("127.0.0.1:16112")),
        ],
    );
    push(
        &mut a,
        "Bridge in-process RPC remains loopback and peer modes remain exclusive",
        "validateBridgeForm",
        vec![
            peers,
            json!({"inprocessConnect":true,"inprocessAddPeer":true}),
            json!("mainnet"),
        ],
        Check::HasKeys(vec!["inprocessRpcListen", "inprocessAddPeer"]),
    );

    let disabled_children = patch(
        &bridge,
        &[
            ("nodeMode", json!("inprocess")),
            ("inprocessPerfMetrics", json!(false)),
            ("inprocessPerfMetricsIntervalSec", json!("")),
            ("internalCpuMiner", json!(false)),
            ("internalCpuMinerThreads", json!("bad")),
        ],
    );
    push(
        &mut a,
        "Bridge disabled metrics and miner parents suppress invalid child drafts",
        "validateBridgeForm",
        vec![
            disabled_children,
            json!({"inprocessPerfMetricsIntervalSec":true}),
            json!("testnet10"),
        ],
        Check::Exact(json!({})),
    );

    for (network, asic_case, miner_case, pacing_case) in [
        (
            "testnet10",
            "testnet10 excludes stale ASIC settings from restored CPU-only profiles",
            "testnet10 requires enabled CPU miner address and bounded threads",
            "testnet10 accepts blank optional CPU pacing but rejects invalid active values",
        ),
        (
            "testnet13",
            "testnet13 excludes stale ASIC settings from restored CPU-only profiles",
            "testnet13 requires enabled CPU miner address and bounded threads",
            "testnet13 accepts blank optional CPU pacing but rejects invalid active values",
        ),
    ] {
        let cpu_only = patch(
            &bridge,
            &[
                ("network", json!(network)),
                ("stratumPort", json!("bad")),
                ("promPort", json!("bad")),
                ("minShareDiff", json!("bad")),
                ("internalCpuMiner", json!(false)),
            ],
        );
        for field in ["stratumPort", "promPort", "minShareDiff", "config"] {
            let mut options = Map::new();
            options.insert(field.to_owned(), Value::Bool(true));
            push(
                &mut a,
                asic_case,
                "bridgeFieldEnabled",
                vec![json!(field), cpu_only.clone(), Value::Object(options)],
                Check::Exact(json!(false)),
            );
        }
        push(
            &mut a,
            asic_case,
            "validateBridgeForm",
            vec![cpu_only, json!({}), json!(network)],
            Check::Exact(json!({})),
        );

        let miner = patch(
            &bridge,
            &[
                ("internalCpuMiner", json!(true)),
                ("internalCpuMinerAddress", json!("")),
                ("internalCpuMinerThreads", json!("0")),
            ],
        );
        push(
            &mut a,
            miner_case,
            "validateBridgeForm",
            vec![miner.clone(), json!({}), json!(network)],
            Check::HasKeys(vec!["internalCpuMinerAddress", "internalCpuMinerThreads"]),
        );
        let valid_miner = patch(
            &miner,
            &[
                (
                    "internalCpuMinerAddress",
                    json!("kaspatest:checked-by-backend"),
                ),
                ("internalCpuMinerThreads", json!("1")),
            ],
        );
        push(
            &mut a,
            miner_case,
            "validateBridgeForm",
            vec![valid_miner.clone(), json!({}), json!(network)],
            Check::Exact(json!({})),
        );
        push(
            &mut a,
            miner_case,
            "validateBridgeForm",
            vec![
                patch(&valid_miner, &[("internalCpuMinerThreads", json!("257"))]),
                json!({}),
                json!(network),
            ],
            Check::HasKeys(vec!["internalCpuMinerThreads"]),
        );

        let pacing = patch(
            &valid_miner,
            &[
                ("internalCpuMinerThrottleMs", json!("")),
                ("internalCpuMinerTemplatePollMs", json!("")),
            ],
        );
        push(
            &mut a,
            pacing_case,
            "validateBridgeForm",
            vec![pacing.clone(), json!({}), json!(network)],
            Check::Exact(json!({})),
        );
        push(
            &mut a,
            pacing_case,
            "validateBridgeForm",
            vec![
                patch(&pacing, &[("internalCpuMinerTemplatePollMs", json!("0"))]),
                json!({}),
                json!(network),
            ],
            Check::HasKeys(vec!["internalCpuMinerTemplatePollMs"]),
        );
        push(
            &mut a,
            pacing_case,
            "validateBridgeForm",
            vec![
                patch(&pacing, &[("internalCpuMinerThrottleMs", json!("60001"))]),
                json!({}),
                json!(network),
            ],
            Check::HasKeys(vec!["internalCpuMinerThrottleMs"]),
        );
    }
    push(
        &mut a,
        "startup readiness never becomes live synchronization or mining evidence",
        "runtimeObservationSummary",
        vec![json!({"readiness":"READY"}), json!(true), json!(true)],
        Check::Contains(vec![
            "RPC: Unknown",
            "Sync: Not reported",
            "CPU: Not reported",
        ]),
    );

    let observed = json!({
        "observation_state":"fresh", "rpc_ready":"true", "synced":"false",
        "cpu_enabled":"true", "cpu_hashes_tried":"9007199254740993",
        "cpu_hashrate_hs":"12.5", "cpu_blocks_submitted":"7", "cpu_blocks_confirmed_blue":"2"
    });
    push(
        &mut a,
        "observed hashing and confirmed-blue counts retain their actual meaning",
        "runtimeObservationSummary",
        vec![observed.clone(), json!(true), json!(true)],
        Check::Contains(vec![
            "Sync: Not synchronized",
            "CPU: Hashing",
            "Hashes: 9007199254740993",
            "Submitted blocks: 7",
            "Confirmed blue blocks: 2",
            "12.50 H/s",
        ]),
    );
    push(
        &mut a,
        "observed hashing and confirmed-blue counts retain their actual meaning",
        "runtimeObservationSummary",
        vec![observed, json!(false), json!(true)],
        Check::NotContains(vec!["Hashing", "9007199254740993", "12.50"]),
    );

    push(
        &mut a,
        "RPC errors and unavailable observations cannot retain positive live states",
        "runtimeObservationSummary",
        vec![
            json!({"observation_state":"fresh","rpc_ready":"false","synced":"unknown","observation_error":"connection refused"}),
            json!(true),
            json!(false),
        ],
        Check::Contains(vec![
            "RPC: Unavailable",
            "Sync: Not reported",
            "connection refused",
        ]),
    );
    push(
        &mut a,
        "RPC errors and unavailable observations cannot retain positive live states",
        "runtimeObservationSummary",
        vec![
            json!({"observation_state":"unavailable","synced":"true","cpu_enabled":"true","cpu_hashrate_hs":"9"}),
            json!(true),
            json!(true),
        ],
        Check::Contains(vec![
            "RPC: Unknown",
            "Sync: Not reported",
            "CPU: Not reported",
        ]),
    );

    a
}
const NODE_BRIDGE: &str = r#"
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
const [gluePath, wasmPath, settingsPath, requestPath, resultPath] = process.argv.slice(2);
const glue = await import(pathToFileURL(gluePath).href);
glue.initSync({ module: await readFile(wasmPath) });
const settings = await import(pathToFileURL(settingsPath).href);
const calls = JSON.parse(await readFile(requestPath, "utf8"));
const results = [];
for (const call of calls) {
  const fn = settings[call.function];
  if (typeof fn !== "function") throw new Error("Missing settings export: " + call.function);
  const value = await fn(...call.args);
  results.push(value === undefined ? { __kgw_undefined__: true } : value);
}
await writeFile(resultPath, JSON.stringify(results), "utf8");
"#;

pub fn run(root: &Path) -> Result<String, String> {
    let assertions = build_assertions();
    let logical_cases = assertions
        .iter()
        .map(|assertion| assertion.case)
        .collect::<BTreeSet<_>>();
    if logical_cases.len() != 49 {
        return Err(format!(
            "settings regression definition drift: expected 49 logical cases, got {}",
            logical_cases.len()
        ));
    }

    let temp = tempfile::tempdir()
        .map_err(|error| format!("failed to create settings regression tempdir: {error}"))?;
    let bridge_path = temp.path().join("settings-contract-driver.mjs");
    let request_path = temp.path().join("requests.json");
    let result_path = temp.path().join("results.json");
    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write generated Node bridge: {error}"))?;
    let invocations = assertions
        .iter()
        .map(|assertion| &assertion.invocation)
        .collect::<Vec<_>>();
    fs::write(
        &request_path,
        serde_json::to_vec(&invocations)
            .map_err(|error| format!("failed to serialize settings requests: {error}"))?,
    )
    .map_err(|error| format!("failed to write settings requests: {error}"))?;

    let glue = root.join(
        "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm.js",
    );
    let wasm = root.join(
        "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm_bg.wasm",
    );
    let settings = root.join("apps/kaspa-gateway-desktop/frontend/src/settings-contract.js");
    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&glue)
        .arg(&wasm)
        .arg(&settings)
        .arg(&request_path)
        .arg(&result_path)
        .current_dir(root)
        .output()
        .map_err(|error| {
            format!("failed to launch Node for settings contract regressions: {error}")
        })?;
    if !output.status.success() {
        return Err(format!(
            "settings contract Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let results: Vec<Value> = serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read settings results: {error}"))?,
    )
    .map_err(|error| format!("failed to parse settings results: {error}"))?;
    if results.len() != assertions.len() {
        return Err(format!(
            "settings result count mismatch: expected {}, got {}",
            assertions.len(),
            results.len()
        ));
    }

    for (index, (assertion, actual)) in assertions.iter().zip(results.iter()).enumerate() {
        verify(&assertion.check, actual).map_err(|error| {
            format!(
                "settings contract regression failed: case={:?} assertion={} function={} error={error}",
                assertion.case,
                index + 1,
                assertion.invocation.function
            )
        })?;
    }

    Ok(format!(
        "KGW settings contract Rust-owned regressions PASSED: logical_cases={} assertions={}",
        logical_cases.len(),
        assertions.len()
    ))
}
