use js_sys::{Array, Function, Map, Object, Promise, Reflect, Set};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, future_to_promise, spawn_local};

const PAGE_STORED_EVENT: &str = "kgw://transactions/page-stored";
const INIT_DATASET_KEY: &str = "kgwExplorerCleanInit";
const LIVE_LISTENER_FLAG: &str = "__kgwTxLiveCoreListenerInstalled";
const FILTER_CAPTURE_FLAG: &str = "__kgwClean2FilterCaptureInstalled";

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    let value = property(&global(), "window");
    if present(&value) { value } else { global() }
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

fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call0(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    function(target, name)
        .ok_or_else(|| JsValue::from_str(&format!("missing method {name}")))?
        .call0(target)
}

fn call1(target: &JsValue, name: &str, a: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)
        .ok_or_else(|| JsValue::from_str(&format!("missing method {name}")))?
        .call1(target, a)
}

fn call3(
    target: &JsValue,
    name: &str,
    a: &JsValue,
    b: &JsValue,
    c: &JsValue,
) -> Result<JsValue, JsValue> {
    function(target, name)
        .ok_or_else(|| JsValue::from_str(&format!("missing method {name}")))?
        .call3(target, a, b, c)
}

fn query(scope: &JsValue, selector: &str) -> JsValue {
    call1(scope, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn dataset(target: &JsValue) -> JsValue {
    property(target, "dataset")
}

fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn boolean(value: &JsValue) -> bool {
    crate::js_boolean(value)
}

fn number(value: &JsValue) -> f64 {
    crate::js_number(value)
}

fn object(entries: &[(&str, JsValue)]) -> JsValue {
    let output = Object::new();
    for (key, value) in entries {
        set(output.as_ref(), key, value);
    }
    output.into()
}

fn root() -> JsValue {
    let doc = document();
    let direct =
        call1(&doc, "getElementById", &JsValue::from_str("explorer")).unwrap_or(JsValue::UNDEFINED);
    if present(&direct) {
        return direct;
    }
    let fallback = query(&doc, ".explorer-python-root");
    if present(&fallback) { fallback } else { doc }
}

fn live_root() -> JsValue {
    let doc = document();
    let direct = query(&doc, "#explorer");
    if present(&direct) {
        return direct;
    }
    query(&doc, ".explorer-python-root")
}

fn value(scope: &JsValue, selector: &str) -> String {
    text(&property(&query(scope, selector), "value"))
}

fn state() -> JsValue {
    crate::explorer_state::explorer_state()
}

fn state_bool(name: &str) -> bool {
    boolean(&property(&state(), name))
}

fn state_text(name: &str) -> String {
    text(&property(&state(), name))
}

fn set_state(name: &str, value: &JsValue) {
    set(&state(), name, value);
}

fn rows_from_state() -> JsValue {
    property(&state(), "rows")
}

fn event_prevent(event: &JsValue) {
    let _ = call0(event, "preventDefault");
}

fn stop_event(event: &JsValue) {
    event_prevent(event);
    let _ = call0(event, "stopPropagation");
    let _ = call0(event, "stopImmediatePropagation");
}

fn event_trusted(event: &JsValue) -> bool {
    boolean(&property(event, "isTrusted"))
}

fn bind_property(target: &JsValue, name: &str, callback: Closure<dyn FnMut(JsValue)>) {
    if !present(target) {
        return;
    }
    set(target, name, callback.as_ref().unchecked_ref());
    callback.forget();
}

fn set_status(section: &JsValue, message: &str, level: &str) {
    crate::explorer_runtime::explorer_set_status(
        section.clone(),
        JsValue::from_str(message),
        JsValue::from_str(level),
    );
}

fn clear_table(section: &JsValue, reason: &str, preserve_cancel: bool) {
    let options = Object::new();
    if preserve_cancel {
        set(options.as_ref(), "preserveCancelRequested", &JsValue::TRUE);
    }
    let _ = crate::explorer_render::explorer_clear_transaction_table(
        section.clone(),
        reason.to_owned(),
        options.into(),
    );
}

fn trace(action: &'static str, phase: &'static str, details: JsValue) {
    spawn_local(async move {
        let _ = crate::explorer_runtime::explorer_ui_trace(
            JsValue::from_str(action),
            JsValue::from_str(phase),
            details,
        )
        .await;
    });
}

fn error_text(error: &JsValue) -> String {
    let message = property(error, "message");
    if boolean(&message) {
        text(&message)
    } else {
        text(error)
    }
}

fn ensure_runtime_collections() {
    let win = window();
    if !present(&property(&win, "__kgwExplorerDayTransactionCache")) {
        let map = Map::new();
        set(&win, "__kgwExplorerDayTransactionCache", map.as_ref());
    }
    if !present(&property(&win, "__kgwExplorerExpandedDateGroups")) {
        let set_value = Set::new(&JsValue::UNDEFINED);
        set(&win, "__kgwExplorerExpandedDateGroups", set_value.as_ref());
    }
}

fn reset_runtime_collections() {
    let win = window();
    let map = Map::new();
    set(&win, "__kgwExplorerDayTransactionCache", map.as_ref());
    let set_value = Set::new(&JsValue::UNDEFINED);
    set(&win, "__kgwExplorerExpandedDateGroups", set_value.as_ref());
}

fn set_global_fetch_busy(busy: bool, message: &str) {
    let win = window();
    let candidate = property(&win, "kgwSetGlobalFetchBusy");
    let Ok(callback) = candidate.dyn_into::<Function>() else {
        return;
    };
    let options = object(&[("text", JsValue::from_str(message))]);
    let _ = callback.call2(&win, &JsValue::from_bool(busy), &options);
}

fn format_count_number(number: f64) -> String {
    if !number.is_finite() {
        return "0".to_owned();
    }
    let raw = format!("{:.0}", number.max(0.0));
    let mut output = String::with_capacity(raw.len() + raw.len() / 3);
    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }
    output.chars().rev().collect()
}

fn format_count(value: &JsValue) -> String {
    format_count_number(number(value))
}

fn parse_range(section: &JsValue) -> (JsValue, JsValue) {
    (
        crate::parse_date_seconds(
            JsValue::from_str(&value(section, "#explorerFromDate")),
            false,
        ),
        crate::parse_date_seconds(JsValue::from_str(&value(section, "#explorerToDate")), true),
    )
}

async fn canonical_address_or_error(section: &JsValue, raw: String) -> Option<String> {
    let normalized = crate::explorer_addresses::explorer_normalize_address(JsValue::from_str(&raw));
    let address =
        crate::explorer_addresses::explorer_canonical_kaspa_address(JsValue::from_str(&normalized))
            .await;
    if address.is_empty() {
        clear_table(section, "Enter a valid Kaspa address.", false);
        set_status(section, "Enter a valid Kaspa address.", "error");
        None
    } else {
        Some(address)
    }
}

async fn apply_explorer_filters_from_database(section: JsValue) -> Result<(), JsValue> {
    if state_bool("busy") {
        set_status(
            &section,
            "Fetch is running. Wait until it finishes before applying filters.",
            "info",
        );
        return Ok(());
    }
    let Some(address) =
        canonical_address_or_error(&section, value(&section, "#explorerAddress")).await
    else {
        return Ok(());
    };
    let (start_ts, end_ts) = parse_range(&section);
    set_status(
        &section,
        "Loading filtered transactions from database...",
        "info",
    );
    let _ = crate::explorer_render::explorer_load_and_render_day_summaries(
        section,
        JsValue::from_str(&address),
        start_ts,
        end_ts,
        "Filter applied. Showing days from local database. Click + to load one day.".to_owned(),
    )
    .await?;
    Ok(())
}

fn bind_address_change(
    input: &JsValue,
    section: &JsValue,
    property_name: &str,
    phase: &'static str,
) {
    if !present(input) {
        return;
    }
    let input_for_event = input.clone();
    let section_for_event = section.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        trace(
            "explorer-address",
            phase,
            object(&[
                ("trusted", JsValue::from_bool(event_trusted(&event))),
                (
                    "valueLength",
                    JsValue::from_f64(text(&property(&input_for_event, "value")).len() as f64),
                ),
            ]),
        );
        let normalized = crate::explorer_addresses::explorer_normalize_address(property(
            &input_for_event,
            "value",
        ));
        set_state("selectedAddress", &JsValue::from_str(&normalized));
        clear_table(&section_for_event, "Address changed. Table cleared.", false);
    }) as Box<dyn FnMut(JsValue)>);
    bind_property(input, property_name, callback);
}

