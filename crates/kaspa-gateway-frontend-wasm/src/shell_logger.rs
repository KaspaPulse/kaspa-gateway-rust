use js_sys::{Array, Date, Function, JSON, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

use crate::{js_boolean, js_string_owned};

const LOG_STORAGE_KEY: &str = "kgw-frontend-log-buffer";
const MAX_BUFFER: u32 = 1500;

fn global() -> JsValue {
    js_sys::global().into()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn set_property(target: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(name), value).map(|_| ())
}

fn method(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call1(target: &JsValue, name: &str, first: &JsValue) -> Result<JsValue, JsValue> {
    method(target, name)
        .ok_or_else(|| JsValue::from_str(name))?
        .call1(target, first)
}

fn call2(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    method(target, name)
        .ok_or_else(|| JsValue::from_str(name))?
        .call2(target, first, second)
}
fn truthy_string(value: &JsValue, fallback: &str) -> String {
    if js_boolean(value) {
        js_string_owned(value)
    } else {
        fallback.to_owned()
    }
}

fn error_stack_or_message(value: &JsValue) -> Option<String> {
    if !value.is_instance_of::<js_sys::Error>() {
        return None;
    }
    for key in ["stack", "message"] {
        let candidate = property(value, key);
        if js_boolean(&candidate) {
            return Some(js_string_owned(&candidate));
        }
    }
    Some(js_string_owned(value))
}

fn safe_details(details: &JsValue) -> String {
    if let Some(text) = error_stack_or_message(details) {
        return text;
    }
    if details.is_object()
        && !details.is_null()
        && let Ok(value) = JSON::stringify(details)
    {
        return String::from(value);
    }
    js_string_owned(details)
}

fn window() -> JsValue {
    property(&global(), "window")
}

fn local_storage() -> Option<JsValue> {
    let storage = property(&global(), "localStorage");
    js_boolean(&storage).then_some(storage)
}

fn location_href() -> String {
    let location = property(&global(), "location");
    let href = property(&location, "href");
    js_string_owned(&href)
}

fn console_call(name: &str, value: &str) {
    let console = property(&global(), "console");
    if let Some(function) = method(&console, name) {
        let _ = function.call1(&console, &JsValue::from_str(value));
    }
}

fn array_from(value: JsValue) -> Array {
    if Array::is_array(&value) {
        value.unchecked_into::<Array>()
    } else {
        Array::new()
    }
}
fn trim_buffer(array: &Array) {
    while array.length() > MAX_BUFFER {
        array.shift();
    }
}

fn push_local(entry: &JsValue) {
    let win = window();
    if js_boolean(&win) {
        let logs = array_from(property(&win, "__KGW_FRONTEND_LOGS"));
        logs.push(entry);
        trim_buffer(&logs);
        let _ = set_property(&win, "__KGW_FRONTEND_LOGS", logs.as_ref());
    }

    let Some(storage) = local_storage() else {
        return;
    };
    let raw = call1(&storage, "getItem", &JsValue::from_str(LOG_STORAGE_KEY))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "[]".to_owned());
    let existing = JSON::parse(&raw).ok().map(array_from).unwrap_or_default();
    existing.push(entry);
    trim_buffer(&existing);
    if let Ok(serialized) = JSON::stringify(existing.as_ref()) {
        let _ = call2(
            &storage,
            "setItem",
            &JsValue::from_str(LOG_STORAGE_KEY),
            serialized.as_ref(),
        );
    }
}

fn tauri_invoke() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    for owner_name in ["core", "tauri"] {
        let owner = property(&tauri, owner_name);
        if let Ok(function) = property(&owner, "invoke").dyn_into::<Function>() {
            return Some(function);
        }
    }
    property(&win, "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .ok()
}

fn absorb_rejection(value: &JsValue) {
    let Some(catch) = method(value, "catch") else {
        return;
    };
    let callback = Closure::<dyn FnMut(JsValue)>::new(|_| {});
    let _ = catch.call1(value, callback.as_ref());
    callback.forget();
}
fn send_to_tauri_log(entry: &JsValue) {
    let Some(invoke) = tauri_invoke() else {
        return;
    };
    let details = js_string_owned(&property(entry, "details"));
    let source = js_string_owned(&property(entry, "source"));
    let target = if source.is_empty() {
        "frontend".to_owned()
    } else {
        format!("frontend:{source}")
    };
    let level = truthy_string(&property(entry, "level"), "log").to_uppercase();
    let message = js_string_owned(&property(entry, "message"));
    let full_message = if details.is_empty() {
        message
    } else {
        format!("{message} :: {details}")
    };

    let request = Object::new();
    let _ = set_property(request.as_ref(), "level", &JsValue::from_str(&level));
    let _ = set_property(request.as_ref(), "target", &JsValue::from_str(&target));
    let _ = set_property(
        request.as_ref(),
        "message",
        &JsValue::from_str(&full_message),
    );
    let args = Object::new();
    let _ = set_property(args.as_ref(), "request", request.as_ref());

    if let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_log_append"),
        args.as_ref(),
    ) {
        absorb_rejection(&result);
    }
}

