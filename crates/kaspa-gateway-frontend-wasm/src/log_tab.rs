use super::{call_method0, call_method1, js_boolean, js_string, method, set_property};
use js_sys::{Array, Function, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

const LOG_STORAGE_KEY: &str = "kgw-log-viewer-state";

thread_local! {
    static LOG_LINES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static FILTERED_LINES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static POLL_TIMER: RefCell<Option<JsValue>> = const { RefCell::new(None) };
    static COPY_STATUS_TIMER: RefCell<Option<JsValue>> = const { RefCell::new(None) };
}

fn raw_string(value: &JsValue) -> String {
    String::from(js_string(value))
}

fn optional_property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn truthy_text(value: &JsValue) -> String {
    if js_boolean(value) {
        raw_string(value)
    } else {
        String::new()
    }
}

fn has_own(target: &JsValue, name: &str) -> bool {
    if target.is_null() || target.is_undefined() {
        return false;
    }
    let object: Object = target.clone().unchecked_into();
    Object::has_own_property(&object, &JsValue::from_str(name))
}

fn call_method2(
    target: &JsValue,
    name: &str,
    first: &JsValue,
    second: &JsValue,
) -> Result<JsValue, JsValue> {
    method(target, name)?.call2(target, first, second)
}

fn global() -> JsValue {
    js_sys::global().into()
}

fn document() -> JsValue {
    optional_property(&global(), "document")
}

fn window() -> JsValue {
    optional_property(&global(), "window")
}

fn query_selector(target: &JsValue, selector: &str) -> JsValue {
    call_method1(target, "querySelector", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED)
}

fn root() -> JsValue {
    let document = document();
    let candidate = call_method1(&document, "getElementById", &JsValue::from_str("log"))
        .unwrap_or(JsValue::UNDEFINED);
    if js_boolean(&candidate) {
        candidate
    } else {
        document
    }
}

fn qs(selector: &str) -> JsValue {
    let local = query_selector(&root(), selector);
    if js_boolean(&local) {
        local
    } else {
        query_selector(&document(), selector)
    }
}

fn current_ui_language() -> &'static str {
    let document = document();
    let mut selector = query_selector(&document, "#languageSelect");
    if !js_boolean(&selector) {
        selector = query_selector(&document, "#language");
    }
    if !js_boolean(&selector) {
        selector = query_selector(&document, "select[name='language']");
    }

    let selected_options = optional_property(&selector, "selectedOptions");
    let selected =
        Reflect::get(&selected_options, &JsValue::from_f64(0.0)).unwrap_or(JsValue::UNDEFINED);
    let selected_text = truthy_text(&optional_property(&selected, "textContent"))
        .trim()
        .to_lowercase();
    let selected_value = truthy_text(&optional_property(&selector, "value"))
        .trim()
        .to_lowercase();
    let document_element = optional_property(&document, "documentElement");
    let html_lang = truthy_text(&optional_property(&document_element, "lang"))
        .trim()
        .to_lowercase();

    let signal = format!("{selected_value} {selected_text} {html_lang}");
    if signal == "ar"
        || signal.contains(" ar ")
        || signal.contains("arabic")
        || signal.contains("العربية")
    {
        "ar"
    } else {
        "en"
    }
}

fn copy_text_for_language(key: &str, lang: &str) -> &'static str {
    match (key, lang) {
        ("copiedButton", "ar") => "تم النسخ",
        ("copiedStatus", "ar") => "تم النسخ إلى الحافظة.",
        ("copyFailedButton", "ar") => "فشل النسخ",
        ("copyFailedStatus", "ar") => "فشل النسخ. الوصول إلى الحافظة غير متاح أو تم رفضه.",
        ("copiedButton", _) => "Copied",
        ("copiedStatus", _) => "Copied to clipboard.",
        ("copyFailedButton", _) => "Copy failed",
        ("copyFailedStatus", _) => "Copy failed. Clipboard access is unavailable or was rejected.",
        _ => "",
    }
}

fn copy_text(key: &str) -> &'static str {
    copy_text_for_language(key, current_ui_language())
}