fn bind_date_change(input: &JsValue, section: &JsValue, phase: &'static str) {
    if !present(input) {
        return;
    }
    let input_for_event = input.clone();
    let section_for_event = section.clone();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        trace(
            "explorer-date",
            phase,
            object(&[
                ("trusted", JsValue::from_bool(event_trusted(&event))),
                ("value", property(&input_for_event, "value")),
            ]),
        );
        clear_table(
            &section_for_event,
            "Date range changed. Table cleared.",
            false,
        );
    }) as Box<dyn FnMut(JsValue)>);
    bind_property(input, "onchange", callback);
}

fn install_events(section: &JsValue) {
    let address_input = query(section, "#explorerAddress");
    let from_date_input = query(section, "#explorerFromDate");
    let to_date_input = query(section, "#explorerToDate");

    {
        let target = query(section, "#explorerFetch");
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            trace(
                "explorer-fetch",
                "r53b3-explorer-fetch-click",
                object(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "addressLength",
                        JsValue::from_f64(value(&section, "#explorerAddress").len() as f64),
                    ),
                    (
                        "fromDate",
                        JsValue::from_str(&value(&section, "#explorerFromDate")),
                    ),
                    (
                        "toDate",
                        JsValue::from_str(&value(&section, "#explorerToDate")),
                    ),
                ]),
            );
            let section = section.clone();
            spawn_local(async move {
                fetch_transactions(section, false).await;
            });
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }
    {
        let target = query(section, "#explorerForceFetch");
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            trace(
                "explorer-fetch",
                "r53b3-explorer-force-fetch-click",
                object(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "addressLength",
                        JsValue::from_f64(value(&section, "#explorerAddress").len() as f64),
                    ),
                    (
                        "fromDate",
                        JsValue::from_str(&value(&section, "#explorerFromDate")),
                    ),
                    (
                        "toDate",
                        JsValue::from_str(&value(&section, "#explorerToDate")),
                    ),
                ]),
            );
            let section = section.clone();
            spawn_local(async move {
                fetch_transactions(section, true).await;
            });
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }
    {
        let target = query(section, "#explorerOpenExplorer");
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            trace(
                "explorer-open",
                "r53b3-explorer-open-explorer-click",
                object(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "addressLength",
                        JsValue::from_f64(value(&section, "#explorerAddress").len() as f64),
                    ),
                ]),
            );
            crate::explorer_export::explorer_open_block_explorer(section.clone());
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }

    bind_address_change(
        &address_input,
        section,
        "oninput",
        "r53b3-explorer-address-input",
    );
    bind_address_change(
        &address_input,
        section,
        "onchange",
        "r53b3-explorer-address-change",
    );
    bind_date_change(&from_date_input, section, "r53b3-explorer-from-date-change");
    bind_date_change(&to_date_input, section, "r53b3-explorer-to-date-change");

    {
        let target = query(section, "#explorerCancel");
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            let section = section.clone();
            spawn_local(async move {
                trace(
                    "explorer-cancel",
                    "r53b3-explorer-cancel-click",
                    object(&[
                        ("trusted", JsValue::from_bool(event_trusted(&event))),
                        ("busyBefore", JsValue::from_bool(state_bool("busy"))),
                        (
                            "cancelBefore",
                            JsValue::from_bool(state_bool("cancelRequested")),
                        ),
                    ]),
                );
                set_state("cancelRequested", &JsValue::TRUE);
                let request_id = state_text("backendCancelRequestId");
                if !request_id.is_empty() {
                    trace(
                        "explorer-cancel",
                        "r57d4-explorer-backend-cancel-invoke",
                        object(&[
                            ("trusted", JsValue::from_bool(event_trusted(&event))),
                            (
                                "requestIdLength",
                                JsValue::from_f64(request_id.len() as f64),
                            ),
                        ]),
                    );
                    if let Err(error) =
                        crate::explorer_runtime::explorer_invoke_cancel_transactions(
                            JsValue::from_str(&request_id),
                        )
                        .await
                    {
                        trace(
                            "explorer-cancel",
                            "r57d4-explorer-backend-cancel-error",
                            object(&[("message", JsValue::from_str(&error_text(&error)))]),
                        );
                    }
                }
                clear_table(&section, "Cancel requested. Table cleared.", true);
            });
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }
    {
        let target = query(section, "#explorerApplyFilter");
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            trace(
                "explorer-filter",
                "r53b3-explorer-apply-filter-click",
                object(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "type",
                        JsValue::from_str(&{
                            let value = value(&section, "#explorerTypeFilter");
                            if value.is_empty() {
                                "ALL".to_owned()
                            } else {
                                value
                            }
                        }),
                    ),
                    (
                        "direction",
                        JsValue::from_str(&{
                            let value = value(&section, "#explorerDirectionFilter");
                            if value.is_empty() {
                                "ALL".to_owned()
                            } else {
                                value
                            }
                        }),
                    ),
                    (
                        "searchLength",
                        JsValue::from_f64(value(&section, "#explorerSearch").len() as f64),
                    ),
                ]),
            );
            let section = section.clone();
            spawn_local(async move {
                if let Err(error) = apply_explorer_filters_from_database(section.clone()).await {
                    crate::explorer_microscope::explorer_microscope_error(
                        "DB FILTER FAILED".to_owned(),
                        error.clone(),
                        Object::new().into(),
                    );
                    set_status(&section, &error_text(&error), "info");
                }
            });
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }
    {
        let target = query(section, "#explorerResetFilter");
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            trace(
                "explorer-filter",
                "r53b3-explorer-reset-filter-click",
                object(&[("trusted", JsValue::from_bool(event_trusted(&event)))]),
            );
            let _ = crate::explorer_render::explorer_reset_filters(section.clone());
            clear_table(&section, "Filters reset. Table cleared.", false);
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }
    for (selector, phase, format) in [
        (
            "#explorerExportCsv",
            "r53b3-explorer-export-csv-click",
            "csv",
        ),
        (
            "#explorerExportHtml",
            "r53b3-explorer-export-html-click",
            "html",
        ),
        (
            "#explorerExportPdf",
            "r53b3-explorer-export-pdf-click",
            "pdf",
        ),
    ] {
        let target = query(section, selector);
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            event_prevent(&event);
            trace(
                "explorer-export",
                phase,
                object(&[("trusted", JsValue::from_bool(event_trusted(&event)))]),
            );
            match format {
                "csv" => crate::explorer_export::explorer_export_csv(section.clone()),
                "html" => crate::explorer_export::explorer_export_html(section.clone()),
                _ => crate::explorer_export::explorer_export_pdf(section.clone()),
            }
        }) as Box<dyn FnMut(JsValue)>);
        bind_property(&target, "onclick", callback);
    }
}

