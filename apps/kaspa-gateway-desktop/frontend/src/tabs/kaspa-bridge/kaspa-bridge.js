import { confirmUserAction } from "../../settings-contract.js";
import { installSettingsLayout, decorateSettingsFields } from "../../settings-layout.js";
import initBridgeRust, {
  bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5 as wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5,
  bridgeById as wasmBridgeById,
  bridgeChecked as wasmBridgeChecked,
  bridgeRefreshInlineCommandTogglesR7 as wasmBridgeRefreshInlineCommandTogglesR7,
  bridgeCommandSetOptionR7 as wasmBridgeCommandSetOptionR7,
  bridgeCommandToggleOptionR7 as wasmBridgeCommandToggleOptionR7,
  bridgeBuildCommandLinesUi as wasmBridgeBuildCommandLinesUi,
  bridgeBuildApplyPayloadUi as wasmBridgeBuildApplyPayloadUi,
  bridgeNormalizeRuntimeError as wasmBridgeNormalizeRuntimeError,
  bridgeV7BlockInprocessIfNodeOwnerRunning as wasmBridgeV7BlockInprocessIfNodeOwnerRunning,
  bridgeR51SetRuntimeButtons as wasmBridgeR51SetRuntimeButtons,
  bridgeR51SetRuntimeUnknown as wasmBridgeR51SetRuntimeUnknown,
  bridgeSetRuntimeErrorV1 as wasmBridgeSetRuntimeErrorV1,
  bridgeSetRuntimeActivityV1 as wasmBridgeSetRuntimeActivityV1,
  bridgeMarkRestartRequiredV1 as wasmBridgeMarkRestartRequiredV1,
  bridgeR51RefreshOne as wasmBridgeR51RefreshOne,
  bridgeR51StartLiveRefresh as wasmBridgeR51StartLiveRefresh,
  bridgeR51CaptureFactoryDefaults as wasmBridgeR51CaptureFactoryDefaults,
  bridgeR51LoadSavedSettings as wasmBridgeR51LoadSavedSettings,
  bridgeR51SaveSettings as wasmBridgeR51SaveSettings,
  bridgeR51SetAsDefaults as wasmBridgeR51SetAsDefaults,
  bridgeR51RestoreDefaults as wasmBridgeR51RestoreDefaults,
  bridgeR51Load as wasmBridgeR51Load,
  bridgeR51Keys as wasmBridgeR51Keys,
  bridgeR51Panel as wasmBridgeR51Panel,
  bridgeR51ReadStructuredInstances as wasmBridgeR51ReadStructuredInstances,
  bridgeR51ReadSettings as wasmBridgeR51ReadSettings,
  bridgeR51WriteSettings as wasmBridgeR51WriteSettings,
  bridgePreviewDeclaresInprocessR65F as wasmBridgePreviewDeclaresInprocessR65F,
  bridgeRuntimeCommandForAction as wasmBridgeRuntimeCommandForAction,
  bridgeRuntimeActionOutcome as wasmBridgeRuntimeActionOutcome,
  bridgeStartWasInprocessR65F as wasmBridgeStartWasInprocessR65F,
  bridgeCurrentNodeModeFromUiR65F as wasmBridgeCurrentNodeModeFromUiR65F,
  bridgeR95BNormalizeNetworkPortValues as wasmBridgeR95BNormalizeNetworkPortValues,
  bridgeInvokeRuntimeCommand as wasmBridgeInvokeRuntimeCommand,
  bridgePreparePreview as wasmBridgePreparePreview,
  bridgePreviewMessage as wasmBridgePreviewMessage,
  bridgePreviewSequence as wasmBridgePreviewSequence,
  bridgeUpdateCommandUi as wasmBridgeUpdateCommandUi,
  bridgeUpdateAllCommandsUi as wasmBridgeUpdateAllCommandsUi,
  bridgeEffectiveSettingsV1 as wasmBridgeEffectiveSettingsV1,
  bridgeSetInstanceCommandOptionUiR13B as wasmBridgeSetInstanceCommandOptionUiR13B,
  bridgeRefreshInstancesUi as wasmBridgeRefreshInstancesUi,
  bridgeInstallInstanceContainerOwnerR11 as wasmBridgeInstallInstanceContainerOwnerR11,
  bridgeInstallAllVisibleInstanceContainerOwnersR11 as wasmBridgeInstallAllVisibleInstanceContainerOwnersR11,

  bridgeDispatchClipboardWrite as wasmBridgeDispatchClipboardWrite,
  bridgeElementId as wasmBridgeElementId,
  bridgeHandleLogAction as wasmBridgeHandleLogAction,
  bridgeInstallLogAutoScrollControls as wasmBridgeInstallLogAutoScrollControls,
  bridgeNetworkEnabled as wasmBridgeNetworkEnabled,
  bridgeNormalizeNetwork as wasmBridgeNormalizeNetwork,
  bridgeReadLastNetwork as wasmBridgeReadLastNetwork,
  bridgeSaveLastNetwork as wasmBridgeSaveLastNetwork,
  bridgeNetworkProfile as wasmBridgeNetworkProfile,
  bridgeNetworkProfiles as wasmBridgeNetworkProfiles,
  bridgeRenderInprocessNodeSettingsUi as wasmBridgeRenderInprocessNodeSettingsUi,
  bridgeRenderSectionsUi as wasmBridgeRenderSectionsUi,
  bridgeRenderNetworkPanelUi as wasmBridgeRenderNetworkPanelUi,
  bridgeRenderAllNetworksUi as wasmBridgeRenderAllNetworksUi,
  bridgeSaveInnerTab as wasmBridgeSaveInnerTab,
  bridgeRenderRawLogBuffer as wasmBridgeRenderRawLogBuffer,
  bridgeSmallOwnerTraceR44D as wasmBridgeSmallOwnerTraceR44D,
  bridgeSetNetworkEnabled as wasmBridgeSetNetworkEnabled,
  bridgeValidateFormUi as wasmBridgeValidateFormUi,
  bridgeRequireValidSettingsUi as wasmBridgeRequireValidSettingsUi,
  bridgeEffectiveInprocessNodeSettingsChecked as wasmBridgeEffectiveInprocessNodeSettingsChecked,
  bridgeSyncModeControlsUi as wasmBridgeSyncModeControlsUi,
  bridgeSyncAllModeControlsUi as wasmBridgeSyncAllModeControlsUi,
  bridgeSetOwnedNodeLockR65E as wasmBridgeSetOwnedNodeLockR65E,
  bridgeValue as wasmBridgeValue,
  bridgeAddInstanceUi as wasmBridgeAddInstanceUi,
  bridgeRemoveInstanceUi as wasmBridgeRemoveInstanceUi,
  bridgeActiveRawLogInstanceId as wasmBridgeActiveRawLogInstanceId,
  bridgeAssertNoPortConflictsR5 as wasmBridgeAssertNoPortConflictsR5,
  bridgeValidateAllPortConflictStatesR33 as wasmBridgeValidateAllPortConflictStatesR33,
  bridgeSchedulePortConflictValidationR33 as wasmBridgeSchedulePortConflictValidationR33,
  bridgeAutofixButtonInitialLabelUiR111G as wasmBridgeAutofixButtonInitialLabelUiR111G,
  bridgeRefreshPortAutofixButtonsUiR37 as wasmBridgeRefreshPortAutofixButtonsUiR37,
  bridgeSchedulePortAutofixRefreshUiR37 as wasmBridgeSchedulePortAutofixRefreshUiR37,
  bridgeApplyPortAutofixUiR37 as wasmBridgeApplyPortAutofixUiR37,
  bridgeInstallPortAutofixButtonUiR37 as wasmBridgeInstallPortAutofixButtonUiR37,
  settingsOwnerButtons as wasmSettingsOwnerButtons,
  settingsOwnerInstall as wasmSettingsOwnerInstall,
  settingsOwnerSetDisabled as wasmSettingsOwnerSetDisabled,
} from "../../../generated/kgw_frontend_wasm/kgw_frontend_wasm.js";

