const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_safe_single_owner_v16.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
};

const staleNeedles = [
  "KGW_SETTINGS_BUTTONS_R4B",
  "KGW_SETTINGS_BUTTONS_R4C",
  "KGW_SETTINGS_BUTTONS_R4D",
  "KGW_SETTINGS_BUTTONS_R6",
  "KGW_SETTINGS_BUTTONS_R7",
  "KGW_SETTINGS_BUTTONS_R7B",
  "KGW_SETTINGS_BUTTONS_R7C",
  "KGW_SETTINGS_BUTTONS_R8",
  "KGW_SETTINGS_BUTTONS_R9",
  "KGW_SETTINGS_BUTTONS_R9B",
  "KGW_SETTINGS_BUTTONS_R10",
  "KGW_SETTINGS_BUTTONS_R11",
  "KGW_SETTINGS_UNIFIED_OWNER_R12",
  "KGW_SETTINGS_UNIFIED_OWNER_R12B",
  "KGW_SETTINGS_UNIFIED_OWNER_R12C",
  "KGW_SETTINGS_UNIFIED_OWNER_R12D",
  "KGW_SETTINGS_UNIFIED_OWNER_R12E",
  "KGW_SETTINGS_SELECTION_TRACE_R12H",
  "KGW_SETTINGS_SELECTION_TRACE_R12H2",
  "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
  "KGW_SETTINGS_CANONICAL_OWNER_R13",
  "KGW_SETTINGS_CANONICAL_OWNER_R13B",
  "KGW_SETTINGS_CANONICAL_OWNER_R13C",
  "KGW_SETTINGS_SINGLE_OWNER_R14",
  "KGW_SETTINGS_SINGLE_OWNER_R14B",
  "KGW_SETTINGS_OWNER_FINAL_V15",
  "kgwNodeSettingsInstallInitialBaselineR7C",
  "kgwBridgeSettingsInstallInitialBaselineR7C",
  "kgwNodeSettingsTraceSuppressedDirtyR9B",
  "kgwBridgeSettingsTraceSuppressedDirtyR9B",
  "kgwNodeUpdateSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateSettingsDirtyButtonsR4D",
  "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateAllSettingsDirtyButtonsR4D"
];

const knownGeneratedFunctionNames = [
  "kgwNodeSettingsInstallInitialBaselineR7C",
  "kgwBridgeSettingsInstallInitialBaselineR7C",
  "kgwNodeSettingsTraceSuppressedDirtyR9B",
  "kgwBridgeSettingsTraceSuppressedDirtyR9B",
  "kgwNodeUpdateSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateSettingsDirtyButtonsR4D",
  "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateAllSettingsDirtyButtonsR4D"
];

const oldWindowOwners = [
  "KGW_SETTINGS_BUTTONS_R4D",
  "KGW_SETTINGS_BUTTONS_R7C",
  "KGW_SETTINGS_BUTTONS_R8",
  "KGW_SETTINGS_BUTTONS_R9B",
  "KGW_SETTINGS_BUTTONS_R10",
  "KGW_SETTINGS_BUTTONS_R11",
  "KGW_SETTINGS_UNIFIED_OWNER_R12",
  "KGW_SETTINGS_UNIFIED_OWNER_R12B",
  "KGW_SETTINGS_UNIFIED_OWNER_R12C",
  "KGW_SETTINGS_UNIFIED_OWNER_R12D",
  "KGW_SETTINGS_UNIFIED_OWNER_R12E",
  "KGW_SETTINGS_SELECTION_TRACE_R12H",
  "KGW_SETTINGS_SELECTION_TRACE_R12H2",
  "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
  "KGW_SETTINGS_CANONICAL_OWNER_R13",
  "KGW_SETTINGS_CANONICAL_OWNER_R13B",
  "KGW_SETTINGS_CANONICAL_OWNER_R13C",
  "KGW_SETTINGS_SINGLE_OWNER_R14",
  "KGW_SETTINGS_SINGLE_OWNER_R14B",
  "KGW_SETTINGS_OWNER_FINAL_V15",
  "KGW_SETTINGS_OWNER_V16"
];

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function writeJson(fileName, data) {
  fs.writeFileSync(path.join(reportDir, fileName), JSON.stringify(data, null, 2) + "\n", "utf8");
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function findMatchingBrace(text, openIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;

  for (let i = openIndex; i < text.length; i += 1) {
    const ch = text[i];
    const next = text[i + 1];

    if (lineComment) {
      if (ch === "\n") lineComment = false;
      continue;
    }

    if (blockComment) {
      if (ch === "*" && next === "/") {
        blockComment = false;
        i += 1;
      }
      continue;
    }

    if (quote) {
      if (escaped) {
        escaped = false;
        continue;
      }
      if (ch === "\\") {
        escaped = true;
        continue;
      }
      if (ch === quote) quote = null;
      continue;
    }

    if (ch === "/" && next === "/") {
      lineComment = true;
      i += 1;
      continue;
    }

    if (ch === "/" && next === "*") {
      blockComment = true;
      i += 1;
      continue;
    }

    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      continue;
    }

    if (ch === "{") depth += 1;
    if (ch === "}") depth -= 1;

    if (depth === 0) return i;
  }

  return -1;
}

