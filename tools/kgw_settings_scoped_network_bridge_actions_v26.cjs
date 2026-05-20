#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];
const backupDir = process.argv[4];

if (!repoRoot || !reportDir || !backupDir) {
  console.error("Usage: node kgw_settings_scoped_network_bridge_actions_v26.cjs <repoRoot> <reportDir> <backupDir>");
  process.exit(2);
}

const PATCH = "KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26";

const files = {
  nodeJs: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  bridgeJs: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  nodeCss: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
  bridgeCss: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css",
  gate: "tools/kgw_global_owner_gate.cjs",
  libRs: "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"
};

const mutableFiles = [files.nodeJs, files.bridgeJs];
const readOnlyFiles = [files.nodeCss, files.bridgeCss, files.gate, files.libRs];

const report = {
  runName: path.basename(reportDir),
  repoRoot,
  reportDir,
  backupDir,
  patch: PATCH,
  startedAt: new Date().toISOString(),
  changedFiles: [],
  filesRead: [],
  backups: [],
  checks: [],
  validationErrors: [],
  rollback: false,
  gitCommit: false,
  gitPush: false
};

function mkdirp(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function abs(rel) {
  return path.join(repoRoot, rel);
}

function safeName(rel) {
  return rel.replace(/[\\/:\*\?"<>\|]/g, "__");
}

function read(rel) {
  return fs.readFileSync(abs(rel), "utf8");
}

function write(rel, text) {
  fs.writeFileSync(abs(rel), text, "utf8");
}

function save(name, text) {
  const out = path.join(reportDir, name);
  mkdirp(path.dirname(out));
  fs.writeFileSync(out, text, "utf8");
  return out;
}

function backup(rel) {
  const out = path.join(backupDir, safeName(rel));
  mkdirp(path.dirname(out));
  fs.copyFileSync(abs(rel), out);
  report.backups.push({ rel, out });
}

function run(command, args, cwd) {
  const result = cp.spawnSync(command, args, {
    cwd: cwd || repoRoot,
    shell: false,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 80
  });

  const item = {
    command: [command].concat(args || []).join(" "),
    cwd: cwd || repoRoot,
    status: result.status,
    signal: result.signal || null,
    stdout: result.stdout || "",
    stderr: result.stderr || "",
    error: result.error ? String(result.error.message || result.error) : null
  };

  report.checks.push(item);

  save(
    "CHECK__" + report.checks.length + "__" + command.replace(/[^\w.-]+/g, "_") + ".log",
    "COMMAND: " + item.command +
      "\nCWD: " + item.cwd +
      "\nSTATUS: " + item.status +
      "\nSIGNAL: " + item.signal +
      "\nERROR: " + item.error +
      "\n\n--- STDOUT ---\n" + item.stdout +
      "\n\n--- STDERR ---\n" + item.stderr
  );

  return item;
}

function count(text, needle) {
  let n = 0;
  let offset = 0;
  while (true) {
    const i = text.indexOf(needle, offset);
    if (i < 0) break;
    n += 1;
    offset = i + needle.length;
  }
  return n;
}

function findMatchingBrace(source, openBraceIndex) {
  let depth = 0;
  let mode = "code";
  let escaped = false;

  for (let i = openBraceIndex; i < source.length; i += 1) {
    const ch = source[i];
    const next = source[i + 1];

    if (mode === "lineComment") {
      if (ch === "\n") mode = "code";
      continue;
    }

    if (mode === "blockComment") {
      if (ch === "*" && next === "/") {
        i += 1;
        mode = "code";
      }
      continue;
    }

    if (mode === "single") {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === "'") mode = "code";
      continue;
    }

    if (mode === "double") {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === "\"") mode = "code";
      continue;
    }

    if (mode === "template") {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === "`") mode = "code";
      continue;
    }

    if (ch === "/" && next === "/") {
      i += 1;
      mode = "lineComment";
      continue;
    }

    if (ch === "/" && next === "*") {
      i += 1;
      mode = "blockComment";
      continue;
    }

    if (ch === "'") {
      mode = "single";
      continue;
    }

    if (ch === "\"") {
      mode = "double";
      continue;
    }

    if (ch === "`") {
      mode = "template";
      continue;
    }

    if (ch === "{") depth += 1;

    if (ch === "}") {
      depth -= 1;
      if (depth === 0) return i;
    }
  }

  return -1;
}

