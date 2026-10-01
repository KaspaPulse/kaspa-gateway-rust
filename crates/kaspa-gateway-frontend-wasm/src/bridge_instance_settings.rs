use js_sys::{Array, Date, Math, Object, Reflect};
use wasm_bindgen::prelude::*;

use crate::{
    bridge_command_options, bridge_frontend_helpers, bridge_instance_ui, bridge_port_orchestration,
    settings_contract,
};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

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

fn global() -> JsValue {
    js_sys::global().into()
}

fn document() -> JsValue {
    property(&global(), "document")
}

fn call1(target: &JsValue, name: &str, first: &JsValue) -> Option<JsValue> {
    let method = property(target, name).dyn_into::<js_sys::Function>().ok()?;
    method.call1(target, first).ok()
}

fn required_function(target: &JsValue, name: &str) -> Result<js_sys::Function, JsValue> {
    property(target, name)
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str(&format!("Bridge R51 callback is unavailable: {name}")))
}

fn call1_required(target: &JsValue, name: &str, first: &JsValue) -> Result<JsValue, JsValue> {
    required_function(target, name)?.call1(target, first)
}

fn call2_required(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    required_function(target, name)?.call2(target, first, second)
}

fn call3_required(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
    third: &JsValue,
) -> Result<JsValue, JsValue> {
    required_function(target, name)?.call3(target, first, second, third)
}

fn event(name: &str) -> Result<JsValue, JsValue> {
    let ctor = property(&global(), "Event")
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str("Bridge R51 Event constructor is unavailable"))?;
    let options = Object::new();
    set(options.as_ref(), "bubbles", &JsValue::TRUE);
    let args = Array::new();
    args.push(&JsValue::from_str(name));
    args.push(options.as_ref());
    Reflect::construct(&ctor, &args)
}

fn dispatch_event(target: &JsValue, name: &str) -> Result<(), JsValue> {
    let event = event(name)?;
    property(target, "dispatchEvent")
        .dyn_into::<js_sys::Function>()
        .map_err(|_| JsValue::from_str("Bridge R51 field dispatchEvent is unavailable"))?
        .call1(target, &event)?;
    Ok(())
}

fn query_selector(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_selector_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let Some(list) = call1(target, "querySelectorAll", &JsValue::from_str(selector)) else {
        return Vec::new();
    };

    let length = property(&list, "length").as_f64().unwrap_or(0.0).max(0.0) as u32;
    (0..length)
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(present)
        })
        .collect()
}

fn field_value(net: &str, name: &str) -> String {
    bridge_frontend_helpers::bridge_value(net.to_owned(), name.to_owned())
}

fn bridge_panel(net: &str) -> JsValue {
    query_selector(
        &document(),
        &format!("[data-bridge-network-panel=\"{net}\"]"),
    )
}

fn bridge_r51_fields_vec(net: &str) -> Vec<JsValue> {
    let panel = bridge_panel(net);
    if !present(&panel) {
        return Vec::new();
    }

    let prefix = format!("bridge-{net}-");
    query_selector_all(&panel, "input, select, textarea")
        .into_iter()
        .filter(|field| {
            let id = property(field, "id").as_string().unwrap_or_default();
            if id.is_empty()
                || !id.starts_with(&prefix)
                || id.ends_with("-commandPreview")
                || id.ends_with("-logOutput")
            {
                return false;
            }
            let toolbar = call1(
                field,
                "closest",
                &JsValue::from_str(".bridge-v7-log-toolbar"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            !present(&toolbar)
        })
        .collect()
}

fn bridge_r51_managed_field(name: &str) -> bool {
    crate::settings_schema::BRIDGE_MANAGED
        .iter()
        .any(|(key, _)| *key == name)
}

fn object_key_count(value: &JsValue) -> u32 {
    if !value.is_object() || value.is_null() {
        return 0;
    }
    Object::keys(&Object::from(value.clone())).length()
}

fn bridge_form(net: &str) -> JsValue {
    let values = Object::new();
    set(values.as_ref(), "network", &JsValue::from_str(net));
    let panel = bridge_panel(net);
    if !present(&panel) {
        return values.into();
    }
    let prefix = format!("bridge-{net}-");
    for field in query_selector_all(
        &panel,
        ".bridge-v7-card input[id], .bridge-v7-card select[id]",
    ) {
        let id = crate::js_string_owned(&property(&field, "id"));
        let Some(name) = id.strip_prefix(&prefix) else {
            continue;
        };
        let value = if crate::js_string_owned(&property(&field, "type")) == "checkbox" {
            JsValue::from_bool(crate::js_boolean(&property(&field, "checked")))
        } else {
            JsValue::from_str(&crate::js_string_owned(&property(&field, "value")))
        };
        set(values.as_ref(), name, &value);
    }
    values.into()
}

fn effective_value(net: &str, name: &str) -> String {
    let values = bridge_form(net);
    let options = bridge_command_options::bridge_command_inline_state_r7(net.to_owned());
    if settings_contract::bridge_field_enabled(name.to_owned(), values, options) {
        field_value(net, name)
    } else {
        String::new()
    }
}

fn node_mode(net: &str) -> &'static str {
    if field_value(net, "nodeMode") == "inprocess" {
        "inprocess"
    } else {
        "external"
    }
}

fn has_config(net: &str) -> bool {
    bridge_command_options::bridge_command_option_enabled_r7(net.to_owned(), "config".to_owned())
        && !field_value(net, "config").is_empty()
}

fn canonical_key_text(key: &str) -> String {
    match key.trim() {
        "stratum" | "stratum_port" => "port".to_owned(),
        "prom_port" => "prom".to_owned(),
        "min_share_diff" => "diff".to_owned(),
        "log_to_file" => "log".to_owned(),
        value => value.to_owned(),
    }
}

fn instance_key_of(part: &str) -> String {
    part.find('=')
        .filter(|index| *index > 0)
        .map(|index| part[..index].trim().to_owned())
        .unwrap_or_default()
}

fn without_keys(parts: Vec<String>, keys: &[&str]) -> Vec<String> {
    let blocked = keys
        .iter()
        .map(|key| canonical_key_text(key))
        .collect::<Vec<_>>();
    parts
        .into_iter()
        .filter(|part| {
            let key = instance_key_of(part);
            key.is_empty() || !blocked.contains(&canonical_key_text(&key))
        })
        .collect()
}
fn append_part(parts: Vec<String>, key: &str, value: &str) -> Vec<String> {
    let clean = value.trim();
    if clean.is_empty() {
        return parts;
    }
    let canonical = canonical_key_text(key);
    let mut filtered = without_keys(parts, &[&canonical]);
    filtered.push(format!("{canonical}={clean}"));
    filtered
}

fn parse_structured_pairs(value: &str) -> Vec<(String, String)> {
    let mut output = Vec::new();
    for part in value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        let Some(eq) = part.find('=') else {
            continue;
        };
        if eq == 0 {
            continue;
        }

        let key = canonical_key_text(part[..eq].trim());
        let raw = part[eq + 1..].trim();
        let field = match key.as_str() {
            "port" => "instancePort",
            "diff" => "instanceDiff",
            "prom" => "instanceProm",
            "log" => "instanceLogToFile",
            "wait" | "block_wait_time" => "instanceBlockWaitTime",
            "extranonce" | "extranonce_size" => "instanceExtranonceSize",
            "var_diff" => "instanceVarDiff",
            "shares_per_min" => "instanceSharesPerMin",
            "var_diff_stats" => "instanceVarDiffStats",
            "pow2_clamp" => "instancePow2Clamp",
            _ => continue,
        };
        let value = if matches!(key.as_str(), "port" | "prom") {
            raw.trim_start_matches(':')
        } else {
            raw
        };
        output.push((field.to_owned(), value.to_owned()));
    }
    output
}
fn parsed_object(value: &str) -> JsValue {
    let output = Object::new();
    for (key, value) in parse_structured_pairs(value) {
        set(output.as_ref(), &key, &JsValue::from_str(&value));
    }
    output.into()
}

