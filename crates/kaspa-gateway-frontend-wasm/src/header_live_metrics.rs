use js_sys::{Array, Date, Function, Intl::NumberFormat, Object, Promise, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

const LIVE_METRICS_REFRESH_MS: f64 = 15_000.0;
const HEADER_CLOCK_TICK_MS: f64 = 1_000.0;
const R81C_OWNER: &str = "KGW_HEADER_PRICE_SELECTED_CURRENCY_OWNER_R81C";
const R85B_OWNER: &str = "KGW_HEADER_SELECTED_CURRENCY_CANONICAL_WRITER_R85B";
const R83B_OWNER: &str = "KGW_HEADER_PRICE_INIT_BINDING_OWNER_R83B";
const RESOLVE_INVOKE: &str = "__kgwHeaderRustResolveInvoke";

thread_local! {
    static LIVE_STARTED: RefCell<bool> = const { RefCell::new(false) };
    static CLOCK_TIMER: RefCell<JsValue> = const { RefCell::new(JsValue::UNDEFINED) };
    static METRICS_TIMER: RefCell<JsValue> = const { RefCell::new(JsValue::UNDEFINED) };
    static HEADER_CLOCK_TIMER: RefCell<JsValue> = const { RefCell::new(JsValue::UNDEFINED) };
    static TOOLTIP_TIMER: RefCell<JsValue> = const { RefCell::new(JsValue::UNDEFINED) };
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
fn property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
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
fn call1(target: &JsValue, name: &str, arg: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)?.call1(target, arg)
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
fn is_present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}
fn truthy(value: &JsValue) -> bool {
    value
        .as_bool()
        .unwrap_or_else(|| is_present(value) && crate::js_boolean(value))
}
fn raw_string(value: &JsValue) -> String {
    if is_present(value) {
        crate::js_string_owned(value)
    } else {
        String::new()
    }
}
fn truthy_text(value: &JsValue) -> String {
    if truthy(value) {
        raw_string(value)
    } else {
        String::new()
    }
}
fn numeric(value: &JsValue) -> f64 {
    crate::js_number(value)
}
fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}
fn set_dataset(target: &JsValue, name: &str, value: &str) -> Result<(), JsValue> {
    set_property(&dataset(target), name, &JsValue::from_str(value))
}
fn set_attr(target: &JsValue, name: &str, value: &str) -> Result<(), JsValue> {
    call2(
        target,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    )
    .map(|_| ())
}
fn remove_attr(target: &JsValue, name: &str) {
    let _ = call1(target, "removeAttribute", &JsValue::from_str(name));
}
fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}
fn get_by_id(id: &str) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(id)).unwrap_or(JsValue::UNDEFINED)
}
fn set_text(target: &JsValue, value: &str) -> Result<(), JsValue> {
    set_property(target, "textContent", &JsValue::from_str(value))
}
fn console_log(message: &str) {
    let console = property(&global(), "console");
    if let Ok(log) = property(&console, "log").dyn_into::<Function>() {
        let _ = log.call1(&console, &JsValue::from_str(message));
    }
}
fn trace(owner: &str, phase: &str, details: &[(&str, JsValue)]) {
    let payload = Object::new();
    let _ = set_property(payload.as_ref(), "scope", &JsValue::from_str("header"));
    let _ = set_property(payload.as_ref(), "owner", &JsValue::from_str(owner));
    let _ = set_property(payload.as_ref(), "phase", &JsValue::from_str(phase));
    for (key, value) in details {
        let _ = set_property(payload.as_ref(), key, value);
    }
    let callback = property(&global(), "kgwUiTrace");
    if let Ok(callback) = callback.dyn_into::<Function>() {
        let _ = callback.call1(&global(), payload.as_ref());
    }
}
fn log_message(message: &str) {
    let callback = property(&window(), "kgwLog");
    if let Ok(callback) = callback.dyn_into::<Function>() {
        let _ = callback.call2(
            &window(),
            &JsValue::from_str("frontend:shell"),
            &JsValue::from_str(message),
        );
    } else {
        console_log(&format!("[KGW][header-live-metrics] {message}"));
    }
}

