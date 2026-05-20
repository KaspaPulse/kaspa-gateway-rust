const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_unified_owner_rebuild_r12e.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"),
  tauriLib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
};

const staleRuntimeMarkers = [
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
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end"
];

const exactR12Pattern = /KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])/;

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function writeJson(fileName, data) {
  fs.writeFileSync(path.join(reportDir, fileName), JSON.stringify(data, null, 2) + "\n", "utf8");
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

    if (depth === 0) {
      return i;
    }
  }

  return -1;
}

function findFunction(text, functionName) {
  const re = new RegExp("function\\s+" + functionName + "\\s*\\([^)]*\\)\\s*\\{");
  const match = text.match(re);
  if (!match || match.index === undefined) return null;

  const start = match.index;
  const open = text.indexOf("{", start);
  const close = findMatchingBrace(text, open);

  if (close < 0) return null;

  return {
    start,
    end: close + 1,
    source: text.slice(start, close + 1)
  };
}

function collectFunctionRangesContaining(text, index) {
  const ranges = [];
  const patterns = [
    /function\s+[A-Za-z0-9_$]+\s*\([^)]*\)\s*\{/g,
    /function\s*\([^)]*\)\s*\{/g,
    /\(\s*function\s+[A-Za-z0-9_$]*\s*\([^)]*\)\s*\{/g,
    /[A-Za-z0-9_$]+\s*=\s*function\s*\([^)]*\)\s*\{/g,
    /[A-Za-z0-9_$]+\s*:\s*function\s*\([^)]*\)\s*\{/g
  ];

  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(text)) !== null) {
      const start = match.index;
      const open = text.indexOf("{", start);
      const close = findMatchingBrace(text, open);
      if (close < 0) continue;

      let end = close + 1;
      const tail = text.slice(end, end + 40);
      const call = tail.match(/^\s*\)\s*\(\s*\)\s*;?/);
      if (call) {
        end += call[0].length;
      } else {
        const semi = tail.match(/^\s*;?/);
        if (semi) end += semi[0].length;
      }

      if (start <= index && end >= index) {
        const source = text.slice(start, end);
        if (!/function\s+installActions\s*\(/.test(source)) {
          ranges.push({ start, end, size: end - start });
        }
      }
    }
  }

  ranges.sort((a, b) => a.size - b.size);
  return ranges;
}

function removeLineContaining(text, index) {
  const start = text.lastIndexOf("\n", index);
  const end = text.indexOf("\n", index);
  const lineStart = start < 0 ? 0 : start + 1;
  const lineEnd = end < 0 ? text.length : end + 1;
  return text.slice(0, lineStart) + text.slice(lineEnd);
}

function removeGeneratedBlocks(source) {
  let text = source;

  const blockPatterns = [
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12E[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12E\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12D[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12D\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12C[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12C\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12B[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12B\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])\s*/g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)[\s\S]*?END_KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)\s*\*\//g,
    /\/\/\s*KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)[\s\S]*?\/\/\s*END_KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)\s*/g
  ];

  for (const pattern of blockPatterns) {
    text = text.replace(pattern, "");
  }

  text = text.replace(/if\s*\(\s*window\.KGW_SETTINGS_UNIFIED_OWNER_R12D[\s\S]*?KGW_SETTINGS_UNIFIED_OWNER_R12D\.install\(root\);\s*\}\s*/g, "");
  text = text.replace(/if\s*\(\s*window\.KGW_SETTINGS_UNIFIED_OWNER_R12C[\s\S]*?KGW_SETTINGS_UNIFIED_OWNER_R12C\.install\(root\);\s*\}\s*/g, "");
  text = text.replace(/if\s*\(\s*window\.KGW_SETTINGS_UNIFIED_OWNER_R12B[\s\S]*?KGW_SETTINGS_UNIFIED_OWNER_R12B\.install\(root\);\s*\}\s*/g, "");
  text = text.replace(/if\s*\(\s*window\.KGW_SETTINGS_UNIFIED_OWNER_R12(?![A-Z0-9_])[\s\S]*?KGW_SETTINGS_UNIFIED_OWNER_R12\.install\(root\);\s*\}\s*/g, "");

  let changed = true;
  let guard = 0;
  const removals = [];

  while (changed && guard < 200) {
    changed = false;
    guard++;

    const activeMarkers = staleRuntimeMarkers.concat(["KGW_SETTINGS_UNIFIED_OWNER_R12"]);

    for (const marker of activeMarkers) {
      const index = marker === "KGW_SETTINGS_UNIFIED_OWNER_R12"
        ? (() => {
            const match = text.match(exactR12Pattern);
            return match && match.index !== undefined ? match.index : -1;
          })()
        : text.indexOf(marker);

      if (index < 0) continue;

      const ranges = collectFunctionRangesContaining(text, index);
      const chosen = ranges.find((range) => range.size <= 80000);

      if (chosen) {
        removals.push({
          marker,
          strategy: "remove-enclosing-function-or-iife",
          start: chosen.start,
          end: chosen.end,
          size: chosen.size
        });

        text = text.slice(0, chosen.start) + "\n" + text.slice(chosen.end);
      } else {
        removals.push({
          marker,
          strategy: "remove-line",
          index
        });

        text = removeLineContaining(text, index);
      }

      changed = true;
      break;
    }
  }

  return { text, removals };
}

