use js_sys::{Array, Date, Function, Intl::NumberFormat, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;

const SOMPI_PER_KAS: f64 = 100_000_000.0;

#[derive(Default)]
struct AddressState {
    known_names: HashMap<String, String>,
    names_loaded: bool,
    top_names_loaded: bool,
    saved_count: usize,
    save_memo: HashMap<String, f64>,
}

thread_local! {
    static STATE: RefCell<AddressState> = RefCell::new(AddressState::default());
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

fn call1(target: &JsValue, name: &str, value: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|function| function.call1(target, value).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn query(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "querySelector", &JsValue::from_str(selector))
}

fn root() -> JsValue {
    let doc = document();
    let by_id = call1(&doc, "getElementById", &JsValue::from_str("explorer"));
    if present(&by_id) {
        return by_id;
    }
    let by_class = query(&doc, ".explorer-python-root");
    if present(&by_class) { by_class } else { doc }
}

fn class_add(target: &JsValue, class_name: &str) {
    let class_list = property(target, "classList");
    let _ = call1(&class_list, "add", &JsValue::from_str(class_name));
}

fn create_element(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag))
}

fn append_child(parent: &JsValue, child: &JsValue) {
    let _ = call1(parent, "appendChild", child);
}

fn js_text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn error_text(error: &JsValue) -> String {
    let message = property(error, "message");
    let raw = if present(&message) {
        js_text(&message)
    } else {
        js_text(error)
    };
    let clean = raw.replace(['\r', '\n', '\t'], " ").trim().to_owned();
    if clean.is_empty() {
        "unknown error".to_owned()
    } else {
        clean
    }
}

fn diagnostic(name: &str, label: &str, details: &JsValue) {
    match name {
        "log" => {
            crate::explorer_microscope::explorer_microscope_log(label.to_owned(), details.clone())
        }
        "warn" => {
            crate::explorer_microscope::explorer_microscope_warn(label.to_owned(), details.clone())
        }
        "error" => {
            let message = js_text(&property(details, "message"));
            crate::explorer_microscope::explorer_microscope_error(
                label.to_owned(),
                JsValue::from_str(&message),
                details.clone(),
            );
        }
        "apiShape" => crate::explorer_microscope::explorer_microscope_api_shape(
            label.to_owned(),
            details.clone(),
        ),
        _ => {}
    }
}

fn diagnostic_message(name: &str, label: &str, message: &str) {
    let details = Object::new();
    set(details.as_ref(), "message", &JsValue::from_str(message));
    diagnostic(name, label, details.as_ref());
}

async fn await_js(value: JsValue) -> Result<JsValue, JsValue> {
    JsFuture::from(Promise::resolve(&value)).await
}

async fn invoke_tauri(command: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let tauri = property(&window(), "__TAURI__");
    for owner in [
        property(&tauri, "core"),
        property(&tauri, "tauri"),
        tauri.clone(),
    ] {
        if let Some(invoke) = function(&owner, "invoke") {
            let result = invoke.call2(&owner, &JsValue::from_str(command), args)?;
            return await_js(result).await;
        }
    }
    let direct = property(&window(), "__TAURI_INVOKE__");
    if let Ok(invoke) = direct.dyn_into::<Function>() {
        let result = invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?;
        return await_js(result).await;
    }
    Err(JsValue::from_str("Tauri invoke API is not available."))
}

