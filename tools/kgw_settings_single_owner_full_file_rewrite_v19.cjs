#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];
const backupDir = process.argv[4];

if (!repoRoot || !reportDir || !backupDir) {
  console.error("Usage: node kgw_settings_single_owner_full_file_rewrite_v19.cjs <repoRoot> <reportDir> <backupDir>");
  process.exit(2);
}

const OWNER = "KGW_SETTINGS_OWNER_V19";
const VISUAL = "KGW_SETTINGS_OWNER_V19_VISUAL";
const DISABLED_CLASS = "kgw-settings-action-disabled-v19";

const targetFiles = {
  nodeJs: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
  bridgeJs: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
  nodeCss: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
  bridgeCss: "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css",
  gate: "tools/kgw_global_owner_gate.cjs"
};

const oldForbiddenTokens = [
  "KGW_SETTINGS_OWNER_V18",
  "KGW_SETTINGS_OWNER_V17",
  "KGW_SETTINGS_OWNER_V16",
  "KGW_SETTINGS_OWNER_FINAL_V15",
  "KGW_SETTINGS_SINGLE_OWNER_R14",
  "KGW_SETTINGS_CANONICAL_OWNER_R13",
  "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
  "KGW_SETTINGS_SELECTION_TRACE_R12",
  "KGW_SETTINGS_UNIFIED_OWNER_R12",
  "KGW_SETTINGS_BUTTONS_R",
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

const report = {
  runName: path.basename(reportDir),
  repoRoot,
  reportDir,
  backupDir,
  owner: OWNER,
  startedAt: new Date().toISOString(),
  changedFiles: [],
  backups: [],
  beforeFiles: [],
  afterFiles: [],
  checks: [],
  validationErrors: [],
  rollback: false,
  gitPush: false,
  gitCommit: false
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

function readFile(rel) {
  return fs.readFileSync(abs(rel), "utf8");
}

function writeFile(rel, text) {
  fs.writeFileSync(abs(rel), text, "utf8");
}

function saveReportFile(name, text) {
  const out = path.join(reportDir, name);
  mkdirp(path.dirname(out));
  fs.writeFileSync(out, text, "utf8");
  return out;
}

function copyForBackup(rel) {
  const source = abs(rel);
  const backup = path.join(backupDir, safeName(rel));
  mkdirp(path.dirname(backup));
  fs.copyFileSync(source, backup);
  report.backups.push({ rel, backup });
  return backup;
}

function saveBefore(rel, text) {
  const out = saveReportFile("FULL_BEFORE__" + safeName(rel), text);
  report.beforeFiles.push({ rel, out });
}

function saveAfter(rel, text) {
  const out = saveReportFile("FULL_AFTER__" + safeName(rel), text);
  report.afterFiles.push({ rel, out });
}

function run(command, args, options) {
  const result = cp.spawnSync(command, args, {
    cwd: options && options.cwd ? options.cwd : repoRoot,
    shell: false,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 80
  });

  const item = {
    command: [command].concat(args || []).join(" "),
    cwd: options && options.cwd ? options.cwd : repoRoot,
    status: result.status,
    stdout: result.stdout || "",
    stderr: result.stderr || ""
  };

  report.checks.push(item);

  saveReportFile(
    "CHECK__" + report.checks.length + "__" + command.replace(/[^\w.-]+/g, "_") + ".log",
    "COMMAND: " + item.command + "\nCWD: " + item.cwd + "\nSTATUS: " + item.status + "\n\n--- STDOUT ---\n" + item.stdout + "\n\n--- STDERR ---\n" + item.stderr
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
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === "'") {
        mode = "code";
      }
      continue;
    }

    if (mode === "double") {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === "\"") {
        mode = "code";
      }
      continue;
    }

    if (mode === "template") {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === "`") {
        mode = "code";
      }
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

    if (ch === "{") {
      depth += 1;
      continue;
    }

    if (ch === "}") {
      depth -= 1;
      if (depth === 0) return i;
    }
  }

  return -1;
}

function removeBlockByStartEnd(text, startToken, endToken) {
  let current = text;
  while (true) {
    const start = current.indexOf(startToken);
    if (start < 0) break;
    const end = current.indexOf(endToken, start + startToken.length);
    if (end < 0) break;
    const after = end + endToken.length;
    current = current.slice(0, start) + "\n" + current.slice(after);
  }
  return current;
}

