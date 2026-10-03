use js_sys::{Array, Function, Reflect};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use wasm_bindgen::{JsCast, prelude::*};

const RAW_LOG_BUFFER_LIMIT: usize = 4096;

#[derive(Clone)]
struct RawLogEntry {
    sequence: u64,
    raw_text: String,
}

thread_local! {
    static RAW_LOG_BUFFERS: RefCell<HashMap<String, BTreeMap<u64, RawLogEntry>>> =
        RefCell::new(HashMap::new());
}

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
}

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

fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, arg).ok()
}

fn document() -> JsValue {
    property(&global(), "document")
}

fn normalized_lower(value: &str, fallback: &str) -> String {
    let value = value.trim().to_lowercase();
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value
    }
}

fn buffer_key(net: &str, role: &str) -> String {
    format!(
        "{}:{}",
        normalized_lower(net, ""),
        normalized_lower(role, "bridge")
    )
}

fn transport_wrapper_text(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("kgw_raw_process_log_v1")
        && (rest.is_empty() || rest.starts_with(';'))
    {
        return true;
    }

    let body = if lower.starts_with("[kgw_child_stdout]") {
        text["[KGW_CHILD_STDOUT]".len()..].trim_start()
    } else if lower.starts_with("[kgw_child_stderr]") {
        text["[KGW_CHILD_STDERR]".len()..].trim_start()
    } else {
        text
    };
    let compact = body
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    compact.starts_with('{')
        && [
            "\"eventkind\":\"diagnostic_transport_record\"",
            "'eventkind':'diagnostic_transport_record'",
            "\"eventkind\":'diagnostic_transport_record'",
            "'eventkind':\"diagnostic_transport_record\"",
        ]
        .iter()
        .any(|needle| compact.contains(needle))
}

fn legacy_transport_text(report: &JsValue) -> String {
    if let Some(text) = report.as_string() {
        return text;
    }
    if !report.is_object() || report.is_null() || Array::is_array(report) {
        return String::new();
    }
    if Array::is_array(&property(report, "entries")) {
        return String::new();
    }
    for name in ["rawText", "raw_text", "line"] {
        let value = property(report, name);
        if present(&value) {
            return crate::js_string_owned(&value);
        }
    }
    String::new()
}

fn first_present(target: &JsValue, names: &[&str]) -> JsValue {
    for name in names {
        let value = property(target, name);
        if present(&value) {
            return value;
        }
    }
    JsValue::UNDEFINED
}

fn sequence_is_valid(sequence: f64) -> bool {
    sequence.is_finite()
        && sequence >= 0.0
        && sequence.fract() == 0.0
        && sequence <= 9_007_199_254_740_991.0
}

fn normalize_entry(
    entry: &JsValue,
    expected_net: &str,
    expected_role: &str,
) -> Option<RawLogEntry> {
    if !entry.is_object() || entry.is_null() {
        return None;
    }
    let raw_text = first_present(entry, &["rawText", "raw_text", "line"]);
    if !present(&raw_text) {
        return None;
    }
    let sequence = crate::js_number(&property(entry, "sequence"));
    if !sequence_is_valid(sequence) {
        return None;
    }

    let network_value = property(entry, "network");
    let network = if crate::js_boolean(&network_value) {
        normalized_lower(&crate::js_string_owned(&network_value), "")
    } else {
        normalized_lower(expected_net, "")
    };
    let role_value = first_present(entry, &["runtimeRole", "runtime_role"]);
    let runtime_role = if crate::js_boolean(&role_value) {
        normalized_lower(&crate::js_string_owned(&role_value), "bridge")
    } else {
        normalized_lower(expected_role, "bridge")
    };
    let stream = normalized_lower(&crate::js_string_owned(&property(entry, "stream")), "");
    if network != normalized_lower(expected_net, "")
        || runtime_role != normalized_lower(expected_role, "bridge")
        || (stream != "stdout" && stream != "stderr")
    {
        return None;
    }

    Some(RawLogEntry {
        sequence: sequence as u64,
        raw_text: crate::js_string_owned(&raw_text),
    })
}

