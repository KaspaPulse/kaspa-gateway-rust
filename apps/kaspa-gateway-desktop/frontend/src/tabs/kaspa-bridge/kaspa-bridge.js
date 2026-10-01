import { confirmUserAction } from "../../settings-contract.js";
import { renderSettingsTabs, installSettingsLayout, decorateSettingsFields } from "../../settings-layout.js";
import initBridgeRust, {
  bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5 as wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5,
  bridgeById as wasmBridgeById,
  bridgeChecked as wasmBridgeChecked,
  bridgeCommandInlineToggleR7 as wasmBridgeCommandInlineToggleR7,
  bridgeRefreshInlineCommandTogglesR7 as wasmBridgeRefreshInlineCommandTogglesR7,
  bridgeCommandSetOptionR7 as wasmBridgeCommandSetOptionR7,
  bridgeCommandToggleOptionR7 as wasmBridgeCommandToggleOptionR7,
  bridgeBuildCommandLinesUi as wasmBridgeBuildCommandLinesUi,
  bridgeBuildApplyPayloadUi as wasmBridgeBuildApplyPayloadUi,
  bridgeDifficultyDatalistR16C as wasmBridgeDifficultyDatalistR16C,
  bridgeDifficultyInputAttrsR16C as wasmBridgeDifficultyInputAttrsR16C,
  bridgeCardInput as wasmBridgeCardInput,
  bridgeRenderRuntime as wasmBridgeRenderRuntime,
  bridgeRenderDifficulty as wasmBridgeRenderDifficulty,
  bridgeRenderLogging as wasmBridgeRenderLogging,
  bridgeRenderPorts as wasmBridgeRenderPorts,
  bridgeRenderCpuMiner as wasmBridgeRenderCpuMiner,
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
  bridgeNormalizeInstanceRecord as wasmBridgeNormalizeInstanceRecord,
  bridgeInstanceCommandCheckboxFromInstancesR13B as wasmBridgeInstanceCommandCheckboxFromInstancesR13B,
  bridgeInstanceCommandSetOptionR13B as wasmBridgeInstanceCommandSetOptionR13B,
  bridgeInstancePreviewTextR8B as wasmBridgeInstancePreviewTextR8B,
  bridgeSyncInstancePreviewRowsR8B as wasmBridgeSyncInstancePreviewRowsR8B,
  bridgeReadInstanceField as wasmBridgeReadInstanceField,
  bridgeInstancePortPlaceholderR49 as wasmBridgeInstancePortPlaceholderR49,
  bridgeInstancePromPlaceholderR49 as wasmBridgeInstancePromPlaceholderR49,

  bridgeDispatchClipboardWrite as wasmBridgeDispatchClipboardWrite,
  bridgeElementId as wasmBridgeElementId,
  bridgeEscapeHtml as wasmBridgeEscapeHtml,
  bridgeHandleLogAction as wasmBridgeHandleLogAction,
  bridgeI18nTextR41 as wasmBridgeI18nTextR41,
  bridgeInstallLogAutoScrollControls as wasmBridgeInstallLogAutoScrollControls,
  bridgeNetworkEnabled as wasmBridgeNetworkEnabled,
  bridgeNetworkPolicyMessage as wasmBridgeNetworkPolicyMessage,
  bridgeNormalizeNetwork as wasmBridgeNormalizeNetwork,
  bridgeReadLastNetwork as wasmBridgeReadLastNetwork,
  bridgeSaveLastNetwork as wasmBridgeSaveLastNetwork,
  bridgeNetworkProfile as wasmBridgeNetworkProfile,
  bridgeNetworkProfiles as wasmBridgeNetworkProfiles,
  bridgeInstanceNetworkKeyR15 as wasmBridgeInstanceNetworkKeyR15,
  bridgeResolveInnerTab as wasmBridgeResolveInnerTab,
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
  bridgeAssignMissingInstancePortsR9 as wasmBridgeAssignMissingInstancePortsR9,
  bridgeCreateInstanceRecordR9 as wasmBridgeCreateInstanceRecordR9,
  bridgeEnsureInstanceState as wasmBridgeEnsureInstanceState,
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
  wasmBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r29b-bridge-instance-command-checkbox-begin", {
    patch: "R29B",
    owner: "bridge-instance-command-composer-r13b",
    instanceId: String(instanceId || ""),
    option: String(name || ""),
    enabled: Boolean(enabled)
  });

  const key = wasmBridgeInstanceCommandSetOptionR13B(
    String(net || ""),
    instanceId,
    String(name || ""),
    Boolean(enabled)
  );
  wasmBridgeR51Panel(net)?.querySelectorAll("[data-bridge-instance-command-option-toggle-r13b]").forEach(toggle => {
    if (String(toggle.dataset.instanceId) !== String(instanceId) || toggle.dataset.bridgeInstanceCommandOptionToggleR13b !== name) return;
    toggle.checked = Boolean(enabled);
    toggle.title = enabled ? "Included in command" : "Excluded from command";
    toggle.setAttribute("aria-label", toggle.title);
  });
  updateCommand(net);
  wasmBridgeSyncInstancePreviewRowsR8B(String(net || ""), bridgeInstances, activeInstance);

  wasmBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r29b-bridge-instance-command-checkbox-complete", {
    patch: "R29B",
    owner: "bridge-instance-command-composer-r13b",
    key: String(key || ""),
    instanceId: String(instanceId || ""),
    option: String(name || ""),
    enabled: Boolean(enabled)
  });
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
function renderInstances(net) {
  net = wasmBridgeInstanceNetworkKeyR15(net, net);
  if (net !== "mainnet") return ""; // Testnet mining is embedded CPU-only, including restored settings.
  wasmBridgeEnsureInstanceState(bridgeInstances, activeInstance, String(net || ""));

  return `
    <div class="bridge-v7-instance-tabs bridge-v7-instance-tabs-r7b">
      ${bridgeInstances[net].map((instance, index) => `
        <span class="kgw-instance-tab">
          <button type="button"
            class="bridge-v7-instance-pill-r7b bridge-v7-instance-pill-r11 ${String(activeInstance[net]) === String(instance.id) || (!activeInstance[net] && index === 0) ? "active" : ""}"
            data-bridge-action="select-instance" data-network="${net}" data-instance-id="${instance.id}">
            Instance ${index + 1}
          </button>
          <button type="button" class="bridge-v7-instance-trash-r11"
            data-bridge-action="remove-instance" data-network="${net}" data-instance-id="${instance.id}"
            title="Delete Instance ${index + 1}" aria-label="Delete Instance ${index + 1}"
            ${bridgeInstances[net].length <= 1 ? "disabled" : ""}>Delete</button>
        </span>`).join("")}
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
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instance")}
              <span class="kgw-command-option-title-text-r8e">Effective instance</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input
              readonly
              data-bridge-instance-preview="true"
              data-network="${net}"
              data-instance-id="${instance.id}"
              value="${wasmBridgeEscapeHtml(wasmBridgeInstancePreviewTextR8B(String(net || ""), instance || {}))}"
              title="${wasmBridgeEscapeHtml(wasmBridgeInstancePreviewTextR8B(String(net || ""), instance || {}))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instancePort")}
              <span class="kgw-command-option-title-text-r8e">port</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${wasmBridgeElementId(net, `instancePort-${instance.id}`)}" data-bridge-instance-field="instancePort" value="${wasmBridgeEscapeHtml(instance.instancePort || "")}" placeholder="${wasmBridgeEscapeHtml(wasmBridgeInstancePortPlaceholderR49(String(net || "")))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceDiff")}
              <span class="kgw-command-option-title-text-r8e">diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${wasmBridgeElementId(net, `instanceDiff-${instance.id}`)}" data-bridge-instance-field="instanceDiff" value="${wasmBridgeEscapeHtml(instance.instanceDiff || "2048")}" placeholder="2048" ${wasmBridgeDifficultyInputAttrsR16C("instanceDiff")} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceProm")}
              <span class="kgw-command-option-title-text-r8e">prom</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${wasmBridgeElementId(net, `instanceProm-${instance.id}`)}" data-bridge-instance-field="instanceProm" value="${wasmBridgeEscapeHtml(instance.instanceProm || "")}" placeholder="${wasmBridgeEscapeHtml(wasmBridgeInstancePromPlaceholderR49(String(net || "")))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceLogToFile")}
              <span class="kgw-command-option-title-text-r8e">log</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${wasmBridgeElementId(net, `instanceLogToFile-${instance.id}`)}" data-bridge-instance-field="instanceLogToFile">
              <option value="not set" ${instance.instanceLogToFile === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instanceLogToFile === "false" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceLogToFile === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceBlockWaitTime")}
              <span class="kgw-command-option-title-text-r8e">wait</span>
            </span>
            <input id="${wasmBridgeElementId(net, `instanceBlockWaitTime-${instance.id}`)}" data-bridge-instance-field="instanceBlockWaitTime" value="${wasmBridgeEscapeHtml(instance.instanceBlockWaitTime || "")}" placeholder="Enable to override global milliseconds" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceExtranonceSize")}
              <span class="kgw-command-option-title-text-r8e">extranonce</span>
            </span>
            <input id="${wasmBridgeElementId(net, `instanceExtranonceSize-${instance.id}`)}" data-bridge-instance-field="instanceExtranonceSize" value="${wasmBridgeEscapeHtml(instance.instanceExtranonceSize || "")}" placeholder="optional" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceVarDiff")}
              <span class="kgw-command-option-title-text-r8e">var_diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${wasmBridgeElementId(net, `instanceVarDiff-${instance.id}`)}" data-bridge-instance-field="instanceVarDiff">
              <option value="not set" ${instance.instanceVarDiff === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instanceVarDiff === "false" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiff === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceVarDiffStats")}
              <span class="kgw-command-option-title-text-r8e">var_stats</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${wasmBridgeElementId(net, `instanceVarDiffStats-${instance.id}`)}" data-bridge-instance-field="instanceVarDiffStats">
              <option value="not set" ${instance.instanceVarDiffStats === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instanceVarDiffStats === "false" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiffStats === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instanceSharesPerMin")}
              <span class="kgw-command-option-title-text-r8e">shares/min</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${wasmBridgeElementId(net, `instanceSharesPerMin-${instance.id}`)}" data-bridge-instance-field="instanceSharesPerMin" value="${wasmBridgeEscapeHtml(instance.instanceSharesPerMin || "")}" placeholder="optional" ${wasmBridgeDifficultyInputAttrsR16C("instanceSharesPerMin")} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${wasmBridgeInstanceCommandCheckboxFromInstancesR13B(bridgeInstances, String(net || ""), instance.id, "instancePow2Clamp")}
              <span class="kgw-command-option-title-text-r8e">pow2</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${wasmBridgeElementId(net, `instancePow2Clamp-${instance.id}`)}" data-bridge-instance-field="instancePow2Clamp">
              <option value="not set" ${instance.instancePow2Clamp === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instancePow2Clamp === "false" ? "selected" : ""}>false</option>
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
    ["basic", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.basic", "Basic")],
    ["rpc", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.rpc", "RPC")],
    ["storage", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.storage", "Storage / Index")],
    ["p2p", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.p2p", "P2P / Network")],
    ["perf", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.performance", "Performance / Logs")],
    ["advanced", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.advanced", "Advanced")],
    ["danger", wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.tab.dangerous", "Dangerous")]
  ];

  const tabButtons = tabs.map(([key, label], index) =>
    `<button type="button" class="bridge-v12d-node-tab${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-inprocess-node-tab="${key}">${wasmBridgeEscapeHtml(label)}</button>`
  ).join("");

  const testnetArgs = net.testnet
    ? `--testnet${net.netsuffix ? " --netsuffix=" + wasmBridgeEscapeHtml(net.netsuffix) : ""}`
    : "mainnet";

  const markup = `
    <div class="bridge-v12d-inprocess-node-settings bridge-v12d-inprocess-inactive" data-net="${net.key}" data-bridge-inprocess-node-settings="${net.key}" data-kgw-owner="KGW_BRIDGE_INPROCESS_KASPAD_ARGS_TABS_V12D">
      <div class="bridge-v12d-node-tabs">${tabButtons}</div>

      <section class="bridge-v12d-node-panel active" data-net="${net.key}" data-bridge-inprocess-node-panel="basic">
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card span2">
            <span data-i18n="bridge.inprocessNodeSettings.appdir">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.appdir", "same --appdir / database path"))}</span>
            <input id="${wasmBridgeElementId(net.key, "inprocessAppdirMirror")}" type="text" value="" readonly>
          </div>
          <div class="bridge-v7-card span2">
            <span data-i18n="bridge.inprocessNodeSettings.testnet">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.testnet", "kaspad network args"))}</span>
            <input id="${wasmBridgeElementId(net.key, "inprocessNetworkArgs")}" type="text" value="${wasmBridgeEscapeHtml(testnetArgs)}" readonly>
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="rpc" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessRpcListen")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.rpcListen">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.rpcListen", "--rpclisten"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessRpcListen")}" type="text" value="127.0.0.1:${wasmBridgeEscapeHtml(net.kaspadPort)}">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessRpcListenBorsh")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.rpcListenBorsh">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.rpcListenBorsh", "--rpclisten-borsh"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessRpcListenBorsh")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessRpcListenJson")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.rpcListenJson">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.rpcListenJson", "--rpclisten-json"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessRpcListenJson")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check danger">
            <input id="${wasmBridgeElementId(net.key, "inprocessUnsafeRpc")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.unsafeRpc">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.unsafeRpc", "--unsaferpc"))}</span>
          </label>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="storage" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <label class="bridge-v7-card check">
            <input id="${wasmBridgeElementId(net.key, "inprocessUtxoIndex")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.utxoIndex">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.utxoIndex", "--utxoindex"))}</span>
          </label>
          <label class="bridge-v7-card check">
            <input id="${wasmBridgeElementId(net.key, "inprocessArchival")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.archival">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.archival", "--archival"))}</span>
          </label>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="p2p" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessListen")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.listen">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.listen", "--listen"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessListen")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessAddPeer")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.addPeer">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.addPeer", "--addpeer"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessAddPeer")}" type="text" value="">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessConnect")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.connect">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.connect", "--connect"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessConnect")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check">
            <input id="${wasmBridgeElementId(net.key, "inprocessDisableUpnp")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.disableUpnp">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.disableUpnp", "--disable-upnp"))}</span>
          </label>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessMaxInpeers")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.maxInpeers">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.maxInpeers", "--maxinpeers"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessMaxInpeers")}" type="number" min="0" max="32" step="1" value="32">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessOutpeers")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.outpeers">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.outpeers", "--outpeers"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessOutpeers")}" type="number" min="0" max="8" step="1" value="8">
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="perf" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          ${wasmBridgeCardInput(String(net.key || ""), "inprocessAsyncThreads", "--async-threads", "16", "", "", "")}
          <label class="bridge-v7-card check">
            <input id="${wasmBridgeElementId(net.key, "inprocessPerfMetrics")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.perfMetrics">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.perfMetrics", "--perf-metrics"))}</span>
          </label>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessPerfMetricsIntervalSec")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.perfMetricsIntervalSec">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.perfMetricsIntervalSec", "--perf-metrics-interval-sec"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessPerfMetricsIntervalSec")}" type="number" min="1" step="1" value="10">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessLogLevel")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.logLevel">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.logLevel", "--loglevel"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessLogLevel")}" type="text" value="info">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessRamScale")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.ramScale">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.ramScale", "--ram-scale"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessRamScale")}" type="number" min="0.1" step="0.1" value="1">
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="advanced" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessConfigfile")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.configfile">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.configfile", "--configfile"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessConfigfile")}" type="text" value="" placeholder="unsupported: managed ownership">
          </div>
          <label class="bridge-v7-card check">
            <input id="${wasmBridgeElementId(net.key, "inprocessYes")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.yes">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.yes", "--yes"))}</span>
          </label>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="danger" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${wasmBridgeCommandInlineToggleR7(String(net.key || ""), "inprocessOverrideParamsFile")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.overrideParamsFile">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.overrideParamsFile", "--override-params-file"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${wasmBridgeElementId(net.key, "inprocessOverrideParamsFile")}" type="text" value="">
          </div>
          <label class="bridge-v7-card check danger">
            <input id="${wasmBridgeElementId(net.key, "inprocessDevnet")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.devnet">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.devnet", "--devnet"))}</span>
          </label>
          <label class="bridge-v7-card check danger">
            <input id="${wasmBridgeElementId(net.key, "inprocessSimnet")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.simnet">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.simnet", "--simnet"))}</span>
          </label>
          <label class="bridge-v7-card check danger">
            <input id="${wasmBridgeElementId(net.key, "inprocessEnableUnsyncedMining")}" type="checkbox">
            <span data-i18n="bridge.inprocessNodeSettings.enableUnsyncedMining">${wasmBridgeEscapeHtml(wasmBridgeI18nTextR41("bridge.inprocessNodeSettings.enableUnsyncedMining", "--enable-unsynced-mining"))}</span>
          </label>
        </div>
      </section>
    </div>`;
  const template = document.createElement("template");
  template.innerHTML = markup;
  const danger = template.content.querySelector('[data-bridge-inprocess-node-panel="danger"] .bridge-v7-grid');
  const unsafe = template.content.querySelector('[id$="-inprocessUnsafeRpc"]')?.closest(".bridge-v7-card");
  if (danger && unsafe) danger.prepend(unsafe);
  if (danger) danger.insertAdjacentHTML("beforebegin", '<p class="kgw-danger-warning">Unsafe RPC exposes RPC beyond loopback. Unsynced mining bypasses synchronization. Enable only when you understand the risk.</p>');
  return template.innerHTML;
}
function renderSections(net) {
  const template = document.createElement("template");
  template.innerHTML = [wasmBridgeRenderRuntime(net || {}), wasmBridgeRenderLogging(net || {}), wasmBridgeRenderDifficulty(net || {}), wasmBridgeRenderPorts(net || {}),
    ...(net.key === "mainnet" ? [] : [wasmBridgeRenderCpuMiner(net || {})])].join("");
  const cards = new Map();
  template.content.querySelectorAll(".bridge-v7-card").forEach(card => {
    const field = card.querySelector("[id]");
    if (!field) return;
    const name = field.id.slice(("bridge-" + net.key + "-").length);
    const testnetAllowed = ["nodeMode", "kaspadAddress", "appdir", "testnet"].includes(name) || name.startsWith("internalCpuMiner");
    if (net.key === "mainnet" || testnetAllowed) cards.set(name, card.outerHTML);
  });
  const take = names => {
    const fields = names.split(" ").map(name => { const card = cards.get(name) || ""; cards.delete(name); return card; }).join("");
    return '<div class="kgw-settings-grid">' + fields + "</div>";
  };
  const inprocess = document.createElement("template");
  inprocess.innerHTML = renderInprocessNodeSettings(net);
  const danger = inprocess.content.querySelector('[data-bridge-inprocess-node-panel="danger"]');
  const dangerBody = danger.innerHTML;
  danger.remove();
  inprocess.content.querySelector('[data-bridge-inprocess-node-tab="danger"]').remove();
  const groups = [
    ["general", "connection", "Connection", take("nodeMode kaspadAddress appdir testnet")],
    ...(net.key === "mainnet" ? [["general", "ports", "Ports", take("stratumPort promPort healthCheckPort webDashboardPort")]] : []),
    ["general", "mining", net.key === "mainnet" ? "Mining" : "CPU Mining",
      take(net.key === "mainnet" ? "minShareDiff blockWaitTime extranonceSize coinbaseTagSuffix" :
        "internalCpuMiner internalCpuMinerAddress internalCpuMinerThreads internalCpuMinerThrottleMs internalCpuMinerTemplatePollMs")],
    ["general", "monitoring", "Monitoring", net.key === "mainnet" ? take("printStats") :
      '<p class="kgw-settings-info">The embedded rkstratum_cpu_miner uses this network&#39;s node. Live Bridge Monitor shows runtime state and raw logs.</p>'],
    ...(net.key === "mainnet" ? [["advanced", "instances", "Instances",
      '<div id="' + wasmBridgeElementId(net.key, "instances") + '">' + renderInstances(net.key) + "</div>"]] : []),
    ["advanced", "inprocessor", "In-Processor", inprocess.innerHTML],
    ...(net.key === "mainnet" ? [["advanced", "difficulty", "Difficulty", take("varDiff sharesPerMin varDiffStats pow2Clamp")]] : []),
    ["advanced", "diagnostics", "Logging / Diagnostics", net.key === "mainnet" ? take("config logToFile approxGeoLookup") :
      '<p class="kgw-settings-info">Raw stdout/stderr logs are available in Live Bridge Monitor. Managed logging and network ownership remain unchanged.</p>'],
    ["advanced", "dangerous", "Dangerous", dangerBody]
  ];
  if (cards.size) throw new Error("Ungrouped Bridge settings: " + [...cards.keys()].join(", "));
  return wasmBridgeDifficultyDatalistR16C() + renderSettingsTabs("bridge", net.key, groups);
}

/* KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
 * Default Bridge inner tab is Live Bridge Monitor.
 * Last selected inner tab is saved per network.
 */
/* R101U inner-tab persistence is Rust-owned in bridge_frontend_helpers.rs. */

function renderNetworkPanel(net, index) {
  /* KGW_BRIDGE_LIVE_MONITOR_TAB_LABEL_ORDER_R101S */
  /* KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
   * Settings is no longer the default inner panel.
   * Default is Live Bridge Monitor unless a valid saved tab exists for this network.
   */
  const activeInnerTab = wasmBridgeResolveInnerTab(String(net.key || ""));
  const logActive = activeInnerTab === "log";
  const settingsActive = activeInnerTab === "settings";

  return `
    <div class="bridge-v7-network-panel${index === 0 ? " active" : ""}" data-bridge-network-panel="${net.key}" data-testid="kgw-bridge-panel-${net.key}"${index === 0 ? "" : " hidden"}>
      <section class="kgw-network-policy${net.experimental ? " is-experimental" : ""}" data-net="${net.key}" data-testid="kgw-bridge-policy-${net.key}">
        <div>
          <strong>${net.label}</strong>${net.experimental ? '<span class="kgw-experimental-badge">Experimental - opt-in required</span>' : ""}
          <span>${wasmBridgeEscapeHtml(wasmBridgeNetworkPolicyMessage(net.key))}</span>
        </div>
        <div class="kgw-network-policy-controls">
          <span id="${wasmBridgeElementId(net.key, "policyStatus")}" class="kgw-network-policy-status">Stopped</span>
          <label>
            <input type="checkbox" data-bridge-network-enabled="${net.key}" data-testid="kgw-bridge-policy-enabled-${net.key}" data-net="${net.key}"${wasmBridgeNetworkEnabled(net.key) ? " checked" : ""}>
            Profile enabled
          </label>
        </div>
      </section>
      <div class="bridge-v7-inner-tabs">
        <button type="button" class="bridge-v7-inner-tab${logActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-tab="log" data-testid="kgw-bridge-live-monitor-${net.key}">Live Bridge Monitor</button>
        <button type="button" class="bridge-v7-inner-tab${settingsActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-tab="settings" data-testid="kgw-bridge-settings-${net.key}">Settings</button>
      </div>

      <div class="bridge-v7-inner-panel${settingsActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-panel="settings"${settingsActive ? "" : " hidden"}>
        <div class="kgw-settings-scroll">
        <section class="bridge-v7-command kgw-effective-preview">
          <div class="kgw-preview-row">
            <strong>Effective bridge settings</strong>
            <button type="button" data-settings-preview-toggle aria-expanded="false" aria-controls="${wasmBridgeElementId(net.key, "previewBody")}">Expand</button>
            <button type="button" class="bridge-v7-copy" data-bridge-action="copy-command" data-net="${net.key}" title="Copy effective settings">Copy settings</button>
            <button type="button" data-bridge-action="copy-path" data-net="${net.key}">Copy data directory</button>
          </div>
          <p id="${wasmBridgeElementId(net.key, "previewStatus")}" class="kgw-preview-status" role="status" aria-live="polite"></p>
          <div class="kgw-preview-body" id="${wasmBridgeElementId(net.key, "previewBody")}" hidden>
            <p class="kgw-preview-help">The embedded node and bridge libraries consume these settings inside KaspaGateway self-workers.</p>
            <textarea id="${wasmBridgeElementId(net.key, "commandPreview")}" aria-label="Effective bridge settings preview" readonly spellcheck="false" wrap="soft"></textarea>
          </div>
        </section>

        <section class="bridge-v7-toolbar">
          <div class="bridge-v7-buttons">
            <button type="button" class="good" data-bridge-action="start" data-testid="kgw-bridge-start-${net.key}" data-net="${net.key}">Start</button>
            <button type="button" data-bridge-action="stop" data-testid="kgw-bridge-stop-${net.key}" data-net="${net.key}">Stop</button>
          </div>

          <div class="bridge-v7-status">
            <span id="${wasmBridgeElementId(net.key, "settingsAuthority")}" class="bridge-v7-runtime-status">Effective settings apply on next Start</span>
          </div>
          <div id="${wasmBridgeElementId(net.key, "runtimeStatus")}" class="bridge-v7-runtime-status" role="status" aria-live="polite"></div>
          <div id="${wasmBridgeElementId(net.key, "runtimeError")}" class="bridge-v7-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        ${renderSections(net)}
        </div>

        <div class="settings-bottom-actions bridge-settings-bottom-actions">
        <button type="button" data-bridge-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-bridge-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-bridge-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        <p class="kgw-settings-help" data-settings-defaults-context="${net.key}">Restore uses KaspaGateway defaults. Settings apply on the next Start.</p>
        </div>

      </div>

      <div class="bridge-v7-inner-panel${logActive ? " active" : ""}" data-net="${net.key}" data-bridge-inner-panel="log" data-testid="kgw-bridge-live-panel-${net.key}"${logActive ? "" : " hidden"}>
        <p id="${wasmBridgeElementId(net.key, "monitorState")}" class="kgw-monitor-state" role="status">Bridge: Stopped. Node connection: not checked.</p>
        <div class="bridge-v7-log-toolbar">
          <button type="button" data-bridge-action="monitor-next" data-net="${net.key}">Configure Node</button>
          <button type="button" data-bridge-action="copy-log" data-testid="kgw-bridge-copy-log-${net.key}" data-net="${net.key}">Copy Log</button>
          <button type="button" data-bridge-action="clear-log" data-testid="kgw-bridge-clear-log-${net.key}" data-net="${net.key}">Clear Log</button>
        </div>
        <div id="${wasmBridgeElementId(net.key, "logEmpty")}" class="bridge-v7-log-empty" data-bridge-log-empty="${net.key}">Bridge is stopped. Choose a node connection in Settings, then start the bridge.</div>
        <pre id="${wasmBridgeElementId(net.key, "logOutput")}" class="bridge-v7-log" data-testid="kgw-bridge-log-output-${net.key}"></pre>
      </div>
</div>`;
}


