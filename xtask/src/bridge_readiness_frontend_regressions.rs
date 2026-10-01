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
        "function kgwBridgeSetRuntimeErrorV1(",
        "async function kgwBridgeV7BlockInprocessIfNodeOwnerRunning",
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
        "function kgwBridgeR51SetRuntimeButtons(",
        "function kgwBridgeR51MaybeActivityNotice(",
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
const transportInvoke = async (command, args) => {
  transportCalls.push({ command: String(command || ""), network: String(args?.network || "") });
  if (String(command || "") === "kgw_settings_context_v1") {
    return { appDir: "APPDIR-" + String(args?.network || "") };
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
    this.classList = {
      contains() { return false; },
      toggle() {},
      add() {},
      remove() {}
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

const documentImpl = {
  getElementById(id) {
    return elements.get(id) || null;
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
  wasmBridgeChecked: wasm.bridgeChecked,
  wasmBridgeStringifyRuntimeResult: wasm.bridgeStringifyRuntimeResult,
  wasmBridgeNormalizeRuntimeError: wasm.bridgeNormalizeRuntimeError,
  wasmBridgeParseRuntimeKeyValueResponse: wasm.bridgeParseRuntimeKeyValueResponse,
  wasmBridgeV7RuntimeRunningFromText: wasm.bridgeV7RuntimeRunningFromText,
  wasmBridgeR51IsRunning: wasm.bridgeR51IsRunning,
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
sandbox.kgwBridgeV7BlockInprocessIfNodeOwnerRunning = async () => false;
sandbox.invokeBridgeIntegratedRuntime = (...args) => invokeRuntime(...args);
sandbox.kgwBridgeRuntimeOwnerTraceR64D = () => {};
sandbox.kgwBridgePreviewDeclaresInprocessR65F = () => false;
sandbox.kgwBridgeR51KickRawLogLiveR134E = () => {};

vm.createContext(sandbox);
vm.runInContext(selected, sandbox, { filename: request.sourceName });
const api = vm.runInContext("({ runtimeRunning: wasmBridgeV7RuntimeRunningFromText, isRunning: wasmBridgeR51IsRunning, setButtons: kgwBridgeR51SetRuntimeButtons, setError: kgwBridgeSetRuntimeErrorV1, setActivity: kgwBridgeSetRuntimeActivityV1, runAction: runBridgeIntegratedAction })", sandbox);

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
  runtimeRunning: {
    liveOnly: api.runtimeRunning("role=node;network=mainnet;running=true"),
    ready: api.runtimeRunning("role=node;network=mainnet;running=true;readiness=READY")
  },
  bridgeRunning: {
    liveOnly: api.isRunning("role=bridge;network=mainnet;running=true"),
    ready: api.isRunning("role=bridge;network=mainnet;running=true;readiness=READY")
  },
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
    if source.matches("wasmBridgeR51Keys()").count() != 2
        || !helpers.contains("js_name = bridgeR51Keys")
        || !helpers.contains("fn bridge_r51_key_texts()")
    {
        return Err("R51 keys call count or Rust export drifted".to_owned());
    }
    Ok(())
}

fn verify_static_contracts(
    source: &str,
    helpers: &str,
    runtime_core: &str,
    start_trace: &str,
) -> Result<(), String> {
    for needle in [
        "wasmBridgeRuntimeErrorFromStatus(status)",
        "kgwBridgeSetRuntimeActivityV1(net, \"Bridge runtime failed after readiness.\", \"failed\")",
        "Tauri invoke resolution and timeout policy are Rust-owned in bridge_start_trace.rs.",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge readiness JavaScript contract missing: {needle}"
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

    for needle in [
        "bridgeResolveInnerTab as wasmBridgeResolveInnerTab",
        "bridgeSaveInnerTab as wasmBridgeSaveInnerTab",
        "wasmBridgeResolveInnerTab(String(net.key || \"\"))",
        "wasmBridgeSaveInnerTab(String(net || \"\"), innerTab.dataset.bridgeInnerTab)",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge R101U direct Rust/WASM binding missing: {needle}"
            ));
        }
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

    for needle in [
        "bridgeNormalizeNetwork as wasmBridgeNormalizeNetwork",
        "bridgeReadLastNetwork as wasmBridgeReadLastNetwork",
        "bridgeSaveLastNetwork as wasmBridgeSaveLastNetwork",
        "wasmBridgeNormalizeNetwork(net)",
        "wasmBridgeSaveLastNetwork(normalized)",
        "wasmBridgeReadLastNetwork()",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge R101W2 direct Rust/WASM binding missing: {needle}"
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
        != 2
    {
        return Err(
            "Bridge R27 DOM installer must have exactly two generated-WASM scheduling call sites"
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
    for needle in [
        "bridgeTranslateRuntimeFeedback as wasmBridgeTranslateRuntimeFeedback",
        "wasmBridgeTranslateRuntimeFeedback(\"runtime.failed\", \"Failed\")",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge runtime-feedback translation direct Rust/WASM binding missing: {needle}"
            ));
        }
    }
    if source.contains("kgwBridgeTranslateRuntime(") {
        return Err(
            "Retired Bridge runtime-feedback translation JavaScript owner/call remains".to_owned(),
        );
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
    if !source.contains("bridgeInstanceNetworkKeyR15 as wasmBridgeInstanceNetworkKeyR15") {
        return Err("Bridge R15 instance network-key Rust/WASM import is missing".to_owned());
    }
    if source.matches("wasmBridgeInstanceNetworkKeyR15(").count() != 6 {
        return Err(
            "Bridge R15 instance network-key owner must have exactly six direct generated-WASM call sites after OP255 moves the R51 persistence lifecycle into Rust"
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
    for needle in [
        "bridgePlainPortOnlyValueR98 as wasmBridgePlainPortOnlyValueR98",
        "bridgeSamePortValueR98 as wasmBridgeSamePortValueR98",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge R98 direct Rust/WASM binding missing: {needle}"
            ));
        }
    }
    if source.matches("wasmBridgePlainPortOnlyValueR98(").count() != 3 {
        return Err(
            "Bridge R98 plain-port owner must have exactly three generated-WASM call sites"
                .to_owned(),
        );
    }
    if source.matches("wasmBridgeSamePortValueR98(").count() != 2 {
        return Err(
            "Bridge R98 same-port owner must have exactly two generated-WASM call sites".to_owned(),
        );
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
        != 3
    {
        return Err(
            "Bridge preview runtime dispatch must use exactly three direct Rust/WASM call sites"
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
    verify_r51_storage_ownership(&full_source, &helper_source)?;
    verify_r51_keys_ownership(&full_source, &helper_source)?;
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
