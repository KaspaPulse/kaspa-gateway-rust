const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node r6.cjs <repoRoot> <reportDir>");
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

function replaceRequired(text, from, to, label) {
  if (!text.includes(from)) {
    throw new Error("Missing required target: " + label);
  }
  return text.split(from).join(to);
}

function replaceFunction(text, name, replacement) {
  const re = new RegExp("function\\s+" + name + "\\s*\\([^)]*\\)\\s*\\{");
  const match = text.match(re);
  if (!match) {
    throw new Error("Function not found: " + name);
  }

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
  const needle = "function " + functionName;
  const idx = text.indexOf(needle);
  if (idx < 0) {
    throw new Error("Insertion function not found: " + functionName);
  }
  return text.slice(0, idx) + block + text.slice(idx);
}

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function nodeR6Block() {
  return `
/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START */
function kgwNodeSettingsActionIsR6(action) {
  return action === "save-settings" || action === "restore-defaults" || action === "set-defaults";
}

function kgwNodeNetFromSettingsEventR6(event, fallbackNet = "") {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [data-node-settings-panel], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i]");

  const raw = [
    target?.dataset?.net,
    target?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    target?.id,
    carrier?.id,
    carrier?.className,
    fallbackNet,
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";
  return fallbackNet || "";
}

function kgwNodeUpdateSettingsDirtyFromEventR6(event, reason) {
  const net = kgwNodeNetFromSettingsEventR6(event);
  updateAllCommands();

  if (net) {
    kgwNodeUpdateSettingsDirtyButtonsR4D(net, reason);
    return;
  }

  kgwNodeUpdateAllSettingsDirtyButtonsR4D(reason + "-fallback-all");
}

function kgwNodeRefreshSettingsDirtyAfterActionR6(net, reason) {
  kgwNodeUpdateSettingsDirtyButtonsR4D(net, reason);
  window.setTimeout(() => kgwNodeUpdateSettingsDirtyButtonsR4D(net, reason + "+250ms"), 250);
  window.setTimeout(() => kgwNodeUpdateSettingsDirtyButtonsR4D(net, reason + "+900ms"), 900);
}
/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_END */

`;
}

function bridgeR6Block() {
  return `
/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START */
function kgwBridgeSettingsActionIsR6(action) {
  return action === "save-settings" || action === "restore-defaults" || action === "set-defaults";
}

function kgwBridgeNetFromSettingsEventR6(event, root, fallbackNet = "") {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [data-bridge-settings-panel], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i]");

  const raw = [
    target?.dataset?.net,
    target?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    target?.id,
    carrier?.id,
    carrier?.className,
    fallbackNet,
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";

  try {
    return fallbackNet || kgwBridgeCurrentVisibleNetwork(root) || "";
  } catch {
    return fallbackNet || "";
  }
}

function kgwBridgeUpdateSettingsDirtyFromEventR6(event, root, reason) {
  const net = kgwBridgeNetFromSettingsEventR6(event, root);
  bridgeSyncAllModeControls();
  updateAllCommands();

  if (net) {
    kgwBridgeUpdateSettingsDirtyButtonsR4D(net, reason);
    return;
  }

  kgwBridgeUpdateAllSettingsDirtyButtonsR4D(reason + "-fallback-all");
}

function kgwBridgeRefreshSettingsDirtyAfterActionR6(net, reason) {
  kgwBridgeUpdateSettingsDirtyButtonsR4D(net, reason);
  window.setTimeout(() => kgwBridgeUpdateSettingsDirtyButtonsR4D(net, reason + "+250ms"), 250);
  window.setTimeout(() => kgwBridgeUpdateSettingsDirtyButtonsR4D(net, reason + "+900ms"), 900);
}
/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_END */

`;
}

