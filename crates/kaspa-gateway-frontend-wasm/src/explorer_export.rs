use js_sys::{Array, Date, Function, Intl::NumberFormat, JSON, Object, Promise, Reflect};
use std::cmp::Ordering;
use std::collections::HashSet;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

const TRACE_MARKER: &str = "KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3";

fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
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

fn is_present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if !is_present(target) {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn set_property(target: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(name), value).map(|_| ())
}

fn function(target: &JsValue, name: &str) -> Result<Function, JsValue> {
    property(target, name)
        .dyn_into::<Function>()
        .map_err(|_| js_error(format!("missing JS method {name}")))
}

fn call0(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    function(target, name)?.call0(target)
}

fn call1(target: &JsValue, name: &str, a: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)?.call1(target, a)
}

fn call2(target: &JsValue, name: &str, a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)?.call2(target, a, b)
}

fn call3(
    target: &JsValue,
    name: &str,
    a: &JsValue,
    b: &JsValue,
    c: &JsValue,
) -> Result<JsValue, JsValue> {
    function(target, name)?.call3(target, a, b, c)
}

fn js_string(value: &JsValue) -> String {
    if is_present(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    }
}

fn truthy(value: &JsValue) -> bool {
    is_present(value) && crate::js_boolean(value)
}

fn truthy_string(value: &JsValue) -> String {
    if truthy(value) {
        js_string(value)
    } else {
        String::new()
    }
}

fn number(value: &JsValue) -> f64 {
    crate::js_number(value)
}

fn error_text(error: &JsValue) -> String {
    let message = truthy_string(&property(error, "message"));
    if message.is_empty() {
        js_string(error)
    } else {
        message
    }
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    if !is_present(target) {
        return JsValue::UNDEFINED;
    }
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(target, "querySelectorAll", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED);
    if !is_present(&list) {
        return Vec::new();
    }
    let length = number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(is_present)
        })
        .collect()
}

fn get_attr(target: &JsValue, name: &str) -> String {
    call1(target, "getAttribute", &JsValue::from_str(name))
        .ok()
        .map(|value| truthy_string(&value))
        .unwrap_or_default()
}