fn invoke_function() -> Option<Function> {
    let tauri = optional_property(&window(), "__TAURI__");
    for owner_name in ["core", "tauri"] {
        let owner = optional_property(&tauri, owner_name);
        let invoke = optional_property(&owner, "invoke");
        if let Some(function) = invoke.dyn_ref::<Function>() {
            return Some(function.clone());
        }
    }
    None
}

fn local_storage() -> JsValue {
    optional_property(&global(), "localStorage")
}

fn load_state() -> JsValue {
    let storage = local_storage();
    let raw = call_method1(&storage, "getItem", &JsValue::from_str(LOG_STORAGE_KEY))
        .unwrap_or(JsValue::UNDEFINED);
    let source = if js_boolean(&raw) {
        raw_string(&raw)
    } else {
        "{}".to_owned()
    };
    JSON::parse(&source).unwrap_or_else(|_| Object::new().into())
}

fn field_value(selector: &str, fallback: &str) -> String {
    let field = qs(selector);
    let value = optional_property(&field, "value");
    if js_boolean(&value) {
        raw_string(&value)
    } else {
        fallback.to_owned()
    }
}

fn field_checked(selector: &str) -> bool {
    optional_property(&qs(selector), "checked")
        .as_bool()
        .unwrap_or(false)
}

fn save_state() -> Result<(), JsValue> {
    let state = Object::new();
    set_property(
        state.as_ref(),
        "severity",
        &JsValue::from_str(&field_value("#logSeverity", "ALL")),
    )?;
    set_property(
        state.as_ref(),
        "search",
        &JsValue::from_str(&field_value("#logSearch", "")),
    )?;
    set_property(
        state.as_ref(),
        "autoScroll",
        &JsValue::from_bool(field_checked("#logAutoScroll")),
    )?;
    set_property(
        state.as_ref(),
        "fontSize",
        &JsValue::from_str(&field_value("#logFontSize", "9")),
    )?;
    let encoded = JSON::stringify(state.as_ref())?;
    let _ = call_method2(
        &local_storage(),
        "setItem",
        &JsValue::from_str(LOG_STORAGE_KEY),
        encoded.as_ref(),
    )?;
    Ok(())
}

fn is_ascii_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn parse_structured_level(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let needle = b"level=\"";
    let mut start = 0usize;

    while start + needle.len() <= bytes.len() {
        let relative = lower[start..].find("level=\"")?;
        let index = start + relative;
        if index > 0 && !is_ascii_space(bytes[index - 1]) {
            start = index + 1;
            continue;
        }
        let value_start = index + needle.len();
        let quote = lower[value_start..].find('"')? + value_start;
        let level = &lower[value_start..quote];
        let valid = matches!(level, "trace" | "debug" | "info" | "warn" | "error");
        let boundary =
            quote + 1 == bytes.len() || bytes.get(quote + 1).copied().is_some_and(is_ascii_space);
        if valid && boundary {
            return Some(level.to_ascii_uppercase());
        }
        start = index + 1;
    }
    None
}

fn take_digits(bytes: &[u8], index: &mut usize, count: usize) -> bool {
    if *index + count > bytes.len() || !bytes[*index..*index + count].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    *index += count;
    true
}

fn take_byte(bytes: &[u8], index: &mut usize, byte: u8) -> bool {
    if bytes.get(*index).copied() != Some(byte) {
        return false;
    }
    *index += 1;
    true
}

fn take_spaces(bytes: &[u8], index: &mut usize) -> bool {
    let start = *index;
    while bytes.get(*index).copied().is_some_and(is_ascii_space) {
        *index += 1;
    }
    *index > start
}

