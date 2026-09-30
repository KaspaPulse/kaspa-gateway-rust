use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_SOURCE: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const BRIDGE_SOURCE: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const NODE_RAW_RUST: &str = "crates/kaspa-gateway-frontend-wasm/src/node_start_trace.rs";
const NODE_TAB_RUST: &str = "crates/kaspa-gateway-frontend-wasm/src/node_tab.rs";
const BRIDGE_PORT_ORCHESTRATION_RUST: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_port_orchestration.rs";
const BRIDGE_START_TRACE_RUST: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_start_trace.rs";

const NODE_BRIDGE: &str = r##"
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { pathToFileURL } = require("node:url");
const { webcrypto } = require("node:crypto");
let frontendWasm = null;
const [nodePath, bridgePath, requestPath, resultPath] = process.argv.slice(2);
const request = JSON.parse(fs.readFileSync(requestPath, "utf8"));

function dataKey(attr) {
  return attr.slice("data-".length).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}
class ClassList {
  constructor(owner) { this.owner = owner; this.items = new Set(); }
  add(...items) { for (const item of items) this.items.add(String(item)); this.sync(); }
  remove(...items) { for (const item of items) this.items.delete(String(item)); this.sync(); }
  contains(item) { return this.items.has(String(item)); }
  toggle(item, force) {
    const key = String(item);
    const enabled = force === undefined ? !this.items.has(key) : Boolean(force);
    if (enabled) this.items.add(key); else this.items.delete(key);
    this.sync();
    return enabled;
  }
  setFromString(value) {
    this.items = new Set(String(value || "").split(/\s+/).filter(Boolean));
    this.sync();
  }
  sync() { this.owner.attributes.class = Array.from(this.items).join(" "); }
}
class TestElement {
  constructor(tagName, document) {
    this.tagName = String(tagName || "").toUpperCase();
    this.ownerDocument = document;
    this.parentElement = null;
    this.children = [];
    this.attributes = {};
    this.dataset = {};
    this.classList = new ClassList(this);
    this.style = { setProperty(name, value) { this[name] = value; } };
    this.listeners = new Map();
    this.hidden = false;
    this.disabled = false;
    this.readOnly = false;
    this.checked = false;
    this.value = "";
    this.textContent = "";
    this.id = "";
    this.type = "";
    this.scrollTop = 0;
    this.scrollHeight = 0;
  }
  appendChild(child) { child.parentElement = this; this.children.push(child); return child; }
  append(...nodes) { for (const node of nodes) this.appendChild(node); }
  contains(node) { return node === this || walk(this).includes(node); }
  setAttribute(name, value) {
    const key = String(name);
    const stringValue = String(value);
    this.attributes[key] = stringValue;
    if (key === "id") this.id = stringValue;
    if (key === "class") this.classList.setFromString(stringValue);
    if (key === "type") this.type = stringValue;
    if (key === "value") this.value = stringValue;
    if (key === "hidden") this.hidden = true;
    if (key === "disabled") this.disabled = true;
    if (key === "readonly") this.readOnly = true;
    if (key === "checked") this.checked = true;
    if (key.startsWith("data-")) this.dataset[dataKey(key)] = stringValue;
  }
  getAttribute(name) { return this.attributes[String(name)] ?? null; }
  addEventListener(type, handler) {
    const list = this.listeners.get(type) || [];
    list.push(handler);
    this.listeners.set(type, list);
  }
  matches(selector) { return matchesSelector(this, selector); }
  closest(selector) {
    let current = this;
    while (current) {
      if (current.matches(selector)) return current;
      current = current.parentElement;
    }
    return null;
  }
  querySelectorAll(selector) { return querySelectorAll(this, selector); }
  querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
}
class TestDocument extends TestElement {
  constructor() {
    super("#document", null);
    this.ownerDocument = this;
    this.readyState = "complete";
    this.head = new TestElement("head", this);
    this.body = new TestElement("body", this);
    this.appendChild(this.head);
    this.appendChild(this.body);
  }
  createElement(tagName) { return new TestElement(tagName, this); }
  getElementById(id) { return walk(this).find((item) => item.id === id) || null; }
}
function walk(root) {
  const out = [];
  for (const child of root.children || []) {
    out.push(child);
    out.push(...walk(child));
  }
  return out;
}
function querySelectorAll(root, selector) {
  const selectors = String(selector).split(",").map((item) => item.trim()).filter(Boolean);
  const nodes = walk(root);
  return nodes.filter((node) => selectors.some((part) => matchesDescendantSelector(node, part)));
}
function matchesDescendantSelector(node, selector) {
  const parts = selector.split(/\s+/).filter(Boolean);
  if (parts.length === 0 || !matchesSelector(node, parts[parts.length - 1])) return false;
  let current = node.parentElement;
  for (let index = parts.length - 2; index >= 0; index -= 1) {
    while (current && !matchesSelector(current, parts[index])) current = current.parentElement;
    if (!current) return false;
    current = current.parentElement;
  }
  return true;
}
function matchesSelector(node, selector) {
  const alternatives = String(selector).split(",").map((item) => item.trim()).filter(Boolean);
  if (alternatives.length > 1) return alternatives.some((item) => matchesSelector(node, item));
  let rest = alternatives[0] || "";
  const tagMatch = rest.match(/^[A-Za-z0-9_-]+/);
  if (tagMatch) {
    if (node.tagName !== tagMatch[0].toUpperCase()) return false;
    rest = rest.slice(tagMatch[0].length);
  }
  for (const match of [...rest.matchAll(/#([A-Za-z0-9_-]+)/g)]) {
    if (node.id !== match[1]) return false;
  }
  for (const match of [...rest.matchAll(/\.([A-Za-z0-9_-]+)/g)]) {
    if (!node.classList.contains(match[1])) return false;
  }
  for (const match of [...rest.matchAll(/\[([^\]=~\^\$\*\|]+)(?:=(?:"([^"]*)"|'([^']*)'|([^\]]+)))?\]/g)]) {
    const attr = match[1].trim();
    const expected = match[2] ?? match[3] ?? (match[4] ? match[4].replace(/^['"]|['"]$/g, "") : undefined);
    const actual = node.getAttribute(attr);
    if (expected === undefined ? actual === null : actual !== expected) return false;
  }
  return true;
}
function createWindow(calls) {
  const document = new TestDocument();
  const storage = new Map();
  const window = {
    document,
    listeners: new Map(),
    localStorage: {
      getItem(key) { return storage.has(String(key)) ? storage.get(String(key)) : null; },
      setItem(key, value) { storage.set(String(key), String(value)); },
      removeItem(key) { storage.delete(String(key)); },
    },
    setTimeout() { return 1; },
    clearTimeout() {},
    setInterval() { return 1; },
    clearInterval() {},
    queueMicrotask(callback) { Promise.resolve().then(callback); },
    navigator: { clipboard: { writeText: async () => { throw new Error("browser clipboard must not be used"); } } },
    crypto: webcrypto,
    TextEncoder,
    console: { ...console, debug() {} },
    CustomEvent: class CustomEvent {
      constructor(type, options) { this.type = type; this.detail = options && options.detail; }
    },
    Event: class Event {
      constructor(type, options) { this.type = type; this.bubbles = Boolean(options && options.bubbles); }
    },
  };
  window.__TAURI__ = {
    core: {
      invoke: async (command, payload) => {
        calls.push({ command, payload });
        if (typeof window.__KGW_TEST_INVOKE_HANDLER === "function") {
          return await window.__KGW_TEST_INVOKE_HANDLER(command, payload);
        }
        if (command === "kgw_copy_text_to_clipboard_v1") return "clipboard_write_v1;copied=true";
        return true;
      },
    },
  };
  window.addEventListener = (type, handler) => {
    const list = window.listeners.get(type) || [];
    list.push(handler);
    window.listeners.set(type, list);
  };
  window.dispatchEvent = (event) => {
    for (const handler of window.listeners.get(event.type) || []) handler(event);
    return true;
  };
  window.window = window;
  document.defaultView = window;
  return window;
}
function stripModuleSyntax(source) {
  source = source.replace(/^import[\s\S]*?from\s+["'][^"']+["'];\s*/gm, "");
  source = source.replace(/^import\s+["'][^"']+["'];\s*/gm, "");
  source = source.replace(/^await\s+init(?:Node|Bridge)Rust\(\);\s*/gm, "");
  source = source.replace(/export\s+async\s+function\s+initKaspaNodeTab/, "async function initKaspaNodeTab");
  source = source.replace(/export\s+async\s+function\s+initKaspaBridgeTab/, "async function initKaspaBridgeTab");
  source = source.replace(/export\s+default\s+initKaspaNodeTab\s*;/, "");
  source = source.replace(/export\s+default\s+initKaspaBridgeTab\s*;/, "");
  source = source.replace(/^export\s*\{[\s\S]*?\}\s*;\s*$/gm, "");
  return source;
}
function evalFrontend(sourcePath, window, exposeSource) {
  const originalSource = fs.readFileSync(sourcePath, "utf8");
  const source = stripModuleSyntax(originalSource);
  const noop = () => {};
  const sandbox = {
    window,
    document: window.document,
    console: window.console,
    navigator: window.navigator,
    localStorage: window.localStorage,
    setTimeout: window.setTimeout,
    clearTimeout: window.clearTimeout,
    setInterval: window.setInterval,
    clearInterval: window.clearInterval,
    queueMicrotask: window.queueMicrotask,
    crypto: window.crypto,
    TextEncoder: window.TextEncoder,
    CustomEvent: window.CustomEvent,
    Event: window.Event,
    applyStatusTone: noop,
    renderStatusSummary: () => "",
    NODE_ENDPOINTS: {},
    NODE_MANAGED: [],
    NODE_REQUIRED: [],
    NODE_OPTIONAL: [],
    NODE_DANGEROUS: [],
    BRIDGE_MANAGED: [],
    BRIDGE_REQUIRED: [],
    BRIDGE_OPTIONAL: [],
    nodeFieldEnabled: () => true,
    bridgeFieldEnabled: () => true,
    validateNodeForm: () => ({ ok: true, errors: {} }),
    validateBridgeForm: () => ({ ok: true, errors: {} }),
    renderFieldErrors: noop,
    endpoint: () => "",
    runtimePresentation: (input = {}) => {
      const transition = String(input.transition || "");
      const process = input.error
        ? "Error"
        : transition
          ? (transition === "starting" ? "Starting" : transition === "stopping" ? "Stopping" : "Transitioning")
          : input.running
            ? "Running"
            : "Stopped";
      return {
        process,
        processLabel: process,
        profile: input.enabled === false ? "Disabled" : "Enabled",
      };
    },
    runtimeObservationSummary: () => "",
    confirmUserAction: async () => true,
    renderSettingsTabs: noop,
    installSettingsLayout: noop,
    decorateSettingsFields: noop,
    revealSettingsField: noop,
    setSettingFieldState: noop,
  };
  if (frontendWasm) {
    for (const match of originalSource.matchAll(/\b([A-Za-z_$][\w$]*)\s+as\s+([A-Za-z_$][\w$]*)/g)) {
      const exported = frontendWasm[match[1]];
      if (typeof exported === "function") sandbox[match[2]] = exported;
    }
  }
  sandbox.globalThis = sandbox;
  vm.runInNewContext(source + "\n" + exposeSource, sandbox, { filename: sourcePath });
}
function element(document, tag, attrs = {}, text = "") {
  const item = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) item.setAttribute(key, value);
  item.textContent = text;
  return item;
}
function installRawLogDom(window, kind, net) {
  const document = window.document;
  const root = element(document, "section", { id: "kaspa-" + kind });
  const tab = element(document, "button", { class: kind + "-tab active", ["data-" + kind + "-network-tab"]: net, "data-net": net }, net);
  const panel = element(document, "div", { class: "active", ["data-" + kind + "-network-panel"]: net });
  const logPanel = element(document, "div", { ["data-" + kind + "-inner-panel"]: "log", "data-net": net });
  const toolbar = element(document, "div", { class: kind === "node" ? "node-v6-log-toolbar" : "bridge-v7-log-toolbar" });
  const copy = element(document, "button", { ["data-" + kind + "-action"]: "copy-log", "data-net": net }, "Copy Log");
  const clear = element(document, "button", { ["data-" + kind + "-action"]: "clear-log", "data-net": net }, "Clear Log");
  const empty = element(document, "div", { id: kind + "-" + net + "-logEmpty", ["data-" + kind + "-log-empty"]: net }, "No child stdout/stderr received yet.");
  const output = element(document, "pre", { id: kind + "-" + net + "-logOutput" });
  const policyStatus = element(document, "span", { id: kind + "-" + net + "-policyStatus" }, "Unknown");
  const start = element(document, "button", { ["data-" + kind + "-action"]: "start", "data-net": net }, "Start");
  const stop = element(document, "button", { ["data-" + kind + "-action"]: "stop", "data-net": net }, "Stop");
  toolbar.append(copy, clear);
  logPanel.append(toolbar, empty, output, policyStatus, start, stop);
  panel.appendChild(logPanel);
  root.append(tab, panel);
  document.body.appendChild(root);
  return { root, copy, clear, empty, output, policyStatus, start, stop };
}
function callCount(calls, command, stage = "") {
  return calls.filter((call) => call.command === command && (!stage || call.payload?.stage === stage)).length;
}
function copyPayload(calls) {
  return calls.find((call) => call.command === "kgw_copy_text_to_clipboard_v1")?.payload || null;
}
async function waitForCondition(predicate, label, attempts = 80) {
  for (let index = 0; index < attempts; index += 1) {
    if (predicate()) return;
    await new Promise(setImmediate);
  }
  throw new Error("timed out waiting for " + label);
}
function prepare(kind) {
  const calls = [];
  const window = createWindow(calls);
  globalThis.__TAURI__ = window.__TAURI__;
  globalThis.document = window.document;
  globalThis.localStorage = window.localStorage;
  globalThis.CustomEvent = window.CustomEvent;
  globalThis.Event = window.Event;
  globalThis.window = globalThis;
  globalThis.self = globalThis;
  installRawLogDom(window, kind, "mainnet");
  installRawLogDom(window, kind, "testnet10");
  if (kind === "node" && frontendWasm) {
    frontendWasm.nodeClearRawLogBuffer("mainnet", "node");
    frontendWasm.nodeClearRawLogBuffer("testnet10", "node");
  }
  if (kind === "node") {
    window.__kgwRaw = {
      apply: (net, role, report) => frontendWasm.nodeApplyRuntimeLogReport(net, role, report),
      action: (action, net, button) => frontendWasm.nodeHandleLogAction(action, net, button),
      refresh: (net, reason) => frontendWasm.nodeRefreshOne(net, reason),
      setButtons: (net, running) => frontendWasm.nodeSetRuntimeButtons(net, Boolean(running), false, "", ""),
      appendLog: () => {},
    };
  } else {
    evalFrontend(bridgePath, window,
      "window.__kgwRaw = { apply: (net, role, report, instanceId) => wasmBridgeApplyRuntimeLogReport(String(net || \"\"), String(role || \"bridge\"), report, String(instanceId || \"\")), action: kgwBridgeHandleLogActionV29, refresh: kgwBridgeR51RefreshOne, setButtons: kgwBridgeR51SetRuntimeButtons, appendLog: () => {} };");
  }
  return { calls, window, api: window.__kgwRaw };
}
async function rawScenario(kind) {
  const { calls, window, api } = prepare(kind);
  const mainnet = window.document.getElementById(kind + "-mainnet-logOutput");
  const empty = window.document.getElementById(kind + "-mainnet-logEmpty");
  const input = request[kind];
  const untyped1 = kind === "node"
    ? api.apply("mainnet", kind, input.legacyTransport)
    : api.apply("mainnet", kind, input.legacyTransport, "1");
  const afterUntyped1 = mainnet.textContent;
  const untyped2 = kind === "node"
    ? api.apply("mainnet", kind, { rawText: input.diagnosticEnvelope })
    : api.apply("mainnet", kind, { rawText: input.diagnosticEnvelope }, "1");
  const afterUntyped2 = mainnet.textContent;
  if (kind === "node") api.apply("mainnet", kind, { entries: input.entries });
  else api.apply("mainnet", kind, { entries: input.entries }, "1");
  const typedText = mainnet.textContent;
  const emptyHidden = empty.hidden;
  api.appendLog("mainnet", input.syntheticLine);
  const afterAppend = mainnet.textContent;
  calls.length = 0;
  const copySelector = `[data-${kind}-action="copy-log"][data-net="mainnet"]`;
  await api.action("copy-log", "mainnet", window.document.querySelector(copySelector));
  const copied = copyPayload(calls);
  const copyCount = callCount(calls, "kgw_copy_text_to_clipboard_v1");
  const successTraceCount = callCount(calls, "kgw_start_trace_frontend_v1", "frontend.copy_log_succeeded");

  if (kind === "node") api.apply("testnet10", kind, { entries: [input.testnetEntry] });
  else api.apply("testnet10", kind, { entries: [input.testnetEntry] }, "1");
  await api.action("clear-log", "mainnet", window.document.querySelector(`[data-${kind}-action="clear-log"][data-net="mainnet"]`));
  const result = {
    untyped1,
    afterUntyped1,
    untyped2,
    afterUntyped2,
    typedText,
    emptyHidden,
    afterAppend,
    copyCount,
    copied,
    successTraceCount,
    afterClearMainnet: mainnet.textContent,
    afterClearEmptyHidden: empty.hidden,
    testnetText: window.document.getElementById(kind + "-testnet10-logOutput").textContent,
  };
  if (kind === "bridge") {
    api.apply("mainnet", kind, {
      entries: [],
      diagnostics: [{ message: input.statusSummary }]
    }, "1");
    result.afterDiagnosticsText = mainnet.textContent;
    calls.length = 0;
    await api.action("copy-log", "mainnet", window.document.querySelector(copySelector));
    result.missingCopyCount = callCount(calls, "kgw_copy_text_to_clipboard_v1");
    result.failureTraceCount = callCount(calls, "kgw_start_trace_frontend_v1", "frontend.copy_log_failed");
  }
  return result;
}
async function lifecycleScenario(kind) {
  const { calls, window, api } = prepare(kind);
  const output = window.document.getElementById(kind + "-mainnet-logOutput");
  const policyStatus = window.document.getElementById(kind + "-mainnet-policyStatus");
  let releaseStatus;
  const blockedStatus = new Promise((resolve) => { releaseStatus = resolve; });
  window.__KGW_TEST_INVOKE_HANDLER = async (command) => {
    if (command === "kgw_runtime_owner_status_v1") return await blockedStatus;
    if (command === "kgw_kgw_runtime_logs_v1") {
      return { entries: [{ sequence: 9001, network: "mainnet", runtimeRole: kind, stream: "stdout", receivedMs: 1, rawText: kind + " startup raw before status" }] };
    }
    return true;
  };
  const first = api.refresh("mainnet", "rust-regression-blocked-status");
  const second = api.refresh("mainnet", "rust-regression-overlap");
  await waitForCondition(
    () => callCount(calls, "kgw_kgw_runtime_logs_v1") >= 1 && output.textContent.length > 0,
    kind + " first raw log before blocked status"
  );
  const firstRaw = output.textContent;
  const statusCalls1 = callCount(calls, "kgw_runtime_owner_status_v1");
  const logCalls1 = callCount(calls, "kgw_kgw_runtime_logs_v1");
  const third = api.refresh("mainnet", "rust-regression-status-still-blocked");
  await waitForCondition(
    () => callCount(calls, "kgw_kgw_runtime_logs_v1") >= 2,
    kind + " second log poll while status remains blocked"
  );
  const statusCalls2 = callCount(calls, "kgw_runtime_owner_status_v1");
  const logCalls2 = callCount(calls, "kgw_kgw_runtime_logs_v1");
  releaseStatus("parallel-owned-self-worker status;role=" + kind + ";network=mainnet;pid=123;running=true;readiness=READY;runtime_error=none");
  await Promise.all([first, second, third]);
  api.setButtons("mainnet", true);
  window.__KGW_TEST_INVOKE_HANDLER = async (command) => {
    if (command === "kgw_runtime_owner_status_v1") throw new Error("transient status transport failure");
    if (command === "kgw_kgw_runtime_logs_v1") return { entries: [] };
    return true;
  };
  await api.refresh("mainnet", "rust-regression-status-error");
  return {
    firstRaw,
    statusCalls1,
    logCalls1,
    statusCalls2,
    logCalls2,
    policyStateAfterStatusError: String(policyStatus.dataset.state || ""),
  };
}
(async () => {
  const frontendRoot = path.resolve(path.dirname(nodePath), "..", "..", "..");
  const generatedDir = path.join(frontendRoot, "generated", "kgw_frontend_wasm");
  const generatedJs = path.join(generatedDir, "kgw_frontend_wasm.js");
  const generatedWasm = path.join(generatedDir, "kgw_frontend_wasm_bg.wasm");
  frontendWasm = await import(pathToFileURL(generatedJs).href);
  frontendWasm.initSync({ module: fs.readFileSync(generatedWasm) });

  const result = {
    node: await rawScenario("node"),
    bridge: await rawScenario("bridge"),
    lifecycle: {
      node: await lifecycleScenario("node"),
      bridge: await lifecycleScenario("bridge"),
    },
  };
  fs.writeFileSync(resultPath, JSON.stringify(result), "utf8");
})().catch((error) => {
  console.error(error?.stack || String(error));
  process.exitCode = 1;
});
"##;

fn request() -> Value {
    let omega = "\u{03a9}";
    json!({
        "node": {
            "legacyTransport": "kgw_raw_process_log_v1;network=mainnet;line=legacy-wrapper",
            "diagnosticEnvelope": "[KGW_CHILD_STDERR] {\"eventKind\":\"diagnostic_transport_record\"}",
            "syntheticLine": "MAINNET initialized.",
            "entries": [
                {"sequence":20,"network":"mainnet","runtimeRole":"node","source":"self-worker","stream":"stderr","receivedMs":2,"rawText":format!("stderr raw line;equals=value;json={{\"kind\":\"stderr\"}};unicode={omega};path=C:\\Kaspa\\stderr.log")},
                {"sequence":10,"network":"mainnet","runtimeRole":"node","source":"self-worker","stream":"stdout","receivedMs":1,"rawText":format!("2026-07-28 15:10:50.082+03:00 [INFO ] kaspad path=C:\\Kaspa\\node\\kaspad.exe;equals=value;json={{\"kind\":\"stdout\"}};unicode={omega}")},
                {"sequence":30,"network":"testnet10","runtimeRole":"node","source":"self-worker","stream":"stdout","receivedMs":3,"rawText":"testnet10 must not mix into mainnet"},
                {"sequence":40,"network":"mainnet","runtimeRole":"bridge","source":"self-worker","stream":"stdout","receivedMs":4,"rawText":"bridge must not mix into node"},
                {"sequence":45,"network":"mainnet","runtimeRole":"node","source":"self-worker","stream":"stdout","receivedMs":5,"rawText":"kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=node;received_ms=1;line=official-literal"},
                {"sequence":46,"network":"mainnet","runtimeRole":"node","source":"self-worker","stream":"stderr","receivedMs":6,"rawText":"[KGW_CHILD_STDERR] {\"eventKind\":\"diagnostic_transport_record\"}"}
            ],
            "testnetEntry":{"sequence":50,"network":"testnet10","runtimeRole":"node","source":"self-worker","stream":"stdout","receivedMs":5,"rawText":"testnet10 raw only"}
        },
        "bridge": {
            "legacyTransport": "kgw_raw_process_log_v1;network=mainnet;line=legacy-wrapper",
            "diagnosticEnvelope": "{\"eventKind\":\"diagnostic_transport_record\"}",
            "syntheticLine": "KGW bridge start confirmed.",
            "statusSummary": "parallel-owned-self-worker status;role=bridge;network=mainnet;running=false;message=no bridge worker status yet",
            "entries": [
                {"sequence":12,"network":"mainnet","runtimeRole":"bridge","bridgeInstanceId":"1","source":"self-worker","stream":"stderr","receivedMs":2,"rawText":format!("bridge stderr;equals=value;json={{\"kind\":\"bridge-stderr\"}};unicode={omega};path=C:\\Kaspa\\bridge.err")},
                {"sequence":11,"network":"mainnet","runtimeRole":"bridge","bridgeInstanceId":"1","source":"self-worker","stream":"stdout","receivedMs":1,"rawText":format!("bridge stdout;equals=value;json={{\"kind\":\"bridge-stdout\"}};unicode={omega};path=C:\\Kaspa\\bridge.exe")},
                {"sequence":13,"network":"mainnet","runtimeRole":"node","source":"self-worker","stream":"stdout","receivedMs":3,"rawText":"node must not mix into bridge"},
                {"sequence":14,"network":"testnet10","runtimeRole":"bridge","bridgeInstanceId":"1","source":"self-worker","stream":"stdout","receivedMs":4,"rawText":"testnet10 must not mix into mainnet bridge"},
                {"sequence":15,"network":"mainnet","runtimeRole":"bridge","bridgeInstanceId":"2","source":"self-worker","stream":"stdout","receivedMs":5,"rawText":"same Bridge process record"},
                {"sequence":16,"network":"mainnet","runtimeRole":"bridge","bridgeInstanceId":"1","source":"self-worker","stream":"stdout","receivedMs":6,"rawText":"kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=bridge;received_ms=1;line=official-literal"},
                {"sequence":17,"network":"mainnet","runtimeRole":"bridge","bridgeInstanceId":"1","source":"self-worker","stream":"stderr","receivedMs":7,"rawText":"{\"source\":\"native\",\"stage\":\"diagnostic_transport.child.stderr\",\"network\":\"mainnet\",\"eventKind\":\"diagnostic_transport_record\"}"}
            ],
            "testnetEntry":{"sequence":30,"network":"testnet10","runtimeRole":"bridge","bridgeInstanceId":"1","source":"self-worker","stream":"stdout","receivedMs":6,"rawText":"testnet10 bridge raw only"}
        }
    })
}

fn run_bridge(root: &Path) -> Result<Value, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-true-raw-log-frontend-")
        .tempdir()
        .map_err(|error| format!("failed to create true-raw-log tempdir: {error}"))?;
    let script = temp.path().join("bridge.cjs");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(&script, NODE_BRIDGE)
        .map_err(|error| format!("failed to write Node bridge: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(&request())
            .map_err(|error| format!("failed to encode raw-log request: {error}"))?,
    )
    .map_err(|error| format!("failed to write raw-log request: {error}"))?;
    let output = Command::new("node")
        .arg(&script)
        .arg(root.join(NODE_SOURCE))
        .arg(root.join(BRIDGE_SOURCE))
        .arg(&request_path)
        .arg(&result_path)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch raw-log Node bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "true raw-log Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read raw-log bridge result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse raw-log bridge result: {error}"))
}

fn expect(actual: &Value, pointer: &str, expected: Value) -> Result<(), String> {
    let found = actual
        .pointer(pointer)
        .ok_or_else(|| format!("true raw-log result missing {pointer}"))?;
    if found != &expected {
        return Err(format!(
            "true raw-log mismatch at {pointer}: expected {expected}, got {found}"
        ));
    }
    Ok(())
}

fn expected_node_raw() -> String {
    let omega = "\u{03a9}";
    [
        format!("2026-07-28 15:10:50.082+03:00 [INFO ] kaspad path=C:\\Kaspa\\node\\kaspad.exe;equals=value;json={{\"kind\":\"stdout\"}};unicode={omega}"),
        format!("stderr raw line;equals=value;json={{\"kind\":\"stderr\"}};unicode={omega};path=C:\\Kaspa\\stderr.log"),
        "kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=node;received_ms=1;line=official-literal".to_owned(),
        "[KGW_CHILD_STDERR] {\"eventKind\":\"diagnostic_transport_record\"}".to_owned(),
    ].join("\n")
}

fn expected_bridge_raw() -> String {
    let omega = "\u{03a9}";
    [
        format!("bridge stdout;equals=value;json={{\"kind\":\"bridge-stdout\"}};unicode={omega};path=C:\\Kaspa\\bridge.exe"),
        format!("bridge stderr;equals=value;json={{\"kind\":\"bridge-stderr\"}};unicode={omega};path=C:\\Kaspa\\bridge.err"),
        "same Bridge process record".to_owned(),
        "kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=bridge;received_ms=1;line=official-literal".to_owned(),
        "{\"source\":\"native\",\"stage\":\"diagnostic_transport.child.stderr\",\"network\":\"mainnet\",\"eventKind\":\"diagnostic_transport_record\"}".to_owned(),
    ].join("\n")
}

pub fn run(root: &Path) -> Result<String, String> {
    let _node = fs::read_to_string(root.join(NODE_SOURCE))
        .map_err(|error| format!("failed to read {NODE_SOURCE}: {error}"))?;
    let bridge = fs::read_to_string(root.join(BRIDGE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_SOURCE}: {error}"))?;
    let node_raw_rust = fs::read_to_string(root.join(NODE_RAW_RUST))
        .map_err(|error| format!("failed to read {NODE_RAW_RUST}: {error}"))?;
    let node_tab_rust = fs::read_to_string(root.join(NODE_TAB_RUST))
        .map_err(|error| format!("failed to read {NODE_TAB_RUST}: {error}"))?;
    let bridge_port_orchestration_rust =
        fs::read_to_string(root.join(BRIDGE_PORT_ORCHESTRATION_RUST))
            .map_err(|error| format!("failed to read {BRIDGE_PORT_ORCHESTRATION_RUST}: {error}"))?;
    let bridge_start_trace_rust = fs::read_to_string(root.join(BRIDGE_START_TRACE_RUST))
        .map_err(|error| format!("failed to read {BRIDGE_START_TRACE_RUST}: {error}"))?;

    if !bridge_port_orchestration_rust
        .contains("#[wasm_bindgen(js_name = bridgeActiveRawLogInstanceId)]")
    {
        return Err("Bridge active raw-log instance Rust owner export is missing".to_owned());
    }
    if bridge_start_trace_rust.contains("\"activeRawLogInstanceId\"") {
        return Err("Bridge start-trace still depends on the retired JavaScript activeRawLogInstanceId callback".to_owned());
    }
    if bridge.contains("function kgwBridgeActiveRawLogInstanceIdV1(") {
        return Err("Retired Bridge active raw-log instance JavaScript helper remains".to_owned());
    }
    if !bridge.contains("bridgeActiveRawLogInstanceId as wasmBridgeActiveRawLogInstanceId") {
        return Err(
            "Bridge frontend does not import the Rust active raw-log instance owner".to_owned(),
        );
    }

    for symbol in [
        "kgwBridgeHandleLogActionV29",
        "kgwBridgeR51RefreshOne",
        "bridgeRenderRawLogBuffer as wasmBridgeRenderRawLogBuffer",
        "bridgeApplyRuntimeLogReport as wasmBridgeApplyRuntimeLogReport",
    ] {
        if !bridge.contains(symbol) {
            return Err(format!(
                "required Bridge frontend raw-log Rust owner binding missing: {symbol}"
            ));
        }
    }
    for retired in [
        "function kgwBridgeRenderRawLogBufferV1(",
        "function kgwBridgeApplyRuntimeLogReportV1(",
        "function kgwBridgeClearRawLogBufferV1(",
    ] {
        if bridge.contains(retired) {
            return Err(format!(
                "retired Bridge raw-log JavaScript wrapper remains: {retired}"
            ));
        }
    }
    if bridge_start_trace_rust.contains("\"clearRawLogBuffer\"") {
        return Err(
            "Bridge start-trace still depends on the retired JavaScript clearRawLogBuffer callback"
                .to_owned(),
        );
    }
    if !bridge_start_trace_rust.contains("crate::bridge_raw_log::bridge_clear_raw_log_buffer(") {
        return Err(
            "Bridge start-trace does not call the Rust raw-log clear owner directly".to_owned(),
        );
    }
    for symbol in [
        "#[wasm_bindgen(js_name = nodeApplyRuntimeLogReport)]",
        "#[wasm_bindgen(js_name = nodeHandleLogAction)]",
    ] {
        if !node_raw_rust.contains(symbol) {
            return Err(format!(
                "required Node Rust raw-log export missing: {symbol}"
            ));
        }
    }
    for symbol in [
        "#[wasm_bindgen(js_name = nodeRefreshOne)]",
        "#[wasm_bindgen(js_name = nodeSetRuntimeButtons)]",
    ] {
        if !node_tab_rust.contains(symbol) {
            return Err(format!(
                "required Node Rust lifecycle export missing: {symbol}"
            ));
        }
    }

    let actual = run_bridge(root)?;
    let node_raw = expected_node_raw();
    let bridge_raw = expected_bridge_raw();

    expect(&actual, "/node/untyped1", json!(0))?;
    expect(&actual, "/node/afterUntyped1", json!(""))?;
    expect(&actual, "/node/untyped2", json!(0))?;
    expect(&actual, "/node/afterUntyped2", json!(""))?;
    expect(&actual, "/node/typedText", json!(node_raw))?;
    expect(&actual, "/node/emptyHidden", json!(true))?;
    expect(&actual, "/node/afterAppend", json!(expected_node_raw()))?;
    expect(&actual, "/node/copyCount", json!(1))?;
    expect(
        &actual,
        "/node/copied/text",
        json!(expected_node_raw().replace('\n', "\r\n")),
    )?;
    expect(&actual, "/node/copied/runtimeRole", json!("node"))?;
    expect(&actual, "/node/copied/bridgeInstanceId", json!(""))?;
    expect(&actual, "/node/successTraceCount", json!(1))?;
    expect(&actual, "/node/afterClearMainnet", json!(""))?;
    expect(&actual, "/node/afterClearEmptyHidden", json!(false))?;
    expect(&actual, "/node/testnetText", json!("testnet10 raw only"))?;

    expect(&actual, "/bridge/untyped1", json!(0))?;
    expect(&actual, "/bridge/afterUntyped1", json!(""))?;
    expect(&actual, "/bridge/untyped2", json!(0))?;
    expect(&actual, "/bridge/afterUntyped2", json!(""))?;
    expect(&actual, "/bridge/typedText", json!(bridge_raw))?;
    expect(&actual, "/bridge/emptyHidden", json!(true))?;
    expect(&actual, "/bridge/afterAppend", json!(expected_bridge_raw()))?;
    expect(&actual, "/bridge/copyCount", json!(1))?;
    expect(
        &actual,
        "/bridge/copied/text",
        json!(expected_bridge_raw().replace('\n', "\r\n")),
    )?;
    expect(&actual, "/bridge/copied/runtimeRole", json!("bridge"))?;
    expect(&actual, "/bridge/copied/bridgeInstanceId", json!("1"))?;
    expect(&actual, "/bridge/successTraceCount", json!(1))?;
    expect(&actual, "/bridge/afterClearMainnet", json!(""))?;
    expect(&actual, "/bridge/afterClearEmptyHidden", json!(false))?;
    expect(
        &actual,
        "/bridge/testnetText",
        json!("testnet10 bridge raw only"),
    )?;
    expect(&actual, "/bridge/afterDiagnosticsText", json!(""))?;
    expect(&actual, "/bridge/missingCopyCount", json!(0))?;
    expect(&actual, "/bridge/failureTraceCount", json!(1))?;

    for kind in ["node", "bridge"] {
        let base = format!("/lifecycle/{kind}");
        expect(
            &actual,
            &(base.clone() + "/firstRaw"),
            json!(format!("{kind} startup raw before status")),
        )?;
        expect(&actual, &(base.clone() + "/statusCalls1"), json!(1))?;
        expect(&actual, &(base.clone() + "/logCalls1"), json!(1))?;
        expect(&actual, &(base.clone() + "/statusCalls2"), json!(1))?;
        expect(&actual, &(base.clone() + "/logCalls2"), json!(2))?;
        let state = actual
            .pointer(&(base + "/policyStateAfterStatusError"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("{kind} lifecycle policy state missing"))?;
        if state.eq_ignore_ascii_case("stopped") {
            return Err(format!(
                "{kind} transient status failure fabricated STOPPED state"
            ));
        }
    }

    Ok("KGW true raw-log frontend Rust owner PASSED \
(typed-only, sequence order, network/role isolation, Copy Log, clear, lifecycle polling)"
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_request_keeps_expected_matrix_shapes() {
        let request = request();
        assert_eq!(request["node"]["entries"].as_array().map(Vec::len), Some(6));
        assert_eq!(
            request["bridge"]["entries"].as_array().map(Vec::len),
            Some(7)
        );
    }

    #[test]
    fn expected_raw_payloads_keep_transport_looking_official_lines() {
        assert!(expected_node_raw().contains("kgw_raw_process_log_v1"));
        assert!(expected_bridge_raw().contains("diagnostic_transport.child.stderr"));
    }

    #[test]
    fn vm_preprocessor_strips_both_frontend_wasm_bootstraps() {
        assert!(NODE_BRIDGE.contains("init(?:Node|Bridge)Rust"));
        assert!(NODE_BRIDGE.contains("initKaspaNodeTab"));
        assert!(NODE_BRIDGE.contains("initKaspaBridgeTab"));
    }
}
