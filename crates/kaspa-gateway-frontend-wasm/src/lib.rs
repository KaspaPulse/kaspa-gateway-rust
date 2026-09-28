mod analysis_binding;
mod analysis_calendar;
mod analysis_view;
mod explorer_addresses;
mod explorer_calendar;
mod explorer_controls;
mod explorer_export;
mod explorer_filters;
mod explorer_force_ui;
mod explorer_live_core;
mod explorer_results;
mod explorer_summary;
mod header_live_metrics;
mod log_tab;
mod node_frontend_helpers;
mod node_path_helpers;
mod node_settings_owner;
mod node_start_trace;
mod node_tab;
mod settings_addresses;
mod settings_contract;
mod settings_database;
mod settings_diagnostics;
mod settings_layout;
mod settings_paths;
mod settings_persistence;
mod settings_profiles;
mod settings_runtime;
mod settings_schema;
mod settings_state;
mod settings_ui;
mod shell_aux;
mod shell_display;
mod shell_i18n;
mod shell_logger;
mod shell_runtime;
mod top_addresses;

use js_sys::{Array, Date, Function, Intl::NumberFormat, JsString, Object, Reflect};
use wasm_bindgen::{JsCast, prelude::*};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = String)]
    fn js_string(value: &JsValue) -> JsString;

    #[wasm_bindgen(js_name = Number)]
    fn js_number(value: &JsValue) -> f64;

    #[wasm_bindgen(js_name = Boolean)]
    fn js_boolean(value: &JsValue) -> bool;
}

fn nullish_to_empty(value: &JsValue) -> JsValue {
    if value.is_null() || value.is_undefined() {
        JsValue::from_str("")
    } else {
        value.clone()
    }
}

fn js_string_owned(value: &JsValue) -> String {
    String::from(js_string(&nullish_to_empty(value)))
}

fn to_english_digits_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '٠' | '۰' => '0',
            '١' | '۱' => '1',
            '٢' | '۲' => '2',
            '٣' | '۳' => '3',
            '٤' | '۴' => '4',
            '٥' | '۵' => '5',
            '٦' | '۶' => '6',
            '٧' | '۷' => '7',
            '٨' | '۸' => '8',
            '٩' | '۹' => '9',
            other => other,
        })
        .collect()
}

fn html_escape_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#039;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[wasm_bindgen(js_name = toEnglishDigits)]
pub fn to_english_digits(value: JsValue) -> String {
    to_english_digits_text(&js_string_owned(&value))
}

#[wasm_bindgen(js_name = pick)]
pub fn pick(values: Array) -> JsValue {
    for index in 0..values.length() {
        let value = values.get(index);
        let is_empty_primitive_string = value.as_string().is_some_and(|text| text.is_empty());
        if !value.is_null() && !value.is_undefined() && !is_empty_primitive_string {
            return value;
        }
    }
    JsValue::NULL
}

#[wasm_bindgen(js_name = toNumber)]
pub fn to_number(value: JsValue, fallback: f64) -> f64 {
    let number = js_number(&value);
    if number.is_finite() { number } else { fallback }
}

#[wasm_bindgen(js_name = kgwClean2SafeText)]
pub fn kgw_clean2_safe_text(value: JsValue) -> String {
    html_escape_text(&js_string_owned(&value))
}

fn normalize_date_input_text(value: &str) -> String {
    let clean: String = to_english_digits_text(value)
        .chars()
        .filter(|ch| ch.is_ascii_digit() || *ch == '-')
        .collect();
    let compact: String = clean.chars().filter(|ch| *ch != '-').collect();

    if compact.len() == 8 && compact.bytes().all(|byte| byte.is_ascii_digit()) {
        return format!("{}-{}-{}", &compact[0..4], &compact[4..6], &compact[6..8]);
    }

    clean.chars().take(10).collect()
}

fn date_pattern_is_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn date_to_epoch_seconds(text: &str) -> JsValue {
    let date = Date::new(&JsValue::from_str(text));
    let millis = date.get_time();
    if millis.is_finite() {
        JsValue::from_f64((millis / 1000.0).floor())
    } else {
        JsValue::NULL
    }
}

fn first_truthy_property(row: &JsValue, names: &[&str]) -> JsValue {
    if row.is_null() || row.is_undefined() {
        return JsValue::UNDEFINED;
    }

    for name in names {
        let key = JsValue::from_str(name);
        let value = Reflect::get(row, &key).unwrap_or(JsValue::UNDEFINED);
        if js_boolean(&value) {
            return value;
        }
    }
    JsValue::UNDEFINED
}

