const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node audit-r5.cjs <repoRoot> <reportDir>");
}

const files = {
  node: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridge: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  settings: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js"),
  main: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/main.js"),
  lib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"),
  gate: path.join(repoRoot, "tools/kgw_global_owner_gate.cjs")
};

function read(file) {
  return fs.existsSync(file) ? fs.readFileSync(file, "utf8") : "";
}

function save(name, text) {
  fs.mkdirSync(reportDir, { recursive: true });
  fs.writeFileSync(path.join(reportDir, name), text, "utf8");
}

function saveJson(name, value) {
  save(name, JSON.stringify(value, null, 2));
}

function count(text, re) {
  const m = text.match(re);
  return m ? m.length : 0;
}

function lineOf(text, index) {
  return text.slice(0, index).split(/\r?\n/).length;
}

function snippet(text, index, before = 900, after = 1700) {
  return text.slice(Math.max(0, index - before), Math.min(text.length, index + after));
}

function findAll(text, re, limit = 200) {
  const out = [];
  let m;
  while ((m = re.exec(text)) !== null) {
    out.push({
      line: lineOf(text, m.index),
      match: m[0].slice(0, 260),
      window: snippet(text, m.index)
    });
    if (out.length >= limit) break;
  }
  return out;
}

function extractFunction(text, functionName) {
  const idx = text.search(new RegExp("function\\s+" + functionName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "\\s*\\("));
  if (idx < 0) return null;

  let brace = text.indexOf("{", idx);
  if (brace < 0) return null;

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

  return {
    name: functionName,
    line: lineOf(text, idx),
    body: text.slice(idx, Math.min(end, idx + 12000))
  };
}

function extractFunctionsByRegex(text, re, limit = 80) {
  const out = [];
  let m;
  while ((m = re.exec(text)) !== null) {
    const name = m[1] || m[0];
    const fn = extractFunction(text, name);
    if (fn) out.push(fn);
    if (out.length >= limit) break;
  }
  return out;
}

