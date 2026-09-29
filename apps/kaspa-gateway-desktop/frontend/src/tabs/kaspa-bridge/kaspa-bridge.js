import { applyStatusTone, renderStatusSummary } from "../../status.js";
import { BRIDGE_MANAGED, BRIDGE_OPTIONAL, bridgeFieldEnabled, validateBridgeForm, renderFieldErrors, runtimePresentation, runtimeObservationSummary, confirmUserAction } from "../../settings-contract.js";
import { renderSettingsTabs, installSettingsLayout, decorateSettingsFields, revealSettingsField, setSettingFieldState } from "../../settings-layout.js";
import initBridgeRust, {
  bridgeApplyRuntimeLogReport as wasmBridgeApplyRuntimeLogReport,
  bridgeApplyRustyKaspaRootOnlyDefaultPathsR5 as wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsR5,
  bridgeById as wasmBridgeById,
  bridgeChecked as wasmBridgeChecked,
  bridgeCommandInlineStateR7 as wasmBridgeCommandInlineStateR7,
  bridgeCommandInlineToggleR7 as wasmBridgeCommandInlineToggleR7,
  bridgeCommandOptionEnabledR7 as wasmBridgeCommandOptionEnabledR7,
  bridgeCommandSetOptionR7 as wasmBridgeCommandSetOptionR7,
  bridgeCommandShouldIncludeR7 as wasmBridgeCommandShouldIncludeR7,
  bridgeCommandToggleOptionR7 as wasmBridgeCommandToggleOptionR7,
  bridgeBuildUpstreamInstanceArg as wasmBridgeBuildUpstreamInstanceArg,
  bridgeDefaultInstanceRecord as wasmBridgeDefaultInstanceRecord,
  bridgeEffectiveSettingsV1 as wasmBridgeEffectiveSettingsV1,
  bridgeNormalizeInstanceRecord as wasmBridgeNormalizeInstanceRecord,
  bridgeParseUnsignedV1 as wasmBridgeParseUnsignedV1,
  bridgeClearRawLogBuffer as wasmBridgeClearRawLogBuffer,
  bridgeInstanceCommandCheckboxR13B as wasmBridgeInstanceCommandCheckboxR13B,
  bridgeInstanceCommandSetOptionR13B as wasmBridgeInstanceCommandSetOptionR13B,
  bridgeInstanceCommandShouldIncludeR13B as wasmBridgeInstanceCommandShouldIncludeR13B,

  bridgeDispatchClipboardWrite as wasmBridgeDispatchClipboardWrite,
  bridgeElementId as wasmBridgeElementId,
  bridgeEscapeHtml as wasmBridgeEscapeHtml,
  bridgeHandleLogAction as wasmBridgeHandleLogAction,
  bridgeI18nTextR41 as wasmBridgeI18nTextR41,
  bridgeNetworkEnabled as wasmBridgeNetworkEnabled,
  bridgeNetworkPolicyMessage as wasmBridgeNetworkPolicyMessage,
  bridgeNetworkProfile as wasmBridgeNetworkProfile,
  bridgeNetworkProfiles as wasmBridgeNetworkProfiles,
  bridgeRenderRawLogBuffer as wasmBridgeRenderRawLogBuffer,
  bridgeSetNetworkEnabled as wasmBridgeSetNetworkEnabled,
  bridgeSmallOwnerTraceR44D as wasmBridgeSmallOwnerTraceR44D,
  bridgeValue as wasmBridgeValue,
  bridgeAddUsedPortR91 as wasmBridgeAddUsedPortR91,
  bridgeApplyPortConflictStartStateR33 as wasmBridgeApplyPortConflictStartStateR33,
  bridgeChooseReplacementPortR37 as wasmBridgeChooseReplacementPortR37,
  bridgeExtractPortsFromTextR5 as wasmBridgeExtractPortsFromTextR5,
  bridgePlanPortAutofixR37 as wasmBridgePlanPortAutofixR37,
  bridgeFindRecommendedOrNearestUnusedPortR35B as wasmBridgeFindRecommendedOrNearestUnusedPortR35B,
  bridgeWriteInstancePortR37 as wasmBridgeWriteInstancePortR37,
  bridgeInstancePortShouldFollowExternalRangeR91 as wasmBridgeInstancePortShouldFollowExternalRangeR91,
  bridgeNormalizePortR9 as wasmBridgeNormalizePortR9,
  bridgePortIsValidR9 as wasmBridgePortIsValidR9,
  bridgeValidatePortConflictsR5 as wasmBridgeValidatePortConflictsR5,
  bridgePortProfileR35B as wasmBridgePortProfileR35B,
  bridgePortProfilesR35B as wasmBridgePortProfilesR35B,
  bridgePushPortR5 as wasmBridgePushPortR5,
  bridgeStaticPortProfileR91 as wasmBridgeStaticPortProfileR91,
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
    keys: () => kgwBridgeR51Keys(),
    readSettings: (net) => kgwBridgeR51ReadSettings(String(net || "")),
    load: (key) => kgwBridgeR51Load(String(key || "")),
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

function kgwBridgeSmallOwnerTraceR44D(net, action, phase, details) {
  return wasmBridgeSmallOwnerTraceR44D(net, action, phase, details || {});
}




function kgwI18nTextR41(key, fallback) {
  return wasmBridgeI18nTextR41(String(key || ""), String(fallback || ""));
}


/* Canonical isolated bridge/node runtime paths.
 * In-process bridge mode shares the same network-specific database at:
 * %LOCALAPPDATA%\KaspaGateway\nodes\<network>
 */







async function kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsR5(net, _options = {}) {
  return await wasmBridgeApplyRustyKaspaRootOnlyDefaultPathsR5(
    String(net || ""),
    (resolvedNet) => updateCommand(resolvedNet)
  );
}

function kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, options = {}) {
  void kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsR5(net, options).catch(() => {});
}

/* KGW_BRIDGE_NETWORK_PORT_RANGES_DEFAULTS_PATCH_R42
 * Defaults now follow the agreed soft network port ranges.
 * These are defaults only. Manual valid unused ports remain accepted anywhere.
 */
const BRIDGE_NETWORKS = wasmBridgeNetworkProfiles();



function kgwBridgeNetworkProfile(net) {
  return wasmBridgeNetworkProfile(String(net || ""));
}

function kgwBridgeNetworkEnabled(net) {
  return wasmBridgeNetworkEnabled(String(net || ""));
}

function kgwBridgeSetNetworkEnabled(net, enabled) {
  wasmBridgeSetNetworkEnabled(String(net || ""), Boolean(enabled));
}

function kgwBridgeNetworkPolicyMessage(net) {
  return wasmBridgeNetworkPolicyMessage(String(net || ""));
}

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

function byId(id) {
  return wasmBridgeById(String(id || ""));
}

function esc(value) {
  return wasmBridgeEscapeHtml(value);
}

function id(net, name) {
  return wasmBridgeElementId(String(net || ""), String(name || ""));
}

function v(net, name) {
  return wasmBridgeValue(String(net || ""), String(name || ""));
}

function c(net, name) {
  return wasmBridgeChecked(String(net || ""), String(name || ""));
}




// KGW_BRIDGE_INSTANCES_COMMAND_CHECKBOX_R13B

function kgwBridgeInstanceCommandRecordR13B(net, instanceId) {
  return (bridgeInstances[net] || []).find(item => String(item.id) === String(instanceId)) || null;
}

function kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, name) {
  return wasmBridgeInstanceCommandShouldIncludeR13B(
    String(net || ""),
    instanceId,
    String(name || ""),
    kgwBridgeInstanceCommandRecordR13B(net, instanceId)
  );
}

function kgwBridgeInstanceCommandCheckboxR13B(net, instanceId, name) {
  return wasmBridgeInstanceCommandCheckboxR13B(
    String(net || ""),
    instanceId,
    String(name || ""),
    kgwBridgeInstanceCommandRecordR13B(net, instanceId)
  );
}

function kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, enabled) {
  kgwBridgeSmallOwnerTraceR44D(net, "command-checkbox", "r29b-bridge-instance-command-checkbox-begin", {
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
  kgwBridgeR51Panel(net)?.querySelectorAll("[data-bridge-instance-command-option-toggle-r13b]").forEach(toggle => {
    if (String(toggle.dataset.instanceId) !== String(instanceId) || toggle.dataset.bridgeInstanceCommandOptionToggleR13b !== name) return;
    toggle.checked = Boolean(enabled);
    toggle.title = enabled ? "Included in command" : "Excluded from command";
    toggle.setAttribute("aria-label", toggle.title);
  });
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

function kgwBridgeCommandInlineStateR7(net) {
  return wasmBridgeCommandInlineStateR7(String(net || ""));
}

function kgwBridgeCommandOptionEnabledR7(net, name) {
  return wasmBridgeCommandOptionEnabledR7(String(net || ""), String(name || ""));
}

function kgwBridgeCommandShouldIncludeR7(net, name) {
  return wasmBridgeCommandShouldIncludeR7(String(net || ""), String(name || ""));
}

function kgwBridgeCommandInlineToggleR7(net, name) {
  return wasmBridgeCommandInlineToggleR7(String(net || ""), String(name || "")); // KGW_BRIDGE_COMMAND_COMPOSER_CHECKBOX_ONLY_R9
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
  wasmBridgeCommandToggleOptionR7(String(net || ""), String(name || ""));
  kgwBridgeRefreshInlineCommandTogglesR7(net);
  updateCommand(net);
}


// KGW_BRIDGE_DIFFICULTY_DATALIST_R16C

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
      ${cardSelect(net.key, "logToFile", "--log-to-file", ["true", "false"], "false")}
      ${cardSelect(net.key, "approxGeoLookup", "--approximate-geo-lookup", ["not set", "true", "false"], "not set", "span2")}
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
      ${cardCheck(net.key, "internalCpuMiner", "Enable CPU Mining", false)}
      ${cardInput(net.key, "internalCpuMinerAddress", "Mining / Reward Address", "", "kaspatest:...", "span2")}
      ${cardInput(net.key, "internalCpuMinerThreads", "CPU Threads", "1", "threads")}
      ${cardInput(net.key, "internalCpuMinerThrottleMs", "Throttle (milliseconds)", "", "optional")}
      ${cardInput(net.key, "internalCpuMinerTemplatePollMs", "Template Poll Interval (milliseconds)", "", "optional", "span2")}
    </div>`;
}


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
function bridgeBuildUpstreamInstanceArg(net, instance) {
  return wasmBridgeBuildUpstreamInstanceArg(String(net || ""), instance || {});
}

function kgwBridgeParseUnsignedV1(label, value, fallback, max = Number.MAX_SAFE_INTEGER) {
  return wasmBridgeParseUnsignedV1(String(label || ""), value, fallback, max);
}

function kgwBridgeEffectiveSettingsV1(net, structuredInstances) {
  return wasmBridgeEffectiveSettingsV1(String(net || ""), structuredInstances || {});
}

function bridgeDefaultInstanceRecord(idValue) {
  return wasmBridgeDefaultInstanceRecord(idValue);
}

function bridgeNormalizeInstanceRecord(raw, fallbackId) {
  return wasmBridgeNormalizeInstanceRecord(raw || {}, fallbackId);
}

function bridgeExtractPortsFromTextR5(value) {
  return Array.from(wasmBridgeExtractPortsFromTextR5(value));
}

function bridgePushPortR5(items, port, role, owner, net) {
  return wasmBridgePushPortR5(items, port, role, owner, net);
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
  return wasmBridgeValidatePortConflictsR5(bridgeCollectConfiguredPortsR5(), String(activeNet || ""));
}

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
function bridgeNormalizePortR9(value) {
  return wasmBridgeNormalizePortR9(value);
}

function bridgePortIsValidR9(value) {
  return wasmBridgePortIsValidR9(value);
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





function bridgeInstancePortShouldFollowExternalRangeR91(net, kind, value) {
  return wasmBridgeInstancePortShouldFollowExternalRangeR91(net, kind, value);
}

function bridgeAddUsedPortR91(used, value) {
  return wasmBridgeAddUsedPortR91(used, value);
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
  if (!kgwBridgeInstanceCommandShouldIncludeR13B(net, instance.id, "instance")) return "Excluded from runtime.";
  const raw = byId(id(net, "commandPreview"))?.dataset.effectiveSettings;
  if (!raw) return "Waiting for validated effective settings.";
  try {
    const resolved = JSON.parse(raw).effectiveBridgeSettings?.instances?.find(item => String(item.instanceId) === String(instance.id));
    return resolved ? JSON.stringify(resolved) : "No effective instance in this configuration.";
  } catch { return "Waiting for validated effective settings."; }
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
}

function bridgeAssertNoPortConflictsR5(net) {
  if (net !== "mainnet") return;
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

  const cfg = kgwBridgeNetworkProfile(net) || {};
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
    kgwBridgeSmallOwnerTraceR44D(net, "port-conflict", "r110h-scoped-start-conflict-check", conflictDetails);
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

  if (!activeRecord) {
    return { ok: true, conflictCount: 0, conflicts: [], message: "" };
  }

  if (!activePort) {
    const msg = "Active Bridge instance has no valid Stratum port.";
    try {
      kgwBridgeSmallOwnerTraceR44D(net, "port-conflict", "r110h-active-instance-port-invalid", {
        ...conflictDetails,
        message: msg
      });
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
 * - Covers mainnet, testnet10, testnet13 through BRIDGE_NETWORKS.
 */
function bridgeApplyPortConflictStartStateR33(net, validation, reason) {
  return wasmBridgeApplyPortConflictStartStateR33(
    String(net || ""),
    validation || {},
    String(reason || "")
  );
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
  if (net !== "mainnet") return ""; // Testnet mining is embedded CPU-only, including restored settings.
  bridgeEnsureInstanceState(net);

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
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instance")}
              <span class="kgw-command-option-title-text-r8e">Effective instance</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input
              readonly
              data-bridge-instance-preview="true"
              data-network="${net}"
              data-instance-id="${instance.id}"
              value="${esc(bridgeInstancePreviewTextR8B(net, instance))}"
              title="${esc(bridgeInstancePreviewTextR8B(net, instance))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instancePort")}
              <span class="kgw-command-option-title-text-r8e">port</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instancePort-${instance.id}`)}" data-bridge-instance-field="instancePort" value="${esc(instance.instancePort || "")}" placeholder="${esc(bridgeInstancePortPlaceholderR49(net))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceDiff")}
              <span class="kgw-command-option-title-text-r8e">diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instanceDiff-${instance.id}`)}" data-bridge-instance-field="instanceDiff" value="${esc(instance.instanceDiff || "2048")}" placeholder="2048" ${kgwBridgeDifficultyInputAttrsR16C("instanceDiff")} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceProm")}
              <span class="kgw-command-option-title-text-r8e">prom</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instanceProm-${instance.id}`)}" data-bridge-instance-field="instanceProm" value="${esc(instance.instanceProm || "")}" placeholder="${esc(bridgeInstancePromPlaceholderR49(net))}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceLogToFile")}
              <span class="kgw-command-option-title-text-r8e">log</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instanceLogToFile-${instance.id}`)}" data-bridge-instance-field="instanceLogToFile">
              <option value="not set" ${instance.instanceLogToFile === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instanceLogToFile === "false" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceLogToFile === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceBlockWaitTime")}
              <span class="kgw-command-option-title-text-r8e">wait</span>
            </span>
            <input id="${id(net, `instanceBlockWaitTime-${instance.id}`)}" data-bridge-instance-field="instanceBlockWaitTime" value="${esc(instance.instanceBlockWaitTime || "")}" placeholder="Enable to override global milliseconds" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceExtranonceSize")}
              <span class="kgw-command-option-title-text-r8e">extranonce</span>
            </span>
            <input id="${id(net, `instanceExtranonceSize-${instance.id}`)}" data-bridge-instance-field="instanceExtranonceSize" value="${esc(instance.instanceExtranonceSize || "")}" placeholder="optional" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceVarDiff")}
              <span class="kgw-command-option-title-text-r8e">var_diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instanceVarDiff-${instance.id}`)}" data-bridge-instance-field="instanceVarDiff">
              <option value="not set" ${instance.instanceVarDiff === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instanceVarDiff === "false" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiff === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceVarDiffStats")}
              <span class="kgw-command-option-title-text-r8e">var_stats</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instanceVarDiffStats-${instance.id}`)}" data-bridge-instance-field="instanceVarDiffStats">
              <option value="not set" ${instance.instanceVarDiffStats === "not set" ? "selected" : ""}>Inherit global</option>
              <option value="false" ${instance.instanceVarDiffStats === "false" ? "selected" : ""}>false</option>
              <option value="true" ${instance.instanceVarDiffStats === "true" ? "selected" : ""}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instanceSharesPerMin")}
              <span class="kgw-command-option-title-text-r8e">shares/min</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="${id(net, `instanceSharesPerMin-${instance.id}`)}" data-bridge-instance-field="instanceSharesPerMin" value="${esc(instance.instanceSharesPerMin || "")}" placeholder="optional" ${kgwBridgeDifficultyInputAttrsR16C("instanceSharesPerMin")} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              ${kgwBridgeInstanceCommandCheckboxR13B(net, instance.id, "instancePow2Clamp")}
              <span class="kgw-command-option-title-text-r8e">pow2</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="${id(net, `instancePow2Clamp-${instance.id}`)}" data-bridge-instance-field="instancePow2Clamp">
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

  const markup = `
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
            <input id="${id(net.key, "inprocessDisableUpnp")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.disableUpnp">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.disableUpnp", "--disable-upnp"))}</span>
          </label>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessMaxInpeers")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.maxInpeers">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.maxInpeers", "--maxinpeers"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessMaxInpeers")}" type="number" min="0" max="32" step="1" value="32">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessOutpeers")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.outpeers">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.outpeers", "--outpeers"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessOutpeers")}" type="number" min="0" max="8" step="1" value="8">
          </div>
        </div>
      </section>

      <section class="bridge-v12d-node-panel" data-net="${net.key}" data-bridge-inprocess-node-panel="perf" hidden>
        <div class="bridge-v7-grid bridge-v12d-inprocess-grid">
          ${cardInput(net.key, "inprocessAsyncThreads", "--async-threads", "16")}
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessPerfMetrics")}" type="checkbox" checked>
            <span data-i18n="bridge.inprocessNodeSettings.perfMetrics">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.perfMetrics", "--perf-metrics"))}</span>
          </label>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessPerfMetricsIntervalSec")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.perfMetricsIntervalSec">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.perfMetricsIntervalSec", "--perf-metrics-interval-sec"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessPerfMetricsIntervalSec")}" type="number" min="1" step="1" value="10">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessLogLevel")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.logLevel">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.logLevel", "--loglevel"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessLogLevel")}" type="text" value="info">
          </div>
          <div class="bridge-v7-card">
      <span class="kgw-command-option-title-row-r8e">
        ${kgwBridgeCommandInlineToggleR7(net.key, "inprocessRamScale")}
        <span class="kgw-command-option-title-text-r8e" data-i18n="bridge.inprocessNodeSettings.ramScale">${esc(kgwI18nTextR41("bridge.inprocessNodeSettings.ramScale", "--ram-scale"))}</span>
      </span> <!-- KGW_BRIDGE_INPROCESS_COMMAND_CHECKBOX_R13B -->
      <input id="${id(net.key, "inprocessRamScale")}" type="number" min="0.1" step="0.1" value="1">
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
      <input id="${id(net.key, "inprocessConfigfile")}" type="text" value="" placeholder="unsupported: managed ownership">
          </div>
          <label class="bridge-v7-card check">
            <input id="${id(net.key, "inprocessYes")}" type="checkbox" checked>
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
  template.innerHTML = [renderRuntime(net), renderLogging(net), renderDifficulty(net), renderPorts(net),
    ...(net.key === "mainnet" ? [] : [renderCpuMiner(net)])].join("");
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
      '<div id="' + id(net.key, "instances") + '">' + renderInstances(net.key) + "</div>"]] : []),
    ["advanced", "inprocessor", "In-Processor", inprocess.innerHTML],
    ...(net.key === "mainnet" ? [["advanced", "difficulty", "Difficulty", take("varDiff sharesPerMin varDiffStats pow2Clamp")]] : []),
    ["advanced", "diagnostics", "Logging / Diagnostics", net.key === "mainnet" ? take("config logToFile approxGeoLookup") :
      '<p class="kgw-settings-info">Raw stdout/stderr logs are available in Live Bridge Monitor. Managed logging and network ownership remain unchanged.</p>'],
    ["advanced", "dangerous", "Dangerous", dangerBody]
  ];
  if (cards.size) throw new Error("Ungrouped Bridge settings: " + [...cards.keys()].join(", "));
  return kgwBridgeDifficultyDatalistR16C() + renderSettingsTabs("bridge", net.key, groups);
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
          <strong>${net.label}</strong>${net.experimental ? '<span class="kgw-experimental-badge">Experimental - opt-in required</span>' : ""}
          <span>${esc(kgwBridgeNetworkPolicyMessage(net.key))}</span>
        </div>
        <div class="kgw-network-policy-controls">
          <span id="${id(net.key, "policyStatus")}" class="kgw-network-policy-status">Stopped</span>
          <label>
            <input type="checkbox" data-bridge-network-enabled="${net.key}" data-testid="kgw-bridge-policy-enabled-${net.key}" data-net="${net.key}"${kgwBridgeNetworkEnabled(net.key) ? " checked" : ""}>
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
            <button type="button" data-settings-preview-toggle aria-expanded="false" aria-controls="${id(net.key, "previewBody")}">Expand</button>
            <button type="button" class="bridge-v7-copy" data-bridge-action="copy-command" data-net="${net.key}" title="Copy effective settings">Copy settings</button>
            <button type="button" data-bridge-action="copy-path" data-net="${net.key}">Copy data directory</button>
          </div>
          <p id="${id(net.key, "previewStatus")}" class="kgw-preview-status" role="status" aria-live="polite"></p>
          <div class="kgw-preview-body" id="${id(net.key, "previewBody")}" hidden>
            <p class="kgw-preview-help">The embedded node and bridge libraries consume these settings inside KaspaGateway self-workers.</p>
            <textarea id="${id(net.key, "commandPreview")}" aria-label="Effective bridge settings preview" readonly spellcheck="false" wrap="soft"></textarea>
          </div>
        </section>

        <section class="bridge-v7-toolbar">
          <div class="bridge-v7-buttons">
            <button type="button" class="good" data-bridge-action="start" data-testid="kgw-bridge-start-${net.key}" data-net="${net.key}">Start</button>
            <button type="button" data-bridge-action="stop" data-testid="kgw-bridge-stop-${net.key}" data-net="${net.key}">Stop</button>
          </div>

          <div class="bridge-v7-status">
            <span id="${id(net.key, "settingsAuthority")}" class="bridge-v7-runtime-status">Effective settings apply on next Start</span>
          </div>
          <div id="${id(net.key, "runtimeStatus")}" class="bridge-v7-runtime-status" role="status" aria-live="polite"></div>
          <div id="${id(net.key, "runtimeError")}" class="bridge-v7-runtime-error" role="status" aria-live="polite" hidden></div>
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
        <p id="${id(net.key, "monitorState")}" class="kgw-monitor-state" role="status">Bridge: Stopped. Node connection: not checked.</p>
        <div class="bridge-v7-log-toolbar">
          <button type="button" data-bridge-action="monitor-next" data-net="${net.key}">Configure Node</button>
          <button type="button" data-bridge-action="copy-log" data-testid="kgw-bridge-copy-log-${net.key}" data-net="${net.key}">Copy Log</button>
          <button type="button" data-bridge-action="clear-log" data-testid="kgw-bridge-clear-log-${net.key}" data-net="${net.key}">Clear Log</button>
        </div>
        <div id="${id(net.key, "logEmpty")}" class="bridge-v7-log-empty" data-bridge-log-empty="${net.key}">Bridge is stopped. Choose a node connection in Settings, then start the bridge.</div>
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
    instanceLogToFile: bridgeReadInstanceField(net, instanceId, "instanceLogToFile") || next.instanceLogToFile || "not set",
    instanceBlockWaitTime: bridgeReadInstanceField(net, instanceId, "instanceBlockWaitTime") || next.instanceBlockWaitTime || "",
    instanceExtranonceSize: bridgeReadInstanceField(net, instanceId, "instanceExtranonceSize") || next.instanceExtranonceSize || "",
    instanceVarDiff: bridgeReadInstanceField(net, instanceId, "instanceVarDiff") || next.instanceVarDiff || "not set",
    instanceSharesPerMin: bridgeReadInstanceField(net, instanceId, "instanceSharesPerMin") || next.instanceSharesPerMin || "",
    instanceVarDiffStats: bridgeReadInstanceField(net, instanceId, "instanceVarDiffStats") || next.instanceVarDiffStats || "not set",
    instancePow2Clamp: bridgeReadInstanceField(net, instanceId, "instancePow2Clamp") || next.instancePow2Clamp || "not set"
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
 * Temporary scoped runtime trace for Bridge Instances across mainnet/testnet10/testnet13.
 * No global click listener. No forbidden legacy phase names.
 */