function buildOwner(tabName) {
  return `
// KGW_SETTINGS_UNIFIED_OWNER_R12E
(function installKgwSettingsUnifiedOwnerR12E() {
  const OWNER = "KGW_SETTINGS_UNIFIED_OWNER_R12E";
  const FEEDBACK_MS = 10000;
  const feedbackTimers = new WeakMap();

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

  function trace(root, event, extra) {
    try {
      const payload = Object.assign({
        owner: OWNER,
        tab: "${tabName}",
        event
      }, extra || {});

      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else if (window.__TAURI__ && typeof window.__TAURI__.invoke === "function") {
        window.__TAURI__.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else {
        console.debug("[KGW_BUTTON_TRACE]", payload);
      }
    } catch (_) {}
  }

  function isActionButton(button) {
    if (!button) return false;

    const text = (button.textContent || "").trim().toLowerCase();
    const action = (
      button.dataset.kgwSettingsAction ||
      button.dataset.action ||
      button.getAttribute("data-action") ||
      button.getAttribute("aria-label") ||
      ""
    ).toLowerCase();

    return (
      action.includes("save") ||
      action.includes("restore") ||
      action.includes("default") ||
      text.includes("save settings") ||
      text.includes("restore defaults") ||
      text.includes("set as defaults") ||
      text.includes("حفظ") ||
      text.includes("استعادة") ||
      text.includes("افتراض")
    );
  }

  function findActionButtons(root) {
    const scope = root || document;
    return Array.from(scope.querySelectorAll("button")).filter(isActionButton);
  }

  function setButtonsDisabled(root, disabled, reason) {
    const buttons = findActionButtons(root);

    buttons.forEach(function (button) {
      button.disabled = !!disabled;
      button.setAttribute("aria-disabled", disabled ? "true" : "false");
      button.classList.toggle("kgw-settings-action-disabled-r12e", !!disabled);
      button.dataset.kgwR12eDisabled = disabled ? "true" : "false";
    });

    trace(root, "native-disabled-owned-r12e", {
      disabled: !!disabled,
      reason: reason || "",
      buttons: buttons.length
    });
  }

  function rememberOriginalLabel(button) {
    if (!button.dataset.kgwR12eOriginalLabel) {
      button.dataset.kgwR12eOriginalLabel = button.textContent || "";
    }
  }

  function feedbackLabelForAction(action, fallbackText) {
    const normalized = String(action || "").toLowerCase();
    const fallback = String(fallbackText || "").toLowerCase();

    if (normalized.includes("restore") || fallback.includes("restore")) {
      return translate("settings.feedback.restored", "Restored");
    }

    if (normalized.includes("default") || fallback.includes("default")) {
      return translate("settings.feedback.setAsDefaults", "Set as defaults");
    }

    return translate("settings.feedback.saved", "Saved");
  }

  function beginFeedback(root, clickedButton, action) {
    if (!clickedButton) return;

    rememberOriginalLabel(clickedButton);

    const previousTimer = feedbackTimers.get(clickedButton);
    if (previousTimer) {
      clearTimeout(previousTimer);
      feedbackTimers.delete(clickedButton);
    }

    const original = clickedButton.dataset.kgwR12eOriginalLabel || clickedButton.textContent || "";
    clickedButton.textContent = feedbackLabelForAction(action, original);

    setButtonsDisabled(root, true, "feedback-start-r12e");

    const timer = setTimeout(function () {
      clickedButton.textContent = original;
      feedbackTimers.delete(clickedButton);
      setButtonsDisabled(root, true, "feedback-end-r12e");
      trace(root, "feedback-complete-r12e", { action: action || "" });
    }, FEEDBACK_MS);

    feedbackTimers.set(clickedButton, timer);

    trace(root, "feedback-start-r12e", {
      action: action || "",
      holdMs: FEEDBACK_MS
    });
  }

  function isRealSettingsChangeTarget(target, root) {
    if (!target || !root || !root.contains(target)) return false;

    const element = target.closest ? target.closest("input, select, textarea, button") : null;
    if (!element) return false;
    if (element.tagName === "BUTTON") return false;
    if (element.disabled) return false;

    return true;
  }

  function install(root) {
    if (!root || root.dataset.kgwSettingsUnifiedOwnerR12e === "installed") return;

    root.dataset.kgwSettingsUnifiedOwnerR12e = "installed";

    setButtonsDisabled(root, true, "initial-load-r12e");

    root.addEventListener("input", function (event) {
      if (!event.isTrusted) {
        trace(root, "ignored-programmatic-input-r12e", {});
        return;
      }

      if (!isRealSettingsChangeTarget(event.target, root)) return;

      setButtonsDisabled(root, false, "trusted-input-r12e");
    }, true);

    root.addEventListener("change", function (event) {
      if (!event.isTrusted) {
        trace(root, "ignored-programmatic-change-r12e", {});
        return;
      }

      if (!isRealSettingsChangeTarget(event.target, root)) return;

      setButtonsDisabled(root, false, "trusted-change-r12e");
    }, true);

    root.addEventListener("click", function (event) {
      const button = event.target && event.target.closest ? event.target.closest("button") : null;
      if (!button || !root.contains(button)) return;
      if (!isActionButton(button)) return;

      const action =
        button.dataset.kgwSettingsAction ||
        button.dataset.action ||
        button.getAttribute("data-action") ||
        button.textContent ||
        "";

      trace(root, "action-click-r12e", { action });
      beginFeedback(root, button, action);
    }, true);

    trace(root, "installed-r12e", {
      buttons: findActionButtons(root).length
    });
  }

  window.KGW_SETTINGS_UNIFIED_OWNER_R12E = {
    install,
    setButtonsDisabled,
    beginFeedback,
    findActionButtons
  };
})();
// END_KGW_SETTINGS_UNIFIED_OWNER_R12E
`;
}

