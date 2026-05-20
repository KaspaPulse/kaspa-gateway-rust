// KGW_SETTINGS_OWNER_V19
(function installKgwSettingsOwnerV19() {
  "use strict";

  const OWNER = "KGW_SETTINGS_OWNER_V19";
  const PATCH = "KGW_SETTINGS_OWNER_V19_SAFE_FEEDBACK_NO_FREEZE_V25B";
  const SCOPE = "node";
  const GLOBAL_NAME = "KGW_NODE_SETTINGS_OWNER_V19";
  const FEEDBACK_MS = 3000;
  const DISABLED_CLASS = "kgw-settings-action-disabled-v19";
  const ROOT_INSTALLED_ATTR = "kgwSettingsOwnerV19";

  const feedbackByRoot = new WeakMap();
  const dirtyByRoot = new WeakMap();

  function lower(value) {
    return String(value || "").toLowerCase();
  }

  function trace(root, phase, details) {
    try {
      const safeDetails = details && typeof details === "object" ? details : {};
      const net = String(safeDetails.network || safeDetails.net || "unknown");
      const action = String(safeDetails.action || "settings-owner");
      const detailsText = JSON.stringify({
        owner: OWNER,
        patch: PATCH,
        scope: SCOPE,
        phase: phase,
        details: safeDetails
      });

      const args = {
        scope: String(SCOPE),
        net: net,
        action: action,
        phase: String(phase || "unknown"),
        details: detailsText
      };

      const tauri = window.__TAURI__;
      if (tauri && tauri.core && typeof tauri.core.invoke === "function") {
        tauri.core.invoke("kgw_frontend_button_trace_v1", args).catch(function (error) {
          console.error("[KGW_SETTINGS_OWNER_V19_TRACE_FAILED]", error, args);
        });
      } else if (tauri && typeof tauri.invoke === "function") {
        tauri.invoke("kgw_frontend_button_trace_v1", args).catch(function (error) {
          console.error("[KGW_SETTINGS_OWNER_V19_TRACE_FAILED]", error, args);
        });
      } else {
        console.debug("[KGW_SETTINGS_OWNER_V19_TRACE_BROWSER]", args);
      }
    } catch (error) {
      console.error("[KGW_SETTINGS_OWNER_V19_TRACE_EXCEPTION]", error);
    }
  }

  function currentLanguage() {
    try {
      const lang = String(document.documentElement.getAttribute("lang") || document.body.getAttribute("lang") || "");
      if (lang) return lower(lang);
    } catch (_) {}

    try {
      const keys = ["kgw.language", "kgw_locale", "language", "locale", "i18nextLng"];
      for (const key of keys) {
        const value = localStorage.getItem(key);
        if (value) return lower(value);
      }
    } catch (_) {}

    try {
      const apiCandidates = [window.kgwI18n, window.KGWI18n, window.KGW_I18N, window.i18n];
      for (const api of apiCandidates) {
        if (!api) continue;
        const values = [api.language, api.lang, api.locale, api.currentLanguage, api.currentLocale];
        for (const value of values) {
          if (value) return lower(value);
        }
        if (typeof api.getLanguage === "function") return lower(api.getLanguage());
        if (typeof api.getLocale === "function") return lower(api.getLocale());
      }
    } catch (_) {}

    return "";
  }

  function isArabic() {
    return currentLanguage().startsWith("ar") || lower(document.dir) === "rtl";
  }

  function translate(key, fallback) {
    try {
      const candidates = [window.kgwT, window.__kgwT, window.t];
      for (const fn of candidates) {
        if (typeof fn === "function") {
          const value = fn(key, fallback);
          if (typeof value === "string" && value.trim() && value !== key) return value;
        }
      }

      const apis = [window.kgwI18n, window.KGWI18n, window.KGW_I18N, window.i18n];
      for (const api of apis) {
        if (api && typeof api.t === "function") {
          const value = api.t(key, fallback);
          if (typeof value === "string" && value.trim() && value !== key) return value;
        }
        if (api && typeof api.translate === "function") {
          const value = api.translate(key, fallback);
          if (typeof value === "string" && value.trim() && value !== key) return value;
        }
      }
    } catch (_) {}
    return fallback;
  }

  function isSettingsControl(element) {
    if (!element || !element.tagName) return false;

    const tag = lower(element.tagName);
    if (tag !== "input" && tag !== "select" && tag !== "textarea") return false;

    const type = lower(element.type);
    if (type === "button" || type === "submit" || type === "reset" || type === "hidden") return false;

    if (element.closest && element.closest(".logs, .log, [data-log], .kgw-log-pane")) return false;

    return true;
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
      text.includes("تم الحفظ") ||
      text.includes("تم الضبط") ||
      text.includes("تمت الاستعادة") ||
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
      (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action || button.dataset.kgwSettingsOwnerV19Action)) ||
        button.getAttribute("data-action") ||
        button.getAttribute("aria-label") ||
        button.textContent ||
        ""
    );

    if (raw.includes("restore") || raw.includes("استعادة")) return "restore";
    if (raw.includes("default") || raw.includes("افتراض") || raw.includes("ضبط")) return "defaults";
    return "save";
  }

  function feedbackText(action) {
    if (isArabic()) {
      if (action === "restore") return "تمت الاستعادة";
      if (action === "defaults") return "تم الضبط";
      return "تم الحفظ";
    }

    if (action === "restore") return translate("settings.feedback.restored", "Restored");
    if (action === "defaults") return translate("settings.feedback.setAsDefaults", "Set");
    return translate("settings.feedback.saved", "Saved");
  }

  function fallbackText(action) {
    if (action === "restore") return "Restore Defaults";
    if (action === "defaults") return "Set as Defaults";
    return "Save Settings";
  }

  function allButtons(root) {
    return Array.from(root.querySelectorAll("button")).filter(isActionButton);
  }

  function buttons(root, network) {
    return allButtons(root).filter(function (button) {
      return !network || network === "all" || networkOf(button) === network;
    });
  }

  function dirtyMap(root) {
    let map = dirtyByRoot.get(root);
    if (!map) {
      map = new Map();
      dirtyByRoot.set(root, map);
    }
    return map;
  }

  function feedbackMap(root) {
    let map = feedbackByRoot.get(root);
    if (!map) {
      map = new Map();
      feedbackByRoot.set(root, map);
    }
    return map;
  }

  function setDisabled(root, network, disabled, reason) {
    buttons(root, network || "all").forEach(function (button) {
      button.disabled = !!disabled;
      button.setAttribute("aria-disabled", disabled ? "true" : "false");
      button.classList.toggle(DISABLED_CLASS, !!disabled);
      button.dataset.kgwSettingsOwnerV19Disabled = disabled ? "true" : "false";
    });

    trace(root, disabled ? "v19-disabled" : "v19-enabled", {
      network: network || "all",
      reason: reason || "unspecified"
    });
  }

  function setDirty(root, network, dirty, reason) {
    dirtyMap(root).set(network, !!dirty);
    setDisabled(root, network, !dirty, reason || (dirty ? "dirty" : "clean"));
  }

  function isFeedbackLabel(text) {
    const value = lower(text);
    return (
      value === "saved" ||
      value === "restored" ||
      value === "set" ||
      value === "set as defaults" ||
      value === "تم الحفظ" ||
      value === "تم الضبط" ||
      value === "تمت الاستعادة"
    );
  }

  function rememberOriginalLabel(button, action) {
    const current = String(button.textContent || "").trim();

    if (!button.dataset.kgwSettingsOwnerV19OriginalLabel || isFeedbackLabel(current)) {
      button.dataset.kgwSettingsOwnerV19OriginalLabel = current && !isFeedbackLabel(current) ? current : fallbackText(action);
    }

    button.dataset.kgwSettingsOwnerV19Action = action;
  }

  function restoreLabel(button) {
    button.textContent = button.dataset.kgwSettingsOwnerV19OriginalLabel || fallbackText(actionName(button));
  }

  function restoreLabels(root, network) {
    buttons(root, network).forEach(function (button) {
      restoreLabel(button);
    });
  }

  function clearFeedback(root, network, reason) {
    const map = feedbackMap(root);
    const active = map.get(network);

    if (!active) return;

    if (active.timer) window.clearTimeout(active.timer);
    if (active.button) restoreLabel(active.button);

    map.delete(network);

    trace(root, "v19-feedback-cleared", {
      network: network,
      reason: reason || "clear"
    });
  }

  function startVisualFeedbackAfterOriginalClick(root, network, button, action) {
    window.setTimeout(function () {
      clearFeedback(root, network, "new-feedback");

      dirtyMap(root).set(network, false);
      rememberOriginalLabel(button, action);

      const label = feedbackText(action);

      button.textContent = label;
      button.setAttribute("aria-label", label);
      button.setAttribute("title", label);

      setDisabled(root, network, true, "feedback-clean-state");

      const timer = window.setTimeout(function () {
        const active = feedbackMap(root).get(network);
        if (!active || active.button !== button) return;

        feedbackMap(root).delete(network);
        restoreLabel(button);

        const dirty = dirtyMap(root).get(network) === true;
        setDisabled(root, network, !dirty, dirty ? "feedback-complete-dirty" : "feedback-complete-clean");

        trace(root, "v19-feedback-complete", {
          network: network,
          action: action,
          holdMs: FEEDBACK_MS,
          dirty: dirty,
          safeNoFreeze: true
        });
      }, FEEDBACK_MS);

      feedbackMap(root).set(network, {
        timer: timer,
        button: button,
        action: action,
        label: label
      });

      trace(root, "v19-feedback-start", {
        network: network,
        action: action,
        holdMs: FEEDBACK_MS,
        label: label,
        visualOnly: true,
        safeNoFreeze: true
      });
    }, 0);
  }

  function install(root) {
    if (!root || root.dataset[ROOT_INSTALLED_ATTR] === "installed") return;

    root.dataset[ROOT_INSTALLED_ATTR] = "installed";
    setDisabled(root, "all", true, "initial");

    root.addEventListener("input", function (event) {
      if (!isSettingsControl(event.target)) return;

      const network = networkOf(event.target);

      if (!event.isTrusted) {
        setDisabled(root, network, true, "input-programmatic");
        return;
      }

      clearFeedback(root, network, "trusted-input");
      restoreLabels(root, network);
      setDirty(root, network, true, "trusted-input");
    }, true);

    root.addEventListener("change", function (event) {
      if (!isSettingsControl(event.target)) return;

      const network = networkOf(event.target);

      if (!event.isTrusted) {
        setDisabled(root, network, true, "change-programmatic");
        return;
      }

      clearFeedback(root, network, "trusted-change");
      restoreLabels(root, network);
      setDirty(root, network, true, "trusted-change");
    }, true);

    root.addEventListener("click", function (event) {
      const button = event.target && event.target.closest ? event.target.closest("button") : null;
      if (!button || !root.contains(button) || !isActionButton(button)) return;

      const network = networkOf(button);
      const action = actionName(button);
      const disabled = !!button.disabled || button.dataset.kgwSettingsOwnerV19Disabled === "true";

      trace(root, "v19-click", {
        network: network,
        action: action,
        disabled: disabled,
        label: String(button.textContent || "").trim()
      });

      if (disabled) {
        event.preventDefault();
        event.stopImmediatePropagation();
        setDisabled(root, network, true, "click-blocked-clean-state");
        return;
      }

      startVisualFeedbackAfterOriginalClick(root, network, button, action);
    }, true);

    trace(root, "v19-owner-installed", {
      scope: SCOPE,
      patch: PATCH,
      feedbackMs: FEEDBACK_MS,
      safeNoFreeze: true
    });
  }

  window[GLOBAL_NAME] = {
    install: install,
    setDisabled: setDisabled,
    buttons: buttons
  };

  window.KGW_SETTINGS_OWNER_V19 = window[GLOBAL_NAME];
})();
// END_KGW_SETTINGS_OWNER_V19




