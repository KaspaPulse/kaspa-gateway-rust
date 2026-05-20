const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"),
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

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function insertBeforeFunction(text, name, block) {
  text = removeBlock(text, "KGW_SETTINGS_OWNER_CONSOLIDATION_R9_START", "KGW_SETTINGS_OWNER_CONSOLIDATION_R9_END");
  const idx = text.indexOf("function " + name);
  if (idx < 0) throw new Error("Insertion function not found: " + name);
  return text.slice(0, idx) + block + text.slice(idx);
}

function nodeR9Block() {
  return `
/* KGW_SETTINGS_OWNER_CONSOLIDATION_R9_START */
let kgwNodeSettingsProgrammaticWriteDepthR9 = 0;

function kgwNodeSettingsIsProgrammaticWriteR9() {
  return kgwNodeSettingsProgrammaticWriteDepthR9 > 0;
}

function kgwNodeSettingsWithProgrammaticWriteR9(callback) {
  kgwNodeSettingsProgrammaticWriteDepthR9 += 1;
  try {
    return callback();
  } finally {
    window.setTimeout(() => {
      kgwNodeSettingsProgrammaticWriteDepthR9 = Math.max(0, kgwNodeSettingsProgrammaticWriteDepthR9 - 1);
    }, 500);
  }
}

function kgwNodeSettingsTraceSuppressedDirtyR9(net, reason) {
  if (typeof kgwNodeSettingsTraceR4D === "function") {
    kgwNodeSettingsTraceR4D(net, "legacy-dirty", "suppressed-by-r9", {
      reason,
      owner: "R7C",
    });
  }
}
/* KGW_SETTINGS_OWNER_CONSOLIDATION_R9_END */

`;
}

function bridgeR9Block() {
  return `
/* KGW_SETTINGS_OWNER_CONSOLIDATION_R9_START */
let kgwBridgeSettingsProgrammaticWriteDepthR9 = 0;

function kgwBridgeSettingsIsProgrammaticWriteR9() {
  return kgwBridgeSettingsProgrammaticWriteDepthR9 > 0;
}

function kgwBridgeSettingsWithProgrammaticWriteR9(callback) {
  kgwBridgeSettingsProgrammaticWriteDepthR9 += 1;
  try {
    return callback();
  } finally {
    window.setTimeout(() => {
      kgwBridgeSettingsProgrammaticWriteDepthR9 = Math.max(0, kgwBridgeSettingsProgrammaticWriteDepthR9 - 1);
    }, 500);
  }
}

function kgwBridgeSettingsTraceSuppressedDirtyR9(net, reason) {
  if (typeof kgwBridgeSettingsTraceR4D === "function") {
    kgwBridgeSettingsTraceR4D(net, "legacy-dirty", "suppressed-by-r9", {
      reason,
      owner: "R7C",
    });
  }
}
/* KGW_SETTINGS_OWNER_CONSOLIDATION_R9_END */

`;
}

function nodeDirtyR4D() {
  return `function kgwNodeUpdateSettingsDirtyButtonsR4D(net, reason = "unknown") {
  kgwNodeSettingsTraceSuppressedDirtyR9(net, reason);
}`;
}

function bridgeDirtyR4D() {
  return `function kgwBridgeUpdateSettingsDirtyButtonsR4D(net, reason = "unknown") {
  kgwBridgeSettingsTraceSuppressedDirtyR9(net, reason);
}`;
}

function nodeSaveOwner() {
  return `function kgwNodeR51SaveSettings(net) {
  kgwNodeR51Store("saved:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Node settings saved successfully.");
}`;
}

function nodeSetDefaultsOwner() {
  return `function kgwNodeR51SetAsDefaults(net) {
  kgwNodeR51Store("default:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Current node settings saved as defaults.");
}`;
}

function nodeRestoreOwner() {
  return `function kgwNodeR51RestoreDefaults(net) {
  kgwNodeSettingsWithProgrammaticWriteR9(() => {
    const defaults = kgwNodeR51Load("default:" + net) || kgwNodeR51Load("factory:" + net);
    kgwNodeR51WriteSettings(net, defaults);
    kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
    appendLog(net, "Node defaults restored successfully.");
  });
}`;
}

function bridgeSaveOwner() {
  return `function kgwBridgeR51SaveSettings(net) {
  kgwBridgeR51Store("saved:" + net, kgwBridgeR51ReadSettings(net));
}`;
}

function bridgeSetDefaultsOwner() {
  return `function kgwBridgeR51SetAsDefaults(net) {
  kgwBridgeR51Store("default:" + net, kgwBridgeR51ReadSettings(net));
}`;
}

