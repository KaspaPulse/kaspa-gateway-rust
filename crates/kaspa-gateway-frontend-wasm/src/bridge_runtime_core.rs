use js_sys::{JSON, Object, Reflect};
use wasm_bindgen::prelude::*;

fn present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if !present(target) {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn set(target: &JsValue, name: &str, value: &JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(name), value);
}

fn stringify_runtime_result_value(value: &JsValue) -> String {
    if value.is_null() || value.is_undefined() {
        return "No response".to_owned();
    }
    if let Some(text) = value.as_string() {
        return text;
    }
    JSON::stringify(value)
        .ok()
        .map(|text| crate::js_string_owned(text.as_ref()))
        .unwrap_or_else(|| crate::js_string_owned(value))
}
fn normalize_runtime_error_value(error: &JsValue) -> String {
    if error.is_null() || error.is_undefined() {
        return "Unknown backend error".to_owned();
    }
    if let Some(text) = error.as_string() {
        return text;
    }
    let message = property(error, "message");
    if crate::js_boolean(&message) {
        return crate::js_string_owned(&message);
    }
    JSON::stringify(error)
        .ok()
        .map(|text| crate::js_string_owned(text.as_ref()))
        .unwrap_or_else(|| crate::js_string_owned(error))
}

fn parse_runtime_fields_text(raw: &str) -> Vec<(String, String)> {
    raw.split(';')
        .filter_map(|part| {
            let index = part.find('=')?;
            if index == 0 {
                return None;
            }
            let key = part[..index].trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_owned(), part[index + 1..].trim().to_owned()))
        })
        .collect()
}
fn last_field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .rev()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_str())
}

fn runtime_running_text(raw: &str) -> bool {
    let fields = parse_runtime_fields_text(raw);
    let ready =
        last_field(&fields, "readiness").is_some_and(|value| value.eq_ignore_ascii_case("READY"));
    ready
        && (matches!(last_field(&fields, "running"), Some("true"))
            || matches!(last_field(&fields, "node_running"), Some("true"))
            || matches!(last_field(&fields, "official_core_running"), Some("true"))
            || matches!(last_field(&fields, "bridge_running"), Some("true"))
            || matches!(last_field(&fields, "bridge_owner_active"), Some("true"))
            || raw.to_ascii_lowercase().contains("running=true"))
}

fn r51_running_text(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.contains("readiness=ready")
        && (raw.contains("running=true")
            || raw.contains("bridge_running=true")
            || raw.contains("bridge_owner_active=true"))
}
fn runtime_error_text(raw: &str) -> String {
    let fields = parse_runtime_fields_text(raw);
    let error = last_field(&fields, "runtime_error")
        .or_else(|| last_field(&fields, "runtimeError"))
        .unwrap_or("")
        .trim();
    if error.is_empty() || error.eq_ignore_ascii_case("none") {
        String::new()
    } else {
        error.to_owned()
    }
}

fn normalize_node_mode_text(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn preview_declares_inprocess_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let no_ws: String = lower.chars().filter(|ch| !ch.is_whitespace()).collect();
    if [
        "--node-mode=in-process",
        "--node-mode=inprocess",
        "node-mode=in-process",
        "node-mode=inprocess",
        "node_mode=in-process",
        "node_mode=inprocess",
    ]
    .iter()
    .any(|needle| no_ws.contains(needle))
    {
        return true;
    }
    let collapsed = collapse_whitespace(&lower);
    collapsed.contains("--node-mode in-process") || collapsed.contains("--node-mode inprocess")
}
#[wasm_bindgen(js_name = bridgeStringifyRuntimeResult)]
pub fn bridge_stringify_runtime_result(value: JsValue) -> String {
    stringify_runtime_result_value(&value)
}

#[wasm_bindgen(js_name = bridgeNormalizeRuntimeError)]
pub fn bridge_normalize_runtime_error(error: JsValue) -> String {
    normalize_runtime_error_value(&error)
}