async fn handle_live_page(event: JsValue) {
    let payload = {
        let candidate = property(&event, "payload");
        if candidate.is_object() && !candidate.is_null() {
            candidate
        } else {
            Object::new().into()
        }
    };
    let section = live_root();
    if !present(&section) || text(&property(&dataset(&section), "kgwFetchBusy")) != "true" {
        return;
    }
    let active_address = {
        let live = crate::explorer_live_core::explorer_live_core_address();
        if live.is_empty() {
            crate::explorer_addresses::explorer_normalize_address(JsValue::from_str(&value(
                &section,
                "#explorerAddress",
            )))
        } else {
            live
        }
    };
    let payload_address = text(&property(&payload, "address"));
    if !payload_address.is_empty()
        && !active_address.is_empty()
        && payload_address != active_address
    {
        return;
    }
    if text(&property(&payload, "phase")) != "page_stored" {
        return;
    }
    let days = property(&payload, "days");
    if Array::is_array(&days) && Array::from(&days).length() > 0 {
        crate::explorer_live_core::explorer_live_core_merge_days(days);
    } else {
        crate::explorer_live_core::explorer_live_core_merge_records(property(&payload, "records"));
    }
    let rows = crate::explorer_live_core::explorer_live_core_rows();
    let status = format!(
        "Fetch is running... page {}, stored {} transactions.",
        text(&property(&payload, "page")),
        format_count(&property(&payload, "stored_total"))
    );
    if !crate::explorer_live_core::explorer_live_core_should_render(payload.clone()) {
        set_status(&section, &status, "info");
        return;
    }
    let _ =
        crate::explorer_render::explorer_render_summaries(section.clone(), rows.clone(), status);
    let mode = if text(&property(&payload, "mode")) == "force" {
        "force"
    } else {
        "normal"
    };
    crate::explorer_force_ui::explorer_force_set_controls_busy(section, true, mode.to_owned());
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "log") {
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW Explorer][live-core] rendered page"),
            &object(&[
                ("mode", property(&payload, "mode")),
                ("page", property(&payload, "page")),
                ("pageStored", property(&payload, "page_stored")),
                ("storedTotal", property(&payload, "stored_total")),
                (
                    "compactDays",
                    JsValue::from_f64(if Array::is_array(&property(&payload, "days")) {
                        Array::from(&property(&payload, "days")).length() as f64
                    } else {
                        0.0
                    }),
                ),
                (
                    "records",
                    JsValue::from_f64(if Array::is_array(&property(&payload, "records")) {
                        Array::from(&property(&payload, "records")).length() as f64
                    } else {
                        0.0
                    }),
                ),
                (
                    "days",
                    JsValue::from_f64(if Array::is_array(&rows) {
                        Array::from(&rows).length() as f64
                    } else {
                        0.0
                    }),
                ),
            ]),
        );
    }
}

