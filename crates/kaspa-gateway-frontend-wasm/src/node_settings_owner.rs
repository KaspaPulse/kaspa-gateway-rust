use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const OWNER: &str = "KGW_SETTINGS_OWNER_V19";
const PATCH: &str = "KGW_SETTINGS_OWNER_V19_SAFE_FEEDBACK_NO_FREEZE_V25B";
const SCOPE: &str = "node";
const FEEDBACK_MS: i32 = 3000;
const DISABLED_CLASS: &str = "kgw-settings-action-disabled-v19";
const ROOT_INSTALLED_ATTR: &str = "kgwSettingsOwnerV19";

#[derive(Clone)]
struct Feedback {
    timer: f64,
    button: JsValue,
}

thread_local! {
    static DIRTY: RefCell<BTreeMap<String, bool>> = const { RefCell::new(BTreeMap::new()) };
    static FEEDBACK: RefCell<BTreeMap<String, Feedback>> = const { RefCell::new(BTreeMap::new()) };
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

fn call0(target: &JsValue, name: &str) -> Option<JsValue> {
    function(target, name)?.call0(target).ok()
}

fn call1(target: &JsValue, name: &str, first: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, first).ok()
}

fn call2(target: &JsValue, name: &str, first: &JsValue, second: &JsValue) -> Option<JsValue> {
    function(target, name)?.call2(target, first, second).ok()
}

fn call3(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
    third: &JsValue,
) -> Option<JsValue> {
    function(target, name)?
        .call3(target, first, second, third)
        .ok()
}

fn text(value: &JsValue) -> String {
    if present(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    }
}

fn lower(value: &JsValue) -> String {
    text(value).to_lowercase()
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn attribute(target: &JsValue, name: &str) -> String {
    call1(target, "getAttribute", &JsValue::from_str(name))
        .map(|value| text(&value))
        .unwrap_or_default()
}

fn set_attribute(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(target, "querySelectorAll", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED);
    let length = property(&list, "length").as_f64().unwrap_or(0.0).max(0.0) as u32;
    (0..length)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(f64::from(index))).ok())
        .filter(present)
        .collect()
}

fn stop_immediate(event: &JsValue) {
    let _ = call0(event, "stopImmediatePropagation");
}

fn prevent_default(event: &JsValue) {
    let _ = call0(event, "preventDefault");
}

fn bool_property(target: &JsValue, name: &str) -> bool {
    crate::js_boolean(&property(target, name))
}

fn set_timeout(callback: &Function, ms: i32) -> Option<f64> {
    function(&window(), "setTimeout")?
        .call2(
            &window(),
            callback.as_ref(),
            &JsValue::from_f64(f64::from(ms)),
        )
        .ok()?
        .as_f64()
}

fn clear_timeout(timer: f64) {
    let _ = call1(&window(), "clearTimeout", &JsValue::from_f64(timer));
}

fn trace_details_text(phase: &str, details: &JsValue) -> String {
    let nested = Object::new();
    set(nested.as_ref(), "owner", &JsValue::from_str(OWNER));
    set(nested.as_ref(), "patch", &JsValue::from_str(PATCH));
    set(nested.as_ref(), "scope", &JsValue::from_str(SCOPE));
    set(nested.as_ref(), "phase", &JsValue::from_str(phase));
    set(nested.as_ref(), "details", details);
    JSON::stringify(nested.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned())
}

