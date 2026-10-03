use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

const STATE_GLOBAL: &str = "__KGW_EXPLORER_STATE_V1";
#[cfg(test)]
const STATE_KEYS: [&str; 5] = [
    "rows",
    "filteredRows",
    "selectedAddress",
    "busy",
    "cancelRequested",
];

fn global() -> JsValue {
    js_sys::global().into()
}

fn is_present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if !is_present(target) {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn set_property(target: &JsValue, name: &str, value: &JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(name), value);
}

fn window() -> JsValue {
    let global = global();
    let candidate = property(&global, "window");
    if is_present(&candidate) {
        candidate
    } else {
        global
    }
}

fn default_state() -> JsValue {
    let state = Object::new();
    set_property(state.as_ref(), "rows", Array::new().as_ref());
    set_property(state.as_ref(), "filteredRows", Array::new().as_ref());
    set_property(state.as_ref(), "selectedAddress", &JsValue::from_str(""));
    set_property(state.as_ref(), "busy", &JsValue::FALSE);
    set_property(state.as_ref(), "cancelRequested", &JsValue::FALSE);
    state.into()
}

fn ensure_default(target: &JsValue, name: &str, value: &JsValue) {
    if !is_present(&property(target, name)) {
        set_property(target, name, value);
    }
}

fn normalize_state(target: &JsValue) {
    ensure_default(target, "rows", Array::new().as_ref());
    ensure_default(target, "filteredRows", Array::new().as_ref());

    ensure_default(target, "selectedAddress", &JsValue::from_str(""));
    ensure_default(target, "busy", &JsValue::FALSE);
    ensure_default(target, "cancelRequested", &JsValue::FALSE);
}

pub(crate) fn explorer_state() -> JsValue {
    let window = window();
    let existing = property(&window, STATE_GLOBAL);
    if existing.is_object() && !existing.is_null() {
        normalize_state(&existing);
        return existing;
    }

    let state = default_state();
    set_property(&window, STATE_GLOBAL, &state);
    state
}

#[wasm_bindgen(js_name = explorerEnsureState)]
pub fn explorer_ensure_state() -> JsValue {
    explorer_state()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_contract_keys_match_legacy_shape() {
        assert_eq!(
            STATE_KEYS,
            [
                "rows",
                "filteredRows",
                "selectedAddress",
                "busy",
                "cancelRequested",
            ]
        );
    }

    #[test]
    fn state_global_name_is_versioned_and_explorer_scoped() {
        assert!(STATE_GLOBAL.starts_with("__KGW_EXPLORER_STATE"));
        assert!(STATE_GLOBAL.ends_with("_V1"));
    }
}
