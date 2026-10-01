use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const BRIDGE_SOURCE: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const BRIDGE_COMMAND_OPTIONS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_command_options.rs";
const BRIDGE_INSTANCE_SETTINGS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_instance_settings.rs";
const WASM_JS: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm.js";
const WASM_BIN: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm_bg.wasm";

const SLICES: &[(&str, &str)] = &[
    ("function kgwBridgeForm(", "function kgwBridgeValidateForm("),
    (
        "const BRIDGE_NETWORKS = wasmBridgeNetworkProfiles();",
        "const bridgeInstances = {",
    ),
];

const NODE_BRIDGE: &str = r##"
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import vm from "node:vm";

const [selectedPath, requestPath, resultPath, wasmJsPath, wasmPath] = process.argv.slice(2);
const selected = await readFile(selectedPath, "utf8");
const request = JSON.parse(await readFile(requestPath, "utf8"));
const wasmModule = await import(pathToFileURL(wasmJsPath).href);
await wasmModule.default({ module_or_path: await readFile(wasmPath) });

const elements = new Map();
for (const [id, value] of Object.entries(request.values)) {
  elements.set(id, {
    id,
    value: String(value),
    checked: false,
    type: "text",
    dispatchEvent() { return true; }
  });
}
let r51FieldsSelector = "";
const r51FilterOnly = [
  { id: "bridge-mainnet-commandPreview", closest() { return null; } },
  { id: "bridge-mainnet-logOutput", closest() { return null; } },
  { id: "bridge-testnet10-foreign", closest() { return null; } },
  {
    id: "bridge-mainnet-toolbarOwned",
    closest(selector) { return String(selector) === ".bridge-v7-log-toolbar" ? {} : null; }
  }
];
const panel = {
  querySelectorAll(selector) {
    const text = String(selector);
    if (text === "input, select, textarea") {
      r51FieldsSelector = text;
      return [...elements.values(), ...r51FilterOnly];
    }
    return [...elements.values()];
  }
};
const r51InlineToggle = {
  checked: true,
  dataset: {
    bridgeCommandOptionToggleR7: "coinbaseTagSuffix",
    net: "mainnet"
  }
};
const r51InstanceToggle = {
  checked: true,
  dataset: {
    instanceId: "one",
    bridgeInstanceCommandOptionToggleR13b: "instanceDiff",
    net: "mainnet"
  }
};
const bridgeRoot = {
  querySelectorAll(selector) {
    const text = String(selector);
    if (text === '[data-bridge-command-option-toggle-r7][data-net="mainnet"]') {
      return [r51InlineToggle];
    }
    if (text === '[data-bridge-instance-command-option-toggle-r13b][data-net="mainnet"]') {
      return [r51InstanceToggle];
    }
    return [];
  }
};
const sandbox = {
  Event: globalThis.Event,
  document: {
    querySelector() { return panel; },
    getElementById(id) {
      return String(id) === "kaspa-bridge" ? bridgeRoot : (elements.get(id) || null);
    }
  },
  console,
  BRIDGE_MANAGED: wasmModule.settingsBridgeManaged(),
  BRIDGE_REQUIRED: new Set(wasmModule.settingsBridgeRequired()),
  BRIDGE_OPTIONAL: new Set(wasmModule.settingsBridgeOptional()),
  bridgeFieldEnabled: (name, values, options) =>
    wasmModule.settingsBridgeFieldEnabled(name, values, options),
  wasmBridgeNetworkProfiles: wasmModule.bridgeNetworkProfiles,
  wasmBridgeNetworkProfile: wasmModule.bridgeNetworkProfile,
  wasmBridgeR51Panel: wasmModule.bridgeR51Panel,
  wasmBridgeR51Fields: wasmModule.bridgeR51Fields,
  wasmBridgeNetworkEnabled: wasmModule.bridgeNetworkEnabled,
  wasmBridgeById: wasmModule.bridgeById,
  wasmBridgeElementId: wasmModule.bridgeElementId,
  wasmBridgeValue: wasmModule.bridgeValue,
  wasmBridgeChecked: wasmModule.bridgeChecked,
  wasmBridgeCommandInlineStateKeyR7: wasmModule.bridgeCommandInlineStateKeyR7,
  wasmBridgeCommandInlineStateR7: wasmModule.bridgeCommandInlineStateR7,
  wasmBridgeCommandInlineToggleR7: wasmModule.bridgeCommandInlineToggleR7,
  wasmBridgeCommandOptionEnabledR7: wasmModule.bridgeCommandOptionEnabledR7,
  wasmBridgeHasConfig: wasmModule.bridgeHasConfig,
  wasmBridgeCommandSetOptionR7: wasmModule.bridgeCommandSetOptionR7,
  wasmBridgeCommandShouldIncludeR7: wasmModule.bridgeCommandShouldIncludeR7,
  wasmBridgeCommandToggleOptionR7: wasmModule.bridgeCommandToggleOptionR7,
  wasmBridgeInstanceCommandCheckboxR13B: wasmModule.bridgeInstanceCommandCheckboxR13B,
  wasmBridgeInstanceCommandOptionEnabledR13B: wasmModule.bridgeInstanceCommandOptionEnabledR13B,
  wasmBridgeInstanceCommandSetOptionR13B: wasmModule.bridgeInstanceCommandSetOptionR13B,
  wasmBridgeInstanceCommandShouldIncludeR13B: wasmModule.bridgeInstanceCommandShouldIncludeR13B,
  wasmBridgeInstanceCommandStateKeyR13B: wasmModule.bridgeInstanceCommandStateKeyR13B,
  bridgeInstanceParseStructured: wasmModule.bridgeInstanceParseStructured,
  wasmBridgeBoolValueV1: wasmModule.bridgeBoolValueV1,
  wasmBridgeBuildUpstreamInstanceArg: wasmModule.bridgeBuildUpstreamInstanceArg,
  wasmBridgeDefaultInstanceRecord: wasmModule.bridgeDefaultInstanceRecord,
  wasmBridgeEffectiveSettingsV1: wasmModule.bridgeEffectiveSettingsV1,
  wasmBridgeInstanceParseStructured: wasmModule.bridgeInstanceParseStructured,
  wasmBridgeInstancePlainValue: wasmModule.bridgeInstancePlainValue,
  wasmBridgeInstancePortValue: wasmModule.bridgeInstancePortValue,
  wasmBridgeNormalizeInstanceRecord: wasmModule.bridgeNormalizeInstanceRecord,
  wasmBridgeOptionalTextV1: wasmModule.bridgeOptionalTextV1,
  wasmBridgeParseDurationMsV1: wasmModule.bridgeParseDurationMsV1,
  wasmBridgeParseUnsignedV1: wasmModule.bridgeParseUnsignedV1,
  wasmBridgePortListenV1: wasmModule.bridgePortListenV1,
  bridgeInstances: {
    mainnet: request.structured.instances,
    testnet10: [],
    testnet13: []
  }
};
sandbox.window = sandbox;
sandbox.globalThis = sandbox;
globalThis.document = sandbox.document;
globalThis.window = sandbox;
sandbox.__kgwBridgeCommandComposerInlineR7 = request.inlineOptions;
sandbox.__kgwBridgeInstanceCommandComposerR13B = {};

