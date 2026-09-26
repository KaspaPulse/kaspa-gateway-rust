use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const MAIN_RELATIVE: &str = "apps/kaspa-gateway-desktop/frontend/main.js";
const START_MARKER: &str = "let kgwShellPendingSavedMainTabR102C = \"\";";
const END_MARKER: &str = "async function openTab(tabId, options = {})";

const NODE_BRIDGE: &str = r##"
const fs = require("node:fs");
const vm = require("node:vm");
const [guardPath, requestPath, resultPath] = process.argv.slice(2);
const guardSource = fs.readFileSync(guardPath, "utf8");
const request = JSON.parse(fs.readFileSync(requestPath, "utf8"));

function runScenario(scenario) {
  let active = scenario.active;
  const timers = [];
  const opens = [];
  const vars = new Map();
  const context = {
    KGW_TABS: ["explorer","kaspa-node","kaspa-bridge","analysis","top-addresses","log","settings"].map(id => ({ id })),
    kgwShellReadLastMainTabR101W2: () => scenario.saved,
    kgwShellDisplayOwnerActiveButtonR59C: () => ({ dataset: { tab: active } }),
    kgwShellDisplayOwnerTabIdVisibleR59C: () => true,
    kgwMainTabTraceR35C: () => {},
    openTab: (tab, options) => { opens.push({ tab, options }); active = tab; return Promise.resolve(true); },
    window: {
      location: { hash: "#" + active },
      setTimeout: (fn, delay) => { timers.push({ fn, delay }); return timers.length; }
    },
    String, Number, Array, Boolean, console
  };
  vm.createContext(context);
  vm.runInContext(guardSource, context);
  const api = vm.runInContext(`({
    schedule: kgwShellScheduleSavedMainTabRestoreR102C,
    record: kgwShellRecordExplicitNavigationR103,
    current: kgwShellExplicitNavigationIsCurrentR103
  })`, context);
  const results = [];
  for (const op of scenario.ops) {
    if (op.kind === "schedule") {
      results.push({ kind: op.kind, value: api.schedule(op.reason), timers: timers.length });
    } else if (op.kind === "record") {
      vars.set(op.id, api.record(op.tab));
      results.push({ kind: op.kind, id: op.id });
    } else if (op.kind === "set-active") {
      active = op.tab;
      context.window.location.hash = "#" + op.tab;
    } else if (op.kind === "run-all") {
      for (const timer of timers) timer.fn();
      results.push({ kind: op.kind, opens: opens.map(entry => entry.tab) });
    } else if (op.kind === "run-first") {
      if (timers[0]) timers[0].fn();
      results.push({ kind: op.kind, opens: opens.map(entry => entry.tab) });
    } else if (op.kind === "current") {
      results.push({ kind: op.kind, id: op.id, value: api.current(vars.get(op.id)) });
    } else {
      throw new Error("unknown AUD-013 operation: " + op.kind);
    }
  }
  return { results };
}

const output = { scenarios: request.scenarios.map(runScenario) };
fs.writeFileSync(resultPath, JSON.stringify(output), "utf8");
"##;