async fn install_tx_live_core_listener() -> Result<(), JsValue> {
    let win = window();
    if boolean(&property(&win, LIVE_LISTENER_FLAG)) {
        return Ok(());
    }
    let event_api = property(&property(&win, "__TAURI__"), "event");
    let Some(listen) = function(&event_api, "listen") else {
        let console = property(&global(), "console");
        if let Some(warn) = function(&console, "warn") {
            let _ = warn.call1(
                &console,
                &JsValue::from_str("[KGW Explorer][live-core] Tauri event listen API not found"),
            );
        }
        return Ok(());
    };
    set(&win, LIVE_LISTENER_FLAG, &JsValue::TRUE);
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        spawn_local(handle_live_page(event));
    }) as Box<dyn FnMut(JsValue)>);
    let result = listen.call2(
        &event_api,
        &JsValue::from_str(PAGE_STORED_EVENT),
        callback.as_ref().unchecked_ref(),
    )?;
    callback.forget();
    let _ = JsFuture::from(Promise::resolve(&result)).await?;
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "log") {
        let _ = log.call1(
            &console,
            &JsValue::from_str("[KGW Explorer][live-core] listener installed"),
        );
    }
    Ok(())
}

fn stop_if_cancelled(section: &JsValue, force: bool, stage: &str) -> bool {
    if !state_bool("cancelRequested") {
        return false;
    }
    trace(
        "explorer-fetch",
        "r56e-explorer-fetch-cancel-observed",
        object(&[
            ("force", JsValue::from_bool(force)),
            ("stage", JsValue::from_str(stage)),
        ]),
    );
    set_status(
        section,
        "Fetch cancelled. Existing local transaction data was preserved.",
        "info",
    );
    true
}

