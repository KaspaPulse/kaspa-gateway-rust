use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const CLEAR_KEY: &str = "kgw.clearLogOnStartup";
const DEV_KEY: &str = "kgw.developerOperationLogs";

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
}

fn document() -> JsValue {
    property(&global(), "document")
}

fn is_present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if !is_present(target) {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn set_property(target: &JsValue, name: &str, value: &JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(name), value);
}

fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|call| call.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn root() -> JsValue {
    call1(
        &document(),
        "getElementById",
        &JsValue::from_str("settings"),
    )
}

fn query(selector: &str) -> JsValue {
    let root = root();
    if !is_present(&root) {
        return JsValue::UNDEFINED;
    }
    call1(&root, "querySelector", &JsValue::from_str(selector))
}

fn create(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag))
}

fn append(parent: &JsValue, child: &JsValue) {
    if let Some(call) = function(parent, "appendChild") {
        let _ = call.call1(parent, child);
    }
}

fn storage() -> JsValue {
    property(&window(), "localStorage")
}

fn storage_get(key: &str) -> String {
    function(&storage(), "getItem")
        .and_then(|call| call.call1(&storage(), &JsValue::from_str(key)).ok())
        .map(|value| crate::js_string_owned(&value))
        .unwrap_or_default()
}

fn storage_set(key: &str, value: &str) {
    if let Some(call) = function(&storage(), "setItem") {
        let _ = call.call2(
            &storage(),
            &JsValue::from_str(key),
            &JsValue::from_str(value),
        );
    }
}

fn set_save_enabled(enabled: bool) {
    let save = query("#settingsSaveSettings");
    if is_present(&save) {
        set_property(&save, "disabled", &JsValue::from_bool(!enabled));
    }
}

fn invoke_api() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    for candidate in [
        property(&property(&tauri, "core"), "invoke"),
        property(&tauri, "invoke"),
        property(&window(), "__TAURI_INVOKE__"),
    ] {
        if let Ok(function) = candidate.dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}

fn trace_change(id: &str, key: &str, checked: bool) {
    let Some(invoke) = invoke_api() else {
        return;
    };
    let details = Object::new();
    set_property(
        details.as_ref(),
        "patch",
        &JsValue::from_str("KGW_SETTINGS_UI_TRACE_PATCH_R48B3"),
    );
    set_property(
        details.as_ref(),
        "owner",
        &JsValue::from_str("settings-rust-log-diagnostics"),
    );
    set_property(details.as_ref(), "targetId", &JsValue::from_str(id));
    set_property(details.as_ref(), "key", &JsValue::from_str(key));
    set_property(details.as_ref(), "checked", &JsValue::from_bool(checked));

    let args = Object::new();
    set_property(args.as_ref(), "scope", &JsValue::from_str("settings"));
    set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    set_property(
        args.as_ref(),
        "action",
        &JsValue::from_str("settings-log-diagnostics"),
    );
    set_property(
        args.as_ref(),
        "phase",
        &JsValue::from_str("r48b3-log-diagnostics-change"),
    );
    let details_text = js_sys::JSON::stringify(details.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    set_property(args.as_ref(), "details", &JsValue::from_str(&details_text));
    if let Ok(value) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&value);
        let ignore = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&ignore);
        ignore.forget();
    }
}

fn log_message(text: &str, enabled: bool) {
    let factory = property(&window(), "kgwCreateLogger");
    let Ok(factory) = factory.dyn_into::<Function>() else {
        return;
    };
    let Ok(logger) = factory.call1(&window(), &JsValue::from_str("settings")) else {
        return;
    };
    if let Some(log) = function(&logger, "log") {
        let _ = log.call1(
            &logger,
            &JsValue::from_str(&format!(
                "{text}: {}",
                if enabled { "enabled" } else { "disabled" }
            )),
        );
    }
}

