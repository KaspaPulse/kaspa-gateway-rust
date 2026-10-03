use js_sys::{Array, Date, Function, JSON, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

use super::{
    call_method0, call_method1, js_boolean, js_number, js_string_owned, method, set_property,
};

fn global() -> JsValue {
    js_sys::global().into()
}
fn window() -> JsValue {
    property(&global(), "window")
}
fn document() -> JsValue {
    property(&global(), "document")
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}
fn truthy(value: &JsValue) -> bool {
    js_boolean(value)
}
fn raw(value: &JsValue) -> String {
    js_string_owned(value)
}

fn root() -> JsValue {
    let doc = document();
    let direct = call_method1(&doc, "getElementById", &JsValue::from_str("explorer"))
        .unwrap_or(JsValue::UNDEFINED);
    if truthy(&direct) {
        return direct;
    }
    let legacy = query(&doc, ".explorer-python-root");
    if truthy(&legacy) { legacy } else { doc }
}
fn query(target: &JsValue, selector: &str) -> JsValue {
    call_method1(target, "querySelector", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED)
}
fn query_all(target: &JsValue, selector: &str) -> Vec<JsValue> {
    let Ok(list) = call_method1(target, "querySelectorAll", &JsValue::from_str(selector)) else {
        return Vec::new();
    };
    let length = js_number(&property(&list, "length"));
    if !length.is_finite() || length <= 0.0 {
        return Vec::new();
    }
    (0..length as u32)
        .filter_map(|index| Reflect::get(&list, &JsValue::from_f64(index as f64)).ok())
        .collect()
}
fn create(tag: &str) -> JsValue {
    call_method1(&document(), "createElement", &JsValue::from_str(tag))
        .unwrap_or(JsValue::UNDEFINED)
}
fn append(parent: &JsValue, child: &JsValue) {
    let _ = call_method1(parent, "appendChild", child);
}
fn set_attr(node: &JsValue, name: &str, value: &str) {
    if let Ok(f) = method(node, "setAttribute") {
        let _ = f.call2(node, &JsValue::from_str(name), &JsValue::from_str(value));
    }
}
fn remove(node: &JsValue) {
    let _ = call_method0(node, "remove");
}
fn dataset(node: &JsValue) -> JsValue {
    property(node, "dataset")
}
fn class_list(node: &JsValue) -> JsValue {
    property(node, "classList")
}
fn add_class(node: &JsValue, class: &str) {
    let _ = call_method1(&class_list(node), "add", &JsValue::from_str(class));
}
fn parent(node: &JsValue) -> JsValue {
    property(node, "parentElement")
}
fn closest(node: &JsValue, selector: &str) -> JsValue {
    call_method1(node, "closest", &JsValue::from_str(selector)).unwrap_or(JsValue::UNDEFINED)
}

fn western_digits(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '٠' | '۰' => '0',
            '١' | '۱' => '1',
            '٢' | '۲' => '2',
            '٣' | '۳' => '3',
            '٤' | '۴' => '4',
            '٥' | '۵' => '5',
            '٦' | '۶' => '6',
            '٧' | '۷' => '7',
            '٨' | '۸' => '8',
            '٩' | '۹' => '9',
            other => other,
        })
        .collect()
}
fn pad2(value: u32) -> String {
    format!("{value:02}")
}
fn iso_from_date(date: &Date) -> String {
    format!(
        "{:04}-{}-{}",
        date.get_full_year(),
        pad2(date.get_month() + 1),
        pad2(date.get_date())
    )
}
fn today_iso() -> String {
    iso_from_date(&Date::new_0())
}
fn valid_iso_shape(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 4 | 7) || c.is_ascii_digit())
}
fn clean_iso(value: &str, fallback: &str) -> String {
    let clean: String = western_digits(if value.is_empty() { fallback } else { value })
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .take(10)
        .collect();
    if valid_iso_shape(&clean) {
        clean
    } else {
        fallback.to_owned()
    }
}
fn parse_iso(value: &str, fallback: &str) -> Date {
    let clean = clean_iso(value, fallback);
    let year = clean[0..4].parse::<u32>().unwrap_or(1970);
    let month = clean[5..7].parse::<u32>().unwrap_or(1);
    let day = clean[8..10].parse::<u32>().unwrap_or(1);
    let date = Date::new_with_year_month_day(year, month.saturating_sub(1) as i32, day as i32);
    if date.get_full_year() == year && date.get_month() + 1 == month && date.get_date() == day {
        date
    } else if clean != fallback {
        parse_iso(fallback, &today_iso())
    } else {
        Date::new_0()
    }
}
fn month_label(year: u32, month: u32) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let label = MONTHS.get(month as usize).copied().unwrap_or("January");
    format!("{label} {year}")
}
fn days_in_month(year: u32, month: u32) -> u32 {
    let next_month = if month == 11 { 0 } else { month + 1 };
    let next_year = if month == 11 { year + 1 } else { year };
    let first_next = Date::new_with_year_month_day(next_year, next_month as i32, 1);
    first_next.set_date(0);
    first_next.get_date()
}