function replaceFunction(source, functionName, replacement) {
  const needle = "function " + functionName;
  const start = source.indexOf(needle);
  if (start < 0) throw new Error("Missing function: " + functionName);

  const open = source.indexOf("{", start);
  if (open < 0) throw new Error("Missing opening brace for: " + functionName);

  const close = findMatchingBrace(source, open);
  if (close < 0) throw new Error("Missing closing brace for: " + functionName);

  let end = close + 1;
  while (end < source.length && /\s/.test(source[end])) end += 1;

  return source.slice(0, start) + replacement.trim() + "\n\n" + source.slice(end);
}

function nodeInstallActionsSource() {
  return `
function installActions(root) {
  // ${PATCH}: Node settings actions are scoped to the exact network that changed.
  if (window.KGW_NODE_SETTINGS_OWNER_V19 && typeof window.KGW_NODE_SETTINGS_OWNER_V19.install === "function") {
    window.KGW_NODE_SETTINGS_OWNER_V19.install(root);
  }

  function normalizeNet(value) {
    const raw = String(value || "").toLowerCase();
    if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
    if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
    if (raw.includes("mainnet")) return "mainnet";
    return "";
  }

  function netFromElement(element) {
    if (!element) return "";
    const carrier = element.closest("[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel], [data-node-section-panel]");

    return normalizeNet(
      [
        element.dataset && element.dataset.net,
        element.dataset && element.dataset.network,
        carrier && carrier.dataset && carrier.dataset.net,
        carrier && carrier.dataset && carrier.dataset.network,
        carrier && carrier.dataset && carrier.dataset.nodeNetworkPanel,
        element.id,
        carrier && carrier.id,
        carrier && carrier.className
      ].filter(Boolean).join(" ")
    );
  }

  function netFromEvent(event) {
    return netFromElement(event && event.target);
  }

  function scopedUpdate(net, reason) {
    if (!net) return;
    if (typeof updateCommand === "function") {
      updateCommand(net);
    }
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
      window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", {
        scope: "node",
        net: net,
        action: "settings-scope",
        phase: "v26-scoped-update",
        details: JSON.stringify({ patch: "${PATCH}", network: net, reason: reason || "unknown" })
      }).catch(function () {});
    }
  }

  root.addEventListener("input", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-change" : "programmatic-change");
  }, true);

  root.addEventListener("click", (event) => {
    const button = event.target && event.target.closest ? event.target.closest("[data-node-action]") : null;
    if (!button || !root.contains(button)) return;

    const action = button.dataset.nodeAction;
    const net = normalizeNet(button.dataset.net || button.dataset.network || netFromElement(button));

    if (!net) return;

    if (action === "save-settings") {
      if (typeof kgwNodeR51SaveSettings === "function") kgwNodeR51SaveSettings(net);
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      if (typeof kgwNodeR51SetAsDefaults === "function") kgwNodeR51SetAsDefaults(net);
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      if (typeof kgwNodeR51RestoreDefaults === "function") kgwNodeR51RestoreDefaults(net);
      scopedUpdate(net, "restore-defaults");
      return;
    }

    if (action === "copy-command") {
      const preview = document.getElementById("node-" + net + "-commandPreview");
      if (preview && navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(String(preview.value || preview.textContent || "")).catch(function () {});
      }
      return;
    }

    if (action === "start" || action === "stop") {
      if (typeof runNodeIntegratedAction === "function") {
        runNodeIntegratedAction(action, net).catch(function (error) {
          if (typeof appendLog === "function") appendLog(net, "Node " + action + " failed: " + (error && error.message ? error.message : String(error)));
        });
      }
    }
  }, false);
}`;
}

