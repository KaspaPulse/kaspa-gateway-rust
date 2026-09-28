import { applyStatusTone, renderStatusSummary } from "../../status.js";
import { NODE_DANGEROUS, runtimePresentation, runtimeObservationSummary, confirmUserAction } from "../../settings-contract.js";
import { installSettingsLayout } from "../../settings-layout.js";
import initNodeRust, {
  nodeApplyRootDefaultPath as wasmNodeApplyRootDefaultPath,
  nodeById as byId,
  nodeChecked as c,
  nodeCommandInlineState as kgwNodeCommandInlineStateR7,
  nodeRefreshInlineCommandToggles as kgwNodeRefreshInlineCommandTogglesR7,
  nodeToggleCommandOptionAndUpdate as wasmNodeToggleCommandOptionAndUpdate,
  nodeApplyRuntimeLogReport as kgwNodeApplyRuntimeLogReportV1,
  nodeCopyLogFailure as kgwNodeCopyLogFailureV1,
  nodeDispatchClipboardWrite as kgwNodeDispatchClipboardWriteV1,
  nodeEffectiveNodeSettings as kgwNodeEffectiveNodeSettings,
  nodeValidateForm as kgwNodeValidateForm,
  nodeSuperMegaIsolatedAdapterStatusPreviewV1 as wasmNodeSuperMegaIsolatedAdapterStatusPreviewV1,
  nodeFinalIsolatedAdapterStartV1 as wasmNodeFinalIsolatedAdapterStartV1,
  nodeFinalIsolatedAdapterStatusV1 as wasmNodeFinalIsolatedAdapterStatusV1,
  nodeFinalIsolatedAdapterStopV1 as wasmNodeFinalIsolatedAdapterStopV1,
  nodeV66RuntimeFeaturePolicyV1 as wasmNodeV66RuntimeFeaturePolicyV1,
  nodeV66IsolatedAdapterStartV1 as wasmNodeV66IsolatedAdapterStartV1,
  nodeV66IsolatedAdapterStatusV1 as wasmNodeV66IsolatedAdapterStatusV1,
  nodeV66IsolatedAdapterStopV1 as wasmNodeV66IsolatedAdapterStopV1,
  nodeV67StartRuntime as wasmNodeV67StartRuntime,
  nodeV67StatusRuntime as wasmNodeV67StatusRuntime,
  nodeV67StopRuntime as wasmNodeV67StopRuntime,
  nodeV67RuntimeFeaturePolicy as wasmNodeV67RuntimeFeaturePolicy,
  nodeElementId as id,
  nodeExplicitTrace as kgwNodeExplicitTraceR27D,
  nodeExplicitOwnerTrace as kgwNodeExplicitOwnerTraceR27D,
  nodeHandleLogAction as kgwNodeHandleLogActionV29,
  nodeI18nText as kgwI18nTextR41,
  nodeInstallDelegatedTabs as wasmNodeInstallDelegatedTabs,
  nodeInstallLogAutoScrollControls as kgwInstallNodeLogAutoScrollControlsR27,
  nodeInstallNetworkTabs as wasmNodeInstallNetworkTabs,
  nodeInstallStartTraceDocumentClickObserver as kgwNodeInstallStartTraceDocumentClickObserverR1,
  nodeNetworkEnabled as kgwNodeNetworkEnabled,
  nodeNetworkProfile as kgwNodeNetworkProfile,
  nodeNetworkProfiles as wasmNodeNetworkProfiles,
  nodePanelStartFromMonitor as panelStartFromMonitor,
  nodePreparePreview as wasmNodePreparePreview,
  nodePreviewMessage as kgwNodePreviewMessage,
  nodePreviewSequence as wasmNodePreviewSequence,
  nodeAssertStartEvidence as kgwNodeAssertStartEvidence,
  nodeNormalizeRuntimeError as normalizeRuntimeError,
  nodeParseRuntimeFields as parseRuntimeFields,
  nodeRuntimeEvidence as kgwNodeRuntimeEvidence,
  nodeR51CaptureFactoryDefaults as kgwNodeR51CaptureFactoryDefaults,
  nodeR51Fields as kgwNodeR51Fields,
  nodeR51Keys as kgwNodeR51Keys,
  nodeR51Load as kgwNodeR51Load,
  nodeR51LoadSavedSettings as wasmNodeR51LoadSavedSettings,
  nodeR51Panel as kgwNodeR51Panel,
  nodeR51ReadSettingsTracked as kgwNodeR51ReadSettings,
  nodeRenderNetworkPanelsHtml as wasmNodeRenderNetworkPanelsHtml,
  nodeR51RestoreDefaultsAction as wasmNodeR51RestoreDefaultsAction,
  nodeR51SaveSettings as kgwNodeR51SaveSettings,
  nodeR51SetAsDefaults as kgwNodeR51SetAsDefaults,
  nodeResolvePublicTauriInvoke as kgwResolvePublicTauriInvokeR1,
  nodeRuntimeActionForCommand as kgwNodeRuntimeActionForCommandR1,
  nodeRuntimeArgs as nodeRuntimeArgs,
  nodeRuntimeErrorFromStatus as kgwNodeRuntimeErrorFromStatus,
  nodeRuntimeIsRunning as kgwNodeR51IsRunning,
  nodeSetNetworkEnabled as kgwNodeSetNetworkEnabled,
  nodeStringifyRuntimeResult as stringifyRuntimeResult,
  nodeSyncDependencies as wasmNodeSyncDependencies,
  nodeSmallOwnerTrace as kgwNodeSmallOwnerTraceR44D,
  nodeStartTraceFrontend as kgwStartTraceFrontendR1,
  nodeTraceActiveNetwork as kgwNodeTraceActiveNetworkR1,
  nodeTraceRenderedStartControls as kgwNodeTraceRenderedStartControlsR1,
  nodeTraceStartButtonState as kgwNodeTraceStartButtonStateR1,
  nodeUpdateCommand as wasmNodeUpdateCommand,
  nodeValue as v,
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
/* KGW_NODE_TRACE_OBSERVER_R1 is Rust-owned in node_start_trace.rs. */


/* Canonical isolated node runtime paths.
 * Each network owns a separate database below:
 * %LOCALAPPDATA%\KaspaGateway\nodes\<network>
 */
/* KGW_NODE_PATH_HELPERS_R5 are Rust-owned in node_path_helpers.rs. */










const NODE_NETWORKS = wasmNodeNetworkProfiles();

// KGW_NODE_COMMAND_COMPOSER_INLINE_TOGGLE_R7
// Root-path restore, command preview debounce/sequence, and option-toggle refresh
// are Rust/WASM-owned in node_frontend_helpers.rs.

/* Node card markup rendering is Rust-owned in node_frontend_helpers.rs. */

// KGW_NODE_LOG_AUTOSCROLL_CONTROLS_R27 is Rust-owned in node_frontend_helpers.rs.

/* KGW_NODE_RAW_LOG_OWNER_V2 is Rust-owned in node_start_trace.rs. */

/* Node settings/network panel HTML is Rust/WASM-owned in node_frontend_helpers.rs. */
function renderAllNetworks(root) {
  const host = root.querySelector("#nodeNetworkPanels");
  if (!host) return;
  host.innerHTML = wasmNodeRenderNetworkPanelsHtml();
  installSettingsLayout(root);
  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
}

/* Effective Node settings + validation + preview orchestration are Rust/WASM-owned. */



// KGW_NODE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45F is Rust/WASM-owned in node_start_trace.rs.
/* KGW_NODE_LAST_NETWORK_RESTORE_R101W2 */
/* R101W2 last-network persistence is Rust-owned in node_frontend_helpers.rs. */
// Network tabs plus delegated inner/section navigation are Rust/WASM-owned
// by node_frontend_helpers.rs. Bridge/runtime callbacks remain explicit at init.

// KGW_NODE_INTEGRATED_RUNTIME_LINKAGE_V1: crash-safe Node action owner calls registered Tauri integrated runtime commands.
// The Node child contract is 90 seconds and the same-EXE parent is bounded at
// 100 seconds. Keep the UI request strictly above both terminal-result boundaries.
const KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS = 110000;
const KGW_NODE_STOP_INVOKE_TIMEOUT_MS = 0;


/* Runtime result/error/field parsing is Rust-owned in node_frontend_helpers.rs. */
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

/* Runtime IPC argument construction is Rust/WASM-owned in node_frontend_helpers.rs. */
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
    const prepared = await wasmNodePreparePreview(net, args.effectiveNodeSettings);
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

/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_START */





/* KGW_NODE_SETTINGS_LIFECYCLE_FIX_R6_END */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */



/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

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
    wasmNodeUpdateCommand(net, kgwIsBridgeOwnedNodeLockedR65E(net));
  }
  return applied;
}

