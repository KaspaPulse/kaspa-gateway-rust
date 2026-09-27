use js_sys::{Array, Function, JSON, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

const COMMAND: &str = "kgw_start_trace_frontend_v1";
const REDACTED: &str = "[redacted]";
const BLOCKED_KEYS: &[&str] = &[
    "secret",
    "token",
    "private",
    "mnemonic",
    "wallet",
    "address",
    "commandpreview",
    "completecommand",
    "arguments",
    "appdir",
    "path",
    "rpcendpoint",
    "stratum",
];

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
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

fn safe_text_str(value: &str, fallback: &str) -> String {
    let mut text = value.replace(['\r', '\n', '\t'], " ").trim().to_owned();
    if text.is_empty() {
        text = fallback.to_owned();
    }
    text.chars().take(220).collect()
}

fn safe_text(value: &JsValue, fallback: &str) -> String {
    safe_text_str(&crate::js_string_owned(value), fallback)
}

fn blocked_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    BLOCKED_KEYS.iter().any(|needle| lower.contains(needle))
}

fn safe_details(value: &JsValue) -> JsValue {
    let source = if value.is_object() && !Array::is_array(value) {
        value.clone()
    } else {
        Object::new().into()
    };
    let output = Object::new();
    for entry in Object::entries(&Object::from(source)).iter() {
        let pair = Array::from(&entry);
        if pair.length() < 2 {
            continue;
        }
        let key = crate::js_string_owned(&pair.get(0));
        let value = pair.get(1);
        let sanitized = if blocked_key(&key) {
            JsValue::from_str(REDACTED)
        } else if Array::is_array(&value) {
            let source = Array::from(&value);
            let result = Array::new();
            for item in source.iter().take(24) {
                result.push(&JsValue::from_str(&safe_text(&item, "")));
            }
            result.into()
        } else if value.is_object() && !value.is_null() {
            safe_details(&value)
        } else if value.as_bool().is_some() || value.as_f64().is_some() {
            value
        } else {
            JsValue::from_str(&safe_text(&value, ""))
        };
        set(output.as_ref(), &key, &sanitized);
    }
    output.into()
}

fn object_keys(value: &JsValue) -> JsValue {
    let output = Array::new();
    if !value.is_object() || value.is_null() {
        return output.into();
    }
    let mut keys = Object::keys(&Object::from(value.clone()))
        .iter()
        .map(|value| crate::js_string_owned(&value))
        .collect::<Vec<_>>();
    keys.sort();
    keys.truncate(24);
    for key in keys {
        output.push(&JsValue::from_str(&key));
    }
    output.into()
}

fn tauri_shape(adapter: &str) -> JsValue {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let legacy = property(&tauri, "tauri");
    let output = Object::new();
    set(
        output.as_ref(),
        "adapter",
        &JsValue::from_str(&safe_text(&JsValue::from_str(adapter), "missing")),
    );
    set(
        output.as_ref(),
        "hasGlobalTauri",
        &JsValue::from_bool(present(&tauri)),
    );
    set(output.as_ref(), "globalKeys", &object_keys(&tauri));
    set(output.as_ref(), "coreKeys", &object_keys(&core));
    set(output.as_ref(), "tauriKeys", &object_keys(&legacy));
    set(
        output.as_ref(),
        "hasCoreInvoke",
        &JsValue::from_bool(property(&core, "invoke").dyn_ref::<Function>().is_some()),
    );
    set(
        output.as_ref(),
        "hasTauriInvoke",
        &JsValue::from_bool(property(&legacy, "invoke").dyn_ref::<Function>().is_some()),
    );
    set(
        output.as_ref(),
        "hasRootInvoke",
        &JsValue::from_bool(property(&tauri, "invoke").dyn_ref::<Function>().is_some()),
    );
    set(
        output.as_ref(),
        "expectedConfiguredGlobal",
        &JsValue::from_str("window.__TAURI__.core.invoke"),
    );
    output.into()
}
fn bound_invoke(owner: &JsValue, invoke: JsValue) -> Option<Function> {
    let invoke = invoke.dyn_into::<Function>().ok()?;
    let bind = function(invoke.as_ref(), "bind")?;
    bind.call1(invoke.as_ref(), owner)
        .ok()?
        .dyn_into::<Function>()
        .ok()
}

fn clipboard_character_count_text(text: &str) -> usize {
    text.chars().count()
}

fn clipboard_line_count_text(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.split('\n').count()
    }
}

fn normalize_clipboard_line_endings_text(text: &str) -> String {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}

fn clipboard_safe_error_text(text: &str) -> String {
    let normalized = text.replace(['\r', '\n', '\t'], " ").trim().to_owned();
    let source = if normalized.is_empty() {
        "clipboard write failed"
    } else {
        normalized.as_str()
    };
    let lower = source.to_ascii_lowercase();
    if [
        "secret", "token", "private", "mnemonic", "wallet", "address",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return "clipboard write failed with a sensitive error".to_owned();
    }
    source.chars().take(360).collect()
}

fn clipboard_placeholder_text(net: &str) -> String {
    let label = match net {
        "mainnet" => "Mainnet",
        "testnet10" => "Testnet 10",
        "testnet13" => "Testnet 13",
        other => other,
    };
    format!("{label} log is empty.")
}

fn document() -> JsValue {
    property(&global(), "document")
}

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, arg).ok()
}

fn call3(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
    third: &JsValue,
) -> Option<JsValue> {
    function(target, name)?
        .call3(target, first, second, third)
        .ok()
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(target, "querySelectorAll", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED);
    if !present(&list) {
        return Vec::new();
    }
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(present)
        })
        .collect()
}

