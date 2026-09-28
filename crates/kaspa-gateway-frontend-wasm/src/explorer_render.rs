use js_sys::{Array, Map, Object, Reflect, Set};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::spawn_local;

const EMPTY_TABLE_HTML: &str = r#"<tr><td colspan="6" class="muted" data-i18n="explorer.noTransactionsToDisplay">No transactions to display.</td></tr>"#;

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    let global = global();
    let candidate = property(&global, "window");
    if present(&candidate) {
        candidate
    } else {
        global
    }
}

fn document() -> JsValue {
    property(&global(), "document")
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

fn set(target: &JsValue, name: &str, value: &JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(name), value);
}

fn function(target: &JsValue, name: &str) -> Option<js_sys::Function> {
    property(target, name).dyn_into::<js_sys::Function>().ok()
}

fn call0(target: &JsValue, name: &str) -> JsValue {
    function(target, name)
        .and_then(|method| method.call0(target).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn call2(target: &JsValue, name: &str, a: &JsValue, b: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call2(target, a, b).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn query(scope: &JsValue, selector: &str) -> JsValue {
    call1(scope, "querySelector", &JsValue::from_str(selector))
}

fn query_all(scope: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(scope, "querySelectorAll", &JsValue::from_str(selector));
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(present)
        })
        .collect()
}

fn scope(section: &JsValue) -> JsValue {
    if present(section) {
        return section.clone();
    }
    let doc = document();
    let explorer = query(&doc, "#explorer");
    if present(&explorer) { explorer } else { doc }
}

fn create_element(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag))
}

fn append(parent: &JsValue, child: &JsValue) {
    let _ = call1(parent, "appendChild", child);
}

fn js_text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn bool_property(target: &JsValue, name: &str) -> bool {
    crate::js_boolean(&property(target, name))
}

fn number_property(target: &JsValue, names: &[&str]) -> f64 {
    for name in names {
        let value = property(target, name);
        if present(&value) {
            let number = crate::js_number(&value);
            if number.is_finite() {
                return number;
            }
        }
    }
    0.0
}

fn text_property(target: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = property(target, name);
        if present(&value) {
            let text = js_text(&value);
            if !text.is_empty() {
                return text;
            }
        }
    }
    String::new()
}

fn state() -> JsValue {
    crate::explorer_state::explorer_state()
}

fn state_array(name: &str) -> Array {
    let value = property(&state(), name);
    if Array::is_array(&value) {
        Array::from(&value)
    } else {
        Array::new()
    }
}

fn set_state_array(name: &str, value: &Array) {
    set(&state(), name, value.as_ref());
}

fn set_state_text(name: &str, value: &str) {
    set(&state(), name, &JsValue::from_str(value));
}

fn set_state_bool(name: &str, value: bool) {
    set(&state(), name, &JsValue::from_bool(value));
}

fn copy_array(value: &Array) -> Array {
    value.slice(0, value.length())
}

fn array_len(name: &str) -> u32 {
    state_array(name).length()
}

fn sync_action_state(section: &JsValue) {
    let _ = crate::explorer_runtime::explorer_sync_action_state(
        section.clone(),
        bool_property(&state(), "busy"),
        array_len("rows"),
        array_len("filteredRows"),
    );
}

fn set_status(section: &JsValue, message: &str, level: &str) {
    let _ = crate::explorer_runtime::explorer_set_status(
        section.clone(),
        JsValue::from_str(message),
        JsValue::from_str(level),
    );
}

fn log_clean2(label: &str, payload: &JsValue) {
    crate::explorer_runtime::explorer_clean2_log(label.to_owned(), payload.clone());
}

fn console_error(message: &str) {
    let console = property(&global(), "console");
    if let Some(error) = function(&console, "error") {
        let _ = error.call1(&console, &JsValue::from_str(message));
    }
}

fn console_warn(message: &str) {
    let console = property(&global(), "console");
    if let Some(warn) = function(&console, "warn") {
        let _ = warn.call1(&console, &JsValue::from_str(message));
    }
}

fn expanded_set() -> JsValue {
    let win = window();
    let existing = property(&win, "__kgwExplorerExpandedDateGroups");
    if present(&existing) {
        return existing;
    }
    let set_value: JsValue = Set::new(&JsValue::UNDEFINED).into();
    set(&win, "__kgwExplorerExpandedDateGroups", &set_value);
    set_value
}

