const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node r4e.cjs <repoRoot> <reportDir>");
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

function replaceAllExact(text, from, to) {
  return text.split(from).join(to);
}

function insertAfter(text, needle, insertion, label) {
  const idx = text.indexOf(needle);
  if (idx < 0) throw new Error("Missing insertion point: " + label);
  return text.slice(0, idx + needle.length) + insertion + text.slice(idx + needle.length);
}

function ensureRustTrace() {
  const text = read(files.lib);
  return {
    commandCount: count(text, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
    tracePrintCount: count(text, /\[KGW_BUTTON_TRACE\]/g),
    handlerRefs: count(text, /kgw_frontend_button_trace_v1/g)
  };
}

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function nodePreClickBlock() {
  return `
/* KGW_NODE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START */
function kgwNodeInstallSettingsButtonPreClickTraceR4E(root) {
  if (!root || root.__kgwNodeSettingsButtonPreClickTraceR4E) return;
  root.__kgwNodeSettingsButtonPreClickTraceR4E = true;

  const tracePointer = (event) => {
    const button = event.target?.closest?.("[data-node-action]");
    if (!button) return;

    const action = button.dataset.nodeAction;
    const net = button.dataset.net;

    if (action !== "save-settings" && action !== "restore-defaults" && action !== "set-defaults") return;

    kgwNodeSettingsTraceR4D(net, action, "pointerdown", {
      disabled: Boolean(button.disabled),
      ariaDisabled: button.getAttribute("aria-disabled"),
      text: String(button.textContent || "").trim(),
      pointerType: event.pointerType || event.type,
      reason: "pre-click-capture",
    });
  };

  root.addEventListener("pointerdown", tracePointer, true);
  root.addEventListener("mousedown", tracePointer, true);
}
/* KGW_NODE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_END */

`;
}

function bridgePreClickBlock() {
  return `
/* KGW_BRIDGE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START */
function kgwBridgeInstallSettingsButtonPreClickTraceR4E(root) {
  if (!root || root.__kgwBridgeSettingsButtonPreClickTraceR4E) return;
  root.__kgwBridgeSettingsButtonPreClickTraceR4E = true;

  const tracePointer = (event) => {
    const button = event.target?.closest?.("[data-bridge-action]");
    if (!button) return;

    const action = button.dataset.bridgeAction;
    const net = button.dataset.net || button.dataset.network || kgwBridgeCurrentVisibleNetwork(root);

    if (action !== "save-settings" && action !== "restore-defaults" && action !== "set-defaults") return;

    kgwBridgeSettingsTraceR4D(net, action, "pointerdown", {
      disabled: Boolean(button.disabled),
      ariaDisabled: button.getAttribute("aria-disabled"),
      text: String(button.textContent || "").trim(),
      pointerType: event.pointerType || event.type,
      reason: "pre-click-capture",
      instanceId: button.dataset.instanceId,
    });
  };

  root.addEventListener("pointerdown", tracePointer, true);
  root.addEventListener("mousedown", tracePointer, true);
}
/* KGW_BRIDGE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_END */

`;
}

function patchNode() {
  let text = read(files.node);
  const before = text;

  text = removeBlock(text, "KGW_NODE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START", "KGW_NODE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_END");

  if (!text.includes("function kgwNodeSettingsTraceR4D(")) {
    throw new Error("Node R4D trace helper not found. Run R4D first or inspect current file.");
  }

  if (!text.includes("function installActions(root) {")) {
    throw new Error("Node installActions(root) not found.");
  }

  const helperIdx = text.indexOf("function installActions(root) {");
  text = text.slice(0, helperIdx) + nodePreClickBlock() + text.slice(helperIdx);

  const installNeedle = "function installActions(root) {\n";
  if (!text.includes("kgwNodeInstallSettingsButtonPreClickTraceR4E(root);")) {
    text = insertAfter(text, installNeedle, "  kgwNodeInstallSettingsButtonPreClickTraceR4E(root);\n", "Node installActions pre-click call");
  }

  text = replaceAllExact(text, 'kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-save-settings");', 'kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-save-settings");');
  text = replaceAllExact(text, 'kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-set-defaults");', 'kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-set-defaults");');
  text = replaceAllExact(text, 'kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults");', 'kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults");');
  text = replaceAllExact(text, 'window.setTimeout(() => kgwNodeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults+900ms"), 900);', 'window.setTimeout(() => kgwNodeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults+900ms"), 900);');

  text = replaceAllExact(text, 'kgwNodeUpdateAllSettingsDirtyButtonsR4D("legacy-call");', 'kgwNodeUpdateAllSettingsDirtyButtonsR4D("legacy-call");');

  write(files.node, text);

  return {
    changed: before !== text,
    preClickMarkers: count(text, /KGW_NODE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START/g),
    preClickCalls: count(text, /kgwNodeInstallSettingsButtonPreClickTraceR4E\(root\)/g),
    allAfterActionCalls: count(text, /kgwNodeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
    localAfterActionCalls: count(text, /kgwNodeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g),
    pointerdownRefs: count(text, /phase.*pointerdown|pointerdown/g)
  };
}

function patchBridge() {
  let text = read(files.bridge);
  const before = text;

  text = removeBlock(text, "KGW_BRIDGE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START", "KGW_BRIDGE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_END");

  if (!text.includes("function kgwBridgeSettingsTraceR4D(")) {
    throw new Error("Bridge R4D trace helper not found. Run R4D first or inspect current file.");
  }

  if (!text.includes("function installActions(root) {")) {
    throw new Error("Bridge installActions(root) not found.");
  }

  const helperIdx = text.indexOf("function installActions(root) {");
  text = text.slice(0, helperIdx) + bridgePreClickBlock() + text.slice(helperIdx);

  const installNeedle = "function installActions(root) {\n";
  if (!text.includes("kgwBridgeInstallSettingsButtonPreClickTraceR4E(root);")) {
    text = insertAfter(text, installNeedle, "  kgwBridgeInstallSettingsButtonPreClickTraceR4E(root);\n", "Bridge installActions pre-click call");
  }

  text = replaceAllExact(text, 'kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-save-settings");', 'kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-save-settings");');
  text = replaceAllExact(text, 'kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-set-defaults");', 'kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-set-defaults");');
  text = replaceAllExact(text, 'kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults");', 'kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults");');
  text = replaceAllExact(text, 'window.setTimeout(() => kgwBridgeUpdateAllSettingsDirtyButtonsR4D("after-restore-defaults+900ms"), 900);', 'window.setTimeout(() => kgwBridgeUpdateSettingsDirtyButtonsR4D(net, "after-restore-defaults+900ms"), 900);');

  write(files.bridge, text);

  return {
    changed: before !== text,
    preClickMarkers: count(text, /KGW_BRIDGE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START/g),
    preClickCalls: count(text, /kgwBridgeInstallSettingsButtonPreClickTraceR4E\(root\)/g),
    allAfterActionCalls: count(text, /kgwBridgeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
    localAfterActionCalls: count(text, /kgwBridgeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g),
    pointerdownRefs: count(text, /phase.*pointerdown|pointerdown/g)
  };
}

function auditAfter() {
  const node = read(files.node);
  const bridge = read(files.bridge);
  const lib = read(files.lib);

  return {
    node: {
      r4dMarkers: count(node, /KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START/g),
      r4ePreClickMarkers: count(node, /KGW_NODE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START/g),
      r4ePreClickInstallCalls: count(node, /kgwNodeInstallSettingsButtonPreClickTraceR4E\(root\)/g),
      allAfterActionCalls: count(node, /kgwNodeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
      localAfterActionCalls: count(node, /kgwNodeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g),
      hardcodedArabicSettingsFeedbackCalls: count(node, /kgwNodeFlashSettingsActionButtonR2\(button,\s*"تم/g)
    },
    bridge: {
      r4dMarkers: count(bridge, /KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START/g),
      r4ePreClickMarkers: count(bridge, /KGW_BRIDGE_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E_START/g),
      r4ePreClickInstallCalls: count(bridge, /kgwBridgeInstallSettingsButtonPreClickTraceR4E\(root\)/g),
      allAfterActionCalls: count(bridge, /kgwBridgeUpdateAllSettingsDirtyButtonsR4D\("after-/g),
      localAfterActionCalls: count(bridge, /kgwBridgeUpdateSettingsDirtyButtonsR4D\(net,\s*"after-/g),
      hardcodedArabicSettingsFeedbackCalls: count(bridge, /kgwBridgeFlashSettingsActionButtonR2\(button,\s*"تم/g)
    },
    lib: {
      commandCount: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrintCount: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g)
    }
  };
}

saveJson("audit-before-r4e.json", {
  nodeLength: read(files.node).length,
  bridgeLength: read(files.bridge).length,
  rust: ensureRustTrace()
});

const changes = {
  rust: ensureRustTrace(),
  node: patchNode(),
  bridge: patchBridge()
};

saveJson("patch-changes-r4e.json", changes);

const after = auditAfter();
saveJson("audit-after-r4e.json", after);

const failures = [];

if (after.node.r4dMarkers !== 1) failures.push("Node R4D marker must exist exactly once.");
if (after.bridge.r4dMarkers !== 1) failures.push("Bridge R4D marker must exist exactly once.");
if (after.node.r4ePreClickMarkers !== 1) failures.push("Node R4E pre-click marker must exist exactly once.");
if (after.bridge.r4ePreClickMarkers !== 1) failures.push("Bridge R4E pre-click marker must exist exactly once.");
if (after.node.r4ePreClickInstallCalls !== 1) failures.push("Node pre-click installer call must exist exactly once.");
if (after.bridge.r4ePreClickInstallCalls !== 1) failures.push("Bridge pre-click installer call must exist exactly once.");
if (after.node.allAfterActionCalls !== 0) failures.push("Node still updates all networks after a single action.");
if (after.bridge.allAfterActionCalls !== 0) failures.push("Bridge still updates all networks after a single action.");
if (after.node.hardcodedArabicSettingsFeedbackCalls !== 0) failures.push("Node still has hard-coded Arabic feedback calls.");
if (after.bridge.hardcodedArabicSettingsFeedbackCalls !== 0) failures.push("Bridge still has hard-coded Arabic feedback calls.");
if (after.lib.commandCount !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.lib.tracePrintCount !== 1) failures.push("Rust trace println marker must exist exactly once.");
if (after.lib.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R4E validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R4E patch complete");
console.log(JSON.stringify(after, null, 2));