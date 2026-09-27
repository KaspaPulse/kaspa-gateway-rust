use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

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

fn safe_text_str(value: &str, fallback: &str) -> String {
    let mut text = value.replace(['\r', '\n', '\t'], " ").trim().to_owned();
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

fn object_keys(value: &JsValue) -> JsValue {
    let output = Array::new();
    if !value.is_object() || value.is_null() {
        return output.into();
    }
    let mut keys = Object::keys(&Object::from(value.clone()))
        .iter()
        .map(|value| crate::js_string_owned(&value))
        .collect::<Vec<_>>();
    keys.sort();
    keys.truncate(24);
    for key in keys {
        output.push(&JsValue::from_str(&key));
    }
    output.into()
}

fn tauri_shape(adapter: &str) -> JsValue {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let legacy = property(&tauri, "tauri");
    let output = Object::new();
    set(
        output.as_ref(),
        "adapter",
        &JsValue::from_str(&safe_text(&JsValue::from_str(adapter), "missing")),
    );
    set(
        output.as_ref(),
        "hasGlobalTauri",
        &JsValue::from_bool(present(&tauri)),
    );
    set(output.as_ref(), "globalKeys", &object_keys(&tauri));
    set(output.as_ref(), "coreKeys", &object_keys(&core));
    set(output.as_ref(), "tauriKeys", &object_keys(&legacy));
    set(
        output.as_ref(),
        "hasCoreInvoke",
        &JsValue::from_bool(property(&core, "invoke").dyn_ref::<Function>().is_some()),
    );
    set(
        output.as_ref(),
        "hasTauriInvoke",
        &JsValue::from_bool(property(&legacy, "invoke").dyn_ref::<Function>().is_some()),
    );
    set(
        output.as_ref(),
        "hasRootInvoke",
        &JsValue::from_bool(property(&tauri, "invoke").dyn_ref::<Function>().is_some()),
    );
    set(
        output.as_ref(),
        "expectedConfiguredGlobal",
        &JsValue::from_str("window.__TAURI__.core.invoke"),
    );
    output.into()
}
fn bound_invoke(owner: &JsValue, invoke: JsValue) -> Option<Function> {
    let invoke = invoke.dyn_into::<Function>().ok()?;
    let bind = function(invoke.as_ref(), "bind")?;
    bind.call1(invoke.as_ref(), owner)
        .ok()?
        .dyn_into::<Function>()
        .ok()
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
    let normalized = text.replace(['\r', '\n', '\t'], " ").trim().to_owned();
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

fn clipboard_placeholder_text(net: &str) -> String {
    let label = match net {
        "mainnet" => "Mainnet",
        "testnet10" => "Testnet 10",
        "testnet13" => "Testnet 13",
        other => other,
    };
    format!("{label} log is empty.")
}

fn resolved_invoke() -> JsValue {
    let tauri = property(&window(), "__TAURI__");
    for (adapter, owner, candidate) in [
        (
            "window.__TAURI__.core.invoke",
            property(&tauri, "core"),
            property(&property(&tauri, "core"), "invoke"),
        ),
        (
            "window.__TAURI__.tauri.invoke",
            property(&tauri, "tauri"),
            property(&property(&tauri, "tauri"), "invoke"),
        ),
        (
            "window.__TAURI__.invoke",
            tauri.clone(),
            property(&tauri, "invoke"),
        ),
    ] {
        if let Some(invoke) = bound_invoke(&owner, candidate) {
            let output = Object::new();
            set(output.as_ref(), "adapter", &JsValue::from_str(adapter));
            set(output.as_ref(), "invoke", invoke.as_ref());
            set(output.as_ref(), "shape", &tauri_shape(adapter));
            return output.into();
        }
    }
    let output = Object::new();
    set(output.as_ref(), "adapter", &JsValue::from_str("missing"));
    set(output.as_ref(), "invoke", &JsValue::NULL);
    set(output.as_ref(), "shape", &tauri_shape("missing"));
    output.into()
}
#[wasm_bindgen(js_name = nodeClipboardCharacterCount)]
pub fn node_clipboard_character_count(text: JsValue) -> u32 {
    clipboard_character_count_text(&crate::js_string_owned(&text)) as u32
}

#[wasm_bindgen(js_name = nodeClipboardLineCount)]
pub fn node_clipboard_line_count(text: JsValue) -> u32 {
    clipboard_line_count_text(&crate::js_string_owned(&text)) as u32
}

#[wasm_bindgen(js_name = nodeNormalizeClipboardLineEndings)]
pub fn node_normalize_clipboard_line_endings(text: JsValue) -> String {
    normalize_clipboard_line_endings_text(&crate::js_string_owned(&text))
}

#[wasm_bindgen(js_name = nodeClipboardSafeError)]
pub fn node_clipboard_safe_error(error: JsValue) -> String {
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

#[wasm_bindgen(js_name = nodeClipboardPlaceholderText)]
pub fn node_clipboard_placeholder_text(net: String) -> String {
    clipboard_placeholder_text(&net)
}

#[wasm_bindgen(js_name = nodeStartTraceTauriShape)]
pub fn node_start_trace_tauri_shape(adapter: String) -> JsValue {
    tauri_shape(&adapter)
}

#[wasm_bindgen(js_name = nodeResolvePublicTauriInvoke)]
pub fn node_resolve_public_tauri_invoke() -> JsValue {
    resolved_invoke()
}

#[wasm_bindgen(js_name = nodeStartTraceFrontend)]
pub fn node_start_trace_frontend(stage: JsValue, options: JsValue) -> bool {
    let resolved = resolved_invoke();
    let Some(invoke) = property(&resolved, "invoke").dyn_into::<Function>().ok() else {
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
    set(&details, "invokeAdapter", &property(&resolved, "adapter"));

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

    if let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str(COMMAND),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |error: JsValue| {
            let console = property(&global(), "console");
            if let Some(log) = function(&console, "error") {
                let message = {
                    let candidate = property(&error, "message");
                    if crate::js_boolean(&candidate) {
                        crate::js_string_owned(&candidate)
                    } else {
                        crate::js_string_owned(&error)
                    }
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
    fn blocked_key_policy_matches_legacy_sensitive_fields() {
        for key in [
            "apiToken",
            "walletAddress",
            "commandPreview",
            "rpcEndpoint",
            "appDir",
        ] {
            assert!(blocked_key(key), "{key}");
        }
        assert!(!blocked_key("network"));
        assert!(!blocked_key("result"));
    }

    #[test]
    fn safe_text_normalizes_and_limits() {
        assert_eq!(safe_text_str(" a\tb\nc ", ""), "a b c");
        assert_eq!(safe_text_str("", "fallback"), "fallback");
        let long = "x".repeat(300);
        assert_eq!(safe_text_str(&long, "").len(), 220);
    }

    #[test]
    fn clipboard_counts_match_legacy_contract() {
        assert_eq!(clipboard_character_count_text("A😀B"), 3);
        assert_eq!(clipboard_line_count_text(""), 0);
        assert_eq!(clipboard_line_count_text("a"), 1);
        assert_eq!(clipboard_line_count_text("a\nb\n"), 3);
    }

    #[test]
    fn clipboard_line_endings_match_legacy_contract() {
        assert_eq!(
            normalize_clipboard_line_endings_text("a\r\nb\rc\nd"),
            "a\r\nb\r\nc\r\nd"
        );
    }

    #[test]
    fn clipboard_safe_error_matches_sensitive_policy() {
        assert_eq!(
            clipboard_safe_error_text(" token leaked\n"),
            "clipboard write failed with a sensitive error"
        );
        assert_eq!(
            clipboard_safe_error_text("  ordinary\t failure  "),
            "ordinary  failure"
        );
        assert_eq!(clipboard_safe_error_text(""), "clipboard write failed");
    }

    #[test]
    fn clipboard_placeholder_labels_match_node_profiles() {
        assert_eq!(
            clipboard_placeholder_text("mainnet"),
            "Mainnet log is empty."
        );
        assert_eq!(
            clipboard_placeholder_text("testnet10"),
            "Testnet 10 log is empty."
        );
        assert_eq!(
            clipboard_placeholder_text("testnet13"),
            "Testnet 13 log is empty."
        );
        assert_eq!(clipboard_placeholder_text("custom"), "custom log is empty.");
    }
}
