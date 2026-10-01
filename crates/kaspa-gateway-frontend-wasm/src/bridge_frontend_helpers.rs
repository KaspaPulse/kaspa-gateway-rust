use super::bridge_command_options::{
    bridge_command_inline_state_r7, bridge_has_config,
    bridge_instance_command_should_include_from_instances_r13b,
};
use super::settings_contract::{
    bridge_field_enabled as settings_bridge_field_enabled,
    render_field_errors as settings_render_field_errors,
    validate_bridge_form as settings_validate_bridge_form,
};
use super::settings_layout::{
    decorate_fields as settings_decorate_fields, reveal_field as settings_reveal_field,
    set_field_state as settings_set_field_state,
};
use super::settings_schema::BRIDGE_MANAGED;
use js_sys::{Array, Error, Function, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BridgeNetworkProfile {
    key: &'static str,
    label: &'static str,
    testnet: bool,
    netsuffix: &'static str,
    kaspad_port: &'static str,
    stratum_port: &'static str,
    prom_port: &'static str,
    dashboard_port: &'static str,
    enabled_by_default: bool,
    experimental: bool,
    runtime: &'static str,
}

const NETWORKS: [BridgeNetworkProfile; 3] = [
    BridgeNetworkProfile {
        key: "mainnet",
        label: "Mainnet",
        testnet: false,
        netsuffix: "",
        kaspad_port: "16110",
        stratum_port: ":5555",
        prom_port: ":2112",
        dashboard_port: "3030",
        enabled_by_default: true,
        experimental: false,
        runtime: "Official Rusty Kaspa",
    },
    BridgeNetworkProfile {
        key: "testnet10",
        label: "Testnet 10",
        testnet: true,
        netsuffix: "10",
        kaspad_port: "16210",
        stratum_port: ":5655",
        prom_port: ":2212",
        dashboard_port: "3130",
        enabled_by_default: true,
        experimental: false,
        runtime: "Official Rusty Kaspa",
    },
    BridgeNetworkProfile {
        key: "testnet13",
        label: "Testnet 13",
        testnet: true,
        netsuffix: "13",
        kaspad_port: "16210",
        stratum_port: ":5755",
        prom_port: ":2312",
        dashboard_port: "3230",
        enabled_by_default: false,
        experimental: true,
        runtime: "DAGKnight - Experimental",
    },
];

fn bridge_r51_key_texts() -> impl Iterator<Item = &'static str> {
    NETWORKS.iter().map(|item| item.key)
}

fn profile(net: &str) -> Option<&'static BridgeNetworkProfile> {
    NETWORKS.iter().find(|item| item.key == net)
}

fn policy_key_text(net: &str) -> String {
    format!(
        "kgw.bridge.network.enabled.{}",
        if net.is_empty() { "unknown" } else { net }
    )
}

