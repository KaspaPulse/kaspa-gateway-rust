use super::settings_schema::{NODE_ENDPOINTS, NODE_MANAGED, NODE_OPTIONAL, NODE_REQUIRED};
use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NetworkProfile {
    key: &'static str,
    label: &'static str,
    testnet: bool,
    netsuffix: &'static str,
    enabled_by_default: bool,
    experimental: bool,
    runtime: &'static str,
}

const NETWORKS: [NetworkProfile; 3] = [
    NetworkProfile {
        key: "mainnet",
        label: "Mainnet",
        testnet: false,
        netsuffix: "",
        enabled_by_default: true,
        experimental: false,
        runtime: "Official Rusty Kaspa",
    },
    NetworkProfile {
        key: "testnet10",
        label: "Testnet 10",
        testnet: true,
        netsuffix: "10",
        enabled_by_default: true,
        experimental: false,
        runtime: "Official Rusty Kaspa",
    },
    NetworkProfile {
        key: "testnet13",
        label: "Testnet 13",
        testnet: true,
        netsuffix: "13",
        enabled_by_default: false,
        experimental: true,
        runtime: "DAGKnight - Experimental",
    },
];

fn profile(net: &str) -> Option<&'static NetworkProfile> {
    NETWORKS.iter().find(|item| item.key == net)
}

fn policy_key_text(net: &str) -> String {
    format!(
        "kgw.node.network.enabled.{}",
        if net.is_empty() { "unknown" } else { net }
    )
}

fn policy_message_text(net: &str) -> String {
    let Some(profile) = profile(net) else {
        return String::new();
    };
    if profile.experimental {
        let mut message =
            "Experimental network. Disabled by default and requires explicit opt-in.".to_owned();
        if net == "testnet13" {
            message.push_str(
                " This Testnet13 build has no DNS seeders. For public sync, set a trusted Testnet13 peer in Connect or Add Peer.",
            );
        }
        message
    } else {
        format!(
            "{}. RPC remains loopback-only and data is isolated per network.",
            profile.runtime
        )
    }
}

fn escape_html_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            other => output.push(other),
        }
    }
    output
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

fn call1(target: &JsValue, name: &str, first: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, first).ok()
}
fn call2(target: &JsValue, name: &str, first: &JsValue, second: &JsValue) -> Option<JsValue> {
    function(target, name)?.call2(target, first, second).ok()
}

fn profile_object(spec: &NetworkProfile) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "key", &JsValue::from_str(spec.key));
    set(output.as_ref(), "label", &JsValue::from_str(spec.label));
    set(
        output.as_ref(),
        "testnet",
        &JsValue::from_bool(spec.testnet),
    );
    set(
        output.as_ref(),
        "netsuffix",
        &JsValue::from_str(spec.netsuffix),
    );
    set(
        output.as_ref(),
        "enabledByDefault",
        &JsValue::from_bool(spec.enabled_by_default),
    );
    if spec.experimental {
        set(output.as_ref(), "experimental", &JsValue::TRUE);
    }
    set(output.as_ref(), "runtime", &JsValue::from_str(spec.runtime));
    output.into()
}

fn local_storage() -> JsValue {
    property(&window(), "localStorage")
}

fn storage_get(key: &str) -> Option<String> {
    let value = call1(&local_storage(), "getItem", &JsValue::from_str(key))?;
    present(&value).then(|| crate::js_string_owned(&value))
}

fn storage_set(key: &str, value: &str) {
    let _ = call2(
        &local_storage(),
        "setItem",
        &JsValue::from_str(key),
        &JsValue::from_str(value),
    );
}

fn translator(owner: &JsValue, name: &str) -> Option<Function> {
    function(owner, name)
}

fn translate_raw(key: &str, fallback: &str) -> JsValue {
    let win = window();
    if let Some(callback) = translator(&win, "kgwT")
        && let Ok(value) =
            callback.call2(&win, &JsValue::from_str(key), &JsValue::from_str(fallback))
    {
        return value;
    }

    for owner_name in ["KGW_I18N", "i18n"] {
        let owner = property(&win, owner_name);
        if let Some(callback) = translator(&owner, "t")
            && let Ok(value) = callback.call2(
                &owner,
                &JsValue::from_str(key),
                &JsValue::from_str(fallback),
            )
        {
            return value;
        }
    }
    JsValue::from_str(fallback)
}

#[wasm_bindgen(js_name = nodeI18nText)]
pub fn node_i18n_text(key: String, fallback: String) -> JsValue {
    translate_raw(&key, &fallback)
}
fn backend_invoke_function() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    let core = property(&tauri, "core");
    let legacy = property(&tauri, "tauri");
    for candidate in [
        property(&core, "invoke"),
        property(&legacy, "invoke"),
        property(&win, "__TAURI_INVOKE__"),
    ] {
        if let Ok(invoke) = candidate.dyn_into::<Function>() {
            return Some(invoke);
        }
    }
    None
}

#[wasm_bindgen(js_name = nodeBackendInvoke)]
pub fn node_backend_invoke(command: String, payload: JsValue) -> Result<Promise, JsValue> {
    let Some(invoke) = backend_invoke_function() else {
        return Err(JsValue::from_str("Tauri invoke is not available"));
    };
    let result = invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(&command), &payload)?;
    Ok(Promise::resolve(&result))
}

#[wasm_bindgen(js_name = nodeNetworkProfiles)]
pub fn node_network_profiles() -> Array {
    let output = Array::new();
    for spec in &NETWORKS {
        output.push(&profile_object(spec));
    }
    output
}

