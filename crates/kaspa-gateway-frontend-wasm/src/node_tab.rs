use js_sys::{Array, Date, Function, Object, Promise, Reflect, Set};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, future_to_promise, spawn_local};

const LOCK_GLOBAL: &str = "__KGW_BRIDGE_OWNED_NODE_LOCKS_R65E";
const LOCK_EVENT: &str = "kgw-bridge-owned-node-lock-r65e";
const LOCK_SOURCE: &str = "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E";
const LOCK_MESSAGE: &str = "This network is display-only because Bridge in-process mode owns the node runtime. Stop the bridge first.";
const LIVE_REFRESH_MS: f64 = 700.0;

thread_local! {
    static LAST_STATUS: RefCell<BTreeMap<String, String>> = const { RefCell::new(BTreeMap::new()) };
    static LAST_LOGS: RefCell<BTreeMap<String, JsValue>> = const { RefCell::new(BTreeMap::new()) };
    static LAST_ACTIVITY_NOTICE: RefCell<BTreeMap<String, f64>> = const { RefCell::new(BTreeMap::new()) };
    static TRANSITIONS: RefCell<BTreeMap<String, String>> = const { RefCell::new(BTreeMap::new()) };
    static STATUS_IN_FLIGHT: RefCell<BTreeMap<String, Promise>> = const { RefCell::new(BTreeMap::new()) };
    static LOGS_IN_FLIGHT: RefCell<BTreeMap<String, Promise>> = const { RefCell::new(BTreeMap::new()) };
    static ACTIONS_IN_FLIGHT: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
    static LIVE_TIMER: RefCell<JsValue> = const { RefCell::new(JsValue::UNDEFINED) };
    static LIVE_REFRESH_TICKS: RefCell<u64> = const { RefCell::new(0) };
    static LIVE_REFRESH_LAST_MS: RefCell<f64> = const { RefCell::new(0.0) };
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

fn call0(target: &JsValue, name: &str) -> Option<JsValue> {
    function(target, name)?.call0(target).ok()
}

fn call1(target: &JsValue, name: &str, a: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, a).ok()
}

fn call2(target: &JsValue, name: &str, a: &JsValue, b: &JsValue) -> Option<JsValue> {
    function(target, name)?.call2(target, a, b).ok()
}

fn call3(target: &JsValue, name: &str, a: &JsValue, b: &JsValue, c: &JsValue) -> Option<JsValue> {
    function(target, name)?.call3(target, a, b, c).ok()
}

fn text(value: &JsValue) -> String {
    if present(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    }
}

fn bool_value(value: &JsValue) -> bool {
    present(value) && crate::js_boolean(value)
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(target, "querySelectorAll", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED);
    let length = property(&list, "length").as_f64().unwrap_or(0.0).max(0.0) as u32;
    (0..length)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(f64::from(index))).ok())
        .filter(present)
        .collect()
}

