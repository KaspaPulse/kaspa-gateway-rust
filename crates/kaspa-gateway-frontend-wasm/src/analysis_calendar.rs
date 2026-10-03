use js_sys::{Array, Date, Function, Object, Reflect};
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
    call_method1(
        &document(),
        "getElementById",
        &JsValue::from_str("analysis"),
    )
    .unwrap_or(JsValue::UNDEFINED)
}
fn query(target: &JsValue, selector: &str) -> JsValue {
    call_method1(target, "querySelector", &JsValue::from_str(selector))
        .unwrap_or(JsValue::UNDEFINED)
}
fn q(selector: &str) -> JsValue {
    let owner = root();
    if truthy(&owner) {
        query(&owner, selector)
    } else {
        JsValue::UNDEFINED
    }
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
fn remove_class(node: &JsValue, class: &str) {
    let _ = call_method1(&class_list(node), "remove", &JsValue::from_str(class));
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
fn range() -> Option<(JsValue, JsValue, JsValue, JsValue)> {
    let from = q("#analysisFromDate");
    let to = q("#analysisToDate");
    if !truthy(&from) || !truthy(&to) {
        return None;
    }
    Some((
        from,
        to,
        q("#analysisFromDateNative"),
        q("#analysisToDateNative"),
    ))
}
pub(crate) fn ensure_today(force: bool) -> String {
    let Some((_from, to, _from_native, to_native)) = range() else {
        return String::new();
    };
    let today = today_iso();
    let touched = raw(&property(&dataset(&to), "kgwAnalysisUserTouched")) == "true"
        || (truthy(&to_native)
            && raw(&property(&dataset(&to_native), "kgwAnalysisUserTouched")) == "true");
    let current = raw(&property(&to, "value"));
    if (force || !touched || current.is_empty() || current != today) && (force || !touched) {
        let _ = set_property(
            &dataset(&to),
            "kgwAnalysisOwnerDefaultR23",
            &JsValue::from_str("today"),
        );
        if truthy(&to_native) {
            let _ = set_property(
                &dataset(&to_native),
                "kgwAnalysisOwnerDefaultR23",
                &JsValue::from_str("today"),
            );
        }
        return set_date(&to, &to_native, &today);
    }
    clean_iso(&current, &today)
}

fn bind_date_touch(node: JsValue) {
    if !truthy(&node) || raw(&property(&dataset(&node), "kgwAnalysisTouchR23")) == "true" {
        return;
    }
    let _ = set_property(
        &dataset(&node),
        "kgwAnalysisTouchR23",
        &JsValue::from_str("true"),
    );
    for event_name in ["input", "change"] {
        let n = node.clone();
        let cb = Closure::wrap(Box::new(move |event: JsValue| {
            if super::analysis_binding::event_trusted(&event) {
                let _ = set_property(
                    &dataset(&n),
                    "kgwAnalysisUserTouched",
                    &JsValue::from_str("true"),
                );
            }
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&node, "addEventListener")
            && f.call2(
                &node,
                &JsValue::from_str(event_name),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
}

fn calendar_button(text: &JsValue, native: &JsValue, role: &'static str) -> JsValue {
    if !truthy(text) {
        return JsValue::UNDEFINED;
    }
    let host = {
        let c = closest(text, ".kgw-analysis-date-field");
        if truthy(&c) { c } else { parent(text) }
    };
    let mut button = if truthy(&host) {
        query(&host, ".analysis-calendar-btn")
    } else {
        JsValue::UNDEFINED
    };
    if !truthy(&button) {
        button = create("button");
        let _ = set_property(&button, "type", &JsValue::from_str("button"));
        let _ = set_property(
            &button,
            "className",
            &JsValue::from_str("analysis-calendar-btn kgw-analysis-calendar-created-r23"),
        );
        let _ = set_property(&button, "textContent", &JsValue::from_str("📅"));
        set_attr(
            &button,
            "aria-label",
            if role == "from" {
                "Open from date calendar"
            } else {
                "Open to date calendar"
            },
        );
        let text2 = text.clone();
        let native2 = native.clone();
        let cb = Closure::wrap(Box::new(move |_event: JsValue| {
            if truthy(&native2) {
                if let Ok(show) = method(&native2, "showPicker") {
                    let _ = show.call0(&native2);
                } else {
                    let _ = call_method0(&native2, "focus");
                    let _ = call_method0(&native2, "click");
                }
            } else {
                let _ = call_method0(&text2, "focus");
            }
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&button, "addEventListener")
            && f.call2(
                &button,
                &JsValue::from_str("click"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
    add_class(&button, "kgw-analysis-filter-calendar-button-r23");
    add_class(
        &button,
        if role == "from" {
            "kgw-analysis-filter-from-calendar-r23"
        } else {
            "kgw-analysis-filter-to-calendar-r23"
        },
    );
    set_attr(&button, "data-kgw-analysis-calendar-role-r23", role);
    set_attr(&button, "type", "button");
    button
}

fn make_date_unit(role: &str, text: &JsValue, native: &JsValue, button: &JsValue) -> JsValue {
    let unit = create("span");
    let _ = set_property(
        &unit,
        "className",
        &JsValue::from_str(&format!(
            "kgw-analysis-filter-date-unit-r23 kgw-analysis-filter-date-{role}-r23"
        )),
    );
    set_attr(&unit, "data-kgw-analysis-filter-date-unit-r23", role);
    if truthy(text) {
        add_class(text, "kgw-analysis-filter-date-text-r23");
        append(&unit, text);
    }
    if truthy(button) {
        add_class(button, "kgw-analysis-filter-date-button-r23");
        append(&unit, button);
    }
    if truthy(native) {
        add_class(native, "kgw-analysis-filter-date-native-r23");
        append(&unit, native);
    }
    unit
}
fn append_label(bar: &JsValue, text: &str, class: &str) {
    let label = create("span");
    let _ = set_property(
        &label,
        "className",
        &JsValue::from_str(&format!("kgw-analysis-filter-label-r23 {class}")),
    );
    let _ = set_property(&label, "textContent", &JsValue::from_str(text));
    append(bar, &label);
}
fn append_control(bar: &JsValue, node: &JsValue, class: &str, label: &str) {
    if !truthy(node) {
        return;
    }
    add_class(node, "kgw-analysis-filter-control-r23");
    add_class(node, class);
    set_attr(node, "data-kgw-analysis-filter-owner-r23", "true");
    set_attr(node, "aria-label", label);
    set_attr(node, "title", label);
    append(bar, node);
}
pub(crate) fn normalize_layout() {
    let owner = root();
    if !truthy(&owner) {
        return;
    }
    let Some((from, to, from_native, to_native)) = range() else {
        return;
    };
    let type_node = q("#analysisType");
    let direction = q("#analysisDirection");
    let search = q("#analysisSearch");
    let filter = q("#analysisFilter");
    let reset = q("#analysisResetFilter");
    let from_button = calendar_button(&from, &from_native, "from");
    let to_button = calendar_button(&to, &to_native, "to");
    let mut host = parent(&filter);
    if !truthy(&host) || Object::is(&host, &owner) {
        host = parent(&search);
    }
    if !truthy(&host) {
        host = owner.clone();
    }
    let bar = create("div");
    let _ = set_property(
        &bar,
        "className",
        &JsValue::from_str("kgw-analysis-filter-bar-rebuild-r23"),
    );
    set_attr(&bar, "data-kgw-analysis-filter-bar-owner-r23", "true");
    append_label(&bar, "From:", "kgw-analysis-filter-from-label-r23");
    append(
        &bar,
        &make_date_unit("from", &from, &from_native, &from_button),
    );
    append_label(&bar, "To:", "kgw-analysis-filter-to-label-r23");
    append(&bar, &make_date_unit("to", &to, &to_native, &to_button));
    append_control(&bar, &type_node, "kgw-analysis-filter-type-r23", "Type");
    append_control(
        &bar,
        &direction,
        "kgw-analysis-filter-direction-r23",
        "Direction",
    );
    append_control(
        &bar,
        &search,
        "kgw-analysis-filter-search-r23",
        "Search by Address or Transaction",
    );
    append_control(&bar, &filter, "kgw-analysis-filter-apply-r23", "Filter");
    append_control(
        &bar,
        &reset,
        "kgw-analysis-filter-reset-r23",
        "Reset Filter",
    );
    if truthy(&search) {
        let _ = set_property(
            &search,
            "placeholder",
            &JsValue::from_str("Search by Address/Transaction..."),
        );
    }
    while truthy(&property(&host, "firstChild")) {
        remove(&property(&host, "firstChild"));
    }
    for c in [
        "kgw-analysis-filter-calendar-r18",
        "kgw-analysis-filter-bar-owner-r20",
        "kgw-analysis-date-calendar-owner-r22",
    ] {
        remove_class(&host, c);
    }
    add_class(&host, "kgw-analysis-filter-host-r23");
    set_attr(&host, "data-kgw-analysis-filter-host-r23", "true");
    append(&host, &bar);
    ensure_today(false);
}

fn close_calendar() {
    for node in query_all(
        &document(),
        ".kgw-calendar-popover[data-kgw-calendar-scope=\"analysis\"]",
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
        if scope.is_empty() || scope == "analysis" {
            let data_object: Object = data.unchecked_into();
            let _ = Reflect::delete_property(&data_object, &JsValue::from_str("kgwCalendarOpen"));
            let _ = Reflect::delete_property(&data_object, &JsValue::from_str("kgwCalendarScope"));
        }
    }
}
fn apply_preset(preset: &str) {
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
        from = "2021-11-07".to_owned();
    }
    if let Some((from_node, to_node, from_native, to_native)) = range() {
        set_date(&from_node, &from_native, &from);
        set_date(&to_node, &to_native, &to);
    }
    super::analysis_view::render_rows();
    close_calendar();
}
fn attach_popover(popover: &JsValue, anchor: &JsValue) {
    for node in query_all(&document(), ".kgw-calendar-popover") {
        remove(&node);
    }
    set_attr(popover, "data-kgw-calendar-scope", "analysis");
    add_class(popover, "kgw-calendar-popover-analysis");
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
fn trace_calendar(phase: &str, details: &[(&str, JsValue)]) {
    super::analysis_binding::trace(
        "analysis-calendar",
        phase,
        super::analysis_binding::details(details),
    );
}
fn open_calendar(text: JsValue, native: JsValue, text_id: String) {
    if !truthy(&text) {
        return;
    }
    let host = {
        let c = closest(&text, ".kgw-analysis-date-field");
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
        &JsValue::from_str("analysis"),
    );
    let active = parse_iso(&raw(&property(&text, "value")), &today_iso());
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
            if mm == 0 {
                mm = 11;
                yy -= 1
            } else {
                mm -= 1
            }
            m.set(mm);
            y.set(yy);
            trace_calendar(
                "r50b-analysis-calendar-prev-click",
                &[("textId", JsValue::from_str(&id))],
            );
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
            if mm == 11 {
                mm = 0;
                yy += 1
            } else {
                mm += 1
            }
            m.set(mm);
            y.set(yy);
            trace_calendar(
                "r50b-analysis-calendar-next-click",
                &[("textId", JsValue::from_str(&id))],
            );
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
        let id = text_id.to_owned();
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let _ = call_method0(&ev, "preventDefault");
            trace_calendar(
                "r50b-analysis-calendar-preset-click",
                &[
                    ("textId", JsValue::from_str(&id)),
                    ("preset", JsValue::from_str(&v)),
                ],
            );
            apply_preset(&v);
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
                "r50b-analysis-calendar-day-click",
                &[
                    ("textId", JsValue::from_str(&id)),
                    ("iso", JsValue::from_str(&i)),
                ],
            );
            set_date(&t, &n, &i);
            super::analysis_view::render_rows();
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
    let _ = set_property(&close, "textContent", &JsValue::from_str("Close"));
    let id = text_id.to_owned();
    let cb = Closure::wrap(Box::new(move |ev: JsValue| {
        let _ = call_method0(&ev, "preventDefault");
        trace_calendar(
            "r50b-analysis-calendar-close-click",
            &[("textId", JsValue::from_str(&id))],
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

fn bind_pair(text_id: &'static str, native_id: &'static str, role: &'static str) {
    let text = q(&format!("#{text_id}"));
    if !truthy(&text) {
        return;
    }
    let native = q(&format!("#{native_id}"));
    set_attr(&text, "lang", "en-US");
    set_attr(&text, "dir", "ltr");
    if truthy(&native) {
        set_attr(&native, "lang", "en-US");
        set_attr(&native, "dir", "ltr");
    }
    let button = calendar_button(&text, &native, role);
    if truthy(&button) && raw(&property(&dataset(&button), "kgwCalendarBound")) != "true" {
        let _ = set_property(
            &dataset(&button),
            "kgwCalendarBound",
            &JsValue::from_str("true"),
        );
        let _ = set_property(&dataset(&button), "dateFor", &JsValue::from_str(text_id));
        let _ = set_property(
            &dataset(&button),
            "nativeFor",
            &JsValue::from_str(native_id),
        );
        let t = text.clone();
        let n = native.clone();
        let id = text_id.to_owned();
        let role2 = role.to_owned();
        let cb = Closure::wrap(Box::new(move |ev: JsValue| {
            let _ = call_method0(&ev, "preventDefault");
            let _ = call_method0(&ev, "stopPropagation");
            trace_calendar(
                "r50b-analysis-calendar-button-click",
                &[
                    ("textId", JsValue::from_str(&id)),
                    ("role", JsValue::from_str(&role2)),
                ],
            );
            let value = {
                let v = raw(&property(&t, "value"));
                if v.is_empty() { today_iso() } else { v }
            };
            set_date(&t, &n, &value);
            open_calendar(t.clone(), n.clone(), id.clone());
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&button, "addEventListener")
            && f.call2(
                &button,
                &JsValue::from_str("click"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
    if truthy(&native) && raw(&property(&dataset(&native), "kgwCalendarNativeBound")) != "true" {
        let _ = set_property(
            &dataset(&native),
            "kgwCalendarNativeBound",
            &JsValue::from_str("true"),
        );
        let t = text.clone();
        let n = native.clone();
        let cb = Closure::wrap(Box::new(move |_ev: JsValue| {
            let current = raw(&property(&t, "value"));
            let clean = clean_iso(&raw(&property(&n, "value")), &current);
            let _ = set_property(&t, "value", &JsValue::from_str(&clean));
            super::analysis_view::render_rows();
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&native, "addEventListener")
            && f.call2(
                &native,
                &JsValue::from_str("change"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
    if raw(&property(&dataset(&text), "kgwCalendarTextBound")) != "true" {
        let _ = set_property(
            &dataset(&text),
            "kgwCalendarTextBound",
            &JsValue::from_str("true"),
        );
        let t = text.clone();
        let cb = Closure::wrap(Box::new(move |_ev: JsValue| {
            let current = raw(&property(&t, "value"));
            let clean = clean_iso(&current, &current);
            let _ = set_property(&t, "value", &JsValue::from_str(&clean));
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&text, "addEventListener")
            && f.call2(
                &text,
                &JsValue::from_str("input"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
        let cb = Closure::wrap(Box::new(move |_ev: JsValue| {
            super::analysis_view::render_rows();
        }) as Box<dyn FnMut(JsValue)>);
        if let Ok(f) = method(&text, "addEventListener")
            && f.call2(
                &text,
                &JsValue::from_str("change"),
                cb.as_ref().unchecked_ref(),
            )
            .is_ok()
        {
            cb.forget();
        }
    }
    bind_date_touch(text);
    if truthy(&native) {
        bind_date_touch(native);
    }
}
pub(crate) fn install() {
    if !truthy(&root()) {
        return;
    }
    normalize_layout();
    bind_pair("analysisFromDate", "analysisFromDateNative", "from");
    bind_pair("analysisToDate", "analysisToDateNative", "to");
    ensure_today(false);
}

#[wasm_bindgen(js_name = analysisCalendarInstall)]
pub fn install_export() {
    install();
}
#[wasm_bindgen(js_name = analysisCalendarResetToToday)]
pub fn reset_to_today_export() -> String {
    ensure_today(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn western_digits_are_normalized() {
        assert_eq!(western_digits("٢٠٢٦-۰۹-۲۷"), "2026-09-27");
    }
    #[test]
    fn iso_shape_is_strict() {
        assert!(valid_iso_shape("2026-09-27"));
        assert!(!valid_iso_shape("27-09-2026"));
    }
    #[test]
    fn clean_iso_falls_back() {
        assert_eq!(clean_iso("٢٠٢٦/09/27", "2026-01-01"), "2026-01-01");
        assert_eq!(clean_iso("٢٠٢٦-۰۹-۲۷", "2026-01-01"), "2026-09-27");
    }
    #[test]
    fn month_labels_are_stable() {
        assert_eq!(month_label(2026, 8), "September 2026");
    }
}
