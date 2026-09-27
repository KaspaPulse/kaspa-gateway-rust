use js_sys::{Array, Date, Function, Intl::NumberFormat, JSON, Object, Promise, Reflect};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

fn global() -> JsValue {
    js_sys::global().into()
}

fn window() -> JsValue {
    property(&global(), "window")
}

#[derive(Default)]
struct DatabaseActionState {
    selected_kind: String,
    restoring: bool,
}

thread_local! {
    static ACTION_STATE: RefCell<DatabaseActionState> =
        RefCell::new(DatabaseActionState::default());
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

fn query(selector: &str) -> JsValue {
    call1(&document(), "querySelector", &JsValue::from_str(selector))
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
        .filter_map(|index| {
            Reflect::get(&list, &JsValue::from_f64(index as f64))
                .ok()
                .filter(is_present)
        })
        .collect()
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

fn db_status(message: &str) {
    let rows = query("#settingsDatabaseRows");
    if is_present(&rows) {
        set_property(
            &property(&rows, "dataset"),
            "kgwLastStatus",
            &JsValue::from_str(message),
        );
    }
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "log") {
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW Settings DB]"),
            &JsValue::from_str(message),
        );
    }
}

async fn invoke_command(command: &str, args: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let invoke =
        invoke_api().ok_or_else(|| JsValue::from_str("Tauri invoke API is not available."))?;
    let value = match args {
        Some(args) => invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?,
        None => invoke.call1(&JsValue::UNDEFINED, &JsValue::from_str(command))?,
    };
    JsFuture::from(Promise::resolve(&value)).await
}

fn selected_kind() -> String {
    let cached = ACTION_STATE.with(|state| state.borrow().selected_kind.clone());
    if !cached.is_empty() {
        return cached;
    }
    let row = query("#settingsDatabaseRows tr.is-selected");
    if !is_present(&row) {
        return String::new();
    }
    let explicit = text(&property(&property(&row, "dataset"), "databaseKind"));
    if !explicit.is_empty() {
        return explicit;
    }
    let cells = property(&row, "cells");
    let first = Reflect::get(&cells, &JsValue::from_f64(0.0)).unwrap_or(JsValue::UNDEFINED);
    crate::settings_contract::settings_db_kind_from_file_name(property(&first, "textContent"))
}

fn stringify(value: &JsValue) -> String {
    JSON::stringify(value)
        .ok()
        .map(|value| text(value.as_ref()))
        .unwrap_or_else(|| "{}".to_owned())
}

fn trace_database(event: &JsValue, phase: &str, action_id: &str, label: &str) {
    let details = Object::new();
    set_property(
        details.as_ref(),
        "trusted",
        &JsValue::from_bool(crate::js_boolean(&property(event, "isTrusted"))),
    );
    set_property(details.as_ref(), "actionId", &JsValue::from_str(action_id));
    set_property(details.as_ref(), "text", &JsValue::from_str(label));
    let args = Object::new();
    set_property(args.as_ref(), "scope", &JsValue::from_str("settings"));
    set_property(args.as_ref(), "net", &JsValue::from_str("ui"));
    set_property(
        args.as_ref(),
        "action",
        &JsValue::from_str("settings-database"),
    );
    set_property(args.as_ref(), "phase", &JsValue::from_str(phase));
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
        let _ = Promise::resolve(&result);
    }
}