await initBridgeRust();

// KGW_SETTINGS_OWNER_V19
// Shared Rust owner: Bridge keeps only callback bindings and compatibility globals.
function kgwBridgeSettingsOwnerCallbacksV19() {
  return {
    scope: "bridge",
    keys: () => wasmBridgeR51Keys(),
    readSettings: (net) => kgwBridgeR51ReadSettingsR249(String(net || "")),
    load: (key) => wasmBridgeR51Load(String(key || "")),
    validateForm: (net, focus = false) => kgwBridgeValidateForm(String(net || ""), Boolean(focus))
  };
}

const KGW_BRIDGE_SETTINGS_OWNER_V19 = {
  install(root) {
    return wasmSettingsOwnerInstall(root, kgwBridgeSettingsOwnerCallbacksV19());
  },
  setDisabled(root, network, disabled, reason = "") {
    return wasmSettingsOwnerSetDisabled(
      root,
      String(network || ""),
      Boolean(disabled),
      String(reason || ""),
      kgwBridgeSettingsOwnerCallbacksV19()
    );
  },
  buttons(root, network = "all") {
    return wasmSettingsOwnerButtons(root, String(network || "all"));
  }
};

window.KGW_BRIDGE_SETTINGS_OWNER_V19 = KGW_BRIDGE_SETTINGS_OWNER_V19;
window.KGW_SETTINGS_OWNER_V19 = KGW_BRIDGE_SETTINGS_OWNER_V19;
// END_KGW_SETTINGS_OWNER_V19









/* Canonical isolated bridge/node runtime paths.
 * In-process bridge mode shares the same network-specific database at:
 * %LOCALAPPDATA%\KaspaGateway\nodes\<network>
 */







/* KGW_BRIDGE_NETWORK_PORT_RANGES_DEFAULTS_PATCH_R42
 * Defaults now follow the agreed soft network port ranges.
 * These are defaults only. Manual valid unused ports remain accepted anywhere.
 */
const BRIDGE_NETWORKS = wasmBridgeNetworkProfiles();











const bridgeInstances = {
  mainnet: [{ id: 1 }],
  testnet10: [{ id: 1 }],
  testnet13: [{ id: 1 }]
};

let activeInstance = {
  mainnet: 1,
  testnet10: 1,
  testnet13: 1
};














// KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B

function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {
  return wasmBridgeSetInstanceCommandOptionUiR13B(
    String(net || ""),
    instanceId,
    String(name || ""),
    Boolean(enabled),
    bridgeInstances,
    activeInstance,
    { updateCommand: (targetNet) => updateCommand(String(targetNet || "")) }
  );
}


// KGW_BRIDGE_COMMAND_COMPOSER_INLINE_TOGGLE_R7 is Rust-owned in bridge_command_options.rs.

// KGW_BRIDGE_COMMAND_COMPOSER_CHECKBOX_ONLY_R9 is Rust-owned in bridge_command_options.rs.


// KGW_BRIDGE_DIFFICULTY_DATALIST_R16C













/* KGW_BRIDGE_INSTANCE_PLACEHOLDER_PORT_RANGES_PATCH_R47
 * Instance placeholder examples now follow the agreed network port ranges.
 * This changes display/help text only. It does not overwrite saved user ports.
 */



/* KGW_BRIDGE_INSTANCE_PHASE1_UPSTREAM_SERIALIZER_R1C
 * Upstream-compatible bridge instance serializer.
 * RKStratum expects one --instance value with comma-separated internal keys:
 * port/prom/diff/log/var_diff/shares_per_min/var_diff_stats/pow2_clamp.
 */
/* KGW_BRIDGE_INSTANCE_EFFECTIVE_SETTINGS_RUST_OWNER_V1 */
/* Port conflict registry/validation ownership lives in Rust bridge_port_core.rs. */
/* KGW_BRIDGE_INSTANCES_PLUS_AUTOPORT_DETAILS_R8B
 * Existing Bridge Instances owner refinement:
 * - + action is routed through installActions.
 * - New instance gets nearest unused stratum port and prom port.
 * - Each instance panel shows a read-only upstream --instance preview row.
 */



/* KGW_BRIDGE_INSTANCES_NO_ADVANCED_NUMERIC_PORTS_R9
 * Existing Bridge Instances owner refinement:
 * - No advanced free-text field.
 * - Initial/default instances receive numeric port/prom values.
 * - User can edit numeric port/prom.
 * - Validator still blocks conflicts.
 */






// KGW_BRIDGE_INSTANCE_PORT_CONFLICT_REPAIR_R110G


// KGW_BRIDGE_AUTOFIX_BUTTON_INITIAL_LABEL_R111G
/* KGW_BRIDGE_SCOPED_START_CONFLICT_R110H is Rust-owned in bridge_port_validation.rs. */

/* KGW_BRIDGE_PORT_CONFLICT_START_GATE_PATCH_R33
 * Existing Bridge port conflict owner enhancement:
 * - Uses existing bridgeValidatePortConflictsR5 registry.
 * - Blocks Start before runtime if any configured port conflict touches the active network.
 * - Updates Start button disabled/title state live.
 * - Save remains allowed with warning.
 * - Covers mainnet, testnet10, testnet13 through BRIDGE_NETWORKS.
 */