function findFunction(text, functionName) {
  const re = new RegExp("function\\s+" + escapeRegExp(functionName) + "\\s*\\([^)]*\\)\\s*\\{");
  const match = text.match(re);
  if (!match || match.index === undefined) return null;

  const start = match.index;
  const open = text.indexOf("{", start);
  const close = findMatchingBrace(text, open);
  if (close < 0) return null;

  return {
    start,
    open,
    end: close + 1,
    source: text.slice(start, close + 1)
  };
}

function removeNamedFunction(text, name) {
  const re = new RegExp("function\\s+" + escapeRegExp(name) + "\\s*\\([^)]*\\)\\s*\\{", "g");
  let changed = false;
  let match;

  while ((match = re.exec(text)) !== null) {
    const start = match.index;
    const open = text.indexOf("{", start);
    const close = findMatchingBrace(text, open);
    if (close < 0) continue;

    let end = close + 1;
    const tail = text.slice(end, end + 20);
    const semi = tail.match(/^\s*;?/);
    if (semi) end += semi[0].length;

    text = text.slice(0, start) + "\n" + text.slice(end);
    changed = true;
    re.lastIndex = 0;
  }

  return { text, changed };
}

function removeGeneratedBlocks(text) {
  const patterns = [
    /\/\/\s*KGW_SETTINGS_OWNER_V16[\s\S]*?\/\/\s*END_KGW_SETTINGS_OWNER_V16\s*/g,
    /\/\/\s*KGW_SETTINGS_OWNER_FINAL_V15[\s\S]*?\/\/\s*END_KGW_SETTINGS_OWNER_FINAL_V15\s*/g,
    /\/\/\s*KGW_SETTINGS_SINGLE_OWNER_R14B[\s\S]*?\/\/\s*END_KGW_SETTINGS_SINGLE_OWNER_R14B\s*/g,
    /\/\/\s*KGW_SETTINGS_SINGLE_OWNER_R14[\s\S]*?\/\/\s*END_KGW_SETTINGS_SINGLE_OWNER_R14\s*/g,
    /\/\/\s*KGW_SETTINGS_CANONICAL_OWNER_R13C[\s\S]*?\/\/\s*END_KGW_SETTINGS_CANONICAL_OWNER_R13C\s*/g,
    /\/\/\s*KGW_SETTINGS_CANONICAL_OWNER_R13B[\s\S]*?\/\/\s*END_KGW_SETTINGS_CANONICAL_OWNER_R13B\s*/g,
    /\/\/\s*KGW_SETTINGS_CANONICAL_OWNER_R13(?![A-Z0-9_])[\s\S]*?\/\/\s*END_KGW_SETTINGS_CANONICAL_OWNER_R13(?![A-Z0-9_])\s*/g,
    /\/\/\s*KGW_SETTINGS_DEEP_OWNER_TRACE_R12I[\s\S]*?\/\/\s*END_KGW_SETTINGS_DEEP_OWNER_TRACE_R12I\s*/g,
    /\/\/\s*KGW_SETTINGS_SELECTION_TRACE_R12H2[\s\S]*?\/\/\s*END_KGW_SETTINGS_SELECTION_TRACE_R12H2\s*/g,
    /\/\/\s*KGW_SETTINGS_SELECTION_TRACE_R12H[\s\S]*?\/\/\s*END_KGW_SETTINGS_SELECTION_TRACE_R12H\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12E[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12E\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12D[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12D\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12C[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12C\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12B[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12B\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])\s*/g,
    /\/\*\s*KGW_SETTINGS_OWNER_V16_VISUAL[\s\S]*?END_KGW_SETTINGS_OWNER_V16_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_OWNER_FINAL_V15_VISUAL[\s\S]*?END_KGW_SETTINGS_OWNER_FINAL_V15_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_SINGLE_OWNER_R14B_VISUAL[\s\S]*?END_KGW_SETTINGS_SINGLE_OWNER_R14B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL[\s\S]*?END_KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)[\s\S]*?END_KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)\s*\*\//g,
    /\/\/\s*KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)[\s\S]*?\/\/\s*END_KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)\s*/g
  ];

  let changed = false;
  for (const pattern of patterns) {
    const before = text;
    text = text.replace(pattern, "");
    if (before !== text) changed = true;
  }

  return { text, changed };
}

