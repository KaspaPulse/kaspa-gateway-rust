use js_sys::{Function, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const CONTROL_SELECTOR: &str = concat!(
    "#explorerAddress,#explorerFromDate,#explorerToDate,",
    "#explorerTypeFilter,#explorerDirectionFilter,#explorerSearch,",
    "#explorerFetchButton,#explorerForceFetchButton,#explorerFilterButton,",
    "#explorerResetFilterButton,button,select,input"
);
const ACTION_SELECTOR: &str = "button,input,select,textarea,a,[role='button'],.btn,.button";

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
fn set_attr(node: &JsValue, name: &str, value: &str) {
    let _ = call2(
        node,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn remove_attr(node: &JsValue, name: &str) {
    let _ = call1(node, "removeAttribute", &JsValue::from_str(name));
}

fn console(method: &str, label: &str, details: &JsValue) {
    let console = property(&global(), "console");
    if let Some(log) = function(&console, method) {
        let _ = log.call2(&console, &JsValue::from_str(label), details);
    }
}

fn force_ui_root(section: &JsValue) -> JsValue {
    if crate::js_boolean(section) {
        return section.clone();
    }
    let doc = document();
    let direct = query(&doc, "#explorer");
    if crate::js_boolean(&direct) {
        direct
    } else {
        query(&doc, ".explorer-python-root")
    }
}
fn force_ui_body(section: &JsValue) -> JsValue {
    let scope = force_ui_root(section);
    if present(&scope) {
        let direct = query(&scope, "#explorerTransactionsBody");
        if present(&direct) {
            return direct;
        }
        let body = query(&scope, "tbody");
        if present(&body) {
            return body;
        }
    }
    query(&document(), "#explorerTransactionsBody")
}

fn set_table_message(section: &JsValue, message: &str) -> bool {
    let body = force_ui_body(section);
    if !present(&body) {
        let details = Object::new();
        set(details.as_ref(), "message", &JsValue::from_str(message));
        console(
            "warn",
            "[KGW Explorer][force-ui] tbody not found for message",
            details.as_ref(),
        );
        return false;
    }
    let escaped = crate::html_escape_text(message);
    set(
        &body,
        "innerHTML",
        &JsValue::from_str(&format!(
            "<tr><td colspan=\"6\" class=\"muted\">{escaped}</td></tr>"
        )),
    );

    let details = Object::new();
    set(details.as_ref(), "message", &JsValue::from_str(message));
    set(
        details.as_ref(),
        "childRows",
        &property(&property(&body, "children"), "length"),
    );
    console(
        "log",
        "[KGW Explorer][force-ui] table message",
        details.as_ref(),
    );
    true
}

fn reset_display_filters(section: &JsValue) -> bool {
    let scope = force_ui_root(section);
    if !present(&scope) {
        return false;
    }
    let _ = crate::explorer_filters::explorer_repair_filter_selects(scope.clone());
    for (selector, value) in [
        ("#explorerTypeFilter", "ALL"),
        ("#explorerDirectionFilter", "ALL"),
        ("#explorerSearch", ""),
    ] {
        let node = query(&scope, selector);
        if present(&node) {
            set(&node, "value", &JsValue::from_str(value));
        }
    }
    console(
        "log",
        "[KGW Explorer][force-ui] force filters reset to ALL",
        &JsValue::UNDEFINED,
    );
    true
}

fn control_key(control: &JsValue) -> String {
    let data = property(control, "dataset");
    let mut parts = Vec::new();
    for value in [
        property(control, "id"),
        property(control, "name"),
        property(control, "value"),
        property(control, "textContent"),
        call1(control, "getAttribute", &JsValue::from_str("aria-label")),
        property(&data, "action"),
        property(&data, "kgwAction"),
    ] {
        let text = crate::js_string_owned(&value);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join(" ").to_lowercase()
}

fn is_cancel_text(id: &str, key: &str) -> bool {
    key.contains("cancel") || id.to_lowercase().contains("cancel")
}

fn is_cancel(control: &JsValue) -> bool {
    is_cancel_text(
        &crate::js_string_owned(&property(control, "id")),
        &control_key(control),
    )
}

fn set_controls_busy(section: &JsValue, busy: bool, mode: &str) -> bool {
    let scope = force_ui_root(section);
    if !present(&scope) {
        return false;
    }
    let data = property(&scope, "dataset");
    set(
        &data,
        "kgwFetchBusy",
        &JsValue::from_str(if busy { "true" } else { "false" }),
    );
    set(
        &data,
        "kgwFetchMode",
        &JsValue::from_str(if busy { mode } else { "" }),
    );

    let controls = query_all(&scope, CONTROL_SELECTOR);
    let mut disabled_count = 0_u32;
    for control in controls {
        if busy && !is_cancel(&control) {
            set(&control, "disabled", &JsValue::TRUE);
            set_attr(&control, "aria-disabled", "true");
            disabled_count += 1;
        } else {
            set(&control, "disabled", &JsValue::FALSE);
            remove_attr(&control, "aria-disabled");
        }
    }

    let details = Object::new();
    set(details.as_ref(), "busy", &JsValue::from_bool(busy));
    set(details.as_ref(), "mode", &JsValue::from_str(mode));
    set(
        details.as_ref(),
        "disabledCount",
        &JsValue::from_f64(disabled_count as f64),
    );
    console(
        "log",
        "[KGW Explorer][force-ui] controls busy",
        details.as_ref(),
    );
    true
}

fn block_busy_action(event: &JsValue) -> bool {
    let doc = document();
    let mut scope = query(&doc, "#explorer");
    if !present(&scope) {
        scope = query(&doc, ".explorer-python-root");
    }
    if !present(&scope)
        || crate::js_string_owned(&property(&property(&scope, "dataset"), "kgwFetchBusy")) != "true"
    {
        return false;
    }
    let target = call1(
        &property(event, "target"),
        "closest",
        &JsValue::from_str(ACTION_SELECTOR),
    );
    if !present(&target) || !crate::js_boolean(&call1(&scope, "contains", &target)) {
        return false;
    }
    if control_key(&target).contains("cancel") {
        return false;
    }

    for name in [
        "preventDefault",
        "stopPropagation",
        "stopImmediatePropagation",
    ] {
        if let Some(method) = function(event, name) {
            let _ = method.call0(event);
        }
    }

    let details = Object::new();
    set(details.as_ref(), "eventType", &property(event, "type"));
    set(details.as_ref(), "id", &property(&target, "id"));
    set(details.as_ref(), "tag", &property(&target, "tagName"));
    let text_content = property(&target, "textContent");
    let text = if crate::js_boolean(&text_content) {
        crate::js_string_owned(&text_content)
    } else {
        crate::js_string_owned(&property(&target, "value"))
    };
    set(details.as_ref(), "text", &JsValue::from_str(text.trim()));
    console(
        "warn",
        "[KGW Explorer][force-ui] blocked action while fetch is busy",
        details.as_ref(),
    );
    true
}

fn install_busy_blocker() -> bool {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwForceUiBusyBlockInstalled")) {
        return true;
    }
    set(&win, "__kgwForceUiBusyBlockInstalled", &JsValue::TRUE);

    let doc = document();
    for name in ["click", "change", "input", "pointerdown"] {
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            let _ = block_busy_action(&event);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call3(
            &doc,
            "addEventListener",
            &JsValue::from_str(name),
            callback.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        );
        callback.forget();
    }
    true
}

#[wasm_bindgen(js_name = explorerForceSetTableMessage)]
pub fn explorer_force_set_table_message(section: JsValue, message: String) -> bool {
    set_table_message(&section, &message)
}

#[wasm_bindgen(js_name = explorerForceResetDisplayFiltersToAll)]
pub fn explorer_force_reset_display_filters_to_all(section: JsValue) -> bool {
    reset_display_filters(&section)
}

#[wasm_bindgen(js_name = explorerForceSetControlsBusy)]
pub fn explorer_force_set_controls_busy(section: JsValue, busy: bool, mode: String) -> bool {
    set_controls_busy(&section, busy, &mode)
}

#[wasm_bindgen(js_name = explorerInstallForceBusyBlocker)]
pub fn explorer_install_force_busy_blocker() -> bool {
    install_busy_blocker()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancel_detection_matches_legacy_contract() {
        assert!(is_cancel_text("explorerCancel", "anything"));
        assert!(is_cancel_text("", "cancel fetch"));
        assert!(is_cancel_text("MY-CANCEL-BUTTON", ""));
        assert!(!is_cancel_text("explorerFetch", "fetch transactions"));
    }

    #[test]
    fn selectors_preserve_force_ui_surface() {
        for expected in [
            "#explorerAddress",
            "#explorerFromDate",
            "#explorerToDate",
            "#explorerTypeFilter",
            "#explorerDirectionFilter",
            "#explorerSearch",
            "#explorerFetchButton",
            "#explorerForceFetchButton",
            "#explorerFilterButton",
            "#explorerResetFilterButton",
            "button",
            "select",
            "input",
        ] {
            assert!(CONTROL_SELECTOR.contains(expected), "{expected}");
        }
        assert!(ACTION_SELECTOR.contains("textarea"));
        assert!(ACTION_SELECTOR.contains("[role='button']"));
    }
}
