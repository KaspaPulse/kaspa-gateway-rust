use regex::Regex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use time::OffsetDateTime;
use unicode_segmentation::UnicodeSegmentation;

const TRACE_PREFIX: &str = "[KGW_START_TRACE] ";
static CAPTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Acceptance {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub line_count: usize,
    pub character_count: usize,
    pub sha256: String,
}

fn sha256_hex(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn line_count(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.replace("\r\n", "\n")
            .replace('\r', "\n")
            .split('\n')
            .count()
    }
}

fn character_count(text: &str) -> usize {
    UnicodeSegmentation::graphemes(text, true).count()
}

fn safe_diagnostic_text(value: &str) -> String {
    let clean = value.replace(['\r', '\n', '\t'], " ").trim().to_owned();
    let lower = clean.to_ascii_lowercase();
    if [
        "secret", "token", "private", "mnemonic", "wallet", "address",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return "redacted-sensitive-value".to_owned();
    }
    clean.chars().take(360).collect()
}

fn safe_file_segment(value: Option<&str>) -> String {
    let source = value.unwrap_or("").trim();
    let mut clean = source
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    clean = clean.trim_matches('_').to_owned();
    if clean.is_empty() {
        clean = "unknown".to_owned();
    }
    clean.chars().take(80).collect()
}

fn transport_wrapper(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    for marker in [
        "kgw_raw_process_log_v1",
        "[kgw_child_stdout]",
        "[kgw_child_stderr]",
        "diagnostic_transport_record",
        ";source=self-worker;",
        ";runtime_role=",
        ";received_ms=",
    ] {
        if lower.contains(marker) {
            return true;
        }
    }
    let trimmed = text.trim_start();
    if !trimmed.starts_with('{') {
        return false;
    }
    let stage = Regex::new(r#"(?i)"stage"\s*:"#).expect("stage regex");
    let network = Regex::new(r#"(?i)"network"\s*:"#).expect("network regex");
    let source = Regex::new(r#"(?i)"source"\s*:"#).expect("source regex");
    let event =
        Regex::new(r#"(?i)"eventKind"\s*:\s*"diagnostic_transport_record""#).expect("event regex");
    stage.is_match(trimmed)
        && network.is_match(trimmed)
        && (source.is_match(trimmed) || event.is_match(trimmed))
}

fn nested_json(value: Option<&Value>) -> Option<Value> {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => serde_json::from_str(text).ok(),
        Some(value) if !value.is_null() => Some(value.clone()),
        _ => None,
    }
}

fn string_value(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) if !text.trim().is_empty() => Some(text.clone()),
        Some(value) if !value.is_null() => {
            let text = value.to_string();
            (!text.trim().is_empty()).then_some(text)
        }
        _ => None,
    }
}
fn first_present(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
        .cloned()
}

fn integer_value(value: Option<&Value>) -> Option<i64> {
    value.and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
    })
}