fn pad2(value: u32) -> String {
    format!("{value:02}")
}
fn date_parts(date: &Date) -> (String, String, String) {
    let year = date.get_full_year();
    let month = pad2(date.get_month() + 1);
    let day = pad2(date.get_date());
    let hours = pad2(date.get_hours());
    let minutes = pad2(date.get_minutes());
    let seconds = pad2(date.get_seconds());
    (
        format!("{hours}:{minutes}:{seconds}"),
        format!("{year}-{month}-{day}"),
        format!("{year}-{month}-{day} {hours}:{minutes}:{seconds}"),
    )
}
fn format_local_datetime(epoch_ms: f64) -> String {
    if epoch_ms == 0.0 || !epoch_ms.is_finite() {
        return "غير معروف".to_owned();
    }
    let date = Date::new(&JsValue::from_f64(epoch_ms));
    if !date.get_time().is_finite() {
        return "غير معروف".to_owned();
    }
    date_parts(&date).2
}
fn now_parts() -> (String, String, String) {
    date_parts(&Date::new_0())
}
fn first_positive_decimal(text: &str) -> f64 {
    let normalized = text.replace(',', "");
    let bytes = normalized.as_bytes();
    for start in 0..bytes.len() {
        if !bytes[start].is_ascii_digit() {
            continue;
        }
        let mut end = start + 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end + 1 < bytes.len() && bytes[end] == b'.' && bytes[end + 1].is_ascii_digit() {
            end += 2;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
        let value = normalized[start..end].parse::<f64>().unwrap_or(0.0);
        return if value.is_finite() && value > 0.0 {
            value
        } else {
            0.0
        };
    }
    0.0
}
fn metric_ids(kind: &str) -> (String, String) {
    let mut chars = kind.chars();
    let cap = match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    };
    (format!("kgwHeader{cap}"), format!("kgwHeader{cap}Box"))
}
fn format_currency(currency: &str, value: f64) -> String {
    let locales = Array::new();
    let options = Object::new();
    let _ = set_property(options.as_ref(), "style", &JsValue::from_str("currency"));
    let _ = set_property(options.as_ref(), "currency", &JsValue::from_str(currency));
    let _ = set_property(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(if value >= 1.0 { 4.0 } else { 8.0 }),
    );
    let formatter = NumberFormat::new(&locales, &options);
    formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(value))
        .map(|value| raw_string(&value))
        .unwrap_or_else(|_| format!("{currency} {value}"))
}
fn build_english_tooltip(label: &str, value: &str, updated_at: &str, source: &str) -> String {
    format!("{label}\nValue: {value}\nLast update: {updated_at}\nSource: {source}")
}

fn selected_currency() -> String {
    let select = query(&document(), "#shellCurrencySelect");
    let raw = if is_present(&select) {
        truthy_text(&property(&select, "value"))
    } else {
        String::new()
    };
    let normalized = if raw.is_empty() {
        "USD".to_owned()
    } else {
        raw
    };
    let normalized = normalized.trim().to_uppercase();
    if normalized.is_empty() {
        "USD".to_owned()
    } else {
        normalized
    }
}
fn numeric_price(prices: &JsValue, currency: &str) -> Option<f64> {
    let lower = currency.to_lowercase();
    let upper = lower.to_uppercase();
    let mut raw = property(prices, &lower);
    if !is_present(&raw) {
        raw = property(prices, &upper);
    }
    let value = numeric(&raw);
    (value.is_finite() && value > 0.0).then_some(value)
}
fn price_snapshot() -> JsValue {
    let snapshot = property(&global(), "kgwHeaderLastKaspaPricesR81C");
    if snapshot.is_object() {
        snapshot
    } else {
        Object::new().into()
    }
}
fn find_price_element() -> JsValue {
    for selector in [
        "#kgwHeaderPrice",
        "[data-kgw-metric='price']",
        "[data-metric='price']",
        "[data-header-metric='price']",
    ] {
        let element = query(&document(), selector);
        if is_present(&element) {
            return element;
        }
    }
    JsValue::UNDEFINED
}
fn render_selected_currency_price(prices: &JsValue, reason: &str) -> bool {
    let currency = selected_currency();
    let value = numeric_price(prices, &currency);
    let element = find_price_element();
    trace(
        R81C_OWNER,
        "r81c-price-selected-currency",
        &[
            ("currency", JsValue::from_str(&currency)),
            ("hasElement", JsValue::from_bool(is_present(&element))),
            ("hasValue", JsValue::from_bool(value.is_some())),
            ("reason", JsValue::from_str(reason)),
        ],
    );
    let Some(value) = value else {
        return false;
    };
    if !is_present(&element) {
        return false;
    }
    let formatted = format_currency(&currency, value);
    let _ = set_text(&element, &formatted);
    let _ = set_attr(&element, "data-kgw-selected-currency", &currency);
    let _ = set_attr(&element, "data-kgw-price-source", "kgw_get_kaspa_prices");
    true
}
fn preserve_existing_non_usd(current: &str) -> bool {
    let existing = current.trim();
    !existing.is_empty()
        && !existing.starts_with('$')
        && !existing.to_uppercase().starts_with("USD")
}
fn selected_currency_metric_value(metric_value: &str, current_text: &str) -> String {
    let currency = selected_currency();
    let prices = price_snapshot();
    let selected = if currency == "USD" {
        let value = first_positive_decimal(metric_value);
        (value > 0.0).then_some(value)
    } else {
        numeric_price(&prices, &currency)
    };
    trace(
        R85B_OWNER,
        "r85b-selected-currency-price-writer",
        &[
            ("currency", JsValue::from_str(&currency)),
            (
                "hasCachedSelectedValue",
                JsValue::from_bool(selected.is_some()),
            ),
            (
                "hasR81CRefresh",
                JsValue::from_bool(
                    property(&global(), "kgwHeaderRefreshSelectedCurrencyPriceR81C")
                        .dyn_ref::<Function>()
                        .is_some(),
                ),
            ),
        ],
    );
    if let Some(value) = selected {
        return format_currency(&currency, value);
    }
    if currency != "USD" {
        if let Ok(refresh) =
            property(&global(), "kgwHeaderRefreshSelectedCurrencyPriceR81C").dyn_into::<Function>()
        {
            let _ = refresh.call1(
                &global(),
                &JsValue::from_str("r85b-missing-selected-currency-cache"),
            );
        }
        if preserve_existing_non_usd(current_text) {
            return current_text.trim().to_owned();
        }
    }
    metric_value.to_owned()
}