fn set_attribute(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn prevent_default(event: &JsValue) {
    let _ = call0(event, "preventDefault");
}

fn stop_propagation(event: &JsValue) {
    let _ = call0(event, "stopPropagation");
}

fn set_style(target: &JsValue, name: &str, value: &str) {
    set(&property(target, "style"), name, &JsValue::from_str(value));
}

fn class_contains(target: &JsValue, class_name: &str) -> bool {
    call1(
        &property(target, "classList"),
        "contains",
        &JsValue::from_str(class_name),
    )
    .is_some_and(|value| bool_value(&value))
}

fn schedule_once<F>(delay_ms: f64, callback: F)
where
    F: FnMut() + 'static,
{
    let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    let _ = call2(
        &window(),
        "setTimeout",
        closure.as_ref(),
        &JsValue::from_f64(delay_ms),
    );
    closure.forget();
}

fn queue_microtask<F>(callback: F)
where
    F: FnMut() + 'static,
{
    let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    if call1(&global(), "queueMicrotask", closure.as_ref()).is_none() {
        let _ = call2(
            &window(),
            "setTimeout",
            closure.as_ref(),
            &JsValue::from_f64(0.0),
        );
    }
    closure.forget();
}

fn transition(net: &str) -> String {
    TRANSITIONS.with(|items| items.borrow().get(net).cloned().unwrap_or_default())
}

fn set_transition(net: &str, value: &str) {
    TRANSITIONS.with(|items| {
        if value.is_empty() {
            items.borrow_mut().remove(net);
        } else {
            items.borrow_mut().insert(net.to_owned(), value.to_owned());
        }
    });
}

fn node_keys() -> Vec<String> {
    crate::node_frontend_helpers::node_r51_keys()
        .iter()
        .map(|value| text(&value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn trace(stage: &str, net: &str, action: &str, result: &str, details: JsValue) {
    let options = Object::new();
    set(options.as_ref(), "network", &JsValue::from_str(net));
    set(options.as_ref(), "action", &JsValue::from_str(action));
    set(options.as_ref(), "result", &JsValue::from_str(result));
    set(options.as_ref(), "details", &details);
    let _ = crate::node_start_trace::node_start_trace_frontend(
        JsValue::from_str(stage),
        options.into(),
    );
}

fn small_trace(net: &str, action: &str, phase: &str, details: JsValue) {
    let _ = crate::node_start_trace::node_small_owner_trace(
        JsValue::from_str(net),
        JsValue::from_str(action),
        JsValue::from_str(phase),
        details,
    );
}

fn explicit_trace(net: &str, action: &str, phase: &str, details: JsValue) {
    let _ = crate::node_start_trace::node_explicit_owner_trace(
        JsValue::from_str(net),
        JsValue::from_str(action),
        JsValue::from_str(phase),
        details,
    );
}

fn root_node() -> JsValue {
    call1(
        &document(),
        "getElementById",
        &JsValue::from_str("kaspa-node"),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

fn lock_store() -> JsValue {
    let win = window();
    let current = property(&win, LOCK_GLOBAL);
    if current.is_object() && !current.is_null() {
        return current;
    }
    let store = Object::new();
    set(&win, LOCK_GLOBAL, store.as_ref());
    store.into()
}

fn bridge_locked(net: &str) -> bool {
    if net.is_empty() {
        return false;
    }
    let item = property(&lock_store(), net);
    bool_value(&property(&item, "locked"))
}

fn dispatch_lock_event(net: &str, locked: bool) {
    let ctor = property(&global(), "CustomEvent");
    let Ok(ctor) = ctor.dyn_into::<Function>() else {
        return;
    };
    let detail = Object::new();
    set(detail.as_ref(), "net", &JsValue::from_str(net));
    set(detail.as_ref(), "locked", &JsValue::from_bool(locked));
    set(detail.as_ref(), "source", &JsValue::from_str(LOCK_SOURCE));
    let options = Object::new();
    set(options.as_ref(), "detail", detail.as_ref());
    let args = Array::new();
    args.push(&JsValue::from_str(LOCK_EVENT));
    args.push(options.as_ref());
    if let Ok(event) = Reflect::construct(&ctor, &args) {
        let _ = call1(&window(), "dispatchEvent", &event);
    }
}

fn set_bridge_lock(net: &str, locked: bool, details: JsValue) {
    if net.is_empty() {
        return;
    }
    let store = lock_store();
    let Ok(store_object) = store.clone().dyn_into::<Object>() else {
        return;
    };
    if locked {
        let item = Object::new();
        set(item.as_ref(), "locked", &JsValue::TRUE);
        set(item.as_ref(), "net", &JsValue::from_str(net));
        set(
            item.as_ref(),
            "reason",
            &JsValue::from_str("bridge-inprocess-owner"),
        );
        set(item.as_ref(), "updatedAt", &JsValue::from_f64(Date::now()));
        set(
            item.as_ref(),
            "details",
            &if details.is_object() {
                details
            } else {
                Object::new().into()
            },
        );
        let _ = Reflect::set(
            store_object.as_ref(),
            &JsValue::from_str(net),
            item.as_ref(),
        );
    } else {
        let _ = Reflect::delete_property(&store_object, &JsValue::from_str(net));
    }
    dispatch_lock_event(net, locked);
}

#[wasm_bindgen(js_name = bridgeSetOwnedNodeLockR65E)]
pub fn bridge_set_owned_node_lock_r65e(net: String, locked: bool, details: JsValue) {
    set_bridge_lock(&net, locked, details);
}

fn settings_callbacks() -> JsValue {
    let callbacks = Object::new();
    let is_locked =
        Closure::wrap(
            Box::new(move |net: JsValue| -> bool { bridge_locked(&text(&net)) })
                as Box<dyn FnMut(JsValue) -> bool>,
        );
    set(callbacks.as_ref(), "isLocked", is_locked.as_ref());
    is_locked.forget();
    callbacks.into()
}

fn render_all_networks(root: &JsValue) -> bool {
    let host = query(root, "#nodeNetworkPanels");
    if !present(&host) {
        return false;
    }
    let Ok(html) = crate::node_frontend_helpers::node_render_network_panels_html() else {
        return false;
    };
    set(&host, "innerHTML", &JsValue::from_str(&html));
    let _ = crate::settings_layout::install_layout(root.clone());
    schedule_once(0.0, || {
        crate::node_frontend_helpers::node_install_log_auto_scroll_controls();
    });
    true
}

fn load_saved_settings() -> Array {
    let applied = crate::node_frontend_helpers::node_r51_load_saved_settings();
    for item in applied.iter() {
        let net = text(&property(&item, "net"));
        if net.is_empty() {
            continue;
        }
        if bool_value(&property(&item, "commandOptionsApplied")) {
            let details = Object::new();
            set(details.as_ref(), "patch", &JsValue::from_str("R38C"));
            set(
                details.as_ref(),
                "owner",
                &JsValue::from_str("node-r51-settings-owner"),
            );
            set(
                details.as_ref(),
                "commandOptionCount",
                &property(&item, "commandOptionsCount"),
            );
            small_trace(
                &net,
                "settings-persistence",
                "r38c-command-options-restored",
                details.into(),
            );
        }
        let _ = crate::node_frontend_helpers::node_update_command(net.clone(), bridge_locked(&net));
    }
    applied
}

fn restore_defaults(net: &str) -> JsValue {
    let begin = Object::new();
    set(begin.as_ref(), "patch", &JsValue::from_str("R29B"));
    set(
        begin.as_ref(),
        "owner",
        &JsValue::from_str("node-r51-settings-owner"),
    );
    small_trace(
        net,
        "restore-defaults",
        "r29b-restore-defaults-begin",
        begin.into(),
    );

    let restored = crate::node_frontend_helpers::node_r51_restore_defaults_action(net.to_owned());
    let loaded = Object::new();
    set(loaded.as_ref(), "patch", &JsValue::from_str("R29B"));
    set(
        loaded.as_ref(),
        "owner",
        &JsValue::from_str("node-r51-settings-owner"),
    );
    set(
        loaded.as_ref(),
        "hasDefaults",
        &property(&restored, "hasDefaults"),
    );
    set(
        loaded.as_ref(),
        "defaultKeyCount",
        &property(&restored, "defaultKeyCount"),
    );
    small_trace(
        net,
        "restore-defaults",
        "r29b-restore-defaults-loaded",
        loaded.into(),
    );

    let net_owned = net.to_owned();
    spawn_local(async move {
        if let Err(error) = crate::node_frontend_helpers::node_apply_root_default_path(
            net_owned.clone(),
            bridge_locked(&net_owned),
        )
        .await
        {
            let message = format!(
                "Rusty Kaspa root-only default path restore failed: {}",
                crate::node_frontend_helpers::node_normalize_runtime_error(error)
            );
            let _ = crate::node_frontend_helpers::node_preview_message(net_owned, message, true);
        }
    });

    let complete = Object::new();
    set(complete.as_ref(), "patch", &JsValue::from_str("R29B"));
    set(
        complete.as_ref(),
        "owner",
        &JsValue::from_str("node-r51-settings-owner"),
    );
    small_trace(
        net,
        "restore-defaults",
        "r29b-restore-defaults-complete",
        complete.into(),
    );
    restored
}

fn runtime_notice(
    net: &str,
    state: &str,
    evidence: &str,
    error: Option<&str>,
    source: Option<&str>,
) {
    let _ = crate::node_frontend_helpers::node_set_runtime_notice(
        net.to_owned(),
        JsValue::from_str(state),
        JsValue::from_str(evidence),
        error.map(JsValue::from_str).unwrap_or(JsValue::NULL),
        source.map(JsValue::from_str).unwrap_or(JsValue::NULL),
    );
}

fn set_runtime_buttons(
    net: &str,
    running: bool,
    bridge_inprocess_locked: bool,
    runtime_error: &str,
    status_text: &str,
) {
    let panel = crate::node_frontend_helpers::node_r51_panel(net.to_owned());
    if !present(&panel) {
        return;
    }

    let display_only_locked = bridge_inprocess_locked || bridge_locked(net);
    let network_enabled = crate::node_frontend_helpers::node_network_enabled(net.to_owned());
    let transition = transition(net);
    let starting = transition == "starting";
    let stopping = transition == "stopping";
    let transition_active = starting || stopping;
    apply_display_only(net, display_only_locked, "runtime-buttons");

    let start = query(
        &panel,
        &format!(r#"[data-node-action="start"][data-net="{net}"]"#),
    );
    let stop = query(
        &panel,
        &format!(r#"[data-node-action="stop"][data-net="{net}"]"#),
    );

    let policy_status = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "policyStatus".to_owned()),
    );
    let state = Object::new();
    set(
        state.as_ref(),
        "enabled",
        &JsValue::from_bool(network_enabled),
    );
    set(state.as_ref(), "running", &JsValue::from_bool(running));
    set(
        state.as_ref(),
        "transition",
        &JsValue::from_str(&transition),
    );
    set(state.as_ref(), "error", &JsValue::from_str(runtime_error));
    let presentation =
        crate::settings_runtime::runtime_presentation(state.into()).unwrap_or(JsValue::UNDEFINED);
    let runtime_state = text(&property(&presentation, "process"));
    let fields =
        crate::node_frontend_helpers::node_parse_runtime_fields(JsValue::from_str(status_text));
    let readiness = crate::settings_runtime::runtime_semantic_readiness(
        &fields,
        running,
        &transition,
        !runtime_error.is_empty(),
        "node",
    )
    .unwrap_or("Awaiting telemetry");

    if present(&policy_status) {
        set(
            &policy_status,
            "textContent",
            &JsValue::from_str(&format!("Node: {runtime_state} | Readiness: {readiness}")),
        );
        set(
            &dataset(&policy_status),
            "state",
            &JsValue::from_str(&runtime_state.to_lowercase()),
        );
        let _ = crate::apply_status_tone(policy_status.clone(), JsValue::from_str(readiness));
    }

    let summary = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "monitorState".to_owned()),
    );
    let observation = crate::settings_runtime::runtime_observation_summary(
        fields.clone(),
        JsValue::from_bool(running),
        JsValue::FALSE,
    )
    .unwrap_or_default();
    let summary_text = format!(
        "{} | Profile: {} | Startup readiness: {} | {}",
        text(&property(&presentation, "processLabel")),
        text(&property(&presentation, "profile")),
        readiness,
        observation
    );
    let _ = crate::render_status_summary(summary, JsValue::from_str(&summary_text));

    let empty = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "logEmpty".to_owned()),
    );
    if present(&empty) {
        let empty_text = if !runtime_error.is_empty() {
            "Node failed. Review the error in Settings and the logs below.".to_owned()
        } else if transition_active {
            format!(
                "Node is {}. Waiting for runtime output.",
                runtime_state.to_lowercase()
            )
        } else if running {
            "Node is running. Waiting for log output.".to_owned()
        } else {
            "Node is stopped. Start the node to view its logs.".to_owned()
        };
        set(&empty, "textContent", &JsValue::from_str(&empty_text));
    }

    let settings_invalid = present(&query(&panel, r#"[aria-invalid="true"]"#))
        || class_contains(
            &crate::node_frontend_helpers::node_by_id(
                crate::node_frontend_helpers::node_element_id(
                    net.to_owned(),
                    "previewStatus".to_owned(),
                ),
            ),
            "kgw-field-error",
        );
    let monitor_start = query(&panel, r#"[data-node-action="monitor-start"]"#);
    if present(&monitor_start) {
        set(
            &monitor_start,
            "hidden",
            &JsValue::from_bool(running || transition_active),
        );
        let label = if display_only_locked {
            "View Node Settings"
        } else if !network_enabled {
            "Enable Profile in Settings"
        } else if settings_invalid {
            "Review Settings"
        } else {
            "Start Node"
        };
        set(&monitor_start, "textContent", &JsValue::from_str(label));
    }

    runtime_notice(
        net,
        &runtime_state,
        if running || starting {
            "Self-worker process owner"
        } else {
            "No process owner"
        },
        None,
        None,
    );

    for field in crate::node_frontend_helpers::node_r51_fields(net.to_owned()).iter() {
        set(&field, "disabled", &JsValue::from_bool(display_only_locked));
        set(&field, "readOnly", &JsValue::from_bool(display_only_locked));
        set(
            &dataset(&field),
            "kgwBridgeInprocessLockedV7",
            &JsValue::from_str(if display_only_locked { "true" } else { "false" }),
        );
        set(
            &field,
            "title",
            &JsValue::from_str(if display_only_locked {
                LOCK_MESSAGE
            } else {
                ""
            }),
        );
    }
    let _ =
        crate::node_frontend_helpers::node_sync_dependencies(net.to_owned(), display_only_locked);

    let preview = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "commandPreview".to_owned()),
    );
    if present(&preview) {
        set(&preview, "readOnly", &JsValue::TRUE);
        set(
            &dataset(&preview),
            "kgwBridgeInprocessLockedV7",
            &JsValue::from_str(if display_only_locked { "true" } else { "false" }),
        );
        set(
            &preview,
            "title",
            &JsValue::from_str(if display_only_locked {
                LOCK_MESSAGE
            } else {
                ""
            }),
        );
    }

    if present(&start) {
        let blocked = running
            || transition_active
            || display_only_locked
            || !network_enabled
            || settings_invalid;
        set(&start, "disabled", &JsValue::from_bool(blocked));
        set_style(&start, "opacity", if blocked { "0.45" } else { "" });
        set_style(&start, "cursor", if blocked { "not-allowed" } else { "" });
        set_attribute(
            &start,
            "aria-disabled",
            if blocked { "true" } else { "false" },
        );
        set(
            &dataset(&start),
            "kgwBridgeInprocessLockedV7",
            &JsValue::from_str(if display_only_locked { "true" } else { "false" }),
        );
        let title = if display_only_locked {
            LOCK_MESSAGE
        } else if !network_enabled {
            "Enable this network before starting it."
        } else if starting {
            "Node is starting."
        } else if stopping {
            "Node is stopping."
        } else if running {
            "Node is running. Stop it before starting again."
        } else {
            "Start node"
        };
        set(&start, "title", &JsValue::from_str(title));
    }

    if present(&stop) {
        let enabled = running && !transition_active && !display_only_locked;
        set(&stop, "disabled", &JsValue::from_bool(!enabled));
        set_style(&stop, "opacity", if enabled { "" } else { "0.45" });
        set_style(&stop, "cursor", if enabled { "" } else { "not-allowed" });
        set_attribute(
            &stop,
            "aria-disabled",
            if enabled { "false" } else { "true" },
        );
        set(
            &dataset(&stop),
            "kgwBridgeInprocessLockedV7",
            &JsValue::from_str(if display_only_locked { "true" } else { "false" }),
        );
        let title = if display_only_locked {
            LOCK_MESSAGE
        } else if starting {
            "Node startup is in progress. Stop becomes available after READY."
        } else if stopping {
            "Node is stopping."
        } else if running {
            "Stop node"
        } else {
            "Node is not running"
        };
        set(&stop, "title", &JsValue::from_str(title));
    }
}

fn set_runtime_unknown(net: &str, message: &str, error_source: &str) {
    let panel = crate::node_frontend_helpers::node_r51_panel(net.to_owned());
    if !present(&panel) {
        return;
    }
    let policy_status = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "policyStatus".to_owned()),
    );
    if present(&policy_status) {
        set(
            &policy_status,
            "textContent",
            &JsValue::from_str("Reconciling"),
        );
        set(
            &dataset(&policy_status),
            "state",
            &JsValue::from_str("reconciling"),
        );
        let _ = crate::apply_status_tone(policy_status, JsValue::from_str("reconciling"));
        let summary = crate::node_frontend_helpers::node_by_id(
            crate::node_frontend_helpers::node_element_id(
                net.to_owned(),
                "monitorState".to_owned(),
            ),
        );
        let _ = crate::render_status_summary(
            summary,
            JsValue::from_str("Node: Reconciling | RPC/synchronization/mining: unknown"),
        );
    }

    for button in [
        query(
            &panel,
            &format!(r#"[data-node-action="start"][data-net="{net}"]"#),
        ),
        query(
            &panel,
            &format!(r#"[data-node-action="stop"][data-net="{net}"]"#),
        ),
    ] {
        if !present(&button) {
            continue;
        }
        set(&button, "disabled", &JsValue::TRUE);
        set_attribute(&button, "aria-disabled", "true");
        set_style(&button, "opacity", "0.45");
        set_style(&button, "cursor", "not-allowed");
        set(&button, "title", &JsValue::from_str(message));
    }

    let current_error = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "runtimeError".to_owned()),
    );
    let preserve = error_source == "status-refresh"
        && !text(&property(&current_error, "textContent"))
            .trim()
            .is_empty()
        && text(&property(&dataset(&current_error), "runtimeErrorSource")) != "status-refresh";
    runtime_notice(
        net,
        "Reconciling",
        "Backend runtime state unavailable",
        if preserve { None } else { Some(message) },
        Some(error_source),
    );
}

fn maybe_activity_notice(net: &str, status_text: &str) {
    let now = Date::now();
    let previous =
        LAST_ACTIVITY_NOTICE.with(|items| items.borrow().get(net).copied().unwrap_or(0.0));
    if now - previous < 15_000.0
        || !crate::node_frontend_helpers::node_runtime_is_running(JsValue::from_str(status_text))
    {
        return;
    }
    LAST_ACTIVITY_NOTICE.with(|items| {
        items.borrow_mut().insert(net.to_owned(), now);
    });
}

fn apply_display_only(net: &str, locked: bool, reason: &str) {
    let panel = crate::node_frontend_helpers::node_r51_panel(net.to_owned());
    if !present(&panel) {
        return;
    }
    set(
        &dataset(&panel),
        "kgwBridgeOwnedNodeDisplayOnlyR65E",
        &JsValue::from_str(if locked { "true" } else { "false" }),
    );
    set_attribute(
        &panel,
        "aria-readonly",
        if locked { "true" } else { "false" },
    );
    set(
        &panel,
        "title",
        &JsValue::from_str(if locked { LOCK_MESSAGE } else { "" }),
    );

    for field in crate::node_frontend_helpers::node_r51_fields(net.to_owned()).iter() {
        set(&field, "disabled", &JsValue::from_bool(locked));
        set(&field, "readOnly", &JsValue::from_bool(locked));
        set(
            &dataset(&field),
            "kgwBridgeOwnedNodeDisplayOnlyR65E",
            &JsValue::from_str(if locked { "true" } else { "false" }),
        );
        set_attribute(
            &field,
            "aria-readonly",
            if locked { "true" } else { "false" },
        );
        set(
            &field,
            "title",
            &JsValue::from_str(if locked { LOCK_MESSAGE } else { "" }),
        );
    }
    let _ = crate::node_frontend_helpers::node_sync_dependencies(net.to_owned(), locked);
    let preview = crate::node_frontend_helpers::node_by_id(
        crate::node_frontend_helpers::node_element_id(net.to_owned(), "commandPreview".to_owned()),
    );
    if present(&preview) {
        set(&preview, "readOnly", &JsValue::TRUE);
        set(
            &dataset(&preview),
            "kgwBridgeOwnedNodeDisplayOnlyR65E",
            &JsValue::from_str(if locked { "true" } else { "false" }),
        );
        set_attribute(&preview, "aria-readonly", "true");
        set(
            &preview,
            "title",
            &JsValue::from_str(if locked { LOCK_MESSAGE } else { "" }),
        );
    }

    for button in query_all(&panel, "[data-node-action]") {
        let action = text(&property(&dataset(&button), "nodeAction"));
        if matches!(
            action.as_str(),
            "start"
                | "stop"
                | "save-settings"
                | "set-defaults"
                | "restore-defaults"
                | "copy-command"
        ) {
            set(&button, "disabled", &JsValue::from_bool(locked));
            set_attribute(
                &button,
                "aria-disabled",
                if locked { "true" } else { "false" },
            );
            set(
                &dataset(&button),
                "kgwBridgeOwnedNodeDisplayOnlyR65E",
                &JsValue::from_str(if locked { "true" } else { "false" }),
            );
            set_style(&button, "opacity", if locked { "0.45" } else { "" });
            set_style(&button, "cursor", if locked { "not-allowed" } else { "" });
            set(
                &button,
                "title",
                &JsValue::from_str(if locked { LOCK_MESSAGE } else { "" }),
            );
        }
    }

    crate::node_settings_owner::node_settings_owner_set_disabled(
        root_node(),
        net.to_owned(),
        locked,
        "runtime-owner-reconcile".to_owned(),
        settings_callbacks(),
    );

    let details = Object::new();
    set(details.as_ref(), "patch", &JsValue::from_str(LOCK_SOURCE));
    set(details.as_ref(), "reason", &JsValue::from_str(reason));
    explicit_trace(
        net,
        "display-only",
        if locked {
            "r65e-node-display-only-enabled"
        } else {
            "r65e-node-display-only-cleared"
        },
        details.into(),
    );
}

async fn bridge_inprocess_locked(net: &str) -> bool {
    let observed = property(&lock_store(), net);
    let resolved = crate::node_start_trace::node_resolve_public_tauri_invoke();
    let Ok(invoke) = property(&resolved, "invoke").dyn_into::<Function>() else {
        return bridge_locked(net);
    };
    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(net));
    set(
        payload.as_ref(),
        "runtimeRole",
        &JsValue::from_str("bridge"),
    );
    let Ok(value) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_runtime_owner_status_v1"),
        payload.as_ref(),
    ) else {
        return bridge_locked(net);
    };
    let Ok(result) = crate::node_start_trace::await_command_with_timeout(
        value,
        "kgw_runtime_owner_status_v1",
        crate::node_start_trace::NODE_RUNTIME_INVOKE_TIMEOUT_MS,
    )
    .await
    else {
        return bridge_locked(net);
    };
    if !Object::is(&property(&lock_store(), net), &observed) {
        return bridge_locked(net);
    }
    let status = crate::node_frontend_helpers::node_stringify_runtime_result(result);
    let fields =
        crate::node_frontend_helpers::node_parse_runtime_fields(JsValue::from_str(&status));
    if text(&property(&fields, "role")) != "bridge"
        || text(&property(&fields, "network")) != net
        || !matches!(
            text(&property(&fields, "running")).as_str(),
            "true" | "false"
        )
    {
        return bridge_locked(net);
    }
    let node_mode = {
        let snake = text(&property(&fields, "node_mode"));
        if snake.is_empty() {
            text(&property(&fields, "nodeMode"))
        } else {
            snake
        }
    }
    .to_lowercase();
    let pid = text(&property(&fields, "pid")).trim().to_owned();
    let locked = text(&property(&fields, "running")) == "true" && node_mode == "inprocess";
    let old_locked = bool_value(&property(&observed, "locked"));
    let old_pid = text(&property(&property(&observed, "details"), "pid"));
    if locked != old_locked || (locked && old_pid != pid) {
        let details = Object::new();
        set(
            details.as_ref(),
            "source",
            &JsValue::from_str("kgw_runtime_owner_status_v1"),
        );
        set(details.as_ref(), "role", &property(&fields, "role"));
        set(details.as_ref(), "nodeMode", &JsValue::from_str(&node_mode));
        set(details.as_ref(), "pid", &JsValue::from_str(&pid));
        set_bridge_lock(net, locked, details.into());
    }
    locked
}

