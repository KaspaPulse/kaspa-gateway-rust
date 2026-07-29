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


/* KGW_NODE_CLICK_SAVE_CHECKBOX_TRACE_PATCH_R29B
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
    tab: "node",
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
    tab: "node",
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

const KGW_START_TRACE_COMMAND_V1 = "kgw_start_trace_frontend_v1";

function kgwStartTraceSafeTextR1(value, fallback = "") {
  const text = String(value ?? "").replace(/[\r\n\t]+/g, " ").trim();
  return (text || fallback).slice(0, 220);
}

function kgwStartTraceSafeDetailsR1(details) {
  const source = details && typeof details === "object" ? details : {};
  const blocked = /(secret|token|private|mnemonic|wallet|address|commandPreview|completeCommand|arguments|appDir|path|rpcEndpoint|stratum)/i;
  const out = {};

  for (const [key, value] of Object.entries(source)) {
    if (blocked.test(key)) {
      out[key] = "[redacted]";
      continue;
    }

    if (Array.isArray(value)) {
      out[key] = value.slice(0, 24).map((item) => kgwStartTraceSafeTextR1(item));
    } else if (value && typeof value === "object") {
      out[key] = kgwStartTraceSafeDetailsR1(value);
    } else if (typeof value === "boolean" || typeof value === "number") {
      out[key] = value;
    } else {
      out[key] = kgwStartTraceSafeTextR1(value);
    }
  }

  return out;
}

function kgwStartTraceTauriShapeR1(adapterName = "") {
  const tauri = window.__TAURI__;
  const keys = (value) => value && typeof value === "object" ? Object.keys(value).sort().slice(0, 24) : [];

  return {
    adapter: kgwStartTraceSafeTextR1(adapterName, "missing"),
    hasGlobalTauri: Boolean(tauri),
    globalKeys: keys(tauri),
    coreKeys: keys(tauri?.core),
    tauriKeys: keys(tauri?.tauri),
    hasCoreInvoke: typeof tauri?.core?.invoke === "function",
    hasTauriInvoke: typeof tauri?.tauri?.invoke === "function",
    hasRootInvoke: typeof tauri?.invoke === "function",
    expectedConfiguredGlobal: "window.__TAURI__.core.invoke"
  };
}

function kgwResolvePublicTauriInvokeR1() {
  const tauri = window.__TAURI__;
  const candidates = [
    { adapter: "window.__TAURI__.core.invoke", owner: tauri?.core, invoke: tauri?.core?.invoke },
    { adapter: "window.__TAURI__.tauri.invoke", owner: tauri?.tauri, invoke: tauri?.tauri?.invoke },
    { adapter: "window.__TAURI__.invoke", owner: tauri, invoke: tauri?.invoke }
  ];

  for (const candidate of candidates) {
    if (typeof candidate.invoke === "function") {
      return {
        adapter: candidate.adapter,
        invoke: candidate.invoke.bind(candidate.owner),
        shape: kgwStartTraceTauriShapeR1(candidate.adapter)
      };
    }
  }

  return {
    adapter: "missing",
    invoke: null,
    shape: kgwStartTraceTauriShapeR1("missing")
  };
}

function kgwStartTraceFrontendR1(stage, options = {}) {
  const resolved = kgwResolvePublicTauriInvokeR1();
  if (typeof resolved.invoke !== "function") return false;

  const network = kgwStartTraceSafeTextR1(options.network || options.net, "unknown");
  const action = kgwStartTraceSafeTextR1(options.action, "unknown");
  const result = kgwStartTraceSafeTextR1(options.result, "observed");
  const details = kgwStartTraceSafeDetailsR1({
    ...(options.details && typeof options.details === "object" ? options.details : {}),
    invokeAdapter: resolved.adapter
  });

  resolved.invoke(KGW_START_TRACE_COMMAND_V1, {
    stage: kgwStartTraceSafeTextR1(stage, "frontend.unknown"),
    network,
    action,
    result,
    details: JSON.stringify(details)
  }).catch(function (error) {
    console.error("[KGW_START_TRACE_FRONTEND_FAILED]", error && error.message ? error.message : String(error));
  });

  return true;
}

function kgwNodeClipboardCharacterCountV1(text) {
  return Array.from(String(text ?? "")).length;
}

function kgwNodeClipboardLineCountV1(text) {
  const value = String(text ?? "");
  return value ? value.split("\n").length : 0;
}

function kgwNodeNormalizeClipboardLineEndingsV1(text) {
  return String(text ?? "").replace(/\r\n/g, "\n").replace(/\r/g, "\n").replace(/\n/g, "\r\n");
}

async function kgwNodeSha256HexV1(text) {
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

function kgwNodeClipboardSafeErrorV1(error) {
  const text = String(error && error.message ? error.message : error || "clipboard write failed")
    .replace(/[\r\n\t]+/g, " ")
    .trim();
  if (/(secret|token|private|mnemonic|wallet|address)/i.test(text)) {
    return "clipboard write failed with a sensitive error";
  }
  return (text || "clipboard write failed").slice(0, 360);
}

function kgwNodeClipboardPlaceholderTextV1(net) {
  const profile = NODE_NETWORKS.find((item) => item.key === net);
  return `${profile?.label || net} log is empty.`;
}

function kgwNodeLogEmptyStateV1(net) {
  return document.getElementById("node-" + net + "-logEmpty");
}

function kgwNodeClipboardStatusElementV1(net) {
  const out = kgwNodeLogOutputV29(net);
  const toolbar = out?.closest?.('[data-node-inner-panel="log"]')?.querySelector?.(".node-v6-log-toolbar");
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

function kgwNodeSetClipboardStatusV1(net, message, state = "info") {
  const status = kgwNodeClipboardStatusElementV1(net);
  if (!status) return false;

  status.textContent = String(message || "");
  status.dataset.state = String(state || "info");
  status.hidden = !status.textContent;
  return true;
}

function kgwNodeReadClipboardRawLogBufferV1(net) {
  const out = kgwNodeLogOutputV29(net);
  const tag = String(out?.tagName || "").toUpperCase();
  const readsValue = tag === "TEXTAREA" || tag === "INPUT";
  const rawText = String(out ? (readsValue ? out.value : out.textContent) : "");
  const placeholder = kgwNodeClipboardPlaceholderTextV1(net);
  const isPlaceholder = rawText.trim() === placeholder.trim();
  const normalizedText = kgwNodeNormalizeClipboardLineEndingsV1(rawText);

  return {
    out,
    rawText,
    normalizedText,
    isPlaceholder,
    characterCount: kgwNodeClipboardCharacterCountV1(normalizedText),
    lineCount: kgwNodeClipboardLineCountV1(normalizedText)
  };
}

async function kgwNodeDispatchClipboardWriteV1(net, text, metadata) {
  const resolved = kgwResolvePublicTauriInvokeR1();
  if (typeof resolved.invoke !== "function") {
    throw new Error("Tauri invoke API is not available. Expected window.__TAURI__.core.invoke from Tauri 2 with withGlobalTauri enabled.");
  }

  kgwStartTraceFrontendR1("frontend.copy_log_dispatched", {
    network: net,
    action: "copy-log",
    result: "dispatched",
    details: {
      commandName: "kgw_copy_text_to_clipboard_v1",
      implementation: "native-tauri-command",
      runtimeRole: metadata.runtimeRole || "node",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || "",
      payloadFieldCount: 7
    }
  });

  return await invokeWithTimeout(
    resolved.invoke,
    "kgw_copy_text_to_clipboard_v1",
    {
      network: net,
      runtimeRole: metadata.runtimeRole || "node",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      text,
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || ""
    },
    KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
  );
}

function kgwNodeTraceNetworkFromElementR1(element, root = document.getElementById("kaspa-node")) {
  const carrier = element?.closest?.("[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel]");
  const raw = [
    element?.dataset?.net,
    element?.dataset?.network,
    carrier?.dataset?.net,
    carrier?.dataset?.network,
    carrier?.dataset?.nodeNetworkPanel,
    carrier?.id,
    carrier?.className
  ].filter(Boolean).join(" ").toLowerCase();

  if (raw.includes("testnet12") || raw.includes("tn12")) return "testnet12";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";

  const active = kgwNodeTraceActiveNetworkR1(root);
  return active || "mainnet";
}

function kgwNodeTraceActiveNetworkR1(root = document.getElementById("kaspa-node")) {
  if (!root) return "";

  const panels = Array.from(root.querySelectorAll("[data-node-network-panel]"));
  const activePanel = panels.find((panel) =>
    !panel.hidden &&
    (panel.classList.contains("active") || panel.dataset.active === "true")
  );
  if (activePanel?.dataset?.nodeNetworkPanel) return activePanel.dataset.nodeNetworkPanel;

  const tab = Array.from(root.querySelectorAll("[data-node-network-tab]")).find((item) =>
    item.classList.contains("active") ||
    item.getAttribute("aria-selected") === "true" ||
    item.dataset.active === "true"
  );
  return tab?.dataset?.nodeNetworkTab || "";
}

function kgwNodeTraceStartButtonStateR1(net) {
  const panel = kgwNodeR51Panel(net);
  const start = panel?.querySelector?.('[data-node-action="start"][data-net="' + net + '"]');
  const stop = panel?.querySelector?.('[data-node-action="stop"][data-net="' + net + '"]');

  return {
    startRendered: Boolean(start),
    startDisabled: Boolean(start?.disabled),
    stopRendered: Boolean(stop),
    stopDisabled: Boolean(stop?.disabled)
  };
}

function kgwNodeInstallStartTraceDocumentClickObserverR1(root) {
  if (window.__kgwStartTraceDocumentClickObserverR1 === true) return;
  window.__kgwStartTraceDocumentClickObserverR1 = true;

  document.addEventListener("click", function (event) {
    const button = event.target?.closest?.("[data-node-action]");
    const nodeRoot = root || document.getElementById("kaspa-node");
    if (!button || !nodeRoot || !nodeRoot.contains(button)) return;

    const action = String(button.dataset.nodeAction || "").trim();
    const network = kgwNodeTraceNetworkFromElementR1(button, nodeRoot);
    const activeNetwork = kgwNodeTraceActiveNetworkR1(nodeRoot);
    const belongsToSettings = Boolean(button.closest('[data-node-inner-panel="settings"]'));
    const belongsToLiveNodeMonitor = Boolean(button.closest('[data-node-inner-panel="log"]'));

    if (action === "copy-log") {
      kgwStartTraceFrontendR1("frontend.copy_log_click_observed", {
        network,
        action: "copy-log",
        result: "observed",
        details: {
          trusted: Boolean(event && event.isTrusted),
          belongsToSettings,
          belongsToLiveNodeMonitor,
          selectedNetwork: activeNetwork,
          buttonDisabled: Boolean(button.disabled),
          inFlight: button.dataset.kgwCopyLogInFlightV1 === "1"
        }
      });
      return;
    }

    if (action !== "start" && action !== "stop") return;

    kgwStartTraceFrontendR1("frontend.capture_click_observed", {
      network,
      action,
      result: "observed",
      details: {
        trusted: Boolean(event && event.isTrusted),
        belongsToSettings,
        belongsToLiveNodeMonitor,
        selectedNetwork: activeNetwork,
        buttonDisabled: Boolean(button.disabled),
        buttonAction: action
      }
    });
  }, true);
}

function kgwNodeTraceRenderedStartControlsR1(root) {
  for (const profile of NODE_NETWORKS) {
    const net = profile.key;
    const settingsPanel = root.querySelector('[data-node-network-panel="' + net + '"] [data-node-inner-panel="settings"]');
    const start = settingsPanel?.querySelector?.('[data-node-action="start"][data-net="' + net + '"]');

    kgwStartTraceFrontendR1("frontend.settings_subtab_rendered", {
      network: net,
      action: "render",
      result: settingsPanel ? "ok" : "missing",
      details: {
        belongsToSettings: Boolean(settingsPanel),
        selectedNetwork: kgwNodeTraceActiveNetworkR1(root)
      }
    });
    kgwStartTraceFrontendR1("frontend.start_control_rendered", {
      network: net,
      action: "start",
      result: start ? "ok" : "missing",
      details: {
        belongsToSettings: Boolean(start && settingsPanel && settingsPanel.contains(start)),
        startDisabled: Boolean(start?.disabled)
      }
    });
  }
}

function kgwNodeRuntimeActionForCommandR1(command) {
  if (command === "kgw_kgw_apply_node_settings_v1") return "start";
  if (command === "kgw_kgw_disable_network_v1") return "stop";
  return "runtime";
}

function kgwNodeSmallOwnerTraceR44D(net, action, phase, details) {
  try {
    const safeNet = String(net || "unknown");
    const safeAction = String(action || "small-owner");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "node",
      net: safeNet,
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_SMALL_NODE_BRIDGE_TRACE_PATCH_R44D",
        existingOwner: "node-small-owner-functions",
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

/* Canonical isolated node runtime paths.
 * Each network owns a separate database below:
 * %LOCALAPPDATA%\KaspaGateway\nodes\<network>
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

function kgwNodeRustyKaspaLocalAppDataRootR5(paths = {}, net = "mainnet") {
  const appRoot = kgwNodeJoinPathR5(kgwNodeExtractUserLocalAppDataR5(paths), "KaspaGateway");
  const nodesRoot = kgwNodeJoinPathR5(appRoot, "nodes");
  return kgwNodeJoinPathR5(nodesRoot, String(net || "mainnet"));
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
  const rustyRoot = kgwNodeRustyKaspaLocalAppDataRootR5(pathHints, net);

  const values = {
    appDir: rustyRoot,
    logDir: kgwNodeJoinPathR5(rustyRoot, "logs"),
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
  { key: "mainnet", label: "MAINNET", testnet: false, netsuffix: "", enabledByDefault: true, runtime: "Official stable v2.0.1" },
  { key: "testnet10", label: "TESTNET10", testnet: true, netsuffix: "10", enabledByDefault: true, runtime: "Official stable v2.0.1" },
  { key: "testnet12", label: "TESTNET12", testnet: true, netsuffix: "12", enabledByDefault: false, experimental: true, runtime: "Experimental TN12 build" }
];

function kgwNodeNetworkPolicyKey(net) {
  return `kgw.node.network.enabled.${String(net || "unknown")}`;
}

function kgwNodeNetworkProfile(net) {
  return NODE_NETWORKS.find((item) => item.key === net) || null;
}

function kgwNodeNetworkEnabled(net) {
  const profile = kgwNodeNetworkProfile(net);
  try {
    const stored = localStorage.getItem(kgwNodeNetworkPolicyKey(net));
    if (stored === "1") return true;
    if (stored === "0") return false;
  } catch (_) {}
  return profile ? profile.enabledByDefault !== false : false;
}

function kgwNodeSetNetworkEnabled(net, enabled) {
  try {
    localStorage.setItem(kgwNodeNetworkPolicyKey(net), enabled ? "1" : "0");
  } catch (_) {}
}

function kgwNodeNetworkPolicyMessage(net) {
  const profile = kgwNodeNetworkProfile(net);
  if (!profile) return "";
  if (profile.experimental) {
    return "Experimental network. Disabled by default and requires explicit opt-in.";
  }
  return `${profile.runtime}. RPC remains loopback-only and data is isolated per network.`;
}

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
  if (!kgwNodeCommandShouldIncludeR7(net, name)) return; // KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7

  if (c(net, name)) lines.push(flag);
}

function addValue(lines, net, name, flag) {
  if (!kgwNodeCommandShouldIncludeR7(net, name)) return; // KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7

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


// KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7
const KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7 = "KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7";

function kgwNodeCommandInlineStateKeyR7(net) {
  return String(net || "mainnet");
}

function kgwNodeCommandInlineStateR7(net) {
  const key = kgwNodeCommandInlineStateKeyR7(net);
  window.__kgwNodeCommandComposerInlineR7 = window.__kgwNodeCommandComposerInlineR7 || {};
  window.__kgwNodeCommandComposerInlineR7[key] = window.__kgwNodeCommandComposerInlineR7[key] || {};
  return window.__kgwNodeCommandComposerInlineR7[key];
}

function kgwNodeCommandOptionEnabledR7(net, name) {
  const state = kgwNodeCommandInlineStateR7(net);
  return state[String(name)] !== false;
}

function kgwNodeCommandShouldIncludeR7(net, name) {
  return kgwNodeCommandOptionEnabledR7(net, name);
}

function kgwNodeCommandInlineToggleR7(net, name) {
  const enabled = kgwNodeCommandOptionEnabledR7(net, name);
  const label = enabled ? "Included" : "Excluded";
  return `<input type="checkbox" class="kgw-command-option-checkbox-r9" data-node-command-option-toggle-r7="${esc(String(name))}" data-net="${esc(String(net))}" ${enabled ? "checked" : ""} aria-label="${enabled ? "Included in command" : "Excluded from command"}" title="${enabled ? "Included in command" : "Excluded from command"}">`; // KGW_NODE_COMMAND_COMPOSER_CHECKBOX_ONLY_R9
}

function kgwNodeRefreshInlineCommandTogglesR7(net) {
  document.querySelectorAll(`[data-node-command-option-toggle-r7][data-net="${CSS.escape(String(net))}"]`).forEach((el) => {
    const name = el.dataset.nodeCommandOptionToggleR7;
    const enabled = kgwNodeCommandOptionEnabledR7(net, name);
    el.checked = enabled;
    el.setAttribute("aria-label", enabled ? "Included in command" : "Excluded from command");
    el.setAttribute("title", enabled ? "Included in command" : "Excluded from command");
    el.classList.toggle("is-on", enabled);
    el.classList.toggle("is-off", !enabled);
  });
}

function kgwNodeToggleCommandOptionR7(net, name) {
  const state = kgwNodeCommandInlineStateR7(net);
  const key = String(name);
  state[key] = state[key] === false;
  kgwNodeRefreshInlineCommandTogglesR7(net);
  updateCommand(net);
}


function cardInput(net, name, label, value = "", placeholder = "", span2 = false) {
  return `
    <div class="node-v6-card${span2 ? " span2" : ""}">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwNodeCommandInlineToggleR7(net, name)}
        <span class="kgw-command-option-title-text-r8e">${esc(label)}</span>
      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->
      <input id="${id(net, name)}" data-testid="kgw-node-field-${esc(net)}-${esc(name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function cardSelect(net, name, label, options, value = "", span2 = false) {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="node-v6-card${span2 ? " span2" : ""}">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwNodeCommandInlineToggleR7(net, name)}
        <span class="kgw-command-option-title-text-r8e">${esc(label)}</span>
      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->
      <select id="${id(net, name)}" data-testid="kgw-node-field-${esc(net)}-${esc(name)}">${opts}</select>
    </div>`;
}

function cardCheck(net, name, label, checked = false, span2 = false) {
  return `
    <label class="node-v6-card check${span2 ? " span2" : ""}">
      <input id="${id(net, name)}" data-testid="kgw-node-field-${esc(net)}-${esc(name)}" type="checkbox"${checked ? " checked" : ""}>
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
    checkbox.addEventListener("change", (event) => {
      kgwNodeSmallOwnerTraceR44D(net, "log-autoscroll", "r51b3-node-log-autoscroll-change", {
        patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
        trusted: Boolean(event && event.isTrusted),
        controlId: String(controlId || ""),
        checked: Boolean(checkbox.checked)
      });
      kgwNodeSetLogAutoScrollR27(net, checkbox.checked);
    });

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

const KGW_NODE_RAW_LOG_BUFFER_LIMIT_V1 = 4096;
const KGW_NODE_RAW_LOG_BUFFERS_V1 = new Map();

function kgwNodeRawLogBufferKeyV1(net, role = "node") {
  return String(net || "").trim().toLowerCase() + ":" + String(role || "node").trim().toLowerCase();
}

function kgwNodeRawLogBufferV1(net, role = "node") {
  const key = kgwNodeRawLogBufferKeyV1(net, role);
  if (!KGW_NODE_RAW_LOG_BUFFERS_V1.has(key)) {
    KGW_NODE_RAW_LOG_BUFFERS_V1.set(key, { records: new Map() });
  }
  return KGW_NODE_RAW_LOG_BUFFERS_V1.get(key);
}

function kgwNodeRawLogTextHasTransportWrapperV1(text) {
  const value = String(text ?? "");
  if (!value) return false;

  const forbidden = [
    "kgw_raw_process_log_v1",
    "[KGW_CHILD_STDOUT]",
    "[KGW_CHILD_STDERR]",
    "diagnostic_transport_record",
    ";source=self-worker;",
    ";runtime_role=",
    ";received_ms="
  ];
  if (forbidden.some((marker) => value.toLowerCase().includes(marker.toLowerCase()))) return true;

  const trimmed = value.trimStart();
  return trimmed.startsWith("{")
    && /"stage"\s*:/.test(trimmed)
    && /"network"\s*:/.test(trimmed)
    && (/"source"\s*:/.test(trimmed) || /"eventKind"\s*:\s*"diagnostic_transport_record"/.test(trimmed));
}

function kgwNodeNormalizeRawLogEntryV1(entry, expectedNet, expectedRole = "node") {
  if (!entry || typeof entry !== "object") return null;

  const rawTextValue = entry.rawText ?? entry.raw_text ?? entry.line;
  if (rawTextValue === undefined || rawTextValue === null) return null;
  if (kgwNodeRawLogTextHasTransportWrapperV1(rawTextValue)) return null;

  const sequence = Number(entry.sequence);
  if (!Number.isSafeInteger(sequence) || sequence < 0) return null;

  const network = String(entry.network || expectedNet || "").trim().toLowerCase();
  const runtimeRole = String(entry.runtimeRole || entry.runtime_role || expectedRole || "node").trim().toLowerCase();
  const stream = String(entry.stream || "").trim().toLowerCase();

  if (network !== String(expectedNet || "").trim().toLowerCase()) return null;
  if (runtimeRole !== String(expectedRole || "node").trim().toLowerCase()) return null;
  if (stream !== "stdout" && stream !== "stderr") return null;

  return Object.freeze({
    sequence,
    network,
    runtimeRole,
    stream,
    receivedMs: Number(entry.receivedMs ?? entry.received_ms ?? 0) || 0,
    rawText: String(rawTextValue)
  });
}

function kgwNodeTrimRawLogBufferV1(buffer) {
  const ordered = Array.from(buffer.records.keys()).sort((a, b) => a - b);
  while (ordered.length > KGW_NODE_RAW_LOG_BUFFER_LIMIT_V1) {
    const sequence = ordered.shift();
    buffer.records.delete(sequence);
  }
}

function kgwNodeVisibleRawLogTextV1(net, role = "node") {
  const buffer = kgwNodeRawLogBufferV1(net, role);
  return Array.from(buffer.records.values())
    .sort((a, b) => a.sequence - b.sequence)
    .map((entry) => entry.rawText)
    .join("\n");
}

function kgwNodeRenderRawLogBufferV1(net, role = "node") {
  const out = byId(id(net, "logOutput"));
  if (!out) return false;

  const text = kgwNodeVisibleRawLogTextV1(net, role);
  out.textContent = text;

  const empty = kgwNodeLogEmptyStateV1(net);
  if (empty) empty.hidden = text.length > 0;

  if (kgwNodeLogAutoScrollEnabledR27(net)) out.scrollTop = out.scrollHeight;
  return true;
}

function kgwNodeApplyRuntimeLogReportV1(net, role, report) {
  const entries = Array.isArray(report?.entries) ? report.entries : [];
  const buffer = kgwNodeRawLogBufferV1(net, role);
  let accepted = 0;

  for (const entry of entries) {
    const normalized = kgwNodeNormalizeRawLogEntryV1(entry, net, role);
    if (!normalized || buffer.records.has(normalized.sequence)) continue;
    buffer.records.set(normalized.sequence, normalized);
    accepted += 1;
  }

  if (accepted > 0) {
    kgwNodeTrimRawLogBufferV1(buffer);
  }

  kgwNodeRenderRawLogBufferV1(net, role);
  return accepted;
}

function kgwNodeClearRawLogBufferV1(net, role = "node") {
  kgwNodeRawLogBufferV1(net, role).records.clear();
  kgwNodeRenderRawLogBufferV1(net, role);
}

async function kgwNodeDispatchRuntimeLogClearV1(net, role = "node") {
  const resolved = kgwResolvePublicTauriInvokeR1();
  if (typeof resolved.invoke !== "function") return null;

  return await invokeWithTimeout(
    resolved.invoke,
    "kgw_kgw_runtime_clear_logs_v1",
    { network: net, runtimeRole: role },
    KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
  );
}

function appendLog(net, message) {
  // Raw monitor text is driven by typed runtime log reports. This legacy hook is
  // intentionally inert so UI status strings cannot become fabricated raw lines.
  void net;
  void message;
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

/* KGW_NODE_FLAT_SIX_COLUMN_OWNER_R101K
 * Existing Node settings owner refinement.
 * Flatten section bodies into one compact six-column field grid per tab.
 */