vm.createContext(sandbox);
vm.runInContext(selected, sandbox, { filename: request.sourceName });
const api = vm.runInContext(
  "({ parse: bridgeInstanceParseStructured, effective: wasmBridgeEffectiveSettingsV1 })",
  sandbox
);

const directInstanceCheckbox = wasmModule.bridgeInstanceCommandCheckboxFromInstancesR13B(
  sandbox.bridgeInstances,
  "mainnet",
  "one",
  "instanceDiff"
);
const inlineEnabledBeforeWrite =
  wasmModule.bridgeCommandOptionEnabledR7("mainnet", "coinbaseTagSuffix");
const nodeModeControl = elements.get("bridge-mainnet-nodeMode");
const nodeModeInitial = wasmModule.bridgeNodeMode("mainnet");
nodeModeControl.value = "inprocess";
const nodeModeExactInprocess = wasmModule.bridgeNodeMode("mainnet");
nodeModeControl.value = "In-Process";
const nodeModeNonExact = wasmModule.bridgeNodeMode("mainnet");
nodeModeControl.value = "external";
const r51FieldIds = Array.from(wasmModule.bridgeR51Fields("mainnet"))
  .map((field) => String(field?.id || ""));
const r26bBridgeInstances = {
  mainnet: [{
    id: "r253-one",
    instance: "",
    instancePort: "5666",
    instanceDiff: "1024",
    instanceProm: "",
    instanceLogToFile: "not set",
    instanceBlockWaitTime: "",
    instanceExtranonceSize: "",
    instanceVarDiff: "not set",
    instanceSharesPerMin: "",
    instanceVarDiffStats: "not set",
    instancePow2Clamp: "not set"
  }],
  testnet10: [],
  testnet13: []
};
const r26bActiveInstance = { mainnet: "r253-one" };
const r26bReadCalls = [];
const r26bStructured = wasmModule.bridgeR51ReadStructuredInstances(
  "mainnet",
  r26bBridgeInstances,
  r26bActiveInstance,
  {
    readInstanceState: (net, instanceId) => {
      r26bReadCalls.push(String(net) + ":" + String(instanceId));
      return {
        id: String(instanceId),
        instance: "",
        instancePort: "5666",
        instanceDiff: "8192",
        instanceProm: "",
        instanceLogToFile: "not set",
        instanceBlockWaitTime: "",
        instanceExtranonceSize: "",
        instanceVarDiff: "not set",
        instanceSharesPerMin: "",
        instanceVarDiffStats: "not set",
        instancePow2Clamp: "not set"
      };
    }
  }
);
elements.set("bridge-mainnet-op249Checkbox", {
  id: "bridge-mainnet-op249Checkbox",
  value: "ignored",
  checked: true,
  type: "checkbox"
});
let r51ReadNormalizeReason = "";
const r51ReadSettings = wasmModule.bridgeR51ReadSettings("mainnet", {
  readStructuredInstances: () => request.structured,
  normalizeNetworkPortValues: (_net, values, reason) => {
    r51ReadNormalizeReason = String(reason || "");
    values.__op249Normalized = true;
    return values;
  }
});
elements.delete("bridge-mainnet-op249Checkbox");
const r51WriteEvents = [];
const r51WriteValueField = {
  id: "bridge-mainnet-op250Value",
  value: "before",
  checked: false,
  type: "text",
  dispatchEvent(event) {
    r51WriteEvents.push(this.id + ":" + String(event?.type || "") + ":" + String(Boolean(event?.bubbles)));
    return true;
  }
};
const r51WriteCheckboxField = {
  id: "bridge-mainnet-op250Checkbox",
  value: "ignored",
  checked: false,
  type: "checkbox",
  dispatchEvent(event) {
    r51WriteEvents.push(this.id + ":" + String(event?.type || "") + ":" + String(Boolean(event?.bubbles)));
    return true;
  }
};
elements.set(r51WriteValueField.id, r51WriteValueField);
elements.set(r51WriteCheckboxField.id, r51WriteCheckboxField);
const r51WriteValues = {
  "bridge-mainnet-op250Value": { type: "value", value: "after" },
  "bridge-mainnet-op250Checkbox": { type: "checkbox", checked: true },
  "bridge-mainnet-logToFile": { type: "value", value: "managed-must-not-apply" },
  "bridge-mainnet-inprocessRpcListenBorsh": { type: "value", value: "" },
  "__kgwBridgeCommandOptionsR38C": {
    op252Inline: true,
    inprocessRpcListenBorsh: true
  },
  "__kgwBridgeInstanceCommandOptionsR38C": {
    one: {
      op252Instance: true,
      instanceBlockWaitTime: true
    },
    two: {
      instanceSharesPerMin: true
    }
  }
};
const r51WriteBridgeInstances = {
  mainnet: request.structured.instances.map((item) => ({ ...item })),
  testnet10: [],
  testnet13: []
};
const r51WriteActiveInstance = { mainnet: "one" };
let r51WriteNormalizeReason = "";
let r51WriteNormalizeCount = 0;
let r51WriteStructuredCount = 0;
let r51WriteRefreshCount = 0;
const r51WriteSetInstanceCalls = [];
let r51WriteUpdateCommandCount = 0;
wasmModule.bridgeR51WriteSettings(
  "mainnet",
  r51WriteValues,
  r51WriteBridgeInstances,
  r51WriteActiveInstance,
  {
    normalizeNetworkPortValues: (_net, values, reason) => {
      r51WriteNormalizeCount += 1;
      r51WriteNormalizeReason = String(reason || "");
      values.__op250Normalized = true;
      return values;
    },
    applyStructuredInstances: () => {
      r51WriteStructuredCount += 1;
      return false;
    },
    refreshInlineCommandToggles: () => {
      r51WriteRefreshCount += 1;
    },
    setInstanceCommandOption: (net, instanceId, name, enabled) => {
      r51WriteSetInstanceCalls.push(
        [String(net), String(instanceId), String(name), String(Boolean(enabled))].join(":")
      );
      wasmModule.bridgeInstanceCommandSetOptionR13B(
        String(net),
        instanceId,
        String(name),
        Boolean(enabled)
      );
    },
    updateCommand: () => {
      r51WriteUpdateCommandCount += 1;
    }
  }
);
const r51WriteInlineEnabled =
  wasmModule.bridgeCommandOptionEnabledR7("mainnet", "op252Inline");