fn policy_message_text(net: &str) -> String {
    let Some(item) = profile(net) else {
        return String::new();
    };
    if item.experimental {
        let mut message =
            "Experimental network. Disabled by default and requires explicit opt-in.".to_owned();
        if net == "testnet13" {
            message.push_str(
                " The bundled Testnet13 node has no DNS seeders. For public sync, set a trusted Testnet13 peer in the node's Connect or Add Peer settings.",
            );
        }
        message
    } else {
        format!(
            "{}. External local-node mode is recommended for mining bridges.",
            item.runtime
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

fn create_bridge_element(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag)).unwrap_or(JsValue::UNDEFINED)
}

fn append_bridge_child(parent: &JsValue, child: &JsValue) {
    let _ = call1(parent, "appendChild", child);
}

fn query_bridge(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_bridge_result(target: &JsValue, selector: &str) -> Result<JsValue, JsValue> {
    let Some(callback) = function(target, "querySelector") else {
        return Err(JsValue::UNDEFINED);
    };
    callback.call1(target, &JsValue::from_str(selector))
}

fn bridge_value_property_text(target: &JsValue) -> Option<String> {
    if !present(target) || !Reflect::has(target, &JsValue::from_str("value")).unwrap_or(false) {
        return None;
    }
    let value = property(target, "value");
    Some(if crate::js_boolean(&value) {
        crate::js_string_owned(&value)
    } else {
        String::new()
    })
}

#[wasm_bindgen(js_name = bridgeCurrentNodeModeFromUiR65F)]
pub fn bridge_current_node_mode_from_ui_r65f(net: String) -> String {
    let direct = bridge_by_id(bridge_element_id(net.clone(), "nodeMode".to_owned()));
    if let Some(value) = bridge_value_property_text(&direct) {
        return value;
    }

    let selector_net = net;
    let document = document();
    let primary = match query_bridge_result(
        &document,
        &format!("[data-bridge-panel=\"{selector_net}\"]"),
    ) {
        Ok(value) => value,
        Err(_) => return String::new(),
    };
    let panel = if present(&primary) {
        primary
    } else {
        match query_bridge_result(&document, &format!("[data-net=\"{selector_net}\"]")) {
            Ok(value) => value,
            Err(_) => return String::new(),
        }
    };
    if present(&panel) {
        let select = match query_bridge_result(
            &panel,
            "[id$=\"-nodeMode\"], [data-bridge-setting=\"nodeMode\"], select[name=\"nodeMode\"]",
        ) {
            Ok(value) => value,
            Err(_) => return String::new(),
        };
        if let Some(value) = bridge_value_property_text(&select) {
            return value;
        }
    }

    String::new()
}

fn bridge_plain_port_only_text(value: &str) -> String {
    let text = value.trim();
    let digits = text.strip_prefix(':').unwrap_or(text);
    if digits.is_empty() || digits.len() > 5 || !digits.as_bytes().iter().all(u8::is_ascii_digit) {
        return text.to_owned();
    }
    let Ok(port) = digits.parse::<u32>() else {
        return text.to_owned();
    };
    if !(1..=65_535).contains(&port) {
        return text.to_owned();
    }
    port.to_string()
}

fn bridge_plain_port_only_js_text(value: &JsValue) -> String {
    let text = if crate::js_boolean(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    };
    bridge_plain_port_only_text(&text)
}

fn r95b_storage_field_id(net: &str, field_name: &str) -> String {
    format!("bridge-{net}-{field_name}")
}

fn r95b_preferred_port(net: &str, kind: &str) -> String {
    let profile = crate::bridge_port_core::bridge_static_port_profile_r91(net.to_owned());
    let range = property(&profile, kind);
    bridge_plain_port_only_js_text(&property(&range, "preferred"))
}

fn r95b_known_stale_sequential_port(net: &str, field_name: &str) -> &'static str {
    match (net, field_name) {
        ("testnet10", "stratumPort") => "5556",
        ("testnet10", "promPort") => "2113",
        ("testnet13", "stratumPort") => "5557",
        ("testnet13", "promPort") => "2114",
        _ => "",
    }
}

fn r95b_display_port_syntax(value: &JsValue) -> bool {
    let raw = if crate::js_boolean(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    };
    let trimmed = raw.trim();
    let digits = trimmed.strip_prefix(':').unwrap_or(trimmed);
    !digits.is_empty()
        && digits.len() <= 5
        && digits.as_bytes().iter().all(u8::is_ascii_digit)
        && trimmed.len() == digits.len() + usize::from(trimmed.starts_with(':'))
}

fn r95b_same_port(left: &str, right: &str) -> bool {
    bridge_plain_port_only_text(left) == bridge_plain_port_only_text(right)
}

fn r95b_normalize_network_port_values(net: &str, values: JsValue, reason: &str) -> JsValue {
    if !values.is_object() || values.is_null() {
        return values;
    }

    let changes = Array::new();
    for (field_name, kind) in [("stratumPort", "stratum"), ("promPort", "prom")] {
        let storage_id = r95b_storage_field_id(net, field_name);
        let item = property(&values, &storage_id);
        if !item.is_object()
            || item.is_null()
            || !Reflect::has(&item, &JsValue::from_str("value")).unwrap_or(false)
        {
            continue;
        }

        let raw_value = property(&item, "value");
        let current = bridge_plain_port_only_js_text(&raw_value);
        let stale = r95b_known_stale_sequential_port(net, field_name);
        let preferred = r95b_preferred_port(net, kind);

        if !stale.is_empty()
            && !preferred.is_empty()
            && r95b_same_port(&current, stale)
            && !r95b_same_port(&current, &preferred)
        {
            let normalized_preferred = bridge_plain_port_only_text(&preferred);
            set(&item, "value", &JsValue::from_str(&normalized_preferred));
            let change = Object::new();
            set(change.as_ref(), "field", &JsValue::from_str(field_name));
            set(change.as_ref(), "from", &JsValue::from_str(&current));
            set(
                change.as_ref(),
                "to",
                &JsValue::from_str(&normalized_preferred),
            );
            changes.push(change.as_ref());
        } else {
            let strict_string_differs = raw_value.as_string().is_none_or(|value| value != current);
            if strict_string_differs && r95b_display_port_syntax(&raw_value) {
                set(&item, "value", &JsValue::from_str(&current));
                let change = Object::new();
                set(change.as_ref(), "field", &JsValue::from_str(field_name));
                // Preserve the legacy R95B ordering: "from" is captured after
                // the display-only assignment, so it equals the normalized value.
                set(change.as_ref(), "from", &JsValue::from_str(&current));
                set(change.as_ref(), "to", &JsValue::from_str(&current));
                set(change.as_ref(), "displayOnly", &JsValue::TRUE);
                changes.push(change.as_ref());
            }
        }
    }

    if changes.length() > 0 {
        let details = Object::new();
        set(details.as_ref(), "patch", &JsValue::from_str("R98"));
        set(
            details.as_ref(),
            "owner",
            &JsValue::from_str("bridge-r51-r95b-settings-owner"),
        );
        set(details.as_ref(), "reason", &JsValue::from_str(reason));
        set(details.as_ref(), "changes", changes.as_ref());
        let _ = bridge_small_owner_trace_r44d(
            JsValue::from_str(net),
            JsValue::from_str("settings-persistence"),
            JsValue::from_str("r98-normalize-port-only-display-values"),
            details.into(),
        );
    }

    values
}

#[wasm_bindgen(js_name = bridgeR95BNormalizeNetworkPortValues)]
pub fn bridge_r95b_normalize_network_port_values(
    net: String,
    values: JsValue,
    reason: String,
) -> JsValue {
    r95b_normalize_network_port_values(&net, values, &reason)
}

#[wasm_bindgen(js_name = bridgePlainPortOnlyValueR98)]
pub fn bridge_plain_port_only_value_r98(value: JsValue) -> String {
    bridge_plain_port_only_js_text(&value)
}

#[wasm_bindgen(js_name = bridgeSamePortValueR98)]
pub fn bridge_same_port_value_r98(left: JsValue, right: JsValue) -> bool {
    bridge_plain_port_only_js_text(&left) == bridge_plain_port_only_js_text(&right)
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

const BRIDGE_R51_STORAGE_PREFIX: &str = "kgw.bridge.direct.v51.";

fn bridge_r51_storage_key_text(key: &str) -> String {
    format!("{BRIDGE_R51_STORAGE_PREFIX}{key}")
}

#[wasm_bindgen(js_name = bridgeR51Store)]
pub fn bridge_r51_store(key: String, value: JsValue) -> Result<(), JsValue> {
    let serialized = JSON::stringify(&value)?;
    let storage = local_storage();
    let setter = function(&storage, "setItem")
        .ok_or_else(|| JsValue::from_str("localStorage.setItem is unavailable"))?;
    setter.call2(
        &storage,
        &JsValue::from_str(&bridge_r51_storage_key_text(&key)),
        serialized.as_ref(),
    )?;
    Ok(())
}

#[wasm_bindgen(js_name = bridgeR51Load)]
pub fn bridge_r51_load(key: String) -> JsValue {
    let storage = local_storage();
    let Some(raw) = call1(
        &storage,
        "getItem",
        &JsValue::from_str(&bridge_r51_storage_key_text(&key)),
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

fn bridge_log_auto_scroll_key_text(net: &str) -> String {
    format!("kgw.bridge.log.autoscroll.{net}")
}

fn bridge_log_auto_scroll_enabled_text(value: Option<&str>) -> bool {
    value != Some("0")
}

#[wasm_bindgen(js_name = bridgeLogAutoScrollEnabled)]
pub fn bridge_log_auto_scroll_enabled(net: String) -> bool {
    bridge_log_auto_scroll_enabled_text(
        storage_get(&bridge_log_auto_scroll_key_text(&net)).as_deref(),
    )
}

#[wasm_bindgen(js_name = bridgeSetLogAutoScroll)]
pub fn bridge_set_log_auto_scroll(net: String, enabled: bool) {
    storage_set(
        &bridge_log_auto_scroll_key_text(&net),
        if enabled { "1" } else { "0" },
    );
    if !enabled {
        return;
    }
    let output = bridge_by_id(bridge_element_id(net, "logOutput".to_owned()));
    if present(&output) {
        let height = property(&output, "scrollHeight");
        set(&output, "scrollTop", &height);
    }
}

#[wasm_bindgen(js_name = bridgeInstallLogAutoScrollControls)]
pub fn bridge_install_log_auto_scroll_controls() {
    let doc = document();
    if !present(&doc) {
        return;
    }

    for profile in NETWORKS {
        let net = profile.key.to_owned();
        let output = bridge_by_id(bridge_element_id(net.clone(), "logOutput".to_owned()));
        if !present(&output) {
            continue;
        }

        let control_id = bridge_element_id(net.clone(), "logAutoScrollR27".to_owned());
        if present(&bridge_by_id(control_id.clone())) {
            continue;
        }

        let label = create_bridge_element("label");
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
            &JsValue::from_str("bridge"),
        );
        let _ = call2(
            &label,
            "setAttribute",
            &JsValue::from_str("title"),
            &JsValue::from_str("Keep the log pinned to the newest raw line."),
        );

        let checkbox = create_bridge_element("input");
        set(&checkbox, "type", &JsValue::from_str("checkbox"));
        set(&checkbox, "id", &JsValue::from_str(&control_id));
        set(
            &checkbox,
            "checked",
            &JsValue::from_bool(bridge_log_auto_scroll_enabled(net.clone())),
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
            let _ = bridge_small_owner_trace_r44d(
                JsValue::from_str(&net_for_change),
                JsValue::from_str("log-autoscroll"),
                JsValue::from_str("r51b3-bridge-log-autoscroll-change"),
                details.into(),
            );
            bridge_set_log_auto_scroll(net_for_change.clone(), checked);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &checkbox,
            "addEventListener",
            &JsValue::from_str("change"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();

        let span = create_bridge_element("span");
        set(
            &span,
            "textContent",
            &translate_raw("common.autoScroll", "Auto-scroll"),
        );
        append_bridge_child(&label, &checkbox);
        append_bridge_child(&label, &span);

        let panel = {
            let candidate = call1(
                &output,
                "closest",
                &JsValue::from_str(
                    ".bridge-v7-inner-panel, [data-bridge-inner-panel], [data-inner-panel], [data-bridge-panel], [data-panel]",
                ),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if present(&candidate) {
                candidate
            } else {
                property(&output, "parentElement")
            }
        };
        let toolbar_selector =
            ".bridge-v7-log-toolbar, .bridge-log-toolbar, [data-bridge-log-toolbar]";
        let toolbar = {
            let candidate = query_bridge(&panel, toolbar_selector);
            if present(&candidate) {
                candidate
            } else {
                query_bridge(&property(&output, "parentElement"), toolbar_selector)
            }
        };

        if present(&toolbar) {
            append_bridge_child(&toolbar, &label);
        } else {
            let parent = property(&output, "parentElement");
            if present(&parent) {
                let _ = call2(&parent, "insertBefore", &label, &output);
            }
        }
    }
}

fn bridge_inner_tab_storage_key_text(net: &str) -> String {
    format!(
        "kgw.bridge.innerTab.{}",
        if net.is_empty() { "unknown" } else { net }
    )
}

fn normalize_bridge_inner_tab_text(value: &str) -> &'static str {
    match value {
        "settings" => "settings",
        "log" => "log",
        _ => "log",
    }
}

#[wasm_bindgen(js_name = bridgeNormalizeInnerTab)]
pub fn bridge_normalize_inner_tab(value: JsValue) -> String {
    normalize_bridge_inner_tab_text(&crate::js_string_owned(&value)).to_owned()
}

#[wasm_bindgen(js_name = bridgeResolveInnerTab)]
pub fn bridge_resolve_inner_tab(net: String) -> String {
    let stored = storage_get(&bridge_inner_tab_storage_key_text(&net)).unwrap_or_default();
    normalize_bridge_inner_tab_text(&stored).to_owned()
}

#[wasm_bindgen(js_name = bridgeSaveInnerTab)]
pub fn bridge_save_inner_tab(net: String, selected: JsValue) -> String {
    let normalized = bridge_normalize_inner_tab(selected);
    storage_set(&bridge_inner_tab_storage_key_text(&net), &normalized);
    normalized
}

fn normalize_bridge_network_text(value: &str) -> String {
    let normalized = value.trim();
    match normalized {
        "mainnet" | "testnet10" | "testnet13" => normalized.to_owned(),
        _ => String::new(),
    }
}

const BRIDGE_LAST_NETWORK_KEY: &str = "kgw.bridge.lastNetwork";

#[wasm_bindgen(js_name = bridgeNormalizeNetwork)]
pub fn bridge_normalize_network(value: JsValue) -> String {
    normalize_bridge_network_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeReadLastNetwork)]
pub fn bridge_read_last_network() -> String {
    storage_get(BRIDGE_LAST_NETWORK_KEY)
        .map(|value| normalize_bridge_network_text(&value))
        .unwrap_or_default()
}

#[wasm_bindgen(js_name = bridgeSaveLastNetwork)]
pub fn bridge_save_last_network(net: JsValue) -> String {
    let normalized = bridge_normalize_network(net);
    if !normalized.is_empty() {
        storage_set(BRIDGE_LAST_NETWORK_KEY, &normalized);
    }
    normalized
}

fn bridge_tab_dataset_text(element: &JsValue, name: &str) -> String {
    crate::js_string_owned(&property(&property(element, "dataset"), name))
}

fn bridge_tab_network(element: &JsValue) -> String {
    for name in ["net", "bridgeNetworkTab", "bridgeNetworkPanel"] {
        let value = bridge_tab_dataset_text(element, name);
        if !value.is_empty() {
            return value;
        }
    }
    String::new()
}

fn bridge_tab_collection(root: &JsValue, selector: &str) -> Vec<JsValue> {
    let Some(collection) = call1(root, "querySelectorAll", &JsValue::from_str(selector)) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let len = crate::js_number(&property(&collection, "length")).max(0.0) as u32;
    for index in 0..len {
        if let Ok(value) = Reflect::get(&collection, &JsValue::from_f64(index as f64))
            && present(&value)
        {
            values.push(value);
        }
    }
    values
}

fn bridge_tab_class_contains(element: &JsValue, name: &str) -> bool {
    call1(
        &property(element, "classList"),
        "contains",
        &JsValue::from_str(name),
    )
    .is_some_and(|value| crate::js_boolean(&value))
}

fn bridge_tab_class_toggle(element: &JsValue, name: &str, enabled: bool) {
    let _ = call2(
        &property(element, "classList"),
        "toggle",
        &JsValue::from_str(name),
        &JsValue::from_bool(enabled),
    );
}

fn bridge_tab_attribute_text(element: &JsValue, name: &str) -> String {
    call1(element, "getAttribute", &JsValue::from_str(name))
        .map(|value| crate::js_string_owned(&value))
        .unwrap_or_default()
}

fn bridge_tab_set_attribute(element: &JsValue, name: &str, value: &str) {
    let _ = call2(
        element,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn bridge_tab_refresh_later(net: String, reason: String, callbacks: JsValue, delay_ms: i32) {
    let callback = Closure::wrap(Box::new(move || {
        let net = net.clone();
        let reason = reason.clone();
        let callbacks = callbacks.clone();
        spawn_local(async move {
            crate::bridge_runtime_core::bridge_r51_refresh_one(net, reason, callbacks).await;
        });
    }) as Box<dyn FnMut()>);
    if let Some(set_timeout) = function(&window(), "setTimeout") {
        let _ = set_timeout.call2(
            &window(),
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(delay_ms as f64),
        );
    }
    callback.forget();
}

fn bridge_tab_select(
    tabs: &[JsValue],
    panels: &[JsValue],
    callbacks: &JsValue,
    selected: &str,
    reason: &str,
    persist: bool,
) -> String {
    let normalized = normalize_bridge_network_text(selected);
    if normalized.is_empty() {
        return String::new();
    }
    if persist {
        storage_set(BRIDGE_LAST_NETWORK_KEY, &normalized);
    }

    for tab in tabs {
        let active = normalize_bridge_network_text(&bridge_tab_network(tab)) == normalized;
        for class_name in ["active", "is-active", "selected"] {
            bridge_tab_class_toggle(tab, class_name, active);
        }
        bridge_tab_set_attribute(tab, "aria-selected", if active { "true" } else { "false" });
        set(
            &property(tab, "dataset"),
            "active",
            &JsValue::from_str(if active { "true" } else { "false" }),
        );
    }

    for panel in panels {
        let active = normalize_bridge_network_text(&bridge_tab_network(panel)) == normalized;
        set(panel, "hidden", &JsValue::from_bool(!active));
        for class_name in ["active", "is-active"] {
            bridge_tab_class_toggle(panel, class_name, active);
        }
        set(
            &property(panel, "dataset"),
            "active",
            &JsValue::from_str(if active { "true" } else { "false" }),
        );
        set(
            &property(panel, "style"),
            "display",
            &JsValue::from_str(if active { "" } else { "none" }),
        );
    }

    if let Some(update) = function(callbacks, "updateCommand") {
        let _ = update.call1(callbacks, &JsValue::from_str(&normalized));
    }
    let live_callbacks_value = property(callbacks, "liveRefreshCallbacks");
    let live_callbacks = live_callbacks_value
        .clone()
        .dyn_into::<Function>()
        .ok()
        .and_then(|callback| callback.call0(callbacks).ok())
        .unwrap_or(live_callbacks_value);
    bridge_tab_refresh_later(
        normalized.clone(),
        format!("network-tab-{reason}"),
        live_callbacks.clone(),
        50,
    );
    bridge_tab_refresh_later(
        normalized.clone(),
        format!("network-tab-{reason}+700ms"),
        live_callbacks,
        700,
    );
    normalized
}

#[wasm_bindgen(js_name = bridgeInstallNetworkTabsUi)]
pub fn bridge_install_network_tabs_ui(root: JsValue, callbacks: JsValue) -> bool {
    if !present(&root) {
        return false;
    }
    let tabs = bridge_tab_collection(&root, "[data-bridge-network-tab]");
    let panels = bridge_tab_collection(&root, "[data-bridge-network-panel]");

    let tabs_for_click = tabs.clone();
    let panels_for_click = panels.clone();
    let callbacks_for_click = callbacks.clone();
    let root_for_click = root.clone();
    let click = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let tab = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-bridge-network-tab]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&tab)
            || !call1(&root_for_click, "contains", &tab)
                .is_some_and(|value| crate::js_boolean(&value))
        {
            return;
        }
        let net = bridge_tab_network(&tab);
        if normalize_bridge_network_text(&net).is_empty() {
            return;
        }
        if let Some(prevent) = function(&event, "preventDefault") {
            let _ = prevent.call0(&event);
        }
        if let Some(stop) = function(&event, "stopPropagation") {
            let _ = stop.call0(&event);
        }
        let details = Object::new();
        set(
            details.as_ref(),
            "patch",
            &JsValue::from_str(
                "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_BRIDGE_LAST_NETWORK_RESTORE_R101W2",
            ),
        );
        set(
            details.as_ref(),
            "trusted",
            &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
        );
        set(details.as_ref(), "selected", &JsValue::from_str(&net));
        set(
            details.as_ref(),
            "text",
            &JsValue::from_str(crate::js_string_owned(&property(&tab, "textContent")).trim()),
        );
        set(details.as_ref(), "persisted", &JsValue::TRUE);
        let _ = bridge_small_owner_trace_r44d(
            JsValue::from_str(&net),
            JsValue::from_str("internal-navigation"),
            JsValue::from_str("r45d-bridge-network-tab-click"),
            details.into(),
        );
        bridge_tab_select(
            &tabs_for_click,
            &panels_for_click,
            &callbacks_for_click,
            &net,
            "click",
            true,
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &root,
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    click.forget();

    let saved = bridge_read_last_network();
    let default_tab = if !saved.is_empty() {
        tabs.iter()
            .find(|tab| normalize_bridge_network_text(&bridge_tab_network(tab)) == saved)
            .cloned()
    } else {
        None
    }
    .or_else(|| {
        tabs.iter()
            .find(|tab| {
                bridge_tab_class_contains(tab, "active")
                    || bridge_tab_class_contains(tab, "is-active")
                    || bridge_tab_attribute_text(tab, "aria-selected") == "true"
                    || bridge_tab_dataset_text(tab, "active") == "true"
            })
            .cloned()
    })
    .or_else(|| {
        tabs.iter()
            .find(|tab| bridge_tab_network(tab) == "mainnet")
            .cloned()
    })
    .or_else(|| tabs.first().cloned());

    if let Some(default_tab) = default_tab {
        let selected = bridge_tab_network(&default_tab);
        bridge_tab_select(
            &tabs,
            &panels,
            &callbacks,
            &selected,
            if saved.is_empty() {
                "initial"
            } else {
                "saved-initial"
            },
            false,
        );
    }

    let tabs_external = tabs.clone();
    let panels_external = panels.clone();
    let callbacks_external = callbacks.clone();
    let external = Closure::wrap(Box::new(move |net: JsValue| {
        let selected = crate::js_string_owned(&net);
        bridge_tab_select(
            &tabs_external,
            &panels_external,
            &callbacks_external,
            &selected,
            "external",
            true,
        );
    }) as Box<dyn FnMut(JsValue)>);
    set(
        &window(),
        "kgwBridgeSelectNetworkTabR63",
        external.as_ref().unchecked_ref(),
    );
    set(
        &window(),
        "kgwBridgeSelectNetworkTabR101W2",
        external.as_ref().unchecked_ref(),
    );
    external.forget();
    true
}

#[wasm_bindgen(js_name = bridgeInstallDelegatedTabsUi)]
pub fn bridge_install_delegated_tabs_ui(root: JsValue, active_instance: JsValue) -> bool {
    if !present(&root) {
        return false;
    }
    let root_for_click = root.clone();
    let active_for_click = active_instance.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");

        let inner_tab = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-bridge-inner-tab]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if present(&inner_tab) {
            let net = bridge_tab_dataset_text(&inner_tab, "net");
            let selected = bridge_save_inner_tab(
                net.clone(),
                property(&property(&inner_tab, "dataset"), "bridgeInnerTab"),
            );
            let panel = query_bridge(
                &root_for_click,
                &format!("[data-bridge-network-panel=\"{net}\"]"),
            );
            let details = Object::new();
            set(
                details.as_ref(),
                "patch",
                &JsValue::from_str(
                    "KGW_INTERNAL_NAV_TRACE_OWNER_R45D+KGW_BRIDGE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U",
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
            let _ = bridge_small_owner_trace_r44d(
                JsValue::from_str(if net.is_empty() { "unknown" } else { &net }),
                JsValue::from_str("internal-navigation"),
                JsValue::from_str("r45d-bridge-inner-tab-click"),
                details.into(),
            );

            for item in bridge_tab_collection(&panel, "[data-bridge-inner-tab]") {
                bridge_tab_class_toggle(&item, "active", Object::is(&item, &inner_tab));
            }
            for item in bridge_tab_collection(&panel, "[data-bridge-inner-panel]") {
                let active = bridge_tab_dataset_text(&item, "bridgeInnerPanel") == selected;
                bridge_tab_class_toggle(&item, "active", active);
                set(&item, "hidden", &JsValue::from_bool(!active));
            }
            return;
        }

        let section_tab = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-bridge-section-tab]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if present(&section_tab) {
            let net = bridge_tab_dataset_text(&section_tab, "net");
            let selected = bridge_tab_dataset_text(&section_tab, "bridgeSectionTab");
            let panel = query_bridge(
                &root_for_click,
                &format!("[data-bridge-network-panel=\"{net}\"]"),
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
            let _ = bridge_small_owner_trace_r44d(
                JsValue::from_str(if net.is_empty() { "unknown" } else { &net }),
                JsValue::from_str("internal-navigation"),
                JsValue::from_str("r45d-bridge-section-tab-click"),
                details.into(),
            );

            for item in bridge_tab_collection(&panel, "[data-bridge-section-tab]") {
                let active = Object::is(&item, &section_tab);
                bridge_tab_class_toggle(&item, "active", active);
                bridge_tab_set_attribute(
                    &item,
                    "aria-selected",
                    if active { "true" } else { "false" },
                );
            }
            for item in bridge_tab_collection(&panel, "[data-bridge-section-panel]") {
                let active = bridge_tab_dataset_text(&item, "bridgeSectionPanel") == selected;
                bridge_tab_class_toggle(&item, "active", active);
                set(&item, "hidden", &JsValue::from_bool(!active));
            }
            return;
        }

        let instance_tab = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-instance-tab]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&instance_tab) {
            return;
        }
        let net = bridge_tab_dataset_text(&instance_tab, "net");
        let selected = crate::js_number(&property(
            &property(&instance_tab, "dataset"),
            "instanceTab",
        ));
        set(&active_for_click, &net, &JsValue::from_f64(selected));
        let selected_text = crate::js_string_owned(&JsValue::from_f64(selected));
        let _ = crate::bridge_raw_log::bridge_render_raw_log_buffer(
            net.clone(),
            "bridge".to_owned(),
            selected_text.clone(),
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
        set(
            details.as_ref(),
            "selected",
            &JsValue::from_str(&selected_text),
        );
        set(
            details.as_ref(),
            "text",
            &JsValue::from_str(
                crate::js_string_owned(&property(&instance_tab, "textContent")).trim(),
            ),
        );
        let _ = bridge_small_owner_trace_r44d(
            JsValue::from_str(if net.is_empty() { "unknown" } else { &net }),
            JsValue::from_str("internal-navigation"),
            JsValue::from_str("r45d-bridge-instance-tab-click"),
            details.into(),
        );

        let panel = query_bridge(
            &root_for_click,
            &format!("[data-bridge-network-panel=\"{net}\"]"),
        );
        for item in bridge_tab_collection(&panel, "[data-instance-tab]") {
            let active =
                crate::js_number(&property(&property(&item, "dataset"), "instanceTab")) == selected;
            bridge_tab_class_toggle(&item, "active", active);
        }
        for item in bridge_tab_collection(&panel, "[data-instance-panel]") {
            let active = crate::js_number(&property(&property(&item, "dataset"), "instancePanel"))
                == selected;
            bridge_tab_class_toggle(&item, "active", active);
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

fn bridge_settings_net_from_element(element: &JsValue) -> String {
    if !present(element) {
        return String::new();
    }
    let carrier = call1(
        element,
        "closest",
        &JsValue::from_str(
            "[data-net], [data-network], [data-bridge-network-panel], [data-bridge-inner-panel], [data-bridge-section-panel], [data-bridge-instance-panel]",
        ),
    )
    .unwrap_or(JsValue::UNDEFINED);
    let mut values = Vec::new();
    for value in [
        property(&property(element, "dataset"), "net"),
        property(&property(element, "dataset"), "network"),
        property(&property(&carrier, "dataset"), "net"),
        property(&property(&carrier, "dataset"), "network"),
        property(&property(&carrier, "dataset"), "bridgeNetworkPanel"),
        property(element, "id"),
        property(&carrier, "id"),
        property(&carrier, "className"),
    ] {
        let value = crate::js_string_owned(&value);
        if !value.is_empty() {
            values.push(value);
        }
    }
    let joined = values.join(" ").to_ascii_lowercase();
    if joined.contains("testnet13") || joined.contains("tn13") {
        "testnet13".to_owned()
    } else if joined.contains("testnet10") || joined.contains("tn10") {
        "testnet10".to_owned()
    } else if joined.contains("mainnet") {
        "mainnet".to_owned()
    } else {
        String::new()
    }
}

fn bridge_settings_scoped_update(
    net: &str,
    reason: &str,
    bridge_instances: &JsValue,
    callbacks: &JsValue,
) {
    if net.is_empty() {
        return;
    }
    let _ = bridge_sync_mode_controls_ui(net.to_owned(), bridge_instances.clone());
    let _ = bridge_update_command_callback(callbacks, net);
    let details = Object::new();
    set(
        details.as_ref(),
        "previousPatch",
        &JsValue::from_str("KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26"),
    );
    set(details.as_ref(), "reason", &JsValue::from_str(reason));
    let _ = bridge_small_owner_trace_r44d(
        JsValue::from_str(net),
        JsValue::from_str("settings-scope"),
        JsValue::from_str("r27d-scoped-update"),
        details.into(),
    );
}

#[wasm_bindgen(js_name = bridgeInstallSettingsEventOwnersUi)]
pub fn bridge_install_settings_event_owners_ui(
    root: JsValue,
    bridge_instances: JsValue,
    callbacks: JsValue,
) -> bool {
    if !present(&root) {
        return false;
    }

    let instances_input = bridge_instances.clone();
    let callbacks_input = callbacks.clone();
    let input = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let matches = call1(
            &target,
            "matches",
            &JsValue::from_str("input, select, textarea"),
        )
        .is_some_and(|value| crate::js_boolean(&value));
        if !matches || crate::js_boolean(&property(&target, "readOnly")) {
            return;
        }
        let id = crate::js_string_owned(&property(&target, "id"));
        if id.ends_with("-commandPreview") || id.ends_with("-logOutput") {
            return;
        }
        let net = bridge_settings_net_from_element(&target);
        let _ = crate::bridge_runtime_core::bridge_mark_restart_required_v1(net.clone());
        bridge_settings_scoped_update(
            &net,
            if crate::js_boolean(&property(&event, "isTrusted")) {
                "trusted-input"
            } else {
                "programmatic-input"
            },
            &instances_input,
            &callbacks_input,
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &root,
        "addEventListener",
        &JsValue::from_str("input"),
        input.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    input.forget();

    let instances_change = bridge_instances;
    let callbacks_change = callbacks;
    let change = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let matches = call1(
            &target,
            "matches",
            &JsValue::from_str("input, select, textarea"),
        )
        .is_some_and(|value| crate::js_boolean(&value));
        if !matches || crate::js_boolean(&property(&target, "readOnly")) {
            return;
        }
        let id = crate::js_string_owned(&property(&target, "id"));
        if id.ends_with("-commandPreview") || id.ends_with("-logOutput") {
            return;
        }

        let net = bridge_settings_net_from_element(&target);
        let _ = crate::bridge_runtime_core::bridge_mark_restart_required_v1(net.clone());
        let trusted = crate::js_boolean(&property(&event, "isTrusted"));
        let network_enabled = call1(
            &target,
            "matches",
            &JsValue::from_str("[data-bridge-network-enabled]"),
        )
        .is_some_and(|value| crate::js_boolean(&value));

        if !network_enabled {
            bridge_settings_scoped_update(
                &net,
                if trusted {
                    "trusted-change"
                } else {
                    "programmatic-change"
                },
                &instances_change,
                &callbacks_change,
            );
            return;
        }

        let target_async = target.clone();
        let net_async = net.clone();
        let instances_async = instances_change.clone();
        let callbacks_async = callbacks_change.clone();
        spawn_local(async move {
            let profile = bridge_network_profile(net_async.clone());
            let was_enabled = bridge_network_enabled(net_async.clone());
            let mut enabled = crate::js_boolean(&property(&target_async, "checked"));
            if enabled && crate::js_boolean(&property(&profile, "experimental")) {
                set(&target_async, "checked", &JsValue::FALSE);
                set(&target_async, "disabled", &JsValue::TRUE);
                let message = "Testnet 13 is experimental and uses a separate non-production runtime. Enable it only for isolated testing. Continue?";
                match crate::settings_contract::confirm_user_action(JsValue::from_str(message))
                    .await
                {
                    Ok(confirmed) => enabled = confirmed,
                    Err(error) => {
                        enabled = false;
                        let normalized =
                            crate::bridge_runtime_core::bridge_normalize_runtime_error(error);
                        let _ = crate::bridge_runtime_core::bridge_set_runtime_error_v1(
                            net_async.clone(),
                            JsValue::from_str(&normalized),
                            JsValue::from_str("settings-network-policy"),
                        );
                    }
                }
                set(&target_async, "disabled", &JsValue::FALSE);
                set(&target_async, "checked", &JsValue::from_bool(enabled));
            }

            bridge_set_network_enabled(net_async.clone(), enabled);
            crate::bridge_runtime_core::bridge_r51_set_runtime_buttons(
                net_async.clone(),
                false,
                String::new(),
                String::new(),
                String::new(),
            );
            if !enabled
                && was_enabled
                && let Some(run) = function(&callbacks_async, "runIntegratedAction")
            {
                let _ = run.call2(
                    &callbacks_async,
                    &JsValue::from_str("stop"),
                    &JsValue::from_str(&net_async),
                );
            }
            bridge_settings_scoped_update(
                &net_async,
                if trusted {
                    "trusted-change"
                } else {
                    "programmatic-change"
                },
                &instances_async,
                &callbacks_async,
            );
        });
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &root,
        "addEventListener",
        &JsValue::from_str("change"),
        change.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    change.forget();
    true
}

#[wasm_bindgen(js_name = bridgeInstallRootActionClickOwnerUi)]
pub fn bridge_install_root_action_click_owner_ui(root: JsValue, callbacks: JsValue) -> bool {
    if !present(&root) {
        return false;
    }
    let root_click = root.clone();
    let callbacks_click = callbacks;
    let click = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let button = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-bridge-action]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&button)
            || !call1(&root_click, "contains", &button)
                .is_some_and(|value| crate::js_boolean(&value))
        {
            return;
        }

        let dataset = property(&button, "dataset");
        let action = crate::js_string_owned(&property(&dataset, "bridgeAction"));
        let direct_net = {
            let net = crate::js_string_owned(&property(&dataset, "net"));
            if net.is_empty() {
                crate::js_string_owned(&property(&dataset, "network"))
            } else {
                net
            }
        };
        let net = {
            let direct = normalize_bridge_network_text(&direct_net);
            if direct.is_empty() {
                bridge_settings_net_from_element(&button)
            } else {
                direct
            }
        };
        if net.is_empty() {
            return;
        }

        let details = Object::new();
        set(
            details.as_ref(),
            "trusted",
            &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
        );
        set(
            details.as_ref(),
            "disabled",
            &JsValue::from_bool(crate::js_boolean(&property(&button, "disabled"))),
        );
        set(details.as_ref(), "id", &property(&button, "id"));
        let instance_id = {
            let value = crate::js_string_owned(&property(&dataset, "instanceId"));
            if value.is_empty() {
                crate::js_string_owned(&property(&dataset, "instance"))
            } else {
                value
            }
        };
        set(
            details.as_ref(),
            "instanceId",
            &JsValue::from_str(&instance_id),
        );
        set(
            details.as_ref(),
            "text",
            &JsValue::from_str(crate::js_string_owned(&property(&button, "textContent")).trim()),
        );
        let _ = bridge_small_owner_trace_r44d(
            JsValue::from_str(&net),
            JsValue::from_str(if action.is_empty() {
                "unknown"
            } else {
                &action
            }),
            JsValue::from_str("r27d-action-click"),
            details.into(),
        );

        match action.as_str() {
            "select-instance" => {
                let _ = call2(
                    &callbacks_click,
                    "selectInstance",
                    &JsValue::from_str(&net),
                    &JsValue::from_str(&instance_id),
                );
            }
            "add-instance" => {
                let _ = call1(&callbacks_click, "addInstance", &JsValue::from_str(&net));
            }
            "remove-instance" => {
                let _ = call2(
                    &callbacks_click,
                    "removeInstance",
                    &JsValue::from_str(&net),
                    &JsValue::from_str(&instance_id),
                );
            }
            "save-settings" => {
                let _ = call2(
                    &callbacks_click,
                    "saveSettings",
                    &JsValue::from_str(&net),
                    &button,
                );
            }
            "set-defaults" => {
                let _ = call2(
                    &callbacks_click,
                    "setDefaults",
                    &JsValue::from_str(&net),
                    &button,
                );
            }
            "restore-defaults" => {
                let _ = call2(
                    &callbacks_click,
                    "restoreDefaults",
                    &JsValue::from_str(&net),
                    &button,
                );
            }
            "copy-log" | "clear-log" => {
                let _ = call3(
                    &callbacks_click,
                    "logAction",
                    &JsValue::from_str(&action),
                    &JsValue::from_str(&net),
                    &button,
                );
            }
            "monitor-next" => {
                let _ = call2(
                    &callbacks_click,
                    "monitorNext",
                    &JsValue::from_str(&net),
                    &button,
                );
            }
            "copy-command" | "copy-path" => {
                let _ = call2(
                    &callbacks_click,
                    "copyValue",
                    &JsValue::from_str(&action),
                    &JsValue::from_str(&net),
                );
            }
            "start" | "stop" => {
                let _ = call2(
                    &callbacks_click,
                    "runtimeAction",
                    &JsValue::from_str(&action),
                    &JsValue::from_str(&net),
                );
            }
            _ => {}
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &root,
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref().unchecked_ref(),
        &JsValue::FALSE,
    );
    click.forget();
    true
}

fn bridge_queue_microtask(callback: Closure<dyn FnMut()>) {
    if let Some(queue) = function(&global(), "queueMicrotask") {
        let _ = queue.call1(&global(), callback.as_ref().unchecked_ref());
        callback.forget();
        return;
    }
    if let Some(set_timeout) = function(&window(), "setTimeout") {
        let _ = set_timeout.call2(
            &window(),
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(0.0),
        );
        callback.forget();
    }
}

fn bridge_update_command_callback(callbacks: &JsValue, net: &str) -> Result<(), JsValue> {
    let Some(update) = function(callbacks, "updateCommand") else {
        return Err(JsValue::from_str(
            "Bridge updateCommand callback is unavailable",
        ));
    };
    update.call1(callbacks, &JsValue::from_str(net)).map(|_| ())
}

fn bridge_checkbox_trace(
    net: &str,
    phase: &str,
    option: &str,
    toggle: &JsValue,
    event: Option<&JsValue>,
    extra: &[(&str, JsValue)],
) {
    let details = Object::new();
    set(details.as_ref(), "patch", &JsValue::from_str("R31"));
    set(
        details.as_ref(),
        "owner",
        &JsValue::from_str("bridge-command-composer-r7"),
    );
    set(details.as_ref(), "option", &JsValue::from_str(option));
    if let Some(event) = event {
        set(
            details.as_ref(),
            "trusted",
            &JsValue::from_bool(crate::js_boolean(&property(event, "isTrusted"))),
        );
    }
    for (key, value) in extra {
        set(details.as_ref(), key, value);
    }
    let _ = bridge_small_owner_trace_r44d(
        JsValue::from_str(net),
        JsValue::from_str("command-checkbox"),
        JsValue::from_str(phase),
        details.into(),
    );
    let _ = toggle;
}

#[wasm_bindgen(js_name = bridgeInstallActionEventOwnersUi)]
pub fn bridge_install_action_event_owners_ui(
    root: JsValue,
    bridge_instances: JsValue,
    active_instance: JsValue,
    callbacks: JsValue,
) -> bool {
    if !present(&root) {
        return false;
    }
    let dataset = property(&root, "dataset");

    if bridge_tab_dataset_text(&root, "kgwBridgeInstancesCommandCheckboxOwnerR13B").is_empty() {
        set(
            &dataset,
            "kgwBridgeInstancesCommandCheckboxOwnerR13B",
            &JsValue::from_str("1"),
        );
        let root_change = root.clone();
        let instances_change = bridge_instances.clone();
        let active_change = active_instance.clone();
        let callbacks_change = callbacks.clone();
        let change = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let include = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-instance-command-option-toggle-r13b]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&include)
                || !call1(&root_change, "contains", &include)
                    .is_some_and(|value| crate::js_boolean(&value))
            {
                return;
            }
            let include_dataset = property(&include, "dataset");
            let net = crate::js_string_owned(&property(&include_dataset, "net"));
            let instance_id = property(&include_dataset, "instanceId");
            let name = crate::js_string_owned(&property(
                &include_dataset,
                "bridgeInstanceCommandOptionToggleR13b",
            ));
            let enabled = crate::js_boolean(&property(&include, "checked"));
            let _ = crate::bridge_command_options::bridge_set_instance_command_option_ui_r13b(
                net,
                instance_id,
                name,
                enabled,
                instances_change.clone(),
                active_change.clone(),
                callbacks_change.clone(),
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("change"),
            change.as_ref().unchecked_ref(),
        );
        change.forget();
    }

    if bridge_tab_dataset_text(&root, "kgwBridgeCommandComposerInlineOwnerR7").is_empty() {
        set(
            &dataset,
            "kgwBridgeCommandComposerInlineOwnerR7",
            &JsValue::from_str("1"),
        );

        let root_pointer = root.clone();
        let pointerdown = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let toggle = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-command-option-toggle-r7]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&toggle)
                || !call1(&root_pointer, "contains", &toggle)
                    .is_some_and(|value| crate::js_boolean(&value))
            {
                return;
            }
            let toggle_dataset = property(&toggle, "dataset");
            let net = crate::js_string_owned(&property(&toggle_dataset, "net"));
            let option =
                crate::js_string_owned(&property(&toggle_dataset, "bridgeCommandOptionToggleR7"));
            bridge_checkbox_trace(
                &net,
                "r31-bridge-command-checkbox-pointerdown",
                &option,
                &toggle,
                Some(&event),
                &[
                    ("tag", property(&toggle, "tagName")),
                    ("type", property(&toggle, "type")),
                    ("checkedBefore", property(&toggle, "checked")),
                ],
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("pointerdown"),
            pointerdown.as_ref().unchecked_ref(),
        );
        pointerdown.forget();

        let root_change = root.clone();
        let callbacks_change = callbacks.clone();
        let change = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let toggle = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-command-option-toggle-r7]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&toggle)
                || !call1(&root_change, "contains", &toggle)
                    .is_some_and(|value| crate::js_boolean(&value))
            {
                return;
            }
            let toggle_dataset = property(&toggle, "dataset");
            let net = crate::js_string_owned(&property(&toggle_dataset, "net"));
            let option =
                crate::js_string_owned(&property(&toggle_dataset, "bridgeCommandOptionToggleR7"));
            let enabled = crate::js_boolean(&property(&toggle, "checked"));
            bridge_checkbox_trace(
                &net,
                "r31-bridge-command-checkbox-change-begin",
                &option,
                &toggle,
                Some(&event),
                &[("checked", JsValue::from_bool(enabled))],
            );

            crate::bridge_command_options::bridge_command_set_option_r7(
                net.clone(),
                option.clone(),
                enabled,
            );
            let update_result = bridge_update_command_callback(&callbacks_change, &net);
            crate::bridge_command_options::bridge_refresh_inline_command_toggles_r7(net.clone());

            if let Err(error) = update_result {
                bridge_checkbox_trace(
                    &net,
                    "r31-bridge-command-checkbox-change-failed",
                    &option,
                    &toggle,
                    None,
                    &[(
                        "message",
                        JsValue::from_str(&crate::js_string_owned(&error)),
                    )],
                );
                return;
            }

            let net_after = net.clone();
            let option_after = option.clone();
            let toggle_after = toggle.clone();
            bridge_queue_microtask(Closure::wrap(Box::new(move || {
                bridge_checkbox_trace(
                    &net_after,
                    "r31-bridge-command-checkbox-change-after-microtask",
                    &option_after,
                    &toggle_after,
                    None,
                    &[("checkedAfter", property(&toggle_after, "checked"))],
                );
            }) as Box<dyn FnMut()>));
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("change"),
            change.as_ref().unchecked_ref(),
        );
        change.forget();

        let root_click = root.clone();
        let callbacks_click = callbacks.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let toggle = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-command-option-toggle-r7]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&toggle)
                || !call1(&root_click, "contains", &toggle)
                    .is_some_and(|value| crate::js_boolean(&value))
            {
                return;
            }
            let toggle_dataset = property(&toggle, "dataset");
            let net = crate::js_string_owned(&property(&toggle_dataset, "net"));
            let option =
                crate::js_string_owned(&property(&toggle_dataset, "bridgeCommandOptionToggleR7"));
            let native_checkbox = call1(
                &toggle,
                "matches",
                &JsValue::from_str("input[type='checkbox']"),
            )
            .is_some_and(|value| crate::js_boolean(&value));
            bridge_checkbox_trace(
                &net,
                "r31-bridge-command-checkbox-click",
                &option,
                &toggle,
                Some(&event),
                &[
                    ("tag", property(&toggle, "tagName")),
                    ("type", property(&toggle, "type")),
                    ("isNativeCheckbox", JsValue::from_bool(native_checkbox)),
                    ("checkedAtClick", property(&toggle, "checked")),
                ],
            );

            if native_checkbox {
                if let Some(stop) = function(&event, "stopPropagation") {
                    let _ = stop.call0(&event);
                }
                let net_after = net.clone();
                let option_after = option.clone();
                let toggle_after = toggle.clone();
                bridge_queue_microtask(Closure::wrap(Box::new(move || {
                    bridge_checkbox_trace(
                        &net_after,
                        "r31-bridge-command-checkbox-click-after-microtask",
                        &option_after,
                        &toggle_after,
                        None,
                        &[("checkedAfter", property(&toggle_after, "checked"))],
                    );
                }) as Box<dyn FnMut()>));
                return;
            }
            if let Some(prevent) = function(&event, "preventDefault") {
                let _ = prevent.call0(&event);
            }
            if let Some(stop) = function(&event, "stopPropagation") {
                let _ = stop.call0(&event);
            }
            crate::bridge_command_options::bridge_command_toggle_option_r7(net.clone(), option);
            let _ = bridge_update_command_callback(&callbacks_click, &net);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("click"),
            click.as_ref().unchecked_ref(),
        );
        click.forget();

        let root_key = root.clone();
        let callbacks_key = callbacks.clone();
        let keydown = Closure::wrap(Box::new(move |event: JsValue| {
            let key = crate::js_string_owned(&property(&event, "key"));
            if key != "Enter" && key != " " {
                return;
            }
            let target = property(&event, "target");
            let toggle = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-command-option-toggle-r7]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&toggle)
                || !call1(&root_key, "contains", &toggle)
                    .is_some_and(|value| crate::js_boolean(&value))
            {
                return;
            }
            if let Some(prevent) = function(&event, "preventDefault") {
                let _ = prevent.call0(&event);
            }
            if let Some(stop) = function(&event, "stopPropagation") {
                let _ = stop.call0(&event);
            }
            let toggle_dataset = property(&toggle, "dataset");
            let net = crate::js_string_owned(&property(&toggle_dataset, "net"));
            let option =
                crate::js_string_owned(&property(&toggle_dataset, "bridgeCommandOptionToggleR7"));
            crate::bridge_command_options::bridge_command_toggle_option_r7(net.clone(), option);
            let _ = bridge_update_command_callback(&callbacks_key, &net);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("keydown"),
            keydown.as_ref().unchecked_ref(),
        );
        keydown.forget();
    }

    if bridge_tab_dataset_text(&root, "kgwBridgeInprocessNodeTabsV12B").is_empty() {
        set(
            &dataset,
            "kgwBridgeInprocessNodeTabsV12B",
            &JsValue::from_str("true"),
        );
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let tab = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-inprocess-node-tab]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&tab) {
                return;
            }
            let tab_dataset = property(&tab, "dataset");
            let net = crate::js_string_owned(&property(&tab_dataset, "net"));
            let key = crate::js_string_owned(&property(&tab_dataset, "bridgeInprocessNodeTab"));
            if net.is_empty() || key.is_empty() {
                return;
            }
            let section = call1(
                &tab,
                "closest",
                &JsValue::from_str("[data-bridge-inprocess-node-settings]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&section) {
                return;
            }
            for item in bridge_tab_collection(&section, "[data-bridge-inprocess-node-tab]") {
                bridge_tab_class_toggle(&item, "active", Object::is(&item, &tab));
            }
            for panel in bridge_tab_collection(&section, "[data-bridge-inprocess-node-panel]") {
                let active = bridge_tab_dataset_text(&panel, "bridgeInprocessNodePanel") == key;
                bridge_tab_class_toggle(&panel, "active", active);
                set(&panel, "hidden", &JsValue::from_bool(!active));
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("click"),
            click.as_ref().unchecked_ref(),
        );
        click.forget();
    }

    true
}

