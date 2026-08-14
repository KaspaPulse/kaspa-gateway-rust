#!/usr/bin/env node
const assert = require("assert");
const fs = require("fs");

const node = fs.readFileSync(
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "utf8",
);
const bridge = fs.readFileSync(
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  "utf8",
);
const ipc = fs.readFileSync(
  "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
  "utf8",
);
const worker = fs.readFileSync(
  "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
  "utf8",
);
const owner = fs.readFileSync(
  "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs",
  "utf8",
);

const schemaFields = [
  "logLevel", "asyncThreads", "ramScale", "yes", "noLogFiles", "sanity",
  "enableUnsyncedMining", "p2pListen", "externalIp", "disableUpnp",
  "disableDnsSeeding", "userAgentComments", "rpcListen", "rpcListenBorsh",
  "rpcListenJson", "rpcMaxClients", "unsafeRpc", "disableGrpc", "connectPeers",
  "addPeers", "outboundTarget", "inboundLimit", "utxoIndex", "archival",
  "resetDb", "perfMetrics", "maxTrackedAddresses", "retentionPeriodDays",
  "perfMetricsIntervalSec", "rocksDbPreset", "rocksDbCacheSize", "rocksDbWalDir",
  "overrideParamsFile", "logDir",
];

assert.ok(node.includes("function kgwNodeEffectiveNodeSettings"));
assert.ok(bridge.includes("function kgwBridgeEffectiveInprocessNodeSettings"));
for (const field of schemaFields) {
  assert.ok(node.includes(field), `Node typed payload missing ${field}`);
  assert.ok(bridge.includes(field), `Bridge in-process typed payload missing ${field}`);
}
assert.ok(node.includes("effectiveNodeSettings: kgwNodeEffectiveNodeSettings(net)"));
assert.ok(bridge.includes("effectiveNodeSettings: kgwBridgeEffectiveInprocessNodeSettings(net)"));
assert.ok(node.includes("Restart required to apply changed effective settings"));
assert.ok(bridge.includes("Restart required to apply changed effective settings"));
assert.ok(node.includes("--configfile is not supported"));
assert.ok(bridge.includes("--configfile is unsupported"));
assert.ok(node.includes("--override-params-file is not supported"));
assert.ok(bridge.includes("In-process --override-params-file is unsupported"));
assert.ok(node.includes("--logdir and --nologfiles cannot be used together"));
assert.ok(node.includes('net.key === "testnet10" ? "16211" : "16311"'));
assert.ok(node.includes('cardCheck(net.key, "disableUpnp", "--disable-upnp", true)'));
assert.ok(node.includes('cardCheck(net.key, "rpcBorshEnabled", "--rpclisten-borsh", false)'));
assert.ok(node.includes('cardCheck(net.key, "rpcJsonEnabled", "--rpclisten-json", false)'));
assert.ok(bridge.includes('id(net.key, "inprocessDisableUpnp")}" type="checkbox" checked'));

assert.ok(ipc.includes("Option<kaspa_gateway_rk_node::EffectiveNodeSettings>"));
assert.ok(ipc.includes("--effective-node-settings-path"));
assert.ok(ipc.includes("kgw_worker_atomic_write_json_v1(&effective_node_settings_path"));
assert.ok(worker.includes("serde_json::from_slice::<kaspa_gateway_rk_node::EffectiveNodeSettings>"));
assert.ok(worker.includes("apply_effective_node_settings(effective_node_settings)"));
assert.ok(owner.includes("fn build_mainline_args"));
assert.ok(owner.includes("fn build_tn12_args"));
for (const rustField of [
  "log_level", "async_threads", "ram_scale", "connect_peers", "add_peers",
  "rpclisten_borsh", "rpclisten_json", "rpc_max_clients", "outbound_target",
  "inbound_limit", "reset_db", "perf_metrics", "retention_period_days",
  "rocksdb_preset", "rocksdb_cache_size", "rocksdb_wal_dir", "override_params_file",
]) {
  assert.ok(owner.includes(rustField), `pinned Args mapping missing ${rustField}`);
}

console.log("KGW effective Node settings gate PASSED");
