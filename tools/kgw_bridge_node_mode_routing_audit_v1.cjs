const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  console.error("Usage: node kgw_bridge_node_mode_routing_audit_v1.cjs <repoRoot> <reportDir>");
  process.exit(2);
}

const files = {
  bridgeJs: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  nodeJs: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  integratedRuntime: "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
  tauriLib: "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
  rkBridge: "crates/kaspa-gateway-rk-bridge/src/lib.rs",
};

function ensureDir(p) {
  fs.mkdirSync(p, { recursive: true });
}

function readRel(rel) {
  const abs = path.join(repoRoot, rel);
  if (!fs.existsSync(abs)) {
    return { rel, abs, exists: false, text: "" };
  }
  return { rel, abs, exists: true, text: fs.readFileSync(abs, "utf8") };
}

function writeText(rel, text) {
  const out = path.join(reportDir, rel);
  ensureDir(path.dirname(out));
  fs.writeFileSync(out, text, "utf8");
}

function lineNumberAt(text, index) {
  return text.slice(0, index).split(/\r?\n/).length;
}

function findAll(text, pattern) {
  const results = [];
  const re = pattern instanceof RegExp ? new RegExp(pattern.source, pattern.flags.includes("g") ? pattern.flags : pattern.flags + "g") : new RegExp(pattern, "g");
  let m;
  while ((m = re.exec(text)) !== null) {
    results.push({
      index: m.index,
      line: lineNumberAt(text, m.index),
      match: m[0],
      groups: m.slice(1),
    });
    if (m.index === re.lastIndex) re.lastIndex++;
  }
  return results;
}

function extractAround(text, needleOrRegex, beforeLines = 45, afterLines = 65) {
  let index = -1;
  if (needleOrRegex instanceof RegExp) {
    const m = text.match(needleOrRegex);
    if (m) index = m.index ?? -1;
  } else {
    index = text.indexOf(needleOrRegex);
  }
  if (index < 0) return null;

  const lines = text.split(/\r?\n/);
  const targetLine = lineNumberAt(text, index);
  const start = Math.max(1, targetLine - beforeLines);
  const end = Math.min(lines.length, targetLine + afterLines);
  const numbered = [];
  for (let i = start; i <= end; i++) {
    numbered.push(String(i).padStart(6, " ") + " | " + lines[i - 1]);
  }
  return {
    targetLine,
    start,
    end,
    text: numbered.join("\n"),
  };
}

function extractFunctionByName(text, name) {
  const patterns = [
    new RegExp(`function\\s+${name}\\s*\\(`),
    new RegExp(`const\\s+${name}\\s*=\\s*\\(`),
    new RegExp(`let\\s+${name}\\s*=\\s*\\(`),
    new RegExp(`async\\s+function\\s+${name}\\s*\\(`),
    new RegExp(`fn\\s+${name}\\s*\\(`),
    new RegExp(`pub\\s+fn\\s+${name}\\s*\\(`),
    new RegExp(`async\\s+fn\\s+${name}\\s*\\(`),
    new RegExp(`pub\\s+async\\s+fn\\s+${name}\\s*\\(`),
  ];

  let start = -1;
  for (const p of patterns) {
    const m = text.match(p);
    if (m) {
      start = m.index ?? -1;
      break;
    }
  }
  if (start < 0) return null;

  let brace = text.indexOf("{", start);
  if (brace < 0) return null;

  let depth = 0;
  let inString = false;
  let stringQuote = "";
  let escape = false;

  for (let i = brace; i < text.length; i++) {
    const ch = text[i];

    if (inString) {
      if (escape) {
        escape = false;
      } else if (ch === "\\") {
        escape = true;
      } else if (ch === stringQuote) {
        inString = false;
      }
      continue;
    }

    if (ch === '"' || ch === "'" || ch === "`") {
      inString = true;
      stringQuote = ch;
      continue;
    }

    if (ch === "{") depth++;
    if (ch === "}") {
      depth--;
      if (depth === 0) {
        const end = i + 1;
        const startLine = lineNumberAt(text, start);
        const endLine = lineNumberAt(text, end);
        return {
          name,
          startLine,
          endLine,
          text: text.slice(start, end),
        };
      }
    }
  }

  return null;
}

