use super::settings_schema::{
    BRIDGE_MANAGED, BRIDGE_OPTIONAL, BRIDGE_REQUIRED, NODE_DANGEROUS, NODE_ENDPOINTS, NODE_MANAGED,
    NODE_OPTIONAL, NODE_REQUIRED,
};
use super::{
    call_method0, call_method1, get_property, js_boolean, js_number, js_string, method,
    set_property,
};
use js_sys::{Array, Function, Object, Promise, Reflect};
use std::net::Ipv6Addr;
use std::str::FromStr;
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

fn raw_string(value: &JsValue) -> String {
    String::from(js_string(value))
}

fn legacy_or_empty(value: &JsValue) -> String {
    if js_boolean(value) {
        raw_string(value)
    } else {
        String::new()
    }
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        JsValue::UNDEFINED
    } else {
        Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
    }
}

fn text_property(target: &JsValue, name: &str) -> String {
    let value = property(target, name);
    let source = if value.is_null() || value.is_undefined() {
        JsValue::from_str("")
    } else {
        value
    };
    raw_string(&source).trim().to_owned()
}

fn strict_true(value: &JsValue) -> bool {
    value.as_bool() == Some(true)
}

fn strict_false(value: &JsValue) -> bool {
    value.as_bool() == Some(false)
}

fn strip_edge_brackets(value: &str) -> &str {
    let value = value.strip_prefix('[').unwrap_or(value);
    value.strip_suffix(']').unwrap_or(value)
}

fn is_host_text(value: &str) -> bool {
    let host = strip_edge_brackets(value.trim());
    if host.is_empty()
        || host.len() > 253
        || host
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '/' | '\\' | '@' | '?' | '#'))
    {
        return false;
    }

    if host.contains(':') {
        return Ipv6Addr::from_str(host).is_ok();
    }

    if host.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        let parts = host.split('.').collect::<Vec<_>>();
        return parts.len() == 4
            && parts.iter().all(|part| {
                (1..=3).contains(&part.len())
                    && part.bytes().all(|byte| byte.is_ascii_digit())
                    && part.parse::<u16>().is_ok_and(|number| number <= 255)
            });
    }

    let hostname = host.strip_suffix('.').unwrap_or(host);
    hostname.split('.').all(|part| {
        if part.is_empty() || part.len() > 63 {
            return false;
        }
        let bytes = part.as_bytes();
        bytes[0].is_ascii_alphanumeric()
            && bytes[bytes.len() - 1].is_ascii_alphanumeric()
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
    })
}

fn is_port_text(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value
            .parse::<u32>()
            .is_ok_and(|number| (1..=65_535).contains(&number))
}

fn split_endpoint_text(value: &str) -> Option<(String, String)> {
    let text = value.trim();
    let separator = text.rfind(':')?;
    let (host, suffix) = text.split_at(separator);
    let port = suffix.strip_prefix(':')?;
    if port.is_empty() || !port.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some((strip_edge_brackets(host).to_owned(), port.to_owned()))
}

fn is_loopback_text(value: &str) -> bool {
    let normalized = strip_edge_brackets(value).to_ascii_lowercase();
    normalized == "localhost"
        || normalized == "::1"
        || (normalized.starts_with("127.") && is_host_text(&normalized))
}

#[derive(Clone)]
struct Listener {
    host: String,
    port: String,
    field: String,
}

fn listeners_overlap_core(a_host: &str, a_port: f64, b_host: &str, b_port: f64) -> bool {
    if a_port != b_port {
        return false;
    }
    let normalize = |host: &str| {
        if is_loopback_text(host) {
            "loopback".to_owned()
        } else {
            host.to_ascii_lowercase()
        }
    };
    normalize(a_host) == normalize(b_host)
        || [a_host, b_host]
            .iter()
            .any(|host| matches!(*host, "0.0.0.0" | "::"))
}

fn absolute_directory_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
    {
        return true;
    }
    if value.starts_with('/') {
        return true;
    }
    if let Some(rest) = value.strip_prefix(r"\\") {
        let mut parts = rest.split('\\');
        return parts.next().is_some_and(|part| !part.is_empty())
            && parts.next().is_some_and(|part| !part.is_empty());
    }
    false
}