fn port_value_text(value: &str) -> String {
    let clean = value.trim().trim_start_matches(':');
    if clean.is_empty() {
        String::new()
    } else {
        format!(":{clean}")
    }
}

fn plain_value_text(value: &str) -> String {
    value.trim().to_owned()
}

fn optional_text(value: &str) -> Option<String> {
    let clean = value.trim();
    (!clean.is_empty()).then(|| clean.to_owned())
}

fn port_listen_text(value: &str, fallback: &str) -> Option<String> {
    let clean = if value.trim().is_empty() {
        fallback.trim()
    } else {
        value.trim()
    };
    if clean.is_empty() {
        return None;
    }
    if clean.bytes().all(|byte| byte.is_ascii_digit()) {
        Some(format!(":{clean}"))
    } else {
        Some(clean.to_owned())
    }
}

fn parse_unsigned_text(
    label: &str,
    value: &str,
    fallback: Option<u64>,
    max: u64,
) -> Result<Option<u64>, String> {
    let clean = value.trim();
    if clean.is_empty() {
        return Ok(fallback);
    }
    if !clean.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{label} must be an unsigned integer"));
    }
    let parsed = clean
        .parse::<u64>()
        .map_err(|_| format!("{label} is outside the supported range"))?;
    if parsed > MAX_SAFE_INTEGER || parsed > max {
        return Err(format!("{label} is outside the supported range"));
    }
    Ok(Some(parsed))
}

fn parse_duration_text(
    label: &str,
    value: &str,
    fallback: Option<u64>,
) -> Result<Option<u64>, String> {
    let clean = value.trim().to_ascii_lowercase();
    if clean.is_empty() {
        return Ok(fallback);
    }
    let (digits, multiplier) = if let Some(raw) = clean.strip_suffix("ms") {
        (raw, 1_u64)
    } else if let Some(raw) = clean.strip_suffix('s') {
        (raw, 1000_u64)
    } else {
        (clean.as_str(), 1_u64)
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "{label} must use milliseconds or an integer with ms/s suffix"
        ));
    }

    let amount = digits
        .parse::<u64>()
        .map_err(|_| format!("{label} must use milliseconds or an integer with ms/s suffix"))?;
    let milliseconds = amount
        .checked_mul(multiplier)
        .filter(|value| *value <= MAX_SAFE_INTEGER)
        .ok_or_else(|| format!("{label} must be greater than zero"))?;
    if milliseconds == 0 {
        return Err(format!("{label} must be greater than zero"));
    }
    Ok(Some(milliseconds))
}

fn bool_value_text(value: &str, fallback: Option<bool>) -> Result<Option<bool>, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "not set" => Ok(fallback),
        "true" => Ok(Some(true)),
        "false" => Ok(Some(false)),
        _ => Err("Bridge boolean setting must be true or false".to_owned()),
    }
}

fn js_option_number(value: Option<u64>) -> JsValue {
    value
        .map(|number| JsValue::from_f64(number as f64))
        .unwrap_or(JsValue::NULL)
}

fn js_option_bool(value: Option<bool>) -> JsValue {
    value.map(JsValue::from_bool).unwrap_or(JsValue::NULL)
}

fn js_option_string(value: Option<String>) -> JsValue {
    value
        .map(|text| JsValue::from_str(&text))
        .unwrap_or(JsValue::NULL)
}