function removeOldInstallCallsFromInstallActions(text) {
  const installActions = findFunction(text, "installActions");
  if (!installActions) throw new Error("installActions(root) not found");

  let body = installActions.source;
  const before = body;

  for (const owner of oldWindowOwners) {
    const escaped = escapeRegExp(owner);
    body = body.replace(new RegExp("if\\s*\\(\\s*window\\." + escaped + "[\\s\\S]*?" + escaped + "\\.install\\(root\\);\\s*\\}\\s*", "g"), "");
  }

  for (const fn of knownGeneratedFunctionNames) {
    const escaped = escapeRegExp(fn);
    body = body.replace(new RegExp("^.*\\b" + escaped + "\\s*\\([^\\n;]*\\)\\s*;?\\s*$\\n?", "gm"), "");
  }

  return {
    text: text.slice(0, installActions.start) + body + text.slice(installActions.end),
    changed: before !== body
  };
}

function cleanJs(text) {
  const removals = [];

  let result = removeGeneratedBlocks(text);
  text = result.text;
  if (result.changed) removals.push("generated-blocks");

  for (const fn of knownGeneratedFunctionNames) {
    result = removeNamedFunction(text, fn);
    text = result.text;
    if (result.changed) removals.push("function:" + fn);
  }

  result = removeOldInstallCallsFromInstallActions(text);
  text = result.text;
  if (result.changed) removals.push("installActions-old-calls");

  return { text, removals };
}

