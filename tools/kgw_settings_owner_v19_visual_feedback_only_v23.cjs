#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];
const backupDir = process.argv[4];

if (!repoRoot || !reportDir || !backupDir) {
  console.error("Usage: node kgw_settings_owner_v19_visual_feedback_only_v23.cjs <repoRoot> <reportDir> <backupDir>");
  process.exit(2);
}

const OWNER = "KGW_SETTINGS_OWNER_V19";
const PATCH = "KGW_SETTINGS_OWNER_V19_VISUAL_FEEDBACK_ONLY_V23";
const VISUAL = "KGW_SETTINGS_OWNER_V19_VISUAL";
const DISABLED_CLASS = "kgw-settings-action-disabled-v19";

const mutableFiles = [
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"
];

const cssFiles = [
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
  "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"
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
    const i = text.indexOf(needle, offset);
    if (i < 0) break;
    n += 1;
    offset = i + needle.length;
  }
  return n;
}

function ownerSource(scope) {
  const globalName = scope === "node" ? "KGW_NODE_SETTINGS_OWNER_V19" : "KGW_BRIDGE_SETTINGS_OWNER_V19";

  return `// ${OWNER}
(function installKgwSettingsOwnerV19() {
  "use strict";

  const OWNER = "${OWNER}";
  const PATCH = "${PATCH}";
  const SCOPE = "${scope}";
  const GLOBAL_NAME = "${globalName}";
  const FEEDBACK_MS = 3000;
  const DISABLED_CLASS = "${DISABLED_CLASS}";
  const ROOT_INSTALLED_ATTR = "kgwSettingsOwnerV19";

  const feedbackByRoot = new WeakMap();
  const dirtyByRoot = new WeakMap();

  function lower(value) {
    return String(value || "").toLowerCase();
  }

  function trace(root, phase, details) {
    try {
      const safeDetails = details && typeof details === "object" ? details : {};
      const net = String(safeDetails.network || safeDetails.net || "unknown");
      const action = String(safeDetails.action || "settings-owner");
      const detailsText = JSON.stringify({
        owner: OWNER,
        patch: PATCH,
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
  }

  function translate(key, fallback) {
    try {
      const candidates = [window.kgwT, window.__kgwT, window.t];
      for (const fn of candidates) {
        if (typeof fn === "function") {
          const value = fn(key, fallback);
          if (typeof value === "string" && value.trim() && value !== key) return value;
        }
      }

      const apis = [window.kgwI18n, window.KGWI18n, window.KGW_I18N, window.i18n];
      for (const api of apis) {
        if (api && typeof api.t === "function") {
          const value = api.t(key, fallback);
          if (typeof value === "string" && value.trim() && value !== key) return value;
        }
        if (api && typeof api.translate === "function") {
          const value = api.translate(key, fallback);
          if (typeof value === "string" && value.trim() && value !== key) return value;
        }
      }
    } catch (_) {}
    return fallback;
  }

  function isSettingsControl(element) {
    if (!element || !element.tagName) return false;

    const tag = lower(element.tagName);
    if (tag !== "input" && tag !== "select" && tag !== "textarea") return false;

    const type = lower(element.type);
    if (type === "button" || type === "submit" || type === "reset" || type === "hidden") return false;

    if (element.closest && element.closest(".logs, .log, [data-log], .kgw-log-pane")) return false;

    return true;
  }

  function isActionButton(element) {
    if (!element || !element.tagName || lower(element.tagName) !== "button") return false;

    const text = lower(element.textContent);
    const action = lower(
      (element.dataset && (element.dataset.kgwSettingsAction || element.dataset.action)) ||
        element.getAttribute("data-action") ||
        element.getAttribute("aria-label") ||
        ""
    );

    return (
      action.includes("save") ||
      action.includes("restore") ||
      action.includes("default") ||
      text.includes("save settings") ||
      text.includes("restore defaults") ||
      text.includes("set as defaults") ||
      text.includes("saved") ||
      text.includes("restored") ||
      text.includes("حفظ") ||
      text.includes("استعادة") ||
      text.includes("افتراض")
    );
  }

  function networkOf(element) {
    let current = element;

    while (current && current !== document) {
      const dataset = current.dataset || {};
      const direct =
        dataset.network ||
        dataset.net ||
        dataset.kgwNetwork ||
        current.getAttribute("data-network") ||
        current.getAttribute("data-net") ||
        current.getAttribute("data-kgw-network");

      if (direct) return String(direct);

      const id = lower(current.id);
      const cls = lower(current.className);

      if (id.includes("testnet12") || cls.includes("testnet12") || id.includes("tn12") || cls.includes("tn12")) return "testnet12";
      if (id.includes("testnet10") || cls.includes("testnet10") || id.includes("tn10") || cls.includes("tn10")) return "testnet10";
      if (id.includes("mainnet") || cls.includes("mainnet")) return "mainnet";

      current = current.parentElement;
    }

    return "mainnet";
  }

  function actionName(button) {
    const raw = lower(
      (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
        button.getAttribute("data-action") ||
        button.getAttribute("aria-label") ||
        button.textContent ||
        ""
    );

    if (raw.includes("restore") || raw.includes("استعادة")) return "restore";
    if (raw.includes("default") || raw.includes("افتراض")) return "defaults";
    return "save";
  }

  function feedbackText(action) {
    if (action === "restore") return translate("settings.feedback.restored", "Restored");
    if (action === "defaults") return translate("settings.feedback.setAsDefaults", "Set as defaults");
    return translate("settings.feedback.saved", "Saved");
  }

  function fallbackText(action) {
    if (action === "restore") return "Restore Defaults";
    if (action === "defaults") return "Set as Defaults";
    return "Save Settings";
  }

  function allButtons(root) {
    return Array.from(root.querySelectorAll("button")).filter(isActionButton);
  }

  function buttons(root, network) {
    return allButtons(root).filter(function (button) {
      return !network || network === "all" || networkOf(button) === network;
    });
  }

  function dirtyMap(root) {
    let map = dirtyByRoot.get(root);
    if (!map) {
      map = new Map();
      dirtyByRoot.set(root, map);
    }
    return map;
  }

  function setDirty(root, network, dirty, reason) {
    dirtyMap(root).set(network, !!dirty);
    setDisabled(root, network, !dirty, reason || (dirty ? "dirty" : "clean"));
  }

  function feedbackMap(root) {
    let map = feedbackByRoot.get(root);
    if (!map) {
      map = new Map();
      feedbackByRoot.set(root, map);
    }
    return map;
  }

  function rememberLabel(button, action) {
    const current = String(button.textContent || "").trim();
    const normalized = lower(current);

    if (
      !button.dataset.kgwSettingsOwnerV19OriginalLabel ||
      normalized === "saved" ||
      normalized === "restored" ||
      normalized === "set as defaults"
    ) {
      button.dataset.kgwSettingsOwnerV19OriginalLabel =
        current && normalized !== "saved" && normalized !== "restored" && normalized !== "set as defaults"
          ? current
          : fallbackText(action);
    }
  }

  function restoreLabel(button, action) {
    button.textContent = button.dataset.kgwSettingsOwnerV19OriginalLabel || fallbackText(action);
  }

  function restoreLabels(root, network) {
    buttons(root, network).forEach(function (button) {
      restoreLabel(button, actionName(button));
    });
  }

  function clearFeedback(root, network, reason) {
    const map = feedbackMap(root);
    const active = map.get(network);

    if (active) {
      if (active.timer) window.clearTimeout(active.timer);
      if (active.interval) window.clearInterval(active.interval);
      if (active.button) restoreLabel(active.button, active.action);
      map.delete(network);
      trace(root, "v19-feedback-cleared", { network: network, reason: reason || "clear" });
    }
  }

  function setDisabled(root, network, disabled, reason) {
    buttons(root, network || "all").forEach(function (button) {
      button.disabled = !!disabled;
      button.setAttribute("aria-disabled", disabled ? "true" : "false");
      button.classList.toggle(DISABLED_CLASS, !!disabled);
      button.dataset.kgwSettingsOwnerV19Disabled = disabled ? "true" : "false";
    });

    trace(root, disabled ? "v19-disabled" : "v19-enabled", {
      network: network || "all",
      reason: reason || "unspecified"
    });
  }

  function startVisualFeedbackAfterOriginalClick(root, network, button, action) {
    window.setTimeout(function () {
      clearFeedback(root, network, "new-feedback");

      dirtyMap(root).set(network, false);
      rememberLabel(button, action);

      const label = feedbackText(action);
      button.textContent = label;
      setDisabled(root, network, true, "feedback-clean-state");

      const interval = window.setInterval(function () {
        const active = feedbackMap(root).get(network);
        if (active && active.button === button) {
          button.textContent = label;
        }
      }, 200);

      const timer = window.setTimeout(function () {
        const active = feedbackMap(root).get(network);
        if (!active || active.button !== button) return;

        window.clearInterval(interval);
        feedbackMap(root).delete(network);
        restoreLabel(button, action);

        const dirty = dirtyMap(root).get(network) === true;
        setDisabled(root, network, !dirty, dirty ? "feedback-complete-dirty" : "feedback-complete-clean");

        trace(root, "v19-feedback-complete", {
          network: network,
          action: action,
          holdMs: FEEDBACK_MS,
          dirty: dirty
        });
      }, FEEDBACK_MS);

      feedbackMap(root).set(network, {
        timer: timer,
        interval: interval,
        button: button,
        action: action
      });

      trace(root, "v19-feedback-start", {
        network: network,
        action: action,
        holdMs: FEEDBACK_MS,
        label: label,
        visualOnly: true
      });
    }, 0);
  }

  function install(root) {
    if (!root || root.dataset[ROOT_INSTALLED_ATTR] === "installed") return;

    root.dataset[ROOT_INSTALLED_ATTR] = "installed";
    setDisabled(root, "all", true, "initial");

    root.addEventListener("input", function (event) {
      if (!isSettingsControl(event.target)) return;

      const network = networkOf(event.target);

      if (!event.isTrusted) {
        setDisabled(root, network, true, "input-programmatic");
        return;
      }

      clearFeedback(root, network, "trusted-input");
      restoreLabels(root, network);
      setDirty(root, network, true, "trusted-input");
    }, true);

    root.addEventListener("change", function (event) {
      if (!isSettingsControl(event.target)) return;

      const network = networkOf(event.target);

      if (!event.isTrusted) {
        setDisabled(root, network, true, "change-programmatic");
        return;
      }

      clearFeedback(root, network, "trusted-change");
      restoreLabels(root, network);
      setDirty(root, network, true, "trusted-change");
    }, true);

    root.addEventListener("click", function (event) {
      const button = event.target && event.target.closest ? event.target.closest("button") : null;
      if (!button || !root.contains(button) || !isActionButton(button)) return;

      const network = networkOf(button);
      const action = actionName(button);
      const disabled = !!button.disabled || button.dataset.kgwSettingsOwnerV19Disabled === "true";

      trace(root, "v19-click", {
        network: network,
        action: action,
        disabled: disabled,
        label: String(button.textContent || "").trim()
      });

      if (disabled) {
        event.preventDefault();
        event.stopImmediatePropagation();
        setDisabled(root, network, true, "click-blocked-clean-state");
        return;
      }

      startVisualFeedbackAfterOriginalClick(root, network, button, action);
    }, true);

    trace(root, "v19-owner-installed", { scope: SCOPE, patch: PATCH, feedbackMs: FEEDBACK_MS });
  }

  window[GLOBAL_NAME] = {
    install: install,
    setDisabled: setDisabled,
    buttons: buttons
  };

  window.KGW_SETTINGS_OWNER_V19 = window[GLOBAL_NAME];
})();
// END_KGW_SETTINGS_OWNER_V19
`;
}