fn positive_duration(value: &str) -> bool {
    let (digits, suffix) = if let Some(value) = value.strip_suffix("ms") {
        (value, "ms")
    } else if let Some(value) = value.strip_suffix('s') {
        (value, "s")
    } else {
        (value, "")
    };
    let _ = suffix;
    !digits.is_empty()
        && digits.as_bytes()[0].is_ascii_digit()
        && digits.as_bytes()[0] != b'0'
        && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn contains_key(pairs: &[(&str, &str)], name: &str) -> bool {
    pairs.iter().any(|(key, _)| *key == name)
}

fn contains_name(names: &[&str], name: &str) -> bool {
    names.contains(&name)
}

fn set_error(errors: &Object, key: &str, message: &str) {
    let _ = Reflect::set(
        errors.as_ref(),
        &JsValue::from_str(key),
        &JsValue::from_str(message),
    );
}

fn number_for_text(value: &str) -> f64 {
    js_number(&JsValue::from_str(value))
}

fn safe_integer(number: f64) -> bool {
    number.is_finite() && number.fract() == 0.0 && number.abs() <= MAX_SAFE_INTEGER
}

fn bound_text(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn node_field_enabled_inner(name: &str, values: &JsValue, options: &JsValue) -> bool {
    if contains_key(NODE_MANAGED, name) {
        return false;
    }
    if let Some((parent, _, _, _, _)) = NODE_ENDPOINTS
        .iter()
        .find(|(_, host, port, _, _)| *host == name || *port == name)
    {
        return strict_true(&property(values, parent));
    }
    if name == "logDir" && js_boolean(&property(values, "noLogFiles")) {
        return false;
    }
    if name == "perfMetricsInterval" && !js_boolean(&property(values, "perfMetrics")) {
        return false;
    }
    if name == "rocksDbCacheSize"
        && (strict_false(&property(options, "rocksDbPreset"))
            || text_property(values, "rocksDbPreset") != "hdd")
    {
        return false;
    }
    if contains_key(NODE_REQUIRED, name) {
        return true;
    }
    !contains_name(NODE_OPTIONAL, name) || strict_true(&property(options, name))
}

fn bridge_field_enabled_inner(
    name: &str,
    values: &JsValue,
    options: &JsValue,
    network_override: Option<&str>,
) -> bool {
    if contains_key(BRIDGE_MANAGED, name) {
        return false;
    }
    let network = network_override
        .map(ToOwned::to_owned)
        .or_else(|| property(values, "network").as_string())
        .unwrap_or_default();
    if matches!(network.as_str(), "testnet10" | "testnet13")
        && !name.starts_with("inprocess")
        && !name.starts_with("internalCpuMiner")
        && !matches!(name, "nodeMode" | "kaspadAddress" | "appdir" | "testnet")
    {
        return false;
    }

    let config_value = property(values, "config");
    let config = strict_true(&property(options, "config"))
        && !legacy_or_empty(&config_value).trim().is_empty();
    if config && !matches!(name, "config" | "nodeMode") && !name.starts_with("inprocess") {
        return false;
    }
    if name.starts_with("inprocess")
        && property(values, "nodeMode").as_string().as_deref() != Some("inprocess")
    {
        return false;
    }
    if name == "kaspadAddress"
        && property(values, "nodeMode").as_string().as_deref() != Some("external")
    {
        return false;
    }
    if name.starts_with("internalCpuMiner")
        && name != "internalCpuMiner"
        && !js_boolean(&property(values, "internalCpuMiner"))
    {
        return false;
    }
    if name == "inprocessPerfMetricsIntervalSec"
        && !js_boolean(&property(values, "inprocessPerfMetrics"))
    {
        return false;
    }
    if contains_name(BRIDGE_REQUIRED, name) {
        return true;
    }
    if contains_name(BRIDGE_OPTIONAL, name) {
        return strict_true(&property(options, name));
    }
    !strict_false(&property(options, name))
}

fn pairs_object(pairs: &[(&str, &str)]) -> Object {
    let object = Object::new();
    for (key, value) in pairs {
        let _ = Reflect::set(
            object.as_ref(),
            &JsValue::from_str(key),
            &JsValue::from_str(value),
        );
    }
    object
}

fn names_array(names: &[&str]) -> Array {
    let array = Array::new();
    for value in names {
        array.push(&JsValue::from_str(value));
    }
    array
}

#[wasm_bindgen(js_name = settingsNodeEndpoints)]
pub fn node_endpoints() -> Array {
    let rows = Array::new();
    for (parent, host, port, output, rpc) in NODE_ENDPOINTS {
        let row = Array::new();
        row.push(&JsValue::from_str(parent));
        row.push(&JsValue::from_str(host));
        row.push(&JsValue::from_str(port));
        row.push(&JsValue::from_str(output));
        row.push(&JsValue::from_bool(*rpc));
        rows.push(&row);
    }
    rows
}

#[wasm_bindgen(js_name = settingsNodeManaged)]
pub fn node_managed() -> Object {
    pairs_object(NODE_MANAGED)
}

#[wasm_bindgen(js_name = settingsNodeRequired)]
pub fn node_required() -> Object {
    pairs_object(NODE_REQUIRED)
}

#[wasm_bindgen(js_name = settingsNodeOptional)]
pub fn node_optional() -> Array {
    names_array(NODE_OPTIONAL)
}

#[wasm_bindgen(js_name = settingsNodeDangerous)]
pub fn node_dangerous() -> Object {
    pairs_object(NODE_DANGEROUS)
}

#[wasm_bindgen(js_name = settingsBridgeManaged)]
pub fn bridge_managed() -> Object {
    pairs_object(BRIDGE_MANAGED)
}

#[wasm_bindgen(js_name = settingsBridgeRequired)]
pub fn bridge_required() -> Array {
    names_array(BRIDGE_REQUIRED)
}

#[wasm_bindgen(js_name = settingsBridgeOptional)]
pub fn bridge_optional() -> Array {
    names_array(BRIDGE_OPTIONAL)
}

#[wasm_bindgen(js_name = settingsIsHost)]
pub fn is_host(value: JsValue) -> bool {
    is_host_text(&legacy_or_empty(&value))
}

#[wasm_bindgen(js_name = settingsIsPort)]
pub fn is_port(value: JsValue) -> bool {
    is_port_text(&raw_string(&value))
}

#[wasm_bindgen(js_name = settingsEndpoint)]
pub fn endpoint(host: JsValue, port: JsValue) -> String {
    let plain = strip_edge_brackets(&raw_string(&host)).to_owned();
    let host = if plain.contains(':') {
        format!("[{plain}]")
    } else {
        plain
    };
    format!("{host}:{}", raw_string(&port))
}

#[wasm_bindgen(js_name = settingsSplitEndpoint)]
pub fn split_endpoint(value: JsValue) -> JsValue {
    let Some((host, port)) = split_endpoint_text(&legacy_or_empty(&value)) else {
        return JsValue::NULL;
    };
    let result = Object::new();
    let _ = Reflect::set(
        result.as_ref(),
        &JsValue::from_str("host"),
        &JsValue::from_str(&host),
    );
    let _ = Reflect::set(
        result.as_ref(),
        &JsValue::from_str("port"),
        &JsValue::from_str(&port),
    );
    result.into()
}

#[wasm_bindgen(js_name = settingsIsLoopback)]
pub fn is_loopback(host: JsValue) -> bool {
    is_loopback_text(&raw_string(&host))
}

#[wasm_bindgen(js_name = settingsListenersOverlap)]
pub fn listeners_overlap(a: JsValue, b: JsValue) -> bool {
    if !js_boolean(&a) || !js_boolean(&b) {
        return false;
    }
    let a_port = js_number(&property(&a, "port"));
    let b_port = js_number(&property(&b, "port"));
    listeners_overlap_core(
        &raw_string(&property(&a, "host")),
        a_port,
        &raw_string(&property(&b, "host")),
        b_port,
    )
}

#[wasm_bindgen(js_name = settingsNodeFieldEnabled)]
pub fn node_field_enabled(name: String, values: JsValue, options: JsValue) -> bool {
    node_field_enabled_inner(&name, &values, &options)
}

#[wasm_bindgen(js_name = settingsValidateNodeForm)]
pub fn validate_node_form(values: JsValue, options: JsValue, network: String) -> JsValue {
    let errors = Object::new();

    for (parent, host_key, port_key, _, rpc) in NODE_ENDPOINTS {
        if !js_boolean(&property(&values, parent)) {
            continue;
        }
        let host = text_property(&values, host_key);
        let port = text_property(&values, port_key);
        if !is_host_text(&host) {
            set_error(&errors, host_key, "Enter a valid IP address or hostname.");
        }
        if !is_port_text(&port) {
            set_error(&errors, port_key, "Enter a whole port from 1 to 65535.");
        }
        if *rpc
            && !js_boolean(&property(&values, "unsafeRpc"))
            && is_host_text(&host)
            && !is_loopback_text(&host)
        {
            set_error(
                &errors,
                host_key,
                "RPC must use loopback unless unsafe RPC is explicitly enabled.",
            );
        }
    }

    if !js_boolean(&property(&values, "rpcListenEnabled")) {
        set_error(
            &errors,
            "rpcListenEnabled",
            "gRPC is required in managed mode.",
        );
    }
    if js_boolean(&property(&values, "noGrpc")) {
        set_error(
            &errors,
            "noGrpc",
            "gRPC cannot be disabled in managed mode.",
        );
    }
    if js_boolean(&property(&values, "connectEnabled"))
        && js_boolean(&property(&values, "addPeerEnabled"))
    {
        set_error(
            &errors,
            "addPeerHost",
            "Use either connect-only peers or additional peers, not both.",
        );
    }

    let ranges = [
        ("asyncThreads", 1.0, MAX_SAFE_INTEGER, true),
        ("ramScale", 0.1, 10.0, false),
        ("rpcMaxClients", 1.0, 16.0, true),
        ("outPeers", 0.0, 8.0, true),
        ("maxInPeers", 0.0, 32.0, true),
        ("maxTrackedAddresses", 0.0, MAX_SAFE_INTEGER, true),
        ("retentionDays", f64::from_bits(1), MAX_SAFE_INTEGER, false),
        ("perfMetricsInterval", 1.0, MAX_SAFE_INTEGER, true),
        ("rocksDbCacheSize", 1.0, MAX_SAFE_INTEGER, true),
    ];
    for (key, min, max, integer) in ranges {
        if !node_field_enabled_inner(key, &values, &options) {
            continue;
        }
        let raw = text_property(&values, key);
        let number = number_for_text(&raw);
        let invalid_integer =
            integer && (!safe_integer(number) || !raw.bytes().all(|byte| byte.is_ascii_digit()));
        if raw.is_empty() || !number.is_finite() || number < min || number > max || invalid_integer
        {
            let kind = if integer {
                "a whole number"
            } else {
                "a number"
            };
            let range = if max < MAX_SAFE_INTEGER {
                format!(" from {} to {}", bound_text(min), bound_text(max))
            } else if min == 0.0 {
                " greater than or equal to 0".to_owned()
            } else {
                " greater than 0".to_owned()
            };
            set_error(&errors, key, &format!("Enter {kind}{range}."));
        }
    }

    for key in NODE_OPTIONAL {
        if !node_field_enabled_inner(key, &values, &options) {
            continue;
        }
        let text = text_property(&values, key);
        if text.is_empty() {
            set_error(&errors, key, "Enter a value or turn this option off.");
        } else if text.len() > 4096 || text.chars().any(|ch| (ch as u32) <= 0x1f) {
            set_error(
                &errors,
                key,
                "Use at most 4096 characters without control characters.",
            );
        }
    }

    for key in ["rocksDbWalDir", "logDir"] {
        let text = text_property(&values, key);
        if node_field_enabled_inner(key, &values, &options)
            && !text.is_empty()
            && !absolute_directory_path(&text)
        {
            set_error(&errors, key, "Enter a full directory path.");
        }
    }

    if network == "mainnet" && js_boolean(&property(&values, "enableUnsyncedMining")) {
        set_error(
            &errors,
            "enableUnsyncedMining",
            "Available only for test networks.",
        );
    }

    let mut listeners = Vec::new();
    for (parent, host, port, _, _) in NODE_ENDPOINTS.iter().take(4) {
        if js_boolean(&property(&values, parent)) {
            listeners.push(Listener {
                host: text_property(&values, host),
                port: text_property(&values, port),
                field: (*port).to_owned(),
            });
        }
    }
    for index in 0..listeners.len() {
        for other in listeners.iter().skip(index + 1) {
            if listeners_overlap_core(
                &listeners[index].host,
                number_for_text(&listeners[index].port),
                &other.host,
                number_for_text(&other.port),
            ) {
                set_error(
                    &errors,
                    &other.field,
                    "This listener conflicts with another enabled listener.",
                );
            }
        }
    }

    errors.into()
}

#[wasm_bindgen(js_name = settingsBridgeFieldEnabled)]
pub fn bridge_field_enabled(name: String, values: JsValue, options: JsValue) -> bool {
    bridge_field_enabled_inner(&name, &values, &options, None)
}

#[wasm_bindgen(js_name = settingsValidateBridgeForm)]
pub fn validate_bridge_form(values: JsValue, options: JsValue, network: String) -> JsValue {
    let errors = Object::new();
    let active = |key: &str| bridge_field_enabled_inner(key, &values, &options, Some(&network));

    for key in BRIDGE_OPTIONAL {
        if active(key) && text_property(&values, key).is_empty() {
            set_error(&errors, key, "Enter a value or turn this option off.");
        }
    }

    let config = text_property(&values, "config");
    if active("config") && !config.is_empty() && !absolute_directory_path(&config) {
        set_error(
            &errors,
            "config",
            "Enter the full path to an existing bridge configuration file.",
        );
    }

    for key in [
        "kaspadAddress",
        "inprocessRpcListen",
        "inprocessRpcListenBorsh",
        "inprocessRpcListenJson",
        "inprocessListen",
        "inprocessAddPeer",
        "inprocessConnect",
    ] {
        if !active(key) {
            continue;
        }
        let text = text_property(&values, key);
        match split_endpoint_text(&text) {
            Some((host, port)) if is_host_text(&host) && is_port_text(&port) => {
                if key.starts_with("inprocessRpc")
                    && !js_boolean(&property(&values, "inprocessUnsafeRpc"))
                    && !is_loopback_text(&host)
                {
                    set_error(
                        &errors,
                        key,
                        "RPC must use loopback unless unsafe RPC is explicitly enabled.",
                    );
                }
            }
            _ => set_error(&errors, key, "Enter a valid host:port (port 1 to 65535)."),
        }
    }

    for key in ["stratumPort", "promPort"] {
        if !active(key) {
            continue;
        }
        let raw = text_property(&values, key);
        let endpoint = if raw.contains(':') {
            raw
        } else {
            format!(":{raw}")
        };
        let valid = split_endpoint_text(&endpoint).is_some_and(|(host, port)| {
            is_port_text(&port) && (host.is_empty() || is_host_text(&host))
        });
        if !valid {
            set_error(&errors, key, "Enter a listener port from 1 to 65535.");
        }
    }

    let ranges = [
        ("minShareDiff", 1.0, 4_294_967_295.0),
        ("sharesPerMin", 1.0, 4_294_967_295.0),
        ("extranonceSize", 0.0, 8.0),
        ("inprocessOutpeers", 0.0, 8.0),
        ("inprocessMaxInpeers", 0.0, 32.0),
        ("inprocessPerfMetricsIntervalSec", 1.0, MAX_SAFE_INTEGER),
        ("inprocessAsyncThreads", 1.0, 65_535.0),
        ("internalCpuMinerThreads", 1.0, 256.0),
        ("internalCpuMinerThrottleMs", 0.0, 60_000.0),
        ("internalCpuMinerTemplatePollMs", 1.0, 60_000.0),
    ];
    for (key, min, max) in ranges {
        if !active(key) {
            continue;
        }
        let raw = text_property(&values, key);
        if key.starts_with("internalCpuMiner") && raw.is_empty() && key != "internalCpuMinerThreads"
        {
            continue;
        }
        let number = number_for_text(&raw);
        if !raw.bytes().all(|byte| byte.is_ascii_digit())
            || !safe_integer(number)
            || number < min
            || number > max
        {
            set_error(
                &errors,
                key,
                &format!(
                    "Enter a whole number from {} to {}.",
                    bound_text(min),
                    bound_text(max)
                ),
            );
        }
    }

    if active("inprocessRamScale") {
        let raw = text_property(&values, "inprocessRamScale");
        let number = number_for_text(&raw);
        if raw.is_empty() || !number.is_finite() || !(0.1..=10.0).contains(&number) {
            set_error(
                &errors,
                "inprocessRamScale",
                "Enter a number from 0.1 to 10.",
            );
        }
    }

    let block_wait = text_property(&values, "blockWaitTime");
    if active("blockWaitTime") && !positive_duration(&block_wait) {
        set_error(
            &errors,
            "blockWaitTime",
            "Enter a positive duration, for example 50ms or 1s.",
        );
    }
    if active("inprocessConnect") && active("inprocessAddPeer") {
        set_error(
            &errors,
            "inprocessAddPeer",
            "Use either connect-only peers or additional peers, not both.",
        );
    }
    if js_boolean(&property(&values, "internalCpuMiner"))
        && text_property(&values, "internalCpuMinerAddress").is_empty()
    {
        set_error(
            &errors,
            "internalCpuMinerAddress",
            "Enter a test-network mining address.",
        );
    }
    if network == "mainnet" && js_boolean(&property(&values, "internalCpuMiner")) {
        set_error(
            &errors,
            "internalCpuMiner",
            "CPU mining is available only for test networks.",
        );
    }
    if network == "mainnet" && js_boolean(&property(&values, "inprocessEnableUnsyncedMining")) {
        set_error(
            &errors,
            "inprocessEnableUnsyncedMining",
            "Available only for test networks.",
        );
    }

    errors.into()
}

fn collection_items(collection: &JsValue) -> Vec<JsValue> {
    let length = property(collection, "length")
        .as_f64()
        .unwrap_or(0.0)
        .max(0.0) as u32;
    (0..length)
        .map(|index| {
            Reflect::get(collection, &JsValue::from_f64(index as f64)).unwrap_or(JsValue::UNDEFINED)
        })
        .collect()
}

fn call_method2(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    method(target, name)?.call2(target, first, second)
}

#[wasm_bindgen(js_name = settingsRenderFieldErrors)]
pub fn render_field_errors(root: JsValue, prefix: String, errors: JsValue) -> Result<(), JsValue> {
    if !js_boolean(&root) {
        return Ok(());
    }

    let existing = call_method1(
        &root,
        "querySelectorAll",
        &JsValue::from_str("[data-settings-error]"),
    )?;
    for element in collection_items(&existing) {
        let _ = call_method0(&element, "remove")?;
    }

    let invalid = call_method1(
        &root,
        "querySelectorAll",
        &JsValue::from_str("[aria-invalid=\"true\"]"),
    )?;
    for element in collection_items(&invalid) {
        let _ = call_method1(
            &element,
            "removeAttribute",
            &JsValue::from_str("aria-invalid"),
        )?;
        let dataset = get_property(&element, "dataset")?;
        let error_id = property(&dataset, "settingsErrorId");
        if js_boolean(&error_id) {
            let described = call_method1(
                &element,
                "getAttribute",
                &JsValue::from_str("aria-describedby"),
            )?;
            let described = legacy_or_empty(&described);
            let error_id = raw_string(&error_id);
            let remaining = described
                .split(' ')
                .filter(|id| *id != error_id)
                .collect::<Vec<_>>()
                .join(" ");
            if !remaining.is_empty() {
                let _ = call_method2(
                    &element,
                    "setAttribute",
                    &JsValue::from_str("aria-describedby"),
                    &JsValue::from_str(&remaining),
                )?;
            } else {
                let _ = call_method1(
                    &element,
                    "removeAttribute",
                    &JsValue::from_str("aria-describedby"),
                )?;
            }
            let dataset_object: Object = dataset.unchecked_into();
            let _ =
                Reflect::delete_property(&dataset_object, &JsValue::from_str("settingsErrorId"))?;
        }
    }

    let global = js_sys::global();
    let document = get_property(&global, "document")?;
    let errors_object: Object = errors.unchecked_into();
    let names = Object::keys(&errors_object);
    for index in 0..names.length() {
        let name = names.get(index).as_string().unwrap_or_default();
        let message = property(errors_object.as_ref(), &name);
        let field_id = format!("{prefix}{name}");
        let field = call_method1(&document, "getElementById", &JsValue::from_str(&field_id))?;
        if !js_boolean(&field) || !js_boolean(&call_method1(&root, "contains", &field)?) {
            continue;
        }

        let error = call_method1(&document, "createElement", &JsValue::from_str("small"))?;
        let error_id = format!("{prefix}{name}-error");
        set_property(&error, "id", &JsValue::from_str(&error_id))?;
        let error_dataset = get_property(&error, "dataset")?;
        set_property(&error_dataset, "settingsError", &JsValue::from_str(&name))?;
        set_property(&error, "className", &JsValue::from_str("kgw-field-error"))?;
        set_property(&error, "textContent", &message)?;
        let _ = call_method2(
            &field,
            "setAttribute",
            &JsValue::from_str("aria-invalid"),
            &JsValue::from_str("true"),
        )?;
        let field_dataset = get_property(&field, "dataset")?;
        set_property(
            &field_dataset,
            "settingsErrorId",
            &JsValue::from_str(&error_id),
        )?;
        let existing_described = call_method1(
            &field,
            "getAttribute",
            &JsValue::from_str("aria-describedby"),
        )?;
        let current = legacy_or_empty(&existing_described);
        let described = if current.is_empty() {
            error_id.clone()
        } else {
            format!("{current} {error_id}")
        };
        let _ = call_method2(
            &field,
            "setAttribute",
            &JsValue::from_str("aria-describedby"),
            &JsValue::from_str(&described),
        )?;
        let owner = call_method1(
            &field,
            "closest",
            &JsValue::from_str(".node-v6-card, .bridge-v7-card"),
        )?;
        if js_boolean(&owner) {
            let _ = call_method1(&owner, "appendChild", &error)?;
        }
    }

    Ok(())
}

async fn await_value(value: JsValue) -> Result<JsValue, JsValue> {
    if value.is_instance_of::<Promise>() {
        JsFuture::from(value.unchecked_into::<Promise>()).await
    } else {
        Ok(value)
    }
}

#[wasm_bindgen(js_name = settingsConfirmUserAction)]
pub async fn confirm_user_action(message: JsValue) -> Result<bool, JsValue> {
    let global = js_sys::global();
    let window = get_property(&global, "window")?;
    let tauri = property(&window, "__TAURI__");
    let core = property(&tauri, "core");
    let invoke = property(&core, "invoke");
    if let Some(function) = invoke.dyn_ref::<Function>() {
        let options = Object::new();
        set_property(
            options.as_ref(),
            "title",
            &JsValue::from_str("KaspaGateway"),
        )?;
        set_property(options.as_ref(), "message", &message)?;
        set_property(options.as_ref(), "kind", &JsValue::from_str("warning"))?;
        set_property(options.as_ref(), "buttons", &JsValue::from_str("OkCancel"))?;
        let answer = function.call2(
            &core,
            &JsValue::from_str("plugin:dialog|message"),
            options.as_ref(),
        )?;
        return Ok(await_value(answer).await?.as_string().as_deref() == Some("Ok"));
    }

    let confirm = get_property(&window, "confirm")?.dyn_into::<Function>()?;
    let answer = confirm.call1(&window, &message)?;
    Ok(await_value(answer).await?.as_bool() == Some(true))
}

fn first_truthy_text_property(target: &JsValue, names: &[&str], fallback: &str) -> String {
    for name in names {
        let value = property(target, name);
        if js_boolean(&value) {
            return raw_string(&value).trim().to_owned();
        }
    }
    fallback.to_owned()
}

fn settings_address_short_text(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= 28 {
        return value.to_owned();
    }
    let left = chars.iter().take(14).collect::<String>();
    let right = chars
        .iter()
        .rev()
        .take(12)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{left}\u{2026}{right}")
}

fn settings_is_kaspa_address_text(value: &str) -> bool {
    let text = value.trim();
    text.starts_with("kaspa:") || text.starts_with("kaspatest:")
}

#[wasm_bindgen(js_name = settingsAddressShort)]
pub fn settings_address_short(value: JsValue) -> String {
    settings_address_short_text(&raw_string(&value))
}

#[wasm_bindgen(js_name = settingsIsKaspaAddress)]
pub fn settings_is_kaspa_address(value: JsValue) -> bool {
    settings_is_kaspa_address_text(&raw_string(&value))
}

#[wasm_bindgen(js_name = settingsAddressNormalize)]
pub fn settings_address_normalize(record: JsValue) -> JsValue {
    let address = first_truthy_text_property(&record, &["address", "Address"], "");
    let raw_name = first_truthy_text_property(&record, &["name", "Name", "label", "Label"], "");
    let network = first_truthy_text_property(&record, &["network", "Network"], "mainnet");
    let generated_prefix = format!("Kaspa {}", settings_address_short_text(&address));
    let name = if raw_name == generated_prefix || raw_name.starts_with("Kaspa kaspa:") {
        String::new()
    } else {
        raw_name
    };

    let output = Object::new();
    let _ = set_property(output.as_ref(), "address", &JsValue::from_str(&address));
    let _ = set_property(output.as_ref(), "name", &JsValue::from_str(&name));
    let _ = set_property(output.as_ref(), "network", &JsValue::from_str(&network));
    output.into()
}

fn western_digits_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch as u32 {
            code @ 0x0660..=0x0669 => char::from_u32(u32::from(b'0') + code - 0x0660).unwrap(),
            code @ 0x06F0..=0x06F9 => char::from_u32(u32::from(b'0') + code - 0x06F0).unwrap(),
            _ => ch,
        })
        .collect()
}