function kgwI18nTextR41(key, fallback) {
  try {
    if (window.kgwT && typeof window.kgwT === "function") return window.kgwT(key, fallback);
    if (window.KGW_I18N && typeof window.KGW_I18N.t === "function") return window.KGW_I18N.t(key, fallback);
    if (window.i18n && typeof window.i18n.t === "function") return window.i18n.t(key, fallback);
  } catch (_) {
  }
  return fallback;
}

function kgwGuardBlockReasonV3(fields, raw) {
  const direct = fields && (fields.block_reason || fields.reason || fields.required_feature_reason);
  if (direct && String(direct).trim()) {
    return String(direct).trim();
  }

  const text = stringifyRuntimeResult(raw);
  const match = text.match(/(?:^|;)reason=([^;]+)/);
  if (match && match[1] && match[1].trim()) {
    return match[1].trim();
  }

  return "guard blocked without detailed reason";
}

/* KGW_NODE_RUSTY_KASPA_ROOT_ONLY_DEFAULT_PATHS_FIX_R5
 * Canonical node runtime default path owner.
 * The only generated default path is the current user's LocalAppData rusty-kaspa root.
 * Example runtime value: %LOCALAPPDATA%\rusty-kaspa.
 * No network suffix, no logs suffix, no KGW app-data root.
 * Kaspa owns/completes internal database layout below this root.
 */
function kgwNodeBackendInvokeR5(command, payload = {}) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_INVOKE__;

  if (typeof invoke !== "function") {
    return Promise.reject(new Error("Tauri invoke is not available"));
  }

  return invoke(command, payload);
}

function kgwNodeJoinPathR5(root, child) {
  const base = String(root || "").replace(/[\\/]+$/, "");
  if (!base) return "";
  return base + "\\" + child;
}

function kgwNodeExtractUserLocalAppDataR5(paths) {
  const values = Object.values(paths || {}).map((value) => String(value || ""));
  for (const value of values) {
    const match = value.match(/^([A-Za-z]:[\\/]Users[\\/][^\\/]+[\\/]AppData)[\\/](?:Local|Roaming)(?:[\\/].*)?$/i);
    if (match && match[1]) {
      return match[1] + "\\Local";
    }
  }
  return "%LOCALAPPDATA%";
}

function kgwNodeRustyKaspaLocalAppDataRootR5(paths = {}) {
  return kgwNodeJoinPathR5(kgwNodeExtractUserLocalAppDataR5(paths), "rusty-kaspa");
}

function kgwNodeIsEmptyOrGeneratedPathR5(value) {
  const text = String(value || "");
  return text.trim() === "" || /^[A-Za-z]:[\\/]+Users[\\/]+[^\\/]+AppData[\\/]+(?:Local|Roaming)[\\/]+(?:rusty-kaspa|KaspaGateway)(?:[\\/].*)?$/i.test(text) || /^%LOCALAPPDATA%[\\/]+rusty-kaspa(?:[\\/].*)?$/i.test(text);
}

async function kgwNodeLoadEnvironmentPathHintsR5() {
  try {
    const defaults = await kgwNodeBackendInvokeR5("settings_defaults");
    return defaults && defaults.paths ? defaults.paths : {};
  } catch (_) {
    return {};
  }
}

async function kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5(net, options = {}) {
  const force = options.force === true;
  const pathHints = await kgwNodeLoadEnvironmentPathHintsR5();
  const rustyRoot = kgwNodeRustyKaspaLocalAppDataRootR5(pathHints);

  const values = {
    appDir: rustyRoot,
    logDir: rustyRoot,
    configFile: "",
    rocksDbWalDir: "",
    overrideParamsFile: ""
  };

  Object.entries(values).forEach(([name, value]) => {
    const field = byId(id(net, name));
    if (!field) return;
    const current = String(field.value || "");
    if (force || kgwNodeIsEmptyOrGeneratedPathR5(current)) {
      field.value = value;
      field.dispatchEvent(new Event("input", { bubbles: true }));
      field.dispatchEvent(new Event("change", { bubbles: true }));
    }
  });

  updateCommand(net);
  return values;
}

function kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, options = {}) {
  void kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5(net, options).catch((error) => {
    appendLog(net, "Rusty Kaspa root-only default path restore failed: " + normalizeRuntimeError(error));
  });
}

const NODE_NETWORKS = [
  { key: "mainnet", label: "MAINNET", testnet: false, netsuffix: "" },
  { key: "testnet10", label: "TESTNET10", testnet: true, netsuffix: "10" },
  { key: "testnet12", label: "TESTNET12", testnet: true, netsuffix: "12" }
];

function byId(id) {
  return document.getElementById(id);
}

