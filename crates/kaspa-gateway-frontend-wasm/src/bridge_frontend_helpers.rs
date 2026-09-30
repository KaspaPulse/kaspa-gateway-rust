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
