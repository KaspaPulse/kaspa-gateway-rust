use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

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