fn set_attr(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn object(entries: &[(&str, JsValue)]) -> JsValue {
    let output = Object::new();
    for (key, value) in entries {
        let _ = set_property(output.as_ref(), key, value);
    }
    output.into()
}

fn safe_json(value: &JsValue) -> String {
    JSON::stringify(value)
        .ok()
        .map(String::from)
        .unwrap_or_else(|| js_string(value))
}

fn storage_get(key: &str) -> JsValue {
    let storage = property(&window(), "localStorage");
    call1(&storage, "getItem", &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

fn current_locale() -> String {
    let root = property(&document(), "documentElement");
    let attr = call1(&root, "getAttribute", &JsValue::from_str("lang"))
        .ok()
        .map(|value| truthy_string(&value))
        .unwrap_or_default();
    for value in [
        attr,
        truthy_string(&property(&window(), "kgwCurrentLocale")),
        truthy_string(&storage_get("kgw.locale")),
    ] {
        if !value.is_empty() {
            return value;
        }
    }
    "en".to_owned()
}

fn invoke_api() -> Result<Function, JsValue> {
    let tauri = property(&window(), "__TAURI__");
    let core_invoke = property(&property(&tauri, "core"), "invoke");
    if let Ok(invoke) = core_invoke.dyn_into::<Function>() {
        return Ok(invoke);
    }
    let legacy_invoke = property(&property(&tauri, "tauri"), "invoke");
    if let Ok(invoke) = legacy_invoke.dyn_into::<Function>() {
        return Ok(invoke);
    }
    let direct_invoke = property(&tauri, "invoke");
    if let Ok(invoke) = direct_invoke.dyn_into::<Function>() {
        return Ok(invoke);
    }
    property(&window(), "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .map_err(|_| js_error("Tauri invoke API is not available."))
}

async fn invoke_command(command: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let invoke = invoke_api()?;
    let result = invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?;
    JsFuture::from(Promise::resolve(&result)).await
}

async fn trace_export(phase: &str, details: &JsValue) -> bool {
    let payload = Object::new();
    let _ = set_property(payload.as_ref(), "marker", &JsValue::from_str(TRACE_MARKER));
    let _ = set_property(
        payload.as_ref(),
        "at",
        &JsValue::from_str(&String::from(Date::new_0().to_iso_string())),
    );
    if let Some(source) = details.dyn_ref::<Object>() {
        for key in Object::keys(source).iter() {
            let value = Reflect::get(source, &key).unwrap_or(JsValue::UNDEFINED);
            let _ = Reflect::set(payload.as_ref(), &key, &value);
        }
    }

    let console = property(&global(), "console");
    let _ = call3(
        &console,
        "info",
        &JsValue::from_str("[KGW][export][explorer][trace]"),
        &JsValue::from_str(phase),
        payload.as_ref(),
    );

    if invoke_api().is_err() {
        return false;
    }
    let args = object(&[
        ("scope", JsValue::from_str("explorer")),
        ("net", JsValue::from_str("ui")),
        ("action", JsValue::from_str("export")),
        ("phase", JsValue::from_str(phase)),
        ("details", JsValue::from_str(&safe_json(payload.as_ref()))),
    ]);
    match invoke_command("kgw_frontend_button_trace_v1", &args).await {
        Ok(_) => true,
        Err(error) => {
            let _ = call3(
                &console,
                "warn",
                &JsValue::from_str("[KGW][export][explorer][trace-failed]"),
                &JsValue::from_str(phase),
                &error,
            );
            false
        }
    }
}

fn spawn_trace(phase: &'static str, details: JsValue) {
    spawn_local(async move {
        let _ = trace_export(phase, &details).await;
    });
}

fn computed_style(node: &JsValue) -> JsValue {
    call1(&window(), "getComputedStyle", node).unwrap_or(JsValue::UNDEFINED)
}

fn export_button_info(button: &JsValue) -> JsValue {
    if !is_present(button) {
        return Object::new().into();
    }
    let rect = call0(button, "getBoundingClientRect").unwrap_or(JsValue::UNDEFINED);
    let style = computed_style(button);
    object(&[
        (
            "id",
            JsValue::from_str(&truthy_string(&property(button, "id"))),
        ),
        (
            "text",
            JsValue::from_str(truthy_string(&property(button, "textContent")).trim()),
        ),
        (
            "disabled",
            JsValue::from_bool(crate::js_boolean(&property(button, "disabled"))),
        ),
        (
            "ariaDisabled",
            JsValue::from_str(&get_attr(button, "aria-disabled")),
        ),
        (
            "hidden",
            JsValue::from_bool(crate::js_boolean(&property(button, "hidden"))),
        ),
        (
            "display",
            JsValue::from_str(&truthy_string(&property(&style, "display"))),
        ),
        (
            "visibility",
            JsValue::from_str(&truthy_string(&property(&style, "visibility"))),
        ),
        (
            "width",
            if is_present(&rect) {
                JsValue::from_f64(number(&property(&rect, "width")).round())
            } else {
                JsValue::NULL
            },
        ),
        (
            "height",
            if is_present(&rect) {
                JsValue::from_f64(number(&property(&rect, "height")).round())
            } else {
                JsValue::NULL
            },
        ),
    ])
}

fn clean_header(value: &str) -> String {
    value.replace('↕', "").trim().to_owned()
}

fn set_export_status(section: &JsValue, message: &str, level: &str) {
    let status = {
        let first = query(section, "#explorerStatus");
        if is_present(&first) {
            first
        } else {
            let second = query(section, "[data-explorer-status]");
            if is_present(&second) {
                second
            } else {
                query(section, ".explorer-status")
            }
        }
    };
    if is_present(&status) {
        let _ = set_property(&status, "hidden", &JsValue::FALSE);
        let _ = call1(&status, "removeAttribute", &JsValue::from_str("hidden"));
        let _ = set_property(&status, "textContent", &JsValue::from_str(message));
        let _ = set_property(
            &property(&status, "dataset"),
            "kgwExplorerExportStatus",
            &JsValue::from_str(level),
        );
    }
    let console = property(&global(), "console");
    let _ = call2(
        &console,
        "info",
        &JsValue::from_str("[KGW][export][explorer]"),
        &JsValue::from_str(message),
    );
}

fn selected_address(section: &JsValue) -> String {
    truthy_string(&property(&query(section, "#explorerAddress"), "value"))
        .trim()
        .to_owned()
}

fn is_kaspa_address_text(value: &str) -> bool {
    let text = value.trim().to_ascii_lowercase();
    if !text.starts_with("kaspa") {
        return false;
    }
    let Some(colon) = text.find(':') else {
        return false;
    };
    colon >= 5
        && text[5..colon]
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

fn explorer_table(section: &JsValue) -> JsValue {
    for selector in [
        "#explorerTransactionsTable",
        "[data-explorer-transactions-table]",
        "table",
    ] {
        let table = query(section, selector);
        if is_present(&table) {
            return table;
        }
    }
    JsValue::UNDEFINED
}

fn empty_explorer_row(row: &[String]) -> bool {
    let text = row.join(" ").trim().to_ascii_lowercase();
    text.is_empty()
        || text.contains("load an address")
        || text.contains("no explorer rows")
        || text.contains("enter a valid kaspa address")
        || text.contains("table cleared")
}

fn clean_url(value: &str) -> String {
    let mut text = value.trim().to_owned();
    while text
        .chars()
        .last()
        .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '\'' | '"' | '<' | '>'))
    {
        text.pop();
    }
    text
}

fn classify_url(value: &str) -> &'static str {
    let clean = clean_url(value);
    let lower = clean.to_ascii_lowercase();
    if lower.contains("/tx/")
        || lower.contains("/txs/")
        || lower.contains("/transaction/")
        || lower.contains("/transactions/")
    {
        "tx"
    } else if lower.contains("/address/")
        || lower.contains("/addresses/")
        || lower.contains("kaspa:")
    {
        "address"
    } else {
        ""
    }
}

fn unique_urls(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        let clean = clean_url(&value);
        if !clean.is_empty() && !output.contains(&clean) {
            output.push(clean);
        }
    }
    output
}

fn raw_export_clean_text(value: &str) -> String {
    value.trim().to_owned()
}

fn raw_export_string(value: &JsValue) -> String {
    raw_export_clean_text(&js_string(value))
}

fn raw_export_number_string(value: &JsValue, digits: u32) -> String {
    let number = number(value);
    if !number.is_finite() {
        return String::new();
    }
    let locales = Array::new();
    let options = Object::new();
    let _ = set_property(
        options.as_ref(),
        "minimumFractionDigits",
        &JsValue::from_f64(0.0),
    );
    let _ = set_property(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(digits as f64),
    );
    let formatter = NumberFormat::new(&locales, &options);
    formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(number))
        .map(|value| js_string(&value))
        .unwrap_or_default()
}

fn raw_export_tx_url_text(value: &str) -> String {
    let clean = raw_export_clean_text(value);
    if clean.len() >= 32 && clean.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        format!("https://explorer.kaspa.org/txs/{clean}")
    } else {
        String::new()
    }
}

fn raw_export_address_url_text(value: &str) -> String {
    let clean = raw_export_clean_text(value);
    if clean.starts_with("kaspa:") {
        format!("https://explorer.kaspa.org/addresses/{clean}")
    } else {
        String::new()
    }
}

fn push_raw_export_value(output: &mut Vec<String>, value: &JsValue) {
    let clean = raw_export_string(value);
    if !clean.is_empty() && !output.contains(&clean) {
        output.push(clean);
    }
}

fn raw_export_join_addresses(values: &Array) -> String {
    let mut output = Vec::new();
    for value in values.iter() {
        if Array::is_array(&value) {
            for item in Array::from(&value).iter() {
                push_raw_export_value(&mut output, &item);
            }
        } else {
            push_raw_export_value(&mut output, &value);
        }
    }
    output.join(" | ")
}

fn first_truthy_value(target: &JsValue, names: &[&str], fallback: &str) -> JsValue {
    for name in names {
        let value = property(target, name);
        if truthy(&value) {
            return value;
        }
    }
    JsValue::from_str(fallback)
}

