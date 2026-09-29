use std::collections::HashSet;

use js_sys::{Array, Object, Reflect, RegExp};
use wasm_bindgen::{JsCast, prelude::*};

use crate::bridge_frontend_helpers;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PortRange {
    min: u16,
    max: u16,
    preferred: u16,
    instance_start: Option<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PortProfile {
    stratum: PortRange,
    prom: PortRange,
    dashboard: PortRange,
}

const MAINNET: PortProfile = PortProfile {
    stratum: PortRange {
        min: 5500,
        max: 5599,
        preferred: 5555,
        instance_start: Some(5556),
    },
    prom: PortRange {
        min: 2100,
        max: 2199,
        preferred: 2112,
        instance_start: Some(2113),
    },
    dashboard: PortRange {
        min: 3000,
        max: 3099,
        preferred: 3030,
        instance_start: None,
    },
};

const TESTNET10: PortProfile = PortProfile {
    stratum: PortRange {
        min: 5600,
        max: 5699,
        preferred: 5655,
        instance_start: Some(5656),
    },
    prom: PortRange {
        min: 2200,
        max: 2299,
        preferred: 2212,
        instance_start: Some(2213),
    },
    dashboard: PortRange {
        min: 3100,
        max: 3199,
        preferred: 3130,
        instance_start: None,
    },
};

const TESTNET13: PortProfile = PortProfile {
    stratum: PortRange {
        min: 5700,
        max: 5799,
        preferred: 5755,
        instance_start: Some(5756),
    },
    prom: PortRange {
        min: 2300,
        max: 2399,
        preferred: 2312,
        instance_start: Some(2313),
    },
    dashboard: PortRange {
        min: 3200,
        max: 3299,
        preferred: 3230,
        instance_start: None,
    },
};

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

fn static_profile(net: &str) -> PortProfile {
    match net {
        "testnet10" => TESTNET10,
        "testnet13" => TESTNET13,
        _ => MAINNET,
    }
}

fn normalize_port_text(value: &str) -> String {
    let trimmed = value.trim();
    trimmed.strip_prefix(':').unwrap_or(trimmed).to_owned()
}

fn parse_valid_port(value: &str) -> Option<u16> {
    let normalized = normalize_port_text(value);
    if normalized.is_empty()
        || normalized.len() > 5
        || !normalized.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let port = normalized.parse::<u32>().ok()?;
    (1..=65_535).contains(&port).then_some(port as u16)
}

fn normalize_soft_text(value: &str) -> String {
    parse_valid_port(value)
        .map(|port| port.to_string())
        .unwrap_or_default()
}

fn range_object(range: PortRange) -> JsValue {
    let out = Object::new();
    set(out.as_ref(), "min", &JsValue::from_f64(range.min as f64));
    set(out.as_ref(), "max", &JsValue::from_f64(range.max as f64));
    set(
        out.as_ref(),
        "preferred",
        &JsValue::from_f64(range.preferred as f64),
    );
    if let Some(start) = range.instance_start {
        set(
            out.as_ref(),
            "instanceStart",
            &JsValue::from_f64(start as f64),
        );
    }
    out.into()
}

fn profile_object(profile: PortProfile) -> JsValue {
    let out = Object::new();
    set(out.as_ref(), "stratum", &range_object(profile.stratum));
    set(out.as_ref(), "prom", &range_object(profile.prom));
    set(out.as_ref(), "dashboard", &range_object(profile.dashboard));
    out.into()
}

fn range_number(value: &JsValue, name: &str, fallback: u16) -> u16 {
    property(value, name)
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 1.0 && *n <= 65_535.0 && n.fract() == 0.0)
        .map(|n| n as u16)
        .unwrap_or(fallback)
}

fn range_from_js(value: &JsValue) -> PortRange {
    let min = range_number(value, "min", 1);
    let max = range_number(value, "max", 65_535).max(min);
    let preferred = range_number(value, "preferred", min);
    let instance_start = property(value, "instanceStart")
        .as_f64()
        .filter(|n| n.is_finite() && *n >= 1.0 && *n <= 65_535.0 && n.fract() == 0.0)
        .map(|n| n as u16);
    PortRange {
        min,
        max,
        preferred,
        instance_start,
    }
}

fn copy_range(fallback: &JsValue) -> JsValue {
    range_object(range_from_js(fallback))
}

