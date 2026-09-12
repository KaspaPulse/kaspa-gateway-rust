import assert from "node:assert/strict";
import path from "node:path";
import { runtimePortProfile } from "../helpers/runtime-ports.mjs";
import {
  clickTestId,
  openNodeSettings,
  setControlCheckedByTestId,
  setControlValueByTestId,
  shutdownAllRuntimeWorkers,
  stopRuntime,
  waitForOwnerStatus,
  waitForStopped,
} from "../helpers/tauri-app.mjs";
import {
  killExactOwnedProcess,
  waitForPort,
  waitForPortFree,
  waitUntil,
} from "../helpers/windows.mjs";
import { caseDir, ensureDir, writeJson } from "../helpers/paths.mjs";

const ports = runtimePortProfile();
const networks = ["mainnet", "testnet10"];
function nodePorts(network) {
  const profile = ports[network];
  assert.ok(profile, `missing runtime port profile for ${network}`);
  return profile;
}

function assertExactOwner(status, network) {
  const fields = status?.fields || {};
  assert.equal(String(fields.network || "").toLowerCase(), network);
  assert.equal(String(fields.role || "").toLowerCase(), "node");
  assert.equal(String(fields.running || "").toLowerCase(), "true");
  assert.equal(String(fields.readiness || "").toUpperCase(), "READY");
  assert.equal(Number(fields.worker_pid || 0), Number(status.pid));
  assert.ok(Number(fields.worker_start_time || 0) > 0, "worker start time is missing");
  assert.ok(String(fields.worker_executable || "").trim(), "worker executable is missing");
  assert.ok(Number(fields.parent_pid || 0) > 0, "parent PID is missing");
  assert.ok(Number(fields.parent_start_time || 0) > 0, "parent start time is missing");
  assert.ok(String(fields.parent_executable || "").trim(), "parent executable is missing");
  return fields;
}

async function configureAndStartNode(network, outputDirectory, label) {
  const profile = nodePorts(network);
  await openNodeSettings(network);
  await setControlValueByTestId(`kgw-node-field-${network}-rpcListenHost`, "127.0.0.1");
  await setControlValueByTestId(`kgw-node-field-${network}-rpcListenPort`, String(profile.rpcPort));
  await setControlCheckedByTestId(`kgw-node-field-${network}-listenEnabled`, true);
  await setControlValueByTestId(`kgw-node-field-${network}-listenHost`, "127.0.0.1");
  await setControlValueByTestId(`kgw-node-field-${network}-listenPort`, String(profile.p2pPort));
  await clickTestId(`kgw-node-start-${network}`);

  const status = await waitForOwnerStatus({ network, runtimeRole: "node", timeoutMs: 180000 });
  const fields = assertExactOwner(status, network);
  await waitForPort("127.0.0.1", profile.rpcPort, 180000);
  await waitForPort("127.0.0.1", profile.p2pPort, 180000);
  await writeJson(path.join(outputDirectory, `${label}-owner-status.json`), status);
  return { status, fields };
}

async function waitForNodeUi(network, expectedRunning) {
  return await waitUntil(`node UI ${network} running=${expectedRunning}`, 45000, 500, async () => {
    await openNodeSettings(network);
    const state = await browser.execute((net) => {
      const start = document.querySelector(`[data-testid="kgw-node-start-${net}"]`);
      const stop = document.querySelector(`[data-testid="kgw-node-stop-${net}"]`);
      return {
        startExists: Boolean(start),
        stopExists: Boolean(stop),
        startDisabled: Boolean(start?.disabled || start?.getAttribute("aria-disabled") === "true"),
        stopDisabled: Boolean(stop?.disabled || stop?.getAttribute("aria-disabled") === "true"),
      };
    }, network);
    if (!state.startExists || !state.stopExists) return false;
    if (expectedRunning && state.startDisabled && !state.stopDisabled) return state;
    if (!expectedRunning && !state.startDisabled && state.stopDisabled) return state;
    return false;
  });
}

async function stopAndVerify(network, outputDirectory, label) {
  const profile = nodePorts(network);
  await stopRuntime(network, "node");
  const stopped = await waitForStopped({ network, runtimeRole: "node", timeoutMs: 60000 });
  await waitForPortFree("127.0.0.1", profile.rpcPort, 60000);
  await waitForPortFree("127.0.0.1", profile.p2pPort, 60000);
  const ui = await waitForNodeUi(network, false);
  await writeJson(path.join(outputDirectory, `${label}-stopped.json`), { stopped, ui });
}
async function exerciseRecovery(network) {
  const outputDirectory = await ensureDir(caseDir(`${network}-node-recovery`));
  const first = await configureAndStartNode(network, outputDirectory, "start-1");
  await waitForNodeUi(network, true);
  await stopAndVerify(network, outputDirectory, "normal-stop");

  const second = await configureAndStartNode(network, outputDirectory, "restart-2");
  assert.notEqual(second.status.pid, first.status.pid, "restart must acquire a new worker PID");
  await waitForNodeUi(network, true);

  const crashEvidencePath = path.join(outputDirectory, "forced-crash.json");
  await killExactOwnedProcess({
    pid: second.status.pid,
    expectedExecutable: second.fields.worker_executable,
    expectedStartTime: Number(second.fields.worker_start_time),
    outputPath: crashEvidencePath,
  });
  const reconciled = await waitForStopped({ network, runtimeRole: "node", timeoutMs: 60000 });
  const profile = nodePorts(network);
  await waitForPortFree("127.0.0.1", profile.rpcPort, 60000);
  await waitForPortFree("127.0.0.1", profile.p2pPort, 60000);
  const uiAfterCrash = await waitForNodeUi(network, false);
  await writeJson(path.join(outputDirectory, "crash-reconciled.json"), {
    killedPid: second.status.pid,
    reconciled,
    ui: uiAfterCrash,
  });

  const third = await configureAndStartNode(network, outputDirectory, "recovery-3");
  assert.notEqual(third.status.pid, second.status.pid, "crash recovery must acquire a new worker PID");
  await waitForNodeUi(network, true);
  await stopAndVerify(network, outputDirectory, "final-stop");
}

describe("Kaspa Gateway focused Windows node lifecycle recovery", () => {
  afterEach(async () => {
    await shutdownAllRuntimeWorkers();
  });

  for (const network of networks) {
    it(`${network}: restart and exact-owner crash recovery reconcile to runtime truth`, async () => {
      await exerciseRecovery(network);
    });
  }
});