async fn invoke_first(candidates: Vec<(&'static str, JsValue)>) -> Result<JsValue, JsValue> {
    let mut last_error = None;
    for (command, args) in candidates {
        match invoke_tauri(command, &args).await {
            Ok(value) => return Ok(value),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| JsValue::from_str("No Tauri command candidate succeeded.")))
}

fn empty_args() -> JsValue {
    Object::new().into()
}

fn args1(name: &str, value: JsValue) -> JsValue {
    let args = Object::new();
    set(args.as_ref(), name, &value);
    args.into()
}

fn nested_args(name: &str, field: &str, value: JsValue) -> JsValue {
    let nested = Object::new();
    set(nested.as_ref(), field, &value);
    args1(name, nested.into())
}

fn normalize_address_text(value: &str) -> String {
    value.trim().to_owned()
}

fn split_network_prefix(value: &str) -> Option<&str> {
    for prefix in ["kaspatest:", "kaspadev:", "kaspasim:", "kaspa:"] {
        if value
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
        {
            return value.get(prefix.len()..);
        }
    }
    None
}

fn is_kaspa_address_text(value: &str) -> bool {
    let clean = normalize_address_text(value);
    let Some(tail) = split_network_prefix(&clean) else {
        return false;
    };
    tail.len() >= 50 && tail.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn lookup_keys_text(address: &str) -> Vec<String> {
    let clean = normalize_address_text(address);
    let lower = clean.to_lowercase();
    let no_prefix = split_network_prefix(&clean).unwrap_or(&clean).to_owned();
    let no_prefix_lower = no_prefix.to_lowercase();
    let mut output = Vec::new();
    for value in [
        clean,
        lower,
        no_prefix.clone(),
        no_prefix_lower.clone(),
        if no_prefix.is_empty() {
            String::new()
        } else {
            format!("kaspa:{no_prefix}")
        },
        if no_prefix_lower.is_empty() {
            String::new()
        } else {
            format!("kaspa:{no_prefix_lower}")
        },
    ] {
        if !value.is_empty() && !output.iter().any(|current| current == &value) {
            output.push(value);
        }
    }
    output
}

fn store_known_name(address: &str, name: &str) {
    let clean_name = name.trim();
    if clean_name.is_empty() {
        return;
    }
    let keys = lookup_keys_text(address);
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for key in keys {
            state.known_names.insert(key, clean_name.to_owned());
        }
    });
}
fn known_name(address: &str) -> Option<String> {
    let keys = lookup_keys_text(address);
    STATE.with(|state| {
        let state = state.borrow();
        keys.iter()
            .find_map(|key| state.known_names.get(key).cloned())
    })
}

fn first_text(target: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = property(target, name);
        if present(&value) {
            let text = js_text(&value).trim().to_owned();
            if !text.is_empty() {
                return text;
            }
        }
    }
    String::new()
}

fn array_items(value: &JsValue) -> Vec<JsValue> {
    if Array::is_array(value) {
        Array::from(value).iter().collect()
    } else {
        Vec::new()
    }
}
fn list_from_shape(value: &JsValue, names: &[&str]) -> Vec<JsValue> {
    if Array::is_array(value) {
        return array_items(value);
    }
    for name in names {
        let candidate = property(value, name);
        if Array::is_array(&candidate) {
            return array_items(&candidate);
        }
    }
    Vec::new()
}

fn ingest_known_name_entry(address: &str, value: &JsValue) {
    let name = if let Some(text) = value.as_string() {
        text
    } else {
        first_text(
            value,
            &["name", "known_name", "KnownName", "label", "display_name"],
        )
    };
    store_known_name(address, &name);
}

fn ingest_known_names(value: &JsValue) {
    if value.is_object() && !Array::is_array(value) {
        let object = Object::from(value.clone());
        for entry in Object::entries(&object).iter() {
            let pair = Array::from(&entry);
            if pair.length() >= 2 {
                ingest_known_name_entry(&js_text(&pair.get(0)), &pair.get(1));
            }
        }
    }

    for item in list_from_shape(value, &["addresses", "items", "names"]) {
        if Array::is_array(&item) {
            let pair = Array::from(&item);
            if pair.length() >= 2 {
                store_known_name(&js_text(&pair.get(0)), &js_text(&pair.get(1)));
            }
            continue;
        }
        if item.is_object() {
            let address = first_text(&item, &["address", "Address", "addr", "kaspa_address"]);
            let name = first_text(
                &item,
                &[
                    "name",
                    "known_name",
                    "KnownName",
                    "Known Name",
                    "label",
                    "display_name",
                ],
            );
            store_known_name(&address, &name);
        }
    }
}

pub(crate) fn invalidate_address_names() {
    STATE.with(|state| state.borrow_mut().names_loaded = false);
}
async fn load_known_names_internal() {
    let already_loaded = STATE.with(|state| state.borrow().names_loaded);
    diagnostic_message(
        "log",
        "NAMES LOAD START",
        if already_loaded { "cached" } else { "load" },
    );
    if already_loaded {
        return;
    }
    STATE.with(|state| state.borrow_mut().names_loaded = true);

    let candidates = vec![
        ("top_addresses_load_known_names", empty_args()),
        ("get_all_addresses", empty_args()),
    ];
    match invoke_first(candidates).await {
        Ok(value) => {
            diagnostic("apiShape", "NAMES API RAW SHAPE", &value);
            ingest_known_names(&value);
        }
        Err(error) => {
            diagnostic_message("warn", "NAMES LOAD FAILED", &error_text(&error));
        }
    }
}

