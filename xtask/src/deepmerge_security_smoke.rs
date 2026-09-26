use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_BRIDGE: &str = r#"
import { readFile, writeFile } from "node:fs/promises";
import { deepmerge, deepmergeCustom } from "deepmerge-ts";

const [requestPath, resultPath] = process.argv.slice(2);
const request = JSON.parse(await readFile(requestPath, "utf8"));
let result;

if (request.action === "config") {
  const mergeConfig = deepmergeCustom({
    mergeArrays: ([oldValue, newValue], utils, meta) => {
      if (meta?.key === "services") {
        const primitiveOldValues = oldValue.filter((value) => typeof value !== "object");
        return Array.from(new Set(deepmerge(newValue, primitiveOldValues)));
      }
      return utils.actions.defaultMerge;
    }
  });
  const merged = mergeConfig(request.left, request.right);
  result = { services: merged.services, specs: merged.specs };
} else if (request.action === "circular") {
  const left = { left: true };
  left.self = left;
  const right = { right: true };
  right.self = right;
  const merged = deepmerge(left, right);
  result = {
    left: merged.left === true,
    right: merged.right === true,
    self: merged.self === merged
  };
} else {
  throw new Error("unknown deepmerge smoke action: " + request.action);
}

await writeFile(resultPath, JSON.stringify(result), "utf8");
"#;

fn run_bridge(root: &Path, request: &Value) -> Result<Value, String> {
    let e2e = root.join("e2e");
    let temp = tempfile::Builder::new()
        .prefix(".kgw-deepmerge-smoke-")
        .tempdir_in(&e2e)
        .map_err(|error| format!("failed to create deepmerge smoke tempdir: {error}"))?;
    let bridge_path = temp.path().join("bridge.mjs");
    let request_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");

    fs::write(&bridge_path, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write deepmerge Node bridge: {error}"))?;
    fs::write(
        &request_path,
        serde_json::to_vec(request)
            .map_err(|error| format!("failed to serialize deepmerge request: {error}"))?,
    )
    .map_err(|error| format!("failed to write deepmerge request: {error}"))?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg(&request_path)
        .arg(&result_path)
        .current_dir(&e2e)
        .output()
        .map_err(|error| format!("failed to launch Node for deepmerge security smoke: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "deepmerge Node bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    serde_json::from_slice(
        &fs::read(&result_path)
            .map_err(|error| format!("failed to read deepmerge smoke result: {error}"))?,
    )
    .map_err(|error| format!("failed to parse deepmerge smoke result: {error}"))
}

fn verify(label: &str, actual: &Value, expected: Value) -> Result<(), String> {
    if actual == &expected {
        Ok(())
    } else {
        Err(format!(
            "{label} mismatch: expected {expected}, got {actual}"
        ))
    }
}
pub fn run(root: &Path) -> Result<String, String> {
    let config = run_bridge(
        root,
        &json!({
            "action": "config",
            "left": {
                "services": ["legacy-service", ["legacy-service-object", {"enabled": true}]],
                "specs": ["legacy.e2e.js"]
            },
            "right": {
                "services": ["current-service"],
                "specs": ["current.e2e.js"]
            }
        }),
    )?;
    verify(
        "WebdriverIO deepmergeCustom compatibility",
        &config,
        json!({
            "services": ["current-service", "legacy-service"],
            "specs": ["legacy.e2e.js", "current.e2e.js"]
        }),
    )?;

    let circular = run_bridge(root, &json!({"action": "circular"}))?;
    verify(
        "deepmerge circular-reference safety",
        &circular,
        json!({"left": true, "right": true, "self": true}),
    )?;

    Ok("deepmerge security compatibility Rust-owned smoke PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_accepts_exact_result_and_rejects_drift() {
        let expected = json!({"value": ["a", "b"]});
        assert!(verify("fixture", &expected, expected.clone()).is_ok());
        assert!(verify("fixture", &json!({"value": ["b", "a"]}), expected).is_err());
    }
}
