#!/usr/bin/env node
const assert = require("assert");
const fs = require("fs");
const vm = require("vm");

const sourcePath = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const source = fs.readFileSync(sourcePath, "utf8");

class Element {
  constructor() {
    this.textContent = "";
    this.hidden = false;
    this.disabled = false;
    this.style = {};
    this.title = "";
    this.dataset = {};
    this.value = "";
  }
}

const elements = new Map();
for (const name of ["runtimeError", "runtimeStatus", "policyStatus"]) {
  elements.set(`bridge-mainnet-${name}`, new Element());
}
const start = new Element();
const stop = new Element();
const panel = {
  querySelector(selector) {
    return selector.includes('data-bridge-action="start"') ? start : stop;
  },
};

const local = new Map([["kgw.bridge.network.enabled.mainnet", "1"]]);
let invokeRuntime = async () => "";
const sandbox = {
  document: {
    getElementById(id) {
      return elements.get(id) || null;
    },
    querySelector(selector) {
      return selector.includes('data-bridge-network-panel="mainnet"') ? panel : null;
    },
  },
  localStorage: {
    getItem(key) {
      return local.get(String(key)) || null;
    },
  },
  console,
  setTimeout,
  clearTimeout,
  CustomEvent: class CustomEvent {},
};
sandbox.window = sandbox;
sandbox.globalThis = sandbox;
sandbox.kgwBridgeR51Panel = () => panel;
sandbox.updateCommand = () => "--node-mode=external";
sandbox.bridgeAssertNoPortConflictsR5 = () => ({ ok: true });
sandbox.kgwBridgeV7BlockInprocessIfNodeOwnerRunning = async () => false;
sandbox.invokeBridgeIntegratedRuntime = (...args) => invokeRuntime(...args);
sandbox.kgwBridgeStartWasInprocessR65F = () => false;
sandbox.kgwBridgeRuntimeOwnerTraceR64D = () => {};
sandbox.kgwSetBridgeOwnedNodeLockR65E = () => {};
sandbox.kgwBridgeCurrentNodeModeFromUiR65F = () => "external";
sandbox.kgwBridgePreviewDeclaresInprocessR65F = () => false;
sandbox.kgwBridgeR51KickRawLogLiveR134E = () => {};

const selected = [
  source.slice(source.indexOf("const BRIDGE_NETWORKS = ["), source.indexOf("function kgwBridgeSetNetworkEnabled")),
  source.slice(source.indexOf("function byId("), source.indexOf("function esc(")),
  source.slice(source.indexOf("function id("), source.indexOf("function iid(")),
  source.slice(source.indexOf("function kgwBridgeV7RuntimeRunningFromText("), source.indexOf("function kgwBridgeSetRuntimeErrorV1(")),
  source.slice(source.indexOf("function kgwBridgeSetRuntimeErrorV1("), source.indexOf("async function kgwBridgeV7BlockInprocessIfNodeOwnerRunning")),
  source.slice(source.indexOf("function stringifyRuntimeResult("), source.indexOf("function yesNo(")),
  source.slice(source.indexOf("const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();"), source.indexOf("function getTauriInvoke(")),
  source.slice(source.indexOf("async function runBridgeIntegratedAction("), source.indexOf("/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */")),
  source.slice(source.indexOf("function kgwBridgeR51IsRunning("), source.indexOf("function kgwBridgeR51Delta(")),
].join("\n");

vm.runInNewContext(
  `${selected}\nthis.api = { kgwBridgeV7RuntimeRunningFromText, kgwBridgeR51IsRunning, kgwBridgeR51SetRuntimeButtons, kgwBridgeSetRuntimeErrorV1, kgwBridgeSetRuntimeActivityV1, runBridgeIntegratedAction };`,
  sandbox,
  { filename: sourcePath },
);

const { api } = sandbox;

assert.strictEqual(
  api.kgwBridgeV7RuntimeRunningFromText("role=node;network=mainnet;running=true"),
  false,
  "cross-tab owner checks must not treat pre-readiness liveness as Running",
);
assert.strictEqual(
  api.kgwBridgeV7RuntimeRunningFromText("role=node;network=mainnet;running=true;readiness=READY"),
  true,
  "cross-tab owner checks may treat only READY ownership as Running",
);

assert.strictEqual(
  api.kgwBridgeR51IsRunning("role=bridge;network=mainnet;running=true"),
  false,
  "liveness without READY must not display Running",
);
assert.strictEqual(
  api.kgwBridgeR51IsRunning("role=bridge;network=mainnet;running=true;readiness=READY"),
  true,
  "READY plus owner status must display Running",
);

api.kgwBridgeR51SetRuntimeButtons("mainnet", false, "starting");
assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Starting");
assert.strictEqual(start.disabled, true, "Start must remain disabled while attestation is pending");
assert.strictEqual(stop.disabled, false, "the existing transition state keeps Stop available during startup");