fn profile_object(spec: &BridgeNetworkProfile) -> JsValue {
    let output = Object::new();
    for (key, value) in [
        ("key", spec.key),
        ("label", spec.label),
        ("netsuffix", spec.netsuffix),
        ("kaspadPort", spec.kaspad_port),
        ("stratumPort", spec.stratum_port),
        ("promPort", spec.prom_port),
        ("dashboardPort", spec.dashboard_port),
        ("runtime", spec.runtime),
    ] {
        set(output.as_ref(), key, &JsValue::from_str(value));
    }
    set(
        output.as_ref(),
        "testnet",
        &JsValue::from_bool(spec.testnet),
    );
    set(
        output.as_ref(),
        "enabledByDefault",
        &JsValue::from_bool(spec.enabled_by_default),
    );
    if spec.experimental {
        set(output.as_ref(), "experimental", &JsValue::TRUE);
    }
    output.into()
}
fn translator(owner: &JsValue, name: &str) -> Option<Function> {
    function(owner, name)
}

fn runtime_feedback_terminal_text<'a>(key: &'a str, fallback: &'a str) -> &'a str {
    if fallback.is_empty() { key } else { fallback }
}

fn first_truthy_global(owner: &JsValue, names: &[&str]) -> JsValue {
    for name in names {
        let candidate = property(owner, name);
        if crate::js_boolean(&candidate) {
            return candidate;
        }
    }
    JsValue::UNDEFINED
}

