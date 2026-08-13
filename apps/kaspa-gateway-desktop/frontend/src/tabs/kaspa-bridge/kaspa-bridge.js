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


/* KGW_BRIDGE_CLICK_SAVE_CHECKBOX_TRACE_PATCH_R29B
 * Dev-gated click/save/checkbox trace inside existing KGW_SETTINGS_OWNER_V19.
 * No new listener. No document capture. No MutationObserver.
 */
function kgwSettingsTraceDatasetR29B(target) {
  const out = {};
  try {
    const ds = target && target.dataset ? target.dataset : {};
    for (const key of Object.keys(ds)) {
      if (/^(net|network|nodeAction|bridgeAction|nodeCommandOptionToggleR7|bridgeCommandOptionToggleR7|bridgeInstanceCommandOptionToggleR13B|instanceId|bridgeInstanceField|kgw)/i.test(key)) {
        out[key] = String(ds[key] || "").slice(0, 160);
      }
    }
  } catch (_) {}
  return out;
}

function kgwSettingsTraceTargetSnapshotR29B(target) {
  const snapshot = {
    tag: String(target && target.tagName || ""),
    id: String(target && target.id || ""),
    name: String(target && target.name || ""),
    type: String(target && target.type || ""),
    className: String(target && target.className || "").slice(0, 220),
    dataset: kgwSettingsTraceDatasetR29B(target)
  };

  try {
    if (target && (target.type === "checkbox" || target.type === "radio")) {
      snapshot.checked = Boolean(target.checked);
    } else if (target && "value" in target) {
      const value = String(target.value ?? "");
      snapshot.valueLength = value.length;
      snapshot.valuePreview = value.slice(0, 180);
    }
  } catch (_) {}

  return snapshot;
}

function kgwSettingsTracePreviewSnapshotR29B(root, network) {
  try {
    const netText = String(network || "");
    const candidates = Array.from(root.querySelectorAll("textarea, input, pre, code, [id*='commandPreview'], [data-kgw-command-preview]"));
    const preview = candidates.find(function (item) {
      const id = String(item.id || "");
      return id.indexOf(netText) >= 0 && /commandpreview/i.test(id);
    }) || candidates.find(function (item) {
      return /commandpreview/i.test(String(item.id || ""));
    });

    if (!preview) return { found: false };

    const text = String(("value" in preview ? preview.value : preview.textContent) || "");
    return {
      found: true,
      id: String(preview.id || ""),
      length: text.length,
      preview: text.slice(0, 260)
    };
  } catch (error) {
    return {
      found: false,
      error: String(error && error.message ? error.message : error)
    };
  }
}

function kgwSettingsTraceEventDetailsR29B(root, event, network, reason) {
  return {
    patch: "R29B",
    owner: OWNER,
    scope: SCOPE,
    tab: "bridge",
    reason: String(reason || ""),
    eventType: String(event && event.type || ""),
    trusted: Boolean(event && event.isTrusted),
    network: String(network || ""),
    target: kgwSettingsTraceTargetSnapshotR29B(event && event.target),
    preview: kgwSettingsTracePreviewSnapshotR29B(root, network)
  };
}

function kgwSettingsTraceButtonDetailsR29B(root, event, button, network, action, extra) {
  const details = {
    patch: "R29B",
    owner: OWNER,
    scope: SCOPE,
    tab: "bridge",
    reason: "settings-action-button",
    eventType: String(event && event.type || ""),
    trusted: Boolean(event && event.isTrusted),
    network: String(network || ""),
    action: String(action || ""),
    button: kgwSettingsTraceTargetSnapshotR29B(button),
    disabled: Boolean(button && button.disabled),
    label: String(button && button.textContent || "").trim().slice(0, 160),
    preview: kgwSettingsTracePreviewSnapshotR29B(root, network)
  };

  if (extra && typeof extra === "object") {
    for (const key of Object.keys(extra)) details[key] = extra[key];
  }

  return details;
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

      trace(root, "r29b-settings-control-" + String(event.type || "input") + "-seen", kgwSettingsTraceEventDetailsR29B(root, event, network, "settings-control"));
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

      trace(root, "r29b-settings-control-" + String(event.type || "change") + "-seen", kgwSettingsTraceEventDetailsR29B(root, event, network, "settings-control"));
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

      trace(root, "r29b-settings-action-click", kgwSettingsTraceButtonDetailsR29B(root, event, button, network, action, { disabled: disabled }));
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


/* Canonical isolated bridge/node runtime paths.
 * In-process bridge mode shares the same network-specific database at:
 * %LOCALAPPDATA%\KaspaGateway\nodes\<network>
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

function kgwBridgeRustyKaspaLocalAppDataRootR5(paths = {}, net = "mainnet") {
  const appRoot = kgwBridgeJoinPathR5(kgwBridgeExtractUserLocalAppDataR5(paths), "KaspaGateway");
  const nodesRoot = kgwBridgeJoinPathR5(appRoot, "nodes");
  return kgwBridgeJoinPathR5(nodesRoot, String(net || "mainnet"));
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
  const rustyRoot = kgwBridgeRustyKaspaLocalAppDataRootR5(pathHints, net);

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

/* KGW_BRIDGE_NETWORK_PORT_RANGES_DEFAULTS_PATCH_R42
 * Defaults now follow the agreed soft network port ranges.
 * These are defaults only. Manual valid unused ports remain accepted anywhere.
 */
const BRIDGE_NETWORKS = [
  { key: "mainnet", label: "MAINNET", testnet: false, netsuffix: "", kaspadPort: "16110", stratumPort: ":5555", promPort: ":2112", dashboardPort: "3030", enabledByDefault: true, runtime: "Official stable v2.0.1" },
  { key: "testnet10", label: "TESTNET10", testnet: true, netsuffix: "10", kaspadPort: "16210", stratumPort: ":5655", promPort: ":2212", dashboardPort: "3130", enabledByDefault: true, runtime: "Official stable v2.0.1" },
  { key: "testnet12", label: "TESTNET12", testnet: true, netsuffix: "12", kaspadPort: "16310", stratumPort: ":5755", promPort: ":2312", dashboardPort: "3230", enabledByDefault: false, experimental: true, runtime: "Experimental TN12 build" }
];

function kgwBridgeNetworkPolicyKey(net) {
  return `kgw.bridge.network.enabled.${String(net || "unknown")}`;
}

function kgwBridgeNetworkProfile(net) {
  return BRIDGE_NETWORKS.find((item) => item.key === net) || null;
}

function kgwBridgeNetworkEnabled(net) {
  const profile = kgwBridgeNetworkProfile(net);
  try {
    const stored = localStorage.getItem(kgwBridgeNetworkPolicyKey(net));
    if (stored === "1") return true;
    if (stored === "0") return false;
  } catch (_) {}
  return profile ? profile.enabledByDefault !== false : false;
}

function kgwBridgeSetNetworkEnabled(net, enabled) {
  try {
    localStorage.setItem(kgwBridgeNetworkPolicyKey(net), enabled ? "1" : "0");
  } catch (_) {}
}

function kgwBridgeNetworkPolicyMessage(net) {
  const profile = kgwBridgeNetworkProfile(net);
  if (!profile) return "";
  if (profile.experimental) {
    return "Experimental network. Disabled by default and requires explicit opt-in.";
  }
  return `${profile.runtime}. External local-node mode is recommended for mining bridges.`;
}

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


// KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B
const KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B = "KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B";

function kgwBridgeInstanceCommandStateKeyR13B(net, instanceId, name) {
  return `${String(net || "mainnet")}::${String(instanceId || "1")}::${String(name || "")}`;
}

function kgwBridgeInstanceCommandOptionEnabledR13B(net, instanceId, name) {
  const key = kgwBridgeInstanceCommandStateKeyR13B(net, instanceId, name);
  window.__kgwBridgeInstanceCommandComposerR13B = window.__kgwBridgeInstanceCommandComposerR13B || {};
  return window.__kgwBridgeInstanceCommandComposerR13B[key] !== false;
}

function kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, name) {
  return kgwBridgeInstanceCommandOptionEnabledR13B(net, instanceId, name);
}

function kgwBridgeInstanceCommandCheckboxR13B(net, instanceId, name) {
  const enabled = kgwBridgeInstanceCommandOptionEnabledR13B(net, instanceId, name);
  return `<input type="checkbox" class="kgw-command-option-checkbox-r9 kgw-bridge-instance-command-checkbox-r13b" data-bridge-instance-command-option-toggle-r13b="${esc(String(name))}" data-net="${esc(String(net))}" data-instance-id="${esc(String(instanceId))}" ${enabled ? "checked" : ""} aria-label="${enabled ? "Included in command" : "Excluded from command"}" title="${enabled ? "Included in command" : "Excluded from command"}">`;
}

function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {
  kgwBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r29b-bridge-instance-command-checkbox-begin", {
    patch: "R29B",
    owner: "bridge-instance-command-composer-r13b",
    instanceId: String(instanceId || ""),
    option: String(name || ""),
    enabled: Boolean(enabled)
  });

  const key = kgwBridgeInstanceCommandStateKeyR13B(net, instanceId, name);
  window.__kgwBridgeInstanceCommandComposerR13B = window.__kgwBridgeInstanceCommandComposerR13B || {};
  window.__kgwBridgeInstanceCommandComposerR13B[key] = Boolean(enabled);
  updateCommand(net);
  bridgeSyncInstancePreviewRowsR8B(net);

  kgwBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r29b-bridge-instance-command-checkbox-complete", {
    patch: "R29B",
    owner: "bridge-instance-command-composer-r13b",
    key: String(key || ""),
    instanceId: String(instanceId || ""),
    option: String(name || ""),
    enabled: Boolean(enabled)
  });
}


function addFlag(lines, net, name, flag) {
  if (!kgwBridgeCommandShouldIncludeR7(net, name)) return; // KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7

  if (c(net, name)) lines.push(flag);
}

function addValue(lines, net, name, flag) {
  if (!kgwBridgeCommandShouldIncludeR7(net, name)) return; // KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7

  const value = v(net, name);
  if (value) lines.push(`${flag}=${value}`);
}

function addBoolValue(lines, net, name, flag) {
  const value = v(net, name);
  if (value && value !== "not set") lines.push(`${flag}=${value}`);
}



// KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7
const KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7 = "KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7";

function kgwBridgeCommandInlineStateKeyR7(net) {
  return String(net || "mainnet");
}

function kgwBridgeCommandInlineStateR7(net) {
  const key = kgwBridgeCommandInlineStateKeyR7(net);
  window.__kgwBridgeCommandComposerInlineR7 = window.__kgwBridgeCommandComposerInlineR7 || {};
  window.__kgwBridgeCommandComposerInlineR7[key] = window.__kgwBridgeCommandComposerInlineR7[key] || {};
  return window.__kgwBridgeCommandComposerInlineR7[key];
}

function kgwBridgeCommandOptionEnabledR7(net, name) {
  const state = kgwBridgeCommandInlineStateR7(net);
  return state[String(name)] !== false;
}

function kgwBridgeCommandShouldIncludeR7(net, name) {
  return kgwBridgeCommandOptionEnabledR7(net, name);
}

function kgwBridgeCommandInlineToggleR7(net, name) {
  const enabled = kgwBridgeCommandOptionEnabledR7(net, name);
  const label = enabled ? "Included" : "Excluded";
  return `<input type="checkbox" class="kgw-command-option-checkbox-r9" data-bridge-command-option-toggle-r7="${esc(String(name))}" data-net="${esc(String(net))}" ${enabled ? "checked" : ""} aria-label="${enabled ? "Included in command" : "Excluded from command"}" title="${enabled ? "Included in command" : "Excluded from command"}">`; // KGW_BRIDGE_COMMAND_COMPOSER_CHECKBOX_ONLY_R9
}

function kgwBridgeRefreshInlineCommandTogglesR7(net) {
  document.querySelectorAll(`[data-bridge-command-option-toggle-r7][data-net="${CSS.escape(String(net))}"]`).forEach((el) => {
    const name = el.dataset.bridgeCommandOptionToggleR7;
    const enabled = kgwBridgeCommandOptionEnabledR7(net, name);
    el.checked = enabled;
    el.setAttribute("aria-label", enabled ? "Included in command" : "Excluded from command");
    el.setAttribute("title", enabled ? "Included in command" : "Excluded from command");
    el.classList.toggle("is-on", enabled);
    el.classList.toggle("is-off", !enabled);
  });
}

function kgwBridgeToggleCommandOptionR7(net, name) {
  const state = kgwBridgeCommandInlineStateR7(net);
  const key = String(name);
  state[key] = state[key] === false;
  kgwBridgeRefreshInlineCommandTogglesR7(net);
  updateCommand(net);
}


// KGW_BRIDGE_DIFFICULTY_DATALIST_R16C
const KGW_BRIDGE_DIFFICULTY_DATALIST_R16C = "KGW_BRIDGE_DIFFICULTY_DATALIST_R16C";

function kgwBridgeDifficultyPresetValuesR16C() {
  return [
    "1",
    "2",
    "4",
    "8",
    "16",
    "32",
    "64",
    "128",
    "256",
    "512",
    "1024",
    "2048",
    "4096",
    "8192",
    "16384",
    "32768",
    "65536"
  ];
}

function kgwBridgeDifficultyDatalistIdR16C() {
  return "kgw-bridge-difficulty-presets-r16c";
}

function kgwBridgeDifficultyDatalistR16C() {
  return `<datalist id="${kgwBridgeDifficultyDatalistIdR16C()}">${kgwBridgeDifficultyPresetValuesR16C().map((value) => `<option value="${esc(value)}"></option>`).join("")}</datalist>`;
}

function kgwBridgeDifficultyInputAttrsR16C(name) {
  const key = String(name || "");
  if (!["minShareDiff", "sharesPerMin", "instanceDiff", "instanceSharesPerMin"].includes(key)) return "";
  return `list="${kgwBridgeDifficultyDatalistIdR16C()}" inputmode="numeric" autocomplete="off" data-kgw-difficulty-preset-r16c="${esc(key)}"`;
}

function cardInput(net, name, label, value = "", placeholder = "", span = "", inputAttrs = "") {
  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net, name)}
        <span class="kgw-command-option-title-text-r8e">${esc(label)}</span>
      </span> <!-- KGW_BRIDGE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->
      <input ${inputAttrs} id="${id(net, name)}" data-testid="kgw-bridge-field-${esc(net)}-${esc(name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function cardSelect(net, name, label, options, value = "", span = "") {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net, name)}
        <span class="kgw-command-option-title-text-r8e">${esc(label)}</span>
      </span> <!-- KGW_BRIDGE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->
      <select id="${id(net, name)}" data-testid="kgw-bridge-field-${esc(net)}-${esc(name)}">${opts}</select>
    </div>`;
}

function cardCheck(net, name, label, checked = false, span = "") {
  return `
    <label class="bridge-v7-card check${span ? " " + span : ""}">
      <input id="${id(net, name)}" data-testid="kgw-bridge-field-${esc(net)}-${esc(name)}" type="checkbox"${checked ? " checked" : ""}>
      <span>${esc(label)}</span>
    </label>`;
}

function instanceInput(net, instanceId, name, label, value = "", placeholder = "", span = "") {
  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeInstanceCommandCheckboxR13B(net, instanceId, name)}
        <span class="kgw-command-option-title-text-r8e">${esc(label)}</span>
      </span> <!-- KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B -->
      <input id="${iid(net, instanceId, name)}" data-testid="kgw-bridge-instance-field-${esc(net)}-${esc(instanceId)}-${esc(name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function instanceSelect(net, instanceId, name, label, options, value = "", span = "") {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeInstanceCommandCheckboxR13B(net, instanceId, name)}
        <span class="kgw-command-option-title-text-r8e">${esc(label)}</span>
      </span> <!-- KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B -->
      <select id="${iid(net, instanceId, name)}" data-testid="kgw-bridge-instance-field-${esc(net)}-${esc(instanceId)}-${esc(name)}">${opts}</select>
    </div>`;
}

