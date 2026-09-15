import assert from "node:assert/strict";
import fs from "node:fs";
import { runtimePortProfile } from "./runtime-ports.mjs";

const defaults = runtimePortProfile({});
assert.deepEqual(defaults.mainnet, { rpcPort: 16110, p2pPort: 16111, bridgePort: 5556 });
assert.deepEqual(defaults.testnet10, { rpcPort: 16210, p2pPort: 16211, bridgePort: 5656 });

const isolated = runtimePortProfile({
  KGW_E2E_MAINNET_RPC_PORT: "16120",
  KGW_E2E_MAINNET_P2P_PORT: "16121",
  KGW_E2E_MAINNET_BRIDGE_PORT: "5566",
});
assert.deepEqual(isolated.mainnet, { rpcPort: 16120, p2pPort: 16121, bridgePort: 5566 });
assert.throws(() => runtimePortProfile({ KGW_E2E_MAINNET_RPC_PORT: "80" }), /1024\.\.65535/);
assert.throws(() => runtimePortProfile({ KGW_E2E_MAINNET_RPC_PORT: "abc" }), /integer TCP port/);
const matrixSource = fs.readFileSync(new URL("../specs/zero-touch-live-matrix.e2e.js", import.meta.url), "utf8");
assert.ok(
  matrixSource.includes('setControlCheckedByTestId(`kgw-node-field-${network}-listenEnabled`, true)'),
  "isolated P2P validation must enable the --listen control before changing host/port",
);
console.log("runtime port profile smoke: PASS");
