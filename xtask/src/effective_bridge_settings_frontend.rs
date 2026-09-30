use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const BRIDGE_SOURCE: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const BRIDGE_COMMAND_OPTIONS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_command_options.rs";
const WASM_JS: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm.js";
const WASM_BIN: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm_bg.wasm";

const SLICES: &[(&str, &str)] = &[
    ("function bridgeNodeMode(", "function bridgeControlCard("),
    (
        "function kgwBridgeR51Panel(",
        "function kgwBridgeR51Fields(",
    ),
    ("function kgwBridgeForm(", "function kgwBridgeValidateForm("),
    (
        "const BRIDGE_NETWORKS = wasmBridgeNetworkProfiles();",
        "const bridgeInstances = {",
    ),
    (
        "function kgwBridgeEffectiveSettingsV1(",
        "/* Port conflict registry/validation ownership lives in Rust bridge_port_core.rs. */",
    ),
    (
        "function bridgeProfile(",
        "/* KGW_BRIDGE_NETWORK_PORT_PROFILES_SOFT_POLICY_PATCH_R35B",
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
  elements.set(id, { id, value: String(value), checked: false, type: "text" });
}
const panel = {
  querySelectorAll() { return [...elements.values()]; }
};
const sandbox = {
  document: {
    querySelector() { return panel; },
    getElementById(id) { return elements.get(id) || null; }
  },
  console,
  BRIDGE_MANAGED: wasmModule.settingsBridgeManaged(),
  BRIDGE_REQUIRED: new Set(wasmModule.settingsBridgeRequired()),
  BRIDGE_OPTIONAL: new Set(wasmModule.settingsBridgeOptional()),
  bridgeFieldEnabled: (name, values, options) =>
    wasmModule.settingsBridgeFieldEnabled(name, values, options),
  wasmBridgeNetworkProfiles: wasmModule.bridgeNetworkProfiles,
  wasmBridgeNetworkProfile: wasmModule.bridgeNetworkProfile,
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
  "({ parse: bridgeInstanceParseStructured, effective: kgwBridgeEffectiveSettingsV1 })",
  sandbox
);

const directInstanceCheckbox = wasmModule.bridgeInstanceCommandCheckboxFromInstancesR13B(
  sandbox.bridgeInstances,
  "mainnet",
  "one",
  "instanceDiff"
);
const output = {
  directOwners: {
    hasConfigInitially: wasmModule.bridgeHasConfig("mainnet"),
    inlineEnabled: wasmModule.bridgeCommandOptionEnabledR7("mainnet", "coinbaseTagSuffix"),
    inlineToggleHasMarker: wasmModule.bridgeCommandInlineToggleR7("mainnet", "coinbaseTagSuffix").includes('data-bridge-command-option-toggle-r7="coinbaseTagSuffix"'),
    instanceShouldInclude: wasmModule.bridgeInstanceCommandShouldIncludeFromInstancesR13B(
      sandbox.bridgeInstances,
      "mainnet",
      "one",
      "instanceDiff"
    ),
    instanceCheckboxHasId: directInstanceCheckbox.includes('data-instance-id="one"'),
    instanceCheckboxChecked: directInstanceCheckbox.includes("checked")
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
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge command-option wrapper remains in JavaScript: {forbidden}"
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