fn first_ascii_date(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    chars.windows(10).find_map(|window| {
        let valid = window[4] == '-'
            && window[7] == '-'
            && window
                .iter()
                .enumerate()
                .all(|(index, ch)| matches!(index, 4 | 7) || ch.is_ascii_digit());
        valid.then(|| window.iter().collect())
    })
}

fn js_number_or_zero(value: JsValue) -> f64 {
    let source = if js_boolean(&value) {
        value
    } else {
        JsValue::from_f64(0.0)
    };
    js_number(&source)
}

fn format_three_fraction_digits(number: f64) -> String {
    let locales = Array::new();
    let options = Object::new();
    let _ = Reflect::set(
        options.as_ref(),
        &JsValue::from_str("minimumFractionDigits"),
        &JsValue::from_f64(3.0),
    );
    let _ = Reflect::set(
        options.as_ref(),
        &JsValue::from_str("maximumFractionDigits"),
        &JsValue::from_f64(3.0),
    );
    let formatter = NumberFormat::new(&locales, &options);
    let value = formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(number))
        .unwrap_or(JsValue::UNDEFINED);
    js_string_owned(&value)
}

#[wasm_bindgen(js_name = normalizeDateInputValue)]
pub fn normalize_date_input_value(value: JsValue) -> String {
    normalize_date_input_text(&js_string_owned(&value))
}

#[wasm_bindgen(js_name = parseDateSeconds)]
pub fn parse_date_seconds(value: JsValue, end_of_day: bool) -> JsValue {
    let normalized = normalize_date_input_value(value);
    if !date_pattern_is_valid(&normalized) {
        return JsValue::NULL;
    }
    let suffix = if end_of_day { "T23:59:59" } else { "T00:00:00" };
    date_to_epoch_seconds(&format!("{normalized}{suffix}"))
}

fn utc_day_to_epoch_seconds(value: JsValue, end_of_day: bool) -> JsValue {
    let day = String::from(js_string(&value));
    let result = date_to_epoch_seconds(&format!("{day}T00:00:00Z"));
    let Some(base) = result.as_f64() else {
        return JsValue::NULL;
    };
    JsValue::from_f64(if end_of_day { base + 86_399.0 } else { base })
}

#[wasm_bindgen(js_name = kgwDayToEpochSeconds)]
pub fn kgw_day_to_epoch_seconds(value: JsValue, end_of_day: bool) -> JsValue {
    utc_day_to_epoch_seconds(value, end_of_day)
}

#[wasm_bindgen(js_name = kgwTxDayToEpochSeconds)]
pub fn kgw_tx_day_to_epoch_seconds(value: JsValue, end_of_day: bool) -> JsValue {
    utc_day_to_epoch_seconds(value, end_of_day)
}

#[wasm_bindgen(js_name = kgwClean2DayToSeconds)]
pub fn kgw_clean2_day_to_seconds(value: JsValue, end_of_day: bool) -> JsValue {
    utc_day_to_epoch_seconds(value, end_of_day)
}

#[wasm_bindgen(js_name = kgwTransactionDateKey)]
pub fn kgw_transaction_date_key(row: JsValue) -> String {
    let raw = first_truthy_property(&row, &["date", "day", "datetime", "timestamp", "time"]);
    let source = if js_boolean(&raw) {
        raw
    } else {
        JsValue::from_str("")
    };
    first_ascii_date(&String::from(js_string(&source))).unwrap_or_else(|| "Unknown Date".to_owned())
}

#[wasm_bindgen(js_name = formatKas)]
pub fn format_kas(value: JsValue) -> String {
    let number = js_number_or_zero(value);
    if !number.is_finite() || number == 0.0 {
        String::new()
    } else {
        format_three_fraction_digits(number)
    }
}

#[wasm_bindgen(js_name = formatUsd)]
pub fn format_usd(value: JsValue) -> String {
    format_three_fraction_digits(js_number_or_zero(value))
}

#[wasm_bindgen(js_name = kgwSummaryFormatKas)]
pub fn kgw_summary_format_kas(value: JsValue) -> String {
    format_kas(value)
}