#[wasm_bindgen(js_name = nodeNetworkPolicyKey)]
pub fn node_network_policy_key(net: String) -> String {
    policy_key_text(&net)
}

#[wasm_bindgen(js_name = nodeNetworkProfile)]
pub fn node_network_profile(net: String) -> JsValue {
    profile(&net).map(profile_object).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen(js_name = nodeNetworkEnabled)]
pub fn node_network_enabled(net: String) -> bool {
    let fallback = profile(&net).is_some_and(|item| item.enabled_by_default);
    match storage_get(&policy_key_text(&net)).as_deref() {
        Some("1") => true,
        Some("0") => false,
        _ => fallback,
    }
}

#[wasm_bindgen(js_name = nodeSetNetworkEnabled)]
pub fn node_set_network_enabled(net: String, enabled: bool) {
    storage_set(&policy_key_text(&net), if enabled { "1" } else { "0" });
}

#[wasm_bindgen(js_name = nodeNetworkPolicyMessage)]
pub fn node_network_policy_message(net: String) -> String {
    policy_message_text(&net)
}

#[wasm_bindgen(js_name = nodeById)]
pub fn node_by_id(id: String) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(&id)).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen(js_name = nodeEscapeHtml)]
pub fn node_escape_html(value: JsValue) -> String {
    escape_html_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = nodeElementId)]
pub fn node_element_id(net: String, name: String) -> String {
    format!("node-{net}-{name}")
}

#[wasm_bindgen(js_name = nodeValue)]
pub fn node_value(net: String, name: String) -> String {
    let element = node_by_id(node_element_id(net, name));
    if !present(&element) {
        return String::new();
    }
    crate::js_string_owned(&property(&element, "value"))
        .trim()
        .to_owned()
}

#[wasm_bindgen(js_name = nodeChecked)]
pub fn node_checked(net: String, name: String) -> bool {
    let element = node_by_id(node_element_id(net, name));
    present(&element) && crate::js_boolean(&property(&element, "checked"))
}

const COMMAND_STATE_GLOBAL: &str = "__kgwNodeCommandComposerInlineR7";

fn schema_pair_contains(items: &[(&str, &str)], name: &str) -> bool {
    items.iter().any(|(key, _)| *key == name)
}

fn schema_endpoint_field(name: &str) -> bool {
    NODE_ENDPOINTS
        .iter()
        .any(|(_, host, port, _, _)| *host == name || *port == name)
}

fn command_state_key_text(net: &str) -> String {
    if net.is_empty() {
        "mainnet".to_owned()
    } else {
        net.to_owned()
    }
}

fn command_state_object(net: &str) -> JsValue {
    let win = window();
    let mut root = property(&win, COMMAND_STATE_GLOBAL);
    if !root.is_object() || root.is_null() {
        root = Object::new().into();
        set(&win, COMMAND_STATE_GLOBAL, &root);
    }

    let key = command_state_key_text(net);
    let mut state = property(&root, &key);
    if !state.is_object() || state.is_null() {
        state = Object::new().into();
        set(&root, &key, &state);
    }
    state
}

fn command_option_enabled_inner(net: &str, name: &str) -> bool {
    if schema_pair_contains(NODE_REQUIRED, name) {
        return true;
    }
    let state = command_state_object(net);
    let value = property(&state, name);
    if NODE_OPTIONAL.contains(&name) {
        value.as_bool() == Some(true)
    } else {
        value.as_bool() != Some(false)
    }
}

fn command_toggle_markup(net: &str, name: &str, enabled: bool) -> String {
    let checked = if enabled { "checked" } else { "" };
    format!(
        "<input type=\"checkbox\" class=\"kgw-command-option-checkbox-r9\" data-node-command-option-toggle-r7=\"{}\" data-net=\"{}\" {checked} aria-label=\"Use {}\" title=\"Enable this optional setting\">",
        escape_html_text(name),
        escape_html_text(net),
        escape_html_text(name),
    )
}

fn command_inline_toggle_html(net: &str, name: &str) -> String {
    if schema_pair_contains(NODE_MANAGED, name)
        || schema_pair_contains(NODE_REQUIRED, name)
        || schema_endpoint_field(name)
    {
        return String::new();
    }
    command_toggle_markup(net, name, command_option_enabled_inner(net, name))
}

fn refresh_command_toggles(net: &str) {
    let Some(list) = call1(
        &document(),
        "querySelectorAll",
        &JsValue::from_str("[data-node-command-option-toggle-r7]"),
    ) else {
        return;
    };
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return;
    }

    for index in 0..length as u32 {
        let Ok(element) = Reflect::get(&list, &JsValue::from_f64(index as f64)) else {
            continue;
        };
        let data = property(&element, "dataset");
        if crate::js_string_owned(&property(&data, "net")) != net {
            continue;
        }
        let name = crate::js_string_owned(&property(&data, "nodeCommandOptionToggleR7"));
        let enabled = command_option_enabled_inner(net, &name);
        set(&element, "checked", &JsValue::from_bool(enabled));
        let label = if enabled {
            "Included in command"
        } else {
            "Excluded from command"
        };
        let _ = call2(
            &element,
            "setAttribute",
            &JsValue::from_str("aria-label"),
            &JsValue::from_str(label),
        );
        let _ = call2(
            &element,
            "setAttribute",
            &JsValue::from_str("title"),
            &JsValue::from_str(label),
        );
        let classes = property(&element, "classList");
        let _ = call2(
            &classes,
            "toggle",
            &JsValue::from_str("is-on"),
            &JsValue::from_bool(enabled),
        );
        let _ = call2(
            &classes,
            "toggle",
            &JsValue::from_str("is-off"),
            &JsValue::from_bool(!enabled),
        );
    }
}

