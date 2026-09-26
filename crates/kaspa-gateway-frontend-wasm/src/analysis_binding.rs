use super::{
    call_method0, call_method1, js_boolean, js_number, js_string_owned, method, set_property,
};
use js_sys::{Array, Date, Function, JSON, Object, Promise, Reflect};
use std::cell::{Cell, RefCell};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

thread_local! {
    static CURRENT_ADDRESS: RefCell<String> = const { RefCell::new(String::new()) };
    static LAST_REPORT: RefCell<JsValue> = const { RefCell::new(JsValue::NULL) };
    static RUNNING: Cell<bool> = const { Cell::new(false) };
    static ADDRESSES_LOADED: Cell<bool> = const { Cell::new(false) };
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct AddressRow {
    address: String,
    label: String,
}

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    optional_property(&global(), "window")
}

fn document() -> JsValue {
    optional_property(&global(), "document")
}

fn optional_property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn call_method2(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    method(target, name)?.call2(target, first, second)
}

fn call_method3(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
    third: &JsValue,
) -> Result<JsValue, JsValue> {
    method(target, name)?.call3(target, first, second, third)
}

fn raw_string(value: &JsValue) -> String {
    js_string_owned(value)
}

fn truthy_text(value: &JsValue) -> String {
    if js_boolean(value) {
        raw_string(value)
    } else {
        String::new()
    }
}

fn root() -> JsValue {
    call_method1(
        &document(),
        "querySelector",
        &JsValue::from_str("#analysis"),
    )
    .unwrap_or(JsValue::UNDEFINED)
}