fn trace(phase: &str, details: JsValue) {
    let net = {
        let network = property(&details, "network");
        if crate::js_boolean(&network) {
            text(&network)
        } else {
            let value = property(&details, "net");
            if crate::js_boolean(&value) {
                text(&value)
            } else {
                "unknown".to_owned()
            }
        }
    };
    let action = {
        let value = property(&details, "action");
        if crate::js_boolean(&value) {
            text(&value)
        } else {
            "settings-owner".to_owned()
        }
    };
    let args = Object::new();
    set(args.as_ref(), "scope", &JsValue::from_str(SCOPE));
    set(args.as_ref(), "net", &JsValue::from_str(&net));
    set(args.as_ref(), "action", &JsValue::from_str(&action));
    set(
        args.as_ref(),
        "phase",
        &JsValue::from_str(if phase.is_empty() { "unknown" } else { phase }),
    );
    set(
        args.as_ref(),
        "details",
        &JsValue::from_str(&trace_details_text(phase, &details)),
    );

    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let (owner, invoke) = if function(&core, "invoke").is_some() {
        (core, property(&property(&tauri, "core"), "invoke"))
    } else if function(&tauri, "invoke").is_some() {
        (tauri.clone(), property(&tauri, "invoke"))
    } else {
        let console = property(&global(), "console");
        let _ = call2(
            &console,
            "debug",
            &JsValue::from_str("[KGW_SETTINGS_OWNER_V19_TRACE_BROWSER]"),
            args.as_ref(),
        );
        return;
    };
    let Ok(invoke) = invoke.dyn_into::<Function>() else {
        return;
    };
    if let Ok(result) = invoke.call2(
        &owner,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }
}

fn trace_dataset(target: &JsValue) -> JsValue {
    let source = dataset(target);
    let output = Object::new();
    let prefixes = [
        "net",
        "network",
        "nodeaction",
        "bridgeaction",
        "nodecommandoptiontoggler7",
        "bridgecommandoptiontoggler7",
        "bridgeinstancecommandoptiontoggler13b",
        "instanceid",
        "bridgeinstancefield",
        "kgw",
    ];
    if source.is_object() && !source.is_null() {
        for key in Object::keys(&Object::from(source.clone())).iter() {
            let name = text(&key);
            if prefixes
                .iter()
                .any(|prefix| name.to_lowercase().starts_with(prefix))
            {
                let value: String = text(&property(&source, &name)).chars().take(160).collect();
                set(output.as_ref(), &name, &JsValue::from_str(&value));
            }
        }
    }
    output.into()
}

fn target_snapshot(target: &JsValue) -> JsValue {
    let output = Object::new();
    for (key, name) in [
        ("tag", "tagName"),
        ("id", "id"),
        ("name", "name"),
        ("type", "type"),
    ] {
        set(
            output.as_ref(),
            key,
            &JsValue::from_str(&text(&property(target, name))),
        );
    }
    let class_name: String = text(&property(target, "className"))
        .chars()
        .take(220)
        .collect();
    set(
        output.as_ref(),
        "className",
        &JsValue::from_str(&class_name),
    );
    set(output.as_ref(), "dataset", &trace_dataset(target));

    let kind = lower(&property(target, "type"));
    if kind == "checkbox" || kind == "radio" {
        set(
            output.as_ref(),
            "checked",
            &JsValue::from_bool(bool_property(target, "checked")),
        );
    } else {
        let value = property(target, "value");
        if present(&value) {
            let value = text(&value);
            set(
                output.as_ref(),
                "valueLength",
                &JsValue::from_f64(value.chars().count() as f64),
            );
            let preview: String = value.chars().take(180).collect();
            set(
                output.as_ref(),
                "valuePreview",
                &JsValue::from_str(&preview),
            );
        }
    }
    output.into()
}

fn preview_snapshot(root: &JsValue, network: &str) -> JsValue {
    let output = Object::new();
    let candidates = query_all(
        root,
        "textarea, input, pre, code, [id*='commandPreview'], [data-kgw-command-preview]",
    );
    let mut selected = candidates
        .iter()
        .find(|item| {
            let id = text(&property(item, "id"));
            id.contains(network) && id.to_lowercase().contains("commandpreview")
        })
        .cloned();
    if selected.is_none() {
        selected = candidates
            .iter()
            .find(|item| {
                text(&property(item, "id"))
                    .to_lowercase()
                    .contains("commandpreview")
            })
            .cloned();
    }
    let Some(selected) = selected else {
        set(output.as_ref(), "found", &JsValue::FALSE);
        return output.into();
    };
    let raw = {
        let value = property(&selected, "value");
        if present(&value) {
            value
        } else {
            property(&selected, "textContent")
        }
    };
    let raw = text(&raw);
    set(output.as_ref(), "found", &JsValue::TRUE);
    set(
        output.as_ref(),
        "id",
        &JsValue::from_str(&text(&property(&selected, "id"))),
    );
    set(
        output.as_ref(),
        "length",
        &JsValue::from_f64(raw.chars().count() as f64),
    );
    let preview: String = raw.chars().take(260).collect();
    set(output.as_ref(), "preview", &JsValue::from_str(&preview));
    output.into()
}