fn copy_object_fields(source: &JsValue, output: &Object) {
    if !source.is_object() || source.is_null() {
        return;
    }

    for entry in Object::entries(&Object::from(source.clone())).iter() {
        let pair = Array::from(&entry);
        if pair.length() >= 2 {
            let key = crate::js_string_owned(&pair.get(0));
            set(output.as_ref(), &key, &pair.get(1));
        }
    }
}

fn first_truthy_text(values: &[JsValue]) -> String {
    for value in values {
        if crate::js_boolean(value) {
            return crate::js_string_owned(value);
        }
    }
    String::new()
}

fn default_instance_record_impl(id_value: JsValue) -> JsValue {
    let output = Object::new();
    let id = if crate::js_boolean(&id_value) {
        id_value
    } else {
        JsValue::from_f64(Date::now() + (Math::random() * 1000.0).floor())
    };
    set(output.as_ref(), "id", &id);
    for (key, value) in [
        ("instance", ""),
        ("instanceDiff", "2048"),
        ("instanceLogToFile", "not set"),
        ("instanceBlockWaitTime", ""),
        ("instanceExtranonceSize", ""),
        ("instanceVarDiff", "not set"),
        ("instanceSharesPerMin", ""),
        ("instanceVarDiffStats", "not set"),
        ("instancePow2Clamp", "not set"),
    ] {
        set(output.as_ref(), key, &JsValue::from_str(value));
    }
    set(output.as_ref(), "instancePort", &JsValue::NULL);
    set(output.as_ref(), "instanceProm", &JsValue::NULL);
    output.into()
}

fn normalize_instance_record_impl(raw: JsValue, fallback_id: JsValue) -> JsValue {
    let source = if raw.is_object() && !raw.is_null() {
        raw
    } else {
        Object::new().into()
    };
    let source_id = property(&source, "id");
    let chosen_id = if crate::js_boolean(&source_id) {
        source_id.clone()
    } else {
        fallback_id
    };
    let defaults = default_instance_record_impl(chosen_id);
    let output = Object::new();
    copy_object_fields(&defaults, &output);
    copy_object_fields(&source, &output);
    let parsed = parsed_object(&crate::js_string_owned(&property(&source, "instance")));

    let id = {
        let source_id = property(&source, "id");

        if crate::js_boolean(&source_id) {
            source_id
        } else {
            property(&defaults, "id")
        }
    };
    set(output.as_ref(), "id", &id);
    set(output.as_ref(), "instance", &JsValue::from_str(""));

    for (field, fallback) in [
        ("instancePort", ""),
        ("instanceDiff", "2048"),
        ("instanceProm", ""),
        ("instanceLogToFile", "not set"),
        ("instanceBlockWaitTime", ""),
        ("instanceExtranonceSize", ""),
        ("instanceVarDiff", "not set"),
        ("instanceSharesPerMin", ""),
        ("instanceVarDiffStats", "not set"),
        ("instancePow2Clamp", "not set"),
    ] {
        let value = first_truthy_text(&[
            property(&source, field),
            property(&parsed, field),
            JsValue::from_str(fallback),
        ]);
        set(output.as_ref(), field, &JsValue::from_str(&value));
    }
    output.into()
}

fn read_instance_supplement(
    net: &str,
    instance_id: &JsValue,
    field_name: &str,
    fallback: &JsValue,
) -> String {
    let instance_id = crate::js_string_owned(instance_id);
    let id = bridge_frontend_helpers::bridge_element_id(
        net.to_owned(),
        format!("{field_name}-{instance_id}"),
    );
    let element = bridge_frontend_helpers::bridge_by_id(id);
    let current = if present(&element) {
        if crate::js_string_owned(&property(&element, "type")) == "checkbox" {
            if crate::js_boolean(&property(&element, "checked")) {
                "true".to_owned()
            } else {
                String::new()
            }
        } else {
            crate::js_string_owned(&property(&element, "value"))
                .trim()
                .to_owned()
        }
    } else {
        String::new()
    };
    if current.is_empty() {
        crate::js_string_owned(fallback).trim().to_owned()
    } else {
        current
    }
}
fn build_upstream_instance_arg_impl(net: &str, instance: &JsValue) -> String {
    let instance_id = property(instance, "id");
    let mut parts = Vec::new();
    for (field, key, port_style) in [
        ("instancePort", "port", true),
        ("instanceDiff", "diff", false),
        ("instanceProm", "prom", true),
        ("instanceLogToFile", "log", false),
        ("instanceBlockWaitTime", "wait", false),
        ("instanceExtranonceSize", "extranonce", false),
        ("instanceVarDiff", "var_diff", false),
        ("instanceSharesPerMin", "shares_per_min", false),
        ("instanceVarDiffStats", "var_diff_stats", false),
        ("instancePow2Clamp", "pow2_clamp", false),
    ] {
        if !bridge_command_options::bridge_instance_command_should_include_r13b(
            net.to_owned(),
            instance_id.clone(),
            field.to_owned(),
            instance.clone(),
        ) {
            continue;
        }
        let raw = read_instance_supplement(net, &instance_id, field, &property(instance, field));
        let value = if port_style {
            port_value_text(&raw)
        } else {
            plain_value_text(&raw)
        };
        parts = append_part(parts, key, &value);
    }
    parts.join(",")
}

fn profile_string(profile: &JsValue, name: &str, fallback: &str) -> String {
    let value = crate::js_string_owned(&property(profile, name));
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value
    }
}

fn effective_node_integer_text(name: &str, value: &str, fallback: f64) -> Result<f64, String> {
    let clean = value.trim();
    if clean.is_empty() {
        return Ok(fallback);
    }
    let number = clean
        .parse::<f64>()
        .map_err(|_| format!("{name} must be an integer."))?;
    if !number.is_finite() || number.fract() != 0.0 {
        return Err(format!("{name} must be an integer."));
    }
    Ok(number)
}