fn dynamic_range_object(
    fallback: &JsValue,
    min: u16,
    max: u16,
    preferred: u16,
    instance_start: Option<u16>,
    external_base: Option<u16>,
    dynamic: Option<bool>,
) -> JsValue {
    let parsed = range_from_js(fallback);
    let out = Object::new();
    set(out.as_ref(), "min", &JsValue::from_f64(min as f64));
    set(out.as_ref(), "max", &JsValue::from_f64(max as f64));
    set(
        out.as_ref(),
        "preferred",
        &JsValue::from_f64(preferred as f64),
    );
    if let Some(start) = instance_start.or(parsed.instance_start) {
        set(
            out.as_ref(),
            "instanceStart",
            &JsValue::from_f64(start as f64),
        );
    }
    if let Some(base) = external_base {
        set(
            out.as_ref(),
            "externalBase",
            &JsValue::from_str(&base.to_string()),
        );
        set(
            out.as_ref(),
            "externalOwner",
            &JsValue::from_str(if dynamic == Some(true) {
                "bridge-level-port-setting"
            } else {
                "static-profile-matching-current-bridge-setting"
            }),
        );
        set(
            out.as_ref(),
            "dynamicFromExternalSetting",
            &JsValue::from_bool(dynamic.unwrap_or(false)),
        );
    }
    out.into()
}

fn first_capture(regex: &RegExp, source: &str) -> Option<String> {
    let captures = regex.exec(source)?;
    let candidate = if captures.length() > 2 && crate::js_boolean(&captures.get(2)) {
        captures.get(2)
    } else {
        captures.get(1)
    };
    let value = crate::js_string_owned(&candidate);
    (!value.is_empty()).then_some(value)
}

fn extract_ports_text(source: &str) -> Vec<String> {
    let patterns = [
        RegExp::new(
            r"(?:^|[,\s=])(?:port|stratum|stratum_port|prom|prom_port|rpc|rpclisten|listen|dashboard|web_dashboard_port)=?(?:127\.0\.0\.1:|0\.0\.0\.0:|localhost:|:)?(\d{2,5})(?=$|[,\s])",
            "gi",
        ),
        RegExp::new(r"(?:127\.0\.0\.1|0\.0\.0\.0|localhost):(\d{2,5})", "gi"),
        RegExp::new(r"(^|[^\d]):(\d{2,5})(?=$|[,\s])", "g"),
    ];
    let mut ports = Vec::new();
    for regex in patterns {
        while let Some(raw) = first_capture(&regex, source) {
            if let Some(port) = parse_valid_port(&raw) {
                ports.push(port.to_string());
            }
        }
    }
    ports
}

fn normalize_port_literal_text(value: &str) -> String {
    let source = value.trim();
    if source.is_empty() {
        return String::new();
    }
    let host = RegExp::new(
        r"(?:^|[^0-9])(?:127\.0\.0\.1|0\.0\.0\.0|localhost)?:(\d{1,5})(?:$|[^0-9])",
        "i",
    );
    if let Some(raw) = first_capture(&host, source)
        && let Some(port) = parse_valid_port(&raw)
    {
        return port.to_string();
    }
    if let Some(port) = parse_valid_port(source) {
        return port.to_string();
    }
    extract_ports_text(source)
        .into_iter()
        .next()
        .unwrap_or_default()
}

fn logical_key(net: &str, role: &str, owner: &str) -> String {
    let suffix = match role {
        "default-kaspad-rpc" | "bridge-external-kaspad" | "inprocess-rpc" => "node-rpc",
        "default-stratum" | "bridge-stratum" => "bridge-stratum",
        "default-prometheus" | "bridge-prometheus" => "bridge-prometheus",
        _ => return format!("{net}:{role}:{owner}"),
    };
    format!("{net}:{suffix}")
}

fn set_has(set_value: &JsValue, value: &str) -> bool {
    let Ok(function) = property(set_value, "has").dyn_into::<js_sys::Function>() else {
        return false;
    };
    function
        .call1(set_value, &JsValue::from_str(value))
        .ok()
        .is_some_and(|result| crate::js_boolean(&result))
}

fn set_add(set_value: &JsValue, value: &str) {
    if let Ok(function) = property(set_value, "add").dyn_into::<js_sys::Function>() {
        let _ = function.call1(set_value, &JsValue::from_str(value));
    }
}