pub(crate) fn trace_metadata(line: &str) -> Value {
    if !line.starts_with(TRACE_PREFIX) {
        return json!({
            "parsed": false,
            "error": "not a KGW start trace line"
        });
    }
    let outer: Value = match serde_json::from_str(&line[TRACE_PREFIX.len()..]) {
        Ok(value) => value,
        Err(error) => {
            return json!({
                "parsed": false,
                "error": safe_diagnostic_text(&error.to_string())
            });
        }
    };
    let details = nested_json(outer.get("details"));
    let extra = details
        .as_ref()
        .and_then(|value| nested_json(value.get("extra")));

    let network = first_present(&[
        string_value(outer.get("network")),
        details
            .as_ref()
            .and_then(|value| string_value(value.get("network"))),
        extra
            .as_ref()
            .and_then(|value| string_value(value.get("network"))),
    ]);
    let runtime_role = first_present(&[
        details
            .as_ref()
            .and_then(|v| string_value(v.get("runtimeRole"))),
        details
            .as_ref()
            .and_then(|v| string_value(v.get("runtime_role"))),
        extra
            .as_ref()
            .and_then(|v| string_value(v.get("runtimeRole"))),
        extra
            .as_ref()
            .and_then(|v| string_value(v.get("runtime_role"))),
    ]);
    let bridge_instance_id = first_present(&[
        details
            .as_ref()
            .and_then(|v| string_value(v.get("bridgeInstanceId"))),
        details
            .as_ref()
            .and_then(|v| string_value(v.get("bridge_instance_id"))),
        extra
            .as_ref()
            .and_then(|v| string_value(v.get("bridgeInstanceId"))),
        extra
            .as_ref()
            .and_then(|v| string_value(v.get("bridge_instance_id"))),
    ]);
    let expected_sha256 = first_present(&[
        details.as_ref().and_then(|v| string_value(v.get("sha256"))),
        extra.as_ref().and_then(|v| string_value(v.get("sha256"))),
    ]);
    let expected_line_count = details
        .as_ref()
        .and_then(|v| integer_value(v.get("lineCount")))
        .or_else(|| {
            details
                .as_ref()
                .and_then(|v| integer_value(v.get("line_count")))
        })
        .or_else(|| {
            extra
                .as_ref()
                .and_then(|v| integer_value(v.get("lineCount")))
        })
        .or_else(|| {
            extra
                .as_ref()
                .and_then(|v| integer_value(v.get("line_count")))
        });
    let expected_character_count = details
        .as_ref()
        .and_then(|v| integer_value(v.get("characterCount")))
        .or_else(|| {
            details
                .as_ref()
                .and_then(|v| integer_value(v.get("character_count")))
        })
        .or_else(|| {
            extra
                .as_ref()
                .and_then(|v| integer_value(v.get("characterCount")))
        })
        .or_else(|| {
            extra
                .as_ref()
                .and_then(|v| integer_value(v.get("character_count")))
        });

    json!({
        "parsed": true,
        "error": Value::Null,
        "source": outer.get("source").cloned().unwrap_or(Value::Null),
        "stage": outer.get("stage").cloned().unwrap_or(Value::Null),
        "network": network,
        "runtime_role": runtime_role,
        "bridge_instance_id": bridge_instance_id,
        "expected_sha256": expected_sha256,
        "expected_line_count": expected_line_count,
        "expected_character_count": expected_character_count
    })
}

fn capture_id(metadata: &Value) -> String {
    let seq = CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1;
    format!(
        "{}-{seq:04}-{}-{}-{}-{}",
        OffsetDateTime::now_utc().unix_timestamp_nanos(),
        safe_file_segment(metadata["stage"].as_str()),
        safe_file_segment(metadata["network"].as_str()),
        safe_file_segment(metadata["runtime_role"].as_str()),
        safe_file_segment(metadata["bridge_instance_id"].as_str()),
    )
}

pub(crate) fn capture_from_text(
    text: &str,
    trace_line: &str,
    output_directory: &Path,
    reason: &str,
) -> Result<Value, String> {
    fs::create_dir_all(output_directory)
        .map_err(|error| format!("failed to create {}: {error}", output_directory.display()))?;
    let metadata = trace_metadata(trace_line);
    let actual_sha256 = sha256_hex(text);
    let expected_sha256 = metadata["expected_sha256"].as_str().unwrap_or("");
    let sha256_match =
        !expected_sha256.is_empty() && expected_sha256.eq_ignore_ascii_case(&actual_sha256);
    let wrapper = transport_wrapper(text);
    let mut errors = Vec::new();
    if metadata["parsed"].as_bool() != Some(true) {
        errors.push(format!(
            "Trace line could not be parsed: {}",
            metadata["error"].as_str().unwrap_or("unknown error")
        ));
    }
    if expected_sha256.is_empty() {
        errors.push("Trace line did not include expected SHA256.".to_owned());
    }
    if !sha256_match {
        errors.push("Expected SHA256 does not match event-time clipboard SHA256.".to_owned());
    }
    if wrapper {
        errors.push(
            "Clipboard payload contains diagnostic transport text and was not saved as raw output."
                .to_owned(),
        );
    }

    let id = capture_id(&metadata);
    let mut payload_file = output_directory.join(format!("{id}.raw.txt"));
    let metadata_file = output_directory.join(format!("{id}.capture.json"));
    let raw_payload_saved = !wrapper;
    if raw_payload_saved {
        fs::write(&payload_file, text.as_bytes())
            .map_err(|error| format!("failed to write {}: {error}", payload_file.display()))?;
    } else {
        payload_file = PathBuf::new();
    }

    let capture = json!({
        "capture_id": id,
        "reason": reason,
        "event_stage": metadata["stage"].clone(),
        "trace_source": metadata["source"].clone(),
        "capture_timestamp": OffsetDateTime::now_utc().unix_timestamp_nanos().to_string(),
        "network": metadata["network"].clone(),
        "runtime_role": metadata["runtime_role"].clone(),
        "bridge_instance_id": metadata["bridge_instance_id"].clone(),
        "line_count": line_count(text),
        "character_count": character_count(text),
        "expected_line_count": metadata["expected_line_count"].clone(),
        "expected_character_count": metadata["expected_character_count"].clone(),
        "expected_sha256": expected_sha256,
        "actual_sha256": actual_sha256,
        "sha256_match": sha256_match,
        "transport_wrapper_detected": wrapper,
        "raw_payload_saved": raw_payload_saved,
        "payload_file": if raw_payload_saved {
            Value::String(payload_file.to_string_lossy().into_owned())
        } else {
            Value::Null
        },
        "metadata_file": metadata_file.to_string_lossy(),
        "valid": errors.is_empty(),
        "errors": errors
    });
    let bytes = serde_json::to_vec_pretty(&capture)
        .map_err(|error| format!("capture JSON serialization failed: {error}"))?;
    fs::write(&metadata_file, bytes)
        .map_err(|error| format!("failed to write {}: {error}", metadata_file.display()))?;
    Ok(capture)
}