function bridgeReadInstanceState(net, instanceId) {
  const current = bridgeInstances[net].find((instance) => String(instance.id) === String(instanceId)) || {};
  const next = wasmBridgeNormalizeInstanceRecord(current || {}, Date.now() + Math.floor(Math.random() * 1000));

  return wasmBridgeAssignMissingInstancePortsR9(bridgeInstances, String(net || ""), {
    id: next.id || instanceId || Date.now() + Math.floor(Math.random() * 1000),
    instance: "",
    instancePort: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instancePort") || next.instancePort || "",
    instanceDiff: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceDiff") || next.instanceDiff || "2048",
    instanceProm: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceProm") || next.instanceProm || "",
    instanceLogToFile: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceLogToFile") || next.instanceLogToFile || "not set",
    instanceBlockWaitTime: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceBlockWaitTime") || next.instanceBlockWaitTime || "",
    instanceExtranonceSize: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceExtranonceSize") || next.instanceExtranonceSize || "",
    instanceVarDiff: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceVarDiff") || next.instanceVarDiff || "not set",
    instanceSharesPerMin: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceSharesPerMin") || next.instanceSharesPerMin || "",
    instanceVarDiffStats: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instanceVarDiffStats") || next.instanceVarDiffStats || "not set",
    instancePow2Clamp: wasmBridgeReadInstanceField(String(net || ""), instanceId, "instancePow2Clamp") || next.instancePow2Clamp || "not set"
  });
}

