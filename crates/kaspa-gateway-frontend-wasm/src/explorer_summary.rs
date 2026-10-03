use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

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

fn first_nullish(target: &JsValue, names: &[&str], fallback: JsValue) -> JsValue {
    for name in names {
        let value = property(target, name);
        if present(&value) {
            return value;
        }
    }
    fallback
}
fn first_truthy(target: &JsValue, names: &[&str], fallback: &str) -> JsValue {
    for name in names {
        let value = property(target, name);
        if crate::js_boolean(&value) {
            return value;
        }
    }
    JsValue::from_str(fallback)
}

fn number_or_zero(value: &JsValue) -> f64 {
    let number = crate::js_number(value);
    if number.is_nan() || number == 0.0 {
        0.0
    } else {
        number
    }
}

fn current_usd_price() -> Result<f64, JsValue> {
    crate::parse_header_usd_price()
}

fn usd_for_kas_with_price(value_kas: f64, price_usd: f64) -> f64 {
    let kas = value_kas.abs();
    if !kas.is_finite() || !price_usd.is_finite() || price_usd <= 0.0 {
        0.0
    } else {
        kas * price_usd
    }
}
fn summary_usd_with_price(
    explicit_usd: f64,
    incoming_kas: f64,
    outgoing_kas: f64,
    net_kas: f64,
    price_usd: f64,
) -> f64 {
    if explicit_usd.is_finite() && explicit_usd > 0.0 {
        return explicit_usd;
    }
    let gross_kas = incoming_kas.abs() + outgoing_kas.abs();
    let kas_for_usd = if gross_kas > 0.0 {
        gross_kas
    } else {
        net_kas.abs()
    };
    if price_usd > 0.0 {
        kas_for_usd * price_usd
    } else {
        0.0
    }
}

fn day_prefix_text(value: &str) -> String {
    value.chars().take(10).collect()
}

fn day_text(value: &JsValue) -> String {
    day_prefix_text(&crate::js_string_owned(value))
}

fn normalize_day_summaries(result: &JsValue, price_usd: f64) -> JsValue {
    let items = if Array::is_array(result) {
        Array::from(result)
    } else {
        let days = property(result, "days");
        if Array::is_array(&days) {
            Array::from(&days)
        } else {
            let summaries = property(result, "summaries");
            if Array::is_array(&summaries) {
                Array::from(&summaries)
            } else {
                Array::new()
            }
        }
    };

    let output = Array::new();
    for item in items.iter() {
        let incoming = number_or_zero(&first_nullish(
            &item,
            &["incoming_kas", "incomingKas"],
            JsValue::from_f64(0.0),
        ));
        let outgoing = number_or_zero(&first_nullish(
            &item,
            &["outgoing_kas", "outgoingKas"],
            JsValue::from_f64(0.0),
        ));
        let net = number_or_zero(&first_nullish(
            &item,
            &["net_kas", "netKas"],
            JsValue::from_f64(incoming - outgoing),
        ));
        let explicit = crate::js_number(&first_nullish(
            &item,
            &["value_usd", "valueUsd", "usd_value", "usdValue", "value"],
            JsValue::UNDEFINED,
        ));
        let count = number_or_zero(&first_nullish(
            &item,
            &["count", "tx_count", "transactions_count"],
            JsValue::from_f64(0.0),
        ));
        let day = day_text(&first_truthy(&item, &["day", "date"], ""));
        if day.is_empty() {
            continue;
        }

        let normalized = Object::new();
        set(normalized.as_ref(), "__kgwDaySummary", &JsValue::TRUE);
        set(normalized.as_ref(), "day", &JsValue::from_str(&day));
        set(normalized.as_ref(), "count", &JsValue::from_f64(count));
        set(
            normalized.as_ref(),
            "incoming_kas",
            &JsValue::from_f64(incoming),
        );
        set(
            normalized.as_ref(),
            "outgoing_kas",
            &JsValue::from_f64(outgoing),
        );
        set(normalized.as_ref(), "net_kas", &JsValue::from_f64(net));
        set(
            normalized.as_ref(),
            "value_usd",
            &JsValue::from_f64(summary_usd_with_price(
                explicit, incoming, outgoing, net, price_usd,
            )),
        );
        output.push(normalized.as_ref());
    }
    output.into()
}

#[wasm_bindgen(js_name = explorerSummaryCurrentUsdPrice)]
pub fn explorer_summary_current_usd_price() -> Result<f64, JsValue> {
    current_usd_price()
}

#[wasm_bindgen(js_name = explorerSummaryUsdForKas)]
pub fn explorer_summary_usd_for_kas(value: JsValue) -> Result<f64, JsValue> {
    let price = current_usd_price()?;
    Ok(usd_for_kas_with_price(crate::js_number(&value), price))
}

#[wasm_bindgen(js_name = explorerNormalizeDaySummaries)]
pub fn explorer_normalize_day_summaries(result: JsValue) -> Result<JsValue, JsValue> {
    let price = current_usd_price()?;
    Ok(normalize_day_summaries(&result, price))
}
#[wasm_bindgen(js_name = explorerSummaryUsdForSummary)]
pub fn explorer_summary_usd_for_summary(summary: JsValue) -> Result<f64, JsValue> {
    let explicit = crate::js_number(&first_nullish(
        &summary,
        &["value_usd", "valueUsd", "usd_value", "usdValue"],
        JsValue::from_f64(0.0),
    ));
    if explicit.is_finite() && explicit > 0.0 {
        return Ok(explicit);
    }
    let incoming = number_or_zero(&first_nullish(
        &summary,
        &["incoming_kas", "incomingKas"],
        JsValue::from_f64(0.0),
    ));
    let outgoing = number_or_zero(&first_nullish(
        &summary,
        &["outgoing_kas", "outgoingKas"],
        JsValue::from_f64(0.0),
    ));
    let net = number_or_zero(&first_nullish(
        &summary,
        &["net_kas", "netKas"],
        JsValue::from_f64(0.0),
    ));
    let gross = incoming.abs() + outgoing.abs();
    let kas_for_usd = if gross > 0.0 { gross } else { net.abs() };
    Ok(usd_for_kas_with_price(kas_for_usd, current_usd_price()?))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usd_for_kas_matches_finite_positive_contract() {
        assert_eq!(usd_for_kas_with_price(-2.5, 0.2), 0.5);
        assert_eq!(usd_for_kas_with_price(2.5, 0.0), 0.0);
        assert_eq!(usd_for_kas_with_price(f64::INFINITY, 0.2), 0.0);
    }

    #[test]
    fn summary_usd_prefers_explicit_then_gross_then_net() {
        assert_eq!(summary_usd_with_price(7.0, 1.0, 2.0, -1.0, 10.0), 7.0);
        assert_eq!(summary_usd_with_price(0.0, 1.0, 2.0, -1.0, 10.0), 30.0);
        assert_eq!(summary_usd_with_price(0.0, 0.0, 0.0, -2.0, 10.0), 20.0);
    }

    #[test]
    fn day_prefix_is_ten_characters() {
        assert_eq!(day_prefix_text("2026-09-28T12:34:56"), "2026-09-28");
    }
}