#[wasm_bindgen(js_name = nodeCommandInlineStateKey)]
pub fn node_command_inline_state_key(net: String) -> String {
    command_state_key_text(&net)
}

#[wasm_bindgen(js_name = nodeCommandInlineState)]
pub fn node_command_inline_state(net: String) -> JsValue {
    command_state_object(&net)
}

#[wasm_bindgen(js_name = nodeCommandOptionEnabled)]
pub fn node_command_option_enabled(net: String, name: String) -> bool {
    command_option_enabled_inner(&net, &name)
}

#[wasm_bindgen(js_name = nodeCommandShouldInclude)]
pub fn node_command_should_include(net: String, name: String) -> bool {
    command_option_enabled_inner(&net, &name)
}

#[wasm_bindgen(js_name = nodeCommandInlineToggle)]
pub fn node_command_inline_toggle(net: String, name: String) -> String {
    command_inline_toggle_html(&net, &name)
}

#[wasm_bindgen(js_name = nodeRefreshInlineCommandToggles)]
pub fn node_refresh_inline_command_toggles(net: String) {
    refresh_command_toggles(&net);
}

#[wasm_bindgen(js_name = nodeToggleCommandOption)]
pub fn node_toggle_command_option(net: String, name: String) -> bool {
    let state = command_state_object(&net);
    let enabled = property(&state, &name).as_bool() == Some(false);
    set(&state, &name, &JsValue::from_bool(enabled));
    refresh_command_toggles(&net);
    enabled
}

const COMMAND_OPTIONS_KEY: &str = "__kgwNodeCommandOptionsR38C";

fn command_option_restore_enabled(name: &str, enabled: bool, has_value: bool) -> bool {
    enabled && (!NODE_OPTIONAL.contains(&name) || has_value)
}

fn read_command_options(net: &str) -> JsValue {
    let output = Object::new();
    let Some(root) = call1(
        &document(),
        "getElementById",
        &JsValue::from_str("kaspa-node"),
    ) else {
        return output.into();
    };
    if !present(&root) {
        return output.into();
    }
    let Some(list) = call1(
        &root,
        "querySelectorAll",
        &JsValue::from_str("[data-node-command-option-toggle-r7]"),
    ) else {
        return output.into();
    };
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return output.into();
    }
    for index in 0..length as u32 {
        let Ok(item) = Reflect::get(&list, &JsValue::from_f64(index as f64)) else {
            continue;
        };
        let data = property(&item, "dataset");
        if crate::js_string_owned(&property(&data, "net")) != net {
            continue;
        }
        let name = crate::js_string_owned(&property(&data, "nodeCommandOptionToggleR7"));
        if name.is_empty() {
            continue;
        }
        set(
            output.as_ref(),
            &name,
            &JsValue::from_bool(crate::js_boolean(&property(&item, "checked"))),
        );
    }
    output.into()
}

fn apply_command_options(net: &str, values: &JsValue) -> JsValue {
    let result = Object::new();
    set(result.as_ref(), "applied", &JsValue::FALSE);
    set(result.as_ref(), "count", &JsValue::from_f64(0.0));

    let options = property(values, COMMAND_OPTIONS_KEY);
    if !options.is_object() || options.is_null() {
        return result.into();
    }

    let state = command_state_object(net);
    let entries = Object::entries(&Object::from(options));
    let mut count = 0_u32;
    for entry in entries.iter() {
        let pair = Array::from(&entry);
        if pair.length() < 2 {
            continue;
        }
        let name = crate::js_string_owned(&pair.get(0));
        let enabled = crate::js_boolean(&pair.get(1));
        let has_value = !node_value(net.to_owned(), name.clone()).is_empty();
        set(
            &state,
            &name,
            &JsValue::from_bool(command_option_restore_enabled(&name, enabled, has_value)),
        );
        count += 1;
    }
    refresh_command_toggles(net);
    set(result.as_ref(), "applied", &JsValue::TRUE);
    set(
        result.as_ref(),
        "count",
        &JsValue::from_f64(f64::from(count)),
    );
    result.into()
}

#[wasm_bindgen(js_name = nodeCommandOptionsKey)]
pub fn node_command_options_key() -> String {
    COMMAND_OPTIONS_KEY.to_owned()
}

#[wasm_bindgen(js_name = nodeReadCommandOptions)]
pub fn node_read_command_options(net: String) -> JsValue {
    read_command_options(&net)
}

#[wasm_bindgen(js_name = nodeApplyCommandOptions)]
pub fn node_apply_command_options(net: String, values: JsValue) -> JsValue {
    apply_command_options(&net, &values)
}

const R51_STORAGE_PREFIX: &str = "kgw.node.direct.v51.";

fn r51_keys_array() -> Array {
    let output = Array::new();
    for spec in NETWORKS {
        output.push(&JsValue::from_str(spec.key));
    }
    output
}

fn r51_panel(net: &str) -> JsValue {
    query(
        &document(),
        &format!("[data-node-network-panel=\"{}\"]", escape_html_text(net)),
    )
}

fn r51_fields_vec(net: &str) -> Vec<JsValue> {
    let panel = r51_panel(net);
    if !present(&panel) {
        return Vec::new();
    }
    let Some(list) = call1(
        &panel,
        "querySelectorAll",
        &JsValue::from_str("input, select, textarea"),
    ) else {
        return Vec::new();
    };
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    let prefix = format!("node-{net}-");
    (0..length as u32)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .filter(|field| {
            let id = crate::js_string_owned(&property(field, "id"));
            if id.is_empty()
                || !id.starts_with(&prefix)
                || id.ends_with("-commandPreview")
                || id.ends_with("-logOutput")
            {
                return false;
            }
            let toolbar = call1(field, "closest", &JsValue::from_str(".node-v6-log-toolbar"))
                .unwrap_or(JsValue::UNDEFINED);
            !present(&toolbar)
        })
        .collect()
}