fn first_nullish_value(target: &JsValue, names: &[&str]) -> JsValue {
    for name in names {
        let value = property(target, name);
        if is_present(&value) {
            return value;
        }
    }
    JsValue::from_f64(0.0)
}

fn js_or_zero_number(value: &JsValue) -> f64 {
    let value = number(value);
    if value.is_nan() || value == 0.0 {
        0.0
    } else {
        value
    }
}

fn raw_export_normalize_tx(row: &JsValue) -> JsValue {
    let txid = raw_export_string(&first_truthy_value(
        row,
        &["txid", "transactionId", "transaction_id", "id", "hash"],
        "",
    ));
    let from_values = Array::new();
    for name in ["from_address", "fromAddress", "from"] {
        from_values.push(&property(row, name));
    }
    let from_address = raw_export_join_addresses(&from_values);
    let to_values = Array::new();
    for name in ["to_address", "toAddress", "to"] {
        to_values.push(&property(row, name));
    }
    let to_address = raw_export_join_addresses(&to_values);
    let counterparty = raw_export_string(&first_truthy_value(
        row,
        &["counterparty", "counterParty"],
        "",
    ));
    let timestamp_ms = js_or_zero_number(&first_nullish_value(
        row,
        &["timestamp_ms", "timestampMs", "timestamp"],
    ));
    let amount = js_or_zero_number(&first_nullish_value(
        row,
        &["amount", "amount_kas", "amountKas"],
    ));
    let value = js_or_zero_number(&first_nullish_value(
        row,
        &["value", "value_usd", "valueUsd"],
    ));
    let transaction_url = raw_export_tx_url_text(&txid);
    let address_urls = Array::new();
    address_urls.push(&JsValue::from_str(&raw_export_address_url_text(
        &from_address,
    )));
    address_urls.push(&JsValue::from_str(&raw_export_address_url_text(
        &to_address,
    )));
    address_urls.push(&JsValue::from_str(&raw_export_address_url_text(
        &counterparty,
    )));
    let address_url = raw_export_join_addresses(&address_urls);

    object(&[
        (
            "datetime",
            JsValue::from_str(&raw_export_string(&first_truthy_value(
                row,
                &["datetime", "date_time", "dateTime"],
                "",
            ))),
        ),
        ("txid", JsValue::from_str(&txid)),
        (
            "direction",
            JsValue::from_str(&raw_export_string(&first_truthy_value(
                row,
                &["direction"],
                "unknown",
            ))),
        ),
        ("fromAddress", JsValue::from_str(&from_address)),
        ("toAddress", JsValue::from_str(&to_address)),
        ("counterparty", JsValue::from_str(&counterparty)),
        ("amount", JsValue::from_f64(amount)),
        (
            "blockScore",
            JsValue::from_str(&raw_export_string(&first_truthy_value(
                row,
                &["block_score", "blockScore", "block_height", "blockHeight"],
                "",
            ))),
        ),
        ("timestampMs", JsValue::from_f64(timestamp_ms)),
        (
            "type",
            JsValue::from_str(&raw_export_string(&first_truthy_value(
                row,
                &["type", "tx_type", "txType"],
                "transfer",
            ))),
        ),
        ("value", JsValue::from_f64(value)),
        (
            "date",
            JsValue::from_str(&raw_export_string(&first_truthy_value(
                row,
                &["date", "day"],
                "",
            ))),
        ),
        ("transactionUrl", JsValue::from_str(&transaction_url)),
        ("addressUrl", JsValue::from_str(&address_url)),
    ])
}

#[wasm_bindgen(js_name = explorerRawExportStringV2)]
pub fn explorer_raw_export_string_v2(value: JsValue) -> String {
    raw_export_string(&value)
}

#[wasm_bindgen(js_name = explorerRawExportNumberV2)]
pub fn explorer_raw_export_number_v2(value: JsValue, digits: u32) -> String {
    raw_export_number_string(&value, digits)
}

#[wasm_bindgen(js_name = explorerRawExportTxUrlV2)]
pub fn explorer_raw_export_tx_url_v2(value: JsValue) -> String {
    raw_export_tx_url_text(&raw_export_string(&value))
}

#[wasm_bindgen(js_name = explorerRawExportAddressUrlV2)]
pub fn explorer_raw_export_address_url_v2(value: JsValue) -> String {
    raw_export_address_url_text(&raw_export_string(&value))
}

#[wasm_bindgen(js_name = explorerRawExportJoinAddressesV2)]
pub fn explorer_raw_export_join_addresses_v2(values: Array) -> String {
    raw_export_join_addresses(&values)
}

#[wasm_bindgen(js_name = explorerRawExportNormalizeRawTxV2)]
pub fn explorer_raw_export_normalize_raw_tx_v2(row: JsValue) -> JsValue {
    raw_export_normalize_tx(&row)
}

fn raw_export_day_text(row: &JsValue) -> Option<String> {
    if !truthy(&property(row, "__kgwDaySummary")) {
        return None;
    }
    let day = raw_export_string(&property(row, "day"));
    if day.is_empty() {
        return None;
    }
    Some(day.chars().take(10).collect())
}

