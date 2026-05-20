const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node patcher.cjs <repoRoot> <reportDir>");
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

function count(text, re) {
  const m = text.match(re);
  return m ? m.length : 0;
}

function saveJson(name, value) {
  fs.mkdirSync(reportDir, { recursive: true });
  fs.writeFileSync(path.join(reportDir, name), JSON.stringify(value, null, 2), "utf8");
}

function windowAround(text, needle) {
  const i = text.indexOf(needle);
  if (i < 0) return null;
  return text.slice(Math.max(0, i - 700), Math.min(text.length, i + 1600));
}

function stripOldGeneratedBlocks(text) {
  const before = text;
  const markers = [
    "KGW_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R3",
    "KGW_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4",
    "KGW_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R3",
    "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4",
    "KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B",
    "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R3",
    "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4",
    "KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B"
  ];

  for (const marker of markers) {
    const re = new RegExp("\\n?\\s*// " + marker + "_START[\\s\\S]*?// " + marker + "_END\\s*\\n?", "g");
    text = text.replace(re, "\n");
  }

  text = text.replace(/\n{4,}/g, "\n\n\n");
  return { text, changed: before !== text };
}

function frontendBlock(scope) {
  const cap = scope === "node" ? "NODE" : "BRIDGE";
  const fn = scope === "node" ? "kgwNodeInstallSettingsButtonsTraceDirtyFixR4B" : "kgwBridgeInstallSettingsButtonsTraceDirtyFixR4B";

  return `
/* KGW ${cap} settings buttons trace and dirty-state owner R4B.
   This owner does not create buttons. It only observes existing buttons.
*/
// KGW_${cap}_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START
(function ${fn}() {
  "use strict";

  var KGW_SCOPE = "${scope}";
  var KGW_HOLD_MS = 3500;
  var KGW_BASELINE_DELAY_MS = 900;
  var KGW_SCAN_MS = 800;
  var installedKey = "__kgw_" + KGW_SCOPE + "_settings_buttons_trace_dirty_fix_r4b_installed";
  var states = new Map();

  var labels = {
    en: { save: "Saved", restore: "Restored", defaults: "Set as defaults" },
    ar: { save: "تم الحفظ", restore: "تمت الاستعادة", defaults: "تم الضبط كافتراضي" },
    de: { save: "Gespeichert", restore: "Wiederhergestellt", defaults: "Als Standard festgelegt" },
    es: { save: "Guardado", restore: "Restaurado", defaults: "Establecido como predeterminado" },
    fr: { save: "Enregistré", restore: "Restauré", defaults: "Défini par défaut" }
  };

  function lang() {
    var raw = [
      document.documentElement && document.documentElement.lang,
      document.body && document.body.getAttribute("lang"),
      localStorage.getItem("kgw-language"),
      localStorage.getItem("kgw_lang"),
      localStorage.getItem("language"),
      localStorage.getItem("locale"),
      "en"
    ].filter(Boolean)[0] || "en";
    raw = String(raw).toLowerCase().split("-")[0];
    return labels[raw] ? raw : "en";
  }

  function tr(action) {
    var l = lang();
    return labels[l][action] || labels.en[action] || action;
  }

  function textOf(el) {
    return String((el && (el.textContent || el.value || el.getAttribute("aria-label"))) || "").replace(/\\s+/g, " ").trim();
  }

  function inferAction(button) {
    if (!button) return null;
    var raw = [
      button.getAttribute("data-action"),
      button.getAttribute("data-kgw-action"),
      button.id,
      button.name,
      button.className,
      button.getAttribute("aria-label"),
      textOf(button)
    ].filter(Boolean).join(" ").toLowerCase();

    if (/save|حفظ|speichern|guardar|enregistrer/.test(raw)) return "save";
    if (/restore|reset|default settings|استعاد|استرجاع|wiederher|restaur|restablecer/.test(raw)) return "restore";
    if (/set as default|set defaults|as defaults|كافتراضي|standard fest|predeterminado|défini/.test(raw)) return "defaults";
    return null;
  }

  function inferNet(el) {
    var raw = "";
    try {
      var carrier = el && el.closest && (
        el.closest("[data-network]") ||
        el.closest("[data-net]") ||
        el.closest("[data-kaspa-network]") ||
        el.closest("[id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i], [class*='mainnet' i], [class*='testnet10' i], [class*='testnet12' i]")
      );
      raw = [
        carrier && carrier.getAttribute && carrier.getAttribute("data-network"),
        carrier && carrier.getAttribute && carrier.getAttribute("data-net"),
        carrier && carrier.getAttribute && carrier.getAttribute("data-kaspa-network"),
        carrier && carrier.id,
        carrier && carrier.className,
        el && el.getAttribute && el.getAttribute("data-network"),
        el && el.getAttribute && el.getAttribute("data-net")
      ].filter(Boolean).join(" ").toLowerCase();
    } catch (_) {}

    if (raw.indexOf("testnet12") >= 0 || raw.indexOf("tn12") >= 0) return "testnet12";
    if (raw.indexOf("testnet10") >= 0 || raw.indexOf("tn10") >= 0) return "testnet10";
    if (raw.indexOf("mainnet") >= 0) return "mainnet";
    return "default";
  }

  function rootOf(el) {
    if (!el || !el.closest) return document;
    return (
      el.closest("[data-network]") ||
      el.closest("[data-net]") ||
      el.closest("section") ||
      el.closest("main") ||
      el.closest("[class*='" + KGW_SCOPE + "' i]") ||
      document
    );
  }

  function keyOf(root, net) {
    var id = "document";
    try {
      id = (root && (root.id || root.getAttribute("data-network") || root.getAttribute("data-net") || root.className)) || "document";
    } catch (_) {}
    return KGW_SCOPE + "::" + net + "::" + String(id).slice(0, 120);
  }

  function isIgnoredControl(el) {
    var raw = [
      el && el.id,
      el && el.name,
      el && el.className,
      el && el.getAttribute && el.getAttribute("aria-label"),
      el && el.getAttribute && el.getAttribute("data-action")
    ].filter(Boolean).join(" ").toLowerCase();

    return /log|stdout|stderr|console|runtime|status|height|hashrate|balance|address-display|command-preview|preview|filter|search/.test(raw);
  }

  function snapshot(root) {
    return JSON.stringify(Array.prototype.slice.call((root || document).querySelectorAll("input, select, textarea"))
      .filter(function (el) { return !el.disabled && !el.readOnly && !isIgnoredControl(el); })
      .map(function (el, i) {
        var id = el.id || el.name || el.getAttribute("data-key") || el.getAttribute("aria-label") || String(i);
        var val = (el.type === "checkbox" || el.type === "radio") ? (el.checked ? "1" : "0") : String(el.value == null ? "" : el.value);
        return [String(id), val];
      })
      .sort(function (a, b) { return a[0].localeCompare(b[0]); }));
  }

  function buttons(root) {
    return Array.prototype.slice.call((root || document).querySelectorAll("button, input[type='button'], input[type='submit']"))
      .map(function (button) { return { button: button, action: inferAction(button) }; })
      .filter(function (x) { return x.action === "save" || x.action === "restore" || x.action === "defaults"; });
  }

  function setDisabled(button, disabled) {
    if (!button) return;
    button.disabled = !!disabled;
    button.setAttribute("aria-disabled", disabled ? "true" : "false");
    button.classList.toggle("kgw-settings-action-disabled-r4b", !!disabled);
  }

  function state(root, net) {
    var key = keyOf(root, net);
    if (!states.has(key)) {
      states.set(key, { key: key, root: root, net: net, baseline: null, dirty: false, timers: new Map() });
    }
    return states.get(key);
  }

  function trace(net, action, phase, details) {
    var payload = {
      scope: KGW_SCOPE,
      net: String(net || "default"),
      action: String(action || "unknown"),
      phase: String(phase || "unknown"),
      details: JSON.stringify(Object.assign({ at: new Date().toISOString() }, details || {}))
    };

    try {
      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", payload).catch(function () {});
        return;
      }
      if (window.__TAURI__ && window.__TAURI__.tauri && typeof window.__TAURI__.tauri.invoke === "function") {
        window.__TAURI__.tauri.invoke("kgw_frontend_button_trace_v1", payload).catch(function () {});
        return;
      }
      if (typeof window.__TAURI_INVOKE__ === "function") {
        window.__TAURI_INVOKE__("kgw_frontend_button_trace_v1", payload).catch(function () {});
        return;
      }
    } catch (_) {}

    try { console.log("[KGW_BUTTON_TRACE]", payload); } catch (_) {}
  }

  function evaluate(root, net, reason) {
    var st = state(root, net);
    var current = snapshot(root);

    if (st.baseline == null) {
      st.baseline = current;
      st.dirty = false;
    } else {
      st.dirty = current !== st.baseline;
    }

    var found = buttons(root);
    found.forEach(function (x) {
      x.button.setAttribute("data-kgw-settings-button-r4b", "1");
      setDisabled(x.button, !st.dirty);
    });

    trace(net, "dirty", "dirty-evaluated", {
      reason: reason,
      dirty: st.dirty,
      buttonCount: found.length,
      snapshotLength: current.length,
      baselineLength: st.baseline ? st.baseline.length : 0
    });
  }

  function establishBaseline(root, net, reason) {
    var st = state(root, net);
    st.baseline = snapshot(root);
    st.dirty = false;

    buttons(root).forEach(function (x) {
      x.button.setAttribute("data-kgw-settings-button-r4b", "1");
      setDisabled(x.button, true);
    });

    trace(net, "dirty", "baseline-established", {
      reason: reason,
      snapshotLength: st.baseline.length,
      buttonCount: buttons(root).length
    });
  }

  function flash(button, action, net) {
    var root = rootOf(button);
    var st = state(root, net);
    var original = button.getAttribute("data-kgw-original-label-r4b") || textOf(button);
    var label = tr(action);
    var started = Date.now();

    button.setAttribute("data-kgw-original-label-r4b", original);
    button.setAttribute("data-kgw-feedback-active-r4b", "1");

    clearTimeout(st.timers.get(button));
    button.textContent = label;

    trace(net, action, "feedback-start", { label: label, language: lang(), holdMs: KGW_HOLD_MS, original: original });

    var keep = setInterval(function () {
      if (Date.now() - started >= KGW_HOLD_MS) {
        clearInterval(keep);
        return;
      }
      if (button.getAttribute("data-kgw-feedback-active-r4b") === "1") {
        button.textContent = label;
      }
    }, 120);

    var timer = setTimeout(function () {
      clearInterval(keep);
      button.removeAttribute("data-kgw-feedback-active-r4b");
      button.textContent = original;
      st.timers.delete(button);
      establishBaseline(root, net, "after-" + action);
      trace(net, action, "feedback-restore", { elapsedMs: Date.now() - started, restoredLabel: original });
    }, KGW_HOLD_MS);

    st.timers.set(button, timer);
  }

  function onChange(event) {
    var target = event && event.target;
    if (!target || isIgnoredControl(target)) return;
    var root = rootOf(target);
    var net = inferNet(target);
    evaluate(root, net, event.type || "change");
  }

  function onClick(event) {
    var button = event && event.target && event.target.closest ? event.target.closest("button, input[type='button'], input[type='submit']") : null;
    var action = inferAction(button);
    if (!button || !action) return;

    var root = rootOf(button);
    var net = inferNet(button);
    var st = state(root, net);

    trace(net, action, "click-received", {
      disabled: !!button.disabled,
      ariaDisabled: button.getAttribute("aria-disabled"),
      dirty: st.dirty,
      text: textOf(button)
    });

    if (button.disabled || button.getAttribute("aria-disabled") === "true") {
      trace(net, action, "click-ignored-disabled", { dirty: st.dirty });
      return;
    }

    trace(net, action, "action-start", { dirty: st.dirty });
    setTimeout(function () {
      flash(button, action, net);
      trace(net, action, "action-complete", { dirtyBeforeBaselineRefresh: st.dirty });
    }, 0);
  }

  function scan(reason) {
    var roots = new Map();
    buttons(document).forEach(function (x) {
      var root = rootOf(x.button);
      var net = inferNet(x.button);
      roots.set(keyOf(root, net), { root: root, net: net });
      if (state(root, net).baseline == null) {
        setDisabled(x.button, true);
      }
    });

    roots.forEach(function (item) {
      if (state(item.root, item.net).baseline == null) {
        setTimeout(function () { establishBaseline(item.root, item.net, reason || "scan"); }, KGW_BASELINE_DELAY_MS);
      } else {
        evaluate(item.root, item.net, reason || "scan");
      }
    });

    if (roots.size) {
      trace("all", "scan", "buttons-discovered", { reason: reason, rootCount: roots.size });
    }
  }

  function install() {
    if (window[installedKey]) return;
    window[installedKey] = true;

    document.addEventListener("click", onClick, true);
    document.addEventListener("input", onChange, true);
    document.addEventListener("change", onChange, true);

    try {
      var observer = new MutationObserver(function () {
        clearTimeout(observer.__kgwTimer);
        observer.__kgwTimer = setTimeout(function () { scan("mutation"); }, 180);
      });
      observer.observe(document.documentElement || document.body, {
        childList: true,
        subtree: true,
        attributes: true,
        attributeFilter: ["disabled", "aria-disabled", "class", "data-network", "data-net"]
      });
    } catch (_) {}

    scan("install");
    setInterval(function () { scan("interval"); }, KGW_SCAN_MS);
    trace("all", "install", "installed", { scope: KGW_SCOPE, holdMs: KGW_HOLD_MS, scanMs: KGW_SCAN_MS });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", install, { once: true });
  } else {
    install();
  }
})();
// KGW_${cap}_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_END
`;
}

