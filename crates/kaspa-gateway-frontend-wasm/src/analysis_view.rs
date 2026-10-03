use super::{
    call_method0, call_method1, js_boolean, js_number, js_string_owned, method, set_property,
};
use js_sys::{Array, Date, Intl::NumberFormat, Object, Reflect};
use std::cell::RefCell;
use std::collections::BTreeSet;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

thread_local! {
    static ROWS: RefCell<Vec<JsValue>> = const { RefCell::new(Vec::new()) };
    static FILTERED_ROWS: RefCell<Vec<JsValue>> = const { RefCell::new(Vec::new()) };
    static EXPANDED_ROWS: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

fn global() -> JsValue {
    js_sys::global().into()
}
fn document() -> JsValue {
    property(&global(), "document")
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}
fn truthy(value: &JsValue) -> bool {
    js_boolean(value)
}
fn raw(value: &JsValue) -> String {
    js_string_owned(value).trim().to_owned()
}

fn root() -> JsValue {
    call_method1(
        &document(),
        "getElementById",
        &JsValue::from_str("analysis"),
    )
    .unwrap_or(JsValue::UNDEFINED)
}
fn q(selector: &str) -> JsValue {
    let owner = root();
    if !truthy(&owner) {
        return JsValue::UNDEFINED;
    }
    call_method1(&owner, "querySelector", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED)
}
fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let Ok(list) = call_method1(target, "querySelectorAll", &JsValue::from_str(selector)) else {
        return Vec::new();
    };
    let length = js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .collect()
}
fn first_nullish(row: &JsValue, names: &[&str], fallback: JsValue) -> JsValue {
    for name in names {
        let value = property(row, name);
        if !value.is_null() && !value.is_undefined() {
            return value;
        }
    }
    fallback
}

fn first_truthy(row: &JsValue, names: &[&str], fallback: JsValue) -> JsValue {
    for name in names {
        let value = property(row, name);
        if truthy(&value) {
            return value;
        }
    }
    fallback
}
fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn number_text(value: &str) -> Option<f64> {
    let text = value
        .replace(',', "")
        .replace(" KAS", "")
        .replace(" USD", "");
    if text.is_empty() {
        return None;
    }
    let number = text.parse::<f64>().ok()?;
    number.is_finite().then_some(number)
}
fn number_value(value: &JsValue) -> Option<f64> {
    number_text(&raw(value))
}