fn closest(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "closest", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn contains(root: &JsValue, node: &JsValue) -> bool {
    call1(root, "contains", node).is_some_and(|value| crate::js_boolean(&value))
}

fn class_contains(target: &JsValue, class_name: &str) -> bool {
    let class_list = property(target, "classList");
    call1(&class_list, "contains", &JsValue::from_str(class_name))
        .is_some_and(|value| crate::js_boolean(&value))
}

fn attribute(target: &JsValue, name: &str) -> String {
    call1(target, "getAttribute", &JsValue::from_str(name))
        .filter(present)
        .map(|value| crate::js_string_owned(&value))
        .unwrap_or_default()
}

fn dataset_value(target: &JsValue, name: &str) -> String {
    crate::js_string_owned(&property(&property(target, "dataset"), name))
}

fn node_root(root: &JsValue) -> JsValue {
    if present(root) {
        root.clone()
    } else {
        call1(
            &document(),
            "getElementById",
            &JsValue::from_str("kaspa-node"),
        )
        .unwrap_or(JsValue::UNDEFINED)
    }
}

fn network_from_trace_text(raw: &str) -> Option<&'static str> {
    let raw = raw.to_ascii_lowercase();
    if raw.contains("testnet13") || raw.contains("tn13") {
        Some("testnet13")
    } else if raw.contains("testnet10") || raw.contains("tn10") {
        Some("testnet10")
    } else if raw.contains("mainnet") {
        Some("mainnet")
    } else {
        None
    }
}

fn trace_active_network(root: &JsValue) -> String {
    let root = node_root(root);
    if !present(&root) {
        return String::new();
    }

    for panel in query_all(&root, "[data-node-network-panel]") {
        let active = !crate::js_boolean(&property(&panel, "hidden"))
            && (class_contains(&panel, "active") || dataset_value(&panel, "active") == "true");
        if active {
            let value = dataset_value(&panel, "nodeNetworkPanel");
            if !value.is_empty() {
                return value;
            }
        }
    }

    for tab in query_all(&root, "[data-node-network-tab]") {
        let active = class_contains(&tab, "active")
            || attribute(&tab, "aria-selected") == "true"
            || dataset_value(&tab, "active") == "true";
        if active {
            let value = dataset_value(&tab, "nodeNetworkTab");
            if !value.is_empty() {
                return value;
            }
        }
    }
    String::new()
}

fn trace_network_from_element(element: &JsValue, root: &JsValue) -> String {
    let carrier = closest(
        element,
        "[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel]",
    );
    let fields = [
        dataset_value(element, "net"),
        dataset_value(element, "network"),
        dataset_value(&carrier, "net"),
        dataset_value(&carrier, "network"),
        dataset_value(&carrier, "nodeNetworkPanel"),
        crate::js_string_owned(&property(&carrier, "id")),
        crate::js_string_owned(&property(&carrier, "className")),
    ];
    let raw = fields
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    network_from_trace_text(&raw)
        .map(str::to_owned)
        .or_else(|| {
            let active = trace_active_network(root);
            (!active.is_empty()).then_some(active)
        })
        .unwrap_or_else(|| "mainnet".to_owned())
}

fn trace_start_button_state(net: &str) -> JsValue {
    let panel = query(&document(), &format!("[data-node-network-panel=\"{net}\"]"));
    let start = query(
        &panel,
        &format!("[data-node-action=\"start\"][data-net=\"{net}\"]"),
    );
    let stop = query(
        &panel,
        &format!("[data-node-action=\"stop\"][data-net=\"{net}\"]"),
    );
    let output = Object::new();
    set(
        output.as_ref(),
        "startRendered",
        &JsValue::from_bool(present(&start)),
    );
    set(
        output.as_ref(),
        "startDisabled",
        &JsValue::from_bool(present(&start) && crate::js_boolean(&property(&start, "disabled"))),
    );
    set(
        output.as_ref(),
        "stopRendered",
        &JsValue::from_bool(present(&stop)),
    );
    set(
        output.as_ref(),
        "stopDisabled",
        &JsValue::from_bool(present(&stop) && crate::js_boolean(&property(&stop, "disabled"))),
    );
    output.into()
}

fn emit_start_trace(stage: &str, network: &str, action: &str, result: &str, details: JsValue) {
    let options = Object::new();
    set(options.as_ref(), "network", &JsValue::from_str(network));
    set(options.as_ref(), "action", &JsValue::from_str(action));
    set(options.as_ref(), "result", &JsValue::from_str(result));
    set(options.as_ref(), "details", &details);
    let _ = node_start_trace_frontend(JsValue::from_str(stage), options.into());
}