function esc(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function id(net, name) {
  return `node-${net}-${name}`;
}

function kgwDefaultRustyKaspaRootR41() {
  return "";
}

function v(net, name) {
  const el = byId(id(net, name));
  return el ? String(el.value || "").trim() : "";
}

function c(net, name) {
  const el = byId(id(net, name));
  return Boolean(el && el.checked);
}

function addFlag(lines, net, name, flag) {
  if (c(net, name)) lines.push(flag);
}

function addValue(lines, net, name, flag) {
  const value = v(net, name);
  if (value) lines.push(`${flag}=${value}`);
}

function addHostPort(lines, net, enabledName, flag, hostName, portName) {
  if (!c(net, enabledName)) return;

  const host = v(net, hostName);
  const port = v(net, portName);

  if (host && port) lines.push(`${flag}=${host}:${port}`);
  else if (host) lines.push(`${flag}=${host}`);
}

function cardInput(net, name, label, value = "", placeholder = "", span2 = false) {
  return `
    <div class="node-v6-card${span2 ? " span2" : ""}">
      <span>${esc(label)}</span>
      <input id="${id(net, name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function cardSelect(net, name, label, options, value = "", span2 = false) {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="node-v6-card${span2 ? " span2" : ""}">
      <span>${esc(label)}</span>
      <select id="${id(net, name)}">${opts}</select>
    </div>`;
}

function cardCheck(net, name, label, checked = false, span2 = false) {
  return `
    <label class="node-v6-card check${span2 ? " span2" : ""}">
      <input id="${id(net, name)}" type="checkbox"${checked ? " checked" : ""}>
      <span>${esc(label)}</span>
    </label>`;
}


function nodeLogLineBelongsToNode(line) {
  const text = String(line || "");

  if (!text.trim()) return false;

  if (/\[self-worker\]\[bridge\]/i.test(text)) return false;
  if (/\brole=bridge\b/i.test(text)) return false;
  if (/^Bridge\s+(status|logs)\s*\[/i.test(text)) return false;
  if (/^KGW bridge\b/i.test(text)) return false;
  if (/\bstratum-bridge\b/i.test(text)) return false;
  if (/\b(start-official-external-node|stratum_listen|kaspa_rpc_endpoint)\b/i.test(text)) return false;

  return true;
}


// KGW_NODE_LOG_AUTOSCROLL_CONTROLS_R27_START
function kgwNodeLogAutoScrollKeyR27(net) {
  return `kgw.node.log.autoscroll.${net}`;
}

function kgwNodeLogAutoScrollEnabledR27(net) {
  try {
    return localStorage.getItem(kgwNodeLogAutoScrollKeyR27(net)) !== "0";
  } catch (_) {
    return true;
  }
}

function kgwNodeSetLogAutoScrollR27(net, enabled) {
  try {
    localStorage.setItem(kgwNodeLogAutoScrollKeyR27(net), enabled ? "1" : "0");
  } catch (_) {}

  const out = byId(id(net, "logOutput"));
  if (enabled && out) out.scrollTop = out.scrollHeight;
}

function kgwInstallNodeLogAutoScrollControlsR27() {
  if (typeof document === "undefined") return;
  if (!Array.isArray(NODE_NETWORKS)) return;

  for (const profile of NODE_NETWORKS) {
    const net = profile.key;
    const out = byId(id(net, "logOutput"));
    if (!out) continue;

    const controlId = id(net, "logAutoScrollR27");
    if (byId(controlId)) continue;

    const label = document.createElement("label");
    label.className = "kgw-log-autoscroll-toggle";
    label.setAttribute("data-kgw-log-autoscroll", "node");
    label.setAttribute("title", "Keep the log pinned to the newest raw line.");

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.id = controlId;
    checkbox.checked = kgwNodeLogAutoScrollEnabledR27(net);
    checkbox.addEventListener("change", () => kgwNodeSetLogAutoScrollR27(net, checkbox.checked));

    const span = document.createElement("span");
    span.textContent = kgwI18nTextR41("common.autoScroll", "Auto-scroll");

    label.appendChild(checkbox);
    label.appendChild(span);

    const panel = out.closest(".node-v6-inner-panel, [data-node-inner-panel], [data-inner-panel], [data-node-panel], [data-panel]") || out.parentElement;
    const toolbar =
      panel?.querySelector(".node-v6-log-toolbar, .node-log-toolbar, [data-node-log-toolbar]") ||
      out.parentElement?.querySelector(".node-v6-log-toolbar, .node-log-toolbar, [data-node-log-toolbar]");

    if (toolbar) {
      toolbar.appendChild(label);
    } else {
      out.parentElement?.insertBefore(label, out);
    }
  }
}
// KGW_NODE_LOG_AUTOSCROLL_CONTROLS_R27_END

function appendLog(net, message) {
  // KGW_RESTORE_NODE_TAB_FROM_XML_R18_SAFE_RAW_LOGS
  const out = byId(id(net, "logOutput"));
  const profile = NODE_NETWORKS.find((item) => item.key === net);
  const emptyText = `${profile?.label || net} log is empty.`;

  if (!out) return;

  const previousText = out.textContent === emptyText ? "" : String(out.textContent || "");
  const lines = previousText ? previousText.split("\n").filter(Boolean) : [];
  const rawText = String(message ?? "");
  const accepted = [];

  for (const rawLine of rawText.split(/\r?\n/)) {
    const cleanLine = rawLine.trimEnd();

    if (!cleanLine.trim()) continue;
    if (/^Node\s+(status|logs)\s*\[/i.test(cleanLine)) continue;
    if (/^Bridge\s+(status|logs)\s*\[/i.test(cleanLine)) continue;
    if (/^KGW\s+/i.test(cleanLine)) continue;
    if (/parallel-owned-self-worker/i.test(cleanLine)) continue;

    if (typeof nodeLogLineBelongsToNode === "function" && !nodeLogLineBelongsToNode(cleanLine)) {
      continue;
    }

    accepted.push(cleanLine);
  }

  if (accepted.length === 0) {
    if (out.textContent === emptyText) out.textContent = "";
    return;
  }

  for (const line of accepted) lines.push(line);
  while (lines.length > 3000) lines.shift();

  out.textContent = lines.join("\n");
  if (kgwNodeLogAutoScrollEnabledR27(net)) out.scrollTop = out.scrollHeight;
}

function renderRuntime(net) {
  const networkIdentityControls = net.testnet
    ? `
      ${cardCheck(net.key, "testnet", "--testnet", true)}
      ${cardInput(net.key, "netsuffix", "--netsuffix", net.netsuffix, "required")}`
    : "";

  return `
    <div class="node-v6-grid">
      ${networkIdentityControls}
      ${cardSelect(net.key, "logLevel", "--loglevel", ["off", "error", "warn", "info", "debug", "trace"], "info")}
      ${cardInput(net.key, "asyncThreads", "--async-threads", "16")}
      ${cardInput(net.key, "ramScale", "--ram-scale", "1")}
      ${cardCheck(net.key, "yes", "--yes", true)}
      ${cardCheck(net.key, "noLogFiles", "--nologfiles", true)}
      ${cardCheck(net.key, "sanity", "--sanity", false)}
      ${cardCheck(net.key, "enableUnsyncedMining", "--enable-unsynced-mining", false, true)}
    </div>`;
}


function renderNetwork(net) {
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "listenEnabled", "--listen", true)}
      ${cardInput(net.key, "listenHost", "listen host", "0.0.0.0")}
      ${cardInput(net.key, "listenPort", "listen port", net.testnet ? "16211" : "16111")}
      ${cardCheck(net.key, "externalIpEnabled", "--externalip", false)}
      ${cardInput(net.key, "externalIpHost", "external host", "", "ip")}
      ${cardInput(net.key, "externalIpPort", "external port", "", "port")}
      ${cardCheck(net.key, "disableUpnp", "--disable-upnp", false)}
      ${cardCheck(net.key, "noDnsSeed", "--nodnsseed", false)}
      ${cardInput(net.key, "uaComment", "--uacomment", "", "comment", true)}
    </div>`;
}

function renderRpc(net) {
  const base = net.key === "mainnet" ? 16110 : net.key === "testnet10" ? 16210 : 16310;
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "rpcListenEnabled", "--rpclisten", true)}
      ${cardInput(net.key, "rpcListenHost", "RPC host", "127.0.0.1")}
      ${cardInput(net.key, "rpcListenPort", "RPC port", String(base))}
      ${cardCheck(net.key, "rpcBorshEnabled", "--rpclisten-borsh", true)}
      ${cardInput(net.key, "rpcBorshHost", "Borsh host", "127.0.0.1")}
      ${cardInput(net.key, "rpcBorshPort", "Borsh port", String(base + 1000))}
      ${cardCheck(net.key, "rpcJsonEnabled", "--rpclisten-json", true)}
      ${cardInput(net.key, "rpcJsonHost", "JSON host", "127.0.0.1")}
      ${cardInput(net.key, "rpcJsonPort", "JSON port", String(base + 2000))}
      ${cardInput(net.key, "rpcMaxClients", "--rpcmaxclients", "128")}
      ${cardCheck(net.key, "unsafeRpc", "--unsaferpc", false)}
      ${cardCheck(net.key, "noGrpc", "--nogrpc", false)}
    </div>`;
}

function renderPeers(net) {
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "connectEnabled", "--connect", false)}
      ${cardInput(net.key, "connectHost", "connect host", "", "host")}
      ${cardInput(net.key, "connectPort", "connect port", "", "port")}
      ${cardCheck(net.key, "addPeerEnabled", "--addpeer", false)}
      ${cardInput(net.key, "addPeerHost", "peer host", "", "host")}
      ${cardInput(net.key, "addPeerPort", "peer port", "", "port")}
      ${cardInput(net.key, "outPeers", "--outpeers", "8")}
      ${cardInput(net.key, "maxInPeers", "--maxinpeers", "128")}
    </div>`;
}

function renderDatabase(net) {
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "utxoIndex", "--utxoindex", true)}
      ${cardCheck(net.key, "archival", "--archival", false)}
      ${cardCheck(net.key, "resetDb", "--reset-db", false)}
      ${cardCheck(net.key, "perfMetrics", "--perf-metrics", true)}
      ${cardInput(net.key, "maxTrackedAddresses", "--max-tracked-addresses", "", "0")}
      ${cardInput(net.key, "retentionDays", "--retention-period-days", "", "optional")}
      ${cardInput(net.key, "perfMetricsInterval", "--perf-metrics-interval-sec", "", "optional", true)}
    </div>`;
}