fn own_metric_value_element(kind: &str, element: &JsValue) {
    if !is_present(element) {
        return;
    }
    remove_attr(element, "data-i18n");
    let _ = set_attr(element, "data-kgw-no-i18n", "true");
    let _ = set_dataset(element, "kgwLiveMetricValue", kind);
}
fn publish_kaspa_usd_price(value: &str) {
    let price = first_positive_decimal(value);
    if price <= 0.0 {
        return;
    }
    let price_value = JsValue::from_f64(price);
    let win = window();
    for key in [
        "__kgwKaspaUsdPrice",
        "__kgwHeaderPriceUsd",
        "__kgwLastKasPriceUsd",
        "__kaspaPriceUsd",
        "kaspaPriceUsd",
    ] {
        let _ = set_property(&win, key, &price_value);
    }
    let document_element = property(&document(), "documentElement");
    let _ = set_dataset(&document_element, "kgwKaspaUsdPrice", &price.to_string());

    let detail = Object::new();
    let _ = set_property(detail.as_ref(), "priceUsd", &price_value);
    let options = Object::new();
    let _ = set_property(options.as_ref(), "detail", detail.as_ref());
    if let Ok(constructor) = property(&global(), "CustomEvent").dyn_into::<Function>() {
        let args = Array::new();
        args.push(&JsValue::from_str("kgw:kaspa-price-updated"));
        args.push(options.as_ref());
        if let Ok(event) = Reflect::construct(&constructor, &args) {
            let _ = call1(&win, "dispatchEvent", event.as_ref());
        }
    }
}
fn set_metric(kind: &str, metric: &JsValue) {
    let (value_id, box_id) = metric_ids(kind);
    let value_element = get_by_id(&value_id);
    if !is_present(&value_element) {
        return;
    }
    own_metric_value_element(kind, &value_element);
    let status_raw = truthy_text(&property(metric, "status"));
    let status = if status_raw.is_empty() {
        "error".to_owned()
    } else {
        status_raw
    };
    let raw = truthy_text(&property(metric, "value"));
    let raw_value = if raw.is_empty() {
        if status == "error" {
            "Error".to_owned()
        } else {
            "Loading".to_owned()
        }
    } else {
        raw
    };
    let current_text = raw_string(&property(&value_element, "textContent"));
    let value = if kind == "price" && status == "ok" {
        selected_currency_metric_value(&raw_value, &current_text)
    } else {
        raw_value.clone()
    };
    let source_raw = truthy_text(&property(metric, "source"));
    let source = if source_raw.is_empty() {
        "Rust backend".to_owned()
    } else {
        source_raw
    };
    let updated_at = format_local_datetime(numeric(&property(metric, "updated_at_epoch_ms")));
    let error = truthy_text(&property(metric, "error"));
    let _ = set_text(&value_element, &value);
    if kind == "price" {
        let _ = set_dataset(&value_element, "kgwSelectedCurrency", &selected_currency());
        let _ = set_dataset(&value_element, "kgwPriceWriter", R85B_OWNER);
    }
    let _ = set_dataset(&value_element, "liveStatus", &status);
    if kind == "price" && status == "ok" {
        publish_kaspa_usd_price(&raw_value);
    }

    let box_element = get_by_id(&box_id);
    if is_present(&box_element) {
        let _ = set_dataset(&box_element, "liveStatus", &status);
        let title = if error.is_empty() {
            format!("آخر تحديث: {updated_at}\nالمصدر: {source}\nالقيمة: {value}\nالحالة: {status}")
        } else {
            format!(
                "آخر تحديث: {updated_at}\nالمصدر: {source}\nالقيمة: {value}\nالحالة: {status}\nالخطأ: {error}"
            )
        };
        let _ = set_property(&box_element, "title", &JsValue::from_str(&title));
    }
}
fn set_invoke_error(message: &str) {
    for kind in ["price", "hashrate", "difficulty"] {
        let metric = Object::new();
        let _ = set_property(metric.as_ref(), "value", &JsValue::from_str("Error"));
        let _ = set_property(metric.as_ref(), "status", &JsValue::from_str("error"));
        let _ = set_property(
            metric.as_ref(),
            "source",
            &JsValue::from_str("Tauri invoke"),
        );
        let _ = set_property(
            metric.as_ref(),
            "updated_at_epoch_ms",
            &JsValue::from_f64(Date::now()),
        );
        let _ = set_property(metric.as_ref(), "error", &JsValue::from_str(message));
        set_metric(kind, metric.as_ref());
    }
}
fn apply_snapshot(snapshot: &JsValue) {
    set_metric("price", &property(snapshot, "price"));
    set_metric("hashrate", &property(snapshot, "hashrate"));
    set_metric("difficulty", &property(snapshot, "difficulty"));
}

