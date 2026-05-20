const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node r4d.cjs <repoRoot> <reportDir>");
}

const files = {
  node: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridge: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  lib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
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

function stripGeneratedBlocks(text) {
  const before = text;
  const markers = [
    "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B"
  ];

  for (const marker of markers) {
    const re = new RegExp("\\n?\\s*// " + marker + "_START[\\s\\S]*?// " + marker + "_END\\s*\\n?", "g");
    text = text.replace(re, "\n");
  }

  text = text.replace(/\/\* KGW [A-Z]+ settings buttons trace and dirty-state owner R4B\.[\s\S]*?\*\//g, "");
  text = text.replace(/\n{4,}/g, "\n\n\n");

  return { text, changed: before !== text };
}

function ensureRustTrace(file) {
  let text = read(file);
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
    if (insertAt < 0) {
      throw new Error("No tauri command insertion point found in lib.rs");
    }

    text = text.slice(0, insertAt) + command + "\n" + text.slice(insertAt);
  }

  const handlerRe = /tauri::generate_handler!\s*\[([\s\S]*?)\]/m;
  const m = text.match(handlerRe);
  if (!m) {
    throw new Error("Could not find tauri::generate_handler![...] in lib.rs");
  }

  const names = m[1].split(",").map((x) => x.trim()).filter(Boolean);
  if (!names.includes("kgw_frontend_button_trace_v1")) {
    names.push("kgw_frontend_button_trace_v1");
  }

  text = text.replace(handlerRe, "tauri::generate_handler![\n            " + names.join(",\n            ") + "\n        ]");

  write(file, text);

  return {
    changed: before !== text,
    commandCount: count(text, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
    tracePrintCount: count(text, /\[KGW_BUTTON_TRACE\]/g),
    handlerRefs: count(text, /kgw_frontend_button_trace_v1/g)
  };
}

function nodeHelperBlock() {
  return `
/* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START */
function kgwNodeSettingsTraceR4D(net, action, phase, details = {}) {
  const payload = {
    scope: "node",
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
    // Trace must never break UI actions.
  }

  try {
    console.log("[KGW_BUTTON_TRACE]", payload);
  } catch {
    // Ignore console failures.
  }
}

function kgwNodeCurrentLanguageR4D() {
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

function kgwNodeSettingsActionLabelR4D(action, fallback) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = kgwNodeCurrentLanguageR4D();
  return labels[lang]?.[action] || fallback || action;
}

function kgwNodeSettingsButtonStateR4D(net, action) {
  const buttons = Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`));
  return buttons.map((button) => ({
    text: String(button.textContent || "").trim(),
    disabled: Boolean(button.disabled),
    ariaDisabled: button.getAttribute("aria-disabled"),
  }));
}

function kgwNodeUpdateSettingsDirtyButtonsR4D(net, reason = "unknown") {
  const current = kgwNodeR51ReadSettings(net);
  const saved = kgwNodeReferenceSettingsR2(net, "saved");
  const defaults = kgwNodeReferenceSettingsR2(net, "default");

  const differsFromSaved = !kgwNodeSettingsEqualR2(current, saved);
  const differsFromDefault = !kgwNodeSettingsEqualR2(current, defaults);

  kgwNodeSetSettingsActionEnabledR2(net, "save-settings", differsFromSaved);
  kgwNodeSetSettingsActionEnabledR2(net, "set-defaults", differsFromDefault);
  kgwNodeSetSettingsActionEnabledR2(net, "restore-defaults", differsFromDefault);

  kgwNodeSettingsTraceR4D(net, "dirty", "dirty-evaluated", {
    reason,
    differsFromSaved,
    differsFromDefault,
    currentFields: Object.keys(current || {}).length,
    savedFields: Object.keys(saved || {}).length,
    defaultFields: Object.keys(defaults || {}).length,
    saveButtons: kgwNodeSettingsButtonStateR4D(net, "save-settings"),
    restoreButtons: kgwNodeSettingsButtonStateR4D(net, "restore-defaults"),
    setDefaultButtons: kgwNodeSettingsButtonStateR4D(net, "set-defaults"),
  });
}

function kgwNodeUpdateAllSettingsDirtyButtonsR4D(reason = "unknown") {
  if (typeof NODE_NETWORKS === "undefined") return;
  NODE_NETWORKS.forEach((net) => kgwNodeUpdateSettingsDirtyButtonsR4D(net.key, reason));
}
/* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END */