function removeFunctionByName(text, functionName) {
  let current = text;
  const needle = "function " + functionName;
  while (true) {
    const start = current.indexOf(needle);
    if (start < 0) break;

    const brace = current.indexOf("{", start);
    if (brace < 0) break;

    const endBrace = findMatchingBrace(current, brace);
    if (endBrace < 0) break;

    let end = endBrace + 1;
    while (end < current.length && /\s/.test(current[end])) end += 1;
    if (current[end] === ";") end += 1;

    current = current.slice(0, start) + "\n" + current.slice(end);
  }
  return current;
}

function replaceFunctionByName(text, functionName, replacement) {
  const needle = "function " + functionName;
  const start = text.indexOf(needle);
  if (start < 0) {
    throw new Error("Missing function: " + functionName);
  }

  const brace = text.indexOf("{", start);
  if (brace < 0) {
    throw new Error("Missing opening brace for function: " + functionName);
  }

  const endBrace = findMatchingBrace(text, brace);
  if (endBrace < 0) {
    throw new Error("Missing closing brace for function: " + functionName);
  }

  let end = endBrace + 1;
  while (end < text.length && /\s/.test(text[end])) end += 1;
  if (text[end] === ";") end += 1;

  return text.slice(0, start) + replacement.trim() + "\n\n" + text.slice(end);
}

function removeKnownSettingOwnerLayers(text, scope) {
  let current = text;

  const blockPairs = [
    ["// KGW_SETTINGS_OWNER_V18", "// END_KGW_SETTINGS_OWNER_V18"],
    ["/* KGW_SETTINGS_OWNER_V18 */", "/* END_KGW_SETTINGS_OWNER_V18 */"],
    ["/* KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START */", "/* KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_END */"],
    ["/* KGW_NODE_SETTINGS_SIMPLE_UX_R7C_START */", "/* KGW_NODE_SETTINGS_SIMPLE_UX_R7C_END */"],
    ["/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C_START */", "/* KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C_END */"],
    ["/* KGW_SETTINGS_OWNER_CONSOLIDATION_R9B_START */", "/* KGW_SETTINGS_OWNER_CONSOLIDATION_R9B_END */"],
    ["/* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START */", "/* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END */"],
    ["/* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START */", "/* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END */"],
    ["/* KGW_SETTINGS_UNIFIED_OWNER_R12_START */", "/* KGW_SETTINGS_UNIFIED_OWNER_R12_END */"],
    ["/* KGW_SETTINGS_CANONICAL_OWNER_R13_START */", "/* KGW_SETTINGS_CANONICAL_OWNER_R13_END */"],
    ["/* KGW_SETTINGS_SINGLE_OWNER_R14_START */", "/* KGW_SETTINGS_SINGLE_OWNER_R14_END */"],
    ["/* KGW_SETTINGS_OWNER_FINAL_V15_START */", "/* KGW_SETTINGS_OWNER_FINAL_V15_END */"],
    ["/* KGW_SETTINGS_OWNER_V16_START */", "/* KGW_SETTINGS_OWNER_V16_END */"],
    ["/* KGW_SETTINGS_OWNER_V17_START */", "/* KGW_SETTINGS_OWNER_V17_END */"]
  ];

  for (const pair of blockPairs) {
    current = removeBlockByStartEnd(current, pair[0], pair[1]);
  }

  const prefix = scope === "node" ? "kgwNode" : "kgwBridge";
  const oldFunctions = [
    prefix + "SettingsNormalizeR7C",
    prefix + "SettingsSnapshotR7C",
    prefix + "SettingsChangedR7C",
    prefix + "SettingsNetFromEventR7C",
    prefix + "SettingsLanguageR7C",
    prefix + "SettingsFeedbackLabelR7C",
    prefix + "SettingsAfterActionR7C",
    prefix + "SettingsIsProgrammaticWriteR9B",
    prefix + "SettingsWithProgrammaticWriteR9B",
    prefix + "SettingsIsFeedbackLockedR11",
    prefix + "StableSettingsStringR2",
    prefix + "IsGeneratedRustyKaspaRootValueR2",
    prefix + "NormalizeReferenceSettingsR2",
    prefix + "SettingsEqualR2",
    prefix + "ReferenceSettingsR2",
    prefix + "SettingsTraceR4D",
    prefix + "CurrentLanguageR4D",
    prefix + "SettingsActionLabelR4D"
  ];

  for (const fn of oldFunctions) {
    current = removeFunctionByName(current, fn);
  }

  return current;
}

