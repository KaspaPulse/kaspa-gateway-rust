use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const BRIDGE_SOURCE: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const BRIDGE_HELPERS_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_frontend_helpers.rs";
const BRIDGE_RUNTIME_CORE_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_runtime_core.rs";
const BRIDGE_START_TRACE_SOURCE: &str =
    "crates/kaspa-gateway-frontend-wasm/src/bridge_start_trace.rs";
const WASM_JS: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm.js";
const WASM_BIN: &str =
    "apps/kaspa-gateway-desktop/frontend/generated/kgw_frontend_wasm/kgw_frontend_wasm_bg.wasm";

const SLICES: &[(&str, &str)] = &[
    (
        "const BRIDGE_NETWORKS = wasmBridgeNetworkProfiles();",
        "const bridgeInstances = {",
    ),
    (
        "function kgwBridgeSetRuntimeErrorV1(",
        "async function kgwBridgeV7BlockInprocessIfNodeOwnerRunning",
    ),
    (
        "const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();",
        "/* KGW_BRIDGE_START_TRACE_V1 is Rust-owned in bridge_start_trace.rs. */",
    ),
    (
        "async function runBridgeIntegratedAction(",
        "/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */",
    ),
    (
        "function kgwBridgeR51SetRuntimeButtons(",
        "function kgwBridgeR51MaybeActivityNotice(",
    ),
];

const NODE_BRIDGE: &str = r##"
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";
import vm from "node:vm";

const [selectedPath, requestPath, resultPath, wasmJsPath, wasmPath] = process.argv.slice(2);
const selected = await readFile(selectedPath, "utf8");
const request = JSON.parse(await readFile(requestPath, "utf8"));
const wasm = await import(pathToFileURL(wasmJsPath).href);
await wasm.default({ module_or_path: await readFile(wasmPath) });

globalThis.window = globalThis;
const transportCalls = [];
globalThis.__TAURI__ = {
  core: {
    invoke: async (command, args) => {
      transportCalls.push({ command: String(command || ""), network: String(args?.network || "") });
      return "transport-ok";
    }
  }
};
const transportAvailable = wasm.bridgeRuntimeInvokeAvailable();
const transportResult = await wasm.bridgeInvokeRuntimeCommand(
  "kgw_kgw_runtime_logs_v1",
  { network: "mainnet", runtimeRole: "bridge" }
);

class NodeLike {
  constructor() {
    this.children = [];
    this.textContent = "";
    this.className = "";
  }
  appendChild(node) {
    this.children.push(node);
    return node;
  }
  querySelector(selector) {
    if (selector === ".kgw-status-value") {
      const stack = [...this.children];
      while (stack.length) {
        const node = stack.shift();
        if (node?.className === "kgw-status-value") return node;
        if (Array.isArray(node?.children)) stack.push(...node.children);
      }
    }
    return null;
  }
}
class Element extends NodeLike {
  constructor() {
    super();
    this.hidden = false;
    this.disabled = false;
    this.style = {};
    this.title = "";
    this.dataset = {};
    this.value = "";
    this.attributes = {};
    this.classList = {
      contains() { return false; },
      toggle() {},
      add() {},
      remove() {}
    };
  }
  setAttribute(key, value) {
    this.attributes[key] = String(value);
    this[key] = String(value);
  }
  removeAttribute(key) {
    delete this.attributes[key];
  }
  replaceChildren(...nodes) {
    this.children = nodes;
  }
}
const elements = new Map();
for (const name of [
  "runtimeError",
  "runtimeStatus",
  "policyStatus",
  "monitorState",
  "logEmpty",
  "settingsAuthority",
  "previewStatus",
  "commandPreview"
]) {
  elements.set("bridge-mainnet-" + name, new Element());
}
const start = new Element();
const stop = new Element();
const next = new Element();
const panel = {
  querySelector(selector) {
    if (selector.includes('data-bridge-action="start"')) return start;
    if (selector.includes('data-bridge-action="stop"')) return stop;
    if (selector.includes('data-bridge-action="monitor-next"')) return next;
    return null;
  }
};
const documentImpl = {
  getElementById(id) {
    return elements.get(id) || null;
  },
  querySelector(selector) {
    return selector.includes('data-bridge-network-panel="mainnet"') ? panel : null;
  },
  createDocumentFragment() {
    return new NodeLike();
  },
  createTextNode(text) {
    const node = new NodeLike();
    node.textContent = String(text);
    return node;
  },
  createElement() {
    return new Element();
  }
};
globalThis.document = documentImpl;

