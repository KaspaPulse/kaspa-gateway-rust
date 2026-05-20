#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];
const backupDir = process.argv[4];

if (!repoRoot || !reportDir || !backupDir) {
  console.error("Usage: node kgw_settings_owner_v19_feedback_dirty_fix_v22.cjs <repoRoot> <reportDir> <backupDir>");
  process.exit(2);
}

const OWNER = "KGW_SETTINGS_OWNER_V19";
const PATCH = "KGW_SETTINGS_OWNER_V19_FEEDBACK_DIRTY_FIX_V22";

const mutableFiles = [
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
];

const readOnlyFiles = [
  "tools/kgw_global_owner_gate.cjs",
  "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"
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

function count(text, needle) {
  let n = 0;
  let offset = 0;
  while (true) {
    const index = text.indexOf(needle, offset);
    if (index < 0) break;
    n += 1;
    offset = index + needle.length;
  }
  return n;
}

function replaceExact(text, from, to, rel) {
  if (!text.includes(from)) {
    throw new Error(rel + " missing exact block:\n" + from.slice(0, 500));
  }
  return text.split(from).join(to);
}

function patchFile(rel) {
  const before = read(rel);
  save("FULL_BEFORE__" + safeName(rel), before);

  if (!before.includes(OWNER)) {
    throw new Error(rel + " does not contain " + OWNER);
  }

  if (!before.includes("KGW_SETTINGS_OWNER_V19_TRACE_CONTRACT_FIX_V21")) {
    throw new Error(rel + " does not contain the V21 trace contract fix. Refusing blind patch.");
  }

  let after = before;

  after = replaceExact(
    after,
    "  const FEEDBACK_MS = 10000;",
    "  const FEEDBACK_MS = 5000;",
    rel
  );

  after = replaceExact(
    after,
    "  const locks = new WeakMap();\n  const timers = new WeakMap();",
    "  const locks = new WeakMap();\n  const timers = new WeakMap();\n  const dirtyDuringFeedback = new WeakMap();",
    rel
  );

  after = replaceExact(
    after,
    `  function lockMap(root) {
    let map = locks.get(root);
    if (!map) {
      map = new Map();
      locks.set(root, map);
    }
    return map;
  }`,
    `  function lockMap(root) {
    let map = locks.get(root);
    if (!map) {
      map = new Map();
      locks.set(root, map);
    }
    return map;
  }

  // ${PATCH}
  function dirtyMap(root) {
    let map = dirtyDuringFeedback.get(root);
    if (!map) {
      map = new Map();
      dirtyDuringFeedback.set(root, map);
    }
    return map;
  }

  function markDirtyDuringFeedback(root, network, reason) {
    dirtyMap(root).set(network, true);
    trace(root, "v19-dirty-during-feedback", { network: network, reason: reason || "trusted-change-during-feedback" });
  }

  function consumeDirtyDuringFeedback(root, network) {
    const map = dirtyMap(root);
    const value = map.get(network) === true;
    map.set(network, false);
    return value;
  }`,
    rel
  );

  after = replaceExact(
    after,
    `      if (isLocked(root, network)) {
        setDisabled(root, network, true, "input-locked");
        return;
      }
      if (!event.isTrusted) {
        setDisabled(root, network, true, "input-programmatic");
        return;
      }
      setDisabled(root, network, false, "trusted-input");`,
    `      if (isLocked(root, network)) {
        if (event.isTrusted) {
          markDirtyDuringFeedback(root, network, "trusted-input-during-feedback");
        }
        setDisabled(root, network, true, event.isTrusted ? "trusted-input-pending-feedback" : "input-programmatic-locked");
        return;
      }
      if (!event.isTrusted) {
        setDisabled(root, network, true, "input-programmatic");
        return;
      }
      setDisabled(root, network, false, "trusted-input");`,
    rel
  );

  after = replaceExact(
    after,
    `      if (isLocked(root, network)) {
        setDisabled(root, network, true, "change-locked");
        return;
      }
      if (!event.isTrusted) {
        setDisabled(root, network, true, "change-programmatic");
        return;
      }
      setDisabled(root, network, false, "trusted-change");`,
    `      if (isLocked(root, network)) {
        if (event.isTrusted) {
          markDirtyDuringFeedback(root, network, "trusted-change-during-feedback");
        }
        setDisabled(root, network, true, event.isTrusted ? "trusted-change-pending-feedback" : "change-programmatic-locked");
        return;
      }
      if (!event.isTrusted) {
        setDisabled(root, network, true, "change-programmatic");
        return;
      }
      setDisabled(root, network, false, "trusted-change");`,
    rel
  );

  after = replaceExact(
    after,
    `  function startFeedback(root, network, button, action) {
    setLock(root, network);
    rememberLabel(button, action);
    window.setTimeout(function () {
      button.textContent = feedbackText(action);
      setDisabled(root, network, true, "feedback-start");
      const oldTimer = timers.get(button);
      if (oldTimer) window.clearTimeout(oldTimer);
      const timer = window.setTimeout(function () {
        timers.delete(button);
        clearLock(root, network);
        restoreLabel(button, action);
        setDisabled(root, network, true, "feedback-complete");
        trace(root, "v19-feedback-complete", { network: network, action: action });
      }, FEEDBACK_MS);
      timers.set(button, timer);
      trace(root, "v19-feedback-start", { network: network, action: action, holdMs: FEEDBACK_MS });
    }, 0);
  }`,
    `  function startFeedback(root, network, button, action) {
    setLock(root, network);
    dirtyMap(root).set(network, false);
    rememberLabel(button, action);

    const label = feedbackText(action);
    const oldTimer = timers.get(button);
    if (oldTimer) window.clearTimeout(oldTimer);

    button.textContent = label;
    setDisabled(root, network, true, "feedback-start");

    const labelKeeper = window.setInterval(function () {
      if (isLocked(root, network)) {
        button.textContent = label;
      }
    }, 250);

    const timer = window.setTimeout(function () {
      window.clearInterval(labelKeeper);
      timers.delete(button);
      clearLock(root, network);
      restoreLabel(button, action);

      if (consumeDirtyDuringFeedback(root, network)) {
        setDisabled(root, network, false, "feedback-complete-with-pending-dirty");
      } else {
        setDisabled(root, network, true, "feedback-complete");
      }

      trace(root, "v19-feedback-complete", { network: network, action: action, holdMs: FEEDBACK_MS });
    }, FEEDBACK_MS);

    timers.set(button, timer);
    trace(root, "v19-feedback-start", { network: network, action: action, holdMs: FEEDBACK_MS, label: label });
  }`,
    rel
  );

  if (after.includes("const FEEDBACK_MS = 10000")) {
    report.validationErrors.push(rel + " still has FEEDBACK_MS 10000.");
  }

  if (!after.includes("const FEEDBACK_MS = 5000")) {
    report.validationErrors.push(rel + " missing FEEDBACK_MS 5000.");
  }

  if (!after.includes(PATCH)) {
    report.validationErrors.push(rel + " missing V22 marker.");
  }

  if (!after.includes("v19-dirty-during-feedback")) {
    report.validationErrors.push(rel + " missing dirty during feedback trace.");
  }

  if (!after.includes("feedback-complete-with-pending-dirty")) {
    report.validationErrors.push(rel + " missing pending dirty completion behavior.");
  }

  if (!after.includes("setInterval(function ()")) {
    report.validationErrors.push(rel + " missing label keeper interval.");
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
    "# " + (success ? "SUCCESS" : "FAILED") + " - KGW Settings Owner V19 Feedback Dirty Fix V22",
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

  patchFile(mutableFiles[0]);
  patchFile(mutableFiles[1]);

  if (report.validationErrors.length) {
    throw new Error("Validation failed before gates.");
  }

  const nodeJs = read(mutableFiles[0]);
  const bridgeJs = read(mutableFiles[1]);

  const oldForbidden = [
    "KGW_SETTINGS_OWNER_V18",
    "KGW_SETTINGS_OWNER_V17",
    "KGW_SETTINGS_OWNER_V16",
    "KGW_SETTINGS_OWNER_FINAL_V15",
    "KGW_SETTINGS_SINGLE_OWNER_R14",
    "KGW_SETTINGS_CANONICAL_OWNER_R13",
    "KGW_SETTINGS_UNIFIED_OWNER_R12",
    "settings-buttons",
    "nativeDisabledExpected",
    "click-received",
    "click-ignored-disabled",
    "action-start",
    "auto-baseline-before-input"
  ];

  for (const token of oldForbidden) {
    if (nodeJs.includes(token)) report.validationErrors.push("Node JS contains old forbidden token: " + token);
    if (bridgeJs.includes(token)) report.validationErrors.push("Bridge JS contains old forbidden token: " + token);
  }

  if (count(nodeJs, "KGW_SETTINGS_OWNER_V19") < 2) report.validationErrors.push("Node JS V19 owner marker missing/incomplete.");
  if (count(bridgeJs, "KGW_SETTINGS_OWNER_V19") < 2) report.validationErrors.push("Bridge JS V19 owner marker missing/incomplete.");

  if (report.validationErrors.length) {
    throw new Error("Post-patch static validation failed.");
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

  finish(true, "V19 now uses 5s feedback, preserves feedback labels, and records trusted changes during feedback.");
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