fn event(name: &str, bubbles: bool) -> JsValue {
    let ctor = property(&global(), "Event");
    let Ok(ctor) = ctor.dyn_into::<Function>() else {
        return JsValue::UNDEFINED;
    };
    let options = Object::new();
    let _ = set_property(options.as_ref(), "bubbles", &JsValue::from_bool(bubbles));
    let args = Array::new();
    args.push(&JsValue::from_str(name));
    args.push(options.as_ref());
    Reflect::construct(&ctor, &args).unwrap_or(JsValue::UNDEFINED)
}
fn dispatch(node: &JsValue, name: &str, bubbles: bool) {
    let ev = event(name, bubbles);
    if truthy(&ev) {
        let _ = call_method1(node, "dispatchEvent", &ev);
    }
}
fn set_date(text: &JsValue, native: &JsValue, value: &str) -> String {
    let clean = clean_iso(value, &today_iso());
    if truthy(text) {
        let _ = set_property(text, "value", &JsValue::from_str(&clean));
        dispatch(text, "input", true);
        dispatch(text, "change", true);
    }
    if truthy(native) {
        let _ = set_property(native, "value", &JsValue::from_str(&clean));
        dispatch(native, "change", true);
    }
    clean
}

const TRACE_PATCH: &str = "KGW_EXPLORER_SAFE_CONTROLS_TRACE_PATCH_R53B3";
const TRACE_OWNER: &str = "explorer-existing-safe-controls-owner";
const LAUNCH_DATE: &str = "2021-11-07";

fn q_in(section: &JsValue, id: &str) -> JsValue {
    query(section, &format!("#{id}"))
}

fn explorer_range(
    section: &JsValue,
    text_id: &str,
) -> Option<(JsValue, JsValue, JsValue, JsValue)> {
    if !matches!(text_id, "explorerFromDate" | "explorerToDate") {
        return None;
    }
    let from = q_in(section, "explorerFromDate");
    let to = q_in(section, "explorerToDate");
    if !truthy(&from) || !truthy(&to) {
        return None;
    }
    Some((
        from,
        to,
        q_in(section, "explorerFromDateNative"),
        q_in(section, "explorerToDateNative"),
    ))
}

fn set_date_plain(text: &JsValue, native: &JsValue, value: &str) -> String {
    let clean = crate::normalize_date_input_text(value);
    if truthy(text) {
        let _ = set_property(text, "value", &JsValue::from_str(&clean));
    }
    if truthy(native) && valid_iso_shape(&clean) {
        let _ = set_property(native, "value", &JsValue::from_str(&clean));
    }
    clean
}

fn explorer_i18n_text(key: &str, fallback: &str) -> String {
    for api_name in ["kgwI18n", "KGW_I18N", "i18n"] {
        let api = property(&global(), api_name);
        if let Ok(t) = property(&api, "t").dyn_into::<Function>()
            && let Ok(value) = t.call1(&api, &JsValue::from_str(key))
        {
            let text = raw(&value);
            if !text.trim().is_empty() && text != key {
                return text;
            }
        }
    }
    if let Ok(t) = property(&global(), "t").dyn_into::<Function>()
        && let Ok(value) = t.call1(&global(), &JsValue::from_str(key))
    {
        let text = raw(&value);
        if !text.trim().is_empty() && text != key {
            return text;
        }
    }
    fallback.to_owned()
}

