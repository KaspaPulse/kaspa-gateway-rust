use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const BRIDGE_SOURCE: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const BRIDGE_HELPERS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs";
const BRIDGE_RUNTIME_CORE_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_runtime_core.rs";
const BRIDGE_START_TRACE_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_start_trace.rs";
const BRIDGE_INSTANCE_SETTINGS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_instance_settings.rs";
const BRIDGE_COMMAND_OPTIONS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_command_options.rs";
const BRIDGE_INSTANCE_UI_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_instance_ui.rs";
const BRIDGE_RENDER_SOURCE: &str = "crates/kaspa-gateway-frontend-wasm/src/bridge_render.rs";
const BRIDGE_PORT_UI_SOURCE: &str = "crates/kaspa-gateway-frontend-wasm/src/bridge_port_ui.rs";
const WASM_JS: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm.js";
const WASM_BIN: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm_bg.wasm";

const SLICES: &[(&str, &str)] = &[
    (
        "const BRIDGE_NETWORKS = wasmBridgeNetworkProfiles();",
        "const bridgeInstances = {",
    ),
    (
        "const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();",
        "/* KGW_BRIDGE_START_TRACE_V1 is Rust-owned in bridge_start_trace.rs. */",
    ),
    (
        "async function runBridgeIntegratedAction(",
        "/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */",
    ),
    (
        "function kgwBridgeR51LiveRefreshCallbacksR257(",
        "// KGW_BRIDGE_RAW_LOG_LIVE_EXACT_R134E",
    ),
];

const NODE_BRIDGE: &str = r##"
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import vm from "node:vm";

const [selectedPath, requestPath, resultPath, wasmJsPath, wasmPath, productPath] = process.argv.slice(2);
const selected = await readFile(selectedPath, "utf8");
const request = JSON.parse(await readFile(requestPath, "utf8"));
const wasm = await import(pathToFileURL(wasmJsPath).href);
await wasm.default({ module_or_path: await readFile(wasmPath) });

globalThis.window = globalThis;
const innerTabStore = new Map();
globalThis.localStorage = {
  getItem(key) {
    return innerTabStore.has(String(key)) ? innerTabStore.get(String(key)) : null;
  },
  setItem(key, value) {
    innerTabStore.set(String(key), String(value));
  }
};

const r51Storage = globalThis.localStorage;
const r51Prefix = "kgw.bridge.direct.v51.";
const r51StorageOwnership = {
  missing: wasm.bridgeR51Load("saved:missing"),
  networkKeys: wasm.bridgeR51Keys()
};
for (const key of ["saved:mainnet", "default:testnet10", "factory:testnet13"]) {
  wasm.bridgeR51Store(key, { key, value: 51 });
}
r51StorageOwnership.roundTrips = ["saved:mainnet", "default:testnet10", "factory:testnet13"]
  .map((key) => wasm.bridgeR51Load(key));
r51StorageOwnership.exactKeys = [...innerTabStore.keys()].sort();
r51StorageOwnership.rawSaved = innerTabStore.get(r51Prefix + "saved:mainnet");
innerTabStore.set(r51Prefix + "malformed", "{");
r51StorageOwnership.malformed = wasm.bridgeR51Load("malformed");
r51StorageOwnership.falsey = [false, 0, "", null].map((value) => {
  wasm.bridgeR51Store("falsey", value);
  return wasm.bridgeR51Load("falsey");
});
const r51Throws = (action) => { try { action(); return false; } catch { return true; } };
const r51Cycle = {}; r51Cycle.self = r51Cycle;
r51StorageOwnership.circularThrows = r51Throws(() => wasm.bridgeR51Store("cycle", r51Cycle));
r51StorageOwnership.circularNotWritten = !innerTabStore.has(r51Prefix + "cycle");
globalThis.localStorage = {
  getItem() { throw new Error("storage read denied"); },
  setItem() { throw new Error("storage write denied"); }
};
r51StorageOwnership.readFailure = wasm.bridgeR51Load("saved:mainnet");
r51StorageOwnership.writeFailureThrows = r51Throws(() => wasm.bridgeR51Store("saved:mainnet", {}));
delete globalThis.localStorage;
r51StorageOwnership.unavailableLoad = wasm.bridgeR51Load("saved:mainnet");
r51StorageOwnership.unavailableStoreThrows = r51Throws(() => wasm.bridgeR51Store("saved:mainnet", {}));
globalThis.localStorage = r51Storage;

let r51Current = { value: "factory" };
const r51Writes = [];
const r51Reads = [];
globalThis.localStorage = {
  ...r51Storage,
  getItem(key) { r51Reads.push(key); return r51Storage.getItem(key); }
};
const r51Callbacks = {
  readSettings: () => r51Current,
  writeSettings: (net, values) => r51Writes.push({ net: String(net), values }),
  normalizeNetworkPortValues: (_net, values) => values,
  requireValidSettings: () => {},
  updateCommand: () => {}
};
innerTabStore.clear();
wasm.bridgeR51CaptureFactoryDefaults(r51Callbacks);
r51Current = { value: "changed" };
wasm.bridgeR51CaptureFactoryDefaults(r51Callbacks);
r51StorageOwnership.factoryPreserved = wasm.bridgeR51Load("factory:mainnet");

wasm.bridgeR51LoadSavedSettings(r51Callbacks);
r51StorageOwnership.missingSavedUsesCurrent = r51Writes.filter((entry) => entry.net === "mainnet").at(-1)?.values ?? null;
r51Current = { value: "saved" };
wasm.bridgeR51SaveSettings("mainnet", r51Callbacks);
r51Current = { value: "unsaved" };
wasm.bridgeR51LoadSavedSettings(r51Callbacks);
r51StorageOwnership.savedLoaded = r51Writes.filter((entry) => entry.net === "mainnet").at(-1)?.values ?? null;
r51Current = { value: "default" };
wasm.bridgeR51SetAsDefaults("mainnet", r51Callbacks);
r51Reads.length = 0;
wasm.bridgeR51RestoreDefaults("mainnet", r51Callbacks);
r51StorageOwnership.defaultRestored = r51Writes.filter((entry) => entry.net === "mainnet").at(-1)?.values ?? null;
r51StorageOwnership.defaultReadOrder = [...r51Reads];
wasm.bridgeR51Store("default:mainnet", false);
r51Reads.length = 0;
wasm.bridgeR51RestoreDefaults("mainnet", r51Callbacks);
r51StorageOwnership.factoryRestored = r51Writes.filter((entry) => entry.net === "mainnet").at(-1)?.values ?? null;
r51StorageOwnership.fallbackReadOrder = [...r51Reads];
innerTabStore.delete(r51Prefix + "factory:mainnet");
wasm.bridgeR51RestoreDefaults("mainnet", r51Callbacks);
r51StorageOwnership.noDefaults = r51Writes.filter((entry) => entry.net === "mainnet").at(-1)?.values ?? null;
// OP255: drain fire-and-forget Restore Defaults path refreshes before later transport assertions.
await new Promise((resolve) => setTimeout(resolve, 0));
globalThis.localStorage = r51Storage;
innerTabStore.clear();

const portOnlyNormalizationOwnership = {
  plain: wasm.bridgePlainPortOnlyValueR98("5655"),
  colon: wasm.bridgePlainPortOnlyValueR98(":5655"),
  trimmedLeadingZeros: wasm.bridgePlainPortOnlyValueR98("  :080  "),
  numeric: wasm.bridgePlainPortOnlyValueR98(5655),
  falseyZero: wasm.bridgePlainPortOnlyValueR98(0),
  falseyFalse: wasm.bridgePlainPortOnlyValueR98(false),
  falseyNull: wasm.bridgePlainPortOnlyValueR98(null),
  zeroString: wasm.bridgePlainPortOnlyValueR98("0"),
  colonZero: wasm.bridgePlainPortOnlyValueR98(":00000"),
  max: wasm.bridgePlainPortOnlyValueR98("65535"),
  tooHigh: wasm.bridgePlainPortOnlyValueR98("65536"),
  tooLong: wasm.bridgePlainPortOnlyValueR98("123456"),
  hostPort: wasm.bridgePlainPortOnlyValueR98(" host:5655 "),
  sameColonLeadingZeros: wasm.bridgeSamePortValueR98(":00080", "80"),
  sameWhitespace: wasm.bridgeSamePortValueR98(" 5655 ", ":5655"),
  different: wasm.bridgeSamePortValueR98("5556", "5655")
};

const r95bPortNormalizationOwnership = {};
const r95bTestnet10 = {
  "bridge-testnet10-stratumPort": { value: "5556" },
  "bridge-testnet10-promPort": { value: ":2113" }
};
const r95bTestnet10Returned = wasm.bridgeR95BNormalizeNetworkPortValues(
  "testnet10",
  r95bTestnet10,
  "harness-stale-testnet10"
);
r95bPortNormalizationOwnership.testnet10Identity =
  r95bTestnet10Returned === r95bTestnet10;
r95bPortNormalizationOwnership.testnet10Stratum =
  r95bTestnet10["bridge-testnet10-stratumPort"].value;
r95bPortNormalizationOwnership.testnet10Prom =
  r95bTestnet10["bridge-testnet10-promPort"].value;

const r95bTestnet13 = {
  "bridge-testnet13-stratumPort": { value: ":5557" },
  "bridge-testnet13-promPort": { value: "2114" }
};
wasm.bridgeR95BNormalizeNetworkPortValues(
  "testnet13",
  r95bTestnet13,
  "harness-stale-testnet13"
);
r95bPortNormalizationOwnership.testnet13Stratum =
  r95bTestnet13["bridge-testnet13-stratumPort"].value;
r95bPortNormalizationOwnership.testnet13Prom =
  r95bTestnet13["bridge-testnet13-promPort"].value;

const r95bDisplayOnly = {
  "bridge-mainnet-stratumPort": { value: ":05555" },
  "bridge-mainnet-promPort": { value: "  :02112  " }
};
wasm.bridgeR95BNormalizeNetworkPortValues(
  "mainnet",
  r95bDisplayOnly,
  "harness-display-only"
);
r95bPortNormalizationOwnership.displayStratum =
  r95bDisplayOnly["bridge-mainnet-stratumPort"].value;
r95bPortNormalizationOwnership.displayProm =
  r95bDisplayOnly["bridge-mainnet-promPort"].value;

const r95bCustom = {
  "bridge-testnet10-stratumPort": { value: "5678" },
  "bridge-testnet10-promPort": { value: "2277" }
};
wasm.bridgeR95BNormalizeNetworkPortValues(
  "testnet10",
  r95bCustom,
  "harness-custom"
);
r95bPortNormalizationOwnership.customStratum =
  r95bCustom["bridge-testnet10-stratumPort"].value;
r95bPortNormalizationOwnership.customProm =
  r95bCustom["bridge-testnet10-promPort"].value;

const instanceNetworkKeyOwnership = {
  directString: wasm.bridgeInstanceNetworkKeyR15(" testnet10 ", "mainnet"),
  directObject: wasm.bridgeInstanceNetworkKeyR15({ key: " testnet13 " }, "mainnet"),
  fallbackString: wasm.bridgeInstanceNetworkKeyR15("devnet", " testnet10 "),
  fallbackObject: wasm.bridgeInstanceNetworkKeyR15(null, { key: "testnet13" }),
  directWins: wasm.bridgeInstanceNetworkKeyR15("mainnet", { key: "testnet10" }),
  invalidDefault: wasm.bridgeInstanceNetworkKeyR15({ key: "devnet" }, "unknown"),
  numericIgnored: wasm.bridgeInstanceNetworkKeyR15(10, "testnet10"),
  nonStringObjectKeyIgnored: wasm.bridgeInstanceNetworkKeyR15(
    { key: 10 },
    { key: "testnet13" }
  )
};

const bridgeOwnedNodeLockEvents = [];
globalThis.CustomEvent = function CustomEvent(type, options = {}) {
  this.type = String(type || "");
  this.detail = options.detail || null;
};
globalThis.dispatchEvent = (event) => {
  bridgeOwnedNodeLockEvents.push(event);
  return true;
};
wasm.bridgeSetOwnedNodeLockR65E("", true, { ignored: true });
wasm.bridgeSetOwnedNodeLockR65E("mainnet", true, { source: "op236-smoke", pid: "4242" });
const bridgeOwnedNodeLockRecord =
  globalThis.__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E?.mainnet || null;
const bridgeOwnedNodeLockOwnership = {
  locked: Boolean(bridgeOwnedNodeLockRecord?.locked),
  net: String(bridgeOwnedNodeLockRecord?.net || ""),
  reason: String(bridgeOwnedNodeLockRecord?.reason || ""),
  updatedAtPositive: Number(bridgeOwnedNodeLockRecord?.updatedAt || 0) > 0,
  detailsSource: String(bridgeOwnedNodeLockRecord?.details?.source || ""),
  detailsPid: String(bridgeOwnedNodeLockRecord?.details?.pid || ""),
  lockEventType: String(bridgeOwnedNodeLockEvents[0]?.type || ""),
  lockEventNet: String(bridgeOwnedNodeLockEvents[0]?.detail?.net || ""),
  lockEventLocked: Boolean(bridgeOwnedNodeLockEvents[0]?.detail?.locked),
  lockEventSource: String(bridgeOwnedNodeLockEvents[0]?.detail?.source || "")
};
wasm.bridgeSetOwnedNodeLockR65E("mainnet", false, { source: "op236-unlock" });
bridgeOwnedNodeLockOwnership.unlockedAbsent =
  !Object.prototype.hasOwnProperty.call(
    globalThis.__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E || {},
    "mainnet"
  );
bridgeOwnedNodeLockOwnership.eventCount = bridgeOwnedNodeLockEvents.length;
bridgeOwnedNodeLockOwnership.unlockEventType =
  String(bridgeOwnedNodeLockEvents[1]?.type || "");
bridgeOwnedNodeLockOwnership.unlockEventNet =
  String(bridgeOwnedNodeLockEvents[1]?.detail?.net || "");
bridgeOwnedNodeLockOwnership.unlockEventLocked =
  Boolean(bridgeOwnedNodeLockEvents[1]?.detail?.locked);
bridgeOwnedNodeLockOwnership.unlockEventSource =
  String(bridgeOwnedNodeLockEvents[1]?.detail?.source || "");

globalThis.kgwT = (key) =>
  key === "runtime.failed" ? "Runtime Function Failed" : key;
const runtimeTranslationFunction = wasm.bridgeTranslateRuntimeFeedback(
  "runtime.failed",
  "Failed"
);
globalThis.kgwT = (key) => key;
globalThis.__kgwI18nDictR107 = {
  "runtime.failed": "Flat Dictionary Failed"
};
const runtimeTranslationFunctionKeyFallsThrough =
  wasm.bridgeTranslateRuntimeFeedback("runtime.failed", "Failed");
globalThis.kgwT = () => {
  throw new Error("runtime translation failure");
};
globalThis.__kgwI18nDictR107 = {
  "runtime.failed": "Dictionary After Runtime Throw"
};
const runtimeTranslationRuntimeThrows =
  wasm.bridgeTranslateRuntimeFeedback("runtime.failed", "Failed");
delete globalThis.kgwT;
globalThis.__kgwI18nDictR107 = {
  runtime: { failed: "Nested Dictionary Failed" }
};
const runtimeTranslationNested = wasm.bridgeTranslateRuntimeFeedback(
  "runtime.failed",
  "Failed"
);
globalThis.__kgwI18nDictR107 = {};
const runtimeTranslationFallback = wasm.bridgeTranslateRuntimeFeedback(
  "runtime.failed",
  "Failed"
);
const runtimeTranslationFallbackToKey = wasm.bridgeTranslateRuntimeFeedback(
  "runtime.failed",
  ""
);
globalThis.kgwT = {};
globalThis.kgwI18n = () => "Must Not Win";
globalThis.__kgwI18nDictR107 = {
  "runtime.failed": "First Truthy Non Function Uses Dictionary"
};
const runtimeTranslationFirstTruthyRuntime =
  wasm.bridgeTranslateRuntimeFeedback("runtime.failed", "Failed");
delete globalThis.kgwT;
delete globalThis.kgwI18n;
delete globalThis.__kgwI18nDictR107;
delete globalThis.__kgwI18nDict;
delete globalThis.kgwI18nDict;
delete globalThis.__KGW_I18N_DICT__;
const runtimeTranslationOwnership = {
  runtimeFunction: String(runtimeTranslationFunction || ""),
  runtimeKeyFallsThrough: String(runtimeTranslationFunctionKeyFallsThrough || ""),
  runtimeThrows: String(runtimeTranslationRuntimeThrows || ""),
  nestedDictionary: String(runtimeTranslationNested || ""),
  fallback: String(runtimeTranslationFallback || ""),
  fallbackToKey: String(runtimeTranslationFallbackToKey || ""),
  firstTruthyRuntime: String(runtimeTranslationFirstTruthyRuntime || "")
};

const innerTabOwnership = {
  defaultMainnet: wasm.bridgeResolveInnerTab("mainnet"),
  normalizeInvalid: wasm.bridgeNormalizeInnerTab("invalid"),
  saveSettings: wasm.bridgeSaveInnerTab("mainnet", "settings"),
  savedMainnet: wasm.bridgeResolveInnerTab("mainnet"),
  defaultTestnet10: wasm.bridgeResolveInnerTab("testnet10"),
  saveInvalid: wasm.bridgeSaveInnerTab("testnet10", "invalid"),
  savedTestnet10: wasm.bridgeResolveInnerTab("testnet10"),
  mainnetAfterTestnet: wasm.bridgeResolveInnerTab("mainnet")
};
const lastNetworkOwnership = {
  initial: wasm.bridgeReadLastNetwork(),
  normalizeMainnet: wasm.bridgeNormalizeNetwork(" mainnet "),
  normalizeInvalid: wasm.bridgeNormalizeNetwork("devnet"),
  saveTestnet10: wasm.bridgeSaveLastNetwork(" testnet10 "),
  savedAfterValid: wasm.bridgeReadLastNetwork(),
  saveInvalid: wasm.bridgeSaveLastNetwork("devnet"),
  savedAfterInvalid: wasm.bridgeReadLastNetwork()
};

const transportCalls = [];
let transportOwnerStatus = "transport-ok";
const transportInvoke = async (command, args) => {
  transportCalls.push({
    command: String(command || ""),
    network: String(args?.network || ""),
    runtimeRole: String(args?.runtimeRole || "")
  });
  if (String(command || "") === "kgw_settings_context_v1") {
    return { appDir: "APPDIR-" + String(args?.network || "") };
  }
  if (String(command || "") === "kgw_runtime_owner_status_v1") {
    return transportOwnerStatus;
  }
  return "transport-ok";
};
globalThis.__TAURI__ = { core: { invoke: transportInvoke } };
const transportAvailable = wasm.bridgeRuntimeInvokeAvailable();
const transportResult = await wasm.bridgeInvokeRuntimeCommand(
  "kgw_kgw_runtime_logs_v1",
  { network: "mainnet", runtimeRole: "bridge" }
);
const previewTransportResult = await wasm.bridgePreparePreview(
  "mainnet",
  { network: "mainnet", bridgeCommandPreview: "kaspad --test" }
);
const previewTransportCommand = String(transportCalls.at(-1)?.command || "");
delete globalThis.__TAURI__;
let previewUnavailableError = "";
try {
  await wasm.bridgePreparePreview("mainnet", { network: "mainnet" });
} catch (error) {
  previewUnavailableError = String(error?.message || error || "");
}
globalThis.__TAURI__ = { core: { invoke: transportInvoke } };

class NodeLike {
  constructor() {
    this.children = [];
    this.textContent = "";
    this.className = "";
    this.parentElement = null;
  }
  appendChild(node) {
    if (node && typeof node === "object") node.parentElement = this;
    this.children.push(node);
    return node;
  }
  insertBefore(node, reference) {
    if (node && typeof node === "object") node.parentElement = this;
    const index = this.children.indexOf(reference);
    if (index >= 0) this.children.splice(index, 0, node);
    else this.children.push(node);
    return node;
  }
  querySelector(selector) {
    if (selector === ".kgw-status-value") {
      const stack = [...this.children];
      while (stack.length) {
        const node = stack.shift();
        if (node?.className === "kgw-status-value") return node;
        if (Array.isArray(node?.children)) stack.push(...node.children);
      }
    }
    return null;
  }
}
class Element extends NodeLike {
  constructor() {
    super();
    this.hidden = false;
    this.disabled = false;
    this.style = {};
    this.title = "";
    this.dataset = {};
    this.value = "";
    this.attributes = {};
    this.listeners = {};
    const classes = new Set();
    this.classList = {
      contains(name) { return classes.has(String(name)); },
      toggle(name, force) {
        const token = String(name);
        if (force === undefined) {
          if (classes.has(token)) {
            classes.delete(token);
            return false;
          }
          classes.add(token);
          return true;
        }
        if (force) {
          classes.add(token);
          return true;
        }
        classes.delete(token);
        return false;
      },
      add(...names) { for (const name of names) classes.add(String(name)); },
      remove(...names) { for (const name of names) classes.delete(String(name)); }
    };
  }
  setAttribute(key, value) {
    this.attributes[key] = String(value);
    this[key] = String(value);
  }
  addEventListener(type, listener) {
    this.listeners[String(type)] = listener;
  }
  closest() {
    return null;
  }
  removeAttribute(key) {
    delete this.attributes[key];
  }
  replaceChildren(...nodes) {
    this.children = nodes;
    for (const node of nodes) {
      if (node && typeof node === "object") node.parentElement = this;
    }
  }
}
const elements = new Map();
for (const name of [
  "runtimeError",
  "runtimeStatus",
  "policyStatus",
  "monitorState",
  "logEmpty",
  "logOutput",
  "settingsAuthority",
  "previewStatus",
  "commandPreview",
  "appdir",
  "inprocessAppdirMirror"
]) {
  elements.set("bridge-mainnet-" + name, new Element());
}
const mainnetNodeMode = new Element();
mainnetNodeMode.value = "external";
elements.set(wasm.bridgeElementId("mainnet", "nodeMode"), mainnetNodeMode);
const start = new Element();
const stop = new Element();
const next = new Element();
const panel = {
  querySelector(selector) {
    if (selector.includes('data-bridge-action="start"')) return start;
    if (selector.includes('data-bridge-action="stop"')) return stop;
    if (selector.includes('data-bridge-action="monitor-next"')) return next;
    return null;
  }
};

const directNodeMode = new Element();
directNodeMode.value = "inprocess";
elements.set(wasm.bridgeElementId("r65f-direct", "nodeMode"), directNodeMode);

const primaryPanelNodeMode = new Element();
primaryPanelNodeMode.value = "external";
const primaryModePanel = {
  querySelector(selector) {
    return String(selector).includes("nodeMode") ? primaryPanelNodeMode : null;
  }
};

const secondaryPanelNodeMode = new Element();
secondaryPanelNodeMode.value = "secondary";
const secondaryModePanel = {
  querySelector(selector) {
    return String(selector).includes("nodeMode") ? secondaryPanelNodeMode : null;
  }
};

const precedenceDirectNodeMode = new Element();
precedenceDirectNodeMode.value = "";
elements.set(wasm.bridgeElementId("r65f-precedence", "nodeMode"), precedenceDirectNodeMode);
const precedencePanelNodeMode = new Element();
precedencePanelNodeMode.value = "must-not-win";
const precedenceModePanel = {
  querySelector(selector) {
    return String(selector).includes("nodeMode") ? precedencePanelNodeMode : null;
  }
};

function op264InlineToggle(net, name) {
  const toggle = new Element();
  toggle.dataset.net = String(net || "");
  toggle.dataset.bridgeCommandOptionToggleR7 = String(name || "");
  return toggle;
}
const op264MainAlpha = op264InlineToggle("op264", "customAlpha");
const op264MainBeta = op264InlineToggle("op264", "customBeta");
const op264OtherAlpha = op264InlineToggle("op264-other", "customAlpha");
const op264InlineToggles = [op264MainAlpha, op264MainBeta, op264OtherAlpha];