function patchFrontend(file, scope) {
  let text = read(file);
  const before = text;
  const stripped = stripOldGeneratedBlocks(text);
  text = stripped.text.replace(/\s*$/, "\n") + frontendBlock(scope);

  write(file, text);

  return {
    file,
    scope,
    strippedOldBlocks: stripped.changed,
    changed: before !== text,
    r4bMarkers: count(text, new RegExp("KGW_" + scope.toUpperCase() + "_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START", "g")),
    traceRefs: count(text, /kgw_frontend_button_trace_v1/g),
    dirtyRefs: count(text, /dirty-evaluated/g),
    window: windowAround(text, "SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START")
  };
}

function patchLib(file) {
  let text = read(file);
  const before = text;
  const commandName = "kgw_frontend_button_trace_v1";

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

    const insertPoints = ["#[tauri::command]", "pub fn run()", "fn main()"];
    let inserted = false;

    for (const marker of insertPoints) {
      const i = text.indexOf(marker);
      if (i >= 0) {
        text = text.slice(0, i) + command + "\n" + text.slice(i);
        inserted = true;
        break;
      }
    }

    if (!inserted) {
      throw new Error("Could not find insertion point for kgw_frontend_button_trace_v1 in lib.rs");
    }
  }

  const handlerRe = /tauri::generate_handler!\s*\[([\s\S]*?)\]/m;
  const m = text.match(handlerRe);
  if (!m) {
    throw new Error("Could not find tauri::generate_handler![...] in lib.rs");
  }

  const existing = m[1].split(",").map(x => x.trim()).filter(Boolean);
  if (!existing.includes(commandName)) existing.push(commandName);

  const body = existing.join(",\n            ");
  text = text.replace(handlerRe, "tauri::generate_handler![\n            " + body + "\n        ]");

  write(file, text);

  return {
    file,
    changed: before !== text,
    commandCount: count(text, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
    tracePrintCount: count(text, /\[KGW_BUTTON_TRACE\]/g),
    handlerRefs: count(text, /kgw_frontend_button_trace_v1/g),
    window: windowAround(text, "kgw_frontend_button_trace_v1")
  };
}