fn effective_node_number_text(name: &str, value: &str, fallback: f64) -> Result<f64, String> {
    let clean = value.trim();
    if clean.is_empty() {
        return Ok(fallback);
    }
    let number = clean
        .parse::<f64>()
        .map_err(|_| format!("{name} must be a finite number."))?;
    if !number.is_finite() {
        return Err(format!("{name} must be a finite number."));
    }
    Ok(number)
}

fn effective_node_integer(net: &str, name: &str, fallback: f64) -> Result<f64, JsValue> {
    effective_node_integer_text(name, &effective_value(net, name), fallback)
        .map_err(|error| JsValue::from_str(&error))
}

fn effective_node_number(net: &str, name: &str, fallback: f64) -> Result<f64, JsValue> {
    if !bridge_command_options::bridge_command_should_include_r7(net.to_owned(), name.to_owned()) {
        return Ok(fallback);
    }
    effective_node_number_text(name, &field_value(net, name), fallback)
        .map_err(|error| JsValue::from_str(&error))
}

fn command_optional_text(net: &str, name: &str) -> JsValue {
    if !bridge_command_options::bridge_command_should_include_r7(net.to_owned(), name.to_owned()) {
        return JsValue::NULL;
    }
    js_option_string(optional_text(&field_value(net, name)))
}

fn one_peer_array(value: String) -> Array {
    let output = Array::new();
    if !value.is_empty() {
        output.push(&JsValue::from_str(&value));
    }
    output
}

fn effective_inprocess_node_settings_impl(net: &str) -> Result<JsValue, JsValue> {
    if !field_value(net, "inprocessConfigfile").is_empty() {
        return Err(JsValue::from_str(
            "In-process --configfile is unsupported because the desktop owns network and database isolation.",
        ));
    }
    if bridge_frontend_helpers::bridge_checked(net.to_owned(), "inprocessDevnet".to_owned())
        || bridge_frontend_helpers::bridge_checked(net.to_owned(), "inprocessSimnet".to_owned())
    {
        return Err(JsValue::from_str(
            "Devnet and simnet cannot override the selected desktop network tab.",
        ));
    }
    if !field_value(net, "inprocessOverrideParamsFile").is_empty() {
        return Err(JsValue::from_str(
            "In-process --override-params-file is unsupported because the desktop owns the selected network identity.",
        ));
    }

    let profile = bridge_frontend_helpers::bridge_network_profile(net.to_owned());
    let kaspad_port = profile_string(&profile, "kaspadPort", "16110");
    let rpc_listen = {
        let configured = field_value(net, "inprocessRpcListen");
        if configured.is_empty() {
            format!("127.0.0.1:{kaspad_port}")
        } else {
            configured
        }
    };
    let add_peer = if bridge_command_options::bridge_command_should_include_r7(
        net.to_owned(),
        "inprocessAddPeer".to_owned(),
    ) {
        field_value(net, "inprocessAddPeer")
    } else {
        String::new()
    };
    let connect_peer = if bridge_command_options::bridge_command_should_include_r7(
        net.to_owned(),
        "inprocessConnect".to_owned(),
    ) {
        field_value(net, "inprocessConnect")
    } else {
        String::new()
    };
    let log_level = if bridge_command_options::bridge_command_should_include_r7(
        net.to_owned(),
        "inprocessLogLevel".to_owned(),
    ) {
        let value = field_value(net, "inprocessLogLevel");
        if value.is_empty() {
            "info".to_owned()
        } else {
            value
        }
    } else {
        "info".to_owned()
    };
    let testnet = crate::js_boolean(&property(&profile, "testnet"));
    let checked =
        |name: &str| bridge_frontend_helpers::bridge_checked(net.to_owned(), name.to_owned());

    let output = Object::new();
    set(output.as_ref(), "logLevel", &JsValue::from_str(&log_level));
    set(
        output.as_ref(),
        "asyncThreads",
        &JsValue::from_f64(effective_node_integer(net, "inprocessAsyncThreads", 16.0)?),
    );
    set(
        output.as_ref(),
        "ramScale",
        &JsValue::from_f64(effective_node_number(net, "inprocessRamScale", 1.0)?),
    );
    set(
        output.as_ref(),
        "yes",
        &JsValue::from_bool(checked("inprocessYes")),
    );
    set(output.as_ref(), "noLogFiles", &JsValue::TRUE);
    set(output.as_ref(), "sanity", &JsValue::FALSE);
    set(
        output.as_ref(),
        "enableUnsyncedMining",
        &JsValue::from_bool(checked("inprocessEnableUnsyncedMining") && testnet),
    );
    set(
        output.as_ref(),
        "p2pListen",
        &command_optional_text(net, "inprocessListen"),
    );
    set(output.as_ref(), "externalIp", &JsValue::NULL);
    set(
        output.as_ref(),
        "disableUpnp",
        &JsValue::from_bool(checked("inprocessDisableUpnp")),
    );
    set(output.as_ref(), "disableDnsSeeding", &JsValue::FALSE);
    set(output.as_ref(), "userAgentComments", Array::new().as_ref());
    set(
        output.as_ref(),
        "rpcListen",
        &JsValue::from_str(&rpc_listen),
    );
    set(
        output.as_ref(),
        "rpcListenBorsh",
        &command_optional_text(net, "inprocessRpcListenBorsh"),
    );
    set(
        output.as_ref(),
        "rpcListenJson",
        &command_optional_text(net, "inprocessRpcListenJson"),
    );
    set(output.as_ref(), "rpcMaxClients", &JsValue::from_f64(16.0));
    set(
        output.as_ref(),
        "unsafeRpc",
        &JsValue::from_bool(checked("inprocessUnsafeRpc")),
    );
    set(output.as_ref(), "disableGrpc", &JsValue::FALSE);
    set(
        output.as_ref(),
        "connectPeers",
        one_peer_array(connect_peer).as_ref(),
    );
    set(
        output.as_ref(),
        "addPeers",
        one_peer_array(add_peer).as_ref(),
    );
    set(
        output.as_ref(),
        "outboundTarget",
        &JsValue::from_f64(effective_node_integer(net, "inprocessOutpeers", 8.0)?),
    );
    set(
        output.as_ref(),
        "inboundLimit",
        &JsValue::from_f64(effective_node_integer(net, "inprocessMaxInpeers", 32.0)?),
    );
    set(
        output.as_ref(),
        "utxoIndex",
        &JsValue::from_bool(checked("inprocessUtxoIndex")),
    );
    set(
        output.as_ref(),
        "archival",
        &JsValue::from_bool(checked("inprocessArchival")),
    );
    set(output.as_ref(), "resetDb", &JsValue::FALSE);
    set(
        output.as_ref(),
        "perfMetrics",
        &JsValue::from_bool(checked("inprocessPerfMetrics")),
    );
    set(
        output.as_ref(),
        "maxTrackedAddresses",
        &JsValue::from_f64(0.0),
    );
    set(output.as_ref(), "retentionPeriodDays", &JsValue::NULL);
    set(
        output.as_ref(),
        "perfMetricsIntervalSec",
        &JsValue::from_f64(effective_node_integer(
            net,
            "inprocessPerfMetricsIntervalSec",
            10.0,
        )?),
    );
    set(output.as_ref(), "rocksDbPreset", &JsValue::NULL);
    set(output.as_ref(), "rocksDbCacheSize", &JsValue::NULL);
    set(output.as_ref(), "rocksDbWalDir", &JsValue::NULL);
    set(output.as_ref(), "overrideParamsFile", &JsValue::NULL);
    set(output.as_ref(), "logDir", &JsValue::NULL);
    Ok(output.into())
}

