use js_sys::{Array, Function, JSON, Object, Reflect};
use std::collections::HashSet;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::spawn_local;

const STORAGE_KEY: &str = "kgw.shell.display.preferences.v71";
const CANONICAL_SETTINGS_KEY: &str = "kgw-settings-python-exact-state";
const DISPLAY_OWNER_MARKER: &str = "KGW_SHELL_DISPLAY_AND_ACTIVE_TAB_OWNER_R59C";

const LANGUAGES: [(&str, &str); 12] = [
    ("en", "English"),
    ("ar", "Arabic"),
    ("de", "German"),
    ("es", "Spanish"),
    ("fr", "French"),
    ("hi", "Hindi"),
    ("id", "Indonesian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("ru", "Russian"),
    ("tr", "Turkish"),
    ("zh-CN", "Chinese (Simplified)"),
];
const CURRENCIES: [(&str, &str); 17] = [
    ("USD", "USD"),
    ("SAR", "SAR"),
    ("EUR", "EUR"),
    ("GBP", "GBP"),
    ("CAD", "CAD"),
    ("AUD", "AUD"),
    ("CHF", "CHF"),
    ("JPY", "JPY"),
    ("KRW", "KRW"),
    ("CNY", "CNY"),
    ("TRY", "TRY"),
    ("RUB", "RUB"),
    ("INR", "INR"),
    ("IDR", "IDR"),
    ("SGD", "SGD"),
    ("BRL", "BRL"),
    ("HKD", "HKD"),
];
const TAB_OPTIONS: [(&str, &str); 6] = [
    ("explorer", "Explorer"),
    ("kaspa-node", "Kaspa Node"),
    ("kaspa-bridge", "Kaspa Bridge"),
    ("analysis", "Analysis"),
    ("top-addresses", "Top Addresses"),
    ("log", "Log"),
];

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
fn func(target: &JsValue, name: &str) -> Option<Function> {
    property(target, name).dyn_into::<Function>().ok()
}
fn call0(target: &JsValue, name: &str) -> JsValue {
    func(target, name)
        .and_then(|f| f.call0(target).ok())
        .unwrap_or(JsValue::UNDEFINED)
}
fn call1(target: &JsValue, name: &str, a: &JsValue) -> JsValue {
    func(target, name)
        .and_then(|f| f.call1(target, a).ok())
        .unwrap_or(JsValue::UNDEFINED)
}
fn call2(target: &JsValue, name: &str, a: &JsValue, b: &JsValue) -> JsValue {
    func(target, name)
        .and_then(|f| f.call2(target, a, b).ok())
        .unwrap_or(JsValue::UNDEFINED)
}
fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}
fn truthy(value: &JsValue) -> bool {
    crate::js_boolean(value)
}
fn dataset(value: &JsValue) -> JsValue {
    property(value, "dataset")
}
fn query_all(selector: &str) -> Vec<JsValue> {
    let list = call1(
        &document(),
        "querySelectorAll",
        &JsValue::from_str(selector),
    );
    let len = crate::js_number(&property(&list, "length"));
    if !len.is_finite() || len <= 0.0 {
        return Vec::new();
    }
    (0..len as u32)
        .filter_map(|i| Reflect::get(&list, &JsValue::from_f64(i as f64)).ok())
        .filter(present)
        .collect()
}
fn by_id(id: &str) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(id))
}
fn create(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag))
}
fn storage() -> JsValue {
    let direct = property(&global(), "localStorage");
    if present(&direct) {
        direct
    } else {
        property(&window(), "localStorage")
    }
}
fn storage_get(key: &str) -> String {
    text(&call1(&storage(), "getItem", &JsValue::from_str(key)))
}
fn storage_set(key: &str, value: &str) {
    let _ = call2(
        &storage(),
        "setItem",
        &JsValue::from_str(key),
        &JsValue::from_str(value),
    );
}
fn json_parse(value: &str) -> JsValue {
    JSON::parse(value).unwrap_or(JsValue::NULL)
}
fn json_stringify(value: &JsValue) -> String {
    JSON::stringify(value)
        .ok()
        .map(|v| text(v.as_ref()))
        .unwrap_or_else(|| "{}".to_owned())
}
fn values(value: &JsValue) -> Vec<String> {
    if !Array::is_array(value) {
        return Vec::new();
    }
    Array::from(value)
        .iter()
        .map(|v| text(&v).trim().to_owned())
        .filter(|v| !v.is_empty())
        .collect()
}
fn unique_known(input: Vec<String>, known: &[&str], fallback: &str) -> Vec<String> {
    let known: HashSet<&str> = known.iter().copied().collect();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for value in input {
        if known.contains(value.as_str()) && seen.insert(value.clone()) {
            out.push(value);
        }
    }
    if out.is_empty() {
        out.push(fallback.to_owned());
    }
    out
}
fn default_values() -> (Vec<String>, Vec<String>, Vec<String>) {
    (
        vec!["en".to_owned()],
        vec!["USD".to_owned()],
        vec![
            "kaspa-node".to_owned(),
            "kaspa-bridge".to_owned(),
            "settings".to_owned(),
        ],
    )
}
fn normalized_parts(input: &JsValue) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (default_languages, default_currencies, default_tabs) = default_values();
    let source = if input.is_object() {
        input.clone()
    } else {
        JsValue::UNDEFINED
    };
    let known_languages: Vec<&str> = LANGUAGES.iter().map(|(v, _)| *v).collect();
    let known_currencies: Vec<&str> = CURRENCIES.iter().map(|(v, _)| *v).collect();
    let mut known_tabs: Vec<&str> = TAB_OPTIONS.iter().map(|(v, _)| *v).collect();
    known_tabs.push("settings");
    let languages = unique_known(
        if present(&source) {
            values(&property(&source, "languages"))
        } else {
            default_languages
        },
        &known_languages,
        "en",
    );
    let currencies = unique_known(
        if present(&source) {
            values(&property(&source, "currencies"))
        } else {
            default_currencies
        },
        &known_currencies,
        "USD",
    );
    let mut tabs = unique_known(
        if present(&source) {
            values(&property(&source, "tabs"))
        } else {
            default_tabs
        },
        &known_tabs,
        "kaspa-node",
    );
    if !tabs.iter().any(|v| v == "settings") {
        tabs.push("settings".to_owned());
    }
    (languages, currencies, tabs)
}
fn array_value(items: &[String]) -> JsValue {
    let out = Array::new();
    for item in items {
        out.push(&JsValue::from_str(item));
    }
    out.into()
}
fn prefs_object(languages: &[String], currencies: &[String], tabs: &[String]) -> JsValue {
    let out = Object::new();
    set(out.as_ref(), "languages", &array_value(languages));
    set(out.as_ref(), "currencies", &array_value(currencies));
    set(out.as_ref(), "tabs", &array_value(tabs));
    out.into()
}
fn normalize(input: &JsValue) -> JsValue {
    let (languages, currencies, tabs) = normalized_parts(input);
    prefs_object(&languages, &currencies, &tabs)
}
fn defaults_object() -> JsValue {
    let (languages, currencies, tabs) = default_values();
    let out = prefs_object(&languages, &currencies, &tabs);
    set(&out, "activeTab", &JsValue::from_str("kaspa-node"));
    out
}
fn read_canonical() -> Option<JsValue> {
    let raw = storage_get(CANONICAL_SETTINGS_KEY);
    if raw.trim().is_empty() {
        return None;
    }
    let saved = json_parse(&raw);
    let checks = property(&saved, "checks");
    if !checks.is_object() {
        return None;
    }
    let entries = Object::entries(&Object::from(checks));
    let mut languages = Vec::new();
    let mut currencies = Vec::new();
    let mut tabs = Vec::new();
    for entry in entries.iter() {
        let pair = Array::from(&entry);
        let key = text(&pair.get(0));
        if !truthy(&pair.get(1)) {
            continue;
        }
        if let Some(v) = key.strip_prefix("language:") {
            languages.push(v.to_owned());
        } else if let Some(v) = key.strip_prefix("currency:") {
            currencies.push(v.to_owned());
        } else if let Some(v) = key.strip_prefix("tab:") {
            tabs.push(v.to_owned());
        }
    }
    if languages.is_empty() || currencies.is_empty() || tabs.is_empty() {
        return None;
    }
    Some(normalize(&prefs_object(&languages, &currencies, &tabs)))
}
fn read_legacy() -> JsValue {
    let raw = storage_get(STORAGE_KEY);
    if raw.trim().is_empty() {
        return defaults_object();
    }
    normalize(&json_parse(&raw))
}
fn read_preferences() -> JsValue {
    read_canonical().unwrap_or_else(read_legacy)
}
fn save_preferences(input: &JsValue) -> JsValue {
    let normalized = normalize(input);
    storage_set(STORAGE_KEY, &json_stringify(&normalized));
    normalized
}
fn option_values(select: &JsValue) -> Vec<String> {
    let options = property(select, "options");
    let len = crate::js_number(&property(&options, "length"));
    if !len.is_finite() || len <= 0.0 {
        return Vec::new();
    }
    (0..len as u32)
        .filter_map(|i| Reflect::get(&options, &JsValue::from_f64(i as f64)).ok())
        .map(|option| text(&property(&option, "value")))
        .collect()
}
fn dispatch_change(select: &JsValue) {
    let event = if let Ok(event_ctor) = property(&global(), "Event").dyn_into::<Function>() {
        let options = Object::new();
        set(options.as_ref(), "bubbles", &JsValue::TRUE);
        Reflect::construct(&event_ctor, &{
            let args = Array::new();
            args.push(&JsValue::from_str("change"));
            args.push(options.as_ref());
            args
        })
        .unwrap_or(JsValue::UNDEFINED)
    } else {
        JsValue::UNDEFINED
    };
    if present(&event) {
        let _ = call1(select, "dispatchEvent", &event);
    }
}
fn rebuild_select(id: &str, options: &[(&str, &str)], selected: &[String]) {
    let select = by_id(id);
    if !present(&select) {
        return;
    }
    let previous = text(&property(&select, "value"));
    let allowed: Vec<String> = if selected.is_empty() {
        vec![options[0].0.to_owned()]
    } else {
        selected.to_vec()
    };
    let desired: Vec<String> = options
        .iter()
        .filter(|(v, _)| allowed.iter().any(|a| a == v))
        .map(|(v, _)| (*v).to_owned())
        .collect();
    if option_values(&select) != desired {
        let fragment = call0(&document(), "createDocumentFragment");
        for (value, label) in options
            .iter()
            .filter(|(v, _)| allowed.iter().any(|a| a == v))
        {
            let option = create("option");
            set(&option, "value", &JsValue::from_str(value));
            set(&option, "textContent", &JsValue::from_str(label));
            let _ = call1(&fragment, "appendChild", &option);
        }
        let _ = call1(&select, "replaceChildren", &fragment);
    }
    if allowed.iter().any(|v| v == &previous) {
        set(&select, "value", &JsValue::from_str(&previous));
    } else if let Some(first) = allowed.first() {
        set(&select, "value", &JsValue::from_str(first));
        dispatch_change(&select);
    }
}
fn tab_buttons() -> Vec<JsValue> {
    query_all("[data-tab]")
}
fn tab_visible(button: &JsValue) -> bool {
    if !present(button) || truthy(&property(button, "hidden")) {
        return false;
    }
    if text(&property(&dataset(button), "kgwDisplayVisible")) == "false" {
        return false;
    }
    if text(&call1(
        button,
        "getAttribute",
        &JsValue::from_str("aria-hidden"),
    )) == "true"
    {
        return false;
    }
    if text(&property(&property(button, "style"), "display")) == "none" {
        return false;
    }
    true
}
fn tab_id_visible(tab_id: &str) -> bool {
    if tab_id.is_empty() {
        return false;
    }
    let buttons = tab_buttons();
    if buttons.is_empty() {
        return true;
    }
    let matching: Vec<JsValue> = buttons
        .into_iter()
        .filter(|b| text(&property(&dataset(b), "tab")) == tab_id)
        .collect();
    if matching.is_empty() {
        return true;
    }
    matching.iter().any(tab_visible)
}
fn preferred_tab() -> String {
    let buttons = tab_buttons();
    for wanted in ["kaspa-node", "kaspa-bridge", "settings"] {
        if buttons
            .iter()
            .any(|b| text(&property(&dataset(b), "tab")) == wanted && tab_visible(b))
        {
            return wanted.to_owned();
        }
    }
    buttons
        .iter()
        .find(|b| !text(&property(&dataset(b), "tab")).is_empty() && tab_visible(b))
        .map(|b| text(&property(&dataset(b), "tab")))
        .unwrap_or_else(|| "kaspa-node".to_owned())
}
fn resolve_tab(tab_id: &str, reason: &str) -> String {
    let requested = tab_id.trim();
    if !requested.is_empty() && tab_id_visible(requested) {
        return requested.to_owned();
    }
    let fallback = preferred_tab();
    let console = property(&global(), "console");
    if let Some(log) = func(&console, "info") {
        let details = Object::new();
        set(details.as_ref(), "patch", &JsValue::from_str("R59C"));
        set(details.as_ref(), "reason", &JsValue::from_str(reason));
        let requested_value = if requested.is_empty() {
            JsValue::NULL
        } else {
            JsValue::from_str(requested)
        };
        set(details.as_ref(), "requestedTab", &requested_value);
        set(
            details.as_ref(),
            "fallbackTab",
            &JsValue::from_str(&fallback),
        );
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW][shell][canonical-tab-resolved]"),
            details.as_ref(),
        );
    }
    if fallback.is_empty() {
        if requested.is_empty() {
            "kaspa-node".to_owned()
        } else {
            requested.to_owned()
        }
    } else {
        fallback
    }
}
fn active_button() -> Option<JsValue> {
    tab_buttons().into_iter().find(|button| {
        truthy(&call1(
            &property(button, "classList"),
            "contains",
            &JsValue::from_str("active"),
        )) || truthy(&call1(
            &property(button, "classList"),
            "contains",
            &JsValue::from_str("is-active"),
        )) || text(&call1(
            button,
            "getAttribute",
            &JsValue::from_str("aria-selected"),
        )) == "true"
            || text(&call1(
                button,
                "getAttribute",
                &JsValue::from_str("aria-current"),
            )) == "page"
    })
}
fn ensure_active(reason: &str) -> bool {
    let active_id = active_button()
        .map(|b| text(&property(&dataset(&b), "tab")))
        .unwrap_or_default();
    let saved = crate::shell_runtime::saved_main_tab();
    let mut resolved = resolve_tab(&active_id, reason);
    if (active_id.is_empty() || resolved != active_id) && !saved.is_empty() {
        resolved = saved.clone();
    }
    if resolved.is_empty() || resolved == active_id {
        crate::shell_runtime::schedule_saved_main_tab_restore("ensure-active-no-change".to_owned());
        return false;
    }
    let allow_hidden = !saved.is_empty() && saved == resolved;
    let target = resolved.clone();
    spawn_local(async move {
        let options = Object::new();
        set(options.as_ref(), "persist", &JsValue::FALSE);
        set(
            options.as_ref(),
            "allowHiddenSavedTab",
            &JsValue::from_bool(allow_hidden),
        );
        set(
            options.as_ref(),
            "reason",
            &JsValue::from_str(if allow_hidden {
                "display-owner-restore-saved-tab"
            } else {
                "display-owner-ensure-active"
            }),
        );
        let _ = crate::shell_runtime::open_tab(target, options.into()).await;
    });
    crate::shell_runtime::schedule_saved_main_tab_restore(
        "ensure-active-after-schedule".to_owned(),
    );
    true
}
fn style_display(node: &JsValue, visible: bool) {
    let style = property(node, "style");
    if visible {
        let _ = call1(&style, "removeProperty", &JsValue::from_str("display"));
    } else if let Some(set_property) = func(&style, "setProperty") {
        let _ = set_property.call3(
            &style,
            &JsValue::from_str("display"),
            &JsValue::from_str("none"),
            &JsValue::from_str("important"),
        );
    }
}
fn apply_tabs(selected: &[String]) {
    let mut tabs = selected.to_vec();
    if tabs.is_empty() {
        tabs = vec![
            "kaspa-node".to_owned(),
            "kaspa-bridge".to_owned(),
            "settings".to_owned(),
        ];
    }
    if !tabs.iter().any(|v| v == "settings") {
        tabs.push("settings".to_owned());
    }
    let visible: HashSet<&str> = tabs.iter().map(String::as_str).collect();
    for button in tab_buttons() {
        let id = text(&property(&dataset(&button), "tab"));
        let show = id == "settings" || visible.contains(id.as_str());
        set(&button, "hidden", &JsValue::from_bool(!show));
        style_display(&button, show);
        let list = property(&button, "classList");
        let _ = call1(
            &list,
            if show { "remove" } else { "add" },
            &JsValue::from_str("kgw-display-hidden"),
        );
        let _ = call2(
            &button,
            "setAttribute",
            &JsValue::from_str("aria-hidden"),
            &JsValue::from_str(if show { "false" } else { "true" }),
        );
        set(
            &dataset(&button),
            "kgwDisplayVisible",
            &JsValue::from_str(if show { "true" } else { "false" }),
        );
        set(
            &dataset(&button),
            "kgwDisplayOwner",
            &JsValue::from_str("R75"),
        );
    }
    ensure_active("apply-tabs-r75");
}
fn known_languages() -> Vec<&'static str> {
    LANGUAGES.iter().map(|(v, _)| *v).collect()
}
fn known_currencies() -> Vec<&'static str> {
    CURRENCIES.iter().map(|(v, _)| *v).collect()
}
fn option_value(option: &JsValue) -> String {
    for value in [
        property(option, "value"),
        property(&dataset(option), "value"),
        property(&dataset(option), "language"),
        property(&dataset(option), "currency"),
        call1(option, "getAttribute", &JsValue::from_str("data-lang")),
        call1(option, "getAttribute", &JsValue::from_str("data-currency")),
        property(option, "textContent"),
    ] {
        let value = text(&value).trim().to_owned();
        if !value.is_empty() {
            return value;
        }
    }
    String::new()
}
fn ensure_select_universe(select: &JsValue, kind: &str) {
    let source: Vec<(&str, &str)> = if kind == "language" {
        LANGUAGES.to_vec()
    } else {
        let mut out = vec![("KAS", "KAS")];
        out.extend(CURRENCIES);
        out
    };
    for (value, label) in source {
        let exists = option_values(select).iter().any(|v| v == value);
        if exists {
            continue;
        }
        let option = create("option");
        set(&option, "value", &JsValue::from_str(value));
        set(&option, "textContent", &JsValue::from_str(label));
        let key = if kind == "language" {
            if value == "zh-CN" {
                "common.lang.zh.cn".to_owned()
            } else {
                format!("common.lang.{value}")
            }
        } else {
            format!("ui.shell.{}", value.to_lowercase())
        };
        set(&dataset(&option), "i18n", &JsValue::from_str(&key));
        let _ = call1(select, "appendChild", &option);
    }
}
fn select_looks_like_set(select: &JsValue, universe: &[&str], kind: &str) -> bool {
    if !present(&property(select, "options")) {
        return false;
    }
    let attr = [
        property(select, "id"),
        property(select, "name"),
        property(select, "className"),
        call1(select, "getAttribute", &JsValue::from_str("aria-label")),
        call1(select, "getAttribute", &JsValue::from_str("data-testid")),
        property(&dataset(select), "role"),
        property(&dataset(select), "kind"),
    ]
    .iter()
    .map(|v| text(v).to_lowercase())
    .collect::<Vec<_>>()
    .join(" ");
    if attr.contains(kind) {
        return true;
    }
    let vals: Vec<String> = option_values(select)
        .into_iter()
        .filter(|v| !v.is_empty())
        .collect();
    if vals.is_empty() {
        return false;
    }
    let known: HashSet<&str> = universe.iter().copied().collect();
    let hits = vals.iter().filter(|v| known.contains(v.as_str())).count();
    hits >= 2 && hits >= vals.len().div_ceil(2)
}
fn apply_select_visibility(select: &JsValue, allowed: &[String], reason: &str) {
    let allowed_set: HashSet<&str> = allowed.iter().map(String::as_str).collect();
    let options = property(select, "options");
    let len = crate::js_number(&property(&options, "length"));
    let mut first = String::new();
    let mut shown = 0usize;
    let mut hidden = 0usize;
    if len.is_finite() && len > 0.0 {
        for i in 0..len as u32 {
            let option =
                Reflect::get(&options, &JsValue::from_f64(i as f64)).unwrap_or(JsValue::UNDEFINED);
            let value = option_value(&option);
            let visible = allowed_set.is_empty() || allowed_set.contains(value.as_str());
            if visible && first.is_empty() {
                first = value.clone();
            }
            set(&option, "hidden", &JsValue::from_bool(!visible));
            set(&option, "disabled", &JsValue::from_bool(!visible));
            style_display(&option, visible);
            if visible {
                shown += 1;
            } else {
                hidden += 1;
            }
        }
    }
    let current = text(&property(select, "value"));
    if !first.is_empty() && !allowed_set.contains(current.as_str()) {
        set(select, "value", &JsValue::from_str(&first));
        dispatch_change(select);
    }
    set(
        &dataset(select),
        "kgwDisplayReason",
        &JsValue::from_str(reason),
    );
    set(
        &dataset(select),
        "kgwDisplayShown",
        &JsValue::from_str(&shown.to_string()),
    );
    set(
        &dataset(select),
        "kgwDisplayHidden",
        &JsValue::from_str(&hidden.to_string()),
    );
}
fn set_selects(kind: &str, allowed: &[String], reason: &str) {
    let universe = if kind == "language" {
        known_languages()
    } else {
        known_currencies()
    };
    let mut nodes = Vec::<JsValue>::new();
    let selectors: &[&str] = if kind == "language" {
        &[
            "#shellLanguageSelect",
            "#languageSelect",
            "#kgwLanguageSelect",
            "#appLanguageSelect",
            "#settingsLanguageSelect",
            "select[name='language']",
            "select[data-language-select]",
            "select[id*='language' i]",
        ]
    } else {
        &[
            "#shellCurrencySelect",
            "#currencySelect",
            "#kgwCurrencySelect",
            "#appCurrencySelect",
            "#settingsCurrencySelect",
            "select[name='currency']",
            "select[data-currency-select]",
            "select[id*='currency' i]",
        ]
    };
    for selector in selectors {
        for node in query_all(selector) {
            if text(&property(&node, "tagName")) == "SELECT"
                && !nodes.iter().any(|n| Object::is(n, &node))
            {
                nodes.push(node);
            }
        }
    }
    for node in query_all("select") {
        if select_looks_like_set(&node, &universe, kind)
            && !nodes.iter().any(|n| Object::is(n, &node))
        {
            nodes.push(node);
        }
    }
    let explicit = by_id(if kind == "language" {
        "shellLanguageSelect"
    } else {
        "shellCurrencySelect"
    });
    if present(&explicit) && !nodes.iter().any(|n| Object::is(n, &explicit)) {
        nodes.insert(0, explicit);
    }
    for node in nodes {
        let id = text(&property(&node, "id"));
        if (kind == "language" && id == "shellLanguageSelect")
            || (kind == "currency" && id == "shellCurrencySelect")
        {
            ensure_select_universe(&node, kind);
        }
        apply_select_visibility(&node, allowed, reason);
    }
}
fn apply_loose_menu(kind: &str, allowed: &[String], reason: &str) {
    let universe = if kind == "language" {
        known_languages()
    } else {
        known_currencies()
    };
    let universe: HashSet<&str> = universe.into_iter().collect();
    let allowed: HashSet<&str> = allowed.iter().map(String::as_str).collect();
    let selectors: &[&str] = if kind == "language" {
        &[
            "[data-language-option]",
            "[data-lang-option]",
            "[data-lang]",
            "[data-language]",
            "[data-value]",
        ]
    } else {
        &["[data-currency-option]", "[data-currency]", "[data-value]"]
    };
    let mut nodes = Vec::new();
    for selector in selectors {
        for node in query_all(selector) {
            if !nodes.iter().any(|n| Object::is(n, &node)) {
                nodes.push(node);
            }
        }
    }
    for node in nodes {
        let mut value = String::new();
        for candidate in [
            property(&dataset(&node), "value"),
            property(&dataset(&node), "language"),
            property(&dataset(&node), "currency"),
            call1(&node, "getAttribute", &JsValue::from_str("data-lang")),
            call1(&node, "getAttribute", &JsValue::from_str("data-currency")),
        ] {
            let candidate = text(&candidate).trim().to_owned();
            if !candidate.is_empty() {
                value = candidate;
                break;
            }
        }
        if value.is_empty() || !universe.contains(value.as_str()) {
            continue;
        }
        let visible = allowed.is_empty() || allowed.contains(value.as_str());
        set(&node, "hidden", &JsValue::from_bool(!visible));
        let _ = call2(
            &node,
            "setAttribute",
            &JsValue::from_str("aria-hidden"),
            &JsValue::from_str(if visible { "false" } else { "true" }),
        );
        style_display(&node, visible);
        set(
            &dataset(&node),
            "kgwDisplayReason",
            &JsValue::from_str(reason),
        );
    }
}
fn apply_direct(input: &JsValue, reason: &str) -> JsValue {
    let normalized = normalize(input);
    let languages = values(&property(&normalized, "languages"));
    let currencies = values(&property(&normalized, "currencies"));
    let tabs = values(&property(&normalized, "tabs"));
    apply_tabs(&tabs);
    set_selects("language", &languages, reason);
    set_selects("currency", &currencies, reason);
    apply_loose_menu("language", &languages, reason);
    apply_loose_menu("currency", &currencies, reason);
    let detail = Object::new();
    set(detail.as_ref(), "languages", &array_value(&languages));
    set(detail.as_ref(), "currencies", &array_value(&currencies));
    set(detail.as_ref(), "tabs", &array_value(&tabs));
    let custom = property(&global(), "CustomEvent")
        .dyn_into::<Function>()
        .ok()
        .and_then(|ctor| {
            let opts = Object::new();
            set(opts.as_ref(), "detail", detail.as_ref());
            let args = Array::new();
            args.push(&JsValue::from_str("kgw:shell-display-applied-r78"));
            args.push(opts.as_ref());
            Reflect::construct(&ctor, &args).ok()
        });
    if let Some(event) = custom {
        let _ = call1(&window(), "dispatchEvent", &event);
    }
    normalized
}
fn apply(input: &JsValue, reason: &str) -> JsValue {
    let source = if input.is_null() || input.is_undefined() {
        read_preferences()
    } else {
        input.clone()
    };
    let prefs = save_preferences(&source);
    let languages = values(&property(&prefs, "languages"));
    let currencies = values(&property(&prefs, "currencies"));
    let tabs = values(&property(&prefs, "tabs"));
    rebuild_select("shellLanguageSelect", &LANGUAGES, &languages);
    rebuild_select("shellCurrencySelect", &CURRENCIES, &currencies);
    apply_tabs(&tabs);
    set(
        &dataset(&property(&document(), "documentElement")),
        "kgwDisplayPreferencesR71",
        &JsValue::from_str(reason),
    );
    publish_owner();
    ensure_active(reason);
    prefs
}
fn function0(f: impl Fn() -> JsValue + 'static) -> JsValue {
    Closure::wrap(Box::new(f) as Box<dyn FnMut() -> JsValue>).into_js_value()
}
fn function1(f: impl Fn(JsValue) -> JsValue + 'static) -> JsValue {
    Closure::wrap(Box::new(move |a: JsValue| f(a)) as Box<dyn FnMut(JsValue) -> JsValue>)
        .into_js_value()
}
fn function2(f: impl Fn(JsValue, JsValue) -> JsValue + 'static) -> JsValue {
    Closure::wrap(Box::new(move |a: JsValue, b: JsValue| f(a, b))
        as Box<dyn FnMut(JsValue, JsValue) -> JsValue>)
    .into_js_value()
}
fn publish_owner() {
    let owner = Object::new();
    set(
        owner.as_ref(),
        "marker",
        &JsValue::from_str(DISPLAY_OWNER_MARKER),
    );
    set(owner.as_ref(), "defaults", &function0(defaults_object));
    set(owner.as_ref(), "read", &function0(read_preferences));
    set(
        owner.as_ref(),
        "save",
        &function1(|input| save_preferences(&input)),
    );
    set(
        owner.as_ref(),
        "applyPreferences",
        &function2(|input, reason| apply(&input, &text(&reason))),
    );
    set(
        owner.as_ref(),
        "resolveTabId",
        &function2(|tab, reason| JsValue::from_str(&resolve_tab(&text(&tab), &text(&reason)))),
    );
    set(
        owner.as_ref(),
        "ensureActiveTab",
        &function1(|reason| JsValue::from_bool(ensure_active(&text(&reason)))),
    );
    set(
        &window(),
        "kgwShellDisplayAndActiveTabOwnerR59C",
        owner.as_ref(),
    );
}
fn publish_preferences_api() {
    let api = Object::new();
    set(api.as_ref(), "storageKey", &JsValue::from_str(STORAGE_KEY));
    set(api.as_ref(), "defaults", &function0(defaults_object));
    set(
        api.as_ref(),
        "normalize",
        &function1(|input| normalize(&input)),
    );
    set(api.as_ref(), "read", &function0(read_preferences));
    set(
        api.as_ref(),
        "save",
        &function1(|input| save_preferences(&input)),
    );
    set(
        api.as_ref(),
        "apply",
        &function2(|input, reason| apply(&input, &text(&reason))),
    );
    let language_options = Array::new();
    for (value, label) in LANGUAGES {
        let pair = Array::new();
        pair.push(&JsValue::from_str(value));
        pair.push(&JsValue::from_str(label));
        language_options.push(pair.as_ref());
    }
    let currency_options = Array::new();
    for (value, label) in CURRENCIES {
        let pair = Array::new();
        pair.push(&JsValue::from_str(value));
        pair.push(&JsValue::from_str(label));
        currency_options.push(pair.as_ref());
    }
    let tab_options = Array::new();
    for (value, label) in TAB_OPTIONS {
        let pair = Array::new();
        pair.push(&JsValue::from_str(value));
        pair.push(&JsValue::from_str(label));
        tab_options.push(pair.as_ref());
    }
    set(api.as_ref(), "languageOptions", language_options.as_ref());
    set(api.as_ref(), "currencyOptions", currency_options.as_ref());
    set(api.as_ref(), "tabOptions", tab_options.as_ref());
    set(&window(), "kgwShellDisplayPreferencesR71", api.as_ref());
}
fn install_event_listener() {
    let callback = Closure::wrap(Box::new(move |event: JsValue| {
        let detail = property(&event, "detail");
        let source = if present(&detail) {
            detail
        } else {
            read_preferences()
        };
        let _ = apply(&source, "event");
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("kgw:shell-display-preferences-changed"),
        callback.as_ref().unchecked_ref(),
    );
    callback.forget();
}
fn boot() {
    let prefs = read_preferences();
    let _ = apply(&prefs, "boot");
}
#[wasm_bindgen(js_name = shellDisplayApplyDirect)]
pub fn shell_display_apply_direct(prefs: JsValue, reason: String) -> JsValue {
    apply_direct(
        &prefs,
        if reason.is_empty() {
            "direct-shell-apply"
        } else {
            &reason
        },
    )
}
#[wasm_bindgen(js_name = shellDisplayInstall)]
pub fn install() {
    if present(&property(&window(), "kgwShellDisplayPreferencesR71")) {
        return;
    }
    publish_owner();
    publish_preferences_api();
    let direct = function2(|prefs, reason| apply_direct(&prefs, &text(&reason)));
    set(
        &window(),
        "kgwShellApplyDisplayPreferencesDirectR73",
        &direct,
    );
    install_event_listener();
    let ready = text(&property(&document(), "readyState"));
    if ready == "loading" {
        let callback = Closure::once_into_js(boot);
        let _ = call2(
            &document(),
            "addEventListener",
            &JsValue::from_str("DOMContentLoaded"),
            &callback,
        );
    } else {
        let timeout = property(&window(), "setTimeout")
            .dyn_into::<Function>()
            .ok();
        if let Some(timeout) = timeout {
            let callback = Closure::once_into_js(boot);
            let _ = timeout.call2(&window(), &callback, &JsValue::from_f64(0.0));
        } else {
            boot();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_sets_match_frozen_shell_contract() {
        assert_eq!(LANGUAGES.len(), 12);
        assert_eq!(CURRENCIES.len(), 17);
        assert!(
            TAB_OPTIONS
                .iter()
                .any(|(value, _)| *value == "top-addresses")
        );
        assert_eq!(
            DISPLAY_OWNER_MARKER,
            "KGW_SHELL_DISPLAY_AND_ACTIVE_TAB_OWNER_R59C"
        );
    }

    #[test]
    fn defaults_preserve_required_settings_tab() {
        let (languages, currencies, tabs) = default_values();
        assert_eq!(languages, ["en"]);
        assert_eq!(currencies, ["USD"]);
        assert!(tabs.iter().any(|value| value == "settings"));
        assert_eq!(STORAGE_KEY, "kgw.shell.display.preferences.v71");
        assert_eq!(CANONICAL_SETTINGS_KEY, "kgw-settings-python-exact-state");
    }
}
