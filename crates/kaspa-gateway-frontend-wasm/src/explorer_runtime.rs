use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;

const TRACE_PATCH: &str = "KGW_EXPLORER_SAFE_CONTROLS_TRACE_PATCH_R53B3";
const TRACE_OWNER: &str = "explorer-existing-safe-controls-owner";
const EXPORT_SELECTORS: [&str; 3] = [
    "#explorerExportCsv",
    "#explorerExportHtml",
    "#explorerExportPdf",
];

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
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
fn set(target: &JsValue, name: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(target, &JsValue::from_str(name), value).map(|_| ())
}

fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call1(target: &JsValue, name: &str, value: &JsValue) -> Result<JsValue, JsValue> {
    function(target, name)
        .ok_or_else(|| js_error(format!("missing JS method {name}")))?
        .call1(target, value)
}

fn call2(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    function(target, name)
        .ok_or_else(|| js_error(format!("missing JS method {name}")))?
        .call2(target, first, second)
}

fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}

fn query(scope: &JsValue, selector: &str) -> JsValue {
    call1(scope, "querySelector", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}
fn required_query(scope: &JsValue, selector: &str) -> Result<JsValue, JsValue> {
    let node = query(scope, selector);
    if present(&node) {
        Ok(node)
    } else {
        Err(js_error(format!(
            "Explorer required element was not found: {selector}"
        )))
    }
}

fn invoke_api() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    let core = property(&tauri, "core");
    let legacy = property(&tauri, "tauri");
    for candidate in [
        property(&core, "invoke"),
        property(&legacy, "invoke"),
        property(&win, "__TAURI_INVOKE__"),
    ] {
        if let Ok(function) = candidate.dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}

async fn invoke_command(command: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let invoke = invoke_api().ok_or_else(|| js_error("Tauri invoke API is not available."))?;
    let value = invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?;
    JsFuture::from(Promise::resolve(&value)).await
}
fn request_args(request: &JsValue) -> JsValue {
    let args = Object::new();
    let _ = set(args.as_ref(), "request", request);
    args.into()
}

fn defaulted_text(value: &JsValue, fallback: &str) -> String {
    if crate::js_boolean(value) {
        crate::js_string_owned(value)
    } else {
        fallback.to_owned()
    }
}

fn details_object(value: &JsValue) -> JsValue {
    if crate::js_boolean(value) && value.is_object() {
        value.clone()
    } else {
        Object::new().into()
    }
}

fn stringify(value: &JsValue) -> Option<String> {
    JSON::stringify(value)
        .ok()
        .map(|text| crate::js_string_owned(text.as_ref()))
}

fn console_method(name: &str) -> Option<(JsValue, Function)> {
    let console = property(&global(), "console");
    function(&console, name).map(|method| (console, method))
}

fn set_disabled(node: &JsValue, disabled: bool) -> Result<(), JsValue> {
    set(node, "disabled", &JsValue::from_bool(disabled))
}
fn set_status_impl(section: &JsValue, message: &JsValue, state: &JsValue) -> bool {
    let clean_message = if crate::js_boolean(message) {
        crate::js_string_owned(message)
    } else {
        String::new()
    };
    let state_text = crate::js_string_owned(state);
    let node = query(section, "#explorerStatus");
    if present(&node) {
        let _ = set(&node, "hidden", &JsValue::FALSE);
        let _ = set(&node, "textContent", &JsValue::from_str(&clean_message));
        let dataset = property(&node, "dataset");
        let _ = set(&dataset, "state", &JsValue::from_str(&state_text));
        let role = if state_text == "error" {
            "alert"
        } else {
            "status"
        };
        let live = if state_text == "error" {
            "assertive"
        } else {
            "polite"
        };
        let _ = call2(
            &node,
            "setAttribute",
            &JsValue::from_str("role"),
            &JsValue::from_str(role),
        );
        let _ = call2(
            &node,
            "setAttribute",
            &JsValue::from_str("aria-live"),
            &JsValue::from_str(live),
        );
    }
    let win = window();
    if let Some(progress) = function(&win, "kgwSetGlobalFetchProgressText") {
        let _ = progress.call1(&win, &JsValue::from_str(&clean_message));
    }
    if let Some((console, info)) = console_method("info") {
        let _ = info.call2(&console, &JsValue::from_str("[KGW Explorer]"), message);
    }
    present(&node)
}

fn sync_action_state_impl(
    section: &JsValue,
    busy: bool,
    rows_count: u32,
    filtered_rows_count: u32,
) -> Result<(), JsValue> {
    let _ = crate::explorer_controls::explorer_apply_local_busy_controls(section.clone(), busy);
    set_disabled(&required_query(section, "#explorerFetch")?, busy)?;
    set_disabled(&required_query(section, "#explorerForceFetch")?, busy)?;
    set_disabled(&required_query(section, "#explorerCancel")?, !busy)?;

    let details = Object::new();
    set(details.as_ref(), "busy", &JsValue::from_bool(busy))?;
    set(
        details.as_ref(),
        "rows",
        &JsValue::from_f64(rows_count as f64),
    )?;
    set(
        details.as_ref(),
        "filteredRows",
        &JsValue::from_f64(filtered_rows_count as f64),
    )?;
    crate::explorer_microscope::explorer_microscope_log(
        "SYNC ACTION STATE".to_owned(),
        details.into(),
    );

    let disable_exports = filtered_rows_count == 0;
    for selector in EXPORT_SELECTORS {
        let node = query(section, selector);
        if present(&node) {
            set_disabled(&node, disable_exports)?;
        }
    }
    Ok(())
}

fn extract_rows_impl(result: JsValue) -> Result<JsValue, JsValue> {
    crate::explorer_microscope::explorer_microscope_api_shape(
        "UNIFIED RESULT RAW SHAPE".to_owned(),
        result.clone(),
    );
    let rows = crate::explorer_results::explorer_normalize_unified_result(result)?;
    let array = Array::from(&rows);
    let details = Object::new();
    set(
        details.as_ref(),
        "rows",
        &JsValue::from_f64(array.length() as f64),
    )?;
    let sample = array.slice(0, 3);
    set(details.as_ref(), "sample", sample.as_ref())?;
    crate::explorer_microscope::explorer_microscope_log(
        "UNIFIED RESULT NORMALIZED ROWS".to_owned(),
        details.into(),
    );
    Ok(rows)
}

async fn ui_trace_impl(action: JsValue, phase: JsValue, details: JsValue) -> JsValue {
    let Some(invoke) = invoke_api() else {
        return JsValue::FALSE;
    };
    let action_text = defaulted_text(&action, "explorer-ui");
    let phase_text = defaulted_text(&phase, "unknown");
    let nested = Object::new();
    let _ = set(nested.as_ref(), "patch", &JsValue::from_str(TRACE_PATCH));
    let _ = set(nested.as_ref(), "owner", &JsValue::from_str(TRACE_OWNER));
    let _ = set(nested.as_ref(), "action", &JsValue::from_str(&action_text));
    let _ = set(nested.as_ref(), "phase", &JsValue::from_str(&phase_text));
    let _ = set(nested.as_ref(), "details", &details_object(&details));
    let Some(serialized) = stringify(nested.as_ref()) else {
        return JsValue::FALSE;
    };
    let args = Object::new();
    let _ = set(args.as_ref(), "scope", &JsValue::from_str("explorer"));
    let _ = set(args.as_ref(), "net", &JsValue::from_str("ui"));
    let _ = set(args.as_ref(), "action", &JsValue::from_str(&action_text));
    let _ = set(args.as_ref(), "phase", &JsValue::from_str(&phase_text));
    let _ = set(args.as_ref(), "details", &JsValue::from_str(&serialized));
    let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) else {
        return JsValue::FALSE;
    };
    match JsFuture::from(Promise::resolve(&result)).await {
        Ok(value) => value,
        Err(_) => JsValue::UNDEFINED,
    }
}
fn filter_trace_impl(label: &JsValue, payload: &JsValue) {
    let text = format!("[KGW Explorer][filter] {}", crate::js_string_owned(label));
    let Some((console, log)) = console_method("log") else {
        return;
    };
    if log
        .call2(&console, &JsValue::from_str(&text), payload)
        .is_err()
    {
        let _ = log.call1(&console, &JsValue::from_str(&text));
    }
}

