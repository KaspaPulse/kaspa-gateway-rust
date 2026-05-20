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

function removeOldSimpleBlocks(text, scope) {
  for (const suffix of ["R7", "R7B", "R7C"]) {
    text = removeBlock(text, `KGW_${scope}_SETTINGS_SIMPLE_UX_${suffix}_START`, `KGW_${scope}_SETTINGS_SIMPLE_UX_${suffix}_END`);
  }
  return text;
}

function replaceInstallActionBranch(body, action, replacement) {
  const needle = `if (action === "${action}") {`;
  const start = body.indexOf(needle);
  if (start < 0) throw new Error("Action branch not found: " + action);

  const brace = body.indexOf("{", start);
  let depth = 0;
  let end = brace;

  for (; end < body.length; end++) {
    const ch = body[end];
    if (ch === "{") depth++;
    if (ch === "}") {
      depth--;
      if (depth === 0) {
        end++;
        break;
      }
    }
  }

  return body.slice(0, start) + replacement + body.slice(end);
}

function replaceInstallListeners(body, kind, inputBlock, changeBlock, initialLine) {
  const headerRe = /function\s+installActions\s*\(\s*root\s*\)\s*\{/;
  if (!headerRe.test(body)) throw new Error(kind + " installActions header not found");

  const removeListener = (src, eventName) => {
    const idx = src.indexOf(`root.addEventListener("${eventName}"`);
    if (idx < 0) throw new Error(kind + " listener not found: " + eventName);

    const paren = src.indexOf("(", idx);
    let depth = 0;
    let end = paren;

    for (; end < src.length; end++) {
      const ch = src[end];
      if (ch === "(") depth++;
      if (ch === ")") {
        depth--;
        if (depth === 0) {
          end++;
          break;
        }
      }
    }

    if (src.slice(end, end + 1) === ";") end++;
    return src.slice(0, idx) + src.slice(end);
  };

  body = removeListener(body, "input");
  body = removeListener(body, "change");

  body = body.replace(headerRe, (m) => m + "\n" + initialLine + inputBlock + changeBlock);
  return body;
}

function nodeBlock() {
  return `
/* KGW_NODE_SETTINGS_SIMPLE_UX_R7C_START */
const kgwNodeSettingsBaselineR7C = new Map();

function kgwNodeSettingsNormalizeR7C(value) {
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

function kgwNodeSettingsSnapshotR7C(net) {
  return kgwNodeSettingsNormalizeR7C(kgwNodeR51ReadSettings(net));
}

function kgwNodeSettingsSetButtonsEnabledR7C(net, enabled, reason = "unknown") {
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

function kgwNodeSettingsAcceptBaselineR7C(net, reason = "baseline") {
  kgwNodeSettingsBaselineR7C.set(net, kgwNodeSettingsSnapshotR7C(net));
  kgwNodeSettingsSetButtonsEnabledR7C(net, false, reason);
}

function kgwNodeSettingsChangedR7C(net, reason = "change") {
  if (!net) return;

  if (!kgwNodeSettingsBaselineR7C.has(net)) {
    kgwNodeSettingsAcceptBaselineR7C(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = kgwNodeSettingsSnapshotR7C(net);
  const baseline = kgwNodeSettingsBaselineR7C.get(net);
  kgwNodeSettingsSetButtonsEnabledR7C(net, current !== baseline, reason);
}

function kgwNodeSettingsNetFromEventR7C(event) {
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

function kgwNodeSettingsLanguageR7C() {
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

function kgwNodeSettingsFeedbackLabelR7C(action) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = kgwNodeSettingsLanguageR7C();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function kgwNodeSettingsInstallInitialBaselineR7C() {
  if (typeof NODE_NETWORKS === "undefined") return;

  const apply = (reason) => {
    NODE_NETWORKS.forEach((net) => kgwNodeSettingsAcceptBaselineR7C(net.key, reason));
  };

  apply("initial-load");
  window.setTimeout(() => apply("initial-load+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load+2500ms"), 2500);
}

function kgwNodeSettingsAfterActionR7C(net, action) {
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action), 0);
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action + "+1000ms"), 1000);
}
/* KGW_NODE_SETTINGS_SIMPLE_UX_R7C_END */

`;
}

function bridgeBlock() {
  return `
/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C_START */
const kgwBridgeSettingsBaselineR7C = new Map();

function kgwBridgeSettingsNormalizeR7C(value) {
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

function kgwBridgeSettingsSnapshotR7C(net) {
  return kgwBridgeSettingsNormalizeR7C(kgwBridgeR51ReadSettings(net));
}

function kgwBridgeSettingsSetButtonsEnabledR7C(net, enabled, reason = "unknown") {
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

function kgwBridgeSettingsAcceptBaselineR7C(net, reason = "baseline") {
  kgwBridgeSettingsBaselineR7C.set(net, kgwBridgeSettingsSnapshotR7C(net));
  kgwBridgeSettingsSetButtonsEnabledR7C(net, false, reason);
}

function kgwBridgeSettingsChangedR7C(net, reason = "change") {
  if (!net) return;

  if (!kgwBridgeSettingsBaselineR7C.has(net)) {
    kgwBridgeSettingsAcceptBaselineR7C(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = kgwBridgeSettingsSnapshotR7C(net);
  const baseline = kgwBridgeSettingsBaselineR7C.get(net);
  kgwBridgeSettingsSetButtonsEnabledR7C(net, current !== baseline, reason);
}

function kgwBridgeSettingsNetFromEventR7C(event, root) {
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

function kgwBridgeSettingsLanguageR7C() {
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

function kgwBridgeSettingsFeedbackLabelR7C(action) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = kgwBridgeSettingsLanguageR7C();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function kgwBridgeSettingsInstallInitialBaselineR7C() {
  if (typeof BRIDGE_NETWORKS === "undefined") return;

  const apply = (reason) => {
    BRIDGE_NETWORKS.forEach((net) => kgwBridgeSettingsAcceptBaselineR7C(net.key, reason));
  };

  apply("initial-load");
  window.setTimeout(() => apply("initial-load+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load+2500ms"), 2500);
}

function kgwBridgeSettingsAfterActionR7C(net, action) {
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action), 0);
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action + "+1000ms"), 1000);
}
/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C_END */

`;
}

function patchCommon(kind, text) {
  const isNode = kind === "node";
  const scope = isNode ? "NODE" : "BRIDGE";
  const prefix = isNode ? "kgwNode" : "kgwBridge";
  const actionDataset = isNode ? "nodeAction" : "bridgeAction";
  const actionSelector = isNode ? "data-node-action" : "data-bridge-action";
  const networksConst = isNode ? "NODE_NETWORKS" : "BRIDGE_NETWORKS";
  const readFn = `${prefix}R51ReadSettings`;
  const setEnabledFn = `${prefix}SetSettingsActionEnabledR2`;
  const flashFn = `${prefix}FlashSettingsActionButtonR2`;
  const block = isNode ? nodeBlock() : bridgeBlock();

  text = removeOldSimpleBlocks(text, scope);
  text = insertBeforeFunction(text, readFn, block);

  text = replaceFunction(text, setEnabledFn, `function ${setEnabledFn}(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[${actionSelector}="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = !enabled;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);
  });
}`);

  text = replaceFunction(text, flashFn, `function ${flashFn}(button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const action = button.dataset.${actionDataset};
  const net = button.dataset.net || button.dataset.network || "";

  button.dataset.kgwOriginalLabel = original;

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[${actionSelector}="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.textContent = label;
  });

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR7C);
  button.__kgwSettingsFeedbackTimerR7C = window.setTimeout(() => {
    targets.forEach((target) => {
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });
  }, 10000);
}`);

  const inputBlock = isNode
    ? `  root.addEventListener("input", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "input");
  });

`
    : `  root.addEventListener("input", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "input");
  });

`;

  const changeBlock = isNode
    ? `  root.addEventListener("change", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "change");
  });

`
    : `  root.addEventListener("change", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "change");
  });

`;

  const initialLine = isNode
    ? `  kgwNodeSettingsInstallInitialBaselineR7C();

`
    : `  kgwBridgeSettingsInstallInitialBaselineR7C();

`;

  const b = functionBounds(text, "installActions");
  let body = b.body;
  body = replaceInstallListeners(body, kind, inputBlock, changeBlock, initialLine);

  const saveBranch = isNode
    ? `    if (action === "save-settings") {
      kgwNodeSettingsTraceR4D(net, action, "action-start", { beforeDisabled: Boolean(button.disabled) });
      kgwNodeR51SaveSettings(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsFeedbackLabelR7C(action));
      kgwNodeSettingsAfterActionR7C(net, action);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsFeedbackLabelR7C(action), holdMs: 10000 });
      return;
    }`
    : `    if (action === "save-settings") {
      kgwBridgeSettingsTraceR4D(net, action, "action-start", { beforeDisabled: Boolean(button.disabled), instanceId });
      kgwBridgeR51SaveSettings(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsFeedbackLabelR7C(action));
      kgwBridgeSettingsAfterActionR7C(net, action);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsFeedbackLabelR7C(action), holdMs: 10000, instanceId });
      return;
    }`;

  const restoreBranch = isNode
    ? `    if (action === "restore-defaults") {
      kgwNodeSettingsTraceR4D(net, action, "action-start", { beforeDisabled: Boolean(button.disabled) });
      kgwNodeR51RestoreDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsFeedbackLabelR7C(action));
      kgwNodeSettingsAfterActionR7C(net, action);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsFeedbackLabelR7C(action), holdMs: 10000 });
      return;
    }`
    : `    if (action === "restore-defaults") {
      kgwBridgeSettingsTraceR4D(net, action, "action-start", { beforeDisabled: Boolean(button.disabled), instanceId });
      kgwBridgeR51RestoreDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsFeedbackLabelR7C(action));
      kgwBridgeSettingsAfterActionR7C(net, action);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsFeedbackLabelR7C(action), holdMs: 10000, instanceId });
      return;
    }`;

  const defaultsBranch = isNode
    ? `    if (action === "set-defaults") {
      kgwNodeSettingsTraceR4D(net, action, "action-start", { beforeDisabled: Boolean(button.disabled) });
      kgwNodeR51SetAsDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsFeedbackLabelR7C(action));
      kgwNodeSettingsAfterActionR7C(net, action);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsFeedbackLabelR7C(action), holdMs: 10000 });
      return;
    }`
    : `    if (action === "set-defaults") {
      kgwBridgeSettingsTraceR4D(net, action, "action-start", { beforeDisabled: Boolean(button.disabled), instanceId });
      kgwBridgeR51SetAsDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsFeedbackLabelR7C(action));
      kgwBridgeSettingsAfterActionR7C(net, action);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsFeedbackLabelR7C(action), holdMs: 10000, instanceId });
      return;
    }`;

  body = replaceInstallActionBranch(body, "save-settings", saveBranch);
  body = replaceInstallActionBranch(body, "restore-defaults", restoreBranch);
  body = replaceInstallActionBranch(body, "set-defaults", defaultsBranch);

  text = text.slice(0, b.start) + body + text.slice(b.end);
  return text;
}

function patchNode() {
  const before = read(files.node);
  const text = patchCommon("node", before);
  write(files.node, text);

  return {
    changed: before !== text,
    r7cMarkers: count(text, /KGW_NODE_SETTINGS_SIMPLE_UX_R7C_START/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    feedbackTenSeconds: count(text, /10000/g),
    initialBaselineCalls: count(text, /kgwNodeSettingsInstallInitialBaselineR7C\(\)/g),
    labels: count(text, /kgwNodeSettingsFeedbackLabelR7C/g),
  };
}

function patchBridge() {
  const before = read(files.bridge);
  const text = patchCommon("bridge", before);
  write(files.bridge, text);

  return {
    changed: before !== text,
    r7cMarkers: count(text, /KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C_START/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    feedbackTenSeconds: count(text, /10000/g),
    initialBaselineCalls: count(text, /kgwBridgeSettingsInstallInitialBaselineR7C\(\)/g),
    labels: count(text, /kgwBridgeSettingsFeedbackLabelR7C/g),
  };
}

function auditAfter() {
  const node = read(files.node);
  const bridge = read(files.bridge);
  const lib = read(files.lib);

  const nodeSetFn = functionBounds(node, "kgwNodeSetSettingsActionEnabledR2").body;
  const bridgeSetFn = functionBounds(bridge, "kgwBridgeSetSettingsActionEnabledR2").body;
  const nodeFlashFn = functionBounds(node, "kgwNodeFlashSettingsActionButtonR2").body;
  const bridgeFlashFn = functionBounds(bridge, "kgwBridgeFlashSettingsActionButtonR2").body;
  const nodeInstall = functionBounds(node, "installActions").body;
  const bridgeInstall = functionBounds(bridge, "installActions").body;

  return {
    node: {
      r7cMarkers: count(node, /KGW_NODE_SETTINGS_SIMPLE_UX_R7C_START/g),
      setFnNativeDisabledByEnabled: nodeSetFn.includes("button.disabled = !enabled"),
      setFnForcesDisabledFalse: nodeSetFn.includes("button.disabled = false"),
      feedbackTenSeconds: nodeFlashFn.includes("10000"),
      initialBaselineInInstallActions: nodeInstall.includes("kgwNodeSettingsInstallInitialBaselineR7C()"),
      actionLabels: count(nodeInstall, /kgwNodeSettingsFeedbackLabelR7C/g),
      actionAfterBaseline: count(nodeInstall, /kgwNodeSettingsAfterActionR7C/g),
    },
    bridge: {
      r7cMarkers: count(bridge, /KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C_START/g),
      setFnNativeDisabledByEnabled: bridgeSetFn.includes("button.disabled = !enabled"),
      setFnForcesDisabledFalse: bridgeSetFn.includes("button.disabled = false"),
      feedbackTenSeconds: bridgeFlashFn.includes("10000"),
      initialBaselineInInstallActions: bridgeInstall.includes("kgwBridgeSettingsInstallInitialBaselineR7C()"),
      actionLabels: count(bridgeInstall, /kgwBridgeSettingsFeedbackLabelR7C/g),
      actionAfterBaseline: count(bridgeInstall, /kgwBridgeSettingsAfterActionR7C/g),
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r7c.json", {
  nodeLength: read(files.node).length,
  bridgeLength: read(files.bridge).length,
  rustLength: read(files.lib).length,
});

const changes = {
  node: patchNode(),
  bridge: patchBridge(),
};

saveJson("patch-changes-r7c.json", changes);

const after = auditAfter();
saveJson("audit-after-r7c.json", after);

const failures = [];

if (after.node.r7cMarkers !== 1) failures.push("Node R7C marker must exist exactly once.");
if (after.bridge.r7cMarkers !== 1) failures.push("Bridge R7C marker must exist exactly once.");
if (!after.node.setFnNativeDisabledByEnabled) failures.push("Node settings buttons must use native disabled=!enabled.");
if (!after.bridge.setFnNativeDisabledByEnabled) failures.push("Bridge settings buttons must use native disabled=!enabled.");
if (after.node.setFnForcesDisabledFalse) failures.push("Node set-enabled owner still forces disabled=false.");
if (after.bridge.setFnForcesDisabledFalse) failures.push("Bridge set-enabled owner still forces disabled=false.");
if (!after.node.feedbackTenSeconds) failures.push("Node feedback timer must be 10000ms.");
if (!after.bridge.feedbackTenSeconds) failures.push("Bridge feedback timer must be 10000ms.");
if (!after.node.initialBaselineInInstallActions) failures.push("Node initial baseline call missing in installActions.");
if (!after.bridge.initialBaselineInInstallActions) failures.push("Bridge initial baseline call missing in installActions.");
if (after.node.actionLabels < 3) failures.push("Node action branches must use R7C language labels.");
if (after.bridge.actionLabels < 3) failures.push("Bridge action branches must use R7C language labels.");
if (after.node.actionAfterBaseline < 3) failures.push("Node action branches must accept baseline after action.");
if (after.bridge.actionAfterBaseline < 3) failures.push("Bridge action branches must accept baseline after action.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R7C validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R7C patch complete");
console.log(JSON.stringify(after, null, 2));