use js_sys::{Array, Function, Object, Promise, Reflect};
use std::cell::Cell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

thread_local! {
    static BUSY: Cell<bool> = const { Cell::new(false) };
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

fn q(selector: &str) -> JsValue {
    let root = call1(
        &document(),
        "getElementById",
        &JsValue::from_str("settings"),
    );
    if !is_present(&root) {
        return JsValue::UNDEFINED;
    }
    call1(&root, "querySelector", &JsValue::from_str(selector))
}

fn qa(selector: &str) -> Vec<JsValue> {
    let root = call1(
        &document(),
        "getElementById",
        &JsValue::from_str("settings"),
    );
    if !is_present(&root) {
        return Vec::new();
    }
    let list = call1(&root, "querySelectorAll", &JsValue::from_str(selector));
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(is_present)
        })
        .collect()
}

fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn set_attr(target: &JsValue, name: &str, value: &str) {
    if let Some(call) = function(target, "setAttribute") {
        let _ = call.call2(target, &JsValue::from_str(name), &JsValue::from_str(value));
    }
}

fn append_child(parent: &JsValue, child: &JsValue) {
    if let Some(call) = function(parent, "appendChild") {
        let _ = call.call1(parent, child);
    }
}

fn error_text(error: &JsValue) -> String {
    let message = text(&property(error, "message"));
    if message.is_empty() {
        text(error)
    } else {
        message
    }
}

fn status(message: &str, kind: &str) {
    let mut node = q("#settingsProfileStatus");
    let profile = q("#settingsApiProfile");
    if !is_present(&node) && is_present(&profile) {
        node = call1(&document(), "createElement", &JsValue::from_str("div"));
        set_property(&node, "id", &JsValue::from_str("settingsProfileStatus"));
        set_attr(&node, "role", "status");
        set_attr(&node, "aria-live", "polite");
        let parent = property(&profile, "parentElement");
        if is_present(&parent) {
            append_child(&parent, &node);
        }
    }
    if is_present(&node) {
        set_property(&node, "textContent", &JsValue::from_str(message));
        set_property(&dataset(&node), "kind", &JsValue::from_str(kind));
        let _ = crate::apply_status_tone(node, JsValue::from_str(kind));
    }
}

fn invoke_api() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    for candidate in [
        property(&property(&tauri, "core"), "invoke"),
        property(&property(&tauri, "tauri"), "invoke"),
        property(&tauri, "invoke"),
        property(&window(), "__TAURI_INVOKE__"),
    ] {
        if let Ok(function) = candidate.dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}

async fn invoke(command: &str, args: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let call = invoke_api().ok_or_else(|| JsValue::from_str("Tauri invoke is not available"))?;
    let result = match args {
        Some(args) => call.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?,
        None => call.call1(&JsValue::UNDEFINED, &JsValue::from_str(command))?,
    };
    JsFuture::from(Promise::resolve(&result)).await
}

fn endpoint_reset_supported(key: &str, persisted: bool) -> bool {
    key == "base_url" || persisted
}

fn update_reset_availability(row: &JsValue) {
    let button = q("#settingsResetSelectedEndpoint");
    if !is_present(&button) {
        return;
    }
    let data = dataset(row);
    let key = text(&property(&data, "apiKey"));
    let persisted = text(&property(&data, "kgwPersistedEndpoint")) == "true";
    let supported = endpoint_reset_supported(&key, persisted);
    set_property(&button, "disabled", &JsValue::from_bool(!supported));
    set_attr(
        &button,
        "aria-disabled",
        if supported { "false" } else { "true" },
    );
    set_property(
        &button,
        "title",
        &JsValue::from_str(if supported {
            "Reset the selected persisted endpoint to its canonical default."
        } else {
            "This endpoint is not stored by the current Settings contract."
        }),
    );
}

fn combine_url() {
    let base = q("#settingsApiBase");
    let path = q("#settingsApiPath");
    let preview = q("#settingsApiPreview");
    if !is_present(&base) || !is_present(&path) || !is_present(&preview) {
        return;
    }
    let left = text(&property(&base, "value"))
        .trim_end_matches('/')
        .to_owned();
    let right = text(&property(&path, "value"))
        .trim_start_matches('/')
        .to_owned();
    let value = if !left.is_empty() && !right.is_empty() {
        format!("{left}/{right}")
    } else if !left.is_empty() {
        left
    } else {
        right
    };
    set_property(&preview, "value", &JsValue::from_str(&value));
}