fn find_nearest_unused_impl(start_port: &str, used: &JsValue) -> Result<String, JsValue> {
    let mut port = parse_valid_port(start_port).unwrap_or(1) as u32;
    while port <= 65_535 && set_has(used, &port.to_string()) {
        port += 1;
    }
    if port > 65_535 {
        return Err(JsValue::from_str(
            "No available TCP port was found for bridge instance allocation.",
        ));
    }
    let value = port.to_string();
    set_add(used, &value);
    Ok(value)
}

fn port_in_range_impl(port: &str, range: &JsValue) -> bool {
    let Some(port) = parse_valid_port(port) else {
        return false;
    };
    let parsed = range_from_js(range);
    port >= parsed.min && port <= parsed.max
}

fn find_unused_in_range_impl(range: &JsValue, used: &JsValue, fallback_start: &str) -> String {
    let parsed = range_from_js(range);
    let start = parse_valid_port(fallback_start)
        .or(parsed.instance_start)
        .unwrap_or(parsed.preferred);
    let begin = start.clamp(parsed.min, parsed.max);

    for port in begin..=parsed.max {
        let value = port.to_string();
        if !set_has(used, &value) {
            set_add(used, &value);
            return value;
        }
    }
    if begin > parsed.min {
        for port in parsed.min..begin {
            let value = port.to_string();
            if !set_has(used, &value) {
                set_add(used, &value);
                return value;
            }
        }
    }
    String::new()
}

fn external_base_port(net: &str, kind: &str, fallback_range: &JsValue) -> String {
    let field_name = match kind {
        "prom" => "promPort",
        "dashboard" => "webDashboardPort",
        _ => "stratumPort",
    };
    let profile_field = match kind {
        "prom" => "promPort",
        "dashboard" => "dashboardPort",
        _ => "stratumPort",
    };
    let current = bridge_frontend_helpers::bridge_value(net.to_owned(), field_name.to_owned());
    let network_profile = bridge_frontend_helpers::bridge_network_profile(net.to_owned());
    let profile_value = crate::js_string_owned(&property(&network_profile, profile_field));

    for candidate in [
        current,
        profile_value,
        crate::js_string_owned(&property(fallback_range, "preferred")),
        crate::js_string_owned(&property(fallback_range, "min")),
    ] {
        let normalized = normalize_port_literal_text(&candidate);
        if !normalized.is_empty() {
            return normalized;
        }
    }
    String::new()
}

fn range_from_external_base(base_port: &str, fallback_range: &JsValue) -> JsValue {
    let fallback = range_from_js(fallback_range);
    let Some(base) = parse_valid_port(&normalize_port_literal_text(base_port)) else {
        return copy_range(fallback_range);
    };

    if base == fallback.preferred {
        return dynamic_range_object(
            fallback_range,
            fallback.min,
            fallback.max,
            fallback.preferred,
            fallback.instance_start,
            Some(base),
            Some(false),
        );
    }

    let start = if base < 65_535 { base + 1 } else { base };
    let max = (start as u32 + 98).min(65_535) as u16;
    dynamic_range_object(
        fallback_range,
        start,
        max,
        base,
        Some(start),
        Some(base),
        Some(true),
    )
}

fn profile_for_current_settings(net: &str) -> JsValue {
    let static_value = profile_object(static_profile(net));
    let output = Object::new();
    for kind in ["stratum", "prom", "dashboard"] {
        let fallback = property(&static_value, kind);
        let base = external_base_port(net, kind, &fallback);
        set(
            output.as_ref(),
            kind,
            &range_from_external_base(&base, &fallback),
        );
    }
    output.into()
}

fn owner_key(owner: &JsValue) -> String {
    [
        crate::js_string_owned(&property(owner, "net")),
        crate::js_string_owned(&property(owner, "role")),
        crate::js_string_owned(&property(owner, "owner")),
    ]
    .join("|")
}

fn unique_owners_impl(owners: &JsValue) -> Vec<JsValue> {
    let input = if Array::is_array(owners) {
        Array::from(owners)
    } else {
        Array::new()
    };
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    for owner in input.iter() {
        let key = owner_key(&owner);
        if key.is_empty() || !seen.insert(key) {
            continue;
        }
        output.push(owner);
    }
    output
}

