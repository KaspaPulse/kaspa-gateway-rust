use js_sys::{Array, Date, Function, JSON, Object, Reflect};
use std::cell::Cell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

thread_local! {
    static FONT_SIZE: Cell<u32> = const { Cell::new(11) };
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

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn call2(target: &JsValue, name: &str, a: &JsValue, b: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call2(target, a, b).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn call3(target: &JsValue, name: &str, a: &JsValue, b: &JsValue, c: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call3(target, a, b, c).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn root() -> JsValue {
    let doc = document();
    let direct = call1(&doc, "getElementById", &JsValue::from_str("explorer"));
    if present(&direct) {
        return direct;
    }
    let legacy = call1(
        &doc,
        "querySelector",
        &JsValue::from_str(".explorer-python-root"),
    );
    if present(&legacy) { legacy } else { doc }
}

fn query(scope: &JsValue, selector: &str) -> JsValue {
    call1(scope, "querySelector", &JsValue::from_str(selector))
}

fn query_all(scope: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(scope, "querySelectorAll", &JsValue::from_str(selector));
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

fn style_property(node: &JsValue, name: &str, value: &str, priority: &str) {
    let style = property(node, "style");
    let _ = call3(
        &style,
        "setProperty",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
        &JsValue::from_str(priority),
    );
}

fn remove_attr(node: &JsValue, name: &str) {
    let _ = call1(node, "removeAttribute", &JsValue::from_str(name));
}

fn set_attr(node: &JsValue, name: &str, value: &str) {
    let _ = call2(
        node,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn class_add(node: &JsValue, class: &str) {
    let _ = call1(
        &property(node, "classList"),
        "add",
        &JsValue::from_str(class),
    );
}

fn class_remove(node: &JsValue, class: &str) {
    let _ = call1(
        &property(node, "classList"),
        "remove",
        &JsValue::from_str(class),
    );
}

fn normalize_font_size_text(value: &str) -> u32 {
    let digits: String = crate::to_english_digits_text(value)
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .take(2)
        .collect();
    let parsed = digits.parse::<u32>().unwrap_or(9);
    parsed.clamp(6, 24)
}

fn normalize_font_size(value: &JsValue) -> u32 {
    let raw = if value.is_null() || value.is_undefined() {
        "9".to_owned()
    } else {
        crate::js_string_owned(value)
    };
    normalize_font_size_text(&raw)
}

fn current_font_size() -> u32 {
    FONT_SIZE.with(Cell::get).clamp(6, 24)
}

fn microscope_font_log(section: &JsValue, table: &JsValue, size: u32) {
    let style = property(section, "style");
    let css_variable = crate::js_string_owned(&call1(
        &style,
        "getPropertyValue",
        &JsValue::from_str("--explorer-table-font-size"),
    ));
    let computed = call1(&window(), "getComputedStyle", table);
    let table_font_size = crate::js_string_owned(&property(&computed, "fontSize"));
    let payload = Object::new();
    for (key, value) in [
        ("stage", JsValue::from_str("TABLE FONT SIZE APPLIED")),
        (
            "at",
            JsValue::from_str(&String::from(Date::new_0().to_iso_string())),
        ),
        ("size", JsValue::from_f64(size as f64)),
        ("cssVariable", JsValue::from_str(&css_variable)),
        ("tableFontSize", JsValue::from_str(&table_font_size)),
    ] {
        set(payload.as_ref(), key, &value);
    }
    let encoded = JSON::stringify(payload.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    let console = property(&global(), "console");
    if let Some(info) = function(&console, "info") {
        let _ = info.call1(
            &console,
            &JsValue::from_str(&format!(
                "[KGW][microscope][explorer] TABLE FONT SIZE APPLIED :: {encoded}"
            )),
        );
    }
}

fn set_table_font_size_impl(section: &JsValue) -> u32 {
    if !present(section) {
        return current_font_size();
    }
    let size = current_font_size();
    let input = query(section, "#explorerTableFontSize");
    if present(&input) {
        set(&input, "value", &JsValue::from_str(&size.to_string()));
    }
    let table = query(section, "#explorerTransactionsTable");
    if !present(&table) {
        return size;
    }
    style_property(
        section,
        "--explorer-table-font-size",
        &format!("{size}px"),
        "",
    );
    style_property(&table, "font-size", &format!("{size}px"), "important");
    for cell in query_all(&table, "th, td") {
        style_property(&cell, "font-size", &format!("{size}px"), "important");
    }
    microscope_font_log(section, &table, size);
    size
}

fn apply_font_size_impl(section: &JsValue, raw_value: &JsValue) -> u32 {
    let size = normalize_font_size(raw_value);
    FONT_SIZE.with(|state| state.set(size));
    set_table_font_size_impl(section)
}

fn bind_font_spinbox_impl(section: &JsValue) -> bool {
    if !present(section) {
        return false;
    }
    let input = query(section, "#explorerTableFontSize");
    if !present(&input) {
        return false;
    }
    let data = property(&input, "dataset");
    if crate::js_string_owned(&property(&data, "kgwFontBound")) == "1" {
        return false;
    }
    set(&data, "kgwFontBound", &JsValue::from_str("1"));

    for event_name in ["input", "blur"] {
        let section = section.clone();
        let input_for_callback = input.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let raw = property(&input_for_callback, "value");
            apply_font_size_impl(&section, &raw);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &input,
            "addEventListener",
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    let dec = query(section, "#explorerTableFontDecrease");
    if present(&dec) {
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let next = current_font_size().saturating_sub(1);
            apply_font_size_impl(&section, &JsValue::from_f64(next as f64));
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &dec,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    let inc = query(section, "#explorerTableFontIncrease");
    if present(&inc) {
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let next = current_font_size().saturating_add(1);
            apply_font_size_impl(&section, &JsValue::from_f64(next as f64));
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &inc,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }
    true
}

fn control_key(control: &JsValue) -> String {
    let mut parts = Vec::new();
    for value in [
        property(control, "id"),
        property(control, "name"),
        property(control, "className"),
        call1(control, "getAttribute", &JsValue::from_str("aria-label")),
        call1(control, "getAttribute", &JsValue::from_str("title")),
        property(control, "textContent"),
    ] {
        let text = crate::js_string_owned(&value);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join(" ").to_lowercase()
}

fn busy_allowed_text(id: &str, key: &str) -> bool {
    id == "explorerCancel"
        || key.contains("cancel")
        || key.contains("إلغاء")
        || key.contains("الغاء")
}

fn busy_allowed(control: &JsValue) -> bool {
    if !present(control) {
        return false;
    }
    let id = crate::js_string_owned(&property(control, "id"));
    busy_allowed_text(&id, &control_key(control))
}

fn set_control_locked(control: &JsValue, locked: bool) {
    if !present(control) {
        return;
    }
    let can_disable = Reflect::has(control, &JsValue::from_str("disabled")).unwrap_or(false);
    let data = property(control, "dataset");
    let previous = property(&data, "kgwPrevDisabled");

    if locked {
        if previous.is_undefined() {
            let was_disabled = can_disable && crate::js_boolean(&property(control, "disabled"));
            set(
                &data,
                "kgwPrevDisabled",
                &JsValue::from_str(if was_disabled { "true" } else { "false" }),
            );
        }
        if can_disable {
            set(control, "disabled", &JsValue::TRUE);
        }
        set_attr(control, "aria-disabled", "true");
        class_add(control, "disabled");
        style_property(control, "pointer-events", "none", "");
        style_property(control, "cursor", "not-allowed", "");
        return;
    }

    if !previous.is_undefined() {
        if can_disable {
            set(
                control,
                "disabled",
                &JsValue::from_bool(crate::js_string_owned(&previous) == "true"),
            );
        }
        if let Ok(object) = data.dyn_into::<Object>() {
            let _ = Reflect::delete_property(&object, &JsValue::from_str("kgwPrevDisabled"));
        }
    } else if can_disable {
        set(control, "disabled", &JsValue::FALSE);
    }
    remove_attr(control, "aria-disabled");
    class_remove(control, "disabled");
    class_remove(control, "is-disabled");
    style_property(control, "pointer-events", "", "");
    style_property(control, "cursor", "", "");
}

fn custom_busy_event(busy: bool) -> Option<JsValue> {
    let constructor = property(&global(), "CustomEvent")
        .dyn_into::<Function>()
        .ok()?;
    let detail = Object::new();
    set(detail.as_ref(), "busy", &JsValue::from_bool(busy));
    let init = Object::new();
    set(init.as_ref(), "detail", detail.as_ref());
    let args = Array::new();
    args.push(&JsValue::from_str("kgw:explorer-fetch-busy"));
    args.push(init.as_ref());
    Reflect::construct(&constructor, &args).ok()
}

fn apply_busy_controls_impl(section: &JsValue, busy: bool) -> bool {
    let root_node = if present(section) {
        section.clone()
    } else {
        root()
    };
    if !present(&root_node) {
        return false;
    }
    for control in query_all(&root_node, "button,input,select,textarea,a,[role='button']") {
        if busy_allowed(&control) {
            set_control_locked(&control, false);
            if Reflect::has(&control, &JsValue::from_str("disabled")).unwrap_or(false) {
                set(&control, "disabled", &JsValue::from_bool(!busy));
            }
        } else {
            set_control_locked(&control, busy);
        }
    }

    let win = window();
    let previous_busy = crate::js_boolean(&property(&win, "__kgwExplorerFetchBusy"));
    set(&win, "__kgwExplorerFetchBusy", &JsValue::from_bool(busy));
    if previous_busy != busy
        && let Some(event) = custom_busy_event(busy)
    {
        let _ = call1(&win, "dispatchEvent", &event);
    }
    if let Some(policy) = function(&win, "kgwApplyShellExplorerFetchBusyPolicy") {
        let _ = policy.call1(&win, &JsValue::from_bool(busy));
    }
    true
}

#[wasm_bindgen(js_name = explorerSetTableFontSize)]
pub fn explorer_set_table_font_size(section: JsValue) -> u32 {
    set_table_font_size_impl(&section)
}

#[wasm_bindgen(js_name = explorerApplyFontSize)]
pub fn explorer_apply_font_size(section: JsValue, raw_value: JsValue) -> u32 {
    apply_font_size_impl(&section, &raw_value)
}

#[wasm_bindgen(js_name = explorerBindFontSpinbox)]
pub fn explorer_bind_font_spinbox(section: JsValue) -> bool {
    bind_font_spinbox_impl(&section)
}

#[wasm_bindgen(js_name = explorerApplyLocalBusyControls)]
pub fn explorer_apply_local_busy_controls(section: JsValue, busy: bool) -> bool {
    apply_busy_controls_impl(&section, busy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_size_normalization_matches_legacy_contract() {
        assert_eq!(normalize_font_size_text("11"), 11);
        assert_eq!(normalize_font_size_text("٢٥"), 24);
        assert_eq!(normalize_font_size_text("۵"), 6);
        assert_eq!(normalize_font_size_text("12abc34"), 12);
        assert_eq!(normalize_font_size_text(""), 9);
        assert_eq!(normalize_font_size_text("100"), 10);
    }

    #[test]
    fn busy_allowed_policy_preserves_cancel_variants() {
        assert!(busy_allowed_text("explorerCancel", ""));
        assert!(busy_allowed_text("", "cancel fetch"));
        assert!(busy_allowed_text("", "إلغاء العملية"));
        assert!(busy_allowed_text("", "الغاء العملية"));
        assert!(!busy_allowed_text("explorerFetch", "fetch transactions"));
    }
}