function bridgeInstallActionsSource() {
  return `
function installActions(root) {
  // ${PATCH}: Bridge settings actions are scoped to the exact bridge/network that changed.
  if (window.KGW_BRIDGE_SETTINGS_OWNER_V19 && typeof window.KGW_BRIDGE_SETTINGS_OWNER_V19.install === "function") {
    window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root);
  }

  function normalizeNet(value) {
    const raw = String(value || "").toLowerCase();
    if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
    if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
    if (raw.includes("mainnet")) return "mainnet";
    return "";
  }

  function netFromElement(element) {
    if (!element) return "";
    const carrier = element.closest("[data-net], [data-network], [data-bridge-network-panel], [data-bridge-inner-panel], [data-bridge-section-panel], [data-bridge-instance-panel]");

    return normalizeNet(
      [
        element.dataset && element.dataset.net,
        element.dataset && element.dataset.network,
        carrier && carrier.dataset && carrier.dataset.net,
        carrier && carrier.dataset && carrier.dataset.network,
        carrier && carrier.dataset && carrier.dataset.bridgeNetworkPanel,
        element.id,
        carrier && carrier.id,
        carrier && carrier.className
      ].filter(Boolean).join(" ")
    );
  }

  function netFromEvent(event) {
    return netFromElement(event && event.target);
  }

  function scopedUpdate(net, reason) {
    if (!net) return;

    if (typeof bridgeSyncModeControls === "function") {
      bridgeSyncModeControls(net);
    }

    if (typeof updateCommand === "function") {
      updateCommand(net);
    }

    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
      window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", {
        scope: "bridge",
        net: net,
        action: "settings-scope",
        phase: "v26-scoped-update",
        details: JSON.stringify({ patch: "${PATCH}", network: net, reason: reason || "unknown" })
      }).catch(function () {});
    }
  }

  root.addEventListener("input", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-change" : "programmatic-change");
  }, true);

  root.addEventListener("click", (event) => {
    const button = event.target && event.target.closest ? event.target.closest("[data-bridge-action]") : null;
    if (!button || !root.contains(button)) return;

    const action = button.dataset.bridgeAction;
    const net = normalizeNet(button.dataset.net || button.dataset.network || netFromElement(button));

    if (!net) return;

    if (action === "save-settings") {
      if (typeof kgwBridgeR51SaveSettings === "function") kgwBridgeR51SaveSettings(net);
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      if (typeof kgwBridgeR51SetAsDefaults === "function") kgwBridgeR51SetAsDefaults(net);
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      if (typeof kgwBridgeR51RestoreDefaults === "function") kgwBridgeR51RestoreDefaults(net);
      scopedUpdate(net, "restore-defaults");
      return;
    }

    if (action === "copy-command") {
      const preview = document.getElementById("bridge-" + net + "-commandPreview");
      if (preview && navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(String(preview.value || preview.textContent || "")).catch(function () {});
      }
      return;
    }

    if (action === "start" || action === "stop") {
      if (typeof runBridgeIntegratedAction === "function") {
        runBridgeIntegratedAction(action, net).catch(function (error) {
          if (typeof appendLog === "function") appendLog(net, "Bridge " + action + " failed: " + (error && error.message ? error.message : String(error)));
        });
      }
    }
  }, false);
}`;
}