#[wasm_bindgen(js_name = bridgePortProfilesR35B)]
pub fn bridge_port_profiles_r35b() -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "mainnet", &profile_object(MAINNET));
    set(output.as_ref(), "testnet10", &profile_object(TESTNET10));
    set(output.as_ref(), "testnet13", &profile_object(TESTNET13));
    output.into()
}

#[wasm_bindgen(js_name = bridgeStaticPortProfileR91)]
pub fn bridge_static_port_profile_r91(net: String) -> JsValue {
    profile_object(static_profile(&net))
}

#[wasm_bindgen(js_name = bridgeExtractPortsFromTextR5)]
pub fn bridge_extract_ports_from_text_r5(value: JsValue) -> Array {
    let output = Array::new();
    for port in extract_ports_text(&crate::js_string_owned(&value)) {
        output.push(&JsValue::from_str(&port));
    }
    output
}

#[wasm_bindgen(js_name = bridgePushPortR5)]
pub fn bridge_push_port_r5(
    items: Array,
    port: JsValue,
    role: JsValue,
    owner: JsValue,
    net: JsValue,
) {
    let Some(port) = parse_valid_port(&crate::js_string_owned(&port)) else {
        return;
    };
    let row = Object::new();
    set(row.as_ref(), "port", &JsValue::from_str(&port.to_string()));
    let role = crate::js_string_owned(&role);
    set(
        row.as_ref(),
        "role",
        &JsValue::from_str(if role.is_empty() { "unknown" } else { &role }),
    );
    set(
        row.as_ref(),
        "owner",
        &JsValue::from_str(&crate::js_string_owned(&owner)),
    );
    set(
        row.as_ref(),
        "net",
        &JsValue::from_str(&crate::js_string_owned(&net)),
    );
    items.push(row.as_ref());
}

#[wasm_bindgen(js_name = bridgePortConflictLogicalKeyR64F)]
pub fn bridge_port_conflict_logical_key_r64f(item: JsValue) -> String {
    logical_key(
        &crate::js_string_owned(&property(&item, "net")),
        &crate::js_string_owned(&property(&item, "role")),
        &crate::js_string_owned(&property(&item, "owner")),
    )
}

#[wasm_bindgen(js_name = bridgePortOwnersRepresentSameLogicalEndpointR64F)]
pub fn bridge_port_owners_represent_same_logical_endpoint_r64f(owners: JsValue) -> bool {
    let input = if Array::is_array(&owners) {
        Array::from(&owners)
    } else {
        Array::new()
    };
    let mut keys = HashSet::new();
    for owner in input.iter() {
        keys.insert(logical_key(
            &crate::js_string_owned(&property(&owner, "net")),
            &crate::js_string_owned(&property(&owner, "role")),
            &crate::js_string_owned(&property(&owner, "owner")),
        ));
    }
    keys.len() <= 1
}

#[wasm_bindgen(js_name = bridgeNormalizePortR9)]
pub fn bridge_normalize_port_r9(value: JsValue) -> String {
    normalize_port_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgePortIsValidR9)]
pub fn bridge_port_is_valid_r9(value: JsValue) -> bool {
    parse_valid_port(&crate::js_string_owned(&value)).is_some()
}

#[wasm_bindgen(js_name = bridgeFindNearestUnusedPortR9)]
pub fn bridge_find_nearest_unused_port_r9(
    start_port: JsValue,
    used_ports: JsValue,
) -> Result<String, JsValue> {
    find_nearest_unused_impl(&crate::js_string_owned(&start_port), &used_ports)
}

#[wasm_bindgen(js_name = bridgeNormalizePortLiteralR91)]
pub fn bridge_normalize_port_literal_r91(value: JsValue) -> String {
    normalize_port_literal_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeExternalBasePortR91)]
pub fn bridge_external_base_port_r91(net: String, kind: String, fallback_range: JsValue) -> String {
    external_base_port(&net, &kind, &fallback_range)
}

#[wasm_bindgen(js_name = bridgeRangeFromExternalBaseR91)]
pub fn bridge_range_from_external_base_r91(base_port: JsValue, fallback_range: JsValue) -> JsValue {
    range_from_external_base(&crate::js_string_owned(&base_port), &fallback_range)
}

#[wasm_bindgen(js_name = bridgePortProfileR35B)]
pub fn bridge_port_profile_r35b(net: String) -> JsValue {
    profile_for_current_settings(&net)
}

