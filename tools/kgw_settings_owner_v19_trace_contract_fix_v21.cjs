#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];
const backupDir = process.argv[4];

if (!repoRoot || !reportDir || !backupDir) {
  console.error("Usage: node kgw_settings_owner_v19_trace_contract_fix_v21.cjs <repoRoot> <reportDir> <backupDir>");
  process.exit(2);
}

const OWNER = "KGW_SETTINGS_OWNER_V19";
const PATCH = "KGW_SETTINGS_OWNER_V19_TRACE_CONTRACT_FIX_V21";

const files = [
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
  "tools/kgw_global_owner_gate.cjs"
];

const mutableFiles = [
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
];

const report = {
  runName: path.basename(reportDir),
  repoRoot,
  reportDir,
  backupDir,
  owner: OWNER,
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
  if (start < 0) throw new Error("Missing function " + functionName);

  const open = source.indexOf("{", start);
  if (open < 0) throw new Error("Missing opening brace for " + functionName);

  const close = findMatchingBrace(source, open);
  if (close < 0) throw new Error("Missing closing brace for " + functionName);

  return source.slice(0, start) + replacement.trim() + source.slice(close + 1);
}

function traceFunctionSource() {
  return `
  // ${PATCH}
  function trace(root, phase, details) {
    try {
      const safeDetails = details && typeof details === "object" ? details : {};
      const net = String(safeDetails.network || safeDetails.net || "unknown");
      const action = String(safeDetails.action || "settings-owner");
      const detailsText = JSON.stringify({
        owner: OWNER,
        scope: SCOPE,
        phase: phase,
        details: safeDetails
      });

      const args = {
        scope: String(SCOPE),
        net: net,
        action: action,
        phase: String(phase || "unknown"),
        details: detailsText
      };

      const tauri = window.__TAURI__;
      if (tauri && tauri.core && typeof tauri.core.invoke === "function") {
        tauri.core.invoke("kgw_frontend_button_trace_v1", args).catch(function (error) {
          console.error("[KGW_SETTINGS_OWNER_V19_TRACE_FAILED]", error, args);
        });
      } else if (tauri && typeof tauri.invoke === "function") {
        tauri.invoke("kgw_frontend_button_trace_v1", args).catch(function (error) {
          console.error("[KGW_SETTINGS_OWNER_V19_TRACE_FAILED]", error, args);
        });
      } else {
        console.debug("[KGW_SETTINGS_OWNER_V19_TRACE_BROWSER]", args);
      }
    } catch (error) {
      console.error("[KGW_SETTINGS_OWNER_V19_TRACE_EXCEPTION]", error);
    }
  }`;
}

function patchJs(rel) {
  const before = read(rel);
  save("FULL_BEFORE__" + safeName(rel), before);

  if (!before.includes(OWNER)) {
    throw new Error(rel + " does not contain " + OWNER);
  }

  if (!before.includes('tauri.core.invoke("kgw_frontend_button_trace_v1", { payload: payload })')) {
    throw new Error(rel + " does not contain the old wrong payload-based trace invocation. Refusing blind patch.");
  }

  if (!before.includes("function trace(root, phase, details)")) {
    throw new Error(rel + " does not contain V19 trace(root, phase, details).");
  }

  const after = replaceFunction(before, "trace", traceFunctionSource());

  if (after.includes('tauri.core.invoke("kgw_frontend_button_trace_v1", { payload: payload })')) {
    report.validationErrors.push(rel + " still contains old tauri.core payload invocation.");
  }

  if (after.includes('tauri.invoke("kgw_frontend_button_trace_v1", { payload: payload })')) {
    report.validationErrors.push(rel + " still contains old tauri payload invocation.");
  }

  if (!after.includes("KGW_SETTINGS_OWNER_V19_TRACE_CONTRACT_FIX_V21")) {
    report.validationErrors.push(rel + " missing V21 trace contract marker.");
  }

  if (!after.includes("scope: String(SCOPE)") || !after.includes("net: net") || !after.includes("action: action") || !after.includes("phase: String(phase || \"unknown\")") || !after.includes("details: detailsText")) {
    report.validationErrors.push(rel + " missing Rust command argument contract fields.");
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
    "# " + (success ? "SUCCESS" : "FAILED") + " - KGW Settings Owner V19 Trace Contract Fix V21",
    "",
    "- Repository: `" + repoRoot + "`",
    "- Report dir: `" + reportDir + "`",
    "- Backup dir: `" + backupDir + "`",
    "- Owner: `" + OWNER + "`",
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

  for (const rel of files) {
    if (!fs.existsSync(abs(rel))) throw new Error("Missing required file: " + rel);
    const text = read(rel);
    save("FULL_READ__" + safeName(rel), text);
    report.filesRead.push({ rel, bytes: Buffer.byteLength(text, "utf8") });
  }

  for (const rel of mutableFiles) {
    backup(rel);
  }

  patchJs(mutableFiles[0]);
  patchJs(mutableFiles[1]);

  if (report.validationErrors.length) {
    throw new Error("Validation failed before running gates.");
  }

  const nodeCheck = run("node", ["--check", abs(mutableFiles[0])]);
  if (nodeCheck.status !== 0) throw new Error("node --check failed for kaspa-node.js");

  const bridgeCheck = run("node", ["--check", abs(mutableFiles[1])]);
  if (bridgeCheck.status !== 0) throw new Error("node --check failed for kaspa-bridge.js");

  const gate = "tools/kgw_global_owner_gate.cjs";
  const gateCheck = run("node", [abs(gate), repoRoot]);
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

  finish(true, "V19 trace invocation now matches the Rust command contract.");
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
