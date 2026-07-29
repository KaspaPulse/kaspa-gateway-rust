#!/usr/bin/env node
const assert = require("assert");
const { webcrypto } = require("crypto");
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const repo = process.cwd();
const nodeJsPath = path.join(repo, "apps", "kaspa-gateway-desktop", "frontend", "src", "tabs", "kaspa-node", "kaspa-node.js");
const bridgeJsPath = path.join(repo, "apps", "kaspa-gateway-desktop", "frontend", "src", "tabs", "kaspa-bridge", "kaspa-bridge.js");

function fail(error) {
  console.error("KGW true raw log frontend tests FAILED");
  console.error(error && error.stack ? error.stack : String(error));
  process.exit(1);
}

function dataKey(attr) {
  return attr
    .slice("data-".length)
    .replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

class ClassList {
  constructor(owner) {
    this.owner = owner;
    this.items = new Set();
  }

  add(...items) {
    for (const item of items) this.items.add(String(item));
    this.sync();
  }

  remove(...items) {
    for (const item of items) this.items.delete(String(item));
    this.sync();
  }

  contains(item) {
    return this.items.has(String(item));
  }

  toggle(item, force) {
    const key = String(item);
    const enabled = force === undefined ? !this.items.has(key) : Boolean(force);
    if (enabled) this.items.add(key);
    else this.items.delete(key);
    this.sync();
    return enabled;
  }

  setFromString(value) {
    this.items = new Set(String(value || "").split(/\s+/).filter(Boolean));
    this.sync();
  }

  sync() {
    this.owner.attributes.class = Array.from(this.items).join(" ");
  }
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

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }

  append(...nodes) {
    for (const node of nodes) this.appendChild(node);
  }

  contains(node) {
    if (node === this) return true;
    return walk(this).includes(node);
  }

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

  getAttribute(name) {
    return this.attributes[String(name)] ?? null;
  }

  addEventListener(type, handler) {
    const list = this.listeners.get(type) || [];
    list.push(handler);
    this.listeners.set(type, list);
  }

  matches(selector) {
    return matchesSelector(this, selector);
  }

  closest(selector) {
    let current = this;
    while (current) {
      if (current.matches(selector)) return current;
      current = current.parentElement;
    }
    return null;
  }

  querySelectorAll(selector) {
    return querySelectorAll(this, selector);
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }
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

  createElement(tagName) {
    return new TestElement(tagName, this);
  }

  getElementById(id) {
    return walk(this).find((item) => item.id === id) || null;
  }
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
  if (parts.length === 0) return false;
  if (!matchesSelector(node, parts[parts.length - 1])) return false;

  let current = node.parentElement;
  for (let index = parts.length - 2; index >= 0; index -= 1) {
    while (current && !matchesSelector(current, parts[index])) {
      current = current.parentElement;
    }
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

  const idMatches = [...rest.matchAll(/#([A-Za-z0-9_-]+)/g)];
  for (const match of idMatches) {
    if (node.id !== match[1]) return false;
  }

  const classMatches = [...rest.matchAll(/\.([A-Za-z0-9_-]+)/g)];
  for (const match of classMatches) {
    if (!node.classList.contains(match[1])) return false;
  }

  const attrMatches = [...rest.matchAll(/\[([^\]=~\^\$\*\|]+)(?:=(?:"([^"]*)"|'([^']*)'|([^\]]+)))?\]/g)];
  for (const match of attrMatches) {
    const attr = match[1].trim();
    const expected = match[2] ?? match[3] ?? (match[4] ? match[4].replace(/^['"]|['"]$/g, "") : undefined);
    const actual = node.getAttribute(attr);
    if (expected === undefined) {
      if (actual === null) return false;
    } else if (actual !== expected) {
      return false;
    }
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
      constructor(type, options) {
        this.type = type;
        this.detail = options && options.detail;
      }
    },
    Event: class Event {
      constructor(type, options) {
        this.type = type;
        this.bubbles = Boolean(options && options.bubbles);
      }
    },
  };
  window.__TAURI__ = {
    core: {
      invoke: async (command, payload) => {
        calls.push({ command, payload });
        if (command === "kgw_copy_text_to_clipboard_v1") {
          return "clipboard_write_v1;copied=true";
        }
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

function evalFrontend(sourcePath, window, exposeSource) {
  const source = fs.readFileSync(sourcePath, "utf8")
    .replace(/export\s+async\s+function\s+initKaspaNodeTab/, "async function initKaspaNodeTab")
    .replace(/export\s+async\s+function\s+initKaspaBridgeTab/, "async function initKaspaBridgeTab")
    .replace(/export\s+default\s+initKaspaNodeTab\s*;/, "")
    .replace(/export\s+default\s+initKaspaBridgeTab\s*;/, "");
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
  };
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

  toolbar.append(copy, clear);
  logPanel.append(toolbar, empty, output);
  panel.appendChild(logPanel);
  root.append(tab, panel);
  document.body.appendChild(root);

  return { root, copy, clear, empty, output };
}

function assertNoTransportText(text, label) {
  for (const forbidden of [
    "kgw_raw_process_log_v1",
    "source=self-worker",
    "diagnostic_transport_record",
    "runtime_role=",
    "stream=stdout",
    "stream=stderr",
    "received_ms=",
    "[KGW_CHILD_STDOUT]",
    "[KGW_CHILD_STDERR]",
  ]) {
    assert.ok(!String(text).includes(forbidden), label + " must not contain " + forbidden + ": " + text);
  }
}

function copyCalls(calls) {
  return calls.filter((call) => call.command === "kgw_copy_text_to_clipboard_v1");
}

function traceCalls(calls, stage) {
  return calls.filter((call) => call.command === "kgw_start_trace_frontend_v1" && (!stage || call.payload.stage === stage));
}

async function nodeRawLogFrontendTests() {
  const calls = [];
  const window = createWindow(calls);
  installRawLogDom(window, "node", "mainnet");
  installRawLogDom(window, "node", "testnet10");
  evalFrontend(
    nodeJsPath,
    window,
    "window.__kgwTrueRawNodeTest = { kgwNodeApplyRuntimeLogReportV1, kgwNodeHandleLogActionV29, kgwNodeClearRawLogBufferV1, kgwNodeVisibleRawLogTextV1, appendLog };",
  );
  const api = window.__kgwTrueRawNodeTest;
  const unicode = String.fromCodePoint(0x03a9);
  const stdoutLine = `2026-07-28 15:10:50.082+03:00 [INFO ] kaspad path=C:\\Kaspa\\node\\kaspad.exe;equals=value;json={"kind":"stdout"};unicode=${unicode}`;
  const stderrLine = `stderr raw line;equals=value;json={"kind":"stderr"};unicode=${unicode};path=C:\\Kaspa\\stderr.log`;

  api.kgwNodeApplyRuntimeLogReportV1("mainnet", "node", {
    entries: [
      { sequence: 20, network: "mainnet", runtimeRole: "node", source: "self-worker", stream: "stderr", receivedMs: 2, rawText: stderrLine },
      { sequence: 10, network: "mainnet", runtimeRole: "node", source: "self-worker", stream: "stdout", receivedMs: 1, rawText: stdoutLine },
      { sequence: 30, network: "testnet10", runtimeRole: "node", source: "self-worker", stream: "stdout", receivedMs: 3, rawText: "testnet10 must not mix into mainnet" },
      { sequence: 40, network: "mainnet", runtimeRole: "bridge", source: "self-worker", stream: "stdout", receivedMs: 4, rawText: "bridge must not mix into node" },
      { sequence: 45, network: "mainnet", runtimeRole: "node", source: "self-worker", stream: "stdout", receivedMs: 5, rawText: "kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=node;received_ms=1;line=fake" },
      { sequence: 46, network: "mainnet", runtimeRole: "node", source: "self-worker", stream: "stderr", receivedMs: 6, rawText: "[KGW_CHILD_STDERR] {\"eventKind\":\"diagnostic_transport_record\"}" },
    ],
  });

  const mainnet = window.document.getElementById("node-mainnet-logOutput");
  const mainnetEmpty = window.document.getElementById("node-mainnet-logEmpty");
  assert.strictEqual(mainnet.textContent, stdoutLine + "\n" + stderrLine, "Sequence ordering is preserved even when node events arrive out of order");
  assert.strictEqual(mainnetEmpty.hidden, true, "Node empty state must be separate from raw log text");
  assertNoTransportText(mainnet.textContent, "Displayed node raw log");

  api.appendLog("mainnet", "MAINNET initialized.");
  assert.strictEqual(mainnet.textContent, stdoutLine + "\n" + stderrLine, "No fabricated raw lines are created by legacy appendLog");

  calls.length = 0;
  await api.kgwNodeHandleLogActionV29("copy-log", "mainnet", window.document.querySelector('[data-node-action="copy-log"][data-net="mainnet"]'));
  assert.strictEqual(copyCalls(calls).length, 1, "Node Copy Log must invoke native clipboard once");
  assert.strictEqual(copyCalls(calls)[0].payload.text, (stdoutLine + "\n" + stderrLine).replace(/\n/g, "\r\n"), "Node Copy Log equals visible raw output byte-for-byte after newline normalization");
  assert.strictEqual(copyCalls(calls)[0].payload.runtimeRole, "node", "Node Copy Log must pass runtime role metadata");
  assert.strictEqual(copyCalls(calls)[0].payload.bridgeInstanceId, "", "Node Copy Log must not invent a bridge instance");
  assertNoTransportText(copyCalls(calls)[0].payload.text, "Copied node raw log");
  assert.strictEqual(traceCalls(calls, "frontend.copy_log_succeeded").length, 1, "Node Copy Log must emit an event-time success trace");

  api.kgwNodeApplyRuntimeLogReportV1("testnet10", "node", {
    entries: [{ sequence: 50, network: "testnet10", runtimeRole: "node", source: "self-worker", stream: "stdout", receivedMs: 5, rawText: "testnet10 raw only" }],
  });
  await api.kgwNodeHandleLogActionV29("clear-log", "mainnet", window.document.querySelector('[data-node-action="clear-log"][data-net="mainnet"]'));
  assert.strictEqual(mainnet.textContent, "", "Node Clear Log affects only the selected buffer");
  assert.strictEqual(window.document.getElementById("node-mainnet-logEmpty").hidden, false, "Node empty output is shown outside raw text");
  assert.strictEqual(window.document.getElementById("node-testnet10-logOutput").textContent, "testnet10 raw only", "Mainnet clear must not affect testnet10");
}

async function bridgeRawLogFrontendTests() {
  const calls = [];
  const window = createWindow(calls);
  installRawLogDom(window, "bridge", "mainnet");
  installRawLogDom(window, "bridge", "testnet10");
  evalFrontend(
    bridgeJsPath,
    window,
    "window.__kgwTrueRawBridgeTest = { kgwBridgeApplyRuntimeLogReportV1, kgwBridgeHandleLogActionV29, kgwBridgeClearRawLogBufferV1, kgwBridgeRenderRawLogBufferV1, appendLog };",
  );
  const api = window.__kgwTrueRawBridgeTest;
  const unicode = String.fromCodePoint(0x03a9);
  const stdoutLine = `bridge stdout;equals=value;json={"kind":"bridge-stdout"};unicode=${unicode};path=C:\\Kaspa\\bridge.exe`;
  const stderrLine = `bridge stderr;equals=value;json={"kind":"bridge-stderr"};unicode=${unicode};path=C:\\Kaspa\\bridge.err`;

  api.kgwBridgeApplyRuntimeLogReportV1("mainnet", "bridge", {
    entries: [
      { sequence: 12, network: "mainnet", runtimeRole: "bridge", bridgeInstanceId: "1", source: "self-worker", stream: "stderr", receivedMs: 2, rawText: stderrLine },
      { sequence: 11, network: "mainnet", runtimeRole: "bridge", bridgeInstanceId: "1", source: "self-worker", stream: "stdout", receivedMs: 1, rawText: stdoutLine },
      { sequence: 13, network: "mainnet", runtimeRole: "node", source: "self-worker", stream: "stdout", receivedMs: 3, rawText: "node must not mix into bridge" },
      { sequence: 14, network: "testnet10", runtimeRole: "bridge", bridgeInstanceId: "1", source: "self-worker", stream: "stdout", receivedMs: 4, rawText: "testnet10 must not mix into mainnet bridge" },
      { sequence: 15, network: "mainnet", runtimeRole: "bridge", bridgeInstanceId: "2", source: "self-worker", stream: "stdout", receivedMs: 5, rawText: "wrong bridge instance must not mix" },
      { sequence: 16, network: "mainnet", runtimeRole: "bridge", bridgeInstanceId: "1", source: "self-worker", stream: "stdout", receivedMs: 6, rawText: "kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=bridge;received_ms=1;line=fake" },
      { sequence: 17, network: "mainnet", runtimeRole: "bridge", bridgeInstanceId: "1", source: "self-worker", stream: "stderr", receivedMs: 7, rawText: "{\"source\":\"native\",\"stage\":\"diagnostic_transport.child.stderr\",\"network\":\"mainnet\",\"eventKind\":\"diagnostic_transport_record\"}" },
    ],
  }, "1");

  const mainnet = window.document.getElementById("bridge-mainnet-logOutput");
  assert.strictEqual(mainnet.textContent, stdoutLine + "\n" + stderrLine, "Bridge sequence ordering is preserved for the selected instance");
  assertNoTransportText(mainnet.textContent, "Displayed bridge raw log");
  api.appendLog("mainnet", "KGW bridge start confirmed.");
  assert.strictEqual(mainnet.textContent, stdoutLine + "\n" + stderrLine, "Bridge legacy appendLog must not fabricate raw lines");

  calls.length = 0;
  await api.kgwBridgeHandleLogActionV29("copy-log", "mainnet", window.document.querySelector('[data-bridge-action="copy-log"][data-net="mainnet"]'));
  assert.strictEqual(copyCalls(calls).length, 1, "Bridge Copy Log must invoke native clipboard once");
  assert.strictEqual(copyCalls(calls)[0].payload.text, (stdoutLine + "\n" + stderrLine).replace(/\n/g, "\r\n"), "Bridge Copy Log equals visible raw output byte-for-byte after newline normalization");
  assert.strictEqual(copyCalls(calls)[0].payload.runtimeRole, "bridge", "Bridge Copy Log must pass runtime role metadata");
  assert.strictEqual(copyCalls(calls)[0].payload.bridgeInstanceId, "1", "Bridge Copy Log must pass active bridge instance metadata");
  assertNoTransportText(copyCalls(calls)[0].payload.text, "Copied bridge raw log");
  assert.strictEqual(traceCalls(calls, "frontend.copy_log_succeeded").length, 1, "Bridge Copy Log must emit an event-time success trace");

  api.kgwBridgeApplyRuntimeLogReportV1("testnet10", "bridge", {
    entries: [{ sequence: 30, network: "testnet10", runtimeRole: "bridge", bridgeInstanceId: "1", source: "self-worker", stream: "stdout", receivedMs: 6, rawText: "testnet10 bridge raw only" }],
  }, "1");
  await api.kgwBridgeHandleLogActionV29("clear-log", "mainnet", window.document.querySelector('[data-bridge-action="clear-log"][data-net="mainnet"]'));
  assert.strictEqual(mainnet.textContent, "", "Bridge Clear Log affects only the selected network and instance buffer");
  assert.strictEqual(window.document.getElementById("bridge-mainnet-logEmpty").hidden, false, "Bridge empty output is shown outside raw text");
  assert.strictEqual(window.document.getElementById("bridge-testnet10-logOutput").textContent, "testnet10 bridge raw only", "Mainnet bridge clear must not affect testnet10 bridge");

  api.kgwBridgeApplyRuntimeLogReportV1("mainnet", "bridge", {
    entries: [],
    diagnostics: [{ message: "parallel-owned-self-worker status;role=bridge;network=mainnet;running=false;message=no bridge worker status yet" }],
  }, "1");
  assert.strictEqual(mainnet.textContent, "", "Bridge status summaries must be rejected as raw process output");

  calls.length = 0;
  await api.kgwBridgeHandleLogActionV29("copy-log", "mainnet", window.document.querySelector('[data-bridge-action="copy-log"][data-net="mainnet"]'));
  assert.strictEqual(copyCalls(calls).length, 0, "Missing bridge output must not reach native clipboard");
  assert.strictEqual(traceCalls(calls, "frontend.copy_log_failed").length, 1, "Missing bridge output must emit a Copy Log failure trace");
}

(async () => {
  try {
    await nodeRawLogFrontendTests();
    await bridgeRawLogFrontendTests();
    console.log("KGW true raw log frontend tests PASSED");
  } catch (error) {
    fail(error);
  }
})();
