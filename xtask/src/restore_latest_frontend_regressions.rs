use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const SETTINGS_SOURCE: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js";
const RESTORE_START: &str = "// AUD-001: one restore flight";
const RESTORE_END: &str = "function kgwInstallSettingsDbMaintenanceActions";
const RENDER_START: &str = "async function kgwRenderSettingsAddressRows(";
const RENDER_END: &str = "async function kgwRefreshSettingsAddresses";

const NODE_BRIDGE: &str = r##"
const fs = require("node:fs");
const vm = require("node:vm");
const [restorePath, renderPath, requestPath, resultPath] = process.argv.slice(2);
const restoreSource = fs.readFileSync(restorePath, "utf8");
const renderSource = fs.readFileSync(renderPath, "utf8");
const request = JSON.parse(fs.readFileSync(requestPath, "utf8"));

function harness(mode, consent = true) {
  const nodes = new Map();
  const events = [];
  const controls = request.initialControls.map(disabled => ({ disabled }));
  let restoreCalls = 0;
  let releaseBackend;
  let releaseRender;
  const backendPending = mode === "happy"
    ? new Promise(resolve => { releaseBackend = resolve; })
    : null;
  const renderPending = mode === "happy"
    ? new Promise(resolve => { releaseRender = resolve; })
    : null;
  const invoke = mode === "missing-invoke" ? null : async command => {
    if (command === "get_all_addresses") {
      if (mode === "read-error") throw new Error("fresh read failed");
      return request.addressRecords;
    }
    restoreCalls += 1;
    if (mode === "happy") return backendPending;
    if (mode === "native-error") {
      throw new Error("target changed; current data preserved");
    }
    if (mode === "ok-false") return { ...request.good, ok: false };
    if (mode === "missing-target") return { ...request.good, backup_path: null };
    if (mode === "bad-rows") return { ...request.good, rows: null };
    return request.good;
  };
  const context = {
    console: { log() {}, error() {} },
    document: {
      getElementById: id => nodes.get(id),
      createElement: () => ({
        dataset: {},
        setAttribute(key, value) { this[key] = value; }
      }),
      querySelector: () => ({
        appendChild(node) { nodes.set(node.id, node); }
      }),
      querySelectorAll: () => controls
    },
    KGW_SETTINGS_DB_ACTION_STATE: { restoring: false },
    KGW_SETTINGS_ADDRESS_STATE: { restoreEpoch: 0 },
    confirm: () => consent,
    applyStatusTone() {},
    kgwSettingsDbInvoke: () => invoke,
    kgwSettingsDbStatus: message => events.push(message),
    kgwRenderSettingsDatabaseRows: () => events.push("database rows"),
    kgwRenderSettingsAddressRows: async (_rows, options) => {
      events.push("address rows");
      if (!options || options.localOnly !== true) {
        throw new Error("restore must render address rows local-only");
      }
      if (mode === "render-error") return false;
      if (mode === "happy") return renderPending;
      return true;
    },
    kgwClearSettingsAddressFields() {},
    kgwSettingsAddressSetStatus() {},
    kgwSettingsAddressNow: () => "test-time"
  };
  vm.createContext(context);
  vm.runInContext(restoreSource, context);
  return {
    context,
    events,
    controls,
    run: () => context.kgwSettingsRestoreLatest(),
    status: () => nodes.get("settingsRestoreStatus"),
    restoreCalls: () => restoreCalls,
    releaseBackend,
    releaseRender
  };
}

async function runHappy() {
  const h = harness("happy");
  const first = h.run();
  const runningState = h.status()?.dataset?.state ?? null;
  const controlsDuring = h.controls.map(node => node.disabled);
  const second = await h.run();
  const callsAfterRepeat = h.restoreCalls();
  h.releaseBackend(request.good);
  await new Promise(setImmediate);
  const afterBackendState = h.status()?.dataset?.state ?? null;
  h.releaseRender(true);
  const result = await first;
  return {
    runningState,
    controlsDuring,
    second,
    callsAfterRepeat,
    afterBackendState,
    resultBackup: result?.backup_path ?? null,
    finalState: h.status()?.dataset?.state ?? null,
    events: h.events,
    controlsAfter: h.controls.map(node => node.disabled),
    restoring: h.context.KGW_SETTINGS_DB_ACTION_STATE.restoring
  };
}