#[cfg(windows)]
pub(crate) fn capture_from_clipboard(
    trace_line: &str,
    output_directory: &Path,
    reason: &str,
) -> Result<Value, String> {
    match crate::e2e_clipboard::read_text() {
        Ok(text) => capture_from_text(&text, trace_line, output_directory, reason),
        Err(error) => capture_read_failure(trace_line, output_directory, reason, &error),
    }
}

#[cfg(windows)]
fn capture_read_failure(
    trace_line: &str,
    output_directory: &Path,
    reason: &str,
    error: &str,
) -> Result<Value, String> {
    fs::create_dir_all(output_directory)
        .map_err(|cause| format!("failed to create {}: {cause}", output_directory.display()))?;
    let metadata = trace_metadata(trace_line);
    let id = format!(
        "{}-{:04}-clipboard-unavailable",
        OffsetDateTime::now_utc().unix_timestamp_nanos(),
        CAPTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1
    );
    let metadata_file = output_directory.join(format!("{id}.capture.json"));
    let capture = json!({
        "capture_id": id,
        "reason": reason,
        "event_stage": metadata["stage"].clone(),
        "trace_source": metadata["source"].clone(),
        "capture_timestamp": OffsetDateTime::now_utc().unix_timestamp_nanos().to_string(),
        "network": metadata["network"].clone(),
        "runtime_role": metadata["runtime_role"].clone(),
        "bridge_instance_id": metadata["bridge_instance_id"].clone(),
        "line_count": Value::Null,
        "character_count": Value::Null,
        "expected_line_count": metadata["expected_line_count"].clone(),
        "expected_character_count": metadata["expected_character_count"].clone(),
        "expected_sha256": metadata["expected_sha256"].clone(),
        "actual_sha256": Value::Null,
        "sha256_match": false,
        "transport_wrapper_detected": false,
        "raw_payload_saved": false,
        "payload_file": Value::Null,
        "metadata_file": metadata_file.to_string_lossy(),
        "valid": false,
        "errors": [format!(
            "Clipboard read failed: {}",
            safe_diagnostic_text(error)
        )]
    });
    let bytes = serde_json::to_vec_pretty(&capture)
        .map_err(|cause| format!("capture JSON serialization failed: {cause}"))?;
    fs::write(&metadata_file, bytes)
        .map_err(|cause| format!("failed to write {}: {cause}", metadata_file.display()))?;
    Ok(capture)
}
#[cfg(windows)]
pub(crate) fn capture_with_acceptance_from_clipboard(
    trace_line: &str,
    output_directory: &Path,
    reason: &str,
    network: &str,
    runtime_role: &str,
) -> Result<Value, String> {
    let mut capture = capture_from_clipboard(trace_line, output_directory, reason)?;
    let text = capture["payload_file"]
        .as_str()
        .filter(|path| !path.is_empty())
        .map(fs::read_to_string)
        .transpose()
        .map_err(|error| format!("failed to read captured raw payload: {error}"))?
        .unwrap_or_default();
    let bridge_instance_id = capture["bridge_instance_id"].as_str();
    let acceptance = payload_acceptance(&text, network, runtime_role, bridge_instance_id);
    capture["acceptance_errors"] = json!(acceptance.errors);
    capture["acceptance_warnings"] = json!(acceptance.warnings);
    Ok(capture)
}

fn matches(pattern: &str, text: &str) -> bool {
    Regex::new(pattern)
        .expect("raw-log acceptance regex")
        .is_match(text)
}

