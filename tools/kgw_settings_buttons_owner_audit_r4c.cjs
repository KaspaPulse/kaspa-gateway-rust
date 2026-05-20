const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node audit.cjs <repoRoot> <reportDir>");
}

const files = {
  node: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridge: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
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

function windows(text, patterns) {
  const out = [];
  for (const item of patterns) {
    const label = item.label;
    const re = item.re;
    let match;
    while ((match = re.exec(text)) !== null) {
      const start = Math.max(0, match.index - 1200);
      const end = Math.min(text.length, match.index + 2200);
      out.push({
        label,
        line: lineOf(text, match.index),
        match: match[0].slice(0, 200),
        window: text.slice(start, end)
      });
      if (out.length > 120) break;
    }
  }
  return out;
}

function functionsAround(text, needles) {
  const out = [];
  const fnRe = /(?:async\s+)?function\s+([A-Za-z0-9_$]+)\s*\([^)]*\)\s*\{/g;
  let m;
  while ((m = fnRe.exec(text)) !== null) {
    const name = m[1];
    const start = m.index;
    const end = Math.min(text.length, start + 6000);
    const body = text.slice(start, end);
    if (needles.some((n) => body.includes(n) || name.includes(n))) {
      out.push({
        name,
        line: lineOf(text, start),
        preview: body.slice(0, 3500)
      });
    }
  }
  return out;
}

function auditFrontend(kind, text) {
  const patterns = [
    { label: "installActions", re: /installActions\s*\(/g },
    { label: "Save Settings text", re: /Save Settings|حفظ|Saved|تم الحفظ/g },
    { label: "Restore Defaults text", re: /Restore Defaults|Restored|تمت الاستعادة/g },
    { label: "Set as Defaults text", re: /Set as Defaults|Set as defaults|تم الضبط كافتراضي/g },
    { label: "button disabled", re: /\.disabled\s*=|setAttribute\s*\(\s*["']aria-disabled["']|aria-disabled/g },
    { label: "R4B marker", re: /KGW_[A-Z]+_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START/g },
    { label: "R4B scanner", re: /setInterval\s*\(\s*function\s*\(\)\s*\{\s*scan|MutationObserver|buttons-discovered|dirty-evaluated/g },
    { label: "settings save functions", re: /R51SaveSettings|SaveSettings|saveSettings/g },
    { label: "settings restore functions", re: /R51RestoreDefaults|RestoreDefaults|restoreDefaults/g },
    { label: "settings defaults functions", re: /R51SetAsDefaults|SetAsDefaults|setAsDefaults/g },
    { label: "i18n", re: /kgwI18n|document\.documentElement\.lang|localStorage\.getItem\(["'](?:kgw-language|kgw_lang|language|locale)/g }
  ];

  const ownerNeedles = [
    "installActions",
    "Save Settings",
    "Restore Defaults",
    "Set as Defaults",
    "R51SaveSettings",
    "R51RestoreDefaults",
    "R51SetAsDefaults",
    "dirty-evaluated",
    "buttons-discovered",
    "kgw_frontend_button_trace_v1"
  ];

  return {
    kind,
    length: text.length,
    counts: {
      r4bMarkers: count(text, /SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START/g),
      r4Markers: count(text, /SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4_START/g),
      r3Markers: count(text, /SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R3_START/g),
      mutationObserver: count(text, /MutationObserver/g),
      scanInterval: count(text, /setInterval\s*\(\s*function\s*\(\)\s*\{\s*scan/g),
      buttonsDiscovered: count(text, /buttons-discovered/g),
      dirtyEvaluated: count(text, /dirty-evaluated/g),
      clickReceived: count(text, /click-received/g),
      feedbackStart: count(text, /feedback-start/g),
      feedbackRestore: count(text, /feedback-restore/g),
      traceInvoke: count(text, /kgw_frontend_button_trace_v1/g),
      disabledAssignments: count(text, /\.disabled\s*=/g),
      ariaDisabledAssignments: count(text, /aria-disabled/g)
    },
    windows: windows(text, patterns),
    candidateFunctions: functionsAround(text, ownerNeedles)
  };
}

function auditLib(text) {
  return {
    length: text.length,
    counts: {
      traceCommandFunction: count(text, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrintMarker: count(text, /\[KGW_BUTTON_TRACE\]/g),
      generateHandler: count(text, /tauri::generate_handler!\s*\[/g),
      traceCommandRefs: count(text, /kgw_frontend_button_trace_v1/g)
    },
    windows: windows(text, [
      { label: "trace command", re: /kgw_frontend_button_trace_v1/g },
      { label: "generate_handler", re: /tauri::generate_handler!\s*\[/g },
      { label: "KGW_BUTTON_TRACE", re: /\[KGW_BUTTON_TRACE\]/g }
    ])
  };
}

const nodeText = read(files.node);
const bridgeText = read(files.bridge);
const libText = read(files.lib);
const gateText = read(files.gate);

save("kaspa-node.full-source.js.txt", nodeText);
save("kaspa-bridge.full-source.js.txt", bridgeText);
save("lib.rs.full-source.txt", libText);
if (gateText) save("kgw_global_owner_gate.full-source.cjs.txt", gateText);

const audit = {
  files,
  timestamp: new Date().toISOString(),
  node: auditFrontend("node", nodeText),
  bridge: auditFrontend("bridge", bridgeText),
  lib: auditLib(libText),
  gate: {
    exists: fs.existsSync(files.gate),
    length: gateText.length,
    r4bRefs: count(gateText, /R4B|SETTINGS_BUTTONS_RUNTIME_TRACE/g),
    windows: windows(gateText, [
      { label: "settings buttons gate", re: /SETTINGS_BUTTONS|kgw_frontend_button_trace_v1|dirty|feedback/g }
    ])
  },
  conclusions: []
};

if (audit.node.counts.r4bMarkers === 1) {
  audit.conclusions.push("Node contains exactly one R4B generated owner.");
}
if (audit.bridge.counts.r4bMarkers === 1) {
  audit.conclusions.push("Bridge contains exactly one R4B generated owner.");
}
if (audit.node.counts.scanInterval > 0 || audit.bridge.counts.scanInterval > 0) {
  audit.conclusions.push("R4B uses interval scanning; runtime logs showed this is too broad and noisy.");
}
if (audit.node.counts.mutationObserver > 0 || audit.bridge.counts.mutationObserver > 0) {
  audit.conclusions.push("R4B uses document-level MutationObserver; this can re-evaluate unrelated roots.");
}
if (audit.lib.counts.traceCommandFunction === 1 && audit.lib.counts.tracePrintMarker === 1) {
  audit.conclusions.push("Rust trace command exists exactly once and should be kept unless handler wiring fails.");
}
if (audit.node.counts.dirtyEvaluated > 0 || audit.bridge.counts.dirtyEvaluated > 0) {
  audit.conclusions.push("The next repair should remove noisy dirty-evaluated interval scanning and patch the existing button action owner directly.");
}

saveJson("owner-audit-r4c.json", audit);

const summary = [
  "KGW Settings Buttons Owner Audit R4C",
  "",
  "Node counts:",
  JSON.stringify(audit.node.counts, null, 2),
  "",
  "Bridge counts:",
  JSON.stringify(audit.bridge.counts, null, 2),
  "",
  "Rust counts:",
  JSON.stringify(audit.lib.counts, null, 2),
  "",
  "Conclusions:",
  audit.conclusions.map((x) => "- " + x).join("\n"),
  "",
  "Next correct patch:",
  "- Remove R4B document-wide observer and interval scanning.",
  "- Keep Rust kgw_frontend_button_trace_v1 if valid.",
  "- Patch existing installActions(root) / R51 action handlers only.",
  "- Scope dirty-state to the exact root that owns Save Settings / Restore Defaults / Set as Defaults.",
  "- Do not evaluate roots with buttonCount 0.",
  "- Do not scan document every interval.",
  "- Do not use exit in PowerShell scripts."
].join("\n");

save("SUMMARY.txt", summary);
console.log(summary);