#[wasm_bindgen(js_name = bridgeNormalizePortSoftR35B)]
pub fn bridge_normalize_port_soft_r35b(value: JsValue) -> String {
    normalize_soft_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgePortInRangeR35B)]
pub fn bridge_port_in_range_r35b(port: JsValue, range: JsValue) -> bool {
    port_in_range_impl(&crate::js_string_owned(&port), &range)
}

#[wasm_bindgen(js_name = bridgeFindUnusedPortInRangeR35B)]
pub fn bridge_find_unused_port_in_range_r35b(
    range: JsValue,
    used_ports: JsValue,
    fallback_start: JsValue,
) -> String {
    find_unused_in_range_impl(
        &range,
        &used_ports,
        &crate::js_string_owned(&fallback_start),
    )
}

#[wasm_bindgen(js_name = bridgeFindRecommendedOrNearestUnusedPortR35B)]
pub fn bridge_find_recommended_or_nearest_unused_port_r35b(
    net: String,
    kind: String,
    used_ports: JsValue,
    fallback_start: JsValue,
) -> Result<String, JsValue> {
    let profile = profile_for_current_settings(&net);
    let mut range = property(&profile, &kind);
    if !present(&range) {
        range = property(&profile, "stratum");
    }
    let direct = crate::js_string_owned(&fallback_start);
    let fallback = if !direct.is_empty() {
        direct
    } else {
        ["instanceStart", "preferred", "min"]
            .iter()
            .map(|key| crate::js_string_owned(&property(&range, key)))
            .find(|value| !value.is_empty())
            .unwrap_or_else(|| "1".to_owned())
    };
    let inside = find_unused_in_range_impl(&range, &used_ports, &fallback);
    if !inside.is_empty() {
        return Ok(inside);
    }
    find_nearest_unused_impl(&fallback, &used_ports)
}

#[wasm_bindgen(js_name = bridgePortIsInsideAnyKnownRangeR91)]
pub fn bridge_port_is_inside_any_known_range_r91(net: String, kind: String, port: JsValue) -> bool {
    let normalized = normalize_soft_text(&crate::js_string_owned(&port));
    if normalized.is_empty() {
        return false;
    }
    let current = profile_for_current_settings(&net);
    let static_value = profile_object(static_profile(&net));
    for profile in [current, static_value] {
        let range = property(&profile, &kind);
        if present(&range) && port_in_range_impl(&normalized, &range) {
            return true;
        }
    }
    false
}

#[wasm_bindgen(js_name = bridgeInstancePortShouldFollowExternalRangeR91)]
pub fn bridge_instance_port_should_follow_external_range_r91(
    net: String,
    kind: String,
    value: JsValue,
) -> bool {
    let normalized = normalize_port_text(&crate::js_string_owned(&value));
    if normalized.is_empty() || parse_valid_port(&normalized).is_none() {
        return true;
    }
    bridge_port_is_inside_any_known_range_r91(net, kind, JsValue::from_str(&normalized))
}

#[wasm_bindgen(js_name = bridgeAddUsedPortR91)]
pub fn bridge_add_used_port_r91(used: JsValue, value: JsValue) {
    if let Some(port) = parse_valid_port(&crate::js_string_owned(&value)) {
        set_add(&used, &port.to_string());
    }
}

#[wasm_bindgen(js_name = bridgeInstanceIdFromOwnerR37)]
pub fn bridge_instance_id_from_owner_r37(owner: JsValue) -> String {
    crate::js_string_owned(&property(&owner, "owner"))
        .strip_prefix("instance:")
        .unwrap_or("")
        .to_owned()
}

#[wasm_bindgen(js_name = bridgeNormalizePortR37)]
pub fn bridge_normalize_port_r37(value: JsValue) -> String {
    normalize_port_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = bridgeInstancePortKindForConflictR37)]
pub fn bridge_instance_port_kind_for_conflict_r37(
    instance: JsValue,
    conflict_port: JsValue,
) -> String {
    let old_port = normalize_port_text(&crate::js_string_owned(&conflict_port));
    if normalize_port_text(&crate::js_string_owned(&property(
        &instance,
        "instancePort",
    ))) == old_port
    {
        "stratum".to_owned()
    } else if normalize_port_text(&crate::js_string_owned(&property(
        &instance,
        "instanceProm",
    ))) == old_port
    {
        "prom".to_owned()
    } else {
        String::new()
    }
}

