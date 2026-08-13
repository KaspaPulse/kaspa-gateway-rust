#!/usr/bin/env node
const assert = require("assert");
const { webcrypto } = require("crypto");
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const repo = process.cwd();
const nodeJsPath = path.join(
  repo,
  "apps",
  "kaspa-gateway-desktop",
  "frontend",
  "src",
  "tabs",
  "kaspa-node",
  "kaspa-node.js",
);
const source = fs.readFileSync(nodeJsPath, "utf8");
const tauriConfigPath = path.join(
  repo,
  "apps",
  "kaspa-gateway-desktop",
  "src-tauri",
  "tauri.conf.json",
);

function fail(message) {
  console.error("KGW start button frontend tests FAILED");
  console.error(message);
  process.exit(1);
}

function assertIncludes(text, needle, message) {
  assert.ok(text.includes(needle), message + " (`" + needle + "` missing)");
}

function extractBetween(text, start, end) {
  const startIndex = text.indexOf(start);
  assert.ok(startIndex >= 0, "missing start marker: " + start);
  const endIndex = text.indexOf(end, startIndex + start.length);
  assert.ok(endIndex > startIndex, "missing end marker: " + end);
  return text.slice(startIndex, endIndex);
}

function staticPlacementTests() {
  const renderStart = source.indexOf("function renderNetworkPanel");
  assert.ok(renderStart >= 0, "missing renderNetworkPanel source");
  const renderSource = source.slice(renderStart);
  const settingsPanel = extractBetween(
    renderSource,
    'data-node-inner-panel="settings"',
    'data-node-inner-panel="log"',
  );
  const logPanel = extractBetween(renderSource, 'data-node-inner-panel="log"', "</div>`;");

  assertIncludes(settingsPanel, 'data-node-action="start"', "Settings must own Start");
  assertIncludes(settingsPanel, 'data-node-action="stop"', "Settings must own Stop");
  assertIncludes(settingsPanel, "data-node-network-enabled", "Settings must own network enable");
  assertIncludes(settingsPanel, "kgw-network-policy", "Settings must own network policy");
  assertIncludes(settingsPanel, "runtimeError", "Settings must expose runtime errors");
  assertIncludes(settingsPanel, "runtimeStatus", "Settings must expose runtime status");

  assert.ok(!logPanel.includes('data-node-action="start"'), "Live Node Monitor must not contain Start");
  assert.ok(!logPanel.includes('data-node-action="stop"'), "Live Node Monitor must not contain Stop");
  assert.ok(!logPanel.includes("data-node-network-enabled"), "Live Node Monitor must not contain network enable");
  assert.ok(!logPanel.includes("kgw-network-policy"), "Live Node Monitor must not contain network policy");
  assertIncludes(logPanel, 'data-node-action="copy-log"', "Live Node Monitor must contain Copy Log");
  assertIncludes(logPanel, 'data-node-action="clear-log"', "Live Node Monitor must contain Clear Log");
  assertIncludes(logPanel, "node-v6-log-metadata", "Live Node Monitor must contain stream/source metadata");

  const startMatches = source.match(/<button[^>]+data-node-action="start"/g) || [];
  const stopMatches = source.match(/<button[^>]+data-node-action="stop"/g) || [];
  assert.strictEqual(startMatches.length, 1, "Start control markup must not be duplicated");
  assert.strictEqual(stopMatches.length, 1, "Stop control markup must not be duplicated");
  assert.ok(!/<button[^>]+\s+id\s*=[^>]+data-node-action="start"/.test(source), "Start control must not use duplicate generated IDs");
  assert.ok(!/<button[^>]+data-node-action="start"[^>]+\s+id\s*=/.test(source), "Start control must not use duplicate generated IDs");
  assert.ok(!/<button[^>]+\s+id\s*=[^>]+data-node-action="stop"/.test(source), "Stop control must not use duplicate generated IDs");
  assert.ok(!/<button[^>]+data-node-action="stop"[^>]+\s+id\s*=/.test(source), "Stop control must not use duplicate generated IDs");

  assert.ok(!/appendLog\([^)]*initialized/i.test(source), "Synthetic initialized text must not be inserted into raw logs");
  assert.ok(!/appendLog\([^)]*node settings saved/i.test(source), "Settings success text must not be inserted into raw logs");
  assert.ok(!/appendLog\([^)]*node .* response/i.test(source), "Synthetic start response text must not be inserted into raw logs");
  assert.ok(
    source.includes("const KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS = 110000"),
    "Node Start must wait through the backend readiness window",
  );
  assert.ok(
    /function kgwNodeR51IsRunning[\s\S]*readiness=READY/.test(source),
    "Node status polling must require READY before displaying Running",
  );
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

  toString() {
    return Array.from(this.items).join(" ");
  }
}