/* KGW_BRIDGE_INSTANCES_NETWORK_KEY_FIX_R15
 * Canonical network-key resolver for Bridge Instances.
 * Normalizes Bridge Instances network keys for mainnet, testnet10, and testnet13.
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


function removeInstance(net, instanceId) {
  kgwBridgeSmallOwnerTraceR44D(net, "remove-instance", "r44d-owner-begin", { instanceId: String(instanceId || "") });
  bridgeEnsureInstanceState(net);
  if (bridgeInstances[net].length <= 1) return;
  const removedIndex = bridgeInstances[net].findIndex(instance => String(instance.id) === String(instanceId));
  bridgeInstances[net] = bridgeInstances[net].filter((instance) => String(instance.id) !== String(instanceId));
  if (!bridgeInstances[net].some(instance => String(instance.id) === String(activeInstance[net]))) {
    activeInstance[net] = bridgeInstances[net][Math.max(0, removedIndex - 1)].id;
  }
  bridgeRefreshInstances(net);
  kgwBridgeSmallOwnerTraceR44D(net, "remove-instance", "r44d-owner-complete", { instanceId: String(instanceId || "") });
}


function renderAllNetworks(root) {
  const host = root.querySelector("#bridgeNetworkPanels");
  if (!host) return;
  host.innerHTML = BRIDGE_NETWORKS.map(renderNetworkPanel).join("");
  installSettingsLayout(root);


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
const KGW_BRIDGE_PORT_PROFILES_R35B = wasmBridgePortProfilesR35B();

/* KGW_BRIDGE_INSTANCE_EXTERNAL_PORT_RANGE_OWNER_R91
 * Existing Bridge port-profile owner refinement.
 * Instance stratum/prometheus port ranges are now derived from the current
 * bridge-level network settings outside the instance editor:
 * - stratum instances follow --stratum-port + 1 onward.
 * - prom instances follow --prom-port + 1 onward.
 * - each network remains isolated: mainnet, testnet10, testnet13.
 * - valid clearly manual out-of-range instance ports are preserved.
 */
