// KGW_SETTINGS_OWNER_V19
(function installKgwSettingsOwnerV19() {
  "use strict";

  const OWNER = "KGW_SETTINGS_OWNER_V19";
  const PATCH = "KGW_SETTINGS_OWNER_V19_SAFE_FEEDBACK_NO_FREEZE_V25B";
  const SCOPE = "bridge";
  const GLOBAL_NAME = "KGW_BRIDGE_SETTINGS_OWNER_V19";
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

      trace(root, "r44h2-input-seen", {
        patch: "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2",
        trusted: Boolean(event && event.isTrusted),
        targetId: String(event.target && event.target.id || ""),
        targetName: String(event.target && event.target.name || ""),
        targetTag: String(event.target && event.target.tagName || "")
      });

      if (!event.isTrusted) {
        setDisabled(root, network, true, "input-programmatic");
        trace(root, "r44h2-input-programmatic-disabled", {
        patch: "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2",
        trusted: Boolean(event && event.isTrusted),
        targetId: String(event.target && event.target.id || ""),
        targetName: String(event.target && event.target.name || ""),
        targetTag: String(event.target && event.target.tagName || "")
      });
        return;
      }

      clearFeedback(root, network, "trusted-input");
      restoreLabels(root, network);
      setDirty(root, network, true, "trusted-input");
      trace(root, "r44h2-trusted-input-dirty", {
        patch: "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2",
        trusted: Boolean(event && event.isTrusted),
        targetId: String(event.target && event.target.id || ""),
        targetName: String(event.target && event.target.name || ""),
        targetTag: String(event.target && event.target.tagName || "")
      });
    }, true);

    root.addEventListener("change", function (event) {
      if (!isSettingsControl(event.target)) return;

      const network = networkOf(event.target);

      trace(root, "r44h2-change-seen", {
        patch: "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2",
        trusted: Boolean(event && event.isTrusted),
        targetId: String(event.target && event.target.id || ""),
        targetName: String(event.target && event.target.name || ""),
        targetTag: String(event.target && event.target.tagName || "")
      });

      if (!event.isTrusted) {
        setDisabled(root, network, true, "change-programmatic");
        trace(root, "r44h2-change-programmatic-disabled", {
        patch: "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2",
        trusted: Boolean(event && event.isTrusted),
        targetId: String(event.target && event.target.id || ""),
        targetName: String(event.target && event.target.name || ""),
        targetTag: String(event.target && event.target.tagName || "")
      });
        return;
      }

      clearFeedback(root, network, "trusted-change");
      restoreLabels(root, network);
      setDirty(root, network, true, "trusted-change");
      trace(root, "r44h2-trusted-change-dirty", {
        patch: "KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2",
        trusted: Boolean(event && event.isTrusted),
        targetId: String(event.target && event.target.id || ""),
        targetName: String(event.target && event.target.name || ""),
        targetTag: String(event.target && event.target.tagName || "")
      });
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

function kgwBridgeSmallOwnerTraceR44D(net, action, phase, details) {
  try {
    const safeNet = String(net || "unknown");
    const safeAction = String(action || "small-owner");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "bridge",
      net: safeNet,
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_SMALL_NODE_BRIDGE_TRACE_PATCH_R44D",
        existingOwner: "bridge-small-owner-functions",
        network: safeNet,
        action: safeAction,
        phase: safePhase,
        details: safeDetails
      })
    };
    const tauri = window.__TAURI__;
    const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
      ? tauri.core.invoke.bind(tauri.core)
      : tauri && typeof tauri.invoke === "function"
        ? tauri.invoke.bind(tauri)
        : window.__TAURI_INVOKE__;
    if (typeof invoke === "function") {
      invoke("kgw_frontend_button_trace_v1", args).catch(function () {});
    }
  } catch (_) {}
}




function kgwI18nTextR41(key, fallback) {
  try {
    if (window.kgwT && typeof window.kgwT === "function") return window.kgwT(key, fallback);
    if (window.KGW_I18N && typeof window.KGW_I18N.t === "function") return window.KGW_I18N.t(key, fallback);
    if (window.i18n && typeof window.i18n.t === "function") return window.i18n.t(key, fallback);
  } catch (_) {
  }
  return fallback;
}


/* KGW_BRIDGE_RUSTY_KASPA_ROOT_ONLY_DEFAULT_PATHS_FIX_R5
 * Canonical bridge runtime default path owner.
 * The only generated default path is the current user's LocalAppData rusty-kaspa root.
 * Example runtime value: %LOCALAPPDATA%\rusty-kaspa.
 * No bridge suffix, no network suffix, no KGW app-data root.
 * Kaspa bridge/runtime owns/completes any internal layout below this root.
 */
function kgwBridgeBackendInvokeR5(command, payload = {}) {
  const invoke =
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_INVOKE__;

  if (typeof invoke !== "function") {
    return Promise.reject(new Error("Tauri invoke is not available"));
  }

  return invoke(command, payload);
}

function kgwBridgeJoinPathR5(root, child) {
  const base = String(root || "").replace(/[\\/]+$/, "");
  if (!base) return "";
  return base + "\\" + child;
}

function kgwBridgeExtractUserLocalAppDataR5(paths) {
  const values = Object.values(paths || {}).map((value) => String(value || ""));
  for (const value of values) {
    const match = value.match(/^([A-Za-z]:[\\/]Users[\\/][^\\/]+[\\/]AppData)[\\/](?:Local|Roaming)(?:[\\/].*)?$/i);
    if (match && match[1]) {
      return match[1] + "\\Local";
    }
  }
  return "%LOCALAPPDATA%";
}

function kgwBridgeRustyKaspaLocalAppDataRootR5(paths = {}) {
  return kgwBridgeJoinPathR5(kgwBridgeExtractUserLocalAppDataR5(paths), "rusty-kaspa");
}

function kgwBridgeIsEmptyOrGeneratedPathR5(value) {
  const text = String(value || "");
  return text.trim() === "" || /^[A-Za-z]:[\\/]+Users[\\/]+[^\\/]+AppData[\\/]+(?:Local|Roaming)[\\/]+(?:rusty-kaspa|KaspaGateway)(?:[\\/].*)?$/i.test(text) || /^%LOCALAPPDATA%[\\/]+rusty-kaspa(?:[\\/].*)?$/i.test(text);
}

async function kgwBridgeLoadEnvironmentPathHintsR5() {
  try {
    const defaults = await kgwBridgeBackendInvokeR5("settings_defaults");
    return defaults && defaults.paths ? defaults.paths : {};
  } catch (_) {
    return {};
  }
}

async function kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsR5(net, options = {}) {
  const force = options.force === true;
  const pathHints = await kgwBridgeLoadEnvironmentPathHintsR5();
  const rustyRoot = kgwBridgeRustyKaspaLocalAppDataRootR5(pathHints);

  const values = {
    appdir: rustyRoot,
    config: ""
  };

  Object.entries(values).forEach(([name, value]) => {
    const field = byId(id(net, name));
    if (!field) return;
    const current = String(field.value || "");
    if (force || kgwBridgeIsEmptyOrGeneratedPathR5(current)) {
      field.value = value;
      field.dispatchEvent(new Event("input", { bubbles: true }));
      field.dispatchEvent(new Event("change", { bubbles: true }));
    }
  });

  updateCommand(net);
  return values;
}

function kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, options = {}) {
  void kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsR5(net, options).catch(() => {});
}

const BRIDGE_NETWORKS = [
  { key: "mainnet", label: "MAINNET", testnet: false, netsuffix: "", kaspadPort: "16110", stratumPort: ":5555", promPort: ":2112" },
  { key: "testnet10", label: "TESTNET10", testnet: true, netsuffix: "10", kaspadPort: "16210", stratumPort: ":5556", promPort: ":2113" },
  { key: "testnet12", label: "TESTNET12", testnet: true, netsuffix: "12", kaspadPort: "16310", stratumPort: ":5557", promPort: ":2114" }
];

const bridgeInstances = {
  mainnet: [{ id: 1 }],
  testnet10: [{ id: 1 }],
  testnet12: [{ id: 1 }]
};

let activeInstance = {
  mainnet: 1,
  testnet10: 1,
  testnet12: 1
};

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
  return `bridge-${net}-${name}`;
}


function iid(net, instanceId, name) {
  return `bridge-${net}-i${instanceId}-${name}`;
}

function v(net, name) {
  const el = byId(id(net, name));
  return el ? String(el.value || "").trim() : "";
}

function c(net, name) {
  const el = byId(id(net, name));
  return Boolean(el && el.checked);
}

function iv(net, instanceId, name) {
  const el = byId(iid(net, instanceId, name));
  return el ? String(el.value || "").trim() : "";
}

function ic(net, instanceId, name) {
  const el = byId(iid(net, instanceId, name));
  return Boolean(el && el.checked);
}

function addFlag(lines, net, name, flag) {
  if (c(net, name)) lines.push(flag);
}

function addValue(lines, net, name, flag) {
  const value = v(net, name);
  if (value) lines.push(`${flag}=${value}`);
}

function addBoolValue(lines, net, name, flag) {
  const value = v(net, name);
  if (value && value !== "not set") lines.push(`${flag}=${value}`);
}


function cardInput(net, name, label, value = "", placeholder = "", span = "") {
  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <input id="${id(net, name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function cardSelect(net, name, label, options, value = "", span = "") {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <select id="${id(net, name)}">${opts}</select>
    </div>`;
}

function cardCheck(net, name, label, checked = false, span = "") {
  return `
    <label class="bridge-v7-card check${span ? " " + span : ""}">
      <input id="${id(net, name)}" type="checkbox"${checked ? " checked" : ""}>
      <span>${esc(label)}</span>
    </label>`;
}

function instanceInput(net, instanceId, name, label, value = "", placeholder = "", span = "") {
  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <input id="${iid(net, instanceId, name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function instanceSelect(net, instanceId, name, label, options, value = "", span = "") {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <select id="${iid(net, instanceId, name)}">${opts}</select>
    </div>`;
}

function instanceCheck(net, instanceId, name, label, checked = false, span = "") {
  return `
    <label class="bridge-v7-card check${span ? " " + span : ""}">
      <input id="${iid(net, instanceId, name)}" type="checkbox"${checked ? " checked" : ""}>
      <span>${esc(label)}</span>
    </label>`;
}

function renderRuntime(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardSelect(net.key, "nodeMode", "--node-mode", ["external", "inprocess"], "external")}
      ${net.key === "mainnet" ? "" : cardCheck(net.key, "testnet", "--testnet", net.testnet)}
      ${cardInput(net.key, "config", "--config", "", "config.yaml")}
      ${cardInput(net.key, "appdir", "--appdir", "", "app dir")}
      ${cardInput(net.key, "kaspadAddress", "--kaspad-address", `127.0.0.1:${net.kaspadPort}`)}
      ${cardInput(net.key, "blockWaitTime", "--block-wait-time", "50ms")}
      ${cardInput(net.key, "healthCheckPort", "--health-check-port", "", "optional")}
      ${cardInput(net.key, "webDashboardPort", "--web-dashboard-port", "", ":3030")}
    </div>`;
}

function renderDifficulty(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardInput(net.key, "minShareDiff", "--min-share-diff", "8192")}
      ${cardInput(net.key, "sharesPerMin", "--shares-per-min", "30")}
      ${cardSelect(net.key, "varDiff", "--var-diff", ["true", "false"], "true")}
      ${cardSelect(net.key, "varDiffStats", "--var-diff-stats", ["true", "false"], "true")}
      ${cardSelect(net.key, "pow2Clamp", "--pow2-clamp", ["true", "false"], "true")}
      ${cardInput(net.key, "extranonceSize", "--extranonce-size", "0")}
      ${cardInput(net.key, "coinbaseTagSuffix", "--coinbase-tag-suffix", "", "optional", "span2")}
    </div>`;
}

function renderLogging(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardSelect(net.key, "printStats", "--print-stats", ["true", "false"], "true")}
      ${cardSelect(net.key, "logToFile", "--log-to-file", ["true", "false"], "true")}
      ${cardSelect(net.key, "approxGeoLookup", "--approximate-geo-lookup", ["not set", "true", "false"], "not set", "span2")}
      ${cardInput(net.key, "startupDelay", "Startup delay sec", "0")}
      ${cardCheck(net.key, "startOnLaunch", "Start on launch", false)}
      ${cardCheck(net.key, "autoRestart", "Auto-reconnect", true)}
    </div>`;
}

function renderPorts(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardInput(net.key, "stratumPort", "--stratum-port", net.stratumPort)}
      ${cardInput(net.key, "promPort", "--prom-port", net.promPort)}
    </div>`;
}

