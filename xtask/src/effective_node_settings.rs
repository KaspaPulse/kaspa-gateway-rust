use std::fs;
use std::path::Path;

const NODE_PATH: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const BRIDGE_PATH: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const IPC_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs";
const WORKER_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
const OWNER_PATH: &str = "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs";
const SCHEMA_PATH: &str = "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs";

const SCHEMA_FIELDS: &[&str] = &[
    "logLevel",
    "asyncThreads",
    "ramScale",
    "yes",
    "noLogFiles",
    "sanity",
    "enableUnsyncedMining",
    "p2pListen",
    "externalIp",
    "disableUpnp",
    "disableDnsSeeding",
    "userAgentComments",
    "rpcListen",
    "rpcListenBorsh",
    "rpcListenJson",
    "rpcMaxClients",
    "unsafeRpc",
    "disableGrpc",
    "connectPeers",
    "addPeers",
    "outboundTarget",
    "inboundLimit",
    "utxoIndex",
    "archival",
    "resetDb",
    "perfMetrics",
    "maxTrackedAddresses",
    "retentionPeriodDays",
    "perfMetricsIntervalSec",
    "rocksDbPreset",
    "rocksDbCacheSize",
    "rocksDbWalDir",
    "overrideParamsFile",
    "logDir",
];

const OWNER_FIELDS: &[&str] = &[
    "log_level",
    "async_threads",
    "ram_scale",
    "connect_peers",
    "add_peers",
    "rpclisten_borsh",
    "rpclisten_json",
    "rpc_max_clients",
    "outbound_target",
    "inbound_limit",
    "reset_db",
    "perf_metrics",
    "retention_period_days",
    "rocksdb_preset",
    "rocksdb_cache_size",
    "rocksdb_wal_dir",
    "override_params_file",
];
struct Sources {
    node: String,
    bridge: String,
    ipc: String,
    worker: String,
    owner: String,
    schema: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let sources = Sources {
        node: read(root, NODE_PATH)?,
        bridge: read(root, BRIDGE_PATH)?,
        ipc: read(root, IPC_PATH)?,
        worker: read(root, WORKER_PATH)?,
        owner: read(root, OWNER_PATH)?,
        schema: normalize_newlines(&read(root, SCHEMA_PATH)?),
    };
    validate(&sources)?;
    Ok("KGW effective Node settings gate PASSED".to_owned())
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative)).map_err(|error| {
        format!("effective Node settings gate: failed to read {relative}: {error}")
    })
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn require(source: &str, needle: &str, label: &str) -> Result<(), String> {
    if source.contains(needle) {
        Ok(())
    } else {
        Err(format!("{label} missing required contract: {needle}"))
    }
}