fn raw_export_datetime_desc(a: &JsValue, b: &JsValue) -> Ordering {
    let a_time = number(&property(a, "timestampMs"));
    let b_time = number(&property(b, "timestampMs"));
    if a_time != b_time {
        return b_time.partial_cmp(&a_time).unwrap_or(Ordering::Equal);
    }
    let a_text = js_string(&property(a, "datetime"));
    let b_text = js_string(&property(b, "datetime"));
    let left = JsValue::from_str(&b_text);
    let right = JsValue::from_str(&a_text);
    let value = call1(&left, "localeCompare", &right)
        .ok()
        .map(|value| number(&value))
        .unwrap_or(0.0);
    if value < 0.0 {
        Ordering::Less
    } else if value > 0.0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

async fn build_raw_export_table(section: &JsValue) -> Result<JsValue, JsValue> {
    let root = crate::explorer_runtime::explorer_clean2_section(section.clone());
    let state = crate::explorer_state::explorer_state();
    let selected = raw_export_string(&property(&state, "selectedAddress"));
    let raw_address = if selected.is_empty() {
        raw_export_string(&property(&query(&root, "#explorerAddress"), "value"))
    } else {
        selected
    };
    let address =
        crate::explorer_addresses::explorer_normalize_address(JsValue::from_str(&raw_address));
    if !crate::explorer_controls::explorer_is_kaspa_address(JsValue::from_str(&address)) {
        return Err(js_error(
            "Enter a valid Kaspa address before exporting raw transactions.",
        ));
    }

    let filtered = property(&state, "filteredRows");
    let visible_rows = if Array::is_array(&filtered) {
        Array::from(&filtered)
    } else {
        Array::new()
    };
    if visible_rows.length() == 0 {
        return Err(js_error(
            "No explorer rows are available for export. Fetch transactions first, then export.",
        ));
    }

    let summary_days = visible_rows
        .iter()
        .filter_map(|row| raw_export_day_text(&row))
        .collect::<Vec<_>>();

    let mut raw_rows = Vec::<JsValue>::new();
    let mut seen_txids = HashSet::<String>::new();
    if summary_days.is_empty() {
        for row in visible_rows.iter() {
            let normalized = raw_export_normalize_tx(&row);
            let txid = js_string(&property(&normalized, "txid"));
            if txid.is_empty() || !seen_txids.insert(txid) {
                continue;
            }
            raw_rows.push(normalized);
        }
    } else {
        const MAX_EXPORT_DAY_PAGES: u32 = 10_000;
        let page_size = crate::explorer_runtime::DAY_TRANSACTION_PAGE_SIZE;
        for day in &summary_days {
            let mut completed = false;
            for page_index in 0..MAX_EXPORT_DAY_PAGES {
                let offset = page_index.saturating_mul(page_size);
                let day_rows = crate::explorer_runtime::explorer_clean2_load_day_transaction_page(
                    root.clone(),
                    JsValue::from_str(&address),
                    day.clone(),
                    offset,
                )
                .await?;
                if !Array::is_array(&day_rows) {
                    completed = true;
                    break;
                }
                let page = Array::from(&day_rows);
                let page_len = page.length();
                for row in page.iter() {
                    let normalized = raw_export_normalize_tx(&row);
                    let txid = js_string(&property(&normalized, "txid"));
                    if txid.is_empty() || !seen_txids.insert(txid) {
                        continue;
                    }
                    raw_rows.push(normalized);
                }
                if page_len < page_size {
                    completed = true;
                    break;
                }
            }
            if !completed {
                return Err(js_error(
                    "Explorer raw export exceeded the bounded per-day page limit.",
                ));
            }
        }
    }

    raw_rows.sort_by(raw_export_datetime_desc);
    if raw_rows.is_empty() {
        return Err(js_error(
            "Explorer raw transaction export found no transaction rows. Expand/fetch data first, then export.",
        ));
    }

    let output_rows = Array::new();
    for row in &raw_rows {
        let values = Array::new();
        for value in [
            js_string(&property(row, "datetime")),
            js_string(&property(row, "txid")),
            js_string(&property(row, "direction")),
            js_string(&property(row, "fromAddress")),
            js_string(&property(row, "toAddress")),
            raw_export_number_string(&property(row, "amount"), 8),
            js_string(&property(row, "blockScore")),
            {
                let timestamp = property(row, "timestampMs");
                if truthy(&timestamp) {
                    js_string(&timestamp)
                } else {
                    String::new()
                }
            },
            js_string(&property(row, "type")),
            raw_export_number_string(&property(row, "value"), 2),
            js_string(&property(row, "date")),
            js_string(&property(row, "transactionUrl")),
            js_string(&property(row, "addressUrl")),
        ] {
            values.push(&JsValue::from_str(&value));
        }
        output_rows.push(values.as_ref());
    }

    let headers = Array::new();
    for header in [
        "Date/Time",
        "Transaction ID",
        "Direction",
        "From Address(es)",
        "To Address(es)",
        "Amount (KAS)",
        "Block Score",
        "timestamp",
        "Type:",
        "Value (USD)",
        "date",
        "Transaction URL",
        "Address URL",
    ] {
        headers.push(&JsValue::from_str(header));
    }

    let report = object(&[
        (
            "at",
            JsValue::from_str(&String::from(Date::new_0().to_iso_string())),
        ),
        ("days", JsValue::from_f64(summary_days.len() as f64)),
        ("rows", JsValue::from_f64(output_rows.length() as f64)),
        ("address", JsValue::from_str(&address)),
    ]);
    set_property(&window(), "__KGW_EXPLORER_RAW_EXPORT_LAST_V2", &report)?;

    Ok(object(&[
        (
            "title",
            JsValue::from_str("Kaspa Gateway Explorer Transactions"),
        ),
        (
            "subtitle",
            JsValue::from_str(&format!(
                "Address: {address} | Raw transactions: {}",
                output_rows.length()
            )),
        ),
        ("headers", headers.into()),
        ("rows", output_rows.into()),
    ]))
}

#[wasm_bindgen(js_name = explorerBuildRawExportTableV2)]
pub async fn explorer_build_raw_export_table_v2(section: JsValue) -> Result<JsValue, JsValue> {
    build_raw_export_table(&section).await
}

fn build_client_table(section: &JsValue) -> Result<JsValue, JsValue> {
    let table = explorer_table(section);
    if !is_present(&table) {
        return Err(js_error("Explorer transactions table was not found."));
    }

    let mut headers = query_all(&table, "thead th")
        .into_iter()
        .map(|node| clean_header(&truthy_string(&property(&node, "textContent"))))
        .collect::<Vec<_>>();

    struct Row {
        values: Vec<String>,
        tx_url: String,
        address_urls: String,
    }

    let mut rows = Vec::new();
    for tr in query_all(&table, "tbody tr") {
        let cells = query_all(&tr, "td");
        let values = cells
            .iter()
            .map(|cell| {
                truthy_string(&property(cell, "textContent"))
                    .trim()
                    .to_owned()
            })
            .collect::<Vec<_>>();
        if values.is_empty()
            || !values.iter().any(|value| !value.is_empty())
            || empty_explorer_row(&values)
        {
            continue;
        }

        let mut urls = Vec::new();
        for cell in &cells {
            for anchor in query_all(cell, "a[href]") {
                let attr = get_attr(&anchor, "href");
                let value = if attr.is_empty() {
                    truthy_string(&property(&anchor, "href"))
                } else {
                    attr
                };
                if !value.is_empty() {
                    urls.push(value);
                }
            }
        }
        let urls = unique_urls(urls);
        let tx_url = urls
            .iter()
            .find(|value| classify_url(value) == "tx")
            .cloned()
            .unwrap_or_default();
        let address_urls = urls
            .iter()
            .filter(|value| classify_url(value) == "address")
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");
        rows.push(Row {
            values,
            tx_url,
            address_urls,
        });
    }

    let include_tx_url = rows.iter().any(|row| !row.tx_url.is_empty());
    let include_address_url = rows.iter().any(|row| !row.address_urls.is_empty());
    if include_tx_url {
        headers.push("Transaction URL".to_owned());
    }
    if include_address_url {
        headers.push("Address URL".to_owned());
    }

    let output_rows = rows
        .into_iter()
        .map(|row| {
            let mut values = row.values;
            if include_tx_url {
                values.push(row.tx_url);
            }
            if include_address_url {
                values.push(row.address_urls);
            }
            values
        })
        .collect::<Vec<_>>();

    spawn_trace(
        "table-scan",
        object(&[
            ("headers", JsValue::from_f64(headers.len() as f64)),
            ("rows", JsValue::from_f64(output_rows.len() as f64)),
            ("includeTxUrl", JsValue::from_bool(include_tx_url)),
            ("includeAddressUrl", JsValue::from_bool(include_address_url)),
        ]),
    );

    if headers.is_empty() || output_rows.is_empty() {
        return Err(js_error(
            "No explorer rows are available for export. Fetch transactions first, then export.",
        ));
    }

    let output = Object::new();
    let _ = set_property(
        output.as_ref(),
        "title",
        &JsValue::from_str("Kaspa Gateway Explorer Transactions"),
    );
    let _ = set_property(
        output.as_ref(),
        "subtitle",
        &JsValue::from_str("Exported from Explorer tab"),
    );
    let header_array = Array::new();
    for header in headers {
        header_array.push(&JsValue::from_str(&header));
    }
    let _ = set_property(output.as_ref(), "headers", header_array.as_ref());
    let row_array = Array::new();
    for row in output_rows {
        let values = Array::new();
        for value in row {
            values.push(&JsValue::from_str(&value));
        }
        row_array.push(values.as_ref());
    }
    let _ = set_property(output.as_ref(), "rows", row_array.as_ref());
    Ok(output.into())
}

async fn client_table(section: &JsValue) -> Result<JsValue, JsValue> {
    let state = crate::explorer_state::explorer_state();
    if Array::is_array(&property(&state, "filteredRows")) {
        build_raw_export_table(section).await
    } else {
        build_client_table(section)
    }
}

fn kgw_export_t(key: &str) -> String {
    let candidates = [
        property(&window(), "kgwT"),
        property(&window(), "kgwTranslate"),
        property(&window(), "t"),
        property(&property(&window(), "__KGW_I18N__"), "t"),
        property(&property(&window(), "kgwI18n"), "t"),
        property(&property(&window(), "KGWI18n"), "t"),
        property(&property(&window(), "i18n"), "t"),
    ];
    for candidate in candidates {
        let Ok(function) = candidate.dyn_into::<Function>() else {
            continue;
        };
        let Ok(value) = function.call1(&JsValue::UNDEFINED, &JsValue::from_str(key)) else {
            continue;
        };
        let text = js_string(&value);
        if !text.is_empty() && text != key {
            return text;
        }
    }
    key.to_owned()
}

fn export_root(section: &JsValue) -> JsValue {
    if is_present(section)
        && property(section, "querySelector")
            .dyn_ref::<Function>()
            .is_some()
    {
        section.clone()
    } else {
        document()
    }
}

fn hide_export_result_box(section: &JsValue) {
    let root = export_root(section);
    let mut box_node = query(&root, "#explorerExportResult");
    if !is_present(&box_node) {
        box_node = call1(
            &document(),
            "getElementById",
            &JsValue::from_str("explorerExportResult"),
        )
        .unwrap_or(JsValue::UNDEFINED);
    }
    if is_present(&box_node) {
        let _ = set_property(
            &property(&box_node, "style"),
            "display",
            &JsValue::from_str("none"),
        );
        let _ = set_property(&box_node, "textContent", &JsValue::from_str(""));
    }
}

fn hide_export_status(section: &JsValue) {
    let root = export_root(section);
    let status = {
        let first = query(&root, "#explorerStatus");
        if is_present(&first) {
            first
        } else {
            let second = query(&root, "[data-explorer-status]");
            if is_present(&second) {
                second
            } else {
                query(&root, ".explorer-status")
            }
        }
    };
    if is_present(&status) {
        let _ = set_property(&status, "textContent", &JsValue::from_str(""));
        let _ = set_property(&status, "hidden", &JsValue::TRUE);
        set_attr(&status, "hidden", "");
    }
}

fn dialog_api() -> Result<JsValue, JsValue> {
    let dialog = property(&property(&window(), "__TAURI__"), "dialog");
    if !is_present(&dialog)
        || property(&dialog, "save").dyn_ref::<Function>().is_none()
        || property(&dialog, "ask").dyn_ref::<Function>().is_none()
    {
        return Err(js_error(
            "Tauri global dialog API is not available. Expected window.__TAURI__.dialog.save/ask.",
        ));
    }
    Ok(dialog)
}

fn native_dialog_filter(format: &str) -> JsValue {
    let ext = format.trim_start_matches('.').to_ascii_lowercase();
    let output = Object::new();
    let name = if ext.is_empty() {
        "Export files".to_owned()
    } else {
        format!("{} files", ext.to_ascii_uppercase())
    };
    let extensions = Array::new();
    if !ext.is_empty() {
        extensions.push(&JsValue::from_str(&ext));
    }
    let _ = set_property(output.as_ref(), "name", &JsValue::from_str(&name));
    let _ = set_property(output.as_ref(), "extensions", extensions.as_ref());
    output.into()
}

async fn prompt_export_save_path(
    section: &JsValue,
    format: &str,
    default_path: &str,
) -> Result<Option<String>, JsValue> {
    let dialog = dialog_api()?;
    let options = Object::new();
    let _ = set_property(
        options.as_ref(),
        "title",
        &JsValue::from_str(&kgw_export_t("explorer.export.modal.saveTitle")),
    );
    let _ = set_property(
        options.as_ref(),
        "defaultPath",
        &JsValue::from_str(default_path),
    );
    let filters = Array::new();
    filters.push(&native_dialog_filter(format));
    let _ = set_property(options.as_ref(), "filters", filters.as_ref());
    let result = function(&dialog, "save")?.call1(&dialog, options.as_ref())?;
    let selected = JsFuture::from(Promise::resolve(&result)).await?;

    let selected_text = if truthy(&selected) {
        Some(js_string(&selected))
    } else {
        None
    };
    let _ = trace_export(
        "native-save-dialog-result-v8",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("accepted", JsValue::from_bool(selected_text.is_some())),
            ("hasPath", JsValue::from_bool(selected_text.is_some())),
        ]),
    )
    .await;
    hide_export_result_box(section);
    hide_export_status(section);
    Ok(selected_text)
}

