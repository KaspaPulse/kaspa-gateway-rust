use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;

use crate::settings_schema::{BRIDGE_MANAGED, BRIDGE_OPTIONAL, BRIDGE_REQUIRED};

const INLINE_STATE_GLOBAL: &str = "__kgwBridgeCommandComposerInlineR7";
const INSTANCE_STATE_GLOBAL: &str = "__kgwBridgeInstanceCommandComposerR13B";
const INSTANCE_OPTIONAL_FIELDS: &[&str] = &[
    "instanceBlockWaitTime",
    "instanceExtranonceSize",
    "instanceSharesPerMin",
];

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}
fn set_property(target: &JsValue, name: &str, value: &JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(name), value);
}

fn object_or_create(owner: &JsValue, name: &str) -> JsValue {
    let current = property(owner, name);
    if current.is_object() && !current.is_null() {
        return current;
    }
    let created = Object::new();
    set_property(owner, name, created.as_ref());
    created.into()
}

fn inline_key_text(net: &str) -> String {
    let value = net.trim();
    if value.is_empty() {
        "mainnet".to_owned()
    } else {
        value.to_owned()
    }
}

fn instance_key_text(net: &str, instance_id: &str, name: &str) -> String {
    format!(
        "{}::{}::{}",
        if net.trim().is_empty() {
            "mainnet"
        } else {
            net.trim()
        },
        if instance_id.trim().is_empty() {
            "1"
        } else {
            instance_id.trim()
        },
        name.trim()
    )
}
fn managed(name: &str) -> bool {
    BRIDGE_MANAGED
        .iter()
        .any(|(candidate, _)| *candidate == name)
}

fn required(name: &str) -> bool {
    BRIDGE_REQUIRED.contains(&name)
}

fn optional(name: &str) -> bool {
    BRIDGE_OPTIONAL.contains(&name)
}

fn inline_enabled_from_stored(name: &str, stored: Option<bool>) -> bool {
    if required(name) {
        true
    } else if managed(name) {
        false
    } else if optional(name) {
        stored == Some(true)
    } else {
        stored != Some(false)
    }
}

fn inline_state(net: &str) -> JsValue {
    let win = window();
    let root = object_or_create(&win, INLINE_STATE_GLOBAL);
    let key = inline_key_text(net);
    let current = property(&root, &key);
    if current.is_object() && !current.is_null() {
        return current;
    }
    let created = Object::new();
    set_property(&root, &key, created.as_ref());
    created.into()
}

fn instance_state_root() -> JsValue {
    object_or_create(&window(), INSTANCE_STATE_GLOBAL)
}

fn escape_attr(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            other => output.push(other),
        }
    }
    output
}

fn checkbox_html(
    class_name: &str,
    toggle_attr: &str,
    net: &str,
    instance_id: Option<&str>,
    name: &str,
    enabled: bool,
) -> String {
    let checked = if enabled { "checked" } else { "" };
    let label = if enabled {
        "Included in command"
    } else {
        "Excluded from command"
    };
    let instance = instance_id
        .map(|value| format!(" data-instance-id=\"{}\"", escape_attr(value)))
        .unwrap_or_default();
    format!(
        "<input type=\"checkbox\" class=\"{class_name}\" {toggle_attr}=\"{}\" data-net=\"{}\"{instance} {checked} aria-label=\"{label}\" title=\"{label}\">",
        escape_attr(name),
        escape_attr(net)
    )
}

fn record_field_has_text(record: &JsValue, name: &str) -> bool {
    if record.is_null() || record.is_undefined() {
        return false;
    }
    !crate::js_string_owned(&property(record, name))
        .trim()
        .is_empty()
}

fn instance_option_enabled_impl(
    net: &str,
    instance_id: &str,
    name: &str,
    record: &JsValue,
) -> bool {
    let root = instance_state_root();
    let key = instance_key_text(net, instance_id, name);
    let stored = property(&root, &key);
    if !stored.is_undefined() {
        return stored.as_bool() != Some(false);
    }
    if INSTANCE_OPTIONAL_FIELDS.contains(&name) {
        return record_field_has_text(record, name);
    }
    true
}
#[wasm_bindgen(js_name = bridgeCommandInlineStateKeyR7)]
pub fn bridge_command_inline_state_key_r7(net: String) -> String {
    inline_key_text(&net)
}

#[wasm_bindgen(js_name = bridgeCommandInlineStateR7)]
pub fn bridge_command_inline_state_r7(net: String) -> JsValue {
    inline_state(&net)
}

#[wasm_bindgen(js_name = bridgeCommandOptionEnabledR7)]
pub fn bridge_command_option_enabled_r7(net: String, name: String) -> bool {
    let state = inline_state(&net);
    let stored = property(&state, &name);
    let stored = if stored.is_undefined() {
        None
    } else {
        Some(
            stored
                .as_bool()
                .unwrap_or_else(|| crate::js_boolean(&stored)),
        )
    };
    inline_enabled_from_stored(&name, stored)
}

#[wasm_bindgen(js_name = bridgeCommandShouldIncludeR7)]
pub fn bridge_command_should_include_r7(net: String, name: String) -> bool {
    bridge_command_option_enabled_r7(net, name)
}

