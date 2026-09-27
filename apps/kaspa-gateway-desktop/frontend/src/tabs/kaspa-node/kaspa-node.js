import { applyStatusTone, renderStatusSummary } from "../../status.js";
import { NODE_ENDPOINTS, NODE_MANAGED, NODE_REQUIRED, NODE_OPTIONAL, NODE_DANGEROUS, nodeFieldEnabled, validateNodeForm, renderFieldErrors, endpoint, runtimePresentation, runtimeObservationSummary, confirmUserAction } from "../../settings-contract.js";
import { renderSettingsTabs, installSettingsLayout, decorateSettingsFields, revealSettingsField, setSettingFieldState } from "../../settings-layout.js";
import initNodeRust, {
  nodeBackendInvoke as wasmNodeBackendInvoke,
  nodeById as wasmNodeById,
  nodeCardCheck as wasmNodeCardCheck,
  nodeCardInput as wasmNodeCardInput,
  nodeCardSelect as wasmNodeCardSelect,
  nodeChecked as wasmNodeChecked,
  nodeCommandInlineState as wasmNodeCommandInlineState,
  nodeCommandInlineStateKey as wasmNodeCommandInlineStateKey,
  nodeCommandInlineToggle as wasmNodeCommandInlineToggle,
  nodeCommandOptionEnabled as wasmNodeCommandOptionEnabled,
  nodeCommandOptionsKey as wasmNodeCommandOptionsKey,
  nodeCommandShouldInclude as wasmNodeCommandShouldInclude,
  nodeRefreshInlineCommandToggles as wasmNodeRefreshInlineCommandToggles,
  nodeToggleCommandOption as wasmNodeToggleCommandOption,
  nodeCopyLogFailure as wasmNodeCopyLogFailure,
  nodeDispatchClipboardWrite as wasmNodeDispatchClipboardWrite,
  nodeElementId as wasmNodeElementId,
  nodeEscapeHtml as wasmNodeEscapeHtml,
  nodeHandleCopyLog as wasmNodeHandleCopyLog,
  nodeI18nText as wasmNodeI18nText,
  nodeInstallLogAutoScrollControls as wasmNodeInstallLogAutoScrollControls,
  nodeInstallStartTraceDocumentClickObserver as wasmNodeInstallStartTraceDocumentClickObserver,
  nodeLogAutoScrollEnabled as wasmNodeLogAutoScrollEnabled,
  nodeNetworkEnabled as wasmNodeNetworkEnabled,
  nodeNetworkPolicyKey as wasmNodeNetworkPolicyKey,
  nodeNetworkPolicyMessage as wasmNodeNetworkPolicyMessage,
  nodeNetworkProfile as wasmNodeNetworkProfile,
  nodeNetworkProfiles as wasmNodeNetworkProfiles,
  nodeNormalizeInnerTab as wasmNodeNormalizeInnerTab,
  nodeNormalizeNetwork as wasmNodeNormalizeNetwork,
  nodeNormalizeRuntimeError as wasmNodeNormalizeRuntimeError,
  nodeParseRuntimeFields as wasmNodeParseRuntimeFields,
  nodeReadLastNetwork as wasmNodeReadLastNetwork,
  nodeR51CaptureFactoryDefaults as wasmNodeR51CaptureFactoryDefaults,
  nodeR51Fields as wasmNodeR51Fields,
  nodeR51Keys as wasmNodeR51Keys,
  nodeR51Load as wasmNodeR51Load,
  nodeR51LoadSavedSettings as wasmNodeR51LoadSavedSettings,
  nodeR51Panel as wasmNodeR51Panel,
  nodeR51ReadSettings as wasmNodeR51ReadSettings,
  nodeR51RestoreDefaultsAction as wasmNodeR51RestoreDefaultsAction,
  nodeR51SaveSettingsAction as wasmNodeR51SaveSettingsAction,
  nodeR51SetDefaultsAction as wasmNodeR51SetDefaultsAction,
  nodeR51Store as wasmNodeR51Store,
  nodeR51WriteSettings as wasmNodeR51WriteSettings,
  nodeResolveInnerTab as wasmNodeResolveInnerTab,
  nodeResolvePublicTauriInvoke as wasmNodeResolvePublicTauriInvoke,
  nodeRuntimeActionForCommand as wasmNodeRuntimeActionForCommand,
  nodeRuntimeErrorFromStatus as wasmNodeRuntimeErrorFromStatus,
  nodeRuntimeIsRunning as wasmNodeRuntimeIsRunning,
  nodeSaveInnerTab as wasmNodeSaveInnerTab,
  nodeSaveLastNetwork as wasmNodeSaveLastNetwork,
  nodeSetLogAutoScroll as wasmNodeSetLogAutoScroll,
  nodeSetNetworkEnabled as wasmNodeSetNetworkEnabled,
  nodeStringifyRuntimeResult as wasmNodeStringifyRuntimeResult,
  nodeSmallOwnerTrace as wasmNodeSmallOwnerTrace,
  nodeStartTraceFrontend as wasmNodeStartTraceFrontend,
  nodeStartTraceTauriShape as wasmNodeStartTraceTauriShape,
  nodeTraceActiveNetwork as wasmNodeTraceActiveNetwork,
  nodeTraceRenderedStartControls as wasmNodeTraceRenderedStartControls,
  nodeTraceStartButtonState as wasmNodeTraceStartButtonState,
  nodeValue as wasmNodeValue,
} from "../../../generated/kgw_frontend_wasm/kgw_frontend_wasm.js";