function kgwNodeFlatSectionFieldsR101K(html) {
  return String(html || "")
    .replace(/^\s*<div\s+class=["']node-v6-grid["']>\s*/i, "")
    .replace(/\s*<\/div>\s*$/i, "");
}

function kgwNodeFlatGroupBodyR101K(sections) {
  return sections.map(([, body]) => kgwNodeFlatSectionFieldsR101K(body)).join("\n");
}

function renderSections(net) {
  /* KGW_NODE_GROUPED_SETTINGS_TABS_OWNER_R101G */
  /* KGW_NODE_FLAT_SIX_COLUMN_OWNER_R101K
   * General and Advanced remain the only Node internal settings tabs.
   * Each tab is rendered as one flat compact six-column field grid.
   */
  const groups = [
    ["general", "General", [
      ["runtime", renderRuntime(net)],
      ["network", renderNetwork(net)],
      ["rpc", renderRpc(net)]
    ]],
    ["advanced", "Advanced", [
      ["peers", renderPeers(net)],
      ["database", renderDatabase(net)],
      ["rocksdb", renderRocksDb(net)],
      ["paths", renderPaths(net)]
    ]]
  ];

  const tabs = groups.map(([key, label], index) =>
    `<button type="button" class="node-v6-section-tab node-v6-section-tab--grouped${index === 0 ? " active" : ""}" data-net="${net.key}" data-node-section-tab="${key}">${label}</button>`
  ).join("");

  const panels = groups.map(([groupKey, , sections], groupIndex) => {
    const fields = kgwNodeFlatGroupBodyR101K(sections);
    return `
    <section class="node-v6-section node-v6-section-group node-v6-section-group--flat${groupIndex === 0 ? " active" : ""}" data-net="${net.key}" data-node-section-panel="${groupKey}"${groupIndex === 0 ? "" : " hidden"}>
      <div class="node-v6-flat-six-grid" data-node-flat-grid="${groupKey}">${fields}</div>
    </section>`;
  }).join("");

  return `
    <div class="node-v6-section-tabs node-v6-section-tabs--grouped">${tabs}</div>
    <div class="node-v6-sections node-v6-sections--grouped node-v6-sections--flat">${panels}</div>`;
}

/* KGW_NODE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
 * Default Node inner tab is Live Node Monitor.
 * Last selected inner tab is saved per network.
 */
function kgwNodeInnerTabStorageKeyR101U(net) {
  return `kgw.node.innerTab.${String(net || "unknown")}`;
}

function kgwNodeNormalizeInnerTabR101U(value) {
  return value === "settings" || value === "log" ? value : "log";
}

function kgwNodeResolveInnerTabR101U(net) {
  try {
    return kgwNodeNormalizeInnerTabR101U(localStorage.getItem(kgwNodeInnerTabStorageKeyR101U(net)));
  } catch (_) {
    return "log";
  }
}

function kgwNodeSaveInnerTabR101U(net, selected) {
  const normalized = kgwNodeNormalizeInnerTabR101U(selected);
  try {
    localStorage.setItem(kgwNodeInnerTabStorageKeyR101U(net), normalized);
  } catch (_) {}
  return normalized;
}

function renderNetworkPanel(net, index) {
  /* KGW_NODE_LIVE_MONITOR_TAB_LABEL_ORDER_R101S */
  /* KGW_NODE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
   * Settings is no longer the default inner panel.
   * Default is Live Node Monitor unless a valid saved tab exists for this network.
   */
  const activeInnerTab = kgwNodeResolveInnerTabR101U(net.key);
  const logActive = activeInnerTab === "log";
  const settingsActive = activeInnerTab === "settings";

  return `
    <div class="node-v6-network-panel${index === 0 ? " active" : ""}" data-node-network-panel="${net.key}" data-testid="kgw-node-panel-${net.key}"${index === 0 ? "" : " hidden"}>
      <div class="node-v6-inner-tabs">
        <button type="button" class="node-v6-inner-tab${logActive ? " active" : ""}" data-net="${net.key}" data-node-inner-tab="log" data-testid="kgw-node-live-monitor-${net.key}">Live Node Monitor</button>
        <button type="button" class="node-v6-inner-tab${settingsActive ? " active" : ""}" data-net="${net.key}" data-node-inner-tab="settings" data-testid="kgw-node-settings-${net.key}">Settings</button>
      </div>

      <div class="node-v6-inner-panel${settingsActive ? " active" : ""}" data-net="${net.key}" data-node-inner-panel="settings" data-node-settings-panel="${net.key}"${settingsActive ? "" : " hidden"}>
        <section class="kgw-network-policy${net.experimental ? " is-experimental" : ""}" data-net="${net.key}" data-testid="kgw-node-policy-${net.key}">
          <div>
            <strong>${net.label}</strong>
            <span>${esc(kgwNodeNetworkPolicyMessage(net.key))}</span>
          </div>
          <div class="kgw-network-policy-controls">
            <span id="${id(net.key, "policyStatus")}" class="kgw-network-policy-status">Stopped</span>
            <label>
              <input type="checkbox" data-node-network-enabled="${net.key}" data-testid="kgw-node-policy-enabled-${net.key}" data-net="${net.key}"${kgwNodeNetworkEnabled(net.key) ? " checked" : ""}>
              Enabled
            </label>
          </div>
        </section>

        <section class="node-v6-command">
          <div class="node-v6-command-title">Command Preview</div>
          <textarea id="${id(net.key, "commandPreview")}" readonly spellcheck="false" wrap="soft"></textarea>
          <button type="button" class="node-v6-copy" data-node-action="copy-command" data-net="${net.key}" title="Copy command">⧉</button>
        </section>

        <section class="node-v6-toolbar">
          <div class="node-v6-buttons">
            <button type="button" class="good" data-node-action="start" data-testid="kgw-node-start-${net.key}" data-net="${net.key}">Start</button>
            <button type="button" data-node-action="stop" data-testid="kgw-node-stop-${net.key}" data-net="${net.key}">Stop</button>
          </div>

          <div class="node-v6-status">
            <label><input id="${id(net.key, "startOnLaunch")}" type="checkbox"> Auto-start</label>
            <label><input id="${id(net.key, "autoRestart")}" type="checkbox" checked> Restart</label>
            <span id="${id(net.key, "runtimeStatus")}" class="node-v6-runtime-status-pill" data-state="stopped">Stopped</span>
            <span id="${id(net.key, "runtimeEvidence")}" class="node-v6-runtime-evidence">No process owner</span>
          </div>

          <div id="${id(net.key, "runtimeError")}" class="node-v6-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        ${renderSections(net)}

        <div class="settings-bottom-actions node-settings-bottom-actions">
        <button type="button" data-node-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-node-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-node-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        </div>

      </div>

      <div class="node-v6-inner-panel${logActive ? " active" : ""}" data-net="${net.key}" data-node-inner-panel="log" data-testid="kgw-node-live-panel-${net.key}"${logActive ? "" : " hidden"}>
        <div class="node-v6-log-toolbar">
          <span class="node-v6-log-metadata" data-net="${net.key}">Network: ${net.label} · Source: self-worker · Streams: stdout/stderr</span>
          <button type="button" data-node-action="copy-log" data-testid="kgw-node-copy-log-${net.key}" data-net="${net.key}">Copy Log</button>
          <button type="button" data-node-action="clear-log" data-testid="kgw-node-clear-log-${net.key}" data-net="${net.key}">Clear Log</button>
        </div>
        <div id="${id(net.key, "logEmpty")}" class="node-v6-log-empty" data-node-log-empty="${net.key}">No child stdout/stderr received yet.</div>
        <pre id="${id(net.key, "logOutput")}" class="node-v6-log" data-testid="kgw-node-log-output-${net.key}"></pre>
      </div>
</div>`;
}

function renderAllNetworks(root) {
  const host = root.querySelector("#nodeNetworkPanels");
  if (!host) return;
  host.innerHTML = NODE_NETWORKS.map(renderNetworkPanel).join("");


  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
  setTimeout(window.kgwInstallNodeLogScopedControlsV29, 0);
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



// KGW_NODE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45F
function kgwNodeExplicitTraceR27D(net, action, phase, details) {
  try {
    const safeNet = String(net || "unknown");
    const safeAction = String(action || "internal-navigation");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "node",
      net: safeNet,
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_NODE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45F",
        owner: "node-module-visible-explicit-trace-helper",
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
/* KGW_NODE_LAST_NETWORK_RESTORE_R101W2 */
const KGW_NODE_LAST_NETWORK_KEY_R101W2 = "kgw.node.lastNetwork";
function kgwNodeNormalizeNetworkR101W2(value) {
  const normalized = String(value || "").trim();
  return normalized === "mainnet" || normalized === "testnet10" || normalized === "testnet12" ? normalized : "";
}
function kgwNodeReadLastNetworkR101W2() {
  try { return kgwNodeNormalizeNetworkR101W2(localStorage.getItem(KGW_NODE_LAST_NETWORK_KEY_R101W2)); } catch (_) { return ""; }
}
function kgwNodeSaveLastNetworkR101W2(net) {
  const normalized = kgwNodeNormalizeNetworkR101W2(net);
  if (!normalized) return "";
  try { localStorage.setItem(KGW_NODE_LAST_NETWORK_KEY_R101W2, normalized); } catch (_) {}
  return normalized;
}

function installNetworkTabs(root) {
  /* KGW_NODE_LAST_NETWORK_RESTORE_R101W2 */
  const tabs = Array.from(root.querySelectorAll("[data-node-network-tab]"));
  const panels = Array.from(root.querySelectorAll("[data-node-network-panel]"));

  function selectNodeNetwork(selected, reason = "manual", persist = false) {
    const normalized = kgwNodeNormalizeNetworkR101W2(selected);
    if (!normalized) return;
    if (persist) kgwNodeSaveLastNetworkR101W2(normalized);

    tabs.forEach((item) => {
      const active = item.dataset.nodeNetworkTab === normalized;
      item.classList.toggle("active", active);
      item.setAttribute("aria-selected", active ? "true" : "false");
      item.dataset.active = active ? "true" : "false";
    });

    panels.forEach((panel) => {
      const active = panel.dataset.nodeNetworkPanel === normalized;
      panel.classList.toggle("active", active);
      panel.hidden = !active;
      panel.dataset.active = active ? "true" : "false";
    });

    if (normalized && kgwIsBridgeOwnedNodeLockedR65E(normalized)) {
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(normalized, true, "network-tab-select-" + reason);
      kgwNodeR51SetRuntimeButtons(normalized, false, true);
    }

    kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2("network-tab-" + reason);
  }

  tabs.forEach((tab) => {
    tab.addEventListener("click", (event) => {
      const selected = tab.dataset.nodeNetworkTab;
      kgwNodeExplicitTraceR27D(selected || "unknown", "internal-navigation", "r45d-node-network-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_NODE_LAST_NETWORK_RESTORE_R101W2",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected || ""),
        text: String(tab.textContent || "").trim(),
        persisted: true
      });
      selectNodeNetwork(selected, "click", true);
    });
  });

  const saved = kgwNodeReadLastNetworkR101W2();
  const existingActiveTab = tabs.find((tab) => tab.classList.contains("active") || tab.getAttribute("aria-selected") === "true" || tab.dataset.active === "true");
  const defaultTab = (saved && tabs.find((tab) => tab.dataset.nodeNetworkTab === saved)) || existingActiveTab || tabs.find((tab) => tab.dataset.nodeNetworkTab === "mainnet") || tabs[0];
  if (defaultTab) selectNodeNetwork(defaultTab.dataset.nodeNetworkTab, saved ? "saved-initial" : "initial", false);

  kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2("network-tabs-installed");
  window.kgwNodeSelectNetworkTabR101W2 = (net) => selectNodeNetwork(net, "external", true);
}

function installDelegatedTabs(root) {
  root.addEventListener("click", (event) => {
    const innerTab = event.target.closest("[data-node-inner-tab]");
    if (innerTab) {
      const net = innerTab.dataset.net;
      const selected = kgwNodeSaveInnerTabR101U(net, innerTab.dataset.nodeInnerTab);
      const panel = root.querySelector(`[data-node-network-panel="${net}"]`);

      kgwNodeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-node-inner-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_NODE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected || ""),
        text: String(innerTab.textContent || "").trim(),
        persisted: true
      });

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

      kgwNodeExplicitTraceR27D(net || "unknown", "internal-navigation", "r45d-node-section-tab-click", {
        patch: "KGW_INTERNAL_NAV_TRACE_OWNER_R45D",
        trusted: Boolean(event && event.isTrusted),
        selected: String(selected || ""),
        text: String(sectionTab.textContent || "").trim()
      });

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
  return kgwResolvePublicTauriInvokeR1().invoke;
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

function kgwNodeSetRuntimeNotice(net, state, evidence = "", errorText = null) {
  const status = byId(id(net, "runtimeStatus"));
  const evidenceNode = byId(id(net, "runtimeEvidence"));
  const errorNode = byId(id(net, "runtimeError"));
  const normalizedState = String(state || "Stopped");
  const stateKey = normalizedState.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "stopped";

  if (status) {
    status.textContent = normalizedState;
    status.dataset.state = stateKey;
  }

  if (evidenceNode) {
    evidenceNode.textContent = evidence || "No process owner";
  }

  if (errorNode && errorText !== null && errorText !== undefined) {
    const text = String(errorText || "").trim();
    errorNode.textContent = text;
    errorNode.hidden = !text;
  }
}

function kgwNodeRuntimeEvidence(result) {
  const text = stringifyRuntimeResult(result);
  const fields = parseRuntimeFields(text);
  const pid = String(fields.pid || "").trim();
  const owner = fields.owner || fields.source || "self-worker";
  const role = fields.role || fields.runtime_role || fields.runtimeRole || "node";
  const state = fields.runtime_state || fields.runtimeState || (pid ? "running" : "");

  return {
    text,
    fields,
    pid,
    owner,
    role,
    state,
  };
}

function kgwNodeAssertStartEvidence(net, result) {
  const evidence = kgwNodeRuntimeEvidence(result);
  const responseNetwork = String(evidence.fields.network || "").trim();

  if (/start_blocked=true|start_allowed=false/i.test(evidence.text)) {
    throw new Error(evidence.text);
  }

  if (responseNetwork && responseNetwork !== String(net || "")) {
    throw new Error("Backend start response used the wrong network: " + evidence.text);
  }

  if (!/^[0-9]+$/.test(evidence.pid)) {
    throw new Error("Backend start response did not include process ID evidence: " + evidence.text);
  }

  return evidence;
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
      experimentalNetworkOptIn: net === "testnet12" && kgwNodeNetworkEnabled(net),
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
  const action = kgwNodeRuntimeActionForCommandR1(command);
  const resolved = kgwResolvePublicTauriInvokeR1();

  kgwStartTraceFrontendR1("frontend.invoke_adapter_selected", {
    network: net,
    action,
    result: resolved.invoke ? "selected" : "missing",
    details: {
      commandName: command,
      adapter: resolved.adapter,
      shape: resolved.shape
    }
  });

  if (!resolved.invoke) {
    throw new Error("Tauri invoke API is not available. Expected window.__TAURI__.core.invoke from Tauri 2 with withGlobalTauri enabled.");
  }

  const args = nodeRuntimeArgs(net, command);
  kgwStartTraceFrontendR1("frontend.invoke_dispatched", {
    network: net,
    action,
    result: "dispatched",
    details: {
      commandName: command,
      payloadFieldCount: Object.keys(args).length,
      nodePreviewPresent: Boolean(args.nodeCommandPreview),
      bridgePreviewPresent: Boolean(args.bridgeCommandPreview),
      runtimeRolePresent: Boolean(args.runtimeRole),
      timeoutMs: KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
    }
  });

  try {
    const result = await invokeWithTimeout(resolved.invoke, command, args, KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS);
    const evidence = kgwNodeRuntimeEvidence(result);
    kgwStartTraceFrontendR1("frontend.invoke_resolved", {
      network: net,
      action,
      result: "resolved",
      details: {
        commandName: command,
        hasPid: /^[0-9]+$/.test(evidence.pid),
        owner: evidence.owner,
        role: evidence.role,
        state: evidence.state
      }
    });
    return result;
  } catch (error) {
    kgwStartTraceFrontendR1("frontend.invoke_rejected", {
      network: net,
      action,
      result: "rejected",
      details: {
        commandName: command,
        error: normalizeRuntimeError(error)
      }
    });
    throw error;
  }
}


async function runNodeIntegratedAction(action, net) {
  const commandByAction = {
    start: "kgw_kgw_apply_node_settings_v1",
    stop: "kgw_kgw_disable_network_v1"
  };

  const command = commandByAction[action];
  if (!command) return false;

  const nodeRoot = document.getElementById("kaspa-node");
  const activeNetwork = kgwNodeTraceActiveNetworkR1(nodeRoot);
  const profile = kgwNodeNetworkProfile(net);
  const networkEnabled = kgwNodeNetworkEnabled(net);

  kgwStartTraceFrontendR1("frontend.start_action_identified", {
    network: net,
    action,
    result: "identified",
    details: {
      commandName: command,
      action,
      controlNetwork: net,
      activeNetwork
    }
  });
  kgwStartTraceFrontendR1("frontend.active_network_resolved", {
    network: net,
    action,
    result: activeNetwork && activeNetwork !== net ? "mismatch" : "ok",
    details: {
      controlNetwork: net,
      activeNetwork,
      selectedNetwork: activeNetwork || net
    }
  });

  if (action === "start") {
    const experimentalNetwork = Boolean(profile?.experimental);
    const explicitOptIn = experimentalNetwork && networkEnabled;
    kgwStartTraceFrontendR1("frontend.experimental_opt_in_evaluated", {
      network: net,
      action,
      result: !experimentalNetwork || explicitOptIn ? "allowed" : "blocked",
      details: {
        experimentalNetwork,
        explicitOptIn,
        networkEnabled
      }
    });
  }

  if (action === "start" && !networkEnabled) {
    kgwNodeSetRuntimeNotice(net, "Disabled", "No process owner", "This network is disabled. Enable it in Settings before starting.");
    kgwNodeR51SetRuntimeButtons(net, false, false);
    return true;
  }

  if (action === "start" || action === "stop") {
    const bridgeInprocessLocked = kgwIsBridgeOwnedNodeLockedR65E(net);

    if (bridgeInprocessLocked) {
      kgwNodeR51SetRuntimeButtons(net, false, true);
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(net, true, "action-guard");
      kgwNodeSetRuntimeNotice(net, "Blocked", "Bridge in-process owner", "This network is display-only because Bridge in-process mode owns the node runtime. Stop the bridge first.");
      return true;
    }
  }

  if (!window.__kgwR29NodeInFlight) {
    window.__kgwR29NodeInFlight = new Set();
  }

  const inFlightKey = net + ":" + action;
  if (window.__kgwR29NodeInFlight.has(inFlightKey)) {
    kgwNodeSetRuntimeNotice(net, action === "start" ? "Starting" : "Stopping", "Transition in progress", "A " + action + " request is already in progress for this network.");
    return true;
  }

  window.__kgwR29NodeInFlight.add(inFlightKey);
  KGW_NODE_R51_TRANSITIONS[net] = action === "start" ? "starting" : "stopping";
  kgwNodeSetRuntimeNotice(net, action === "start" ? "Starting" : "Stopping", "Waiting for backend response", "");
  kgwNodeR51SetRuntimeButtons(net, action === "stop", kgwIsBridgeOwnedNodeLockedR65E(net));

  try {
    const result = await invokeNodeIntegratedRuntime(command, net);

    if (action === "start") {
      const evidence = kgwNodeAssertStartEvidence(net, result);
      kgwNodeSetRuntimeNotice(
        net,
        "Running",
        "pid=" + evidence.pid + ";owner=" + evidence.owner + ";role=" + evidence.role,
        ""
      );
      KGW_NODE_R51_TRANSITIONS[net] = "";
      kgwNodeR51SetRuntimeButtons(net, true, kgwIsBridgeOwnedNodeLockedR65E(net));
    } else {
      kgwNodeSetRuntimeNotice(net, "Stopped", "No process owner", "");
      KGW_NODE_R51_TRANSITIONS[net] = "";
      kgwNodeR51SetRuntimeButtons(net, false, kgwIsBridgeOwnedNodeLockedR65E(net));
    }

    return true;
  } catch (error) {
    KGW_NODE_R51_TRANSITIONS[net] = "";
    const errorText = normalizeRuntimeError(error);
    const runningAfterFailure = action === "stop";
    kgwNodeR51SetRuntimeButtons(net, runningAfterFailure, kgwIsBridgeOwnedNodeLockedR65E(net));
    kgwNodeSetRuntimeNotice(net, runningAfterFailure ? "Running" : "Stopped", runningAfterFailure ? "Stop failed" : "No process owner", errorText);
    kgwStartTraceFrontendR1("frontend.button_state_restored_after_failure", {
      network: net,
      action,
      result: "restored",
      details: {
        error: errorText,
        runningAfterFailure,
        state: kgwNodeTraceStartButtonStateR1(net)
      }
    });
    return true;
  } finally {
    KGW_NODE_R51_TRANSITIONS[net] = "";
    window.__kgwR29NodeInFlight.delete(inFlightKey);
  }
}
/* KGW_R51_DIRECT_NODE_LOG_RUNTIME_SETTINGS_OWNER */
const KGW_NODE_R51_STORAGE_PREFIX = "kgw.node.direct.v51.";
const KGW_NODE_R51_LAST_STATUS = {};
const KGW_NODE_R51_LAST_LOGS = {};
const KGW_NODE_R51_LAST_ACTIVITY_NOTICE = {};
const KGW_NODE_R51_TRANSITIONS = {};
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
  values[KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C] = kgwNodeR51ReadCommandOptionsR38C(net);

  for (const field of kgwNodeR51Fields(net)) {
    if (!field.id) continue;

    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }

  kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-read-settings-command-options", {
    patch: "R38C",
    owner: "node-r51-settings-owner",
    commandOptionCount: Object.keys(values[KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C] || {}).length
  });

  return values;
}


/* KGW_NODE_COMMAND_CHECKBOX_PERSISTENCE_PATCH_R38C
 * Persist Node command include/exclude checkboxes by semantic keys, not empty DOM ids.
 * This patches the existing R51 settings persistence owner only.
 */
const KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C = "__kgwNodeCommandOptionsR38C";

function kgwNodeR51ReadCommandOptionsR38C(net) {
  const state = {};
  try {
    const root = document.getElementById("kaspa-node");
    if (!root) return state;

    for (const item of root.querySelectorAll('[data-node-command-option-toggle-r7][data-net="' + String(net || "") + '"]')) {
      const name = String(item.dataset.nodeCommandOptionToggleR7 || "");
      if (!name) continue;
      state[name] = Boolean(item.checked);
    }
  } catch (_) {}
  return state;
}

function kgwNodeR51ApplyCommandOptionsR38C(net, values) {
  try {
    const commandOptions = values && values[KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C];

    if (commandOptions && typeof commandOptions === "object") {
      const state = kgwNodeCommandInlineStateR7(net);
      for (const [name, enabled] of Object.entries(commandOptions)) {
        state[String(name)] = Boolean(enabled);
      }
      kgwNodeRefreshInlineCommandTogglesR7(net);
      updateCommand(net);
    }

    kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restored", {
      patch: "R38C",
      owner: "node-r51-settings-owner",
      commandOptionCount: commandOptions && typeof commandOptions === "object" ? Object.keys(commandOptions).length : 0
    });
  } catch (error) {
    kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restore-failed", {
      patch: "R38C",
      owner: "node-r51-settings-owner",
      message: error && error.message ? error.message : String(error)
    });
  }
}