#[wasm_bindgen(js_name = kgwSummaryFormatUsd)]
pub fn kgw_summary_format_usd(value: JsValue) -> String {
    let number = js_number_or_zero(value);
    if !number.is_finite() || number <= 0.0 {
        String::new()
    } else {
        format_three_fraction_digits(number)
    }
}

#[wasm_bindgen(js_name = kgwClean2Kas)]
pub fn kgw_clean2_kas(value: JsValue) -> String {
    format_kas(value)
}

#[wasm_bindgen(js_name = kgwClean2Usd)]
pub fn kgw_clean2_usd(value: JsValue) -> String {
    kgw_summary_format_usd(value)
}

fn normalize_status_text(value: &str) -> String {
    let mut normalized = String::new();
    let mut pending_space = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !normalized.is_empty() {
                pending_space = true;
            }
            continue;
        }
        if pending_space {
            normalized.push(' ');
            pending_space = false;
        }
        normalized.push(ch);
    }
    normalized
}

fn status_tone_text(value: &str) -> &'static str {
    match normalize_status_text(value).as_str() {
        "running" | "active" | "enabled" | "healthy" | "ok" | "success" | "hashing"
        | "synchronized" | "synced" => "positive",
        "stopped" | "disabled" | "error" | "failed" | "offline" | "unavailable"
        | "disconnected" | "not connected" | "not ready" => "negative",
        "ready" | "available" | "connected" | "verified" | "validated" | "complete"
        | "completed" => "ready",
        "warning"
        | "warn"
        | "degraded"
        | "partial"
        | "starting"
        | "stopping"
        | "syncing"
        | "not synchronized"
        | "not synced"
        | "pending"
        | "loading"
        | "busy"
        | "fetching"
        | "validating"
        | "reconciling"
        | "waiting for work"
        | "no recent hashing reported" => "warning",
        _ => "neutral",
    }
}

fn status_tone_from_js(value: &JsValue) -> &'static str {
    let source = if js_boolean(value) {
        value.clone()
    } else {
        JsValue::from_str("")
    };
    status_tone_text(&String::from(js_string(&source)))
}

fn get_property(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    Reflect::get(target, &JsValue::from_str(name))
}

fn set_property(target: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(name), value).map(|_| ())
}

fn method(target: &JsValue, name: &str) -> Result<Function, JsValue> {
    get_property(target, name)?.dyn_into::<Function>()
}

fn call_method0(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    method(target, name)?.call0(target)
}

fn call_method1(target: &JsValue, name: &str, argument: &JsValue) -> Result<JsValue, JsValue> {
    method(target, name)?.call1(target, argument)
}