function command(cmd, args, cwd) {
  try {
    const result = cp.spawnSync(cmd, args, {
      cwd,
      shell: false,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 12,
    });
    return {
      ok: result.status === 0,
      status: result.status,
      stdout: result.stdout || "",
      stderr: result.stderr || "",
      error: result.error ? String(result.error.stack || result.error.message || result.error) : "",
    };
  } catch (err) {
    return {
      ok: false,
      status: null,
      stdout: "",
      stderr: "",
      error: String(err.stack || err.message || err),
    };
  }
}

ensureDir(reportDir);
ensureDir(path.join(reportDir, "full-files"));
ensureDir(path.join(reportDir, "extracts"));

const loaded = {};
for (const [key, rel] of Object.entries(files)) {
  loaded[key] = readRel(rel);
  if (loaded[key].exists) {
    writeText(path.join("full-files", rel.replace(/[\\/]/g, "__")), loaded[key].text);
  }
}

const findings = [];
const blockers = [];
const warnings = [];
const evidence = {};

function addFinding(severity, area, message, data = {}) {
  findings.push({ severity, area, message, ...data });
}

function requireFile(key) {
  if (!loaded[key].exists) {
    blockers.push(`Missing required file: ${loaded[key].rel}`);
    return false;
  }
  return true;
}

for (const key of Object.keys(files)) {
  requireFile(key);
}