function kgwNodeR51WriteSettings(net, values) {
  if (!values || typeof values !== "object") return;

  for (const field of kgwNodeR51Fields(net)) {
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

  kgwNodeR51ApplyCommandOptionsR38C(net, values);

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
    }
  }
}

/* KGW_NODE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */


function kgwNodeR51SaveSettings(net) {
  kgwNodeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-begin", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });

  const values = kgwNodeR51ReadSettings(net);
  kgwNodeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-read-settings", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    keyCount: Object.keys(values || {}).length,
    checkboxCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "checkbox").length,
    valueCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "value").length,
    structuredInstanceCount: (values && values.__kgwBridgeStructuredInstancesR26B && Array.isArray(values.__kgwBridgeStructuredInstancesR26B.instances)) ? values.__kgwBridgeStructuredInstancesR26B.instances.length : 0,
    hasActiveStructuredInstance: Boolean(values && values.__kgwBridgeActiveInstanceR26B)
  });

  kgwNodeR51Store("saved:" + net, values);

  const saved = kgwNodeR51Load("saved:" + net);
  kgwNodeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    savedKey: "saved:" + String(net || ""),
    persisted: Boolean(saved),
    persistedKeyCount: saved && typeof saved === "object" ? Object.keys(saved).length : 0
  });
}

