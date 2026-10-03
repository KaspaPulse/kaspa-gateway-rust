use js_sys::{Array, Function, JSON, Object, Reflect};
use std::collections::HashSet;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

use crate::{bridge_frontend_helpers, bridge_port_core, bridge_port_orchestration};

const TIMER_KEY: &str = "__kgwBridgePortConflictValidationTimerR33";

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

fn window() -> JsValue {
    let global: JsValue = js_sys::global().into();
    let value = property(&global, "window");
    if present(&value) { value } else { global }
}

fn truthy_text(target: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = property(target, name);
        if crate::js_boolean(&value) {
            return crate::js_string_owned(&value);
        }
    }
    String::new()
}

fn normalized_valid_port(value: &str) -> String {
    bridge_port_core::parse_valid_port(value)
        .map(|port| port.to_string())
        .unwrap_or_default()
}

fn default_validation() -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "ok", &JsValue::TRUE);
    set(output.as_ref(), "conflicts", Array::new().as_ref());
    output.into()
}

fn default_scoped_result() -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "ok", &JsValue::TRUE);
    set(output.as_ref(), "conflictCount", &JsValue::from_f64(0.0));
    set(output.as_ref(), "conflicts", Array::new().as_ref());
    set(output.as_ref(), "message", &JsValue::from_str(""));
    output.into()
}
fn trace_scoped_start(net: &str, phase: &str, details: &JsValue) {
    let _ = bridge_frontend_helpers::bridge_small_owner_trace_r44d(
        JsValue::from_str(net),
        JsValue::from_str("port-conflict"),
        JsValue::from_str(phase),
        details.clone(),
    );
}

fn fallback_structured_instances(
    net: &str,
    bridge_instances: &JsValue,
    active_instance: &JsValue,
) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "version", &JsValue::from_f64(1.0));
    set(
        output.as_ref(),
        "activeInstance",
        &JsValue::from_str(&crate::js_string_owned(&property(active_instance, net))),
    );
    let instances = property(bridge_instances, net);
    if Array::is_array(&instances) {
        set(output.as_ref(), "instances", &instances);
    } else {
        set(output.as_ref(), "instances", Array::new().as_ref());
    }
    output.into()
}

fn read_structured_instances(
    net: &str,
    reader: &JsValue,
    bridge_instances: &JsValue,
    active_instance: &JsValue,
) -> Result<JsValue, JsValue> {
    if let Ok(reader) = reader.clone().dyn_into::<Function>() {
        return reader.call1(&JsValue::UNDEFINED, &JsValue::from_str(net));
    }
    Ok(fallback_structured_instances(
        net,
        bridge_instances,
        active_instance,
    ))
}

fn scoped_start_assertion(
    net: &str,
    structured_reader: &JsValue,
    bridge_instances: &JsValue,
    active_instance: &JsValue,
) -> Result<JsValue, JsValue> {
    if net != "mainnet" {
        return Ok(JsValue::UNDEFINED);
    }

    let cfg = bridge_frontend_helpers::bridge_network_profile(net.to_owned());
    let default_port = normalized_valid_port(&truthy_text(&cfg, &["stratumPort", "port"]));
    let structured =
        read_structured_instances(net, structured_reader, bridge_instances, active_instance)?;
    let instances_value = property(&structured, "instances");
    let instances = if Array::is_array(&instances_value) {
        Array::from(&instances_value)
    } else {
        Array::new()
    };
    let active_id = crate::js_string_owned(&property(&structured, "activeInstance"));
    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for item in instances.iter() {
        if !item.is_object() || item.is_null() {
            continue;
        }
        let id = crate::js_string_owned(&property(&item, "id"));
        let key = if id.is_empty() {
            JSON::stringify(&item)
                .ok()
                .map(|value| crate::js_string_owned(value.as_ref()))
                .unwrap_or_default()
        } else {
            id
        };
        if seen.insert(key) {
            unique.push(item);
        }
    }

    let active_record = if !active_id.is_empty() {
        unique
            .iter()
            .find(|item| crate::js_string_owned(&property(item, "id")) == active_id)
            .cloned()
            .or_else(|| unique.first().cloned())
    } else {
        unique.first().cloned()
    };

    let active_port = active_record
        .as_ref()
        .map(|record| {
            normalized_valid_port(&truthy_text(
                record,
                &["instancePort", "port", "stratumPort"],
            ))
        })
        .unwrap_or_default();

    let details = Object::new();
    set(details.as_ref(), "patch", &JsValue::from_str("R110H"));
    set(
        details.as_ref(),
        "owner",
        &JsValue::from_str("existing-bridge-port-conflict-owner-r5-scoped-start"),
    );
    set(details.as_ref(), "network", &JsValue::from_str(net));
    set(
        details.as_ref(),
        "defaultPort",
        &JsValue::from_str(&default_port),
    );
    set(
        details.as_ref(),
        "activeInstanceId",
        &JsValue::from_str(&active_id),
    );
    set(
        details.as_ref(),
        "activeInstancePort",
        &JsValue::from_str(&active_port),
    );
    set(
        details.as_ref(),
        "instanceCount",
        &JsValue::from_f64(instances.length() as f64),
    );
    set(
        details.as_ref(),
        "uniqueInstanceCount",
        &JsValue::from_f64(unique.len() as f64),
    );
    set(
        details.as_ref(),
        "policy",
        &JsValue::from_str(
            "start checks current network active instance only; stale cross-network conflicts are not blockers",
        ),
    );
    trace_scoped_start(net, "r110h-scoped-start-conflict-check", details.as_ref());

    let Some(_record) = active_record else {
        return Ok(default_scoped_result());
    };
    if active_port.is_empty() {
        let message = "Active Bridge instance has no valid Stratum port.";
        set(details.as_ref(), "message", &JsValue::from_str(message));
        trace_scoped_start(net, "r110h-active-instance-port-invalid", details.as_ref());
        return Err(JsValue::from_str(message));
    }

    Ok(default_scoped_result())
}