function renderCpuMiner(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardCheck(net.key, "internalCpuMiner", "--internal-cpu-miner", false)}
      ${cardInput(net.key, "internalCpuMinerAddress", "--internal-cpu-miner-address", "", "kaspa:...", "span2")}
      ${cardInput(net.key, "internalCpuMinerThreads", "--internal-cpu-miner-threads", "", "threads")}
      ${cardInput(net.key, "internalCpuMinerThrottleMs", "--internal-cpu-miner-throttle-ms", "", "optional")}
      ${cardInput(net.key, "internalCpuMinerTemplatePollMs", "--internal-cpu-miner-template-poll-ms", "", "optional", "span2")}
    </div>`;
}

function renderInstancePanel(net, instanceId) {
  return `
    <section class="bridge-v7-instance-panel" data-net="${net.key}" data-instance-panel="${instanceId}"${activeInstance[net.key] === instanceId ? "" : " hidden"}>
      <div class="bridge-v7-grid">
        <div class="bridge-v7-card span3">
          <span>--instance</span>
          <textarea id="${iid(net.key, instanceId, "instance")}" class="bridge-v7-instance-text" placeholder="port=:5555,diff=2048,prom=:2114"></textarea>
        </div>
        ${instanceSelect(net.key, instanceId, "instanceLogToFile", "instance log", ["not set", "true", "false"], "not set")}
        ${instanceSelect(net.key, instanceId, "instanceVarDiff", "instance var_diff", ["not set", "true", "false"], "not set")}
        ${instanceSelect(net.key, instanceId, "instanceVarDiffStats", "instance var_diff_stats", ["not set", "true", "false"], "not set")}
        ${instanceInput(net.key, instanceId, "instanceSharesPerMin", "instance shares_per_min", "", "optional")}
        ${instanceSelect(net.key, instanceId, "instancePow2Clamp", "instance pow2_clamp", ["not set", "true", "false"], "not set")}
        <div class="bridge-v7-card buttons">
          <button type="button" data-bridge-action="duplicate-instance" data-net="${net.key}" data-instance="${instanceId}">Duplicate</button>
          <button type="button" class="danger" data-bridge-action="remove-instance" data-net="${net.key}" data-instance="${instanceId}">Remove</button>
        </div>
      </div>
    </section>`;
}


function bridgeNormalizeInstance(raw) {
  const value = String(raw || "").trim();

  if (!value) return "";

  if (/^\d+$/.test(value)) {
    return `port=:${value}`;
  }

  if (/^:\d{2,5}$/.test(value)) {
    return `port=${value}`;
  }

  return value;
}

/* KGW_BRIDGE_INSTANCE_PHASE1_UPSTREAM_SERIALIZER_R1C
 * Upstream-compatible bridge instance serializer.
 * RKStratum expects one --instance value with comma-separated internal keys:
 * port/prom/diff/log/var_diff/shares_per_min/var_diff_stats/pow2_clamp.
 */
function bridgeInstanceSplitParts(value) {
  return String(value || "")
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean);
}

function bridgeInstanceKeyOf(part) {
  const eq = String(part || "").indexOf("=");
  return eq > 0 ? String(part).slice(0, eq).trim() : "";
}

function bridgeInstanceCanonicalKey(key) {
  const normalized = String(key || "").trim();

  if (normalized === "stratum" || normalized === "stratum_port") return "port";
  if (normalized === "prom_port") return "prom";
  if (normalized === "min_share_diff") return "diff";
  if (normalized === "log_to_file") return "log";

  return normalized;
}

function bridgeInstanceWithoutKeys(parts, keys) {
  const blocked = new Set(keys.map(bridgeInstanceCanonicalKey));
  return parts.filter((part) => {
    const key = bridgeInstanceKeyOf(part);
    if (!key) return true;
    return !blocked.has(bridgeInstanceCanonicalKey(key));
  });
}

function bridgeInstanceAppend(parts, key, value) {
  const clean = String(value || "").trim();
  if (!clean) return parts;

  const canonical = bridgeInstanceCanonicalKey(key);
  const filtered = bridgeInstanceWithoutKeys(parts, [canonical]);
  filtered.push(`${canonical}=${clean}`);
  return filtered;
}

function bridgeInstanceReadSupplement(net, instanceId, fieldName, fallbackValue) {
  const value = bridgeReadInstanceField(net, instanceId, fieldName);
  if (value !== "") return value;
  return String(fallbackValue || "").trim();
}

function bridgeBuildUpstreamInstanceArg(net, instance) {
  const instanceId = instance?.id;
  let parts = [];

  parts = bridgeInstanceAppend(parts, "port", bridgeInstancePortValue(bridgeInstanceReadSupplement(net, instanceId, "instancePort", instance?.instancePort)));
  parts = bridgeInstanceAppend(parts, "diff", bridgeInstancePlainValue(bridgeInstanceReadSupplement(net, instanceId, "instanceDiff", instance?.instanceDiff)));
  parts = bridgeInstanceAppend(parts, "prom", bridgeInstancePortValue(bridgeInstanceReadSupplement(net, instanceId, "instanceProm", instance?.instanceProm)));

  parts = bridgeInstanceAppend(parts, "log", bridgeInstanceReadSupplement(net, instanceId, "instanceLogToFile", instance?.instanceLogToFile));
  parts = bridgeInstanceAppend(parts, "var_diff", bridgeInstanceReadSupplement(net, instanceId, "instanceVarDiff", instance?.instanceVarDiff));
  parts = bridgeInstanceAppend(parts, "shares_per_min", bridgeInstanceReadSupplement(net, instanceId, "instanceSharesPerMin", instance?.instanceSharesPerMin));
  parts = bridgeInstanceAppend(parts, "var_diff_stats", bridgeInstanceReadSupplement(net, instanceId, "instanceVarDiffStats", instance?.instanceVarDiffStats));
  parts = bridgeInstanceAppend(parts, "pow2_clamp", bridgeInstanceReadSupplement(net, instanceId, "instancePow2Clamp", instance?.instancePow2Clamp));

  return parts.join(",");
}

/* KGW_BRIDGE_INSTANCES_FIELDS_TRASH_R6
 * Existing Bridge Instances owner refinement:
 * - Port, diff, and prom are first-class fields in the existing Instances tab.
 * - Trash icon is next to the instance name.
 * - Existing serializer still emits one upstream-compatible --instance value.
 */
function bridgeInstanceParseStructured(value) {
  const parsed = {};
  const parts = String(value || "").split(",").map((part) => part.trim()).filter(Boolean);

  for (const part of parts) {
    const eq = part.indexOf("=");
    if (eq <= 0) continue;

    const key = bridgeInstanceCanonicalKey(part.slice(0, eq).trim());
    const rawValue = part.slice(eq + 1).trim();

    if (key === "port") parsed.instancePort = rawValue.replace(/^:/, "");
    if (key === "diff") parsed.instanceDiff = rawValue;
    if (key === "prom") parsed.instanceProm = rawValue.replace(/^:/, "");
  }

  return parsed;
}

function bridgeInstancePortValue(value) {
  const clean = String(value || "").trim().replace(/^:/, "");
  return clean ? ":" + clean : "";
}

function bridgeInstancePlainValue(value) {
  return String(value || "").trim();
}

/* KGW_BRIDGE_INSTANCES_UI_PORT_VALIDATOR_R5
 * Existing Bridge owner enhancement:
 * - Instance optional booleans default to false.
 * - Instance tab has remove control next to the name.
 * - Strict port validation blocks duplicate ports before runtime.
 */
function bridgeDefaultInstanceRecord(idValue) {
  return {
    id: idValue || (Date.now() + Math.floor(Math.random() * 1000)),
    instance: "",
    instancePort: null,
    instanceDiff: "2048",
    instanceProm: null,
    instanceLogToFile: "false",
    instanceVarDiff: "false",
    instanceSharesPerMin: "",
    instanceVarDiffStats: "false",
    instancePow2Clamp: "false"
  };
}

function bridgeNormalizeInstanceRecord(raw, fallbackId) {
  const source = raw && typeof raw === "object" ? raw : {};
  const defaults = bridgeDefaultInstanceRecord(source.id || fallbackId);
  const parsed = bridgeInstanceParseStructured(source.instance || "");

  return {
    ...defaults,
    ...source,
    id: source.id || defaults.id,
    instance: "",
    instancePort: String(source.instancePort || parsed.instancePort || ""),
    instanceDiff: String(source.instanceDiff || parsed.instanceDiff || defaults.instanceDiff || "2048"),
    instanceProm: String(source.instanceProm || parsed.instanceProm || ""),
    instanceLogToFile: String(source.instanceLogToFile || "false"),
    instanceVarDiff: String(source.instanceVarDiff || "false"),
    instanceSharesPerMin: String(source.instanceSharesPerMin || ""),
    instanceVarDiffStats: String(source.instanceVarDiffStats || "false"),
    instancePow2Clamp: String(source.instancePow2Clamp || "false")
  };
}

function bridgeExtractPortsFromTextR5(value) {
  const text = String(value || "");
  const ports = [];

  const patterns = [
    /(?:^|[,\s=])(?:port|stratum|stratum_port|prom|prom_port|rpc|rpclisten|listen|dashboard|web_dashboard_port)=?(?:127\.0\.0\.1:|0\.0\.0\.0:|localhost:|:)?(\d{2,5})(?=$|[,\s])/g,
    /(?:127\.0\.0\.1|0\.0\.0\.0|localhost):(\d{2,5})/g,
    /(^|[^\d]):(\d{2,5})(?=$|[,\s])/g
  ];

  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(text))) {
      const raw = match[2] || match[1];
      const port = Number(raw);
      if (Number.isInteger(port) && port > 0 && port <= 65535) ports.push(String(port));
    }
  }

  return ports;
}

function bridgePushPortR5(items, port, role, owner, net) {
  const normalized = String(port || "").trim().replace(/^:/, "");
  if (!/^\d{1,5}$/.test(normalized)) return;

  const numeric = Number(normalized);
  if (!Number.isInteger(numeric) || numeric < 1 || numeric > 65535) return;

  items.push({
    port: String(numeric),
    role: String(role || "unknown"),
    owner: String(owner || ""),
    net: String(net || "")
  });
}

function bridgeCollectConfiguredPortsR5() {
  const items = [];

  for (const profile of BRIDGE_NETWORKS) {
    const net = profile.key;

    bridgePushPortR5(items, profile.kaspadPort, "default-kaspad-rpc", "BRIDGE_NETWORKS.kaspadPort", net);
    bridgePushPortR5(items, profile.stratumPort, "default-stratum", "BRIDGE_NETWORKS.stratumPort", net);
    bridgePushPortR5(items, profile.promPort, "default-prometheus", "BRIDGE_NETWORKS.promPort", net);

    const fields = [
      ["stratumPort", "bridge-stratum"],
      ["promPort", "bridge-prometheus"],
      ["webDashboardPort", "bridge-dashboard"],
      ["healthCheckPort", "bridge-health"],
      ["kaspadAddress", "bridge-external-kaspad"],
      ["inprocessRpcListen", "inprocess-rpc"],
      ["inprocessRpcListenBorsh", "inprocess-rpc-borsh"],
      ["inprocessRpcListenJson", "inprocess-rpc-json"],
      ["inprocessListen", "inprocess-p2p"]
    ];

    for (const [field, role] of fields) {
      const value = v(net, field);
      for (const port of bridgeExtractPortsFromTextR5(value)) {
        bridgePushPortR5(items, port, role, field, net);
      }
    }

    bridgeEnsureInstanceState(net);

    for (const instance of bridgeInstances[net]) {
      const instanceText = bridgeBuildUpstreamInstanceArg(net, instance);

      for (const port of bridgeExtractPortsFromTextR5(instanceText)) {
        bridgePushPortR5(items, port, "instance", "instance:" + String(instance.id), net);
      }
    }
  }

  return items;
}

function bridgeValidatePortConflictsR5(activeNet) {
  const items = bridgeCollectConfiguredPortsR5();
  const byPort = new Map();

  for (const item of items) {
    if (!byPort.has(item.port)) byPort.set(item.port, []);
    byPort.get(item.port).push(item);
  }

  const conflicts = [];

  for (const [port, owners] of byPort.entries()) {
    const uniqueOwners = new Set(owners.map((item) => item.net + ":" + item.role + ":" + item.owner));

    if (uniqueOwners.size <= 1) continue;

    const touchesActiveNet = owners.some((item) => item.net === activeNet);
    const touchesInstance = owners.some((item) => item.role === "instance");

    if (touchesActiveNet || touchesInstance) {
      conflicts.push({
        port,
        owners
      });
    }
  }

  return {
    ok: conflicts.length === 0,
    conflicts,
    message: conflicts.map((item) => {
      const owners = item.owners.map((owner) => owner.net + "/" + owner.role + "/" + owner.owner).join(" | ");
      return "port " + item.port + " => " + owners;
    }).join("; ")
  };
}

/* KGW_BRIDGE_INSTANCES_PLUS_AUTOPORT_DETAILS_R8B
 * Existing Bridge Instances owner refinement:
 * - + action is routed through installActions.
 * - New instance gets nearest unused stratum port and prom port.
 * - Each instance panel shows a read-only upstream --instance preview row.
 */
function bridgeUsedPortSetR8B() {
  return new Set(bridgeCollectConfiguredPortsR5().map((item) => String(item.port || "").trim()).filter(Boolean));
}

function bridgeFindNearestUnusedPortR8B(startPort, usedPorts) {
  let port = Number(String(startPort || "").replace(/^:/, ""));

  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    port = 1;
  }

  while (port <= 65535 && usedPorts.has(String(port))) {
    port += 1;
  }

  if (port > 65535) {
    throw new Error("No available TCP port was found for bridge instance allocation.");
  }

  usedPorts.add(String(port));
  return String(port);
}

function bridgeAllocateInstancePortsR8B(net) {
  const profile = bridgeProfile(net) || {};
  const usedPorts = bridgeUsedPortSetR8B();

  const baseStratum = v(net, "stratumPort") || profile.stratumPort || 5555;
  const instancePort = bridgeFindNearestUnusedPortR8B(baseStratum, usedPorts);

  const baseProm = v(net, "promPort") || profile.promPort || 2114;
  const instanceProm = bridgeFindNearestUnusedPortR8B(baseProm, usedPorts);

  return {
    instancePort,
    instanceProm
  };
}

/* KGW_BRIDGE_INSTANCES_NO_ADVANCED_NUMERIC_PORTS_R9
 * Existing Bridge Instances owner refinement:
 * - No advanced free-text field.
 * - Initial/default instances receive numeric port/prom values.
 * - User can edit numeric port/prom.
 * - Validator still blocks conflicts.
 */
function bridgeNormalizePortR9(value) {
  return String(value || "").trim().replace(/^:/, "");
}

function bridgePortIsValidR9(value) {
  const port = Number(bridgeNormalizePortR9(value));
  return Number.isInteger(port) && port >= 1 && port <= 65535;
}

function bridgeUsedPortSetR9(skipNet, skipInstanceId) {
  const used = new Set();

  function add(value) {
    const port = bridgeNormalizePortR9(value);
    if (!port) return;
    const numeric = Number(port);
    if (Number.isInteger(numeric) && numeric >= 1 && numeric <= 65535) {
      used.add(String(numeric));
    }
  }

  for (const profile of BRIDGE_NETWORKS) {
    add(profile.kaspadPort);
    add(profile.stratumPort);
    add(profile.promPort);

    const fields = [
      "stratumPort",
      "promPort",
      "webDashboardPort",
      "healthCheckPort",
      "kaspadAddress",
      "inprocessRpcListen",
      "inprocessRpcListenBorsh",
      "inprocessRpcListenJson",
      "inprocessListen"
    ];

    for (const field of fields) {
      const value = v(profile.key, field);
      for (const port of bridgeExtractPortsFromTextR5(value)) add(port);
    }
  }

  for (const [net, list] of Object.entries(bridgeInstances)) {
    if (!Array.isArray(list)) continue;

    for (const item of list) {
      if (String(net) === String(skipNet) && String(item.id) === String(skipInstanceId)) continue;

      add(item.instancePort);
      add(item.instanceProm);

      for (const port of bridgeExtractPortsFromTextR5(item.instance || "")) {
        add(port);
      }
    }
  }

  return used;
}

function bridgeFindNearestUnusedPortR9(startPort, usedPorts) {
  let port = Number(bridgeNormalizePortR9(startPort));

  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    port = 1;
  }

  while (port <= 65535 && usedPorts.has(String(port))) {
    port += 1;
  }

  if (port > 65535) {
    throw new Error("No available TCP port was found for bridge instance allocation.");
  }

  usedPorts.add(String(port));
  return String(port);
}

function bridgeAssignMissingInstancePortsR9(net, instance) {
  const profile = bridgeProfile(net) || {};
  const used = bridgeUsedPortSetR9(net, instance.id);

  const currentPort = bridgeNormalizePortR9(instance.instancePort);
  const currentProm = bridgeNormalizePortR9(instance.instanceProm);

  if (bridgePortIsValidR9(currentPort)) used.add(currentPort);
  if (bridgePortIsValidR9(currentProm)) used.add(currentProm);

  if (!bridgePortIsValidR9(instance.instancePort)) {
    instance.instancePort = bridgeFindNearestUnusedPortR9(v(net, "stratumPort") || profile.stratumPort || 5555, used);
  } else {
    instance.instancePort = currentPort;
  }

  if (!bridgePortIsValidR9(instance.instanceProm)) {
    instance.instanceProm = bridgeFindNearestUnusedPortR9(v(net, "promPort") || profile.promPort || 2112, used);
  } else {
    instance.instanceProm = currentProm;
  }

  if (!String(instance.instanceDiff || "").trim()) {
    instance.instanceDiff = v(net, "minShareDiff") || "2048";
  }

  return instance;
}

function bridgeCreateInstanceRecordR9(net) {
  const record = bridgeDefaultInstanceRecord(Date.now() + Math.floor(Math.random() * 1000));
  return bridgeAssignMissingInstancePortsR9(net, record);
}

function bridgeInstancePreviewTextR8B(net, instance) {
  const value = bridgeBuildUpstreamInstanceArg(net, instance);
  return value ? "--instance=" + value : "--instance=";
}

function bridgeSyncInstancePreviewRowsR8B(net) {
  const root = document.getElementById("kaspa-bridge");
  if (!root) return;

  bridgeEnsureInstanceState(net);

  for (const preview of root.querySelectorAll('[data-bridge-instance-preview][data-network="' + net + '"]')) {
    const instanceId = preview.dataset.instanceId;
    const instance = bridgeInstances[net].find((item) => String(item.id) === String(instanceId));
    const text = instance ? bridgeInstancePreviewTextR8B(net, instance) : "--instance=";

    preview.value = text;
    preview.textContent = text;
    preview.title = text;
  }
}

function bridgeAssertNoPortConflictsR5(net) {
  const validation = bridgeValidatePortConflictsR5(net);

  if (!validation.ok) {
    throw new Error("Bridge port conflict: " + validation.message);
  }

  return validation;
}

function bridgeReadInstanceField(net, instanceId, fieldName) {
  const el = byId(id(net, `${fieldName}-${instanceId}`));
  if (!el) return "";

  if (el.type === "checkbox") return el.checked ? "true" : "";
  return String(el.value || "").trim();
}

function bridgeInstanceBoolArg(lines, net, instanceId, fieldName, flag) {
  const value = bridgeReadInstanceField(net, instanceId, fieldName);
  if (value === "true" || value === "false") {
    lines.push(`${flag}=${value}`);
  }
}

function bridgeInstanceValueArg(lines, net, instanceId, fieldName, flag) {
  const value = bridgeReadInstanceField(net, instanceId, fieldName);
  if (value) {
    lines.push(`${flag}=${value}`);
  }
}

function bridgeCollectCommandPorts(lines) {
  const ports = [];

  for (const part of lines) {
    const text = String(part || "");
    const values = [];

    const eq = text.match(/=(0\.0\.0\.0:|127\.0\.0\.1:|localhost:|:)?(\d{2,5})(\b|,)/);
    if (eq) values.push(eq[2]);

    const instancePorts = [...text.matchAll(/(?:port|prom_port)=(:|0\.0\.0\.0:|127\.0\.0\.1:|localhost:)?(\d{2,5})/g)];
    for (const match of instancePorts) values.push(match[2]);

    for (const value of values) {
      ports.push(value);
    }
  }

  return ports;
}

function bridgeDuplicatePorts(lines) {
  const ports = bridgeCollectCommandPorts(lines);
  const seen = new Set();
  const dup = new Set();

  for (const port of ports) {
    if (seen.has(port)) dup.add(port);
    seen.add(port);
  }

  return [...dup];
}

function bridgeEnsureInstanceState(net) {
  if (!Array.isArray(bridgeInstances[net])) {
    bridgeInstances[net] = [];
  }

  if (bridgeInstances[net].length === 0) {
    bridgeInstances[net].push(bridgeDefaultInstanceRecord(Date.now()));
  }

  bridgeInstances[net] = bridgeInstances[net].map((instance, index) => {
    const fallbackId = instance && instance.id ? instance.id : Date.now() + index;
    const normalized = bridgeNormalizeInstanceRecord(instance, fallbackId);
    return bridgeAssignMissingInstancePortsR9(net, normalized);
  });

  if (!activeInstance[net] && bridgeInstances[net][0]) {
    activeInstance[net] = bridgeInstances[net][0].id;
  }
}


function renderInstances(net) {
  net = bridgeInstanceNetworkKeyR15(net, net);
  bridgeEnsureInstanceState(net);

  return `
    <div class="bridge-v7-instance-tabs bridge-v7-instance-tabs-r7b">
      ${bridgeInstances[net].map((instance, index) => `
        <button
          type="button"
          class="bridge-v7-instance-pill-r7b bridge-v7-instance-pill-r11 ${String(activeInstance[net]) === String(instance.id) || (!activeInstance[net] && index === 0) ? "active" : ""}"
          data-bridge-action="select-instance"
          data-network="${net}"
          data-instance-id="${instance.id}">
          <span class="bridge-v7-instance-title-r7b">Instance ${index + 1}</span>
          <span
            role="button"
            tabindex="-1"
            class="bridge-v7-instance-trash-inline-r7b bridge-v7-instance-trash-r9 bridge-v7-instance-trash-r11"
            data-bridge-action="remove-instance"
            data-network="${net}"
            data-instance-id="${instance.id}"
            title="Delete Instance ${index + 1}"
            aria-label="Delete Instance ${index + 1}"
            data-disabled="${bridgeInstances[net].length <= 1 ? "true" : "false"}">🗑</span>
        </button>`).join("")}
      <button
        type="button"
        class="bridge-v7-instance-add bridge-v7-instance-add-r7b bridge-v7-instance-add-r11"
        data-bridge-action="add-instance"
        data-network="${net}"
        aria-label="Add Instance"
        title="Add Instance">+</button>
    </div>

    <div class="bridge-v7-instance-stack bridge-v7-instance-stack-r7b">
      ${bridgeInstances[net].map((instance, index) => `
        <section
          class="bridge-v7-instance-panel bridge-v7-instance-panel-r7b ${String(activeInstance[net]) === String(instance.id) || (!activeInstance[net] && index === 0) ? "active" : ""}"
          data-bridge-instance-panel="${instance.id}">
          <label class="bridge-v7-card bridge-v7-instance-preview-card-r8b">
            <span>--instance preview</span>
            <input
              readonly
              data-bridge-instance-preview="true"
              data-network="${net}"
              data-instance-id="${instance.id}"
              value="${bridgeInstancePreviewTextR8B(net, instance)}"
              title="${bridgeInstancePreviewTextR8B(net, instance)}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>port</span>
            <input id="${id(net, `instancePort-${instance.id}`)}" data-bridge-instance-field="instancePort" value="${instance.instancePort || ""}" placeholder="5558" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>diff</span>
            <input id="${id(net, `instanceDiff-${instance.id}`)}" data-bridge-instance-field="instanceDiff" value="${instance.instanceDiff || "2048"}" placeholder="2048" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>prom</span>
            <input id="${id(net, `instanceProm-${instance.id}`)}" data-bridge-instance-field="instanceProm" value="${instance.instanceProm || ""}" placeholder="2115" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>log</span>
            <select id="${id(net, `instanceLogToFile-${instance.id}`)}" data-bridge-instance-field="instanceLogToFile">
              <option value="false" ${instance.instanceLogToFile !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceLogToFile === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>var_diff</span>
            <select id="${id(net, `instanceVarDiff-${instance.id}`)}" data-bridge-instance-field="instanceVarDiff">
              <option value="false" ${instance.instanceVarDiff !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiff === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>var_stats</span>
            <select id="${id(net, `instanceVarDiffStats-${instance.id}`)}" data-bridge-instance-field="instanceVarDiffStats">
              <option value="false" ${instance.instanceVarDiffStats !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiffStats === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>shares/min</span>
            <input id="${id(net, `instanceSharesPerMin-${instance.id}`)}" data-bridge-instance-field="instanceSharesPerMin" value="${instance.instanceSharesPerMin || ""}" placeholder="optional" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span>pow2</span>
            <select id="${id(net, `instancePow2Clamp-${instance.id}`)}" data-bridge-instance-field="instancePow2Clamp">
              <option value="false" ${instance.instancePow2Clamp !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instancePow2Clamp === "true" ? "selected" : ""}>true</option>
            </select>
          </label>
        </section>
      `).join("")}
    </div>`;
}


// KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D
function renderInprocessNodeSettings(net) {
  const tabs = [
    ["basic", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.basic", "Basic")],
    ["rpc", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.rpc", "RPC")],
    ["storage", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.storage", "Storage / Index")],
    ["p2p", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.p2p", "P2P / Network")],
    ["perf", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.performance", "Performance / Logs")],
    ["advanced", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.advanced", "Advanced")],
    ["danger", kgwI18nTextR41("bridge.inprocessNodeSettings.tab.dangerous", "Dangerous")]
  ];

  const tabButtons = tabs.map(([key, label], index) =>
    `<button type="button" class="bridge-v12d-node-tab${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-inprocess-node-tab="${key}">${esc(label)}</button>`
  ).join("");

  const testnetArgs = net.testnet
    ? `--testnet${net.netsuffix ? " --netsuffix=" + esc(net.netsuffix) : ""}`
    : "mainnet";

  return `
    <div class="bridge-v12d-inprocess-node-settings bridge-v12d-inprocess-inactive" data-net="${net.key}" data-bridge-inprocess-node-settings="${net.key}" data-kgw-owner="KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D">
      <div class="bridge-v12d-node-tabs">${tabButtons}</div>

      <section class="bridge-v12d-node-panel active" data-net="${net.key}" data-bridge-inprocess-node-panel="basic">
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card span2">
            <span data-i18n="bridge.inprocessNodeSettings.appdir">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.appdir", "same --appdir / database path"))}</span>
            <input id="${id(net.key, "inprocessAppdirMirror")}" type="text" value="" readonly>
          </div>
          <div class="bridge-v7-card span2">
            <span data-i18n="bridge.inprocessNodeSettings.testnet">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.testnet", "kaspad network args"))}</span>
            <input id="${id(net.key, "inprocessNetworkArgs")}" type="text" value="${esc(testnetArgs)}" readonly>
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="rpc" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.rpcListen">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.rpcListen", "--rpclisten"))}</span>
            <input id="${id(net.key, "inprocessRpcListen")}" type="text" value="127.0.0.1:${esc(net.kaspadPort)}">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.rpcListenBorsh">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.rpcListenBorsh", "--rpclisten-borsh"))}</span>
            <input id="${id(net.key, "inprocessRpcListenBorsh")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.rpcListenJson">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.rpcListenJson", "--rpclisten-json"))}</span>
            <input id="${id(net.key, "inprocessRpcListenJson")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check danger">
            <input id="${id(net.key, "inprocessUnsafeRpc")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.unsafeRpc">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.unsafeRpc", "--unsaferpc"))}</span>
          </label>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="storage" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessUtxoIndex")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.utxoIndex">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.utxoIndex", "--utxoindex"))}</span>
          </label>
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessArchival")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.archival">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.archival", "--archival"))}</span>
          </label>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="p2p" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.listen">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.listen", "--listen"))}</span>
            <input id="${id(net.key, "inprocessListen")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.addPeer">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.addPeer", "--addpeer"))}</span>
            <input id="${id(net.key, "inprocessAddPeer")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.connect">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.connect", "--connect"))}</span>
            <input id="${id(net.key, "inprocessConnect")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessDisableUpnp")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.disableUpnp">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.disableUpnp", "--disable-upnp"))}</span>
          </label>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.maxInpeers">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.maxInpeers", "--maxinpeers"))}</span>
            <input id="${id(net.key, "inprocessMaxInpeers")}" type="number" min="0" step="1" value="">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.outpeers">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.outpeers", "--outpeers"))}</span>
            <input id="${id(net.key, "inprocessOutpeers")}" type="number" min="0" step="1" value="">
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="perf" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessPerfMetrics")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.perfMetrics">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.perfMetrics", "--perf-metrics"))}</span>
          </label>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.perfMetricsIntervalSec">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.perfMetricsIntervalSec", "--perf-metrics-interval-sec"))}</span>
            <input id="${id(net.key, "inprocessPerfMetricsIntervalSec")}" type="number" min="1" step="1" value="">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.logLevel">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.logLevel", "--loglevel"))}</span>
            <input id="${id(net.key, "inprocessLogLevel")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.ramScale">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.ramScale", "--ram-scale"))}</span>
            <input id="${id(net.key, "inprocessRamScale")}" type="number" min="0.1" step="0.1" value="">
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="advanced" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.configfile">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.configfile", "--configfile"))}</span>
            <input id="${id(net.key, "inprocessConfigfile")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessYes")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.yes">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.yes", "--yes"))}</span>
          </label>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="danger" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
            <span data-i18n="bridge.inprocessNodeSettings.overrideParamsFile">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.overrideParamsFile", "--override-params-file"))}</span>
            <input id="${id(net.key, "inprocessOverrideParamsFile")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check danger">
            <input id="${id(net.key, "inprocessDevnet")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.devnet">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.devnet", "--devnet"))}</span>
          </label>
          <label class="bridge-v7-card check danger">
            <input id="${id(net.key, "inprocessSimnet")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.simnet">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.simnet", "--simnet"))}</span>
          </label>
          <label class="bridge-v7-card check danger">
            <input id="${id(net.key, "inprocessEnableUnsyncedMining")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.enableUnsyncedMining">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.enableUnsyncedMining", "--enable-unsynced-mining"))}</span>
          </label>
        </div>
      </section>
    </div>`;
}
function renderSections(net) {
  const sections = [
    ["runtime", "Runtime", renderRuntime(net)],
    ["inprocess-node", "In-Process Node", renderInprocessNodeSettings(net)],
    ["difficulty", "Difficulty", renderDifficulty(net)],
    ["logging", "Logging", renderLogging(net)],
    ["ports", "Ports / Paths", renderPorts(net)],
    ...(net.key === "mainnet" ? [] : [["cpu", "CPU Miner", renderCpuMiner(net)]]),
    ["instances", "Instances", renderInstances(net.key)]
  ];

  const tabs = sections.map(([key, label], index) =>
    `<button type="button" class="bridge-v7-section-tab${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-section-tab="${key}">${label}</button>`
  ).join("");

  const panels = sections.map(([key, , body], index) => {
    const panelId = key === "instances" ? ` id="${id(net.key, "instances")}"` : "";
    return `<section${panelId} class="bridge-v7-section${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-section-panel="${key}"${index === 0 ? "" : " hidden"}>${body}</section>`;
  }).join("");

  return `
    <div class="bridge-v7-section-tabs">${tabs}</div>
    <div class="bridge-v7-sections">${panels}</div>`;
}

