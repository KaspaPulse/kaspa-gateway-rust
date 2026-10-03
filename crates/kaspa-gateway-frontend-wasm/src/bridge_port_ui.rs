use std::collections::HashSet;

use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

use crate::{
    bridge_frontend_helpers, bridge_port_core, bridge_port_orchestration, bridge_port_validation,
};

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

fn list_values(value: &JsValue) -> Vec<JsValue> {
    let length = property(value, "length").as_f64().unwrap_or(0.0).max(0.0) as u32;
    (0..length)
        .filter_map(|index| Reflect::get(value, &JsValue::from_f64(index as f64)).ok())
        .collect()
}

fn query_all(root: &JsValue, selector: &str) -> Vec<JsValue> {
    call1(root, "querySelectorAll", &JsValue::from_str(selector))
        .map(|value| list_values(&value))
        .unwrap_or_default()
}
fn port_event_dataset_text(target: &JsValue, name: &str) -> String {
    crate::js_string_owned(&property(&property(target, "dataset"), name))
}

fn port_event_net(target: &JsValue) -> String {
    let net = port_event_dataset_text(target, "net");
    if net.is_empty() {
        port_event_dataset_text(target, "network")
    } else {
        net
    }
}

fn port_event_relevant(target: &JsValue) -> bool {
    let hay = [
        crate::js_string_owned(&property(target, "id")),
        crate::js_string_owned(&property(target, "name")),
        port_event_dataset_text(target, "bridgeInstanceField"),
        port_event_dataset_text(target, "bridgeSetting"),
    ]
    .join(" ")
    .to_ascii_lowercase();
    [
        "port",
        "prom",
        "listen",
        "rpc",
        "dashboard",
        "kaspad",
        "instance",
    ]
    .iter()
    .any(|needle| hay.contains(needle))
}

fn schedule_once(delay_ms: f64, callback: Closure<dyn FnMut()>) {
    let win = window();
    if let Some(set_timeout) = function(&win, "setTimeout") {
        let _ = set_timeout.call2(
            &win,
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(delay_ms),
        );
        callback.forget();
    }
}

fn auto_fix_contract(key: &str) -> (&'static str, &'static str) {
    match key {
        "conflictingButton" => ("bridge.autofixPorts.conflictingButton", "Auto Fix Ports"),
        "fixingButton" => ("bridge.autofixPorts.fixingButton", "Fixing Ports..."),
        "fixedButton" => ("bridge.autofixPorts.fixedButton", "Ports Fixed"),
        "failedButton" => ("bridge.autofixPorts.failedButton", "Auto Fix Failed"),
        "disabledButton" => ("bridge.autofixPorts.disabledButton", "Auto Fix Ports"),
        "title" => ("bridge.autofixPorts.title", "Auto Fix Ports"),
        "changedPrefix" => ("bridge.autofixPorts.changedPrefix", "Changed ports"),
        _ => ("bridge.autofixPorts.button", "Auto Fix Ports"),
    }
}

fn auto_fix_text(key: &str) -> String {
    let (i18n_key, fallback) = auto_fix_contract(key);
    let translated =
        bridge_frontend_helpers::bridge_i18n_text_r41(i18n_key.to_owned(), fallback.to_owned());
    let text = crate::js_string_owned(&translated).trim().to_owned();
    if text.is_empty() || text == i18n_key || text.starts_with("bridge.autofixPorts.") {
        fallback.to_owned()
    } else {
        text
    }
}

#[wasm_bindgen(js_name = bridgeAutoFixTextUiR54D3)]
pub fn bridge_auto_fix_text_ui_r54d3(key: String) -> String {
    auto_fix_text(&key)
}
#[wasm_bindgen(js_name = bridgeAutofixButtonInitialLabelUiR111G)]
pub fn bridge_autofix_button_initial_label_ui_r111g(root: JsValue) {
    let scope = if present(&root) { root } else { document() };
    for element in query_all(&scope, "button, [role='button']") {
        if crate::js_string_owned(&property(&element, "textContent")).trim()
            != "bridge.autofixPorts.button"
        {
            continue;
        }
        set(
            &element,
            "textContent",
            &JsValue::from_str("Auto Fix Ports"),
        );
        let _ = call2(
            &element,
            "setAttribute",
            &JsValue::from_str("data-i18n"),
            &JsValue::from_str("bridge.autofixPorts.button"),
        );
        let _ = call2(
            &element,
            "setAttribute",
            &JsValue::from_str("data-kgw-owner"),
            &JsValue::from_str("bridgeInstances"),
        );
    }
}

