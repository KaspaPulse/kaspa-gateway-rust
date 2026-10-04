use js_sys::{Date, Function, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, future_to_promise, spawn_local};

const R51_LIVE_REFRESH_MS: f64 = 700.0;

thread_local! {
    static R51_LAST_STATUS: RefCell<BTreeMap<String, String>> = const { RefCell::new(BTreeMap::new()) };
    static R51_LAST_ACTIVITY_NOTICE: RefCell<BTreeMap<String, f64>> = const { RefCell::new(BTreeMap::new()) };
    static R51_STATUS_IN_FLIGHT: RefCell<BTreeMap<String, Promise>> = const { RefCell::new(BTreeMap::new()) };
    static R51_LOGS_IN_FLIGHT: RefCell<BTreeMap<String, Promise>> = const { RefCell::new(BTreeMap::new()) };
    static R51_LIVE_TIMER: RefCell<JsValue> = const { RefCell::new(JsValue::UNDEFINED) };
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

fn call2(target: &JsValue, name: &str, first: &JsValue, second: &JsValue) -> Option<JsValue> {
    function(target, name)?.call2(target, first, second).ok()
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn set_attribute(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn set_style(target: &JsValue, name: &str, value: &str) {
    set(&property(target, "style"), name, &JsValue::from_str(value));
}

fn class_contains(target: &JsValue, class_name: &str) -> bool {
    call1(
        &property(target, "classList"),
        "contains",
        &JsValue::from_str(class_name),
    )
    .is_some_and(|value| crate::js_boolean(&value))
}

fn text(value: &JsValue) -> String {
    if present(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    }
}

async fn r51_invoke_runtime(
    callbacks: &JsValue,
    command: &str,
    net: &str,
) -> Result<JsValue, JsValue> {
    let Some(callback) = function(callbacks, "invokeRuntime") else {
        return Err(JsValue::from_str(
            "Bridge refresh invokeRuntime callback is unavailable",
        ));
    };
    let result = callback.call2(
        callbacks,
        &JsValue::from_str(command),
        &JsValue::from_str(net),
    )?;
    JsFuture::from(Promise::resolve(&result)).await
}

fn r51_transition_active(callbacks: &JsValue, net: &str) -> bool {
    call1(callbacks, "transitionActive", &JsValue::from_str(net))
        .is_some_and(|value| crate::js_boolean(&value))
}

fn r51_active_raw_log_instance_id(callbacks: &JsValue, net: &str) -> String {
    call1(callbacks, "activeRawLogInstanceId", &JsValue::from_str(net))
        .map(|value| text(&value))
        .unwrap_or_default()
}

fn js_or_empty(value: &JsValue) -> String {
    if crate::js_boolean(value) {
        text(value)
    } else {
        String::new()
    }
}

fn set_runtime_error_inner(net: &str, error_text: &JsValue, error_source: &JsValue) -> bool {
    let error_node = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(
            net.to_owned(),
            "runtimeError".to_owned(),
        ),
    );
    if !present(&error_node) {
        return false;
    }
    let message = js_or_empty(error_text).trim().to_owned();
    set(&error_node, "textContent", &JsValue::from_str(&message));
    set(
        &error_node,
        "hidden",
        &JsValue::from_bool(message.is_empty()),
    );
    set(
        &dataset(&error_node),
        "runtimeErrorSource",
        &JsValue::from_str(&js_or_empty(error_source)),
    );
    let _ = crate::apply_status_tone(error_node, JsValue::from_str("error"));
    true
}

fn set_runtime_activity_inner(net: &str, message: &JsValue, state: &JsValue) -> bool {
    let status_node = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(
            net.to_owned(),
            "runtimeStatus".to_owned(),
        ),
    );
    if !present(&status_node) {
        return false;
    }
    let message = js_or_empty(message).trim().to_owned();
    set(&status_node, "textContent", &JsValue::from_str(&message));
    let explicit_state = js_or_empty(state);
    let tone = if explicit_state.is_empty() {
        let policy_status = crate::bridge_frontend_helpers::bridge_by_id(
            crate::bridge_frontend_helpers::bridge_element_id(
                net.to_owned(),
                "policyStatus".to_owned(),
            ),
        );
        text(&property(&dataset(&policy_status), "state"))
    } else {
        explicit_state
    };
    let _ = crate::apply_status_tone(status_node, JsValue::from_str(&tone));
    true
}

fn mark_restart_required_inner(net: &str) -> bool {
    let authority = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(
            net.to_owned(),
            "settingsAuthority".to_owned(),
        ),
    );
    if !present(&authority) {
        return false;
    }
    let status = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(
            net.to_owned(),
            "runtimeStatus".to_owned(),
        ),
    );
    let running = text(&property(&status, "textContent"))
        .to_ascii_lowercase()
        .contains("running");
    set(
        &authority,
        "textContent",
        &JsValue::from_str(if running {
            "Restart required to apply changed effective settings"
        } else {
            "Effective settings apply on next Start"
        }),
    );
    set(
        &dataset(&authority),
        "restartRequired",
        &JsValue::from_str(if running { "true" } else { "false" }),
    );
    true
}