function renderRocksDb(net) {
  return `
    <div class="node-v6-grid">
      ${cardSelect(net.key, "rocksDbPreset", "--rocksdb-preset", ["", "default", "hdd"], "")}
      ${cardInput(net.key, "rocksDbCacheSize", "--rocksdb-cache-size", "", "MB")}
      ${cardInput(net.key, "rocksDbWalDir", "--rocksdb-wal-dir", "", "path", true)}
      ${cardInput(net.key, "overrideParamsFile", "--override-params-file", "", "json path", true)}
    </div>`;
}

function renderPaths(net) {
  return `
    <div class="node-v6-grid">
      ${cardInput(net.key, "configFile", "--configfile", "", "kaspa.conf")}
      ${cardInput(net.key, "appDir", "--appdir", "", "app dir")}
      ${cardInput(net.key, "logDir", "--logdir", "", "log dir")}
    </div>`;
}

function renderSections(net) {
  const sections = [
    ["runtime", "Runtime", renderRuntime(net)],
    ["network", "Network", renderNetwork(net)],
    ["rpc", "RPC", renderRpc(net)],
    ["peers", "Peers", renderPeers(net)],
    ["database", "Database", renderDatabase(net)],
    ["rocksdb", "RocksDB", renderRocksDb(net)],
    ["paths", "Paths", renderPaths(net)]
  ];

  const tabs = sections.map(([key, label], index) =>
    `<button type="button" class="node-v6-section-tab${index === 0 ? " active" : ""}" data-net="${net.key}" data-node-section-tab="${key}">${label}</button>`
  ).join("");

  const panels = sections.map(([key, , body], index) =>
    `<section class="node-v6-section${index === 0 ? " active" : ""}" data-net="${net.key}" data-node-section-panel="${key}"${index === 0 ? "" : " hidden"}>${body}</section>`
  ).join("");

  return `
    <div class="node-v6-section-tabs">${tabs}</div>
    <div class="node-v6-sections">${panels}</div>`;
}

function renderNetworkPanel(net, index) {
  return `
    <div class="node-v6-network-panel${index === 0 ? " active" : ""}" data-node-network-panel="${net.key}"${index === 0 ? "" : " hidden"}>
      <div class="node-v6-inner-tabs">
        <button type="button" class="node-v6-inner-tab active" data-net="${net.key}" data-node-inner-tab="settings">Settings</button>
        <button type="button" class="node-v6-inner-tab" data-net="${net.key}" data-node-inner-tab="log">Log</button>
      </div>

      <div class="node-v6-inner-panel active" data-net="${net.key}" data-node-inner-panel="settings">
        <section class="node-v6-command">
          <div class="node-v6-command-title">Command Preview</div>
          <textarea id="${id(net.key, "commandPreview")}" readonly spellcheck="false" wrap="soft"></textarea>
          <button type="button" class="node-v6-copy" data-node-action="copy-command" data-net="${net.key}" title="Copy command">⧉</button>
        </section>

        <section class="node-v6-toolbar">
          <div class="node-v6-buttons">
            <button type="button" class="good" data-node-action="start" data-net="${net.key}">Start</button>
            <button type="button" data-node-action="stop" data-net="${net.key}">Stop</button>
          </div>

          <div class="node-v6-status">
            <label><input id="${id(net.key, "startOnLaunch")}" type="checkbox"> Launch</label>
            <label><input id="${id(net.key, "autoRestart")}" type="checkbox" checked> Restart</label>
          </div>
        </section>

        ${renderSections(net)}

        <div class="settings-bottom-actions node-settings-bottom-actions">
        <button type="button" data-node-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-node-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-node-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        </div>

      </div>

      <div class="node-v6-inner-panel" data-net="${net.key}" data-node-inner-panel="log" hidden>
        <div class="node-v6-log-toolbar">
          <button type="button" data-node-action="copy-log" data-net="${net.key}">Copy Log</button>
          <button type="button" data-node-action="clear-log" data-net="${net.key}">Clear Log</button>
        </div>
        <pre id="${id(net.key, "logOutput")}" class="node-v6-log">${net.label} log is empty.</pre>
      </div>
</div>`;
}

function renderAllNetworks(root) {
  const host = root.querySelector("#nodeNetworkPanels");
  if (!host) return;
  host.innerHTML = NODE_NETWORKS.map(renderNetworkPanel).join("");


  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
}


function buildCommandLines(net) {
  const profile = NODE_NETWORKS.find((item) => item.key === net);
  const lines = ["kaspad"];

  if (profile?.testnet) {
    addFlag(lines, net, "testnet", "--testnet");
    addValue(lines, net, "netsuffix", "--netsuffix");
  }

  addValue(lines, net, "logLevel", "--loglevel");
  addValue(lines, net, "asyncThreads", "--async-threads");
  addValue(lines, net, "ramScale", "--ram-scale");
  addFlag(lines, net, "yes", "--yes");
  addFlag(lines, net, "noLogFiles", "--nologfiles");
  addFlag(lines, net, "sanity", "--sanity");
  addFlag(lines, net, "enableUnsyncedMining", "--enable-unsynced-mining");

  addHostPort(lines, net, "listenEnabled", "--listen", "listenHost", "listenPort");
  addHostPort(lines, net, "externalIpEnabled", "--externalip", "externalIpHost", "externalIpPort");
  addFlag(lines, net, "disableUpnp", "--disable-upnp");
  addFlag(lines, net, "noDnsSeed", "--nodnsseed");
  addValue(lines, net, "uaComment", "--uacomment");

  addHostPort(lines, net, "rpcListenEnabled", "--rpclisten", "rpcListenHost", "rpcListenPort");
  addHostPort(lines, net, "rpcBorshEnabled", "--rpclisten-borsh", "rpcBorshHost", "rpcBorshPort");
  addHostPort(lines, net, "rpcJsonEnabled", "--rpclisten-json", "rpcJsonHost", "rpcJsonPort");
  addValue(lines, net, "rpcMaxClients", "--rpcmaxclients");
  addFlag(lines, net, "unsafeRpc", "--unsaferpc");
  addFlag(lines, net, "noGrpc", "--nogrpc");

  addHostPort(lines, net, "connectEnabled", "--connect", "connectHost", "connectPort");
  addHostPort(lines, net, "addPeerEnabled", "--addpeer", "addPeerHost", "addPeerPort");
  addValue(lines, net, "outPeers", "--outpeers");
  addValue(lines, net, "maxInPeers", "--maxinpeers");

  addFlag(lines, net, "utxoIndex", "--utxoindex");
  addFlag(lines, net, "archival", "--archival");
  addFlag(lines, net, "resetDb", "--reset-db");
  addFlag(lines, net, "perfMetrics", "--perf-metrics");
  addValue(lines, net, "maxTrackedAddresses", "--max-tracked-addresses");
  addValue(lines, net, "retentionDays", "--retention-period-days");
  addValue(lines, net, "perfMetricsInterval", "--perf-metrics-interval-sec");

  addValue(lines, net, "rocksDbPreset", "--rocksdb-preset");
  addValue(lines, net, "rocksDbCacheSize", "--rocksdb-cache-size");
  addValue(lines, net, "rocksDbWalDir", "--rocksdb-wal-dir");
  addValue(lines, net, "overrideParamsFile", "--override-params-file");

  addValue(lines, net, "configFile", "--configfile");
  addValue(lines, net, "appDir", "--appdir");
  addValue(lines, net, "logDir", "--logdir");

  return lines;
}


function kgwExtractNodeOwnerFlags(result) {
  const raw = stringifyRuntimeResult(result);
  const fields = {};

  for (const part of raw.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;

    const key = part.slice(0, index).trim();
    const value = part.slice(index + 1).trim();
    if (key) fields[key] = value;
  }

  return fields.flags || "";
}

