use js_sys::{Array, Function, JSON, Reflect};
use wasm_bindgen::{JsCast, prelude::*};

use crate::{
    bridge_command_options, bridge_frontend_helpers, bridge_port_core, bridge_port_orchestration,
};

const EXCLUDED_TEXT: &str = "Excluded from runtime.";
const WAITING_TEXT: &str = "Waiting for validated effective settings.";
const MISSING_EFFECTIVE_TEXT: &str = "No effective instance in this configuration.";

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
fn call1(target: &JsValue, name: &str, first: &JsValue) -> Option<JsValue> {
    function(target, name)?.call1(target, first).ok()
}

fn query_all(root: &JsValue, selector: &str) -> Vec<JsValue> {
    let Some(list) = call1(root, "querySelectorAll", &JsValue::from_str(selector)) else {
        return Vec::new();
    };
    let length = property(&list, "length").as_f64().unwrap_or(0.0).max(0.0) as u32;
    (0..length)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .collect()
}

fn command_preview_effective_settings(net: &str) -> String {
    let id =
        bridge_frontend_helpers::bridge_element_id(net.to_owned(), "commandPreview".to_owned());
    let preview = bridge_frontend_helpers::bridge_by_id(id);
    let dataset = property(&preview, "dataset");
    crate::js_string_owned(&property(&dataset, "effectiveSettings"))
}

fn find_effective_instance(raw: &str, instance_id: &str) -> Option<JsValue> {
    let parsed = JSON::parse(raw).ok()?;
    let settings = property(&parsed, "effectiveBridgeSettings");
    let instances = property(&settings, "instances");
    if !Array::is_array(&instances) {
        return None;
    }
    Array::from(&instances)
        .iter()
        .find(|item| crate::js_string_owned(&property(item, "instanceId")) == instance_id)
}

fn preview_text(net: &str, instance: &JsValue) -> String {
    let instance_id = property(instance, "id");
    if !bridge_command_options::bridge_instance_command_should_include_r13b(
        net.to_owned(),
        instance_id.clone(),
        "instance".to_owned(),
        instance.clone(),
    ) {
        return EXCLUDED_TEXT.to_owned();
    }

    let raw = command_preview_effective_settings(net);
    if raw.is_empty() {
        return WAITING_TEXT.to_owned();
    }

    let Some(resolved) = find_effective_instance(&raw, &crate::js_string_owned(&instance_id))
    else {
        return if JSON::parse(&raw).is_ok() {
            MISSING_EFFECTIVE_TEXT.to_owned()
        } else {
            WAITING_TEXT.to_owned()
        };
    };

    JSON::stringify(&resolved)
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| MISSING_EFFECTIVE_TEXT.to_owned())
}
fn find_instance(bridge_instances: &JsValue, net: &str, instance_id: &str) -> Option<JsValue> {
    let values = property(bridge_instances, net);
    if !Array::is_array(&values) {
        return None;
    }
    Array::from(&values)
        .iter()
        .find(|item| crate::js_string_owned(&property(item, "id")) == instance_id)
}

#[wasm_bindgen(js_name = bridgeInstancePreviewTextR8B)]
pub fn bridge_instance_preview_text_r8b(net: String, instance: JsValue) -> String {
    preview_text(&net, &instance)
}

#[wasm_bindgen(js_name = bridgeSyncInstancePreviewRowsR8B)]
pub fn bridge_sync_instance_preview_rows_r8b(
    net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> Result<(), JsValue> {
    let root = bridge_frontend_helpers::bridge_by_id("kaspa-bridge".to_owned());
    if !present(&root) {
        return Ok(());
    }

    bridge_port_orchestration::bridge_ensure_instance_state(
        bridge_instances.clone(),
        active_instance,
        net.clone(),
    )?;
    let selector = format!(r#"[data-bridge-instance-preview][data-network="{net}"]"#);
    for preview in query_all(&root, &selector) {
        let dataset = property(&preview, "dataset");
        let instance_id = crate::js_string_owned(&property(&dataset, "instanceId"));
        let text = find_instance(&bridge_instances, &net, &instance_id)
            .map(|instance| preview_text(&net, &instance))
            .unwrap_or_else(|| "--instance=".to_owned());

        set(&preview, "value", &JsValue::from_str(&text));
        set(&preview, "textContent", &JsValue::from_str(&text));
        set(&preview, "title", &JsValue::from_str(&text));
    }
    Ok(())
}

#[wasm_bindgen(js_name = bridgeReadInstanceField)]
pub fn bridge_read_instance_field(net: String, instance_id: JsValue, field_name: String) -> String {
    let id = bridge_frontend_helpers::bridge_element_id(
        net,
        format!("{}-{}", field_name, crate::js_string_owned(&instance_id)),
    );
    let element = bridge_frontend_helpers::bridge_by_id(id);
    if !present(&element) {
        return String::new();
    }
    if crate::js_string_owned(&property(&element, "type")) == "checkbox" {
        return if crate::js_boolean(&property(&element, "checked")) {
            "true".to_owned()
        } else {
            String::new()
        };
    }
    crate::js_string_owned(&property(&element, "value"))
        .trim()
        .to_owned()
}

fn placeholder_fallback(section: &str) -> &'static str {
    match section {
        "prom" => "2113",
        _ => "5556",
    }
}

fn placeholder_value(net: &str, section: &str) -> String {
    let profile = bridge_port_core::bridge_port_profile_r35b(net.to_owned());
    let group = property(&profile, section);
    let value = property(&group, "instanceStart");
    if crate::js_boolean(&value) {
        crate::js_string_owned(&value)
    } else {
        placeholder_fallback(section).to_owned()
    }
}

#[wasm_bindgen(js_name = bridgeInstancePortPlaceholderR49)]
pub fn bridge_instance_port_placeholder_r49(net: String) -> String {
    placeholder_value(&net, "stratum")
}

#[wasm_bindgen(js_name = bridgeInstancePromPlaceholderR49)]
pub fn bridge_instance_prom_placeholder_r49(net: String) -> String {
    placeholder_value(&net, "prom")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_copy_matches_legacy_contract() {
        assert_eq!(EXCLUDED_TEXT, "Excluded from runtime.");
        assert_eq!(WAITING_TEXT, "Waiting for validated effective settings.");
        assert_eq!(
            MISSING_EFFECTIVE_TEXT,
            "No effective instance in this configuration."
        );
    }

    #[test]
    fn placeholder_fallbacks_match_legacy_contract() {
        assert_eq!(placeholder_fallback("stratum"), "5556");
        assert_eq!(placeholder_fallback("prom"), "2113");
        assert_eq!(placeholder_fallback("unknown"), "5556");
    }
}