if (blockers.length === 0) {
  const bridgeJs = loaded.bridgeJs.text;
  const tauriLib = loaded.tauriLib.text;
  const integratedRuntime = loaded.integratedRuntime.text;
  const rkBridge = loaded.rkBridge.text;

  const frontendNeedles = [
    "buildCommandLines",
    "buildApplyPayload",
    "nodeMode",
    "--node-mode",
    "runtimeRole",
    "nodeKind",
    "bridgeKind",
    "official-external-node",
    "official-inprocess-node",
    "invoke(",
  ];

  evidence.bridgeJsNeedles = {};
  for (const n of frontendNeedles) {
    evidence.bridgeJsNeedles[n] = findAll(bridgeJs, n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).slice(0, 50);
  }

  const rustNeedles = [
    "kgw_apply_command_preview_overrides",
    "kgw_worker_start",
    "try_run_kgw_self_worker_from_args",
    "kgw_run_bridge_self_worker",
    "BridgeRuntimeSettings",
    "BridgeRuntimeMode::OfficialExternalNode",
    "BridgeRuntimeMode::OfficialInProcessNode",
    "StartOfficialExternalNode",
    "StartOfficialInProcessNode",
    "--node-mode",
    "node-mode",
    "node_mode",
    "kaspad",
    "rpclisten",
    "utxoindex",
  ];

  evidence.tauriLibNeedles = {};
  for (const n of rustNeedles) {
    evidence.tauriLibNeedles[n] = findAll(tauriLib, n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).slice(0, 80);
  }

  evidence.integratedRuntimeNeedles = {};
  for (const n of rustNeedles) {
    evidence.integratedRuntimeNeedles[n] = findAll(integratedRuntime, n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).slice(0, 80);
  }

  evidence.rkBridgeNeedles = {};
  for (const n of rustNeedles) {
    evidence.rkBridgeNeedles[n] = findAll(rkBridge, n.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).slice(0, 80);
  }

  const extracts = [
    ["bridgeJs_buildCommandLines", loaded.bridgeJs, "buildCommandLines"],
    ["bridgeJs_buildApplyPayload", loaded.bridgeJs, "buildApplyPayload"],
    ["bridgeJs_installActions", loaded.bridgeJs, "installActions"],
    ["integratedRuntime_applyOverrides", loaded.integratedRuntime, "kgw_apply_command_preview_overrides"],
    ["integratedRuntime_workerStart", loaded.integratedRuntime, "kgw_worker_start"],
    ["tauriLib_trySelfWorkerArgs", loaded.tauriLib, "try_run_kgw_self_worker_from_args"],
    ["tauriLib_runBridgeSelfWorker", loaded.tauriLib, "kgw_run_bridge_self_worker"],
    ["rkBridge_startOfficialBridgeOwner", loaded.rkBridge, "start_official_bridge_owner_thread_v1"],
    ["rkBridge_startMainlineBridgeOwner", loaded.rkBridge, "start_mainline_bridge_owner_thread"],
    ["rkBridge_startTn12BridgeOwner", loaded.rkBridge, "start_tn12_bridge_owner_thread"],
    ["rkBridge_bridgeEventFromSettings", loaded.rkBridge, "bridge_service_event_from_settings_v1"],
  ];

  const extractedFunctions = {};
  for (const [label, file, fn] of extracts) {
    const ex = extractFunctionByName(file.text, fn);
    extractedFunctions[label] = ex ? {
      rel: file.rel,
      name: fn,
      startLine: ex.startLine,
      endLine: ex.endLine,
    } : null;

    if (ex) {
      writeText(path.join("extracts", `${label}.txt`), ex.text);
    } else {
      const around = extractAround(file.text, fn, 55, 90);
      if (around) {
        writeText(path.join("extracts", `${label}_around.txt`), around.text);
      }
    }
  }
  evidence.extractedFunctions = extractedFunctions;

  const bridgePayloadExtract = extractedFunctions.bridgeJs_buildApplyPayload
    ? fs.readFileSync(path.join(reportDir, "extracts", "bridgeJs_buildApplyPayload.txt"), "utf8")
    : "";

  const runBridgeExtract = extractedFunctions.tauriLib_runBridgeSelfWorker
    ? fs.readFileSync(path.join(reportDir, "extracts", "tauriLib_runBridgeSelfWorker.txt"), "utf8")
    : "";

  const rkMainlineExtract = extractedFunctions.rkBridge_startMainlineBridgeOwner
    ? fs.readFileSync(path.join(reportDir, "extracts", "rkBridge_startMainlineBridgeOwner.txt"), "utf8")
    : "";

  const rkTn12Extract = extractedFunctions.rkBridge_startTn12BridgeOwner
    ? fs.readFileSync(path.join(reportDir, "extracts", "rkBridge_startTn12BridgeOwner.txt"), "utf8")
    : "";

  const hasFrontendNodeMode = /\bnodeMode\b/.test(bridgeJs) && /--node-mode/.test(bridgeJs);
  const payloadForcesExternal =
    /bridgeKind\s*:\s*["']official-external-node["']/.test(bridgePayloadExtract) ||
    /nodeKind\s*:\s*["']remote["']/.test(bridgePayloadExtract);

  if (hasFrontendNodeMode) {
    addFinding(
      "INFO",
      "frontend-preview",
      "Bridge tab appears to build or display --node-mode from a nodeMode setting.",
      { file: loaded.bridgeJs.rel }
    );
  } else {
    addFinding(
      "HIGH",
      "frontend-preview",
      "Bridge tab did not show a clear nodeMode / --node-mode preview owner.",
      { file: loaded.bridgeJs.rel }
    );
  }

  if (payloadForcesExternal) {
    addFinding(
      "CRITICAL",
      "frontend-start-payload",
      "Bridge Start payload appears to force external-node semantics even when preview may show inprocess.",
      {
        file: loaded.bridgeJs.rel,
        evidence: ["bridgeKind: official-external-node", "nodeKind: remote"].filter(s => bridgePayloadExtract.includes(s.split(": ")[0])),
      }
    );
  } else {
    addFinding(
      "WARN",
      "frontend-start-payload",
      "Bridge Start payload did not clearly force external mode in the extracted buildApplyPayload function; inspect extract manually.",
      { file: loaded.bridgeJs.rel }
    );
  }

  const backendForcesExternal =
    /mode\s*:\s*kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialExternalNode/.test(runBridgeExtract) ||
    /BridgeRuntimeMode::OfficialExternalNode/.test(runBridgeExtract);

  const backendParsesNodeMode =
    /node[-_]mode/.test(runBridgeExtract) ||
    /OfficialInProcessNode/.test(runBridgeExtract) ||
    /--node-mode/.test(runBridgeExtract);

  if (backendForcesExternal && !backendParsesNodeMode) {
    addFinding(
      "CRITICAL",
      "tauri-self-worker",
      "kgw_run_bridge_self_worker appears to construct BridgeRuntimeSettings with OfficialExternalNode and does not parse node-mode into OfficialInProcessNode.",
      { file: loaded.tauriLib.rel }
    );
  } else if (backendForcesExternal && backendParsesNodeMode) {
    addFinding(
      "HIGH",
      "tauri-self-worker",
      "kgw_run_bridge_self_worker references external mode and node-mode/inprocess markers; inspect whether branching is real or dead.",
      { file: loaded.tauriLib.rel }
    );
  } else {
    addFinding(
      "WARN",
      "tauri-self-worker",
      "Could not prove forced external mode from kgw_run_bridge_self_worker extract.",
      { file: loaded.tauriLib.rel }
    );
  }

  const enumHasInprocess =
    /OfficialInProcessNode/.test(rkBridge) &&
    /StartOfficialInProcessNode/.test(rkBridge);

  if (enumHasInprocess) {
    addFinding(
      "INFO",
      "rk-bridge-api",
      "Bridge crate exposes OfficialInProcessNode / StartOfficialInProcessNode names.",
      { file: loaded.rkBridge.rel }
    );
  } else {
    addFinding(
      "HIGH",
      "rk-bridge-api",
      "Bridge crate does not expose clear in-process runtime names.",
      { file: loaded.rkBridge.rel }
    );
  }

  const inprocessEmptyBlocks = findAll(rkBridge, /if\s+event\.mode\s*==\s*BridgeRuntimeMode::OfficialInProcessNode\s*\{\s*\}/);
  if (inprocessEmptyBlocks.length > 0) {
    addFinding(
      "CRITICAL",
      "rk-bridge-implementation",
      "Bridge owner contains empty OfficialInProcessNode branches; in-process mode name exists but appears unimplemented.",
      { file: loaded.rkBridge.rel, lines: inprocessEmptyBlocks.map(x => x.line) }
    );
  }

  const mainlineUsesRpcApi =
    /KaspaApi::new\s*\(\s*event\.kaspa_rpc_endpoint/.test(rkMainlineExtract);
  const tn12UsesRpcApi =
    /KaspaApi::new\s*\(\s*event\.kaspa_rpc_endpoint/.test(rkTn12Extract);

  if (mainlineUsesRpcApi || tn12UsesRpcApi) {
    addFinding(
      "HIGH",
      "rk-bridge-owner-runtime",
      "Bridge owner startup uses event.kaspa_rpc_endpoint via KaspaApi::new, which is external-node style connectivity.",
      {
        file: loaded.rkBridge.rel,
        mainlineUsesRpcApi,
        tn12UsesRpcApi,
      }
    );
  }

  const kaspadLaunchSignals = [
    /kaspad/i,
    /start.*node/i,
    /spawn/i,
    /Command::new/i,
    /StartOfficialInProcessNode/,
    /--utxoindex/,
    /--rpclisten/,
    /appdir/,
  ];

  const inprocessImplementationSignals = kaspadLaunchSignals
    .map(re => ({ pattern: String(re), mainline: re.test(rkMainlineExtract), tn12: re.test(rkTn12Extract), runBridge: re.test(runBridgeExtract) }));

  evidence.inprocessImplementationSignals = inprocessImplementationSignals;

  const hasSpawnInOwner = /Command::new|\.spawn\s*\(/.test(rkMainlineExtract + "\n" + rkTn12Extract + "\n" + runBridgeExtract);
  const hasKaspadArgsInOwner = /--utxoindex|--rpclisten|appdir|kaspad/i.test(rkMainlineExtract + "\n" + rkTn12Extract + "\n" + runBridgeExtract);

  if (!hasSpawnInOwner && !hasKaspadArgsInOwner) {
    addFinding(
      "CRITICAL",
      "inprocess-missing-runtime",
      "No clear kaspad spawn or kaspad-argument handling was found in the bridge self-worker/owner extracts. This strongly indicates in-process mode is not actually implemented.",
      { files: [loaded.tauriLib.rel, loaded.rkBridge.rel] }
    );
  } else {
    addFinding(
      "WARN",
      "inprocess-runtime-signals",
      "Some kaspad/spawn/argument signals exist; inspect extracts to decide whether they are real implementation or unrelated text.",
      { signals: inprocessImplementationSignals }
    );
  }

  const commandPreviewHasSeparator =
    /--/.test(bridgeJs) &&
    /rpclisten/.test(bridgeJs);

  if (commandPreviewHasSeparator) {
    addFinding(
      "INFO",
      "frontend-command-preview",
      "Bridge command preview appears to include in-process separator/kaspad-argument concepts.",
      { file: loaded.bridgeJs.rel }
    );
  } else {
    addFinding(
      "HIGH",
      "frontend-command-preview",
      "Bridge command preview does not clearly include the required in-process '--' separator and kaspad args.",
      { file: loaded.bridgeJs.rel }
    );
  }

  const nextPatchTargets = [
    {
      file: loaded.bridgeJs.rel,
      requiredChange: "buildApplyPayload must preserve selected nodeMode and bridge runtime mode instead of forcing official-external-node / remote.",
    },
    {
      file: loaded.tauriLib.rel,
      requiredChange: "kgw_run_bridge_self_worker must parse selected node-mode / bridge kind into BridgeRuntimeMode::OfficialExternalNode or OfficialInProcessNode.",
    },
    {
      file: loaded.rkBridge.rel,
      requiredChange: "OfficialInProcessNode path must actually start or attach an in-process kaspad runtime and pass kaspad args after the '--' separator, instead of using only event.kaspa_rpc_endpoint.",
    },
  ];

  evidence.nextPatchTargets = nextPatchTargets;

  const verdict =
    findings.some(f => f.severity === "CRITICAL")
      ? "BRIDGE_INPROCESS_ROUTING_BROKEN_CONFIRMED"
      : findings.some(f => f.severity === "HIGH")
        ? "BRIDGE_INPROCESS_ROUTING_RISK_FOUND"
        : "BRIDGE_INPROCESS_ROUTING_NOT_PROVEN_BROKEN";

  evidence.verdict = verdict;
}

const nodeChecks = {};
if (loaded.bridgeJs.exists) {
  nodeChecks.bridgeJsSyntax = command("node", ["--check", loaded.bridgeJs.abs], repoRoot);
}
if (loaded.nodeJs.exists) {
  nodeChecks.nodeJsSyntax = command("node", ["--check", loaded.nodeJs.abs], repoRoot);
}

writeText("EVIDENCE.json", JSON.stringify(evidence, null, 2));
writeText("FINDINGS.json", JSON.stringify({ blockers, warnings, findings, nodeChecks }, null, 2));

const critical = findings.filter(f => f.severity === "CRITICAL");
const high = findings.filter(f => f.severity === "HIGH");
const info = findings.filter(f => f.severity === "INFO");
const warn = findings.filter(f => f.severity === "WARN");

const reportLines = [];
reportLines.push(`# KGW Bridge Node Mode Routing Audit V1`);
reportLines.push("");
reportLines.push(`- Repository: ${repoRoot}`);
reportLines.push(`- Report directory: ${reportDir}`);
reportLines.push(`- Source mutation: no`);
reportLines.push(`- Backup created: no, audit only`);
reportLines.push(`- Git commit: no`);
reportLines.push(`- Git push: no`);
reportLines.push("");
reportLines.push(`## Verdict`);
reportLines.push("");
if (blockers.length > 0) {
  reportLines.push(`FAILED: required files are missing.`);
} else if (critical.length > 0) {
  reportLines.push(`CONFIRMED: Bridge in-process routing is broken or incomplete.`);
} else if (high.length > 0) {
  reportLines.push(`RISK FOUND: Bridge in-process routing has high-risk issues, but no critical proof was found.`);
} else {
  reportLines.push(`NOT CONFIRMED: no major routing break was proven by this audit.`);
}
reportLines.push("");
reportLines.push(`## Blockers`);
reportLines.push("");
if (blockers.length === 0) {
  reportLines.push(`- None`);
} else {
  for (const b of blockers) reportLines.push(`- ${b}`);
}
reportLines.push("");
reportLines.push(`## Critical Findings`);
reportLines.push("");
if (critical.length === 0) reportLines.push(`- None`);
for (const f of critical) {
  reportLines.push(`- [${f.area}] ${f.message}`);
  if (f.file) reportLines.push(`  - File: ${f.file}`);
  if (f.lines) reportLines.push(`  - Lines: ${f.lines.join(", ")}`);
}
reportLines.push("");
reportLines.push(`## High Findings`);
reportLines.push("");
if (high.length === 0) reportLines.push(`- None`);
for (const f of high) {
  reportLines.push(`- [${f.area}] ${f.message}`);
  if (f.file) reportLines.push(`  - File: ${f.file}`);
}
reportLines.push("");
reportLines.push(`## Warnings`);
reportLines.push("");
if (warn.length === 0) reportLines.push(`- None`);
for (const f of warn) {
  reportLines.push(`- [${f.area}] ${f.message}`);
  if (f.file) reportLines.push(`  - File: ${f.file}`);
}
reportLines.push("");
reportLines.push(`## Informational Findings`);
reportLines.push("");
if (info.length === 0) reportLines.push(`- None`);
for (const f of info) {
  reportLines.push(`- [${f.area}] ${f.message}`);
  if (f.file) reportLines.push(`  - File: ${f.file}`);
}
reportLines.push("");
reportLines.push(`## Extracts Saved`);
reportLines.push("");
reportLines.push(`- full-files/`);
reportLines.push(`- extracts/`);
reportLines.push(`- EVIDENCE.json`);
reportLines.push(`- FINDINGS.json`);
reportLines.push("");
reportLines.push(`## Expected Correct Model From RKStratumTN12 Docs`);
reportLines.push("");
reportLines.push(`- external mode: stratum-bridge connects to an already running kaspad through RPC.`);
reportLines.push(`- inprocess mode: stratum-bridge starts/runs kaspad internally and kaspad args must be passed after the required '--' separator.`);
reportLines.push(`- internal CPU miner is a separate flag axis and must not be treated as equivalent to inprocess node mode.`);
reportLines.push("");
reportLines.push(`## Next Safe Step`);
reportLines.push("");
if (critical.length > 0 || high.length > 0) {
  reportLines.push(`Write a patch only after reviewing this report. The patch must preserve selected bridge nodeMode from Bridge tab into Tauri backend, map it into BridgeRuntimeMode, and implement or route true in-process owner behavior without adding duplicate UI/runtime owners.`);
} else {
  reportLines.push(`Manually inspect extracts before patching because the audit did not prove the break.`);
}
reportLines.push("");

const reportName = blockers.length > 0 ? "REPORT_FAILED.md" : "REPORT_SUCCESS.md";
writeText(reportName, reportLines.join("\n"));

console.log(reportLines.join("\n"));

if (blockers.length > 0) {
  process.exit(1);
}
process.exit(0);