#[wasm_bindgen(js_name = explorerInvokeUnifiedFetch)]
pub async fn explorer_invoke_unified_fetch(request: JsValue) -> Result<JsValue, JsValue> {
    invoke_command("explorer_transactions", &request_args(&request)).await
}

#[wasm_bindgen(js_name = explorerInvokeCancelTransactions)]
pub async fn explorer_invoke_cancel_transactions(request_id: JsValue) -> Result<JsValue, JsValue> {
    let args = Object::new();
    set(args.as_ref(), "requestId", &request_id)?;
    invoke_command("explorer_cancel_transactions", args.as_ref()).await
}

#[wasm_bindgen(js_name = explorerInvokeGroupedTransactions)]
pub async fn explorer_invoke_grouped_transactions(request: JsValue) -> Result<JsValue, JsValue> {
    invoke_command(
        "explorer_list_transactions_grouped_rust",
        &request_args(&request),
    )
    .await
}
#[wasm_bindgen(js_name = explorerInvokeDaySummaries)]
pub async fn explorer_invoke_day_summaries(request: JsValue) -> Result<JsValue, JsValue> {
    invoke_command(
        "explorer_transaction_day_summaries_rust",
        &request_args(&request),
    )
    .await
}

#[wasm_bindgen(js_name = explorerSetStatus)]
pub fn explorer_set_status(section: JsValue, message: JsValue, state: JsValue) -> bool {
    set_status_impl(&section, &message, &state)
}

#[wasm_bindgen(js_name = explorerSyncActionState)]
pub fn explorer_sync_action_state(
    section: JsValue,
    busy: bool,
    rows_count: u32,
    filtered_rows_count: u32,
) -> Result<(), JsValue> {
    sync_action_state_impl(&section, busy, rows_count, filtered_rows_count)
}

#[wasm_bindgen(js_name = explorerExtractRowsFromUnifiedResult)]
pub fn explorer_extract_rows_from_unified_result(result: JsValue) -> Result<JsValue, JsValue> {
    extract_rows_impl(result)
}
#[wasm_bindgen(js_name = explorerUiTrace)]
pub async fn explorer_ui_trace(action: JsValue, phase: JsValue, details: JsValue) -> JsValue {
    ui_trace_impl(action, phase, details).await
}

#[wasm_bindgen(js_name = explorerFilterTrace)]
pub fn explorer_filter_trace(label: JsValue, payload: JsValue) {
    filter_trace_impl(&label, &payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_identity_is_stable() {
        assert_eq!(TRACE_PATCH, "KGW_EXPLORER_SAFE_CONTROLS_TRACE_PATCH_R53B3");
        assert_eq!(TRACE_OWNER, "explorer-existing-safe-controls-owner");
    }

    #[test]
    fn export_selector_contract_is_exact() {
        assert_eq!(
            EXPORT_SELECTORS,
            [
                "#explorerExportCsv",
                "#explorerExportHtml",
                "#explorerExportPdf",
            ]
        );
    }
}
