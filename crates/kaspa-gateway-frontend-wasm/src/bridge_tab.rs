use js_sys::{Array, Function, Object, Promise, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{future_to_promise, spawn_local};

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
    property(&js_sys::global().into(), "window")
}
fn document() -> JsValue {
    property(&js_sys::global().into(), "document")
}
fn default_bridge_instances() -> JsValue {
    let output = Object::new();
    for net in ["mainnet", "testnet10", "testnet13"] {
        let items = Array::new();
        let item = Object::new();
        set(item.as_ref(), "id", &JsValue::from_f64(1.0));
        items.push(item.as_ref());
        set(output.as_ref(), net, items.as_ref());
    }
    output.into()
}

fn default_active_instance() -> JsValue {
    let output = Object::new();
    for net in ["mainnet", "testnet10", "testnet13"] {
        set(output.as_ref(), net, &JsValue::from_f64(1.0));
    }
    output.into()
}

thread_local! {
    static BRIDGE_INSTANCES: RefCell<Option<JsValue>> = const { RefCell::new(None) };
    static ACTIVE_INSTANCE: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}
fn bridge_instances() -> JsValue {
    BRIDGE_INSTANCES.with(|cell| {
        cell.borrow_mut()
            .get_or_insert_with(default_bridge_instances)
            .clone()
    })
}

fn active_instance() -> JsValue {
    ACTIVE_INSTANCE.with(|cell| {
        cell.borrow_mut()
            .get_or_insert_with(default_active_instance)
            .clone()
    })
}

