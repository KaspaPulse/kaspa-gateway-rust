use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct ProcessResult {
    code: i32,
    text: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let artifact_root = unique_artifact_root(root);
    fs::create_dir_all(&artifact_root).map_err(|error| {
        format!(
            "failed to create zero-touch result-writer test root {}: {error}",
            artifact_root.display()
        )
    })?;

    let evidence = root.join("tools/kgw_zero_touch_evidence.ps1");
    if !evidence.is_file() {
        return Err(format!(
            "missing zero-touch evidence helper: {}",
            evidence.display()
        ));
    }

    test_success_json(root, &evidence, &artifact_root)?;
    test_atomic_replacement(root, &evidence, &artifact_root)?;
    test_failed_primary_preserves_existing(root, &evidence, &artifact_root)?;
    test_exception_serialization(root, &evidence)?;
    test_emergency_fallback(root, &evidence, &artifact_root)?;
    test_powershell_version_guard(root, &evidence)?;
    test_required_ports(root, &evidence)?;

    Ok(format!(
        "KGW zero-touch result writer tests PASSED\nTest artifacts: {}",
        artifact_root.display()
    ))
}

fn unique_artifact_root(root: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    root.join("artifacts/zero-touch-result-writer-tests")
        .join(format!("rust-{}-{stamp}", std::process::id()))
}

fn ps_quote(value: impl AsRef<Path>) -> String {
    let text = value.as_ref().to_string_lossy().replace('\'', "''");
    format!("'{text}'")
}

fn source_prefix(evidence: &Path) -> String {
    format!(". {}; ", ps_quote(evidence))
}

fn run_pwsh(root: &Path, script: &str) -> Result<ProcessResult, String> {
    let output = Command::new("pwsh")
        .args(["-NoLogo", "-NoProfile", "-Command", script])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch pwsh: {error}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.stderr.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok(ProcessResult {
        code: output.status.code().unwrap_or(1),
        text,
    })
}

fn expect_success(root: &Path, script: &str, label: &str) -> Result<String, String> {
    let result = run_pwsh(root, script)?;
    if result.code != 0 {
        return Err(format!(
            "{label} failed with exit code {}: {}",
            result.code,
            result.text.trim()
        ));
    }
    Ok(result.text)
}

fn read_json(path: &Path) -> Result<Value, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid JSON {}: {error}", path.display()))
}

fn require(condition: bool, message: impl Into<String>) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn test_success_json(root: &Path, evidence: &Path, dir: &Path) -> Result<(), String> {
    let path = dir.join("success.json");
    let script = format!(
        "{}$v=[ordered]@{{completed=$true;success=$true;exit_code=0;labels=@('Mainnet Node','Testnet10 Bridge');nested=[ordered]@{{pid=1234;port=16110;endpoint='127.0.0.1:16110'}}}}; Write-KgwZeroTouchJsonFile -Value $v -Path {} -Depth 8",
        source_prefix(evidence),
        ps_quote(&path)
    );
    expect_success(root, &script, "successful result serialization")?;

    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    require(
        !bytes.starts_with(&[0xef, 0xbb, 0xbf]),
        "result JSON must be UTF-8 without BOM",
    )?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    require(
        value.get("success").and_then(Value::as_bool) == Some(true),
        "successful result serialization flag mismatch",
    )?;
    require(
        value.pointer("/nested/pid").and_then(Value::as_i64) == Some(1234),
        "PID must serialize as integer",
    )?;
    require(
        value.pointer("/nested/port").and_then(Value::as_i64) == Some(16110),
        "port must serialize as integer",
    )
}

fn test_atomic_replacement(root: &Path, evidence: &Path, dir: &Path) -> Result<(), String> {
    let path = dir.join("atomic.json");
    fs::write(&path, br#"{"version":1}"#).map_err(|error| error.to_string())?;
    let script = format!(
        "{}Write-KgwZeroTouchJsonFile -Value ([ordered]@{{version=2;replaced=$true}}) -Path {} -Depth 4",
        source_prefix(evidence),
        ps_quote(&path)
    );
    expect_success(root, &script, "atomic replacement")?;
    let value = read_json(&path)?;
    require(
        value.get("version").and_then(Value::as_i64) == Some(2),
        "atomic replacement did not write version 2",
    )?;
    let leftovers = fs::read_dir(dir)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(".atomic.json.") && name.ends_with(".tmp")
        })
        .count();
    require(leftovers == 0, "atomic replacement left temp files")
}