fn collected_ports(bridge_instances: &JsValue, active_instance: &JsValue) -> Array {
    bridge_port_orchestration::bridge_collect_configured_ports_r5(
        bridge_instances.clone(),
        active_instance.clone(),
    )
    .unwrap_or_else(|_| Array::new())
}
fn validation_for(net: &str, bridge_instances: &JsValue, active_instance: &JsValue) -> JsValue {
    bridge_port_core::bridge_validate_port_conflicts_r5(
        collected_ports(bridge_instances, active_instance),
        net.to_owned(),
    )
}

fn validate_and_apply(
    net: &str,
    reason: &str,
    bridge_instances: &JsValue,
    active_instance: &JsValue,
) -> JsValue {
    bridge_port_validation::bridge_validate_and_apply_port_conflict_state_r33(
        bridge_instances.clone(),
        active_instance.clone(),
        net.to_owned(),
        reason.to_owned(),
    )
    .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
}
#[wasm_bindgen(js_name = bridgeValidateAndApplyPortConflictStateUiR33)]
pub fn bridge_validate_and_apply_port_conflict_state_ui_r33(
    net: String,
    reason: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> JsValue {
    validate_and_apply(net.trim(), &reason, &bridge_instances, &active_instance)
}

#[wasm_bindgen(js_name = bridgeValidateAllPortConflictStatesUiR33)]
pub fn bridge_validate_all_port_conflict_states_ui_r33(
    reason: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> JsValue {
    bridge_port_validation::bridge_validate_all_port_conflict_states_r33(
        bridge_instances,
        active_instance,
        reason,
    )
    .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
}
fn clear_timeout_slot(slot: &str) {
    let win = window();
    let timeout = property(&win, slot);
    if present(&timeout) {
        let _ = call1(&win, "clearTimeout", &timeout);
    }
}

fn schedule_timeout(slot: &'static str, delay_ms: f64, callback: Closure<dyn FnMut()>) {
    let win = window();
    clear_timeout_slot(slot);
    let id = function(&win, "setTimeout")
        .and_then(|set_timeout| {
            set_timeout
                .call2(
                    &win,
                    callback.as_ref().unchecked_ref(),
                    &JsValue::from_f64(delay_ms),
                )
                .ok()
        })
        .unwrap_or(JsValue::UNDEFINED);
    set(&win, slot, &id);
    callback.forget();
}

#[wasm_bindgen(js_name = bridgeSchedulePortConflictValidationUiR33)]
pub fn bridge_schedule_port_conflict_validation_ui_r33(
    net: String,
    reason: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> Result<(), JsValue> {
    bridge_port_validation::bridge_schedule_port_conflict_validation_r33(
        bridge_instances,
        active_instance,
        net,
        reason,
    )
}
fn trace_autofix(net: &str, phase: &str, details: &JsValue) {
    let payload = Object::new();
    set(payload.as_ref(), "patch", &JsValue::from_str("R37"));
    set(
        payload.as_ref(),
        "owner",
        &JsValue::from_str("bridge-existing-port-conflict-owner-autofix"),
    );
    set(
        payload.as_ref(),
        "policy",
        &JsValue::from_str("user-triggered-only-change-actual-conflicts"),
    );
    let safe_details = if details.is_object() && !details.is_null() {
        details.clone()
    } else {
        JsValue::UNDEFINED
    };
    set(payload.as_ref(), "details", &safe_details);
    let _ = bridge_frontend_helpers::bridge_small_owner_trace_r44d(
        JsValue::from_str(net),
        JsValue::from_str("port-autofix"),
        JsValue::from_str(phase),
        payload.into(),
    );
}

#[wasm_bindgen(js_name = bridgeTracePortAutofixUiR37)]
pub fn bridge_trace_port_autofix_ui_r37(net: String, phase: String, details: JsValue) {
    trace_autofix(&net, &phase, &details);
}

fn autofix_buttons() -> Vec<JsValue> {
    let doc = document();
    let root = call1(&doc, "getElementById", &JsValue::from_str("kaspa-bridge"))
        .unwrap_or(JsValue::UNDEFINED);
    if !present(&root) {
        return Vec::new();
    }
    query_all(&root, r#"[data-bridge-action="auto-fix-ports-r37"]"#)
}

#[wasm_bindgen(js_name = bridgeAutofixButtonsUiR37)]
pub fn bridge_autofix_buttons_ui_r37() -> Array {
    let output = Array::new();
    for button in autofix_buttons() {
        output.push(&button);
    }
    output
}

fn refresh_autofix_buttons(bridge_instances: &JsValue, active_instance: &JsValue) {
    for button in autofix_buttons() {
        let dataset = property(&button, "dataset");
        let net = crate::js_string_owned(&property(&dataset, "net"));
        let validation = validation_for(&net, bridge_instances, active_instance);
        let plan = bridge_port_core::bridge_plan_port_autofix_r37(
            net,
            validation.clone(),
            bridge_instances.clone(),
        );
        let changes = property(&plan, "changes");
        let change_count = if Array::is_array(&changes) {
            Array::from(&changes).length()
        } else {
            0
        };
        let enabled = present(&validation)
            && !crate::js_boolean(&property(&validation, "ok"))
            && change_count > 0;
        set(&button, "disabled", &JsValue::from_bool(!enabled));
        let class_list = property(&button, "classList");
        let _ = call2(
            &class_list,
            "toggle",
            &JsValue::from_str("kgw-port-autofix-ready-r37"),
            &JsValue::from_bool(enabled),
        );
        set(
            &dataset,
            "kgwPortAutofixReadyR37",
            &JsValue::from_str(if enabled { "true" } else { "false" }),
        );
        set(
            &button,
            "title",
            &JsValue::from_str(if enabled {
                "Auto-fix conflicting instance ports only. Valid non-conflicting manual ports stay unchanged."
            } else {
                "No auto-fixable instance port conflicts for this network."
            }),
        );
        set(
            &button,
            "textContent",
            &JsValue::from_str(&auto_fix_text(if enabled {
                "conflictingButton"
            } else {
                "button"
            })),
        );
    }
}
#[wasm_bindgen(js_name = bridgeRefreshPortAutofixButtonsUiR37)]
pub fn bridge_refresh_port_autofix_buttons_ui_r37(
    _reason: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) {
    refresh_autofix_buttons(&bridge_instances, &active_instance);
}

#[wasm_bindgen(js_name = bridgeSchedulePortAutofixRefreshUiR37)]
pub fn bridge_schedule_port_autofix_refresh_ui_r37(
    net: String,
    reason: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) {
    let callback = Closure::wrap(Box::new(move || {
        if net.is_empty() {
            let _ = bridge_validate_all_port_conflict_states_ui_r33(
                format!("r37-refresh-all-{}", reason),
                bridge_instances.clone(),
                active_instance.clone(),
            );
        } else {
            let _ = validate_and_apply(
                &net,
                &format!("r37-refresh-{reason}"),
                &bridge_instances,
                &active_instance,
            );
        }
        refresh_autofix_buttons(&bridge_instances, &active_instance);
    }) as Box<dyn FnMut()>);
    schedule_timeout("__kgwBridgePortAutofixRefreshTimerR37", 80.0, callback);
}

fn invoke_callback(callback: &JsValue, argument: &str) {
    if let Some(function) = callback.dyn_ref::<Function>() {
        let _ = function.call1(&JsValue::UNDEFINED, &JsValue::from_str(argument));
    }
}

fn set_runtime_activity(callback: &JsValue, net: &str, message: &str) {
    if let Some(function) = callback.dyn_ref::<Function>() {
        let _ = function.call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str(net),
            &JsValue::from_str(message),
        );
    }
}

#[wasm_bindgen(js_name = bridgeApplyPortAutofixUiR37)]
pub fn bridge_apply_port_autofix_ui_r37(
    active_net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
    refresh_instances: JsValue,
    update_command: JsValue,
    runtime_activity: JsValue,
) -> JsValue {
    let net = active_net.trim().to_owned();
    let validation = validation_for(&net, &bridge_instances, &active_instance);
    let collected = collected_ports(&bridge_instances, &active_instance);
    let begin = Object::new();
    set(begin.as_ref(), "patch2", &JsValue::from_str("R45"));
    set(
        begin.as_ref(),
        "mode",
        &JsValue::from_str("rust-iterative-global-used-ports"),
    );
    set(begin.as_ref(), "activeNet", &JsValue::from_str(&net));
    trace_autofix(&net, "r37-port-autofix-begin", begin.as_ref());

    let result = bridge_port_core::bridge_apply_port_autofix_r37(
        net.clone(),
        validation.clone(),
        bridge_instances.clone(),
        collected.into(),
        8,
    );
    let changes_value = property(&result, "changes");
    let changes = if Array::is_array(&changes_value) {
        Array::from(&changes_value)
    } else {
        Array::new()
    };
    let final_validation = {
        let value = property(&result, "validation");
        if present(&value) { value } else { validation }
    };
    let mut touched = HashSet::<String>::new();
    for change in changes.iter() {
        let change_net = {
            let value = crate::js_string_owned(&property(&change, "net"));
            if value.is_empty() { net.clone() } else { value }
        };
        touched.insert(change_net.clone());
        trace_autofix(&change_net, "r37-port-autofix-change", &change);
    }

    if changes.length() == 0 {
        let details = Object::new();
        set(
            details.as_ref(),
            "reason",
            &JsValue::from_str(if crate::js_boolean(&property(&final_validation, "ok")) {
                "no-conflicts"
            } else {
                "no-instance-conflicts-can-be-autofixed"
            }),
        );
        set(details.as_ref(), "patch2", &JsValue::from_str("R45"));
        trace_autofix(&net, "r37-port-autofix-noop", details.as_ref());
    }

    for touched_net in &touched {
        invoke_callback(&refresh_instances, touched_net);
        invoke_callback(&update_command, touched_net);
        let _ = validate_and_apply(
            touched_net,
            "r45-autofix",
            &bridge_instances,
            &active_instance,
        );
    }
    if !net.is_empty() && !touched.contains(&net) {
        invoke_callback(&update_command, &net);
        let _ = validate_and_apply(
            &net,
            "r45-autofix-active",
            &bridge_instances,
            &active_instance,
        );
    }
    let _ = bridge_validate_all_port_conflict_states_ui_r33(
        "r45-autofix-all".to_owned(),
        bridge_instances.clone(),
        active_instance.clone(),
    );
    refresh_autofix_buttons(&bridge_instances, &active_instance);

    let complete = Object::new();
    set(complete.as_ref(), "patch2", &JsValue::from_str("R45"));
    set(
        complete.as_ref(),
        "changedCount",
        &JsValue::from_f64(changes.length() as f64),
    );
    let final_ok = crate::js_boolean(&property(&final_validation, "ok"));
    set(complete.as_ref(), "finalOk", &JsValue::from_bool(final_ok));
    let conflicts = property(&final_validation, "conflicts");
    set(
        complete.as_ref(),
        "finalConflictCount",
        &JsValue::from_f64(if Array::is_array(&conflicts) {
            Array::from(&conflicts).length() as f64
        } else {
            0.0
        }),
    );
    set(complete.as_ref(), "changes", changes.as_ref());
    trace_autofix(&net, "r37-port-autofix-complete", complete.as_ref());

    let message = format!(
        "{} {} conflicting instance port(s).{}",
        auto_fix_text("changedPrefix"),
        changes.length(),
        if final_ok {
            " Conflicts cleared."
        } else {
            " Some conflicts remain."
        }
    );
    set_runtime_activity(&runtime_activity, &net, &message);

    let output = Object::new();
    set(
        output.as_ref(),
        "changed",
        &JsValue::from_f64(changes.length() as f64),
    );
    set(output.as_ref(), "changes", changes.as_ref());
    set(output.as_ref(), "finalOk", &JsValue::from_bool(final_ok));
    output.into()
}

fn find_action_button(root: &JsValue, net: &str, action: &str) -> JsValue {
    let id_value = bridge_frontend_helpers::bridge_element_id(net.to_owned(), action.to_owned());
    let by_id = bridge_frontend_helpers::bridge_by_id(id_value);
    if present(&by_id) {
        return by_id;
    }
    call1(
        root,
        "querySelector",
        &JsValue::from_str(&format!(
            r#"[data-bridge-action="{action}"][data-net="{net}"]"#
        )),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

#[wasm_bindgen(js_name = bridgeInstallPortAutofixButtonUiR37)]
pub fn bridge_install_port_autofix_button_ui_r37(
    root: JsValue,
    bridge_instances: JsValue,
    active_instance: JsValue,
) {
    if !present(&root) {
        return;
    }
    for old_host in query_all(&root, r#"[data-kgw-bridge-port-autofix-host-r40="true"]"#) {
        let _ = call0(&old_host, "remove");
    }

    for profile in bridge_frontend_helpers::bridge_network_profiles().iter() {
        let net = crate::js_string_owned(&property(&profile, "key"));
        if net.is_empty() {
            continue;
        }
        let selector = format!(r#"[data-bridge-action="auto-fix-ports-r37"][data-net="{net}"]"#);
        let existing = query_all(&root, &selector);
        let mut button = existing.iter().find_map(|item| {
            (crate::js_string_owned(&property(
                &property(item, "dataset"),
                "kgwBridgePortAutofixNextToStopR44",
            )) == "true")
                .then(|| item.clone())
        });

        for item in existing {
            if button
                .as_ref()
                .is_none_or(|current| !Object::is(current, &item))
            {
                let _ = call0(&item, "remove");
            }
        }

        if button.is_none() {
            let created = call1(&document(), "createElement", &JsValue::from_str("button"))
                .unwrap_or(JsValue::UNDEFINED);
            set(&created, "type", &JsValue::from_str("button"));
            let dataset = property(&created, "dataset");
            set(
                &dataset,
                "bridgeAction",
                &JsValue::from_str("auto-fix-ports-r37"),
            );
            set(&dataset, "net", &JsValue::from_str(&net));
            set(
                &dataset,
                "kgwBridgePortAutofixNextToStopR44",
                &JsValue::from_str("true"),
            );
            button = Some(created);
        }
        let button = button.unwrap_or(JsValue::UNDEFINED);
        set(
            &button,
            "className",
            &JsValue::from_str("kgw-bridge-port-autofix-next-to-stop-r44"),
        );
        set(
            &button,
            "textContent",
            &JsValue::from_str(&auto_fix_text("button")),
        );
        set(
            &button,
            "title",
            &JsValue::from_str(&auto_fix_text("title")),
        );

        let stop = find_action_button(&root, &net, "stop");
        let start = find_action_button(&root, &net, "start");
        let anchor = if present(&stop) { stop.clone() } else { start };
        let parent = property(&anchor, "parentNode");
        if present(&anchor) && present(&parent) {
            if !Object::is(&property(&button, "parentNode"), &parent)
                || (present(&stop) && !Object::is(&property(&button, "previousSibling"), &stop))
            {
                let reference = if present(&stop) {
                    property(&stop, "nextSibling")
                } else {
                    property(&anchor, "nextSibling")
                };
                let _ = call2(&parent, "insertBefore", &button, &reference);
            }
        } else if !present(&property(&button, "parentNode")) {
            let _ = call1(&root, "appendChild", &button);
        }
    }
    refresh_autofix_buttons(&bridge_instances, &active_instance);
}

#[wasm_bindgen(js_name = bridgeInstallPortEventOwnersUi)]
pub fn bridge_install_port_event_owners_ui(
    root: JsValue,
    bridge_instances: JsValue,
    active_instance: JsValue,
    callbacks: JsValue,
) -> bool {
    if !present(&root) {
        return false;
    }
    let root_dataset = property(&root, "dataset");

    if port_event_dataset_text(&root, "kgwBridgePortConflictValidationOwnerR33").is_empty() {
        set(
            &root_dataset,
            "kgwBridgePortConflictValidationOwnerR33",
            &JsValue::from_str("1"),
        );

        for (event_name, reason) in [("input", "input"), ("change", "change")] {
            let root_for_event = root.clone();
            let instances_for_event = bridge_instances.clone();
            let active_for_event = active_instance.clone();
            let reason = reason.to_owned();
            let callback = Closure::wrap(Box::new(move |event: JsValue| {
                let target = property(&event, "target");
                if !present(&target)
                    || !call1(&root_for_event, "contains", &target)
                        .is_some_and(|value| crate::js_boolean(&value))
                    || !port_event_relevant(&target)
                {
                    return;
                }
                let net = port_event_net(&target);
                let _ = bridge_schedule_port_conflict_validation_ui_r33(
                    net.clone(),
                    reason.clone(),
                    instances_for_event.clone(),
                    active_for_event.clone(),
                );
                bridge_schedule_port_autofix_refresh_ui_r37(
                    net,
                    reason.clone(),
                    instances_for_event.clone(),
                    active_for_event.clone(),
                );
            }) as Box<dyn FnMut(JsValue)>);
            let _ = call2(
                &root,
                "addEventListener",
                &JsValue::from_str(event_name),
                callback.as_ref().unchecked_ref(),
            );
            callback.forget();
        }

        let instances_install = bridge_instances.clone();
        let active_install = active_instance.clone();
        schedule_once(
            100.0,
            Closure::wrap(Box::new(move || {
                let _ = bridge_validate_all_port_conflict_states_ui_r33(
                    "install".to_owned(),
                    instances_install.clone(),
                    active_install.clone(),
                );
            }) as Box<dyn FnMut()>),
        );
    }

    if port_event_dataset_text(&root, "kgwBridgePortAutofixOwnerR37").is_empty() {
        set(
            &root_dataset,
            "kgwBridgePortAutofixOwnerR37",
            &JsValue::from_str("1"),
        );
        bridge_install_port_autofix_button_ui_r37(
            root.clone(),
            bridge_instances.clone(),
            active_instance.clone(),
        );

        let root_click = root.clone();
        let instances_click = bridge_instances.clone();
        let active_click = active_instance.clone();
        let callbacks_click = callbacks.clone();
        let click = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let button = call1(
                &target,
                "closest",
                &JsValue::from_str("[data-bridge-action=\"auto-fix-ports-r37\"]"),
            )
            .unwrap_or(JsValue::UNDEFINED);
            if !present(&button)
                || !call1(&root_click, "contains", &button)
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
            let net = port_event_dataset_text(&button, "net");
            let result = bridge_apply_port_autofix_ui_r37(
                net.clone(),
                instances_click.clone(),
                active_click.clone(),
                property(&callbacks_click, "refreshInstances"),
                property(&callbacks_click, "updateCommand"),
                property(&callbacks_click, "runtimeActivity"),
            );
            let changed = crate::js_number(&property(&result, "changed")).max(0.0) as u32;
            let feedback = if changed > 0 {
                format!("Fixed {changed} Port(s)")
            } else {
                "No Fix Needed".to_owned()
            };
            set(&button, "textContent", &JsValue::from_str(&feedback));

            let instances_feedback = instances_click.clone();
            let active_feedback = active_click.clone();
            schedule_once(
                1200.0,
                Closure::wrap(Box::new(move || {
                    bridge_refresh_port_autofix_buttons_ui_r37(
                        "button-feedback".to_owned(),
                        instances_feedback.clone(),
                        active_feedback.clone(),
                    );
                }) as Box<dyn FnMut()>),
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &root,
            "addEventListener",
            &JsValue::from_str("click"),
            click.as_ref().unchecked_ref(),
        );
        click.forget();

        let instances_install = bridge_instances;
        let active_install = active_instance;
        schedule_once(
            120.0,
            Closure::wrap(Box::new(move || {
                bridge_refresh_port_autofix_buttons_ui_r37(
                    "install".to_owned(),
                    instances_install.clone(),
                    active_install.clone(),
                );
            }) as Box<dyn FnMut()>),
        );
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autofix_i18n_contract_has_safe_fallbacks() {
        for (key, expected) in [
            ("button", "Auto Fix Ports"),
            ("conflictingButton", "Auto Fix Ports"),
            ("fixingButton", "Fixing Ports..."),
            ("fixedButton", "Ports Fixed"),
            ("failedButton", "Auto Fix Failed"),
            ("disabledButton", "Auto Fix Ports"),
            ("title", "Auto Fix Ports"),
            ("changedPrefix", "Changed ports"),
            ("unknown", "Auto Fix Ports"),
        ] {
            assert_eq!(auto_fix_contract(key).1, expected, "{key}");
        }
    }
}