const documentImpl = {
  getElementById(id) {
    return elements.get(id) || null;
  },
  querySelectorAll(selector) {
    const text = String(selector);
    if (text === "[data-bridge-command-option-toggle-r7]") return op264InlineToggles;
    return [];
  },
  querySelector(selector) {
    const text = String(selector);
    if (text.includes('data-bridge-network-panel="mainnet"')) return panel;
    if (text.includes('data-bridge-panel="r65f-primary"')) return primaryModePanel;
    if (text.includes('data-net="r65f-secondary"')) return secondaryModePanel;
    if (text.includes('data-bridge-panel="r65f-precedence"')) return precedenceModePanel;
    if (text.includes('r65f-error')) throw new Error("synthetic R65F selector failure");
    return null;
  },
  createDocumentFragment() {
    return new NodeLike();
  },
  createTextNode(text) {
    const node = new NodeLike();
    node.textContent = String(text);
    return node;
  },
  createElement() {
    return new Element();
  }
};
globalThis.document = documentImpl;

wasm.bridgeCommandSetOptionR7("op264", "customAlpha", true);
wasm.bridgeCommandSetOptionR7("op264", "customBeta", false);
wasm.bridgeCommandSetOptionR7("op264-other", "customAlpha", false);
op264OtherAlpha.checked = true;
op264OtherAlpha.title = "untouched";
wasm.bridgeRefreshInlineCommandTogglesR7("op264");
const inlineCommandToggleOwnership = {
  initial: {
    alphaChecked: Boolean(op264MainAlpha.checked),
    alphaAria: String(op264MainAlpha.attributes["aria-label"] || ""),
    alphaTitle: String(op264MainAlpha.title || ""),
    alphaIsOn: op264MainAlpha.classList.contains("is-on"),
    alphaIsOff: op264MainAlpha.classList.contains("is-off"),
    betaChecked: Boolean(op264MainBeta.checked),
    betaAria: String(op264MainBeta.attributes["aria-label"] || ""),
    betaTitle: String(op264MainBeta.title || ""),
    betaIsOn: op264MainBeta.classList.contains("is-on"),
    betaIsOff: op264MainBeta.classList.contains("is-off"),
    otherChecked: Boolean(op264OtherAlpha.checked),
    otherTitle: String(op264OtherAlpha.title || "")
  },
  toggledEnabled: wasm.bridgeCommandToggleOptionR7("op264", "customAlpha"),
  afterToggle: {
    alphaChecked: Boolean(op264MainAlpha.checked),
    alphaAria: String(op264MainAlpha.attributes["aria-label"] || ""),
    alphaTitle: String(op264MainAlpha.title || ""),
    alphaIsOn: op264MainAlpha.classList.contains("is-on"),
    alphaIsOff: op264MainAlpha.classList.contains("is-off"),
    otherChecked: Boolean(op264OtherAlpha.checked),
    otherTitle: String(op264OtherAlpha.title || "")
  }
};

const inprocessNodeOwnerGuardAlerts = [];
globalThis.alert = (message) => {
  inprocessNodeOwnerGuardAlerts.push(String(message || ""));
};

mainnetNodeMode.value = "external";
transportOwnerStatus = "role=node;network=mainnet;running=true;readiness=READY";
const guardExternalCallsBefore = transportCalls.length;
const guardExternal = await wasm.bridgeV7BlockInprocessIfNodeOwnerRunning("mainnet");
const guardExternalCalls = transportCalls.length - guardExternalCallsBefore;

mainnetNodeMode.value = "inprocess";
const guardRunningCallsBefore = transportCalls.length;
const guardRunning = await wasm.bridgeV7BlockInprocessIfNodeOwnerRunning("mainnet");
const guardRunningCalls = transportCalls.length - guardRunningCallsBefore;
const guardRunningCall = transportCalls.at(-1) || {};

transportOwnerStatus = "role=node;network=mainnet;running=false;readiness=READY";
const guardStoppedCallsBefore = transportCalls.length;
const guardStopped = await wasm.bridgeV7BlockInprocessIfNodeOwnerRunning("mainnet");
const guardStoppedCalls = transportCalls.length - guardStoppedCallsBefore;

const inprocessNodeOwnerGuard = {
  external: guardExternal,
  externalCalls: guardExternalCalls,
  running: guardRunning,
  runningCalls: guardRunningCalls,
  stopped: guardStopped,
  stoppedCalls: guardStoppedCalls,
  command: String(guardRunningCall.command || ""),
  network: String(guardRunningCall.network || ""),
  runtimeRole: String(guardRunningCall.runtimeRole || ""),
  alert: String(inprocessNodeOwnerGuardAlerts.at(-1) || "")
};
mainnetNodeMode.value = "external";
transportOwnerStatus = "transport-ok";

const mainnetMinerKey = wasm.bridgeElementId("mainnet", "internalCpuMiner");
const previousMainnetMiner = elements.get(mainnetMinerKey) || null;
const mainnetMiner = previousMainnetMiner || new Element();
const previousMainnetMinerChecked = Boolean(mainnetMiner.checked);
mainnetMiner.checked = true;
elements.set(mainnetMinerKey, mainnetMiner);
const startOptionsMainnet = wasm.bridgeStartOptions("mainnet");
mainnetMiner.checked = previousMainnetMinerChecked;
if (!previousMainnetMiner) elements.delete(mainnetMinerKey);

function op261Field(name, value = "", checked = false) {
  const field = new Element();
  field.value = String(value);
  field.checked = Boolean(checked);
  elements.set(wasm.bridgeElementId("op261", name), field);
  return field;
}
op261Field("config", "C:\\bridge-op261.toml");
op261Field("internalCpuMiner", "", true);
op261Field("internalCpuMinerAddress", "kaspa:op261-address");
const op261Threads = op261Field("internalCpuMinerThreads", "");
op261Field("internalCpuMinerThrottleMs", "250");
op261Field("internalCpuMinerTemplatePollMs", "");
wasm.bridgeCommandSetOptionR7("op261", "config", true);
const startOptionsEnabled = wasm.bridgeStartOptions("op261");
wasm.bridgeCommandSetOptionR7("op261", "config", false);
const startOptionsConfigDisabled = wasm.bridgeStartOptions("op261");
wasm.bridgeCommandSetOptionR7("op261", "config", true);
op261Threads.value = "257";
let startOptionsInvalidError = "";
try {
  wasm.bridgeStartOptions("op261");
} catch (error) {
  startOptionsInvalidError = String(error?.message || error || "");
}
op261Threads.value = "";
const startOptionsOwnership = {
  mainnet: startOptionsMainnet,
  enabled: startOptionsEnabled,
  configDisabled: startOptionsConfigDisabled,
  invalidError: startOptionsInvalidError
};

const previewMessageClasses = new Set();
const previewMessageNode = new Element();
previewMessageNode.classList = {
  contains(name) { return previewMessageClasses.has(String(name)); },
  toggle(name, force) {
    const key = String(name);
    const enabled = force === undefined ? !previewMessageClasses.has(key) : Boolean(force);
    if (enabled) previewMessageClasses.add(key);
    else previewMessageClasses.delete(key);
    return enabled;
  },
  add(name) { previewMessageClasses.add(String(name)); },
  remove(name) { previewMessageClasses.delete(String(name)); }
};
elements.set(wasm.bridgeElementId("op262", "previewStatus"), previewMessageNode);
const previewMessageMissing = wasm.bridgePreviewMessage("op262-missing", "Missing", false);
const previewMessageValidatingResult = wasm.bridgePreviewMessage(
  "op262",
  "Validating effective settings...",
  false
);
const previewMessageValidating = {
  result: previewMessageValidatingResult,
  text: String(previewMessageNode.textContent || ""),
  tone: String(previewMessageNode.dataset.statusTone || ""),
  errorClass: previewMessageNode.classList.contains("kgw-field-error")
};
const previewMessageErrorResult = wasm.bridgePreviewMessage("op262", "Preview failed", true);
const previewMessageError = {
  result: previewMessageErrorResult,
  text: String(previewMessageNode.textContent || ""),
  tone: String(previewMessageNode.dataset.statusTone || ""),
  errorClass: previewMessageNode.classList.contains("kgw-field-error")
};
const previewMessageVerifiedResult = wasm.bridgePreviewMessage("op262", "Preview verified", false);
const previewMessageVerified = {
  result: previewMessageVerifiedResult,
  text: String(previewMessageNode.textContent || ""),
  tone: String(previewMessageNode.dataset.statusTone || ""),
  errorClass: previewMessageNode.classList.contains("kgw-field-error")
};
const previewMessageOwnership = {
  missing: previewMessageMissing,
  validating: previewMessageValidating,
  error: previewMessageError,
  verified: previewMessageVerified
};

let defaultPathUpdateCalls = 0;
let defaultPathUpdateNet = "";
const defaultPathContextCallsBefore = transportCalls.filter(
  (call) => call.command === "kgw_settings_context_v1"
).length;
wasm.bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(
  "mainnet",
  (net) => {
    defaultPathUpdateCalls += 1;
    defaultPathUpdateNet = String(net || "");
  }
);
for (let attempt = 0; attempt < 8 && defaultPathUpdateCalls === 0; attempt += 1) {
  await new Promise((resolve) => setTimeout(resolve, 0));
}
const defaultPathContextCalls = transportCalls.filter(
  (call) => call.command === "kgw_settings_context_v1"
);
const defaultPathOwnership = {
  appdir: String(elements.get("bridge-mainnet-appdir")?.value || ""),
  appdirTitle: String(elements.get("bridge-mainnet-appdir")?.title || ""),
  mirror: String(elements.get("bridge-mainnet-inprocessAppdirMirror")?.value || ""),
  mirrorTitle: String(elements.get("bridge-mainnet-inprocessAppdirMirror")?.title || ""),
  updateCalls: defaultPathUpdateCalls,
  updateNet: defaultPathUpdateNet,
  contextDelta: defaultPathContextCalls.length - defaultPathContextCallsBefore,
  contextNetwork: String(defaultPathContextCalls.at(-1)?.network || "")
};

const currentNodeModeOwnership = {
  direct: wasm.bridgeCurrentNodeModeFromUiR65F("r65f-direct"),
  primaryPanel: wasm.bridgeCurrentNodeModeFromUiR65F("r65f-primary"),
  secondaryPanel: wasm.bridgeCurrentNodeModeFromUiR65F("r65f-secondary"),
  directEmptyPrecedence: wasm.bridgeCurrentNodeModeFromUiR65F("r65f-precedence"),
  missing: wasm.bridgeCurrentNodeModeFromUiR65F("r65f-missing"),
  selectorError: wasm.bridgeCurrentNodeModeFromUiR65F("r65f-error")
};

const bridgeLogOutput = elements.get("bridge-mainnet-logOutput");
const bridgeLogParent = new Element();
const bridgeLogToolbar = new Element();
bridgeLogParent.querySelector = (selector) =>
  String(selector).includes("bridge-log-toolbar") ? bridgeLogToolbar : null;
bridgeLogOutput.parentElement = bridgeLogParent;
bridgeLogOutput.scrollHeight = 321;
bridgeLogOutput.scrollTop = 7;

const bridgeTestnetLogOutput = new Element();
elements.set("bridge-testnet10-logOutput", bridgeTestnetLogOutput);
const bridgeTestnetLogParent = new Element();
bridgeTestnetLogParent.appendChild(bridgeTestnetLogOutput);

wasm.bridgeInstallLogAutoScrollControls();
const installedLogAutoScrollLabel = bridgeLogToolbar.children[0] || null;
const installedLogAutoScrollCheckbox =
  installedLogAutoScrollLabel?.children?.[0] || null;
const installedLogAutoScrollText =
  installedLogAutoScrollLabel?.children?.[1] || null;
const installedTestnetLogAutoScrollLabel =
  bridgeTestnetLogParent.children[0] || null;
const installedTestnetLogAutoScrollCheckbox =
  installedTestnetLogAutoScrollLabel?.children?.[0] || null;
const installedTestnetLogOutputAfter =
  bridgeTestnetLogParent.children[1] || null;
const logAutoScrollInstallerOwnership = {
  toolbarChildren: bridgeLogToolbar.children.length,
  labelClass: String(installedLogAutoScrollLabel?.className || ""),
  scope: String(
    installedLogAutoScrollLabel?.attributes?.["data-kgw-log-autoscroll"] || ""
  ),
  title: String(installedLogAutoScrollLabel?.attributes?.title || ""),
  checkboxType: String(installedLogAutoScrollCheckbox?.type || ""),
  checkboxId: String(installedLogAutoScrollCheckbox?.id || ""),
  checkboxChecked: Boolean(installedLogAutoScrollCheckbox?.checked),
  spanText: String(installedLogAutoScrollText?.textContent || ""),
  changeListener: typeof installedLogAutoScrollCheckbox?.listeners?.change === "function",
  fallbackChildren: bridgeTestnetLogParent.children.length,
  fallbackInsertedBeforeOutput:
    installedTestnetLogOutputAfter === bridgeTestnetLogOutput,
  fallbackLabelClass: String(installedTestnetLogAutoScrollLabel?.className || ""),
  fallbackCheckboxId: String(installedTestnetLogAutoScrollCheckbox?.id || "")
};
bridgeLogOutput.scrollTop = 11;
installedLogAutoScrollCheckbox.checked = false;
installedLogAutoScrollCheckbox.listeners.change({ isTrusted: true });
const logAutoScrollInstallerDisabled = {
  stored: innerTabStore.get("kgw.bridge.log.autoscroll.mainnet") || "",
  scrollTop: bridgeLogOutput.scrollTop,
  traceCommand: String(transportCalls.at(-1)?.command || "")
};
installedLogAutoScrollCheckbox.checked = true;
installedLogAutoScrollCheckbox.listeners.change({ isTrusted: false });
const logAutoScrollInstallerEnabled = {
  stored: innerTabStore.get("kgw.bridge.log.autoscroll.mainnet") || "",
  scrollTop: bridgeLogOutput.scrollTop,
  traceCommand: String(transportCalls.at(-1)?.command || "")
};

// Preserve the independent OP232 persistence/scroll smoke baseline after the
// OP233 DOM-installer event smoke mutates the same synthetic output element.
bridgeLogOutput.scrollTop = 7;
const logAutoScrollDefaultMainnet = wasm.bridgeLogAutoScrollEnabled("mainnet");
wasm.bridgeSetLogAutoScroll("mainnet", false);
const logAutoScrollDisabledMainnet = wasm.bridgeLogAutoScrollEnabled("mainnet");
const logAutoScrollStoredAfterDisable =
  innerTabStore.get("kgw.bridge.log.autoscroll.mainnet") || "";
const logAutoScrollScrollAfterDisable = bridgeLogOutput.scrollTop;
wasm.bridgeSetLogAutoScroll("mainnet", true);
const logAutoScrollEnabledMainnet = wasm.bridgeLogAutoScrollEnabled("mainnet");
const logAutoScrollStoredAfterEnable =
  innerTabStore.get("kgw.bridge.log.autoscroll.mainnet") || "";
const logAutoScrollScrollAfterEnable = bridgeLogOutput.scrollTop;
const logAutoScrollOwnership = {
  defaultMainnet: logAutoScrollDefaultMainnet,
  disabledMainnet: logAutoScrollDisabledMainnet,
  storedAfterDisable: logAutoScrollStoredAfterDisable,
  scrollAfterDisable: logAutoScrollScrollAfterDisable,
  enabledMainnet: logAutoScrollEnabledMainnet,
  storedAfterEnable: logAutoScrollStoredAfterEnable,
  scrollAfterEnable: logAutoScrollScrollAfterEnable,
  defaultTestnet10: wasm.bridgeLogAutoScrollEnabled("testnet10")
};

const local = new Map([["kgw.bridge.network.enabled.mainnet", "1"]]);
let invokeRuntime = async () => "";
const sandbox = {
  document: documentImpl,
  localStorage: {
    getItem(key) {
      return local.get(String(key)) || null;
    }
  },
  console,
  setTimeout,
  clearTimeout,
  CustomEvent: class CustomEvent {},
  runtimePresentation: wasm.runtimePresentation,
  runtimeObservationSummary: wasm.runtimeObservationSummary,
  applyStatusTone: wasm.applyStatusTone,
  renderStatusSummary: wasm.renderStatusSummary,
  wasmBridgeNetworkProfiles: wasm.bridgeNetworkProfiles,
  wasmBridgeNetworkProfile: wasm.bridgeNetworkProfile,
  wasmBridgeNetworkEnabled: wasm.bridgeNetworkEnabled,
  wasmBridgeById: wasm.bridgeById,
  wasmBridgeElementId: wasm.bridgeElementId,
  wasmBridgePreviewMessage: wasm.bridgePreviewMessage,
  wasmBridgeChecked: wasm.bridgeChecked,
  wasmBridgeStringifyRuntimeResult: wasm.bridgeStringifyRuntimeResult,
  wasmBridgeNormalizeRuntimeError: wasm.bridgeNormalizeRuntimeError,
  wasmBridgeParseRuntimeKeyValueResponse: wasm.bridgeParseRuntimeKeyValueResponse,
  wasmBridgeV7RuntimeRunningFromText: wasm.bridgeV7RuntimeRunningFromText,
  wasmBridgeV7BlockInprocessIfNodeOwnerRunning: wasm.bridgeV7BlockInprocessIfNodeOwnerRunning,
  wasmBridgeR51IsRunning: wasm.bridgeR51IsRunning,
  wasmBridgeR51SetRuntimeButtons: wasm.bridgeR51SetRuntimeButtons,
  wasmBridgeR51SetRuntimeUnknown: wasm.bridgeR51SetRuntimeUnknown,
  wasmBridgeSetRuntimeErrorV1: wasm.bridgeSetRuntimeErrorV1,
  wasmBridgeSetRuntimeActivityV1: wasm.bridgeSetRuntimeActivityV1,
  wasmBridgeMarkRestartRequiredV1: wasm.bridgeMarkRestartRequiredV1,
  wasmBridgeR51RefreshOne: wasm.bridgeR51RefreshOne,
  wasmBridgeR51StartLiveRefresh: wasm.bridgeR51StartLiveRefresh,
  wasmBridgeActiveRawLogInstanceId: wasm.bridgeActiveRawLogInstanceId,
  wasmBridgeRuntimeErrorFromStatus: wasm.bridgeRuntimeErrorFromStatus,
  wasmBridgeNormalizeNodeModeR65F: wasm.bridgeNormalizeNodeModeR65F,
  wasmBridgePreviewDeclaresInprocessR65F: wasm.bridgePreviewDeclaresInprocessR65F,
  wasmBridgeRuntimeCommandForAction: wasm.bridgeRuntimeCommandForAction,
  wasmBridgeRuntimeActionOutcome: wasm.bridgeRuntimeActionOutcome,
  wasmBridgeStartWasInprocessR65F: wasm.bridgeStartWasInprocessR65F,
  wasmBridgeCurrentNodeModeFromUiR65F: wasm.bridgeCurrentNodeModeFromUiR65F,
  wasmBridgeNodeMode: wasm.bridgeNodeMode,
  wasmBridgeSetOwnedNodeLockR65E: wasm.bridgeSetOwnedNodeLockR65E,
  wasmBridgeAssertNoPortConflictsR5: wasm.bridgeAssertNoPortConflictsR5,
  wasmBridgeTranslateRuntimeFeedback: wasm.bridgeTranslateRuntimeFeedback
};
sandbox.window = sandbox;
sandbox.globalThis = sandbox;
sandbox.bridgeInstances = { mainnet: [], testnet10: [], testnet13: [] };
sandbox.activeInstance = { mainnet: "", testnet10: "", testnet13: "" };
sandbox.wasmBridgeR51Panel = wasm.bridgeR51Panel;
sandbox.updateCommand = () => "--node-mode=external";
sandbox.kgwBridgeValidateForm = () => ({});
sandbox.c = () => false;
sandbox.confirmUserAction = async () => true;
sandbox.invokeBridgeIntegratedRuntime = (...args) => invokeRuntime(...args);
sandbox.kgwBridgeRuntimeOwnerTraceR64D = () => {};
sandbox.kgwBridgePreviewDeclaresInprocessR65F = () => false;
sandbox.kgwBridgeR51KickRawLogLiveR134E = () => {};

vm.createContext(sandbox);
vm.runInContext(selected, sandbox, { filename: request.sourceName });
const api = vm.runInContext("({ runtimeRunning: wasmBridgeV7RuntimeRunningFromText, isRunning: wasmBridgeR51IsRunning, setButtons: (net, running, transition = \"\", runtimeError = \"\", statusText = \"\") => wasmBridgeR51SetRuntimeButtons(String(net || \"\"), Boolean(running), String(transition || \"\"), String(runtimeError || \"\"), String(statusText || \"\")), setUnknown: (net, message, source = \"\") => wasmBridgeR51SetRuntimeUnknown(String(net || \"\"), String(message || \"\"), String(source || \"\")), refreshOne: (net, reason = \"harness\") => wasmBridgeR51RefreshOne(String(net || \"\"), String(reason || \"harness\"), kgwBridgeR51LiveRefreshCallbacksR257()), setError: wasmBridgeSetRuntimeErrorV1, setActivity: wasmBridgeSetRuntimeActivityV1, markRestart: (net) => wasmBridgeMarkRestartRequiredV1(String(net || \"\")), runAction: runBridgeIntegratedAction })", sandbox);

function snapshot() {
  return {
    policy: elements.get("bridge-mainnet-policyStatus").textContent,
    policyState: elements.get("bridge-mainnet-policyStatus").dataset.state || "",
    startDisabled: Boolean(start.disabled),
    stopDisabled: Boolean(stop.disabled),
    runtimeError: elements.get("bridge-mainnet-runtimeError").textContent,
    runtimeErrorHidden: Boolean(elements.get("bridge-mainnet-runtimeError").hidden),
    runtimeStatus: elements.get("bridge-mainnet-runtimeStatus").textContent
  };
}

invokeRuntime = async (command) => {
  if (String(command || "") === "kgw_kgw_runtime_logs_v1") {
    throw new Error("synthetic log refresh unavailable");
  }
  if (String(command || "") === "kgw_runtime_owner_status_v1") {
    return "role=bridge;network=mainnet;running=true;readiness=READY";
  }
  return "";
};
await api.refreshOne("mainnet", "harness-live-refresh");
const liveRefresh = snapshot();

api.setActivity("mainnet", "Bridge is RUNNING now", "stopped");
api.markRestart("mainnet");
const restartRunning = {
  text: elements.get("bridge-mainnet-settingsAuthority").textContent,
  restartRequired: elements.get("bridge-mainnet-settingsAuthority").dataset.restartRequired || ""
};
api.setActivity("mainnet", "Bridge is stopped.", "running");
api.markRestart("mainnet");
const restartStopped = {
  text: elements.get("bridge-mainnet-settingsAuthority").textContent,
  restartRequired: elements.get("bridge-mainnet-settingsAuthority").dataset.restartRequired || ""
};

// The Start/Stop lifecycle smoke controls invokeRuntime with unresolved
// synthetic promises. Disable only the action-settled refresh callback so
// background refresh cannot steal those synthetic resolvers.
sandbox.wasmBridgeR51RefreshOne = async () => undefined;