function dataKey(attr) {
  return attr
    .slice("data-".length)
    .replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
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
  }

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }

  append(...nodes) {
    for (const node of nodes) this.appendChild(node);
  }

  remove() {
    if (!this.parentElement) return;
    this.parentElement.children = this.parentElement.children.filter((child) => child !== this);
    this.parentElement = null;
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

  addEventListener(type, handler, options = false) {
    const list = this.listeners.get(type) || [];
    list.push({ handler, capture: options === true || Boolean(options && options.capture) });
    this.listeners.set(type, list);
  }

  dispatchEvent(event) {
    event.target = event.target || this;
    event.currentTarget = null;
    event.defaultPrevented = false;
    event.cancelBubble = false;
    event.immediateStopped = false;
    event.preventDefault = () => { event.defaultPrevented = true; };
    event.stopPropagation = () => { event.cancelBubble = true; };
    event.stopImmediatePropagation = () => {
      event.cancelBubble = true;
      event.immediateStopped = true;
    };

    const path = [];
    let current = this;
    while (current) {
      path.unshift(current);
      current = current.parentElement;
    }

    for (const node of path) {
      const listeners = (node.listeners.get(event.type) || []).filter((item) => item.capture);
      for (const listener of listeners) {
        event.currentTarget = node;
        listener.handler(event);
        if (event.immediateStopped) return !event.defaultPrevented;
      }
      if (event.cancelBubble) return !event.defaultPrevented;
    }

    for (const node of path.reverse()) {
      const listeners = (node.listeners.get(event.type) || []).filter((item) => !item.capture);
      for (const listener of listeners) {
        event.currentTarget = node;
        listener.handler(event);
        if (event.immediateStopped) return !event.defaultPrevented;
      }
      if (event.cancelBubble) return !event.defaultPrevented;
    }

    return !event.defaultPrevented;
  }

  click() {
    if (this.disabled) return false;
    return this.dispatchEvent({ type: "click", isTrusted: true });
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

  set innerHTML(value) {
    this.children = [];
    parseHtmlInto(this, String(value || ""));
  }

  get innerHTML() {
    return "";
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

function parseHtmlInto(parent, html) {
  const stack = [parent];
  const tokens = html.match(/<\/?[^>]+>|[^<]+/g) || [];
  const voidTags = new Set(["INPUT", "BR", "HR", "IMG", "META", "LINK"]);

  for (const token of tokens) {
    if (token.startsWith("</")) {
      if (stack.length > 1) stack.pop();
      continue;
    }

    if (token.startsWith("<")) {
      const tagMatch = token.match(/^<\s*([A-Za-z0-9-]+)/);
      if (!tagMatch) continue;
      const element = parent.ownerDocument.createElement(tagMatch[1]);
      const attrText = token.replace(/^<\s*[A-Za-z0-9-]+/, "").replace(/\/?\s*>$/, "");
      const attrRegex = /([:@A-Za-z0-9_-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+)))?/g;
      let attrMatch;
      while ((attrMatch = attrRegex.exec(attrText))) {
        const name = attrMatch[1];
        const value = attrMatch[2] ?? attrMatch[3] ?? attrMatch[4] ?? "";
        element.setAttribute(name, value);
      }
      stack[stack.length - 1].appendChild(element);
      if (!voidTags.has(element.tagName) && !token.endsWith("/>")) stack.push(element);
      continue;
    }

    const text = token.replace(/\s+/g, " ").trim();
    if (text) {
      const current = stack[stack.length - 1];
      current.textContent = (current.textContent + " " + text).trim();
    }
  }
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

function createHarness(options = {}) {
  const document = new TestDocument();
  const storage = new Map();
  const timers = [];
  const window = {
    document,
    listeners: new Map(),
    localStorage: {
      getItem(key) { return storage.has(String(key)) ? storage.get(String(key)) : null; },
      setItem(key, value) { storage.set(String(key), String(value)); },
      removeItem(key) { storage.delete(String(key)); },
    },
    setTimeout(callback) {
      timers.push(callback);
      return timers.length;
    },
    clearTimeout() {},
    setInterval() { return 1; },
    clearInterval() {},
    queueMicrotask(callback) { Promise.resolve().then(callback); },
    confirm() { return false; },
    navigator: { clipboard: { writeText: async () => {} } },
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
  if (options.tauri) {
    window.__TAURI__ = options.tauri;
  }
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

  const root = document.createElement("section");
  root.setAttribute("id", "kaspa-node");
  root.setAttribute("class", "page node-v6-root");
  root.setAttribute("data-kgw-tab", "kaspa-node");
  const shell = document.createElement("div");
  shell.setAttribute("class", "node-v6-shell");
  const tabs = document.createElement("div");
  tabs.setAttribute("class", "node-v6-network-tabs");
  for (const [net, label, active] of [
    ["mainnet", "Mainnet", true],
    ["testnet10", "Testnet 10", false],
    ["testnet12", "Testnet 12 · Experimental", false],
  ]) {
    const button = document.createElement("button");
    button.setAttribute("type", "button");
    button.setAttribute("class", "node-v6-network-tab" + (active ? " active" : ""));
    button.setAttribute("data-node-network-tab", net);
    button.setAttribute("data-net", net);
    button.textContent = label;
    tabs.appendChild(button);
  }
  const panels = document.createElement("div");
  panels.setAttribute("id", "nodeNetworkPanels");
  panels.setAttribute("class", "node-v6-network-panels");
  shell.appendChild(tabs);
  shell.appendChild(panels);
  root.appendChild(shell);
  document.body.appendChild(root);

  const sandbox = {
    window,
    document,
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

  const executable = source
    .replace(/export\s+async\s+function\s+initKaspaNodeTab/, "async function initKaspaNodeTab")
    .replace(/export\s+default\s+initKaspaNodeTab\s*;/, "")
    + "\nwindow.__kgwStartButtonTest = { initKaspaNodeTab, getTauriInvoke, kgwResolvePublicTauriInvokeR1, kgwStartTraceTauriShapeR1 };\n";
  vm.runInNewContext(executable, sandbox, { filename: nodeJsPath });

  return { window, document, root };
}

async function flush() {
  await Promise.resolve();
  await Promise.resolve();
  await new Promise((resolve) => setImmediate(resolve));
  await new Promise((resolve) => setImmediate(resolve));
  await new Promise((resolve) => setTimeout(resolve, 25));
}

function startCalls(calls) {
  return calls.filter((call) => call.command === "kgw_kgw_apply_node_settings_v1");
}

function traceCalls(calls) {
  return calls.filter((call) => call.command === "kgw_start_trace_frontend_v1");
}

function traceStages(calls) {
  return traceCalls(calls).map((call) => call.payload.stage);
}

function copyCalls(calls) {
  return calls.filter((call) => call.command === "kgw_copy_text_to_clipboard_v1");
}

function parsedTraceDetails(call) {
  return JSON.parse(call.payload.details || "{}");
}

async function dynamicClickTests() {
  const { window, document, root } = createHarness();
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();

  const mainnetLogPanel = root.querySelector('[data-node-network-panel="mainnet"] [data-node-inner-panel="log"]');
  const mainnetSettingsPanel = root.querySelector('[data-node-network-panel="mainnet"] [data-node-inner-panel="settings"]');
  assert.ok(mainnetLogPanel, "mainnet log panel must render");
  assert.ok(mainnetSettingsPanel, "mainnet settings panel must render");
  assert.strictEqual(mainnetLogPanel.querySelectorAll('[data-node-action="start"]').length, 0, "rendered Live Node Monitor must not contain Start");
  assert.strictEqual(mainnetLogPanel.querySelectorAll('[data-node-action="stop"]').length, 0, "rendered Live Node Monitor must not contain Stop");
  assert.strictEqual(mainnetSettingsPanel.querySelectorAll('[data-node-action="start"]').length, 1, "rendered Settings must contain one Start");
  assert.strictEqual(mainnetSettingsPanel.querySelectorAll('[data-node-action="stop"]').length, 1, "rendered Settings must contain one Stop");

  const calls = [];
  window.__TAURI__ = {
    tauri: {
      invoke: async (command, payload) => {
        calls.push({ command, payload });
        if (command !== "kgw_kgw_apply_node_settings_v1") return "ignored";
        return "parallel-owned-self-worker started;role=node;network=" + payload.network + ";pid=4242;owner=self-worker;runtime_state=running;readiness=READY";
      },
    },
  };

  calls.length = 0;
  root.querySelector('[data-node-action="start"][data-net="mainnet"]').click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 1, "mainnet Start click must invoke exactly once");
  assert.strictEqual(startCalls(calls)[0].payload.network, "mainnet");
  assert.strictEqual(startCalls(calls)[0].payload.nodeKind, "integrated-as-daemon");
  assert.strictEqual(startCalls(calls)[0].payload.bridgeKind, "disable");
  assert.strictEqual(startCalls(calls)[0].payload.runtimeRole, "node");
  assert.ok(traceStages(calls).includes("frontend.capture_click_observed"), "capture observer must trace the physical-style Start click");
  const capture = traceCalls(calls).find((call) => call.payload.stage === "frontend.capture_click_observed");
  assert.strictEqual(capture.payload.network, "mainnet");
  assert.strictEqual(capture.payload.action, "start");
  assert.strictEqual(parsedTraceDetails(capture).belongsToSettings, true, "capture trace must report Settings ownership");

  root.querySelector('[data-node-network-tab="testnet10"]').click();
  calls.length = 0;
  root.querySelector('[data-node-action="start"][data-net="testnet10"]').click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 1, "testnet10 Start click must invoke exactly once after tab switch");
  assert.strictEqual(startCalls(calls)[0].payload.network, "testnet10");

  calls.length = 0;
  const testnet12Start = root.querySelector('[data-node-action="start"][data-net="testnet12"]');
  testnet12Start.click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 0, "testnet12 Start must not invoke while opt-in is disabled");

  window.__TAURI__.tauri.invoke = async (command, payload) => {
    calls.push({ command, payload });
    if (command === "kgw_start_trace_frontend_v1") return true;
    throw new Error("spawn_failed=true;runtime_role=node;network=mainnet;source=self-worker;error=Access is denied.");
  };
  const mainnetStart = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  mainnetStart.disabled = false;
  calls.length = 0;
  mainnetStart.click();
  await flush();
  const errorNode = document.getElementById("node-mainnet-runtimeError");
  const statusNode = document.getElementById("node-mainnet-runtimeStatus");
  const evidenceNode = document.getElementById("node-mainnet-runtimeEvidence");
  assert.strictEqual(startCalls(calls).length, 1, "failed Start click must still invoke exactly once");
  assert.strictEqual(mainnetStart.disabled, false, "failed Start must restore button state");
  assert.ok(traceStages(calls).includes("frontend.invoke_rejected"), "failed Start must trace invoke rejection");
  assert.ok(traceStages(calls).includes("frontend.button_state_restored_after_failure"), "failed Start must trace button restoration");
  assert.ok(
    errorNode.textContent.includes("Access is denied."),
    "failed Start must expose original error, actual text: " + errorNode.textContent + "; status=" + statusNode.textContent + "; evidence=" + evidenceNode.textContent,
  );

  const rawLog = document.getElementById("node-mainnet-logOutput").textContent;
  assert.ok(!/initialized|parallel-owned-self-worker started|KGW node start response/i.test(rawLog), "raw log must not contain synthetic startup success text");
}

function configuredTauriInvokeResolverTests() {
  const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, "utf8"));
  assert.strictEqual(tauriConfig.app.withGlobalTauri, true, "Tauri config must expose the supported global API");

  const calls = [];
  const { window } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          return true;
        },
      },
    },
  });

  const resolved = window.__kgwStartButtonTest.kgwResolvePublicTauriInvokeR1();
  assert.strictEqual(resolved.adapter, "window.__TAURI__.core.invoke", "Tauri 2 global core invoke must win");
  assert.strictEqual(typeof resolved.invoke, "function", "resolver must return a public invoke function");
  assert.strictEqual(resolved.shape.hasCoreInvoke, true, "resolver shape must detect core.invoke");
  assert.strictEqual(resolved.shape.expectedConfiguredGlobal, "window.__TAURI__.core.invoke");
}