fn r51_fields_array(net: &str) -> Array {
    let output = Array::new();
    for field in r51_fields_vec(net) {
        output.push(&field);
    }
    output
}

fn r51_read_settings(net: &str) -> JsValue {
    let values = Object::new();
    let command_options = read_command_options(net);
    set(values.as_ref(), COMMAND_OPTIONS_KEY, &command_options);

    let prefix = format!("node-{net}-");
    for field in r51_fields_vec(net) {
        let id = crate::js_string_owned(&property(&field, "id"));
        if id.is_empty() {
            continue;
        }
        let name = id.strip_prefix(&prefix).unwrap_or(&id);
        if schema_pair_contains(NODE_MANAGED, name) {
            continue;
        }
        let item = Object::new();
        if crate::js_string_owned(&property(&field, "type")) == "checkbox" {
            set(item.as_ref(), "type", &JsValue::from_str("checkbox"));
            set(
                item.as_ref(),
                "checked",
                &JsValue::from_bool(crate::js_boolean(&property(&field, "checked"))),
            );
        } else {
            set(item.as_ref(), "type", &JsValue::from_str("value"));
            set(
                item.as_ref(),
                "value",
                &JsValue::from_str(&crate::js_string_owned(&property(&field, "value"))),
            );
        }
        set(values.as_ref(), &id, item.as_ref());
    }
    values.into()
}

fn dispatch_bubbling_event(target: &JsValue, name: &str) {
    let Ok(constructor) = property(&global(), "Event").dyn_into::<Function>() else {
        return;
    };
    let options = Object::new();
    set(options.as_ref(), "bubbles", &JsValue::TRUE);
    let args = Array::new();
    args.push(&JsValue::from_str(name));
    args.push(options.as_ref());
    if let Ok(event) = Reflect::construct(&constructor, &args) {
        let _ = call1(target, "dispatchEvent", &event);
    }
}

fn r51_write_settings(net: &str, values: &JsValue) -> JsValue {
    let result = Object::new();
    set(result.as_ref(), "applied", &JsValue::FALSE);
    set(result.as_ref(), "commandOptionsApplied", &JsValue::FALSE);
    set(
        result.as_ref(),
        "commandOptionsCount",
        &JsValue::from_f64(0.0),
    );
    if !values.is_object() || values.is_null() {
        return result.into();
    }

    let prefix = format!("node-{net}-");
    for field in r51_fields_vec(net) {
        let id = crate::js_string_owned(&property(&field, "id"));
        if id.is_empty() {
            continue;
        }
        let name = id.strip_prefix(&prefix).unwrap_or(&id);
        if schema_pair_contains(NODE_MANAGED, name) {
            continue;
        }
        let item = property(values, &id);
        if !present(&item) {
            continue;
        }
        if crate::js_string_owned(&property(&field, "type")) == "checkbox" {
            set(
                &field,
                "checked",
                &JsValue::from_bool(crate::js_boolean(&property(&item, "checked"))),
            );
        } else if Reflect::has(&item, &JsValue::from_str("value")).unwrap_or(false) {
            set(
                &field,
                "value",
                &JsValue::from_str(&crate::js_string_owned(&property(&item, "value"))),
            );
        }
        dispatch_bubbling_event(&field, "input");
        dispatch_bubbling_event(&field, "change");
    }

    let command = apply_command_options(net, values);
    set(
        result.as_ref(),
        "commandOptionsApplied",
        &property(&command, "applied"),
    );
    set(
        result.as_ref(),
        "commandOptionsCount",
        &property(&command, "count"),
    );
    set(result.as_ref(), "applied", &JsValue::TRUE);
    result.into()
}

fn r51_storage_key(key: &str) -> String {
    format!("{R51_STORAGE_PREFIX}{key}")
}

fn r51_store(key: &str, value: &JsValue) -> Result<(), JsValue> {
    let serialized = JSON::stringify(value)?;
    let storage = local_storage();
    let setter = function(&storage, "setItem")
        .ok_or_else(|| JsValue::from_str("localStorage.setItem is unavailable"))?;
    setter.call2(
        &storage,
        &JsValue::from_str(&r51_storage_key(key)),
        serialized.as_ref(),
    )?;
    Ok(())
}

fn r51_load(key: &str) -> JsValue {
    let storage = local_storage();
    let Some(raw) = call1(
        &storage,
        "getItem",
        &JsValue::from_str(&r51_storage_key(key)),
    ) else {
        return JsValue::NULL;
    };
    if !present(&raw) {
        return JsValue::NULL;
    }
    let text = crate::js_string_owned(&raw);
    if text.is_empty() {
        return JsValue::NULL;
    }
    JSON::parse(&text).unwrap_or(JsValue::NULL)
}

fn r51_capture_factory_defaults() -> Result<(), JsValue> {
    for spec in NETWORKS {
        let key = format!("factory:{}", spec.key);
        if !crate::js_boolean(&r51_load(&key)) {
            r51_store(&key, &r51_read_settings(spec.key))?;
        }
    }
    Ok(())
}