async function runFailure(mode) {
  const h = harness(mode);
  const result = await h.run();
  const status = h.status();
  return {
    mode,
    result,
    state: status?.dataset?.state ?? null,
    role: status?.role ?? null,
    restoring: h.context.KGW_SETTINGS_DB_ACTION_STATE.restoring,
    controls: h.controls.map(node => node.disabled)
  };
}

async function runCancelled() {
  const h = harness("happy", false);
  const result = await h.run();
  return {
    result,
    state: h.status()?.dataset?.state ?? null,
    restoring: h.context.KGW_SETTINGS_DB_ACTION_STATE.restoring,
    controls: h.controls.map(node => node.disabled),
    restoreCalls: h.restoreCalls()
  };
}

async function runStaleRender() {
  let releaseOld;
  const rows = { innerHTML: "restored OLD" };
  const state = { restoreEpoch: 0 };
  const context = {
    KGW_SETTINGS_ADDRESS_STATE: state,
    kgwSettingsAddressElements: () => ({ rows }),
    kgwEnrichSettingsAddressRows: () =>
      new Promise(resolve => { releaseOld = resolve; })
  };
  vm.createContext(context);
  vm.runInContext(renderSource, context);
  const oldRender = context.kgwRenderSettingsAddressRows(request.addressRecords);
  state.restoreEpoch += 1;
  releaseOld(request.addressRecords);
  return { result: await oldRender, html: rows.innerHTML };
}

(async () => {
  const failures = [];
  for (const mode of request.failureModes) {
    failures.push(await runFailure(mode));
  }
  const output = {
    happy: await runHappy(),
    failures,
    cancelled: await runCancelled(),
    stale: await runStaleRender()
  };
  fs.writeFileSync(resultPath, JSON.stringify(output), "utf8");
})().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
"##;

fn slice_between<'a>(
    source: &'a str,
    start: &str,
    end: &str,
    label: &str,
) -> Result<&'a str, String> {
    let start_index = source
        .find(start)
        .ok_or_else(|| format!("{label} start marker missing: {start}"))?;
    let relative_end = source[start_index..]
        .find(end)
        .ok_or_else(|| format!("{label} end marker missing: {end}"))?;
    let end_index = start_index + relative_end;
    if end_index <= start_index {
        return Err(format!("{label} markers reversed: {start} -> {end}"));
    }
    Ok(&source[start_index..end_index])
}

fn request() -> Value {
    json!({
        "initialControls": [false, true],
        "good": {
            "ok": true,
            "backup_path": "audit/OLD",
            "rows": [],
            "message": "Restored verified OLD"
        },
        "addressRecords": [{"name": "OLD"}],
        "failureModes": [
            "native-error",
            "ok-false",
            "missing-target",
            "bad-rows",
            "read-error",
            "render-error",
            "missing-invoke"
        ]
    })
}

fn run_bridge(root: &Path, restore_source: &str, render_source: &str) -> Result<Value, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-restore-latest-")
        .tempdir()
        .map_err(|error| format!("failed to create Restore regression tempdir: {error}"))?;
    let bridge_path = temp.path().join("bridge.cjs");
    let restore_path = temp.path().join("restore.js");
    let render_path = temp.path().join("render.js");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write Restore Node bridge: {error}"))?;
    fs::write(&restore_path, restore_source.as_bytes())
        .map_err(|error| format!("failed to write Restore owner slice: {error}"))?;
    fs::write(&render_path, render_source.as_bytes())
        .map_err(|error| format!("failed to write Restore render slice: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(&request())
            .map_err(|error| format!("failed to serialize Restore request: {error}"))?,
    )
    .map_err(|error| format!("failed to write Restore request: {error}"))?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&restore_path)
        .arg(&render_path)
        .arg(&request_path)
        .arg(&result_path)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch Restore Node bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Restore Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read Restore result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse Restore result: {error}"))
}