const local = new Map([["kgw.bridge.network.enabled.mainnet", "1"]]);
let invokeRuntime = async () => "";
const sandbox = {
  document: documentImpl,
  localStorage: {
    getItem(key) {
      return local.get(String(key)) || null;
    }
  },
  console,
  setTimeout,
  clearTimeout,
  CustomEvent: class CustomEvent {},
  runtimePresentation: wasm.runtimePresentation,
  runtimeObservationSummary: wasm.runtimeObservationSummary,
  applyStatusTone: wasm.applyStatusTone,
  renderStatusSummary: wasm.renderStatusSummary,
  wasmBridgeNetworkProfiles: wasm.bridgeNetworkProfiles,
  wasmBridgeNetworkProfile: wasm.bridgeNetworkProfile,
  wasmBridgeNetworkEnabled: wasm.bridgeNetworkEnabled,
  wasmBridgeById: wasm.bridgeById,
  wasmBridgeElementId: wasm.bridgeElementId,
  wasmBridgeChecked: wasm.bridgeChecked,
  wasmBridgeStringifyRuntimeResult: wasm.bridgeStringifyRuntimeResult,
  wasmBridgeNormalizeRuntimeError: wasm.bridgeNormalizeRuntimeError,
  wasmBridgeParseRuntimeKeyValueResponse: wasm.bridgeParseRuntimeKeyValueResponse,
  wasmBridgeV7RuntimeRunningFromText: wasm.bridgeV7RuntimeRunningFromText,
  wasmBridgeR51IsRunning: wasm.bridgeR51IsRunning,
  wasmBridgeRuntimeErrorFromStatus: wasm.bridgeRuntimeErrorFromStatus,
  wasmBridgeNormalizeNodeModeR65F: wasm.bridgeNormalizeNodeModeR65F,
  wasmBridgePreviewDeclaresInprocessR65F: wasm.bridgePreviewDeclaresInprocessR65F,
  wasmBridgeRuntimeCommandForAction: wasm.bridgeRuntimeCommandForAction,
  wasmBridgeRuntimeActionOutcome: wasm.bridgeRuntimeActionOutcome,
  wasmBridgeStartWasInprocessR65F: wasm.bridgeStartWasInprocessR65F
};
sandbox.window = sandbox;
sandbox.globalThis = sandbox;
sandbox.kgwBridgeR51Panel = () => panel;
sandbox.updateCommand = () => "--node-mode=external";
sandbox.bridgeAssertNoPortConflictsR5 = () => ({ ok: true });
sandbox.kgwBridgeValidateForm = () => ({});
sandbox.c = () => false;
sandbox.confirmUserAction = async () => true;
sandbox.kgwBridgeV7BlockInprocessIfNodeOwnerRunning = async () => false;
sandbox.invokeBridgeIntegratedRuntime = (...args) => invokeRuntime(...args);
sandbox.kgwBridgeRuntimeOwnerTraceR64D = () => {};
sandbox.kgwSetBridgeOwnedNodeLockR65E = () => {};
sandbox.kgwBridgeCurrentNodeModeFromUiR65F = () => "external";
sandbox.kgwBridgePreviewDeclaresInprocessR65F = () => false;
sandbox.kgwBridgeR51KickRawLogLiveR134E = () => {};
sandbox.kgwBridgeTranslateRuntime = (_key, fallback) => fallback;
sandbox.bridgeNodeMode = () => "external";

vm.createContext(sandbox);
vm.runInContext(selected, sandbox, { filename: request.sourceName });
const api = vm.runInContext("({ runtimeRunning: wasmBridgeV7RuntimeRunningFromText, isRunning: wasmBridgeR51IsRunning, setButtons: kgwBridgeR51SetRuntimeButtons, setError: kgwBridgeSetRuntimeErrorV1, setActivity: kgwBridgeSetRuntimeActivityV1, runAction: runBridgeIntegratedAction })", sandbox);

function snapshot() {
  return {
    policy: elements.get("bridge-mainnet-policyStatus").textContent,
    policyState: elements.get("bridge-mainnet-policyStatus").dataset.state || "",
    startDisabled: Boolean(start.disabled),
    stopDisabled: Boolean(stop.disabled),
    runtimeError: elements.get("bridge-mainnet-runtimeError").textContent,
    runtimeErrorHidden: Boolean(elements.get("bridge-mainnet-runtimeError").hidden),
    runtimeStatus: elements.get("bridge-mainnet-runtimeStatus").textContent
  };
}

