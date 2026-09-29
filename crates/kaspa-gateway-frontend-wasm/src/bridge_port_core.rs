use std::collections::HashSet;

use js_sys::{Array, Function, Object, Reflect, RegExp, Set as JsSet};
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct PortOwnerRecord {
    port: String,
    net: String,
    role: String,
    owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PortConflictRecord {
    port: String,
    owners: Vec<PortOwnerRecord>,
}

fn owner_record_from_js(value: &JsValue, fallback_port: &str) -> PortOwnerRecord {
    let port = {
        let direct = crate::js_string_owned(&property(value, "port"));
        if direct.is_empty() {
            fallback_port.to_owned()
        } else {
            direct
        }
    };
    PortOwnerRecord {
        port,
        net: crate::js_string_owned(&property(value, "net")),
        role: crate::js_string_owned(&property(value, "role")),
        owner: crate::js_string_owned(&property(value, "owner")),
    }
}

fn owner_record_key(owner: &PortOwnerRecord) -> String {
    format!("{}:{}:{}", owner.net, owner.role, owner.owner)
}

fn records_share_logical_endpoint(owners: &[PortOwnerRecord]) -> bool {
    let keys = owners
        .iter()
        .map(|owner| logical_key(&owner.net, &owner.role, &owner.owner))
        .collect::<HashSet<_>>();
    keys.len() <= 1
}

fn validate_port_conflicts_records(
    records: &[PortOwnerRecord],
    active_net: &str,
) -> Vec<PortConflictRecord> {
    let mut groups: Vec<(String, Vec<PortOwnerRecord>)> = Vec::new();
    for record in records {
        if record.port.is_empty() {
            continue;
        }
        if let Some((_, owners)) = groups.iter_mut().find(|(port, _)| port == &record.port) {
            owners.push(record.clone());
        } else {
            groups.push((record.port.clone(), vec![record.clone()]));
        }
    }

    let mut conflicts = Vec::new();
    for (port, owners) in groups {
        let unique_owner_count = owners
            .iter()
            .map(owner_record_key)
            .collect::<HashSet<_>>()
            .len();
        if unique_owner_count <= 1 || records_share_logical_endpoint(&owners) {
            continue;
        }

        let touches_active_net = owners.iter().any(|owner| owner.net == active_net);
        let touches_instance = owners.iter().any(|owner| owner.role == "instance");
        if touches_active_net || touches_instance {
            conflicts.push(PortConflictRecord { port, owners });
        }
    }
    conflicts
}

fn owner_record_to_js(owner: &PortOwnerRecord) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "net", &JsValue::from_str(&owner.net));
    set(output.as_ref(), "role", &JsValue::from_str(&owner.role));
    set(output.as_ref(), "owner", &JsValue::from_str(&owner.owner));
    output.into()
}

fn conflict_record_to_js(conflict: &PortConflictRecord) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "port", &JsValue::from_str(&conflict.port));
    let owners = Array::new();
    for owner in &conflict.owners {
        owners.push(&owner_record_to_js(owner));
    }
    set(output.as_ref(), "owners", owners.as_ref());
    output.into()
}

