use js_sys::{Array, Function, JSON, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

const COMMAND: &str = "kgw_start_trace_frontend_v1";
const REDACTED: &str = "[redacted]";
const BLOCKED_KEYS: &[&str] = &[
    "secret",
    "token",
    "private",
    "mnemonic",
    "wallet",
    "address",
    "commandpreview",
    "completecommand",
    "arguments",
    "appdir",
    "path",
    "rpcendpoint",
    "stratum",
];

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

fn document() -> JsValue {
    property(&global(), "document")
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn closest(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "closest", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn set_attr(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn collapse_control_whitespace(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut control_run = false;
    for ch in value.chars() {
        if matches!(ch, '\r' | '\n' | '\t') {
            if !control_run {
                output.push(' ');
                control_run = true;
            }
        } else {
            output.push(ch);
            control_run = false;
        }
    }
    output
}

fn safe_text_str(value: &str, fallback: &str) -> String {
    let mut text = collapse_control_whitespace(value).trim().to_owned();
    if text.is_empty() {
        text = fallback.to_owned();
    }
    text.chars().take(220).collect()
}

fn safe_text(value: &JsValue, fallback: &str) -> String {
    safe_text_str(&crate::js_string_owned(value), fallback)
}

fn blocked_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    BLOCKED_KEYS.iter().any(|needle| lower.contains(needle))
}
fn safe_details(value: &JsValue) -> JsValue {
    let source = if value.is_object() && !Array::is_array(value) {
        value.clone()
    } else {
        Object::new().into()
    };
    let output = Object::new();
    for entry in Object::entries(&Object::from(source)).iter() {
        let pair = Array::from(&entry);
        if pair.length() < 2 {
            continue;
        }
        let key = crate::js_string_owned(&pair.get(0));
        let value = pair.get(1);
        let sanitized = if blocked_key(&key) {
            JsValue::from_str(REDACTED)
        } else if Array::is_array(&value) {
            let source = Array::from(&value);
            let result = Array::new();
            for item in source.iter().take(24) {
                result.push(&JsValue::from_str(&safe_text(&item, "")));
            }
            result.into()
        } else if value.is_object() && !value.is_null() {
            safe_details(&value)
        } else if value.as_bool().is_some() || value.as_f64().is_some() {
            value
        } else {
            JsValue::from_str(&safe_text(&value, ""))
        };
        set(output.as_ref(), &key, &sanitized);
    }
    output.into()
}

fn clipboard_character_count_text(text: &str) -> usize {
    text.chars().count()
}

fn clipboard_line_count_text(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.split('\n').count()
    }
}

fn normalize_clipboard_line_endings_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

fn clipboard_safe_error_text(text: &str) -> String {
    let normalized = collapse_control_whitespace(text).trim().to_owned();
    let source = if normalized.is_empty() {
        "clipboard write failed"
    } else {
        normalized.as_str()
    };
    let lower = source.to_ascii_lowercase();
    if [
        "secret", "token", "private", "mnemonic", "wallet", "address",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return "clipboard write failed with a sensitive error".to_owned();
    }
    source.chars().take(360).collect()
}

async fn sha256_hex_text(text: &str) -> String {
    let crypto = {
        let candidate = property(&window(), "crypto");
        if present(&candidate) {
            candidate
        } else {
            property(&global(), "crypto")
        }
    };
    let subtle = property(&crypto, "subtle");
    let Ok(digest) = property(&subtle, "digest").dyn_into::<Function>() else {
        return String::new();
    };
    let bytes = Uint8Array::from(text.as_bytes());
    let Ok(result) = digest.call2(&subtle, &JsValue::from_str("SHA-256"), bytes.as_ref()) else {
        return String::new();
    };
    let Ok(resolved) = JsFuture::from(Promise::resolve(&result)).await else {
        return String::new();
    };
    let mut output = String::new();
    for byte in Uint8Array::new(&resolved).to_vec() {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn bridge_log_output_id(net: &str) -> String {
    format!("bridge-{net}-logOutput")
}

fn bridge_status_selector(net: &str) -> String {
    format!(".kgw-copy-log-status-v1[data-net=\"{net}\"]")
}

fn bridge_log_output(net: &str) -> JsValue {
    call1(
        &document(),
        "getElementById",
        &JsValue::from_str(&bridge_log_output_id(net)),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

fn clipboard_status_element(net: &str) -> JsValue {
    let out = bridge_log_output(net);
    if !present(&out) {
        return JsValue::UNDEFINED;
    }
    let panel = closest(&out, "[data-bridge-inner-panel=\"log\"]");
    if !present(&panel) {
        return JsValue::UNDEFINED;
    }
    let toolbar = query(&panel, ".bridge-v7-log-toolbar");
    if !present(&toolbar) {
        return JsValue::UNDEFINED;
    }
    let existing = query(&toolbar, &bridge_status_selector(net));
    if present(&existing) {
        return existing;
    }
    let status = call1(&document(), "createElement", &JsValue::from_str("span"))
        .unwrap_or(JsValue::UNDEFINED);
    if !present(&status) {
        return JsValue::UNDEFINED;
    }
    set_attr(&status, "class", "kgw-copy-log-status-v1");
    set(
        &property(&status, "dataset"),
        "net",
        &JsValue::from_str(net),
    );
    set_attr(&status, "data-net", net);
    set_attr(&status, "role", "status");
    set_attr(&status, "aria-live", "polite");
    let _ = call1(&toolbar, "appendChild", &status);
    status
}

fn set_clipboard_status(net: &str, message: &str, state: &str) -> bool {
    let status = clipboard_status_element(net);
    if !present(&status) {
        return false;
    }
    set(&status, "textContent", &JsValue::from_str(message));
    set(
        &property(&status, "dataset"),
        "state",
        &JsValue::from_str(state),
    );
    let _ = crate::apply_status_tone_js(status.clone(), JsValue::from_str(state));
    set(&status, "hidden", &JsValue::from_bool(message.is_empty()));
    true
}

fn read_clipboard_raw_log_buffer(net: &str) -> JsValue {
    let out = bridge_log_output(net);
    let tag = crate::js_string_owned(&property(&out, "tagName")).to_uppercase();
    let raw_text = if present(&out) {
        if tag == "TEXTAREA" || tag == "INPUT" {
            crate::js_string_owned(&property(&out, "value"))
        } else {
            crate::js_string_owned(&property(&out, "textContent"))
        }
    } else {
        String::new()
    };
    let normalized = normalize_clipboard_line_endings_text(&raw_text);
    let output = Object::new();
    set(output.as_ref(), "out", &out);
    set(output.as_ref(), "rawText", &JsValue::from_str(&raw_text));
    set(
        output.as_ref(),
        "normalizedText",
        &JsValue::from_str(&normalized),
    );
    set(
        output.as_ref(),
        "characterCount",
        &JsValue::from_f64(clipboard_character_count_text(&normalized) as f64),
    );
    set(
        output.as_ref(),
        "lineCount",
        &JsValue::from_f64(clipboard_line_count_text(&normalized) as f64),
    );
    output.into()
}

fn resolve_invoke() -> Option<Function> {
    let window = window();
    let tauri = property(&window, "__TAURI__");
    let core = property(&tauri, "core");
    for candidate in [
        property(&core, "invoke"),
        property(&tauri, "invoke"),
        property(&window, "__TAURI_INVOKE__"),
    ] {
        if let Ok(function) = candidate.dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}

fn metadata_text(metadata: &JsValue, name: &str, fallback: &str) -> String {
    let value = property(metadata, name);
    if crate::js_boolean(&value) {
        crate::js_string_owned(&value)
    } else {
        fallback.to_owned()
    }
}

fn metadata_number(metadata: &JsValue, name: &str) -> f64 {
    let value = property(metadata, name);
    let number = crate::js_number(&value);
    if number.is_finite() { number } else { 0.0 }
}

fn emit_trace(stage: &str, net: &str, action: &str, result: &str, details: JsValue) {
    let options = Object::new();
    set(options.as_ref(), "network", &JsValue::from_str(net));
    set(options.as_ref(), "action", &JsValue::from_str(action));
    set(options.as_ref(), "result", &JsValue::from_str(result));
    set(options.as_ref(), "details", &details);
    let _ = bridge_start_trace_frontend(JsValue::from_str(stage), options.into());
}

async fn dispatch_clipboard_write_impl(
    net: &str,
    text: &str,
    metadata: &JsValue,
) -> Result<JsValue, JsValue> {
    let Some(invoke) = resolve_invoke() else {
        return Err(JsValue::from_str(
            "Tauri invoke API is not available for Copy Log.",
        ));
    };
    let runtime_role = metadata_text(metadata, "runtimeRole", "bridge");
    let bridge_instance_id = metadata_text(metadata, "bridgeInstanceId", "");
    let character_count = metadata_number(metadata, "characterCount");
    let line_count = metadata_number(metadata, "lineCount");
    let sha256 = metadata_text(metadata, "sha256", "");

    let details = Object::new();
    for (key, value) in [
        (
            "commandName",
            JsValue::from_str("kgw_copy_text_to_clipboard_v1"),
        ),
        ("implementation", JsValue::from_str("native-tauri-command")),
        ("runtimeRole", JsValue::from_str(&runtime_role)),
        ("bridgeInstanceId", JsValue::from_str(&bridge_instance_id)),
        ("characterCount", JsValue::from_f64(character_count)),
        ("lineCount", JsValue::from_f64(line_count)),
        ("sha256", JsValue::from_str(&sha256)),
        ("payloadFieldCount", JsValue::from_f64(7.0)),
    ] {
        set(details.as_ref(), key, &value);
    }
    emit_trace(
        "frontend.copy_log_dispatched",
        net,
        "copy-log",
        "dispatched",
        details.into(),
    );

    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(net));
    set(
        payload.as_ref(),
        "runtimeRole",
        &JsValue::from_str(&runtime_role),
    );
    set(
        payload.as_ref(),
        "bridgeInstanceId",
        &JsValue::from_str(&bridge_instance_id),
    );
    set(payload.as_ref(), "text", &JsValue::from_str(text));
    set(
        payload.as_ref(),
        "characterCount",
        &JsValue::from_f64(character_count),
    );
    set(
        payload.as_ref(),
        "lineCount",
        &JsValue::from_f64(line_count),
    );
    set(payload.as_ref(), "sha256", &JsValue::from_str(&sha256));

    let call = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_copy_text_to_clipboard_v1"),
        payload.as_ref(),
    )?;
    crate::node_start_trace::await_command_with_timeout(
        call,
        "kgw_copy_text_to_clipboard_v1",
        110_000,
    )
    .await
}

fn original_log_action_label_text(value: &str) -> String {
    let text = value.trim();
    if text.is_empty() {
        "Log Action".to_owned()
    } else {
        text.to_owned()
    }
}

fn restore_log_action_label_impl(button: &JsValue) {
    if !present(button) {
        return;
    }
    let data = property(button, "dataset");
    let original = property(&data, "kgwLogOriginalLabelV29");
    if crate::js_boolean(&original) {
        set(
            button,
            "textContent",
            &JsValue::from_str(&crate::js_string_owned(&original)),
        );
    }
    let classes = property(button, "classList");
    let _ = call1(
        &classes,
        "remove",
        &JsValue::from_str("kgw-log-action-feedback"),
    );
    if let Ok(object) = data.dyn_into::<Object>() {
        let _ = Reflect::delete_property(&object, &JsValue::from_str("kgwDoneLabel"));
    }
}

fn flash_log_action_button_impl(button: &JsValue, done_label: &str) {
    if !present(button) {
        return;
    }
    let data = property(button, "dataset");
    let original = property(&data, "kgwLogOriginalLabelV29");
    if !crate::js_boolean(&original) {
        let current = crate::js_string_owned(&property(button, "textContent"));
        set(
            &data,
            "kgwLogOriginalLabelV29",
            &JsValue::from_str(&original_log_action_label_text(&current)),
        );
    }

    let browser = window();
    let timer = property(button, "__kgwLogActionFeedbackTimerV29");
    if present(&timer) {
        let _ = call1(&browser, "clearTimeout", &timer);
    }

    set(button, "textContent", &JsValue::from_str(done_label));
    set(&data, "kgwDoneLabel", &JsValue::from_str(done_label));
    let classes = property(button, "classList");
    let _ = call1(
        &classes,
        "add",
        &JsValue::from_str("kgw-log-action-feedback"),
    );

    let button_for_timeout = button.clone();
    let callback = Closure::wrap(Box::new(move || {
        restore_log_action_label_impl(&button_for_timeout);
    }) as Box<dyn FnMut()>);
    if let Some(timer_id) = call2(
        &browser,
        "setTimeout",
        callback.as_ref(),
        &JsValue::from_f64(1600.0),
    ) {
        set(button, "__kgwLogActionFeedbackTimerV29", &timer_id);
    }
    callback.forget();
}

fn dependency(deps: &JsValue, name: &str) -> Option<Function> {
    property(deps, name).dyn_into::<Function>().ok()
}

fn dependency_apply(deps: &JsValue, name: &str, args: &[JsValue]) -> JsValue {
    let Some(function) = dependency(deps, name) else {
        return JsValue::UNDEFINED;
    };
    let values = Array::new();
    for value in args {
        values.push(value);
    }
    function
        .apply(&JsValue::UNDEFINED, &values)
        .unwrap_or(JsValue::UNDEFINED)
}

fn translate_runtime(deps: &JsValue, key: &str, fallback: &str) -> String {
    let value = dependency_apply(
        deps,
        "translateRuntime",
        &[JsValue::from_str(key), JsValue::from_str(fallback)],
    );
    let text = crate::js_string_owned(&value);
    if text.is_empty() {
        fallback.to_owned()
    } else {
        text
    }
}

fn active_raw_log_instance_id(deps: &JsValue, net: &str) -> String {
    crate::js_string_owned(&dependency_apply(
        deps,
        "activeRawLogInstanceId",
        &[JsValue::from_str(net)],
    ))
}

fn small_owner_trace(deps: &JsValue, net: &str, action: &str, phase: &str, details: JsValue) {
    let _ = dependency_apply(
        deps,
        "smallOwnerTrace",
        &[
            JsValue::from_str(net),
            JsValue::from_str(action),
            JsValue::from_str(phase),
            details,
        ],
    );
}

fn clear_raw_log_buffer(deps: &JsValue, net: &str, instance_id: &str) {
    let _ = dependency_apply(
        deps,
        "clearRawLogBuffer",
        &[
            JsValue::from_str(net),
            JsValue::from_str("bridge"),
            JsValue::from_str(instance_id),
        ],
    );
}

fn dispatch_runtime_log_clear(deps: &JsValue, net: &str) {
    let result = dependency_apply(
        deps,
        "dispatchRuntimeLogClear",
        &[JsValue::from_str(net), JsValue::from_str("bridge")],
    );
    if !present(&result) {
        return;
    }
    let promise = Promise::resolve(&result);
    let catch = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
    let _ = promise.catch(&catch);
    catch.forget();
}

fn delete_dataset_key(target: &JsValue, key: &str) {
    let data = property(target, "dataset");
    if let Ok(object) = data.dyn_into::<Object>() {
        let _ = Reflect::delete_property(&object, &JsValue::from_str(key));
    }
}

fn clipboard_safe_error_value(error: &JsValue) -> String {
    let message = property(error, "message");
    let source = if crate::js_boolean(&message) {
        crate::js_string_owned(&message)
    } else if crate::js_boolean(error) {
        crate::js_string_owned(error)
    } else {
        "clipboard write failed".to_owned()
    };
    clipboard_safe_error_text(&source)
}

fn copy_log_failure_impl(
    net: &str,
    button: &JsValue,
    error: &JsValue,
    details: &JsValue,
    deps: &JsValue,
) {
    let safe_error = clipboard_safe_error_value(error);
    let _ = set_clipboard_status(net, &safe_error, "error");
    let failed_label = translate_runtime(deps, "log.copyFailed", "Copy failed");
    flash_log_action_button_impl(button, &failed_label);

    let trace_details = safe_details(details);
    set(&trace_details, "runtimeRole", &JsValue::from_str("bridge"));
    set(&trace_details, "safeError", &JsValue::from_str(&safe_error));
    set(
        &trace_details,
        "userFeedbackDisplayed",
        &JsValue::from_bool(true),
    );
    emit_trace(
        "frontend.copy_log_failed",
        net,
        "copy-log",
        "error",
        trace_details,
    );
}

async fn handle_log_action_impl(
    action: &str,
    net: &str,
    button: &JsValue,
    deps: &JsValue,
) -> Result<(), JsValue> {
    let trace_action = if action.is_empty() {
        "log-action"
    } else {
        action
    };
    let click_details = Object::new();
    set(
        click_details.as_ref(),
        "patch",
        &JsValue::from_str("KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3"),
    );
    set(click_details.as_ref(), "action", &JsValue::from_str(action));
    set(
        click_details.as_ref(),
        "buttonId",
        &JsValue::from_str(&crate::js_string_owned(&property(button, "id"))),
    );
    set(
        click_details.as_ref(),
        "buttonText",
        &JsValue::from_str(crate::js_string_owned(&property(button, "textContent")).trim()),
    );
    small_owner_trace(
        deps,
        net,
        trace_action,
        "r51b3-bridge-log-action-click",
        click_details.into(),
    );
    small_owner_trace(
        deps,
        net,
        trace_action,
        "r44d-owner-begin",
        Object::new().into(),
    );

    if !present(&bridge_log_output(net)) {
        return Ok(());
    }

    if action == "copy-log" {
        let instance_id = active_raw_log_instance_id(deps, net);
        let belongs_to_live_bridge_monitor =
            present(&closest(button, r#"[data-bridge-inner-panel="log"]"#));

        let resolved_details = Object::new();
        set(
            resolved_details.as_ref(),
            "runtimeRole",
            &JsValue::from_str("bridge"),
        );
        set(
            resolved_details.as_ref(),
            "bridgeInstanceId",
            &JsValue::from_str(&instance_id),
        );
        set(
            resolved_details.as_ref(),
            "belongsToLiveBridgeMonitor",
            &JsValue::from_bool(belongs_to_live_bridge_monitor),
        );
        emit_trace(
            "frontend.copy_log_network_resolved",
            net,
            "copy-log",
            if net.is_empty() { "error" } else { "ok" },
            resolved_details.into(),
        );

        let data = property(button, "dataset");
        if crate::js_string_owned(&property(&data, "kgwCopyLogInFlightV1")) == "1" {
            let duplicate_details = Object::new();
            set(
                duplicate_details.as_ref(),
                "reason",
                &JsValue::from_str("duplicate-copy"),
            );
            set(
                duplicate_details.as_ref(),
                "bridgeInstanceId",
                &JsValue::from_str(&instance_id),
            );
            set(
                duplicate_details.as_ref(),
                "belongsToLiveBridgeMonitor",
                &JsValue::from_bool(belongs_to_live_bridge_monitor),
            );
            copy_log_failure_impl(
                net,
                button,
                &JsValue::from_str("Copy Log is already in progress for this bridge buffer."),
                duplicate_details.as_ref(),
                deps,
            );
            return Ok(());
        }

        let original_disabled = crate::js_boolean(&property(button, "disabled"));
        if present(button) {
            set(
                &property(button, "dataset"),
                "kgwCopyLogInFlightV1",
                &JsValue::from_str("1"),
            );
            set(button, "disabled", &JsValue::from_bool(true));
        }

        let copy_result: Result<(), JsValue> = async {
            let buffer = read_clipboard_raw_log_buffer(net);
            let buffer_out = property(&buffer, "out");
            let normalized_text = crate::js_string_owned(&property(&buffer, "normalizedText"));
            let character_count = property(&buffer, "characterCount");
            let line_count = property(&buffer, "lineCount");
            if !present(&buffer_out) || normalized_text.trim().is_empty() {
                let prepared = Object::new();
                set(
                    prepared.as_ref(),
                    "rawLogBufferSelected",
                    &JsValue::from_bool(present(&buffer_out)),
                );
                set(
                    prepared.as_ref(),
                    "runtimeRole",
                    &JsValue::from_str("bridge"),
                );
                set(
                    prepared.as_ref(),
                    "bridgeInstanceId",
                    &JsValue::from_str(&instance_id),
                );
                set(prepared.as_ref(), "characterCount", &character_count);
                set(prepared.as_ref(), "lineCount", &line_count);
                set(prepared.as_ref(), "sha256", &JsValue::from_str(""));
                emit_trace(
                    "frontend.copy_log_content_prepared",
                    net,
                    "copy-log",
                    "error",
                    prepared.into(),
                );
                return Err(JsValue::from_str(&format!(
                    "Copy Log requires a non-empty raw log buffer for {net}."
                )));
            }

            let sha256 = sha256_hex_text(&normalized_text).await;
            let metadata = Object::new();
            set(
                metadata.as_ref(),
                "runtimeRole",
                &JsValue::from_str("bridge"),
            );
            set(
                metadata.as_ref(),
                "bridgeInstanceId",
                &JsValue::from_str(&instance_id),
            );
            set(metadata.as_ref(), "characterCount", &character_count);
            set(metadata.as_ref(), "lineCount", &line_count);
            set(metadata.as_ref(), "sha256", &JsValue::from_str(&sha256));

            let prepared = Object::new();
            set(
                prepared.as_ref(),
                "rawLogBufferSelected",
                &JsValue::from_bool(true),
            );
            for name in [
                "runtimeRole",
                "bridgeInstanceId",
                "characterCount",
                "lineCount",
                "sha256",
            ] {
                set(prepared.as_ref(), name, &property(metadata.as_ref(), name));
            }
            emit_trace(
                "frontend.copy_log_content_prepared",
                net,
                "copy-log",
                "ok",
                prepared.into(),
            );

            dispatch_clipboard_write_impl(net, &normalized_text, metadata.as_ref()).await?;
            let copied_label = translate_runtime(deps, "log.copied", "Copied");
            flash_log_action_button_impl(button, &copied_label);
            let _ = set_clipboard_status(net, &copied_label, "ok");

            let success = Object::new();
            for name in [
                "runtimeRole",
                "bridgeInstanceId",
                "characterCount",
                "lineCount",
                "sha256",
            ] {
                set(success.as_ref(), name, &property(metadata.as_ref(), name));
            }
            set(
                success.as_ref(),
                "userFeedbackDisplayed",
                &JsValue::from_bool(true),
            );
            emit_trace(
                "frontend.copy_log_succeeded",
                net,
                "copy-log",
                "ok",
                success.into(),
            );
            Ok(())
        }
        .await;

        if let Err(error) = copy_result {
            let failure_details = Object::new();
            set(
                failure_details.as_ref(),
                "bridgeInstanceId",
                &JsValue::from_str(&instance_id),
            );
            set(
                failure_details.as_ref(),
                "belongsToLiveBridgeMonitor",
                &JsValue::from_bool(belongs_to_live_bridge_monitor),
            );
            copy_log_failure_impl(net, button, &error, failure_details.as_ref(), deps);
        }

        if present(button) {
            set(button, "disabled", &JsValue::from_bool(original_disabled));
            delete_dataset_key(button, "kgwCopyLogInFlightV1");
        }
        return Ok(());
    }

    if action == "clear-log" {
        let instance_id = active_raw_log_instance_id(deps, net);
        clear_raw_log_buffer(deps, net, &instance_id);
        dispatch_runtime_log_clear(deps, net);
        let deleted_label = translate_runtime(deps, "log.deleted", "Deleted");
        flash_log_action_button_impl(button, &deleted_label);
    }
    small_owner_trace(
        deps,
        net,
        trace_action,
        "r44d-owner-complete",
        Object::new().into(),
    );
    Ok(())
}

#[wasm_bindgen(js_name = bridgeHandleLogAction)]
pub async fn bridge_handle_log_action(
    action: String,
    net: String,
    button: JsValue,
    deps: JsValue,
) -> Result<(), JsValue> {
    handle_log_action_impl(&action, &net, &button, &deps).await
}

#[wasm_bindgen(js_name = bridgeRestoreLogActionLabel)]
pub fn bridge_restore_log_action_label(button: JsValue) {
    restore_log_action_label_impl(&button);
}

#[wasm_bindgen(js_name = bridgeFlashLogActionButton)]
pub fn bridge_flash_log_action_button(button: JsValue, done_label: String) {
    flash_log_action_button_impl(&button, &done_label);
}

#[wasm_bindgen(js_name = bridgeDispatchClipboardWrite)]
pub async fn bridge_dispatch_clipboard_write(
    net: String,
    text: String,
    metadata: JsValue,
) -> Result<JsValue, JsValue> {
    dispatch_clipboard_write_impl(&net, &text, &metadata).await
}

#[wasm_bindgen(js_name = bridgeClipboardStatusElement)]
pub fn bridge_clipboard_status_element(net: String) -> JsValue {
    clipboard_status_element(&net)
}

#[wasm_bindgen(js_name = bridgeSetClipboardStatus)]
pub fn bridge_set_clipboard_status(net: String, message: String, state: String) -> bool {
    set_clipboard_status(&net, &message, &state)
}

#[wasm_bindgen(js_name = bridgeReadClipboardRawLogBuffer)]
pub fn bridge_read_clipboard_raw_log_buffer(net: String) -> JsValue {
    read_clipboard_raw_log_buffer(&net)
}

#[wasm_bindgen(js_name = bridgeClipboardCharacterCount)]
pub fn bridge_clipboard_character_count(text: JsValue) -> u32 {
    clipboard_character_count_text(&crate::js_string_owned(&text)) as u32
}

#[wasm_bindgen(js_name = bridgeClipboardLineCount)]
pub fn bridge_clipboard_line_count(text: JsValue) -> u32 {
    clipboard_line_count_text(&crate::js_string_owned(&text)) as u32
}

#[wasm_bindgen(js_name = bridgeNormalizeClipboardLineEndings)]
pub fn bridge_normalize_clipboard_line_endings(text: JsValue) -> String {
    normalize_clipboard_line_endings_text(&crate::js_string_owned(&text))
}

#[wasm_bindgen(js_name = bridgeClipboardSafeError)]
pub fn bridge_clipboard_safe_error(error: JsValue) -> String {
    let message = property(&error, "message");
    let source = if crate::js_boolean(&message) {
        crate::js_string_owned(&message)
    } else if crate::js_boolean(&error) {
        crate::js_string_owned(&error)
    } else {
        "clipboard write failed".to_owned()
    };
    clipboard_safe_error_text(&source)
}

#[wasm_bindgen(js_name = bridgeSha256Hex)]
pub async fn bridge_sha256_hex(text: JsValue) -> String {
    sha256_hex_text(&crate::js_string_owned(&text)).await
}

#[wasm_bindgen(js_name = bridgeStartTraceFrontend)]
pub fn bridge_start_trace_frontend(stage: JsValue, options: JsValue) -> bool {
    let Some(invoke) = resolve_invoke() else {
        return false;
    };
    let network_source = {
        let network = property(&options, "network");
        if crate::js_boolean(&network) {
            network
        } else {
            property(&options, "net")
        }
    };
    let details_source = property(&options, "details");
    let details = safe_details(&if details_source.is_object() {
        details_source
    } else {
        Object::new().into()
    });
    let args = Object::new();
    set(
        args.as_ref(),
        "stage",
        &JsValue::from_str(&safe_text(&stage, "frontend.unknown")),
    );
    set(
        args.as_ref(),
        "network",
        &JsValue::from_str(&safe_text(&network_source, "unknown")),
    );
    set(
        args.as_ref(),
        "action",
        &JsValue::from_str(&safe_text(&property(&options, "action"), "unknown")),
    );
    set(
        args.as_ref(),
        "result",
        &JsValue::from_str(&safe_text(&property(&options, "result"), "observed")),
    );
    let serialized = JSON::stringify(&details)
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    set(args.as_ref(), "details", &JsValue::from_str(&serialized));
    let call = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str(COMMAND),
        args.as_ref(),
    );
    if let Ok(result) = call {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |error: JsValue| {
            let console = property(&global(), "console");
            if let Ok(log) = property(&console, "error").dyn_into::<Function>() {
                let candidate = property(&error, "message");
                let message = if crate::js_boolean(&candidate) {
                    crate::js_string_owned(&candidate)
                } else {
                    crate::js_string_owned(&error)
                };
                let _ = log.call2(
                    &console,
                    &JsValue::from_str("[KGW_START_TRACE_FRONTEND_FAILED]"),
                    &JsValue::from_str(&message),
                );
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }
    true
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_key_policy_matches_bridge_legacy() {
        for key in [
            "apiToken",
            "walletAddress",
            "commandPreview",
            "completeCommand",
            "rpcEndpoint",
            "stratumPort",
            "appDir",
        ] {
            assert!(blocked_key(key), "{key}");
        }
        assert!(!blocked_key("network"));
        assert!(!blocked_key("result"));
    }

    #[test]
    fn safe_text_matches_bridge_legacy_shape() {
        assert_eq!(safe_text_str(" a\tb\nc ", ""), "a b c");
        assert_eq!(safe_text_str("", "fallback"), "fallback");
        let long = "x".repeat(300);
        assert_eq!(safe_text_str(&long, "").len(), 220);
    }
    #[test]
    fn blocked_key_matching_is_case_insensitive_and_substring_based() {
        assert!(blocked_key("PrivateKey"));
        assert!(blocked_key("someWalletMetadata"));
        assert!(blocked_key("ArgumentsPreview"));
        assert!(!blocked_key("bridgeInstanceId"));
    }

    #[test]
    fn clipboard_counts_and_line_endings_match_legacy() {
        assert_eq!(clipboard_character_count_text("A😀B"), 3);
        assert_eq!(clipboard_line_count_text(""), 0);
        assert_eq!(clipboard_line_count_text("a"), 1);
        assert_eq!(clipboard_line_count_text("a\nb\n"), 3);
        assert_eq!(
            normalize_clipboard_line_endings_text("a\r\nb\rc\nd"),
            "a\r\nb\r\nc\r\nd"
        );
    }

    #[test]
    fn clipboard_safe_error_matches_bridge_legacy() {
        assert_eq!(
            clipboard_safe_error_text(" token\r\n\t leaked "),
            "clipboard write failed with a sensitive error"
        );
        assert_eq!(
            clipboard_safe_error_text(" ordinary\r\n\t failure "),
            "ordinary  failure"
        );
        assert_eq!(clipboard_safe_error_text(""), "clipboard write failed");
    }

    #[test]
    fn control_whitespace_runs_collapse_like_legacy_regex() {
        assert_eq!(safe_text_str("a\r\n\tb", ""), "a b");
        assert_eq!(clipboard_safe_error_text("a\r\n\tb"), "a b");
    }

    #[test]
    fn bridge_clipboard_dom_identifiers_match_legacy() {
        assert_eq!(bridge_log_output_id("mainnet"), "bridge-mainnet-logOutput");
        assert_eq!(
            bridge_status_selector("testnet10"),
            ".kgw-copy-log-status-v1[data-net=\"testnet10\"]"
        );
    }

    #[test]
    fn log_feedback_original_label_matches_legacy_fallback() {
        assert_eq!(original_log_action_label_text(" Copy Log "), "Copy Log");
        assert_eq!(original_log_action_label_text(""), "Log Action");
        assert_eq!(original_log_action_label_text("   "), "Log Action");
    }
}
