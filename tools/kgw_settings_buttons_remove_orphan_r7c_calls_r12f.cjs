const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_remove_orphan_r7c_calls_r12f.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
};

const orphanCalls = [
  "kgwNodeSettingsInstallInitialBaselineR7C",
  "kgwBridgeSettingsInstallInitialBaselineR7C"
];

const staleRuntimeMarkers = [
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end",
  "KGW_SETTINGS_BUTTONS_R7C",
  "KGW_SETTINGS_BUTTONS_R8",
  "KGW_SETTINGS_BUTTONS_R9B",
  "KGW_SETTINGS_BUTTONS_R10",
  "KGW_SETTINGS_BUTTONS_R11"
];

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function writeJson(fileName, data) {
  fs.writeFileSync(path.join(reportDir, fileName), JSON.stringify(data, null, 2) + "\n", "utf8");
}

function removeOrphanCallLines(text, callName) {
  const before = text;
  const linePattern = new RegExp("^.*\\b" + callName + "\\s*\\([^\\n;]*\\)\\s*;?\\s*$\\n?", "gm");
  text = text.replace(linePattern, "");

  const inlinePattern = new RegExp("\\s*" + callName + "\\s*\\([^;]*\\)\\s*;?", "g");
  text = text.replace(inlinePattern, "");

  return {
    text,
    removed: before !== text
  };
}

function patchJs(text, label) {
  const before = text;
  const removals = [];

  for (const callName of orphanCalls) {
    const result = removeOrphanCallLines(text, callName);
    text = result.text;
    if (result.removed) {
      removals.push(callName);
    }
  }

  const stillPresent = orphanCalls.filter((callName) => text.includes(callName));

  const errors = [];

  if (stillPresent.length) {
    errors.push(label + ": orphan calls still present: " + stillPresent.join(", "));
  }

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")) {
    errors.push(label + ": R12E owner marker missing");
  }

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E.install(root);")) {
    errors.push(label + ": R12E install call missing");
  }

  if (!/button\.disabled\s*=\s*!!disabled/.test(text)) {
    errors.push(label + ": native button.disabled ownership missing");
  }

  const stale = staleRuntimeMarkers.filter((marker) => text.includes(marker));
  if (stale.length) {
    errors.push(label + ": stale runtime markers still present: " + stale.join(", "));
  }

  return {
    text,
    changed: before !== text,
    removals,
    errors
  };
}

function validateCss(text, label) {
  const errors = [];

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL")) {
    errors.push(label + ": R12E visual marker missing");
  }

  if (!text.includes("kgw-settings-action-disabled-r12e")) {
    errors.push(label + ": R12E disabled visual class missing");
  }

  return errors;
}

function main() {
  const before = {
    nodeJs: read(files.nodeJs),
    bridgeJs: read(files.bridgeJs),
    nodeCss: read(files.nodeCss),
    bridgeCss: read(files.bridgeCss)
  };

  const beforeAudit = {
    nodeOrphans: orphanCalls.filter((name) => before.nodeJs.includes(name)),
    bridgeOrphans: orphanCalls.filter((name) => before.bridgeJs.includes(name)),
    nodeHasR12E: before.nodeJs.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: before.bridgeJs.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    nodeStaleMarkers: staleRuntimeMarkers.filter((marker) => before.nodeJs.includes(marker)),
    bridgeStaleMarkers: staleRuntimeMarkers.filter((marker) => before.bridgeJs.includes(marker))
  };

  const nodePatch = patchJs(before.nodeJs, "Node");
  const bridgePatch = patchJs(before.bridgeJs, "Bridge");

  write(files.nodeJs, nodePatch.text);
  write(files.bridgeJs, bridgePatch.text);

  const afterNode = read(files.nodeJs);
  const afterBridge = read(files.bridgeJs);
  const afterNodeCss = read(files.nodeCss);
  const afterBridgeCss = read(files.bridgeCss);

  const validationErrors = [
    ...nodePatch.errors,
    ...bridgePatch.errors,
    ...validateCss(afterNodeCss, "Node CSS"),
    ...validateCss(afterBridgeCss, "Bridge CSS")
  ];

  const afterAudit = {
    validationErrors,
    nodeChanged: nodePatch.changed,
    bridgeChanged: bridgePatch.changed,
    nodeRemovals: nodePatch.removals,
    bridgeRemovals: bridgePatch.removals,
    nodeOrphansRemaining: orphanCalls.filter((name) => afterNode.includes(name)),
    bridgeOrphansRemaining: orphanCalls.filter((name) => afterBridge.includes(name)),
    nodeHasR12E: afterNode.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: afterBridge.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    nodeNativeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(afterNode),
    bridgeNativeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(afterBridge)
  };

  writeJson("audit-before-r12f.json", beforeAudit);
  writeJson("audit-after-r12f.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R12F validation failed:\n- " + validationErrors.join("\n- "));
  }

  if (!nodePatch.changed && !bridgePatch.changed) {
    throw new Error("R12F found no orphan R7C calls to remove. This means the runtime error may come from cached frontend output or another source path.");
  }

  console.log("# R12F orphan R7C call cleanup passed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
