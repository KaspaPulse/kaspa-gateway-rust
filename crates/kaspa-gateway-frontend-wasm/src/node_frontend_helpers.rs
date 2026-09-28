use super::settings_contract::{
    endpoint as settings_endpoint, node_field_enabled as settings_node_field_enabled,
    render_field_errors as settings_render_field_errors,
    validate_node_form as settings_validate_node_form,
};
use super::settings_layout::{
    decorate_fields as settings_decorate_fields, reveal_field as settings_reveal_field,
    set_field_state as settings_set_field_state,
};
use super::settings_schema::{
    NODE_DANGEROUS, NODE_ENDPOINTS, NODE_MANAGED, NODE_OPTIONAL, NODE_REQUIRED,
};
use js_sys::{Array, Error, Function, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

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

#[derive(Clone, Copy)]
struct IsolatedCompatRoute {
    command: &'static str,
    unavailable: &'static str,
    success: &'static str,
    failure: &'static str,
    include_command_in_log: bool,
}

fn isolated_compat_route(family: &str, action: &str) -> Option<IsolatedCompatRoute> {
    let (command, unavailable, success, failure, include_command_in_log) = match (family, action) {
        ("preview", "status") => (
            "rk_isolated_adapter_status_preview_v1",
            "[kaspa-node] isolated adapter preview unavailable: Tauri invoke not found",
            "[kaspa-node] isolated adapter preview:",
            "[kaspa-node] isolated adapter preview failed:",
            false,
        ),
        ("final", "start") => (
            "rk_final_isolated_adapter_start_v1",
            "[kaspa-node] final isolated runtime unavailable: Tauri invoke not found",
            "[kaspa-node] final isolated runtime:",
            "[kaspa-node] final isolated runtime failed:",
            true,
        ),
        ("final", "status") => (
            "rk_final_isolated_adapter_status_v1",
            "[kaspa-node] final isolated runtime unavailable: Tauri invoke not found",
            "[kaspa-node] final isolated runtime:",
            "[kaspa-node] final isolated runtime failed:",
            true,
        ),
        ("final", "stop") => (
            "rk_final_isolated_adapter_stop_v1",
            "[kaspa-node] final isolated runtime unavailable: Tauri invoke not found",
            "[kaspa-node] final isolated runtime:",
            "[kaspa-node] final isolated runtime failed:",
            true,
        ),
        ("v66", "policy") | ("v67", "policy") => (
            "rk_v66_runtime_feature_policy_v1",
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime unavailable: Tauri invoke not found"
            } else {
                "[kaspa-node] V67 runtime unavailable: Tauri invoke not found"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime:"
            } else {
                "[kaspa-node] V67 runtime:"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime failed:"
            } else {
                "[kaspa-node] V67 runtime failed:"
            },
            true,
        ),
        ("v66", "start") | ("v67", "start") => (
            "rk_v66_isolated_adapter_start_v1",
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime unavailable: Tauri invoke not found"
            } else {
                "[kaspa-node] V67 runtime unavailable: Tauri invoke not found"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime:"
            } else {
                "[kaspa-node] V67 runtime:"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime failed:"
            } else {
                "[kaspa-node] V67 runtime failed:"
            },
            true,
        ),
        ("v66", "status") | ("v67", "status") => (
            "rk_v66_isolated_adapter_status_v1",
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime unavailable: Tauri invoke not found"
            } else {
                "[kaspa-node] V67 runtime unavailable: Tauri invoke not found"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime:"
            } else {
                "[kaspa-node] V67 runtime:"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime failed:"
            } else {
                "[kaspa-node] V67 runtime failed:"
            },
            true,
        ),
        ("v66", "stop") | ("v67", "stop") => (
            "rk_v66_isolated_adapter_stop_v1",
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime unavailable: Tauri invoke not found"
            } else {
                "[kaspa-node] V67 runtime unavailable: Tauri invoke not found"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime:"
            } else {
                "[kaspa-node] V67 runtime:"
            },
            if family == "v66" {
                "[kaspa-node] V66 isolated runtime failed:"
            } else {
                "[kaspa-node] V67 runtime failed:"
            },
            true,
        ),
        _ => return None,
    };
    Some(IsolatedCompatRoute {
        command,
        unavailable,
        success,
        failure,
        include_command_in_log,
    })
}

fn isolated_compat_invoke_function() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    for candidate in [
        property(&property(&tauri, "core"), "invoke"),
        property(&property(&tauri, "tauri"), "invoke"),
        property(&win, "__TAURI_IPC__"),
    ] {
        if let Ok(invoke) = candidate.dyn_into::<Function>() {
            return Some(invoke);
        }
    }
    None
}

fn console_one(method: &str, message: &str) {
    let console = property(&global(), "console");
    if let Some(callback) = function(&console, method) {
        let _ = callback.call1(&console, &JsValue::from_str(message));
    }
}

fn console_value(method: &str, message: &str, value: &JsValue) {
    let console = property(&global(), "console");
    if let Some(callback) = function(&console, method) {
        let _ = callback.call2(&console, &JsValue::from_str(message), value);
    }
}

fn console_command_value(method: &str, message: &str, command: &str, value: &JsValue) {
    let console = property(&global(), "console");
    if let Some(callback) = function(&console, method) {
        let _ = callback.call3(
            &console,
            &JsValue::from_str(message),
            &JsValue::from_str(command),
            value,
        );
    }
}

fn isolated_compat_payload(network: JsValue, app_dir_name: Option<JsValue>) -> JsValue {
    let payload = Object::new();
    set(payload.as_ref(), "network", &network);
    if let Some(app_dir_name) = app_dir_name {
        set(payload.as_ref(), "appDirName", &app_dir_name);
    }
    payload.into()
}

async fn isolated_compat_invoke(route: IsolatedCompatRoute, payload: JsValue) -> JsValue {
    let Some(invoke) = isolated_compat_invoke_function() else {
        console_one("warn", route.unavailable);
        return JsValue::NULL;
    };
    let result = match invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str(route.command),
        &payload,
    ) {
        Ok(value) => value,
        Err(error) => {
            if route.include_command_in_log {
                console_command_value("warn", route.failure, route.command, &error);
            } else {
                console_value("warn", route.failure, &error);
            }
            return JsValue::NULL;
        }
    };
    match JsFuture::from(Promise::resolve(&result)).await {
        Ok(value) => {
            if route.include_command_in_log {
                console_command_value("log", route.success, route.command, &value);
            } else {
                console_value("log", route.success, &value);
            }
            value
        }
        Err(error) => {
            if route.include_command_in_log {
                console_command_value("warn", route.failure, route.command, &error);
            } else {
                console_value("warn", route.failure, &error);
            }
            JsValue::NULL
        }
    }
}

async fn isolated_compat_call(
    family: &str,
    action: &str,
    network: JsValue,
    app_dir_name: Option<JsValue>,
) -> JsValue {
    let Some(route) = isolated_compat_route(family, action) else {
        return JsValue::NULL;
    };
    isolated_compat_invoke(route, isolated_compat_payload(network, app_dir_name)).await
}

#[wasm_bindgen(js_name = nodeSuperMegaIsolatedAdapterStatusPreviewV1)]
pub async fn node_super_mega_isolated_adapter_status_preview_v1(network: JsValue) -> JsValue {
    isolated_compat_call("preview", "status", network, None).await
}

#[wasm_bindgen(js_name = nodeFinalIsolatedAdapterStartV1)]
pub async fn node_final_isolated_adapter_start_v1(
    network: JsValue,
    app_dir_name: JsValue,
) -> JsValue {
    isolated_compat_call("final", "start", network, Some(app_dir_name)).await
}

#[wasm_bindgen(js_name = nodeFinalIsolatedAdapterStatusV1)]
pub async fn node_final_isolated_adapter_status_v1(network: JsValue) -> JsValue {
    isolated_compat_call("final", "status", network, None).await
}

#[wasm_bindgen(js_name = nodeFinalIsolatedAdapterStopV1)]
pub async fn node_final_isolated_adapter_stop_v1(network: JsValue) -> JsValue {
    isolated_compat_call("final", "stop", network, None).await
}

#[wasm_bindgen(js_name = nodeV66RuntimeFeaturePolicyV1)]
pub async fn node_v66_runtime_feature_policy_v1(network: JsValue) -> JsValue {
    isolated_compat_call("v66", "policy", network, None).await
}

#[wasm_bindgen(js_name = nodeV66IsolatedAdapterStartV1)]
pub async fn node_v66_isolated_adapter_start_v1(
    network: JsValue,
    app_dir_name: JsValue,
) -> JsValue {
    isolated_compat_call("v66", "start", network, Some(app_dir_name)).await
}

#[wasm_bindgen(js_name = nodeV66IsolatedAdapterStatusV1)]
pub async fn node_v66_isolated_adapter_status_v1(network: JsValue) -> JsValue {
    isolated_compat_call("v66", "status", network, None).await
}

#[wasm_bindgen(js_name = nodeV66IsolatedAdapterStopV1)]
pub async fn node_v66_isolated_adapter_stop_v1(network: JsValue) -> JsValue {
    isolated_compat_call("v66", "stop", network, None).await
}

#[wasm_bindgen(js_name = nodeV67StartRuntime)]
pub async fn node_v67_start_runtime(network: JsValue, app_dir_name: JsValue) -> JsValue {
    isolated_compat_call("v67", "start", network, Some(app_dir_name)).await
}

#[wasm_bindgen(js_name = nodeV67StatusRuntime)]
pub async fn node_v67_status_runtime(network: JsValue) -> JsValue {
    isolated_compat_call("v67", "status", network, None).await
}

#[wasm_bindgen(js_name = nodeV67StopRuntime)]
pub async fn node_v67_stop_runtime(network: JsValue) -> JsValue {
    isolated_compat_call("v67", "stop", network, None).await
}