async fn resolve_invoke() -> Result<Function, JsValue> {
    let resolver = property(&global(), RESOLVE_INVOKE)
        .dyn_into::<Function>()
        .map_err(|_| js_error("Tauri invoke resolver is not available"))?;
    let result = resolver.call0(&global())?;
    let resolved = JsFuture::from(Promise::resolve(&result)).await?;
    resolved
        .dyn_into::<Function>()
        .map_err(|_| js_error("Tauri invoke API is not available"))
}
async fn invoke(command: &str) -> Result<JsValue, JsValue> {
    let callback = resolve_invoke().await?;
    let result = callback.call1(&JsValue::UNDEFINED, &JsValue::from_str(command))?;
    JsFuture::from(Promise::resolve(&result)).await
}
async fn refresh_header_metrics(force: bool) {
    let command = if force {
        "kgw_live_metrics_refresh_now"
    } else {
        "kgw_live_metrics_snapshot"
    };
    match invoke(command).await {
        Ok(snapshot) => apply_snapshot(&snapshot),
        Err(error) => {
            let message = raw_string(&property(&error, "message"));
            let message = if message.is_empty() {
                raw_string(&error)
            } else {
                message
            };
            set_invoke_error(&message);
            log_message(&format!("live metrics invoke failed :: {message}"));
        }
    }
}

fn set_clock() {
    let (time, date, _) = now_parts();
    let time_element = get_by_id("kgwHeaderClockTime");
    let date_element = get_by_id("kgwHeaderClockDate");
    if is_present(&time_element) {
        let _ = set_text(&time_element, &time);
    }
    if is_present(&date_element) {
        let _ = set_text(&date_element, &date);
    }
}
fn set_interval(callback: &JsValue, millis: f64) -> JsValue {
    call2(
        &window(),
        "setInterval",
        callback,
        &JsValue::from_f64(millis),
    )
    .unwrap_or(JsValue::UNDEFINED)
}
fn clear_interval(handle: &JsValue) {
    if is_present(handle) {
        let _ = call1(&window(), "clearInterval", handle);
    }
}
fn replace_clock_timer() {
    CLOCK_TIMER.with(|slot| {
        clear_interval(&slot.borrow());
        let callback = Closure::<dyn FnMut()>::new(set_clock);
        let handle = set_interval(callback.as_ref(), HEADER_CLOCK_TICK_MS);
        *slot.borrow_mut() = handle;
        callback.forget();
    });
}
fn replace_metrics_timer() {
    METRICS_TIMER.with(|slot| {
        clear_interval(&slot.borrow());
        let callback = Closure::<dyn FnMut()>::new(|| {
            spawn_local(async {
                refresh_header_metrics(false).await;
            });
        });
        let handle = set_interval(callback.as_ref(), LIVE_METRICS_REFRESH_MS);
        *slot.borrow_mut() = handle;
        callback.forget();
    });
}