function audit() {
  const nodeText = read(files.node);
  const bridgeText = read(files.bridge);
  const libText = read(files.lib);

  return {
    node: {
      length: nodeText.length,
      oldR3Refs: count(nodeText, /DirtyButtonsR3|TRACE_DIRTY_FIX_R3/g),
      oldR4Refs: count(nodeText, /DirtyButtonsR4|TRACE_DIRTY_FIX_R4/g),
      r4bMarkers: count(nodeText, /KGW_NODE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START/g),
      traceRefs: count(nodeText, /kgw_frontend_button_trace_v1/g),
      window: windowAround(nodeText, "installActions")
    },
    bridge: {
      length: bridgeText.length,
      oldR3Refs: count(bridgeText, /DirtyButtonsR3|TRACE_DIRTY_FIX_R3/g),
      oldR4Refs: count(bridgeText, /DirtyButtonsR4|TRACE_DIRTY_FIX_R4/g),
      r4bMarkers: count(bridgeText, /KGW_BRIDGE_SETTINGS_BUTTONS_RUNTIME_TRACE_DIRTY_FIX_R4B_START/g),
      traceRefs: count(bridgeText, /kgw_frontend_button_trace_v1/g),
      window: windowAround(bridgeText, "installActions")
    },
    lib: {
      length: libText.length,
      commandCount: count(libText, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrintCount: count(libText, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(libText, /kgw_frontend_button_trace_v1/g),
      handlerWindow: windowAround(libText, "tauri::generate_handler!")
    }
  };
}

saveJson("audit-before.json", audit());

const changes = {
  node: patchFrontend(files.node, "node"),
  bridge: patchFrontend(files.bridge, "bridge"),
  lib: patchLib(files.lib)
};

saveJson("patch-changes.json", changes);

const after = audit();
saveJson("audit-after.json", after);

const failures = [];
if (after.node.r4bMarkers !== 1) failures.push("Node R4B marker must exist exactly once.");
if (after.bridge.r4bMarkers !== 1) failures.push("Bridge R4B marker must exist exactly once.");
if (after.lib.commandCount !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.lib.tracePrintCount !== 1) failures.push("Rust trace println marker must exist exactly once.");
if (after.lib.handlerRefs < 2) failures.push("Rust trace command must be defined and registered in generate_handler.");

if (failures.length) {
  throw new Error("Static validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R4B patch complete");
console.log(JSON.stringify({
  nodeR4BMarkers: after.node.r4bMarkers,
  bridgeR4BMarkers: after.bridge.r4bMarkers,
  rustTraceCommandCount: after.lib.commandCount,
  rustTracePrintCount: after.lib.tracePrintCount,
  rustTraceHandlerRefs: after.lib.handlerRefs
}, null, 2));