pub(crate) fn payload_acceptance(
    text: &str,
    network: &str,
    runtime_role: &str,
    bridge_instance_id: Option<&str>,
) -> Acceptance {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let network = network.trim().to_ascii_lowercase();
    let role = runtime_role.trim().to_ascii_lowercase();

    if text.trim().is_empty() {
        errors.push("Raw payload is empty.".to_owned());
    }
    if transport_wrapper(text) {
        errors.push("Raw payload contains transport wrapper text.".to_owned());
    }

    if role == "node" && network == "testnet10" {
        if !matches(r"(?i)\bkaspad\b|kaspa", text) {
            errors.push(
                "Testnet10 Node raw payload does not contain direct kaspad stdout/stderr evidence."
                    .to_owned(),
            );
        }
        if !matches(
            r"(?i)testnet10|testnet-10|tn10|kaspa-gateway-testnet10",
            text,
        ) {
            errors.push(
                "Testnet10 Node raw payload does not contain Testnet10 application or data directory evidence."
                    .to_owned(),
            );
        }
        if matches(
            r"(?i)mainnet.*(kaspad|kaspa-gateway|appdata|data|db)|kaspa-gateway-mainnet|\\mainnet\\|/mainnet/",
            text,
        ) {
            errors
                .push("Testnet10 Node raw payload contains Mainnet node path evidence.".to_owned());
        }
        if !matches(r"(?i)\brpc\b|\bp2p\b|listen|port|16210|16211", text) {
            warnings.push(
                "Testnet10 Node raw payload did not expose RPC or P2P evidence in the captured lines."
                    .to_owned(),
            );
        }
    }

    if role == "bridge" {
        if matches(
            r"(?i)parallel-owned-self-worker status;role=bridge|no bridge worker status yet|Controller diagnostics are available separately|diagnosticLineCount=",
            text,
        ) {
            errors.push(
                "Bridge status summary was captured instead of raw child process output."
                    .to_owned(),
            );
        }
        if !matches(
            r"(?i)\bbridge\b|stratum|rk-bridge|test-self-worker stdout role=bridge|test-self-worker stderr role=bridge",
            text,
        ) {
            errors.push(
                "Bridge raw payload does not contain bridge child stdout/stderr evidence."
                    .to_owned(),
            );
        }
        if matches(
            r"(?i)test-self-worker stdout role=node|test-self-worker stderr role=node|runtimeRole.?node",
            text,
        ) {
            errors.push("Bridge raw payload contains node buffer records.".to_owned());
        }
        if bridge_instance_id.is_none_or(|value| value.trim().is_empty()) {
            warnings
                .push("Bridge instance identity was not provided by the capture trace.".to_owned());
        }
    }

    Acceptance {
        passed: errors.is_empty(),
        errors,
        warnings,
        line_count: line_count(text),
        character_count: character_count(text),
        sha256: sha256_hex(text),
    }
}

