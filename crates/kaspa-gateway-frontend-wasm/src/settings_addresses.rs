use js_sys::{Array, Date, Function, Intl::NumberFormat, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

#[derive(Default)]
struct AddressState {
    known_names: HashMap<String, String>,
    balances: HashMap<String, Option<f64>>,
    price_usd: f64,
    loading: bool,
    restore_epoch: u64,
}

thread_local! {
    static STATE: RefCell<AddressState> = RefCell::new(AddressState::default());
}

#[derive(Clone)]
struct AddressRow {
    name: String,
    address: String,
    known_name: String,
    balance_kas: Option<f64>,
    value_usd: Option<f64>,
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

fn is_present(value: &JsValue) -> bool {
    !value.is_null() && !value.is_undefined()
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if !is_present(target) {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn set_property(target: &JsValue, name: &str, value: &JsValue) {
    let _ = Reflect::set(target, &JsValue::from_str(name), value);
}
fn function(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}

fn call0(target: &JsValue, name: &str) -> JsValue {
    function(target, name)
        .and_then(|callback| callback.call0(target).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|callback| callback.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn closest(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "closest", &JsValue::from_str(selector))
}
fn by_id(id: &str) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(id))
}

fn query(selector: &str) -> JsValue {
    call1(&document(), "querySelector", &JsValue::from_str(selector))
}

fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(target, "querySelectorAll", &JsValue::from_str(selector));
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(is_present)
        })
        .collect()
}

fn create_element(name: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(name))
}

fn append(parent: &JsValue, child: &JsValue) {
    let _ = call1(parent, "appendChild", child);
}
fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn truthy_text(value: &JsValue) -> String {
    if crate::js_boolean(value) {
        text(value)
    } else {
        String::new()
    }
}

fn error_text(error: &JsValue) -> String {
    let message = property(error, "message");
    if crate::js_boolean(&message) {
        text(&message)
    } else {
        text(error)
    }
}

fn valid_address_text(value: &str) -> bool {
    let text = value.trim();
    text.starts_with("kaspa:") || text.starts_with("kaspatest:")
}

fn first_truthy_text(target: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = property(target, name);
        if crate::js_boolean(&value) {
            return text(&value).trim().to_owned();
        }
    }
    String::new()
}
fn normalize_record(record: &JsValue) -> AddressRow {
    let normalized = crate::settings_contract::settings_address_normalize(record.clone());
    AddressRow {
        name: truthy_text(&property(&normalized, "name")),
        address: truthy_text(&property(&normalized, "address")),
        known_name: String::new(),
        balance_kas: None,
        value_usd: None,
    }
}

fn address_elements() -> (JsValue, JsValue, JsValue, JsValue, JsValue) {
    (
        by_id("settingsAddressName"),
        by_id("settingsAddressValue"),
        by_id("settingsAddressRows"),
        by_id("settingsAddressLastUpdated"),
        by_id("settingsAddressExplorer"),
    )
}

fn now_text() -> String {
    let date = Date::new_0();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}
fn format_number(value: f64, digits: u32) -> String {
    if !value.is_finite() {
        return "--".to_owned();
    }
    let locales = Array::new();
    let options = Object::new();
    set_property(
        options.as_ref(),
        "minimumFractionDigits",
        &JsValue::from_f64(digits as f64),
    );
    set_property(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(digits as f64),
    );
    let formatter = NumberFormat::new(&locales, &options);
    formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(value))
        .ok()
        .map(|value| text(&value))
        .unwrap_or_else(|| format!("{value:.digits$}", digits = digits as usize))
}

fn set_status(message: &str, state: &str) {
    let (_, _, _, last_updated, _) = address_elements();
    if is_present(&last_updated) {
        set_property(&last_updated, "textContent", &JsValue::from_str(message));
    }
    let _ = crate::apply_status_tone_js(last_updated, JsValue::from_str(state));
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "log") {
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW Settings Addresses]"),
            &JsValue::from_str(message),
        );
    }
}