fn install_start_trace_document_click_observer(root: &JsValue) -> bool {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwStartTraceDocumentClickObserverR1")) {
        return false;
    }
    set(
        &win,
        "__kgwStartTraceDocumentClickObserverR1",
        &JsValue::TRUE,
    );

    let root_for_click = root.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let button = closest(&target, "[data-node-action]");
        let node_root = node_root(&root_for_click);
        if !present(&button) || !present(&node_root) || !contains(&node_root, &button) {
            return;
        }

        let action = dataset_value(&button, "nodeAction").trim().to_owned();
        let network = trace_network_from_element(&button, &node_root);
        let active_network = trace_active_network(&node_root);
        let belongs_to_settings =
            present(&closest(&button, "[data-node-inner-panel=\"settings\"]"));
        let belongs_to_log = present(&closest(&button, "[data-node-inner-panel=\"log\"]"));

        if action == "copy-log" {
            let details = Object::new();
            set(
                details.as_ref(),
                "trusted",
                &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
            );
            set(
                details.as_ref(),
                "belongsToSettings",
                &JsValue::from_bool(belongs_to_settings),
            );
            set(
                details.as_ref(),
                "belongsToLiveNodeMonitor",
                &JsValue::from_bool(belongs_to_log),
            );
            set(
                details.as_ref(),
                "selectedNetwork",
                &JsValue::from_str(&active_network),
            );
            set(
                details.as_ref(),
                "buttonDisabled",
                &JsValue::from_bool(crate::js_boolean(&property(&button, "disabled"))),
            );
            set(
                details.as_ref(),
                "inFlight",
                &JsValue::from_bool(dataset_value(&button, "kgwCopyLogInFlightV1") == "1"),
            );
            emit_start_trace(
                "frontend.copy_log_click_observed",
                &network,
                "copy-log",
                "observed",
                details.into(),
            );
            return;
        }

        if action != "start" && action != "stop" {
            return;
        }
        let details = Object::new();
        set(
            details.as_ref(),
            "trusted",
            &JsValue::from_bool(crate::js_boolean(&property(&event, "isTrusted"))),
        );
        set(
            details.as_ref(),
            "belongsToSettings",
            &JsValue::from_bool(belongs_to_settings),
        );
        set(
            details.as_ref(),
            "belongsToLiveNodeMonitor",
            &JsValue::from_bool(belongs_to_log),
        );
        set(
            details.as_ref(),
            "selectedNetwork",
            &JsValue::from_str(&active_network),
        );
        set(
            details.as_ref(),
            "buttonDisabled",
            &JsValue::from_bool(crate::js_boolean(&property(&button, "disabled"))),
        );
        set(
            details.as_ref(),
            "buttonAction",
            &JsValue::from_str(&action),
        );
        emit_start_trace(
            "frontend.capture_click_observed",
            &network,
            &action,
            "observed",
            details.into(),
        );
    }) as Box<dyn FnMut(JsValue)>);

    let _ = call3(
        &document(),
        "addEventListener",
        &JsValue::from_str("click"),
        callback.as_ref(),
        &JsValue::TRUE,
    );
    callback.forget();
    true
}

fn trace_rendered_start_controls(root: &JsValue) -> bool {
    let root = node_root(root);
    if !present(&root) {
        return false;
    }
    for net in ["mainnet", "testnet10", "testnet13"] {
        let settings_panel = query(
            &root,
            &format!("[data-node-network-panel=\"{net}\"] [data-node-inner-panel=\"settings\"]"),
        );
        let start = query(
            &settings_panel,
            &format!("[data-node-action=\"start\"][data-net=\"{net}\"]"),
        );
        let render_details = Object::new();
        set(
            render_details.as_ref(),
            "belongsToSettings",
            &JsValue::from_bool(present(&settings_panel)),
        );
        set(
            render_details.as_ref(),
            "selectedNetwork",
            &JsValue::from_str(&trace_active_network(&root)),
        );
        emit_start_trace(
            "frontend.settings_subtab_rendered",
            net,
            "render",
            if present(&settings_panel) {
                "ok"
            } else {
                "missing"
            },
            render_details.into(),
        );

        let start_details = Object::new();
        set(
            start_details.as_ref(),
            "belongsToSettings",
            &JsValue::from_bool(
                present(&start) && present(&settings_panel) && contains(&settings_panel, &start),
            ),
        );
        set(
            start_details.as_ref(),
            "startDisabled",
            &JsValue::from_bool(
                present(&start) && crate::js_boolean(&property(&start, "disabled")),
            ),
        );
        emit_start_trace(
            "frontend.start_control_rendered",
            net,
            "start",
            if present(&start) { "ok" } else { "missing" },
            start_details.into(),
        );
    }
    true
}

fn runtime_action_for_command_text(command: &str) -> &'static str {
    match command {
        "kgw_kgw_apply_node_settings_v1" => "start",
        "kgw_kgw_disable_network_v1" => "stop",
        _ => "runtime",
    }
}

fn truthy_text_or(value: &JsValue, fallback: &str) -> String {
    if crate::js_boolean(value) {
        crate::js_string_owned(value)
    } else {
        fallback.to_owned()
    }
}

fn small_owner_invoke() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    if let Some(invoke) = bound_invoke(&core, property(&core, "invoke")) {
        return Some(invoke);
    }
    if let Some(invoke) = bound_invoke(&tauri, property(&tauri, "invoke")) {
        return Some(invoke);
    }
    property(&window(), "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .ok()
}

fn small_owner_trace(net: JsValue, action: JsValue, phase: JsValue, details: JsValue) -> bool {
    let safe_net = truthy_text_or(&net, "unknown");
    let safe_action = truthy_text_or(&action, "small-owner");
    let safe_phase = truthy_text_or(&phase, "unknown");
    let safe_details = if details.is_object() {
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
        &JsValue::from_str("node-small-owner-functions"),
    );
    set(nested.as_ref(), "network", &JsValue::from_str(&safe_net));
    set(nested.as_ref(), "action", &JsValue::from_str(&safe_action));
    set(nested.as_ref(), "phase", &JsValue::from_str(&safe_phase));
    set(nested.as_ref(), "details", &safe_details);

    let args = Object::new();
    set(args.as_ref(), "scope", &JsValue::from_str("node"));
    set(args.as_ref(), "net", &JsValue::from_str(&safe_net));
    set(args.as_ref(), "action", &JsValue::from_str(&safe_action));
    set(args.as_ref(), "phase", &JsValue::from_str(&safe_phase));
    let serialized = JSON::stringify(nested.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    set(args.as_ref(), "details", &JsValue::from_str(&serialized));

    let Some(invoke) = small_owner_invoke() else {
        return false;
    };
    if let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }
    true
}

