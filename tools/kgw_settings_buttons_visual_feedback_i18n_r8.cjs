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

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function appendOrReplaceCss(text) {
  const block = `
/* KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START */
button.kgw-settings-action-disabled,
button[data-kgw-settings-action-disabled="1"],
button[aria-disabled="true"].kgw-settings-action-disabled,
button:disabled.kgw-settings-action-disabled {
  opacity: 0.42;
  filter: grayscale(0.35);
  cursor: not-allowed;
  pointer-events: none;
  box-shadow: none;
  transform: none;
}

button.kgw-settings-action-disabled:hover,
button[data-kgw-settings-action-disabled="1"]:hover,
button[aria-disabled="true"].kgw-settings-action-disabled:hover,
button:disabled.kgw-settings-action-disabled:hover {
  opacity: 0.42;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}
/* KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_END */
`;

  text = removeBlock(text, "KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START", "KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_END");
  return text.trimEnd() + "\n\n" + block + "\n";
}

function patchCss(file) {
  const before = read(file);
  const after = appendOrReplaceCss(before);
  write(file, after);

  return {
    changed: before !== after,
    markerCount: count(after, /KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START/g),
    opacityRefs: count(after, /opacity:\s*0\.42/g),
    disabledClassRefs: count(after, /kgw-settings-action-disabled/g),
  };
}

function nodeFlashFunctionR8() {
  return `function kgwNodeFlashSettingsActionButtonR2(button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const action = button.dataset.nodeAction;
  const net = button.dataset.net || "";

  button.dataset.kgwOriginalLabel = original;

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.dataset.kgwFeedbackActive = "1";
    target.dataset.kgwFeedbackUntil = String(Date.now() + 10000);
    target.textContent = label;
  });

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR8);
  button.__kgwSettingsFeedbackTimerR8 = window.setTimeout(() => {
    targets.forEach((target) => {
      target.dataset.kgwFeedbackActive = "0";
      target.dataset.kgwFeedbackUntil = "0";
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });
  }, 10000);
}`;
}

function bridgeFlashFunctionR8() {
  return `function kgwBridgeFlashSettingsActionButtonR2(button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const action = button.dataset.bridgeAction;
  const net = button.dataset.net || button.dataset.network || "";

  button.dataset.kgwOriginalLabel = original;

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.dataset.kgwFeedbackActive = "1";
    target.dataset.kgwFeedbackUntil = String(Date.now() + 10000);
    target.textContent = label;
  });

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR8);
  button.__kgwSettingsFeedbackTimerR8 = window.setTimeout(() => {
    targets.forEach((target) => {
      target.dataset.kgwFeedbackActive = "0";
      target.dataset.kgwFeedbackUntil = "0";
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });
  }, 10000);
}`;
}

function nodeSetEnabledFunctionR8() {
  return `function kgwNodeSetSettingsActionEnabledR2(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    const feedbackActive = button.dataset.kgwFeedbackActive === "1" && Number(button.dataset.kgwFeedbackUntil || "0") > Date.now();

    button.disabled = !enabled;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);

    if (feedbackActive) {
      button.dataset.kgwFeedbackPreservedByR8 = "1";
    }
  });
}`;
}

function bridgeSetEnabledFunctionR8() {
  return `function kgwBridgeSetSettingsActionEnabledR2(net, action, enabled) {
  const buttons = Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    const feedbackActive = button.dataset.kgwFeedbackActive === "1" && Number(button.dataset.kgwFeedbackUntil || "0") > Date.now();

    button.disabled = !enabled;
    button.setAttribute("aria-disabled", enabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = enabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !enabled);

    if (feedbackActive) {
      button.dataset.kgwFeedbackPreservedByR8 = "1";
    }
  });
}`;
}

function patchTranslatorFunction(text, name, kind) {
  const fn = functionBounds(text, name).body;
  const isNode = kind === "node";
  const fnName = isNode ? "kgwNodeSettingsFeedbackLabelR7C" : "kgwBridgeSettingsFeedbackLabelR7C";
  const langFn = isNode ? "kgwNodeSettingsLanguageR7C" : "kgwBridgeSettingsLanguageR7C";

  const replacement = `function ${fnName}(action) {
  const keyMap = {
    "save-settings": "settings.feedback.saved",
    "restore-defaults": "settings.feedback.restored",
    "set-defaults": "settings.feedback.setAsDefaults",
  };

  const key = keyMap[action];

  if (key) {
    const candidates = [
      window.kgwT,
      window.t,
      window.__kgwT,
      window.__t,
      window.kgwI18n?.t,
      window.KGW_I18N?.t,
      window.i18n?.t,
      window.__kgwI18n?.t,
    ].filter((candidate) => typeof candidate === "function");

    for (const translate of candidates) {
      try {
        const value = translate(key);
        if (value && value !== key) return value;
      } catch {
        // Continue to the fallback labels below.
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

  const lang = ${langFn}();
  return labels[lang]?.[action] || labels.en[action] || action;
}`;

  return replaceFunction(text, name, replacement);
}