fn expect(actual: &Value, pointer: &str, expected: Value) -> Result<(), String> {
    let value = actual
        .pointer(pointer)
        .ok_or_else(|| format!("Restore result missing {pointer}"))?;
    if value != &expected {
        return Err(format!(
            "Restore regression mismatch at {pointer}: expected {expected}, got {value}"
        ));
    }
    Ok(())
}

pub fn run(root: &Path) -> Result<String, String> {
    let source = fs::read_to_string(root.join(SETTINGS_SOURCE))
        .map_err(|error| format!("failed to read {SETTINGS_SOURCE}: {error}"))?;
    if !source.contains("await kgwSettingsRestoreLatest();") {
        return Err("real Restore button no longer calls guarded owner".to_owned());
    }
    let restore_source = slice_between(&source, RESTORE_START, RESTORE_END, "Restore owner")?;
    let render_source = slice_between(&source, RENDER_START, RENDER_END, "Restore render")?;
    let actual = run_bridge(root, restore_source, render_source)?;

    expect(&actual, "/happy/runningState", json!("running"))?;
    expect(&actual, "/happy/controlsDuring", json!([true, true]))?;
    expect(&actual, "/happy/second", Value::Null)?;
    expect(&actual, "/happy/callsAfterRepeat", json!(1))?;
    expect(&actual, "/happy/afterBackendState", json!("running"))?;
    expect(&actual, "/happy/resultBackup", json!("audit/OLD"))?;
    expect(&actual, "/happy/finalState", json!("success"))?;
    expect(&actual, "/happy/controlsAfter", json!([false, true]))?;
    expect(&actual, "/happy/restoring", json!(false))?;
    let events = actual["happy"]["events"]
        .as_array()
        .ok_or_else(|| "Restore happy events missing".to_owned())?;
    let address_index = events
        .iter()
        .position(|value| value == "address rows")
        .ok_or_else(|| "Restore happy address-row event missing".to_owned())?;
    let success_index = events
        .iter()
        .position(|value| value == "Restored verified OLD")
        .ok_or_else(|| "Restore happy success event missing".to_owned())?;
    if address_index >= success_index {
        return Err("Restore success became visible before address rendering".to_owned());
    }

    let failures = actual["failures"]
        .as_array()
        .ok_or_else(|| "Restore failures array missing".to_owned())?;
    let modes = request()["failureModes"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if failures.len() != modes.len() {
        return Err(format!(
            "Restore failure matrix length mismatch: expected {}, got {}",
            modes.len(),
            failures.len()
        ));
    }
    for (index, mode) in modes.iter().enumerate() {
        let base = format!("/failures/{index}");
        expect(&actual, &(base.clone() + "/mode"), mode.clone())?;
        expect(&actual, &(base.clone() + "/result"), Value::Null)?;
        expect(&actual, &(base.clone() + "/state"), json!("error"))?;
        expect(&actual, &(base.clone() + "/role"), json!("alert"))?;
        expect(&actual, &(base.clone() + "/restoring"), json!(false))?;
        expect(&actual, &(base + "/controls"), json!([false, true]))?;
    }

    expect(&actual, "/cancelled/result", Value::Null)?;
    expect(&actual, "/cancelled/state", json!("cancelled"))?;
    expect(&actual, "/cancelled/restoring", json!(false))?;
    expect(&actual, "/cancelled/controls", json!([false, true]))?;
    expect(&actual, "/cancelled/restoreCalls", json!(0))?;
    expect(&actual, "/stale/result", json!(false))?;
    expect(&actual, "/stale/html", json!("restored OLD"))?;

    Ok("AUD-001 Restore frontend Rust owner PASSED \
(single-flight, ordered success, seven failures, cancellation, stale refresh)"
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_markers_fail_closed() {
        assert!(slice_between("missing", "a", "b", "test").is_err());
        assert_eq!(
            slice_between("x START body END y", "START", "END", "test").unwrap(),
            "START body "
        );
    }

    #[test]
    fn rust_owns_seven_failure_modes() {
        assert_eq!(request()["failureModes"].as_array().map(Vec::len), Some(7));
    }
}