async fn load_top_names_internal() {
    let already_loaded = STATE.with(|state| state.borrow().top_names_loaded);
    if already_loaded {
        return;
    }
    STATE.with(|state| state.borrow_mut().top_names_loaded = true);

    let direct = args1("limit", JsValue::from_f64(10_000.0));
    let nested = nested_args("request", "limit", JsValue::from_f64(10_000.0));
    match invoke_first(vec![
        ("fetch_top_addresses_rust", direct),
        ("fetch_top_addresses_rust", nested),
    ])
    .await
    {
        Ok(value) => {
            let rows = list_from_shape(&value, &["rows"]);
            let mut stored = 0_u32;
            for row in rows {
                let address = first_text(&row, &["address", "Address"]);
                let name = first_text(
                    &row,
                    &["known_name", "KnownName", "Known Name", "name", "label"],
                );
                if !address.is_empty() && !name.is_empty() {
                    store_known_name(&address, &name);
                    stored += 1;
                }
            }
            let details = Object::new();
            set(
                details.as_ref(),
                "stored",
                &JsValue::from_f64(stored as f64),
            );
            diagnostic("log", "TOP NAME SOURCE LOAD DONE", details.as_ref());
        }
        Err(error) => {
            diagnostic_message("warn", "TOP NAME SOURCE LOAD FAILED", &error_text(&error));
        }
    }
}

async fn resolve_known_name_internal(address: &str) -> String {
    load_known_names_internal().await;
    if let Some(name) = known_name(address) {
        return name;
    }
    load_top_names_internal().await;
    known_name(address).unwrap_or_default()
}

fn memo_key(address: &str, name: &str) -> String {
    format!("{}|{}", address.trim(), name.trim())
}

fn should_skip_save(address: &str, name: &str) -> bool {
    let key = memo_key(address, name);
    let now = Date::now();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let last = state.save_memo.get(&key).copied().unwrap_or(0.0);
        if now - last < 10_000.0 {
            true
        } else {
            state.save_memo.insert(key, now);
            false
        }
    })
}

pub(crate) async fn save_address_to_database_internal(
    address: &str,
    name: &str,
) -> Result<JsValue, JsValue> {
    let clean_address = normalize_address_text(address);
    let clean_name = name.trim().to_owned();
    if !is_kaspa_address_text(&clean_address) || should_skip_save(&clean_address, &clean_name) {
        return Ok(JsValue::from_str(""));
    }
    let args = Object::new();
    set(args.as_ref(), "address", &JsValue::from_str(&clean_address));
    set(args.as_ref(), "name", &JsValue::from_str(&clean_name));
    match invoke_tauri("save_address", args.as_ref()).await {
        Ok(value) => Ok(value),
        Err(error) => {
            diagnostic_message("error", "SAVE ADDRESS FAILED", &error_text(&error));
            Err(error)
        }
    }
}

fn add_class_by_id(section: &JsValue, id: &str, class_name: &str) {
    let node = query(section, &format!("#{id}"));
    if present(&node) {
        class_add(&node, class_name);
    }
}

fn apply_python_load_address_style(section: &JsValue) {
    let scope = if present(section) {
        section.clone()
    } else {
        root()
    };
    let input = query(&scope, "#explorerAddress");
    let balance = query(&scope, "#explorerBalanceValue");
    if !present(&input) || !present(&balance) {
        return;
    }
    let fieldset = call1(&input, "closest", &JsValue::from_str("fieldset"));
    if !present(&fieldset) {
        return;
    }
    class_add(&fieldset, "kgw-python-load-address-owner");
    for id in [
        "explorerFetch",
        "explorerForceFetch",
        "explorerOpenExplorer",
        "explorerCancel",
    ] {
        add_class_by_id(&scope, id, "kgw-python-load-address-button");
    }
    class_add(&balance, "kgw-python-balance-kas");
    let usd = query(&scope, "#explorerBalanceUsdValue");
    if present(&usd) {
        class_add(&usd, "kgw-python-balance-fiat");
    }
    let name = query(&scope, "#explorerAddressNameValue");
    if present(&name) {
        class_add(&name, "kgw-python-wallet-name");
    }
}