fn select_endpoint_internal(row: &JsValue) {
    if !is_present(row) {
        return;
    }
    for node in qa(".tree-row") {
        if let Some(class_list) = Some(property(&node, "classList")).filter(is_present)
            && let Some(remove) = function(&class_list, "remove")
        {
            let _ = remove.call1(&class_list, &JsValue::from_str("is-selected"));
        }
    }
    let class_list = property(row, "classList");
    if let Some(add) = function(&class_list, "add") {
        let _ = add.call1(&class_list, &JsValue::from_str("is-selected"));
    }
    let data = dataset(row);
    for (selector, key) in [
        ("#settingsApiKey", "apiKey"),
        ("#settingsApiDescription", "apiDesc"),
        ("#settingsApiBase", "apiBase"),
        ("#settingsApiPath", "apiPath"),
    ] {
        let node = q(selector);
        if is_present(&node) {
            set_property(&node, "value", &property(&data, key));
        }
    }
    combine_url();
    update_reset_availability(row);
}

fn array(value: &JsValue) -> Array {
    if Array::is_array(value) {
        Array::from(value)
    } else {
        Array::new()
    }
}

fn active_profile(settings: &JsValue) -> JsValue {
    let profiles = array(&property(settings, "api_profiles"));
    let active = text(&property(settings, "active_api_profile"));
    let mut first = JsValue::UNDEFINED;
    for profile in profiles.iter() {
        if !is_present(&first) {
            first = profile.clone();
        }
        if text(&property(&profile, "name")) == active {
            return profile;
        }
    }
    first
}

fn apply_backend_profiles(settings: &JsValue) {
    if !is_present(settings) || !settings.is_object() {
        return;
    }
    let profiles = array(&property(settings, "api_profiles"));
    let select = q("#settingsApiProfile");
    if is_present(&select) {
        set_property(&select, "innerHTML", &JsValue::from_str(""));
        for profile in profiles.iter() {
            let option = call1(&document(), "createElement", &JsValue::from_str("option"));
            let name = text(&property(&profile, "name"));
            set_property(&option, "value", &JsValue::from_str(&name));
            set_property(&option, "textContent", &JsValue::from_str(&name));
            append_child(&select, &option);
        }
        let active = text(&property(settings, "active_api_profile"));
        let first = profiles.get(0);
        let fallback = if is_present(&first) {
            text(&property(&first, "name"))
        } else {
            String::new()
        };
        set_property(
            &select,
            "value",
            &JsValue::from_str(if active.is_empty() {
                &fallback
            } else {
                &active
            }),
        );
    }

    let active = active_profile(settings);
    let base_url = text(&property(&active, "base_url"));
    let endpoints = array(&property(&active, "endpoints"));
    for row in qa(".tree-row[data-api-key]") {
        let data = dataset(&row);
        let key = text(&property(&data, "apiKey"));
        set_property(&data, "kgwPersistedEndpoint", &JsValue::from_str("false"));
        if key == "base_url" && is_present(&active) {
            set_property(&data, "apiBase", &JsValue::from_str(&base_url));
            set_property(&data, "apiPath", &JsValue::from_str(""));
            set_property(&data, "kgwPersistedEndpoint", &JsValue::from_str("true"));
            continue;
        }
        for endpoint in endpoints.iter() {
            if text(&property(&endpoint, "name")) == key {
                set_property(&data, "apiBase", &JsValue::from_str(&base_url));
                set_property(&data, "apiPath", &property(&endpoint, "path"));
                set_property(&data, "kgwPersistedEndpoint", &JsValue::from_str("true"));
                break;
            }
        }
    }
    let selected = {
        let selected = q(".tree-row.is-selected");
        if is_present(&selected) {
            selected
        } else {
            q(".tree-row")
        }
    };
    select_endpoint_internal(&selected);
}

async fn refresh_profiles() -> Result<JsValue, JsValue> {
    let settings = invoke("settings_load", None).await?;
    apply_backend_profiles(&settings);
    Ok(settings)
}

async fn mutate(command: &'static str, args: JsValue, success: &'static str) {
    let should_run = BUSY.with(|busy| {
        if busy.get() {
            false
        } else {
            busy.set(true);
            true
        }
    });
    if !should_run {
        return;
    }
    status("Working...", "loading");
    match invoke(command, Some(&args)).await {
        Ok(settings) => {
            apply_backend_profiles(&settings);
            status(success, "success");
        }
        Err(error) => status(&format!("Action failed: {}", error_text(&error)), "error"),
    }
    BUSY.with(|busy| busy.set(false));
}

fn prompt(message: &str, default_value: &str) -> Option<String> {
    let prompt = property(&window(), "prompt").dyn_into::<Function>().ok()?;
    let value = prompt
        .call2(
            &window(),
            &JsValue::from_str(message),
            &JsValue::from_str(default_value),
        )
        .ok()?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(text(&value))
    }
}

fn args1(key: &str, value: &str) -> JsValue {
    let args = Object::new();
    set_property(args.as_ref(), key, &JsValue::from_str(value));
    args.into()
}

fn args2(key1: &str, value1: &str, key2: &str, value2: &str) -> JsValue {
    let args = Object::new();
    set_property(args.as_ref(), key1, &JsValue::from_str(value1));
    set_property(args.as_ref(), key2, &JsValue::from_str(value2));
    args.into()
}