async fn copy_export_path_to_clipboard(path: &str) -> bool {
    let clipboard = property(&property(&window(), "navigator"), "clipboard");
    let writer = property(&clipboard, "writeText");
    if path.is_empty() || writer.dyn_ref::<Function>().is_none() {
        let _ = trace_export(
            "copy-path-skipped",
            &object(&[
                ("reason", JsValue::from_str("clipboard-api-unavailable")),
                ("outputPath", JsValue::from_str(path)),
            ]),
        )
        .await;
        return false;
    }
    let result = match writer
        .dyn_into::<Function>()
        .and_then(|writer| writer.call1(&clipboard, &JsValue::from_str(path)))
    {
        Ok(result) => result,
        Err(error) => {
            let _ = trace_export(
                "copy-path-error",
                &object(&[
                    ("outputPath", JsValue::from_str(path)),
                    ("error", JsValue::from_str(&error_text(&error))),
                ]),
            )
            .await;
            return false;
        }
    };
    match JsFuture::from(Promise::resolve(&result)).await {
        Ok(_) => {
            let _ = trace_export(
                "copy-path-success",
                &object(&[("outputPath", JsValue::from_str(path))]),
            )
            .await;
            true
        }
        Err(error) => {
            let _ = trace_export(
                "copy-path-error",
                &object(&[
                    ("outputPath", JsValue::from_str(path)),
                    ("error", JsValue::from_str(&error_text(&error))),
                ]),
            )
            .await;
            false
        }
    }
}

