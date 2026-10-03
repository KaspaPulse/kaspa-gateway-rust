use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use xtask::zero_touch_evidence::{dispatch, result};
use xtask::zero_touch_result_io::atomic_write;

pub fn run(root: &Path) -> Result<String, String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    test_atomic_writer(dir.path())?;
    test_failure_receipt(root, dir.path())?;
    test_stage_ports(root)?;
    test_invalid_stage_port(root)?;
    Ok(format!(
        "KGW zero-touch native result/evidence tests PASSED\nTest artifacts: {}",
        dir.path().display()
    ))
}

fn require(ok: bool, message: impl Into<String>) -> Result<(), String> {
    if ok { Ok(()) } else { Err(message.into()) }
}

fn test_atomic_writer(dir: &Path) -> Result<(), String> {
    let path = dir.join("result.json");
    atomic_write(
        &path,
        &json!({
            "success": true,
            "pid": 9_007_199_254_740_993_i64,
            "labels": ["Mainnet Node", "Testnet10 Bridge"]
        }),
    )?;
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    require(
        !bytes.starts_with(&[0xef, 0xbb, 0xbf]),
        "JSON writer emitted BOM",
    )?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    require(value["success"] == true, "success flag changed")?;
    require(
        value["pid"].as_i64() == Some(9_007_199_254_740_993_i64),
        "signed integer changed",
    )?;

    let stable = bytes.clone();
    require(
        atomic_write(&path, &json!({"bad": 1.25})).is_err(),
        "float must be rejected",
    )?;
    require(
        fs::read(&path).map_err(|e| e.to_string())? == stable,
        "failed write changed target",
    )?;

    atomic_write(&path, &json!({"version": 2, "arabic": "مرحبا"}))?;
    let replaced: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    require(replaced["version"] == 2, "atomic replacement failed")
}

fn test_failure_receipt(root: &Path, dir: &Path) -> Result<(), String> {
    let request = json!({
        "repository": root,
        "artifact_directory": dir,
        "started_at": "2026-07-29T00:00:00Z",
        "original_exit_code": 37,
        "original_failed_stage": "WebdriverIO zero-touch live matrix",
        "executable_path": "",
        "writer_error": {
            "message": "synthetic writer failure",
            "type": "FixtureError",
            "stack": "fixture stack"
        },
        "validation_errors": ["original validation detail"]
    });
    let receipt = result::failure(&request)?;
    require(
        receipt["success"] == false,
        "failure receipt claimed success",
    )?;
    require(receipt["exit_code"] == 37, "failure exit code changed")?;
    require(
        receipt["original_exit_code"] == 37,
        "original exit code changed",
    )?;
    require(
        receipt["failed_stage"] == "Zero-touch result writing",
        "writer failure stage changed",
    )?;
    require(
        receipt
            .pointer("/result_writer_error/message")
            .and_then(Value::as_str)
            == Some("synthetic writer failure"),
        "writer error message missing",
    )
}

fn stages(root: &Path) -> Result<Value, String> {
    dispatch("stages", root, root, None)
}

fn port_list(value: &Value, slug: &str) -> Vec<i64> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .find(|stage| stage["Slug"].as_str() == Some(slug))
        .and_then(|stage| stage["RequiredPorts"].as_array())
        .map(|items| items.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default()
}

fn test_stage_ports(root: &Path) -> Result<(), String> {
    let value = stages(root)?;
    require(
        port_list(&value, "mainnet-node") == vec![16110, 16111],
        "default Mainnet evidence ports mismatch",
    )?;
    require(
        port_list(&value, "testnet10-node") == vec![16210, 16211],
        "default Testnet10 evidence ports mismatch",
    )?;
    require(
        port_list(&value, "mainnet-bridge") == vec![5556],
        "default Mainnet Bridge port mismatch",
    )?;
    require(
        port_list(&value, "testnet10-bridge") == vec![5656],
        "default Testnet10 Bridge port mismatch",
    )
}

fn test_invalid_stage_port(root: &Path) -> Result<(), String> {
    const KEY: &str = "KGW_E2E_MAINNET_RPC_PORT";
    let before = std::env::var_os(KEY);
    // This xtask gate is single-process and restores the inherited environment before return.
    unsafe { std::env::set_var(KEY, "abc") };
    let rejected = stages(root).is_err();
    match before {
        Some(value) => unsafe { std::env::set_var(KEY, value) },
        None => unsafe { std::env::remove_var(KEY) },
    }
    require(rejected, "invalid evidence TCP port must fail closed")
}
