const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_single_owner_final_r14.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
};

const staleTokens = [
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
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end",
  "feedback-lock-enforce",
  "auto-baseline-before-input",
  "nativeDisabledExpected",
  "click-received",
  "click-ignored-disabled",
  "action-start",
  "settings-buttons",
  "kgwNodeSettingsInstallInitialBaselineR7C",
  "kgwBridgeSettingsInstallInitialBaselineR7C",
  "kgwNodeSettingsTraceSuppressedDirtyR9B",
  "kgwBridgeSettingsTraceSuppressedDirtyR9B",
  "kgwNodeUpdateSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateSettingsDirtyButtonsR4D",
  "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateAllSettingsDirtyButtonsR4D"
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

  for (let i = openIndex; i < text.length; i++) {
    const ch = text[i];
    const next = text[i + 1];

    if (lineComment) {
      if (ch === "\n") lineComment = false;
      continue;
    }

    if (blockComment) {
      if (ch === "*" && next === "/") {
        blockComment = false;
        i++;
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

      if (ch === quote) {
        quote = null;
      }

      continue;
    }

    if (ch === "/" && next === "/") {
      lineComment = true;
      i++;
      continue;
    }

    if (ch === "/" && next === "*") {
      blockComment = true;
      i++;
      continue;
    }

    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      continue;
    }

    if (ch === "{") depth++;
    if (ch === "}") depth--;

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

function removeRange(text, start, end) {
  return text.slice(0, start) + "\n" + text.slice(end);
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

    text = removeRange(text, start, end);
    changed = true;
    re.lastIndex = 0;
  }

  return { text, changed };
}

function findEnclosingFunctionRange(text, tokenIndex) {
  const patterns = [
    /function\s+[A-Za-z0-9_$]+\s*\([^)]*\)\s*\{/g,
    /\(\s*function\s+[A-Za-z0-9_$]*\s*\([^)]*\)\s*\{/g,
    /[A-Za-z0-9_$]+\s*=\s*function\s*\([^)]*\)\s*\{/g,
    /[A-Za-z0-9_$]+\s*:\s*function\s*\([^)]*\)\s*\{/g
  ];

  const ranges = [];

  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(text)) !== null) {
      const start = match.index;
      const open = text.indexOf("{", start);
      const close = findMatchingBrace(text, open);
      if (close < 0) continue;

      let end = close + 1;
      const tail = text.slice(end, end + 30);
      const call = tail.match(/^\s*\)\s*\(\s*\)\s*;?/);
      if (call) end += call[0].length;

      if (start <= tokenIndex && end >= tokenIndex) {
        const source = text.slice(start, end);
        if (!/function\s+installActions\s*\(/.test(source)) {
          ranges.push({ start, end, size: end - start });
        }
      }
    }
  }

  ranges.sort((a, b) => a.size - b.size);
  return ranges[0] || null;
}

function removeLineContaining(text, token) {
  const before = text;
  const pattern = new RegExp("^.*" + escapeRegExp(token) + ".*$\\n?", "gm");
  text = text.replace(pattern, "");
  return { text, changed: before !== text };
}

