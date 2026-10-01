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

fn required_function(target: &JsValue, name: &str) -> Result<Function, JsValue> {
    function(target, name).ok_or_else(|| {
        JsValue::from_str(&format!(
            "Bridge instance UI callback is unavailable: {name}"
        ))
    })
}

fn call1_required(target: &JsValue, name: &str, first: &JsValue) -> Result<JsValue, JsValue> {
    required_function(target, name)?.call1(target, first)
}

fn call2_required(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    required_function(target, name)?.call2(target, first, second)
}

fn document() -> JsValue {
    let global: JsValue = js_sys::global().into();
    property(&property(&global, "window"), "document")
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

fn escaped_text(value: &str) -> String {
    bridge_frontend_helpers::bridge_escape_html(JsValue::from_str(value))
}

fn instance_field(instance: &JsValue, name: &str, fallback: &str) -> String {
    let value = crate::js_string_owned(&property(instance, name));
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value
    }
}

fn selected(current: &str, expected: &str) -> &'static str {
    if current == expected { "selected" } else { "" }
}

fn instance_checkbox(
    bridge_instances: &JsValue,
    net: &str,
    instance_id: &JsValue,
    name: &str,
) -> String {
    bridge_command_options::bridge_instance_command_checkbox_from_instances_r13b(
        bridge_instances.clone(),
        net.to_owned(),
        instance_id.clone(),
        name.to_owned(),
    )
}