#[wasm_bindgen(js_name = shellLoggerLog)]
pub fn log(level: JsValue, message: JsValue, details: JsValue, source: JsValue) -> JsValue {
    let entry = Object::new();
    let time = String::from(Date::new_0().to_iso_string());
    let level = truthy_string(&level, "log");
    let source = js_string_owned(&source);
    let message = truthy_string(&message, "");
    let details = safe_details(&details);

    let _ = set_property(entry.as_ref(), "time", &JsValue::from_str(&time));
    let _ = set_property(entry.as_ref(), "level", &JsValue::from_str(&level));
    let _ = set_property(entry.as_ref(), "source", &JsValue::from_str(&source));
    let _ = set_property(entry.as_ref(), "message", &JsValue::from_str(&message));
    let _ = set_property(entry.as_ref(), "details", &JsValue::from_str(&details));
    let _ = set_property(entry.as_ref(), "href", &JsValue::from_str(&location_href()));

    let suffix = if details.is_empty() {
        String::new()
    } else {
        format!(" :: {details}")
    };
    let line = format!("[KGW-FRONTEND][{level}][{source}] {time} - {message}{suffix}");
    console_call(
        match level.as_str() {
            "error" => "error",
            "warn" => "warn",
            _ => "log",
        },
        &line,
    );
    push_local(entry.as_ref());
    send_to_tauri_log(entry.as_ref());
    entry.into()
}
fn translated_shell_error() -> String {
    let win = window();
    if let Ok(translate) = property(&win, "kgwT").dyn_into::<Function>()
        && let Ok(value) = translate.call1(
            &win,
            &JsValue::from_str("runtime.shellErrorNavigationPreserved"),
        )
    {
        return js_string_owned(&value);
    }
    "Shell/runtime error. Navigation was preserved.".to_owned()
}

fn fatal_error_text(error: &JsValue) -> String {
    error_stack_or_message(error).unwrap_or_else(|| js_string_owned(error))
}

#[wasm_bindgen(js_name = shellLoggerFatal)]
pub fn fatal(error: JsValue, source: JsValue) {
    let _ = log(
        JsValue::from_str("error"),
        JsValue::from_str("fatal"),
        error.clone(),
        source,
    );

    let document = property(&global(), "document");
    if !js_boolean(&document) {
        return;
    }
    let mut panel = call1(
        &document,
        "getElementById",
        &JsValue::from_str("kgwFatalPanel"),
    )
    .unwrap_or(JsValue::NULL);
    if !js_boolean(&panel) {
        panel = match call1(&document, "createElement", &JsValue::from_str("pre")) {
            Ok(value) => value,
            Err(_) => return,
        };
        let _ = set_property(&panel, "id", &JsValue::from_str("kgwFatalPanel"));
        let style = property(&panel, "style");
        let css = [
            "position:fixed",
            "right:12px",
            "bottom:12px",
            "width:min(720px, calc(100vw - 24px))",
            "max-height:260px",
            "overflow:auto",
            "z-index:999999",
            "padding:10px",
            "margin:0",
            "border:1px solid rgba(248,113,113,.65)",
            "border-radius:8px",
            "background:#111827",
            "color:#fca5a5",
            "font:12px Consolas, monospace",
            "white-space:pre-wrap",
            "box-shadow:0 12px 28px rgba(0,0,0,.42)",
        ]
        .join(";");
        let _ = set_property(&style, "cssText", &JsValue::from_str(&css));
        let body = property(&document, "body");
        let _ = call1(&body, "appendChild", &panel);
    }
    let text = format!(
        "{}\n\n{}",
        translated_shell_error(),
        fatal_error_text(&error)
    );
    let _ = set_property(&panel, "textContent", &JsValue::from_str(&text));
}

#[wasm_bindgen(js_name = shellLoggerGetBufferedLogs)]
pub fn get_buffered_logs() -> JsValue {
    let Some(storage) = local_storage() else {
        return Array::new().into();
    };
    let raw = call1(&storage, "getItem", &JsValue::from_str(LOG_STORAGE_KEY))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "[]".to_owned());
    JSON::parse(&raw)
        .ok()
        .filter(Array::is_array)
        .unwrap_or_else(|| Array::new().into())
}

