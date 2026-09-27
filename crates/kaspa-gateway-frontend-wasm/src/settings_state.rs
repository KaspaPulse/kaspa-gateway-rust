use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const STORAGE_KEY: &str = "kgw-settings-python-exact-state";

#[derive(Clone, Copy)]
struct DisplayGroup {
    name: &'static str,
    selector: &'static str,
    attr: &'static str,
    select_all: &'static str,
    fallback: &'static str,
}

const GROUPS: [DisplayGroup; 3] = [
    DisplayGroup {
        name: "languages",
        selector: "[data-settings-language]",
        attr: "data-settings-language",
        select_all: "#settingsLangSelectAll",
        fallback: "en",
    },
    DisplayGroup {
        name: "currencies",
        selector: "[data-settings-currency]",
        attr: "data-settings-currency",
        select_all: "#settingsCurrencySelectAll",
        fallback: "USD",
    },
    DisplayGroup {
        name: "tabs",
        selector: "[data-settings-visible-tab]",
        attr: "data-settings-visible-tab",
        select_all: "#settingsTabSelectAll",
        fallback: "explorer",
    },
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

fn call1(target: &JsValue, name: &str, value: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|call| call.call1(target, value).ok())
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

fn attr(target: &JsValue, name: &str) -> String {
    function(target, "getAttribute")
        .and_then(|call| call.call1(target, &JsValue::from_str(name)).ok())
        .map(|value| text(&value))
        .unwrap_or_default()
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn css_escape(value: &str) -> String {
    let css = property(&global(), "CSS");
    function(&css, "escape")
        .and_then(|call| call.call1(&css, &JsValue::from_str(value)).ok())
        .map(|value| text(&value))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| value.to_owned())
}

fn storage() -> JsValue {
    property(&window(), "localStorage")
}

fn storage_set(value: &JsValue) {
    let Ok(serialized) = js_sys::JSON::stringify(value) else {
        return;
    };
    if let Some(call) = function(&storage(), "setItem") {
        let _ = call.call2(
            &storage(),
            &JsValue::from_str(STORAGE_KEY),
            serialized.as_ref(),
        );
    }
}

fn set_save_enabled(enabled: bool) {
    let save = q("#settingsSaveSettings");
    if is_present(&save) {
        set_property(&save, "disabled", &JsValue::from_bool(!enabled));
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

fn group_nodes(group: DisplayGroup) -> Vec<JsValue> {
    qa(group.selector)
        .into_iter()
        .filter(|node| text(&property(node, "type")) == "checkbox")
        .collect()
}

fn checked_count(group: DisplayGroup) -> usize {
    group_nodes(group)
        .iter()
        .filter(|node| crate::js_boolean(&property(node, "checked")))
        .count()
}

fn set_fallback(group: DisplayGroup) {
    let nodes = group_nodes(group);
    let mut found = false;
    for node in &nodes {
        let value = attr(node, group.attr);
        let checked = value == group.fallback;
        set_property(node, "checked", &JsValue::from_bool(checked));
        found |= checked;
    }
    if !found && let Some(first) = nodes.first() {
        set_property(first, "checked", &JsValue::TRUE);
    }
    update_select_all(group.select_all, group.selector);
}

fn logger_warn(message: &str, groups: &[DisplayGroup]) {
    let factory = property(&window(), "kgwCreateLogger");
    let Ok(factory) = factory.dyn_into::<Function>() else {
        return;
    };
    let Ok(logger) = factory.call1(&window(), &JsValue::from_str("settings")) else {
        return;
    };
    let Some(warn) = function(&logger, "warn") else {
        return;
    };
    let details = Object::new();
    let names = Array::new();
    for group in groups {
        names.push(&JsValue::from_str(group.name));
    }
    set_property(details.as_ref(), "groups", names.as_ref());
    let _ = warn.call2(&logger, &JsValue::from_str(message), details.as_ref());
}

fn ensure_defaults() -> Vec<DisplayGroup> {
    let mut repaired = Vec::new();
    for group in GROUPS {
        let nodes = group_nodes(group);
        if !nodes.is_empty()
            && !nodes
                .iter()
                .any(|node| crate::js_boolean(&property(node, "checked")))
        {
            set_fallback(group);
            repaired.push(group);
        }
    }
    repaired
}

fn validate_for_save() -> bool {
    let zero = GROUPS
        .iter()
        .copied()
        .filter(|group| {
            let nodes = group_nodes(*group);
            !nodes.is_empty()
                && !nodes
                    .iter()
                    .any(|node| crate::js_boolean(&property(node, "checked")))
        })
        .collect::<Vec<_>>();
    if zero.is_empty() {
        return true;
    }
    for group in &zero {
        set_fallback(*group);
    }
    logger_warn(
        "display selection repaired silently to fixed fallback",
        &zero,
    );
    true
}

fn schedule_guard(group: DisplayGroup) {
    let callback = Closure::once_into_js(move || {
        if checked_count(group) == 0 {
            set_fallback(group);
            logger_warn(
                "display selection repaired silently to fixed fallback",
                &[group],
            );
            set_save_enabled(true);
        }
    });
    let timeout = property(&window(), "setTimeout");
    if let Ok(timeout) = timeout.dyn_into::<Function>() {
        let _ = timeout.call2(&window(), &callback, &JsValue::from_f64(0.0));
    }
}

fn bind_minimum_guards() {
    for group in GROUPS {
        let master = q(group.select_all);
        if is_present(&master) {
            let data = dataset(&master);
            if text(&property(&data, "kgwMinimumSelectionBound")) != "true" {
                set_property(
                    &data,
                    "kgwMinimumSelectionBound",
                    &JsValue::from_str("true"),
                );
                let change = Closure::wrap(Box::new(move |_event: JsValue| {
                    schedule_guard(group);
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
        }

        for node in group_nodes(group) {
            let data = dataset(&node);
            if text(&property(&data, "kgwMinimumSelectionBound")) == "true" {
                continue;
            }
            set_property(
                &data,
                "kgwMinimumSelectionBound",
                &JsValue::from_str("true"),
            );
            let change = Closure::wrap(Box::new(move |_event: JsValue| {
                schedule_guard(group);
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
    }
}

fn display_key(node: &JsValue) -> String {
    let data = dataset(node);
    for (name, prefix) in [
        ("settingsLanguage", "language:"),
        ("settingsCurrency", "currency:"),
        ("settingsVisibleTab", "tab:"),
    ] {
        let value = text(&property(&data, name));
        if !value.is_empty() {
            return format!("{prefix}{value}");
        }
    }
    String::new()
}

fn display_nodes() -> Vec<JsValue> {
    qa("[data-settings-language], [data-settings-currency], [data-settings-visible-tab]")
        .into_iter()
        .filter(|node| text(&property(node, "type")) == "checkbox")
        .collect()
}

fn known_entries() -> Array {
    let entries = Array::new();
    for node in display_nodes() {
        let entry = Object::new();
        set_property(
            entry.as_ref(),
            "key",
            &JsValue::from_str(&display_key(&node)),
        );
        set_property(
            entry.as_ref(),
            "id",
            &JsValue::from_str(&text(&property(&node, "id"))),
        );
        entries.push(entry.as_ref());
    }
    entries
}

fn checks_with_defaults(checks: JsValue) -> JsValue {
    crate::settings_contract::settings_display_checks_with_defaults(checks, known_entries())
}

fn selected_keys(checks: JsValue, prefix: &str) -> Array {
    crate::settings_contract::settings_selected_display_keys(checks, prefix.to_owned())
}

fn apply_display_checks(checks: &JsValue) {
    if !checks.is_object() {
        return;
    }
    for node in display_nodes() {
        let key = display_key(&node);
        if key.is_empty() {
            continue;
        }
        let checked = Reflect::get(checks, &JsValue::from_str(&key))
            .ok()
            .is_some_and(|value| value.as_bool() == Some(true));
        set_property(&node, "checked", &JsValue::from_bool(checked));
        let id = text(&property(&node, "id"));
        if !id.is_empty() {
            let _ = Reflect::set(
                checks,
                &JsValue::from_str(&id),
                &JsValue::from_bool(checked),
            );
        }
    }
    for (master, child) in [
        ("#settingsLangSelectAll", "[data-settings-language]"),
        ("#settingsCurrencySelectAll", "[data-settings-currency]"),
        ("#settingsTabSelectAll", "[data-settings-visible-tab]"),
    ] {
        update_select_all(master, child);
    }
}

fn activate(kind: &str, tab: &str) {
    let (button_selector, button_key, panel_selector, panel_key) = if kind == "outer" {
        (
            "[data-settings-tab]",
            "settingsTab",
            "[data-settings-panel]",
            "settingsPanel",
        )
    } else {
        (
            "[data-settings-inner-tab]",
            "settingsInnerTab",
            "[data-settings-inner-panel]",
            "settingsInnerPanel",
        )
    };
    for button in qa(button_selector) {
        let active = text(&property(&dataset(&button), button_key)) == tab;
        let class_list = property(&button, "classList");
        if let Some(toggle) = function(&class_list, "toggle") {
            let _ = toggle.call2(
                &class_list,
                &JsValue::from_str("active"),
                &JsValue::from_bool(active),
            );
        }
        if let Some(set_attr) = function(&button, "setAttribute") {
            let _ = set_attr.call2(
                &button,
                &JsValue::from_str("aria-selected"),
                &JsValue::from_str(if active { "true" } else { "false" }),
            );
        }
    }
    for panel in qa(panel_selector) {
        let active = text(&property(&dataset(&panel), panel_key)) == tab;
        let class_list = property(&panel, "classList");
        if let Some(toggle) = function(&class_list, "toggle") {
            let _ = toggle.call2(
                &class_list,
                &JsValue::from_str("active"),
                &JsValue::from_bool(active),
            );
        }
    }
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

fn active_value(selector: &str, key: &str, fallback: &str) -> String {
    qa(selector)
        .first()
        .map(|node| text(&property(&dataset(node), key)))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_owned())
}

fn collect_state() -> JsValue {
    let state = Object::new();
    let inputs = Object::new();
    let checks = Object::new();
    for node in qa("input, select") {
        let id = text(&property(&node, "id"));
        if id.is_empty() {
            continue;
        }
        if text(&property(&node, "type")) == "checkbox" {
            set_property(
                checks.as_ref(),
                &id,
                &JsValue::from_bool(crate::js_boolean(&property(&node, "checked"))),
            );
        } else {
            set_property(inputs.as_ref(), &id, &property(&node, "value"));
        }
    }
    for (index, node) in display_nodes().into_iter().enumerate() {
        let key = {
            let key = display_key(&node);
            if key.is_empty() {
                format!("check:{index}")
            } else {
                key
            }
        };
        set_property(
            checks.as_ref(),
            &key,
            &JsValue::from_bool(crate::js_boolean(&property(&node, "checked"))),
        );
    }
    set_property(state.as_ref(), "inputs", inputs.as_ref());
    set_property(state.as_ref(), "checks", checks.as_ref());
    set_property(
        state.as_ref(),
        "activeOuter",
        &JsValue::from_str(&active_value(
            "[data-settings-tab].active",
            "settingsTab",
            "api-performance",
        )),
    );
    set_property(
        state.as_ref(),
        "activeInner",
        &JsValue::from_str(&active_value(
            "[data-settings-inner-tab].active",
            "settingsInnerTab",
            "general",
        )),
    );
    state.into()
}

fn apply_state(state: &JsValue) {
    if !state.is_object() {
        return;
    }
    let inputs = property(state, "inputs");
    if inputs.is_object() {
        for key in Object::keys(inputs.unchecked_ref::<Object>()).iter() {
            let id = text(&key);
            let node = q(&format!("#{}", css_escape(&id)));
            if is_present(&node)
                && let Ok(value) = Reflect::get(&inputs, &key)
            {
                set_property(&node, "value", &value);
            }
        }
    }
    let checks = property(state, "checks");
    if checks.is_object() {
        for key in Object::keys(checks.unchecked_ref::<Object>()).iter() {
            let id = text(&key);
            let mut node = q(&format!("#{}", css_escape(&id)));
            if !is_present(&node) {
                for (prefix, attr_name) in [
                    ("language:", "data-settings-language"),
                    ("currency:", "data-settings-currency"),
                    ("tab:", "data-settings-visible-tab"),
                ] {
                    if let Some(value) = id.strip_prefix(prefix) {
                        node = q(&format!("[{attr_name}=\"{}\"]", css_escape(value)));
                        break;
                    }
                }
            }
            if is_present(&node) && text(&property(&node, "type")) == "checkbox" {
                let checked = Reflect::get(&checks, &key)
                    .ok()
                    .is_some_and(|value| value.as_bool() == Some(true));
                set_property(&node, "checked", &JsValue::from_bool(checked));
            }
        }
    }
    for (master, child) in [
        ("#settingsLangSelectAll", "[data-settings-language]"),
        ("#settingsCurrencySelectAll", "[data-settings-currency]"),
        ("#settingsTabSelectAll", "[data-settings-visible-tab]"),
    ] {
        update_select_all(master, child);
    }
    let outer = text(&property(state, "activeOuter"));
    let inner = text(&property(state, "activeInner"));
    if !outer.is_empty() {
        activate("outer", &outer);
    }
    if !inner.is_empty() {
        activate("inner", &inner);
    }
    combine_url();
}

fn apply_shell(state: &JsValue, reason: &str) -> JsValue {
    let checks = property(state, "checks");
    let prefs = crate::settings_contract::settings_display_preferences(checks);
    if !is_present(&prefs) {
        return JsValue::NULL;
    }
    let owner = property(&window(), "kgwShellApplyDisplayPreferencesDirectR73");
    if let Ok(owner) = owner.dyn_into::<Function>() {
        let _ = owner.call2(&window(), &prefs, &JsValue::from_str(reason));
    }
    prefs
}

fn build_canonical(reason: &str, persist: bool) -> JsValue {
    let base = collect_state();
    let checks = checks_with_defaults(property(&base, "checks"));
    set_property(&base, "checks", &checks);
    apply_display_checks(&checks);

    let state = collect_state();
    let checks = checks_with_defaults(property(&state, "checks"));
    set_property(&state, "checks", &checks);
    apply_display_checks(&checks);

    let final_state = collect_state();
    let checks = checks_with_defaults(property(&final_state, "checks"));
    set_property(&final_state, "checks", &checks);
    if persist {
        storage_set(&final_state);
    }
    let _ = apply_shell(&final_state, reason);
    final_state
}

fn reapply_state(state: &JsValue, reason: &str) -> JsValue {
    let checks = property(state, "checks");
    if !checks.is_object() {
        return build_canonical(&format!("{reason}-fallback"), true);
    }
    apply_display_checks(&checks);
    collect_state()
}

#[wasm_bindgen(js_name = settingsStateCollect)]
pub fn state_collect() -> JsValue {
    collect_state()
}

#[wasm_bindgen(js_name = settingsStateApply)]
pub fn state_apply(state: JsValue) {
    apply_state(&state);
}

#[wasm_bindgen(js_name = settingsStateActivateOuter)]
pub fn activate_outer(tab: String) {
    activate("outer", &tab);
}

#[wasm_bindgen(js_name = settingsStateActivateInner)]
pub fn activate_inner(tab: String) {
    activate("inner", &tab);
}

#[wasm_bindgen(js_name = settingsStateCombineUrl)]
pub fn state_combine_url() {
    combine_url();
}

#[wasm_bindgen(js_name = settingsDisplayEnsureDefaults)]
pub fn display_ensure_defaults(_reason: String) -> Array {
    let repaired = ensure_defaults();
    repaired
        .into_iter()
        .map(|group| JsValue::from_str(group.name))
        .collect()
}

#[wasm_bindgen(js_name = settingsDisplayValidateForSave)]
pub fn display_validate_for_save() -> bool {
    validate_for_save()
}

#[wasm_bindgen(js_name = settingsDisplayBindMinimumGuards)]
pub fn display_bind_minimum_guards() {
    bind_minimum_guards();
}

#[wasm_bindgen(js_name = settingsDisplayBuildCanonicalDefaultState)]
pub fn display_build_canonical_default_state(reason: String, persist: bool) -> JsValue {
    build_canonical(&reason, persist)
}

#[wasm_bindgen(js_name = settingsDisplayReapplyState)]
pub fn display_reapply_state(state: JsValue, reason: String) -> JsValue {
    reapply_state(&state, &reason)
}

#[wasm_bindgen(js_name = settingsDisplayStateLooksLegacyAllSelected)]
pub fn display_state_looks_legacy(state: JsValue) -> bool {
    crate::settings_contract::settings_display_state_missing_contract(property(&state, "checks"))
}

#[wasm_bindgen(js_name = settingsDisplayApplyShellFromState)]
pub fn display_apply_shell_from_state(state: JsValue, reason: String) -> JsValue {
    apply_shell(&state, &reason)
}

#[wasm_bindgen(js_name = settingsDisplaySelectedKeys)]
pub fn display_selected_keys(checks: JsValue, prefix: String) -> Array {
    selected_keys(checks, &prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_groups_preserve_fixed_fallbacks() {
        assert_eq!(GROUPS[0].fallback, "en");
        assert_eq!(GROUPS[1].fallback, "USD");
        assert_eq!(GROUPS[2].fallback, "explorer");
    }

    #[test]
    fn storage_key_preserves_existing_contract() {
        assert_eq!(STORAGE_KEY, "kgw-settings-python-exact-state");
    }
}