fn logs_task(net: &str) -> Promise {
    if let Some(existing) = LOGS_IN_FLIGHT.with(|items| items.borrow().get(net).cloned()) {
        return existing;
    }
    let net_owned = net.to_owned();
    let promise = future_to_promise(async move {
        if let Ok(report) = crate::node_start_trace::node_invoke_integrated_runtime(
            "kgw_kgw_runtime_logs_v1".to_owned(),
            net_owned.clone(),
        )
        .await
        {
            let _ = crate::node_start_trace::node_apply_runtime_log_report(
                net_owned.clone(),
                "node".to_owned(),
                report.clone(),
            );
            LAST_LOGS.with(|items| {
                items.borrow_mut().insert(net_owned.clone(), report);
            });
        }
        LOGS_IN_FLIGHT.with(|items| {
            items.borrow_mut().remove(&net_owned);
        });
        Ok(JsValue::UNDEFINED)
    });
    LOGS_IN_FLIGHT.with(|items| {
        items.borrow_mut().insert(net.to_owned(), promise.clone());
    });
    promise
}

fn status_task(net: &str) -> Option<Promise> {
    if matches!(transition(net).as_str(), "starting" | "stopping") {
        return None;
    }
    if let Some(existing) = STATUS_IN_FLIGHT.with(|items| items.borrow().get(net).cloned()) {
        return Some(existing);
    }
    let net_owned = net.to_owned();
    let promise = future_to_promise(async move {
        let outcome = crate::node_start_trace::node_invoke_integrated_runtime(
            "kgw_runtime_owner_status_v1".to_owned(),
            net_owned.clone(),
        )
        .await;
        match outcome {
            Ok(raw) => {
                let status = crate::node_frontend_helpers::node_stringify_runtime_result(raw);
                let locked = bridge_inprocess_locked(&net_owned).await;
                let running = crate::node_frontend_helpers::node_runtime_is_running(
                    JsValue::from_str(&status),
                );
                let runtime_error = crate::node_frontend_helpers::node_runtime_error_from_status(
                    JsValue::from_str(&status),
                );
                let fields = crate::node_frontend_helpers::node_parse_runtime_fields(
                    JsValue::from_str(&status),
                );
                let error_node = crate::node_frontend_helpers::node_by_id(
                    crate::node_frontend_helpers::node_element_id(
                        net_owned.clone(),
                        "runtimeError".to_owned(),
                    ),
                );
                if runtime_error.is_empty()
                    && text(&property(&fields, "role")) == "node"
                    && text(&property(&fields, "network")) == net_owned
                    && matches!(
                        text(&property(&fields, "running")).as_str(),
                        "true" | "false"
                    )
                    && text(&property(&dataset(&error_node), "runtimeErrorSource"))
                        == "status-refresh"
                {
                    runtime_notice(
                        &net_owned,
                        if running { "Running" } else { "Stopped" },
                        "",
                        Some(""),
                        None,
                    );
                }
                set_runtime_buttons(&net_owned, running, locked, &runtime_error, &status);
                if !running && !runtime_error.is_empty() {
                    let failed = crate::node_frontend_helpers::node_i18n_text(
                        "runtime.failed".to_owned(),
                        "Failed".to_owned(),
                    );
                    runtime_notice(
                        &net_owned,
                        &text(&failed),
                        "Official runtime terminated after READY",
                        Some(&runtime_error),
                        None,
                    );
                }

                let changed = LAST_STATUS.with(|items| {
                    items
                        .borrow()
                        .get(&net_owned)
                        .is_none_or(|value| value != &status)
                });
                if changed {
                    LAST_STATUS.with(|items| {
                        items.borrow_mut().insert(net_owned.clone(), status.clone());
                    });
                    let authority = crate::node_frontend_helpers::node_by_id(
                        crate::node_frontend_helpers::node_element_id(
                            net_owned.clone(),
                            "settingsAuthority".to_owned(),
                        ),
                    );
                    if present(&authority)
                        && (!running
                            || text(&property(&dataset(&authority), "restartRequired")) != "true")
                    {
                        let value = crate::node_frontend_helpers::node_i18n_text(
                            if running {
                                "runtime.effectiveSettingsActive".to_owned()
                            } else {
                                "runtime.effectiveSettingsNextStart".to_owned()
                            },
                            if running {
                                "Effective settings are active for this runtime".to_owned()
                            } else {
                                "Effective settings apply on next Start".to_owned()
                            },
                        );
                        set(&authority, "textContent", &value);
                        set(
                            &dataset(&authority),
                            "restartRequired",
                            &JsValue::from_str("false"),
                        );
                    }
                }
                maybe_activity_notice(&net_owned, &status);
            }
            Err(error) => {
                let message = format!(
                    "Status refresh failed: {}",
                    crate::node_frontend_helpers::node_normalize_runtime_error(error)
                );
                set_runtime_unknown(&net_owned, &message, "status-refresh");
            }
        }
        STATUS_IN_FLIGHT.with(|items| {
            items.borrow_mut().remove(&net_owned);
        });
        Ok(JsValue::UNDEFINED)
    });
    STATUS_IN_FLIGHT.with(|items| {
        items.borrow_mut().insert(net.to_owned(), promise.clone());
    });
    Some(promise)
}