fn explorer_invoke() -> Option<Function> {
    let win = window();
    let tauri = property(&win, "__TAURI__");
    for candidate in [
        property(&property(&tauri, "core"), "invoke"),
        property(&property(&tauri, "tauri"), "invoke"),
        property(&win, "__TAURI_INVOKE__"),
    ] {
        if let Ok(function) = candidate.dyn_into::<Function>() {
            return Some(function);
        }
    }
    None
}

fn trace_ui(action: &str, phase: &str, details: &[(&str, JsValue)]) {
    let Some(invoke) = explorer_invoke() else {
        return;
    };
    let detail_object = Object::new();
    for (key, value) in details {
        let _ = set_property(detail_object.as_ref(), key, value);
    }
    let nested = Object::new();
    for (key, value) in [
        ("patch", JsValue::from_str(TRACE_PATCH)),
        ("owner", JsValue::from_str(TRACE_OWNER)),
        ("action", JsValue::from_str(action)),
        ("phase", JsValue::from_str(phase)),
    ] {
        let _ = set_property(nested.as_ref(), key, &value);
    }
    let _ = set_property(nested.as_ref(), "details", detail_object.as_ref());
    let serialized = JSON::stringify(nested.as_ref())
        .ok()
        .map(|v| raw(v.as_ref()))
        .unwrap_or_else(|| "{}".to_owned());
    let args = Object::new();
    for (key, value) in [
        ("scope", "explorer"),
        ("net", "ui"),
        ("action", action),
        ("phase", phase),
    ] {
        let _ = set_property(args.as_ref(), key, &JsValue::from_str(value));
    }
    let _ = set_property(args.as_ref(), "details", &JsValue::from_str(&serialized));
    if let Ok(result) = invoke.call2(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_frontend_button_trace_v1"),
        args.as_ref(),
    ) {
        let promise = Promise::resolve(&result);
        let catch = Closure::wrap(Box::new(move |_error: JsValue| {}) as Box<dyn FnMut(JsValue)>);
        let _ = promise.catch(&catch);
        catch.forget();
    }
}

fn trace_calendar(phase: &str, details: &[(&str, JsValue)]) {
    trace_ui("explorer-calendar", phase, details);
}