fn q(selector: &str) -> JsValue {
    let root = root();
    if !js_boolean(&root) {
        return JsValue::UNDEFINED;
    }
    call_method1(&root, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn invoke_function() -> Option<Function> {
    let win = window();
    let tauri = optional_property(&win, "__TAURI__");
    for owner_name in ["core", "tauri"] {
        let owner = optional_property(&tauri, owner_name);
        if let Ok(function) = optional_property(&owner, "invoke").dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}

async fn await_value(value: JsValue) -> Result<JsValue, JsValue> {
    if value.is_instance_of::<Promise>() {
        JsFuture::from(value.unchecked_into::<Promise>()).await
    } else {
        Ok(value)
    }
}

fn error_text(error: &JsValue) -> String {
    let message = optional_property(error, "message");
    if js_boolean(&message) {
        raw_string(&message)
    } else {
        raw_string(error)
    }
}

fn console_call(name: &str, values: &[JsValue]) {
    let console = optional_property(&global(), "console");
    let function_value = optional_property(&console, name);
    let Some(function) = function_value.dyn_ref::<Function>() else {
        return;
    };
    let args = Array::new();
    for value in values {
        args.push(value);
    }
    let _ = Reflect::apply(function, &console, &args);
}

fn invoke_now(command: &str, args: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let Some(call) = invoke_function() else {
        return Err(JsValue::from_str("Tauri invoke API is not available."));
    };
    match args {
        Some(args) => call.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args),
        None => call.call1(&JsValue::UNDEFINED, &JsValue::from_str(command)),
    }
}

fn absorb_async(value: JsValue) {
    spawn_local(async move {
        let _ = await_value(value).await;
    });
}

fn append_log(level: &str, message: &str) {
    let request = Object::new();
    let _ = set_property(request.as_ref(), "level", &JsValue::from_str(level));
    let _ = set_property(request.as_ref(), "target", &JsValue::from_str("analysis"));
    let _ = set_property(request.as_ref(), "message", &JsValue::from_str(message));
    let args = Object::new();
    let _ = set_property(args.as_ref(), "request", request.as_ref());
    if let Ok(value) = invoke_now("kgw_log_append", Some(args.as_ref())) {
        absorb_async(value);
    }
}

fn safe_json(value: &JsValue) -> String {
    JSON::stringify(value)
        .map(String::from)
        .unwrap_or_else(|_| raw_string(value))
}

fn log_analysis(message: &str, data: Option<&JsValue>) {
    let suffix = data.map(safe_json).unwrap_or_default();
    console_call(
        "log",
        &[
            JsValue::from_str("[KGW Analysis]"),
            JsValue::from_str(message),
            data.cloned().unwrap_or_else(|| JsValue::from_str("")),
        ],
    );
    if suffix.is_empty() {
        append_log("INFO", message);
    } else {
        append_log("INFO", &format!("{message} {suffix}"));
    }
}

fn log_analysis_error(message: &str, error: &JsValue) {
    let text = format!("{message}: {}", error_text(error));
    console_call(
        "error",
        &[
            JsValue::from_str("[KGW Analysis]"),
            JsValue::from_str(&text),
            error.clone(),
        ],
    );
    append_log("ERROR", &text);
}

fn dataset(element: &JsValue) -> JsValue {
    optional_property(element, "dataset")
}

fn set_status(message: &str, state: &str) {
    let node = q("#analysisStatus");
    if !js_boolean(&node) {
        return;
    }
    let _ = set_property(&node, "textContent", &JsValue::from_str(message));
    let data = dataset(&node);
    if !js_boolean(&data) {
        return;
    }
    if state.is_empty() {
        let object: Object = data.clone().unchecked_into();
        let _ = Reflect::delete_property(&object, &JsValue::from_str("state"));
    } else {
        let _ = set_property(&data, "state", &JsValue::from_str(state));
    }
}

fn selected_address() -> String {
    let select = q("#analysisAddressSelect");
    let select_value = truthy_text(&optional_property(&select, "value"))
        .trim()
        .to_owned();
    if !select_value.is_empty() {
        return select_value;
    }
    truthy_text(&optional_property(&q("#analysisAddressInput"), "value"))
        .trim()
        .to_owned()
}

fn update_run_state() {
    let running = RUNNING.get();
    let address = selected_address();
    let run = q("#analysisRun");
    if js_boolean(&run) {
        let _ = set_property(
            &run,
            "disabled",
            &JsValue::from_bool(running || address.is_empty()),
        );
    }
    let cancel = q("#analysisCancel");
    if js_boolean(&cancel) {
        let _ = set_property(&cancel, "disabled", &JsValue::from_bool(!running));
    }
}

fn translate_i18n(key: &str) -> String {
    let win = window();
    for name in ["kgwT", "kgwI18n"] {
        if let Some(function) = optional_property(&win, name).dyn_ref::<Function>()
            && let Ok(value) = function.call1(&win, &JsValue::from_str(key))
        {
            let text = truthy_text(&value);
            if !text.is_empty() && text != key {
                return text;
            }
        }
    }
    key.to_owned()
}

fn array_source(payload: &JsValue) -> Array {
    if Array::is_array(payload) {
        return payload.clone().unchecked_into::<Array>();
    }
    for key in ["addresses", "items", "rows"] {
        let value = optional_property(payload, key);
        if Array::is_array(&value) {
            return value.unchecked_into::<Array>();
        }
    }
    Array::new()
}

fn first_text_property(value: &JsValue, names: &[&str]) -> String {
    for name in names {
        let candidate = optional_property(value, name);
        let text = truthy_text(&candidate).trim().to_owned();
        if !text.is_empty() {
            return text;
        }
    }
    String::new()
}

fn normalize_address_rows(payload: &JsValue) -> Vec<AddressRow> {
    let source = array_source(payload);
    let mut rows = Vec::new();
    for index in 0..source.length() {
        let item = source.get(index);
        let address = if let Some(text) = item.as_string() {
            text.trim().to_owned()
        } else {
            first_text_property(
                &item,
                &["address", "kaspa_address", "wallet_address", "value"],
            )
        };
        if address.is_empty() || rows.iter().any(|row: &AddressRow| row.address == address) {
            continue;
        }
        let mut label = if item.as_string().is_some() {
            address.clone()
        } else {
            first_text_property(&item, &["name", "label", "address_name", "title"])
        };
        if label.is_empty() {
            label = address.clone();
        }
        rows.push(AddressRow { address, label });
    }
    rows.sort_by(|left, right| left.label.cmp(&right.label));
    rows
}

fn rows_to_js(rows: &[AddressRow]) -> Array {
    let output = Array::new();
    for row in rows {
        let value = Object::new();
        let _ = set_property(value.as_ref(), "address", &JsValue::from_str(&row.address));
        let _ = set_property(value.as_ref(), "label", &JsValue::from_str(&row.label));
        output.push(value.as_ref());
    }
    output
}

fn render_addresses(select: &JsValue, rows: &[AddressRow], current: &str) -> Result<(), JsValue> {
    set_property(select, "innerHTML", &JsValue::from_str(""))?;
    let document = document();
    let empty = call_method1(&document, "createElement", &JsValue::from_str("option"))?;
    set_property(&empty, "value", &JsValue::from_str(""))?;
    set_property(
        &empty,
        "textContent",
        &JsValue::from_str(&translate_i18n("ui.explorer.select.saved.address")),
    )?;
    let _ = call_method2(
        &empty,
        "setAttribute",
        &JsValue::from_str("data-i18n"),
        &JsValue::from_str("ui.explorer.select.saved.address"),
    )?;
    let _ = call_method1(select, "appendChild", &empty)?;

    for row in rows {
        let option = call_method1(&document, "createElement", &JsValue::from_str("option"))?;
        set_property(&option, "value", &JsValue::from_str(&row.address))?;
        let text = if row.label == row.address {
            row.address.clone()
        } else {
            format!("{} â€” {}", row.label, row.address)
        };
        set_property(&option, "textContent", &JsValue::from_str(&text))?;
        let _ = call_method1(select, "appendChild", &option)?;
    }
    if !current.is_empty() && rows.iter().any(|row| row.address == current) {
        set_property(select, "value", &JsValue::from_str(current))?;
    }
    Ok(())
}

async fn finish_load_saved_addresses(
    select: JsValue,
    current: String,
    pending: JsValue,
) -> Result<JsValue, JsValue> {
    let payload = await_value(pending).await?;
    let rows = normalize_address_rows(&payload);
    render_addresses(&select, &rows, &current)?;
    ADDRESSES_LOADED.set(true);
    if rows.is_empty() {
        set_status("No saved addresses found. Use manual input.", "warn");
    } else {
        set_status("Select an address to analyze.", "ok");
    }
    update_run_state();
    let data = Object::new();
    let _ = set_property(
        data.as_ref(),
        "count",
        &JsValue::from_f64(rows.len() as f64),
    );
    log_analysis("analysis addresses loaded", Some(data.as_ref()));
    Ok(rows_to_js(&rows).into())
}

fn begin_load_saved_addresses() {
    let select = q("#analysisAddressSelect");
    if !js_boolean(&select) {
        return;
    }
    let Some(_) = invoke_function() else {
        set_status(
            "Tauri invoke API is not available. Manual input is available.",
            "error",
        );
        update_run_state();
        return;
    };
    let current = truthy_text(&optional_property(&select, "value"));
    set_status("Loading saved addresses...", "loading");
    match invoke_now("get_all_addresses", None) {
        Ok(pending) => {
            spawn_local(async move {
                if let Err(error) = finish_load_saved_addresses(select, current, pending).await {
                    log_analysis_error("Failed to load analysis addresses", &error);
                    set_status(
                        "Could not load saved addresses. Manual input is still available.",
                        "error",
                    );
                }
            });
        }
        Err(error) => {
            log_analysis_error("Failed to load analysis addresses", &error);
            set_status(
                "Could not load saved addresses. Manual input is still available.",
                "error",
            );
        }
    }
}

#[wasm_bindgen(js_name = analysisLoadSavedAddresses)]
pub async fn load_saved_addresses() -> Result<JsValue, JsValue> {
    let select = q("#analysisAddressSelect");
    if !js_boolean(&select) {
        return Ok(Array::new().into());
    }
    if invoke_function().is_none() {
        set_status(
            "Tauri invoke API is not available. Manual input is available.",
            "error",
        );
        update_run_state();
        return Ok(Array::new().into());
    }
    let current = truthy_text(&optional_property(&select, "value"));
    set_status("Loading saved addresses...", "loading");
    let pending = invoke_now("get_all_addresses", None)?;
    finish_load_saved_addresses(select, current, pending).await
}

fn normalize_time_range_text(value: &str) -> String {
    let normalized = value.trim();
    match normalized {
        "" => "all".to_owned(),
        "30d" => "last_month".to_owned(),
        "90d" => "last_3_months".to_owned(),
        "1y" => "last_year".to_owned(),
        other => other.to_owned(),
    }
}

#[wasm_bindgen(js_name = analysisNormalizeTimeRange)]
pub fn normalize_time_range(value: JsValue) -> String {
    normalize_time_range_text(&raw_string(&value))
}

fn time_range() -> String {
    let mut node = q("#analysisTimeRange");
    if !js_boolean(&node) {
        node = q("[data-analysis-time-range]");
    }
    normalize_time_range_text(&truthy_text(&optional_property(&node, "value")))
}

fn format_metric_value(metric: &JsValue) -> String {
    if metric.is_null() || metric.is_undefined() {
        return "â€”".to_owned();
    }
    let value = optional_property(metric, "value");
    if !value.is_null() && !value.is_undefined() && !raw_string(&value).is_empty() {
        return raw_string(&value);
    }
    let raw_number = optional_property(metric, "raw_number");
    if !raw_number.is_null() && !raw_number.is_undefined() {
        return raw_string(&raw_number);
    }
    let raw_sompi = optional_property(metric, "raw_sompi");
    if !raw_sompi.is_null() && !raw_sompi.is_undefined() {
        let number = js_number(&raw_sompi) / 100_000_000.0;
        return raw_string(&JsValue::from_f64(number));
    }
    "â€”".to_owned()
}

fn metric_map(report: &JsValue) -> Vec<(String, String)> {
    let metrics_value = optional_property(report, "metrics");
    let metrics = if Array::is_array(&metrics_value) {
        metrics_value.unchecked_into::<Array>()
    } else {
        Array::new()
    };
    let mut map = Vec::new();
    for index in 0..metrics.length() {
        let metric = metrics.get(index);
        let label = truthy_text(&optional_property(&metric, "label")).to_lowercase();
        map.push((label, format_metric_value(&metric)));
    }
    map
}

fn pick_metric(map: &[(String, String)], names: &[&str]) -> String {
    for name in names {
        let key = name.to_lowercase();
        if let Some((_, value)) = map.iter().find(|(label, _)| label == &key) {
            return value.clone();
        }
    }
    "â€”".to_owned()
}

fn summary_from_report(report: &JsValue) -> Object {
    let map = metric_map(report);
    let output = Object::new();
    let total_metric = pick_metric(&map, &["Total Transactions"]);
    let total_transactions = if total_metric != "â€”" {
        total_metric
    } else {
        let raw = optional_property(report, "total_transactions");
        if raw.is_null() || raw.is_undefined() {
            "0".to_owned()
        } else {
            raw_string(&raw)
        }
    };
    for (key, value) in [
        ("totalInflow", pick_metric(&map, &["Total Inflow (KAS)"])),
        ("totalOutflow", pick_metric(&map, &["Total Outflow (KAS)"])),
        ("netFlow", pick_metric(&map, &["Net Flow (KAS)"])),
        ("avgInflow", pick_metric(&map, &["Avg Inflow (KAS)"])),
        ("avgOutflow", pick_metric(&map, &["Avg Outflow (KAS)"])),
        ("totalTransactions", total_transactions),
        (
            "largestInflow",
            pick_metric(&map, &["Largest Inflow (KAS)"]),
        ),
        (
            "largestOutflow",
            pick_metric(&map, &["Largest Outflow (KAS)"]),
        ),
        (
            "uniqueCounterparties",
            pick_metric(&map, &["Unique Counterparties"]),
        ),
        (
            "firstTransaction",
            pick_metric(&map, &["First Transaction"]),
        ),
        ("lastTransaction", pick_metric(&map, &["Last Transaction"])),
        ("durationDays", pick_metric(&map, &["Duration (Days)"])),
    ] {
        let _ = set_property(output.as_ref(), key, &JsValue::from_str(&value));
    }
    output
}

fn array_or_empty(value: JsValue) -> Array {
    if Array::is_array(&value) {
        value.unchecked_into::<Array>()
    } else {
        Array::new()
    }
}

fn js_or_default(value: JsValue, default: JsValue) -> JsValue {
    if js_boolean(&value) { value } else { default }
}

fn date_locale_string(value: JsValue) -> String {
    let number = js_number(&value);
    let date: JsValue = Date::new(&JsValue::from_f64(number)).into();
    call_method0(&date, "toLocaleString")
        .map(|value| raw_string(&value))
        .unwrap_or_default()
}

fn counterparty_rows(report: &JsValue) -> Option<Array> {
    let counterparties = array_or_empty(optional_property(report, "counterparties"));
    if counterparties.length() == 0 {
        return None;
    }
    let output = Array::new();
    for index in 0..counterparties.length() {
        let row = counterparties.get(index);
        let counterparty = truthy_text(&optional_property(&row, "counterparty"));
        let transactions = js_or_default(
            optional_property(&row, "transactions"),
            JsValue::from_f64(0.0),
        );
        let net = js_or_default(optional_property(&row, "net_kas"), JsValue::from_str(""));
        let details = array_or_empty(optional_property(&row, "details"));
        let object = Object::new();
        for (key, value) in [
            ("name", JsValue::from_str("Counterparty")),
            ("datetime", JsValue::from_str("")),
            ("address", JsValue::from_str(&counterparty)),
            ("transactionId", JsValue::from_str(&counterparty)),
            ("txsDir", JsValue::from_str(&raw_string(&transactions))),
            ("direction", JsValue::from_str("COUNTERPARTY")),
            ("netFlow", JsValue::from_str(&raw_string(&net))),
            ("amount", JsValue::from_str(&raw_string(&net))),
            ("valueUsd", JsValue::from_str("")),
            ("blockScore", JsValue::from_str("")),
            ("type", JsValue::from_str("COUNTERPARTY")),
            ("details", details.into()),
            ("transactions", transactions),
        ] {
            let _ = set_property(object.as_ref(), key, &value);
        }
        output.push(object.as_ref());
    }
    Some(output)
}

fn rows_from_report(report: &JsValue) -> Array {
    if let Some(rows) = counterparty_rows(report) {
        return rows;
    }
    let rows = array_or_empty(optional_property(report, "rows"));
    let output = Array::new();
    for index in 0..rows.length() {
        let row = rows.get(index);
        let timestamp = optional_property(&row, "timestamp_ms");
        let datetime = if js_boolean(&timestamp) {
            date_locale_string(timestamp.clone())
        } else {
            String::new()
        };
        let name = if datetime.is_empty() {
            "Transaction".to_owned()
        } else {
            datetime.clone()
        };
        let counterparty = truthy_text(&optional_property(&row, "counterparty"));
        let address = if counterparty.is_empty() {
            truthy_text(&optional_property(&row, "address"))
        } else {
            counterparty
        };
        let txid = truthy_text(&optional_property(&row, "txid"));
        let direction = truthy_text(&optional_property(&row, "direction"));
        let amount = optional_property(&row, "amount_kas");
        let tx_type = {
            let text = truthy_text(&optional_property(&row, "tx_type"));
            if text.is_empty() {
                "TX".to_owned()
            } else {
                text
            }
        };
        let details = array_or_empty(optional_property(&row, "details"));
        let object = Object::new();
        for (key, value) in [
            ("name", JsValue::from_str(&name)),
            ("datetime", JsValue::from_str(&datetime)),
            ("address", JsValue::from_str(&address)),
            ("transactionId", JsValue::from_str(&txid)),
            ("txsDir", JsValue::from_str(&direction)),
            ("direction", JsValue::from_str(&direction)),
            ("netFlow", JsValue::from_str(&raw_string(&amount))),
            ("amount", JsValue::from_str(&raw_string(&amount))),
            ("valueUsd", JsValue::from_str("")),
            ("blockScore", JsValue::from_str("")),
            ("type", JsValue::from_str(&tx_type)),
            ("details", details.into()),
        ] {
            let _ = set_property(object.as_ref(), key, &value);
        }
        output.push(object.as_ref());
    }
    output
}

fn custom_event(name: &str, detail: JsValue) -> Result<JsValue, JsValue> {
    let constructor = optional_property(&global(), "CustomEvent").dyn_into::<Function>()?;
    let init = Object::new();
    set_property(init.as_ref(), "detail", &detail)?;
    let args = Array::new();
    args.push(&JsValue::from_str(name));
    args.push(init.as_ref());
    Reflect::construct(&constructor, &args)
}

fn dispatch_analysis(detail: JsValue) {
    let win = window();
    if let Ok(event) = custom_event("kgw:analysis", detail) {
        let _ = call_method1(&win, "dispatchEvent", &event);
    }
}

fn emit_analysis_data(report: &JsValue) {
    let payload = Object::new();
    let summary = summary_from_report(report);
    let rows = rows_from_report(report);
    let counterparties = rows_from_report(report);
    let _ = set_property(payload.as_ref(), "summary", summary.as_ref());
    let _ = set_property(payload.as_ref(), "rows", rows.as_ref());
    let _ = set_property(payload.as_ref(), "counterparties", counterparties.as_ref());
    let _ = set_property(payload.as_ref(), "rawReport", report);
    dispatch_analysis(payload.clone().into());

    let win = window();
    if let Some(function) = optional_property(&win, "kgwSetAnalysisData").dyn_ref::<Function>() {
        let _ = function.call1(&win, payload.as_ref());
    }
}

fn set_export_enabled(enabled: bool) {
    for selector in [
        "#analysisExportCsv",
        "#analysisExportHtml",
        "#analysisExportPdf",
    ] {
        let node = q(selector);
        if js_boolean(&node) {
            let _ = set_property(&node, "disabled", &JsValue::from_bool(!enabled));
        }
    }
}

#[wasm_bindgen(js_name = analysisClear)]
pub fn clear_analysis_view() {
    LAST_REPORT.with(|report| *report.borrow_mut() = JsValue::NULL);
    set_export_enabled(false);
    let detail = Object::new();
    let rows = Array::new();
    let summary = Object::new();
    let _ = set_property(detail.as_ref(), "rows", rows.as_ref());
    let _ = set_property(detail.as_ref(), "summary", summary.as_ref());
    dispatch_analysis(detail.into());
}

fn analysis_request(address: &str) -> Object {
    let request = Object::new();
    let _ = set_property(request.as_ref(), "address", &JsValue::from_str(address));
    let _ = set_property(
        request.as_ref(),
        "time_range",
        &JsValue::from_str(&time_range()),
    );
    let _ = set_property(request.as_ref(), "limit", &JsValue::from_f64(5000.0));
    let _ = set_property(
        request.as_ref(),
        "include_all_saved_addresses",
        &JsValue::from_bool(false),
    );
    let args = Object::new();
    let _ = set_property(args.as_ref(), "request", request.as_ref());
    args
}

async fn run_analysis_internal() -> Result<JsValue, JsValue> {
    if RUNNING.get() {
        return Ok(JsValue::UNDEFINED);
    }
    if invoke_function().is_none() {
        set_status("Tauri invoke API is not available.", "error");
        return Ok(JsValue::UNDEFINED);
    }
    let address = selected_address();
    if address.is_empty() {
        set_status("Select or enter a Kaspa address before analysis.", "error");
        update_run_state();
        return Ok(JsValue::UNDEFINED);
    }

    RUNNING.set(true);
    CURRENT_ADDRESS.with(|current| *current.borrow_mut() = address.clone());
    set_export_enabled(false);
    update_run_state();
    clear_analysis_view();
    set_status("Running analysis_report...", "loading");

    let request = analysis_request(&address);
    let request_value: JsValue = request.clone().into();
    let request_inner = optional_property(&request_value, "request");
    let log_data = Object::new();
    let _ = set_property(log_data.as_ref(), "address", &JsValue::from_str(&address));
    let _ = set_property(
        log_data.as_ref(),
        "time_range",
        &optional_property(&request_inner, "time_range"),
    );
    log_analysis("analysis_report requested", Some(log_data.as_ref()));

    let outcome = match invoke_now("analysis_report", Some(request.as_ref())) {
        Ok(value) => await_value(value).await,
        Err(error) => Err(error),
    };

    match outcome {
        Ok(report) => {
            if !RUNNING.get() {
                set_status("Analysis cancelled.", "warn");
            } else {
                LAST_REPORT.with(|last| *last.borrow_mut() = report.clone());
                emit_analysis_data(&report);
                set_export_enabled(true);
                let total = optional_property(&report, "total_transactions");
                let total_text = if js_boolean(&total) {
                    raw_string(&total)
                } else {
                    "0".to_owned()
                };
                set_status(
                    &format!("Analysis complete. Transactions: {total_text}"),
                    "ok",
                );
                let data = Object::new();
                let _ = set_property(data.as_ref(), "address", &JsValue::from_str(&address));
                let _ = set_property(data.as_ref(), "total_transactions", &total);
                log_analysis("analysis_report complete", Some(data.as_ref()));
            }
        }
        Err(error) => {
            log_analysis_error("analysis_report failed", &error);
            set_status(&error_text(&error), "error");
            set_export_enabled(false);
        }
    }
    RUNNING.set(false);
    update_run_state();
    Ok(JsValue::UNDEFINED)
}

#[wasm_bindgen(js_name = analysisRun)]
pub async fn run_analysis() -> Result<JsValue, JsValue> {
    run_analysis_internal().await
}

pub(crate) fn event_trusted(event: &JsValue) -> bool {
    optional_property(event, "isTrusted")
        .as_bool()
        .unwrap_or(false)
}

fn prevent_default(event: &JsValue) {
    let _ = call_method0(event, "preventDefault");
}

pub(crate) fn trace(action: &str, phase: &str, details: JsValue) {
    let Some(call) = invoke_function() else {
        return;
    };
    let inner = Object::new();
    for (key, value) in [
        (
            "patch",
            JsValue::from_str("KGW_ANALYSIS_START_CANCEL_TRACE_PATCH_R50D3"),
        ),
        (
            "owner",
            JsValue::from_str("analysis-rust-binding-bindControls-owner"),
        ),
        ("action", JsValue::from_str(action)),
        ("phase", JsValue::from_str(phase)),
        (
            "details",
            if details.is_object() {
                details
            } else {
                Object::new().into()
            },
        ),
    ] {
        let _ = set_property(inner.as_ref(), key, &value);
    }
    let encoded = JSON::stringify(inner.as_ref())
        .map(JsValue::from)
        .unwrap_or_else(|_| JsValue::from_str("{}"));
    let args = Object::new();
    for (key, value) in [
        ("scope", JsValue::from_str("analysis")),
        ("net", JsValue::from_str("ui")),
        ("action", JsValue::from_str(action)),
        ("phase", JsValue::from_str(phase)),
        ("details", encoded),
    ] {
        let _ = set_property(args.as_ref(), key, &value);
    }
    if let Ok(value) = call.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        absorb_async(value);
    }
}

pub(crate) fn details(entries: &[(&str, JsValue)]) -> JsValue {
    let object = Object::new();
    for (key, value) in entries {
        let _ = set_property(object.as_ref(), key, value);
    }
    object.into()
}

fn bind_node_once(
    selector: &str,
    marker: &str,
    event_name: &str,
    callback: Closure<dyn FnMut(JsValue)>,
) {
    let node = q(selector);
    if !js_boolean(&node) {
        return;
    }
    let data = dataset(&node);
    if truthy_text(&optional_property(&data, marker)) == "true" {
        return;
    }
    let _ = set_property(&data, marker, &JsValue::from_str("true"));
    if call_method2(
        &node,
        "addEventListener",
        &JsValue::from_str(event_name),
        callback.as_ref().unchecked_ref(),
    )
    .is_ok()
    {
        callback.forget();
    }
}

fn bind_controls() {
    bind_node_once(
        "#analysisRun",
        "kgwAnalysisBound",
        "click",
        Closure::wrap(Box::new(move |event: JsValue| {
            prevent_default(&event);
            let run = q("#analysisRun");
            let address = selected_address();
            trace(
                "analysis-run",
                "r50d3-analysis-run-click",
                details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "id",
                        JsValue::from_str(&truthy_text(&optional_property(&run, "id"))),
                    ),
                    (
                        "text",
                        JsValue::from_str(
                            truthy_text(&optional_property(&run, "textContent")).trim(),
                        ),
                    ),
                    ("runningBefore", JsValue::from_bool(RUNNING.get())),
                    ("selectedAddress", JsValue::from_str(&address)),
                ]),
            );
            spawn_local(async {
                if let Err(error) = run_analysis_internal().await {
                    log_analysis_error("Analysis failed", &error);
                    set_status(&error_text(&error), "error");
                    RUNNING.set(false);
                    update_run_state();
                }
            });
        }) as Box<dyn FnMut(JsValue)>),
    );

    bind_node_once(
        "#analysisCancel",
        "kgwAnalysisBound",
        "click",
        Closure::wrap(Box::new(move |event: JsValue| {
            prevent_default(&event);
            let cancel = q("#analysisCancel");
            let current = CURRENT_ADDRESS.with(|value| value.borrow().clone());
            trace(
                "analysis-cancel",
                "r50d3-analysis-cancel-click",
                details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "id",
                        JsValue::from_str(&truthy_text(&optional_property(&cancel, "id"))),
                    ),
                    (
                        "text",
                        JsValue::from_str(
                            truthy_text(&optional_property(&cancel, "textContent")).trim(),
                        ),
                    ),
                    ("runningBefore", JsValue::from_bool(RUNNING.get())),
                    ("currentAddress", JsValue::from_str(&current)),
                ]),
            );
            RUNNING.set(false);
            set_status("Analysis cancelled.", "warn");
            update_run_state();
            append_log("WARN", "Analysis cancelled by user.");
        }) as Box<dyn FnMut(JsValue)>),
    );

    bind_node_once(
        "#analysisAddressSelect",
        "kgwAnalysisBound",
        "change",
        Closure::wrap(Box::new(move |event: JsValue| {
            let select = q("#analysisAddressSelect");
            let value = truthy_text(&optional_property(&select, "value"));
            trace(
                "analysis-address",
                "r50d3-analysis-address-select-change",
                details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "id",
                        JsValue::from_str(&truthy_text(&optional_property(&select, "id"))),
                    ),
                    (
                        "valueLength",
                        JsValue::from_f64(value.encode_utf16().count() as f64),
                    ),
                ]),
            );
            if !value.trim().is_empty() {
                let input = q("#analysisAddressInput");
                if js_boolean(&input) {
                    let _ = set_property(&input, "value", &JsValue::from_str(""));
                }
            }
            update_run_state();
        }) as Box<dyn FnMut(JsValue)>),
    );

    bind_node_once(
        "#analysisAddressInput",
        "kgwAnalysisBound",
        "input",
        Closure::wrap(Box::new(move |event: JsValue| {
            let input = q("#analysisAddressInput");
            let value = truthy_text(&optional_property(&input, "value"));
            trace(
                "analysis-address",
                "r50d3-analysis-address-input",
                details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "id",
                        JsValue::from_str(&truthy_text(&optional_property(&input, "id"))),
                    ),
                    (
                        "valueLength",
                        JsValue::from_f64(value.encode_utf16().count() as f64),
                    ),
                ]),
            );
            if !value.trim().is_empty() {
                let select = q("#analysisAddressSelect");
                if js_boolean(&select) {
                    let _ = set_property(&select, "value", &JsValue::from_str(""));
                }
            }
            update_run_state();
        }) as Box<dyn FnMut(JsValue)>),
    );
}

