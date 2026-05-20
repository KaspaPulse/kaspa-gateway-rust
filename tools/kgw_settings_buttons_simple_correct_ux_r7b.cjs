const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

const files = {
  node: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridge: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  lib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"),
};

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function saveJson(name, value) {
  fs.mkdirSync(reportDir, { recursive: true });
  fs.writeFileSync(path.join(reportDir, name), JSON.stringify(value, null, 2), "utf8");
}

function count(text, re) {
  const m = text.match(re);
  return m ? m.length : 0;
}

function functionBounds(text, name) {
  const re = new RegExp("function\\s+" + name + "\\s*\\([^)]*\\)\\s*\\{");
  const m = text.match(re);
  if (!m) throw new Error("Function not found: " + name);

  const start = m.index;
  const brace = text.indexOf("{", start);
  let depth = 0;
  let end = brace;

  for (; end < text.length; end++) {
    const ch = text[end];
    if (ch === "{") depth++;
    if (ch === "}") {
      depth--;
      if (depth === 0) {
        end++;
        break;
      }
    }
  }

  return { start, end, body: text.slice(start, end) };
}

function replaceFunction(text, name, replacement) {
  const b = functionBounds(text, name);
  return text.slice(0, b.start) + replacement + text.slice(b.end);
}

function insertBeforeFunction(text, name, block) {
  const idx = text.indexOf("function " + name);
  if (idx < 0) throw new Error("Insertion point not found before function: " + name);
  return text.slice(0, idx) + block + text.slice(idx);
}

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function removeExistingR7(text, scope) {
  text = removeBlock(text, `KGW_${scope}_SETTINGS_SIMPLE_UX_R7_START`, `KGW_${scope}_SETTINGS_SIMPLE_UX_R7_END`);
  text = removeBlock(text, `KGW_${scope}_SETTINGS_SIMPLE_UX_R7B_START`, `KGW_${scope}_SETTINGS_SIMPLE_UX_R7B_END`);
  return text;
}

