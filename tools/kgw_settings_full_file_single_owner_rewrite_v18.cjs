const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_full_file_single_owner_rewrite_v18.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"),
  globalGate: path.join(repoRoot, "tools/kgw_global_owner_gate.cjs")
};

const OLD_OWNER_MARKERS = [
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
  "KGW_SETTINGS_OWNER_V16",
  "KGW_SETTINGS_OWNER_V17"
];

const OLD_RUNTIME_MARKERS = [
  "settings-buttons",
  "nativeDisabledExpected",
  "click-received",
  "click-ignored-disabled",
  "action-start",
  "auto-baseline-before-input",
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end",
  "feedback-lock-enforce"
];

const OLD_FUNCTION_NAMES = [
  "kgwNodeSettingsInstallInitialBaselineR7C",
  "kgwBridgeSettingsInstallInitialBaselineR7C",
  "kgwNodeSettingsTraceSuppressedDirtyR9B",
  "kgwBridgeSettingsTraceSuppressedDirtyR9B",
  "kgwNodeUpdateSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateSettingsDirtyButtonsR4D",
  "kgwNodeUpdateAllSettingsDirtyButtonsR4D",
  "kgwBridgeUpdateAllSettingsDirtyButtonsR4D",
  "kgwNodeSettingsSetButtonsEnabledR7C",
  "kgwBridgeSettingsSetButtonsEnabledR7C",
  "kgwNodeSettingsForceButtonsDisabledR11",
  "kgwBridgeSettingsForceButtonsDisabledR11"
];

const NEW_OWNER = "KGW_SETTINGS_OWNER_V18";
const NEW_VISUAL = "KGW_SETTINGS_OWNER_V18_VISUAL";

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function readText(file) {
  return fs.existsSync(file) ? fs.readFileSync(file, "utf8") : "";
}

function writeText(file, text) {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, text, "utf8");
}

function saveReportText(name, text) {
  writeText(path.join(reportDir, name), text);
}

function saveReportJson(name, data) {
  saveReportText(name, JSON.stringify(data, null, 2) + "\n");
}

function containsAny(text, needles) {
  return needles.filter((needle) => text.includes(needle));
}

function indexOfAny(text, needles, fromIndex) {
  let bestIndex = -1;
  let bestNeedle = "";
  for (const needle of needles) {
    const index = text.indexOf(needle, fromIndex);
    if (index >= 0 && (bestIndex < 0 || index < bestIndex)) {
      bestIndex = index;
      bestNeedle = needle;
    }
  }
  return { index: bestIndex, needle: bestNeedle };
}