function ownerSource(scope) {
  const globalName = scope === "node" ? "KGW_NODE_SETTINGS_OWNER_V19" : "KGW_BRIDGE_SETTINGS_OWNER_V19";

  return [
    "// " + OWNER,
    "(function installKgwSettingsOwnerV19() {",
    "  \"use strict\";",
    "  const OWNER = \"" + OWNER + "\";",
    "  const SCOPE = \"" + scope + "\";",
    "  const GLOBAL_NAME = \"" + globalName + "\";",
    "  const FEEDBACK_MS = 10000;",
    "  const DISABLED_CLASS = \"" + DISABLED_CLASS + "\";",
    "  const ROOT_INSTALLED_ATTR = \"kgwSettingsOwnerV19\";",
    "  const locks = new WeakMap();",
    "  const timers = new WeakMap();",
    "",
    "  function lower(value) { return String(value || \"\").toLowerCase(); }",
    "",
    "  function trace(root, phase, details) {",
    "    try {",
    "      const payload = { owner: OWNER, scope: SCOPE, phase: phase, details: details || {} };",
    "      const tauri = window.__TAURI__;",
    "      if (tauri && tauri.core && typeof tauri.core.invoke === \"function\") {",
    "        tauri.core.invoke(\"kgw_frontend_button_trace_v1\", { payload: payload }).catch(function () {});",
    "      } else if (tauri && typeof tauri.invoke === \"function\") {",
    "        tauri.invoke(\"kgw_frontend_button_trace_v1\", { payload: payload }).catch(function () {});",
    "      } else {",
    "        console.debug(\"[\" + OWNER + \"]\", payload);",
    "      }",
    "    } catch (_) {}",
    "  }",
    "",
    "  function translate(key, fallback) {",
    "    try {",
    "      const candidates = [window.kgwT, window.__kgwT, window.t];",
    "      for (const fn of candidates) {",
    "        if (typeof fn === \"function\") {",
    "          const value = fn(key, fallback);",
    "          if (typeof value === \"string\" && value.trim() && value !== key) return value;",
    "        }",
    "      }",
    "      const apis = [window.kgwI18n, window.KGWI18n, window.KGW_I18N, window.i18n];",
    "      for (const api of apis) {",
    "        if (api && typeof api.t === \"function\") {",
    "          const value = api.t(key, fallback);",
    "          if (typeof value === \"string\" && value.trim() && value !== key) return value;",
    "        }",
    "        if (api && typeof api.translate === \"function\") {",
    "          const value = api.translate(key, fallback);",
    "          if (typeof value === \"string\" && value.trim() && value !== key) return value;",
    "        }",
    "      }",
    "    } catch (_) {}",
    "    return fallback;",
    "  }",
    "",
    "  function isSettingsControl(element) {",
    "    if (!element || !element.tagName) return false;",
    "    const tag = lower(element.tagName);",
    "    if (tag !== \"input\" && tag !== \"select\" && tag !== \"textarea\") return false;",
    "    const type = lower(element.type);",
    "    if (type === \"button\" || type === \"submit\" || type === \"reset\" || type === \"hidden\") return false;",
    "    if (element.closest && element.closest(\".logs, .log, [data-log], .kgw-log-pane\")) return false;",
    "    return true;",
    "  }",
    "",
    "  function isActionButton(element) {",
    "    if (!element || !element.tagName || lower(element.tagName) !== \"button\") return false;",
    "    const text = lower(element.textContent);",
    "    const action = lower((element.dataset && (element.dataset.kgwSettingsAction || element.dataset.action)) || element.getAttribute(\"data-action\") || element.getAttribute(\"aria-label\") || \"\");",
    "    return action.includes(\"save\") || action.includes(\"restore\") || action.includes(\"default\") || text.includes(\"save settings\") || text.includes(\"restore defaults\") || text.includes(\"set as defaults\") || text.includes(\"saved\") || text.includes(\"restored\") || text.includes(\"حفظ\") || text.includes(\"استعادة\") || text.includes(\"افتراض\");",
    "  }",
    "",
    "  function networkOf(element) {",
    "    let current = element;",
    "    while (current && current !== document) {",
    "      const dataset = current.dataset || {};",
    "      const direct = dataset.network || dataset.net || dataset.kgwNetwork || current.getAttribute(\"data-network\") || current.getAttribute(\"data-net\") || current.getAttribute(\"data-kgw-network\");",
    "      if (direct) return String(direct);",
    "      const id = lower(current.id);",
    "      const cls = lower(current.className);",
    "      if (id.includes(\"testnet12\") || cls.includes(\"testnet12\") || id.includes(\"tn12\") || cls.includes(\"tn12\")) return \"testnet12\";",
    "      if (id.includes(\"testnet10\") || cls.includes(\"testnet10\") || id.includes(\"tn10\") || cls.includes(\"tn10\")) return \"testnet10\";",
    "      if (id.includes(\"mainnet\") || cls.includes(\"mainnet\")) return \"mainnet\";",
    "      current = current.parentElement;",
    "    }",
    "    return \"mainnet\";",
    "  }",
    "",
    "  function actionName(button) {",
    "    const raw = lower((button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) || button.getAttribute(\"data-action\") || button.getAttribute(\"aria-label\") || button.textContent || \"\");",
    "    if (raw.includes(\"restore\") || raw.includes(\"استعادة\")) return \"restore\";",
    "    if (raw.includes(\"default\") || raw.includes(\"افتراض\")) return \"defaults\";",
    "    return \"save\";",
    "  }",
    "",
    "  function feedbackText(action) {",
    "    if (action === \"restore\") return translate(\"settings.feedback.restored\", \"Restored\");",
    "    if (action === \"defaults\") return translate(\"settings.feedback.setAsDefaults\", \"Set as defaults\");",
    "    return translate(\"settings.feedback.saved\", \"Saved\");",
    "  }",
    "",
    "  function fallbackText(action) {",
    "    if (action === \"restore\") return \"Restore Defaults\";",
    "    if (action === \"defaults\") return \"Set as Defaults\";",
    "    return \"Save Settings\";",
    "  }",
    "",
    "  function allButtons(root) {",
    "    return Array.from(root.querySelectorAll(\"button\")).filter(isActionButton);",
    "  }",
    "",
    "  function buttons(root, network) {",
    "    return allButtons(root).filter(function (button) {",
    "      return !network || network === \"all\" || networkOf(button) === network;",
    "    });",
    "  }",
    "",
    "  function setDisabled(root, network, disabled, reason) {",
    "    buttons(root, network || \"all\").forEach(function (button) {",
    "      button.disabled = !!disabled;",
    "      button.setAttribute(\"aria-disabled\", disabled ? \"true\" : \"false\");",
    "      button.classList.toggle(DISABLED_CLASS, !!disabled);",
    "      button.dataset.kgwSettingsOwnerV19Disabled = disabled ? \"true\" : \"false\";",
    "    });",
    "    trace(root, disabled ? \"v19-disabled\" : \"v19-enabled\", { network: network || \"all\", reason: reason || \"unspecified\" });",
    "  }",
    "",
    "  function lockMap(root) {",
    "    let map = locks.get(root);",
    "    if (!map) {",
    "      map = new Map();",
    "      locks.set(root, map);",
    "    }",
    "    return map;",
    "  }",
    "",
    "  function isLocked(root, network) {",
    "    return Date.now() < Number(lockMap(root).get(network) || 0);",
    "  }",
    "",
    "  function setLock(root, network) {",
    "    lockMap(root).set(network, Date.now() + FEEDBACK_MS);",
    "  }",
    "",
    "  function clearLock(root, network) {",
    "    lockMap(root).set(network, 0);",
    "  }",
    "",
    "  function rememberLabel(button, action) {",
    "    const current = String(button.textContent || \"\").trim();",
    "    const normalized = lower(current);",
    "    if (!button.dataset.kgwSettingsOwnerV19OriginalLabel || normalized === \"saved\" || normalized === \"restored\") {",
    "      button.dataset.kgwSettingsOwnerV19OriginalLabel = current && normalized !== \"saved\" && normalized !== \"restored\" ? current : fallbackText(action);",
    "    }",
    "  }",
    "",
    "  function restoreLabel(button, action) {",
    "    button.textContent = button.dataset.kgwSettingsOwnerV19OriginalLabel || fallbackText(action);",
    "  }",
    "",
    "  function startFeedback(root, network, button, action) {",
    "    setLock(root, network);",
    "    rememberLabel(button, action);",
    "    window.setTimeout(function () {",
    "      button.textContent = feedbackText(action);",
    "      setDisabled(root, network, true, \"feedback-start\");",
    "      const oldTimer = timers.get(button);",
    "      if (oldTimer) window.clearTimeout(oldTimer);",
    "      const timer = window.setTimeout(function () {",
    "        timers.delete(button);",
    "        clearLock(root, network);",
    "        restoreLabel(button, action);",
    "        setDisabled(root, network, true, \"feedback-complete\");",
    "        trace(root, \"v19-feedback-complete\", { network: network, action: action });",
    "      }, FEEDBACK_MS);",
    "      timers.set(button, timer);",
    "      trace(root, \"v19-feedback-start\", { network: network, action: action, holdMs: FEEDBACK_MS });",
    "    }, 0);",
    "  }",
    "",
    "  function install(root) {",
    "    if (!root || root.dataset[ROOT_INSTALLED_ATTR] === \"installed\") return;",
    "    root.dataset[ROOT_INSTALLED_ATTR] = \"installed\";",
    "    setDisabled(root, \"all\", true, \"initial\");",
    "",
    "    root.addEventListener(\"input\", function (event) {",
    "      if (!isSettingsControl(event.target)) return;",
    "      const network = networkOf(event.target);",
    "      if (isLocked(root, network)) {",
    "        setDisabled(root, network, true, \"input-locked\");",
    "        return;",
    "      }",
    "      if (!event.isTrusted) {",
    "        setDisabled(root, network, true, \"input-programmatic\");",
    "        return;",
    "      }",
    "      setDisabled(root, network, false, \"trusted-input\");",
    "    }, true);",
    "",
    "    root.addEventListener(\"change\", function (event) {",
    "      if (!isSettingsControl(event.target)) return;",
    "      const network = networkOf(event.target);",
    "      if (isLocked(root, network)) {",
    "        setDisabled(root, network, true, \"change-locked\");",
    "        return;",
    "      }",
    "      if (!event.isTrusted) {",
    "        setDisabled(root, network, true, \"change-programmatic\");",
    "        return;",
    "      }",
    "      setDisabled(root, network, false, \"trusted-change\");",
    "    }, true);",
    "",
    "    root.addEventListener(\"click\", function (event) {",
    "      const button = event.target && event.target.closest ? event.target.closest(\"button\") : null;",
    "      if (!button || !root.contains(button) || !isActionButton(button)) return;",
    "      const network = networkOf(button);",
    "      const action = actionName(button);",
    "      trace(root, \"v19-click\", { network: network, action: action, disabled: !!button.disabled, locked: isLocked(root, network), label: String(button.textContent || \"\").trim() });",
    "      if (isLocked(root, network) || button.dataset.kgwSettingsOwnerV19Disabled === \"true\") {",
    "        event.preventDefault();",
    "        event.stopImmediatePropagation();",
    "        setDisabled(root, network, true, \"click-blocked\");",
    "        return;",
    "      }",
    "      startFeedback(root, network, button, action);",
    "    }, true);",
    "",
    "    trace(root, \"v19-owner-installed\", { scope: SCOPE });",
    "  }",
    "",
    "  window[GLOBAL_NAME] = { install: install, setDisabled: setDisabled, buttons: buttons };",
    "  window.KGW_SETTINGS_OWNER_V19 = window[GLOBAL_NAME];",
    "})();",
    "// END_KGW_SETTINGS_OWNER_V19",
    ""
  ].join("\n");
}