fn export_clean_id(value: &str) -> String {
    let mut clean = value.trim().to_owned();
    while clean
        .chars()
        .last()
        .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '\'' | '"' | '<' | '>'))
    {
        clean.pop();
    }
    clean
}

fn analysis_address_url(address: &str) -> String {
    let clean = export_clean_id(address);
    if clean.starts_with("kaspa:") {
        format!("https://explorer.kaspa.org/addresses/{clean}")
    } else {
        String::new()
    }
}

fn analysis_tx_url(txid: &str) -> String {
    let clean = export_clean_id(txid);
    let valid = clean.len() >= 32 && clean.bytes().all(|byte| byte.is_ascii_hexdigit());
    if valid {
        format!("https://explorer.kaspa.org/txs/{clean}")
    } else {
        String::new()
    }
}

fn parse_display_number(value: &str) -> Option<f64> {
    let normalized = value.replace(',', "");
    normalized
        .split_whitespace()
        .next()
        .and_then(|part| part.parse::<f64>().ok())
        .filter(|number| number.is_finite())
}

struct AnalysisExportGroup {
    key: String,
    name: String,
    address: String,
    rows: Vec<JsValue>,
    total_kas: f64,
    total_usd: f64,
}

fn first_export_text(row: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = optional_property(row, name);
        if js_boolean(&value) {
            let text = raw_string(&value);
            if !text.is_empty() {
                return text;
            }
        }
    }
    String::new()
}