function renderNetworkPanel(net, index) {
  return `
    <div class="bridge-v7-network-panel${index === 0 ? " active" : ""}" data-bridge-network-panel="${net.key}"${index === 0 ? "" : " hidden"}>
      <div class="bridge-v7-inner-tabs">
        <button type="button" class="bridge-v7-inner-tab active" data-net="${net.key}" data-bridge-inner-tab="settings">Settings</button>
        <button type="button" class="bridge-v7-inner-tab" data-net="${net.key}" data-bridge-inner-tab="log">Log</button>
      </div>

      <div class="bridge-v7-inner-panel active" data-net="${net.key}" data-bridge-inner-panel="settings">
        <section class="bridge-v7-command">
          <div class="bridge-v7-command-title">Command Preview</div>
          <textarea id="${id(net.key, "commandPreview")}" readonly spellcheck="false" wrap="soft"></textarea>
          <button type="button" class="bridge-v7-copy" data-bridge-action="copy-command" data-net="${net.key}" title="Copy command">⧉</button>
        </section>

        <section class="bridge-v7-toolbar">
          <div class="bridge-v7-buttons">
            <button type="button" class="good" data-bridge-action="start" data-net="${net.key}">Start</button>
            <button type="button" data-bridge-action="stop" data-net="${net.key}">Stop</button>
          </div>

          <div class="bridge-v7-status">
            <label><input id="${id(net.key, "launch")}" type="checkbox"> Launch</label>
            <label><input id="${id(net.key, "restart")}" type="checkbox" checked> Restart</label>
          </div>
        </section>

        ${renderSections(net)}

        <div class="settings-bottom-actions bridge-settings-bottom-actions">
        <button type="button" data-bridge-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-bridge-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-bridge-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        </div>

      </div>

      <div class="bridge-v7-inner-panel" data-net="${net.key}" data-bridge-inner-panel="log" hidden>
        <div class="bridge-v7-log-toolbar">
          <button type="button" data-bridge-action="copy-log" data-net="${net.key}">Copy Log</button>
          <button type="button" data-bridge-action="clear-log" data-net="${net.key}">Clear Log</button>
        </div>
        <pre id="${id(net.key, "logOutput")}" class="bridge-v7-log"></pre>
      </div>
</div>`;
}