fn display_defaults() -> [(&'static str, bool); 9] {
    [
        ("language:en", true),
        ("currency:USD", true),
        ("tab:kaspa-node", true),
        ("tab:kaspa-bridge", true),
        ("tab:settings", true),
        ("tab:explorer", false),
        ("tab:analysis", false),
        ("tab:top-addresses", false),
        ("tab:log", false),
    ]
}

fn copy_object(value: &JsValue) -> Object {
    let output = Object::new();
    if !value.is_object() {
        return output;
    }
    for key in Object::keys(value.unchecked_ref::<Object>()).iter() {
        let current = Reflect::get(value, &key).unwrap_or(JsValue::UNDEFINED);
        let _ = Reflect::set(output.as_ref(), &key, &current);
    }
    output
}

fn starts_with_display_prefix(key: &str) -> bool {
    key.starts_with("language:") || key.starts_with("currency:") || key.starts_with("tab:")
}

fn known_display_entry(value: &JsValue) -> (String, String) {
    (text_property(value, "key"), text_property(value, "id"))
}

fn selected_display_values(checks: &JsValue, prefix: &str) -> Vec<String> {
    if !checks.is_object() {
        return Vec::new();
    }
    Object::keys(checks.unchecked_ref::<Object>())
        .iter()
        .filter_map(|key| {
            let key = raw_string(&key);
            if !key.starts_with(prefix) {
                return None;
            }
            let value = Reflect::get(checks, &JsValue::from_str(&key)).ok()?;
            strict_true(&value).then(|| key[prefix.len()..].to_owned())
        })
        .collect()
}

fn unique_or(values: Vec<String>, fallback: &[&str]) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        if !output.contains(&value) {
            output.push(value);
        }
    }
    if output.is_empty() {
        output.extend(fallback.iter().map(|value| (*value).to_owned()));
    }
    output
}