async function actionLifecycle() {
  const result = {};

  let resolveStart;
  invokeRuntime = async () => await new Promise(resolve => { resolveStart = resolve; });
  const startPromise = api.runAction("start", "mainnet");
  await Promise.resolve();
  await Promise.resolve();
  await new Promise(resolve => setImmediate(resolve));
  result.startPending = snapshot();
  resolveStart(request.startReady);
  await startPromise;
  result.startReady = snapshot();

  invokeRuntime = async () => { throw new Error(request.startError); };
  await api.runAction("start", "mainnet");
  result.startFailed = snapshot();

  let resolveStop;
  invokeRuntime = async () => await new Promise(resolve => { resolveStop = resolve; });
  api.setButtons("mainnet", true);
  const stopPromise = api.runAction("stop", "mainnet");
  await Promise.resolve();
  await new Promise(resolve => setImmediate(resolve));
  result.stopPending = snapshot();
  resolveStop(request.stopGraceful);
  await stopPromise;
  result.stopGraceful = snapshot();

  invokeRuntime = async () => request.stopForced;
  api.setButtons("mainnet", true);
  await api.runAction("stop", "mainnet");
  result.stopForced = snapshot();

  invokeRuntime = async () => request.stopFailed;
  api.setButtons("mainnet", true);
  await api.runAction("stop", "mainnet");
  result.stopFailed = snapshot();

  return result;
}

api.setButtons("mainnet", false, "starting");
const starting = snapshot();
api.setButtons("mainnet", true, "stopping");
const stopping = snapshot();
api.setButtons("mainnet", false);
api.setError("mainnet", "occupied listener port");
api.setActivity("mainnet", "Bridge start failed.");
const visibleFailure = snapshot();

const output = {
  r51StorageOwnership,
  defaultPathOwnership,
  portOnlyNormalizationOwnership,
  r95bPortNormalizationOwnership,
  instanceNetworkKeyOwnership,
  currentNodeModeOwnership,
  bridgeOwnedNodeLockOwnership,
  runtimeTranslationOwnership,
  innerTabOwnership,
  lastNetworkOwnership,
  logAutoScrollOwnership,
  logAutoScrollInstallerOwnership,
  logAutoScrollInstallerDisabled,
  logAutoScrollInstallerEnabled,
  transport: {
    available: Boolean(transportAvailable),
    result: String(transportResult || ""),
    calls: transportCalls
  },
  previewTransport: {
    result: String(previewTransportResult || ""),
    command: previewTransportCommand,
    unavailableError: previewUnavailableError
  },
  inprocessNodeOwnerGuard,
  inlineCommandToggleOwnership,
  startOptionsOwnership,
  previewMessageOwnership,
  runtimeRunning: {
    liveOnly: api.runtimeRunning("role=node;network=mainnet;running=true"),
    ready: api.runtimeRunning("role=node;network=mainnet;running=true;readiness=READY")
  },
  bridgeRunning: {
    liveOnly: api.isRunning("role=bridge;network=mainnet;running=true"),
    ready: api.isRunning("role=bridge;network=mainnet;running=true;readiness=READY")
  },
  liveRefresh,
  restartRunning,
  restartStopped,
  starting,
  stopping,
  visibleFailure,
  lifecycle: await actionLifecycle()
};
await writeFile(resultPath, JSON.stringify(output), "utf8");
"##;

fn slice_between<'a>(source: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    let start_index = source
        .find(start)
        .ok_or_else(|| format!("Bridge readiness source start marker missing: {start}"))?;
    let relative_end = source[start_index..]
        .find(end)
        .ok_or_else(|| format!("Bridge readiness source end marker missing: {end}"))?;
    let end_index = start_index + relative_end;
    if end_index <= start_index {
        return Err(format!(
            "Bridge readiness source markers reversed: {start} -> {end}"
        ));
    }
    Ok(&source[start_index..end_index])
}

fn selected_source(root: &Path) -> Result<String, String> {
    let source = fs::read_to_string(root.join(BRIDGE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_SOURCE}: {error}"))?;
    let mut selected = String::new();
    for &(start, end) in SLICES {
        selected.push_str(slice_between(&source, start, end)?);
        selected.push('\n');
    }
    Ok(selected)
}
fn request() -> Value {
    json!({
        "sourceName": BRIDGE_SOURCE,
        "startReady":
            "parallel-owned-self-worker started;role=bridge;network=mainnet;pid=42;running=true;readiness=READY",
        "startError": "occupied listener port",
        "stopGraceful":
            "parallel-owned-self-worker stopped;role=bridge;network=mainnet;running=false;graceful=true;forced=false",
        "stopForced":
            "parallel-owned-self-worker stopped;role=bridge;network=mainnet;running=false;graceful=false;forced=true;reason=graceful stop timed out",
        "stopFailed":
            "parallel-owned-self-worker stopped with graceful failure;role=bridge;network=mainnet;running=false;graceful=false;forced=false;stop_failed=true;stop_outcome=FAILED;reason=official shutdown failed"
    })
}

fn run_bridge(root: &Path, selected: &str) -> Result<Value, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-bridge-readiness-")
        .tempdir()
        .map_err(|error| format!("failed to create Bridge readiness tempdir: {error}"))?;
    let bridge_path = temp.path().join("bridge.mjs");
    let selected_path = temp.path().join("selected.js");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write Bridge readiness Node bridge: {error}"))?;
    fs::write(&selected_path, selected.as_bytes())
        .map_err(|error| format!("failed to write Bridge readiness selected source: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(&request())
            .map_err(|error| format!("failed to serialize Bridge readiness request: {error}"))?,
    )
    .map_err(|error| format!("failed to write Bridge readiness request: {error}"))?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&selected_path)
        .arg(&request_path)
        .arg(&result_path)
        .arg(root.join(WASM_JS))
        .arg(root.join(WASM_BIN))
        .arg(root.join(BRIDGE_SOURCE))
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch Bridge readiness Node bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Bridge readiness Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read Bridge readiness result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse Bridge readiness result: {error}"))
}

fn expect(actual: &Value, pointer: &str, expected: Value) -> Result<(), String> {
    let value = actual
        .pointer(pointer)
        .ok_or_else(|| format!("Bridge readiness result missing {pointer}"))?;
    if value != &expected {
        return Err(format!(
            "Bridge readiness mismatch at {pointer}: expected {expected}, got {value}"
        ));
    }
    Ok(())
}

fn verify_r51_storage_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for forbidden in [
        "kgwBridgeR51Load(",
        "kgwBridgeR51Store(",
        "bridgeR51Store as wasmBridgeR51Store",
        "wasmBridgeR51Store(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired or direct R51 JavaScript storage ownership remains: {forbidden}"
            ));
        }
    }
    for required in ["bridgeR51Load as wasmBridgeR51Load", "wasmBridgeR51Load("] {
        if !source.contains(required) {
            return Err(format!(
                "R51 generated-WASM load ownership missing: {required}"
            ));
        }
    }
    if source.matches("wasmBridgeR51Load(").count() != 1
        || !helpers.contains("js_name = bridgeR51Store")
        || !helpers.contains("js_name = bridgeR51Load")
        || !helpers.contains("\"kgw.bridge.direct.v51.\"")
    {
        return Err("R51 storage call counts, exports or prefix drifted".to_owned());
    }
    Ok(())
}

fn verify_r51_keys_ownership(source: &str, helpers: &str) -> Result<(), String> {
    if source.contains("kgwBridgeR51Keys") {
        return Err("Legacy JavaScript R51 keys owner remains".to_owned());
    }
    for required in ["bridgeR51Keys as wasmBridgeR51Keys", "wasmBridgeR51Keys()"] {
        if !source.contains(required) {
            return Err(format!("R51 keys Rust/WASM ownership missing: {required}"));
        }
    }
    if source.matches("wasmBridgeR51Keys()").count() != 1
        || !helpers.contains("js_name = bridgeR51Keys")
        || !helpers.contains("fn bridge_r51_key_texts()")
    {
        return Err("R51 keys call count or Rust export drifted".to_owned());
    }
    Ok(())
}

fn verify_r51_runtime_presentation_ownership(
    source: &str,
    runtime_core: &str,
) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeR51SetRuntimeButtons(",
        "function kgwBridgeR51SetRuntimeUnknown(",
        "kgwBridgeR51SetRuntimeButtons(",
        "kgwBridgeR51SetRuntimeUnknown(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R51 runtime-presentation JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    for required in [
        "bridgeR51SetRuntimeButtons as wasmBridgeR51SetRuntimeButtons",
        "bridgeR51SetRuntimeUnknown as wasmBridgeR51SetRuntimeUnknown",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge R51 runtime-presentation Rust/WASM binding missing: {required}"
            ));
        }
    }
    if source.matches("wasmBridgeR51SetRuntimeButtons(").count() != 8
        || source.matches("wasmBridgeR51SetRuntimeUnknown(").count() != 1
    {
        return Err("Bridge R51 runtime-presentation direct-call count drifted after OP290 moves network-policy refresh into Rust".to_owned());
    }
    for required in [
        "js_name = bridgeR51SetRuntimeButtons",
        "pub fn bridge_r51_set_runtime_buttons(",
        "js_name = bridgeR51SetRuntimeUnknown",
        "pub fn bridge_r51_set_runtime_unknown(",
        "Bridge: Reconciling | RPC/synchronization/mining: unknown",
        "Enable Profile in Settings",
        "Reconciling runtime state.",
    ] {
        if !runtime_core.contains(required) {
            return Err(format!(
                "Bridge R51 runtime-presentation Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_runtime_notice_ownership(source: &str, runtime_core: &str) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeSetRuntimeErrorV1(",
        "function kgwBridgeSetRuntimeActivityV1(",
        "kgwBridgeSetRuntimeErrorV1(",
        "kgwBridgeSetRuntimeActivityV1(",
        "function kgwBridgeR51RuntimePresentationCallbacksR256(",
        "kgwBridgeR51RuntimePresentationCallbacksR256()",
        "setRuntimeError:",
        "setRuntimeActivity:",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge runtime-notice JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    for required in [
        "bridgeSetRuntimeErrorV1 as wasmBridgeSetRuntimeErrorV1",
        "bridgeSetRuntimeActivityV1 as wasmBridgeSetRuntimeActivityV1",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge runtime-notice Rust/WASM binding missing: {required}"
            ));
        }
    }
    if source.matches("wasmBridgeSetRuntimeErrorV1(").count() != 12
        || source.matches("wasmBridgeSetRuntimeActivityV1(").count() != 9
    {
        return Err("Bridge runtime-notice direct-call count drifted after OP290 moves network-policy error handling into Rust".to_owned());
    }
    for required in [
        "fn set_runtime_error_inner(",
        "fn set_runtime_activity_inner(",
        "js_name = bridgeSetRuntimeErrorV1",
        "pub fn bridge_set_runtime_error_v1(",
        "js_name = bridgeSetRuntimeActivityV1",
        "pub fn bridge_set_runtime_activity_v1(",
        "\"runtimeError\".to_owned()",
        "\"runtimeStatus\".to_owned()",
        "\"runtimeErrorSource\"",
    ] {
        if !runtime_core.contains(required) {
            return Err(format!(
                "Bridge runtime-notice Rust owner contract missing: {required}"
            ));
        }
    }
    for retired_callback in ["\"setRuntimeError\"", "\"setRuntimeActivity\""] {
        if runtime_core.contains(retired_callback) {
            return Err(format!(
                "Retired Bridge runtime-notice callback dispatch remains in Rust: {retired_callback}"
            ));
        }
    }
    Ok(())
}