async function kgwLoadNodeOwnerCommandPreview(net, fallbackText) {
  const invoke = getTauriInvoke();
  if (!invoke) return fallbackText;

  try {
    const result = await invokeWithTimeout(
      invoke,
      KGW_NODE_RUNTIME_FLAGS_OWNER_COMMAND,
      { network: net },
      KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
    );

    const flags = kgwExtractNodeOwnerFlags(result);
    return flags ? "kaspad " + flags : fallbackText;
  } catch (_) {
    return fallbackText;
  }
}

function updateCommand(net) {
  const preview = byId(id(net, "commandPreview"));
  if (!preview) return;

  const lines = buildCommandLines(net);
  const first = lines.shift() || "kaspad";
  const fallbackText = lines.length ? `${first} ${lines.join(" ")}` : first;

  preview.value = fallbackText;

  kgwLoadNodeOwnerCommandPreview(net, fallbackText).then((ownerText) => {
    if (ownerText) preview.value = ownerText;
  });
}

function updateAllCommands() {
  NODE_NETWORKS.forEach((net) => updateCommand(net.key));
}


function installNetworkTabs(root) {
  const tabs = root.querySelectorAll("[data-node-network-tab]");
  const panels = root.querySelectorAll("[data-node-network-panel]");

  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      const selected = tab.dataset.nodeNetworkTab;

      tabs.forEach((item) => item.classList.toggle("active", item === tab));

      panels.forEach((panel) => {
        const active = panel.dataset.nodeNetworkPanel === selected;
        panel.classList.toggle("active", active);
        panel.hidden = !active;
      });
    });
  });
}

function installDelegatedTabs(root) {
  root.addEventListener("click", (event) => {
    const innerTab = event.target.closest("[data-node-inner-tab]");
    if (innerTab) {
      const net = innerTab.dataset.net;
      const selected = innerTab.dataset.nodeInnerTab;
      const panel = root.querySelector(`[data-node-network-panel="${net}"]`);

      panel.querySelectorAll("[data-node-inner-tab]").forEach((item) => {
        item.classList.toggle("active", item === innerTab);
      });

      panel.querySelectorAll("[data-node-inner-panel]").forEach((item) => {
        const active = item.dataset.nodeInnerPanel === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });

      return;
    }

    const sectionTab = event.target.closest("[data-node-section-tab]");
    if (sectionTab) {
      const net = sectionTab.dataset.net;
      const selected = sectionTab.dataset.nodeSectionTab;
      const panel = root.querySelector(`[data-node-network-panel="${net}"]`);

      panel.querySelectorAll("[data-node-section-tab]").forEach((item) => {
        item.classList.toggle("active", item === sectionTab);
      });

      panel.querySelectorAll("[data-node-section-panel]").forEach((item) => {
        const active = item.dataset.nodeSectionPanel === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });
    }
  });
}

// KGW_NODE_INTEGRATED_RUNTIME_LINKAGE_V1: crash-safe Node action owner calls registered Tauri integrated runtime commands.
const KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS = 30000;
const KGW_NODE_RUNTIME_FLAGS_OWNER_COMMAND = "rk_integrated_node_runtime_flags_v1";

function getTauriInvoke() {
  const tauri = window.__TAURI__;
  return tauri?.core?.invoke || tauri?.invoke || window.__TAURI_INVOKE__ || null;
}

function stringifyRuntimeResult(result) {
  if (result == null) return "No response";
  if (typeof result === "string") return result;
  try {
    return JSON.stringify(result);
  } catch {
    return String(result);
  }
}

function normalizeRuntimeError(error) {
  if (error == null) return "Unknown backend error";
  if (typeof error === "string") return error;
  if (error.message) return error.message;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function parseRuntimeFields(result) {
  const raw = stringifyRuntimeResult(result);
  const fields = {};

  for (const part of raw.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;

    const key = part.slice(0, index).trim();
    const value = part.slice(index + 1).trim();
    if (key) fields[key] = value;
  }

  return fields;
}


function nodeRuntimeArgs(net, command) {
  if (command === "kgw_kgw_apply_node_settings_v1") {
    const preview = byId(id(net, "commandPreview"))?.value || "";

    return {
      network: net,
      nodeKind: "integrated-as-daemon",
      bridgeKind: "disable",
      nodeCommandPreview: preview,
      bridgeCommandPreview: "",
      runtimeRole: "node",
    };
  }

  if (
    command === "kgw_kgw_disable_network_v1" ||
    command === "kgw_runtime_owner_status_v1" ||
    command === "kgw_kgw_runtime_logs_v1"
  ) {
    return { network: net, runtimeRole: "node" };
  }

  return { network: net };
}
function invokeWithTimeout(invoke, command, args, timeoutMs) {
  let timer = null;

  const timeout = new Promise((_, reject) => {
    timer = window.setTimeout(() => {
      reject(new Error(command + " timed out after " + timeoutMs + "ms"));
    }, timeoutMs);
  });

  return Promise.race([
    invoke(command, args),
    timeout
  ]).finally(() => {
    if (timer != null) window.clearTimeout(timer);
  });
}

async function invokeNodeIntegratedRuntime(command, net) {
  const invoke = getTauriInvoke();
  if (!invoke) {
    throw new Error("Tauri invoke is unavailable in this window.");
  }

  return await invokeWithTimeout(invoke, command, nodeRuntimeArgs(net, command), KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS);
}


async function runNodeIntegratedAction(action, net) {
  const commandByAction = {
    start: "kgw_kgw_apply_node_settings_v1",
    stop: "kgw_kgw_disable_network_v1"
  };

  const command = commandByAction[action];
  if (!command) return false;

  if (!window.__kgwR29NodeInFlight) {
    window.__kgwR29NodeInFlight = new Set();
  }

  const inFlightKey = net + ":" + action;
  if (window.__kgwR29NodeInFlight.has(inFlightKey)) {
    appendLog(net, "KGW node " + action + " already in progress. Duplicate click ignored.");
    return true;
  }

  window.__kgwR29NodeInFlight.add(inFlightKey);

  try {
    updateCommand(net);
    appendLog(net, "KGW node " + action + " requested via existing button.");
    appendLog(net, "KGW node flags: " + (byId(id(net, "commandPreview"))?.value || ""));

    const result = await invokeNodeIntegratedRuntime(command, net);
    appendLog(net, "KGW node " + action + " response: " + stringifyRuntimeResult(result));

    if (action === "start") {

    }
return true;
  } catch (error) {
    appendLog(net, "KGW node " + action + " failed: " + normalizeRuntimeError(error));
    return true;
  } finally {
    window.__kgwR29NodeInFlight.delete(inFlightKey);
  }
}
/* KGW_R51_DIRECT_NODE_LOG_RUNTIME_SETTINGS_OWNER */
const KGW_NODE_R51_STORAGE_PREFIX = "kgw.node.direct.v51.";
const KGW_NODE_R51_LAST_STATUS = {};
const KGW_NODE_R51_LAST_LOGS = {};
const KGW_NODE_R51_LAST_ACTIVITY_NOTICE = {};
let KGW_NODE_R51_TIMER = null;

function kgwNodeR51Keys() {
  return NODE_NETWORKS.map((item) => item.key);
}

function kgwNodeR51Panel(net) {
  return document.querySelector(`[data-node-network-panel="${net}"]`);
}

function kgwNodeR51Fields(net) {
  const panel = kgwNodeR51Panel(net);
  if (!panel) return [];

  return Array.from(panel.querySelectorAll("input, select, textarea")).filter((field) => {
    if (!field.id || !field.id.startsWith(`node-${net}-`)) return false;
    if (field.id.endsWith("-commandPreview")) return false;
    if (field.id.endsWith("-logOutput")) return false;
    if (field.readOnly) return false;
    return true;
  });
}


/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START */
function kgwNodeSettingsActionIsR6(action) {
  return action === "save-settings" || action === "restore-defaults" || action === "set-defaults";
}

function kgwNodeNetFromSettingsEventR6(event, fallbackNet = "") {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [data-node-settings-panel], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i]");

  const raw = [
    target?.dataset?.net,
    target?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    target?.id,
    carrier?.id,
    carrier?.className,
    fallbackNet,
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";
  return fallbackNet || "";
}


/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_END */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */
const kgwNodeSettingsFeedbackLocksR11 = new Map();


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

function kgwNodeR51ReadSettings(net) {
  const values = {};

  for (const field of kgwNodeR51Fields(net)) {
    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }

  return values;
}

function kgwNodeR51WriteSettings(net, values) {
  if (!values || typeof values !== "object") return;

  for (const field of kgwNodeR51Fields(net)) {
    const item = values[field.id];
    if (!item) continue;

    if (field.type === "checkbox") {
      field.checked = Boolean(item.checked);
    } else if ("value" in item) {
      field.value = String(item.value ?? "");
    }

    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
  }

  updateCommand(net);
}

function kgwNodeR51Store(key, value) {
  localStorage.setItem(KGW_NODE_R51_STORAGE_PREFIX + key, JSON.stringify(value));
}

function kgwNodeR51Load(key) {
  try {
    return JSON.parse(localStorage.getItem(KGW_NODE_R51_STORAGE_PREFIX + key) || "null");
  } catch {
    return null;
  }
}

function kgwNodeR51CaptureFactoryDefaults() {
  for (const net of kgwNodeR51Keys()) {
    if (!kgwNodeR51Load("factory:" + net)) {
      kgwNodeR51Store("factory:" + net, kgwNodeR51ReadSettings(net));
    }
  }
}

function kgwNodeR51LoadSavedSettings() {
  for (const net of kgwNodeR51Keys()) {
    const saved = kgwNodeR51Load("saved:" + net);
    if (saved) {
      kgwNodeR51WriteSettings(net, saved);
      appendLog(net, "Saved node settings loaded.");
    }
  }
}

/* KGW_NODE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */


function kgwNodeR51SaveSettings(net) {
  kgwNodeR51Store("saved:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Node settings saved successfully.");
}

function kgwNodeR51SetAsDefaults(net) {
  kgwNodeR51Store("default:" + net, kgwNodeR51ReadSettings(net));
  appendLog(net, "Current node settings saved as defaults.");
}

function kgwNodeR51RestoreDefaults(net) {
  kgwNodeSettingsWithProgrammaticWriteR9B(() => {
    const defaults = kgwNodeR51Load("default:" + net) || kgwNodeR51Load("factory:" + net);
    kgwNodeR51WriteSettings(net, defaults);
    kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
    appendLog(net, "Node defaults restored successfully.");
  });
}

function kgwNodeR51IsRunning(text) {
  const value = String(text || "");
  return /running=true/.test(value) || /node_running=true/.test(value) || /official_core_running=true/.test(value);
}

function kgwNodeR51SetRuntimeButtons(net, running) {
  const panel = kgwNodeR51Panel(net);
  if (!panel) return;

  const start = panel.querySelector(`[data-node-action="start"][data-net="${net}"]`);
  const stop = panel.querySelector(`[data-node-action="stop"][data-net="${net}"]`);

  if (start) {
    start.disabled = Boolean(running);
    start.style.opacity = running ? "0.45" : "";
    start.style.cursor = running ? "not-allowed" : "";
    start.title = running ? "Node is running. Stop it before starting again." : "Start node";
  }

  if (stop) {
    stop.disabled = !running;
    stop.style.opacity = running ? "" : "0.45";
    stop.style.cursor = running ? "" : "not-allowed";
    stop.title = running ? "Stop node" : "Node is not running";
  }
}

function kgwNodeR51Delta(previous, current) {
  const before = String(previous || "");
  const after = String(current || "");

  if (!after || before === after) return "";
  if (after.startsWith(before)) return after.slice(before.length).trim();

  return after.trim();
}

function kgwNodeR51MaybeActivityNotice(net, statusText) {
  const now = Date.now();
  const last = KGW_NODE_R51_LAST_ACTIVITY_NOTICE[net] || 0;

  if (now - last < 15000) return;

  if (!kgwNodeR51IsRunning(statusText)) return;

  KGW_NODE_R51_LAST_ACTIVITY_NOTICE[net] = now;

}

async function kgwNodeR51RefreshOne(net, reason = "live") {
  try {
    const status = stringifyRuntimeResult(await invokeNodeIntegratedRuntime("kgw_runtime_owner_status_v1", net));
    kgwNodeR51SetRuntimeButtons(net, kgwNodeR51IsRunning(status));

    if (KGW_NODE_R51_LAST_STATUS[net] !== status) {
      KGW_NODE_R51_LAST_STATUS[net] = status;
      
    }

    kgwNodeR51MaybeActivityNotice(net, status);
  } catch (error) {
    kgwNodeR51SetRuntimeButtons(net, false);
  }

  try {
    const logs = stringifyRuntimeResult(await invokeNodeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net));
    const delta = kgwNodeR51Delta(KGW_NODE_R51_LAST_LOGS[net], logs);

    if (delta) {
      KGW_NODE_R51_LAST_LOGS[net] = logs;
      appendLog(net, delta);
    }
  } catch {
    // Runtime may not be ready yet.
  }
}