pub(crate) fn apply_status_tone_js(element: JsValue, state: JsValue) -> Result<(), JsValue> {
    if !js_boolean(&element) {
        return Ok(());
    }

    let dataset = get_property(&element, "dataset")?;
    let key = get_property(&dataset, "i18n")?;
    if js_boolean(&key) {
        let global = js_sys::global();
        let translate = get_property(&global, "kgwT")?;
        if let Some(function) = translate.dyn_ref::<Function>() {
            let translated = function.call1(&global, &key)?;
            let current_text = get_property(&element, "textContent")?;
            if current_text != translated {
                let _ = call_method1(&element, "removeAttribute", &JsValue::from_str("data-i18n"))?;
            }
        }
    }

    set_property(
        &dataset,
        "statusTone",
        &JsValue::from_str(status_tone_from_js(&state)),
    )
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct StatusSummaryPart {
    label: String,
    value: String,
    tone: &'static str,
}

fn status_summary_parts(text: &str) -> Vec<StatusSummaryPart> {
    text.split(" | ")
        .map(|part| {
            let split = part.find(": ");
            let (label, value) = if let Some(index) = split {
                (part[..index + 2].to_owned(), part[index + 2..].to_owned())
            } else {
                (String::new(), part.to_owned())
            };
            let tone = if label == "RPC error: " {
                status_tone_text("error")
            } else {
                status_tone_text(&value)
            };
            StatusSummaryPart { label, value, tone }
        })
        .collect()
}

#[wasm_bindgen(js_name = statusTone)]
pub fn status_tone(state: JsValue) -> String {
    status_tone_from_js(&state).to_owned()
}

#[wasm_bindgen(js_name = applyStatusTone)]
pub fn apply_status_tone(element: JsValue, state: JsValue) -> Result<(), JsValue> {
    apply_status_tone_js(element, state)
}

#[wasm_bindgen(js_name = renderStatusSummary)]
pub fn render_status_summary(element: JsValue, text: JsValue) -> Result<(), JsValue> {
    if !js_boolean(&element) {
        return Ok(());
    }

    let dataset = get_property(&element, "dataset")?;
    if let (Some(current), Some(requested)) = (
        get_property(&dataset, "statusSummary")?.as_string(),
        text.as_string(),
    ) && current == requested
    {
        let existing = call_method1(
            &element,
            "querySelector",
            &JsValue::from_str(".kgw-status-value"),
        )?;
        if js_boolean(&existing) {
            return Ok(());
        }
    }

    let rendered_text = String::from(js_string(&text));
    let global = js_sys::global();
    let document = get_property(&global, "document")?;
    let content = call_method0(&document, "createDocumentFragment")?;

    for (index, part) in status_summary_parts(&rendered_text).iter().enumerate() {
        if index > 0 {
            let separator = call_method1(&document, "createTextNode", &JsValue::from_str(" | "))?;
            let _ = call_method1(&content, "appendChild", &separator)?;
        }

        let item = call_method1(&document, "createElement", &JsValue::from_str("span"))?;
        set_property(&item, "className", &JsValue::from_str("kgw-status-item"))?;

        let label = call_method1(&document, "createTextNode", &JsValue::from_str(&part.label))?;
        let _ = call_method1(&item, "appendChild", &label)?;

        let badge = call_method1(&document, "createElement", &JsValue::from_str("span"))?;
        set_property(&badge, "className", &JsValue::from_str("kgw-status-value"))?;
        set_property(&badge, "textContent", &JsValue::from_str(&part.value))?;
        apply_status_tone_js(
            badge.clone(),
            JsValue::from_str(if part.label == "RPC error: " {
                "error"
            } else {
                &part.value
            }),
        )?;
        let _ = call_method1(&item, "appendChild", &badge)?;
        let _ = call_method1(&content, "appendChild", &item)?;
    }

    let _ = call_method1(&element, "replaceChildren", &content)?;
    set_property(&dataset, "statusSummary", &text)
}

fn optional_property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn positive_price_candidate(value: &JsValue) -> Option<f64> {
    let source = if js_boolean(value) {
        value.clone()
    } else {
        JsValue::from_str("")
    };
    let normalized = String::from(js_string(&source)).replace(',', "");
    let number = js_number(&JsValue::from_str(&normalized));
    (number.is_finite() && number > 0.0).then_some(number)
}

fn first_decimal_text(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
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
        return Some(value[start..end].to_owned());
    }
    None
}

fn first_usd_decimal_text(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
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

        let mut tail = end;
        while tail < value.len() {
            let Some(ch) = value[tail..].chars().next() else {
                break;
            };
            if !ch.is_whitespace() {
                break;
            }
            tail += ch.len_utf8();
        }
        if tail + 3 <= value.len()
            && value
                .get(tail..tail + 3)
                .is_some_and(|suffix| suffix.eq_ignore_ascii_case("USD"))
        {
            return Some(value[start..end].to_owned());
        }
    }
    None
}

fn decimal_text_to_positive_number(value: Option<String>) -> Option<f64> {
    let number = value?.parse::<f64>().ok()?;
    (number.is_finite() && number > 0.0).then_some(number)
}