fn verify_mark_restart_required_ownership(source: &str, runtime_core: &str) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeMarkRestartRequiredV1(",
        "kgwBridgeMarkRestartRequiredV1(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge mark-restart-required JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    if source.contains("bridgeMarkRestartRequiredV1 as wasmBridgeMarkRestartRequiredV1")
        || source.contains("wasmBridgeMarkRestartRequiredV1(")
    {
        return Err("Bridge mark-restart-required JavaScript binding/calls must be fully retired after OP290 moves scoped settings events into Rust".to_owned());
    }
    for required in [
        "fn mark_restart_required_inner(",
        "js_name = bridgeMarkRestartRequiredV1",
        "pub fn bridge_mark_restart_required_v1(",
        "\"settingsAuthority\".to_owned()",
        "\"runtimeStatus\".to_owned()",
        ".to_ascii_lowercase()",
        ".contains(\"running\")",
        "Restart required to apply changed effective settings",
        "Effective settings apply on next Start",
        "\"restartRequired\"",
    ] {
        if !runtime_core.contains(required) {
            return Err(format!(
                "Bridge mark-restart-required Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_r51_live_refresh_ownership(source: &str, runtime_core: &str) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeR51MaybeActivityNotice(",
        "async function kgwBridgeR51RefreshOne(",
        "function kgwBridgeR51RefreshAll(",
        "function kgwBridgeR51StartLiveRefresh(",
        "KGW_BRIDGE_R51_LAST_STATUS",
        "KGW_BRIDGE_R51_LAST_LOGS",
        "KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE",
        "KGW_BRIDGE_R51_STATUS_IN_FLIGHT",
        "KGW_BRIDGE_R51_LOGS_IN_FLIGHT",
        "KGW_BRIDGE_R51_TIMER",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R51 live-refresh JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    for required in [
        "bridgeR51RefreshOne as wasmBridgeR51RefreshOne",
        "bridgeR51StartLiveRefresh as wasmBridgeR51StartLiveRefresh",
        "function kgwBridgeR51LiveRefreshCallbacksR257(",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge R51 live-refresh Rust/WASM binding missing: {required}"
            ));
        }
    }
    if source.matches("wasmBridgeR51RefreshOne(").count() != 5
        || source.matches("wasmBridgeR51StartLiveRefresh(").count() != 2
        || source
            .matches("kgwBridgeR51LiveRefreshCallbacksR257()")
            .count()
            != 9
    {
        return Err(
            "Bridge R51 live-refresh direct-call or callback-factory count drifted".to_owned(),
        );
    }
    for required in [
        "const R51_LIVE_REFRESH_MS: f64 = 700.0;",
        "static R51_LAST_STATUS:",
        "static R51_LAST_ACTIVITY_NOTICE:",
        "static R51_STATUS_IN_FLIGHT:",
        "static R51_LOGS_IN_FLIGHT:",
        "static R51_LIVE_TIMER:",
        "fn r51_maybe_activity_notice(",
        "fn r51_logs_task(",
        "fn r51_status_task(",
        "async fn r51_refresh_one_impl(",
        "fn r51_refresh_all_impl(",
        "fn r51_start_live_refresh_impl(",
        "js_name = bridgeR51RefreshOne",
        "pub async fn bridge_r51_refresh_one(",
        "js_name = bridgeR51RefreshAll",
        "pub fn bridge_r51_refresh_all(",
        "js_name = bridgeR51StartLiveRefresh",
        "pub fn bridge_r51_start_live_refresh(",
        "kgw_kgw_runtime_logs_v1",
        "kgw_runtime_owner_status_v1",
        "Bridge runtime failed after readiness.",
        "Status refresh failed: {}",
    ] {
        if !runtime_core.contains(required) {
            return Err(format!(
                "Bridge R51 live-refresh Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_inprocess_node_owner_guard_ownership(
    source: &str,
    start_trace: &str,
) -> Result<(), String> {
    for forbidden in [
        "async function kgwBridgeV7BlockInprocessIfNodeOwnerRunning(",
        "kgwBridgeV7BlockInprocessIfNodeOwnerRunning(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge in-process node-owner guard JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    if !source.contains(
        "bridgeV7BlockInprocessIfNodeOwnerRunning as wasmBridgeV7BlockInprocessIfNodeOwnerRunning",
    ) {
        return Err("Bridge in-process node-owner guard Rust/WASM binding missing".to_owned());
    }
    if source
        .matches("wasmBridgeV7BlockInprocessIfNodeOwnerRunning(")
        .count()
        != 1
    {
        return Err("Bridge in-process node-owner guard call count drifted".to_owned());
    }
    for required in [
        "INPROCESS_NODE_OWNER_RUNNING_MESSAGE",
        "async fn block_inprocess_if_node_owner_running_impl(",
        "js_name = bridgeV7BlockInprocessIfNodeOwnerRunning",
        "pub async fn bridge_v7_block_inprocess_if_node_owner_running(",
        "bridge_frontend_helpers::bridge_node_mode",
        "runtime_invoke_available()",
        "invoke_runtime_command_impl(\"kgw_runtime_owner_status_v1\"",
        "\"network\"",
        "\"runtimeRole\"",
        "\"node\"",
        "bridge_runtime_core::bridge_v7_runtime_running_from_text",
        "function(&window(), \"alert\")",
        "Cannot start bridge in in-process mode because the same-network node is already running. Stop the node first, or switch bridge node mode to External.",
    ] {
        if !start_trace.contains(required) {
            return Err(format!(
                "Bridge in-process node-owner guard Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_r95b_port_normalization_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeR95BStorageFieldId(",
        "function kgwBridgeR95BPreferredPort(",
        "function kgwBridgeR95BKnownStaleSequentialPort(",
        "function kgwBridgeR95BNormalizeNetworkPortValues(",
        "kgwBridgeR95BStorageFieldId(",
        "kgwBridgeR95BPreferredPort(",
        "kgwBridgeR95BKnownStaleSequentialPort(",
        "kgwBridgeR95BNormalizeNetworkPortValues(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R95B port-normalization JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    if !source.contains(
        "bridgeR95BNormalizeNetworkPortValues as wasmBridgeR95BNormalizeNetworkPortValues",
    ) {
        return Err("Bridge R95B port-normalization Rust/WASM binding missing".to_owned());
    }
    if source
        .matches("wasmBridgeR95BNormalizeNetworkPortValues(")
        .count()
        != 3
    {
        return Err("Bridge R95B port-normalization direct-call count drifted".to_owned());
    }
    for required in [
        "fn r95b_storage_field_id(",
        "fn r95b_preferred_port(",
        "fn r95b_known_stale_sequential_port(",
        "fn r95b_display_port_syntax(",
        "fn r95b_same_port(",
        "fn r95b_normalize_network_port_values(",
        "js_name = bridgeR95BNormalizeNetworkPortValues",
        "pub fn bridge_r95b_normalize_network_port_values(",
        "bridge_static_port_profile_r91",
        r#"("testnet10", "stratumPort") => "5556""#,
        r#"("testnet10", "promPort") => "2113""#,
        r#"("testnet13", "stratumPort") => "5557""#,
        r#"("testnet13", "promPort") => "2114""#,
        r#""displayOnly""#,
        r#""bridge-r51-r95b-settings-owner""#,
        r#""r98-normalize-port-only-display-values""#,
        "bridge_small_owner_trace_r44d(",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge R95B port-normalization Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_start_options_ownership(
    source: &str,
    helpers: &str,
    instance_settings: &str,
) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeStartOptions(",
        "kgwBridgeStartOptions(",
        "bridgeStartOptions as wasmBridgeStartOptions",
        "wasmBridgeStartOptions(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge start-options JavaScript ownership remains after OP270: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeBuildApplyPayloadUi",
        "crate::bridge_instance_settings::bridge_start_options(",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge start-options Rust caller contract missing after OP270: {required}"
            ));
        }
    }
    for required in [
        "fn bridge_start_options_inner(",
        "js_name = bridgeStartOptions",
        "pub fn bridge_start_options(",
        "bridge_command_options::bridge_has_config",
        "bridge_frontend_helpers::bridge_checked",
        "net != \"mainnet\"",
        "\"configFile\"",
        "\"internalCpuMiner\"",
        "\"internalCpuMinerAddress\"",
        "\"internalCpuMinerThreads\"",
        "\"internalCpuMinerThrottleMs\"",
        "\"internalCpuMinerTemplatePollMs\"",
        "\"CPU threads\"",
        "\"CPU throttle\"",
        "\"Template poll interval\"",
        "Some(1)",
        "60_000",
    ] {
        if !instance_settings.contains(required) {
            return Err(format!(
                "Bridge start-options Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_network_panel_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeRenderNetworkPanelUi as wasmBridgeRenderNetworkPanelUi",
        "// KGW_BRIDGE_NETWORK_PANEL_RUST_OWNER_V1",
        "function renderNetworkPanel(net, index) {",
        "return wasmBridgeRenderNetworkPanelUi(net || {}, Number(index) || 0, {",
        "renderSections: (profile) => renderSections(profile)",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge network-panel Rust/WASM binding missing: {required}"
            ));
        }
    }

    let wrapper = slice_between(source, "function renderNetworkPanel(net, index) {", "\n}")?;
    for forbidden in [
        "wasmBridgeResolveInnerTab(",
        "wasmBridgeNetworkPolicyMessage(",
        "wasmBridgeElementId(",
        "kgw-network-policy",
        "kgw-preview-row",
        "kgw-monitor-state",
        "data-bridge-inner-tab=",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge network-panel JavaScript rendering remains: {forbidden}"
            ));
        }
    }

    for forbidden in [
        "bridgeResolveInnerTab as wasmBridgeResolveInnerTab",
        "bridgeNetworkPolicyMessage as wasmBridgeNetworkPolicyMessage",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge network-panel-only JavaScript binding remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeRenderNetworkPanelUi",
        "pub fn bridge_render_network_panel_ui(",
        "bridge_resolve_inner_tab(net.clone())",
        "bridge_network_enabled(net.clone())",
        "bridge_network_policy_message(net.clone())",
        "bridge_element_id(net.clone(), \"policyStatus\".to_owned())",
        "\"renderSections\"",
        "kgw-experimental-badge",
        "data-bridge-inner-tab=\"log\"",
        "data-bridge-inner-tab=\"settings\"",
        "data-bridge-action=\"start\"",
        "data-bridge-action=\"save-settings\"",
        "data-bridge-action=\"monitor-next\"",
        "data-testid=\"kgw-bridge-log-output-{net}\"",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge network-panel Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_render_all_networks_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeRenderAllNetworksUi as wasmBridgeRenderAllNetworksUi",
        "// KGW_BRIDGE_RENDER_ALL_NETWORKS_RUST_OWNER_V1",
        "function renderAllNetworks(root) {",
        "return wasmBridgeRenderAllNetworksUi(root, {",
        "renderNetworkPanel: (profile, index) => renderNetworkPanel(profile, index)",
        "installSettingsLayout: (targetRoot) => installSettingsLayout(targetRoot)",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge render-all-networks Rust/WASM binding missing: {required}"
            ));
        }
    }
    let wrapper = slice_between(source, "function renderAllNetworks(root) {", "\n}")?;
    for forbidden in [
        "querySelector(\"#bridgeNetworkPanels\")",
        "BRIDGE_NETWORKS.map(",
        "host.innerHTML =",
        "setTimeout(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge render-all-networks JavaScript orchestration remains: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeRenderAllNetworksUi",
        "pub fn bridge_render_all_networks_ui(",
        "query_bridge(&root, \"#bridgeNetworkPanels\")",
        "for (index, profile) in NETWORKS.iter().enumerate()",
        "set(&host, \"innerHTML\"",
        "\"renderNetworkPanel\"",
        "\"installSettingsLayout\"",
        "bridge_install_log_auto_scroll_controls();",
        "\"kgwInstallBridgeLogScopedControlsV29\"",
        "\"setTimeout\"",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge render-all-networks Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_instance_click_owner_ownership(source: &str, instance_ui: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallInstanceContainerOwnerR11 as wasmBridgeInstallInstanceContainerOwnerR11",
        "// KGW_BRIDGE_INSTANCE_CLICK_OWNER_RUST_OWNER_V1",
        "function bridgeInstallInstanceContainerOwnerR11(container, net) {",
        "return wasmBridgeInstallInstanceContainerOwnerR11(",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge instance click-owner Rust/WASM binding missing: {required}"
            ));
        }
    }
    let wrapper = slice_between(
        source,
        "function bridgeInstallInstanceContainerOwnerR11(container, net) {",
        "\n}",
    )?;
    for forbidden in [
        "container.onclick =",
        "event.preventDefault()",
        "event.stopPropagation()",
        "control.dataset.bridgeAction",
        "wasmBridgeInstanceNetworkKeyR15(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge instance click-owner JavaScript orchestration remains: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeInstallInstanceContainerOwnerR11",
        "pub fn bridge_install_instance_container_owner_r11(",
        "KGW_BRIDGE_INSTANCES_REBUILD_CLICK_OWNER_R11",
        "r45d-bridge-instance-control-click",
        "\"add-instance\" | \"select-instance\" | \"remove-instance\"",
        "preventDefault",
        "stopPropagation",
        "bridge_render_raw_log_buffer(",
        "\"removeInstance\"",
    ] {
        if !instance_ui.contains(required) {
            return Err(format!(
                "Bridge instance click-owner Rust contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_visible_instance_owners_ownership(source: &str, instance_ui: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallAllVisibleInstanceContainerOwnersR11 as wasmBridgeInstallAllVisibleInstanceContainerOwnersR11",
        "// KGW_BRIDGE_VISIBLE_INSTANCE_OWNERS_RUST_OWNER_V1",
        "function bridgeInstallAllVisibleInstanceContainerOwnersR11(root) {",
        "return wasmBridgeInstallAllVisibleInstanceContainerOwnersR11({",
        "installInstanceContainerOwner: (container, targetNet) =>",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge visible-instance-owner Rust/WASM binding missing: {required}"
            ));
        }
    }

    let wrapper = slice_between(
        source,
        "function bridgeInstallAllVisibleInstanceContainerOwnersR11(root) {",
        "\n}",
    )?;
    for forbidden in [
        "for (const profile of BRIDGE_NETWORKS)",
        "wasmBridgeById(",
        "wasmBridgeElementId(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge visible-instance-owner JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeInstallAllVisibleInstanceContainerOwnersR11",
        "pub fn bridge_install_all_visible_instance_container_owners_r11(",
        "for net in [\"mainnet\", \"testnet10\", \"testnet13\"]",
        "bridge_element_id(net.to_owned(), \"instances\".to_owned())",
        "bridge_by_id(container_id)",
        "\"installInstanceContainerOwner\"",
        "installed += 1",
    ] {
        if !instance_ui.contains(required) {
            return Err(format!(
                "Bridge visible-instance-owner Rust contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_instance_refresh_ownership(source: &str, instance_ui: &str) -> Result<(), String> {
    for required in [
        "bridgeRefreshInstancesUi as wasmBridgeRefreshInstancesUi",
        "// KGW_BRIDGE_INSTANCE_REFRESH_RUST_OWNER_V1",
        "function bridgeRefreshInstances(net) {",
        "return wasmBridgeRefreshInstancesUi(",
        "decorateSettingsFields: (container) => decorateSettingsFields(container)",
        "installInstanceContainerOwner: (container, targetNet) =>",
        "updateCommand: (targetNet) => updateCommand(String(targetNet || \"\"))",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge instance-refresh Rust/WASM binding missing: {required}"
            ));
        }
    }

    let wrapper = slice_between(source, "function bridgeRefreshInstances(net) {", "\n}")?;
    for forbidden in [
        "wasmBridgeInstanceNetworkKeyR15(",
        "document.querySelector(",
        "container.innerHTML =",
        "container.id =",
        "renderInstances(net)",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge instance-refresh JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeRefreshInstancesUi",
        "pub fn bridge_refresh_instances_ui(",
        "bridge_instance_network_key_r15(",
        "bridge_element_id(net.clone(), \"instances\".to_owned())",
        "querySelector",
        "bridge_render_instances_ui(",
        "set(&container, \"innerHTML\"",
        "call1_required(&callbacks, \"decorateSettingsFields\"",
        "\"installInstanceContainerOwner\"",
        "call1_required(&callbacks, \"updateCommand\"",
    ] {
        if !instance_ui.contains(required) {
            return Err(format!(
                "Bridge instance-refresh Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_network_tabs_ownership(source: &str, helpers: &str) -> Result<(), String> {
    if !source.contains("bridgeInstallNetworkTabsUi as wasmBridgeInstallNetworkTabsUi")
        || !source.contains("// KGW_BRIDGE_NETWORK_TABS_RUST_OWNER_V1")
        || !source.contains("return wasmBridgeInstallNetworkTabsUi(root, {")
    {
        return Err("Bridge network-tabs Rust/WASM binding missing".to_owned());
    }
    let wrapper = slice_between(source, "function installNetworkTabs(root) {", "\n}")?;
    if wrapper.contains("root.addEventListener(")
        || wrapper.contains("wasmBridgeNormalizeNetwork(")
        || wrapper.contains("querySelectorAll(")
    {
        return Err("Retired Bridge network-tabs JavaScript orchestration remains".to_owned());
    }
    for required in [
        "js_name = bridgeInstallNetworkTabsUi",
        "pub fn bridge_install_network_tabs_ui(",
        "r45d-bridge-network-tab-click",
        "kgwBridgeSelectNetworkTabR63",
        "kgwBridgeSelectNetworkTabR101W2",
        "bridge_r51_refresh_one(",
        "network-tab-{reason}+700ms",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge network-tabs Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_delegated_tabs_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallDelegatedTabsUi as wasmBridgeInstallDelegatedTabsUi",
        "// KGW_BRIDGE_DELEGATED_TABS_RUST_OWNER_V1",
        "function installDelegatedTabs(root) {",
        "return wasmBridgeInstallDelegatedTabsUi(root, activeInstance);",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge delegated-tabs Rust/WASM binding missing: {required}"
            ));
        }
    }
    let wrapper = slice_between(source, "function installDelegatedTabs(root) {", "\n}")?;
    for forbidden in [
        "root.addEventListener(",
        "querySelectorAll(",
        "wasmBridgeSaveInnerTab(",
        "wasmBridgeRenderRawLogBuffer(",
        "kgwBridgeExplicitTraceR27D(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge delegated-tabs JavaScript orchestration remains: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeInstallDelegatedTabsUi",
        "pub fn bridge_install_delegated_tabs_ui(",
        "[data-bridge-inner-tab]",
        "[data-bridge-inner-panel]",
        "[data-bridge-section-tab]",
        "[data-bridge-section-panel]",
        "[data-instance-tab]",
        "[data-instance-panel]",
        "r45d-bridge-inner-tab-click",
        "r45d-bridge-section-tab-click",
        "r45d-bridge-instance-tab-click",
        "bridge_render_raw_log_buffer(",
        "bridge_save_inner_tab(",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge delegated-tabs Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_action_event_owners_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallActionEventOwnersUi as wasmBridgeInstallActionEventOwnersUi",
        "wasmBridgeInstallActionEventOwnersUi(root, bridgeInstances, activeInstance, {",
        "updateCommand: (net) => updateCommand(String(net || \"\"))",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge action-event owners Rust/WASM binding missing: {required}"
            ));
        }
    }
    for forbidden in [
        "root.dataset.kgwBridgeInstancesCommandCheckboxOwnerR13B",
        "root.dataset.kgwBridgeCommandComposerInlineOwnerR7",
        "KGW_BRIDGE_COMMAND_CHECKBOX_FIRST_CLICK_FIX_TRACE_PATCH_R31",
        "KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D_ACTIONS",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge action-event JavaScript owner remains after OP288: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeInstallActionEventOwnersUi",
        "pub fn bridge_install_action_event_owners_ui(",
        "kgwBridgeInstancesCommandCheckboxOwnerR13B",
        "bridge_set_instance_command_option_ui_r13b(",
        "kgwBridgeCommandComposerInlineOwnerR7",
        "r31-bridge-command-checkbox-pointerdown",
        "r31-bridge-command-checkbox-change-begin",
        "r31-bridge-command-checkbox-click",
        "bridge_command_toggle_option_r7(",
        "bridge_refresh_inline_command_toggles_r7(",
        "kgwBridgeInprocessNodeTabsV12B",
        "[data-bridge-inprocess-node-tab]",
        "bridgeInprocessNodePanel",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge action-event Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_settings_event_owners_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallSettingsEventOwnersUi as wasmBridgeInstallSettingsEventOwnersUi",
        "wasmBridgeInstallSettingsEventOwnersUi(root, bridgeInstances, {",
        "runIntegratedAction: (action, net) => runBridgeIntegratedAction(",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge settings-event owners Rust/WASM binding missing: {required}"
            ));
        }
    }
    let actions = slice_between(source, "function installActions(root) {", "\n}")?;
    for forbidden in [
        "root.addEventListener(\"input\"",
        "root.addEventListener(\"change\"",
        "wasmBridgeMarkRestartRequiredV1(",
        "wasmBridgeSetNetworkEnabled(",
        "wasmBridgeNetworkEnabled(",
        "wasmBridgeNetworkProfile(",
    ] {
        if actions.contains(forbidden) {
            return Err(format!(
                "Retired Bridge settings-event JavaScript ownership remains after OP290: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeInstallSettingsEventOwnersUi",
        "pub fn bridge_install_settings_event_owners_ui(",
        "bridge_settings_net_from_element(",
        "bridge_mark_restart_required_v1(",
        "[data-bridge-network-enabled]",
        "confirm_user_action(",
        "bridge_set_network_enabled(",
        "bridge_r51_set_runtime_buttons(",
        "runIntegratedAction",
        "trusted-input",
        "programmatic-change",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge settings-event Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_root_action_click_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallRootActionClickOwnerUi as wasmBridgeInstallRootActionClickOwnerUi",
        "wasmBridgeInstallRootActionClickOwnerUi(root, {",
        "selectInstance: (net, instanceId) =>",
        "saveSettings: (net, button) =>",
        "runtimeAction: (action, net) =>",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge root action-click Rust/WASM binding missing: {required}"
            ));
        }
    }
    let actions = slice_between(source, "function installActions(root) {", "\n}")?;
    for forbidden in [
        "root.addEventListener(\"click\"",
        "const button = event.target",
        "const action = button.dataset.bridgeAction",
        "netFromElement(",
        "normalizeNet(",
    ] {
        if actions.contains(forbidden) {
            return Err(format!(
                "Retired Bridge root action-click JavaScript dispatcher remains after OP291: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeInstallRootActionClickOwnerUi",
        "pub fn bridge_install_root_action_click_owner_ui(",
        "[data-bridge-action]",
        "bridge_settings_net_from_element(",
        "r27d-action-click",
        "\"select-instance\"",
        "\"save-settings\"",
        "\"set-defaults\"",
        "\"restore-defaults\"",
        "\"copy-log\" | \"clear-log\"",
        "\"copy-command\" | \"copy-path\"",
        "\"start\" | \"stop\"",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge root action-click Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_port_event_owners_ownership(source: &str, port_ui: &str) -> Result<(), String> {
    for required in [
        "bridgeInstallPortEventOwnersUi as wasmBridgeInstallPortEventOwnersUi",
        "wasmBridgeInstallPortEventOwnersUi(root, bridgeInstances, activeInstance, {",
        "refreshInstances: (net) => bridgeRefreshInstances(String(net || \"\"))",
        "runtimeActivity: (net, message) =>",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge port-event owners Rust/WASM binding missing: {required}"
            ));
        }
    }
    for forbidden in [
        "root.dataset.kgwBridgePortConflictValidationOwnerR33",
        "root.dataset.kgwBridgePortAutofixOwnerR37",
        "wasmBridgeSchedulePortConflictValidationR33(",
        "wasmBridgeSchedulePortAutofixRefreshUiR37(",
        "wasmBridgeValidateAllPortConflictStatesR33(",
        "wasmBridgeInstallPortAutofixButtonUiR37(",
        "wasmBridgeApplyPortAutofixUiR37(",
        "wasmBridgeRefreshPortAutofixButtonsUiR37(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge port-event JavaScript ownership remains after OP289: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeInstallPortEventOwnersUi",
        "pub fn bridge_install_port_event_owners_ui(",
        "kgwBridgePortConflictValidationOwnerR33",
        "bridge_schedule_port_conflict_validation_ui_r33(",
        "bridge_schedule_port_autofix_refresh_ui_r37(",
        "bridge_validate_all_port_conflict_states_ui_r33(",
        "kgwBridgePortAutofixOwnerR37",
        "bridge_install_port_autofix_button_ui_r37(",
        "[data-bridge-action=\\\"auto-fix-ports-r37\\\"]",
        "bridge_apply_port_autofix_ui_r37(",
        "button-feedback",
        "No Fix Needed",
    ] {
        if !port_ui.contains(required) {
            return Err(format!(
                "Bridge port-event Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_inprocess_node_settings_renderer_ownership(
    source: &str,
    bridge_render: &str,
) -> Result<(), String> {
    for required in [
        "bridgeRenderInprocessNodeSettingsUi as wasmBridgeRenderInprocessNodeSettingsUi",
        "function renderInprocessNodeSettings(net) {",
        "return wasmBridgeRenderInprocessNodeSettingsUi(net || {});",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge in-process node-settings Rust/WASM binding missing: {required}"
            ));
        }
    }
    let wrapper = slice_between(source, "function renderInprocessNodeSettings(net) {", "\n}")?;
    for forbidden in [
        "const tabs = [",
        "document.createElement(",
        "wasmBridgeI18nTextR41(",
        "wasmBridgeCommandInlineToggleR7(",
        "wasmBridgeCardInput(",
        "wasmBridgeElementId(",
        "wasmBridgeEscapeHtml(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge in-process node-settings JavaScript renderer remains: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeRenderInprocessNodeSettingsUi",
        "pub fn bridge_render_inprocess_node_settings_ui(",
        "bridge.inprocessNodeSettings.tab.basic",
        "bridge.inprocessNodeSettings.tab.dangerous",
        "inprocessRpcListen",
        "inprocessRpcListenBorsh",
        "inprocessRpcListenJson",
        "inprocessUtxoIndex",
        "inprocessArchival",
        "inprocessListen",
        "inprocessAddPeer",
        "inprocessConnect",
        "inprocessDisableUpnp",
        "inprocessMaxInpeers",
        "inprocessOutpeers",
        "inprocessAsyncThreads",
        "inprocessPerfMetrics",
        "inprocessPerfMetricsIntervalSec",
        "inprocessLogLevel",
        "inprocessRamScale",
        "inprocessConfigfile",
        "inprocessYes",
        "inprocessUnsafeRpc",
        "inprocessOverrideParamsFile",
        "inprocessDevnet",
        "inprocessSimnet",
        "inprocessEnableUnsyncedMining",
        "Unsafe RPC exposes RPC beyond loopback.",
        "bridge_command_inline_toggle_r7(",
    ] {
        if !bridge_render.contains(required) {
            return Err(format!(
                "Bridge in-process node-settings Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_settings_sections_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeRenderSectionsUi as wasmBridgeRenderSectionsUi",
        "// KGW_BRIDGE_SETTINGS_SECTIONS_RUST_OWNER_V1",
        "function renderSections(net) {",
        "return wasmBridgeRenderSectionsUi(net || {}, {",
        "renderInprocessNodeSettings: (profile) => renderInprocessNodeSettings(profile)",
        "bridgeInstances,",
        "activeInstance",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge settings-sections Rust/WASM binding missing: {required}"
            ));
        }
    }
    let wrapper = slice_between(source, "function renderSections(net) {", "\n}")?;
    for forbidden in [
        "document.createElement(\"template\")",
        "new Map()",
        "renderSettingsTabs(",
        "wasmBridgeRenderRuntime(",
        "wasmBridgeRenderDifficulty(",
        "wasmBridgeRenderLogging(",
        "wasmBridgeRenderPorts(",
        "wasmBridgeRenderCpuMiner(",
        "wasmBridgeDifficultyDatalistR16C(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge settings-sections JavaScript orchestration remains: {forbidden}"
            ));
        }
    }
    for forbidden in [
        "renderSettingsTabs, installSettingsLayout",
        "bridgeDifficultyDatalistR16C as wasmBridgeDifficultyDatalistR16C",
        "bridgeRenderRuntime as wasmBridgeRenderRuntime",
        "bridgeRenderDifficulty as wasmBridgeRenderDifficulty",
        "bridgeRenderLogging as wasmBridgeRenderLogging",
        "bridgeRenderPorts as wasmBridgeRenderPorts",
        "bridgeRenderCpuMiner as wasmBridgeRenderCpuMiner",
        "bridgeRenderInstancesUi as wasmBridgeRenderInstancesUi",
        "function renderInstances(net) {",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge settings-section/render helper JavaScript binding remains: {forbidden}"
            ));
        }
    }
    for required in [
        "js_name = bridgeRenderSectionsUi",
        "pub fn bridge_render_sections_ui(",
        "crate::bridge_render::bridge_render_runtime(",
        "crate::bridge_render::bridge_render_logging(",
        "crate::bridge_render::bridge_render_difficulty(",
        "crate::bridge_render::bridge_render_ports(",
        "crate::bridge_render::bridge_render_cpu_miner(",
        "querySelectorAll",
        "bridge_take_setting_cards(",
        "\"renderInprocessNodeSettings\"",
        "crate::bridge_instance_ui::bridge_render_instances_ui(",
        "crate::bridge_render::bridge_difficulty_datalist_r16c()",
        "crate::settings_layout::render_tabs_native(",
        "Ungrouped Bridge settings:",
        "Raw stdout/stderr logs are available in Live Bridge Monitor.",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge settings-sections Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_instances_renderer_ownership(source: &str, instance_ui: &str) -> Result<(), String> {
    for forbidden in [
        "bridgeRenderInstancesUi as wasmBridgeRenderInstancesUi",
        "function renderInstances(net) {",
        "wasmBridgeRenderInstancesUi(",
        "bridgeInstances[net].map(",
        "wasmBridgeEnsureInstanceState(",
        "wasmBridgeInstanceCommandCheckboxFromInstancesR13B(",
        "wasmBridgeInstancePreviewTextR8B(",
        "wasmBridgeInstancePortPlaceholderR49(",
        "wasmBridgeInstancePromPlaceholderR49(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge instances-renderer JavaScript binding/ownership remains after OP283: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeRenderInstancesUi",
        "pub fn bridge_render_instances_ui(",
        "bridge_instance_network_key_r15(",
        "bridge_ensure_instance_state(",
        "bridge_instance_command_checkbox_from_instances_r13b(",
        "preview_text(&net, &instance)",
        "placeholder_value(&net, \"stratum\")",
        "placeholder_value(&net, \"prom\")",
        "bridge_difficulty_input_attrs_r16c(",
        "data-bridge-action=\"select-instance\"",
        "data-bridge-action=\"remove-instance\"",
        "data-bridge-action=\"add-instance\"",
        "data-bridge-instance-field=\"instancePort\"",
        "data-bridge-instance-field=\"instancePow2Clamp\"",
    ] {
        if !instance_ui.contains(required) {
            return Err(format!(
                "Bridge instances-renderer Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_instance_command_option_ownership(
    source: &str,
    command_options: &str,
) -> Result<(), String> {
    for required in [
        "bridgeSetInstanceCommandOptionUiR13B as wasmBridgeSetInstanceCommandOptionUiR13B",
        "function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {",
        "return wasmBridgeSetInstanceCommandOptionUiR13B(",
        "{ updateCommand: (targetNet) => updateCommand(String(targetNet || \"\")) }",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge instance command-option Rust/WASM binding missing: {required}"
            ));
        }
    }

    let wrapper = slice_between(
        source,
        "function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {",
        "\n}",
    )?;
    for forbidden in [
        "wasmBridgeSmallOwnerTraceR44D(",
        "wasmBridgeInstanceCommandSetOptionR13B(",
        "querySelectorAll(",
        "wasmBridgeSyncInstancePreviewRowsR8B(",
        "toggle.checked",
        "toggle.setAttribute(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge instance command-option JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeSetInstanceCommandOptionUiR13B",
        "pub fn bridge_set_instance_command_option_ui_r13b(",
        "bridge_instance_command_set_option_r13b(",
        "[data-bridge-instance-command-option-toggle-r13b]",
        "bridgeInstanceCommandOptionToggleR13b",
        "call1_required(&callbacks, \"updateCommand\"",
        "bridge_sync_instance_preview_rows_r8b(",
        "r29b-bridge-instance-command-checkbox-begin",
        "r29b-bridge-instance-command-checkbox-complete",
        "Included in command",
        "Excluded from command",
    ] {
        if !command_options.contains(required) {
            return Err(format!(
                "Bridge instance command-option Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_instance_mutation_ownership(source: &str, instance_settings: &str) -> Result<(), String> {
    for required in [
        "bridgeAddInstanceUi as wasmBridgeAddInstanceUi",
        "bridgeRemoveInstanceUi as wasmBridgeRemoveInstanceUi",
        "// KGW_BRIDGE_INSTANCE_MUTATION_RUST_OWNER_V1",
        "function addInstance(net) {",
        "return wasmBridgeAddInstanceUi(",
        "function removeInstance(net, instanceId) {",
        "return wasmBridgeRemoveInstanceUi(",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge instance-mutation Rust/WASM binding missing: {required}"
            ));
        }
    }

    let add_wrapper = slice_between(source, "function addInstance(net) {", "\n}")?;
    let remove_wrapper =
        slice_between(source, "function removeInstance(net, instanceId) {", "\n}")?;
    for forbidden in [
        "wasmBridgeSmallOwnerTraceR44D(",
        "wasmBridgeEnsureInstanceState(",
        "wasmBridgeCreateInstanceRecordR9(",
        "bridgeInstances[net].push(",
        "bridgeInstances[net] = bridgeInstances[net].filter(",
        ".findIndex(",
        "Math.max(",
    ] {
        if add_wrapper.contains(forbidden) || remove_wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge instance-mutation JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeAddInstanceUi",
        "pub fn bridge_add_instance_ui(",
        "js_name = bridgeRemoveInstanceUi",
        "pub fn bridge_remove_instance_ui(",
        "bridge_port_orchestration::bridge_ensure_instance_state(",
        "bridge_port_orchestration::bridge_create_instance_record_r9(",
        "instances.push(&next)",
        "set(&active_instance, &net, &next_id)",
        "input.length() <= 1",
        "removed_index: Option<u32>",
        "saturating_sub(1)",
        "call1_required(&callbacks, \"refreshInstances\"",
        "call1_required(&callbacks, \"updateCommand\"",
        "bridge_raw_log::bridge_render_raw_log_buffer(",
        "JsValue::from_str(\"r44d-owner-begin\")",
        "JsValue::from_str(\"r44d-owner-complete\")",
    ] {
        if !instance_settings.contains(required) {
            return Err(format!(
                "Bridge instance-mutation Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_instance_state_structured_reader_ownership(
    source: &str,
    instance_settings: &str,
) -> Result<(), String> {
    for required in [
        "// KGW_BRIDGE_INSTANCE_STATE_RUST_OWNER_V1",
        "function kgwBridgeR51ReadStructuredInstancesR253(net) {",
        "return wasmBridgeR51ReadStructuredInstances(",
        "bridgeR51ReadStructuredInstances as wasmBridgeR51ReadStructuredInstances",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge instance-state/structured-reader Rust/WASM binding missing: {required}"
            ));
        }
    }

    if source
        .matches("wasmBridgeR51ReadStructuredInstances(")
        .count()
        != 1
    {
        return Err("Bridge structured-reader direct-call count drifted".to_owned());
    }

    for forbidden in [
        "function bridgeReadInstanceState(",
        "bridgeNormalizeInstanceRecord as wasmBridgeNormalizeInstanceRecord",
        "bridgeReadInstanceField as wasmBridgeReadInstanceField",
        "bridgeAssignMissingInstancePortsR9 as wasmBridgeAssignMissingInstancePortsR9",
        "readInstanceState: (",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge instance-state JavaScript ownership remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeReadInstanceStateUi",
        "pub fn bridge_read_instance_state_ui(",
        "fn bridge_read_instance_state_impl(",
        "js_name = bridgeR51ReadStructuredInstances",
        "pub fn bridge_r51_read_structured_instances(\n    net: String,\n    bridge_instances: JsValue,\n    active_instance: JsValue,\n) -> JsValue {",
        "bridge_instance_ui::bridge_read_instance_field(",
        "bridge_port_orchestration::bridge_assign_missing_instance_ports_r9(",
        "bridge_read_instance_state_impl(&net, bridge_instances, &fallback_id)",
        "\"instanceDiff\", \"2048\"",
        "\"instanceLogToFile\", \"not set\"",
        "\"instanceVarDiff\", \"not set\"",
        "\"instanceVarDiffStats\", \"not set\"",
        "\"instancePow2Clamp\", \"not set\"",
    ] {
        if !instance_settings.contains(required) {
            return Err(format!(
                "Bridge instance-state/structured-reader Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_command_preview_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeUpdateCommandUi as wasmBridgeUpdateCommandUi",
        "bridgeUpdateAllCommandsUi as wasmBridgeUpdateAllCommandsUi",
        "// KGW_BRIDGE_COMMAND_PREVIEW_RUST_OWNER_V1",
        "function updateCommand(net) {",
        "return wasmBridgeUpdateCommandUi(",
        "function updateAllCommands() {",
        "return wasmBridgeUpdateAllCommandsUi(",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge command-preview Rust/WASM binding missing: {required}"
            ));
        }
    }

    if source.matches("wasmBridgeUpdateCommandUi(").count() != 1 {
        return Err("Bridge update-command direct-call count drifted".to_owned());
    }
    if source.matches("wasmBridgeUpdateAllCommandsUi(").count() != 1 {
        return Err("Bridge update-all-commands direct-call count drifted".to_owned());
    }
    if source.contains("KGW_BRIDGE_PREVIEW_REQUESTS") {
        return Err("Retired Bridge preview request map remains in JavaScript".to_owned());
    }

    let update_wrapper = slice_between(source, "function updateCommand(net) {", "\n}")?;
    let update_all_wrapper = slice_between(source, "function updateAllCommands() {", "\n}")?;
    for forbidden in [
        "setTimeout(",
        "clearTimeout(",
        "wasmBridgePreparePreview(",
        "wasmBridgeReassignInstancePortsFromExternalRangeR91(",
        "wasmBridgeSyncInstancePreviewRowsR8B(",
        "kgwBridgeValidateForm(",
        "wasmBridgePreviewMessage(",
        "bridgeSyncModeControls(",
        "bridgeSyncAllModeControls(",
        "BRIDGE_NETWORKS.forEach(",
    ] {
        if update_wrapper.contains(forbidden) || update_all_wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge command-preview JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "struct BridgePreviewState",
        "static BRIDGE_PREVIEWS:",
        "fn clear_bridge_preview_timer(",
        "fn begin_bridge_preview_sequence(",
        "fn store_bridge_preview_timer(",
        "fn bridge_preview_sequence_is_current(",
        "async fn finish_bridge_preview_update(",
        "fn bridge_update_command_inner(",
        "js_name = bridgeUpdateCommandUi",
        "pub fn bridge_update_command_ui(",
        "js_name = bridgeUpdateAllCommandsUi",
        "pub fn bridge_update_all_commands_ui(",
        "js_name = bridgePreviewSequence",
        "pub fn bridge_preview_sequence(",
        "bridge_sync_mode_controls_ui(",
        "bridge_reassign_instance_ports_from_external_range_r91(",
        "bridge_sync_instance_preview_rows_r8b(",
        "bridge_validate_form_ui(",
        "bridge_build_apply_payload_ui(",
        "crate::bridge_start_trace::bridge_prepare_preview(",
        "bridge_preview_message_inner(",
        "\"clearTimeout\"",
        "\"setTimeout\"",
        "JsValue::from_f64(180.0)",
        "\"typed-effective-settings-preview\"",
        "\"effectiveSettings\"",
        "\"kgwBridgeCommandOwner\"",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge command-preview Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_preview_message_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgePreviewMessage(",
        "kgwBridgePreviewMessage(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge preview-message JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    if !source.contains("bridgePreviewMessage as wasmBridgePreviewMessage") {
        return Err("Bridge preview-message Rust/WASM binding missing".to_owned());
    }
    if source.matches("wasmBridgePreviewMessage(").count() != 2 {
        return Err(
            "Bridge preview-message must have exactly two remaining direct JavaScript calls after OP274 moves command-preview status ownership into Rust"
                .to_owned(),
        );
    }
    for required in [
        "fn bridge_preview_message_inner(",
        "js_name = bridgePreviewMessage",
        "pub fn bridge_preview_message(",
        "\"previewStatus\".to_owned()",
        "\"kgw-field-error\"",
        "message.starts_with(\"Validating\")",
        "\"error\"",
        "\"validating\"",
        "\"verified\"",
        "apply_status_tone_js",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge preview-message Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_inline_command_toggle_ownership(
    source: &str,
    command_options: &str,
) -> Result<(), String> {
    for forbidden in [
        "function kgwBridgeRefreshInlineCommandTogglesR7(",
        "function kgwBridgeToggleCommandOptionR7(",
        "kgwBridgeRefreshInlineCommandTogglesR7(",
        "kgwBridgeToggleCommandOptionR7(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge inline command-toggle JavaScript ownership remains: {forbidden}"
            ));
        }
    }

    if !source
        .contains("bridgeRefreshInlineCommandTogglesR7 as wasmBridgeRefreshInlineCommandTogglesR7")
    {
        return Err(
            "Bridge inline command-toggle remaining write-settings refresh binding is missing"
                .to_owned(),
        );
    }
    for forbidden in [
        "bridgeCommandToggleOptionR7 as wasmBridgeCommandToggleOptionR7",
        "wasmBridgeCommandToggleOptionR7(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Bridge inline command-toggle direct JavaScript event binding must remain retired after OP288: {forbidden}"
            ));
        }
    }
    if source
        .matches("wasmBridgeRefreshInlineCommandTogglesR7(")
        .count()
        != 1
    {
        return Err(
            "Bridge inline command-toggle remaining write-settings refresh call count drifted"
                .to_owned(),
        );
    }

    for required in [
        "fn refresh_inline_command_toggles_r7(",
        "js_name = bridgeRefreshInlineCommandTogglesR7",
        "pub fn bridge_refresh_inline_command_toggles_r7(",
        "js_name = bridgeCommandToggleOptionR7",
        "pub fn bridge_command_toggle_option_r7(",
        "[data-bridge-command-option-toggle-r7]",
        "bridgeCommandOptionToggleR7",
        "\"checked\"",
        "\"aria-label\"",
        "\"title\"",
        "\"is-on\"",
        "\"is-off\"",
        "refresh_inline_command_toggles_r7(&net);",
    ] {
        if !command_options.contains(required) {
            return Err(format!(
                "Bridge inline command-toggle Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_dependency_sync_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for forbidden in [
        "bridgeSyncDependencies as wasmBridgeSyncDependencies",
        "function kgwBridgeSyncDependencies(",
        "wasmBridgeSyncDependencies(",
        "bridgeFieldEnabled(name, values, options)",
        "setSettingFieldState(field,",
        "const values = kgwBridgeForm(net), options =",
        "toggle.disabled = Boolean(BRIDGE_MANAGED[name]",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge dependency-sync JavaScript ownership remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeSyncDependencies",
        "pub fn bridge_sync_dependencies(",
        "settings_bridge_field_enabled(",
        "[data-bridge-command-option-toggle-r7]",
        "[data-bridge-instance-command-option-toggle-r13b]",
        "bridgeCommandOptionToggleR7",
        "bridgeInstanceCommandOptionToggleR13b",
        "\"aria-labelledby\"",
        "settings_set_field_state(",
        "settings_decorate_fields(panel)",
        "bridge_dependency_toggle_disabled(",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge dependency-sync Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_mode_controls_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeSyncModeControlsUi as wasmBridgeSyncModeControlsUi",
        "// KGW_BRIDGE_MODE_CONTROLS_RUST_OWNER_V1",
        "function bridgeSyncModeControls(net) {",
        "return wasmBridgeSyncModeControlsUi(String(net || \"\"), bridgeInstances);",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge mode-controls Rust/WASM binding missing: {required}"
            ));
        }
    }

    let wrapper = slice_between(source, "function bridgeSyncModeControls(net) {", "\n}")?;
    for forbidden in [
        "wasmBridgeNetworkProfile(",
        "wasmBridgeHasConfig(",
        "wasmBridgeNodeMode(",
        "wasmBridgeChecked(",
        "bridgeSetDisabled(",
        "bridgeSyncInprocessNodeSettingsV12D(",
        "kgwBridgeSyncDependencies(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge mode-controls JavaScript orchestration remains in wrapper: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeSyncModeControlsUi",
        "pub fn bridge_sync_mode_controls_ui(",
        "Config mode is active. Clear --config to edit explicit CLI flags.",
        "Network identity is owned by the selected Mainnet/Testnet tab.",
        "In-process mode owns kaspad args after the -- separator.",
        "Enable --internal-cpu-miner first.",
        "bridge_sync_inprocess_node_settings_v12d(net.clone())",
        "bridge_sync_dependencies(net, bridge_instances)",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge mode-controls Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_command_orchestration_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeBuildCommandLinesUi as wasmBridgeBuildCommandLinesUi",
        "bridgeSyncAllModeControlsUi as wasmBridgeSyncAllModeControlsUi",
        "// KGW_BRIDGE_COMMAND_ORCHESTRATION_RUST_OWNER_V1",
        "return wasmBridgeSyncAllModeControlsUi(bridgeInstances);",
        "wasmBridgeBuildCommandLinesUi(String(net || \"\"), bridgeInstances, activeInstance)",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge command-orchestration Rust/WASM binding missing: {required}"
            ));
        }
    }

    for forbidden in [
        "bridgeBuildCommandLines as wasmBridgeBuildCommandLines",
        "wasmBridgeBuildCommandLines(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge command-orchestration JavaScript binding remains: {forbidden}"
            ));
        }
    }

    let all_wrapper = slice_between(source, "function bridgeSyncAllModeControls() {", "\n}")?;
    for forbidden in ["BRIDGE_NETWORKS.forEach(", "bridgeSyncModeControls("] {
        if all_wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge all-mode JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    let build_wrapper = slice_between(source, "function buildCommandLines(net) {", "\n}")?;
    for forbidden in [
        "bridgeSyncModeControls(",
        "bridgeInstances[net]",
        "wasmBridgeEnsureInstanceState(",
        "wasmBridgeBuildCommandLines(",
    ] {
        if build_wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge command-builder JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeSyncAllModeControlsUi",
        "pub fn bridge_sync_all_mode_controls_ui(",
        "for net in bridge_r51_key_texts()",
        "bridge_sync_mode_controls_ui(net.to_owned(), bridge_instances.clone())",
        "js_name = bridgeBuildCommandLinesUi",
        "pub fn bridge_build_command_lines_ui(",
        "crate::bridge_port_orchestration::bridge_ensure_instance_state(",
        "crate::bridge_command_builder::bridge_build_command_lines(",
        "Array::is_array(&instances_value)",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge command-orchestration Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_inprocess_mode_controls_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for forbidden in [
        "bridgeSetDisabledUi as wasmBridgeSetDisabledUi",
        "bridgeSyncInprocessNodeSettingsV12D as wasmBridgeSyncInprocessNodeSettingsV12D",
        "function bridgeSetDisabled(",
        "function bridgeSyncInprocessNodeSettingsV12D(",
        "wasmBridgeSetDisabledUi(",
        "wasmBridgeSyncInprocessNodeSettingsV12D(",
        "function bridgeControlCard(",
        "const fields = [\n    \"inprocessAppdirMirror\"",
        "section.classList.toggle(\"bridge-v12d-inprocess-inactive\"",
        "appdirMirror.value =",
        "networkArgs.value = profile.testnet",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge in-process mode-controls JavaScript orchestration remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeSetDisabledUi",
        "pub fn bridge_set_disabled_ui(",
        "fn bridge_control_card_inner(",
        "\"bridge-v7-mode-disabled\"",
        "js_name = bridgeSyncInprocessNodeSettingsV12D",
        "pub fn bridge_sync_inprocess_node_settings_v12d(",
        "\"bridge-v12d-inprocess-inactive\"",
        "\"bridge-v12d-inprocess-active\"",
        "\"kgwInprocessNodeActive\"",
        "\"inprocessAppdirMirror\"",
        "\"inprocessNetworkArgs\"",
        "\"inprocessOverrideParamsFile\"",
        "\"inprocessEnableUnsyncedMining\"",
        "Dangerous development-only kaspad flag is disabled on mainnet.",
        "Used only when Bridge Node Mode is In-Process.",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge in-process mode-controls Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_apply_payload_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeBuildApplyPayloadUi as wasmBridgeBuildApplyPayloadUi",
        "// KGW_BRIDGE_APPLY_PAYLOAD_RUST_OWNER_V1",
        "function buildApplyPayload(net, command) {",
        "return wasmBridgeBuildApplyPayloadUi(",
        "kgwBridgeR51ReadStructuredInstancesR253,",
        "buildCommandLines",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge apply-payload Rust/WASM binding missing: {required}"
            ));
        }
    }

    if source.matches("wasmBridgeBuildApplyPayloadUi(").count() != 1 {
        return Err("Bridge apply-payload direct-call count drifted".to_owned());
    }

    let wrapper = slice_between(source, "function buildApplyPayload(net, command) {", "\n}")?;
    for forbidden in [
        "nodeKind:",
        "bridgeKind:",
        "bridgeActiveInstanceId",
        "bridgeStructuredInstances",
        "effectiveNodeSettings:",
        "effectiveBridgeSettings:",
        "bridgeOptions:",
        "experimentalNetworkOptIn:",
        "wasmBridgeBuildUpstreamInstanceArg(",
        "wasmBridgeStartOptions(",
        "wasmBridgeAssertNoPortConflictsR5(",
    ] {
        if wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge apply-payload JavaScript orchestration remains in wrapper: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeBuildApplyPayloadUi",
        "pub fn bridge_build_apply_payload_ui(",
        "kgw_kgw_apply_node_settings_v1",
        "kgw_kgw_disable_network_v1",
        "kgw_runtime_owner_status_v1",
        "kgw_kgw_runtime_logs_v1",
        "crate::bridge_port_validation::bridge_assert_no_port_conflicts_r5(",
        "bridge_apply_payload_command_preview(",
        "bridge_apply_payload_structured_instances(",
        "bridge_apply_payload_active_record(",
        "bridge_apply_payload_active_instance_id(",
        "bridge_effective_inprocess_node_settings_checked(",
        "crate::bridge_instance_settings::bridge_effective_settings_v1(",
        "crate::bridge_instance_settings::bridge_start_options(",
        "crate::bridge_instance_settings::bridge_build_upstream_instance_arg(",
        "\"official-inprocess-node\"",
        "\"official-external-node\"",
        "\"experimentalNetworkOptIn\"",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge apply-payload Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_require_valid_settings_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeRequireValidSettingsUi as wasmBridgeRequireValidSettingsUi",
        "bridgeEffectiveInprocessNodeSettingsChecked as wasmBridgeEffectiveInprocessNodeSettingsChecked",
        "// KGW_BRIDGE_REQUIRE_VALID_SETTINGS_RUST_OWNER_V1",
        "return wasmBridgeEffectiveInprocessNodeSettingsChecked(String(net || \"\"), bridgeInstances);",
        "return wasmBridgeRequireValidSettingsUi(String(net || \"\"), bridgeInstances, activeInstance, kgwBridgeR51ReadStructuredInstancesR253);",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge require-valid-settings Rust/WASM binding missing: {required}"
            ));
        }
    }

    for call in [
        "wasmBridgeRequireValidSettingsUi(",
        "wasmBridgeEffectiveInprocessNodeSettingsChecked(",
    ] {
        if source.matches(call).count() != 1 {
            return Err(format!(
                "Bridge require-valid-settings direct-call count drifted: {call}"
            ));
        }
    }

    if source.contains("wasmBridgeEffectiveInprocessNodeSettings(") {
        return Err(
            "Retired Bridge require-valid-settings JavaScript orchestration remains: wasmBridgeEffectiveInprocessNodeSettings("
                .to_owned(),
        );
    }

    let effective_wrapper = slice_between(
        source,
        "function kgwBridgeEffectiveInprocessNodeSettings(net) {",
        "\n}",
    )?;
    for forbidden in [
        "kgwBridgeValidateForm(",
        "Object.keys(errors)",
        "throw new Error",
    ] {
        if effective_wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge effective-inprocess JavaScript orchestration remains in wrapper: {forbidden}"
            ));
        }
    }

    let require_wrapper = slice_between(
        source,
        "function kgwBridgeRequireValidSettings(net) {",
        "\n}",
    )?;
    for forbidden in [
        "kgwBridgeValidateForm(",
        "Object.keys(errors)",
        "throw new Error",
        "wasmBridgeAssertNoPortConflictsR5(",
        "kgwBridgeEffectiveInprocessNodeSettings(net);",
        "wasmBridgeEffectiveSettingsV1(",
    ] {
        if require_wrapper.contains(forbidden) {
            return Err(format!(
                "Retired Bridge require-valid-settings JavaScript orchestration remains in wrapper: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeRequireValidSettingsUi",
        "pub fn bridge_require_valid_settings_ui(",
        "js_name = bridgeEffectiveInprocessNodeSettingsChecked",
        "pub fn bridge_effective_inprocess_node_settings_checked(",
        "fn bridge_first_validation_error(",
        "crate::bridge_port_validation::bridge_assert_no_port_conflicts_r5(",
        "crate::bridge_instance_settings::bridge_effective_inprocess_node_settings(",
        "crate::bridge_instance_settings::bridge_effective_settings_v1(",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge require-valid-settings Rust owner contract missing: {required}"
            ));
        }
    }
    Ok(())
}