const r51WriteOptionalInlineDisabled =
  !wasmModule.bridgeCommandOptionEnabledR7("mainnet", "inprocessRpcListenBorsh");
const r51WriteInstanceEnabled =
  wasmModule.bridgeInstanceCommandOptionEnabledR13B(
    "mainnet",
    "one",
    "op252Instance",
    r51WriteBridgeInstances.mainnet[0]
  );
const r51WriteOptionalInstanceEnabled =
  wasmModule.bridgeInstanceCommandOptionEnabledR13B(
    "mainnet",
    "one",
    "instanceBlockWaitTime",
    r51WriteBridgeInstances.mainnet[0]
  );
const r51WriteEmptyOptionalInstanceDisabled =
  !wasmModule.bridgeInstanceCommandOptionEnabledR13B(
    "mainnet",
    "two",
    "instanceSharesPerMin",
    r51WriteBridgeInstances.mainnet[1]
  );
const output = {
  directOwners: {
    hasConfigInitially: wasmModule.bridgeHasConfig("mainnet"),
    inlineEnabled: inlineEnabledBeforeWrite,
    inlineToggleHasMarker: wasmModule.bridgeCommandInlineToggleR7("mainnet", "coinbaseTagSuffix").includes('data-bridge-command-option-toggle-r7="coinbaseTagSuffix"'),
    instanceShouldInclude: wasmModule.bridgeInstanceCommandShouldIncludeFromInstancesR13B(
      sandbox.bridgeInstances,
      "mainnet",
      "one",
      "instanceDiff"
    ),
    instanceCheckboxHasId: directInstanceCheckbox.includes('data-instance-id="one"'),
    instanceCheckboxChecked: directInstanceCheckbox.includes("checked"),
    nodeModeInitial,
    nodeModeExactInprocess,
    nodeModeNonExact,
    r51FieldsSelector,
    r51FieldsIncludesNodeMode: r51FieldIds.includes("bridge-mainnet-nodeMode"),
    r51FieldsExcludesCommandPreview: !r51FieldIds.includes("bridge-mainnet-commandPreview"),
    r51FieldsExcludesLogOutput: !r51FieldIds.includes("bridge-mainnet-logOutput"),
    r51FieldsExcludesOtherNetwork: !r51FieldIds.includes("bridge-testnet10-foreign"),
    r51FieldsExcludesToolbarOwned: !r51FieldIds.includes("bridge-mainnet-toolbarOwned"),
    r26bStructuredVersion: r26bStructured.version,
    r26bStructuredActive: String(r26bStructured.activeInstance || ""),
    r26bStructuredCount: Array.isArray(r26bStructured.instances) ? r26bStructured.instances.length : -1,
    r26bStructuredDiff: String(r26bStructured.instances?.[0]?.instanceDiff || ""),
    r26bReadCalls,
    r51ReadActiveInstance: r51ReadSettings.__kgwBridgeActiveInstanceR26B,
    r51ReadStructuredCount: r51ReadSettings.__kgwBridgeStructuredInstancesR26B?.instances?.length ?? -1,
    r51ReadCommandOption: Boolean(r51ReadSettings.__kgwBridgeCommandOptionsR38C?.coinbaseTagSuffix),
    r51ReadInstanceCommandOption: Boolean(r51ReadSettings.__kgwBridgeInstanceCommandOptionsR38C?.one?.instanceDiff),
    r51ReadNodeModeType: r51ReadSettings["bridge-mainnet-nodeMode"]?.type ?? "",
    r51ReadNodeModeValue: r51ReadSettings["bridge-mainnet-nodeMode"]?.value ?? "",
    r51ReadCheckboxType: r51ReadSettings["bridge-mainnet-op249Checkbox"]?.type ?? "",
    r51ReadCheckboxChecked: Boolean(r51ReadSettings["bridge-mainnet-op249Checkbox"]?.checked),
    r51ReadExcludesManaged:
      !("bridge-mainnet-logToFile" in r51ReadSettings) &&
      !("bridge-mainnet-healthCheckPort" in r51ReadSettings) &&
      !("bridge-mainnet-webDashboardPort" in r51ReadSettings),
    r51ReadNormalized: Boolean(r51ReadSettings.__op249Normalized),
    r51ReadNormalizeReason,
    r51WriteValue: r51WriteValueField.value,
    r51WriteCheckboxChecked: Boolean(r51WriteCheckboxField.checked),
    r51WriteManagedPreserved: elements.get("bridge-mainnet-logToFile")?.value === "false",
    r51WriteEvents,
    r51WriteNormalized: Boolean(r51WriteValues.__op250Normalized),
    r51WriteNormalizeReason,
    r51WriteNormalizeCount,
    r51WriteStructuredCount,
    r51WriteRefreshCount,
    r51WriteSetInstanceCalls,
    r51WriteUpdateCommandCount,
    r51WriteInlineEnabled,
    r51WriteOptionalInlineDisabled,
    r51WriteInstanceEnabled,
    r51WriteOptionalInstanceEnabled,
    r51WriteEmptyOptionalInstanceDisabled
  }
};
for (const step of request.steps) {
  if (step.op === "parse") {
    output[step.label] = api.parse(step.value);
  } else if (step.op === "effective") {
    output[step.label] = api.effective(step.net, request.structured);
  } else if (step.op === "set-instance-option") {
    const [net, instanceId, name] = String(step.key || "").split("::");
    wasmModule.bridgeInstanceCommandSetOptionR13B(net, instanceId, name, Boolean(step.value));
  } else if (step.op === "set-inline-option") {
    wasmModule.bridgeCommandSetOptionR7(step.net, step.name, Boolean(step.value));
  } else if (step.op === "set-element") {
    const existing = elements.get(step.id);
    if (existing) {
      existing.value = String(step.value);
    } else {
      elements.set(step.id, {
        id: step.id, value: String(step.value), checked: false, type: "text"
      });
    }
  } else {
    throw new Error("unknown effective-bridge step: " + step.op);
  }
}
output.directOwners.hasConfigEnabled = wasmModule.bridgeHasConfig("mainnet");
await writeFile(resultPath, JSON.stringify(output), "utf8");
"##;