fn warn(parts: &[JsValue]) {
    let console = property(&global(), "console");
    if let Some(callback) = function(&console, "warn") {
        match parts {
            [a] => {
                let _ = callback.call1(&console, a);
            }
            [a, b] => {
                let _ = callback.call2(&console, a, b);
            }
            [a, b, c] => {
                let _ = callback.call3(&console, a, b, c);
            }
            _ => {}
        }
    }
}

fn invoke_api() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    if let Ok(invoke) = property(&core, "invoke").dyn_into::<Function>() {
        return Some(invoke);
    }
    let legacy = property(&tauri, "tauri");
    if let Ok(invoke) = property(&legacy, "invoke").dyn_into::<Function>() {
        return Some(invoke);
    }
    property(&window(), "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .ok()
}

async fn invoke_command(command: &str, args: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let invoke =
        invoke_api().ok_or_else(|| JsValue::from_str("Tauri invoke API is not available."))?;
    let result = match args {
        Some(args) => invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?,
        None => invoke.call1(&JsValue::UNDEFINED, &JsValue::from_str(command))?,
    };
    JsFuture::from(Promise::resolve(&result)).await
}

fn records_from_container(value: &JsValue) -> Vec<JsValue> {
    if Array::is_array(value) {
        return Array::from(value).iter().collect();
    }
    for key in ["addresses", "items"] {
        let nested = property(value, key);
        if Array::is_array(&nested) {
            return Array::from(&nested).iter().collect();
        }
    }
    Vec::new()
}
async fn load_known_names() {
    let Ok(value) = invoke_command("settings_fetch_address_names", None).await else {
        warn(&[JsValue::from_str(
            "[KGW Settings Addresses] known names load failed",
        )]);
        return;
    };
    let mut names = HashMap::new();
    for item in records_from_container(&value) {
        let address = first_truthy_text(&item, &["address", "Address"]);
        let name = first_truthy_text(
            &item,
            &["name", "Name", "known_name", "knownName", "label", "Label"],
        );
        if valid_address_text(&address) && !name.is_empty() {
            names.insert(address, name);
        }
    }
    STATE.with(|state| state.borrow_mut().known_names = names);
}

fn first_truthy_number(values: &[JsValue]) -> f64 {
    for value in values {
        if crate::js_boolean(value) {
            return crate::js_number(value);
        }
    }
    0.0
}
async fn load_price() {
    match invoke_command("kgw_get_kaspa_prices", None).await {
        Ok(prices) => {
            let nested_prices = property(&prices, "prices");
            let values = property(&prices, "values");
            let usd = first_truthy_number(&[
                property(&nested_prices, "usd"),
                property(&prices, "usd"),
                property(&values, "USD"),
                property(&values, "usd"),
            ]);
            STATE.with(|state| {
                state.borrow_mut().price_usd = if usd.is_finite() { usd } else { 0.0 };
            });
        }
        Err(error) => warn(&[
            JsValue::from_str("[KGW Settings Addresses] price load failed"),
            error,
        ]),
    }
}

fn first_present_number(target: &JsValue, names: &[&str]) -> f64 {
    for name in names {
        let value = property(target, name);
        if is_present(&value) {
            return crate::js_number(&value);
        }
    }
    0.0
}
async fn load_balance(address: &str) -> Option<f64> {
    if !valid_address_text(address) {
        return None;
    }
    if let Some(value) = STATE.with(|state| state.borrow().balances.get(address).copied()) {
        return value;
    }

    let args = Object::new();
    set_property(args.as_ref(), "address", &JsValue::from_str(address));
    match invoke_command("explorer_fetch_balance", Some(args.as_ref())).await {
        Ok(result) => {
            let value =
                first_present_number(&result, &["balance_kas", "balanceKas", "kas", "balance"]);
            let normalized = if value.is_finite() { value } else { 0.0 };
            STATE.with(|state| {
                state
                    .borrow_mut()
                    .balances
                    .insert(address.to_owned(), Some(normalized));
            });
            Some(normalized)
        }
        Err(error) => {
            warn(&[
                JsValue::from_str("[KGW Settings Addresses] balance fetch failed"),
                JsValue::from_str(address),
                error,
            ]);
            STATE.with(|state| {
                state.borrow_mut().balances.insert(address.to_owned(), None);
            });
            None
        }
    }
}