function bridgeRestoreOwner() {
  return `function kgwBridgeR51RestoreDefaults(net) {
  kgwBridgeSettingsWithProgrammaticWriteR9(() => {
    const defaults = kgwBridgeR51Load("default:" + net) || kgwBridgeR51Load("factory:" + net);
    kgwBridgeR51WriteSettings(net, defaults);
    kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  });
}`;
}

function patchInstallProgrammaticGuard(text, kind) {
  const b = functionBounds(text, "installActions");
  let body = b.body;

  if (kind === "node") {
    body = body.replace(
      `  root.addEventListener("input", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "input");
  });`,
      `  root.addEventListener("input", (event) => {
    updateAllCommands();
    if (kgwNodeSettingsIsProgrammaticWriteR9()) return;
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "input");
  });`
    );

    body = body.replace(
      `  root.addEventListener("change", (event) => {
    updateAllCommands();
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "change");
  });`,
      `  root.addEventListener("change", (event) => {
    updateAllCommands();
    if (kgwNodeSettingsIsProgrammaticWriteR9()) return;
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "change");
  });`
    );
  } else {
    body = body.replace(
      `  root.addEventListener("input", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "input");
  });`,
      `  root.addEventListener("input", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    if (kgwBridgeSettingsIsProgrammaticWriteR9()) return;
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "input");
  });`
    );

    body = body.replace(
      `  root.addEventListener("change", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "change");
  });`,
      `  root.addEventListener("change", (event) => {
    bridgeSyncAllModeControls();
    updateAllCommands();
    if (kgwBridgeSettingsIsProgrammaticWriteR9()) return;
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "change");
  });`
    );
  }

  return text.slice(0, b.start) + body + text.slice(b.end);
}

function patchAfterActionDelay(text, kind) {
  if (kind === "node") {
    return replaceFunction(text, "kgwNodeSettingsAfterActionR7C", `function kgwNodeSettingsAfterActionR7C(net, action) {
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action + "+9500ms"), 9500);
  window.setTimeout(() => kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action + "+10100ms"), 10100);
}`);
  }

  return replaceFunction(text, "kgwBridgeSettingsAfterActionR7C", `function kgwBridgeSettingsAfterActionR7C(net, action) {
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action + "+9500ms"), 9500);
  window.setTimeout(() => kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action + "+10100ms"), 10100);
}`);
}

