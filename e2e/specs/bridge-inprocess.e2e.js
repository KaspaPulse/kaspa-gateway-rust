import assert from "node:assert/strict";
import path from "node:path";
import { runtimePortProfile } from "../helpers/runtime-ports.mjs";
import {
  clickTestId,
  openBridgeSettings,
  readBridgeRuntimeSelection,
  setControlValueById,
  setControlValueByTestId,
  shutdownAllRuntimeWorkers,
  stopRuntime,
  waitForOwnerStatus,
  waitForStopped,
} from "../helpers/tauri-app.mjs";
import { waitForPort, waitForPortFree, waitUntil } from "../helpers/windows.mjs";
import { caseDir, ensureDir, writeJson } from "../helpers/paths.mjs";

const ports = runtimePortProfile();
const networks = ["mainnet", "testnet10"];

function profileFor(network) {
  const profile = ports[network];
  assert.ok(profile, `missing runtime port profile for ${network}`);
  return profile;
}
async function ensureInprocessOptionEnabled(network, name) {
  const result = await browser.execute((net, optionName) => {
    const selector = `[data-bridge-command-option-toggle-r7="${optionName}"][data-net="${net}"]`;
    const toggle = document.querySelector(selector);
    if (!toggle) return { ok: false, reason: "missing" };
    if (!toggle.checked) toggle.click();
    return { ok: Boolean(toggle.checked), checked: Boolean(toggle.checked) };
  }, network, name);
  assert.equal(result?.ok, true, `bridge command option ${name}/${network} is not enabled`);
}

async function waitForInprocessControls(network) {
  return await waitUntil(`in-process bridge controls ${network}`, 30000, 250, async () => {
    return await browser.execute((net) => {
      const section = document.querySelector(`[data-bridge-inprocess-node-settings="${net}"]`);
      const rpc = document.querySelector(`#bridge-${net}-inprocessRpcListen`);
      const p2p = document.querySelector(`#bridge-${net}-inprocessListen`);
      if (section?.dataset?.kgwInprocessNodeActive !== "true") return false;
      if (!rpc || !p2p || rpc.disabled || p2p.disabled) return false;
      return { rpc: String(rpc.value || ""), p2p: String(p2p.value || "") };
    }, network);
  });
}
async function configureAndStartInprocessBridge(network, outputDirectory) {
  const profile = profileFor(network);
  await openBridgeSettings(network);
  await setControlValueByTestId(`kgw-bridge-field-${network}-nodeMode`, "inprocess");
  await waitForInprocessControls(network);

  await ensureInprocessOptionEnabled(network, "inprocessRpcListen");
  await ensureInprocessOptionEnabled(network, "inprocessListen");
  await setControlValueById(`bridge-${network}-inprocessRpcListen`, `127.0.0.1:${profile.rpcPort}`);
  await setControlValueById(`bridge-${network}-inprocessListen`, `127.0.0.1:${profile.p2pPort}`);

  let selection = await readBridgeRuntimeSelection(network);
  assert.ok(selection.bridgeInstanceId, "bridge instance id is missing");
  await setControlValueById(
    `bridge-${network}-instancePort-${selection.bridgeInstanceId}`,
    String(profile.bridgePort),
  );
  selection = await readBridgeRuntimeSelection(network);
  assert.equal(selection.bridgePort, profile.bridgePort, "bridge instance port did not retain isolated value");

  await clickTestId(`kgw-bridge-start-${network}`);
  const status = await waitForOwnerStatus({ network, runtimeRole: "bridge", timeoutMs: 180000 });
  await writeJson(path.join(outputDirectory, "owner-status.json"), status);
  const fields = status?.fields || {};
  assert.equal(String(fields.network || "").toLowerCase(), network);
  assert.equal(String(fields.role || "").toLowerCase(), "bridge");
  assert.equal(String(fields.running || "").toLowerCase(), "true");
  assert.equal(String(fields.readiness || "").toUpperCase(), "READY");
  assert.equal(String(fields.node_mode || "").toLowerCase(), "inprocess");
  assert.equal(String(fields.node_kind || "").toLowerCase(), "integrated-inproc");
  assert.equal(String(fields.bridge_kind || "").toLowerCase(), "official-inprocess-node");
  assert.equal(Number(fields.worker_pid || 0), Number(status.pid));
  assert.ok(Number(fields.worker_start_time || 0) > 0, "worker start time is missing");
  assert.ok(String(fields.worker_executable || "").trim(), "worker executable is missing");

  await waitForPort("127.0.0.1", profile.rpcPort, 180000);
  await waitForPort("127.0.0.1", profile.p2pPort, 180000);
  await waitForPort("127.0.0.1", profile.bridgePort, 180000);
  await writeJson(path.join(outputDirectory, "ports-ready.json"), {
    rpc: profile.rpcPort,
    p2p: profile.p2pPort,
    stratum: profile.bridgePort,
  });
  return status;
}
async function stopAndVerifyInprocessBridge(network, outputDirectory) {
  const profile = profileFor(network);
  await stopRuntime(network, "bridge");
  const stopped = await waitForStopped({ network, runtimeRole: "bridge", timeoutMs: 60000 });
  await waitForPortFree("127.0.0.1", profile.bridgePort, 60000);
  await waitForPortFree("127.0.0.1", profile.rpcPort, 60000);
  await waitForPortFree("127.0.0.1", profile.p2pPort, 60000);
  await writeJson(path.join(outputDirectory, "stopped.json"), { stopped });
}

async function exerciseInprocessBridge(network) {
  const outputDirectory = await ensureDir(caseDir(`${network}-bridge-inprocess`));
  try {
    await configureAndStartInprocessBridge(network, outputDirectory);
  } finally {
    await stopAndVerifyInprocessBridge(network, outputDirectory);
  }
}

describe("Kaspa Gateway focused Windows in-process Bridge lifecycle", () => {
  afterEach(async () => {
    await shutdownAllRuntimeWorkers();
  });

  for (const network of networks) {
    it(`${network}: in-process Bridge owns integrated node and frees all listeners on stop`, async () => {
      await exerciseInprocessBridge(network);
    });
  }
});
