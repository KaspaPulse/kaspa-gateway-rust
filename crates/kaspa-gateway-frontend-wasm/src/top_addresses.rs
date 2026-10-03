use js_sys::{Array, Date, Function, Intl::NumberFormat, JSON, Object, Promise, Reflect};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

const TRACE_PATCH: &str = "KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49D";
const TRACE_OWNER: &str = "top-addresses-installButtonHandlers-safe-owner";

struct TopState {
    rows: Vec<JsValue>,
    filtered_rows: Vec<JsValue>,
    prices: JsValue,
    sort_column: String,
    sort_direction: String,
    running: bool,
    loaded_once: bool,
    last_updated_text: String,
}

impl TopState {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            filtered_rows: Vec::new(),
            prices: Object::new().into(),
            sort_column: "rank".to_owned(),
            sort_direction: "asc".to_owned(),
            running: false,
            loaded_once: false,
            last_updated_text: "--".to_owned(),
        }
    }
}

thread_local! {
    static STATE: RefCell<TopState> = RefCell::new(TopState::new());
}

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

fn numeric(value: &JsValue) -> f64 {
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
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(target, "querySelectorAll", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED);
    if !is_present(&list) {
        return Vec::new();
    }
    let length = numeric(&property(&list, "length"));
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

fn create_element(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag)).unwrap_or(JsValue::UNDEFINED)
}

fn set_attr(target: &JsValue, name: &str, value: &str) {
    let _ = call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn append_child(parent: &JsValue, child: &JsValue) {
    let _ = call1(parent, "appendChild", child);
}
fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn prevent_default(event: &JsValue) {
    let _ = call0(event, "preventDefault");
}

fn root() -> JsValue {
    let doc = document();
    for selector in [
        "#top-addresses",
        "[data-tab='top-addresses']",
        "[data-tab-id='top-addresses']",
        ".top-addresses-tab",
    ] {
        let value = query(&doc, selector);
        if is_present(&value) {
            return value;
        }
    }
    JsValue::UNDEFINED
}

fn q(selector: &str) -> JsValue {
    let value = root();
    if is_present(&value) {
        query(&value, selector)
    } else {
        JsValue::UNDEFINED
    }
}

fn qa(selector: &str) -> Vec<JsValue> {
    let value = root();
    if is_present(&value) {
        query_all(&value, selector)
    } else {
        Vec::new()
    }
}

fn attribute(target: &JsValue, name: &str) -> String {
    call1(target, "getAttribute", &JsValue::from_str(name))
        .ok()
        .map(|value| truthy_string(&value))
        .unwrap_or_default()
}
fn button_meta(button: &JsValue) -> String {
    [
        truthy_string(&property(button, "id")),
        truthy_string(&property(button, "name")),
        truthy_string(&property(button, "textContent")),
        truthy_string(&property(button, "title")),
        attribute(button, "aria-label"),
        truthy_string(&property(&dataset(button), "action")),
    ]
    .join(" ")
}

fn button_by_terms(terms: &[&str]) -> JsValue {
    for button in qa("button") {
        let meta = button_meta(&button).to_lowercase();
        if terms.iter().any(|term| meta.contains(&term.to_lowercase())) {
            return button;
        }
    }
    JsValue::UNDEFINED
}

fn first_query(selectors: &[&str]) -> JsValue {
    for selector in selectors {
        let value = q(selector);
        if is_present(&value) {
            return value;
        }
    }
    JsValue::UNDEFINED
}

fn find_refresh_button() -> JsValue {
    let direct = first_query(&["#refreshTopAddresses", "#topAddressesRefresh"]);
    if is_present(&direct) {
        direct
    } else {
        button_by_terms(&["refresh", "reload", "fetch", "update", "تحديث", "جلب"])
    }
}

fn find_filter_button() -> JsValue {
    let direct = first_query(&["#topAddressesFilter", "#filterTopAddresses"]);
    if is_present(&direct) {
        direct
    } else {
        button_by_terms(&["filter", "فلتر", "تصفية"])
    }
}
fn find_reset_button() -> JsValue {
    let direct = first_query(&["#topAddressesReset", "#resetTopAddresses"]);
    if is_present(&direct) {
        direct
    } else {
        button_by_terms(&["reset filter", "reset", "إعادة", "اعادة", "مسح"])
    }
}

fn find_csv_button() -> JsValue {
    let direct = first_query(&["#topAddressesExportCsv", "#saveTopAddressesCsv"]);
    if is_present(&direct) {
        direct
    } else {
        button_by_terms(&["save as csv", "csv"])
    }
}

fn find_html_button() -> JsValue {
    let direct = first_query(&["#topAddressesExportHtml", "#saveTopAddressesHtml"]);
    if is_present(&direct) {
        direct
    } else {
        button_by_terms(&["save as html", "html"])
    }
}
fn find_pdf_button() -> JsValue {
    let direct = first_query(&["#topAddressesExportPdf", "#saveTopAddressesPdf"]);
    if is_present(&direct) {
        direct
    } else {
        button_by_terms(&["save as pdf", "pdf"])
    }
}

fn find_search_input() -> JsValue {
    let direct = first_query(&["#topAddressesSearch", "input[type='search']"]);
    if is_present(&direct) {
        return direct;
    }
    for node in qa("input") {
        let meta = [
            truthy_string(&property(&node, "id")),
            truthy_string(&property(&node, "name")),
            truthy_string(&property(&node, "placeholder")),
            attribute(&node, "aria-label"),
        ]
        .join(" ")
        .to_lowercase();
        if meta.contains("search by rank")
            || meta.contains("search")
            || meta.contains("address")
            || meta.contains("name")
        {
            return node;
        }
    }
    JsValue::UNDEFINED
}

fn table_parts() -> Option<(JsValue, JsValue)> {
    let table = {
        let direct = q("table");
        if is_present(&direct) {
            direct
        } else {
            q("[data-top-addresses-table]")
        }
    };
    if !is_present(&table) {
        return None;
    }
    let mut thead = query(&table, "thead");
    let mut tbody = query(&table, "tbody");
    if !is_present(&thead) {
        thead = create_element("thead");
        let _ = call1(&table, "prepend", &thead);
    }
    if !is_present(&tbody) {
        tbody = create_element("tbody");
        append_child(&table, &tbody);
    }
    Some((thead, tbody))
}

fn set_status(message: &str, state: &str) {
    let mut node = first_query(&[
        "#topAddressesStatus",
        "[data-top-addresses-status]",
        ".top-addresses-status",
    ]);
    if !is_present(&node) {
        node = create_element("div");
        let _ = set_property(&node, "id", &JsValue::from_str("topAddressesStatus"));
        let _ = set_property(
            &node,
            "className",
            &JsValue::from_str("top-addresses-status"),
        );
        let shell = {
            let shell = q(".ta-python-shell");
            if is_present(&shell) { shell } else { root() }
        };
        if is_present(&shell) {
            let _ = call1(&shell, "prepend", &node);
        }
    }
    if is_present(&node) {
        let _ = set_property(&node, "textContent", &JsValue::from_str(message));
        let _ = set_property(&dataset(&node), "state", &JsValue::from_str(state));
        set_attr(
            &node,
            "role",
            if state == "error" { "alert" } else { "status" },
        );
        set_attr(
            &node,
            "aria-live",
            if state == "error" {
                "assertive"
            } else {
                "polite"
            },
        );
    }
    let console = property(&global(), "console");
    let _ = call3(
        &console,
        "log",
        &JsValue::from_str("[KGW Top Addresses]"),
        &JsValue::from_str(state),
        &JsValue::from_str(message),
    );
}
fn set_last_updated(text: &str) {
    let value = if text.is_empty() { "--" } else { text };
    STATE.with(|state| state.borrow_mut().last_updated_text = value.to_owned());
    let explicit = first_query(&[
        "#topAddressesLastUpdated",
        "#topAddressesLastUpdatedValue",
        "[data-top-addresses-last-updated]",
    ]);
    if is_present(&explicit) {
        let _ = set_property(&explicit, "textContent", &JsValue::from_str(value));
        return;
    }
    for element in qa("*") {
        let content = truthy_string(&property(&element, "textContent"))
            .trim()
            .to_owned();
        let children = property(&element, "children");
        if content.starts_with("Last Updated:") && numeric(&property(&children, "length")) == 0.0 {
            let _ = set_property(
                &element,
                "textContent",
                &JsValue::from_str(&format!("Last Updated: {value}")),
            );
            break;
        }
    }
}
fn format_date_time(date: &Date) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

fn format_number(value: f64, digits: u32) -> String {
    let value = if value.is_finite() { value } else { 0.0 };
    let locales = Array::new();
    let options = Object::new();
    let _ = set_property(
        options.as_ref(),
        "minimumFractionDigits",
        &JsValue::from_f64(digits as f64),
    );
    let _ = set_property(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(digits as f64),
    );
    let formatter = NumberFormat::new(&locales, &options);
    formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(value))
        .map(|value| js_string(&value))
        .unwrap_or_else(|_| format!("{value:.digits$}", digits = digits as usize))
}