#[wasm_bindgen(js_name = nodeV67RuntimeFeaturePolicy)]
pub async fn node_v67_runtime_feature_policy(network: JsValue) -> JsValue {
    isolated_compat_call("v67", "policy", network, None).await
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

fn schema_pair_value<'a>(items: &'a [(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    items
        .iter()
        .find_map(|(key, value)| (*key == name).then_some(*value))
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

fn node_form_values(net: &str) -> JsValue {
    let values = Object::new();
    let panel = r51_panel(net);
    if !present(&panel) {
        return values.into();
    }
    let Some(list) = call1(
        &panel,
        "querySelectorAll",
        &JsValue::from_str(".node-v6-card input[id], .node-v6-card select[id]"),
    ) else {
        return values.into();
    };
    let length = crate::js_number(&property(&list, "length"));
    let prefix = format!("node-{net}-");
    for index in 0..length.max(0.0) as u32 {
        let Ok(field) = Reflect::get(&list, &JsValue::from_f64(index as f64)) else {
            continue;
        };
        let id = crate::js_string_owned(&property(&field, "id"));
        let Some(name) = id.strip_prefix(&prefix) else {
            continue;
        };
        let value = if crate::js_string_owned(&property(&field, "type")) == "checkbox" {
            JsValue::from_bool(crate::js_boolean(&property(&field, "checked")))
        } else {
            property(&field, "value")
        };
        set(values.as_ref(), name, &value);
    }
    values.into()
}

fn node_preview_message_inner(net: &str, message: &str, error: bool) -> bool {
    let element = node_by_id(node_element_id(net.to_owned(), "previewMessage".to_owned()));
    if !present(&element) {
        return false;
    }
    set(&element, "textContent", &JsValue::from_str(message));
    let class_list = property(&element, "classList");
    let _ = call2(
        &class_list,
        "toggle",
        &JsValue::from_str("kgw-field-error"),
        &JsValue::from_bool(error),
    );
    let tone = if error {
        "error"
    } else if message.starts_with("Validating") {
        "validating"
    } else {
        "verified"
    };
    let _ = crate::apply_status_tone_js(element, JsValue::from_str(tone));
    true
}

fn node_sync_dependencies_inner(net: &str, locked: bool) -> bool {
    let values = node_form_values(net);
    let options = command_state_object(net);
    let panel = r51_panel(net);
    if !present(&panel) {
        return false;
    }

    let keys = Object::keys(&Object::from(values.clone()));
    for key in keys.iter() {
        let name = crate::js_string_owned(&key);
        let field = node_by_id(node_element_id(net.to_owned(), name.clone()));
        if !present(&field) {
            continue;
        }
        let managed = schema_pair_value(NODE_MANAGED, &name);
        let experimental_only = name == "enableUnsyncedMining" && net == "mainnet";
        let active = settings_node_field_enabled(name.clone(), values.clone(), options.clone())
            && !experimental_only;

        set(
            &field,
            "disabled",
            &JsValue::from_bool(locked || (!active && name != "appDir")),
        );
        set(
            &field,
            "readOnly",
            &JsValue::from_bool(locked || managed.is_some()),
        );

        let title = if locked {
            "Stop the bridge that owns this node to edit settings.".to_owned()
        } else if let Some(message) = managed {
            message.to_owned()
        } else if experimental_only {
            "Available only on test networks.".to_owned()
        } else if !active {
            "Enable the parent option to use this value.".to_owned()
        } else {
            crate::js_string_owned(&property(&field, "value"))
        };
        set(&field, "title", &JsValue::from_str(&title));

        if let Some(card) = call1(&field, "closest", &JsValue::from_str(".node-v6-card")) {
            let class_list = property(&card, "classList");
            let _ = call2(
                &class_list,
                "toggle",
                &JsValue::from_str("kgw-field-inactive"),
                &JsValue::from_bool(!active),
            );
        }

        let state = if let Some(message) = managed {
            if message.to_ascii_lowercase().contains("unsupported") {
                "Unsupported".to_owned()
            } else {
                "Managed".to_owned()
            }
        } else if schema_pair_contains(NODE_DANGEROUS, &name) {
            "Dangerous".to_owned()
        } else if let Some(required) = schema_pair_value(NODE_REQUIRED, &name) {
            if form_text(&values, &name) == required {
                "KGW default".to_owned()
            } else {
                "Custom value".to_owned()
            }
        } else if experimental_only {
            "Test networks only".to_owned()
        } else if !active {
            "Not active".to_owned()
        } else {
            String::new()
        };
        let _ = settings_set_field_state(field, state);
    }

    if let Some(toggles) = call1(
        &panel,
        "querySelectorAll",
        &JsValue::from_str("[data-node-command-option-toggle-r7]"),
    ) {
        let length = crate::js_number(&property(&toggles, "length"));
        for index in 0..length.max(0.0) as u32 {
            let Ok(toggle) = Reflect::get(&toggles, &JsValue::from_f64(index as f64)) else {
                continue;
            };
            let name = crate::js_string_owned(&property(
                &property(&toggle, "dataset"),
                "nodeCommandOptionToggleR7",
            ));
            let disabled = locked
                || (name == "logDir" && form_checked(&values, "noLogFiles"))
                || (name == "perfMetricsInterval" && !form_checked(&values, "perfMetrics"))
                || (name == "rocksDbCacheSize"
                    && (!crate::js_boolean(&property(&options, "rocksDbPreset"))
                        || form_text(&values, "rocksDbPreset") != "hdd"));
            set(&toggle, "disabled", &JsValue::from_bool(disabled));
        }
    }

    let _ = settings_decorate_fields(panel);
    true
}

fn panel_start_from_monitor_inner(net: &str) -> bool {
    let panel = r51_panel(net);
    let start = query(&panel, r#"[data-node-action="start"]"#);
    if !present(&start) {
        return false;
    }
    if crate::js_boolean(&property(&start, "disabled")) {
        let settings_tab = query(&panel, r#"[data-node-inner-tab="settings"]"#);
        if let Some(click) = function(&settings_tab, "click") {
            let _ = click.call0(&settings_tab);
        }
        let title = crate::js_string_owned(&property(&start, "title"));
        let message = if title.is_empty() {
            "Check the profile and settings before starting."
        } else {
            &title
        };
        let _ = node_preview_message_inner(net, message, true);
    } else if let Some(click) = function(&start, "click") {
        let _ = click.call0(&start);
    }
    true
}

fn form_text(values: &JsValue, name: &str) -> String {
    crate::js_string_owned(&property(values, name))
        .trim()
        .to_owned()
}

fn form_checked(values: &JsValue, name: &str) -> bool {
    crate::js_boolean(&property(values, name))
}

fn first_node_validation_error(net: &str, values: &JsValue, options: &JsValue) -> Option<String> {
    let errors = settings_validate_node_form(values.clone(), options.clone(), net.to_owned());
    if !errors.is_object() || errors.is_null() {
        return None;
    }
    let keys = Object::keys(&Object::from(errors.clone()));
    if keys.length() == 0 {
        return None;
    }
    let key = crate::js_string_owned(&keys.get(0));
    let message = crate::js_string_owned(&property(&errors, &key));
    (!message.is_empty()).then_some(message)
}

fn effective_number_value(
    name: &str,
    fallback: JsValue,
    integer: bool,
    values: &JsValue,
    options: &JsValue,
) -> Result<JsValue, JsValue> {
    if !settings_node_field_enabled(name.to_owned(), values.clone(), options.clone()) {
        return Ok(fallback);
    }
    let raw = form_text(values, name);
    if raw.is_empty() {
        return Ok(fallback);
    }
    let number = crate::js_number(&JsValue::from_str(&raw));
    if !number.is_finite() || (integer && number.fract() != 0.0) {
        let kind = if integer {
            "an integer"
        } else {
            "a finite number"
        };
        return Err(Error::new(&format!("{name} must be {kind}.")).into());
    }
    Ok(JsValue::from_f64(number))
}

fn effective_endpoint_value(
    enabled_name: &str,
    host_name: &str,
    port_name: &str,
    values: &JsValue,
) -> Result<JsValue, JsValue> {
    if !form_checked(values, enabled_name) {
        return Ok(JsValue::NULL);
    }
    let host = form_text(values, host_name);
    let port = form_text(values, port_name);
    if host.is_empty() && port.is_empty() {
        return Err(Error::new(&format!("{enabled_name} requires a host and port.")).into());
    }
    let port_number = crate::js_number(&JsValue::from_str(&port));
    if host.is_empty()
        || port.is_empty()
        || !port.bytes().all(|byte| byte.is_ascii_digit())
        || !port_number.is_finite()
        || !(1.0..=65_535.0).contains(&port_number)
    {
        return Err(Error::new(&format!(
            "{enabled_name} requires a host and a port between 1 and 65535."
        ))
        .into());
    }
    Ok(JsValue::from_str(&settings_endpoint(
        JsValue::from_str(&host),
        JsValue::from_str(&port),
    )))
}

fn optional_text_value(text: String) -> JsValue {
    if text.is_empty() {
        JsValue::NULL
    } else {
        JsValue::from_str(&text)
    }
}

fn text_array(value: Option<String>) -> JsValue {
    let array = Array::new();
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        array.push(&JsValue::from_str(&value));
    }
    array.into()
}

fn node_effective_node_settings_inner(net: &str) -> Result<JsValue, JsValue> {
    let values = node_form_values(net);
    let options = command_state_object(net);

    if let Some(message) = first_node_validation_error(net, &values, &options) {
        return Err(Error::new(&message).into());
    }

    let rpc_listen = effective_endpoint_value(
        "rpcListenEnabled",
        "rpcListenHost",
        "rpcListenPort",
        &values,
    )?;
    if rpc_listen.is_null() || rpc_listen.is_undefined() {
        return Err(
            Error::new("The managed desktop owner requires gRPC RPC to remain enabled.").into(),
        );
    }

    if !form_text(&values, "configFile").is_empty() {
        return Err(Error::new(
            "--configfile is not supported by the managed desktop owner because network and database ownership must remain authoritative.",
        )
        .into());
    }
    if !form_text(&values, "overrideParamsFile").is_empty() {
        return Err(Error::new(
            "--override-params-file is not supported because the desktop owns the selected network identity.",
        )
        .into());
    }
    if form_checked(&values, "noLogFiles")
        && settings_node_field_enabled("logDir".to_owned(), values.clone(), options.clone())
        && !form_text(&values, "logDir").is_empty()
    {
        return Err(Error::new("--logdir and --nologfiles cannot be used together.").into());
    }

    let connect =
        effective_endpoint_value("connectEnabled", "connectHost", "connectPort", &values)?;
    let add_peer =
        effective_endpoint_value("addPeerEnabled", "addPeerHost", "addPeerPort", &values)?;
    let p2p_listen =
        effective_endpoint_value("listenEnabled", "listenHost", "listenPort", &values)?;
    let external_ip = effective_endpoint_value(
        "externalIpEnabled",
        "externalIpHost",
        "externalIpPort",
        &values,
    )?;
    let rpc_borsh =
        effective_endpoint_value("rpcBorshEnabled", "rpcBorshHost", "rpcBorshPort", &values)?;
    let rpc_json =
        effective_endpoint_value("rpcJsonEnabled", "rpcJsonHost", "rpcJsonPort", &values)?;

    let output = Object::new();
    let log_level = if command_option_enabled_inner(net, "logLevel") {
        let value = form_text(&values, "logLevel");
        if value.is_empty() {
            "info".to_owned()
        } else {
            value
        }
    } else {
        "info".to_owned()
    };
    set(output.as_ref(), "logLevel", &JsValue::from_str(&log_level));
    set(
        output.as_ref(),
        "asyncThreads",
        &effective_number_value(
            "asyncThreads",
            JsValue::from_f64(16.0),
            true,
            &values,
            &options,
        )?,
    );
    set(
        output.as_ref(),
        "ramScale",
        &effective_number_value("ramScale", JsValue::from_f64(1.0), false, &values, &options)?,
    );
    for (key, source) in [
        ("yes", "yes"),
        ("noLogFiles", "noLogFiles"),
        ("sanity", "sanity"),
        ("disableUpnp", "disableUpnp"),
        ("disableDnsSeeding", "noDnsSeed"),
        ("unsafeRpc", "unsafeRpc"),
        ("disableGrpc", "noGrpc"),
        ("utxoIndex", "utxoIndex"),
        ("archival", "archival"),
        ("resetDb", "resetDb"),
        ("perfMetrics", "perfMetrics"),
    ] {
        set(
            output.as_ref(),
            key,
            &JsValue::from_bool(form_checked(&values, source)),
        );
    }
    let testnet = profile(net).is_some_and(|profile| profile.testnet);
    set(
        output.as_ref(),
        "enableUnsyncedMining",
        &JsValue::from_bool(form_checked(&values, "enableUnsyncedMining") && testnet),
    );
    set(output.as_ref(), "p2pListen", &p2p_listen);
    set(output.as_ref(), "externalIp", &external_ip);
    let ua_comment = command_option_enabled_inner(net, "uaComment")
        .then(|| form_text(&values, "uaComment"))
        .filter(|value| !value.is_empty());
    set(
        output.as_ref(),
        "userAgentComments",
        &text_array(ua_comment),
    );
    set(output.as_ref(), "rpcListen", &rpc_listen);
    set(output.as_ref(), "rpcListenBorsh", &rpc_borsh);
    set(output.as_ref(), "rpcListenJson", &rpc_json);
    set(
        output.as_ref(),
        "rpcMaxClients",
        &effective_number_value(
            "rpcMaxClients",
            JsValue::from_f64(16.0),
            true,
            &values,
            &options,
        )?,
    );
    set(
        output.as_ref(),
        "connectPeers",
        &text_array((!connect.is_null()).then(|| crate::js_string_owned(&connect))),
    );
    set(
        output.as_ref(),
        "addPeers",
        &text_array((!add_peer.is_null()).then(|| crate::js_string_owned(&add_peer))),
    );
    set(
        output.as_ref(),
        "outboundTarget",
        &effective_number_value("outPeers", JsValue::from_f64(8.0), true, &values, &options)?,
    );
    set(
        output.as_ref(),
        "inboundLimit",
        &effective_number_value(
            "maxInPeers",
            JsValue::from_f64(32.0),
            true,
            &values,
            &options,
        )?,
    );
    set(
        output.as_ref(),
        "maxTrackedAddresses",
        &effective_number_value(
            "maxTrackedAddresses",
            JsValue::from_f64(0.0),
            true,
            &values,
            &options,
        )?,
    );

    let retention = if command_option_enabled_inner(net, "retentionDays")
        && !form_text(&values, "retentionDays").is_empty()
    {
        effective_number_value("retentionDays", JsValue::NULL, false, &values, &options)?
    } else {
        JsValue::NULL
    };
    set(output.as_ref(), "retentionPeriodDays", &retention);
    set(
        output.as_ref(),
        "perfMetricsIntervalSec",
        &effective_number_value(
            "perfMetricsInterval",
            JsValue::from_f64(10.0),
            true,
            &values,
            &options,
        )?,
    );

    let rocks_preset = if command_option_enabled_inner(net, "rocksDbPreset") {
        optional_text_value(form_text(&values, "rocksDbPreset"))
    } else {
        JsValue::NULL
    };
    set(output.as_ref(), "rocksDbPreset", &rocks_preset);

    let rocks_cache = if settings_node_field_enabled(
        "rocksDbCacheSize".to_owned(),
        values.clone(),
        options.clone(),
    ) && !form_text(&values, "rocksDbCacheSize").is_empty()
    {
        effective_number_value("rocksDbCacheSize", JsValue::NULL, true, &values, &options)?
    } else {
        JsValue::NULL
    };
    set(output.as_ref(), "rocksDbCacheSize", &rocks_cache);

    let rocks_wal = if command_option_enabled_inner(net, "rocksDbWalDir") {
        optional_text_value(form_text(&values, "rocksDbWalDir"))
    } else {
        JsValue::NULL
    };
    set(output.as_ref(), "rocksDbWalDir", &rocks_wal);
    set(output.as_ref(), "overrideParamsFile", &JsValue::NULL);

    let log_dir =
        if settings_node_field_enabled("logDir".to_owned(), values.clone(), options.clone()) {
            optional_text_value(form_text(&values, "logDir"))
        } else {
            JsValue::NULL
        };
    set(output.as_ref(), "logDir", &log_dir);

    Ok(output.into())
}

fn node_validate_form_inner(net: &str, focus: bool) -> JsValue {
    let values = node_form_values(net);
    let options = command_state_object(net);
    let errors = settings_validate_node_form(values, options, net.to_owned());
    let panel = r51_panel(net);
    let _ = settings_render_field_errors(panel.clone(), format!("node-{net}-"), errors.clone());

    let keys = if errors.is_object() && !errors.is_null() {
        Object::keys(&Object::from(errors.clone()))
    } else {
        Array::new()
    };
    if focus && keys.length() > 0 {
        let name = crate::js_string_owned(&keys.get(0));
        let field = node_by_id(node_element_id(net.to_owned(), name));
        let settings_tab = query(&panel, r#"[data-node-inner-tab="settings"]"#);
        if let Some(click) = function(&settings_tab, "click") {
            let _ = click.call0(&settings_tab);
        }

        let section = call1(
            &field,
            "closest",
            &JsValue::from_str("[data-node-section-panel]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        let section_name = crate::js_string_owned(&property(
            &property(&section, "dataset"),
            "nodeSectionPanel",
        ));
        if !section_name.is_empty() {
            let section_tab = query(
                &panel,
                &format!(r#"[data-node-section-tab="{section_name}"]"#),
            );
            if let Some(click) = function(&section_tab, "click") {
                let _ = click.call0(&section_tab);
            }
        }
        settings_reveal_field(field.clone());
        if let Some(focus_fn) = function(&field, "focus") {
            let _ = focus_fn.call0(&field);
        }
    }
    errors
}

fn runtime_notice_state_key(value: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            out.push(ch);
            pending_dash = false;
        } else if !out.is_empty() {
            pending_dash = true;
        }
    }
    if out.is_empty() {
        "stopped".to_owned()
    } else {
        out
    }
}

fn set_runtime_notice_inner(
    net: &str,
    state: &JsValue,
    evidence: &JsValue,
    error_text: &JsValue,
    error_source: &JsValue,
) -> bool {
    let normalized_state = {
        let value = crate::js_string_owned(state);
        if value.is_empty() {
            "Stopped".to_owned()
        } else {
            value
        }
    };
    let state_key = runtime_notice_state_key(&normalized_state);

    let status = node_by_id(node_element_id(net.to_owned(), "runtimeStatus".to_owned()));
    if present(&status) {
        set(
            &status,
            "textContent",
            &JsValue::from_str(&normalized_state),
        );
        set(
            &property(&status, "dataset"),
            "state",
            &JsValue::from_str(&state_key),
        );
        let _ = crate::apply_status_tone_js(status, JsValue::from_str(&state_key));
    }

    let evidence_node = node_by_id(node_element_id(
        net.to_owned(),
        "runtimeEvidence".to_owned(),
    ));
    if present(&evidence_node) {
        let evidence_text = crate::js_string_owned(evidence);
        set(
            &evidence_node,
            "textContent",
            &JsValue::from_str(if evidence_text.is_empty() {
                "No process owner"
            } else {
                &evidence_text
            }),
        );
    }

    let error_node = node_by_id(node_element_id(net.to_owned(), "runtimeError".to_owned()));
    if present(&error_node) && !error_text.is_null() && !error_text.is_undefined() {
        let text = crate::js_string_owned(error_text).trim().to_owned();
        set(&error_node, "textContent", &JsValue::from_str(&text));
        set(&error_node, "hidden", &JsValue::from_bool(text.is_empty()));
        set(
            &property(&error_node, "dataset"),
            "runtimeErrorSource",
            &JsValue::from_str(&crate::js_string_owned(error_source)),
        );
        let _ = crate::apply_status_tone_js(error_node, JsValue::from_str("error"));
    }
    true
}

fn mark_restart_required_inner(net: &str) -> bool {
    let authority = node_by_id(node_element_id(
        net.to_owned(),
        "settingsAuthority".to_owned(),
    ));
    if !present(&authority) {
        return false;
    }
    let status = node_by_id(node_element_id(net.to_owned(), "runtimeStatus".to_owned()));
    let running =
        crate::js_string_owned(&property(&property(&status, "dataset"), "state")) == "running";
    set(
        &authority,
        "textContent",
        &JsValue::from_str(if running {
            "Restart required to apply changed effective settings"
        } else {
            "Effective settings apply on next Start"
        }),
    );
    set(
        &property(&authority, "dataset"),
        "restartRequired",
        &JsValue::from_str(if running { "true" } else { "false" }),
    );
    true
}

#[wasm_bindgen(js_name = nodePreviewMessage)]
pub fn node_preview_message(net: String, message: String, error: bool) -> bool {
    node_preview_message_inner(&net, &message, error)
}

#[wasm_bindgen(js_name = nodeSetRuntimeNotice)]
pub fn node_set_runtime_notice(
    net: String,
    state: JsValue,
    evidence: JsValue,
    error_text: JsValue,
    error_source: JsValue,
) -> bool {
    set_runtime_notice_inner(&net, &state, &evidence, &error_text, &error_source)
}

#[wasm_bindgen(js_name = nodeMarkRestartRequired)]
pub fn node_mark_restart_required(net: String) -> bool {
    mark_restart_required_inner(&net)
}

#[wasm_bindgen(js_name = nodeSyncDependencies)]
pub fn node_sync_dependencies(net: String, locked: bool) -> bool {
    node_sync_dependencies_inner(&net, locked)
}

#[wasm_bindgen(js_name = nodePanelStartFromMonitor)]
pub fn node_panel_start_from_monitor(net: String) -> bool {
    panel_start_from_monitor_inner(&net)
}

#[wasm_bindgen(js_name = nodeValidateForm)]
pub fn node_validate_form(net: String, focus: bool) -> JsValue {
    node_validate_form_inner(&net, focus)
}

#[wasm_bindgen(js_name = nodeRequireValidSettings)]
pub fn node_require_valid_settings(net: String) -> Result<(), JsValue> {
    let errors = node_validate_form_inner(&net, true);
    let values = if errors.is_object() && !errors.is_null() {
        Object::values(&Object::from(errors))
    } else {
        Array::new()
    };
    if values.length() > 0 {
        return Err(Error::new(&crate::js_string_owned(&values.get(0))).into());
    }
    let _ = node_effective_node_settings_inner(&net)?;
    Ok(())
}

#[wasm_bindgen(js_name = nodeEffectiveNodeSettings)]
pub fn node_effective_node_settings(net: String) -> Result<JsValue, JsValue> {
    let _ = node_validate_form_inner(&net, false);
    node_effective_node_settings_inner(&net)
}

#[wasm_bindgen(js_name = nodeRuntimeArgs)]
pub fn node_runtime_args(net: String, command: String) -> Result<JsValue, JsValue> {
    let output = Object::new();
    set(output.as_ref(), "network", &JsValue::from_str(&net));

    if command == "kgw_kgw_apply_node_settings_v1" {
        let preview = property(
            &node_by_id(node_element_id(net.clone(), "commandPreview".to_owned())),
            "value",
        );
        set(
            output.as_ref(),
            "nodeKind",
            &JsValue::from_str("integrated-as-daemon"),
        );
        set(output.as_ref(), "bridgeKind", &JsValue::from_str("disable"));
        set(
            output.as_ref(),
            "nodeCommandPreview",
            &JsValue::from_str(&crate::js_string_owned(&preview)),
        );
        set(
            output.as_ref(),
            "bridgeCommandPreview",
            &JsValue::from_str(""),
        );
        set(
            output.as_ref(),
            "effectiveNodeSettings",
            &node_effective_node_settings_inner(&net)?,
        );
        set(output.as_ref(), "runtimeRole", &JsValue::from_str("node"));
        set(
            output.as_ref(),
            "experimentalNetworkOptIn",
            &JsValue::from_bool(net == "testnet13" && node_network_enabled(net.clone())),
        );
    } else if matches!(
        command.as_str(),
        "kgw_kgw_disable_network_v1" | "kgw_runtime_owner_status_v1" | "kgw_kgw_runtime_logs_v1"
    ) {
        set(output.as_ref(), "runtimeRole", &JsValue::from_str("node"));
    }

    Ok(output.into())
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

fn r51_object_key_count(value: &JsValue) -> u32 {
    if !value.is_object() || value.is_null() {
        return 0;
    }
    Object::keys(&Object::from(value.clone())).length()
}

fn r51_settings_summary(values: &JsValue) -> JsValue {
    let result = Object::new();
    let mut checkbox_count = 0_u32;
    let mut value_count = 0_u32;
    if values.is_object() && !values.is_null() {
        for entry in Object::entries(&Object::from(values.clone())).iter() {
            let pair = Array::from(&entry);
            if pair.length() < 2 {
                continue;
            }
            match crate::js_string_owned(&property(&pair.get(1), "type")).as_str() {
                "checkbox" => checkbox_count += 1,
                "value" => value_count += 1,
                _ => {}
            }
        }
    }
    let structured = property(values, "__kgwBridgeStructuredInstancesR26B");
    let instances = property(&structured, "instances");
    set(
        result.as_ref(),
        "keyCount",
        &JsValue::from_f64(f64::from(r51_object_key_count(values))),
    );
    set(
        result.as_ref(),
        "checkboxCount",
        &JsValue::from_f64(f64::from(checkbox_count)),
    );
    set(
        result.as_ref(),
        "valueCount",
        &JsValue::from_f64(f64::from(value_count)),
    );
    set(
        result.as_ref(),
        "structuredInstanceCount",
        &JsValue::from_f64(if Array::is_array(&instances) {
            Array::from(&instances).length() as f64
        } else {
            0.0
        }),
    );
    set(
        result.as_ref(),
        "hasActiveStructuredInstance",
        &JsValue::from_bool(crate::js_boolean(&property(
            values,
            "__kgwBridgeActiveInstanceR26B",
        ))),
    );
    result.into()
}

fn r51_persist_action(net: &str, kind: &str) -> Result<JsValue, JsValue> {
    let values = r51_read_settings(net);
    let key = format!("{kind}:{net}");
    r51_store(&key, &values)?;
    let persisted = r51_load(&key);
    let result = Object::from(r51_settings_summary(&values));
    set(result.as_ref(), "storageKey", &JsValue::from_str(&key));
    set(
        result.as_ref(),
        "persisted",
        &JsValue::from_bool(crate::js_boolean(&persisted)),
    );
    set(
        result.as_ref(),
        "persistedKeyCount",
        &JsValue::from_f64(f64::from(r51_object_key_count(&persisted))),
    );
    Ok(result.into())
}

fn r51_restore_action(net: &str) -> JsValue {
    let defaults = {
        let stored = r51_load(&format!("default:{net}"));
        if crate::js_boolean(&stored) {
            stored
        } else {
            r51_load(&format!("factory:{net}"))
        }
    };
    let result = Object::new();
    set(
        result.as_ref(),
        "hasDefaults",
        &JsValue::from_bool(crate::js_boolean(&defaults)),
    );
    set(
        result.as_ref(),
        "defaultKeyCount",
        &JsValue::from_f64(f64::from(r51_object_key_count(&defaults))),
    );
    let write_result = r51_write_settings(net, &defaults);
    set(result.as_ref(), "writeResult", &write_result);
    result.into()
}

fn r51_trace(net: &str, action: &str, phase: &str, details: JsValue) {
    let _ = crate::node_start_trace::node_small_owner_trace(
        JsValue::from_str(net),
        JsValue::from_str(action),
        JsValue::from_str(phase),
        details,
    );
}

fn r51_trace_details() -> Object {
    let details = Object::new();
    set(details.as_ref(), "patch", &JsValue::from_str("R29B"));
    set(
        details.as_ref(),
        "owner",
        &JsValue::from_str("node-r51-settings-owner"),
    );
    details
}

fn r51_persist_trace_spec(
    kind: &str,
) -> Option<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)> {
    match kind {
        "saved" => Some((
            "save-settings",
            "r29b-save-begin",
            "r29b-save-read-settings",
            "r29b-save-complete",
            "savedKey",
        )),
        "default" => Some((
            "set-defaults",
            "r29b-set-defaults-begin",
            "r29b-set-defaults-read-settings",
            "r29b-set-defaults-complete",
            "defaultKey",
        )),
        _ => None,
    }
}

fn r51_read_settings_tracked(net: &str) -> JsValue {
    let values = r51_read_settings(net);
    let details = r51_trace_details();
    let options = property(&values, COMMAND_OPTIONS_KEY);
    let count = if options.is_object() && !options.is_null() {
        Object::keys(&Object::from(options)).length()
    } else {
        0
    };
    set(
        details.as_ref(),
        "commandOptionCount",
        &JsValue::from_f64(f64::from(count)),
    );
    r51_trace(
        net,
        "settings-persistence",
        "r38c-read-settings-command-options",
        details.into(),
    );
    values
}

fn r51_persist_tracked(net: &str, kind: &str) -> Result<JsValue, JsValue> {
    node_require_valid_settings(net.to_owned())?;
    let Some((action, begin_phase, read_phase, complete_phase, key_field)) =
        r51_persist_trace_spec(kind)
    else {
        return Err(Error::new("unsupported R51 persistence kind").into());
    };

    r51_trace(net, action, begin_phase, r51_trace_details().into());
    let result = r51_persist_action(net, kind)?;

    let read = r51_trace_details();
    for key in [
        "keyCount",
        "checkboxCount",
        "valueCount",
        "structuredInstanceCount",
        "hasActiveStructuredInstance",
    ] {
        set(read.as_ref(), key, &property(&result, key));
    }
    r51_trace(net, action, read_phase, read.into());

    let complete = r51_trace_details();
    let storage_key = {
        let value = property(&result, "storageKey");
        if present(&value) {
            crate::js_string_owned(&value)
        } else {
            format!("{kind}:{net}")
        }
    };
    set(
        complete.as_ref(),
        key_field,
        &JsValue::from_str(&storage_key),
    );
    set(
        complete.as_ref(),
        "persisted",
        &property(&result, "persisted"),
    );
    set(
        complete.as_ref(),
        "persistedKeyCount",
        &property(&result, "persistedKeyCount"),
    );
    r51_trace(net, action, complete_phase, complete.into());
    Ok(result)
}

#[wasm_bindgen(js_name = nodeR51ReadSettingsTracked)]
pub fn node_r51_read_settings_tracked(net: String) -> JsValue {
    r51_read_settings_tracked(&net)
}

#[wasm_bindgen(js_name = nodeR51SaveSettings)]
pub fn node_r51_save_settings(net: String) -> Result<JsValue, JsValue> {
    r51_persist_tracked(&net, "saved")
}

#[wasm_bindgen(js_name = nodeR51SetAsDefaults)]
pub fn node_r51_set_as_defaults(net: String) -> Result<JsValue, JsValue> {
    r51_persist_tracked(&net, "default")
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

#[wasm_bindgen(js_name = nodeR51SaveSettingsAction)]
pub fn node_r51_save_settings_action(net: String) -> Result<JsValue, JsValue> {
    r51_persist_action(&net, "saved")
}

#[wasm_bindgen(js_name = nodeR51SetDefaultsAction)]
pub fn node_r51_set_defaults_action(net: String) -> Result<JsValue, JsValue> {
    r51_persist_action(&net, "default")
}

#[wasm_bindgen(js_name = nodeR51RestoreDefaultsAction)]
pub fn node_r51_restore_defaults_action(net: String) -> JsValue {
    r51_restore_action(&net)
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

fn render_input_card(
    net: &str,
    name: &str,
    label: &str,
    value: &str,
    placeholder: &str,
    span2: bool,
) -> String {
    let toggle = command_inline_toggle_html(net, name);
    card_input_html(net, name, label, value, placeholder, span2, &toggle)
}

fn render_select_card(
    net: &str,
    name: &str,
    label: &str,
    options: &[&str],
    value: &str,
    span2: bool,
) -> String {
    let options = options
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    let toggle = command_inline_toggle_html(net, name);
    card_select_html(net, name, label, &options, value, span2, &toggle)
}

fn render_check_card(net: &str, name: &str, label: &str, checked: bool, span2: bool) -> String {
    card_check_html(net, name, label, checked, span2)
}

fn insert_input_card(
    cards: &mut BTreeMap<String, String>,
    net: &str,
    name: &str,
    label: &str,
    value: &str,
    placeholder: &str,
    span2: bool,
) {
    cards.insert(
        name.to_owned(),
        render_input_card(net, name, label, value, placeholder, span2),
    );
}

fn node_render_cards(profile: &NetworkProfile) -> BTreeMap<String, String> {
    let net = profile.key;
    let mut cards = BTreeMap::new();
    if profile.testnet {
        cards.insert(
            "testnet".to_owned(),
            render_check_card(net, "testnet", "--testnet", true, false),
        );
        insert_input_card(
            &mut cards,
            net,
            "netsuffix",
            "--netsuffix",
            profile.netsuffix,
            "required",
            false,
        );
    }
    cards.insert(
        "logLevel".to_owned(),
        render_select_card(
            net,
            "logLevel",
            "--loglevel",
            &["off", "error", "warn", "info", "debug", "trace"],
            "info",
            false,
        ),
    );
    insert_input_card(
        &mut cards,
        net,
        "asyncThreads",
        "--async-threads",
        "16",
        "",
        false,
    );
    insert_input_card(&mut cards, net, "ramScale", "--ram-scale", "1", "", false);
    cards.insert(
        "yes".to_owned(),
        render_check_card(net, "yes", "--yes", true, false),
    );
    cards.insert(
        "noLogFiles".to_owned(),
        render_check_card(net, "noLogFiles", "--nologfiles", true, false),
    );
    cards.insert(
        "sanity".to_owned(),
        render_check_card(net, "sanity", "--sanity", false, false),
    );
    cards.insert(
        "enableUnsyncedMining".to_owned(),
        render_check_card(
            net,
            "enableUnsyncedMining",
            "--enable-unsynced-mining",
            false,
            true,
        ),
    );

    let p2p_port = match net {
        "mainnet" => "16111",
        "testnet10" => "16211",
        _ => "16711",
    };
    cards.insert(
        "listenEnabled".to_owned(),
        render_check_card(net, "listenEnabled", "--listen", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "listenHost",
        "listen host",
        "0.0.0.0",
        "",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "listenPort",
        "listen port",
        p2p_port,
        "",
        false,
    );
    cards.insert(
        "externalIpEnabled".to_owned(),
        render_check_card(net, "externalIpEnabled", "--externalip", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "externalIpHost",
        "external host",
        "",
        "ip",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "externalIpPort",
        "external port",
        "",
        "port",
        false,
    );
    cards.insert(
        "disableUpnp".to_owned(),
        render_check_card(net, "disableUpnp", "--disable-upnp", true, false),
    );
    cards.insert(
        "noDnsSeed".to_owned(),
        render_check_card(net, "noDnsSeed", "--nodnsseed", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "uaComment",
        "--uacomment",
        "",
        "comment",
        true,
    );

    let rpc_base = match net {
        "mainnet" => 16_110,
        _ => 16_210,
    };
    cards.insert(
        "rpcListenEnabled".to_owned(),
        render_check_card(net, "rpcListenEnabled", "--rpclisten", true, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcListenHost",
        "RPC host",
        "127.0.0.1",
        "",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcListenPort",
        "RPC port",
        &rpc_base.to_string(),
        "",
        false,
    );
    cards.insert(
        "rpcBorshEnabled".to_owned(),
        render_check_card(net, "rpcBorshEnabled", "--rpclisten-borsh", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcBorshHost",
        "Borsh host",
        "127.0.0.1",
        "",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcBorshPort",
        "Borsh port",
        &(rpc_base + 1000).to_string(),
        "",
        false,
    );
    cards.insert(
        "rpcJsonEnabled".to_owned(),
        render_check_card(net, "rpcJsonEnabled", "--rpclisten-json", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcJsonHost",
        "JSON host",
        "127.0.0.1",
        "",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcJsonPort",
        "JSON port",
        &(rpc_base + 2000).to_string(),
        "",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "rpcMaxClients",
        "--rpcmaxclients (managed max 16)",
        "16",
        "",
        false,
    );
    cards.insert(
        "unsafeRpc".to_owned(),
        render_check_card(net, "unsafeRpc", "--unsaferpc", false, false),
    );
    cards.insert(
        "noGrpc".to_owned(),
        render_check_card(net, "noGrpc", "--nogrpc", false, false),
    );

    cards.insert(
        "connectEnabled".to_owned(),
        render_check_card(net, "connectEnabled", "--connect", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "connectHost",
        "connect host",
        "",
        "host",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "connectPort",
        "connect port",
        "",
        "port",
        false,
    );
    cards.insert(
        "addPeerEnabled".to_owned(),
        render_check_card(net, "addPeerEnabled", "--addpeer", false, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "addPeerHost",
        "peer host",
        "",
        "host",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "addPeerPort",
        "peer port",
        "",
        "port",
        false,
    );
    insert_input_card(&mut cards, net, "outPeers", "--outpeers", "8", "", false);
    insert_input_card(
        &mut cards,
        net,
        "maxInPeers",
        "--maxinpeers (managed max 32)",
        "32",
        "",
        false,
    );

    cards.insert(
        "utxoIndex".to_owned(),
        render_check_card(net, "utxoIndex", "--utxoindex", true, false),
    );
    cards.insert(
        "archival".to_owned(),
        render_check_card(net, "archival", "--archival", false, false),
    );
    cards.insert(
        "resetDb".to_owned(),
        render_check_card(net, "resetDb", "--reset-db", false, false),
    );
    cards.insert(
        "perfMetrics".to_owned(),
        render_check_card(net, "perfMetrics", "--perf-metrics", true, false),
    );
    insert_input_card(
        &mut cards,
        net,
        "maxTrackedAddresses",
        "--max-tracked-addresses",
        "",
        "0",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "retentionDays",
        "--retention-period-days",
        "",
        "optional",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "perfMetricsInterval",
        "--perf-metrics-interval-sec",
        "",
        "optional",
        true,
    );

    cards.insert(
        "rocksDbPreset".to_owned(),
        render_select_card(
            net,
            "rocksDbPreset",
            "--rocksdb-preset",
            &["", "default", "hdd"],
            "",
            false,
        ),
    );
    insert_input_card(
        &mut cards,
        net,
        "rocksDbCacheSize",
        "--rocksdb-cache-size",
        "",
        "MB",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "rocksDbWalDir",
        "--rocksdb-wal-dir",
        "",
        "path",
        true,
    );
    insert_input_card(
        &mut cards,
        net,
        "overrideParamsFile",
        "--override-params-file (unsupported: managed network)",
        "",
        "not supported",
        true,
    );

    insert_input_card(
        &mut cards,
        net,
        "configFile",
        "--configfile (unsupported: managed ownership)",
        "",
        "not supported",
        false,
    );
    insert_input_card(
        &mut cards,
        net,
        "appDir",
        "--appdir (managed per network)",
        "",
        "managed by desktop",
        false,
    );
    insert_input_card(&mut cards, net, "logDir", "--logdir", "", "log dir", false);
    cards
}

const NODE_RENDER_GROUPS: [(&str, &str, &str, &[&str]); 12] = [
    (
        "general",
        "basic",
        "Basic",
        &["testnet", "netsuffix", "utxoIndex", "yes"],
    ),
    (
        "general",
        "networking",
        "Networking",
        &[
            "listenEnabled",
            "listenHost",
            "listenPort",
            "externalIpEnabled",
            "externalIpHost",
            "externalIpPort",
            "disableUpnp",
            "noDnsSeed",
            "uaComment",
        ],
    ),
    (
        "general",
        "rpc",
        "RPC",
        &["rpcListenEnabled", "rpcListenHost", "rpcListenPort"],
    ),
    (
        "general",
        "performance",
        "Performance",
        &["asyncThreads", "ramScale", "outPeers", "maxInPeers"],
    ),
    (
        "general",
        "storage",
        "Storage",
        &["appDir", "rocksDbPreset", "rocksDbCacheSize"],
    ),
    (
        "general",
        "logging",
        "Logging",
        &["logLevel", "noLogFiles", "logDir"],
    ),
    (
        "advanced",
        "p2p",
        "P2P",
        &[
            "connectEnabled",
            "connectHost",
            "connectPort",
            "addPeerEnabled",
            "addPeerHost",
            "addPeerPort",
        ],
    ),
    (
        "advanced",
        "rpc-advanced",
        "RPC Advanced",
        &[
            "rpcBorshEnabled",
            "rpcBorshHost",
            "rpcBorshPort",
            "rpcJsonEnabled",
            "rpcJsonHost",
            "rpcJsonPort",
            "rpcMaxClients",
            "noGrpc",
        ],
    ),
    (
        "advanced",
        "database",
        "Database",
        &[
            "archival",
            "maxTrackedAddresses",
            "retentionDays",
            "rocksDbWalDir",
            "configFile",
            "sanity",
        ],
    ),
    (
        "advanced",
        "metrics",
        "Metrics",
        &["perfMetrics", "perfMetricsInterval"],
    ),
    (
        "advanced",
        "experimental",
        "Experimental",
        &["overrideParamsFile"],
    ),
    (
        "advanced",
        "dangerous",
        "Dangerous",
        &["resetDb", "unsafeRpc", "enableUnsyncedMining"],
    ),
];

fn render_node_sections(profile: &NetworkProfile) -> Result<String, String> {
    let mut cards = node_render_cards(profile);
    let mut groups = Vec::with_capacity(NODE_RENDER_GROUPS.len());
    for (section, key, label, names) in NODE_RENDER_GROUPS {
        let mut fields = String::new();
        for name in names {
            if let Some(card) = cards.remove(*name) {
                fields.push_str(&card);
            }
        }
        let note = match key {
            "dangerous" => "<p class=\"kgw-danger-warning\">Reset DB removes network data. Unsafe RPC can expose privileged methods. Unsynced mining bypasses synchronization. Existing confirmations and network restrictions still apply.</p>".to_owned(),
            "experimental" => format!(
                "<p class=\"kgw-settings-info\">{}</p>",
                escape_html_text(&policy_message_text(profile.key))
            ),
            _ => String::new(),
        };
        groups.push((
            section,
            key,
            label,
            format!("{note}<div class=\"kgw-settings-grid\">{fields}</div>"),
        ));
    }
    if !cards.is_empty() {
        return Err(format!(
            "Ungrouped node settings: {}",
            cards.keys().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    Ok(crate::settings_layout::render_tabs_native(
        "node",
        profile.key,
        &groups,
    ))
}

fn render_node_network_panel(
    profile: &NetworkProfile,
    index: usize,
    active_inner_tab: &str,
    enabled: bool,
) -> Result<String, String> {
    let net = profile.key;
    let panel_active = if index == 0 { " active" } else { "" };
    let panel_hidden = if index == 0 { "" } else { " hidden" };
    let log_active = active_inner_tab == "log";
    let settings_active = active_inner_tab == "settings";
    let log_class = if log_active { " active" } else { "" };
    let settings_class = if settings_active { " active" } else { "" };
    let log_hidden = if log_active { "" } else { " hidden" };
    let settings_hidden = if settings_active { "" } else { " hidden" };
    let experimental_class = if profile.experimental {
        " is-experimental"
    } else {
        ""
    };
    let experimental_badge = if profile.experimental {
        "<span class=\"kgw-experimental-badge\">Experimental - opt-in required</span>"
    } else {
        ""
    };
    let enabled_attr = if enabled { " checked" } else { "" };
    let policy_message = escape_html_text(&policy_message_text(net));
    let sections = render_node_sections(profile)?;
    let id = |name: &str| format!("node-{net}-{name}");

    Ok(format!(
        r#"
    <div class="node-v6-network-panel{panel_active}" data-node-network-panel="{net}" data-testid="kgw-node-panel-{net}"{panel_hidden}>
      <div class="node-v6-inner-tabs">
        <button type="button" class="node-v6-inner-tab{log_class}" data-net="{net}" data-node-inner-tab="log" data-testid="kgw-node-live-monitor-{net}">Live Node Monitor</button>
        <button type="button" class="node-v6-inner-tab{settings_class}" data-net="{net}" data-node-inner-tab="settings" data-testid="kgw-node-settings-{net}">Settings</button>
      </div>

      <div class="node-v6-inner-panel{settings_class}" data-net="{net}" data-node-inner-panel="settings" data-node-settings-panel="{net}"{settings_hidden}>
        <div class="kgw-settings-scroll">
        <section class="kgw-network-policy{experimental_class}" data-net="{net}" data-testid="kgw-node-policy-{net}">
          <div>
            <strong>{label}</strong>{experimental_badge}
            <span>{policy_message}</span>
          </div>
          <div class="kgw-network-policy-controls">
            <span id="{policy_status}" class="kgw-network-policy-status">Stopped</span>
            <label>
              <input type="checkbox" data-node-network-enabled="{net}" data-testid="kgw-node-policy-enabled-{net}" data-net="{net}"{enabled_attr}>
              Profile enabled
            </label>
          </div>
        </section>

        <section class="node-v6-command kgw-effective-preview">
          <div class="kgw-preview-row">
            <strong>Effective node settings</strong>
            <button type="button" data-settings-preview-toggle aria-expanded="false" aria-controls="{preview_body}">Expand</button>
            <button type="button" class="node-v6-copy" data-node-action="copy-command" data-net="{net}" title="Copy effective settings">Copy command</button>
            <button type="button" data-node-action="copy-path" data-net="{net}">Copy data directory</button>
          </div>
          <p id="{preview_message}" class="kgw-preview-message" role="status" aria-live="polite"></p>
          <div class="kgw-preview-body" id="{preview_body}" hidden>
            <p class="kgw-preview-help">The embedded node library consumes these equivalent arguments inside KaspaGateway self-workers.</p>
            <textarea id="{command_preview}" aria-label="Effective node settings preview" readonly spellcheck="false" wrap="soft"></textarea>
            <details class="kgw-arguments"><summary>Argument list</summary><pre id="{argument_list}"></pre></details>
          </div>
        </section>

        <section class="node-v6-toolbar">
          <div class="node-v6-buttons">
            <button type="button" class="good" data-node-action="start" data-testid="kgw-node-start-{net}" data-net="{net}">Start</button>
            <button type="button" data-node-action="stop" data-testid="kgw-node-stop-{net}" data-net="{net}">Stop</button>
          </div>

          <div class="node-v6-status">
            <span id="{runtime_status}" class="node-v6-runtime-status-pill" data-state="stopped">Stopped</span>
            <span id="{runtime_evidence}" class="node-v6-runtime-evidence">No process owner</span>
            <span id="{settings_authority}" class="node-v6-runtime-evidence">Effective settings apply on next Start</span>
          </div>

          <div id="{runtime_error}" class="node-v6-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        {sections}
        </div>

        <div class="settings-bottom-actions node-settings-bottom-actions">
        <button type="button" data-node-action="save-settings" data-net="{net}">Save Settings</button>
        <button type="button" data-node-action="restore-defaults" data-net="{net}">Restore Defaults</button>
        <button type="button" data-node-action="set-defaults" data-net="{net}">Set as Defaults</button>
        <p class="kgw-settings-help" data-settings-defaults-context="{net}">Restore uses KaspaGateway defaults. Settings apply on the next Start.</p>
        </div>

      </div>

      <div class="node-v6-inner-panel{log_class}" data-net="{net}" data-node-inner-panel="log" data-testid="kgw-node-live-panel-{net}"{log_hidden}>
        <p id="{monitor_state}" class="kgw-monitor-state" role="status">Node: Stopped</p>
        <div class="node-v6-log-toolbar">
          <button type="button" data-node-action="monitor-start" data-net="{net}">Start Node</button>
          <span class="node-v6-log-metadata" data-net="{net}">Network: {label} | Source: self-worker | Streams: stdout/stderr</span>
          <button type="button" data-node-action="copy-log" data-testid="kgw-node-copy-log-{net}" data-net="{net}">Copy Log</button>
          <button type="button" data-node-action="clear-log" data-testid="kgw-node-clear-log-{net}" data-net="{net}">Clear Log</button>
        </div>
        <div id="{log_empty}" class="node-v6-log-empty" data-node-log-empty="{net}">Node is stopped. Start the node to view its logs.</div>
        <pre id="{log_output}" class="node-v6-log" data-testid="kgw-node-log-output-{net}"></pre>
      </div>
</div>"#,
        label = profile.label,
        policy_status = id("policyStatus"),
        preview_body = id("previewBody"),
        preview_message = id("previewMessage"),
        command_preview = id("commandPreview"),
        argument_list = id("argumentList"),
        runtime_status = id("runtimeStatus"),
        runtime_evidence = id("runtimeEvidence"),
        settings_authority = id("settingsAuthority"),
        runtime_error = id("runtimeError"),
        monitor_state = id("monitorState"),
        log_empty = id("logEmpty"),
        log_output = id("logOutput"),
    ))
}

fn render_node_network_panels_browser() -> Result<String, String> {
    let mut output = String::new();
    for (index, profile) in NETWORKS.iter().enumerate() {
        let active_inner_tab = node_resolve_inner_tab(profile.key.to_owned());
        let enabled = node_network_enabled(profile.key.to_owned());
        output.push_str(&render_node_network_panel(
            profile,
            index,
            &active_inner_tab,
            enabled,
        )?);
    }
    Ok(output)
}

#[wasm_bindgen(js_name = nodeRenderNetworkPanelsHtml)]
pub fn node_render_network_panels_html() -> Result<String, JsValue> {
    render_node_network_panels_browser().map_err(|error| JsValue::from_str(&error))
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

#[derive(Default)]
struct NodePreviewState {
    sequence: u64,
    timer: Option<JsValue>,
}

thread_local! {
    static NODE_PREVIEWS: RefCell<BTreeMap<String, NodePreviewState>> =
        const { RefCell::new(BTreeMap::new()) };
}

fn clear_preview_timer(timer: Option<JsValue>) {
    let Some(timer) = timer else {
        return;
    };
    if let Some(clear) = function(&window(), "clearTimeout") {
        let _ = clear.call1(&window(), &timer);
    }
}

fn begin_preview_sequence(net: &str) -> u64 {
    NODE_PREVIEWS.with(|states| {
        let mut states = states.borrow_mut();
        let state = states.entry(net.to_owned()).or_default();
        clear_preview_timer(state.timer.take());
        state.sequence = state.sequence.saturating_add(1);
        state.sequence
    })
}

fn store_preview_timer(net: &str, sequence: u64, timer: JsValue) {
    NODE_PREVIEWS.with(|states| {
        let mut states = states.borrow_mut();
        let state = states.entry(net.to_owned()).or_default();
        if state.sequence == sequence {
            state.timer = Some(timer);
        }
    });
}

fn preview_sequence(net: &str) -> u64 {
    NODE_PREVIEWS.with(|states| {
        states
            .borrow()
            .get(net)
            .map(|state| state.sequence)
            .unwrap_or_default()
    })
}

fn preview_sequence_is_current(net: &str, sequence: u64) -> bool {
    preview_sequence(net) == sequence
}

async fn preview_invoke_with_timeout(
    command: &str,
    payload: JsValue,
    timeout_ms: u32,
) -> Result<JsValue, JsValue> {
    let invoke = node_backend_invoke(command.to_owned(), payload)?;
    if timeout_ms == 0 {
        return JsFuture::from(invoke).await;
    }

    let command_for_timeout = command.to_owned();
    let timeout = Promise::new(&mut move |_resolve, reject| {
        let command_for_callback = command_for_timeout.clone();
        let callback = Closure::wrap(Box::new(move || {
            let _ = reject.call1(
                &JsValue::UNDEFINED,
                &JsValue::from_str(&format!(
                    "{command_for_callback} timed out after {timeout_ms}ms"
                )),
            );
        }) as Box<dyn FnMut()>);
        if let Some(set_timeout) = function(&window(), "setTimeout") {
            let _ = set_timeout.call2(
                &window(),
                callback.as_ref().unchecked_ref(),
                &JsValue::from_f64(timeout_ms as f64),
            );
        }
        callback.forget();
    });

    let candidates = Array::new();
    candidates.push(invoke.as_ref());
    candidates.push(timeout.as_ref());
    JsFuture::from(Promise::race(candidates.as_ref())).await
}

async fn prepare_preview_inner(net: &str, effective: JsValue) -> Result<JsValue, JsValue> {
    if backend_invoke_function().is_none() {
        return Err(
            Error::new("Connect to the desktop backend to validate these settings.").into(),
        );
    }
    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(net));
    set(payload.as_ref(), "effectiveNodeSettings", &effective);
    preview_invoke_with_timeout("kgw_node_settings_preview_v1", payload.into(), 10_000).await
}

async fn finish_preview_update(net: String, sequence: u64) {
    let result = async {
        let effective = node_effective_node_settings_inner(&net)?;
        let result = prepare_preview_inner(&net, effective.clone()).await?;
        if !preview_sequence_is_current(&net, sequence) {
            return Ok::<(), JsValue>(());
        }

        let preview = node_by_id(node_element_id(net.clone(), "commandPreview".to_owned()));
        if !present(&preview) {
            return Ok(());
        }
        let command = crate::js_string_owned(&property(&result, "command"));
        set(&preview, "value", &JsValue::from_str(&command));
        set(
            &property(&preview, "dataset"),
            "effectiveSettingsAuthority",
            &JsValue::from_str("validated-backend-settings"),
        );

        let arguments = property(&result, "arguments");
        let serialized = JSON::stringify(&arguments)
            .ok()
            .map(|value| crate::js_string_owned(value.as_ref()))
            .unwrap_or_else(|| "[]".to_owned());
        set(
            &property(&preview, "dataset"),
            "arguments",
            &JsValue::from_str(&serialized),
        );

        let app_dir = crate::js_string_owned(&property(&result, "appDir"));
        let path = node_by_id(node_element_id(net.clone(), "appDir".to_owned()));
        if present(&path) {
            set(&path, "value", &JsValue::from_str(&app_dir));
            set(&path, "title", &JsValue::from_str(&app_dir));
        }

        let argument_list = node_by_id(node_element_id(net.clone(), "argumentList".to_owned()));
        if present(&argument_list) {
            let joined = if Array::is_array(&arguments) {
                Array::from(&arguments)
                    .iter()
                    .map(|value| crate::js_string_owned(&value))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            };
            set(
                &argument_list,
                "textContent",
                &JsValue::from_str(&joined),
            );
        }

        let available_threads = crate::js_string_owned(&property(&result, "availableCpuThreads"));
        let async_threads = crate::js_string_owned(&property(&effective, "asyncThreads"));
        let ram_scale = crate::js_string_owned(&property(&effective, "ramScale"));
        node_preview_message_inner(
            &net,
            &format!(
                "Embedded kaspad - {available_threads} CPU threads available - configured {async_threads} threads, RAM scale {ram_scale}. The managed data directory is included."
            ),
            false,
        );
        Ok(())
    }
    .await;

    if let Err(error) = result
        && preview_sequence_is_current(&net, sequence)
    {
        node_preview_message_inner(&net, &normalize_runtime_error_value(&error), true);
    }
}

fn update_command_inner(net: &str, locked: bool) -> u64 {
    let preview = node_by_id(node_element_id(net.to_owned(), "commandPreview".to_owned()));
    if !present(&preview) {
        return preview_sequence(net);
    }

    node_sync_dependencies_inner(net, locked);
    let errors = node_validate_form_inner(net, false);
    let sequence = begin_preview_sequence(net);

    set(&preview, "value", &JsValue::from_str(""));
    set(
        &property(&preview, "dataset"),
        "effectiveSettingsAuthority",
        &JsValue::from_str("validating"),
    );

    if errors.is_object() && Object::keys(&Object::from(errors)).length() > 0 {
        node_preview_message_inner(
            net,
            "Correct the highlighted fields before saving or starting.",
            true,
        );
        return sequence;
    }

    node_preview_message_inner(net, "Validating effective settings...", false);
    let net_for_timer = net.to_owned();
    let callback = Closure::once_into_js(move || {
        let net_for_task = net_for_timer.clone();
        spawn_local(async move {
            finish_preview_update(net_for_task, sequence).await;
        });
    });
    if let Some(set_timeout) = function(&window(), "setTimeout")
        && let Ok(timer) = set_timeout.call2(&window(), &callback, &JsValue::from_f64(160.0))
    {
        store_preview_timer(net, sequence, timer);
    }
    sequence
}

#[wasm_bindgen(js_name = nodeUpdateCommand)]
pub fn node_update_command(net: String, locked: bool) -> u64 {
    update_command_inner(&net, locked)
}

#[wasm_bindgen(js_name = nodePreviewSequence)]
pub fn node_preview_sequence(net: String) -> u64 {
    preview_sequence(&net)
}

#[wasm_bindgen(js_name = nodePreparePreview)]
pub async fn node_prepare_preview(net: String, effective: JsValue) -> Result<JsValue, JsValue> {
    prepare_preview_inner(&net, effective).await
}

#[wasm_bindgen(js_name = nodeApplyRootDefaultPath)]
pub async fn node_apply_root_default_path(net: String, locked: bool) -> Result<JsValue, JsValue> {
    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(&net));
    let context = JsFuture::from(node_backend_invoke(
        "kgw_settings_context_v1".to_owned(),
        payload.into(),
    )?)
    .await?;
    let app_dir = crate::js_string_owned(&property(&context, "appDir"));
    let field = node_by_id(node_element_id(net.clone(), "appDir".to_owned()));
    if present(&field) {
        set(&field, "value", &JsValue::from_str(&app_dir));
        set(&field, "title", &JsValue::from_str(&app_dir));
    }
    update_command_inner(&net, locked);
    let output = Object::new();
    set(output.as_ref(), "appDir", &JsValue::from_str(&app_dir));
    Ok(output.into())
}

#[wasm_bindgen(js_name = nodeToggleCommandOptionAndUpdate)]
pub fn node_toggle_command_option_and_update(net: String, name: String, locked: bool) -> bool {
    let enabled = node_toggle_command_option(net.clone(), name);
    update_command_inner(&net, locked);
    enabled
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

fn query_all_values(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let Some(list) = call1(target, "querySelectorAll", &JsValue::from_str(selector)) else {
        return Vec::new();
    };
    let length = crate::js_number(&property(&list, "length"));
    (0..length.max(0.0) as u32)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .filter(present)
        .collect()
}

fn dataset_text(target: &JsValue, name: &str) -> String {
    crate::js_string_owned(&property(&property(target, "dataset"), name))
}

fn attribute_text(target: &JsValue, name: &str) -> String {
    call1(target, "getAttribute", &JsValue::from_str(name))
        .filter(present)
        .map(|value| crate::js_string_owned(&value))
        .unwrap_or_default()
}

fn set_attribute_text(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn class_contains(target: &JsValue, name: &str) -> bool {
    call1(
        &property(target, "classList"),
        "contains",
        &JsValue::from_str(name),
    )
    .is_some_and(|value| crate::js_boolean(&value))
}

fn class_toggle(target: &JsValue, name: &str, active: bool) {
    let _ = call2(
        &property(target, "classList"),
        "toggle",
        &JsValue::from_str(name),
        &JsValue::from_bool(active),
    );
}

fn callback_function(callbacks: &JsValue, name: &str) -> Option<Function> {
    property(callbacks, name).dyn_into::<Function>().ok()
}

fn callback_locked(callbacks: &JsValue, net: &str) -> bool {
    callback_function(callbacks, "isLocked")
        .and_then(|function| {
            function
                .call1(&JsValue::UNDEFINED, &JsValue::from_str(net))
                .ok()
        })
        .is_some_and(|value| crate::js_boolean(&value))
}

fn callback_apply_display_only(callbacks: &JsValue, net: &str, locked: bool, reason: &str) {
    if let Some(function) = callback_function(callbacks, "applyDisplayOnly") {
        let _ = function.call3(
            &JsValue::UNDEFINED,
            &JsValue::from_str(net),
            &JsValue::from_bool(locked),
            &JsValue::from_str(reason),
        );
    }
}

fn callback_set_runtime_buttons(callbacks: &JsValue, net: &str, running: bool, locked: bool) {
    if let Some(function) = callback_function(callbacks, "setRuntimeButtons") {
        let _ = function.call3(
            &JsValue::UNDEFINED,
            &JsValue::from_str(net),
            &JsValue::from_bool(running),
            &JsValue::from_bool(locked),
        );
    }
}

fn callback_hydrate(callbacks: &JsValue, reason: &str) {
    if let Some(function) = callback_function(callbacks, "hydrate") {
        let _ = function.call1(&JsValue::UNDEFINED, &JsValue::from_str(reason));
    }
}

fn trace_navigation(net: &str, phase: &str, selected: &str, text: &str, persisted: bool) {
    let details = Object::new();
    set(
        details.as_ref(),
        "patch",
        &JsValue::from_str(
            "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_NODE_LAST_NETWORK_RESTORE_R101W2",
        ),
    );
    set(details.as_ref(), "selected", &JsValue::from_str(selected));
    set(details.as_ref(), "text", &JsValue::from_str(text));
    set(
        details.as_ref(),
        "persisted",
        &JsValue::from_bool(persisted),
    );
    let _ = crate::node_start_trace::node_explicit_trace(
        JsValue::from_str(if net.is_empty() { "unknown" } else { net }),
        JsValue::from_str("internal-navigation"),
        JsValue::from_str(phase),
        details.into(),
    );
}

fn select_node_network(
    tabs: &[JsValue],
    panels: &[JsValue],
    callbacks: &JsValue,
    selected: &str,
    reason: &str,
    persist: bool,
) -> String {
    let normalized = normalize_network_text(selected);
    if normalized.is_empty() {
        return String::new();
    }
    if persist {
        storage_set(LAST_NETWORK_KEY, &normalized);
    }

    for tab in tabs {
        let active = dataset_text(tab, "nodeNetworkTab") == normalized;
        class_toggle(tab, "active", active);
        set_attribute_text(tab, "aria-selected", if active { "true" } else { "false" });
        set(
            &property(tab, "dataset"),
            "active",
            &JsValue::from_str(if active { "true" } else { "false" }),
        );
    }

    for panel in panels {
        let active = dataset_text(panel, "nodeNetworkPanel") == normalized;
        class_toggle(panel, "active", active);
        set(panel, "hidden", &JsValue::from_bool(!active));
        set(
            &property(panel, "dataset"),
            "active",
            &JsValue::from_str(if active { "true" } else { "false" }),
        );
    }

    if callback_locked(callbacks, &normalized) {
        callback_apply_display_only(
            callbacks,
            &normalized,
            true,
            &format!("network-tab-select-{reason}"),
        );
        callback_set_runtime_buttons(callbacks, &normalized, false, true);
    }
    callback_hydrate(callbacks, &format!("network-tab-{reason}"));
    normalized
}

#[wasm_bindgen(js_name = nodeInstallNetworkTabs)]
pub fn node_install_network_tabs(root: JsValue, callbacks: JsValue) -> bool {
    if !present(&root) {
        return false;
    }
    let tabs = query_all_values(&root, "[data-node-network-tab]");
    let panels = query_all_values(&root, "[data-node-network-panel]");

    for tab in tabs.clone() {
        let tabs_for_click = tabs.clone();
        let panels_for_click = panels.clone();
        let callbacks_for_click = callbacks.clone();
        let tab_for_click = tab.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            let selected = dataset_text(&tab_for_click, "nodeNetworkTab");
            set(
                &property(&event, "__kgwNodeNavigationObserved"),
                "value",
                &JsValue::TRUE,
            );
            let text = crate::js_string_owned(&property(&tab_for_click, "textContent"))
                .trim()
                .to_owned();
            trace_navigation(
                &selected,
                "r45d-node-network-tab-click",
                &selected,
                &text,
                true,
            );
            select_node_network(
                &tabs_for_click,
                &panels_for_click,
                &callbacks_for_click,
                &selected,
                "click",
                true,
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &tab,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    let saved = storage_get(LAST_NETWORK_KEY)
        .map(|value| normalize_network_text(&value))
        .unwrap_or_default();
    let existing = tabs.iter().find(|tab| {
        class_contains(tab, "active")
            || attribute_text(tab, "aria-selected") == "true"
            || dataset_text(tab, "active") == "true"
    });
    let default_tab = if !saved.is_empty() {
        tabs.iter()
            .find(|tab| dataset_text(tab, "nodeNetworkTab") == saved)
            .cloned()
    } else {
        None
    }
    .or_else(|| existing.cloned())
    .or_else(|| {
        tabs.iter()
            .find(|tab| dataset_text(tab, "nodeNetworkTab") == "mainnet")
            .cloned()
    })
    .or_else(|| tabs.first().cloned());

    if let Some(default_tab) = default_tab {
        let selected = dataset_text(&default_tab, "nodeNetworkTab");
        let reason = if saved.is_empty() {
            "initial"
        } else {
            "saved-initial"
        };
        select_node_network(&tabs, &panels, &callbacks, &selected, reason, false);
    }

    callback_hydrate(&callbacks, "network-tabs-installed");

    let tabs_for_external = tabs.clone();
    let panels_for_external = panels.clone();
    let callbacks_for_external = callbacks.clone();
    let external = Closure::wrap(Box::new(move |net: JsValue| {
        let selected = crate::js_string_owned(&net);
        select_node_network(
            &tabs_for_external,
            &panels_for_external,
            &callbacks_for_external,
            &selected,
            "external",
            true,
        );
    }) as Box<dyn FnMut(JsValue)>);
    set(
        &window(),
        "kgwNodeSelectNetworkTabR101W2",
        external.as_ref().unchecked_ref(),
    );
    external.forget();
    true
}

#[wasm_bindgen(js_name = nodeInstallDelegatedTabs)]
pub fn node_install_delegated_tabs(root: JsValue) -> bool {
    if !present(&root) {
        return false;
    }
    let root_for_click = root.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let inner_tab = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-node-inner-tab]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if present(&inner_tab) {
            let net = dataset_text(&inner_tab, "net");
            let selected = node_save_inner_tab(
                net.clone(),
                property(&property(&inner_tab, "dataset"), "nodeInnerTab"),
            );
            let panel = query(
                &root_for_click,
                &format!("[data-node-network-panel=\"{net}\"]"),
            );
            let details = Object::new();
            set(
                details.as_ref(),
                "patch",
                &JsValue::from_str(
                    "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_NODE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U",
                ),
            );
            set(
                details.as_ref(),
                "trusted",
                &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
            );
            set(details.as_ref(), "selected", &JsValue::from_str(&selected));
            set(
                details.as_ref(),
                "text",
                &JsValue::from_str(
                    crate::js_string_owned(&property(&inner_tab, "textContent")).trim(),
                ),
            );
            set(details.as_ref(), "persisted", &JsValue::TRUE);
            let _ = crate::node_start_trace::node_explicit_trace(
                JsValue::from_str(if net.is_empty() { "unknown" } else { &net }),
                JsValue::from_str("internal-navigation"),
                JsValue::from_str("r45d-node-inner-tab-click"),
                details.into(),
            );

            for item in query_all_values(&panel, "[data-node-inner-tab]") {
                class_toggle(&item, "active", Object::is(&item, &inner_tab));
            }
            for item in query_all_values(&panel, "[data-node-inner-panel]") {
                let active = dataset_text(&item, "nodeInnerPanel") == selected;
                class_toggle(&item, "active", active);
                set(&item, "hidden", &JsValue::from_bool(!active));
            }
            return;
        }

        let section_tab = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-node-section-tab]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&section_tab) {
            return;
        }
        let net = dataset_text(&section_tab, "net");
        let selected = dataset_text(&section_tab, "nodeSectionTab");
        let panel = query(
            &root_for_click,
            &format!("[data-node-network-panel=\"{net}\"]"),
        );
        let details = Object::new();
        set(
            details.as_ref(),
            "patch",
            &JsValue::from_str("KGW_INTERNAL_NAV_TRACE_OWNER_R45D"),
        );
        set(
            details.as_ref(),
            "trusted",
            &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
        );
        set(details.as_ref(), "selected", &JsValue::from_str(&selected));
        set(
            details.as_ref(),
            "text",
            &JsValue::from_str(
                crate::js_string_owned(&property(&section_tab, "textContent")).trim(),
            ),
        );
        let _ = crate::node_start_trace::node_explicit_trace(
            JsValue::from_str(if net.is_empty() { "unknown" } else { &net }),
            JsValue::from_str("internal-navigation"),
            JsValue::from_str("r45d-node-section-tab-click"),
            details.into(),
        );

        for item in query_all_values(&panel, "[data-node-section-tab]") {
            let active = Object::is(&item, &section_tab);
            class_toggle(&item, "active", active);
            set_attribute_text(
                &item,
                "aria-selected",
                if active { "true" } else { "false" },
            );
        }
        for item in query_all_values(&panel, "[data-node-section-panel]") {
            let active = dataset_text(&item, "nodeSectionPanel") == selected;
            class_toggle(&item, "active", active);
            set(&item, "hidden", &JsValue::from_bool(!active));
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &root,
        "addEventListener",
        &JsValue::from_str("click"),
        callback.as_ref().unchecked_ref(),
    );
    callback.forget();
    true
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeEvidenceText {
    text: String,
    fields: Vec<(String, String)>,
    pid: String,
    owner: String,
    role: String,
    state: String,
}

fn last_runtime_field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    fields
        .iter()
        .rev()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.as_str())
}

fn truthy_runtime_field<'a>(fields: &'a [(String, String)], key: &str) -> Option<&'a str> {
    last_runtime_field(fields, key).filter(|value| !value.is_empty())
}

fn runtime_evidence_text(raw: &str) -> RuntimeEvidenceText {
    let fields = parse_runtime_fields_text(raw);
    let pid = last_runtime_field(&fields, "pid")
        .unwrap_or("")
        .trim()
        .to_owned();
    let owner = truthy_runtime_field(&fields, "owner")
        .or_else(|| truthy_runtime_field(&fields, "source"))
        .unwrap_or("self-worker")
        .to_owned();
    let role = truthy_runtime_field(&fields, "role")
        .or_else(|| truthy_runtime_field(&fields, "runtime_role"))
        .or_else(|| truthy_runtime_field(&fields, "runtimeRole"))
        .unwrap_or("node")
        .to_owned();
    let state = truthy_runtime_field(&fields, "runtime_state")
        .or_else(|| truthy_runtime_field(&fields, "runtimeState"))
        .unwrap_or(if pid.is_empty() { "" } else { "running" })
        .to_owned();
    RuntimeEvidenceText {
        text: raw.to_owned(),
        fields,
        pid,
        owner,
        role,
        state,
    }
}

fn runtime_fields_object(fields: &[(String, String)]) -> JsValue {
    let output = Object::new();
    for (key, value) in fields {
        set(output.as_ref(), key, &JsValue::from_str(value));
    }
    output.into()
}

fn runtime_evidence_object(evidence: &RuntimeEvidenceText) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "text", &JsValue::from_str(&evidence.text));
    set(
        output.as_ref(),
        "fields",
        &runtime_fields_object(&evidence.fields),
    );
    set(output.as_ref(), "pid", &JsValue::from_str(&evidence.pid));
    set(
        output.as_ref(),
        "owner",
        &JsValue::from_str(&evidence.owner),
    );
    set(output.as_ref(), "role", &JsValue::from_str(&evidence.role));
    set(
        output.as_ref(),
        "state",
        &JsValue::from_str(&evidence.state),
    );
    output.into()
}

fn assert_start_evidence_text(net: &str, raw: &str) -> Result<RuntimeEvidenceText, String> {
    let evidence = runtime_evidence_text(raw);
    let lower = evidence.text.to_ascii_lowercase();
    if lower.contains("start_blocked=true") || lower.contains("start_allowed=false") {
        return Err(evidence.text.clone());
    }

    let response_network = last_runtime_field(&evidence.fields, "network")
        .unwrap_or("")
        .trim();
    if !response_network.is_empty() && response_network != net {
        return Err(format!(
            "Backend start response used the wrong network: {}",
            evidence.text
        ));
    }

    if evidence.pid.is_empty() || !evidence.pid.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "Backend start response did not include process ID evidence: {}",
            evidence.text
        ));
    }

    let readiness = last_runtime_field(&evidence.fields, "readiness").unwrap_or("");
    if !readiness.eq_ignore_ascii_case("READY") {
        return Err(format!(
            "Backend Start did not provide role readiness evidence: {}",
            evidence.text
        ));
    }
    Ok(evidence)
}

#[wasm_bindgen(js_name = nodeRuntimeEvidence)]
pub fn node_runtime_evidence(value: JsValue) -> JsValue {
    let raw = stringify_runtime_result_value(&value);
    runtime_evidence_object(&runtime_evidence_text(&raw))
}

#[wasm_bindgen(js_name = nodeAssertStartEvidence)]
pub fn node_assert_start_evidence(net: String, value: JsValue) -> Result<JsValue, JsValue> {
    let raw = stringify_runtime_result_value(&value);
    assert_start_evidence_text(&net, &raw)
        .map(|evidence| runtime_evidence_object(&evidence))
        .map_err(|message| Error::new(&message).into())
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

const LOG_FONT_MIN_SIZE: i32 = 10;
const LOG_FONT_MAX_SIZE: i32 = 18;
const LOG_FONT_DEFAULT_SIZE: i32 = 12;

fn log_font_storage_key(net: &str) -> String {
    format!("kgw.node.log.fontSize.{net}")
}

fn clamp_log_font_size(value: &str) -> i32 {
    value
        .trim()
        .parse::<i32>()
        .unwrap_or(LOG_FONT_DEFAULT_SIZE)
        .clamp(LOG_FONT_MIN_SIZE, LOG_FONT_MAX_SIZE)
}

fn read_log_font_size(net: &str) -> i32 {
    storage_get(&log_font_storage_key(net))
        .map(|value| clamp_log_font_size(&value))
        .unwrap_or(LOG_FONT_DEFAULT_SIZE)
}

fn write_log_font_size(net: &str, size: i32) -> i32 {
    let final_size = size.clamp(LOG_FONT_MIN_SIZE, LOG_FONT_MAX_SIZE);
    storage_set(&log_font_storage_key(net), &final_size.to_string());
    final_size
}

fn log_output(net: &str) -> JsValue {
    node_by_id(node_element_id(net.to_owned(), "logOutput".to_owned()))
}

fn log_toolbar(net: &str) -> JsValue {
    let root = query(&document(), "#kaspa-node");
    if !present(&root) {
        return JsValue::UNDEFINED;
    }
    let copy_selector =
        format!(".node-v6-log-toolbar [data-node-action='copy-log'][data-net='{net}']");
    let copy_button = query(&root, &copy_selector);
    if present(&copy_button) {
        let toolbar = call1(
            &copy_button,
            "closest",
            &JsValue::from_str(".node-v6-log-toolbar"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if present(&toolbar) {
            return toolbar;
        }
    }
    let panel = query(&root, &format!("[data-net='{net}']"));
    if present(&panel) {
        query(&panel, ".node-v6-log-toolbar")
    } else {
        JsValue::UNDEFINED
    }
}

fn set_style_property(style: &JsValue, name: &str, value: &str, priority: &str) {
    if let Some(set_property) = function(style, "setProperty") {
        let _ = set_property.call3(
            style,
            &JsValue::from_str(name),
            &JsValue::from_str(value),
            &JsValue::from_str(priority),
        );
    }
}

fn apply_log_font_size(net: &str) {
    let output = log_output(net);
    let size = read_log_font_size(net);
    if present(&output) {
        set(
            &property(&output, "dataset"),
            "kgwLogFontSizePane",
            &JsValue::from_str("v29"),
        );
        let style = property(&output, "style");
        set_style_property(&style, "--kgw-log-font-size", &format!("{size}px"), "");
        set_style_property(&style, "font-size", "var(--kgw-log-font-size)", "important");
        set_style_property(&style, "line-height", "1.45", "important");
    }

    let toolbar = log_toolbar(net);
    if present(&toolbar) {
        let value = query(
            &toolbar,
            &format!(".kgw-log-font-size-value[data-net='{net}']"),
        );
        if present(&value) {
            set(
                &value,
                "textContent",
                &JsValue::from_str(&format!("{size}px")),
            );
        }
    }
}

fn log_font_button(label: &str, title: &str) -> JsValue {
    let button = create_element("button");
    set(&button, "type", &JsValue::from_str("button"));
    set(
        &button,
        "className",
        &JsValue::from_str("kgw-log-font-size-button"),
    );
    set(&button, "textContent", &JsValue::from_str(label));
    set(&button, "title", &JsValue::from_str(title));
    let _ = call2(
        &button,
        "setAttribute",
        &JsValue::from_str("aria-label"),
        &JsValue::from_str(title),
    );
    set(
        &property(&button, "dataset"),
        "kgwLogFontOwner",
        &JsValue::from_str("v29"),
    );
    button
}

fn bind_log_font_button(button: &JsValue, net: &str, delta: i32, phase: &'static str) {
    let net = net.to_owned();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        if let Some(prevent) = function(&event, "preventDefault") {
            let _ = prevent.call0(&event);
        }
        if let Some(stop) = function(&event, "stopPropagation") {
            let _ = stop.call0(&event);
        }
        let previous_size = read_log_font_size(&net);
        let next_size = if delta == 0 {
            LOG_FONT_DEFAULT_SIZE
        } else {
            previous_size + delta
        };
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
            "previousSize",
            &JsValue::from_f64(previous_size as f64),
        );
        set(
            details.as_ref(),
            "nextSize",
            &JsValue::from_f64(next_size as f64),
        );
        let _ = crate::node_start_trace::node_small_owner_trace(
            JsValue::from_str(&net),
            JsValue::from_str("log-font-size"),
            JsValue::from_str(phase),
            details.into(),
        );
        write_log_font_size(&net, next_size);
        apply_log_font_size(&net);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        button,
        "addEventListener",
        &JsValue::from_str("click"),
        callback.as_ref().unchecked_ref(),
    );
    callback.forget();
}

fn install_log_font_controls_for_network(net: &str) {
    let toolbar = log_toolbar(net);
    if !present(&toolbar) {
        return;
    }
    for duplicate in query_all_values(&toolbar, ".kgw-log-font-size-controls") {
        if let Some(remove) = function(&duplicate, "remove") {
            let _ = remove.call0(&duplicate);
        }
    }

    let controls = create_element("div");
    set(
        &controls,
        "className",
        &JsValue::from_str("kgw-log-font-size-controls"),
    );
    let controls_dataset = property(&controls, "dataset");
    set(&controls_dataset, "kind", &JsValue::from_str("node"));
    set(&controls_dataset, "net", &JsValue::from_str(net));
    set(
        &controls_dataset,
        "marker",
        &JsValue::from_str("KGW_NODE_LOG_SCOPED_CONTROLS_V29"),
    );

    let decrease = log_font_button("A-", "Decrease log font size");
    let value = create_element("span");
    set(
        &value,
        "className",
        &JsValue::from_str("kgw-log-font-size-value"),
    );
    set(&property(&value, "dataset"), "net", &JsValue::from_str(net));
    set(
        &value,
        "textContent",
        &JsValue::from_str(&format!("{}px", read_log_font_size(net))),
    );
    let increase = log_font_button("A+", "Increase log font size");
    let reset = log_font_button("Reset", "Reset log font size");

    bind_log_font_button(&decrease, net, -1, "r51b3-node-log-font-decrease-click");
    bind_log_font_button(&increase, net, 1, "r51b3-node-log-font-increase-click");
    bind_log_font_button(&reset, net, 0, "r51b3-node-log-font-reset-click");

    append_child(&controls, &decrease);
    append_child(&controls, &value);
    append_child(&controls, &increase);
    append_child(&controls, &reset);
    append_child(&toolbar, &controls);
    apply_log_font_size(net);
}

fn install_all_log_font_controls() {
    for profile in NETWORKS {
        install_log_font_controls_for_network(profile.key);
    }
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
    install_all_log_font_controls();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_font_size_contract_matches_legacy_bounds() {
        assert_eq!(
            log_font_storage_key("mainnet"),
            "kgw.node.log.fontSize.mainnet"
        );
        assert_eq!(clamp_log_font_size(""), LOG_FONT_DEFAULT_SIZE);
        assert_eq!(clamp_log_font_size("9"), LOG_FONT_MIN_SIZE);
        assert_eq!(clamp_log_font_size("12"), 12);
        assert_eq!(clamp_log_font_size("99"), LOG_FONT_MAX_SIZE);
    }

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
    fn isolated_runtime_compat_routes_match_legacy_contract() {
        let cases = [
            (
                "preview",
                "status",
                "rk_isolated_adapter_status_preview_v1",
                false,
            ),
            ("final", "start", "rk_final_isolated_adapter_start_v1", true),
            (
                "final",
                "status",
                "rk_final_isolated_adapter_status_v1",
                true,
            ),
            ("final", "stop", "rk_final_isolated_adapter_stop_v1", true),
            ("v66", "policy", "rk_v66_runtime_feature_policy_v1", true),
            ("v66", "start", "rk_v66_isolated_adapter_start_v1", true),
            ("v66", "status", "rk_v66_isolated_adapter_status_v1", true),
            ("v66", "stop", "rk_v66_isolated_adapter_stop_v1", true),
            ("v67", "policy", "rk_v66_runtime_feature_policy_v1", true),
            ("v67", "start", "rk_v66_isolated_adapter_start_v1", true),
            ("v67", "status", "rk_v66_isolated_adapter_status_v1", true),
            ("v67", "stop", "rk_v66_isolated_adapter_stop_v1", true),
        ];
        for (family, action, command, include_command_in_log) in cases {
            let route = isolated_compat_route(family, action).expect("compat route");
            assert_eq!(route.command, command, "{family}/{action}");
            assert_eq!(
                route.include_command_in_log, include_command_in_log,
                "{family}/{action}"
            );
        }
        assert!(isolated_compat_route("preview", "start").is_none());
        assert!(isolated_compat_route("v66", "missing").is_none());
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
    fn runtime_evidence_defaults_and_aliases_match_legacy_contract() {
        let evidence = runtime_evidence_text(
            "network=mainnet;pid=123;source=worker;runtime_role=node;runtimeState=ready",
        );
        assert_eq!(evidence.pid, "123");
        assert_eq!(evidence.owner, "worker");
        assert_eq!(evidence.role, "node");
        assert_eq!(evidence.state, "ready");

        let fallback = runtime_evidence_text("pid=42");
        assert_eq!(fallback.owner, "self-worker");
        assert_eq!(fallback.role, "node");
        assert_eq!(fallback.state, "running");

        let empty = runtime_evidence_text("");
        assert_eq!(empty.pid, "");
        assert_eq!(empty.state, "");
    }

    #[test]
    fn start_evidence_validation_matches_legacy_contract() {
        let ok = assert_start_evidence_text(
            "mainnet",
            "network=mainnet;pid=123;readiness=READY;running=true",
        )
        .expect("valid evidence");
        assert_eq!(ok.pid, "123");

        let wrong_network = "network=testnet10;pid=123;readiness=READY;running=true";
        assert_eq!(
            assert_start_evidence_text("mainnet", wrong_network).unwrap_err(),
            format!("Backend start response used the wrong network: {wrong_network}")
        );

        let bad_pid = "network=mainnet;pid=abc;readiness=READY";
        assert_eq!(
            assert_start_evidence_text("mainnet", bad_pid).unwrap_err(),
            format!("Backend start response did not include process ID evidence: {bad_pid}")
        );

        let not_ready = "network=mainnet;pid=123;readiness=STARTING";
        assert_eq!(
            assert_start_evidence_text("mainnet", not_ready).unwrap_err(),
            format!("Backend Start did not provide role readiness evidence: {not_ready}")
        );

        let blocked = "start_blocked=true;network=mainnet;pid=123;readiness=READY";
        assert_eq!(
            assert_start_evidence_text("mainnet", blocked).unwrap_err(),
            blocked
        );
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
    fn node_renderer_group_and_panel_ownership_contract_is_complete() {
        let expected = [
            "testnet",
            "netsuffix",
            "utxoIndex",
            "yes",
            "listenEnabled",
            "listenHost",
            "listenPort",
            "externalIpEnabled",
            "externalIpHost",
            "externalIpPort",
            "disableUpnp",
            "noDnsSeed",
            "uaComment",
            "rpcListenEnabled",
            "rpcListenHost",
            "rpcListenPort",
            "asyncThreads",
            "ramScale",
            "outPeers",
            "maxInPeers",
            "appDir",
            "rocksDbPreset",
            "rocksDbCacheSize",
            "logLevel",
            "noLogFiles",
            "logDir",
            "connectEnabled",
            "connectHost",
            "connectPort",
            "addPeerEnabled",
            "addPeerHost",
            "addPeerPort",
            "rpcBorshEnabled",
            "rpcBorshHost",
            "rpcBorshPort",
            "rpcJsonEnabled",
            "rpcJsonHost",
            "rpcJsonPort",
            "rpcMaxClients",
            "noGrpc",
            "archival",
            "maxTrackedAddresses",
            "retentionDays",
            "rocksDbWalDir",
            "configFile",
            "sanity",
            "perfMetrics",
            "perfMetricsInterval",
            "overrideParamsFile",
            "resetDb",
            "unsafeRpc",
            "enableUnsyncedMining",
        ];
        let mut grouped = std::collections::BTreeSet::new();
        for (_, _, _, names) in NODE_RENDER_GROUPS {
            for name in names {
                assert!(grouped.insert(*name), "duplicate renderer field: {name}");
            }
        }
        assert_eq!(
            grouped,
            expected
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
        );

        let source = include_str!("node_frontend_helpers.rs");
        let start = source
            .find("fn render_node_network_panel(")
            .expect("Rust panel renderer");
        let end = source[start..]
            .find("#[wasm_bindgen(js_name = nodeRenderNetworkPanelsHtml)]")
            .map(|offset| start + offset)
            .expect("Rust renderer export");
        let render = &source[start..end];
        let settings_start = render
            .find("data-node-inner-panel=\"settings\"")
            .expect("settings panel");
        let log_start = render
            .find("data-node-inner-panel=\"log\"")
            .expect("log panel");
        let settings = &render[settings_start..log_start];
        let log = &render[log_start..];
        assert_eq!(settings.matches("data-node-action=\"start\"").count(), 1);
        assert_eq!(settings.matches("data-node-action=\"stop\"").count(), 1);
        assert!(settings.contains("data-node-network-enabled"));
        assert!(!log.contains("data-node-action=\"start\""));
        assert!(!log.contains("data-node-action=\"stop\""));
        assert!(log.contains("data-node-action=\"copy-log\""));
        assert!(log.contains("data-node-action=\"clear-log\""));
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
    fn runtime_notice_state_key_matches_legacy_normalization() {
        assert_eq!(runtime_notice_state_key(" Running "), "running");
        assert_eq!(runtime_notice_state_key("Settings error"), "settings-error");
        assert_eq!(runtime_notice_state_key("  "), "stopped");
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
