const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"),
  frontendRoot: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend"),
  lib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"),
};

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function saveJson(name, value) {
  fs.mkdirSync(reportDir, { recursive: true });
  fs.writeFileSync(path.join(reportDir, name), JSON.stringify(value, null, 2), "utf8");
}

function count(text, re) {
  const m = text.match(re);
  return m ? m.length : 0;
}

function functionBounds(text, name) {
  const re = new RegExp("function\\s+" + name + "\\s*\\([^)]*\\)\\s*\\{");
  const m = text.match(re);
  if (!m) throw new Error("Function not found: " + name);

  const start = m.index;
  const brace = text.indexOf("{", start);
  let depth = 0;
  let end = brace;

  for (; end < text.length; end++) {
    const ch = text[end];
    if (ch === "{") depth++;
    if (ch === "}") {
      depth--;
      if (depth === 0) {
        end++;
        break;
      }
    }
  }

  return { start, end, body: text.slice(start, end) };
}

function replaceFunction(text, name, replacement) {
  const b = functionBounds(text, name);
  return text.slice(0, b.start) + replacement + text.slice(b.end);
}

function insertBeforeFunction(text, name, block) {
  const idx = text.indexOf("function " + name);
  if (idx < 0) throw new Error("Insertion function not found: " + name);
  return text.slice(0, idx) + block + text.slice(idx);
}

function removeNamedBlock(text, start, end) {
  const commentRe = new RegExp("\\n?\\/\\*\\s*" + start + "\\s*\\*\\/[\\s\\S]*?\\/\\*\\s*" + end + "\\s*\\*\\/\\n?", "g");
  text = text.replace(commentRe, "\n");

  const lineRe = new RegExp("\\n?\\s*//\\s*" + start + "[\\s\\S]*?//\\s*" + end + "\\s*\\n?", "g");
  text = text.replace(lineRe, "\n");

  return text;
}

function stripOldGeneratedBlocks(text) {
  const bases = [
    "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D",
    "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D",
    "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6",
    "KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6",
    "KGW_NODE_SETTINGS_SIMPLE_UX_R7",
    "KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7",
    "KGW_NODE_SETTINGS_SIMPLE_UX_R7B",
    "KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7B",
    "KGW_NODE_SETTINGS_SIMPLE_UX_R7C",
    "KGW_BRIDGE_SETTINGS_SIMPLE_UX_R7C",
    "KGW_SETTINGS_OWNER_CONSOLIDATION_R9",
    "KGW_SETTINGS_OWNER_CONSOLIDATION_R9B",
    "KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11",
    "KGW_SETTINGS_BUTTONS_PRECLICK_TRACE_R4E",
    "KGW_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_SETTINGS_UNIFIED_OWNER_R12"
  ];

  for (const base of bases) {
    text = removeNamedBlock(text, base + "_START", base + "_END");
  }

  text = text.replace(/\n{4,}/g, "\n\n\n");
  return text;
}