fn parse_native_level(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    if !take_digits(bytes, &mut index, 4)
        || !take_byte(bytes, &mut index, b'-')
        || !take_digits(bytes, &mut index, 2)
        || !take_byte(bytes, &mut index, b'-')
        || !take_digits(bytes, &mut index, 2)
        || !take_spaces(bytes, &mut index)
        || !take_digits(bytes, &mut index, 2)
        || !take_byte(bytes, &mut index, b':')
        || !take_digits(bytes, &mut index, 2)
        || !take_byte(bytes, &mut index, b':')
        || !take_digits(bytes, &mut index, 2)
    {
        return None;
    }
    if bytes
        .get(index)
        .copied()
        .is_some_and(|byte| matches!(byte, b'.' | b','))
    {
        index += 1;
        let before = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index == before {
            return None;
        }
    }
    if !take_spaces(bytes, &mut index)
        || !take_byte(bytes, &mut index, b'-')
        || !take_spaces(bytes, &mut index)
    {
        return None;
    }
    let level_start = index;
    while bytes.get(index).is_some_and(u8::is_ascii_alphabetic) {
        index += 1;
    }
    let level = text[level_start..index].to_ascii_uppercase();
    if !matches!(
        level.as_str(),
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR"
    ) || !take_spaces(bytes, &mut index)
        || !take_byte(bytes, &mut index, b'-')
        || !take_spaces(bytes, &mut index)
        || !take_byte(bytes, &mut index, b'[')
    {
        return None;
    }
    let thread_start = index;
    while bytes.get(index).copied().is_some_and(|byte| byte != b']') {
        index += 1;
    }
    if index == thread_start
        || !take_byte(bytes, &mut index, b']')
        || !take_spaces(bytes, &mut index)
        || !take_byte(bytes, &mut index, b'-')
        || !take_spaces(bytes, &mut index)
    {
        return None;
    }
    Some(level)
}

fn parse_log_level_text(text: &str) -> String {
    parse_structured_level(text)
        .or_else(|| parse_native_level(text))
        .unwrap_or_else(|| "UNKNOWN".to_owned())
}

#[wasm_bindgen(js_name = kgwLogParseLevel)]
pub fn parse_log_level(line: JsValue) -> String {
    let source = if js_boolean(&line) {
        raw_string(&line)
    } else {
        String::new()
    };
    parse_log_level_text(&source)
}

fn severity_rank(level: &str) -> u8 {
    match level.to_ascii_uppercase().as_str() {
        "TRACE" => 0,
        "DEBUG" => 1,
        "INFO" => 2,
        "WARN" => 3,
        "ERROR" => 4,
        _ => 2,
    }
}

fn should_show_text(line: &str, severity: &str, search: &str) -> bool {
    let level = parse_log_level_text(line);
    if severity != "ALL" && (level == "UNKNOWN" || severity_rank(&level) < severity_rank(severity))
    {
        return false;
    }
    let search = search.trim().to_lowercase();
    search.is_empty() || line.to_lowercase().contains(&search)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn highlight_line(line: &str) -> String {
    // Preserve the legacy ordering exactly. Its attribute-highlighting expressions
    // operate after ampersand escaping, so raw structured log quotes remain plain.
    escape_html(line)
}

fn render() -> Result<(), JsValue> {
    let output = qs("#logOutput");
    if !js_boolean(&output) {
        return Ok(());
    }
    let severity = field_value("#logSeverity", "ALL");
    let search = field_value("#logSearch", "");

    let filtered = LOG_LINES.with(|lines| {
        lines
            .borrow()
            .iter()
            .filter(|line| should_show_text(line, &severity, &search))
            .cloned()
            .collect::<Vec<_>>()
    });
    let rendered = filtered
        .iter()
        .map(|line| highlight_line(line))
        .collect::<Vec<_>>()
        .join("\n");
    FILTERED_LINES.with(|lines| *lines.borrow_mut() = filtered);

    set_property(&output, "innerHTML", &JsValue::from_str(&rendered))?;
    let font_size = field_value("#logFontSize", "9");
    let style = optional_property(&output, "style");
    set_property(
        &style,
        "fontSize",
        &JsValue::from_str(&format!("{font_size}px")),
    )?;

    if field_checked("#logAutoScroll") {
        let height = optional_property(&output, "scrollHeight");
        set_property(&output, "scrollTop", &height)?;
    }

    save_state()
}

async fn await_value(value: JsValue) -> Result<JsValue, JsValue> {
    if value.is_instance_of::<Promise>() {
        JsFuture::from(value.unchecked_into::<Promise>()).await
    } else {
        Ok(value)
    }
}

fn error_text(error: &JsValue) -> String {
    let message = optional_property(error, "message");
    if js_boolean(&message) {
        raw_string(&message)
    } else {
        raw_string(error)
    }
}

async fn refresh_log() -> Result<(), JsValue> {
    let output = qs("#logOutput");
    let Some(call) = invoke_function() else {
        if js_boolean(&output) {
            let translate = optional_property(&window(), "kgwT");
            let text = if let Some(function) = translate.dyn_ref::<Function>() {
                function
                    .call1(
                        &window(),
                        &JsValue::from_str("log.tauriInvokeApiUnavailable"),
                    )
                    .map(|value| raw_string(&value))
                    .unwrap_or_else(|_| "Tauri invoke API is not available.".to_owned())
            } else {
                "Tauri invoke API is not available.".to_owned()
            };
            set_property(&output, "textContent", &JsValue::from_str(&text))?;
        }
        return Ok(());
    };

    let args = Object::new();
    set_property(args.as_ref(), "maxLines", &JsValue::from_f64(5000.0))?;
    let result = call.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_log_read"),
        args.as_ref(),
    );
    match result {
        Ok(value) => match await_value(value).await {
            Ok(value) => {
                let array = Array::from(&value);
                let lines = array
                    .iter()
                    .map(|value| raw_string(&value))
                    .collect::<Vec<_>>();
                LOG_LINES.with(|stored| *stored.borrow_mut() = lines);
                render()
            }
            Err(error) => {
                if js_boolean(&output) {
                    set_property(
                        &output,
                        "textContent",
                        &JsValue::from_str(&format!("Failed to read log: {}", error_text(&error))),
                    )?;
                }
                Ok(())
            }
        },
        Err(error) => {
            if js_boolean(&output) {
                set_property(
                    &output,
                    "textContent",
                    &JsValue::from_str(&format!("Failed to read log: {}", error_text(&error))),
                )?;
            }
            Ok(())
        }
    }
}

