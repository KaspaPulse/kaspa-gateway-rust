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

function patchNode(text) {
  text = replaceFunction(text, "kgwNodeSettingsInstallInitialBaselineR7C", `function kgwNodeSettingsInstallInitialBaselineR7C() {
  if (typeof NODE_NETWORKS === "undefined") return;
  if (window.__kgwNodeSettingsInitialBaselineR10Installed) return;
  window.__kgwNodeSettingsInitialBaselineR10Installed = true;

  kgwNodeSettingsProgrammaticWriteDepthR9B += 1;

  const apply = (reason) => {
    NODE_NETWORKS.forEach((net) => kgwNodeSettingsAcceptBaselineR7C(net.key, reason));
  };

  apply("initial-load-r10");
  window.setTimeout(() => apply("initial-load-r10+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load-r10+2500ms"), 2500);
  window.setTimeout(() => {
    kgwNodeSettingsProgrammaticWriteDepthR9B = Math.max(0, kgwNodeSettingsProgrammaticWriteDepthR9B - 1);
  }, 3000);
}`);

  text = replaceFunction(text, "kgwNodeSettingsAfterActionR7C", `function kgwNodeSettingsAfterActionR7C(net, action) {
  kgwNodeSettingsWithProgrammaticWriteR9B(() => {
    kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action + "-immediate-r10");
  });

  window.setTimeout(() => {
    kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action + "+1000ms-r10");
  }, 1000);
}`);

  return text;
}

function patchBridge(text) {
  text = replaceFunction(text, "kgwBridgeSettingsInstallInitialBaselineR7C", `function kgwBridgeSettingsInstallInitialBaselineR7C() {
  if (typeof BRIDGE_NETWORKS === "undefined") return;
  if (window.__kgwBridgeSettingsInitialBaselineR10Installed) return;
  window.__kgwBridgeSettingsInitialBaselineR10Installed = true;

  kgwBridgeSettingsProgrammaticWriteDepthR9B += 1;

  const apply = (reason) => {
    BRIDGE_NETWORKS.forEach((net) => kgwBridgeSettingsAcceptBaselineR7C(net.key, reason));
  };

  apply("initial-load-r10");
  window.setTimeout(() => apply("initial-load-r10+1000ms"), 1000);
  window.setTimeout(() => apply("initial-load-r10+2500ms"), 2500);
  window.setTimeout(() => {
    kgwBridgeSettingsProgrammaticWriteDepthR9B = Math.max(0, kgwBridgeSettingsProgrammaticWriteDepthR9B - 1);
  }, 3000);
}`);

  text = replaceFunction(text, "kgwBridgeSettingsAfterActionR7C", `function kgwBridgeSettingsAfterActionR7C(net, action) {
  kgwBridgeSettingsWithProgrammaticWriteR9B(() => {
    kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action + "-immediate-r10");
  });

  window.setTimeout(() => {
    kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action + "+1000ms-r10");
  }, 1000);
}`);

  return text;
}

function auditAfter() {
  const node = read(files.nodeJs);
  const bridge = read(files.bridgeJs);
  const lib = read(files.lib);

  const nodeInitial = functionBounds(node, "kgwNodeSettingsInstallInitialBaselineR7C").body;
  const bridgeInitial = functionBounds(bridge, "kgwBridgeSettingsInstallInitialBaselineR7C").body;
  const nodeAfter = functionBounds(node, "kgwNodeSettingsAfterActionR7C").body;
  const bridgeAfter = functionBounds(bridge, "kgwBridgeSettingsAfterActionR7C").body;

  return {
    node: {
      initialUsesR10Guard: nodeInitial.includes("__kgwNodeSettingsInitialBaselineR10Installed"),
      initialSuppressesProgrammaticEvents: nodeInitial.includes("kgwNodeSettingsProgrammaticWriteDepthR9B += 1"),
      afterActionImmediate: nodeAfter.includes("after-\" + action + \"-immediate-r10"),
      afterActionNo9500: !nodeAfter.includes("9500"),
      afterActionNo10100: !nodeAfter.includes("10100"),
      nativeDisabledByEnabled: functionBounds(node, "kgwNodeSetSettingsActionEnabledR2").body.includes("button.disabled = !enabled"),
      feedbackTenSeconds: functionBounds(node, "kgwNodeFlashSettingsActionButtonR2").body.includes("10000"),
      oldDirtySuppressed: functionBounds(node, "kgwNodeUpdateSettingsDirtyButtonsR4D").body.includes("kgwNodeSettingsTraceSuppressedDirtyR9B"),
      inputGuard: functionBounds(node, "installActions").body.includes("kgwNodeSettingsIsProgrammaticWriteR9B()"),
    },
    bridge: {
      initialUsesR10Guard: bridgeInitial.includes("__kgwBridgeSettingsInitialBaselineR10Installed"),
      initialSuppressesProgrammaticEvents: bridgeInitial.includes("kgwBridgeSettingsProgrammaticWriteDepthR9B += 1"),
      afterActionImmediate: bridgeAfter.includes("after-\" + action + \"-immediate-r10"),
      afterActionNo9500: !bridgeAfter.includes("9500"),
      afterActionNo10100: !bridgeAfter.includes("10100"),
      nativeDisabledByEnabled: functionBounds(bridge, "kgwBridgeSetSettingsActionEnabledR2").body.includes("button.disabled = !enabled"),
      feedbackTenSeconds: functionBounds(bridge, "kgwBridgeFlashSettingsActionButtonR2").body.includes("10000"),
      oldDirtySuppressed: functionBounds(bridge, "kgwBridgeUpdateSettingsDirtyButtonsR4D").body.includes("kgwBridgeSettingsTraceSuppressedDirtyR9B"),
      inputGuard: functionBounds(bridge, "installActions").body.includes("kgwBridgeSettingsIsProgrammaticWriteR9B()"),
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r10.json", {
  nodeLength: read(files.nodeJs).length,
  bridgeLength: read(files.bridgeJs).length,
});

const beforeNode = read(files.nodeJs);
const beforeBridge = read(files.bridgeJs);

const afterNode = patchNode(beforeNode);
const afterBridge = patchBridge(beforeBridge);

write(files.nodeJs, afterNode);
write(files.bridgeJs, afterBridge);

saveJson("patch-changes-r10.json", {
  nodeChanged: beforeNode !== afterNode,
  bridgeChanged: beforeBridge !== afterBridge,
});

const after = auditAfter();
saveJson("audit-after-r10.json", after);

const failures = [];

for (const [scope, data] of Object.entries({ node: after.node, bridge: after.bridge })) {
  if (!data.initialUsesR10Guard) failures.push(scope + " initial baseline guard missing.");
  if (!data.initialSuppressesProgrammaticEvents) failures.push(scope + " initial baseline does not suppress programmatic events.");
  if (!data.afterActionImmediate) failures.push(scope + " after-action does not disable immediately.");
  if (!data.afterActionNo9500) failures.push(scope + " still has 9500ms after-action delayed owner.");
  if (!data.afterActionNo10100) failures.push(scope + " still has 10100ms after-action delayed owner.");
  if (!data.nativeDisabledByEnabled) failures.push(scope + " native disabled=!enabled missing.");
  if (!data.feedbackTenSeconds) failures.push(scope + " 10s feedback missing.");
  if (!data.oldDirtySuppressed) failures.push(scope + " old dirty owner not suppressed.");
  if (!data.inputGuard) failures.push(scope + " input/change guard missing.");
}

if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R10 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R10 patch complete");
console.log(JSON.stringify(after, null, 2));