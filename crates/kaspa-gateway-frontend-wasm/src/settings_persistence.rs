use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use std::cell::Cell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const STORAGE_KEY: &str = "kgw-settings-python-exact-state";

thread_local! {
    static SAVE_IN_FLIGHT: Cell<bool> = const { Cell::new(false) };
}

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

fn q(selector: &str) -> JsValue {
    let root = root();
    if !is_present(&root) {
        return JsValue::UNDEFINED;
    }
    call1(&root, "querySelector", &JsValue::from_str(selector))
}

fn qa(selector: &str) -> Vec<JsValue> {
    let root = root();
    if !is_present(&root) {
        return Vec::new();
    }
    let list = call1(&root, "querySelectorAll", &JsValue::from_str(selector));
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .filter(is_present)
        .collect()
}

fn storage() -> JsValue {
    property(&window(), "localStorage")
}

fn storage_remove() {
    if let Some(call) = function(&storage(), "removeItem") {
        let _ = call.call1(&storage(), &JsValue::from_str(STORAGE_KEY));
    }
}

fn storage_get() -> String {
    function(&storage(), "getItem")
        .and_then(|call| call.call1(&storage(), &JsValue::from_str(STORAGE_KEY)).ok())
        .map(|value| crate::js_string_owned(&value))
        .unwrap_or_default()
}

fn storage_set_pretty(state: &JsValue) -> Result<(), JsValue> {
    let json = property(&global(), "JSON");
    let stringify = function(&json, "stringify")
        .ok_or_else(|| JsValue::from_str("JSON.stringify is unavailable"))?;
    let serialized = stringify.call3(&json, state, &JsValue::NULL, &JsValue::from_f64(2.0))?;
    let set_item = function(&storage(), "setItem")
        .ok_or_else(|| JsValue::from_str("localStorage.setItem is unavailable"))?;
    set_item.call2(&storage(), &JsValue::from_str(STORAGE_KEY), &serialized)?;
    Ok(())
}

fn set_save_enabled(enabled: bool) {
    let save = q("#settingsSaveSettings");
    if is_present(&save) {
        set_property(&save, "disabled", &JsValue::from_bool(!enabled));
    }
}