fn format_number(value: f64, max_digits: u32) -> String {
    let locales = Array::new();
    locales.push(&JsValue::from_str("en-US"));
    let options = Object::new();
    let _ = set_property(
        options.as_ref(),
        "minimumFractionDigits",
        &JsValue::from_f64(0.0),
    );
    let _ = set_property(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(max_digits as f64),
    );
    let formatter = NumberFormat::new(&locales, &options);
    formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(value))
        .map(|value| raw(&value))
        .unwrap_or_else(|_| value.to_string())
}
fn format_count(value: &JsValue) -> String {
    number_value(value)
        .map(|number| format_number(number, 0))
        .unwrap_or_else(|| raw(value))
}
fn format_usd(value: &JsValue) -> String {
    number_value(value)
        .map(|number| format!("{}{}", "$", format_number(number, 2)))
        .unwrap_or_else(|| raw(value))
}
fn middle_trim(value: &str, head: usize, tail: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    if chars.len() <= head + tail + 3 {
        return value.to_owned();
    }
    let first = chars[..head].iter().collect::<String>();
    let last = chars[chars.len() - tail..].iter().collect::<String>();
    format!("{first}…{last}")
}
fn title_case(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return "—".to_owned();
    }
    value
        .split(|ch: char| ch.is_whitespace() || ch == '_' || ch == '-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn direction_case(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return "—".to_owned();
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return "—".to_owned();
    };
    format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
}
fn flow_sign(value: &JsValue) -> String {
    let Some(number) = number_value(value) else {
        return raw(value);
    };
    let formatted = format_number(number, 3);
    if number > 0.0 {
        format!("+{formatted}")
    } else {
        formatted
    }
}
fn row_key(row: &JsValue, index: usize) -> String {
    [
        index.to_string(),
        raw(&first_nullish(row, &["type"], JsValue::from_str(""))),
        raw(&first_nullish(
            row,
            &["address", "counterparty"],
            JsValue::from_str(""),
        )),
        raw(&first_nullish(
            row,
            &["transactionId", "txid"],
            JsValue::from_str(""),
        )),
        raw(&first_nullish(
            row,
            &["name", "knownName"],
            JsValue::from_str(""),
        )),
    ]
    .join("|")
}
fn child_rows(row: &JsValue) -> Vec<JsValue> {
    for name in [
        "transactions",
        "children",
        "details",
        "items",
        "txList",
        "tx_list",
        "rows",
    ] {
        let value = property(row, name);
        if Array::is_array(&value) {
            return Array::from(&value).iter().collect();
        }
    }
    Vec::new()
}
fn normalize_child(tx: &JsValue) -> JsValue {
    if !tx.is_object() {
        return Object::new().into();
    }
    let timestamp = first_nullish(
        tx,
        &[
            "timestamp_ms",
            "timestampMs",
            "timestamp",
            "datetime",
            "date_time",
            "time",
        ],
        JsValue::from_str(""),
    );
    let mut datetime = raw(&timestamp);
    if let Some(number) = number_value(&timestamp)
        && number > 0.0
    {
        let millis = if number > 10_000_000_000.0 {
            number
        } else {
            number * 1000.0
        };
        let date = Date::new(&JsValue::from_f64(millis));
        if date.get_time().is_finite() {
            datetime = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                date.get_full_year(),
                date.get_month() + 1,
                date.get_date(),
                date.get_hours(),
                date.get_minutes(),
                date.get_seconds()
            );
        }
    }
    let output = Object::new();
    let fields = [
        ("datetime", JsValue::from_str(&datetime)),
        (
            "id",
            first_truthy(
                tx,
                &["txid", "transaction_id", "transactionId", "id", "hash"],
                JsValue::from_str(""),
            ),
        ),
        (
            "direction",
            first_truthy(tx, &["direction", "txsDir"], JsValue::from_str("")),
        ),
        (
            "amount",
            first_nullish(
                tx,
                &["amount_kas", "amountKas", "amount", "value"],
                JsValue::from_str(""),
            ),
        ),
        (
            "value",
            first_nullish(tx, &["value_usd", "valueUsd"], JsValue::from_str("")),
        ),
        (
            "block",
            first_nullish(
                tx,
                &["block_score", "blockScore", "block_height", "blockHeight"],
                JsValue::from_str(""),
            ),
        ),
        (
            "type",
            first_truthy(tx, &["tx_type", "txType", "type"], JsValue::from_str("")),
        ),
        (
            "counterparty",
            first_truthy(
                tx,
                &["counterparty", "address", "from_address", "to_address"],
                JsValue::from_str(""),
            ),
        ),
    ];
    for (name, value) in fields {
        let _ = set_property(output.as_ref(), name, &value);
    }
    output.into()
}
fn current_usd_price() -> Option<f64> {
    let body = property(&document(), "body");
    let text = raw(&property(&body, "textContent"));
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
            index += 1;
        }
        let number_text = &text[start..index];
        let rest = text[index..].trim_start();
        if rest.starts_with("USD")
            && let Ok(value) = number_text.parse::<f64>()
            && value.is_finite()
            && value > 0.0
        {
            return Some(value);
        }
    }
    None
}
fn usd_value(amount: &JsValue, explicit: &JsValue) -> String {
    if number_value(explicit).is_some() {
        return format_usd(explicit);
    }
    let Some(amount) = number_value(amount) else {
        return "—".to_owned();
    };
    let Some(price) = current_usd_price() else {
        return "—".to_owned();
    };
    format!("{}{}", "$", format_number(amount.abs() * price, 2))
}
fn normalize_row(row: &JsValue) -> JsValue {
    let output = Object::new();
    let details = first_nullish(
        row,
        &["details", "children", "items", "txList", "tx_list", "rows"],
        {
            let transactions = property(row, "transactions");
            if Array::is_array(&transactions) {
                transactions
            } else {
                Array::new().into()
            }
        },
    );
    let mappings: [(&str, &[&str]); 15] = [
        ("name", &["name", "knownName", "datetime"]),
        ("knownName", &["knownName", "name"]),
        ("datetime", &["datetime"]),
        ("address", &["address", "counterparty"]),
        ("transactionId", &["transactionId", "txid"]),
        ("txCount", &["txCount", "txs", "transactions", "count"]),
        ("txsDir", &["txsDir", "direction"]),
        ("direction", &["direction"]),
        ("netFlow", &["netFlow", "amount", "net_kas"]),
        ("amount", &["amount", "amount_kas", "net_kas"]),
        ("valueUsd", &["valueUsd", "value"]),
        ("blockScore", &["blockScore", "block"]),
        ("type", &["type", "tx_type"]),
        ("firstSeen", &["firstSeen", "firstTransaction", "first"]),
        ("lastSeen", &["lastSeen", "lastTransaction", "last"]),
    ];
    for (name, aliases) in mappings {
        let _ = set_property(
            output.as_ref(),
            name,
            &first_nullish(row, aliases, JsValue::from_str("")),
        );
    }
    let _ = set_property(output.as_ref(), "details", &details);
    let _ = set_property(output.as_ref(), "transactions", &details);
    output.into()
}
fn row_matches(row: &JsValue, search: &str, row_type: &str, direction: &str) -> bool {
    let text = [
        "name",
        "datetime",
        "address",
        "transactionId",
        "type",
        "direction",
        "txsDir",
        "netFlow",
        "amount",
        "blockScore",
    ]
    .iter()
    .map(|name| raw(&property(row, name)))
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase();
    let actual_type = raw(&property(row, "type")).to_uppercase();
    let actual_direction = {
        let value = raw(&property(row, "direction"));
        if value.is_empty() {
            raw(&property(row, "txsDir"))
        } else {
            value
        }
    }
    .to_uppercase();
    (search.is_empty() || text.contains(search))
        && (row_type == "ALL" || actual_type == row_type)
        && (direction == "ALL" || actual_direction == direction)
}
fn apply_filter() {
    let search = raw(&property(&q("#analysisSearch"), "value")).to_lowercase();
    let row_type = {
        let value = raw(&property(&q("#analysisType"), "value")).to_uppercase();
        if value.is_empty() {
            "ALL".to_owned()
        } else {
            value
        }
    };
    let direction = {
        let value = raw(&property(&q("#analysisDirection"), "value")).to_uppercase();
        if value.is_empty() {
            "ALL".to_owned()
        } else {
            value
        }
    };
    let rows = ROWS.with(|rows| rows.borrow().clone());
    FILTERED_ROWS.with(|filtered| {
        *filtered.borrow_mut() = rows
            .into_iter()
            .filter(|row| row_matches(row, &search, &row_type, &direction))
            .collect()
    });
}
fn set_summary(summary: &JsValue) {
    let values = [
        "totalInflow",
        "totalOutflow",
        "netFlow",
        "avgInflow",
        "avgOutflow",
        "totalTransactions",
        "largestInflow",
        "largestOutflow",
        "uniqueCounterparties",
        "firstTransaction",
        "lastTransaction",
        "durationDays",
    ];
    let nodes = query_all(&root(), ".analysis-metric strong");
    for (index, node) in nodes.iter().enumerate() {
        let value = values
            .get(index)
            .map(|name| property(summary, name))
            .unwrap_or(JsValue::UNDEFINED);
        let raw_value = js_string_owned(&value);
        let text = if value.is_null() || value.is_undefined() || raw_value.is_empty() {
            "—".to_owned()
        } else {
            raw_value
        };
        let _ = set_property(node, "textContent", &JsValue::from_str(&text));
        let _ = set_property(node, "title", &JsValue::from_str(&text));
    }
}
fn set_controls_enabled(enabled: bool) {
    for selector in [
        "#analysisSearch",
        "#analysisType",
        "#analysisDirection",
        "#analysisFilter",
        "#analysisResetFilter",
        "#analysisExportCsv",
        "#analysisExportHtml",
        "#analysisExportPdf",
    ] {
        let node = q(selector);
        if truthy(&node) {
            let _ = set_property(&node, "disabled", &JsValue::from_bool(!enabled));
        }
    }
    let pdf = q("#analysisExportPdf");
    if truthy(&pdf) {
        let _ = set_property(&pdf, "title", &JsValue::from_str(""));
    }
}
fn render_children(key: &str, row: &JsValue) -> String {
    let children = child_rows(row);
    if children.is_empty() {
        return format!(
            r#"<tr class="kgw-analysis-python-child-row-r9 kgw-analysis-python-tree-empty-r14" data-kgw-analysis-python-child="true" data-row-key="{}"><td colspan="7" class="kgw-analysis-python-empty-r9">No transaction rows found for this parent.</td></tr>"#,
            html_escape(key)
        );
    }
    children.iter().map(|tx|{
        let item=normalize_child(tx);
        let datetime=raw(&property(&item,"datetime"));
        let id={let id=raw(&property(&item,"id"));if id.is_empty(){raw(&property(&item,"counterparty"))}else{id}};
        let direction=direction_case(&raw(&property(&item,"direction")));
        let amount_value=property(&item,"amount"); let amount=flow_sign(&amount_value); let value=usd_value(&amount_value,&property(&item,"value"));
        let block=raw(&property(&item,"block")); let kind=title_case(&raw(&property(&item,"type")));
        format!(r#"<tr class="kgw-analysis-python-child-row-r9 kgw-analysis-python-tree-child-r14 kgw-analysis-child-columns-r16" data-kgw-analysis-python-child="true" data-row-key="{}"><td class="kgw-analysis-tree-date-r14">{}</td><td class="kgw-analysis-tree-id-r14" title="{}"><span class="kgw-analysis-mono">{}</span></td><td class="kgw-analysis-tree-dir-r14">{}</td><td class="kgw-analysis-tree-amount-r14">{}</td><td class="kgw-analysis-tree-value-r14">{}</td><td class="kgw-analysis-tree-block-r14">{}</td><td class="kgw-analysis-tree-type-r14">{}</td></tr>"#,html_escape(key),html_escape(if datetime.is_empty(){"—"}else{&datetime}),html_escape(&id),html_escape(if id.is_empty(){"—"}else{&id}),html_escape(&direction),html_escape(if amount.is_empty(){"—"}else{&amount}),html_escape(&value),html_escape(if block.is_empty(){"—"}else{&block}),html_escape(&kind))
    }).collect::<Vec<_>>().join("")
}
fn empty_message() -> &'static str {
    if ROWS.with(|rows| rows.borrow().is_empty()) {
        "Load an address to see analysis."
    } else {
        "No rows match the current filter."
    }
}
fn bind_expand_handler() {
    let body = q("#analysisRows");
    if !truthy(&body) {
        return;
    }
    let data = property(&body, "dataset");
    if raw(&property(&data, "kgwAnalysisPythonLazyR9")) == "true" {
        return;
    }
    let _ = set_property(&data, "kgwAnalysisPythonLazyR9", &JsValue::from_str("true"));
    let body_for_event = body.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let Ok(button) = call_method1(
            &target,
            "closest",
            &JsValue::from_str("[data-kgw-analysis-python-toggle-r9]"),
        ) else {
            return;
        };
        if !truthy(&button) {
            return;
        }
        let contained = method(&body_for_event, "contains")
            .and_then(|f| f.call1(&body_for_event, &button))
            .map(|v| truthy(&v))
            .unwrap_or(false);
        if !contained {
            return;
        }
        let _ = call_method0(&event, "preventDefault");
        let key = raw(
            &call_method1(&button, "getAttribute", &JsValue::from_str("data-row-key"))
                .unwrap_or(JsValue::UNDEFINED),
        );
        if key.is_empty() {
            return;
        }
        EXPANDED_ROWS.with(|rows| {
            let mut rows = rows.borrow_mut();
            if !rows.remove(&key) {
                rows.insert(key);
            }
        });
        render_rows_internal();
    }) as Box<dyn FnMut(JsValue)>);
    if let Ok(add) = method(&body, "addEventListener")
        && add
            .call2(
                &body,
                &JsValue::from_str("click"),
                callback.as_ref().unchecked_ref(),
            )
            .is_ok()
    {
        callback.forget();
    }
}
fn render_rows_internal() {
    let body = q("#analysisRows");
    if !truthy(&body) {
        return;
    }
    apply_filter();
    bind_expand_handler();
    let filtered = FILTERED_ROWS.with(|rows| rows.borrow().clone());
    if filtered.is_empty() {
        let html = format!(
            r#"<tr class="analysis-empty-row kgw-analysis-python-tree-empty-r14"><td colspan="7">{}</td></tr>"#,
            html_escape(empty_message())
        );
        let _ = set_property(&body, "innerHTML", &JsValue::from_str(&html));
        return;
    }
    let expanded = EXPANDED_ROWS.with(|rows| rows.borrow().clone());
    let html=filtered.iter().enumerate().map(|(index,row)|{
        let key=row_key(row,index); let is_expanded=expanded.contains(&key);
        let primary={let value=raw(&first_truthy(row,&["name","knownName","datetime"],JsValue::from_str("Counterparty")));if value.is_empty(){"Counterparty".to_owned()}else{value}};
        let address_or_tx=raw(&first_truthy(row,&["address","transactionId"],JsValue::from_str("")));
        let coinbase_source=if !address_or_tx.is_empty(){address_or_tx.clone()}else{primary.clone()}.to_lowercase();
        let is_coinbase=coinbase_source.contains("coinbase")||coinbase_source.contains("mining");
        let parent=if is_coinbase{"Coinbase / Mining".to_owned()}else{primary};
        let id_display=if is_coinbase{String::new()}else{middle_trim(&address_or_tx,64,18)};
        let tx_count=format_count(&first_truthy(row,&["txCount","txs","transactions","txsDir"],JsValue::from_str("")));
        let flow_value=first_truthy(row,&["netFlow","amount"],JsValue::from_str(""));
        let flow=number_value(&flow_value).map(|n|format_number(n,2)).unwrap_or_else(||raw(&flow_value));
        let usd=usd_value(&flow_value,&first_truthy(row,&["valueUsd","value"],JsValue::from_str("")));
        let block=middle_trim(&raw(&property(row,"blockScore")),12,8);
        let raw_type=raw(&property(row,"type"));
        let type_display=if is_coinbase{String::new()}else if raw_type.is_empty(){"—".to_owned()}else{raw_type};
        let plus=if is_expanded{"⊟"}else{"⊞"}; let children=if is_expanded{render_children(&key,row)}else{String::new()};
        format!(r#"<tr class="kgw-analysis-result-row kgw-analysis-python-parent-row-r9 kgw-analysis-python-tree-parent-r14" data-row-key="{}"><td class="kgw-analysis-tree-name-r14" title="{}"><button type="button" class="kgw-analysis-python-toggle-r9 kgw-analysis-tree-toggle-r14" data-kgw-analysis-python-toggle-r9="true" data-row-key="{}" aria-expanded="{}" title="{}">{}</button><span class="kgw-analysis-tree-parent-label-r14">{}</span></td><td class="kgw-analysis-tree-id-r14" title="{}"><span class="kgw-analysis-mono">{}</span></td><td class="kgw-analysis-tree-txs-r14">{}</td><td class="kgw-analysis-tree-amount-r14">{}</td><td class="kgw-analysis-tree-value-r14">{}</td><td class="kgw-analysis-tree-block-r14">{}</td><td class="kgw-analysis-tree-type-r14">{}</td></tr>{}"#,html_escape(&key),html_escape(&parent),html_escape(&key),if is_expanded{"true"}else{"false"},if is_expanded{"Hide transactions"}else{"Show transactions"},plus,html_escape(&parent),html_escape(&address_or_tx),html_escape(&id_display),html_escape(if tx_count.is_empty(){"—"}else{&tx_count}),html_escape(if flow.is_empty(){"—"}else{&flow}),html_escape(&usd),html_escape(&block),html_escape(&type_display),children)
    }).collect::<Vec<_>>().join("");
    let _ = set_property(&body, "innerHTML", &JsValue::from_str(&html));
}

#[wasm_bindgen(js_name = analysisViewSetData)]
pub fn set_analysis_data(payload: JsValue) {
    EXPANDED_ROWS.with(|rows| rows.borrow_mut().clear());
    let source = if Array::is_array(&payload) {
        payload.clone()
    } else {
        let rows = property(&payload, "rows");
        if Array::is_array(&rows) {
            rows
        } else {
            let counterparties = property(&payload, "counterparties");
            if Array::is_array(&counterparties) {
                counterparties
            } else {
                Array::new().into()
            }
        }
    };
    let rows = Array::from(&source)
        .iter()
        .map(|row| normalize_row(&row))
        .collect::<Vec<_>>();
    let enabled = !rows.is_empty();
    ROWS.with(|state| *state.borrow_mut() = rows);
    let summary = property(&payload, "summary");
    set_summary(&if summary.is_object() {
        summary
    } else {
        Object::new().into()
    });
    set_controls_enabled(enabled);
    render_rows_internal();
}
#[wasm_bindgen(js_name = analysisViewRenderRows)]
pub fn render_rows() {
    render_rows_internal();
}
#[wasm_bindgen(js_name = analysisViewApplyFilter)]
pub fn apply_filter_export() {
    apply_filter();
}
#[wasm_bindgen(js_name = analysisViewFilteredRows)]
pub fn filtered_rows() -> Array {
    let output = Array::new();
    FILTERED_ROWS.with(|rows| {
        for row in rows.borrow().iter() {
            output.push(row);
        }
    });
    output
}
#[wasm_bindgen(js_name = analysisViewResetExpansion)]
pub fn reset_expansion() {
    EXPANDED_ROWS.with(|rows| rows.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaping_preserves_legacy_text_contract() {
        assert_eq!(html_escape("<a&b>"), "&lt;a&amp;b&gt;");
    }
    #[test]
    fn middle_trim_is_stable() {
        assert_eq!(middle_trim("abcdef", 2, 2), "abcdef");
        assert_eq!(middle_trim("abcdefghijklmnop", 4, 3), "abcd…nop");
    }
    #[test]
    fn title_and_direction_case_match_view_contract() {
        assert_eq!(title_case("coinbase_reward"), "Coinbase Reward");
        assert_eq!(direction_case("OUT"), "Out");
        assert_eq!(direction_case(""), "—");
    }
    #[test]
    fn number_text_accepts_display_suffixes() {
        assert_eq!(number_text("1,234.5 KAS"), Some(1234.5));
        assert_eq!(number_text(""), None);
    }
}