async fn refresh_one_impl(net: String) {
    let logs = logs_task(&net);
    let status = status_task(&net);
    let _ = JsFuture::from(logs).await;
    if let Some(status) = status {
        let _ = JsFuture::from(status).await;
    }
}

fn hydrate(reason: &str) {
    let store = lock_store();
    let mut keys = BTreeSet::new();
    keys.extend(node_keys());
    if store.is_object()
        && let Ok(store_object) = store.clone().dyn_into::<Object>()
    {
        for key in Object::keys(&store_object).iter() {
            let key = text(&key);
            if !key.is_empty() {
                keys.insert(key);
            }
        }
    }
    for net in keys {
        if bridge_locked(&net) {
            apply_display_only(&net, true, reason);
            set_runtime_buttons(&net, false, true, "", "");
        }
    }
}

fn install_lock_hydration() {
    let win = window();
    if bool_value(&property(
        &win,
        "__KGW_NODE_BRIDGE_OWNED_DISPLAY_ONLY_HYDRATION_R65H2_INSTALLED",
    )) {
        return;
    }
    set(
        &win,
        "__KGW_NODE_BRIDGE_OWNED_DISPLAY_ONLY_HYDRATION_R65H2_INSTALLED",
        &JsValue::TRUE,
    );
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let detail = property(&event, "detail");
        let net = text(&property(&detail, "net"));
        let locked = bool_value(&property(&detail, "locked"));
        if !net.is_empty() {
            apply_display_only(&net, locked, "lock-event");
            set_runtime_buttons(&net, false, locked, "", "");
        }
        schedule_once(0.0, || hydrate("lock-event-late"));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &win,
        "addEventListener",
        &JsValue::from_str(LOCK_EVENT),
        callback.as_ref(),
    );
    callback.forget();
    schedule_once(0.0, || hydrate("module-init"));
}

fn refresh_all(reason: &str) {
    if reason == "poll" {
        LIVE_REFRESH_TICKS.with(|value| {
            let mut ticks = value.borrow_mut();
            *ticks = ticks.saturating_add(1);
        });
        LIVE_REFRESH_LAST_MS.with(|value| *value.borrow_mut() = Date::now());
    }
    hydrate(&format!("{reason}-before-refresh"));
    for net in node_keys() {
        spawn_local(refresh_one_impl(net));
    }
    hydrate(&format!("{reason}-after-refresh"));
}

fn start_live_refresh() {
    LIVE_TIMER.with(|timer| {
        let existing = timer.borrow().clone();
        if present(&existing) {
            let _ = call1(&window(), "clearInterval", &existing);
        }
    });
    refresh_all("initial");
    let callback = Closure::wrap(Box::new(move || refresh_all("poll")) as Box<dyn FnMut()>);
    let timer = call2(
        &window(),
        "setInterval",
        callback.as_ref(),
        &JsValue::from_f64(LIVE_REFRESH_MS),
    )
    .unwrap_or(JsValue::UNDEFINED);
    LIVE_TIMER.with(|value| *value.borrow_mut() = timer);
    callback.forget();
}

fn dangerous_warning(net: &str) -> String {
    let dangerous = crate::settings_contract::node_dangerous();
    let mut lines = Vec::new();
    for entry in Object::entries(&dangerous).iter() {
        let pair = Array::from(&entry);
        let key = text(&pair.get(0));
        if crate::node_frontend_helpers::node_checked(net.to_owned(), key.clone()) {
            lines.push(format!("{key}: {}", text(&pair.get(1))));
        }
    }
    lines.join("\n")
}

