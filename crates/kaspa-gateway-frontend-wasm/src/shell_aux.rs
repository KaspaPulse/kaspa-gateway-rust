use js_sys::{Function, Object, Promise, Reflect};
use std::cell::{Cell, RefCell};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

thread_local! {
    static BUSY: Cell<bool> = const { Cell::new(false) };
    static LAST_TEXT: RefCell<String> = RefCell::new("Ready".to_owned());
}

const GITHUB_URL: &str = "https://github.com/KaspaPulse";
const DONATIONS_URL: &str = "https://kaspa.stream/addresses/kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
const TWITTER_URL: &str = "https://x.com/KaspaPulse";

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
fn delete(target: &JsValue, name: &str) {
    if let Ok(object) = target.clone().dyn_into::<Object>() {
        let _ = Reflect::delete_property(&object, &JsValue::from_str(name));
    }
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
    let length = crate::js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .filter(present)
        .collect()
}
fn by_id(id: &str) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(id))
}
fn create(tag: &str) -> JsValue {
    call1(&document(), "createElement", &JsValue::from_str(tag))
}
fn set_attr(node: &JsValue, name: &str, value: &str) {
    let _ = call2(
        node,
        "setAttribute",
        &JsValue::from_str(name),
        &JsValue::from_str(value),
    );
}
fn remove_attr(node: &JsValue, name: &str) {
    let _ = call1(node, "removeAttribute", &JsValue::from_str(name));
}
fn class_call(node: &JsValue, method: &str, class_name: &str) {
    let _ = call1(
        &property(node, "classList"),
        method,
        &JsValue::from_str(class_name),
    );
}
fn closest(node: &JsValue, selector: &str) -> JsValue {
    call1(node, "closest", &JsValue::from_str(selector))
}
fn console(method: &str, message: &str) {
    let target = property(&global(), "console");
    if let Some(f) = func(&target, method) {
        let _ = f.call1(&target, &JsValue::from_str(message));
    }
}
fn tauri_invoke() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    for value in [
        property(&property(&tauri, "core"), "invoke"),
        property(&property(&tauri, "tauri"), "invoke"),
        property(&window(), "__TAURI_INVOKE__"),
    ] {
        if let Ok(function) = value.dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}
async fn invoke(command: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let function = tauri_invoke().ok_or_else(|| JsValue::from_str("Tauri invoke unavailable"))?;
    let result = function.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?;
    JsFuture::from(Promise::resolve(&result)).await
}