fn normalized_rows(records: &JsValue) -> Vec<AddressRow> {
    if !Array::is_array(records) {
        return Vec::new();
    }
    Array::from(records)
        .iter()
        .map(|record| normalize_record(&record))
        .filter(|row| valid_address_text(&row.address))
        .collect()
}

async fn enrich_rows(records: &JsValue) -> Vec<AddressRow> {
    load_known_names().await;
    load_price().await;
    let mut rows = normalized_rows(records);
    for row in &mut rows {
        row.known_name = STATE.with(|state| {
            state
                .borrow()
                .known_names
                .get(&row.address)
                .cloned()
                .unwrap_or_default()
        });
        row.balance_kas = load_balance(&row.address).await;
        row.value_usd = row
            .balance_kas
            .map(|balance| STATE.with(|state| balance * state.borrow().price_usd));
    }
    rows
}
fn translated_empty_text() -> String {
    let translator = property(&window(), "kgwT");
    if let Ok(translator) = translator.dyn_into::<Function>()
        && let Ok(value) =
            translator.call1(&window(), &JsValue::from_str("settings.noSavedAddresses"))
    {
        let translated = text(&value);
        if !translated.is_empty() {
            return translated;
        }
    }
    "No saved addresses.".to_owned()
}

fn set_selected_row(row: &AddressRow) {
    let (name, address, _, _, explorer) = address_elements();
    if is_present(&name) {
        set_property(&name, "value", &JsValue::from_str(&row.name));
    }
    if is_present(&address) {
        set_property(&address, "value", &JsValue::from_str(&row.address));
    }
    if is_present(&explorer) {
        let disabled = row.address.is_empty();
        set_property(&explorer, "disabled", &JsValue::from_bool(disabled));
        let classes = property(&explorer, "classList");
        if let Some(toggle) = function(&classes, "toggle") {
            let _ = toggle.call2(
                &classes,
                &JsValue::from_str("disabled-btn"),
                &JsValue::from_bool(disabled),
            );
        }
    }
}
fn restore_epoch_from_options(options: &JsValue) -> u64 {
    let supplied = property(options, "restoreEpoch");
    if is_present(&supplied) {
        let value = crate::js_number(&supplied);
        if value.is_finite() && value >= 0.0 {
            return value as u64;
        }
    }
    STATE.with(|state| state.borrow().restore_epoch)
}

async fn render_rows_internal(records: JsValue, options: JsValue) -> bool {
    let (_, _, rows_node, _, _) = address_elements();
    if !is_present(&rows_node) {
        return false;
    }
    let epoch = restore_epoch_from_options(&options);
    let local_only = property(&options, "localOnly").as_bool() == Some(true);
    let rows = if local_only {
        normalized_rows(&records)
    } else {
        enrich_rows(&records).await
    };
    if epoch != STATE.with(|state| state.borrow().restore_epoch) {
        return false;
    }
    set_property(&rows_node, "innerHTML", &JsValue::from_str(""));
    if rows.is_empty() {
        let tr = create_element("tr");
        let td = create_element("td");
        set_property(&td, "colSpan", &JsValue::from_f64(5.0));
        set_property(
            &td,
            "textContent",
            &JsValue::from_str(&translated_empty_text()),
        );
        append(&tr, &td);
        append(&rows_node, &tr);
        return true;
    }
    for row in rows {
        let tr = create_element("tr");
        set_property(
            &property(&tr, "dataset"),
            "address",
            &JsValue::from_str(&row.address),
        );
        set_property(
            &property(&tr, "style"),
            "cursor",
            &JsValue::from_str("pointer"),
        );
        let balance = row
            .balance_kas
            .map(|value| format_number(value, 2))
            .unwrap_or_else(|| "--".to_owned());
        let value = row
            .value_usd
            .map(|value| format!("{} USD", format_number(value, 2)))
            .unwrap_or_else(|| "--".to_owned());
        for cell in [
            row.name.clone(),
            row.address.clone(),
            row.known_name.clone(),
            balance,
            value,
        ] {
            let td = create_element("td");
            set_property(&td, "textContent", &JsValue::from_str(&cell));
            append(&tr, &td);
        }
        let selected_row = row.clone();
        let rows_for_click = rows_node.clone();
        let tr_for_click = tr.clone();
        let click = Closure::<dyn FnMut(JsValue)>::new(move |_event| {
            for node in query_all(&rows_for_click, "tr") {
                let _ = call1(
                    &property(&node, "classList"),
                    "remove",
                    &JsValue::from_str("is-selected"),
                );
            }
            let _ = call1(
                &property(&tr_for_click, "classList"),
                "add",
                &JsValue::from_str("is-selected"),
            );
            set_selected_row(&selected_row);
        });
        if let Some(listener) = function(&tr, "addEventListener") {
            let _ = listener.call2(
                &tr,
                &JsValue::from_str("click"),
                click.as_ref().unchecked_ref(),
            );
        }
        click.forget();
        append(&rows_node, &tr);
    }
    true
}