fn slice_between<'a>(source: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    let start_index = source
        .find(start)
        .ok_or_else(|| format!("effective Bridge source start marker missing: {start}"))?;
    let relative_end = source[start_index..]
        .find(end)
        .ok_or_else(|| format!("effective Bridge source end marker missing: {end}"))?;
    let end_index = start_index + relative_end;
    if end_index <= start_index {
        return Err(format!(
            "effective Bridge source markers reversed: {start} -> {end}"
        ));
    }
    Ok(&source[start_index..end_index])
}

fn selected_source(root: &Path) -> Result<String, String> {
    let source = fs::read_to_string(root.join(BRIDGE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_SOURCE}: {error}"))?;
    let mut selected = String::new();
    for &(start, end) in SLICES {
        selected.push_str(slice_between(&source, start, end)?);
        selected.push('\n');
    }
    Ok(selected)
}

fn verify_direct_command_option_ownership(root: &Path) -> Result<(), String> {
    let source = fs::read_to_string(root.join(BRIDGE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_SOURCE}: {error}"))?;
    let rust = fs::read_to_string(root.join(BRIDGE_COMMAND_OPTIONS_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_COMMAND_OPTIONS_SOURCE}: {error}"))?;
    let instance_settings = fs::read_to_string(root.join(BRIDGE_INSTANCE_SETTINGS_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_INSTANCE_SETTINGS_SOURCE}: {error}"))?;

    for needle in [
        "bridgeInstanceCommandShouldIncludeFromInstancesR13B",
        "bridgeInstanceCommandCheckboxFromInstancesR13B",
        "bridgeHasConfig",
    ] {
        if !rust.contains(needle) {
            return Err(format!(
                "Bridge command-options Rust direct-owner export missing: {needle}"
            ));
        }
    }

    for forbidden in [
        "function kgwBridgeInstanceCommandRecordR13B(",
        "function kgwBridgeInstanceCommandShouldIncludeR13B(",
        "function kgwBridgeInstanceCommandCheckboxR13B(",
        "function kgwBridgeCommandInlineStateR7(",
        "function kgwBridgeCommandOptionEnabledR7(",
        "function kgwBridgeCommandInlineToggleR7(",
        "function bridgeHasConfig(",
        "function kgwBridgeEffectiveSettingsV1(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge command-option wrapper remains in JavaScript: {forbidden}"
            ));
        }
    }

    if !source.contains("wasmBridgeEffectiveSettingsV1 as kgwBridgeEffectiveSettingsV1") {
        return Err(
            "Bridge effective-settings compatibility export is not a direct Rust/WASM alias"
                .to_owned(),
        );
    }

    if source.contains("function bridgeProfile(") {
        return Err("Retired Bridge profile lookup JavaScript seam remains".to_owned());
    }
    if source
        .matches("wasmBridgeNetworkProfile(String(net || \"\"))")
        .count()
        != 2
    {
        return Err(
            "Bridge profile lookup must use exactly two direct Rust/WASM call sites".to_owned(),
        );
    }

    if source.contains("function bridgeNodeMode(") {
        return Err("Retired Bridge node-mode JavaScript owner remains".to_owned());
    }
    if !source.contains("bridgeNodeMode as wasmBridgeNodeMode") {
        return Err("Bridge node-mode Rust/WASM import is missing".to_owned());
    }
    if source
        .matches("wasmBridgeNodeMode(String(net || \"\"))")
        .count()
        != 6
    {
        return Err("Bridge node-mode must use exactly six direct Rust/WASM call sites".to_owned());
    }

    if source.contains("function kgwBridgeR51Panel(") {
        return Err("Retired Bridge R51 panel JavaScript owner remains".to_owned());
    }
    if !source.contains("bridgeR51Panel as wasmBridgeR51Panel") {
        return Err("Bridge R51 panel Rust/WASM import is missing".to_owned());
    }
    if source.matches("wasmBridgeR51Panel(").count() != 7 {
        return Err(
            "Bridge R51 panel must use exactly seven direct Rust/WASM call sites".to_owned(),
        );
    }
    if !instance_settings.contains("js_name = bridgeR51Panel") {
        return Err("Bridge R51 panel Rust export is missing".to_owned());
    }

    if source.contains("function kgwBridgeR51Fields(") {
        return Err("Retired Bridge R51 fields JavaScript owner remains".to_owned());
    }
    if source.contains("bridgeR51Fields as wasmBridgeR51Fields")
        || source.matches("wasmBridgeR51Fields(").count() != 0
    {
        return Err(
            "Bridge R51 fields must have no direct JavaScript import/call sites after OP250 moves the remaining WriteSettings enumeration into Rust".to_owned(),
        );
    }
    if !instance_settings.contains("js_name = bridgeR51Fields")
        || !instance_settings.contains("fn bridge_r51_fields_vec(")
    {
        return Err("Bridge R51 fields Rust owner/export is missing".to_owned());
    }

    if source.contains("function kgwBridgeR51ReadSettings(") {
        return Err("Retired Bridge R51 ReadSettings JavaScript owner remains".to_owned());
    }
    if !source.contains("bridgeR51ReadSettings as wasmBridgeR51ReadSettings") {
        return Err("Bridge R51 ReadSettings Rust/WASM import is missing".to_owned());
    }
    if source.matches("wasmBridgeR51ReadSettings(").count() != 1 {
        return Err(
            "Bridge R51 ReadSettings must have exactly one thin Rust/WASM wrapper call".to_owned(),
        );
    }
    if source.matches("kgwBridgeR51ReadSettingsR249(").count() != 6 {
        return Err(
            "Bridge R51 ReadSettings thin wrapper must own exactly five consumers plus its definition"
                .to_owned(),
        );
    }
    for needle in [
        "js_name = bridgeR51ReadSettings",
        "pub fn bridge_r51_read_settings(",
        "readStructuredInstances",
        "normalizeNetworkPortValues",
        "bridge_command_options::bridge_r51_read_command_options_r38c(&net)",
        "bridge_command_options::bridge_r51_read_instance_command_options_r38c(&net)",
    ] {
        if !instance_settings.contains(needle) {
            return Err(format!(
                "Bridge R51 ReadSettings Rust ownership contract missing: {needle}"
            ));
        }
    }
    for forbidden in [
        "function kgwBridgeR51ReadCommandOptionsR38C(",
        "function kgwBridgeR51ReadInstanceCommandOptionsR38C(",
        "readCommandOptions:",
        "readInstanceCommandOptions:",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R38C command-option reader JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    for needle in [
        "pub(crate) fn bridge_r51_read_command_options_r38c(",
        "pub(crate) fn bridge_r51_read_instance_command_options_r38c(",
        "data-bridge-command-option-toggle-r7][data-net=",
        "data-bridge-instance-command-option-toggle-r13b][data-net=",
        "bridgeCommandOptionToggleR7",
        "bridgeInstanceCommandOptionToggleR13b",
        "instanceId",
    ] {
        if !rust.contains(needle) {
            return Err(format!(
                "Bridge R38C command-option Rust reader contract missing: {needle}"
            ));
        }
    }

    for forbidden in [
        "function kgwBridgeR51CommitInstanceDomStateR26B(",
        "function kgwBridgeR51ReadStructuredInstancesR26B(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R26B structured-read JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    if !source.contains("bridgeR51ReadStructuredInstances as wasmBridgeR51ReadStructuredInstances")
    {
        return Err("Bridge R26B structured-read Rust/WASM import is missing".to_owned());
    }
    if source
        .matches("wasmBridgeR51ReadStructuredInstances(")
        .count()
        != 1
    {
        return Err(
            "Bridge R26B structured-read must have exactly one thin Rust/WASM wrapper call"
                .to_owned(),
        );
    }
    if source
        .matches("kgwBridgeR51ReadStructuredInstancesR253(")
        .count()
        != 4
    {
        return Err(
            "Bridge R26B structured-read thin wrapper must own exactly three direct calls plus its definition"
                .to_owned(),
        );
    }
    for needle in [
        "js_name = bridgeR51ReadStructuredInstances",
        "pub fn bridge_r51_read_structured_instances(",
        "fn bridge_r51_commit_instance_dom_state_r26b(",
        "\"readInstanceState\"",
        "bridge_port_orchestration::bridge_ensure_instance_state(",
        "normalize_instance_record_impl(",
        "r26b-commit-instance-dom-state-failed",
    ] {
        if !instance_settings.contains(needle) {
            return Err(format!(
                "Bridge R26B structured-read Rust ownership contract missing: {needle}"
            ));
        }
    }

    if source.contains("function kgwBridgeR51WriteSettings(") {
        return Err("Retired Bridge R51 WriteSettings JavaScript owner remains".to_owned());
    }
    if !source.contains("bridgeR51WriteSettings as wasmBridgeR51WriteSettings") {
        return Err("Bridge R51 WriteSettings Rust/WASM import is missing".to_owned());
    }
    if source.matches("wasmBridgeR51WriteSettings(").count() != 1 {
        return Err(
            "Bridge R51 WriteSettings must have exactly one thin Rust/WASM wrapper call".to_owned(),
        );
    }
    if source.matches("kgwBridgeR51WriteSettingsR250(").count() != 4 {
        return Err(
            "Bridge R51 WriteSettings thin wrapper must own exactly three consumers plus its definition"
                .to_owned(),
        );
    }
    for needle in [
        "js_name = bridgeR51WriteSettings",
        "pub fn bridge_r51_write_settings(",
        "applyStructuredInstances",
        "bridge_command_options::bridge_r51_apply_command_options_r38c(",
        "updateCommand",
        "bridge_port_orchestration::bridge_reassign_instance_ports_from_external_range_r91",
        "bridge_instance_ui::bridge_sync_instance_preview_rows_r8b",
    ] {
        if !instance_settings.contains(needle) {
            return Err(format!(
                "Bridge R51 WriteSettings Rust ownership contract missing: {needle}"
            ));
        }
    }
    for forbidden in [
        "function kgwBridgeR51ApplyCommandOptionsR38C(",
        "applyCommandOptions:",
        "KGW_BRIDGE_R51_COMMAND_OPTIONS_KEY_R38C",
        "KGW_BRIDGE_R51_INSTANCE_COMMAND_OPTIONS_KEY_R38C",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge R38C ApplyCommandOptions JavaScript ownership remains: {forbidden}"
            ));
        }
    }
    for needle in [
        "pub(crate) fn bridge_r51_apply_command_options_r38c(",
        "__kgwBridgeCommandOptionsR38C",
        "__kgwBridgeInstanceCommandOptionsR38C",
        "refreshInlineCommandToggles",
        "setInstanceCommandOption",
        "r38c-command-options-restored",
        "r38c-command-options-restore-failed",
    ] {
        if !rust.contains(needle) {
            return Err(format!(
                "Bridge R38C ApplyCommandOptions Rust ownership contract missing: {needle}"
            ));
        }
    }

    Ok(())
}