function ensureRustTraceCommand() {
  let text = read(files.lib);
  const before = text;

  if (!/fn\s+kgw_frontend_button_trace_v1\s*\(/.test(text)) {
    const command = [
      "#[tauri::command]",
      "fn kgw_frontend_button_trace_v1(scope: String, net: String, action: String, phase: String, details: String) -> bool {",
      "    println!(",
      "        \"[KGW_BUTTON_TRACE] scope={} net={} action={} phase={} details={}\",",
      "        scope, net, action, phase, details",
      "    );",
      "    true",
      "}",
      ""
    ].join("\n");

    const insertAt = text.indexOf("#[tauri::command]");
    if (insertAt < 0) throw new Error("No tauri command insertion point found in lib.rs");
    text = text.slice(0, insertAt) + command + "\n" + text.slice(insertAt);
  }

  const handlerRe = /tauri::generate_handler!\s*\[([\s\S]*?)\]/m;
  const m = text.match(handlerRe);
  if (!m) throw new Error("Could not find tauri::generate_handler![...] in lib.rs");

  const names = m[1].split(",").map((x) => x.trim()).filter(Boolean);
  if (!names.includes("kgw_frontend_button_trace_v1")) names.push("kgw_frontend_button_trace_v1");

  text = text.replace(handlerRe, "tauri::generate_handler![\n            " + names.join(",\n            ") + "\n        ]");
  write(files.lib, text);

  return {
    changed: before !== text,
    traceCommand: count(text, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
    tracePrint: count(text, /\[KGW_BUTTON_TRACE\]/g),
    handlerRefs: count(text, /kgw_frontend_button_trace_v1/g),
  };
}

function unifiedOwnerBlock(kind) {
  const isNode = kind === "node";
  const scope = isNode ? "node" : "bridge";
  const cap = isNode ? "Node" : "Bridge";
  const varPrefix = isNode ? "kgwNode" : "kgwBridge";
  const networks = isNode ? "NODE_NETWORKS" : "BRIDGE_NETWORKS";
  const actionAttr = isNode ? "data-node-action" : "data-bridge-action";
  const datasetAction = isNode ? "nodeAction" : "bridgeAction";
  const readSettings = `${varPrefix}R51ReadSettings`;
  const saveSettings = `${varPrefix}R51SaveSettings`;
  const restoreDefaults = `${varPrefix}R51RestoreDefaults`;
  const setDefaults = `${varPrefix}R51SetAsDefaults`;
  const currentVisible = isNode ? "''" : "kgwBridgeCurrentVisibleNetwork(root)";

  return `
/* KGW_SETTINGS_UNIFIED_OWNER_R12_START */
const ${varPrefix}SettingsOwnerStateR12 = new Map();

function ${varPrefix}SettingsOwnerR12State(net) {
  if (!${varPrefix}SettingsOwnerStateR12.has(net)) {
    ${varPrefix}SettingsOwnerStateR12.set(net, {
      baseline: "",
      lockedUntil: 0,
      feedbackTimer: null,
      feedbackInterval: null,
      installedRoots: new WeakSet(),
    });
  }

  return ${varPrefix}SettingsOwnerStateR12.get(net);
}

function ${varPrefix}SettingsOwnerR12IsAction(action) {
  return action === "save-settings" || action === "restore-defaults" || action === "set-defaults";
}

function ${varPrefix}SettingsOwnerR12Normalize(value) {
  const seen = new WeakSet();

  const normalize = (item) => {
    if (item === null || typeof item !== "object") return item;
    if (seen.has(item)) return "[Circular]";
    seen.add(item);

    if (Array.isArray(item)) return item.map(normalize);

    return Object.keys(item).sort().reduce((acc, key) => {
      acc[key] = normalize(item[key]);
      return acc;
    }, {});
  };

  return JSON.stringify(normalize(value || {}));
}

function ${varPrefix}SettingsOwnerR12Snapshot(net) {
  return ${varPrefix}SettingsOwnerR12Normalize(${readSettings}(net));
}

function ${varPrefix}SettingsOwnerR12Trace(net, action, phase, details = {}) {
  const payload = {
    scope: "${scope}",
    net: String(net || "unknown"),
    action: String(action || "unknown"),
    phase: String(phase || "unknown"),
    details: JSON.stringify(Object.assign({ at: new Date().toISOString() }, details || {})),
  };

  try {
    if (window.__TAURI__?.core?.invoke) {
      window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", payload).catch(() => {});
      return;
    }

    if (window.__TAURI__?.tauri?.invoke) {
      window.__TAURI__.tauri.invoke("kgw_frontend_button_trace_v1", payload).catch(() => {});
      return;
    }

    if (typeof window.__TAURI_INVOKE__ === "function") {
      window.__TAURI_INVOKE__("kgw_frontend_button_trace_v1", payload).catch(() => {});
      return;
    }
  } catch {
    // Trace must never break settings UI.
  }

  try {
    console.log("[KGW_BUTTON_TRACE]", payload);
  } catch {
    // Ignore console failures.
  }
}

function ${varPrefix}SettingsTraceR4D(net, action, phase, details = {}) {
  ${varPrefix}SettingsOwnerR12Trace(net, action, phase, details);
}

function ${varPrefix}SettingsActionIsR6(action) {
  return ${varPrefix}SettingsOwnerR12IsAction(action);
}

function ${varPrefix}UpdateSettingsDirtyButtonsR4D(net, reason = "legacy-call") {
  ${varPrefix}SettingsOwnerR12Trace(net, "legacy-dirty", "ignored-by-r12", { reason, owner: "R12" });
}

function ${varPrefix}UpdateAllSettingsDirtyButtonsR4D(reason = "legacy-call") {
  if (typeof ${networks} === "undefined") return;
  ${networks}.forEach((net) => ${varPrefix}UpdateSettingsDirtyButtonsR4D(net.key, reason));
}

function ${varPrefix}UpdateSettingsDirtyButtonsR2(net, reason = "legacy-call") {
  ${varPrefix}UpdateSettingsDirtyButtonsR4D(net, reason);
}

function ${varPrefix}UpdateAllSettingsDirtyButtonsR2(reason = "legacy-call") {
  ${varPrefix}UpdateAllSettingsDirtyButtonsR4D(reason);
}

function ${varPrefix}SettingsOwnerR12Buttons(net, action = "") {
  const selector = action
    ? \`[${actionAttr}="\${action}"][data-net="\${net}"]\`
    : \`[${actionAttr}][data-net="\${net}"]\`;

  return Array.from(document.querySelectorAll(selector))
    .filter((button) => ${varPrefix}SettingsOwnerR12IsAction(button.dataset.${datasetAction}));
}

function ${varPrefix}SettingsOwnerR12SetEnabled(net, enabled, reason = "unknown") {
  const state = ${varPrefix}SettingsOwnerR12State(net);
  const locked = state.lockedUntil > Date.now();
  const finalEnabled = locked ? false : Boolean(enabled);

  ["save-settings", "restore-defaults", "set-defaults"].forEach((action) => {
    ${varPrefix}SettingsOwnerR12Buttons(net, action).forEach((button) => {
      button.disabled = !finalEnabled;
      button.setAttribute("aria-disabled", finalEnabled ? "false" : "true");
      button.dataset.kgwSettingsActionDisabled = finalEnabled ? "0" : "1";
      button.classList.toggle("kgw-settings-action-disabled", !finalEnabled);
    });
  });

  ${varPrefix}SettingsOwnerR12Trace(net, "settings-buttons", finalEnabled ? "enabled" : "disabled", {
    reason,
    locked,
    nativeDisabledExpected: !finalEnabled,
  });
}

function ${varPrefix}SetSettingsActionEnabledR2(net, action, enabled) {
  ${varPrefix}SettingsOwnerR12SetEnabled(net, enabled, "compat-set-enabled-" + action);
}

function ${varPrefix}SettingsOwnerR12AcceptBaseline(net, reason = "baseline") {
  const state = ${varPrefix}SettingsOwnerR12State(net);
  state.baseline = ${varPrefix}SettingsOwnerR12Snapshot(net);
  ${varPrefix}SettingsOwnerR12SetEnabled(net, false, reason);
}

function ${varPrefix}SettingsOwnerR12Changed(net, reason = "change") {
  if (!net) return;

  const state = ${varPrefix}SettingsOwnerR12State(net);

  if (state.lockedUntil > Date.now()) {
    ${varPrefix}SettingsOwnerR12SetEnabled(net, false, reason + "-locked");
    return;
  }

  if (!state.baseline) {
    ${varPrefix}SettingsOwnerR12AcceptBaseline(net, "auto-baseline-before-" + reason);
    return;
  }

  const current = ${varPrefix}SettingsOwnerR12Snapshot(net);
  ${varPrefix}SettingsOwnerR12SetEnabled(net, current !== state.baseline, reason);
}

function ${varPrefix}SettingsOwnerR12NetFromEvent(event, root) {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i], [class*='mainnet' i], [class*='testnet10' i], [class*='testnet12' i]");

  const raw = [
    target?.dataset?.net,
    target?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    target?.id,
    carrier?.id,
    carrier?.className,
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";

  try {
    return ${currentVisible} || "";
  } catch {
    return "";
  }
}

function ${varPrefix}SettingsOwnerR12Language() {
  const raw = [
    document.documentElement?.lang,
    document.body?.getAttribute("lang"),
    localStorage.getItem("kgw-language"),
    localStorage.getItem("kgw_lang"),
    localStorage.getItem("language"),
    localStorage.getItem("locale"),
    "en",
  ].filter(Boolean)[0];

  const lang = String(raw || "en").toLowerCase().split("-")[0];
  return ["ar", "de", "en", "es", "fr"].includes(lang) ? lang : "en";
}

function ${varPrefix}SettingsOwnerR12Label(action) {
  const keyMap = {
    "save-settings": "settings.feedback.saved",
    "restore-defaults": "settings.feedback.restored",
    "set-defaults": "settings.feedback.setAsDefaults",
  };

  const key = keyMap[action];

  if (key) {
    const translators = [
      window.kgwT,
      window.t,
      window.__kgwT,
      window.__t,
      window.kgwI18n?.t,
      window.KGW_I18N?.t,
      window.i18n?.t,
      window.__kgwI18n?.t,
    ].filter((candidate) => typeof candidate === "function");

    for (const translate of translators) {
      try {
        const value = translate(key);
        if (value && value !== key) return value;
      } catch {
        // Continue fallback.
      }
    }
  }

  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = ${varPrefix}SettingsOwnerR12Language();
  return labels[lang]?.[action] || labels.en[action] || action;
}

function ${varPrefix}SettingsOwnerR12LockFeedback(net, action, button, label) {
  const state = ${varPrefix}SettingsOwnerR12State(net);
  const targets = ${varPrefix}SettingsOwnerR12Buttons(net, action);
  const original = button.dataset.kgwOriginalLabel || button.textContent;

  window.clearTimeout(state.feedbackTimer);
  window.clearInterval(state.feedbackInterval);

  state.lockedUntil = Date.now() + 10000;
  state.feedbackAction = action;
  state.feedbackLabel = label;

  targets.forEach((target) => {
    target.dataset.kgwOriginalLabel = target.dataset.kgwOriginalLabel || original;
    target.dataset.kgwFeedbackActive = "1";
    target.dataset.kgwFeedbackUntil = String(state.lockedUntil);
    target.textContent = label;
  });

  ${varPrefix}SettingsOwnerR12SetEnabled(net, false, "feedback-lock-start-" + action);

  state.feedbackInterval = window.setInterval(() => {
    if (state.lockedUntil <= Date.now()) return;

    targets.forEach((target) => {
      target.textContent = label;
      target.disabled = true;
      target.setAttribute("aria-disabled", "true");
      target.dataset.kgwSettingsActionDisabled = "1";
      target.classList.add("kgw-settings-action-disabled");
    });
  }, 100);

  state.feedbackTimer = window.setTimeout(() => {
    window.clearInterval(state.feedbackInterval);
    state.lockedUntil = 0;

    targets.forEach((target) => {
      target.dataset.kgwFeedbackActive = "0";
      target.dataset.kgwFeedbackUntil = "0";
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });

    ${varPrefix}SettingsOwnerR12AcceptBaseline(net, "feedback-lock-end-" + action);
  }, 10000);
}

function ${varPrefix}FlashSettingsActionButtonR2(button, label) {
  const action = button.dataset.${datasetAction};
  const net = button.dataset.net || button.dataset.network || "";
  if (action && net) {
    ${varPrefix}SettingsOwnerR12LockFeedback(net, action, button, label);
    return;
  }

  const original = button.dataset.kgwOriginalLabel || button.textContent;
  button.dataset.kgwOriginalLabel = original;
  button.textContent = label;
  window.setTimeout(() => {
    button.textContent = button.dataset.kgwOriginalLabel || original;
  }, 10000);
}

function ${varPrefix}SettingsOwnerR12HandleAction(net, action, button) {
  if (!net || !${varPrefix}SettingsOwnerR12IsAction(action)) return false;

  const state = ${varPrefix}SettingsOwnerR12State(net);

  ${varPrefix}SettingsOwnerR12Trace(net, action, "click-received", {
    disabled: Boolean(button.disabled),
    ariaDisabled: button.getAttribute("aria-disabled"),
    text: String(button.textContent || "").trim(),
    locked: state.lockedUntil > Date.now(),
  });

  if (button.disabled || state.lockedUntil > Date.now()) {
    ${varPrefix}SettingsOwnerR12Trace(net, action, "click-ignored-disabled", {
      disabled: Boolean(button.disabled),
      locked: state.lockedUntil > Date.now(),
    });
    return true;
  }

  ${varPrefix}SettingsOwnerR12Trace(net, action, "action-start", {
    beforeDisabled: Boolean(button.disabled),
  });

  if (action === "save-settings") {
    ${saveSettings}(net);
  } else if (action === "restore-defaults") {
    ${restoreDefaults}(net);
  } else if (action === "set-defaults") {
    ${setDefaults}(net);
  }

  ${varPrefix}SettingsOwnerR12AcceptBaseline(net, "after-" + action + "-immediate-r12");

  const label = ${varPrefix}SettingsOwnerR12Label(action);
  ${varPrefix}SettingsOwnerR12LockFeedback(net, action, button, label);

  ${varPrefix}SettingsOwnerR12Trace(net, action, "action-complete", {
    label,
    holdMs: 10000,
  });

  return true;
}

function ${varPrefix}SettingsOwnerR12Install(root) {
  if (!root || root.__kgwSettingsOwnerR12Installed) return;
  root.__kgwSettingsOwnerR12Installed = true;

  const applyInitial = (reason) => {
    if (typeof ${networks} === "undefined") return;
    ${networks}.forEach((net) => ${varPrefix}SettingsOwnerR12AcceptBaseline(net.key, reason));
  };

  applyInitial("initial-load-r12");

  root.addEventListener("input", (event) => {
    if (!event.isTrusted) return;
    const net = ${varPrefix}SettingsOwnerR12NetFromEvent(event, root);
    if (!net) return;
    ${isNode ? "updateAllCommands();" : "bridgeSyncAllModeControls();\n    updateAllCommands();"}
    ${varPrefix}SettingsOwnerR12Changed(net, "trusted-input");
  });

  root.addEventListener("change", (event) => {
    if (!event.isTrusted) return;
    const net = ${varPrefix}SettingsOwnerR12NetFromEvent(event, root);
    if (!net) return;
    ${isNode ? "updateAllCommands();" : "bridgeSyncAllModeControls();\n    updateAllCommands();"}
    ${varPrefix}SettingsOwnerR12Changed(net, "trusted-change");
  });
}
/* KGW_SETTINGS_UNIFIED_OWNER_R12_END */

`;
}

function removeRootListeners(body, eventName) {
  let result = body;

  while (true) {
    const idx = result.indexOf(`root.addEventListener("${eventName}"`);
    if (idx < 0) break;

    const paren = result.indexOf("(", idx);
    let depth = 0;
    let end = paren;

    for (; end < result.length; end++) {
      const ch = result[end];
      if (ch === "(") depth++;
      if (ch === ")") {
        depth--;
        if (depth === 0) {
          end++;
          break;
        }
      }
    }

    if (result.slice(end, end + 1) === ";") end++;
    result = result.slice(0, idx) + result.slice(end);
  }

  return result;
}

function replaceActionBranch(body, action, replacement) {
  const needle = `if (action === "${action}") {`;
  const start = body.indexOf(needle);
  if (start < 0) throw new Error("Action branch not found: " + action);

  const brace = body.indexOf("{", start);
  let depth = 0;
  let end = brace;

  for (; end < body.length; end++) {
    const ch = body[end];
    if (ch === "{") depth++;
    if (ch === "}") {
      depth--;
      if (depth === 0) {
        end++;
        break;
      }
    }
  }

  return body.slice(0, start) + replacement + body.slice(end);
}

function patchInstallActions(text, kind) {
  const isNode = kind === "node";
  const prefix = isNode ? "kgwNode" : "kgwBridge";
  const datasetAction = isNode ? "nodeAction" : "bridgeAction";
  const netLine = isNode
    ? `    const net = button.dataset.net;`
    : `    const net = button.dataset.net || button.dataset.network || kgwBridgeCurrentVisibleNetwork(root);`;

  const b = functionBounds(text, "installActions");
  let body = b.body;

  body = removeRootListeners(body, "input");
  body = removeRootListeners(body, "change");

  body = body.replace(
    /function\s+installActions\s*\(\s*root\s*\)\s*\{/,
    (m) => `${m}\n  ${prefix}SettingsOwnerR12Install(root);\n`
  );

  const makeBranch = (action) => `    if (action === "${action}") {
      ${netLine}
      ${prefix}SettingsOwnerR12HandleAction(net, action, button);
      return;
    }`;

  body = replaceActionBranch(body, "save-settings", makeBranch("save-settings"));
  body = replaceActionBranch(body, "restore-defaults", makeBranch("restore-defaults"));
  body = replaceActionBranch(body, "set-defaults", makeBranch("set-defaults"));

  return text.slice(0, b.start) + body + text.slice(b.end);
}

function patchNodeOwners(text) {
  text = replaceFunction(text, "kgwNodeR51SaveSettings", `function kgwNodeR51SaveSettings(net) {
  kgwNodeR51Store("saved:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Node settings saved successfully.");
}`);

  text = replaceFunction(text, "kgwNodeR51SetAsDefaults", `function kgwNodeR51SetAsDefaults(net) {
  kgwNodeR51Store("default:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Current node settings saved as defaults.");
}`);

  text = replaceFunction(text, "kgwNodeR51RestoreDefaults", `function kgwNodeR51RestoreDefaults(net) {
  const defaults = kgwNodeR51Load("default:" + net) || kgwNodeR51Load("factory:" + net);
  kgwNodeR51WriteSettings(net, defaults);
  kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  updateAllCommands();
  appendLog(net, "Node defaults restored successfully.");
}`);

  return text;
}

function patchBridgeOwners(text) {
  text = replaceFunction(text, "kgwBridgeR51SaveSettings", `function kgwBridgeR51SaveSettings(net) {
  kgwBridgeR51Store("saved:" + net, kgwBridgeR51ReadSettings(net));
}`);

  text = replaceFunction(text, "kgwBridgeR51SetAsDefaults", `function kgwBridgeR51SetAsDefaults(net) {
  kgwBridgeR51Store("default:" + net, kgwBridgeR51ReadSettings(net));
}`);

  text = replaceFunction(text, "kgwBridgeR51RestoreDefaults", `function kgwBridgeR51RestoreDefaults(net) {
  const defaults = kgwBridgeR51Load("default:" + net) || kgwBridgeR51Load("factory:" + net);
  kgwBridgeR51WriteSettings(net, defaults);
  bridgeSyncAllModeControls();
  updateAllCommands();
}`);

  return text;
}

function patchJs(file, kind) {
  const before = read(file);
  let text = before;

  text = stripOldGeneratedBlocks(text);

  const isNode = kind === "node";
  const readFn = isNode ? "kgwNodeR51ReadSettings" : "kgwBridgeR51ReadSettings";
  text = insertBeforeFunction(text, readFn, unifiedOwnerBlock(kind));

  if (isNode) {
    text = patchNodeOwners(text);
  } else {
    text = patchBridgeOwners(text);
  }

  text = patchInstallActions(text, kind);

  write(file, text);

  return {
    changed: before !== text,
    r12Markers: count(text, /KGW_SETTINGS_UNIFIED_OWNER_R12_START/g),
    oldMarkers: count(text, /R4D_START|R4B_START|R6_START|R7_START|R7B_START|R7C_START|R8_START|R9_START|R9B_START|R10|R11_START/g),
    ownerInstallRefs: count(text, /SettingsOwnerR12Install/g),
    handleActionRefs: count(text, /SettingsOwnerR12HandleAction/g),
    trustedEventRefs: count(text, /event\.isTrusted/g),
    feedbackLockRefs: count(text, /lockedUntil|feedbackInterval|10000/g),
  };
}

function cssBlock() {
  return `
/* KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_START */
button.kgw-settings-action-disabled,
button[data-kgw-settings-action-disabled="1"],
button[aria-disabled="true"].kgw-settings-action-disabled,
button:disabled.kgw-settings-action-disabled,
button:disabled[data-kgw-settings-action-disabled="1"] {
  opacity: 0.42 !important;
  filter: grayscale(0.35);
  cursor: not-allowed !important;
  pointer-events: none;
  box-shadow: none !important;
  transform: none !important;
}

button.kgw-settings-action-disabled:hover,
button[data-kgw-settings-action-disabled="1"]:hover,
button[aria-disabled="true"].kgw-settings-action-disabled:hover,
button:disabled.kgw-settings-action-disabled:hover,
button:disabled[data-kgw-settings-action-disabled="1"]:hover {
  opacity: 0.42 !important;
  cursor: not-allowed !important;
  box-shadow: none !important;
  transform: none !important;
}
/* KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_END */
`;
}

function patchCss(file) {
  const before = read(file);
  let text = before;

  text = removeNamedBlock(text, "KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START", "KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_END");
  text = removeNamedBlock(text, "KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_START", "KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_END");
  text = text.trimEnd() + "\n\n" + cssBlock() + "\n";

  write(file, text);

  return {
    changed: before !== text,
    r12VisualMarker: count(text, /KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_START/g),
    opacityImportant: count(text, /opacity:\s*0\.42\s*!important/g),
  };
}

function walk(dir, out = []) {
  if (!fs.existsSync(dir)) return out;

  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);

    if (entry.isDirectory()) {
      if (!["node_modules", "dist", "target", ".git"].includes(entry.name)) walk(p, out);
    } else {
      out.push(p);
    }
  }

  return out;
}

function setDeep(obj, dottedKey, value) {
  const parts = dottedKey.split(".");
  let current = obj;

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!current[part] || typeof current[part] !== "object" || Array.isArray(current[part])) {
      current[part] = {};
    }
    current = current[part];
  }

  current[parts[parts.length - 1]] = value;
}