async fn fetch_body(
    section: JsValue,
    address: String,
    start_ts: JsValue,
    end_ts: JsValue,
    request_id: String,
    force: bool,
) -> Result<(), JsValue> {
    if force {
        crate::explorer_force_ui::explorer_force_reset_display_filters_to_all(section.clone());
        crate::explorer_force_ui::explorer_force_set_table_message(
            section.clone(),
            "Force refresh is running... staging replacement history while current local data remains available until verified promotion.".to_owned(),
        );
        set_status(
            &section,
            "Force refresh is running... current local history is preserved until the replacement is complete.",
            "info",
        );
    } else {
        set_status(&section, "Loading days from local database...", "info");
        let first_rows = crate::explorer_runtime::explorer_clean2_load_summaries(
            section.clone(),
            JsValue::from_str(&address),
            start_ts.clone(),
            end_ts.clone(),
        )
        .await?;
        if stop_if_cancelled(&section, force, "after-local-summary-load") {
            return Ok(());
        }
        let count = if Array::is_array(&first_rows) {
            Array::from(&first_rows).length()
        } else {
            0
        };
        let status = if count > 0 {
            format!(
                "{} days loaded from local database. Fetch is updating...",
                format_count(&JsValue::from_f64(count as f64))
            )
        } else {
            "No saved transaction days found yet. Fetch is running...".to_owned()
        };
        let _ =
            crate::explorer_render::explorer_render_summaries(section.clone(), first_rows, status);
        crate::explorer_live_core::explorer_live_core_seed_rows(address.clone(), rows_from_state());
        if stop_if_cancelled(&section, force, "after-initial-render") {
            return Ok(());
        }
        crate::explorer_force_ui::explorer_force_set_controls_busy(
            section.clone(),
            true,
            "normal".to_owned(),
        );
    }

    let _ = crate::explorer_addresses::explorer_fetch_balance(
        section.clone(),
        JsValue::from_str(&address),
    )
    .await;
    let _ = crate::explorer_addresses::explorer_refresh_address_name(
        section.clone(),
        JsValue::from_str(&address),
    )
    .await;
    let _ = crate::explorer_addresses::explorer_save_address_to_database(
        JsValue::from_str(&address),
        JsValue::from_str(""),
    )
    .await;

    if stop_if_cancelled(&section, force, "after-address-side-effects") {
        return Ok(());
    }

    let request = Object::new();
    set(request.as_ref(), "address", &JsValue::from_str(&address));
    set(request.as_ref(), "force", &JsValue::from_bool(force));
    set(request.as_ref(), "start_ts", &start_ts);
    set(request.as_ref(), "end_ts", &end_ts);
    set(
        request.as_ref(),
        "request_id",
        &JsValue::from_str(&request_id),
    );
    let tx_type = if force {
        "ALL".to_owned()
    } else {
        crate::explorer_filters::explorer_normalize_tx_type_filter_value(value(
            &section,
            "#explorerTypeFilter",
        ))
    };
    let direction = if force {
        "ALL".to_owned()
    } else {
        crate::explorer_filters::explorer_normalize_direction_filter_value(value(
            &section,
            "#explorerDirectionFilter",
        ))
    };
    let search = if force {
        String::new()
    } else {
        value(&section, "#explorerSearch")
    };
    set(request.as_ref(), "tx_type", &JsValue::from_str(&tx_type));
    set(
        request.as_ref(),
        "direction",
        &JsValue::from_str(&direction),
    );
    set(
        request.as_ref(),
        "search_query",
        &JsValue::from_str(&search),
    );

    let console = property(&global(), "console");
    if let Some(log) = function(&console, "log") {
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW Explorer][force-ui] fetch request"),
            request.as_ref(),
        );
    }

    let result = crate::explorer_runtime::explorer_invoke_unified_fetch(request.into()).await?;
    crate::explorer_runtime::explorer_clean2_log(
        "compact sync result received; table reloads bounded summaries from local database"
            .to_owned(),
        result,
    );

    if stop_if_cancelled(&section, force, "after-unified-fetch") {
        return Ok(());
    }

    let final_rows = crate::explorer_runtime::explorer_clean2_load_summaries(
        section.clone(),
        JsValue::from_str(&address),
        start_ts,
        end_ts,
    )
    .await?;
    if stop_if_cancelled(&section, force, "after-final-summary-load") {
        return Ok(());
    }
    let count = if Array::is_array(&final_rows) {
        Array::from(&final_rows).length()
    } else {
        0
    };
    let message = format!(
        "{} done. Showing {} days from local database. Click + to load one day.",
        if force { "Force fetch" } else { "Fetch" },
        format_count(&JsValue::from_f64(count as f64))
    );
    let _ = crate::explorer_render::explorer_render_summaries(section, final_rows, message);
    Ok(())
}