function replaceInstallActionsListeners(text, kind, inputBlock, changeBlock, initialLine) {
  const b = functionBounds(text, "installActions");
  let body = b.body;

  const installHeader = body.match(/function\s+installActions\s*\(\s*root\s*\)\s*\{/);
  if (!installHeader) throw new Error(kind + " installActions header not found");

  const removeListener = (src, eventName) => {
    const re = new RegExp("\\n\\s*root\\.addEventListener\\(\\s*[\"']" + eventName + "[\"']\\s*,[\\s\\S]*?\\n\\s*\\}\\);", "m");
    const next = src.replace(re, "");
    if (next === src) {
      throw new Error(kind + " could not remove root.addEventListener(" + eventName + ")");
    }
    return next;
  };

  body = removeListener(body, "input");
  body = removeListener(body, "change");

  body = body.replace(
    /function\s+installActions\s*\(\s*root\s*\)\s*\{\s*/,
    "function installActions(root) {\n" + initialLine + inputBlock + changeBlock + "\n"
  );

  return text.slice(0, b.start) + body + text.slice(b.end);
}

function nodeBlock() {
  return `
/* KGW_NODE_SETTINGS_SIMPLE_UX_R7B_START */
const kgwNodeSettingsBaselineR7B = new Map();

function kgwNodeSettingsNormalizeR7B(value) {
  const normalize = (item) => {
    if (item === null || typeof item !== "object") return item;
    if (Array.isArray(item)) return item.map(normalize);
    return Object.keys(item).sort().reduce((acc, key) => {
      acc[key] = normalize(item[key]);
      return acc;
    }, {});
  };
  return JSON.stringify(normalize(value || {}));
}

function kgwNodeSettingsSnapshotR7B(net) {
  return kgwNodeSettingsNormalizeR7B(kgwNodeR51ReadSettings(net));
}

function kgwNodeSettingsSetButtonsEnabledR7B(net, enabled, reason = "unknown") {
  ["save-settings", "restore-defaults", "set-defaults"].forEach((action) => {
    kgwNodeSetSettingsActionEnabledR2(net, action, Boolean(enabled));
  });

  if (typeof kgwNodeSettingsTraceR4D === "function") {
    kgwNodeSettingsTraceR4D(net, "settings-buttons", enabled ? "enabled" : "disabled", {
      reason,
      nativeDisabledExpected: !enabled,
    });
  }
}

function kgwNodeSettingsAcceptBaselineR7B(net, reason = "baseline") {
  kgwNodeSettingsBaselineR7B.set(net, kgwNodeSettingsSnapshotR7B(net));
  kgwNodeSettingsSetButtonsEnabledR7B(net, false, reason);
}

function kgwNodeSettingsChangedR7B(net, reason = "change") {
  if (!net) return;
  if (!kgwNodeSettingsBaselineR7B.has(net)) {
    kgwNodeSettingsAcceptBaselineR7B(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = kgwNodeSettingsSnapshotR7B(net);
  const baseline = kgwNodeSettingsBaselineR7B.get(net);
  kgwNodeSettingsSetButtonsEnabledR7B(net, current !== baseline, reason);
}

function kgwNodeSettingsNetFromEventR7B(event) {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i], [class*='mainnet' i], [class*='testnet10' i], [class*='testnet12' i]");
  const raw = [
    target?.dataset?.net,
    target?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    target?.id,
    carrier?.id,
    carrier?.className,
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";
  return "";
}

function kgwNodeSettingsLanguageR7B() {
  const raw = [
    document.documentElement?.lang,
    document.body?.getAttribute("lang"),
    localStorage.getItem("kgw-language"),
    localStorage.getItem("kgw_lang"),
    localStorage.getItem("language"),
    localStorage.getItem("locale"),
    "en",
  ].filter(Boolean)[0];

  const lang = String(raw || "en").toLowerCase().split("-")[0];
  return ["ar", "de", "en", "es", "fr"].includes(lang) ? lang : "en";
}

function kgwNodeSettingsFeedbackLabelR7B(action) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };
  const lang = kgwNodeSettingsLanguageR7B();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function kgwNodeSettingsInstallInitialBaselineR7B() {
  if (typeof NODE_NETWORKS === "undefined") return;

  const apply = (reason) => {
    NODE_NETWORKS.forEach((net) => kgwNodeSettingsAcceptBaselineR7B(net.key, reason));
  };

  apply("initial-load");
  window.setTimeout(() => apply("initial-load+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load+2500ms"), 2500);
}

function kgwNodeSettingsAfterActionR7B(net, action) {
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7B(net, "after-" + action), 0);
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7B(net, "after-" + action + "+1000ms"), 1000);
}
/* KGW_NODE_SETTINGS_SIMPLE_UX_R7B_END */

`;
}

function bridgeBlock() {
  return `
/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7B_START */
const kgwBridgeSettingsBaselineR7B = new Map();

function kgwBridgeSettingsNormalizeR7B(value) {
  const normalize = (item) => {
    if (item === null || typeof item !== "object") return item;
    if (Array.isArray(item)) return item.map(normalize);
    return Object.keys(item).sort().reduce((acc, key) => {
      acc[key] = normalize(item[key]);
      return acc;
    }, {});
  };
  return JSON.stringify(normalize(value || {}));
}

function kgwBridgeSettingsSnapshotR7B(net) {
  return kgwBridgeSettingsNormalizeR7B(kgwBridgeR51ReadSettings(net));
}

function kgwBridgeSettingsSetButtonsEnabledR7B(net, enabled, reason = "unknown") {
  ["save-settings", "restore-defaults", "set-defaults"].forEach((action) => {
    kgwBridgeSetSettingsActionEnabledR2(net, action, Boolean(enabled));
  });

  if (typeof kgwBridgeSettingsTraceR4D === "function") {
    kgwBridgeSettingsTraceR4D(net, "settings-buttons", enabled ? "enabled" : "disabled", {
      reason,
      nativeDisabledExpected: !enabled,
    });
  }
}

function kgwBridgeSettingsAcceptBaselineR7B(net, reason = "baseline") {
  kgwBridgeSettingsBaselineR7B.set(net, kgwBridgeSettingsSnapshotR7B(net));
  kgwBridgeSettingsSetButtonsEnabledR7B(net, false, reason);
}

function kgwBridgeSettingsChangedR7B(net, reason = "change") {
  if (!net) return;
  if (!kgwBridgeSettingsBaselineR7B.has(net)) {
    kgwBridgeSettingsAcceptBaselineR7B(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = kgwBridgeSettingsSnapshotR7B(net);
  const baseline = kgwBridgeSettingsBaselineR7B.get(net);
  kgwBridgeSettingsSetButtonsEnabledR7B(net, current !== baseline, reason);
}

function kgwBridgeSettingsNetFromEventR7B(event, root) {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i], [class*='mainnet' i], [class*='testnet10' i], [class*='testnet12' i]");
  const raw = [
    target?.dataset?.net,
    target?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    target?.id,
    carrier?.id,
    carrier?.className,
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";

  try {
    return kgwBridgeCurrentVisibleNetwork(root) || "";
  } catch {
    return "";
  }
}

function kgwBridgeSettingsLanguageR7B() {
  const raw = [
    document.documentElement?.lang,
    document.body?.getAttribute("lang"),
    localStorage.getItem("kgw-language"),
    localStorage.getItem("kgw_lang"),
    localStorage.getItem("language"),
    localStorage.getItem("locale"),
    "en",
  ].filter(Boolean)[0];

  const lang = String(raw || "en").toLowerCase().split("-")[0];
  return ["ar", "de", "en", "es", "fr"].includes(lang) ? lang : "en";
}

function kgwBridgeSettingsFeedbackLabelR7B(action) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };
  const lang = kgwBridgeSettingsLanguageR7B();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function kgwBridgeSettingsInstallInitialBaselineR7B() {
  if (typeof BRIDGE_NETWORKS === "undefined") return;

  const apply = (reason) => {
    BRIDGE_NETWORKS.forEach((net) => kgwBridgeSettingsAcceptBaselineR7B(net.key, reason));
  };

  apply("initial-load");
  window.setTimeout(() => apply("initial-load+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load+2500ms"), 2500);
}

function kgwBridgeSettingsAfterActionR7B(net, action) {
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7B(net, "after-" + action), 0);
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7B(net, "after-" + action + "+1000ms"), 1000);
}
/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7B_END */

`;
}

function patchCommon(kind, text) {
  const isNode = kind === "node";
  const cap = isNode ? "NODE" : "BRIDGE";
  const prefix = isNode ? "kgwNode" : "kgwBridge";
  const dataAction = isNode ? "nodeAction" : "bridgeAction";
  const selectorAction = isNode ? "data-node-action" : "data-bridge-action";
  const readFn = `${prefix}R51ReadSettings`;
  const setEnabledFn = `${prefix}SetSettingsActionEnabledR2`;
  const flashFn = `${prefix}FlashSettingsActionButtonR2`;
  const block = isNode ? nodeBlock() : bridgeBlock();

  text = removeExistingR7(text, cap);
  text = insertBeforeFunction(text, readFn, block);

  text = replaceFunction(text, setEnabledFn, `function ${setEnabledFn}(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[${selectorAction}="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = !enabled;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);
  });
}`);

  text = replaceFunction(text, flashFn, `function ${flashFn}(button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const action = button.dataset.${dataAction};
  const net = button.dataset.net || button.dataset.network || "";

  button.dataset.kgwOriginalLabel = original;

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[${selectorAction}="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.textContent = label;
  });

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR7B);
  button.__kgwSettingsFeedbackTimerR7B = window.setTimeout(() => {
    targets.forEach((target) => {
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });
  }, 10000);
}`);

  const inputBlock = isNode
    ? `  root.addEventListener("input", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7B(event);
    if (net) kgwNodeSettingsChangedR7B(net, "input");
  });

`
    : `  root.addEventListener("input", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7B(event, root);
    if (net) kgwBridgeSettingsChangedR7B(net, "input");
  });

`;

  const changeBlock = isNode
    ? `  root.addEventListener("change", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7B(event);
    if (net) kgwNodeSettingsChangedR7B(net, "change");
  });

`
    : `  root.addEventListener("change", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7B(event, root);
    if (net) kgwBridgeSettingsChangedR7B(net, "change");
  });

`;

  const initialLine = isNode
    ? "  kgwNodeSettingsInstallInitialBaselineR7B();\n\n"
    : "  kgwBridgeSettingsInstallInitialBaselineR7B();\n\n";

  text = replaceInstallActionsListeners(text, kind, inputBlock, changeBlock, initialLine);

  const actionLabelR4D = isNode ? "kgwNodeSettingsActionLabelR4D" : "kgwBridgeSettingsActionLabelR4D";
  const feedbackR7B = isNode ? "kgwNodeSettingsFeedbackLabelR7B" : "kgwBridgeSettingsFeedbackLabelR7B";
  const afterR7B = isNode ? "kgwNodeSettingsAfterActionR7B" : "kgwBridgeSettingsAfterActionR7B";
  const updateDirtyR4D = isNode ? "kgwNodeUpdateSettingsDirtyButtonsR4D" : "kgwBridgeUpdateSettingsDirtyButtonsR4D";
  const updateAllR4D = isNode ? "kgwNodeUpdateAllSettingsDirtyButtonsR4D" : "kgwBridgeUpdateAllSettingsDirtyButtonsR4D";
  const refreshR6 = isNode ? "kgwNodeRefreshSettingsDirtyAfterActionR6" : "kgwBridgeRefreshSettingsDirtyAfterActionR6";
  const traceR4D = isNode ? "kgwNodeSettingsTraceR4D" : "kgwBridgeSettingsTraceR4D";

  text = text.replace(new RegExp(`${prefix}FlashSettingsActionButtonR2\\(button,\\s*${actionLabelR4D}\\(action,\\s*"Saved"\\)\\);\\s*\\n\\s*(?:${updateDirtyR4D}\\(net,\\s*"after-save-settings"\\);|${updateAllR4D}\\("after-save-settings"\\);)`, "g"),
    `${prefix}FlashSettingsActionButtonR2(button, ${feedbackR7B}(action));\n      ${afterR7B}(net, action)`);

  text = text.replace(new RegExp(`${prefix}FlashSettingsActionButtonR2\\(button,\\s*${actionLabelR4D}\\(action,\\s*"Restored"\\)\\);\\s*\\n\\s*(?:${refreshR6}\\(net,\\s*"after-restore-defaults"\\);|${updateDirtyR4D}\\(net,\\s*"after-restore-defaults"\\);\\s*\\n\\s*window\\.setTimeout\\(\\(\\) => ${updateDirtyR4D}\\(net,\\s*"after-restore-defaults\\+900ms"\\),\\s*900\\);|${updateAllR4D}\\("after-restore-defaults"\\);\\s*\\n\\s*window\\.setTimeout\\(\\(\\) => ${updateAllR4D}\\("after-restore-defaults\\+900ms"\\),\\s*900\\);)`, "g"),
    `${prefix}FlashSettingsActionButtonR2(button, ${feedbackR7B}(action));\n      ${afterR7B}(net, action)`);

  text = text.replace(new RegExp(`${prefix}FlashSettingsActionButtonR2\\(button,\\s*${actionLabelR4D}\\(action,\\s*"Set as defaults"\\)\\);\\s*\\n\\s*(?:${updateDirtyR4D}\\(net,\\s*"after-set-defaults"\\);|${updateAllR4D}\\("after-set-defaults"\\);)`, "g"),
    `${prefix}FlashSettingsActionButtonR2(button, ${feedbackR7B}(action));\n      ${afterR7B}(net, action)`);

  text = text.replace(new RegExp(`${traceR4D}\\(net, action, "action-complete", \\{ label: ${actionLabelR4D}\\(action, "Saved"\\) \\}\\);`, "g"),
    `${traceR4D}(net, action, "action-complete", { label: ${feedbackR7B}(action), holdMs: 10000 });`);

  text = text.replace(new RegExp(`${traceR4D}\\(net, action, "action-complete", \\{ label: ${actionLabelR4D}\\(action, "Restored"\\) \\}\\);`, "g"),
    `${traceR4D}(net, action, "action-complete", { label: ${feedbackR7B}(action), holdMs: 10000 });`);

  text = text.replace(new RegExp(`${traceR4D}\\(net, action, "action-complete", \\{ label: ${actionLabelR4D}\\(action, "Set as defaults"\\) \\}\\);`, "g"),
    `${traceR4D}(net, action, "action-complete", { label: ${feedbackR7B}(action), holdMs: 10000 });`);

  return text;
}

function patchNode() {
  const before = read(files.node);
  const text = patchCommon("node", before);
  write(files.node, text);

  return {
    changed: before !== text,
    r7bMarkers: count(text, /KGW_NODE_SETTINGS_SIMPLE_UX_R7B_START/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    feedbackTenSeconds: count(text, /10000/g),
    initialBaselineCalls: count(text, /kgwNodeSettingsInstallInitialBaselineR7B\(\)/g),
    r7bLabels: count(text, /kgwNodeSettingsFeedbackLabelR7B/g),
  };
}

function patchBridge() {
  const before = read(files.bridge);
  const text = patchCommon("bridge", before);
  write(files.bridge, text);

  return {
    changed: before !== text,
    r7bMarkers: count(text, /KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7B_START/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    feedbackTenSeconds: count(text, /10000/g),
    initialBaselineCalls: count(text, /kgwBridgeSettingsInstallInitialBaselineR7B\(\)/g),
    r7bLabels: count(text, /kgwBridgeSettingsFeedbackLabelR7B/g),
  };
}

function auditAfter() {
  const node = read(files.node);
  const bridge = read(files.bridge);
  const lib = read(files.lib);

  return {
    node: {
      r7bMarkers: count(node, /KGW_NODE_SETTINGS_SIMPLE_UX_R7B_START/g),
      nativeDisabledByEnabled: count(node, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      nativeAlwaysEnabledSettingsOwner: count(node, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*false[\s\S]*?\}/g),
      feedbackTenSeconds: count(node, /kgwNodeFlashSettingsActionButtonR2[\s\S]*?10000/g),
      initialBaselineCalls: count(node, /kgwNodeSettingsInstallInitialBaselineR7B\(\)/g),
      r7bLabels: count(node, /kgwNodeSettingsFeedbackLabelR7B/g),
    },
    bridge: {
      r7bMarkers: count(bridge, /KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7B_START/g),
      nativeDisabledByEnabled: count(bridge, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      nativeAlwaysEnabledSettingsOwner: count(bridge, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*false[\s\S]*?\}/g),
      feedbackTenSeconds: count(bridge, /kgwBridgeFlashSettingsActionButtonR2[\s\S]*?10000/g),
      initialBaselineCalls: count(bridge, /kgwBridgeSettingsInstallInitialBaselineR7B\(\)/g),
      r7bLabels: count(bridge, /kgwBridgeSettingsFeedbackLabelR7B/g),
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r7b.json", {
  nodeLength: read(files.node).length,
  bridgeLength: read(files.bridge).length,
  rustLength: read(files.lib).length,
});

const changes = {
  node: patchNode(),
  bridge: patchBridge(),
};

saveJson("patch-changes-r7b.json", changes);

const after = auditAfter();
saveJson("audit-after-r7b.json", after);

const failures = [];

if (after.node.r7bMarkers !== 1) failures.push("Node R7B marker must exist exactly once.");
if (after.bridge.r7bMarkers !== 1) failures.push("Bridge R7B marker must exist exactly once.");
if (after.node.nativeDisabledByEnabled < 1) failures.push("Node buttons must use native disabled=!enabled.");
if (after.bridge.nativeDisabledByEnabled < 1) failures.push("Bridge buttons must use native disabled=!enabled.");
if (after.node.nativeAlwaysEnabledSettingsOwner !== 0) failures.push("Node settings owner still forces disabled=false.");
if (after.bridge.nativeAlwaysEnabledSettingsOwner !== 0) failures.push("Bridge settings owner still forces disabled=false.");
if (after.node.feedbackTenSeconds < 1) failures.push("Node feedback timer must be 10000ms.");
if (after.bridge.feedbackTenSeconds < 1) failures.push("Bridge feedback timer must be 10000ms.");
if (after.node.initialBaselineCalls < 1) failures.push("Node initial baseline install missing.");
if (after.bridge.initialBaselineCalls < 1) failures.push("Bridge initial baseline install missing.");
if (after.node.r7bLabels < 3) failures.push("Node language-aware R7B labels missing.");
if (after.bridge.r7bLabels < 3) failures.push("Bridge language-aware R7B labels missing.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R7B validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R7B patch complete");
console.log(JSON.stringify(after, null, 2));