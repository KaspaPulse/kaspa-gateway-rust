use js_sys::{Array, Error, Object, Reflect, RegExp};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};

use crate::{js_number, js_string};

pub const RAW_LOG_REJECTION_MARKERS: [&str; 7] = [
    "kgw_raw_process_log_v1",
    "[KGW_CHILD_STDOUT]",
    "[KGW_CHILD_STDERR]",
    "diagnostic_transport_record",
    ";source=self-worker;",
    ";runtime_role=",
    ";received_ms=",
];

fn string_or_empty(value: &JsValue) -> String {
    if value.is_null() || value.is_undefined() {
        String::new()
    } else {
        String::from(js_string(value))
    }
}

fn js_truthy(value: &JsValue) -> bool {
    if value.is_null() || value.is_undefined() {
        return false;
    }
    if let Some(value) = value.as_bool() {
        return value;
    }
    if let Some(value) = value.as_f64() {
        return value != 0.0 && !value.is_nan();
    }
    if let Some(value) = value.as_string() {
        return !value.is_empty();
    }
    true
}

fn sha256_hex_text(text: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(text.as_bytes());
    format!("{:x}", digest.finalize())
}

fn normalize_clipboard_text_value(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace("\n", "\r\n")
}

fn line_count_value(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .count()
}

fn transport_line_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r#"(?x)^\s*\{.*"stage"\s*:.*"network"\s*:(?:.*"source"\s*:|.*"eventKind"\s*:\s*"diagnostic_transport_record")"#,
        )
        .expect("valid transport wrapper regex")
    })
}

fn contains_transport_wrapper_value(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    if RAW_LOG_REJECTION_MARKERS
        .iter()
        .any(|marker| lower.contains(&marker.to_ascii_lowercase()))
    {
        return true;
    }
    text.lines()
        .any(|line| transport_line_regex().is_match(line.trim_start()))
}

fn parse_key_value_line_value(text: &str) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    for part in text.split(';') {
        let Some(index) = part.find('=') else {
            continue;
        };
        if index == 0 {
            continue;
        }
        fields.push((
            part[..index].trim().to_owned(),
            part[index + 1..].trim().to_owned(),
        ));
    }
    fields
}

fn field<'a>(fields: &'a [(String, String)], name: &str) -> &'a str {
    fields
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value.as_str()))
        .unwrap_or("")
}

fn is_stopped_owner_status_value(text: &str) -> bool {
    let fields = parse_key_value_line_value(text);
    let running = field(&fields, "running");
    if running.eq_ignore_ascii_case("false") {
        return true;
    }
    static STOPPED: OnceLock<Regex> = OnceLock::new();
    let regex = STOPPED.get_or_init(|| {
        Regex::new(r"(?i)no .*worker status yet|stopped").expect("valid stopped regex")
    });
    regex.is_match(text) && !running.eq_ignore_ascii_case("true")
}

pub fn parse_key_value_line_native(text: &str) -> Vec<(String, String)> {
    parse_key_value_line_value(text)
}

pub fn pid_from_status_native(text: &str) -> Option<u32> {
    let fields = parse_key_value_line_value(text);
    field(&fields, "pid")
        .parse::<u32>()
        .ok()
        .filter(|pid| *pid > 0)
}

pub fn is_stopped_owner_status_native(text: &str) -> bool {
    is_stopped_owner_status_value(text)
}

fn assertion_error(message: &str) -> JsValue {
    let error = Error::new(message);
    error.set_name("AssertionError");
    let _ = Reflect::set(
        error.as_ref(),
        &JsValue::from_str("code"),
        &JsValue::from_str("ERR_ASSERTION"),
    );
    error.into()
}

fn strict_string_mismatch(message: &str, actual: &str, expected: &str, multiline: bool) -> JsValue {
    let detail = if multiline {
        format!("{message}\n+ actual - expected\n\n+ '{actual}'\n- '{expected}'\n")
    } else {
        format!("{message}\n\n'{actual}' !== '{expected}'\n")
    };
    assertion_error(&detail)
}

