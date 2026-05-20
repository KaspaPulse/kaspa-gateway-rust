const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_canonical_owner_rebuild_r13c.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
};

const exactOldOwnerPatterns = [
  /KGW_SETTINGS_CANONICAL_OWNER_R13(?![A-Z0-9_])/,
  /KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])/
];

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
  "KGW_SETTINGS_UNIFIED_OWNER_R12B",
  "KGW_SETTINGS_UNIFIED_OWNER_R12C",
  "KGW_SETTINGS_UNIFIED_OWNER_R12D",
  "KGW_SETTINGS_UNIFIED_OWNER_R12E",
  "KGW_SETTINGS_SELECTION_TRACE_R12H",
  "KGW_SETTINGS_SELECTION_TRACE_R12H2",
  "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
  "KGW_SETTINGS_CANONICAL_OWNER_R13B",
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end",
  "feedback-lock-enforce",
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

function removeLinesContaining(text, token) {
  const before = text;
  const pattern = new RegExp("^.*" + escapeRegExp(token) + ".*$\\n?", "gm");
  text = text.replace(pattern, "");
  return { text, changed: before !== text };
}

function removeCallLines(text, token) {
  const before = text;
  const fullLine = new RegExp("^.*\\b" + escapeRegExp(token) + "\\b.*$\\n?", "gm");
  text = text.replace(fullLine, "");
  const inlineCall = new RegExp("\\s*\\b" + escapeRegExp(token) + "\\s*\\([^;]*\\)\\s*;?", "g");
  text = text.replace(inlineCall, "");
  return { text, changed: before !== text };
}