fn ensure_selected_currency_price_owner(reason: &str) -> bool {
    let select = query(&document(), "#shellCurrencySelect");
    let price_element = find_price_element();
    let refresh = property(&global(), "kgwHeaderRefreshSelectedCurrencyPriceR81C");
    let has_refresh = refresh.dyn_ref::<Function>().is_some();
    trace(
        R83B_OWNER,
        "r83b-header-price-owner-ensure",
        &[
            ("reason", JsValue::from_str(reason)),
            ("hasSelect", JsValue::from_bool(is_present(&select))),
            (
                "hasPriceElement",
                JsValue::from_bool(is_present(&price_element)),
            ),
            ("hasR81CRefresh", JsValue::from_bool(has_refresh)),
        ],
    );
    if let Ok(refresh) = refresh.dyn_into::<Function>() {
        let _ = refresh.call1(&global(), &JsValue::from_str(reason));
        true
    } else {
        false
    }
}
fn bind_currency_select_fallback() -> bool {
    let select = query(&document(), "#shellCurrencySelect");
    if !is_present(&select) {
        trace(R83B_OWNER, "r83b-currency-select-missing", &[]);
        return false;
    }
    if raw_string(&property(
        &dataset(&select),
        "kgwHeaderPriceInitBindingOwnerR83B",
    )) == "1"
    {
        return true;
    }
    let _ = set_dataset(&select, "kgwHeaderPriceInitBindingOwnerR83B", "1");
    let callback = Closure::<dyn FnMut(JsValue)>::new(|_| {
        ensure_selected_currency_price_owner("shell-currency-change-r83b");
    });
    let _ = call2(
        &select,
        "addEventListener",
        &JsValue::from_str("change"),
        callback.as_ref(),
    );
    callback.forget();
    let options = property(&select, "options");
    let option_count = numeric(&property(&options, "length"));
    trace(
        R83B_OWNER,
        "r83b-currency-select-bound",
        &[
            ("optionCount", JsValue::from_f64(option_count)),
            (
                "selectedCurrency",
                JsValue::from_str(&raw_string(&property(&select, "value"))),
            ),
        ],
    );
    true
}
fn boot_selected_currency_price(reason: &str) {
    bind_currency_select_fallback();
    ensure_selected_currency_price_owner(reason);
}