function canonicalInstallActions(scope) {
  if (scope === "node") {
    return [
      "function installActions(root) {",
      "  if (window.KGW_NODE_SETTINGS_OWNER_V19 && typeof window.KGW_NODE_SETTINGS_OWNER_V19.install === \"function\") {",
      "    window.KGW_NODE_SETTINGS_OWNER_V19.install(root);",
      "  }",
      "",
      "  root.addEventListener(\"change\", () => {",
      "    updateAllCommands();",
      "  });",
      "}"
    ].join("\n");
  }

  return [
    "function installActions(root) {",
    "  if (window.KGW_BRIDGE_SETTINGS_OWNER_V19 && typeof window.KGW_BRIDGE_SETTINGS_OWNER_V19.install === \"function\") {",
    "    window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root);",
    "  }",
    "",
    "  root.addEventListener(\"change\", () => {",
    "    bridgeSyncAllModeControls();",
    "    updateAllCommands();",
    "  });",
    "}"
  ].join("\n");
}

function rewriteJs(text, scope) {
  let current = text.replace(/^\uFEFF/, "");
  current = removeKnownSettingOwnerLayers(current, scope);
  current = replaceFunctionByName(current, "installActions", canonicalInstallActions(scope));
  current = ownerSource(scope) + "\n" + current.trimStart();
  current = current.replace(/\n{4,}/g, "\n\n\n");
  return current;
}