fn resolved_invoke() -> JsValue {
    let tauri = property(&window(), "__TAURI__");
    for (adapter, owner, candidate) in [
        (
            "window.__TAURI__.core.invoke",
            property(&tauri, "core"),
            property(&property(&tauri, "core"), "invoke"),
        ),
        (
            "window.__TAURI__.tauri.invoke",
            property(&tauri, "tauri"),
            property(&property(&tauri, "tauri"), "invoke"),
        ),
        (
            "window.__TAURI__.invoke",
            tauri.clone(),
            property(&tauri, "invoke"),
        ),
    ] {
        if let Some(invoke) = bound_invoke(&owner, candidate) {
            let output = Object::new();
            set(output.as_ref(), "adapter", &JsValue::from_str(adapter));
            set(output.as_ref(), "invoke", invoke.as_ref());
            set(output.as_ref(), "shape", &tauri_shape(adapter));
            return output.into();
        }
    }
    let output = Object::new();
    set(output.as_ref(), "adapter", &JsValue::from_str("missing"));
    set(output.as_ref(), "invoke", &JsValue::NULL);
    set(output.as_ref(), "shape", &tauri_shape("missing"));
    output.into()
}
#[derive(Clone)]
struct NodeClipboardBuffer {
    out: JsValue,
    normalized_text: String,
    is_placeholder: bool,
    character_count: u32,
    line_count: u32,
}