fn action_in_flight(key: &str) -> bool {
    ACTIONS_IN_FLIGHT.with(|items| items.borrow().contains(key))
}

fn set_action_in_flight(key: &str, active: bool) {
    ACTIONS_IN_FLIGHT.with(|items| {
        if active {
            items.borrow_mut().insert(key.to_owned());
        } else {
            items.borrow_mut().remove(key);
        }
    });
    let win = window();
    let value = property(&win, "__kgwR29NodeInFlight");
    let set_value = if value.dyn_ref::<Set>().is_some() {
        value
    } else {
        let created = Set::new(&JsValue::UNDEFINED);
        set(&win, "__kgwR29NodeInFlight", created.as_ref());
        created.into()
    };
    if let Ok(js_set) = set_value.dyn_into::<Set>() {
        if active {
            js_set.add(&JsValue::from_str(key));
        } else {
            js_set.delete(&JsValue::from_str(key));
        }
    }
}

async fn run_integrated_action(action: String, net: String) -> bool {
    let command = match action.as_str() {
        "start" => "kgw_kgw_apply_node_settings_v1",
        "stop" => "kgw_kgw_disable_network_v1",
        _ => return false,
    };
    let node_root = root_node();
    let active_network = crate::node_start_trace::node_trace_active_network(node_root);
    let profile = crate::node_frontend_helpers::node_network_profile(net.clone());
    let network_enabled = crate::node_frontend_helpers::node_network_enabled(net.clone());

    let identified = Object::new();
    set(
        identified.as_ref(),
        "commandName",
        &JsValue::from_str(command),
    );
    set(identified.as_ref(), "action", &JsValue::from_str(&action));
    set(
        identified.as_ref(),
        "controlNetwork",
        &JsValue::from_str(&net),
    );
    set(
        identified.as_ref(),
        "activeNetwork",
        &JsValue::from_str(&active_network),
    );
    trace(
        "frontend.start_action_identified",
        &net,
        &action,
        "identified",
        identified.into(),
    );
    let resolved = Object::new();
    set(
        resolved.as_ref(),
        "controlNetwork",
        &JsValue::from_str(&net),
    );
    set(
        resolved.as_ref(),
        "activeNetwork",
        &JsValue::from_str(&active_network),
    );
    set(
        resolved.as_ref(),
        "selectedNetwork",
        &JsValue::from_str(if active_network.is_empty() {
            &net
        } else {
            &active_network
        }),
    );
    trace(
        "frontend.active_network_resolved",
        &net,
        &action,
        if !active_network.is_empty() && active_network != net {
            "mismatch"
        } else {
            "ok"
        },
        resolved.into(),
    );

    if action == "start" {
        let errors = crate::node_frontend_helpers::node_validate_form(net.clone(), true);
        if errors.is_object()
            && let Ok(object) = errors.clone().dyn_into::<Object>()
            && Object::keys(&object).length() > 0
        {
            return true;
        }
        let warnings = dangerous_warning(&net);
        if !warnings.is_empty() {
            let app_dir =
                crate::node_frontend_helpers::node_value(net.clone(), "appDir".to_owned());
            let message = format!(
                "Start {net} with these settings?\n\n{warnings}\n\nData directory: {app_dir}"
            );
            match crate::settings_contract::confirm_user_action(JsValue::from_str(&message)).await {
                Ok(true) => {}
                _ => return true,
            }
        }
        let experimental = bool_value(&property(&profile, "experimental"));
        let explicit_opt_in = experimental && network_enabled;
        let details = Object::new();
        set(
            details.as_ref(),
            "experimentalNetwork",
            &JsValue::from_bool(experimental),
        );
        set(
            details.as_ref(),
            "explicitOptIn",
            &JsValue::from_bool(explicit_opt_in),
        );
        set(
            details.as_ref(),
            "networkEnabled",
            &JsValue::from_bool(network_enabled),
        );
        trace(
            "frontend.experimental_opt_in_evaluated",
            &net,
            &action,
            if !experimental || explicit_opt_in {
                "allowed"
            } else {
                "blocked"
            },
            details.into(),
        );
    }

    if action == "start" && !network_enabled {
        runtime_notice(
            &net,
            "Disabled",
            "No process owner",
            Some("This network is disabled. Enable it in Settings before starting."),
            None,
        );
        set_runtime_buttons(&net, false, false, "", "");
        return true;
    }

    let locked = bridge_locked(&net);
    if locked {
        set_runtime_buttons(&net, false, true, "", "");
        apply_display_only(&net, true, "action-guard");
        runtime_notice(
            &net,
            "Blocked",
            "Bridge in-process owner",
            Some(LOCK_MESSAGE),
            None,
        );
        return true;
    }

    let key = format!("{net}:{action}");
    if action_in_flight(&key) {
        runtime_notice(
            &net,
            if action == "start" {
                "Starting"
            } else {
                "Stopping"
            },
            "Transition in progress",
            Some(&format!(
                "A {action} request is already in progress for this network."
            )),
            None,
        );
        return true;
    }

    set_action_in_flight(&key, true);
    set_transition(
        &net,
        if action == "start" {
            "starting"
        } else {
            "stopping"
        },
    );
    set_runtime_buttons(&net, action == "stop", bridge_locked(&net), "", "");
    runtime_notice(
        &net,
        if action == "start" {
            "Starting"
        } else {
            "Stopping"
        },
        "Waiting for backend response",
        Some(""),
        None,
    );

    let outcome =
        crate::node_start_trace::node_invoke_integrated_runtime(command.to_owned(), net.clone())
            .await;

    match outcome {
        Ok(result) if action == "start" => {
            match crate::node_frontend_helpers::node_assert_start_evidence(net.clone(), result) {
                Ok(evidence) => {
                    set_transition(&net, "");
                    set_runtime_buttons(&net, true, bridge_locked(&net), "", "");
                    runtime_notice(
                        &net,
                        "Running",
                        &format!(
                            "pid={};owner={};role={};readiness=READY",
                            text(&property(&evidence, "pid")),
                            text(&property(&evidence, "owner")),
                            text(&property(&evidence, "role"))
                        ),
                        Some(""),
                        None,
                    );
                }
                Err(error) => {
                    let message = crate::node_frontend_helpers::node_normalize_runtime_error(error);
                    set_transition(&net, "");
                    set_runtime_unknown(&net, &message, "");
                }
            }
        }
        Ok(result) => {
            let evidence = crate::node_frontend_helpers::node_runtime_evidence(result);
            let fields = property(&evidence, "fields");
            let graceful = text(&property(&fields, "graceful")).to_lowercase() == "true";
            let forced = text(&property(&fields, "forced")).to_lowercase() == "true";
            let stop_failed = text(&property(&fields, "stop_failed")) == "true";
            let stopped = text(&property(&fields, "running")) == "false"
                && (graceful
                    || forced
                    || stop_failed
                    || text(&property(&fields, "already_stopped")) == "true");
            if stopped {
                let warning = if forced {
                    format!("Stop required FORCED termination. {}", {
                        let reason = text(&property(&fields, "reason"));
                        if reason.is_empty() {
                            text(&property(&evidence, "text"))
                        } else {
                            reason
                        }
                    })
                } else if stop_failed {
                    format!(
                        "Official graceful shutdown failed, but the worker process exited. {}",
                        {
                            let reason = text(&property(&fields, "reason"));
                            if reason.is_empty() {
                                text(&property(&evidence, "text"))
                            } else {
                                reason
                            }
                        }
                    )
                } else {
                    String::new()
                };
                set_transition(&net, "");
                set_runtime_buttons(&net, false, bridge_locked(&net), "", "");
                runtime_notice(
                    &net,
                    "Stopped",
                    if forced {
                        "FORCED termination confirmed"
                    } else if stop_failed {
                        "Worker exited after graceful shutdown failure"
                    } else if graceful {
                        "Graceful official shutdown confirmed"
                    } else {
                        "Already stopped"
                    },
                    Some(&warning),
                    None,
                );
            } else {
                let message = format!(
                    "Backend Stop did not confirm terminal process exit: {}",
                    text(&property(&evidence, "text"))
                );
                set_transition(&net, "");
                set_runtime_unknown(&net, &message, "");
            }
        }
        Err(error) => {
            set_transition(&net, "");
            let message = crate::node_frontend_helpers::node_normalize_runtime_error(error);
            set_runtime_unknown(&net, &message, "");
            let details = Object::new();
            set(details.as_ref(), "error", &JsValue::from_str(&message));
            set(details.as_ref(), "runningAfterFailure", &JsValue::NULL);
            set(details.as_ref(), "reconciliationRequired", &JsValue::TRUE);
            set(
                details.as_ref(),
                "state",
                &crate::node_start_trace::node_trace_start_button_state(net.clone()),
            );
            trace(
                "frontend.button_state_restored_after_failure",
                &net,
                &action,
                "restored",
                details.into(),
            );
        }
    }

    set_transition(&net, "");
    set_action_in_flight(&key, false);
    let refresh_net = net.clone();
    schedule_once(0.0, move || {
        spawn_local(refresh_one_impl(refresh_net.clone()));
    });
    true
}