fn key_of(element: &JsValue) -> String {
    if !present(element) {
        return String::new();
    }
    let mut parts = vec![
        text(&property(element, "id")),
        text(&property(element, "className")),
        text(&call1(
            element,
            "getAttribute",
            &JsValue::from_str("data-tab"),
        )),
        text(&call1(
            element,
            "getAttribute",
            &JsValue::from_str("data-target"),
        )),
        text(&call1(
            element,
            "getAttribute",
            &JsValue::from_str("aria-controls"),
        )),
        text(&call1(element, "getAttribute", &JsValue::from_str("href"))),
        text(&property(element, "textContent")).trim().to_owned(),
    ];
    parts.retain(|part| !part.is_empty());
    parts.join(" ").to_lowercase()
}
fn tab_name(element: &JsValue) -> &'static str {
    let key = key_of(element);
    if key.contains("explorer") || key.contains("المستكشف") {
        "explorer"
    } else if key.contains(" log")
        || key == "log"
        || key.contains("#log")
        || key.contains("لوج")
        || key.contains("السجل")
    {
        "log"
    } else if key.contains("kaspa node") || key.contains("kaspa-node") {
        "kaspa-node"
    } else if key.contains("kaspa bridge") || key.contains("kaspa-bridge") {
        "kaspa-bridge"
    } else if key.contains("analysis") {
        "analysis"
    } else if key.contains("top addresses") || key.contains("top-addresses") {
        "top-addresses"
    } else if key.contains("settings") {
        "settings"
    } else {
        ""
    }
}
fn top_tab_control(element: &JsValue) -> bool {
    if !present(element) || present(&closest(element, "#kaspa-node,#kaspa-bridge")) {
        return false;
    }
    let name = tab_name(element);
    if name.is_empty() {
        return false;
    }
    let key = key_of(element);
    truthy(&call1(
        element,
        "matches",
        &JsValue::from_str("button,a,[role='tab'],[data-tab],[data-target],[aria-controls]"),
    )) || key.contains("tab")
        || key.contains("nav")
}
fn set_tab_enabled(element: &JsValue, enabled: bool) {
    if !present(element) {
        return;
    }
    if present(&property(element, "disabled")) {
        set(element, "disabled", &JsValue::from_bool(!enabled));
    }
    let style = property(element, "style");
    if enabled {
        remove_attr(element, "disabled");
        remove_attr(element, "aria-disabled");
        class_call(element, "remove", "disabled");
        class_call(element, "remove", "is-disabled");
        set(&style, "pointerEvents", &JsValue::from_str("auto"));
        set(&style, "cursor", &JsValue::from_str("pointer"));
    } else {
        set_attr(element, "aria-disabled", "true");
        class_call(element, "add", "disabled");
        set(&style, "pointerEvents", &JsValue::from_str("none"));
        set(&style, "cursor", &JsValue::from_str("not-allowed"));
    }
}
fn apply_tab_busy_policy(busy: bool) {
    set(
        &window(),
        "__kgwExplorerFetchBusy",
        &JsValue::from_bool(busy),
    );
    for element in query_all("button,a,[role='tab'],[data-tab],[data-target],[aria-controls]") {
        if !top_tab_control(&element) {
            continue;
        }
        let name = tab_name(&element);
        set_tab_enabled(&element, !busy || name == "explorer" || name == "log");
    }
}
fn inside_explorer(element: &JsValue) -> bool {
    present(&closest(element, "#explorer,.explorer-python-root"))
}
fn save_and_set_disabled(element: &JsValue, disabled: bool) {
    if !present(element) || present(&closest(element, "#kaspa-node,#kaspa-bridge")) {
        return;
    }
    let data = dataset(element);
    let current = property(&data, "kgwGlobalPrevDisabled");
    if disabled {
        if current.is_undefined() {
            set(
                &data,
                "kgwGlobalPrevDisabled",
                &JsValue::from_str(if truthy(&property(element, "disabled")) {
                    "true"
                } else {
                    "false"
                }),
            );
        }
        set(element, "disabled", &JsValue::TRUE);
        set_attr(element, "aria-disabled", "true");
        class_call(element, "add", "disabled");
        class_call(element, "add", "is-disabled");
        set(
            &property(element, "style"),
            "pointerEvents",
            &JsValue::from_str("none"),
        );
        set(
            &property(element, "style"),
            "cursor",
            &JsValue::from_str("not-allowed"),
        );
    } else {
        if !current.is_undefined() {
            set(
                element,
                "disabled",
                &JsValue::from_bool(text(&current) == "true"),
            );
            delete(&data, "kgwGlobalPrevDisabled");
        } else {
            set(element, "disabled", &JsValue::FALSE);
        }
        remove_attr(element, "aria-disabled");
        class_call(element, "remove", "disabled");
        class_call(element, "remove", "is-disabled");
        set(
            &property(element, "style"),
            "pointerEvents",
            &JsValue::from_str(""),
        );
        set(
            &property(element, "style"),
            "cursor",
            &JsValue::from_str(""),
        );
    }
}
fn lock_header_dropdowns(busy: bool) {
    for select in query_all("select") {
        let id = text(&property(&select, "id"));
        if id == "shellLanguageSelect" || id == "shellCurrencySelect" || inside_explorer(&select) {
            continue;
        }
        save_and_set_disabled(&select, busy);
    }
}
fn ensure_progress_style() {
    if present(&by_id("kgwInlineReadyProgressStyle")) {
        return;
    }
    let style = create("style");
    set(
        &style,
        "id",
        &JsValue::from_str("kgwInlineReadyProgressStyle"),
    );
    set(
        &style,
        "textContent",
        &JsValue::from_str(
            "#kgwShellReadyStatus.kgw-ready-fetching{display:flex;align-items:center;gap:10px;min-width:0;width:100%;overflow:hidden}\
#kgwShellReadyStatus .kgw-ready-progress-track{width:280px;max-width:34vw;min-width:160px;height:8px;overflow:hidden;background:rgba(96,165,250,.18);border:0;border-radius:999px;flex:0 0 auto}\
#kgwShellReadyStatus .kgw-ready-progress-bar{height:100%;width:38%;border-radius:999px;background:linear-gradient(90deg,rgba(125,211,252,.2),rgba(125,211,252,.95),rgba(125,211,252,.2));animation:kgwReadyProgressSlide 1.05s linear infinite}\
#kgwShellReadyStatus .kgw-ready-progress-text{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}@keyframes kgwReadyProgressSlide{0%{transform:translateX(-130%)}100%{transform:translateX(290%)}}",
        ),
    );
    let _ = call1(&property(&document(), "head"), "appendChild", &style);
}
fn progress_node() -> JsValue {
    ensure_progress_style();
    let ready = by_id("kgwShellReadyStatus");
    if present(&ready) {
        let data = dataset(&ready);
        if property(&data, "kgwOriginalReady").is_undefined() {
            let original = {
                let value = text(&property(&ready, "textContent"));
                if value.is_empty() {
                    "Ready".to_owned()
                } else {
                    value
                }
            };
            set(&data, "kgwOriginalReady", &JsValue::from_str(&original));
        }
        let old = by_id("kgwGlobalFetchProgress");
        if present(&old) {
            let _ = call0(&old, "remove");
        }
        return ready;
    }
    let existing = by_id("kgwGlobalFetchProgress");
    if present(&existing) {
        return existing;
    }
    let node = create("div");
    set(&node, "id", &JsValue::from_str("kgwGlobalFetchProgress"));
    set(&dataset(&node), "busy", &JsValue::from_str("false"));
    set(
        &property(&node, "style"),
        "display",
        &JsValue::from_str("none"),
    );
    let _ = call1(&property(&document(), "body"), "appendChild", &node);
    node
}
fn translated_ready() -> String {
    let translator = property(&window(), "kgwT");
    if let Ok(function) = translator.dyn_into::<Function>()
        && let Ok(value) = function.call1(&window(), &JsValue::from_str("runtime.ready"))
    {
        let value = text(&value);
        if !value.is_empty() {
            return value;
        }
    }
    "Ready".to_owned()
}
fn set_progress_visible(busy: bool, message: &str) {
    BUSY.set(busy);
    if !message.trim().is_empty() {
        LAST_TEXT.with(|last| *last.borrow_mut() = message.trim().to_owned());
    }
    let node = progress_node();
    if text(&property(&node, "id")) == "kgwShellReadyStatus" {
        if busy {
            set(&dataset(&node), "busy", &JsValue::from_str("true"));
            class_call(&node, "add", "kgw-ready-fetching");
            set(
                &node,
                "innerHTML",
                &JsValue::from_str(
                    "<span class=\"kgw-ready-progress-track\" aria-hidden=\"true\"><span class=\"kgw-ready-progress-bar\"></span></span><span class=\"kgw-ready-progress-text\"></span>",
                ),
            );
            let label = call1(
                &node,
                "querySelector",
                &JsValue::from_str(".kgw-ready-progress-text"),
            );
            let value = LAST_TEXT.with(|last| last.borrow().clone());
            if present(&label) {
                set(&label, "textContent", &JsValue::from_str(&value));
            }
        } else {
            set(&dataset(&node), "busy", &JsValue::from_str("false"));
            class_call(&node, "remove", "kgw-ready-fetching");
            set(
                &node,
                "textContent",
                &JsValue::from_str(&translated_ready()),
            );
        }
    } else {
        set(
            &dataset(&node),
            "busy",
            &JsValue::from_str(if busy { "true" } else { "false" }),
        );
        let value = if busy {
            LAST_TEXT.with(|last| last.borrow().clone())
        } else {
            "Ready".to_owned()
        };
        set(&node, "textContent", &JsValue::from_str(&value));
    }
    let _ = crate::apply_status_tone_js(
        node,
        JsValue::from_str(if busy { "loading" } else { "ready" }),
    );
}
fn set_global_fetch_busy(busy: bool, detail: &JsValue) {
    let value = property(detail, "text");
    let fallback = property(detail, "status");
    let message = if truthy(&value) {
        text(&value)
    } else if truthy(&fallback) {
        text(&fallback)
    } else if busy {
        "Fetching transactions...".to_owned()
    } else {
        "Ready".to_owned()
    };
    lock_header_dropdowns(busy);
    set_progress_visible(busy, &message);
}
fn url_for_key(key: &str) -> Option<&'static str> {
    match key.trim().to_lowercase().as_str() {
        "github" => Some(GITHUB_URL),
        "donations" => Some(DONATIONS_URL),
        "twitter" => Some(TWITTER_URL),
        _ => None,
    }
}
async fn try_method(owner: &JsValue, name: &str, url: &str) -> bool {
    let Some(function) = func(owner, name) else {
        return false;
    };
    let Ok(result) = function.call1(owner, &JsValue::from_str(url)) else {
        return false;
    };
    JsFuture::from(Promise::resolve(&result)).await.is_ok()
}
async fn open_external(url: String) -> bool {
    if !url.to_lowercase().starts_with("https://") {
        return false;
    }
    let tauri = property(&window(), "__TAURI__");
    let opener = property(&tauri, "opener");
    if try_method(&opener, "openUrl", &url).await || try_method(&opener, "open", &url).await {
        return true;
    }
    let shell = property(&tauri, "shell");
    if try_method(&shell, "open", &url).await || try_method(&shell, "openUrl", &url).await {
        return true;
    }
    for (command, key) in [
        ("open_external_url", "url"),
        ("plugin:opener|open_url", "url"),
        ("plugin:opener|openUrl", "url"),
        ("plugin:shell|open", "path"),
        ("open_url", "url"),
    ] {
        let args = Object::new();
        set(args.as_ref(), key, &JsValue::from_str(&url));
        if invoke(command, args.as_ref()).await.is_ok() {
            return true;
        }
    }
    if let Some(function) = func(&window(), "open") {
        return function
            .call3(
                &window(),
                &JsValue::from_str(&url),
                &JsValue::from_str("_blank"),
                &JsValue::from_str("noopener,noreferrer"),
            )
            .ok()
            .is_some_and(|v| present(&v));
    }
    false
}
fn normalize_footer_buttons() {
    for element in query_all("[data-kgw-link]") {
        let key = text(&call1(
            &element,
            "getAttribute",
            &JsValue::from_str("data-kgw-link"),
        ));
        let Some(url) = url_for_key(&key) else {
            continue;
        };
        set_attr(&element, "data-kgw-footer-url", url);
        set_attr(&element, "title", url);
        set(
            &property(&element, "style"),
            "cursor",
            &JsValue::from_str("pointer"),
        );
    }
}
fn install_event_owners() {
    let busy_event = Closure::wrap(Box::new(move |event: JsValue| {
        let detail = property(&event, "detail");
        let busy = truthy(&property(&detail, "busy"));
        apply_tab_busy_policy(busy);
        set_global_fetch_busy(busy, &detail);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call2(
        &window(),
        "addEventListener",
        &JsValue::from_str("kgw:explorer-fetch-busy"),
        busy_event.as_ref().unchecked_ref(),
    );
    busy_event.forget();

    let tab_guard = Closure::wrap(Box::new(move |event: JsValue| {
        if !truthy(&property(&window(), "__kgwExplorerFetchBusy")) {
            return;
        }
        let target = property(&event, "target");
        let tab = closest(
            &target,
            "button,a,[role='tab'],[data-tab],[data-target],[aria-controls]",
        );
        if !top_tab_control(&tab) {
            return;
        }
        let name = tab_name(&tab);
        if name == "explorer" || name == "log" {
            set_tab_enabled(&tab, true);
            return;
        }
        let _ = call0(&event, "preventDefault");
        let _ = call0(&event, "stopPropagation");
        let _ = call0(&event, "stopImmediatePropagation");
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3_capture(&document(), "addEventListener", "click", &tab_guard);
    tab_guard.forget();

    for event_name in ["pointerdown", "click", "input", "change"] {
        let name = event_name.to_owned();
        let callback = Closure::wrap(Box::new(move |event: JsValue| {
            if !truthy(&property(&window(), "__kgwExplorerFetchBusy")) {
                return;
            }
            let target = closest(
                &property(&event, "target"),
                "select,button,input,textarea,a,[role='button']",
            );
            if !present(&target)
                || inside_explorer(&target)
                || text(&property(&target, "tagName")) != "SELECT"
            {
                return;
            }
            let id = text(&property(&target, "id"));
            if id == "shellLanguageSelect" || id == "shellCurrencySelect" {
                return;
            }
            let _ = call0(&event, "preventDefault");
            let _ = call0(&event, "stopPropagation");
            let _ = call0(&event, "stopImmediatePropagation");
            console(
                "warn",
                &format!("[KGW Explorer][busy-ui] blocked {name} on non-shell select"),
            );
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call3_capture(&document(), "addEventListener", event_name, &callback);
        callback.forget();
    }

    let footer = Closure::wrap(Box::new(move |event: JsValue| {
        let target = closest(&property(&event, "target"), "[data-kgw-link]");
        if !present(&target) {
            return;
        }
        let key = text(&call1(
            &target,
            "getAttribute",
            &JsValue::from_str("data-kgw-link"),
        ));
        let Some(url) = url_for_key(&key) else {
            return;
        };
        let _ = call0(&event, "preventDefault");
        let _ = call0(&event, "stopPropagation");
        let url = url.to_owned();
        spawn_local(async move {
            if !open_external(url).await {
                console("warn", "[KGW] Failed to open footer URL");
            }
        });
    }) as Box<dyn FnMut(JsValue)>);
    let _ = call3_capture(&document(), "addEventListener", "click", &footer);
    footer.forget();
}
fn call3_capture(
    target: &JsValue,
    method: &str,
    event_name: &str,
    callback: &Closure<dyn FnMut(JsValue)>,
) -> JsValue {
    let Some(function) = func(target, method) else {
        return JsValue::UNDEFINED;
    };
    function
        .call3(
            target,
            &JsValue::from_str(event_name),
            callback.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )
        .unwrap_or(JsValue::UNDEFINED)
}
fn install_globals() {
    let busy = Closure::wrap(Box::new(move |busy: JsValue, detail: JsValue| {
        set_global_fetch_busy(truthy(&busy), &detail);
    }) as Box<dyn FnMut(JsValue, JsValue)>);
    set(
        &window(),
        "kgwSetGlobalFetchBusy",
        busy.as_ref().unchecked_ref(),
    );
    busy.forget();

    let progress = Closure::wrap(Box::new(move |value: JsValue| {
        if !BUSY.with(Cell::get) && !truthy(&property(&window(), "__kgwExplorerFetchBusy")) {
            return;
        }
        let value = text(&value).trim().to_owned();
        if !value.is_empty() {
            set_progress_visible(true, &value);
        }
    }) as Box<dyn FnMut(JsValue)>);
    set(
        &window(),
        "kgwSetGlobalFetchProgressText",
        progress.as_ref().unchecked_ref(),
    );
    progress.forget();

    let install_footer = Closure::wrap(Box::new(normalize_footer_buttons) as Box<dyn FnMut()>);
    set(
        &window(),
        "kgwInstallFooterLinksOwnerPatch",
        install_footer.as_ref().unchecked_ref(),
    );
    install_footer.forget();
}
#[wasm_bindgen(js_name = shellAuxInstall)]
pub fn install() {
    if truthy(&property(&window(), "__kgwShellAuxRustInstalled")) {
        return;
    }
    set(&window(), "__kgwShellAuxRustInstalled", &JsValue::TRUE);
    apply_tab_busy_policy(truthy(&property(&window(), "__kgwExplorerFetchBusy")));
    normalize_footer_buttons();
    install_globals();
    install_event_owners();
    console("log", "[KGW shell aux] Rust busy/footer owner installed");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn footer_urls_are_https_and_complete() {
        for key in ["github", "donations", "twitter"] {
            let url = url_for_key(key).unwrap();
            assert!(url.starts_with("https://"));
        }
        assert!(url_for_key("unknown").is_none());
    }
    #[test]
    fn canonical_busy_tab_names_are_stable() {
        assert_eq!(["explorer", "log"].len(), 2);
    }
}