fn request() -> Value {
    json!({
        "sourceName": BRIDGE_SOURCE,
        "values": {
            "bridge-mainnet-nodeMode": "external",
            "bridge-mainnet-kaspadAddress": "127.0.0.1:16110",
            "bridge-mainnet-blockWaitTime": "1500ms",
            "bridge-mainnet-printStats": "false",
            "bridge-mainnet-logToFile": "false",
            "bridge-mainnet-healthCheckPort": "",
            "bridge-mainnet-webDashboardPort": "",
            "bridge-mainnet-varDiff": "true",
            "bridge-mainnet-sharesPerMin": "20",
            "bridge-mainnet-varDiffStats": "false",
            "bridge-mainnet-extranonceSize": "2",
            "bridge-mainnet-pow2Clamp": "false",
            "bridge-mainnet-coinbaseTagSuffix": "owner-test",
            "bridge-mainnet-stratumPort": "5555",
            "bridge-mainnet-minShareDiff": "8192",
            "bridge-mainnet-promPort": "2112",
            "bridge-mainnet-config": ""
        },
        "inlineOptions": {"mainnet": {"coinbaseTagSuffix": true}},

        "structured": {
            "activeInstance": "one",
            "instances": [
                {
                    "id": "one",
                    "instancePort": "5556",
                    "instanceDiff": "4096",
                    "instanceProm": "2113",
                    "instanceLogToFile": "not set",
                    "instanceBlockWaitTime": "2500ms",
                    "instanceExtranonceSize": "4",
                    "instanceVarDiff": "not set",
                    "instanceSharesPerMin": "30",
                    "instanceVarDiffStats": "true",
                    "instancePow2Clamp": "not set"
                },
                {
                    "id": "two",
                    "instancePort": "5557",
                    "instanceDiff": "2048",
                    "instanceProm": "2114",
                    "instanceLogToFile": "true",
                    "instanceVarDiff": "false",
                    "instanceVarDiffStats": "false",
                    "instancePow2Clamp": "true"
                }
            ]
        },

        "steps": [
            {"op": "parse", "label": "parsed", "value": "port=:5556,prom=:2113,wait=2500ms,extranonce=4,log=false,var_diff=true,shares_per_min=30,var_diff_stats=true,pow2_clamp=true"},
            {"op": "effective", "label": "initial", "net": "mainnet"},
            {"op": "set-instance-option", "key": "mainnet::two::instance", "value": false},
            {"op": "set-instance-option", "key": "mainnet::one::instanceProm", "value": false},
            {"op": "effective", "label": "instance_filtered", "net": "mainnet"},
            {"op": "set-inline-option", "net": "mainnet", "name": "promPort", "value": false},
            {"op": "effective", "label": "prom_disabled", "net": "mainnet"},
            {"op": "set-inline-option", "net": "mainnet", "name": "promPort", "value": true},
            {"op": "set-element", "id": "bridge-mainnet-inprocessRpcListen", "value": "127.0.0.1:19110"},
            {"op": "set-element", "id": "bridge-mainnet-nodeMode", "value": "inprocess"},
            {"op": "effective", "label": "inprocess", "net": "mainnet"},
            {"op": "set-element", "id": "bridge-mainnet-nodeMode", "value": "external"},
            {"op": "set-element", "id": "bridge-mainnet-config", "value": "/tmp/official-bridge.yaml"},
            {"op": "effective", "label": "config_draft_disabled", "net": "mainnet"},
            {"op": "set-inline-option", "net": "mainnet", "name": "config", "value": true},
            {"op": "effective", "label": "config_enabled", "net": "mainnet"}
        ]
    })
}