fn effective_settings_impl(net: &str, structured_instances: &JsValue) -> Result<JsValue, JsValue> {
    if has_config(net) {
        return Ok(JsValue::NULL);
    }
    let profile = bridge_frontend_helpers::bridge_network_profile(net.to_owned());
    let kaspad_port = profile_string(&profile, "kaspadPort", "16110");
    let stratum_port = profile_string(&profile, "stratumPort", "");

    if net != "mainnet" {
        let global = Object::new();
        let endpoint = if node_mode(net) == "inprocess" {
            field_value(net, "inprocessRpcListen")
        } else {
            field_value(net, "kaspadAddress")
        };
        set(
            global.as_ref(),
            "kaspaRpcEndpoint",
            &js_option_string(port_listen_text(
                &endpoint,
                &format!("127.0.0.1:{kaspad_port}"),
            )),
        );
        set(global.as_ref(), "logToFile", &JsValue::FALSE);
        set(global.as_ref(), "healthCheckListen", &JsValue::NULL);
        set(global.as_ref(), "webDashboardListen", &JsValue::NULL);
        set(global.as_ref(), "approximateGeoLookup", &JsValue::FALSE);

        let output = Object::new();
        set(output.as_ref(), "version", &JsValue::from_f64(1.0));
        set(output.as_ref(), "global", global.as_ref());
        set(output.as_ref(), "instances", Array::new().as_ref());
        return Ok(output.into());
    }

    let records_value = property(structured_instances, "instances");
    let records = if Array::is_array(&records_value) {
        Array::from(&records_value)
    } else {
        Array::new()
    };
    let instances = Array::new();
    for (index, raw) in records.iter().enumerate() {
        let instance = normalize_instance_record_impl(raw, JsValue::from_f64((index + 1) as f64));
        let instance_id = property(&instance, "id");

        if !bridge_command_options::bridge_instance_command_should_include_r13b(
            net.to_owned(),
            instance_id.clone(),
            "instance".to_owned(),
            instance.clone(),
        ) {
            continue;
        }
        let included = |field: &str| {
            bridge_command_options::bridge_instance_command_should_include_r13b(
                net.to_owned(),
                instance_id.clone(),
                field.to_owned(),
                instance.clone(),
            )
        };
        let global_min = parse_unsigned_text(
            "Minimum share difficulty",
            &effective_value(net, "minShareDiff"),
            Some(8192),
            4_294_967_295,
        )
        .map_err(|error| JsValue::from_str(&error))?;

        let row = Object::new();
        let id_text = first_truthy_text(&[
            property(&instance, "id"),
            JsValue::from_f64((index + 1) as f64),
        ]);
        set(row.as_ref(), "instanceId", &JsValue::from_str(&id_text));

        let stratum_source = if included("instancePort") {
            crate::js_string_owned(&property(&instance, "instancePort"))
        } else {
            String::new()
        };
        let stratum_fallback = if index == 0 {
            let current = effective_value(net, "stratumPort");
            if current.is_empty() {
                stratum_port.clone()
            } else {
                current
            }
        } else {
            String::new()
        };

        set(
            row.as_ref(),
            "stratumListen",
            &js_option_string(port_listen_text(&stratum_source, &stratum_fallback)),
        );

        let diff_source = if included("instanceDiff") {
            crate::js_string_owned(&property(&instance, "instanceDiff"))
        } else {
            String::new()
        };
        set(
            row.as_ref(),
            "minShareDiff",
            &js_option_number(
                parse_unsigned_text(
                    "Instance minimum share difficulty",
                    &diff_source,
                    global_min,
                    4_294_967_295,
                )
                .map_err(|error| JsValue::from_str(&error))?,
            ),
        );

        let prom_source = if included("instanceProm") {
            crate::js_string_owned(&property(&instance, "instanceProm"))
        } else {
            String::new()
        };
        let prom_fallback = if index == 0 {
            effective_value(net, "promPort")
        } else {
            String::new()
        };
        set(
            row.as_ref(),
            "prometheusListen",
            &js_option_string(port_listen_text(&prom_source, &prom_fallback)),
        );

        let log_value = if included("instanceLogToFile") {
            bool_value_text(
                &crate::js_string_owned(&property(&instance, "instanceLogToFile")),
                None,
            )
            .map_err(|error| JsValue::from_str(&error))?
        } else {
            None
        };
        set(row.as_ref(), "logToFile", &js_option_bool(log_value));

        let wait_text = crate::js_string_owned(&property(&instance, "instanceBlockWaitTime"));
        let wait = if included("instanceBlockWaitTime") && !wait_text.is_empty() {
            parse_duration_text("Instance block wait time", &wait_text, None)
                .map_err(|error| JsValue::from_str(&error))?
        } else {
            None
        };
        set(row.as_ref(), "blockWaitTimeMs", &js_option_number(wait));
        let extranonce_text =
            crate::js_string_owned(&property(&instance, "instanceExtranonceSize"));
        let extranonce = if included("instanceExtranonceSize") && !extranonce_text.is_empty() {
            parse_unsigned_text("Instance extranonce size", &extranonce_text, None, 8)
                .map_err(|error| JsValue::from_str(&error))?
        } else {
            None
        };
        set(
            row.as_ref(),
            "extranonceSize",
            &js_option_number(extranonce),
        );

        for (field, output_name) in [
            ("instanceVarDiff", "varDiff"),
            ("instanceVarDiffStats", "varDiffStats"),
            ("instancePow2Clamp", "pow2Clamp"),
        ] {
            let value = if included(field) {
                bool_value_text(&crate::js_string_owned(&property(&instance, field)), None)
                    .map_err(|error| JsValue::from_str(&error))?
            } else {
                None
            };
            set(row.as_ref(), output_name, &js_option_bool(value));
        }

        let shares_text = crate::js_string_owned(&property(&instance, "instanceSharesPerMin"));
        let shares = if included("instanceSharesPerMin") && !shares_text.is_empty() {
            parse_unsigned_text(
                "Instance shares per minute",
                &shares_text,
                None,
                4_294_967_295,
            )
            .map_err(|error| JsValue::from_str(&error))?
        } else {
            None
        };
        set(row.as_ref(), "sharesPerMin", &js_option_number(shares));
        instances.push(row.as_ref());
    }
    if instances.length() == 0 {
        return Err(JsValue::from_str(
            "At least one Bridge instance is required",
        ));
    }

    let global = Object::new();
    let endpoint = if node_mode(net) == "inprocess" {
        field_value(net, "inprocessRpcListen")
    } else {
        effective_value(net, "kaspadAddress")
    };
    set(
        global.as_ref(),
        "kaspaRpcEndpoint",
        &js_option_string(port_listen_text(
            &endpoint,
            &format!("127.0.0.1:{kaspad_port}"),
        )),
    );

    set(
        global.as_ref(),
        "blockWaitTimeMs",
        &js_option_number(
            parse_duration_text(
                "Block wait time",
                &effective_value(net, "blockWaitTime"),
                Some(1000),
            )
            .map_err(|error| JsValue::from_str(&error))?,
        ),
    );
    set(
        global.as_ref(),
        "printStats",
        &js_option_bool(
            bool_value_text(&effective_value(net, "printStats"), Some(true))
                .map_err(|error| JsValue::from_str(&error))?,
        ),
    );

    set(
        global.as_ref(),
        "logToFile",
        &js_option_bool(
            bool_value_text(&effective_value(net, "logToFile"), Some(false))
                .map_err(|error| JsValue::from_str(&error))?,
        ),
    );
    set(
        global.as_ref(),
        "healthCheckListen",
        &js_option_string(port_listen_text(
            &effective_value(net, "healthCheckPort"),
            "",
        )),
    );
    set(
        global.as_ref(),
        "webDashboardListen",
        &js_option_string(port_listen_text(
            &effective_value(net, "webDashboardPort"),
            "",
        )),
    );

    set(
        global.as_ref(),
        "varDiff",
        &js_option_bool(
            bool_value_text(&effective_value(net, "varDiff"), Some(true))
                .map_err(|error| JsValue::from_str(&error))?,
        ),
    );
    set(
        global.as_ref(),
        "sharesPerMin",
        &js_option_number(
            parse_unsigned_text(
                "Shares per minute",
                &effective_value(net, "sharesPerMin"),
                Some(20),
                4_294_967_295,
            )
            .map_err(|error| JsValue::from_str(&error))?,
        ),
    );

    set(
        global.as_ref(),
        "varDiffStats",
        &js_option_bool(
            bool_value_text(&effective_value(net, "varDiffStats"), Some(false))
                .map_err(|error| JsValue::from_str(&error))?,
        ),
    );
    set(
        global.as_ref(),
        "extranonceSize",
        &js_option_number(
            parse_unsigned_text(
                "Extranonce size",
                &effective_value(net, "extranonceSize"),
                Some(0),
                8,
            )
            .map_err(|error| JsValue::from_str(&error))?,
        ),
    );
    set(
        global.as_ref(),
        "pow2Clamp",
        &js_option_bool(
            bool_value_text(&effective_value(net, "pow2Clamp"), Some(false))
                .map_err(|error| JsValue::from_str(&error))?,
        ),
    );
    set(
        global.as_ref(),
        "coinbaseTagSuffix",
        &js_option_string(optional_text(&effective_value(net, "coinbaseTagSuffix"))),
    );
    set(global.as_ref(), "approximateGeoLookup", &JsValue::FALSE);

    let output = Object::new();
    set(output.as_ref(), "version", &JsValue::from_f64(1.0));
    set(output.as_ref(), "global", global.as_ref());
    set(output.as_ref(), "instances", instances.as_ref());
    Ok(output.into())
}

