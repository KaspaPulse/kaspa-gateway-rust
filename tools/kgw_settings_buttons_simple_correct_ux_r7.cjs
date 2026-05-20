const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node r7.cjs <repoRoot> <reportDir>");
}

const files = {
  node: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridge: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  lib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
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

function replaceFunction(text, name, replacement) {
  const re = new RegExp("function\\s+" + name + "\\s*\\([^)]*\\)\\s*\\{");
  const match = text.match(re);
  if (!match) throw new Error("Function not found: " + name);

  const start = match.index;
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

  return text.slice(0, start) + replacement + text.slice(end);
}

function insertBeforeFunction(text, functionName, block) {
  const idx = text.indexOf("function " + functionName);
  if (idx < 0) throw new Error("Insertion function not found: " + functionName);
  return text.slice(0, idx) + block + text.slice(idx);
}

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function replaceKnown(text, candidates, to, label) {
  for (const from of candidates) {
    if (text.includes(from)) {
      return text.split(from).join(to);
    }
  }
  throw new Error("Missing known target: " + label);
}

function nodeBlock() {
  return `
/* KGW_NODE_SETTINGS_SIMPLE_UX_R7_START */
const kgwNodeSettingsBaselineR7 = new Map();

function kgwNodeSettingsNormalizeR7(value) {
  const seen = new WeakSet();

  const normalize = (item) => {
    if (item === null || typeof item !== "object") return item;
    if (seen.has(item)) return "[Circular]";
    seen.add(item);

    if (Array.isArray(item)) return item.map(normalize);

    return Object.keys(item)
      .sort()
      .reduce((acc, key) => {
        acc[key] = normalize(item[key]);
        return acc;
      }, {});
  };

  return JSON.stringify(normalize(value || {}));
}

function kgwNodeSettingsSnapshotR7(net) {
  return kgwNodeSettingsNormalizeR7(kgwNodeR51ReadSettings(net));
}

function kgwNodeSettingsSetButtonsEnabledR7(net, enabled, reason = "unknown") {
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

function kgwNodeSettingsAcceptBaselineR7(net, reason = "baseline") {
  kgwNodeSettingsBaselineR7.set(net, kgwNodeSettingsSnapshotR7(net));
  kgwNodeSettingsSetButtonsEnabledR7(net, false, reason);

  if (typeof kgwNodeSettingsTraceR4D === "function") {
    kgwNodeSettingsTraceR4D(net, "settings-baseline", "accepted", {
      reason,
      snapshotLength: String(kgwNodeSettingsBaselineR7.get(net) || "").length,
    });
  }
}

function kgwNodeSettingsChangedR7(net, reason = "change") {
  if (!net) return;

  if (!kgwNodeSettingsBaselineR7.has(net)) {
    kgwNodeSettingsAcceptBaselineR7(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = kgwNodeSettingsSnapshotR7(net);
  const baseline = kgwNodeSettingsBaselineR7.get(net);
  const changed = current !== baseline;

  kgwNodeSettingsSetButtonsEnabledR7(net, changed, reason);

  if (typeof kgwNodeSettingsTraceR4D === "function") {
    kgwNodeSettingsTraceR4D(net, "settings-dirty", "evaluated", {
      reason,
      changed,
      currentLength: current.length,
      baselineLength: String(baseline || "").length,
    });
  }
}

function kgwNodeSettingsNetFromEventR7(event) {
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

function kgwNodeSettingsCurrentLanguageR7() {
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

function kgwNodeSettingsFeedbackLabelR7(action) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = kgwNodeSettingsCurrentLanguageR7();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function kgwNodeSettingsInstallInitialBaselineR7() {
  if (typeof NODE_NETWORKS === "undefined") return;

  const apply = (reason) => {
    NODE_NETWORKS.forEach((net) => {
      kgwNodeSettingsAcceptBaselineR7(net.key, reason);
    });
  };

  apply("initial-load");
  window.setTimeout(() => apply("initial-load+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load+2500ms"), 2500);
}

function kgwNodeSettingsAfterActionR7(net, action) {
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7(net, "after-" + action), 0);
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7(net, "after-" + action + "+1000ms"), 1000);
}
/* KGW_NODE_SETTINGS_SIMPLE_UX_R7_END */

`;
}

function bridgeBlock() {
  return `
/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7_START */
const kgwBridgeSettingsBaselineR7 = new Map();

function kgwBridgeSettingsNormalizeR7(value) {
  const seen = new WeakSet();

  const normalize = (item) => {
    if (item === null || typeof item !== "object") return item;
    if (seen.has(item)) return "[Circular]";
    seen.add(item);

    if (Array.isArray(item)) return item.map(normalize);

    return Object.keys(item)
      .sort()
      .reduce((acc, key) => {
        acc[key] = normalize(item[key]);
        return acc;
      }, {});
  };

  return JSON.stringify(normalize(value || {}));
}

function kgwBridgeSettingsSnapshotR7(net) {
  return kgwBridgeSettingsNormalizeR7(kgwBridgeR51ReadSettings(net));
}

function kgwBridgeSettingsSetButtonsEnabledR7(net, enabled, reason = "unknown") {
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

function kgwBridgeSettingsAcceptBaselineR7(net, reason = "baseline") {
  kgwBridgeSettingsBaselineR7.set(net, kgwBridgeSettingsSnapshotR7(net));
  kgwBridgeSettingsSetButtonsEnabledR7(net, false, reason);

  if (typeof kgwBridgeSettingsTraceR4D === "function") {
    kgwBridgeSettingsTraceR4D(net, "settings-baseline", "accepted", {
      reason,
      snapshotLength: String(kgwBridgeSettingsBaselineR7.get(net) || "").length,
    });
  }
}

function kgwBridgeSettingsChangedR7(net, reason = "change") {
  if (!net) return;

  if (!kgwBridgeSettingsBaselineR7.has(net)) {
    kgwBridgeSettingsAcceptBaselineR7(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = kgwBridgeSettingsSnapshotR7(net);
  const baseline = kgwBridgeSettingsBaselineR7.get(net);
  const changed = current !== baseline;

  kgwBridgeSettingsSetButtonsEnabledR7(net, changed, reason);

  if (typeof kgwBridgeSettingsTraceR4D === "function") {
    kgwBridgeSettingsTraceR4D(net, "settings-dirty", "evaluated", {
      reason,
      changed,
      currentLength: current.length,
      baselineLength: String(baseline || "").length,
    });
  }
}

function kgwBridgeSettingsNetFromEventR7(event, root) {
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

function kgwBridgeSettingsCurrentLanguageR7() {
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

function kgwBridgeSettingsFeedbackLabelR7(action) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = kgwBridgeSettingsCurrentLanguageR7();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function kgwBridgeSettingsInstallInitialBaselineR7() {
  if (typeof BRIDGE_NETWORKS === "undefined") return;

  const apply = (reason) => {
    BRIDGE_NETWORKS.forEach((net) => {
      kgwBridgeSettingsAcceptBaselineR7(net.key, reason);
    });
  };

  apply("initial-load");
  window.setTimeout(() => apply("initial-load+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load+2500ms"), 2500);
}

function kgwBridgeSettingsAfterActionR7(net, action) {
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7(net, "after-" + action), 0);
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7(net, "after-" + action + "+1000ms"), 1000);
}
/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7_END */

`;
}

function patchNode() {
  let text = read(files.node);
  const before = text;

  text = removeBlock(text, "KGW_NODE_SETTINGS_SIMPLE_UX_R7_START", "KGW_NODE_SETTINGS_SIMPLE_UX_R7_END");
  text = insertBeforeFunction(text, "kgwNodeR51ReadSettings", nodeBlock());

  text = replaceFunction(text, "kgwNodeSetSettingsActionEnabledR2", `function kgwNodeSetSettingsActionEnabledR2(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = !enabled;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);
  });
}`);

  text = replaceFunction(text, "kgwNodeFlashSettingsActionButtonR2", `function kgwNodeFlashSettingsActionButtonR2(button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const action = button.dataset.nodeAction;
  const net = button.dataset.net || "";

  button.dataset.kgwOriginalLabel = original;

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.textContent = label;
  });

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR7);
  button.__kgwSettingsFeedbackTimerR7 = window.setTimeout(() => {
    targets.forEach((target) => {
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });
  }, 10000);
}`);

  text = replaceKnown(text, [
    `function installActions(root) {
  root.addEventListener("input", (event) => {
    kgwNodeUpdateSettingsDirtyFromEventR6(event, "input");
  });

  root.addEventListener("change", (event) => {
    kgwNodeUpdateSettingsDirtyFromEventR6(event, "change");
  });`,
    `function installActions(root) {
  root.addEventListener("input", () => {
    updateAllCommands();
    kgwNodeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });

  root.addEventListener("change", () => {
    updateAllCommands();
    kgwNodeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });`
  ], `function installActions(root) {
  kgwNodeSettingsInstallInitialBaselineR7();

  root.addEventListener("input", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7(event);
    if (net) kgwNodeSettingsChangedR7(net, "input");
  });

  root.addEventListener("change", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7(event);
    if (net) kgwNodeSettingsChangedR7(net, "change");
  });`, "Node installActions simple baseline input/change");

  text = replaceKnown(text, [
    `      kgwNodeR51SaveSettings(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, "Saved"));
      kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-save-settings");
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsActionLabelR4D(action, "Saved") });`,
    `      kgwNodeR51SaveSettings(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, "Saved"));
      kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-save-settings");
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsActionLabelR4D(action, "Saved") });`
  ], `      kgwNodeR51SaveSettings(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsFeedbackLabelR7(action));
      kgwNodeSettingsAfterActionR7(net, action);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsFeedbackLabelR7(action), holdMs: 10000 });`, "Node save action R7");

  text = replaceKnown(text, [
    `      kgwNodeR51RestoreDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, "Restored"));
      kgwNodeRefreshSettingsDirtyAfterActionR6(net, "after-restore-defaults");
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsActionLabelR4D(action, "Restored") });`,
    `      kgwNodeR51RestoreDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, "Restored"));
      kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults");
      window.setTimeout(() => kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults+900ms"), 900);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsActionLabelR4D(action, "Restored") });`
  ], `      kgwNodeR51RestoreDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsFeedbackLabelR7(action));
      kgwNodeSettingsAfterActionR7(net, action);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsFeedbackLabelR7(action), holdMs: 10000 });`, "Node restore action R7");

  text = replaceKnown(text, [
    `      kgwNodeR51SetAsDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, "Set as defaults"));
      kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-set-defaults");
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsActionLabelR4D(action, "Set as defaults") });`,
    `      kgwNodeR51SetAsDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, "Set as defaults"));
      kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-set-defaults");
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsActionLabelR4D(action, "Set as defaults") });`
  ], `      kgwNodeR51SetAsDefaults(net);
      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsFeedbackLabelR7(action));
      kgwNodeSettingsAfterActionR7(net, action);
      kgwNodeSettingsTraceR4D(net, action, "action-complete", { label: kgwNodeSettingsFeedbackLabelR7(action), holdMs: 10000 });`, "Node set defaults action R7");

  write(files.node, text);

  return {
    changed: before !== text,
    r7Markers: count(text, /KGW_NODE_SETTINGS_SIMPLE_UX_R7_START/g),
    nativeDisabledFalseOwner: count(text, /button\.disabled\s*=\s*false/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    feedbackTenSeconds: count(text, /10000/g),
    initialBaselineCalls: count(text, /kgwNodeSettingsInstallInitialBaselineR7\(\)/g),
    r7FeedbackLabels: count(text, /kgwNodeSettingsFeedbackLabelR7/g)
  };
}

function patchBridge() {
  let text = read(files.bridge);
  const before = text;

  text = removeBlock(text, "KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7_START", "KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7_END");
  text = insertBeforeFunction(text, "kgwBridgeR51ReadSettings", bridgeBlock());

  text = replaceFunction(text, "kgwBridgeSetSettingsActionEnabledR2", `function kgwBridgeSetSettingsActionEnabledR2(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = !enabled;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);
  });
}`);

  text = replaceFunction(text, "kgwBridgeFlashSettingsActionButtonR2", `function kgwBridgeFlashSettingsActionButtonR2(button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const action = button.dataset.bridgeAction;
  const net = button.dataset.net || button.dataset.network || "";

  button.dataset.kgwOriginalLabel = original;

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.textContent = label;
  });

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR7);
  button.__kgwSettingsFeedbackTimerR7 = window.setTimeout(() => {
    targets.forEach((target) => {
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });
  }, 10000);
}`);

  text = replaceKnown(text, [
    `function installActions(root) {
  root.addEventListener("input", (event) => {
    kgwBridgeUpdateSettingsDirtyFromEventR6(event, root, "input");
  });

  root.addEventListener("change", (event) => {
    kgwBridgeUpdateSettingsDirtyFromEventR6(event, root, "change");
  });`,
    `function installActions(root) {
  root.addEventListener("input", () => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    kgwBridgeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });

  root.addEventListener("change", () => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    kgwBridgeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });`
  ], `function installActions(root) {
  kgwBridgeSettingsInstallInitialBaselineR7();

  root.addEventListener("input", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7(event, root);
    if (net) kgwBridgeSettingsChangedR7(net, "input");
  });

  root.addEventListener("change", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7(event, root);
    if (net) kgwBridgeSettingsChangedR7(net, "change");
  });`, "Bridge installActions simple baseline input/change");

  text = replaceKnown(text, [
    `      kgwBridgeR51SaveSettings(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, "Saved"));
      kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-save-settings");
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsActionLabelR4D(action, "Saved") });`,
    `      kgwBridgeR51SaveSettings(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, "Saved"));
      kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-save-settings");
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsActionLabelR4D(action, "Saved") });`
  ], `      kgwBridgeR51SaveSettings(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsFeedbackLabelR7(action));
      kgwBridgeSettingsAfterActionR7(net, action);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsFeedbackLabelR7(action), holdMs: 10000 });`, "Bridge save action R7");

  text = replaceKnown(text, [
    `      kgwBridgeR51RestoreDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, "Restored"));
      kgwBridgeRefreshSettingsDirtyAfterActionR6(net, "after-restore-defaults");
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsActionLabelR4D(action, "Restored") });`,
    `      kgwBridgeR51RestoreDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, "Restored"));
      kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults");
      window.setTimeout(() => kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults+900ms"), 900);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsActionLabelR4D(action, "Restored") });`
  ], `      kgwBridgeR51RestoreDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsFeedbackLabelR7(action));
      kgwBridgeSettingsAfterActionR7(net, action);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsFeedbackLabelR7(action), holdMs: 10000 });`, "Bridge restore action R7");

  text = replaceKnown(text, [
    `      kgwBridgeR51SetAsDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, "Set as defaults"));
      kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-set-defaults");
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsActionLabelR4D(action, "Set as defaults") });`,
    `      kgwBridgeR51SetAsDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, "Set as defaults"));
      kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-set-defaults");
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsActionLabelR4D(action, "Set as defaults") });`
  ], `      kgwBridgeR51SetAsDefaults(net);
      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsFeedbackLabelR7(action));
      kgwBridgeSettingsAfterActionR7(net, action);
      kgwBridgeSettingsTraceR4D(net, action, "action-complete", { label: kgwBridgeSettingsFeedbackLabelR7(action), holdMs: 10000 });`, "Bridge set defaults action R7");

  write(files.bridge, text);

  return {
    changed: before !== text,
    r7Markers: count(text, /KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7_START/g),
    nativeDisabledFalseOwner: count(text, /button\.disabled\s*=\s*false/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    feedbackTenSeconds: count(text, /10000/g),
    initialBaselineCalls: count(text, /kgwBridgeSettingsInstallInitialBaselineR7\(\)/g),
    r7FeedbackLabels: count(text, /kgwBridgeSettingsFeedbackLabelR7/g)
  };
}

function auditAfter() {
  const node = read(files.node);
  const bridge = read(files.bridge);
  const lib = read(files.lib);

  return {
    node: {
      r7Markers: count(node, /KGW_NODE_SETTINGS_SIMPLE_UX_R7_START/g),
      nativeDisabledByEnabled: count(node, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      nativeAlwaysEnabledSettingsOwner: count(node, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*false[\s\S]*?\}/g),
      feedbackTenSeconds: count(node, /kgwNodeFlashSettingsActionButtonR2[\s\S]*?10000/g),
      initialBaselineCalls: count(node, /kgwNodeSettingsInstallInitialBaselineR7\(\)/g),
      r7FeedbackLabels: count(node, /kgwNodeSettingsFeedbackLabelR7/g)
    },
    bridge: {
      r7Markers: count(bridge, /KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7_START/g),
      nativeDisabledByEnabled: count(bridge, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      nativeAlwaysEnabledSettingsOwner: count(bridge, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*false[\s\S]*?\}/g),
      feedbackTenSeconds: count(bridge, /kgwBridgeFlashSettingsActionButtonR2[\s\S]*?10000/g),
      initialBaselineCalls: count(bridge, /kgwBridgeSettingsInstallInitialBaselineR7\(\)/g),
      r7FeedbackLabels: count(bridge, /kgwBridgeSettingsFeedbackLabelR7/g)
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g)
    }
  };
}

saveJson("audit-before-r7.json", {
  nodeLength: read(files.node).length,
  bridgeLength: read(files.bridge).length,
  rustLength: read(files.lib).length
});

const changes = {
  node: patchNode(),
  bridge: patchBridge()
};

saveJson("patch-changes-r7.json", changes);

const after = auditAfter();
saveJson("audit-after-r7.json", after);

const failures = [];

if (after.node.r7Markers !== 1) failures.push("Node R7 marker must exist exactly once.");
if (after.bridge.r7Markers !== 1) failures.push("Bridge R7 marker must exist exactly once.");
if (after.node.nativeDisabledByEnabled < 1) failures.push("Node buttons must use native disabled=!enabled.");
if (after.bridge.nativeDisabledByEnabled < 1) failures.push("Bridge buttons must use native disabled=!enabled.");
if (after.node.nativeAlwaysEnabledSettingsOwner !== 0) failures.push("Node settings owner still forces disabled=false.");
if (after.bridge.nativeAlwaysEnabledSettingsOwner !== 0) failures.push("Bridge settings owner still forces disabled=false.");
if (after.node.feedbackTenSeconds < 1) failures.push("Node feedback timer must be 10000ms.");
if (after.bridge.feedbackTenSeconds < 1) failures.push("Bridge feedback timer must be 10000ms.");
if (after.node.initialBaselineCalls < 1) failures.push("Node initial baseline install missing.");
if (after.bridge.initialBaselineCalls < 1) failures.push("Bridge initial baseline install missing.");
if (after.node.r7FeedbackLabels < 3) failures.push("Node language-aware R7 labels missing.");
if (after.bridge.r7FeedbackLabels < 3) failures.push("Bridge language-aware R7 labels missing.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R7 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R7 patch complete");
console.log(JSON.stringify(after, null, 2));