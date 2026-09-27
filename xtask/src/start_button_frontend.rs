use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_BRIDGE: &str = r########"#!/usr/bin/env node
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
    source.includes("const KGW_NODE_STOP_INVOKE_TIMEOUT_MS = 0"),
    "Node Stop must defer terminality to the async backend without a JS wall-clock cutoff",
  );
  assert.ok(source.includes('action === "start" ? "Starting" : "Stopping"'), "Stop click must enter Stopping before backend completion");
  assert.ok(source.includes('evidence.fields.running === "false"'), "Stopped requires terminal backend liveness evidence");
  assert.ok(source.includes("Stop required FORCED termination."), "forced Stop must be visible outside raw logs");
  assert.ok(!/appendLog\([^\n]*(FORCED|graceful|stop_outcome|Stopping)/i.test(source), "Stop control diagnostics must remain outside raw Node logs");
  assert.ok(
    source.includes("nodeRuntimeIsRunning as wasmNodeRuntimeIsRunning") && /function kgwNodeR51IsRunning\(text\)\s*\{\s*return wasmNodeRuntimeIsRunning\(text\);\s*\}/.test(source),
    "Node status polling must delegate READY/running classification to the Rust/WASM owner",
  );
  assert.ok(
    source.includes("nodeRuntimeErrorFromStatus as wasmNodeRuntimeErrorFromStatus") && source.includes("kgwNodeRuntimeErrorFromStatus(status)"),
    "Node status polling must surface typed post-READY runtime failures through the Rust/WASM owner",
  );
  assert.ok(
    source.includes('kgwNodeTranslateRuntimeV29("runtime.failed", "Failed")'),
    "Node post-READY failure must remain visible outside raw logs",
  );
  assert.ok(
    source.includes("nodeCommandInlineState as wasmNodeCommandInlineState")
      && source.includes("nodeCommandOptionEnabled as wasmNodeCommandOptionEnabled")
      && source.includes("nodeCommandInlineToggle as wasmNodeCommandInlineToggle")
      && source.includes("nodeRefreshInlineCommandToggles as wasmNodeRefreshInlineCommandToggles")
      && source.includes("nodeToggleCommandOption as wasmNodeToggleCommandOption"),
    "Node command-composer state/policy/toggle ownership must be delegated to Rust/WASM",
  );
  assert.ok(
    !source.includes("window.__kgwNodeCommandComposerInlineR7 = window.__kgwNodeCommandComposerInlineR7 || {}"),
    "Node command-composer state initialization must not remain implemented in hand-maintained JS",
  );
  assert.ok(
    !source.includes("NODE_ENDPOINTS.some(row => row[1] === name || row[2] === name)"),
    "Node command-composer schema policy must not remain implemented in hand-maintained JS",
  );
  assert.ok(
    source.includes("nodeR51Keys as wasmNodeR51Keys")
      && source.includes("nodeR51Panel as wasmNodeR51Panel")
      && source.includes("nodeR51Fields as wasmNodeR51Fields")
      && source.includes("nodeR51ReadSettings as wasmNodeR51ReadSettings")
      && source.includes("nodeR51WriteSettings as wasmNodeR51WriteSettings")
      && source.includes("nodeR51Store as wasmNodeR51Store")
      && source.includes("nodeR51Load as wasmNodeR51Load")
      && source.includes("nodeR51CaptureFactoryDefaults as wasmNodeR51CaptureFactoryDefaults")
      && source.includes("nodeR51LoadSavedSettings as wasmNodeR51LoadSavedSettings"),
    "Node R51 persistence core must be delegated to the Rust/WASM owner",
  );
  assert.ok(
    source.includes("const KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C = wasmNodeCommandOptionsKey();")
      && /function kgwNodeR51ReadSettings\(net\)\s*\{\s*const values = wasmNodeR51ReadSettings/.test(source)
      && /function kgwNodeR51WriteSettings\(net, values\)[\s\S]*?wasmNodeR51WriteSettings/.test(source)
      && source.includes('return wasmNodeR51Store(String(key || ""), value);')
      && source.includes('return wasmNodeR51Load(String(key || ""));'),
    "Node R51 wrappers must remain thin Rust/WASM adapters with JS-only trace/update glue",
  );
  assert.ok(
    !source.includes("const commandOptions = values && values[KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C];")
      && !source.includes("state[String(name)] = Boolean(enabled) && (!NODE_OPTIONAL.has(name)")
      && !source.includes('const KGW_NODE_R51_STORAGE_PREFIX = "kgw.node.direct.v51.";')
      && !source.includes("localStorage.setItem(KGW_NODE_R51_STORAGE_PREFIX"),
    "Node R51 persistence implementation must not return to hand-maintained JS",
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
    if (this.tagName === "TEMPLATE") this.content = this;
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

  get outerHTML() {
    const tag = String(this.tagName || "div").toLowerCase();
    const attrs = Object.entries(this.attributes)
      .map(([name, value]) => value === "" ? name : name + '="' + String(value).replace(/"/g, "&quot;") + '"')
      .join(" ");
    const opening = "<" + tag + (attrs ? " " + attrs : "") + ">";
    if (["input", "br", "hr", "img", "meta", "link"].includes(tag)) return opening;
    const body = (this.textContent || "") + this.children.map(child => child.outerHTML || "").join("");
    return opening + body + "</" + tag + ">";
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
    ["testnet13", "Testnet 13 آ· Experimental", false],
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

  const importPrelude = `
const applyStatusTone = (node, state) => { if (node) node.dataset.state = String(state || "").toLowerCase(); };
const renderStatusSummary = (node, text) => { if (node) node.textContent = String(text || ""); };
const NODE_ENDPOINTS = [["rpcListenEnabled","rpcListenHost","rpcListenPort","rpcListen",true],["rpcBorshEnabled","rpcBorshHost","rpcBorshPort","rpcListenBorsh",true],["rpcJsonEnabled","rpcJsonHost","rpcJsonPort","rpcListenJson",true],["listenEnabled","listenHost","listenPort","p2pListen",false],["externalIpEnabled","externalIpHost","externalIpPort","externalIp",false],["connectEnabled","connectHost","connectPort","connectPeers",false],["addPeerEnabled","addPeerHost","addPeerPort","addPeers",false]];
const NODE_MANAGED = { appDir:"Managed", configFile:"Unsupported", overrideParamsFile:"Unsupported", testnet:"Managed", netsuffix:"Managed", noGrpc:"Managed", rpcListenEnabled:"Managed" };
const NODE_REQUIRED = { logLevel:"info", asyncThreads:"16", ramScale:"1", rpcMaxClients:"16", outPeers:"8", maxInPeers:"32" };
const NODE_OPTIONAL = new Set(["uaComment","retentionDays","maxTrackedAddresses","perfMetricsInterval","rocksDbPreset","rocksDbCacheSize","rocksDbWalDir","logDir"]);
const NODE_DANGEROUS = { resetDb:"Deletes database", unsafeRpc:"Unsafe RPC", enableUnsyncedMining:"Unsynced mining" };
const nodeFieldEnabled = () => true;
const validateNodeForm = () => ({});
const renderFieldErrors = () => {};
const endpoint = (host, port) => String(host) + ":" + String(port);
const runtimePresentation = ({enabled, running, transition, error}) => ({ process: error ? "Failed" : transition === "starting" ? "Starting" : transition === "stopping" ? "Stopping" : running ? "Running" : "Stopped", processLabel: error ? "Node: Failed" : transition === "starting" ? "Node: Starting" : transition === "stopping" ? "Node: Stopping" : running ? "Node: Running" : "Node: Stopped", profile: enabled ? "Enabled" : "Disabled" });
const runtimeObservationSummary = () => "RPC/synchronization/mining: unknown";
const confirmUserAction = async () => true;
const renderSettingsTabs = (_scope, _key, groups) => groups.map(group => group[3]).join("");
const installSettingsLayout = () => {};
const decorateSettingsFields = () => {};
const revealSettingsField = () => {};
const setSettingFieldState = () => {};
const wasmNodeI18nText = (_key, fallback) => String(fallback ?? "");
const wasmNodeBackendInvoke = (command, payload = {}) => {
  const tauri = window.__TAURI__;
  const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
    ? tauri.core.invoke
    : tauri && tauri.tauri && typeof tauri.tauri.invoke === "function"
      ? tauri.tauri.invoke
      : window.__TAURI_INVOKE__;
  if (typeof invoke !== "function") return Promise.reject(new Error("Tauri invoke is not available"));
  return invoke(command, payload);
};
const wasmNodeNetworkProfiles = () => [
  { key: "mainnet", label: "Mainnet", testnet: false, netsuffix: "", enabledByDefault: true, runtime: "Official Rusty Kaspa" },
  { key: "testnet10", label: "Testnet 10", testnet: true, netsuffix: "10", enabledByDefault: true, runtime: "Official Rusty Kaspa" },
  { key: "testnet13", label: "Testnet 13", testnet: true, netsuffix: "13", enabledByDefault: false, experimental: true, runtime: "DAGKnight - Experimental" },
];
const wasmNodeNetworkPolicyKey = (net) => "kgw.node.network.enabled." + String(net || "unknown");
const wasmNodeNetworkProfile = (net) => wasmNodeNetworkProfiles().find((item) => item.key === String(net || "")) || null;
const wasmNodeNetworkEnabled = (net) => {
  const stored = localStorage.getItem(wasmNodeNetworkPolicyKey(net));
  if (stored === "1") return true;
  if (stored === "0") return false;
  const profile = wasmNodeNetworkProfile(net);
  return profile ? profile.enabledByDefault !== false : false;
};
const wasmNodeSetNetworkEnabled = (net, enabled) => localStorage.setItem(wasmNodeNetworkPolicyKey(net), enabled ? "1" : "0");
const wasmNodeNetworkPolicyMessage = (net) => {
  const profile = wasmNodeNetworkProfile(net);
  if (!profile) return "";
  if (profile.experimental) {
    return "Experimental network. Disabled by default and requires explicit opt-in."
      + (String(net) === "testnet13" ? " This Testnet13 build has no DNS seeders. For public sync, set a trusted Testnet13 peer in Connect or Add Peer." : "");
  }
  return String(profile.runtime) + ". RPC remains loopback-only and data is isolated per network.";
};
const wasmNodeById = (id) => document.getElementById(String(id || ""));
const wasmNodeEscapeHtml = (value) => String(value ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");
const wasmNodeElementId = (net, name) => "node-" + String(net || "") + "-" + String(name || "");
const wasmNodeValue = (net, name) => {
  const element = wasmNodeById(wasmNodeElementId(net, name));
  return element ? String(element.value || "").trim() : "";
};
const wasmNodeChecked = (net, name) => {
  const element = wasmNodeById(wasmNodeElementId(net, name));
  return Boolean(element && element.checked);
};
const wasmNodeCommandInlineStateKey = (net) => String(net || "mainnet");
const wasmNodeCommandInlineState = (net) => {
  const key = wasmNodeCommandInlineStateKey(net);
  window.__kgwNodeCommandComposerInlineR7 = window.__kgwNodeCommandComposerInlineR7 || {};
  window.__kgwNodeCommandComposerInlineR7[key] = window.__kgwNodeCommandComposerInlineR7[key] || {};
  return window.__kgwNodeCommandComposerInlineR7[key];
};
const wasmNodeCommandOptionEnabled = (net, name) => {
  if (Object.hasOwn(NODE_REQUIRED, name)) return true;
  const state = wasmNodeCommandInlineState(net);
  if (NODE_OPTIONAL.has(name)) return state[String(name)] === true;
  return state[String(name)] !== false;
};
const wasmNodeCommandShouldInclude = (net, name) => wasmNodeCommandOptionEnabled(net, name);
const wasmNodeCommandInlineToggle = (net, name) => {
  if (NODE_MANAGED[name] || Object.hasOwn(NODE_REQUIRED, name)
      || NODE_ENDPOINTS.some((row) => row[1] === name || row[2] === name)) return "";
  const enabled = wasmNodeCommandOptionEnabled(net, name);
  return '<input type="checkbox" class="kgw-command-option-checkbox-r9" data-node-command-option-toggle-r7="'
    + wasmNodeEscapeHtml(name) + '" data-net="' + wasmNodeEscapeHtml(net) + '" '
    + (enabled ? "checked" : "") + ' aria-label="Use ' + wasmNodeEscapeHtml(name)
    + '" title="Enable this optional setting">';
};
const wasmNodeRefreshInlineCommandToggles = (net) => {
  for (const el of document.querySelectorAll("[data-node-command-option-toggle-r7]")) {
    if (String(el.dataset.net || "") !== String(net || "")) continue;
    const name = String(el.dataset.nodeCommandOptionToggleR7 || "");
    const enabled = wasmNodeCommandOptionEnabled(net, name);
    el.checked = enabled;
    el.setAttribute("aria-label", enabled ? "Included in command" : "Excluded from command");
    el.setAttribute("title", enabled ? "Included in command" : "Excluded from command");
    el.classList.toggle("is-on", enabled);
    el.classList.toggle("is-off", !enabled);
  }
};
const wasmNodeToggleCommandOption = (net, name) => {
  const state = wasmNodeCommandInlineState(net);
  const key = String(name || "");
  state[key] = state[key] === false;
  wasmNodeRefreshInlineCommandToggles(net);
  return state[key];
};
const wasmNodeCommandOptionsKey = () => "__kgwNodeCommandOptionsR38C";
const wasmNodeReadCommandOptions = (net) => {
  const state = {};
  const root = document.getElementById("kaspa-node");
  if (!root) return state;
  for (const item of root.querySelectorAll("[data-node-command-option-toggle-r7]")) {
    if (String(item.dataset.net || "") !== String(net || "")) continue;
    const name = String(item.dataset.nodeCommandOptionToggleR7 || "");
    if (name) state[name] = Boolean(item.checked);
  }
  return state;
};
const wasmNodeApplyCommandOptions = (net, values) => {
  const result = { applied: false, count: 0 };
  const commandOptions = values && values[wasmNodeCommandOptionsKey()];
  if (!commandOptions || typeof commandOptions !== "object") return result;
  const state = wasmNodeCommandInlineState(net);
  for (const [name, enabled] of Object.entries(commandOptions)) {
    state[String(name)] = Boolean(enabled)
      && (!NODE_OPTIONAL.has(name) || Boolean(wasmNodeValue(net, name)));
    result.count += 1;
  }
  wasmNodeRefreshInlineCommandToggles(net);
  result.applied = true;
  return result;
};
const wasmNodeR51Keys = () => wasmNodeNetworkProfiles().map((item) => item.key);
const wasmNodeR51Panel = (net) =>
  document.querySelector('[data-node-network-panel="' + String(net || "") + '"]');
const wasmNodeR51Fields = (net) => {
  const panel = wasmNodeR51Panel(net);
  if (!panel) return [];
  const prefix = "node-" + String(net || "") + "-";
  return Array.from(panel.querySelectorAll("input, select, textarea")).filter((field) => {
    if (!field.id || !field.id.startsWith(prefix)) return false;
    if (field.id.endsWith("-commandPreview") || field.id.endsWith("-logOutput")) return false;
    return !field.closest(".node-v6-log-toolbar");
  });
};
const wasmNodeR51ReadSettings = (net) => {
  const values = {};
  values[wasmNodeCommandOptionsKey()] = wasmNodeReadCommandOptions(net);
  const prefix = "node-" + String(net || "") + "-";
  for (const field of wasmNodeR51Fields(net)) {
    const name = String(field.id || "").slice(prefix.length);
    if (!field.id || NODE_MANAGED[name]) continue;
    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }
  return values;
};
const wasmNodeR51WriteSettings = (net, values) => {
  const result = { applied: false, commandOptionsApplied: false, commandOptionsCount: 0 };
  if (!values || typeof values !== "object") return result;
  const prefix = "node-" + String(net || "") + "-";
  for (const field of wasmNodeR51Fields(net)) {
    const name = String(field.id || "").slice(prefix.length);
    if (!field.id || NODE_MANAGED[name]) continue;
    const item = values[field.id];
    if (!item) continue;
    if (field.type === "checkbox") field.checked = Boolean(item.checked);
    else if (Object.hasOwn(item, "value")) field.value = String(item.value ?? "");
    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
  }
  const command = wasmNodeApplyCommandOptions(net, values);
  result.commandOptionsApplied = Boolean(command && command.applied);
  result.commandOptionsCount = Number(command && command.count || 0);
  result.applied = true;
  return result;
};
const wasmNodeR51StoragePrefix = "kgw.node.direct.v51.";
const wasmNodeR51Store = (key, value) =>
  localStorage.setItem(wasmNodeR51StoragePrefix + String(key || ""), JSON.stringify(value));
const wasmNodeR51Load = (key) => {
  try { return JSON.parse(localStorage.getItem(wasmNodeR51StoragePrefix + String(key || "")) || "null"); }
  catch { return null; }
};
const wasmNodeR51CaptureFactoryDefaults = () => {
  for (const net of wasmNodeR51Keys()) {
    if (!wasmNodeR51Load("factory:" + net)) {
      wasmNodeR51Store("factory:" + net, wasmNodeR51ReadSettings(net));
    }
  }
};
const wasmNodeR51LoadSavedSettings = () => {
  const applied = [];
  for (const net of wasmNodeR51Keys()) {
    const saved = wasmNodeR51Load("saved:" + net);
    if (!saved) continue;
    const result = wasmNodeR51WriteSettings(net, saved);
    applied.push({
      net,
      commandOptionsApplied: Boolean(result.commandOptionsApplied),
      commandOptionsCount: Number(result.commandOptionsCount || 0),
    });
  }
  return applied;
};
const wasmNodeCardInput = (net, name, label, value = "", placeholder = "", span2 = false, toggle = "") =>
  '\\n    <div class="node-v6-card' + (span2 ? ' span2' : '') + '">'
  + '\\n      <span class="kgw-command-option-title-row-r8e">'
  + '\\n        ' + toggle
  + '\\n        <label for="' + wasmNodeElementId(net, name) + '" class="kgw-command-option-title-text-r8e">' + wasmNodeEscapeHtml(label) + '</label>'
  + '\\n      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->'
  + '\\n      <input id="' + wasmNodeElementId(net, name) + '" data-testid="kgw-node-field-' + wasmNodeEscapeHtml(net) + '-' + wasmNodeEscapeHtml(name) + '" type="text" value="' + wasmNodeEscapeHtml(value) + '" placeholder="' + wasmNodeEscapeHtml(placeholder) + '">'
  + '\\n    </div>';
const wasmNodeCardSelect = (net, name, label, options, value = "", span2 = false, toggle = "") => {
  const opts = Array.from(options || [], (item) => String(item ?? "")).map((item) => {
    const selected = item === String(value ?? "") ? ' selected' : '';
    return '<option value="' + wasmNodeEscapeHtml(item) + '"' + selected + '>' + wasmNodeEscapeHtml(item || "not set") + '</option>';
  }).join('');
  return '\\n    <div class="node-v6-card' + (span2 ? ' span2' : '') + '">'
    + '\\n      <span class="kgw-command-option-title-row-r8e">'
    + '\\n        ' + toggle
    + '\\n        <label for="' + wasmNodeElementId(net, name) + '" class="kgw-command-option-title-text-r8e">' + wasmNodeEscapeHtml(label) + '</label>'
    + '\\n      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->'
    + '\\n      <select id="' + wasmNodeElementId(net, name) + '" data-testid="kgw-node-field-' + wasmNodeEscapeHtml(net) + '-' + wasmNodeEscapeHtml(name) + '">' + opts + '</select>'
    + '\\n    </div>';
};
const wasmNodeCardCheck = (net, name, label, checked = false, span2 = false) =>
  '\\n    <label class="node-v6-card check' + (span2 ? ' span2' : '') + '">'
  + '\\n      <input id="' + wasmNodeElementId(net, name) + '" data-testid="kgw-node-field-' + wasmNodeEscapeHtml(net) + '-' + wasmNodeEscapeHtml(name) + '" type="checkbox"' + (checked ? ' checked' : '') + '>'
  + '\\n      <span>' + wasmNodeEscapeHtml(label) + '</span>'
  + '\\n    </label>';
const wasmNodeNormalizeInnerTab = (value) => value === "settings" || value === "log" ? value : "log";
const wasmNodeResolveInnerTab = (net) => {
  try { return wasmNodeNormalizeInnerTab(localStorage.getItem("kgw.node.innerTab." + String(net || "unknown"))); }
  catch { return "log"; }
};
const wasmNodeSaveInnerTab = (net, selected) => {
  const normalized = wasmNodeNormalizeInnerTab(selected);
  try { localStorage.setItem("kgw.node.innerTab." + String(net || "unknown"), normalized); } catch {}
  return normalized;
};
const wasmNodeNormalizeNetwork = (value) => {
  const normalized = String(value ?? "").trim();
  return ["mainnet", "testnet10", "testnet13"].includes(normalized) ? normalized : "";
};
const wasmNodeReadLastNetwork = () => {
  try { return wasmNodeNormalizeNetwork(localStorage.getItem("kgw.node.lastNetwork")); } catch { return ""; }
};
const wasmNodeSaveLastNetwork = (net) => {
  const normalized = wasmNodeNormalizeNetwork(net);
  if (!normalized) return "";
  try { localStorage.setItem("kgw.node.lastNetwork", normalized); } catch {}
  return normalized;
};
const wasmNodeStringifyRuntimeResult = (result) => {
  if (result == null) return "No response";
  if (typeof result === "string") return result;
  try { return JSON.stringify(result); } catch { return String(result); }
};
const wasmNodeNormalizeRuntimeError = (error) => {
  if (error == null) return "Unknown backend error";
  if (typeof error === "string") return error;
  if (error.message) return error.message;
  try { return JSON.stringify(error); } catch { return String(error); }
};
const wasmNodeParseRuntimeFields = (result) => {
  const raw = wasmNodeStringifyRuntimeResult(result);
  const fields = {};
  for (const part of raw.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;
    const key = part.slice(0, index).trim();
    const value = part.slice(index + 1).trim();
    if (key) fields[key] = value;
  }
  return fields;
};
const wasmNodeRuntimeIsRunning = (text) => {
  const value = String(text || "");
  return /readiness=READY/i.test(value)
    && (/running=true/.test(value) || /node_running=true/.test(value) || /official_core_running=true/.test(value));
};
const wasmNodeRuntimeErrorFromStatus = (text) => {
  const fields = wasmNodeParseRuntimeFields(text);
  const error = String(fields.runtime_error || fields.runtimeError || "").trim();
  return error && error.toLowerCase() !== "none" ? error : "";
};
const wasmNodeLogAutoScrollEnabled = (net) =>
  localStorage.getItem("kgw.node.log.autoscroll." + String(net || "")) !== "0";
const wasmNodeSetLogAutoScroll = (net, enabled) => {
  localStorage.setItem("kgw.node.log.autoscroll." + String(net || ""), enabled ? "1" : "0");
  const out = document.getElementById("node-" + String(net || "") + "-logOutput");
  if (enabled && out) out.scrollTop = out.scrollHeight;
};
const wasmNodeInstallLogAutoScrollControls = () => {
  for (const profile of wasmNodeNetworkProfiles()) {
    const net = profile.key;
    const out = document.getElementById("node-" + net + "-logOutput");
    if (!out) continue;
    const controlId = "node-" + net + "-logAutoScrollR27";
    if (document.getElementById(controlId)) continue;
    const label = document.createElement("label");
    label.className = "kgw-log-autoscroll-toggle";
    label.setAttribute("data-kgw-log-autoscroll", "node");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.id = controlId;
    checkbox.checked = wasmNodeLogAutoScrollEnabled(net);
    checkbox.addEventListener("change", () => wasmNodeSetLogAutoScroll(net, checkbox.checked));
    const span = document.createElement("span");
    span.textContent = "Auto-scroll";
    label.appendChild(checkbox);
    label.appendChild(span);
    const panel = out.closest('[data-node-inner-panel="log"]') || out.parentElement;
    const toolbar = panel && panel.querySelector(".node-v6-log-toolbar");
    if (toolbar) toolbar.appendChild(label);
  }
};
const wasmNodeClipboardCharacterCount = (text) => Array.from(String(text ?? "")).length;
const wasmNodeClipboardLineCount = (text) => {
  const value = String(text ?? "");
  return value ? value.split("\\n").length : 0;
};
const wasmNodeNormalizeClipboardLineEndings = (text) =>
  String(text ?? "").replace(/\\r\\n/g, "\\n").replace(/\\r/g, "\\n").replace(/\\n/g, "\\r\\n");
const wasmNodeClipboardSafeError = (error) => {
  const text = String(error && error.message ? error.message : error || "clipboard write failed")
    .replace(/[\\r\\n\\t]+/g, " ")
    .trim();
  if (/(secret|token|private|mnemonic|wallet|address)/i.test(text)) {
    return "clipboard write failed with a sensitive error";
  }
  return (text || "clipboard write failed").slice(0, 360);
};
const wasmNodeClipboardPlaceholderText = (net) => {
  const key = String(net || "");
  const labels = { mainnet: "Mainnet", testnet10: "Testnet 10", testnet13: "Testnet 13" };
  return String(labels[key] || key) + " log is empty.";
};
const wasmNodeCopyLogStatus = (net, message, state = "info") => {
  const out = document.getElementById("node-" + net + "-logOutput");
  const panel = out && out.closest && out.closest('[data-node-inner-panel="log"]');
  const toolbar = panel && panel.querySelector(".node-v6-log-toolbar");
  if (!toolbar) return false;
  let status = toolbar.querySelector('.kgw-copy-log-status-v1[data-net="' + net + '"]');
  if (!status) {
    status = document.createElement("span");
    status.setAttribute("class", "kgw-copy-log-status-v1");
    status.dataset.net = net;
    status.setAttribute("data-net", net);
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    toolbar.appendChild(status);
  }
  status.textContent = String(message || "");
  status.dataset.state = String(state || "info");
  status.hidden = !status.textContent;
  return true;
};
const wasmNodeCopyLogFailure = (net, button, error, details = {}) => {
  const safeError = wasmNodeClipboardSafeError(error);
  wasmNodeCopyLogStatus(net, safeError, "error");
  if (button) button.textContent = "Copy failed";
  wasmNodeStartTraceFrontend("frontend.copy_log_failed", {
    network: net,
    action: "copy-log",
    result: "error",
    details: { ...(details || {}), safeError, userFeedbackDisplayed: true },
  });
  return false;
};
const wasmNodeDispatchClipboardWrite = async (net, text, metadata = {}) => {
  const resolved = wasmNodeResolvePublicTauriInvoke();
  if (typeof resolved.invoke !== "function") {
    throw new Error("Tauri invoke API is not available. Expected window.__TAURI__.core.invoke from Tauri 2 with withGlobalTauri enabled.");
  }
  wasmNodeStartTraceFrontend("frontend.copy_log_dispatched", {
    network: net,
    action: "copy-log",
    result: "dispatched",
    details: {
      commandName: "kgw_copy_text_to_clipboard_v1",
      implementation: "native-tauri-command",
      runtimeRole: metadata.runtimeRole || "node",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || "",
      payloadFieldCount: 7,
    },
  });
  return await resolved.invoke("kgw_copy_text_to_clipboard_v1", {
    network: net,
    runtimeRole: metadata.runtimeRole || "node",
    bridgeInstanceId: metadata.bridgeInstanceId || "",
    text,
    characterCount: metadata.characterCount,
    lineCount: metadata.lineCount,
    sha256: metadata.sha256 || "",
  });
};
const wasmNodeHandleCopyLog = async (net, button) => {
  const copyNetwork = String(net || "").trim();
  const root = document.getElementById("kaspa-node");
  const activeNetwork = wasmNodeTraceActiveNetwork(root);
  const belongsToLiveNodeMonitor = Boolean(button && button.closest && button.closest('[data-node-inner-panel="log"]'));
  if (!copyNetwork) {
    return wasmNodeCopyLogFailure(net, button, "Copy Log could not resolve the active network.", {
      reason: "missing-network", activeNetwork, belongsToLiveNodeMonitor,
    });
  }
  wasmNodeStartTraceFrontend("frontend.copy_log_network_resolved", {
    network: copyNetwork,
    action: "copy-log",
    result: activeNetwork && activeNetwork !== copyNetwork ? "error" : "ok",
    details: { activeNetwork, buttonNetwork: copyNetwork, belongsToLiveNodeMonitor },
  });
  if (activeNetwork && activeNetwork !== copyNetwork) {
    return wasmNodeCopyLogFailure(copyNetwork, button, "Copy Log network mismatch; active network changed before copy started.", {
      reason: "network-mismatch", activeNetwork, buttonNetwork: copyNetwork, belongsToLiveNodeMonitor,
    });
  }
  if (button && button.dataset && button.dataset.kgwCopyLogInFlightV1 === "1") {
    return wasmNodeCopyLogFailure(copyNetwork, button, "Copy Log is already in progress for this network.", {
      reason: "duplicate-copy", activeNetwork, belongsToLiveNodeMonitor,
    });
  }
  const originalDisabled = Boolean(button && button.disabled);
  if (button) {
    button.dataset.kgwCopyLogInFlightV1 = "1";
    button.disabled = true;
  }
  let ok = false;
  try {
    const out = document.getElementById("node-" + copyNetwork + "-logOutput");
    const tag = String(out && out.tagName || "").toUpperCase();
    const rawText = String(out ? ((tag === "TEXTAREA" || tag === "INPUT") ? out.value : out.textContent) : "");
    const normalizedText = wasmNodeNormalizeClipboardLineEndings(rawText);
    const isPlaceholder = rawText.trim() === wasmNodeClipboardPlaceholderText(copyNetwork).trim();
    const characterCount = wasmNodeClipboardCharacterCount(normalizedText);
    const lineCount = wasmNodeClipboardLineCount(normalizedText);
    if (!out || isPlaceholder || !normalizedText.trim()) {
      wasmNodeStartTraceFrontend("frontend.copy_log_content_prepared", {
        network: copyNetwork, action: "copy-log", result: "error",
        details: { rawLogBufferSelected: Boolean(out), placeholderRejected: Boolean(isPlaceholder), runtimeRole: "node", bridgeInstanceId: "", characterCount, lineCount, sha256: "" },
      });
      throw new Error("Copy Log requires a non-empty raw log buffer for " + copyNetwork + ".");
    }
    const bytes = new TextEncoder().encode(normalizedText);
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    const sha256 = Array.from(new Uint8Array(digest)).map((value) => value.toString(16).padStart(2, "0")).join("");
    const metadata = { runtimeRole: "node", bridgeInstanceId: "", characterCount, lineCount, sha256 };
    wasmNodeStartTraceFrontend("frontend.copy_log_content_prepared", {
      network: copyNetwork, action: "copy-log", result: "ok",
      details: { rawLogBufferSelected: true, placeholderRejected: false, ...metadata },
    });
    await wasmNodeDispatchClipboardWrite(copyNetwork, normalizedText, metadata);
    if (button) button.textContent = "Copied";
    wasmNodeCopyLogStatus(copyNetwork, "Copied", "ok");
    wasmNodeStartTraceFrontend("frontend.copy_log_succeeded", {
      network: copyNetwork, action: "copy-log", result: "ok",
      details: { ...metadata, userFeedbackDisplayed: true },
    });
    ok = true;
  } catch (error) {
    wasmNodeCopyLogFailure(copyNetwork, button, error, { activeNetwork, belongsToLiveNodeMonitor });
  } finally {
    if (button) {
      button.disabled = originalDisabled;
      delete button.dataset.kgwCopyLogInFlightV1;
    }
  }
  return ok;
};
const wasmNodeStartTraceTauriShape = (adapterName = "") => {
  const tauri = window.__TAURI__;
  const keys = (value) => value && typeof value === "object" ? Object.keys(value).sort().slice(0, 24) : [];
  return { adapter: String(adapterName || "missing"), hasGlobalTauri: Boolean(tauri), globalKeys: keys(tauri), coreKeys: keys(tauri && tauri.core), tauriKeys: keys(tauri && tauri.tauri), hasCoreInvoke: typeof (tauri && tauri.core && tauri.core.invoke) === "function", hasTauriInvoke: typeof (tauri && tauri.tauri && tauri.tauri.invoke) === "function", hasRootInvoke: typeof (tauri && tauri.invoke) === "function", expectedConfiguredGlobal: "window.__TAURI__.core.invoke" };
};
const wasmNodeResolvePublicTauriInvoke = () => {
  const tauri = window.__TAURI__;
  for (const [adapter, owner, invoke] of [["window.__TAURI__.core.invoke", tauri && tauri.core, tauri && tauri.core && tauri.core.invoke], ["window.__TAURI__.tauri.invoke", tauri && tauri.tauri, tauri && tauri.tauri && tauri.tauri.invoke], ["window.__TAURI__.invoke", tauri, tauri && tauri.invoke]]) {
    if (typeof invoke === "function") return { adapter, invoke: invoke.bind(owner), shape: wasmNodeStartTraceTauriShape(adapter) };
  }
  return { adapter: "missing", invoke: null, shape: wasmNodeStartTraceTauriShape("missing") };
};
const wasmNodeStartTraceFrontend = (stage, options = {}) => {
  const resolved = wasmNodeResolvePublicTauriInvoke();
  if (typeof resolved.invoke !== "function") return false;
  const blocked = /(secret|token|private|mnemonic|wallet|address|commandPreview|completeCommand|arguments|appDir|path|rpcEndpoint|stratum)/i;
  const safeText = (value, fallback = "") => String(value ?? "").replace(/[\\r\\n\\t]+/g, " ").trim().slice(0, 220) || fallback;
  const sanitize = (value) => Object.fromEntries(Object.entries(value && typeof value === "object" ? value : {}).map(([key, item]) => [key, blocked.test(key) ? "[redacted]" : Array.isArray(item) ? item.slice(0,24).map(v => safeText(v)) : item && typeof item === "object" ? sanitize(item) : typeof item === "boolean" || typeof item === "number" ? item : safeText(item)]));
  const details = sanitize({ ...(options.details && typeof options.details === "object" ? options.details : {}), invokeAdapter: resolved.adapter });
  Promise.resolve(resolved.invoke("kgw_start_trace_frontend_v1", { stage: safeText(stage, "frontend.unknown"), network: safeText(options.network || options.net, "unknown"), action: safeText(options.action, "unknown"), result: safeText(options.result, "observed"), details: JSON.stringify(details) })).catch(() => {});
  return true;
};
const wasmNodeTraceActiveNetwork = (root = document.getElementById("kaspa-node")) => {
  if (!root) return "";
  const panel = Array.from(root.querySelectorAll("[data-node-network-panel]")).find((item) =>
    !item.hidden && (item.classList.contains("active") || item.dataset.active === "true"));
  if (panel && panel.dataset.nodeNetworkPanel) return panel.dataset.nodeNetworkPanel;
  const tab = Array.from(root.querySelectorAll("[data-node-network-tab]")).find((item) =>
    item.classList.contains("active") || item.getAttribute("aria-selected") === "true" || item.dataset.active === "true");
  return tab && tab.dataset.nodeNetworkTab || "";
};
const wasmNodeTraceNetworkFromElement = (element, root = document.getElementById("kaspa-node")) => {
  const carrier = element && element.closest &&
    element.closest("[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel]");
  const raw = [
    element && element.dataset && element.dataset.net,
    element && element.dataset && element.dataset.network,
    carrier && carrier.dataset && carrier.dataset.net,
    carrier && carrier.dataset && carrier.dataset.network,
    carrier && carrier.dataset && carrier.dataset.nodeNetworkPanel,
    carrier && carrier.id,
    carrier && carrier.className,
  ].filter(Boolean).join(" ").toLowerCase();
  if (raw.includes("testnet13") || raw.includes("tn13")) return "testnet13";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";
  return wasmNodeTraceActiveNetwork(root) || "mainnet";
};
const wasmNodeTraceStartButtonState = (net) => {
  const panel = document.querySelector('[data-node-network-panel="' + net + '"]');
  const start = panel && panel.querySelector('[data-node-action="start"][data-net="' + net + '"]');
  const stop = panel && panel.querySelector('[data-node-action="stop"][data-net="' + net + '"]');
  return { startRendered: Boolean(start), startDisabled: Boolean(start && start.disabled), stopRendered: Boolean(stop), stopDisabled: Boolean(stop && stop.disabled) };
};
const wasmNodeInstallStartTraceDocumentClickObserver = (root) => {
  if (window.__kgwStartTraceDocumentClickObserverR1 === true) return false;
  window.__kgwStartTraceDocumentClickObserverR1 = true;
  document.addEventListener("click", (event) => {
    const button = event.target && event.target.closest && event.target.closest("[data-node-action]");
    const nodeRoot = root || document.getElementById("kaspa-node");
    if (!button || !nodeRoot || !nodeRoot.contains(button)) return;
    const action = String(button.dataset.nodeAction || "").trim();
    const network = wasmNodeTraceNetworkFromElement(button, nodeRoot);
    const activeNetwork = wasmNodeTraceActiveNetwork(nodeRoot);
    const belongsToSettings = Boolean(button.closest('[data-node-inner-panel="settings"]'));
    const belongsToLiveNodeMonitor = Boolean(button.closest('[data-node-inner-panel="log"]'));
    if (action === "copy-log") {
      wasmNodeStartTraceFrontend("frontend.copy_log_click_observed", {
        network,
        action: "copy-log",
        result: "observed",
        details: { trusted: Boolean(event && event.isTrusted), belongsToSettings, belongsToLiveNodeMonitor, selectedNetwork: activeNetwork, buttonDisabled: Boolean(button.disabled), inFlight: button.dataset.kgwCopyLogInFlightV1 === "1" },
      });
      return;
    }
    if (action !== "start" && action !== "stop") return;
    wasmNodeStartTraceFrontend("frontend.capture_click_observed", {
      network,
      action,
      result: "observed",
      details: { trusted: Boolean(event && event.isTrusted), belongsToSettings, belongsToLiveNodeMonitor, selectedNetwork: activeNetwork, buttonDisabled: Boolean(button.disabled), buttonAction: action },
    });
  }, true);
  return true;
};
const wasmNodeTraceRenderedStartControls = (root) => {
  for (const net of ["mainnet", "testnet10", "testnet13"]) {
    const settingsPanel = root && root.querySelector('[data-node-network-panel="' + net + '"] [data-node-inner-panel="settings"]');
    const start = settingsPanel && settingsPanel.querySelector('[data-node-action="start"][data-net="' + net + '"]');
    wasmNodeStartTraceFrontend("frontend.settings_subtab_rendered", {
      network: net,
      action: "render",
      result: settingsPanel ? "ok" : "missing",
      details: { belongsToSettings: Boolean(settingsPanel), selectedNetwork: wasmNodeTraceActiveNetwork(root) },
    });
    wasmNodeStartTraceFrontend("frontend.start_control_rendered", {
      network: net,
      action: "start",
      result: start ? "ok" : "missing",
      details: { belongsToSettings: Boolean(start && settingsPanel && settingsPanel.contains(start)), startDisabled: Boolean(start && start.disabled) },
    });
  }
  return true;
};
const wasmNodeRuntimeActionForCommand = (command) =>
  command === "kgw_kgw_apply_node_settings_v1" ? "start" :
    command === "kgw_kgw_disable_network_v1" ? "stop" : "runtime";
const wasmNodeSmallOwnerTrace = (net, action, phase, details) => {
  const safeNet = String(net || "unknown");
  const safeAction = String(action || "small-owner");
  const safePhase = String(phase || "unknown");
  const tauri = window.__TAURI__;
  const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
    ? tauri.core.invoke.bind(tauri.core)
    : tauri && typeof tauri.invoke === "function"
      ? tauri.invoke.bind(tauri)
      : window.__TAURI_INVOKE__;
  if (typeof invoke !== "function") return false;
  Promise.resolve(invoke("kgw_frontend_button_trace_v1", {
    scope: "node",
    net: safeNet,
    action: safeAction,
    phase: safePhase,
    details: JSON.stringify({ patch: "KGW_SMALL_NODE_BRIDGE_TRACE_PATCH_R44D", existingOwner: "node-small-owner-functions", network: safeNet, action: safeAction, phase: safePhase, details: details && typeof details === "object" ? details : {} }),
  })).catch(() => {});
  return true;
};
`;
  const executable = importPrelude + source
    .replace(/^import[\s\S]*?from\s+["'][^"']+["'];\s*/gm, "")
    .replace(/^import\s+["'][^"']+["'];\s*/gm, "")
    .replace(/^await\s+initNodeRust\(\);\s*/gm, "")
    .replace(/export\s+async\s+function\s+initKaspaNodeTab/, "async function initKaspaNodeTab")
    .replace(/export\s*\{[^}]+\}\s*;?/g, "")
    .replace(/export\s+default\s+initKaspaNodeTab\s*;/, "")
    + "\nwindow.__kgwStartButtonTest = { initKaspaNodeTab, getTauriInvoke, kgwResolvePublicTauriInvokeR1, kgwStartTraceTauriShapeR1, kgwNodeR51SetRuntimeButtons, KGW_NODE_R51_TRANSITIONS };\n";
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
    core: {
      invoke: async (command, payload) => {
        calls.push({ command, payload });
        if (command !== "kgw_kgw_apply_node_settings_v1") return "ignored";
        return "parallel-owned-self-worker started;role=node;network=" + payload.network + ";pid=4242;owner=self-worker;runtime_state=running;readiness=READY";
      },
    },
  };

  calls.length = 0;
  const mainnetStartReady = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  mainnetStartReady.disabled = false;
  mainnetStartReady.click();
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
  const testnet10StartReady = root.querySelector('[data-node-action="start"][data-net="testnet10"]');
  testnet10StartReady.disabled = false;
  testnet10StartReady.click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 1, "testnet10 Start click must invoke exactly once after tab switch");
  assert.strictEqual(startCalls(calls)[0].payload.network, "testnet10");

  calls.length = 0;
  const testnet13Start = root.querySelector('[data-node-action="start"][data-net="testnet13"]');
  testnet13Start.click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 0, "testnet13 Start must not invoke while opt-in is disabled");

  window.__TAURI__.core.invoke = async (command, payload) => {
    calls.push({ command, payload });
    if (command === "kgw_start_trace_frontend_v1" || command === "kgw_frontend_button_trace_v1") return true;
    if (command === "kgw_kgw_apply_node_settings_v1") {
      throw new Error("spawn_failed=true;runtime_role=node;network=mainnet;source=self-worker;error=Access is denied.");
    }
    if (command === "kgw_runtime_owner_status_v1") {
      throw new Error("runtime status unavailable after rejected Start");
    }
    return true;
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
  assert.strictEqual(mainnetStart.disabled, true, "failed Start must remain non-startable until backend reconciliation proves terminal state");
  assert.ok(traceStages(calls).includes("frontend.invoke_rejected"), "failed Start must trace invoke rejection");
  assert.ok(traceStages(calls).includes("frontend.button_state_restored_after_failure"), "failed Start must trace button restoration");
  assert.strictEqual(statusNode.textContent, "Reconciling", "failed Start must expose Reconciling until backend status is known");
  assert.ok(
    errorNode.textContent.includes("Access is denied."),
    "failed Start must expose original error, actual text: " + errorNode.textContent + "; status=" + statusNode.textContent + "; evidence=" + evidenceNode.textContent,
  );

  const rawLog = document.getElementById("node-mainnet-logOutput").textContent;
  assert.ok(!/initialized|parallel-owned-self-worker started|KGW node start response/i.test(rawLog), "raw log must not contain synthetic startup success text");
}

async function startupStopAvailabilityTests() {
  const { window, root } = createHarness();
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();
  const api = window.__kgwStartButtonTest;
  const stop = root.querySelector('[data-node-action="stop"][data-net="mainnet"]');
  api.KGW_NODE_R51_TRANSITIONS.mainnet = "starting";
  api.kgwNodeR51SetRuntimeButtons("mainnet", false, false);
  assert.strictEqual(stop.disabled, true, "Node Stop must remain disabled until READY; pre-READY cancellation is not an owned backend contract");
  api.KGW_NODE_R51_TRANSITIONS.mainnet = "";
  api.kgwNodeR51SetRuntimeButtons("mainnet", true, false);
  assert.strictEqual(stop.disabled, false, "Node Stop must become available after READY/running truth is confirmed");
}

async function stopTruthfulnessTests() {
  const calls = [];
  let resolveStop;
  let stopResponse = null;
  const { window, document, root } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          if (command === "kgw_kgw_disable_network_v1") {
            if (stopResponse !== null) return stopResponse;
            return await new Promise((resolve) => { resolveStop = resolve; });
          }
          if (command === "kgw_runtime_owner_status_v1") {
            return "parallel-owned-self-worker status;role=node;network=" + payload.network + ";running=false";
          }
          if (command === "kgw_kgw_runtime_logs_v1") return { entries: [] };
          return true;
        },
      },
    },
  });
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();

  const start = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  const stop = root.querySelector('[data-node-action="stop"][data-net="mainnet"]');
  start.disabled = true;
  stop.disabled = false;
  stop.click();
  await flush();
  assert.strictEqual(document.getElementById("node-mainnet-runtimeStatus").textContent, "Stopping");
  assert.strictEqual(stop.disabled, true, "duplicate Stop must remain disabled while backend Stop is pending");

  resolveStop("parallel-owned-self-worker stopped;role=node;network=mainnet;running=false;graceful=true;forced=false");
  await flush();
  assert.strictEqual(document.getElementById("node-mainnet-runtimeStatus").textContent, "Stopped");
  assert.strictEqual(document.getElementById("node-mainnet-runtimeEvidence").textContent, "Graceful official shutdown confirmed");

  stopResponse = "parallel-owned-self-worker stopped;role=node;network=mainnet;running=false;graceful=false;forced=true;reason=graceful stop timed out";
  start.disabled = true;
  stop.disabled = false;
  stop.click();
  await flush();
  assert.ok(document.getElementById("node-mainnet-runtimeError").textContent.includes("FORCED"));
  assert.strictEqual(document.getElementById("node-mainnet-runtimeEvidence").textContent, "FORCED termination confirmed");

  stopResponse = "parallel-owned-self-worker stopped with graceful failure;role=node;network=mainnet;running=false;graceful=false;forced=false;stop_failed=true;stop_outcome=FAILED;reason=official shutdown failed";
  start.disabled = true;
  stop.disabled = false;
  stop.click();
  await flush();
  assert.ok(document.getElementById("node-mainnet-runtimeError").textContent.includes("Official graceful shutdown failed"));
  assert.strictEqual(document.getElementById("node-mainnet-runtimeEvidence").textContent, "Worker exited after graceful shutdown failure");
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
  assert.strictEqual(start.disabled, true, "missing invoke API must keep Start disabled until IPC/runtime truth can be reconciled");
  assert.strictEqual(document.getElementById("node-mainnet-runtimeStatus").textContent, "Reconciling", "missing invoke API must expose Reconciling rather than fabricate Stopped");
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
  mainnetLog.textContent = "Mainnet log is empty.";
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
    await startupStopAvailabilityTests();
    await stopTruthfulnessTests();
    await missingInvokeApiVisibleErrorTest();
    await tracePayloadSafetyTests();
    await copyLogFrontendTests();
    console.log("KGW start button and Copy Log frontend tests PASSED");
  } catch (error) {
    fail(error && error.stack ? error.stack : String(error));
  }
})();
"########;

pub fn run(root: &Path) -> Result<String, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-start-button-frontend-")
        .tempdir()
        .map_err(|error| format!("failed to create start-button tempdir: {error}"))?;
    let bridge = temp.path().join("bridge.cjs");
    fs::write(&bridge, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write start-button Node bridge: {error}"))?;
    let output = Command::new("node")
        .arg(&bridge)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch start-button Node bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "start-button frontend bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_frontend_bridge_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let message = run(root).unwrap();
        assert!(message.contains("PASSED"), "{message}");
    }

    #[test]
    fn generated_bridge_is_interop_only_and_loads_real_frontend() {
        assert!(NODE_BRIDGE.contains("apps"));
        assert!(NODE_BRIDGE.contains("kaspa-node.js"));
        assert!(NODE_BRIDGE.contains("vm.runInNewContext"));
        assert!(NODE_BRIDGE.contains("KGW start button and Copy Log frontend tests PASSED"));
    }
}