fn structured_reader() -> JsValue {
    let instances = bridge_instances();
    let active = active_instance();
    let callback = Closure::wrap(Box::new(move |net: JsValue| -> JsValue {
        crate::bridge_instance_settings::bridge_r51_read_structured_instances(
            crate::js_string_owned(&net),
            instances.clone(),
            active.clone(),
        )
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    let value = callback.as_ref().clone();
    callback.forget();
    value
}

fn command_lines_builder() -> JsValue {
    let instances = bridge_instances();
    let active = active_instance();
    let callback = Closure::wrap(Box::new(move |net: JsValue| -> Result<JsValue, JsValue> {
        crate::bridge_frontend_helpers::bridge_build_command_lines_ui(
            crate::js_string_owned(&net),
            instances.clone(),
            active.clone(),
        )
        .map(Into::into)
    })
        as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>);
    let value = callback.as_ref().clone();
    callback.forget();
    value
}

fn update_command(net: &str) -> String {
    crate::bridge_frontend_helpers::bridge_update_command_ui(
        net.to_owned(),
        bridge_instances(),
        active_instance(),
        structured_reader(),
        command_lines_builder(),
    )
}

fn update_command_callback() -> JsValue {
    let callback = Closure::wrap(Box::new(move |net: JsValue| -> String {
        update_command(&crate::js_string_owned(&net))
    }) as Box<dyn FnMut(JsValue) -> String>);
    let value = callback.as_ref().clone();
    callback.forget();
    value
}
fn refresh_instances(net: &str) -> Result<String, JsValue> {
    let callbacks = Object::new();
    let decorate = Closure::wrap(Box::new(move |container: JsValue| -> Result<(), JsValue> {
        crate::settings_layout::decorate_fields(container)
    }) as Box<dyn FnMut(JsValue) -> Result<(), JsValue>>);
    set(
        callbacks.as_ref(),
        "decorateSettingsFields",
        decorate.as_ref(),
    );

    let install = Closure::wrap(Box::new(
        move |container: JsValue, net: JsValue| -> Result<bool, JsValue> {
            install_instance_owner(container, &crate::js_string_owned(&net))
        },
    )
        as Box<dyn FnMut(JsValue, JsValue) -> Result<bool, JsValue>>);
    set(
        callbacks.as_ref(),
        "installInstanceContainerOwner",
        install.as_ref(),
    );

    let update = update_command_callback();
    set(callbacks.as_ref(), "updateCommand", &update);
    let result = crate::bridge_instance_ui::bridge_refresh_instances_ui(
        net.to_owned(),
        bridge_instances(),
        active_instance(),
        callbacks.into(),
    );
    decorate.forget();
    install.forget();
    result
}

fn add_instance(net: &str) -> Result<bool, JsValue> {
    let callbacks = Object::new();
    let refresh = Closure::wrap(Box::new(move |net: JsValue| -> Result<String, JsValue> {
        refresh_instances(&crate::js_string_owned(&net))
    }) as Box<dyn FnMut(JsValue) -> Result<String, JsValue>>);
    set(callbacks.as_ref(), "refreshInstances", refresh.as_ref());
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    let result = crate::bridge_instance_settings::bridge_add_instance_ui(
        net.to_owned(),
        bridge_instances(),
        active_instance(),
        callbacks.into(),
    );
    refresh.forget();
    result
}

fn remove_instance(net: &str, instance_id: JsValue) -> Result<bool, JsValue> {
    let callbacks = Object::new();
    let refresh = Closure::wrap(Box::new(move |net: JsValue| -> Result<String, JsValue> {
        refresh_instances(&crate::js_string_owned(&net))
    }) as Box<dyn FnMut(JsValue) -> Result<String, JsValue>>);
    set(callbacks.as_ref(), "refreshInstances", refresh.as_ref());
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    let result = crate::bridge_instance_settings::bridge_remove_instance_ui(
        net.to_owned(),
        instance_id,
        bridge_instances(),
        active_instance(),
        callbacks.into(),
    );
    refresh.forget();
    result
}
fn install_instance_owner(container: JsValue, net: &str) -> Result<bool, JsValue> {
    let callbacks = Object::new();
    let add = Closure::wrap(Box::new(move |net: JsValue| {
        let _ = add_instance(&crate::js_string_owned(&net));
    }) as Box<dyn FnMut(JsValue)>);
    set(callbacks.as_ref(), "addInstance", add.as_ref());

    let refresh = Closure::wrap(Box::new(move |net: JsValue| {
        let _ = refresh_instances(&crate::js_string_owned(&net));
    }) as Box<dyn FnMut(JsValue)>);
    set(callbacks.as_ref(), "refreshInstances", refresh.as_ref());

    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );

    let remove = Closure::wrap(Box::new(move |net: JsValue, instance: JsValue| {
        let _ = remove_instance(&crate::js_string_owned(&net), instance);
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(callbacks.as_ref(), "removeInstance", remove.as_ref());

    let result = crate::bridge_instance_ui::bridge_install_instance_container_owner_r11(
        container,
        net.to_owned(),
        active_instance(),
        callbacks.into(),
    );
    add.forget();
    refresh.forget();
    remove.forget();
    result
}

fn install_visible_instance_owners() -> Result<u32, JsValue> {
    let callbacks = Object::new();
    let install = Closure::wrap(Box::new(
        move |container: JsValue, net: JsValue| -> Result<bool, JsValue> {
            install_instance_owner(container, &crate::js_string_owned(&net))
        },
    )
        as Box<dyn FnMut(JsValue, JsValue) -> Result<bool, JsValue>>);
    set(
        callbacks.as_ref(),
        "installInstanceContainerOwner",
        install.as_ref(),
    );
    let result =
        crate::bridge_instance_ui::bridge_install_all_visible_instance_container_owners_r11(
            callbacks.into(),
        );
    install.forget();
    result
}

fn render_sections(profile: JsValue) -> Result<String, JsValue> {
    let callbacks = Object::new();
    let render = Closure::wrap(Box::new(move |profile: JsValue| -> String {
        crate::bridge_render::bridge_render_inprocess_node_settings_ui(profile)
    }) as Box<dyn FnMut(JsValue) -> String>);
    set(
        callbacks.as_ref(),
        "renderInprocessNodeSettings",
        render.as_ref(),
    );
    set(callbacks.as_ref(), "bridgeInstances", &bridge_instances());
    set(callbacks.as_ref(), "activeInstance", &active_instance());
    let result =
        crate::bridge_frontend_helpers::bridge_render_sections_ui(profile, callbacks.into());
    render.forget();
    result
}
fn render_network_panel(profile: JsValue, index: u32) -> Result<String, JsValue> {
    let callbacks = Object::new();
    let render = Closure::wrap(
        Box::new(move |profile: JsValue| -> Result<String, JsValue> { render_sections(profile) })
            as Box<dyn FnMut(JsValue) -> Result<String, JsValue>>,
    );
    set(callbacks.as_ref(), "renderSections", render.as_ref());
    let result = crate::bridge_frontend_helpers::bridge_render_network_panel_ui(
        profile,
        index,
        callbacks.into(),
    );
    render.forget();
    result
}

fn render_all_networks(root: JsValue) -> Result<bool, JsValue> {
    let callbacks = Object::new();
    let render = Closure::wrap(Box::new(
        move |profile: JsValue, index: JsValue| -> Result<String, JsValue> {
            render_network_panel(profile, crate::js_number(&index).max(0.0) as u32)
        },
    )
        as Box<dyn FnMut(JsValue, JsValue) -> Result<String, JsValue>>);
    set(callbacks.as_ref(), "renderNetworkPanel", render.as_ref());

    let layout = Closure::wrap(Box::new(move |root: JsValue| -> Result<(), JsValue> {
        crate::settings_layout::install_layout(root)
    }) as Box<dyn FnMut(JsValue) -> Result<(), JsValue>>);
    set(callbacks.as_ref(), "installSettingsLayout", layout.as_ref());

    let result =
        crate::bridge_frontend_helpers::bridge_render_all_networks_ui(root, callbacks.into());
    render.forget();
    layout.forget();
    result
}

fn write_settings_callbacks() -> JsValue {
    let callbacks = Object::new();
    let refresh = Closure::wrap(Box::new(move |net: JsValue| {
        let _ = refresh_instances(&crate::js_string_owned(&net));
    }) as Box<dyn FnMut(JsValue)>);
    set(callbacks.as_ref(), "refreshInstances", refresh.as_ref());
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    let output = crate::bridge_instance_settings::bridge_r51_write_settings_callbacks_r250(
        bridge_instances(),
        active_instance(),
        callbacks.into(),
    );
    refresh.forget();
    output
}
fn persistence_callbacks() -> JsValue {
    let callbacks = Object::new();

    let read = Closure::wrap(Box::new(move |net: JsValue| -> Result<JsValue, JsValue> {
        crate::bridge_instance_settings::bridge_r51_read_settings_owned(
            crate::js_string_owned(&net),
            bridge_instances(),
            active_instance(),
        )
    })
        as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>);
    set(callbacks.as_ref(), "readSettings", read.as_ref());

    let write = Closure::wrap(Box::new(
        move |net: JsValue, values: JsValue| -> Result<(), JsValue> {
            crate::bridge_instance_settings::bridge_r51_write_settings(
                crate::js_string_owned(&net),
                values,
                bridge_instances(),
                active_instance(),
                write_settings_callbacks(),
            )
        },
    )
        as Box<dyn FnMut(JsValue, JsValue) -> Result<(), JsValue>>);
    set(callbacks.as_ref(), "writeSettings", write.as_ref());

    let normalize = Closure::wrap(Box::new(
        move |net: JsValue, values: JsValue, reason: JsValue| -> JsValue {
            crate::bridge_frontend_helpers::bridge_r95b_normalize_network_port_values(
                crate::js_string_owned(&net),
                values,
                crate::js_string_owned(&reason),
            )
        },
    )
        as Box<dyn FnMut(JsValue, JsValue, JsValue) -> JsValue>);
    set(
        callbacks.as_ref(),
        "normalizeNetworkPortValues",
        normalize.as_ref(),
    );

    let require = Closure::wrap(Box::new(move |net: JsValue| -> Result<(), JsValue> {
        crate::bridge_frontend_helpers::bridge_require_valid_settings_ui(
            crate::js_string_owned(&net),
            bridge_instances(),
            active_instance(),
            structured_reader(),
        )
    }) as Box<dyn FnMut(JsValue) -> Result<(), JsValue>>);
    set(callbacks.as_ref(), "requireValidSettings", require.as_ref());
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );

    read.forget();
    write.forget();
    normalize.forget();
    require.forget();
    callbacks.into()
}
fn live_refresh_callbacks() -> JsValue {
    let callbacks = Object::new();

    let invoke = Closure::wrap(Box::new(move |command: JsValue, net: JsValue| -> Promise {
        let command = crate::js_string_owned(&command);
        let net = crate::js_string_owned(&net);
        future_to_promise(async move {
            crate::bridge_frontend_helpers::bridge_invoke_integrated_runtime_ui(
                command,
                net,
                bridge_instances(),
                active_instance(),
                structured_reader(),
                command_lines_builder(),
            )
            .await
        })
    }) as Box<dyn FnMut(JsValue, JsValue) -> Promise>);
    set(callbacks.as_ref(), "invokeRuntime", invoke.as_ref());

    let transition = Closure::wrap(Box::new(move |net: JsValue| -> bool {
        crate::bridge_frontend_helpers::bridge_runtime_transition_active_r300(
            crate::js_string_owned(&net),
        )
    }) as Box<dyn FnMut(JsValue) -> bool>);
    set(callbacks.as_ref(), "transitionActive", transition.as_ref());

    let raw = Closure::wrap(Box::new(move |net: JsValue| -> String {
        crate::bridge_port_orchestration::bridge_active_raw_log_instance_id(
            bridge_instances(),
            active_instance(),
            crate::js_string_owned(&net),
        )
        .unwrap_or_default()
    }) as Box<dyn FnMut(JsValue) -> String>);
    set(callbacks.as_ref(), "activeRawLogInstanceId", raw.as_ref());

    invoke.forget();
    transition.forget();
    raw.forget();
    callbacks.into()
}
fn run_integrated_action(action: String, net: String) -> Promise {
    let callbacks = Object::new();

    let confirm = Closure::wrap(Box::new(move |message: JsValue| -> Promise {
        future_to_promise(async move {
            let confirmed = crate::settings_contract::confirm_user_action(message).await?;
            Ok(JsValue::from_bool(confirmed))
        })
    }) as Box<dyn FnMut(JsValue) -> Promise>);
    set(callbacks.as_ref(), "confirmUserAction", confirm.as_ref());

    let validate = Closure::wrap(Box::new(move |net: JsValue, focus: JsValue| -> JsValue {
        crate::bridge_frontend_helpers::bridge_validate_form_ui(
            crate::js_string_owned(&net),
            bridge_instances(),
            crate::js_boolean(&focus),
        )
    }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
    set(callbacks.as_ref(), "validateForm", validate.as_ref());
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );

    let live =
        Closure::wrap(Box::new(move || -> JsValue { live_refresh_callbacks() })
            as Box<dyn FnMut() -> JsValue>);
    set(callbacks.as_ref(), "liveRefreshCallbacks", live.as_ref());

    let callback_value: JsValue = callbacks.into();
    confirm.forget();
    validate.forget();
    live.forget();

    future_to_promise(async move {
        crate::bridge_frontend_helpers::bridge_run_integrated_action_ui(
            action,
            net,
            bridge_instances(),
            active_instance(),
            structured_reader(),
            command_lines_builder(),
            callback_value,
        )
        .await
        .map(JsValue::from_bool)
    })
}

fn install_network_tabs(root: JsValue) -> bool {
    let callbacks = Object::new();
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    let live =
        Closure::wrap(Box::new(move || -> JsValue { live_refresh_callbacks() })
            as Box<dyn FnMut() -> JsValue>);
    set(callbacks.as_ref(), "liveRefreshCallbacks", live.as_ref());
    let result =
        crate::bridge_frontend_helpers::bridge_install_network_tabs_ui(root, callbacks.into());
    live.forget();
    result
}
fn set_button_action_result(button: &JsValue, result: &str) {
    set(
        &property(button, "dataset"),
        "kgwSettingsActionResult",
        &JsValue::from_str(result),
    );
}

fn settings_action(action: &str, net: &str, button: &JsValue) {
    let result = match action {
        "save" => crate::bridge_instance_settings::bridge_r51_save_settings(
            net.to_owned(),
            persistence_callbacks(),
        ),
        "defaults" => crate::bridge_instance_settings::bridge_r51_set_as_defaults(
            net.to_owned(),
            persistence_callbacks(),
        ),
        "restore" => crate::bridge_instance_settings::bridge_r51_restore_defaults(
            net.to_owned(),
            persistence_callbacks(),
        ),
        _ => Ok(()),
    };
    match result {
        Ok(()) => set_button_action_result(button, "success"),
        Err(error) => {
            set_button_action_result(button, "failed");
            let message = crate::bridge_runtime_core::bridge_normalize_runtime_error(error);
            crate::bridge_runtime_core::bridge_set_runtime_error_v1(
                net.to_owned(),
                JsValue::from_str(&message),
                JsValue::UNDEFINED,
            );
        }
    }
}

fn copy_value(action: String, net: String) {
    spawn_local(async move {
        let result = copy_value_inner(&action, &net).await;
        if let Err(error) = result {
            let message = crate::bridge_runtime_core::bridge_normalize_runtime_error(error);
            crate::bridge_frontend_helpers::bridge_preview_message(
                net,
                format!("Copy failed: {message}"),
                true,
            );
        }
    });
}
async fn copy_value_inner(action: &str, net: &str) -> Result<(), JsValue> {
    let mut text =
        crate::bridge_frontend_helpers::bridge_value(net.to_owned(), "appdir".to_owned());
    if action == "copy-command" {
        let errors = crate::bridge_frontend_helpers::bridge_validate_form_ui(
            net.to_owned(),
            bridge_instances(),
            false,
        );
        if Reflect::own_keys(&errors)?.length() > 0 {
            let values = Object::values(&Object::from(errors));
            return Err(JsValue::from_str(&crate::js_string_owned(&values.get(0))));
        }
        let sequence = crate::bridge_frontend_helpers::bridge_preview_sequence(net.to_owned());
        let payload = crate::bridge_frontend_helpers::bridge_build_apply_payload_ui(
            net.to_owned(),
            "kgw_kgw_apply_node_settings_v1".to_owned(),
            bridge_instances(),
            active_instance(),
            structured_reader(),
            command_lines_builder(),
        )?;
        let preview =
            crate::bridge_start_trace::bridge_prepare_preview(net.to_owned(), payload).await?;
        if crate::bridge_frontend_helpers::bridge_preview_sequence(net.to_owned()) != sequence {
            return Err(JsValue::from_str(
                "Settings changed while copying. Try again.",
            ));
        }
        text = js_sys::JSON::stringify(&preview)
            .map(|value| crate::js_string_owned(value.as_ref()))
            .unwrap_or_default();
    }
    if text.is_empty() {
        return Err(JsValue::from_str("There is no validated value to copy."));
    }
    let metadata = Object::new();
    set(
        metadata.as_ref(),
        "characterCount",
        &JsValue::from_f64(text.chars().count() as f64),
    );
    set(
        metadata.as_ref(),
        "lineCount",
        &JsValue::from_f64(text.lines().count().max(1) as f64),
    );
    crate::bridge_start_trace::bridge_dispatch_clipboard_write(
        net.to_owned(),
        text,
        metadata.into(),
    )
    .await?;
    crate::bridge_frontend_helpers::bridge_preview_message(
        net.to_owned(),
        if action == "copy-path" {
            "Data directory copied."
        } else {
            "Effective settings copied."
        }
        .to_owned(),
        false,
    );
    Ok(())
}
fn root_action_callbacks() -> JsValue {
    let callbacks = Object::new();
    set(
        callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );

    let select = Closure::wrap(Box::new(move |net: JsValue, instance: JsValue| {
        let net = crate::js_string_owned(&net);
        set(&active_instance(), &net, &instance);
        let _ = refresh_instances(&net);
        let active_id = crate::js_string_owned(&property(&active_instance(), &net));
        let _ = crate::bridge_raw_log::bridge_render_raw_log_buffer(
            net,
            "bridge".to_owned(),
            active_id,
        );
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(callbacks.as_ref(), "selectInstance", select.as_ref());

    let add = Closure::wrap(Box::new(move |net: JsValue| {
        let _ = add_instance(&crate::js_string_owned(&net));
    }) as Box<dyn FnMut(JsValue)>);
    set(callbacks.as_ref(), "addInstance", add.as_ref());

    let remove = Closure::wrap(Box::new(move |net: JsValue, instance: JsValue| {
        let _ = remove_instance(&crate::js_string_owned(&net), instance);
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(callbacks.as_ref(), "removeInstance", remove.as_ref());

    select.forget();
    add.forget();
    remove.forget();
    root_action_settings_callbacks(&callbacks);
    root_action_misc_callbacks(&callbacks);
    callbacks.into()
}

fn root_action_settings_callbacks(callbacks: &Object) {
    for (name, action) in [
        ("saveSettings", "save"),
        ("setDefaults", "defaults"),
        ("restoreDefaults", "restore"),
    ] {
        let action = action.to_owned();
        let callback = Closure::wrap(Box::new(move |net: JsValue, button: JsValue| {
            settings_action(&action, &crate::js_string_owned(&net), &button);
        }) as Box<dyn FnMut(JsValue, JsValue)>);
        set(callbacks.as_ref(), name, callback.as_ref());
        callback.forget();
    }
}
fn root_action_misc_callbacks(callbacks: &Object) {
    let log = Closure::wrap(
        Box::new(move |action: JsValue, net: JsValue, button: JsValue| {
            let action = crate::js_string_owned(&action);
            let net = crate::js_string_owned(&net);
            spawn_local(async move {
                let deps = Object::new();
                set(deps.as_ref(), "bridgeInstances", &bridge_instances());
                set(deps.as_ref(), "activeInstance", &active_instance());
                let _ = crate::bridge_start_trace::bridge_handle_log_action(
                    action,
                    net,
                    button,
                    deps.into(),
                )
                .await;
            });
        }) as Box<dyn FnMut(JsValue, JsValue, JsValue)>,
    );
    set(callbacks.as_ref(), "logAction", log.as_ref());

    let monitor = Closure::wrap(Box::new(move |net: JsValue, button: JsValue| {
        let net = crate::js_string_owned(&net);
        let panel = crate::bridge_instance_settings::bridge_r51_panel(net);
        let next = crate::js_string_owned(&property(&property(&button, "dataset"), "nextAction"));
        let selector = if next == "start" {
            "[data-bridge-action=\"start\"]"
        } else {
            "[data-bridge-inner-tab=\"settings\"]"
        };
        if let Some(query) = function(&panel, "querySelector")
            && let Ok(element) = query.call1(&panel, &JsValue::from_str(selector))
            && present(&element)
            && let Some(click) = function(&element, "click")
        {
            let _ = click.call0(&element);
        }
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(callbacks.as_ref(), "monitorNext", monitor.as_ref());

    let copy = Closure::wrap(Box::new(move |action: JsValue, net: JsValue| {
        copy_value(
            crate::js_string_owned(&action),
            crate::js_string_owned(&net),
        );
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(callbacks.as_ref(), "copyValue", copy.as_ref());
    let runtime = Closure::wrap(Box::new(move |action: JsValue, net: JsValue| {
        let action = crate::js_string_owned(&action);
        let net = crate::js_string_owned(&net);
        let promise = run_integrated_action(action.clone(), net.clone());
        let net_error = net.clone();
        let action_error = action.clone();
        let catch = Closure::wrap(Box::new(move |error: JsValue| {
            crate::bridge_runtime_core::bridge_r51_set_runtime_buttons(
                net_error.clone(),
                false,
                String::new(),
                String::new(),
                String::new(),
            );
            let message = crate::bridge_runtime_core::bridge_normalize_runtime_error(error);
            crate::bridge_runtime_core::bridge_set_runtime_error_v1(
                net_error.clone(),
                JsValue::from_str(&message),
                JsValue::UNDEFINED,
            );
            crate::bridge_runtime_core::bridge_set_runtime_activity_v1(
                net_error.clone(),
                JsValue::from_str(&format!("Bridge {action_error} failed.")),
                JsValue::UNDEFINED,
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(callbacks.as_ref(), "runtimeAction", runtime.as_ref());

    log.forget();
    monitor.forget();
    copy.forget();
    runtime.forget();
}
fn install_actions(root: JsValue) {
    let event_callbacks = Object::new();
    set(
        event_callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    crate::bridge_frontend_helpers::bridge_install_action_event_owners_ui(
        root.clone(),
        bridge_instances(),
        active_instance(),
        event_callbacks.into(),
    );

    let port_callbacks = Object::new();
    let refresh = Closure::wrap(Box::new(move |net: JsValue| {
        let _ = refresh_instances(&crate::js_string_owned(&net));
    }) as Box<dyn FnMut(JsValue)>);
    set(
        port_callbacks.as_ref(),
        "refreshInstances",
        refresh.as_ref(),
    );
    set(
        port_callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    let runtime_activity = Closure::wrap(Box::new(move |net: JsValue, message: JsValue| {
        crate::bridge_runtime_core::bridge_set_runtime_activity_v1(
            crate::js_string_owned(&net),
            message,
            JsValue::UNDEFINED,
        );
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(
        port_callbacks.as_ref(),
        "runtimeActivity",
        runtime_activity.as_ref(),
    );
    crate::bridge_port_ui::bridge_install_port_event_owners_ui(
        root.clone(),
        bridge_instances(),
        active_instance(),
        port_callbacks.into(),
    );
    refresh.forget();
    runtime_activity.forget();
    let settings_callbacks = Object::new();
    set(
        settings_callbacks.as_ref(),
        "updateCommand",
        &update_command_callback(),
    );
    let run = Closure::wrap(Box::new(move |action: JsValue, net: JsValue| {
        let _ = run_integrated_action(
            crate::js_string_owned(&action),
            crate::js_string_owned(&net),
        );
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(
        settings_callbacks.as_ref(),
        "runIntegratedAction",
        run.as_ref(),
    );
    crate::bridge_frontend_helpers::bridge_install_settings_event_owners_ui(
        root.clone(),
        bridge_instances(),
        settings_callbacks.into(),
    );
    run.forget();

    let owner_callbacks = crate::bridge_frontend_helpers::bridge_settings_owner_callbacks_v19(
        bridge_instances(),
        active_instance(),
    );
    crate::node_settings_owner::settings_owner_install(root.clone(), owner_callbacks);

    let _ = install_visible_instance_owners();
    crate::bridge_frontend_helpers::bridge_install_root_action_click_owner_ui(
        root,
        bridge_instances(),
        root_action_callbacks(),
    );
}
fn schedule(delay: f64, callback: impl FnMut() + 'static) {
    let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    if let Some(set_timeout) = function(&window(), "setTimeout") {
        let _ = set_timeout.call2(
            &window(),
            closure.as_ref().unchecked_ref(),
            &JsValue::from_f64(delay),
        );
    }
    closure.forget();
}

fn install_compatibility_globals() {
    let callbacks = crate::bridge_frontend_helpers::bridge_settings_owner_callbacks_v19(
        bridge_instances(),
        active_instance(),
    );
    let owner = Object::new();

    let install_callbacks = callbacks.clone();
    let install = Closure::wrap(Box::new(move |root: JsValue| -> bool {
        crate::node_settings_owner::settings_owner_install(root, install_callbacks.clone())
    }) as Box<dyn FnMut(JsValue) -> bool>);
    set(owner.as_ref(), "install", install.as_ref());

    let disabled_callbacks = callbacks.clone();
    let disabled = Closure::wrap(Box::new(
        move |root: JsValue, net: JsValue, disabled: JsValue, reason: JsValue| {
            crate::node_settings_owner::settings_owner_set_disabled(
                root,
                crate::js_string_owned(&net),
                crate::js_boolean(&disabled),
                crate::js_string_owned(&reason),
                disabled_callbacks.clone(),
            );
        },
    ) as Box<dyn FnMut(JsValue, JsValue, JsValue, JsValue)>);
    set(owner.as_ref(), "setDisabled", disabled.as_ref());
    let buttons = Closure::wrap(Box::new(move |root: JsValue, net: JsValue| -> Array {
        crate::node_settings_owner::settings_owner_buttons(root, {
            let value = crate::js_string_owned(&net);
            if value.is_empty() {
                "all".to_owned()
            } else {
                value
            }
        })
    }) as Box<dyn FnMut(JsValue, JsValue) -> Array>);
    set(owner.as_ref(), "buttons", buttons.as_ref());

    let owner_value: JsValue = owner.into();
    set(&window(), "KGW_BRIDGE_SETTINGS_OWNER_V19", &owner_value);
    set(&window(), "KGW_SETTINGS_OWNER_V19", &owner_value);
    install.forget();
    disabled.forget();
    buttons.forget();
}

#[wasm_bindgen(js_name = bridgeEffectiveInprocessNodeSettingsOwned)]
pub fn bridge_effective_inprocess_node_settings_owned(net: String) -> Result<JsValue, JsValue> {
    crate::bridge_frontend_helpers::bridge_effective_inprocess_node_settings_checked(
        net,
        bridge_instances(),
    )
}

#[wasm_bindgen(js_name = bridgeValidateFormOwned)]
pub fn bridge_validate_form_owned(net: String, focus: bool) -> JsValue {
    crate::bridge_frontend_helpers::bridge_validate_form_ui(net, bridge_instances(), focus)
}
#[wasm_bindgen(js_name = bridgeInitKaspaBridgeTab)]
pub fn bridge_init_kaspa_bridge_tab(root: JsValue) -> bool {
    let bridge_root = if present(&root) {
        root
    } else {
        function(&document(), "getElementById")
            .and_then(|get| {
                get.call1(&document(), &JsValue::from_str("kaspa-bridge"))
                    .ok()
            })
            .unwrap_or(JsValue::UNDEFINED)
    };
    if !present(&bridge_root) {
        return false;
    }
    let dataset = property(&bridge_root, "dataset");
    if crate::js_string_owned(&property(&dataset, "kgwBridgeV7Ready")) == "true" {
        return false;
    }
    set(&dataset, "kgwBridgeV7Ready", &JsValue::from_str("true"));
    install_compatibility_globals();

    if render_all_networks(bridge_root.clone()).is_err() {
        return false;
    }
    let persistence = persistence_callbacks();
    let _ =
        crate::bridge_instance_settings::bridge_r51_capture_factory_defaults(persistence.clone());
    let _ = crate::bridge_instance_settings::bridge_r51_load_saved_settings(persistence);
    for net in ["mainnet", "testnet10", "testnet13"] {
        crate::bridge_runtime_core::bridge_r51_set_runtime_buttons(
            net.to_owned(),
            false,
            String::new(),
            String::new(),
            String::new(),
        );
    }
    crate::bridge_frontend_helpers::bridge_sync_all_mode_controls_ui(bridge_instances());
    install_network_tabs(bridge_root.clone());
    crate::bridge_frontend_helpers::bridge_install_delegated_tabs_ui(
        bridge_root.clone(),
        active_instance(),
    );
    install_actions(bridge_root.clone());
    crate::bridge_frontend_helpers::bridge_sync_all_mode_controls_ui(bridge_instances());
    crate::bridge_frontend_helpers::bridge_update_all_commands_ui(
        bridge_instances(),
        active_instance(),
        structured_reader(),
        command_lines_builder(),
    );

    for net in ["mainnet", "testnet10", "testnet13"] {
        if let Ok(update) = update_command_callback().dyn_into::<Function>() {
            crate::bridge_frontend_helpers::bridge_apply_rusty_kaspa_root_only_default_paths_soon_r5(
                net.to_owned(), update,
            );
        }
    }
    schedule(0.0, || {
        crate::bridge_frontend_helpers::bridge_update_all_commands_ui(
            bridge_instances(),
            active_instance(),
            structured_reader(),
            command_lines_builder(),
        );
    });
    schedule(150.0, || {
        crate::bridge_frontend_helpers::bridge_update_all_commands_ui(
            bridge_instances(),
            active_instance(),
            structured_reader(),
            command_lines_builder(),
        );
    });
    crate::bridge_frontend_helpers::bridge_sync_all_mode_controls_ui(bridge_instances());
    crate::bridge_frontend_helpers::bridge_update_all_commands_ui(
        bridge_instances(),
        active_instance(),
        structured_reader(),
        command_lines_builder(),
    );
    crate::bridge_runtime_core::bridge_r51_start_live_refresh(live_refresh_callbacks());

    schedule(0.0, || {
        crate::bridge_frontend_helpers::bridge_install_log_auto_scroll_controls();
    });
    schedule(0.0, || {
        crate::bridge_frontend_helpers::bridge_install_log_font_controls_v29();
    });
    crate::bridge_port_ui::bridge_autofix_button_initial_label_ui_r111g(document());
    true
}