fn event_trace_details(root: &JsValue, event: &JsValue, network: &str, reason: &str) -> JsValue {
    let output = Object::new();
    for (key, value) in [
        ("patch", "R29B"),
        ("owner", OWNER),
        ("scope", SCOPE),
        ("tab", "node"),
        ("reason", reason),
        ("network", network),
    ] {
        set(output.as_ref(), key, &JsValue::from_str(value));
    }
    set(
        output.as_ref(),
        "eventType",
        &JsValue::from_str(&text(&property(event, "type"))),
    );
    set(
        output.as_ref(),
        "trusted",
        &JsValue::from_bool(bool_property(event, "isTrusted")),
    );
    set(
        output.as_ref(),
        "target",
        &target_snapshot(&property(event, "target")),
    );
    set(output.as_ref(), "preview", &preview_snapshot(root, network));
    output.into()
}

fn button_trace_details(
    root: &JsValue,
    event: &JsValue,
    button: &JsValue,
    network: &str,
    action: &str,
    disabled: bool,
) -> JsValue {
    let output = Object::from(event_trace_details(
        root,
        event,
        network,
        "settings-action-button",
    ));
    set(output.as_ref(), "action", &JsValue::from_str(action));
    set(output.as_ref(), "button", &target_snapshot(button));
    set(output.as_ref(), "disabled", &JsValue::from_bool(disabled));
    let label: String = text(&property(button, "textContent"))
        .trim()
        .chars()
        .take(160)
        .collect();
    set(output.as_ref(), "label", &JsValue::from_str(&label));
    output.into()
}

fn current_language() -> String {
    let doc = document();
    for node in [property(&doc, "documentElement"), property(&doc, "body")] {
        let lang = attribute(&node, "lang");
        if !lang.is_empty() {
            return lang.to_lowercase();
        }
    }
    let storage = property(&window(), "localStorage");
    for key in [
        "kgw.language",
        "kgw_locale",
        "language",
        "locale",
        "i18nextLng",
    ] {
        let value = call1(&storage, "getItem", &JsValue::from_str(key))
            .map(|item| text(&item))
            .unwrap_or_default();
        if !value.is_empty() {
            return value.to_lowercase();
        }
    }
    for name in ["kgwI18n", "KGWI18n", "KGW_I18N", "i18n"] {
        let api = property(&window(), name);
        if !present(&api) {
            continue;
        }
        for key in [
            "language",
            "lang",
            "locale",
            "currentLanguage",
            "currentLocale",
        ] {
            let value = text(&property(&api, key));
            if !value.is_empty() {
                return value.to_lowercase();
            }
        }
        for method in ["getLanguage", "getLocale"] {
            if let Some(value) = call0(&api, method) {
                let value = text(&value);
                if !value.is_empty() {
                    return value.to_lowercase();
                }
            }
        }
    }
    String::new()
}

fn is_arabic() -> bool {
    current_language().starts_with("ar") || lower(&property(&document(), "dir")) == "rtl"
}

fn translate(key: &str, fallback: &str) -> String {
    for name in ["kgwT", "__kgwT", "t"] {
        let candidate = property(&window(), name);
        if let Ok(candidate) = candidate.dyn_into::<Function>()
            && let Ok(value) = candidate.call2(
                &window(),
                &JsValue::from_str(key),
                &JsValue::from_str(fallback),
            )
        {
            let value = text(&value);
            if !value.trim().is_empty() && value != key {
                return value;
            }
        }
    }
    for name in ["kgwI18n", "KGWI18n", "KGW_I18N", "i18n"] {
        let api = property(&window(), name);
        for method in ["t", "translate"] {
            if let Some(candidate) = function(&api, method)
                && let Ok(value) =
                    candidate.call2(&api, &JsValue::from_str(key), &JsValue::from_str(fallback))
            {
                let value = text(&value);
                if !value.trim().is_empty() && value != key {
                    return value;
                }
            }
        }
    }
    fallback.to_owned()
}

