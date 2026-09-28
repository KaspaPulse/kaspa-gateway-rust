use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, prelude::*};

const TYPE_OPTIONS_HTML: &str = r#"
      <option value="ALL">ALL</option>
      <option value="coinbase">coinbase</option>
      <option value="transfer">transfer</option>
    "#;
const DIRECTION_OPTIONS_HTML: &str = r#"
      <option value="ALL">ALL</option>
      <option value="incoming">incoming</option>
      <option value="outgoing">outgoing</option>
    "#;

fn global() -> JsValue {
    js_sys::global().into()
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

fn call1(target: &JsValue, name: &str, argument: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call1(target, argument).ok())
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

fn scope(section: &JsValue) -> JsValue {
    if present(section) {
        section.clone()
    } else {
        root()
    }
}

fn query(section: &JsValue, selector: &str) -> JsValue {
    call1(section, "querySelector", &JsValue::from_str(selector))
}

fn query_all(section: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(section, "querySelectorAll", &JsValue::from_str(selector));
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

fn js_text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn value_or_fallback(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn current_value(node: &JsValue, fallback: &str) -> String {
    if !present(node) {
        return fallback.to_owned();
    }
    let value = js_text(&property(node, "value"));
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value
    }
}
fn normalize_choice(value: &str, allowed: &[&str]) -> String {
    let lower = value.to_lowercase();
    if allowed.contains(&lower.as_str()) {
        if lower == "all" {
            "ALL".to_owned()
        } else {
            lower
        }
    } else {
        "ALL".to_owned()
    }
}

fn option_values(select: &JsValue) -> Array {
    let output = Array::new();
    if !present(select) {
        return output;
    }
    for option in query_all(select, "option") {
        output.push(&JsValue::from_str(&current_value(&option, "")));
    }
    output
}

fn install_options(select: &JsValue, html: &str, allowed: &[&str]) -> Array {
    if !present(select) {
        return Array::new();
    }
    let previous = current_value(select, "ALL");
    set(select, "innerHTML", &JsValue::from_str(html));
    let selected = normalize_choice(&previous, allowed);
    set(select, "value", &JsValue::from_str(&selected));
    option_values(select)
}

fn console_log(message: &str, payload: &JsValue) {
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "log") {
        let _ = log.call2(&console, &JsValue::from_str(message), payload);
    }
}

fn ensure_filter_options_impl(section: &JsValue) -> JsValue {
    let scope = scope(section);
    let type_el = query(&scope, "#explorerTypeFilter");
    let direction_el = query(&scope, "#explorerDirectionFilter");

    let type_options = install_options(
        &type_el,
        TYPE_OPTIONS_HTML,
        &["all", "coinbase", "transfer"],
    );
    let direction_options = install_options(
        &direction_el,
        DIRECTION_OPTIONS_HTML,
        &["all", "incoming", "outgoing"],
    );

    let payload = Object::new();
    set(payload.as_ref(), "typeOptions", type_options.as_ref());
    set(
        payload.as_ref(),
        "directionOptions",
        direction_options.as_ref(),
    );
    console_log(
        "[KGW Explorer][filter] canonical dropdown options installed",
        payload.as_ref(),
    );
    payload.into()
}

fn read_filter_state_impl(section: &JsValue) -> JsValue {
    let scope = scope(section);
    let _ = ensure_filter_options_impl(&scope);
    let type_el = query(&scope, "#explorerTypeFilter");
    let direction_el = query(&scope, "#explorerDirectionFilter");
    let search_el = query(&scope, "#explorerSearch");
    let from_el = query(&scope, "#explorerFromDate");
    let to_el = query(&scope, "#explorerToDate");

    let state = Object::new();
    for (key, value) in [
        ("typeId", js_text(&property(&type_el, "id"))),
        ("directionId", js_text(&property(&direction_el, "id"))),
        ("typeValue", current_value(&type_el, "ALL")),
        ("directionValue", current_value(&direction_el, "ALL")),
        ("searchValue", current_value(&search_el, "")),
        ("fromValue", current_value(&from_el, "")),
        ("toValue", current_value(&to_el, "")),
    ] {
        set(state.as_ref(), key, &JsValue::from_str(&value));
    }
    console_log("[KGW Explorer][filter] controls", state.as_ref());
    state.into()
}

fn filter_value_impl(selector: &str, section: &JsValue, fallback: &str) -> String {
    let scope = scope(section);
    let element = query(&scope, selector);
    let raw = if present(&element) {
        js_text(&property(&element, "value"))
    } else {
        String::new()
    };
    value_or_fallback(&raw, fallback)
}
fn build_list_request_impl(
    section: &JsValue,
    address: JsValue,
    start_ts: JsValue,
    end_ts: JsValue,
    limit: JsValue,
) -> JsValue {
    let state = read_filter_state_impl(section);
    let request = Object::new();
    set(request.as_ref(), "address", &address);
    set(request.as_ref(), "start_ts", &start_ts);
    set(request.as_ref(), "end_ts", &end_ts);
    set(request.as_ref(), "tx_type", &property(&state, "typeValue"));
    set(
        request.as_ref(),
        "direction",
        &property(&state, "directionValue"),
    );
    set(
        request.as_ref(),
        "search_query",
        &property(&state, "searchValue"),
    );
    set(request.as_ref(), "limit", &limit);
    console_log("[KGW Explorer][filter] request", request.as_ref());
    request.into()
}

fn filter_build_request_impl(
    section: &JsValue,
    address: JsValue,
    start_ts: JsValue,
    end_ts: JsValue,
    limit: JsValue,
) -> JsValue {
    let scope = scope(section);
    let request = Object::new();
    set(request.as_ref(), "address", &address);
    set(request.as_ref(), "start_ts", &start_ts);
    set(request.as_ref(), "end_ts", &end_ts);
    set(
        request.as_ref(),
        "tx_type",
        &JsValue::from_str(&filter_value_impl("#explorerTypeFilter", &scope, "ALL")),
    );
    set(
        request.as_ref(),
        "direction",
        &JsValue::from_str(&filter_value_impl(
            "#explorerDirectionFilter",
            &scope,
            "ALL",
        )),
    );
    let search = query(&scope, "#explorerSearch");
    set(
        request.as_ref(),
        "search_query",
        &JsValue::from_str(&current_value(&search, "")),
    );
    set(request.as_ref(), "limit", &limit);
    console_log("[KGW Explorer][filter-owner] request", request.as_ref());
    request.into()
}
#[wasm_bindgen(js_name = explorerEnsureFilterOptions)]
pub fn explorer_ensure_filter_options(section: JsValue) -> JsValue {
    ensure_filter_options_impl(&section)
}

#[wasm_bindgen(js_name = explorerReadFilterState)]
pub fn explorer_read_filter_state(section: JsValue) -> JsValue {
    read_filter_state_impl(&section)
}

#[wasm_bindgen(js_name = explorerBuildListRequest)]
pub fn explorer_build_list_request(
    section: JsValue,
    address: JsValue,
    start_ts: JsValue,
    end_ts: JsValue,
    limit: JsValue,
) -> JsValue {
    build_list_request_impl(&section, address, start_ts, end_ts, limit)
}

#[wasm_bindgen(js_name = explorerFilterValue)]
pub fn explorer_filter_value(selector: String, section: JsValue, fallback: String) -> String {
    filter_value_impl(&selector, &section, &fallback)
}
#[wasm_bindgen(js_name = explorerFilterBuildRequest)]
pub fn explorer_filter_build_request(
    section: JsValue,
    address: JsValue,
    start_ts: JsValue,
    end_ts: JsValue,
    limit: JsValue,
) -> JsValue {
    filter_build_request_impl(&section, address, start_ts, end_ts, limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_filter_choices_match_legacy() {
        assert_eq!(
            normalize_choice("ALL", &["all", "coinbase", "transfer"]),
            "ALL"
        );
        assert_eq!(
            normalize_choice("CoinBase", &["all", "coinbase", "transfer"]),
            "coinbase"
        );
        assert_eq!(
            normalize_choice("transfer", &["all", "coinbase", "transfer"]),
            "transfer"
        );
        assert_eq!(
            normalize_choice("address", &["all", "coinbase", "transfer"]),
            "ALL"
        );
        assert_eq!(
            normalize_choice("incoming", &["all", "incoming", "outgoing"]),
            "incoming"
        );
        assert_eq!(
            normalize_choice("bad", &["all", "incoming", "outgoing"]),
            "ALL"
        );
    }

    #[test]
    fn filter_value_trimming_and_fallback_match_legacy() {
        assert_eq!(value_or_fallback(" incoming ", "ALL"), "incoming");
        assert_eq!(value_or_fallback("   ", "ALL"), "ALL");
        assert_eq!(value_or_fallback("", ""), "");
    }

    #[test]
    fn option_templates_preserve_exact_contract_values() {
        for needle in [
            r#"value="ALL""#,
            r#"value="coinbase""#,
            r#"value="transfer""#,
        ] {
            assert!(TYPE_OPTIONS_HTML.contains(needle));
        }
        for needle in [
            r#"value="ALL""#,
            r#"value="incoming""#,
            r#"value="outgoing""#,
        ] {
            assert!(DIRECTION_OPTIONS_HTML.contains(needle));
        }
    }
}