fn export_groups(rows: Vec<JsValue>) -> Vec<AnalysisExportGroup> {
    let mut groups: Vec<AnalysisExportGroup> = Vec::new();
    for row in rows {
        let address = first_export_text(&row, &["address", "counterparty"]);
        let name = first_export_text(&row, &["name", "knownName"]);
        let key = if !address.is_empty() {
            address.clone()
        } else if !name.is_empty() {
            name.clone()
        } else {
            "Unknown Counterparty".to_owned()
        };
        let index = groups.iter().position(|group| group.key == key);
        let target = if let Some(index) = index {
            index
        } else {
            groups.push(AnalysisExportGroup {
                key: key.clone(),
                name: name.clone(),
                address: address.clone(),
                rows: Vec::new(),
                total_kas: 0.0,
                total_usd: 0.0,
            });
            groups.len() - 1
        };
        let kas = first_export_text(&row, &["amount", "netFlow"]);
        let usd = first_export_text(&row, &["valueUsd"]);
        if let Some(value) = parse_display_number(&kas) {
            groups[target].total_kas += value;
        }
        if let Some(value) = parse_display_number(&usd) {
            groups[target].total_usd += value;
        }
        groups[target].rows.push(row);
    }
    groups.sort_by(|left, right| {
        let left = if !left.name.is_empty() {
            &left.name
        } else if !left.address.is_empty() {
            &left.address
        } else {
            &left.key
        };
        let right = if !right.name.is_empty() {
            &right.name
        } else if !right.address.is_empty() {
            &right.address
        } else {
            &right.key
        };
        left.to_lowercase().cmp(&right.to_lowercase())
    });
    groups
}