fn format_balance_kas(value: f64) -> String {
    let locales = Array::new();
    let options = Object::new();
    set(
        options.as_ref(),
        "minimumFractionDigits",
        &JsValue::from_f64(0.0),
    );
    set(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(3.0),
    );
    let formatter = NumberFormat::new(&locales, &options);
    let formatted = formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(value))
        .ok()
        .map(|value| js_text(&value))
        .unwrap_or_else(|| format!("{value:.3}"));
    format!("{formatted} KAS")
}

fn set_balance(section: &JsValue, balance: Option<f64>) {
    let scope = if present(section) {
        section.clone()
    } else {
        root()
    };
    let node = query(&scope, "#explorerBalanceValue");
    if present(&node) {
        let text = balance
            .filter(|value| value.is_finite())
            .map(format_balance_kas)
            .unwrap_or_else(|| "N/A".to_owned());
        set(&node, "textContent", &JsValue::from_str(&text));
    }

    let usd_node = query(&scope, "#explorerBalanceUsdValue");
    if present(&usd_node) {
        let price = crate::parse_header_usd_price().unwrap_or(0.0);
        if let Some(value) = balance.filter(|value| value.is_finite() && *value > 0.0)
            && price > 0.0
        {
            let usd = crate::format_usd(JsValue::from_f64(value * price));
            set(
                &usd_node,
                "textContent",
                &JsValue::from_str(&format!("({usd} USD)")),
            );
            set(&usd_node, "hidden", &JsValue::FALSE);
            set(
                &property(&usd_node, "style"),
                "display",
                &JsValue::from_str("block"),
            );
        } else {
            set(&usd_node, "textContent", &JsValue::from_str(""));
            set(&usd_node, "hidden", &JsValue::TRUE);
        }
    }
    apply_python_load_address_style(&scope);
}

fn set_address_name(section: &JsValue, name: &str) {
    let scope = if present(section) {
        section.clone()
    } else {
        root()
    };
    let clean = name.trim();
    let node = query(&scope, "#explorerAddressNameValue");
    if present(&node) {
        set(&node, "textContent", &JsValue::from_str(clean));
        set(&node, "title", &JsValue::from_str(clean));
        set(&node, "hidden", &JsValue::from_bool(clean.is_empty()));
        set(
            &property(&node, "style"),
            "display",
            &JsValue::from_str(if clean.is_empty() { "none" } else { "block" }),
        );
    }
    let balance = query(&scope, "#explorerBalanceValue");
    if present(&balance) && !clean.is_empty() {
        set(
            &balance,
            "title",
            &JsValue::from_str(&format!("Wallet: {clean}")),
        );
    }
    apply_python_load_address_style(&scope);
}

pub(crate) async fn refresh_address_name_internal(section: JsValue, address: &str) -> String {
    let name = resolve_known_name_internal(address).await;
    set_address_name(&section, &name);
    if !name.is_empty() {
        let _ = save_address_to_database_internal(address, &name).await;
    }
    name
}