fn test_failed_primary_preserves_existing(
    root: &Path,
    evidence: &Path,
    dir: &Path,
) -> Result<(), String> {
    let path = dir.join("failed-primary.json");
    fs::write(&path, br#"{"stable":true}"#).map_err(|error| error.to_string())?;
    let tools = root.join("tools");
    let script = format!(
        "{}try {{ Write-KgwZeroTouchJsonFile -Value ([ordered]@{{bad=(Get-Item -LiteralPath {})}}) -Path {} -Depth 4; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 17 }}",
        source_prefix(evidence),
        ps_quote(&tools),
        ps_quote(&path)
    );
    let result = run_pwsh(root, &script)?;
    require(
        result.code == 17,
        format!(
            "primary writer must reject complex .NET objects; exit={} output={}",
            result.code,
            result.text.trim()
        ),
    )?;
    require(
        ["FileInfo", "DirectoryInfo", "JSON-safe"]
            .iter()
            .any(|needle| result.text.contains(needle)),
        format!(
            "failed serialization must report rejected object type: {}",
            result.text.trim()
        ),
    )?;
    require(
        fs::read_to_string(&path).map_err(|error| error.to_string())? == r#"{"stable":true}"#,
        "failed primary serialization must preserve existing result file",
    )
}
fn test_exception_serialization(root: &Path, evidence: &Path) -> Result<(), String> {
    let script = format!(
        "{}try {{ throw 'synthetic writer failure' }} catch {{ ConvertTo-KgwZeroTouchSerializableException -Value $_ | ConvertTo-Json -Depth 8 -Compress }}",
        source_prefix(evidence)
    );
    let output = expect_success(root, &script, "exception serialization")?;
    let value: Value = serde_json::from_str(output.trim())
        .map_err(|error| format!("exception JSON invalid: {error}: {output}"))?;
    require(
        value.get("message").and_then(Value::as_str) == Some("synthetic writer failure"),
        "exception message serialization mismatch",
    )?;
    require(
        value
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty()),
        "exception type serialization missing",
    )?;
    require(
        value.get("stack").is_some(),
        "exception stack serialization missing",
    )
}

fn test_emergency_fallback(root: &Path, evidence: &Path, dir: &Path) -> Result<(), String> {
    let path = dir.join("fallback.json");
    let script = format!(
        "{}try {{ throw 'synthetic writer failure' }} catch {{ $e=ConvertTo-KgwZeroTouchSerializableException -Value $_ }}; $v=New-KgwZeroTouchWriterFailureResultObject -Repository {} -ArtifactDirectory {} -StartedAt '2026-07-29T00:00:00.0000000Z' -OriginalExitCode 37 -OriginalFailedStage 'WebdriverIO zero-touch live matrix' -ExecutablePath '' -WriterError $e -ValidationErrors @('original validation detail'); Write-KgwZeroTouchEmergencyJsonFile -Value $v -Path {} -Depth 8",
        source_prefix(evidence),
        ps_quote(root),
        ps_quote(dir),
        ps_quote(&path)
    );
    expect_success(root, &script, "emergency fallback serialization")?;
    let value = read_json(&path)?;
    require(
        value.get("success").and_then(Value::as_bool) == Some(false),
        "fallback success flag must be false",
    )?;
    require(
        value.get("exit_code").and_then(Value::as_i64) == Some(37),
        "fallback must preserve original exit code",
    )?;
    require(
        value.get("original_exit_code").and_then(Value::as_i64) == Some(37),
        "fallback original_exit_code mismatch",
    )?;
    require(
        value.get("failed_stage").and_then(Value::as_str) == Some("Zero-touch result writing"),
        "fallback failed_stage mismatch",
    )?;
    require(
        value
            .pointer("/result_writer_error/message")
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty()),
        "fallback writer exception missing",
    )
}