fn install_row_selection() -> Result<(), JsValue> {
    let win = window();
    if crate::js_boolean(&property(&win, "__kgwSettingsDbRowSelectionInstalled")) {
        return Ok(());
    }
    set_property(&win, "__kgwSettingsDbRowSelectionInstalled", &JsValue::TRUE);
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        let row = closest(&target, "#settingsDatabaseRows tr");
        if !is_present(&row) {
            return;
        }
        let cells = property(&row, "cells");
        let first = Reflect::get(&cells, &JsValue::from_f64(0.0)).unwrap_or(JsValue::UNDEFINED);
        let kind = crate::settings_contract::settings_db_kind_from_file_name(property(
            &first,
            "textContent",
        ));
        if kind.is_empty() {
            return;
        }
        for node in query_all("#settingsDatabaseRows tr") {
            let _ = call1(
                &property(&node, "classList"),
                "remove",
                &JsValue::from_str("is-selected"),
            );
        }
        let _ = call1(
            &property(&row, "classList"),
            "add",
            &JsValue::from_str("is-selected"),
        );
        set_property(
            &property(&row, "dataset"),
            "databaseKind",
            &JsValue::from_str(&kind),
        );
        ACTION_STATE.with(|state| state.borrow_mut().selected_kind = kind.clone());
        trace_database(
            &event,
            "r48b3-database-row-select",
            &kind,
            text(&property(&row, "textContent")).trim(),
        );
        db_status(&format!("Selected database: {kind}"));
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

fn restore_status(message: &str, state: &str) {
    let mut node = by_id("settingsRestoreStatus");
    if !is_present(&node) {
        node = create_element("p");
        set_property(&node, "id", &JsValue::from_str("settingsRestoreStatus"));
        if let Some(set) = function(&node, "setAttribute") {
            let _ = set.call2(
                &node,
                &JsValue::from_str("aria-live"),
                &JsValue::from_str("polite"),
            );
        }
        let panel = query("#settings [data-settings-panel='database-maintenance'] .database-panel");
        if is_present(&panel) {
            append(&panel, &node);
        }
    }
    let _ = function(&node, "setAttribute").and_then(|set| {
        set.call2(
            &node,
            &JsValue::from_str("role"),
            &JsValue::from_str(if state == "error" { "alert" } else { "status" }),
        )
        .ok()
    });
    set_property(
        &property(&node, "dataset"),
        "state",
        &JsValue::from_str(state),
    );
    set_property(&node, "textContent", &JsValue::from_str(message));
    let _ = crate::apply_status_tone_js(node, JsValue::from_str(state));
    db_status(message);
}

fn browser_confirm(message: &str) -> bool {
    function(&window(), "confirm")
        .and_then(|confirm| confirm.call1(&window(), &JsValue::from_str(message)).ok())
        .map(|value| crate::js_boolean(&value))
        .unwrap_or(false)
}

fn browser_prompt(message: &str) -> String {
    function(&window(), "prompt")
        .and_then(|prompt| prompt.call1(&window(), &JsValue::from_str(message)).ok())
        .filter(is_present)
        .map(|value| text(&value))
        .unwrap_or_default()
}

fn browser_alert(message: &str) {
    if let Some(alert) = function(&window(), "alert") {
        let _ = alert.call1(&window(), &JsValue::from_str(message));
    }
}

async fn invoke_action(command: &str, args: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let result = invoke_command(command, args).await?;
    let rows = property(&result, "rows");
    if Array::is_array(&rows) {
        render_rows(rows);
    } else {
        refresh_internal().await;
    }
    let message = property(&result, "message");
    if crate::js_boolean(&message) {
        db_status(&text(&message));
    }
    Ok(result)
}

async fn restore_latest() -> Result<JsValue, JsValue> {
    if ACTION_STATE.with(|state| state.borrow().restoring) {
        return Ok(JsValue::NULL);
    }
    ACTION_STATE.with(|state| state.borrow_mut().restoring = true);
    let controls = query_all(
        "#settingsDbRefresh,#settingsDbCompact,#settingsDbClearCaches,#settingsDbBackup,#settingsDbRestore,#settingsDbDelete",
    );
    let disabled = controls
        .iter()
        .map(|node| crate::js_boolean(&property(node, "disabled")))
        .collect::<Vec<_>>();
    for node in &controls {
        set_property(node, "disabled", &JsValue::TRUE);
    }

    let result = restore_latest_inner().await;
    for (node, was_disabled) in controls.iter().zip(disabled) {
        set_property(node, "disabled", &JsValue::from_bool(was_disabled));
    }
    ACTION_STATE.with(|state| state.borrow_mut().restoring = false);
    result
}

fn restore_failure(error: &JsValue, restored: bool) -> JsValue {
    let prefix = if restored {
        "Restore data completed, but UI verification failed: "
    } else {
        "Restore failed: "
    };
    let message = format!("{prefix}{}", error_text(error));
    restore_status(&message, "error");
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "error") {
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW Settings DB] restore failed"),
            error,
        );
    }
    JsValue::NULL
}

