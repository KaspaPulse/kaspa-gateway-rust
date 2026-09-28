use js_sys::{Array, Date, Function, JSON, Math, Object, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

const PREFIX: &str = "[KGW][microscope][explorer]";
const ELEMENT_IDS: &[&str] = &[
    "explorer",
    "explorerAddress",
    "explorerAddressOptions",
    "explorerBalanceValue",
    "explorerAddressNameValue",
    "explorerBalanceUsdValue",
    "explorerFromDate",
    "explorerToDate",
    "explorerDirectionFilter",
    "explorerTypeFilter",
    "explorerSearch",
    "explorerFetch",
    "explorerForceFetch",
    "explorerOpenExplorer",
    "explorerCancel",
    "explorerTransactionsTable",
    "explorerTransactionsBody",
    "explorerExportControls",
    "explorerExportCsv",
    "explorerExportHtml",
    "explorerExportPdf",
    "explorerTableFontSize",
    "explorerStatus",
];

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

fn call0(target: &JsValue, name: &str) -> JsValue {
    function(target, name)
        .and_then(|method| method.call0(target).ok())
        .unwrap_or(JsValue::UNDEFINED)
}
fn call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn query(scope: &JsValue, selector: &str) -> JsValue {
    call1(scope, "querySelector", &JsValue::from_str(selector))
}

fn computed_style(node: &JsValue) -> JsValue {
    let win = window();
    function(&win, "getComputedStyle")
        .and_then(|method| method.call1(&win, node).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn exact_string(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn microscope_now() -> String {
    String::from(Date::new_0().to_iso_string())
}

fn error_object(value: &JsValue) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "message", &property(value, "message"));
    set(output.as_ref(), "stack", &property(value, "stack"));
    output.into()
}
fn microscope_safe_json(value: &JsValue) -> String {
    let replacer = Closure::wrap(Box::new(move |_key: JsValue, value: JsValue| -> JsValue {
        if exact_string(&value.js_typeof()) == "bigint" {
            return JsValue::from_str(&exact_string(&value));
        }
        if value.is_instance_of::<js_sys::Error>() {
            return error_object(&value);
        }
        value
    }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);

    let encoded = JSON::stringify_with_replacer(value, replacer.as_ref().unchecked_ref());
    match encoded {
        Ok(text) => crate::js_string_owned(text.as_ref()),
        Err(_) => exact_string(value),
    }
}

fn overlay(target: &JsValue, details: &JsValue) {
    if !present(details) || !details.is_object() || Array::is_array(details) {
        return;
    }
    for entry in Object::entries(&Object::from(details.clone())).iter() {
        let pair = Array::from(&entry);
        if pair.length() < 2 {
            continue;
        }
        set(target, &exact_string(&pair.get(0)), &pair.get(1));
    }
}

fn payload(stage: &str, details: &JsValue) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "stage", &JsValue::from_str(stage));
    set(output.as_ref(), "at", &JsValue::from_str(&microscope_now()));
    overlay(output.as_ref(), details);
    output.into()
}
fn log_line(level: &str, stage: &str, details: &JsValue) {
    let value = payload(stage, details);
    let text = format!("{PREFIX} {stage} :: {}", microscope_safe_json(&value));
    let console = property(&global(), "console");
    if let Some(method) = function(&console, level) {
        let _ = method.call1(&console, &JsValue::from_str(&text));
    }
}

fn error_payload(stage: &str, error: &JsValue, details: &JsValue) -> JsValue {
    let output = Object::new();
    set(output.as_ref(), "stage", &JsValue::from_str(stage));
    set(output.as_ref(), "at", &JsValue::from_str(&microscope_now()));
    let message = property(error, "message");
    let message = if crate::js_boolean(&message) {
        exact_string(&message)
    } else {
        exact_string(error)
    };
    let stack = property(error, "stack");
    let stack_text = if crate::js_boolean(&stack) {
        exact_string(&stack)
    } else {
        String::new()
    };
    set(output.as_ref(), "error", &JsValue::from_str(&message));
    set(output.as_ref(), "stack", &JsValue::from_str(&stack_text));
    overlay(output.as_ref(), details);
    output.into()
}
fn error_line(stage: &str, error: &JsValue, details: &JsValue) {
    let value = error_payload(stage, error, details);
    let text = format!("{PREFIX} {stage} :: {}", microscope_safe_json(&value));
    let console = property(&global(), "console");
    if let Some(method) = function(&console, "error") {
        let _ = method.call1(&console, &JsValue::from_str(&text));
    }
}

fn rounded_property(rect: &JsValue, name: &str) -> JsValue {
    let value = crate::js_number(&property(rect, name));
    if value.is_finite() {
        JsValue::from_f64(Math::round(value))
    } else {
        JsValue::NULL
    }
}

fn nullable_string(value: JsValue) -> JsValue {
    if present(&value) {
        JsValue::from_str(&exact_string(&value))
    } else {
        JsValue::NULL
    }
}

fn element_report(section: &JsValue) -> JsValue {
    let report = Object::new();
    let doc = document();

    for id in ELEMENT_IDS {
        let selector = format!("#{id}");
        let mut node = if present(section) {
            query(section, &selector)
        } else {
            JsValue::UNDEFINED
        };
        if !present(&node) {
            node = call1(&doc, "getElementById", &JsValue::from_str(id));
        }

        let item = Object::new();
        set(item.as_ref(), "exists", &JsValue::from_bool(present(&node)));
        set(
            item.as_ref(),
            "tag",
            &nullable_string(property(&node, "tagName")),
        );
        set(
            item.as_ref(),
            "hidden",
            &JsValue::from_bool(crate::js_boolean(&property(&node, "hidden"))),
        );
        set(
            item.as_ref(),
            "disabled",
            &JsValue::from_bool(crate::js_boolean(&property(&node, "disabled"))),
        );

        if present(&node) {
            let style = computed_style(&node);
            set(
                item.as_ref(),
                "display",
                &nullable_string(property(&style, "display")),
            );
            set(
                item.as_ref(),
                "visibility",
                &nullable_string(property(&style, "visibility")),
            );
            let rect = call0(&node, "getBoundingClientRect");
            set(item.as_ref(), "width", &rounded_property(&rect, "width"));
            set(item.as_ref(), "height", &rounded_property(&rect, "height"));
        } else {
            set(item.as_ref(), "display", &JsValue::NULL);
            set(item.as_ref(), "visibility", &JsValue::NULL);
            set(item.as_ref(), "width", &JsValue::NULL);
            set(item.as_ref(), "height", &JsValue::NULL);
        }

        let text = property(&node, "textContent");
        let text_length = if present(&text) {
            crate::js_string_owned(&text).encode_utf16().count() as f64
        } else {
            0.0
        };
        set(item.as_ref(), "textLength", &JsValue::from_f64(text_length));

        let has_value =
            present(&node) && Reflect::has(&node, &JsValue::from_str("value")).unwrap_or(false);
        set(
            item.as_ref(),
            "value",
            if has_value {
                property(&node, "value")
            } else {
                JsValue::UNDEFINED
            }
            .as_ref(),
        );
        set(report.as_ref(), id, item.as_ref());
    }

    let report_value: JsValue = report.into();
    log_line("info", "DOM REPORT", &report_value);
    report_value
}
fn box_report(node: &JsValue) -> JsValue {
    if !present(node) {
        return JsValue::NULL;
    }
    let rect = call0(node, "getBoundingClientRect");
    if !present(&rect) {
        return JsValue::NULL;
    }
    let style = computed_style(node);
    let output = Object::new();
    for name in ["x", "y", "width", "height"] {
        set(output.as_ref(), name, &rounded_property(&rect, name));
    }
    set(
        output.as_ref(),
        "display",
        &JsValue::from_str(&crate::js_string_owned(&property(&style, "display"))),
    );
    set(
        output.as_ref(),
        "overflow",
        &JsValue::from_str(&crate::js_string_owned(&property(&style, "overflow"))),
    );
    let grid = property(&style, "gridTemplateRows");
    let grid_text = if crate::js_boolean(&grid) {
        exact_string(&grid)
    } else {
        String::new()
    };
    set(
        output.as_ref(),
        "gridTemplateRows",
        &JsValue::from_str(&grid_text),
    );
    output.into()
}

fn layout_report(section: &JsValue) -> JsValue {
    let table_zone = query(section, ".explorer-table-zone");
    let footer = query(section, "#explorerExportControls");
    let load = query(section, ".explorer-load-card");
    let filter = query(section, ".explorer-filter-card");
    let shell = query(section, ".explorer-clean-shell");
    let report = Object::new();
    set(report.as_ref(), "shell", &box_report(&shell));
    set(report.as_ref(), "load", &box_report(&load));
    set(report.as_ref(), "filter", &box_report(&filter));
    set(report.as_ref(), "tableZone", &box_report(&table_zone));
    set(report.as_ref(), "footer", &box_report(&footer));

    let viewport = Object::new();
    set(
        viewport.as_ref(),
        "width",
        &property(&window(), "innerWidth"),
    );
    set(
        viewport.as_ref(),
        "height",
        &property(&window(), "innerHeight"),
    );
    set(report.as_ref(), "viewport", viewport.as_ref());

    let report_value: JsValue = report.into();
    log_line("info", "LAYOUT REPORT", &report_value);
    report_value
}

fn array_length_or_null(value: &JsValue) -> JsValue {
    if Array::is_array(value) {
        property(value, "length")
    } else {
        JsValue::NULL
    }
}

fn api_shape(label: &str, value: &JsValue) {
    if Array::is_array(value) {
        let details = Object::new();
        set(details.as_ref(), "kind", &JsValue::from_str("array"));
        set(details.as_ref(), "length", &property(value, "length"));
        let first = Reflect::get(value, &JsValue::from_f64(0.0)).unwrap_or(JsValue::UNDEFINED);
        set(
            details.as_ref(),
            "first",
            if crate::js_boolean(&first) {
                first
            } else {
                JsValue::NULL
            }
            .as_ref(),
        );
        log_line("info", label, details.as_ref());
        return;
    }

    if present(value) && value.is_object() {
        let details = Object::new();
        set(details.as_ref(), "kind", &JsValue::from_str("object"));
        set(
            details.as_ref(),
            "keys",
            Object::keys(&Object::from(value.clone())).as_ref(),
        );

        let groups = property(value, "groups");
        let rows = property(value, "rows");
        let transactions = property(value, "transactions");
        set(
            details.as_ref(),
            "groupsLength",
            &array_length_or_null(&groups),
        );
        set(details.as_ref(), "rowsLength", &array_length_or_null(&rows));
        set(
            details.as_ref(),
            "transactionsLength",
            &array_length_or_null(&transactions),
        );
        let first_group = if Array::is_array(&groups) {
            Reflect::get(&groups, &JsValue::from_f64(0.0)).unwrap_or(JsValue::UNDEFINED)
        } else {
            JsValue::UNDEFINED
        };
        if crate::js_boolean(&first_group) {
            set(
                details.as_ref(),
                "firstGroupKeys",
                Object::keys(&Object::from(first_group.clone())).as_ref(),
            );
            set(details.as_ref(), "firstGroup", &first_group);
        } else {
            set(details.as_ref(), "firstGroupKeys", &JsValue::NULL);
            set(details.as_ref(), "firstGroup", &JsValue::NULL);
        }
        log_line("info", label, details.as_ref());
        return;
    }

    let details = Object::new();
    set(details.as_ref(), "kind", &value.js_typeof());
    set(details.as_ref(), "value", value);
    log_line("info", label, details.as_ref());
}
#[wasm_bindgen(js_name = explorerMicroscopeLog)]
pub fn explorer_microscope_log(stage: String, details: JsValue) {
    log_line("info", &stage, &details);
}

#[wasm_bindgen(js_name = explorerMicroscopeWarn)]
pub fn explorer_microscope_warn(stage: String, details: JsValue) {
    log_line("warn", &stage, &details);
}

#[wasm_bindgen(js_name = explorerMicroscopeError)]
pub fn explorer_microscope_error(stage: String, error: JsValue, details: JsValue) {
    error_line(&stage, &error, &details);
}

#[wasm_bindgen(js_name = explorerMicroscopeElementReport)]
pub fn explorer_microscope_element_report(section: JsValue) -> JsValue {
    element_report(&section)
}

#[wasm_bindgen(js_name = explorerMicroscopeLayoutReport)]
pub fn explorer_microscope_layout_report(section: JsValue) -> JsValue {
    layout_report(&section)
}

#[wasm_bindgen(js_name = explorerMicroscopeStateReport)]
pub fn explorer_microscope_state_report(label: String, snapshot: JsValue) {
    log_line("info", &label, &snapshot);
}

#[wasm_bindgen(js_name = explorerMicroscopeApiShape)]
pub fn explorer_microscope_api_shape(label: String, value: JsValue) {
    api_shape(&label, &value);
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_id_contract_preserves_legacy_surface() {
        assert_eq!(ELEMENT_IDS.len(), 23);
        for id in [
            "explorer",
            "explorerAddress",
            "explorerTransactionsTable",
            "explorerTransactionsBody",
            "explorerExportControls",
            "explorerExportCsv",
            "explorerExportHtml",
            "explorerExportPdf",
            "explorerTableFontSize",
            "explorerStatus",
        ] {
            assert!(ELEMENT_IDS.contains(&id), "{id}");
        }
    }

    #[test]
    fn microscope_prefix_is_stable() {
        assert_eq!(PREFIX, "[KGW][microscope][explorer]");
    }
}