function patchLocaleJsonFiles() {
  const translations = {
    en: {
      "settings.feedback.saved": "Saved",
      "settings.feedback.restored": "Restored",
      "settings.feedback.setAsDefaults": "Set as defaults",
    },
    ar: {
      "settings.feedback.saved": "تم الحفظ",
      "settings.feedback.restored": "تمت الاستعادة",
      "settings.feedback.setAsDefaults": "تم الضبط كافتراضي",
    },
    de: {
      "settings.feedback.saved": "Gespeichert",
      "settings.feedback.restored": "Wiederhergestellt",
      "settings.feedback.setAsDefaults": "Als Standard festgelegt",
    },
    es: {
      "settings.feedback.saved": "Guardado",
      "settings.feedback.restored": "Restaurado",
      "settings.feedback.setAsDefaults": "Establecido como predeterminado",
    },
    fr: {
      "settings.feedback.saved": "Enregistré",
      "settings.feedback.restored": "Restauré",
      "settings.feedback.setAsDefaults": "Défini par défaut",
    },
  };

  const jsonFiles = walk(files.frontendRoot)
    .filter((file) => file.endsWith(".json"))
    .filter((file) => /locale|locales|i18n|lang|language|translations/i.test(file));

  const patched = [];

  for (const file of jsonFiles) {
    const base = path.basename(file, ".json").toLowerCase();
    const lang = Object.keys(translations).find((code) => base === code || base.startsWith(code + "-") || base.includes("." + code));
    if (!lang) continue;

    let data;
    try {
      data = JSON.parse(read(file));
    } catch {
      continue;
    }

    const before = JSON.stringify(data);

    for (const [key, value] of Object.entries(translations[lang])) {
      setDeep(data, key, value);
    }

    if (before !== JSON.stringify(data)) {
      write(file, JSON.stringify(data, null, 2) + "\n");
      patched.push({ file, lang, keys: Object.keys(translations[lang]) });
    }
  }

  return { candidates: jsonFiles, patched };
}