fn normalize_net(value: &str) -> String {
    let normalized = value.trim();
    match normalized {
        "mainnet" | "testnet10" | "testnet13" => normalized.to_owned(),
        _ => String::new(),
    }
}

fn first_normalized_net<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    values
        .into_iter()
        .map(normalize_net)
        .find(|net| !net.is_empty())
        .unwrap_or_default()
}

fn net_from_element(element: &JsValue) -> String {
    if !present(element) {
        return String::new();
    }
    let carrier = call1(
        element,
        "closest",
        &JsValue::from_str(
            "[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel], [data-node-section-panel]",
        ),
    )
    .unwrap_or(JsValue::UNDEFINED);
    let mut values = Vec::new();
    for value in [
        property(&dataset(element), "net"),
        property(&dataset(element), "network"),
        property(&dataset(&carrier), "net"),
        property(&dataset(&carrier), "network"),
        property(&dataset(&carrier), "nodeNetworkPanel"),
        property(element, "id"),
        property(&carrier, "id"),
        property(&carrier, "className"),
    ] {
        let value = text(&value);
        if !value.is_empty() {
            values.push(value);
        }
    }
    first_normalized_net(values.iter().map(String::as_str))
}

fn scoped_update(net: &str, reason: &str) {
    if net.is_empty() {
        return;
    }
    let _ = crate::node_frontend_helpers::node_update_command(net.to_owned(), bridge_locked(net));
    let details = Object::new();
    set(
        details.as_ref(),
        "previousPatch",
        &JsValue::from_str("KGW_SETTINGS_SCOPED_NETWORK_BRIDGE_ACTIONS_V26"),
    );
    set(details.as_ref(), "reason", &JsValue::from_str(reason));
    explicit_trace(net, "settings-scope", "r27d-scoped-update", details.into());
}

fn trace_checkbox(toggle: &JsValue, phase: &str, include_checked_after: bool) {
    let details = Object::new();
    set(details.as_ref(), "patch", &JsValue::from_str("R31"));
    set(
        details.as_ref(),
        "owner",
        &JsValue::from_str("node-command-composer-r7"),
    );
    set(
        details.as_ref(),
        "option",
        &property(&dataset(toggle), "nodeCommandOptionToggleR7"),
    );
    if include_checked_after {
        set(
            details.as_ref(),
            "checkedAfter",
            &property(toggle, "checked"),
        );
    } else {
        set(details.as_ref(), "tag", &property(toggle, "tagName"));
        set(details.as_ref(), "type", &property(toggle, "type"));
        set(
            details.as_ref(),
            "checkedBefore",
            &property(toggle, "checked"),
        );
    }
    small_trace(
        &text(&property(&dataset(toggle), "net")),
        "command-checkbox",
        phase,
        details.into(),
    );
}