`;
}

function bridgeHelperBlock() {
  return `
/* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START */
function kgwBridgeSettingsTraceR4D(net, action, phase, details = {}) {
  const payload = {
    scope: "bridge",
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
    // Trace must never break UI actions.
  }

  try {
    console.log("[KGW_BUTTON_TRACE]", payload);
  } catch {
    // Ignore console failures.
  }
}

function kgwBridgeCurrentLanguageR4D() {
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

function kgwBridgeSettingsActionLabelR4D(action, fallback) {
  const labels = {
    en: { "save-settings": "Saved", "restore-defaults": "Restored", "set-defaults": "Set as defaults" },
    ar: { "save-settings": "تم الحفظ", "restore-defaults": "تمت الاستعادة", "set-defaults": "تم الضبط كافتراضي" },
    de: { "save-settings": "Gespeichert", "restore-defaults": "Wiederhergestellt", "set-defaults": "Als Standard festgelegt" },
    es: { "save-settings": "Guardado", "restore-defaults": "Restaurado", "set-defaults": "Establecido como predeterminado" },
    fr: { "save-settings": "Enregistré", "restore-defaults": "Restauré", "set-defaults": "Défini par défaut" },
  };

  const lang = kgwBridgeCurrentLanguageR4D();
  return labels[lang]?.[action] || fallback || action;
}

function kgwBridgeSettingsButtonStateR4D(net, action) {
  const buttons = Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`));
  return buttons.map((button) => ({
    text: String(button.textContent || "").trim(),
    disabled: Boolean(button.disabled),
    ariaDisabled: button.getAttribute("aria-disabled"),
  }));
}

function kgwBridgeUpdateSettingsDirtyButtonsR4D(net, reason = "unknown") {
  const current = kgwBridgeR51ReadSettings(net);
  const saved = kgwBridgeReferenceSettingsR2(net, "saved");
  const defaults = kgwBridgeReferenceSettingsR2(net, "default");

  const differsFromSaved = !kgwBridgeSettingsEqualR2(current, saved);
  const differsFromDefault = !kgwBridgeSettingsEqualR2(current, defaults);

  kgwBridgeSetSettingsActionEnabledR2(net, "save-settings", differsFromSaved);
  kgwBridgeSetSettingsActionEnabledR2(net, "set-defaults", differsFromDefault);
  kgwBridgeSetSettingsActionEnabledR2(net, "restore-defaults", differsFromDefault);

  kgwBridgeSettingsTraceR4D(net, "dirty", "dirty-evaluated", {
    reason,
    differsFromSaved,
    differsFromDefault,
    currentFields: Object.keys(current || {}).length,
    savedFields: Object.keys(saved || {}).length,
    defaultFields: Object.keys(defaults || {}).length,
    saveButtons: kgwBridgeSettingsButtonStateR4D(net, "save-settings"),
    restoreButtons: kgwBridgeSettingsButtonStateR4D(net, "restore-defaults"),
    setDefaultButtons: kgwBridgeSettingsButtonStateR4D(net, "set-defaults"),
  });
}

function kgwBridgeUpdateAllSettingsDirtyButtonsR4D(reason = "unknown") {
  if (typeof BRIDGE_NETWORKS === "undefined") return;
  BRIDGE_NETWORKS.forEach((net) => kgwBridgeUpdateSettingsDirtyButtonsR4D(net.key, reason));
}
/* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END */

`;
}

function insertOrReplaceBlock(text, startMarker, endMarker, block, beforeNeedle) {
  const re = new RegExp("\\n?/\\* " + startMarker + " \\*/[\\s\\S]*?/\\* " + endMarker + " \\*/\\n?", "g");
  text = text.replace(re, "\n");

  if (text.includes(startMarker) || text.includes(endMarker)) {
    const simpleRe = new RegExp("\\n?[\\s\\S]*?" + startMarker + "[\\s\\S]*?" + endMarker + "[\\s\\S]*?\\n?", "g");
    text = text.replace(simpleRe, "\n");
  }

  const idx = text.indexOf(beforeNeedle);
  if (idx < 0) {
    throw new Error("Could not find insertion point: " + beforeNeedle);
  }

  return text.slice(0, idx) + block + text.slice(idx);
}

function replaceRequired(text, from, to, label) {
  if (!text.includes(from)) {
    throw new Error("Missing required replacement target: " + label);
  }
  return text.split(from).join(to);
}

function patchNode(file) {
  let text = read(file);
  const before = text;

  text = stripGeneratedBlocks(text).text;

  text = text.replace(/\n?\/\* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START \*\/[\s\S]*?\/\* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END \*\/\n?/g, "\n");
  text = text.replace(/\n?\/\* KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START \*\/[\s\S]*?KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END \*\/\n?/g, "\n");

  const insertAt = "function installActions(root) {";
  if (!text.includes("KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START")) {
    const idx = text.indexOf(insertAt);
    if (idx < 0) throw new Error("Node installActions not found.");
    text = text.slice(0, idx) + nodeHelperBlock() + text.slice(idx);
  }

  text = replaceRequired(
    text,
    "kgwNodeUpdateAllSettingsDirtyButtonsR2();",
    "kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");",
    "Node dirty update call replacement"
  );

  text = replaceRequired(
    text,
    "    const action = button.dataset.nodeAction;\n    const net = button.dataset.net;\n",
    "    const action = button.dataset.nodeAction;\n    const net = button.dataset.net;\n\n    kgwNodeSettingsTraceR4D(net, action, \"click-received\", {\n      disabled: Boolean(button.disabled),\n      ariaDisabled: button.getAttribute(\"aria-disabled\"),\n      text: String(button.textContent || \"\").trim(),\n    });\n",
    "Node click trace insertion"
  );

  text = replaceRequired(
    text,
    "    if (action === \"save-settings\") {\n      kgwNodeR51SaveSettings(net);\n      kgwNodeFlashSettingsActionButtonR2(button, \"تم الحفظ\");\n      kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");\n      return;\n    }\n",
    "    if (action === \"save-settings\") {\n      kgwNodeSettingsTraceR4D(net, action, \"action-start\", { beforeDisabled: Boolean(button.disabled) });\n      kgwNodeR51SaveSettings(net);\n      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, \"Saved\"));\n      kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"after-save-settings\");\n      kgwNodeSettingsTraceR4D(net, action, \"action-complete\", { label: kgwNodeSettingsActionLabelR4D(action, \"Saved\") });\n      return;\n    }\n",
    "Node save action branch"
  );

  text = replaceRequired(
    text,
    "    if (action === \"restore-defaults\") {\n      kgwNodeR51RestoreDefaults(net);\n      kgwNodeFlashSettingsActionButtonR2(button, \"تمت الاستعادة\");\n      kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");\n      return;\n    }\n",
    "    if (action === \"restore-defaults\") {\n      kgwNodeSettingsTraceR4D(net, action, \"action-start\", { beforeDisabled: Boolean(button.disabled) });\n      kgwNodeR51RestoreDefaults(net);\n      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, \"Restored\"));\n      kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"after-restore-defaults\");\n      window.setTimeout(() => kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"after-restore-defaults+900ms\"), 900);\n      kgwNodeSettingsTraceR4D(net, action, \"action-complete\", { label: kgwNodeSettingsActionLabelR4D(action, \"Restored\") });\n      return;\n    }\n",
    "Node restore action branch"
  );

  text = replaceRequired(
    text,
    "    if (action === \"set-defaults\") {\n      kgwNodeR51SetAsDefaults(net);\n      kgwNodeFlashSettingsActionButtonR2(button, \"تم الضبط كافتراضي\");\n      kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");\n      return;\n    }\n",
    "    if (action === \"set-defaults\") {\n      kgwNodeSettingsTraceR4D(net, action, \"action-start\", { beforeDisabled: Boolean(button.disabled) });\n      kgwNodeR51SetAsDefaults(net);\n      kgwNodeFlashSettingsActionButtonR2(button, kgwNodeSettingsActionLabelR4D(action, \"Set as defaults\"));\n      kgwNodeUpdateAllSettingsDirtyButtonsR4D(\"after-set-defaults\");\n      kgwNodeSettingsTraceR4D(net, action, \"action-complete\", { label: kgwNodeSettingsActionLabelR4D(action, \"Set as defaults\") });\n      return;\n    }\n",
    "Node set defaults action branch"
  );

  write(file, text);

  return {
    changed: before !== text,
    r4bMarkers: count(text, /R4B_START/g),
    r4dMarkers: count(text, /KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START/g),
    noisyScanRefs: count(text, /buttons-discovered|dirty-evaluated.*scan|MutationObserver/g),
    traceRefs: count(text, /kgwNodeSettingsTraceR4D/g),
    hardcodedArabicCalls: count(text, /kgwNodeFlashSettingsActionButtonR2\(button,\s*"تم/g)
  };
}

function patchBridge(file) {
  let text = read(file);
  const before = text;

  text = stripGeneratedBlocks(text).text;

  text = text.replace(/\n?\/\* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START \*\/[\s\S]*?\/\* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END \*\/\n?/g, "\n");
  text = text.replace(/\n?\/\* KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START \*\/[\s\S]*?KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_END \*\/\n?/g, "\n");

  const insertAt = "function installActions(root) {";
  if (!text.includes("KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START")) {
    const idx = text.indexOf(insertAt);
    if (idx < 0) throw new Error("Bridge installActions not found.");
    text = text.slice(0, idx) + bridgeHelperBlock() + text.slice(idx);
  }

  text = replaceRequired(
    text,
    "kgwBridgeUpdateAllSettingsDirtyButtonsR2();",
    "kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");",
    "Bridge dirty update call replacement"
  );

  text = replaceRequired(
    text,
    "    const action = button.dataset.bridgeAction;\n    const net = button.dataset.net || button.dataset.network || kgwBridgeCurrentVisibleNetwork(root);\n    const instanceId = button.dataset.instanceId;\n",
    "    const action = button.dataset.bridgeAction;\n    const net = button.dataset.net || button.dataset.network || kgwBridgeCurrentVisibleNetwork(root);\n    const instanceId = button.dataset.instanceId;\n\n    kgwBridgeSettingsTraceR4D(net, action, \"click-received\", {\n      disabled: Boolean(button.disabled),\n      ariaDisabled: button.getAttribute(\"aria-disabled\"),\n      text: String(button.textContent || \"\").trim(),\n      instanceId,\n    });\n",
    "Bridge click trace insertion"
  );

  text = replaceRequired(
    text,
    "    bridgeSyncAllModeControls();\n    updateAllCommands();\n  });\n\n  root.addEventListener(\"click\", async (event) => {",
    "    bridgeSyncAllModeControls();\n    updateAllCommands();\n    kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"change\");\n  });\n\n  root.addEventListener(\"click\", async (event) => {",
    "Bridge change dirty update insertion"
  );

  text = replaceRequired(
    text,
    "    if (action === \"save-settings\") {\n      kgwBridgeR51SaveSettings(net);\n      kgwBridgeFlashSettingsActionButtonR2(button, \"تم الحفظ\");\n      kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");\n      return;\n    }\n",
    "    if (action === \"save-settings\") {\n      kgwBridgeSettingsTraceR4D(net, action, \"action-start\", { beforeDisabled: Boolean(button.disabled) });\n      kgwBridgeR51SaveSettings(net);\n      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, \"Saved\"));\n      kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"after-save-settings\");\n      kgwBridgeSettingsTraceR4D(net, action, \"action-complete\", { label: kgwBridgeSettingsActionLabelR4D(action, \"Saved\") });\n      return;\n    }\n",
    "Bridge save action branch"
  );

  text = replaceRequired(
    text,
    "    if (action === \"restore-defaults\") {\n      kgwBridgeR51RestoreDefaults(net);\n      kgwBridgeFlashSettingsActionButtonR2(button, \"تمت الاستعادة\");\n      kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");\n      return;\n    }\n",
    "    if (action === \"restore-defaults\") {\n      kgwBridgeSettingsTraceR4D(net, action, \"action-start\", { beforeDisabled: Boolean(button.disabled) });\n      kgwBridgeR51RestoreDefaults(net);\n      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, \"Restored\"));\n      kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"after-restore-defaults\");\n      window.setTimeout(() => kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"after-restore-defaults+900ms\"), 900);\n      kgwBridgeSettingsTraceR4D(net, action, \"action-complete\", { label: kgwBridgeSettingsActionLabelR4D(action, \"Restored\") });\n      return;\n    }\n",
    "Bridge restore action branch"
  );

  text = replaceRequired(
    text,
    "    if (action === \"set-defaults\") {\n      kgwBridgeR51SetAsDefaults(net);\n      kgwBridgeFlashSettingsActionButtonR2(button, \"تم الضبط كافتراضي\");\n      kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"legacy-call\");\n      return;\n    }\n",
    "    if (action === \"set-defaults\") {\n      kgwBridgeSettingsTraceR4D(net, action, \"action-start\", { beforeDisabled: Boolean(button.disabled) });\n      kgwBridgeR51SetAsDefaults(net);\n      kgwBridgeFlashSettingsActionButtonR2(button, kgwBridgeSettingsActionLabelR4D(action, \"Set as defaults\"));\n      kgwBridgeUpdateAllSettingsDirtyButtonsR4D(\"after-set-defaults\");\n      kgwBridgeSettingsTraceR4D(net, action, \"action-complete\", { label: kgwBridgeSettingsActionLabelR4D(action, \"Set as defaults\") });\n      return;\n    }\n",
    "Bridge set defaults action branch"
  );

  write(file, text);

  return {
    changed: before !== text,
    r4bMarkers: count(text, /R4B_START/g),
    r4dMarkers: count(text, /KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START/g),
    noisyScanRefs: count(text, /buttons-discovered|dirty-evaluated.*scan|MutationObserver/g),
    traceRefs: count(text, /kgwBridgeSettingsTraceR4D/g),
    hardcodedArabicCalls: count(text, /kgwBridgeFlashSettingsActionButtonR2\(button,\s*"تم/g)
  };
}

function auditAfter() {
  const node = read(files.node);
  const bridge = read(files.bridge);
  const lib = read(files.lib);

  return {
    node: {
      r4bMarkers: count(node, /R4B_START/g),
      r4dMarkers: count(node, /KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START/g),
      nodeTraceRefs: count(node, /kgwNodeSettingsTraceR4D/g),
      nodeDirtyR4DRefs: count(node, /kgwNodeUpdateAllSettingsDirtyButtonsR4D/g),
      hardcodedArabicSettingsFeedbackCalls: count(node, /kgwNodeFlashSettingsActionButtonR2\(button,\s*"تم/g),
      noisyScannerRefs: count(node, /buttons-discovered|KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START/g)
    },
    bridge: {
      r4bMarkers: count(bridge, /R4B_START/g),
      r4dMarkers: count(bridge, /KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_R4D_START/g),
      bridgeTraceRefs: count(bridge, /kgwBridgeSettingsTraceR4D/g),
      bridgeDirtyR4DRefs: count(bridge, /kgwBridgeUpdateAllSettingsDirtyButtonsR4D/g),
      hardcodedArabicSettingsFeedbackCalls: count(bridge, /kgwBridgeFlashSettingsActionButtonR2\(button,\s*"تم/g),
      noisyScannerRefs: count(bridge, /buttons-discovered|KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START/g)
    },
    lib: {
      commandCount: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrintCount: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g)
    }
  };
}

const before = {
  nodeR4B: count(read(files.node), /R4B_START/g),
  bridgeR4B: count(read(files.bridge), /R4B_START/g),
  rustCommand: count(read(files.lib), /fn\s+kgw_frontend_button_trace_v1\s*\(/g)
};

saveJson("audit-before-r4d.json", before);

const changes = {
  rust: ensureRustTrace(files.lib),
  node: patchNode(files.node),
  bridge: patchBridge(files.bridge)
};

saveJson("patch-changes-r4d.json", changes);

const after = auditAfter();
saveJson("audit-after-r4d.json", after);

const failures = [];

if (after.node.r4bMarkers !== 0) failures.push("Node still contains R4B markers.");
if (after.bridge.r4bMarkers !== 0) failures.push("Bridge still contains R4B markers.");
if (after.node.r4dMarkers !== 1) failures.push("Node must contain exactly one R4D marker.");
if (after.bridge.r4dMarkers !== 1) failures.push("Bridge must contain exactly one R4D marker.");
if (after.node.hardcodedArabicSettingsFeedbackCalls !== 0) failures.push("Node still has hard-coded Arabic settings feedback calls.");
if (after.bridge.hardcodedArabicSettingsFeedbackCalls !== 0) failures.push("Bridge still has hard-coded Arabic settings feedback calls.");
if (after.lib.commandCount !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.lib.tracePrintCount !== 1) failures.push("Rust KGW_BUTTON_TRACE println marker must exist exactly once.");
if (after.lib.handlerRefs < 2) failures.push("Rust trace command must be defined and registered.");

if (failures.length) {
  throw new Error("R4D validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R4D patch complete");
console.log(JSON.stringify(after, null, 2));