async fn open_exported_file_after_prompt(final_path: &str, format: &str) -> Result<bool, JsValue> {
    let accepted = super::top_addresses::centered_open_prompt().await?;
    let _ = trace_export(
        "centered-open-file-prompt-v10",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("accepted", JsValue::from_bool(accepted)),
            ("outputPath", JsValue::from_str(final_path)),
        ]),
    )
    .await;
    if !accepted {
        return Ok(false);
    }
    let args = object(&[("path", JsValue::from_str(final_path))]);
    match invoke_command("kgw_open_exported_file_v1", &args).await {
        Ok(_) => {
            let _ = trace_export(
                "open-file-success",
                &object(&[
                    ("format", JsValue::from_str(format)),
                    ("outputPath", JsValue::from_str(final_path)),
                ]),
            )
            .await;
            Ok(true)
        }
        Err(error) => {
            let _ = trace_export(
                "open-file-error",
                &object(&[
                    ("format", JsValue::from_str(format)),
                    ("outputPath", JsValue::from_str(final_path)),
                    ("error", JsValue::from_str(&error_text(&error))),
                ]),
            )
            .await;
            Ok(false)
        }
    }
}

async fn run_backend_export(
    section: JsValue,
    format: &'static str,
    source_button: JsValue,
) -> Result<(), JsValue> {
    let _ = trace_export(
        "run-start",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("button", export_button_info(&source_button)),
        ]),
    )
    .await;
    let _ = invoke_api()?;
    let table = client_table(&section).await?;
    let address = selected_address(&section);
    let rows = property(&table, "rows");
    let row_count = number(&property(&rows, "length"));

    let _ = trace_export(
        "default-path-start",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("reportType", JsValue::from_str("ExplorerTransactions")),
            ("rows", JsValue::from_f64(row_count)),
        ]),
    )
    .await;

    let default_args = object(&[
        ("reportType", JsValue::from_str("ExplorerTransactions")),
        ("format", JsValue::from_str(format)),
    ]);
    let output_path = invoke_command("export_default_path", &default_args).await?;
    let output_path = js_string(&output_path);
    let _ = trace_export(
        "default-path-done",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("outputPath", JsValue::from_str(&output_path)),
        ]),
    )
    .await;

    let Some(selected_path) = prompt_export_save_path(&section, format, &output_path).await? else {
        hide_export_result_box(&section);
        hide_export_status(&section);
        let _ = trace_export(
            "save-path-cancelled",
            &object(&[
                ("format", JsValue::from_str(format)),
                ("outputPath", JsValue::from_str(&output_path)),
            ]),
        )
        .await;
        return Ok(());
    };

    let request = Object::new();
    let _ = set_property(
        request.as_ref(),
        "reportType",
        &JsValue::from_str("ExplorerTransactions"),
    );
    let _ = set_property(request.as_ref(), "format", &JsValue::from_str(format));
    let _ = set_property(
        request.as_ref(),
        "outputPath",
        &JsValue::from_str(&selected_path),
    );
    let address_filter = if is_kaspa_address_text(&address) {
        JsValue::from_str(&address)
    } else {
        JsValue::NULL
    };
    let _ = set_property(request.as_ref(), "addressFilter", &address_filter);
    let _ = set_property(request.as_ref(), "timeRange", &JsValue::from_str("all"));
    let _ = set_property(request.as_ref(), "limit", &JsValue::from_f64(100000.0));
    let locale = current_locale();
    let _ = set_property(request.as_ref(), "locale", &JsValue::from_str(&locale));
    let _ = set_property(request.as_ref(), "clientTable", &table);

    let _ = trace_export(
        "export-report-start",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("outputPath", JsValue::from_str(&selected_path)),
            ("rows", JsValue::from_f64(row_count)),
            ("locale", JsValue::from_str(&locale)),
        ]),
    )
    .await;

    let args = object(&[("request", request.into())]);
    let result = invoke_command("export_report", &args).await?;
    let output_snake = property(&result, "output_path");
    let output_camel = property(&result, "outputPath");
    let final_path = if truthy(&output_snake) {
        js_string(&output_snake)
    } else if truthy(&output_camel) {
        js_string(&output_camel)
    } else {
        selected_path
    };
    let rows_exported = {
        let snake = property(&result, "rows_exported");
        if truthy(&snake) {
            snake
        } else {
            let camel = property(&result, "rowsExported");
            if truthy(&camel) {
                camel
            } else {
                JsValue::from_f64(row_count)
            }
        }
    };
    let bytes_written = {
        let snake = property(&result, "bytes_written");
        if truthy(&snake) {
            snake
        } else {
            let camel = property(&result, "bytesWritten");
            if truthy(&camel) { camel } else { JsValue::NULL }
        }
    };
    let _ = trace_export(
        "success",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("outputPath", JsValue::from_str(&final_path)),
            ("rowsExported", rows_exported),
            ("bytesWritten", bytes_written),
        ]),
    )
    .await;

    let copied = copy_export_path_to_clipboard(&final_path).await;
    hide_export_status(&section);
    let _ = trace_export(
        "native-save-result-v8",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("outputPath", JsValue::from_str(&final_path)),
            ("copiedToClipboard", JsValue::from_bool(copied)),
        ]),
    )
    .await;
    let _ = open_exported_file_after_prompt(&final_path, format).await?;
    Ok(())
}