function auditAfter() {
  const node = read(files.nodeJs);
  const bridge = read(files.bridgeJs);
  const nodeCss = read(files.nodeCss);
  const bridgeCss = read(files.bridgeCss);
  const lib = read(files.lib);

  const nodeInstall = functionBounds(node, "installActions").body;
  const bridgeInstall = functionBounds(bridge, "installActions").body;

  return {
    node: {
      r12Markers: count(node, /KGW_SETTINGS_UNIFIED_OWNER_R12_START/g),
      noOldMarkers: count(node, /R4D_START|R4B_START|R6_START|R7_START|R7B_START|R7C_START|R8_START|R9_START|R9B_START|R11_START/g) === 0,
      installOwner: nodeInstall.includes("kgwNodeSettingsOwnerR12Install(root)"),
      saveBranchOwner: nodeInstall.includes('kgwNodeSettingsOwnerR12HandleAction(net, action, button)'),
      trustedEvents: count(node, /event\.isTrusted/g),
      feedbackTenSeconds: count(node, /10000/g),
      nativeDisabled: functionBounds(node, "kgwNodeSetSettingsActionEnabledR2").body.includes("button.disabled = !finalEnabled"),
      oldDirtyIgnored: functionBounds(node, "kgwNodeUpdateSettingsDirtyButtonsR4D").body.includes("ignored-by-r12"),
    },
    bridge: {
      r12Markers: count(bridge, /KGW_SETTINGS_UNIFIED_OWNER_R12_START/g),
      noOldMarkers: count(bridge, /R4D_START|R4B_START|R6_START|R7_START|R7B_START|R7C_START|R8_START|R9_START|R9B_START|R11_START/g) === 0,
      installOwner: bridgeInstall.includes("kgwBridgeSettingsOwnerR12Install(root)"),
      saveBranchOwner: bridgeInstall.includes('kgwBridgeSettingsOwnerR12HandleAction(net, action, button)'),
      trustedEvents: count(bridge, /event\.isTrusted/g),
      feedbackTenSeconds: count(bridge, /10000/g),
      nativeDisabled: functionBounds(bridge, "kgwBridgeSetSettingsActionEnabledR2").body.includes("button.disabled = !finalEnabled"),
      oldDirtyIgnored: functionBounds(bridge, "kgwBridgeUpdateSettingsDirtyButtonsR4D").body.includes("ignored-by-r12"),
    },
    css: {
      nodeMarker: count(nodeCss, /KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_START/g),
      bridgeMarker: count(bridgeCss, /KGW_SETTINGS_UNIFIED_OWNER_R12_VISUAL_START/g),
      nodeOpacity: count(nodeCss, /opacity:\s*0\.42\s*!important/g),
      bridgeOpacity: count(bridgeCss, /opacity:\s*0\.42\s*!important/g),
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r12.json", {
  nodeLength: read(files.nodeJs).length,
  bridgeLength: read(files.bridgeJs).length,
  nodeCssLength: read(files.nodeCss).length,
  bridgeCssLength: read(files.bridgeCss).length,
});

const changes = {
  rust: ensureRustTraceCommand(),
  nodeJs: patchJs(files.nodeJs, "node"),
  bridgeJs: patchJs(files.bridgeJs, "bridge"),
  nodeCss: patchCss(files.nodeCss),
  bridgeCss: patchCss(files.bridgeCss),
  locale: patchLocaleJsonFiles(),
};

saveJson("patch-changes-r12.json", changes);

const after = auditAfter();
saveJson("audit-after-r12.json", after);

const failures = [];

if (after.node.r12Markers !== 1) failures.push("Node R12 marker must exist exactly once.");
if (after.bridge.r12Markers !== 1) failures.push("Bridge R12 marker must exist exactly once.");
if (!after.node.noOldMarkers) failures.push("Node still has old generated owner markers.");
if (!after.bridge.noOldMarkers) failures.push("Bridge still has old generated owner markers.");
if (!after.node.installOwner) failures.push("Node installActions does not install R12 owner.");
if (!after.bridge.installOwner) failures.push("Bridge installActions does not install R12 owner.");
if (!after.node.saveBranchOwner) failures.push("Node settings action branches do not route to R12.");
if (!after.bridge.saveBranchOwner) failures.push("Bridge settings action branches do not route to R12.");
if (after.node.trustedEvents < 2) failures.push("Node trusted input/change handling missing.");
if (after.bridge.trustedEvents < 2) failures.push("Bridge trusted input/change handling missing.");
if (after.node.feedbackTenSeconds < 1) failures.push("Node 10s feedback missing.");
if (after.bridge.feedbackTenSeconds < 1) failures.push("Bridge 10s feedback missing.");
if (!after.node.nativeDisabled) failures.push("Node native disabled ownership missing.");
if (!after.bridge.nativeDisabled) failures.push("Bridge native disabled ownership missing.");
if (!after.node.oldDirtyIgnored) failures.push("Node old dirty owner is not ignored.");
if (!after.bridge.oldDirtyIgnored) failures.push("Bridge old dirty owner is not ignored.");
if (after.css.nodeMarker !== 1 || after.css.bridgeMarker !== 1) failures.push("R12 disabled visual CSS missing.");
if (after.css.nodeOpacity < 1 || after.css.bridgeOpacity < 1) failures.push("R12 disabled opacity missing.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R12 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R12 unified owner rebuild complete");
console.log(JSON.stringify(after, null, 2));