async fn fetch_transactions(section: JsValue, force: bool) {
    let section = crate::explorer_runtime::explorer_clean2_section(section);
    let initial = value(&section, "#explorerAddress");
    trace(
        "explorer-fetch",
        "r53b3-explorer-fetch-owner-begin",
        object(&[
            ("force", JsValue::from_bool(force)),
            ("addressLength", JsValue::from_f64(initial.len() as f64)),
        ]),
    );
    let Some(address) = canonical_address_or_error(&section, initial).await else {
        return;
    };
    let (start_ts, end_ts) = parse_range(&section);
    set_state("selectedAddress", &JsValue::from_str(&address));
    set_state("busy", &JsValue::TRUE);
    set_state("cancelRequested", &JsValue::FALSE);
    let request_id = format!(
        "explorer-{:.0}-{:016x}",
        js_sys::Date::now(),
        (js_sys::Math::random() * u64::MAX as f64) as u64
    );
    set_state("backendCancelRequestId", &JsValue::from_str(&request_id));

    set_global_fetch_busy(
        true,
        if force {
            "Force fetch is starting..."
        } else {
            "Fetch is starting..."
        },
    );
    reset_runtime_collections();
    let _ = install_tx_live_core_listener().await;
    crate::explorer_live_core::explorer_live_core_reset(address.clone());
    crate::explorer_force_ui::explorer_force_set_controls_busy(
        section.clone(),
        true,
        if force { "force" } else { "normal" }.to_owned(),
    );
    let _ = crate::explorer_render::explorer_sync_current_action_state(section.clone());

    let result = fetch_body(
        section.clone(),
        address.clone(),
        start_ts.clone(),
        end_ts.clone(),
        request_id.clone(),
        force,
    )
    .await;
    if let Err(error) = result {
        let message = error_text(&error);
        trace(
            "explorer-fetch",
            "r53b3-explorer-fetch-owner-error",
            object(&[
                ("force", JsValue::from_bool(force)),
                ("message", JsValue::from_str(&message)),
            ]),
        );
        let console = property(&global(), "console");
        if let Some(log) = function(&console, "error") {
            let _ = log.call2(
                &console,
                &JsValue::from_str("[KGW Explorer][force-ui] fetch failed"),
                &error,
            );
        }
        match crate::explorer_runtime::explorer_clean2_load_summaries(
            section.clone(),
            JsValue::from_str(&address),
            start_ts,
            end_ts,
        )
        .await
        {
            Ok(rows) => {
                let count = if Array::is_array(&rows) {
                    Array::from(&rows).length()
                } else {
                    0
                };
                let _ = crate::explorer_render::explorer_render_summaries(
                    section.clone(),
                    rows,
                    format!(
                        "{} failed. Showing {} days currently in local database.",
                        if force { "Force fetch" } else { "Fetch" },
                        format_count(&JsValue::from_f64(count as f64))
                    ),
                );
            }
            Err(_) => clear_table(&section, &message, false),
        }
    }

    if state_text("backendCancelRequestId") == request_id {
        set_state("backendCancelRequestId", &JsValue::from_str(""));
    }
    trace(
        "explorer-fetch",
        "r53b3-explorer-fetch-owner-finally",
        object(&[
            ("force", JsValue::from_bool(force)),
            (
                "cancelRequested",
                JsValue::from_bool(state_bool("cancelRequested")),
            ),
        ]),
    );
    set_state("busy", &JsValue::FALSE);
    set_global_fetch_busy(false, "Ready");
    crate::explorer_force_ui::explorer_force_set_controls_busy(
        section.clone(),
        false,
        String::new(),
    );
    let _ = crate::explorer_render::explorer_sync_current_action_state(section);
}