fn first_nullish(row: &JsValue, names: &[&str], fallback: JsValue) -> JsValue {
    for name in names {
        let value = property(row, name);
        if is_present(&value) {
            return value;
        }
    }
    fallback
}

fn first_truthy_text(row: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = property(row, name);
        if truthy(&value) {
            return js_string(&value);
        }
    }
    String::new()
}

fn price_usd(prices: &JsValue) -> f64 {
    let value = numeric(&property(prices, "usd"));
    if value.is_finite() { value } else { 0.0 }
}
fn normalize_row_with_price(row: &JsValue, index: usize, fallback_price: f64) -> JsValue {
    let rank_raw = first_nullish(
        row,
        &["rank", "Rank"],
        JsValue::from_f64((index + 1) as f64),
    );
    let mut rank = numeric(&rank_raw);
    if !rank.is_finite() || rank == 0.0 {
        rank = (index + 1) as f64;
    }
    let known_name = first_truthy_text(row, &["known_name", "KnownName", "Known Name"]);
    let address = first_truthy_text(row, &["address", "Address"]);
    let balance_raw = first_nullish(
        row,
        &["balance", "amount", "Balance"],
        JsValue::from_f64(0.0),
    );
    let mut balance = numeric(&balance_raw);
    if !balance.is_finite() || balance == 0.0 {
        balance = 0.0;
    }
    let value_raw = first_nullish(
        row,
        &["total_usd", "value_usd", "ValueUsd"],
        JsValue::from_f64(0.0),
    );
    let mut value_usd = numeric(&value_raw);
    if !value_usd.is_finite() || value_usd <= 0.0 {
        let price_raw = property(row, "kas_price_usd");
        let mut price = if is_present(&price_raw) {
            numeric(&price_raw)
        } else {
            fallback_price
        };
        if !price.is_finite() || price == 0.0 {
            price = 0.0;
        }
        value_usd = balance * price;
    }
    let normalized = Object::new();
    let _ = set_property(normalized.as_ref(), "rank", &JsValue::from_f64(rank));
    let _ = set_property(
        normalized.as_ref(),
        "known_name",
        &JsValue::from_str(&known_name),
    );
    let _ = set_property(normalized.as_ref(), "address", &JsValue::from_str(&address));
    let _ = set_property(normalized.as_ref(), "balance", &JsValue::from_f64(balance));
    let _ = set_property(
        normalized.as_ref(),
        "value_usd",
        &JsValue::from_f64(value_usd),
    );
    normalized.into()
}

fn current_search_text() -> String {
    let input = find_search_input();
    truthy_string(&property(&input, "value"))
        .trim()
        .to_lowercase()
}