function removeOldBlocks(text) {
  const patterns = [
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

  for (const pattern of patterns) {
    text = text.replace(pattern, "");
  }

  return text;
}

function deepCleanOldOwners(text) {
  let changed = false;
  const beforeBlocks = text;
  text = removeOldBlocks(text);
  if (beforeBlocks !== text) changed = true;

  const explicitOldFunctions = [
    "kgwNodeSettingsInstallInitialBaselineR7C",
    "kgwBridgeSettingsInstallInitialBaselineR7C",
    "kgwNodeSettingsTraceSuppressedDirtyR9B",
    "kgwBridgeSettingsTraceSuppressedDirtyR9B",
    "kgwNodeUpdateSettingsDirtyButtonsR4D",
    "kgwBridgeUpdateSettingsDirtyButtonsR4D",
    "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
    "kgwBridgeUpdateAllSettingsDirtyButtonsR4D"
  ];

  for (const name of explicitOldFunctions) {
    const fn = removeNamedFunction(text, name);
    text = fn.text;
    if (fn.changed) changed = true;
  }

  let guard = 0;
  let loopChanged = true;
  const removals = [];

  while (loopChanged && guard < 300) {
    loopChanged = false;
    guard++;

    for (const token of staleTokens) {
      const index = text.indexOf(token);
      if (index < 0) continue;

      const range = findEnclosingFunctionRange(text, index);
      if (range && range.size < 120000) {
        text = removeRange(text, range.start, range.end);
        removals.push({ token, strategy: "remove-enclosing-function", size: range.size });
      } else {
        const result = removeLineContaining(text, token);
        text = result.text;
        removals.push({ token, strategy: "remove-line" });
      }

      loopChanged = true;
      changed = true;
      break;
    }
  }

  return { text, changed, removals };
}

function buildOwner(scopeName) {
  return `
// KGW_SETTINGS_SINGLE_OWNER_R14
(function installKgwSettingsSingleOwnerR14() {
  const OWNER = "KGW_SETTINGS_SINGLE_OWNER_R14";
  const SCOPE = "${scopeName}";
  const FEEDBACK_MS = 10000;
  const lockUntilByRoot = new WeakMap();
  const feedbackTimers = new WeakMap();

  function nowIso() {
    try { return new Date().toISOString(); } catch (_) { return ""; }
  }

  function lower(value) {
    return String(value || "").toLowerCase();
  }

  function trace(root, phase, details) {
    try {
      const payload = {
        owner: OWNER,
        scope: SCOPE,
        phase,
        details: Object.assign({ at: nowIso() }, details || {})
      };

      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else if (window.__TAURI__ && typeof window.__TAURI__.invoke === "function") {
        window.__TAURI__.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else {
        console.debug("[KGW_SETTINGS_SINGLE_OWNER_R14]", payload);
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

  function actionName(button) {
    const raw = lower(
      (button && button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
      (button && button.getAttribute && button.getAttribute("data-action")) ||
      (button && button.getAttribute && button.getAttribute("aria-label")) ||
      (button && button.textContent) ||
      ""
    );

    if (raw.includes("restore") || raw.includes("استعادة")) return "restore-defaults";
    if (raw.includes("default") || raw.includes("افتراض")) return "set-defaults";
    return "save-settings";
  }

  function feedbackText(action) {
    if (action === "restore-defaults") return translate("settings.feedback.restored", "Restored");
    if (action === "set-defaults") return translate("settings.feedback.setAsDefaults", "Set as defaults");
    return translate("settings.feedback.saved", "Saved");
  }

  function fallbackLabel(action) {
    if (action === "restore-defaults") return "Restore Defaults";
    if (action === "set-defaults") return "Set as Defaults";
    return "Save Settings";
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

    const local = lower((element && (element.name || element.id || element.className)) || "");
    if (local.includes("testnet12") || local.includes("tn12")) return "testnet12";
    if (local.includes("testnet10") || local.includes("tn10")) return "testnet10";
    if (local.includes("mainnet")) return "mainnet";

    return "mainnet";
  }

  function buttons(root, network) {
    return Array.from((root || document).querySelectorAll("button"))
      .filter(isActionButton)
      .filter(function (button) {
        return networkOf(button) === network || networkOf(button) === "mainnet" || network === "unknown";
      });
  }

  function describeButtons(root, network) {
    return buttons(root, network).map(function (button) {
      return {
        action: actionName(button),
        text: String(button.textContent || "").trim(),
        disabled: !!button.disabled,
        ariaDisabled: button.getAttribute("aria-disabled") || "",
        r14Disabled: button.dataset ? button.dataset.kgwR14Disabled || "" : ""
      };
    });
  }

  function setButtonsDisabled(root, network, disabled, reason) {
    buttons(root, network).forEach(function (button) {
      button.disabled = !!disabled;
      button.setAttribute("aria-disabled", disabled ? "true" : "false");
      button.classList.toggle("kgw-settings-action-disabled-r14", !!disabled);
      button.dataset.kgwR14Disabled = disabled ? "true" : "false";
      button.dataset.kgwSettingsActionDisabled = disabled ? "1" : "0";
    });

    trace(root, disabled ? "r14-buttons-disabled" : "r14-buttons-enabled", {
      network,
      reason,
      buttons: describeButtons(root, network)
    });
  }

  function lockMap(root) {
    let map = lockUntilByRoot.get(root);
    if (!map) {
      map = new Map();
      lockUntilByRoot.set(root, map);
    }
    return map;
  }

  function isLocked(root, network) {
    const map = lockMap(root);
    return Date.now() < Number(map.get(network) || 0);
  }

  function setLock(root, network, ms) {
    lockMap(root).set(network, Date.now() + ms);
  }

  function clearLock(root, network) {
    lockMap(root).set(network, 0);
  }

  function rememberOriginalLabel(button, action) {
    const current = String(button.textContent || "").trim();
    const lowerCurrent = lower(current);

    if (
      !button.dataset.kgwR14OriginalLabel ||
      lowerCurrent === "saved" ||
      lowerCurrent === "restored" ||
      lowerCurrent.includes("تم الحفظ") ||
      lowerCurrent.includes("تمت الاستعادة")
    ) {
      button.dataset.kgwR14OriginalLabel = current && lowerCurrent !== "saved" && lowerCurrent !== "restored"
        ? current
        : fallbackLabel(action);
    }
  }

  function restoreOriginalLabel(button, action) {
    button.textContent = button.dataset.kgwR14OriginalLabel || fallbackLabel(action);
  }

  function startFeedbackAfterOriginalClick(root, network, button, action) {
    setLock(root, network, FEEDBACK_MS);
    rememberOriginalLabel(button, action);

    setTimeout(function () {
      button.textContent = feedbackText(action);
      setButtonsDisabled(root, network, true, "r14-action-feedback-start");

      const oldTimer = feedbackTimers.get(button);
      if (oldTimer) clearTimeout(oldTimer);

      const timer = setTimeout(function () {
        feedbackTimers.delete(button);
        clearLock(root, network);
        restoreOriginalLabel(button, action);
        setButtonsDisabled(root, network, true, "r14-action-feedback-end");

        trace(root, "r14-action-feedback-complete", {
          network,
          action,
          buttons: describeButtons(root, network)
        });
      }, FEEDBACK_MS);

      feedbackTimers.set(button, timer);

      trace(root, "r14-action-feedback-start", {
        network,
        action,
        holdMs: FEEDBACK_MS,
        buttons: describeButtons(root, network)
      });
    }, 0);
  }

  function install(root) {
    if (!root || root.dataset.kgwSettingsSingleOwnerR14 === "installed") return;
    root.dataset.kgwSettingsSingleOwnerR14 = "installed";

    ["mainnet", "testnet10", "testnet12", "unknown"].forEach(function (network) {
      setButtonsDisabled(root, network, true, "r14-initial-install");
    });

    root.addEventListener("input", function (event) {
      const target = event.target;
      if (!isControl(target)) return;

      const network = networkOf(target);

      if (isLocked(root, network)) {
        setButtonsDisabled(root, network, true, "r14-ignore-input-locked");
        trace(root, "r14-setting-input-ignored-locked", {
          network,
          trusted: !!event.isTrusted
        });
        return;
      }

      if (!event.isTrusted) {
        setButtonsDisabled(root, network, true, "r14-ignore-input-programmatic");
        return;
      }

      setButtonsDisabled(root, network, false, "r14-trusted-input");
    }, true);

    root.addEventListener("change", function (event) {
      const target = event.target;
      if (!isControl(target)) return;

      const network = networkOf(target);

      if (isLocked(root, network)) {
        setButtonsDisabled(root, network, true, "r14-ignore-change-locked");
        trace(root, "r14-setting-change-ignored-locked", {
          network,
          trusted: !!event.isTrusted
        });
        return;
      }

      if (!event.isTrusted) {
        setButtonsDisabled(root, network, true, "r14-ignore-change-programmatic");
        return;
      }

      setButtonsDisabled(root, network, false, "r14-trusted-change");
    }, true);

    root.addEventListener("click", function (event) {
      const button = event.target && event.target.closest ? event.target.closest("button") : null;
      if (!button || !root.contains(button) || !isActionButton(button)) return;

      const network = networkOf(button);
      const action = actionName(button);

      trace(root, "r14-action-click-capture", {
        network,
        action,
        disabled: !!button.disabled,
        locked: isLocked(root, network),
        text: String(button.textContent || "").trim(),
        buttons: describeButtons(root, network)
      });

      if (isLocked(root, network) || button.dataset.kgwR14Disabled === "true") {
        event.preventDefault();
        event.stopImmediatePropagation();
        setButtonsDisabled(root, network, true, "r14-block-click-locked");
        return;
      }

      startFeedbackAfterOriginalClick(root, network, button, action);
    }, true);

    trace(root, "r14-owner-installed", {
      rootId: root.id || "",
      rootClass: String(root.className || "").slice(0, 180),
      mainnetButtons: describeButtons(root, "mainnet")
    });
  }

  window.KGW_SETTINGS_SINGLE_OWNER_R14 = {
    install,
    setButtonsDisabled,
    buttons
  };
})();
// END_KGW_SETTINGS_SINGLE_OWNER_R14
`;
}

function patchJs(file, scopeName) {
  const before = read(file);
  const cleanup = deepCleanOldOwners(before);
  let text = cleanup.text;

  const installActions = findFunction(text, "installActions");
  if (!installActions) {
    throw new Error(scopeName + ": installActions(root) not found");
  }

  let body = installActions.source;
  body = body.replace(/if\s*\(\s*window\.KGW_SETTINGS_SINGLE_OWNER_R14[\s\S]*?KGW_SETTINGS_SINGLE_OWNER_R14\.install\(root\);\s*\}\s*/g, "");

  const injection = `{
  if (window.KGW_SETTINGS_SINGLE_OWNER_R14 && typeof window.KGW_SETTINGS_SINGLE_OWNER_R14.install === "function") {
    window.KGW_SETTINGS_SINGLE_OWNER_R14.install(root);
  }`;

  body = body.replace("{", injection);
  text = text.slice(0, installActions.start) + body + text.slice(installActions.end);
  text = buildOwner(scopeName) + "\n" + text;

  write(file, text);

  return {
    changed: before !== text,
    cleanupChanged: cleanup.changed,
    removals: cleanup.removals,
    hasR14: text.includes("KGW_SETTINGS_SINGLE_OWNER_R14"),
    hasInstallCall: text.includes("KGW_SETTINGS_SINGLE_OWNER_R14.install(root);"),
    staleTokensRemaining: staleTokens.filter((token) => text.includes(token))
  };
}

function patchCss(file) {
  const before = read(file);

  let text = before
    .replace(/\/\*\s*KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL[\s\S]*?END_KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g, "");

  const block = `
/* KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL */
button.kgw-settings-action-disabled-r14,
button[data-kgw-r14-disabled="true"],
button:disabled.kgw-settings-action-disabled-r14 {
  opacity: 0.45;
  filter: grayscale(0.35);
  cursor: not-allowed;
}

button[data-kgw-r14-disabled="false"] {
  opacity: 1;
  filter: none;
  cursor: pointer;
}
/* END_KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL */
`;

  text = text.trimEnd() + "\n\n" + block + "\n";
  write(file, text);

  return {
    changed: before !== text,
    hasVisual: text.includes("KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL"),
    hasClass: text.includes("kgw-settings-action-disabled-r14")
  };
}

function validateJs(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_SINGLE_OWNER_R14")) {
    errors.push(label + ": missing R14 owner");
  }

  if (!text.includes("KGW_SETTINGS_SINGLE_OWNER_R14.install(root);")) {
    errors.push(label + ": missing installActions R14 install call");
  }

  if (!text.includes("button.disabled = !!disabled")) {
    errors.push(label + ": missing native disabled owner");
  }

  if (!text.includes("setTimeout(function ()")) {
    errors.push(label + ": missing deferred feedback lock");
  }

  if (!text.includes("event.stopImmediatePropagation()")) {
    errors.push(label + ": missing repeated click blocker");
  }

  const remaining = staleTokens.filter((token) => text.includes(token));
  if (remaining.length) {
    errors.push(label + ": stale tokens remaining: " + remaining.join(", "));
  }

  return errors;
}

function validateCss(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_SINGLE_OWNER_R14_VISUAL")) {
    errors.push(label + ": missing R14 visual marker");
  }

  if (!text.includes("kgw-settings-action-disabled-r14")) {
    errors.push(label + ": missing R14 disabled class");
  }

  return errors;
}

function main() {
  const beforeAudit = {
    nodeStaleTokens: staleTokens.filter((token) => read(files.nodeJs).includes(token)),
    bridgeStaleTokens: staleTokens.filter((token) => read(files.bridgeJs).includes(token)),
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
    nodeStaleTokensRemaining: staleTokens.filter((token) => read(files.nodeJs).includes(token)),
    bridgeStaleTokensRemaining: staleTokens.filter((token) => read(files.bridgeJs).includes(token))
  };

  writeJson("audit-before-r14.json", beforeAudit);
  writeJson("audit-after-r14.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R14 validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R14 single settings button owner rebuild passed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
