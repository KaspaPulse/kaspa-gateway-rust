const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_buttons_unified_owner_rebuild_r12c.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"),
  tauriLib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
};

const staleMarkers = [
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
  "initial-load-r10",
  "suppressed-by-r9b",
  "feedback-lock-start",
  "feedback-lock-end"
];

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function findFunction(text, functionName) {
  const re = new RegExp("function\\s+" + functionName + "\\s*\\([^)]*\\)\\s*\\{");
  const match = text.match(re);
  if (!match || match.index === undefined) return null;

  const start = match.index;
  const open = text.indexOf("{", start);
  let depth = 0;

  for (let i = open; i < text.length; i++) {
    const ch = text[i];
    if (ch === "{") depth++;
    if (ch === "}") depth--;

    if (depth === 0) {
      return {
        start,
        end: i + 1,
        source: text.slice(start, i + 1)
      };
    }
  }

  return null;
}

function findContainingIife(text, markerIndex) {
  let start = text.lastIndexOf("(function", markerIndex);
  if (start < 0) start = text.lastIndexOf("function", markerIndex);
  if (start < 0) return null;

  const open = text.indexOf("{", start);
  if (open < 0 || open > markerIndex) return null;

  let depth = 0;
  for (let i = open; i < text.length; i++) {
    const ch = text[i];
    if (ch === "{") depth++;
    if (ch === "}") depth--;

    if (depth === 0) {
      let end = i + 1;

      const tail = text.slice(end, end + 20);
      const callMatch = tail.match(/^\s*\)\s*\(\s*\)\s*;?/);
      if (callMatch) {
        end += callMatch[0].length;
      }

      return { start, end };
    }
  }

  return null;
}

function removeRange(text, start, end) {
  return text.slice(0, start) + "\n" + text.slice(end);
}

function removeGeneratedBlocks(source) {
  let text = source;

  const blockPatterns = [
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12C[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12C\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12B[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12B\s*/g,
    /\/\/\s*KGW_SETTINGS_UNIFIED_OWNER_R12[\s\S]*?\/\/\s*END_KGW_SETTINGS_UNIFIED_OWNER_R12\s*/g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)[\s\S]*?END_KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)\s*\*\//g,
    /\/\/\s*KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)[\s\S]*?\/\/\s*END_KGW_SETTINGS_BUTTONS_R(?:4B|4C|4D|6|7|7B|7C|8|9|9B|10|11)\s*/g
  ];

  for (const pattern of blockPatterns) {
    text = text.replace(pattern, "");
  }

  let changed = true;
  let guard = 0;

  while (changed && guard < 50) {
    changed = false;
    guard++;

    for (const marker of staleMarkers) {
      const idx = text.indexOf(marker);
      if (idx < 0) continue;

      const iife = findContainingIife(text, idx);
      if (iife && iife.end > iife.start) {
        text = removeRange(text, iife.start, iife.end);
        changed = true;
        break;
      }

      text = text.replaceAll(marker, "removed-by-r12c");
      changed = true;
      break;
    }
  }

  return text;
}

function buildOwner(tabName) {
  return `
// KGW_SETTINGS_UNIFIED_OWNER_R12C
(function installKgwSettingsUnifiedOwnerR12C() {
  const OWNER = "KGW_SETTINGS_UNIFIED_OWNER_R12C";
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
      button.classList.toggle("kgw-settings-action-disabled-r12c", !!disabled);
      button.dataset.kgwR12cDisabled = disabled ? "true" : "false";
    });

    trace(root, "native-disabled-owned-r12c", {
      disabled: !!disabled,
      reason: reason || "",
      buttons: buttons.length
    });
  }

  function rememberOriginalLabel(button) {
    if (!button.dataset.kgwR12cOriginalLabel) {
      button.dataset.kgwR12cOriginalLabel = button.textContent || "";
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

    const original = clickedButton.dataset.kgwR12cOriginalLabel || clickedButton.textContent || "";
    clickedButton.textContent = feedbackLabelForAction(action, original);

    setButtonsDisabled(root, true, "feedback-start-r12c");

    const timer = setTimeout(function () {
      clickedButton.textContent = original;
      feedbackTimers.delete(clickedButton);
      setButtonsDisabled(root, true, "feedback-end-r12c");
      trace(root, "feedback-complete-r12c", { action: action || "" });
    }, FEEDBACK_MS);

    feedbackTimers.set(clickedButton, timer);

    trace(root, "feedback-start-r12c", {
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
    if (!root || root.dataset.kgwSettingsUnifiedOwnerR12c === "installed") return;

    root.dataset.kgwSettingsUnifiedOwnerR12c = "installed";

    setButtonsDisabled(root, true, "initial-load-r12c");

    root.addEventListener("input", function (event) {
      if (!event.isTrusted) {
        trace(root, "ignored-programmatic-input-r12c", {});
        return;
      }

      if (!isRealSettingsChangeTarget(event.target, root)) return;

      setButtonsDisabled(root, false, "trusted-input-r12c");
    }, true);

    root.addEventListener("change", function (event) {
      if (!event.isTrusted) {
        trace(root, "ignored-programmatic-change-r12c", {});
        return;
      }

      if (!isRealSettingsChangeTarget(event.target, root)) return;

      setButtonsDisabled(root, false, "trusted-change-r12c");
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

      trace(root, "action-click-r12c", { action });
      beginFeedback(root, button, action);
    }, true);

    trace(root, "installed-r12c", {
      buttons: findActionButtons(root).length
    });
  }

  window.KGW_SETTINGS_UNIFIED_OWNER_R12C = {
    install,
    setButtonsDisabled,
    beginFeedback,
    findActionButtons
  };
})();
// END_KGW_SETTINGS_UNIFIED_OWNER_R12C
`;
}