async fn apply_filter(section: JsValue) {
    let section = crate::explorer_runtime::explorer_clean2_section(section);
    let initial = {
        let selected = state_text("selectedAddress");
        if selected.is_empty() {
            value(&section, "#explorerAddress")
        } else {
            selected
        }
    };
    crate::explorer_runtime::explorer_clean2_log(
        "applyFilter start".to_owned(),
        object(&[
            ("address", JsValue::from_str(&initial)),
            (
                "type",
                JsValue::from_str(&{
                    let value = value(&section, "#explorerTypeFilter");
                    if value.is_empty() {
                        "ALL".to_owned()
                    } else {
                        value
                    }
                }),
            ),
            (
                "direction",
                JsValue::from_str(&{
                    let value = value(&section, "#explorerDirectionFilter");
                    if value.is_empty() {
                        "ALL".to_owned()
                    } else {
                        value
                    }
                }),
            ),
            (
                "search",
                JsValue::from_str(&value(&section, "#explorerSearch")),
            ),
        ]),
    );
    let Some(address) = canonical_address_or_error(&section, initial).await else {
        return;
    };
    let (start_ts, end_ts) = parse_range(&section);
    set_status(&section, "Applying filter from local database...", "info");
    reset_runtime_collections();
    match crate::explorer_runtime::explorer_clean2_load_summaries(
        section.clone(),
        JsValue::from_str(&address),
        start_ts,
        end_ts,
    )
    .await
    {
        Ok(rows) => {
            let count = if Array::is_array(&rows) {
                Array::from(&rows).length()
            } else {
                0
            };
            let status = if count > 0 {
                format!(
                    "Filter applied. Showing {} days from local database. Click + to load one day.",
                    format_count(&JsValue::from_f64(count as f64))
                )
            } else {
                "Filter applied. No matching transaction days found.".to_owned()
            };
            let _ = crate::explorer_render::explorer_render_summaries(section, rows, status);
        }
        Err(error) => {
            let console = property(&global(), "console");
            if let Some(log) = function(&console, "error") {
                let _ = log.call2(
                    &console,
                    &JsValue::from_str("[KGW Explorer][clean2] applyFilter failed"),
                    &error,
                );
            }
            clear_table(&section, &error_text(&error), false);
        }
    }
}

fn install_filter_capture() {
    let win = window();
    if boolean(&property(&win, FILTER_CAPTURE_FLAG)) {
        return;
    }
    set(&win, FILTER_CAPTURE_FLAG, &JsValue::TRUE);
    let doc = document();
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        let Ok(target) = call1(
            &target,
            "closest",
            &JsValue::from_str(
                "button,input[type='button'],input[type='submit'],[role='button'],.btn,.button",
            ),
        ) else {
            return;
        };
        if !present(&target) {
            return;
        }
        let section = live_root();
        if present(&section)
            && call1(&section, "contains", &target)
                .ok()
                .is_some_and(|value| !boolean(&value))
        {
            return;
        }
        let carrier = dataset(&target);
        let parts = [
            text(&property(&target, "id")),
            text(&property(&target, "name")),
            text(&property(&target, "value")),
            text(&property(&target, "textContent")),
            call1(&target, "getAttribute", &JsValue::from_str("aria-label"))
                .ok()
                .map(|value| text(&value))
                .unwrap_or_default(),
            text(&property(&carrier, "action")),
            text(&property(&carrier, "kgwAction")),
        ];
        let combined = parts
            .into_iter()
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        if !combined.contains("filter") || combined.contains("reset") {
            return;
        }
        stop_event(&event);
        crate::explorer_runtime::explorer_clean2_log(
            "captured filter click".to_owned(),
            object(&[
                ("id", property(&target, "id")),
                ("value", property(&target, "value")),
                (
                    "text",
                    JsValue::from_str(text(&property(&target, "textContent")).trim()),
                ),
            ]),
        );
        let section = if present(&section) { section } else { root() };
        spawn_local(apply_filter(section));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &doc,
        "addEventListener",
        &JsValue::from_str("click"),
        callback.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    callback.forget();
}