function instanceCheck(net, instanceId, name, label, checked = false, span = "") {
  return `
    <label class="bridge-v7-card check${span ? " " + span : ""}">
      <input id="${iid(net, instanceId, name)}" data-testid="kgw-bridge-instance-field-${esc(net)}-${esc(instanceId)}-${esc(name)}" type="checkbox"${checked ? " checked" : ""}>
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
      ${cardInput(net.key, "minShareDiff", "--min-share-diff", "8192", "", "", kgwBridgeDifficultyInputAttrsR16C("minShareDiff"))}
      ${cardInput(net.key, "sharesPerMin", "--shares-per-min", "30", "", "", kgwBridgeDifficultyInputAttrsR16C("sharesPerMin"))}
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


/* KGW_BRIDGE_INSTANCE_PLACEHOLDER_PORT_RANGES_PATCH_R47
 * Instance placeholder examples now follow the agreed network port ranges.
 * This changes display/help text only. It does not overwrite saved user ports.
 */
function bridgeInstanceExamplePlaceholderR47(net) {
  const profile = typeof bridgePortProfileR35B === "function"
    ? bridgePortProfileR35B(net)
    : null;

  const stratum = profile && profile.stratum && profile.stratum.instanceStart
    ? profile.stratum.instanceStart
    : 5556;

  const prom = profile && profile.prom && profile.prom.instanceStart
    ? profile.prom.instanceStart
    : 2113;

  return "port=:" + String(stratum) + ",diff=2048,prom=:" + String(prom);
}

function renderInstancePanel(net, instanceId) {
  return `
    <section class="bridge-v7-instance-panel" data-net="${net.key}" data-instance-panel="${instanceId}"${activeInstance[net.key] === instanceId ? "" : " hidden"}>
      <div class="bridge-v7-grid">
        <div class="bridge-v7-card span3">
          <span>--instance</span>
          <textarea id="${iid(net.key, instanceId, "instance")}" class="bridge-v7-instance-text" placeholder="${esc(bridgeInstanceExamplePlaceholderR47(net.key))}"></textarea>
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
  // KGW_BRIDGE_INSTANCE_UPSTREAM_SERIALIZATION_CHECKBOX_R13B

  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instancePort")) parts = bridgeInstanceAppend(parts, "port", bridgeInstancePortValue(bridgeInstanceReadSupplement(net, instanceId, "instancePort", instance?.instancePort)));
  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instanceDiff")) parts = bridgeInstanceAppend(parts, "diff", bridgeInstancePlainValue(bridgeInstanceReadSupplement(net, instanceId, "instanceDiff", instance?.instanceDiff)));
  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instanceProm")) parts = bridgeInstanceAppend(parts, "prom", bridgeInstancePortValue(bridgeInstanceReadSupplement(net, instanceId, "instanceProm", instance?.instanceProm)));

  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instanceLogToFile")) parts = bridgeInstanceAppend(parts, "log", bridgeInstanceReadSupplement(net, instanceId, "instanceLogToFile", instance?.instanceLogToFile));
  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instanceVarDiff")) parts = bridgeInstanceAppend(parts, "var_diff", bridgeInstanceReadSupplement(net, instanceId, "instanceVarDiff", instance?.instanceVarDiff));
  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instanceSharesPerMin")) parts = bridgeInstanceAppend(parts, "shares_per_min", bridgeInstanceReadSupplement(net, instanceId, "instanceSharesPerMin", instance?.instanceSharesPerMin));
  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instanceVarDiffStats")) parts = bridgeInstanceAppend(parts, "var_diff_stats", bridgeInstanceReadSupplement(net, instanceId, "instanceVarDiffStats", instance?.instanceVarDiffStats));
  if (kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instancePow2Clamp")) parts = bridgeInstanceAppend(parts, "pow2_clamp", bridgeInstanceReadSupplement(net, instanceId, "instancePow2Clamp", instance?.instancePow2Clamp));

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

function bridgePortConflictLogicalKeyR64F(item) {
  const net = String(item && item.net ? item.net : "");
  const role = String(item && item.role ? item.role : "");
  const owner = String(item && item.owner ? item.owner : "");

  if (
    role === "default-kaspad-rpc" ||
    role === "bridge-external-kaspad" ||
    role === "inprocess-rpc"
  ) {
    return net + ":node-rpc";
  }

  if (
    role === "default-stratum" ||
    role === "bridge-stratum"
  ) {
    return net + ":bridge-stratum";
  }

  if (
    role === "default-prometheus" ||
    role === "bridge-prometheus"
  ) {
    return net + ":bridge-prometheus";
  }

  return net + ":" + role + ":" + owner;
}

function bridgePortOwnersRepresentSameLogicalEndpointR64F(owners) {
  const logicalKeys = new Set(
    owners.map((item) => bridgePortConflictLogicalKeyR64F(item))
  );

  return logicalKeys.size <= 1;
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
    const uniqueOwners = new Set(
      owners.map((item) => item.net + ":" + item.role + ":" + item.owner)
    );

    if (uniqueOwners.size <= 1) continue;

    if (bridgePortOwnersRepresentSameLogicalEndpointR64F(owners)) {
      continue;
    }

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
  const profile = bridgePortProfileR35B(net);
  const usedPorts = bridgeUsedPortSetR8B();

  const instancePort = bridgeFindRecommendedOrNearestUnusedPortR35B(net, "stratum", usedPorts, profile.stratum.instanceStart);
  const instanceProm = bridgeFindRecommendedOrNearestUnusedPortR35B(net, "prom", usedPorts, profile.prom.instanceStart);

  bridgeTracePortProfileR35B(net, "r35b-auto-assign-instance-ports-r8b", {
    instancePort,
    instanceProm,
    stratumRange: [profile.stratum.min, profile.stratum.max],
    promRange: [profile.prom.min, profile.prom.max]
  });

  return { instancePort, instanceProm };
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

function bridgePortIsInsideAnyKnownRangeR91(net, kind, port) {
  const normalized = bridgeNormalizePortR9(port);
  if (!bridgePortIsValidR9(normalized)) return false;

  const currentProfile = bridgePortProfileR35B(net);
  const staticProfile = bridgeStaticPortProfileR91(net);
  const ranges = [
    currentProfile && currentProfile[kind],
    staticProfile && staticProfile[kind]
  ].filter(Boolean);

  return ranges.some((range) => bridgePortInRangeR35B(normalized, range));
}

function bridgeInstancePortShouldFollowExternalRangeR91(net, kind, value) {
  const normalized = bridgeNormalizePortR9(value);

  if (!normalized) return true;
  if (!bridgePortIsValidR9(normalized)) return true;

  return bridgePortIsInsideAnyKnownRangeR91(net, kind, normalized);
}

function bridgeAddUsedPortR91(used, value) {
  const normalized = bridgeNormalizePortR9(value);
  if (bridgePortIsValidR9(normalized)) used.add(String(normalized));
}

function bridgeUsedPortSetExcludingNetworkInstancesR91(activeNet) {
  const used = new Set();

  for (const profile of BRIDGE_NETWORKS) {
    const net = profile.key;

    bridgeAddUsedPortR91(used, profile.kaspadPort);
    bridgeAddUsedPortR91(used, profile.stratumPort);
    bridgeAddUsedPortR91(used, profile.promPort);

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
      const value = v(net, field);
      for (const port of bridgeExtractPortsFromTextR5(value)) {
        bridgeAddUsedPortR91(used, port);
      }
    }
  }

  for (const [net, list] of Object.entries(bridgeInstances)) {
    if (String(net) === String(activeNet)) continue;
    if (!Array.isArray(list)) continue;

    for (const instance of list) {
      bridgeAddUsedPortR91(used, instance && instance.instancePort);
      bridgeAddUsedPortR91(used, instance && instance.instanceProm);

      for (const port of bridgeExtractPortsFromTextR5(instance && instance.instance || "")) {
        bridgeAddUsedPortR91(used, port);
      }
    }
  }

  return used;
}

function bridgeAssignMissingInstancePortsR9(net, instance) {
  const profile = bridgePortProfileR35B(net);
  const used = bridgeUsedPortSetR9(net, instance.id);

  const currentPort = bridgeNormalizePortR9(instance.instancePort);
  const currentProm = bridgeNormalizePortR9(instance.instanceProm);

  const followStratumRange = bridgeInstancePortShouldFollowExternalRangeR91(net, "stratum", currentPort);
  const followPromRange = bridgeInstancePortShouldFollowExternalRangeR91(net, "prom", currentProm);

  const instancePort = !followStratumRange && bridgePortIsValidR9(currentPort)
    ? currentPort
    : bridgeFindRecommendedOrNearestUnusedPortR35B(net, "stratum", used, profile.stratum.instanceStart);

  const instanceProm = !followPromRange && bridgePortIsValidR9(currentProm)
    ? currentProm
    : bridgeFindRecommendedOrNearestUnusedPortR35B(net, "prom", used, profile.prom.instanceStart);

  bridgeTracePortProfileR35B(net, "r91-assign-instance-ports-from-external-range", {
    instanceId: String(instance && instance.id || ""),
    acceptedManualInstancePort: Boolean(!followStratumRange && currentPort && bridgePortIsValidR9(currentPort)),
    acceptedManualInstanceProm: Boolean(!followPromRange && currentProm && bridgePortIsValidR9(currentProm)),
    instancePort,
    instanceProm,
    stratumRange: [profile.stratum.min, profile.stratum.max],
    promRange: [profile.prom.min, profile.prom.max],
    stratumExternalBase: String(profile.stratum.externalBase || ""),
    promExternalBase: String(profile.prom.externalBase || ""),
    policy: "instances follow bridge-level external port settings unless a valid out-of-range manual port is clearly set"
  });

  return { ...instance, instancePort, instanceProm };
}

function bridgeReassignInstancePortsFromExternalRangeR91(net, reason) {
  net = bridgeInstanceNetworkKeyR15(net, net);
  if (!Array.isArray(bridgeInstances[net])) return false;

  const profile = bridgePortProfileR35B(net);
  const used = bridgeUsedPortSetExcludingNetworkInstancesR91(net);
  let changed = false;

  bridgeInstances[net] = bridgeInstances[net].map((raw, index) => {
    const instance = bridgeNormalizeInstanceRecord(raw, raw && raw.id ? raw.id : Date.now() + index);
    const currentPort = bridgeNormalizePortR9(instance.instancePort);
    const currentProm = bridgeNormalizePortR9(instance.instanceProm);

    const shouldFollowPort = bridgeInstancePortShouldFollowExternalRangeR91(net, "stratum", currentPort);
    const shouldFollowProm = bridgeInstancePortShouldFollowExternalRangeR91(net, "prom", currentProm);

    let instancePort = currentPort;
    let instanceProm = currentProm;

    if (shouldFollowPort) {
      instancePort = bridgeFindRecommendedOrNearestUnusedPortR35B(net, "stratum", used, Number(profile.stratum.instanceStart) + index);
      changed = changed || instancePort !== currentPort;
    } else {
      bridgeAddUsedPortR91(used, instancePort);
    }

    if (shouldFollowProm) {
      instanceProm = bridgeFindRecommendedOrNearestUnusedPortR35B(net, "prom", used, Number(profile.prom.instanceStart) + index);
      changed = changed || instanceProm !== currentProm;
    } else {
      bridgeAddUsedPortR91(used, instanceProm);
    }

    return {
      ...instance,
      instance: "",
      instancePort,
      instanceProm
    };
  });

  if (changed) {
    bridgeTracePortProfileR35B(net, "r91-reassign-instances-from-external-range", {
      reason: String(reason || ""),
      stratumRange: [profile.stratum.min, profile.stratum.max],
      promRange: [profile.prom.min, profile.prom.max],
      stratumExternalBase: String(profile.stratum.externalBase || ""),
      promExternalBase: String(profile.prom.externalBase || ""),
      instanceCount: bridgeInstances[net].length
    });
  }

  return changed;
}

function bridgeCreateInstanceRecordR9(net) {
  bridgeReassignInstancePortsFromExternalRangeR91(net, "before-create-instance");
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


// KGW_BRIDGE_INSTANCE_PORT_CONFLICT_REPAIR_R110G
function kgwBridgeRepairInstancePortsBeforeConflictR110G(triggerNet) {
  const nets = Array.isArray(BRIDGE_NETWORK_ORDER)
    ? BRIDGE_NETWORK_ORDER
    : Object.keys(BRIDGE_NETWORKS || {});

  const normalizePort = (value) => {
    const clean = String(value || "").trim().replace(/^:/, "");
    if (!/^\d+$/.test(clean)) return "";
    const n = Number(clean);
    if (!Number.isInteger(n) || n <= 0 || n > 65535) return "";
    return String(n);
  };

  const nextFreePort = (basePort, used) => {
    const base = Number(normalizePort(basePort)) || 5555;
    for (let offset = 1; offset <= 99; offset += 1) {
      const candidate = String(base + offset);
      if (!used.has(candidate)) return candidate;
    }

    for (let candidate = 1024; candidate <= 65535; candidate += 1) {
      const asText = String(candidate);
      if (!used.has(asText)) return asText;
    }

    return "";
  };

  const changed = [];

  for (const net of nets) {
    const cfg = BRIDGE_NETWORKS?.[net] || {};
    const defaultPort = normalizePort(cfg.stratumPort || cfg.port || "");
    const instances = Array.isArray(bridgeInstances?.[net]) ? bridgeInstances[net] : [];
    const used = new Set();

    if (defaultPort) {
      used.add(defaultPort);
    }

    for (const inst of instances) {
      if (!inst || typeof inst !== "object") continue;

      const before = normalizePort(inst.instancePort || inst.port || inst.stratumPort || "");
      let after = before;

      if (!after || used.has(after)) {
        after = nextFreePort(defaultPort || before || "5555", used);
      }

      if (after) {
        used.add(after);
      }

      if (after && after !== before) {
        inst.instancePort = after;
        if (Object.prototype.hasOwnProperty.call(inst, "port")) inst.port = after;
        if (Object.prototype.hasOwnProperty.call(inst, "stratumPort")) inst.stratumPort = after;

        changed.push({
          net,
          id: String(inst.id || ""),
          before,
          after,
          defaultPort,
          triggerNet: String(triggerNet || "")
        });
      }
    }
  }

  if (changed.length > 0) {
    try {
      kgwBridgeTrace?.("bridge", String(triggerNet || ""), "port-conflict", "r110g-instance-ports-repaired", {
        patch: "KGW_BRIDGE_INSTANCE_PORT_CONFLICT_REPAIR_R110G",
        owner: "bridgeAssertNoPortConflictsR5-existing-owner",
        changed
      });
    } catch (_) {}

    try {
      if (typeof renderInstances === "function" && triggerNet) renderInstances(triggerNet);
    } catch (_) {}

    try {
      if (typeof updateCommand === "function" && triggerNet) updateCommand(triggerNet);
    } catch (_) {}
  }

  return changed;
}


// KGW_BRIDGE_AUTOFIX_BUTTON_INITIAL_LABEL_R111G
function kgwBridgeAutofixButtonInitialLabelR111G(root = document) {
  const rawKey = "bridge.autofixPorts.button";
  const fallback = "Auto Fix Ports";

  try {
    const candidates = Array.from(root.querySelectorAll("button, [role='button']"));
    for (const el of candidates) {
      const text = String(el.textContent || "").trim();
      if (text === rawKey) {
        el.textContent = fallback;
        el.setAttribute("data-i18n", rawKey);
        el.setAttribute("data-kgw-owner", "bridgeInstances");
      }
    }
  } catch (_) {}
}

function bridgeAssertNoPortConflictsR5(net) {
  // KGW_BRIDGE_SCOPED_START_CONFLICT_R110H
  // Start validation must be scoped to the requested network/active instance.
  // Stale or duplicated rows from other networks must not block Start.
  const normalizePort = (value) => {
    const clean = String(value || "").trim().replace(/^:/, "");
    if (!/^\d+$/.test(clean)) return "";
    const n = Number(clean);
    if (!Number.isInteger(n) || n <= 0 || n > 65535) return "";
    return String(n);
  };

  const cfg = BRIDGE_NETWORKS?.[net] || {};
  const defaultPort = normalizePort(cfg.stratumPort || cfg.port || "");
  const structured = typeof kgwBridgeR51ReadStructuredInstancesR26B === "function"
    ? kgwBridgeR51ReadStructuredInstancesR26B(net)
    : { activeInstance: String(activeInstance?.[net] || ""), instances: Array.isArray(bridgeInstances?.[net]) ? bridgeInstances[net] : [] };

  const instances = Array.isArray(structured?.instances) ? structured.instances : [];
  const activeId = String(structured?.activeInstance || activeInstance?.[net] || "");
  const uniqueById = new Map();

  for (const item of instances) {
    if (!item || typeof item !== "object") continue;
    const id = String(item.id || "");
    const key = id || JSON.stringify(item);
    if (!uniqueById.has(key)) uniqueById.set(key, item);
  }

  const activeRecord = activeId && uniqueById.has(activeId)
    ? uniqueById.get(activeId)
    : Array.from(uniqueById.values())[0] || null;

  const activePort = normalizePort(
    activeRecord?.instancePort ||
    activeRecord?.port ||
    activeRecord?.stratumPort ||
    ""
  );

  const conflictDetails = {
    patch: "R110H",
    owner: "existing-bridge-port-conflict-owner-r5-scoped-start",
    network: net,
    defaultPort,
    activeInstanceId: activeId,
    activeInstancePort: activePort,
    instanceCount: instances.length,
    uniqueInstanceCount: uniqueById.size,
    policy: "start checks current network active instance only; stale cross-network conflicts are not blockers"
  };

  try {
    kgwBridgeTrace?.("bridge", net, "port-conflict", "r110h-scoped-start-conflict-check", conflictDetails);
  } catch (_) {}

  if (!activeRecord) {
    return { ok: true, conflictCount: 0, conflicts: [], message: "" };
  }

  if (!activePort) {
    const msg = "Active Bridge instance has no valid Stratum port.";
    try {
      kgwBridgeTrace?.("bridge", net, "port-conflict", "r110h-active-instance-port-invalid", {
        ...conflictDetails,
        message: msg
      });
    } catch (_) {}
    throw new Error(msg);
  }

  // If the selected instance uses the network default port, do not block Start here.
  // R110F backend now uses the active instance contract as the runtime start target.
  // The old global blocker incorrectly treated default-vs-instance as two separate listeners.
  return { ok: true, conflictCount: 0, conflicts: [], message: "" };
}


/* KGW_BRIDGE_PORT_CONFLICT_START_GATE_PATCH_R33
 * Existing Bridge port conflict owner enhancement:
 * - Uses existing bridgeValidatePortConflictsR5 registry.
 * - Blocks Start before runtime if any configured port conflict touches the active network.
 * - Updates Start button disabled/title state live.
 * - Save remains allowed with warning.
 * - Covers mainnet, testnet10, testnet12 through BRIDGE_NETWORKS.
 */
function bridgePortConflictCompactSummaryR33(validation) {
  const conflicts = validation && Array.isArray(validation.conflicts) ? validation.conflicts : [];
  return conflicts.map((item) => {
    const owners = Array.isArray(item.owners) ? item.owners : [];
    return {
      port: String(item.port || ""),
      owners: owners.map((owner) => ({
        net: String(owner.net || ""),
        role: String(owner.role || ""),
        owner: String(owner.owner || "")
      }))
    };
  });
}

function bridgePortConflictMessageR33(validation) {
  if (!validation || validation.ok) return "";
  const message = String(validation.message || "").trim();
  if (message) return message;
  return bridgePortConflictCompactSummaryR33(validation).map((item) => {
    return "port " + item.port + " => " + item.owners.map((owner) => owner.net + "/" + owner.role + "/" + owner.owner).join(" | ");
  }).join("; ");
}

function bridgeTracePortConflictR33(net, phase, validation, details) {
  try {
    kgwBridgeSmallOwnerTraceR44D(net, "port-conflict", phase, {
      patch: "R33",
      owner: "existing-bridge-port-conflict-owner-r5-r33",
      ok: Boolean(validation && validation.ok),
      conflictCount: validation && Array.isArray(validation.conflicts) ? validation.conflicts.length : 0,
      message: bridgePortConflictMessageR33(validation).slice(0, 1200),
      conflicts: bridgePortConflictCompactSummaryR33(validation).slice(0, 20),
      details: details && typeof details === "object" ? details : {}
    });
  } catch (_) {}
}

function bridgeStartButtonsForNetR33(net) {
  const root = document.getElementById("kaspa-bridge");
  if (!root) return [];
  const safeNet = String(net || "");
  return Array.from(root.querySelectorAll('[data-bridge-action="start"][data-net="' + safeNet + '"]'));
}

function bridgeApplyPortConflictStartStateR33(net, validation, reason) {
  const buttons = bridgeStartButtonsForNetR33(net);
  const blocked = Boolean(validation && !validation.ok);
  const message = bridgePortConflictMessageR33(validation);

  for (const button of buttons) {
    if (!button) continue;

    if (blocked) {
      button.disabled = true;
      button.classList.add("kgw-port-conflict-blocked-r33");
      button.dataset.kgwPortConflictBlockedR33 = "true";
      button.dataset.kgwPortConflictMessageR33 = message.slice(0, 800);
      button.title = "Port conflict: " + message.slice(0, 700);
    } else if (button.dataset.kgwPortConflictBlockedR33 === "true") {
      button.disabled = false;
      button.classList.remove("kgw-port-conflict-blocked-r33");
      delete button.dataset.kgwPortConflictBlockedR33;
      delete button.dataset.kgwPortConflictMessageR33;
      if (String(button.title || "").startsWith("Port conflict:")) button.title = "";
    }
  }

  if (blocked) {
    bridgeTracePortConflictR33(net, "r33-port-conflict-detected", validation, {
      reason: String(reason || ""),
      startButtonCount: buttons.length
    });
  } else {
    bridgeTracePortConflictR33(net, "r33-port-validation-clear", validation, {
      reason: String(reason || ""),
      startButtonCount: buttons.length
    });
  }

  return {
    ok: !blocked,
    blocked,
    message,
    validation
  };
}

function bridgeValidateAndApplyPortConflictStateR33(net, reason) {
  const normalized = String(net || "").trim();
  if (!normalized) return { ok: true, blocked: false, message: "", validation: { ok: true, conflicts: [] } };

  const validation = bridgeValidatePortConflictsR5(normalized);
  return bridgeApplyPortConflictStartStateR33(normalized, validation, reason);
}

function bridgeValidateAllPortConflictStatesR33(reason) {
  const results = {};
  for (const profile of BRIDGE_NETWORKS) {
    const net = String(profile && profile.key || "");
    if (!net) continue;
    results[net] = bridgeValidateAndApplyPortConflictStateR33(net, reason || "all");
  }
  return results;
}

function bridgeAssertNoPortConflictsBeforeStartR33(net) {
  const result = bridgeValidateAndApplyPortConflictStateR33(net, "pre-start");
  if (result.blocked) {
    bridgeTracePortConflictR33(net, "r33-port-start-blocked", result.validation, {
      reason: "pre-start",
      action: "start"
    });
    kgwBridgeSetRuntimeErrorV1(net, "Bridge start blocked: port conflict. " + result.message);
    return false;
  }
  return true;
}

function bridgeSchedulePortConflictValidationR33(net, reason) {
  const normalized = String(net || "").trim();
  window.clearTimeout(window.__kgwBridgePortConflictValidationTimerR33);
  window.__kgwBridgePortConflictValidationTimerR33 = window.setTimeout(() => {
    if (normalized) {
      bridgeValidateAndApplyPortConflictStateR33(normalized, reason || "scheduled");
    } else {
      bridgeValidateAllPortConflictStatesR33(reason || "scheduled-all");
    }
  }, 60);
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



/* KGW_BRIDGE_INSTANCE_FIELD_PLACEHOLDERS_RANGE_PATCH_R49
 * Field-level instance port placeholders now follow the active network profile.
 * Display/help text only. Does not overwrite saved user ports.
 */
function bridgeInstancePortPlaceholderR49(net) {
  const profile = typeof bridgePortProfileR35B === "function" ? bridgePortProfileR35B(net) : null;
  const value = profile && profile.stratum && profile.stratum.instanceStart ? profile.stratum.instanceStart : 5556;
  return String(value);
}

function bridgeInstancePromPlaceholderR49(net) {
  const profile = typeof bridgePortProfileR35B === "function" ? bridgePortProfileR35B(net) : null;
  const value = profile && profile.prom && profile.prom.instanceStart ? profile.prom.instanceStart : 2113;
  return String(value);
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
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instance")}
              <span class="kgw-command-option-title-text-r8e">--instance preview</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input
              readonly
              data-bridge-instance-preview="true"
              data-network="${net}"
              data-instance-id="${instance.id}"
              value="${bridgeInstancePreviewTextR8B(net, instance)}"
              title="${bridgeInstancePreviewTextR8B(net, instance)}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instancePort")}
              <span class="kgw-command-option-title-text-r8e">port</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instancePort-${instance.id}`)}" data-bridge-instance-field="instancePort" value="${instance.instancePort || ""}" placeholder="${esc(bridgeInstancePortPlaceholderR49(net))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceDiff")}
              <span class="kgw-command-option-title-text-r8e">diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instanceDiff-${instance.id}`)}" data-bridge-instance-field="instanceDiff" value="${instance.instanceDiff || "2048"}" placeholder="2048" ${kgwBridgeDifficultyInputAttrsR16C("instanceDiff")} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceProm")}
              <span class="kgw-command-option-title-text-r8e">prom</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instanceProm-${instance.id}`)}" data-bridge-instance-field="instanceProm" value="${instance.instanceProm || ""}" placeholder="${esc(bridgeInstancePromPlaceholderR49(net))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceLogToFile")}
              <span class="kgw-command-option-title-text-r8e">log</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instanceLogToFile-${instance.id}`)}" data-bridge-instance-field="instanceLogToFile">
              <option value="false" ${instance.instanceLogToFile !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceLogToFile === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceVarDiff")}
              <span class="kgw-command-option-title-text-r8e">var_diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instanceVarDiff-${instance.id}`)}" data-bridge-instance-field="instanceVarDiff">
              <option value="false" ${instance.instanceVarDiff !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiff === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceVarDiffStats")}
              <span class="kgw-command-option-title-text-r8e">var_stats</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instanceVarDiffStats-${instance.id}`)}" data-bridge-instance-field="instanceVarDiffStats">
              <option value="false" ${instance.instanceVarDiffStats !== "true" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiffStats === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceSharesPerMin")}
              <span class="kgw-command-option-title-text-r8e">shares/min</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instanceSharesPerMin-${instance.id}`)}" data-bridge-instance-field="instanceSharesPerMin" value="${instance.instanceSharesPerMin || ""}" placeholder="optional" ${kgwBridgeDifficultyInputAttrsR16C("instanceSharesPerMin")} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instancePow2Clamp")}
              <span class="kgw-command-option-title-text-r8e">pow2</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
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
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessRpcListen")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.rpcListen">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.rpcListen", "--rpclisten"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessRpcListen")}" type="text" value="127.0.0.1:${esc(net.kaspadPort)}">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessRpcListenBorsh")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.rpcListenBorsh">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.rpcListenBorsh", "--rpclisten-borsh"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessRpcListenBorsh")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessRpcListenJson")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.rpcListenJson">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.rpcListenJson", "--rpclisten-json"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
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
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessListen")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.listen">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.listen", "--listen"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessListen")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessAddPeer")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.addPeer">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.addPeer", "--addpeer"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessAddPeer")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessConnect")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.connect">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.connect", "--connect"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessConnect")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessDisableUpnp")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.disableUpnp">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.disableUpnp", "--disable-upnp"))}</span>
          </label>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessMaxInpeers")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.maxInpeers">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.maxInpeers", "--maxinpeers"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessMaxInpeers")}" type="number" min="0" step="1" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessOutpeers")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.outpeers">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.outpeers", "--outpeers"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
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
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessPerfMetricsIntervalSec")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.perfMetricsIntervalSec">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.perfMetricsIntervalSec", "--perf-metrics-interval-sec"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessPerfMetricsIntervalSec")}" type="number" min="1" step="1" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessLogLevel")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.logLevel">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.logLevel", "--loglevel"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessLogLevel")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessRamScale")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.ramScale">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.ramScale", "--ram-scale"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessRamScale")}" type="number" min="0.1" step="0.1" value="">
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="advanced" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessConfigfile")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.configfile">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.configfile", "--configfile"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
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
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessOverrideParamsFile")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.overrideParamsFile">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.overrideParamsFile", "--override-params-file"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
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
/* KGW_BRIDGE_FLAT_TWO_TABS_ULTRA_COMPACT_OWNER_R101N
 * Existing Bridge settings owner refinement.
 * Bridge top-level settings tabs are General / Advanced only.
 * General uses flat ultra-compact fields; Advanced keeps complex owners safely.
 */
function kgwBridgeFlatSectionFieldsR101N(html) {
  return String(html || "")
    .replace(/^\s*<div\s+class=["']bridge-v7-grid["']>\s*/i, "")
    .replace(/\s*<\/div>\s*$/i, "");
}

function kgwBridgeFlatGroupBodyR101N(sections) {
  return sections.map((body) => kgwBridgeFlatSectionFieldsR101N(body)).join("\n");
}

function renderSections(net) {
  /* KGW_BRIDGE_GROUPED_SETTINGS_TABS_OWNER_R101G */
  /* KGW_BRIDGE_GROUPED_SETTINGS_TABS_COMPACT_FIX_R101H */
  /* KGW_BRIDGE_FLAT_TWO_TABS_ULTRA_COMPACT_OWNER_R101N */
  /* KGW_BRIDGE_FOUR_TABS_ULTRA_COMPACT_FIX_R101O */
  /* KGW_BRIDGE_MERGE_ADVANCED_INTO_GENERAL_R101P
   * Advanced fields are merged into General because there is enough space.
   * Bridge internal tabs are General / In-Processor / Instances.
   */
  const groups = [
    ["general", "General"],
    ["inprocessor", "In-Processor"],
    ["instances", "Instances"]
  ];

  const tabs = groups.map(([key, label], index) =>
    `<button type="button" class="bridge-v7-section-tab bridge-v7-section-tab--grouped bridge-v7-section-tab--flat${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-section-tab="${key}">${label}</button>`
  ).join("");

  const generalFields = kgwBridgeFlatGroupBodyR101N([
    renderRuntime(net),
    renderLogging(net),
    renderDifficulty(net),
    renderPorts(net),
    ...(net.key === "mainnet" ? [] : [renderCpuMiner(net)])
  ]);

  const panels = `
    <section class="bridge-v7-section bridge-v7-section-group bridge-v7-section-group--flat active" data-net="${net.key}" data-bridge-section-panel="general">
      ${kgwBridgeDifficultyDatalistR16C()}
      <div class="bridge-v7-flat-eight-grid" data-bridge-flat-grid="general">${generalFields}</div>
    </section>
    <section class="bridge-v7-section bridge-v7-section-group bridge-v7-section-group--flat bridge-v7-section-group--standalone" data-net="${net.key}" data-bridge-section-panel="inprocessor" hidden>
      <div class="bridge-v7-flat-complex-block bridge-v7-flat-inprocess-block" data-bridge-flat-complex="inprocessor">
        ${renderInprocessNodeSettings(net)}
      </div>
    </section>
    <section id="${id(net.key, "instances")}" class="bridge-v7-section bridge-v7-section-group bridge-v7-section-group--flat bridge-v7-section-group--standalone" data-net="${net.key}" data-bridge-section-panel="instances" hidden>
      <div class="bridge-v7-flat-complex-block bridge-v7-flat-instances-block" data-bridge-flat-complex="instances">
        ${renderInstances(net.key)}
      </div>
    </section>`;

  return `
    <div class="bridge-v7-section-tabs bridge-v7-section-tabs--grouped bridge-v7-section-tabs--flat">${tabs}</div>
    <div class="bridge-v7-sections bridge-v7-sections--grouped bridge-v7-sections--flat bridge-v7-sections--four-tabs">${panels}</div>`;
}

/* KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
 * Default Bridge inner tab is Live Bridge Monitor.
 * Last selected inner tab is saved per network.
 */
function kgwBridgeInnerTabStorageKeyR101U(net) {
  return `kgw.bridge.innerTab.${String(net || "unknown")}`;
}

function kgwBridgeNormalizeInnerTabR101U(value) {
  return value === "settings" || value === "log" ? value : "log";
}

function kgwBridgeResolveInnerTabR101U(net) {
  try {
    return kgwBridgeNormalizeInnerTabR101U(localStorage.getItem(kgwBridgeInnerTabStorageKeyR101U(net)));
  } catch (_) {
    return "log";
  }
}

function kgwBridgeSaveInnerTabR101U(net, selected) {
  const normalized = kgwBridgeNormalizeInnerTabR101U(selected);
  try {
    localStorage.setItem(kgwBridgeInnerTabStorageKeyR101U(net), normalized);
  } catch (_) {}
  return normalized;
}

function renderNetworkPanel(net, index) {
  /* KGW_BRIDGE_LIVE_MONITOR_TAB_LABEL_ORDER_R101S */
  /* KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
   * Settings is no longer the default inner panel.
   * Default is Live Bridge Monitor unless a valid saved tab exists for this network.
   */
  const activeInnerTab = kgwBridgeResolveInnerTabR101U(net.key);
  const logActive = activeInnerTab === "log";
  const settingsActive = activeInnerTab === "settings";

  return `
    <div class="bridge-v7-network-panel${index === 0 ? " active" : ""}" data-bridge-network-panel="${net.key}" data-testid="kgw-bridge-panel-${net.key}"${index === 0 ? "" : " hidden"}>
      <section class="kgw-network-policy${net.experimental ? " is-experimental" : ""}" data-net="${net.key}" data-testid="kgw-bridge-policy-${net.key}">
        <div>
          <strong>${net.label}</strong>
          <span>${esc(kgwBridgeNetworkPolicyMessage(net.key))}</span>
        </div>
        <div class="kgw-network-policy-controls">
          <span id="${id(net.key, "policyStatus")}" class="kgw-network-policy-status">Stopped</span>
          <label>
            <input type="checkbox" data-bridge-network-enabled="${net.key}" data-testid="kgw-bridge-policy-enabled-${net.key}" data-net="${net.key}"${kgwBridgeNetworkEnabled(net.key) ? " checked" : ""}>
            Enabled
          </label>
        </div>
      </section>
      <div class="bridge-v7-inner-tabs">
        <button type="button" class="bridge-v7-inner-tab${logActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-tab="log" data-testid="kgw-bridge-live-monitor-${net.key}">Live Bridge Monitor</button>
        <button type="button" class="bridge-v7-inner-tab${settingsActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-tab="settings" data-testid="kgw-bridge-settings-${net.key}">Settings</button>
      </div>

      <div class="bridge-v7-inner-panel${settingsActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-panel="settings"${settingsActive ? "" : " hidden"}>
        <section class="bridge-v7-command">
          <div class="bridge-v7-command-title">Command Preview</div>
          <textarea id="${id(net.key, "commandPreview")}" readonly spellcheck="false" wrap="soft"></textarea>
          <button type="button" class="bridge-v7-copy" data-bridge-action="copy-command" data-net="${net.key}" title="Copy command">⧉</button>
        </section>

        <section class="bridge-v7-toolbar">
          <div class="bridge-v7-buttons">
            <button type="button" class="good" data-bridge-action="start" data-testid="kgw-bridge-start-${net.key}" data-net="${net.key}">Start</button>
            <button type="button" data-bridge-action="stop" data-testid="kgw-bridge-stop-${net.key}" data-net="${net.key}">Stop</button>
          </div>

          <div class="bridge-v7-status">
            <label><input id="${id(net.key, "launch")}" type="checkbox"> Auto-start</label>
            <label><input id="${id(net.key, "restart")}" type="checkbox" checked> Restart</label>
          </div>
          <div id="${id(net.key, "runtimeStatus")}" class="bridge-v7-runtime-status" role="status" aria-live="polite"></div>
          <div id="${id(net.key, "runtimeError")}" class="bridge-v7-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        ${renderSections(net)}

        <div class="settings-bottom-actions bridge-settings-bottom-actions">
        <button type="button" data-bridge-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-bridge-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-bridge-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        </div>

      </div>

      <div class="bridge-v7-inner-panel${logActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-panel="log" data-testid="kgw-bridge-live-panel-${net.key}"${logActive ? "" : " hidden"}>
        <div class="bridge-v7-log-toolbar">
          <button type="button" data-bridge-action="copy-log" data-testid="kgw-bridge-copy-log-${net.key}" data-net="${net.key}">Copy Log</button>
          <button type="button" data-bridge-action="clear-log" data-testid="kgw-bridge-clear-log-${net.key}" data-net="${net.key}">Clear Log</button>
        </div>
        <div id="${id(net.key, "logEmpty")}" class="bridge-v7-log-empty" data-bridge-log-empty="${net.key}">No child stdout/stderr received yet.</div>
        <pre id="${id(net.key, "logOutput")}" class="bridge-v7-log" data-testid="kgw-bridge-log-output-${net.key}"></pre>
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
      kgwBridgeRenderRawLogBufferV1(targetNet, "bridge", String(activeInstance[targetNet] || ""));
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
  kgwBridgeRenderRawLogBufferV1(net, "bridge", String(activeInstance[net] || ""));
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

  activeInstance[net] = instanceId;

  root.querySelectorAll("[data-bridge-instance-panel]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.bridgeInstancePanel === String(instanceId));
  });

  root.querySelectorAll("[data-bridge-action='select-instance']").forEach((button) => {
    button.classList.toggle("active", button.dataset.instanceId === String(instanceId));
  });
  kgwBridgeRenderRawLogBufferV1(net, "bridge", String(instanceId || ""));
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


/* KGW_BRIDGE_NETWORK_PORT_PROFILES_SOFT_POLICY_PATCH_R35B
 * Network port profiles are soft policy:
 * - Used for defaults/suggestions/auto-assignment only.
 * - Manual valid unused ports are accepted, even inside another network's recommended range.
 * - Real conflicts still block Start through R33.
 * - Out-of-profile ports are warning-only.
 */
const KGW_BRIDGE_PORT_PROFILES_R35B = Object.freeze({
  mainnet: Object.freeze({
    stratum: Object.freeze({ min: 5500, max: 5599, preferred: 5555, instanceStart: 5556 }),
    prom: Object.freeze({ min: 2100, max: 2199, preferred: 2112, instanceStart: 2113 }),
    dashboard: Object.freeze({ min: 3000, max: 3099, preferred: 3030 })
  }),
  testnet10: Object.freeze({
    stratum: Object.freeze({ min: 5600, max: 5699, preferred: 5655, instanceStart: 5656 }),
    prom: Object.freeze({ min: 2200, max: 2299, preferred: 2212, instanceStart: 2213 }),
    dashboard: Object.freeze({ min: 3100, max: 3199, preferred: 3130 })
  }),
  testnet12: Object.freeze({
    stratum: Object.freeze({ min: 5700, max: 5799, preferred: 5755, instanceStart: 5756 }),
    prom: Object.freeze({ min: 2300, max: 2399, preferred: 2312, instanceStart: 2313 }),
    dashboard: Object.freeze({ min: 3200, max: 3299, preferred: 3230 })
  })
});

/* KGW_BRIDGE_INSTANCE_EXTERNAL_PORT_RANGE_OWNER_R91
 * Existing Bridge port-profile owner refinement.
 * Instance stratum/prometheus port ranges are now derived from the current
 * bridge-level network settings outside the instance editor:
 * - stratum instances follow --stratum-port + 1 onward.
 * - prom instances follow --prom-port + 1 onward.
 * - each network remains isolated: mainnet, testnet10, testnet12.
 * - valid clearly manual out-of-range instance ports are preserved.
 */
function bridgeStaticPortProfileR91(net) {
  return KGW_BRIDGE_PORT_PROFILES_R35B[String(net || "")] || KGW_BRIDGE_PORT_PROFILES_R35B.mainnet;
}

function bridgeNormalizePortLiteralR91(value) {
  const text = String(value || "").trim();
  if (!text) return "";

  const hostMatch = text.match(/(?:^|[^0-9])(?:127\.0\.0\.1|0\.0\.0\.0|localhost)?:(\d{1,5})(?:$|[^0-9])/i);
  if (hostMatch) {
    const port = Number(hostMatch[1]);
    return Number.isInteger(port) && port >= 1 && port <= 65535 ? String(port) : "";
  }

  const plain = text.replace(/^:/, "");
  if (/^\d{1,5}$/.test(plain)) {
    const port = Number(plain);
    return Number.isInteger(port) && port >= 1 && port <= 65535 ? String(port) : "";
  }

  const extracted = typeof bridgeExtractPortsFromTextR5 === "function"
    ? bridgeExtractPortsFromTextR5(text)
    : [];

  return extracted && extracted.length ? String(extracted[0]) : "";
}

function bridgeExternalBasePortR91(net, kind, fallbackRange) {
  const profile = bridgeProfile(net) || {};
  const fieldByKind = {
    stratum: "stratumPort",
    prom: "promPort",
    dashboard: "webDashboardPort"
  };

  const profileFieldByKind = {
    stratum: "stratumPort",
    prom: "promPort",
    dashboard: "dashboardPort"
  };

  const fieldName = fieldByKind[kind] || "stratumPort";
  const profileFieldName = profileFieldByKind[kind] || fieldName;

  const current = typeof v === "function" ? v(net, fieldName) : "";
  const profileValue = profile && profile[profileFieldName] ? profile[profileFieldName] : "";

  return bridgeNormalizePortLiteralR91(current) ||
    bridgeNormalizePortLiteralR91(profileValue) ||
    bridgeNormalizePortLiteralR91(fallbackRange && fallbackRange.preferred) ||
    bridgeNormalizePortLiteralR91(fallbackRange && fallbackRange.min) ||
    "";
}

function bridgeRangeFromExternalBaseR91(basePort, fallbackRange) {
  const fallback = fallbackRange || {};
  const base = Number(bridgeNormalizePortLiteralR91(basePort));

  if (!Number.isInteger(base) || base < 1 || base > 65535) {
    return { ...fallback };
  }

  if (Number(fallback.preferred) === base) {
    return {
      ...fallback,
      externalBase: String(base),
      externalOwner: "static-profile-matching-current-bridge-setting",
      dynamicFromExternalSetting: false
    };
  }

  const start = base < 65535 ? base + 1 : base;
  const max = Math.min(65535, start + 98);

  return {
    ...fallback,
    min: start,
    max,
    preferred: base,
    instanceStart: start,
    externalBase: String(base),
    externalOwner: "bridge-level-port-setting",
    dynamicFromExternalSetting: true
  };
}

function bridgePortProfileR35B(net) {
  const staticProfile = bridgeStaticPortProfileR91(net);

  return {
    stratum: Object.freeze(bridgeRangeFromExternalBaseR91(
      bridgeExternalBasePortR91(net, "stratum", staticProfile.stratum),
      staticProfile.stratum
    )),
    prom: Object.freeze(bridgeRangeFromExternalBaseR91(
      bridgeExternalBasePortR91(net, "prom", staticProfile.prom),
      staticProfile.prom
    )),
    dashboard: Object.freeze(bridgeRangeFromExternalBaseR91(
      bridgeExternalBasePortR91(net, "dashboard", staticProfile.dashboard),
      staticProfile.dashboard
    ))
  };
}

function bridgeNormalizePortSoftR35B(value) {
  const normalized = String(value || "").trim().replace(/^:/, "");
  if (!/^\d{1,5}$/.test(normalized)) return "";
  const port = Number(normalized);
  if (!Number.isInteger(port) || port < 1 || port > 65535) return "";
  return String(port);
}

function bridgePortInRangeR35B(port, range) {
  const n = Number(bridgeNormalizePortSoftR35B(port));
  return Boolean(range && Number.isInteger(n) && n >= Number(range.min) && n <= Number(range.max));
}

function bridgeFindUnusedPortInRangeR35B(range, usedPorts, fallbackStart) {
  const start = Number(bridgeNormalizePortSoftR35B(fallbackStart || (range && range.instanceStart) || (range && range.preferred) || (range && range.min) || 1));
  const min = Number(range && range.min || 1);
  const max = Number(range && range.max || 65535);
  const begin = Math.max(min, Math.min(max, Number.isInteger(start) ? start : min));

  for (let port = begin; port <= max; port += 1) {
    if (!usedPorts.has(String(port))) {
      usedPorts.add(String(port));
      return String(port);
    }
  }

  for (let port = min; port < begin; port += 1) {
    if (!usedPorts.has(String(port))) {
      usedPorts.add(String(port));
      return String(port);
    }
  }

  return "";
}

function bridgeFindRecommendedOrNearestUnusedPortR35B(net, kind, usedPorts, fallbackStart) {
  const profile = bridgePortProfileR35B(net);
  const range = profile[kind] || profile.stratum;
  const inside = bridgeFindUnusedPortInRangeR35B(range, usedPorts, fallbackStart || range.instanceStart || range.preferred || range.min);
  if (inside) return inside;

  return bridgeFindNearestUnusedPortR9(fallbackStart || range.preferred || range.min || 1, usedPorts);
}

function bridgeClassifyPortProfileR35B(net, kind, port) {
  const normalized = bridgeNormalizePortSoftR35B(port);
  if (!normalized) {
    return { ok: false, warning: true, invalid: true, message: "invalid port" };
  }

  const profile = bridgePortProfileR35B(net);
  const range = profile[kind] || profile.stratum;

  if (bridgePortInRangeR35B(normalized, range)) {
    return { ok: true, warning: false, invalid: false, port: normalized, message: "" };
  }

  return {
    ok: true,
    warning: true,
    invalid: false,
    port: normalized,
    message: kind + " port " + normalized + " is outside recommended " + String(net || "") + " range " + range.min + "-" + range.max + ". Accepted if unused."
  };
}

function bridgeCollectPortProfileWarningsR35B(net) {
  const warnings = [];
  const profile = bridgeProfile(net) || {};
  const activeNet = String(net || "");

  const bridgeStratum = bridgeNormalizePortSoftR35B(v(activeNet, "stratumPort") || profile.stratumPort || "");
  const bridgeProm = bridgeNormalizePortSoftR35B(v(activeNet, "promPort") || profile.promPort || "");
  const bridgeDashboard = bridgeNormalizePortSoftR35B(v(activeNet, "webDashboardPort") || profile.dashboardPort || "");

  for (const item of [
    { kind: "stratum", port: bridgeStratum, owner: "bridge-stratum" },
    { kind: "prom", port: bridgeProm, owner: "bridge-prometheus" },
    { kind: "dashboard", port: bridgeDashboard, owner: "bridge-dashboard" }
  ]) {
    if (!item.port) continue;
    const status = bridgeClassifyPortProfileR35B(activeNet, item.kind, item.port);
    if (status.warning) warnings.push({ ...status, owner: item.owner, kind: item.kind });
  }

  const list = Array.isArray(bridgeInstances[activeNet]) ? bridgeInstances[activeNet] : [];
  for (const instance of list) {
    const instanceId = instance && instance.id;
    const instancePort = bridgeNormalizePortSoftR35B(bridgeInstanceReadSupplement(activeNet, instanceId, "instancePort", instance && instance.instancePort));
    const instanceProm = bridgeNormalizePortSoftR35B(bridgeInstanceReadSupplement(activeNet, instanceId, "instanceProm", instance && instance.instanceProm));

    if (instancePort) {
      const status = bridgeClassifyPortProfileR35B(activeNet, "stratum", instancePort);
      if (status.warning) warnings.push({ ...status, owner: "instance:" + String(instanceId || ""), kind: "stratum" });
    }

    if (instanceProm) {
      const status = bridgeClassifyPortProfileR35B(activeNet, "prom", instanceProm);
      if (status.warning) warnings.push({ ...status, owner: "instance:" + String(instanceId || ""), kind: "prom" });
    }
  }

  return warnings;
}

function bridgePortProfileWarningMessageR35B(net) {
  const warnings = bridgeCollectPortProfileWarningsR35B(net);
  if (!warnings.length) return "";
  return warnings.map((item) => item.owner + " " + item.message).join("; ");
}

function bridgeTracePortProfileR35B(net, phase, details) {
  try {
    kgwBridgeSmallOwnerTraceR44D(net, "port-profile", phase, {
      patch: "R35B",
      owner: "bridge-network-port-profile-soft-policy",
      policy: "manual-valid-unused-ports-accepted-even-inside-other-network-range",
      details: details && typeof details === "object" ? details : {}
    });
  } catch (_) {}
}


/* KGW_BRIDGE_PORT_CONFLICT_AUTOFIX_PATCH_R37
 * User-triggered Auto Fix for actual conflicting ports only.
 * Soft policy:
 * - No silent migration on load.
 * - Valid non-conflicting manual ports stay unchanged.
 * - Ports inside another network profile are accepted if unused.
 * - Prefer keeping bridge-level/default ports.
 * - Prefer changing conflicting instance ports.
 * - R33 remains the Start blocker.
 */
function bridgeTracePortAutofixR37(net, phase, details) {
  try {
    kgwBridgeSmallOwnerTraceR44D(net, "port-autofix", phase, {
      patch: "R37",
      owner: "bridge-existing-port-conflict-owner-autofix",
      policy: "user-triggered-only-change-actual-conflicts",
      details: details && typeof details === "object" ? details : {}
    });
  } catch (_) {}
}

function bridgePortOwnerPriorityR37(owner) {
  const role = String(owner && owner.role || "");
  const source = String(owner && owner.owner || "");

  if (role.startsWith("default-")) return 10;
  if (role.startsWith("bridge-")) return 20;
  if (source.startsWith("BRIDGE_NETWORKS.")) return 25;
  if (role.startsWith("inprocess-")) return 35;
  if (role === "instance") return 80;
  return 50;
}

function bridgeInstanceIdFromOwnerR37(owner) {
  const raw = String(owner && owner.owner || "");
  return raw.startsWith("instance:") ? raw.slice("instance:".length) : "";
}

function bridgeNormalizePortR37(value) {
  return String(value || "").trim().replace(/^:/, "");
}

function bridgeInstancePortKindForConflictR37(instance, conflictPort) {
  const oldPort = bridgeNormalizePortR37(conflictPort);
  if (bridgeNormalizePortR37(instance && instance.instancePort) === oldPort) return "stratum";
  if (bridgeNormalizePortR37(instance && instance.instanceProm) === oldPort) return "prom";
  return "";
}

function bridgeAutofixChangeKeyR37(change) {
  return [
    String(change && change.net || ""),
    String(change && change.instanceId || ""),
    String(change && change.kind || ""),
    String(change && change.oldPort || "")
  ].join(":");
}


/* KGW_BRIDGE_AUTOFIX_GLOBAL_USED_PORTS_PATCH_R45
 * Strengthens existing R37 Auto Fix:
 * - De-duplicates repeated conflict owners.
 * - Uses global used ports across all bridge networks when selecting replacements.
 * - If active network instance conflicts with another network instance, changes active network instance first.
 * - Applies multiple passes so a replacement cannot leave a new conflict behind.
 * Port policy remains soft: manual valid unused ports are accepted anywhere.
 */
function bridgeOwnerKeyR45(owner) {
  return [
    String(owner && owner.net || ""),
    String(owner && owner.role || ""),
    String(owner && owner.owner || "")
  ].join("|");
}

function bridgeUniqueConflictOwnersR45(owners) {
  const out = [];
  const seen = new Set();
  for (const owner of Array.isArray(owners) ? owners : []) {
    const key = bridgeOwnerKeyR45(owner);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    out.push(owner);
  }
  return out;
}

function bridgeConfiguredPortRecordsR45() {
  const records = [];
  let collected = [];

  try {
    collected = bridgeCollectConfiguredPortsR5();
  } catch (_) {
    collected = [];
  }

  if (Array.isArray(collected)) {
    for (const item of collected) {
      if (!item) continue;

      if (Array.isArray(item.owners)) {
        for (const owner of item.owners) {
          records.push({
            port: String(item.port || ""),
            net: String(owner && owner.net || ""),
            role: String(owner && owner.role || ""),
            owner: String(owner && owner.owner || "")
          });
        }
      } else {
        records.push({
          port: String(item.port || ""),
          net: String(item.net || ""),
          role: String(item.role || ""),
          owner: String(item.owner || "")
        });
      }
    }
  } else if (collected && typeof collected === "object") {
    for (const [port, owners] of Object.entries(collected)) {
      if (Array.isArray(owners)) {
        for (const owner of owners) {
          records.push({
            port: String(port || ""),
            net: String(owner && owner.net || ""),
            role: String(owner && owner.role || ""),
            owner: String(owner && owner.owner || "")
          });
        }
      }
    }
  }

  return records.filter((item) => item.port);
}

function bridgeGlobalUsedPortsForAutofixR45(change, plannedUsed) {
  const used = new Set();
  const targetNet = String(change && change.net || "");
  const targetInstanceOwner = "instance:" + String(change && change.instanceId || "");
  const oldPort = String(change && change.oldPort || "");

  for (const item of bridgeConfiguredPortRecordsR45()) {
    const port = String(item && item.port || "").trim().replace(/^:/, "");
    if (!port) continue;

    const isTargetOldPort =
      String(item.net || "") === targetNet &&
      String(item.owner || "") === targetInstanceOwner &&
      port === oldPort;

    if (!isTargetOldPort) {
      used.add(port);
    }
  }

  for (const item of plannedUsed || []) {
    const port = String(item || "").trim().replace(/^:/, "");
    if (port) used.add(port);
  }

  return used;
}

function bridgeOwnersToAutofixR45(activeNet, owners) {
  const normalizedActive = String(activeNet || "");
  const uniqueOwners = bridgeUniqueConflictOwnersR45(owners);
  const instanceOwners = uniqueOwners.filter((owner) => String(owner && owner.role || "") === "instance");
  const protectedOwners = uniqueOwners.filter((owner) => String(owner && owner.role || "") !== "instance");

  if (instanceOwners.length === 0) return [];

  if (protectedOwners.length > 0) {
    return instanceOwners;
  }

  const activeInstanceOwners = instanceOwners.filter((owner) => String(owner && owner.net || "") === normalizedActive);
  if (activeInstanceOwners.length > 0) {
    return activeInstanceOwners;
  }

  return instanceOwners.slice(1);
}

function bridgeRefreshAutofixTouchedNetsR45(touchedNets, activeNet, reason) {
  for (const touchedNet of touchedNets) {
    bridgeRefreshInstances(touchedNet);
    updateCommand(touchedNet);
    bridgeValidateAndApplyPortConflictStateR33(touchedNet, reason || "r45-autofix");
  }

  if (activeNet && !touchedNets.has(activeNet)) {
    updateCommand(activeNet);
    bridgeValidateAndApplyPortConflictStateR33(activeNet, reason || "r45-autofix-active");
  }

  bridgeValidateAllPortConflictStatesR33(reason || "r45-autofix-all");
  bridgeRefreshPortAutofixButtonsR37(reason || "r45-autofix-buttons");
}


function bridgePlanPortAutofixR37(activeNet) {
  const validation = bridgeValidatePortConflictsR5(activeNet);
  const changes = [];
  const seen = new Set();

  if (!validation || validation.ok || !Array.isArray(validation.conflicts)) {
    return { validation, changes };
  }

  for (const conflict of validation.conflicts) {
    const owners = bridgeUniqueConflictOwnersR45(conflict.owners);
    if (owners.length < 2) continue;

    const ownersToChange = bridgeOwnersToAutofixR45(activeNet, owners);

    for (const owner of ownersToChange) {
      const net = String(owner.net || "");
      const instanceId = bridgeInstanceIdFromOwnerR37(owner);
      if (!net || !instanceId) continue;

      const list = Array.isArray(bridgeInstances[net]) ? bridgeInstances[net] : [];
      const instance = list.find((item) => String(item && item.id) === String(instanceId));
      if (!instance) continue;

      const kind = bridgeInstancePortKindForConflictR37(instance, conflict.port);
      if (!kind) continue;

      const change = {
        net,
        instanceId,
        kind,
        oldPort: String(conflict.port || ""),
        changedOwner: {
          net: String(owner && owner.net || ""),
          role: String(owner && owner.role || ""),
          owner: String(owner && owner.owner || "")
        }
      };

      const key = bridgeAutofixChangeKeyR37(change);
      if (seen.has(key)) continue;
      seen.add(key);
      changes.push(change);
    }
  }

  return { validation, changes };
}

function bridgeChooseReplacementPortR37(change, plannedUsed) {
  const net = String(change && change.net || "");
  const kind = String(change && change.kind || "stratum");
  const instanceId = String(change && change.instanceId || "");
  const profile = bridgePortProfileR35B(net);
  const range = kind === "prom" ? profile.prom : profile.stratum;
  const used = bridgeGlobalUsedPortsForAutofixR45(change, plannedUsed);

  const list = Array.isArray(bridgeInstances[net]) ? bridgeInstances[net] : [];
  const instance = list.find((row) => String(row && row.id) === instanceId);

  if (instance) {
    if (kind === "prom") {
      const other = bridgeNormalizePortR37(instance.instancePort);
      if (other && other !== String(change.oldPort || "")) used.add(other);
    } else {
      const other = bridgeNormalizePortR37(instance.instanceProm);
      if (other && other !== String(change.oldPort || "")) used.add(other);
    }
  }

  return bridgeFindRecommendedOrNearestUnusedPortR35B(
    net,
    kind === "prom" ? "prom" : "stratum",
    used,
    range.instanceStart || range.preferred || range.min
  );
}

function bridgeWriteInstancePortR37(change, newPort) {
  const net = String(change && change.net || "");
  const instanceId = String(change && change.instanceId || "");
  const kind = String(change && change.kind || "");
  const list = Array.isArray(bridgeInstances[net]) ? bridgeInstances[net] : [];
  const instance = list.find((row) => String(row && row.id) === instanceId);
  if (!instance) return false;

  const fieldName = kind === "prom" ? "instanceProm" : "instancePort";
  instance[fieldName] = String(newPort || "");

  const field = byId(id(net, fieldName + "-" + instanceId));
  if (field) {
    field.value = String(newPort || "");
    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
  }

  return true;
}

function bridgeApplyPortAutofixR37(activeNet) {
  const net = String(activeNet || "");
  const allChanged = [];
  const maxPasses = 8;

  bridgeTracePortAutofixR37(net, "r37-port-autofix-begin", {
    patch2: "R45",
    mode: "iterative-global-used-ports",
    activeNet: net
  });

  for (let pass = 1; pass <= maxPasses; pass += 1) {
    const plan = bridgePlanPortAutofixR37(net);

    bridgeTracePortAutofixR37(net, "r45-port-autofix-pass-plan", {
      pass,
      conflictCount: plan.validation && Array.isArray(plan.validation.conflicts) ? plan.validation.conflicts.length : 0,
      plannedChangeCount: plan.changes.length
    });

    if (!plan.changes.length) {
      if (pass === 1) {
        bridgeTracePortAutofixR37(net, "r37-port-autofix-noop", {
          reason: plan.validation && plan.validation.ok ? "no-conflicts" : "no-instance-conflicts-can-be-autofixed",
          patch2: "R45"
        });
      }
      break;
    }

    const passChanged = [];
    const plannedUsed = new Set();

    for (const change of plan.changes) {
      const newPort = bridgeChooseReplacementPortR37(change, plannedUsed);
      if (!newPort) continue;

      const ok = bridgeWriteInstancePortR37(change, newPort);
      if (!ok) continue;

      plannedUsed.add(String(newPort));

      const applied = {
        ...change,
        newPort: String(newPort || ""),
        pass
      };

      passChanged.push(applied);
      allChanged.push(applied);

      bridgeTracePortAutofixR37(change.net, "r37-port-autofix-change", applied);
    }

    if (!passChanged.length) break;

    const touchedNets = new Set(passChanged.map((item) => String(item.net || "")).filter(Boolean));
    bridgeRefreshAutofixTouchedNetsR45(touchedNets, net, "r45-autofix-pass-" + String(pass));

    const after = bridgeValidatePortConflictsR5(net);
    if (after && after.ok) {
      break;
    }
  }

  const finalValidation = bridgeValidatePortConflictsR5(net);
  const touchedNets = new Set(allChanged.map((item) => String(item.net || "")).filter(Boolean));
  bridgeRefreshAutofixTouchedNetsR45(touchedNets, net, "r45-autofix-final");

  bridgeTracePortAutofixR37(net, "r37-port-autofix-complete", {
    patch2: "R45",
    changedCount: allChanged.length,
    finalOk: Boolean(finalValidation && finalValidation.ok),
    finalConflictCount: finalValidation && Array.isArray(finalValidation.conflicts) ? finalValidation.conflicts.length : 0,
    changes: allChanged.slice(0, 80)
  });

  kgwBridgeSetRuntimeActivityV1(
    net,
    kgwBridgeAutoFixTextR54D3("changedPrefix") + " " + String(allChanged.length) + " conflicting instance port(s)." +
      (finalValidation && finalValidation.ok ? " Conflicts cleared." : " Some conflicts remain.")
  );

  return { changed: allChanged.length, changes: allChanged, finalOk: Boolean(finalValidation && finalValidation.ok) };
}

function bridgeAutofixButtonsR37() {
  const root = document.getElementById("kaspa-bridge");
  if (!root) return [];
  return Array.from(root.querySelectorAll('[data-bridge-action="auto-fix-ports-r37"]'));
}

function bridgeRefreshPortAutofixButtonsR37(reason) {
  for (const button of bridgeAutofixButtonsR37()) {
    const net = String(button.dataset.net || "");
    const validation = bridgeValidatePortConflictsR5(net);
    const plan = bridgePlanPortAutofixR37(net);
    const enabled = Boolean(validation && !validation.ok && plan.changes.length);

    button.disabled = !enabled;
    button.classList.toggle("kgw-port-autofix-ready-r37", enabled);
    button.dataset.kgwPortAutofixReadyR37 = enabled ? "true" : "false";
    button.title = enabled
      ? "Auto-fix conflicting instance ports only. Valid non-conflicting manual ports stay unchanged."
      : "No auto-fixable instance port conflicts for this network.";

    if (enabled) {
      button.textContent = kgwBridgeAutoFixTextR54D3("conflictingButton");
    } else {
      button.textContent = kgwBridgeAutoFixTextR54D3("button");
    }
  }
}

function bridgeSchedulePortAutofixRefreshR37(net, reason) {
  window.clearTimeout(window.__kgwBridgePortAutofixRefreshTimerR37);
  window.__kgwBridgePortAutofixRefreshTimerR37 = window.setTimeout(() => {
    if (net) {
      bridgeValidateAndApplyPortConflictStateR33(net, "r37-refresh-" + String(reason || ""));
    } else {
      bridgeValidateAllPortConflictStatesR33("r37-refresh-all-" + String(reason || ""));
    }
    bridgeRefreshPortAutofixButtonsR37(reason || "scheduled");
  }, 80);
}

/* KGW_BRIDGE_AUTOFIX_I18N_PATCH_R54D3
 * Local i18n wrapper for existing Bridge Auto Fix labels/log prefix.
 */
/* KGW_BRIDGE_AUTOFIX_I18N_PATCH_R54D3
 * Existing Bridge Auto Fix i18n owner.
 * KGW_BRIDGE_AUTOFIX_I18N_OWNER_SAFE_FALLBACK_R112D:
 * Never return a raw bridge.autofixPorts.* key to the UI.
 */
function kgwBridgeAutoFixTextR54D3(key) {
  const map = {
    button: "bridge.autofixPorts.button",
    conflictingButton: "bridge.autofixPorts.conflictingButton",
    fixingButton: "bridge.autofixPorts.fixingButton",
    fixedButton: "bridge.autofixPorts.fixedButton",
    failedButton: "bridge.autofixPorts.failedButton",
    disabledButton: "bridge.autofixPorts.disabledButton",
    title: "bridge.autofixPorts.title",
    changedPrefix: "bridge.autofixPorts.changedPrefix"
  };

  const fallback = {
    button: "Auto Fix Ports",
    conflictingButton: "Auto Fix Ports",
    fixingButton: "Fixing Ports...",
    fixedButton: "Ports Fixed",
    failedButton: "Auto Fix Failed",
    disabledButton: "Auto Fix Ports",
    title: "Auto Fix Ports",
    changedPrefix: "Changed ports"
  };

  const i18nKey = map[key] || map.button;
  const fallbackText = fallback[key] || fallback.button;

  const cleanTranslated = (value) => {
    const text = String(value || "").trim();
    if (!text) return "";
    if (text === i18nKey) return "";
    if (/^bridge\.autofixPorts\./.test(text)) return "";
    return text;
  };

  try {
    if (typeof t === "function") {
      const translated = cleanTranslated(t(i18nKey));
      if (translated) return translated;
    }
  } catch (_) {}

  try {
    if (typeof translate === "function") {
      const translated = cleanTranslated(translate(i18nKey));
      if (translated) return translated;
    }
  } catch (_) {}

  return fallbackText;
}

function bridgeInstallPortAutofixButtonR37(root) {
  /* KGW_BRIDGE_AUTOFIX_BUTTON_NEXT_TO_STOP_PATCH_R44
   * Layout-only replacement:
   * - Removes the full-width R40 Auto Fix banner.
   * - Moves the same R37 Auto Fix button beside Stop in the runtime controls row.
   * - Keeps R37 action/click handler and R33/R35B/R42 port policy unchanged.
   */
  if (!root) return;

  for (const oldHost of Array.from(root.querySelectorAll('[data-kgw-bridge-port-autofix-host-r40="true"]'))) {
    oldHost.remove();
  }

  for (const profile of BRIDGE_NETWORKS) {
    const net = String(profile && profile.key || "");
    if (!net) continue;

    const selector = '[data-bridge-action="auto-fix-ports-r37"][data-net="' + net + '"]';
    const existingButtons = Array.from(root.querySelectorAll(selector));

    let button = existingButtons.find((item) => item.dataset.kgwBridgePortAutofixNextToStopR44 === "true") || null;

    for (const item of existingButtons) {
      if (item !== button) item.remove();
    }

    if (!button) {
      button = document.createElement("button");
      button.type = "button";
      button.dataset.bridgeAction = "auto-fix-ports-r37";
      button.dataset.net = net;
      button.dataset.kgwBridgePortAutofixNextToStopR44 = "true";
    }

    button.className = "kgw-bridge-port-autofix-next-to-stop-r44";
    button.textContent = kgwBridgeAutoFixTextR54D3("button");
    button.title = kgwBridgeAutoFixTextR54D3("title");
    button.style.display = "inline-flex";
    button.style.alignItems = "center";
    button.style.justifyContent = "center";
    button.style.width = "150px";
    button.style.minWidth = "150px";
    button.style.maxWidth = "170px";
    button.style.height = "29px";
    button.style.marginLeft = "8px";
    button.style.padding = "0 10px";
    button.style.whiteSpace = "nowrap";
    button.style.overflow = "hidden";
    button.style.textOverflow = "ellipsis";
    button.style.border = "1px solid rgba(148, 163, 184, 0.7)";
    button.style.borderRadius = "0";
    button.style.background = "rgba(71, 85, 105, 0.95)";
    button.style.color = "#fff";
    button.style.fontSize = "12px";
    button.style.cursor = "pointer";

    const stopButton =
      byId(id(net, "stop")) ||
      root.querySelector('[data-bridge-action="stop"][data-net="' + net + '"]') ||
      root.querySelector('[data-net="' + net + '"][id$="-stop"]');

    const startButton =
      byId(id(net, "start")) ||
      root.querySelector('[data-bridge-action="start"][data-net="' + net + '"]') ||
      root.querySelector('[data-net="' + net + '"][id$="-start"]');

    const anchor = stopButton || startButton;

    if (anchor && anchor.parentNode) {
      if (button.parentNode !== anchor.parentNode) {
        anchor.parentNode.insertBefore(button, stopButton ? stopButton.nextSibling : anchor.nextSibling);
      } else if (stopButton && button.previousSibling !== stopButton) {
        anchor.parentNode.insertBefore(button, stopButton.nextSibling);
      }
    } else if (!button.parentNode) {
      root.appendChild(button);
    }
  }

  bridgeRefreshPortAutofixButtonsR37("r44-next-to-stop-install");
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

const KGW_BRIDGE_RAW_LOG_BUFFER_LIMIT_V1 = 4096;
const KGW_BRIDGE_RAW_LOG_BUFFERS_V1 = new Map();

function kgwBridgeActiveRawLogInstanceIdV1(net) {
  bridgeEnsureInstanceState(net);
  return String(activeInstance?.[net] || (bridgeInstances?.[net]?.[0] && bridgeInstances[net][0].id) || "");
}

function kgwBridgeRawLogBufferKeyV1(net, role = "bridge", instanceId = "") {
  // Official Bridge output is process-wide for one role/network owner. The
  // upstream logger does not attribute records to individual listeners.
  void instanceId;
  return [
    String(net || "").trim().toLowerCase(),
    String(role || "bridge").trim().toLowerCase()
  ].join(":");
}

function kgwBridgeRawLogBufferV1(net, role = "bridge", instanceId = "") {
  const key = kgwBridgeRawLogBufferKeyV1(net, role, instanceId);
  if (!KGW_BRIDGE_RAW_LOG_BUFFERS_V1.has(key)) {
    KGW_BRIDGE_RAW_LOG_BUFFERS_V1.set(key, { records: new Map() });
  }
  return KGW_BRIDGE_RAW_LOG_BUFFERS_V1.get(key);
}

function kgwBridgeNormalizeRawLogEntryV1(entry, expectedNet, expectedRole = "bridge", expectedInstanceId = "") {
  void expectedInstanceId;
  if (!entry || typeof entry !== "object") return null;

  const rawTextValue = entry.rawText ?? entry.raw_text ?? entry.line;
  if (rawTextValue === undefined || rawTextValue === null) return null;
  const sequence = Number(entry.sequence);
  if (!Number.isSafeInteger(sequence) || sequence < 0) return null;

  const network = String(entry.network || expectedNet || "").trim().toLowerCase();
  const runtimeRole = String(entry.runtimeRole || entry.runtime_role || expectedRole || "bridge").trim().toLowerCase();
  const bridgeInstanceId = String(entry.bridgeInstanceId ?? entry.bridge_instance_id ?? "").trim();
  const stream = String(entry.stream || "").trim().toLowerCase();

  if (network !== String(expectedNet || "").trim().toLowerCase()) return null;
  if (runtimeRole !== String(expectedRole || "bridge").trim().toLowerCase()) return null;
  // Official Bridge logging is process-wide; upstream exposes no structural
  // listener identifier for a record. Never filter official text by a UI instance.
  if (stream !== "stdout" && stream !== "stderr") return null;

  return Object.freeze({
    sequence,
    network,
    runtimeRole,
    bridgeInstanceId,
    stream,
    receivedMs: Number(entry.receivedMs ?? entry.received_ms ?? 0) || 0,
    rawText: String(rawTextValue)
  });
}

function kgwBridgeTrimRawLogBufferV1(buffer) {
  const ordered = Array.from(buffer.records.keys()).sort((a, b) => a - b);
  while (ordered.length > KGW_BRIDGE_RAW_LOG_BUFFER_LIMIT_V1) {
    const sequence = ordered.shift();
    buffer.records.delete(sequence);
  }
}

function kgwBridgeVisibleRawLogTextV1(net, role = "bridge", instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  const buffer = kgwBridgeRawLogBufferV1(net, role, instanceId);
  return Array.from(buffer.records.values())
    .sort((a, b) => a.sequence - b.sequence)
    .map((entry) => entry.rawText)
    .join("\n");
}

function kgwBridgeLogEmptyStateV1(net) {
  return document.getElementById("bridge-" + net + "-logEmpty");
}

function kgwBridgeRenderRawLogBufferV1(net, role = "bridge", instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  const out = byId(id(net, "logOutput"));
  if (!out) return false;

  const text = kgwBridgeVisibleRawLogTextV1(net, role, instanceId);
  out.textContent = text;

  const empty = kgwBridgeLogEmptyStateV1(net);
  if (empty) empty.hidden = text.length > 0;

  if (kgwBridgeLogAutoScrollEnabledR27(net)) out.scrollTop = out.scrollHeight;
  return true;
}

function kgwBridgeApplyRuntimeLogReportV1(net, role, report, instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  const entries = Array.isArray(report?.entries) ? report.entries : [];
  const buffer = kgwBridgeRawLogBufferV1(net, role, instanceId);
  let accepted = 0;

  for (const entry of entries) {
    const normalized = kgwBridgeNormalizeRawLogEntryV1(entry, net, role, instanceId);
    if (!normalized || buffer.records.has(normalized.sequence)) continue;
    buffer.records.set(normalized.sequence, normalized);
    accepted += 1;
  }

  if (accepted > 0) {
    kgwBridgeTrimRawLogBufferV1(buffer);
  }

  kgwBridgeRenderRawLogBufferV1(net, role, instanceId);
  return accepted;
}

function kgwBridgeClearRawLogBufferV1(net, role = "bridge", instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  kgwBridgeRawLogBufferV1(net, role, instanceId).records.clear();
  kgwBridgeRenderRawLogBufferV1(net, role, instanceId);
}

async function kgwBridgeDispatchRuntimeLogClearV1(net, role = "bridge") {
  const invoke = getTauriInvoke();
  if (!invoke) return null;
  return await invokeWithTimeout(invoke, "kgw_kgw_runtime_clear_logs_v1", buildApplyPayload(net, "kgw_kgw_runtime_clear_logs_v1"), KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS);
}

function appendLog(net, message) {
  // Raw monitor text is driven by typed runtime log reports. This legacy hook is
  // intentionally inert so bridge status strings cannot become fabricated raw lines.
  void net;
  void message;
}
// KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D_HELPER
function bridgeInprocessAddKaspadValueArgV12D(lines, flag, value) {
  const clean = String(value || "").trim();
  if (!clean) return;
  lines.push(`${flag}=${clean}`);
}


// KGW_BRIDGE_INPROCESS_SERIALIZATION_CHECKBOX_R13B
function bridgeInprocessAddKaspadValueArgR13B(lines, net, name, flag, value) {
  if (!kgwBridgeCommandShouldIncludeR7(net, name)) return;
  bridgeInprocessAddKaspadValueArgV12D(lines, flag, value);
}

function bridgeInprocessAddKaspadFlagR13B(lines, net, name, flag) {
  if (!kgwBridgeCommandShouldIncludeR7(net, name)) return;
  if (c(net, name)) lines.push(flag);
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

    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessRpcListen", "--rpclisten", v(net, "inprocessRpcListen") || `127.0.0.1:${profile.kaspadPort}`);
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessRpcListenBorsh", "--rpclisten-borsh", v(net, "inprocessRpcListenBorsh"));
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessRpcListenJson", "--rpclisten-json", v(net, "inprocessRpcListenJson"));

    bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessUnsafeRpc", "--unsaferpc");
    bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessUtxoIndex", "--utxoindex");
    bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessArchival", "--archival");

    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessListen", "--listen", v(net, "inprocessListen"));
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessAddPeer", "--addpeer", v(net, "inprocessAddPeer"));
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessConnect", "--connect", v(net, "inprocessConnect"));

    bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessDisableUpnp", "--disable-upnp");

    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessMaxInpeers", "--maxinpeers", v(net, "inprocessMaxInpeers"));
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessOutpeers", "--outpeers", v(net, "inprocessOutpeers"));

    bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessPerfMetrics", "--perf-metrics");
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessPerfMetricsIntervalSec", "--perf-metrics-interval-sec", v(net, "inprocessPerfMetricsIntervalSec"));
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessLogLevel", "--loglevel", v(net, "inprocessLogLevel"));
    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessRamScale", "--ram-scale", v(net, "inprocessRamScale"));

    bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessConfigfile", "--configfile", v(net, "inprocessConfigfile"));
    bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessYes", "--yes");

    if (net !== "mainnet") {
      bridgeInprocessAddKaspadValueArgR13B(kaspadArgs, net, "inprocessOverrideParamsFile", "--override-params-file", v(net, "inprocessOverrideParamsFile"));
      bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessDevnet", "--devnet");
      bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessSimnet", "--simnet");
      bridgeInprocessAddKaspadFlagR13B(kaspadArgs, net, "inprocessEnableUnsyncedMining", "--enable-unsynced-mining");
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
    if (instanceDefinition && kgwBridgeInstanceCommandShouldIncludeR13B(net, instance.id, "instance")) {
      lines.push(`--instance=${instanceDefinition}`);
    } // KGW_BRIDGE_INSTANCE_WHOLE_ARG_CHECKBOX_R13B
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
    bridgeReassignInstancePortsFromExternalRangeR91(net, "update-command");
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

    const profileWarning = bridgePortProfileWarningMessageR35B(net);

    if (allDuplicatePorts.length || !portValidation.ok) {
      preview.dataset.kgwBridgeCommandWarning = portValidation.message || `duplicate ports: ${allDuplicatePorts.join(",")}`;
      preview.classList.add("bridge-v7-command-warning");
    } else if (profileWarning) {
      preview.dataset.kgwBridgeCommandWarning = profileWarning;
      preview.classList.add("bridge-v7-command-warning");
      bridgeTracePortProfileR35B(net, "r35b-out-of-profile-warning", {
        warning: profileWarning.slice(0, 1200),
        policy: "warning-only; start is not blocked unless a real conflict exists"
      });
    } else {
      delete preview.dataset.kgwBridgeCommandWarning;
      preview.classList.remove("bridge-v7-command-warning");
    }

    bridgeApplyPortConflictStartStateR33(net, portValidation, "update-command");

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
/* KGW_BRIDGE_LAST_NETWORK_RESTORE_R101W2 */
const KGW_BRIDGE_LAST_NETWORK_KEY_R101W2 = "kgw.bridge.lastNetwork";
function kgwBridgeNormalizeNetworkR101W2(value) {
  const normalized = String(value || "").trim();
  return normalized === "mainnet" || normalized === "testnet10" || normalized === "testnet12" ? normalized : "";
}
function kgwBridgeReadLastNetworkR101W2() {
  try { return kgwBridgeNormalizeNetworkR101W2(localStorage.getItem(KGW_BRIDGE_LAST_NETWORK_KEY_R101W2)); } catch (_) { return ""; }
}
function kgwBridgeSaveLastNetworkR101W2(net) {
  const normalized = kgwBridgeNormalizeNetworkR101W2(net);
  if (!normalized) return "";
  try { localStorage.setItem(KGW_BRIDGE_LAST_NETWORK_KEY_R101W2, normalized); } catch (_) {}
  return normalized;
}

function installNetworkTabs(root) {
  // KGW_R63_DIRECT_BRIDGE_NETWORK_TAB_SWITCH_OWNER
  // KGW_BRIDGE_LAST_NETWORK_RESTORE_R101W2
  const networkTabSelector = "[data-bridge-network-tab]";
  const networkPanelSelector = "[data-bridge-network-panel]";

  function normalizeNetFromElement(element) {
    if (!element) return "";
    return element.dataset.net || element.dataset.bridgeNetworkTab || element.dataset.bridgeNetworkPanel || "";
  }

  function allNetworkTabs() { return Array.from(root.querySelectorAll(networkTabSelector)); }
  function allNetworkPanels() { return Array.from(root.querySelectorAll(networkPanelSelector)); }

  function selectBridgeNetwork(net, reason = "manual", persist = false) {
    const normalized = kgwBridgeNormalizeNetworkR101W2(net);
    if (!normalized) return;
    if (persist) kgwBridgeSaveLastNetworkR101W2(normalized);

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

    if (typeof updateCommand === "function") updateCommand(normalized);
    if (typeof kgwBridgeR51RefreshOne === "function") {
      window.setTimeout(() => kgwBridgeR51RefreshOne(normalized, "network-tab-" + reason), 50);
      window.setTimeout(() => kgwBridgeR51RefreshOne(normalized, "network-tab-" + reason + "+700ms"), 700);
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
      patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_BRIDGE_LAST_NETWORK_RESTORE_R101W2",
      trusted: Boolean(event && event.isTrusted),
      selected: String(net || ""),
      text: String(tab.textContent || "").trim(),
      persisted: true
    });
    selectBridgeNetwork(net, "click", true);
  }, true);

  const saved = kgwBridgeReadLastNetworkR101W2();
  const existingActiveTab = allNetworkTabs().find((tab) => tab.classList.contains("active") || tab.classList.contains("is-active") || tab.getAttribute("aria-selected") === "true" || tab.dataset.active === "true");
  const defaultTab = (saved && allNetworkTabs().find((tab) => normalizeNetFromElement(tab) === saved)) || existingActiveTab || allNetworkTabs().find((tab) => normalizeNetFromElement(tab) === "mainnet") || allNetworkTabs()[0];
  if (defaultTab) selectBridgeNetwork(normalizeNetFromElement(defaultTab), saved ? "saved-initial" : "initial", false);

  window.kgwBridgeSelectNetworkTabR63 = (net) => selectBridgeNetwork(net, "external", true);
  window.kgwBridgeSelectNetworkTabR101W2 = window.kgwBridgeSelectNetworkTabR63;
}

function installDelegatedTabs(root) {
  root.addEventListener("click", (event) => {
    const innerTab = event.target.closest("[data-bridge-inner-tab]");
    if (innerTab) {
      const net = innerTab.dataset.net;
      const selected = kgwBridgeSaveInnerTabR101U(net, innerTab.dataset.bridgeInnerTab);
      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      kgwBridgeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-bridge-inner-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected || ""),
        text: String(innerTab.textContent || "").trim(),
        persisted: true
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
      kgwBridgeRenderRawLogBufferV1(net, "bridge", String(selected));

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
// The Bridge child contract is 101 seconds and the same-EXE parent is bounded at
// 110 seconds. Keep the UI request strictly above both terminal-result boundaries.
const KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS = 120000;
const KGW_BRIDGE_RUNTIME_FLAGS_OWNER_COMMAND = "rk_integrated_bridge_runtime_flags_v1";
const KGW_BRIDGE_START_TRACE_COMMAND_V1 = "kgw_start_trace_frontend_v1";
const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();

function getTauriInvoke() {
  const tauri = window.__TAURI__;
  return tauri?.core?.invoke || tauri?.invoke || window.__TAURI_INVOKE__ || null;
}

function kgwBridgeStartTraceSafeTextV1(value, fallback = "") {
  const text = String(value ?? "").replace(/[\r\n\t]+/g, " ").trim();
  return (text || fallback).slice(0, 220);
}

function kgwBridgeStartTraceSafeDetailsV1(details) {
  const source = details && typeof details === "object" ? details : {};
  const blocked = /(secret|token|private|mnemonic|wallet|address|commandPreview|completeCommand|arguments|appDir|path|rpcEndpoint|stratum)/i;
  const out = {};

  for (const [key, value] of Object.entries(source)) {
    if (blocked.test(key)) {
      out[key] = "[redacted]";
      continue;
    }

    if (Array.isArray(value)) {
      out[key] = value.slice(0, 24).map((item) => kgwBridgeStartTraceSafeTextV1(item));
    } else if (value && typeof value === "object") {
      out[key] = kgwBridgeStartTraceSafeDetailsV1(value);
    } else if (typeof value === "boolean" || typeof value === "number") {
      out[key] = value;
    } else {
      out[key] = kgwBridgeStartTraceSafeTextV1(value);
    }
  }

  return out;
}

function kgwBridgeStartTraceFrontendV1(stage, options = {}) {
  const invoke = getTauriInvoke();
  if (typeof invoke !== "function") return false;

  const network = kgwBridgeStartTraceSafeTextV1(options.network || options.net, "unknown");
  const action = kgwBridgeStartTraceSafeTextV1(options.action, "unknown");
  const result = kgwBridgeStartTraceSafeTextV1(options.result, "observed");
  const details = kgwBridgeStartTraceSafeDetailsV1(options.details && typeof options.details === "object" ? options.details : {});

  invoke(KGW_BRIDGE_START_TRACE_COMMAND_V1, {
    stage: kgwBridgeStartTraceSafeTextV1(stage, "frontend.unknown"),
    network,
    action,
    result,
    details: JSON.stringify(details)
  }).catch(function (error) {
    console.error("[KGW_START_TRACE_FRONTEND_FAILED]", error && error.message ? error.message : String(error));
  });

  return true;
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

    // KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
    // Start must honor the selected Bridge Instance, not only the generic network Stratum port.
    const structuredInstances = typeof kgwBridgeR51ReadStructuredInstancesR26B === "function"
      ? kgwBridgeR51ReadStructuredInstancesR26B(net)
      : { activeInstance: String(activeInstance?.[net] || ""), instances: Array.isArray(bridgeInstances?.[net]) ? bridgeInstances[net] : [] };
    const bridgeActiveInstanceId = String(structuredInstances?.activeInstance || activeInstance?.[net] || "");
    const bridgeActiveInstanceRecord = Array.isArray(structuredInstances?.instances)
      ? structuredInstances.instances.find((item) => String(item?.id || "") === bridgeActiveInstanceId) || structuredInstances.instances[0] || null
      : null;
    const bridgeActiveInstance = bridgeActiveInstanceRecord && typeof bridgeBuildUpstreamInstanceArg === "function"
      ? bridgeBuildUpstreamInstanceArg(net, bridgeActiveInstanceRecord)
      : "";
    const bridgeActiveInstancePort = String(bridgeActiveInstanceRecord?.instancePort || "").trim().replace(/^:/, "");

    return {
      network: net,
      runtimeRole: "bridge",
      nodeKind: nodeMode === "inprocess" ? "integrated-inproc" : "remote",
      bridgeKind: nodeMode === "inprocess" ? "official-inprocess-node" : "official-external-node",
      nodeCommandPreview: "",
      bridgeCommandPreview: preview,
      bridgeActiveInstanceId,
      bridgeActiveInstance,
      bridgeActiveInstancePort,
      bridgeStructuredInstances: JSON.stringify(structuredInstances || {}),
      experimentalNetworkOptIn: net === "testnet12" && kgwBridgeNetworkEnabled(net),
    };
  }

  if (
    command === "kgw_kgw_disable_network_v1" ||
    command === "kgw_runtime_owner_status_v1" ||
    command === "kgw_kgw_runtime_logs_v1" ||
    command === "kgw_kgw_runtime_clear_logs_v1"
  ) {
    return { network: net, runtimeRole: "bridge", bridgeInstanceId: String(activeInstance?.[net] || "") };
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
  const ready = String(fields.readiness || "").toUpperCase() === "READY";
  return (
    ready &&
    (fields.running === "true" ||
      fields.node_running === "true" ||
      fields.official_core_running === "true" ||
      fields.bridge_running === "true" ||
      fields.bridge_owner_active === "true" ||
      /running=true/i.test(String(text || "")))
  );
}

function kgwBridgeSetRuntimeErrorV1(net, errorText = "") {
  const errorNode = byId(id(net, "runtimeError"));
  if (!errorNode) return;
  const text = String(errorText || "").trim();
  errorNode.textContent = text;
  errorNode.hidden = !text;
}

function kgwBridgeSetRuntimeActivityV1(net, message = "") {
  const statusNode = byId(id(net, "runtimeStatus"));
  if (!statusNode) return;
  statusNode.textContent = String(message || "").trim();
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

// KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E
function kgwBridgeOwnedNodeLockStoreR65E() {
  if (!window.__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E || typeof window.__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E !== "object") {
    window.__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E = {};
  }
  return window.__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E;
}

function kgwSetBridgeOwnedNodeLockR65E(net, locked, details) {
  const key = String(net || "");
  if (!key) return;
  const store = kgwBridgeOwnedNodeLockStoreR65E();

  if (locked) {
    store[key] = {
      locked: true,
      net: key,
      reason: "bridge-inprocess-owner",
      updatedAt: Date.now(),
      details: details && typeof details === "object" ? details : {}
    };
  } else {
    delete store[key];
  }

  try {
    window.dispatchEvent(new CustomEvent("kgw-bridge-owned-node-lock-r65e", {
      detail: {
        net: key,
        locked: Boolean(locked),
        source: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E"
      }
    }));
  } catch (_) {}
}

function kgwIsBridgeOwnedNodeLockedR65E(net) {
  const key = String(net || "");
  if (!key) return false;
  const store = kgwBridgeOwnedNodeLockStoreR65E();
  return Boolean(store[key] && store[key].locked);
}

// KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_MAINNET_IMMEDIATE_R65F
function kgwBridgeNormalizeNodeModeR65F(value) {
  return String(value || "").toLowerCase().replace(/[^a-z0-9]+/g, "");
}

function kgwBridgeCurrentNodeModeFromUiR65F(net) {
  try {
    const direct = byId(id(net, "nodeMode"));
    if (direct && "value" in direct) return String(direct.value || "");
  } catch (_) {}

  try {
    const panel = document.querySelector('[data-bridge-panel="' + String(net || "") + '"]') ||
      document.querySelector('[data-net="' + String(net || "") + '"]');
    if (panel) {
      const select = panel.querySelector('[id$="-nodeMode"], [data-bridge-setting="nodeMode"], select[name="nodeMode"]');
      if (select && "value" in select) return String(select.value || "");
    }
  } catch (_) {}

  return "";
}

function kgwBridgePreviewDeclaresInprocessR65F(preview) {
  const text = String(preview || "").toLowerCase();
  return /--node-mode\s*=\s*in-?process/.test(text) ||
    /--node-mode\s+in-?process/.test(text) ||
    /node-mode=in-?process/.test(text) ||
    /node_mode=in-?process/.test(text);
}

function kgwBridgeStartWasInprocessR65F(net, fields, preview) {
  const fieldMode = kgwBridgeNormalizeNodeModeR65F(fields && (fields.node_mode || fields.nodeMode));
  const uiMode = kgwBridgeNormalizeNodeModeR65F(kgwBridgeCurrentNodeModeFromUiR65F(net));
  const previewMode = kgwBridgePreviewDeclaresInprocessR65F(preview);

  return fieldMode === "inprocess" ||
    fieldMode === "inproc" ||
    uiMode === "inprocess" ||
    uiMode === "inproc" ||
    previewMode;
}

async function runBridgeIntegratedAction(action, net) {
  function kgwBridgeRuntimeOwnerTraceR64D(phase, details) {
    try {
      const safeNet = String(net || "unknown");
      const safeAction = String(action || "unknown");
      const safePhase = String(phase || "unknown");
      const payload = {
        patch: "KGW_BRIDGE_RUNTIME_OWNER_TRACE_R64D",
        owner: "runBridgeIntegratedAction-existing-owner",
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

  kgwBridgeRuntimeOwnerTraceR64D("r64d-runtime-owner-enter", {
    action: String(action || ""),
    net: String(net || "")
  });

  const commandByAction = {
    start: "kgw_kgw_apply_node_settings_v1",
    stop: "kgw_kgw_disable_network_v1"
  };

  const command = commandByAction[action];

  if (!command) {
    kgwBridgeRuntimeOwnerTraceR64D("r64d-invalid-action-return", {
      action: String(action || "")
    });
    return false;
  }

  if (action === "start" && !kgwBridgeNetworkEnabled(net)) {
    kgwBridgeSetRuntimeErrorV1(
      net,
      "Bridge start blocked: this network is disabled. Enable it in the network policy bar first."
    );
    kgwBridgeR51SetRuntimeButtons(net, false);
    return true;
  }

  if (action === "start") {
    kgwBridgeRuntimeOwnerTraceR64D("r64d-preflight-begin", {
      command
    });

    // KGW_BRIDGE_RUNTIME_START_SCOPED_CONFLICT_R111F
    // Use the registered scoped conflict owner instead of the retired global R33 pre-start blocker.
    const scopedConflictResultR111F = bridgeAssertNoPortConflictsR5(net);

    kgwBridgeRuntimeOwnerTraceR64D("r111f-scoped-conflict-owner-result", {
      owner: "bridgeRuntimeStartOwner",
      conflictOwner: "bridgeInstances.bridgeAssertNoPortConflictsR5",
      ok: scopedConflictResultR111F && typeof scopedConflictResultR111F === "object"
        ? scopedConflictResultR111F.ok !== false
        : true,
      conflictCount: scopedConflictResultR111F && typeof scopedConflictResultR111F === "object"
        ? Number(scopedConflictResultR111F.conflictCount || 0)
        : 0
    });

    if (scopedConflictResultR111F && typeof scopedConflictResultR111F === "object" && scopedConflictResultR111F.ok === false) {
      kgwBridgeSetRuntimeErrorV1(net, String(scopedConflictResultR111F.message || "Bridge listener port conflict."));
      kgwBridgeRuntimeOwnerTraceR64D("r111f-scoped-conflict-start-blocked-return", {
        reason: "scoped-port-conflict",
        conflictCount: Number(scopedConflictResultR111F.conflictCount || 0),
        message: String(scopedConflictResultR111F.message || "")
      });
      return true;
    }

    const blockedBySameNetworkNode = await kgwBridgeV7BlockInprocessIfNodeOwnerRunning(net);

    kgwBridgeRuntimeOwnerTraceR64D("r64d-preflight-result", {
      blockedBySameNetworkNode: Boolean(blockedBySameNetworkNode)
    });

    if (blockedBySameNetworkNode) {
      kgwBridgeSetRuntimeErrorV1(
        net,
        "Bridge start blocked: same-network node is already running in in-process mode."
      );

      kgwBridgeRuntimeOwnerTraceR64D("r64d-preflight-blocked-return", {
        reason: "same-network-node-running-inprocess"
      });

      return true;
    }
  }

  const inFlightKey = net + ":" + action;

  kgwBridgeRuntimeOwnerTraceR64D("r64d-inflight-check", {
    inFlightKey,
    alreadyInFlight: KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(inFlightKey)
  });

  if (KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(inFlightKey)) {
    kgwBridgeSetRuntimeActivityV1(net, "Bridge " + action + " already in progress.");

    kgwBridgeRuntimeOwnerTraceR64D("r64d-inflight-duplicate-return", {
      inFlightKey
    });

    return true;
  }

  KGW_BRIDGE_RUNTIME_IN_FLIGHT.add(inFlightKey);
  kgwBridgeSetRuntimeErrorV1(net, "");
  if (action === "start") {
    kgwBridgeR51SetRuntimeButtons(net, false, "starting");
  }

  kgwBridgeRuntimeOwnerTraceR64D("r64d-inflight-added", {
    inFlightKey
  });

  try {
    kgwBridgeRuntimeOwnerTraceR64D("r64d-preview-begin", {
      command
    });

    const preview = updateCommand(net) || byId(id(net, "commandPreview"))?.value || "";

    kgwBridgeRuntimeOwnerTraceR64D("r64d-preview-ready", {
      hasPreview: Boolean(preview),
      previewLength: String(preview || "").length
    });

    kgwBridgeSetRuntimeActivityV1(net, "Bridge " + action + " requested.");

    kgwBridgeRuntimeOwnerTraceR64D("r64d-invoke-begin", {
      command,
      hasPreview: Boolean(preview)
    });

    const result = await invokeBridgeIntegratedRuntime(command, net);

    kgwBridgeRuntimeOwnerTraceR64D("r64d-invoke-result", {
      resultType: typeof result,
      resultStringLength: String(result ?? "").length
    });

    const raw = stringifyRuntimeResult(result);
    const parsed = parseRuntimeKeyValueResponse(result);
    const fields = parsed.fields || {};

    kgwBridgeRuntimeOwnerTraceR64D("r64d-response-parsed", {
      rawLength: String(raw || "").length,
      fieldKeys: Object.keys(fields)
    });

    if (action === "start") {
      const confirmedStarted =
        String(fields.readiness || "").toUpperCase() === "READY" &&
        (/parallel-owned-self-worker\s+started/i.test(raw) ||
          /parallel-owned-self-worker\s+already\s+running/i.test(raw) ||
          (/role=bridge/i.test(raw) && /started|running=true|already running/i.test(raw)) ||
          fields.running === "true" ||
          fields.bridge_running === "true" ||
          fields.bridge_owner_active === "true");

      const blocked =
        fields.start_blocked === "true" ||
        fields.start_allowed === "false" ||
        /blocked|not enabled|failed/i.test(raw);

      kgwBridgeRuntimeOwnerTraceR64D("r64d-start-confirmation-evaluated", {
        confirmedStarted: Boolean(confirmedStarted),
        blocked: Boolean(blocked)
      });

      if (confirmedStarted && !blocked) {
        kgwBridgeSetRuntimeErrorV1(net, "");
        kgwBridgeR51SetRuntimeButtons(net, true);
        const bridgeNodeMode = String(fields.node_mode || fields.nodeMode || "").toLowerCase();
        const bridgeStartWasInprocess = kgwBridgeStartWasInprocessR65F(net, fields, preview);
        if (bridgeStartWasInprocess) {
          kgwSetBridgeOwnedNodeLockR65E(net, true, {
            source: "bridge-start-confirmed-r65f",
            action: "start",
            nodeMode: bridgeNodeMode,
            uiNodeMode: kgwBridgeCurrentNodeModeFromUiR65F(net),
            previewDeclaredInprocess: kgwBridgePreviewDeclaresInprocessR65F(preview),
            pid: String(fields.pid || "")
          });
        }
        kgwBridgeRuntimeOwnerTraceR64D("r65f-bridge-owned-node-lock-evaluated", {
          patch: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_MAINNET_IMMEDIATE_R65F",
          bridgeNodeMode,
          uiNodeMode: kgwBridgeCurrentNodeModeFromUiR65F(net),
          previewDeclaredInprocess: kgwBridgePreviewDeclaresInprocessR65F(preview),
          bridgeStartWasInprocess
        });
        kgwBridgeSetRuntimeActivityV1(net, "Bridge READY attestation confirmed.");
        kgwBridgeR51KickRawLogLiveR134E(net, "bridge-start-confirmed");
      } else if (blocked) {
        kgwBridgeR51SetRuntimeButtons(net, false);
        kgwBridgeSetRuntimeErrorV1(net, raw);
        kgwBridgeSetRuntimeActivityV1(net, "Bridge start failed.");
      } else {
        kgwBridgeR51SetRuntimeButtons(net, false);
        kgwBridgeSetRuntimeErrorV1(net, "Backend Start did not provide READY attestation: " + raw);
        kgwBridgeSetRuntimeActivityV1(net, "Bridge start was not confirmed by READY attestation.");
      }
    }

    if (action === "stop") {
      const confirmedStopped =
        /parallel-owned-self-worker\s+stopped/i.test(raw) ||
        fields.running === "false" ||
        fields.bridge_running === "false";

      kgwBridgeRuntimeOwnerTraceR64D("r64d-stop-confirmation-evaluated", {
        confirmedStopped: Boolean(confirmedStopped)
      });

      if (confirmedStopped) {
        kgwBridgeR51SetRuntimeButtons(net, false);
        kgwBridgeSetRuntimeErrorV1(net, "");
        kgwSetBridgeOwnedNodeLockR65E(net, false, {
          source: "bridge-stop-confirmed",
          action: "stop"
        });
        kgwBridgeSetRuntimeActivityV1(net, "Bridge stop confirmed.");
      } else {
        kgwBridgeSetRuntimeActivityV1(net, "Bridge stop was not confirmed by runtime response.");
      }
    }

    kgwBridgeRuntimeOwnerTraceR64D("r64d-runtime-owner-return-success", {
      action: String(action || "")
    });

    return true;
  } catch (error) {
    const message = error && error.message ? error.message : String(error);

    kgwBridgeRuntimeOwnerTraceR64D("r64d-runtime-owner-catch", {
      message
    });

    kgwBridgeR51SetRuntimeButtons(net, action === "stop");
    kgwBridgeSetRuntimeErrorV1(net, message);
    kgwBridgeSetRuntimeActivityV1(net, "Bridge " + action + " failed.");

    return true;
  } finally {
    KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);

    kgwBridgeRuntimeOwnerTraceR64D("r64d-runtime-owner-finally", {
      inFlightKey
    });
  }
}
/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */
const KGW_BRIDGE_R51_STORAGE_PREFIX = "kgw.bridge.direct.v51.";
const KGW_BRIDGE_R51_LAST_STATUS = {};
const KGW_BRIDGE_R51_LAST_LOGS = {};
const KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE = {};
let KGW_BRIDGE_R51_TIMER = null;


/* KGW_BRIDGE_SETTINGS_STRUCTURED_INSTANCES_PERSISTENCE_PATCH_R26B
 * Bridge settings persistence must not rely only on dynamic DOM field ids.
 * Bridge Instances use runtime-generated ids, so saved field-id maps can become stale after reload/re-render.
 * This patch keeps the existing R51 settings owner and stores/restores structured bridgeInstances state.
 * No new persistence owner. No document listener. No MutationObserver.
 */
const KGW_BRIDGE_R51_STRUCTURED_INSTANCES_KEY_R26B = "__kgwBridgeStructuredInstancesR26B";
const KGW_BRIDGE_R51_ACTIVE_INSTANCE_KEY_R26B = "__kgwBridgeActiveInstanceR26B";

function kgwBridgeR51CommitInstanceDomStateR26B(net) {
  try {
    net = bridgeInstanceNetworkKeyR15(net, net);
    if (!net) return [];

    bridgeEnsureInstanceState(net);

    if (!Array.isArray(bridgeInstances[net])) {
      bridgeInstances[net] = [];
    }

    bridgeInstances[net] = bridgeInstances[net].map((instance, index) => {
      const fallbackId = instance && instance.id ? instance.id : Date.now() + index;
      const instanceId = instance && instance.id ? instance.id : fallbackId;
      return bridgeReadInstanceState(net, instanceId);
    });

    bridgeEnsureInstanceState(net);
    return Array.isArray(bridgeInstances[net]) ? bridgeInstances[net] : [];
  } catch (error) {
    try {
      kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r26b-commit-instance-dom-state-failed", {
        message: error && error.message ? error.message : String(error)
      });
    } catch (_) {}
    return Array.isArray(bridgeInstances && bridgeInstances[net]) ? bridgeInstances[net] : [];
  }
}

function kgwBridgeR51ReadStructuredInstancesR26B(net) {
  net = bridgeInstanceNetworkKeyR15(net, net);
  const committed = kgwBridgeR51CommitInstanceDomStateR26B(net);

  const instances = committed.map((instance, index) => {
    const fallbackId = instance && instance.id ? instance.id : Date.now() + index;
    return bridgeNormalizeInstanceRecord(instance, fallbackId);
  }).filter(Boolean);

  const active = activeInstance[net] || (instances[0] && instances[0].id) || "";

  return {
    version: 1,
    activeInstance: String(active || ""),
    instances
  };
}


/* KGW_BRIDGE_COMMAND_CHECKBOX_PERSISTENCE_PATCH_R38C
 * Persist command include/exclude checkboxes by semantic keys, not empty DOM ids.
 * This patches the existing R51 settings persistence owner only.
 */
const KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C = "__kgwBridgeCommandOptionsR38C";
const KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C = "__kgwBridgeInstanceCommandOptionsR38C";

function kgwBridgeR51ReadCommandOptionsR38C(net) {
  const state = {};
  try {
    const root = document.getElementById("kaspa-bridge");
    if (!root) return state;

    for (const item of root.querySelectorAll('[data-bridge-command-option-toggle-r7][data-net="' + String(net || "") + '"]')) {
      const name = String(item.dataset.bridgeCommandOptionToggleR7 || "");
      if (!name) continue;
      state[name] = Boolean(item.checked);
    }
  } catch (_) {}
  return state;
}

function kgwBridgeR51ReadInstanceCommandOptionsR38C(net) {
  const state = {};
  try {
    const root = document.getElementById("kaspa-bridge");
    if (!root) return state;

    for (const item of root.querySelectorAll('[data-bridge-instance-command-option-toggle-r13-b][data-net="' + String(net || "") + '"]')) {
      const instanceId = String(item.dataset.instanceId || "");
      const name = String(item.dataset.bridgeInstanceCommandOptionToggleR13B || "");
      if (!instanceId || !name) continue;
      state[instanceId] = state[instanceId] || {};
      state[instanceId][name] = Boolean(item.checked);
    }
  } catch (_) {}
  return state;
}

function kgwBridgeR51ApplyCommandOptionsR38C(net, values) {
  try {
    const commandOptions = values && values[KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C];
    if (commandOptions && typeof commandOptions === "object") {
      const state = kgwBridgeCommandInlineStateR7(net);
      for (const [name, enabled] of Object.entries(commandOptions)) {
        state[String(name)] = Boolean(enabled);
      }
      kgwBridgeRefreshInlineCommandTogglesR7(net);
    }

    const instanceOptions = values && values[KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C];
    if (instanceOptions && typeof instanceOptions === "object") {
      for (const [instanceId, options] of Object.entries(instanceOptions)) {
        if (!options || typeof options !== "object") continue;
        for (const [name, enabled] of Object.entries(options)) {
          kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, Boolean(enabled));
        }
      }
      bridgeSyncInstancePreviewRowsR8B(net);
    }

    updateCommand(net);

    kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restored", {
      patch: "R38C",
      owner: "bridge-r51-settings-owner",
      commandOptionCount: commandOptions && typeof commandOptions === "object" ? Object.keys(commandOptions).length : 0,
      instanceCount: instanceOptions && typeof instanceOptions === "object" ? Object.keys(instanceOptions).length : 0
    });
  } catch (error) {
    kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restore-failed", {
      patch: "R38C",
      owner: "bridge-r51-settings-owner",
      message: error && error.message ? error.message : String(error)
    });
  }
}


function kgwBridgeR51ApplyStructuredInstancesR26B(net, values) {
  try {
    net = bridgeInstanceNetworkKeyR15(net, net);
    if (!values || typeof values !== "object") return false;

    const payload = values[KGW_BRIDGE_R51_STRUCTURED_INSTANCES_KEY_R26B];
    if (!payload || typeof payload !== "object" || !Array.isArray(payload.instances)) return false;

    const normalized = payload.instances.map((instance, index) => {
      const fallbackId = instance && instance.id ? instance.id : Date.now() + index;
      return bridgeNormalizeInstanceRecord(instance, fallbackId);
    }).filter(Boolean);

    bridgeInstances[net] = normalized.length
      ? normalized
      : [bridgeDefaultInstanceRecord(Date.now())];

    const wantedActive = String(payload.activeInstance || values[KGW_BRIDGE_R51_ACTIVE_INSTANCE_KEY_R26B] || "");
    const exists = bridgeInstances[net].some((instance) => String(instance.id) === wantedActive);

    activeInstance[net] = exists
      ? wantedActive
      : String((bridgeInstances[net][0] && bridgeInstances[net][0].id) || "");

    bridgeRefreshInstances(net);
    kgwBridgeRenderRawLogBufferV1(net, "bridge", String(activeInstance[net] || ""));

    try {
      kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r26b-structured-instances-restored", {
        count: bridgeInstances[net].length,
        activeInstance: String(activeInstance[net] || "")
      });
    } catch (_) {}

    return true;
  } catch (error) {
    try {
      kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r26b-apply-structured-instances-failed", {
        message: error && error.message ? error.message : String(error)
      });
    } catch (_) {}
    return false;
  }
}

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

  const structuredInstances = kgwBridgeR51ReadStructuredInstancesR26B(net);
  values[KGW_BRIDGE_R51_STRUCTURED_INSTANCES_KEY_R26B] = structuredInstances;
  values[KGW_BRIDGE_R51_ACTIVE_INSTANCE_KEY_R26B] = structuredInstances.activeInstance;
  values[KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C] = kgwBridgeR51ReadCommandOptionsR38C(net);
  values[KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C] = kgwBridgeR51ReadInstanceCommandOptionsR38C(net);

  for (const field of kgwBridgeR51Fields(net)) {
    if (!field.id) continue;

    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }

  kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-read-settings-command-options", {
    patch: "R38C",
    owner: "bridge-r51-settings-owner",
    commandOptionCount: Object.keys(values[KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C] || {}).length,
    instanceCommandOptionInstanceCount: Object.keys(values[KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C] || {}).length
  });

  return kgwBridgeR95BNormalizeNetworkPortValues(net, values, "read-settings");
}

/* KGW_BRIDGE_NETWORK_PORT_RANGE_R51_OWNER_FIX_R95B
 * Existing R51 Bridge settings owner refinement.
 *
 * Runtime screenshots showed stale network-level bridge ports:
 * - testnet10 was replayed as :5556 / :2113
 * - testnet12 was replayed as :5557 / :2114
 *
 * Correct bridge-level network ranges already exist in KGW_BRIDGE_PORT_PROFILES_R35B:
 * - mainnet   :5555 / :2112
 * - testnet10 :5655 / :2212
 * - testnet12 :5755 / :2312
 *
 * This patch normalizes only known stale sequential saved/default values while
 * preserving explicit custom user ports.
 */
/* KGW_BRIDGE_PORT_ONLY_COLON_DISPLAY_FIX_R98
 * Existing R95B/R51 Bridge settings owner refinement.
 *
 * Port-only UI fields must display plain numbers such as 5655, not :5655.
 * Host:port fields and command preview syntax remain untouched.
 */
function kgwBridgeR98PlainPortOnlyValue(value) {
  const text = String(value || "").trim();
  const match = text.match(/^:?(\d{1,5})$/);
  if (!match) return text;

  const port = Number(match[1]);
  if (!Number.isInteger(port) || port < 1 || port > 65535) return text;

  return String(port);
}

function kgwBridgeR98SamePortValue(left, right) {
  return kgwBridgeR98PlainPortOnlyValue(left) === kgwBridgeR98PlainPortOnlyValue(right);
}

function kgwBridgeR95BNormalizePlainPortValue(value) {
  return kgwBridgeR98PlainPortOnlyValue(value);
}

function kgwBridgeR95BStorageFieldId(net, fieldName) {
  return "bridge-" + String(net || "") + "-" + String(fieldName || "");
}

function kgwBridgeR95BPreferredPort(net, kind) {
  const profile = typeof bridgeStaticPortProfileR91 === "function"
    ? bridgeStaticPortProfileR91(net)
    : (KGW_BRIDGE_PORT_PROFILES_R35B[String(net || "")] || KGW_BRIDGE_PORT_PROFILES_R35B.mainnet);

  const range = profile && profile[kind];
  return range && range.preferred ? kgwBridgeR98PlainPortOnlyValue(range.preferred) : "";
}

function kgwBridgeR95BKnownStaleSequentialPort(net, fieldName) {
  const stale = {
    testnet10: {
      stratumPort: "5556",
      promPort: "2113"
    },
    testnet12: {
      stratumPort: "5557",
      promPort: "2114"
    }
  };

  return stale[String(net || "")] && stale[String(net || "")][fieldName]
    ? stale[String(net || "")][fieldName]
    : "";
}

function kgwBridgeR95BNormalizeNetworkPortValues(net, values, reason) {
  if (!values || typeof values !== "object") return values;

  const fields = [
    { fieldName: "stratumPort", kind: "stratum" },
    { fieldName: "promPort", kind: "prom" }
  ];

  const changes = [];

  for (const field of fields) {
    const storageId = kgwBridgeR95BStorageFieldId(net, field.fieldName);
    const item = values[storageId];

    if (!item || typeof item !== "object" || !("value" in item)) continue;

    const current = kgwBridgeR95BNormalizePlainPortValue(item.value);
    const stale = kgwBridgeR95BKnownStaleSequentialPort(net, field.fieldName);
    const preferred = kgwBridgeR95BPreferredPort(net, field.kind);

    if (stale && preferred && kgwBridgeR98SamePortValue(current, stale) && !kgwBridgeR98SamePortValue(current, preferred)) {
      item.value = kgwBridgeR98PlainPortOnlyValue(preferred);
      changes.push({
        field: field.fieldName,
        from: current,
        to: item.value
      });
    } else if (current !== item.value && /^:?\d{1,5}$/.test(String(item.value || "").trim())) {
      item.value = current;
      changes.push({
        field: field.fieldName,
        from: String(item.value || ""),
        to: current,
        displayOnly: true
      });
    }
  }

  if (changes.length && typeof kgwBridgeSmallOwnerTraceR44D === "function") {
    kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r98-normalize-port-only-display-values", {
      patch: "R98",
      owner: "bridge-r51-r95b-settings-owner",
      reason: String(reason || ""),
      changes
    });
  }

  return values;
}

function kgwBridgeR51WriteSettings(net, values) {
  if (!values || typeof values !== "object") return;

  values = kgwBridgeR95BNormalizeNetworkPortValues(net, values, "write-settings");

  kgwBridgeR51ApplyStructuredInstancesR26B(net, values);

  for (const field of kgwBridgeR51Fields(net)) {
    if (!field.id) continue;

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

  kgwBridgeR51ApplyCommandOptionsR38C(net, values);

  if (typeof bridgeReassignInstancePortsFromExternalRangeR91 === "function") {
    bridgeReassignInstancePortsFromExternalRangeR91(net, "r95b-r51-write-settings-normalized-network-ports");
  }

  bridgeSyncInstancePreviewRowsR8B(net);
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
      kgwBridgeR51WriteSettings(net, kgwBridgeR95BNormalizeNetworkPortValues(net, saved, "load-saved-settings"));
    } else {
      kgwBridgeR51WriteSettings(net, kgwBridgeR95BNormalizeNetworkPortValues(net, kgwBridgeR51ReadSettings(net), "load-current-settings"));
    }
  }
}

/* KGW_BRIDGE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */


function kgwBridgeR51SaveSettings(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-begin", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner"
  });

  const values = kgwBridgeR51ReadSettings(net);
  kgwBridgeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-read-settings", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner",
    keyCount: Object.keys(values || {}).length,
    checkboxCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "checkbox").length,
    valueCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "value").length,
    structuredInstanceCount: (values && values.__kgwBridgeStructuredInstancesR26B && Array.isArray(values.__kgwBridgeStructuredInstancesR26B.instances)) ? values.__kgwBridgeStructuredInstancesR26B.instances.length : 0,
    hasActiveStructuredInstance: Boolean(values && values.__kgwBridgeActiveInstanceR26B)
  });

  kgwBridgeR51Store("saved:" + net, values);

  const saved = kgwBridgeR51Load("saved:" + net);
  kgwBridgeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-complete", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner",
    savedKey: "saved:" + String(net || ""),
    persisted: Boolean(saved),
    persistedKeyCount: saved && typeof saved === "object" ? Object.keys(saved).length : 0
  });
}

function kgwBridgeR51SetAsDefaults(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-begin", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner"
  });

  const values = kgwBridgeR51ReadSettings(net);
  kgwBridgeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-read-settings", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner",
    keyCount: Object.keys(values || {}).length,
    checkboxCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "checkbox").length,
    valueCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "value").length,
    structuredInstanceCount: (values && values.__kgwBridgeStructuredInstancesR26B && Array.isArray(values.__kgwBridgeStructuredInstancesR26B.instances)) ? values.__kgwBridgeStructuredInstancesR26B.instances.length : 0,
    hasActiveStructuredInstance: Boolean(values && values.__kgwBridgeActiveInstanceR26B)
  });

  kgwBridgeR51Store("default:" + net, values);

  const stored = kgwBridgeR51Load("default:" + net);
  kgwBridgeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-complete", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner",
    defaultKey: "default:" + String(net || ""),
    persisted: Boolean(stored),
    persistedKeyCount: stored && typeof stored === "object" ? Object.keys(stored).length : 0
  });
}

function kgwBridgeR51RestoreDefaults(net) {
  kgwBridgeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-begin", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner"
  });

  kgwBridgeSettingsWithProgrammaticWriteR9B(() => {
    const defaults = kgwBridgeR51Load("default:" + net) || kgwBridgeR51Load("factory:" + net);
    kgwBridgeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-loaded", {
      patch: "R29B",
      owner: "bridge-r51-settings-owner",
      hasDefaults: Boolean(defaults),
      defaultKeyCount: defaults && typeof defaults === "object" ? Object.keys(defaults).length : 0
    });
    kgwBridgeR51WriteSettings(net, defaults);
    kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  });

  kgwBridgeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-complete", {
    patch: "R29B",
    owner: "bridge-r51-settings-owner"
  });
}

function kgwBridgeR51IsRunning(text) {
  const value = String(text || "");
  return /readiness=READY/i.test(value) && (/running=true/.test(value) || /bridge_running=true/.test(value) || /bridge_owner_active=true/.test(value));
}

function kgwBridgeR51SetRuntimeButtons(net, running, transition = "") {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return;

  const networkEnabled = kgwBridgeNetworkEnabled(net);
  const start = panel.querySelector(`[data-bridge-action="start"][data-net="${net}"]`);
  const stop = panel.querySelector(`[data-bridge-action="stop"][data-net="${net}"]`);
  const policyStatus = byId(id(net, "policyStatus"));

  if (policyStatus) {
    const state = !networkEnabled ? "Disabled" : transition === "starting" ? "Starting" : running ? "Running" : "Stopped";
    policyStatus.textContent = state;
    policyStatus.dataset.state = state.toLowerCase();
  }

  if (start) {
    const startBlocked = Boolean(running || transition === "starting" || !networkEnabled);
    start.disabled = startBlocked;
    start.style.opacity = startBlocked ? "0.45" : "";
    start.style.cursor = startBlocked ? "not-allowed" : "";
    start.title = !networkEnabled
      ? "Enable this network before starting it."
      : running
        ? "Bridge is running. Stop it before starting again."
        : "Start bridge";
  }

  if (stop) {
    const stopEnabled = Boolean(running || transition === "starting");
    stop.disabled = !stopEnabled;
    stop.style.opacity = stopEnabled ? "" : "0.45";
    stop.style.cursor = stopEnabled ? "" : "not-allowed";
    stop.title = stopEnabled ? "Stop bridge" : "Bridge is not running";
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
    const running = kgwBridgeR51IsRunning(status);
    const starting = KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(net + ":start");
    kgwBridgeR51SetRuntimeButtons(net, running, starting && !running ? "starting" : "");

    if (KGW_BRIDGE_R51_LAST_STATUS[net] !== status) {
      KGW_BRIDGE_R51_LAST_STATUS[net] = status;
      
    }

    kgwBridgeR51MaybeActivityNotice(net, status);
  } catch (error) {
    kgwBridgeR51SetRuntimeButtons(net, false);
  }

  try {
    const report = await invokeBridgeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net);
    const instanceId = kgwBridgeActiveRawLogInstanceIdV1(net);
    kgwBridgeApplyRuntimeLogReportV1(net, "bridge", report, instanceId);
    KGW_BRIDGE_R51_LAST_LOGS[net] = report;
  } catch {
    // Runtime may not be ready yet.
  }
}


// KGW_BRIDGE_RAW_LOG_LIVE_EXACT_R134E
// Raw bridge log live helper only: no parsing, no ASIC table, no bridge behavior duplication.
function kgwBridgeR51KickRawLogLiveR134E(net, reason = "bridge-start") {
  try {
    KGW_BRIDGE_R51_LAST_LOGS[net] = "";

    if (typeof kgwBridgeR51StartLiveRefresh === "function") {
      kgwBridgeR51StartLiveRefresh();
    }

    if (typeof kgwBridgeR51RefreshOne === "function") {
      window.setTimeout(function () { kgwBridgeR51RefreshOne(net, reason + "-0"); }, 0);
      window.setTimeout(function () { kgwBridgeR51RefreshOne(net, reason + "-350"); }, 350);
      window.setTimeout(function () { kgwBridgeR51RefreshOne(net, reason + "-1000"); }, 1000);
      window.setTimeout(function () { kgwBridgeR51RefreshOne(net, reason + "-2500"); }, 2500);
    }
  } catch (error) {
    console.warn("[KGW_BRIDGE_RAW_LOG_LIVE_EXACT_R134E_FAILED]", error);
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

function kgwBridgeClipboardCharacterCountV1(text) {
  return Array.from(String(text ?? "")).length;
}

function kgwBridgeClipboardLineCountV1(text) {
  const value = String(text ?? "");
  return value ? value.split("\n").length : 0;
}

function kgwBridgeNormalizeClipboardLineEndingsV1(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").replace(/\n/g, "\r\n");
}

async function kgwBridgeSha256HexV1(text) {
  try {
    const cryptoApi = window.crypto || globalThis.crypto;
    const Encoder = window.TextEncoder || globalThis.TextEncoder;
    if (!cryptoApi?.subtle?.digest || typeof Encoder !== "function") return "";

    const bytes = new Encoder().encode(String(text ?? ""));
    const digest = await cryptoApi.subtle.digest("SHA-256", bytes);
    return Array.from(new Uint8Array(digest))
      .map((value) => value.toString(16).padStart(2, "0"))
      .join("");
  } catch (_) {
    return "";
  }
}

function kgwBridgeClipboardSafeErrorV1(error) {
  const text = String(error && error.message ? error.message : error || "clipboard write failed")
    .replace(/[\r\n\t]+/g, " ")
    .trim();
  if (/(secret|token|private|mnemonic|wallet|address)/i.test(text)) {
    return "clipboard write failed with a sensitive error";
  }
  return (text || "clipboard write failed").slice(0, 360);
}

function kgwBridgeClipboardStatusElementV1(net) {
  const out = kgwBridgeLogOutputV29(net);
  const toolbar = out?.closest?.('[data-bridge-inner-panel="log"]')?.querySelector?.(".bridge-v7-log-toolbar");
  if (!toolbar) return null;

  let status = toolbar.querySelector('.kgw-copy-log-status-v1[data-net="' + net + '"]');
  if (!status) {
    status = document.createElement("span");
    status.setAttribute("class", "kgw-copy-log-status-v1");
    status.dataset.net = net;
    status.setAttribute("data-net", net);
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    toolbar.appendChild(status);
  }
  return status;
}

function kgwBridgeSetClipboardStatusV1(net, message, state = "info") {
  const status = kgwBridgeClipboardStatusElementV1(net);
  if (!status) return false;

  status.textContent = String(message || "");
  status.dataset.state = String(state || "info");
  status.hidden = !status.textContent;
  return true;
}

function kgwBridgeReadClipboardRawLogBufferV1(net) {
  const out = kgwBridgeLogOutputV29(net);
  const tag = String(out?.tagName || "").toUpperCase();
  const readsValue = tag === "TEXTAREA" || tag === "INPUT";
  const rawText = String(out ? (readsValue ? out.value : out.textContent) : "");
  const normalizedText = kgwBridgeNormalizeClipboardLineEndingsV1(rawText);

  return {
    out,
    rawText,
    normalizedText,
    characterCount: kgwBridgeClipboardCharacterCountV1(normalizedText),
    lineCount: kgwBridgeClipboardLineCountV1(normalizedText)
  };
}

async function kgwBridgeDispatchClipboardWriteV1(net, text, metadata) {
  const invoke = getTauriInvoke();
  if (typeof invoke !== "function") {
    throw new Error("Tauri invoke API is not available for Copy Log.");
  }

  kgwBridgeStartTraceFrontendV1("frontend.copy_log_dispatched", {
    network: net,
    action: "copy-log",
    result: "dispatched",
    details: {
      commandName: "kgw_copy_text_to_clipboard_v1",
      implementation: "native-tauri-command",
      runtimeRole: metadata.runtimeRole || "bridge",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || "",
      payloadFieldCount: 7
    }
  });

  return await invokeWithTimeout(
    invoke,
    "kgw_copy_text_to_clipboard_v1",
    {
      network: net,
      runtimeRole: metadata.runtimeRole || "bridge",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      text,
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || ""
    },
    KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS
  );
}

function kgwBridgeCopyLogFailureV1(net, button, error, details = {}) {
  const safeError = kgwBridgeClipboardSafeErrorV1(error);
  kgwBridgeSetClipboardStatusV1(net, safeError, "error");
  kgwBridgeFlashLogActionButtonV29(button, kgwBridgeTranslateRuntimeV29("log.copyFailed", "Copy failed"));
  kgwBridgeStartTraceFrontendV1("frontend.copy_log_failed", {
    network: net,
    action: "copy-log",
    result: "error",
    details: {
      ...details,
      runtimeRole: "bridge",
      safeError,
      userFeedbackDisplayed: true
    }
  });
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
    const instanceId = kgwBridgeActiveRawLogInstanceIdV1(net);
    const belongsToLiveBridgeMonitor = Boolean(button?.closest?.('[data-bridge-inner-panel="log"]'));

    kgwBridgeStartTraceFrontendV1("frontend.copy_log_network_resolved", {
      network: net,
      action: "copy-log",
      result: net ? "ok" : "error",
      details: {
        runtimeRole: "bridge",
        bridgeInstanceId: instanceId,
        belongsToLiveBridgeMonitor
      }
    });

    if (button?.dataset?.kgwCopyLogInFlightV1 === "1") {
      kgwBridgeCopyLogFailureV1(net, button, "Copy Log is already in progress for this bridge buffer.", {
        reason: "duplicate-copy",
        bridgeInstanceId: instanceId,
        belongsToLiveBridgeMonitor
      });
      return;
    }

    const originalDisabled = Boolean(button && button.disabled);
    if (button) {
      button.dataset.kgwCopyLogInFlightV1 = "1";
      button.disabled = true;
    }

    try {
      const buffer = kgwBridgeReadClipboardRawLogBufferV1(net);
      if (!buffer.out || !buffer.normalizedText.trim()) {
        kgwBridgeStartTraceFrontendV1("frontend.copy_log_content_prepared", {
          network: net,
          action: "copy-log",
          result: "error",
          details: {
            rawLogBufferSelected: Boolean(buffer.out),
            runtimeRole: "bridge",
            bridgeInstanceId: instanceId,
            characterCount: buffer.characterCount,
            lineCount: buffer.lineCount,
            sha256: ""
          }
        });
        throw new Error("Copy Log requires a non-empty raw log buffer for " + net + ".");
      }

      const sha256 = await kgwBridgeSha256HexV1(buffer.normalizedText);
      const metadata = {
        runtimeRole: "bridge",
        bridgeInstanceId: instanceId,
        characterCount: buffer.characterCount,
        lineCount: buffer.lineCount,
        sha256
      };

      kgwBridgeStartTraceFrontendV1("frontend.copy_log_content_prepared", {
        network: net,
        action: "copy-log",
        result: "ok",
        details: {
          rawLogBufferSelected: true,
          runtimeRole: metadata.runtimeRole,
          bridgeInstanceId: metadata.bridgeInstanceId,
          characterCount: metadata.characterCount,
          lineCount: metadata.lineCount,
          sha256: metadata.sha256 || ""
        }
      });

      await kgwBridgeDispatchClipboardWriteV1(net, buffer.normalizedText, metadata);
      kgwBridgeFlashLogActionButtonV29(button, kgwBridgeTranslateRuntimeV29("log.copied", "Copied"));
      kgwBridgeSetClipboardStatusV1(net, kgwBridgeTranslateRuntimeV29("log.copied", "Copied"), "ok");
      kgwBridgeStartTraceFrontendV1("frontend.copy_log_succeeded", {
        network: net,
        action: "copy-log",
        result: "ok",
        details: {
          runtimeRole: metadata.runtimeRole,
          bridgeInstanceId: metadata.bridgeInstanceId,
          characterCount: metadata.characterCount,
          lineCount: metadata.lineCount,
          sha256: metadata.sha256 || "",
          userFeedbackDisplayed: true
        }
      });
    } catch (error) {
      kgwBridgeCopyLogFailureV1(net, button, error, {
        bridgeInstanceId: instanceId,
        belongsToLiveBridgeMonitor
      });
    } finally {
      if (button) {
        button.disabled = originalDisabled;
        delete button.dataset.kgwCopyLogInFlightV1;
      }
    }
    return;
  }

  if (action === "clear-log") {
    kgwBridgeClearRawLogBufferV1(net, "bridge", kgwBridgeActiveRawLogInstanceIdV1(net));
    kgwBridgeDispatchRuntimeLogClearV1(net, "bridge").catch(() => {});
    kgwBridgeFlashLogActionButtonV29(button, kgwBridgeTranslateRuntimeV29("log.deleted", "Deleted"));
  }
  kgwBridgeSmallOwnerTraceR44D(net, String(action || "log-action"), "r44d-owner-complete", {});
}
/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29_END */

function installActions(root) {
  if (!root.dataset.kgwBridgePortConflictValidationOwnerR33) {
    root.dataset.kgwBridgePortConflictValidationOwnerR33 = "1";

    root.addEventListener("input", (event) => {
      const target = event && event.target;
      const net = target && target.dataset ? (target.dataset.net || target.dataset.network || "") : "";
      const hay = [
        target && target.id,
        target && target.name,
        target && target.dataset && target.dataset.bridgeInstanceField,
        target && target.dataset && target.dataset.bridgeSetting
      ].map((value) => String(value || "").toLowerCase()).join(" ");

      if (/port|prom|listen|rpc|dashboard|kaspad|instance/.test(hay)) {
        bridgeSchedulePortConflictValidationR33(net, "input");
        bridgeSchedulePortAutofixRefreshR37(net, "input");
      }
    });

    root.addEventListener("change", (event) => {
      const target = event && event.target;
      const net = target && target.dataset ? (target.dataset.net || target.dataset.network || "") : "";
      const hay = [
        target && target.id,
        target && target.name,
        target && target.dataset && target.dataset.bridgeInstanceField,
        target && target.dataset && target.dataset.bridgeSetting
      ].map((value) => String(value || "").toLowerCase()).join(" ");

      if (/port|prom|listen|rpc|dashboard|kaspad|instance/.test(hay)) {
        bridgeSchedulePortConflictValidationR33(net, "change");
        bridgeSchedulePortAutofixRefreshR37(net, "change");
      }
    });

    window.setTimeout(() => bridgeValidateAllPortConflictStatesR33("install"), 100);
  }

  if (!root.dataset.kgwBridgePortAutofixOwnerR37) {
    root.dataset.kgwBridgePortAutofixOwnerR37 = "1";

    bridgeInstallPortAutofixButtonR37(root);

    root.addEventListener("click", (event) => {
      const button = event.target && event.target.closest('[data-bridge-action="auto-fix-ports-r37"]');
      if (!button || !root.contains(button)) return;

      event.preventDefault();
      event.stopPropagation();

      const net = button.dataset.net || "";
      const result = bridgeApplyPortAutofixR37(net);

      button.textContent = result.changed ? "Fixed " + String(result.changed) + " Port(s)" : "No Fix Needed";
      window.setTimeout(() => bridgeRefreshPortAutofixButtonsR37("button-feedback"), 1200);
    });

    window.setTimeout(() => bridgeRefreshPortAutofixButtonsR37("install"), 120);
  }

  if (!root.dataset.kgwBridgeInstancesCommandCheckboxOwnerR13B) {
    root.dataset.kgwBridgeInstancesCommandCheckboxOwnerR13B = "1"; // KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_ACTION_R13B

    root.addEventListener("change", (event) => {
      const include = event.target.closest("[data-bridge-instance-command-option-toggle-r13b]");
      if (include && root.contains(include)) {
        kgwBridgeSetInstanceCommandOptionR13B(
          include.dataset.net,
          include.dataset.instanceId,
          include.dataset.bridgeInstanceCommandOptionToggleR13B,
          include.checked
        );
      }
    });
  }
  if (!root.dataset.kgwBridgeCommandComposerInlineOwnerR7) {
    root.dataset.kgwBridgeCommandComposerInlineOwnerR7 = "1";

    /* KGW_BRIDGE_COMMAND_CHECKBOX_FIRST_CLICK_FIX_TRACE_PATCH_R31
     Native checkbox first-click fix:
     - pointerdown/click/change traces are scoped to this existing Bridge root owner.
     - native checkbox clicks are not preventDefault() blocked.
     - checked state is committed from the change event.
     - non-checkbox fallback keeps the legacy click toggle path.
   */
    root.addEventListener("pointerdown", (event) => {
      const toggle = event.target.closest("[data-bridge-command-option-toggle-r7]");
      if (!toggle || !root.contains(toggle)) return;

      kgwBridgeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-bridge-command-checkbox-pointerdown", {
        patch: "R31",
        owner: "bridge-command-composer-r7",
        option: String(toggle.dataset.bridgeCommandOptionToggleR7 || ""),
        tag: String(toggle.tagName || ""),
        type: String(toggle.type || ""),
        checkedBefore: Boolean(toggle.checked),
        trusted: Boolean(event && event.isTrusted)
      });
    });

    root.addEventListener("change", (event) => {
      const toggle = event.target.closest("[data-bridge-command-option-toggle-r7]");
      if (!toggle || !root.contains(toggle)) return;

      const net = toggle.dataset.net;
      const option = toggle.dataset.bridgeCommandOptionToggleR7;
      const enabled = Boolean(toggle.checked);

      kgwBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r31-bridge-command-checkbox-change-begin", {
        patch: "R31",
        owner: "bridge-command-composer-r7",
        option: String(option || ""),
        checked: enabled,
        trusted: Boolean(event && event.isTrusted)
      });

      try {
        if (typeof kgwBridgeCommandInlineStateR7 === "function") {
          const state = kgwBridgeCommandInlineStateR7(net);
          state[String(option)] = enabled;
          updateCommand(net);
          if (typeof kgwBridgeRefreshInlineCommandTogglesR7 === "function") {
            kgwBridgeRefreshInlineCommandTogglesR7(net);
          }
        } else if (typeof kgwBridgeToggleCommandOptionR7 === "function") {
          kgwBridgeToggleCommandOptionR7(net, option);
        }

        queueMicrotask(() => {
          kgwBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r31-bridge-command-checkbox-change-after-microtask", {
            patch: "R31",
            owner: "bridge-command-composer-r7",
            option: String(option || ""),
            checkedAfter: Boolean(toggle.checked)
          });
        });
      } catch (error) {
        kgwBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r31-bridge-command-checkbox-change-failed", {
          patch: "R31",
          owner: "bridge-command-composer-r7",
          option: String(option || ""),
          message: error && error.message ? error.message : String(error)
        });
      }
    });

    root.addEventListener("click", (event) => {
      const toggle = event.target.closest("[data-bridge-command-option-toggle-r7]");
      if (toggle && root.contains(toggle)) {
        const isNativeCheckbox = toggle.matches && toggle.matches("input[type='checkbox']");

        kgwBridgeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-bridge-command-checkbox-click", {
          patch: "R31",
          owner: "bridge-command-composer-r7",
          option: String(toggle.dataset.bridgeCommandOptionToggleR7 || ""),
          tag: String(toggle.tagName || ""),
          type: String(toggle.type || ""),
          isNativeCheckbox: Boolean(isNativeCheckbox),
          checkedAtClick: Boolean(toggle.checked),
          trusted: Boolean(event && event.isTrusted)
        });

        if (isNativeCheckbox) {
          event.stopPropagation();
          queueMicrotask(() => {
            kgwBridgeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-bridge-command-checkbox-click-after-microtask", {
              patch: "R31",
              owner: "bridge-command-composer-r7",
              option: String(toggle.dataset.bridgeCommandOptionToggleR7 || ""),
              checkedAfter: Boolean(toggle.checked)
            });
          });
          return;
        }

        event.preventDefault();
        event.stopPropagation();
        kgwBridgeToggleCommandOptionR7(toggle.dataset.net, toggle.dataset.bridgeCommandOptionToggleR7);
      }
    });

    root.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      const toggle = event.target.closest("[data-bridge-command-option-toggle-r7]");
      if (toggle && root.contains(toggle)) {
        event.preventDefault();
        event.stopPropagation();
        kgwBridgeToggleCommandOptionR7(toggle.dataset.net, toggle.dataset.bridgeCommandOptionToggleR7);
      }
    });
  }
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

    if (target.matches("[data-bridge-network-enabled]")) {
      const profile = kgwBridgeNetworkProfile(net);
      let enabled = Boolean(target.checked);

      if (enabled && profile?.experimental) {
        const confirmed = window.confirm(
          "Testnet 12 is experimental and uses a separate non-production runtime. Enable it only for isolated testing. Continue?"
        );
        if (!confirmed) {
          enabled = false;
          target.checked = false;
        }
      }

      kgwBridgeSetNetworkEnabled(net, enabled);
      kgwBridgeR51SetRuntimeButtons(net, false);

      if (!enabled) {
        void runBridgeIntegratedAction("stop", net);
      }
    }

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
      kgwBridgeRenderRawLogBufferV1(net, "bridge", String(activeInstance[net] || ""));
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
          kgwBridgeR51SetRuntimeButtons(net, false);
          kgwBridgeSetRuntimeErrorV1(net, error && error.message ? error.message : String(error));
          kgwBridgeSetRuntimeActivityV1(net, "Bridge " + action + " failed.");
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
  BRIDGE_NETWORKS.forEach((net) => kgwBridgeR51SetRuntimeButtons(net.key, false));
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


// KGW_BRIDGE_AUTOFIX_BUTTON_INITIAL_LABEL_R111G
try { kgwBridgeAutofixButtonInitialLabelR111G(document); } catch (_) {}