fn verify_full_form_validation_ownership(source: &str, helpers: &str) -> Result<(), String> {
    for required in [
        "bridgeValidateFormUi as wasmBridgeValidateFormUi",
        "// KGW_BRIDGE_FULL_FORM_VALIDATION_RUST_OWNER_V1",
        "return wasmBridgeValidateFormUi(String(net || \"\"), bridgeInstances, Boolean(focus));",
    ] {
        if !source.contains(required) {
            return Err(format!(
                "Bridge full-form validation Rust/WASM binding missing: {required}"
            ));
        }
    }

    if source.matches("wasmBridgeValidateFormUi(").count() != 1 {
        return Err("Bridge full-form validation direct-call count drifted".to_owned());
    }

    for forbidden in [
        "function kgwBridgeForm(",
        "validateBridgeForm(",
        "renderFieldErrors(",
        "revealSettingsField(",
        "!/^[1-9]\\d*(ms|s)?$/.test(",
        "Number.isSafeInteger(Number(raw))",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge full-form validation JavaScript ownership remains: {forbidden}"
            ));
        }
    }

    for required in [
        "js_name = bridgeValidateFormUi",
        "pub fn bridge_validate_form_ui(",
        "bridge_form_values_inner(&net)",
        "settings_validate_bridge_form(",
        "bridge_instance_duration_valid(",
        "bridge_instance_integer_valid(",
        "settings_render_field_errors(",
        "[data-bridge-inner-tab=\\\"settings\\\"]",
        "[data-bridge-section-panel]",
        "[data-bridge-section-tab=\\\"{section_name}\\\"]",
        "settings_reveal_field(",
        "call0(&field, \"focus\")",
    ] {
        if !helpers.contains(required) {
            return Err(format!(
                "Bridge full-form validation Rust owner contract missing: {required}"
            ));
        }
    }

    Ok(())
}