function buildOwner(scopeName) {
  return `
// KGW_SETTINGS_OWNER_V16
(function installKgwSettingsOwnerV16() {
  const OWNER = "KGW_SETTINGS_OWNER_V16";
  const SCOPE = "${scopeName}";
  const FEEDBACK_MS = 10000;
  const locks = new WeakMap();
  const timers = new WeakMap();

  function lower(value) {
    return String(value || "").toLowerCase();
  }

  function trace(root, phase, details) {
    try {
      const payload = { owner: OWNER, scope: SCOPE, phase, details: details || {} };
      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else if (window.__TAURI__ && typeof window.__TAURI__.invoke === "function") {
        window.__TAURI__.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else {
        console.debug("[KGW_SETTINGS_OWNER_V16]", payload);
      }
    } catch (_) {}
  }

  function translate(key, fallback) {
    try {
      const candidates = [window.kgwI18n, window.KGWI18n, window.i18n];
      for (const api of candidates) {
        if (api && typeof api.t === "function") {
          const value = api.t(key);
          if (typeof value === "string" && value.trim()) return value;
        }
        if (api && typeof api.translate === "function") {
          const value = api.translate(key);
          if (typeof value === "string" && value.trim()) return value;
        }
      }
      if (typeof window.t === "function") {
        const value = window.t(key);
        if (typeof value === "string" && value.trim()) return value;
      }
    } catch (_) {}
    return fallback;
  }

  function isControl(element) {
    if (!element || !element.tagName) return false;
    const tag = lower(element.tagName);
    if (tag !== "input" && tag !== "select" && tag !== "textarea") return false;
    const type = lower(element.type);
    return !(type === "button" || type === "submit" || type === "reset" || type === "hidden");
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

  function buttons(root, network) {
    return Array.from(root.querySelectorAll("button")).filter(isActionButton).filter(function (button) {
      return network === "unknown" || networkOf(button) === network;
    });
  }

  function setDisabled(root, network, disabled, reason) {
    buttons(root, network).forEach(function (button) {
      button.disabled = !!disabled;
      button.setAttribute("aria-disabled", disabled ? "true" : "false");
      button.classList.toggle("kgw-settings-action-disabled-v16", !!disabled);
      button.dataset.kgwV16Disabled = disabled ? "true" : "false";
    });

    trace(root, disabled ? "v16-disabled" : "v16-enabled", { network, reason });
  }

  function lockMap(root) {
    let map = locks.get(root);
    if (!map) {
      map = new Map();
      locks.set(root, map);
    }
    return map;
  }

  function isLocked(root, network) {
    return Date.now() < Number(lockMap(root).get(network) || 0);
  }

  function setLock(root, network) {
    lockMap(root).set(network, Date.now() + FEEDBACK_MS);
  }

  function clearLock(root, network) {
    lockMap(root).set(network, 0);
  }

  function rememberLabel(button, action) {
    const current = String(button.textContent || "").trim();
    const normalized = lower(current);
    if (!button.dataset.kgwV16OriginalLabel || normalized === "saved" || normalized === "restored") {
      button.dataset.kgwV16OriginalLabel = current && normalized !== "saved" && normalized !== "restored"
        ? current
        : fallbackText(action);
    }
  }

  function restoreLabel(button, action) {
    button.textContent = button.dataset.kgwV16OriginalLabel || fallbackText(action);
  }

  function startFeedback(root, network, button, action) {
    setLock(root, network);
    rememberLabel(button, action);

    setTimeout(function () {
      button.textContent = feedbackText(action);
      setDisabled(root, network, true, "feedback-start");

      const oldTimer = timers.get(button);
      if (oldTimer) clearTimeout(oldTimer);

      const timer = setTimeout(function () {
        timers.delete(button);
        clearLock(root, network);
        restoreLabel(button, action);
        setDisabled(root, network, true, "feedback-end");
        trace(root, "v16-feedback-complete", { network, action });
      }, FEEDBACK_MS);

      timers.set(button, timer);
      trace(root, "v16-feedback-start", { network, action, holdMs: FEEDBACK_MS });
    }, 0);
  }

  function install(root) {
    if (!root || root.dataset.kgwSettingsOwnerV16 === "installed") return;
    root.dataset.kgwSettingsOwnerV16 = "installed";

    ["mainnet", "testnet10", "testnet12", "unknown"].forEach(function (network) {
      setDisabled(root, network, true, "initial");
    });

    root.addEventListener("input", function (event) {
      if (!isControl(event.target)) return;
      const network = networkOf(event.target);

      if (isLocked(root, network)) {
        setDisabled(root, network, true, "input-locked");
        return;
      }

      if (!event.isTrusted) {
        setDisabled(root, network, true, "input-programmatic");
        return;
      }

      setDisabled(root, network, false, "trusted-input");
    }, true);

    root.addEventListener("change", function (event) {
      if (!isControl(event.target)) return;
      const network = networkOf(event.target);

      if (isLocked(root, network)) {
        setDisabled(root, network, true, "change-locked");
        return;
      }

      if (!event.isTrusted) {
        setDisabled(root, network, true, "change-programmatic");
        return;
      }

      setDisabled(root, network, false, "trusted-change");
    }, true);

    root.addEventListener("click", function (event) {
      const button = event.target && event.target.closest ? event.target.closest("button") : null;
      if (!button || !root.contains(button) || !isActionButton(button)) return;

      const network = networkOf(button);
      const action = actionName(button);

      trace(root, "v16-click", {
        network,
        action,
        disabled: !!button.disabled,
        locked: isLocked(root, network),
        label: String(button.textContent || "").trim()
      });

      if (isLocked(root, network) || button.dataset.kgwV16Disabled === "true") {
        event.preventDefault();
        event.stopImmediatePropagation();
        setDisabled(root, network, true, "click-blocked");
        return;
      }

      startFeedback(root, network, button, action);
    }, true);

    trace(root, "v16-owner-installed", { scope: SCOPE });
  }

  window.KGW_SETTINGS_OWNER_V16 = {
    install,
    setDisabled,
    buttons
  };
})();
// END_KGW_SETTINGS_OWNER_V16
`;
}

function patchJs(file, scopeName) {
  const before = read(file);
  const cleanup = cleanJs(before);
  let text = cleanup.text;

  const installActions = findFunction(text, "installActions");
  if (!installActions) throw new Error(scopeName + ": installActions(root) not found");

  let body = installActions.source;
  body = body.replace(/if\s*\(\s*window\.KGW_SETTINGS_OWNER_V16[\s\S]*?KGW_SETTINGS_OWNER_V16\.install\(root\);\s*\}\s*/g, "");

  const injection = `{
  if (window.KGW_SETTINGS_OWNER_V16 && typeof window.KGW_SETTINGS_OWNER_V16.install === "function") {
    window.KGW_SETTINGS_OWNER_V16.install(root);
  }`;

  body = body.replace("{", injection);
  text = text.slice(0, installActions.start) + body + text.slice(installActions.end);
  text = buildOwner(scopeName) + "\n" + text;

  write(file, text);

  return {
    changed: before !== text,
    removals: cleanup.removals,
    hasV16: text.includes("KGW_SETTINGS_OWNER_V16"),
    hasInstallCall: text.includes("KGW_SETTINGS_OWNER_V16.install(root);"),
    staleRemaining: staleNeedles.filter((needle) => text.includes(needle))
  };
}