await initNodeRust();

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
  } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
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
  } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }

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
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }

    try {
      const keys = ["kgw.language", "kgw_locale", "language", "locale", "i18nextLng"];
      for (const key of keys) {
        const value = localStorage.getItem(key);
        if (value) return lower(value);
      }
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }

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
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }

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
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
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
      text.includes("طھظ… ط§ظ„ط­ظپط¸") ||
      text.includes("طھظ… ط§ظ„ط¶ط¨ط·") ||
      text.includes("طھظ…طھ ط§ظ„ط§ط³طھط¹ط§ط¯ط©") ||
      text.includes("ط­ظپط¸") ||
      text.includes("ط§ط³طھط¹ط§ط¯ط©") ||
      text.includes("ط§ظپطھط±ط§ط¶")
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

      if (id.includes("testnet13") || cls.includes("testnet13") || id.includes("tn13") || cls.includes("tn13")) return "testnet13";
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

    if (raw.includes("restore") || raw.includes("ط§ط³طھط¹ط§ط¯ط©")) return "restore";
    if (raw.includes("default") || raw.includes("ط§ظپطھط±ط§ط¶") || raw.includes("ط¶ط¨ط·")) return "defaults";
    return "save";
  }

  function feedbackText(action) {
    if (isArabic()) {
      if (action === "restore") return "طھظ…طھ ط§ظ„ط§ط³طھط¹ط§ط¯ط©";
      if (action === "defaults") return "طھظ… ط§ظ„ط¶ط¨ط·";
      return "طھظ… ط§ظ„ط­ظپط¸";
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

  function setDisabled(root, network, _disabled, reason) {
    const networks = network && network !== "all" ? [network] : kgwNodeR51Keys();
    const canonical = value => Array.isArray(value) ? value.map(canonical) : value && typeof value === "object"
      ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
    const stable = value => JSON.stringify(canonical(value));
    for (const net of networks) {
      const current = kgwNodeR51ReadSettings(net);
      const saved = kgwNodeR51Load("saved:" + net) || kgwNodeR51Load("factory:" + net) || current;
      const personal = kgwNodeR51Load("default:" + net);
      const defaults = personal || kgwNodeR51Load("factory:" + net) || current;
      const sameSaved = stable(current) === stable(saved);
      const sameDefaults = stable(current) === stable(defaults);
      buttons(root, net).forEach(button => {
        const action = actionName(button);
        const disabled = kgwIsBridgeOwnedNodeLockedR65E(net) || (action === "save" ? sameSaved : sameDefaults);
        button.disabled = disabled;
        button.setAttribute("aria-disabled", String(disabled));
        button.classList.toggle(DISABLED_CLASS, disabled);
        button.dataset.kgwSettingsOwnerV19Disabled = String(disabled);
        button.title = disabled
          ? (action === "save" ? "No unsaved changes." : "Current settings already match the selected defaults.")
          : (action === "restore" ? "Load " + (personal ? "your personal" : "KaspaGateway") + " defaults. Save to retain them."
            : action === "defaults" ? "Use current settings as your personal defaults for this network." : "Save this network's settings.");
      });
      const helper = root.querySelector('[data-settings-defaults-context="' + net + '"]');
      if (helper) helper.textContent = "Restore uses " + (personal ? "personal defaults" : "KaspaGateway defaults") + ". Settings apply on the next Start.";
    }
    trace(root, "settings-actions-reconciled", { network, reason });
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
      value === "طھظ… ط§ظ„ط­ظپط¸" ||
      value === "طھظ… ط§ظ„ط¶ط¨ط·" ||
      value === "طھظ…طھ ط§ظ„ط§ط³طھط¹ط§ط¯ط©"
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
    button.setAttribute("aria-label", button.textContent);
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
      if (button.dataset.kgwSettingsActionResult !== "success") return;
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

      if (action !== "restore" && Object.keys(kgwNodeValidateForm(network, true)).length) {
        event.preventDefault();
        event.stopImmediatePropagation();
        return;
      }
      button.dataset.kgwSettingsActionResult = "pending";
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

/* KGW_START_TRACE_R1 is Rust-owned in node_start_trace.rs. */
function kgwStartTraceTauriShapeR1(adapterName = "") {
  return wasmNodeStartTraceTauriShape(String(adapterName || ""));
}

function kgwResolvePublicTauriInvokeR1() {
  return wasmNodeResolvePublicTauriInvoke();
}

function kgwStartTraceFrontendR1(stage, options = {}) {
  return wasmNodeStartTraceFrontend(stage, options || {});
}
function kgwNodeLogEmptyStateV1(net) {
  return document.getElementById("node-" + net + "-logEmpty");
}

async function kgwNodeDispatchClipboardWriteV1(net, text, metadata = {}) {
  return await wasmNodeDispatchClipboardWrite(String(net || ""), String(text ?? ""), metadata || {});
}

/* KGW_NODE_TRACE_OBSERVER_R1 is Rust-owned in node_start_trace.rs. */


function kgwNodeTraceActiveNetworkR1(root = document.getElementById("kaspa-node")) {
  return wasmNodeTraceActiveNetwork(root);
}

function kgwNodeTraceStartButtonStateR1(net) {
  return wasmNodeTraceStartButtonState(String(net || ""));
}

function kgwNodeInstallStartTraceDocumentClickObserverR1(root) {
  return wasmNodeInstallStartTraceDocumentClickObserver(root);
}

function kgwNodeTraceRenderedStartControlsR1(root) {
  return wasmNodeTraceRenderedStartControls(root);
}

function kgwNodeRuntimeActionForCommandR1(command) {
  return wasmNodeRuntimeActionForCommand(String(command || ""));
}

function kgwNodeSmallOwnerTraceR44D(net, action, phase, details) {
  return wasmNodeSmallOwnerTrace(net, action, phase, details);
}

function kgwI18nTextR41(key, fallback) {
  return wasmNodeI18nText(String(key || ""), String(fallback || ""));
}



/* Canonical isolated node runtime paths.
 * Each network owns a separate database below:
 * %LOCALAPPDATA%\KaspaGateway\nodes\<network>
 */
function kgwNodeBackendInvokeR5(command, payload = {}) {
  return wasmNodeBackendInvoke(String(command || ""), payload || {});
}

/* KGW_NODE_PATH_HELPERS_R5 are Rust-owned in node_path_helpers.rs. */










async function kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5(net, _options = {}) {
  const context = await kgwNodeBackendInvokeR5("kgw_settings_context_v1", {network: net});
  const field = byId(id(net, "appDir"));
  if (field) { field.value = context.appDir; field.title = context.appDir; }
  updateCommand(net);
  return {appDir: context.appDir};
}

function kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, options = {}) {
  void kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5(net, options).catch((error) => {
    appendLog(net, "Rusty Kaspa root-only default path restore failed: " + normalizeRuntimeError(error));
  });
}

const NODE_NETWORKS = wasmNodeNetworkProfiles();

function kgwNodeNetworkPolicyKey(net) {
  return wasmNodeNetworkPolicyKey(String(net || ""));
}

function kgwNodeNetworkProfile(net) {
  return wasmNodeNetworkProfile(String(net || ""));
}

function kgwNodeNetworkEnabled(net) {
  return wasmNodeNetworkEnabled(String(net || ""));
}

function kgwNodeSetNetworkEnabled(net, enabled) {
  wasmNodeSetNetworkEnabled(String(net || ""), Boolean(enabled));
}

function kgwNodeNetworkPolicyMessage(net) {
  return wasmNodeNetworkPolicyMessage(String(net || ""));
}

function byId(id) {
  return wasmNodeById(String(id || ""));
}

function esc(value) {
  return wasmNodeEscapeHtml(value);
}

function id(net, name) {
  return wasmNodeElementId(String(net || ""), String(name || ""));
}

function v(net, name) {
  return wasmNodeValue(String(net || ""), String(name || ""));
}

function c(net, name) {
  return wasmNodeChecked(String(net || ""), String(name || ""));
}








// KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7
/* Command-composer state/policy/toggle ownership is Rust/WASM-owned. */
function kgwNodeCommandInlineStateKeyR7(net) {
  return wasmNodeCommandInlineStateKey(String(net || ""));
}

function kgwNodeCommandInlineStateR7(net) {
  return wasmNodeCommandInlineState(String(net || ""));
}

function kgwNodeCommandOptionEnabledR7(net, name) {
  return wasmNodeCommandOptionEnabled(String(net || ""), String(name || ""));
}

function kgwNodeCommandShouldIncludeR7(net, name) {
  return wasmNodeCommandShouldInclude(String(net || ""), String(name || ""));
}

function kgwNodeCommandInlineToggleR7(net, name) {
  return wasmNodeCommandInlineToggle(String(net || ""), String(name || ""));
}

function kgwNodeRefreshInlineCommandTogglesR7(net) {
  wasmNodeRefreshInlineCommandToggles(String(net || ""));
}

function kgwNodeToggleCommandOptionR7(net, name) {
  wasmNodeToggleCommandOption(String(net || ""), String(name || ""));
  updateCommand(net);
}


/* Node card markup rendering is Rust-owned in node_frontend_helpers.rs. */
function cardInput(net, name, label, value = "", placeholder = "", span2 = false) {
  return wasmNodeCardInput(
    String(net || ""),
    String(name || ""),
    String(label || ""),
    String(value ?? ""),
    String(placeholder ?? ""),
    Boolean(span2),
    kgwNodeCommandInlineToggleR7(net, name)
  );
}

function cardSelect(net, name, label, options, value = "", span2 = false) {
  return wasmNodeCardSelect(
    String(net || ""),
    String(name || ""),
    String(label || ""),
    Array.from(options || [], (item) => String(item ?? "")),
    String(value ?? ""),
    Boolean(span2),
    kgwNodeCommandInlineToggleR7(net, name)
  );
}

function cardCheck(net, name, label, checked = false, span2 = false) {
  return wasmNodeCardCheck(
    String(net || ""),
    String(name || ""),
    String(label || ""),
    Boolean(checked),
    Boolean(span2)
  );
}





// KGW_NODE_LOG_AUTOSCROLL_CONTROLS_R27 is Rust-owned in node_frontend_helpers.rs.
function kgwNodeLogAutoScrollEnabledR27(net) {
  return wasmNodeLogAutoScrollEnabled(String(net || ""));
}

function kgwNodeSetLogAutoScrollR27(net, enabled) {
  wasmNodeSetLogAutoScroll(String(net || ""), Boolean(enabled));
}

function kgwInstallNodeLogAutoScrollControlsR27() {
  wasmNodeInstallLogAutoScrollControls();
}

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

function kgwNodeRawLogTextHasTransportWrapperV1(value) {
  const text = typeof value === "string" ? value.trim() : "";
  if (!text) return false;
  if (/^kgw_raw_process_log_v1(?:;|$)/i.test(text)) return true;
  return /^(?:\[KGW_CHILD_STD(?:OUT|ERR)\]\s*)?\{[\s\S]*["']eventKind["']\s*:\s*["']diagnostic_transport_record["']/i.test(text);
}

function kgwNodeLegacyTransportReportTextV1(report) {
  if (typeof report === "string") return report;
  if (!report || typeof report !== "object" || Array.isArray(report) || Array.isArray(report.entries)) return "";
  return String(report.rawText ?? report.raw_text ?? report.line ?? "");
}

function kgwNodeNormalizeRawLogEntryV1(entry, expectedNet, expectedRole = "node") {
  if (!entry || typeof entry !== "object") return null;

  const rawTextValue = entry.rawText ?? entry.raw_text ?? entry.line;
  if (rawTextValue === undefined || rawTextValue === null) return null;
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
  const legacyTransportText = kgwNodeLegacyTransportReportTextV1(report);
  if (kgwNodeRawLogTextHasTransportWrapperV1(legacyTransportText)) return 0;
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
  const p2pPort = net.key === "mainnet" ? "16111" : net.key === "testnet10" ? "16211" : "16711";
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "listenEnabled", "--listen", false)}
      ${cardInput(net.key, "listenHost", "listen host", "0.0.0.0")}
      ${cardInput(net.key, "listenPort", "listen port", p2pPort)}
      ${cardCheck(net.key, "externalIpEnabled", "--externalip", false)}
      ${cardInput(net.key, "externalIpHost", "external host", "", "ip")}
      ${cardInput(net.key, "externalIpPort", "external port", "", "port")}
      ${cardCheck(net.key, "disableUpnp", "--disable-upnp", true)}
      ${cardCheck(net.key, "noDnsSeed", "--nodnsseed", false)}
      ${cardInput(net.key, "uaComment", "--uacomment", "", "comment", true)}
    </div>`;
}

function renderRpc(net) {
  const base = net.key === "mainnet" ? 16110 : net.key === "testnet10" ? 16210 : 16210;
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "rpcListenEnabled", "--rpclisten", true)}
      ${cardInput(net.key, "rpcListenHost", "RPC host", "127.0.0.1")}
      ${cardInput(net.key, "rpcListenPort", "RPC port", String(base))}
      ${cardCheck(net.key, "rpcBorshEnabled", "--rpclisten-borsh", false)}
      ${cardInput(net.key, "rpcBorshHost", "Borsh host", "127.0.0.1")}
      ${cardInput(net.key, "rpcBorshPort", "Borsh port", String(base + 1000))}
      ${cardCheck(net.key, "rpcJsonEnabled", "--rpclisten-json", false)}
      ${cardInput(net.key, "rpcJsonHost", "JSON host", "127.0.0.1")}
      ${cardInput(net.key, "rpcJsonPort", "JSON port", String(base + 2000))}
      ${cardInput(net.key, "rpcMaxClients", "--rpcmaxclients (managed max 16)", "16")}
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
      ${cardInput(net.key, "maxInPeers", "--maxinpeers (managed max 32)", "32")}
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
      ${cardInput(net.key, "overrideParamsFile", "--override-params-file (unsupported: managed network)", "", "not supported", true)}
    </div>`;
}

function renderPaths(net) {
  return `
    <div class="node-v6-grid">
      ${cardInput(net.key, "configFile", "--configfile (unsupported: managed ownership)", "", "not supported")}
      ${cardInput(net.key, "appDir", "--appdir (managed per network)", "", "managed by desktop")}
      ${cardInput(net.key, "logDir", "--logdir", "", "log dir")}
    </div>`;
}

function renderSections(net) {
  const template = document.createElement("template");
  template.innerHTML = [renderRuntime(net), renderNetwork(net), renderRpc(net), renderPeers(net),
    renderDatabase(net), renderRocksDb(net), renderPaths(net)].join("");
  const cards = new Map();
  template.content.querySelectorAll(".node-v6-card").forEach(card => {
    const field = card.querySelector("[id]");
    if (field) cards.set(field.id.slice(("node-" + net.key + "-").length), card.outerHTML);
  });
  const definitions = [
    ["general", "basic", "Basic", "testnet netsuffix utxoIndex yes"],
    ["general", "networking", "Networking", "listenEnabled listenHost listenPort externalIpEnabled externalIpHost externalIpPort disableUpnp noDnsSeed uaComment"],
    ["general", "rpc", "RPC", "rpcListenEnabled rpcListenHost rpcListenPort"],
    ["general", "performance", "Performance", "asyncThreads ramScale outPeers maxInPeers"],
    ["general", "storage", "Storage", "appDir rocksDbPreset rocksDbCacheSize"],
    ["general", "logging", "Logging", "logLevel noLogFiles logDir"],
    ["advanced", "p2p", "P2P", "connectEnabled connectHost connectPort addPeerEnabled addPeerHost addPeerPort"],
    ["advanced", "rpc-advanced", "RPC Advanced", "rpcBorshEnabled rpcBorshHost rpcBorshPort rpcJsonEnabled rpcJsonHost rpcJsonPort rpcMaxClients noGrpc"],
    ["advanced", "database", "Database", "archival maxTrackedAddresses retentionDays rocksDbWalDir configFile sanity"],
    ["advanced", "metrics", "Metrics", "perfMetrics perfMetricsInterval"],
    ["advanced", "experimental", "Experimental", "overrideParamsFile"],
    ["advanced", "dangerous", "Dangerous", "resetDb unsafeRpc enableUnsyncedMining"]
  ];
  const groups = definitions.map(([section, key, label, names]) => {
    const fields = names.split(" ").map(name => { const card = cards.get(name) || ""; cards.delete(name); return card; }).join("");
    const note = key === "dangerous"
      ? '<p class="kgw-danger-warning">Reset DB removes network data. Unsafe RPC can expose privileged methods. Unsynced mining bypasses synchronization. Existing confirmations and network restrictions still apply.</p>'
      : key === "experimental" ? '<p class="kgw-settings-info">' + esc(kgwNodeNetworkPolicyMessage(net.key)) + "</p>" : "";
    return [section, key, label, note + '<div class="kgw-settings-grid">' + fields + "</div>"];
  });
  if (cards.size) throw new Error("Ungrouped node settings: " + [...cards.keys()].join(", "));
  return renderSettingsTabs("node", net.key, groups);
}

/* R101U inner-tab persistence is Rust-owned in node_frontend_helpers.rs. */
function kgwNodeNormalizeInnerTabR101U(value) {
  return wasmNodeNormalizeInnerTab(value);
}

function kgwNodeResolveInnerTabR101U(net) {
  return wasmNodeResolveInnerTab(String(net || ""));
}

function kgwNodeSaveInnerTabR101U(net, selected) {
  return wasmNodeSaveInnerTab(String(net || ""), selected);
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
        <div class="kgw-settings-scroll">
        <section class="kgw-network-policy${net.experimental ? " is-experimental" : ""}" data-net="${net.key}" data-testid="kgw-node-policy-${net.key}">
          <div>
            <strong>${net.label}</strong>${net.experimental ? '<span class="kgw-experimental-badge">Experimental - opt-in required</span>' : ""}
            <span>${esc(kgwNodeNetworkPolicyMessage(net.key))}</span>
          </div>
          <div class="kgw-network-policy-controls">
            <span id="${id(net.key, "policyStatus")}" class="kgw-network-policy-status">Stopped</span>
            <label>
              <input type="checkbox" data-node-network-enabled="${net.key}" data-testid="kgw-node-policy-enabled-${net.key}" data-net="${net.key}"${kgwNodeNetworkEnabled(net.key) ? " checked" : ""}>
              Profile enabled
            </label>
          </div>
        </section>

        <section class="node-v6-command kgw-effective-preview">
          <div class="kgw-preview-row">
            <strong>Effective node settings</strong>
            <button type="button" data-settings-preview-toggle aria-expanded="false" aria-controls="${id(net.key, "previewBody")}">Expand</button>
            <button type="button" class="node-v6-copy" data-node-action="copy-command" data-net="${net.key}" title="Copy effective settings">Copy command</button>
            <button type="button" data-node-action="copy-path" data-net="${net.key}">Copy data directory</button>
          </div>
          <p id="${id(net.key, "previewMessage")}" class="kgw-preview-message" role="status" aria-live="polite"></p>
          <div class="kgw-preview-body" id="${id(net.key, "previewBody")}" hidden>
            <p class="kgw-preview-help">The embedded node library consumes these equivalent arguments inside KaspaGateway self-workers.</p>
            <textarea id="${id(net.key, "commandPreview")}" aria-label="Effective node settings preview" readonly spellcheck="false" wrap="soft"></textarea>
            <details class="kgw-arguments"><summary>Argument list</summary><pre id="${id(net.key, "argumentList")}"></pre></details>
          </div>
        </section>

        <section class="node-v6-toolbar">
          <div class="node-v6-buttons">
            <button type="button" class="good" data-node-action="start" data-testid="kgw-node-start-${net.key}" data-net="${net.key}">Start</button>
            <button type="button" data-node-action="stop" data-testid="kgw-node-stop-${net.key}" data-net="${net.key}">Stop</button>
          </div>

          <div class="node-v6-status">
            <span id="${id(net.key, "runtimeStatus")}" class="node-v6-runtime-status-pill" data-state="stopped">Stopped</span>
            <span id="${id(net.key, "runtimeEvidence")}" class="node-v6-runtime-evidence">No process owner</span>
            <span id="${id(net.key, "settingsAuthority")}" class="node-v6-runtime-evidence">Effective settings apply on next Start</span>
          </div>

          <div id="${id(net.key, "runtimeError")}" class="node-v6-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        ${renderSections(net)}
        </div>

        <div class="settings-bottom-actions node-settings-bottom-actions">
        <button type="button" data-node-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-node-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-node-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        <p class="kgw-settings-help" data-settings-defaults-context="${net.key}">Restore uses KaspaGateway defaults. Settings apply on the next Start.</p>
        </div>

      </div>

      <div class="node-v6-inner-panel${logActive ? " active" : ""}" data-net="${net.key}" data-node-inner-panel="log" data-testid="kgw-node-live-panel-${net.key}"${logActive ? "" : " hidden"}>
        <p id="${id(net.key, "monitorState")}" class="kgw-monitor-state" role="status">Node: Stopped</p>
        <div class="node-v6-log-toolbar">
          <button type="button" data-node-action="monitor-start" data-net="${net.key}">Start Node</button>
          <span class="node-v6-log-metadata" data-net="${net.key}">Network: ${net.label} | Source: self-worker | Streams: stdout/stderr</span>
          <button type="button" data-node-action="copy-log" data-testid="kgw-node-copy-log-${net.key}" data-net="${net.key}">Copy Log</button>
          <button type="button" data-node-action="clear-log" data-testid="kgw-node-clear-log-${net.key}" data-net="${net.key}">Clear Log</button>
        </div>
        <div id="${id(net.key, "logEmpty")}" class="node-v6-log-empty" data-node-log-empty="${net.key}">Node is stopped. Start the node to view its logs.</div>
        <pre id="${id(net.key, "logOutput")}" class="node-v6-log" data-testid="kgw-node-log-output-${net.key}"></pre>
      </div>
</div>`;
}

function renderAllNetworks(root) {
  const host = root.querySelector("#nodeNetworkPanels");
  if (!host) return;
  host.innerHTML = NODE_NETWORKS.map(renderNetworkPanel).join("");
  installSettingsLayout(root);


  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
  setTimeout(window.kgwInstallNodeLogScopedControlsV29, 0);
}




function kgwNodeEffectiveNumber(net, name, fallback, integer = false) {
  if (!kgwNodeSettingActive(net, name)) return fallback;
  const raw = v(net, name);
  if (!raw) return fallback;
  const value = Number(raw);
  if (!Number.isFinite(value) || (integer && !Number.isInteger(value))) {
    throw new Error(`${name} must be ${integer ? "an integer" : "a finite number"}.`);
  }
  return value;
}

function kgwNodeEffectiveEndpoint(net, enabledName, hostName, portName, _fallback = null) {
  if (!c(net, enabledName)) return null;
  const host = v(net, hostName);
  const port = v(net, portName);
  if (!host && !port) throw new Error(enabledName + " requires a host and port.");
  if (!host || !/^\d+$/.test(port) || Number(port) < 1 || Number(port) > 65535) {
    throw new Error(`${enabledName} requires a host and a port between 1 and 65535.`);
  }
  return endpoint(host, port);
}

function kgwNodeEffectiveNodeSettings(net) {
  const errors = kgwNodeValidateForm(net);
  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
  const profile = kgwNodeNetworkProfile(net);
  const rpcBase = net === "mainnet" ? 16110 : net === "testnet10" ? 16210 : 16210;
  const grpc = kgwNodeEffectiveEndpoint(
    net,
    "rpcListenEnabled",
    "rpcListenHost",
    "rpcListenPort",
    `127.0.0.1:${rpcBase}`
  );
  if (!grpc) throw new Error("The managed desktop owner requires gRPC RPC to remain enabled.");

  if (v(net, "configFile")) {
    throw new Error("--configfile is not supported by the managed desktop owner because network and database ownership must remain authoritative.");
  }
  if (v(net, "overrideParamsFile")) {
    throw new Error("--override-params-file is not supported because the desktop owns the selected network identity.");
  }
  if (c(net, "noLogFiles") && kgwNodeSettingActive(net, "logDir") && v(net, "logDir")) {
    throw new Error("--logdir and --nologfiles cannot be used together.");
  }
  const connect = kgwNodeEffectiveEndpoint(net, "connectEnabled", "connectHost", "connectPort");
  const addPeer = kgwNodeEffectiveEndpoint(net, "addPeerEnabled", "addPeerHost", "addPeerPort");
  const userAgentComment = kgwNodeCommandShouldIncludeR7(net, "uaComment") ? v(net, "uaComment") : "";

  return {
    logLevel: kgwNodeCommandShouldIncludeR7(net, "logLevel") ? v(net, "logLevel") || "info" : "info",
    asyncThreads: kgwNodeEffectiveNumber(net, "asyncThreads", 16, true),
    ramScale: kgwNodeEffectiveNumber(net, "ramScale", 1),
    yes: c(net, "yes"),
    noLogFiles: c(net, "noLogFiles"),
    sanity: c(net, "sanity"),
    enableUnsyncedMining: c(net, "enableUnsyncedMining") && Boolean(profile?.testnet),
    p2pListen: kgwNodeEffectiveEndpoint(net, "listenEnabled", "listenHost", "listenPort"),
    externalIp: kgwNodeEffectiveEndpoint(net, "externalIpEnabled", "externalIpHost", "externalIpPort"),
    disableUpnp: c(net, "disableUpnp"),
    disableDnsSeeding: c(net, "noDnsSeed"),
    userAgentComments: userAgentComment ? [userAgentComment] : [],
    rpcListen: grpc,
    rpcListenBorsh: kgwNodeEffectiveEndpoint(net, "rpcBorshEnabled", "rpcBorshHost", "rpcBorshPort"),
    rpcListenJson: kgwNodeEffectiveEndpoint(net, "rpcJsonEnabled", "rpcJsonHost", "rpcJsonPort"),
    rpcMaxClients: kgwNodeEffectiveNumber(net, "rpcMaxClients", 16, true),
    unsafeRpc: c(net, "unsafeRpc"),
    disableGrpc: c(net, "noGrpc"),
    connectPeers: connect ? [connect] : [],
    addPeers: addPeer ? [addPeer] : [],
    outboundTarget: kgwNodeEffectiveNumber(net, "outPeers", 8, true),
    inboundLimit: kgwNodeEffectiveNumber(net, "maxInPeers", 32, true),
    utxoIndex: c(net, "utxoIndex"),
    archival: c(net, "archival"),
    resetDb: c(net, "resetDb"),
    perfMetrics: c(net, "perfMetrics"),
    maxTrackedAddresses: kgwNodeEffectiveNumber(net, "maxTrackedAddresses", 0, true),
    retentionPeriodDays: kgwNodeCommandShouldIncludeR7(net, "retentionDays") && v(net, "retentionDays")
      ? kgwNodeEffectiveNumber(net, "retentionDays", null)
      : null,
    perfMetricsIntervalSec: kgwNodeEffectiveNumber(net, "perfMetricsInterval", 10, true),
    rocksDbPreset: kgwNodeCommandShouldIncludeR7(net, "rocksDbPreset") ? v(net, "rocksDbPreset") || null : null,
    rocksDbCacheSize: kgwNodeSettingActive(net, "rocksDbCacheSize") && v(net, "rocksDbCacheSize")
      ? kgwNodeEffectiveNumber(net, "rocksDbCacheSize", null, true)
      : null,
    rocksDbWalDir: kgwNodeCommandShouldIncludeR7(net, "rocksDbWalDir") ? v(net, "rocksDbWalDir") || null : null,
    overrideParamsFile: null,
    logDir: kgwNodeSettingActive(net, "logDir") ? v(net, "logDir") || null : null,
  };
}






const KGW_NODE_PREVIEWS = new Map();
function updateCommand(net) {
  const preview = byId(id(net, "commandPreview"));
  if (!preview) return;
  kgwNodeSyncDependencies(net);
  const errors = kgwNodeValidateForm(net);
  const state = KGW_NODE_PREVIEWS.get(net) || {sequence:0};
  clearTimeout(state.timer); state.sequence++;
  KGW_NODE_PREVIEWS.set(net,state);
  const sequence = state.sequence;
  preview.value = "";
  preview.dataset.effectiveSettingsAuthority = "validating";
  if (Object.keys(errors).length) {
    kgwNodePreviewMessage(net, "Correct the highlighted fields before saving or starting.", true);
    return;
  }
  kgwNodePreviewMessage(net, "Validating effective settings...");
  state.timer = setTimeout(async () => {
    try {
      const effective = kgwNodeEffectiveNodeSettings(net);
      const result = await kgwNodePreparePreview(net, effective);
      if (state.sequence !== sequence) return;
      preview.value = result.command;
      preview.dataset.effectiveSettingsAuthority = "validated-backend-settings";
      preview.dataset.arguments = JSON.stringify(result.arguments);
      const path = byId(id(net, "appDir")); if (path) { path.value = result.appDir; path.title = result.appDir; }
      const args = byId(id(net, "argumentList")); if (args) args.textContent = result.arguments.join("\n");
      kgwNodePreviewMessage(net, "Embedded kaspad - " + result.availableCpuThreads + " CPU threads available - configured " +
        effective.asyncThreads + " threads, RAM scale " + effective.ramScale + ". The managed data directory is included.");
    } catch (error) {
      if (state.sequence === sequence) kgwNodePreviewMessage(net, normalizeRuntimeError(error), true);
    }
  }, 160);
}
function kgwNodePreviewMessage(net, message, error = false) {
  const el = byId(id(net, "previewMessage"));
  if (el) {
    el.textContent = message;
    el.classList.toggle("kgw-field-error", error);
    applyStatusTone(el, error ? "error" : message.startsWith("Validating") ? "validating" : "verified");
  }
}
async function kgwNodePreparePreview(net, effective) {
  const invoke = getTauriInvoke();
  if (!invoke) throw new Error("Connect to the desktop backend to validate these settings.");
  return invokeWithTimeout(invoke, "kgw_node_settings_preview_v1",
    {network:net,effectiveNodeSettings:effective}, 10000);
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
  } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
}
/* KGW_NODE_LAST_NETWORK_RESTORE_R101W2 */
/* R101W2 last-network persistence is Rust-owned in node_frontend_helpers.rs. */
function kgwNodeNormalizeNetworkR101W2(value) {
  return wasmNodeNormalizeNetwork(value);
}
function kgwNodeReadLastNetworkR101W2() {
  return wasmNodeReadLastNetwork();
}
function kgwNodeSaveLastNetworkR101W2(net) {
  return wasmNodeSaveLastNetwork(net);
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
        item.setAttribute("aria-selected", String(item === sectionTab));
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
// The Node child contract is 90 seconds and the same-EXE parent is bounded at
// 100 seconds. Keep the UI request strictly above both terminal-result boundaries.
const KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS = 110000;
const KGW_NODE_STOP_INVOKE_TIMEOUT_MS = 0;


function getTauriInvoke() {
  return kgwResolvePublicTauriInvokeR1().invoke;
}

/* Runtime result/error/field parsing is Rust-owned in node_frontend_helpers.rs. */
function stringifyRuntimeResult(result) {
  return wasmNodeStringifyRuntimeResult(result);
}

function normalizeRuntimeError(error) {
  return wasmNodeNormalizeRuntimeError(error);
}

function parseRuntimeFields(result) {
  return wasmNodeParseRuntimeFields(result);
}

function kgwNodeSetRuntimeNotice(net, state, evidence = "", errorText = null, errorSource = "") {
  const status = byId(id(net, "runtimeStatus"));
  const evidenceNode = byId(id(net, "runtimeEvidence"));
  const errorNode = byId(id(net, "runtimeError"));
  const normalizedState = String(state || "Stopped");
  const stateKey = normalizedState.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "stopped";

  if (status) {
    status.textContent = normalizedState;
    status.dataset.state = stateKey;
    applyStatusTone(status, stateKey);
  }

  if (evidenceNode) {
    evidenceNode.textContent = evidence || "No process owner";
  }

  if (errorNode && errorText !== null && errorText !== undefined) {
    const text = String(errorText || "").trim();
    errorNode.textContent = text;
    errorNode.hidden = !text;
    errorNode.dataset.runtimeErrorSource = errorSource;
    applyStatusTone(errorNode, "error");
  }
}

function kgwNodeMarkRestartRequired(net) {
  const authority = byId(id(net, "settingsAuthority"));
  if (!authority) return;
  const running = byId(id(net, "runtimeStatus"))?.dataset?.state === "running";
  authority.textContent = running
    ? "Restart required to apply changed effective settings"
    : "Effective settings apply on next Start";
  authority.dataset.restartRequired = running ? "true" : "false";
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

  if (String(evidence.fields.readiness || "").toUpperCase() !== "READY") {
    throw new Error("Backend Start did not provide role readiness evidence: " + evidence.text);
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
      effectiveNodeSettings: kgwNodeEffectiveNodeSettings(net),
      runtimeRole: "node",
      experimentalNetworkOptIn: net === "testnet13" && kgwNodeNetworkEnabled(net),
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
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    return Promise.resolve().then(() => invoke(command, args));
  }

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
  if (command === "kgw_kgw_apply_node_settings_v1") {
    const prepared = await kgwNodePreparePreview(net, args.effectiveNodeSettings);
    args.nodeCommandPreview = prepared.command;
  }
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
      timeoutMs: action === "stop" ? KGW_NODE_STOP_INVOKE_TIMEOUT_MS : KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
    }
  });

  try {
    const invokeTimeoutMs = action === "stop"
      ? KGW_NODE_STOP_INVOKE_TIMEOUT_MS
      : KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS;
    const result = await invokeWithTimeout(resolved.invoke, command, args, invokeTimeoutMs);
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
    if (Object.keys(kgwNodeValidateForm(net, true)).length) return true;
    const dangerous = Object.entries(NODE_DANGEROUS).filter(([key]) => c(net, key));
    if (dangerous.length && !await confirmUserAction("Start " + net + " with these settings?\n\n" +
        dangerous.map(([key, warning]) => key + ": " + warning).join("\n") +
        "\n\nData directory: " + v(net, "appDir"))) return true;
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
  kgwNodeR51SetRuntimeButtons(net, action === "stop", kgwIsBridgeOwnedNodeLockedR65E(net));
  kgwNodeSetRuntimeNotice(net, action === "start" ? "Starting" : "Stopping", "Waiting for backend response", "");

  try {
    const result = await invokeNodeIntegratedRuntime(command, net);

    if (action === "start") {
      const evidence = kgwNodeAssertStartEvidence(net, result);
      KGW_NODE_R51_TRANSITIONS[net] = "";
      kgwNodeR51SetRuntimeButtons(net, true, kgwIsBridgeOwnedNodeLockedR65E(net));
      kgwNodeSetRuntimeNotice(
        net,
        "Running",
        "pid=" + evidence.pid + ";owner=" + evidence.owner + ";role=" + evidence.role + ";readiness=READY",
        ""
      );
    } else {
      const evidence = kgwNodeRuntimeEvidence(result);
      const graceful = String(evidence.fields.graceful || "").toLowerCase() === "true";
      const forced = String(evidence.fields.forced || "").toLowerCase() === "true";
      const stopFailed = evidence.fields.stop_failed === "true";
      const stopped = evidence.fields.running === "false" && (graceful || forced || stopFailed || evidence.fields.already_stopped === "true");
      if (!stopped) {
        throw new Error("Backend Stop did not confirm terminal process exit: " + evidence.text);
      }
      const forcedWarning = forced
        ? "Stop required FORCED termination. " + (evidence.fields.reason || evidence.text)
        : stopFailed
          ? "Official graceful shutdown failed, but the worker process exited. " + (evidence.fields.reason || evidence.text)
        : "";
      KGW_NODE_R51_TRANSITIONS[net] = "";
      kgwNodeR51SetRuntimeButtons(net, false, kgwIsBridgeOwnedNodeLockedR65E(net));
      kgwNodeSetRuntimeNotice(
        net,
        "Stopped",
        forced ? "FORCED termination confirmed" : stopFailed ? "Worker exited after graceful shutdown failure" : graceful ? "Graceful official shutdown confirmed" : "Already stopped",
        forcedWarning
      );
    }

    return true;
  } catch (error) {
    KGW_NODE_R51_TRANSITIONS[net] = "";
    const errorText = normalizeRuntimeError(error);
    kgwNodeR51SetRuntimeUnknown(net, errorText);
    kgwStartTraceFrontendR1("frontend.button_state_restored_after_failure", {
      network: net,
      action,
      result: "restored",
      details: {
        error: errorText,
        runningAfterFailure: null,
        reconciliationRequired: true,
        state: kgwNodeTraceStartButtonStateR1(net)
      }
    });
    return true;
  } finally {
    KGW_NODE_R51_TRANSITIONS[net] = "";
    window.__kgwR29NodeInFlight.delete(inFlightKey);
    window.setTimeout(() => { if (typeof kgwNodeR51RefreshOne === "function") void kgwNodeR51RefreshOne(net, "action-settled"); }, 0);
  }
}
/* KGW_R51_DIRECT_NODE_LOG_RUNTIME_SETTINGS_OWNER */
const KGW_NODE_R51_LAST_STATUS = {};
const KGW_NODE_R51_LAST_LOGS = {};
const KGW_NODE_R51_LAST_ACTIVITY_NOTICE = {};
const KGW_NODE_R51_TRANSITIONS = {};
const KGW_NODE_R51_STATUS_IN_FLIGHT = new Map();
const KGW_NODE_R51_LOGS_IN_FLIGHT = new Map();
let KGW_NODE_R51_TIMER = null;

function kgwNodeR51Keys() {
  return Array.from(wasmNodeR51Keys());
}

function kgwNodeR51Panel(net) {
  return wasmNodeR51Panel(String(net || ""));
}

function kgwNodeR51Fields(net) {
  return Array.from(wasmNodeR51Fields(String(net || "")));
}


/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START */





/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_END */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */



/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

function kgwNodeR51ReadSettings(net) {
  const values = wasmNodeR51ReadSettings(String(net || ""));
  kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-read-settings-command-options", {
    patch: "R38C",
    owner: "node-r51-settings-owner",
    commandOptionCount: Object.keys(values?.[KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C] || {}).length
  });
  return values;
}


/* KGW_NODE_COMMAND_CHECKBOX_PERSISTENCE_PATCH_R38C
 * Persist Node command include/exclude checkboxes by semantic keys, not empty DOM ids.
 * This patches the existing R51 settings persistence owner only.
 */
const KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C = wasmNodeCommandOptionsKey();

function kgwNodeR51WriteSettings(net, values) {
  if (!values || typeof values !== "object") return null;
  try {
    const result = wasmNodeR51WriteSettings(String(net || ""), values);
    if (result?.commandOptionsApplied) {
      kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restored", {
        patch: "R38C",
        owner: "node-r51-settings-owner",
        commandOptionCount: Number(result.commandOptionsCount || 0)
      });
    }
    if (result?.applied) updateCommand(net);
    return result;
  } catch (error) {
    kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restore-failed", {
      patch: "R38C",
      owner: "node-r51-settings-owner",
      message: error && error.message ? error.message : String(error)
    });
    throw error;
  }
}

function kgwNodeR51Store(key, value) {
  return wasmNodeR51Store(String(key || ""), value);
}

function kgwNodeR51Load(key) {
  try {
    return wasmNodeR51Load(String(key || ""));
  } catch {
    return null;
  }
}

function kgwNodeR51CaptureFactoryDefaults() {
  return wasmNodeR51CaptureFactoryDefaults();
}

function kgwNodeR51LoadSavedSettings() {
  const applied = Array.from(wasmNodeR51LoadSavedSettings() || []);
  for (const item of applied) {
    const net = String(item?.net || "");
    if (!net) continue;
    if (item?.commandOptionsApplied) {
      kgwNodeSmallOwnerTraceR44D(net, "settings-persistence", "r38c-command-options-restored", {
        patch: "R38C",
        owner: "node-r51-settings-owner",
        commandOptionCount: Number(item.commandOptionsCount || 0)
      });
    }
    updateCommand(net);
  }
  return applied;
}

/* KGW_NODE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */


function kgwNodeR51SaveSettings(net) {
  kgwNodeRequireValidSettings(net);
  kgwNodeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-begin", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });

  const result = wasmNodeR51SaveSettingsAction(String(net || ""));
  kgwNodeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-read-settings", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    keyCount: Number(result?.keyCount || 0),
    checkboxCount: Number(result?.checkboxCount || 0),
    valueCount: Number(result?.valueCount || 0),
    structuredInstanceCount: Number(result?.structuredInstanceCount || 0),
    hasActiveStructuredInstance: Boolean(result?.hasActiveStructuredInstance)
  });
  kgwNodeSmallOwnerTraceR44D(net, "save-settings", "r29b-save-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    savedKey: String(result?.storageKey || ("saved:" + String(net || ""))),
    persisted: Boolean(result?.persisted),
    persistedKeyCount: Number(result?.persistedKeyCount || 0)
  });
  return result;
}

function kgwNodeR51SetAsDefaults(net) {
  kgwNodeRequireValidSettings(net);
  kgwNodeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-begin", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });

  const result = wasmNodeR51SetDefaultsAction(String(net || ""));
  kgwNodeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-read-settings", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    keyCount: Number(result?.keyCount || 0),
    checkboxCount: Number(result?.checkboxCount || 0),
    valueCount: Number(result?.valueCount || 0),
    structuredInstanceCount: Number(result?.structuredInstanceCount || 0),
    hasActiveStructuredInstance: Boolean(result?.hasActiveStructuredInstance)
  });
  kgwNodeSmallOwnerTraceR44D(net, "set-defaults", "r29b-set-defaults-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner",
    defaultKey: String(result?.storageKey || ("default:" + String(net || ""))),
    persisted: Boolean(result?.persisted),
    persistedKeyCount: Number(result?.persistedKeyCount || 0)
  });
  return result;
}

/* R9B compatibility boundary: current input/change owners identify programmatic writes via Event.isTrusted. */
function kgwNodeSettingsWithProgrammaticWriteR9B(callback) {
  return callback();
}
function kgwNodeR51RestoreDefaults(net) {
  kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-begin", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });

  const result = kgwNodeSettingsWithProgrammaticWriteR9B(() => {
    const restored = wasmNodeR51RestoreDefaultsAction(String(net || ""));
    kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-loaded", {
      patch: "R29B",
      owner: "node-r51-settings-owner",
      hasDefaults: Boolean(restored?.hasDefaults),
      defaultKeyCount: Number(restored?.defaultKeyCount || 0)
    });
    kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true });
    return restored;
  });

  kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });
  return result;
}

function kgwNodeR51IsRunning(text) {
  return wasmNodeRuntimeIsRunning(text);
}

function kgwNodeRuntimeErrorFromStatus(text) {
  return wasmNodeRuntimeErrorFromStatus(text);
}

function kgwNodeR51SetRuntimeButtons(net, running, bridgeInprocessLocked = false, runtimeError = "", statusText = "") {
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
  const presentation = runtimePresentation({enabled: networkEnabled, running, transition, error: runtimeError});
  const runtimeState = presentation.process;

  if (policyStatus) {
    policyStatus.textContent = "Node: " + runtimeState;
    policyStatus.dataset.state = runtimeState.toLowerCase();
    applyStatusTone(policyStatus, runtimeState);
  }
  const summary = byId(id(net, "monitorState"));
  renderStatusSummary(summary, presentation.processLabel + " | Profile: " + presentation.profile +
    " | Startup readiness: " + (running ? "Verified" : transition ? "Pending" : "Not ready") +
    " | " + runtimeObservationSummary(parseRuntimeFields(statusText), running, false));
  const empty = byId(id(net, "logEmpty"));
  if (empty) empty.textContent = runtimeError ? "Node failed. Review the error in Settings and the logs below."
    : transitionActive ? "Node is " + runtimeState.toLowerCase() + ". Waiting for runtime output."
    : running ? "Node is running. Waiting for log output." : "Node is stopped. Start the node to view its logs.";
  const settingsInvalid = Boolean(panel.querySelector('[aria-invalid="true"]') ||
    byId(id(net, "previewStatus"))?.classList.contains("kgw-field-error"));
  const monitorStart = panel.querySelector('[data-node-action="monitor-start"]');
  if (monitorStart) {
    monitorStart.hidden = running || transitionActive;
    monitorStart.textContent = displayOnlyLocked ? "View Node Settings" : !networkEnabled ? "Enable Profile in Settings" : settingsInvalid ? "Review Settings" : "Start Node";
  }

  kgwNodeSetRuntimeNotice(
    net,
    runtimeState,
    running || starting ? "Self-worker process owner" : "No process owner",
    null
  );

  for (const field of kgwNodeR51Fields(net)) {
    field.disabled = Boolean(displayOnlyLocked);
    field.readOnly = Boolean(displayOnlyLocked);
    field.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    field.title = displayOnlyLocked ? lockMessage : "";
  }
  kgwNodeSyncDependencies(net, displayOnlyLocked);

  const preview = byId(id(net, "commandPreview"));
  if (preview) {
    preview.readOnly = true;
    preview.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    preview.title = displayOnlyLocked ? lockMessage : "";
  }

  if (start) {
    const startBlocked = Boolean(running || transitionActive || displayOnlyLocked || !networkEnabled || settingsInvalid);
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
    const stopEnabled = Boolean(running && !transitionActive && !displayOnlyLocked);
    stop.disabled = !stopEnabled;
    stop.style.opacity = stopEnabled ? "" : "0.45";
    stop.style.cursor = stopEnabled ? "" : "not-allowed";
    stop.setAttribute("aria-disabled", stopEnabled ? "false" : "true");
    stop.dataset.kgwBridgeInprocessLockedV7 = displayOnlyLocked ? "true" : "false";
    stop.title = displayOnlyLocked
      ? lockMessage
      : starting
        ? "Node startup is in progress. Stop becomes available after READY."
        : stopping
          ? "Node is stopping."
          : running
            ? "Stop node"
            : "Node is not running";
  }
}

function kgwNodeR51SetRuntimeUnknown(net, message = "Runtime status is temporarily unavailable. Reconciling with the backend.", errorSource = "") {
  const panel = kgwNodeR51Panel(net);
  if (!panel) return;

  const policyStatus = byId(id(net, "policyStatus"));
  if (policyStatus) {
    policyStatus.textContent = "Reconciling";
    policyStatus.dataset.state = "reconciling";
    applyStatusTone(policyStatus, "reconciling");
    const summary = byId(id(net, "monitorState"));
    renderStatusSummary(summary, "Node: Reconciling | RPC/synchronization/mining: unknown");
  }

  const start = panel.querySelector('[data-node-action="start"][data-net="' + net + '"]');
  const stop = panel.querySelector('[data-node-action="stop"][data-net="' + net + '"]');
  for (const button of [start, stop]) {
    if (!button) continue;
    button.disabled = true;
    button.setAttribute?.("aria-disabled", "true");
    button.style.opacity = "0.45";
    button.style.cursor = "not-allowed";
    button.title = message;
  }

  const currentError = byId(id(net, "runtimeError"));
  const preserveActionError = errorSource === "status-refresh" && currentError?.textContent?.trim() &&
    currentError.dataset.runtimeErrorSource !== "status-refresh";
  kgwNodeSetRuntimeNotice(net, "Reconciling", "Backend runtime state unavailable",
    preserveActionError ? null : message, errorSource);
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
  } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
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

  kgwNodeSyncDependencies(net, locked);
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

  window.KGW_NODE_SETTINGS_OWNER_V19?.setDisabled(document.getElementById("kaspa-node"), net, locked, "runtime-owner-reconcile");

  try {
    kgwNodeExplicitTraceR27D(net, "display-only", locked ? "r65e-node-display-only-enabled" : "r65e-node-display-only-cleared", {
      patch: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E",
      reason: reason || "unknown"
    });
  } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
}

async function kgwNodeR51BridgeInprocessLockedV7(net) {
  const observedLock = kgwBridgeOwnedNodeLockStoreR65E()[net];
  try {
    const invoke = getTauriInvoke();
    if (!invoke) return kgwIsBridgeOwnedNodeLockedR65E(net);
    const result = stringifyRuntimeResult(await invokeWithTimeout(
      invoke, "kgw_runtime_owner_status_v1",
      { network: net, runtimeRole: "bridge" }, KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS
    ));
    // A Start may have installed a newer owner while this status request was pending.
    if (kgwBridgeOwnedNodeLockStoreR65E()[net] !== observedLock) return kgwIsBridgeOwnedNodeLockedR65E(net);
    const fields = parseRuntimeFields(result);
    if (fields.role !== "bridge" || fields.network !== String(net) ||
        !["true", "false"].includes(fields.running)) return kgwIsBridgeOwnedNodeLockedR65E(net);
    const nodeMode = String(fields.node_mode || fields.nodeMode || "").toLowerCase();
    const pid = String(fields.pid || "").trim();
    const locked = fields.running === "true" && nodeMode === "inprocess";
    if (locked !== Boolean(observedLock?.locked) || (locked && observedLock?.details?.pid !== pid)) {
      kgwSetBridgeOwnedNodeLockR65E(net, locked, {
        source: "kgw_runtime_owner_status_v1", role: fields.role, nodeMode, pid
      });
    }
    return locked;
  } catch (_) {
    return kgwIsBridgeOwnedNodeLockedR65E(net);
  }
}


async function kgwNodeR51RefreshOne(net, _reason = "live") {
  const transition = String(KGW_NODE_R51_TRANSITIONS[net] || "");
  const transitionActive = transition === "starting" || transition === "stopping";

  let logsTask = KGW_NODE_R51_LOGS_IN_FLIGHT.get(net);
  if (!logsTask) {
    logsTask = (async () => {
      try {
        const report = await invokeNodeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net);
        kgwNodeApplyRuntimeLogReportV1(net, "node", report);
        KGW_NODE_R51_LAST_LOGS[net] = report;
      } catch (_) {
        // No raw buffer may exist before the child is spawned. Never fabricate text.
      } finally {
        if (KGW_NODE_R51_LOGS_IN_FLIGHT.get(net) === logsTask) {
          KGW_NODE_R51_LOGS_IN_FLIGHT.delete(net);
        }
      }
    })();
    KGW_NODE_R51_LOGS_IN_FLIGHT.set(net, logsTask);
  }

  let statusTask = Promise.resolve();
  if (!transitionActive) {
    statusTask = KGW_NODE_R51_STATUS_IN_FLIGHT.get(net);
    if (!statusTask) {
      statusTask = (async () => {
        try {
          const status = stringifyRuntimeResult(await invokeNodeIntegratedRuntime("kgw_runtime_owner_status_v1", net));
          const bridgeInprocessLocked = await kgwNodeR51BridgeInprocessLockedV7(net);
          const running = kgwNodeR51IsRunning(status);
          const runtimeError = kgwNodeRuntimeErrorFromStatus(status);
          const statusFields = parseRuntimeFields(status);
          const errorNode = byId(id(net, "runtimeError"));
          // A recovered poll must not leave its transient error or erase an action failure.
          if (!runtimeError && statusFields.role === "node" && statusFields.network === String(net) &&
              ["true", "false"].includes(statusFields.running) &&
              errorNode?.dataset.runtimeErrorSource === "status-refresh") {
            kgwNodeSetRuntimeNotice(net, running ? "Running" : "Stopped", "", "");
          }
          kgwNodeR51SetRuntimeButtons(net, running, bridgeInprocessLocked, runtimeError, status);
          if (!running && runtimeError) {
            kgwNodeSetRuntimeNotice(
              net,
              kgwNodeTranslateRuntimeV29("runtime.failed", "Failed"),
              "Official runtime terminated after READY",
              runtimeError,
            );
          }

          if (KGW_NODE_R51_LAST_STATUS[net] !== status) {
            KGW_NODE_R51_LAST_STATUS[net] = status;
            const authority = byId(id(net, "settingsAuthority"));
            if (authority && (!running || authority.dataset.restartRequired !== "true")) {
              authority.textContent = running
                ? kgwI18nTextR41("runtime.effectiveSettingsActive", "Effective settings are active for this runtime")
                : kgwI18nTextR41("runtime.effectiveSettingsNextStart", "Effective settings apply on next Start");
              authority.dataset.restartRequired = "false";
            }
          }
          kgwNodeR51MaybeActivityNotice(net, status);
        } catch (error) {
          kgwNodeR51SetRuntimeUnknown(net, "Status refresh failed: " + normalizeRuntimeError(error), "status-refresh");
        } finally {
          if (KGW_NODE_R51_STATUS_IN_FLIGHT.get(net) === statusTask) {
            KGW_NODE_R51_STATUS_IN_FLIGHT.delete(net);
          }
        }
      })();
      KGW_NODE_R51_STATUS_IN_FLIGHT.set(net, statusTask);
    }
  }

  await Promise.allSettled([logsTask, statusTask]);
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

function kgwNodeInstallBridgeOwnedDisplayOnlyHydrationR65H2(_root) {
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
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
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
  return wasmNodeCopyLogFailure(String(net || ""), button || null, error, details || {});
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
    await wasmNodeHandleCopyLog(String(net || ""), button || null);
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
    if (raw.includes("testnet13") || raw.includes("tn13")) return "testnet13";
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
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
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
    kgwNodeMarkRestartRequired(net);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", async (event) => {
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
    kgwNodeMarkRestartRequired(net);

    if (target.matches("[data-node-network-enabled]")) {
      const profile = kgwNodeNetworkProfile(net);
      const wasEnabled = kgwNodeNetworkEnabled(net);
      let enabled = Boolean(target.checked);

      if (enabled && profile?.experimental) {
        target.checked = false;
        target.disabled = true;
        try {
          enabled = (await confirmUserAction("Testnet 13 is experimental and uses a separate non-production runtime. Enable it only for isolated testing. Continue?")) === true;
        } catch (error) {
          enabled = false;
          kgwNodeSetRuntimeNotice(net, "Stopped", "Profile remains disabled", normalizeRuntimeError(error));
        } finally { target.disabled = false; target.checked = enabled; }
      }

      kgwNodeSetNetworkEnabled(net, enabled);
      kgwNodeR51SetRuntimeButtons(net, false, kgwIsBridgeOwnedNodeLockedR65E(net));

      if (!enabled && wasEnabled) {
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
      try {
        kgwNodeR51SaveSettings(net);
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        kgwNodeSetRuntimeNotice(net, "Settings error", "", normalizeRuntimeError(error));
      }
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      try {
        kgwNodeR51SetAsDefaults(net);
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        kgwNodeSetRuntimeNotice(net, "Settings error", "", normalizeRuntimeError(error));
      }
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      try {
        kgwNodeR51RestoreDefaults(net);
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        kgwNodeSetRuntimeNotice(net, "Settings error", "", normalizeRuntimeError(error));
      }
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

    if (action === "monitor-start") {
      panelStartFromMonitor(net);
      return;
    }
    if (action === "copy-command" || action === "copy-path") {
      void (async () => {
        let value = v(net, "appDir");
        if (action === "copy-command") {
          const errors = kgwNodeValidateForm(net);
          if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
          const sequence = KGW_NODE_PREVIEWS.get(net)?.sequence;
          const result = await kgwNodePreparePreview(net, kgwNodeEffectiveNodeSettings(net));
          if (KGW_NODE_PREVIEWS.get(net)?.sequence !== sequence) throw new Error("Settings changed while copying. Try again.");
          value = result.command;
        }
        if (!value) throw new Error("There is no validated value to copy.");
        await kgwNodeDispatchClipboardWriteV1(net, value, {characterCount: [...value].length, lineCount: value.split(/\r?\n/).length});
        kgwNodePreviewMessage(net, action === "copy-path" ? "Data directory copied." : "Command copied.");
      })().catch(error => kgwNodePreviewMessage(net, "Copy failed: " + normalizeRuntimeError(error), true));
      return;
    }

    if (action === "start" || action === "stop") {
      if (typeof runNodeIntegratedAction === "function") {
        runNodeIntegratedAction(action, net).catch(function (error) {
          const message = error && error.message ? error.message : String(error);
          kgwNodeSetRuntimeNotice(net, action === "stop" ? "Running" : "Stopped", action === "stop" ? "Stop failed" : "No process owner", message);
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
  const NETWORKS = ["mainnet", "testnet10", "testnet13"];
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
    } catch (_) { /* Best-effort secondary operation; primary node behavior is preserved. */ }
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
 * - testnet13 requires isolated-real-runtime-tn13 build
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


function kgwNodeForm(net) {
  const values = {};
  const panel = kgwNodeR51Panel(net);
  panel?.querySelectorAll(".node-v6-card input[id], .node-v6-card select[id]").forEach(field => {
    const name = field.id.slice(("node-" + net + "-").length);
    values[name] = field.type === "checkbox" ? field.checked : field.value;
  });
  return values;
}
function kgwNodeSettingActive(net, name) {
  return nodeFieldEnabled(name, kgwNodeForm(net), kgwNodeCommandInlineStateR7(net));
}
function kgwNodeValidateForm(net, focus = false) {
  const errors = validateNodeForm(kgwNodeForm(net), kgwNodeCommandInlineStateR7(net), net);
  const panel = kgwNodeR51Panel(net);
  renderFieldErrors(panel, "node-" + net + "-", errors);
  if (focus && Object.keys(errors).length) {
    const field = byId(id(net, Object.keys(errors)[0]));
    panel?.querySelector('[data-node-inner-tab="settings"]')?.click();
    const section = field?.closest("[data-node-section-panel]");
    panel?.querySelector('[data-node-section-tab="' + section?.dataset.nodeSectionPanel + '"]')?.click();
    revealSettingsField(field);
    field?.focus();
  }
  return errors;
}
function kgwNodeRequireValidSettings(net) {
  const errors = kgwNodeValidateForm(net, true);
  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
  kgwNodeEffectiveNodeSettings(net);
}
function kgwNodeSyncDependencies(net, locked = kgwIsBridgeOwnedNodeLockedR65E(net)) {
  const values = kgwNodeForm(net), options = kgwNodeCommandInlineStateR7(net);
  const panel = kgwNodeR51Panel(net);
  for (const [name] of Object.entries(values)) {
    const field = byId(id(net, name));
    const managed = NODE_MANAGED[name];
    const experimentalOnly = name === "enableUnsyncedMining" && net === "mainnet";
    const active = nodeFieldEnabled(name, values, options) && !experimentalOnly;
    field.disabled = Boolean(locked || (!active && name !== "appDir"));
    field.readOnly = Boolean(locked || managed);
    field.title = locked ? "Stop the bridge that owns this node to edit settings."
      : managed || (experimentalOnly ? "Available only on test networks." : !active ? "Enable the parent option to use this value." : field.value || "");
    field.closest(".node-v6-card")?.classList.toggle("kgw-field-inactive", !active);
    const state = managed ? (/unsupported/i.test(managed) ? "Unsupported" : "Managed")
      : NODE_DANGEROUS[name] ? "Dangerous"
      : Object.hasOwn(NODE_REQUIRED, name) ? (String(values[name]) === NODE_REQUIRED[name] ? "KGW default" : "Custom value")
      : experimentalOnly ? "Test networks only"
      : !active ? "Not active" : "";
    setSettingFieldState(field, state);
  }
  panel?.querySelectorAll("[data-node-command-option-toggle-r7]").forEach(toggle => {
    const name = toggle.dataset.nodeCommandOptionToggleR7;
    toggle.disabled = Boolean(locked || (name === "logDir" && values.noLogFiles) ||
      (name === "perfMetricsInterval" && !values.perfMetrics) ||
      (name === "rocksDbCacheSize" && (!options.rocksDbPreset || values.rocksDbPreset !== "hdd")));
  });
  decorateSettingsFields(panel);
}

export { kgwNodeEffectiveNodeSettings, kgwNodeValidateForm };

function panelStartFromMonitor(net) {
  const panel = kgwNodeR51Panel(net);
  const start = panel?.querySelector('[data-node-action="start"]');
  if (start?.disabled) {
    panel.querySelector('[data-node-inner-tab="settings"]')?.click();
    kgwNodePreviewMessage(net, start.title || "Check the profile and settings before starting.", true);
  } else start?.click();
}