#[wasm_bindgen(js_name = bridgeRenderInstancesUi)]
pub fn bridge_render_instances_ui(
    net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
) -> Result<String, JsValue> {
    let net = bridge_frontend_helpers::bridge_instance_network_key_r15(
        JsValue::from_str(&net),
        JsValue::from_str(&net),
    );
    if net != "mainnet" {
        return Ok(String::new());
    }

    bridge_port_orchestration::bridge_ensure_instance_state(
        bridge_instances.clone(),
        active_instance.clone(),
        net.clone(),
    )?;

    let values = property(&bridge_instances, &net);
    let instances = if Array::is_array(&values) {
        Array::from(&values)
    } else {
        Array::new()
    };
    let active_id = crate::js_string_owned(&property(&active_instance, &net));
    let count = instances.length();

    let mut tabs = String::new();
    let mut panels = String::new();

    for (index, instance) in instances.iter().enumerate() {
        let id_value = property(&instance, "id");
        let id = crate::js_string_owned(&id_value);
        let id_escaped = escaped_text(&id);
        let is_active = active_id == id || (active_id.is_empty() && index == 0);
        let active_class = if is_active { "active" } else { "" };
        let disabled = if count <= 1 { "disabled" } else { "" };
        let number = index + 1;

        tabs.push_str(&format!(
            r#"
        <span class="kgw-instance-tab">
          <button type="button"
            class="bridge-v7-instance-pill-r7b bridge-v7-instance-pill-r11 {active_class}"
            data-bridge-action="select-instance" data-network="{net}" data-instance-id="{id_escaped}">
            Instance {number}
          </button>
          <button type="button" class="bridge-v7-instance-trash-r11"
            data-bridge-action="remove-instance" data-network="{net}" data-instance-id="{id_escaped}"
            title="Delete Instance {number}" aria-label="Delete Instance {number}"
            {disabled}>Delete</button>
        </span>"#
        ));

        let preview = escaped_text(&preview_text(&net, &instance));
        let instance_port = escaped_text(&instance_field(&instance, "instancePort", ""));
        let instance_diff = escaped_text(&instance_field(&instance, "instanceDiff", "2048"));
        let instance_prom = escaped_text(&instance_field(&instance, "instanceProm", ""));
        let block_wait = escaped_text(&instance_field(&instance, "instanceBlockWaitTime", ""));
        let extranonce = escaped_text(&instance_field(&instance, "instanceExtranonceSize", ""));
        let shares_per_min = escaped_text(&instance_field(&instance, "instanceSharesPerMin", ""));
        let port_placeholder = escaped_text(&placeholder_value(&net, "stratum"));
        let prom_placeholder = escaped_text(&placeholder_value(&net, "prom"));
        let diff_attrs =
            crate::bridge_render::bridge_difficulty_input_attrs_r16c("instanceDiff".to_owned());
        let shares_attrs = crate::bridge_render::bridge_difficulty_input_attrs_r16c(
            "instanceSharesPerMin".to_owned(),
        );
        let log_to_file = instance_field(&instance, "instanceLogToFile", "");
        let var_diff = instance_field(&instance, "instanceVarDiff", "");
        let var_diff_stats = instance_field(&instance, "instanceVarDiffStats", "");
        let pow2_clamp = instance_field(&instance, "instancePow2Clamp", "");

        let checkbox = |name: &str| instance_checkbox(&bridge_instances, &net, &id_value, name);
        let element_id = |name: &str| {
            bridge_frontend_helpers::bridge_element_id(net.clone(), format!("{name}-{id}"))
        };

        panels.push_str(&format!(
            r#"
        <section
          class="bridge-v7-instance-panel bridge-v7-instance-panel-r7b {active_class}"
          data-bridge-instance-panel="{id_escaped}">
          <label class="bridge-v7-card bridge-v7-instance-preview-card-r8b">
            <span class="kgw-command-option-title-row-r8e">
              {instance_checkbox}
              <span class="kgw-command-option-title-text-r8e">Effective instance</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input
              readonly
              data-bridge-instance-preview="true"
              data-network="{net}"
              data-instance-id="{id_escaped}"
              value="{preview}"
              title="{preview}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {port_checkbox}
              <span class="kgw-command-option-title-text-r8e">port</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="{port_id}" data-bridge-instance-field="instancePort" value="{instance_port}" placeholder="{port_placeholder}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {diff_checkbox}
              <span class="kgw-command-option-title-text-r8e">diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="{diff_id}" data-bridge-instance-field="instanceDiff" value="{instance_diff}" placeholder="2048" {diff_attrs} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {prom_checkbox}
              <span class="kgw-command-option-title-text-r8e">prom</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="{prom_id}" data-bridge-instance-field="instanceProm" value="{instance_prom}" placeholder="{prom_placeholder}" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {log_checkbox}
              <span class="kgw-command-option-title-text-r8e">log</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="{log_id}" data-bridge-instance-field="instanceLogToFile">
              <option value="not set" {log_not_set}>Inherit global</option>
              <option value="false" {log_false}>false</option>
              <option value="true" {log_true}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {wait_checkbox}
              <span class="kgw-command-option-title-text-r8e">wait</span>
            </span>
            <input id="{wait_id}" data-bridge-instance-field="instanceBlockWaitTime" value="{block_wait}" placeholder="Enable to override global milliseconds" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {extranonce_checkbox}
              <span class="kgw-command-option-title-text-r8e">extranonce</span>
            </span>
            <input id="{extranonce_id}" data-bridge-instance-field="instanceExtranonceSize" value="{extranonce}" placeholder="optional" />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {var_diff_checkbox}
              <span class="kgw-command-option-title-text-r8e">var_diff</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="{var_diff_id}" data-bridge-instance-field="instanceVarDiff">
              <option value="not set" {var_diff_not_set}>Inherit global</option>
              <option value="false" {var_diff_false}>false</option>
              <option value="true" {var_diff_true}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {var_stats_checkbox}
              <span class="kgw-command-option-title-text-r8e">var_stats</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="{var_stats_id}" data-bridge-instance-field="instanceVarDiffStats">
              <option value="not set" {var_stats_not_set}>Inherit global</option>
              <option value="false" {var_stats_false}>false</option>
              <option value="true" {var_stats_true}>true</option>
            </select>
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {shares_checkbox}
              <span class="kgw-command-option-title-text-r8e">shares/min</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <input id="{shares_id}" data-bridge-instance-field="instanceSharesPerMin" value="{shares_per_min}" placeholder="optional" {shares_attrs} />
          </label>

          <label class="bridge-v7-card bridge-v7-instance-card-r7b">
            <span class="kgw-command-option-title-row-r8e">
              {pow2_checkbox}
              <span class="kgw-command-option-title-text-r8e">pow2</span>
            </span> <!-- KGW_BRIDGE_RENDER_INSTANCES_COMMAND_CHECKBOX_R13B -->
            <select id="{pow2_id}" data-bridge-instance-field="instancePow2Clamp">
              <option value="not set" {pow2_not_set}>Inherit global</option>
              <option value="false" {pow2_false}>false</option>
              <option value="true" {pow2_true}>true</option>
            </select>
          </label>
        </section>
"#,
            instance_checkbox = checkbox("instance"),
            port_checkbox = checkbox("instancePort"),
            diff_checkbox = checkbox("instanceDiff"),
            prom_checkbox = checkbox("instanceProm"),
            log_checkbox = checkbox("instanceLogToFile"),
            wait_checkbox = checkbox("instanceBlockWaitTime"),
            extranonce_checkbox = checkbox("instanceExtranonceSize"),
            var_diff_checkbox = checkbox("instanceVarDiff"),
            var_stats_checkbox = checkbox("instanceVarDiffStats"),
            shares_checkbox = checkbox("instanceSharesPerMin"),
            pow2_checkbox = checkbox("instancePow2Clamp"),
            port_id = element_id("instancePort"),
            diff_id = element_id("instanceDiff"),
            prom_id = element_id("instanceProm"),
            log_id = element_id("instanceLogToFile"),
            wait_id = element_id("instanceBlockWaitTime"),
            extranonce_id = element_id("instanceExtranonceSize"),
            var_diff_id = element_id("instanceVarDiff"),
            var_stats_id = element_id("instanceVarDiffStats"),
            shares_id = element_id("instanceSharesPerMin"),
            pow2_id = element_id("instancePow2Clamp"),
            log_not_set = selected(&log_to_file, "not set"),
            log_false = selected(&log_to_file, "false"),
            log_true = selected(&log_to_file, "true"),
            var_diff_not_set = selected(&var_diff, "not set"),
            var_diff_false = selected(&var_diff, "false"),
            var_diff_true = selected(&var_diff, "true"),
            var_stats_not_set = selected(&var_diff_stats, "not set"),
            var_stats_false = selected(&var_diff_stats, "false"),
            var_stats_true = selected(&var_diff_stats, "true"),
            pow2_not_set = selected(&pow2_clamp, "not set"),
            pow2_false = selected(&pow2_clamp, "false"),
            pow2_true = selected(&pow2_clamp, "true"),
        ));
    }

    Ok(format!(
        r#"
    <div class="bridge-v7-instance-tabs bridge-v7-instance-tabs-r7b">
      {tabs}
      <button
        type="button"
        class="bridge-v7-instance-add bridge-v7-instance-add-r7b bridge-v7-instance-add-r11"
        data-bridge-action="add-instance"
        data-network="{net}"
        aria-label="Add Instance"
        title="Add Instance">+</button>
    </div>

    <div class="bridge-v7-instance-stack bridge-v7-instance-stack-r7b">
      {panels}
    </div>"#
    ))
}