function patchJs(rel, sourceFactory, ownerKind) {
  const before = read(rel);
  save("FULL_BEFORE__" + safeName(rel), before);

  if (!before.includes("function installActions(root)")) {
    throw new Error(rel + " missing installActions(root).");
  }

  if (!before.includes("KGW_SETTINGS_OWNER_V19")) {
    throw new Error(rel + " missing V19 settings owner.");
  }

  const beforeInstallStart = before.indexOf("function installActions(root)");
  const beforeInstallOpen = before.indexOf("{", beforeInstallStart);
  const beforeInstallClose = findMatchingBrace(before, beforeInstallOpen);
  const beforeInstallBlock = before.slice(beforeInstallStart, beforeInstallClose + 1);
  save("INSTALL_ACTIONS_BEFORE__" + safeName(rel), beforeInstallBlock);

  const after = replaceFunction(before, "installActions", sourceFactory());
  const afterInstallStart = after.indexOf("function installActions(root)");
  const afterInstallOpen = after.indexOf("{", afterInstallStart);
  const afterInstallClose = findMatchingBrace(after, afterInstallOpen);
  const afterInstallBlock = after.slice(afterInstallStart, afterInstallClose + 1);
  save("INSTALL_ACTIONS_AFTER__" + safeName(rel), afterInstallBlock);

  if (!afterInstallBlock.includes(PATCH)) {
    report.validationErrors.push(rel + " installActions missing V26 marker.");
  }

  if (afterInstallBlock.includes("updateAllCommands()")) {
    report.validationErrors.push(rel + " installActions still calls updateAllCommands().");
  }

  if (afterInstallBlock.includes("bridgeSyncAllModeControls()")) {
    report.validationErrors.push(rel + " installActions still calls bridgeSyncAllModeControls().");
  }

  if (ownerKind === "node") {
    if (!afterInstallBlock.includes("kgwNodeR51SaveSettings(net)")) report.validationErrors.push(rel + " missing scoped node save.");
    if (!afterInstallBlock.includes("kgwNodeR51SetAsDefaults(net)")) report.validationErrors.push(rel + " missing scoped node set-defaults.");
    if (!afterInstallBlock.includes("kgwNodeR51RestoreDefaults(net)")) report.validationErrors.push(rel + " missing scoped node restore.");
    if (!afterInstallBlock.includes("updateCommand(net)")) report.validationErrors.push(rel + " missing scoped node updateCommand(net).");
  }

  if (ownerKind === "bridge") {
    if (!afterInstallBlock.includes("kgwBridgeR51SaveSettings(net)")) report.validationErrors.push(rel + " missing scoped bridge save.");
    if (!afterInstallBlock.includes("kgwBridgeR51SetAsDefaults(net)")) report.validationErrors.push(rel + " missing scoped bridge set-defaults.");
    if (!afterInstallBlock.includes("kgwBridgeR51RestoreDefaults(net)")) report.validationErrors.push(rel + " missing scoped bridge restore.");
    if (!afterInstallBlock.includes("bridgeSyncModeControls(net)")) report.validationErrors.push(rel + " missing scoped bridgeSyncModeControls(net).");
    if (!afterInstallBlock.includes("updateCommand(net)")) report.validationErrors.push(rel + " missing scoped bridge updateCommand(net).");
  }

  write(rel, after);
  save("FULL_AFTER__" + safeName(rel), after);
  report.changedFiles.push(rel);
}

function rollback() {
  report.rollback = true;
  for (const item of report.backups) {
    fs.copyFileSync(item.out, abs(item.rel));
  }
}

function finish(success, reason) {
  report.finishedAt = new Date().toISOString();
  report.success = success;
  report.reason = reason || "";
  save("REPORT.json", JSON.stringify(report, null, 2));

  const md = [
    "# " + (success ? "SUCCESS" : "FAILED") + " - KGW Settings Scoped Network Bridge Actions V26",
    "",
    "- Repository: `" + repoRoot + "`",
    "- Report dir: `" + reportDir + "`",
    "- Backup dir: `" + backupDir + "`",
    "- Patch: `" + PATCH + "`",
    "- Rollback: `" + report.rollback + "`",
    "- Git commit: `false`",
    "- Git push: `false`",
    "- Reason: " + (reason || ""),
    "",
    "## Changed files",
    "",
    report.changedFiles.map((x) => "- `" + x + "`").join("\n") || "- none",
    "",
    "## Validation errors",
    "",
    report.validationErrors.map((x) => "- " + x).join("\n") || "- none",
    "",
    "## Checks",
    "",
    report.checks.map((x) => "- `" + x.command + "` => `" + x.status + "`").join("\n") || "- none",
    ""
  ].join("\n");

  save(success ? "REPORT_SUCCESS.md" : "REPORT_FAILED.md", md);
}