fn close_calendar() {
    for node in query_all(
        &document(),
        ".kgw-calendar-popover[data-kgw-calendar-scope=\"explorer\"]",
    ) {
        remove(&node);
    }
    let owner = root();
    if !truthy(&owner) {
        return;
    }
    for node in query_all(&owner, "[data-kgw-calendar-open='1']") {
        let data = dataset(&node);
        let scope = raw(&property(&data, "kgwCalendarScope"));
        if scope.is_empty() || scope == "explorer" {
            let data_object: Object = data.unchecked_into();
            let _ = Reflect::delete_property(&data_object, &JsValue::from_str("kgwCalendarOpen"));
            let _ = Reflect::delete_property(&data_object, &JsValue::from_str("kgwCalendarScope"));
        }
    }
}
fn apply_preset(preset: &str, text_id: &str) {
    let today = parse_iso(&today_iso(), &today_iso());
    let mut from = iso_from_date(&today);
    let to = iso_from_date(&today);
    if preset == "last7" {
        today.set_time(today.get_time() - 6.0 * 86_400_000.0);
        from = iso_from_date(&today);
    } else if preset == "last30" {
        today.set_time(today.get_time() - 29.0 * 86_400_000.0);
        from = iso_from_date(&today);
    } else if preset == "thisMonth" {
        today.set_date(1);
        from = iso_from_date(&today);
    } else if preset == "sinceLaunch" {
        from = LAUNCH_DATE.to_owned();
    }
    let section = root();
    if let Some((from_node, to_node, from_native, to_native)) = explorer_range(&section, text_id) {
        set_date(&from_node, &from_native, &from);
        set_date(&to_node, &to_native, &to);
    } else {
        let active = q_in(&section, text_id);
        let value = if preset == "today" { &to } else { &from };
        set_date(&active, &JsValue::UNDEFINED, value);
    }
    close_calendar();
}
fn attach_popover(popover: &JsValue, anchor: &JsValue) {
    for node in query_all(&document(), ".kgw-calendar-popover") {
        remove(&node);
    }
    set_attr(popover, "data-kgw-calendar-scope", "explorer");
    add_class(popover, "kgw-calendar-popover-explorer");
    append(&property(&document(), "body"), popover);
    let rect = call_method0(anchor, "getBoundingClientRect").unwrap_or(JsValue::UNDEFINED);
    let iw = js_number(&property(&window(), "innerWidth"));
    let ih = js_number(&property(&window(), "innerHeight"));
    let width = 236.0_f64.min((iw - 24.0).max(218.0));
    let height = 326.0_f64.min(ih - 24.0);
    let left = js_number(&property(&rect, "left"))
        .max(12.0)
        .min((iw - width - 12.0).max(12.0));
    let bottom = js_number(&property(&rect, "bottom"));
    let top0 = bottom + 6.0;
    let top = if top0 + height <= ih - 12.0 {
        top0
    } else {
        (js_number(&property(&rect, "top")) - height - 6.0).max(12.0)
    };
    let style = property(popover, "style");
    for (k, v) in [
        ("position", "fixed".to_owned()),
        ("left", format!("{}px", left.round())),
        ("top", format!("{}px", top.round())),
        ("width", format!("{}px", width.round())),
        ("maxHeight", format!("{}px", height.round())),
        ("zIndex", "2147483000".to_owned()),
    ] {
        let _ = set_property(&style, k, &JsValue::from_str(&v));
    }
}
fn open_calendar(text: JsValue, native: JsValue, text_id: String, fallback: String) {
    if !truthy(&text) {
        return;
    }
    let host = {
        let c = closest(&text, ".explorer-date-combo, .kgw-analysis-date-field");
        if truthy(&c) { c } else { parent(&text) }
    };
    let was_open = raw(&property(&dataset(&host), "kgwCalendarOpen")) == "1";
    close_calendar();
    if was_open {
        return;
    }
    let _ = set_property(&dataset(&host), "kgwCalendarOpen", &JsValue::from_str("1"));
    let _ = set_property(
        &dataset(&host),
        "kgwCalendarScope",
        &JsValue::from_str("explorer"),
    );
    let active = parse_iso(&raw(&property(&text, "value")), &fallback);
    let year = std::rc::Rc::new(std::cell::Cell::new(active.get_full_year()));
    let month = std::rc::Rc::new(std::cell::Cell::new(active.get_month()));
    let popover = create("div");
    let _ = set_property(
        &popover,
        "className",
        &JsValue::from_str("kgw-calendar-popover"),
    );
    set_attr(&popover, "lang", "en-US");
    set_attr(&popover, "dir", "ltr");
    set_attr(&popover, "role", "dialog");
    set_attr(&popover, "aria-label", "Date picker");
    render_calendar(&popover, &text, &native, &text_id, year, month);
    attach_popover(&popover, &text);
}