fn transaction_cache() -> JsValue {
    let win = window();
    let existing = property(&win, "__kgwExplorerDayTransactionCache");
    if present(&existing) {
        return existing;
    }
    let map_value: JsValue = Map::new().into();
    set(&win, "__kgwExplorerDayTransactionCache", &map_value);
    map_value
}

fn reset_transaction_cache() {
    let map_value: JsValue = Map::new().into();
    set(&window(), "__kgwExplorerDayTransactionCache", &map_value);
}

fn set_has(set_value: &JsValue, key: &str) -> bool {
    crate::js_boolean(&call1(set_value, "has", &JsValue::from_str(key)))
}

fn set_add(set_value: &JsValue, key: &str) {
    let _ = call1(set_value, "add", &JsValue::from_str(key));
}

fn set_delete(set_value: &JsValue, key: &str) {
    let _ = call1(set_value, "delete", &JsValue::from_str(key));
}

fn map_has(map_value: &JsValue, key: &str) -> bool {
    crate::js_boolean(&call1(map_value, "has", &JsValue::from_str(key)))
}

fn map_get(map_value: &JsValue, key: &str) -> JsValue {
    call1(map_value, "get", &JsValue::from_str(key))
}

fn map_set(map_value: &JsValue, key: &str, value: &JsValue) {
    let _ = call2(map_value, "set", &JsValue::from_str(key), value);
}

fn locale_number(value: f64) -> String {
    let source = JsValue::from_f64(value);
    let rendered = call0(&source, "toLocaleString");
    if present(&rendered) {
        js_text(&rendered)
    } else {
        value.to_string()
    }
}

fn safe_text(value: &str) -> String {
    crate::kgw_clean2_safe_text(JsValue::from_str(value))
}

fn format_kas(value: f64) -> String {
    crate::kgw_clean2_kas(JsValue::from_f64(value))
}

fn format_usd(value: f64) -> String {
    crate::kgw_clean2_usd(JsValue::from_f64(value))
}