#[wasm_bindgen(js_name = parseHeaderUsdPrice)]
pub fn parse_header_usd_price() -> Result<f64, JsValue> {
    let global = js_sys::global();
    let window = optional_property(&global, "window");
    let document = get_property(&global, "document")?;
    let document_element = optional_property(&document, "documentElement");
    let dataset = optional_property(&document_element, "dataset");

    let direct = [
        optional_property(&window, "__kgwKaspaUsdPrice"),
        optional_property(&window, "__kgwHeaderPriceUsd"),
        optional_property(&window, "__kgwLastKasPriceUsd"),
        optional_property(&window, "__kaspaPriceUsd"),
        optional_property(&window, "kaspaPriceUsd"),
        optional_property(&dataset, "kgwKaspaUsdPrice"),
    ];
    for item in &direct {
        if let Some(number) = positive_price_candidate(item) {
            return Ok(number);
        }
    }

    let header_price = call_method1(
        &document,
        "getElementById",
        &JsValue::from_str("kgwHeaderPrice"),
    )?;
    let header_text_value = optional_property(&header_price, "textContent");
    let header_source = if js_boolean(&header_text_value) {
        header_text_value
    } else {
        JsValue::from_str("")
    };
    let header_text = String::from(js_string(&header_source)).replace(',', "");
    if let Some(number) = decimal_text_to_positive_number(first_decimal_text(&header_text)) {
        return Ok(number);
    }

    let body = optional_property(&document, "body");
    let body_text_value = optional_property(&body, "innerText");
    let body_source = if js_boolean(&body_text_value) {
        body_text_value
    } else {
        JsValue::from_str("")
    };
    let body_text = String::from(js_string(&body_source)).replace(',', "");
    Ok(decimal_text_to_positive_number(first_usd_decimal_text(&body_text)).unwrap_or(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_and_persian_digits_are_normalized() {
        assert_eq!(to_english_digits_text("A١٢٣-۴۵۶-٧۸۹٠"), "A123-456-7890");
    }

    #[test]
    fn safe_text_preserves_escape_order() {
        assert_eq!(
            html_escape_text("A&B <tag> \"quote\" 'single'"),
            "A&amp;B &lt;tag&gt; &quot;quote&quot; &#039;single&#039;"
        );
    }

    #[test]
    fn plain_text_is_stable() {
        assert_eq!(to_english_digits_text("abc-123"), "abc-123");
        assert_eq!(html_escape_text("abc-123"), "abc-123");
    }

    #[test]
    fn explorer_header_decimal_parsing_preserves_legacy_shape() {
        assert_eq!(
            first_decimal_text("KAS $1234.56 USD"),
            Some("1234.56".to_owned())
        );
        assert_eq!(first_decimal_text("-1 USD"), Some("1".to_owned()));
        assert_eq!(first_decimal_text("prefix .5 USD"), Some("5".to_owned()));
        assert_eq!(first_decimal_text("no price"), None);
        assert_eq!(decimal_text_to_positive_number(Some("0".to_owned())), None);
        assert_eq!(
            decimal_text_to_positive_number(Some("0.25".to_owned())),
            Some(0.25)
        );
    }

    #[test]
    fn explorer_header_body_usd_parsing_matches_contract() {
        assert_eq!(
            first_usd_decimal_text("Kaspa Price 0.55 USD"),
            Some("0.55".to_owned())
        );
        assert_eq!(
            first_usd_decimal_text("Market 0.66usd now"),
            Some("0.66".to_owned())
        );
        assert_eq!(
            first_usd_decimal_text("1 EUR then 2.5 USD"),
            Some("2.5".to_owned())
        );
        assert_eq!(first_usd_decimal_text("Price 3.0 EUR"), None);
    }

    #[test]
    fn status_tone_normalization_matches_frontend_contract() {
        for (input, expected) in [
            ("running", "positive"),
            (" Running ", "positive"),
            ("NOT_READY", "negative"),
            ("not-ready", "negative"),
            ("completed", "ready"),
            ("Validating", "warning"),
            ("waiting   for-work", "warning"),
            ("something else", "neutral"),
        ] {
            assert_eq!(status_tone_text(input), expected, "{input}");
        }
    }

    #[test]
    fn status_summary_parts_match_frontend_contract() {
        assert_eq!(
            status_summary_parts("Node: Running | RPC: Ready | Mining: hashing"),
            vec![
                StatusSummaryPart {
                    label: "Node: ".to_owned(),
                    value: "Running".to_owned(),
                    tone: "positive",
                },
                StatusSummaryPart {
                    label: "RPC: ".to_owned(),
                    value: "Ready".to_owned(),
                    tone: "ready",
                },
                StatusSummaryPart {
                    label: "Mining: ".to_owned(),
                    value: "hashing".to_owned(),
                    tone: "positive",
                },
            ]
        );
        assert_eq!(
            status_summary_parts("RPC error: timeout | Sync: unknown"),
            vec![
                StatusSummaryPart {
                    label: "RPC error: ".to_owned(),
                    value: "timeout".to_owned(),
                    tone: "negative",
                },
                StatusSummaryPart {
                    label: "Sync: ".to_owned(),
                    value: "unknown".to_owned(),
                    tone: "neutral",
                },
            ]
        );
        assert_eq!(
            status_summary_parts("Ready"),
            vec![StatusSummaryPart {
                label: String::new(),
                value: "Ready".to_owned(),
                tone: "ready",
            }]
        );
    }
}
