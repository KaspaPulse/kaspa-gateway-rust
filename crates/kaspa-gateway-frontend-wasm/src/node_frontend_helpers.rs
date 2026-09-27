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