async fn restore_latest_inner() -> Result<JsValue, JsValue> {
    if !browser_confirm(
        "Restore the latest database backup? The current data will be kept in a safety backup.",
    ) {
        restore_status("Restore cancelled. No data changed.", "cancelled");
        return Ok(JsValue::NULL);
    }

    let address_epoch = crate::settings_addresses::increment_restore_epoch();
    restore_status("Restoring the selected backup...", "running");
    let result = match invoke_command("kgw_settings_database_restore_latest", None).await {
        Ok(value) => value,
        Err(error) => return Ok(restore_failure(&error, false)),
    };
    let rows = property(&result, "rows");
    let ok = crate::js_boolean(&property(&result, "ok"));
    let backup_path = property(&result, "backup_path");
    if !ok || !crate::js_boolean(&backup_path) || !Array::is_array(&rows) {
        let message = property(&result, "message");
        let error = if crate::js_boolean(&message) {
            message
        } else {
            JsValue::from_str("Restore did not return a verified result.")
        };
        return Ok(restore_failure(&error, false));
    }

    let records = match invoke_command("get_all_addresses", None).await {
        Ok(value) => value,
        Err(error) => return Ok(restore_failure(&error, true)),
    };
    if !Array::is_array(&records) {
        return Ok(restore_failure(
            &JsValue::from_str("Restored address state could not be read."),
            true,
        ));
    }
    render_rows(rows);
    let options = Object::new();
    set_property(options.as_ref(), "localOnly", &JsValue::TRUE);
    set_property(
        options.as_ref(),
        "restoreEpoch",
        &JsValue::from_f64(address_epoch as f64),
    );
    let rendered = crate::settings_addresses::render_rows(records, options.into()).await;
    if !rendered {
        return Ok(restore_failure(
            &JsValue::from_str("Restored address rows were not rendered."),
            true,
        ));
    }
    crate::settings_addresses::clear_fields();
    crate::settings_addresses::set_status_export(
        format!("Last Updated: {}", crate::settings_addresses::now()),
        "success".to_owned(),
    );
    let message = text(&property(&result, "message"));
    restore_status(&message, "success");
    Ok(result)
}

async fn run_action(id: &str) -> Result<(), JsValue> {
    match id {
        "settingsDbRefresh" => {
            refresh_internal().await;
        }
        "settingsDbCompact" => {
            let _ = invoke_action("kgw_settings_database_compact", None).await?;
        }
        "settingsDbClearCaches" => {
            if browser_confirm("Clear application cache rows?") {
                let _ = invoke_action("kgw_settings_database_clear_caches", None).await?;
            }
        }
        "settingsDbBackup" => {
            let _ = invoke_action("kgw_settings_database_backup", None).await?;
        }
        "settingsDbRestore" => {
            let _ = restore_latest().await?;
        }
        "settingsDbDelete" => {
            let database = selected_kind();
            if database.is_empty() {
                browser_alert("Select a database row first.");
                return Ok(());
            }
            let typed = browser_prompt(&format!(
                "Type DELETE to delete and reinitialize: {database}"
            ));
            if typed != "DELETE" {
                return Ok(());
            }
            let args = Object::new();
            set_property(args.as_ref(), "database", &JsValue::from_str(&database));
            let _ = invoke_action("kgw_settings_database_delete", Some(args.as_ref())).await?;
        }
        _ => {}
    }
    Ok(())
}

fn report_action_error(error: JsValue) {
    db_status(&format!("Database action failed: {}", error_text(&error)));
    let console = property(&global(), "console");
    if let Some(log) = function(&console, "error") {
        let _ = log.call2(
            &console,
            &JsValue::from_str("[KGW Settings DB] action failed"),
            &error,
        );
    }
}

#[wasm_bindgen(js_name = settingsDatabaseInstallMaintenance)]
pub fn install_maintenance() -> Result<(), JsValue> {
    let win = window();
    if crate::js_boolean(&property(
        &win,
        "__kgwSettingsDbMaintenanceActionsInstalled",
    )) {
        return Ok(());
    }
    set_property(
        &win,
        "__kgwSettingsDbMaintenanceActionsInstalled",
        &JsValue::TRUE,
    );
    install_row_selection()?;

    let callback = Closure::<dyn FnMut(JsValue)>::new(move |event| {
        let target = property(&event, "target");
        let button = closest(
            &target,
            "#settingsDbRefresh,#settingsDbCompact,#settingsDbClearCaches,#settingsDbBackup,#settingsDbRestore,#settingsDbDelete",
        );
        if !is_present(&button) {
            return;
        }
        let _ = call0(&event, "preventDefault");
        let _ = call0(&event, "stopImmediatePropagation");
        if ACTION_STATE.with(|state| state.borrow().restoring) {
            return;
        }
        let id = text(&property(&button, "id"));
        let label = text(&property(&button, "textContent")).trim().to_owned();
        trace_database(&event, "r48b3-database-action-click", &id, &label);
        spawn_local(async move {
            if let Err(error) = run_action(&id).await {
                report_action_error(error);
            }
        });
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
