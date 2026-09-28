use js_sys::{Array, Date, Function, JSON, Object, Promise, Reflect};
use std::cell::Cell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

thread_local! {
    static FONT_SIZE: Cell<u32> = const { Cell::new(11) };
    static FILTER_BUSY: Cell<bool> = const { Cell::new(false) };
    static FILTER_BUSY_STARTED_AT: Cell<f64> = const { Cell::new(0.0) };
    static FILTER_LAST_MUTATION_AT: Cell<f64> = const { Cell::new(0.0) };
    static FILTER_POLL_TIMER: Cell<Option<f64>> = const { Cell::new(None) };
    static FILTER_UNLOCK_TIMER: Cell<Option<f64>> = const { Cell::new(None) };
    static FILTER_BUSY_OWNER_INSTALLED: Cell<bool> = const { Cell::new(false) };
}

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
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

fn call3(target: &JsValue, name: &str, a: &JsValue, b: &JsValue, c: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|method| method.call3(target, a, b, c).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn root() -> JsValue {
    let doc = document();
    let direct = call1(&doc, "getElementById", &JsValue::from_str("explorer"));
    if present(&direct) {
        return direct;
    }
    let legacy = call1(
        &doc,
        "querySelector",
        &JsValue::from_str(".explorer-python-root"),
    );
    if present(&legacy) { legacy } else { doc }
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

fn style_property(node: &JsValue, name: &str, value: &str, priority: &str) {
    let style = property(node, "style");
    let _ = call3(
        &style,
        "setProperty",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
        &JsValue::from_str(priority),
    );
}

fn remove_attr(node: &JsValue, name: &str) {
    let _ = call1(node, "removeAttribute", &JsValue::from_str(name));
}

fn set_attr(node: &JsValue, name: &str, value: &str) {
    let _ = call2(
        node,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}

fn class_add(node: &JsValue, class: &str) {
    let _ = call1(
        &property(node, "classList"),
        "add",
        &JsValue::from_str(class),
    );
}

fn class_remove(node: &JsValue, class: &str) {
    let _ = call1(
        &property(node, "classList"),
        "remove",
        &JsValue::from_str(class),
    );
}

fn normalize_font_size_text(value: &str) -> u32 {
    let digits: String = crate::to_english_digits_text(value)
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .take(2)
        .collect();
    let parsed = digits.parse::<u32>().unwrap_or(9);
    parsed.clamp(6, 24)
}

fn normalize_font_size(value: &JsValue) -> u32 {
    let raw = if value.is_null() || value.is_undefined() {
        "9".to_owned()
    } else {
        crate::js_string_owned(value)
    };
    normalize_font_size_text(&raw)
}

fn current_font_size() -> u32 {
    FONT_SIZE.with(Cell::get).clamp(6, 24)
}

fn microscope_font_log(section: &JsValue, table: &JsValue, size: u32) {
    let style = property(section, "style");
    let css_variable = crate::js_string_owned(&call1(
        &style,
        "getPropertyValue",
        &JsValue::from_str("--explorer-table-font-size"),
    ));
    let computed = call1(&window(), "getComputedStyle", table);
    let table_font_size = crate::js_string_owned(&property(&computed, "fontSize"));
    let payload = Object::new();
    for (key, value) in [
        ("stage", JsValue::from_str("TABLE FONT SIZE APPLIED")),
        (
            "at",
            JsValue::from_str(&String::from(Date::new_0().to_iso_string())),
        ),
        ("size", JsValue::from_f64(size as f64)),
        ("cssVariable", JsValue::from_str(&css_variable)),
        ("tableFontSize", JsValue::from_str(&table_font_size)),
    ] {
        set(payload.as_ref(), key, &value);
    }
    let encoded = JSON::stringify(payload.as_ref())
        .ok()
        .map(|value| crate::js_string_owned(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    let console = property(&global(), "console");
    if let Some(info) = function(&console, "info") {
        let _ = info.call1(
            &console,
            &JsValue::from_str(&format!(
                "[KGW][microscope][explorer] TABLE FONT SIZE APPLIED :: {encoded}"
            )),
        );
    }
}

fn set_table_font_size_impl(section: &JsValue) -> u32 {
    if !present(section) {
        return current_font_size();
    }
    let size = current_font_size();
    let input = query(section, "#explorerTableFontSize");
    if present(&input) {
        set(&input, "value", &JsValue::from_str(&size.to_string()));
    }
    let table = query(section, "#explorerTransactionsTable");
    if !present(&table) {
        return size;
    }
    style_property(
        section,
        "--explorer-table-font-size",
        &format!("{size}px"),
        "",
    );
    style_property(&table, "font-size", &format!("{size}px"), "important");
    for cell in query_all(&table, "th, td") {
        style_property(&cell, "font-size", &format!("{size}px"), "important");
    }
    microscope_font_log(section, &table, size);
    size
}

fn apply_font_size_impl(section: &JsValue, raw_value: &JsValue) -> u32 {
    let size = normalize_font_size(raw_value);
    FONT_SIZE.with(|state| state.set(size));
    set_table_font_size_impl(section)
}

fn bind_font_spinbox_impl(section: &JsValue) -> bool {
    if !present(section) {
        return false;
    }
    let input = query(section, "#explorerTableFontSize");
    if !present(&input) {
        return false;
    }
    let data = property(&input, "dataset");
    if crate::js_string_owned(&property(&data, "kgwFontBound")) == "1" {
        return false;
    }
    set(&data, "kgwFontBound", &JsValue::from_str("1"));

    for event_name in ["input", "blur"] {
        let section = section.clone();
        let input_for_callback = input.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let raw = property(&input_for_callback, "value");
            apply_font_size_impl(&section, &raw);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &input,
            "addEventListener",
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    let dec = query(section, "#explorerTableFontDecrease");
    if present(&dec) {
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let next = current_font_size().saturating_sub(1);
            apply_font_size_impl(&section, &JsValue::from_f64(next as f64));
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &dec,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    let inc = query(section, "#explorerTableFontIncrease");
    if present(&inc) {
        let section = section.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let next = current_font_size().saturating_add(1);
            apply_font_size_impl(&section, &JsValue::from_f64(next as f64));
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &inc,
            "addEventListener",
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }
    true
}

fn control_key(control: &JsValue) -> String {
    let mut parts = Vec::new();
    for value in [
        property(control, "id"),
        property(control, "name"),
        property(control, "className"),
        call1(control, "getAttribute", &JsValue::from_str("aria-label")),
        call1(control, "getAttribute", &JsValue::from_str("title")),
        property(control, "textContent"),
    ] {
        let text = crate::js_string_owned(&value);
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join(" ").to_lowercase()
}

fn busy_allowed_text(id: &str, key: &str) -> bool {
    id == "explorerCancel"
        || key.contains("cancel")
        || key.contains("إلغاء")
        || key.contains("الغاء")
}

fn busy_allowed(control: &JsValue) -> bool {
    if !present(control) {
        return false;
    }
    let id = crate::js_string_owned(&property(control, "id"));
    busy_allowed_text(&id, &control_key(control))
}

fn set_control_locked(control: &JsValue, locked: bool) {
    if !present(control) {
        return;
    }
    let can_disable = Reflect::has(control, &JsValue::from_str("disabled")).unwrap_or(false);
    let data = property(control, "dataset");
    let previous = property(&data, "kgwPrevDisabled");

    if locked {
        if previous.is_undefined() {
            let was_disabled = can_disable && crate::js_boolean(&property(control, "disabled"));
            set(
                &data,
                "kgwPrevDisabled",
                &JsValue::from_str(if was_disabled { "true" } else { "false" }),
            );
        }
        if can_disable {
            set(control, "disabled", &JsValue::TRUE);
        }
        set_attr(control, "aria-disabled", "true");
        class_add(control, "disabled");
        style_property(control, "pointer-events", "none", "");
        style_property(control, "cursor", "not-allowed", "");
        return;
    }

    if !previous.is_undefined() {
        if can_disable {
            set(
                control,
                "disabled",
                &JsValue::from_bool(crate::js_string_owned(&previous) == "true"),
            );
        }
        if let Ok(object) = data.dyn_into::<Object>() {
            let _ = Reflect::delete_property(&object, &JsValue::from_str("kgwPrevDisabled"));
        }
    } else if can_disable {
        set(control, "disabled", &JsValue::FALSE);
    }
    remove_attr(control, "aria-disabled");
    class_remove(control, "disabled");
    class_remove(control, "is-disabled");
    style_property(control, "pointer-events", "", "");
    style_property(control, "cursor", "", "");
}

fn custom_busy_event(busy: bool) -> Option<JsValue> {
    let constructor = property(&global(), "CustomEvent")
        .dyn_into::<Function>()
        .ok()?;
    let detail = Object::new();
    set(detail.as_ref(), "busy", &JsValue::from_bool(busy));
    let init = Object::new();
    set(init.as_ref(), "detail", detail.as_ref());
    let args = Array::new();
    args.push(&JsValue::from_str("kgw:explorer-fetch-busy"));
    args.push(init.as_ref());
    Reflect::construct(&constructor, &args).ok()
}

fn apply_busy_controls_impl(section: &JsValue, busy: bool) -> bool {
    let root_node = if present(section) {
        section.clone()
    } else {
        root()
    };
    if !present(&root_node) {
        return false;
    }
    for control in query_all(&root_node, "button,input,select,textarea,a,[role='button']") {
        if busy_allowed(&control) {
            set_control_locked(&control, false);
            if Reflect::has(&control, &JsValue::from_str("disabled")).unwrap_or(false) {
                set(&control, "disabled", &JsValue::from_bool(!busy));
            }
        } else {
            set_control_locked(&control, busy);
        }
    }

    let win = window();
    let previous_busy = crate::js_boolean(&property(&win, "__kgwExplorerFetchBusy"));
    set(&win, "__kgwExplorerFetchBusy", &JsValue::from_bool(busy));
    if previous_busy != busy
        && let Some(event) = custom_busy_event(busy)
    {
        let _ = call1(&win, "dispatchEvent", &event);
    }
    if let Some(policy) = function(&win, "kgwApplyShellExplorerFetchBusyPolicy") {
        let _ = policy.call1(&win, &JsValue::from_bool(busy));
    }
    true
}

#[wasm_bindgen(js_name = explorerSetTableFontSize)]
pub fn explorer_set_table_font_size(section: JsValue) -> u32 {
    set_table_font_size_impl(&section)
}

#[wasm_bindgen(js_name = explorerApplyFontSize)]
pub fn explorer_apply_font_size(section: JsValue, raw_value: JsValue) -> u32 {
    apply_font_size_impl(&section, &raw_value)
}

#[wasm_bindgen(js_name = explorerBindFontSpinbox)]
pub fn explorer_bind_font_spinbox(section: JsValue) -> bool {
    bind_font_spinbox_impl(&section)
}

#[wasm_bindgen(js_name = explorerApplyLocalBusyControls)]
pub fn explorer_apply_local_busy_controls(section: JsValue, busy: bool) -> bool {
    apply_busy_controls_impl(&section, busy)
}

fn manual_address_value_impl(section: &JsValue) -> String {
    let scope = if present(section) {
        section.clone()
    } else {
        root()
    };
    crate::js_string_owned(&property(&query(&scope, "#explorerAddress"), "value"))
        .trim()
        .to_owned()
}

fn manual_kaspa_address_text(value: &str) -> bool {
    let text = value.trim().to_ascii_lowercase();
    let tail = if let Some(value) = text.strip_prefix("kaspa:") {
        value
    } else if let Some(value) = text.strip_prefix("kaspatest:") {
        value
    } else {
        return false;
    };
    tail.len() >= 50 && tail.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn error_text(error: &JsValue) -> String {
    let message = property(error, "message");
    let value = if present(&message) {
        crate::js_string_owned(&message)
    } else {
        crate::js_string_owned(error)
    };
    if value.trim().is_empty() {
        "unknown error".to_owned()
    } else {
        value.replace(['\r', '\n', '\t'], " ").trim().to_owned()
    }
}

async fn await_js(value: JsValue) -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::resolve(&value)).await
}

fn set_explorer_status(section: &JsValue, message: &str, state: &str) {
    let scope = if present(section) {
        section.clone()
    } else {
        root()
    };
    let node = query(&scope, "#explorerStatus");
    if present(&node) {
        set(&node, "hidden", &JsValue::FALSE);
        set(&node, "textContent", &JsValue::from_str(message));
        set(
            &property(&node, "dataset"),
            "state",
            &JsValue::from_str(state),
        );
        set_attr(
            &node,
            "role",
            if state == "error" { "alert" } else { "status" },
        );
        set_attr(
            &node,
            "aria-live",
            if state == "error" {
                "assertive"
            } else {
                "polite"
            },
        );
    }
    if let Some(progress) = function(&window(), "kgwSetGlobalFetchProgressText") {
        let _ = progress.call1(&window(), &JsValue::from_str(message));
    }
    if let Some(info) = function(&property(&global(), "console"), "info") {
        let _ = info.call2(
            &property(&global(), "console"),
            &JsValue::from_str("[KGW Explorer]"),
            &JsValue::from_str(message),
        );
    }
}

async fn save_manual_address_impl(section: JsValue) -> bool {
    let scope = if present(&section) { section } else { root() };
    let address = manual_address_value_impl(&scope);
    if !manual_kaspa_address_text(&address) {
        set_explorer_status(&scope, "Invalid Kaspa address.", "info");
        return false;
    }

    match crate::explorer_addresses::save_address_to_database_internal(&address, "").await {
        Ok(_) => {
            crate::explorer_addresses::invalidate_address_names();
            let _ = crate::explorer_addresses::load_saved_addresses_internal(scope.clone()).await;
            let _ =
                crate::explorer_addresses::refresh_address_name_internal(scope.clone(), &address)
                    .await;
            set_explorer_status(&scope, "Address saved.", "info");
            if let Some(refresh) = function(&window(), "kgwRefreshSettingsAddresses")
                && let Ok(value) = refresh.call0(&window())
            {
                let _ = await_js(value).await;
            }
            true
        }
        Err(error) => {
            set_explorer_status(
                &scope,
                &format!("Save address failed: {}", error_text(&error)),
                "info",
            );
            false
        }
    }
}

fn event_with_bubbles(name: &str) -> Option<JsValue> {
    let constructor = property(&global(), "Event").dyn_into::<Function>().ok()?;
    let init = Object::new();
    set(init.as_ref(), "bubbles", &JsValue::TRUE);
    let args = Array::new();
    args.push(&JsValue::from_str(name));
    args.push(init.as_ref());
    Reflect::construct(&constructor, &args).ok()
}

fn close_address_dropdown(input: &JsValue, dropdown: &JsValue) {
    set(dropdown, "hidden", &JsValue::TRUE);
    set_attr(input, "aria-expanded", "false");
}

fn dropdown_number(value: JsValue, fallback: f64) -> f64 {
    let number = crate::js_number(&value);
    if number.is_finite() { number } else { fallback }
}

fn place_address_dropdown(input: &JsValue, dropdown: &JsValue) {
    let Some(rect_fn) = function(input, "getBoundingClientRect") else {
        return;
    };
    let Ok(rect) = rect_fn.call0(input) else {
        return;
    };
    let doc_root = property(&document(), "documentElement");
    let right = dropdown_number(property(&rect, "right"), 0.0);
    let bottom = dropdown_number(property(&rect, "bottom"), 0.0);
    let rect_width = dropdown_number(property(&rect, "width"), 220.0);
    let viewport_width = dropdown_number(
        property(&window(), "innerWidth"),
        dropdown_number(property(&doc_root, "clientWidth"), right),
    );
    let viewport_height = dropdown_number(
        property(&window(), "innerHeight"),
        dropdown_number(property(&doc_root, "clientHeight"), bottom),
    );
    let width = rect_width.max(220.0);
    let left = dropdown_number(property(&rect, "left"), 8.0)
        .min(viewport_width - width - 8.0)
        .max(8.0);
    let top = (bottom + 4.0).min(viewport_height - 64.0).max(8.0);
    let style = property(dropdown, "style");
    for (name, value) in [
        ("position", "fixed".to_owned()),
        ("left", format!("{left}px")),
        ("top", format!("{top}px")),
        ("width", format!("{width}px")),
        ("transform", "none".to_owned()),
    ] {
        set(&style, name, &JsValue::from_str(&value));
    }
}

fn translated_saved_addresses_empty() -> String {
    for name in ["kgwT", "kgwTranslate", "t"] {
        if let Some(translate) = function(&window(), name)
            && let Ok(value) = translate.call2(
                &window(),
                &JsValue::from_str("ui.explorer.noSavedAddresses"),
                &JsValue::from_str("No saved addresses"),
            )
        {
            let text = crate::js_string_owned(&value);
            if !text.is_empty() && text != "ui.explorer.noSavedAddresses" {
                return text;
            }
        }
    }
    "No saved addresses".to_owned()
}

fn apply_saved_address(input: &JsValue, dropdown: &JsValue, address: &str) {
    set(input, "value", &JsValue::from_str(address));
    for event_name in ["input", "change"] {
        if let Some(event) = event_with_bubbles(event_name) {
            let _ = call1(input, "dispatchEvent", &event);
        }
    }
    close_address_dropdown(input, dropdown);
}

fn read_saved_address_options(datalist: &JsValue) -> Vec<String> {
    let mut output = Vec::new();
    for option in query_all(datalist, "option") {
        let value = {
            let candidate = crate::js_string_owned(&property(&option, "value"));
            if candidate.trim().is_empty() {
                crate::js_string_owned(&property(&option, "textContent"))
            } else {
                candidate
            }
        };
        let value = value.trim().to_owned();
        if !value.is_empty() && !output.contains(&value) {
            output.push(value);
        }
    }
    output
}

fn render_address_dropdown(input: &JsValue, datalist: &JsValue, dropdown: &JsValue) {
    if let Some(replace) = function(dropdown, "replaceChildren") {
        let _ = replace.call0(dropdown);
    } else {
        set(dropdown, "textContent", &JsValue::from_str(""));
    }
    let addresses = read_saved_address_options(datalist);
    if addresses.is_empty() {
        let empty = call1(&document(), "createElement", &JsValue::from_str("div"));
        set(
            &empty,
            "className",
            &JsValue::from_str("kgw-explorer-address-dropdown-empty"),
        );
        set(
            &empty,
            "textContent",
            &JsValue::from_str(&translated_saved_addresses_empty()),
        );
        let _ = call1(dropdown, "appendChild", &empty);
        return;
    }

    for address in addresses {
        let button = call1(&document(), "createElement", &JsValue::from_str("button"));
        set(&button, "type", &JsValue::from_str("button"));
        set(
            &button,
            "className",
            &JsValue::from_str("kgw-explorer-address-dropdown-option"),
        );
        set_attr(&button, "role", "option");
        let label = if address.chars().count() > 54 {
            format!("{}...", address.chars().take(54).collect::<String>())
        } else {
            address.clone()
        };
        set(&button, "textContent", &JsValue::from_str(&label));
        set(&button, "title", &JsValue::from_str(&address));

        let down = Closure::wrap(Box::new(move |event: JsValue| {
            if let Some(prevent) = function(&event, "preventDefault") {
                let _ = prevent.call0(&event);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &button,
            "addEventListener",
            &JsValue::from_str("mousedown"),
            down.as_ref().unchecked_ref(),
        );
        down.forget();

        let input_for_click = input.clone();
        let dropdown_for_click = dropdown.clone();
        let address_for_click = address.clone();
        let click = Closure::wrap(Box::new(move |_event: JsValue| {
            apply_saved_address(&input_for_click, &dropdown_for_click, &address_for_click);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &button,
            "addEventListener",
            &JsValue::from_str("click"),
            click.as_ref().unchecked_ref(),
        );
        click.forget();
        let _ = call1(dropdown, "appendChild", &button);
    }
}

fn open_address_dropdown(input: &JsValue, datalist: &JsValue, dropdown: &JsValue) {
    render_address_dropdown(input, datalist, dropdown);
    place_address_dropdown(input, dropdown);
    set(dropdown, "hidden", &JsValue::FALSE);
    set_attr(input, "aria-expanded", "true");
}

fn schedule_address_autosave(section: JsValue, input: JsValue, reason: &'static str, delay: f64) {
    let win = window();
    let prior = property(&input, "__kgwExplorerAutosaveTimer");
    if let Some(clear) = function(&win, "clearTimeout")
        && let Some(timer) = prior.as_f64()
    {
        let _ = clear.call1(&win, &JsValue::from_f64(timer));
    }

    let input_for_timeout = input.clone();
    let callback = Closure::wrap(Box::new(move || {
        let address = manual_address_value_impl(&section);
        if !manual_kaspa_address_text(&address) {
            return;
        }
        let last = crate::js_string_owned(&property(
            &input_for_timeout,
            "__kgwExplorerLastAutosavedAddress",
        ));
        if address == last {
            return;
        }
        set(
            &input_for_timeout,
            "__kgwExplorerLastAutosavedAddress",
            &JsValue::from_str(&address),
        );
        let section_async = section.clone();
        let input_async = input_for_timeout.clone();
        spawn_local(async move {
            if save_manual_address_impl(section_async.clone()).await {
                set_explorer_status(
                    &section_async,
                    if reason == "paste" {
                        "Address pasted and saved."
                    } else {
                        "Address saved."
                    },
                    "info",
                );
                let _ =
                    crate::explorer_addresses::load_saved_addresses_internal(section_async.clone())
                        .await;
            } else {
                set(
                    &input_async,
                    "__kgwExplorerLastAutosavedAddress",
                    &JsValue::from_str(""),
                );
            }
        });
    }) as Box<dyn FnMut()>);

    if let Some(set_timeout) = function(&win, "setTimeout")
        && let Ok(timer) = set_timeout.call2(
            &win,
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(delay),
        )
    {
        set(&input, "__kgwExplorerAutosaveTimer", &timer);
    }
    callback.forget();
}

fn install_manual_address_save_impl() -> bool {
    let section = root();
    let input = query(&section, "#explorerAddress");
    let datalist = query(&section, "#explorerAddressOptions");
    let dropdown = query(&section, "#explorerAddressDropdown");
    if !present(&input) || !present(&datalist) || !present(&dropdown) {
        return false;
    }

    let body = property(&document(), "body");
    if !Object::is(&property(&dropdown, "parentElement"), &body) {
        let _ = call1(&body, "appendChild", &dropdown);
    }

    let data = property(&input, "dataset");
    if crate::js_string_owned(&property(&data, "kgwExplorerCustomDropdownR5Installed")) == "1" {
        return true;
    }
    set(
        &data,
        "kgwExplorerCustomDropdownR5Installed",
        &JsValue::from_str("1"),
    );

    remove_attr(&input, "list");
    set_attr(&input, "aria-haspopup", "listbox");
    set_attr(&input, "aria-controls", "explorerAddressDropdown");
    set_attr(&input, "aria-expanded", "false");

    for event_name in ["focus", "click"] {
        let input_for_open = input.clone();
        let datalist_for_open = datalist.clone();
        let dropdown_for_open = dropdown.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            open_address_dropdown(&input_for_open, &datalist_for_open, &dropdown_for_open);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &input,
            "addEventListener",
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    {
        let input_for_key = input.clone();
        let datalist_for_key = datalist.clone();
        let dropdown_for_key = dropdown.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            let key = crate::js_string_owned(&property(&event, "key"));
            if key == "ArrowDown" || key == "Enter" {
                open_address_dropdown(&input_for_key, &datalist_for_key, &dropdown_for_key);
            } else if key == "Escape" {
                close_address_dropdown(&input_for_key, &dropdown_for_key);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &input,
            "addEventListener",
            &JsValue::from_str("keydown"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    for (event_name, reason, delay) in [
        ("paste", "paste", 40.0),
        ("input", "input", 350.0),
        ("change", "change", 0.0),
        ("blur", "blur", 0.0),
    ] {
        let section_for_event = section.clone();
        let input_for_event = input.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            schedule_address_autosave(
                section_for_event.clone(),
                input_for_event.clone(),
                reason,
                delay,
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &input,
            "addEventListener",
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    {
        let input_for_resize = input.clone();
        let dropdown_for_resize = dropdown.clone();
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            if !crate::js_boolean(&property(&dropdown_for_resize, "hidden")) {
                place_address_dropdown(&input_for_resize, &dropdown_for_resize);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &window(),
            "addEventListener",
            &JsValue::from_str("resize"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    {
        let input_for_doc = input.clone();
        let dropdown_for_doc = dropdown.clone();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            let inside_dropdown = crate::js_boolean(&call1(&dropdown_for_doc, "contains", &target));
            if Object::is(&target, &input_for_doc) || inside_dropdown {
                return;
            }
            close_address_dropdown(&input_for_doc, &dropdown_for_doc);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &document(),
            "addEventListener",
            &JsValue::from_str("mousedown"),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    render_address_dropdown(&input, &datalist, &dropdown);
    true
}

fn normalize_filter_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn filter_scope() -> JsValue {
    let doc = document();
    for selector in [
        "#explorer",
        ".explorer-python-root",
        "[data-tab-panel='explorer']",
    ] {
        let node = query(&doc, selector);
        if present(&node) {
            return node;
        }
    }
    doc
}

fn filter_attr(node: &JsValue, name: &str) -> String {
    crate::js_string_owned(&call1(node, "getAttribute", &JsValue::from_str(name)))
}

fn filter_text_of(node: &JsValue) -> String {
    normalize_filter_text(
        &[
            crate::js_string_owned(&property(node, "textContent")),
            crate::js_string_owned(&property(node, "value")),
            filter_attr(node, "aria-label"),
        ]
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or_default(),
    )
}

fn explorer_visible() -> bool {
    let scope = filter_scope();
    if Object::is(&scope, &document()) {
        return true;
    }
    let rect = call1(&scope, "getBoundingClientRect", &JsValue::UNDEFINED);
    if !present(&rect) {
        return true;
    }
    let width = crate::js_number(&property(&rect, "width"));
    let height = crate::js_number(&property(&rect, "height"));
    width > 0.0 && height > 0.0
}

fn is_address_control(node: &JsValue) -> bool {
    if !present(node) {
        return false;
    }
    let id = normalize_filter_text(&crate::js_string_owned(&property(node, "id")));
    let name = normalize_filter_text(&crate::js_string_owned(&property(node, "name")));
    let label = normalize_filter_text(&filter_attr(node, "aria-label"));
    let placeholder = normalize_filter_text(&filter_attr(node, "placeholder"));
    if [id, name, label, placeholder]
        .iter()
        .any(|value| value.contains("address"))
    {
        return true;
    }
    if crate::js_string_owned(&property(node, "tagName")).eq_ignore_ascii_case("SELECT") {
        let option_text = query_all(node, "option")
            .into_iter()
            .take(25)
            .map(|option| {
                format!(
                    "{} {}",
                    crate::js_string_owned(&property(&option, "value")),
                    crate::js_string_owned(&property(&option, "textContent"))
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        return option_text.contains("kaspa:")
            || option_text.contains("saved address")
            || option_text.contains("select saved");
    }
    false
}

fn unique_nodes(nodes: Vec<JsValue>) -> Vec<JsValue> {
    let mut output = Vec::new();
    for node in nodes {
        if !output.iter().any(|existing| Object::is(existing, &node)) {
            output.push(node);
        }
    }
    output
}

fn address_controls() -> Vec<JsValue> {
    unique_nodes(query_all(
        &filter_scope(),
        "select,input,#explorerAddressSelect,#explorerSavedAddressSelect,#savedAddressSelect,[data-role='address-select']",
    ))
    .into_iter()
    .filter(is_address_control)
    .collect()
}

fn contains_kaspa_address_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let mut offset = 0usize;
    while let Some(found) = lower[offset..].find("kaspa:") {
        let start = offset + found + "kaspa:".len();
        let count = lower[start..]
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric())
            .count();
        if count >= 20 {
            return true;
        }
        offset = start;
        if offset >= lower.len() {
            break;
        }
    }
    false
}

fn has_selected_address() -> bool {
    for node in address_controls() {
        let value = crate::js_string_owned(&property(&node, "value"));
        if value.contains("kaspa:") {
            return true;
        }
        if crate::js_string_owned(&property(&node, "tagName")).eq_ignore_ascii_case("SELECT") {
            let index = crate::js_number(&property(&node, "selectedIndex"));
            if index.is_finite() && index >= 0.0 {
                let options = property(&node, "options");
                let selected =
                    Reflect::get(&options, &JsValue::from_f64(index)).unwrap_or(JsValue::UNDEFINED);
                if crate::js_string_owned(&property(&selected, "textContent")).contains("kaspa:") {
                    return true;
                }
            }
        }
    }
    contains_kaspa_address_text(&crate::js_string_owned(&property(
        &filter_scope(),
        "textContent",
    )))
}

fn is_action_button(node: &JsValue) -> bool {
    if !crate::js_string_owned(&property(node, "tagName")).eq_ignore_ascii_case("BUTTON") {
        return false;
    }
    let id = normalize_filter_text(&crate::js_string_owned(&property(node, "id")));
    let text = filter_text_of(node);
    let i18n = normalize_filter_text(&crate::js_string_owned(&property(
        &property(node, "dataset"),
        "i18n",
    )));
    id.contains("fetch")
        || id.contains("forcefetch")
        || id.contains("cancel")
        || id.contains("openexplorer")
        || i18n.contains("fetch")
        || i18n.contains("cancel")
        || matches!(
            text.as_str(),
            "fetch" | "force fetch" | "cancel" | "explorer"
        )
}

fn is_filter_control(node: &JsValue) -> bool {
    if !present(node) {
        return false;
    }
    let scope = filter_scope();
    if !Object::is(&scope, &document()) && !crate::js_boolean(&call1(&scope, "contains", node)) {
        return false;
    }
    if is_address_control(node) || is_action_button(node) {
        return false;
    }
    let id = normalize_filter_text(&crate::js_string_owned(&property(node, "id")));
    let name = normalize_filter_text(&crate::js_string_owned(&property(node, "name")));
    let placeholder = normalize_filter_text(&filter_attr(node, "placeholder"));
    let i18n = normalize_filter_text(&crate::js_string_owned(&property(
        &property(node, "dataset"),
        "i18n",
    )));
    let text = filter_text_of(node);
    let tag = crate::js_string_owned(&property(node, "tagName")).to_uppercase();
    let input_type = crate::js_string_owned(&property(node, "type")).to_lowercase();

    if tag == "INPUT" && input_type == "date" {
        return true;
    }
    if tag == "INPUT" && (input_type == "search" || placeholder.contains("search")) {
        return true;
    }
    if tag == "SELECT" {
        if [id, name].iter().any(|value| {
            value.contains("language") || value.contains("currency") || value.contains("theme")
        }) {
            return false;
        }
        return true;
    }
    if tag == "BUTTON" {
        return id.contains("filter")
            || id.contains("reset")
            || i18n.contains("filter")
            || i18n.contains("reset")
            || matches!(text.as_str(), "filter" | "reset filter");
    }
    false
}

fn filter_controls() -> Vec<JsValue> {
    unique_nodes(query_all(
        &filter_scope(),
        "input[type='date'],input[type='search'],input[placeholder*='Search'],input[placeholder*='Address'],input[placeholder*='Transaction'],select,button,#explorerFilter,#explorerResetFilter",
    ))
    .into_iter()
    .filter(is_filter_control)
    .collect()
}

fn find_action_button(id: &str, phrase: &str, contains: bool) -> JsValue {
    let scope = filter_scope();
    let direct = query(&scope, id);
    if present(&direct) {
        return direct;
    }
    query_all(&scope, "button")
        .into_iter()
        .find(|button| {
            let text = filter_text_of(button);
            if contains {
                text.contains(phrase)
            } else {
                text == phrase
            }
        })
        .unwrap_or(JsValue::UNDEFINED)
}

fn action_buttons() -> (JsValue, JsValue, JsValue) {
    (
        find_action_button("#explorerFetch", "fetch", false),
        find_action_button("#explorerForceFetch", "force fetch", true),
        find_action_button("#explorerCancel", "cancel", false),
    )
}

fn set_filter_availability(enabled: bool, reason: &str) {
    for node in filter_controls() {
        set(&node, "disabled", &JsValue::from_bool(!enabled));
        set_attr(
            &node,
            "aria-disabled",
            if enabled { "false" } else { "true" },
        );
        set(
            &property(&node, "dataset"),
            "kgwExplorerFilterLifecycle",
            &JsValue::from_str(reason),
        );
    }
    let scope = filter_scope();
    if !Object::is(&scope, &document()) {
        let data = property(&scope, "dataset");
        if present(&data) {
            set(
                &data,
                "kgwExplorerFiltersEnabled",
                &JsValue::from_str(if enabled { "true" } else { "false" }),
            );
            set(
                &data,
                "kgwExplorerFiltersReason",
                &JsValue::from_str(reason),
            );
        }
    }
    let root = property(&document(), "documentElement");
    let data = property(&root, "dataset");
    set(
        &data,
        "kgwExplorerFiltersEnabled",
        &JsValue::from_str(if enabled { "true" } else { "false" }),
    );
    set(
        &data,
        "kgwExplorerFiltersReason",
        &JsValue::from_str(reason),
    );
}

fn refresh_filter_availability(reason: &str) {
    if FILTER_BUSY.with(Cell::get) {
        set_filter_availability(false, &format!("busy:{reason}"));
        return;
    }
    let selected = has_selected_address();
    set_filter_availability(
        selected,
        &format!(
            "{}:{reason}",
            if selected {
                "address-selected"
            } else {
                "no-address"
            }
        ),
    );
}

fn fetch_buttons_idle() -> bool {
    let (fetch, force_fetch, _) = action_buttons();
    [fetch, force_fetch]
        .into_iter()
        .all(|button| !present(&button) || !crate::js_boolean(&property(&button, "disabled")))
}

fn cancel_looks_idle() -> bool {
    let (_, _, cancel) = action_buttons();
    if !present(&cancel) {
        return true;
    }
    let style = call1(&window(), "getComputedStyle", &cancel);
    let hidden = crate::js_string_owned(&property(&style, "display")) == "none"
        || crate::js_string_owned(&property(&style, "visibility")) == "hidden"
        || property(&cancel, "offsetParent").is_null();
    hidden
        || crate::js_boolean(&property(&cancel, "disabled"))
        || filter_attr(&cancel, "aria-disabled") == "true"
}

fn stop_filter_busy_timers() {
    let win = window();
    FILTER_POLL_TIMER.with(|timer| {
        if let Some(value) = timer.take() {
            let _ = call1(&win, "clearInterval", &JsValue::from_f64(value));
        }
    });
    FILTER_UNLOCK_TIMER.with(|timer| {
        if let Some(value) = timer.take() {
            let _ = call1(&win, "clearTimeout", &JsValue::from_f64(value));
        }
    });
}

fn end_filter_busy(reason: &str) {
    FILTER_BUSY.with(|state| state.set(false));
    stop_filter_busy_timers();
    refresh_filter_availability(&format!("fetch-{reason}"));
}

fn begin_filter_busy(reason: &str) {
    let now = Date::now();
    FILTER_BUSY.with(|state| state.set(true));
    FILTER_BUSY_STARTED_AT.with(|state| state.set(now));
    FILTER_LAST_MUTATION_AT.with(|state| state.set(now));
    set_filter_availability(false, reason);
    stop_filter_busy_timers();

    let interval = Closure::wrap(Box::new(move || {
        if !FILTER_BUSY.with(Cell::get) {
            return;
        }
        let now = Date::now();
        let elapsed = now - FILTER_BUSY_STARTED_AT.with(Cell::get);
        let quiet = now - FILTER_LAST_MUTATION_AT.with(Cell::get);
        if elapsed > 1800.0 && quiet > 900.0 && fetch_buttons_idle() && cancel_looks_idle() {
            end_filter_busy("buttons-idle");
            return;
        }
        if elapsed > 90_000.0 && fetch_buttons_idle() {
            end_filter_busy("watchdog");
        }
    }) as Box<dyn FnMut()>);
    let timer = call2(
        &window(),
        "setInterval",
        interval.as_ref().unchecked_ref(),
        &JsValue::from_f64(500.0),
    );
    FILTER_POLL_TIMER.with(|state| state.set(timer.as_f64()));
    interval.forget();

    let timeout = Closure::wrap(Box::new(move || {
        if FILTER_BUSY.with(Cell::get) && fetch_buttons_idle() {
            end_filter_busy("max-timeout");
        }
    }) as Box<dyn FnMut()>);
    let timer = call2(
        &window(),
        "setTimeout",
        timeout.as_ref().unchecked_ref(),
        &JsValue::from_f64(180_000.0),
    );
    FILTER_UNLOCK_TIMER.with(|state| state.set(timer.as_f64()));
    timeout.forget();
}

fn event_button(target: &JsValue) -> JsValue {
    call1(target, "closest", &JsValue::from_str("button"))
}

fn fetch_click_target(target: &JsValue) -> bool {
    let button = event_button(target);
    if !present(&button) {
        return false;
    }
    let id = normalize_filter_text(&crate::js_string_owned(&property(&button, "id")));
    let text = filter_text_of(&button);
    let i18n = normalize_filter_text(&crate::js_string_owned(&property(
        &property(&button, "dataset"),
        "i18n",
    )));
    id == "explorerfetch"
        || id == "explorerforcefetch"
        || i18n.contains("fetch")
        || matches!(text.as_str(), "fetch" | "force fetch")
}

fn cancel_click_target(target: &JsValue) -> bool {
    let button = event_button(target);
    if !present(&button) {
        return false;
    }
    let id = normalize_filter_text(&crate::js_string_owned(&property(&button, "id")));
    let text = filter_text_of(&button);
    let i18n = normalize_filter_text(&crate::js_string_owned(&property(
        &property(&button, "dataset"),
        "i18n",
    )));
    id == "explorercancel" || text == "cancel" || i18n.contains("cancel")
}

fn schedule_filter_action(reason: String, delay: f64, end_busy: bool) {
    let callback = Closure::wrap(Box::new(move || {
        if end_busy {
            end_filter_busy(&reason);
        } else {
            refresh_filter_availability(&reason);
        }
    }) as Box<dyn FnMut()>);
    let _ = call2(
        &window(),
        "setTimeout",
        callback.as_ref().unchecked_ref(),
        &JsValue::from_f64(delay),
    );
    callback.forget();
}

fn install_filter_invoke_readonly_marker() {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let legacy = property(&tauri, "tauri");
    let owner = if present(&core) { core } else { legacy };
    if present(&owner) && property(&owner, "invoke").dyn_ref::<Function>().is_some() {
        set(
            &property(&property(&document(), "documentElement"), "dataset"),
            "kgwExplorerInvokeReadonlySafeV1",
            &JsValue::from_str("true"),
        );
    }
}

fn install_filter_busy_owner_impl() -> bool {
    if FILTER_BUSY_OWNER_INSTALLED.with(Cell::get) {
        return false;
    }
    FILTER_BUSY_OWNER_INSTALLED.with(|state| state.set(true));
    install_filter_invoke_readonly_marker();

    for event_name in ["change", "input"] {
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            let target = property(&event, "target");
            if is_address_control(&target) {
                schedule_filter_action(format!("address-{event_name}"), 0.0, false);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call3(
            &document(),
            "addEventListener",
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        );
        callback.forget();
    }

    let click = Closure::wrap(Box::new(move |event: JsValue| {
        if !explorer_visible() {
            return;
        }
        let target = property(&event, "target");
        if fetch_click_target(&target) {
            begin_filter_busy("fetch-click");
        } else if cancel_click_target(&target) {
            schedule_filter_action("cancel-click".to_owned(), 150.0, true);
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3(
        &document(),
        "addEventListener",
        &JsValue::from_str("click"),
        click.as_ref().unchecked_ref(),
        &JsValue::TRUE,
    );
    click.forget();

    for event_name in [
        "kgw:explorer-fetch-complete",
        "kgw:explorer-fetch-failed",
        "kgw:explorer-fetch-cancelled",
        "kgw:transactions-loaded",
        "kgw:tab-opened",
        "kgw:tab-opened-after-mount",
    ] {
        let callback = Closure::wrap(Box::new(move |_event: JsValue| {
            let end = event_name.contains("fetch") || event_name.contains("transactions");
            schedule_filter_action(event_name.to_owned(), 150.0, end);
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &window(),
            "addEventListener",
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
        );
        callback.forget();
    }

    if let Ok(constructor) = property(&global(), "MutationObserver").dyn_into::<Function>() {
        let callback = Closure::wrap(Box::new(move |_records: JsValue, _observer: JsValue| {
            if FILTER_BUSY.with(Cell::get) {
                FILTER_LAST_MUTATION_AT.with(|state| state.set(Date::now()));
            } else {
                schedule_filter_action("dom-mutation".to_owned(), 80.0, false);
            }
        }) as Box<dyn FnMut(JsValue, JsValue)>);
        let args = Array::new();
        args.push(callback.as_ref().unchecked_ref());
        if let Ok(observer) = Reflect::construct(&constructor, &args) {
            let options = Object::new();
            set(options.as_ref(), "childList", &JsValue::TRUE);
            set(options.as_ref(), "subtree", &JsValue::TRUE);
            set(options.as_ref(), "attributes", &JsValue::TRUE);
            let attributes = Array::new();
            for name in ["disabled", "aria-disabled", "style", "class", "value"] {
                attributes.push(&JsValue::from_str(name));
            }
            set(options.as_ref(), "attributeFilter", attributes.as_ref());
            let _ = call2(
                &observer,
                "observe",
                &property(&document(), "documentElement"),
                options.as_ref(),
            );
        }
        callback.forget();
    }

    let periodic = Closure::wrap(Box::new(move || {
        install_filter_invoke_readonly_marker();
        if !FILTER_BUSY.with(Cell::get) {
            refresh_filter_availability("periodic");
        }
    }) as Box<dyn FnMut()>);
    let _ = call2(
        &window(),
        "setInterval",
        periodic.as_ref().unchecked_ref(),
        &JsValue::from_f64(1000.0),
    );
    periodic.forget();

    refresh_filter_availability("install");
    true
}

#[wasm_bindgen(js_name = explorerInstallFilterBusyLock)]
pub fn explorer_install_filter_busy_lock() -> bool {
    install_filter_busy_owner_impl()
}

#[wasm_bindgen(js_name = explorerRefreshFilterAvailability)]
pub fn explorer_refresh_filter_availability(reason: String) {
    refresh_filter_availability(&reason);
}

#[wasm_bindgen(js_name = explorerSetFilterBusy)]
pub fn explorer_set_filter_busy(value: bool, reason: String) {
    if value {
        begin_filter_busy(&reason);
    } else {
        end_filter_busy(&reason);
    }
}

#[wasm_bindgen(js_name = explorerManualAddressValue)]
pub fn explorer_manual_address_value(section: JsValue) -> String {
    manual_address_value_impl(&section)
}

#[wasm_bindgen(js_name = explorerIsKaspaAddress)]
pub fn explorer_is_kaspa_address(value: JsValue) -> bool {
    manual_kaspa_address_text(&crate::js_string_owned(&value))
}

#[wasm_bindgen(js_name = explorerSaveManualAddress)]
pub async fn explorer_save_manual_address(section: JsValue) -> bool {
    save_manual_address_impl(section).await
}

#[wasm_bindgen(js_name = explorerInstallManualAddressSave)]
pub fn explorer_install_manual_address_save() -> bool {
    install_manual_address_save_impl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_size_normalization_matches_legacy_contract() {
        assert_eq!(normalize_font_size_text("11"), 11);
        assert_eq!(normalize_font_size_text("٢٥"), 24);
        assert_eq!(normalize_font_size_text("۵"), 6);
        assert_eq!(normalize_font_size_text("12abc34"), 12);
        assert_eq!(normalize_font_size_text(""), 9);
        assert_eq!(normalize_font_size_text("100"), 10);
    }

    #[test]
    fn busy_allowed_policy_preserves_cancel_variants() {
        assert!(busy_allowed_text("explorerCancel", ""));
        assert!(busy_allowed_text("", "cancel fetch"));
        assert!(busy_allowed_text("", "إلغاء العملية"));
        assert!(busy_allowed_text("", "الغاء العملية"));
        assert!(!busy_allowed_text("explorerFetch", "fetch transactions"));
    }

    #[test]
    fn manual_address_validation_matches_legacy_owner() {
        let main = format!("kaspa:{}", "a".repeat(50));
        let test = format!("kaspatest:{}", "b1".repeat(25));
        let short = format!("kaspa:{}", "a".repeat(49));
        assert!(manual_kaspa_address_text(&main));
        assert!(manual_kaspa_address_text(&test));
        assert!(!manual_kaspa_address_text(&short));
        assert!(!manual_kaspa_address_text(&format!(
            "kaspadev:{}",
            "a".repeat(50)
        )));
        assert!(!manual_kaspa_address_text(&format!(
            "kaspa:{}-",
            "a".repeat(49)
        )));
    }
}