fn row_search_text(row: &JsValue) -> String {
    [
        js_string(&property(row, "rank")),
        js_string(&property(row, "known_name")),
        js_string(&property(row, "address")),
        js_string(&property(row, "balance")),
        js_string(&property(row, "value_usd")),
    ]
    .join(" ")
    .to_lowercase()
}

fn locale_compare(a: &str, b: &str) -> Ordering {
    let left = JsValue::from_str(a);
    let value = call1(&left, "localeCompare", &JsValue::from_str(b))
        .ok()
        .map(|value| numeric(&value))
        .unwrap_or(0.0);
    if value < 0.0 {
        Ordering::Less
    } else if value > 0.0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn compare_rows(a: &JsValue, b: &JsValue, column: &str) -> Ordering {
    let av = property(a, column);
    let bv = property(b, column);
    match (av.as_f64(), bv.as_f64()) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        _ => locale_compare(&js_string(&av), &js_string(&bv)),
    }
}

fn apply_filter() {
    let search = current_search_text();
    let (rows, prices, column, direction) = STATE.with(|state| {
        let state = state.borrow();
        (
            state.rows.clone(),
            state.prices.clone(),
            state.sort_column.clone(),
            state.sort_direction.clone(),
        )
    });
    let fallback_price = price_usd(&prices);
    let mut filtered: Vec<JsValue> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| normalize_row_with_price(row, index, fallback_price))
        .filter(|row| search.is_empty() || row_search_text(row).contains(&search))
        .collect();
    filtered.sort_by(|a, b| {
        let order = compare_rows(a, b, &column);
        if direction == "desc" {
            order.reverse()
        } else {
            order
        }
    });
    STATE.with(|state| state.borrow_mut().filtered_rows = filtered);
    render_table();
}

fn toggle_sort(column: &str) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.sort_column == column {
            state.sort_direction = if state.sort_direction == "asc" {
                "desc".to_owned()
            } else {
                "asc".to_owned()
            };
        } else {
            state.sort_column = column.to_owned();
            state.sort_direction = if column == "rank" {
                "asc".to_owned()
            } else {
                "desc".to_owned()
            };
        }
    });
    apply_filter();
}

fn render_header(thead: &JsValue) {
    let _ = set_property(thead, "innerHTML", &JsValue::from_str(""));
    let tr = create_element("tr");
    for (label, key) in [
        ("Rank", "rank"),
        ("Known Name", "known_name"),
        ("Address", "address"),
        ("Balance (KAS)", "balance"),
        ("Value (USD)", "value_usd"),
    ] {
        let th = create_element("th");
        let _ = set_property(
            &th,
            "textContent",
            &JsValue::from_str(&format!("{label} ↕")),
        );
        let _ = set_property(
            &property(&th, "style"),
            "cursor",
            &JsValue::from_str("pointer"),
        );
        let key = key.to_owned();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            toggle_sort(&key);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &th,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
        append_child(&tr, &th);
    }
    append_child(thead, &tr);
}

fn translated_empty_text() -> String {
    let translator = property(&window(), "kgwT");
    if let Ok(translator) = translator.dyn_into::<Function>()
        && let Ok(value) = translator.call1(
            &window(),
            &JsValue::from_str("topAddresses.noTopAddressesLoaded"),
        )
    {
        let text = js_string(&value);
        if !text.is_empty() {
            return text;
        }
    }
    "No top addresses loaded.".to_owned()
}

fn render_table() {
    let Some((thead, tbody)) = table_parts() else {
        return;
    };
    render_header(&thead);
    let _ = set_property(&tbody, "innerHTML", &JsValue::from_str(""));
    let rows = STATE.with(|state| state.borrow().filtered_rows.clone());
    if rows.is_empty() {
        let tr = create_element("tr");
        let td = create_element("td");
        let _ = set_property(&td, "colSpan", &JsValue::from_f64(5.0));
        let _ = set_property(
            &td,
            "textContent",
            &JsValue::from_str(&translated_empty_text()),
        );
        append_child(&tr, &td);
        append_child(&tbody, &tr);
        return;
    }
    for row in rows {
        let tr = create_element("tr");
        let cells = [
            js_string(&property(&row, "rank")),
            js_string(&property(&row, "known_name")),
            js_string(&property(&row, "address")),
            format_number(numeric(&property(&row, "balance")), 2),
            format!(
                "{} USD",
                format_number(numeric(&property(&row, "value_usd")), 2)
            ),
        ];
        for value in cells {
            let td = create_element("td");
            let _ = set_property(&td, "textContent", &JsValue::from_str(&value));
            append_child(&tr, &td);
        }
        append_child(&tbody, &tr);
    }
}

fn escape_csv_text(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn storage_get(key: &str) -> JsValue {
    let storage = {
        let local = property(&global(), "localStorage");
        if is_present(&local) {
            local
        } else {
            property(&window(), "localStorage")
        }
    };
    call1(&storage, "getItem", &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED)
}

fn locale() -> String {
    let doc = document();
    let root = property(&doc, "documentElement");
    let from_attr = call1(&root, "getAttribute", &JsValue::from_str("lang"))
        .ok()
        .map(|value| truthy_string(&value))
        .unwrap_or_default();
    for value in [
        from_attr,
        truthy_string(&property(&window(), "kgwCurrentLocale")),
        truthy_string(&storage_get("kgw.locale")),
    ] {
        if !value.is_empty() {
            return value;
        }
    }
    "en".to_owned()
}

fn clean_address(address: &str) -> String {
    let mut value = address.trim().to_owned();
    while value
        .chars()
        .last()
        .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '\'' | '"' | '<' | '>'))
    {
        value.pop();
    }
    value
}

fn address_url(address: &str) -> String {
    let clean = clean_address(address);
    if clean.starts_with("kaspa:") {
        format!("https://explorer.kaspa.org/addresses/{clean}")
    } else {
        String::new()
    }
}