fn render_summaries_impl(
    section: JsValue,
    rows: JsValue,
    status_text: String,
) -> Result<(), JsValue> {
    let root = crate::explorer_runtime::explorer_clean2_section(section);
    let body = crate::explorer_runtime::explorer_clean2_body(root.clone());
    if !present(&body) {
        console_error("[KGW Explorer][clean2] tbody not found");
        return Ok(());
    }

    let normalized = crate::explorer_summary::explorer_normalize_day_summaries(rows)?;
    let summaries = Array::from(&normalized);
    set_state_array("rows", &summaries);
    set_state_array("filteredRows", &copy_array(&summaries));
    set(&body, "innerHTML", &JsValue::from_str(""));

    if summaries.length() == 0 {
        set(&body, "innerHTML", &JsValue::from_str(EMPTY_TABLE_HTML));
        if !status_text.is_empty() {
            set_status(&root, &status_text, "");
        }
        sync_action_state(&root);

        let details = Object::new();
        set(details.as_ref(), "tbodyFound", &JsValue::TRUE);
        set(
            details.as_ref(),
            "childRows",
            &JsValue::from_f64(crate::js_number(&property(
                &property(&body, "children"),
                "length",
            ))),
        );
        log_clean2("render empty", details.as_ref());
        return Ok(());
    }

    let mut total_count = 0.0;
    let mut total_net_kas = 0.0;
    let mut total_usd = 0.0;
    for summary in summaries.iter() {
        total_count += number_property(&summary, &["count"]);
        total_net_kas += number_property(&summary, &["net_kas", "netKas"]);
        total_usd += number_property(&summary, &["value_usd", "valueUsd"]);
    }

    let fragment = call0(&document(), "createDocumentFragment");
    let total_tr = create_element("tr");
    set(&total_tr, "className", &JsValue::from_str("kgw-total-row"));
    let total_html = format!(
        "<td data-i18n=\"explorer.total\">Total</td><td>{} transactions</td><td></td><td>{}</td><td>{}</td><td></td>",
        locale_number(total_count),
        format_kas(total_net_kas),
        format_usd(total_usd),
    );
    set(&total_tr, "innerHTML", &JsValue::from_str(&total_html));
    append(&fragment, &total_tr);

    let expanded = expanded_set();
    let cache = transaction_cache();
    for summary in summaries.iter() {
        let day = text_property(&summary, &["day"]);
        if day.is_empty() {
            continue;
        }
        let count = number_property(&summary, &["count"]);
        let net_kas = number_property(&summary, &["net_kas", "netKas"]);
        let is_expanded = set_has(&expanded, &day);
        let sign = if is_expanded { "-" } else { "+" };

        let day_tr = create_element("tr");
        set(
            &day_tr,
            "className",
            &JsValue::from_str("kgw-day-group-row"),
        );
        set(
            &property(&day_tr, "dataset"),
            "kgwDateGroup",
            &JsValue::from_str(&day),
        );
        let day_html = format!(
            "<td>{sign} {}</td><td>{} transactions</td><td></td><td>{}</td><td>{}</td><td></td>",
            safe_text(&day),
            locale_number(count),
            format_kas(net_kas),
            format_usd(number_property(&summary, &["value_usd", "valueUsd"])),
        );
        set(&day_tr, "innerHTML", &JsValue::from_str(&day_html));
        append(&fragment, &day_tr);

        if !is_expanded {
            continue;
        }

        if !map_has(&cache, &day) {
            let loading = create_element("tr");
            set(
                &loading,
                "className",
                &JsValue::from_str("kgw-transaction-row"),
            );
            set(
                &loading,
                "innerHTML",
                &JsValue::from_str(&format!(
                    "<td colspan=\"6\" class=\"muted\">Loading transactions for {}...</td>",
                    safe_text(&day),
                )),
            );
            append(&fragment, &loading);
            continue;
        }

        let cached = map_get(&cache, &day);
        if !Array::is_array(&cached) {
            continue;
        }

        let cached_rows = Array::from(&cached);
        let current_price =
            crate::explorer_summary::explorer_summary_current_usd_price().unwrap_or(0.0);
        for tx in cached_rows.iter() {
            let txid = text_property(&tx, &["txid", "transaction_id", "transactionId"]);
            let amount = number_property(&tx, &["amount_kas", "amountKas", "amount"]);
            let mut explicit_usd = None;
            for key in ["value_usd", "valueUsd", "usd_value", "usdValue", "value"] {
                let candidate = property(&tx, key);
                if present(&candidate) {
                    let number = crate::js_number(&candidate);
                    if number.is_finite() {
                        explicit_usd = Some(number);
                        break;
                    }
                }
            }
            let usd = explicit_usd.unwrap_or_else(|| amount.abs() * current_price);
            let tx_tr = create_element("tr");
            set(
                &tx_tr,
                "className",
                &JsValue::from_str("kgw-transaction-row"),
            );
            set(
                &property(&tx_tr, "dataset"),
                "kgwParentDateGroup",
                &JsValue::from_str(&day),
            );
            let tx_html = format!(
                "<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
                safe_text(&text_property(
                    &tx,
                    &["datetime", "date_time", "time", "timestamp"],
                )),
                safe_text(&txid),
                safe_text(&text_property(&tx, &["direction"])),
                format_kas(amount),
                format_usd(usd),
                safe_text(&text_property(&tx, &["tx_type", "type"])),
            );
            set(&tx_tr, "innerHTML", &JsValue::from_str(&tx_html));
            append(&fragment, &tx_tr);
        }
    }

    append(&body, &fragment);

    for row in query_all(&body, "[data-kgw-date-group]") {
        let root_for_click = root.clone();
        let status_for_click = status_text.clone();
        let row_for_click = row.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let day = js_text(&property(
                &property(&row_for_click, "dataset"),
                "kgwDateGroup",
            ));
            if day.is_empty() {
                return;
            }

            let expanded = expanded_set();
            if set_has(&expanded, &day) {
                set_delete(&expanded, &day);
                let rows = property(&state(), "rows");
                let _ =
                    render_summaries_impl(root_for_click.clone(), rows, status_for_click.clone());
                return;
            }

            set_add(&expanded, &day);
            let rows = property(&state(), "rows");
            let _ = render_summaries_impl(root_for_click.clone(), rows, status_for_click.clone());

            let cache = transaction_cache();
            if map_has(&cache, &day) {
                return;
            }

            let root_async = root_for_click.clone();
            let status_async = status_for_click.clone();
            spawn_local(async move {
                let current_state = state();
                let selected = js_text(&property(&current_state, "selectedAddress"));
                let address = if selected.is_empty() {
                    let input = query(&root_async, "#explorerAddress");
                    crate::explorer_addresses::explorer_normalize_address(property(&input, "value"))
                } else {
                    selected
                };

                let loaded = crate::explorer_runtime::explorer_clean2_load_day_transactions(
                    root_async.clone(),
                    JsValue::from_str(&address),
                    day.clone(),
                )
                .await;

                let cache = transaction_cache();
                match loaded {
                    Ok(rows) if Array::is_array(&rows) => {
                        map_set(&cache, &day, &rows);
                    }
                    Ok(_) => {
                        let empty: JsValue = Array::new().into();
                        map_set(&cache, &day, &empty);
                    }
                    Err(error) => {
                        console_error(&format!(
                            "[KGW Explorer][clean2] day transactions load failed: {}",
                            js_text(&error),
                        ));
                        let empty: JsValue = Array::new().into();
                        map_set(&cache, &day, &empty);
                    }
                }

                let rows = property(&state(), "rows");
                let _ = render_summaries_impl(root_async, rows, status_async);
            });
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &row,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    if !status_text.is_empty() {
        set_status(&root, &status_text, "");
    }
    sync_action_state(&root);

    let details = Object::new();
    set(
        details.as_ref(),
        "days",
        &JsValue::from_f64(summaries.length() as f64),
    );
    set(
        details.as_ref(),
        "childRows",
        &property(&property(&body, "children"), "length"),
    );
    let preview = js_text(&property(&body, "innerText"))
        .chars()
        .take(260)
        .collect::<String>();
    set(
        details.as_ref(),
        "textPreview",
        &JsValue::from_str(&preview),
    );
    log_clean2("render done", details.as_ref());
    Ok(())
}

fn render_table_impl(section: JsValue) -> Result<(), JsValue> {
    let state = state();
    let filtered = property(&state, "filteredRows");
    let rows = if Array::is_array(&filtered) {
        filtered
    } else {
        let fallback = property(&state, "rows");
        if Array::is_array(&fallback) {
            fallback
        } else {
            Array::new().into()
        }
    };
    render_summaries_impl(scope(&section), rows, String::new())
}

fn preserve_cancel(options: &JsValue) -> bool {
    present(options) && crate::js_boolean(&property(options, "preserveCancelRequested"))
}

fn clear_table_impl(section: JsValue, reason: String, options: JsValue) -> Result<(), JsValue> {
    let root = scope(&section);
    set_state_array("rows", &Array::new());
    set_state_array("filteredRows", &Array::new());
    if !preserve_cancel(&options) {
        set_state_bool("cancelRequested", false);
    }

    render_table_impl(root.clone())?;
    sync_action_state(&root);
    if !reason.is_empty() {
        set_status(&root, &reason, "");
    }
    Ok(())
}

fn reset_filters_impl(section: JsValue) -> Result<(), JsValue> {
    let root = scope(&section);
    for (selector, value) in [
        ("#explorerDirectionFilter", "ALL"),
        ("#explorerTypeFilter", "ALL"),
        ("#explorerSearch", ""),
    ] {
        let node = query(&root, selector);
        if present(&node) {
            set(&node, "value", &JsValue::from_str(value));
        }
    }
    let rows = state_array("rows");
    set_state_array("filteredRows", &copy_array(&rows));
    render_table_impl(root)
}

fn displayed_days_status_text(rendered_days: &str) -> String {
    format!("Displayed {rendered_days} days from local database. Click + to load one day.")
}

fn displayed_days_status(days: u32) -> String {
    displayed_days_status_text(&locale_number(days as f64))
}

async fn load_and_render_impl(
    section: JsValue,
    address: JsValue,
    start_ts: JsValue,
    end_ts: JsValue,
    status_text: String,
) -> Result<JsValue, JsValue> {
    reset_transaction_cache();
    let summaries = crate::explorer_runtime::explorer_load_transaction_day_summaries_from_db(
        section.clone(),
        address.clone(),
        start_ts,
        end_ts,
    )
    .await?;
    let address_text = js_text(&address);
    set_state_text("selectedAddress", &address_text);
    let count = if Array::is_array(&summaries) {
        Array::from(&summaries).length()
    } else {
        0
    };
    let message = if status_text.is_empty() {
        displayed_days_status(count)
    } else {
        status_text
    };
    render_summaries_impl(section, summaries.clone(), message)?;
    Ok(summaries)
}

fn price_rerender_root() -> JsValue {
    let doc = document();
    let direct = query(&doc, "#explorer");
    if present(&direct) {
        return direct;
    }
    let fallback = query(&doc, ".explorer-python-root");
    if present(&fallback) { fallback } else { doc }
}

fn install_price_rerender_impl() -> bool {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwExplorerPriceRerenderV1Installed")) {
        return false;
    }
    set(
        &win,
        "__kgwExplorerPriceRerenderV1Installed",
        &JsValue::TRUE,
    );

    let callback = Closure::wrap(Box::new(move |_event: JsValue| {
        let section = price_rerender_root();
        let rows = property(&state(), "rows");
        if !present(&section) || !Array::is_array(&rows) {
            return;
        }
        let has_day_summary = Array::from(&rows)
            .iter()
            .any(|row| crate::js_boolean(&property(&row, "__kgwDaySummary")));
        if !has_day_summary {
            return;
        }
        if let Err(error) = render_summaries_impl(section, rows, String::new()) {
            console_warn(&format!(
                "[KGW Explorer] price rerender failed: {}",
                js_text(&error)
            ));
        }
    }) as Box<dyn FnMut(JsValue)>);

    let registered = function(&win, "addEventListener")
        .and_then(|add| {
            add.call2(
                &win,
                &JsValue::from_str("kgw:kaspa-price-updated"),
                callback.as_ref().unchecked_ref(),
            )
            .ok()
        })
        .is_some();
    if registered {
        callback.forget();
        true
    } else {
        set(
            &win,
            "__kgwExplorerPriceRerenderV1Installed",
            &JsValue::FALSE,
        );
        false
    }
}