fn is_plain_js_object(value: &JsValue) -> bool {
    !value.is_null() && crate::js_string_owned(&value.js_typeof()) == "object"
}

#[wasm_bindgen(js_name = bridgeTranslateRuntimeFeedback)]
pub fn bridge_translate_runtime_feedback(key: String, fallback: String) -> JsValue {
    let win = window();
    let key_value = JsValue::from_str(&key);
    let runtime = first_truthy_global(&win, &["kgwT", "kgwI18n", "__kgwT"]);
    if let Ok(callback) = runtime.dyn_into::<Function>()
        && let Ok(value) = callback.call1(&JsValue::UNDEFINED, &key_value)
        && crate::js_boolean(&value)
        && !Object::is(&value, &key_value)
    {
        return value;
    }

    let dict = first_truthy_global(
        &win,
        &[
            "__kgwI18nDictR107",
            "__kgwI18nDict",
            "kgwI18nDict",
            "__KGW_I18N_DICT__",
        ],
    );
    if is_plain_js_object(&dict) {
        let flat = property(&dict, &key);
        if flat
            .as_string()
            .is_some_and(|value| !value.trim().is_empty())
        {
            return flat;
        }

        let mut node = dict;
        for part in key.split('.') {
            if !is_plain_js_object(&node) {
                node = JsValue::NULL;
                break;
            }
            node = property(&node, part);
        }
        if node
            .as_string()
            .is_some_and(|value| !value.trim().is_empty())
        {
            return node;
        }
    }

    JsValue::from_str(runtime_feedback_terminal_text(&key, &fallback))
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

fn backend_invoke_function() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    for candidate in [
        property(&property(&tauri, "core"), "invoke"),
        property(&property(&tauri, "tauri"), "invoke"),
        property(&win, "__TAURI_INVOKE__"),
    ] {
        if let Ok(invoke) = candidate.dyn_into::<Function>() {
            return Some(invoke);
        }
    }
    None
}

fn bound_invoke(owner: &JsValue, candidate: JsValue) -> Option<Function> {
    let invoke = candidate.dyn_into::<Function>().ok()?;
    let bind = function(invoke.as_ref(), "bind")?;
    bind.call1(invoke.as_ref(), owner)
        .ok()?
        .dyn_into::<Function>()
        .ok()
}

fn trace_invoke_function() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    let core = property(&tauri, "core");
    if let Some(invoke) = bound_invoke(&core, property(&core, "invoke")) {
        return Some(invoke);
    }
    if let Some(invoke) = bound_invoke(&tauri, property(&tauri, "invoke")) {
        return Some(invoke);
    }
    property(&win, "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .ok()
}

#[wasm_bindgen(js_name = bridgeI18nTextR41)]
pub fn bridge_i18n_text_r41(key: String, fallback: String) -> JsValue {
    translate_raw(&key, &fallback)
}
#[wasm_bindgen(js_name = bridgeBackendInvokeR5)]
pub fn bridge_backend_invoke_r5(command: String, payload: JsValue) -> Result<Promise, JsValue> {
    let Some(invoke) = backend_invoke_function() else {
        return Err(JsValue::from_str("Tauri invoke is not available"));
    };
    let result = invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(&command), &payload)?;
    Ok(Promise::resolve(&result))
}

fn bridge_take_setting_cards(cards: &mut BTreeMap<String, String>, names: &str) -> String {
    let fields = names
        .split_whitespace()
        .map(|name| cards.remove(name).unwrap_or_default())
        .collect::<String>();
    format!("<div class=\"kgw-settings-grid\">{fields}</div>")
}

#[wasm_bindgen(js_name = bridgeRenderSectionsUi)]
pub fn bridge_render_sections_ui(
    profile_value: JsValue,
    callbacks: JsValue,
) -> Result<String, JsValue> {
    let net = crate::js_string_owned(&property(&profile_value, "key"));
    if profile(&net).is_none() {
        return Err(JsValue::from(Error::new("Unknown Bridge network profile")));
    }

    let mut source = String::new();
    source.push_str(&crate::bridge_render::bridge_render_runtime(
        profile_value.clone(),
    ));
    source.push_str(&crate::bridge_render::bridge_render_logging(
        profile_value.clone(),
    ));
    source.push_str(&crate::bridge_render::bridge_render_difficulty(
        profile_value.clone(),
    ));
    source.push_str(&crate::bridge_render::bridge_render_ports(
        profile_value.clone(),
    ));
    if net != "mainnet" {
        source.push_str(&crate::bridge_render::bridge_render_cpu_miner(
            profile_value.clone(),
        ));
    }

    let template = create_bridge_element("template");
    set(&template, "innerHTML", &JsValue::from_str(&source));
    let content = property(&template, "content");
    let cards_collection = call1(
        &content,
        "querySelectorAll",
        &JsValue::from_str(".bridge-v7-card"),
    )
    .ok_or_else(|| JsValue::from(Error::new("Bridge settings cards are unavailable")))?;

    let mut cards = BTreeMap::<String, String>::new();
    let prefix = format!("bridge-{net}-");
    for index in 0..bridge_collection_len(&cards_collection) {
        let Some(card) = bridge_collection_item(&cards_collection, index) else {
            continue;
        };
        let field = query_bridge(&card, "[id]");
        if !present(&field) {
            continue;
        }
        let id = crate::js_string_owned(&property(&field, "id"));
        let Some(name) = id.strip_prefix(&prefix) else {
            continue;
        };
        let testnet_allowed = matches!(name, "nodeMode" | "kaspadAddress" | "appdir" | "testnet")
            || name.starts_with("internalCpuMiner");
        if net == "mainnet" || testnet_allowed {
            cards.insert(
                name.to_owned(),
                crate::js_string_owned(&property(&card, "outerHTML")),
            );
        }
    }

    let render_inprocess: Function = property(&callbacks, "renderInprocessNodeSettings")
        .dyn_into()
        .map_err(|_| {
            JsValue::from(Error::new(
                "Bridge in-process renderer callback is unavailable",
            ))
        })?;
    let inprocess_markup = render_inprocess.call1(&callbacks, &profile_value)?;
    let inprocess = create_bridge_element("template");
    set(
        &inprocess,
        "innerHTML",
        &JsValue::from_str(&crate::js_string_owned(&inprocess_markup)),
    );
    let inprocess_content = property(&inprocess, "content");
    let danger = query_bridge(
        &inprocess_content,
        "[data-bridge-inprocess-node-panel=\"danger\"]",
    );
    if !present(&danger) {
        return Err(JsValue::from(Error::new(
            "Bridge in-process dangerous panel is unavailable",
        )));
    }
    let danger_body = crate::js_string_owned(&property(&danger, "innerHTML"));
    let _ = call0(&danger, "remove");
    let danger_tab = query_bridge(
        &inprocess_content,
        "[data-bridge-inprocess-node-tab=\"danger\"]",
    );
    if present(&danger_tab) {
        let _ = call0(&danger_tab, "remove");
    }
    let inprocess_html = crate::js_string_owned(&property(&inprocess, "innerHTML"));

    let mut groups: Vec<(&str, &str, &str, String)> = Vec::new();
    groups.push((
        "general",
        "connection",
        "Connection",
        bridge_take_setting_cards(&mut cards, "nodeMode kaspadAddress appdir testnet"),
    ));
    if net == "mainnet" {
        groups.push((
            "general",
            "ports",
            "Ports",
            bridge_take_setting_cards(
                &mut cards,
                "stratumPort promPort healthCheckPort webDashboardPort",
            ),
        ));
    }
    groups.push((
        "general",
        "mining",
        if net == "mainnet" { "Mining" } else { "CPU Mining" },
        bridge_take_setting_cards(
            &mut cards,
            if net == "mainnet" {
                "minShareDiff blockWaitTime extranonceSize coinbaseTagSuffix"
            } else {
                "internalCpuMiner internalCpuMinerAddress internalCpuMinerThreads internalCpuMinerThrottleMs internalCpuMinerTemplatePollMs"
            },
        ),
    ));
    groups.push((
        "general",
        "monitoring",
        "Monitoring",
        if net == "mainnet" {
            bridge_take_setting_cards(&mut cards, "printStats")
        } else {
            "<p class=\"kgw-settings-info\">The embedded rkstratum_cpu_miner uses this network&#39;s node. Live Bridge Monitor shows runtime state and raw logs.</p>".to_owned()
        },
    ));
    if net == "mainnet" {
        let instances = format!(
            "<div id=\"{}\">{}</div>",
            bridge_element_id(net.clone(), "instances".to_owned()),
            crate::bridge_instance_ui::bridge_render_instances_ui(
                net.clone(),
                property(&callbacks, "bridgeInstances"),
                property(&callbacks, "activeInstance"),
            )?
        );
        groups.push(("advanced", "instances", "Instances", instances));
    }
    groups.push(("advanced", "inprocessor", "In-Processor", inprocess_html));
    if net == "mainnet" {
        groups.push((
            "advanced",
            "difficulty",
            "Difficulty",
            bridge_take_setting_cards(&mut cards, "varDiff sharesPerMin varDiffStats pow2Clamp"),
        ));
    }
    groups.push((
        "advanced",
        "diagnostics",
        "Logging / Diagnostics",
        if net == "mainnet" {
            bridge_take_setting_cards(&mut cards, "config logToFile approxGeoLookup")
        } else {
            "<p class=\"kgw-settings-info\">Raw stdout/stderr logs are available in Live Bridge Monitor. Managed logging and network ownership remain unchanged.</p>".to_owned()
        },
    ));
    groups.push(("advanced", "dangerous", "Dangerous", danger_body));

    if !cards.is_empty() {
        return Err(JsValue::from(Error::new(&format!(
            "Ungrouped Bridge settings: {}",
            cards.keys().cloned().collect::<Vec<_>>().join(", ")
        ))));
    }

    Ok(format!(
        "{}{}",
        crate::bridge_render::bridge_difficulty_datalist_r16c(),
        crate::settings_layout::render_tabs_native("bridge", &net, &groups),
    ))
}