fn error_text(error: &JsValue) -> String {
    let message = crate::js_string_owned(&property(error, "message"));
    if message.is_empty() {
        crate::js_string_owned(error)
    } else {
        message
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

fn trace(phase: &str, details: &JsValue) {
    let Some(invoke) = invoke_api() else {
        return;
    };
    let nested = Object::new();
    set_property(
        nested.as_ref(),
        "patch",
        &JsValue::from_str("KGW_SETTINGS_UI_TRACE_PATCH_R48B3"),
    );
    set_property(
        nested.as_ref(),
        "owner",
        &JsValue::from_str("settings-rust-persistence"),
    );
    set_property(nested.as_ref(), "phase", &JsValue::from_str(phase));
    set_property(nested.as_ref(), "details", details);
    let nested_text = JSON::stringify(nested.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    let args = Object::new();
    set_property(args.as_ref(), "scope", &JsValue::from_str("settings"));
    set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    set_property(
        args.as_ref(),
        "action",
        &JsValue::from_str("settings-action"),
    );
    set_property(args.as_ref(), "phase", &JsValue::from_str(phase));
    set_property(args.as_ref(), "details", &JsValue::from_str(&nested_text));
    if let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let ignore = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&ignore);
        ignore.forget();
    }
}

fn logger_log(message: &str) {
    let factory = property(&window(), "kgwCreateLogger");
    let Ok(factory) = factory.dyn_into::<Function>() else {
        return;
    };
    let Ok(logger) = factory.call1(&window(), &JsValue::from_str("settings")) else {
        return;
    };
    if let Some(log) = function(&logger, "log") {
        let details = Object::new();
        set_property(details.as_ref(), "patch", &JsValue::from_str("R75"));
        set_property(
            details.as_ref(),
            "owner",
            &JsValue::from_str("settings-single-save-direct-shell"),
        );
        let _ = log.call2(&logger, &JsValue::from_str(message), details.as_ref());
    }
}

fn target_details() -> JsValue {
    let details = Object::new();
    set_property(
        details.as_ref(),
        "targetId",
        &JsValue::from_str("settingsSaveSettings"),
    );
    details.into()
}

fn prefs_join(prefs: &JsValue, name: &str) -> String {
    let value = property(prefs, name);
    if !Array::is_array(&value) {
        return String::new();
    }
    Array::from(&value)
        .iter()
        .map(|value| crate::js_string_owned(&value))
        .collect::<Vec<_>>()
        .join(",")
}

#[wasm_bindgen(js_name = settingsPersistenceSave)]
pub async fn save() -> bool {
    let should_start = SAVE_IN_FLIGHT.with(|flag| {
        if flag.get() {
            false
        } else {
            flag.set(true);
            true
        }
    });
    if !should_start {
        trace("r75-save-skip-busy", &target_details());
        return false;
    }

    trace("r75-save-begin", &target_details());

    let result = async {
        crate::settings_paths::repair_before_save().await;

        if !crate::settings_state::display_validate_for_save() {
            trace("r75-save-validation-failed", &target_details());
            set_save_enabled(true);
            return false;
        }

        let collected = crate::settings_state::state_collect();
        let state =
            crate::settings_state::display_reapply_state(collected, "settings-save-r75".to_owned());

        if let Err(error) = storage_set_pretty(&state) {
            let details = Object::new();
            set_property(
                details.as_ref(),
                "message",
                &JsValue::from_str(&error_text(&error)),
            );
            trace("r75-save-storage-error", details.as_ref());
        }

        let prefs = crate::settings_state::display_apply_shell_from_state(
            state,
            "settings-save-r75".to_owned(),
        );

        set_save_enabled(false);
        logger_log("settings saved");

        let details = Object::new();
        set_property(
            details.as_ref(),
            "targetId",
            &JsValue::from_str("settingsSaveSettings"),
        );
        set_property(
            details.as_ref(),
            "hasPrefs",
            &JsValue::from_bool(is_present(&prefs)),
        );
        for (name, key) in [
            ("languages", "languages"),
            ("currencies", "currencies"),
            ("tabs", "tabs"),
        ] {
            set_property(
                details.as_ref(),
                key,
                &JsValue::from_str(&prefs_join(&prefs, name)),
            );
        }
        trace("r75-save-complete", details.as_ref());
        true
    }
    .await;

    SAVE_IN_FLIGHT.with(|flag| flag.set(false));
    result
}

fn option_value_enabled(value: Option<bool>) -> bool {
    value != Some(false)
}

fn option_enabled(options: &JsValue, name: &str) -> bool {
    option_value_enabled(property(options, name).as_bool())
}

fn reset_defaults_sync(options: &JsValue) {
    let should_clear = option_enabled(options, "clearStorage");
    let should_apply_shell = option_enabled(options, "applyShell");

    if should_clear {
        storage_remove();
    }

    for node in qa("input[type='checkbox']") {
        let id = crate::js_string_owned(&property(&node, "id"));
        let checked = id != "settingsStartWindows" && id != "settingsEnableAutoRefresh";
        set_property(&node, "checked", &JsValue::from_bool(checked));
    }

    for (id, value) in [
        ("settingsLoggingLevel", "INFO"),
        ("settingsDatabasePath", ""),
        ("settingsExportPath", ""),
        ("settingsLogPath", ""),
        ("settingsBackupPath", ""),
        ("settingsApiProfile", "default"),
        ("settingsApiTimeout", "30"),
        ("settingsApiRetries", "3"),
        ("settingsApiRateLimit", "60"),
        ("settingsCacheTtl", "300"),
        ("settingsMaxConnections", "10"),
        ("settingsRequestTimeout", "30"),
    ] {
        let node = call1(&document(), "getElementById", &JsValue::from_str(id));
        if is_present(&node) {
            set_property(&node, "value", &JsValue::from_str(value));
        }
    }

    crate::settings_state::activate_outer("api-performance".to_owned());
    crate::settings_state::activate_inner("general".to_owned());

    let state = crate::settings_state::display_build_canonical_default_state(
        "settings-reset-defaults-r69".to_owned(),
        should_clear,
    );
    let _ = crate::settings_state::display_ensure_defaults("reset-defaults-r69-guard".to_owned());

    if should_apply_shell {
        let _ = crate::settings_state::display_apply_shell_from_state(
            state,
            "settings-reset-defaults-r69".to_owned(),
        );
    }

    set_save_enabled(false);
}

pub(crate) fn reset_defaults_for_init(options: JsValue) {
    reset_defaults_sync(&options);
    spawn_paths("reset-defaults");
}

#[wasm_bindgen(js_name = settingsPersistenceResetDefaults)]
pub async fn reset_defaults(options: JsValue) {
    reset_defaults_sync(&options);
    let _ = crate::settings_paths::load_defaults("reset-defaults".to_owned()).await;
}

fn schedule_shell(state: JsValue, reason: &'static str) {
    let callback = Closure::once_into_js(move || {
        let _ = crate::settings_state::display_apply_shell_from_state(state, reason.to_owned());
    });
    if let Ok(timeout) = property(&window(), "setTimeout").dyn_into::<Function>() {
        let _ = timeout.call2(&window(), &callback, &JsValue::from_f64(0.0));
    }
}

fn spawn_paths(reason: &'static str) {
    wasm_bindgen_futures::spawn_local(async move {
        let _ = crate::settings_paths::load_defaults(reason.to_owned()).await;
    });
}

#[wasm_bindgen(js_name = settingsPersistenceLoadSaved)]
pub fn load_saved() {
    let raw = storage_get();
    let parsed = if raw.is_empty() {
        Ok(JsValue::NULL)
    } else {
        JSON::parse(&raw)
    };

    match parsed {
        Ok(saved) if crate::js_boolean(&saved) => {
            if crate::settings_state::display_state_looks_legacy(saved.clone()) {
                let state = crate::settings_state::display_build_canonical_default_state(
                    "settings-load-missing-display-contract-r104".to_owned(),
                    true,
                );
                schedule_shell(state, "settings-load-missing-display-contract-r104");
                spawn_paths("settings-load-missing-display-contract-r104");
                return;
            }

            crate::settings_state::state_apply(saved.clone());
            let state = crate::settings_state::display_reapply_state(
                saved,
                "settings-load-r104".to_owned(),
            );
            schedule_shell(state, "settings-load-r104");
            spawn_paths("settings-load-r104");
        }
        Ok(_) => {
            let _ = crate::settings_state::display_build_canonical_default_state(
                "settings-load-defaults-r104".to_owned(),
                true,
            );
            spawn_paths("settings-load-defaults-r104");
        }
        Err(_) => {
            let _ = crate::settings_state::display_build_canonical_default_state(
                "settings-load-error-defaults-r104".to_owned(),
                false,
            );
            spawn_paths("settings-load-error-r104");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_key_is_stable() {
        assert_eq!(STORAGE_KEY, "kgw-settings-python-exact-state");
    }

    #[test]
    fn option_defaults_are_enabled() {
        assert!(option_value_enabled(None));
        assert!(option_value_enabled(Some(true)));
        assert!(!option_value_enabled(Some(false)));
    }
}