fn copy_log_call2(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Option<JsValue> {
    function(target, name)?.call2(target, first, second).ok()
}

fn copy_log_set_attr(target: &JsValue, name: &str, value: &str) {
    let _ = copy_log_call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn copy_log_output(net: &str) -> JsValue {
    call1(
        &document(),
        "getElementById",
        &JsValue::from_str(&format!("node-{net}-logOutput")),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

fn copy_log_status_element(net: &str) -> JsValue {
    let out = copy_log_output(net);
    if !present(&out) {
        return JsValue::UNDEFINED;
    }
    let panel = closest(&out, "[data-node-inner-panel=\"log\"]");
    if !present(&panel) {
        return JsValue::UNDEFINED;
    }
    let toolbar = query(&panel, ".node-v6-log-toolbar");
    if !present(&toolbar) {
        return JsValue::UNDEFINED;
    }

    let selector = format!(".kgw-copy-log-status-v1[data-net=\"{net}\"]");
    let existing = query(&toolbar, &selector);
    if present(&existing) {
        return existing;
    }

    let status = call1(&document(), "createElement", &JsValue::from_str("span"))
        .unwrap_or(JsValue::UNDEFINED);
    if !present(&status) {
        return JsValue::UNDEFINED;
    }
    copy_log_set_attr(&status, "class", "kgw-copy-log-status-v1");
    set(
        &property(&status, "dataset"),
        "net",
        &JsValue::from_str(net),
    );
    copy_log_set_attr(&status, "data-net", net);
    copy_log_set_attr(&status, "role", "status");
    copy_log_set_attr(&status, "aria-live", "polite");
    let _ = call1(&toolbar, "appendChild", &status);
    status
}

fn set_copy_log_status(net: &str, message: &str, state: &str) -> bool {
    let status = copy_log_status_element(net);
    if !present(&status) {
        return false;
    }
    set(&status, "textContent", &JsValue::from_str(message));
    set(
        &property(&status, "dataset"),
        "state",
        &JsValue::from_str(state),
    );
    let _ = crate::apply_status_tone_js(status.clone(), JsValue::from_str(state));
    set(&status, "hidden", &JsValue::from_bool(message.is_empty()));
    true
}

fn read_copy_log_buffer(net: &str) -> NodeClipboardBuffer {
    let out = copy_log_output(net);
    let tag = crate::js_string_owned(&property(&out, "tagName")).to_uppercase();
    let raw_text = if present(&out) {
        if tag == "TEXTAREA" || tag == "INPUT" {
            crate::js_string_owned(&property(&out, "value"))
        } else {
            crate::js_string_owned(&property(&out, "textContent"))
        }
    } else {
        String::new()
    };
    let placeholder = clipboard_placeholder_text(net);
    let is_placeholder = raw_text.trim() == placeholder.trim();
    let normalized_text = normalize_clipboard_line_endings_text(&raw_text);
    NodeClipboardBuffer {
        out,
        is_placeholder,
        character_count: clipboard_character_count_text(&normalized_text) as u32,
        line_count: clipboard_line_count_text(&normalized_text) as u32,
        normalized_text,
    }
}

fn translate_copy_log_text(key: &str, fallback: &str) -> String {
    for name in ["kgwT", "kgwI18n", "__kgwT"] {
        let candidate = property(&window(), name);
        if let Ok(function) = candidate.dyn_into::<Function>()
            && let Ok(value) = function.call2(
                &window(),
                &JsValue::from_str(key),
                &JsValue::from_str(fallback),
            )
        {
            let text = crate::js_string_owned(&value);
            if !text.is_empty() && text != key {
                return text;
            }
        }
    }
    fallback.to_owned()
}

fn restore_copy_log_button_label(button: &JsValue) {
    if !present(button) {
        return;
    }
    let dataset = property(button, "dataset");
    let original = crate::js_string_owned(&property(&dataset, "kgwLogOriginalLabelV29"));
    if !original.is_empty() {
        set(button, "textContent", &JsValue::from_str(&original));
    }
    let class_list = property(button, "classList");
    let _ = call1(
        &class_list,
        "remove",
        &JsValue::from_str("kgw-log-action-feedback"),
    );
    if let Ok(dataset_object) = dataset.dyn_into::<Object>() {
        let _ = Reflect::delete_property(&dataset_object, &JsValue::from_str("kgwDoneLabel"));
    }
}

fn flash_copy_log_button(button: &JsValue, done_label: &str) {
    if !present(button) {
        return;
    }
    let dataset = property(button, "dataset");
    let original = crate::js_string_owned(&property(&dataset, "kgwLogOriginalLabelV29"));
    if original.is_empty() {
        let text = crate::js_string_owned(&property(button, "textContent"));
        let value = if text.trim().is_empty() {
            "Log Action"
        } else {
            text.trim()
        };
        set(
            &dataset,
            "kgwLogOriginalLabelV29",
            &JsValue::from_str(value),
        );
    }

    let old_timer = property(button, "__kgwLogActionFeedbackTimerV29");
    if present(&old_timer) {
        let _ = call1(&window(), "clearTimeout", &old_timer);
    }

    set(button, "textContent", &JsValue::from_str(done_label));
    set(&dataset, "kgwDoneLabel", &JsValue::from_str(done_label));
    let class_list = property(button, "classList");
    let _ = call1(
        &class_list,
        "add",
        &JsValue::from_str("kgw-log-action-feedback"),
    );

    let button_for_timer = button.clone();
    let callback = Closure::wrap(Box::new(move || {
        restore_copy_log_button_label(&button_for_timer);
    }) as Box<dyn FnMut()>);
    if let Some(timer) = copy_log_call2(
        &window(),
        "setTimeout",
        callback.as_ref(),
        &JsValue::from_f64(1600.0),
    ) {
        set(button, "__kgwLogActionFeedbackTimerV29", &timer);
    }
    callback.forget();
}

fn clone_object(value: &JsValue) -> JsValue {
    let output = Object::new();
    if value.is_object() && !value.is_null() && !Array::is_array(value) {
        for entry in Object::entries(&Object::from(value.clone())).iter() {
            let pair = Array::from(&entry);
            if pair.length() >= 2 {
                let key = crate::js_string_owned(&pair.get(0));
                set(output.as_ref(), &key, &pair.get(1));
            }
        }
    }
    output.into()
}

fn copy_log_failure_impl(net: &str, button: &JsValue, error: &JsValue, details: &JsValue) -> bool {
    let message = property(error, "message");
    let source = if crate::js_boolean(&message) {
        crate::js_string_owned(&message)
    } else if crate::js_boolean(error) {
        crate::js_string_owned(error)
    } else {
        "clipboard write failed".to_owned()
    };
    let safe_error = clipboard_safe_error_text(&source);
    let _ = set_copy_log_status(net, &safe_error, "error");
    flash_copy_log_button(
        button,
        &translate_copy_log_text("log.copyFailed", "Copy failed"),
    );

    let merged = clone_object(details);
    set(&merged, "safeError", &JsValue::from_str(&safe_error));
    set(&merged, "userFeedbackDisplayed", &JsValue::TRUE);
    emit_start_trace("frontend.copy_log_failed", net, "copy-log", "error", merged);
    false
}

async fn copy_log_sha256_hex(text: &str) -> String {
    let crypto = {
        let from_window = property(&window(), "crypto");
        if present(&from_window) {
            from_window
        } else {
            property(&global(), "crypto")
        }
    };
    let subtle = property(&crypto, "subtle");
    let Some(digest) = function(&subtle, "digest") else {
        return String::new();
    };
    let bytes = Uint8Array::from(text.as_bytes());
    let Ok(result) = digest.call2(&subtle, &JsValue::from_str("SHA-256"), bytes.as_ref()) else {
        return String::new();
    };
    let Ok(resolved) = JsFuture::from(Promise::resolve(&result)).await else {
        return String::new();
    };
    let bytes = Uint8Array::new(&resolved).to_vec();
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn metadata_text(metadata: &JsValue, name: &str, fallback: &str) -> String {
    let value = property(metadata, name);
    if crate::js_boolean(&value) {
        crate::js_string_owned(&value)
    } else {
        fallback.to_owned()
    }
}

fn metadata_number(metadata: &JsValue, name: &str) -> f64 {
    let value = property(metadata, name);
    let number = crate::js_number(&value);
    if number.is_finite() { number } else { 0.0 }
}

async fn await_with_timeout(value: JsValue, timeout_ms: u32) -> Result<JsValue, JsValue> {
    let timeout = Promise::new(&mut |_resolve, reject| {
        let reject_for_timer = reject.clone();
        let callback = Closure::wrap(Box::new(move || {
            let _ = reject_for_timer.call1(
                &JsValue::UNDEFINED,
                &JsValue::from_str("Tauri invoke timed out."),
            );
        }) as Box<dyn FnMut()>);
        let _ = copy_log_call2(
            &window(),
            "setTimeout",
            callback.as_ref(),
            &JsValue::from_f64(timeout_ms as f64),
        );
        callback.forget();
    });
    let values = Array::new();
    values.push(Promise::resolve(&value).as_ref());
    values.push(timeout.as_ref());
    JsFuture::from(Promise::race(values.as_ref())).await
}

async fn dispatch_clipboard_write_impl(
    net: &str,
    text: &str,
    metadata: &JsValue,
) -> Result<JsValue, JsValue> {
    let resolved = resolved_invoke();
    let Some(invoke) = property(&resolved, "invoke").dyn_into::<Function>().ok() else {
        return Err(JsValue::from_str(
            "Tauri invoke API is not available. Expected window.__TAURI__.core.invoke from Tauri 2 with withGlobalTauri enabled.",
        ));
    };
    let runtime_role = metadata_text(metadata, "runtimeRole", "node");
    let bridge_instance_id = metadata_text(metadata, "bridgeInstanceId", "");
    let character_count = metadata_number(metadata, "characterCount");
    let line_count = metadata_number(metadata, "lineCount");
    let sha256 = metadata_text(metadata, "sha256", "");

    let trace_details = Object::new();
    for (key, value) in [
        (
            "commandName",
            JsValue::from_str("kgw_copy_text_to_clipboard_v1"),
        ),
        ("implementation", JsValue::from_str("native-tauri-command")),
        ("runtimeRole", JsValue::from_str(&runtime_role)),
        ("bridgeInstanceId", JsValue::from_str(&bridge_instance_id)),
        ("characterCount", JsValue::from_f64(character_count)),
        ("lineCount", JsValue::from_f64(line_count)),
        ("sha256", JsValue::from_str(&sha256)),
        ("payloadFieldCount", JsValue::from_f64(7.0)),
    ] {
        set(trace_details.as_ref(), key, &value);
    }
    emit_start_trace(
        "frontend.copy_log_dispatched",
        net,
        "copy-log",
        "dispatched",
        trace_details.into(),
    );

    let payload = Object::new();
    set(payload.as_ref(), "network", &JsValue::from_str(net));
    set(
        payload.as_ref(),
        "runtimeRole",
        &JsValue::from_str(&runtime_role),
    );
    set(
        payload.as_ref(),
        "bridgeInstanceId",
        &JsValue::from_str(&bridge_instance_id),
    );
    set(payload.as_ref(), "text", &JsValue::from_str(text));
    set(
        payload.as_ref(),
        "characterCount",
        &JsValue::from_f64(character_count),
    );
    set(
        payload.as_ref(),
        "lineCount",
        &JsValue::from_f64(line_count),
    );
    set(payload.as_ref(), "sha256", &JsValue::from_str(&sha256));

    let result = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_copy_text_to_clipboard_v1"),
        payload.as_ref(),
    )?;
    await_with_timeout(result, 110_000).await
}

async fn handle_copy_log_impl(net: &str, button: &JsValue) -> bool {
    let root = node_root(&JsValue::UNDEFINED);
    let active_network = trace_active_network(&root);
    let belongs_to_live_node_monitor = present(&closest(button, "[data-node-inner-panel=\"log\"]"));
    let copy_network = net.trim().to_owned();

    if copy_network.is_empty() {
        let details = Object::new();
        set(
            details.as_ref(),
            "reason",
            &JsValue::from_str("missing-network"),
        );
        set(
            details.as_ref(),
            "activeNetwork",
            &JsValue::from_str(&active_network),
        );
        set(
            details.as_ref(),
            "belongsToLiveNodeMonitor",
            &JsValue::from_bool(belongs_to_live_node_monitor),
        );
        return copy_log_failure_impl(
            net,
            button,
            &JsValue::from_str("Copy Log could not resolve the active network."),
            details.as_ref(),
        );
    }

    let network_result = if !active_network.is_empty() && active_network != copy_network {
        "error"
    } else {
        "ok"
    };
    let network_details = Object::new();
    set(
        network_details.as_ref(),
        "activeNetwork",
        &JsValue::from_str(&active_network),
    );
    set(
        network_details.as_ref(),
        "buttonNetwork",
        &JsValue::from_str(&copy_network),
    );
    set(
        network_details.as_ref(),
        "belongsToLiveNodeMonitor",
        &JsValue::from_bool(belongs_to_live_node_monitor),
    );
    emit_start_trace(
        "frontend.copy_log_network_resolved",
        &copy_network,
        "copy-log",
        network_result,
        network_details.into(),
    );

    if network_result == "error" {
        let details = Object::new();
        set(
            details.as_ref(),
            "reason",
            &JsValue::from_str("network-mismatch"),
        );
        set(
            details.as_ref(),
            "activeNetwork",
            &JsValue::from_str(&active_network),
        );
        set(
            details.as_ref(),
            "buttonNetwork",
            &JsValue::from_str(&copy_network),
        );
        set(
            details.as_ref(),
            "belongsToLiveNodeMonitor",
            &JsValue::from_bool(belongs_to_live_node_monitor),
        );
        return copy_log_failure_impl(
            &copy_network,
            button,
            &JsValue::from_str(
                "Copy Log network mismatch; active network changed before copy started.",
            ),
            details.as_ref(),
        );
    }

    let button_dataset = property(button, "dataset");
    if crate::js_string_owned(&property(&button_dataset, "kgwCopyLogInFlightV1")) == "1" {
        let details = Object::new();
        set(
            details.as_ref(),
            "reason",
            &JsValue::from_str("duplicate-copy"),
        );
        set(
            details.as_ref(),
            "activeNetwork",
            &JsValue::from_str(&active_network),
        );
        set(
            details.as_ref(),
            "belongsToLiveNodeMonitor",
            &JsValue::from_bool(belongs_to_live_node_monitor),
        );
        return copy_log_failure_impl(
            &copy_network,
            button,
            &JsValue::from_str("Copy Log is already in progress for this network."),
            details.as_ref(),
        );
    }

    let original_disabled = present(button) && crate::js_boolean(&property(button, "disabled"));
    if present(button) {
        set(
            &button_dataset,
            "kgwCopyLogInFlightV1",
            &JsValue::from_str("1"),
        );
        set(button, "disabled", &JsValue::TRUE);
    }

    let outcome: Result<(), JsValue> = async {
        let buffer = read_copy_log_buffer(&copy_network);
        if !present(&buffer.out)
            || buffer.is_placeholder
            || buffer.normalized_text.trim().is_empty()
        {
            let details = Object::new();
            set(
                details.as_ref(),
                "rawLogBufferSelected",
                &JsValue::from_bool(present(&buffer.out)),
            );
            set(
                details.as_ref(),
                "placeholderRejected",
                &JsValue::from_bool(buffer.is_placeholder),
            );
            set(details.as_ref(), "runtimeRole", &JsValue::from_str("node"));
            set(details.as_ref(), "bridgeInstanceId", &JsValue::from_str(""));
            set(
                details.as_ref(),
                "characterCount",
                &JsValue::from_f64(buffer.character_count as f64),
            );
            set(
                details.as_ref(),
                "lineCount",
                &JsValue::from_f64(buffer.line_count as f64),
            );
            set(details.as_ref(), "sha256", &JsValue::from_str(""));
            emit_start_trace(
                "frontend.copy_log_content_prepared",
                &copy_network,
                "copy-log",
                "error",
                details.into(),
            );
            return Err(JsValue::from_str(&format!(
                "Copy Log requires a non-empty raw log buffer for {copy_network}."
            )));
        }

        let sha256 = copy_log_sha256_hex(&buffer.normalized_text).await;
        let metadata = Object::new();
        set(metadata.as_ref(), "runtimeRole", &JsValue::from_str("node"));
        set(
            metadata.as_ref(),
            "bridgeInstanceId",
            &JsValue::from_str(""),
        );
        set(
            metadata.as_ref(),
            "characterCount",
            &JsValue::from_f64(buffer.character_count as f64),
        );
        set(
            metadata.as_ref(),
            "lineCount",
            &JsValue::from_f64(buffer.line_count as f64),
        );
        set(metadata.as_ref(), "sha256", &JsValue::from_str(&sha256));

        let prepared = Object::new();
        set(prepared.as_ref(), "rawLogBufferSelected", &JsValue::TRUE);
        set(prepared.as_ref(), "placeholderRejected", &JsValue::FALSE);
        for key in [
            "runtimeRole",
            "bridgeInstanceId",
            "characterCount",
            "lineCount",
            "sha256",
        ] {
            set(prepared.as_ref(), key, &property(metadata.as_ref(), key));
        }
        emit_start_trace(
            "frontend.copy_log_content_prepared",
            &copy_network,
            "copy-log",
            "ok",
            prepared.into(),
        );

        let _ = dispatch_clipboard_write_impl(
            &copy_network,
            &buffer.normalized_text,
            metadata.as_ref(),
        )
        .await?;

        let copied_text = translate_copy_log_text("log.copied", "Copied");
        flash_copy_log_button(button, &copied_text);
        let _ = set_copy_log_status(&copy_network, &copied_text, "ok");

        let succeeded = Object::new();
        for key in [
            "runtimeRole",
            "bridgeInstanceId",
            "characterCount",
            "lineCount",
            "sha256",
        ] {
            set(succeeded.as_ref(), key, &property(metadata.as_ref(), key));
        }
        set(succeeded.as_ref(), "userFeedbackDisplayed", &JsValue::TRUE);
        emit_start_trace(
            "frontend.copy_log_succeeded",
            &copy_network,
            "copy-log",
            "ok",
            succeeded.into(),
        );
        Ok(())
    }
    .await;

    if let Err(error) = &outcome {
        let details = Object::new();
        set(
            details.as_ref(),
            "activeNetwork",
            &JsValue::from_str(&active_network),
        );
        set(
            details.as_ref(),
            "belongsToLiveNodeMonitor",
            &JsValue::from_bool(belongs_to_live_node_monitor),
        );
        let _ = copy_log_failure_impl(&copy_network, button, error, details.as_ref());
    }

    if present(button) {
        set(button, "disabled", &JsValue::from_bool(original_disabled));
        if let Ok(dataset_object) = property(button, "dataset").dyn_into::<Object>() {
            let _ = Reflect::delete_property(
                &dataset_object,
                &JsValue::from_str("kgwCopyLogInFlightV1"),
            );
        }
    }

    outcome.is_ok()
}

#[wasm_bindgen(js_name = nodeDispatchClipboardWrite)]
pub async fn node_dispatch_clipboard_write(
    net: String,
    text: String,
    metadata: JsValue,
) -> Result<JsValue, JsValue> {
    dispatch_clipboard_write_impl(&net, &text, &metadata).await
}

#[wasm_bindgen(js_name = nodeCopyLogFailure)]
pub fn node_copy_log_failure(
    net: String,
    button: JsValue,
    error: JsValue,
    details: JsValue,
) -> bool {
    copy_log_failure_impl(&net, &button, &error, &details)
}

#[wasm_bindgen(js_name = nodeHandleCopyLog)]
pub async fn node_handle_copy_log(net: String, button: JsValue) -> bool {
    handle_copy_log_impl(&net, &button).await
}

#[wasm_bindgen(js_name = nodeClipboardCharacterCount)]
pub fn node_clipboard_character_count(text: JsValue) -> u32 {
    clipboard_character_count_text(&crate::js_string_owned(&text)) as u32
}

#[wasm_bindgen(js_name = nodeClipboardLineCount)]
pub fn node_clipboard_line_count(text: JsValue) -> u32 {
    clipboard_line_count_text(&crate::js_string_owned(&text)) as u32
}

#[wasm_bindgen(js_name = nodeNormalizeClipboardLineEndings)]
pub fn node_normalize_clipboard_line_endings(text: JsValue) -> String {
    normalize_clipboard_line_endings_text(&crate::js_string_owned(&text))
}

#[wasm_bindgen(js_name = nodeClipboardSafeError)]
pub fn node_clipboard_safe_error(error: JsValue) -> String {
    let message = property(&error, "message");
    let source = if crate::js_boolean(&message) {
        crate::js_string_owned(&message)
    } else if crate::js_boolean(&error) {
        crate::js_string_owned(&error)
    } else {
        "clipboard write failed".to_owned()
    };
    clipboard_safe_error_text(&source)
}

#[wasm_bindgen(js_name = nodeClipboardPlaceholderText)]
pub fn node_clipboard_placeholder_text(net: String) -> String {
    clipboard_placeholder_text(&net)
}

#[wasm_bindgen(js_name = nodeTraceNetworkFromElement)]
pub fn node_trace_network_from_element(element: JsValue, root: JsValue) -> String {
    trace_network_from_element(&element, &root)
}

#[wasm_bindgen(js_name = nodeTraceActiveNetwork)]
pub fn node_trace_active_network(root: JsValue) -> String {
    trace_active_network(&root)
}

#[wasm_bindgen(js_name = nodeTraceStartButtonState)]
pub fn node_trace_start_button_state(net: String) -> JsValue {
    trace_start_button_state(&net)
}

#[wasm_bindgen(js_name = nodeInstallStartTraceDocumentClickObserver)]
pub fn node_install_start_trace_document_click_observer(root: JsValue) -> bool {
    install_start_trace_document_click_observer(&root)
}

#[wasm_bindgen(js_name = nodeTraceRenderedStartControls)]
pub fn node_trace_rendered_start_controls(root: JsValue) -> bool {
    trace_rendered_start_controls(&root)
}

#[wasm_bindgen(js_name = nodeRuntimeActionForCommand)]
pub fn node_runtime_action_for_command(command: String) -> String {
    runtime_action_for_command_text(&command).to_owned()
}

#[wasm_bindgen(js_name = nodeSmallOwnerTrace)]
pub fn node_small_owner_trace(
    net: JsValue,
    action: JsValue,
    phase: JsValue,
    details: JsValue,
) -> bool {
    small_owner_trace(net, action, phase, details)
}

#[wasm_bindgen(js_name = nodeStartTraceTauriShape)]
pub fn node_start_trace_tauri_shape(adapter: String) -> JsValue {
    tauri_shape(&adapter)
}

#[wasm_bindgen(js_name = nodeResolvePublicTauriInvoke)]
pub fn node_resolve_public_tauri_invoke() -> JsValue {
    resolved_invoke()
}

#[wasm_bindgen(js_name = nodeStartTraceFrontend)]
pub fn node_start_trace_frontend(stage: JsValue, options: JsValue) -> bool {
    let resolved = resolved_invoke();
    let Some(invoke) = property(&resolved, "invoke").dyn_into::<Function>().ok() else {
        return false;
    };
    let network_source = {
        let network = property(&options, "network");
        if crate::js_boolean(&network) {
            network
        } else {
            property(&options, "net")
        }
    };
    let details_source = property(&options, "details");
    let details = safe_details(&if details_source.is_object() {
        details_source
    } else {
        Object::new().into()
    });
    set(&details, "invokeAdapter", &property(&resolved, "adapter"));

    let args = Object::new();
    set(
        args.as_ref(),
        "stage",
        &JsValue::from_str(&safe_text(&stage, "frontend.unknown")),
    );
    set(
        args.as_ref(),
        "network",
        &JsValue::from_str(&safe_text(&network_source, "unknown")),
    );
    set(
        args.as_ref(),
        "action",
        &JsValue::from_str(&safe_text(&property(&options, "action"), "unknown")),
    );
    set(
        args.as_ref(),
        "result",
        &JsValue::from_str(&safe_text(&property(&options, "result"), "observed")),
    );
    let serialized = JSON::stringify(&details)
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    set(args.as_ref(), "details", &JsValue::from_str(&serialized));

    if let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str(COMMAND),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |error: JsValue| {
            let console = property(&global(), "console");
            if let Some(log) = function(&console, "error") {
                let message = {
                    let candidate = property(&error, "message");
                    if crate::js_boolean(&candidate) {
                        crate::js_string_owned(&candidate)
                    } else {
                        crate::js_string_owned(&error)
                    }
                };
                let _ = log.call2(
                    &console,
                    &JsValue::from_str("[KGW_START_TRACE_FRONTEND_FAILED]"),
                    &JsValue::from_str(&message),
                );
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_key_policy_matches_legacy_sensitive_fields() {
        for key in [
            "apiToken",
            "walletAddress",
            "commandPreview",
            "rpcEndpoint",
            "appDir",
        ] {
            assert!(blocked_key(key), "{key}");
        }
        assert!(!blocked_key("network"));
        assert!(!blocked_key("result"));
    }

    #[test]
    fn safe_text_normalizes_and_limits() {
        assert_eq!(safe_text_str(" a\tb\nc ", ""), "a b c");
        assert_eq!(safe_text_str("", "fallback"), "fallback");
        let long = "x".repeat(300);
        assert_eq!(safe_text_str(&long, "").len(), 220);
    }

    #[test]
    fn trace_network_tokens_preserve_legacy_precedence() {
        assert_eq!(network_from_trace_text("mainnet"), Some("mainnet"));
        assert_eq!(
            network_from_trace_text("panel tn10 active"),
            Some("testnet10")
        );
        assert_eq!(
            network_from_trace_text("testnet13 tn10 mainnet"),
            Some("testnet13")
        );
        assert_eq!(network_from_trace_text("unknown"), None);
    }

    #[test]
    fn runtime_action_mapping_matches_legacy_contract() {
        assert_eq!(
            runtime_action_for_command_text("kgw_kgw_apply_node_settings_v1"),
            "start"
        );
        assert_eq!(
            runtime_action_for_command_text("kgw_kgw_disable_network_v1"),
            "stop"
        );
        assert_eq!(runtime_action_for_command_text("other"), "runtime");
    }

    #[test]
    fn clipboard_counts_match_legacy_contract() {
        assert_eq!(clipboard_character_count_text("A😀B"), 3);
        assert_eq!(clipboard_line_count_text(""), 0);
        assert_eq!(clipboard_line_count_text("a"), 1);
        assert_eq!(clipboard_line_count_text("a\nb\n"), 3);
    }

    #[test]
    fn clipboard_line_endings_match_legacy_contract() {
        assert_eq!(
            normalize_clipboard_line_endings_text("a\r\nb\rc\nd"),
            "a\r\nb\r\nc\r\nd"
        );
    }

    #[test]
    fn clipboard_safe_error_matches_sensitive_policy() {
        assert_eq!(
            clipboard_safe_error_text(" token leaked\n"),
            "clipboard write failed with a sensitive error"
        );
        assert_eq!(
            clipboard_safe_error_text("  ordinary\t failure  "),
            "ordinary  failure"
        );
        assert_eq!(clipboard_safe_error_text(""), "clipboard write failed");
    }

    #[test]
    fn clipboard_placeholder_labels_match_node_profiles() {
        assert_eq!(
            clipboard_placeholder_text("mainnet"),
            "Mainnet log is empty."
        );
        assert_eq!(
            clipboard_placeholder_text("testnet10"),
            "Testnet 10 log is empty."
        );
        assert_eq!(
            clipboard_placeholder_text("testnet13"),
            "Testnet 13 log is empty."
        );
        assert_eq!(clipboard_placeholder_text("custom"), "custom log is empty.");
    }
}