pub(crate) fn self_test() -> Result<String, String> {
    let output_directory = std::env::temp_dir().join(format!(
        "kgw-raw-clipboard-capture-self-test-{}-{}",
        std::process::id(),
        OffsetDateTime::now_utc().unix_timestamp_nanos()
    ));
    let raw = "test-self-worker stdout role=bridge network=testnet10\r\ntest-self-worker stderr role=bridge network=testnet10";
    let sha = sha256_hex(raw);
    let trace = format!(
        r#"[KGW_START_TRACE] {{"timestamp":1,"source":"frontend","stage":"frontend.copy_log_succeeded","network":"testnet10","action":"copy-log","result":"ok","details":"{{\"runtimeRole\":\"bridge\",\"bridgeInstanceId\":\"bridge-a\",\"characterCount\":{},\"lineCount\":{},\"sha256\":\"{}\"}}"}}"#,
        character_count(raw),
        line_count(raw),
        sha
    );
    let capture = capture_from_text(raw, &trace, &output_directory, "self-test")?;
    let mut errors = Vec::new();
    if capture["valid"].as_bool() != Some(true) {
        errors.push("Event-time capture should pass with matching SHA256.".to_owned());
    }
    let payload = capture["payload_file"].as_str().map(PathBuf::from);
    if payload.as_ref().is_none_or(|path| !path.is_file()) {
        errors.push("Event-time capture did not save the raw payload file.".to_owned());
    } else if fs::read_to_string(payload.as_ref().expect("payload path")).unwrap_or_default() != raw
    {
        errors.push(
            "Saved raw payload file does not exactly match event-time clipboard text.".to_owned(),
        );
    }

    let mismatch = capture_from_text(
        "changed later",
        &trace,
        &output_directory,
        "self-test-mismatch",
    )?;
    if mismatch["valid"].as_bool() == Some(true) || mismatch["sha256_match"].as_bool() == Some(true)
    {
        errors.push("Capture with mismatched SHA256 should fail.".to_owned());
    }

    let wrapped = capture_from_text(
        "[KGW_CHILD_STDERR] diagnostic_transport_record",
        &trace,
        &output_directory,
        "self-test-wrapper",
    )?;
    if wrapped["raw_payload_saved"].as_bool() == Some(true)
        || wrapped["valid"].as_bool() == Some(true)
    {
        errors.push("Diagnostic transport output must not be saved as a raw payload.".to_owned());
    }

    let status = payload_acceptance(
        "parallel-owned-self-worker status;role=bridge;network=mainnet;running=false;message=no bridge worker status yet",
        "mainnet",
        "bridge",
        Some("bridge-a"),
    );
    if status.passed {
        errors.push("Bridge status summary should be rejected as raw process output.".to_owned());
    }
    let missing = payload_acceptance("", "testnet10", "bridge", Some("bridge-a"));
    if missing.passed {
        errors.push("Missing bridge output should fail acceptance.".to_owned());
    }
    let transport = payload_acceptance(
        "kgw_raw_process_log_v1;network=mainnet;source=self-worker;runtime_role=bridge;received_ms=1;line=fake",
        "mainnet",
        "bridge",
        Some("bridge-a"),
    );
    if transport.passed {
        errors.push("Transport wrappers should fail acceptance.".to_owned());
    }

    if errors.is_empty() {
        Ok(format!(
            "KGW raw log clipboard capture self-test PASS\nOUTPUT_DIRECTORY={}",
            output_directory.display()
        ))
    } else {
        Err(format!(
            "KGW raw log clipboard capture self-test FAILED\n- {}",
            errors.join("\n- ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trace_for(raw: &str) -> String {
        format!(
            r#"[KGW_START_TRACE] {{"timestamp":1,"source":"frontend","stage":"frontend.copy_log_succeeded","network":"testnet10","details":"{{\"runtimeRole\":\"bridge\",\"bridgeInstanceId\":\"bridge-a\",\"characterCount\":{},\"lineCount\":{},\"sha256\":\"{}\"}}"}}"#,
            character_count(raw),
            line_count(raw),
            sha256_hex(raw)
        )
    }

    #[test]
    fn metadata_extracts_nested_trace_contract() {
        let raw = "bridge test-self-worker stdout role=bridge network=testnet10";
        let meta = trace_metadata(&trace_for(raw));
        assert_eq!(meta["parsed"], true);
        assert_eq!(meta["network"], "testnet10");
        assert_eq!(meta["runtime_role"], "bridge");
        assert_eq!(meta["bridge_instance_id"], "bridge-a");
        assert_eq!(meta["expected_sha256"], sha256_hex(raw));
    }

    #[test]
    fn invalid_trace_fails_closed() {
        let meta = trace_metadata("not-a-trace");
        assert_eq!(meta["parsed"], false);
        assert_eq!(meta["error"], "not a KGW start trace line");
    }
    #[test]
    fn transport_wrappers_are_detected() {
        assert!(transport_wrapper(
            "[KGW_CHILD_STDOUT] diagnostic_transport_record"
        ));
        assert!(transport_wrapper(
            r#"{"stage":"x","network":"mainnet","source":"self-worker"}"#
        ));
        assert!(!transport_wrapper(
            "direct kaspad stdout testnet10 rpc listen"
        ));
    }

    #[test]
    fn capture_hash_and_payload_are_stable() {
        let dir = std::env::temp_dir().join(format!(
            "kgw-capture-unit-{}",
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        ));
        let raw = "bridge test-self-worker stdout role=bridge network=testnet10";
        let capture = capture_from_text(raw, &trace_for(raw), &dir, "unit").unwrap();
        assert_eq!(capture["valid"], true);
        assert_eq!(capture["sha256_match"], true);
        assert_eq!(capture["actual_sha256"], sha256_hex(raw));
        let payload = PathBuf::from(capture["payload_file"].as_str().unwrap());
        assert_eq!(fs::read_to_string(payload).unwrap(), raw);
    }

    #[test]
    fn bridge_acceptance_rejects_status_summary() {
        let result = payload_acceptance(
            "parallel-owned-self-worker status;role=bridge;network=mainnet;running=false;message=no bridge worker status yet",
            "mainnet",
            "bridge",
            Some("bridge-a"),
        );
        assert!(!result.passed);
    }

    #[test]
    fn character_count_uses_grapheme_clusters() {
        assert_eq!(character_count("a\u{301}🦀"), 2);
    }

    #[test]
    fn deterministic_self_test_passes() {
        let result = self_test().unwrap();
        assert!(result.contains("PASS"));
    }
}