function bridgeRefreshInstances(net) {
  net = wasmBridgeInstanceNetworkKeyR15(net, net);

  const container =
    wasmBridgeById(wasmBridgeElementId(net, "instances")) ||
    document.querySelector(`[data-bridge-network-panel="${net}"] [data-bridge-section-panel="instances"]`);

  if (container) {
    if (!container.id) {
      container.id = wasmBridgeElementId(net, "instances");
    }

    container.innerHTML = renderInstances(net);
    decorateSettingsFields(container);
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
  net = wasmBridgeInstanceNetworkKeyR15(net, net);
  if (!container || !net) return;

  container.dataset.kgwBridgeInstancesClickOwner = "KGW_BRIDGE_INSTANCES_REBUILD_CLICK_OWNER_R11";
  container.onclick = function bridgeInstancesContainerClickOwnerR11(event) {
    const control = event.target && event.target.closest
      ? event.target.closest("[data-bridge-action]")
      : null;

    if (!control || !container.contains(control)) return;

    const action = control.dataset.bridgeAction || "";
    const targetNet = wasmBridgeInstanceNetworkKeyR15(control.dataset.network, net);

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
      wasmBridgeRenderRawLogBuffer(String(targetNet || ""), "bridge", String(activeInstance[targetNet] || ""));
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
    const container = wasmBridgeById(wasmBridgeElementId(profile.key, "instances"));
    if (container) {
      bridgeInstallInstanceContainerOwnerR11(container, profile.key);
    }
  }
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



function addInstance(net) {
  wasmBridgeSmallOwnerTraceR44D(net, "add-instance", "r44d-owner-begin", {});
  net = wasmBridgeInstanceNetworkKeyR15(net, net);
  wasmBridgeEnsureInstanceState(bridgeInstances, activeInstance, String(net || ""));

  const next = wasmBridgeCreateInstanceRecordR9(bridgeInstances, String(net || ""));
  bridgeInstances[net].push(next);
  activeInstance[net] = next.id;

  bridgeRefreshInstances(net);
  wasmBridgeRenderRawLogBuffer(String(net || ""), "bridge", String(activeInstance[net] || ""));
  updateCommand(net);
  wasmBridgeSmallOwnerTraceR44D(net, "add-instance", "r44d-owner-complete", {});
}


function removeInstance(net, instanceId) {
  wasmBridgeSmallOwnerTraceR44D(net, "remove-instance", "r44d-owner-begin", { instanceId: String(instanceId || "") });
  wasmBridgeEnsureInstanceState(bridgeInstances, activeInstance, String(net || ""));
  if (bridgeInstances[net].length <= 1) return;
  const removedIndex = bridgeInstances[net].findIndex(instance => String(instance.id) === String(instanceId));
  bridgeInstances[net] = bridgeInstances[net].filter((instance) => String(instance.id) !== String(instanceId));
  if (!bridgeInstances[net].some(instance => String(instance.id) === String(activeInstance[net]))) {
    activeInstance[net] = bridgeInstances[net][Math.max(0, removedIndex - 1)].id;
  }
  bridgeRefreshInstances(net);
  wasmBridgeSmallOwnerTraceR44D(net, "remove-instance", "r44d-owner-complete", { instanceId: String(instanceId || "") });
}


function renderAllNetworks(root) {
  const host = root.querySelector("#bridgeNetworkPanels");
  if (!host) return;
  host.innerHTML = BRIDGE_NETWORKS.map(renderNetworkPanel).join("");
  installSettingsLayout(root);


  setTimeout(wasmBridgeInstallLogAutoScrollControls, 0);
  setTimeout(window.kgwInstallBridgeLogScopedControlsV29, 0);
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
function kgwBridgeR51ReadStructuredInstancesR253(net) {
  return wasmBridgeR51ReadStructuredInstances(
    String(net || ""),
    bridgeInstances,
    activeInstance,
    {
      readInstanceState: (targetNet, instanceId) =>
        bridgeReadInstanceState(String(targetNet || ""), instanceId)
    }
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