#[wasm_bindgen(js_name = bridgeRenderNetworkPanelUi)]
pub fn bridge_render_network_panel_ui(
    profile_value: JsValue,
    index: u32,
    callbacks: JsValue,
) -> Result<String, JsValue> {
    let net = crate::js_string_owned(&property(&profile_value, "key"));
    let Some(network_profile) = profile(&net) else {
        return Err(JsValue::from(Error::new("Unknown Bridge network profile")));
    };

    let render_sections: Function = property(&callbacks, "renderSections")
        .dyn_into()
        .map_err(|_| JsValue::from(Error::new("Bridge render-sections callback is unavailable")))?;
    let sections = crate::js_string_owned(&render_sections.call1(&callbacks, &profile_value)?);

    let active_inner_tab = bridge_resolve_inner_tab(net.clone());
    let log_active = active_inner_tab == "log";
    let settings_active = active_inner_tab == "settings";
    let panel_active = index == 0;
    let experimental = network_profile.experimental;
    let enabled = bridge_network_enabled(net.clone());
    let policy = escape_html_text(&bridge_network_policy_message(net.clone()));

    let policy_status = bridge_element_id(net.clone(), "policyStatus".to_owned());
    let preview_body = bridge_element_id(net.clone(), "previewBody".to_owned());
    let preview_status = bridge_element_id(net.clone(), "previewStatus".to_owned());
    let command_preview = bridge_element_id(net.clone(), "commandPreview".to_owned());
    let settings_authority = bridge_element_id(net.clone(), "settingsAuthority".to_owned());
    let runtime_status = bridge_element_id(net.clone(), "runtimeStatus".to_owned());
    let runtime_error = bridge_element_id(net.clone(), "runtimeError".to_owned());
    let monitor_state = bridge_element_id(net.clone(), "monitorState".to_owned());
    let log_empty = bridge_element_id(net.clone(), "logEmpty".to_owned());
    let log_output = bridge_element_id(net.clone(), "logOutput".to_owned());
    let badge = if experimental {
        "<span class=\"kgw-experimental-badge\">Experimental - opt-in required</span>"
    } else {
        ""
    };

    Ok(format!(
        r#"
    <div class="bridge-v7-network-panel{panel_class}" data-bridge-network-panel="{net}" data-testid="kgw-bridge-panel-{net}"{panel_hidden}>
      <section class="kgw-network-policy{experimental_class}" data-net="{net}" data-testid="kgw-bridge-policy-{net}">
        <div>
          <strong>{label}</strong>{badge}
          <span>{policy}</span>
        </div>
        <div class="kgw-network-policy-controls">
          <span id="{policy_status}" class="kgw-network-policy-status">Stopped</span>
          <label>
            <input type="checkbox" data-bridge-network-enabled="{net}" data-testid="kgw-bridge-policy-enabled-{net}" data-net="{net}"{enabled_checked}>
            Profile enabled
          </label>
        </div>
      </section>
      <div class="bridge-v7-inner-tabs">
        <button type="button" class="bridge-v7-inner-tab{log_tab_class}" data-net="{net}" data-bridge-inner-tab="log" data-testid="kgw-bridge-live-monitor-{net}">Live Bridge Monitor</button>
        <button type="button" class="bridge-v7-inner-tab{settings_tab_class}" data-net="{net}" data-bridge-inner-tab="settings" data-testid="kgw-bridge-settings-{net}">Settings</button>
      </div>

      <div class="bridge-v7-inner-panel{settings_panel_class}" data-net="{net}" data-bridge-inner-panel="settings"{settings_hidden}>
        <div class="kgw-settings-scroll">
        <section class="bridge-v7-command kgw-effective-preview">
          <div class="kgw-preview-row">
            <strong>Effective bridge settings</strong>
            <button type="button" data-settings-preview-toggle aria-expanded="false" aria-controls="{preview_body}">Expand</button>
            <button type="button" class="bridge-v7-copy" data-bridge-action="copy-command" data-net="{net}" title="Copy effective settings">Copy settings</button>
            <button type="button" data-bridge-action="copy-path" data-net="{net}">Copy data directory</button>
          </div>
          <p id="{preview_status}" class="kgw-preview-status" role="status" aria-live="polite"></p>
          <div class="kgw-preview-body" id="{preview_body}" hidden>
            <p class="kgw-preview-help">The embedded node and bridge libraries consume these settings inside KaspaGateway self-workers.</p>
            <textarea id="{command_preview}" aria-label="Effective bridge settings preview" readonly spellcheck="false" wrap="soft"></textarea>
          </div>
        </section>

        <section class="bridge-v7-toolbar">
          <div class="bridge-v7-buttons">
            <button type="button" class="good" data-bridge-action="start" data-testid="kgw-bridge-start-{net}" data-net="{net}">Start</button>
            <button type="button" data-bridge-action="stop" data-testid="kgw-bridge-stop-{net}" data-net="{net}">Stop</button>
          </div>

          <div class="bridge-v7-status">
            <span id="{settings_authority}" class="bridge-v7-runtime-status">Effective settings apply on next Start</span>
          </div>
          <div id="{runtime_status}" class="bridge-v7-runtime-status" role="status" aria-live="polite"></div>
          <div id="{runtime_error}" class="bridge-v7-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        {sections}
        </div>

        <div class="settings-bottom-actions bridge-settings-bottom-actions">
        <button type="button" data-bridge-action="save-settings" data-net="{net}">Save Settings</button>
        <button type="button" data-bridge-action="restore-defaults" data-net="{net}">Restore Defaults</button>
        <button type="button" data-bridge-action="set-defaults" data-net="{net}">Set as Defaults</button>
        <p class="kgw-settings-help" data-settings-defaults-context="{net}">Restore uses KaspaGateway defaults. Settings apply on the next Start.</p>
        </div>

      </div>

      <div class="bridge-v7-inner-panel{log_panel_class}" data-net="{net}" data-bridge-inner-panel="log" data-testid="kgw-bridge-live-panel-{net}"{log_hidden}>
        <p id="{monitor_state}" class="kgw-monitor-state" role="status">Bridge: Stopped. Node connection: not checked.</p>
        <div class="bridge-v7-log-toolbar">
          <button type="button" data-bridge-action="monitor-next" data-net="{net}">Configure Node</button>
          <button type="button" data-bridge-action="copy-log" data-testid="kgw-bridge-copy-log-{net}" data-net="{net}">Copy Log</button>
          <button type="button" data-bridge-action="clear-log" data-testid="kgw-bridge-clear-log-{net}" data-net="{net}">Clear Log</button>
        </div>
        <div id="{log_empty}" class="bridge-v7-log-empty" data-bridge-log-empty="{net}">Bridge is stopped. Choose a node connection in Settings, then start the bridge.</div>
        <pre id="{log_output}" class="bridge-v7-log" data-testid="kgw-bridge-log-output-{net}"></pre>
      </div>
</div>"#,
        panel_class = if panel_active { " active" } else { "" },
        panel_hidden = if panel_active { "" } else { " hidden" },
        experimental_class = if experimental { " is-experimental" } else { "" },
        label = network_profile.label,
        enabled_checked = if enabled { " checked" } else { "" },
        log_tab_class = if log_active { " active" } else { "" },
        settings_tab_class = if settings_active { " active" } else { "" },
        settings_panel_class = if settings_active { " active" } else { "" },
        settings_hidden = if settings_active { "" } else { " hidden" },
        log_panel_class = if log_active { " active" } else { "" },
        log_hidden = if log_active { "" } else { " hidden" },
    ))
}

#[wasm_bindgen(js_name = bridgeRenderAllNetworksUi)]
pub fn bridge_render_all_networks_ui(root: JsValue, callbacks: JsValue) -> Result<bool, JsValue> {
    let host = query_bridge(&root, "#bridgeNetworkPanels");
    if !present(&host) {
        return Ok(false);
    }

    let render: Function = property(&callbacks, "renderNetworkPanel")
        .dyn_into()
        .map_err(|_| {
            JsValue::from(Error::new(
                "Bridge render-network-panel callback is unavailable",
            ))
        })?;
    let install_layout: Function = property(&callbacks, "installSettingsLayout")
        .dyn_into()
        .map_err(|_| JsValue::from(Error::new("Bridge settings-layout callback is unavailable")))?;

    let mut panels = Vec::with_capacity(NETWORKS.len());
    for (index, profile) in NETWORKS.iter().enumerate() {
        let rendered = render.call2(
            &callbacks,
            &profile_object(profile),
            &JsValue::from_f64(index as f64),
        )?;
        panels.push(crate::js_string_owned(&rendered));
    }
    set(&host, "innerHTML", &JsValue::from_str(&panels.join("")));
    let _ = install_layout.call1(&callbacks, &root)?;

    let auto_scroll = Closure::once_into_js(move || {
        bridge_install_log_auto_scroll_controls();
    });
    if let Some(set_timeout) = function(&window(), "setTimeout") {
        let _ = set_timeout.call2(&window(), &auto_scroll, &JsValue::from_f64(0.0));
    }

    let scoped_controls = Closure::once_into_js(move || {
        let current_window = window();
        if let Some(installer) = function(&current_window, "kgwInstallBridgeLogScopedControlsV29") {
            let _ = installer.call0(&current_window);
        }
    });
    if let Some(set_timeout) = function(&window(), "setTimeout") {
        let _ = set_timeout.call2(&window(), &scoped_controls, &JsValue::from_f64(0.0));
    }

    Ok(true)
}

#[wasm_bindgen(js_name = bridgeNetworkProfiles)]
pub fn bridge_network_profiles() -> Array {
    let output = Array::new();
    for spec in &NETWORKS {
        output.push(&profile_object(spec));
    }
    output
}

#[wasm_bindgen(js_name = bridgeR51Keys)]
pub fn bridge_r51_keys() -> Array {
    let output = Array::new();
    for key in bridge_r51_key_texts() {
        output.push(&JsValue::from_str(key));
    }
    output
}

#[wasm_bindgen(js_name = bridgeNetworkPolicyKey)]
pub fn bridge_network_policy_key(net: String) -> String {
    policy_key_text(&net)
}

#[wasm_bindgen(js_name = bridgeNetworkProfile)]
pub fn bridge_network_profile(net: String) -> JsValue {
    profile(&net).map(profile_object).unwrap_or(JsValue::NULL)
}

fn bridge_instance_network_key_candidate(value: &JsValue) -> Option<String> {
    if let Some(value) = value.as_string() {
        return Some(value);
    }
    if value.is_null() || crate::js_string_owned(&value.js_typeof()) != "object" {
        return None;
    }
    property(value, "key").as_string()
}

fn bridge_instance_network_key_text(value: Option<&str>, fallback: Option<&str>) -> &'static str {
    for candidate in [value, fallback].into_iter().flatten() {
        let normalized = candidate.trim();
        if let Some(item) = profile(normalized) {
            return item.key;
        }
    }
    "mainnet"
}

#[wasm_bindgen(js_name = bridgeInstanceNetworkKeyR15)]
pub fn bridge_instance_network_key_r15(value: JsValue, fallback: JsValue) -> String {
    let value_candidate = bridge_instance_network_key_candidate(&value);
    let fallback_candidate = bridge_instance_network_key_candidate(&fallback);
    bridge_instance_network_key_text(value_candidate.as_deref(), fallback_candidate.as_deref())
        .to_owned()
}

#[wasm_bindgen(js_name = bridgeNetworkEnabled)]
pub fn bridge_network_enabled(net: String) -> bool {
    let fallback = profile(&net).is_some_and(|item| item.enabled_by_default);
    match storage_get(&policy_key_text(&net)).as_deref() {
        Some("1") => true,
        Some("0") => false,
        _ => fallback,
    }
}
#[wasm_bindgen(js_name = bridgeSetNetworkEnabled)]
pub fn bridge_set_network_enabled(net: String, enabled: bool) {
    storage_set(&policy_key_text(&net), if enabled { "1" } else { "0" });
}

#[wasm_bindgen(js_name = bridgeNetworkPolicyMessage)]
pub fn bridge_network_policy_message(net: String) -> String {
    policy_message_text(&net)
}

fn bridge_managed_message(name: &str) -> Option<&'static str> {
    BRIDGE_MANAGED
        .iter()
        .find_map(|(key, value)| (*key == name).then_some(*value))
}

fn bridge_dependency_forbidden(net: &str, name: &str) -> bool {
    net == "mainnet" && matches!(name, "internalCpuMiner" | "inprocessEnableUnsyncedMining")
}

fn bridge_dependency_state(managed: Option<&str>, forbidden: bool, active: bool) -> &'static str {
    if let Some(message) = managed {
        if message.to_ascii_lowercase().contains("unsupported") {
            "Unsupported"
        } else {
            "Managed"
        }
    } else if forbidden {
        "Test networks only"
    } else if !active {
        "Not active"
    } else {
        ""
    }
}

fn bridge_dependency_toggle_disabled(
    name: &str,
    managed: bool,
    node_mode: &str,
    internal_cpu_miner: bool,
    inprocess_perf_metrics: bool,
) -> bool {
    managed
        || (name.starts_with("inprocess") && node_mode != "inprocess")
        || (name.starts_with("internalCpuMiner")
            && name != "internalCpuMiner"
            && !internal_cpu_miner)
        || (name == "inprocessPerfMetricsIntervalSec" && !inprocess_perf_metrics)
}

fn bridge_collection_len(collection: &JsValue) -> u32 {
    crate::js_number(&property(collection, "length")).max(0.0) as u32
}

fn bridge_collection_item(collection: &JsValue, index: u32) -> Option<JsValue> {
    Reflect::get(collection, &JsValue::from_f64(index as f64)).ok()
}

fn bridge_form_values_inner(net: &str) -> JsValue {
    let values = Object::new();
    set(values.as_ref(), "network", &JsValue::from_str(net));

    let panel = crate::bridge_instance_settings::bridge_r51_panel(net.to_owned());
    if !present(&panel) {
        return values.into();
    }

    let Some(fields) = call1(
        &panel,
        "querySelectorAll",
        &JsValue::from_str(".bridge-v7-card input[id], .bridge-v7-card select[id]"),
    ) else {
        return values.into();
    };

    let prefix = format!("bridge-{net}-");
    for index in 0..bridge_collection_len(&fields) {
        let Some(field) = bridge_collection_item(&fields, index) else {
            continue;
        };
        let id = crate::js_string_owned(&property(&field, "id"));
        let Some(name) = id.strip_prefix(&prefix) else {
            continue;
        };
        let value = if crate::js_string_owned(&property(&field, "type")) == "checkbox" {
            property(&field, "checked")
        } else {
            property(&field, "value")
        };
        set(values.as_ref(), name, &value);
    }

    values.into()
}