fn conflict_message_records(conflicts: &[PortConflictRecord]) -> String {
    conflicts
        .iter()
        .map(|conflict| {
            let owners = conflict
                .owners
                .iter()
                .map(|owner| format!("{}/{}/{}", owner.net, owner.role, owner.owner))
                .collect::<Vec<_>>()
                .join(" | ");
            format!("port {} => {}", conflict.port, owners)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn conflicts_from_validation(validation: &JsValue) -> Vec<PortConflictRecord> {
    let conflicts = property(validation, "conflicts");
    if !Array::is_array(&conflicts) {
        return Vec::new();
    }

    Array::from(&conflicts)
        .iter()
        .map(|conflict| {
            let port = crate::js_string_owned(&property(&conflict, "port"));
            let owners_value = property(&conflict, "owners");
            let owners = if Array::is_array(&owners_value) {
                Array::from(&owners_value)
                    .iter()
                    .map(|owner| owner_record_from_js(&owner, &port))
                    .collect()
            } else {
                Vec::new()
            };
            PortConflictRecord { port, owners }
        })
        .collect()
}

fn port_conflict_message(validation: &JsValue) -> String {
    if !present(validation) || crate::js_boolean(&property(validation, "ok")) {
        return String::new();
    }
    let message = crate::js_string_owned(&property(validation, "message"))
        .trim()
        .to_owned();
    if !message.is_empty() {
        return message;
    }
    conflict_message_records(&conflicts_from_validation(validation))
}

fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, arg).ok()
}

fn node_list_values(value: &JsValue) -> Vec<JsValue> {
    let length = property(value, "length")
        .as_f64()
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or(0.0) as u32;
    (0..length)
        .filter_map(|index| {
            Reflect::get(value, &JsValue::from_f64(index as f64))
                .ok()
                .filter(present)
        })
        .collect()
}

fn start_buttons_for_net(net: &str) -> Vec<JsValue> {
    let global = js_sys::global();
    let document = property(&global, "document");
    let root = call1(
        &document,
        "getElementById",
        &JsValue::from_str("kaspa-bridge"),
    )
    .unwrap_or(JsValue::UNDEFINED);
    if !present(&root) {
        return Vec::new();
    }
    let selector = format!("[data-bridge-action=\"start\"][data-net=\"{}\"]", net);
    let list = call1(&root, "querySelectorAll", &JsValue::from_str(&selector))
        .unwrap_or(JsValue::UNDEFINED);
    node_list_values(&list)
}

fn trace_port_conflict(net: &str, phase: &str, validation: &JsValue, details: &JsValue) -> bool {
    let conflicts = conflicts_from_validation(validation);
    let payload = Object::new();
    set(payload.as_ref(), "patch", &JsValue::from_str("R33"));
    set(
        payload.as_ref(),
        "owner",
        &JsValue::from_str("existing-bridge-port-conflict-owner-r5-r33"),
    );
    set(
        payload.as_ref(),
        "ok",
        &JsValue::from_bool(crate::js_boolean(&property(validation, "ok"))),
    );
    set(
        payload.as_ref(),
        "conflictCount",
        &JsValue::from_f64(conflicts.len() as f64),
    );
    let message: String = port_conflict_message(validation)
        .chars()
        .take(1200)
        .collect();
    set(payload.as_ref(), "message", &JsValue::from_str(&message));

    let compact = Array::new();
    for conflict in conflicts.iter().take(20) {
        compact.push(&conflict_record_to_js(conflict));
    }
    set(payload.as_ref(), "conflicts", compact.as_ref());
    let safe_details = if details.is_object() && !details.is_null() {
        details.clone()
    } else {
        Object::new().into()
    };
    set(payload.as_ref(), "details", &safe_details);

    bridge_frontend_helpers::bridge_small_owner_trace_r44d(
        JsValue::from_str(net),
        JsValue::from_str("port-conflict"),
        JsValue::from_str(phase),
        payload.into(),
    )
}

fn apply_port_conflict_start_state(net: &str, validation: &JsValue, reason: &str) -> JsValue {
    let buttons = start_buttons_for_net(net);
    let blocked = present(validation) && !crate::js_boolean(&property(validation, "ok"));
    let message = port_conflict_message(validation);

    for button in &buttons {
        let dataset = property(button, "dataset");
        let class_list = property(button, "classList");
        if blocked {
            set(button, "disabled", &JsValue::TRUE);
            let _ = call1(
                &class_list,
                "add",
                &JsValue::from_str("kgw-port-conflict-blocked-r33"),
            );
            set(
                &dataset,
                "kgwPortConflictBlockedR33",
                &JsValue::from_str("true"),
            );
            let short_message: String = message.chars().take(800).collect();
            set(
                &dataset,
                "kgwPortConflictMessageR33",
                &JsValue::from_str(&short_message),
            );
            let title_message: String = message.chars().take(700).collect();
            set(
                button,
                "title",
                &JsValue::from_str(&format!("Port conflict: {title_message}")),
            );
        } else if crate::js_string_owned(&property(&dataset, "kgwPortConflictBlockedR33")) == "true"
        {
            set(button, "disabled", &JsValue::FALSE);
            let _ = call1(
                &class_list,
                "remove",
                &JsValue::from_str("kgw-port-conflict-blocked-r33"),
            );
            let dataset_object = Object::from(dataset.clone());
            let _ = Reflect::delete_property(
                &dataset_object,
                &JsValue::from_str("kgwPortConflictBlockedR33"),
            );
            let _ = Reflect::delete_property(
                &dataset_object,
                &JsValue::from_str("kgwPortConflictMessageR33"),
            );
            if crate::js_string_owned(&property(button, "title")).starts_with("Port conflict:") {
                set(button, "title", &JsValue::from_str(""));
            }
        }
    }

    let details = Object::new();
    set(details.as_ref(), "reason", &JsValue::from_str(reason));
    set(
        details.as_ref(),
        "startButtonCount",
        &JsValue::from_f64(buttons.len() as f64),
    );
    trace_port_conflict(
        net,
        if blocked {
            "r33-port-conflict-detected"
        } else {
            "r33-port-validation-clear"
        },
        validation,
        details.as_ref(),
    );

    let output = Object::new();
    set(output.as_ref(), "ok", &JsValue::from_bool(!blocked));
    set(output.as_ref(), "blocked", &JsValue::from_bool(blocked));
    set(output.as_ref(), "message", &JsValue::from_str(&message));
    set(output.as_ref(), "validation", validation);
    output.into()
}

fn configured_port_records(collected: &JsValue) -> Vec<PortOwnerRecord> {
    let mut records = Vec::new();
    if Array::is_array(collected) {
        for item in Array::from(collected).iter() {
            if !present(&item) {
                continue;
            }
            let port = crate::js_string_owned(&property(&item, "port"));
            let owners = property(&item, "owners");
            if Array::is_array(&owners) {
                for owner in Array::from(&owners).iter() {
                    records.push(owner_record_from_js(&owner, &port));
                }
            } else {
                records.push(owner_record_from_js(&item, ""));
            }
        }
        return records;
    }

    if collected.is_object() && !collected.is_null() {
        let object = Object::from(collected.clone());
        for entry in Object::entries(&object).iter() {
            let pair = Array::from(&entry);
            if pair.length() < 2 {
                continue;
            }
            let port = crate::js_string_owned(&pair.get(0));
            let owners = pair.get(1);
            if !Array::is_array(&owners) {
                continue;
            }
            for owner in Array::from(&owners).iter() {
                records.push(owner_record_from_js(&owner, &port));
            }
        }
    }
    records
}

fn global_used_port_values(
    records: &[PortOwnerRecord],
    change: &JsValue,
    planned_used: &[String],
) -> HashSet<String> {
    let target_net = crate::js_string_owned(&property(change, "net"));
    let target_owner = format!(
        "instance:{}",
        crate::js_string_owned(&property(change, "instanceId"))
    );
    let old_port = normalize_port_text(&crate::js_string_owned(&property(change, "oldPort")));
    let mut used = HashSet::new();

    for record in records {
        let port = normalize_port_text(&record.port);
        if port.is_empty() {
            continue;
        }
        let is_target_old_port =
            record.net == target_net && record.owner == target_owner && port == old_port;
        if !is_target_old_port {
            used.insert(port);
        }
    }
    for port in planned_used {
        let normalized = normalize_port_text(port);
        if !normalized.is_empty() {
            used.insert(normalized);
        }
    }
    used
}

fn js_set_from_values(values: &HashSet<String>) -> JsSet {
    let output = JsSet::new(&JsValue::UNDEFINED);
    for value in values {
        output.add(&JsValue::from_str(value));
    }
    output
}

fn instance_for_change(bridge_instances: &JsValue, net: &str, instance_id: &str) -> JsValue {
    let list = property(bridge_instances, net);
    if !Array::is_array(&list) {
        return JsValue::UNDEFINED;
    }
    Array::from(&list)
        .iter()
        .find(|item| crate::js_string_owned(&property(item, "id")) == instance_id)
        .unwrap_or(JsValue::UNDEFINED)
}

fn plan_port_autofix(
    active_net: &str,
    validation: &JsValue,
    bridge_instances: &JsValue,
) -> JsValue {
    let changes = Array::new();
    let seen = std::cell::RefCell::new(HashSet::<String>::new());
    let conflicts = property(validation, "conflicts");

    if crate::js_boolean(&property(validation, "ok")) || !Array::is_array(&conflicts) {
        let output = Object::new();
        set(output.as_ref(), "validation", validation);
        set(output.as_ref(), "changes", changes.as_ref());
        return output.into();
    }

    for conflict in Array::from(&conflicts).iter() {
        let owners = property(&conflict, "owners");
        if !Array::is_array(&owners) {
            continue;
        }
        let unique = bridge_unique_conflict_owners_r45(owners.clone());
        if unique.length() < 2 {
            continue;
        }
        let owners_to_change = bridge_owners_to_autofix_r45(active_net.to_owned(), unique.into());
        for owner in owners_to_change.iter() {
            let net = crate::js_string_owned(&property(&owner, "net"));
            let owner_text = crate::js_string_owned(&property(&owner, "owner"));
            let Some(instance_id) = owner_text.strip_prefix("instance:") else {
                continue;
            };
            if net.is_empty() || instance_id.is_empty() {
                continue;
            }

            let instance = instance_for_change(bridge_instances, &net, instance_id);
            if !present(&instance) {
                continue;
            }
            let old_port = crate::js_string_owned(&property(&conflict, "port"));
            let normalized_old = normalize_port_text(&old_port);
            let kind = if normalize_port_text(&crate::js_string_owned(&property(
                &instance,
                "instancePort",
            ))) == normalized_old
            {
                "stratum"
            } else if normalize_port_text(&crate::js_string_owned(&property(
                &instance,
                "instanceProm",
            ))) == normalized_old
            {
                "prom"
            } else {
                ""
            };
            if kind.is_empty() {
                continue;
            }

            let key = format!("{net}:{instance_id}:{kind}:{old_port}");
            if !seen.borrow_mut().insert(key) {
                continue;
            }

            let changed_owner = Object::new();
            set(
                changed_owner.as_ref(),
                "net",
                &JsValue::from_str(&crate::js_string_owned(&property(&owner, "net"))),
            );
            set(
                changed_owner.as_ref(),
                "role",
                &JsValue::from_str(&crate::js_string_owned(&property(&owner, "role"))),
            );
            set(
                changed_owner.as_ref(),
                "owner",
                &JsValue::from_str(&crate::js_string_owned(&property(&owner, "owner"))),
            );

            let change = Object::new();
            set(change.as_ref(), "net", &JsValue::from_str(&net));
            set(
                change.as_ref(),
                "instanceId",
                &JsValue::from_str(instance_id),
            );
            set(change.as_ref(), "kind", &JsValue::from_str(kind));
            set(change.as_ref(), "oldPort", &JsValue::from_str(&old_port));
            set(change.as_ref(), "changedOwner", changed_owner.as_ref());
            changes.push(change.as_ref());
        }
    }

    let output = Object::new();
    set(output.as_ref(), "validation", validation);
    set(output.as_ref(), "changes", changes.as_ref());
    output.into()
}

#[wasm_bindgen(js_name = bridgePlanPortAutofixR37)]
pub fn bridge_plan_port_autofix_r37(
    active_net: String,
    validation: JsValue,
    bridge_instances: JsValue,
) -> JsValue {
    plan_port_autofix(&active_net, &validation, &bridge_instances)
}

#[wasm_bindgen(js_name = bridgeChooseReplacementPortR37)]
pub fn bridge_choose_replacement_port_r37(
    change: JsValue,
    planned_used: JsValue,
    bridge_instances: JsValue,
    collected_records: JsValue,
) -> Result<String, JsValue> {
    let planned = if Array::is_array(&planned_used) {
        Array::from(&planned_used)
            .iter()
            .map(|value| crate::js_string_owned(&value))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    choose_replacement_port(&change, &planned, &bridge_instances, &collected_records)
}

#[wasm_bindgen(js_name = bridgeWriteInstancePortR37)]
pub fn bridge_write_instance_port_r37(
    bridge_instances: JsValue,
    net: String,
    instance_id: String,
    kind: String,
    new_port: String,
) -> bool {
    let list = property(&bridge_instances, &net);
    if !Array::is_array(&list) {
        return false;
    }
    let instance = Array::from(&list)
        .iter()
        .find(|item| crate::js_string_owned(&property(item, "id")) == instance_id)
        .unwrap_or(JsValue::UNDEFINED);
    if !present(&instance) {
        return false;
    }
    let field_name = if kind == "prom" {
        "instanceProm"
    } else {
        "instancePort"
    };
    let normalized = normalize_soft_text(&new_port);
    if normalized.is_empty() {
        return false;
    }
    set(&instance, field_name, &JsValue::from_str(&normalized));
    true
}

fn records_to_js(records: &[PortOwnerRecord]) -> Array {
    let output = Array::new();
    for record in records {
        let row = Object::new();
        set(row.as_ref(), "port", &JsValue::from_str(&record.port));
        set(row.as_ref(), "net", &JsValue::from_str(&record.net));
        set(row.as_ref(), "role", &JsValue::from_str(&record.role));
        set(row.as_ref(), "owner", &JsValue::from_str(&record.owner));
        output.push(row.as_ref());
    }
    output
}

fn apply_port_autofix(
    active_net: &str,
    initial_validation: &JsValue,
    bridge_instances: &JsValue,
    collected_records: &JsValue,
    max_passes: usize,
) -> JsValue {
    let mut records = configured_port_records(collected_records);
    let mut validation = initial_validation.clone();
    let all_changed = Array::new();

    for pass in 1..=max_passes {
        let plan = plan_port_autofix(active_net, &validation, bridge_instances);
        let changes = property(&plan, "changes");
        if !Array::is_array(&changes) {
            break;
        }
        if Array::from(&changes).length() == 0 {
            break;
        }

        let mut pass_changed = 0_u32;
        let mut planned_used = (0..all_changed.length())
            .map(|index| crate::js_string_owned(&property(&all_changed.get(index), "newPort")))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        for change in Array::from(&changes).iter() {
            let records_js = records_to_js(&records);
            let Ok(new_port) =
                choose_replacement_port(&change, &planned_used, bridge_instances, &records_js)
            else {
                continue;
            };
            if new_port.is_empty() {
                continue;
            }

            let net = crate::js_string_owned(&property(&change, "net"));
            let instance_id = crate::js_string_owned(&property(&change, "instanceId"));
            let kind = crate::js_string_owned(&property(&change, "kind"));
            let old_port =
                normalize_port_text(&crate::js_string_owned(&property(&change, "oldPort")));
            if !bridge_write_instance_port_r37(
                bridge_instances.clone(),
                net.clone(),
                instance_id.clone(),
                kind,
                new_port.clone(),
            ) {
                continue;
            }

            let owner = format!("instance:{instance_id}");
            let mut record_updated = false;
            for record in &mut records {
                if record.net == net
                    && record.owner == owner
                    && normalize_port_text(&record.port) == old_port
                {
                    record.port = new_port.clone();
                    record_updated = true;
                    break;
                }
            }
            if !record_updated {
                records.push(PortOwnerRecord {
                    port: new_port.clone(),
                    net: net.clone(),
                    role: "instance".to_owned(),
                    owner,
                });
            }

            let applied = Object::new();
            for key in ["net", "instanceId", "kind", "oldPort", "changedOwner"] {
                let value = property(&change, key);
                set(applied.as_ref(), key, &value);
            }
            set(applied.as_ref(), "newPort", &JsValue::from_str(&new_port));
            set(applied.as_ref(), "pass", &JsValue::from_f64(pass as f64));
            all_changed.push(applied.as_ref());
            planned_used.push(new_port);
            pass_changed += 1;
        }

        if pass_changed == 0 {
            break;
        }

        let conflicts = validate_port_conflicts_records(&records, active_net);
        let conflict_values = Array::new();
        for conflict in &conflicts {
            conflict_values.push(&conflict_record_to_js(conflict));
        }
        let next = Object::new();
        set(
            next.as_ref(),
            "ok",
            &JsValue::from_bool(conflicts.is_empty()),
        );
        set(next.as_ref(), "conflicts", conflict_values.as_ref());
        set(
            next.as_ref(),
            "message",
            &JsValue::from_str(&conflict_message_records(&conflicts)),
        );
        validation = next.into();

        if crate::js_boolean(&property(&validation, "ok")) {
            break;
        }
    }

    let output = Object::new();
    set(output.as_ref(), "changes", all_changed.as_ref());
    set(output.as_ref(), "validation", &validation);
    set(
        output.as_ref(),
        "finalOk",
        &JsValue::from_bool(crate::js_boolean(&property(&validation, "ok"))),
    );
    output.into()
}

#[wasm_bindgen(js_name = bridgeApplyPortAutofixR37)]
pub fn bridge_apply_port_autofix_r37(
    active_net: String,
    validation: JsValue,
    bridge_instances: JsValue,
    collected_records: JsValue,
    max_passes: u32,
) -> JsValue {
    apply_port_autofix(
        &active_net,
        &validation,
        &bridge_instances,
        &collected_records,
        max_passes as usize,
    )
}

fn choose_replacement_port(
    change: &JsValue,
    planned_used: &[String],
    bridge_instances: &JsValue,
    collected_records: &JsValue,
) -> Result<String, JsValue> {
    let net = crate::js_string_owned(&property(change, "net"));
    let kind = {
        let value = crate::js_string_owned(&property(change, "kind"));
        if value == "prom" {
            "prom".to_owned()
        } else {
            "stratum".to_owned()
        }
    };
    let instance_id = crate::js_string_owned(&property(change, "instanceId"));
    let old_port = normalize_port_text(&crate::js_string_owned(&property(change, "oldPort")));
    let records = configured_port_records(collected_records);
    let used_values = global_used_port_values(&records, change, planned_used);
    let used = js_set_from_values(&used_values);

    let instance = instance_for_change(bridge_instances, &net, &instance_id);
    if present(&instance) {
        let other_name = if kind == "prom" {
            "instancePort"
        } else {
            "instanceProm"
        };
        let other = normalize_port_text(&crate::js_string_owned(&property(&instance, other_name)));
        if !other.is_empty() && other != old_port {
            set_add(used.as_ref(), &other);
        }
    }

    let profile = profile_for_current_settings(&net);
    let range = property(&profile, &kind);
    let fallback = ["instanceStart", "preferred", "min"]
        .iter()
        .map(|key| crate::js_string_owned(&property(&range, key)))
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| "1".to_owned());

    let in_range = find_unused_in_range_impl(&range, used.as_ref(), &fallback);
    if !in_range.is_empty() {
        return Ok(in_range);
    }
    find_nearest_unused_impl(&fallback, used.as_ref())
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

#[wasm_bindgen(js_name = bridgeValidatePortConflictsR5)]
pub fn bridge_validate_port_conflicts_r5(items: Array, active_net: String) -> JsValue {
    let records = items
        .iter()
        .map(|item| owner_record_from_js(&item, ""))
        .collect::<Vec<_>>();
    let conflicts = validate_port_conflicts_records(&records, &active_net);

    let conflict_values = Array::new();
    for conflict in &conflicts {
        conflict_values.push(&conflict_record_to_js(conflict));
    }

    let output = Object::new();
    set(
        output.as_ref(),
        "ok",
        &JsValue::from_bool(conflicts.is_empty()),
    );
    set(output.as_ref(), "conflicts", conflict_values.as_ref());
    set(
        output.as_ref(),
        "message",
        &JsValue::from_str(&conflict_message_records(&conflicts)),
    );
    output.into()
}

#[wasm_bindgen(js_name = bridgePortConflictCompactSummaryR33)]
pub fn bridge_port_conflict_compact_summary_r33(validation: JsValue) -> Array {
    let output = Array::new();
    for conflict in conflicts_from_validation(&validation) {
        output.push(&conflict_record_to_js(&conflict));
    }
    output
}

#[wasm_bindgen(js_name = bridgePortConflictMessageR33)]
pub fn bridge_port_conflict_message_r33(validation: JsValue) -> String {
    port_conflict_message(&validation)
}

#[wasm_bindgen(js_name = bridgeTracePortConflictR33)]
pub fn bridge_trace_port_conflict_r33(
    net: String,
    phase: String,
    validation: JsValue,
    details: JsValue,
) -> bool {
    trace_port_conflict(&net, &phase, &validation, &details)
}

#[wasm_bindgen(js_name = bridgeStartButtonsForNetR33)]
pub fn bridge_start_buttons_for_net_r33(net: String) -> Array {
    let output = Array::new();
    for button in start_buttons_for_net(&net) {
        output.push(&button);
    }
    output
}

#[wasm_bindgen(js_name = bridgeApplyPortConflictStartStateR33)]
pub fn bridge_apply_port_conflict_start_state_r33(
    net: String,
    validation: JsValue,
    reason: String,
) -> JsValue {
    apply_port_conflict_start_state(&net, &validation, &reason)
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

    #[test]
    fn conflict_validation_matches_active_network_and_instance_policy() {
        let records = vec![
            PortOwnerRecord {
                port: "5555".to_owned(),
                net: "mainnet".to_owned(),
                role: "bridge-stratum".to_owned(),
                owner: "stratumPort".to_owned(),
            },
            PortOwnerRecord {
                port: "5555".to_owned(),
                net: "mainnet".to_owned(),
                role: "instance".to_owned(),
                owner: "instance:1".to_owned(),
            },
            PortOwnerRecord {
                port: "5655".to_owned(),
                net: "testnet10".to_owned(),
                role: "bridge-stratum".to_owned(),
                owner: "stratumPort".to_owned(),
            },
            PortOwnerRecord {
                port: "5655".to_owned(),
                net: "testnet13".to_owned(),
                role: "bridge-stratum".to_owned(),
                owner: "stratumPort".to_owned(),
            },
        ];

        let mainnet = validate_port_conflicts_records(&records, "mainnet");
        assert_eq!(mainnet.len(), 1);
        assert_eq!(mainnet[0].port, "5555");

        let testnet10 = validate_port_conflicts_records(&records, "testnet10");
        assert_eq!(testnet10.len(), 2);
        assert_eq!(testnet10[0].port, "5555");
        assert_eq!(testnet10[1].port, "5655");
    }

    #[test]
    fn conflict_validation_preserves_logical_endpoint_alias_exception() {
        let records = vec![
            PortOwnerRecord {
                port: "16110".to_owned(),
                net: "mainnet".to_owned(),
                role: "default-kaspad-rpc".to_owned(),
                owner: "profile".to_owned(),
            },
            PortOwnerRecord {
                port: "16110".to_owned(),
                net: "mainnet".to_owned(),
                role: "bridge-external-kaspad".to_owned(),
                owner: "kaspadAddress".to_owned(),
            },
        ];

        assert!(validate_port_conflicts_records(&records, "mainnet").is_empty());
    }

    #[test]
    fn conflict_message_preserves_legacy_shape_and_order() {
        let conflicts = vec![PortConflictRecord {
            port: "5555".to_owned(),
            owners: vec![
                PortOwnerRecord {
                    port: "5555".to_owned(),
                    net: "mainnet".to_owned(),
                    role: "bridge-stratum".to_owned(),
                    owner: "stratumPort".to_owned(),
                },
                PortOwnerRecord {
                    port: "5555".to_owned(),
                    net: "mainnet".to_owned(),
                    role: "instance".to_owned(),
                    owner: "instance:7".to_owned(),
                },
            ],
        }];
        assert_eq!(
            conflict_message_records(&conflicts),
            "port 5555 => mainnet/bridge-stratum/stratumPort | mainnet/instance/instance:7"
        );
    }
}
