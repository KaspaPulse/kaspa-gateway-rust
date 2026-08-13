#!/usr/bin/env node
"use strict";

/**
 * KGW Program Unified Gate
 *
 * Single program-wide wrapper gate.
 *
 * Responsibilities:
 * - Run syntax checks for important tools and changed JS owners.
 * - Run global owner gate.
 * - Run i18n contract gate.
 * - Run i18n locale coverage gate.
 * - Run runtime repository binding gate offline.
 * - Run runtime repository binding gate online/latest by default.
 * - Run Bridge/Node mode routing audit with repo/report args.
 * - Run runtime trace owner audit with repo/report args.
 * - Run parallel self-worker runtime gate.
 * - Run raw runtime log provenance gate.
 *
 * This wrapper calls existing gates; it does not duplicate their logic.
 * It does not mutate source.
 */

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const args = process.argv.slice(2);

function hasFlag(name) {
  return args.includes(name);
}

function readOption(name, fallback = "") {
  const index = args.indexOf(name);
  if (index >= 0 && args[index + 1]) return args[index + 1];
  return fallback;
}

function normalizePath(value) {
  return path.resolve(String(value || "."));
}

const repoRoot = normalizePath(readOption("--repo-root", process.cwd()));
const strict = hasFlag("--strict") || true;
const jsonMode = hasFlag("--json");
const offlineOnly = hasFlag("--offline-only");
const skipOnline = hasFlag("--skip-online") || offlineOnly;
const skipTrace = hasFlag("--skip-trace");
const skipRuntime = hasFlag("--skip-runtime");
const skipNodeBridge = hasFlag("--skip-node-bridge");

const stamp = new Date().toISOString().replace(/[:.]/g, "-");
const defaultReportDir = path.join(repoRoot, "reports", `kgw_program_unified_gate_${stamp}`);
const reportDir = normalizePath(readOption("--report-dir", defaultReportDir));

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function writeFile(name, content) {
  fs.writeFileSync(path.join(reportDir, name), content, "utf8");
}

function relPath(absOrRel) {
  return path.relative(repoRoot, path.resolve(repoRoot, absOrRel)).replace(/\\/g, "/");
}

function exists(rel) {
  return fs.existsSync(path.join(repoRoot, rel));
}

ensureDir(reportDir);

const startedAt = new Date().toISOString();
const nodeExe = process.execPath;

const results = [];

function runStep(name, command, stepArgs, options = {}) {
  const required = options.required !== false;
  const cwd = options.cwd || repoRoot;
  const logName = `${name.replace(/[^a-z0-9_.-]+/gi, "_")}.log`;
  const logPath = path.join(reportDir, logName);

  const printable = [command, ...stepArgs].join(" ");
  const started = new Date().toISOString();

  let code = 1;
  let stdout = "";
  let stderr = "";
  let errorMessage = "";

  try {
    const result = cp.spawnSync(command, stepArgs, {
      cwd,
      encoding: "utf8",
      windowsHide: true,
      shell: false,
      env: {
        ...process.env,
        KGW_UNIFIED_GATE: "1"
      }
    });

    code = typeof result.status === "number" ? result.status : (result.error ? 1 : 0);
    stdout = result.stdout || "";
    stderr = result.stderr || "";
    errorMessage = result.error ? String(result.error.message || result.error) : "";
  } catch (error) {
    code = 1;
    errorMessage = error && error.message ? error.message : String(error);
  }

  const ended = new Date().toISOString();

  fs.writeFileSync(
    logPath,
    [
      `# ${name}`,
      `started=${started}`,
      `ended=${ended}`,
      `cwd=${cwd}`,
      `required=${required}`,
      `command=${printable}`,
      `code=${code}`,
      `error=${errorMessage}`,
      "",
      "--- STDOUT ---",
      stdout,
      "",
      "--- STDERR ---",
      stderr
    ].join("\n"),
    "utf8"
  );

  const record = {
    name,
    command,
    args: stepArgs,
    cwd,
    required,
    code,
    ok: code === 0,
    log: logPath,
    error: errorMessage
  };

  results.push(record);
  return record;
}

function nodeCheck(rel) {
  if (!exists(rel)) {
    results.push({
      name: `node_check_missing_${rel}`,
      command: nodeExe,
      args: ["--check", rel],
      cwd: repoRoot,
      required: true,
      code: 1,
      ok: false,
      log: "",
      error: `Missing file: ${rel}`
    });
    return;
  }

  runStep(`node_check_${rel.replace(/[\\/]/g, "_")}`, nodeExe, ["--check", rel]);
}

