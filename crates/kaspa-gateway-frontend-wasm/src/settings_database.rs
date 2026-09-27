use js_sys::{Array, Date, Function, Intl::NumberFormat, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

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

fn call1(target: &JsValue, name: &str, arg: &JsValue) -> JsValue {
    function(target, name)
        .and_then(|callback| callback.call1(target, arg).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn closest(target: &JsValue, selector: &str) -> JsValue {
    call1(target, "closest", &JsValue::from_str(selector))
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
fn error_text(error: &JsValue) -> String {
    let message = property(error, "message");
    if crate::js_boolean(&message) {
        text(&message)
    } else {
        text(error)
    }
}

fn by_id(id: &str) -> JsValue {
    call1(&document(), "getElementById", &JsValue::from_str(id))
}

fn sentinel_modified(raw: &str) -> Option<String> {
    if raw.is_empty() {
        Some("--".to_owned())
    } else if matches!(raw, "missing" | "unknown") {
        Some(raw.to_owned())
    } else {
        None
    }
}

fn format_modified(value: &JsValue) -> String {
    let raw = text(value);
    if let Some(sentinel) = sentinel_modified(&raw) {
        return sentinel;
    }
    let ms = crate::js_number(&JsValue::from_str(&raw));
    if !ms.is_finite() {
        return raw;
    }
    let date = Date::new(&JsValue::from_f64(ms));
    if !date.get_time().is_finite() {
        return raw;
    }
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

fn format_size(value: &JsValue) -> String {
    let source = if crate::js_boolean(value) {
        value.clone()
    } else {
        JsValue::from_f64(0.0)
    };
    let number = crate::js_number(&source);
    let locales = Array::new();
    let options = Object::new();
    set_property(
        options.as_ref(),
        "minimumFractionDigits",
        &JsValue::from_f64(2.0),
    );
    set_property(
        options.as_ref(),
        "maximumFractionDigits",
        &JsValue::from_f64(2.0),
    );
    let formatter = NumberFormat::new(&locales, &options);
    formatter
        .format()
        .call1(&JsValue::UNDEFINED, &JsValue::from_f64(number))
        .ok()
        .map(|value| text(&value))
        .unwrap_or_else(|| format!("{number:.2}"))
}

fn append_single_row(tbody: &JsValue, message: &str) {
    set_property(tbody, "innerHTML", &JsValue::from_str(""));
    let tr = create_element("tr");
    let td = create_element("td");
    set_property(&td, "colSpan", &JsValue::from_f64(4.0));
    set_property(&td, "textContent", &JsValue::from_str(message));
    append(&tr, &td);
    append(tbody, &tr);
}

fn no_database_files_text() -> String {
    let translator = property(&window(), "kgwT");
    if let Ok(translator) = translator.dyn_into::<Function>()
        && let Ok(value) = translator.call1(
            &window(),
            &JsValue::from_str("settings.noDatabaseFilesFound"),
        )
    {
        let translated = text(&value);
        if !translated.is_empty() {
            return translated;
        }
    }
    "No database files found.".to_owned()
}
#[wasm_bindgen(js_name = settingsDatabaseRenderRows)]
pub fn render_rows(rows: JsValue) {
    let tbody = by_id("settingsDatabaseRows");
    if !is_present(&tbody) {
        return;
    }
    set_property(&tbody, "innerHTML", &JsValue::from_str(""));
    if !Array::is_array(&rows) || Array::from(&rows).length() == 0 {
        append_single_row(&tbody, &no_database_files_text());
        return;
    }

    for row in Array::from(&rows).iter() {
        let tr = create_element("tr");
        let file = property(&row, "file");
        let file = if crate::js_boolean(&file) {
            text(&file)
        } else {
            "--".to_owned()
        };
        let details = property(&row, "details");
        let details = if crate::js_boolean(&details) {
            text(&details)
        } else if crate::js_boolean(&property(&row, "exists")) {
            "available".to_owned()
        } else {
            "missing".to_owned()
        };
        let cells = [
            file,
            format_size(&property(&row, "size_kb")),
            format_modified(&property(&row, "last_modified")),
            details,
        ];
        for value in cells {
            let td = create_element("td");
            set_property(&td, "textContent", &JsValue::from_str(&value));
            append(&tr, &td);
        }
        append(&tbody, &tr);
    }
}

fn invoke_api() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let core_invoke = property(&core, "invoke");
    if let Ok(invoke) = core_invoke.dyn_into::<Function>() {
        return Some(invoke);
    }
    let legacy = property(&tauri, "tauri");
    let legacy_invoke = property(&legacy, "invoke");
    if let Ok(invoke) = legacy_invoke.dyn_into::<Function>() {
        return Some(invoke);
    }
    property(&window(), "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .ok()
}
async fn refresh_internal() {
    let tbody = by_id("settingsDatabaseRows");
    if !is_present(&tbody) {
        return;
    }
    let Some(invoke) = invoke_api() else {
        append_single_row(&tbody, "Tauri invoke API is not available.");
        return;
    };
    append_single_row(&tbody, "Loading database status...");

    let result = invoke.call1(
        &JsValue::UNDEFINED,
        &JsValue::from_str("kgw_settings_database_status"),
    );
    match result {
        Ok(value) => match JsFuture::from(Promise::resolve(&value)).await {
            Ok(rows) => render_rows(rows),
            Err(error) => append_single_row(
                &tbody,
                &format!("Failed to load database status: {}", error_text(&error)),
            ),
        },
        Err(error) => append_single_row(
            &tbody,
            &format!("Failed to load database status: {}", error_text(&error)),
        ),
    }
}
fn spawn_refresh() {
    spawn_local(async {
        refresh_internal().await;
    });
}

fn schedule_refresh(delay_ms: f64) {
    let callback = Closure::<dyn FnMut()>::new(spawn_refresh);
    if let Some(timeout) = function(&window(), "setTimeout") {
        let _ = timeout.call2(
            &window(),
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(delay_ms),
        );
    }
    callback.forget();
}

#[wasm_bindgen(js_name = settingsDatabaseRefresh)]
pub async fn refresh() {
    refresh_internal().await;
}

#[wasm_bindgen(js_name = settingsDatabaseInstall)]
pub fn install() -> Result<(), JsValue> {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwSettingsDatabaseStatusInstalled")) {
        return Ok(());
    }
    set_property(&win, "__kgwSettingsDatabaseStatusInstalled", &JsValue::TRUE);
    let refresh_click = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        if !is_present(&closest(&target, "#settingsDbRefresh")) {
            return;
        }
        if let Some(prevent) = function(&event, "preventDefault") {
            let _ = prevent.call0(&event);
        }
        spawn_refresh();
    });
    function(&document(), "addEventListener")
        .ok_or_else(|| JsValue::from_str("document.addEventListener unavailable"))?
        .call3(
            &document(),
            &JsValue::from_str("click"),
            refresh_click.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )?;
    refresh_click.forget();

    let tab_click = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        if is_present(&closest(
            &target,
            "[data-settings-tab=\"database-maintenance\"]",
        )) {
            schedule_refresh(100.0);
        }
    });
    function(&document(), "addEventListener")
        .ok_or_else(|| JsValue::from_str("document.addEventListener unavailable"))?
        .call3(
            &document(),
            &JsValue::from_str("click"),
            tab_click.as_ref().unchecked_ref(),
            &JsValue::TRUE,
        )?;
    tab_click.forget();
    schedule_refresh(300.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modified_sentinels_match_legacy_contract() {
        assert_eq!(sentinel_modified(""), Some("--".to_owned()));
        assert_eq!(sentinel_modified("missing"), Some("missing".to_owned()));
        assert_eq!(sentinel_modified("unknown"), Some("unknown".to_owned()));
        assert_eq!(sentinel_modified("123"), None);
    }
}