fn refresh_saved_addresses() {
    let callback = property(&window(), "kgwRefreshSavedAddresses");
    if let Ok(callback) = callback.dyn_into::<Function>() {
        let _ = callback.call0(&window());
    }
}
async fn refresh_internal() -> JsValue {
    if STATE.with(|state| state.borrow().loading) {
        return Array::new().into();
    }
    if invoke_api().is_none() {
        set_status("Last Updated: Tauri invoke API is not available.", "error");
        return Array::new().into();
    }
    let epoch = STATE.with(|state| state.borrow().restore_epoch);
    STATE.with(|state| state.borrow_mut().loading = true);
    set_status("Last Updated: loading...", "loading");

    let result = invoke_command("get_all_addresses", None).await;
    let output = match result {
        Ok(records) => {
            if epoch == STATE.with(|state| state.borrow().restore_epoch) {
                let options = Object::new();
                set_property(
                    options.as_ref(),
                    "restoreEpoch",
                    &JsValue::from_f64(epoch as f64),
                );
                if render_rows_internal(records.clone(), options.into()).await {
                    set_status(&format!("Last Updated: {}", now_text()), "success");
                    refresh_saved_addresses();
                    records
                } else {
                    Array::new().into()
                }
            } else {
                Array::new().into()
            }
        }
        Err(error) => {
            if epoch == STATE.with(|state| state.borrow().restore_epoch) {
                let options = Object::new();
                set_property(
                    options.as_ref(),
                    "restoreEpoch",
                    &JsValue::from_f64(epoch as f64),
                );
                let _ = render_rows_internal(Array::new().into(), options.into()).await;
                set_status(
                    &format!("Last Updated: failed - {}", error_text(&error)),
                    "error",
                );
            }
            Array::new().into()
        }
    };
    STATE.with(|state| state.borrow_mut().loading = false);
    output
}
fn notify_saved_addresses_changed() {
    let constructor = property(&global(), "CustomEvent");
    if let Ok(constructor) = constructor.dyn_into::<Function>() {
        let args = Array::new();
        args.push(&JsValue::from_str("kgw:saved-addresses-changed"));
        if let Ok(event) = Reflect::construct(&constructor, &args) {
            let _ = call1(&window(), "dispatchEvent", &event);
        }
    }
}

fn clear_fields_internal() {
    let (name, address, _, _, explorer) = address_elements();
    if is_present(&name) {
        set_property(&name, "value", &JsValue::from_str(""));
    }
    if is_present(&address) {
        set_property(&address, "value", &JsValue::from_str(""));
    }
    if is_present(&explorer) {
        set_property(&explorer, "disabled", &JsValue::TRUE);
        let _ = call1(
            &property(&explorer, "classList"),
            "add",
            &JsValue::from_str("disabled-btn"),
        );
    }
}