async function missingInvokeApiVisibleErrorTest() {
  const { window, document, root } = createHarness();
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();

  const start = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  start.disabled = false;
  start.click();
  await flush();

  const errorNode = document.getElementById("node-mainnet-runtimeError");
  assert.ok(errorNode.textContent.includes("Tauri invoke API is not available"), "missing invoke API must be visible in the UI");
  assert.strictEqual(start.disabled, false, "missing invoke API must restore Start button state");
}

async function tracePayloadSafetyTests() {
  const calls = [];
  const { window, root } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          if (command === "kgw_kgw_apply_node_settings_v1") {
            return "parallel-owned-self-worker started;role=node;network=" + payload.network + ";pid=4242;owner=self-worker;runtime_state=running;readiness=READY";
          }
          return true;
        },
      },
    },
  });

  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();
  calls.length = 0;
  root.querySelector('[data-node-action="start"][data-net="mainnet"]').click();
  await flush();

  assert.strictEqual(startCalls(calls).length, 1, "trace safety run must invoke Start exactly once");
  const serializedTrace = JSON.stringify(traceCalls(calls).map((call) => call.payload));
  assert.ok(!/secret|token|private|mnemonic|wallet/i.test(serializedTrace), "trace payload must exclude secret-like fields");
  assert.ok(!/nodeCommandPreview|bridgeCommandPreview|--rpc|--appdir|--stratum/i.test(serializedTrace), "trace payload must not expose command arguments or preview fields");
}