fn nearest_explorer_section(node: &JsValue) -> JsValue {
    let nearest =
        call1(node, "closest", &JsValue::from_str("#explorer")).unwrap_or(JsValue::UNDEFINED);
    if is_present(&nearest) {
        nearest
    } else {
        let root = query(&document(), "#explorer");
        if is_present(&root) { root } else { document() }
    }
}

fn button_format(button: &JsValue) -> Option<&'static str> {
    match truthy_string(&property(button, "id")).as_str() {
        "explorerExportCsv" => Some("csv"),
        "explorerExportHtml" => Some("html"),
        "explorerExportPdf" => Some("pdf"),
        _ => None,
    }
}

fn find_export_button(event: &JsValue) -> JsValue {
    let target = property(event, "target");
    if property(&target, "closest").dyn_ref::<Function>().is_none() {
        return JsValue::UNDEFINED;
    }
    call1(
        &target,
        "closest",
        &JsValue::from_str("#explorerExportCsv, #explorerExportHtml, #explorerExportPdf"),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

async fn handle_export_button_click(event: JsValue) {
    let button = find_export_button(&event);
    if !is_present(&button) {
        return;
    }
    let Some(format) = button_format(&button) else {
        return;
    };
    let section = nearest_explorer_section(&button);
    let _ = call0(&event, "preventDefault");
    let _ = call0(&event, "stopPropagation");
    let _ = call0(&event, "stopImmediatePropagation");

    let meta = export_button_info(&button);
    let _ = trace_export(
        "r53b3-explorer-export-owner-click",
        &object(&[
            ("format", JsValue::from_str(format)),
            ("button", meta.clone()),
        ]),
    )
    .await;
    let _ = trace_export(
        "click",
        &object(&[("format", JsValue::from_str(format)), ("button", meta)]),
    )
    .await;

    let disabled = crate::js_boolean(&property(&button, "disabled"))
        || get_attr(&button, "aria-disabled") == "true";
    if disabled {
        set_export_status(
            &section,
            "Export button is disabled. Fetch transactions first, then export.",
            "blocked",
        );
        let _ = trace_export(
            "blocked-disabled",
            &object(&[
                ("format", JsValue::from_str(format)),
                ("button", export_button_info(&button)),
            ]),
        )
        .await;
        return;
    }

    if let Err(error) = run_backend_export(section.clone(), format, button).await {
        let message = error_text(&error);
        set_export_status(&section, &message, "error");
        let _ = trace_export(
            "error",
            &object(&[
                ("format", JsValue::from_str(format)),
                ("error", JsValue::from_str(&message)),
                (
                    "stack",
                    JsValue::from_str(&truthy_string(&property(&error, "stack"))),
                ),
            ]),
        )
        .await;
    }
}

fn install_click_owner_now() {
    let win = window();
    if crate::js_boolean(&property(
        &win,
        "__KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3_INSTALLED",
    )) {
        return;
    }
    let _ = set_property(
        &win,
        "__KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3_INSTALLED",
        &JsValue::TRUE,
    );

    let pointer = Closure::wrap(Box::new(move |event: JsValue| {
        let button = find_export_button(&event);
        if !is_present(&button) {
            return;
        }
        let Some(format) = button_format(&button) else {
            return;
        };
        spawn_trace(
            "pointerdown",
            object(&[
                ("format", JsValue::from_str(format)),
                ("button", export_button_info(&button)),
            ]),
        );
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &document(),
        "addEventListener",
        &JsValue::from_str("pointerdown"),
        pointer.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    pointer.forget();

    let click = Closure::wrap(Box::new(move |event: JsValue| {
        let button = find_export_button(&event);
        if !is_present(&button) {
            return;
        }
        spawn_local(handle_export_button_click(event));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &document(),
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    click.forget();

    spawn_trace(
        "owner-installed",
        object(&[
            (
                "csvExists",
                JsValue::from_bool(is_present(
                    &call1(
                        &document(),
                        "getElementById",
                        &JsValue::from_str("explorerExportCsv"),
                    )
                    .unwrap_or(JsValue::UNDEFINED),
                )),
            ),
            (
                "htmlExists",
                JsValue::from_bool(is_present(
                    &call1(
                        &document(),
                        "getElementById",
                        &JsValue::from_str("explorerExportHtml"),
                    )
                    .unwrap_or(JsValue::UNDEFINED),
                )),
            ),
            (
                "pdfExists",
                JsValue::from_bool(is_present(
                    &call1(
                        &document(),
                        "getElementById",
                        &JsValue::from_str("explorerExportPdf"),
                    )
                    .unwrap_or(JsValue::UNDEFINED),
                )),
            ),
        ]),
    );
}

#[wasm_bindgen(js_name = explorerExportInstall)]
pub fn explorer_export_install() {
    if truthy_string(&property(&document(), "readyState")) == "loading" {
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            install_click_owner_now();
        }) as Box<dyn FnMut(JsValue)>);
        let options = Object::new();
        let _ = set_property(options.as_ref(), "once", &JsValue::TRUE);
        let _ = call3(
            &document(),
            "addEventListener",
            &JsValue::from_str("DOMContentLoaded"),
            callback.as_ref().unchecked_ref(),
            options.as_ref(),
        );
        callback.forget();
    } else {
        install_click_owner_now();
    }
}

#[wasm_bindgen(js_name = explorerOpenBlockExplorer)]
pub fn explorer_open_block_explorer(section: JsValue) {
    let address = selected_address(&export_root(&section));
    spawn_trace(
        "open-block-explorer",
        object(&[(
            "hasAddress",
            JsValue::from_bool(is_kaspa_address_text(&address)),
        )]),
    );
    if !is_kaspa_address_text(&address) {
        return;
    }
    let encoded = property(&global(), "encodeURIComponent")
        .dyn_into::<Function>()
        .ok()
        .and_then(|function| {
            function
                .call1(&JsValue::UNDEFINED, &JsValue::from_str(&address))
                .ok()
        })
        .map(|value| js_string(&value))
        .unwrap_or(address);
    let _ = call2(
        &window(),
        "open",
        &JsValue::from_str(&format!("https://explorer.kaspa.org/addresses/{encoded}")),
        &JsValue::from_str("_blank"),
    );
}

fn spawn_export(section: JsValue, format: &'static str, button_id: &'static str) {
    let section = export_root(&section);
    let button = {
        let local = query(&section, &format!("#{button_id}"));
        if is_present(&local) {
            local
        } else {
            call1(&document(), "getElementById", &JsValue::from_str(button_id))
                .unwrap_or(JsValue::UNDEFINED)
        }
    };
    spawn_local(async move {
        if let Err(error) = run_backend_export(section.clone(), format, button).await {
            let message = error_text(&error);
            set_export_status(&section, &message, "error");
            let _ = trace_export(
                "error",
                &object(&[
                    ("format", JsValue::from_str(format)),
                    ("error", JsValue::from_str(&message)),
                    (
                        "stack",
                        JsValue::from_str(&truthy_string(&property(&error, "stack"))),
                    ),
                ]),
            )
            .await;
        }
    });
}

#[wasm_bindgen(js_name = explorerExportCsv)]
pub fn explorer_export_csv(section: JsValue) {
    spawn_export(section, "csv", "explorerExportCsv");
}

#[wasm_bindgen(js_name = explorerExportHtml)]
pub fn explorer_export_html(section: JsValue) {
    spawn_export(section, "html", "explorerExportHtml");
}

#[wasm_bindgen(js_name = explorerExportPdf)]
pub fn explorer_export_pdf(section: JsValue) {
    spawn_export(section, "pdf", "explorerExportPdf");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kaspa_address_contract_matches_legacy_shape() {
        assert!(is_kaspa_address_text("kaspa:qabc"));
        assert!(is_kaspa_address_text(" KASPATEST10:qabc "));
        assert!(!is_kaspa_address_text("bitcoin:qabc"));
        assert!(!is_kaspa_address_text("kaspa"));
        assert!(!is_kaspa_address_text("kaspa-foo:qabc"));
    }

    #[test]
    fn url_cleanup_and_classification_match_legacy_contract() {
        assert_eq!(clean_url(" https://x/txs/abc<> \t"), "https://x/txs/abc");
        assert_eq!(classify_url("https://x/tx/abc"), "tx");
        assert_eq!(classify_url("https://x/transactions/abc"), "tx");
        assert_eq!(classify_url("https://x/addresses/kaspa:q"), "address");
        assert_eq!(classify_url("https://x/other"), "");
    }

    #[test]
    fn unique_url_order_is_stable() {
        assert_eq!(
            unique_urls(vec![
                "https://x/txs/a".to_owned(),
                "https://x/txs/a<>".to_owned(),
                " https://x/addresses/kaspa:q ".to_owned(),
            ]),
            vec![
                "https://x/txs/a".to_owned(),
                "https://x/addresses/kaspa:q".to_owned(),
            ]
        );
    }

    #[test]
    fn raw_export_text_and_urls_match_legacy_contract() {
        assert_eq!(raw_export_clean_text("  abc  "), "abc");
        let txid = "ABCDEF0123456789abcdef0123456789";
        assert_eq!(
            raw_export_tx_url_text(txid),
            format!("https://explorer.kaspa.org/txs/{txid}")
        );
        assert_eq!(raw_export_tx_url_text("abcd"), "");
        assert_eq!(raw_export_tx_url_text(&"g".repeat(32)), "");
        assert_eq!(
            raw_export_address_url_text(" kaspa:qabc "),
            "https://explorer.kaspa.org/addresses/kaspa:qabc"
        );
        assert_eq!(raw_export_address_url_text("KASPA:qabc"), "");
    }

    #[test]
    fn raw_export_url_cleaning_keeps_internal_text_unchanged() {
        assert_eq!(
            raw_export_address_url_text("kaspa:q1 | kaspa:q2"),
            "https://explorer.kaspa.org/addresses/kaspa:q1 | kaspa:q2"
        );
        assert_eq!(
            raw_export_tx_url_text("0123456789abcdef0123456789abcdef "),
            "https://explorer.kaspa.org/txs/0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn empty_row_contract_preserves_legacy_sentinels() {
        assert!(empty_explorer_row(&["Load an address".to_owned()]));
        assert!(empty_explorer_row(&["table cleared".to_owned()]));
        assert!(!empty_explorer_row(&["real tx".to_owned()]));
    }
}
