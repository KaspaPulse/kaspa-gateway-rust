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

fn runtime_command_for_action_text(action: &str) -> &'static str {
    match action {
        "start" => "kgw_kgw_apply_node_settings_v1",
        "stop" => "kgw_kgw_disable_network_v1",
        _ => "",
    }
}

fn strip_required_whitespace(value: &str) -> Option<&str> {
    let trimmed = value.trim_start();
    (trimmed.len() < value.len()).then_some(trimmed)
}

fn parallel_worker_started_text(lower: &str) -> bool {
    const OWNER: &str = "parallel-owned-self-worker";
    let mut offset = 0usize;
    while let Some(relative) = lower[offset..].find(OWNER) {
        let end = offset + relative + OWNER.len();
        let rest = &lower[end..];
        if let Some(after_space) = strip_required_whitespace(rest) {
            if after_space.starts_with("started") {
                return true;
            }
            if let Some(after_already) = after_space.strip_prefix("already")
                && let Some(after_second_space) = strip_required_whitespace(after_already)
                && after_second_space.starts_with("running")
            {
                return true;
            }
        }
        offset = end;
        if offset >= lower.len() {
            break;
        }
    }
    false
}

fn start_outcome_text(raw: &str, fields: &[(String, String)]) -> (bool, bool) {
    let lower = raw.to_ascii_lowercase();
    let ready =
        last_field(fields, "readiness").is_some_and(|value| value.eq_ignore_ascii_case("READY"));
    let bridge_role_started = lower.contains("role=bridge")
        && (lower.contains("started")
            || lower.contains("running=true")
            || lower.contains("already running"));
    let confirmed_started = ready
        && (parallel_worker_started_text(&lower)
            || bridge_role_started
            || matches!(last_field(fields, "running"), Some("true"))
            || matches!(last_field(fields, "bridge_running"), Some("true"))
            || matches!(last_field(fields, "bridge_owner_active"), Some("true")));
    let blocked = matches!(last_field(fields, "start_blocked"), Some("true"))
        || matches!(last_field(fields, "start_allowed"), Some("false"))
        || lower.contains("blocked")
        || lower.contains("not enabled")
        || lower.contains("failed");
    (confirmed_started, blocked)
}

fn stop_outcome_text(raw: &str, fields: &[(String, String)]) -> (bool, bool, bool) {
    let _ = raw;
    let forced = matches!(last_field(fields, "forced"), Some("true"));
    let stop_failed = matches!(last_field(fields, "stop_failed"), Some("true"));
    let confirmed_stopped = matches!(last_field(fields, "running"), Some("false"))
        && (matches!(last_field(fields, "graceful"), Some("true"))
            || forced
            || stop_failed
            || matches!(last_field(fields, "already_stopped"), Some("true")));
    (confirmed_stopped, forced, stop_failed)
}

fn start_was_inprocess_text(field_mode: &str, ui_mode: &str, preview: &str) -> bool {
    let field_mode = normalize_node_mode_text(field_mode);
    let ui_mode = normalize_node_mode_text(ui_mode);
    field_mode == "inprocess"
        || field_mode == "inproc"
        || ui_mode == "inprocess"
        || ui_mode == "inproc"
        || preview_declares_inprocess_text(preview)
}

#[wasm_bindgen(js_name = bridgeRuntimeCommandForAction)]
pub fn bridge_runtime_command_for_action(action: String) -> String {
    runtime_command_for_action_text(&action).to_owned()
}

#[wasm_bindgen(js_name = bridgeRuntimeActionOutcome)]
pub fn bridge_runtime_action_outcome(action: String, value: JsValue) -> JsValue {
    let raw = stringify_runtime_result_value(&value).trim().to_owned();
    let fields = parse_runtime_fields_text(&raw);
    let fields_object = Object::new();
    for (key, value) in &fields {
        set(fields_object.as_ref(), key, &JsValue::from_str(value));
    }

    let (confirmed_started, blocked) = start_outcome_text(&raw, &fields);
    let (confirmed_stopped, forced, stop_failed) = stop_outcome_text(&raw, &fields);
    let output = Object::new();
    set(output.as_ref(), "action", &JsValue::from_str(&action));
    set(output.as_ref(), "raw", &JsValue::from_str(&raw));
    set(output.as_ref(), "fields", fields_object.as_ref());
    set(
        output.as_ref(),
        "confirmedStarted",
        &JsValue::from_bool(confirmed_started),
    );
    set(output.as_ref(), "blocked", &JsValue::from_bool(blocked));
    set(
        output.as_ref(),
        "confirmedStopped",
        &JsValue::from_bool(confirmed_stopped),
    );
    set(output.as_ref(), "forced", &JsValue::from_bool(forced));
    set(
        output.as_ref(),
        "stopFailed",
        &JsValue::from_bool(stop_failed),
    );
    output.into()
}