fn is_settings_control(element: &JsValue) -> bool {
    if !present(element) {
        return false;
    }
    let tag = lower(&property(element, "tagName"));
    if !matches!(tag.as_str(), "input" | "select" | "textarea") {
        return false;
    }
    let kind = lower(&property(element, "type"));
    if matches!(kind.as_str(), "button" | "submit" | "reset" | "hidden") {
        return false;
    }
    call1(
        element,
        "closest",
        &JsValue::from_str(".logs, .log, [data-log], .kgw-log-pane"),
    )
    .is_none_or(|value| !present(&value))
}
fn is_action_button(element: &JsValue) -> bool {
    if lower(&property(element, "tagName")) != "button" {
        return false;
    }
    let ds = dataset(element);
    let mut action = text(&property(&ds, "kgwSettingsAction"));
    if action.is_empty() {
        action = text(&property(&ds, "action"));
    }
    if action.is_empty() {
        action = attribute(element, "data-action");
    }
    if action.is_empty() {
        action = attribute(element, "aria-label");
    }
    let action = action.to_lowercase();
    let label = lower(&property(element, "textContent"));
    ["save", "restore", "default"]
        .iter()
        .any(|term| action.contains(term))
        || [
            "save settings",
            "restore defaults",
            "set as defaults",
            "saved",
            "restored",
        ]
        .iter()
        .any(|term| label.contains(term))
}
fn network_of(element: &JsValue) -> String {
    let doc = document();
    let mut current = element.clone();
    while present(&current) && !Object::is(&current, &doc) {
        let ds = dataset(&current);
        for name in ["network", "net", "kgwNetwork"] {
            let value = text(&property(&ds, name));
            if !value.is_empty() {
                return value;
            }
        }
        for name in ["data-network", "data-net", "data-kgw-network"] {
            let value = attribute(&current, name);
            if !value.is_empty() {
                return value;
            }
        }
        let combined = format!(
            "{} {}",
            lower(&property(&current, "id")),
            lower(&property(&current, "className"))
        );
        if combined.contains("testnet13") || combined.contains("tn13") {
            return "testnet13".to_owned();
        }
        if combined.contains("testnet10") || combined.contains("tn10") {
            return "testnet10".to_owned();
        }
        if combined.contains("mainnet") {
            return "mainnet".to_owned();
        }
        current = property(&current, "parentElement");
    }
    "mainnet".to_owned()
}
fn action_name(button: &JsValue) -> &'static str {
    let ds = dataset(button);
    let mut raw = text(&property(&ds, "kgwSettingsAction"));
    if raw.is_empty() {
        raw = text(&property(&ds, "action"));
    }
    if raw.is_empty() {
        raw = text(&property(&ds, "kgwSettingsOwnerV19Action"));
    }
    if raw.is_empty() {
        raw = attribute(button, "data-action");
    }
    if raw.is_empty() {
        raw = attribute(button, "aria-label");
    }
    if raw.is_empty() {
        raw = text(&property(button, "textContent"));
    }
    let raw = raw.to_lowercase();
    if raw.contains("restore") {
        "restore"
    } else if raw.contains("default") {
        "defaults"
    } else {
        "save"
    }
}

fn fallback_text(action: &str) -> &'static str {
    match action {
        "restore" => "Restore Defaults",
        "defaults" => "Set as Defaults",
        _ => "Save Settings",
    }
}

fn feedback_text(action: &str) -> String {
    if is_arabic() {
        return match action {
            "restore" => "تمت الاستعادة",
            "defaults" => "تم الضبط",
            _ => "تم الحفظ",
        }
        .to_owned();
    }
    match action {
        "restore" => translate("settings.feedback.restored", "Restored"),
        "defaults" => translate("settings.feedback.setAsDefaults", "Set"),
        _ => translate("settings.feedback.saved", "Saved"),
    }
}
fn is_feedback_label_text(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "saved" | "restored" | "set" | "set as defaults" | "تم الحفظ" | "تم الضبط" | "تمت الاستعادة"
    )
}

fn all_buttons(root: &JsValue) -> Vec<JsValue> {
    query_all(root, "button")
        .into_iter()
        .filter(is_action_button)
        .collect()
}