try {
  mkdirp(reportDir);
  mkdirp(backupDir);

  for (const rel of mutableFiles.concat(readOnlyFiles)) {
    if (!fs.existsSync(abs(rel))) {
      throw new Error("Missing required file: " + rel);
    }

    const text = read(rel);
    save("FULL_READ__" + safeName(rel), text);
    report.filesRead.push({ rel, bytes: Buffer.byteLength(text, "utf8") });
  }

  for (const rel of mutableFiles) {
    backup(rel);
  }

  patchJs(files.nodeJs, nodeInstallActionsSource, "node");
  patchJs(files.bridgeJs, bridgeInstallActionsSource, "bridge");

  if (report.validationErrors.length) {
    throw new Error("Validation failed before gates.");
  }

  const nodeCheck = run("node", ["--check", abs(files.nodeJs)]);
  if (nodeCheck.status !== 0) throw new Error("node --check failed for kaspa-node.js");

  const bridgeCheck = run("node", ["--check", abs(files.bridgeJs)]);
  if (bridgeCheck.status !== 0) throw new Error("node --check failed for kaspa-bridge.js");

  const gateCheck = run("node", [abs(files.gate), repoRoot]);
  if (gateCheck.status !== 0) throw new Error("kgw_global_owner_gate.cjs failed");

  const i18nContract = "tools/kgw_i18n_contract_gate.cjs";
  if (fs.existsSync(abs(i18nContract))) {
    const check = run("node", [abs(i18nContract), repoRoot]);
    if (check.status !== 0) throw new Error("kgw_i18n_contract_gate.cjs failed");
  }

  const i18nCoverage = "tools/kgw_i18n_locale_coverage_gate.cjs";
  if (fs.existsSync(abs(i18nCoverage))) {
    const check = run("node", [abs(i18nCoverage), repoRoot]);
    if (check.status !== 0) throw new Error("kgw_i18n_locale_coverage_gate.cjs failed");
  }

  const fmt = run("cargo", ["fmt", "--all"]);
  if (fmt.status !== 0) throw new Error("cargo fmt --all failed");

  const fmtCheck = run("cargo", ["fmt", "--all", "--", "--check"]);
  if (fmtCheck.status !== 0) throw new Error("cargo fmt --all -- --check failed");

  const cargoCheck = run("cargo", ["check", "-p", "kaspa-gateway-desktop", "--no-default-features", "--features", "official-kaspa-runtime-all rkstratum_cpu_miner"]);
  if (cargoCheck.status !== 0) throw new Error("cargo check failed");

  finish(true, "Node and Bridge installActions are now scoped per exact network/bridge. No all-network settings propagation remains inside installActions.");
  console.log("");
  console.log("# SUCCESS");
  console.log("Report: " + path.join(reportDir, "REPORT_SUCCESS.md"));
  console.log("Backup: " + backupDir);
  process.exit(0);
} catch (error) {
  try {
    rollback();
  } catch (rollbackError) {
    report.validationErrors.push("Rollback failed: " + rollbackError.message);
  }

  finish(false, error && error.message ? error.message : String(error));

  console.error("");
  console.error("# FAILED");
  console.error("Reason: " + (error && error.message ? error.message : String(error)));
  console.error("Rollback: " + report.rollback);
  console.error("Report: " + path.join(reportDir, "REPORT_FAILED.md"));
  console.error("Backup: " + backupDir);
  process.exit(1);
}