function injectUnifiedOwner(jsText, tabName) {
  const cleanup = removeGeneratedBlocks(jsText);
  let text = cleanup.text;
  const installActions = findFunction(text, "installActions");
  const owner = buildOwner(tabName);

  if (installActions) {
    let body = installActions.source;

    if (!body.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E.install(root);")) {
      const injection = `{
  if (window.KGW_SETTINGS_UNIFIED_OWNER_R12E && typeof window.KGW_SETTINGS_UNIFIED_OWNER_R12E.install === "function") {
    window.KGW_SETTINGS_UNIFIED_OWNER_R12E.install(root);
  }`;
      body = body.replace("{", injection);
    }

    text = text.slice(0, installActions.start) + body + text.slice(installActions.end);
    return { text: owner + "\n" + text, removals: cleanup.removals };
  }

  const fallback = `
document.addEventListener("DOMContentLoaded", function () {
  const root = document.body;
  if (window.KGW_SETTINGS_UNIFIED_OWNER_R12E && typeof window.KGW_SETTINGS_UNIFIED_OWNER_R12E.install === "function") {
    window.KGW_SETTINGS_UNIFIED_OWNER_R12E.install(root);
  }
});
`;

  return { text: owner + "\n" + text + "\n" + fallback, removals: cleanup.removals };
}

function patchCss(cssText) {
  let text = cssText;

  const patterns = [
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL\s*\*\//g
  ];

  for (const pattern of patterns) {
    text = text.replace(pattern, "");
  }

  const block = `
/* KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL */
button.kgw-settings-action-disabled-r12e,
button[data-kgw-r12e-disabled="true"],
button:disabled.kgw-settings-action-disabled-r12e {
  opacity: 0.45;
  filter: grayscale(0.35);
  cursor: not-allowed;
}

button[data-kgw-r12e-disabled="false"] {
  opacity: 1;
  filter: none;
}
/* END_KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL */
`;

  return text.trimEnd() + "\n\n" + block + "\n";
}

