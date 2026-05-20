const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_remove_orphan_dirty_owners_r12g.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
};

const orphanNames = [
  "kgwNodeSettingsInstallInitialBaselineR7C",
  "kgwBridgeSettingsInstallInitialBaselineR7C",
  "kgwNodeSettingsTraceSuppressedDirtyR9B",
  "kgwBridgeSettingsTraceSuppressedDirtyR9B",
  "kgwNodeUpdateSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateSettingsDirtyButtonsR4D",
  "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateAllSettingsDirtyButtonsR4D"
];

const staleMarkers = [
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end",
  "KGW_SETTINGS_BUTTONS_R4D",
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

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function findMatchingBrace(text, openIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;

  for (let i = openIndex; i < text.length; i++) {
    const ch = text[i];
    const next = text[i + 1];

    if (lineComment) {
      if (ch === "\n") lineComment = false;
      continue;
    }

    if (blockComment) {
      if (ch === "*" && next === "/") {
        blockComment = false;
        i++;
      }
      continue;
    }

    if (quote) {
      if (escaped) {
        escaped = false;
        continue;
      }

      if (ch === "\\") {
        escaped = true;
        continue;
      }

      if (ch === quote) {
        quote = null;
      }

      continue;
    }

    if (ch === "/" && next === "/") {
      lineComment = true;
      i++;
      continue;
    }

    if (ch === "/" && next === "*") {
      blockComment = true;
      i++;
      continue;
    }

    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      continue;
    }

    if (ch === "{") depth++;
    if (ch === "}") depth--;

    if (depth === 0) return i;
  }

  return -1;
}

function removeNamedFunction(text, name) {
  const before = text;
  const re = new RegExp("function\\s+" + escapeRegExp(name) + "\\s*\\([^)]*\\)\\s*\\{", "g");

  let changed = false;
  let match;

  while ((match = re.exec(text)) !== null) {
    const start = match.index;
    const open = text.indexOf("{", start);
    const close = findMatchingBrace(text, open);

    if (close < 0) {
      continue;
    }

    let end = close + 1;
    const tail = text.slice(end, end + 10);
    const semi = tail.match(/^\s*;?/);
    if (semi) end += semi[0].length;

    text = text.slice(0, start) + "\n" + text.slice(end);
    changed = true;
    re.lastIndex = 0;
  }

  return { text, changed: changed || before !== text };
}

function removeCallStatements(text, name) {
  const before = text;

  const fullLine = new RegExp("^.*\\b" + escapeRegExp(name) + "\\s*\\([^\\n;]*\\)\\s*;?\\s*$\\n?", "gm");
  text = text.replace(fullLine, "");

  const inline = new RegExp("\\s*\\b" + escapeRegExp(name) + "\\s*\\([^;]*\\)\\s*;?", "g");
  text = text.replace(inline, "");

  return { text, changed: before !== text };
}

function removeStaleMarkerLines(text) {
  let changed = false;

  for (const marker of staleMarkers) {
    const pattern = new RegExp("^.*" + escapeRegExp(marker) + ".*$\\n?", "gm");
    const before = text;
    text = text.replace(pattern, "");
    if (before !== text) changed = true;
  }

  return { text, changed };
}

function patchJs(text, label) {
  const before = text;
  const removedFunctions = [];
  const removedCalls = [];

  for (const name of orphanNames) {
    const fnResult = removeNamedFunction(text, name);
    text = fnResult.text;
    if (fnResult.changed) removedFunctions.push(name);

    const callResult = removeCallStatements(text, name);
    text = callResult.text;
    if (callResult.changed) removedCalls.push(name);
  }

  const markerResult = removeStaleMarkerLines(text);
  text = markerResult.text;

  const errors = [];

  const remainingOrphans = orphanNames.filter((name) => text.includes(name));
  if (remainingOrphans.length) {
    errors.push(label + ": orphan names still present: " + remainingOrphans.join(", "));
  }

  const remainingMarkers = staleMarkers.filter((marker) => text.includes(marker));
  if (remainingMarkers.length) {
    errors.push(label + ": stale markers still present: " + remainingMarkers.join(", "));
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

  return {
    text,
    changed: before !== text,
    removedFunctions,
    removedCalls,
    removedMarkerLines: markerResult.changed,
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
    nodeOrphans: orphanNames.filter((name) => before.nodeJs.includes(name)),
    bridgeOrphans: orphanNames.filter((name) => before.bridgeJs.includes(name)),
    nodeStaleMarkers: staleMarkers.filter((marker) => before.nodeJs.includes(marker)),
    bridgeStaleMarkers: staleMarkers.filter((marker) => before.bridgeJs.includes(marker)),
    nodeHasR12E: before.nodeJs.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: before.bridgeJs.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")
  };

  const nodePatch = patchJs(before.nodeJs, "Node");
  const bridgePatch = patchJs(before.bridgeJs, "Bridge");

  write(files.nodeJs, nodePatch.text);
  write(files.bridgeJs, bridgePatch.text);

  const after = {
    nodeJs: read(files.nodeJs),
    bridgeJs: read(files.bridgeJs),
    nodeCss: read(files.nodeCss),
    bridgeCss: read(files.bridgeCss)
  };

  const validationErrors = [
    ...nodePatch.errors,
    ...bridgePatch.errors,
    ...validateCss(after.nodeCss, "Node CSS"),
    ...validateCss(after.bridgeCss, "Bridge CSS")
  ];

  const afterAudit = {
    validationErrors,
    nodeChanged: nodePatch.changed,
    bridgeChanged: bridgePatch.changed,
    nodeRemovedFunctions: nodePatch.removedFunctions,
    bridgeRemovedFunctions: bridgePatch.removedFunctions,
    nodeRemovedCalls: nodePatch.removedCalls,
    bridgeRemovedCalls: bridgePatch.removedCalls,
    nodeOrphansRemaining: orphanNames.filter((name) => after.nodeJs.includes(name)),
    bridgeOrphansRemaining: orphanNames.filter((name) => after.bridgeJs.includes(name)),
    nodeStaleMarkersRemaining: staleMarkers.filter((marker) => after.nodeJs.includes(marker)),
    bridgeStaleMarkersRemaining: staleMarkers.filter((marker) => after.bridgeJs.includes(marker)),
    nodeHasR12E: after.nodeJs.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: after.bridgeJs.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    nodeNativeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(after.nodeJs),
    bridgeNativeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(after.bridgeJs)
  };

  writeJson("audit-before-r12g.json", beforeAudit);
  writeJson("audit-after-r12g.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R12G validation failed:\n- " + validationErrors.join("\n- "));
  }

  if (!nodePatch.changed && !bridgePatch.changed) {
    throw new Error("R12G found no stale dirty owner fragments to remove. Runtime may be using cached frontend or a different source path.");
  }

  console.log("# R12G stale dirty-owner cleanup passed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