async function actionLifecycle() {
  const result = {};

  let resolveStart;
  invokeRuntime = async () => await new Promise(resolve => { resolveStart = resolve; });
  const startPromise = api.runAction("start", "mainnet");
  await Promise.resolve();
  await Promise.resolve();
  await new Promise(resolve => setImmediate(resolve));
  result.startPending = snapshot();
  resolveStart(request.startReady);
  await startPromise;
  result.startReady = snapshot();

  invokeRuntime = async () => { throw new Error(request.startError); };
  await api.runAction("start", "mainnet");
  result.startFailed = snapshot();

  let resolveStop;
  invokeRuntime = async () => await new Promise(resolve => { resolveStop = resolve; });
  api.setButtons("mainnet", true);
  const stopPromise = api.runAction("stop", "mainnet");
  await Promise.resolve();
  await new Promise(resolve => setImmediate(resolve));
  result.stopPending = snapshot();
  resolveStop(request.stopGraceful);
  await stopPromise;
  result.stopGraceful = snapshot();

  invokeRuntime = async () => request.stopForced;
  api.setButtons("mainnet", true);
  await api.runAction("stop", "mainnet");
  result.stopForced = snapshot();

  invokeRuntime = async () => request.stopFailed;
  api.setButtons("mainnet", true);
  await api.runAction("stop", "mainnet");
  result.stopFailed = snapshot();

  return result;
}

api.setButtons("mainnet", false, "starting");
const starting = snapshot();
api.setButtons("mainnet", true, "stopping");
const stopping = snapshot();
api.setButtons("mainnet", false);
api.setError("mainnet", "occupied listener port");
api.setActivity("mainnet", "Bridge start failed.");
const visibleFailure = snapshot();

const output = {
  transport: {
    available: Boolean(transportAvailable),
    result: String(transportResult || ""),
    calls: transportCalls
  },
  runtimeRunning: {
    liveOnly: api.runtimeRunning("role=node;network=mainnet;running=true"),
    ready: api.runtimeRunning("role=node;network=mainnet;running=true;readiness=READY")
  },
  bridgeRunning: {
    liveOnly: api.isRunning("role=bridge;network=mainnet;running=true"),
    ready: api.isRunning("role=bridge;network=mainnet;running=true;readiness=READY")
  },
  starting,
  stopping,
  visibleFailure,
  lifecycle: await actionLifecycle()
};
await writeFile(resultPath, JSON.stringify(output), "utf8");
"##;