function replaceOwnerBlock(rel, text, scope) {
  const startToken = "// KGW_SETTINGS_OWNER_V19";
  const endToken = "// END_KGW_SETTINGS_OWNER_V19";

  const start = text.indexOf(startToken);
  if (start < 0) throw new Error(rel + " missing owner start marker.");

  const end = text.indexOf(endToken, start);
  if (end < 0) throw new Error(rel + " missing owner end marker.");

  const afterEnd = end + endToken.length;
  const next = text.slice(afterEnd).startsWith("\r\n") ? afterEnd + 2 : text.slice(afterEnd).startsWith("\n") ? afterEnd + 1 : afterEnd;

  return text.slice(0, start) + ownerSource(scope) + "\n" + text.slice(next);
}

function patchJs(rel, scope) {
  const before = read(rel);
  save("FULL_BEFORE__" + safeName(rel), before);

  if (!before.includes("window.KGW_" + scope.toUpperCase() + "_SETTINGS_OWNER_V19.install(root)") && !before.includes("KGW_" + scope.toUpperCase() + "_SETTINGS_OWNER_V19")) {
    throw new Error(rel + " does not appear routed to V19 owner.");
  }

  const after = replaceOwnerBlock(rel, before, scope);

  if (!after.includes(PATCH)) report.validationErrors.push(rel + " missing V23 patch marker.");
  if (!after.includes("const FEEDBACK_MS = 3000")) report.validationErrors.push(rel + " missing 3000ms feedback.");
  if (after.includes("const FEEDBACK_MS = 5000")) report.validationErrors.push(rel + " still contains 5000ms feedback.");
  if (after.includes("const FEEDBACK_MS = 10000")) report.validationErrors.push(rel + " still contains 10000ms feedback.");
  if (after.includes("input-locked")) report.validationErrors.push(rel + " still contains input-locked.");
  if (after.includes("change-locked")) report.validationErrors.push(rel + " still contains change-locked.");
  if (after.includes("dirtyDuringFeedback")) report.validationErrors.push(rel + " still contains dirtyDuringFeedback layer.");
  if (!after.includes("visualOnly: true")) report.validationErrors.push(rel + " missing visualOnly feedback marker.");
  if (!after.includes("clearFeedback(root, network, \"trusted-input\")")) report.validationErrors.push(rel + " does not clear feedback on trusted input.");
  if (!after.includes("clearFeedback(root, network, \"trusted-change\")")) report.validationErrors.push(rel + " does not clear feedback on trusted change.");

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
    "# " + (success ? "SUCCESS" : "FAILED") + " - KGW Settings Owner V19 Visual Feedback Only V23",
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

  for (const rel of mutableFiles.concat(cssFiles).concat(readOnlyFiles)) {
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

  patchJs(mutableFiles[0], "node");
  patchJs(mutableFiles[1], "bridge");

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
    "auto-baseline-before-input",
    "input-locked",
    "change-locked",
    "dirtyDuringFeedback"
  ];

  for (const token of oldForbidden) {
    if (nodeJs.includes(token)) report.validationErrors.push("Node JS contains forbidden token: " + token);
    if (bridgeJs.includes(token)) report.validationErrors.push("Bridge JS contains forbidden token: " + token);
  }

  if (count(nodeJs, "KGW_SETTINGS_OWNER_V19") < 2) report.validationErrors.push("Node JS V19 owner marker missing/incomplete.");
  if (count(bridgeJs, "KGW_SETTINGS_OWNER_V19") < 2) report.validationErrors.push("Bridge JS V19 owner marker missing/incomplete.");

  if (!nodeJs.includes("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)")) {
    report.validationErrors.push("Node installActions is not routed to V19.");
  }

  if (!bridgeJs.includes("window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)")) {
    report.validationErrors.push("Bridge installActions is not routed to V19.");
  }

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

  finish(true, "V19 rewritten as visual-only 3-second feedback. No feedback lock remains.");
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
