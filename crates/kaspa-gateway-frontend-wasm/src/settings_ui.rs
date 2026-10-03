use js_sys::{Date, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::spawn_local;

const NUMERIC_IDS: &[&str] = &[
    "settingsApiTimeout",
    "settingsRetryAttempts",
    "settingsBackoffFactor",
    "settingsMaxWorkers",
    "settingsMaxPages",
    "settingsPageDelay",
    "settingsPriceCacheHours",
    "settingsNetworkCacheHours",
    "settingsRefreshInterval",
];

const PLACEHOLDER_ACTIONS: &[&str] = &[
    "settingsAddressAdd",
    "settingsAddressDelete",
    "settingsAddressClear",
    "settingsAddressRefresh",
    "settingsDbRefresh",
    "settingsDbCompact",
    "settingsDbClearCaches",
    "settingsDbBackup",
    "settingsDbRestore",
    "settingsDbDelete",
];

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

fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn event_trusted(event: &JsValue) -> bool {
    crate::js_boolean(&property(event, "isTrusted"))
}

fn css_escape(value: &str) -> String {
    let css = property(&global(), "CSS");
    function(&css, "escape")
        .and_then(|call| call.call1(&css, &JsValue::from_str(value)).ok())
        .map(|value| text(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| value.to_owned())
}

fn set_save_enabled(enabled: bool) {
    let save = q("#settingsSaveSettings");
    if is_present(&save) {
        set_property(&save, "disabled", &JsValue::from_bool(!enabled));
    }
}

fn mark_dirty() {
    set_save_enabled(true);
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

fn trace(action: &str, phase: &str, details: &JsValue) {
    let Some(invoke) = invoke_api() else {
        return;
    };
    let nested = Object::new();
    for (name, value) in [
        ("patch", "KGW_SETTINGS_UI_TRACE_PATCH_R48B3"),
        ("owner", "settings-rust-ui-owner"),
        ("action", action),
        ("phase", phase),
    ] {
        set_property(nested.as_ref(), name, &JsValue::from_str(value));
    }
    set_property(nested.as_ref(), "details", details);
    let nested_text = js_sys::JSON::stringify(nested.as_ref())
        .ok()
        .map(|value| text(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());

    let args = Object::new();
    set_property(args.as_ref(), "scope", &JsValue::from_str("settings"));
    set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    set_property(args.as_ref(), "action", &JsValue::from_str(action));
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

fn logger(level: &str, message: &str, details: Option<&JsValue>) {
    let factory = property(&window(), "kgwCreateLogger");
    let Ok(factory) = factory.dyn_into::<Function>() else {
        return;
    };
    let Ok(logger) = factory.call1(&window(), &JsValue::from_str("settings")) else {
        return;
    };
    let Some(call) = function(&logger, level) else {
        return;
    };
    match details {
        Some(details) => {
            let _ = call.call2(&logger, &JsValue::from_str(message), details);
        }
        None => {
            let _ = call.call1(&logger, &JsValue::from_str(message));
        }
    }
}

fn details_pairs(values: &[(&str, String)]) -> JsValue {
    let details = Object::new();
    for (name, value) in values {
        set_property(details.as_ref(), name, &JsValue::from_str(value));
    }
    details.into()
}

fn normalize_numeric_fields() {
    for id in NUMERIC_IDS {
        let node = q(&format!("#{}", css_escape(id)));
        if !is_present(&node) {
            continue;
        }
        set_property(&node, "type", &JsValue::from_str("text"));
        set_property(&node, "inputMode", &JsValue::from_str("decimal"));
        set_property(&node, "dir", &JsValue::from_str("ltr"));
        let normalized =
            crate::settings_contract::settings_to_western_digits(property(&node, "value"));
        set_property(&node, "value", &JsValue::from_str(&normalized));

        let data = dataset(&node);
        if text(&property(&data, "westernDigitBound")) == "true" {
            continue;
        }
        set_property(&data, "westernDigitBound", &JsValue::from_str("true"));
        let node_for_input = node.clone();
        let input = Closure::wrap(Box::new(move |_event: JsValue| {
            let old = text(&property(&node_for_input, "value"));
            let normalized =
                crate::settings_contract::settings_to_western_digits(JsValue::from_str(&old));
            if old != normalized {
                let pos = property(&node_for_input, "selectionStart");
                set_property(&node_for_input, "value", &JsValue::from_str(&normalized));
                if let Some(set_range) = function(&node_for_input, "setSelectionRange") {
                    let _ = set_range.call2(&node_for_input, &pos, &pos);
                }
            }
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&node, "addEventListener") {
            let _ = add.call2(
                &node,
                &JsValue::from_str("input"),
                input.as_ref().unchecked_ref(),
            );
        }
        input.forget();
    }
}

fn update_select_all(master_selector: &str, child_selector: &str) {
    let master = q(master_selector);
    let children = qa(child_selector);
    if !is_present(&master) || children.is_empty() {
        return;
    }
    let all = children
        .iter()
        .all(|node| crate::js_boolean(&property(node, "checked")));
    let any = children
        .iter()
        .any(|node| crate::js_boolean(&property(node, "checked")));
    set_property(&master, "checked", &JsValue::from_bool(all));
    set_property(&master, "indeterminate", &JsValue::from_bool(!all && any));
}

fn bind_select_all(master_selector: &'static str, child_selector: &'static str) {
    let master = q(master_selector);
    let children = qa(child_selector);
    if !is_present(&master) {
        return;
    }

    let master_data = dataset(&master);
    if text(&property(&master_data, "bound")) != "true" {
        set_property(&master_data, "bound", &JsValue::from_str("true"));
        let master_for_change = master.clone();
        let children_for_change = children.clone();
        let change = Closure::wrap(Box::new(move |event: JsValue| {
            let checked = crate::js_boolean(&property(&master_for_change, "checked"));
            for node in &children_for_change {
                set_property(node, "checked", &JsValue::from_bool(checked));
            }
            set_property(&master_for_change, "indeterminate", &JsValue::FALSE);
            let details = details_pairs(&[
                ("masterSelector", master_selector.to_owned()),
                ("childSelector", child_selector.to_owned()),
                ("checked", checked.to_string()),
                ("trusted", event_trusted(&event).to_string()),
            ]);
            trace(
                "settings-select-all",
                "r48b3-select-all-master-change",
                &details,
            );
            mark_dirty();
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&master, "addEventListener") {
            let _ = add.call2(
                &master,
                &JsValue::from_str("change"),
                change.as_ref().unchecked_ref(),
            );
        }
        change.forget();
    }

    for node in children {
        let data = dataset(&node);
        if text(&property(&data, "bound")) == "true" {
            continue;
        }
        set_property(&data, "bound", &JsValue::from_str("true"));
        let node_for_change = node.clone();
        let change = Closure::wrap(Box::new(move |event: JsValue| {
            update_select_all(master_selector, child_selector);
            let node_data = dataset(&node_for_change);
            let value = [
                text(&property(&node_data, "settingsLanguage")),
                text(&property(&node_data, "settingsCurrency")),
                text(&property(&node_data, "settingsVisibleTab")),
            ]
            .into_iter()
            .find(|value| !value.is_empty())
            .unwrap_or_default();
            let details = details_pairs(&[
                ("masterSelector", master_selector.to_owned()),
                ("childSelector", child_selector.to_owned()),
                ("targetId", text(&property(&node_for_change, "id"))),
                ("targetValue", value),
                ("trusted", event_trusted(&event).to_string()),
            ]);
            trace(
                "settings-select-all",
                "r48b3-select-all-child-change",
                &details,
            );
            mark_dirty();
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&node, "addEventListener") {
            let _ = add.call2(
                &node,
                &JsValue::from_str("change"),
                change.as_ref().unchecked_ref(),
            );
        }
        change.forget();
    }
    update_select_all(master_selector, child_selector);
}

fn bind_navigation() {
    for button in qa("[data-settings-tab]") {
        let data = dataset(&button);
        if text(&property(&data, "bound")) == "true" {
            continue;
        }
        set_property(&data, "bound", &JsValue::from_str("true"));
        let button_for_click = button.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let data = dataset(&button_for_click);
            let selected = text(&property(&data, "settingsTab"));
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("selected", selected.clone()),
                (
                    "text",
                    text(&property(&button_for_click, "textContent"))
                        .trim()
                        .to_owned(),
                ),
            ]);
            trace("settings-navigation", "r48b3-settings-tab-click", &details);
            crate::settings_state::activate_outer(selected);
            mark_dirty();
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&button, "addEventListener") {
            let _ = add.call2(
                &button,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }

    for button in qa("[data-settings-inner-tab]") {
        let data = dataset(&button);
        if text(&property(&data, "bound")) == "true" {
            continue;
        }
        set_property(&data, "bound", &JsValue::from_str("true"));
        let button_for_click = button.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let data = dataset(&button_for_click);
            let selected = text(&property(&data, "settingsInnerTab"));
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("selected", selected.clone()),
                (
                    "text",
                    text(&property(&button_for_click, "textContent"))
                        .trim()
                        .to_owned(),
                ),
            ]);
            trace(
                "settings-navigation",
                "r48b3-settings-inner-tab-click",
                &details,
            );
            crate::settings_state::activate_inner(selected);
            mark_dirty();
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&button, "addEventListener") {
            let _ = add.call2(
                &button,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }
}

fn bind_inputs() {
    for node in qa("input, select") {
        let data = dataset(&node);
        if text(&property(&data, "changeBound")) == "true" {
            continue;
        }
        set_property(&data, "changeBound", &JsValue::from_str("true"));

        for (event_name, phase) in [
            ("input", "r48b3-settings-input"),
            ("change", "r48b3-settings-change"),
        ] {
            let node_for_event = node.clone();
            let callback = Closure::wrap(Box::new(move |event: JsValue| {
                let value = if text(&property(&node_for_event, "type")) == "checkbox" {
                    crate::js_boolean(&property(&node_for_event, "checked")).to_string()
                } else {
                    text(&property(&node_for_event, "value"))
                };
                let details = details_pairs(&[
                    ("trusted", event_trusted(&event).to_string()),
                    ("targetId", text(&property(&node_for_event, "id"))),
                    ("targetName", text(&property(&node_for_event, "name"))),
                    ("targetTag", text(&property(&node_for_event, "tagName"))),
                    ("value", value),
                ]);
                trace("settings-choice", phase, &details);
                crate::settings_state::state_combine_url();
                mark_dirty();
            }) as Box<dyn FnMut(JsValue)>);
            if let Some(add) = function(&node, "addEventListener") {
                let _ = add.call2(
                    &node,
                    &JsValue::from_str(event_name),
                    callback.as_ref().unchecked_ref(),
                );
            }
            callback.forget();
        }
    }
}

fn bind_endpoint_rows() {
    for row in qa(".tree-row") {
        let data = dataset(&row);
        if text(&property(&data, "bound")) == "true" {
            continue;
        }
        set_property(&data, "bound", &JsValue::from_str("true"));
        let row_for_click = row.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                (
                    "apiKey",
                    text(&property(&dataset(&row_for_click), "apiKey")),
                ),
                (
                    "text",
                    text(&property(&row_for_click, "textContent"))
                        .trim()
                        .to_owned(),
                ),
            ]);
            trace("settings-endpoint", "r48b3-endpoint-row-click", &details);
            crate::settings_profiles::select_endpoint(row_for_click.clone());
            mark_dirty();
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&row, "addEventListener") {
            let _ = add.call2(
                &row,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }
}

fn bind_field_actions() {
    for button in qa("[data-clear-for]") {
        let data = dataset(&button);
        if text(&property(&data, "bound")) == "true" {
            continue;
        }
        set_property(&data, "bound", &JsValue::from_str("true"));
        let button_for_click = button.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let target_id = text(&property(&dataset(&button_for_click), "clearFor"));
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("target", target_id.clone()),
                (
                    "text",
                    text(&property(&button_for_click, "textContent"))
                        .trim()
                        .to_owned(),
                ),
            ]);
            trace("settings-field-action", "r48b3-clear-for-click", &details);
            let target = q(&format!("#{}", css_escape(&target_id)));
            if is_present(&target) {
                set_property(&target, "value", &JsValue::from_str(""));
            }
            mark_dirty();
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&button, "addEventListener") {
            let _ = add.call2(
                &button,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }

    for button in qa("[data-browse-for]") {
        let data = dataset(&button);
        if text(&property(&data, "bound")) == "true" {
            continue;
        }
        set_property(&data, "bound", &JsValue::from_str("true"));
        let button_for_click = button.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            if let Some(prevent) = function(&event, "preventDefault") {
                let _ = prevent.call0(&event);
            }
            let target_id = text(&property(&dataset(&button_for_click), "browseFor"));
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("target", target_id.clone()),
                (
                    "text",
                    text(&property(&button_for_click, "textContent"))
                        .trim()
                        .to_owned(),
                ),
            ]);
            trace("settings-field-action", "r48b3-browse-for-click", &details);
            spawn_local(async move {
                let _ = crate::settings_paths::browse(target_id).await;
            });
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&button, "addEventListener") {
            let _ = add.call2(
                &button,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }
}

fn bind_save_reset() {
    let reset = q("#settingsResetDefaults");
    if is_present(&reset) && text(&property(&dataset(&reset), "bound")) != "true" {
        set_property(&dataset(&reset), "bound", &JsValue::from_str("true"));
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("targetId", "settingsResetDefaults".to_owned()),
            ]);
            trace("settings-action", "r48b3-reset-defaults-click", &details);
            spawn_local(async move {
                crate::settings_persistence::reset_defaults(Object::new().into()).await;
            });
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&reset, "addEventListener") {
            let _ = add.call2(
                &reset,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }

    let save = q("#settingsSaveSettings");
    if is_present(&save) && text(&property(&dataset(&save), "bound")) != "true" {
        let data = dataset(&save);
        set_property(&data, "bound", &JsValue::from_str("true"));
        set_property(&data, "kgwSettingsSaveOwner", &JsValue::from_str("R75"));
        let save_for_click = save.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            if let Some(prevent) = function(&event, "preventDefault") {
                let _ = prevent.call0(&event);
            }
            let disabled = crate::js_boolean(&property(&save_for_click, "disabled"));
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("targetId", "settingsSaveSettings".to_owned()),
                ("disabled", disabled.to_string()),
            ]);
            trace("settings-action", "r75-save-press", &details);
            if disabled {
                trace(
                    "settings-action",
                    "r75-save-skip-disabled",
                    &details_pairs(&[("targetId", "settingsSaveSettings".to_owned())]),
                );
                return;
            }
            spawn_local(async move {
                let _ = crate::settings_persistence::save().await;
            });
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&save, "addEventListener") {
            let _ = add.call2(
                &save,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }
}

fn locale_time() -> String {
    let date = Date::new_0();
    let options = Object::new();
    set_property(options.as_ref(), "hour12", &JsValue::FALSE);
    function(date.as_ref(), "toLocaleTimeString")
        .and_then(|call| {
            call.call2(date.as_ref(), &JsValue::from_str("en-US"), options.as_ref())
                .ok()
        })
        .map(|value| text(&value))
        .unwrap_or_default()
}

fn bind_placeholder_actions() {
    for id in PLACEHOLDER_ACTIONS {
        let button = q(&format!("#{}", css_escape(id)));
        if !is_present(&button) || text(&property(&dataset(&button), "bound")) == "true" {
            continue;
        }
        set_property(&dataset(&button), "bound", &JsValue::from_str("true"));
        let button_for_click = button.clone();
        let id = *id;
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let details = details_pairs(&[
                ("trusted", event_trusted(&event).to_string()),
                ("actionId", id.to_owned()),
                (
                    "text",
                    text(&property(&button_for_click, "textContent"))
                        .trim()
                        .to_owned(),
                ),
            ]);
            trace(
                "settings-placeholder-action",
                "r48b3-placeholder-action-click",
                &details,
            );
            if id == "settingsAddressClear" {
                for selector in ["#settingsAddressName", "#settingsAddressValue"] {
                    let node = q(selector);
                    if is_present(&node) {
                        set_property(&node, "value", &JsValue::from_str(""));
                    }
                }
            }
            if id == "settingsAddressRefresh" {
                let updated = q("#settingsAddressLastUpdated");
                if is_present(&updated) {
                    set_property(
                        &updated,
                        "textContent",
                        &JsValue::from_str(&format!("Last Updated: {}", locale_time())),
                    );
                }
            }
            mark_dirty();
            let details = Object::new();
            set_property(details.as_ref(), "id", &JsValue::from_str(id));
            logger("log", "settings action", Some(details.as_ref()));
        }) as Box<dyn FnMut(JsValue)>);
        if let Some(add) = function(&button, "addEventListener") {
            let _ = add.call2(
                &button,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
    }
}

fn bind_static_actions() {
    bind_navigation();
    bind_select_all("#settingsLangSelectAll", "[data-settings-language]");
    bind_select_all("#settingsCurrencySelectAll", "[data-settings-currency]");
    bind_select_all("#settingsTabSelectAll", "[data-settings-visible-tab]");
    crate::settings_state::display_bind_minimum_guards();
    bind_inputs();
    bind_endpoint_rows();
    bind_field_actions();
    bind_save_reset();
    bind_placeholder_actions();
}

#[wasm_bindgen(js_name = settingsUiInitTab)]
pub fn init_tab() {
    let node = root();
    if !is_present(&node) {
        logger("warn", "settings root missing", None);
        return;
    }
    let data = dataset(&node);
    if text(&property(&data, "settingsPythonInitialized")) == "true" {
        return;
    }
    set_property(
        &data,
        "settingsPythonInitialized",
        &JsValue::from_str("true"),
    );

    bind_static_actions();

    let options = Object::new();
    set_property(options.as_ref(), "clearStorage", &JsValue::FALSE);
    set_property(options.as_ref(), "applyShell", &JsValue::FALSE);
    crate::settings_persistence::reset_defaults_for_init(options.into());
    crate::settings_persistence::load_saved();

    let _ = crate::settings_addresses::install_io();
    crate::settings_profiles::install();
    let _ = crate::settings_state::display_ensure_defaults("settings-init".to_owned());
    normalize_numeric_fields();

    spawn_local(async {
        let _ = crate::settings_paths::load_defaults("settings-init".to_owned()).await;
    });

    let first_endpoint = q(".tree-row");
    if is_present(&first_endpoint) && !is_present(&q(".tree-row.is-selected")) {
        crate::settings_profiles::select_endpoint(first_endpoint);
    }

    crate::settings_diagnostics::install();
    set_save_enabled(false);
    logger("log", "settings python exact ui initialized", None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_fields_preserve_existing_contract() {
        assert_eq!(NUMERIC_IDS.len(), 9);
        assert!(NUMERIC_IDS.contains(&"settingsRefreshInterval"));
    }

    #[test]
    fn placeholder_action_set_is_stable() {
        assert_eq!(PLACEHOLDER_ACTIONS.len(), 10);
        assert!(PLACEHOLDER_ACTIONS.contains(&"settingsDbDelete"));
        assert!(PLACEHOLDER_ACTIONS.contains(&"settingsAddressRefresh"));
    }
}