fn export_row(values: &[String]) -> JsValue {
    let row = Array::new();
    for value in values {
        row.push(&JsValue::from_str(value));
    }
    row.into()
}

fn analysis_client_table() -> Result<JsValue, JsValue> {
    super::analysis_view::apply_filter_export();
    let filtered = super::analysis_view::filtered_rows();
    let rows = filtered.iter().collect::<Vec<_>>();
    let filtered_count = rows.len();
    let groups = export_groups(rows);
    let output_rows = Array::new();

    for group in &groups {
        let group_address = if !group.address.is_empty() {
            group.address.clone()
        } else {
            group.key.clone()
        };
        output_rows.push(&export_row(&[
            "Counterparty".to_owned(),
            group.name.clone(),
            group_address.clone(),
            analysis_address_url(&group_address),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ]));

        for row in &group.rows {
            let address = {
                let direct = first_export_text(row, &["address", "counterparty"]);
                if direct.is_empty() {
                    group.address.clone()
                } else {
                    direct
                }
            };
            let txid = first_export_text(row, &["transactionId", "txid"]);
            output_rows.push(&export_row(&[
                String::new(),
                {
                    let value = first_export_text(row, &["name", "knownName"]);
                    if value.is_empty() {
                        group.name.clone()
                    } else {
                        value
                    }
                },
                address.clone(),
                analysis_address_url(&address),
                first_export_text(row, &["datetime"]),
                txid.clone(),
                analysis_tx_url(&txid),
                first_export_text(row, &["direction", "txsDir"]),
                first_export_text(row, &["amount", "netFlow"]),
                first_export_text(row, &["valueUsd"]),
                first_export_text(row, &["blockScore"]),
                first_export_text(row, &["type"]),
            ]));
        }

        output_rows.push(&export_row(&[
            "Total".to_owned(),
            group.name.clone(),
            group_address.clone(),
            analysis_address_url(&group_address),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            group.total_kas.to_string(),
            group.total_usd.to_string(),
            String::new(),
            String::new(),
        ]));
        output_rows.push(&export_row(&vec![String::new(); 12]));
    }

    if output_rows.length() == 0 {
        return Err(JsValue::from_str(
            "No analysis rows are available for export.",
        ));
    }

    let headers = Array::new();
    for header in [
        "Counterparty",
        "Known Name",
        "Counterparty Address",
        "Counterparty URL",
        "Date/Time",
        "Transaction ID",
        "Transaction URL",
        "Direction",
        "Amount (KAS)",
        "Value (USD)",
        "Block Score",
        "Type",
    ] {
        headers.push(&JsValue::from_str(header));
    }
    let table = Object::new();
    let _ = set_property(
        table.as_ref(),
        "title",
        &JsValue::from_str("Kaspa Gateway Analysis Report"),
    );
    let _ = set_property(
        table.as_ref(),
        "subtitle",
        &JsValue::from_str(&format!(
            "Counterparty groups: {} | Rows: {}",
            groups.len(),
            filtered_count
        )),
    );
    let _ = set_property(table.as_ref(), "headers", headers.as_ref());
    let _ = set_property(table.as_ref(), "rows", output_rows.as_ref());
    Ok(table.into())
}