function rewriteCss(text) {
  let current = text.replace(/^\uFEFF/, "");
  current = removeBlockByStartEnd(current, "/* KGW_SETTINGS_OWNER_V18_VISUAL */", "/* END_KGW_SETTINGS_OWNER_V18_VISUAL */");
  current = removeBlockByStartEnd(current, "/* KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START */", "/* KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_END */");

  const visual = [
    "",
    "/* " + VISUAL + " */",
    "button." + DISABLED_CLASS + ",",
    "button[data-kgw-settings-owner-v19-disabled=\"true\"],",
    "button:disabled." + DISABLED_CLASS + " {",
    "  opacity: 0.45;",
    "  filter: grayscale(0.35);",
    "  cursor: not-allowed;",
    "  box-shadow: none;",
    "  transform: none;",
    "}",
    "",
    "button[data-kgw-settings-owner-v19-disabled=\"false\"] {",
    "  opacity: 1;",
    "  filter: none;",
    "  cursor: pointer;",
    "}",
    "/* END_" + VISUAL + " */",
    ""
  ].join("\n");

  current = current.trimEnd() + "\n" + visual;
  current = current.replace(/\n{4,}/g, "\n\n\n");
  return current;
}

function gateSource() {
  return [
    "#!/usr/bin/env node",
    "\"use strict\";",
    "",
    "const fs = require(\"fs\");",
    "const path = require(\"path\");",
    "",
    "const repoRoot = process.argv[2] || process.cwd();",
    "const owner = \"KGW_SETTINGS_OWNER_V19\";",
    "",
    "const files = [",
    "  \"apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js\",",
    "  \"apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js\",",
    "  \"apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css\",",
    "  \"apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css\"",
    "];",
    "",
    "const forbidden = [",
    "  \"KGW_SETTINGS_OWNER_V18\",",
    "  \"KGW_SETTINGS_OWNER_V17\",",
    "  \"KGW_SETTINGS_OWNER_V16\",",
    "  \"KGW_SETTINGS_OWNER_FINAL_V15\",",
    "  \"KGW_SETTINGS_SINGLE_OWNER_R14\",",
    "  \"KGW_SETTINGS_CANONICAL_OWNER_R13\",",
    "  \"KGW_SETTINGS_DEEP_OWNER_TRACE_R12I\",",
    "  \"KGW_SETTINGS_SELECTION_TRACE_R12\",",
    "  \"KGW_SETTINGS_UNIFIED_OWNER_R12\",",
    "  \"KGW_NODE_SETTINGS_SIMPLE_UX_R7C\",",
    "  \"KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C\",",
    "  \"KGW_SETTINGS_OWNER_CONSOLIDATION_R9B\",",
    "  \"KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8\",",
    "  \"KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D\",",
    "  \"KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D\",",
    "  \"settings-buttons\",",
    "  \"nativeDisabledExpected\",",
    "  \"click-received\",",
    "  \"click-ignored-disabled\",",
    "  \"action-start\",",
    "  \"auto-baseline-before-input\",",
    "  \"kgwNodeSettingsChangedR7C\",",
    "  \"kgwBridgeSettingsChangedR7C\",",
    "  \"kgwNodeSettingsAfterActionR7C\",",
    "  \"kgwBridgeSettingsAfterActionR7C\",",
    "  \"kgwNodeSettingsIsFeedbackLockedR11\",",
    "  \"kgwBridgeSettingsIsFeedbackLockedR11\"",
    "];",
    "",
    "const errors = [];",
    "",
    "function countText(text, needle) {",
    "  let count = 0;",
    "  let offset = 0;",
    "  while (true) {",
    "    const index = text.indexOf(needle, offset);",
    "    if (index < 0) break;",
    "    count += 1;",
    "    offset = index + needle.length;",
    "  }",
    "  return count;",
    "}",
    "",
    "for (const rel of files) {",
    "  const full = path.join(repoRoot, rel);",
    "  if (!fs.existsSync(full)) {",
    "    errors.push(\"Missing file: \" + rel);",
    "    continue;",
    "  }",
    "  const text = fs.readFileSync(full, \"utf8\");",
    "  for (const token of forbidden) {",
    "    if (text.includes(token)) errors.push(rel + \" contains forbidden old owner token: \" + token);",
    "  }",
    "}",
    "",
    "const nodeJs = fs.readFileSync(path.join(repoRoot, files[0]), \"utf8\");",
    "const bridgeJs = fs.readFileSync(path.join(repoRoot, files[1]), \"utf8\");",
    "const nodeCss = fs.readFileSync(path.join(repoRoot, files[2]), \"utf8\");",
    "const bridgeCss = fs.readFileSync(path.join(repoRoot, files[3]), \"utf8\");",
    "",
    "if (countText(nodeJs, owner) < 2) errors.push(\"Node JS owner V19 marker missing or incomplete.\");",
    "if (countText(bridgeJs, owner) < 2) errors.push(\"Bridge JS owner V19 marker missing or incomplete.\");",
    "if (!nodeCss.includes(\"KGW_SETTINGS_OWNER_V19_VISUAL\")) errors.push(\"Node CSS V19 visual marker missing.\");",
    "if (!bridgeCss.includes(\"KGW_SETTINGS_OWNER_V19_VISUAL\")) errors.push(\"Bridge CSS V19 visual marker missing.\");",
    "if (!nodeJs.includes(\"window.KGW_NODE_SETTINGS_OWNER_V19.install(root)\")) errors.push(\"Node installActions is not routed to the single V19 owner.\");",
    "if (!bridgeJs.includes(\"window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)\")) errors.push(\"Bridge installActions is not routed to the single V19 owner.\");",
    "",
    "if (errors.length) {",
    "  console.error(\"KGW global owner gate failed:\");",
    "  for (const error of errors) console.error(\"- \" + error);",
    "  process.exit(1);",
    "}",
    "",
    "console.log(\"KGW global owner gate passed: single settings owner V19 only.\");",
    ""
  ].join("\n");
}