fn buttons(root: &JsValue, network: &str) -> Vec<JsValue> {
    all_buttons(root)
        .into_iter()
        .filter(|button| network.is_empty() || network == "all" || network_of(button) == network)
        .collect()
}

fn canonical(value: &JsValue) -> JsValue {
    if Array::is_array(value) {
        let output = Array::new();
        for item in Array::from(value).iter() {
            output.push(&canonical(&item));
        }
        return output.into();
    }
    if value.is_object() && !value.is_null() {
        let source = Object::from(value.clone());
        let mut keys = Object::keys(&source)
            .iter()
            .map(|item| text(&item))
            .collect::<Vec<_>>();
        keys.sort();
        let output = Object::new();
        for key in keys {
            set(output.as_ref(), &key, &canonical(&property(value, &key)));
        }
        return output.into();
    }
    value.clone()
}
fn stable(value: &JsValue) -> String {
    JSON::stringify(&canonical(value))
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_default()
}

fn bridge_locked(callbacks: &JsValue, net: &str) -> bool {
    let callback = property(callbacks, "isLocked");
    let Ok(callback) = callback.dyn_into::<Function>() else {
        return false;
    };
    callback
        .call1(callbacks, &JsValue::from_str(net))
        .ok()
        .is_some_and(|value| crate::js_boolean(&value))
}

fn loaded_or(value: JsValue, fallback: JsValue) -> JsValue {
    if crate::js_boolean(&value) {
        value
    } else {
        fallback
    }
}

fn set_disabled(root: &JsValue, network: &str, reason: &str, callbacks: &JsValue) {
    let networks = if !network.is_empty() && network != "all" {
        vec![network.to_owned()]
    } else {
        super::node_frontend_helpers::node_r51_keys()
            .iter()
            .map(|value| text(&value))
            .collect()
    };

    for net in networks {
        let current = super::node_frontend_helpers::node_r51_read_settings_tracked(net.clone());
        let saved = loaded_or(
            super::node_frontend_helpers::node_r51_load(format!("saved:{net}")),
            loaded_or(
                super::node_frontend_helpers::node_r51_load(format!("factory:{net}")),
                current.clone(),
            ),
        );
        let personal = super::node_frontend_helpers::node_r51_load(format!("default:{net}"));
        let defaults = loaded_or(
            personal.clone(),
            loaded_or(
                super::node_frontend_helpers::node_r51_load(format!("factory:{net}")),
                current.clone(),
            ),
        );
        let same_saved = stable(&current) == stable(&saved);
        let same_defaults = stable(&current) == stable(&defaults);
        let locked = bridge_locked(callbacks, &net);
        for button in buttons(root, &net) {
            let action = action_name(&button);
            let disabled = locked
                || if action == "save" {
                    same_saved
                } else {
                    same_defaults
                };
            set(&button, "disabled", &JsValue::from_bool(disabled));
            set_attribute(&button, "aria-disabled", &disabled.to_string());
            let class_list = property(&button, "classList");
            let _ = call2(
                &class_list,
                "toggle",
                &JsValue::from_str(DISABLED_CLASS),
                &JsValue::from_bool(disabled),
            );
            set(
                &dataset(&button),
                "kgwSettingsOwnerV19Disabled",
                &JsValue::from_str(&disabled.to_string()),
            );
            let title = if disabled {
                if action == "save" {
                    "No unsaved changes.".to_owned()
                } else {
                    "Current settings already match the selected defaults.".to_owned()
                }
            } else if action == "restore" {
                format!(
                    "Load {} defaults. Save to retain them.",
                    if crate::js_boolean(&personal) {
                        "your personal"
                    } else {
                        "KaspaGateway"
                    }
                )
            } else if action == "defaults" {
                "Use current settings as your personal defaults for this network.".to_owned()
            } else {
                "Save this network's settings.".to_owned()
            };
            set(&button, "title", &JsValue::from_str(&title));
        }
        let helper = query(root, &format!("[data-settings-defaults-context=\"{net}\"]"));
        if present(&helper) {
            let label = if crate::js_boolean(&personal) {
                "personal defaults"
            } else {
                "KaspaGateway defaults"
            };
            set(
                &helper,
                "textContent",
                &JsValue::from_str(&format!(
                    "Restore uses {label}. Settings apply on the next Start."
                )),
            );
        }
    }

    let details = Object::new();
    set(details.as_ref(), "network", &JsValue::from_str(network));
    set(details.as_ref(), "reason", &JsValue::from_str(reason));
    trace("settings-actions-reconciled", details.into());
}