fn get(value: &JsValue, key: &str) -> JsValue {
    if value.is_null() || value.is_undefined() {
        JsValue::UNDEFINED
    } else {
        Reflect::get(value, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
    }
}

#[wasm_bindgen(js_name = rawLogRejectionMarkers)]
pub fn raw_log_rejection_markers() -> JsValue {
    let array = Array::new();
    for marker in RAW_LOG_REJECTION_MARKERS {
        array.push(&JsValue::from_str(marker));
    }
    Object::freeze(array.unchecked_ref()).into()
}

#[wasm_bindgen(js_name = sha256Hex)]
pub fn sha256_hex(value: JsValue) -> String {
    sha256_hex_text(&string_or_empty(&value))
}

#[wasm_bindgen(js_name = normalizeClipboardText)]
pub fn normalize_clipboard_text(value: JsValue) -> String {
    normalize_clipboard_text_value(&string_or_empty(&value))
}

#[wasm_bindgen(js_name = lineCount)]
pub fn line_count(value: JsValue) -> usize {
    line_count_value(&string_or_empty(&value))
}

#[wasm_bindgen(js_name = containsTransportWrapper)]
pub fn contains_transport_wrapper(value: JsValue) -> bool {
    contains_transport_wrapper_value(&string_or_empty(&value))
}

#[wasm_bindgen(js_name = parseKeyValueLine)]
pub fn parse_key_value_line(value: JsValue) -> Result<JsValue, JsValue> {
    let object = Object::new();
    for (key, value) in parse_key_value_line_value(&string_or_empty(&value)) {
        Reflect::set(
            object.as_ref(),
            &JsValue::from_str(&key),
            &JsValue::from_str(&value),
        )?;
    }
    Ok(object.into())
}

#[wasm_bindgen(js_name = pidFromStatus)]
pub fn pid_from_status(value: JsValue) -> JsValue {
    let fields = parse_key_value_line_value(&string_or_empty(&value));
    let raw = field(&fields, "pid");
    let number = js_number(&JsValue::from_str(raw));
    if number.is_finite()
        && number.fract() == 0.0
        && number > 0.0
        && number <= 9_007_199_254_740_991.0
    {
        JsValue::from_f64(number)
    } else {
        JsValue::NULL
    }
}

#[wasm_bindgen(js_name = isStoppedOwnerStatus)]
pub fn is_stopped_owner_status(value: JsValue) -> bool {
    is_stopped_owner_status_value(&string_or_empty(&value))
}

fn first_truthy(values: &[JsValue]) -> JsValue {
    values
        .iter()
        .find(|value| js_truthy(value))
        .cloned()
        .unwrap_or(JsValue::UNDEFINED)
}

fn first_non_nullish(values: &[JsValue]) -> JsValue {
    values
        .iter()
        .find(|value| !value.is_null() && !value.is_undefined())
        .cloned()
        .unwrap_or(JsValue::UNDEFINED)
}

fn assert_no_transport_wrappers_value(text: &str, label: &str) -> Result<(), JsValue> {
    if contains_transport_wrapper_value(text) {
        return Err(assertion_error(&format!(
            "{label} contains diagnostic transport wrapper text\n\ntrue !== false\n"
        )));
    }
    Ok(())
}

#[wasm_bindgen(js_name = assertNoTransportWrappers)]
pub fn assert_no_transport_wrappers(text: JsValue, label: JsValue) -> Result<(), JsValue> {
    assert_no_transport_wrappers_value(&string_or_empty(&text), &string_or_empty(&label))
}

fn regex_array(value: &JsValue) -> Array {
    if Array::is_array(value) {
        value.clone().unchecked_into()
    } else {
        Array::new()
    }
}

#[wasm_bindgen(js_name = assertDirectRawPayload)]
pub fn assert_direct_raw_payload(options: JsValue) -> Result<(), JsValue> {
    let text_value = get(&options, "text");
    let text = string_or_empty(&text_value);
    let label = string_or_empty(&get(&options, "label"));
    if text.trim().is_empty() {
        return Err(assertion_error(&format!("{label} raw payload is empty")));
    }
    assert_no_transport_wrappers_value(&text, &label)?;

    for value in regex_array(&get(&options, "mustMatch")).iter() {
        let regex: RegExp = value.unchecked_into();
        if !regex.test(&text) {
            return Err(assertion_error(&format!(
                "{label} did not match {}",
                String::from(regex.to_string())
            )));
        }
    }
    for value in regex_array(&get(&options, "mustNotMatch")).iter() {
        let regex: RegExp = value.unchecked_into();
        if regex.test(&text) {
            return Err(assertion_error(&format!(
                "{label} unexpectedly matched {}",
                String::from(regex.to_string())
            )));
        }
    }
    Ok(())
}

#[wasm_bindgen(js_name = assertRuntimeLogReport)]
pub fn assert_runtime_log_report(report: JsValue, options: JsValue) -> Result<(), JsValue> {
    let expected_version = "kgw_runtime_logs_v1";
    let actual_version_value = get(&report, "version");
    let actual_version = actual_version_value.as_string().unwrap_or_default();
    if actual_version_value.as_string().as_deref() != Some(expected_version) {
        return Err(strict_string_mismatch(
            "runtime log report version",
            &actual_version,
            expected_version,
            true,
        ));
    }

    let entries_value = get(&report, "entries");
    if !Array::is_array(&entries_value) {
        return Err(assertion_error(
            "runtime log report entries must be an array",
        ));
    }
    let entries: Array = entries_value.unchecked_into();
    let expected_network = string_or_empty(&get(&options, "network"));
    let expected_role = string_or_empty(&get(&options, "role"));
    if entries.length() == 0 {
        return Err(assertion_error(&format!(
            "expected raw {expected_role} entries for {expected_network}"
        )));
    }

    let bridge_expected = get(&options, "bridgeInstanceId");
    for entry in entries.iter() {
        let network = string_or_empty(&first_truthy(&[
            get(&entry, "network"),
            JsValue::from_str(""),
        ]))
        .to_lowercase();
        if network != expected_network {
            return Err(strict_string_mismatch(
                "entry network",
                &network,
                &expected_network,
                true,
            ));
        }

        let role = string_or_empty(&first_truthy(&[
            get(&entry, "runtimeRole"),
            get(&entry, "runtime_role"),
            JsValue::from_str(""),
        ]))
        .to_lowercase();
        if role != expected_role {
            return Err(strict_string_mismatch(
                "entry runtime role",
                &role,
                &expected_role,
                false,
            ));
        }

        let stream = string_or_empty(&first_truthy(&[
            get(&entry, "stream"),
            JsValue::from_str(""),
        ]))
        .to_lowercase();
        if !matches!(stream.as_str(), "stdout" | "stderr") {
            return Err(assertion_error("entry stream"));
        }

        if js_truthy(&bridge_expected) {
            let actual_bridge = string_or_empty(&first_truthy(&[
                get(&entry, "bridgeInstanceId"),
                get(&entry, "bridge_instance_id"),
                JsValue::from_str(""),
            ]));
            let expected_bridge = string_or_empty(&bridge_expected);
            if actual_bridge != expected_bridge {
                return Err(strict_string_mismatch(
                    "bridge instance",
                    &actual_bridge,
                    &expected_bridge,
                    false,
                ));
            }
        }

        let raw_text = string_or_empty(&first_non_nullish(&[
            get(&entry, "rawText"),
            get(&entry, "raw_text"),
            JsValue::from_str(""),
        ]));
        assert_no_transport_wrappers_value(&raw_text, "runtime rawText")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashing_newlines_and_line_counts_match_legacy_contract() {
        assert_eq!(
            sha256_hex_text("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            normalize_clipboard_text_value("a\nb\rc\r\nd"),
            "a\r\nb\r\nc\r\nd"
        );
        assert_eq!(line_count_value(""), 0);
        assert_eq!(line_count_value("a\nb\rc\r\nd"), 4);
    }

    #[test]
    fn transport_wrapper_detection_matches_expected_boundaries() {
        assert!(!contains_transport_wrapper_value("plain child output"));
        assert!(contains_transport_wrapper_value("[KGW_CHILD_STDOUT] hello"));
        assert!(contains_transport_wrapper_value(
            r#"  {"stage":"x","network":"mainnet","source":"child"}"#
        ));
        assert!(contains_transport_wrapper_value(
            r#"{"stage":"x","network":"mainnet","eventKind":"diagnostic_transport_record"}"#
        ));
        assert!(!contains_transport_wrapper_value(
            r#"{"stage":"x","network":"mainnet","eventKind":"normal"}"#
        ));
    }

    #[test]
    fn key_value_parser_preserves_input_order_and_legacy_empty_key() {
        assert_eq!(
            parse_key_value_line_value(
                " pid = 0012 ; running = false ; invalid ; =x ; role = node "
            ),
            vec![
                ("pid".to_owned(), "0012".to_owned()),
                ("running".to_owned(), "false".to_owned()),
                ("".to_owned(), "x".to_owned()),
                ("role".to_owned(), "node".to_owned()),
            ]
        );
    }

    #[test]
    fn stopped_owner_semantics_match_legacy() {
        assert!(is_stopped_owner_status_value(
            "role=node;running=false;pid=42"
        ));
        assert!(!is_stopped_owner_status_value(
            "role=node;running=true;state=stopped"
        ));
        assert!(is_stopped_owner_status_value("no node worker status yet"));
        assert!(is_stopped_owner_status_value(
            "role=node;running=;state=stopped"
        ));
        assert!(!is_stopped_owner_status_value(
            "role=node;running=;state=ready"
        ));
    }
}