fn make_row(id: &'static str, text: &'static str, key: &'static str) -> JsValue {
    let label = create("label");
    let style = property(&label, "style");
    for (name, value) in [
        ("display", "flex"),
        ("alignItems", "center"),
        ("gap", "10px"),
        ("margin", "8px 0"),
        ("cursor", "pointer"),
    ] {
        set_property(&style, name, &JsValue::from_str(value));
    }

    let input = create("input");
    set_property(&input, "id", &JsValue::from_str(id));
    set_property(&input, "type", &JsValue::from_str("checkbox"));
    set_property(
        &input,
        "checked",
        &JsValue::from_bool(storage_get(key) == "1"),
    );

    let input_for_change = input.clone();
    let change = Closure::wrap(Box::new(move |_event: JsValue| {
        let checked = crate::js_boolean(&property(&input_for_change, "checked"));
        storage_set(key, if checked { "1" } else { "0" });
        trace_change(id, key, checked);
        set_save_enabled(true);
        log_message(text, checked);
    }) as Box<dyn FnMut(JsValue)>);
    if let Some(add) = function(&input, "addEventListener") {
        let _ = add.call2(
            &input,
            &JsValue::from_str("change"),
            change.as_ref().unchecked_ref(),
        );
    }
    change.forget();

    let span = create("span");
    set_property(&span, "textContent", &JsValue::from_str(text));
    append(&label, &input);
    append(&label, &span);
    label
}

fn translated_title() -> String {
    let translator = property(&window(), "kgwT");
    if let Ok(translator) = translator.dyn_into::<Function>()
        && let Ok(value) =
            translator.call1(&window(), &JsValue::from_str("settings.logDiagnostics"))
    {
        let value = crate::js_string_owned(&value);
        if !value.is_empty() && value != "settings.logDiagnostics" {
            return value;
        }
    }
    "Log & Diagnostics".to_owned()
}

fn fallback_advanced_box(settings_root: &JsValue) -> JsValue {
    let list = call1(
        settings_root,
        "querySelectorAll",
        &JsValue::from_str("fieldset, section, div"),
    );
    let length = crate::js_number(&property(&list, "length"));
    if length.is_finite() && length > 0.0 {
        for index in 0..length as u32 {
            let Ok(node) = Reflect::get(&list, &JsValue::from_f64(index as f64)) else {
                continue;
            };
            let content = crate::js_string_owned(&property(&node, "textContent"));
            if content.contains("Logging Level")
                || content.contains("Check for updates on startup")
                || content.contains("Start with Windows")
            {
                return node;
            }
        }
    }
    settings_root.clone()
}
fn contains_diagnostics() -> bool {
    is_present(&query("#settingsClearLogOnStartup"))
        || is_present(&query("#settingsDeveloperOperationLogs"))
}

#[wasm_bindgen(js_name = settingsDiagnosticsInstall)]
pub fn install() {
    if contains_diagnostics() {
        return;
    }
    let settings_root = root();
    if !is_present(&settings_root) {
        return;
    }

    let block = create("div");
    set_property(&block, "id", &JsValue::from_str("settingsLogDiagnostics"));
    let style = property(&block, "style");
    for (name, value) in [
        ("marginTop", "12px"),
        ("paddingTop", "10px"),
        ("borderTop", "1px solid rgba(120,160,210,0.35)"),
    ] {
        set_property(&style, name, &JsValue::from_str(value));
    }

    let title = create("div");
    set_property(
        &title,
        "textContent",
        &JsValue::from_str(&translated_title()),
    );
    let title_style = property(&title, "style");
    set_property(&title_style, "fontWeight", &JsValue::from_str("700"));
    set_property(&title_style, "marginBottom", &JsValue::from_str("8px"));
    append(&block, &title);
    append(
        &block,
        &make_row(
            "settingsClearLogOnStartup",
            "Clear log on startup",
            CLEAR_KEY,
        ),
    );
    append(
        &block,
        &make_row(
            "settingsDeveloperOperationLogs",
            "Developer operation logs",
            DEV_KEY,
        ),
    );

    let logging = query("#settingsLoggingLevel");
    let advanced = if is_present(&logging) {
        for selector in ["fieldset", ".settings-section", "div"] {
            let candidate = call1(&logging, "closest", &JsValue::from_str(selector));
            if is_present(&candidate) {
                append(&candidate, &block);
                let style = property(&candidate, "style");
                set_property(&style, "overflow", &JsValue::from_str("visible"));
                set_property(&style, "minHeight", &JsValue::from_str("250px"));
                return;
            }
        }
        JsValue::UNDEFINED
    } else {
        JsValue::UNDEFINED
    };
    let target = if is_present(&advanced) {
        advanced
    } else {
        fallback_advanced_box(&settings_root)
    };
    append(&target, &block);
    let target_style = property(&target, "style");
    set_property(&target_style, "overflow", &JsValue::from_str("visible"));
    set_property(&target_style, "minHeight", &JsValue::from_str("250px"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_keys_remain_stable() {
        assert_eq!(CLEAR_KEY, "kgw.clearLogOnStartup");
        assert_eq!(DEV_KEY, "kgw.developerOperationLogs");
    }
}