fn analysis_locale() -> String {
    let root = optional_property(&document(), "documentElement");
    let from_attr = call_method1(&root, "getAttribute", &JsValue::from_str("lang"))
        .ok()
        .map(|value| truthy_text(&value))
        .unwrap_or_default();
    for value in [
        from_attr,
        truthy_text(&optional_property(&window(), "kgwCurrentLocale")),
        {
            let storage = optional_property(&window(), "localStorage");
            call_method1(&storage, "getItem", &JsValue::from_str("kgw.locale"))
                .ok()
                .map(|value| truthy_text(&value))
                .unwrap_or_default()
        },
    ] {
        if !value.is_empty() {
            return value;
        }
    }
    "en".to_owned()
}

fn set_analysis_export_status(message: &str) {
    let status = q("#analysisStatus");
    if js_boolean(&status) {
        let _ = set_property(&status, "textContent", &JsValue::from_str(message));
    }
    let data = Object::new();
    let _ = set_property(data.as_ref(), "message", &JsValue::from_str(message));
    log_analysis("analysis export", Some(data.as_ref()));
}

fn analysis_dialog_api() -> Result<JsValue, JsValue> {
    let dialog = optional_property(&optional_property(&window(), "__TAURI__"), "dialog");
    if optional_property(&dialog, "save")
        .dyn_ref::<Function>()
        .is_none()
        || optional_property(&dialog, "ask")
            .dyn_ref::<Function>()
            .is_none()
    {
        return Err(JsValue::from_str(
            "Tauri global dialog API is not available. Expected window.__TAURI__.dialog.save/ask.",
        ));
    }
    Ok(dialog)
}