fn saved_address_item(item: &JsValue) -> (String, String) {
    let address = first_text(
        item,
        &["address", "kaspa_address", "kaspaAddress", "wallet_address"],
    );
    let name = first_text(item, &["name", "known_name", "label", "alias"]);
    (address, name)
}
pub(crate) async fn load_saved_addresses_internal(section: JsValue) -> bool {
    let scope = if present(&section) { section } else { root() };
    let input = query(&scope, "#explorerAddress");
    let list = query(&scope, "#explorerAddressOptions");
    if !present(&input) {
        return false;
    }
    let tag = js_text(&property(&input, "tagName")).to_lowercase();
    let is_select = tag == "select";
    if !is_select && !present(&list) {
        return false;
    }

    let result = match invoke_first(vec![
        ("explorer_saved_addresses", empty_args()),
        ("get_all_addresses", empty_args()),
        ("list_addresses", empty_args()),
    ])
    .await
    {
        Ok(value) => value,
        Err(error) => {
            diagnostic_message("warn", "LOAD SAVED ADDRESSES FAILED", &error_text(&error));
            return false;
        }
    };
    let addresses = list_from_shape(&result, &["addresses", "items"]);
    STATE.with(|state| state.borrow_mut().saved_count = addresses.len());

    if is_select {
        set(&input, "innerHTML", &JsValue::from_str(""));
        let placeholder = create_element("option");
        set(&placeholder, "value", &JsValue::from_str(""));
        set(
            &placeholder,
            "textContent",
            &JsValue::from_str("Select saved address..."),
        );
        append_child(&input, &placeholder);
        let mut seen = Vec::<String>::new();
        for item in &addresses {
            let (address, name) = saved_address_item(item);
            if !address.starts_with("kaspa:") || seen.iter().any(|value| value == &address) {
                continue;
            }
            seen.push(address.clone());
            let option = create_element("option");
            set(&option, "value", &JsValue::from_str(&address));
            let label = if name.is_empty() {
                address.clone()
            } else {
                format!("{name} — {address}")
            };
            set(&option, "textContent", &JsValue::from_str(&label));
            append_child(&input, &option);
            if !name.is_empty() {
                store_known_name(&address, &name);
            }
        }
        return true;
    }

    set(&list, "innerHTML", &JsValue::from_str(""));
    for item in &addresses {
        let (address, name) = saved_address_item(item);
        if address.is_empty() {
            continue;
        }
        let option = create_element("option");
        set(&option, "value", &JsValue::from_str(&address));
        if !name.is_empty() {
            set(&option, "label", &JsValue::from_str(&name));
            store_known_name(&address, &name);
        }
        append_child(&list, &option);
    }
    let _ = crate::explorer_controls::explorer_install_manual_address_save();
    true
}

fn balance_value(report: &JsValue) -> Option<f64> {
    for name in ["balance_kas", "balanceKas", "balance"] {
        let value = property(report, name);
        if present(&value) {
            let number = crate::js_number(&value);
            if number.is_finite() {
                return Some(number);
            }
        }
    }
    for name in ["balance_sompi", "balanceSompi"] {
        let value = property(report, name);
        if present(&value) {
            let number = crate::js_number(&value);
            if number.is_finite() {
                return Some(number / SOMPI_PER_KAS);
            }
        }
    }
    None
}

pub(crate) async fn fetch_balance_internal(section: JsValue, address: &str) -> Option<f64> {
    set_balance(&section, None);
    let _ = refresh_address_name_internal(section.clone(), address).await;
    let direct = args1("address", JsValue::from_str(address));
    let nested = nested_args("request", "address", JsValue::from_str(address));
    let report = match invoke_first(vec![
        ("explorer_fetch_balance", direct.clone()),
        ("explorer_fetch_balance", nested.clone()),
        ("explorer_balance", direct),
        ("explorer_balance", nested),
    ])
    .await
    {
        Ok(value) => value,
        Err(error) => {
            set_balance(&section, None);
            diagnostic_message("error", "BALANCE FETCH FAILED", &error_text(&error));
            return None;
        }
    };
    diagnostic("apiShape", "BALANCE RAW RESULT", &report);
    let balance = balance_value(&report);
    set_balance(&section, balance);
    balance
}

async fn canonical_address_internal(address: &str) -> String {
    let clean = normalize_address_text(address);
    if !is_kaspa_address_text(&clean) {
        return String::new();
    }
    match invoke_tauri(
        "validate_kaspa_address",
        &args1("address", JsValue::from_str(&clean)),
    )
    .await
    {
        Ok(value) => normalize_address_text(&js_text(&value)),
        Err(_) => String::new(),
    }
}

fn diagnostics_snapshot(address: &str) -> JsValue {
    let keys = lookup_keys_text(address);
    let output = Object::new();
    let key_array = Array::new();
    let matches = Array::new();
    STATE.with(|state| {
        let state = state.borrow();
        set(
            output.as_ref(),
            "namesLoaded",
            &JsValue::from_bool(state.names_loaded),
        );
        set(
            output.as_ref(),
            "topNamesLoaded",
            &JsValue::from_bool(state.top_names_loaded),
        );
        set(
            output.as_ref(),
            "namesCount",
            &JsValue::from_f64(state.known_names.len() as f64),
        );
        set(
            output.as_ref(),
            "savedCount",
            &JsValue::from_f64(state.saved_count as f64),
        );
        for key in &keys {
            key_array.push(&JsValue::from_str(key));
            if let Some(value) = state.known_names.get(key) {
                let item = Object::new();
                set(item.as_ref(), "key", &JsValue::from_str(key));
                set(item.as_ref(), "value", &JsValue::from_str(value));
                matches.push(item.as_ref());
            }
        }
    });
    set(output.as_ref(), "lookupKeys", key_array.as_ref());
    set(output.as_ref(), "matches", matches.as_ref());
    output.into()
}