async function copyLogFrontendTests() {
  const calls = [];
  let copyReject = null;
  let copyPending = false;
  let resolvePendingCopy = null;
  let navigatorWriteCount = 0;
  const { window, document, root } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          if (command === "kgw_start_trace_frontend_v1") return true;
          if (command === "kgw_copy_text_to_clipboard_v1") {
            if (copyReject) throw copyReject;
            if (copyPending) {
              return await new Promise((resolve) => {
                resolvePendingCopy = resolve;
              });
            }
            return "clipboard_write_v1;network=" + payload.network + ";characters=" + payload.characterCount + ";lines=" + payload.lineCount + ";sha256=" + payload.sha256 + ";copied=true";
          }
          if (command === "kgw_runtime_owner_status_v1") return "parallel-owned-self-worker status;role=node;network=" + payload.network + ";running=false";
          if (command === "kgw_kgw_runtime_logs_v1") return "";
          return true;
        },
      },
    },
  });

  window.navigator.clipboard.writeText = async () => {
    navigatorWriteCount += 1;
    throw new Error("browser clipboard must not be used for Copy Log");
  };

  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();
  calls.length = 0;

  const mainnetLog = document.getElementById("node-mainnet-logOutput");
  const mainnetCopy = root.querySelector('[data-node-action="copy-log"][data-net="mainnet"]');
  mainnetLog.textContent = "mainnet raw line 1\nmainnet raw line 2";
  mainnetCopy.click();
  await flush();

  assert.strictEqual(copyCalls(calls).length, 1, "Copy Log must invoke native clipboard exactly once");
  const copied = copyCalls(calls)[0].payload;
  assert.strictEqual(copied.network, "mainnet", "Copy Log must pass the active network");
  assert.strictEqual(copied.text, "mainnet raw line 1\r\nmainnet raw line 2", "Copy Log must preserve multiline raw text order with Windows line endings");
  assert.strictEqual(copied.lineCount, 2, "Copy Log must pass line count metadata");
  assert.strictEqual(copied.characterCount, Array.from(copied.text).length, "Copy Log must pass character count metadata");
  assert.strictEqual(String(copied.sha256 || "").length, 64, "Copy Log should pass SHA-256 metadata when Web Crypto is available");
  assert.strictEqual(navigatorWriteCount, 0, "Copy Log must not use navigator.clipboard as the primary path");
  assert.ok(traceStages(calls).includes("frontend.copy_log_click_observed"), "Copy Log physical-style click must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_network_resolved"), "Copy Log network resolution must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_content_prepared"), "Copy Log content metadata must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_dispatched"), "Copy Log native dispatch must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_succeeded"), "Copy Log success feedback must be traced after native success");
  assert.ok(!JSON.stringify(traceCalls(calls).map((call) => call.payload)).includes("mainnet raw line"), "Copy Log trace must exclude raw clipboard content");
  assert.strictEqual(mainnetLog.textContent, "mainnet raw line 1\nmainnet raw line 2", "Copy Log must not insert synthetic content into raw logs");

  calls.length = 0;
  copyReject = new Error("native clipboard failure");
  mainnetCopy.textContent = "Copy Log";
  mainnetCopy.disabled = false;
  mainnetCopy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 1, "Clipboard failure must still call native clipboard once");
  assert.ok(traceStages(calls).includes("frontend.copy_log_failed"), "Clipboard failure must be traced");
  assert.strictEqual(mainnetCopy.disabled, false, "Clipboard failure must restore the button state");
  assert.ok(/copy failed/i.test(mainnetCopy.textContent), "Clipboard failure must not display Copied feedback");
  assert.ok(root.querySelector('.kgw-copy-log-status-v1[data-net="mainnet"]').textContent.includes("native clipboard failure"), "Clipboard failure must be visible");
  copyReject = null;

  calls.length = 0;
  mainnetLog.textContent = "MAINNET log is empty.";
  mainnetCopy.textContent = "Copy Log";
  mainnetCopy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 0, "Empty or placeholder logs must not invoke native clipboard");
  assert.ok(traceStages(calls).includes("frontend.copy_log_failed"), "Empty Copy Log rejection must be traced");

  calls.length = 0;
  mainnetLog.textContent = "duplicate guard line";
  copyPending = true;
  resolvePendingCopy = null;
  mainnetCopy.disabled = false;
  mainnetCopy.textContent = "Copy Log";
  mainnetCopy.click();
  mainnetCopy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 1, "Duplicate Copy Log clicks must not invoke native clipboard twice");
  assert.strictEqual(typeof resolvePendingCopy, "function", "pending Copy Log test must hold the native clipboard promise");
  resolvePendingCopy("clipboard_write_v1;network=mainnet;characters=20;lines=1;copied=true");
  copyPending = false;
  await flush();

  const testnet10Tab = root.querySelector('[data-node-network-tab="testnet10"]');
  testnet10Tab.click();
  await flush();
  const testnet10Log = document.getElementById("node-testnet10-logOutput");
  const testnet10Copy = root.querySelector('[data-node-action="copy-log"][data-net="testnet10"]');
  mainnetLog.textContent = "mainnet must not be copied";
  testnet10Log.textContent = "testnet10 raw line 1\ntestnet10 raw line 2";
  calls.length = 0;
  testnet10Copy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 1, "testnet10 Copy Log must invoke native clipboard once");
  assert.strictEqual(copyCalls(calls)[0].payload.network, "testnet10", "Copy Log must pass testnet10 when testnet10 is active");
  assert.ok(copyCalls(calls)[0].payload.text.includes("testnet10 raw line 1"), "Copy Log must copy the active network buffer");
  assert.ok(!copyCalls(calls)[0].payload.text.includes("mainnet must not be copied"), "Copy Log must not mix mainnet into testnet10");

  calls.length = 0;
  testnet10Log.textContent = "testnet10 clear target";
  mainnetLog.textContent = "mainnet should remain after clear";
  root.querySelector('[data-node-action="clear-log"][data-net="testnet10"]').click();
  await flush();
  assert.strictEqual(testnet10Log.textContent, "", "Clear Log must affect the active network log");
  assert.strictEqual(mainnetLog.textContent, "mainnet should remain after clear", "Clear Log must not clear another network log");

  const largeLines = Array.from({ length: 1600 }, (_, index) => "large raw line " + index);
  testnet10Log.textContent = largeLines.join("\n");
  calls.length = 0;
  testnet10Copy.textContent = "Copy Log";
  testnet10Copy.click();
  await flush();
  const largeCopy = copyCalls(calls)[0].payload.text;
  assert.strictEqual(largeCopy, largeLines.join("\r\n"), "Large Copy Log text must not be silently truncated or reordered");
}

(async () => {
  try {
    staticPlacementTests();
    configuredTauriInvokeResolverTests();
    await dynamicClickTests();
    await missingInvokeApiVisibleErrorTest();
    await tracePayloadSafetyTests();
    await copyLogFrontendTests();
    console.log("KGW start button and Copy Log frontend tests PASSED");
  } catch (error) {
    fail(error && error.stack ? error.stack : String(error));
  }
})();