function injectUnifiedOwner(jsText, tabName) {
  let text = removeGeneratedBlocks(jsText);
  const installActions = findFunction(text, "installActions");
  const owner = buildOwner(tabName);

  if (installActions) {
    let body = installActions.source;

    body = body.replace(/if\s*\(\s*window\.KGW_SETTINGS_UNIFIED_OWNER_R12B[\s\S]*?KGW_SETTINGS_UNIFIED_OWNER_R12B\.install\(root\);\s*\}\s*/g, "");
    body = body.replace(/if\s*\(\s*window\.KGW_SETTINGS_UNIFIED_OWNER_R12[\s\S]*?KGW_SETTINGS_UNIFIED_OWNER_R12\.install\(root\);\s*\}\s*/g, "");

    if (!body.includes("KGW_SETTINGS_UNIFIED_OWNER_R12C.install(root);")) {
      const injection = `{
  if (window.KGW_SETTINGS_UNIFIED_OWNER_R12C && typeof window.KGW_SETTINGS_UNIFIED_OWNER_R12C.install === "function") {
    window.KGW_SETTINGS_UNIFIED_OWNER_R12C.install(root);
  }`;
      body = body.replace("{", injection);
    }

    text = text.slice(0, installActions.start) + body + text.slice(installActions.end);
    return owner + "\n" + text;
  }

  const fallback = `
document.addEventListener("DOMContentLoaded", function () {
  const root = document.body;
  if (window.KGW_SETTINGS_UNIFIED_OWNER_R12C && typeof window.KGW_SETTINGS_UNIFIED_OWNER_R12C.install === "function") {
    window.KGW_SETTINGS_UNIFIED_OWNER_R12C.install(root);
  }
});
`;

  return owner + "\n" + text + "\n" + fallback;
}

function patchCss(cssText) {
  let text = cssText;

  const patterns = [
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL\s*\*\//g,
    /\/\*\s*KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL[\s\S]*?END_KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL\s*\*\//g
  ];

  for (const pattern of patterns) {
    text = text.replace(pattern, "");
  }

  const block = `
/* KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL */
button.kgw-settings-action-disabled-r12c,
button[data-kgw-r12c-disabled="true"],
button:disabled.kgw-settings-action-disabled-r12c {
  opacity: 0.45;
  filter: grayscale(0.35);
  cursor: not-allowed;
}

button[data-kgw-r12c-disabled="false"] {
  opacity: 1;
  filter: none;
}
/* END_KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL */
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

function validateJs(text, label) {
  const errors = [];

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12C")) {
    errors.push(label + ": missing unified owner marker");
  }

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12C.install(root);")) {
    errors.push(label + ": installActions or fallback does not call R12C owner");
  }

  if (!/button\.disabled\s*=\s*!!disabled/.test(text)) {
    errors.push(label + ": native button.disabled ownership missing");
  }

  for (const marker of staleMarkers) {
    if (text.includes(marker)) {
      errors.push(label + ": stale marker still present: " + marker);
    }
  }

  return errors;
}

function validateCss(text, label) {
  const errors = [];

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12C_VISUAL")) {
    errors.push(label + ": missing disabled visual marker");
  }

  if (!text.includes("kgw-settings-action-disabled-r12c")) {
    errors.push(label + ": missing disabled visual class");
  }

  if (text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12B_VISUAL")) {
    errors.push(label + ": stale R12B visual marker still present");
  }

  return errors;
}

function writeJson(fileName, data) {
  fs.writeFileSync(path.join(reportDir, fileName), JSON.stringify(data, null, 2) + "\n", "utf8");
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
    nodeStaleMarkers: staleMarkers.filter((m) => before.nodeJs.includes(m)),
    bridgeStaleMarkers: staleMarkers.filter((m) => before.bridgeJs.includes(m)),
    nodeHasInstallActions: !!findFunction(before.nodeJs, "installActions"),
    bridgeHasInstallActions: !!findFunction(before.bridgeJs, "installActions"),
    rustTracePresent: before.tauriLib.includes("kgw_frontend_button_trace_v1")
  };

  write(files.nodeJs, injectUnifiedOwner(before.nodeJs, "node"));
  write(files.bridgeJs, injectUnifiedOwner(before.bridgeJs, "bridge"));
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
    nodeR12CCount: (afterNodeJs.match(/KGW_SETTINGS_UNIFIED_OWNER_R12C/g) || []).length,
    bridgeR12CCount: (afterBridgeJs.match(/KGW_SETTINGS_UNIFIED_OWNER_R12C/g) || []).length,
    nodeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(afterNodeJs),
    bridgeDisabledOwnership: /button\.disabled\s*=\s*!!disabled/.test(afterBridgeJs),
    remainingNodeStaleMarkers: staleMarkers.filter((m) => afterNodeJs.includes(m)),
    remainingBridgeStaleMarkers: staleMarkers.filter((m) => afterBridgeJs.includes(m))
  };

  writeJson("audit-before-r12c.json", auditBefore);
  writeJson("audit-after-r12c.json", auditAfter);

  if (validationErrors.length) {
    throw new Error("R12C validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R12C patch validation passed");
  console.log(JSON.stringify(auditAfter, null, 2));
}

main();