function kgwNodeR51SetAsDefaults(net) {
  kgwNodeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-begin", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });

  const values = kgwNodeR51ReadSettings(net);
  kgwNodeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-read-settings", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    keyCount: Object.keys(values || {}).length,
    checkboxCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "checkbox").length,
    valueCount: Object.keys(values || {}).filter((key) => values[key] && values[key].type === "value").length,
    structuredInstanceCount: (values && values.__kgwBridgeStructuredInstancesR26B && Array.isArray(values.__kgwBridgeStructuredInstancesR26B.instances)) ? values.__kgwBridgeStructuredInstancesR26B.instances.length : 0,
    hasActiveStructuredInstance: Boolean(values && values.__kgwBridgeActiveInstanceR26B)
  });

  kgwNodeR51Store("default:" + net, values);

  const stored = kgwNodeR51Load("default:" + net);
  kgwNodeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    defaultKey: "default:" + String(net || ""),
    persisted: Boolean(stored),
    persistedKeyCount: stored && typeof stored === "object" ? Object.keys(stored).length : 0
  });
}

function kgwNodeR51RestoreDefaults(net) {
  kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-begin", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });

  kgwNodeSettingsWithProgrammaticWriteR9B(() => {
    const defaults = kgwNodeR51Load("default:" + net) || kgwNodeR51Load("factory:" + net);
    kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-loaded", {
      patch: "R29B",
      owner: "node-r51-settings-owner",
      hasDefaults: Boolean(defaults),
      defaultKeyCount: defaults && typeof defaults === "object" ? Object.keys(defaults).length : 0
    });
    kgwNodeR51WriteSettings(net, defaults);
    kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
  });

  kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });
}