fn valid_currency(value: &str) -> Option<String> {
    let text = value.trim().to_uppercase();
    (text.len() == 3 && text.bytes().all(|byte| byte.is_ascii_uppercase())).then_some(text)
}

fn selected_currency() -> String {
    let selectors = [
        "#topAddressesCurrency",
        "[data-top-addresses-currency]",
        "[name='topAddressesCurrency']",
    ];
    for selector in selectors {
        let node = query(&document(), selector);
        if let Some(currency) = valid_currency(&truthy_string(&property(&node, "value"))) {
            return currency;
        }
    }
    for value in [
        property(&window(), "kgwSelectedCurrency"),
        property(&window(), "KGW_SELECTED_CURRENCY"),
        storage_get("kgw.currency"),
        storage_get("kgw.selectedCurrency"),
    ] {
        if let Some(currency) = valid_currency(&truthy_string(&value)) {
            return currency;
        }
    }
    "USD".to_owned()
}

fn value_for_currency(row: &JsValue, currency: &str) -> String {
    let lower = if currency.is_empty() {
        "usd".to_owned()
    } else {
        currency.to_lowercase()
    };
    let upper = if currency.is_empty() {
        "USD".to_owned()
    } else {
        currency.to_uppercase()
    };
    for key in [
        format!("value_{lower}"),
        format!("value_{upper}"),
        format!("value{upper}"),
        "value".to_owned(),
        "value_usd".to_owned(),
        "valueUSD".to_owned(),
    ] {
        let value = property(row, &key);
        if is_present(&value) && !js_string(&value).trim().is_empty() {
            return js_string(&value);
        }
    }
    String::new()
}

fn client_table() -> Result<JsValue, JsValue> {
    let (rows, last_updated) = STATE.with(|state| {
        let state = state.borrow();
        (state.filtered_rows.clone(), state.last_updated_text.clone())
    });
    if rows.is_empty() {
        return Err(js_error("No top address rows are available for export."));
    }
    let currency = selected_currency();
    let output = Object::new();
    let _ = set_property(
        output.as_ref(),
        "title",
        &JsValue::from_str("Kaspa Gateway Top Addresses"),
    );
    let _ = set_property(
        output.as_ref(),
        "subtitle",
        &JsValue::from_str(&format!(
            "Last Updated: {} | Currency: {}",
            if last_updated.is_empty() {
                "--"
            } else {
                &last_updated
            },
            currency
        )),
    );
    let headers = Array::new();
    for value in [
        "Rank".to_owned(),
        "Known Name".to_owned(),
        "Address".to_owned(),
        "Address URL".to_owned(),
        "Balance (KAS)".to_owned(),
        format!("Value ({currency})"),
    ] {
        headers.push(&JsValue::from_str(&value));
    }
    let _ = set_property(output.as_ref(), "headers", headers.as_ref());
    let output_rows = Array::new();
    for row in rows {
        let values = Array::new();
        for value in [
            js_string(&property(&row, "rank")),
            {
                let known = property(&row, "known_name");
                if is_present(&known) {
                    js_string(&known)
                } else {
                    js_string(&property(&row, "knownName"))
                }
            },
            js_string(&property(&row, "address")),
            address_url(&js_string(&property(&row, "address"))),
            js_string(&property(&row, "balance")),
            value_for_currency(&row, &currency),
        ] {
            values.push(&JsValue::from_str(&value));
        }
        output_rows.push(values.as_ref());
    }
    let _ = set_property(output.as_ref(), "rows", output_rows.as_ref());
    Ok(output.into())
}
fn native_dialog_filter(format: &str) -> JsValue {
    let ext = format.trim_start_matches('.').to_lowercase();
    let output = Object::new();
    let name = if ext.is_empty() {
        "Export files".to_owned()
    } else {
        format!("{} files", ext.to_uppercase())
    };
    let extensions = Array::new();
    if !ext.is_empty() {
        extensions.push(&JsValue::from_str(&ext));
    }
    let _ = set_property(output.as_ref(), "name", &JsValue::from_str(&name));
    let _ = set_property(output.as_ref(), "extensions", extensions.as_ref());
    output.into()
}

#[derive(Clone)]
struct PromptText {
    title: &'static str,
    message: &'static str,
    open: &'static str,
    cancel: &'static str,
    dir: &'static str,
}

fn prompt_text(locale: &str) -> PromptText {
    let locale = locale.to_lowercase();
    if locale.starts_with("ar") {
        PromptText {
            title: "تم الحفظ",
            message: "تم حفظ الملف بنجاح. هل تريد فتحه الآن؟",
            open: "فتح",
            cancel: "إلغاء",
            dir: "rtl",
        }
    } else if locale.starts_with("de") {
        PromptText {
            title: "Gespeichert",
            message: "Die Datei wurde gespeichert. Möchten Sie sie jetzt öffnen?",
            open: "Öffnen",
            cancel: "Abbrechen",
            dir: "ltr",
        }
    } else if locale.starts_with("es") {
        PromptText {
            title: "Guardado",
            message: "El archivo se ha guardado. ¿Quieres abrirlo ahora?",
            open: "Abrir",
            cancel: "Cancelar",
            dir: "ltr",
        }
    } else if locale.starts_with("fr") {
        PromptText {
            title: "Enregistré",
            message: "Le fichier a été enregistré. Voulez-vous l’ouvrir maintenant ?",
            open: "Ouvrir",
            cancel: "Annuler",
            dir: "ltr",
        }
    } else {
        PromptText {
            title: "Saved",
            message: "The file was saved successfully. Do you want to open it now?",
            open: "Open",
            cancel: "Cancel",
            dir: "ltr",
        }
    }
}

fn prompt_object(locale: &str) -> JsValue {
    let labels = prompt_text(locale);
    let output = Object::new();
    for (key, value) in [
        ("title", labels.title),
        ("message", labels.message),
        ("open", labels.open),
        ("cancel", labels.cancel),
        ("dir", labels.dir),
    ] {
        let _ = set_property(output.as_ref(), key, &JsValue::from_str(value));
    }
    output.into()
}

