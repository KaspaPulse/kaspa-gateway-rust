#!/usr/bin/env node
"use strict";

const assert = require("assert");
const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const nodePath = path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js");
const bridgePath = path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js");
const commandsPath = path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/commands.rs");

const node = fs.readFileSync(nodePath, "utf8");
const bridge = fs.readFileSync(bridgePath, "utf8");
const commands = fs.readFileSync(commandsPath, "utf8");
const activeProductSource = [node, bridge, commands].join("\n");

for (const claim of [
  "startOnLaunch",
  "autoRestart",
  "Auto-start",
  "Start on launch",
  "Auto-reconnect",
  "Startup delay sec",
  "auto_start",
  "auto_restart",
]) {
  assert.ok(
    !activeProductSource.includes(claim),
    `unsupported runtime automation claim remains visible or persisted: ${claim}`,
  );
}

for (const source of [node, bridge]) {
  assert.ok(source.includes("Effective settings apply on next Start"));
  assert.ok(source.includes('restartRequired = running ? "true" : "false"'));
  assert.ok(source.includes("Restart required to apply changed effective settings"));
}

assert.ok(node.includes('data-node-action="start"'));
assert.ok(node.includes('data-node-action="stop"'));
assert.ok(bridge.includes('data-bridge-action="start"'));
assert.ok(bridge.includes('data-bridge-action="stop"'));

console.log("KGW runtime automation claims gate PASSED");