fn ensure_header_clock_dom() -> Option<(JsValue, JsValue, JsValue)> {
    let root = get_by_id("kgwHeaderClock");
    if !is_present(&root) {
        return None;
    }
    let mut time_element = get_by_id("kgwHeaderClockTime");
    let mut date_element = get_by_id("kgwHeaderClockDate");
    if !is_present(&time_element) || !is_present(&date_element) {
        let _ = set_property(
            &root,
            "innerHTML",
            &JsValue::from_str(
                "<div id=\"kgwHeaderClockTime\"></div><div id=\"kgwHeaderClockDate\"></div>",
            ),
        );
        time_element = get_by_id("kgwHeaderClockTime");
        date_element = get_by_id("kgwHeaderClockDate");
    }
    let _ = set_property(&root, "hidden", &JsValue::FALSE);
    let style = property(&root, "style");
    let _ = set_property(&style, "display", &JsValue::from_str("flex"));
    let _ = set_property(&style, "visibility", &JsValue::from_str("visible"));
    let _ = set_property(&style, "opacity", &JsValue::from_str("1"));
    Some((root, time_element, date_element))
}
fn render_header_clock() {
    let Some((root, time_element, date_element)) = ensure_header_clock_dom() else {
        return;
    };
    let (time, date, stamp) = now_parts();
    let _ = set_text(&time_element, &time);
    let _ = set_text(&date_element, &date);
    let _ = set_property(
        &root,
        "title",
        &JsValue::from_str(&format!("Local time: {stamp}")),
    );
}
fn install_header_clock() {
    render_header_clock();
    HEADER_CLOCK_TIMER.with(|slot| {
        clear_interval(&slot.borrow());
        let callback = Closure::<dyn FnMut()>::new(render_header_clock);
        let handle = set_interval(callback.as_ref(), HEADER_CLOCK_TICK_MS);
        *slot.borrow_mut() = handle;
        callback.forget();
    });
}
fn find_metric_element(metric_key: &str) -> JsValue {
    let lower = metric_key.to_lowercase();
    let selectors = [
        format!("#kgwHeader{metric_key}Value"),
        format!("#kgwHeader{metric_key}"),
        format!("[data-kgw-metric-key=\"{lower}\"]"),
        format!("[data-kgw-metric=\"{lower}\"]"),
        format!("[data-kgw-header-metric=\"{lower}\"]"),
    ];
    for selector in selectors {
        let element = query(&document(), &selector);
        if is_present(&element) {
            return element;
        }
    }
    JsValue::UNDEFINED
}
fn metric_meta(element: &JsValue) -> (String, String, String) {
    let value = {
        let value = raw_string(&property(element, "textContent"))
            .trim()
            .to_owned();
        if value.is_empty() {
            "N/A".to_owned()
        } else {
            value
        }
    };
    let data = dataset(element);
    let source = [property(&data, "kgwSource"), property(&data, "source")]
        .into_iter()
        .map(|value| raw_string(&value).trim().to_owned())
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| "Unknown".to_owned());
    let updated_at = [
        property(&data, "kgwUpdatedAt"),
        property(&data, "updatedAt"),
        property(&data, "lastUpdated"),
    ]
    .into_iter()
    .map(|value| raw_string(&value).trim().to_owned())
    .find(|value| !value.is_empty())
    .unwrap_or_else(|| now_parts().2);
    (value, source, updated_at)
}
fn apply_english_tooltips() {
    for (key, label) in [
        ("Price", "Price"),
        ("Hashrate", "Hashrate"),
        ("Difficulty", "Difficulty"),
    ] {
        let element = find_metric_element(key);
        if !is_present(&element) {
            continue;
        }
        let (value, source, updated_at) = metric_meta(&element);
        let tooltip = build_english_tooltip(label, &value, &updated_at, &source);
        let _ = set_property(&element, "title", &JsValue::from_str(&tooltip));
    }
}
fn refresh_clock_and_english_tooltips() {
    render_header_clock();
    apply_english_tooltips();
}
fn install_english_clock_and_tooltips() {
    let dom = Closure::<dyn FnMut(JsValue)>::new(|_| {
        install_header_clock();
        apply_english_tooltips();
    });
    let _ = call2(
        &document(),
        "addEventListener",
        &JsValue::from_str("DOMContentLoaded"),
        dom.as_ref(),
    );
    dom.forget();

    let load = Closure::<dyn FnMut(JsValue)>::new(|_| {
        install_header_clock();
        apply_english_tooltips();
    });
    let _ = call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("load"),
        load.as_ref(),
    );
    load.forget();

    TOOLTIP_TIMER.with(|slot| {
        clear_interval(&slot.borrow());
        let callback = Closure::<dyn FnMut()>::new(refresh_clock_and_english_tooltips);
        let handle = set_interval(callback.as_ref(), LIVE_METRICS_REFRESH_MS);
        *slot.borrow_mut() = handle;
        callback.forget();
    });
}

