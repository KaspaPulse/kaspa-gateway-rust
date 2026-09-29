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
}