function localStorageKeys(text) {
  const keys = new Set();
  const patterns = [
    /localStorage\.getItem\(\s*["'`]([^"'`]+)["'`]\s*\)/g,
    /localStorage\.setItem\(\s*["'`]([^"'`]+)["'`]\s*,/g,
    /localStorage\.removeItem\(\s*["'`]([^"'`]+)["'`]\s*\)/g
  ];

  for (const re of patterns) {
    let m;
    while ((m = re.exec(text)) !== null) keys.add(m[1]);
  }

  return Array.from(keys).sort();
}

function auditTab(kind, text) {
  const prefix = kind === "node" ? "kgwNode" : "kgwBridge";
  const actionAttr = kind === "node" ? "data-node-action" : "data-bridge-action";

  const keyFunctions = [
    `${prefix}R51ReadSettings`,
    `${prefix}R51SaveSettings`,
    `${prefix}R51RestoreDefaults`,
    `${prefix}R51SetAsDefaults`,
    `${prefix}ReferenceSettingsR2`,
    `${prefix}SettingsEqualR2`,
    `${prefix}SetSettingsActionEnabledR2`,
    `${prefix}UpdateSettingsDirtyButtonsR2`,
    `${prefix}UpdateAllSettingsDirtyButtonsR2`,
    `${prefix}UpdateSettingsDirtyButtonsR4D`,
    `${prefix}UpdateAllSettingsDirtyButtonsR4D`,
    `${prefix}FlashSettingsActionButtonR2`,
    `${prefix}SettingsTraceR4D`,
    "installActions"
  ];

  const functions = {};
  for (const fn of keyFunctions) {
    functions[fn] = extractFunction(text, fn);
  }

  const actionButtonOccurrences = findAll(
    text,
    new RegExp(actionAttr + "|save-settings|restore-defaults|set-defaults", "g"),
    160
  );

  const storageKeys = localStorageKeys(text);

  const referenceWindows = findAll(
    text,
    /saved|default|defaults|factory|current|dirty|Restore Defaults|Set as Defaults|Save Settings/g,
    140
  );

  const eventWindows = findAll(
    text,
    /addEventListener\(["'](?:click|change|input|pointerdown|mousedown)["']|dataset\.(?:nodeAction|bridgeAction)|closest\(/g,
    120
  );

  const pathWindows = findAll(
    text,
    /rusty-kaspa|LOCALAPPDATA|appDir|logDir|dataDir|kgw.*path|ApplyRustyKaspaRootOnly|RestoreDefaults|SetAsDefaults/g,
    160
  );

  return {
    kind,
    length: text.length,
    counts: {
      installActions: count(text, /function\s+installActions\s*\(/g),
      readSettings: count(text, new RegExp(prefix + "R51ReadSettings", "g")),
      saveSettings: count(text, new RegExp(prefix + "R51SaveSettings", "g")),
      restoreDefaults: count(text, new RegExp(prefix + "R51RestoreDefaults", "g")),
      setAsDefaults: count(text, new RegExp(prefix + "R51SetAsDefaults", "g")),
      referenceSettingsR2: count(text, new RegExp(prefix + "ReferenceSettingsR2", "g")),
      settingsEqualR2: count(text, new RegExp(prefix + "SettingsEqualR2", "g")),
      setActionEnabledR2: count(text, new RegExp(prefix + "SetSettingsActionEnabledR2", "g")),
      dirtyR2: count(text, new RegExp(prefix + "UpdateSettingsDirtyButtonsR2", "g")),
      dirtyAllR2: count(text, new RegExp(prefix + "UpdateAllSettingsDirtyButtonsR2", "g")),
      dirtyR4D: count(text, new RegExp(prefix + "UpdateSettingsDirtyButtonsR4D", "g")),
      dirtyAllR4D: count(text, new RegExp(prefix + "UpdateAllSettingsDirtyButtonsR4D", "g")),
      flashR2: count(text, new RegExp(prefix + "FlashSettingsActionButtonR2", "g")),
      traceR4D: count(text, new RegExp(prefix + "SettingsTraceR4D", "g")),
      r4bMarkers: count(text, /R4B_START|R4B_END/g),
      r4dMarkers: count(text, /R4D_START|R4D_END/g),
      r4eMarkers: count(text, /R4E_START|R4E_END/g),
      hardcodedArabicFeedback: count(text, new RegExp(prefix + 'FlashSettingsActionButtonR2\\(button,\\s*"تم', "g")),
      updateAllAfterAction: count(text, new RegExp(prefix + 'UpdateAllSettingsDirtyButtonsR4D\\("after-', "g")),
      localAfterAction: count(text, new RegExp(prefix + 'UpdateSettingsDirtyButtonsR4D\\(net,\\s*"after-', "g")),
      localStorageSet: count(text, /localStorage\.setItem/g),
      localStorageGet: count(text, /localStorage\.getItem/g),
      localStorageRemove: count(text, /localStorage\.removeItem/g)
    },
    localStorageKeys: storageKeys,
    functions,
    actionButtonOccurrences,
    referenceWindows,
    eventWindows,
    pathWindows
  };
}

function auditSettingsFile(text) {
  const functions = extractFunctionsByRegex(
    text,
    /function\s+([A-Za-z0-9_$]*(?:Settings|Preference|Language|Currency|Tab|Save|Load|Default|Restore)[A-Za-z0-9_$]*)\s*\(/g,
    120
  );

  return {
    exists: text.length > 0,
    length: text.length,
    counts: {
      localStorageSet: count(text, /localStorage\.setItem/g),
      localStorageGet: count(text, /localStorage\.getItem/g),
      localStorageRemove: count(text, /localStorage\.removeItem/g),
      languageRefs: count(text, /language|locale|kgw-language|kgw_lang/g),
      currencyRefs: count(text, /currency|fiat|usd|sar|eur/gi),
      tabRefs: count(text, /tab|visible|visibility|enabled/gi),
      nodeRefs: count(text, /node|kaspa-node|Node/g),
      bridgeRefs: count(text, /bridge|kaspa-bridge|Bridge/g),
      defaultRefs: count(text, /default|restore|factory/gi),
      saveRefs: count(text, /save|persist|storage/gi)
    },
    localStorageKeys: localStorageKeys(text),
    candidateFunctions: functions,
    storageWindows: findAll(text, /localStorage\.(?:getItem|setItem|removeItem)|indexedDB|settings|preferences|default|restore|language|currency|tabs?/gi, 200)
  };
}

function auditRust(text) {
  return {
    length: text.length,
    counts: {
      tauriCommands: count(text, /#\[tauri::command\]/g),
      traceCommand: count(text, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(text, /\[KGW_BUTTON_TRACE\]/g),
      generateHandler: count(text, /tauri::generate_handler!\s*\[/g),
      settingsCommands: count(text, /settings|config|preference|load|save|restore|default/gi)
    },
    traceWindows: findAll(text, /kgw_frontend_button_trace_v1|KGW_BUTTON_TRACE|generate_handler/gi, 60),
    settingsWindows: findAll(text, /settings|config|preference|load|save|restore|default/gi, 160)
  };
}

const nodeText = read(files.node);
const bridgeText = read(files.bridge);
const settingsText = read(files.settings);
const mainText = read(files.main);
const libText = read(files.lib);
const gateText = read(files.gate);

save("kaspa-node.full-source.js.txt", nodeText);
save("kaspa-bridge.full-source.js.txt", bridgeText);
if (settingsText) save("settings.full-source.js.txt", settingsText);
if (mainText) save("main.full-source.js.txt", mainText);
save("lib.rs.full-source.txt", libText);
if (gateText) save("kgw_global_owner_gate.full-source.cjs.txt", gateText);

const audit = {
  timestamp: new Date().toISOString(),
  files,
  node: auditTab("node", nodeText),
  bridge: auditTab("bridge", bridgeText),
  settings: auditSettingsFile(settingsText),
  main: auditSettingsFile(mainText),
  rust: auditRust(libText),
  gate: {
    exists: gateText.length > 0,
    length: gateText.length,
    counts: {
      settingsRefs: count(gateText, /settings|button|dirty|trace|default|save|restore/gi),
      r4Refs: count(gateText, /R4B|R4D|R4E|kgw_frontend_button_trace_v1/g)
    },
    windows: findAll(gateText, /settings|button|dirty|trace|default|save|restore|kgw_frontend_button_trace_v1/gi, 120)
  },
  findings: [],
  nextPatchPrinciples: []
};

function addFinding(condition, text) {
  if (condition) audit.findings.push(text);
}

addFinding(audit.node.counts.readSettings > 0, "Node has explicit R51 read settings owner.");
addFinding(audit.node.counts.saveSettings > 0, "Node has explicit R51 save settings owner.");
addFinding(audit.node.counts.referenceSettingsR2 > 0, "Node dirty-state depends on ReferenceSettingsR2 saved/default references.");
addFinding(audit.bridge.counts.readSettings > 0, "Bridge has explicit R51 read settings owner.");
addFinding(audit.bridge.counts.saveSettings > 0, "Bridge has explicit R51 save settings owner.");
addFinding(audit.bridge.counts.referenceSettingsR2 > 0, "Bridge dirty-state depends on ReferenceSettingsR2 saved/default references.");
addFinding(audit.settings.counts.localStorageSet > 0 || audit.settings.counts.localStorageGet > 0, "settings.js participates in persisted UI/global settings and must be included in fixes.");
addFinding(audit.node.counts.updateAllAfterAction > 0, "Node still updates all networks after some action; patch should be local to net.");
addFinding(audit.bridge.counts.updateAllAfterAction > 0, "Bridge still updates all networks after some action; patch should be local to net.");
addFinding(audit.node.counts.r4eMarkers > 0 || audit.bridge.counts.r4eMarkers > 0, "R4E remnants may exist if rollback did not fully restore; verify before patch.");
addFinding(audit.node.counts.hardcodedArabicFeedback > 0 || audit.bridge.counts.hardcodedArabicFeedback > 0, "Hard-coded Arabic feedback still exists somewhere and must be replaced by selected-language labels.");

audit.nextPatchPrinciples = [
  "Do not patch trace alone.",
  "Patch only after confirming load/save/default owners.",
  "For each net, compare current settings against saved settings and default settings using the real owner functions.",
  "After Save Settings, refresh only that net saved reference and dirty-state.",
  "After Set as Defaults, refresh only that net default reference and dirty-state.",
  "After Restore Defaults, apply defaults to DOM, then refresh only that net dirty-state after async dynamic paths complete.",
  "Disabled button press attempts require pointerdown trace because click does not fire on disabled buttons.",
  "Do not use document-wide MutationObserver or interval scanners.",
  "Do not use update-all after a single network action unless the original owner truly requires it.",
  "Do not commit or push."
];

saveJson("owner-audit-r5.json", audit);

const summary = [
  "KGW Settings Load/Save/Dirty Owner Deep Audit R5",
  "",
  "Node counts:",
  JSON.stringify(audit.node.counts, null, 2),
  "",
  "Bridge counts:",
  JSON.stringify(audit.bridge.counts, null, 2),
  "",
  "Settings.js counts:",
  JSON.stringify(audit.settings.counts, null, 2),
  "",
  "Rust counts:",
  JSON.stringify(audit.rust.counts, null, 2),
  "",
  "Findings:",
  audit.findings.map((x) => "- " + x).join("\n"),
  "",
  "Next patch principles:",
  audit.nextPatchPrinciples.map((x) => "- " + x).join("\n"),
  "",
  "Important report files:",
  "- owner-audit-r5.json",
  "- kaspa-node.full-source.js.txt",
  "- kaspa-bridge.full-source.js.txt",
  "- settings.full-source.js.txt if present",
  "- main.full-source.js.txt if present",
  "- lib.rs.full-source.txt"
].join("\n");

save("SUMMARY.txt", summary);

console.log(summary);