function patchCss(file) {
  const before = read(file);

  let text = before.replace(/\/\*\s*KGW_SETTINGS_OWNER_V16_VISUAL[\s\S]*?END_KGW_SETTINGS_OWNER_V16_VISUAL\s*\*\//g, "");

  const oldVisuals = [
    "KGW_SETTINGS_OWNER_FINAL_V15_VISUAL",
    "KGW_SETTINGS_SINGLE_OWNER_R14B_VISUAL",
    "KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL",
    "KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL",
    "KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL",
    "KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL",
    "KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL",
    "KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL",
    "KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL",
    "KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL"
  ];

  for (const marker of oldVisuals) {
    const base = marker.replace("_VISUAL", "");
    text = text.replace(new RegExp("/\\*\\s*" + escapeRegExp(marker) + "[\\s\\S]*?END_" + escapeRegExp(marker) + "\\s*\\*/", "g"), "");
  }

  const block = `
/* KGW_SETTINGS_OWNER_V16_VISUAL */
button.kgw-settings-action-disabled-v16,
button[data-kgw-v16-disabled="true"],
button:disabled.kgw-settings-action-disabled-v16 {
  opacity: 0.45;
  filter: grayscale(0.35);
  cursor: not-allowed;
}

button[data-kgw-v16-disabled="false"] {
  opacity: 1;
  filter: none;
  cursor: pointer;
}
/* END_KGW_SETTINGS_OWNER_V16_VISUAL */
`;

  text = text.trimEnd() + "\n\n" + block + "\n";
  write(file, text);

  return {
    changed: before !== text,
    hasVisual: text.includes("KGW_SETTINGS_OWNER_V16_VISUAL"),
    hasClass: text.includes("kgw-settings-action-disabled-v16")
  };
}

function validateJs(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_OWNER_V16")) errors.push(label + ": missing V16 owner");
  if (!text.includes("KGW_SETTINGS_OWNER_V16.install(root);")) errors.push(label + ": missing install call");
  if (!text.includes("button.disabled = !!disabled")) errors.push(label + ": missing native disabled assignment");
  if (!text.includes("event.stopImmediatePropagation()")) errors.push(label + ": missing repeated-click blocker");
  if (!text.includes("setTimeout(function ()")) errors.push(label + ": missing deferred feedback");

  const remaining = staleNeedles.filter((needle) => text.includes(needle));
  if (remaining.length) errors.push(label + ": stale generated needles remain: " + remaining.join(", "));

  return errors;
}

function validateCss(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_OWNER_V16_VISUAL")) errors.push(label + ": missing visual marker");
  if (!text.includes("kgw-settings-action-disabled-v16")) errors.push(label + ": missing visual class");

  return errors;
}

function main() {
  const beforeAudit = {
    nodeStale: staleNeedles.filter((needle) => read(files.nodeJs).includes(needle)),
    bridgeStale: staleNeedles.filter((needle) => read(files.bridgeJs).includes(needle)),
    nodeHasInstallActions: !!findFunction(read(files.nodeJs), "installActions"),
    bridgeHasInstallActions: !!findFunction(read(files.bridgeJs), "installActions")
  };

  const nodeResult = patchJs(files.nodeJs, "node");
  const bridgeResult = patchJs(files.bridgeJs, "bridge");
  const nodeCssResult = patchCss(files.nodeCss);
  const bridgeCssResult = patchCss(files.bridgeCss);

  const validationErrors = [
    ...validateJs(files.nodeJs, "Node"),
    ...validateJs(files.bridgeJs, "Bridge"),
    ...validateCss(files.nodeCss, "Node CSS"),
    ...validateCss(files.bridgeCss, "Bridge CSS")
  ];

  const afterAudit = {
    validationErrors,
    nodeResult,
    bridgeResult,
    nodeCssResult,
    bridgeCssResult,
    nodeStaleRemaining: staleNeedles.filter((needle) => read(files.nodeJs).includes(needle)),
    bridgeStaleRemaining: staleNeedles.filter((needle) => read(files.bridgeJs).includes(needle))
  };

  writeJson("audit-before-v16.json", beforeAudit);
  writeJson("audit-after-v16.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("V16 validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# V16 safe single settings owner patch passed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