fn r51_set_runtime_error(net: &str, message: &str, source: &str) {
    let _ = set_runtime_error_inner(net, &JsValue::from_str(message), &JsValue::from_str(source));
}

fn r51_set_runtime_activity(net: &str, message: &str, state: &str) {
    let _ = set_runtime_activity_inner(net, &JsValue::from_str(message), &JsValue::from_str(state));
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

fn r51_maybe_activity_notice(net: &str, status_text: &str) {
    let now = Date::now();
    let previous =
        R51_LAST_ACTIVITY_NOTICE.with(|items| items.borrow().get(net).copied().unwrap_or(0.0));
    if now - previous < 15_000.0 || !r51_running_text(status_text) {
        return;
    }
    R51_LAST_ACTIVITY_NOTICE.with(|items| {
        items.borrow_mut().insert(net.to_owned(), now);
    });
}

fn r51_logs_task(net: &str, callbacks: &JsValue) -> Promise {
    if let Some(existing) = R51_LOGS_IN_FLIGHT.with(|items| items.borrow().get(net).cloned()) {
        return existing;
    }
    let net_owned = net.to_owned();
    let callbacks_owned = callbacks.clone();
    let promise = future_to_promise(async move {
        if let Ok(report) =
            r51_invoke_runtime(&callbacks_owned, "kgw_kgw_runtime_logs_v1", &net_owned).await
        {
            let instance_id = r51_active_raw_log_instance_id(&callbacks_owned, &net_owned);
            let _ = crate::bridge_raw_log::bridge_apply_runtime_log_report(
                net_owned.clone(),
                "bridge".to_owned(),
                report,
                instance_id,
            );
        }
        R51_LOGS_IN_FLIGHT.with(|items| {
            items.borrow_mut().remove(&net_owned);
        });
        Ok(JsValue::UNDEFINED)
    });
    R51_LOGS_IN_FLIGHT.with(|items| {
        items.borrow_mut().insert(net.to_owned(), promise.clone());
    });
    promise
}

fn r51_status_task(net: &str, callbacks: &JsValue) -> Option<Promise> {
    if r51_transition_active(callbacks, net) {
        return None;
    }
    if let Some(existing) = R51_STATUS_IN_FLIGHT.with(|items| items.borrow().get(net).cloned()) {
        return Some(existing);
    }
    let net_owned = net.to_owned();
    let callbacks_owned = callbacks.clone();
    let promise = future_to_promise(async move {
        match r51_invoke_runtime(&callbacks_owned, "kgw_runtime_owner_status_v1", &net_owned).await
        {
            Ok(raw) => {
                let status = stringify_runtime_result_value(&raw);
                let running = r51_running_text(&status);
                let runtime_error = runtime_error_text(&status);
                let fields = parse_runtime_fields_text(&status);
                let role = last_field(&fields, "role").unwrap_or("");
                let network = last_field(&fields, "network").unwrap_or("");
                let running_field = last_field(&fields, "running").unwrap_or("");
                let error_node = crate::bridge_frontend_helpers::bridge_by_id(
                    crate::bridge_frontend_helpers::bridge_element_id(
                        net_owned.clone(),
                        "runtimeError".to_owned(),
                    ),
                );
                if runtime_error.is_empty()
                    && role == "bridge"
                    && network == net_owned
                    && matches!(running_field, "true" | "false")
                    && text(&property(&dataset(&error_node), "runtimeErrorSource"))
                        == "status-refresh"
                {
                    r51_set_runtime_error(&net_owned, "", "");
                    r51_set_runtime_activity(
                        &net_owned,
                        if running {
                            "Bridge is running."
                        } else {
                            "Bridge is stopped."
                        },
                        if running { "running" } else { "stopped" },
                    );
                }

                bridge_r51_set_runtime_buttons(
                    net_owned.clone(),
                    running,
                    String::new(),
                    runtime_error.clone(),
                    status.clone(),
                );

                if !running && !runtime_error.is_empty() {
                    r51_set_runtime_error(&net_owned, &runtime_error, "");
                    r51_set_runtime_activity(
                        &net_owned,
                        "Bridge runtime failed after readiness.",
                        "failed",
                    );
                    let policy_status = crate::bridge_frontend_helpers::bridge_by_id(
                        crate::bridge_frontend_helpers::bridge_element_id(
                            net_owned.clone(),
                            "policyStatus".to_owned(),
                        ),
                    );
                    if present(&policy_status) {
                        let failed =
                            crate::bridge_frontend_helpers::bridge_translate_runtime_feedback(
                                "runtime.failed".to_owned(),
                                "Failed".to_owned(),
                            );
                        set(&policy_status, "textContent", &failed);
                        set(
                            &dataset(&policy_status),
                            "state",
                            &JsValue::from_str("failed"),
                        );
                        let _ =
                            crate::apply_status_tone(policy_status, JsValue::from_str("failed"));
                    }
                }

                let changed = R51_LAST_STATUS.with(|items| {
                    items
                        .borrow()
                        .get(&net_owned)
                        .is_none_or(|value| value != &status)
                });
                if changed {
                    R51_LAST_STATUS.with(|items| {
                        items.borrow_mut().insert(net_owned.clone(), status.clone());
                    });
                    let authority = crate::bridge_frontend_helpers::bridge_by_id(
                        crate::bridge_frontend_helpers::bridge_element_id(
                            net_owned.clone(),
                            "settingsAuthority".to_owned(),
                        ),
                    );
                    if present(&authority)
                        && (!running
                            || text(&property(&dataset(&authority), "restartRequired")) != "true")
                    {
                        set(
                            &authority,
                            "textContent",
                            &JsValue::from_str(if running {
                                "Effective settings are active for this runtime"
                            } else {
                                "Effective settings apply on next Start"
                            }),
                        );
                        set(
                            &dataset(&authority),
                            "restartRequired",
                            &JsValue::from_str("false"),
                        );
                    }
                }
                r51_maybe_activity_notice(&net_owned, &status);
            }
            Err(error) => {
                let message = format!(
                    "Status refresh failed: {}",
                    normalize_runtime_error_value(&error)
                );
                bridge_r51_set_runtime_unknown(
                    net_owned.clone(),
                    message,
                    "status-refresh".to_owned(),
                );
            }
        }
        R51_STATUS_IN_FLIGHT.with(|items| {
            items.borrow_mut().remove(&net_owned);
        });
        Ok(JsValue::UNDEFINED)
    });
    R51_STATUS_IN_FLIGHT.with(|items| {
        items.borrow_mut().insert(net.to_owned(), promise.clone());
    });
    Some(promise)
}

async fn r51_refresh_one_impl(net: String, callbacks: JsValue) {
    let logs = r51_logs_task(&net, &callbacks);
    let status = r51_status_task(&net, &callbacks);
    let _ = JsFuture::from(logs).await;
    if let Some(status) = status {
        let _ = JsFuture::from(status).await;
    }
}

fn r51_refresh_all_impl(_reason: &str, callbacks: &JsValue) {
    for net in crate::bridge_frontend_helpers::bridge_r51_keys().iter() {
        let net = text(&net);
        if !net.is_empty() {
            spawn_local(r51_refresh_one_impl(net, callbacks.clone()));
        }
    }
}

fn r51_start_live_refresh_impl(callbacks: JsValue) {
    R51_LIVE_TIMER.with(|timer| {
        let existing = timer.borrow().clone();
        if present(&existing) {
            let _ = call1(&window(), "clearInterval", &existing);
        }
    });
    r51_refresh_all_impl("initial", &callbacks);
    let poll_callbacks = callbacks.clone();
    let callback = Closure::wrap(Box::new(move || {
        r51_refresh_all_impl("poll", &poll_callbacks);
    }) as Box<dyn FnMut()>);
    let timer = call2(
        &window(),
        "setInterval",
        callback.as_ref(),
        &JsValue::from_f64(R51_LIVE_REFRESH_MS),
    )
    .unwrap_or(JsValue::UNDEFINED);
    R51_LIVE_TIMER.with(|value| *value.borrow_mut() = timer);
    callback.forget();
}

#[wasm_bindgen(js_name = bridgeR51RefreshOne)]
pub async fn bridge_r51_refresh_one(net: String, _reason: String, callbacks: JsValue) {
    r51_refresh_one_impl(net, callbacks).await;
}

#[wasm_bindgen(js_name = bridgeR51RefreshAll)]
pub fn bridge_r51_refresh_all(reason: String, callbacks: JsValue) {
    r51_refresh_all_impl(&reason, &callbacks);
}

#[wasm_bindgen(js_name = bridgeR51StartLiveRefresh)]
pub fn bridge_r51_start_live_refresh(callbacks: JsValue) {
    r51_start_live_refresh_impl(callbacks);
}

#[wasm_bindgen(js_name = bridgeR51SetRuntimeButtons)]
pub fn bridge_r51_set_runtime_buttons(
    net: String,
    running: bool,
    transition: String,
    runtime_error: String,
    status_text: String,
) {
    let panel = crate::bridge_instance_settings::bridge_r51_panel(net.clone());
    if !present(&panel) {
        return;
    }

    let network_enabled = crate::bridge_frontend_helpers::bridge_network_enabled(net.clone());
    let start = query(
        &panel,
        &format!(r#"[data-bridge-action="start"][data-net="{net}"]"#),
    );
    let stop = query(
        &panel,
        &format!(r#"[data-bridge-action="stop"][data-net="{net}"]"#),
    );
    let policy_status = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(net.clone(), "policyStatus".to_owned()),
    );

    let state = Object::new();
    set(state.as_ref(), "role", &JsValue::from_str("Bridge"));
    set(
        state.as_ref(),
        "enabled",
        &JsValue::from_bool(network_enabled),
    );
    set(state.as_ref(), "running", &JsValue::from_bool(running));
    set(
        state.as_ref(),
        "transition",
        &JsValue::from_str(&transition),
    );
    set(state.as_ref(), "error", &JsValue::from_str(&runtime_error));
    let presentation =
        crate::settings_runtime::runtime_presentation(state.into()).unwrap_or(JsValue::UNDEFINED);
    let process = text(&property(&presentation, "process"));
    let process_label = text(&property(&presentation, "processLabel"));
    let profile = text(&property(&presentation, "profile"));
    let parsed = bridge_parse_runtime_key_value_response(JsValue::from_str(&status_text));
    let fields = property(&parsed, "fields");
    let readiness = crate::settings_runtime::runtime_semantic_readiness(
        &fields,
        running,
        &transition,
        !runtime_error.is_empty(),
        "bridge",
    )
    .unwrap_or("Awaiting telemetry");

    if present(&policy_status) {
        set(
            &policy_status,
            "textContent",
            &JsValue::from_str(&format!("{process_label} | Readiness: {readiness}")),
        );
        set(
            &dataset(&policy_status),
            "state",
            &JsValue::from_str(&process.to_lowercase()),
        );
        let _ = crate::apply_status_tone(policy_status.clone(), JsValue::from_str(readiness));
    }

    let summary = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(net.clone(), "monitorState".to_owned()),
    );
    let observation = crate::settings_runtime::runtime_observation_summary(
        fields.clone(),
        JsValue::from_bool(running),
        JsValue::from_bool(net != "mainnet"),
    )
    .unwrap_or_default();
    let summary_text = format!(
        "{process_label} | Profile: {profile} | Startup readiness: {} | {observation}",
        readiness
    );
    let _ = crate::render_status_summary(summary, JsValue::from_str(&summary_text));

    let empty = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(net.clone(), "logEmpty".to_owned()),
    );
    if present(&empty) {
        let empty_text = if !runtime_error.is_empty() {
            "Bridge failed. Review the error in Settings and the logs below.".to_owned()
        } else if !transition.is_empty() {
            format!(
                "Bridge is {}. Waiting for runtime output.",
                process.to_lowercase()
            )
        } else if running {
            "Bridge is running. Waiting for log output.".to_owned()
        } else if crate::bridge_frontend_helpers::bridge_node_mode(net.clone()) == "inprocess" {
            "Bridge is stopped. Start Bridge to start its owned node and Stratum service."
                .to_owned()
        } else {
            "Bridge is stopped. Start Bridge checks the configured node connection before starting the service.".to_owned()
        };
        set(&empty, "textContent", &JsValue::from_str(&empty_text));
    }

    let settings_invalid = present(&query(&panel, r#"[aria-invalid="true"]"#))
        || class_contains(
            &crate::bridge_frontend_helpers::bridge_by_id(
                crate::bridge_frontend_helpers::bridge_element_id(
                    net.clone(),
                    "previewStatus".to_owned(),
                ),
            ),
            "kgw-field-error",
        );

    let next = query(&panel, r#"[data-bridge-action="monitor-next"]"#);
    if present(&next) {
        set(
            &next,
            "hidden",
            &JsValue::from_bool(running || !transition.is_empty()),
        );
        let next_text = if !network_enabled {
            "Enable Profile in Settings"
        } else if settings_invalid {
            "Review Settings"
        } else {
            "Start Bridge"
        };
        set(&next, "textContent", &JsValue::from_str(next_text));
        set(
            &dataset(&next),
            "nextAction",
            &JsValue::from_str(if network_enabled && !settings_invalid {
                "start"
            } else {
                "settings"
            }),
        );
    }

    if present(&start) {
        let start_blocked = running
            || transition == "starting"
            || transition == "stopping"
            || !network_enabled
            || settings_invalid;
        set(&start, "disabled", &JsValue::from_bool(start_blocked));
        set_attribute(
            &start,
            "aria-disabled",
            if start_blocked { "true" } else { "false" },
        );
        set_style(&start, "opacity", if start_blocked { "0.45" } else { "" });
        set_style(
            &start,
            "cursor",
            if start_blocked { "not-allowed" } else { "" },
        );
        let title = if !network_enabled {
            "Enable this network before starting it."
        } else if running {
            "Bridge is running. Stop it before starting again."
        } else {
            "Start bridge"
        };
        set(&start, "title", &JsValue::from_str(title));
    }

    if present(&stop) {
        let stop_enabled = running && transition != "starting" && transition != "stopping";
        set(&stop, "disabled", &JsValue::from_bool(!stop_enabled));
        set_attribute(
            &stop,
            "aria-disabled",
            if stop_enabled { "false" } else { "true" },
        );
        set_style(&stop, "opacity", if stop_enabled { "" } else { "0.45" });
        set_style(
            &stop,
            "cursor",
            if stop_enabled { "" } else { "not-allowed" },
        );
        let title = if transition == "starting" {
            "Bridge startup is in progress. Stop becomes available after READY."
        } else if stop_enabled {
            "Stop bridge"
        } else {
            "Bridge is not running"
        };
        set(&stop, "title", &JsValue::from_str(title));
    }
}

#[wasm_bindgen(js_name = bridgeMarkRestartRequiredV1)]
pub fn bridge_mark_restart_required_v1(net: String) -> bool {
    mark_restart_required_inner(&net)
}

#[wasm_bindgen(js_name = bridgeSetRuntimeErrorV1)]
pub fn bridge_set_runtime_error_v1(
    net: String,
    error_text: JsValue,
    error_source: JsValue,
) -> bool {
    set_runtime_error_inner(&net, &error_text, &error_source)
}

#[wasm_bindgen(js_name = bridgeSetRuntimeActivityV1)]
pub fn bridge_set_runtime_activity_v1(net: String, message: JsValue, state: JsValue) -> bool {
    set_runtime_activity_inner(&net, &message, &state)
}

#[wasm_bindgen(js_name = bridgeR51SetRuntimeUnknown)]
pub fn bridge_r51_set_runtime_unknown(net: String, message: String, error_source: String) {
    let panel = crate::bridge_instance_settings::bridge_r51_panel(net.clone());
    if !present(&panel) {
        return;
    }

    let policy_status = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(net.clone(), "policyStatus".to_owned()),
    );
    if present(&policy_status) {
        set(
            &policy_status,
            "textContent",
            &JsValue::from_str("Reconciling"),
        );
        set(
            &dataset(&policy_status),
            "state",
            &JsValue::from_str("reconciling"),
        );
        let _ = crate::apply_status_tone(policy_status, JsValue::from_str("reconciling"));
        let summary = crate::bridge_frontend_helpers::bridge_by_id(
            crate::bridge_frontend_helpers::bridge_element_id(
                net.clone(),
                "monitorState".to_owned(),
            ),
        );
        let _ = crate::render_status_summary(
            summary,
            JsValue::from_str("Bridge: Reconciling | RPC/synchronization/mining: unknown"),
        );
    }

    for button in [
        query(
            &panel,
            &format!(r#"[data-bridge-action="start"][data-net="{net}"]"#),
        ),
        query(
            &panel,
            &format!(r#"[data-bridge-action="stop"][data-net="{net}"]"#),
        ),
    ] {
        if !present(&button) {
            continue;
        }
        set(&button, "disabled", &JsValue::TRUE);
        set_attribute(&button, "aria-disabled", "true");
        set_style(&button, "opacity", "0.45");
        set_style(&button, "cursor", "not-allowed");
        set(&button, "title", &JsValue::from_str(&message));
    }

    r51_set_runtime_activity(&net, "Reconciling runtime state.", "");

    let current_error = crate::bridge_frontend_helpers::bridge_by_id(
        crate::bridge_frontend_helpers::bridge_element_id(net.clone(), "runtimeError".to_owned()),
    );
    let preserve_action_error = error_source == "status-refresh"
        && !text(&property(&current_error, "textContent"))
            .trim()
            .is_empty()
        && text(&property(&dataset(&current_error), "runtimeErrorSource")) != "status-refresh";
    if !preserve_action_error {
        r51_set_runtime_error(&net, &message, &error_source);
    }
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