function bridgeReadInstanceState(net, instanceId) {
  const current = bridgeInstances[net].find((instance) => String(instance.id) === String(instanceId)) || {};
  const next = bridgeNormalizeInstanceRecord(current, Date.now() + Math.floor(Math.random() * 1000));

  return bridgeAssignMissingInstancePortsR9(net, {
    id: next.id || instanceId || Date.now() + Math.floor(Math.random() * 1000),
    instance: "",
    instancePort: bridgeReadInstanceField(net, instanceId, "instancePort") || next.instancePort || "",
    instanceDiff: bridgeReadInstanceField(net, instanceId, "instanceDiff") || next.instanceDiff || "2048",
    instanceProm: bridgeReadInstanceField(net, instanceId, "instanceProm") || next.instanceProm || "",
    instanceLogToFile: bridgeReadInstanceField(net, instanceId, "instanceLogToFile") || next.instanceLogToFile || "false",
    instanceVarDiff: bridgeReadInstanceField(net, instanceId, "instanceVarDiff") || next.instanceVarDiff || "false",
    instanceSharesPerMin: bridgeReadInstanceField(net, instanceId, "instanceSharesPerMin") || next.instanceSharesPerMin || "",
    instanceVarDiffStats: bridgeReadInstanceField(net, instanceId, "instanceVarDiffStats") || next.instanceVarDiffStats || "false",
    instancePow2Clamp: bridgeReadInstanceField(net, instanceId, "instancePow2Clamp") || next.instancePow2Clamp || "false"
  });
}

function bridgeRefreshInstances(net) {
  net = bridgeInstanceNetworkKeyR15(net, net);

  const container =
    byId(id(net, "instances")) ||
    document.querySelector(`[data-bridge-network-panel="${net}"] [data-bridge-section-panel="instances"]`);

  if (container) {
    if (!container.id) {
      container.id = id(net, "instances");
    }

    container.innerHTML = renderInstances(net);
    bridgeInstallInstanceContainerOwnerR11(container, net);
  }

  updateCommand(net);
}

/* KGW_BRIDGE_INSTANCES_PATCHMARKER_RUNTIME_FIX_R13B: fixes undefined runtime owner marker assignment. */
/* KGW_BRIDGE_INSTANCES_REBUILD_CLICK_OWNER_R11
 * One rebuilt Bridge Instances click owner.
 * It lives only on the rendered Instances container.
 * It handles + / select / delete via closest('[data-bridge-action]').
 * No document/window/global listener.
 */
function bridgeInstallInstanceContainerOwnerR11(container, net) {
  net = bridgeInstanceNetworkKeyR15(net, net);
  if (!container || !net) return;

  container.dataset.kgwBridgeInstancesClickOwner = "KGW_BRIDGE_INSTANCES_REBUILD_CLICK_OWNER_R11";
  container.onclick = function bridgeInstancesContainerClickOwnerR11(event) {
    const control = event.target && event.target.closest
      ? event.target.closest("[data-bridge-action]")
      : null;

    if (!control || !container.contains(control)) return;

    const action = control.dataset.bridgeAction || "";
    const targetNet = bridgeInstanceNetworkKeyR15(control.dataset.network, net);

    kgwBridgeExplicitTraceR27D(targetNet || "unknown", "internal-navigation", "r45d-bridge-instance-control-click", {
      patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
      trusted: Boolean(event && event.isTrusted),
      action: String(action || ""),
      instanceId: String(control.dataset.instanceId || control.dataset.instance || ""),
      text: String(control.textContent || "").trim()
    });

    if (!["add-instance", "select-instance", "remove-instance"].includes(action)) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (action === "add-instance") {
      addInstance(targetNet);
      return;
    }

    if (action === "select-instance") {
      activeInstance[targetNet] = control.dataset.instanceId;
      bridgeRefreshInstances(targetNet);
      updateCommand(targetNet);
      return;
    }

    if (action === "remove-instance") {
      if (control.dataset.disabled === "true" || control.disabled) return;
      removeInstance(targetNet, control.dataset.instanceId);
      updateCommand(targetNet);
    }
  };
}

function bridgeInstallAllVisibleInstanceContainerOwnersR11(root) {
  const scope = root || document;
  if (!scope) return;

  for (const profile of BRIDGE_NETWORKS) {
    const container = byId(id(profile.key, "instances"));
    if (container) {
      bridgeInstallInstanceContainerOwnerR11(container, profile.key);
    }
  }
}

/* KGW_BRIDGE_INSTANCES_TEMP_TRACE_REMOVED
 * Temporary scoped runtime trace for Bridge Instances across mainnet/testnet10/testnet12.
 * No global click listener. No forbidden legacy phase names.
 */



/* KGW_BRIDGE_INSTANCES_NETWORK_KEY_FIX_R15
 * Canonical network-key resolver for Bridge Instances.
 * Normalizes Bridge Instances network keys for mainnet, testnet10, and testnet12.
 */
function bridgeInstanceNetworkKeyR15(value, fallback) {
  const known = new Set(BRIDGE_NETWORKS.map((item) => item.key));
  const candidates = [];

  if (typeof value === "string") candidates.push(value);
  if (value && typeof value === "object" && typeof value.key === "string") candidates.push(value.key);

  if (typeof fallback === "string") candidates.push(fallback);
  if (fallback && typeof fallback === "object" && typeof fallback.key === "string") candidates.push(fallback.key);

  for (const candidate of candidates) {
    const normalized = String(candidate || "").trim();
    if (known.has(normalized)) return normalized;
  }

  return "mainnet";
}

/* KGW_BRIDGE_INSTANCES_ADD_CLICK_BIND_R10
 * Scoped Bridge Instances button binder.
 * This is not a global listener. It binds only the rendered instance container
 * and replaces onclick handlers idempotently after each render.
 */



function addInstance(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "add-instance", "r44d-owner-begin", {});
  net = bridgeInstanceNetworkKeyR15(net, net);
  bridgeEnsureInstanceState(net);

  const next = bridgeCreateInstanceRecordR9(net);
  bridgeInstances[net].push(next);
  activeInstance[net] = next.id;

  bridgeRefreshInstances(net);
  updateCommand(net);
  kgwBridgeSmallOwnerTraceR44D(net, "add-instance", "r44d-owner-complete", {});
}

function duplicateInstance(net, instanceId) {
  bridgeEnsureInstanceState(net);
  bridgeInstances[net].push(bridgeReadInstanceState(net, instanceId));
  bridgeRefreshInstances(net);
}

function removeInstance(net, instanceId) {
  kgwBridgeSmallOwnerTraceR44D(net, "remove-instance", "r44d-owner-begin", { instanceId: String(instanceId || "") });
  bridgeEnsureInstanceState(net);
  if (bridgeInstances[net].length <= 1) return;
  bridgeInstances[net] = bridgeInstances[net].filter((instance) => String(instance.id) !== String(instanceId));
  bridgeRefreshInstances(net);
  kgwBridgeSmallOwnerTraceR44D(net, "remove-instance", "r44d-owner-complete", { instanceId: String(instanceId || "") });
}

function selectInstance(net, instanceId) {
  kgwBridgeSmallOwnerTraceR44D(net, "select-instance", "r44d-owner-begin", { instanceId: String(instanceId || "") });
  const root = byId(id(net, "instances"));
  if (!root) return;

  root.querySelectorAll("[data-bridge-instance-panel]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.bridgeInstancePanel === String(instanceId));
  });

  root.querySelectorAll("[data-bridge-action='select-instance']").forEach((button) => {
    button.classList.toggle("active", button.dataset.instanceId === String(instanceId));
  });
  kgwBridgeSmallOwnerTraceR44D(net, "select-instance", "r44d-owner-complete", { instanceId: String(instanceId || "") });
}