fn test_powershell_version_guard(root: &Path, evidence: &Path) -> Result<(), String> {
    let reject = format!(
        "{}try {{ Assert-KgwZeroTouchPowerShell7 -MajorVersion 5; exit 0 }} catch {{ exit 23 }}",
        source_prefix(evidence)
    );
    let result = run_pwsh(root, &reject)?;
    require(
        result.code == 23,
        "PowerShell major version below 7 must be rejected",
    )?;

    let accept = format!(
        "{}Assert-KgwZeroTouchPowerShell7 -MajorVersion 7",
        source_prefix(evidence)
    );
    expect_success(root, &accept, "PowerShell 7 acceptance")?;
    Ok(())
}
fn stages_json(root: &Path, evidence: &Path, overrides: &[(&str, &str)]) -> Result<Value, String> {
    let mut prefix = source_prefix(evidence);
    for (name, value) in overrides {
        prefix.push_str(&format!("$env:{name}='{}'; ", value.replace('\'', "''")));
    }
    prefix.push_str("Get-KgwZeroTouchRequiredStages | ConvertTo-Json -Depth 8 -Compress");
    let output = expect_success(root, &prefix, "required stages")?;
    serde_json::from_str(output.trim())
        .map_err(|error| format!("required stages JSON invalid: {error}: {output}"))
}

fn stage_ports<'a>(value: &'a Value, slug: &str) -> Option<&'a Vec<Value>> {
    value.as_array()?.iter().find_map(|stage| {
        (stage.get("Slug").and_then(Value::as_str) == Some(slug))
            .then(|| stage.get("RequiredPorts")?.as_array())
            .flatten()
    })
}

fn ports(values: &[Value]) -> Vec<i64> {
    values.iter().filter_map(Value::as_i64).collect()
}

fn test_required_ports(root: &Path, evidence: &Path) -> Result<(), String> {
    let defaults = stages_json(root, evidence, &[])?;
    require(
        stage_ports(&defaults, "mainnet-node")
            .map(|items| ports(items) == vec![16110, 16111])
            .unwrap_or(false),
        "default Mainnet evidence ports mismatch",
    )?;

    let isolated = stages_json(
        root,
        evidence,
        &[
            ("KGW_E2E_MAINNET_RPC_PORT", "16120"),
            ("KGW_E2E_MAINNET_P2P_PORT", "16121"),
            ("KGW_E2E_MAINNET_BRIDGE_PORT", "5566"),
            ("KGW_E2E_TESTNET10_RPC_PORT", "16220"),
            ("KGW_E2E_TESTNET10_P2P_PORT", "16221"),
            ("KGW_E2E_TESTNET10_BRIDGE_PORT", "5666"),
        ],
    )?;
    for (slug, expected) in [
        ("mainnet-node", vec![16120, 16121]),
        ("mainnet-bridge", vec![5566]),
        ("testnet10-node", vec![16220, 16221]),
        ("testnet10-bridge", vec![5666]),
    ] {
        require(
            stage_ports(&isolated, slug)
                .map(|items| ports(items) == expected)
                .unwrap_or(false),
            format!("isolated evidence ports mismatch for {slug}"),
        )?;
    }

    let invalid = format!(
        "{}$env:KGW_E2E_MAINNET_RPC_PORT='abc'; try {{ [void](Get-KgwZeroTouchRequiredStages); exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 31 }}",
        source_prefix(evidence)
    );
    let result = run_pwsh(root, &invalid)?;
    require(
        result.code == 31 && result.text.contains("integer TCP port in 1024..65535"),
        format!(
            "invalid evidence port override must fail closed: code={} output={}",
            result.code,
            result.text.trim()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ps_quote_escapes_single_quotes() {
        assert_eq!(ps_quote(Path::new(r"C:\a'b\c")), r#"'C:\a''b\c'"#);
    }

    #[test]
    fn stage_port_lookup_is_fail_closed() {
        let value: Value = serde_json::json!([
            {"Slug":"mainnet-node","RequiredPorts":[16110,16111]}
        ]);
        assert_eq!(
            stage_ports(&value, "mainnet-node").map(|items| ports(items)),
            Some(vec![16110, 16111])
        );
        assert!(stage_ports(&value, "missing").is_none());
    }
}
