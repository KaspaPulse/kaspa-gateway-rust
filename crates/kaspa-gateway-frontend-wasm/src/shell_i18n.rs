use js_sys::{Array, Function, Object, Promise, Reflect};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, future_to_promise, spawn_local};

const STORAGE_KEY: &str = "kgw.shell.language.v73";
const LANGUAGES: [&str; 12] = [
    "en", "ar", "de", "es", "fr", "hi", "id", "ja", "ko", "ru", "tr", "zh-CN",
];
const TEXT_SELECTOR: &str =
    "button,a,label,span,strong,legend,th,td,option,h1,h2,h3,h4,p,small,div";

#[derive(Default)]
struct RuntimeState {
    observer: Option<JsValue>,
    pending: bool,
    applying: bool,
    dict: JsValue,
    fallback: JsValue,
    selected: JsValue,
    lang: String,
    reverse: HashMap<String, String>,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, JsValue>> = RefCell::new(HashMap::new());
    static RUNTIME: RefCell<RuntimeState> = RefCell::new(RuntimeState::default());
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
fn by_id(id: &str) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(id))
}
fn query_all(root: &JsValue, selector: &str) -> Vec<JsValue> {
    let list = call1(root, "querySelectorAll", &JsValue::from_str(selector));
    let len = crate::js_number(&property(&list, "length"));
    if !len.is_finite() || len <= 0.0 {
        return Vec::new();
    }
    (0..len as u32)
        .filter_map(|i| Reflect::get(&list, &JsValue::from_f64(i as f64)).ok())
        .filter(present)
        .collect()
}
fn storage() -> JsValue {
    let local = property(&global(), "localStorage");
    if present(&local) {
        local
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
fn normalize_language(lang: &str) -> String {
    if LANGUAGES.contains(&lang) {
        lang.to_owned()
    } else {
        "en".to_owned()
    }
}
fn normalized_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
fn translation_candidate(value: &str, max_len: usize) -> bool {
    let normalized = normalized_text(value);
    !normalized.is_empty()
        && normalized.chars().count() <= max_len
        && normalized
            .chars()
            .any(|ch| ch.is_ascii_alphabetic() || ('\u{0600}'..='\u{06ff}').contains(&ch))
}
fn object_entries(value: &JsValue) -> Vec<(String, JsValue)> {
    if !value.is_object() || Array::is_array(value) {
        return Vec::new();
    }
    let entries = Object::entries(&Object::from(value.clone()));
    entries
        .iter()
        .filter_map(|entry| {
            let pair = Array::from(&entry);
            (pair.length() >= 2).then(|| (text(&pair.get(0)), pair.get(1)))
        })
        .collect()
}
fn flatten_strings(source: &JsValue, prefix: &str, out: &JsValue) {
    for (key, value) in object_entries(source) {
        let next = if prefix.is_empty() {
            key
        } else {
            format!("{prefix}.{key}")
        };
        if value.is_object() && !Array::is_array(&value) {
            flatten_strings(&value, &next, out);
        } else if value.as_string().is_some() {
            set(out, &next, &value);
        }
    }
}
fn flattened(source: &JsValue) -> JsValue {
    let out: JsValue = Object::new().into();
    flatten_strings(source, "", &out);
    out
}
fn merge_dicts(fallback: &JsValue, selected: &JsValue) -> JsValue {
    let out: JsValue = Object::new().into();
    for (key, value) in object_entries(fallback) {
        set(&out, &key, &value);
    }
    for (key, value) in object_entries(selected) {
        set(&out, &key, &value);
    }
    out
}
async fn fetch_language(lang: &str) -> Result<JsValue, JsValue> {
    let fetch = property(&window(), "fetch")
        .dyn_into::<Function>()
        .map_err(|_| JsValue::from_str("fetch unavailable"))?;
    let options = Object::new();
    set(options.as_ref(), "cache", &JsValue::from_str("no-store"));
    let url = format!("./i18n/{lang}.json");
    let response = fetch.call2(&window(), &JsValue::from_str(&url), options.as_ref())?;
    let response = JsFuture::from(Promise::resolve(&response)).await?;
    if !truthy(&property(&response, "ok")) {
        return Err(JsValue::from_str(&format!(
            "HTTP {}",
            text(&property(&response, "status"))
        )));
    }
    let json = call0(&response, "json");
    let json = JsFuture::from(Promise::resolve(&json)).await?;
    Ok(flattened(&json))
}
async fn load_language(lang: &str) -> JsValue {
    let normalized = normalize_language(lang);
    if let Some(cached) = CACHE.with(|cache| cache.borrow().get(&normalized).cloned()) {
        return cached;
    }
    let loaded = match fetch_language(&normalized).await {
        Ok(value) => value,
        Err(error) => {
            let console = property(&global(), "console");
            if let Some(warn) = func(&console, "warn") {
                let _ = warn.call3(
                    &console,
                    &JsValue::from_str("[KGW i18n] Failed to load language"),
                    &JsValue::from_str(&normalized),
                    &error,
                );
            }
            if normalized == "en" {
                Object::new().into()
            } else {
                fetch_language("en")
                    .await
                    .unwrap_or_else(|_| Object::new().into())
            }
        }
    };
    CACHE.with(|cache| {
        cache.borrow_mut().insert(normalized, loaded.clone());
    });
    loaded
}
fn build_reverse(dicts: &[JsValue], max_len: usize) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for dict in dicts {
        for (key, value) in object_entries(dict) {
            let Some(raw) = value.as_string() else {
                continue;
            };
            if !translation_candidate(&raw, max_len) {
                continue;
            }
            out.entry(normalized_text(&raw)).or_insert(key);
        }
    }
    out
}
fn should_skip(element: &JsValue) -> bool {
    if !present(element) || func(element, "matches").is_none() {
        return true;
    }
    let excluded = call1(
        element,
        "closest",
        &JsValue::from_str("script,style,svg,canvas,[data-kgw-no-i18n='true']"),
    );
    if present(&excluded) {
        return true;
    }
    matches!(
        text(&property(element, "id")).as_str(),
        "kgwHeaderPrice" | "kgwHeaderHashrate" | "kgwHeaderDifficulty"
    )
}
fn has_attr(element: &JsValue, name: &str) -> bool {
    truthy(&call1(element, "hasAttribute", &JsValue::from_str(name)))
}
fn attr(element: &JsValue, name: &str) -> String {
    text(&call1(element, "getAttribute", &JsValue::from_str(name)))
}
fn bind_node(element: &JsValue, reverse: &HashMap<String, String>) {
    if should_skip(element) || !present(&dataset(element)) {
        return;
    }
    let data = dataset(element);
    if text(&property(&data, "i18n")).is_empty()
        && let Some(key) = reverse.get(&normalized_text(&text(&property(element, "textContent"))))
    {
        set(&data, "i18n", &JsValue::from_str(key));
    }
    for (attribute, dataset_key) in [
        ("title", "i18nTitle"),
        ("placeholder", "i18nPlaceholder"),
        ("aria-label", "i18nAriaLabel"),
    ] {
        if has_attr(element, attribute)
            && text(&property(&data, dataset_key)).is_empty()
            && let Some(key) = reverse.get(&normalized_text(&attr(element, attribute)))
        {
            set(&data, dataset_key, &JsValue::from_str(key));
        }
    }
}
fn bind_missing(root: &JsValue, fallback: &JsValue, selected: &JsValue) {
    let scope = if func(root, "querySelectorAll").is_some() {
        root.clone()
    } else {
        document()
    };
    let reverse = build_reverse(&[fallback.clone(), selected.clone()], 160);
    for element in query_all(&scope, TEXT_SELECTOR) {
        bind_node(&element, &reverse);
    }
    set(
        &dataset(&property(&document(), "documentElement")),
        "kgwI18nBoundR99",
        &JsValue::from_str("ready"),
    );
}
fn set_text_safely(element: &JsValue, value: &str) {
    if !present(element) {
        return;
    }
    let tag = text(&property(element, "tagName")).to_lowercase();
    if matches!(tag.as_str(), "select" | "input" | "textarea") {
        return;
    }
    let children = property(element, "childNodes");
    let len = crate::js_number(&property(&children, "length"));
    let mut text_node = None;
    let mut has_element = false;
    if len.is_finite() && len > 0.0 {
        for i in 0..len as u32 {
            let node =
                Reflect::get(&children, &JsValue::from_f64(i as f64)).unwrap_or(JsValue::UNDEFINED);
            let node_type = crate::js_number(&property(&node, "nodeType"));
            if node_type == 1.0 {
                has_element = true;
            } else if node_type == 3.0
                && !normalized_text(&text(&property(&node, "nodeValue"))).is_empty()
                && text_node.is_none()
            {
                text_node = Some(node);
            }
        }
    }
    if !has_element {
        if text(&property(element, "textContent")) != value {
            set(element, "textContent", &JsValue::from_str(value));
        }
        return;
    }
    if let Some(node) = text_node {
        let old = text(&property(&node, "nodeValue"));
        let prefix = old.chars().next().is_some_and(char::is_whitespace);
        let suffix = old.chars().last().is_some_and(char::is_whitespace);
        let mut output = String::new();
        if prefix {
            output.push(' ');
        }
        output.push_str(value);
        if suffix {
            output.push(' ');
        }
        set(&node, "nodeValue", &JsValue::from_str(&output));
    }
}
fn apply_dictionary(dict: &JsValue, lang: &str) {
    let root = property(&document(), "documentElement");
    set(&root, "lang", &JsValue::from_str(lang));
    set(
        &root,
        "dir",
        &JsValue::from_str(if lang == "ar" { "rtl" } else { "ltr" }),
    );
    for element in query_all(&document(), "[data-i18n]") {
        let key = text(&property(&dataset(&element), "i18n"));
        let value = property(dict, &key);
        if let Some(value) = value.as_string() {
            set_text_safely(&element, &value);
        }
    }
    for (selector, data_key, attribute) in [
        ("[data-i18n-title]", "i18nTitle", "title"),
        ("[data-i18n-placeholder]", "i18nPlaceholder", "placeholder"),
        ("[data-i18n-aria-label]", "i18nAriaLabel", "aria-label"),
    ] {
        for element in query_all(&document(), selector) {
            let key = text(&property(&dataset(&element), data_key));
            if let Some(value) = property(dict, &key).as_string() {
                let _ = call2(
                    &element,
                    "setAttribute",
                    &JsValue::from_str(attribute),
                    &JsValue::from_str(&value),
                );
            }
        }
    }
    set(
        &dataset(&root),
        "kgwLanguageApplied",
        &JsValue::from_str(lang),
    );
}
fn mark_dynamic(root: &JsValue) {
    let reverse = RUNTIME.with(|state| state.borrow().reverse.clone());
    if reverse.is_empty() {
        return;
    }
    let scope = if func(root, "querySelectorAll").is_some() {
        root.clone()
    } else {
        document()
    };
    if crate::js_number(&property(&scope, "nodeType")) == 1.0 {
        bind_node(&scope, &reverse);
    }
    for element in query_all(&scope, TEXT_SELECTOR) {
        bind_node(&element, &reverse);
    }
}
fn set_timeout(callback: JsValue, delay: f64) {
    if let Ok(timeout) = property(&window(), "setTimeout").dyn_into::<Function>() {
        let _ = timeout.call2(&window(), &callback, &JsValue::from_f64(delay));
    }
}
fn schedule_reapply(reason: String) {
    let should_run = RUNTIME.with(|state| {
        let mut state = state.borrow_mut();
        if state.pending || state.applying || !present(&state.dict) || state.lang.is_empty() {
            false
        } else {
            state.pending = true;
            true
        }
    });
    if !should_run {
        return;
    }
    let callback = Closure::once_into_js(move || {
        let snapshot = RUNTIME.with(|state| {
            let mut state = state.borrow_mut();
            state.pending = false;
            if !present(&state.dict) || state.lang.is_empty() {
                None
            } else {
                state.applying = true;
                Some((state.dict.clone(), state.lang.clone()))
            }
        });
        let Some((dict, lang)) = snapshot else {
            return;
        };
        mark_dynamic(&document());
        apply_dictionary(&dict, &lang);
        set(
            &dataset(&property(&document(), "documentElement")),
            "kgwLastDynamicI18nReapplyR102",
            &JsValue::from_str(&reason),
        );
        let clear = Closure::once_into_js(|| {
            RUNTIME.with(|state| state.borrow_mut().applying = false);
        });
        set_timeout(clear, 0.0);
    });
    set_timeout(callback, 32.0);
}
fn install_observer() {
    let already = RUNTIME.with(|state| state.borrow().observer.is_some());
    if already {
        return;
    }
    let Ok(ctor) = property(&global(), "MutationObserver").dyn_into::<Function>() else {
        return;
    };
    let callback = Closure::wrap(Box::new(move |mutations: JsValue, _observer: JsValue| {
        let applying = RUNTIME.with(|state| state.borrow().applying);
        if applying || !Array::is_array(&mutations) {
            return;
        }
        for mutation in Array::from(&mutations).iter() {
            let kind = text(&property(&mutation, "type"));
            if kind == "childList" {
                let added = property(&mutation, "addedNodes");
                if crate::js_number(&property(&added, "length")) > 0.0 {
                    schedule_reapply("child-list".to_owned());
                    return;
                }
            } else if kind == "attributes" {
                let name = text(&property(&mutation, "attributeName"));
                if matches!(name.as_str(), "title" | "placeholder" | "aria-label") {
                    schedule_reapply("attribute-change".to_owned());
                    return;
                }
            } else if kind == "characterData" {
                schedule_reapply("text-change".to_owned());
                return;
            }
        }
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    let args = Array::new();
    args.push(callback.as_ref().unchecked_ref());
    let Ok(observer) = Reflect::construct(&ctor, &args) else {
        return;
    };
    callback.forget();
    let options = Object::new();
    set(options.as_ref(), "childList", &JsValue::TRUE);
    set(options.as_ref(), "subtree", &JsValue::TRUE);
    set(options.as_ref(), "attributes", &JsValue::TRUE);
    let filters = Array::new();
    for name in ["title", "placeholder", "aria-label"] {
        filters.push(&JsValue::from_str(name));
    }
    set(options.as_ref(), "attributeFilter", filters.as_ref());
    set(options.as_ref(), "characterData", &JsValue::TRUE);
    let _ = call2(
        &observer,
        "observe",
        &property(&document(), "documentElement"),
        options.as_ref(),
    );
    RUNTIME.with(|state| state.borrow_mut().observer = Some(observer));
    set(
        &dataset(&property(&document(), "documentElement")),
        "kgwDynamicI18nObserverR102",
        &JsValue::from_str("installed"),
    );
}
fn update_runtime(dict: JsValue, lang: String, fallback: JsValue, selected: JsValue) {
    let reverse = build_reverse(&[fallback.clone(), selected.clone()], 180);
    RUNTIME.with(|state| {
        let mut state = state.borrow_mut();
        state.dict = dict;
        state.lang = lang;
        state.fallback = fallback;
        state.selected = selected;
        state.reverse = reverse;
    });
    install_observer();
    mark_dynamic(&document());
}
fn translate(key: &str, fallback: &str) -> String {
    let dict = property(&window(), "__kgwI18nDictR107");
    let value = property(&dict, key);
    if let Some(value) = value.as_string()
        && !value.is_empty()
    {
        return value;
    }
    if !fallback.is_empty() {
        fallback.to_owned()
    } else {
        key.to_owned()
    }
}
fn current_language() -> String {
    let select = by_id("shellLanguageSelect");
    let selected = text(&property(&select, "value"));
    if !selected.is_empty() {
        normalize_language(&selected)
    } else {
        normalize_language(&storage_get(STORAGE_KEY))
    }
}
fn dispatch_language_applied(lang: &str, reason: &str) {
    let Ok(ctor) = property(&global(), "CustomEvent").dyn_into::<Function>() else {
        return;
    };
    let detail = Object::new();
    set(detail.as_ref(), "language", &JsValue::from_str(lang));
    set(detail.as_ref(), "reason", &JsValue::from_str(reason));
    let opts = Object::new();
    set(opts.as_ref(), "detail", detail.as_ref());
    let args = Array::new();
    args.push(&JsValue::from_str("kgw:language-applied"));
    args.push(opts.as_ref());
    if let Ok(event) = Reflect::construct(&ctor, &args) {
        let _ = call1(&window(), "dispatchEvent", &event);
    }
}
async fn set_language_internal(lang: String, reason: String, dispatch: bool) -> String {
    let normalized = normalize_language(&lang);
    let selected = load_language(&normalized).await;
    let fallback = if normalized == "en" {
        selected.clone()
    } else {
        load_language("en").await
    };
    bind_missing(&document(), &fallback, &selected);
    let dict = merge_dicts(&fallback, &selected);
    set(&window(), "__kgwI18nDictR107", &dict);
    set(
        &window(),
        "__kgwI18nLangR107",
        &JsValue::from_str(&normalized),
    );
    update_runtime(dict.clone(), normalized.clone(), fallback, selected);
    storage_set(STORAGE_KEY, &normalized);
    apply_dictionary(&dict, &normalized);
    let select = by_id("shellLanguageSelect");
    if present(&select) && text(&property(&select, "value")) != normalized {
        set(&select, "value", &JsValue::from_str(&normalized));
    }
    if dispatch {
        dispatch_language_applied(&normalized, &reason);
    } else {
        set(
            &dataset(&property(&document(), "documentElement")),
            "kgwLanguageSilentReapplyReason",
            &JsValue::from_str(&reason),
        );
    }
    normalized
}
fn install_select_listener() {
    let select = by_id("shellLanguageSelect");
    if !present(&select) || text(&property(&dataset(&select), "kgwI18nR73")) == "1" {
        return;
    }
    set(&dataset(&select), "kgwI18nR73", &JsValue::from_str("1"));
    let select_for_change = select.clone();
    let callback = Closure::wrap(Box::new(move |_event: JsValue| {
        let lang = text(&property(&select_for_change, "value"));
        spawn_local(async move {
            let _ = set_language_internal(lang, "dropdown".to_owned(), true).await;
        });
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &select,
        "addEventListener",
        &JsValue::from_str("change"),
        callback.as_ref().unchecked_ref(),
    );
    callback.forget();
}
fn publish_translation_api() {
    if !present(&property(&window(), "__kgwI18nDictR107")) {
        set(&window(), "__kgwI18nDictR107", Object::new().as_ref());
    }
    if text(&property(&window(), "__kgwI18nLangR107")).is_empty() {
        set(&window(), "__kgwI18nLangR107", &JsValue::from_str("en"));
    }
    let translate_fn = Closure::wrap(Box::new(move |key: JsValue, fallback: JsValue| -> JsValue {
        JsValue::from_str(&translate(&text(&key), &text(&fallback)))
    }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
    let function = translate_fn.as_ref().clone();
    set(&window(), "kgwT", &function);
    set(&window(), "kgwI18n", &function);
    translate_fn.forget();

    let set_language = Closure::wrap(Box::new(move |lang: JsValue, reason: JsValue| -> JsValue {
        let lang = text(&lang);
        let reason = {
            let value = text(&reason);
            if value.is_empty() {
                "manual".to_owned()
            } else {
                value
            }
        };
        future_to_promise(async move {
            Ok(JsValue::from_str(
                &set_language_internal(lang, reason, true).await,
            ))
        })
        .into()
    }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>);
    set(
        &window(),
        "kgwSetLanguageR73",
        set_language.as_ref().unchecked_ref(),
    );
    set_language.forget();

    let apply = Closure::wrap(Box::new(move || -> JsValue {
        future_to_promise(async move {
            let lang = current_language();
            Ok(JsValue::from_str(
                &set_language_internal(lang, "reapply".to_owned(), true).await,
            ))
        })
        .into()
    }) as Box<dyn FnMut() -> JsValue>);
    set(
        &window(),
        "kgwApplyLanguageR73",
        apply.as_ref().unchecked_ref(),
    );
    apply.forget();

    let silent = Closure::wrap(Box::new(move |reason: JsValue| -> JsValue {
        let reason = {
            let value = text(&reason);
            if value.is_empty() {
                "silent-reapply".to_owned()
            } else {
                value
            }
        };
        future_to_promise(async move {
            let lang = current_language();
            Ok(JsValue::from_str(
                &set_language_internal(lang, reason, false).await,
            ))
        })
        .into()
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    set(
        &window(),
        "kgwReapplyLanguageSilentlyR89",
        silent.as_ref().unchecked_ref(),
    );
    silent.forget();
}
fn install_runtime_events() {
    let display = Closure::wrap(Box::new(move |_event: JsValue| {
        let callback = Closure::once_into_js(|| {
            install_select_listener();
            let lang = current_language();
            spawn_local(async move {
                let _ = set_language_internal(lang, "display-preferences".to_owned(), true).await;
            });
        });
        set_timeout(callback, 0.0);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("kgw:shell-display-preferences-changed"),
        display.as_ref().unchecked_ref(),
    );
    display.forget();

    let tab_opened = Closure::wrap(Box::new(move |_event: JsValue| {
        let callback = Closure::once_into_js(|| {
            let lang = current_language();
            spawn_local(async move {
                let _ = set_language_internal(lang, "tab-opened".to_owned(), true).await;
            });
        });
        set_timeout(callback, 0.0);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("kgw:tab-opened"),
        tab_opened.as_ref().unchecked_ref(),
    );
    tab_opened.forget();

    let click = Closure::wrap(Box::new(move |event: JsValue| {
        let target = property(&event, "target");
        if !present(&call1(&target, "closest", &JsValue::from_str("[data-tab]"))) {
            return;
        }
        let callback = Closure::once_into_js(|| {
            let lang = current_language();
            spawn_local(async move {
                let _ = set_language_internal(lang, "tab-click".to_owned(), true).await;
            });
        });
        set_timeout(callback, 50.0);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3_event(&document(), "click", click.as_ref().unchecked_ref(), true);
    click.forget();
}
fn call3_event(target: &JsValue, event: &str, callback: &JsValue, capture: bool) -> JsValue {
    func(target, "addEventListener")
        .and_then(|f| {
            f.call3(
                target,
                &JsValue::from_str(event),
                callback,
                &JsValue::from_bool(capture),
            )
            .ok()
        })
        .unwrap_or(JsValue::UNDEFINED)
}
fn boot() {
    install_select_listener();
    let lang = current_language();
    spawn_local(async move {
        let _ = set_language_internal(lang, "boot".to_owned(), true).await;
    });
}

#[wasm_bindgen(js_name = shellI18nInstall)]
pub fn install() {
    if truthy(&property(&window(), "kgwShellI18nLanguageSwitchOwnerR73")) {
        return;
    }
    set(
        &window(),
        "kgwShellI18nLanguageSwitchOwnerR73",
        &JsValue::TRUE,
    );
    publish_translation_api();
    install_runtime_events();
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
        let callback = Closure::once_into_js(boot);
        set_timeout(callback, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_language_contract_is_stable() {
        assert_eq!(LANGUAGES.len(), 12);
        assert_eq!(normalize_language("ar"), "ar");
        assert_eq!(normalize_language("zh-CN"), "zh-CN");
        assert_eq!(normalize_language("xx"), "en");
        assert_eq!(STORAGE_KEY, "kgw.shell.language.v73");
    }

    #[test]
    fn text_normalization_matches_runtime_contract() {
        assert_eq!(normalized_text("  Hello \n world  "), "Hello world");
        assert!(translation_candidate("الإعدادات", 160));
        assert!(translation_candidate("Settings", 160));
        assert!(!translation_candidate("12345", 160));
    }
}