fn bridge_instance_duration_valid(raw: &str) -> bool {
    let value = raw.trim();
    let digits = if let Some(value) = value.strip_suffix("ms") {
        value
    } else if let Some(value) = value.strip_suffix('s') {
        value
    } else {
        value
    };
    !digits.is_empty()
        && digits.as_bytes()[0].is_ascii_digit()
        && digits.as_bytes()[0] != b'0'
        && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn bridge_instance_integer_valid(raw: &str, min: u64, max: u64) -> bool {
    let value = raw.trim();
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value
            .parse::<u64>()
            .is_ok_and(|number| number >= min && number <= max)
}

fn bridge_validation_error(errors: &JsValue, key: &str, message: &str) {
    set(errors, key, &JsValue::from_str(message));
}

#[wasm_bindgen(js_name = bridgeValidateFormUi)]
pub fn bridge_validate_form_ui(net: String, bridge_instances: JsValue, focus: bool) -> JsValue {
    let values = bridge_form_values_inner(&net);
    let options = bridge_command_inline_state_r7(net.clone());
    let errors = settings_validate_bridge_form(values, options, net.clone());
    let panel = crate::bridge_instance_settings::bridge_r51_panel(net.clone());

    if net == "mainnet" && !bridge_has_config(net.clone()) {
        let instances = property(&bridge_instances, &net);
        for index in 0..bridge_collection_len(&instances) {
            let Some(instance) = bridge_collection_item(&instances, index) else {
                continue;
            };
            let instance_id_value = property(&instance, "id");
            let instance_id = crate::js_string_owned(&instance_id_value);
            if instance_id.is_empty()
                || !bridge_instance_command_should_include_from_instances_r13b(
                    bridge_instances.clone(),
                    net.clone(),
                    instance_id_value.clone(),
                    "instance".to_owned(),
                )
            {
                continue;
            }

            let wait_name = format!("instanceBlockWaitTime-{instance_id}");
            let wait_field = bridge_by_id(bridge_element_id(net.clone(), wait_name.clone()));
            if present(&wait_field)
                && bridge_instance_command_should_include_from_instances_r13b(
                    bridge_instances.clone(),
                    net.clone(),
                    instance_id_value.clone(),
                    "instanceBlockWaitTime".to_owned(),
                )
            {
                let raw = crate::js_string_owned(&property(&wait_field, "value"));
                if !bridge_instance_duration_valid(&raw) {
                    bridge_validation_error(
                        &errors,
                        &wait_name,
                        "Enter a positive duration, for example 50ms or 1s.",
                    );
                }
            }

            for (name, min, max) in [
                ("instanceDiff", 1_u64, 4_294_967_295_u64),
                ("instanceExtranonceSize", 0_u64, 8_u64),
                ("instanceSharesPerMin", 1_u64, 4_294_967_295_u64),
            ] {
                if !bridge_instance_command_should_include_from_instances_r13b(
                    bridge_instances.clone(),
                    net.clone(),
                    instance_id_value.clone(),
                    name.to_owned(),
                ) {
                    continue;
                }
                let field_name = format!("{name}-{instance_id}");
                let field = bridge_by_id(bridge_element_id(net.clone(), field_name.clone()));
                if !present(&field) {
                    continue;
                }
                let raw = crate::js_string_owned(&property(&field, "value"));
                if !bridge_instance_integer_valid(&raw, min, max) {
                    bridge_validation_error(
                        &errors,
                        &field_name,
                        &format!("Enter a whole number from {min} to {max}."),
                    );
                }
            }
        }
    }

    let _ = settings_render_field_errors(panel.clone(), format!("bridge-{net}-"), errors.clone());

    if focus {
        let keys = Object::keys(&Object::from(errors.clone()));
        if keys.length() > 0 {
            let first_key = crate::js_string_owned(&keys.get(0));
            let field = bridge_by_id(bridge_element_id(net.clone(), first_key));
            let inner_tab = query_bridge(&panel, "[data-bridge-inner-tab=\"settings\"]");
            if present(&inner_tab) {
                let _ = call0(&inner_tab, "click");
            }

            if present(&field) {
                if let Some(section) = call1(
                    &field,
                    "closest",
                    &JsValue::from_str("[data-bridge-section-panel]"),
                ) && present(&section)
                {
                    let section_name = crate::js_string_owned(&property(
                        &property(&section, "dataset"),
                        "bridgeSectionPanel",
                    ));
                    if !section_name.is_empty() {
                        let tab = query_bridge(
                            &panel,
                            &format!("[data-bridge-section-tab=\"{section_name}\"]"),
                        );
                        if present(&tab) {
                            let _ = call0(&tab, "click");
                        }
                    }
                }
                settings_reveal_field(field.clone());
                let _ = call0(&field, "focus");
            }
        }
    }

    errors
}

fn bridge_control_card_inner(element: &JsValue) -> JsValue {
    if !present(element) {
        return JsValue::UNDEFINED;
    }
    call1(element, "closest", &JsValue::from_str(".bridge-v7-card")).unwrap_or(JsValue::UNDEFINED)
}

#[wasm_bindgen(js_name = bridgeSetDisabledUi)]
pub fn bridge_set_disabled_ui(net: String, name: String, disabled: bool, reason: String) -> bool {
    let element = bridge_by_id(bridge_element_id(net, name));
    if !present(&element) {
        return false;
    }

    set(&element, "disabled", &JsValue::from_bool(disabled));
    let card = bridge_control_card_inner(&element);
    if present(&card) {
        let class_list = property(&card, "classList");
        let _ = call2(
            &class_list,
            "toggle",
            &JsValue::from_str("bridge-v7-mode-disabled"),
            &JsValue::from_bool(disabled),
        );
        set(
            &card,
            "title",
            &JsValue::from_str(if disabled { &reason } else { "" }),
        );
    }
    true
}

#[wasm_bindgen(js_name = bridgeSyncInprocessNodeSettingsV12D)]
pub fn bridge_sync_inprocess_node_settings_v12d(net: String) -> bool {
    let Some(network_profile) = profile(&net) else {
        return false;
    };

    let active = bridge_node_mode(net.clone()) == "inprocess" && !bridge_has_config(net.clone());
    let section = query_bridge(
        &document(),
        &format!("[data-bridge-inprocess-node-settings=\"{net}\"]"),
    );
    if present(&section) {
        let class_list = property(&section, "classList");
        let _ = call2(
            &class_list,
            "toggle",
            &JsValue::from_str("bridge-v12d-inprocess-inactive"),
            &JsValue::from_bool(!active),
        );
        let _ = call2(
            &class_list,
            "toggle",
            &JsValue::from_str("bridge-v12d-inprocess-active"),
            &JsValue::from_bool(active),
        );
        let dataset = property(&section, "dataset");
        set(
            &dataset,
            "kgwInprocessNodeActive",
            &JsValue::from_str(if active { "true" } else { "false" }),
        );
    }

    let appdir_mirror = bridge_by_id(bridge_element_id(
        net.clone(),
        "inprocessAppdirMirror".to_owned(),
    ));
    if present(&appdir_mirror) {
        let appdir = bridge_value(net.clone(), "appdir".to_owned());
        let value = if appdir.is_empty() {
            crate::js_string_owned(&translate_raw(
                "bridge.inprocessNodeSettings.sameAsAppdir",
                "same as --appdir",
            ))
        } else {
            appdir
        };
        set(&appdir_mirror, "value", &JsValue::from_str(&value));
        set(&appdir_mirror, "readOnly", &JsValue::TRUE);
    }

    let network_args = bridge_by_id(bridge_element_id(
        net.clone(),
        "inprocessNetworkArgs".to_owned(),
    ));
    if present(&network_args) {
        let value = if network_profile.testnet {
            if network_profile.netsuffix.is_empty() {
                "--testnet".to_owned()
            } else {
                format!("--testnet --netsuffix={}", network_profile.netsuffix)
            }
        } else {
            "mainnet".to_owned()
        };
        set(&network_args, "value", &JsValue::from_str(&value));
        set(&network_args, "readOnly", &JsValue::TRUE);
    }

    const FIELDS: &[&str] = &[
        "inprocessAppdirMirror",
        "inprocessNetworkArgs",
        "inprocessRpcListen",
        "inprocessRpcListenBorsh",
        "inprocessRpcListenJson",
        "inprocessUnsafeRpc",
        "inprocessUtxoIndex",
        "inprocessArchival",
        "inprocessListen",
        "inprocessAddPeer",
        "inprocessConnect",
        "inprocessDisableUpnp",
        "inprocessMaxInpeers",
        "inprocessOutpeers",
        "inprocessPerfMetrics",
        "inprocessPerfMetricsIntervalSec",
        "inprocessLogLevel",
        "inprocessRamScale",
        "inprocessConfigfile",
        "inprocessYes",
        "inprocessOverrideParamsFile",
        "inprocessDevnet",
        "inprocessSimnet",
        "inprocessEnableUnsyncedMining",
    ];
    const MAINNET_DANGER: &[&str] = &[
        "inprocessOverrideParamsFile",
        "inprocessDevnet",
        "inprocessSimnet",
        "inprocessEnableUnsyncedMining",
    ];
    let inactive_reason = crate::js_string_owned(&translate_raw(
        "bridge.inprocessNodeSettings.externalInactive",
        "Used only when Bridge Node Mode is In-Process.",
    ));

    for name in FIELDS {
        let mainnet_danger = net == "mainnet" && MAINNET_DANGER.contains(name);
        let reason = if mainnet_danger {
            "Dangerous development-only kaspad flag is disabled on mainnet.".to_owned()
        } else {
            inactive_reason.clone()
        };
        bridge_set_disabled_ui(
            net.clone(),
            (*name).to_owned(),
            !active || mainnet_danger,
            reason,
        );
    }

    for name in ["inprocessAppdirMirror", "inprocessNetworkArgs"] {
        let control = bridge_by_id(bridge_element_id(net.clone(), name.to_owned()));
        if present(&control) {
            set(&control, "readOnly", &JsValue::TRUE);
        }
    }

    true
}

#[wasm_bindgen(js_name = bridgeSyncModeControlsUi)]
pub fn bridge_sync_mode_controls_ui(net: String, bridge_instances: JsValue) -> bool {
    let Some(network_profile) = profile(&net) else {
        return false;
    };

    let config_mode = bridge_has_config(net.clone());
    let node_mode = bridge_node_mode(net.clone());
    let internal_miner_enabled = bridge_checked(net.clone(), "internalCpuMiner".to_owned());

    const EXPLICIT_BRIDGE_FIELDS: &[&str] = &[
        "testnet",
        "nodeMode",
        "appdir",
        "kaspadAddress",
        "blockWaitTime",
        "printStats",
        "logToFile",
        "healthCheckPort",
        "webDashboardPort",
        "varDiff",
        "sharesPerMin",
        "varDiffStats",
        "extranonceSize",
        "pow2Clamp",
        "coinbaseTagSuffix",
        "approxGeoLookup",
        "stratumPort",
        "minShareDiff",
        "promPort",
        "internalCpuMiner",
        "internalCpuMinerAddress",
        "internalCpuMinerThreads",
        "internalCpuMinerThrottleMs",
        "internalCpuMinerTemplatePollMs",
    ];

    for name in EXPLICIT_BRIDGE_FIELDS {
        bridge_set_disabled_ui(
            net.clone(),
            (*name).to_owned(),
            config_mode,
            "Config mode is active. Clear --config to edit explicit CLI flags.".to_owned(),
        );
    }

    bridge_sync_inprocess_node_settings_v12d(net.clone());

    if config_mode {
        bridge_sync_dependencies(net, bridge_instances);
        return true;
    }

    let testnet_control = bridge_by_id(bridge_element_id(net.clone(), "testnet".to_owned()));
    if present(&testnet_control) {
        set(
            &testnet_control,
            "checked",
            &JsValue::from_bool(network_profile.testnet),
        );
    }
    bridge_set_disabled_ui(
        net.clone(),
        "testnet".to_owned(),
        true,
        "Network identity is owned by the selected Mainnet/Testnet tab.".to_owned(),
    );

    bridge_set_disabled_ui(
        net.clone(),
        "kaspadAddress".to_owned(),
        node_mode != "external",
        if node_mode == "external" {
            String::new()
        } else {
            "In-process mode owns kaspad args after the -- separator.".to_owned()
        },
    );

    for name in [
        "internalCpuMinerAddress",
        "internalCpuMinerThreads",
        "internalCpuMinerThrottleMs",
        "internalCpuMinerTemplatePollMs",
    ] {
        bridge_set_disabled_ui(
            net.clone(),
            name.to_owned(),
            !internal_miner_enabled,
            "Enable --internal-cpu-miner first.".to_owned(),
        );
    }

    bridge_sync_dependencies(net, bridge_instances);
    true
}

#[wasm_bindgen(js_name = bridgeSyncAllModeControlsUi)]
pub fn bridge_sync_all_mode_controls_ui(bridge_instances: JsValue) -> bool {
    let mut synced = false;
    for net in bridge_r51_key_texts() {
        synced |= bridge_sync_mode_controls_ui(net.to_owned(), bridge_instances.clone());
    }
    synced
}

#[wasm_bindgen(js_name = bridgeBuildCommandLinesUi)]
pub fn bridge_build_command_lines_ui(
    net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> Result<Array, JsValue> {
    bridge_sync_mode_controls_ui(net.clone(), bridge_instances.clone());
    crate::bridge_port_orchestration::bridge_ensure_instance_state(
        bridge_instances.clone(),
        active_instance,
        net.clone(),
    )?;

    let instances_value = property(&bridge_instances, &net);
    let instances = if Array::is_array(&instances_value) {
        Array::from(&instances_value)
    } else {
        Array::new()
    };
    Ok(crate::bridge_command_builder::bridge_build_command_lines(
        net, instances,
    ))
}

#[wasm_bindgen(js_name = bridgeSyncDependencies)]
pub fn bridge_sync_dependencies(net: String, bridge_instances: JsValue) -> bool {
    let values = bridge_form_values_inner(&net);
    let options = bridge_command_inline_state_r7(net.clone());
    let panel = crate::bridge_instance_settings::bridge_r51_panel(net.clone());
    if !present(&panel) {
        return false;
    }

    let keys = Object::keys(&Object::from(values.clone()));
    for key in keys.iter() {
        let name = crate::js_string_owned(&key);
        if name.starts_with("instance") {
            continue;
        }
        let field = bridge_by_id(bridge_element_id(net.clone(), name.clone()));
        if !present(&field) {
            continue;
        }

        let managed = bridge_managed_message(&name);
        let forbidden = bridge_dependency_forbidden(&net, &name);
        let active = settings_bridge_field_enabled(name.clone(), values.clone(), options.clone())
            && !forbidden;
        let disabled = !active && !matches!(name.as_str(), "appdir" | "inprocessAppdirMirror");

        set(&field, "disabled", &JsValue::from_bool(disabled));
        set(&field, "readOnly", &JsValue::from_bool(managed.is_some()));

        let title = if let Some(message) = managed {
            message.to_owned()
        } else if forbidden {
            "Test networks only.".to_owned()
        } else if !active {
            "Enable the parent option to use this value.".to_owned()
        } else {
            crate::js_string_owned(&property(&field, "value"))
        };
        set(&field, "title", &JsValue::from_str(&title));

        if let Some(card) = call1(&field, "closest", &JsValue::from_str(".bridge-v7-card")) {
            let class_list = property(&card, "classList");
            let _ = call2(
                &class_list,
                "toggle",
                &JsValue::from_str("kgw-field-inactive"),
                &JsValue::from_bool(!active),
            );
            set(&card, "title", &JsValue::from_str(&title));
            let _ = call1(
                &card,
                "removeAttribute",
                &JsValue::from_str("data-i18n-title"),
            );

            let label = query_bridge(&card, ".kgw-command-option-title-text-r8e, span");
            if present(&label) {
                let label_id = format!("{}-label", crate::js_string_owned(&property(&field, "id")));
                set(&label, "id", &JsValue::from_str(&label_id));
                let _ = call2(
                    &field,
                    "setAttribute",
                    &JsValue::from_str("aria-labelledby"),
                    &JsValue::from_str(&label_id),
                );
            }
        }

        let _ = settings_set_field_state(
            field,
            bridge_dependency_state(managed, forbidden, active).to_owned(),
        );
    }

    if let Some(toggles) = call1(
        &panel,
        "querySelectorAll",
        &JsValue::from_str("[data-bridge-command-option-toggle-r7]"),
    ) {
        let node_mode = crate::js_string_owned(&property(&values, "nodeMode"));
        let internal_cpu_miner = crate::js_boolean(&property(&values, "internalCpuMiner"));
        let inprocess_perf_metrics = crate::js_boolean(&property(&values, "inprocessPerfMetrics"));

        for index in 0..bridge_collection_len(&toggles) {
            let Some(toggle) = bridge_collection_item(&toggles, index) else {
                continue;
            };
            let name = crate::js_string_owned(&property(
                &property(&toggle, "dataset"),
                "bridgeCommandOptionToggleR7",
            ));
            let disabled = bridge_dependency_toggle_disabled(
                &name,
                bridge_managed_message(&name).is_some(),
                &node_mode,
                internal_cpu_miner,
                inprocess_perf_metrics,
            );
            set(&toggle, "disabled", &JsValue::from_bool(disabled));
        }
    }

    let has_config = bridge_has_config(net.clone());
    if let Some(toggles) = call1(
        &panel,
        "querySelectorAll",
        &JsValue::from_str("[data-bridge-instance-command-option-toggle-r13b]"),
    ) {
        for index in 0..bridge_collection_len(&toggles) {
            let Some(toggle) = bridge_collection_item(&toggles, index) else {
                continue;
            };
            let dataset = property(&toggle, "dataset");
            let instance_id = crate::js_string_owned(&property(&dataset, "instanceId"));
            let name = crate::js_string_owned(&property(
                &dataset,
                "bridgeInstanceCommandOptionToggleR13b",
            ));
            if instance_id.is_empty() || name.is_empty() {
                continue;
            }

            let field = bridge_by_id(bridge_element_id(
                net.clone(),
                format!("{name}-{instance_id}"),
            ));

            if name == "instanceLogToFile" {
                set(&toggle, "disabled", &JsValue::TRUE);
                if present(&field) {
                    set(&field, "disabled", &JsValue::TRUE);
                    if let Some(message) = bridge_managed_message("logToFile") {
                        set(&field, "title", &JsValue::from_str(message));
                    }
                }
                continue;
            }

            let parent_active = !has_config
                && bridge_instance_command_should_include_from_instances_r13b(
                    bridge_instances.clone(),
                    net.clone(),
                    JsValue::from_str(&instance_id),
                    "instance".to_owned(),
                );
            set(
                &toggle,
                "disabled",
                &JsValue::from_bool(has_config || (name != "instance" && !parent_active)),
            );

            if present(&field) {
                let checked = crate::js_boolean(&property(&toggle, "checked"));
                set(
                    &field,
                    "disabled",
                    &JsValue::from_bool(!checked || !parent_active),
                );
                let state = if has_config {
                    "Managed"
                } else if !parent_active {
                    "Not active"
                } else if !checked {
                    "Override off"
                } else {
                    "Custom value"
                };
                let _ = settings_set_field_state(field.clone(), state.to_owned());

                if let Some(card) = call1(&field, "closest", &JsValue::from_str(".bridge-v7-card"))
                {
                    let label = query_bridge(&card, ".kgw-command-option-title-text-r8e");
                    if present(&label) {
                        let label_id =
                            format!("{}-label", crate::js_string_owned(&property(&field, "id")));
                        set(&label, "id", &JsValue::from_str(&label_id));
                        let _ = call2(
                            &field,
                            "setAttribute",
                            &JsValue::from_str("aria-labelledby"),
                            &JsValue::from_str(&label_id),
                        );
                    }
                }
            }
        }
    }

    let _ = settings_decorate_fields(panel);
    true
}

fn bridge_preview_message_inner(net: &str, message: &str, error: bool) -> bool {
    let element = bridge_by_id(bridge_element_id(
        net.to_owned(),
        "previewStatus".to_owned(),
    ));
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

#[wasm_bindgen(js_name = bridgePreviewMessage)]
pub fn bridge_preview_message(net: String, message: String, error: bool) -> bool {
    bridge_preview_message_inner(&net, &message, error)
}

#[derive(Default)]
struct BridgePreviewState {
    sequence: u64,
    timer: Option<JsValue>,
}

thread_local! {
    static BRIDGE_PREVIEWS: RefCell<BTreeMap<String, BridgePreviewState>> =
        const { RefCell::new(BTreeMap::new()) };
}

fn clear_bridge_preview_timer(timer: Option<JsValue>) {
    let Some(timer) = timer else {
        return;
    };
    if let Some(clear) = function(&window(), "clearTimeout") {
        let _ = clear.call1(&window(), &timer);
    }
}

fn begin_bridge_preview_sequence(net: &str) -> u64 {
    BRIDGE_PREVIEWS.with(|states| {
        let mut states = states.borrow_mut();
        let state = states.entry(net.to_owned()).or_default();
        clear_bridge_preview_timer(state.timer.take());
        state.sequence = state.sequence.saturating_add(1);
        state.sequence
    })
}

fn store_bridge_preview_timer(net: &str, sequence: u64, timer: JsValue) {
    BRIDGE_PREVIEWS.with(|states| {
        let mut states = states.borrow_mut();
        let state = states.entry(net.to_owned()).or_default();
        if state.sequence == sequence {
            state.timer = Some(timer);
        }
    });
}

fn bridge_preview_sequence_value(net: &str) -> u64 {
    BRIDGE_PREVIEWS.with(|states| {
        states
            .borrow()
            .get(net)
            .map(|state| state.sequence)
            .unwrap_or_default()
    })
}

fn bridge_preview_sequence_is_current(net: &str, sequence: u64) -> bool {
    bridge_preview_sequence_value(net) == sequence
}

fn bridge_preview_element(net: &str) -> JsValue {
    bridge_by_id(bridge_element_id(
        net.to_owned(),
        "commandPreview".to_owned(),
    ))
}

fn bridge_preview_result_json(result: &JsValue) -> String {
    JSON::stringify(result)
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned())
}

async fn finish_bridge_preview_update(
    net: String,
    sequence: u64,
    payload: JsValue,
    bridge_instances: JsValue,
    active_instance: JsValue,
) {
    let result = crate::bridge_start_trace::bridge_prepare_preview(net.clone(), payload).await;
    if !bridge_preview_sequence_is_current(&net, sequence) {
        return;
    }

    let preview = bridge_preview_element(&net);
    if !present(&preview) {
        return;
    }

    match result {
        Ok(result) => {
            let serialized = bridge_preview_result_json(&result);
            set(&preview, "value", &JsValue::from_str(&serialized));
            let dataset = property(&preview, "dataset");
            set(
                &dataset,
                "effectiveSettings",
                &JsValue::from_str(&serialized),
            );
            let _ = crate::bridge_instance_ui::bridge_sync_instance_preview_rows_r8b(
                net.clone(),
                bridge_instances,
                active_instance,
            );
            set(
                &dataset,
                "kgwBridgeCommandOwner",
                &JsValue::from_str("typed-effective-settings-preview"),
            );
            set(&dataset, "kgwBridgeNetwork", &JsValue::from_str(&net));
            let class_list = property(&preview, "classList");
            let _ = call1(
                &class_list,
                "remove",
                &JsValue::from_str("bridge-v7-command-warning"),
            );

            let app_dir = property(&result, "appDir");
            for name in ["appdir", "inprocessAppdirMirror"] {
                let field = bridge_by_id(bridge_element_id(net.clone(), name.to_owned()));
                if present(&field) {
                    set(&field, "value", &app_dir);
                    set(&field, "title", &app_dir);
                }
            }
            bridge_preview_message_inner(
                &net,
                "Validated by the same settings resolver used by Start. Embedded libraries; no external executable.",
                false,
            );
        }
        Err(error) => {
            set(&preview, "value", &JsValue::from_str(""));
            bridge_preview_message_inner(
                &net,
                &crate::bridge_runtime_core::bridge_normalize_runtime_error(error),
                true,
            );
        }
    }
}

fn bridge_update_command_inner(
    net: &str,
    bridge_instances: JsValue,
    active_instance: JsValue,
    structured_reader: JsValue,
    build_command_lines: JsValue,
) -> String {
    let preview = bridge_preview_element(net);
    if !present(&preview) {
        return String::new();
    }

    let sequence = begin_bridge_preview_sequence(net);
    set(&preview, "value", &JsValue::from_str(""));
    let dataset = property(&preview, "dataset");
    let dataset_object = Object::from(dataset.clone());
    let _ = Reflect::delete_property(&dataset_object, &JsValue::from_str("effectiveSettings"));

    let result = (|| -> Result<(JsValue, String), JsValue> {
        bridge_sync_mode_controls_ui(net.to_owned(), bridge_instances.clone());
        let active_net =
            bridge_instance_network_key_r15(JsValue::from_str(net), JsValue::from_str(net));
        let _ = crate::bridge_port_orchestration::bridge_reassign_instance_ports_from_external_range_r91(
            bridge_instances.clone(),
            active_net,
            "update-command".to_owned(),
        )?;
        crate::bridge_instance_ui::bridge_sync_instance_preview_rows_r8b(
            net.to_owned(),
            bridge_instances.clone(),
            active_instance.clone(),
        )?;
        let errors = bridge_validate_form_ui(net.to_owned(), bridge_instances.clone(), false);
        if let Some(message) = bridge_first_validation_error(&errors) {
            return Err(Error::new(&message).into());
        }

        let payload = bridge_build_apply_payload_ui(
            net.to_owned(),
            "kgw_kgw_apply_node_settings_v1".to_owned(),
            bridge_instances.clone(),
            active_instance.clone(),
            structured_reader,
            build_command_lines,
        )?;
        let command_preview = crate::js_string_owned(&property(&payload, "bridgeCommandPreview"));
        Ok((payload, command_preview))
    })();

    let (payload, command_preview) = match result {
        Ok(value) => value,
        Err(error) => {
            bridge_preview_message_inner(
                net,
                &crate::bridge_runtime_core::bridge_normalize_runtime_error(error),
                true,
            );
            return String::new();
        }
    };

    bridge_preview_message_inner(net, "Validating effective settings...", false);
    let net_for_timer = net.to_owned();
    let instances_for_timer = bridge_instances;
    let active_for_timer = active_instance;
    let callback = Closure::once_into_js(move || {
        let net_for_task = net_for_timer.clone();
        let payload_for_task = payload.clone();
        let instances_for_task = instances_for_timer.clone();
        let active_for_task = active_for_timer.clone();
        spawn_local(async move {
            finish_bridge_preview_update(
                net_for_task,
                sequence,
                payload_for_task,
                instances_for_task,
                active_for_task,
            )
            .await;
        });
    });
    if let Some(set_timeout) = function(&window(), "setTimeout")
        && let Ok(timer) = set_timeout.call2(&window(), &callback, &JsValue::from_f64(180.0))
    {
        store_bridge_preview_timer(net, sequence, timer);
    }

    command_preview
}

#[wasm_bindgen(js_name = bridgeUpdateCommandUi)]
pub fn bridge_update_command_ui(
    net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
    structured_reader: JsValue,
    build_command_lines: JsValue,
) -> String {
    bridge_update_command_inner(
        &net,
        bridge_instances,
        active_instance,
        structured_reader,
        build_command_lines,
    )
}

#[wasm_bindgen(js_name = bridgeUpdateAllCommandsUi)]
pub fn bridge_update_all_commands_ui(
    bridge_instances: JsValue,
    active_instance: JsValue,
    structured_reader: JsValue,
    build_command_lines: JsValue,
) -> bool {
    bridge_sync_all_mode_controls_ui(bridge_instances.clone());
    for net in bridge_r51_key_texts() {
        let _ = bridge_update_command_inner(
            net,
            bridge_instances.clone(),
            active_instance.clone(),
            structured_reader.clone(),
            build_command_lines.clone(),
        );
    }
    true
}

#[wasm_bindgen(js_name = bridgePreviewSequence)]
pub fn bridge_preview_sequence(net: String) -> u64 {
    bridge_preview_sequence_value(&net)
}

#[wasm_bindgen(js_name = bridgeById)]
pub fn bridge_by_id(id: String) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(&id)).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen(js_name = bridgeEscapeHtml)]
pub fn bridge_escape_html(value: JsValue) -> String {
    escape_html_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeElementId)]