fn set_dirty(root: &JsValue, network: &str, dirty: bool, reason: &str, callbacks: &JsValue) {
    DIRTY.with(|map| {
        map.borrow_mut().insert(network.to_owned(), dirty);
    });
    set_disabled(
        root,
        network,
        if reason.is_empty() {
            if dirty { "dirty" } else { "clean" }
        } else {
            reason
        },
        callbacks,
    );
}
fn remember_original_label(button: &JsValue, action: &str) {
    let ds = dataset(button);
    let current = text(&property(button, "textContent")).trim().to_owned();
    let original = text(&property(&ds, "kgwSettingsOwnerV19OriginalLabel"));
    if original.is_empty() || is_feedback_label_text(&current) {
        let label = if !current.is_empty() && !is_feedback_label_text(&current) {
            current
        } else {
            fallback_text(action).to_owned()
        };
        set(
            &ds,
            "kgwSettingsOwnerV19OriginalLabel",
            &JsValue::from_str(&label),
        );
    }
    set(&ds, "kgwSettingsOwnerV19Action", &JsValue::from_str(action));
}

fn restore_label(button: &JsValue) {
    let original = text(&property(
        &dataset(button),
        "kgwSettingsOwnerV19OriginalLabel",
    ));
    let label = if original.is_empty() {
        fallback_text(action_name(button)).to_owned()
    } else {
        original
    };
    set(button, "textContent", &JsValue::from_str(&label));
    set_attribute(button, "aria-label", &label);
}

fn restore_labels(root: &JsValue, network: &str) {
    for button in buttons(root, network) {
        restore_label(&button);
    }
}
fn clear_feedback(_root: &JsValue, network: &str, reason: &str) {
    let active = FEEDBACK.with(|map| map.borrow_mut().remove(network));
    let Some(active) = active else {
        return;
    };
    clear_timeout(active.timer);
    restore_label(&active.button);
    let details = Object::new();
    set(details.as_ref(), "network", &JsValue::from_str(network));
    set(
        details.as_ref(),
        "reason",
        &JsValue::from_str(if reason.is_empty() { "clear" } else { reason }),
    );
    trace("v19-feedback-cleared", details.into());
}