function ensureRustTrace(libText) {
  let text = libText;

  if (text.includes("kgw_frontend_button_trace_v1")) {
    return text;
  }

  const command = `
#[tauri::command]
fn kgw_frontend_button_trace_v1(payload: serde_json::Value) {
    println!("[KGW_BUTTON_TRACE] {}", payload);
}
`;

  text = text + "\n" + command;

  const handlerRegex = /tauri::generate_handler!\s*\[([\s\S]*?)\]/m;
  const match = text.match(handlerRegex);

  if (match && !match[1].includes("kgw_frontend_button_trace_v1")) {
    const inner = match[1].trim();
    const nextInner = inner.length
      ? inner + ",\n            kgw_frontend_button_trace_v1"
      : "\n            kgw_frontend_button_trace_v1\n        ";
    text = text.replace(handlerRegex, "tauri::generate_handler![" + nextInner + "]");
  }

  return text;
}

function remainingStaleMarkers(text) {
  const found = staleRuntimeMarkers.filter((marker) => text.includes(marker));
  if (exactR12Pattern.test(text)) {
    found.push("KGW_SETTINGS_UNIFIED_OWNER_R12");
  }
  return found;
}

function validateJs(text, label) {
  const errors = [];

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")) {
    errors.push(label + ": missing unified owner marker");
  }

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E.install(root);")) {
    errors.push(label + ": installActions or fallback does not call R12E owner");
  }

  if (!/button\.disabled\s*=\s*!!disabled/.test(text)) {
    errors.push(label + ": native button.disabled ownership missing");
  }

  const stale = remainingStaleMarkers(text);
  if (stale.length) {
    errors.push(label + ": stale marker still present: " + stale.join(", "));
  }

  return errors;
}

function validateCss(text, label) {
  const errors = [];

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E_VISUAL")) {
    errors.push(label + ": missing disabled visual marker");
  }

  if (!text.includes("kgw-settings-action-disabled-r12e")) {
    errors.push(label + ": missing disabled visual class");
  }

  if (
    text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL") ||
    text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL") ||
    text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12D_VISUAL")
  ) {
    errors.push(label + ": stale R12B/R12C/R12D visual marker still present");
  }

  return errors;
}

function main() {
  const before = {
    nodeJs: read(files.nodeJs),
    nodeCss: read(files.nodeCss),
    bridgeJs: read(files.bridgeJs),
    bridgeCss: read(files.bridgeCss),
    tauriLib: read(files.tauriLib)
  };

  const auditBefore = {
    nodeStaleMarkers: remainingStaleMarkers(before.nodeJs),
    bridgeStaleMarkers: remainingStaleMarkers(before.bridgeJs),
    nodeHasInstallActions: !!findFunction(before.nodeJs, "installActions"),
    bridgeHasInstallActions: !!findFunction(before.bridgeJs, "installActions"),
    rustTracePresent: before.tauriLib.includes("kgw_frontend_button_trace_v1")
  };

  const nodePatch = injectUnifiedOwner(before.nodeJs, "node");
  const bridgePatch = injectUnifiedOwner(before.bridgeJs, "bridge");

  write(files.nodeJs, nodePatch.text);
  write(files.bridgeJs, bridgePatch.text);
  write(files.nodeCss, patchCss(before.nodeCss));
  write(files.bridgeCss, patchCss(before.bridgeCss));
  write(files.tauriLib, ensureRustTrace(before.tauriLib));

  const afterNodeJs = read(files.nodeJs);
  const afterBridgeJs = read(files.bridgeJs);
  const afterNodeCss = read(files.nodeCss);
  const afterBridgeCss = read(files.bridgeCss);

  const validationErrors = [
    ...validateJs(afterNodeJs, "Node"),
    ...validateJs(afterBridgeJs, "Bridge"),
    ...validateCss(afterNodeCss, "Node CSS"),
    ...validateCss(afterBridgeCss, "Bridge CSS")
  ];

  const auditAfter = {
    validationErrors,
    nodeR12ECount: (afterNodeJs.match(/KGW_SETTINGS_UNIFIED_OWNER_R12E/g) || []).length,
    bridgeR12ECount: (afterBridgeJs.match(/KGW_SETTINGS_UNIFIED_OWNER_R12E/g) || []).length,
    nodeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(afterNodeJs),
    bridgeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(afterBridgeJs),
    remainingNodeStaleMarkers: remainingStaleMarkers(afterNodeJs),
    remainingBridgeStaleMarkers: remainingStaleMarkers(afterBridgeJs),
    nodeDeepRemovals: nodePatch.removals,
    bridgeDeepRemovals: bridgePatch.removals
  };

  writeJson("audit-before-r12e.json", auditBefore);
  writeJson("audit-after-r12e.json", auditAfter);

  if (validationErrors.length) {
    throw new Error("R12E validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R12E patch validation passed");
  console.log(JSON.stringify(auditAfter, null, 2));
}

main();