#[wasm_bindgen(js_name = bridgeAutofixChangeKeyR37)]
pub fn bridge_autofix_change_key_r37(change: JsValue) -> String {
    [
        crate::js_string_owned(&property(&change, "net")),
        crate::js_string_owned(&property(&change, "instanceId")),
        crate::js_string_owned(&property(&change, "kind")),
        crate::js_string_owned(&property(&change, "oldPort")),
    ]
    .join(":")
}

#[wasm_bindgen(js_name = bridgeOwnerKeyR45)]
pub fn bridge_owner_key_r45(owner: JsValue) -> String {
    owner_key(&owner)
}

#[wasm_bindgen(js_name = bridgeUniqueConflictOwnersR45)]
pub fn bridge_unique_conflict_owners_r45(owners: JsValue) -> Array {
    let output = Array::new();
    for owner in unique_owners_impl(&owners) {
        output.push(&owner);
    }
    output
}

#[wasm_bindgen(js_name = bridgeOwnersToAutofixR45)]
pub fn bridge_owners_to_autofix_r45(active_net: String, owners: JsValue) -> Array {
    let unique = unique_owners_impl(&owners);
    let instances: Vec<JsValue> = unique
        .iter()
        .filter(|owner| crate::js_string_owned(&property(owner, "role")) == "instance")
        .cloned()
        .collect();
    let protected_count = unique
        .iter()
        .filter(|owner| crate::js_string_owned(&property(owner, "role")) != "instance")
        .count();

    let output = Array::new();
    if instances.is_empty() {
        return output;
    }
    if protected_count > 0 {
        for owner in instances {
            output.push(&owner);
        }
        return output;
    }

    let active: Vec<JsValue> = instances
        .iter()
        .filter(|owner| crate::js_string_owned(&property(owner, "net")) == active_net)
        .cloned()
        .collect();
    if !active.is_empty() {
        for owner in active {
            output.push(&owner);
        }
        return output;
    }

    for owner in instances.into_iter().skip(1) {
        output.push(&owner);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_profiles_match_bridge_contract() {
        assert_eq!(MAINNET.stratum.preferred, 5555);
        assert_eq!(MAINNET.prom.instance_start, Some(2113));
        assert_eq!(TESTNET10.stratum.preferred, 5655);
        assert_eq!(TESTNET13.prom.preferred, 2312);
    }

    #[test]
    fn plain_port_normalization_and_validation_match_legacy() {
        assert_eq!(normalize_port_text(" :5556 "), "5556");
        assert_eq!(normalize_soft_text(":5556"), "5556");
        assert_eq!(normalize_soft_text("0"), "");
        assert_eq!(normalize_soft_text("65536"), "");
        assert!(parse_valid_port("1").is_some());
        assert!(parse_valid_port("65535").is_some());
        assert!(parse_valid_port("12x").is_none());
    }

    #[test]
    fn logical_endpoint_keys_preserve_alias_semantics() {
        assert_eq!(
            logical_key("mainnet", "default-kaspad-rpc", "profile"),
            "mainnet:node-rpc"
        );
        assert_eq!(
            logical_key("mainnet", "bridge-external-kaspad", "field"),
            "mainnet:node-rpc"
        );
        assert_eq!(
            logical_key("testnet10", "bridge-prometheus", "field"),
            "testnet10:bridge-prometheus"
        );
        assert_eq!(
            logical_key("mainnet", "instance", "instance:7"),
            "mainnet:instance:instance:7"
        );
    }

    #[test]
    fn static_range_math_matches_soft_profile_policy() {
        assert_eq!(MAINNET.stratum.min, 5500);
        assert_eq!(MAINNET.stratum.max, 5599);
        assert_eq!(MAINNET.stratum.instance_start, Some(5556));
        let base = 6000_u16;
        let start = base + 1;
        let max = (start as u32 + 98).min(65_535) as u16;
        assert_eq!(start, 6001);
        assert_eq!(max, 6099);
    }

    #[test]
    fn parse_valid_port_rejects_out_of_range_values() {
        assert_eq!(parse_valid_port(":5555"), Some(5555));
        assert_eq!(parse_valid_port(" 2112 "), Some(2112));
        assert_eq!(parse_valid_port("0"), None);
        assert_eq!(parse_valid_port("65536"), None);
        assert_eq!(parse_valid_port("localhost:5555"), None);
    }
}