#[wasm_bindgen(js_name = bridgeCommandSetOptionR7)]
pub fn bridge_command_set_option_r7(net: String, name: String, enabled: bool) -> bool {
    let state = inline_state(&net);
    set_property(&state, &name, &JsValue::from_bool(enabled));
    enabled
}

#[wasm_bindgen(js_name = bridgeCommandToggleOptionR7)]
pub fn bridge_command_toggle_option_r7(net: String, name: String) -> bool {
    let state = inline_state(&net);
    let current = property(&state, &name);
    let enabled = current.as_bool() == Some(false);
    set_property(&state, &name, &JsValue::from_bool(enabled));
    enabled
}

#[wasm_bindgen(js_name = bridgeCommandInlineToggleR7)]
pub fn bridge_command_inline_toggle_r7(net: String, name: String) -> String {
    if managed(&name) || required(&name) {
        return String::new();
    }
    let enabled = bridge_command_option_enabled_r7(net.clone(), name.clone());
    checkbox_html(
        "kgw-command-option-checkbox-r9",
        "data-bridge-command-option-toggle-r7",
        &net,
        None,
        &name,
        enabled,
    )
}
#[wasm_bindgen(js_name = bridgeInstanceCommandStateKeyR13B)]
pub fn bridge_instance_command_state_key_r13b(
    net: String,
    instance_id: JsValue,
    name: String,
) -> String {
    instance_key_text(&net, &crate::js_string_owned(&instance_id), &name)
}

#[wasm_bindgen(js_name = bridgeInstanceCommandOptionEnabledR13B)]
pub fn bridge_instance_command_option_enabled_r13b(
    net: String,
    instance_id: JsValue,
    name: String,
    record: JsValue,
) -> bool {
    instance_option_enabled_impl(&net, &crate::js_string_owned(&instance_id), &name, &record)
}

#[wasm_bindgen(js_name = bridgeInstanceCommandShouldIncludeR13B)]
pub fn bridge_instance_command_should_include_r13b(
    net: String,
    instance_id: JsValue,
    name: String,
    record: JsValue,
) -> bool {
    bridge_instance_command_option_enabled_r13b(net, instance_id, name, record)
}
#[wasm_bindgen(js_name = bridgeInstanceCommandSetOptionR13B)]
pub fn bridge_instance_command_set_option_r13b(
    net: String,
    instance_id: JsValue,
    name: String,
    enabled: bool,
) -> String {
    let instance_id = crate::js_string_owned(&instance_id);
    let key = instance_key_text(&net, &instance_id, &name);
    let root = instance_state_root();
    set_property(&root, &key, &JsValue::from_bool(enabled));
    key
}

#[wasm_bindgen(js_name = bridgeInstanceCommandCheckboxR13B)]
pub fn bridge_instance_command_checkbox_r13b(
    net: String,
    instance_id: JsValue,
    name: String,
    record: JsValue,
) -> String {
    let instance_id = crate::js_string_owned(&instance_id);
    let enabled = instance_option_enabled_impl(&net, &instance_id, &name, &record);
    checkbox_html(
        "kgw-command-option-checkbox-r9 kgw-bridge-instance-command-checkbox-r13b",
        "data-bridge-instance-command-option-toggle-r13b",
        &net,
        Some(&instance_id),
        &name,
        enabled,
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_preserve_legacy_defaults() {
        assert_eq!(inline_key_text(""), "mainnet");
        assert_eq!(inline_key_text("testnet10"), "testnet10");
        assert_eq!(
            instance_key_text("", "", "instanceDiff"),
            "mainnet::1::instanceDiff"
        );
        assert_eq!(
            instance_key_text("mainnet", "two", "instance"),
            "mainnet::two::instance"
        );
    }

    #[test]
    fn inline_policy_matches_schema_contract() {
        assert!(inline_enabled_from_stored("nodeMode", None));
        assert!(!inline_enabled_from_stored("appdir", Some(true)));
        assert!(!inline_enabled_from_stored("config", None));
        assert!(inline_enabled_from_stored("config", Some(true)));
        assert!(inline_enabled_from_stored("promPort", None));
        assert!(!inline_enabled_from_stored("promPort", Some(false)));
    }
    #[test]
    fn checkbox_html_preserves_legacy_shape() {
        let inline = checkbox_html(
            "kgw-command-option-checkbox-r9",
            "data-bridge-command-option-toggle-r7",
            "mainnet",
            None,
            "promPort",
            true,
        );
        assert!(inline.contains("data-bridge-command-option-toggle-r7=\"promPort\""));
        assert!(inline.contains("data-net=\"mainnet\""));
        assert!(inline.contains("checked"));

        let instance = checkbox_html(
            "kgw-command-option-checkbox-r9 kgw-bridge-instance-command-checkbox-r13b",
            "data-bridge-instance-command-option-toggle-r13b",
            "mainnet",
            Some("one"),
            "instanceDiff",
            false,
        );
        assert!(instance.contains("data-instance-id=\"one\""));
        assert!(instance.contains("Excluded from command"));
    }
}