function removeOldBlocks(text) {
  const patterns = [
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

  const oldFunctionNames = [
    "kgwNodeSettingsInstallInitialBaselineR7C",
    "kgwBridgeSettingsInstallInitialBaselineR7C",
    "kgwNodeSettingsTraceSuppressedDirtyR9B",
    "kgwBridgeSettingsTraceSuppressedDirtyR9B",
    "kgwNodeUpdateSettingsDirtyButtonsR4D",
    "kgwBridgeUpdateSettingsDirtyButtonsR4D",
    "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
    "kgwBridgeUpdateAllSettingsDirtyButtonsR4D"
  ];

  for (const name of oldFunctionNames) {
    const fn = removeNamedFunction(text, name);
    text = fn.text;
    if (fn.changed) changed = true;

    const call = removeCallLines(text, name);
    text = call.text;
    if (call.changed) changed = true;
  }

  const oldWindowOwners = [
    "KGW_SETTINGS_CANONICAL_OWNER_R13B",
    "KGW_SETTINGS_CANONICAL_OWNER_R13",
    "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
    "KGW_SETTINGS_SELECTION_TRACE_R12H2",
    "KGW_SETTINGS_SELECTION_TRACE_R12H",
    "KGW_SETTINGS_UNIFIED_OWNER_R12E",
    "KGW_SETTINGS_UNIFIED_OWNER_R12D",
    "KGW_SETTINGS_UNIFIED_OWNER_R12C",
    "KGW_SETTINGS_UNIFIED_OWNER_R12B",
    "KGW_SETTINGS_UNIFIED_OWNER_R12"
  ];

  for (const token of oldWindowOwners) {
    const before = text;
    text = text.replace(new RegExp("if\\s*\\(\\s*window\\." + escapeRegExp(token) + "[\\s\\S]*?" + escapeRegExp(token) + "\\.install\\(root\\);\\s*\\}\\s*", "g"), "");
    if (before !== text) changed = true;
  }

  for (const token of staleTokens) {
    const result = removeCallLines(text, token);
    text = result.text;
    if (result.changed) changed = true;
  }

  return { text, changed };
}

function buildOwner(scopeName) {
  return `
// KGW_SETTINGS_CANONICAL_OWNER_R13C
(function installKgwSettingsCanonicalOwnerR13C() {
  const OWNER = "KGW_SETTINGS_CANONICAL_OWNER_R13C";
  const SCOPE = "${scopeName}";
  const FEEDBACK_MS = 10000;
  const baselines = new Map();
  const feedbackTimers = new WeakMap();
  const enforcementTimers = new WeakMap();

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
        console.debug("[KGW_SETTINGS_CANONICAL_OWNER_R13C]", payload);
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

  function guessNetwork(element) {
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

  function keyFor(element) {
    if (!element) return "";
    const dataset = element.dataset || {};
    return String(
      dataset.kgwSettingKey ||
      dataset.settingKey ||
      dataset.key ||
      dataset.option ||
      dataset.flag ||
      element.name ||
      element.id ||
      element.getAttribute("aria-label") ||
      element.getAttribute("placeholder") ||
      element.className ||
      element.tagName ||
      ""
    ).slice(0, 220);
  }

  function valueOf(element) {
    if (!element) return null;
    const tag = lower(element.tagName);
    const type = lower(element.type);
    if (tag === "input" && (type === "checkbox" || type === "radio")) return !!element.checked;
    return String(element.value || "");
  }

  function readField(element) {
    return {
      key: keyFor(element),
      id: element.id || "",
      name: element.name || "",
      type: lower(element.type),
      tag: lower(element.tagName),
      network: guessNetwork(element),
      value: valueOf(element),
      disabled: !!element.disabled,
      className: String(element.className || "").slice(0, 180)
    };
  }

  function fields(root, network) {
    return Array.from((root || document).querySelectorAll("input, select, textarea"))
      .filter(isControl)
      .map(readField)
      .filter(function (field) {
        return network === "unknown" || field.network === network || field.network === "unknown";
      });
  }

  function snapshot(root, network) {
    const compact = fields(root, network).map(function (field) {
      return {
        key: field.key,
        network: field.network,
        type: field.type,
        value: field.value
      };
    });
    return JSON.stringify(compact);
  }

  function snapshotHash(text) {
    let hash = 0;
    const input = String(text || "");
    for (let i = 0; i < input.length; i += 1) {
      hash = ((hash << 5) - hash + input.charCodeAt(i)) | 0;
    }
    return String(hash);
  }

  function baselineKey(network) {
    return SCOPE + ":" + network;
  }

  function currentBaseline(root, network) {
    return baselines.get(baselineKey(network)) || "";
  }

  function updateBaseline(root, network, reason) {
    const value = snapshot(root, network);
    baselines.set(baselineKey(network), value);
    trace(root, "r13c-baseline-updated", {
      network,
      reason,
      hash: snapshotHash(value)
    });
  }

  function hasRealDirty(root, network) {
    return snapshot(root, network) !== currentBaseline(root, network);
  }

  function buttons(root, network) {
    return Array.from((root || document).querySelectorAll("button"))
      .filter(isActionButton)
      .filter(function (button) {
        const buttonNetwork = guessNetwork(button);
        return network === "unknown" || buttonNetwork === network || buttonNetwork === "unknown";
      });
  }

  function describeButtons(root, network) {
    return buttons(root, network).map(function (button) {
      return {
        action: actionName(button),
        text: String(button.textContent || "").trim().slice(0, 100),
        disabled: !!button.disabled,
        ariaDisabled: button.getAttribute("aria-disabled") || "",
        r13cDisabled: button.dataset ? button.dataset.kgwR13cDisabled || "" : "",
        className: String(button.className || "").slice(0, 160)
      };
    });
  }

  function setDisabled(root, network, disabled, reason) {
    buttons(root, network).forEach(function (button) {
      button.disabled = !!disabled;
      button.setAttribute("aria-disabled", disabled ? "true" : "false");
      button.classList.toggle("kgw-settings-action-disabled-r13c", !!disabled);
      button.dataset.kgwR13cDisabled = disabled ? "true" : "false";
      button.dataset.kgwSettingsActionDisabled = disabled ? "1" : "0";
    });

    trace(root, disabled ? "r13c-buttons-disabled" : "r13c-buttons-enabled", {
      network,
      reason,
      buttons: describeButtons(root, network),
      dirty: hasRealDirty(root, network)
    });
  }

  function lockKey(network) {
    return "kgwR13cLockUntil_" + network;
  }

  function isLocked(root, network) {
    const until = Number(root.dataset[lockKey(network)] || "0");
    return Date.now() < until;
  }

  function rememberOriginalLabel(button) {
    if (!button.dataset.kgwR13cOriginalLabel) {
      button.dataset.kgwR13cOriginalLabel = button.textContent || "";
    }
  }

  function restoreOriginalLabel(button) {
    if (button.dataset.kgwR13cOriginalLabel) {
      button.textContent = button.dataset.kgwR13cOriginalLabel;
    }
  }

  function startActionLock(root, network, clickedButton, action) {
    const until = Date.now() + FEEDBACK_MS;
    root.dataset[lockKey(network)] = String(until);

    rememberOriginalLabel(clickedButton);
    clickedButton.textContent = feedbackText(action);

    setDisabled(root, network, true, "r13c-action-lock-start");

    const previousTimer = feedbackTimers.get(clickedButton);
    if (previousTimer) clearTimeout(previousTimer);

    const previousEnforcer = enforcementTimers.get(clickedButton);
    if (previousEnforcer) clearInterval(previousEnforcer);

    const enforcer = setInterval(function () {
      if (!isLocked(root, network)) {
        clearInterval(enforcer);
        enforcementTimers.delete(clickedButton);
        return;
      }
      setDisabled(root, network, true, "r13c-action-lock-enforce");
    }, 150);

    enforcementTimers.set(clickedButton, enforcer);

    const timer = setTimeout(function () {
      clearInterval(enforcer);
      enforcementTimers.delete(clickedButton);
      feedbackTimers.delete(clickedButton);

      restoreOriginalLabel(clickedButton);
      root.dataset[lockKey(network)] = "0";

      updateBaseline(root, network, "r13c-action-complete-" + action);
      setDisabled(root, network, true, "r13c-action-lock-end");

      trace(root, "r13c-action-complete", {
        network,
        action,
        buttons: describeButtons(root, network)
      });
    }, FEEDBACK_MS);

    feedbackTimers.set(clickedButton, timer);

    trace(root, "r13c-action-lock-start", {
      network,
      action,
      holdMs: FEEDBACK_MS,
      buttons: describeButtons(root, network)
    });
  }

  function install(root) {
    if (!root || root.dataset.kgwSettingsCanonicalOwnerR13c === "installed") return;
    root.dataset.kgwSettingsCanonicalOwnerR13c = "installed";

    ["mainnet", "testnet10", "testnet12", "unknown"].forEach(function (network) {
      updateBaseline(root, network, "r13c-initial-install");
      setDisabled(root, network, true, "r13c-initial-install");
    });

    root.addEventListener("input", function (event) {
      const target = event.target;
      if (!isControl(target)) return;

      const network = guessNetwork(target);

      if (isLocked(root, network)) {
        setDisabled(root, network, true, "r13c-ignore-input-locked");
        trace(root, "r13c-setting-input-ignored-locked", {
          network,
          isTrusted: !!event.isTrusted,
          field: readField(target)
        });
        return;
      }

      if (!event.isTrusted) {
        setDisabled(root, network, true, "r13c-ignore-input-programmatic");
        trace(root, "r13c-setting-input-ignored-programmatic", {
          network,
          field: readField(target)
        });
        return;
      }

      const dirty = hasRealDirty(root, network);

      trace(root, "r13c-setting-input", {
        network,
        field: readField(target),
        dirty,
        currentHash: snapshotHash(snapshot(root, network)),
        baselineHash: snapshotHash(currentBaseline(root, network))
      });

      setDisabled(root, network, !dirty, dirty ? "r13c-trusted-input-dirty" : "r13c-trusted-input-back-to-baseline");
    }, true);

    root.addEventListener("change", function (event) {
      const target = event.target;
      if (!isControl(target)) return;

      const network = guessNetwork(target);

      if (isLocked(root, network)) {
        setDisabled(root, network, true, "r13c-ignore-change-locked");
        trace(root, "r13c-setting-change-ignored-locked", {
          network,
          isTrusted: !!event.isTrusted,
          field: readField(target)
        });
        return;
      }

      if (!event.isTrusted) {
        setDisabled(root, network, true, "r13c-ignore-change-programmatic");
        trace(root, "r13c-setting-change-ignored-programmatic", {
          network,
          field: readField(target)
        });
        return;
      }

      const dirty = hasRealDirty(root, network);

      trace(root, "r13c-setting-change", {
        network,
        field: readField(target),
        dirty,
        currentHash: snapshotHash(snapshot(root, network)),
        baselineHash: snapshotHash(currentBaseline(root, network))
      });

      setDisabled(root, network, !dirty, dirty ? "r13c-trusted-change-dirty" : "r13c-trusted-change-back-to-baseline");
    }, true);

    root.addEventListener("click", function (event) {
      const button = event.target && event.target.closest ? event.target.closest("button") : null;
      if (!button || !root.contains(button) || !isActionButton(button)) return;

      const network = guessNetwork(button);
      const action = actionName(button);

      trace(root, "r13c-action-click-capture", {
        network,
        action,
        disabled: !!button.disabled,
        locked: isLocked(root, network),
        text: String(button.textContent || "").trim(),
        buttons: describeButtons(root, network),
        dirty: hasRealDirty(root, network)
      });

      if (button.disabled || isLocked(root, network)) {
        event.preventDefault();
        event.stopImmediatePropagation();

        setDisabled(root, network, true, "r13c-block-click-disabled-or-locked");

        trace(root, "r13c-action-click-blocked", {
          network,
          action,
          text: String(button.textContent || "").trim()
        });

        return;
      }

      startActionLock(root, network, button, action);

      setTimeout(function () {
        setDisabled(root, network, true, "r13c-post-action-enforce-0ms");
      }, 0);

      setTimeout(function () {
        setDisabled(root, network, true, "r13c-post-action-enforce-300ms");
      }, 300);

      setTimeout(function () {
        setDisabled(root, network, true, "r13c-post-action-enforce-1500ms");
      }, 1500);
    }, true);

    trace(root, "r13c-canonical-owner-installed", {
      rootId: root.id || "",
      rootClass: String(root.className || "").slice(0, 180),
      mainnetButtons: describeButtons(root, "mainnet"),
      testnet10Buttons: describeButtons(root, "testnet10"),
      testnet12Buttons: describeButtons(root, "testnet12")
    });
  }

  window.KGW_SETTINGS_CANONICAL_OWNER_R13C = {
    install,
    snapshot,
    buttons,
    setDisabled,
    updateBaseline
  };
})();
// END_KGW_SETTINGS_CANONICAL_OWNER_R13C
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

  body = body.replace(/if\s*\(\s*window\.KGW_SETTINGS_CANONICAL_OWNER_R13C[\s\S]*?KGW_SETTINGS_CANONICAL_OWNER_R13C\.install\(root\);\s*\}\s*/g, "");

  const injection = `{
  if (window.KGW_SETTINGS_CANONICAL_OWNER_R13C && typeof window.KGW_SETTINGS_CANONICAL_OWNER_R13C.install === "function") {
    window.KGW_SETTINGS_CANONICAL_OWNER_R13C.install(root);
  }`;

  body = body.replace("{", injection);
  text = text.slice(0, installActions.start) + body + text.slice(installActions.end);
  text = buildOwner(scopeName) + "\n" + text;

  write(file, text);

  return {
    changed: before !== text,
    cleanupChanged: cleanup.changed,
    hasR13C: text.includes("KGW_SETTINGS_CANONICAL_OWNER_R13C"),
    hasInstallCall: text.includes("KGW_SETTINGS_CANONICAL_OWNER_R13C.install(root);"),
    staleTokensRemaining: staleTokens.filter((token) => text.includes(token)),
    exactOldOwnerMatches: exactOldOwnerPatterns.filter((pattern) => pattern.test(text)).map(String)
  };
}

function patchCss(file) {
  const before = read(file);

  let text = before
    .replace(/\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13B_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL[\s\S]*?END_KGW_SETTINGS_CANONICAL_OWNER_R13_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g, "")
    .replace(/\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g, "");

  const block = `
/* KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL */
button.kgw-settings-action-disabled-r13c,
button[data-kgw-r13c-disabled="true"],
button:disabled.kgw-settings-action-disabled-r13c {
  opacity: 0.45;
  filter: grayscale(0.35);
  cursor: not-allowed;
  pointer-events: none;
}

button[data-kgw-r13c-disabled="false"] {
  opacity: 1;
  filter: none;
  cursor: pointer;
}
/* END_KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL */
`;

  text = text.trimEnd() + "\n\n" + block + "\n";
  write(file, text);

  return {
    changed: before !== text,
    hasVisual: text.includes("KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL"),
    hasClass: text.includes("kgw-settings-action-disabled-r13c")
  };
}

function validateJs(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_CANONICAL_OWNER_R13C")) {
    errors.push(label + ": missing R13C owner");
  }

  if (!text.includes("KGW_SETTINGS_CANONICAL_OWNER_R13C.install(root);")) {
    errors.push(label + ": missing installActions R13C install call");
  }

  if (!text.includes("button.disabled = !!disabled")) {
    errors.push(label + ": missing native button.disabled ownership");
  }

  if (!text.includes("event.stopImmediatePropagation()")) {
    errors.push(label + ": missing repeated-click blocker");
  }

  if (!text.includes("r13c-action-lock-enforce")) {
    errors.push(label + ": missing R13C lock enforcement");
  }

  const remaining = staleTokens.filter((token) => text.includes(token));
  if (remaining.length) {
    errors.push(label + ": stale tokens remaining: " + remaining.join(", "));
  }

  const exactOld = exactOldOwnerPatterns.filter((pattern) => pattern.test(text)).map(String);
  if (exactOld.length) {
    errors.push(label + ": exact old owner tokens remaining: " + exactOld.join(", "));
  }

  return errors;
}

function validateCss(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_CANONICAL_OWNER_R13C_VISUAL")) {
    errors.push(label + ": missing R13C visual marker");
  }

  if (!text.includes("kgw-settings-action-disabled-r13c")) {
    errors.push(label + ": missing R13C disabled class");
  }

  return errors;
}

function main() {
  const beforeAudit = {
    nodeStaleTokens: staleTokens.filter((token) => read(files.nodeJs).includes(token)),
    bridgeStaleTokens: staleTokens.filter((token) => read(files.bridgeJs).includes(token)),
    nodeExactOldOwnerMatches: exactOldOwnerPatterns.filter((pattern) => pattern.test(read(files.nodeJs))).map(String),
    bridgeExactOldOwnerMatches: exactOldOwnerPatterns.filter((pattern) => pattern.test(read(files.bridgeJs))).map(String),
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
    bridgeStaleTokensRemaining: staleTokens.filter((token) => read(files.bridgeJs).includes(token)),
    nodeExactOldOwnerMatches: exactOldOwnerPatterns.filter((pattern) => pattern.test(read(files.nodeJs))).map(String),
    bridgeExactOldOwnerMatches: exactOldOwnerPatterns.filter((pattern) => pattern.test(read(files.bridgeJs))).map(String)
  };

  writeJson("audit-before-r13c.json", beforeAudit);
  writeJson("audit-after-r13c.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R13C validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R13C canonical settings button owner rebuild passed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