function kgwNodeR51RefreshAll(reason = "live") {
  for (const net of kgwNodeR51Keys()) {
    kgwNodeR51RefreshOne(net, reason);
  }
}

function kgwNodeR51StartLiveRefresh() {
  if (KGW_NODE_R51_TIMER != null) {
    clearInterval(KGW_NODE_R51_TIMER);
  }

  kgwNodeR51RefreshAll("initial");

  KGW_NODE_R51_TIMER = setInterval(() => {
    kgwNodeR51RefreshAll("poll");
  }, 700);
}

function installKgwNodeR51BottomStyle() {
  if (document.getElementById("kgw-node-r51-bottom-style")) return;

  const style = document.createElement("style");
  style.id = "kgw-node-r51-bottom-style";
  style.textContent = `
    [data-node-network-panel] {
      position: relative;
      min-height: 680px;
      padding-bottom: 48px;
    }

    .node-settings-bottom-actions {
      position: absolute;
      right: 12px;
      bottom: 8px;
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      z-index: 2;
    }

    .node-settings-bottom-actions button {
      min-width: 112px;
      height: 28px;
      padding: 4px 10px;
      border: 1px solid rgba(148, 163, 184, 0.55);
      background: rgba(80, 80, 80, 0.9);
      color: #fff;
      cursor: pointer;
      font-size: 12px;
      line-height: 1;
    }
  `;
  document.head.appendChild(style);
}

/* KGW_NODE_SETTINGS_BUTTON_FEEDBACK_FIX_R1
 * Settings action buttons must confirm successful user actions immediately.
 * The existing Node action owner calls this helper after save/restore/set-default succeeds.
 */
/* KGW_NODE_SETTINGS_BUTTON_FEEDBACK_HOLD_FIX_R2
 * Keep settings button success labels visible long enough for the user.
 * The helper repeats the label during the hold window to survive fast UI re-renders.
 */


function installActions(root) {
  // KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26: Node settings actions are scoped to the exact network that changed.
  if (window.KGW_NODE_SETTINGS_OWNER_V19 && typeof window.KGW_NODE_SETTINGS_OWNER_V19.install === "function") {
    window.KGW_NODE_SETTINGS_OWNER_V19.install(root);
  }

  function normalizeNet(value) {
    const raw = String(value || "").toLowerCase();
    if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
    if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
    if (raw.includes("mainnet")) return "mainnet";
    return "";
  }

  function netFromElement(element) {
    if (!element) return "";
    const carrier = element.closest("[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel], [data-node-section-panel]");

    return normalizeNet(
      [
        element.dataset && element.dataset.net,
        element.dataset && element.dataset.network,
        carrier && carrier.dataset && carrier.dataset.net,
        carrier && carrier.dataset && carrier.dataset.network,
        carrier && carrier.dataset && carrier.dataset.nodeNetworkPanel,
        element.id,
        carrier && carrier.id,
        carrier && carrier.className
      ].filter(Boolean).join(" ")
    );
  }

  function netFromEvent(event) {
    return netFromElement(event && event.target);
  }

  function scopedUpdate(net, reason) {
    if (!net) return;
    if (typeof updateCommand === "function") {
      updateCommand(net);
    }
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
      window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", {
        scope: "node",
        net: net,
        action: "settings-scope",
        phase: "v26-scoped-update",
        details: JSON.stringify({ patch: "KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26", network: net, reason: reason || "unknown" })
      }).catch(function () {});
    }
  }

  root.addEventListener("input", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-change" : "programmatic-change");
  }, true);

  root.addEventListener("click", (event) => {
    const button = event.target && event.target.closest ? event.target.closest("[data-node-action]") : null;
    if (!button || !root.contains(button)) return;

    const action = button.dataset.nodeAction;
    const net = normalizeNet(button.dataset.net || button.dataset.network || netFromElement(button));

    if (!net) return;

    if (action === "save-settings") {
      if (typeof kgwNodeR51SaveSettings === "function") kgwNodeR51SaveSettings(net);
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      if (typeof kgwNodeR51SetAsDefaults === "function") kgwNodeR51SetAsDefaults(net);
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      if (typeof kgwNodeR51RestoreDefaults === "function") kgwNodeR51RestoreDefaults(net);
      scopedUpdate(net, "restore-defaults");
      return;
    }

    if (action === "copy-command") {
      const preview = document.getElementById("node-" + net + "-commandPreview");
      if (preview && navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(String(preview.value || preview.textContent || "")).catch(function () {});
      }
      return;
    }

    if (action === "start" || action === "stop") {
      if (typeof runNodeIntegratedAction === "function") {
        runNodeIntegratedAction(action, net).catch(function (error) {
          if (typeof appendLog === "function") appendLog(net, "Node " + action + " failed: " + (error && error.message ? error.message : String(error)));
        });
      }
    }
  }, false);
}

