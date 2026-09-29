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
}