#[wasm_bindgen(js_name = explorerNormalizeAddress)]
pub fn explorer_normalize_address(value: JsValue) -> String {
    normalize_address_text(&js_text(&value))
}

#[wasm_bindgen(js_name = explorerAddressLookupKeys)]
pub fn explorer_address_lookup_keys(value: JsValue) -> Array {
    let output = Array::new();
    for key in lookup_keys_text(&js_text(&value)) {
        output.push(&JsValue::from_str(&key));
    }
    output
}

#[wasm_bindgen(js_name = explorerCanonicalKaspaAddress)]
pub async fn explorer_canonical_kaspa_address(value: JsValue) -> String {
    canonical_address_internal(&js_text(&value)).await
}

#[wasm_bindgen(js_name = explorerLoadKnownAddressNames)]
pub async fn explorer_load_known_address_names() -> JsValue {
    load_known_names_internal().await;
    diagnostics_snapshot("")
}
#[wasm_bindgen(js_name = explorerSaveAddressToDatabase)]
pub async fn explorer_save_address_to_database(
    address: JsValue,
    name: JsValue,
) -> Result<JsValue, JsValue> {
    save_address_to_database_internal(&js_text(&address), &js_text(&name)).await
}

#[wasm_bindgen(js_name = explorerRefreshAddressName)]
pub async fn explorer_refresh_address_name(section: JsValue, address: JsValue) -> String {
    refresh_address_name_internal(section, &js_text(&address)).await
}

#[wasm_bindgen(js_name = explorerLoadSavedAddresses)]
pub async fn explorer_load_saved_addresses(section: JsValue) -> bool {
    load_saved_addresses_internal(section).await
}

#[wasm_bindgen(js_name = explorerFetchBalance)]
pub async fn explorer_fetch_balance(section: JsValue, address: JsValue) -> JsValue {
    fetch_balance_internal(section, &js_text(&address))
        .await
        .map(JsValue::from_f64)
        .unwrap_or(JsValue::NULL)
}
#[wasm_bindgen(js_name = explorerAddressDiagnosticsSnapshot)]
pub fn explorer_address_diagnostics_snapshot(address: JsValue) -> JsValue {
    diagnostics_snapshot(&js_text(&address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_validation_matches_legacy_shape() {
        assert!(is_kaspa_address_text(&format!("kaspa:{}", "a".repeat(50))));
        assert!(is_kaspa_address_text(&format!(
            "kaspatest:{}",
            "b1".repeat(25)
        )));
        assert!(is_kaspa_address_text(&format!(
            "kaspadev:{}",
            "c".repeat(50)
        )));
        assert!(is_kaspa_address_text(&format!(
            "kaspasim:{}",
            "d".repeat(50)
        )));
        assert!(!is_kaspa_address_text(&format!("kaspa:{}", "a".repeat(49))));
        assert!(!is_kaspa_address_text("not-an-address"));
    }

    #[test]
    fn lookup_keys_preserve_legacy_aliases() {
        let address = format!("kaspatest:{}", "Ab".repeat(25));
        let keys = lookup_keys_text(&address);
        let payload = "Ab".repeat(25);
        assert!(keys.contains(&address));
        assert!(keys.contains(&address.to_lowercase()));
        assert!(keys.contains(&payload));
        assert!(keys.contains(&payload.to_lowercase()));
        assert!(keys.contains(&format!("kaspa:{payload}")));
        assert!(keys.contains(&format!("kaspa:{}", payload.to_lowercase())));
    }

    #[test]
    fn save_memo_key_is_trimmed_and_stable() {
        assert_eq!(memo_key(" kaspa:abc ", " Alice "), "kaspa:abc|Alice");
    }

    #[test]
    fn balance_sompi_constant_is_exact() {
        assert_eq!(SOMPI_PER_KAS, 100_000_000.0);
    }
}
