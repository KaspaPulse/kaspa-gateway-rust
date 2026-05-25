#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  console.error("Usage: node kgw_runtime_trace_owner_audit_v20.cjs <repoRoot> <reportDir>");
  process.exit(2);
}

const report = {
  runName: path.basename(reportDir),
  repoRoot,
  reportDir,
  startedAt: new Date().toISOString(),
  mutation: false,
  gitCommit: false,
  gitPush: false,
  filesRead: [],
  findings: [],
  errors: [],
  checks: []
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

function save(name, text) {
  const out = path.join(reportDir, name);
  mkdirp(path.dirname(out));
  fs.writeFileSync(out, text, "utf8");
  return out;
}

function exists(rel) {
  return fs.existsSync(abs(rel));
}

function read(rel) {
  const full = abs(rel);
  if (!fs.existsSync(full)) {
    report.errors.push("Missing file: " + rel);
    return "";
  }
  const text = fs.readFileSync(full, "utf8");
  const out = save("FULL_READ__" + safeName(rel), text);
  report.filesRead.push({ rel, out, bytes: Buffer.byteLength(text, "utf8") });
  return text;
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

function lineHits(text, needle) {
  const lines = text.split(/\r?\n/);
  const hits = [];
  for (let i = 0; i < lines.length; i += 1) {
    if (lines[i].includes(needle)) {
      hits.push({ line: i + 1, text: lines[i] });
    }
  }
  return hits;
}

function addFinding(level, title, data) {
  report.findings.push({ level, title, data: data || {} });
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

function finish(success, reason) {
  report.finishedAt = new Date().toISOString();
  report.success = success;
  report.reason = reason || "";
  save("REPORT.json", JSON.stringify(report, null, 2));

  const md = [
    "# " + (success ? "SUCCESS" : "FAILED") + " - KGW Runtime Trace Owner Audit V20",
    "",
    "- Repository: `" + repoRoot + "`",
    "- Report dir: `" + reportDir + "`",
    "- Source mutation: `false`",
    "- Git commit: `false`",
    "- Git push: `false`",
    "- Reason: " + (reason || ""),
    "",
    "## Findings",
    "",
    report.findings.map((f) => {
      return "- **" + f.level + "** " + f.title + "\n  ```json\n  " + JSON.stringify(f.data || {}, null, 2).replace(/\n/g, "\n  ") + "\n  ```";
    }).join("\n") || "- none",
    "",
    "## Errors",
    "",
    report.errors.map((x) => "- " + x).join("\n") || "- none",
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

  const rels = [
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css",
    "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
    "apps/kaspa-gateway-desktop/src-tauri/src/main.rs",
    "apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json",
    "tools/kgw_global_owner_gate.cjs",
    "package.json",
    "apps/kaspa-gateway-desktop/package.json"
  ];

  const texts = {};
  for (const rel of rels) {
    if (exists(rel)) texts[rel] = read(rel);
  }

  const nodeJsRel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
  const bridgeJsRel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
  const libRsRel = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
  const mainRsRel = "apps/kaspa-gateway-desktop/src-tauri/src/main.rs";
  const gateRel = "tools/kgw_global_owner_gate.cjs";

  const nodeJs = texts[nodeJsRel] || "";
  const bridgeJs = texts[bridgeJsRel] || "";
  const libRs = texts[libRsRel] || "";
  const mainRs = texts[mainRsRel] || "";
  const gate = texts[gateRel] || "";

  addFinding("INFO", "V19 frontend owner markers", {
    nodeOwnerCount: count(nodeJs, "KGW_SETTINGS_OWNER_V19"),
    bridgeOwnerCount: count(bridgeJs, "KGW_SETTINGS_OWNER_V19"),
    nodeInstallRoute: nodeJs.includes("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)"),
    bridgeInstallRoute: bridgeJs.includes("window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)"),
    nodeTraceInvokeCount: count(nodeJs, "kgw_frontend_button_trace_v1"),
    bridgeTraceInvokeCount: count(bridgeJs, "kgw_frontend_button_trace_v1")
  });

  const oldRuntimeTokens = [
    "action=settings-buttons",
    "nativeDisabledExpected",
    "click-received",
    "click-ignored-disabled",
    "action-start",
    "auto-baseline-before-input",
    "KGW_SETTINGS_OWNER_V18",
    "KGW_SETTINGS_OWNER_V17",
    "KGW_SETTINGS_OWNER_V16",
    "KGW_SETTINGS_OWNER_FINAL_V15"
  ];

  const oldHits = {};
  for (const token of oldRuntimeTokens) {
    const n = count(nodeJs, token) + count(bridgeJs, token);
    if (n > 0) oldHits[token] = n;
  }

  addFinding(Object.keys(oldHits).length ? "ERROR" : "OK", "Old owner/runtime tokens in Node/Bridge JS", oldHits);

  const rustCombined = libRs + "\n" + mainRs;

  addFinding("INFO", "Rust trace command presence", {
    commandNameCount: count(rustCombined, "kgw_frontend_button_trace_v1"),
    tauriCommandAttrNearName: rustCombined.includes("#[tauri::command]") && rustCombined.includes("kgw_frontend_button_trace_v1"),
    invokeHandlerMentionsCommand: rustCombined.includes("generate_handler") && rustCombined.includes("kgw_frontend_button_trace_v1"),
    printlnNearTraceCommand: rustCombined.includes("println!") && rustCombined.includes("kgw_frontend_button_trace_v1"),
    commandHitsInLib: lineHits(libRs, "kgw_frontend_button_trace_v1").slice(0, 20),
    commandHitsInMain: lineHits(mainRs, "kgw_frontend_button_trace_v1").slice(0, 20)
  });

  addFinding("INFO", "Global owner gate file", {
    exists: !!gate,
    hasV19: gate.includes("KGW_SETTINGS_OWNER_V19"),
    hasOldRegexRisk: gate.includes("new RegExp") || gate.includes(".match(") || gate.includes(".replace("),
    lineCount: gate ? gate.split(/\r?\n/).length : 0
  });

  const nodeCheck = run("node", ["--check", abs(nodeJsRel)], repoRoot);
  const bridgeCheck = run("node", ["--check", abs(bridgeJsRel)], repoRoot);

  if (exists(gateRel)) {
    run("node", [abs(gateRel), repoRoot], repoRoot);
  }

  run("cargo", ["check", "-p", "kaspa-gateway-desktop", "--no-default-features", "--features", "official-kaspa-runtime-all rkstratum_cpu_miner"], repoRoot);

  /* KGW_RUNTIME_TRACE_OWNER_OPTIONAL_DEV_PROBE_FIX_R100A3
   * The program-wide unified gate must be deterministic and must not open
   * a long-running Tauri dev process by default. The previous hard-coded
   * npm.cmd spawn could fail with spawn EINVAL on Windows gate runners.
   *
   * Dev probe is now opt-in only:
   *   KGW_TRACE_AUDIT_DEV_PROBE=1
   */
  const devProbeEnabled = String(process.env.KGW_TRACE_AUDIT_DEV_PROBE || "").trim() === "1";

  if (!devProbeEnabled) {
    addFinding("INFO", "Tauri dev 12-second probe", {
      skipped: true,
      reason: "Disabled by default for stable unified gate execution.",
      enableWith: "KGW_TRACE_AUDIT_DEV_PROBE=1",
      patch: "R100A3"
    });
  } else {
    const appRoot = abs("apps/kaspa-gateway-desktop");
    const npmCommand =
      process.env.KGW_NPM_CMD ||
      (process.platform === "win32" && fs.existsSync("C:\\Program Files\\nodejs\\npm.cmd")
        ? "C:\\Program Files\\nodejs\\npm.cmd"
        : (process.platform === "win32" ? "npm.cmd" : "npm"));

    const devProbe = cp.spawn(
      npmCommand,
      ["run", "tauri", "--", "dev", "--features", "official-kaspa-runtime-all rkstratum_cpu_miner"],
      {
        cwd: appRoot,
        shell: process.platform === "win32",
        windowsHide: true
      }
    );

    let stdout = "";
    let stderr = "";
    let exited = false;
    let exitCode = null;
    let exitSignal = null;
    let spawnError = null;

    devProbe.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });

    devProbe.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    devProbe.on("error", (error) => {
      spawnError = error && error.message ? error.message : String(error);
      exited = true;
    });

    devProbe.on("exit", (code, signal) => {
      exited = true;
      exitCode = code;
      exitSignal = signal;
    });

    const start = Date.now();
    while (Date.now() - start < 12000) {
      Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 250);
      if (exited) break;
    }

    if (!exited) {
      try {
        devProbe.kill("SIGTERM");
      } catch (_) {}
    }

    save(
      "DEV_PROBE_12_SECONDS.log",
      "Enabled: true" +
        "\nCommand: " + npmCommand +
        "\nExitedWithin12Seconds: " + exited +
        "\nExitCode: " + exitCode +
        "\nExitSignal: " + exitSignal +
        "\nSpawnError: " + (spawnError || "") +
        "\n\n--- STDOUT ---\n" + stdout +
        "\n\n--- STDERR ---\n" + stderr
    );

    addFinding(exited && !spawnError ? "ERROR" : "INFO", "Tauri dev 12-second probe", {
      enabled: true,
      command: npmCommand,
      exitedWithin12Seconds: exited,
      exitCode,
      exitSignal,
      spawnError,
      stdoutTail: stdout.slice(-4000),
      stderrTail: stderr.slice(-4000),
      interpretation: spawnError
        ? "The optional dev probe could not spawn; this no longer blocks the unified gate unless explicitly reviewed."
        : (exited
          ? "The app/dev process returned to PowerShell quickly. Frontend runtime trace cannot appear if the app exits before UI interaction."
          : "The dev process stayed alive for 12 seconds. Lack of trace is more likely command registration/printing/frontend loading, not immediate exit.")
    });
  }

  const critical = [];

  if (count(nodeJs, "KGW_SETTINGS_OWNER_V19") < 2) critical.push("Node V19 owner marker missing/incomplete.");
  if (count(bridgeJs, "KGW_SETTINGS_OWNER_V19") < 2) critical.push("Bridge V19 owner marker missing/incomplete.");
  if (!nodeJs.includes("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)")) critical.push("Node installActions not routed to V19 owner.");
  if (!bridgeJs.includes("window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)")) critical.push("Bridge installActions not routed to V19 owner.");
  if (Object.keys(oldHits).length) critical.push("Old owner/runtime tokens still exist in frontend JS.");
  if (!rustCombined.includes("kgw_frontend_button_trace_v1")) critical.push("Rust command kgw_frontend_button_trace_v1 is missing from lib.rs/main.rs.");
  if (nodeCheck.status !== 0) critical.push("node --check failed for node JS.");
  if (bridgeCheck.status !== 0) critical.push("node --check failed for bridge JS.");

  if (critical.length) {
    for (const item of critical) report.errors.push(item);
    finish(false, "Critical audit blockers found.");
    process.exit(1);
  }

  finish(true, "Audit completed. Review findings for exact runtime trace cause.");
  process.exit(0);
} catch (error) {
  report.errors.push(error && error.message ? error.message : String(error));
  finish(false, error && error.message ? error.message : String(error));
  process.exit(1);
}
