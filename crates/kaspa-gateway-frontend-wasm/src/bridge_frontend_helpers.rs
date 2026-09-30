use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

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

fn call1(target: &JsValue, name: &str, first: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, first).ok()
}

fn call2(target: &JsValue, name: &str, first: &JsValue, second: &JsValue) -> Option<JsValue> {
    function(target, name)?.call2(target, first, second).ok()
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

#[wasm_bindgen(js_name = bridgeNetworkProfiles)]
pub fn bridge_network_profiles() -> Array {
    let output = Array::new();
    for spec in &NETWORKS {
        output.push(&profile_object(spec));
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

#[cfg(test)]
mod tests {
    use super::*;

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