function kgwNodeR51IsRunning(text) {
  const value = String(text || "");
  return /running=true/.test(value) || /node_running=true/.test(value) || /official_core_running=true/.test(value);
}

function kgwNodeR51SetRuntimeButtons(net, running, bridgeInprocessLocked = false) {
  const panel = kgwNodeR51Panel(net);
  if (!panel) return;

  const displayOnlyLocked = Boolean(bridgeInprocessLocked || kgwIsBridgeOwnedNodeLockedR65E(net));
  const networkEnabled = kgwNodeNetworkEnabled(net);
  const transition = String(KGW_NODE_R51_TRANSITIONS[net] || "");
  const starting = transition === "starting";
  const stopping = transition === "stopping";
  const transitionActive = Boolean(starting || stopping);
  kgwNodeApplyBridgeOwnedDisplayOnlyR65E(net, displayOnlyLocked, "runtime-buttons");

  const start = panel.querySelector('[data-node-action="start"][data-net="' + net + '"]');
  const stop = panel.querySelector('[data-node-action="stop"][data-net="' + net + '"]');
  const lockMessage = "This node is owned by Bridge in-process mode. Stop the bridge first.";
  const policyStatus = byId(id(net, "policyStatus"));
  const runtimeState = !networkEnabled
    ? "Disabled"
    : starting
      ? "Starting"
      : stopping
        ? "Stopping"
        : running
          ? "Running"
          : "Stopped";

  if (policyStatus) {
    policyStatus.textContent = runtimeState;
    policyStatus.dataset.state = runtimeState.toLowerCase();
  }

  kgwNodeSetRuntimeNotice(
    net,
    runtimeState,
    running || starting ? "Self-worker process owner" : "No process owner",
    ""
  );

  for (const field of kgwNodeR51Fields(net)) {
    field.disabled = Boolean(displayOnlyLocked);
    field.readOnly = Boolean(displayOnlyLocked);
    field.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    field.title = displayOnlyLocked ? lockMessage : "";
  }

  const preview = byId(id(net, "commandPreview"));
  if (preview) {
    preview.readOnly = true;
    preview.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    preview.title = displayOnlyLocked ? lockMessage : "";
  }

  if (start) {
    const startBlocked = Boolean(running || transitionActive || displayOnlyLocked || !networkEnabled);
    start.disabled = startBlocked;
    start.style.opacity = startBlocked ? "0.45" : "";
    start.style.cursor = startBlocked ? "not-allowed" : "";
    start.setAttribute("aria-disabled", startBlocked ? "true" : "false");
    start.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    start.title = displayOnlyLocked
      ? lockMessage
      : !networkEnabled
        ? "Enable this network before starting it."
        : starting
          ? "Node is starting."
          : stopping
            ? "Node is stopping."
            : running
          ? "Node is running. Stop it before starting again."
          : "Start node";
  }

  if (stop) {
    const stopEnabled = Boolean((running || starting) && !stopping && !displayOnlyLocked);
    stop.disabled = !stopEnabled;
    stop.style.opacity = stopEnabled ? "" : "0.45";
    stop.style.cursor = stopEnabled ? "" : "not-allowed";
    stop.setAttribute("aria-disabled", stopEnabled ? "false" : "true");
    stop.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    stop.title = displayOnlyLocked
      ? lockMessage
      : starting
        ? "Stop node startup"
        : stopping
          ? "Node is stopping."
          : running
            ? "Stop node"
            : "Node is not running";
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

// KGW_NODE_BRIDGE_INPROCESS_LOCK_V7
// KGW_NODE_DISPLAY_ONLY_WHEN_BRIDGE_INPROCESS_R65B
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

function kgwNodeApplyBridgeOwnedDisplayOnlyR65E(net, locked, reason) {
  const panel = kgwNodeR51Panel(net);
  if (!panel) return;

  const message = "This network is display-only because Bridge in-process mode owns the node runtime. Stop the bridge first.";

  panel.dataset.kgwBridgeOwnedNodeDisplayOnlyR65E = locked ? "true" : "false";
  panel.setAttribute("aria-readonly", locked ? "true" : "false");
  panel.title = locked ? message : "";

  for (const field of kgwNodeR51Fields(net)) {
    field.disabled = Boolean(locked);
    field.readOnly = Boolean(locked);
    field.dataset.kgwBridgeOwnedNodeDisplayOnlyR65E = locked ? "true" : "false";
    field.setAttribute("aria-readonly", locked ? "true" : "false");
    field.title = locked ? message : "";
  }

  const preview = byId(id(net, "commandPreview"));
  if (preview) {
    preview.readOnly = true;
    preview.dataset.kgwBridgeOwnedNodeDisplayOnlyR65E = locked ? "true" : "false";
    preview.setAttribute("aria-readonly", "true");
    preview.title = locked ? message : "";
  }

  const actionButtons = panel.querySelectorAll("[data-node-action]");
  actionButtons.forEach(function (button) {
    const action = String(button.dataset.nodeAction || "");
    if (action === "start" || action === "stop" || action === "save-settings" || action === "set-defaults" || action === "restore-defaults" || action === "copy-command") {
      button.disabled = Boolean(locked);
      button.setAttribute("aria-disabled", locked ? "true" : "false");
      button.dataset.kgwBridgeOwnedNodeDisplayOnlyR65E = locked ? "true" : "false";
      button.style.opacity = locked ? "0.45" : "";
      button.style.cursor = locked ? "not-allowed" : "";
      button.title = locked ? message : "";
    }
  });

  try {
    kgwNodeExplicitTraceR27D(net, "display-only", locked ? "r65e-node-display-only-enabled" : "r65e-node-display-only-cleared", {
      patch: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E",
      reason: reason || "unknown"
    });
  } catch (_) {}
}

async function kgwNodeR51BridgeInprocessLockedV7(net) {
  if (kgwIsBridgeOwnedNodeLockedR65E(net)) {
    return true;
  }

  try {
    const invoke = getTauriInvoke();
    if (!invoke) return false;

    const result = stringifyRuntimeResult(await invokeWithTimeout(
      invoke,
      "kgw_runtime_owner_status_v1",
      { network: net, runtimeRole: "bridge" },
      KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
    ));

    const fields = parseRuntimeFields(result);
    const nodeMode = String(fields.node_mode || fields.nodeMode || "").toLowerCase();
    const role = String(fields.role || "").toLowerCase();
    const statusNetwork = String(fields.network || net || "");
    const pid = String(fields.pid || "").trim();

    const bridgeLooksAlive =
      fields.running === "true" ||
      fields.bridge_running === "true" ||
      fields.bridge_owner_active === "true" ||
      /^[0-9]+$/.test(pid) ||
      /running=true/i.test(result) ||
      /pid=[0-9]+/i.test(result);

    const sameNetwork = !statusNetwork || statusNetwork === String(net || "");
    const locked = sameNetwork && role === "bridge" && nodeMode === "inprocess" && bridgeLooksAlive;

    if (locked) {
      kgwSetBridgeOwnedNodeLockR65E(net, true, {
        source: "kgw_runtime_owner_status_v1",
        role,
        nodeMode,
        pid
      });
    }

    return locked;
  } catch (_) {
    return kgwIsBridgeOwnedNodeLockedR65E(net);
  }
}


async function kgwNodeR51RefreshOne(net, reason = "live") {
  try {
    const status = stringifyRuntimeResult(await invokeNodeIntegratedRuntime("kgw_runtime_owner_status_v1", net));
    const bridgeInprocessLocked = await kgwNodeR51BridgeInprocessLockedV7(net);
    kgwNodeR51SetRuntimeButtons(net, kgwNodeR51IsRunning(status), bridgeInprocessLocked);

    if (KGW_NODE_R51_LAST_STATUS[net] !== status) {
      KGW_NODE_R51_LAST_STATUS[net] = status;
      
    }

    kgwNodeR51MaybeActivityNotice(net, status);
  } catch (error) {
    const bridgeInprocessLocked = await kgwNodeR51BridgeInprocessLockedV7(net);
    kgwNodeR51SetRuntimeButtons(net, false, bridgeInprocessLocked);
  }

  try {
    const report = await invokeNodeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net);
    kgwNodeApplyRuntimeLogReportV1(net, "node", report);
    KGW_NODE_R51_LAST_LOGS[net] = report;
  } catch {
    // Runtime may not be ready yet.
  }
}

// KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_HYDRATION_R65H2
function kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2(reason = "hydrate") {
  const store = kgwBridgeOwnedNodeLockStoreR65E();
  const keys = new Set();

  for (const net of kgwNodeR51Keys()) keys.add(String(net || ""));
  for (const key of Object.keys(store || {})) keys.add(String(key || ""));

  for (const net of keys) {
    if (!net) continue;
    const locked = kgwIsBridgeOwnedNodeLockedR65E(net);
    if (locked) {
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(net, true, reason);
      kgwNodeR51SetRuntimeButtons(net, false, true);
    }
  }
}

function kgwNodeInstallBridgeOwnedDisplayOnlyHydrationR65H2(root) {
  if (window.__KGW_NODE_BRIDGE_OWNED_DISPLAY_ONLY_HYDRATION_R65H2_INSTALLED) return;
  window.__KGW_NODE_BRIDGE_OWNED_DISPLAY_ONLY_HYDRATION_R65H2_INSTALLED = true;

  window.addEventListener("kgw-bridge-owned-node-lock-r65e", function (event) {
    const detail = event && event.detail ? event.detail : {};
    const net = String(detail.net || "");
    const locked = Boolean(detail.locked);
    if (net) {
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(net, locked, "lock-event");
      kgwNodeR51SetRuntimeButtons(net, false, locked);
    }
    window.setTimeout(function () { kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2("lock-event-late"); }, 0);
  });

  window.setTimeout(function () { kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2("module-init"); }, 0);
}

function kgwNodeR51RefreshAll(reason = "live") {
  kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2(String(reason || "live") + "-before-refresh");
  for (const net of kgwNodeR51Keys()) {
    kgwNodeR51RefreshOne(net, reason);
  }
  kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2(String(reason || "live") + "-after-refresh");
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



/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29_START */
function kgwNodeTranslateRuntimeV29(key, fallback) {
  const runtime = window.kgwT || window.kgwI18n || window.__kgwT;
  if (typeof runtime === "function") {
    try {
      const value = runtime(key, fallback);
      if (value && value !== key) return value;
    } catch (_) {}
  }
  return fallback || key;
}

function kgwNodeLogOutputV29(net) {
  return document.getElementById("node-" + net + "-logOutput");
}

function kgwNodeRestoreLogActionLabelV29(button) {
  if (!button) return;
  const original = button.dataset.kgwLogOriginalLabelV29;
  if (original) button.textContent = original;
  button.classList.remove("kgw-log-action-feedback");
  delete button.dataset.kgwDoneLabel;
}

function kgwNodeFlashLogActionButtonV29(button, doneLabel) {
  if (!button) return;

  if (!button.dataset.kgwLogOriginalLabelV29) {
    button.dataset.kgwLogOriginalLabelV29 = String(button.textContent || "").trim() || "Log Action";
  }

  window.clearTimeout(button.__kgwLogActionFeedbackTimerV29);

  button.textContent = doneLabel;
  button.dataset.kgwDoneLabel = doneLabel;
  button.classList.add("kgw-log-action-feedback");

  button.__kgwLogActionFeedbackTimerV29 = window.setTimeout(() => {
    kgwNodeRestoreLogActionLabelV29(button);
  }, 1600);
}

function kgwNodeCopyLogFailureV1(net, button, error, details = {}) {
  const safeError = kgwNodeClipboardSafeErrorV1(error);
  kgwNodeSetClipboardStatusV1(net, safeError, "error");
  kgwNodeFlashLogActionButtonV29(button, kgwNodeTranslateRuntimeV29("log.copyFailed", "Copy failed"));
  kgwStartTraceFrontendR1("frontend.copy_log_failed", {
    network: net,
    action: "copy-log",
    result: "error",
    details: {
      ...details,
      safeError,
      userFeedbackDisplayed: true
    }
  });
}

async function kgwNodeHandleLogActionV29(action, net, button) {
  
  kgwNodeSmallOwnerTraceR44D(net, String(action || "log-action"), "r51b3-node-log-action-click", {
    patch: "KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3",
    action: String(action || ""),
    buttonId: String(button && button.id || ""),
    buttonText: String(button && button.textContent || "").trim()
  });
  kgwNodeSmallOwnerTraceR44D(net, String(action || "log-action"), "r44d-owner-begin", {});
  const out = kgwNodeLogOutputV29(net);
  if (!out && action !== "copy-log") return;

  if (action === "copy-log") {
    const nodeRoot = document.getElementById("kaspa-node");
    const activeNetwork = kgwNodeTraceActiveNetworkR1(nodeRoot);
    const belongsToLiveNodeMonitor = Boolean(button?.closest?.('[data-node-inner-panel="log"]'));
    const copyNetwork = String(net || "").trim();

    if (!copyNetwork) {
      kgwNodeCopyLogFailureV1(net, button, "Copy Log could not resolve the active network.", {
        reason: "missing-network",
        activeNetwork,
        belongsToLiveNodeMonitor
      });
      return;
    }

    kgwStartTraceFrontendR1("frontend.copy_log_network_resolved", {
      network: copyNetwork,
      action: "copy-log",
      result: activeNetwork && activeNetwork !== copyNetwork ? "error" : "ok",
      details: {
        activeNetwork,
        buttonNetwork: copyNetwork,
        belongsToLiveNodeMonitor
      }
    });

    if (activeNetwork && activeNetwork !== copyNetwork) {
      kgwNodeCopyLogFailureV1(copyNetwork, button, "Copy Log network mismatch; active network changed before copy started.", {
        reason: "network-mismatch",
        activeNetwork,
        buttonNetwork: copyNetwork,
        belongsToLiveNodeMonitor
      });
      return;
    }

    if (button?.dataset?.kgwCopyLogInFlightV1 === "1") {
      kgwNodeCopyLogFailureV1(copyNetwork, button, "Copy Log is already in progress for this network.", {
        reason: "duplicate-copy",
        activeNetwork,
        belongsToLiveNodeMonitor
      });
      return;
    }

    const originalDisabled = Boolean(button && button.disabled);
    if (button) {
      button.dataset.kgwCopyLogInFlightV1 = "1";
      button.disabled = true;
    }

    try {
      const buffer = kgwNodeReadClipboardRawLogBufferV1(copyNetwork);

      if (!buffer.out || buffer.isPlaceholder || !buffer.normalizedText.trim()) {
        kgwStartTraceFrontendR1("frontend.copy_log_content_prepared", {
          network: copyNetwork,
          action: "copy-log",
          result: "error",
          details: {
            rawLogBufferSelected: Boolean(buffer.out),
            placeholderRejected: Boolean(buffer.isPlaceholder),
            runtimeRole: "node",
            bridgeInstanceId: "",
            characterCount: buffer.characterCount,
            lineCount: buffer.lineCount,
            sha256: ""
          }
        });
        throw new Error("Copy Log requires a non-empty raw log buffer for " + copyNetwork + ".");
      }

      const sha256 = await kgwNodeSha256HexV1(buffer.normalizedText);
      const metadata = {
        runtimeRole: "node",
        bridgeInstanceId: "",
        characterCount: buffer.characterCount,
        lineCount: buffer.lineCount,
        sha256
      };

      kgwStartTraceFrontendR1("frontend.copy_log_content_prepared", {
        network: copyNetwork,
        action: "copy-log",
        result: "ok",
        details: {
          rawLogBufferSelected: true,
          placeholderRejected: false,
          runtimeRole: metadata.runtimeRole,
          bridgeInstanceId: metadata.bridgeInstanceId,
          characterCount: metadata.characterCount,
          lineCount: metadata.lineCount,
          sha256: metadata.sha256 || ""
        }
      });

      await kgwNodeDispatchClipboardWriteV1(copyNetwork, buffer.normalizedText, metadata);

      kgwNodeFlashLogActionButtonV29(button, kgwNodeTranslateRuntimeV29("log.copied", "Copied"));
      kgwNodeSetClipboardStatusV1(copyNetwork, kgwNodeTranslateRuntimeV29("log.copied", "Copied"), "ok");
      kgwStartTraceFrontendR1("frontend.copy_log_succeeded", {
        network: copyNetwork,
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
      kgwNodeCopyLogFailureV1(copyNetwork, button, error, {
        activeNetwork,
        belongsToLiveNodeMonitor
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
    kgwNodeClearRawLogBufferV1(net, "node");
    kgwNodeDispatchRuntimeLogClearV1(net, "node").catch(() => {});
    kgwNodeFlashLogActionButtonV29(button, kgwNodeTranslateRuntimeV29("log.deleted", "Deleted"));
  }
  kgwNodeSmallOwnerTraceR44D(net, String(action || "log-action"), "r44d-owner-complete", {});
}
/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29_END */

function installActions(root) {
  if (!root.dataset.kgwNodeCommandComposerInlineOwnerR7) {
    root.dataset.kgwNodeCommandComposerInlineOwnerR7 = "1";

    /* KGW_NODE_COMMAND_CHECKBOX_FIRST_CLICK_FIX_TRACE_PATCH_R31
     Native checkbox first-click fix:
     - pointerdown/click/change traces are scoped to this existing Node root owner.
     - native checkbox clicks are not preventDefault() blocked.
     - checked state is committed from the change event.
     - non-checkbox fallback keeps the legacy click toggle path.
   */
    root.addEventListener("pointerdown", (event) => {
      const toggle = event.target.closest("[data-node-command-option-toggle-r7]");
      if (!toggle || !root.contains(toggle)) return;

      kgwNodeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-node-command-checkbox-pointerdown", {
        patch: "R31",
        owner: "node-command-composer-r7",
        option: String(toggle.dataset.nodeCommandOptionToggleR7 || ""),
        tag: String(toggle.tagName || ""),
        type: String(toggle.type || ""),
        checkedBefore: Boolean(toggle.checked),
        trusted: Boolean(event && event.isTrusted)
      });
    });

    root.addEventListener("change", (event) => {
      const toggle = event.target.closest("[data-node-command-option-toggle-r7]");
      if (!toggle || !root.contains(toggle)) return;

      const net = toggle.dataset.net;
      const option = toggle.dataset.nodeCommandOptionToggleR7;
      const enabled = Boolean(toggle.checked);

      kgwNodeSmallOwnerTraceR44D(net, "command-checkbox", "r31-node-command-checkbox-change-begin", {
        patch: "R31",
        owner: "node-command-composer-r7",
        option: String(option || ""),
        checked: enabled,
        trusted: Boolean(event && event.isTrusted)
      });

      try {
        if (typeof kgwNodeCommandInlineStateR7 === "function") {
          const state = kgwNodeCommandInlineStateR7(net);
          state[String(option)] = enabled;
          updateCommand(net);
          if (typeof kgwNodeRefreshInlineCommandTogglesR7 === "function") {
            kgwNodeRefreshInlineCommandTogglesR7(net);
          }
        } else if (typeof kgwNodeToggleCommandOptionR7 === "function") {
          kgwNodeToggleCommandOptionR7(net, option);
        }

        queueMicrotask(() => {
          kgwNodeSmallOwnerTraceR44D(net, "command-checkbox", "r31-node-command-checkbox-change-after-microtask", {
            patch: "R31",
            owner: "node-command-composer-r7",
            option: String(option || ""),
            checkedAfter: Boolean(toggle.checked)
          });
        });
      } catch (error) {
        kgwNodeSmallOwnerTraceR44D(net, "command-checkbox", "r31-node-command-checkbox-change-failed", {
          patch: "R31",
          owner: "node-command-composer-r7",
          option: String(option || ""),
          message: error && error.message ? error.message : String(error)
        });
      }
    });

    root.addEventListener("click", (event) => {
      const toggle = event.target.closest("[data-node-command-option-toggle-r7]");
      if (toggle && root.contains(toggle)) {
        const isNativeCheckbox = toggle.matches && toggle.matches("input[type='checkbox']");

        kgwNodeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-node-command-checkbox-click", {
          patch: "R31",
          owner: "node-command-composer-r7",
          option: String(toggle.dataset.nodeCommandOptionToggleR7 || ""),
          tag: String(toggle.tagName || ""),
          type: String(toggle.type || ""),
          isNativeCheckbox: Boolean(isNativeCheckbox),
          checkedAtClick: Boolean(toggle.checked),
          trusted: Boolean(event && event.isTrusted)
        });

        if (isNativeCheckbox) {
          event.stopPropagation();
          queueMicrotask(() => {
            kgwNodeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-node-command-checkbox-click-after-microtask", {
              patch: "R31",
              owner: "node-command-composer-r7",
              option: String(toggle.dataset.nodeCommandOptionToggleR7 || ""),
              checkedAfter: Boolean(toggle.checked)
            });
          });
          return;
        }

        event.preventDefault();
        event.stopPropagation();
        kgwNodeToggleCommandOptionR7(toggle.dataset.net, toggle.dataset.nodeCommandOptionToggleR7);
      }
    });

    root.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      const toggle = event.target.closest("[data-node-command-option-toggle-r7]");
      if (toggle && root.contains(toggle)) {
        event.preventDefault();
        event.stopPropagation();
        kgwNodeToggleCommandOptionR7(toggle.dataset.net, toggle.dataset.nodeCommandOptionToggleR7);
      }
    });
  }
  // KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26: Node settings actions are scoped to the exact network that changed.
  if (window.KGW_NODE_SETTINGS_OWNER_V19 && typeof window.KGW_NODE_SETTINGS_OWNER_V19.install === "function") {
    window.KGW_NODE_SETTINGS_OWNER_V19.install(root);
  }

  kgwNodeInstallBridgeOwnedDisplayOnlyHydrationR65H2(root);
  kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2("install-actions");
  window.setTimeout(function () {
    kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2("install-actions-late");
  }, 0);

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


  // KGW_EXPLICIT_TRACE_OWNER_R27D_NODE_BEGIN
  function kgwNodeExplicitTraceR27D(net, action, phase, details) {
    try {
      const safeNet = String(net || "unknown");
      const safeAction = String(action || "unknown");
      const safePhase = String(phase || "unknown");
      const payload = {
        patch: "KGW_EXPLICIT_TRACE_EXACT_ANCHOR_PATCH_R27D",
        owner: "node-existing-owner",
        network: safeNet,
        action: safeAction,
        phase: safePhase,
        details: details && typeof details === "object" ? details : {}
      };

      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", {
          scope: "node",
          net: safeNet,
          action: safeAction,
          phase: safePhase,
          details: JSON.stringify(payload)
        }).catch(function () {});
      }
    } catch (_) {}
  }
  // KGW_EXPLICIT_TRACE_OWNER_R27D_NODE_END

  function scopedUpdate(net, reason) {
    if (!net) return;
    if (typeof updateCommand === "function") {
      updateCommand(net);
    }
    kgwNodeExplicitTraceR27D(net, "settings-scope", "r27d-scoped-update", {
      previousPatch: "KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26",
      reason: reason || "unknown"
    });
  }

  root.addEventListener("input", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.disabled || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const inputNet = netFromEvent(event);
    if (inputNet && kgwIsBridgeOwnedNodeLockedR65E(inputNet)) {
      event.preventDefault();
      event.stopPropagation();
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(inputNet, true, "input-guard");
      kgwNodeExplicitTraceR27D(inputNet, "display-only", "r65e-node-input-blocked", {
        patch: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E",
        targetId: String(target.id || "")
      });
      return;
    }

    const net = netFromEvent(event);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.disabled || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const changeNet = netFromEvent(event);
    if (changeNet && kgwIsBridgeOwnedNodeLockedR65E(changeNet)) {
      event.preventDefault();
      event.stopPropagation();
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(changeNet, true, "change-guard");
      kgwNodeExplicitTraceR27D(changeNet, "display-only", "r65e-node-change-blocked", {
        patch: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E",
        targetId: String(target.id || "")
      });
      return;
    }

    const net = netFromEvent(event);

    if (target.matches("[data-node-network-enabled]")) {
      const profile = kgwNodeNetworkProfile(net);
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

      kgwNodeSetNetworkEnabled(net, enabled);
      kgwNodeR51SetRuntimeButtons(net, false, kgwIsBridgeOwnedNodeLockedR65E(net));

      if (!enabled) {
        void runNodeIntegratedAction("stop", net);
      }
    }

    scopedUpdate(net, event.isTrusted ? "trusted-change" : "programmatic-change");
  }, true);

  root.addEventListener("click", (event) => {
    const button = event.target && event.target.closest ? event.target.closest("[data-node-action]") : null;
    if (!button || !root.contains(button)) return;

    const action = button.dataset.nodeAction;
    const net = normalizeNet(button.dataset.net || button.dataset.network || netFromElement(button));

    if (!net) return;

    const lockedBeforeAction = kgwIsBridgeOwnedNodeLockedR65E(net);

    if (action !== "start" && action !== "stop") {
      kgwNodeExplicitTraceR27D(net, String(action || "unknown"), "r27d-action-click", {
        trusted: Boolean(event && event.isTrusted),
        disabled: Boolean(button.disabled || lockedBeforeAction),
        id: String(button.id || ""),
        text: String(button.textContent || "").trim(),
        bridgeOwnedDisplayOnly: Boolean(lockedBeforeAction)
      });
    }

    if (lockedBeforeAction && (action === "start" || action === "stop" || action === "save-settings" || action === "set-defaults" || action === "restore-defaults" || action === "copy-command")) {
      event.preventDefault();
      event.stopPropagation();
      kgwNodeApplyBridgeOwnedDisplayOnlyR65E(net, true, "click-guard");
      if (typeof appendLog === "function" && (action === "start" || action === "stop")) {
        kgwNodeSetRuntimeNotice(net, "Blocked", "Bridge in-process owner", "This network is display-only because Bridge in-process mode owns the node runtime. Stop the bridge first.");
      }
      return;
    }

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

    if (action === "copy-log" || action === "clear-log") {
      event.preventDefault();
      event.stopPropagation();
      kgwNodeHandleLogActionV29(action, net, button).catch(function (error) {
        if (action === "copy-log") {
          kgwNodeCopyLogFailureV1(net, button, error, {
            reason: "unhandled-copy-error"
          });
        }
      });
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


export async function initKaspaNodeTab(root) {

const nodeRoot = root || document.getElementById("kaspa-node");
  if (!nodeRoot || nodeRoot.dataset.kgwNodeV6Ready === "true") return;

  nodeRoot.dataset.kgwNodeV6Ready = "true";

  renderAllNetworks(nodeRoot);
  kgwNodeTraceRenderedStartControlsR1(nodeRoot);
  kgwNodeInstallStartTraceDocumentClickObserverR1(nodeRoot);
  {
    const resolved = kgwResolvePublicTauriInvokeR1();
    kgwStartTraceFrontendR1("frontend.tauri_invoke_api_availability", {
      network: kgwNodeTraceActiveNetworkR1(nodeRoot) || "mainnet",
      action: "init",
      result: resolved.invoke ? "available" : "missing",
      details: {
        adapter: resolved.adapter,
        shape: resolved.shape
      }
    });
  }
  kgwNodeR51CaptureFactoryDefaults();
  kgwNodeR51LoadSavedSettings();
  NODE_NETWORKS.forEach((net) => kgwNodeR51SetRuntimeButtons(net.key, false, false));
  installNetworkTabs(nodeRoot);
  installDelegatedTabs(nodeRoot);
  installActions(nodeRoot);
updateAllCommands();
  NODE_NETWORKS.forEach((net) => kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net.key, { force: false })); /* KGW_NODE_DYNAMIC_PATHS_INIT_R3 */
  installKgwNodeR51BottomStyle();
  kgwNodeR51StartLiveRefresh();

  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
}


/* KGW_NODE_LOG_SCOPED_CONTROLS_V29_START */
(function installKgwLogScopedToolbarControlsV29() {
  "use strict";

  const KIND = "node";
  const ROOT_SELECTOR = "#kaspa-node";
  const TOOLBAR_SELECTOR = ".node-v6-log-toolbar";
  const ACTION_ATTR = "data-node-action";
  const PREFIX = "node";
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
    controls.dataset.marker = "KGW_NODE_LOG_SCOPED_CONTROLS_V29";

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
      kgwNodeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-node-log-font-decrease-click", {
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
      kgwNodeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-node-log-font-increase-click", {
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
      kgwNodeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-node-log-font-reset-click", {
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

  window.kgwInstallNodeLogScopedControlsV29 = installAll;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", installAll, { once: true });
  } else {
    window.setTimeout(installAll, 0);
  }
})();
/* KGW_NODE_LOG_SCOPED_CONTROLS_V29_END */

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