fn start_visual_feedback(
    root: JsValue,
    network: String,
    button: JsValue,
    action: String,
    callbacks: JsValue,
) {
    let callback = Closure::wrap(Box::new(move || {
        if text(&property(&dataset(&button), "kgwSettingsActionResult")) != "success" {
            return;
        }
        clear_feedback(&root, &network, "new-feedback");
        DIRTY.with(|map| {
            map.borrow_mut().insert(network.clone(), false);
        });
        remember_original_label(&button, &action);
        let label = feedback_text(&action);
        set(&button, "textContent", &JsValue::from_str(&label));
        set_attribute(&button, "aria-label", &label);
        set_attribute(&button, "title", &label);
        set_disabled(&root, &network, "feedback-clean-state", &callbacks);
        let root_late = root.clone();
        let network_late = network.clone();
        let button_late = button.clone();
        let action_late = action.clone();
        let callbacks_late = callbacks.clone();
        let late = Closure::wrap(Box::new(move || {
            let matches = FEEDBACK.with(|map| {
                map.borrow()
                    .get(&network_late)
                    .is_some_and(|active| Object::is(&active.button, &button_late))
            });
            if !matches {
                return;
            }
            FEEDBACK.with(|map| {
                map.borrow_mut().remove(&network_late);
            });
            restore_label(&button_late);
            let dirty = DIRTY.with(|map| map.borrow().get(&network_late).copied().unwrap_or(false));
            set_disabled(
                &root_late,
                &network_late,
                if dirty {
                    "feedback-complete-dirty"
                } else {
                    "feedback-complete-clean"
                },
                &callbacks_late,
            );
            let details = Object::new();
            set(
                details.as_ref(),
                "network",
                &JsValue::from_str(&network_late),
            );
            set(details.as_ref(), "action", &JsValue::from_str(&action_late));
            set(
                details.as_ref(),
                "holdMs",
                &JsValue::from_f64(f64::from(FEEDBACK_MS)),
            );
            set(details.as_ref(), "dirty", &JsValue::from_bool(dirty));
            set(details.as_ref(), "safeNoFreeze", &JsValue::TRUE);
            trace("v19-feedback-complete", details.into());
        }) as Box<dyn FnMut()>);
        let timer = set_timeout(late.as_ref().unchecked_ref(), FEEDBACK_MS).unwrap_or(0.0);
        late.forget();
        FEEDBACK.with(|map| {
            map.borrow_mut().insert(
                network.clone(),
                Feedback {
                    timer,
                    button: button.clone(),
                },
            );
        });

        let details = Object::new();
        set(details.as_ref(), "network", &JsValue::from_str(&network));
        set(details.as_ref(), "action", &JsValue::from_str(&action));
        set(
            details.as_ref(),
            "holdMs",
            &JsValue::from_f64(f64::from(FEEDBACK_MS)),
        );
        set(details.as_ref(), "label", &JsValue::from_str(&label));
        set(details.as_ref(), "visualOnly", &JsValue::TRUE);
        set(details.as_ref(), "safeNoFreeze", &JsValue::TRUE);
        trace("v19-feedback-start", details.into());
    }) as Box<dyn FnMut()>);
    let _ = set_timeout(callback.as_ref().unchecked_ref(), 0);
    callback.forget();
}

fn change_trace_details(event: &JsValue) -> JsValue {
    let output = Object::new();
    set(
        output.as_ref(),
        "patch",
        &JsValue::from_str("KGW_SETTINGS_CHANGE_TRACE_OWNER_R44H2"),
    );
    set(
        output.as_ref(),
        "trusted",
        &JsValue::from_bool(bool_property(event, "isTrusted")),
    );
    let target = property(event, "target");
    for (key, name) in [
        ("targetId", "id"),
        ("targetName", "name"),
        ("targetTag", "tagName"),
    ] {
        set(
            output.as_ref(),
            key,
            &JsValue::from_str(&text(&property(&target, name))),
        );
    }
    output.into()
}