#[wasm_bindgen(js_name = bridgeR51Panel)]
pub fn bridge_r51_panel(net: String) -> JsValue {
    bridge_panel(&net)
}

#[wasm_bindgen(js_name = bridgeR51Fields)]
pub fn bridge_r51_fields(net: String) -> Array {
    let output = Array::new();
    for field in bridge_r51_fields_vec(&net) {
        output.push(&field);
    }
    output
}

#[wasm_bindgen(js_name = bridgeR51ReadSettings)]
pub fn bridge_r51_read_settings(net: String, callbacks: JsValue) -> Result<JsValue, JsValue> {
    const STRUCTURED_INSTANCES_KEY: &str = "__kgwBridgeStructuredInstancesR26B";
    const ACTIVE_INSTANCE_KEY: &str = "__kgwBridgeActiveInstanceR26B";
    const COMMAND_OPTIONS_KEY: &str = "__kgwBridgeCommandOptionsR38C";
    const INSTANCE_COMMAND_OPTIONS_KEY: &str = "__kgwBridgeInstanceCommandOptionsR38C";

    let net_value = JsValue::from_str(&net);
    let values = Object::new();

    let structured = call1_required(&callbacks, "readStructuredInstances", &net_value)?;
    set(values.as_ref(), STRUCTURED_INSTANCES_KEY, &structured);
    set(
        values.as_ref(),
        ACTIVE_INSTANCE_KEY,
        &property(&structured, "activeInstance"),
    );

    let command_options = call1_required(&callbacks, "readCommandOptions", &net_value)?;
    set(values.as_ref(), COMMAND_OPTIONS_KEY, &command_options);

    let instance_command_options =
        call1_required(&callbacks, "readInstanceCommandOptions", &net_value)?;
    set(
        values.as_ref(),
        INSTANCE_COMMAND_OPTIONS_KEY,
        &instance_command_options,
    );

    let prefix = format!("bridge-{net}-");
    for field in bridge_r51_fields_vec(&net) {
        let id = crate::js_string_owned(&property(&field, "id"));
        if id.is_empty() {
            continue;
        }
        let name = id.strip_prefix(&prefix).unwrap_or(&id);
        if bridge_r51_managed_field(name) {
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

    let trace = Object::new();
    set(trace.as_ref(), "patch", &JsValue::from_str("R38C"));
    set(
        trace.as_ref(),
        "owner",
        &JsValue::from_str("bridge-r51-settings-owner"),
    );
    set(
        trace.as_ref(),
        "commandOptionCount",
        &JsValue::from_f64(f64::from(object_key_count(&command_options))),
    );
    set(
        trace.as_ref(),
        "instanceCommandOptionInstanceCount",
        &JsValue::from_f64(f64::from(object_key_count(&instance_command_options))),
    );
    bridge_frontend_helpers::bridge_small_owner_trace_r44d(
        net_value.clone(),
        JsValue::from_str("settings-persistence"),
        JsValue::from_str("r38c-read-settings-command-options"),
        trace.into(),
    );

    call3_required(
        &callbacks,
        "normalizeNetworkPortValues",
        &net_value,
        values.as_ref(),
        &JsValue::from_str("read-settings"),
    )
}

#[wasm_bindgen(js_name = bridgeR51WriteSettings)]
pub fn bridge_r51_write_settings(
    net: String,
    values: JsValue,
    bridge_instances: JsValue,
    active_instance: JsValue,
    callbacks: JsValue,
) -> Result<(), JsValue> {
    if !values.is_object() || values.is_null() {
        return Ok(());
    }

    let net_value = JsValue::from_str(&net);
    let values = call3_required(
        &callbacks,
        "normalizeNetworkPortValues",
        &net_value,
        &values,
        &JsValue::from_str("write-settings"),
    )?;

    let _ = call2_required(&callbacks, "applyStructuredInstances", &net_value, &values)?;

    let prefix = format!("bridge-{net}-");
    for field in bridge_r51_fields_vec(&net) {
        let id = crate::js_string_owned(&property(&field, "id"));
        if id.is_empty() {
            continue;
        }
        let name = id.strip_prefix(&prefix).unwrap_or(&id);
        if bridge_r51_managed_field(name) {
            continue;
        }

        let item = property(&values, &id);
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
            let item_value = property(&item, "value");
            let text = if present(&item_value) {
                crate::js_string_owned(&item_value)
            } else {
                String::new()
            };
            set(&field, "value", &JsValue::from_str(&text));
        }

        dispatch_event(&field, "input")?;
        dispatch_event(&field, "change")?;
    }

    let _ = call2_required(&callbacks, "applyCommandOptions", &net_value, &values)?;

    let network_key = bridge_frontend_helpers::bridge_instance_network_key_r15(
        net_value.clone(),
        net_value.clone(),
    );
    bridge_port_orchestration::bridge_reassign_instance_ports_from_external_range_r91(
        bridge_instances.clone(),
        network_key,
        "r95b-r51-write-settings-normalized-network-ports".to_owned(),
    )?;
    bridge_instance_ui::bridge_sync_instance_preview_rows_r8b(
        net.clone(),
        bridge_instances,
        active_instance,
    )?;

    let _ = call1_required(&callbacks, "updateCommand", &net_value)?;
    Ok(())
}

#[wasm_bindgen(js_name = bridgeInstanceParseStructured)]
pub fn bridge_instance_parse_structured(value: JsValue) -> JsValue {
    parsed_object(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeInstancePortValue)]
pub fn bridge_instance_port_value(value: JsValue) -> String {
    port_value_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeInstancePlainValue)]