function patchNode() {
  let text = read(files.node);
  const before = text;

  text = removeBlock(text, "KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START", "KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_END");
  text = insertBeforeFunction(text, "kgwNodeR51ReadSettings", nodeR6Block());

  text = replaceFunction(text, "kgwNodeSetSettingsActionEnabledR2", `function kgwNodeSetSettingsActionEnabledR2(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = false;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);
  });
}`);

  text = replaceFunction(text, "kgwNodeR51SaveSettings", `function kgwNodeR51SaveSettings(net) {
  kgwNodeR51Store("saved:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Node settings saved successfully.");
  kgwNodeUpdateSettingsDirtyButtonsR4D(net, "save-owner");
}`);

  text = replaceFunction(text, "kgwNodeR51SetAsDefaults", `function kgwNodeR51SetAsDefaults(net) {
  kgwNodeR51Store("default:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Current node settings saved as defaults.");
  kgwNodeUpdateSettingsDirtyButtonsR4D(net, "set-defaults-owner");
}`);

  text = replaceFunction(text, "kgwNodeR51RestoreDefaults", `function kgwNodeR51RestoreDefaults(net) {
  const defaults = kgwNodeR51Load("default:" + net) || kgwNodeR51Load("factory:" + net);
  kgwNodeR51WriteSettings(net, defaults);
  kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  appendLog(net, "Node defaults restored successfully.");
  kgwNodeRefreshSettingsDirtyAfterActionR6(net, "restore-defaults-owner");
}`);

  text = replaceRequired(
    text,
    `  function applyLabel() {
    const targets = action
      ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"]\`))
      : [button];`,
    `  function applyLabel() {
    const net = button.dataset.net || "";
    const targets = action && net
      ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`))
      : [button];`,
    "Node feedback apply target scope"
  );

  text = replaceRequired(
    text,
    `  function restoreLabel() {
    const targets = action
      ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"]\`))
      : [button];`,
    `  function restoreLabel() {
    const net = button.dataset.net || "";
    const targets = action && net
      ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`))
      : [button];`,
    "Node feedback restore target scope"
  );

  text = replaceRequired(
    text,
    `function installActions(root) {
  root.addEventListener("input", () => {
    updateAllCommands();
    kgwNodeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });

  root.addEventListener("change", () => {
    updateAllCommands();
    kgwNodeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });`,
    `function installActions(root) {
  root.addEventListener("input", (event) => {
    kgwNodeUpdateSettingsDirtyFromEventR6(event, "input");
  });

  root.addEventListener("change", (event) => {
    kgwNodeUpdateSettingsDirtyFromEventR6(event, "change");
  });`,
    "Node installActions input/change owner"
  );

  text = replaceRequired(
    text,
    `    kgwNodeSettingsTraceR4D(net, action, "click-received", {
      disabled: Boolean(button.disabled),
      ariaDisabled: button.getAttribute("aria-disabled"),
      text: String(button.textContent || "").trim(),
    });

    if (action === "start" || action === "stop") {`,
    `    kgwNodeSettingsTraceR4D(net, action, "click-received", {
      disabled: Boolean(button.disabled),
      ariaDisabled: button.getAttribute("aria-disabled"),
      text: String(button.textContent || "").trim(),
      kgwSettingsActionDisabled: button.dataset.kgwSettingsActionDisabled,
    });

    if (kgwNodeSettingsActionIsR6(action) && button.getAttribute("aria-disabled") === "true") {
      kgwNodeSettingsTraceR4D(net, action, "click-ignored-disabled", {
        disabled: Boolean(button.disabled),
        ariaDisabled: button.getAttribute("aria-disabled"),
        text: String(button.textContent || "").trim(),
      });
      return;
    }

    if (action === "start" || action === "stop") {`,
    "Node disabled settings click guard"
  );

  text = replaceRequired(
    text,
    `      kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-save-settings");`,
    `      kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-save-settings");`,
    "Node save local dirty"
  );

  text = replaceRequired(
    text,
    `      kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults");
      window.setTimeout(() => kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults+900ms"), 900);`,
    `      kgwNodeRefreshSettingsDirtyAfterActionR6(net, "after-restore-defaults");`,
    "Node restore local dirty"
  );

  text = replaceRequired(
    text,
    `      kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-set-defaults");`,
    `      kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-set-defaults");`,
    "Node set-defaults local dirty"
  );

  write(files.node, text);

  return {
    changed: before !== text,
    r6Markers: count(text, /KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START/g),
    updateAllAfterAction: count(text, /kgwNodeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
    flashActionWideSelectors: count(text, /querySelectorAll\(`\[data-node-action="\$\{action\}"\]`\)/g),
    nativeDisabledSettingsOwner: count(text, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
    ariaDisabledGuard: count(text, /click-ignored-disabled/g),
    localAfterAction: count(text, /kgwNodeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g)
  };
}

function patchBridge() {
  let text = read(files.bridge);
  const before = text;

  text = removeBlock(text, "KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START", "KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_END");
  text = insertBeforeFunction(text, "kgwBridgeR51ReadSettings", bridgeR6Block());

  text = replaceFunction(text, "kgwBridgeSetSettingsActionEnabledR2", `function kgwBridgeSetSettingsActionEnabledR2(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = false;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);
  });
}`);

  text = replaceFunction(text, "kgwBridgeR51SaveSettings", `function kgwBridgeR51SaveSettings(net) {
  kgwBridgeR51Store("saved:" + net, kgwBridgeR51ReadSettings(net));
  kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "save-owner");
}`);

  text = replaceFunction(text, "kgwBridgeR51SetAsDefaults", `function kgwBridgeR51SetAsDefaults(net) {
  kgwBridgeR51Store("default:" + net, kgwBridgeR51ReadSettings(net));
  kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "set-defaults-owner");
}`);

  text = replaceFunction(text, "kgwBridgeR51RestoreDefaults", `function kgwBridgeR51RestoreDefaults(net) {
  const defaults = kgwBridgeR51Load("default:" + net) || kgwBridgeR51Load("factory:" + net);
  kgwBridgeR51WriteSettings(net, defaults);
  kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  kgwBridgeRefreshSettingsDirtyAfterActionR6(net, "restore-defaults-owner");
}`);

  text = replaceRequired(
    text,
    `  function applyLabel() {
    const targets = action
      ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"]\`))
      : [button];`,
    `  function applyLabel() {
    const net = button.dataset.net || button.dataset.network || "";
    const targets = action && net
      ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`))
      : [button];`,
    "Bridge feedback apply target scope"
  );

  text = replaceRequired(
    text,
    `  function restoreLabel() {
    const targets = action
      ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"]\`))
      : [button];`,
    `  function restoreLabel() {
    const net = button.dataset.net || button.dataset.network || "";
    const targets = action && net
      ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`))
      : [button];`,
    "Bridge feedback restore target scope"
  );

  text = replaceRequired(
    text,
    `    bridgeSyncAllModeControls();
    updateAllCommands();
    kgwBridgeUpdateAllSettingsDirtyButtonsR4D("legacy-call");
  });`,
    `    kgwBridgeUpdateSettingsDirtyFromEventR6(event, root, "input");
  });`,
    "Bridge input dirty owner"
  );

  text = replaceRequired(
    text,
    `    bridgeSyncAllModeControls();
    updateAllCommands();
    kgwBridgeUpdateAllSettingsDirtyButtonsR4D("change");
  });`,
    `    kgwBridgeUpdateSettingsDirtyFromEventR6(event, root, "change");
  });`,
    "Bridge change dirty owner"
  );

  text = replaceRequired(
    text,
    `    kgwBridgeSettingsTraceR4D(net, action, "click-received", {
      disabled: Boolean(button.disabled),
      ariaDisabled: button.getAttribute("aria-disabled"),
      text: String(button.textContent || "").trim(),
      instanceId,
    });

    if (!net) {`,
    `    kgwBridgeSettingsTraceR4D(net, action, "click-received", {
      disabled: Boolean(button.disabled),
      ariaDisabled: button.getAttribute("aria-disabled"),
      text: String(button.textContent || "").trim(),
      instanceId,
      kgwSettingsActionDisabled: button.dataset.kgwSettingsActionDisabled,
    });

    if (kgwBridgeSettingsActionIsR6(action) && button.getAttribute("aria-disabled") === "true") {
      kgwBridgeSettingsTraceR4D(net, action, "click-ignored-disabled", {
        disabled: Boolean(button.disabled),
        ariaDisabled: button.getAttribute("aria-disabled"),
        text: String(button.textContent || "").trim(),
        instanceId,
      });
      return;
    }

    if (!net) {`,
    "Bridge disabled settings click guard"
  );

  text = replaceRequired(
    text,
    `      kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-save-settings");`,
    `      kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-save-settings");`,
    "Bridge save local dirty"
  );

  text = replaceRequired(
    text,
    `      kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults");
      window.setTimeout(() => kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults+900ms"), 900);`,
    `      kgwBridgeRefreshSettingsDirtyAfterActionR6(net, "after-restore-defaults");`,
    "Bridge restore local dirty"
  );

  text = replaceRequired(
    text,
    `      kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-set-defaults");`,
    `      kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-set-defaults");`,
    "Bridge set-defaults local dirty"
  );

  write(files.bridge, text);

  return {
    changed: before !== text,
    r6Markers: count(text, /KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START/g),
    updateAllAfterAction: count(text, /kgwBridgeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
    flashActionWideSelectors: count(text, /querySelectorAll\(`\[data-bridge-action="\$\{action\}"\]`\)/g),
    nativeDisabledSettingsOwner: count(text, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
    ariaDisabledGuard: count(text, /click-ignored-disabled/g),
    localAfterAction: count(text, /kgwBridgeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g)
  };
}

function auditAfter() {
  const node = read(files.node);
  const bridge = read(files.bridge);
  const lib = read(files.lib);

  return {
    node: {
      r6Markers: count(node, /KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START/g),
      updateAllAfterAction: count(node, /kgwNodeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
      flashActionWideSelectors: count(node, /querySelectorAll\(`\[data-node-action="\$\{action\}"\]`\)/g),
      nativeDisabledSettingsOwner: count(node, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      disabledGuard: count(node, /kgwNodeSettingsActionIsR6\(action\).*?click-ignored-disabled/gs),
      localAfterAction: count(node, /kgwNodeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g),
      saveOwnerLocal: count(node, /function\s+kgwNodeR51SaveSettings[\s\S]*?kgwNodeUpdateSettingsDirtyButtonsR4D\(net,\s*"save-owner"\)/g),
      restoreOwnerLocal: count(node, /function\s+kgwNodeR51RestoreDefaults[\s\S]*?kgwNodeRefreshSettingsDirtyAfterActionR6\(net,\s*"restore-defaults-owner"\)/g),
      setDefaultsOwnerLocal: count(node, /function\s+kgwNodeR51SetAsDefaults[\s\S]*?kgwNodeUpdateSettingsDirtyButtonsR4D\(net,\s*"set-defaults-owner"\)/g)
    },
    bridge: {
      r6Markers: count(bridge, /KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START/g),
      updateAllAfterAction: count(bridge, /kgwBridgeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
      flashActionWideSelectors: count(bridge, /querySelectorAll\(`\[data-bridge-action="\$\{action\}"\]`\)/g),
      nativeDisabledSettingsOwner: count(bridge, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      disabledGuard: count(bridge, /kgwBridgeSettingsActionIsR6\(action\).*?click-ignored-disabled/gs),
      localAfterAction: count(bridge, /kgwBridgeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g),
      saveOwnerLocal: count(bridge, /function\s+kgwBridgeR51SaveSettings[\s\S]*?kgwBridgeUpdateSettingsDirtyButtonsR4D\(net,\s*"save-owner"\)/g),
      restoreOwnerLocal: count(bridge, /function\s+kgwBridgeR51RestoreDefaults[\s\S]*?kgwBridgeRefreshSettingsDirtyAfterActionR6\(net,\s*"restore-defaults-owner"\)/g),
      setDefaultsOwnerLocal: count(bridge, /function\s+kgwBridgeR51SetAsDefaults[\s\S]*?kgwBridgeUpdateSettingsDirtyButtonsR4D\(net,\s*"set-defaults-owner"\)/g)
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g)
    }
  };
}

saveJson("audit-before-r6.json", {
  nodeLength: read(files.node).length,
  bridgeLength: read(files.bridge).length,
  rustLength: read(files.lib).length
});

const changes = {
  node: patchNode(),
  bridge: patchBridge()
};

saveJson("patch-changes-r6.json", changes);

const after = auditAfter();
saveJson("audit-after-r6.json", after);

const failures = [];

if (after.node.r6Markers !== 1) failures.push("Node R6 marker must exist exactly once.");
if (after.bridge.r6Markers !== 1) failures.push("Bridge R6 marker must exist exactly once.");
if (after.node.updateAllAfterAction !== 0) failures.push("Node still has update-all after a single action.");
if (after.bridge.updateAllAfterAction !== 0) failures.push("Bridge still has update-all after a single action.");
if (after.node.flashActionWideSelectors !== 0) failures.push("Node feedback still targets all buttons of same action.");
if (after.bridge.flashActionWideSelectors !== 0) failures.push("Bridge feedback still targets all buttons of same action.");
if (after.node.nativeDisabledSettingsOwner !== 0) failures.push("Node settings action owner still uses native disabled=!enabled.");
if (after.bridge.nativeDisabledSettingsOwner !== 0) failures.push("Bridge settings action owner still uses native disabled=!enabled.");
if (after.node.disabledGuard < 1) failures.push("Node disabled click guard missing.");
if (after.bridge.disabledGuard < 1) failures.push("Bridge disabled click guard missing.");
if (after.node.saveOwnerLocal < 1) failures.push("Node save owner is not local.");
if (after.bridge.saveOwnerLocal < 1) failures.push("Bridge save owner is not local.");
if (after.node.restoreOwnerLocal < 1) failures.push("Node restore owner is not local.");
if (after.bridge.restoreOwnerLocal < 1) failures.push("Bridge restore owner is not local.");
if (after.node.setDefaultsOwnerLocal < 1) failures.push("Node set-defaults owner is not local.");
if (after.bridge.setDefaultsOwnerLocal < 1) failures.push("Bridge set-defaults owner is not local.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R6 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R6 patch complete");
console.log(JSON.stringify(after, null, 2));