/* KGW_NODE_LOG_BUTTON_FEEDBACK_OWNER_V1 */

/* KGW_NODE_LOG_FEEDBACK_I18N_OWNER_V1 */

function kgwNodeTranslateRuntime(key, fallback) {
  const runtime = window.kgwT || window.kgwI18n || window.__kgwT;
  if (typeof runtime === "function") {
    try {
      const value = runtime(key);
      if (value && value !== key) return value;
    } catch {
      // Translation fallback must never break button feedback.
    }
  }

  const dict =
    window.__kgwI18nDictR107 ||
    window.__kgwI18nDict ||
    window.kgwI18nDict ||
    window.__KGW_I18N_DICT__;

  if (dict && typeof dict === "object") {
    const flat = dict[key];
    if (typeof flat === "string" && flat.trim()) return flat;

    let node = dict;
    for (const part of String(key).split(".")) {
      if (!node || typeof node !== "object") {
        node = null;
        break;
      }
      node = node[part];
    }

    if (typeof node === "string" && node.trim()) return node;
  }

  return fallback || key;
}

function kgwNodeFlashLogActionButton(button, doneLabel) {
  if (!button) return;

  if (!button.dataset.kgwOriginalLabel) {
    button.dataset.kgwOriginalLabel = String(button.textContent || "").trim();
  }

  button.dataset.kgwDoneLabel = doneLabel;
  button.classList.add("kgw-log-action-feedback");

  window.clearTimeout(button.__kgwLogActionFeedbackTimer);
  button.__kgwLogActionFeedbackTimer = window.setTimeout(() => {
    button.classList.remove("kgw-log-action-feedback");
    delete button.dataset.kgwDoneLabel;
  }, 1400);
}

function kgwInstallNodeLogButtonFeedback(root) {
  if (!root || root.dataset.kgwNodeLogButtonFeedback === "true") return;
  root.dataset.kgwNodeLogButtonFeedback = "true";

  root.addEventListener("click", (event) => {
    const button = event.target.closest("[data-node-action='copy-log'], [data-node-action='clear-log']");
    if (!button || !root.contains(button)) return;

    const action = button.dataset.nodeAction;
    window.setTimeout(() => {
      kgwNodeFlashLogActionButton(button, action === "copy-log" ? kgwNodeTranslateRuntime("log.copied", "Copied") : kgwNodeTranslateRuntime("log.deleted", "Deleted"));
    }, 0);
  }, true);
}

export async function initKaspaNodeTab(root) {

const nodeRoot = root || document.getElementById("kaspa-node");
  if (!nodeRoot || nodeRoot.dataset.kgwNodeV6Ready === "true") return;

  nodeRoot.dataset.kgwNodeV6Ready = "true";

  renderAllNetworks(nodeRoot);
  kgwNodeR51CaptureFactoryDefaults();
  kgwNodeR51LoadSavedSettings();
  installNetworkTabs(nodeRoot);
  installDelegatedTabs(nodeRoot);
  installActions(nodeRoot);
  kgwInstallNodeLogButtonFeedback(nodeRoot);
  updateAllCommands();
  NODE_NETWORKS.forEach((net) => kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net.key, { force: false })); /* KGW_NODE_DYNAMIC_PATHS_INIT_R3 */
  installKgwNodeR51BottomStyle();
  kgwNodeR51StartLiveRefresh();

  NODE_NETWORKS.forEach((net) => appendLog(net.key, `${net.label} initialized.`));


  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
}

export default initKaspaNodeTab;

if (typeof window !== "undefined") {
  window.initKaspaNodeTab = initKaspaNodeTab;
}

function kgwNodeParseKeyValueResponse(text) {
  return String(text || "")
    .split(";")
    .map((part) => part.trim())
    .filter(Boolean)
    .reduce((acc, part) => {
      const index = part.indexOf("=");
      if (index > 0) {
        acc[part.slice(0, index)] = part.slice(index + 1);
      }
      return acc;
    }, {});
}

/* kgwSuperMegaIsolatedAdapterStatusPreviewV1
 * Phase V42-V48 SuperMega:
 * Preview selected isolated runtime adapter owner without starting runtime.
 * This is status/route preview only.
 */
async function kgwSuperMegaIsolatedAdapterStatusPreviewV1(network) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_IPC__;

  if (typeof invoke !== "function") {
    console.warn("[kaspa-node] isolated adapter preview unavailable: Tauri invoke not found");
    return null;
  }

  try {
    const result = await invoke("rk_isolated_adapter_status_preview_v1", { network });
    console.log("[kaspa-node] isolated adapter preview:", result);
    return result;
  } catch (error) {
    console.warn("[kaspa-node] isolated adapter preview failed:", error);
    return null;
  }
}

window.kgwSuperMegaIsolatedAdapterStatusPreviewV1 = kgwSuperMegaIsolatedAdapterStatusPreviewV1;

/* kgwFinalIsolatedAdapterRuntimeV1
 * Final isolated adapter runtime bridge.
 * These helpers call real Start/Status/Stop IPC commands.
 */
async function kgwFinalIsolatedAdapterInvokeV1(command, payload) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_IPC__;

  if (typeof invoke !== "function") {
    console.warn("[kaspa-node] final isolated runtime unavailable: Tauri invoke not found");
    return null;
  }

  try {
    const result = await invoke(command, payload);
    console.log("[kaspa-node] final isolated runtime:", command, result);
    return result;
  } catch (error) {
    console.warn("[kaspa-node] final isolated runtime failed:", command, error);
    return null;
  }
}

async function kgwFinalIsolatedAdapterStartV1(network, appDirName) {
  return kgwFinalIsolatedAdapterInvokeV1("rk_final_isolated_adapter_start_v1", {
    network,
    appDirName,
  });
}

async function kgwFinalIsolatedAdapterStatusV1(network) {
  return kgwFinalIsolatedAdapterInvokeV1("rk_final_isolated_adapter_status_v1", {
    network,
  });
}

async function kgwFinalIsolatedAdapterStopV1(network) {
  return kgwFinalIsolatedAdapterInvokeV1("rk_final_isolated_adapter_stop_v1", {
    network,
  });
}

window.kgwFinalIsolatedAdapterStartV1 = kgwFinalIsolatedAdapterStartV1;
window.kgwFinalIsolatedAdapterStatusV1 = kgwFinalIsolatedAdapterStatusV1;
window.kgwFinalIsolatedAdapterStopV1 = kgwFinalIsolatedAdapterStopV1;

/* kgwV66FinalRuntimeIsolationV1
 * Final runtime feature isolation:
 * - mainnet/testnet10 require isolated-real-runtime-mainline build
 * - testnet12 requires isolated-real-runtime-tn12 build
 * - one binary must not link both Rusty Kaspa owners.
 */
async function kgwV66Invoke(command, payload) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_IPC__;

  if (typeof invoke !== "function") {
    console.warn("[kaspa-node] V66 isolated runtime unavailable: Tauri invoke not found");
    return null;
  }

  try {
    const result = await invoke(command, payload);
    console.log("[kaspa-node] V66 isolated runtime:", command, result);
    return result;
  } catch (error) {
    console.warn("[kaspa-node] V66 isolated runtime failed:", command, error);
    return null;
  }
}

async function kgwV66RuntimeFeaturePolicyV1(network) {
  return kgwV66Invoke("rk_v66_runtime_feature_policy_v1", { network });
}

async function kgwV66IsolatedAdapterStartV1(network, appDirName) {
  return kgwV66Invoke("rk_v66_isolated_adapter_start_v1", { network, appDirName });
}

async function kgwV66IsolatedAdapterStatusV1(network) {
  return kgwV66Invoke("rk_v66_isolated_adapter_status_v1", { network });
}

async function kgwV66IsolatedAdapterStopV1(network) {
  return kgwV66Invoke("rk_v66_isolated_adapter_stop_v1", { network });
}

window.kgwV66RuntimeFeaturePolicyV1 = kgwV66RuntimeFeaturePolicyV1;
window.kgwV66IsolatedAdapterStartV1 = kgwV66IsolatedAdapterStartV1;
window.kgwV66IsolatedAdapterStatusV1 = kgwV66IsolatedAdapterStatusV1;
window.kgwV66IsolatedAdapterStopV1 = kgwV66IsolatedAdapterStopV1;

/* kgwV67FinalRuntimeStartStopRewireV1
 * Final UI rewire:
 * Start/Status/Stop must call V66 isolated adapter commands, not old rk_integrated_node_* commands.
 */