fn current_prompt_locale() -> String {
    let root = property(&document(), "documentElement");
    let lang = truthy_string(&property(&root, "lang"));
    if !lang.is_empty() {
        lang
    } else {
        let stored = truthy_string(&storage_get("kgw.language"));
        if stored.is_empty() {
            "en".to_owned()
        } else {
            stored
        }
    }
}

fn ensure_prompt_style() {
    let id = "kgw-export-centered-open-prompt-v10-style";
    let existing =
        call1(&document(), "getElementById", &JsValue::from_str(id)).unwrap_or(JsValue::UNDEFINED);
    if is_present(&existing) {
        return;
    }
    let style = create_element("style");
    let _ = set_property(&style, "id", &JsValue::from_str(id));
    let css = r#"
.kgw-export-open-prompt-v10-backdrop{position:fixed;inset:0;z-index:2147483000;display:flex;align-items:center;justify-content:center;padding:24px;background:rgba(0,0,0,.28);box-sizing:border-box}
.kgw-export-open-prompt-v10-card{width:min(440px,calc(100vw - 48px));min-height:132px;border-radius:14px;border:1px solid rgba(124,171,255,.28);background:#101827;color:#f8fbff;box-shadow:0 22px 70px rgba(0,0,0,.45);overflow:hidden;font-family:inherit}
.kgw-export-open-prompt-v10-card[dir="rtl"]{text-align:right}
.kgw-export-open-prompt-v10-header{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:15px 18px 10px;border-bottom:1px solid rgba(255,255,255,.08);font-weight:800;letter-spacing:.01em}
.kgw-export-open-prompt-v10-close{width:30px;height:30px;border:0;border-radius:10px;background:transparent;color:#d7e7ff;font-size:22px;line-height:1;cursor:pointer}
.kgw-export-open-prompt-v10-close:hover{background:rgba(255,255,255,.10)}
.kgw-export-open-prompt-v10-body{padding:18px;color:#e7eefb;font-size:14px;line-height:1.6}
.kgw-export-open-prompt-v10-actions{display:flex;justify-content:flex-end;gap:10px;padding:0 18px 18px}
.kgw-export-open-prompt-v10-card[dir="rtl"] .kgw-export-open-prompt-v10-actions{justify-content:flex-start}
.kgw-export-open-prompt-v10-button{min-width:94px;height:34px;border-radius:10px;border:1px solid rgba(124,171,255,.26);background:rgba(255,255,255,.08);color:#f8fbff;font-weight:700;cursor:pointer}
.kgw-export-open-prompt-v10-button:hover{background:rgba(255,255,255,.13)}
.kgw-export-open-prompt-v10-primary{border-color:rgba(112,180,255,.65);background:#4f88d9;color:#fff}
.kgw-export-open-prompt-v10-primary:hover{background:#5b96ee}
"#;
    let _ = set_property(&style, "textContent", &JsValue::from_str(css));
    let head = property(&document(), "head");
    append_child(&head, &style);
}

fn finish_prompt(
    backdrop: &JsValue,
    previous_active: &JsValue,
    resolver: &Function,
    settled: &Rc<Cell<bool>>,
    accepted: bool,
) {
    if settled.replace(true) {
        return;
    }
    let _ = call0(backdrop, "remove");
    if is_present(previous_active) {
        let _ = call0(previous_active, "focus");
    }
    let _ = resolver.call1(&JsValue::UNDEFINED, &JsValue::from_bool(accepted));
}

fn bind_prompt_click(
    node: &JsValue,
    backdrop: &JsValue,
    previous_active: &JsValue,
    resolver: &Function,
    settled: &Rc<Cell<bool>>,
    accepted: bool,
) {
    let backdrop = backdrop.clone();
    let previous_active = previous_active.clone();
    let resolver = resolver.clone();
    let settled = settled.clone();
    let callback = Closure::wrap(Box::new(move |_event: JsValue| {
        finish_prompt(&backdrop, &previous_active, &resolver, &settled, accepted);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        node,
        "addEventListener",
        &JsValue::from_str("click"),
        callback.as_ref().unchecked_ref(),
    );
    callback.forget();
}

pub(crate) async fn centered_open_prompt() -> Result<bool, JsValue> {
    ensure_prompt_style();
    let labels = prompt_text(&current_prompt_locale());
    let previous_active = property(&document(), "activeElement");
    let backdrop = create_element("div");
    let _ = set_property(
        &backdrop,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-backdrop"),
    );
    set_attr(&backdrop, "role", "presentation");
    set_attr(&backdrop, "tabindex", "-1");

    let card = create_element("div");
    let _ = set_property(
        &card,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-card"),
    );
    set_attr(&card, "role", "dialog");
    set_attr(&card, "aria-modal", "true");
    set_attr(&card, "aria-labelledby", "kgw-export-open-prompt-v10-title");
    set_attr(&card, "dir", labels.dir);

    let header = create_element("div");
    let _ = set_property(
        &header,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-header"),
    );
    let title = create_element("div");
    let _ = set_property(
        &title,
        "id",
        &JsValue::from_str("kgw-export-open-prompt-v10-title"),
    );
    let _ = set_property(&title, "textContent", &JsValue::from_str(labels.title));
    let close = create_element("button");
    let _ = set_property(&close, "type", &JsValue::from_str("button"));
    let _ = set_property(
        &close,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-close"),
    );
    set_attr(&close, "aria-label", labels.cancel);
    let _ = set_property(&close, "textContent", &JsValue::from_str("×"));

    let body = create_element("div");
    let _ = set_property(
        &body,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-body"),
    );
    let _ = set_property(&body, "textContent", &JsValue::from_str(labels.message));

    let actions = create_element("div");
    let _ = set_property(
        &actions,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-actions"),
    );
    let cancel = create_element("button");
    let _ = set_property(&cancel, "type", &JsValue::from_str("button"));
    let _ = set_property(
        &cancel,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-button"),
    );
    let _ = set_property(&cancel, "textContent", &JsValue::from_str(labels.cancel));
    let open = create_element("button");
    let _ = set_property(&open, "type", &JsValue::from_str("button"));
    let _ = set_property(
        &open,
        "className",
        &JsValue::from_str("kgw-export-open-prompt-v10-button kgw-export-open-prompt-v10-primary"),
    );
    let _ = set_property(&open, "textContent", &JsValue::from_str(labels.open));

    append_child(&header, &title);
    append_child(&header, &close);
    append_child(&actions, &cancel);
    append_child(&actions, &open);
    append_child(&card, &header);
    append_child(&card, &body);
    append_child(&card, &actions);
    append_child(&backdrop, &card);

    let backdrop_for_promise = backdrop.clone();
    let previous_for_promise = previous_active.clone();
    let close_for_promise = close.clone();
    let cancel_for_promise = cancel.clone();
    let open_for_promise = open.clone();
    let promise = Promise::new(&mut move |resolve, _reject| {
        let settled = Rc::new(Cell::new(false));
        bind_prompt_click(
            &close_for_promise,
            &backdrop_for_promise,
            &previous_for_promise,
            &resolve,
            &settled,
            false,
        );
        bind_prompt_click(
            &cancel_for_promise,
            &backdrop_for_promise,
            &previous_for_promise,
            &resolve,
            &settled,
            false,
        );
        bind_prompt_click(
            &open_for_promise,
            &backdrop_for_promise,
            &previous_for_promise,
            &resolve,
            &settled,
            true,
        );

        let backdrop_for_mouse = backdrop_for_promise.clone();
        let previous_for_mouse = previous_for_promise.clone();
        let resolver_for_mouse = resolve.clone();
        let settled_for_mouse = settled.clone();
        let mouse = Closure::wrap(Box::new(move |event: JsValue| {
            if settled_for_mouse.get() {
                return;
            }
            if Object::is(&property(&event, "target"), &backdrop_for_mouse) {
                finish_prompt(
                    &backdrop_for_mouse,
                    &previous_for_mouse,
                    &resolver_for_mouse,
                    &settled_for_mouse,
                    false,
                );
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &backdrop_for_promise,
            "addEventListener",
            &JsValue::from_str("mousedown"),
            mouse.as_ref().unchecked_ref(),
        );
        mouse.forget();

        let backdrop_for_key = backdrop_for_promise.clone();
        let previous_for_key = previous_for_promise.clone();
        let resolver_for_key = resolve.clone();
        let settled_for_key = settled.clone();
        let keydown = Closure::wrap(Box::new(move |event: JsValue| {
            if settled_for_key.get() {
                return;
            }
            let key = js_string(&property(&event, "key"));
            if key == "Escape" || key == "Enter" {
                prevent_default(&event);
                finish_prompt(
                    &backdrop_for_key,
                    &previous_for_key,
                    &resolver_for_key,
                    &settled_for_key,
                    key == "Enter",
                );
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &backdrop_for_promise,
            "addEventListener",
            &JsValue::from_str("keydown"),
            keydown.as_ref().unchecked_ref(),
        );
        keydown.forget();
    });

    append_child(&property(&document(), "body"), &backdrop);
    let _ = call0(&open, "focus");
    let result = JsFuture::from(promise).await?;
    Ok(crate::js_boolean(&result))
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
async fn native_save_path(format: &str, default_path: &str) -> Result<Option<String>, JsValue> {
    let dialog = dialog_api()?;
    let options = Object::new();
    let _ = set_property(options.as_ref(), "title", &JsValue::from_str("Save export"));
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
    if truthy(&selected) {
        Ok(Some(js_string(&selected)))
    } else {
        Ok(None)
    }
}

fn invoke_api() -> Result<Function, JsValue> {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let core_invoke = property(&core, "invoke");
    if let Ok(invoke) = core_invoke.dyn_into::<Function>() {
        return Ok(invoke);
    }
    let legacy = property(&tauri, "tauri");
    property(&legacy, "invoke")
        .dyn_into::<Function>()
        .map_err(|_| js_error("Tauri invoke API is not available."))
}

async fn invoke_command(command: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let invoke = invoke_api()?;
    let result = invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?;
    JsFuture::from(Promise::resolve(&result)).await
}

async fn ask_open(final_path: &str) -> Result<bool, JsValue> {
    if !centered_open_prompt().await? {
        return Ok(false);
    }
    let args = Object::new();
    let _ = set_property(args.as_ref(), "path", &JsValue::from_str(final_path));
    invoke_command("kgw_open_exported_file_v1", args.as_ref()).await?;
    Ok(true)
}

async fn export_backend(format: &str) -> Result<(), JsValue> {
    let default_args = Object::new();
    let _ = set_property(
        default_args.as_ref(),
        "reportType",
        &JsValue::from_str("TopAddresses"),
    );
    let _ = set_property(default_args.as_ref(), "format", &JsValue::from_str(format));
    let output_path = invoke_command("export_default_path", default_args.as_ref()).await?;
    let output_path = js_string(&output_path);
    let Some(selected_path) = native_save_path(format, &output_path).await? else {
        set_status("Export cancelled.", "info");
        return Ok(());
    };
    let request = Object::new();
    let _ = set_property(
        request.as_ref(),
        "reportType",
        &JsValue::from_str("TopAddresses"),
    );
    let _ = set_property(request.as_ref(), "format", &JsValue::from_str(format));
    let _ = set_property(
        request.as_ref(),
        "outputPath",
        &JsValue::from_str(&selected_path),
    );
    let _ = set_property(request.as_ref(), "addressFilter", &JsValue::NULL);
    let _ = set_property(request.as_ref(), "timeRange", &JsValue::from_str("all"));
    let _ = set_property(request.as_ref(), "limit", &JsValue::from_f64(100000.0));
    let _ = set_property(request.as_ref(), "locale", &JsValue::from_str(&locale()));
    let table = client_table()?;
    let _ = set_property(request.as_ref(), "clientTable", &table);
    let args = Object::new();
    let _ = set_property(args.as_ref(), "request", request.as_ref());
    let result = invoke_command("export_report", args.as_ref()).await?;
    let output_snake = property(&result, "output_path");
    let output_camel = property(&result, "outputPath");
    let final_path = if truthy(&output_snake) {
        js_string(&output_snake)
    } else if truthy(&output_camel) {
        js_string(&output_camel)
    } else {
        selected_path
    };
    set_status(&format!("Export completed: {final_path}"), "info");
    let _ = ask_open(&final_path).await?;
    Ok(())
}

fn spawn_export(format: &'static str) {
    spawn_local(async move {
        if let Err(error) = export_backend(format).await {
            set_status(&error_text(&error), "info");
        }
    });
}

fn export_csv() {
    spawn_export("csv");
}
fn export_html() {
    spawn_export("html");
}

fn export_pdf() {
    spawn_export("pdf");
}

async fn refresh_internal() {
    let already_running = STATE.with(|state| state.borrow().running);
    if already_running {
        set_status(
            "LOADING — Top addresses fetch is already running...",
            "loading",
        );
        return;
    }
    if invoke_api().is_err() {
        set_status("ERROR — Tauri invoke API is not available.", "error");
        return;
    }
    STATE.with(|state| state.borrow_mut().running = true);
    set_status("LOADING — Refreshing top addresses...", "loading");
    let args = Object::new();
    let _ = set_property(args.as_ref(), "limit", &JsValue::from_f64(10000.0));
    match invoke_command("fetch_top_addresses_rust", args.as_ref()).await {
        Ok(response) => {
            let rows_value = property(&response, "rows");
            let rows = if Array::is_array(&rows_value) {
                let array = Array::from(&rows_value);
                array.iter().collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let prices = property(&response, "prices");
            let prices = if prices.is_object() {
                prices
            } else {
                Object::new().into()
            };
            let row_count = rows.len();
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.rows = rows;
                state.prices = prices;
                state.loaded_once = true;
            });
            set_last_updated(&format_date_time(&Date::new_0()));
            apply_filter();
            if row_count == 0 {
                set_status(
                    "EMPTY — No top addresses were returned for the current request.",
                    "empty",
                );
            } else {
                set_status(
                    &format!("SUCCESS — Loaded {row_count} top addresses."),
                    "success",
                );
            }
        }
        Err(error) => {
            let console = property(&global(), "console");
            let _ = call1(&console, "error", &error);
            set_status(&format!("ERROR — {}", error_text(&error)), "error");
        }
    }
    STATE.with(|state| state.borrow_mut().running = false);
}
fn stringify(value: &JsValue) -> String {
    JSON::stringify(value)
        .ok()
        .map(|value| js_string(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned())
}

fn trace_ui(action: &str, phase: &str, details: &JsValue) {
    let safe_action = if action.is_empty() {
        "top-addresses-ui"
    } else {
        action
    };
    let safe_phase = if phase.is_empty() { "unknown" } else { phase };
    let nested = Object::new();
    let _ = set_property(nested.as_ref(), "patch", &JsValue::from_str(TRACE_PATCH));
    let _ = set_property(nested.as_ref(), "owner", &JsValue::from_str(TRACE_OWNER));
    let _ = set_property(nested.as_ref(), "action", &JsValue::from_str(safe_action));
    let _ = set_property(nested.as_ref(), "phase", &JsValue::from_str(safe_phase));
    let _ = set_property(nested.as_ref(), "details", details);
    let args = Object::new();
    let _ = set_property(args.as_ref(), "scope", &JsValue::from_str("top-addresses"));
    let _ = set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    let _ = set_property(args.as_ref(), "action", &JsValue::from_str(safe_action));
    let _ = set_property(args.as_ref(), "phase", &JsValue::from_str(safe_phase));
    let _ = set_property(
        args.as_ref(),
        "details",
        &JsValue::from_str(&stringify(nested.as_ref())),
    );
    let tauri = property(&window(), "__TAURI__");
    let invoke = {
        let core = property(&tauri, "core");
        let core_invoke = property(&core, "invoke");
        if core_invoke.dyn_ref::<Function>().is_some() {
            core_invoke
        } else {
            let direct = property(&tauri, "invoke");
            if direct.dyn_ref::<Function>().is_some() {
                direct
            } else {
                property(&window(), "__TAURI_INVOKE__")
            }
        }
    };
    if let Ok(invoke) = invoke.dyn_into::<Function>()
        && let Ok(result) = invoke.call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str("kgw_frontend_button_trace_v1"),
            args.as_ref(),
        )
    {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }
}

fn button_trace_details(button: &JsValue, event: &JsValue) -> JsValue {
    let details = Object::new();
    let _ = set_property(
        details.as_ref(),
        "trusted",
        &JsValue::from_bool(crate::js_boolean(&property(event, "isTrusted"))),
    );
    let _ = set_property(details.as_ref(), "element", &JsValue::from_str("button"));
    let _ = set_property(
        details.as_ref(),
        "id",
        &JsValue::from_str(&truthy_string(&property(button, "id"))),
    );
    let _ = set_property(
        details.as_ref(),
        "text",
        &JsValue::from_str(truthy_string(&property(button, "textContent")).trim()),
    );
    let _ = set_property(
        details.as_ref(),
        "dataset",
        &JsValue::from_str(&stringify(&dataset(button))),
    );
    details.into()
}

fn bind_button(button: JsValue, key: &'static str, handler: fn()) {
    if !is_present(&button) {
        return;
    }
    if truthy_string(&property(&dataset(&button), "kgwTopHandler")) == key {
        return;
    }
    let _ = set_property(&dataset(&button), "kgwTopHandler", &JsValue::from_str(key));
    let button_for_event = button.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        trace_ui(
            "top-addresses-click",
            "r49d-top-addresses-click",
            &button_trace_details(&button_for_event, &event),
        );
        prevent_default(&event);
        handler();
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &button,
        "addEventListener",
        &JsValue::from_str("click"),
        callback.as_ref().unchecked_ref(),
    );
    callback.forget();
}

fn reset_filter() {
    let search = find_search_input();
    if is_present(&search) {
        let _ = set_property(&search, "value", &JsValue::from_str(""));
    }
    apply_filter();
}

fn refresh_spawn() {
    spawn_local(refresh_internal());
}

fn install_button_handlers() {
    bind_button(find_refresh_button(), "refresh", refresh_spawn);
    bind_button(find_filter_button(), "filter", apply_filter);
    bind_button(find_reset_button(), "reset", reset_filter);
    bind_button(find_csv_button(), "csv", export_csv);
    bind_button(find_html_button(), "html", export_html);
    bind_button(find_pdf_button(), "pdf", export_pdf);

    let search = find_search_input();
    if !is_present(&search) || truthy(&property(&dataset(&search), "kgwTopSearch")) {
        return;
    }
    let _ = set_property(&dataset(&search), "kgwTopSearch", &JsValue::from_str("1"));
    let input = Closure::wrap(Box::new(move |_event: JsValue| {
        apply_filter();
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &search,
        "addEventListener",
        &JsValue::from_str("input"),
        input.as_ref().unchecked_ref(),
    );
    input.forget();

    let keydown = Closure::wrap(Box::new(move |event: JsValue| {
        if js_string(&property(&event, "key")) == "Enter" {
            prevent_default(&event);
            apply_filter();
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &search,
        "addEventListener",
        &JsValue::from_str("keydown"),
        keydown.as_ref().unchecked_ref(),
    );
    keydown.forget();
}

#[wasm_bindgen(js_name = topAddressesInitTab)]
pub fn top_addresses_init_tab() {
    install_button_handlers();
    if STATE.with(|state| state.borrow().loaded_once) {
        apply_filter();
    } else {
        refresh_spawn();
    }
}

#[wasm_bindgen(js_name = topAddressesRefresh)]
pub fn top_addresses_refresh() {
    refresh_spawn();
}

#[wasm_bindgen(js_name = topAddressesNormalizeRow)]
pub fn top_addresses_normalize_row(row: JsValue, index: u32) -> JsValue {
    let fallback = STATE.with(|state| price_usd(&state.borrow().prices));
    normalize_row_with_price(&row, index as usize, fallback)
}

#[wasm_bindgen(js_name = topAddressesEscapeCsv)]
pub fn top_addresses_escape_csv(value: JsValue) -> String {
    escape_csv_text(&js_string(&value))
}

#[wasm_bindgen(js_name = topAddressesEscapeHtml)]
pub fn top_addresses_escape_html(value: JsValue) -> String {
    escape_html_text(&js_string(&value))
}

#[wasm_bindgen(js_name = topAddressesAddressUrl)]
pub fn top_addresses_address_url(value: JsValue) -> String {
    address_url(&js_string(&value))
}

#[wasm_bindgen(js_name = topAddressesSelectedCurrency)]
pub fn top_addresses_selected_currency() -> String {
    selected_currency()
}

#[wasm_bindgen(js_name = topAddressesValueForCurrency)]
pub fn top_addresses_value_for_currency(row: JsValue, currency: String) -> String {
    value_for_currency(&row, &currency)
}

#[wasm_bindgen(js_name = topAddressesClientTable)]
pub fn top_addresses_client_table() -> Result<JsValue, JsValue> {
    client_table()
}

#[wasm_bindgen(js_name = topAddressesNativeDialogFilter)]
pub fn top_addresses_native_dialog_filter(format: String) -> JsValue {
    native_dialog_filter(&format)
}

#[wasm_bindgen(js_name = topAddressesPromptText)]
pub fn top_addresses_prompt_text(locale: String) -> JsValue {
    prompt_object(&locale)
}

#[wasm_bindgen(js_name = topAddressesContractSetPrices)]
pub fn top_addresses_contract_set_prices(value: JsValue) {
    STATE.with(|state| state.borrow_mut().prices = value);
}

#[wasm_bindgen(js_name = topAddressesContractSetFilteredRows)]
pub fn top_addresses_contract_set_filtered_rows(value: Array) {
    STATE.with(|state| state.borrow_mut().filtered_rows = value.iter().collect());
}

#[wasm_bindgen(js_name = topAddressesContractSetLastUpdatedText)]
pub fn top_addresses_contract_set_last_updated_text(value: String) {
    STATE.with(|state| state.borrow_mut().last_updated_text = value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_url_matches_legacy_cleanup() {
        assert_eq!(
            address_url(" kaspa:qabc<> "),
            "https://explorer.kaspa.org/addresses/kaspa:qabc"
        );
        assert_eq!(address_url("kaspatest:qabc"), "");
    }

    #[test]
    fn escaping_matches_legacy_contract() {
        assert_eq!(escape_csv_text("a\"b"), "\"a\"\"b\"");
        assert_eq!(
            escape_html_text("<a x=\"1\">'&</a>"),
            "&lt;a x=&quot;1&quot;&gt;&#39;&amp;&lt;/a&gt;"
        );
    }

    #[test]
    fn prompt_locales_preserve_direction_and_copy() {
        let ar = prompt_text("AR-SA");
        assert_eq!(ar.dir, "rtl");
        assert_eq!(ar.open, "فتح");
        let fr = prompt_text("fr");
        assert_eq!(fr.title, "Enregistré");
        assert_eq!(fr.dir, "ltr");
    }

    #[test]
    fn currency_validation_is_exact_three_ascii_letters() {
        assert_eq!(valid_currency(" sar "), Some("SAR".to_owned()));
        assert_eq!(valid_currency("EURO"), None);
        assert_eq!(valid_currency("12A"), None);
    }
}