fn normalize_prices(result: &JsValue) -> JsValue {
    if !result.is_object() {
        return Object::new().into();
    }
    let prices = property(result, "prices");
    if prices.is_object() {
        prices
    } else {
        result.clone()
    }
}
#[wasm_bindgen(js_name = headerLiveMetricsRefreshSelectedCurrencyPrice)]
pub async fn refresh_selected_currency_price(reason: JsValue) -> bool {
    let currency = selected_currency();
    let reason = {
        let value = truthy_text(&reason);
        if value.is_empty() {
            "manual".to_owned()
        } else {
            value
        }
    };
    trace(
        R81C_OWNER,
        "r81c-currency-price-refresh",
        &[
            ("currency", JsValue::from_str(&currency)),
            ("reason", JsValue::from_str(&reason)),
        ],
    );
    match invoke("kgw_get_kaspa_prices").await {
        Ok(result) => {
            let prices = normalize_prices(&result);
            let _ = set_property(&global(), "kgwHeaderLastKaspaPricesR81C", &prices);
            render_selected_currency_price(&prices, &reason)
        }
        Err(error) => {
            let message = {
                let text = raw_string(&property(&error, "message"));
                if text.is_empty() {
                    raw_string(&error)
                } else {
                    text
                }
            };
            trace(
                R81C_OWNER,
                "r81c-currency-price-refresh-error",
                &[
                    ("currency", JsValue::from_str(&currency)),
                    ("message", JsValue::from_str(&message)),
                ],
            );
            false
        }
    }
}
fn bind_currency_select_r81c() -> bool {
    let select = query(&document(), "#shellCurrencySelect");
    if !is_present(&select) {
        trace(R81C_OWNER, "r81c-currency-select-missing", &[]);
        return false;
    }
    if raw_string(&property(
        &dataset(&select),
        "kgwHeaderPriceCurrencyOwnerR81C",
    )) == "1"
    {
        return true;
    }
    let _ = set_dataset(&select, "kgwHeaderPriceCurrencyOwnerR81C", "1");
    let callback = Closure::<dyn FnMut(JsValue)>::new(|_| {
        spawn_local(async {
            refresh_selected_currency_price(JsValue::from_str("shell-currency-change")).await;
        });
    });
    let _ = call2(
        &select,
        "addEventListener",
        &JsValue::from_str("change"),
        callback.as_ref(),
    );
    callback.forget();
    let option_count = numeric(&property(&property(&select, "options"), "length"));
    let display = Array::new();
    for currency in [
        "usd", "sar", "eur", "gbp", "chf", "aud", "cad", "jpy", "krw", "rub", "cny", "try", "inr",
        "idr", "hkd", "sgd", "brl",
    ] {
        display.push(&JsValue::from_str(currency));
    }
    trace(
        R81C_OWNER,
        "r81c-currency-select-bound",
        &[
            ("optionCount", JsValue::from_f64(option_count)),
            ("selectedCurrency", JsValue::from_str(&selected_currency())),
            ("displayCurrencies", display.into()),
        ],
    );
    true
}
fn boot_r81c() {
    bind_currency_select_r81c();
    spawn_local(async {
        refresh_selected_currency_price(JsValue::from_str("boot")).await;
    });
}
fn install_r81c_owner() {
    let owner = property(&global(), R81C_OWNER);
    if truthy(&owner) {
        return;
    }
    let _ = set_property(&global(), R81C_OWNER, &JsValue::TRUE);
    if raw_string(&property(&document(), "readyState")) == "loading" {
        let callback = Closure::<dyn FnMut(JsValue)>::new(|_| boot_r81c());
        let options = Object::new();
        let _ = set_property(options.as_ref(), "once", &JsValue::TRUE);
        let _ = call3(
            &document(),
            "addEventListener",
            &JsValue::from_str("DOMContentLoaded"),
            callback.as_ref(),
            options.as_ref(),
        );
        callback.forget();
    } else {
        boot_r81c();
    }
}

#[wasm_bindgen(js_name = headerLiveMetricsInit)]
pub fn init_header_live_metrics() {
    let already_started = LIVE_STARTED.with(|started| {
        let mut started = started.borrow_mut();
        if *started {
            true
        } else {
            *started = true;
            false
        }
    });
    if already_started {
        return;
    }
    set_clock();
    spawn_local(async {
        refresh_header_metrics(true).await;
    });
    boot_selected_currency_price("initHeaderLiveMetrics-r83b");
    replace_clock_timer();
    replace_metrics_timer();
    log_message("header live metrics invoke module installed :: 15s refresh");
}

#[wasm_bindgen(js_name = headerLiveMetricsInstallModule)]
pub fn install_module() {
    if raw_string(&property(&document(), "readyState")) == "loading" {
        let callback = Closure::<dyn FnMut(JsValue)>::new(|_| {
            init_header_live_metrics();
            boot_selected_currency_price("DOMContentLoaded-r83b");
        });
        let options = Object::new();
        let _ = set_property(options.as_ref(), "once", &JsValue::TRUE);
        let _ = call3(
            &document(),
            "addEventListener",
            &JsValue::from_str("DOMContentLoaded"),
            callback.as_ref(),
            options.as_ref(),
        );
        callback.forget();
    } else {
        init_header_live_metrics();
        boot_selected_currency_price("module-loaded-r83b");
    }
    install_english_clock_and_tooltips();
    install_r81c_owner();
}