fn install_event_listener(root: &JsValue, kind: &'static str, callbacks: &JsValue) {
    let root_for_callback = root.clone();
    let callbacks_for_callback = callbacks.clone();
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        if !is_settings_control(&target) {
            return;
        }
        let network = network_of(&target);
        let event_type = text(&property(&event, "type"));
        trace(
            &format!(
                "r29b-settings-control-{}-seen",
                if event_type.is_empty() {
                    kind
                } else {
                    &event_type
                }
            ),
            event_trace_details(&root_for_callback, &event, &network, "settings-control"),
        );
        trace(&format!("r44h2-{kind}-seen"), change_trace_details(&event));
        if !bool_property(&event, "isTrusted") {
            set_disabled(
                &root_for_callback,
                &network,
                &format!("{kind}-programmatic"),
                &callbacks_for_callback,
            );
            trace(
                &format!("r44h2-{kind}-programmatic-disabled"),
                change_trace_details(&event),
            );
            return;
        }
        clear_feedback(&root_for_callback, &network, &format!("trusted-{kind}"));
        restore_labels(&root_for_callback, &network);
        set_dirty(
            &root_for_callback,
            &network,
            true,
            &format!("trusted-{kind}"),
            &callbacks_for_callback,
        );
        trace(
            &format!("r44h2-trusted-{kind}-dirty"),
            change_trace_details(&event),
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        root,
        "addEventListener",
        &JsValue::from_str(kind),
        closure.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    closure.forget();
}
fn install_click_listener(root: &JsValue, callbacks: &JsValue) {
    let root_for_callback = root.clone();
    let callbacks_for_callback = callbacks.clone();
    let closure = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let button =
            call1(&target, "closest", &JsValue::from_str("button")).unwrap_or(JsValue::UNDEFINED);
        if !present(&button)
            || !call1(&root_for_callback, "contains", &button)
                .is_some_and(|value| crate::js_boolean(&value))
            || !is_action_button(&button)
        {
            return;
        }
        let network = network_of(&button);
        let action = action_name(&button).to_owned();
        let disabled = bool_property(&button, "disabled")
            || text(&property(&dataset(&button), "kgwSettingsOwnerV19Disabled")) == "true";

        trace(
            "r29b-settings-action-click",
            button_trace_details(
                &root_for_callback,
                &event,
                &button,
                &network,
                &action,
                disabled,
            ),
        );
        let details = Object::new();
        set(details.as_ref(), "network", &JsValue::from_str(&network));
        set(details.as_ref(), "action", &JsValue::from_str(&action));
        set(details.as_ref(), "disabled", &JsValue::from_bool(disabled));
        set(
            details.as_ref(),
            "label",
            &JsValue::from_str(text(&property(&button, "textContent")).trim()),
        );
        trace("v19-click", details.into());

        if disabled {
            prevent_default(&event);
            stop_immediate(&event);
            set_disabled(
                &root_for_callback,
                &network,
                "click-blocked-clean-state",
                &callbacks_for_callback,
            );
            return;
        }
        if action != "restore" {
            let errors = super::node_frontend_helpers::node_validate_form(network.clone(), true);
            if errors.is_object()
                && !errors.is_null()
                && Object::keys(&Object::from(errors)).length() > 0
            {
                prevent_default(&event);
                stop_immediate(&event);
                return;
            }
        }
        set(
            &dataset(&button),
            "kgwSettingsActionResult",
            &JsValue::from_str("pending"),
        );
        start_visual_feedback(
            root_for_callback.clone(),
            network,
            button,
            action,
            callbacks_for_callback.clone(),
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        root,
        "addEventListener",
        &JsValue::from_str("click"),
        closure.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    closure.forget();
}

#[wasm_bindgen(js_name = nodeInstallSettingsOwner)]
pub fn node_install_settings_owner(root: JsValue, callbacks: JsValue) -> bool {
    if !present(&root) {
        return false;
    }
    let ds = dataset(&root);
    if text(&property(&ds, ROOT_INSTALLED_ATTR)) == "installed" {
        return false;
    }
    set(&ds, ROOT_INSTALLED_ATTR, &JsValue::from_str("installed"));
    set_disabled(&root, "all", "initial", &callbacks);
    install_event_listener(&root, "input", &callbacks);
    install_event_listener(&root, "change", &callbacks);
    install_click_listener(&root, &callbacks);

    let details = Object::new();
    set(details.as_ref(), "scope", &JsValue::from_str(SCOPE));
    set(details.as_ref(), "patch", &JsValue::from_str(PATCH));
    set(
        details.as_ref(),
        "feedbackMs",
        &JsValue::from_f64(f64::from(FEEDBACK_MS)),
    );
    set(details.as_ref(), "safeNoFreeze", &JsValue::TRUE);
    trace("v19-owner-installed", details.into());
    true
}

#[wasm_bindgen(js_name = nodeSettingsOwnerSetDisabled)]
pub fn node_settings_owner_set_disabled(
    root: JsValue,
    network: String,
    _disabled: bool,
    reason: String,
    callbacks: JsValue,
) {
    set_disabled(&root, &network, &reason, &callbacks);
}

#[wasm_bindgen(js_name = nodeSettingsOwnerButtons)]
pub fn node_settings_owner_buttons(root: JsValue, network: String) -> Array {
    let output = Array::new();
    for button in buttons(&root, &network) {
        output.push(&button);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_fallbacks_preserve_contract() {
        assert_eq!(fallback_text("save"), "Save Settings");
        assert_eq!(fallback_text("restore"), "Restore Defaults");
        assert_eq!(fallback_text("defaults"), "Set as Defaults");
    }

    #[test]
    fn feedback_labels_are_recognized() {
        for label in ["Saved", "Restored", "Set", "Set as Defaults"] {
            assert!(is_feedback_label_text(label));
        }
        assert!(!is_feedback_label_text("Save Settings"));
    }
}