fn analysis_dialog_filter(format: &str) -> JsValue {
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

async fn analysis_native_save_path(
    format: &str,
    default_path: &str,
) -> Result<Option<String>, JsValue> {
    let dialog = analysis_dialog_api()?;
    let options = Object::new();
    let _ = set_property(options.as_ref(), "title", &JsValue::from_str("Save export"));
    let _ = set_property(
        options.as_ref(),
        "defaultPath",
        &JsValue::from_str(default_path),
    );
    let filters = Array::new();
    filters.push(&analysis_dialog_filter(format));
    let _ = set_property(options.as_ref(), "filters", filters.as_ref());
    let result = method(&dialog, "save")?.call1(&dialog, options.as_ref())?;
    let selected = await_value(result).await?;
    if js_boolean(&selected) {
        Ok(Some(raw_string(&selected)))
    } else {
        Ok(None)
    }
}

async fn analysis_export_backend(format: &'static str) -> Result<(), JsValue> {
    let default_args = Object::new();
    let _ = set_property(
        default_args.as_ref(),
        "reportType",
        &JsValue::from_str("Analysis"),
    );
    let _ = set_property(default_args.as_ref(), "format", &JsValue::from_str(format));
    let output_path = await_value(invoke_now(
        "export_default_path",
        Some(default_args.as_ref()),
    )?)
    .await?;
    let output_path = raw_string(&output_path);
    let Some(selected_path) = analysis_native_save_path(format, &output_path).await? else {
        set_analysis_export_status("Export cancelled.");
        return Ok(());
    };

    let address = {
        let select = truthy_text(&optional_property(&q("#analysisAddressSelect"), "value"));
        if select.trim().is_empty() {
            truthy_text(&optional_property(&q("#analysisAddressInput"), "value"))
        } else {
            select
        }
    };
    let time_range = {
        let value = truthy_text(&optional_property(&q("#analysisTimeRange"), "value"));
        if value.is_empty() {
            "all".to_owned()
        } else {
            value
        }
    };
    let request = Object::new();
    for (name, value) in [
        ("reportType", JsValue::from_str("Analysis")),
        ("format", JsValue::from_str(format)),
        ("outputPath", JsValue::from_str(&selected_path)),
        (
            "addressFilter",
            if address.trim().is_empty() {
                JsValue::NULL
            } else {
                JsValue::from_str(address.trim())
            },
        ),
        ("timeRange", JsValue::from_str(&time_range)),
        ("limit", JsValue::from_f64(100000.0)),
        ("locale", JsValue::from_str(&analysis_locale())),
        ("clientTable", analysis_client_table()?),
    ] {
        let _ = set_property(request.as_ref(), name, &value);
    }
    let args = Object::new();
    let _ = set_property(args.as_ref(), "request", request.as_ref());
    let result = await_value(invoke_now("export_report", Some(args.as_ref()))?).await?;
    let snake = optional_property(&result, "output_path");
    let camel = optional_property(&result, "outputPath");
    let final_path = if js_boolean(&snake) {
        raw_string(&snake)
    } else if js_boolean(&camel) {
        raw_string(&camel)
    } else {
        selected_path
    };
    set_analysis_export_status(&format!("Export completed: {final_path}"));

    if super::top_addresses::centered_open_prompt().await? {
        let open_args = Object::new();
        let _ = set_property(open_args.as_ref(), "path", &JsValue::from_str(&final_path));
        let _ = await_value(invoke_now(
            "kgw_open_exported_file_v1",
            Some(open_args.as_ref()),
        )?)
        .await?;
    }
    Ok(())
}

fn spawn_analysis_export(format: &'static str) {
    spawn_local(async move {
        if let Err(error) = analysis_export_backend(format).await {
            set_analysis_export_status(&error_text(&error));
        }
    });
}

fn bind_analysis_export_button(selector: &'static str, format: &'static str) {
    bind_node_once(
        selector,
        "kgwAnalysisExportPhase1",
        "click",
        Closure::wrap(Box::new(move |event: JsValue| {
            prevent_default(&event);
            let button = q(selector);
            trace(
                "analysis-export",
                "r50b-analysis-export-click",
                details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    ("format", JsValue::from_str(format)),
                    ("selector", JsValue::from_str(selector)),
                    (
                        "id",
                        JsValue::from_str(&truthy_text(&optional_property(&button, "id"))),
                    ),
                    (
                        "text",
                        JsValue::from_str(
                            truthy_text(&optional_property(&button, "textContent")).trim(),
                        ),
                    ),
                ]),
            );
            spawn_analysis_export(format);
        }) as Box<dyn FnMut(JsValue)>),
    );
}