fn guard_source(source: &str) -> Result<&str, String> {
    let start = source
        .find(START_MARKER)
        .ok_or_else(|| "AUD-013 restore guard start marker missing".to_owned())?;
    let end = source
        .find(END_MARKER)
        .ok_or_else(|| "AUD-013 restore guard end marker missing".to_owned())?;
    if end <= start {
        return Err("AUD-013 restore guard markers are reversed".to_owned());
    }
    Ok(&source[start..end])
}
fn request() -> Value {
    json!({
        "scenarios": [
            {
                "saved": "kaspa-bridge",
                "active": "kaspa-node",
                "ops": [
                    {"kind": "schedule", "reason": "boot-after-open-tab"},
                    {"kind": "record", "id": "explicit", "tab": "settings"},
                    {"kind": "set-active", "tab": "settings"},
                    {"kind": "run-all"},
                    {"kind": "current", "id": "explicit"}
                ]
            },
            {
                "saved": "kaspa-bridge",
                "active": "kaspa-node",
                "ops": [
                    {"kind": "schedule", "reason": "boot-after-open-tab"},
                    {"kind": "run-first"}
                ]
            },
            {
                "saved": "kaspa-bridge",
                "active": "kaspa-node",
                "ops": [
                    {"kind": "record", "id": "first", "tab": "settings"},
                    {"kind": "record", "id": "second", "tab": "analysis"},
                    {"kind": "current", "id": "first"},
                    {"kind": "current", "id": "second"}
                ]
            },
            {
                "saved": "kaspa-bridge",
                "active": "kaspa-node",
                "ops": [
                    {"kind": "record", "id": "explicit", "tab": "settings"},
                    {"kind": "schedule", "reason": "ensure-active-no-change"}
                ]
            }
        ]
    })
}
fn expected() -> Value {
    json!({
        "scenarios": [
            {"results": [
                {"kind": "schedule", "value": true, "timers": 5},
                {"kind": "record", "id": "explicit"},
                {"kind": "run-all", "opens": []},
                {"kind": "current", "id": "explicit", "value": true}
            ]},
            {"results": [
                {"kind": "schedule", "value": true, "timers": 5},
                {"kind": "run-first", "opens": ["kaspa-bridge"]}
            ]},
            {"results": [
                {"kind": "record", "id": "first"},
                {"kind": "record", "id": "second"},
                {"kind": "current", "id": "first", "value": false},
                {"kind": "current", "id": "second", "value": true}
            ]},
            {"results": [
                {"kind": "record", "id": "explicit"},
                {"kind": "schedule", "value": false, "timers": 0}
            ]}
        ]
    })
}

fn verify_source_integration(source: &str) -> Result<(), String> {
    for needle in [
        "explicitNavigationGeneration: explicitNavigationGenerationR103",
        "!kgwShellExplicitNavigationIsCurrentR103(explicitNavigationGenerationR103)",
        "kgwShellRecordExplicitNavigationR103(button.dataset.tab)",
    ] {
        if !source.contains(needle) {
            return Err(format!(
                "AUD-013 source integration guard missing: {needle}"
            ));
        }
    }
    Ok(())
}
fn run_bridge(root: &Path, guard: &str) -> Result<Value, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-aud013-")
        .tempdir()
        .map_err(|error| format!("failed to create AUD-013 tempdir: {error}"))?;
    let bridge_path = temp.path().join("bridge.cjs");
    let guard_path = temp.path().join("guard.js");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write AUD-013 Node bridge: {error}"))?;
    fs::write(&guard_path, guard.as_bytes())
        .map_err(|error| format!("failed to write AUD-013 guard source: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(&request())
            .map_err(|error| format!("failed to serialize AUD-013 request: {error}"))?,
    )
    .map_err(|error| format!("failed to write AUD-013 request: {error}"))?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&guard_path)
        .arg(&request_path)
        .arg(&result_path)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch Node for AUD-013 regression: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "AUD-013 Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read AUD-013 result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse AUD-013 result: {error}"))
}
pub fn run(root: &Path) -> Result<String, String> {
    let source = fs::read_to_string(root.join(MAIN_RELATIVE))
        .map_err(|error| format!("failed to read {MAIN_RELATIVE}: {error}"))?;
    verify_source_integration(&source)?;
    let guard = guard_source(&source)?;
    let actual = run_bridge(root, guard)?;
    let expected = expected();
    if actual != expected {
        return Err(format!(
            "AUD-013 behavior mismatch: expected {expected}, got {actual}"
        ));
    }
    Ok("AUD-013 navigation regression Rust owner PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_markers_fail_closed() {
        assert!(guard_source("missing").is_err());
        let source = format!("{START_MARKER}\nbody\n{END_MARKER}");
        assert_eq!(
            guard_source(&source).unwrap(),
            format!("{START_MARKER}\nbody\n")
        );
    }

    #[test]
    fn rust_owns_four_frozen_behavior_scenarios() {
        assert_eq!(request()["scenarios"].as_array().map(Vec::len), Some(4));
        assert_eq!(expected()["scenarios"].as_array().map(Vec::len), Some(4));
    }
}