fn string_array(values: &[String]) -> Array {
    let output = Array::new();
    for value in values {
        output.push(&JsValue::from_str(value));
    }
    output
}

#[wasm_bindgen(js_name = settingsToWesternDigits)]
pub fn settings_to_western_digits(value: JsValue) -> String {
    western_digits_text(&raw_string(&value))
}

#[wasm_bindgen(js_name = settingsDisplayChecksWithDefaults)]
pub fn settings_display_checks_with_defaults(checks: JsValue, known_entries: Array) -> JsValue {
    let next = copy_object(&checks);

    for key in Object::keys(&next).iter() {
        let key_text = raw_string(&key);
        if starts_with_display_prefix(&key_text) {
            let _ = Reflect::set(next.as_ref(), &key, &JsValue::FALSE);
        }
    }

    let known = known_entries
        .iter()
        .map(|entry| known_display_entry(&entry))
        .collect::<Vec<_>>();
    for (key, id) in &known {
        if !key.is_empty() {
            let _ = set_property(next.as_ref(), key, &JsValue::FALSE);
        }
        if !id.is_empty() {
            let _ = set_property(next.as_ref(), id, &JsValue::FALSE);
        }
    }

    for (key, enabled) in display_defaults() {
        let value = JsValue::from_bool(enabled);
        let _ = set_property(next.as_ref(), key, &value);
        if let Some((_, id)) = known.iter().find(|(known_key, _)| known_key == key)
            && !id.is_empty()
        {
            let _ = set_property(next.as_ref(), id, &value);
        }
    }
    for id in [
        "settingsLangSelectAll",
        "settingsCurrencySelectAll",
        "settingsTabSelectAll",
    ] {
        let _ = set_property(next.as_ref(), id, &JsValue::FALSE);
    }
    next.into()
}