const syntaxTargets = [
  "tools/kgw_program_unified_gate.cjs",
  "tools/kgw_global_owner_gate.cjs",
  "tools/kgw_i18n_contract_gate.cjs",
  "tools/kgw_i18n_locale_coverage_gate.cjs",
  "tools/kgw_runtime_repository_binding_gate.cjs",
  "tools/kgw_bridge_node_mode_routing_audit_v1.cjs",
  "tools/kgw_runtime_trace_owner_audit_v20.cjs",
  "tools/kgw_parallel_self_worker_runtime_gate.cjs",
  "tools/kgw_raw_log_provenance_gate.cjs",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"
];

for (const target of syntaxTargets) {
  nodeCheck(target);
}

runStep("global_owner_gate_strict", nodeExe, ["tools/kgw_global_owner_gate.cjs", "--strict"]);
runStep("i18n_contract_gate", nodeExe, ["tools/kgw_i18n_contract_gate.cjs"]);
runStep("i18n_locale_coverage_gate", nodeExe, ["tools/kgw_i18n_locale_coverage_gate.cjs"]);

runStep(
  "runtime_repository_binding_gate_offline",
  nodeExe,
  ["tools/kgw_runtime_repository_binding_gate.cjs", "--strict", "--offline", "--json"]
);

if (!skipOnline) {
  runStep(
    "runtime_repository_binding_gate_online_latest",
    nodeExe,
    ["tools/kgw_runtime_repository_binding_gate.cjs", "--strict", "--json"]
  );
}

if (!skipNodeBridge) {
  const bridgeNodeReportDir = path.join(reportDir, "bridge_node_mode_routing_audit");
  ensureDir(bridgeNodeReportDir);
  runStep(
    "bridge_node_mode_routing_audit",
    nodeExe,
    ["tools/kgw_bridge_node_mode_routing_audit_v1.cjs", repoRoot, bridgeNodeReportDir]
  );
}

if (!skipTrace) {
  const traceReportDir = path.join(reportDir, "runtime_trace_owner_audit_v20");
  ensureDir(traceReportDir);
  runStep(
    "runtime_trace_owner_audit_v20",
    nodeExe,
    ["tools/kgw_runtime_trace_owner_audit_v20.cjs", repoRoot, traceReportDir]
  );
}

if (!skipRuntime) {
  runStep(
    "parallel_self_worker_runtime_gate",
    nodeExe,
    ["tools/kgw_parallel_self_worker_runtime_gate.cjs"]
  );
  runStep(
    "raw_log_provenance_gate",
    nodeExe,
    ["tools/kgw_raw_log_provenance_gate.cjs"]
  );
}

runStep("git_status_short", "git", ["status", "--short"], { required: false });
runStep("git_diff_tools", "git", ["diff", "--", "tools"], { required: false });

const failedRequired = results.filter((item) => item.required && item.code !== 0);
const ok = failedRequired.length === 0;

const summary = {
  verdict: ok ? "KGW_PROGRAM_UNIFIED_GATE_PASS" : "KGW_PROGRAM_UNIFIED_GATE_FAIL",
  ok,
  strict,
  repoRoot,
  reportDir,
  startedAt,
  finishedAt: new Date().toISOString(),
  options: {
    offlineOnly,
    skipOnline,
    skipTrace,
    skipRuntime,
    skipNodeBridge,
    jsonMode
  },
  resultCount: results.length,
  failedRequiredCount: failedRequired.length,
  failedRequired: failedRequired.map((item) => ({
    name: item.name,
    code: item.code,
    log: relPath(item.log || ""),
    error: item.error
  })),
  results: results.map((item) => ({
    name: item.name,
    ok: item.ok,
    code: item.code,
    required: item.required,
    log: item.log ? relPath(item.log) : "",
    error: item.error
  }))
};

writeFile("kgw_program_unified_gate.summary.json", JSON.stringify(summary, null, 2));

let md = "";
md += "# KGW Program Unified Gate\n\n";
md += `Verdict: ${summary.verdict}\n\n`;
md += `Repository: \`${repoRoot}\`\n\n`;
md += `Report directory: \`${reportDir}\`\n\n`;
md += "## Results\n\n";

for (const item of summary.results) {
  md += `- ${item.ok ? "PASS" : "FAIL"} ${item.name}`;
  md += ` — code ${item.code}`;
  if (item.log) md += ` — ${item.log}`;
  if (item.error) md += ` — ${item.error}`;
  md += "\n";
}

if (failedRequired.length) {
  md += "\n## Failed required gates\n\n";
  for (const item of summary.failedRequired) {
    md += `- ${item.name}: code ${item.code}, log ${item.log}\n`;
  }
}

writeFile("kgw_program_unified_gate.summary.md", md);

if (!jsonMode) {
  console.log(md);
} else {
  console.log(JSON.stringify(summary, null, 2));
}

process.exit(ok ? 0 : 1);