function patchJs(file, kind) {
  const before = read(file);
  let text = before;

  if (kind === "node") {
    text = insertBeforeFunction(text, "kgwNodeR51ReadSettings", nodeR9Block());
    text = replaceFunction(text, "kgwNodeUpdateSettingsDirtyButtonsR4D", nodeDirtyR4D());
    text = replaceFunction(text, "kgwNodeR51SaveSettings", nodeSaveOwner());
    text = replaceFunction(text, "kgwNodeR51SetAsDefaults", nodeSetDefaultsOwner());
    text = replaceFunction(text, "kgwNodeR51RestoreDefaults", nodeRestoreOwner());
    text = patchInstallProgrammaticGuard(text, "node");
    text = patchAfterActionDelay(text, "node");
  } else {
    text = insertBeforeFunction(text, "kgwBridgeR51ReadSettings", bridgeR9Block());
    text = replaceFunction(text, "kgwBridgeUpdateSettingsDirtyButtonsR4D", bridgeDirtyR4D());
    text = replaceFunction(text, "kgwBridgeR51SaveSettings", bridgeSaveOwner());
    text = replaceFunction(text, "kgwBridgeR51SetAsDefaults", bridgeSetDefaultsOwner());
    text = replaceFunction(text, "kgwBridgeR51RestoreDefaults", bridgeRestoreOwner());
    text = patchInstallProgrammaticGuard(text, "bridge");
    text = patchAfterActionDelay(text, "bridge");
  }

  write(file, text);

  return {
    changed: before !== text,
    r9Markers: count(text, /KGW_SETTINGS_OWNER_CONSOLIDATION_R9_START/g),
    suppressedDirty: count(text, /suppressed-by-r9/g),
    programmaticGuardRefs: count(text, /IsProgrammaticWriteR9\(\)/g),
    afterActionDelay9500: count(text, /9500/g),
    oldDirtySetters: count(text, /differsFromSaved|differsFromDefault|SetSettingsActionEnabledR2\(net,\s*"save-settings"/g),
  };
}

function auditAfter() {
  const node = read(files.nodeJs);
  const bridge = read(files.bridgeJs);
  const nodeInstall = functionBounds(node, "installActions").body;
  const bridgeInstall = functionBounds(bridge, "installActions").body;
  const lib = read(files.lib);

  return {
    node: {
      r9Markers: count(node, /KGW_SETTINGS_OWNER_CONSOLIDATION_R9_START/g),
      dirtySuppressed: functionBounds(node, "kgwNodeUpdateSettingsDirtyButtonsR4D").body.includes("suppressed-by-r9"),
      saveOwnerNoDirty: !functionBounds(node, "kgwNodeR51SaveSettings").body.includes("UpdateSettingsDirtyButtons"),
      setDefaultsOwnerNoDirty: !functionBounds(node, "kgwNodeR51SetAsDefaults").body.includes("UpdateSettingsDirtyButtons"),
      restoreOwnerProgrammatic: functionBounds(node, "kgwNodeR51RestoreDefaults").body.includes("kgwNodeSettingsWithProgrammaticWriteR9"),
      inputGuard: nodeInstall.includes("kgwNodeSettingsIsProgrammaticWriteR9()"),
      afterActionDelayed: functionBounds(node, "kgwNodeSettingsAfterActionR7C").body.includes("9500"),
      nativeDisabledByEnabled: functionBounds(node, "kgwNodeSetSettingsActionEnabledR2").body.includes("button.disabled = !enabled"),
      feedbackTenSeconds: functionBounds(node, "kgwNodeFlashSettingsActionButtonR2").body.includes("10000"),
    },
    bridge: {
      r9Markers: count(bridge, /KGW_SETTINGS_OWNER_CONSOLIDATION_R9_START/g),
      dirtySuppressed: functionBounds(bridge, "kgwBridgeUpdateSettingsDirtyButtonsR4D").body.includes("suppressed-by-r9"),
      saveOwnerNoDirty: !functionBounds(bridge, "kgwBridgeR51SaveSettings").body.includes("UpdateSettingsDirtyButtons"),
      setDefaultsOwnerNoDirty: !functionBounds(bridge, "kgwBridgeR51SetAsDefaults").body.includes("UpdateSettingsDirtyButtons"),
      restoreOwnerProgrammatic: functionBounds(bridge, "kgwBridgeR51RestoreDefaults").body.includes("kgwBridgeSettingsWithProgrammaticWriteR9"),
      inputGuard: bridgeInstall.includes("kgwBridgeSettingsIsProgrammaticWriteR9()"),
      afterActionDelayed: functionBounds(bridge, "kgwBridgeSettingsAfterActionR7C").body.includes("9500"),
      nativeDisabledByEnabled: functionBounds(bridge, "kgwBridgeSetSettingsActionEnabledR2").body.includes("button.disabled = !enabled"),
      feedbackTenSeconds: functionBounds(bridge, "kgwBridgeFlashSettingsActionButtonR2").body.includes("10000"),
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r9.json", {
  nodeLength: read(files.nodeJs).length,
  bridgeLength: read(files.bridgeJs).length,
});

const changes = {
  node: patchJs(files.nodeJs, "node"),
  bridge: patchJs(files.bridgeJs, "bridge"),
};

saveJson("patch-changes-r9.json", changes);

const after = auditAfter();
saveJson("audit-after-r9.json", after);

const failures = [];

if (after.node.r9Markers !== 1) failures.push("Node R9 marker must exist exactly once.");
if (after.bridge.r9Markers !== 1) failures.push("Bridge R9 marker must exist exactly once.");
if (!after.node.dirtySuppressed) failures.push("Node old dirty updater is not suppressed.");
if (!after.bridge.dirtySuppressed) failures.push("Bridge old dirty updater is not suppressed.");
if (!after.node.saveOwnerNoDirty) failures.push("Node save owner still calls old dirty updater.");
if (!after.bridge.saveOwnerNoDirty) failures.push("Bridge save owner still calls old dirty updater.");
if (!after.node.setDefaultsOwnerNoDirty) failures.push("Node set-defaults owner still calls old dirty updater.");
if (!after.bridge.setDefaultsOwnerNoDirty) failures.push("Bridge set-defaults owner still calls old dirty updater.");
if (!after.node.restoreOwnerProgrammatic) failures.push("Node restore owner does not suppress programmatic events.");
if (!after.bridge.restoreOwnerProgrammatic) failures.push("Bridge restore owner does not suppress programmatic events.");
if (!after.node.inputGuard) failures.push("Node input/change guards missing.");
if (!after.bridge.inputGuard) failures.push("Bridge input/change guards missing.");
if (!after.node.afterActionDelayed) failures.push("Node after-action baseline is not delayed near feedback end.");
if (!after.bridge.afterActionDelayed) failures.push("Bridge after-action baseline is not delayed near feedback end.");
if (!after.node.nativeDisabledByEnabled) failures.push("Node native disabled=!enabled missing.");
if (!after.bridge.nativeDisabledByEnabled) failures.push("Bridge native disabled=!enabled missing.");
if (!after.node.feedbackTenSeconds) failures.push("Node 10s feedback missing.");
if (!after.bridge.feedbackTenSeconds) failures.push("Bridge 10s feedback missing.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R9 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R9 patch complete");
console.log(JSON.stringify(after, null, 2));