fn run_bridge(root: &Path, selected: &str) -> Result<Value, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-effective-bridge-")
        .tempdir()
        .map_err(|error| format!("failed to create effective Bridge tempdir: {error}"))?;
    let bridge_path = temp.path().join("bridge.mjs");
    let selected_path = temp.path().join("selected.js");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write temporary Node bridge: {error}"))?;
    fs::write(&selected_path, selected.as_bytes())
        .map_err(|error| format!("failed to write selected Bridge source: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(&request())
            .map_err(|error| format!("failed to serialize effective Bridge request: {error}"))?,
    )
    .map_err(|error| format!("failed to write effective Bridge request: {error}"))?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&selected_path)
        .arg(&request_path)
        .arg(&result_path)
        .arg(root.join(WASM_JS))
        .arg(root.join(WASM_BIN))
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch Node effective Bridge bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "effective Bridge Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read effective Bridge result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse effective Bridge result: {error}"))
}

fn expect_pointer(actual: &Value, pointer: &str, expected: Value) -> Result<(), String> {
    let value = actual
        .pointer(pointer)
        .ok_or_else(|| format!("effective Bridge result missing {pointer}"))?;
    if value != &expected {
        return Err(format!(
            "effective Bridge mismatch at {pointer}: expected {expected}, got {value}"
        ));
    }
    Ok(())
}