fn render_calendar(
    popover: &JsValue,
    text: &JsValue,
    native: &JsValue,
    text_id: &str,
    year: std::rc::Rc<std::cell::Cell<u32>>,
    month: std::rc::Rc<std::cell::Cell<u32>>,
) {
    let _ = set_property(popover, "textContent", &JsValue::from_str(""));
    let header = create("div");
    let _ = set_property(
        &header,
        "className",
        &JsValue::from_str("kgw-calendar-header"),
    );
    let prev = create("button");
    let next = create("button");
    let title = create("div");
    for (button, label, aria) in [(&prev, "‹", "Previous month"), (&next, "›", "Next month")] {
        let _ = set_property(button, "type", &JsValue::from_str("button"));
        let _ = set_property(button, "className", &JsValue::from_str("kgw-calendar-nav"));
        let _ = set_property(button, "textContent", &JsValue::from_str(label));
        set_attr(button, "aria-label", aria);
    }
    let _ = set_property(
        &title,
        "className",
        &JsValue::from_str("kgw-calendar-title"),
    );
    let _ = set_property(
        &title,
        "textContent",
        &JsValue::from_str(&month_label(year.get(), month.get())),
    );
    {
        let p = popover.clone();
        let t = text.clone();
        let n = native.clone();
        let id = text_id.to_owned();
        let y = year.clone();
        let m = month.clone();
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let _ = call_method0(&ev, "preventDefault");
            let mut mm = m.get();
            let mut yy = y.get();
            trace_calendar(
                "r53b3-explorer-calendar-prev-click",
                &[
                    ("trusted", property(&ev, "isTrusted")),
                    ("textId", JsValue::from_str(&id)),
                    ("year", JsValue::from_f64(yy as f64)),
                    ("month", JsValue::from_f64(mm as f64)),
                ],
            );
            if mm == 0 {
                mm = 11;
                yy = yy.saturating_sub(1);
            } else {
                mm -= 1;
            }
            m.set(mm);
            y.set(yy);
            render_calendar(&p, &t, &n, &id, y.clone(), m.clone());
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&prev, "addEventListener")
            && f.call2(
                &prev,
                &JsValue::from_str("click"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
    {
        let p = popover.clone();
        let t = text.clone();
        let n = native.clone();
        let id = text_id.to_owned();
        let y = year.clone();
        let m = month.clone();
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let _ = call_method0(&ev, "preventDefault");
            let mut mm = m.get();
            let mut yy = y.get();
            trace_calendar(
                "r53b3-explorer-calendar-next-click",
                &[
                    ("trusted", property(&ev, "isTrusted")),
                    ("textId", JsValue::from_str(&id)),
                    ("year", JsValue::from_f64(yy as f64)),
                    ("month", JsValue::from_f64(mm as f64)),
                ],
            );
            if mm == 11 {
                mm = 0;
                yy += 1;
            } else {
                mm += 1;
            }
            m.set(mm);
            y.set(yy);
            render_calendar(&p, &t, &n, &id, y.clone(), m.clone());
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&next, "addEventListener")
            && f.call2(
                &next,
                &JsValue::from_str("click"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
    append(&header, &prev);
    append(&header, &title);
    append(&header, &next);
    append(popover, &header);
    let presets = create("div");
    let _ = set_property(
        &presets,
        "className",
        &JsValue::from_str("kgw-calendar-presets"),
    );
    for (value, label) in [
        ("today", "Today"),
        ("last7", "Last 7 Days"),
        ("last30", "Last 30 Days"),
        ("thisMonth", "This Month"),
        ("sinceLaunch", "Since Launch"),
    ] {
        let b = create("button");
        let _ = set_property(&b, "type", &JsValue::from_str("button"));
        let _ = set_property(&b, "className", &JsValue::from_str("kgw-calendar-preset"));
        let _ = set_property(&b, "textContent", &JsValue::from_str(label));
        let v = value.to_owned();
        let label = label.to_owned();
        let id = text_id.to_owned();
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let _ = call_method0(&ev, "preventDefault");
            trace_calendar(
                "r53b3-explorer-calendar-preset-click",
                &[
                    ("trusted", property(&ev, "isTrusted")),
                    ("textId", JsValue::from_str(&id)),
                    ("preset", JsValue::from_str(&v)),
                    ("label", JsValue::from_str(&label)),
                ],
            );
            apply_preset(&v, &id);
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&b, "addEventListener")
            && f.call2(&b, &JsValue::from_str("click"), cb.as_ref().unchecked_ref())
                .is_ok()
        {
            cb.forget();
        }
        append(&presets, &b);
    }
    append(popover, &presets);
    let weekdays = create("div");
    let _ = set_property(
        &weekdays,
        "className",
        &JsValue::from_str("kgw-calendar-weekdays"),
    );
    for day in ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] {
        let s = create("span");
        let _ = set_property(&s, "textContent", &JsValue::from_str(day));
        append(&weekdays, &s);
    }
    append(popover, &weekdays);
    let grid = create("div");
    let _ = set_property(&grid, "className", &JsValue::from_str("kgw-calendar-grid"));
    let first = Date::new_with_year_month_day(year.get(), month.get() as i32, 1);
    for _ in 0..first.get_day() {
        let e = create("span");
        let _ = set_property(&e, "className", &JsValue::from_str("kgw-calendar-empty"));
        append(&grid, &e);
    }
    let selected = clean_iso(&raw(&property(text, "value")), &today_iso());
    let today = today_iso();
    for day in 1..=days_in_month(year.get(), month.get()) {
        let date = Date::new_with_year_month_day(year.get(), month.get() as i32, day as i32);
        let iso = iso_from_date(&date);
        let b = create("button");
        let _ = set_property(&b, "type", &JsValue::from_str("button"));
        let _ = set_property(&b, "className", &JsValue::from_str("kgw-calendar-day"));
        let _ = set_property(&b, "textContent", &JsValue::from_str(&day.to_string()));
        let _ = set_property(&dataset(&b), "iso", &JsValue::from_str(&iso));
        if iso == selected {
            add_class(&b, "is-selected")
        }
        if iso == today {
            add_class(&b, "is-today")
        }
        let t = text.clone();
        let n = native.clone();
        let i = iso.clone();
        let id = text_id.to_owned();
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let _ = call_method0(&ev, "preventDefault");
            trace_calendar(
                "r53b3-explorer-calendar-day-click",
                &[
                    ("trusted", property(&ev, "isTrusted")),
                    ("textId", JsValue::from_str(&id)),
                    ("iso", JsValue::from_str(&i)),
                ],
            );
            set_date(&t, &n, &i);
            close_calendar();
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&b, "addEventListener")
            && f.call2(&b, &JsValue::from_str("click"), cb.as_ref().unchecked_ref())
                .is_ok()
        {
            cb.forget();
        }
        append(&grid, &b);
    }
    append(popover, &grid);
    let footer = create("div");
    let _ = set_property(
        &footer,
        "className",
        &JsValue::from_str("kgw-calendar-footer"),
    );
    let close = create("button");
    let _ = set_property(&close, "type", &JsValue::from_str("button"));
    let _ = set_property(
        &close,
        "className",
        &JsValue::from_str("kgw-calendar-close"),
    );
    let _ = set_property(
        &close,
        "textContent",
        &JsValue::from_str(&explorer_i18n_text("calendar.close", "Close")),
    );
    let id = text_id.to_owned();
    let cb = Closure::wrap(Box::new(move |ev: JsValue| {
        let _ = call_method0(&ev, "preventDefault");
        trace_calendar(
            "r53b3-explorer-calendar-close-click",
            &[
                ("trusted", property(&ev, "isTrusted")),
                ("textId", JsValue::from_str(&id)),
            ],
        );
        close_calendar();
    }) as Box<dyn FnMut(JsValue)>);
    if let Ok(f) = method(&close, "addEventListener")
        && f.call2(
            &close,
            &JsValue::from_str("click"),
            cb.as_ref().unchecked_ref(),
        )
        .is_ok()
    {
        cb.forget();
    }
    append(&footer, &close);
    append(popover, &footer);
}

fn trace_date_event(phase: &str, event: &JsValue, text_id: &str, extra: &[(&str, JsValue)]) {
    let mut details = vec![
        ("trusted", property(event, "isTrusted")),
        ("textId", JsValue::from_str(text_id)),
    ];
    details.extend(extra.iter().cloned());
    trace_ui("explorer-date", phase, &details);
}

fn bind_date_control(
    section: &JsValue,
    text_id: &'static str,
    native_id: &'static str,
    button_id: &'static str,
    fallback: String,
) -> bool {
    let text = q_in(section, text_id);
    let native = q_in(section, native_id);
    let button = q_in(section, button_id);
    if !truthy(&text) || !truthy(&button) {
        return false;
    }
    let current = raw(&property(&text, "value"));
    set_date_plain(
        &text,
        &native,
        if current.is_empty() {
            &fallback
        } else {
            &current
        },
    );
    if raw(&property(&dataset(&text), "kgwDateBound")) == "1" {
        return false;
    }
    let _ = set_property(&dataset(&text), "kgwDateBound", &JsValue::from_str("1"));
    set_attr(&text, "lang", "en-US");
    set_attr(&text, "dir", "ltr");
    if truthy(&native) {
        set_attr(&native, "lang", "en-US");
        set_attr(&native, "dir", "ltr");
    }
    {
        let text_for_cb = text.clone();
        let native_for_cb = native.clone();
        let cb = Closure::wrap(Box::new(move |event: JsValue| {
            let value = raw(&property(&text_for_cb, "value"));
            trace_date_event(
                "r53b3-explorer-date-text-input",
                &event,
                text_id,
                &[("value", JsValue::from_str(&value))],
            );
            let clean = crate::normalize_date_input_text(&value);
            if clean != value {
                let _ = set_property(&text_for_cb, "value", &JsValue::from_str(&clean));
            }
            if truthy(&native_for_cb) && valid_iso_shape(&clean) {
                let _ = set_property(&native_for_cb, "value", &JsValue::from_str(&clean));
            }
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(add) = method(&text, "addEventListener")
            && add
                .call2(
                    &text,
                    &JsValue::from_str("input"),
                    cb.as_ref().unchecked_ref(),
                )
                .is_ok()
        {
            cb.forget();
        }
    }
    {
        let text_for_cb = text.clone();
        let native_for_cb = native.clone();
        let fallback = fallback.clone();
        let cb = Closure::wrap(Box::new(move |event: JsValue| {
            let value = raw(&property(&text_for_cb, "value"));
            trace_date_event(
                "r53b3-explorer-date-text-blur",
                &event,
                text_id,
                &[("value", JsValue::from_str(&value))],
            );
            set_date_plain(
                &text_for_cb,
                &native_for_cb,
                if value.is_empty() { &fallback } else { &value },
            );
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(add) = method(&text, "addEventListener")
            && add
                .call2(
                    &text,
                    &JsValue::from_str("blur"),
                    cb.as_ref().unchecked_ref(),
                )
                .is_ok()
        {
            cb.forget();
        }
    }
    if truthy(&native) {
        let text = text.clone();
        let native_for_cb = native.clone();
        let fallback = fallback.clone();
        let cb = Closure::wrap(Box::new(move |event: JsValue| {
            let value = raw(&property(&native_for_cb, "value"));
            trace_date_event(
                "r53b3-explorer-date-native-change",
                &event,
                text_id,
                &[
                    ("nativeId", JsValue::from_str(native_id)),
                    ("value", JsValue::from_str(&value)),
                ],
            );
            set_date_plain(
                &text,
                &native_for_cb,
                if value.is_empty() { &fallback } else { &value },
            );
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(add) = method(&native, "addEventListener")
            && add
                .call2(
                    &native,
                    &JsValue::from_str("change"),
                    cb.as_ref().unchecked_ref(),
                )
                .is_ok()
        {
            cb.forget();
        }
    }
    set_attr(&button, "lang", "en-US");
    set_attr(&button, "dir", "ltr");
    {
        let text = text.clone();
        let native = native.clone();
        let fallback = fallback.clone();
        let cb = Closure::wrap(Box::new(move |event: JsValue| {
            let _ = call_method0(&event, "preventDefault");
            let _ = call_method0(&event, "stopPropagation");
            trace_calendar(
                "r53b3-explorer-calendar-button-click",
                &[
                    ("trusted", property(&event, "isTrusted")),
                    ("textId", JsValue::from_str(text_id)),
                    ("nativeId", JsValue::from_str(native_id)),
                    ("buttonId", JsValue::from_str(button_id)),
                ],
            );
            let value = raw(&property(&text, "value"));
            set_date_plain(
                &text,
                &native,
                if value.is_empty() { &fallback } else { &value },
            );
            open_calendar(
                text.clone(),
                native.clone(),
                text_id.to_owned(),
                fallback.clone(),
            );
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(add) = method(&button, "addEventListener")
            && add
                .call2(
                    &button,
                    &JsValue::from_str("click"),
                    cb.as_ref().unchecked_ref(),
                )
                .is_ok()
        {
            cb.forget();
        }
    }
    true
}
#[wasm_bindgen(js_name = explorerDefaultDates)]
pub fn explorer_default_dates(section: JsValue) -> bool {
    if !truthy(&section) {
        return false;
    }
    let today = today_iso();
    let from = bind_date_control(
        &section,
        "explorerFromDate",
        "explorerFromDateNative",
        "explorerFromDatePicker",
        LAUNCH_DATE.to_owned(),
    );
    let to = bind_date_control(
        &section,
        "explorerToDate",
        "explorerToDateNative",
        "explorerToDatePicker",
        today,
    );
    from || to
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn western_digits_are_normalized() {
        assert_eq!(western_digits("٢٠٢٦-۰۹-۲۸"), "2026-09-28");
    }
    #[test]
    fn iso_shape_is_strict() {
        assert!(valid_iso_shape("2026-09-28"));
        assert!(!valid_iso_shape("28-09-2026"));
    }
    #[test]
    fn month_labels_are_stable() {
        assert_eq!(month_label(2026, 8), "September 2026");
    }
    #[test]
    fn launch_date_contract_is_stable() {
        assert_eq!(LAUNCH_DATE, "2021-11-07");
    }
}
