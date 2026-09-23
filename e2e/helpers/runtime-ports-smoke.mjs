import assert from "node:assert/strict";
import fs from "node:fs";
import { runtimePortProfile } from "./runtime-ports.mjs";

const defaults = runtimePortProfile({});
assert.deepEqual(defaults.mainnet, { rpcPort: 16110, p2pPort: 16111, bridgePort: 5556, externalBridgeListeners: true });
assert.deepEqual(defaults.testnet10, { rpcPort: 16210, p2pPort: 16211, bridgePort: 5656, externalBridgeListeners: false });

const isolated = runtimePortProfile({
  KGW_E2E_MAINNET_RPC_PORT: "16120",
  KGW_E2E_MAINNET_P2P_PORT: "16121",
  KGW_E2E_MAINNET_BRIDGE_PORT: "5566",
});
assert.deepEqual(isolated.mainnet, { rpcPort: 16120, p2pPort: 16121, bridgePort: 5566, externalBridgeListeners: true });
assert.throws(() => runtimePortProfile({ KGW_E2E_MAINNET_RPC_PORT: "80" }), /1024\.\.65535/);
assert.throws(() => runtimePortProfile({ KGW_E2E_MAINNET_RPC_PORT: "abc" }), /integer TCP port/);
const matrixSource = fs.readFileSync(new URL("../specs/zero-touch-live-matrix.e2e.js", import.meta.url), "utf8");
const inprocessSource = fs.readFileSync(new URL("../specs/bridge-inprocess.e2e.js", import.meta.url), "utf8");
const tauriHelperSource = fs.readFileSync(new URL("./tauri-app.mjs", import.meta.url), "utf8");
assert.ok(
  matrixSource.includes('setControlCheckedByTestId(`kgw-node-field-${network}-listenEnabled`, true)'),
  "isolated P2P validation must enable the --listen control before changing host/port",
);
assert.ok(
  matrixSource.includes("externalBridgeListeners: runtimePorts.testnet10.externalBridgeListeners"),
  "zero-touch matrix must consume Testnet10 CPU-only listener policy",
);
assert.ok(
  matrixSource.includes("if (status.externalBridgeListeners)"),
  "zero-touch Stratum probe must be gated by external-listener policy",
);
assert.ok(
  inprocessSource.includes("if (profile.externalBridgeListeners)"),
  "in-process Bridge listener assertions must be policy-gated",
);
assert.ok(
  inprocessSource.includes("CPU-only Bridge must not expose ASIC instances"),
  "in-process Testnet10 must reject fabricated ASIC instance expectations",
);
assert.ok(
  tauriHelperSource.includes("const bridgeInstanceId = active?.dataset?.instanceId ? String(active.dataset.instanceId) : null;"),
  "Bridge selection helper must preserve absence of CPU-only instances",
);
assert.ok(
  !tauriHelperSource.includes('active?.dataset?.instanceId || "1"'),
  "Bridge selection helper must not fabricate instance id 1",
);
console.log("runtime port profile smoke: PASS");