fn bind_analysis_export_buttons() {
    bind_analysis_export_button("#analysisExportCsv", "csv");
    bind_analysis_export_button("#analysisExportHtml", "html");
    bind_analysis_export_button("#analysisExportPdf", "pdf");
}
fn bind_when_ready() {
    if !js_boolean(&root()) {
        return;
    }
    bind_controls();
    bind_analysis_export_buttons();
    update_run_state();
    if !ADDRESSES_LOADED.get() {
        begin_load_saved_addresses();
    }
}

fn add_global_listener(target: &JsValue, event: &str, callback: Closure<dyn FnMut(JsValue)>) {
    if call_method2(
        target,
        "addEventListener",
        &JsValue::from_str(event),
        callback.as_ref().unchecked_ref(),
    )
    .is_ok()
    {
        callback.forget();
    }
}

#[wasm_bindgen(js_name = analysisInstallBinding)]
pub fn install_binding() -> Result<(), JsValue> {
    let win = window();
    if js_boolean(&optional_property(&win, "__kgwRustAnalysisBindingV2")) {
        return Ok(());
    }
    set_property(
        &win,
        "__kgwRustAnalysisBindingV2",
        &JsValue::from_bool(true),
    )?;

    bind_when_ready();

    let document = document();
    let ready = Closure::wrap(Box::new(move |_event: JsValue| {
        bind_when_ready();
    }) as Box<dyn FnMut(JsValue)>);
    let options = Object::new();
    set_property(options.as_ref(), "once", &JsValue::from_bool(true))?;
    if call_method3(
        &document,
        "addEventListener",
        &JsValue::from_str("DOMContentLoaded"),
        ready.as_ref().unchecked_ref(),
        options.as_ref(),
    )
    .is_ok()
    {
        ready.forget();
    }

    let win = window();
    add_global_listener(
        &win,
        "kgw:tab:shown",
        Closure::wrap(Box::new(move |_event: JsValue| {
            ADDRESSES_LOADED.set(false);
            bind_when_ready();
        }) as Box<dyn FnMut(JsValue)>),
    );
    add_global_listener(
        &win,
        "kgw:saved-addresses-changed",
        Closure::wrap(Box::new(move |_event: JsValue| {
            ADDRESSES_LOADED.set(false);
            begin_load_saved_addresses();
        }) as Box<dyn FnMut(JsValue)>),
    );

    log_analysis(
        "Rust analysis binding installed: analysis_report owner.",
        None,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_range_mapping_matches_legacy_contract() {
        for (input, expected) in [
            ("", "all"),
            ("30d", "last_month"),
            ("90d", "last_3_months"),
            ("1y", "last_year"),
            ("all", "all"),
            ("custom", "custom"),
        ] {
            assert_eq!(normalize_time_range_text(input), expected);
        }
    }

    #[test]
    fn address_rows_are_unique_and_sorted() {
        let mut rows = [
            AddressRow {
                address: "kaspa:qbeta".to_owned(),
                label: "Beta".to_owned(),
            },
            AddressRow {
                address: "kaspa:qalpha".to_owned(),
                label: "Alpha".to_owned(),
            },
        ];
        rows.sort_by(|left, right| left.label.cmp(&right.label));
        assert_eq!(rows[0].address, "kaspa:qalpha");
        assert_eq!(rows[1].address, "kaspa:qbeta");
    }

    #[test]
    fn analysis_export_urls_preserve_legacy_contract() {
        assert_eq!(
            analysis_address_url(" kaspa:qabc<> "),
            "https://explorer.kaspa.org/addresses/kaspa:qabc"
        );
        assert_eq!(analysis_address_url("kaspatest:qabc"), "");
        let txid = "a".repeat(32);
        assert_eq!(
            analysis_tx_url(&txid),
            format!("https://explorer.kaspa.org/txs/{txid}")
        );
        assert_eq!(analysis_tx_url("not-a-tx"), "");
    }

    #[test]
    fn analysis_export_number_parsing_matches_display_values() {
        assert_eq!(parse_display_number("1,234.5 KAS"), Some(1234.5));
        assert_eq!(parse_display_number("-0.25"), Some(-0.25));
        assert_eq!(parse_display_number(""), None);
    }
}