function renderAllNetworks(root) {
  const host = root.querySelector("#bridgeNetworkPanels");
  if (!host) return;
  host.innerHTML = BRIDGE_NETWORKS.map(renderNetworkPanel).join("");


  setTimeout(kgwInstallBridgeLogAutoScrollControlsR27, 0);
  setTimeout(window.kgwInstallBridgeLogScopedControlsV29, 0);
}

function bridgeProfile(net) {
  return BRIDGE_NETWORKS.find((item) => item.key === net);
}

function bridgeNodeMode(net) {
  const value = v(net, "nodeMode");
  return value === "inprocess" ? "inprocess" : "external";
}

function bridgeHasConfig(net) {
  return Boolean(v(net, "config"));
}

function bridgeControl(net, name) {
  return byId(id(net, name));
}

function bridgeControlCard(el) {
  return el ? el.closest(".bridge-v7-card") : null;
}

function bridgeSetDisabled(net, name, disabled, reason = "") {
  const el = bridgeControl(net, name);
  if (!el) return;

  el.disabled = Boolean(disabled);

  const card = bridgeControlCard(el);
  if (card) {
    card.classList.toggle("bridge-v7-mode-disabled", Boolean(disabled));
    card.title = disabled ? reason : "";
  }
}

function bridgeSyncInprocessNodeSettingsV12D(net) {
  const profile = bridgeProfile(net);
  if (!profile) return;

  const nodeMode = bridgeNodeMode(net);
  const active = nodeMode === "inprocess" && !bridgeHasConfig(net);
  const section = document.querySelector(`[data-bridge-inprocess-node-settings="${net}"]`);

  if (section) {
    section.classList.toggle("bridge-v12d-inprocess-inactive", !active);
    section.classList.toggle("bridge-v12d-inprocess-active", active);
    section.dataset.kgwInprocessNodeActive = active ? "true" : "false";
  }

  const appdirMirror = bridgeControl(net, "inprocessAppdirMirror");
  if (appdirMirror) {
    appdirMirror.value = v(net, "appdir") || kgwI18nTextR41("bridge.inprocessNodeSettings.sameAsAppdir", "same as --appdir");
    appdirMirror.readOnly = true;
  }

  const networkArgs = bridgeControl(net, "inprocessNetworkArgs");
  if (networkArgs) {
    networkArgs.value = profile.testnet
      ? `--testnet${profile.netsuffix ? " --netsuffix=" + profile.netsuffix : ""}`
      : "mainnet";
    networkArgs.readOnly = true;
  }

  const fields = [
    "inprocessAppdirMirror",
    "inprocessNetworkArgs",
    "inprocessRpcListen",
    "inprocessRpcListenBorsh",
    "inprocessRpcListenJson",
    "inprocessUnsafeRpc",
    "inprocessUtxoIndex",
    "inprocessArchival",
    "inprocessListen",
    "inprocessAddPeer",
    "inprocessConnect",
    "inprocessDisableUpnp",
    "inprocessMaxInpeers",
    "inprocessOutpeers",
    "inprocessPerfMetrics",
    "inprocessPerfMetricsIntervalSec",
    "inprocessLogLevel",
    "inprocessRamScale",
    "inprocessConfigfile",
    "inprocessYes",
    "inprocessOverrideParamsFile",
    "inprocessDevnet",
    "inprocessSimnet",
    "inprocessEnableUnsyncedMining"
  ];

  for (const name of fields) {
    const mainnetDanger =
      net === "mainnet" &&
      [
        "inprocessOverrideParamsFile",
        "inprocessDevnet",
        "inprocessSimnet",
        "inprocessEnableUnsyncedMining"
      ].includes(name);

    bridgeSetDisabled(
      net,
      name,
      !active || mainnetDanger,
      mainnetDanger
        ? "Dangerous development-only kaspad flag is disabled on mainnet."
        : kgwI18nTextR41("bridge.inprocessNodeSettings.externalInactive", "Used only when Bridge Node Mode is In-Process.")
    );
  }

  const readonlyFields = ["inprocessAppdirMirror", "inprocessNetworkArgs"];
  for (const name of readonlyFields) {
    const control = bridgeControl(net, name);
    if (control) control.readOnly = true;
  }
}


function bridgeSyncModeControls(net) {
  const profile = bridgeProfile(net);
  if (!profile) return;

  const configMode = bridgeHasConfig(net);
  const nodeMode = bridgeNodeMode(net);
  const internalMinerEnabled = c(net, "internalCpuMiner");

  const explicitBridgeFields = [
    "testnet",
    "nodeMode",
    "appdir",
    "kaspadAddress",
    "blockWaitTime",
    "printStats",
    "logToFile",
    "healthCheckPort",
    "webDashboardPort",
    "varDiff",
    "sharesPerMin",
    "varDiffStats",
    "extranonceSize",
    "pow2Clamp",
    "coinbaseTagSuffix",
    "approxGeoLookup",
    "stratumPort",
    "minShareDiff",
    "promPort",
    "internalCpuMiner",
    "internalCpuMinerAddress",
    "internalCpuMinerThreads",
    "internalCpuMinerThrottleMs",
    "internalCpuMinerTemplatePollMs"
  ];

  for (const name of explicitBridgeFields) {
    bridgeSetDisabled(net, name, configMode, "Config mode is active. Clear --config to edit explicit CLI flags.");
  }

  bridgeSyncInprocessNodeSettingsV12D(net);

  if (configMode) return;

  const testnetControl = bridgeControl(net, "testnet");
  if (testnetControl) {
    testnetControl.checked = Boolean(profile.testnet);
  }
  bridgeSetDisabled(net, "testnet", true, "Network identity is owned by the selected Mainnet/Testnet tab.");

  if (nodeMode === "external") {
    bridgeSetDisabled(net, "kaspadAddress", false, "");
  } else {
    bridgeSetDisabled(net, "kaspadAddress", true, "In-process mode owns kaspad args after the -- separator.");
  }

  for (const name of [
    "internalCpuMinerAddress",
    "internalCpuMinerThreads",
    "internalCpuMinerThrottleMs",
    "internalCpuMinerTemplatePollMs"
  ]) {
    bridgeSetDisabled(net, name, !internalMinerEnabled, "Enable --internal-cpu-miner first.");
  }
}

function bridgeSyncAllModeControls() {
  BRIDGE_NETWORKS.forEach((item) => bridgeSyncModeControls(item.key));
}

function addRawValue(lines, flag, value) {
  const normalized = String(value || "").trim();
  if (normalized) lines.push(`${flag}=${normalized}`);
}


function bridgeLogLineBelongsToBridge(_line) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  return true;
}


// KGW_BRIDGE_LOG_AUTOSCROLL_CONTROLS_R27_START
function kgwBridgeLogAutoScrollKeyR27(net) {
  return `kgw.bridge.log.autoscroll.${net}`;
}

function kgwBridgeLogAutoScrollEnabledR27(net) {
  try {
    return localStorage.getItem(kgwBridgeLogAutoScrollKeyR27(net)) !== "0";
  } catch (_) {
    return true;
  }
}

function kgwBridgeSetLogAutoScrollR27(net, enabled) {
  try {
    localStorage.setItem(kgwBridgeLogAutoScrollKeyR27(net), enabled ? "1" : "0");
  } catch (_) {}

  const out = byId(id(net, "logOutput"));
  if (enabled && out) out.scrollTop = out.scrollHeight;
}

function kgwInstallBridgeLogAutoScrollControlsR27() {
  if (typeof document === "undefined") return;
  if (!Array.isArray(BRIDGE_NETWORKS)) return;

  for (const profile of BRIDGE_NETWORKS) {
    const net = profile.key;
    const out = byId(id(net, "logOutput"));
    if (!out) continue;

    const controlId = id(net, "logAutoScrollR27");
    if (byId(controlId)) continue;

    const label = document.createElement("label");
    label.className = "kgw-log-autoscroll-toggle";
    label.setAttribute("data-kgw-log-autoscroll", "bridge");
    label.setAttribute("title", "Keep the log pinned to the newest raw line.");

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.id = controlId;
    checkbox.checked = kgwBridgeLogAutoScrollEnabledR27(net);
    checkbox.addEventListener("change", (event) => {
      kgwBridgeSmallOwnerTraceR44D(net, "log-autoscroll", "r51b3-bridge-log-autoscroll-change", {
        patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
        trusted: Boolean(event && event.isTrusted),
        controlId: String(controlId || ""),
        checked: Boolean(checkbox.checked)
      });
      kgwBridgeSetLogAutoScrollR27(net, checkbox.checked);
    });

    const span = document.createElement("span");
    span.textContent = kgwI18nTextR41("common.autoScroll", "Auto-scroll");

    label.appendChild(checkbox);
    label.appendChild(span);

    const panel = out.closest(".bridge-v7-inner-panel, [data-bridge-inner-panel], [data-inner-panel], [data-bridge-panel], [data-panel]") || out.parentElement;
    const toolbar =
      panel?.querySelector(".bridge-v7-log-toolbar, .bridge-log-toolbar, [data-bridge-log-toolbar]") ||
      out.parentElement?.querySelector(".bridge-v7-log-toolbar, .bridge-log-toolbar, [data-bridge-log-toolbar]");

    if (toolbar) {
      toolbar.appendChild(label);
    } else {
      out.parentElement?.insertBefore(label, out);
    }
  }
}
// KGW_BRIDGE_LOG_AUTOSCROLL_CONTROLS_R27_END

function appendLog(net, message) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  const out = byId(id(net, "logOutput"));
  if (!out) return;

  const rawText = String(message ?? "");
  if (rawText.length === 0) {
    return;
  }

  const previousText = String(out.textContent || "");
  const lines = previousText ? previousText.split("\n") : [];

  for (const rawLine of rawText.split(/\r?\n/)) {
    lines.push(rawLine.trimEnd());
  }

  while (lines.length > 3000) lines.shift();

  out.textContent = lines.join("\n");
  if (kgwBridgeLogAutoScrollEnabledR27(net)) out.scrollTop = out.scrollHeight;
}
// KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D_HELPER
function bridgeInprocessAddKaspadValueArgV12D(lines, flag, value) {
  const clean = String(value || "").trim();
  if (!clean) return;
  lines.push(`${flag}=${clean}`);
}


function buildCommandLines(net) {
  bridgeSyncModeControls(net);
  bridgeEnsureInstanceState(net);

  const profile = bridgeProfile(net);
  const lines = ["stratum-bridge"];
  const kaspadArgs = [];
  const nodeMode = bridgeNodeMode(net);
  const configValue = v(net, "config");

  if (configValue) {
    addRawValue(lines, "--config", configValue);
    addValue(lines, net, "nodeMode", "--node-mode");
    addValue(lines, net, "webDashboardPort", "--web-dashboard-port");
    return lines;
  }

  if (profile?.testnet) {
    lines.push("--testnet");
  }

  addValue(lines, net, "nodeMode", "--node-mode");
  addValue(lines, net, "appdir", "--appdir");

  if (nodeMode === "external") {
    addValue(lines, net, "kaspadAddress", "--kaspad-address");
  } else if (nodeMode === "inprocess") {
    if (profile?.testnet) {
      kaspadArgs.push("--testnet");
      if (profile.netsuffix) {
        kaspadArgs.push(`--netsuffix=${profile.netsuffix}`);
      }
    }

    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--rpclisten", v(net, "inprocessRpcListen") || `127.0.0.1:${profile.kaspadPort}`);
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--rpclisten-borsh", v(net, "inprocessRpcListenBorsh"));
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--rpclisten-json", v(net, "inprocessRpcListenJson"));

    if (c(net, "inprocessUnsafeRpc")) kaspadArgs.push("--unsaferpc");
    if (c(net, "inprocessUtxoIndex")) kaspadArgs.push("--utxoindex");
    if (c(net, "inprocessArchival")) kaspadArgs.push("--archival");

    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--listen", v(net, "inprocessListen"));
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--addpeer", v(net, "inprocessAddPeer"));
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--connect", v(net, "inprocessConnect"));

    if (c(net, "inprocessDisableUpnp")) kaspadArgs.push("--disable-upnp");

    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--maxinpeers", v(net, "inprocessMaxInpeers"));
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--outpeers", v(net, "inprocessOutpeers"));

    if (c(net, "inprocessPerfMetrics")) kaspadArgs.push("--perf-metrics");
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--perf-metrics-interval-sec", v(net, "inprocessPerfMetricsIntervalSec"));
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--loglevel", v(net, "inprocessLogLevel"));
    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--ram-scale", v(net, "inprocessRamScale"));

    bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--configfile", v(net, "inprocessConfigfile"));
    if (c(net, "inprocessYes")) kaspadArgs.push("--yes");

    if (net !== "mainnet") {
      bridgeInprocessAddKaspadValueArgV12D(kaspadArgs, "--override-params-file", v(net, "inprocessOverrideParamsFile"));
      if (c(net, "inprocessDevnet")) kaspadArgs.push("--devnet");
      if (c(net, "inprocessSimnet")) kaspadArgs.push("--simnet");
      if (c(net, "inprocessEnableUnsyncedMining")) kaspadArgs.push("--enable-unsynced-mining");
    }
  }

  addValue(lines, net, "blockWaitTime", "--block-wait-time");
  addValue(lines, net, "printStats", "--print-stats");
  addValue(lines, net, "logToFile", "--log-to-file");
  addValue(lines, net, "healthCheckPort", "--health-check-port");
  addValue(lines, net, "webDashboardPort", "--web-dashboard-port");
  addValue(lines, net, "varDiff", "--var-diff");
  addValue(lines, net, "sharesPerMin", "--shares-per-min");
  addValue(lines, net, "varDiffStats", "--var-diff-stats");
  addValue(lines, net, "extranonceSize", "--extranonce-size");
  addValue(lines, net, "pow2Clamp", "--pow2-clamp");
  addValue(lines, net, "coinbaseTagSuffix", "--coinbase-tag-suffix");
  addBoolValue(lines, net, "approxGeoLookup", "--approximate-geo-lookup");
  addValue(lines, net, "stratumPort", "--stratum-port");
  addValue(lines, net, "minShareDiff", "--min-share-diff");
  addValue(lines, net, "promPort", "--prom-port");

  for (const instance of bridgeInstances[net]) {
    const instanceDefinition = bridgeBuildUpstreamInstanceArg(net, instance);
    if (instanceDefinition) {
      lines.push(`--instance=${instanceDefinition}`);
    }
  }

  if (c(net, "internalCpuMiner") && net !== "mainnet") {
    addFlag(lines, net, "internalCpuMiner", "--internal-cpu-miner");
    addValue(lines, net, "internalCpuMinerAddress", "--internal-cpu-miner-address");
    addValue(lines, net, "internalCpuMinerThreads", "--internal-cpu-miner-threads");
    addValue(lines, net, "internalCpuMinerThrottleMs", "--internal-cpu-miner-throttle-ms");
    addValue(lines, net, "internalCpuMinerTemplatePollMs", "--internal-cpu-miner-template-poll-ms");
  }

  if (kaspadArgs.length) {
    lines.push("--", ...kaspadArgs);
  }

  return lines;
}