async fn clear_log() -> Result<(), JsValue> {
    let Some(call) = invoke_function() else {
        return Ok(());
    };
    let value = call.call1(&JsValue::UNDEFINED, &JsValue::from_str("kgw_log_clear"))?;
    let _ = await_value(value).await?;
    refresh_log().await
}

fn show_copy_status(message: &str, success: bool) -> Result<(), JsValue> {
    let mut node = qs("#logCopyStatus");
    if !js_boolean(&node) {
        let document = document();
        node = call_method1(&document, "createElement", &JsValue::from_str("span"))?;
        set_property(&node, "id", &JsValue::from_str("logCopyStatus"))?;
        set_property(&node, "className", &JsValue::from_str("log-copy-status"))?;
        let _ = call_method2(
            &node,
            "setAttribute",
            &JsValue::from_str("role"),
            &JsValue::from_str("status"),
        )?;
        let _ = call_method2(
            &node,
            "setAttribute",
            &JsValue::from_str("aria-live"),
            &JsValue::from_str("polite"),
        )?;
        let copy = qs("#logCopy");
        let parent = optional_property(&copy, "parentElement");
        let host = if js_boolean(&parent) { parent } else { root() };
        let _ = call_method1(&host, "appendChild", &node)?;
    }

    set_property(&node, "hidden", &JsValue::FALSE)?;
    set_property(&node, "textContent", &JsValue::from_str(message))?;

    let button = qs("#logCopy");
    if js_boolean(&button) {
        let dataset = optional_property(&button, "dataset");
        let original = optional_property(&dataset, "originalText");
        let old_text = if js_boolean(&original) {
            raw_string(&original)
        } else {
            truthy_text(&optional_property(&button, "textContent"))
        };
        if !js_boolean(&original) {
            set_property(&dataset, "originalText", &JsValue::from_str(&old_text))?;
        }
        set_property(
            &button,
            "textContent",
            &JsValue::from_str(copy_text(if success {
                "copiedButton"
            } else {
                "copyFailedButton"
            })),
        )?;
    }

    let global = global();
    COPY_STATUS_TIMER.with(|timer| {
        if let Some(id) = timer.borrow_mut().take()
            && let Some(clear) = optional_property(&global, "clearTimeout").dyn_ref::<Function>()
        {
            let _ = clear.call1(&global, &id);
        }
    });

    let node_for_timeout = node.clone();
    let button_for_timeout = button.clone();
    let callback = Closure::wrap(Box::new(move || {
        let _ = set_property(&node_for_timeout, "textContent", &JsValue::from_str(""));
        let _ = set_property(&node_for_timeout, "hidden", &JsValue::TRUE);
        if js_boolean(&button_for_timeout) {
            let dataset = optional_property(&button_for_timeout, "dataset");
            let original = optional_property(&dataset, "originalText");
            if js_boolean(&original) {
                let _ = set_property(&button_for_timeout, "textContent", &original);
            }
        }
    }) as Box<dyn FnMut()>);

    if let Some(set_timeout) = optional_property(&global, "setTimeout").dyn_ref::<Function>() {
        let id = set_timeout.call2(
            &global,
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(2200.0),
        )?;
        COPY_STATUS_TIMER.with(|timer| *timer.borrow_mut() = Some(id));
        callback.forget();
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardApiState {
    Unavailable,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardFallbackState {
    Unavailable,
    Succeeded,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardDecision {
    Succeeded,
    TryFallback,
    PreserveApiError,
    Unavailable,
    FallbackRejected,
}

fn clipboard_after_api(api: ClipboardApiState) -> ClipboardDecision {
    match api {
        ClipboardApiState::Succeeded => ClipboardDecision::Succeeded,
        ClipboardApiState::Unavailable | ClipboardApiState::Failed => {
            ClipboardDecision::TryFallback
        }
    }
}

fn clipboard_after_fallback(
    api: ClipboardApiState,
    fallback: ClipboardFallbackState,
) -> ClipboardDecision {
    match fallback {
        ClipboardFallbackState::Succeeded => ClipboardDecision::Succeeded,
        ClipboardFallbackState::Rejected => ClipboardDecision::FallbackRejected,
        ClipboardFallbackState::Unavailable => match api {
            ClipboardApiState::Failed => ClipboardDecision::PreserveApiError,
            ClipboardApiState::Unavailable => ClipboardDecision::Unavailable,
            ClipboardApiState::Succeeded => ClipboardDecision::Succeeded,
        },
    }
}

#[wasm_bindgen(js_name = kgwLogCopyTextToClipboard)]
pub async fn copy_text_to_clipboard(text: JsValue, env: JsValue) -> Result<bool, JsValue> {
    let clipboard = if has_own(&env, "clipboard") {
        optional_property(&env, "clipboard")
    } else {
        let navigator = optional_property(&global(), "navigator");
        optional_property(&navigator, "clipboard")
    };
    let text = raw_string(&text);
    let mut api_error = None;
    let mut api_state = ClipboardApiState::Unavailable;

    if js_boolean(&clipboard) {
        let write_text = optional_property(&clipboard, "writeText");
        if let Some(function) = write_text.dyn_ref::<Function>() {
            match function.call1(&clipboard, &JsValue::from_str(&text)) {
                Ok(value) => match await_value(value).await {
                    Ok(_) => api_state = ClipboardApiState::Succeeded,
                    Err(error) => {
                        api_state = ClipboardApiState::Failed;
                        api_error = Some(error);
                    }
                },
                Err(error) => {
                    api_state = ClipboardApiState::Failed;
                    api_error = Some(error);
                }
            }
        }
    }
    if clipboard_after_api(api_state) == ClipboardDecision::Succeeded {
        return Ok(true);
    }

    let document = if has_own(&env, "document") {
        optional_property(&env, "document")
    } else {
        document()
    };
    let create_element = optional_property(&document, "createElement");
    let exec_command = optional_property(&document, "execCommand");
    let body = optional_property(&document, "body");
    if !js_boolean(&document)
        || create_element.dyn_ref::<Function>().is_none()
        || exec_command.dyn_ref::<Function>().is_none()
        || !js_boolean(&body)
    {
        return Err(
            match clipboard_after_fallback(api_state, ClipboardFallbackState::Unavailable) {
                ClipboardDecision::PreserveApiError => {
                    api_error.unwrap_or_else(|| JsValue::from_str("Clipboard copy is unavailable."))
                }
                ClipboardDecision::Unavailable => {
                    JsValue::from_str("Clipboard copy is unavailable.")
                }
                _ => JsValue::from_str("Clipboard copy is unavailable."),
            },
        );
    }

    let textarea = create_element
        .unchecked_ref::<Function>()
        .call1(&document, &JsValue::from_str("textarea"))?;
    set_property(&textarea, "value", &JsValue::from_str(&text))?;
    let _ = call_method1(&body, "appendChild", &textarea)?;

    let result = (|| -> Result<bool, JsValue> {
        let _ = call_method0(&textarea, "select")?;
        let copied = exec_command
            .unchecked_ref::<Function>()
            .call1(&document, &JsValue::from_str("copy"))?;
        match clipboard_after_fallback(
            api_state,
            if copied.as_bool() == Some(true) {
                ClipboardFallbackState::Succeeded
            } else {
                ClipboardFallbackState::Rejected
            },
        ) {
            ClipboardDecision::Succeeded => Ok(true),
            ClipboardDecision::FallbackRejected => {
                Err(JsValue::from_str("Clipboard fallback was rejected."))
            }
            _ => Err(JsValue::from_str("Clipboard copy is unavailable.")),
        }
    })();
    let _ = call_method0(&textarea, "remove");
    result
}

async fn copy_log() -> Result<bool, JsValue> {
    let text = FILTERED_LINES.with(|lines| lines.borrow().join("\n"));
    match copy_text_to_clipboard(text.into(), Object::new().into()).await {
        Ok(true) => {
            show_copy_status(copy_text("copiedStatus"), true)?;
            Ok(true)
        }
        Ok(false) => {
            show_copy_status(copy_text("copyFailedStatus"), false)?;
            Ok(false)
        }
        Err(error) => {
            let _ = show_copy_status(copy_text("copyFailedStatus"), false);
            Err(error)
        }
    }
}

fn set_if_present(selector: &str, property_name: &str, value: &JsValue) -> Result<(), JsValue> {
    let field = qs(selector);
    if js_boolean(&field) {
        set_property(&field, property_name, value)?;
    }
    Ok(())
}

fn apply_initial_state() -> Result<(), JsValue> {
    let state = load_state();
    let severity = optional_property(&state, "severity");
    if js_boolean(&severity) {
        set_if_present("#logSeverity", "value", &severity)?;
    }
    let search = optional_property(&state, "search");
    if js_boolean(&search) {
        set_if_present("#logSearch", "value", &search)?;
    }
    let auto_scroll = optional_property(&state, "autoScroll");
    set_if_present(
        "#logAutoScroll",
        "checked",
        &JsValue::from_bool(auto_scroll.as_bool() != Some(false)),
    )?;
    let font_size = optional_property(&state, "fontSize");
    if js_boolean(&font_size) {
        set_if_present("#logFontSize", "value", &font_size)?;
    }
    Ok(())
}

fn event_details(entries: &[(&str, JsValue)]) -> JsValue {
    let object = Object::new();
    for (key, value) in entries {
        let _ = set_property(object.as_ref(), key, value);
    }
    object.into()
}

fn ui_trace(action: &str, phase: &str, details: JsValue) {
    let Some(call) = invoke_function() else {
        return;
    };
    let inner = Object::new();
    let _ = set_property(
        inner.as_ref(),
        "patch",
        &JsValue::from_str("KGW_LOG_UI_TRACE_PATCH_R49B2"),
    );
    let _ = set_property(
        inner.as_ref(),
        "owner",
        &JsValue::from_str("log-existing-initLogTab-owner"),
    );
    let _ = set_property(inner.as_ref(), "action", &JsValue::from_str(action));
    let _ = set_property(inner.as_ref(), "phase", &JsValue::from_str(phase));
    let safe_details = if details.is_object() {
        details
    } else {
        Object::new().into()
    };
    let _ = set_property(inner.as_ref(), "details", &safe_details);
    let encoded = JSON::stringify(inner.as_ref())
        .map(JsValue::from)
        .unwrap_or_else(|_| JsValue::from_str("{}"));

    let args = Object::new();
    let _ = set_property(args.as_ref(), "scope", &JsValue::from_str("log"));
    let _ = set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    let _ = set_property(args.as_ref(), "action", &JsValue::from_str(action));
    let _ = set_property(args.as_ref(), "phase", &JsValue::from_str(phase));
    let _ = set_property(args.as_ref(), "details", &encoded);

    if let Ok(value) = call.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        spawn_local(async move {
            let _ = await_value(value).await;
        });
    }
}

fn event_trusted(event: &JsValue) -> bool {
    optional_property(event, "isTrusted")
        .as_bool()
        .unwrap_or(false)
}

fn add_listener(
    selector: &str,
    event_name: &str,
    callback: Closure<dyn FnMut(JsValue)>,
) -> Result<(), JsValue> {
    let element = qs(selector);
    if !js_boolean(&element) {
        return Ok(());
    }
    let _ = call_method2(
        &element,
        "addEventListener",
        &JsValue::from_str(event_name),
        callback.as_ref().unchecked_ref(),
    )?;
    callback.forget();
    Ok(())
}

#[wasm_bindgen(js_name = kgwLogInitTab)]
pub fn init_log_tab() -> Result<(), JsValue> {
    apply_initial_state()?;

    add_listener(
        "#logSeverity",
        "change",
        Closure::wrap(Box::new(move |event: JsValue| {
            ui_trace(
                "log-filter",
                "r49b2-log-severity-change",
                event_details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    ("value", JsValue::from_str(&field_value("#logSeverity", ""))),
                ]),
            );
            let _ = render();
        }) as Box<dyn FnMut(JsValue)>),
    )?;

    add_listener(
        "#logSearch",
        "input",
        Closure::wrap(Box::new(move |event: JsValue| {
            let length = field_value("#logSearch", "").encode_utf16().count() as f64;
            ui_trace(
                "log-filter",
                "r49b2-log-search-input",
                event_details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    ("valueLength", JsValue::from_f64(length)),
                ]),
            );
            let _ = render();
        }) as Box<dyn FnMut(JsValue)>),
    )?;

    add_listener(
        "#logAutoScroll",
        "change",
        Closure::wrap(Box::new(move |event: JsValue| {
            ui_trace(
                "log-option",
                "r49b2-log-autoscroll-change",
                event_details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    (
                        "checked",
                        JsValue::from_bool(field_checked("#logAutoScroll")),
                    ),
                ]),
            );
            let _ = save_state();
        }) as Box<dyn FnMut(JsValue)>),
    )?;

    add_listener(
        "#logFontSize",
        "change",
        Closure::wrap(Box::new(move |event: JsValue| {
            ui_trace(
                "log-option",
                "r49b2-log-font-size-change",
                event_details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    ("value", JsValue::from_str(&field_value("#logFontSize", ""))),
                ]),
            );
            let _ = render();
        }) as Box<dyn FnMut(JsValue)>),
    )?;

    add_listener(
        "#logClear",
        "click",
        Closure::wrap(Box::new(move |event: JsValue| {
            let visible = FILTERED_LINES.with(|lines| lines.borrow().len()) as f64;
            ui_trace(
                "log-action",
                "r49b2-log-clear-click",
                event_details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    ("visibleLines", JsValue::from_f64(visible)),
                ]),
            );
            spawn_local(async {
                if let Err(error) = clear_log().await {
                    let console = optional_property(&global(), "console");
                    let _ = call_method1(&console, "error", &error);
                }
            });
        }) as Box<dyn FnMut(JsValue)>),
    )?;

    add_listener(
        "#logCopy",
        "click",
        Closure::wrap(Box::new(move |event: JsValue| {
            let visible = FILTERED_LINES.with(|lines| lines.borrow().len()) as f64;
            ui_trace(
                "log-action",
                "r49b2-log-copy-click",
                event_details(&[
                    ("trusted", JsValue::from_bool(event_trusted(&event))),
                    ("visibleLines", JsValue::from_f64(visible)),
                ]),
            );
            spawn_local(async {
                if let Err(error) = copy_log().await {
                    let console = optional_property(&global(), "console");
                    let _ = call_method1(&console, "error", &error);
                }
            });
        }) as Box<dyn FnMut(JsValue)>),
    )?;

    let global = global();
    POLL_TIMER.with(|timer| {
        if let Some(id) = timer.borrow_mut().take()
            && let Some(clear_interval) =
                optional_property(&global, "clearInterval").dyn_ref::<Function>()
        {
            let _ = clear_interval.call1(&global, &id);
        }
    });

    spawn_local(async {
        let _ = refresh_log().await;
    });

    let callback = Closure::wrap(Box::new(move || {
        spawn_local(async {
            let _ = refresh_log().await;
        });
    }) as Box<dyn FnMut()>);
    if let Some(set_interval) = optional_property(&global, "setInterval").dyn_ref::<Function>() {
        let id = set_interval.call2(
            &global,
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(1200.0),
        )?;
        POLL_TIMER.with(|timer| *timer.borrow_mut() = Some(id));
        callback.forget();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_parsing_matches_legacy_contract() {
        for level in ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"] {
            assert_eq!(
                parse_log_level_text(&format!(
                    "2026-09-14 12:34:56 - {level} - [MainThread] - audit - message"
                )),
                level
            );
            assert_eq!(
                parse_log_level_text(&format!(
                    "time=\"x\" level=\"{}\" target=\"audit\" msg=\"message\"",
                    level.to_ascii_lowercase()
                )),
                level
            );
        }
        assert_eq!(
            parse_log_level_text("2026-09-14 12:34:56.123 - INFO - [worker-1] - x"),
            "INFO"
        );
        assert_eq!(
            parse_log_level_text("2026-09-14 12:34:56,123 - WARN - [worker-1] - x"),
            "WARN"
        );
        assert_eq!(
            parse_log_level_text("ordinary text contains ERROR but is not a native log record"),
            "UNKNOWN"
        );
        assert_eq!(
            parse_log_level_text("2026-09-14 12:34:56 ERROR message"),
            "UNKNOWN"
        );
    }

    #[test]
    fn severity_and_search_filter_match_legacy_rules() {
        let info = "2026-09-14 12:34:56 - INFO - [MainThread] - audit - Hello Kaspa";
        assert!(should_show_text(info, "ALL", ""));
        assert!(should_show_text(info, "INFO", "kaspa"));
        assert!(!should_show_text(info, "WARN", ""));
        assert!(!should_show_text("plain text", "TRACE", ""));
        assert!(!should_show_text(info, "ALL", "missing"));
    }

    #[test]
    fn html_escaping_preserves_legacy_order() {
        assert_eq!(highlight_line("<tag>&value>"), "&lt;tag&gt;&amp;value&gt;");
        assert_eq!(
            highlight_line("time=\"x\" level=\"INFO\""),
            "time=\"x\" level=\"INFO\""
        );
    }

    #[test]
    fn clipboard_decision_matches_legacy_api_and_fallback_contracts() {
        assert_eq!(
            clipboard_after_api(ClipboardApiState::Succeeded),
            ClipboardDecision::Succeeded
        );
        assert_eq!(
            clipboard_after_api(ClipboardApiState::Unavailable),
            ClipboardDecision::TryFallback
        );
        assert_eq!(
            clipboard_after_api(ClipboardApiState::Failed),
            ClipboardDecision::TryFallback
        );
        assert_eq!(
            clipboard_after_fallback(
                ClipboardApiState::Unavailable,
                ClipboardFallbackState::Succeeded,
            ),
            ClipboardDecision::Succeeded
        );
        assert_eq!(
            clipboard_after_fallback(ClipboardApiState::Failed, ClipboardFallbackState::Succeeded,),
            ClipboardDecision::Succeeded
        );
        assert_eq!(
            clipboard_after_fallback(
                ClipboardApiState::Unavailable,
                ClipboardFallbackState::Rejected,
            ),
            ClipboardDecision::FallbackRejected
        );
        assert_eq!(
            clipboard_after_fallback(
                ClipboardApiState::Unavailable,
                ClipboardFallbackState::Unavailable,
            ),
            ClipboardDecision::Unavailable
        );
        assert_eq!(
            clipboard_after_fallback(
                ClipboardApiState::Failed,
                ClipboardFallbackState::Unavailable,
            ),
            ClipboardDecision::PreserveApiError
        );
    }

    #[test]
    fn bilingual_copy_status_strings_are_owned_by_rust() {
        assert_eq!(copy_text_for_language("copiedButton", "en"), "Copied");
        assert_eq!(copy_text_for_language("copiedButton", "ar"), "تم النسخ");
        assert_eq!(
            copy_text_for_language("copyFailedStatus", "ar"),
            "فشل النسخ. الوصول إلى الحافظة غير متاح أو تم رفضه."
        );
    }
}