async fn save_internal() {
    let (name, address, _, _, _) = address_elements();
    if invoke_api().is_none() {
        set_status("Last Updated: Tauri invoke API is not available.", "error");
        return;
    }
    let clean_address = truthy_text(&property(&address, "value")).trim().to_owned();
    let clean_name = truthy_text(&property(&name, "value")).trim().to_owned();
    if !valid_address_text(&clean_address) {
        set_status("Last Updated: invalid Kaspa address.", "error");
        return;
    }
    set_status("Last Updated: saving...", "loading");
    let args = Object::new();
    set_property(args.as_ref(), "address", &JsValue::from_str(&clean_address));
    set_property(args.as_ref(), "name", &JsValue::from_str(&clean_name));
    match invoke_command("save_address", Some(args.as_ref())).await {
        Ok(_) => {
            STATE.with(|state| {
                state.borrow_mut().balances.remove(&clean_address);
            });
            let _ = refresh_internal().await;
            notify_saved_addresses_changed();
        }
        Err(error) => set_status(
            &format!("Last Updated: save failed - {}", error_text(&error)),
            "error",
        ),
    }
}

async fn delete_internal() {
    let (_, address, _, _, _) = address_elements();
    if invoke_api().is_none() {
        set_status("Last Updated: Tauri invoke API is not available.", "error");
        return;
    }
    let clean_address = truthy_text(&property(&address, "value")).trim().to_owned();
    if !valid_address_text(&clean_address) {
        set_status("Last Updated: select a saved Kaspa address first.", "info");
        return;
    }
    set_status("Last Updated: deleting...", "loading");
    let args = Object::new();
    set_property(args.as_ref(), "address", &JsValue::from_str(&clean_address));
    match invoke_command("delete_saved_address", Some(args.as_ref())).await {
        Ok(_) => {
            STATE.with(|state| {
                state.borrow_mut().balances.remove(&clean_address);
            });
            clear_fields_internal();
            let _ = refresh_internal().await;
            notify_saved_addresses_changed();
        }
        Err(error) => set_status(
            &format!("Last Updated: delete failed - {}", error_text(&error)),
            "error",
        ),
    }
}

fn stringify(value: &JsValue) -> String {
    JSON::stringify(value)
        .ok()
        .map(|value| text(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned())
}
fn trace_action(event: &JsValue, action: &str, button: &JsValue) {
    let details = Object::new();
    set_property(
        details.as_ref(),
        "trusted",
        &JsValue::from_bool(crate::js_boolean(&property(event, "isTrusted"))),
    );
    set_property(details.as_ref(), "action", &JsValue::from_str(action));
    set_property(
        details.as_ref(),
        "text",
        &JsValue::from_str(truthy_text(&property(button, "textContent")).trim()),
    );
    let args = Object::new();
    set_property(args.as_ref(), "scope", &JsValue::from_str("settings"));
    set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    set_property(
        args.as_ref(),
        "action",
        &JsValue::from_str("settings-addresses"),
    );
    set_property(
        args.as_ref(),
        "phase",
        &JsValue::from_str("r48b3-address-action-click"),
    );
    set_property(
        args.as_ref(),
        "details",
        &JsValue::from_str(&stringify(details.as_ref())),
    );
    if let Some(invoke) = invoke_api()
        && let Ok(result) = invoke.call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str("kgw_frontend_button_trace_v1"),
            args.as_ref(),
        )
    {
        let promise = Promise::resolve(&result);
        let catch = Closure::<dyn FnMut(JsValue)>::new(move |_error| {});
        let _ = promise.catch(&catch);
        catch.forget();
    }
}