pub fn bridge_instance_plain_value(value: JsValue) -> String {
    plain_value_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeOptionalTextV1)]
pub fn bridge_optional_text_v1(value: JsValue) -> JsValue {
    js_option_string(optional_text(&crate::js_string_owned(&value)))
}

#[wasm_bindgen(js_name = bridgePortListenV1)]
pub fn bridge_port_listen_v1(value: JsValue, fallback: JsValue) -> JsValue {
    js_option_string(port_listen_text(
        &crate::js_string_owned(&value),
        &crate::js_string_owned(&fallback),
    ))
}

#[wasm_bindgen(js_name = bridgeParseUnsignedV1)]
pub fn bridge_parse_unsigned_v1(
    label: String,
    value: JsValue,
    fallback: JsValue,
    max: f64,
) -> Result<JsValue, JsValue> {
    let fallback_number = fallback.as_f64().map(|number| number as u64);
    let max = if max.is_finite() && max >= 0.0 {
        max.min(MAX_SAFE_INTEGER as f64) as u64
    } else {
        MAX_SAFE_INTEGER
    };

    parse_unsigned_text(
        &label,
        &crate::js_string_owned(&value),
        fallback_number,
        max,
    )
    .map(js_option_number)
    .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = bridgeParseDurationMsV1)]