function validateNoOldTokens(rel, text) {
  for (const token of oldForbiddenTokens) {
    if (text.includes(token)) {
      report.validationErrors.push(rel + " still contains forbidden token: " + token);
    }
  }
}

function rollback() {
  report.rollback = true;
  for (const item of report.backups) {
    fs.copyFileSync(item.backup, abs(item.rel));
  }
}

function finish(success, reason) {
  report.finishedAt = new Date().toISOString();
  report.success = success;
  report.reason = reason || "";
  saveReportFile("REPORT.json", JSON.stringify(report, null, 2));

  const md = [
    "# " + (success ? "SUCCESS" : "FAILED") + " - KGW Settings Single Owner Full File Rewrite V19",
    "",
    "- Repository: `" + repoRoot + "`",
    "- Report dir: `" + reportDir + "`",
    "- Backup dir: `" + backupDir + "`",
    "- Owner: `" + OWNER + "`",
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

  saveReportFile(success ? "REPORT_SUCCESS.md" : "REPORT_FAILED.md", md);
}

try {
  mkdirp(reportDir);
  mkdirp(backupDir);

  const allRel = [targetFiles.nodeJs, targetFiles.bridgeJs, targetFiles.nodeCss, targetFiles.bridgeCss, targetFiles.gate];

  for (const rel of allRel) {
    if (!fs.existsSync(abs(rel))) throw new Error("Missing required file: " + rel);
    copyForBackup(rel);
    saveBefore(rel, readFile(rel));
  }

  const nodeAfter = rewriteJs(readFile(targetFiles.nodeJs), "node");
  const bridgeAfter = rewriteJs(readFile(targetFiles.bridgeJs), "bridge");
  const nodeCssAfter = rewriteCss(readFile(targetFiles.nodeCss));
  const bridgeCssAfter = rewriteCss(readFile(targetFiles.bridgeCss));
  const gateAfter = gateSource();

  writeFile(targetFiles.nodeJs, nodeAfter);
  writeFile(targetFiles.bridgeJs, bridgeAfter);
  writeFile(targetFiles.nodeCss, nodeCssAfter);
  writeFile(targetFiles.bridgeCss, bridgeCssAfter);
  writeFile(targetFiles.gate, gateAfter);

  report.changedFiles.push(targetFiles.nodeJs, targetFiles.bridgeJs, targetFiles.nodeCss, targetFiles.bridgeCss, targetFiles.gate);

  saveAfter(targetFiles.nodeJs, nodeAfter);
  saveAfter(targetFiles.bridgeJs, bridgeAfter);
  saveAfter(targetFiles.nodeCss, nodeCssAfter);
  saveAfter(targetFiles.bridgeCss, bridgeCssAfter);
  saveAfter(targetFiles.gate, gateAfter);

  validateNoOldTokens(targetFiles.nodeJs, nodeAfter);
  validateNoOldTokens(targetFiles.bridgeJs, bridgeAfter);
  validateNoOldTokens(targetFiles.nodeCss, nodeCssAfter);
  validateNoOldTokens(targetFiles.bridgeCss, bridgeCssAfter);

  if (report.validationErrors.length) {
    throw new Error("Static validation failed before gates.");
  }

  const nodeCheck = run("node", ["--check", abs(targetFiles.nodeJs)]);
  if (nodeCheck.status !== 0) throw new Error("node --check failed for kaspa-node.js");

  const bridgeCheck = run("node", ["--check", abs(targetFiles.bridgeJs)]);
  if (bridgeCheck.status !== 0) throw new Error("node --check failed for kaspa-bridge.js");

  const gateCheck = run("node", [abs(targetFiles.gate), repoRoot]);
  if (gateCheck.status !== 0) throw new Error("kgw_global_owner_gate.cjs failed");

  const i18nContract = path.join(repoRoot, "tools", "kgw_i18n_contract_gate.cjs");
  if (fs.existsSync(i18nContract)) {
    const c = run("node", [i18nContract, repoRoot]);
    if (c.status !== 0) throw new Error("kgw_i18n_contract_gate.cjs failed");
  }

  const i18nCoverage = path.join(repoRoot, "tools", "kgw_i18n_locale_coverage_gate.cjs");
  if (fs.existsSync(i18nCoverage)) {
    const c = run("node", [i18nCoverage, repoRoot]);
    if (c.status !== 0) throw new Error("kgw_i18n_locale_coverage_gate.cjs failed");
  }

  const fmt = run("cargo", ["fmt", "--all"]);
  if (fmt.status !== 0) throw new Error("cargo fmt --all failed");

  const fmtCheck = run("cargo", ["fmt", "--all", "--", "--check"]);
  if (fmtCheck.status !== 0) throw new Error("cargo fmt --all -- --check failed");

  const cargoCheck = run("cargo", ["check", "-p", "kaspa-gateway-desktop", "--no-default-features", "--features", "official-kaspa-runtime-all rkstratum_cpu_miner"]);
  if (cargoCheck.status !== 0) throw new Error("cargo check failed");

  finish(true, "Single V19 settings owner rewrite completed successfully.");
  console.log("");
  console.log("# SUCCESS");
  console.log("Report: " + path.join(reportDir, "REPORT_SUCCESS.md"));
  console.log("Report JSON: " + path.join(reportDir, "REPORT.json"));
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
  console.error("Report JSON: " + path.join(reportDir, "REPORT.json"));
  console.error("Backup: " + backupDir);
  process.exit(1);
}