#[wasm_bindgen(js_name = headerLiveMetricsExtractUsdPrice)]
pub fn extract_usd_price(value: JsValue) -> f64 {
    first_positive_decimal(&raw_string(&value))
}
#[wasm_bindgen(js_name = headerLiveMetricsFormatLocalDateTime)]
pub fn exported_format_local_datetime(epoch_ms: f64) -> String {
    format_local_datetime(epoch_ms)
}
#[wasm_bindgen(js_name = headerLiveMetricsFormatEnglishTime)]
pub fn format_english_time(epoch_ms: f64) -> String {
    date_parts(&Date::new(&JsValue::from_f64(epoch_ms))).0
}
#[wasm_bindgen(js_name = headerLiveMetricsFormatEnglishDate)]
pub fn format_english_date(epoch_ms: f64) -> String {
    date_parts(&Date::new(&JsValue::from_f64(epoch_ms))).1
}
#[wasm_bindgen(js_name = headerLiveMetricsFormatEnglishStamp)]
pub fn format_english_stamp(epoch_ms: f64) -> String {
    date_parts(&Date::new(&JsValue::from_f64(epoch_ms))).2
}
#[wasm_bindgen(js_name = headerLiveMetricsMetricIds)]
pub fn exported_metric_ids(kind: String) -> JsValue {
    let (value_id, box_id) = metric_ids(&kind);
    let output = Object::new();
    let _ = set_property(output.as_ref(), "valueId", &JsValue::from_str(&value_id));
    let _ = set_property(output.as_ref(), "boxId", &JsValue::from_str(&box_id));
    output.into()
}
#[wasm_bindgen(js_name = headerLiveMetricsNumericPrice)]
pub fn exported_numeric_price(prices: JsValue, currency: String) -> JsValue {
    numeric_price(&prices, &currency)
        .map(JsValue::from_f64)
        .unwrap_or(JsValue::NULL)
}
#[wasm_bindgen(js_name = headerLiveMetricsFormatCurrency)]
pub fn exported_format_currency(currency: String, value: f64) -> String {
    format_currency(&currency, value)
}
#[wasm_bindgen(js_name = headerLiveMetricsSelectedCurrencyValue)]
pub fn exported_selected_currency_value(metric_value: String, current_text: String) -> String {
    selected_currency_metric_value(&metric_value, &current_text)
}
#[wasm_bindgen(js_name = headerLiveMetricsBuildEnglishTooltip)]
pub fn exported_build_english_tooltip(
    label: String,
    value: String,
    updated_at: String,
    source: String,
) -> String {
    build_english_tooltip(&label, &value, &updated_at, &source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_price_parser_matches_legacy_shape() {
        assert_eq!(first_positive_decimal("$0.12345678 USD"), 0.12345678);
        assert_eq!(first_positive_decimal("KAS 1,234.50 USD"), 1234.5);
        assert_eq!(first_positive_decimal("N/A"), 0.0);
        assert_eq!(first_positive_decimal("0 USD"), 0.0);
        assert_eq!(first_positive_decimal("12. USD"), 12.0);
    }

    #[test]
    fn metric_ids_preserve_shell_contract() {
        assert_eq!(
            metric_ids("price"),
            ("kgwHeaderPrice".to_owned(), "kgwHeaderPriceBox".to_owned())
        );
        assert_eq!(
            metric_ids("hashrate"),
            (
                "kgwHeaderHashrate".to_owned(),
                "kgwHeaderHashrateBox".to_owned()
            )
        );
    }

    #[test]
    fn non_usd_fallback_preserves_existing_selected_currency_text() {
        assert!(preserve_existing_non_usd("SAR 0.44"));
        assert!(!preserve_existing_non_usd("$0.12"));
        assert!(!preserve_existing_non_usd("USD 0.12"));
        assert!(!preserve_existing_non_usd("  "));
    }

    #[test]
    fn english_tooltip_shape_is_stable() {
        assert_eq!(
            build_english_tooltip("Price", "1.23", "2026-09-26 12:34:56", "Rust backend"),
            "Price\nValue: 1.23\nLast update: 2026-09-26 12:34:56\nSource: Rust backend"
        );
    }
}