#[wasm_bindgen(js_name = settingsSelectedDisplayKeys)]
pub fn settings_selected_display_keys(checks: JsValue, prefix: String) -> Array {
    string_array(&selected_display_values(&checks, &prefix))
}

#[wasm_bindgen(js_name = settingsDisplayStateMissingContract)]
pub fn settings_display_state_missing_contract(checks: JsValue) -> bool {
    if !checks.is_object() {
        return true;
    }
    ["language:", "currency:", "tab:"]
        .iter()
        .any(|prefix| selected_display_values(&checks, prefix).is_empty())
}

#[wasm_bindgen(js_name = settingsDisplayPreferences)]
pub fn settings_display_preferences(checks: JsValue) -> JsValue {
    if !checks.is_object() {
        return JsValue::NULL;
    }
    let languages = unique_or(selected_display_values(&checks, "language:"), &["en"]);
    let currencies = unique_or(selected_display_values(&checks, "currency:"), &["USD"]);
    let mut tabs = unique_or(
        selected_display_values(&checks, "tab:"),
        &["kaspa-node", "kaspa-bridge", "settings"],
    );
    if !tabs.iter().any(|tab| tab == "settings") {
        tabs.push("settings".to_owned());
    }
    let output = Object::new();
    let _ = set_property(
        output.as_ref(),
        "languages",
        string_array(&languages).as_ref(),
    );
    let _ = set_property(
        output.as_ref(),
        "currencies",
        string_array(&currencies).as_ref(),
    );
    let _ = set_property(output.as_ref(), "tabs", string_array(&tabs).as_ref());
    output.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_helpers_preserve_legacy_contracts() {
        assert_eq!(settings_address_short_text("short"), "short");
        let long = "kaspa:abcdefghijklmnopqrstuvwxyz0123456789";
        let shortened = settings_address_short_text(long);
        assert_eq!(shortened.chars().count(), 27);
        assert!(shortened.contains('\u{2026}'));
        assert!(shortened.starts_with(&long.chars().take(14).collect::<String>()));
        assert!(
            shortened.ends_with(
                &long
                    .chars()
                    .rev()
                    .take(12)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<String>()
            )
        );
        assert!(settings_is_kaspa_address_text(" kaspa:qabc "));
        assert!(settings_is_kaspa_address_text("kaspatest:qabc"));
        assert!(!settings_is_kaspa_address_text("kaspa-dev:qabc"));
        assert!(!settings_is_kaspa_address_text("btc:qabc"));
    }

    #[test]
    fn western_digits_cover_arabic_and_persian_forms() {
        let arabic = (0x0661..=0x0669)
            .chain(std::iter::once(0x0660))
            .filter_map(char::from_u32)
            .collect::<String>();
        let persian = (0x06F1..=0x06F9)
            .chain(std::iter::once(0x06F0))
            .filter_map(char::from_u32)
            .collect::<String>();
        let mixed = format!(
            "12.{}{}",
            char::from_u32(0x0663).unwrap(),
            char::from_u32(0x06F4).unwrap()
        );
        assert_eq!(western_digits_text(&arabic), "1234567890");
        assert_eq!(western_digits_text(&persian), "1234567890");
        assert_eq!(western_digits_text(&mixed), "12.34");
    }

    #[test]
    fn display_defaults_preserve_canonical_shell_contract() {
        assert_eq!(display_defaults()[0], ("language:en", true));
        assert!(display_defaults().contains(&("currency:USD", true)));
        assert!(display_defaults().contains(&("tab:settings", true)));
        assert!(display_defaults().contains(&("tab:explorer", false)));
    }

    #[test]
    fn host_validation_matches_contract_examples() {
        for host in [
            "127.0.0.1",
            "001.002.003.004",
            "::1",
            "[2001:db8::1]",
            "node.example",
            "localhost",
            "node.example.",
        ] {
            assert!(is_host_text(host), "{host}");
        }
        for host in [
            "",
            "300.1.2.3",
            "127.0.0",
            "-bad",
            "bad host",
            "a/b",
            "::::",
            "bad-.example",
        ] {
            assert!(!is_host_text(host), "{host}");
        }
    }

    #[test]
    fn endpoint_split_and_loopback_contracts_hold() {
        assert_eq!(
            split_endpoint_text("[2001:db8::1]:16110"),
            Some(("2001:db8::1".to_owned(), "16110".to_owned()))
        );
        assert_eq!(
            split_endpoint_text(":5555"),
            Some((String::new(), "5555".to_owned()))
        );
        assert!(split_endpoint_text("missing").is_none());
        assert!(is_loopback_text("localhost"));
        assert!(is_loopback_text("127.9.8.7"));
        assert!(is_loopback_text("[::1]"));
        assert!(!is_loopback_text("192.168.1.1"));
    }

    #[test]
    fn listener_overlap_preserves_wildcard_and_loopback_alias_rules() {
        assert!(listeners_overlap_core(
            "0.0.0.0",
            16110.0,
            "127.0.0.1",
            16110.0
        ));
        assert!(listeners_overlap_core(
            "localhost",
            16110.0,
            "127.0.0.1",
            16110.0
        ));
        assert!(!listeners_overlap_core(
            "192.168.1.1",
            16110.0,
            "192.168.1.2",
            16110.0
        ));
    }

    #[test]
    fn paths_and_positive_durations_match_legacy_contract() {
        for path in [r"C:\KGW\bridge.yaml", r"\\server\share\x", "/tmp/x"] {
            assert!(absolute_directory_path(path), "{path}");
        }
        for path in ["relative.yaml", r"\\server", "C:relative"] {
            assert!(!absolute_directory_path(path), "{path}");
        }
        for value in ["1", "50ms", "1s"] {
            assert!(positive_duration(value), "{value}");
        }
        for value in ["", "0", "0ms", "01ms", "-1s", "1.0s", "1MS"] {
            assert!(!positive_duration(value), "{value}");
        }
    }
}
