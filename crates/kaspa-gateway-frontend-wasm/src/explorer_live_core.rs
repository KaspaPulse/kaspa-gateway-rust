use js_sys::{Array, Date, Object, Reflect};
use std::cell::RefCell;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

const SOMPI_PER_KAS: f64 = 100_000_000.0;
const RENDER_THROTTLE_MS: f64 = 1_500.0;

#[derive(Clone, Debug, PartialEq)]
struct LiveSummary {
    day: String,
    count: f64,
    incoming_kas: f64,
    outgoing_kas: f64,
    net_kas: f64,
    value_usd: f64,
}

impl LiveSummary {
    fn empty(day: String) -> Self {
        Self {
            day,
            count: 0.0,
            incoming_kas: 0.0,
            outgoing_kas: 0.0,
            net_kas: 0.0,
            value_usd: 0.0,
        }
    }
    fn recompute(&mut self, price: f64) {
        self.net_kas = self.incoming_kas - self.outgoing_kas;
        self.value_usd = if price.is_finite() && price > 0.0 {
            (self.incoming_kas.abs() + self.outgoing_kas.abs()) * price
        } else {
            0.0
        };
    }
}

#[derive(Default)]
struct LiveState {
    address: String,
    days: BTreeMap<String, LiveSummary>,
    last_render_ms: f64,
}

thread_local! {
    static STATE: RefCell<LiveState> = RefCell::new(LiveState::default());
}

fn present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if !present(target) {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}
fn first_present(target: &JsValue, names: &[&str]) -> JsValue {
    for name in names {
        let value = property(target, name);
        if present(&value) {
            return value;
        }
    }
    JsValue::from_f64(0.0)
}

fn number_or_zero(value: &JsValue) -> f64 {
    let value = crate::js_number(value);
    if value.is_nan() || value == 0.0 {
        0.0
    } else {
        value
    }
}

fn truthy_number(target: &JsValue, name: &str) -> f64 {
    let value = property(target, name);
    if crate::js_boolean(&value) {
        number_or_zero(&value)
    } else {
        0.0
    }
}

fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn current_price() -> f64 {
    crate::parse_header_usd_price().unwrap_or(0.0)
}
fn day_from_millis(value: &JsValue) -> String {
    let millis = number_or_zero(value);
    if !millis.is_finite() || millis <= 0.0 {
        return String::new();
    }
    String::from(Date::new(&JsValue::from_f64(millis)).to_iso_string())
        .chars()
        .take(10)
        .collect()
}

fn summary_from_row(row: &JsValue) -> Option<LiveSummary> {
    if !crate::js_boolean(&property(row, "__kgwDaySummary")) {
        return None;
    }
    let day = text(&property(row, "day"));
    if day.is_empty() {
        return None;
    }
    Some(LiveSummary {
        day,
        count: number_or_zero(&property(row, "count")),
        incoming_kas: number_or_zero(&property(row, "incoming_kas")),
        outgoing_kas: number_or_zero(&property(row, "outgoing_kas")),
        net_kas: number_or_zero(&property(row, "net_kas")),
        value_usd: number_or_zero(&property(row, "value_usd")),
    })
}

fn summary_to_js(summary: &LiveSummary) -> JsValue {
    let output = Object::new();
    let target: &JsValue = output.as_ref();
    for (key, value) in [
        ("__kgwDaySummary", JsValue::TRUE),
        ("day", JsValue::from_str(&summary.day)),
        ("count", JsValue::from_f64(summary.count)),
        ("incoming_kas", JsValue::from_f64(summary.incoming_kas)),
        ("outgoing_kas", JsValue::from_f64(summary.outgoing_kas)),
        ("net_kas", JsValue::from_f64(summary.net_kas)),
        ("value_usd", JsValue::from_f64(summary.value_usd)),
    ] {
        let _ = Reflect::set(target, &JsValue::from_str(key), &value);
    }
    output.into()
}

fn merge_record(summary: &mut LiveSummary, amount_sompi: f64, direction: &str, price: f64) {
    let amount_kas = amount_sompi.abs() / SOMPI_PER_KAS;
    summary.count += 1.0;
    if direction.eq_ignore_ascii_case("outgoing") {
        summary.outgoing_kas += amount_kas;
    } else {
        summary.incoming_kas += amount_kas;
    }
    summary.recompute(price);
}

fn merge_day(
    summary: &mut LiveSummary,
    count: f64,
    incoming_sompi: f64,
    outgoing_sompi: f64,
    price: f64,
) {
    summary.count += count;
    summary.incoming_kas += incoming_sompi.abs() / SOMPI_PER_KAS;
    summary.outgoing_kas += outgoing_sompi.abs() / SOMPI_PER_KAS;
    summary.recompute(price);
}
fn should_render_at(page: f64, now: f64, last: f64) -> (bool, f64) {
    if page <= 1.0 || page % 2.0 == 0.0 || now - last >= RENDER_THROTTLE_MS {
        (true, now)
    } else {
        (false, last)
    }
}

#[wasm_bindgen(js_name = explorerLiveCoreReset)]
pub fn explorer_live_core_reset(address: String) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.address = address;
        state.days.clear();
    });
}