fn expect_len(actual: &Value, pointer: &str, expected: usize) -> Result<(), String> {
    let value = actual
        .pointer(pointer)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("effective Bridge result is not an array at {pointer}"))?;
    if value.len() != expected {
        return Err(format!(
            "effective Bridge length mismatch at {pointer}: expected {expected}, got {}",
            value.len()
        ));
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<String, String> {
    verify_direct_command_option_ownership(root)?;
    let selected = selected_source(root)?;
    let actual = run_bridge(root, &selected)?;

    expect_pointer(&actual, "/directOwners/hasConfigInitially", json!(false))?;
    expect_pointer(&actual, "/directOwners/hasConfigEnabled", json!(true))?;
    expect_pointer(&actual, "/directOwners/inlineEnabled", json!(true))?;
    expect_pointer(&actual, "/directOwners/inlineToggleHasMarker", json!(true))?;
    expect_pointer(&actual, "/directOwners/instanceShouldInclude", json!(true))?;
    expect_pointer(&actual, "/directOwners/instanceCheckboxHasId", json!(true))?;
    expect_pointer(
        &actual,
        "/directOwners/instanceCheckboxChecked",
        json!(true),
    )?;
    expect_pointer(&actual, "/directOwners/nodeModeInitial", json!("external"))?;
    expect_pointer(
        &actual,
        "/directOwners/nodeModeExactInprocess",
        json!("inprocess"),
    )?;
    expect_pointer(&actual, "/directOwners/nodeModeNonExact", json!("external"))?;
    expect_pointer(
        &actual,
        "/directOwners/r51FieldsSelector",
        json!("input, select, textarea"),
    )?;
    for pointer in [
        "/directOwners/r51FieldsIncludesNodeMode",
        "/directOwners/r51FieldsExcludesCommandPreview",
        "/directOwners/r51FieldsExcludesLogOutput",
        "/directOwners/r51FieldsExcludesOtherNetwork",
        "/directOwners/r51FieldsExcludesToolbarOwned",
        "/directOwners/r51ReadCommandOption",
        "/directOwners/r51ReadInstanceCommandOption",
        "/directOwners/r51ReadCheckboxChecked",
        "/directOwners/r51ReadExcludesManaged",
        "/directOwners/r51ReadNormalized",
    ] {
        expect_pointer(&actual, pointer, json!(true))?;
    }
    expect_pointer(&actual, "/directOwners/r26bStructuredVersion", json!(1))?;
    expect_pointer(
        &actual,
        "/directOwners/r26bStructuredActive",
        json!("r253-one"),
    )?;
    expect_pointer(&actual, "/directOwners/r26bStructuredCount", json!(1))?;
    expect_pointer(&actual, "/directOwners/r26bStructuredDiff", json!("8192"))?;
    expect_pointer(
        &actual,
        "/directOwners/r26bReadCalls",
        json!(["mainnet:r253-one"]),
    )?;
    expect_pointer(&actual, "/directOwners/r51ReadActiveInstance", json!("one"))?;
    expect_pointer(&actual, "/directOwners/r51ReadStructuredCount", json!(2))?;
    expect_pointer(&actual, "/directOwners/r51ReadNodeModeType", json!("value"))?;
    expect_pointer(
        &actual,
        "/directOwners/r51ReadNodeModeValue",
        json!("external"),
    )?;
    expect_pointer(
        &actual,
        "/directOwners/r51ReadCheckboxType",
        json!("checkbox"),
    )?;
    expect_pointer(
        &actual,
        "/directOwners/r51ReadNormalizeReason",
        json!("read-settings"),
    )?;
    expect_pointer(&actual, "/directOwners/r51WriteValue", json!("after"))?;
    for pointer in [
        "/directOwners/r51WriteCheckboxChecked",
        "/directOwners/r51WriteManagedPreserved",
        "/directOwners/r51WriteNormalized",
    ] {
        expect_pointer(&actual, pointer, json!(true))?;
    }
    expect_pointer(
        &actual,
        "/directOwners/r51WriteNormalizeReason",
        json!("write-settings"),
    )?;
    for pointer in [
        "/directOwners/r51WriteNormalizeCount",
        "/directOwners/r51WriteStructuredCount",
        "/directOwners/r51WriteRefreshCount",
    ] {
        expect_pointer(&actual, pointer, json!(1))?;
    }
    expect_pointer(
        &actual,
        "/directOwners/r51WriteUpdateCommandCount",
        json!(2),
    )?;
    for pointer in [
        "/directOwners/r51WriteInlineEnabled",
        "/directOwners/r51WriteOptionalInlineDisabled",
        "/directOwners/r51WriteInstanceEnabled",
        "/directOwners/r51WriteOptionalInstanceEnabled",
        "/directOwners/r51WriteEmptyOptionalInstanceDisabled",
    ] {
        expect_pointer(&actual, pointer, json!(true))?;
    }
    expect_pointer(
        &actual,
        "/directOwners/r51WriteSetInstanceCalls",
        json!([
            "mainnet:one:op252Instance:true",
            "mainnet:one:instanceBlockWaitTime:true",
            "mainnet:two:instanceSharesPerMin:false"
        ]),
    )?;
    expect_pointer(
        &actual,
        "/directOwners/r51WriteEvents",
        json!([
            "bridge-mainnet-op250Value:input:true",
            "bridge-mainnet-op250Value:change:true",
            "bridge-mainnet-op250Checkbox:input:true",
            "bridge-mainnet-op250Checkbox:change:true"
        ]),
    )?;

    expect_pointer(&actual, "/parsed/instanceBlockWaitTime", json!("2500ms"))?;
    expect_pointer(&actual, "/parsed/instanceExtranonceSize", json!("4"))?;
    expect_pointer(&actual, "/initial/version", json!(1))?;
    expect_pointer(&actual, "/initial/global/blockWaitTimeMs", json!(1500))?;
    expect_pointer(&actual, "/initial/global/printStats", json!(false))?;
    expect_pointer(&actual, "/initial/global/logToFile", json!(false))?;
    expect_pointer(
        &actual,
        "/initial/global/coinbaseTagSuffix",
        json!("owner-test"),
    )?;
    expect_len(&actual, "/initial/instances", 2)?;
    expect_pointer(
        &actual,
        "/initial/instances/0/stratumListen",
        json!(":5556"),
    )?;
    expect_pointer(
        &actual,
        "/initial/instances/0/prometheusListen",
        json!(":2113"),
    )?;
    expect_pointer(&actual, "/initial/instances/0/logToFile", Value::Null)?;
    expect_pointer(&actual, "/initial/instances/0/varDiff", Value::Null)?;
    expect_pointer(&actual, "/initial/instances/0/blockWaitTimeMs", json!(2500))?;
    expect_pointer(&actual, "/initial/instances/0/extranonceSize", json!(4))?;
    expect_pointer(&actual, "/initial/instances/1/logToFile", json!(true))?;
    expect_pointer(&actual, "/initial/instances/1/varDiff", json!(false))?;

    expect_len(&actual, "/instance_filtered/instances", 1)?;
    expect_pointer(
        &actual,
        "/instance_filtered/instances/0/prometheusListen",
        json!(":2112"),
    )?;
    expect_pointer(
        &actual,
        "/prom_disabled/instances/0/prometheusListen",
        Value::Null,
    )?;
    expect_pointer(
        &actual,
        "/inprocess/global/kaspaRpcEndpoint",
        json!("127.0.0.1:19110"),
    )?;
    expect_pointer(&actual, "/config_draft_disabled/version", json!(1))?;
    expect_pointer(&actual, "/config_enabled", Value::Null)?;

    Ok("Effective Bridge settings frontend Rust owner PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_slice_markers_fail_closed() {
        assert!(slice_between("missing", "start", "end").is_err());
        assert_eq!(
            slice_between("prefix START body END tail", "START", "END").unwrap(),
            "START body "
        );
    }

    #[test]
    fn rust_owns_original_effective_bridge_scenario_sequence() {
        let request = request();
        let steps = request["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 16);
        assert_eq!(steps[0]["label"], "parsed");
        assert_eq!(steps[1]["label"], "initial");
        assert_eq!(steps[15]["label"], "config_enabled");
    }
}