pub fn bridge_element_id(net: String, name: String) -> String {
    format!("bridge-{net}-{name}")
}

#[wasm_bindgen(js_name = bridgeInstanceElementId)]
pub fn bridge_instance_element_id(net: String, instance_id: JsValue, name: String) -> String {
    format!(
        "bridge-{net}-i{}-{name}",
        crate::js_string_owned(&instance_id)
    )
}
#[wasm_bindgen(js_name = bridgeValue)]
pub fn bridge_value(net: String, name: String) -> String {
    let element = bridge_by_id(bridge_element_id(net, name));
    if !present(&element) {
        return String::new();
    }
    crate::js_string_owned(&property(&element, "value"))
        .trim()
        .to_owned()
}

fn bridge_first_validation_error(errors: &JsValue) -> Option<String> {
    if !errors.is_object() || errors.is_null() {
        return None;
    }
    let values = Object::values(&Object::from(errors.clone()));
    if values.length() > 0 {
        Some(crate::js_string_owned(&values.get(0)))
    } else {
        None
    }
}

#[wasm_bindgen(js_name = bridgeEffectiveInprocessNodeSettingsChecked)]
pub fn bridge_effective_inprocess_node_settings_checked(
    net: String,
    bridge_instances: JsValue,
) -> Result<JsValue, JsValue> {
    if bridge_node_mode(net.clone()) != "inprocess" {
        return Ok(JsValue::NULL);
    }
    let errors = bridge_validate_form_ui(net.clone(), bridge_instances, false);
    if let Some(message) = bridge_first_validation_error(&errors) {
        return Err(Error::new(&message).into());
    }
    crate::bridge_instance_settings::bridge_effective_inprocess_node_settings(net)
}

#[wasm_bindgen(js_name = bridgeRequireValidSettingsUi)]
pub fn bridge_require_valid_settings_ui(
    net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
    structured_reader: JsValue,
) -> Result<(), JsValue> {
    let errors = bridge_validate_form_ui(net.clone(), bridge_instances.clone(), true);
    if let Some(message) = bridge_first_validation_error(&errors) {
        return Err(Error::new(&message).into());
    }
    crate::bridge_port_validation::bridge_assert_no_port_conflicts_r5(
        net.clone(),
        structured_reader.clone(),
        bridge_instances.clone(),
        active_instance,
    )?;
    bridge_effective_inprocess_node_settings_checked(net.clone(), bridge_instances)?;
    let reader: Function = structured_reader.dyn_into().map_err(|_| {
        JsValue::from(Error::new(
            "Bridge structured instance reader is unavailable",
        ))
    })?;
    let structured = reader.call1(&JsValue::NULL, &JsValue::from_str(&net))?;
    let structured = if crate::js_boolean(&structured) {
        structured
    } else {
        Object::new().into()
    };
    crate::bridge_instance_settings::bridge_effective_settings_v1(net, structured)?;
    Ok(())
}
fn bridge_apply_payload_structured_instances(
    net: &str,
    bridge_instances: &JsValue,
    active_instance: &JsValue,
    structured_reader: &JsValue,
) -> Result<JsValue, JsValue> {
    if let Ok(reader) = structured_reader.clone().dyn_into::<Function>() {
        let value = reader.call1(&JsValue::UNDEFINED, &JsValue::from_str(net))?;
        if crate::js_boolean(&value) {
            return Ok(value);
        }
    }

    let fallback = Object::new();
    set(
        fallback.as_ref(),
        "activeInstance",
        &property(active_instance, net),
    );
    let instances = property(bridge_instances, net);
    let instances = if Array::is_array(&instances) {
        instances
    } else {
        Array::new().into()
    };
    set(fallback.as_ref(), "instances", &instances);
    Ok(fallback.into())
}

fn bridge_apply_payload_command_preview(
    net: &str,
    build_command_lines: &JsValue,
) -> Result<String, JsValue> {
    let builder: Function = build_command_lines.clone().dyn_into().map_err(|_| {
        JsValue::from(Error::new(
            "Bridge build-command-lines callback is unavailable",
        ))
    })?;
    let lines = builder.call1(&JsValue::UNDEFINED, &JsValue::from_str(net))?;
    let lines = Array::from(&lines);
    let mut parts = Vec::with_capacity(lines.length() as usize);
    for value in lines.iter() {
        parts.push(crate::js_string_owned(&value));
    }
    Ok(parts.join(" "))
}

fn bridge_apply_payload_active_record(
    structured_instances: &JsValue,
    active_instance_id: &str,
) -> JsValue {
    let raw_instances = property(structured_instances, "instances");
    if !Array::is_array(&raw_instances) {
        return JsValue::NULL;
    }
    let instances = Array::from(&raw_instances);
    let first = instances.get(0);
    for item in instances.iter() {
        if crate::js_string_owned(&property(&item, "id")) == active_instance_id {
            return item;
        }
    }
    first
}

fn bridge_apply_payload_active_instance_id(
    net: &str,
    structured_instances: &JsValue,
    active_instance: &JsValue,
) -> String {
    let structured_id = property(structured_instances, "activeInstance");
    if crate::js_boolean(&structured_id) {
        crate::js_string_owned(&structured_id)
    } else {
        crate::js_string_owned(&property(active_instance, net))
    }
}

#[wasm_bindgen(js_name = bridgeBuildApplyPayloadUi)]
pub fn bridge_build_apply_payload_ui(
    net: String,
    command: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
    structured_reader: JsValue,
    build_command_lines: JsValue,
) -> Result<JsValue, JsValue> {
    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(&net));

    if command == "kgw_kgw_apply_node_settings_v1" {
        crate::bridge_port_validation::bridge_assert_no_port_conflicts_r5(
            net.clone(),
            structured_reader.clone(),
            bridge_instances.clone(),
            active_instance.clone(),
        )?;

        let preview = bridge_apply_payload_command_preview(&net, &build_command_lines)?;
        let node_mode = bridge_node_mode(net.clone());
        let structured_instances = bridge_apply_payload_structured_instances(
            &net,
            &bridge_instances,
            &active_instance,
            &structured_reader,
        )?;
        let active_instance_id =
            bridge_apply_payload_active_instance_id(&net, &structured_instances, &active_instance);
        let active_record =
            bridge_apply_payload_active_record(&structured_instances, &active_instance_id);
        let active_arg = if active_record.is_null() || active_record.is_undefined() {
            String::new()
        } else {
            crate::bridge_instance_settings::bridge_build_upstream_instance_arg(
                net.clone(),
                active_record.clone(),
            )
        };
        let active_port = if active_record.is_null() || active_record.is_undefined() {
            String::new()
        } else {
            crate::js_string_owned(&property(&active_record, "instancePort"))
                .trim()
                .trim_start_matches(':')
                .to_owned()
        };

        set(
            payload.as_ref(),
            "runtimeRole",
            &JsValue::from_str("bridge"),
        );
        set(
            payload.as_ref(),
            "nodeKind",
            &JsValue::from_str(if node_mode == "inprocess" {
                "integrated-inproc"
            } else {
                "remote"
            }),
        );
        set(
            payload.as_ref(),
            "bridgeKind",
            &JsValue::from_str(if node_mode == "inprocess" {
                "official-inprocess-node"
            } else {
                "official-external-node"
            }),
        );
        set(
            payload.as_ref(),
            "nodeCommandPreview",
            &JsValue::from_str(""),
        );
        set(
            payload.as_ref(),
            "bridgeCommandPreview",
            &JsValue::from_str(&preview),
        );
        set(
            payload.as_ref(),
            "bridgeActiveInstanceId",
            &JsValue::from_str(&active_instance_id),
        );
        set(
            payload.as_ref(),
            "bridgeActiveInstance",
            &JsValue::from_str(&active_arg),
        );
        set(
            payload.as_ref(),
            "bridgeActiveInstancePort",
            &JsValue::from_str(&active_port),
        );
        let structured_text = JSON::stringify(&structured_instances)?
            .as_string()
            .unwrap_or_else(|| "{}".to_owned());
        set(
            payload.as_ref(),
            "bridgeStructuredInstances",
            &JsValue::from_str(&structured_text),
        );
        let effective_node =
            bridge_effective_inprocess_node_settings_checked(net.clone(), bridge_instances)?;
        set(payload.as_ref(), "effectiveNodeSettings", &effective_node);
        let effective_bridge = crate::bridge_instance_settings::bridge_effective_settings_v1(
            net.clone(),
            structured_instances,
        )?;
        set(
            payload.as_ref(),
            "effectiveBridgeSettings",
            &effective_bridge,
        );
        let bridge_options = crate::bridge_instance_settings::bridge_start_options(net.clone())?;
        set(payload.as_ref(), "bridgeOptions", &bridge_options);
        set(
            payload.as_ref(),
            "experimentalNetworkOptIn",
            &JsValue::from_bool(net == "testnet13" && bridge_network_enabled(net)),
        );
        return Ok(payload.into());
    }

    if matches!(
        command.as_str(),
        "kgw_kgw_disable_network_v1" | "kgw_runtime_owner_status_v1" | "kgw_kgw_runtime_logs_v1"
    ) {
        set(
            payload.as_ref(),
            "runtimeRole",
            &JsValue::from_str("bridge"),
        );
        set(
            payload.as_ref(),
            "bridgeInstanceId",
            &JsValue::from_str(&crate::js_string_owned(&property(&active_instance, &net))),
        );
    }

    Ok(payload.into())
}