api.kgwBridgeR51SetRuntimeButtons("mainnet", true, "stopping");
assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Stopping");
assert.strictEqual(start.disabled, true, "Start must remain disabled while Stop is pending");
assert.strictEqual(stop.disabled, true, "duplicate Stop must remain disabled while Stop is pending");

assert.ok(
  source.includes("const KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS = 120000"),
  "Bridge Start must wait through the backend readiness window",
);

api.kgwBridgeR51SetRuntimeButtons("mainnet", false);
api.kgwBridgeSetRuntimeErrorV1("mainnet", "occupied listener port");
api.kgwBridgeSetRuntimeActivityV1("mainnet", "Bridge start failed.");
assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Stopped");
assert.strictEqual(elements.get("bridge-mainnet-runtimeError").textContent, "occupied listener port");
assert.strictEqual(elements.get("bridge-mainnet-runtimeError").hidden, false);
assert.strictEqual(elements.get("bridge-mainnet-runtimeStatus").textContent, "Bridge start failed.");

async function pendingInvokeLifecycleTest() {
  let resolveStart;
  invokeRuntime = async () => await new Promise((resolve) => {
    resolveStart = resolve;
  });

  const startPromise = api.runBridgeIntegratedAction("start", "mainnet");
  await Promise.resolve();
  await Promise.resolve();
  await new Promise((resolve) => setImmediate(resolve));
  assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Starting");
  assert.strictEqual(start.disabled, true, "pending backend Start must not display a startable/stopped state");

  resolveStart("parallel-owned-self-worker started;role=bridge;network=mainnet;pid=42;running=true;readiness=READY");
  await startPromise;
  assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Running");
  assert.strictEqual(start.disabled, true);

  invokeRuntime = async () => {
    throw new Error("occupied listener port");
  };
  await api.runBridgeIntegratedAction("start", "mainnet");
  assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Stopped");
  assert.strictEqual(elements.get("bridge-mainnet-runtimeError").textContent, "occupied listener port");

  let resolveStop;
  invokeRuntime = async () => await new Promise((resolve) => {
    resolveStop = resolve;
  });
  api.kgwBridgeR51SetRuntimeButtons("mainnet", true);
  const stopPromise = api.runBridgeIntegratedAction("stop", "mainnet");
  await Promise.resolve();
  await new Promise((resolve) => setImmediate(resolve));
  assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Stopping");
  resolveStop("parallel-owned-self-worker stopped;role=bridge;network=mainnet;running=false;graceful=true;forced=false");
  await stopPromise;
  assert.strictEqual(elements.get("bridge-mainnet-policyStatus").textContent, "Stopped");
  assert.strictEqual(elements.get("bridge-mainnet-runtimeStatus").textContent, "Bridge graceful official shutdown confirmed.");

  invokeRuntime = async () => "parallel-owned-self-worker stopped;role=bridge;network=mainnet;running=false;graceful=false;forced=true;reason=graceful stop timed out";
  api.kgwBridgeR51SetRuntimeButtons("mainnet", true);
  await api.runBridgeIntegratedAction("stop", "mainnet");
  assert.ok(elements.get("bridge-mainnet-runtimeError").textContent.includes("FORCED"));
  assert.strictEqual(elements.get("bridge-mainnet-runtimeStatus").textContent, "Bridge FORCED termination confirmed.");

  invokeRuntime = async () => "parallel-owned-self-worker stopped with graceful failure;role=bridge;network=mainnet;running=false;graceful=false;forced=false;stop_failed=true;stop_outcome=FAILED;reason=official shutdown failed";
  api.kgwBridgeR51SetRuntimeButtons("mainnet", true);
  await api.runBridgeIntegratedAction("stop", "mainnet");
  assert.ok(elements.get("bridge-mainnet-runtimeError").textContent.includes("Official graceful shutdown failed"));
  assert.strictEqual(elements.get("bridge-mainnet-runtimeStatus").textContent, "Bridge worker exited after graceful shutdown failure.");
}

const rawPaneWrites = source.match(/appendLog\([^\n]*(READY|readiness|start failed|start response|start confirmed)/gi) || [];
assert.deepStrictEqual(rawPaneWrites, [], "typed readiness diagnostics must not enter the raw bridge log pane");
assert.ok(!/appendLog\([^\n]*(FORCED|graceful|stop_outcome|Stopping)/i.test(source), "Stop control diagnostics must remain outside raw Bridge logs");
assert.ok(source.includes('experimental: true'));
assert.ok(source.includes('enabledByDefault: false'));
assert.ok(source.includes("requires explicit opt-in"));

pendingInvokeLifecycleTest()
  .then(() => console.log("KGW bridge readiness frontend tests PASSED"))
  .catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