function kgwExtractBridgeOwnerFlags(result) {
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

async function kgwLoadBridgeOwnerCommandPreview(net, fallbackText) {
  const invoke = getTauriInvoke();
  if (!invoke) return fallbackText;

  try {
    const result = await invokeWithTimeout(
      invoke,
      KGW_BRIDGE_RUNTIME_FLAGS_OWNER_COMMAND,
      { network: net },
      KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS
    );

    const flags = kgwExtractBridgeOwnerFlags(result);
    return flags ? "stratum-bridge " + flags : fallbackText;
  } catch (_) {
    return fallbackText;
  }
}


function updateCommand(net) {
  const preview = byId(id(net, "commandPreview"));
  if (!preview) return "";

  try {
    bridgeSyncModeControls(net);
    bridgeSyncInstancePreviewRowsR8B(net);

    const lines = buildCommandLines(net);
    const duplicatePorts = bridgeDuplicatePorts(lines);
    const portValidation = bridgeValidatePortConflictsR5(net);
    const allDuplicatePorts = [...new Set([...duplicatePorts, ...portValidation.conflicts.map((item) => item.port)])];
    const first = lines.shift() || "stratum-bridge";
    const text = lines.length ? `${first} ${lines.join(" ")}` : first;

    preview.value = text;
    preview.textContent = text;
    preview.dataset.kgwBridgeCommandOwner = "readme-instance-command-owner";
    preview.dataset.kgwBridgeNetwork = net;

    if (allDuplicatePorts.length || !portValidation.ok) {
      preview.dataset.kgwBridgeCommandWarning = portValidation.message || `duplicate ports: ${allDuplicatePorts.join(",")}`;
      preview.classList.add("bridge-v7-command-warning");
    } else {
      delete preview.dataset.kgwBridgeCommandWarning;
      preview.classList.remove("bridge-v7-command-warning");
    }

    bridgeSyncInstancePreviewRowsR8B(net);
    return text;
  } catch (error) {
    const message = "stratum-bridge # command preview error: " + normalizeRuntimeError(error);
    preview.value = message;
    preview.textContent = message;
    preview.dataset.kgwBridgeCommandOwner = "readme-instance-command-owner-error";
    preview.dataset.kgwBridgeNetwork = net;
    preview.dataset.kgwBridgeCommandWarning = normalizeRuntimeError(error);
    preview.classList.add("bridge-v7-command-warning");
    return message;
  }
}


function updateAllCommands() {
  bridgeSyncAllModeControls();
  BRIDGE_NETWORKS.forEach((net) => updateCommand(net.key));
}



// KGW_BRIDGE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45E
function kgwBridgeExplicitTraceR27D(net, action, phase, details) {
  try {
    const safeNet = String(net || "unknown");
    const safeAction = String(action || "internal-navigation");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "bridge",
      net: safeNet,
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_BRIDGE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45E",
        owner: "bridge-module-visible-explicit-trace-helper",
        network: safeNet,
        action: safeAction,
        phase: safePhase,
        details: safeDetails
      })
    };
    const tauri = window.__TAURI__;
    const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
      ? tauri.core.invoke.bind(tauri.core)
      : tauri && typeof tauri.invoke === "function"
        ? tauri.invoke.bind(tauri)
        : window.__TAURI_INVOKE__;
    if (typeof invoke === "function") {
      invoke("kgw_frontend_button_trace_v1", args).catch(function () {});
    }
  } catch (_) {}
}
function installNetworkTabs(root) {
  // KGW_R63_DIRECT_BRIDGE_NETWORK_TAB_SWITCH_OWNER
  const networkTabSelector = "[data-bridge-network-tab]";
  const networkPanelSelector = "[data-bridge-network-panel]";

  function normalizeNetFromElement(element) {
    if (!element) return "";
    return element.dataset.net || element.dataset.bridgeNetworkTab || element.dataset.bridgeNetworkPanel || "";
  }

  function allNetworkTabs() {
    return Array.from(root.querySelectorAll(networkTabSelector));
  }

  function allNetworkPanels() {
    return Array.from(root.querySelectorAll(networkPanelSelector));
  }

  function selectBridgeNetwork(net, reason = "manual") {
    const normalized = String(net || "").trim();

    if (!normalized) return;

    kgwBridgeExplicitTraceR27D(normalized, "internal-navigation", "r45d-bridge-network-select", {
      patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
      reason: String(reason || ""),
      selected: normalized
    });

    const tabs = allNetworkTabs();
    const panels = allNetworkPanels();

    for (const tab of tabs) {
      const tabNet = normalizeNetFromElement(tab);
      const active = tabNet === normalized;

      tab.classList.toggle("active", active);
      tab.classList.toggle("is-active", active);
      tab.classList.toggle("selected", active);
      tab.setAttribute("aria-selected", active ? "true" : "false");
      tab.dataset.active = active ? "true" : "false";
    }

    for (const panel of panels) {
      const panelNet = normalizeNetFromElement(panel);
      const active = panelNet === normalized;

      panel.hidden = !active;
      panel.classList.toggle("active", active);
      panel.classList.toggle("is-active", active);
      panel.dataset.active = active ? "true" : "false";
      panel.style.display = active ? "" : "none";
    }

    if (typeof updateCommand === "function") {
      updateCommand(normalized);
    }

    if (typeof kgwBridgeR51RefreshOne === "function") {
      window.setTimeout(() => kgwBridgeR51RefreshOne(normalized, "network-tab-" + reason), 50);
      window.setTimeout(() => kgwBridgeR51RefreshOne(normalized, "network-tab-" + reason + "+700ms"), 700);
    }

    if (typeof appendLog === "function") {
    }
  }

  root.addEventListener("click", (event) => {
    const tab = event.target.closest(networkTabSelector);

    if (!tab || !root.contains(tab)) return;

    const net = normalizeNetFromElement(tab);

    if (!net) return;

    event.preventDefault();
    event.stopPropagation();

    kgwBridgeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-bridge-network-tab-click", {
      patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
      trusted: Boolean(event && event.isTrusted),
      selected: String(net || ""),
      text: String(tab.textContent || "").trim()
    });

    selectBridgeNetwork(net, "click");
  }, true);

  const existingActiveTab = allNetworkTabs().find((tab) => {
    return tab.classList.contains("active") ||
      tab.classList.contains("is-active") ||
      tab.getAttribute("aria-selected") === "true" ||
      tab.dataset.active === "true";
  });

  const defaultTab =
    existingActiveTab ||
    allNetworkTabs().find((tab) => normalizeNetFromElement(tab) === "mainnet") ||
    allNetworkTabs()[0];

  if (defaultTab) {
    selectBridgeNetwork(normalizeNetFromElement(defaultTab), "initial");
  }

  window.kgwBridgeSelectNetworkTabR63 = selectBridgeNetwork;
}

function installDelegatedTabs(root) {
  root.addEventListener("click", (event) => {
    const innerTab = event.target.closest("[data-bridge-inner-tab]");
    if (innerTab) {
      const net = innerTab.dataset.net;
      const selected = innerTab.dataset.bridgeInnerTab;
      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      kgwBridgeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-bridge-inner-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected || ""),
        text: String(innerTab.textContent || "").trim()
      });

      panel.querySelectorAll("[data-bridge-inner-tab]").forEach((item) => {
        item.classList.toggle("active", item === innerTab);
      });

      panel.querySelectorAll("[data-bridge-inner-panel]").forEach((item) => {
        const active = item.dataset.bridgeInnerPanel === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });

      return;
    }

    const sectionTab = event.target.closest("[data-bridge-section-tab]");
    if (sectionTab) {
      const net = sectionTab.dataset.net;
      const selected = sectionTab.dataset.bridgeSectionTab;
      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      kgwBridgeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-bridge-section-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected || ""),
        text: String(sectionTab.textContent || "").trim()
      });

      panel.querySelectorAll("[data-bridge-section-tab]").forEach((item) => {
        item.classList.toggle("active", item === sectionTab);
      });

      panel.querySelectorAll("[data-bridge-section-panel]").forEach((item) => {
        const active = item.dataset.bridgeSectionPanel === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });

      return;
    }

    const instanceTab = event.target.closest("[data-instance-tab]");
    if (instanceTab) {
      const net = instanceTab.dataset.net;
      const selected = Number(instanceTab.dataset.instanceTab);
      activeInstance[net] = selected;

      kgwBridgeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-bridge-instance-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected),
        text: String(instanceTab.textContent || "").trim()
      });

      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      panel.querySelectorAll("[data-instance-tab]").forEach((item) => {
        item.classList.toggle("active", Number(item.dataset.instanceTab) === selected);
      });

      panel.querySelectorAll("[data-instance-panel]").forEach((item) => {
        const active = Number(item.dataset.instancePanel) === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });
    }
  });
}

// KGW_BRIDGE_INTEGRATED_RUNTIME_LINKAGE_V1: readable Bridge runtime response + duplicate-click guard.
const KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS = 7000;
const KGW_BRIDGE_RUNTIME_FLAGS_OWNER_COMMAND = "rk_integrated_bridge_runtime_flags_v1";
const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();

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

function parseRuntimeKeyValueResponse(value) {
  const raw = stringifyRuntimeResult(value);
  const text = raw.trim();

  if (!text || !text.includes("=")) {
    return { raw: text, fields: {} };
  }

  const fields = {};
  for (const part of text.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;

    const key = part.slice(0, index).trim();
    const fieldValue = part.slice(index + 1).trim();
    if (key) fields[key] = fieldValue;
  }

  return { raw: text, fields };
}

function yesNo(value) {
  if (value === true || value === "true") return "yes";
  if (value === false || value === "false") return "no";
  return value == null || value === "" ? "unknown" : String(value);
}

function readableRuntimeSummary(title, result) {
  const parsed = parseRuntimeKeyValueResponse(result);
  const f = parsed.fields;

  if (!Object.keys(f).length) {
    return title + ": " + parsed.raw;
  }

  const blocked = f.start_blocked === "true" || f.start_allowed === "false";
  const lines = [];

  if (blocked) {
    lines.push(title + ": blocked");
  } else if (f.running === "true") {
    lines.push(title + ": running");
  } else if (f.running === "false") {
    lines.push(title + ": stopped");
  } else {
    lines.push(title + ": response");
  }

  if (f.block_reason) lines.push("Reason: " + f.block_reason);
  if (f.network) lines.push("Network: " + f.network);
  if (f.dynamic_preflight_passed) lines.push("Preflight: " + (f.dynamic_preflight_passed === "true" ? "passed" : "failed"));
  if (f.explicit_start_enabled) lines.push("Explicit start enabled: " + yesNo(f.explicit_start_enabled));
  if (f.compile_time_start_enabled) lines.push("Compile-time start enabled: " + yesNo(f.compile_time_start_enabled));
  if (f.runtime_starts_processes) lines.push("Runtime starts processes: " + yesNo(f.runtime_starts_processes));
  if (f.running) lines.push("Running: " + yesNo(f.running));
  if (f.healthy) lines.push("Healthy: " + yesNo(f.healthy));
  if (f.message) lines.push("Message: " + f.message);

  return lines.join("\n");
}

function appendReadableRuntimeResult(_net, _title, _result) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  // Runtime action summaries are UI/status data and must not be written into the raw bridge log pane.
}