/* R33 per-network validate/apply state is Rust-owned; JS retains only live callers below. */









/* KGW_BRIDGE_INSTANCE_FIELD_PLACEHOLDERS_RANGE_PATCH_R49
 * Field-level instance port placeholders now follow the active network profile.
 * Display/help text only. Does not overwrite saved user ports.
 */
// KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D
function renderInprocessNodeSettings(net) {
  return wasmBridgeRenderInprocessNodeSettingsUi(net || {});
}
// KGW_BRIDGE_SETTINGS_SECTIONS_RUST_OWNER_V1
function renderSections(net) {
  return wasmBridgeRenderSectionsUi(net || {}, {
    renderInprocessNodeSettings: (profile) => renderInprocessNodeSettings(profile),
    bridgeInstances,
    activeInstance
  });
}

/* KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
 * Default Bridge inner tab is Live Bridge Monitor.
 * Last selected inner tab is saved per network.
 */
/* R101U inner-tab persistence is Rust-owned in bridge_frontend_helpers.rs. */

// KGW_BRIDGE_NETWORK_PANEL_RUST_OWNER_V1
function renderNetworkPanel(net, index) {
  return wasmBridgeRenderNetworkPanelUi(net || {}, Number(index) || 0, {
    renderSections: (profile) => renderSections(profile)
  });
}


// KGW_BRIDGE_INSTANCE_REFRESH_RUST_OWNER_V1
function bridgeRefreshInstances(net) {
  return wasmBridgeRefreshInstancesUi(
    String(net || ""),
    bridgeInstances,
    activeInstance,
    {
      decorateSettingsFields: (container) => decorateSettingsFields(container),
      installInstanceContainerOwner: (container, targetNet) =>
        bridgeInstallInstanceContainerOwnerR11(container, String(targetNet || "")),
      updateCommand: (targetNet) => updateCommand(String(targetNet || ""))
    }
  );
}

/* KGW_BRIDGE_INSTANCES_PATCHMARKER_RUNTIME_FIX_R13B: fixes undefined runtime owner marker assignment. */
/* KGW_BRIDGE_INSTANCES_REBUILD_CLICK_OWNER_R11
 * One rebuilt Bridge Instances click owner.
 * It lives only on the rendered Instances container.
 * It handles + / select / delete via closest('[data-bridge-action]').
 * No document/window/global listener.
 */
// KGW_BRIDGE_INSTANCE_CLICK_OWNER_RUST_OWNER_V1
function bridgeInstallInstanceContainerOwnerR11(container, net) {
  return wasmBridgeInstallInstanceContainerOwnerR11(
    container,
    String(net || ""),
    activeInstance,
    {
      addInstance: (targetNet) => addInstance(String(targetNet || "")),
      refreshInstances: (targetNet) => bridgeRefreshInstances(String(targetNet || "")),
      updateCommand: (targetNet) => updateCommand(String(targetNet || "")),
      removeInstance: (targetNet, instanceId) =>
        removeInstance(String(targetNet || ""), String(instanceId || ""))
    }
  );
}

// KGW_BRIDGE_VISIBLE_INSTANCE_OWNERS_RUST_OWNER_V1
function bridgeInstallAllVisibleInstanceContainerOwnersR11(root) {
  void root;
  return wasmBridgeInstallAllVisibleInstanceContainerOwnersR11({
    installInstanceContainerOwner: (container, targetNet) =>
      bridgeInstallInstanceContainerOwnerR11(container, String(targetNet || ""))
  });
}

/* KGW_BRIDGE_INSTANCES_TEMP_TRACE_REMOVED
 * Temporary scoped runtime trace for Bridge Instances across mainnet/testnet10/testnet13.
 * No global click listener. No forbidden legacy phase names.
 */



/* KGW_BRIDGE_INSTANCES_ADD_CLICK_BIND_R10
 * Scoped Bridge Instances button binder.
 * This is not a global listener. It binds only the rendered instance container
 * and replaces onclick handlers idempotently after each render.
 */



// KGW_BRIDGE_INSTANCE_MUTATION_RUST_OWNER_V1
function addInstance(net) {
  return wasmBridgeAddInstanceUi(
    String(net || ""),
    bridgeInstances,
    activeInstance,
    {
      refreshInstances: (targetNet) => bridgeRefreshInstances(String(targetNet || "")),
      updateCommand: (targetNet) => updateCommand(String(targetNet || ""))
    }
  );
}


function removeInstance(net, instanceId) {
  return wasmBridgeRemoveInstanceUi(
    String(net || ""),
    instanceId,
    bridgeInstances,
    activeInstance,
    {
      refreshInstances: (targetNet) => bridgeRefreshInstances(String(targetNet || "")),
      updateCommand: (targetNet) => updateCommand(String(targetNet || ""))
    }
  );
}


// KGW_BRIDGE_RENDER_ALL_NETWORKS_RUST_OWNER_V1
function renderAllNetworks(root) {
  return wasmBridgeRenderAllNetworksUi(root, {
    renderNetworkPanel: (profile, index) => renderNetworkPanel(profile, index),
    installSettingsLayout: (targetRoot) => installSettingsLayout(targetRoot)
  });
}

/* KGW_BRIDGE_NETWORK_PORT_PROFILES_SOFT_POLICY_PATCH_R35B
 * Network port profiles are soft policy:
 * - Used for defaults/suggestions/auto-assignment only.
 * - Manual valid unused ports are accepted, even inside another network's recommended range.
 * - Real conflicts still block Start through R33.
 * - Out-of-profile ports are warning-only.
 */

/* KGW_BRIDGE_INSTANCE_EXTERNAL_PORT_RANGE_OWNER_R91
 * Existing Bridge port-profile owner refinement.
 * Instance stratum/prometheus port ranges are now derived from the current
 * bridge-level network settings outside the instance editor:
 * - stratum instances follow --stratum-port + 1 onward.
 * - prom instances follow --prom-port + 1 onward.
 * - each network remains isolated: mainnet, testnet10, testnet13.
 * - valid clearly manual out-of-range instance ports are preserved.
 */









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
/* R37 trace ownership lives in Rust bridge_port_ui.rs. */
/* KGW_BRIDGE_AUTOFIX_GLOBAL_USED_PORTS_PATCH_R45
 * Strengthens existing R37 Auto Fix:
 * - De-duplicates repeated conflict owners.
 * - Uses global used ports across all bridge networks when selecting replacements.
 * - If active network instance conflicts with another network instance, changes active network instance first.
 * - Applies multiple passes so a replacement cannot leave a new conflict behind.
 * Port policy remains soft: manual valid unused ports are accepted anywhere.
 */