fn schedule_refresh(delay_ms: f64) {
    let callback = Closure::<dyn FnMut()>::new(move || {
        spawn_local(async {
            let _ = refresh_internal().await;
        });
    });
    if let Some(timeout) = function(&window(), "setTimeout") {
        let _ = timeout.call2(
            &window(),
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(delay_ms),
        );
    }
    callback.forget();
}
#[wasm_bindgen(js_name = settingsAddressesInstallManage)]
pub fn install_manage() -> Result<(), JsValue> {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwSettingsManageAddressesInstalled")) {
        return Ok(());
    }
    set_property(
        &win,
        "__kgwSettingsManageAddressesInstalled",
        &JsValue::TRUE,
    );

    let actions = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        let add = closest(&target, "#settingsAddressAdd");
        let del = closest(&target, "#settingsAddressDelete");
        let clear = closest(&target, "#settingsAddressClear");
        let refresh = closest(&target, "#settingsAddressRefresh");
        if ![&add, &del, &clear, &refresh]
            .iter()
            .any(|node| is_present(node))
        {
            return;
        }
        if let Some(prevent) = function(&event, "preventDefault") {
            let _ = prevent.call0(&event);
        }
        if let Some(stop) = function(&event, "stopImmediatePropagation") {
            let _ = stop.call0(&event);
        }
        let (action, button) = if is_present(&add) {
            ("add", add)
        } else if is_present(&del) {
            ("delete", del)
        } else if is_present(&clear) {
            ("clear", clear)
        } else {
            ("refresh", refresh)
        };
        trace_action(&event, action, &button);
        match action {
            "add" => spawn_local(save_internal()),
            "delete" => spawn_local(delete_internal()),
            "clear" => clear_fields_internal(),
            "refresh" => {
                STATE.with(|state| state.borrow_mut().balances.clear());
                spawn_local(async {
                    let _ = refresh_internal().await;
                });
            }
            _ => {}
        }
    });
    function(&document(), "addEventListener")
        .ok_or_else(|| JsValue::from_str("document.addEventListener unavailable"))?
        .call3(
            &document(),
            &JsValue::from_str("click"),
            actions.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )?;
    actions.forget();

    let tab = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        if is_present(&closest(
            &target,
            "[data-settings-tab=\"manage-addresses\"]",
        )) {
            schedule_refresh(100.0);
        }
    });
    function(&document(), "addEventListener")
        .ok_or_else(|| JsValue::from_str("document.addEventListener unavailable"))?
        .call3(
            &document(),
            &JsValue::from_str("click"),
            tab.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )?;
    tab.forget();
    schedule_refresh(500.0);
    Ok(())
}

#[wasm_bindgen(js_name = settingsAddressesRefresh)]
pub async fn refresh() -> JsValue {
    refresh_internal().await
}

#[wasm_bindgen(js_name = settingsAddressesRenderRows)]
pub async fn render_rows(records: JsValue, options: JsValue) -> bool {
    render_rows_internal(records, options).await
}

fn selected_input_address() -> String {
    for selector in [
        "#settingsAddressValue",
        "#settingsAddressAddress",
        "#settingsAddressInput",
    ] {
        let node = query(selector);
        let value = truthy_text(&property(&node, "value")).trim().to_owned();
        if !value.is_empty() {
            return value;
        }
    }
    String::new()
}

fn explorer_url(address: &str) -> String {
    let clean = address.trim();
    if valid_address_text(clean) {
        format!("https://explorer.kaspa.org/addresses/{clean}")
    } else {
        String::new()
    }
}

fn open_explorer(address: &str) -> bool {
    let url = explorer_url(address);
    if url.is_empty() {
        warn(&[
            JsValue::from_str("[KGW Settings Addresses] invalid address for explorer open"),
            JsValue::from_str(address.trim()),
        ]);
        return false;
    }
    function(&window(), "open")
        .and_then(|open| {
            open.call3(
                &window(),
                &JsValue::from_str(&url),
                &JsValue::from_str("_blank"),
                &JsValue::from_str("noopener,noreferrer"),
            )
            .ok()
        })
        .is_some()
}

async fn delete_transactions_internal() {
    let address = selected_input_address();
    if address.is_empty() {
        set_status("Last Updated: select an address first", "info");
        return;
    }
    let prompt = format!(
        "Clear all cached transactions for this address?\n\n{address}\n\nThe saved address will remain."
    );
    let confirmed = crate::settings_contract::confirm_user_action(JsValue::from_str(&prompt))
        .await
        .unwrap_or(false);
    if !confirmed {
        return;
    }

    let button = by_id("settingsAddressDeleteTransactions");
    if is_present(&button) {
        set_property(&button, "disabled", &JsValue::TRUE);
    }
    let args = Object::new();
    set_property(args.as_ref(), "address", &JsValue::from_str(&address));

    match invoke_command(
        "explorer_delete_transactions_for_address",
        Some(args.as_ref()),
    )
    .await
    {
        Ok(deleted) => {
            set_status(
                &format!(
                    "Last Updated: cleared {} cached transactions",
                    text(&deleted)
                ),
                "success",
            );
            let refresh = by_id("settingsAddressRefresh");
            if is_present(&refresh) {
                let _ = call0(&refresh, "click");
            }
        }
        Err(error) => {
            set_status(
                &format!(
                    "Last Updated: clear transactions failed - {}",
                    error_text(&error)
                ),
                "error",
            );
        }
    }
    if is_present(&button) {
        set_property(&button, "disabled", &JsValue::FALSE);
    }
}