#[wasm_bindgen(js_name = explorerSyncCurrentActionState)]
pub fn explorer_sync_current_action_state(section: JsValue) -> Result<(), JsValue> {
    let root = scope(&section);
    crate::explorer_runtime::explorer_sync_action_state(
        root,
        bool_property(&state(), "busy"),
        array_len("rows"),
        array_len("filteredRows"),
    )
}

#[wasm_bindgen(js_name = explorerInstallPriceRerender)]
pub fn explorer_install_price_rerender() -> bool {
    install_price_rerender_impl()
}

#[wasm_bindgen(js_name = explorerRenderSummaries)]
pub fn explorer_render_summaries(
    section: JsValue,
    rows: JsValue,
    status_text: String,
) -> Result<(), JsValue> {
    render_summaries_impl(section, rows, status_text)
}

#[wasm_bindgen(js_name = explorerRenderTable)]
pub fn explorer_render_table(section: JsValue) -> Result<(), JsValue> {
    render_table_impl(section)
}

#[wasm_bindgen(js_name = explorerClearTransactionTable)]
pub fn explorer_clear_transaction_table(
    section: JsValue,
    reason: String,
    options: JsValue,
) -> Result<(), JsValue> {
    clear_table_impl(section, reason, options)
}

#[wasm_bindgen(js_name = explorerResetFilters)]
pub fn explorer_reset_filters(section: JsValue) -> Result<(), JsValue> {
    reset_filters_impl(section)
}

#[wasm_bindgen(js_name = explorerLoadAndRenderDaySummaries)]
pub async fn explorer_load_and_render_day_summaries(
    section: JsValue,
    address: JsValue,
    start_ts: JsValue,
    end_ts: JsValue,
    status_text: String,
) -> Result<JsValue, JsValue> {
    load_and_render_impl(section, address, start_ts, end_ts, status_text).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_status_preserves_legacy_copy() {
        assert_eq!(
            displayed_days_status_text("3"),
            "Displayed 3 days from local database. Click + to load one day."
        );
    }

    #[test]
    fn empty_table_contract_preserves_i18n_key_and_columns() {
        assert!(EMPTY_TABLE_HTML.contains("explorer.noTransactionsToDisplay"));
        assert!(EMPTY_TABLE_HTML.contains("colspan=\"6\""));
    }
}