fn r51_load_saved_settings() -> Array {
    let applied = Array::new();
    for spec in NETWORKS {
        let key = format!("saved:{}", spec.key);
        let saved = r51_load(&key);
        if !crate::js_boolean(&saved) {
            continue;
        }
        let result = r51_write_settings(spec.key, &saved);
        let entry = Object::new();
        set(entry.as_ref(), "net", &JsValue::from_str(spec.key));
        set(
            entry.as_ref(),
            "commandOptionsApplied",
            &property(&result, "commandOptionsApplied"),
        );
        set(
            entry.as_ref(),
            "commandOptionsCount",
            &property(&result, "commandOptionsCount"),
        );
        applied.push(entry.as_ref());
    }
    applied
}

#[wasm_bindgen(js_name = nodeR51Keys)]
pub fn node_r51_keys() -> Array {
    r51_keys_array()
}

#[wasm_bindgen(js_name = nodeR51Panel)]
pub fn node_r51_panel(net: String) -> JsValue {
    r51_panel(&net)
}

#[wasm_bindgen(js_name = nodeR51Fields)]
pub fn node_r51_fields(net: String) -> Array {
    r51_fields_array(&net)
}

#[wasm_bindgen(js_name = nodeR51ReadSettings)]
pub fn node_r51_read_settings(net: String) -> JsValue {
    r51_read_settings(&net)
}

#[wasm_bindgen(js_name = nodeR51WriteSettings)]
pub fn node_r51_write_settings(net: String, values: JsValue) -> JsValue {
    r51_write_settings(&net, &values)
}

#[wasm_bindgen(js_name = nodeR51Store)]
pub fn node_r51_store(key: String, value: JsValue) -> Result<(), JsValue> {
    r51_store(&key, &value)
}

#[wasm_bindgen(js_name = nodeR51Load)]
pub fn node_r51_load(key: String) -> JsValue {
    r51_load(&key)
}

#[wasm_bindgen(js_name = nodeR51CaptureFactoryDefaults)]
pub fn node_r51_capture_factory_defaults() -> Result<(), JsValue> {
    r51_capture_factory_defaults()
}

#[wasm_bindgen(js_name = nodeR51LoadSavedSettings)]
pub fn node_r51_load_saved_settings() -> Array {
    r51_load_saved_settings()
}

fn card_input_html(
    net: &str,
    name: &str,
    label: &str,
    value: &str,
    placeholder: &str,
    span2: bool,
    toggle: &str,
) -> String {
    let span = if span2 { " span2" } else { "" };
    let id = format!("node-{net}-{name}");
    format!(
        "\n    <div class=\"node-v6-card{span}\">\n      <span class=\"kgw-command-option-title-row-r8e\">\n        {toggle}\n        <label for=\"{id}\" class=\"kgw-command-option-title-text-r8e\">{}</label>\n      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->\n      <input id=\"{id}\" data-testid=\"kgw-node-field-{}-{}\" type=\"text\" value=\"{}\" placeholder=\"{}\">\n    </div>",
        escape_html_text(label),
        escape_html_text(net),
        escape_html_text(name),
        escape_html_text(value),
        escape_html_text(placeholder),
    )
}

fn card_select_html(
    net: &str,
    name: &str,
    label: &str,
    options: &[String],
    value: &str,
    span2: bool,
    toggle: &str,
) -> String {
    let span = if span2 { " span2" } else { "" };
    let id = format!("node-{net}-{name}");
    let opts = options
        .iter()
        .map(|item| {
            let selected = if item == value { " selected" } else { "" };
            let text = if item.is_empty() { "not set" } else { item };
            format!(
                "<option value=\"{}\"{selected}>{}</option>",
                escape_html_text(item),
                escape_html_text(text)
            )
        })
        .collect::<String>();
    format!(
        "\n    <div class=\"node-v6-card{span}\">\n      <span class=\"kgw-command-option-title-row-r8e\">\n        {toggle}\n        <label for=\"{id}\" class=\"kgw-command-option-title-text-r8e\">{}</label>\n      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->\n      <select id=\"{id}\" data-testid=\"kgw-node-field-{}-{}\">{opts}</select>\n    </div>",
        escape_html_text(label),
        escape_html_text(net),
        escape_html_text(name),
    )
}

fn card_check_html(net: &str, name: &str, label: &str, checked: bool, span2: bool) -> String {
    let span = if span2 { " span2" } else { "" };
    let checked = if checked { " checked" } else { "" };
    let id = format!("node-{net}-{name}");
    format!(
        "\n    <label class=\"node-v6-card check{span}\">\n      <input id=\"{id}\" data-testid=\"kgw-node-field-{}-{}\" type=\"checkbox\"{checked}>\n      <span>{}</span>\n    </label>",
        escape_html_text(net),
        escape_html_text(name),
        escape_html_text(label),
    )
}

#[wasm_bindgen(js_name = nodeCardInput)]
pub fn node_card_input(
    net: String,
    name: String,
    label: String,
    value: String,
    placeholder: String,
    span2: bool,
    toggle: String,
) -> String {
    card_input_html(&net, &name, &label, &value, &placeholder, span2, &toggle)
}

#[wasm_bindgen(js_name = nodeCardSelect)]
pub fn node_card_select(
    net: String,
    name: String,
    label: String,
    options: Array,
    value: String,
    span2: bool,
    toggle: String,
) -> String {
    let options = options
        .iter()
        .map(|value| crate::js_string_owned(&value))
        .collect::<Vec<_>>();
    card_select_html(&net, &name, &label, &options, &value, span2, &toggle)
}

#[wasm_bindgen(js_name = nodeCardCheck)]
pub fn node_card_check(
    net: String,
    name: String,
    label: String,
    checked: bool,
    span2: bool,
) -> String {
    card_check_html(&net, &name, &label, checked, span2)
}