fn bridge_node_mode_text(value: &str) -> &'static str {
    if value == "inprocess" {
        "inprocess"
    } else {
        "external"
    }
}

#[wasm_bindgen(js_name = bridgeNodeMode)]
pub fn bridge_node_mode(net: String) -> String {
    bridge_node_mode_text(&bridge_value(net, "nodeMode".to_owned())).to_owned()
}

#[wasm_bindgen(js_name = bridgeChecked)]
pub fn bridge_checked(net: String, name: String) -> bool {
    let element = bridge_by_id(bridge_element_id(net, name));
    present(&element) && crate::js_boolean(&property(&element, "checked"))
}

#[wasm_bindgen(js_name = bridgeSmallOwnerTraceR44D)]
pub fn bridge_small_owner_trace_r44d(
    net: JsValue,
    action: JsValue,
    phase: JsValue,
    details: JsValue,
) -> bool {
    let Some(invoke) = trace_invoke_function() else {
        return false;
    };
    let safe_net = {
        let value = crate::js_string_owned(&net);
        if value.is_empty() {
            "unknown".to_owned()
        } else {
            value
        }
    };
    let safe_action = {
        let value = crate::js_string_owned(&action);
        if value.is_empty() {
            "small-owner".to_owned()
        } else {
            value
        }
    };
    let safe_phase = {
        let value = crate::js_string_owned(&phase);
        if value.is_empty() {
            "unknown".to_owned()
        } else {
            value
        }
    };
    let safe_details = if details.is_object() && !details.is_null() {
        details
    } else {
        Object::new().into()
    };
    let nested = Object::new();
    set(
        nested.as_ref(),
        "patch",
        &JsValue::from_str("KGW_SMALL_NODE_BRIDGE_TRACE_PATCH_R44D"),
    );
    set(
        nested.as_ref(),
        "existingOwner",
        &JsValue::from_str("bridge-small-owner-functions"),
    );
    set(nested.as_ref(), "network", &JsValue::from_str(&safe_net));
    set(nested.as_ref(), "action", &JsValue::from_str(&safe_action));
    set(nested.as_ref(), "phase", &JsValue::from_str(&safe_phase));
    set(nested.as_ref(), "details", &safe_details);

    let args = Object::new();
    set(args.as_ref(), "scope", &JsValue::from_str("bridge"));
    set(args.as_ref(), "net", &JsValue::from_str(&safe_net));
    set(args.as_ref(), "action", &JsValue::from_str(&safe_action));
    set(args.as_ref(), "phase", &JsValue::from_str(&safe_phase));
    let details_text = JSON::stringify(nested.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    set(args.as_ref(), "details", &JsValue::from_str(&details_text));

    let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) else {
        return false;
    };
    let promise = Promise::resolve(&result);
    let catch = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
    let _ = promise.catch(&catch);
    catch.forget();
    true
}

#[wasm_bindgen(js_name = bridgeApplyRustyKaspaRootOnlyDefaultPathsR5)]
pub async fn bridge_apply_rusty_kaspa_root_only_default_paths_r5(
    net: String,
    update_command: Function,
) -> Result<JsValue, JsValue> {
    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(&net));
    let context = JsFuture::from(bridge_backend_invoke_r5(
        "kgw_settings_context_v1".to_owned(),
        payload.into(),
    )?)
    .await?;
    let app_dir = property(&context, "appDir");
    for name in ["appdir", "inprocessAppdirMirror"] {
        let field = bridge_by_id(bridge_element_id(net.clone(), name.to_owned()));
        if present(&field) {
            set(&field, "value", &app_dir);
            set(&field, "title", &app_dir);
        }
    }
    let _ = update_command.call1(&JsValue::UNDEFINED, &JsValue::from_str(&net));
    let output = Object::new();
    set(output.as_ref(), "appdir", &app_dir);
    Ok(output.into())
}

#[wasm_bindgen(js_name = bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5)]
pub fn bridge_apply_rusty_kaspa_root_only_default_paths_soon_r5(
    net: String,
    update_command: Function,
) {
    spawn_local(async move {
        let _ = bridge_apply_rusty_kaspa_root_only_default_paths_r5(net, update_command).await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_instance_duration_validation_matches_legacy_contract() {
        for valid in ["1", "50", "1s", "50ms", "999999ms", " 1s "] {
            assert!(bridge_instance_duration_valid(valid), "{valid}");
        }
        for invalid in [
            "", "0", "01", "0s", "0ms", "ms", "s", "1m", "-1", "1.0", "1 ms",
        ] {
            assert!(!bridge_instance_duration_valid(invalid), "{invalid}");
        }
    }

    #[test]
    fn bridge_instance_integer_validation_matches_legacy_contract() {
        assert!(bridge_instance_integer_valid("1", 1, 4_294_967_295));
        assert!(bridge_instance_integer_valid(
            "4294967295",
            1,
            4_294_967_295
        ));
        assert!(bridge_instance_integer_valid("0008", 0, 8));
        assert!(bridge_instance_integer_valid("0", 0, 8));
        assert!(!bridge_instance_integer_valid("", 0, 8));
        assert!(!bridge_instance_integer_valid("-1", 0, 8));
        assert!(!bridge_instance_integer_valid("1.0", 0, 8));
        assert!(!bridge_instance_integer_valid("9", 0, 8));
        assert!(!bridge_instance_integer_valid(
            "4294967296",
            1,
            4_294_967_295
        ));
        assert!(!bridge_instance_integer_valid(
            "9007199254740992",
            1,
            4_294_967_295
        ));
    }

    #[test]
    fn bridge_dependency_policy_matches_legacy_contract() {
        assert!(bridge_dependency_forbidden("mainnet", "internalCpuMiner"));
        assert!(bridge_dependency_forbidden(
            "mainnet",
            "inprocessEnableUnsyncedMining"
        ));
        assert!(!bridge_dependency_forbidden(
            "testnet10",
            "internalCpuMiner"
        ));
        assert_eq!(
            bridge_dependency_state(Some("unsupported by runtime"), false, true),
            "Unsupported"
        );
        assert_eq!(
            bridge_dependency_state(Some("Managed by KGW"), false, true),
            "Managed"
        );
        assert_eq!(
            bridge_dependency_state(None, true, true),
            "Test networks only"
        );
        assert_eq!(bridge_dependency_state(None, false, false), "Not active");
        assert_eq!(bridge_dependency_state(None, false, true), "");
    }

    #[test]
    fn bridge_dependency_toggle_rules_match_legacy_contract() {
        assert!(bridge_dependency_toggle_disabled(
            "config", true, "external", false, false
        ));
        assert!(bridge_dependency_toggle_disabled(
            "inprocessPerfMetricsIntervalSec",
            false,
            "external",
            false,
            false
        ));
        assert!(bridge_dependency_toggle_disabled(
            "internalCpuMinerThreads",
            false,
            "inprocess",
            false,
            true
        ));
        assert!(!bridge_dependency_toggle_disabled(
            "internalCpuMiner",
            false,
            "inprocess",
            false,
            true
        ));
        assert!(!bridge_dependency_toggle_disabled(
            "inprocessPerfMetricsIntervalSec",
            false,
            "inprocess",
            false,
            true
        ));
    }

    #[test]
    fn bridge_network_profiles_match_legacy_defaults() {
        assert_eq!(NETWORKS[0].key, "mainnet");
        assert_eq!(NETWORKS[0].kaspad_port, "16110");
        assert_eq!(NETWORKS[0].stratum_port, ":5555");
        assert_eq!(NETWORKS[1].stratum_port, ":5655");
        assert_eq!(NETWORKS[2].stratum_port, ":5755");
        assert!(!NETWORKS[2].enabled_by_default);
        assert!(NETWORKS[2].experimental);
    }
    #[test]
    fn r51_keys_follow_canonical_network_order() {
        let mut keys = bridge_r51_key_texts();
        assert_eq!(keys.next(), Some("mainnet"));
        assert_eq!(keys.next(), Some("testnet10"));
        assert_eq!(keys.next(), Some("testnet13"));
        assert_eq!(keys.next(), None);
    }

    #[test]
    fn policy_contract_matches_bridge_copy() {
        assert_eq!(
            policy_key_text("mainnet"),
            "kgw.bridge.network.enabled.mainnet"
        );
        assert_eq!(policy_key_text(""), "kgw.bridge.network.enabled.unknown");
        assert_eq!(
            policy_message_text("mainnet"),
            "Official Rusty Kaspa. External local-node mode is recommended for mining bridges."
        );
        assert!(policy_message_text("testnet13").contains("no DNS seeders"));
        assert_eq!(policy_message_text("unknown"), "");
    }

    #[test]
    fn instance_network_key_r15_matches_legacy_candidate_order() {
        assert_eq!(
            bridge_instance_network_key_text(Some(" testnet10 "), Some("mainnet")),
            "testnet10"
        );
        assert_eq!(
            bridge_instance_network_key_text(Some("mainnet"), Some("testnet13")),
            "mainnet"
        );
        assert_eq!(
            bridge_instance_network_key_text(Some("devnet"), Some(" testnet13 ")),
            "testnet13"
        );
        assert_eq!(
            bridge_instance_network_key_text(None, Some("testnet10")),
            "testnet10"
        );
        assert_eq!(
            bridge_instance_network_key_text(Some(""), Some("unknown")),
            "mainnet"
        );
        assert_eq!(bridge_instance_network_key_text(None, None), "mainnet");
    }

    #[test]
    fn r51_storage_prefix_matches_bridge_legacy_contract() {
        assert_eq!(
            bridge_r51_storage_key_text("saved:mainnet"),
            "kgw.bridge.direct.v51.saved:mainnet"
        );
        assert_eq!(
            bridge_r51_storage_key_text("factory:testnet10"),
            "kgw.bridge.direct.v51.factory:testnet10"
        );
    }

    #[test]
    fn inner_tab_persistence_policy_matches_legacy_contract() {
        assert_eq!(
            bridge_inner_tab_storage_key_text(""),
            "kgw.bridge.innerTab.unknown"
        );
        assert_eq!(
            bridge_inner_tab_storage_key_text("mainnet"),
            "kgw.bridge.innerTab.mainnet"
        );
        assert_eq!(normalize_bridge_inner_tab_text("settings"), "settings");
        assert_eq!(normalize_bridge_inner_tab_text("log"), "log");
        assert_eq!(normalize_bridge_inner_tab_text("unknown"), "log");
        assert_eq!(normalize_bridge_inner_tab_text(""), "log");
    }

    #[test]
    fn last_network_persistence_policy_matches_legacy_contract() {
        assert_eq!(BRIDGE_LAST_NETWORK_KEY, "kgw.bridge.lastNetwork");
        assert_eq!(normalize_bridge_network_text(" mainnet "), "mainnet");
        assert_eq!(normalize_bridge_network_text("testnet10"), "testnet10");
        assert_eq!(normalize_bridge_network_text("testnet13"), "testnet13");
        assert_eq!(normalize_bridge_network_text("devnet"), "");
        assert_eq!(normalize_bridge_network_text(""), "");
    }

    #[test]
    fn log_auto_scroll_persistence_policy_matches_legacy_contract() {
        assert_eq!(
            bridge_log_auto_scroll_key_text("mainnet"),
            "kgw.bridge.log.autoscroll.mainnet"
        );
        assert!(bridge_log_auto_scroll_enabled_text(None));
        assert!(bridge_log_auto_scroll_enabled_text(Some("1")));
        assert!(bridge_log_auto_scroll_enabled_text(Some("unexpected")));
        assert!(!bridge_log_auto_scroll_enabled_text(Some("0")));
    }

    #[test]
    fn runtime_feedback_terminal_fallback_matches_legacy_contract() {
        assert_eq!(
            runtime_feedback_terminal_text("runtime.failed", "Failed"),
            "Failed"
        );
        assert_eq!(
            runtime_feedback_terminal_text("runtime.failed", ""),
            "runtime.failed"
        );
        assert_eq!(runtime_feedback_terminal_text("", ""), "");
    }

    #[test]
    fn node_mode_canonicalization_matches_exact_legacy_contract() {
        assert_eq!(bridge_node_mode_text("inprocess"), "inprocess");
        assert_eq!(bridge_node_mode_text("external"), "external");
        assert_eq!(bridge_node_mode_text("In-Process"), "external");
        assert_eq!(bridge_node_mode_text(" inprocess "), "external");
        assert_eq!(bridge_node_mode_text(""), "external");
        assert_eq!(bridge_node_mode_text("remote"), "external");
    }

    #[test]
    fn port_only_normalization_matches_legacy_contract() {
        for (input, expected) in [
            ("5655", "5655"),
            (":5655", "5655"),
            ("  :080  ", "80"),
            ("00001", "1"),
            ("65535", "65535"),
            ("0", "0"),
            (":00000", ":00000"),
            ("65536", "65536"),
            ("123456", "123456"),
            (":12x", ":12x"),
            (" host:5655 ", "host:5655"),
            ("", ""),
        ] {
            assert_eq!(bridge_plain_port_only_text(input), expected);
        }
        assert_eq!(
            bridge_plain_port_only_text(":00080"),
            bridge_plain_port_only_text("80")
        );
        assert_ne!(
            bridge_plain_port_only_text("5556"),
            bridge_plain_port_only_text("5655")
        );
    }

    #[test]
    fn r95b_port_normalization_helpers_match_legacy_contract() {
        assert_eq!(
            r95b_storage_field_id("testnet10", "stratumPort"),
            "bridge-testnet10-stratumPort"
        );
        assert_eq!(r95b_storage_field_id("", "promPort"), "bridge--promPort");
        assert_eq!(
            r95b_known_stale_sequential_port("testnet10", "stratumPort"),
            "5556"
        );
        assert_eq!(
            r95b_known_stale_sequential_port("testnet10", "promPort"),
            "2113"
        );
        assert_eq!(
            r95b_known_stale_sequential_port("testnet13", "stratumPort"),
            "5557"
        );
        assert_eq!(
            r95b_known_stale_sequential_port("testnet13", "promPort"),
            "2114"
        );
        assert_eq!(
            r95b_known_stale_sequential_port("mainnet", "stratumPort"),
            ""
        );
        assert!(r95b_same_port(":05655", "5655"));
        assert!(!r95b_same_port("5556", "5655"));
    }

    #[test]
    fn escape_and_ids_match_legacy_contract() {
        assert_eq!(
            escape_html_text(r#"<a x="1">&"#),
            "&lt;a x=&quot;1&quot;&gt;&amp;"
        );
        assert_eq!(
            bridge_element_id("mainnet".to_owned(), "rpcPort".to_owned()),
            "bridge-mainnet-rpcPort"
        );
    }
}