#[wasm_bindgen(js_name = settingsAddressesInstallExplorerOpen)]
pub fn install_explorer_open() -> Result<(), JsValue> {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwSettingsAddressExplorerOpenInstalled")) {
        return Ok(());
    }
    set_property(
        &win,
        "__kgwSettingsAddressExplorerOpenInstalled",
        &JsValue::TRUE,
    );
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        let button = closest(&target, "#settingsAddressExplorer");
        if !is_present(&button) {
            return;
        }
        let _ = call0(&event, "preventDefault");
        let _ = call0(&event, "stopImmediatePropagation");
        let address = selected_input_address();
        if !address.is_empty() {
            let _ = open_explorer(&address);
        }
    });

    function(&document(), "addEventListener")
        .ok_or_else(|| JsValue::from_str("document.addEventListener unavailable"))?
        .call3(
            &document(),
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )?;
    callback.forget();
    Ok(())
}

#[wasm_bindgen(js_name = settingsAddressesInstallDeleteTransactions)]
pub fn install_delete_transactions() -> Result<(), JsValue> {
    let win = window();
    if crate::js_boolean(&property(
        &win,
        "__kgwSettingsAddressDeleteTransactionsInstalled",
    )) {
        return Ok(());
    }
    set_property(
        &win,
        "__kgwSettingsAddressDeleteTransactionsInstalled",
        &JsValue::TRUE,
    );
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        let button = closest(&target, "#settingsAddressDeleteTransactions");

        if !is_present(&button) {
            return;
        }
        let _ = call0(&event, "preventDefault");
        spawn_local(delete_transactions_internal());
    });
    function(&document(), "addEventListener")
        .ok_or_else(|| JsValue::from_str("document.addEventListener unavailable"))?
        .call3(
            &document(),
            &JsValue::from_str("click"),
            callback.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )?;
    callback.forget();
    Ok(())
}

#[wasm_bindgen(js_name = settingsAddressesClearFields)]
pub fn clear_fields() {
    clear_fields_internal();
}

#[wasm_bindgen(js_name = settingsAddressesSetStatus)]
pub fn set_status_export(message: String, state: String) {
    set_status(&message, if state.is_empty() { "info" } else { &state });
}

#[wasm_bindgen(js_name = settingsAddressesNow)]
pub fn now() -> String {
    now_text()
}

#[wasm_bindgen(js_name = settingsAddressesIncrementRestoreEpoch)]
pub fn increment_restore_epoch() -> u64 {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.restore_epoch = state.restore_epoch.saturating_add(1);
        state.restore_epoch
    })
}

#[wasm_bindgen(js_name = settingsAddressesRestoreEpoch)]
pub fn restore_epoch() -> u64 {
    STATE.with(|state| state.borrow().restore_epoch)
}

#[wasm_bindgen(js_name = settingsAddressesOpenExplorer)]
pub fn open_explorer_export(address: String) -> bool {
    open_explorer(&address)
}

#[wasm_bindgen(js_name = settingsAddressesInstallAll)]
pub fn install_all() -> Result<(), JsValue> {
    install_manage()?;
    install_explorer_open()?;
    install_delete_transactions()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_validation_matches_legacy_scope() {
        assert!(valid_address_text("kaspa:q123"));
        assert!(valid_address_text("kaspatest:q123"));
        assert!(!valid_address_text("bitcoin:q123"));
    }

    #[test]
    fn explorer_url_requires_kaspa_family_address() {
        assert_eq!(
            explorer_url("kaspa:qabc"),
            "https://explorer.kaspa.org/addresses/kaspa:qabc"
        );
        assert_eq!(explorer_url("btc:qabc"), "");
    }
}
