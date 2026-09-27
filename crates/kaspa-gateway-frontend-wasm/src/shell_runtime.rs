use js_sys::{Array, Function, Object, Promise, Reflect};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

const THEME_KEY: &str = "kgw-shell-theme";
const LAST_TAB_KEY: &str = "kgw.shell.lastMainTab";
const DEFAULT_TAB: &str = "kaspa-node";
const RESTORE_DELAYS: [i32; 5] = [0, 80, 250, 800, 1600];

thread_local! {
    static TABS: RefCell<Vec<JsValue>> = const { RefCell::new(Vec::new()) };
    static LOADED: RefCell<HashMap<String, JsValue>> = RefCell::new(HashMap::new());
    static MOUNTED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static INITIALIZED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    static BOOT_RUNNING: Cell<bool> = const { Cell::new(false) };
    static BOOT_DONE: Cell<bool> = const { Cell::new(false) };
    static PENDING_SAVED_TAB: RefCell<String> = const { RefCell::new(String::new()) };
    static EXPLICIT_GENERATION: Cell<u32> = const { Cell::new(0) };
    static EXPLICIT_SEEN: Cell<bool> = const { Cell::new(false) };
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
fn present(v: &JsValue) -> bool {
    !v.is_null() && !v.is_undefined()
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
fn delete_property(target: &JsValue, name: &str) {
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
fn text(v: &JsValue) -> String {
    crate::js_string_owned(v)
}
fn truthy(v: &JsValue) -> bool {
    crate::js_boolean(v)
}
fn dataset(v: &JsValue) -> JsValue {
    property(v, "dataset")
}
fn query(selector: &str) -> JsValue {
    call1(&document(), "querySelector", &JsValue::from_str(selector))
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
fn css_escape(value: &str) -> String {
    let css = property(&global(), "CSS");
    func(&css, "escape")
        .and_then(|f| f.call1(&css, &JsValue::from_str(value)).ok())
        .map(|v| text(&v))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| value.to_owned())
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
fn class_call(node: &JsValue, method: &str, name: &str) {
    let _ = call1(
        &property(node, "classList"),
        method,
        &JsValue::from_str(name),
    );
}
fn class_toggle(node: &JsValue, name: &str, state: bool) {
    let _ = call2(
        &property(node, "classList"),
        "toggle",
        &JsValue::from_str(name),
        &JsValue::from_bool(state),
    );
}
fn log(message: &str, detail: &str) {
    let console = property(&global(), "console");
    if let Some(f) = func(&console, "log") {
        let _ = f.call2(
            &console,
            &JsValue::from_str(message),
            &JsValue::from_str(detail),
        );
    }
}
fn warn(message: &str) {
    let console = property(&global(), "console");
    if let Some(f) = func(&console, "warn") {
        let _ = f.call1(&console, &JsValue::from_str(message));
    }
}
fn tauri_invoke() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    for value in [
        property(&property(&tauri, "core"), "invoke"),
        property(&property(&tauri, "tauri"), "invoke"),
        property(&tauri, "invoke"),
        property(&window(), "__TAURI_INVOKE__"),
    ] {
        if let Ok(f) = value.dyn_into::<Function>() {
            return Some(f);
        }
    }
    None
}
async fn invoke(command: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let f =
        tauri_invoke().ok_or_else(|| JsValue::from_str("Tauri invoke API is not available."))?;
    let result = f.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?;
    JsFuture::from(Promise::resolve(&result)).await
}
fn tab_id(tab: &JsValue) -> String {
    text(&property(tab, "id"))
}
fn tabs() -> Vec<JsValue> {
    TABS.with(|tabs| tabs.borrow().clone())
}
fn all_tab_ids() -> Vec<String> {
    tabs()
        .iter()
        .map(tab_id)
        .filter(|v| !v.is_empty())
        .collect()
}
fn known_tab(input: &str) -> String {
    let candidate = input.trim().trim_start_matches('#');
    all_tab_ids()
        .into_iter()
        .find(|id| id == candidate)
        .unwrap_or_default()
}
fn tab_by_id(input: &str) -> Result<JsValue, JsValue> {
    let requested = known_tab(input);
    let candidate = if requested.is_empty() {
        resolve_startup_tab(input, "tab-by-id")
    } else {
        requested
    };
    let list = tabs();
    for tab in &list {
        if tab_id(tab) == candidate {
            return Ok(tab.clone());
        }
    }
    for fallback in [DEFAULT_TAB, "settings"] {
        for tab in &list {
            if tab_id(tab) == fallback {
                return Ok(tab.clone());
            }
        }
    }
    Err(JsValue::from_str(
        "No valid KGW tab configuration is available.",
    ))
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
fn read_last_tab() -> String {
    known_tab(&storage_get(LAST_TAB_KEY))
}
fn save_last_tab(tab: &str) -> String {
    let candidate = known_tab(tab);
    if !candidate.is_empty() {
        storage_set(LAST_TAB_KEY, &candidate);
    }
    candidate
}
fn display_owner() -> JsValue {
    property(&window(), "kgwShellDisplayAndActiveTabOwnerR59C")
}
fn resolve_startup_tab(input: &str, reason: &str) -> String {
    let requested = input.trim().trim_start_matches('#');
    let owner = display_owner();
    if let Some(resolve) = func(&owner, "resolveTabId")
        && let Ok(value) = resolve.call2(
            &owner,
            &JsValue::from_str(requested),
            &JsValue::from_str(reason),
        )
    {
        let resolved = text(&value);
        if !resolved.is_empty() {
            return resolved;
        }
    }
    let known = known_tab(requested);
    if !known.is_empty() {
        known
    } else {
        DEFAULT_TAB.to_owned()
    }
}
fn resolve_initial(hash: &str) -> String {
    let saved = read_last_tab();
    if !saved.is_empty() {
        return saved;
    }
    let requested = known_tab(hash);
    if !requested.is_empty() {
        requested
    } else {
        resolve_startup_tab("", "boot-initial-default")
    }
}

#[cfg(test)]
#[derive(Debug, Default, Clone)]
struct NavigationGuard {
    generation: u32,
    explicit_seen: bool,
    pending: String,
}
#[cfg(test)]
impl NavigationGuard {
    fn record(&mut self, tab: &str) -> u32 {
        if !tab.is_empty() {
            self.generation = self.generation.saturating_add(1);
            self.explicit_seen = true;
            self.pending.clear();
        }
        self.generation
    }
    fn current(&self, generation: u32) -> bool {
        generation == self.generation
    }
    fn restore_token_current(&self, generation: u32) -> bool {
        !self.explicit_seen && generation == self.generation
    }
}
fn record_explicit(tab: &str) -> u32 {
    let known = known_tab(tab);
    let generation = EXPLICIT_GENERATION.with(|g| {
        if known.is_empty() {
            return g.get();
        }
        let next = g.get().saturating_add(1);
        g.set(next);
        next
    });
    if !known.is_empty() {
        EXPLICIT_SEEN.set(true);
        PENDING_SAVED_TAB.with(|p| p.borrow_mut().clear());
    }
    generation
}
fn generation_current(generation: u32) -> bool {
    EXPLICIT_GENERATION.with(|g| g.get() == generation)
}
fn restore_token_current(generation: u32) -> bool {
    !EXPLICIT_SEEN.with(Cell::get) && generation_current(generation)
}
fn should_bypass_display_filter(tab: &str, options: &JsValue) -> bool {
    let requested = known_tab(tab);
    if requested.is_empty() {
        return false;
    }
    if truthy(&property(options, "allowHiddenSavedTab")) {
        return true;
    }
    let saved = read_last_tab();
    !saved.is_empty() && saved == requested && !truthy(&property(options, "persist"))
}
fn trace_tab(tab: &str, phase: &str, details: &JsValue) {
    let Some(f) = tauri_invoke() else {
        return;
    };
    let payload = Object::new();
    for (k, v) in [
        ("patch", "KGW_EXPLICIT_MAIN_TAB_TRACE_PATCH_R35C"),
        ("owner", "rust-shell-tab-owner"),
        ("tabId", tab),
        ("phase", phase),
    ] {
        set(payload.as_ref(), k, &JsValue::from_str(v));
    }
    set(payload.as_ref(), "details", details);
    let details_text = js_sys::JSON::stringify(payload.as_ref())
        .ok()
        .map(|v| text(v.as_ref()))
        .unwrap_or_default();
    let args = Object::new();
    for (k, v) in [
        ("scope", "shell"),
        ("net", "ui"),
        ("action", "tab-navigation"),
        ("phase", phase),
    ] {
        set(args.as_ref(), k, &JsValue::from_str(v));
    }
    set(args.as_ref(), "details", &JsValue::from_str(&details_text));
    if let Ok(result) = f.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let ignore = Closure::wrap(Box::new(move |_e: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&ignore);
        ignore.forget();
    }
}
fn strict_style(node: &JsValue, name: &str, value: &str) {
    let style = property(node, "style");
    if let Some(f) = func(&style, "setProperty") {
        let _ = f.call3(
            &style,
            &JsValue::from_str(name),
            &JsValue::from_str(value),
            &JsValue::from_str("important"),
        );
    } else {
        set(&style, name, &JsValue::from_str(value));
    }
}
fn hide_panel(panel: &JsValue) {
    if !present(panel) {
        return;
    }
    class_call(panel, "remove", "active");
    set(panel, "hidden", &JsValue::TRUE);
    set_attr(panel, "hidden", "");
    set_attr(panel, "aria-hidden", "true");
    set(
        &dataset(panel),
        "kgwShellActive",
        &JsValue::from_str("false"),
    );
    set(
        &dataset(panel),
        "kgwShellHidden",
        &JsValue::from_str("true"),
    );
    for (k, v) in [
        ("display", "none"),
        ("visibility", "hidden"),
        ("opacity", "0"),
        ("pointer-events", "none"),
        ("z-index", "0"),
    ] {
        strict_style(panel, k, v);
    }
}
fn show_panel(panel: &JsValue) {
    if !present(panel) {
        return;
    }
    class_call(panel, "add", "page");
    class_call(panel, "add", "active");
    set(panel, "hidden", &JsValue::FALSE);
    remove_attr(panel, "hidden");
    set_attr(panel, "aria-hidden", "false");
    set(
        &dataset(panel),
        "kgwShellActive",
        &JsValue::from_str("true"),
    );
    set(
        &dataset(panel),
        "kgwShellHidden",
        &JsValue::from_str("false"),
    );
    for (k, v) in [
        ("position", "absolute"),
        ("inset", "0"),
        ("width", "100%"),
        ("height", "100%"),
        ("margin", "0"),
        ("display", "block"),
        ("visibility", "visible"),
        ("opacity", "1"),
        ("pointer-events", "auto"),
        ("z-index", "50"),
    ] {
        strict_style(panel, k, v);
    }
}
fn ensure_single_panel(tab_id: &str) -> JsValue {
    let main = by_id("kgwMain");
    if !present(&main) {
        return by_id(tab_id);
    }
    let selector = format!("#{}", css_escape(tab_id));
    let all = query_all(&selector);
    if all.is_empty() {
        let panel = create("section");
        set(&panel, "id", &JsValue::from_str(tab_id));
        set(&panel, "className", &JsValue::from_str("page"));
        set(&dataset(&panel), "tabPanel", &JsValue::from_str(tab_id));
        set(&dataset(&panel), "kgwOwner", &JsValue::from_str(tab_id));
        hide_panel(&panel);
        let _ = call1(&main, "appendChild", &panel);
        return panel;
    }
    let mut keeper = all[0].clone();
    for node in &all {
        if Object::is(&property(node, "parentElement"), &main)
            && text(&property(&dataset(node), "kgwMounted")) == "true"
        {
            keeper = node.clone();
            break;
        }
    }
    if !Object::is(&property(&keeper, "parentElement"), &main) {
        let _ = call1(&main, "appendChild", &keeper);
    }
    for node in all {
        if !Object::is(&node, &keeper) {
            let _ = call0(&node, "remove");
        }
    }
    class_call(&keeper, "add", "page");
    set(&dataset(&keeper), "tabPanel", &JsValue::from_str(tab_id));
    set(&dataset(&keeper), "kgwOwner", &JsValue::from_str(tab_id));
    keeper
}
fn normalize_panels(selected: &str) {
    let main = by_id("kgwMain");
    if !present(&main) {
        return;
    }
    for tab in tabs() {
        let id = tab_id(&tab);
        let panel = ensure_single_panel(&id);
        if id == selected {
            show_panel(&panel)
        } else {
            hide_panel(&panel)
        }
    }
}
fn activate(tab_id_value: &str) -> Result<String, JsValue> {
    let selected = tab_id(&tab_by_id(tab_id_value)?);
    normalize_panels(&selected);
    for tab in tabs() {
        let id = tab_id(&tab);
        let active = id == selected;
        let selector = format!("[data-tab=\"{}\"]", css_escape(&id));
        for button in query_all(&selector) {
            class_toggle(&button, "active", active);
            set_attr(
                &button,
                "aria-selected",
                if active { "true" } else { "false" },
            );
        }
    }
    let location = property(&window(), "location");
    let hash = text(&property(&location, "hash"))
        .trim_start_matches('#')
        .to_owned();
    if hash != selected {
        let history = property(&window(), "history");
        if let Some(f) = func(&history, "replaceState") {
            let _ = f.call3(
                &history,
                &JsValue::NULL,
                &JsValue::from_str(""),
                &JsValue::from_str(&format!("#{selected}")),
            );
        }
    }
    Ok(selected)
}
fn ensure_css(tab: &JsValue) {
    let id = tab_id(tab);
    if present(&query(&format!(
        "link[data-kgw-tab-css=\"{}\"]",
        css_escape(&id)
    ))) {
        return;
    }
    let link = create("link");
    set(&link, "rel", &JsValue::from_str("stylesheet"));
    set(&link, "href", &property(tab, "css"));
    set(&dataset(&link), "kgwTabCss", &JsValue::from_str(&id));
    let _ = call1(&property(&document(), "head"), "appendChild", &link);
}
fn parse_tab_html(tab: &JsValue) -> Result<JsValue, JsValue> {
    let id = tab_id(tab);
    let template = create("template");
    set(
        &template,
        "innerHTML",
        &JsValue::from_str(text(&property(tab, "html")).trim()),
    );
    let content = property(&template, "content");
    let panel = call1(
        &content,
        "querySelector",
        &JsValue::from_str(&format!("#{}", css_escape(&id))),
    );
    if present(&panel) {
        Ok(panel)
    } else {
        Err(JsValue::from_str(&format!(
            "{id}: tab template does not contain #{id}"
        )))
    }
}
fn prepare_panel(panel: &JsValue, tab: &JsValue, active: bool) {
    let id = tab_id(tab);
    set(panel, "id", &JsValue::from_str(&id));
    set(&dataset(panel), "tabPanel", &JsValue::from_str(&id));
    set(&dataset(panel), "kgwMounted", &JsValue::from_str("true"));
    set(&dataset(panel), "kgwOwner", &JsValue::from_str(&id));
    class_call(panel, "add", "page");
    if active {
        show_panel(panel)
    } else {
        hide_panel(panel)
    }
}
fn current_selected() -> String {
    let active = query("[data-tab].active");
    let active = if present(&active) {
        active
    } else {
        query("[data-tab][aria-selected=\"true\"]")
    };
    let id = text(&property(&dataset(&active), "tab"));
    if !known_tab(&id).is_empty() {
        return resolve_startup_tab(&id, "current-selected-button");
    }
    let hash = text(&property(&property(&window(), "location"), "hash"));
    if !known_tab(&hash).is_empty() {
        return resolve_startup_tab(&hash, "current-selected-hash");
    }
    DEFAULT_TAB.to_owned()
}
fn mount_tab(tab: &JsValue) -> Result<(), JsValue> {
    let id = tab_id(tab);
    let mut current = ensure_single_panel(&id);
    let active = id == current_selected();
    if text(&property(&dataset(&current), "kgwMounted")) == "true" {
        prepare_panel(&current, tab, active);
        MOUNTED.with(|m| {
            m.borrow_mut().insert(id);
        });
        return Ok(());
    }
    let main = by_id("kgwMain");
    if !present(&main) {
        return Err(JsValue::from_str("Missing shell main container #kgwMain"));
    }
    if !present(&current) {
        current = create("section");
        set(&current, "id", &JsValue::from_str(&id));
        let _ = call1(&main, "appendChild", &current);
    }
    let panel = parse_tab_html(tab)?;
    prepare_panel(&panel, tab, active);
    if let Some(f) = func(&current, "replaceWith") {
        let _ = f.call1(&current, &panel);
    }
    MOUNTED.with(|m| {
        m.borrow_mut().insert(id);
    });
    Ok(())
}
async fn init_tab(tab: &JsValue) -> Result<(), JsValue> {
    let id = tab_id(tab);
    ensure_css(tab);
    mount_tab(tab)?;
    let module = if let Some(existing) = LOADED.with(|m| m.borrow().get(&id).cloned()) {
        existing
    } else {
        let factory = property(tab, "module")
            .dyn_into::<Function>()
            .map_err(|_| JsValue::from_str("tab module factory missing"))?;
        let result = factory.call0(&JsValue::UNDEFINED)?;
        let module = JsFuture::from(Promise::resolve(&result)).await?;
        LOADED.with(|m| {
            m.borrow_mut().insert(id.clone(), module.clone());
        });
        module
    };
    let already = INITIALIZED.with(|s| s.borrow().contains(&id));
    if !already {
        let name = text(&property(tab, "init"));
        let candidate = property(&module, &name);
        let fallback = property(&window(), &name);
        if let Ok(f) = candidate
            .dyn_into::<Function>()
            .or_else(|_| fallback.dyn_into::<Function>())
        {
            let result = f.call0(&JsValue::UNDEFINED)?;
            let _ = JsFuture::from(Promise::resolve(&result)).await?;
        }
        INITIALIZED.with(|s| {
            s.borrow_mut().insert(id.clone());
        });
    }
    let panel = by_id(&id);
    if present(&panel) {
        set(&dataset(&panel), "kgwMounted", &JsValue::from_str("true"));
        set(
            &dataset(&panel),
            "kgwInitialized",
            &JsValue::from_str("true"),
        );
        set(&dataset(&panel), "kgwOwner", &JsValue::from_str(&id));
        class_call(&panel, "add", "page");
    }
    Ok(())
}
fn close_calendar(reason: &str) {
    for node in query_all(".kgw-calendar-popover") {
        let _ = call0(&node, "remove");
    }
    for node in query_all("[data-kgw-calendar-open='1']") {
        let data = dataset(&node);
        delete_property(&data, "kgwCalendarOpen");
        delete_property(&data, "kgwCalendarScope");
    }
    set(
        &dataset(&property(&document(), "documentElement")),
        "kgwCalendarLifecycleCleanupR11B",
        &JsValue::from_str(reason),
    );
}
async fn open_tab_internal(tab_id_value: String, options: JsValue) -> Result<bool, JsValue> {
    let requested = tab_id_value;
    let persist = truthy(&property(&options, "persist"));
    let allow_hidden = truthy(&property(&options, "allowHiddenSavedTab"));
    let reason = {
        let r = text(&property(&options, "reason"));
        if r.is_empty() {
            "programmatic".to_owned()
        } else {
            r
        }
    };
    let gen_value = property(&options, "explicitNavigationGeneration");
    let explicit = gen_value
        .as_f64()
        .filter(|v| v.is_finite())
        .map(|v| v as u32);
    let bypass = allow_hidden || should_bypass_display_filter(&requested, &options);
    let owner = display_owner();
    let resolved = if bypass {
        known_tab(&requested)
    } else if let Some(f) = func(&owner, "resolveTabId") {
        f.call2(
            &owner,
            &JsValue::from_str(&requested),
            &JsValue::from_str("openTab"),
        )
        .ok()
        .map(|v| text(&v))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| requested.clone())
    } else {
        requested.clone()
    };
    close_calendar("open-tab-before-init");
    let tab = tab_by_id(&resolved)?;
    init_tab(&tab).await?;
    if explicit.is_some_and(|g| !generation_current(g)) {
        return Ok(false);
    }
    let selected = activate(&tab_id(&tab))?;
    if persist {
        let saved = save_last_tab(&selected);
        PENDING_SAVED_TAB.with(|p| *p.borrow_mut() = saved);
    }
    let details = Object::new();
    set(
        details.as_ref(),
        "requestedTabId",
        &JsValue::from_str(&requested),
    );
    set(details.as_ref(), "openReason", &JsValue::from_str(&reason));
    set(
        details.as_ref(),
        "persistAllowed",
        &JsValue::from_bool(persist),
    );
    trace_tab(&selected, "r35c-open-tab", details.as_ref());
    let detail = Object::new();
    set(detail.as_ref(), "tabId", &JsValue::from_str(&selected));
    let init = Object::new();
    set(init.as_ref(), "detail", detail.as_ref());
    if let Ok(event) = js_sys::Reflect::construct(
        &property(&global(), "CustomEvent").dyn_into::<Function>()?,
        &{
            let a = Array::new();
            a.push(&JsValue::from_str("kgw:tab-opened"));
            a.push(init.as_ref());
            a
        },
    ) {
        let _ = call1(&window(), "dispatchEvent", &event);
    }
    if let Ok(f) = property(&window(), "kgwReapplyLanguageSilentlyR89").dyn_into::<Function>() {
        let cb = Closure::once_into_js(move || {
            let _ = f.call1(&window(), &JsValue::from_str("tab-opened-after-mount"));
        });
        let _ = call2(&window(), "setTimeout", &cb, &JsValue::from_f64(0.0));
    }
    Ok(true)
}
fn schedule_saved_restore(reason: String) -> bool {
    if EXPLICIT_SEEN.with(Cell::get) {
        return false;
    }
    let saved = read_last_tab();
    if saved.is_empty() {
        return false;
    }
    let generation = EXPLICIT_GENERATION.with(Cell::get);
    PENDING_SAVED_TAB.with(|p| *p.borrow_mut() = saved.clone());
    for delay in RESTORE_DELAYS {
        let reason = reason.clone();
        let saved = saved.clone();
        let cb = Closure::once_into_js(move || {
            if !restore_token_current(generation) {
                return;
            }
            let pending = PENDING_SAVED_TAB.with(|p| {
                let v = p.borrow().clone();
                if v.is_empty() { saved.clone() } else { v }
            });
            if pending.is_empty() {
                return;
            }
            let hash = text(&property(&property(&window(), "location"), "hash"))
                .trim_start_matches('#')
                .to_owned();
            if hash == pending {
                return;
            }
            let details = Object::new();
            set(details.as_ref(), "reason", &JsValue::from_str(&reason));
            set(details.as_ref(), "delay", &JsValue::from_f64(delay as f64));
            trace_tab(
                &pending,
                "r102c-saved-main-tab-deferred-restore",
                details.as_ref(),
            );
            let opts = Object::new();
            set(opts.as_ref(), "persist", &JsValue::FALSE);
            set(opts.as_ref(), "allowHiddenSavedTab", &JsValue::TRUE);
            set(
                opts.as_ref(),
                "reason",
                &JsValue::from_str("r102c-deferred-saved-main-tab-restore"),
            );
            spawn_local(async move {
                let _ = open_tab_internal(pending, opts.into()).await;
            });
        });
        let _ = call2(
            &window(),
            "setTimeout",
            &cb,
            &JsValue::from_f64(delay as f64),
        );
    }
    true
}
async fn hydrate_version() {
    let title = by_id("kgwAppVersionTitle");
    if !present(&title) {
        return;
    }
    set(&title, "textContent", &JsValue::from_str("KaspaGateway"));
    delete_property(&dataset(&title), "versionSource");
    let Ok(value) = invoke("kgw_app_version_v1", Object::new().as_ref()).await else {
        return;
    };
    let version = text(&value).trim().to_owned();
    let parts: Vec<_> = version.split(['.', '-', '+']).collect();
    if parts.len() < 3
        || !parts[0..3]
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
    {
        warn("[KGW_APP_VERSION_HYDRATE_FAILED] invalid package version");
        return;
    }
    set(
        &title,
        "textContent",
        &JsValue::from_str(&format!("KaspaGateway V{version}")),
    );
    set(
        &dataset(&title),
        "versionSource",
        &JsValue::from_str("cargo-pkg-version"),
    );
}
fn read_theme() -> String {
    let v = storage_get(THEME_KEY);
    if v.is_empty() { "dark".to_owned() } else { v }
}
fn apply_theme_internal(theme: &str) -> String {
    let allowed = ["dark", "superhero", "kaspa", "slate", "midnight", "blue"];
    let value = if allowed.contains(&theme) {
        theme
    } else {
        "dark"
    };
    set(
        &dataset(&property(&document(), "documentElement")),
        "kgwTheme",
        &JsValue::from_str(value),
    );
    let select = by_id("shellThemeSelect");
    if present(&select) && text(&property(&select, "value")) != value {
        set(&select, "value", &JsValue::from_str(value));
    }
    storage_set(THEME_KEY, value);
    value.to_owned()
}
fn bind_navigation() {
    for button in query_all("[data-tab]") {
        if text(&property(&dataset(&button), "kgwBound")) == "true" {
            continue;
        }
        set(&dataset(&button), "kgwBound", &JsValue::from_str("true"));
        let b = button.clone();
        let cb = Closure::wrap(Box::new(move |event: JsValue| {
            let tab = text(&property(&dataset(&b), "tab"));
            let trusted = truthy(&property(&event, "isTrusted"));
            let generation = record_explicit(&tab);
            let opts = Object::new();
            set(opts.as_ref(), "persist", &JsValue::from_bool(trusted));
            set(
                opts.as_ref(),
                "reason",
                &JsValue::from_str(if trusted {
                    "trusted-main-tab-click"
                } else {
                    "untrusted-main-tab-click"
                }),
            );
            set(
                opts.as_ref(),
                "explicitNavigationGeneration",
                &JsValue::from_f64(generation as f64),
            );
            spawn_local(async move {
                if let Err(e) = open_tab_internal(tab, opts.into()).await {
                    warn(&format!("shell openTab failed: {}", text(&e)));
                }
            });
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &button,
            "addEventListener",
            &JsValue::from_str("click"),
            cb.as_ref().unchecked_ref(),
        );
        cb.forget();
    }
}
fn bind_theme() {
    let select = by_id("shellThemeSelect");
    if present(&select) && text(&property(&dataset(&select), "kgwBound")) != "true" {
        set(&dataset(&select), "kgwBound", &JsValue::from_str("true"));
        let s = select.clone();
        let cb = Closure::wrap(Box::new(move |_e: JsValue| {
            apply_theme_internal(&text(&property(&s, "value")));
        }) as Box<dyn FnMut(JsValue)>);
        let _ = call2(
            &select,
            "addEventListener",
            &JsValue::from_str("change"),
            cb.as_ref().unchecked_ref(),
        );
        cb.forget();
    }
    apply_theme_internal(&read_theme());
}
async fn boot_internal() {
    if BOOT_RUNNING.with(Cell::get) || BOOT_DONE.with(Cell::get) {
        return;
    }
    BOOT_RUNNING.set(true);
    bind_theme();
    bind_navigation();
    hydrate_version().await;
    let saved = read_last_tab();
    if !saved.is_empty() {
        PENDING_SAVED_TAB.with(|p| *p.borrow_mut() = saved.clone());
    }
    let hash = text(&property(&property(&window(), "location"), "hash"));
    let initial = resolve_initial(&hash);
    let opts = Object::new();
    set(opts.as_ref(), "persist", &JsValue::FALSE);
    set(
        opts.as_ref(),
        "allowHiddenSavedTab",
        &JsValue::from_bool(!saved.is_empty() && saved == initial),
    );
    set(opts.as_ref(), "reason", &JsValue::from_str("boot-restore"));
    if let Err(e) = open_tab_internal(initial.clone(), opts.into()).await {
        warn(&format!("Shell boot failed: {}", text(&e)));
        BOOT_RUNNING.set(false);
        return;
    }
    schedule_saved_restore("boot-after-open-tab".to_owned());
    BOOT_DONE.set(true);
    BOOT_RUNNING.set(false);
    log("shell boot complete", &initial);
}
#[wasm_bindgen(js_name = shellRuntimeInstall)]
pub fn install(tabs: JsValue) {
    TABS.with(|state| *state.borrow_mut() = Array::from(&tabs).iter().collect());
    let ready = text(&property(&document(), "readyState"));
    if ready == "loading" {
        let cb = Closure::once_into_js(move || spawn_local(boot_internal()));
        let _ = call2(
            &document(),
            "addEventListener",
            &JsValue::from_str("DOMContentLoaded"),
            &cb,
        );
    } else {
        spawn_local(boot_internal());
    }
}
#[wasm_bindgen(js_name = shellRuntimeOpenTab)]
pub async fn open_tab(tab_id: String, options: JsValue) -> Result<bool, JsValue> {
    open_tab_internal(tab_id, options).await
}
#[wasm_bindgen(js_name = shellRuntimeActivateTab)]
pub fn activate_tab(tab_id: String) -> Result<String, JsValue> {
    activate(&tab_id)
}
#[wasm_bindgen(js_name = shellRuntimeApplyTheme)]
pub fn apply_theme(theme: String) -> String {
    apply_theme_internal(&theme)
}
#[wasm_bindgen(js_name = shellRuntimeScheduleSavedRestore)]
pub fn schedule_saved_main_tab_restore(reason: String) -> bool {
    schedule_saved_restore(reason)
}
#[wasm_bindgen(js_name = shellRuntimeRecordExplicitNavigation)]
pub fn record_explicit_navigation(tab_id: String) -> u32 {
    record_explicit(&tab_id)
}
#[wasm_bindgen(js_name = shellRuntimeSavedMainTab)]
pub fn saved_main_tab() -> String {
    read_last_tab()
}
#[wasm_bindgen(js_name = shellRuntimeTraceTab)]
pub fn trace_tab_export(tab_id: String, phase: String, details: JsValue) {
    trace_tab(&tab_id, &phase, &details);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_navigation_generation_wins() {
        let mut g = NavigationGuard::default();
        assert_eq!(g.record("settings"), 1);
        assert!(!g.restore_token_current(0));
        assert!(g.current(1));
        assert_eq!(g.record("analysis"), 2);
        assert!(!g.current(1));
        assert!(g.current(2));
    }
    #[test]
    fn restore_delays_match_frozen_aud013_contract() {
        assert_eq!(RESTORE_DELAYS, [0, 80, 250, 800, 1600]);
    }
    #[test]
    fn theme_allowlist_is_stable() {
        assert_eq!(DEFAULT_TAB, "kaspa-node");
        assert_eq!(THEME_KEY, "kgw-shell-theme");
        assert_eq!(LAST_TAB_KEY, "kgw.shell.lastMainTab");
    }
}