fn validate(s: &Sources) -> Result<(), String> {
    require(&s.node, "function kgwNodeEffectiveNodeSettings", "Node")?;
    require(
        &s.bridge,
        "function kgwBridgeEffectiveInprocessNodeSettings",
        "Bridge",
    )?;
    for field in SCHEMA_FIELDS {
        require(&s.node, field, "Node typed payload")?;
        require(&s.bridge, field, "Bridge in-process typed payload")?;
    }

    for (source, needle, label) in [
        (
            &s.node,
            "effectiveNodeSettings: kgwNodeEffectiveNodeSettings(net)",
            "Node",
        ),
        (
            &s.bridge,
            "effectiveNodeSettings: kgwBridgeEffectiveInprocessNodeSettings(net)",
            "Bridge",
        ),
        (
            &s.node,
            "Restart required to apply changed effective settings",
            "Node",
        ),
        (
            &s.bridge,
            "Restart required to apply changed effective settings",
            "Bridge",
        ),
        (&s.node, "--configfile is not supported", "Node"),
        (&s.bridge, "--configfile is unsupported", "Bridge"),
        (&s.node, "--override-params-file is not supported", "Node"),
        (
            &s.bridge,
            "In-process --override-params-file is unsupported",
            "Bridge",
        ),
        (
            &s.node,
            "--logdir and --nologfiles cannot be used together",
            "Node",
        ),
        (
            &s.node,
            r#"net.key === "testnet10" ? "16211" : "16711""#,
            "Node",
        ),
        (
            &s.node,
            r#"cardCheck(net.key, "disableUpnp", "--disable-upnp", true)"#,
            "Node",
        ),
        (
            &s.node,
            r#"cardCheck(net.key, "rpcBorshEnabled", "--rpclisten-borsh", false)"#,
            "Node",
        ),
        (
            &s.node,
            r#"cardCheck(net.key, "rpcJsonEnabled", "--rpclisten-json", false)"#,
            "Node",
        ),
        (
            &s.bridge,
            r#"id(net.key, "inprocessDisableUpnp")}" type="checkbox" checked"#,
            "Bridge",
        ),
    ] {
        require(source, needle, label)?;
    }

    for (rust_field, frontend_field, legacy_field) in [
        ("rocksdb_preset", "rocksDbPreset", "rocksdbPreset"),
        ("rocksdb_cache_size", "rocksDbCacheSize", "rocksdbCacheSize"),
        ("rocksdb_wal_dir", "rocksDbWalDir", "rocksdbWalDir"),
    ] {
        let contract = format!(
            "#[serde(rename = \"{frontend_field}\", alias = \"{legacy_field}\")]\n    pub {rust_field}"
        );
        require(&s.schema, &contract, "EffectiveNodeSettings serde contract")?;
    }
    for needle in [
        "Option<kaspa_gateway_rk_node::EffectiveNodeSettings>",
        "--effective-node-settings-path",
        "kgw_worker_atomic_write_json_v1(&effective_node_settings_path",
    ] {
        require(&s.ipc, needle, "IPC")?;
    }

    for needle in [
        "serde_json::from_slice::<kaspa_gateway_rk_node::EffectiveNodeSettings>",
        "apply_effective_node_settings(effective_node_settings)",
    ] {
        require(&s.worker, needle, "Worker")?;
    }

    require(&s.owner, "fn build_mainline_args", "Owner")?;
    require(&s.owner, "fn build_tn13_args", "Owner")?;
    for field in OWNER_FIELDS {
        require(&s.owner, field, "Pinned Args mapping")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Sources {
        let common = SCHEMA_FIELDS.join(" ");
        let node = format!(
            "function kgwNodeEffectiveNodeSettings {common}\n\
effectiveNodeSettings: kgwNodeEffectiveNodeSettings(net)\n\
Restart required to apply changed effective settings\n\
--configfile is not supported\n--override-params-file is not supported\n\
--logdir and --nologfiles cannot be used together\n\
net.key === \"testnet10\" ? \"16211\" : \"16711\"\n\
cardCheck(net.key, \"disableUpnp\", \"--disable-upnp\", true)\n\
cardCheck(net.key, \"rpcBorshEnabled\", \"--rpclisten-borsh\", false)\n\
cardCheck(net.key, \"rpcJsonEnabled\", \"--rpclisten-json\", false)"
        );
        let bridge = format!(
            "function kgwBridgeEffectiveInprocessNodeSettings {common}\n\
effectiveNodeSettings: kgwBridgeEffectiveInprocessNodeSettings(net)\n\
Restart required to apply changed effective settings\n\
--configfile is unsupported\nIn-process --override-params-file is unsupported\n\
id(net.key, \"inprocessDisableUpnp\")}}\" type=\"checkbox\" checked"
        );
        let schema = [
            ("rocksdb_preset", "rocksDbPreset", "rocksdbPreset"),
            ("rocksdb_cache_size", "rocksDbCacheSize", "rocksdbCacheSize"),
            ("rocksdb_wal_dir", "rocksDbWalDir", "rocksdbWalDir"),
        ]
        .into_iter()
        .map(|(rust_field, frontend, legacy)| {
            format!("#[serde(rename = \"{frontend}\", alias = \"{legacy}\")]\n    pub {rust_field}")
        })
        .collect::<Vec<_>>()
        .join("\n");
        Sources {
            node,
            bridge,
            ipc: "Option<kaspa_gateway_rk_node::EffectiveNodeSettings> --effective-node-settings-path kgw_worker_atomic_write_json_v1(&effective_node_settings_path".to_owned(),
            worker: "serde_json::from_slice::<kaspa_gateway_rk_node::EffectiveNodeSettings> apply_effective_node_settings(effective_node_settings)".to_owned(),
            owner: format!("fn build_mainline_args fn build_tn13_args {}", OWNER_FIELDS.join(" ")),
            schema,
        }
    }

    #[test]
    fn complete_contract_passes() {
        assert!(validate(&fixture()).is_ok());
    }

    #[test]
    fn every_schema_field_is_required_in_node_and_bridge() {
        for field in SCHEMA_FIELDS {
            let mut s = fixture();
            s.node = s.node.replace(field, "");
            assert!(validate(&s).is_err(), "node {field}");
            let mut s = fixture();
            s.bridge = s.bridge.replace(field, "");
            assert!(validate(&s).is_err(), "bridge {field}");
        }
    }
    #[test]
    fn serde_alias_contract_is_exact_and_newline_normalized() {
        let mut s = fixture();
        s.schema = s.schema.replace("rocksdbPreset", "legacyWrong");
        assert!(validate(&s).is_err());

        let normalized = normalize_newlines(
            "#[serde(rename = \"rocksDbPreset\", alias = \"rocksdbPreset\")]\r\n    pub rocksdb_preset",
        );
        assert!(normalized.contains(
            "#[serde(rename = \"rocksDbPreset\", alias = \"rocksdbPreset\")]\n    pub rocksdb_preset"
        ));
    }

    #[test]
    fn required_markers_fail_closed_by_surface() {
        let cases = [
            (
                "node",
                "effectiveNodeSettings: kgwNodeEffectiveNodeSettings(net)",
            ),
            (
                "bridge",
                "effectiveNodeSettings: kgwBridgeEffectiveInprocessNodeSettings(net)",
            ),
            ("ipc", "--effective-node-settings-path"),
            (
                "worker",
                "apply_effective_node_settings(effective_node_settings)",
            ),
            ("owner", "fn build_mainline_args"),
        ];
        for (surface, marker) in cases {
            let mut s = fixture();
            match surface {
                "node" => s.node = s.node.replace(marker, ""),
                "bridge" => s.bridge = s.bridge.replace(marker, ""),
                "ipc" => s.ipc = s.ipc.replace(marker, ""),
                "worker" => s.worker = s.worker.replace(marker, ""),
                "owner" => s.owner = s.owner.replace(marker, ""),
                _ => unreachable!(),
            }
            assert!(validate(&s).is_err(), "{surface}:{marker}");
        }
    }

    #[test]
    fn every_owner_mapping_is_required() {
        for field in OWNER_FIELDS {
            let mut s = fixture();
            s.owner = s.owner.replace(field, "");
            assert!(validate(&s).is_err(), "{field}");
        }
    }
}