fn validate_and_apply(
    bridge_instances: &JsValue,
    active_instance: &JsValue,
    net: &str,
    reason: &str,
) -> Result<JsValue, JsValue> {
    let normalized = net.trim();
    if normalized.is_empty() {
        let output = Object::new();
        set(output.as_ref(), "ok", &JsValue::TRUE);
        set(output.as_ref(), "blocked", &JsValue::FALSE);
        set(output.as_ref(), "message", &JsValue::from_str(""));
        set(output.as_ref(), "validation", &default_validation());
        return Ok(output.into());
    }

    let items = bridge_port_orchestration::bridge_collect_configured_ports_r5(
        bridge_instances.clone(),
        active_instance.clone(),
    )?;
    let validation =
        bridge_port_core::bridge_validate_port_conflicts_r5(items, normalized.to_owned());
    Ok(
        bridge_port_core::bridge_apply_port_conflict_start_state_r33(
            normalized.to_owned(),
            validation,
            reason.to_owned(),
        ),
    )
}

fn validate_all(
    bridge_instances: &JsValue,
    active_instance: &JsValue,
    reason: &str,
) -> Result<JsValue, JsValue> {
    let results = Object::new();
    for profile in bridge_frontend_helpers::bridge_network_profiles().iter() {
        let net = crate::js_string_owned(&property(&profile, "key"));
        if net.is_empty() {
            continue;
        }
        let result = validate_and_apply(
            bridge_instances,
            active_instance,
            &net,
            if reason.is_empty() { "all" } else { reason },
        )?;
        set(results.as_ref(), &net, &result);
    }
    Ok(results.into())
}

fn schedule_validation(
    bridge_instances: JsValue,
    active_instance: JsValue,
    net: String,
    reason: String,
) -> Result<(), JsValue> {
    let browser = window();
    if let Some(clear) = function(&browser, "clearTimeout") {
        let _ = clear.call1(&browser, &property(&browser, TIMER_KEY));
    }

    let normalized = net.trim().to_owned();
    let bridge_instances_for_timer = bridge_instances.clone();
    let active_instance_for_timer = active_instance.clone();
    let reason_for_timer = reason.clone();
    let callback = Closure::wrap(Box::new(move || {
        let result = if normalized.is_empty() {
            validate_all(
                &bridge_instances_for_timer,
                &active_instance_for_timer,
                if reason_for_timer.is_empty() {
                    "scheduled-all"
                } else {
                    &reason_for_timer
                },
            )
        } else {
            validate_and_apply(
                &bridge_instances_for_timer,
                &active_instance_for_timer,
                &normalized,
                if reason_for_timer.is_empty() {
                    "scheduled"
                } else {
                    &reason_for_timer
                },
            )
        };
        if let Err(error) = result {
            wasm_bindgen::throw_val(error);
        }
    }) as Box<dyn FnMut()>);

    let Some(set_timeout) = function(&browser, "setTimeout") else {
        return Err(JsValue::from_str("window.setTimeout is not available"));
    };
    let timer = set_timeout.call2(
        &browser,
        callback.as_ref().unchecked_ref(),
        &JsValue::from_f64(60.0),
    )?;
    set(&browser, TIMER_KEY, &timer);
    callback.forget();
    Ok(())
}
#[wasm_bindgen(js_name = bridgeAssertNoPortConflictsR5)]
pub fn bridge_assert_no_port_conflicts_r5(
    net: String,
    structured_reader: JsValue,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> Result<JsValue, JsValue> {
    scoped_start_assertion(
        &net,
        &structured_reader,
        &bridge_instances,
        &active_instance,
    )
}

#[wasm_bindgen(js_name = bridgeValidateAndApplyPortConflictStateR33)]
pub fn bridge_validate_and_apply_port_conflict_state_r33(
    bridge_instances: JsValue,
    active_instance: JsValue,
    net: String,
    reason: String,
) -> Result<JsValue, JsValue> {
    validate_and_apply(&bridge_instances, &active_instance, &net, &reason)
}

#[wasm_bindgen(js_name = bridgeValidateAllPortConflictStatesR33)]
pub fn bridge_validate_all_port_conflict_states_r33(
    bridge_instances: JsValue,
    active_instance: JsValue,
    reason: String,
) -> Result<JsValue, JsValue> {
    validate_all(&bridge_instances, &active_instance, &reason)
}

#[wasm_bindgen(js_name = bridgeSchedulePortConflictValidationR33)]
pub fn bridge_schedule_port_conflict_validation_r33(
    bridge_instances: JsValue,
    active_instance: JsValue,
    net: String,
    reason: String,
) -> Result<(), JsValue> {
    schedule_validation(bridge_instances, active_instance, net, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_port_normalization_matches_start_gate_contract() {
        assert_eq!(normalized_valid_port(" :05555 "), "5555");
        assert_eq!(normalized_valid_port("65535"), "65535");
        assert_eq!(normalized_valid_port("0"), "");
        assert_eq!(normalized_valid_port("65536"), "");
        assert_eq!(normalized_valid_port("localhost:5555"), "");
    }
}
