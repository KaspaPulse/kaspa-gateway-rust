//! Runtime presentation is derived from observed state, never from startup claims.
use super::{get_property, js_boolean, set_property};
use js_sys::{Function, JsString, Number, Object, Reflect, RegExp, Symbol, TypeError};
use wasm_bindgen::{JsCast, prelude::*};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Object)]
    fn boxed(value: &JsValue) -> Object;
    #[wasm_bindgen(catch, js_name = String)]
    fn string_value(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch, js_name = Number)]
    fn number_value(value: &JsValue) -> Result<f64, JsValue>;
}

fn property(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    if target.is_null() || target.is_undefined() {
        return Err(TypeError::new("Runtime state cannot be null or undefined").into());
    }
    get_property(boxed(target).as_ref(), name)
}

fn default_property(target: &JsValue, name: &str, default: JsValue) -> Result<JsValue, JsValue> {
    let value = property(target, name)?;
    Ok(if value.is_undefined() { default } else { value })
}

fn equals_text(value: &JsValue, expected: &str) -> bool {
    value.as_string().as_deref() == Some(expected)
}

fn object_like(value: &JsValue) -> bool {
    value.is_object() || value.is_function()
}

fn abstract_string(value: &JsValue) -> Result<String, JsValue> {
    if value.is_symbol() {
        return Err(TypeError::new("Cannot convert a Symbol value to a string").into());
    }
    string_value(value).map(String::from)
}

// JavaScript concatenation uses the default primitive hint, unlike String(object).
fn concatenated_text(value: &JsValue) -> Result<String, JsValue> {
    if !object_like(value) {
        return abstract_string(value);
    }
    let exotic = Reflect::get(value, Symbol::to_primitive().as_ref())?;
    if !exotic.is_null() && !exotic.is_undefined() {
        let function = exotic
            .dyn_ref::<Function>()
            .ok_or_else(|| JsValue::from(TypeError::new("Symbol.toPrimitive must be callable")))?;
        let primitive = function.call1(value, &JsValue::from_str("default"))?;
        if object_like(&primitive) {
            return Err(TypeError::new("Cannot convert object to primitive value").into());
        }
        return abstract_string(&primitive);
    }
    for name in ["valueOf", "toString"] {
        let candidate = get_property(value, name)?;
        if let Some(function) = candidate.dyn_ref::<Function>() {
            let primitive = function.call0(value)?;
            if !object_like(&primitive) {
                return abstract_string(&primitive);
            }
        }
    }
    Err(TypeError::new("Cannot convert object to primitive value").into())
}

fn process_state(transition: &str, failed: bool, running: bool) -> &'static str {
    match transition {
        "starting" => "Starting",
        "stopping" => "Stopping",
        _ if failed => "Failed",
        _ if running => "Running",
        _ => "Stopped",
    }
}

fn network_state(running: bool, synced: Option<bool>) -> &'static str {
    match (running, synced) {
        (false, _) => "Not connected",
        (true, Some(true)) => "Synchronized",
        (true, Some(false)) => "Not synchronized",
        (true, None) => "Synchronization not reported",
    }
}

#[wasm_bindgen(js_name = runtimePresentation)]
pub fn runtime_presentation(state: JsValue) -> Result<JsValue, JsValue> {
    // Preserve destructuring order and undefined-only defaults from the public ABI.
    let role = default_property(&state, "role", JsValue::from_str("Node"))?;
    let enabled = default_property(&state, "enabled", JsValue::TRUE)?;
    let running = default_property(&state, "running", JsValue::FALSE)?;
    let transition = default_property(&state, "transition", JsValue::from_str(""))?;
    let error = default_property(&state, "error", JsValue::from_str(""))?;
    let synced = property(&state, "synced")?;
    let running = js_boolean(&running);
    let process = process_state(
        transition.as_string().as_deref().unwrap_or(""),
        js_boolean(&error),
        running,
    );
    let result = Object::new();
    set_property(result.as_ref(), "application", &JsValue::from_str("Ready"))?;
    set_property(
        result.as_ref(),
        "profile",
        &JsValue::from_str(if js_boolean(&enabled) {
            "Enabled"
        } else {
            "Disabled"
        }),
    )?;
    set_property(result.as_ref(), "process", &JsValue::from_str(process))?;
    set_property(
        result.as_ref(),
        "processLabel",
        &JsValue::from_str(&format!("{}: {process}", concatenated_text(&role)?)),
    )?;
    set_property(
        result.as_ref(),
        "network",
        &JsValue::from_str(network_state(running, synced.as_bool())),
    )?;
    Ok(result.into())
}

fn decimal_property(target: &JsValue, name: &str) -> Result<bool, JsValue> {
    let value = property(target, name)?;
    let value = if js_boolean(&value) {
        value
    } else {
        JsValue::from_str("")
    };
    // Retain ECMAScript anchor/coercion behavior and never round integer counters.
    Ok(RegExp::new(r"^\d+$", "").test(&abstract_string(&value)?))
}