#[wasm_bindgen(js_name = shellLoggerClearBufferedLogs)]
pub fn clear_buffered_logs() {
    if let Some(storage) = local_storage() {
        let _ = call1(&storage, "removeItem", &JsValue::from_str(LOG_STORAGE_KEY));
    }
    let win = window();
    if js_boolean(&win) {
        let logs = Array::new();
        let _ = set_property(&win, "__KGW_FRONTEND_LOGS", logs.as_ref());
    }
}

fn event_value(event: &JsValue, name: &str) -> JsValue {
    property(event, name)
}

fn install_error_listeners(win: &JsValue) {
    let error_callback = Closure::<dyn FnMut(JsValue)>::new(|event| {
        let details = Object::new();
        for key in ["message", "filename", "lineno", "colno"] {
            let _ = set_property(details.as_ref(), key, &event_value(&event, key));
        }
        let error = event_value(&event, "error");
        let error_text = if js_boolean(&error) {
            error_stack_or_message(&error)
                .map(|text| JsValue::from_str(&text))
                .unwrap_or(error)
        } else {
            error
        };
        let _ = set_property(details.as_ref(), "error", &error_text);
        let _ = log(
            JsValue::from_str("error"),
            JsValue::from_str("window error"),
            details.into(),
            JsValue::from_str("window"),
        );
    });
    let _ = call2(
        win,
        "addEventListener",
        &JsValue::from_str("error"),
        error_callback.as_ref(),
    );
    error_callback.forget();

    let rejection_callback = Closure::<dyn FnMut(JsValue)>::new(|event| {
        let reason = event_value(&event, "reason");
        let details = if js_boolean(&reason) {
            error_stack_or_message(&reason)
                .map(|text| JsValue::from_str(&text))
                .unwrap_or(reason)
        } else {
            reason
        };
        let _ = log(
            JsValue::from_str("error"),
            JsValue::from_str("unhandled rejection"),
            details,
            JsValue::from_str("promise"),
        );
    });
    let _ = call2(
        win,
        "addEventListener",
        &JsValue::from_str("unhandledrejection"),
        rejection_callback.as_ref(),
    );
    rejection_callback.forget();
}

fn startup_clear_once(win: &JsValue) {
    if js_boolean(&property(win, "__kgwClearLogOnStartupDone")) {
        return;
    }
    let _ = set_property(win, "__kgwClearLogOnStartupDone", &JsValue::from_bool(true));
    let Some(storage) = local_storage() else {
        return;
    };
    let enabled = call1(
        &storage,
        "getItem",
        &JsValue::from_str("kgw.clearLogOnStartup"),
    )
    .ok()
    .and_then(|value| value.as_string())
    .is_some_and(|value| value == "1");
    if !enabled {
        return;
    }
    let Some(invoke) = tauri_invoke() else {
        return;
    };
    match invoke.call1(&JsValue::UNDEFINED, &JsValue::from_str("kgw_log_clear")) {
        Ok(result) => {
            if let Some(then) = method(&result, "then") {
                let success = Closure::<dyn FnMut(JsValue)>::new(|_| {
                    console_call("info", "[KGW] log cleared on startup by user setting");
                });
                let _ = then.call1(&result, success.as_ref());
                success.forget();
            } else {
                console_call("info", "[KGW] log cleared on startup by user setting");
            }
            if let Some(catch) = method(&result, "catch") {
                let failure = Closure::<dyn FnMut(JsValue)>::new(|error| {
                    console_call(
                        "warn",
                        &format!(
                            "[KGW] failed to clear log on startup {}",
                            js_string_owned(&error)
                        ),
                    );
                });
                let _ = catch.call1(&result, failure.as_ref());
                failure.forget();
            }
        }
        Err(error) => console_call(
            "warn",
            &format!(
                "[KGW] failed to clear log on startup {}",
                js_string_owned(&error)
            ),
        ),
    }
}

#[wasm_bindgen(js_name = shellLoggerInstall)]
pub fn install() -> bool {
    let win = window();
    if !js_boolean(&win) || js_boolean(&property(&win, "__KGW_LOGGER_INSTALLED")) {
        return false;
    }
    if set_property(&win, "__KGW_LOGGER_INSTALLED", &JsValue::from_bool(true)).is_err() {
        return false;
    }
    install_error_listeners(&win);
    let _ = log(
        JsValue::from_str("log"),
        JsValue::from_str("logger installed"),
        JsValue::from_str(""),
        JsValue::from_str("logger"),
    );
    startup_clear_once(&win);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_preserve_legacy_storage_contract() {
        assert_eq!(LOG_STORAGE_KEY, "kgw-frontend-log-buffer");
        assert_eq!(MAX_BUFFER, 1500);
    }
}