/* Auto Fix button enumeration lives in Rust bridge_port_ui.rs. */
/* KGW_BRIDGE_AUTOFIX_I18N_PATCH_R54D3
 * Local i18n wrapper for existing Bridge Auto Fix labels/log prefix.
 */
/* KGW_BRIDGE_AUTOFIX_I18N_PATCH_R54D3
 * Existing Bridge Auto Fix i18n owner.
 * KGW_BRIDGE_AUTOFIX_I18N_OWNER_SAFE_FALLBACK_R112D:
 * Never return a raw bridge.autofixPorts.* key to the UI.
 */
/* R54D3 Auto Fix text ownership lives in Rust bridge_port_ui.rs. */

// KGW_BRIDGE_MODE_CONTROLS_RUST_OWNER_V1
function bridgeSyncModeControls(net) {
  return wasmBridgeSyncModeControlsUi(String(net || ""), bridgeInstances);
}

// KGW_BRIDGE_COMMAND_ORCHESTRATION_RUST_OWNER_V1
function bridgeSyncAllModeControls() {
  return wasmBridgeSyncAllModeControlsUi(bridgeInstances);
}

// Bridge R27 log-auto-scroll persistence, scroll behavior, and DOM installer are Rust-owned in bridge_frontend_helpers.rs.

function buildCommandLines(net) {
  return Array.from(
    wasmBridgeBuildCommandLinesUi(String(net || ""), bridgeInstances, activeInstance)
  );
}
// KGW_BRIDGE_REQUIRE_VALID_SETTINGS_RUST_OWNER_V1
function kgwBridgeEffectiveInprocessNodeSettings(net) {
  return wasmBridgeEffectiveInprocessNodeSettingsChecked(String(net || ""), bridgeInstances);
}





// KGW_BRIDGE_COMMAND_PREVIEW_RUST_OWNER_V1
function updateCommand(net) {
  return wasmBridgeUpdateCommandUi(
    String(net || ""),
    bridgeInstances,
    activeInstance,
    kgwBridgeR51ReadStructuredInstancesR253,
    buildCommandLines
  );
}

function updateAllCommands() {
  return wasmBridgeUpdateAllCommandsUi(
    bridgeInstances,
    activeInstance,
    kgwBridgeR51ReadStructuredInstancesR253,
    buildCommandLines
  );
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
}
/* KGW_BRIDGE_LAST_NETWORK_RESTORE_R101W2 */
/* R101W2 last-network persistence is Rust-owned in bridge_frontend_helpers.rs. */

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
    const normalized = wasmBridgeNormalizeNetwork(net);
    if (!normalized) return;
    if (persist) wasmBridgeSaveLastNetwork(normalized);

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
    window.setTimeout(() => {
      void wasmBridgeR51RefreshOne(
        String(normalized || ""),
        "network-tab-" + reason,
        kgwBridgeR51LiveRefreshCallbacksR257()
      );
    }, 50);
    window.setTimeout(() => {
      void wasmBridgeR51RefreshOne(
        String(normalized || ""),
        "network-tab-" + reason + "+700ms",
        kgwBridgeR51LiveRefreshCallbacksR257()
      );
    }, 700);
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

  const saved = wasmBridgeReadLastNetwork();
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
      const selected = wasmBridgeSaveInnerTab(String(net || ""), innerTab.dataset.bridgeInnerTab);
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
        item.setAttribute("aria-selected", String(item === sectionTab));
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
      wasmBridgeRenderRawLogBuffer(String(net || ""), "bridge", String(selected));

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
// Tauri invoke resolution and timeout policy are Rust-owned in bridge_start_trace.rs.
const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();

/* KGW_BRIDGE_START_TRACE_V1 is Rust-owned in bridge_start_trace.rs. */

/* KGW_BRIDGE_START_OPTIONS is Rust-owned in bridge_instance_settings.rs. */

// KGW_BRIDGE_APPLY_PAYLOAD_RUST_OWNER_V1
function buildApplyPayload(net, command) {
  return wasmBridgeBuildApplyPayloadUi(
    String(net || ""),
    String(command || ""),
    bridgeInstances,
    activeInstance,
    kgwBridgeR51ReadStructuredInstancesR253,
    buildCommandLines
  );
}
async function invokeBridgeIntegratedRuntime(command, net) {
  const payload = buildApplyPayload(net, command);
  if (command === "kgw_kgw_apply_node_settings_v1") {
    const errors = kgwBridgeValidateForm(net, true);
    if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
    await wasmBridgePreparePreview(String(net || ""), payload);
  }
  return await wasmBridgeInvokeRuntimeCommand(command, payload);
}


// KGW_BRIDGE_INPROCESS_SAME_DB_OWNER_V7 is Rust-owned in bridge_start_trace.rs.

// KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E is Rust-owned in node_tab.rs.

// KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_MAINNET_IMMEDIATE_R65F DOM read is Rust-owned in bridge_frontend_helpers.rs.

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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
  }

  kgwBridgeRuntimeOwnerTraceR64D("r64d-runtime-owner-enter", {
    action: String(action || ""),
    net: String(net || "")
  });

  const command = wasmBridgeRuntimeCommandForAction(String(action || ""));

  if (!command) {
    kgwBridgeRuntimeOwnerTraceR64D("r64d-invalid-action-return", {
      action: String(action || "")
    });
    return false;
  }

  if (action === "start" && !wasmBridgeNetworkEnabled(net)) {
    wasmBridgeSetRuntimeErrorV1(
      net,
      "Bridge start blocked: this network is disabled. Enable it in the network policy bar first."
    );
    wasmBridgeR51SetRuntimeButtons(String(net || ""), false, "", "", "");
    return true;
  }

  if (action === "start") {
    if (Object.keys(kgwBridgeValidateForm(net, true)).length) return true;
    if ((wasmBridgeChecked(net, "internalCpuMiner") || wasmBridgeChecked(net, "inprocessUnsafeRpc") || wasmBridgeChecked(net, "inprocessEnableUnsyncedMining")) &&
        !await confirmUserAction("Start " + net + " with the selected advanced risk settings?\n\nUnsafe RPC exposes RPC beyond loopback. Unsynced mining bypasses synchronization. CPU mining uses additional CPU resources.")) return true;
    kgwBridgeRuntimeOwnerTraceR64D("r64d-preflight-begin", {
      command
    });

    // KGW_BRIDGE_RUNTIME_START_SCOPED_CONFLICT_R111F
    // Use the registered scoped conflict owner instead of the retired global R33 pre-start blocker.
    const scopedConflictResultR111F = wasmBridgeAssertNoPortConflictsR5(
      String(net || ""),
      typeof kgwBridgeR51ReadStructuredInstancesR253 === "function" ? kgwBridgeR51ReadStructuredInstancesR253 : null,
      bridgeInstances,
      activeInstance
    );

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
      wasmBridgeSetRuntimeErrorV1(net, String(scopedConflictResultR111F.message || "Bridge listener port conflict."));
      kgwBridgeRuntimeOwnerTraceR64D("r111f-scoped-conflict-start-blocked-return", {
        reason: "scoped-port-conflict",
        conflictCount: Number(scopedConflictResultR111F.conflictCount || 0),
        message: String(scopedConflictResultR111F.message || "")
      });
      return true;
    }

    const blockedBySameNetworkNode = await wasmBridgeV7BlockInprocessIfNodeOwnerRunning(String(net || ""));

    kgwBridgeRuntimeOwnerTraceR64D("r64d-preflight-result", {
      blockedBySameNetworkNode: Boolean(blockedBySameNetworkNode)
    });

    if (blockedBySameNetworkNode) {
      wasmBridgeSetRuntimeErrorV1(
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
    wasmBridgeSetRuntimeActivityV1(net, "Bridge " + action + " already in progress.");

    kgwBridgeRuntimeOwnerTraceR64D("r64d-inflight-duplicate-return", {
      inFlightKey
    });

    return true;
  }

  KGW_BRIDGE_RUNTIME_IN_FLIGHT.add(inFlightKey);
  wasmBridgeSetRuntimeErrorV1(net, "");
  wasmBridgeR51SetRuntimeButtons(String(net || ""), action === "stop", action === "start" ? "starting" : "stopping", "", "");

  kgwBridgeRuntimeOwnerTraceR64D("r64d-inflight-added", {
    inFlightKey
  });

  try {
    kgwBridgeRuntimeOwnerTraceR64D("r64d-preview-begin", {
      command
    });

    const preview = updateCommand(net) || wasmBridgeById(wasmBridgeElementId(net, "commandPreview"))?.value || "";

    kgwBridgeRuntimeOwnerTraceR64D("r64d-preview-ready", {
      hasPreview: Boolean(preview),
      previewLength: String(preview || "").length
    });

    wasmBridgeSetRuntimeActivityV1(net, "Bridge " + action + " requested.");

    kgwBridgeRuntimeOwnerTraceR64D("r64d-invoke-begin", {
      command,
      hasPreview: Boolean(preview)
    });

    const result = await invokeBridgeIntegratedRuntime(command, net);

    kgwBridgeRuntimeOwnerTraceR64D("r64d-invoke-result", {
      resultType: typeof result,
      resultStringLength: String(result ?? "").length
    });

    const outcome = wasmBridgeRuntimeActionOutcome(String(action || ""), result);
    const raw = String(outcome?.raw || "");
    const fields = outcome?.fields || {};

    kgwBridgeRuntimeOwnerTraceR64D("r64d-response-parsed", {
      rawLength: String(raw || "").length,
      fieldKeys: Object.keys(fields)
    });

    if (action === "start") {
      const confirmedStarted = Boolean(outcome?.confirmedStarted);
      const blocked = Boolean(outcome?.blocked);

      kgwBridgeRuntimeOwnerTraceR64D("r64d-start-confirmation-evaluated", {
        confirmedStarted: Boolean(confirmedStarted),
        blocked: Boolean(blocked)
      });

      if (confirmedStarted && !blocked) {
        wasmBridgeSetRuntimeErrorV1(net, "");
        wasmBridgeR51SetRuntimeButtons(String(net || ""), true, "", "", "");
        const bridgeNodeMode = String(fields.node_mode || fields.nodeMode || "").toLowerCase();
        const bridgeStartWasInprocess = wasmBridgeStartWasInprocessR65F(
          fields,
          String(wasmBridgeCurrentNodeModeFromUiR65F(String(net || "")) || ""),
          String(preview || "")
        );
        if (bridgeStartWasInprocess) {
          wasmBridgeSetOwnedNodeLockR65E(String(net || ""), true, {
            source: "bridge-start-confirmed-r65f",
            action: "start",
            nodeMode: bridgeNodeMode,
            uiNodeMode: wasmBridgeCurrentNodeModeFromUiR65F(String(net || "")),
            previewDeclaredInprocess: wasmBridgePreviewDeclaresInprocessR65F(preview),
            pid: String(fields.pid || "")
          });
        }
        kgwBridgeRuntimeOwnerTraceR64D("r65f-bridge-owned-node-lock-evaluated", {
          patch: "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_MAINNET_IMMEDIATE_R65F",
          bridgeNodeMode,
          uiNodeMode: wasmBridgeCurrentNodeModeFromUiR65F(String(net || "")),
          previewDeclaredInprocess: wasmBridgePreviewDeclaresInprocessR65F(preview),
          bridgeStartWasInprocess
        });
        wasmBridgeSetRuntimeActivityV1(net, "Bridge READY attestation confirmed.", "ready");
        kgwBridgeR51KickRawLogLiveR134E(net, "bridge-start-confirmed");
      } else if (blocked) {
        wasmBridgeR51SetRuntimeButtons(String(net || ""), false, "", "", "");
        wasmBridgeSetRuntimeErrorV1(net, raw);
        wasmBridgeSetRuntimeActivityV1(net, "Bridge start failed.", "failed");
      } else {
        wasmBridgeR51SetRuntimeButtons(String(net || ""), false, "", "", "");
        wasmBridgeSetRuntimeErrorV1(net, "Backend Start did not provide READY attestation: " + raw);
        wasmBridgeSetRuntimeActivityV1(net, "Bridge start was not confirmed by READY attestation.", "warning");
      }
    }

    if (action === "stop") {
      const confirmedStopped = Boolean(outcome?.confirmedStopped);

      kgwBridgeRuntimeOwnerTraceR64D("r64d-stop-confirmation-evaluated", {
        confirmedStopped: Boolean(confirmedStopped)
      });

      if (confirmedStopped) {
        const forced = Boolean(outcome?.forced);
        const stopFailed = Boolean(outcome?.stopFailed);
        KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);
        wasmBridgeR51SetRuntimeButtons(String(net || ""), false, "", "", "");
        wasmBridgeSetRuntimeErrorV1(
          net,
          forced
            ? "Stop required FORCED termination. " + String(fields.reason || raw)
            : stopFailed
              ? "Official graceful shutdown failed, but the worker process exited. " + String(fields.reason || raw)
              : ""
        );
        wasmBridgeSetOwnedNodeLockR65E(String(net || ""), false, {
          source: "bridge-stop-confirmed",
          action: "stop"
        });
        wasmBridgeSetRuntimeActivityV1(
          net,
          forced ? "Bridge FORCED termination confirmed." : stopFailed ? "Bridge worker exited after graceful shutdown failure." : fields.graceful === "true" ? "Bridge graceful official shutdown confirmed." : "Bridge already stopped."
        );
      } else {
        throw new Error("Backend Stop did not confirm terminal process exit: " + raw);
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

    wasmBridgeR51SetRuntimeUnknown(
      String(net || ""),
      String(message || "Runtime status is temporarily unavailable. Reconciling with the backend."),
      ""
    );
    wasmBridgeSetRuntimeActivityV1(net, "Bridge " + action + " failed; reconciling runtime state.");

    return true;
  } finally {
    KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);
    window.setTimeout(() => {
      void wasmBridgeR51RefreshOne(
        String(net || ""),
        "action-settled",
        kgwBridgeR51LiveRefreshCallbacksR257()
      );
    }, 0);

    kgwBridgeRuntimeOwnerTraceR64D("r64d-runtime-owner-finally", {
      inFlightKey
    });
  }
}
/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */


/* KGW_BRIDGE_SETTINGS_STRUCTURED_INSTANCES_PERSISTENCE_PATCH_R26B
 * Bridge settings persistence must not rely only on dynamic DOM field ids.
 * Bridge Instances use runtime-generated ids, so saved field-id maps can become stale after reload/re-render.
 * This patch keeps the existing R51 settings owner and stores/restores structured bridgeInstances state.
 * No new persistence owner. No document listener. No MutationObserver.
 */
// KGW_BRIDGE_INSTANCE_STATE_RUST_OWNER_V1
function kgwBridgeR51ReadStructuredInstancesR253(net) {
  return wasmBridgeR51ReadStructuredInstances(
    String(net || ""),
    bridgeInstances,
    activeInstance
  );
}


/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START */



/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_END */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

function kgwBridgeR51ReadSettingsCallbacksR249() {
  return {
    readStructuredInstances: (net) => kgwBridgeR51ReadStructuredInstancesR253(String(net || "")),
    normalizeNetworkPortValues: (net, values, reason) =>
      wasmBridgeR95BNormalizeNetworkPortValues(String(net || ""), values, String(reason || ""))
  };
}

function kgwBridgeR51ReadSettingsR249(net) {
  return wasmBridgeR51ReadSettings(
    String(net || ""),
    kgwBridgeR51ReadSettingsCallbacksR249()
  );
}

/* KGW_BRIDGE_NETWORK_PORT_RANGE_R51_OWNER_FIX_R95B
 * Existing R51 Bridge settings owner refinement.
 *
 * Runtime screenshots showed stale network-level bridge ports:
 * - testnet10 was replayed as :5556 / :2113
 * - testnet13 was replayed as :5557 / :2114
 *
 * Correct bridge-level network ranges are Rust-owned in bridge_port_core.rs:
 * - mainnet   :5555 / :2112
 * - testnet10 :5655 / :2212
 * - testnet13 :5755 / :2312
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
// Bridge R98 plain-port normalization and same-port comparison are Rust-owned in bridge_frontend_helpers.rs.

// Bridge R95B/R98 settings port normalization is Rust-owned in bridge_frontend_helpers.rs.

function kgwBridgeR51WriteSettingsCallbacksR250() {
  return {
    normalizeNetworkPortValues: (net, values, reason) =>
      wasmBridgeR95BNormalizeNetworkPortValues(String(net || ""), values, String(reason || "")),
    refreshInstances: (net) =>
      bridgeRefreshInstances(String(net || "")),
    refreshInlineCommandToggles: (net) =>
      wasmBridgeRefreshInlineCommandTogglesR7(String(net || "")),
    setInstanceCommandOption: (net, instanceId, name, enabled) =>
      kgwBridgeSetInstanceCommandOptionR13B(
        String(net || ""),
        instanceId,
        String(name || ""),
        Boolean(enabled)
      ),
    updateCommand: (net) => updateCommand(String(net || ""))
  };
}

function kgwBridgeR51WriteSettingsR250(net, values) {
  return wasmBridgeR51WriteSettings(
    String(net || ""),
    values,
    bridgeInstances,
    activeInstance,
    kgwBridgeR51WriteSettingsCallbacksR250()
  );
}

/* KGW_BRIDGE_DIRTY_SETTINGS_BUTTONS_FIX_R2
 * Settings buttons must show whether the current panel has unsaved/default differences.
 * No changes: Save Settings / Restore Defaults / Set as Defaults are disabled.
 */
function kgwBridgeR51PersistenceCallbacksR255() {
  return {
    readSettings: (net) => kgwBridgeR51ReadSettingsR249(String(net || "")),
    writeSettings: (net, values) => kgwBridgeR51WriteSettingsR250(String(net || ""), values),
    normalizeNetworkPortValues: (net, values, reason) =>
      wasmBridgeR95BNormalizeNetworkPortValues(String(net || ""), values, String(reason || "")),
    requireValidSettings: (net) => kgwBridgeRequireValidSettings(String(net || "")),
    updateCommand: (net) => updateCommand(String(net || ""))
  };
}

/* R9B programmatic restore relies on Event.isTrusted in the current input/change owners; no JavaScript callback wrapper is required. */

function kgwBridgeR51LiveRefreshCallbacksR257() {
  return {
    invokeRuntime: (command, net) =>
      invokeBridgeIntegratedRuntime(String(command || ""), String(net || "")),
    transitionActive: (net) =>
      KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(String(net || "") + ":start") ||
      KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(String(net || "") + ":stop"),
    activeRawLogInstanceId: (net) =>
      wasmBridgeActiveRawLogInstanceId(
        bridgeInstances,
        activeInstance,
        String(net || "")
      )
  };
}