function buildApplyPayload(net, command) {
  if (command === "kgw_kgw_apply_node_settings_v1") {
    bridgeAssertNoPortConflictsR5(net);

    const preview = updateCommand(net) || byId(id(net, "commandPreview"))?.value || "";
    const nodeMode = bridgeNodeMode(net) === "inprocess" ? "inprocess" : "external";

    return {
      network: net,
      runtimeRole: "bridge",
      nodeKind: nodeMode === "inprocess" ? "integrated-inproc" : "remote",
      bridgeKind: nodeMode === "inprocess" ? "official-inprocess-node" : "official-external-node",
      nodeCommandPreview: "",
      bridgeCommandPreview: preview,
    };
  }

  if (
    command === "kgw_kgw_disable_network_v1" ||
    command === "kgw_runtime_owner_status_v1" ||
    command === "kgw_kgw_runtime_logs_v1"
  ) {
    return { network: net, runtimeRole: "bridge" };
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

async function invokeBridgeIntegratedRuntime(command, net) {
  const invoke = getTauriInvoke();
  if (!invoke) {
    throw new Error("Tauri invoke is unavailable in this window.");
  }

  return await invokeWithTimeout(invoke, command, buildApplyPayload(net, command), KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS);
}


// KGW_BRIDGE_INPROCESS_SAME_DB_OWNER_V7
function kgwBridgeV7RuntimeRunningFromText(text) {
  const parsed = parseRuntimeKeyValueResponse(text);
  const fields = parsed.fields || {};
  return (
    fields.running === "true" ||
    fields.node_running === "true" ||
    fields.official_core_running === "true" ||
    fields.bridge_running === "true" ||
    fields.bridge_owner_active === "true" ||
    /running=true/i.test(String(text || ""))
  );
}

async function kgwBridgeV7BlockInprocessIfNodeOwnerRunning(net) {
  if (bridgeNodeMode(net) !== "inprocess") return false;

  const invoke = getTauriInvoke();
  if (!invoke) return false;

  const result = await invokeWithTimeout(
    invoke,
    "kgw_runtime_owner_status_v1",
    { network: net, runtimeRole: "node" },
    KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS
  );

  if (!kgwBridgeV7RuntimeRunningFromText(result)) return false;

  const message =
    "Cannot start bridge in in-process mode because the same-network node is already running. Stop the node first, or switch bridge node mode to External.";

  try {
    window.alert(message);
  } catch (_) {}

  return true;
}

async function runBridgeIntegratedAction(action, net) {

  const commandByAction = {
    start: "kgw_kgw_apply_node_settings_v1",
    stop: "kgw_kgw_disable_network_v1"
  };

  const command = commandByAction[action];
  if (!command) return false;

  if (action === "start" && await kgwBridgeV7BlockInprocessIfNodeOwnerRunning(net)) {
    return true;
  }

  const inFlightKey = net + ":" + action;

  if (KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(inFlightKey)) {
    return true;
  }

  KGW_BRIDGE_RUNTIME_IN_FLIGHT.add(inFlightKey);

  try {
    const preview = updateCommand(net) || byId(id(net, "commandPreview"))?.value || "";

    const result = await invokeBridgeIntegratedRuntime(command, net);
    const raw = stringifyRuntimeResult(result);
    const parsed = parseRuntimeKeyValueResponse(result);
    const fields = parsed.fields || {};

    if (action === "start") {
      const confirmedStarted =
        /parallel-owned-self-worker\s+started/i.test(raw) ||
        /parallel-owned-self-worker\s+already\s+running/i.test(raw) ||
        (/role=bridge/i.test(raw) && /started|running=true|already running/i.test(raw)) ||
        fields.running === "true" ||
        fields.bridge_running === "true" ||
        fields.bridge_owner_active === "true";

      const blocked =
        fields.start_blocked === "true" ||
        fields.start_allowed === "false" ||
        /blocked|not enabled|failed/i.test(raw);

      if (confirmedStarted && !blocked) {

      } else {

      }
    }

    if (action === "stop") {
      const confirmedStopped =
        /parallel-owned-self-worker\s+stopped/i.test(raw) ||
        fields.running === "false" ||
        fields.bridge_running === "false";

      if (confirmedStopped) {
      } else {
      }
    }
  } catch (error) {
  }
finally {
    KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);
  }

  return true;
}
/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */
const KGW_BRIDGE_R51_STORAGE_PREFIX = "kgw.bridge.direct.v51.";
const KGW_BRIDGE_R51_LAST_STATUS = {};
const KGW_BRIDGE_R51_LAST_LOGS = {};
const KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE = {};
let KGW_BRIDGE_R51_TIMER = null;

function kgwBridgeR51Keys() {
  return BRIDGE_NETWORKS.map((item) => item.key);
}

function kgwBridgeR51Panel(net) {
  return document.querySelector(`[data-bridge-network-panel="${net}"]`);
}

function kgwBridgeR51Fields(net) {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return [];

  return Array.from(panel.querySelectorAll("input, select, textarea")).filter((field) => {
    if (!field.id || !field.id.startsWith(`bridge-${net}-`)) return false;
    if (field.id.endsWith("-commandPreview")) return false;
    if (field.id.endsWith("-logOutput")) return false;
    if (field.readOnly) return false;
    return true;
  });
}


/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START */
function kgwBridgeSettingsActionIsR6(action) {
  return action === "save-settings" || action === "restore-defaults" || action === "set-defaults";
}

function kgwBridgeNetFromSettingsEventR6(event, root, fallbackNet = "") {
  const target = event?.target;
  const carrier = target?.closest?.("[data-net], [data-network], [data-bridge-settings-panel], [id*='mainnet' i], [id*='testnet10' i], [id*='testnet12' i]");

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

  try {
    return fallbackNet || kgwBridgeCurrentVisibleNetwork(root) || "";
  } catch {
    return fallbackNet || "";
  }
}


/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_END */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */
const kgwBridgeSettingsFeedbackLocksR11 = new Map();


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

function kgwBridgeR51ReadSettings(net) {
  const values = {};

  for (const field of kgwBridgeR51Fields(net)) {
    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }

  return values;
}

function kgwBridgeR51WriteSettings(net, values) {
  if (!values || typeof values !== "object") return;

  for (const field of kgwBridgeR51Fields(net)) {
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

function kgwBridgeR51Store(key, value) {
  localStorage.setItem(KGW_BRIDGE_R51_STORAGE_PREFIX + key, JSON.stringify(value));
}

function kgwBridgeR51Load(key) {
  try {
    return JSON.parse(localStorage.getItem(KGW_BRIDGE_R51_STORAGE_PREFIX + key) || "null");
  } catch {
    return null;
  }
}

function kgwBridgeR51CaptureFactoryDefaults() {
  for (const net of kgwBridgeR51Keys()) {
    if (!kgwBridgeR51Load("factory:" + net)) {
      kgwBridgeR51Store("factory:" + net, kgwBridgeR51ReadSettings(net));
    }
  }
}

function kgwBridgeR51LoadSavedSettings() {
  for (const net of kgwBridgeR51Keys()) {
    const saved = kgwBridgeR51Load("saved:" + net);
    if (saved) {
      kgwBridgeR51WriteSettings(net, saved);
    }
  }
}

/* KGW_BRIDGE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */


function kgwBridgeR51SaveSettings(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "save-settings", "r44d-owner-begin", {});
  kgwBridgeR51Store("saved:" + net, kgwBridgeR51ReadSettings(net));
  kgwBridgeSmallOwnerTraceR44D(net, "save-settings", "r44d-owner-complete", {});
}

function kgwBridgeR51SetAsDefaults(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "set-defaults", "r44d-owner-begin", {});
  kgwBridgeR51Store("default:" + net, kgwBridgeR51ReadSettings(net));
  kgwBridgeSmallOwnerTraceR44D(net, "set-defaults", "r44d-owner-complete", {});
}

function kgwBridgeR51RestoreDefaults(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "restore-defaults", "r44d-owner-begin", {});
  kgwBridgeSettingsWithProgrammaticWriteR9B(() => {
    const defaults = kgwBridgeR51Load("default:" + net) || kgwBridgeR51Load("factory:" + net);
    kgwBridgeR51WriteSettings(net, defaults);
    kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  });
  kgwBridgeSmallOwnerTraceR44D(net, "restore-defaults", "r44d-owner-complete", {});
}

function kgwBridgeR51IsRunning(text) {
  const value = String(text || "");
  return /running=true/.test(value) || /bridge_running=true/.test(value) || /bridge_owner_active=true/.test(value);
}

function kgwBridgeR51SetRuntimeButtons(net, running) {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return;

  const start = panel.querySelector(`[data-bridge-action="start"][data-net="${net}"]`);
  const stop = panel.querySelector(`[data-bridge-action="stop"][data-net="${net}"]`);

  if (start) {
    start.disabled = Boolean(running);
    start.style.opacity = running ? "0.45" : "";
    start.style.cursor = running ? "not-allowed" : "";
    start.title = running ? "Bridge is running. Stop it before starting again." : "Start bridge";
  }

  if (stop) {
    stop.disabled = !running;
    stop.style.opacity = running ? "" : "0.45";
    stop.style.cursor = running ? "" : "not-allowed";
    stop.title = running ? "Stop bridge" : "Bridge is not running";
  }
}

function kgwBridgeR51Delta(previous, current) {
  const before = String(previous || "");
  const after = String(current || "");

  if (!after || before === after) return "";
  if (after.startsWith(before)) return after.slice(before.length).trim();

  return after.trim();
}

function kgwBridgeR51MaybeActivityNotice(net, statusText) {
  const now = Date.now();
  const last = KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE[net] || 0;

  if (now - last < 15000) return;

  if (!kgwBridgeR51IsRunning(statusText)) return;

  KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE[net] = now;

}

async function kgwBridgeR51RefreshOne(net, reason = "live") {
  try {
    const status = stringifyRuntimeResult(await invokeBridgeIntegratedRuntime("kgw_runtime_owner_status_v1", net));
    kgwBridgeR51SetRuntimeButtons(net, kgwBridgeR51IsRunning(status));

    if (KGW_BRIDGE_R51_LAST_STATUS[net] !== status) {
      KGW_BRIDGE_R51_LAST_STATUS[net] = status;
      
    }

    kgwBridgeR51MaybeActivityNotice(net, status);
  } catch (error) {
    kgwBridgeR51SetRuntimeButtons(net, false);
  }

  try {
    const logs = stringifyRuntimeResult(await invokeBridgeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net));
    const delta = kgwBridgeR51Delta(KGW_BRIDGE_R51_LAST_LOGS[net], logs);

    if (delta) {
      KGW_BRIDGE_R51_LAST_LOGS[net] = logs;
      appendLog(net, delta);
    }
  } catch {
    // Runtime may not be ready yet.
  }
}

function kgwBridgeR51RefreshAll(reason = "live") {
  for (const net of kgwBridgeR51Keys()) {
    kgwBridgeR51RefreshOne(net, reason);
  }
}

function kgwBridgeR51StartLiveRefresh() {
  if (KGW_BRIDGE_R51_TIMER != null) {
    clearInterval(KGW_BRIDGE_R51_TIMER);
  }

  kgwBridgeR51RefreshAll("initial");

  KGW_BRIDGE_R51_TIMER = setInterval(() => {
    kgwBridgeR51RefreshAll("poll");
  }, 700);
}