fn verify_static_contracts(
    source: &str,
    helpers: &str,
    runtime_core: &str,
    start_trace: &str,
) -> Result<(), String> {
    for needle in
        ["Tauri invoke resolution and timeout policy are Rust-owned in bridge_start_trace.rs."]
    {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge readiness JavaScript contract missing: {needle}"
            ));
        }
    }
    for needle in [
        "let runtime_error = runtime_error_text(&status);",
        "Bridge runtime failed after readiness.",
    ] {
        if !runtime_core.contains(needle) {
            return Err(format!(
                "Bridge readiness Rust runtime contract missing: {needle}"
            ));
        }
    }
    for needle in [
        "experimental: true",
        "enabled_by_default: false",
        "requires explicit opt-in",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge readiness Rust owner contract missing: {needle}"
            ));
        }
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgeNormalizeInnerTab)]",
        "#[wasm_bindgen(js_name = bridgeResolveInnerTab)]",
        "#[wasm_bindgen(js_name = bridgeSaveInnerTab)]",
        "kgw.bridge.innerTab.{}",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R101U inner-tab Rust owner contract missing: {needle}"
            ));
        }
    }

    for forbidden in [
        "bridgeSaveInnerTab as wasmBridgeSaveInnerTab",
        "wasmBridgeSaveInnerTab(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Bridge R101U direct JavaScript binding must remain retired after OP287: {forbidden}"
            ));
        }
    }
    if !helpers.contains("bridge_save_inner_tab(")
        || !helpers.contains("pub fn bridge_install_delegated_tabs_ui(")
    {
        return Err(
            "Bridge R101U delegated-tabs Rust owner must persist inner-tab state after OP287"
                .to_owned(),
        );
    }
    for forbidden in [
        "bridgeResolveInnerTab as wasmBridgeResolveInnerTab",
        "wasmBridgeResolveInnerTab(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Bridge R101U resolve binding/call must remain retired after OP284: {forbidden}"
            ));
        }
    }
    if !helpers.contains("bridge_resolve_inner_tab(net.clone())") {
        return Err(
            "Bridge R101U network-panel Rust owner must resolve persisted inner-tab state"
                .to_owned(),
        );
    }

    for forbidden in [
        "function kgwBridgeInnerTabStorageKeyR101U(",
        "function kgwBridgeNormalizeInnerTabR101U(",
        "function kgwBridgeResolveInnerTabR101U(",
        "function kgwBridgeSaveInnerTabR101U(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R101U JavaScript owner remains: {forbidden}"
            ));
        }
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgeNormalizeNetwork)]",
        "#[wasm_bindgen(js_name = bridgeReadLastNetwork)]",
        "#[wasm_bindgen(js_name = bridgeSaveLastNetwork)]",
        "kgw.bridge.lastNetwork",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R101W2 last-network Rust owner contract missing: {needle}"
            ));
        }
    }

    for forbidden in [
        "bridgeNormalizeNetwork as wasmBridgeNormalizeNetwork",
        "bridgeReadLastNetwork as wasmBridgeReadLastNetwork",
        "bridgeSaveLastNetwork as wasmBridgeSaveLastNetwork",
        "wasmBridgeNormalizeNetwork(",
        "wasmBridgeSaveLastNetwork(",
        "wasmBridgeReadLastNetwork(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Bridge R101W2 direct JavaScript binding must remain retired after OP286: {forbidden}"
            ));
        }
    }
    for needle in [
        "pub fn bridge_install_network_tabs_ui(",
        "normalize_bridge_network_text(selected)",
        "storage_set(BRIDGE_LAST_NETWORK_KEY, &normalized)",
        "let saved = bridge_read_last_network();",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R101W2 network-tab Rust owner contract missing after OP286: {needle}"
            ));
        }
    }

    for retired in [
        "const KGW_BRIDGE_LAST_NETWORK_KEY_R101W2 =",
        "function kgwBridgeNormalizeNetworkR101W2(",
        "function kgwBridgeReadLastNetworkR101W2(",
        "function kgwBridgeSaveLastNetworkR101W2(",
    ] {
        if source.contains(retired) {
            return Err(format!(
                "Retired Bridge R101W2 JavaScript owner remains: {retired}"
            ));
        }
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgeLogAutoScrollEnabled)]",
        "#[wasm_bindgen(js_name = bridgeSetLogAutoScroll)]",
        "#[wasm_bindgen(js_name = bridgeInstallLogAutoScrollControls)]",
        "kgw.bridge.log.autoscroll.{net}",
        "bridge_log_auto_scroll_enabled_text",
        "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
        "r51b3-bridge-log-autoscroll-change",
        ".bridge-v7-log-toolbar, .bridge-log-toolbar, [data-bridge-log-toolbar]",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R27 log-auto-scroll Rust owner contract missing: {needle}"
            ));
        }
    }

    for needle in [
        "bridgeInstallLogAutoScrollControls as wasmBridgeInstallLogAutoScrollControls",
        "setTimeout(wasmBridgeInstallLogAutoScrollControls, 0);",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge R27 DOM installer Rust/WASM binding missing: {needle}"
            ));
        }
    }
    if source
        .matches("setTimeout(wasmBridgeInstallLogAutoScrollControls, 0);")
        .count()
        != 1
    {
        return Err(
            "Bridge R27 DOM installer must have exactly one remaining direct JavaScript scheduling call site after OP282 moves render-all-networks deferred installation into Rust"
                .to_owned(),
        );
    }

    for retired in [
        "function kgwBridgeLogAutoScrollKeyR27(",
        "function kgwBridgeLogAutoScrollEnabledR27(",
        "function kgwBridgeSetLogAutoScrollR27(",
        "function kgwInstallBridgeLogAutoScrollControlsR27(",
        "wasmBridgeLogAutoScrollEnabled",
        "wasmBridgeSetLogAutoScroll",
    ] {
        if source.contains(retired) {
            return Err(format!(
                "Retired Bridge R27 JavaScript owner/seam remains: {retired}"
            ));
        }
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgeTranslateRuntimeFeedback)]",
        "[\"kgwT\", \"kgwI18n\", \"__kgwT\"]",
        "\"__kgwI18nDictR107\"",
        "\"__kgwI18nDict\"",
        "\"kgwI18nDict\"",
        "\"__KGW_I18N_DICT__\"",
        "runtime_feedback_terminal_text",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge runtime-feedback translation Rust owner contract missing: {needle}"
            ));
        }
    }
    if source.contains("bridgeTranslateRuntimeFeedback as wasmBridgeTranslateRuntimeFeedback")
        || source.contains("wasmBridgeTranslateRuntimeFeedback(")
        || source.contains("kgwBridgeTranslateRuntime(")
    {
        return Err(
            "Retired Bridge runtime-feedback translation JavaScript binding/owner remains"
                .to_owned(),
        );
    }
    for needle in [
        "bridge_frontend_helpers::bridge_translate_runtime_feedback(",
        "\"runtime.failed\".to_owned()",
        "\"Failed\".to_owned()",
    ] {
        if !runtime_core.contains(needle) {
            return Err(format!(
                "Bridge runtime-feedback translation Rust runtime binding missing: {needle}"
            ));
        }
    }

    for needle in [
        "bridgeSetOwnedNodeLockR65E as wasmBridgeSetOwnedNodeLockR65E",
        "wasmBridgeSetOwnedNodeLockR65E(String(net || \"\"), true, {",
        "wasmBridgeSetOwnedNodeLockR65E(String(net || \"\"), false, {",
    ] {
        if !source.contains(needle) {
            return Err(format!("Bridge R65E Rust/WASM binding missing: {needle}"));
        }
    }
    for retired in [
        "function kgwBridgeOwnedNodeLockStoreR65E(",
        "function kgwSetBridgeOwnedNodeLockR65E(",
        "kgwSetBridgeOwnedNodeLockR65E(",
        "__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E",
    ] {
        if source.contains(retired) {
            return Err(format!(
                "Retired Bridge R65E JavaScript lock owner remains: {retired}"
            ));
        }
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgeCurrentNodeModeFromUiR65F)]",
        "Reflect::has(target, &JsValue::from_str(\"value\"))",
        "[data-bridge-panel=\\\"{selector_net}\\\"]",
        "[data-net=\\\"{selector_net}\\\"]",
        "[id$=\\\"-nodeMode\\\"], [data-bridge-setting=\\\"nodeMode\\\"], select[name=\\\"nodeMode\\\"]",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R65F current-node-mode Rust DOM owner contract missing: {needle}"
            ));
        }
    }
    for needle in [
        "bridgeCurrentNodeModeFromUiR65F as wasmBridgeCurrentNodeModeFromUiR65F",
        "wasmBridgeCurrentNodeModeFromUiR65F(String(net || \"\"))",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge R65F current-node-mode direct Rust/WASM binding missing: {needle}"
            ));
        }
    }
    if source
        .matches("wasmBridgeCurrentNodeModeFromUiR65F(String(net || \"\"))")
        .count()
        != 3
    {
        return Err(
            "Bridge R65F current-node-mode owner must have exactly three generated-WASM call sites"
                .to_owned(),
        );
    }
    if source.contains("function kgwBridgeCurrentNodeModeFromUiR65F(") {
        return Err("Retired Bridge R65F current-node-mode JavaScript owner remains".to_owned());
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgeInstanceNetworkKeyR15)]",
        "fn bridge_instance_network_key_candidate(",
        "fn bridge_instance_network_key_text(",
        "property(value, \"key\").as_string()",
        "profile(normalized)",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R15 instance network-key Rust owner contract missing: {needle}"
            ));
        }
    }
    if source.contains("bridgeInstanceNetworkKeyR15 as wasmBridgeInstanceNetworkKeyR15")
        || source.contains("wasmBridgeInstanceNetworkKeyR15(")
    {
        return Err(
            "Bridge R15 instance network-key JavaScript binding/calls must be fully retired after OP281 moves the final container-click canonicalization into Rust"
                .to_owned(),
        );
    }
    if source.contains("function bridgeInstanceNetworkKeyR15(") {
        return Err("Retired Bridge R15 JavaScript network-key owner remains".to_owned());
    }

    for needle in [
        "#[wasm_bindgen(js_name = bridgePlainPortOnlyValueR98)]",
        "#[wasm_bindgen(js_name = bridgeSamePortValueR98)]",
        "fn bridge_plain_port_only_text(",
        "strip_prefix(':')",
        "1..=65_535",
        "u8::is_ascii_digit",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge R98 port-only normalization Rust owner contract missing: {needle}"
            ));
        }
    }
    for forbidden in [
        "bridgePlainPortOnlyValueR98 as wasmBridgePlainPortOnlyValueR98",
        "bridgeSamePortValueR98 as wasmBridgeSamePortValueR98",
        "wasmBridgePlainPortOnlyValueR98(",
        "wasmBridgeSamePortValueR98(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R98 direct JavaScript binding remains after OP263: {forbidden}"
            ));
        }
    }
    for retired in [
        "function kgwBridgeR98PlainPortOnlyValue(",
        "function kgwBridgeR98SamePortValue(",
        "function kgwBridgeR95BNormalizePlainPortValue(",
        "kgwBridgeR98PlainPortOnlyValue(",
        "kgwBridgeR98SamePortValue(",
        "kgwBridgeR95BNormalizePlainPortValue(",
    ] {
        if source.contains(retired) {
            return Err(format!(
                "Retired Bridge R98/R95B JavaScript normalization owner remains: {retired}"
            ));
        }
    }

    for needle in [
        "bridgeStringifyRuntimeResult",
        "bridgeNormalizeRuntimeError",
        "bridgeParseRuntimeKeyValueResponse",
        "bridgeV7RuntimeRunningFromText",
        "bridgeR51IsRunning",
        "bridgeRuntimeErrorFromStatus",
        "bridgeNormalizeNodeModeR65F",
        "bridgePreviewDeclaresInprocessR65F",
        "bridgeRuntimeCommandForAction",
        "bridgeRuntimeActionOutcome",
        "bridgeStartWasInprocessR65F",
    ] {
        if !runtime_core.contains(needle) {
            return Err(format!("Bridge runtime-core Rust export missing: {needle}"));
        }
    }

    for forbidden in [
        "function stringifyRuntimeResult(",
        "function normalizeRuntimeError(",
        "function parseRuntimeKeyValueResponse(",
        "function kgwBridgeV7RuntimeRunningFromText(",
        "function kgwBridgeNormalizeNodeModeR65F(",
        "function kgwBridgePreviewDeclaresInprocessR65F(",
        "function kgwBridgeStartWasInprocessR65F(",
        "function kgwBridgeR51IsRunning(",
        "function kgwBridgeRuntimeErrorFromStatus(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge runtime parser remains in JavaScript: {forbidden}"
            ));
        }
    }

    if source.contains("function bridgeAssertNoPortConflictsR5(") {
        return Err("Retired Bridge scoped port-conflict wrapper remains in JavaScript".to_owned());
    }

    for needle in [
        "bridgeRuntimeInvokeAvailable",
        "bridgeInvokeRuntimeCommand",
        "bridgePreparePreview",
        "BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS",
        "BRIDGE_PREVIEW_INVOKE_TIMEOUT_MS",
        "BRIDGE_PREVIEW_UNAVAILABLE_ERROR",
    ] {
        if !start_trace.contains(needle) {
            return Err(format!(
                "Bridge start-trace Rust transport export missing: {needle}"
            ));
        }
    }

    if source.contains("function kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(") {
        return Err(
            "Retired Bridge Rusty-Kaspa default-paths fire-and-forget JavaScript helper remains"
                .to_owned(),
        );
    }
    if !source.contains(
        "bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5 as wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5",
    ) {
        return Err(
            "Bridge Rusty-Kaspa default-paths fire-and-forget Rust/WASM import is missing"
                .to_owned(),
        );
    }
    if source
        .matches("wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(")
        .count()
        != 1
    {
        return Err(
            "Bridge Rusty-Kaspa default-paths fire-and-forget owner must have exactly one direct JavaScript Rust/WASM call site after OP255 moves Restore Defaults into Rust"
                .to_owned(),
        );
    }
    if !helpers
        .contains("#[wasm_bindgen(js_name = bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5)]")
        || !helpers.contains("pub fn bridge_apply_rusty_kaspa_root_only_default_paths_soon_r5(")
    {
        return Err(
            "Bridge Rusty-Kaspa default-paths fire-and-forget Rust owner is missing".to_owned(),
        );
    }

    if source.contains("async function kgwBridgePreparePreview(") {
        return Err("Retired Bridge preview runtime dispatch JavaScript helper remains".to_owned());
    }
    if !source.contains("bridgePreparePreview as wasmBridgePreparePreview") {
        return Err("Bridge preview runtime dispatch Rust/WASM import is missing".to_owned());
    }
    if source
        .matches("wasmBridgePreparePreview(String(net || \"\"),")
        .count()
        != 2
    {
        return Err(
            "Bridge preview runtime dispatch must use exactly two remaining direct JavaScript Rust/WASM call sites after OP274 moves update-command preview dispatch into Rust"
                .to_owned(),
        );
    }

    for forbidden in [
        "function getTauriInvoke(",
        "function invokeWithTimeout(",
        "const KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS =",
        "const KGW_BRIDGE_STOP_INVOKE_TIMEOUT_MS =",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge invoke transport remains in JavaScript: {forbidden}"
            ));
        }
    }

    for line in source.lines() {
        let lower = line.to_ascii_lowercase();
        if !lower.contains("appendlog(") {
            continue;
        }
        if [
            "ready",
            "readiness",
            "start failed",
            "start response",
            "start confirmed",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        {
            return Err(format!(
                "typed readiness diagnostic leaked into raw Bridge log pane: {line}"
            ));
        }
        if ["forced", "graceful", "stop_outcome", "stopping"]
            .iter()
            .any(|needle| lower.contains(needle))
        {
            return Err(format!(
                "Stop control diagnostic leaked into raw Bridge log pane: {line}"
            ));
        }
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<String, String> {
    let full_source = fs::read_to_string(root.join(BRIDGE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_SOURCE}: {error}"))?;
    let helper_source = fs::read_to_string(root.join(BRIDGE_HELPERS_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_HELPERS_SOURCE}: {error}"))?;
    let runtime_core_source = fs::read_to_string(root.join(BRIDGE_RUNTIME_CORE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_RUNTIME_CORE_SOURCE}: {error}"))?;
    let start_trace_source = fs::read_to_string(root.join(BRIDGE_START_TRACE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_START_TRACE_SOURCE}: {error}"))?;
    let instance_settings_source = fs::read_to_string(root.join(BRIDGE_INSTANCE_SETTINGS_SOURCE))
        .map_err(|error| {
        format!("failed to read {BRIDGE_INSTANCE_SETTINGS_SOURCE}: {error}")
    })?;
    let command_options_source = fs::read_to_string(root.join(BRIDGE_COMMAND_OPTIONS_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_COMMAND_OPTIONS_SOURCE}: {error}"))?;
    let instance_ui_source = fs::read_to_string(root.join(BRIDGE_INSTANCE_UI_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_INSTANCE_UI_SOURCE}: {error}"))?;
    let bridge_render_source = fs::read_to_string(root.join(BRIDGE_RENDER_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_RENDER_SOURCE}: {error}"))?;
    let bridge_port_ui_source = fs::read_to_string(root.join(BRIDGE_PORT_UI_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_PORT_UI_SOURCE}: {error}"))?;
    verify_r51_storage_ownership(&full_source, &helper_source)?;
    verify_r51_keys_ownership(&full_source, &helper_source)?;
    verify_r51_runtime_presentation_ownership(&full_source, &runtime_core_source)?;
    verify_runtime_notice_ownership(&full_source, &runtime_core_source)?;
    verify_mark_restart_required_ownership(&full_source, &runtime_core_source)?;
    verify_r51_live_refresh_ownership(&full_source, &runtime_core_source)?;
    verify_inprocess_node_owner_guard_ownership(&full_source, &start_trace_source)?;
    verify_start_options_ownership(&full_source, &helper_source, &instance_settings_source)?;
    verify_network_panel_ownership(&full_source, &helper_source)?;
    verify_render_all_networks_ownership(&full_source, &helper_source)?;
    verify_instance_click_owner_ownership(&full_source, &instance_ui_source)?;
    verify_visible_instance_owners_ownership(&full_source, &instance_ui_source)?;
    verify_instance_refresh_ownership(&full_source, &instance_ui_source)?;
    verify_network_tabs_ownership(&full_source, &helper_source)?;
    verify_delegated_tabs_ownership(&full_source, &helper_source)?;
    verify_action_event_owners_ownership(&full_source, &helper_source)?;
    verify_settings_event_owners_ownership(&full_source, &helper_source)?;
    verify_root_action_click_ownership(&full_source, &helper_source)?;
    verify_port_event_owners_ownership(&full_source, &bridge_port_ui_source)?;
    verify_inprocess_node_settings_renderer_ownership(&full_source, &bridge_render_source)?;
    verify_settings_sections_ownership(&full_source, &helper_source)?;
    verify_instances_renderer_ownership(&full_source, &instance_ui_source)?;
    verify_instance_command_option_ownership(&full_source, &command_options_source)?;
    verify_instance_mutation_ownership(&full_source, &instance_settings_source)?;
    verify_instance_state_structured_reader_ownership(&full_source, &instance_settings_source)?;
    verify_preview_message_ownership(&full_source, &helper_source)?;
    verify_r95b_port_normalization_ownership(&full_source, &helper_source)?;
    verify_inline_command_toggle_ownership(&full_source, &command_options_source)?;
    verify_dependency_sync_ownership(&full_source, &helper_source)?;
    verify_mode_controls_ownership(&full_source, &helper_source)?;
    verify_command_orchestration_ownership(&full_source, &helper_source)?;
    verify_command_preview_ownership(&full_source, &helper_source)?;
    verify_inprocess_mode_controls_ownership(&full_source, &helper_source)?;
    verify_full_form_validation_ownership(&full_source, &helper_source)?;
    verify_require_valid_settings_ownership(&full_source, &helper_source)?;
    verify_apply_payload_ownership(&full_source, &helper_source)?;
    verify_static_contracts(
        &full_source,
        &helper_source,
        &runtime_core_source,
        &start_trace_source,
    )?;
    let selected = selected_source(root)?;
    let actual = run_bridge(root, &selected)?;

    expect(
        &actual,
        "/r51StorageOwnership",
        json!({
            "missing": null,
            "networkKeys": ["mainnet", "testnet10", "testnet13"],
            "roundTrips": [
                {"key": "saved:mainnet", "value": 51},
                {"key": "default:testnet10", "value": 51},
                {"key": "factory:testnet13", "value": 51}
            ],
            "exactKeys": ["kgw.bridge.direct.v51.default:testnet10",
                "kgw.bridge.direct.v51.factory:testnet13", "kgw.bridge.direct.v51.saved:mainnet"],
            "rawSaved": "{\"key\":\"saved:mainnet\",\"value\":51}",
            "malformed": null, "falsey": [false, 0, "", null],
            "circularThrows": true, "circularNotWritten": true,
            "readFailure": null, "writeFailureThrows": true,
            "unavailableLoad": null, "unavailableStoreThrows": true,
            "factoryPreserved": {"value": "factory"},
            "missingSavedUsesCurrent": {"value": "changed"},
            "savedLoaded": {"value": "saved"},
            "defaultRestored": {"value": "default"},
            "defaultReadOrder": ["kgw.bridge.direct.v51.default:mainnet"],
            "factoryRestored": {"value": "factory"},
            "fallbackReadOrder": ["kgw.bridge.direct.v51.default:mainnet",
                "kgw.bridge.direct.v51.factory:mainnet"],
            "noDefaults": null
        }),
    )?;

    expect(&actual, "/transport/available", json!(true))?;
    expect(&actual, "/transport/result", json!("transport-ok"))?;
    expect(
        &actual,
        "/transport/calls/0/command",
        json!("kgw_kgw_runtime_logs_v1"),
    )?;
    expect(&actual, "/transport/calls/0/network", json!("mainnet"))?;
    expect(&actual, "/inprocessNodeOwnerGuard/external", json!(false))?;
    expect(&actual, "/inprocessNodeOwnerGuard/externalCalls", json!(0))?;
    expect(&actual, "/inprocessNodeOwnerGuard/running", json!(true))?;
    expect(&actual, "/inprocessNodeOwnerGuard/runningCalls", json!(1))?;
    expect(&actual, "/inprocessNodeOwnerGuard/stopped", json!(false))?;
    expect(&actual, "/inprocessNodeOwnerGuard/stoppedCalls", json!(1))?;
    expect(
        &actual,
        "/inprocessNodeOwnerGuard/command",
        json!("kgw_runtime_owner_status_v1"),
    )?;
    expect(
        &actual,
        "/inprocessNodeOwnerGuard/network",
        json!("mainnet"),
    )?;
    expect(
        &actual,
        "/inprocessNodeOwnerGuard/runtimeRole",
        json!("node"),
    )?;
    expect(
        &actual,
        "/inprocessNodeOwnerGuard/alert",
        json!(
            "Cannot start bridge in in-process mode because the same-network node is already running. Stop the node first, or switch bridge node mode to External."
        ),
    )?;

    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/alphaChecked",
        json!(true),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/alphaAria",
        json!("Included in command"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/alphaTitle",
        json!("Included in command"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/alphaIsOn",
        json!(true),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/alphaIsOff",
        json!(false),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/betaChecked",
        json!(false),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/betaAria",
        json!("Excluded from command"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/betaTitle",
        json!("Excluded from command"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/betaIsOn",
        json!(false),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/betaIsOff",
        json!(true),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/otherChecked",
        json!(true),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/initial/otherTitle",
        json!("untouched"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/toggledEnabled",
        json!(false),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/alphaChecked",
        json!(false),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/alphaAria",
        json!("Excluded from command"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/alphaTitle",
        json!("Excluded from command"),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/alphaIsOn",
        json!(false),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/alphaIsOff",
        json!(true),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/otherChecked",
        json!(true),
    )?;
    expect(
        &actual,
        "/inlineCommandToggleOwnership/afterToggle/otherTitle",
        json!("untouched"),
    )?;

    expect(
        &actual,
        "/startOptionsOwnership/mainnet/internalCpuMiner/enabled",
        json!(false),
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/enabled/configFile",
        json!("C:\\bridge-op261.toml"),
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/enabled/internalCpuMiner/enabled",
        json!(true),
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/enabled/internalCpuMiner/address",
        json!("kaspa:op261-address"),
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/enabled/internalCpuMiner/threads",
        json!(1),
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/enabled/internalCpuMiner/throttleMs",
        json!(250),
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/enabled/internalCpuMiner/templatePollMs",
        Value::Null,
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/configDisabled/configFile",
        Value::Null,
    )?;
    expect(
        &actual,
        "/startOptionsOwnership/invalidError",
        json!("CPU threads is outside the supported range"),
    )?;

    expect(&actual, "/previewMessageOwnership/missing", json!(false))?;
    expect(
        &actual,
        "/previewMessageOwnership/validating/result",
        json!(true),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/validating/text",
        json!("Validating effective settings..."),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/validating/tone",
        json!("warning"),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/validating/errorClass",
        json!(false),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/error/result",
        json!(true),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/error/text",
        json!("Preview failed"),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/error/tone",
        json!("negative"),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/error/errorClass",
        json!(true),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/verified/result",
        json!(true),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/verified/text",
        json!("Preview verified"),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/verified/tone",
        json!("ready"),
    )?;
    expect(
        &actual,
        "/previewMessageOwnership/verified/errorClass",
        json!(false),
    )?;

    expect(&actual, "/previewTransport/result", json!("transport-ok"))?;
    expect(
        &actual,
        "/previewTransport/command",
        json!("kgw_runtime_settings_preview_v1"),
    )?;
    expect(
        &actual,
        "/previewTransport/unavailableError",
        json!("The desktop runtime is unavailable."),
    )?;

    expect(
        &actual,
        "/defaultPathOwnership/appdir",
        json!("APPDIR-mainnet"),
    )?;
    expect(
        &actual,
        "/defaultPathOwnership/appdirTitle",
        json!("APPDIR-mainnet"),
    )?;
    expect(
        &actual,
        "/defaultPathOwnership/mirror",
        json!("APPDIR-mainnet"),
    )?;
    expect(
        &actual,
        "/defaultPathOwnership/mirrorTitle",
        json!("APPDIR-mainnet"),
    )?;
    expect(&actual, "/defaultPathOwnership/updateCalls", json!(1))?;
    expect(&actual, "/defaultPathOwnership/updateNet", json!("mainnet"))?;
    expect(&actual, "/defaultPathOwnership/contextDelta", json!(1))?;
    expect(
        &actual,
        "/defaultPathOwnership/contextNetwork",
        json!("mainnet"),
    )?;

    expect(
        &actual,
        "/portOnlyNormalizationOwnership/plain",
        json!("5655"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/colon",
        json!("5655"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/trimmedLeadingZeros",
        json!("80"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/numeric",
        json!("5655"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/falseyZero",
        json!(""),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/falseyFalse",
        json!(""),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/falseyNull",
        json!(""),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/zeroString",
        json!("0"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/colonZero",
        json!(":00000"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/max",
        json!("65535"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/tooHigh",
        json!("65536"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/tooLong",
        json!("123456"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/hostPort",
        json!("host:5655"),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/sameColonLeadingZeros",
        json!(true),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/sameWhitespace",
        json!(true),
    )?;
    expect(
        &actual,
        "/portOnlyNormalizationOwnership/different",
        json!(false),
    )?;

    expect(
        &actual,
        "/r95bPortNormalizationOwnership/testnet10Identity",
        json!(true),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/testnet10Stratum",
        json!("5655"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/testnet10Prom",
        json!("2212"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/testnet13Stratum",
        json!("5755"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/testnet13Prom",
        json!("2312"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/displayStratum",
        json!("5555"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/displayProm",
        json!("2112"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/customStratum",
        json!("5678"),
    )?;
    expect(
        &actual,
        "/r95bPortNormalizationOwnership/customProm",
        json!("2277"),
    )?;

    expect(
        &actual,
        "/instanceNetworkKeyOwnership/directString",
        json!("testnet10"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/directObject",
        json!("testnet13"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/fallbackString",
        json!("testnet10"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/fallbackObject",
        json!("testnet13"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/directWins",
        json!("mainnet"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/invalidDefault",
        json!("mainnet"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/numericIgnored",
        json!("testnet10"),
    )?;
    expect(
        &actual,
        "/instanceNetworkKeyOwnership/nonStringObjectKeyIgnored",
        json!("testnet13"),
    )?;

    expect(
        &actual,
        "/currentNodeModeOwnership/direct",
        json!("inprocess"),
    )?;
    expect(
        &actual,
        "/currentNodeModeOwnership/primaryPanel",
        json!("external"),
    )?;
    expect(
        &actual,
        "/currentNodeModeOwnership/secondaryPanel",
        json!("secondary"),
    )?;
    expect(
        &actual,
        "/currentNodeModeOwnership/directEmptyPrecedence",
        json!(""),
    )?;
    expect(&actual, "/currentNodeModeOwnership/missing", json!(""))?;
    expect(
        &actual,
        "/currentNodeModeOwnership/selectorError",
        json!(""),
    )?;

    expect(&actual, "/bridgeOwnedNodeLockOwnership/locked", json!(true))?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/net",
        json!("mainnet"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/reason",
        json!("bridge-inprocess-owner"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/updatedAtPositive",
        json!(true),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/detailsSource",
        json!("op236-smoke"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/detailsPid",
        json!("4242"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/lockEventType",
        json!("kgw-bridge-owned-node-lock-r65e"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/lockEventNet",
        json!("mainnet"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/lockEventLocked",
        json!(true),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/lockEventSource",
        json!("KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/unlockedAbsent",
        json!(true),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/eventCount",
        json!(2),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/unlockEventType",
        json!("kgw-bridge-owned-node-lock-r65e"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/unlockEventNet",
        json!("mainnet"),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/unlockEventLocked",
        json!(false),
    )?;
    expect(
        &actual,
        "/bridgeOwnedNodeLockOwnership/unlockEventSource",
        json!("KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E"),
    )?;

    expect(
        &actual,
        "/runtimeTranslationOwnership/runtimeFunction",
        json!("Runtime Function Failed"),
    )?;
    expect(
        &actual,
        "/runtimeTranslationOwnership/runtimeKeyFallsThrough",
        json!("Flat Dictionary Failed"),
    )?;
    expect(
        &actual,
        "/runtimeTranslationOwnership/runtimeThrows",
        json!("Dictionary After Runtime Throw"),
    )?;
    expect(
        &actual,
        "/runtimeTranslationOwnership/nestedDictionary",
        json!("Nested Dictionary Failed"),
    )?;
    expect(
        &actual,
        "/runtimeTranslationOwnership/fallback",
        json!("Failed"),
    )?;
    expect(
        &actual,
        "/runtimeTranslationOwnership/fallbackToKey",
        json!("runtime.failed"),
    )?;
    expect(
        &actual,
        "/runtimeTranslationOwnership/firstTruthyRuntime",
        json!("First Truthy Non Function Uses Dictionary"),
    )?;

    expect(&actual, "/innerTabOwnership/defaultMainnet", json!("log"))?;
    expect(&actual, "/innerTabOwnership/normalizeInvalid", json!("log"))?;
    expect(
        &actual,
        "/innerTabOwnership/saveSettings",
        json!("settings"),
    )?;
    expect(
        &actual,
        "/innerTabOwnership/savedMainnet",
        json!("settings"),
    )?;
    expect(&actual, "/innerTabOwnership/defaultTestnet10", json!("log"))?;
    expect(&actual, "/innerTabOwnership/saveInvalid", json!("log"))?;
    expect(&actual, "/innerTabOwnership/savedTestnet10", json!("log"))?;
    expect(
        &actual,
        "/innerTabOwnership/mainnetAfterTestnet",
        json!("settings"),
    )?;

    expect(&actual, "/lastNetworkOwnership/initial", json!(""))?;
    expect(
        &actual,
        "/lastNetworkOwnership/normalizeMainnet",
        json!("mainnet"),
    )?;
    expect(&actual, "/lastNetworkOwnership/normalizeInvalid", json!(""))?;
    expect(
        &actual,
        "/lastNetworkOwnership/saveTestnet10",
        json!("testnet10"),
    )?;
    expect(
        &actual,
        "/lastNetworkOwnership/savedAfterValid",
        json!("testnet10"),
    )?;
    expect(&actual, "/lastNetworkOwnership/saveInvalid", json!(""))?;
    expect(
        &actual,
        "/lastNetworkOwnership/savedAfterInvalid",
        json!("testnet10"),
    )?;

    expect(
        &actual,
        "/logAutoScrollOwnership/defaultMainnet",
        json!(true),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/disabledMainnet",
        json!(false),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/storedAfterDisable",
        json!("0"),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/scrollAfterDisable",
        json!(7),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/enabledMainnet",
        json!(true),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/storedAfterEnable",
        json!("1"),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/scrollAfterEnable",
        json!(321),
    )?;
    expect(
        &actual,
        "/logAutoScrollOwnership/defaultTestnet10",
        json!(true),
    )?;

    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/toolbarChildren",
        json!(1),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/labelClass",
        json!("kgw-log-autoscroll-toggle"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/scope",
        json!("bridge"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/title",
        json!("Keep the log pinned to the newest raw line."),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/checkboxType",
        json!("checkbox"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/checkboxId",
        json!("bridge-mainnet-logAutoScrollR27"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/checkboxChecked",
        json!(true),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/spanText",
        json!("Auto-scroll"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/changeListener",
        json!(true),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/fallbackChildren",
        json!(2),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/fallbackInsertedBeforeOutput",
        json!(true),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/fallbackLabelClass",
        json!("kgw-log-autoscroll-toggle"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerOwnership/fallbackCheckboxId",
        json!("bridge-testnet10-logAutoScrollR27"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerDisabled/stored",
        json!("0"),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerDisabled/scrollTop",
        json!(11),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerDisabled/traceCommand",
        json!("kgw_frontend_button_trace_v1"),
    )?;
    expect(&actual, "/logAutoScrollInstallerEnabled/stored", json!("1"))?;
    expect(
        &actual,
        "/logAutoScrollInstallerEnabled/scrollTop",
        json!(321),
    )?;
    expect(
        &actual,
        "/logAutoScrollInstallerEnabled/traceCommand",
        json!("kgw_frontend_button_trace_v1"),
    )?;

    expect(&actual, "/runtimeRunning/liveOnly", json!(false))?;
    expect(&actual, "/runtimeRunning/ready", json!(true))?;
    expect(&actual, "/bridgeRunning/liveOnly", json!(false))?;
    expect(&actual, "/bridgeRunning/ready", json!(true))?;
    expect(&actual, "/liveRefresh/policy", json!("Bridge: Running"))?;
    expect(&actual, "/liveRefresh/startDisabled", json!(true))?;
    expect(&actual, "/liveRefresh/stopDisabled", json!(false))?;
    expect(
        &actual,
        "/restartRunning/text",
        json!("Restart required to apply changed effective settings"),
    )?;
    expect(&actual, "/restartRunning/restartRequired", json!("true"))?;
    expect(
        &actual,
        "/restartStopped/text",
        json!("Effective settings apply on next Start"),
    )?;
    expect(&actual, "/restartStopped/restartRequired", json!("false"))?;

    expect(&actual, "/starting/policy", json!("Bridge: Starting"))?;
    expect(&actual, "/starting/startDisabled", json!(true))?;
    expect(&actual, "/starting/stopDisabled", json!(true))?;
    expect(&actual, "/stopping/policy", json!("Bridge: Stopping"))?;
    expect(&actual, "/stopping/startDisabled", json!(true))?;
    expect(&actual, "/stopping/stopDisabled", json!(true))?;

    expect(&actual, "/visibleFailure/policy", json!("Bridge: Stopped"))?;
    expect(
        &actual,
        "/visibleFailure/runtimeError",
        json!("occupied listener port"),
    )?;
    expect(&actual, "/visibleFailure/runtimeErrorHidden", json!(false))?;
    expect(
        &actual,
        "/visibleFailure/runtimeStatus",
        json!("Bridge start failed."),
    )?;

    expect(
        &actual,
        "/lifecycle/startPending/policy",
        json!("Bridge: Starting"),
    )?;
    expect(
        &actual,
        "/lifecycle/startPending/startDisabled",
        json!(true),
    )?;
    expect(
        &actual,
        "/lifecycle/startReady/policy",
        json!("Bridge: Running"),
    )?;
    expect(&actual, "/lifecycle/startReady/startDisabled", json!(true))?;
    expect(
        &actual,
        "/lifecycle/startReady/runtimeStatus",
        json!("Bridge READY attestation confirmed."),
    )?;

    expect(
        &actual,
        "/lifecycle/startFailed/policy",
        json!("Reconciling"),
    )?;
    expect(
        &actual,
        "/lifecycle/startFailed/runtimeError",
        json!("occupied listener port"),
    )?;
    expect(
        &actual,
        "/lifecycle/stopPending/policy",
        json!("Bridge: Stopping"),
    )?;
    expect(
        &actual,
        "/lifecycle/stopGraceful/policy",
        json!("Bridge: Stopped"),
    )?;
    expect(
        &actual,
        "/lifecycle/stopGraceful/runtimeStatus",
        json!("Bridge graceful official shutdown confirmed."),
    )?;
    let forced_error = actual
        .pointer("/lifecycle/stopForced/runtimeError")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !forced_error.contains("FORCED") {
        return Err(format!(
            "Bridge forced-stop error lost FORCED classification: {forced_error}"
        ));
    }
    expect(
        &actual,
        "/lifecycle/stopForced/runtimeStatus",
        json!("Bridge FORCED termination confirmed."),
    )?;
    let failed_error = actual
        .pointer("/lifecycle/stopFailed/runtimeError")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !failed_error.contains("Official graceful shutdown failed") {
        return Err(format!(
            "Bridge graceful-stop failure message lost classification: {failed_error}"
        ));
    }
    expect(
        &actual,
        "/lifecycle/stopFailed/runtimeStatus",
        json!("Bridge worker exited after graceful shutdown failure."),
    )?;

    Ok("Bridge readiness frontend Rust owner PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r51_storage_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_r51_storage_ownership(source, helpers).is_ok());
        for mutation in [
            source.replacen("wasmBridgeR51Load(", "kgwBridgeR51Load(", 1),
            source.replacen("wasmBridgeR51Load(", "wasmBridgeR51Store(", 1),
            source.replace(
                "bridgeR51Load as wasmBridgeR51Load,",
                "bridgeR51Load as wasmBridgeR51Load,\n  bridgeR51Store as wasmBridgeR51Store,",
            ),
            source.replace(
                "bridgeR51Load as wasmBridgeR51Load",
                "unowned as wasmBridgeR51Load",
            ),
        ] {
            assert!(verify_r51_storage_ownership(&mutation, helpers).is_err());
        }
        assert!(
            verify_r51_storage_ownership(
                source,
                &helpers.replace("kgw.bridge.direct.v51.", "wrong.")
            )
            .is_err()
        );
        assert!(verify_r51_storage_ownership(source, "").is_err());
    }

    #[test]
    fn r51_keys_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_r51_keys_ownership(source, helpers).is_ok());
        for mutation in [
            source.replacen("wasmBridgeR51Keys()", "kgwBridgeR51Keys()", 1),
            source.replace(
                "bridgeR51Keys as wasmBridgeR51Keys",
                "unowned as wasmBridgeR51Keys",
            ),
        ] {
            assert!(verify_r51_keys_ownership(&mutation, helpers).is_err());
        }
        assert!(
            verify_r51_keys_ownership(
                source,
                &helpers.replace("js_name = bridgeR51Keys", "js_name = missingR51Keys")
            )
            .is_err()
        );
    }

    #[test]
    fn r51_runtime_presentation_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let runtime_core =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_runtime_core.rs");
        assert!(verify_r51_runtime_presentation_ownership(source, runtime_core).is_ok());
        for mutation in [
            source.replacen(
                "wasmBridgeR51SetRuntimeButtons(",
                "kgwBridgeR51SetRuntimeButtons(",
                1,
            ),
            source.replacen(
                "wasmBridgeR51SetRuntimeUnknown(",
                "kgwBridgeR51SetRuntimeUnknown(",
                1,
            ),
            source.replace(
                "bridgeR51SetRuntimeButtons as wasmBridgeR51SetRuntimeButtons",
                "missingRuntimeButtons as wasmBridgeR51SetRuntimeButtons",
            ),
        ] {
            assert!(verify_r51_runtime_presentation_ownership(&mutation, runtime_core).is_err());
        }
        assert!(
            verify_r51_runtime_presentation_ownership(
                source,
                &runtime_core.replace(
                    "js_name = bridgeR51SetRuntimeUnknown",
                    "js_name = missingR51SetRuntimeUnknown",
                ),
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_notice_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let runtime_core =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_runtime_core.rs");
        assert!(verify_runtime_notice_ownership(source, runtime_core).is_ok());
        for mutation in [
            source.replacen(
                "wasmBridgeSetRuntimeErrorV1(",
                "kgwBridgeSetRuntimeErrorV1(",
                1,
            ),
            source.replacen(
                "wasmBridgeSetRuntimeActivityV1(",
                "kgwBridgeSetRuntimeActivityV1(",
                1,
            ),
            source.replace(
                "bridgeSetRuntimeErrorV1 as wasmBridgeSetRuntimeErrorV1",
                "missingRuntimeErrorV1 as wasmBridgeSetRuntimeErrorV1",
            ),
            format!(
                "{source}\nfunction kgwBridgeR51RuntimePresentationCallbacksR256() {{ return {{ setRuntimeError: () => null }}; }}\n"
            ),
        ] {
            assert!(verify_runtime_notice_ownership(&mutation, runtime_core).is_err());
        }
        assert!(
            verify_runtime_notice_ownership(
                source,
                &runtime_core.replace(
                    "js_name = bridgeSetRuntimeActivityV1",
                    "js_name = missingSetRuntimeActivityV1",
                ),
            )
            .is_err()
        );
        assert!(
            verify_runtime_notice_ownership(
                source,
                &runtime_core.replace(
                    "fn set_runtime_error_inner(",
                    "fn missing_set_runtime_error_inner(",
                ),
            )
            .is_err()
        );
    }

    #[test]
    fn mark_restart_required_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let runtime_core =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_runtime_core.rs");
        assert!(verify_mark_restart_required_ownership(source, runtime_core).is_ok());
        for mutation in [
            format!(
                "{source}\n// bridgeMarkRestartRequiredV1 as wasmBridgeMarkRestartRequiredV1\n"
            ),
            format!("{source}\n// wasmBridgeMarkRestartRequiredV1(\"mainnet\");\n"),
            format!("{source}\nfunction kgwBridgeMarkRestartRequiredV1(net) {{ return net; }}\n"),
        ] {
            assert!(verify_mark_restart_required_ownership(&mutation, runtime_core).is_err());
        }
        assert!(
            verify_mark_restart_required_ownership(
                source,
                &runtime_core.replace(
                    "js_name = bridgeMarkRestartRequiredV1",
                    "js_name = missingMarkRestartRequiredV1",
                ),
            )
            .is_err()
        );
        assert!(
            verify_mark_restart_required_ownership(
                source,
                &runtime_core.replace(".contains(\"running\")", ".contains(\"ready\")",),
            )
            .is_err()
        );
    }

    #[test]
    fn inprocess_node_owner_guard_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let start_trace =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_start_trace.rs");
        assert!(verify_inprocess_node_owner_guard_ownership(source, start_trace).is_ok());
        for mutation in [
            source.replacen(
                "wasmBridgeV7BlockInprocessIfNodeOwnerRunning(",
                "kgwBridgeV7BlockInprocessIfNodeOwnerRunning(",
                1,
            ),
            source.replace(
                "bridgeV7BlockInprocessIfNodeOwnerRunning as wasmBridgeV7BlockInprocessIfNodeOwnerRunning",
                "missingBlockInprocessGuard as wasmBridgeV7BlockInprocessIfNodeOwnerRunning",
            ),
            format!(
                "{source}\nasync function kgwBridgeV7BlockInprocessIfNodeOwnerRunning(net) {{ return Boolean(net); }}\n"
            ),
        ] {
            assert!(
                verify_inprocess_node_owner_guard_ownership(&mutation, start_trace).is_err()
            );
        }
        for mutation in [
            start_trace.replace(
                "js_name = bridgeV7BlockInprocessIfNodeOwnerRunning",
                "js_name = missingV7BlockInprocessIfNodeOwnerRunning",
            ),
            start_trace.replace(
                "invoke_runtime_command_impl(\"kgw_runtime_owner_status_v1\"",
                "invoke_runtime_command_impl(\"missing_runtime_owner_status\"",
            ),
            start_trace.replace("\"runtimeRole\"", "\"missingRuntimeRole\""),
        ] {
            assert!(verify_inprocess_node_owner_guard_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn start_options_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        let instance_settings = include_str!(
            "../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_settings.rs"
        );
        assert!(verify_start_options_ownership(source, helpers, instance_settings).is_ok());

        for mutation in [
            format!(
                "{source}\nfunction kgwBridgeStartOptions(net) {{ return {{ configFile: net }}; }}\n"
            ),
            format!("{source}\nconst legacyStartOptions = wasmBridgeStartOptions(\"mainnet\");\n"),
            format!("{source}\n// bridgeStartOptions as wasmBridgeStartOptions\n"),
        ] {
            assert!(verify_start_options_ownership(&mutation, helpers, instance_settings).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeBuildApplyPayloadUi",
                "js_name = missingBuildApplyPayloadUi",
            ),
            helpers.replace(
                "crate::bridge_instance_settings::bridge_start_options(",
                "crate::bridge_instance_settings::missing_bridge_start_options(",
            ),
        ] {
            assert!(verify_start_options_ownership(source, &mutation, instance_settings).is_err());
        }

        for mutation in [
            instance_settings.replace(
                "js_name = bridgeStartOptions",
                "js_name = missingBridgeStartOptions",
            ),
            instance_settings.replace(
                "bridge_command_options::bridge_has_config",
                "bridge_command_options::missing_bridge_has_config",
            ),
            instance_settings.replace("net != \"mainnet\"", "net != \"testnet\""),
            instance_settings.replace("\"CPU threads\"", "\"Missing CPU threads\""),
        ] {
            assert!(verify_start_options_ownership(source, helpers, &mutation).is_err());
        }
    }

    #[test]
    fn network_panel_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_network_panel_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function renderNetworkPanel(net, index) {",
            "function renderNetworkPanel(net, index) {\n  const activeInnerTab = wasmBridgeResolveInnerTab(String(net.key || \"\"));",
            1,
        );
        assert!(verify_network_panel_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeRenderNetworkPanelUi",
            "js_name = missingRenderNetworkPanelUi",
        );
        assert!(verify_network_panel_ownership(source, &missing).is_err());
    }

    #[test]
    fn render_all_networks_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_render_all_networks_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function renderAllNetworks(root) {",
            "function renderAllNetworks(root) {\n  const host = root.querySelector(\"#bridgeNetworkPanels\"); host.innerHTML = BRIDGE_NETWORKS.map(renderNetworkPanel).join(\"\");",
            1,
        );
        assert!(verify_render_all_networks_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeRenderAllNetworksUi",
            "js_name = missingRenderAllNetworksUi",
        );
        assert!(verify_render_all_networks_ownership(source, &missing).is_err());
    }

    #[test]
    fn instance_click_owner_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let instance_ui =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_ui.rs");
        assert!(verify_instance_click_owner_ownership(source, instance_ui).is_ok());

        let legacy = source.replacen(
            "function bridgeInstallInstanceContainerOwnerR11(container, net) {",
            "function bridgeInstallInstanceContainerOwnerR11(container, net) {\n  container.onclick = function(event) { event.preventDefault(); };",
            1,
        );
        assert!(verify_instance_click_owner_ownership(&legacy, instance_ui).is_err());

        let missing = instance_ui.replace(
            "js_name = bridgeInstallInstanceContainerOwnerR11",
            "js_name = missingInstallInstanceContainerOwnerR11",
        );
        assert!(verify_instance_click_owner_ownership(source, &missing).is_err());
    }

    #[test]
    fn visible_instance_owners_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let instance_ui =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_ui.rs");
        assert!(verify_visible_instance_owners_ownership(source, instance_ui).is_ok());

        let legacy = source.replacen(
            "function bridgeInstallAllVisibleInstanceContainerOwnersR11(root) {",
            "function bridgeInstallAllVisibleInstanceContainerOwnersR11(root) {\n  for (const profile of BRIDGE_NETWORKS) { wasmBridgeById(profile.key); }",
            1,
        );
        assert!(verify_visible_instance_owners_ownership(&legacy, instance_ui).is_err());

        let missing = instance_ui.replace(
            "js_name = bridgeInstallAllVisibleInstanceContainerOwnersR11",
            "js_name = missingInstallAllVisibleInstanceContainerOwnersR11",
        );
        assert!(verify_visible_instance_owners_ownership(source, &missing).is_err());
    }

    #[test]
    fn instance_refresh_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let instance_ui =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_ui.rs");
        assert!(verify_instance_refresh_ownership(source, instance_ui).is_ok());

        for mutation in [
            source.replace(
                "bridgeRefreshInstancesUi as wasmBridgeRefreshInstancesUi",
                "missingRefreshInstancesUi as wasmBridgeRefreshInstancesUi",
            ),
            source.replacen(
                "function bridgeRefreshInstances(net) {",
                "function bridgeRefreshInstances(net) {\n  const container = document.querySelector(\"[data-bridge-section-panel=instances]\");",
                1,
            ),
        ] {
            assert!(verify_instance_refresh_ownership(&mutation, instance_ui).is_err());
        }

        for mutation in [
            instance_ui.replace(
                "js_name = bridgeRefreshInstancesUi",
                "js_name = missingRefreshInstancesUi",
            ),
            instance_ui.replace(
                "bridge_render_instances_ui(",
                "missing_render_instances_ui(",
            ),
            instance_ui.replace(
                "call1_required(&callbacks, \"updateCommand\"",
                "call1_required(&callbacks, \"missingUpdateCommand\"",
            ),
        ] {
            assert!(verify_instance_refresh_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn network_tabs_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_network_tabs_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function installNetworkTabs(root) {",
            "function installNetworkTabs(root) {\n  root.addEventListener(\"click\", () => {});",
            1,
        );
        assert!(verify_network_tabs_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeInstallNetworkTabsUi",
            "js_name = missingInstallNetworkTabsUi",
        );
        assert!(verify_network_tabs_ownership(source, &missing).is_err());
    }

    #[test]
    fn delegated_tabs_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_delegated_tabs_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function installDelegatedTabs(root) {",
            "function installDelegatedTabs(root) {\n  root.addEventListener(\"click\", () => {});",
            1,
        );
        assert!(verify_delegated_tabs_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeInstallDelegatedTabsUi",
            "js_name = missingInstallDelegatedTabsUi",
        );
        assert!(verify_delegated_tabs_ownership(source, &missing).is_err());
    }

    #[test]
    fn action_event_owners_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_action_event_owners_ownership(source, helpers).is_ok());

        let legacy =
            format!("{source}\n// KGW_BRIDGE_COMMAND_CHECKBOX_FIRST_CLICK_FIX_TRACE_PATCH_R31\n");
        assert!(verify_action_event_owners_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeInstallActionEventOwnersUi",
            "js_name = missingInstallActionEventOwnersUi",
        );
        assert!(verify_action_event_owners_ownership(source, &missing).is_err());
    }

    #[test]
    fn settings_event_owners_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_settings_event_owners_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function installActions(root) {",
            "function installActions(root) {\n  root.addEventListener(\"input\", () => {});",
            1,
        );
        assert!(verify_settings_event_owners_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeInstallSettingsEventOwnersUi",
            "js_name = missingInstallSettingsEventOwnersUi",
        );
        assert!(verify_settings_event_owners_ownership(source, &missing).is_err());
    }

    #[test]
    fn root_action_click_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_root_action_click_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function installActions(root) {",
            "function installActions(root) {\n  root.addEventListener(\"click\", () => {});",
            1,
        );
        assert!(verify_root_action_click_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeInstallRootActionClickOwnerUi",
            "js_name = missingInstallRootActionClickOwnerUi",
        );
        assert!(verify_root_action_click_ownership(source, &missing).is_err());
    }

    #[test]
    fn port_event_owners_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let port_ui =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_port_ui.rs");
        assert!(verify_port_event_owners_ownership(source, port_ui).is_ok());

        let legacy = format!("{source}\n// root.dataset.kgwBridgePortAutofixOwnerR37 = \"1\";\n");
        assert!(verify_port_event_owners_ownership(&legacy, port_ui).is_err());

        let missing = port_ui.replace(
            "js_name = bridgeInstallPortEventOwnersUi",
            "js_name = missingInstallPortEventOwnersUi",
        );
        assert!(verify_port_event_owners_ownership(source, &missing).is_err());
    }

    #[test]
    fn inprocess_node_settings_renderer_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let bridge_render =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_render.rs");
        assert!(verify_inprocess_node_settings_renderer_ownership(source, bridge_render).is_ok());

        let legacy = source.replacen(
            "function renderInprocessNodeSettings(net) {",
            "function renderInprocessNodeSettings(net) {\n  const tabs = []; const template = document.createElement(\"template\");",
            1,
        );
        assert!(verify_inprocess_node_settings_renderer_ownership(&legacy, bridge_render).is_err());

        let missing = bridge_render.replace(
            "js_name = bridgeRenderInprocessNodeSettingsUi",
            "js_name = missingRenderInprocessNodeSettingsUi",
        );
        assert!(verify_inprocess_node_settings_renderer_ownership(source, &missing).is_err());
    }

    #[test]
    fn settings_sections_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_settings_sections_ownership(source, helpers).is_ok());

        let legacy = source.replacen(
            "function renderSections(net) {",
            "function renderSections(net) {\n  const template = document.createElement(\"template\"); const cards = new Map();",
            1,
        );
        assert!(verify_settings_sections_ownership(&legacy, helpers).is_err());

        let missing = helpers.replace(
            "js_name = bridgeRenderSectionsUi",
            "js_name = missingRenderSectionsUi",
        );
        assert!(verify_settings_sections_ownership(source, &missing).is_err());
    }

    #[test]
    fn instances_renderer_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let instance_ui =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_ui.rs");
        assert!(verify_instances_renderer_ownership(source, instance_ui).is_ok());

        for mutation in [
            format!("{source}\n// bridgeRenderInstancesUi as wasmBridgeRenderInstancesUi\n"),
            format!(
                "{source}\nfunction renderInstances(net) {{ return wasmBridgeRenderInstancesUi(net, bridgeInstances, activeInstance); }}\n"
            ),
        ] {
            assert!(verify_instances_renderer_ownership(&mutation, instance_ui).is_err());
        }

        for mutation in [
            instance_ui.replace(
                "js_name = bridgeRenderInstancesUi",
                "js_name = missingRenderInstancesUi",
            ),
            instance_ui.replace(
                "data-bridge-action=\"add-instance\"",
                "data-bridge-action=\"missing-add-instance\"",
            ),
            instance_ui.replace(
                "bridge_instance_command_checkbox_from_instances_r13b(",
                "missing_instance_command_checkbox_from_instances(",
            ),
        ] {
            assert!(verify_instances_renderer_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn instance_command_option_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let command_options =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_command_options.rs");
        assert!(verify_instance_command_option_ownership(source, command_options).is_ok());

        for mutation in [
            source.replace(
                "bridgeSetInstanceCommandOptionUiR13B as wasmBridgeSetInstanceCommandOptionUiR13B",
                "missingSetInstanceCommandOptionUiR13B as wasmBridgeSetInstanceCommandOptionUiR13B",
            ),
            source.replacen(
                "function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {",
                "function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {\n  wasmBridgeSmallOwnerTraceR44D(net, \"command-checkbox\", \"legacy\", {});",
                1,
            ),
        ] {
            assert!(verify_instance_command_option_ownership(&mutation, command_options).is_err());
        }

        for mutation in [
            command_options.replace(
                "js_name = bridgeSetInstanceCommandOptionUiR13B",
                "js_name = missingSetInstanceCommandOptionUiR13B",
            ),
            command_options.replace(
                "bridge_sync_instance_preview_rows_r8b(",
                "missing_sync_instance_preview_rows(",
            ),
            command_options.replace(
                "r29b-bridge-instance-command-checkbox-complete",
                "missing-command-checkbox-complete",
            ),
        ] {
            assert!(verify_instance_command_option_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn instance_mutation_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let instance_settings = include_str!(
            "../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_settings.rs"
        );
        assert!(verify_instance_mutation_ownership(source, instance_settings).is_ok());

        for mutation in [
            source.replace(
                "bridgeAddInstanceUi as wasmBridgeAddInstanceUi",
                "missingAddInstanceUi as wasmBridgeAddInstanceUi",
            ),
            source.replacen(
                "function addInstance(net) {",
                "function addInstance(net) {\n  bridgeInstances[net].push({});",
                1,
            ),
            source.replacen(
                "function removeInstance(net, instanceId) {",
                "function removeInstance(net, instanceId) {\n  const removedIndex = bridgeInstances[net].findIndex(() => true);",
                1,
            ),
        ] {
            assert!(verify_instance_mutation_ownership(&mutation, instance_settings).is_err());
        }

        for mutation in [
            instance_settings.replace(
                "js_name = bridgeAddInstanceUi",
                "js_name = missingAddInstanceUi",
            ),
            instance_settings.replace(
                "bridge_port_orchestration::bridge_create_instance_record_r9(",
                "bridge_port_orchestration::missing_create_instance_record(",
            ),
            instance_settings.replace(
                "removed_index: Option<u32>",
                "removed_index_missing: Option<u32>",
            ),
        ] {
            assert!(verify_instance_mutation_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn instance_state_structured_reader_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let instance_settings = include_str!(
            "../../crates/kaspa-gateway-frontend-wasm/src/bridge_instance_settings.rs"
        );
        assert!(
            verify_instance_state_structured_reader_ownership(source, instance_settings).is_ok()
        );

        for mutation in [
            source.replace(
                "// KGW_BRIDGE_INSTANCE_STATE_RUST_OWNER_V1",
                "// missing instance-state owner",
            ),
            format!(
                "{source}\nfunction bridgeReadInstanceState(net, instanceId) {{ return [net, instanceId]; }}\n"
            ),
            source.replacen(
                "activeInstance\n  );",
                "activeInstance,\n    { readInstanceState: () => ({}) }\n  );",
                1,
            ),
        ] {
            assert!(
                verify_instance_state_structured_reader_ownership(&mutation, instance_settings)
                    .is_err()
            );
        }

        for (index, mutation) in [
            instance_settings.replace(
                "js_name = bridgeReadInstanceStateUi",
                "js_name = missingReadInstanceStateUi",
            ),
            instance_settings.replace(
                "bridge_instance_ui::bridge_read_instance_field(",
                "bridge_instance_ui::missing_bridge_read_instance_field(",
            ),
            instance_settings.replace(
                "bridge_read_instance_state_impl(&net, bridge_instances, &fallback_id)",
                "missing_instance_state_owner(&net, bridge_instances, &fallback_id)",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            assert!(
                verify_instance_state_structured_reader_ownership(source, &mutation).is_err(),
                "Rust instance-state mutation {index} was not rejected"
            );
        }
    }

    #[test]
    fn command_preview_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_command_preview_ownership(source, helpers).is_ok());

        for mutation in [
            source.replace(
                "bridgeUpdateCommandUi as wasmBridgeUpdateCommandUi",
                "missingUpdateCommandUi as wasmBridgeUpdateCommandUi",
            ),
            source.replace(
                "return wasmBridgeUpdateCommandUi(",
                "return wasmBridgePreparePreview(",
            ),
            source.replacen(
                "function updateAllCommands() {",
                "function updateAllCommands() {\n  bridgeSyncAllModeControls();",
                1,
            ),
        ] {
            assert!(verify_command_preview_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeUpdateCommandUi",
                "js_name = missingUpdateCommandUi",
            ),
            helpers.replace(
                "fn begin_bridge_preview_sequence(",
                "fn missing_begin_bridge_preview_sequence(",
            ),
            helpers.replace(
                "crate::bridge_start_trace::bridge_prepare_preview(",
                "crate::bridge_start_trace::missing_bridge_prepare_preview(",
            ),
        ] {
            assert!(verify_command_preview_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn preview_message_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_preview_message_ownership(source, helpers).is_ok());
        for mutation in [
            source.replacen("wasmBridgePreviewMessage(", "kgwBridgePreviewMessage(", 1),
            source.replace(
                "bridgePreviewMessage as wasmBridgePreviewMessage",
                "missingPreviewMessage as wasmBridgePreviewMessage",
            ),
            format!(
                "{source}\nfunction kgwBridgePreviewMessage(net, message) {{ return [net, message]; }}\n"
            ),
        ] {
            assert!(verify_preview_message_ownership(&mutation, helpers).is_err());
        }
        for mutation in [
            helpers.replace(
                "js_name = bridgePreviewMessage",
                "js_name = missingBridgePreviewMessage",
            ),
            helpers.replace("\"kgw-field-error\"", "\"missing-field-error\""),
            helpers.replace(
                "message.starts_with(\"Validating\")",
                "message.starts_with(\"Checking\")",
            ),
        ] {
            assert!(verify_preview_message_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn r95b_port_normalization_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_r95b_port_normalization_ownership(source, helpers).is_ok());
        for mutation in [
            source.replacen(
                "wasmBridgeR95BNormalizeNetworkPortValues(",
                "kgwBridgeR95BNormalizeNetworkPortValues(",
                1,
            ),
            source.replace(
                "bridgeR95BNormalizeNetworkPortValues as wasmBridgeR95BNormalizeNetworkPortValues",
                "missingR95BNormalizeNetworkPortValues as wasmBridgeR95BNormalizeNetworkPortValues",
            ),
            format!(
                "{source}\nfunction kgwBridgeR95BNormalizeNetworkPortValues(net, values) {{ return values || net; }}\n"
            ),
        ] {
            assert!(verify_r95b_port_normalization_ownership(&mutation, helpers).is_err());
        }
        for mutation in [
            helpers.replace(
                "js_name = bridgeR95BNormalizeNetworkPortValues",
                "js_name = missingR95BNormalizeNetworkPortValues",
            ),
            helpers.replace(
                r#"("testnet10", "stratumPort") => "5556""#,
                r#"("testnet10", "stratumPort") => "5999""#,
            ),
            helpers.replace(
                r#""bridge-r51-r95b-settings-owner""#,
                r#""missing-r95b-settings-owner""#,
            ),
        ] {
            assert!(verify_r95b_port_normalization_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn full_form_validation_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_full_form_validation_ownership(source, helpers).is_ok());

        for mutation in [
            source.replace(
                "bridgeValidateFormUi as wasmBridgeValidateFormUi",
                "missingValidateFormUi as wasmBridgeValidateFormUi",
            ),
            source.replace(
                "return wasmBridgeValidateFormUi(String(net || \"\"), bridgeInstances, Boolean(focus));",
                "return {};",
            ),
            format!(
                "{source}\nfunction kgwBridgeForm(net) {{ return {{ network: net }}; }}\n"
            ),
            format!(
                "{source}\nfunction legacyValidation(raw) {{ return Number.isSafeInteger(Number(raw)); }}\n"
            ),
        ] {
            assert!(verify_full_form_validation_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeValidateFormUi",
                "js_name = missingBridgeValidateFormUi",
            ),
            helpers.replace(
                "settings_validate_bridge_form(",
                "missing_validate_bridge_form(",
            ),
            helpers.replace(
                "settings_render_field_errors(",
                "missing_render_field_errors(",
            ),
            helpers.replace("settings_reveal_field(", "missing_reveal_field("),
        ] {
            assert!(verify_full_form_validation_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn mode_controls_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_mode_controls_ownership(source, helpers).is_ok());

        for mutation in [
            source.replace(
                "bridgeSyncModeControlsUi as wasmBridgeSyncModeControlsUi",
                "missingSyncModeControlsUi as wasmBridgeSyncModeControlsUi",
            ),
            source.replace(
                "return wasmBridgeSyncModeControlsUi(String(net || \"\"), bridgeInstances);",
                "return false;",
            ),
            source.replacen(
                "function bridgeSyncModeControls(net) {",
                "function bridgeSyncModeControls(net) {\n  const nodeMode = wasmBridgeNodeMode(String(net || \"\"));",
                1,
            ),
        ] {
            assert!(verify_mode_controls_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeSyncModeControlsUi",
                "js_name = missingSyncModeControlsUi",
            ),
            helpers.replace(
                "Network identity is owned by the selected Mainnet/Testnet tab.",
                "missing network identity guard",
            ),
            helpers.replace(
                "bridge_sync_dependencies(net, bridge_instances)",
                "missing_dependency_sync(net, bridge_instances)",
            ),
        ] {
            assert!(verify_mode_controls_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn command_orchestration_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_command_orchestration_ownership(source, helpers).is_ok());

        for mutation in [
            source.replace(
                "bridgeBuildCommandLinesUi as wasmBridgeBuildCommandLinesUi",
                "missingBuildCommandLinesUi as wasmBridgeBuildCommandLinesUi",
            ),
            source.replace(
                "return wasmBridgeSyncAllModeControlsUi(bridgeInstances);",
                "BRIDGE_NETWORKS.forEach((item) => bridgeSyncModeControls(item.key));",
            ),
            source.replacen(
                "function buildCommandLines(net) {",
                "function buildCommandLines(net) {\n  wasmBridgeEnsureInstanceState(bridgeInstances, activeInstance, String(net || \"\"));",
                1,
            ),
        ] {
            assert!(verify_command_orchestration_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeSyncAllModeControlsUi",
                "js_name = missingSyncAllModeControlsUi",
            ),
            helpers.replace(
                "crate::bridge_port_orchestration::bridge_ensure_instance_state(",
                "crate::bridge_port_orchestration::missing_ensure_instance_state(",
            ),
            helpers.replace(
                "crate::bridge_command_builder::bridge_build_command_lines(",
                "crate::bridge_command_builder::missing_build_command_lines(",
            ),
        ] {
            assert!(verify_command_orchestration_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn inprocess_mode_controls_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_inprocess_mode_controls_ownership(source, helpers).is_ok());

        for mutation in [
            format!("{source}\n// bridgeSetDisabledUi as wasmBridgeSetDisabledUi\n"),
            format!(
                "{source}\nfunction bridgeSyncInprocessNodeSettingsV12D(net) {{ return wasmBridgeSyncInprocessNodeSettingsV12D(net); }}\n"
            ),
            format!(
                "{source}\nfunction bridgeControlCard(el) {{ return el?.closest(\".bridge-v7-card\"); }}\n"
            ),
        ] {
            assert!(verify_inprocess_mode_controls_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeSyncInprocessNodeSettingsV12D",
                "js_name = missingSyncInprocessNodeSettingsV12D",
            ),
            helpers.replace(
                "\"bridge-v12d-inprocess-active\"",
                "\"missing-inprocess-active\"",
            ),
            helpers.replace(
                "Dangerous development-only kaspad flag is disabled on mainnet.",
                "missing mainnet guard",
            ),
        ] {
            assert!(verify_inprocess_mode_controls_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn apply_payload_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_apply_payload_ownership(source, helpers).is_ok());

        for mutation in [
            source.replace(
                "bridgeBuildApplyPayloadUi as wasmBridgeBuildApplyPayloadUi",
                "missingBuildApplyPayloadUi as wasmBridgeBuildApplyPayloadUi",
            ),
            source.replace(
                "return wasmBridgeBuildApplyPayloadUi(",
                "return { network: net, nodeKind: \"remote\" }; // ",
            ),
            source.replacen(
                "function buildApplyPayload(net, command) {",
                "function buildApplyPayload(net, command) {\n  const bridgeOptions = wasmBridgeStartOptions(String(net || \"\"));",
                1,
            ),
        ] {
            assert!(verify_apply_payload_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeBuildApplyPayloadUi",
                "js_name = missingBuildApplyPayloadUi",
            ),
            helpers.replace(
                "crate::bridge_port_validation::bridge_assert_no_port_conflicts_r5(",
                "missing_assert_no_port_conflicts(",
            ),
            helpers.replace(
                "crate::bridge_instance_settings::bridge_start_options(",
                "missing_bridge_start_options(",
            ),
            helpers.replace("\"official-inprocess-node\"", "\"missing-inprocess-node\""),
        ] {
            assert!(verify_apply_payload_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn require_valid_settings_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_require_valid_settings_ownership(source, helpers).is_ok());

        for mutation in [
            source.replace(
                "bridgeRequireValidSettingsUi as wasmBridgeRequireValidSettingsUi",
                "missingRequireValidSettingsUi as wasmBridgeRequireValidSettingsUi",
            ),
            source.replace(
                "return wasmBridgeEffectiveInprocessNodeSettingsChecked(String(net || \"\"), bridgeInstances);",
                "return null;",
            ),
            source.replacen(
                "function kgwBridgeRequireValidSettings(net) {\n  return wasmBridgeRequireValidSettingsUi(String(net || \"\"), bridgeInstances, activeInstance, kgwBridgeR51ReadStructuredInstancesR253);",
                "function kgwBridgeRequireValidSettings(net) {\n  const errors = kgwBridgeValidateForm(net, true);\n  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);\n  return wasmBridgeRequireValidSettingsUi(String(net || \"\"), bridgeInstances, activeInstance, kgwBridgeR51ReadStructuredInstancesR253);",
                1,
            ),
            format!("{source}\nconst legacy = wasmBridgeEffectiveInprocessNodeSettings(\"mainnet\");\n"),
        ] {
            assert!(verify_require_valid_settings_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeRequireValidSettingsUi",
                "js_name = missingRequireValidSettingsUi",
            ),
            helpers.replace(
                "crate::bridge_port_validation::bridge_assert_no_port_conflicts_r5(",
                "missing_assert_no_port_conflicts(",
            ),
            helpers.replace(
                "crate::bridge_instance_settings::bridge_effective_settings_v1(",
                "missing_effective_settings_v1(",
            ),
            helpers.replace(
                "fn bridge_first_validation_error(",
                "fn missing_first_validation_error(",
            ),
        ] {
            assert!(verify_require_valid_settings_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn dependency_sync_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let helpers =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs");
        assert!(verify_dependency_sync_ownership(source, helpers).is_ok());

        for mutation in [
            format!("{source}\n// bridgeSyncDependencies as wasmBridgeSyncDependencies\n"),
            format!(
                "{source}\nfunction kgwBridgeSyncDependencies(net) {{ return wasmBridgeSyncDependencies(net, bridgeInstances); }}\n"
            ),
            format!(
                "{source}\nfunction legacyBridgeDependencyOwner(name, values, options) {{ return bridgeFieldEnabled(name, values, options); }}\n"
            ),
        ] {
            assert!(verify_dependency_sync_ownership(&mutation, helpers).is_err());
        }

        for mutation in [
            helpers.replace(
                "js_name = bridgeSyncDependencies",
                "js_name = missingBridgeSyncDependencies",
            ),
            helpers.replace(
                "[data-bridge-instance-command-option-toggle-r13b]",
                "[data-missing-instance-command-toggle]",
            ),
            helpers.replace(
                "settings_bridge_field_enabled(",
                "missing_bridge_field_enabled(",
            ),
            helpers.replace(
                "settings_decorate_fields(panel)",
                "missing_decorate_fields(panel)",
            ),
        ] {
            assert!(verify_dependency_sync_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn inline_command_toggle_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let command_options =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_command_options.rs");
        assert!(verify_inline_command_toggle_ownership(source, command_options).is_ok());

        for mutation in [
            source.replacen(
                "wasmBridgeRefreshInlineCommandTogglesR7(",
                "kgwBridgeRefreshInlineCommandTogglesR7(",
                1,
            ),
            format!(
                "{source}\n// bridgeCommandToggleOptionR7 as wasmBridgeCommandToggleOptionR7\nwasmBridgeCommandToggleOptionR7(\"mainnet\", \"config\");\n"
            ),
            source.replace(
                "bridgeRefreshInlineCommandTogglesR7 as wasmBridgeRefreshInlineCommandTogglesR7",
                "missingRefreshInlineCommandTogglesR7 as wasmBridgeRefreshInlineCommandTogglesR7",
            ),
            format!(
                "{source}\nfunction kgwBridgeRefreshInlineCommandTogglesR7(net) {{ return net; }}\n"
            ),
        ] {
            assert!(verify_inline_command_toggle_ownership(&mutation, command_options).is_err());
        }

        for mutation in [
            command_options.replace(
                "js_name = bridgeRefreshInlineCommandTogglesR7",
                "js_name = missingRefreshInlineCommandTogglesR7",
            ),
            command_options.replace(
                "[data-bridge-command-option-toggle-r7]",
                "[data-missing-command-option-toggle]",
            ),
            command_options.replace("\"is-off\"", "\"missing-is-off\""),
            command_options.replace(
                "refresh_inline_command_toggles_r7(&net);",
                "missing_refresh(&net);",
            ),
        ] {
            assert!(verify_inline_command_toggle_ownership(source, &mutation).is_err());
        }
    }

    #[test]
    fn r51_live_refresh_ownership_rejects_legacy_and_contract_drift() {
        let source = include_str!(
            "../../apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
        );
        let runtime_core =
            include_str!("../../crates/kaspa-gateway-frontend-wasm/src/bridge_runtime_core.rs");
        assert!(verify_r51_live_refresh_ownership(source, runtime_core).is_ok());
        for mutation in [
            source.replacen("wasmBridgeR51RefreshOne(", "kgwBridgeR51RefreshOne(", 1),
            source.replace(
                "bridgeR51StartLiveRefresh as wasmBridgeR51StartLiveRefresh",
                "missingStartLiveRefresh as wasmBridgeR51StartLiveRefresh",
            ),
            format!("{source}\nfunction kgwBridgeR51RefreshAll(reason) {{ return reason; }}\n"),
        ] {
            assert!(verify_r51_live_refresh_ownership(&mutation, runtime_core).is_err());
        }
        assert!(
            verify_r51_live_refresh_ownership(
                source,
                &runtime_core.replace(
                    "js_name = bridgeR51StartLiveRefresh",
                    "js_name = missingR51StartLiveRefresh",
                ),
            )
            .is_err()
        );
        assert!(
            verify_r51_live_refresh_ownership(
                source,
                &runtime_core.replace(
                    "static R51_STATUS_IN_FLIGHT:",
                    "static MISSING_R51_STATUS_IN_FLIGHT:",
                ),
            )
            .is_err()
        );
    }

    #[test]
    fn source_markers_fail_closed() {
        assert!(slice_between("missing", "start", "end").is_err());
        assert_eq!(
            slice_between("prefix START body END tail", "START", "END").unwrap(),
            "START body "
        );
    }

    #[test]
    fn rust_owns_start_and_stop_scenarios() {
        let request = request();
        assert!(
            request["startReady"]
                .as_str()
                .unwrap()
                .contains("readiness=READY")
        );
        assert!(
            request["stopForced"]
                .as_str()
                .unwrap()
                .contains("forced=true")
        );
        assert!(
            request["stopFailed"]
                .as_str()
                .unwrap()
                .contains("stop_failed=true")
        );
    }
}
