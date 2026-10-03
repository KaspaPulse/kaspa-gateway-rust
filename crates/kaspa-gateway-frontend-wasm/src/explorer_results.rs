use js_sys::{Array, Date, Function, Object, Reflect};
use std::cmp::Ordering;
use std::collections::HashSet;
use wasm_bindgen::{JsCast, prelude::*};

const SOMPI_PER_KAS: f64 = 100_000_000.0;

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

fn pick(values: &[JsValue]) -> JsValue {
    for value in values {
        let empty_primitive = value.as_string().is_some_and(|text| text.is_empty());
        if present(value) && !empty_primitive {
            return value.clone();
        }
    }
    JsValue::NULL
}

fn pick_properties(target: &JsValue, names: &[&str]) -> JsValue {
    let values = names
        .iter()
        .map(|name| property(target, name))
        .collect::<Vec<_>>();
    pick(&values)
}

fn truthy_pick_string(target: &JsValue, names: &[&str], fallback: &str) -> String {
    let mut values = names
        .iter()
        .map(|name| property(target, name))
        .collect::<Vec<_>>();
    values.push(JsValue::from_str(fallback));
    let value = pick(&values);
    if crate::js_boolean(&value) {
        crate::js_string_owned(&value)
    } else {
        fallback.to_owned()
    }
}

fn to_number(value: &JsValue, fallback: f64) -> f64 {
    let number = crate::js_number(value);
    if number.is_finite() { number } else { fallback }
}

fn number_or_zero(value: &JsValue) -> f64 {
    let number = crate::js_number(value);
    if number.is_nan() || number == 0.0 {
        0.0
    } else {
        number
    }
}

fn first_truthy(target: &JsValue, names: &[&str]) -> JsValue {
    for name in names {
        let value = property(target, name);
        if crate::js_boolean(&value) {
            return value;
        }
    }
    JsValue::from_f64(0.0)
}

fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn locale_compare(left: &str, right: &str) -> Ordering {
    let global: JsValue = js_sys::global().into();
    let string_constructor = property(&global, "String");
    let string_prototype = property(&string_constructor, "prototype");
    let left_value = JsValue::from_str(left);
    let result = function(&string_prototype, "localeCompare")
        .and_then(|method| method.call1(&left_value, &JsValue::from_str(right)).ok())
        .map(|value| crate::js_number(&value))
        .unwrap_or(0.0);
    if result < 0.0 {
        Ordering::Less
    } else if result > 0.0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn timestamp_millis(raw: f64) -> Option<f64> {
    if !raw.is_finite() || raw <= 0.0 {
        return None;
    }
    Some(if raw > 10_000_000_000.0 {
        raw
    } else {
        raw * 1_000.0
    })
}

fn format_date_parts(
    year: u32,
    month_zero_based: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> String {
    format!(
        "{year:04}-{:02}-{day:02} {hour:02}:{minute:02}:{second:02}",
        month_zero_based + 1
    )
}

fn date_time_from_timestamp(value: &JsValue) -> String {
    let raw = to_number(value, 0.0);
    let Some(millis) = timestamp_millis(raw) else {
        return String::new();
    };
    let date = Date::new(&JsValue::from_f64(millis));
    if date.get_time().is_nan() {
        return String::new();
    }
    format_date_parts(
        date.get_full_year(),
        date.get_month(),
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds(),
    )
}

fn normalize_transaction(tx: &JsValue, day: &str, price_usd: f64) -> Option<JsValue> {
    if !tx.is_object() || tx.is_null() {
        return None;
    }

    let txid_value = pick_properties(
        tx,
        &["txid", "transaction_id", "transactionId", "id", "hash"],
    );
    let txid = if crate::js_boolean(&txid_value) {
        crate::js_string_owned(&txid_value).trim().to_owned()
    } else {
        String::new()
    };
    if txid.is_empty() {
        return None;
    }

    let timestamp_ms = to_number(
        &pick_properties(
            tx,
            &[
                "timestamp_ms",
                "timestampMs",
                "timestamp",
                "block_time",
                "blockTime",
            ],
        ),
        0.0,
    );

    let amount_kas = to_number(&pick_properties(tx, &["amount_kas", "amountKas"]), f64::NAN);
    let amount_sompi = to_number(&pick_properties(tx, &["amount_sompi", "amountSompi"]), 0.0);
    let amount = if amount_kas.is_finite() {
        amount_kas
    } else {
        amount_sompi / SOMPI_PER_KAS
    };

    let derived_datetime = date_time_from_timestamp(&JsValue::from_f64(timestamp_ms));
    let datetime_value = pick(&[
        property(tx, "datetime"),
        property(tx, "date_time"),
        property(tx, "dateTime"),
        JsValue::from_str(&derived_datetime),
    ]);
    let datetime = if crate::js_boolean(&datetime_value) {
        crate::js_string_owned(&datetime_value)
    } else {
        String::new()
    };

    let explicit_value = to_number(&property(tx, "value"), f64::NAN);
    let value = if explicit_value.is_finite() {
        to_number(&property(tx, "value"), 0.0)
    } else if price_usd.is_finite() {
        amount * price_usd
    } else {
        0.0
    };

    let date_prefix: String = datetime.chars().take(10).collect();
    let date_value = pick(&[
        property(tx, "date"),
        JsValue::from_str(day),
        JsValue::from_str(&date_prefix),
    ]);
    let date = if crate::js_boolean(&date_value) {
        crate::js_string_owned(&date_value)
    } else {
        String::new()
    };

    let output = Object::new();
    let target: &JsValue = output.as_ref();
    set(target, "date", &JsValue::from_str(&date));
    set(target, "datetime", &JsValue::from_str(&datetime));
    set(target, "txid", &JsValue::from_str(&txid));
    set(
        target,
        "direction",
        &JsValue::from_str(&truthy_pick_string(tx, &["direction"], "unknown")),
    );
    set(target, "amount", &JsValue::from_f64(amount));
    set(target, "value", &JsValue::from_f64(value));
    set(
        target,
        "type",
        &JsValue::from_str(&truthy_pick_string(
            tx,
            &["type", "tx_type", "txType"],
            "transfer",
        )),
    );
    set(
        target,
        "from_address",
        &pick_properties(tx, &["from_address", "fromAddress", "from"]),
    );
    set(
        target,
        "to_address",
        &pick_properties(tx, &["to_address", "toAddress", "to"]),
    );
    set(
        target,
        "counterparty",
        &pick_properties(tx, &["counterparty", "counterParty"]),
    );
    set(
        target,
        "block_height",
        &pick_properties(tx, &["block_height", "blockHeight"]),
    );
    set(target, "timestamp_ms", &JsValue::from_f64(timestamp_ms));
    Some(output.into())
}

fn push_row(
    rows: &mut Vec<JsValue>,
    seen: &mut HashSet<String>,
    tx: &JsValue,
    day: &str,
    price_usd: f64,
) {
    let Some(row) = normalize_transaction(tx, day, price_usd) else {
        return;
    };
    let txid = crate::js_string_owned(&property(&row, "txid"));
    if !seen.insert(txid) {
        return;
    }
    rows.push(row);
}

fn consume_groups(
    rows: &mut Vec<JsValue>,
    seen: &mut HashSet<String>,
    groups: &JsValue,
    price_usd: f64,
) {
    if !Array::is_array(groups) {
        return;
    }
    for group in Array::from(groups).iter() {
        let day_value = first_truthy(&group, &["day", "date"]);
        let day = if crate::js_boolean(&day_value) {
            crate::js_string_owned(&day_value)
        } else {
            String::new()
        };
        let transactions = property(&group, "transactions");
        let group_rows = property(&group, "rows");
        let txs = if Array::is_array(&transactions) {
            Array::from(&transactions)
        } else if Array::is_array(&group_rows) {
            Array::from(&group_rows)
        } else {
            Array::new()
        };
        for tx in txs.iter() {
            push_row(rows, seen, &tx, &day, price_usd);
        }
    }
}

fn normalize_unified_result(result: &JsValue, price_usd: f64) -> JsValue {
    let mut rows = Vec::new();
    let mut seen = HashSet::new();

    if Array::is_array(result) {
        let input = Array::from(result);
        let looks_grouped = input
            .iter()
            .any(|item| Array::is_array(&property(&item, "transactions")));
        if looks_grouped {
            consume_groups(&mut rows, &mut seen, result, price_usd);
        } else {
            for tx in input.iter() {
                push_row(&mut rows, &mut seen, &tx, "", price_usd);
            }
        }
    } else {
        consume_groups(&mut rows, &mut seen, &property(result, "groups"), price_usd);
        let result_rows = property(result, "rows");
        if Array::is_array(&result_rows) {
            for tx in Array::from(&result_rows).iter() {
                push_row(&mut rows, &mut seen, &tx, "", price_usd);
            }
        }
        let transactions = property(result, "transactions");
        if Array::is_array(&transactions) {
            for tx in Array::from(&transactions).iter() {
                push_row(&mut rows, &mut seen, &tx, "", price_usd);
            }
        }
    }

    rows.sort_by(|left, right| {
        let right_ts = to_number(&property(right, "timestamp_ms"), 0.0);
        let left_ts = to_number(&property(left, "timestamp_ms"), 0.0);
        if right_ts != left_ts {
            return right_ts.partial_cmp(&left_ts).unwrap_or(Ordering::Equal);
        }
        locale_compare(
            &crate::js_string_owned(&property(right, "datetime")),
            &crate::js_string_owned(&property(left, "datetime")),
        )
    });

    let output = Array::new();
    for row in rows {
        output.push(&row);
    }
    output.into()
}

fn day_summary_rows(result: &JsValue) -> JsValue {
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
        let day_value = first_truthy(&item, &["day", "date"]);
        let day = if crate::js_boolean(&day_value) {
            crate::js_string_owned(&day_value)
        } else {
            String::new()
        };
        if day.is_empty() {
            continue;
        }
        let normalized = Object::new();
        let target: &JsValue = normalized.as_ref();
        set(target, "__kgwDaySummary", &JsValue::TRUE);
        set(target, "day", &JsValue::from_str(&day));
        set(
            target,
            "count",
            &JsValue::from_f64(number_or_zero(&first_truthy(
                &item,
                &["count", "tx_count", "transactions_count"],
            ))),
        );
        set(
            target,
            "incoming_kas",
            &JsValue::from_f64(number_or_zero(&first_truthy(
                &item,
                &["incoming_kas", "incomingKas"],
            ))),
        );
        set(
            target,
            "outgoing_kas",
            &JsValue::from_f64(number_or_zero(&first_truthy(
                &item,
                &["outgoing_kas", "outgoingKas"],
            ))),
        );
        set(
            target,
            "net_kas",
            &JsValue::from_f64(number_or_zero(&first_truthy(&item, &["net_kas", "netKas"]))),
        );
        output.push(normalized.as_ref());
    }
    output.into()
}

#[wasm_bindgen(js_name = explorerNormalizeUnifiedResult)]
pub fn explorer_normalize_unified_result(result: JsValue) -> Result<JsValue, JsValue> {
    Ok(normalize_unified_result(
        &result,
        crate::parse_header_usd_price()?,
    ))
}

#[wasm_bindgen(js_name = explorerDaySummaryRowsFromResult)]
pub fn explorer_day_summary_rows_from_result(result: JsValue) -> JsValue {
    day_summary_rows(&result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_units_match_legacy_threshold() {
        assert_eq!(timestamp_millis(1_000.0), Some(1_000_000.0));
        assert_eq!(timestamp_millis(10_000_000_001.0), Some(10_000_000_001.0));
        assert_eq!(timestamp_millis(0.0), None);
        assert_eq!(timestamp_millis(f64::INFINITY), None);
    }

    #[test]
    fn date_parts_preserve_local_display_shape() {
        assert_eq!(
            format_date_parts(2026, 8, 28, 7, 4, 9),
            "2026-09-28 07:04:09"
        );
    }
}