function findMatchingBrace(text, openIndex) {
  let depth = 0;
  let quote = "";
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
      if (ch === quote) quote = "";
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

function findFunctionByName(text, functionName) {
  const needle = "function " + functionName;
  const start = text.indexOf(needle);
  if (start < 0) return null;

  const open = text.indexOf("{", start);
  if (open < 0) return null;

  const close = findMatchingBrace(text, open);
  if (close < 0) return null;

  return {
    type: "named-function",
    name: functionName,
    start,
    end: close + 1,
    source: text.slice(start, close + 1)
  };
}

function scanNamedFunctions(text) {
  const ranges = [];
  let index = 0;

  while (index < text.length) {
    const start = text.indexOf("function ", index);
    if (start < 0) break;

    const nameStart = start + "function ".length;
    let nameEnd = nameStart;

    while (nameEnd < text.length && /[A-Za-z0-9_$]/.test(text[nameEnd])) {
      nameEnd += 1;
    }

    const name = text.slice(nameStart, nameEnd);
    const paren = text.indexOf("(", nameEnd);
    const open = text.indexOf("{", paren);

    if (!name || paren < 0 || open < 0) {
      index = nameStart;
      continue;
    }

    const between = text.slice(nameEnd, open);
    if (!between.includes("(") || !between.includes(")")) {
      index = nameEnd;
      continue;
    }

    const close = findMatchingBrace(text, open);
    if (close < 0) {
      index = open + 1;
      continue;
    }

    ranges.push({
      type: "named-function",
      name,
      start,
      end: close + 1,
      source: text.slice(start, close + 1)
    });

    index = close + 1;
  }

  return ranges;
}

function scanIifes(text) {
  const starts = ["(function ", "(function(", "!function "];
  const ranges = [];

  for (const prefix of starts) {
    let index = 0;
    while (index < text.length) {
      const start = text.indexOf(prefix, index);
      if (start < 0) break;

      const open = text.indexOf("{", start);
      if (open < 0) {
        index = start + prefix.length;
        continue;
      }

      const close = findMatchingBrace(text, open);
      if (close < 0) {
        index = open + 1;
        continue;
      }

      let end = close + 1;
      const tail = text.slice(end, end + 80);
      const callEnd = tail.indexOf(";");
      if (tail.trimStart().startsWith(")(") && callEnd >= 0) {
        end += callEnd + 1;
      }

      ranges.push({
        type: "iife",
        name: "",
        start,
        end,
        source: text.slice(start, end)
      });

      index = end;
    }
  }

  return ranges;
}

function scanTopLevelStatementsInFunction(functionSource) {
  const open = functionSource.indexOf("{");
  const close = functionSource.lastIndexOf("}");
  const head = functionSource.slice(0, open + 1);
  const body = functionSource.slice(open + 1, close);
  const tail = functionSource.slice(close);

  const statements = [];
  let start = 0;
  let depthParen = 0;
  let depthBrace = 0;
  let depthBracket = 0;
  let quote = "";
  let escaped = false;
  let lineComment = false;
  let blockComment = false;

  for (let i = 0; i < body.length; i += 1) {
    const ch = body[i];
    const next = body[i + 1];

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
      if (ch === quote) quote = "";
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

    if (ch === "(") depthParen += 1;
    if (ch === ")") depthParen -= 1;
    if (ch === "{") depthBrace += 1;
    if (ch === "}") depthBrace -= 1;
    if (ch === "[") depthBracket += 1;
    if (ch === "]") depthBracket -= 1;

    if (ch === ";" && depthParen === 0 && depthBrace === 0 && depthBracket === 0) {
      statements.push(body.slice(start, i + 1));
      start = i + 1;
    }
  }

  const rest = body.slice(start);
  if (rest.trim()) statements.push(rest);

  return { head, statements, tail };
}

function removeRanges(text, ranges) {
  const sorted = ranges.slice().sort((a, b) => b.start - a.start);
  for (const range of sorted) {
    text = text.slice(0, range.start) + "\n" + text.slice(range.end);
  }
  return text;
}

function removeMarkedBlocks(text) {
  const removals = [];
  let changed = true;

  while (changed) {
    changed = false;

    const allBlockMarkers = [NEW_OWNER, ...OLD_OWNER_MARKERS];

    for (const marker of allBlockMarkers) {
      const startMarker = "// " + marker;
      const endMarker = "// END_" + marker;
      const start = text.indexOf(startMarker);
      if (start < 0) continue;

      const end = text.indexOf(endMarker, start);
      if (end < 0) {
        throw new Error("Found block start without end marker: " + marker);
      }

      const endAfter = end + endMarker.length;
      text = text.slice(0, start) + "\n" + text.slice(endAfter);
      removals.push({ strategy: "marked-js-block", marker });
      changed = true;
      break;
    }
  }

  return { text, removals };
}

function removeVisualBlocks(text) {
  const markers = [
    NEW_VISUAL,
    "KGW_SETTINGS_OWNER_V17_VISUAL",
    "KGW_SETTINGS_OWNER_V16_VISUAL",
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

  const removals = [];
  let changed = true;

  while (changed) {
    changed = false;

    for (const marker of markers) {
      const startMarker = "/* " + marker + " */";
      const endMarker = "/* END_" + marker + " */";
      const start = text.indexOf(startMarker);
      if (start < 0) continue;

      const end = text.indexOf(endMarker, start);
      if (end < 0) {
        throw new Error("Found visual block start without end marker: " + marker);
      }

      const endAfter = end + endMarker.length;
      text = text.slice(0, start) + "\n" + text.slice(endAfter);
      removals.push({ strategy: "visual-block", marker });
      changed = true;
      break;
    }
  }

  return { text, removals };
}

function removeOldOwnerFunctions(text) {
  const removals = [];
  const ranges = [];

  for (const fn of scanNamedFunctions(text)) {
    if (fn.name === "installActions") continue;

    const hasOldMarker = containsAny(fn.source, OLD_OWNER_MARKERS).length > 0;
    const hasOldRuntime = containsAny(fn.source, OLD_RUNTIME_MARKERS).length > 0;
    const isKnownOld = OLD_FUNCTION_NAMES.includes(fn.name);
    const nameLooksOldSettingsOwner =
      fn.name.includes("Settings") &&
      (
        fn.name.includes("Button") ||
        fn.name.includes("Dirty") ||
        fn.name.includes("Baseline") ||
        fn.name.includes("Trace") ||
        fn.name.includes("Disabled") ||
        fn.name.includes("Enabled")
      );

    if ((hasOldMarker || hasOldRuntime || isKnownOld || nameLooksOldSettingsOwner) && fn.source.length < 220000) {
      ranges.push(fn);
    }
  }

  for (const iife of scanIifes(text)) {
    const hasOldMarker = containsAny(iife.source, OLD_OWNER_MARKERS).length > 0;
    const hasOldRuntime = containsAny(iife.source, OLD_RUNTIME_MARKERS).length > 0;

    if ((hasOldMarker || hasOldRuntime) && iife.source.length < 220000) {
      ranges.push(iife);
    }
  }

  const unique = [];
  for (const item of ranges.sort((a, b) => a.start - b.start || b.source.length - a.source.length)) {
    const overlaps = unique.some((existing) => !(item.end <= existing.start || item.start >= existing.end));
    if (!overlaps) unique.push(item);
  }

  if (unique.length) {
    text = removeRanges(text, unique);
    for (const item of unique) {
      removals.push({
        strategy: item.type,
        name: item.name,
        size: item.source.length,
        oldOwnerMarkers: containsAny(item.source, OLD_OWNER_MARKERS),
        oldRuntimeMarkers: containsAny(item.source, OLD_RUNTIME_MARKERS)
      });
    }
  }

  return { text, removals };
}

function cleanInstallActions(text) {
  const fn = findFunctionByName(text, "installActions");
  if (!fn) throw new Error("installActions(root) not found");

  const parsed = scanTopLevelStatementsInFunction(fn.source);
  const removedStatements = [];
  const keptStatements = [];

  for (const statement of parsed.statements) {
    const hasOldOwner = containsAny(statement, OLD_OWNER_MARKERS).length > 0;
    const hasOldRuntime = containsAny(statement, OLD_RUNTIME_MARKERS).length > 0;
    const hasOldFunctionCall = OLD_FUNCTION_NAMES.some((name) => statement.includes(name + "("));
    const hasNewOwnerInstall = statement.includes(NEW_OWNER + ".install(root)");

    if (hasOldOwner || hasOldRuntime || hasOldFunctionCall || hasNewOwnerInstall) {
      removedStatements.push({
        size: statement.length,
        oldOwnerMarkers: containsAny(statement, OLD_OWNER_MARKERS),
        oldRuntimeMarkers: containsAny(statement, OLD_RUNTIME_MARKERS),
        oldFunctionCalls: OLD_FUNCTION_NAMES.filter((name) => statement.includes(name + "(")),
        hadNewOwnerInstall: hasNewOwnerInstall
      });
    } else {
      keptStatements.push(statement);
    }
  }

  const installStatement = `
  if (window.KGW_SETTINGS_OWNER_V18 && typeof window.KGW_SETTINGS_OWNER_V18.install === "function") {
    window.KGW_SETTINGS_OWNER_V18.install(root);
  }
`;

  const rebuiltSource = parsed.head + installStatement + keptStatements.join("") + parsed.tail;
  const rebuiltText = text.slice(0, fn.start) + rebuiltSource + text.slice(fn.end);

  return {
    text: rebuiltText,
    removals: removedStatements
  };
}

function buildOwner(scopeName) {
  return `
// KGW_SETTINGS_OWNER_V18
(function installKgwSettingsOwnerV18() {
  const OWNER = "KGW_SETTINGS_OWNER_V18";
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
        console.debug("[KGW_SETTINGS_OWNER_V18]", payload);
      }
    } catch (_) {}
  }

  function translate(key, fallback) {
    try {
      const apis = [window.kgwI18n, window.KGWI18n, window.i18n];
      for (const api of apis) {
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
      button.classList.toggle("kgw-settings-action-disabled-v18", !!disabled);
      button.dataset.kgwV18Disabled = disabled ? "true" : "false";
    });

    trace(root, disabled ? "v18-disabled" : "v18-enabled", { network, reason });
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
    if (!button.dataset.kgwV18OriginalLabel || normalized === "saved" || normalized === "restored") {
      button.dataset.kgwV18OriginalLabel = current && normalized !== "saved" && normalized !== "restored"
        ? current
        : fallbackText(action);
    }
  }

  function restoreLabel(button, action) {
    button.textContent = button.dataset.kgwV18OriginalLabel || fallbackText(action);
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
        trace(root, "v18-feedback-complete", { network, action });
      }, FEEDBACK_MS);

      timers.set(button, timer);
      trace(root, "v18-feedback-start", { network, action, holdMs: FEEDBACK_MS });
    }, 0);
  }

  function install(root) {
    if (!root || root.dataset.kgwSettingsOwnerV18 === "installed") return;
    root.dataset.kgwSettingsOwnerV18 = "installed";

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

      trace(root, "v18-click", {
        network,
        action,
        disabled: !!button.disabled,
        locked: isLocked(root, network),
        label: String(button.textContent || "").trim()
      });

      if (isLocked(root, network) || button.dataset.kgwV18Disabled === "true") {
        event.preventDefault();
        event.stopImmediatePropagation();
        setDisabled(root, network, true, "click-blocked");
        return;
      }

      startFeedback(root, network, button, action);
    }, true);

    trace(root, "v18-owner-installed", { scope: SCOPE });
  }

  window.KGW_SETTINGS_OWNER_V18 = {
    install,
    setDisabled,
    buttons
  };
})();
// END_KGW_SETTINGS_OWNER_V18
`;
}

function patchJs(file, scopeName) {
  const before = readText(file);
  const fullName = scopeName === "node" ? "full-before/kaspa-node.js" : "full-before/kaspa-bridge.js";
  saveReportText(fullName, before);

  let text = before;

  const blockRemoval = removeMarkedBlocks(text);
  text = blockRemoval.text;

  const functionRemoval = removeOldOwnerFunctions(text);
  text = functionRemoval.text;

  const installCleanup = cleanInstallActions(text);
  text = installCleanup.text;

  const owner = buildOwner(scopeName);
  text = owner + "\n" + text;

  const afterName = scopeName === "node" ? "full-after/kaspa-node.js" : "full-after/kaspa-bridge.js";
  saveReportText(afterName, text);

  writeText(file, text);

  return {
    changed: before !== text,
    markedBlockRemovals: blockRemoval.removals,
    functionRemovals: functionRemoval.removals,
    installActionsRemovals: installCleanup.removals,
    beforeLength: before.length,
    afterLength: text.length,
    oldOwnerRemaining: containsAny(text, OLD_OWNER_MARKERS),
    oldRuntimeRemaining: containsAny(text, OLD_RUNTIME_MARKERS),
    hasNewOwner: text.includes(NEW_OWNER),
    hasInstallCall: text.includes("KGW_SETTINGS_OWNER_V18.install(root);")
  };
}

function patchCss(file, scopeName) {
  const before = readText(file);
  saveReportText(scopeName === "node" ? "full-before/kaspa-node.css" : "full-before/kaspa-bridge.css", before);

  let text = before;
  const visualRemoval = removeVisualBlocks(text);
  text = visualRemoval.text;

  const block = `
/* KGW_SETTINGS_OWNER_V18_VISUAL */
button.kgw-settings-action-disabled-v18,
button[data-kgw-v18-disabled="true"],
button:disabled.kgw-settings-action-disabled-v18 {
  opacity: 0.45;
  filter: grayscale(0.35);
  cursor: not-allowed;
}

button[data-kgw-v18-disabled="false"] {
  opacity: 1;
  filter: none;
  cursor: pointer;
}
/* END_KGW_SETTINGS_OWNER_V18_VISUAL */
`;

  text = text.trimEnd() + "\n\n" + block + "\n";

  saveReportText(scopeName === "node" ? "full-after/kaspa-node.css" : "full-after/kaspa-bridge.css", text);
  writeText(file, text);

  return {
    changed: before !== text,
    visualRemovals: visualRemoval.removals,
    beforeLength: before.length,
    afterLength: text.length,
    hasVisual: text.includes(NEW_VISUAL)
  };
}

function buildGlobalOwnerGate() {
  return `#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2] || process.cwd();

const files = [
  path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
];

const required = [
  "KGW_SETTINGS_OWNER_V18",
  "KGW_SETTINGS_OWNER_V18_VISUAL"
];

const forbidden = [
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
  "KGW_SETTINGS_SELECTION_TRACE_R12H",
  "KGW_SETTINGS_SELECTION_TRACE_R12H2",
  "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
  "KGW_SETTINGS_CANONICAL_OWNER_R13",
  "KGW_SETTINGS_SINGLE_OWNER_R14",
  "KGW_SETTINGS_OWNER_FINAL_V15",
  "KGW_SETTINGS_OWNER_V16",
  "KGW_SETTINGS_OWNER_V17",
  "settings-buttons",
  "nativeDisabledExpected",
  "click-received",
  "click-ignored-disabled",
  "action-start",
  "auto-baseline-before-input"
];

function readText(file) {
  return fs.existsSync(file) ? fs.readFileSync(file, "utf8") : "";
}

function fail(message, details) {
  console.error("# KGW global owner gate failed");
  console.error(message);
  if (details) console.error(JSON.stringify(details, null, 2));
  process.exit(1);
}

const report = {
  repoRoot,
  files: [],
  failures: []
};

for (const file of files) {
  const text = readText(file);
  const item = {
    file,
    exists: fs.existsSync(file),
    requiredFound: required.filter((token) => text.includes(token)),
    forbiddenFound: forbidden.filter((token) => text.includes(token))
  };
  report.files.push(item);

  if (!item.exists) {
    report.failures.push({ file, reason: "missing file" });
    continue;
  }

  if (file.endsWith(".js") && !text.includes("KGW_SETTINGS_OWNER_V18")) {
    report.failures.push({ file, reason: "missing JS owner KGW_SETTINGS_OWNER_V18" });
  }

  if (file.endsWith(".css") && !text.includes("KGW_SETTINGS_OWNER_V18_VISUAL")) {
    report.failures.push({ file, reason: "missing CSS visual KGW_SETTINGS_OWNER_V18_VISUAL" });
  }

  if (item.forbiddenFound.length > 0) {
    report.failures.push({ file, reason: "forbidden owner/runtime markers found", markers: item.forbiddenFound });
  }
}

if (report.failures.length > 0) {
  fail("Owner gate found stale owners or missing required owner.", report);
}

console.log("# KGW global owner gate passed");
console.log(JSON.stringify(report, null, 2));
`;
}

function rewriteGlobalGate() {
  const before = readText(files.globalGate);
  saveReportText("full-before/kgw_global_owner_gate.cjs", before);

  const text = buildGlobalOwnerGate();
  writeText(files.globalGate, text);

  saveReportText("full-after/kgw_global_owner_gate.cjs", text);

  return {
    changed: before !== text,
    beforeLength: before.length,
    afterLength: text.length,
    hasNewGateOwner: text.includes(NEW_OWNER),
    hasBrokenRegexRisk: false
  };
}

function validateJsText(text, label) {
  const errors = [];

  if (!text.includes(NEW_OWNER)) errors.push(label + ": missing " + NEW_OWNER);
  if (!text.includes("KGW_SETTINGS_OWNER_V18.install(root);")) errors.push(label + ": missing installActions owner call");
  if (!text.includes("button.disabled = !!disabled")) errors.push(label + ": missing native disabled assignment");
  if (!text.includes("event.stopImmediatePropagation()")) errors.push(label + ": missing repeated click blocker");
  if (!text.includes("setTimeout(function ()")) errors.push(label + ": missing deferred feedback");

  const oldOwner = containsAny(text, OLD_OWNER_MARKERS);
  if (oldOwner.length) errors.push(label + ": old owner markers remain: " + oldOwner.join(", "));

  const oldRuntime = containsAny(text, OLD_RUNTIME_MARKERS);
  if (oldRuntime.length) errors.push(label + ": old runtime markers remain: " + oldRuntime.join(", "));

  return errors;
}

function validateCssText(text, label) {
  const errors = [];
  if (!text.includes(NEW_VISUAL)) errors.push(label + ": missing " + NEW_VISUAL);
  if (!text.includes("kgw-settings-action-disabled-v18")) errors.push(label + ": missing v18 disabled class");
  return errors;
}

function main() {
  const beforeAudit = {
    nodeOldOwner: containsAny(readText(files.nodeJs), OLD_OWNER_MARKERS),
    bridgeOldOwner: containsAny(readText(files.bridgeJs), OLD_OWNER_MARKERS),
    nodeOldRuntime: containsAny(readText(files.nodeJs), OLD_RUNTIME_MARKERS),
    bridgeOldRuntime: containsAny(readText(files.bridgeJs), OLD_RUNTIME_MARKERS),
    nodeHasInstallActions: !!findFunctionByName(readText(files.nodeJs), "installActions"),
    bridgeHasInstallActions: !!findFunctionByName(readText(files.bridgeJs), "installActions")
  };

  const nodeResult = patchJs(files.nodeJs, "node");
  const bridgeResult = patchJs(files.bridgeJs, "bridge");
  const nodeCssResult = patchCss(files.nodeCss, "node");
  const bridgeCssResult = patchCss(files.bridgeCss, "bridge");
  const gateResult = rewriteGlobalGate();

  const nodeAfter = readText(files.nodeJs);
  const bridgeAfter = readText(files.bridgeJs);
  const nodeCssAfter = readText(files.nodeCss);
  const bridgeCssAfter = readText(files.bridgeCss);

  const validationErrors = [
    ...validateJsText(nodeAfter, "Node"),
    ...validateJsText(bridgeAfter, "Bridge"),
    ...validateCssText(nodeCssAfter, "Node CSS"),
    ...validateCssText(bridgeCssAfter, "Bridge CSS")
  ];

  const afterAudit = {
    validationErrors,
    nodeResult,
    bridgeResult,
    nodeCssResult,
    bridgeCssResult,
    gateResult,
    nodeOldOwnerRemaining: containsAny(nodeAfter, OLD_OWNER_MARKERS),
    bridgeOldOwnerRemaining: containsAny(bridgeAfter, OLD_OWNER_MARKERS),
    nodeOldRuntimeRemaining: containsAny(nodeAfter, OLD_RUNTIME_MARKERS),
    bridgeOldRuntimeRemaining: containsAny(bridgeAfter, OLD_RUNTIME_MARKERS)
  };

  saveReportJson("audit-before-v18.json", beforeAudit);
  saveReportJson("audit-after-v18.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("V18 validation failed:\\n- " + validationErrors.join("\\n- "));
  }

  console.log("# V18 full-file single-owner rewrite passed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
