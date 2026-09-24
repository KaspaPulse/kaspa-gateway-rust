use js_sys::{Array, JsString};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = String)]
    fn js_string(value: &JsValue) -> JsString;

    #[wasm_bindgen(js_name = Number)]
    fn js_number(value: &JsValue) -> f64;
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