/* KGW_NODE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */


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
    void wasmNodeApplyRootDefaultPath(String(net || ""), kgwIsBridgeOwnedNodeLockedR65E(net)).catch((error) => {
      kgwNodePreviewMessage(net, "Rusty Kaspa root-only default path restore failed: " + normalizeRuntimeError(error), true);
    });
    return restored;
  });

  kgwNodeSmallOwnerTraceR44D(net, "restore-defaults", "r29b-restore-defaults-complete", {
    patch: "R29B",
    owner: "node-r51-settings-owner"
  });
  return result;
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
  wasmNodeSyncDependencies(net, displayOnlyLocked);

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

  wasmNodeSyncDependencies(net, locked);
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
    const invoke = kgwResolvePublicTauriInvokeR1().invoke;
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
              kgwI18nTextR41("runtime.failed", "Failed"),
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



/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29 is Rust/WASM-owned in node_start_trace.rs. */

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
          wasmNodeUpdateCommand(net, kgwIsBridgeOwnedNodeLockedR65E(net));
          if (typeof kgwNodeRefreshInlineCommandTogglesR7 === "function") {
            kgwNodeRefreshInlineCommandTogglesR7(net);
          }
        } else {
          wasmNodeToggleCommandOptionAndUpdate(String(net || ""), String(option || ""), kgwIsBridgeOwnedNodeLockedR65E(net));
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
        wasmNodeToggleCommandOptionAndUpdate(String(toggle.dataset.net || ""), String(toggle.dataset.nodeCommandOptionToggleR7 || ""), kgwIsBridgeOwnedNodeLockedR65E(toggle.dataset.net));
      }
    });

    root.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      const toggle = event.target.closest("[data-node-command-option-toggle-r7]");
      if (toggle && root.contains(toggle)) {
        event.preventDefault();
        event.stopPropagation();
        wasmNodeToggleCommandOptionAndUpdate(String(toggle.dataset.net || ""), String(toggle.dataset.nodeCommandOptionToggleR7 || ""), kgwIsBridgeOwnedNodeLockedR65E(toggle.dataset.net));
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


  // KGW_EXPLICIT_TRACE_OWNER_R27D_NODE is Rust/WASM-owned in node_start_trace.rs.

  function scopedUpdate(net, reason) {
    if (!net) return;
    wasmNodeUpdateCommand(net, kgwIsBridgeOwnedNodeLockedR65E(net));
    kgwNodeExplicitOwnerTraceR27D(net, "settings-scope", "r27d-scoped-update", {
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
      kgwNodeExplicitOwnerTraceR27D(inputNet, "display-only", "r65e-node-input-blocked", {
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
      kgwNodeExplicitOwnerTraceR27D(changeNet, "display-only", "r65e-node-change-blocked", {
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
      kgwNodeExplicitOwnerTraceR27D(net, String(action || "unknown"), "r27d-action-click", {
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
          const sequence = wasmNodePreviewSequence(net);
          const result = await wasmNodePreparePreview(net, kgwNodeEffectiveNodeSettings(net));
          if (wasmNodePreviewSequence(net) !== sequence) throw new Error("Settings changed while copying. Try again.");
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
  wasmNodeInstallNetworkTabs(nodeRoot, {
    isLocked: kgwIsBridgeOwnedNodeLockedR65E,
    applyDisplayOnly: kgwNodeApplyBridgeOwnedDisplayOnlyR65E,
    setRuntimeButtons: kgwNodeR51SetRuntimeButtons,
    hydrate: kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2
  });
  wasmNodeInstallDelegatedTabs(nodeRoot);
  installActions(nodeRoot);
  NODE_NETWORKS.forEach((net) => wasmNodeUpdateCommand(net.key, kgwIsBridgeOwnedNodeLockedR65E(net.key)));
  NODE_NETWORKS.forEach((net) => {
    void wasmNodeApplyRootDefaultPath(net.key, kgwIsBridgeOwnedNodeLockedR65E(net.key)).catch((error) => {
      kgwNodePreviewMessage(net.key, "Rusty Kaspa root-only default path restore failed: " + normalizeRuntimeError(error), true);
    });
  }); /* KGW_NODE_DYNAMIC_PATHS_INIT_R3 */
  kgwNodeR51StartLiveRefresh();

  setTimeout(kgwInstallNodeLogAutoScrollControlsR27, 0);
}


/* KGW_NODE_LOG_SCOPED_CONTROLS_V29 is Rust/WASM-owned in node_frontend_helpers.rs. */

export default initKaspaNodeTab;

if (typeof window !== "undefined") {
  window.initKaspaNodeTab = initKaspaNodeTab;
}



/* OP156 isolated runtime compatibility implementation is Rust/WASM-owned. */
window.kgwSuperMegaIsolatedAdapterStatusPreviewV1 = wasmNodeSuperMegaIsolatedAdapterStatusPreviewV1;
window.kgwFinalIsolatedAdapterStartV1 = wasmNodeFinalIsolatedAdapterStartV1;
window.kgwFinalIsolatedAdapterStatusV1 = wasmNodeFinalIsolatedAdapterStatusV1;
window.kgwFinalIsolatedAdapterStopV1 = wasmNodeFinalIsolatedAdapterStopV1;
window.kgwV66RuntimeFeaturePolicyV1 = wasmNodeV66RuntimeFeaturePolicyV1;
window.kgwV66IsolatedAdapterStartV1 = wasmNodeV66IsolatedAdapterStartV1;
window.kgwV66IsolatedAdapterStatusV1 = wasmNodeV66IsolatedAdapterStatusV1;
window.kgwV66IsolatedAdapterStopV1 = wasmNodeV66IsolatedAdapterStopV1;
window.kgwV67StartRuntime = wasmNodeV67StartRuntime;
window.kgwV67StatusRuntime = wasmNodeV67StatusRuntime;
window.kgwV67StopRuntime = wasmNodeV67StopRuntime;
window.kgwV67RuntimeFeaturePolicy = wasmNodeV67RuntimeFeaturePolicy;
/* R35 settings persistence for existing Node tab. */
/* R37 bottom placement for Node settings buttons. */
/* R38 UI freeze protection for Node Start/Stop. */


/* Node form collection, dependency synchronization, and monitor Start routing are Rust/WASM-owned. */

export { kgwNodeEffectiveNodeSettings, kgwNodeValidateForm };