function patchJs(file, kind) {
  const before = read(file);
  let text = before;

  if (kind === "node") {
    text = replaceFunction(text, "kgwNodeFlashSettingsActionButtonR2", nodeFlashFunctionR8());
    text = replaceFunction(text, "kgwNodeSetSettingsActionEnabledR2", nodeSetEnabledFunctionR8());
    text = patchTranslatorFunction(text, "kgwNodeSettingsFeedbackLabelR7C", "node");
  } else {
    text = replaceFunction(text, "kgwBridgeFlashSettingsActionButtonR2", bridgeFlashFunctionR8());
    text = replaceFunction(text, "kgwBridgeSetSettingsActionEnabledR2", bridgeSetEnabledFunctionR8());
    text = patchTranslatorFunction(text, "kgwBridgeSettingsFeedbackLabelR7C", "bridge");
  }

  write(file, text);

  return {
    changed: before !== text,
    feedbackLockRefs: count(text, /kgwFeedbackActive|kgwFeedbackUntil|kgwFeedbackPreservedByR8/g),
    feedbackTimerTenSeconds: count(text, /10000/g),
    translatorKeyRefs: count(text, /settings\.feedback\.(saved|restored|setAsDefaults)/g),
    nativeDisabledByEnabled: count(text, /button\.disabled\s*=\s*!enabled/g),
    fadedClassRefs: count(text, /kgw-settings-action-disabled/g),
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

    const after = JSON.stringify(data, null, 2) + "\n";

    if (before !== JSON.stringify(data)) {
      write(file, after);
      patched.push({ file, lang, keys: Object.keys(translations[lang]) });
    }
  }

  return {
    jsonLocaleCandidates: jsonFiles,
    patched,
  };
}

function auditAfter(localeResult) {
  const nodeJs = read(files.nodeJs);
  const bridgeJs = read(files.bridgeJs);
  const nodeCss = read(files.nodeCss);
  const bridgeCss = read(files.bridgeCss);
  const lib = read(files.lib);

  return {
    nodeJs: {
      feedbackLockRefs: count(nodeJs, /kgwFeedbackActive|kgwFeedbackUntil|kgwFeedbackPreservedByR8/g),
      translatorKeyRefs: count(nodeJs, /settings\.feedback\.(saved|restored|setAsDefaults)/g),
      nativeDisabledByEnabled: count(nodeJs, /function\s+kgwNodeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      noForceDisabledFalse: !functionBounds(nodeJs, "kgwNodeSetSettingsActionEnabledR2").body.includes("button.disabled = false"),
      feedbackTenSeconds: functionBounds(nodeJs, "kgwNodeFlashSettingsActionButtonR2").body.includes("10000"),
    },
    bridgeJs: {
      feedbackLockRefs: count(bridgeJs, /kgwFeedbackActive|kgwFeedbackUntil|kgwFeedbackPreservedByR8/g),
      translatorKeyRefs: count(bridgeJs, /settings\.feedback\.(saved|restored|setAsDefaults)/g),
      nativeDisabledByEnabled: count(bridgeJs, /function\s+kgwBridgeSetSettingsActionEnabledR2[\s\S]*?button\.disabled\s*=\s*!enabled[\s\S]*?\}/g),
      noForceDisabledFalse: !functionBounds(bridgeJs, "kgwBridgeSetSettingsActionEnabledR2").body.includes("button.disabled = false"),
      feedbackTenSeconds: functionBounds(bridgeJs, "kgwBridgeFlashSettingsActionButtonR2").body.includes("10000"),
    },
    nodeCss: {
      marker: count(nodeCss, /KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START/g),
      opacityRefs: count(nodeCss, /opacity:\s*0\.42/g),
      classRefs: count(nodeCss, /kgw-settings-action-disabled/g),
    },
    bridgeCss: {
      marker: count(bridgeCss, /KGW_SETTINGS_ACTION_DISABLED_VISUAL_R8_START/g),
      opacityRefs: count(bridgeCss, /opacity:\s*0\.42/g),
      classRefs: count(bridgeCss, /kgw-settings-action-disabled/g),
    },
    locale: localeResult,
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r8.json", {
  nodeJsLength: read(files.nodeJs).length,
  bridgeJsLength: read(files.bridgeJs).length,
  nodeCssLength: read(files.nodeCss).length,
  bridgeCssLength: read(files.bridgeCss).length,
});

const changes = {
  nodeJs: patchJs(files.nodeJs, "node"),
  bridgeJs: patchJs(files.bridgeJs, "bridge"),
  nodeCss: patchCss(files.nodeCss),
  bridgeCss: patchCss(files.bridgeCss),
};

const localeResult = patchLocaleJsonFiles();
changes.locale = localeResult;

saveJson("patch-changes-r8.json", changes);

const after = auditAfter(localeResult);
saveJson("audit-after-r8.json", after);

const failures = [];

if (after.nodeJs.feedbackLockRefs < 3) failures.push("Node feedback lock refs missing.");
if (after.bridgeJs.feedbackLockRefs < 3) failures.push("Bridge feedback lock refs missing.");
if (after.nodeJs.translatorKeyRefs < 3) failures.push("Node i18n feedback key refs missing.");
if (after.bridgeJs.translatorKeyRefs < 3) failures.push("Bridge i18n feedback key refs missing.");
if (after.nodeJs.nativeDisabledByEnabled < 1) failures.push("Node must keep native disabled=!enabled.");
if (after.bridgeJs.nativeDisabledByEnabled < 1) failures.push("Bridge must keep native disabled=!enabled.");
if (!after.nodeJs.noForceDisabledFalse) failures.push("Node set-enabled owner still forces disabled=false.");
if (!after.bridgeJs.noForceDisabledFalse) failures.push("Bridge set-enabled owner still forces disabled=false.");
if (!after.nodeJs.feedbackTenSeconds) failures.push("Node feedback timer must remain 10000ms.");
if (!after.bridgeJs.feedbackTenSeconds) failures.push("Bridge feedback timer must remain 10000ms.");
if (after.nodeCss.marker !== 1 || after.nodeCss.opacityRefs < 2) failures.push("Node CSS disabled visual style missing.");
if (after.bridgeCss.marker !== 1 || after.bridgeCss.opacityRefs < 2) failures.push("Bridge CSS disabled visual style missing.");
if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R8 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R8 patch complete");
console.log(JSON.stringify(after, null, 2));