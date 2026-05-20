#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2] ? path.resolve(process.argv[2]) : process.cwd();

const files = [
  "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
  "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs",
  "crates/kaspa-gateway-rk-bridge/src/lib.rs"
];

function read(rel) {
  const full = path.join(repoRoot, rel);
  if (!fs.existsSync(full)) return "";
  return fs.readFileSync(full, "utf8");
}

function fail(message, details) {
  console.error("KGW global owner gate failed: " + message);
  for (const detail of details || []) console.error("- " + detail);
  process.exit(1);
}

const contents = Object.fromEntries(files.map((rel) => [rel, read(rel)]));
const all = Object.entries(contents).map(([rel, text]) => "\n### " + rel + "\n" + text).join("\n");
const lib = contents["apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"] || "";

const required = [
  ["V14 SetLoggerError patch marker", "KGW_BRIDGE_INPROCESS_SETLOGGERERROR_V14"],
  ["V14 inprocess mode check", "bridge_owns_inprocess_node"],
  ["V14 skip tracing message", "skipping bridge tracing subscriber before embedded kaspad logger initialization"],
  ["V14 external tracing still preserved", "kgw_init_bridge_self_worker_raw_tracing_r23();"],
  ["V12D bridge UI marker", "KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D"],
  ["V12D command value helper", "bridgeInprocessAddKaspadValueArgV12D"],
  ["V7 bridge inprocess frontend guard", "kgwBridgeV7BlockInprocessIfNodeOwnerRunning"],
  ["V7 node bridge inprocess lock", "kgwNodeR51BridgeInprocessLockedV7"],
  ["V7 backend reports node_mode", "node_mode: String"],
  ["V7 backend blocks bridge when node owns same network", "block_reason=node-tab-owner-running"],
  ["V7 backend blocks node when bridge inprocess owns same network", "block_reason=bridge-inprocess-owner-running"],
  ["V7 same DB policy marker", "same_db_path=true"],
  ["V7 exclusive owner policy marker", "exclusive_node_owner_per_network=true"]
];

const missing = required
  .filter(([_, needle]) => !all.includes(needle))
  .map(([label, needle]) => label + " [" + needle + "]");

if (missing.length) {
  fail("required V7/V12D/V14 markers are missing", missing);
}

const roleBridgeIndex = lib.indexOf('"bridge" => {');
const bridgeRunIndex = lib.indexOf("kgw_run_bridge_self_worker(&network, &appdir, &rpc, &stratum, &args)", Math.max(0, roleBridgeIndex));
const initIndex = lib.indexOf("kgw_init_bridge_self_worker_raw_tracing_r23();", Math.max(0, roleBridgeIndex));
const inprocessConditionIndex = lib.indexOf("if bridge_owns_inprocess_node", Math.max(0, roleBridgeIndex));

if (roleBridgeIndex < 0 || bridgeRunIndex < 0 || initIndex < 0 || inprocessConditionIndex < 0) {
  fail("V14 bridge self-worker structure was not found", []);
}

if (!(roleBridgeIndex < inprocessConditionIndex && inprocessConditionIndex < bridgeRunIndex)) {
  fail("V14 inprocess condition must run before bridge self-worker starts", []);
}

if (!(inprocessConditionIndex < initIndex && initIndex < bridgeRunIndex)) {
  fail("V14 tracing init must be inside external branch before bridge run, not unconditional before mode check", []);
}

const forbidden = [
  ["Old unconditional bridge tracing before role match", 'let result = match role.trim().to_ascii_lowercase().as_str() {\n        "node" =>'],
  ["Runtime ReferenceError source", "addValueArg(kaspadArgs,"],
  ["Double async function syntax", "async async function"],
  ["Old fake bridge raw log placeholder", "Live refresh active; bridge is still running. Waiting for new runtime stdout/stderr lines."],
  ["Empty inprocess bridge marker", "if event.mode == BridgeRuntimeMode::OfficialInProcessNode {}"],
  ["Downgrade inprocess bridge mode to external", "BridgeNodeKind::OfficialInProcessNode => {\n            kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialExternalNode\n        }"],
  ["V9C old UI owner marker", "KGW_BRIDGE_INPROCESS_NODE_SETTINGS_UI_V9C"],
  ["V12B old owner marker", "KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12B"],
  ["V12C old owner marker", "KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12C"]
];

const presentForbidden = forbidden
  .filter(([_, needle]) => all.includes(needle))
  .map(([label, needle]) => label + " [" + needle + "]");

if (presentForbidden.length) {
  fail("forbidden markers are present", presentForbidden);
}

console.log("KGW global owner gate passed: bridge inprocess SetLoggerError V14 markers are valid.");
process.exit(0);