#[wasm_bindgen(js_name = bridgeRefreshInstancesUi)]
pub fn bridge_refresh_instances_ui(
    net: String,
    bridge_instances: JsValue,
    active_instance: JsValue,
    callbacks: JsValue,
) -> Result<String, JsValue> {
    let net = bridge_frontend_helpers::bridge_instance_network_key_r15(
        JsValue::from_str(&net),
        JsValue::from_str(&net),
    );
    if net.is_empty() {
        return Ok(net);
    }

    let container_id =
        bridge_frontend_helpers::bridge_element_id(net.clone(), "instances".to_owned());
    let mut container = bridge_frontend_helpers::bridge_by_id(container_id.clone());
    if !present(&container) {
        let selector = format!(
            r#"[data-bridge-network-panel="{net}"] [data-bridge-section-panel="instances"]"#
        );
        container = call1(&document(), "querySelector", &JsValue::from_str(&selector))
            .unwrap_or(JsValue::UNDEFINED);
    }

    if present(&container) {
        if crate::js_string_owned(&property(&container, "id")).is_empty() {
            set(&container, "id", &JsValue::from_str(&container_id));
        }
        let html = bridge_render_instances_ui(
            net.clone(),
            bridge_instances.clone(),
            active_instance.clone(),
        )?;
        set(&container, "innerHTML", &JsValue::from_str(&html));
        let _ = call1_required(&callbacks, "decorateSettingsFields", &container)?;
        let _ = call2_required(
            &callbacks,
            "installInstanceContainerOwner",
            &container,
            &JsValue::from_str(&net),
        )?;
    }

    let _ = call1_required(&callbacks, "updateCommand", &JsValue::from_str(&net))?;
    Ok(net)
}

#[wasm_bindgen(js_name = bridgeInstallAllVisibleInstanceContainerOwnersR11)]
pub fn bridge_install_all_visible_instance_container_owners_r11(
    callbacks: JsValue,
) -> Result<u32, JsValue> {
    let mut installed = 0_u32;
    for net in ["mainnet", "testnet10", "testnet13"] {
        let container_id =
            bridge_frontend_helpers::bridge_element_id(net.to_owned(), "instances".to_owned());
        let container = bridge_frontend_helpers::bridge_by_id(container_id);
        if !present(&container) {
            continue;
        }
        let _ = call2_required(
            &callbacks,
            "installInstanceContainerOwner",
            &container,
            &JsValue::from_str(net),
        )?;
        installed += 1;
    }
    Ok(installed)
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