// KGW_BRIDGE_RAW_LOG_LIVE_EXACT_R134E
// Raw bridge log live helper only: no parsing, no ASIC table, no bridge behavior duplication.
function kgwBridgeR51KickRawLogLiveR134E(net, reason = "bridge-start") {
  try {
    wasmBridgeR51StartLiveRefresh(kgwBridgeR51LiveRefreshCallbacksR257());
    window.setTimeout(function () {
      void wasmBridgeR51RefreshOne(String(net || ""), reason + "-0", kgwBridgeR51LiveRefreshCallbacksR257());
    }, 0);
    window.setTimeout(function () {
      void wasmBridgeR51RefreshOne(String(net || ""), reason + "-350", kgwBridgeR51LiveRefreshCallbacksR257());
    }, 350);
    window.setTimeout(function () {
      void wasmBridgeR51RefreshOne(String(net || ""), reason + "-1000", kgwBridgeR51LiveRefreshCallbacksR257());
    }, 1000);
    window.setTimeout(function () {
      void wasmBridgeR51RefreshOne(String(net || ""), reason + "-2500", kgwBridgeR51LiveRefreshCallbacksR257());
    }, 2500);
  } catch (error) {
    console.warn("[KGW_BRIDGE_RAW_LOG_LIVE_EXACT_R134E_FAILED]", error);
  }
}



/* KGW_BRIDGE_ACTION_AND_LOG_FEEDBACK_OWNER_V1 */


/* KGW_BRIDGE_LOG_FEEDBACK_I18N_OWNER_V1 */


/* KGW_BRIDGE_SETTINGS_BUTTON_FEEDBACK_FIX_R1
 * Settings action buttons must confirm successful user actions immediately.
 * The existing Bridge action owner calls this helper after save/restore/set-default succeeds.
 */
/* KGW_BRIDGE_SETTINGS_BUTTON_FEEDBACK_HOLD_FIX_R2
 * Keep settings button success labels visible long enough for the user.
 * The helper repeats the label during the hold window to survive fast UI re-renders.
 */