fn slice_between<'a>(source: &'a str, start: &str, end: &str) -> Result<&'a str, String> {
    let start_index = source
        .find(start)
        .ok_or_else(|| format!("Bridge readiness source start marker missing: {start}"))?;
    let relative_end = source[start_index..]
        .find(end)
        .ok_or_else(|| format!("Bridge readiness source end marker missing: {end}"))?;
    let end_index = start_index + relative_end;
    if end_index <= start_index {
        return Err(format!(
            "Bridge readiness source markers reversed: {start} -> {end}"
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
fn request() -> Value {
    json!({
        "sourceName": BRIDGE_SOURCE,
        "startReady":
            "parallel-owned-self-worker started;role=bridge;network=mainnet;pid=42;running=true;readiness=READY",
        "startError": "occupied listener port",
        "stopGraceful":
            "parallel-owned-self-worker stopped;role=bridge;network=mainnet;running=false;graceful=true;forced=false",
        "stopForced":
            "parallel-owned-self-worker stopped;role=bridge;network=mainnet;running=false;graceful=false;forced=true;reason=graceful stop timed out",
        "stopFailed":
            "parallel-owned-self-worker stopped with graceful failure;role=bridge;network=mainnet;running=false;graceful=false;forced=false;stop_failed=true;stop_outcome=FAILED;reason=official shutdown failed"
    })
}

fn run_bridge(root: &Path, selected: &str) -> Result<Value, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-bridge-readiness-")
        .tempdir()
        .map_err(|error| format!("failed to create Bridge readiness tempdir: {error}"))?;
    let bridge_path = temp.path().join("bridge.mjs");
    let selected_path = temp.path().join("selected.js");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write Bridge readiness Node bridge: {error}"))?;
    fs::write(&selected_path, selected.as_bytes())
        .map_err(|error| format!("failed to write Bridge readiness selected source: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(&request())
            .map_err(|error| format!("failed to serialize Bridge readiness request: {error}"))?,
    )
    .map_err(|error| format!("failed to write Bridge readiness request: {error}"))?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&selected_path)
        .arg(&request_path)
        .arg(&result_path)
        .arg(root.join(WASM_JS))
        .arg(root.join(WASM_BIN))
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch Bridge readiness Node bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Bridge readiness Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read Bridge readiness result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse Bridge readiness result: {error}"))
}

fn expect(actual: &Value, pointer: &str, expected: Value) -> Result<(), String> {
    let value = actual
        .pointer(pointer)
        .ok_or_else(|| format!("Bridge readiness result missing {pointer}"))?;
    if value != &expected {
        return Err(format!(
            "Bridge readiness mismatch at {pointer}: expected {expected}, got {value}"
        ));
    }
    Ok(())
}

fn verify_static_contracts(
    source: &str,
    helpers: &str,
    runtime_core: &str,
    start_trace: &str,
) -> Result<(), String> {
    for needle in [
        "wasmBridgeRuntimeErrorFromStatus(status)",
        "kgwBridgeSetRuntimeActivityV1(net, \"Bridge runtime failed after readiness.\", \"failed\")",
        "Tauri invoke resolution and timeout policy are Rust-owned in bridge_start_trace.rs.",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "Bridge readiness JavaScript contract missing: {needle}"
            ));
        }
    }
    for needle in [
        "experimental: true",
        "enabled_by_default: false",
        "requires explicit opt-in",
    ] {
        if !helpers.contains(needle) {
            return Err(format!(
                "Bridge readiness Rust owner contract missing: {needle}"
            ));
        }
    }

    for needle in [
        "bridgeStringifyRuntimeResult",
        "bridgeNormalizeRuntimeError",
        "bridgeParseRuntimeKeyValueResponse",
        "bridgeV7RuntimeRunningFromText",
        "bridgeR51IsRunning",
        "bridgeRuntimeErrorFromStatus",
        "bridgeNormalizeNodeModeR65F",
        "bridgePreviewDeclaresInprocessR65F",
        "bridgeRuntimeCommandForAction",
        "bridgeRuntimeActionOutcome",
        "bridgeStartWasInprocessR65F",
    ] {
        if !runtime_core.contains(needle) {
            return Err(format!("Bridge runtime-core Rust export missing: {needle}"));
        }
    }

    for forbidden in [
        "function stringifyRuntimeResult(",
        "function normalizeRuntimeError(",
        "function parseRuntimeKeyValueResponse(",
        "function kgwBridgeV7RuntimeRunningFromText(",
        "function kgwBridgeNormalizeNodeModeR65F(",
        "function kgwBridgePreviewDeclaresInprocessR65F(",
        "function kgwBridgeStartWasInprocessR65F(",
        "function kgwBridgeR51IsRunning(",
        "function kgwBridgeRuntimeErrorFromStatus(",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge runtime parser remains in JavaScript: {forbidden}"
            ));
        }
    }

    for needle in [
        "bridgeRuntimeInvokeAvailable",
        "bridgeInvokeRuntimeCommand",
        "BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS",
        "BRIDGE_PREVIEW_INVOKE_TIMEOUT_MS",
    ] {
        if !start_trace.contains(needle) {
            return Err(format!(
                "Bridge start-trace Rust transport export missing: {needle}"
            ));
        }
    }

    for forbidden in [
        "function getTauriInvoke(",
        "function invokeWithTimeout(",
        "const KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS =",
        "const KGW_BRIDGE_STOP_INVOKE_TIMEOUT_MS =",
    ] {
        if source.contains(forbidden) {
            return Err(format!(
                "Retired Bridge invoke transport remains in JavaScript: {forbidden}"
            ));
        }
    }

    for line in source.lines() {
        let lower = line.to_ascii_lowercase();
        if !lower.contains("appendlog(") {
            continue;
        }
        if [
            "ready",
            "readiness",
            "start failed",
            "start response",
            "start confirmed",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        {
            return Err(format!(
                "typed readiness diagnostic leaked into raw Bridge log pane: {line}"
            ));
        }
        if ["forced", "graceful", "stop_outcome", "stopping"]
            .iter()
            .any(|needle| lower.contains(needle))
        {
            return Err(format!(
                "Stop control diagnostic leaked into raw Bridge log pane: {line}"
            ));
        }
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<String, String> {
    let full_source = fs::read_to_string(root.join(BRIDGE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_SOURCE}: {error}"))?;
    let helper_source = fs::read_to_string(root.join(BRIDGE_HELPERS_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_HELPERS_SOURCE}: {error}"))?;
    let runtime_core_source = fs::read_to_string(root.join(BRIDGE_RUNTIME_CORE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_RUNTIME_CORE_SOURCE}: {error}"))?;
    let start_trace_source = fs::read_to_string(root.join(BRIDGE_START_TRACE_SOURCE))
        .map_err(|error| format!("failed to read {BRIDGE_START_TRACE_SOURCE}: {error}"))?;
    verify_static_contracts(
        &full_source,
        &helper_source,
        &runtime_core_source,
        &start_trace_source,
    )?;
    let selected = selected_source(root)?;
    let actual = run_bridge(root, &selected)?;

    expect(&actual, "/transport/available", json!(true))?;
    expect(&actual, "/transport/result", json!("transport-ok"))?;
    expect(
        &actual,
        "/transport/calls/0/command",
        json!("kgw_kgw_runtime_logs_v1"),
    )?;
    expect(&actual, "/transport/calls/0/network", json!("mainnet"))?;

    expect(&actual, "/runtimeRunning/liveOnly", json!(false))?;
    expect(&actual, "/runtimeRunning/ready", json!(true))?;
    expect(&actual, "/bridgeRunning/liveOnly", json!(false))?;
    expect(&actual, "/bridgeRunning/ready", json!(true))?;

    expect(&actual, "/starting/policy", json!("Bridge: Starting"))?;
    expect(&actual, "/starting/startDisabled", json!(true))?;
    expect(&actual, "/starting/stopDisabled", json!(true))?;
    expect(&actual, "/stopping/policy", json!("Bridge: Stopping"))?;
    expect(&actual, "/stopping/startDisabled", json!(true))?;
    expect(&actual, "/stopping/stopDisabled", json!(true))?;

    expect(&actual, "/visibleFailure/policy", json!("Bridge: Stopped"))?;
    expect(
        &actual,
        "/visibleFailure/runtimeError",
        json!("occupied listener port"),
    )?;
    expect(&actual, "/visibleFailure/runtimeErrorHidden", json!(false))?;
    expect(
        &actual,
        "/visibleFailure/runtimeStatus",
        json!("Bridge start failed."),
    )?;

    expect(
        &actual,
        "/lifecycle/startPending/policy",
        json!("Bridge: Starting"),
    )?;
    expect(
        &actual,
        "/lifecycle/startPending/startDisabled",
        json!(true),
    )?;
    expect(
        &actual,
        "/lifecycle/startReady/policy",
        json!("Bridge: Running"),
    )?;
    expect(&actual, "/lifecycle/startReady/startDisabled", json!(true))?;
    expect(
        &actual,
        "/lifecycle/startReady/runtimeStatus",
        json!("Bridge READY attestation confirmed."),
    )?;

    expect(
        &actual,
        "/lifecycle/startFailed/policy",
        json!("Reconciling"),
    )?;
    expect(
        &actual,
        "/lifecycle/startFailed/runtimeError",
        json!("occupied listener port"),
    )?;
    expect(
        &actual,
        "/lifecycle/stopPending/policy",
        json!("Bridge: Stopping"),
    )?;
    expect(
        &actual,
        "/lifecycle/stopGraceful/policy",
        json!("Bridge: Stopped"),
    )?;
    expect(
        &actual,
        "/lifecycle/stopGraceful/runtimeStatus",
        json!("Bridge graceful official shutdown confirmed."),
    )?;
    let forced_error = actual
        .pointer("/lifecycle/stopForced/runtimeError")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !forced_error.contains("FORCED") {
        return Err(format!(
            "Bridge forced-stop error lost FORCED classification: {forced_error}"
        ));
    }
    expect(
        &actual,
        "/lifecycle/stopForced/runtimeStatus",
        json!("Bridge FORCED termination confirmed."),
    )?;
    let failed_error = actual
        .pointer("/lifecycle/stopFailed/runtimeError")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !failed_error.contains("Official graceful shutdown failed") {
        return Err(format!(
            "Bridge graceful-stop failure message lost classification: {failed_error}"
        ));
    }
    expect(
        &actual,
        "/lifecycle/stopFailed/runtimeStatus",
        json!("Bridge worker exited after graceful shutdown failure."),
    )?;

    Ok("Bridge readiness frontend Rust owner PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_markers_fail_closed() {
        assert!(slice_between("missing", "start", "end").is_err());
        assert_eq!(
            slice_between("prefix START body END tail", "START", "END").unwrap(),
            "START body "
        );
    }

    #[test]
    fn rust_owns_start_and_stop_scenarios() {
        let request = request();
        assert!(
            request["startReady"]
                .as_str()
                .unwrap()
                .contains("readiness=READY")
        );
        assert!(
            request["stopForced"]
                .as_str()
                .unwrap()
                .contains("forced=true")
        );
        assert!(
            request["stopFailed"]
                .as_str()
                .unwrap()
                .contains("stop_failed=true")
        );
    }
}