fn inner_tab_storage_key_text(net: &str) -> String {
    format!(
        "kgw.node.innerTab.{}",
        if net.is_empty() { "unknown" } else { net }
    )
}

fn normalize_inner_tab_text(value: &str) -> &'static str {
    match value {
        "settings" => "settings",
        "log" => "log",
        _ => "log",
    }
}

#[wasm_bindgen(js_name = nodeNormalizeInnerTab)]
pub fn node_normalize_inner_tab(value: JsValue) -> String {
    normalize_inner_tab_text(&crate::js_string_owned(&value)).to_owned()
}

#[wasm_bindgen(js_name = nodeResolveInnerTab)]
pub fn node_resolve_inner_tab(net: String) -> String {
    let stored = storage_get(&inner_tab_storage_key_text(&net)).unwrap_or_default();
    normalize_inner_tab_text(&stored).to_owned()
}

#[wasm_bindgen(js_name = nodeSaveInnerTab)]
pub fn node_save_inner_tab(net: String, selected: JsValue) -> String {
    let normalized = node_normalize_inner_tab(selected);
    storage_set(&inner_tab_storage_key_text(&net), &normalized);
    normalized
}

fn normalize_network_text(value: &str) -> String {
    let normalized = value.trim();
    match normalized {
        "mainnet" | "testnet10" | "testnet13" => normalized.to_owned(),
        _ => String::new(),
    }
}

const LAST_NETWORK_KEY: &str = "kgw.node.lastNetwork";

#[wasm_bindgen(js_name = nodeNormalizeNetwork)]
pub fn node_normalize_network(value: JsValue) -> String {
    normalize_network_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = nodeReadLastNetwork)]
pub fn node_read_last_network() -> String {
    storage_get(LAST_NETWORK_KEY)
        .map(|value| normalize_network_text(&value))
        .unwrap_or_default()
}

#[wasm_bindgen(js_name = nodeSaveLastNetwork)]
pub fn node_save_last_network(net: JsValue) -> String {
    let normalized = node_normalize_network(net);
    if !normalized.is_empty() {
        storage_set(LAST_NETWORK_KEY, &normalized);
    }
    normalized
}

fn stringify_runtime_result_value(value: &JsValue) -> String {
    if value.is_null() || value.is_undefined() {
        return "No response".to_owned();
    }
    if let Some(text) = value.as_string() {
        return text;
    }
    JSON::stringify(value)
        .ok()
        .map(|text| crate::js_string_owned(text.as_ref()))
        .unwrap_or_else(|| crate::js_string_owned(value))
}

#[wasm_bindgen(js_name = nodeStringifyRuntimeResult)]
pub fn node_stringify_runtime_result(value: JsValue) -> String {
    stringify_runtime_result_value(&value)
}

fn normalize_runtime_error_value(error: &JsValue) -> String {
    if error.is_null() || error.is_undefined() {
        return "Unknown backend error".to_owned();
    }
    if let Some(text) = error.as_string() {
        return text;
    }
    let message = property(error, "message");
    if crate::js_boolean(&message) {
        return crate::js_string_owned(&message);
    }
    JSON::stringify(error)
        .ok()
        .map(|text| crate::js_string_owned(text.as_ref()))
        .unwrap_or_else(|| crate::js_string_owned(error))
}

#[wasm_bindgen(js_name = nodeNormalizeRuntimeError)]
pub fn node_normalize_runtime_error(error: JsValue) -> String {
    normalize_runtime_error_value(&error)
}

fn parse_runtime_fields_text(raw: &str) -> Vec<(String, String)> {
    raw.split(';')
        .filter_map(|part| {
            let index = part.find('=')?;
            if index == 0 {
                return None;
            }
            let key = part[..index].trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_owned(), part[index + 1..].trim().to_owned()))
        })
        .collect()
}

#[wasm_bindgen(js_name = nodeParseRuntimeFields)]
pub fn node_parse_runtime_fields(value: JsValue) -> JsValue {
    let raw = stringify_runtime_result_value(&value);
    let output = Object::new();
    for (key, value) in parse_runtime_fields_text(&raw) {
        set(output.as_ref(), &key, &JsValue::from_str(&value));
    }
    output.into()
}

fn running_status_text(value: &str) -> bool {
    value.to_ascii_lowercase().contains("readiness=ready")
        && (value.contains("running=true")
            || value.contains("node_running=true")
            || value.contains("official_core_running=true"))
}

#[wasm_bindgen(js_name = nodeRuntimeIsRunning)]
pub fn node_runtime_is_running(value: JsValue) -> bool {
    running_status_text(&crate::js_string_owned(&value))
}

fn runtime_error_from_status_text(value: &str) -> String {
    let fields = parse_runtime_fields_text(value);
    let candidate = fields
        .iter()
        .find(|(key, _)| key == "runtime_error")
        .or_else(|| fields.iter().find(|(key, _)| key == "runtimeError"))
        .map(|(_, value)| value.trim())
        .unwrap_or("");
    if candidate.is_empty() || candidate.eq_ignore_ascii_case("none") {
        String::new()
    } else {
        candidate.to_owned()
    }
}

#[wasm_bindgen(js_name = nodeRuntimeErrorFromStatus)]
pub fn node_runtime_error_from_status(value: JsValue) -> String {
    runtime_error_from_status_text(&crate::js_string_owned(&value))
}

fn log_auto_scroll_key_text(net: &str) -> String {
    format!("kgw.node.log.autoscroll.{net}")
}

