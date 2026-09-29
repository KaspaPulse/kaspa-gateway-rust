use js_sys::{Array, Object, Reflect, Set as JsSet};
use wasm_bindgen::{JsCast, prelude::*};

use crate::{bridge_frontend_helpers, bridge_instance_settings, bridge_port_core};

const PORT_FIELDS: [(&str, &str); 9] = [
    ("stratumPort", "bridge-stratum"),
    ("promPort", "bridge-prometheus"),
    ("webDashboardPort", "bridge-dashboard"),
    ("healthCheckPort", "bridge-health"),
    ("kaspadAddress", "bridge-external-kaspad"),
    ("inprocessRpcListen", "inprocess-rpc"),
    ("inprocessRpcListenBorsh", "inprocess-rpc-borsh"),
    ("inprocessRpcListenJson", "inprocess-rpc-json"),
    ("inprocessListen", "inprocess-p2p"),
];

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

fn truthy(value: &JsValue) -> bool {
    present(value) && crate::js_boolean(value)
}

fn object_keys(value: &JsValue) -> Vec<String> {
    if !present(value) {
        return Vec::new();
    }
    let object: Object = value.clone().unchecked_into();
    Object::keys(&object)
        .iter()
        .map(|value| crate::js_string_owned(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn clone_object(value: &JsValue) -> Object {
    let output = Object::new();
    for key in object_keys(value) {
        set(output.as_ref(), &key, &property(value, &key));
    }
    output
}

fn network_profiles() -> Array {
    bridge_frontend_helpers::bridge_network_profiles()
}

fn instance_list(bridge_instances: &JsValue, net: &str) -> Array {
    let value = property(bridge_instances, net);
    if Array::is_array(&value) {
        Array::from(&value)
    } else {
        Array::new()
    }
}

fn add_used(used: &JsSet, value: &JsValue) {
    let normalized = bridge_port_core::bridge_normalize_port_r9(value.clone());
    if bridge_port_core::bridge_port_is_valid_r9(JsValue::from_str(&normalized)) {
        used.add(&JsValue::from_str(&normalized));
    }
}

fn add_extracted_ports(used: &JsSet, value: &str) {
    for port in bridge_port_core::bridge_extract_ports_from_text_r5(JsValue::from_str(value)).iter()
    {
        add_used(used, &port);
    }
}

fn push_configured(items: &Array, port: JsValue, role: &str, owner: &str, net: &str) {
    bridge_port_core::bridge_push_port_r5(
        items.clone(),
        port,
        JsValue::from_str(role),
        JsValue::from_str(owner),
        JsValue::from_str(net),
    );
}

fn trace_port_profile(net: &str, phase: &str, details: JsValue) {
    let payload = Object::new();
    set(payload.as_ref(), "patch", &JsValue::from_str("R35B"));
    set(
        payload.as_ref(),
        "owner",
        &JsValue::from_str("bridge-network-port-profile-soft-policy"),
    );
    set(
        payload.as_ref(),
        "policy",
        &JsValue::from_str("manual-valid-unused-ports-accepted-even-inside-other-network-range"),
    );
    set(payload.as_ref(), "details", &details);
    let _ = bridge_frontend_helpers::bridge_small_owner_trace_r44d(
        JsValue::from_str(net),
        JsValue::from_str("port-profile"),
        JsValue::from_str(phase),
        payload.into(),
    );
}

fn range_pair(profile: &JsValue, kind: &str) -> Array {
    let range = property(profile, kind);
    let output = Array::new();
    output.push(&property(&range, "min"));
    output.push(&property(&range, "max"));
    output
}

fn used_port_set_r9_impl(
    bridge_instances: &JsValue,
    skip_net: &str,
    skip_instance_id: &str,
) -> JsSet {
    let used = JsSet::new(&JsValue::UNDEFINED);
    for profile in network_profiles().iter() {
        let net = crate::js_string_owned(&property(&profile, "key"));
        for key in ["kaspadPort", "stratumPort", "promPort"] {
            add_used(&used, &property(&profile, key));
        }
        for (field, _) in PORT_FIELDS {
            let value = bridge_frontend_helpers::bridge_value(net.clone(), field.to_owned());
            add_extracted_ports(&used, &value);
        }
    }

    for net in object_keys(bridge_instances) {
        for item in instance_list(bridge_instances, &net).iter() {
            let item_id = crate::js_string_owned(&property(&item, "id"));
            if net == skip_net && item_id == skip_instance_id {
                continue;
            }
            add_used(&used, &property(&item, "instancePort"));
            add_used(&used, &property(&item, "instanceProm"));
            add_extracted_ports(&used, &crate::js_string_owned(&property(&item, "instance")));
        }
    }
    used
}

fn used_port_set_excluding_network_instances_impl(
    bridge_instances: &JsValue,
    active_net: &str,
) -> JsSet {
    let used = JsSet::new(&JsValue::UNDEFINED);
    for profile in network_profiles().iter() {
        let net = crate::js_string_owned(&property(&profile, "key"));
        for key in ["kaspadPort", "stratumPort", "promPort"] {
            add_used(&used, &property(&profile, key));
        }
        for (field, _) in PORT_FIELDS {
            let value = bridge_frontend_helpers::bridge_value(net.clone(), field.to_owned());
            add_extracted_ports(&used, &value);
        }
    }

    for net in object_keys(bridge_instances) {
        if net == active_net {
            continue;
        }
        for item in instance_list(bridge_instances, &net).iter() {
            add_used(&used, &property(&item, "instancePort"));
            add_used(&used, &property(&item, "instanceProm"));
            add_extracted_ports(&used, &crate::js_string_owned(&property(&item, "instance")));
        }
    }
    used
}

fn assign_missing_instance_ports_impl(
    bridge_instances: &JsValue,
    net: &str,
    instance: &JsValue,
) -> Result<JsValue, JsValue> {
    let profile = bridge_port_core::bridge_port_profile_r35b(net.to_owned());
    let instance_id = crate::js_string_owned(&property(instance, "id"));
    let used = used_port_set_r9_impl(bridge_instances, net, &instance_id);
    let current_port =
        bridge_port_core::bridge_normalize_port_r9(property(instance, "instancePort"));
    let current_prom =
        bridge_port_core::bridge_normalize_port_r9(property(instance, "instanceProm"));

    let follow_port = bridge_port_core::bridge_instance_port_should_follow_external_range_r91(
        net.to_owned(),
        "stratum".to_owned(),
        JsValue::from_str(&current_port),
    );
    let follow_prom = bridge_port_core::bridge_instance_port_should_follow_external_range_r91(
        net.to_owned(),
        "prom".to_owned(),
        JsValue::from_str(&current_prom),
    );

    let stratum = property(&profile, "stratum");
    let prom = property(&profile, "prom");
    let manual_port =
        !follow_port && bridge_port_core::bridge_port_is_valid_r9(JsValue::from_str(&current_port));
    let manual_prom =
        !follow_prom && bridge_port_core::bridge_port_is_valid_r9(JsValue::from_str(&current_prom));

    let instance_port = if manual_port {
        current_port.clone()
    } else {
        bridge_port_core::bridge_find_recommended_or_nearest_unused_port_r35b(
            net.to_owned(),
            "stratum".to_owned(),
            JsValue::from(used.clone()),
            property(&stratum, "instanceStart"),
        )?
    };
    let instance_prom = if manual_prom {
        current_prom.clone()
    } else {
        bridge_port_core::bridge_find_recommended_or_nearest_unused_port_r35b(
            net.to_owned(),
            "prom".to_owned(),
            JsValue::from(used.clone()),
            property(&prom, "instanceStart"),
        )?
    };

    let details = Object::new();
    set(
        details.as_ref(),
        "instanceId",
        &JsValue::from_str(&instance_id),
    );
    set(
        details.as_ref(),
        "acceptedManualInstancePort",
        &JsValue::from_bool(manual_port),
    );
    set(
        details.as_ref(),
        "acceptedManualInstanceProm",
        &JsValue::from_bool(manual_prom),
    );
    set(
        details.as_ref(),
        "instancePort",
        &JsValue::from_str(&instance_port),
    );
    set(
        details.as_ref(),
        "instanceProm",
        &JsValue::from_str(&instance_prom),
    );
    set(
        details.as_ref(),
        "stratumRange",
        range_pair(&profile, "stratum").as_ref(),
    );
    set(
        details.as_ref(),
        "promRange",
        range_pair(&profile, "prom").as_ref(),
    );
    set(
        details.as_ref(),
        "stratumExternalBase",
        &property(&stratum, "externalBase"),
    );
    set(
        details.as_ref(),
        "promExternalBase",
        &property(&prom, "externalBase"),
    );
    set(
        details.as_ref(),
        "policy",
        &JsValue::from_str(
            "instances follow bridge-level external port settings unless a valid out-of-range manual port is clearly set",
        ),
    );
    trace_port_profile(
        net,
        "r91-assign-instance-ports-from-external-range",
        details.into(),
    );

    let output = clone_object(instance);
    set(
        output.as_ref(),
        "instancePort",
        &JsValue::from_str(&instance_port),
    );
    set(
        output.as_ref(),
        "instanceProm",
        &JsValue::from_str(&instance_prom),
    );
    Ok(output.into())
}

fn reassign_instance_ports_impl(
    bridge_instances: &JsValue,
    net: &str,
    reason: &str,
) -> Result<bool, JsValue> {
    let current = property(bridge_instances, net);
    if !Array::is_array(&current) {
        return Ok(false);
    }

    let profile = bridge_port_core::bridge_port_profile_r35b(net.to_owned());
    let used = used_port_set_excluding_network_instances_impl(bridge_instances, net);
    let input = Array::from(&current);
    let output = Array::new();
    let mut changed = false;

    for (index, raw) in input.iter().enumerate() {
        let raw_id = property(&raw, "id");
        let fallback_id = if truthy(&raw_id) {
            raw_id
        } else {
            JsValue::from_f64(js_sys::Date::now() + index as f64)
        };
        let instance = bridge_instance_settings::bridge_normalize_instance_record(raw, fallback_id);
        let current_port =
            bridge_port_core::bridge_normalize_port_r9(property(&instance, "instancePort"));
        let current_prom =
            bridge_port_core::bridge_normalize_port_r9(property(&instance, "instanceProm"));
        let follow_port = bridge_port_core::bridge_instance_port_should_follow_external_range_r91(
            net.to_owned(),
            "stratum".to_owned(),
            JsValue::from_str(&current_port),
        );
        let follow_prom = bridge_port_core::bridge_instance_port_should_follow_external_range_r91(
            net.to_owned(),
            "prom".to_owned(),
            JsValue::from_str(&current_prom),
        );

        let stratum = property(&profile, "stratum");
        let prom = property(&profile, "prom");
        let instance_port = if follow_port {
            let base = crate::js_number(&property(&stratum, "instanceStart")) + index as f64;
            let next = bridge_port_core::bridge_find_recommended_or_nearest_unused_port_r35b(
                net.to_owned(),
                "stratum".to_owned(),
                JsValue::from(used.clone()),
                JsValue::from_f64(base),
            )?;
            changed |= next != current_port;
            next
        } else {
            add_used(&used, &JsValue::from_str(&current_port));
            current_port.clone()
        };
        let instance_prom = if follow_prom {
            let base = crate::js_number(&property(&prom, "instanceStart")) + index as f64;
            let next = bridge_port_core::bridge_find_recommended_or_nearest_unused_port_r35b(
                net.to_owned(),
                "prom".to_owned(),
                JsValue::from(used.clone()),
                JsValue::from_f64(base),
            )?;
            changed |= next != current_prom;
            next
        } else {
            add_used(&used, &JsValue::from_str(&current_prom));
            current_prom.clone()
        };

        let next = clone_object(&instance);
        set(next.as_ref(), "instance", &JsValue::from_str(""));
        set(
            next.as_ref(),
            "instancePort",
            &JsValue::from_str(&instance_port),
        );
        set(
            next.as_ref(),
            "instanceProm",
            &JsValue::from_str(&instance_prom),
        );
        output.push(next.as_ref());
    }

    set(bridge_instances, net, output.as_ref());
    if changed {
        let details = Object::new();
        set(details.as_ref(), "reason", &JsValue::from_str(reason));
        set(
            details.as_ref(),
            "stratumRange",
            range_pair(&profile, "stratum").as_ref(),
        );
        set(
            details.as_ref(),
            "promRange",
            range_pair(&profile, "prom").as_ref(),
        );
        set(
            details.as_ref(),
            "stratumExternalBase",
            &property(&property(&profile, "stratum"), "externalBase"),
        );
        set(
            details.as_ref(),
            "promExternalBase",
            &property(&property(&profile, "prom"), "externalBase"),
        );
        set(
            details.as_ref(),
            "instanceCount",
            &JsValue::from_f64(output.length() as f64),
        );
        trace_port_profile(
            net,
            "r91-reassign-instances-from-external-range",
            details.into(),
        );
    }
    Ok(changed)
}

fn ensure_instance_state_impl(
    bridge_instances: &JsValue,
    active_instance: &JsValue,
    net: &str,
) -> Result<(), JsValue> {
    let current = property(bridge_instances, net);
    let input = if Array::is_array(&current) {
        Array::from(&current)
    } else {
        let created = Array::new();
        set(bridge_instances, net, created.as_ref());
        created
    };

    if input.length() == 0 {
        let record = bridge_instance_settings::bridge_default_instance_record(JsValue::from_f64(
            js_sys::Date::now(),
        ));
        input.push(&record);
        set(bridge_instances, net, input.as_ref());
    }

    let output = Array::new();
    for (index, raw) in input.iter().enumerate() {
        let raw_id = property(&raw, "id");
        let fallback_id = if truthy(&raw_id) {
            raw_id
        } else {
            JsValue::from_f64(js_sys::Date::now() + index as f64)
        };
        let normalized =
            bridge_instance_settings::bridge_normalize_instance_record(raw, fallback_id);
        let assigned = assign_missing_instance_ports_impl(bridge_instances, net, &normalized)?;
        output.push(&assigned);
    }
    set(bridge_instances, net, output.as_ref());

    if !truthy(&property(active_instance, net)) && output.length() > 0 {
        let first = output.get(0);
        set(active_instance, net, &property(&first, "id"));
    }
    Ok(())
}

#[wasm_bindgen(js_name = bridgeCollectConfiguredPortsR5)]
pub fn bridge_collect_configured_ports_r5(
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> Result<Array, JsValue> {
    let items = Array::new();
    for profile in network_profiles().iter() {
        let net = crate::js_string_owned(&property(&profile, "key"));
        push_configured(
            &items,
            property(&profile, "kaspadPort"),
            "default-kaspad-rpc",
            "BRIDGE_NETWORKS.kaspadPort",
            &net,
        );
        push_configured(
            &items,
            property(&profile, "stratumPort"),
            "default-stratum",
            "BRIDGE_NETWORKS.stratumPort",
            &net,
        );
        push_configured(
            &items,
            property(&profile, "promPort"),
            "default-prometheus",
            "BRIDGE_NETWORKS.promPort",
            &net,
        );

        for (field, role) in PORT_FIELDS {
            let value = bridge_frontend_helpers::bridge_value(net.clone(), field.to_owned());
            for port in
                bridge_port_core::bridge_extract_ports_from_text_r5(JsValue::from_str(&value))
                    .iter()
            {
                push_configured(&items, port, role, field, &net);
            }
        }

        ensure_instance_state_impl(&bridge_instances, &active_instance, &net)?;
        for instance in instance_list(&bridge_instances, &net).iter() {
            let text = bridge_instance_settings::bridge_build_upstream_instance_arg(
                net.clone(),
                instance.clone(),
            );
            for port in
                bridge_port_core::bridge_extract_ports_from_text_r5(JsValue::from_str(&text)).iter()
            {
                let owner = format!(
                    "instance:{}",
                    crate::js_string_owned(&property(&instance, "id"))
                );
                push_configured(&items, port, "instance", &owner, &net);
            }
        }
    }
    Ok(items)
}

#[wasm_bindgen(js_name = bridgeUsedPortSetR9)]
pub fn bridge_used_port_set_r9(
    bridge_instances: JsValue,
    skip_net: String,
    skip_instance_id: JsValue,
) -> JsValue {
    let used = used_port_set_r9_impl(
        &bridge_instances,
        &skip_net,
        &crate::js_string_owned(&skip_instance_id),
    );
    used.into()
}

#[wasm_bindgen(js_name = bridgeUsedPortSetExcludingNetworkInstancesR91)]
pub fn bridge_used_port_set_excluding_network_instances_r91(
    bridge_instances: JsValue,
    active_net: String,
) -> JsValue {
    used_port_set_excluding_network_instances_impl(&bridge_instances, &active_net).into()
}

#[wasm_bindgen(js_name = bridgeAssignMissingInstancePortsR9)]
pub fn bridge_assign_missing_instance_ports_r9(
    bridge_instances: JsValue,
    net: String,
    instance: JsValue,
) -> Result<JsValue, JsValue> {
    assign_missing_instance_ports_impl(&bridge_instances, &net, &instance)
}

#[wasm_bindgen(js_name = bridgeReassignInstancePortsFromExternalRangeR91)]
pub fn bridge_reassign_instance_ports_from_external_range_r91(
    bridge_instances: JsValue,
    net: String,
    reason: String,
) -> Result<bool, JsValue> {
    reassign_instance_ports_impl(&bridge_instances, &net, &reason)
}

#[wasm_bindgen(js_name = bridgeCreateInstanceRecordR9)]
pub fn bridge_create_instance_record_r9(
    bridge_instances: JsValue,
    net: String,
) -> Result<JsValue, JsValue> {
    let _ = reassign_instance_ports_impl(&bridge_instances, &net, "before-create-instance")?;
    let id = js_sys::Date::now() + (js_sys::Math::random() * 1000.0).floor();
    let record = bridge_instance_settings::bridge_default_instance_record(JsValue::from_f64(id));
    assign_missing_instance_ports_impl(&bridge_instances, &net, &record)
}

#[wasm_bindgen(js_name = bridgeEnsureInstanceState)]
pub fn bridge_ensure_instance_state(
    bridge_instances: JsValue,
    active_instance: JsValue,
    net: String,
) -> Result<(), JsValue> {
    ensure_instance_state_impl(&bridge_instances, &active_instance, &net)
}