pub fn bridge_parse_duration_ms_v1(
    label: String,
    value: JsValue,
    fallback: JsValue,
) -> Result<JsValue, JsValue> {
    let fallback_number = fallback.as_f64().map(|number| number as u64);
    parse_duration_text(&label, &crate::js_string_owned(&value), fallback_number)
        .map(js_option_number)
        .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = bridgeBoolValueV1)]
pub fn bridge_bool_value_v1(value: JsValue, fallback: JsValue) -> Result<JsValue, JsValue> {
    let fallback_bool = fallback.as_bool();
    bool_value_text(&crate::js_string_owned(&value), fallback_bool)
        .map(js_option_bool)
        .map_err(|error| JsValue::from_str(&error))
}

#[wasm_bindgen(js_name = bridgeDefaultInstanceRecord)]
pub fn bridge_default_instance_record(id_value: JsValue) -> JsValue {
    default_instance_record_impl(id_value)
}

#[wasm_bindgen(js_name = bridgeNormalizeInstanceRecord)]
pub fn bridge_normalize_instance_record(raw: JsValue, fallback_id: JsValue) -> JsValue {
    normalize_instance_record_impl(raw, fallback_id)
}

#[wasm_bindgen(js_name = bridgeBuildUpstreamInstanceArg)]
pub fn bridge_build_upstream_instance_arg(net: String, instance: JsValue) -> String {
    build_upstream_instance_arg_impl(&net, &instance)
}

#[wasm_bindgen(js_name = bridgeEffectiveSettingsV1)]
pub fn bridge_effective_settings_v1(
    net: String,
    structured_instances: JsValue,
) -> Result<JsValue, JsValue> {
    effective_settings_impl(&net, &structured_instances)
}

#[wasm_bindgen(js_name = bridgeEffectiveInprocessNodeSettings)]
pub fn bridge_effective_inprocess_node_settings(net: String) -> Result<JsValue, JsValue> {
    effective_inprocess_node_settings_impl(&net)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_keys_and_append_match_legacy() {
        assert_eq!(canonical_key_text("stratum_port"), "port");
        assert_eq!(canonical_key_text("prom_port"), "prom");
        assert_eq!(canonical_key_text("min_share_diff"), "diff");
        let parts = append_part(vec!["port=:1".to_owned()], "stratum", ":2");
        assert_eq!(parts, vec!["port=:2"]);
    }

    #[test]
    fn structured_parser_matches_instance_contract() {
        let parsed = parse_structured_pairs(
            "port=:5556,prom=:2113,wait=2500ms,extranonce=4,log=false,var_diff=true,shares_per_min=30,var_diff_stats=true,pow2_clamp=true",
        );
        assert!(parsed.contains(&("instancePort".to_owned(), "5556".to_owned())));
        assert!(parsed.contains(&("instanceProm".to_owned(), "2113".to_owned())));
        assert!(parsed.contains(&("instanceBlockWaitTime".to_owned(), "2500ms".to_owned())));
        assert!(parsed.contains(&("instancePow2Clamp".to_owned(), "true".to_owned())));
    }

    #[test]
    fn scalar_parsers_preserve_legacy_rules() {
        assert_eq!(port_value_text(" :5556 "), ":5556");
        assert_eq!(port_listen_text("5556", ""), Some(":5556".to_owned()));
        assert_eq!(
            port_listen_text("127.0.0.1:16110", ""),
            Some("127.0.0.1:16110".to_owned())
        );
        assert_eq!(parse_unsigned_text("x", "8", None, 8).unwrap(), Some(8));
        assert!(parse_unsigned_text("x", "-1", None, 8).is_err());

        assert_eq!(parse_duration_text("wait", "2s", None).unwrap(), Some(2000));
        assert!(parse_duration_text("wait", "0", None).is_err());
        assert_eq!(bool_value_text("not set", None).unwrap(), None);
        assert_eq!(bool_value_text("true", None).unwrap(), Some(true));
        assert!(bool_value_text("yes", None).is_err());
    }

    #[test]
    fn effective_node_numeric_parsers_preserve_legacy_rules() {
        assert_eq!(
            effective_node_integer_text("inprocessAsyncThreads", "", 16.0).unwrap(),
            16.0
        );
        assert_eq!(
            effective_node_integer_text("inprocessAsyncThreads", "1e2", 16.0).unwrap(),
            100.0
        );
        assert!(effective_node_integer_text("inprocessAsyncThreads", "1.5", 16.0).is_err());
        assert_eq!(
            effective_node_number_text("inprocessRamScale", "1.25", 1.0).unwrap(),
            1.25
        );
        assert!(effective_node_number_text("inprocessRamScale", "NaN", 1.0).is_err());
    }
}