fn log_output(net: &str) -> JsValue {
    call1(
        &document(),
        "getElementById",
        &JsValue::from_str(&format!("bridge-{net}-logOutput")),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

fn empty_state(net: &str) -> JsValue {
    call1(
        &document(),
        "getElementById",
        &JsValue::from_str(&format!("bridge-{net}-logEmpty")),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

fn storage_get(key: &str) -> Option<String> {
    let mut storage = property(&global(), "localStorage");
    if !present(&storage) {
        storage = property(&window(), "localStorage");
    }
    call1(&storage, "getItem", &JsValue::from_str(key)).and_then(|value| value.as_string())
}

fn auto_scroll_enabled(net: &str) -> bool {
    storage_get(&format!("kgw.bridge.log.autoscroll.{net}")).as_deref() != Some("0")
}

fn visible_text(net: &str, role: &str) -> String {
    let key = buffer_key(net, role);
    RAW_LOG_BUFFERS.with(|buffers| {
        buffers
            .borrow()
            .get(&key)
            .map(|records| {
                records
                    .values()
                    .map(|entry| entry.raw_text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    })
}

fn render_impl(net: &str, role: &str) -> bool {
    let out = log_output(net);
    if !present(&out) {
        return false;
    }
    let text = visible_text(net, role);
    set(&out, "textContent", &JsValue::from_str(&text));

    let empty = empty_state(net);
    if present(&empty) {
        set(&empty, "hidden", &JsValue::from_bool(!text.is_empty()));
    }
    if auto_scroll_enabled(net) {
        set(&out, "scrollTop", &property(&out, "scrollHeight"));
    }
    true
}

fn apply_report_impl(net: &str, role: &str, report: &JsValue) -> u32 {
    if transport_wrapper_text(&legacy_transport_text(report)) {
        return 0;
    }
    let entries_value = property(report, "entries");
    let entries = if Array::is_array(&entries_value) {
        Array::from(&entries_value)
    } else {
        Array::new()
    };
    let key = buffer_key(net, role);
    let mut accepted = 0_u32;
    RAW_LOG_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let records = buffers.entry(key).or_insert_with(BTreeMap::new);
        for entry in entries.iter() {
            let Some(normalized) = normalize_entry(&entry, net, role) else {
                continue;
            };
            if records.contains_key(&normalized.sequence) {
                continue;
            }
            records.insert(normalized.sequence, normalized);
            accepted += 1;
        }
        while records.len() > RAW_LOG_BUFFER_LIMIT {
            let Some(first) = records.keys().next().copied() else {
                break;
            };
            records.remove(&first);
        }
    });
    let _ = render_impl(net, role);
    accepted
}

fn clear_impl(net: &str, role: &str) {
    let key = buffer_key(net, role);
    RAW_LOG_BUFFERS.with(|buffers| {
        buffers.borrow_mut().remove(&key);
    });
    let _ = render_impl(net, role);
}

#[wasm_bindgen(js_name = bridgeRenderRawLogBuffer)]
pub fn bridge_render_raw_log_buffer(net: String, role: String, _instance_id: String) -> bool {
    render_impl(&net, &role)
}

#[wasm_bindgen(js_name = bridgeApplyRuntimeLogReport)]
pub fn bridge_apply_runtime_log_report(
    net: String,
    role: String,
    report: JsValue,
    _instance_id: String,
) -> u32 {
    apply_report_impl(&net, &role, &report)
}

#[wasm_bindgen(js_name = bridgeClearRawLogBuffer)]
pub fn bridge_clear_raw_log_buffer(net: String, role: String, _instance_id: String) {
    clear_impl(&net, &role);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_wide_buffer_key_ignores_ui_instance() {
        assert_eq!(buffer_key(" MainNet ", " Bridge "), "mainnet:bridge");
        assert_eq!(buffer_key("testnet10", ""), "testnet10:bridge");
    }

    #[test]
    fn transport_wrapper_detection_matches_frontend_contract() {
        assert!(transport_wrapper_text(
            "kgw_raw_process_log_v1;network=mainnet"
        ));
        assert!(transport_wrapper_text(
            "[KGW_CHILD_STDERR] {\"eventKind\":\"diagnostic_transport_record\"}"
        ));
        assert!(transport_wrapper_text(
            "{\"eventKind\": \"diagnostic_transport_record\"}"
        ));
        assert!(!transport_wrapper_text("official raw stdout text"));
    }

    #[test]
    fn sequence_validation_matches_javascript_safe_integer_contract() {
        assert!(sequence_is_valid(0.0));
        assert!(sequence_is_valid(9_007_199_254_740_991.0));
        assert!(!sequence_is_valid(-1.0));
        assert!(!sequence_is_valid(1.5));
        assert!(!sequence_is_valid(9_007_199_254_740_992.0));
    }
}