#[wasm_bindgen(js_name = explorerLiveCoreAddress)]
pub fn explorer_live_core_address() -> String {
    STATE.with(|state| state.borrow().address.clone())
}

#[wasm_bindgen(js_name = explorerLiveCoreSeedRows)]
pub fn explorer_live_core_seed_rows(address: String, rows: JsValue) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.address.is_empty() {
            state.address = address;
        }
        if !Array::is_array(&rows) {
            return;
        }
        for row in Array::from(&rows).iter() {
            if let Some(summary) = summary_from_row(&row) {
                state.days.insert(summary.day.clone(), summary);
            }
        }
    });
}

#[wasm_bindgen(js_name = explorerLiveCoreMergeRecords)]
pub fn explorer_live_core_merge_records(records: JsValue) {
    if !Array::is_array(&records) {
        return;
    }
    let price = current_price();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for record in Array::from(&records).iter() {
            let day = day_from_millis(&first_present(&record, &["timestamp_ms", "timestampMs"]));
            if day.is_empty() {
                continue;
            }
            let amount = number_or_zero(&first_present(&record, &["amount_sompi", "amountSompi"]));
            let direction = text(&property(&record, "direction")).to_ascii_lowercase();
            let summary = state
                .days
                .entry(day.clone())
                .or_insert_with(|| LiveSummary::empty(day));
            merge_record(summary, amount, &direction, price);
        }
    });
}
#[wasm_bindgen(js_name = explorerLiveCoreMergeDays)]
pub fn explorer_live_core_merge_days(days: JsValue) {
    if !Array::is_array(&days) {
        return;
    }
    let price = current_price();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for item in Array::from(&days).iter() {
            let day: String = text(&property(&item, "day")).chars().take(10).collect();
            if day.is_empty() {
                continue;
            }
            let count = truthy_number(&item, "count");
            let incoming =
                number_or_zero(&first_present(&item, &["incoming_sompi", "incomingSompi"]));
            let outgoing =
                number_or_zero(&first_present(&item, &["outgoing_sompi", "outgoingSompi"]));
            let summary = state
                .days
                .entry(day.clone())
                .or_insert_with(|| LiveSummary::empty(day));
            merge_day(summary, count, incoming, outgoing, price);
        }
    });
}

#[wasm_bindgen(js_name = explorerLiveCoreRows)]
pub fn explorer_live_core_rows() -> JsValue {
    STATE.with(|state| {
        let output = Array::new();
        for summary in state.borrow().days.values().rev() {
            output.push(&summary_to_js(summary));
        }
        output.into()
    })
}

#[wasm_bindgen(js_name = explorerLiveCoreShouldRender)]
pub fn explorer_live_core_should_render(payload: JsValue) -> bool {
    let page_value = property(&payload, "page");
    let page = if crate::js_boolean(&page_value) {
        crate::js_number(&page_value)
    } else {
        0.0
    };
    let now = Date::now();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let (render, last) = should_render_at(page, now, state.last_render_ms);
        state.last_render_ms = last;
        render
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_merge_tracks_direction_and_usd_gross() {
        let mut summary = LiveSummary::empty("2026-09-28".to_owned());
        merge_record(&mut summary, 200_000_000.0, "incoming", 0.25);
        merge_record(&mut summary, 100_000_000.0, "outgoing", 0.25);
        assert_eq!(summary.count, 2.0);
        assert_eq!(summary.incoming_kas, 2.0);
        assert_eq!(summary.outgoing_kas, 1.0);
        assert_eq!(summary.net_kas, 1.0);
        assert_eq!(summary.value_usd, 0.75);
    }

    #[test]
    fn compact_day_merge_accumulates_and_recomputes() {
        let mut summary = LiveSummary::empty("2026-09-28".to_owned());
        merge_day(&mut summary, 3.0, 200_000_000.0, 100_000_000.0, 0.5);
        merge_day(&mut summary, 2.0, 100_000_000.0, 0.0, 0.5);
        assert_eq!(summary.count, 5.0);
        assert_eq!(summary.incoming_kas, 3.0);
        assert_eq!(summary.outgoing_kas, 1.0);
        assert_eq!(summary.net_kas, 2.0);
        assert_eq!(summary.value_usd, 2.0);
    }

    #[test]
    fn throttle_matches_page_and_time_policy() {
        assert_eq!(should_render_at(1.0, 100.0, 0.0), (true, 100.0));
        assert_eq!(should_render_at(2.0, 200.0, 100.0), (true, 200.0));
        assert_eq!(should_render_at(3.0, 500.0, 200.0), (false, 200.0));
        assert_eq!(should_render_at(3.0, 1_800.0, 200.0), (true, 1_800.0));
    }

    #[test]
    fn summaries_sort_descending_by_day() {
        let mut days = BTreeMap::new();
        days.insert(
            "2026-09-27".to_owned(),
            LiveSummary::empty("2026-09-27".to_owned()),
        );
        days.insert(
            "2026-09-28".to_owned(),
            LiveSummary::empty("2026-09-28".to_owned()),
        );
        let order = days
            .values()
            .rev()
            .map(|value| value.day.clone())
            .collect::<Vec<_>>();
        assert_eq!(order, vec!["2026-09-28", "2026-09-27"]);
    }
}
