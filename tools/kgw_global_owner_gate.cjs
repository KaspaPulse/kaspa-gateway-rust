#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2] || process.cwd();
const owner = "KGW_SETTINGS_OWNER_V19";

const files = [
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"
];

const forbidden = [
  "KGW_SETTINGS_OWNER_V18",
  "KGW_SETTINGS_OWNER_V17",
  "KGW_SETTINGS_OWNER_V16",
  "KGW_SETTINGS_OWNER_FINAL_V15",
  "KGW_SETTINGS_SINGLE_OWNER_R14",
  "KGW_SETTINGS_CANONICAL_OWNER_R13",
  "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
  "KGW_SETTINGS_SELECTION_TRACE_R12",
  "KGW_SETTINGS_UNIFIED_OWNER_R12",
  "KGW_NODE_SETTINGS_SIMPLE_UX_R7C",
  "KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C",
  "KGW_SETTINGS_OWNER_CONSOLIDATION_R9B",
  "KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8",
  "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D",
  "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D",
  "settings-buttons",
  "nativeDisabledExpected",
  "click-received",
  "click-ignored-disabled",
  "action-start",
  "auto-baseline-before-input",
  "kgwNodeSettingsChangedR7C",
  "kgwBridgeSettingsChangedR7C",
  "kgwNodeSettingsAfterActionR7C",
  "kgwBridgeSettingsAfterActionR7C",
  "kgwNodeSettingsIsFeedbackLockedR11",
  "kgwBridgeSettingsIsFeedbackLockedR11"
];

const errors = [];

function countText(text, needle) {
  let count = 0;
  let offset = 0;
  while (true) {
    const index = text.indexOf(needle, offset);
    if (index < 0) break;
    count += 1;
    offset = index + needle.length;
  }
  return count;
}

for (const rel of files) {
  const full = path.join(repoRoot, rel);
  if (!fs.existsSync(full)) {
    errors.push("Missing file: " + rel);
    continue;
  }
  const text = fs.readFileSync(full, "utf8");
  for (const token of forbidden) {
    if (text.includes(token)) errors.push(rel + " contains forbidden old owner token: " + token);
  }
}

const nodeJs = fs.readFileSync(path.join(repoRoot, files[0]), "utf8");
const bridgeJs = fs.readFileSync(path.join(repoRoot, files[1]), "utf8");
const nodeCss = fs.readFileSync(path.join(repoRoot, files[2]), "utf8");
const bridgeCss = fs.readFileSync(path.join(repoRoot, files[3]), "utf8");

if (countText(nodeJs, owner) < 2) errors.push("Node JS owner V19 marker missing or incomplete.");
if (countText(bridgeJs, owner) < 2) errors.push("Bridge JS owner V19 marker missing or incomplete.");
if (!nodeCss.includes("KGW_SETTINGS_OWNER_V19_VISUAL")) errors.push("Node CSS V19 visual marker missing.");
if (!bridgeCss.includes("KGW_SETTINGS_OWNER_V19_VISUAL")) errors.push("Bridge CSS V19 visual marker missing.");
if (!nodeJs.includes("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)")) errors.push("Node installActions is not routed to the single V19 owner.");
if (!bridgeJs.includes("window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)")) errors.push("Bridge installActions is not routed to the single V19 owner.");

if (errors.length) {
  console.error("KGW global owner gate failed:");
  for (const error of errors) console.error("- " + error);
  process.exit(1);
}

console.log("KGW global owner gate passed: single settings owner V19 only.");