#[wasm_bindgen(js_name = bridgeStartWasInprocessR65F)]
pub fn bridge_start_was_inprocess_r65f(fields: JsValue, ui_mode: String, preview: String) -> bool {
    let node_mode = property(&fields, "node_mode");
    let field_value = if crate::js_boolean(&node_mode) {
        node_mode
    } else {
        property(&fields, "nodeMode")
    };
    start_was_inprocess_text(&crate::js_string_owned(&field_value), &ui_mode, &preview)
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

    #[test]
    fn runtime_action_command_mapping_matches_legacy_contract() {
        assert_eq!(
            runtime_command_for_action_text("start"),
            "kgw_kgw_apply_node_settings_v1"
        );
        assert_eq!(
            runtime_command_for_action_text("stop"),
            "kgw_kgw_disable_network_v1"
        );
        assert_eq!(runtime_command_for_action_text("status"), "");
        assert_eq!(runtime_command_for_action_text(""), "");
    }

    #[test]
    fn start_outcome_matches_legacy_ready_and_blocked_contract() {
        for raw in [
            "readiness=READY;parallel-owned-self-worker started",
            "readiness=READY;parallel-owned-self-worker   already   running",
            "readiness=READY;role=bridge;state=started",
            "readiness=READY;running=true",
            "readiness=READY;bridge_running=true",
            "readiness=READY;bridge_owner_active=true",
        ] {
            let fields = parse_runtime_fields_text(raw);
            assert!(start_outcome_text(raw, &fields).0, "{raw}");
        }

        for raw in [
            "readiness=STARTING;running=true",
            "readiness=READY;running=false",
            "running=true",
        ] {
            let fields = parse_runtime_fields_text(raw);
            assert!(!start_outcome_text(raw, &fields).0, "{raw}");
        }

        for raw in [
            "readiness=READY;running=true;start_blocked=true",
            "readiness=READY;running=true;start_allowed=false",
            "readiness=READY;running=true;reason=Blocked by policy",
            "readiness=READY;running=true;reason=not enabled",
            "readiness=READY;running=true;reason=failed",
        ] {
            let fields = parse_runtime_fields_text(raw);
            assert!(start_outcome_text(raw, &fields).1, "{raw}");
        }
    }

    #[test]
    fn stop_outcome_matches_legacy_terminal_contract() {
        for raw in [
            "running=false;graceful=true",
            "running=false;forced=true",
            "running=false;stop_failed=true",
            "running=false;already_stopped=true",
        ] {
            let fields = parse_runtime_fields_text(raw);
            assert!(stop_outcome_text(raw, &fields).0, "{raw}");
        }

        for raw in [
            "running=true;graceful=true",
            "running=false;graceful=false",
            "graceful=true",
        ] {
            let fields = parse_runtime_fields_text(raw);
            assert!(!stop_outcome_text(raw, &fields).0, "{raw}");
        }

        let forced = parse_runtime_fields_text("running=false;forced=true");
        assert_eq!(stop_outcome_text("", &forced), (true, true, false));
        let failed = parse_runtime_fields_text("running=false;stop_failed=true");
        assert_eq!(stop_outcome_text("", &failed), (true, false, true));
    }

    #[test]
    fn start_was_inprocess_matches_legacy_sources() {
        assert!(start_was_inprocess_text("in-process", "external", ""));
        assert!(start_was_inprocess_text("in_proc", "external", ""));
        assert!(start_was_inprocess_text("", "in-process", ""));
        assert!(start_was_inprocess_text("", "inproc", ""));
        assert!(start_was_inprocess_text(
            "",
            "external",
            "--node-mode=in-process"
        ));
        assert!(!start_was_inprocess_text(
            "",
            "external",
            "--node-mode=external"
        ));
    }
}