#[wasm_bindgen(js_name = nodeLogAutoScrollEnabled)]
pub fn node_log_auto_scroll_enabled(net: String) -> bool {
    storage_get(&log_auto_scroll_key_text(&net)).as_deref() != Some("0")
}

#[wasm_bindgen(js_name = nodeSetLogAutoScroll)]
pub fn node_set_log_auto_scroll(net: String, enabled: bool) {
    storage_set(
        &log_auto_scroll_key_text(&net),
        if enabled { "1" } else { "0" },
    );
    if !enabled {
        return;
    }
    let output = node_by_id(node_element_id(net, "logOutput".to_owned()));
    if present(&output) {
        let height = property(&output, "scrollHeight");
        set(&output, "scrollTop", &height);
    }
}

fn create_element(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag)).unwrap_or(JsValue::UNDEFINED)
}

fn append_child(parent: &JsValue, child: &JsValue) {
    let _ = call1(parent, "appendChild", child);
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

#[wasm_bindgen(js_name = nodeInstallLogAutoScrollControls)]
pub fn node_install_log_auto_scroll_controls() {
    let doc = document();
    if !present(&doc) {
        return;
    }

    for profile in NETWORKS {
        let net = profile.key.to_owned();
        let output = node_by_id(node_element_id(net.clone(), "logOutput".to_owned()));
        if !present(&output) {
            continue;
        }

        let control_id = node_element_id(net.clone(), "logAutoScrollR27".to_owned());
        if present(&node_by_id(control_id.clone())) {
            continue;
        }

        let label = create_element("label");
        if !present(&label) {
            continue;
        }
        set(
            &label,
            "className",
            &JsValue::from_str("kgw-log-autoscroll-toggle"),
        );
        let _ = call2(
            &label,
            "setAttribute",
            &JsValue::from_str("data-kgw-log-autoscroll"),
            &JsValue::from_str("node"),
        );
        let _ = call2(
            &label,
            "setAttribute",
            &JsValue::from_str("title"),
            &JsValue::from_str("Keep the log pinned to the newest raw line."),
        );

        let checkbox = create_element("input");
        set(&checkbox, "type", &JsValue::from_str("checkbox"));
        set(&checkbox, "id", &JsValue::from_str(&control_id));
        set(
            &checkbox,
            "checked",
            &JsValue::from_bool(node_log_auto_scroll_enabled(net.clone())),
        );

        let net_for_change = net.clone();
        let checkbox_for_change = checkbox.clone();
        let control_for_change = control_id.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            let checked = crate::js_boolean(&property(&checkbox_for_change, "checked"));
            let details = Object::new();
            set(
                details.as_ref(),
                "patch",
                &JsValue::from_str("KGW_NODE_BRIDGE_LOG_CONTROLS_TRACE_PATCH_R51B3"),
            );
            set(
                details.as_ref(),
                "trusted",
                &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
            );
            set(
                details.as_ref(),
                "controlId",
                &JsValue::from_str(&control_for_change),
            );
            set(details.as_ref(), "checked", &JsValue::from_bool(checked));
            let _ = crate::node_start_trace::node_small_owner_trace(
                JsValue::from_str(&net_for_change),
                JsValue::from_str("log-autoscroll"),
                JsValue::from_str("r51b3-node-log-autoscroll-change"),
                details.into(),
            );
            node_set_log_auto_scroll(net_for_change.clone(), checked);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &checkbox,
            "addEventListener",
            &JsValue::from_str("change"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();

        let span = create_element("span");
        set(
            &span,
            "textContent",
            &translate_raw("common.autoScroll", "Auto-scroll"),
        );
        append_child(&label, &checkbox);
        append_child(&label, &span);

        let panel = {
            let candidate = call1(
                &output,
                "closest",
                &JsValue::from_str(
                    ".node-v6-inner-panel, [data-node-inner-panel], [data-inner-panel], [data-node-panel], [data-panel]",
                ),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if present(&candidate) {
                candidate
            } else {
                property(&output, "parentElement")
            }
        };
        let toolbar_selector = ".node-v6-log-toolbar, .node-log-toolbar, [data-node-log-toolbar]";
        let toolbar = {
            let candidate = query(&panel, toolbar_selector);
            if present(&candidate) {
                candidate
            } else {
                query(&property(&output, "parentElement"), toolbar_selector)
            }
        };

        if present(&toolbar) {
            append_child(&toolbar, &label);
        } else {
            let parent = property(&output, "parentElement");
            if present(&parent) {
                let _ = call2(&parent, "insertBefore", &label, &output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_profiles_match_legacy_defaults() {
        assert_eq!(NETWORKS.len(), 3);
        assert_eq!(profile("mainnet").unwrap().label, "Mainnet");
        assert!(profile("mainnet").unwrap().enabled_by_default);
        assert!(profile("testnet10").unwrap().testnet);
        assert_eq!(profile("testnet10").unwrap().netsuffix, "10");
        assert!(!profile("testnet13").unwrap().enabled_by_default);
        assert!(profile("testnet13").unwrap().experimental);
        assert!(profile("missing").is_none());
    }

    #[test]
    fn policy_text_matches_legacy_contract() {
        assert_eq!(policy_key_text(""), "kgw.node.network.enabled.unknown");
        assert_eq!(
            policy_key_text("testnet10"),
            "kgw.node.network.enabled.testnet10"
        );
        assert!(policy_message_text("mainnet").contains("Official Rusty Kaspa"));
        assert!(policy_message_text("testnet13").contains("no DNS seeders"));
        assert_eq!(policy_message_text("missing"), "");
    }

    #[test]
    fn tab_and_network_normalization_match_legacy_contract() {
        assert_eq!(inner_tab_storage_key_text(""), "kgw.node.innerTab.unknown");
        assert_eq!(
            inner_tab_storage_key_text("mainnet"),
            "kgw.node.innerTab.mainnet"
        );
        assert_eq!(normalize_inner_tab_text("settings"), "settings");
        assert_eq!(normalize_inner_tab_text("log"), "log");
        assert_eq!(normalize_inner_tab_text("unknown"), "log");
        assert_eq!(normalize_network_text(" mainnet "), "mainnet");
        assert_eq!(normalize_network_text("testnet10"), "testnet10");
        assert_eq!(normalize_network_text("testnet13"), "testnet13");
        assert_eq!(normalize_network_text("devnet"), "");
    }

    #[test]
    fn runtime_field_parsing_and_running_predicates_match_legacy_contract() {
        assert_eq!(
            parse_runtime_fields_text("role=node; running=true; x=a=b ;bad"),
            vec![
                ("role".to_owned(), "node".to_owned()),
                ("running".to_owned(), "true".to_owned()),
                ("x".to_owned(), "a=b".to_owned()),
            ]
        );
        assert!(running_status_text("readiness=READY;running=true"));
        assert!(running_status_text("READINESS=ready;node_running=true"));
        assert!(!running_status_text("readiness=READY;running=TRUE"));
        assert!(!running_status_text("running=true"));
        assert_eq!(
            runtime_error_from_status_text("runtime_error=boom;running=false"),
            "boom"
        );
        assert_eq!(runtime_error_from_status_text("runtimeError=None"), "");
        assert_eq!(runtime_error_from_status_text("runtime_error=  none "), "");
    }

    #[test]
    fn command_option_persistence_contract_matches_legacy_rule() {
        assert_eq!(COMMAND_OPTIONS_KEY, "__kgwNodeCommandOptionsR38C");
        assert!(command_option_restore_enabled("logLevel", true, false));
        assert!(command_option_restore_enabled("uaComment", true, true));
        assert!(!command_option_restore_enabled("uaComment", true, false));
        assert!(!command_option_restore_enabled("uaComment", false, true));
    }

    #[test]
    fn command_composer_schema_policy_matches_legacy_contract() {
        assert_eq!(command_state_key_text(""), "mainnet");
        assert_eq!(command_state_key_text("testnet10"), "testnet10");
        assert!(schema_pair_contains(NODE_REQUIRED, "logLevel"));
        assert!(schema_pair_contains(NODE_MANAGED, "appDir"));
        assert!(NODE_OPTIONAL.contains(&"uaComment"));
        assert!(schema_endpoint_field("rpcListenHost"));
        assert!(schema_endpoint_field("rpcListenPort"));
        assert!(!schema_endpoint_field("uaComment"));
    }

    #[test]
    fn command_composer_toggle_markup_is_stable_and_escaped() {
        let enabled = command_toggle_markup("main<net", "ua&Comment", true);
        assert!(enabled.contains("kgw-command-option-checkbox-r9"));
        assert!(enabled.contains("data-net=\"main&lt;net\""));
        assert!(enabled.contains("data-node-command-option-toggle-r7=\"ua&amp;Comment\""));
        assert!(enabled.contains(" checked aria-label="));

        let disabled = command_toggle_markup("mainnet", "uaComment", false);
        assert!(disabled.contains("data-net=\"mainnet\""));
        assert!(!disabled.contains(" checked aria-label="));
        assert!(disabled.contains(" aria-label=\"Use uaComment\""));
    }

    #[test]
    fn card_renderers_preserve_markup_and_escaping_contract() {
        let input = card_input_html(
            "mainnet",
            "uaComment",
            "<Label>",
            "A&B",
            "\"hint\"",
            true,
            "<input checked>",
        );
        assert!(input.contains("node-v6-card span2"));
        assert!(input.contains("<input checked>"));
        assert!(input.contains("&lt;Label&gt;"));
        assert!(input.contains("value=\"A&amp;B\""));
        assert!(input.contains("placeholder=\"&quot;hint&quot;\""));
        assert!(input.contains("data-testid=\"kgw-node-field-mainnet-uaComment\""));

        let options = vec![String::new(), "info".to_owned()];
        let select = card_select_html(
            "testnet10",
            "logLevel",
            "--loglevel",
            &options,
            "info",
            false,
            "",
        );
        assert!(select.contains("<option value=\"\">not set</option>"));
        assert!(select.contains("<option value=\"info\" selected>info</option>"));
        assert!(select.contains("id=\"node-testnet10-logLevel\""));

        let check = card_check_html("mainnet", "yes", "<Yes>", true, false);
        assert!(check.contains("type=\"checkbox\" checked"));
        assert!(check.contains("<span>&lt;Yes&gt;</span>"));
    }

    #[test]
    fn log_auto_scroll_keys_match_legacy_contract() {
        assert_eq!(
            log_auto_scroll_key_text("mainnet"),
            "kgw.node.log.autoscroll.mainnet"
        );
        assert_eq!(
            log_auto_scroll_key_text("testnet10"),
            "kgw.node.log.autoscroll.testnet10"
        );
    }

    #[test]
    fn escape_and_ids_match_legacy_contract() {
        assert_eq!(
            escape_html_text("<a x=\"1\">&"),
            "&lt;a x=&quot;1&quot;&gt;&amp;"
        );
        assert_eq!(
            node_element_id("mainnet".to_owned(), "rpcPort".to_owned()),
            "node-mainnet-rpcPort"
        );
    }
}