function bridgeStaticPortProfileR91(net) {
  return wasmBridgeStaticPortProfileR91(net);
}







function bridgePortProfileR35B(net) {
  return wasmBridgePortProfileR35B(net);
}







function bridgeFindRecommendedOrNearestUnusedPortR35B(net, kind, usedPorts, fallbackStart) {
  return wasmBridgeFindRecommendedOrNearestUnusedPortR35B(net, kind, usedPorts, fallbackStart);
}


function bridgeTracePortProfileR35B(net, phase, details) {
  try {
    kgwBridgeSmallOwnerTraceR44D(net, "port-profile", phase, {
      patch: "R35B",
      owner: "bridge-network-port-profile-soft-policy",
      policy: "manual-valid-unused-ports-accepted-even-inside-other-network-range",
      details: details && typeof details === "object" ? details : {}
    });
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
}


/* KGW_BRIDGE_AUTOFIX_GLOBAL_USED_PORTS_PATCH_R45
 * Strengthens existing R37 Auto Fix:
 * - De-duplicates repeated conflict owners.
 * - Uses global used ports across all bridge networks when selecting replacements.
 * - If active network instance conflicts with another network instance, changes active network instance first.
 * - Applies multiple passes so a replacement cannot leave a new conflict behind.
 * Port policy remains soft: manual valid unused ports are accepted anywhere.
 */


function bridgeConfiguredPortRecordsR45() {
  let collected;
  try {
    collected = bridgeCollectConfiguredPortsR5();
  } catch (_) {
    collected = [];
  }
  return collected;
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
  return wasmBridgePlanPortAutofixR37(
    String(activeNet || ""),
    validation || {},
    bridgeInstances || {}
  );
}

function bridgeChooseReplacementPortR37(change, plannedUsed) {
  return wasmBridgeChooseReplacementPortR37(
    change || {},
    Array.from(plannedUsed || []),
    bridgeInstances || {},
    bridgeConfiguredPortRecordsR45() || []
  );
}

function bridgeWriteInstancePortR37(change, newPort) {
  const net = String(change && change.net || "");
  const instanceId = String(change && change.instanceId || "");
  const kind = String(change && change.kind || "");
  const normalizedPort = String(newPort || "").trim().replace(/^:/, "");
  const ok = wasmBridgeWriteInstancePortR37(
    bridgeInstances || {},
    net,
    instanceId,
    kind,
    normalizedPort
  );
  if (!ok) return false;

  const fieldName = kind === "prom" ? "instanceProm" : "instancePort";
  const field = byId(id(net, fieldName + "-" + instanceId));
  if (field) {
    field.value = normalizedPort;
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

function bridgeRefreshPortAutofixButtonsR37(_reason) {
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

  const translated = cleanTranslated(kgwI18nTextR41(i18nKey, fallbackText));
  if (translated) return translated;

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
  return kgwBridgeCommandOptionEnabledR7(net, "config") && Boolean(v(net, "config"));
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

  if (configMode) { kgwBridgeSyncDependencies(net); return; }

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
  kgwBridgeSyncDependencies(net);
}

function bridgeSyncAllModeControls() {
  BRIDGE_NETWORKS.forEach((item) => bridgeSyncModeControls(item.key));
}

function addRawValue(lines, flag, value) {
  const normalized = String(value || "").trim();
  if (normalized) lines.push(`${flag}=${normalized}`);
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

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

function kgwBridgeActiveRawLogInstanceIdV1(net) {
  bridgeEnsureInstanceState(net);
  return String(activeInstance?.[net] || (bridgeInstances?.[net]?.[0] && bridgeInstances[net][0].id) || "");
}

function kgwBridgeRenderRawLogBufferV1(net, role = "bridge", instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  return wasmBridgeRenderRawLogBuffer(String(net || ""), String(role || "bridge"), String(instanceId || ""));
}

function kgwBridgeApplyRuntimeLogReportV1(net, role, report, instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  return wasmBridgeApplyRuntimeLogReport(String(net || ""), String(role || "bridge"), report, String(instanceId || ""));
}

function kgwBridgeClearRawLogBufferV1(net, role = "bridge", instanceId = kgwBridgeActiveRawLogInstanceIdV1(net)) {
  return wasmBridgeClearRawLogBuffer(String(net || ""), String(role || "bridge"), String(instanceId || ""));
}
async function kgwBridgeDispatchRuntimeLogClearV1(net, _role = "bridge") {
  const invoke = getTauriInvoke();
  if (!invoke) return null;
  return await invokeWithTimeout(invoke, "kgw_kgw_runtime_clear_logs_v1", buildApplyPayload(net, "kgw_kgw_runtime_clear_logs_v1"), KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS);
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
  const configValue = bridgeHasConfig(net) ? v(net, "config") : "";

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

function kgwBridgeEffectiveNodeInteger(net, name, fallback) {
  if (!bridgeFieldEnabled(name, kgwBridgeForm(net), kgwBridgeCommandInlineStateR7(net))) return fallback;
  const raw = v(net, name);
  if (!raw) return fallback;
  const value = Number(raw);
  if (!Number.isInteger(value)) throw new Error(`${name} must be an integer.`);
  return value;
}

function kgwBridgeEffectiveNodeNumber(net, name, fallback) {
  if (!kgwBridgeCommandShouldIncludeR7(net, name)) return fallback;
  const raw = v(net, name);
  if (!raw) return fallback;
  const value = Number(raw);
  if (!Number.isFinite(value)) throw new Error(`${name} must be a finite number.`);
  return value;
}

function kgwBridgeEffectiveInprocessNodeSettings(net) {
  if (bridgeNodeMode(net) !== "inprocess") return null;
  const errors = kgwBridgeValidateForm(net);
  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
  const profile = bridgeProfile(net);
  if (v(net, "inprocessConfigfile")) {
    throw new Error("In-process --configfile is unsupported because the desktop owns network and database isolation.");
  }
  if (c(net, "inprocessDevnet") || c(net, "inprocessSimnet")) {
    throw new Error("Devnet and simnet cannot override the selected desktop network tab.");
  }
  if (v(net, "inprocessOverrideParamsFile")) {
    throw new Error("In-process --override-params-file is unsupported because the desktop owns the selected network identity.");
  }
  const rpcListen = v(net, "inprocessRpcListen") || `127.0.0.1:${profile.kaspadPort}`;
  const addPeer = kgwBridgeCommandShouldIncludeR7(net, "inprocessAddPeer") ? v(net, "inprocessAddPeer") : "";
  const connectPeer = kgwBridgeCommandShouldIncludeR7(net, "inprocessConnect") ? v(net, "inprocessConnect") : "";

  return {
    logLevel: kgwBridgeCommandShouldIncludeR7(net, "inprocessLogLevel") ? v(net, "inprocessLogLevel") || "info" : "info",
    asyncThreads: kgwBridgeEffectiveNodeInteger(net, "inprocessAsyncThreads", 16),
    ramScale: kgwBridgeEffectiveNodeNumber(net, "inprocessRamScale", 1),
    yes: c(net, "inprocessYes"),
    noLogFiles: true,
    sanity: false,
    enableUnsyncedMining: c(net, "inprocessEnableUnsyncedMining") && Boolean(profile?.testnet),
    p2pListen: kgwBridgeCommandShouldIncludeR7(net, "inprocessListen") ? v(net, "inprocessListen") || null : null,
    externalIp: null,
    disableUpnp: c(net, "inprocessDisableUpnp"),
    disableDnsSeeding: false,
    userAgentComments: [],
    rpcListen,
    rpcListenBorsh: kgwBridgeCommandShouldIncludeR7(net, "inprocessRpcListenBorsh") ? v(net, "inprocessRpcListenBorsh") || null : null,
    rpcListenJson: kgwBridgeCommandShouldIncludeR7(net, "inprocessRpcListenJson") ? v(net, "inprocessRpcListenJson") || null : null,
    rpcMaxClients: 16,
    unsafeRpc: c(net, "inprocessUnsafeRpc"),
    disableGrpc: false,
    connectPeers: connectPeer ? [connectPeer] : [],
    addPeers: addPeer ? [addPeer] : [],
    outboundTarget: kgwBridgeEffectiveNodeInteger(net, "inprocessOutpeers", 8),
    inboundLimit: kgwBridgeEffectiveNodeInteger(net, "inprocessMaxInpeers", 32),
    utxoIndex: c(net, "inprocessUtxoIndex"),
    archival: c(net, "inprocessArchival"),
    resetDb: false,
    perfMetrics: c(net, "inprocessPerfMetrics"),
    maxTrackedAddresses: 0,
    retentionPeriodDays: null,
    perfMetricsIntervalSec: kgwBridgeEffectiveNodeInteger(net, "inprocessPerfMetricsIntervalSec", 10),
    rocksDbPreset: null,
    rocksDbCacheSize: null,
    rocksDbWalDir: null,
    overrideParamsFile: null,
    logDir: null,
  };
}





const KGW_BRIDGE_PREVIEW_REQUESTS = new Map();
function kgwBridgePreviewMessage(net, message, error = false) {
  const status = byId(id(net, "previewStatus"));
  if (status) {
    status.textContent = message;
    status.classList.toggle("kgw-field-error", error);
    applyStatusTone(status, error ? "error" : message.startsWith("Validating") ? "validating" : "verified");
  }
}
async function kgwBridgePreparePreview(net, payload) {
  const invoke = getTauriInvoke();
  if (!invoke) throw new Error("The desktop runtime is unavailable.");
  return invokeWithTimeout(invoke, "kgw_runtime_settings_preview_v1", payload, 10000);
}
function updateCommand(net) {
  const preview = byId(id(net, "commandPreview"));
  if (!preview) return "";
  const previous = KGW_BRIDGE_PREVIEW_REQUESTS.get(net);
  if (previous?.timer) window.clearTimeout(previous.timer);
  const request = {sequence: (previous?.sequence || 0) + 1, timer: null};
  KGW_BRIDGE_PREVIEW_REQUESTS.set(net, request);
  preview.value = "";
  delete preview.dataset.effectiveSettings;
  try {
    bridgeSyncModeControls(net);
    bridgeReassignInstancePortsFromExternalRangeR91(net, "update-command");
    bridgeSyncInstancePreviewRowsR8B(net);
    const errors = kgwBridgeValidateForm(net);
    if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
    const payload = buildApplyPayload(net, "kgw_kgw_apply_node_settings_v1");
    kgwBridgePreviewMessage(net, "Validating effective settings...");
    request.timer = window.setTimeout(async () => {
      try {
        const result = await kgwBridgePreparePreview(net, payload);
        if (KGW_BRIDGE_PREVIEW_REQUESTS.get(net) !== request) return;
        preview.value = JSON.stringify(result, null, 2);
        preview.dataset.effectiveSettings = JSON.stringify(result);
        bridgeSyncInstancePreviewRowsR8B(net);
        preview.dataset.kgwBridgeCommandOwner = "typed-effective-settings-preview";
        preview.dataset.kgwBridgeNetwork = net;
        preview.classList.remove("bridge-v7-command-warning");
        for (const name of ["appdir", "inprocessAppdirMirror"]) {
          const field = byId(id(net, name)); if (field) { field.value = result.appDir; field.title = result.appDir; }
        }
        kgwBridgePreviewMessage(net, "Validated by the same settings resolver used by Start. Embedded libraries; no external executable.");
      } catch (error) {
        if (KGW_BRIDGE_PREVIEW_REQUESTS.get(net) !== request) return;
        preview.value = "";
        kgwBridgePreviewMessage(net, normalizeRuntimeError(error), true);
      }
    }, 180);
    return payload.bridgeCommandPreview;
  } catch (error) {
    kgwBridgePreviewMessage(net, normalizeRuntimeError(error), true);
    return "";
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
}
/* KGW_BRIDGE_LAST_NETWORK_RESTORE_R101W2 */
const KGW_BRIDGE_LAST_NETWORK_KEY_R101W2 = "kgw.bridge.lastNetwork";
function kgwBridgeNormalizeNetworkR101W2(value) {
  const normalized = String(value || "").trim();
  return normalized === "mainnet" || normalized === "testnet10" || normalized === "testnet13" ? normalized : "";
}
function kgwBridgeReadLastNetworkR101W2() {
  try { return kgwBridgeNormalizeNetworkR101W2(localStorage.getItem(KGW_BRIDGE_LAST_NETWORK_KEY_R101W2)); } catch (_) { return ""; }
}
function kgwBridgeSaveLastNetworkR101W2(net) {
  const normalized = kgwBridgeNormalizeNetworkR101W2(net);
  if (!normalized) return "";
  try { localStorage.setItem(KGW_BRIDGE_LAST_NETWORK_KEY_R101W2, normalized); } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
const KGW_BRIDGE_STOP_INVOKE_TIMEOUT_MS = 0;
const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();

function getTauriInvoke() {
  const tauri = window.__TAURI__;
  return tauri?.core?.invoke || tauri?.invoke || window.__TAURI_INVOKE__ || null;
}

/* KGW_BRIDGE_START_TRACE_V1 is Rust-owned in bridge_start_trace.rs. */

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


function kgwBridgeStartOptions(net) {
  const enabled = net !== "mainnet" && c(net, "internalCpuMiner");
  return {
    configFile: bridgeHasConfig(net) ? v(net, "config") : null,
    internalCpuMiner: enabled ? {
      enabled: true, address: v(net, "internalCpuMinerAddress"),
      threads: kgwBridgeParseUnsignedV1("CPU threads", v(net, "internalCpuMinerThreads"), 1, 256),
      throttleMs: kgwBridgeParseUnsignedV1("CPU throttle", v(net, "internalCpuMinerThrottleMs"), null, 60000),
      templatePollMs: kgwBridgeParseUnsignedV1("Template poll interval", v(net, "internalCpuMinerTemplatePollMs"), null, 60000),
    } : { enabled: false },
  };
}

function buildApplyPayload(net, command) {
  if (command === "kgw_kgw_apply_node_settings_v1") {
    bridgeAssertNoPortConflictsR5(net);

    const preview = buildCommandLines(net).join(" ");
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
      effectiveNodeSettings: kgwBridgeEffectiveInprocessNodeSettings(net),
      effectiveBridgeSettings: kgwBridgeEffectiveSettingsV1(net, structuredInstances),
      bridgeOptions: kgwBridgeStartOptions(net),
      experimentalNetworkOptIn: net === "testnet13" && kgwBridgeNetworkEnabled(net),
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

async function invokeBridgeIntegratedRuntime(command, net) {
  const invoke = getTauriInvoke();
  if (!invoke) {
    throw new Error("Tauri invoke is unavailable in this window.");
  }

  const timeoutMs = command === "kgw_kgw_disable_network_v1"
    ? KGW_BRIDGE_STOP_INVOKE_TIMEOUT_MS
    : KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS;
  const payload = buildApplyPayload(net, command);
  if (command === "kgw_kgw_apply_node_settings_v1") {
    const errors = kgwBridgeValidateForm(net, true);
    if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
    await kgwBridgePreparePreview(net, payload);
  }
  return await invokeWithTimeout(invoke, command, payload, timeoutMs);
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

function kgwBridgeSetRuntimeErrorV1(net, errorText = "", errorSource = "") {
  const errorNode = byId(id(net, "runtimeError"));
  if (!errorNode) return;
  const text = String(errorText || "").trim();
  errorNode.textContent = text;
  errorNode.hidden = !text;
  errorNode.dataset.runtimeErrorSource = errorSource;
  applyStatusTone(errorNode, "error");
}

function kgwBridgeSetRuntimeActivityV1(net, message = "", state = "") {
  const statusNode = byId(id(net, "runtimeStatus"));
  if (!statusNode) return;
  statusNode.textContent = String(message || "").trim();
  applyStatusTone(statusNode, state || byId(id(net, "policyStatus"))?.dataset.state);
}

function kgwBridgeMarkRestartRequiredV1(net) {
  const authority = byId(id(net, "settingsAuthority"));
  if (!authority) return;
  const status = byId(id(net, "runtimeStatus"));
  const running = /running/i.test(String(status?.textContent || ""));
  authority.textContent = running
    ? "Restart required to apply changed effective settings"
    : "Effective settings apply on next Start";
  authority.dataset.restartRequired = running ? "true" : "false";
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
}


// KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_MAINNET_IMMEDIATE_R65F
function kgwBridgeNormalizeNodeModeR65F(value) {
  return String(value || "").toLowerCase().replace(/[^a-z0-9]+/g, "");
}

function kgwBridgeCurrentNodeModeFromUiR65F(net) {
  try {
    const direct = byId(id(net, "nodeMode"));
    if (direct && "value" in direct) return String(direct.value || "");
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

  try {
    const panel = document.querySelector('[data-bridge-panel="' + String(net || "") + '"]') ||
      document.querySelector('[data-net="' + String(net || "") + '"]');
    if (panel) {
      const select = panel.querySelector('[id$="-nodeMode"], [data-bridge-setting="nodeMode"], select[name="nodeMode"]');
      if (select && "value" in select) return String(select.value || "");
    }
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
    if (Object.keys(kgwBridgeValidateForm(net, true)).length) return true;
    if ((c(net, "internalCpuMiner") || c(net, "inprocessUnsafeRpc") || c(net, "inprocessEnableUnsyncedMining")) &&
        !await confirmUserAction("Start " + net + " with the selected advanced risk settings?\n\nUnsafe RPC exposes RPC beyond loopback. Unsynced mining bypasses synchronization. CPU mining uses additional CPU resources.")) return true;
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
  kgwBridgeR51SetRuntimeButtons(net, action === "stop", action === "start" ? "starting" : "stopping");

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
        kgwBridgeSetRuntimeActivityV1(net, "Bridge READY attestation confirmed.", "ready");
        kgwBridgeR51KickRawLogLiveR134E(net, "bridge-start-confirmed");
      } else if (blocked) {
        kgwBridgeR51SetRuntimeButtons(net, false);
        kgwBridgeSetRuntimeErrorV1(net, raw);
        kgwBridgeSetRuntimeActivityV1(net, "Bridge start failed.", "failed");
      } else {
        kgwBridgeR51SetRuntimeButtons(net, false);
        kgwBridgeSetRuntimeErrorV1(net, "Backend Start did not provide READY attestation: " + raw);
        kgwBridgeSetRuntimeActivityV1(net, "Bridge start was not confirmed by READY attestation.", "warning");
      }
    }

    if (action === "stop") {
      const confirmedStopped =
        fields.running === "false" &&
        (fields.graceful === "true" || fields.forced === "true" || fields.stop_failed === "true" || fields.already_stopped === "true");

      kgwBridgeRuntimeOwnerTraceR64D("r64d-stop-confirmation-evaluated", {
        confirmedStopped: Boolean(confirmedStopped)
      });

      if (confirmedStopped) {
        const forced = fields.forced === "true";
        const stopFailed = fields.stop_failed === "true";
        KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);
        kgwBridgeR51SetRuntimeButtons(net, false);
        kgwBridgeSetRuntimeErrorV1(
          net,
          forced
            ? "Stop required FORCED termination. " + String(fields.reason || raw)
            : stopFailed
              ? "Official graceful shutdown failed, but the worker process exited. " + String(fields.reason || raw)
              : ""
        );
        kgwSetBridgeOwnedNodeLockR65E(net, false, {
          source: "bridge-stop-confirmed",
          action: "stop"
        });
        kgwBridgeSetRuntimeActivityV1(
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

    kgwBridgeR51SetRuntimeUnknown(net, message);
    kgwBridgeSetRuntimeActivityV1(net, "Bridge " + action + " failed; reconciling runtime state.");

    return true;
  } finally {
    KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);
    window.setTimeout(() => { if (typeof kgwBridgeR51RefreshOne === "function") void kgwBridgeR51RefreshOne(net, "action-settled"); }, 0);

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
const KGW_BRIDGE_R51_STATUS_IN_FLIGHT = new Map();
const KGW_BRIDGE_R51_LOGS_IN_FLIGHT = new Map();
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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
  return state;
}

function kgwBridgeR51ReadInstanceCommandOptionsR38C(net) {
  const state = {};
  try {
    const root = document.getElementById("kaspa-bridge");
    if (!root) return state;

    for (const item of root.querySelectorAll('[data-bridge-instance-command-option-toggle-r13b][data-net="' + String(net || "") + '"]')) {
      const instanceId = String(item.dataset.instanceId || "");
      const name = String(item.dataset.bridgeInstanceCommandOptionToggleR13b || "");
      if (!instanceId || !name) continue;
      state[instanceId] = state[instanceId] || {};
      state[instanceId][name] = Boolean(item.checked);
    }
  } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
  return state;
}

function kgwBridgeR51ApplyCommandOptionsR38C(net, values) {
  try {
    const commandOptions = values && values[KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C];
    if (commandOptions && typeof commandOptions === "object") {
      for (const [name, enabled] of Object.entries(commandOptions)) {
        wasmBridgeCommandSetOptionR7(
          String(net || ""),
          String(name || ""),
          Boolean(enabled) && (!BRIDGE_OPTIONAL.has(name) || Boolean(String(values[id(net, name)]?.value || "").trim()))
        );
      }
      kgwBridgeRefreshInlineCommandTogglesR7(net);
    }

    const instanceOptions = values && values[KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C];
    if (instanceOptions && typeof instanceOptions === "object") {
      for (const [instanceId, options] of Object.entries(instanceOptions)) {
        if (!options || typeof options !== "object") continue;
        for (const [name, enabled] of Object.entries(options)) {
          const optional = ["instanceBlockWaitTime", "instanceExtranonceSize", "instanceSharesPerMin"].includes(name);
          const record = (bridgeInstances[net] || []).find(item => String(item.id) === String(instanceId));
          kgwBridgeSetInstanceCommandOptionR13B(net, instanceId, name, Boolean(enabled) && (!optional || Boolean(String(record?.[name] || "").trim())));
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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

    return true;
  } catch (error) {
    try {
      kgwBridgeSmallOwnerTraceR44D(net, "settings-persistence", "r26b-apply-structured-instances-failed", {
        message: error && error.message ? error.message : String(error)
      });
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
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
    if (field.closest(".bridge-v7-log-toolbar")) return false;
    return true;
  });
}


/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_START */



/* KGW_BRIDGE_SETTINGS_LIFECYCLE_FIX_R6_END */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */


/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

function kgwBridgeR51ReadSettings(net) {
  const values = {};

  const structuredInstances = kgwBridgeR51ReadStructuredInstancesR26B(net);
  values[KGW_BRIDGE_R51_STRUCTURED_INSTANCES_KEY_R26B] = structuredInstances;
  values[KGW_BRIDGE_R51_ACTIVE_INSTANCE_KEY_R26B] = structuredInstances.activeInstance;
  values[KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C] = kgwBridgeR51ReadCommandOptionsR38C(net);
  values[KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C] = kgwBridgeR51ReadInstanceCommandOptionsR38C(net);

  for (const field of kgwBridgeR51Fields(net)) {
    if (!field.id || BRIDGE_MANAGED[field.id.slice(("bridge-" + net + "-").length)]) continue;

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
 * - testnet13 was replayed as :5557 / :2114
 *
 * Correct bridge-level network ranges already exist in KGW_BRIDGE_PORT_PROFILES_R35B:
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
    testnet13: {
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

    const name = field.id.slice(("bridge-" + net + "-").length);
    if (BRIDGE_MANAGED[name]) continue;
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
  kgwBridgeRequireValidSettings(net);
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
  kgwBridgeRequireValidSettings(net);
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

/* R9B compatibility boundary: current input/change owners identify programmatic writes via Event.isTrusted. */
function kgwBridgeSettingsWithProgrammaticWriteR9B(callback) {
  return callback();
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

function kgwBridgeRuntimeErrorFromStatus(text) {
  const fields = parseRuntimeKeyValueResponse(text).fields || {};
  const error = String(fields.runtime_error || fields.runtimeError || "").trim();
  return error && error.toLowerCase() !== "none" ? error : "";
}

function kgwBridgeR51SetRuntimeButtons(net, running, transition = "", runtimeError = "", statusText = "") {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return;

  const networkEnabled = kgwBridgeNetworkEnabled(net);
  const start = panel.querySelector(`[data-bridge-action="start"][data-net="${net}"]`);
  const stop = panel.querySelector(`[data-bridge-action="stop"][data-net="${net}"]`);
  const policyStatus = byId(id(net, "policyStatus"));

  const presentation = runtimePresentation({role: "Bridge", enabled: networkEnabled, running, transition, error: runtimeError});
  if (policyStatus) {
    policyStatus.textContent = presentation.processLabel;
    policyStatus.dataset.state = presentation.process.toLowerCase();
    applyStatusTone(policyStatus, presentation.process);
  }
  const summary = byId(id(net, "monitorState"));
  renderStatusSummary(summary, presentation.processLabel + " | Profile: " + presentation.profile +
    " | Startup readiness: " + (running ? "Verified" : transition ? "Pending" : "Not ready") +
    " | " + runtimeObservationSummary((parseRuntimeKeyValueResponse(statusText).fields || {}), running, net !== "mainnet"));
  const empty = byId(id(net, "logEmpty"));
  if (empty) empty.textContent = runtimeError ? "Bridge failed. Review the error in Settings and the logs below."
    : transition ? "Bridge is " + presentation.process.toLowerCase() + ". Waiting for runtime output."
    : running ? "Bridge is running. Waiting for log output."
    : bridgeNodeMode(net) === "inprocess" ? "Bridge is stopped. Start Bridge to start its owned node and Stratum service."
    : "Bridge is stopped. Start Bridge checks the configured node connection before starting the service.";
  const settingsInvalid = Boolean(panel.querySelector('[aria-invalid="true"]') ||
    byId(id(net, "previewStatus"))?.classList.contains("kgw-field-error"));
  const next = panel.querySelector('[data-bridge-action="monitor-next"]');
  if (next) {
    next.hidden = running || Boolean(transition);
    next.textContent = !networkEnabled ? "Enable Profile in Settings" : settingsInvalid ? "Review Settings" : "Start Bridge";
    next.dataset.nextAction = networkEnabled && !settingsInvalid ? "start" : "settings";
  }

  if (start) {
    const startBlocked = Boolean(running || transition === "starting" || transition === "stopping" || !networkEnabled || settingsInvalid);
    start.disabled = startBlocked;
    start.setAttribute("aria-disabled", String(startBlocked));
    start.style.opacity = startBlocked ? "0.45" : "";
    start.style.cursor = startBlocked ? "not-allowed" : "";
    start.title = !networkEnabled
      ? "Enable this network before starting it."
      : running
        ? "Bridge is running. Stop it before starting again."
        : "Start bridge";
  }

  if (stop) {
    const stopEnabled = Boolean(running && transition !== "starting" && transition !== "stopping");
    stop.disabled = !stopEnabled;
    stop.setAttribute("aria-disabled", String(!stopEnabled));
    stop.style.opacity = stopEnabled ? "" : "0.45";
    stop.style.cursor = stopEnabled ? "" : "not-allowed";
    stop.title = transition === "starting"
      ? "Bridge startup is in progress. Stop becomes available after READY."
      : stopEnabled
        ? "Stop bridge"
        : "Bridge is not running";
  }
}

function kgwBridgeR51SetRuntimeUnknown(net, message = "Runtime status is temporarily unavailable. Reconciling with the backend.", errorSource = "") {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return;
  const policyStatus = byId(id(net, "policyStatus"));
  if (policyStatus) {
    policyStatus.textContent = "Reconciling";
    policyStatus.dataset.state = "reconciling";
    applyStatusTone(policyStatus, "reconciling");
    const summary = byId(id(net, "monitorState"));
    renderStatusSummary(summary, "Bridge: Reconciling | RPC/synchronization/mining: unknown");
  }
  const start = panel.querySelector(`[data-bridge-action="start"][data-net="${net}"]`);
  const stop = panel.querySelector(`[data-bridge-action="stop"][data-net="${net}"]`);
  for (const button of [start, stop]) {
    if (!button) continue;
    button.disabled = true;
    button.setAttribute?.("aria-disabled", "true");
    button.style.opacity = "0.45";
    button.style.cursor = "not-allowed";
    button.title = message;
  }
  kgwBridgeSetRuntimeActivityV1(net, "Reconciling runtime state.");
  const currentError = byId(id(net, "runtimeError"));
  const preserveActionError = errorSource === "status-refresh" && currentError?.textContent?.trim() &&
    currentError.dataset.runtimeErrorSource !== "status-refresh";
  if (!preserveActionError) kgwBridgeSetRuntimeErrorV1(net, message, errorSource);
}

function kgwBridgeR51MaybeActivityNotice(net, statusText) {
  const now = Date.now();
  const last = KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE[net] || 0;

  if (now - last < 15000) return;

  if (!kgwBridgeR51IsRunning(statusText)) return;

  KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE[net] = now;

}

async function kgwBridgeR51RefreshOne(net, _reason = "live") {
  const transitionActive = KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(net + ":start") ||
    KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(net + ":stop");

  let logsTask = KGW_BRIDGE_R51_LOGS_IN_FLIGHT.get(net);
  if (!logsTask) {
    logsTask = (async () => {
      try {
        const report = await invokeBridgeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net);
        const instanceId = kgwBridgeActiveRawLogInstanceIdV1(net);
        kgwBridgeApplyRuntimeLogReportV1(net, "bridge", report, instanceId);
        KGW_BRIDGE_R51_LAST_LOGS[net] = report;
      } catch (_) {
        // No raw buffer may exist before the child is spawned. Never fabricate text.
      } finally {
        if (KGW_BRIDGE_R51_LOGS_IN_FLIGHT.get(net) === logsTask) {
          KGW_BRIDGE_R51_LOGS_IN_FLIGHT.delete(net);
        }
      }
    })();
    KGW_BRIDGE_R51_LOGS_IN_FLIGHT.set(net, logsTask);
  }

  let statusTask = Promise.resolve();
  if (!transitionActive) {
    statusTask = KGW_BRIDGE_R51_STATUS_IN_FLIGHT.get(net);
    if (!statusTask) {
      statusTask = (async () => {
        try {
          const status = stringifyRuntimeResult(await invokeBridgeIntegratedRuntime("kgw_runtime_owner_status_v1", net));
          const running = kgwBridgeR51IsRunning(status);
          const runtimeError = kgwBridgeRuntimeErrorFromStatus(status);
          const statusFields = parseRuntimeKeyValueResponse(status).fields || {};
          const errorNode = byId(id(net, "runtimeError"));
          // Clear only recovered polling feedback; preserve Start/Stop and runtime errors.
          if (!runtimeError && statusFields.role === "bridge" && statusFields.network === String(net) &&
              ["true", "false"].includes(statusFields.running) &&
              errorNode?.dataset.runtimeErrorSource === "status-refresh") {
            kgwBridgeSetRuntimeErrorV1(net, "");
            kgwBridgeSetRuntimeActivityV1(net, running ? "Bridge is running." : "Bridge is stopped.", running ? "running" : "stopped");
          }
          kgwBridgeR51SetRuntimeButtons(net, running, "", runtimeError, status);
          if (!running && runtimeError) {
            kgwBridgeSetRuntimeErrorV1(net, runtimeError);
            kgwBridgeSetRuntimeActivityV1(net, "Bridge runtime failed after readiness.", "failed");
            const policyStatus = byId(id(net, "policyStatus"));
            if (policyStatus) {
              policyStatus.textContent = kgwBridgeTranslateRuntime("runtime.failed", "Failed");
              policyStatus.dataset.state = "failed";
              applyStatusTone(policyStatus, "failed");
            }
          }

          if (KGW_BRIDGE_R51_LAST_STATUS[net] !== status) {
            KGW_BRIDGE_R51_LAST_STATUS[net] = status;
            const authority = byId(id(net, "settingsAuthority"));
            if (authority && (!running || authority.dataset.restartRequired !== "true")) {
              authority.textContent = running
                ? "Effective settings are active for this runtime"
                : "Effective settings apply on next Start";
              authority.dataset.restartRequired = "false";
            }
          }
          kgwBridgeR51MaybeActivityNotice(net, status);
        } catch (error) {
          kgwBridgeR51SetRuntimeUnknown(net, "Status refresh failed: " + normalizeRuntimeError(error), "status-refresh");
        } finally {
          if (KGW_BRIDGE_R51_STATUS_IN_FLIGHT.get(net) === statusTask) {
            KGW_BRIDGE_R51_STATUS_IN_FLIGHT.delete(net);
          }
        }
      })();
      KGW_BRIDGE_R51_STATUS_IN_FLIGHT.set(net, statusTask);
    }
  }

  await Promise.allSettled([logsTask, statusTask]);
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



/* KGW_BRIDGE_ACTION_AND_LOG_FEEDBACK_OWNER_V1 */


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
    } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }
  }
  return fallback || key;
}


async function kgwBridgeHandleLogActionV29(action, net, button) {
  return await wasmBridgeHandleLogAction(String(action || ""), String(net || ""), button, {
    smallOwnerTrace: kgwBridgeSmallOwnerTraceR44D,
    activeRawLogInstanceId: kgwBridgeActiveRawLogInstanceIdV1,
    translateRuntime: kgwBridgeTranslateRuntimeV29,
    clearRawLogBuffer: kgwBridgeClearRawLogBufferV1,
    dispatchRuntimeLogClear: kgwBridgeDispatchRuntimeLogClearV1
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
        wasmBridgeCommandSetOptionR7(String(net || ""), String(option || ""), enabled);
        updateCommand(net);
        if (typeof kgwBridgeRefreshInlineCommandTogglesR7 === "function") {
          kgwBridgeRefreshInlineCommandTogglesR7(net);
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
    kgwBridgeMarkRestartRequiredV1(net);
    scopedUpdate(net, event.isTrusted ? "trusted-input" : "programmatic-input");
  }, true);

  root.addEventListener("change", async (event) => {
    const target = event.target;
    if (!target || !target.matches || !target.matches("input, select, textarea")) return;
    if (target.readOnly || target.id.endsWith("-commandPreview") || target.id.endsWith("-logOutput")) return;

    const net = netFromEvent(event);
    kgwBridgeMarkRestartRequiredV1(net);

    if (target.matches("[data-bridge-network-enabled]")) {
      const profile = kgwBridgeNetworkProfile(net);
      const wasEnabled = kgwBridgeNetworkEnabled(net);
      let enabled = Boolean(target.checked);

      if (enabled && profile?.experimental) {
        target.checked = false;
        target.disabled = true;
        try {
          enabled = (await confirmUserAction("Testnet 13 is experimental and uses a separate non-production runtime. Enable it only for isolated testing. Continue?")) === true;
        } catch (error) {
          enabled = false;
          kgwBridgeSetRuntimeErrorV1(net, normalizeRuntimeError(error));
        } finally { target.disabled = false; target.checked = enabled; }
      }

      kgwBridgeSetNetworkEnabled(net, enabled);
      kgwBridgeR51SetRuntimeButtons(net, false);

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
      try {
        kgwBridgeR51SaveSettings(net);
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        kgwBridgeSetRuntimeErrorV1(net, normalizeRuntimeError(error));
      }
      scopedUpdate(net, "save-settings");
      return;
    }

    if (action === "set-defaults") {
      try {
        kgwBridgeR51SetAsDefaults(net);
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        kgwBridgeSetRuntimeErrorV1(net, normalizeRuntimeError(error));
      }
      scopedUpdate(net, "set-defaults");
      return;
    }

    if (action === "restore-defaults") {
      try {
        kgwBridgeR51RestoreDefaults(net);
        button.dataset.kgwSettingsActionResult = "success";
      } catch (error) {
        button.dataset.kgwSettingsActionResult = "failed";
        kgwBridgeSetRuntimeErrorV1(net, normalizeRuntimeError(error));
      }
      scopedUpdate(net, "restore-defaults");
      return;
    }

    if (action === "copy-log" || action === "clear-log") {
      kgwBridgeHandleLogActionV29(action, net, button).catch(function () {});
      return;
    }

    if (action === "monitor-next") {
      const panel = kgwBridgeR51Panel(net);
      if (button.dataset.nextAction === "start") panel?.querySelector('[data-bridge-action="start"]')?.click();
      else panel?.querySelector('[data-bridge-inner-tab="settings"]')?.click();
      return;
    }
    if (action === "copy-command" || action === "copy-path") {
      void (async () => {
        let text = v(net, "appdir");
        if (action === "copy-command") {
          const errors = kgwBridgeValidateForm(net);
          if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
          const request = KGW_BRIDGE_PREVIEW_REQUESTS.get(net);
          const result = await kgwBridgePreparePreview(net, buildApplyPayload(net, "kgw_kgw_apply_node_settings_v1"));
          if (KGW_BRIDGE_PREVIEW_REQUESTS.get(net) !== request) throw new Error("Settings changed while copying. Try again.");
          text = JSON.stringify(result, null, 2);
        }
        if (!text) throw new Error("There is no validated value to copy.");
        await wasmBridgeDispatchClipboardWrite(String(net || ""), String(text ?? ""), {characterCount: [...text].length, lineCount: text.split(/\r?\n/).length});
        kgwBridgePreviewMessage(net, action === "copy-path" ? "Data directory copied." : "Effective settings copied.");
      })().catch(error => kgwBridgePreviewMessage(net, "Copy failed: " + normalizeRuntimeError(error), true));
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
try { kgwBridgeAutofixButtonInitialLabelR111G(document); } catch (_) { /* Best-effort secondary operation; primary bridge behavior is preserved. */ }

function kgwBridgeForm(net) {
  const values = { network: net };
  kgwBridgeR51Panel(net)?.querySelectorAll(".bridge-v7-card input[id], .bridge-v7-card select[id]").forEach(field => {
    values[field.id.slice(("bridge-" + net + "-").length)] = field.type === "checkbox" ? field.checked : field.value;
  });
  return values;
}
function kgwBridgeValidateForm(net, focus = false) {
  const errors = validateBridgeForm(kgwBridgeForm(net), kgwBridgeCommandInlineStateR7(net), net);
  const panel = kgwBridgeR51Panel(net);
  for (const instance of net !== "mainnet" || bridgeHasConfig(net) ? [] : bridgeInstances[net] || []) {
    if (!kgwBridgeInstanceCommandShouldIncludeR13B(net, instance.id, "instance")) continue;
    const waitField = byId(id(net, "instanceBlockWaitTime-" + instance.id));
    if (waitField && kgwBridgeInstanceCommandShouldIncludeR13B(net, instance.id, "instanceBlockWaitTime") &&
        !/^[1-9]\d*(ms|s)?$/.test(waitField.value.trim()))
      errors["instanceBlockWaitTime-" + instance.id] = "Enter a positive duration, for example 50ms or 1s.";
    for (const [name, min, max] of [["instanceDiff",1,4294967295],["instanceExtranonceSize",0,8],["instanceSharesPerMin",1,4294967295]]) {
      if (!kgwBridgeInstanceCommandShouldIncludeR13B(net, instance.id, name)) continue;
      const field = byId(id(net, name + "-" + instance.id));
      if (!field) continue;
      const raw = String(field.value || "").trim();

      if (!/^\d+$/.test(raw) || !Number.isSafeInteger(Number(raw)) || Number(raw) < min || Number(raw) > max)
        errors[field.id.slice(("bridge-" + net + "-").length)] = "Enter a whole number from " + min + " to " + max + ".";
    }
  }
  renderFieldErrors(panel, "bridge-" + net + "-", errors);
  if (focus && Object.keys(errors).length) {
    const field = byId(id(net, Object.keys(errors)[0]));
    panel?.querySelector('[data-bridge-inner-tab="settings"]')?.click();
    const section = field?.closest("[data-bridge-section-panel]");
    panel?.querySelector('[data-bridge-section-tab="' + section?.dataset.bridgeSectionPanel + '"]')?.click();
    revealSettingsField(field);
    field?.focus();
  }
  return errors;
}
function kgwBridgeRequireValidSettings(net) {
  const errors = kgwBridgeValidateForm(net, true);
  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
  bridgeAssertNoPortConflictsR5(net);
  kgwBridgeEffectiveInprocessNodeSettings(net);
  kgwBridgeEffectiveSettingsV1(net, kgwBridgeR51ReadStructuredInstancesR26B(net));
}
function kgwBridgeSyncDependencies(net) {
  const values = kgwBridgeForm(net), options = kgwBridgeCommandInlineStateR7(net);
  const panel = kgwBridgeR51Panel(net);
  for (const name of Object.keys(values)) {
    const field = byId(id(net, name)); if (!field) continue;
    if (/^instance/.test(name)) continue;
    const managed = BRIDGE_MANAGED[name];
    const forbidden = net === "mainnet" && ["internalCpuMiner", "inprocessEnableUnsyncedMining"].includes(name);
    const active = bridgeFieldEnabled(name, values, options) && !forbidden;
    field.disabled = !active && !["appdir", "inprocessAppdirMirror"].includes(name);
    field.readOnly = Boolean(managed);
    field.title = managed || (forbidden ? "Test networks only." : !active ? "Enable the parent option to use this value." : field.value || "");
    const card = field.closest(".bridge-v7-card");
    card?.classList.toggle("kgw-field-inactive", !active);
    if (card) { card.title = field.title; card.removeAttribute("data-i18n-title"); }
    const state = managed ? (/unsupported/i.test(managed) ? "Unsupported" : "Managed")
      : forbidden ? "Test networks only"
      : !active ? "Not active" : "";
    setSettingFieldState(field, state);
    if (field.tagName === "INPUT" || field.tagName === "SELECT") {
      const label = card?.querySelector(".kgw-command-option-title-text-r8e, span");
      if (label) { label.id = field.id + "-label"; field.setAttribute("aria-labelledby", label.id); }
    }
  }
  panel?.querySelectorAll("[data-bridge-command-option-toggle-r7]").forEach(toggle => {
    const name = toggle.dataset.bridgeCommandOptionToggleR7;
    toggle.disabled = Boolean(BRIDGE_MANAGED[name] ||
      (name.startsWith("inprocess") && values.nodeMode !== "inprocess") ||
      (name.startsWith("internalCpuMiner") && name !== "internalCpuMiner" && !values.internalCpuMiner) ||
      (name === "inprocessPerfMetricsIntervalSec" && !values.inprocessPerfMetrics));
  });
  panel?.querySelectorAll("[data-bridge-instance-command-option-toggle-r13b]").forEach(toggle => {
    const instanceId = toggle.dataset.instanceId, name = toggle.dataset.bridgeInstanceCommandOptionToggleR13b;
    if (!instanceId || !name) return;
    const field = byId(id(net, name + "-" + instanceId));
    if (name === "instanceLogToFile") {
      toggle.disabled = true;
      if (field) { field.disabled = true; field.title = BRIDGE_MANAGED.logToFile; }
      return;
    }
    const parentActive = !bridgeHasConfig(net) && kgwBridgeInstanceCommandShouldIncludeR13B(net, instanceId, "instance");
    toggle.disabled = bridgeHasConfig(net) || (name !== "instance" && !parentActive);
    if (field) {
      field.disabled = !toggle.checked || !parentActive;
      setSettingFieldState(field, bridgeHasConfig(net) ? "Managed" : !parentActive ? "Not active" : !toggle.checked ? "Override off" : "Custom value");
    }
    const label = field?.closest(".bridge-v7-card")?.querySelector(".kgw-command-option-title-text-r8e");
    if (field && label) { label.id = field.id + "-label"; field.setAttribute("aria-labelledby", label.id); }
  });
  decorateSettingsFields(panel);
}
export { kgwBridgeEffectiveInprocessNodeSettings, kgwBridgeEffectiveSettingsV1, kgwBridgeValidateForm };