async function kgwV67Invoke(command, payload) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_IPC__;

  if (typeof invoke !== "function") {
    console.warn("[kaspa-node] V67 runtime unavailable: Tauri invoke not found");
    return null;
  }

  try {
    const result = await invoke(command, payload);
    console.log("[kaspa-node] V67 runtime:", command, result);
    return result;
  } catch (error) {
    console.warn("[kaspa-node] V67 runtime failed:", command, error);
    return null;
  }
}

async function kgwV67StartRuntime(network, appDirName) {
  return kgwV67Invoke("rk_v66_isolated_adapter_start_v1", {
    network,
    appDirName,
  });
}

async function kgwV67StatusRuntime(network) {
  return kgwV67Invoke("rk_v66_isolated_adapter_status_v1", {
    network,
  });
}

async function kgwV67StopRuntime(network) {
  return kgwV67Invoke("rk_v66_isolated_adapter_stop_v1", {
    network,
  });
}

async function kgwV67RuntimeFeaturePolicy(network) {
  return kgwV67Invoke("rk_v66_runtime_feature_policy_v1", {
    network,
  });
}

window.kgwV67StartRuntime = kgwV67StartRuntime;
window.kgwV67StatusRuntime = kgwV67StatusRuntime;
window.kgwV67StopRuntime = kgwV67StopRuntime;
window.kgwV67RuntimeFeaturePolicy = kgwV67RuntimeFeaturePolicy;

/* R35 settings persistence for existing Node tab. */
/* R37 bottom placement for Node settings buttons. */
/* R38 UI freeze protection for Node Start/Stop. */

(function installKgwLogFontSizeToolbarControlsV2() {
  "use strict";

  const MARKER = "KGW_LOG_FONT_SIZE_CONTROLS_V2";
  const KIND = "node";
  const NETWORKS = ["mainnet", "testnet10", "testnet12"];
  const MIN_SIZE = 10;
  const MAX_SIZE = 18;
  const DEFAULT_SIZE = 12;

  let scheduled = false;

  function storageKey(network) {
    return "kgw." + KIND + ".log.fontSize." + network;
  }

  function clampSize(value) {
    const parsed = Number.parseInt(String(value || ""), 10);
    if (!Number.isFinite(parsed)) return DEFAULT_SIZE;
    return Math.max(MIN_SIZE, Math.min(MAX_SIZE, parsed));
  }

  function readSize(network) {
    try {
      return clampSize(window.localStorage.getItem(storageKey(network)));
    } catch (_) {
      return DEFAULT_SIZE;
    }
  }

  function writeSize(network, size) {
    const finalSize = clampSize(size);
    try {
      window.localStorage.setItem(storageKey(network), String(finalSize));
    } catch (_) {}
    return finalSize;
  }

  function textOf(element) {
    return String(
      [
        element && element.id,
        element && element.className,
        element && element.textContent,
        element && element.getAttribute && element.getAttribute("aria-label"),
        element && element.getAttribute && element.getAttribute("data-network"),
        element && element.getAttribute && element.getAttribute("data-kgw-network")
      ]
        .filter(Boolean)
        .join(" ")
    ).toLowerCase();
  }

  function isVisible(element) {
    if (!element || !element.getBoundingClientRect) return false;
    const rect = element.getBoundingClientRect();
    const style = window.getComputedStyle(element);
    return rect.width > 0 && rect.height > 0 && style.display !== "none" && style.visibility !== "hidden";
  }

  function activeNetwork() {
    const selected = Array.from(
      document.querySelectorAll(
        "[aria-selected='true'], .active, .is-active, .selected, [data-active='true']"
      )
    ).filter(isVisible);

    for (const element of selected) {
      const t = textOf(element);
      for (const network of NETWORKS) {
        if (t.includes(network)) return network;
        if (network === "testnet10" && t.includes("testnet 10")) return network;
        if (network === "testnet12" && t.includes("testnet 12")) return network;
      }
    }

    const visibleLogs = Array.from(document.querySelectorAll("[id*='log'], [class*='log'], pre, textarea"))
      .filter(isVisible);

    for (const element of visibleLogs) {
      let current = element;
      while (current && current !== document.documentElement) {
        const t = textOf(current);
        for (const network of NETWORKS) {
          if (t.includes(network)) return network;
          if (network === "testnet10" && t.includes("testnet 10")) return network;
          if (network === "testnet12" && t.includes("testnet 12")) return network;
        }
        current = current.parentElement;
      }
    }

    return "mainnet";
  }

  function looksLikeLogPane(element) {
    if (!element || !isVisible(element)) return false;

    const tag = String(element.tagName || "").toLowerCase();
    const t = textOf(element);

    if (!(tag === "pre" || tag === "textarea" || tag === "code" || tag === "div")) return false;
    if (!t.includes("log")) return false;
    if (element.classList && element.classList.contains("kgw-log-font-size-controls")) return false;

    const rect = element.getBoundingClientRect();
    if (rect.height < 80) return false;

    return true;
  }

  function visibleLogPanes() {
    return Array.from(
      document.querySelectorAll("pre, textarea, code, [id*='log'], [class*='log'], [data-role*='log']")
    ).filter(looksLikeLogPane);
  }

  function applyFontSize() {
    const network = activeNetwork();
    const size = readSize(network);

    for (const pane of visibleLogPanes()) {
      pane.dataset.kgwLogFontSizePane = "1";
      pane.style.setProperty("--kgw-log-font-size", size + "px");
      pane.style.fontSize = "var(--kgw-log-font-size)";
      pane.style.lineHeight = "1.45";
    }

    const value = document.querySelector(".kgw-log-font-size-controls[data-kind='" + KIND + "'] .kgw-log-font-size-value");
    if (value) value.textContent = size + "px";
  }

  function button(label, title) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "kgw-log-font-size-button";
    b.textContent = label;
    b.title = title;
    b.setAttribute("aria-label", title);
    return b;
  }

  function findVisibleLogToolbar() {
    const existingToolbars = Array.from(
      document.querySelectorAll(
        ".kgw-log-toolbar, .log-toolbar, [class*='log-toolbar'], [role='toolbar']"
      )
    ).filter(isVisible);

    for (const toolbar of existingToolbars) {
      const t = textOf(toolbar);
      if (t.includes("copy log") || t.includes("clear log") || t.includes("auto-scroll") || t.includes("autoscroll")) {
        return toolbar;
      }
    }

    const buttons = Array.from(document.querySelectorAll("button")).filter(isVisible);
    for (const b of buttons) {
      const t = textOf(b);
      if (t.includes("copy log") || t.includes("clear log")) {
        return b.parentElement;
      }
    }

    return null;
  }

  function removeDuplicateControls() {
    const controls = Array.from(document.querySelectorAll(".kgw-log-font-size-controls"));
    controls.forEach((control, index) => {
      if (index > 0) control.remove();
    });
  }

  function installOneToolbarControl() {
    const toolbar = findVisibleLogToolbar();
    if (!toolbar) return;

    removeDuplicateControls();

    let controls = toolbar.querySelector(".kgw-log-font-size-controls[data-kind='" + KIND + "']");
    if (!controls) {
      controls = document.createElement("div");
      controls.className = "kgw-log-font-size-controls";
      controls.dataset.kind = KIND;
      controls.dataset.marker = MARKER;

      const decrease = button("A-", "Decrease log font size");
      const value = document.createElement("span");
      value.className = "kgw-log-font-size-value";
      value.textContent = readSize(activeNetwork()) + "px";

      const increase = button("A+", "Increase log font size");
      const reset = button("Reset", "Reset log font size");

      decrease.addEventListener("click", () => {
        const network = activeNetwork();
        writeSize(network, readSize(network) - 1);
        applyFontSize();
      });

      increase.addEventListener("click", () => {
        const network = activeNetwork();
        writeSize(network, readSize(network) + 1);
        applyFontSize();
      });

      reset.addEventListener("click", () => {
        writeSize(activeNetwork(), DEFAULT_SIZE);
        applyFontSize();
      });

      controls.append(decrease, value, increase, reset);
      toolbar.appendChild(controls);
    }

    applyFontSize();
  }

  function run() {
    scheduled = false;
    installOneToolbarControl();
  }

  function schedule() {
    if (scheduled) return;
    scheduled = true;
    window.requestAnimationFrame(run);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", schedule, { once: true });
  } else {
    schedule();
  }

  document.addEventListener("click", schedule, true);
  document.addEventListener("change", schedule, true);

  const observer = new MutationObserver(schedule);
  observer.observe(document.documentElement, { childList: true, subtree: true });

  window.addEventListener("storage", (event) => {
    if (event.key && event.key.startsWith("kgw." + KIND + ".log.fontSize.")) {
      schedule();
    }
  });
})();