/* KGW_LOG_ACTIONS_SCOPED_OWNER_V29_START */
async function kgwBridgeHandleLogActionV29(action, net, button) {
  return await wasmBridgeHandleLogAction(String(action || ""), String(net || ""), button, {
    bridgeInstances,
    activeInstance
  });
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
        wasmBridgeSchedulePortConflictValidationR33(bridgeInstances, activeInstance, String(net || ""), "input");
        wasmBridgeSchedulePortAutofixRefreshUiR37(String(net || ""), "input", bridgeInstances, activeInstance);
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
        wasmBridgeSchedulePortConflictValidationR33(bridgeInstances, activeInstance, String(net || ""), "change");
        wasmBridgeSchedulePortAutofixRefreshUiR37(String(net || ""), "change", bridgeInstances, activeInstance);
      }
    });

    window.setTimeout(() => wasmBridgeValidateAllPortConflictStatesR33(bridgeInstances, activeInstance, "install"), 100);
  }

  if (!root.dataset.kgwBridgePortAutofixOwnerR37) {
    root.dataset.kgwBridgePortAutofixOwnerR37 = "1";

    wasmBridgeInstallPortAutofixButtonUiR37(root, bridgeInstances, activeInstance);

    root.addEventListener("click", (event) => {
      const button = event.target && event.target.closest('[data-bridge-action="auto-fix-ports-r37"]');
      if (!button || !root.contains(button)) return;

      event.preventDefault();
      event.stopPropagation();

      const net = button.dataset.net || "";
      const result = wasmBridgeApplyPortAutofixUiR37(String(net || ""), bridgeInstances, activeInstance, (targetNet) => bridgeRefreshInstances(targetNet), (targetNet) => updateCommand(targetNet), (targetNet, message) => wasmBridgeSetRuntimeActivityV1(targetNet, message));

      button.textContent = result.changed ? "Fixed " + String(result.changed) + " Port(s)" : "No Fix Needed";
      window.setTimeout(() => wasmBridgeRefreshPortAutofixButtonsUiR37("button-feedback", bridgeInstances, activeInstance), 1200);
    });

    window.setTimeout(() => wasmBridgeRefreshPortAutofixButtonsUiR37("install", bridgeInstances, activeInstance), 120);
  }

  if (!root.dataset.kgwBridgeInstancesCommandCheckboxOwnerR13B) {
    root.dataset.kgwBridgeInstancesCommandCheckboxOwnerR13B = "1"; // KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_ACTION_R13B

    root.addEventListener("change", (event) => {
      const include = event.target.closest("[data-bridge-instance-command-option-toggle-r13b]");
      if (include && root.contains(include)) {
        kgwBridgeSetInstanceCommandOptionR13B(
          include.dataset.net,
          include.dataset.instanceId,
          include.dataset.bridgeInstanceCommandOptionToggleR13b,
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

      wasmBridgeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-bridge-command-checkbox-pointerdown", {
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

      wasmBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r31-bridge-command-checkbox-change-begin", {
        patch: "R31",
        owner: "bridge-command-composer-r7",
        option: String(option || ""),
        checked: enabled,
        trusted: Boolean(event && event.isTrusted)
      });

      try {
        wasmBridgeCommandSetOptionR7(String(net || ""), String(option || ""), enabled);
        updateCommand(net);
        wasmBridgeRefreshInlineCommandTogglesR7(String(net || ""));

        queueMicrotask(() => {
          wasmBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r31-bridge-command-checkbox-change-after-microtask", {
            patch: "R31",
            owner: "bridge-command-composer-r7",
            option: String(option || ""),
            checkedAfter: Boolean(toggle.checked)
          });
        });
      } catch (error) {
        wasmBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r31-bridge-command-checkbox-change-failed", {
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

        wasmBridgeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-bridge-command-checkbox-click", {
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
            wasmBridgeSmallOwnerTraceR44D(toggle.dataset.net, "command-checkbox", "r31-bridge-command-checkbox-click-after-microtask", {
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
        wasmBridgeCommandToggleOptionR7(
          String(toggle.dataset.net || ""),
          String(toggle.dataset.bridgeCommandOptionToggleR7 || "")
        );
        updateCommand(toggle.dataset.net);
      }
    });

    root.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      const toggle = event.target.closest("[data-bridge-command-option-toggle-r7]");
      if (toggle && root.contains(toggle)) {
        event.preventDefault();
        event.stopPropagation();
        wasmBridgeCommandToggleOptionR7(
          String(toggle.dataset.net || ""),
          String(toggle.dataset.bridgeCommandOptionToggleR7 || "")
        );
        updateCommand(toggle.dataset.net);
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
    if (raw.includes("testnet13") || raw.includes("tn13")) return "testnet13";
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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
    wasmBridgeMarkRestartRequiredV1(net);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", async (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    wasmBridgeMarkRestartRequiredV1(net);

    if (target.matches("[data-bridge-network-enabled]")) {
      const profile = wasmBridgeNetworkProfile(net);
      const wasEnabled = wasmBridgeNetworkEnabled(net);
      let enabled = Boolean(target.checked);

      if (enabled && profile?.experimental) {
        target.checked = false;
        target.disabled = true;
        try {
          enabled = (await confirmUserAction("Testnet 13 is experimental and uses a separate non-production runtime. Enable it only for isolated testing. Continue?")) === true;
        } catch (error) {
          enabled = false;
          wasmBridgeSetRuntimeErrorV1(net, wasmBridgeNormalizeRuntimeError(error));
        } finally { target.disabled = false; target.checked = enabled; }
      }

      wasmBridgeSetNetworkEnabled(net, enabled);
      wasmBridgeR51SetRuntimeButtons(String(net || ""), false, "", "", "");

      if (!enabled && wasEnabled) {
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
      wasmBridgeRenderRawLogBuffer(String(net || ""), "bridge", String(activeInstance[net] || ""));
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
      try {
        wasmBridgeR51SaveSettings(String(net || ""), kgwBridgeR51PersistenceCallbacksR255());
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        wasmBridgeSetRuntimeErrorV1(net, wasmBridgeNormalizeRuntimeError(error));
      }
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      try {
        wasmBridgeR51SetAsDefaults(String(net || ""), kgwBridgeR51PersistenceCallbacksR255());
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        wasmBridgeSetRuntimeErrorV1(net, wasmBridgeNormalizeRuntimeError(error));
      }
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      try {
        wasmBridgeR51RestoreDefaults(String(net || ""), kgwBridgeR51PersistenceCallbacksR255());
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        wasmBridgeSetRuntimeErrorV1(net, wasmBridgeNormalizeRuntimeError(error));
      }
      scopedUpdate(net, "restore-defaults");
      return;
    }

    if (action === "copy-log" || action === "clear-log") {
      kgwBridgeHandleLogActionV29(action, net, button).catch(function () {});
      return;
    }

    if (action === "monitor-next") {
      const panel = wasmBridgeR51Panel(net);
      if (button.dataset.nextAction === "start") panel?.querySelector('[data-bridge-action="start"]')?.click();
      else panel?.querySelector('[data-bridge-inner-tab="settings"]')?.click();
      return;
    }
    if (action === "copy-command" || action === "copy-path") {
      void (async () => {
        let text = wasmBridgeValue(net, "appdir");
        if (action === "copy-command") {
          const errors = kgwBridgeValidateForm(net);
          if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
          const requestSequence = wasmBridgePreviewSequence(String(net || ""));
          const result = await wasmBridgePreparePreview(String(net || ""), buildApplyPayload(net, "kgw_kgw_apply_node_settings_v1"));
          if (wasmBridgePreviewSequence(String(net || "")) !== requestSequence) throw new Error("Settings changed while copying. Try again.");
          text = JSON.stringify(result, null, 2);
        }
        if (!text) throw new Error("There is no validated value to copy.");
        await wasmBridgeDispatchClipboardWrite(String(net || ""), String(text ?? ""), {characterCount: [...text].length, lineCount: text.split(/\r?\n/).length});
        wasmBridgePreviewMessage(String(net || ""), action === "copy-path" ? "Data directory copied." : "Effective settings copied.", false);
      })().catch(error => wasmBridgePreviewMessage(String(net || ""), "Copy failed: " + wasmBridgeNormalizeRuntimeError(error), true));
      return;
    }

    if (action === "start" || action === "stop") {
      if (typeof runBridgeIntegratedAction === "function") {
        runBridgeIntegratedAction(action, net).catch(function (error) {
          wasmBridgeR51SetRuntimeButtons(String(net || ""), false, "", "", "");
          wasmBridgeSetRuntimeErrorV1(net, error && error.message ? error.message : String(error));
          wasmBridgeSetRuntimeActivityV1(net, "Bridge " + action + " failed.");
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
  wasmBridgeR51CaptureFactoryDefaults(kgwBridgeR51PersistenceCallbacksR255());
  wasmBridgeR51LoadSavedSettings(kgwBridgeR51PersistenceCallbacksR255());
  BRIDGE_NETWORKS.forEach((net) => wasmBridgeR51SetRuntimeButtons(String(net.key || ""), false, "", "", ""));
  bridgeSyncAllModeControls();
  installNetworkTabs(bridgeRoot);
  installDelegatedTabs(bridgeRoot);
  installActions(bridgeRoot);
  bridgeSyncAllModeControls();
  updateAllCommands();
  BRIDGE_NETWORKS.forEach((net) => wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(String(net.key || ""), updateCommand)); /* KGW_BRIDGE_DYNAMIC_PATHS_INIT_R3 */
  window.setTimeout(updateAllCommands, 0);
  window.setTimeout(updateAllCommands, 150);
  bridgeSyncAllModeControls();
  updateAllCommands();
  wasmBridgeR51StartLiveRefresh(kgwBridgeR51LiveRefreshCallbacksR257());


  setTimeout(wasmBridgeInstallLogAutoScrollControls, 0);
}


/* KGW_BRIDGE_LOG_SCOPED_CONTROLS_V29_START */
(function installKgwLogScopedToolbarControlsV29() {
  "use strict";

  const KIND = "bridge";
  const ROOT_SELECTOR = "#kaspa-bridge";
  const TOOLBAR_SELECTOR = ".bridge-v7-log-toolbar";
  const ACTION_ATTR = "data-bridge-action";
  const PREFIX = "bridge";
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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
      wasmBridgeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-bridge-log-font-decrease-click", {
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
      wasmBridgeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-bridge-log-font-increase-click", {
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
      wasmBridgeSmallOwnerTraceR44D(net, "log-font-size", "r51b3-bridge-log-font-reset-click", {
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
try { wasmBridgeAutofixButtonInitialLabelUiR111G(document); } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

// KGW_BRIDGE_FULL_FORM_VALIDATION_RUST_OWNER_V1
function kgwBridgeValidateForm(net, focus = false) {
  return wasmBridgeValidateFormUi(String(net || ""), bridgeInstances, Boolean(focus));
}
function kgwBridgeRequireValidSettings(net) {
  return wasmBridgeRequireValidSettingsUi(String(net || ""), bridgeInstances, activeInstance, kgwBridgeR51ReadStructuredInstancesR253);
}
export { kgwBridgeEffectiveInprocessNodeSettings, wasmBridgeEffectiveSettingsV1 as kgwBridgeEffectiveSettingsV1, kgwBridgeValidateForm };
