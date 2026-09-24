use js_sys::{Array, Date, Intl::NumberFormat, JsString, Object, Reflect};
use wasm_bindgen::prelude::*;

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
}