fn install_command_checkbox_owner(root: &JsValue) {
    let ds = dataset(root);
    if bool_value(&property(&ds, "kgwNodeCommandComposerInlineOwnerR7")) {
        return;
    }
    set(
        &ds,
        "kgwNodeCommandComposerInlineOwnerR7",
        &JsValue::from_str("1"),
    );

    let root_pointer = root.clone();
    let pointer = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let toggle = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-node-command-option-toggle-r7]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&toggle)
            || !call1(&root_pointer, "contains", &toggle).is_some_and(|value| bool_value(&value))
        {
            return;
        }
        trace_checkbox(&toggle, "r31-node-command-checkbox-pointerdown", false);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        root,
        "addEventListener",
        &JsValue::from_str("pointerdown"),
        pointer.as_ref(),
    );
    pointer.forget();

    let root_change = root.clone();
    let change = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let toggle = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-node-command-option-toggle-r7]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&toggle)
            || !call1(&root_change, "contains", &toggle).is_some_and(|value| bool_value(&value))
        {
            return;
        }
        let net = text(&property(&dataset(&toggle), "net"));
        let option = text(&property(&dataset(&toggle), "nodeCommandOptionToggleR7"));
        let enabled = bool_value(&property(&toggle, "checked"));
        let details = Object::new();
        set(details.as_ref(), "patch", &JsValue::from_str("R31"));
        set(
            details.as_ref(),
            "owner",
            &JsValue::from_str("node-command-composer-r7"),
        );
        set(details.as_ref(), "option", &JsValue::from_str(&option));
        set(details.as_ref(), "checked", &JsValue::from_bool(enabled));
        set(details.as_ref(), "trusted", &property(&event, "isTrusted"));
        small_trace(
            &net,
            "command-checkbox",
            "r31-node-command-checkbox-change-begin",
            details.into(),
        );
        let state = crate::node_frontend_helpers::node_command_inline_state(net.clone());
        set(&state, &option, &JsValue::from_bool(enabled));
        let _ = crate::node_frontend_helpers::node_update_command(net.clone(), bridge_locked(&net));
        crate::node_frontend_helpers::node_refresh_inline_command_toggles(net.clone());
        let toggle_after = toggle.clone();
        queue_microtask(move || {
            trace_checkbox(
                &toggle_after,
                "r31-node-command-checkbox-change-after-microtask",
                true,
            );
        });
    }) as Box<dyn FnMut(JsValue)>);

    let _ = call2(
        root,
        "addEventListener",
        &JsValue::from_str("change"),
        change.as_ref(),
    );
    change.forget();

    let root_click = root.clone();
    let click = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let toggle = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-node-command-option-toggle-r7]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&toggle)
            || !call1(&root_click, "contains", &toggle).is_some_and(|value| bool_value(&value))
        {
            return;
        }
        let matches_checkbox = call1(
            &toggle,
            "matches",
            &JsValue::from_str("input[type='checkbox']"),
        )
        .is_some_and(|value| bool_value(&value));
        let net = text(&property(&dataset(&toggle), "net"));
        let option = text(&property(&dataset(&toggle), "nodeCommandOptionToggleR7"));
        let details = Object::new();
        set(details.as_ref(), "patch", &JsValue::from_str("R31"));
        set(
            details.as_ref(),
            "owner",
            &JsValue::from_str("node-command-composer-r7"),
        );
        set(details.as_ref(), "option", &JsValue::from_str(&option));
        set(
            details.as_ref(),
            "isNativeCheckbox",
            &JsValue::from_bool(matches_checkbox),
        );
        set(
            details.as_ref(),
            "checkedAtClick",
            &property(&toggle, "checked"),
        );
        set(details.as_ref(), "trusted", &property(&event, "isTrusted"));
        small_trace(
            &net,
            "command-checkbox",
            "r31-node-command-checkbox-click",
            details.into(),
        );
        if matches_checkbox {
            stop_propagation(&event);
            let toggle_after = toggle.clone();
            queue_microtask(move || {
                trace_checkbox(
                    &toggle_after,
                    "r31-node-command-checkbox-click-after-microtask",
                    true,
                );
            });
            return;
        }
        prevent_default(&event);
        stop_propagation(&event);
        let _ = crate::node_frontend_helpers::node_toggle_command_option_and_update(
            net.clone(),
            option,
            bridge_locked(&net),
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        root,
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref(),
    );
    click.forget();

    let root_key = root.clone();
    let keydown = Closure::wrap(Box::new(move |event: JsValue| {
        let key = text(&property(&event, "key"));
        if key != "Enter" && key != " " {
            return;
        }
        let target = property(&event, "target");
        let toggle = call1(
            &target,
            "closest",
            &JsValue::from_str("[data-node-command-option-toggle-r7]"),
        )
        .unwrap_or(JsValue::UNDEFINED);
        if !present(&toggle)
            || !call1(&root_key, "contains", &toggle).is_some_and(|value| bool_value(&value))
        {
            return;
        }
        prevent_default(&event);
        stop_propagation(&event);
        let net = text(&property(&dataset(&toggle), "net"));
        let option = text(&property(&dataset(&toggle), "nodeCommandOptionToggleR7"));
        let _ = crate::node_frontend_helpers::node_toggle_command_option_and_update(
            net.clone(),
            option,
            bridge_locked(&net),
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        root,
        "addEventListener",
        &JsValue::from_str("keydown"),
        keydown.as_ref(),
    );
    keydown.forget();
}

fn handle_network_change(target: JsValue, net: String, event_trusted: bool) {
    spawn_local(async move {
        let profile = crate::node_frontend_helpers::node_network_profile(net.clone());
        let was_enabled = crate::node_frontend_helpers::node_network_enabled(net.clone());
        let mut enabled = bool_value(&property(&target, "checked"));
        if enabled && bool_value(&property(&profile, "experimental")) {
            set(&target, "checked", &JsValue::FALSE);
            set(&target, "disabled", &JsValue::TRUE);
            let message = "Testnet 13 is experimental and uses a separate non-production runtime. Enable it only for isolated testing. Continue?";
            enabled = crate::settings_contract::confirm_user_action(JsValue::from_str(message))
                .await
                .unwrap_or(false);
            set(&target, "disabled", &JsValue::FALSE);
            set(&target, "checked", &JsValue::from_bool(enabled));
        }
        crate::node_frontend_helpers::node_set_network_enabled(net.clone(), enabled);
        set_runtime_buttons(&net, false, bridge_locked(&net), "", "");
        if !enabled && was_enabled {
            let stop_net = net.clone();
            spawn_local(async move {
                let _ = run_integrated_action("stop".to_owned(), stop_net).await;
            });
        }
        scoped_update(
            &net,
            if event_trusted {
                "trusted-change"
            } else {
                "programmatic-change"
            },
        );
    });
}

fn install_settings_events(root: &JsValue) {
    let input = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let matches = call1(
            &target,
            "matches",
            &JsValue::from_str("input, select, textarea"),
        )
        .is_some_and(|value| bool_value(&value));
        if !matches
            || bool_value(&property(&target, "readOnly"))
            || bool_value(&property(&target, "disabled"))
        {
            return;
        }
        let id = text(&property(&target, "id"));
        if id.ends_with("-commandPreview") || id.ends_with("-logOutput") {
            return;
        }
        let net = net_from_element(&target);
        if !net.is_empty() && bridge_locked(&net) {
            prevent_default(&event);
            stop_propagation(&event);
            apply_display_only(&net, true, "input-guard");
            let details = Object::new();
            set(details.as_ref(), "patch", &JsValue::from_str(LOCK_SOURCE));
            set(details.as_ref(), "targetId", &JsValue::from_str(&id));
            explicit_trace(
                &net,
                "display-only",
                "r65e-node-input-blocked",
                details.into(),
            );
            return;
        }
        let _ = crate::node_frontend_helpers::node_mark_restart_required(net.clone());
        scoped_update(
            &net,
            if bool_value(&property(&event, "isTrusted")) {
                "trusted-input"
            } else {
                "programmatic-input"
            },
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        root,
        "addEventListener",
        &JsValue::from_str("input"),
        input.as_ref(),
        &JsValue::TRUE,
    );
    input.forget();

    let change = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let matches = call1(
            &target,
            "matches",
            &JsValue::from_str("input, select, textarea"),
        )
        .is_some_and(|value| bool_value(&value));
        if !matches
            || bool_value(&property(&target, "readOnly"))
            || bool_value(&property(&target, "disabled"))
        {
            return;
        }
        let id = text(&property(&target, "id"));
        if id.ends_with("-commandPreview") || id.ends_with("-logOutput") {
            return;
        }
        let net = net_from_element(&target);
        if !net.is_empty() && bridge_locked(&net) {
            prevent_default(&event);
            stop_propagation(&event);
            apply_display_only(&net, true, "change-guard");
            let details = Object::new();
            set(details.as_ref(), "patch", &JsValue::from_str(LOCK_SOURCE));
            set(details.as_ref(), "targetId", &JsValue::from_str(&id));
            explicit_trace(
                &net,
                "display-only",
                "r65e-node-change-blocked",
                details.into(),
            );
            return;
        }
        let _ = crate::node_frontend_helpers::node_mark_restart_required(net.clone());
        let network_enabled_toggle = call1(
            &target,
            "matches",
            &JsValue::from_str("[data-node-network-enabled]"),
        )
        .is_some_and(|value| bool_value(&value));
        if network_enabled_toggle {
            handle_network_change(target, net, bool_value(&property(&event, "isTrusted")));
        } else {
            scoped_update(
                &net,
                if bool_value(&property(&event, "isTrusted")) {
                    "trusted-change"
                } else {
                    "programmatic-change"
                },
            );
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        root,
        "addEventListener",
        &JsValue::from_str("change"),
        change.as_ref(),
        &JsValue::TRUE,
    );
    change.forget();
}

fn action_trace(button: &JsValue, event: &JsValue, net: &str, action: &str, locked: bool) {
    if matches!(action, "start" | "stop") {
        return;
    }
    let details = Object::new();
    set(details.as_ref(), "trusted", &property(event, "isTrusted"));
    set(
        details.as_ref(),
        "disabled",
        &JsValue::from_bool(bool_value(&property(button, "disabled")) || locked),
    );
    set(details.as_ref(), "id", &property(button, "id"));
    set(
        details.as_ref(),
        "text",
        &JsValue::from_str(text(&property(button, "textContent")).trim()),
    );
    set(
        details.as_ref(),
        "bridgeOwnedDisplayOnly",
        &JsValue::from_bool(locked),
    );
    explicit_trace(
        net,
        if action.is_empty() { "unknown" } else { action },
        "r27d-action-click",
        details.into(),
    );
}

fn handle_copy_value(action: String, net: String) {
    spawn_local(async move {
        let outcome: Result<(), JsValue> = async {
            let mut value =
                crate::node_frontend_helpers::node_value(net.clone(), "appDir".to_owned());
            if action == "copy-command" {
                let errors = crate::node_frontend_helpers::node_validate_form(net.clone(), false);
                if errors.is_object()
                    && let Ok(object) = errors.clone().dyn_into::<Object>()
                {
                    let values = Object::values(&object);
                    if values.length() > 0 {
                        return Err(values.get(0));
                    }
                }
                let sequence = crate::node_frontend_helpers::node_preview_sequence(net.clone());
                let effective =
                    crate::node_frontend_helpers::node_effective_node_settings(net.clone())?;
                let result =
                    crate::node_frontend_helpers::node_prepare_preview(net.clone(), effective)
                        .await?;
                if crate::node_frontend_helpers::node_preview_sequence(net.clone()) != sequence {
                    return Err(JsValue::from_str(
                        "Settings changed while copying. Try again.",
                    ));
                }
                value = text(&property(&result, "command"));
            }
            if value.is_empty() {
                return Err(JsValue::from_str("There is no validated value to copy."));
            }
            let metadata = Object::new();
            set(
                metadata.as_ref(),
                "characterCount",
                &JsValue::from_f64(value.chars().count() as f64),
            );
            set(
                metadata.as_ref(),
                "lineCount",
                &JsValue::from_f64(value.lines().count().max(1) as f64),
            );
            crate::node_start_trace::node_dispatch_clipboard_write(
                net.clone(),
                value,
                metadata.into(),
            )
            .await?;
            let _ = crate::node_frontend_helpers::node_preview_message(
                net.clone(),
                if action == "copy-path" {
                    "Data directory copied.".to_owned()
                } else {
                    "Command copied.".to_owned()
                },
                false,
            );
            Ok(())
        }
        .await;
        if let Err(error) = outcome {
            let _ = crate::node_frontend_helpers::node_preview_message(
                net,
                format!(
                    "Copy failed: {}",
                    crate::node_frontend_helpers::node_normalize_runtime_error(error)
                ),
                true,
            );
        }
    });
}

fn install_action_clicks(root: &JsValue) {
    let root_click = root.clone();
    let click = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let button = call1(&target, "closest", &JsValue::from_str("[data-node-action]"))
            .unwrap_or(JsValue::UNDEFINED);
        if !present(&button)
            || !call1(&root_click, "contains", &button).is_some_and(|value| bool_value(&value))
        {
            return;
        }
        let action = text(&property(&dataset(&button), "nodeAction"));
        let raw_net = [
            text(&property(&dataset(&button), "net")),
            text(&property(&dataset(&button), "network")),
            net_from_element(&button),
        ]
        .into_iter()
        .find(|value| !value.is_empty())
        .unwrap_or_default();
        let net = normalize_net(&raw_net);
        if net.is_empty() {
            return;
        }
        let locked = bridge_locked(&net);
        action_trace(&button, &event, &net, &action, locked);

        if locked
            && matches!(
                action.as_str(),
                "start"
                    | "stop"
                    | "save-settings"
                    | "set-defaults"
                    | "restore-defaults"
                    | "copy-command"
            )
        {
            prevent_default(&event);
            stop_propagation(&event);
            apply_display_only(&net, true, "click-guard");
            if matches!(action.as_str(), "start" | "stop") {
                runtime_notice(
                    &net,
                    "Blocked",
                    "Bridge in-process owner",
                    Some(LOCK_MESSAGE),
                    None,
                );
            }
            return;
        }

        match action.as_str() {
            "save-settings" => {
                match crate::node_frontend_helpers::node_r51_save_settings(net.clone()) {
                    Ok(_) => set(
                        &dataset(&button),
                        "kgwSettingsActionResult",
                        &JsValue::from_str("success"),
                    ),
                    Err(error) => {
                        set(
                            &dataset(&button),
                            "kgwSettingsActionResult",
                            &JsValue::from_str("failed"),
                        );
                        let message =
                            crate::node_frontend_helpers::node_normalize_runtime_error(error);
                        runtime_notice(&net, "Settings error", "", Some(&message), None);
                    }
                }
                scoped_update(&net, "save-settings");
            }
            "set-defaults" => {
                match crate::node_frontend_helpers::node_r51_set_as_defaults(net.clone()) {
                    Ok(_) => set(
                        &dataset(&button),
                        "kgwSettingsActionResult",
                        &JsValue::from_str("success"),
                    ),
                    Err(error) => {
                        set(
                            &dataset(&button),
                            "kgwSettingsActionResult",
                            &JsValue::from_str("failed"),
                        );
                        let message =
                            crate::node_frontend_helpers::node_normalize_runtime_error(error);
                        runtime_notice(&net, "Settings error", "", Some(&message), None);
                    }
                }
                scoped_update(&net, "set-defaults");
            }
            "restore-defaults" => {
                let _ = restore_defaults(&net);
                set(
                    &dataset(&button),
                    "kgwSettingsActionResult",
                    &JsValue::from_str("success"),
                );
                scoped_update(&net, "restore-defaults");
            }
            "copy-log" | "clear-log" => {
                prevent_default(&event);
                stop_propagation(&event);
                let action_owned = action.clone();
                let net_owned = net.clone();
                let button_owned = button.clone();
                spawn_local(async move {
                    let _ = crate::node_start_trace::node_handle_log_action(
                        action_owned,
                        net_owned,
                        button_owned,
                    )
                    .await;
                });
            }
            "monitor-start" => {
                let _ = crate::node_frontend_helpers::node_panel_start_from_monitor(net);
            }
            "copy-command" | "copy-path" => handle_copy_value(action, net),
            "start" | "stop" => {
                spawn_local(async move {
                    let _ = run_integrated_action(action, net).await;
                });
            }
            _ => {}
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        root,
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref(),
    );
    click.forget();
}

fn install_actions(root: &JsValue) {
    install_command_checkbox_owner(root);
    let _ =
        crate::node_settings_owner::node_install_settings_owner(root.clone(), settings_callbacks());
    install_lock_hydration();
    hydrate("install-actions");
    schedule_once(0.0, || hydrate("install-actions-late"));
    install_settings_events(root);
    install_action_clicks(root);
}

fn network_callbacks() -> JsValue {
    let callbacks = Object::new();

    let is_locked =
        Closure::wrap(
            Box::new(move |net: JsValue| -> bool { bridge_locked(&text(&net)) })
                as Box<dyn FnMut(JsValue) -> bool>,
        );
    set(callbacks.as_ref(), "isLocked", is_locked.as_ref());
    is_locked.forget();

    let apply = Closure::wrap(
        Box::new(move |net: JsValue, locked: JsValue, reason: JsValue| {
            apply_display_only(&text(&net), bool_value(&locked), &text(&reason));
        }) as Box<dyn FnMut(JsValue, JsValue, JsValue)>,
    );
    set(callbacks.as_ref(), "applyDisplayOnly", apply.as_ref());
    apply.forget();

    let buttons = Closure::wrap(
        Box::new(move |net: JsValue, running: JsValue, locked: JsValue| {
            set_runtime_buttons(
                &text(&net),
                bool_value(&running),
                bool_value(&locked),
                "",
                "",
            );
        }) as Box<dyn FnMut(JsValue, JsValue, JsValue)>,
    );
    set(callbacks.as_ref(), "setRuntimeButtons", buttons.as_ref());
    buttons.forget();

    let hydrate_callback = Closure::wrap(Box::new(move |reason: JsValue| {
        hydrate(&text(&reason));
    }) as Box<dyn FnMut(JsValue)>);
    set(callbacks.as_ref(), "hydrate", hydrate_callback.as_ref());
    hydrate_callback.forget();

    callbacks.into()
}

#[wasm_bindgen(js_name = nodeInitKaspaNodeTab)]
pub fn node_init_kaspa_node_tab(root: JsValue) -> bool {
    let node_root = if present(&root) { root } else { root_node() };
    if !present(&node_root) || text(&property(&dataset(&node_root), "kgwNodeV6Ready")) == "true" {
        return false;
    }
    set(
        &dataset(&node_root),
        "kgwNodeV6Ready",
        &JsValue::from_str("true"),
    );

    render_all_networks(&node_root);
    let _ = crate::node_start_trace::node_trace_rendered_start_controls(node_root.clone());
    let _ = crate::node_start_trace::node_install_start_trace_document_click_observer(
        node_root.clone(),
    );

    let resolved = crate::node_start_trace::node_resolve_public_tauri_invoke();
    let active = crate::node_start_trace::node_trace_active_network(node_root.clone());
    let details = Object::new();
    set(details.as_ref(), "adapter", &property(&resolved, "adapter"));
    set(details.as_ref(), "shape", &property(&resolved, "shape"));
    trace(
        "frontend.tauri_invoke_api_availability",
        if active.is_empty() {
            "mainnet"
        } else {
            &active
        },
        "init",
        if property(&resolved, "invoke")
            .dyn_ref::<Function>()
            .is_some()
        {
            "available"
        } else {
            "missing"
        },
        details.into(),
    );

    let _ = crate::node_frontend_helpers::node_r51_capture_factory_defaults();
    let _ = load_saved_settings();
    for net in node_keys() {
        set_runtime_buttons(&net, false, false, "", "");
    }
    let _ = crate::node_frontend_helpers::node_install_network_tabs(
        node_root.clone(),
        network_callbacks(),
    );
    let _ = crate::node_frontend_helpers::node_install_delegated_tabs(node_root.clone());
    install_actions(&node_root);
    for net in node_keys() {
        let _ = crate::node_frontend_helpers::node_update_command(net.clone(), bridge_locked(&net));
        let net_for_path = net.clone();
        spawn_local(async move {
            if let Err(error) = crate::node_frontend_helpers::node_apply_root_default_path(
                net_for_path.clone(),
                bridge_locked(&net_for_path),
            )
            .await
            {
                let _ = crate::node_frontend_helpers::node_preview_message(
                    net_for_path,
                    format!(
                        "Rusty Kaspa root-only default path restore failed: {}",
                        crate::node_frontend_helpers::node_normalize_runtime_error(error)
                    ),
                    true,
                );
            }
        });
    }
    start_live_refresh();
    schedule_once(0.0, || {
        crate::node_frontend_helpers::node_install_log_auto_scroll_controls();
    });
    true
}

#[wasm_bindgen(js_name = nodeSetRuntimeButtons)]
pub fn node_set_runtime_buttons(
    net: String,
    running: bool,
    bridge_inprocess_locked: bool,
    runtime_error: String,
    status_text: String,
) {
    set_runtime_buttons(
        &net,
        running,
        bridge_inprocess_locked,
        &runtime_error,
        &status_text,
    );
}

#[wasm_bindgen(js_name = nodeLiveRefreshDiagnostics)]
pub fn node_live_refresh_diagnostics() -> JsValue {
    let output = Object::new();
    let timer_present = LIVE_TIMER.with(|value| present(&value.borrow()));
    let ticks = LIVE_REFRESH_TICKS.with(|value| *value.borrow());
    let last_tick_ms = LIVE_REFRESH_LAST_MS.with(|value| *value.borrow());
    let status_in_flight = STATUS_IN_FLIGHT.with(|items| items.borrow().len());
    let logs_in_flight = LOGS_IN_FLIGHT.with(|items| items.borrow().len());
    set(
        output.as_ref(),
        "timerPresent",
        &JsValue::from_bool(timer_present),
    );
    set(output.as_ref(), "ticks", &JsValue::from_f64(ticks as f64));
    set(
        output.as_ref(),
        "lastTickMs",
        &JsValue::from_f64(last_tick_ms),
    );
    set(
        output.as_ref(),
        "statusInFlight",
        &JsValue::from_f64(status_in_flight as f64),
    );
    set(
        output.as_ref(),
        "logsInFlight",
        &JsValue::from_f64(logs_in_flight as f64),
    );
    output.into()
}

#[wasm_bindgen(js_name = nodeRefreshOne)]
pub async fn node_refresh_one(net: String, _reason: String) {
    refresh_one_impl(net).await;
}

#[wasm_bindgen(js_name = nodeRunIntegratedAction)]
pub async fn node_run_integrated_action(action: String, net: String) -> bool {
    run_integrated_action(action, net).await
}

#[wasm_bindgen(js_name = nodeSetRuntimeTransition)]
pub fn node_set_runtime_transition(net: String, value: String) {
    set_transition(&net, &value);
}

#[wasm_bindgen(js_name = nodeBridgeOwnedLocked)]
pub fn node_bridge_owned_locked(net: String) -> bool {
    bridge_locked(&net)
}

#[wasm_bindgen(js_name = nodeApplyBridgeDisplayOnly)]
pub fn node_apply_bridge_display_only(net: String, locked: bool, reason: String) {
    apply_display_only(&net, locked, &reason);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_preserve_live_refresh_contract() {
        assert_eq!(LIVE_REFRESH_MS, 700.0);
        assert_eq!(LOCK_EVENT, "kgw-bridge-owned-node-lock-r65e");
        assert_eq!(LOCK_SOURCE, "KGW_BRIDGE_OWNED_NODE_DISPLAY_ONLY_LOCK_R65E");
    }

    #[test]
    fn network_resolution_returns_first_exact_candidate_without_context_concatenation() {
        assert_eq!(
            first_normalized_net(["", "mainnet", "mainnet-node-settings node-command-option",]),
            "mainnet"
        );
        assert_eq!(
            first_normalized_net([
                "not-a-network",
                " testnet10 ",
                "node-testnet10-commandSettings",
            ]),
            "testnet10"
        );
        assert_eq!(
            first_normalized_net([
                "kgw-node-panel testnet13 extra-noise",
                "testnet13",
                "ignored-after-valid",
            ]),
            "testnet13"
        );
    }

    #[test]
    fn network_resolution_fails_closed_when_only_noisy_context_exists() {
        assert_eq!(
            first_normalized_net([
                "node-mainnet-commandSettings",
                "settings-panel testnet10",
                "node-testnet13-listenHost",
            ]),
            ""
        );
        assert_eq!(normalize_net(" mainnet "), "mainnet");
        assert_eq!(normalize_net("mainnet extra"), "");
    }
}