fn add_action() {
    spawn_local(async {
        let Some(name) = prompt(
            "New API profile name (letters, numbers, dot, dash or underscore):",
            "",
        ) else {
            return;
        };
        mutate(
            "settings_profile_add",
            args1("name", &name),
            "API profile added and saved.",
        )
        .await;
    });
}

fn rename_action() {
    spawn_local(async {
        let current = text(&property(&q("#settingsApiProfile"), "value"));
        if current.is_empty() {
            status("Select an API profile first.", "error");
            return;
        }
        let Some(new_name) = prompt("New API profile name:", &current) else {
            return;
        };
        if new_name == current {
            return;
        }
        mutate(
            "settings_profile_rename",
            args2("name", &current, "newName", &new_name),
            "API profile renamed and saved.",
        )
        .await;
    });
}

fn delete_action() {
    spawn_local(async {
        let name = text(&property(&q("#settingsApiProfile"), "value"));
        if name.is_empty() {
            status("Select an API profile first.", "error");
            return;
        }
        let accepted = crate::settings_contract::confirm_user_action(JsValue::from_str(&format!(
            "Delete API profile \"{name}\"?"
        )))
        .await
        .unwrap_or(false);
        if !accepted {
            return;
        }
        mutate(
            "settings_profile_delete",
            args1("name", &name),
            "API profile deleted and saved.",
        )
        .await;
    });
}

fn select_action() {
    spawn_local(async {
        let name = text(&property(&q("#settingsApiProfile"), "value"));
        if name.is_empty() {
            return;
        }
        mutate(
            "settings_profile_select",
            args1("name", &name),
            "Active API profile saved.",
        )
        .await;
    });
}

fn reset_endpoint_action() {
    spawn_local(async {
        let row = q(".tree-row.is-selected");
        let profile_name = text(&property(&q("#settingsApiProfile"), "value"));
        let endpoint_name = text(&property(&dataset(&row), "apiKey"));
        if !is_present(&row) || profile_name.is_empty() || endpoint_name.is_empty() {
            status("Select a persisted endpoint first.", "error");
            return;
        }
        mutate(
            "settings_reset_selected_endpoint",
            args2("profileName", &profile_name, "endpointName", &endpoint_name),
            "Selected endpoint reset to its canonical default and saved.",
        )
        .await;
    });
}

fn bind_click(id: &str, key: &str, handler: fn()) {
    let button = q(&format!("#{id}"));
    if !is_present(&button) {
        return;
    }
    let data = dataset(&button);
    if text(&property(&data, "kgwProfileRustBound")) == key {
        return;
    }
    set_property(&data, "kgwProfileRustBound", &JsValue::from_str(key));
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        if let Some(prevent) = function(&event, "preventDefault") {
            let _ = prevent.call0(&event);
        }
        handler();
    }) as Box<dyn FnMut(JsValue)>);
    if let Some(add) = function(&button, "addEventListener") {
        let _ = add.call2(
            &button,
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
    }
    callback.forget();
}

fn bind_select() {
    let select = q("#settingsApiProfile");
    if !is_present(&select) {
        return;
    }
    let data = dataset(&select);
    if text(&property(&data, "kgwProfileRustBound")) == "select" {
        return;
    }
    set_property(&data, "kgwProfileRustBound", &JsValue::from_str("select"));
    let callback = Closure::wrap(Box::new(move |_event: JsValue| {
        select_action();
    }) as Box<dyn FnMut(JsValue)>);
    if let Some(add) = function(&select, "addEventListener") {
        let _ = add.call2(
            &select,
            &JsValue::from_str("change"),
            callback.as_ref().unchecked_ref(),
        );
    }
    callback.forget();
}

#[wasm_bindgen(js_name = settingsProfilesInstall)]
pub fn install() {
    bind_click("settingsProfileAdd", "add", add_action);
    bind_click("settingsProfileRename", "rename", rename_action);
    bind_click("settingsProfileDelete", "delete", delete_action);
    bind_click(
        "settingsResetSelectedEndpoint",
        "reset-endpoint",
        reset_endpoint_action,
    );
    bind_select();
    spawn_local(async {
        if let Err(error) = refresh_profiles().await {
            status(
                &format!("Profile load failed: {}", error_text(&error)),
                "error",
            );
        }
    });
}

#[wasm_bindgen(js_name = settingsProfilesSelectEndpoint)]
pub fn select_endpoint(row: JsValue) {
    select_endpoint_internal(&row);
}

#[wasm_bindgen(js_name = settingsProfilesRefresh)]
pub async fn refresh() -> Result<JsValue, JsValue> {
    refresh_profiles().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_reset_support_matches_legacy_contract() {
        assert!(endpoint_reset_supported("base_url", false));
        assert!(endpoint_reset_supported("transactions", true));
        assert!(!endpoint_reset_supported("transactions", false));
    }
}