fn cpu_activity(fresh: bool, enabled: bool, rate: f64, zero_hashes: bool) -> &'static str {
    if !fresh {
        "Not reported"
    } else if !enabled {
        "Disabled"
    } else if rate.is_finite() && rate > 0.0 {
        "Hashing"
    } else if zero_hashes {
        "Waiting for work"
    } else {
        "No recent hashing reported"
    }
}

#[wasm_bindgen(js_name = runtimeObservationSummary)]
pub fn runtime_observation_summary(
    fields: JsValue,
    running: JsValue,
    cpu_only: JsValue,
) -> Result<String, JsValue> {
    let fields = if fields.is_undefined() {
        Object::new().into()
    } else {
        fields
    };
    let fresh =
        js_boolean(&running) && equals_text(&property(&fields, "observation_state")?, "fresh");
    let rpc = if !fresh {
        "Unknown"
    } else if equals_text(&property(&fields, "rpc_ready")?, "true") {
        "Available"
    } else {
        "Unavailable"
    };
    let sync = if !fresh || equals_text(&property(&fields, "synced")?, "unknown") {
        "Not reported"
    } else if equals_text(&property(&fields, "synced")?, "true") {
        "Synchronized"
    } else if equals_text(&property(&fields, "synced")?, "false") {
        "Not synchronized"
    } else {
        "Not reported"
    };
    let mut text = format!("RPC: {rpc} | Sync: {sync}");
    if fresh && decimal_property(&fields, "virtual_daa_score")? {
        text.push_str(&format!(
            " | DAA: {}",
            concatenated_text(&property(&fields, "virtual_daa_score")?)?
        ));
    }
    if js_boolean(&cpu_only) {
        let enabled = fresh && equals_text(&property(&fields, "cpu_enabled")?, "true");
        let rate = if enabled && !equals_text(&property(&fields, "cpu_hashrate_hs")?, "unknown") {
            number_value(&property(&fields, "cpu_hashrate_hs")?)?
        } else {
            f64::NAN
        };
        let zero_hashes = fresh
            && enabled
            && !(rate.is_finite() && rate > 0.0)
            && equals_text(&property(&fields, "cpu_hashes_tried")?, "0");
        text.push_str(&format!(
            " | CPU: {}",
            cpu_activity(fresh, enabled, rate, zero_hashes)
        ));
        if enabled {
            for (field, label) in [
                ("cpu_hashes_tried", "Hashes"),
                ("cpu_blocks_submitted", "Submitted blocks"),
                ("cpu_blocks_confirmed_blue", "Confirmed blue blocks"),
            ] {
                if decimal_property(&fields, field)? {
                    text.push_str(&format!(
                        " | {label}: {}",
                        concatenated_text(&property(&fields, field)?)?
                    ));
                }
            }
            if rate.is_finite() && rate >= 0.0 {
                let fixed = String::from(Number::from(rate).to_fixed(2)?);
                text.push_str(&format!(" | {fixed} H/s"));
            }
        }
    }
    if fresh
        && js_boolean(&property(&fields, "observation_error")?)
        && !equals_text(&property(&fields, "observation_error")?, "none")
    {
        text.push_str(&format!(
            " | RPC error: {}",
            concatenated_text(&property(&fields, "observation_error")?)?
        ));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_have_priority_over_running_and_error() {
        assert_eq!(process_state("starting", true, true), "Starting");
        assert_eq!(process_state("stopping", true, true), "Stopping");
        assert_eq!(process_state("", true, true), "Failed");
        assert_eq!(process_state("", false, true), "Running");
        assert_eq!(process_state("", false, false), "Stopped");
    }

    #[test]
    fn stopped_process_never_implies_network_connection() {
        for synced in [None, Some(false), Some(true)] {
            assert_eq!(network_state(false, synced), "Not connected");
        }
    }

    #[test]
    fn missing_sync_observation_is_not_a_positive_claim() {
        assert_eq!(network_state(true, None), "Synchronization not reported");
        assert_eq!(network_state(true, Some(false)), "Not synchronized");
        assert_eq!(network_state(true, Some(true)), "Synchronized");
    }

    #[test]
    fn stale_and_disabled_cpu_states_override_old_hashrate() {
        assert_eq!(cpu_activity(false, true, 12.5, false), "Not reported");
        assert_eq!(cpu_activity(true, false, 12.5, false), "Disabled");
    }

    #[test]
    fn hashing_requires_positive_finite_observed_rate() {
        assert_eq!(cpu_activity(true, true, 0.01, true), "Hashing");
        for rate in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(cpu_activity(true, true, rate, true), "Waiting for work");
            assert_eq!(
                cpu_activity(true, true, rate, false),
                "No recent hashing reported"
            );
        }
    }
}