fn install_diagnostics_globals() {
    let win = window();
    let diagnostics = Object::new();

    let log = Closure::wrap(Box::new(move |label: JsValue, details: JsValue| {
        crate::explorer_microscope::explorer_microscope_log(text(&label), details);
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(diagnostics.as_ref(), "log", log.as_ref().unchecked_ref());
    log.forget();

    let warn = Closure::wrap(Box::new(move |label: JsValue, details: JsValue| {
        crate::explorer_microscope::explorer_microscope_warn(text(&label), details);
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(diagnostics.as_ref(), "warn", warn.as_ref().unchecked_ref());
    warn.forget();

    let error = Closure::wrap(Box::new(move |label: JsValue, details: JsValue| {
        let message = {
            let candidate = property(&details, "message");
            if boolean(&candidate) {
                text(&candidate)
            } else {
                let value = text(&details);
                if value.is_empty() {
                    "unknown error".to_owned()
                } else {
                    value
                }
            }
        };
        crate::explorer_microscope::explorer_microscope_error(
            text(&label),
            JsValue::from_str(&message),
            details,
        );
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(
        diagnostics.as_ref(),
        "error",
        error.as_ref().unchecked_ref(),
    );
    error.forget();

    let api = Closure::wrap(Box::new(move |label: JsValue, details: JsValue| {
        crate::explorer_microscope::explorer_microscope_api_shape(text(&label), details);
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(
        diagnostics.as_ref(),
        "apiShape",
        api.as_ref().unchecked_ref(),
    );
    api.forget();

    set(
        &win,
        "__kgwExplorerAddressDiagnostics",
        diagnostics.as_ref(),
    );

    let save = Closure::wrap(Box::new(move |section: JsValue| -> Promise {
        let section = if present(&section) { section } else { root() };
        future_to_promise(async move {
            Ok(JsValue::from_bool(
                crate::explorer_controls::explorer_save_manual_address(section).await,
            ))
        })
    }) as Box<dyn FnMut(JsValue) -> Promise>);
    set(
        &win,
        "kgwExplorerSaveManualAddress",
        save.as_ref().unchecked_ref(),
    );
    save.forget();

    let install = Closure::wrap(Box::new(move || -> bool {
        crate::explorer_controls::explorer_install_manual_address_save()
    }) as Box<dyn FnMut() -> bool>);
    set(
        &win,
        "kgwInstallExplorerManualAddressSave",
        install.as_ref().unchecked_ref(),
    );
    install.forget();
}

#[wasm_bindgen(js_name = explorerTabInstall)]
pub fn explorer_tab_install() {
    ensure_runtime_collections();
    crate::explorer_filters::explorer_install_filter_select_repair();
    crate::explorer_controls::explorer_install_filter_busy_lock();
    crate::explorer_render::explorer_install_price_rerender();
    crate::explorer_force_ui::explorer_install_force_busy_blocker();
    crate::explorer_export::explorer_export_install();
    install_filter_capture();
    install_diagnostics_globals();
    crate::explorer_controls::explorer_install_manual_address_save();
}

#[wasm_bindgen(js_name = explorerInitTab)]
pub async fn explorer_init_tab() -> Result<(), JsValue> {
    let section = root();
    crate::explorer_microscope::explorer_microscope_log(
        "INIT START".to_owned(),
        object(&[
            ("sectionId", property(&section, "id")),
            ("sectionClass", property(&section, "className")),
            (
                "sectionTextLength",
                JsValue::from_f64(text(&property(&section, "textContent")).len() as f64),
            ),
        ]),
    );
    crate::explorer_microscope::explorer_microscope_element_report(section.clone());
    crate::explorer_microscope::explorer_microscope_layout_report(section.clone());
    crate::explorer_microscope::explorer_microscope_current_state_report(
        "INIT STATE BEFORE".to_owned(),
    );

    let data = dataset(&section);
    if text(&property(&data, INIT_DATASET_KEY)) == "1" {
        let _ = crate::explorer_render::explorer_sync_current_action_state(section);
        return Ok(());
    }
    set(&data, INIT_DATASET_KEY, &JsValue::from_str("1"));

    crate::explorer_calendar::explorer_default_dates(section.clone());
    install_events(&section);
    crate::explorer_controls::explorer_set_table_font_size(section.clone());
    crate::explorer_controls::explorer_bind_font_spinbox(section.clone());
    let _ = crate::explorer_render::explorer_sync_current_action_state(section.clone());

    let _ = crate::explorer_addresses::explorer_load_known_address_names().await;
    let _ = crate::explorer_addresses::explorer_load_saved_addresses(section.clone()).await;

    let address = crate::explorer_addresses::explorer_normalize_address(JsValue::from_str(&value(
        &section,
        "#explorerAddress",
    )));
    if crate::explorer_controls::explorer_is_kaspa_address(JsValue::from_str(&address)) {
        let _ = crate::explorer_addresses::explorer_fetch_balance(
            section.clone(),
            JsValue::from_str(&address),
        )
        .await;
        let _ = crate::explorer_addresses::explorer_refresh_address_name(
            section.clone(),
            JsValue::from_str(&address),
        )
        .await;
    }

    let _ = crate::explorer_render::explorer_render_table(section.clone());
    crate::explorer_microscope::explorer_microscope_element_report(section.clone());
    crate::explorer_microscope::explorer_microscope_layout_report(section.clone());
    crate::explorer_microscope::explorer_microscope_current_state_report(
        "INIT STATE AFTER".to_owned(),
    );
    set_status(&section, "Ready", "info");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_contract_constants_are_stable() {
        assert_eq!(PAGE_STORED_EVENT, "kgw://transactions/page-stored");
        assert_eq!(INIT_DATASET_KEY, "kgwExplorerCleanInit");
        assert_eq!(LIVE_LISTENER_FLAG, "__kgwTxLiveCoreListenerInstalled");
        assert_eq!(FILTER_CAPTURE_FLAG, "__kgwClean2FilterCaptureInstalled");
    }

    #[test]
    fn count_format_matches_visible_status_shape() {
        assert_eq!(format_count_number(0.0), "0");
        assert_eq!(format_count_number(999.0), "999");
        assert_eq!(format_count_number(1000.0), "1,000");
        assert_eq!(format_count_number(1_234_567.0), "1,234,567");
    }
}