#[wasm_bindgen(js_name = bridgeParseRuntimeKeyValueResponse)]
pub fn bridge_parse_runtime_key_value_response(value: JsValue) -> JsValue {
    let raw = stringify_runtime_result_value(&value).trim().to_owned();
    let fields_object = Object::new();
    if !raw.is_empty() && raw.contains('=') {
        for (key, value) in parse_runtime_fields_text(&raw) {
            set(fields_object.as_ref(), &key, &JsValue::from_str(&value));
        }
    }
    let output = Object::new();
    set(output.as_ref(), "raw", &JsValue::from_str(&raw));
    set(output.as_ref(), "fields", fields_object.as_ref());
    output.into()
}
#[wasm_bindgen(js_name = bridgeV7RuntimeRunningFromText)]
pub fn bridge_v7_runtime_running_from_text(value: JsValue) -> bool {
    runtime_running_text(&stringify_runtime_result_value(&value))
}

#[wasm_bindgen(js_name = bridgeR51IsRunning)]
pub fn bridge_r51_is_running(value: JsValue) -> bool {
    r51_running_text(&stringify_runtime_result_value(&value))
}

#[wasm_bindgen(js_name = bridgeRuntimeErrorFromStatus)]
pub fn bridge_runtime_error_from_status(value: JsValue) -> String {
    runtime_error_text(&stringify_runtime_result_value(&value))
}

#[wasm_bindgen(js_name = bridgeNormalizeNodeModeR65F)]
pub fn bridge_normalize_node_mode_r65f(value: String) -> String {
    normalize_node_mode_text(&value)
}

#[wasm_bindgen(js_name = bridgePreviewDeclaresInprocessR65F)]
pub fn bridge_preview_declares_inprocess_r65f(value: String) -> bool {
    preview_declares_inprocess_text(&value)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_fields_match_legacy_split_contract() {
        assert_eq!(
            parse_runtime_fields_text(
                " role = bridge ; running=true; x=a=b ;bad; =skip; readiness = READY "
            ),
            vec![
                ("role".to_owned(), "bridge".to_owned()),
                ("running".to_owned(), "true".to_owned()),
                ("x".to_owned(), "a=b".to_owned()),
                ("readiness".to_owned(), "READY".to_owned()),
            ]
        );
    }

    #[test]
    fn runtime_running_requires_ready_and_legacy_running_evidence() {
        assert!(runtime_running_text("readiness=READY;running=true"));
        assert!(runtime_running_text(
            "readiness=ready;bridge_owner_active=true"
        ));
        assert!(runtime_running_text("readiness=READY;note=running=true"));
        assert!(!runtime_running_text("readiness=STARTING;running=true"));
        assert!(!runtime_running_text("readiness=READY;running=false"));
    }

    #[test]
    fn r51_running_matches_legacy_predicate() {
        assert!(r51_running_text("x=1;readiness=READY;bridge_running=true"));
        assert!(r51_running_text("READINESS=ready;bridge_owner_active=true"));
        assert!(!r51_running_text("readiness=READY;bridge_running=TRUE"));
        assert!(!r51_running_text("readiness=WAITING;running=true"));
    }
    #[test]
    fn runtime_error_prefers_runtime_error_and_filters_none() {
        assert_eq!(runtime_error_text("runtime_error= boom "), "boom");
        assert_eq!(runtime_error_text("runtimeError=second"), "second");
        assert_eq!(runtime_error_text("runtime_error=none"), "");
        assert_eq!(runtime_error_text("running=true"), "");
    }

    #[test]
    fn node_mode_normalization_matches_legacy_contract() {
        assert_eq!(normalize_node_mode_text("In-Process"), "inprocess");
        assert_eq!(normalize_node_mode_text(" in_proc "), "inproc");
        assert_eq!(normalize_node_mode_text("EXTERNAL"), "external");
    }

    #[test]
    fn inprocess_preview_detection_matches_supported_forms() {
        for value in [
            "--node-mode=in-process",
            "--node-mode = inprocess",
            "--node-mode in-process",
            "--node-mode   inprocess",
            "node-mode=in-process",
            "node_mode=inprocess",
        ] {
            assert!(preview_declares_inprocess_text(value), "{value}");
        }
        assert!(!preview_declares_inprocess_text("--node-mode=external"));
        assert!(!preview_declares_inprocess_text("node_mode=remote"));
    }
}