function installKgwBridgeR51BottomStyle() {
  if (document.getElementById("kgw-bridge-r51-bottom-style")) return;

  const style = document.createElement("style");
  style.id = "kgw-bridge-r51-bottom-style";
  style.textContent = `
    [data-bridge-network-panel] {
      position: relative;
      min-height: 680px;
      padding-bottom: 48px;
    }

    .bridge-settings-bottom-actions {
      position: absolute;
      right: 12px;
      bottom: 8px;
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      z-index: 2;
    }

    .bridge-settings-bottom-actions button {
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


/* KGW_BRIDGE_ACTION_AND_LOG_FEEDBACK_OWNER_V1 */
function kgwBridgeCurrentVisibleNetwork(root) {
  const activePanel = root.querySelector("[data-bridge-network-panel].active, [data-bridge-network-panel].is-active, [data-bridge-network-panel][data-active='true']");
  if (activePanel?.dataset?.bridgeNetworkPanel) return activePanel.dataset.bridgeNetworkPanel;

  const activeTab = root.querySelector("[data-bridge-network-tab].active, [data-bridge-network-tab].is-active, [data-bridge-network-tab][aria-selected='true'], [data-bridge-network-tab][data-active='true']");
  if (activeTab?.dataset?.bridgeNetworkTab) return activeTab.dataset.bridgeNetworkTab;
  if (activeTab?.dataset?.net) return activeTab.dataset.net;

  return "mainnet";
}


/* KGW_BRIDGE_LOG_FEEDBACK_I18N_OWNER_V1 */

function kgwBridgeTranslateRuntime(key, fallback) {
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

function kgwFlashLogActionButton(button, doneLabel) {
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

function kgwBridgeLogCpuMinerDiagnostic(_net) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  // Diagnostics are not raw bridge stdout/stderr.
}

/* KGW_BRIDGE_SETTINGS_BUTTON_FEEDBACK_FIX_R1
 * Settings action buttons must confirm successful user actions immediately.
 * The existing Bridge action owner calls this helper after save/restore/set-default succeeds.
 */
/* KGW_BRIDGE_SETTINGS_BUTTON_FEEDBACK_HOLD_FIX_R2
 * Keep settings button success labels visible long enough for the user.
 * The helper repeats the label during the hold window to survive fast UI re-renders.
 */



/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29_START */
function kgwBridgeTranslateRuntimeV29(key, fallback) {
  const runtime = window.kgwT || window.kgwI18n || window.__kgwT;
  if (typeof runtime === "function") {
    try {
      const value = runtime(key, fallback);
      if (value && value !== key) return value;
    } catch (_) {}
  }
  return fallback || key;
}

function kgwBridgeLogOutputV29(net) {
  return document.getElementById("bridge-" + net + "-logOutput");
}

function kgwBridgeRestoreLogActionLabelV29(button) {
  if (!button) return;
  const original = button.dataset.kgwLogOriginalLabelV29;
  if (original) button.textContent = original;
  button.classList.remove("kgw-log-action-feedback");
  delete button.dataset.kgwDoneLabel;
}

function kgwBridgeFlashLogActionButtonV29(button, doneLabel) {
  if (!button) return;

  if (!button.dataset.kgwLogOriginalLabelV29) {
    button.dataset.kgwLogOriginalLabelV29 = String(button.textContent || "").trim() || "Log Action";
  }

  window.clearTimeout(button.__kgwLogActionFeedbackTimerV29);

  button.textContent = doneLabel;
  button.dataset.kgwDoneLabel = doneLabel;
  button.classList.add("kgw-log-action-feedback");

  button.__kgwLogActionFeedbackTimerV29 = window.setTimeout(() => {
    kgwBridgeRestoreLogActionLabelV29(button);
  }, 1600);
}

async function kgwBridgeHandleLogActionV29(action, net, button) {
  
  kgwBridgeSmallOwnerTraceR44D(net, String(action || "log-action"), "r51b3-bridge-log-action-click", {
    patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
    action: String(action || ""),
    buttonId: String(button && button.id || ""),
    buttonText: String(button && button.textContent || "").trim()
  });
  kgwBridgeSmallOwnerTraceR44D(net, String(action || "log-action"), "r44d-owner-begin", {});
  const out = kgwBridgeLogOutputV29(net);
  if (!out) return;

  if (action === "copy-log") {
    const text = String(out.value || out.textContent || "");
    if (navigator.clipboard && navigator.clipboard.writeText) {
      await navigator.clipboard.writeText(text).catch(() => {});
    }
    kgwBridgeFlashLogActionButtonV29(button, kgwBridgeTranslateRuntimeV29("log.copied", "Copied"));
    return;
  }

  if (action === "clear-log") {
    if ("value" in out) out.value = "";
    out.textContent = "";
    kgwBridgeFlashLogActionButtonV29(button, kgwBridgeTranslateRuntimeV29("log.deleted", "Deleted"));
  }
  kgwBridgeSmallOwnerTraceR44D(net, String(action || "log-action"), "r44d-owner-complete", {});
}
/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29_END */

function installActions(root) {
  // KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D_ACTIONS
  if (root && !root.dataset.kgwBridgeInprocessNodeTabsV12B) {
    root.dataset.kgwBridgeInprocessNodeTabsV12B = "true";
    root.addEventListener("click", (event) => {
      const tab = event.target?.closest?.("[data-bridge-inprocess-node-tab]");
      if (!tab) return;

      const net = tab.dataset.net;
      const key = tab.dataset.bridgeInprocessNodeTab;
      if (!net || !key) return;

      const section = tab.closest("[data-bridge-inprocess-node-settings]");
      if (!section) return;

      for (const item of section.querySelectorAll("[data-bridge-inprocess-node-tab]")) {
        item.classList.toggle("active", item === tab);
      }

      for (const panel of section.querySelectorAll("[data-bridge-inprocess-node-panel]")) {
        const active = panel.dataset.bridgeInprocessNodePanel === key;
        panel.classList.toggle("active", active);
        panel.hidden = !active;
      }
    });
  }


  // KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26: Bridge settings actions are scoped to the exact bridge/network that changed.
  if (window.KGW_BRIDGE_SETTINGS_OWNER_V19 && typeof window.KGW_BRIDGE_SETTINGS_OWNER_V19.install === "function") {
    window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root);
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
    const carrier = element.closest("[data-net], [data-network], [data-bridge-network-panel], [data-bridge-inner-panel], [data-bridge-section-panel], [data-bridge-instance-panel]");

    return normalizeNet(
      [
        element.dataset && element.dataset.net,
        element.dataset && element.dataset.network,
        carrier && carrier.dataset && carrier.dataset.net,
        carrier && carrier.dataset && carrier.dataset.network,
        carrier && carrier.dataset && carrier.dataset.bridgeNetworkPanel,
        element.id,
        carrier && carrier.id,
        carrier && carrier.className
      ].filter(Boolean).join(" ")
    );
  }

  function netFromEvent(event) {
    return netFromElement(event && event.target);
  }


  // KGW_EXPLICIT_TRACE_OWNER_R27D_BRIDGE_BEGIN
  function kgwBridgeExplicitTraceR27D(net, action, phase, details) {
    try {
      const safeNet = String(net || "unknown");
      const safeAction = String(action || "unknown");
      const safePhase = String(phase || "unknown");
      const payload = {
        patch: "KGW_EXPLICIT_TRACE_EXACT_ANCHOR_PATCH_R27D",
        owner: "bridge-existing-owner",
        network: safeNet,
        action: safeAction,
        phase: safePhase,
        details: details && typeof details === "object" ? details : {}
      };

      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", {
          scope: "bridge",
          net: safeNet,
          action: safeAction,
          phase: safePhase,
          details: JSON.stringify(payload)
        }).catch(function () {});
      }
    } catch (_) {}
  }
  // KGW_EXPLICIT_TRACE_OWNER_R27D_BRIDGE_END

  function scopedUpdate(net, reason) {
    if (!net) return;

    if (typeof bridgeSyncModeControls === "function") {
      bridgeSyncModeControls(net);
    }

    if (typeof updateCommand === "function") {
      updateCommand(net);
    }

    kgwBridgeExplicitTraceR27D(net, "settings-scope", "r27d-scoped-update", {
      previousPatch: "KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26",
      reason: reason || "unknown"
    });
  }
  bridgeInstallAllVisibleInstanceContainerOwnersR11(root);

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
    const button = event.target && event.target.closest ? event.target.closest("[data-bridge-action]") : null;
    if (!button || !root.contains(button)) return;

    const action = button.dataset.bridgeAction;
    const net = normalizeNet(button.dataset.net || button.dataset.network || netFromElement(button));

    if (!net) return;



    kgwBridgeExplicitTraceR27D(net, String(action || "unknown"), "r27d-action-click", {
      trusted: Boolean(event && event.isTrusted),
      disabled: Boolean(button.disabled),
      id: String(button.id || ""),
      instanceId: String(button.dataset.instanceId || button.dataset.instance || ""),
      text: String(button.textContent || "").trim()
    });

    if (action === "select-instance") {
      activeInstance[net] = button.dataset.instanceId;
      bridgeRefreshInstances(net);
      scopedUpdate(net, "select-instance");
      return;
    }

    if (action === "add-instance") {
      addInstance(net);
      scopedUpdate(net, "add-instance");
      return;
    }

    if (action === "remove-instance") {
      removeInstance(net, button.dataset.instanceId);
      scopedUpdate(net, "remove-instance");
      return;
    }

    if (action === "save-settings") {
      if (typeof kgwBridgeR51SaveSettings === "function") kgwBridgeR51SaveSettings(net);
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      if (typeof kgwBridgeR51SetAsDefaults === "function") kgwBridgeR51SetAsDefaults(net);
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      if (typeof kgwBridgeR51RestoreDefaults === "function") kgwBridgeR51RestoreDefaults(net);
      scopedUpdate(net, "restore-defaults");
      return;
    }

    if (action === "copy-log" || action === "clear-log") {
      kgwBridgeHandleLogActionV29(action, net, button).catch(function () {});
      return;
    }

    if (action === "copy-command") {
      const preview = document.getElementById("bridge-" + net + "-commandPreview");
      if (preview && navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(String(preview.value || preview.textContent || "")).catch(function () {});
      }
      return;
    }

    if (action === "start" || action === "stop") {
      if (typeof runBridgeIntegratedAction === "function") {
        runBridgeIntegratedAction(action, net).catch(function (error) {
          if (typeof appendLog === "function") appendLog(net, "Bridge " + action + " failed: " + (error && error.message ? error.message : String(error)));
        });
      }
    }
  }, false);
}

export async function initKaspaBridgeTab(root) {

const bridgeRoot = root || document.getElementById("kaspa-bridge");
  if (!bridgeRoot || bridgeRoot.dataset.kgwBridgeV7Ready === "true") return;

  bridgeRoot.dataset.kgwBridgeV7Ready = "true";

  renderAllNetworks(bridgeRoot);
  kgwBridgeR51CaptureFactoryDefaults();
  kgwBridgeR51LoadSavedSettings();
  bridgeSyncAllModeControls();
  installNetworkTabs(bridgeRoot);
  installDelegatedTabs(bridgeRoot);
  installActions(bridgeRoot);
  bridgeSyncAllModeControls();
  updateAllCommands();
  BRIDGE_NETWORKS.forEach((net) => kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net.key, { force: false })); /* KGW_BRIDGE_DYNAMIC_PATHS_INIT_R3 */
  window.setTimeout(updateAllCommands, 0);
  window.setTimeout(updateAllCommands, 150);
  bridgeSyncAllModeControls();
  updateAllCommands();
  installKgwBridgeR51BottomStyle();
  kgwBridgeR51StartLiveRefresh();


  setTimeout(kgwInstallBridgeLogAutoScrollControlsR27, 0);
}


/* KGW_BRIDGE_LOG_SCOPED_CONTROLS_V29_START */
(function installKgwLogScopedToolbarControlsV29() {
  "use strict";

  const KIND = "bridge";
  const ROOT_SELECTOR = "#kaspa-bridge";
  const TOOLBAR_SELECTOR = ".bridge-v7-log-toolbar";
  const ACTION_ATTR = "data-bridge-action";
  const PREFIX = "bridge";
  const NETWORKS = ["mainnet", "testnet10", "testnet12"];
  const MIN_SIZE = 10;
  const MAX_SIZE = 18;
  const DEFAULT_SIZE = 12;

  function clampSize(value) {
    const parsed = Number.parseInt(String(value || ""), 10);
    if (!Number.isFinite(parsed)) return DEFAULT_SIZE;
    return Math.max(MIN_SIZE, Math.min(MAX_SIZE, parsed));
  }

  function storageKey(net) {
    return "kgw." + KIND + ".log.fontSize." + net;
  }

  function readSize(net) {
    try {
      return clampSize(window.localStorage.getItem(storageKey(net)));
    } catch (_) {
      return DEFAULT_SIZE;
    }
  }

  function writeSize(net, size) {
    const finalSize = clampSize(size);
    try {
      window.localStorage.setItem(storageKey(net), String(finalSize));
    } catch (_) {}
    return finalSize;
  }

  function root() {
    return document.querySelector(ROOT_SELECTOR);
  }

  function logOutput(net) {
    return document.getElementById(PREFIX + "-" + net + "-logOutput");
  }

  function toolbar(net) {
    const r = root();
    if (!r) return null;

    const copyButton = r.querySelector(TOOLBAR_SELECTOR + " [" + ACTION_ATTR + "='copy-log'][data-net='" + net + "']");
    if (copyButton) return copyButton.closest(TOOLBAR_SELECTOR);

    const panel = r.querySelector("[data-net='" + net + "']");
    if (!panel) return null;
    return panel.querySelector(TOOLBAR_SELECTOR);
  }

  function makeButton(label, title) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "kgw-log-font-size-button";
    button.textContent = label;
    button.title = title;
    button.setAttribute("aria-label", title);
    button.dataset.kgwLogFontOwner = "v29";
    return button;
  }

  function applyFontSize(net) {
    const out = logOutput(net);
    const size = readSize(net);

    if (out) {
      out.dataset.kgwLogFontSizePane = "v29";
      out.style.setProperty("--kgw-log-font-size", size + "px");
      out.style.setProperty("font-size", "var(--kgw-log-font-size)", "important");
      out.style.setProperty("line-height", "1.45", "important");
    }

    const tb = toolbar(net);
    if (tb) {
      const value = tb.querySelector(".kgw-log-font-size-value[data-net='" + net + "']");
      if (value) value.textContent = size + "px";
    }
  }

  function removeToolbarDuplicates(tb) {
    if (!tb) return;
    tb.querySelectorAll(".kgw-log-font-size-controls").forEach((item) => item.remove());
  }

  function installForNetwork(net) {
    const tb = toolbar(net);
    if (!tb) return;

    removeToolbarDuplicates(tb);

    const controls = document.createElement("div");
    controls.className = "kgw-log-font-size-controls";
    controls.dataset.kind = KIND;
    controls.dataset.net = net;
    controls.dataset.marker = "KGW_BRIDGE_LOG_SCOPED_CONTROLS_V29";

    const decrease = makeButton("A-", "Decrease log font size");
    const value = document.createElement("span");
    value.className = "kgw-log-font-size-value";
    value.dataset.net = net;
    value.textContent = readSize(net) + "px";

    const increase = makeButton("A+", "Increase log font size");
    const reset = makeButton("Reset", "Reset log font size");

    decrease.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      const previousSize = readSize(net);
      kgwBridgeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-bridge-log-font-decrease-click", {
        patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
        trusted: Boolean(event && event.isTrusted),
        previousSize,
        nextSize: previousSize - 1
      });
      writeSize(net, previousSize - 1);
      applyFontSize(net);
    });

    increase.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      const previousSize = readSize(net);
      kgwBridgeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-bridge-log-font-increase-click", {
        patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
        trusted: Boolean(event && event.isTrusted),
        previousSize,
        nextSize: previousSize + 1
      });
      writeSize(net, previousSize + 1);
      applyFontSize(net);
    });

    reset.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      const previousSize = readSize(net);
      kgwBridgeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-bridge-log-font-reset-click", {
        patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
        trusted: Boolean(event && event.isTrusted),
        previousSize,
        nextSize: DEFAULT_SIZE
      });
      writeSize(net, DEFAULT_SIZE);
      applyFontSize(net);
    });

    controls.append(decrease, value, increase, reset);
    tb.appendChild(controls);

    applyFontSize(net);
  }

  function installAll() {
    for (const net of NETWORKS) {
      installForNetwork(net);
    }
  }

  window.kgwInstallBridgeLogScopedControlsV29 = installAll;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", installAll, { once: true });
  } else {
    window.setTimeout(installAll, 0);
  }
})();
/* KGW_BRIDGE_LOG_SCOPED_CONTROLS_V29_END */

export default initKaspaBridgeTab;

if (typeof window !== "undefined") {
  window.initKaspaBridgeTab = initKaspaBridgeTab;
}
