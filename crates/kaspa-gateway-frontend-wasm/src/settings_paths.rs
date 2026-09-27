use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::JsFuture;

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

fn by_id(id: &str) -> JsValue {
    function(&document(), "getElementById")
        .and_then(|get| get.call1(&document(), &JsValue::from_str(id)).ok())
        .unwrap_or(JsValue::UNDEFINED)
}

fn text(value: &JsValue) -> String {
    crate::js_string_owned(value)
}

fn stale_user_path(value: &str) -> bool {
    value
        .replace('\\', "/")
        .to_ascii_lowercase()
        .contains("/appdata/roaming/kaspagateway")
}

fn first_path(paths: &JsValue, names: &[&str]) -> String {
    for name in names {
        let value = property(paths, name);
        if crate::js_boolean(&value) {
            return text(&value);
        }
    }
    String::new()
}
fn apply_paths(paths: &JsValue, force: bool) {
    for (id, names) in [
        (
            "settingsDatabasePath",
            &["database", "database_path", "data"][..],
        ),
        ("settingsExportPath", &["exports", "export_path"][..]),
        ("settingsLogPath", &["logs", "log_path"][..]),
        ("settingsBackupPath", &["backups", "backup_path"][..]),
    ] {
        let value = first_path(paths, names);
        if value.is_empty() {
            continue;
        }
        let node = by_id(id);
        if !is_present(&node) {
            continue;
        }
        let current = text(&property(&node, "value"));
        if force || current.trim().is_empty() || stale_user_path(&current) {
            set_property(&node, "value", &JsValue::from_str(&value));
        }
    }
}

fn invoke_api() -> Option<Function> {
    let tauri = property(&window(), "__TAURI__");
    let core = property(&tauri, "core");
    let direct = property(&core, "invoke");
    if let Ok(invoke) = direct.dyn_into::<Function>() {
        return Some(invoke);
    }
    let legacy = property(&property(&tauri, "tauri"), "invoke");
    if let Ok(invoke) = legacy.dyn_into::<Function>() {
        return Some(invoke);
    }
    property(&window(), "__TAURI_INVOKE__")
        .dyn_into::<Function>()
        .ok()
}
async fn invoke_defaults() -> Result<JsValue, JsValue> {
    let invoke = invoke_api().ok_or_else(|| JsValue::from_str("Tauri invoke is not available"))?;
    let value = invoke.call1(&JsValue::UNDEFINED, &JsValue::from_str("settings_defaults"))?;
    JsFuture::from(Promise::resolve(&value)).await
}

fn console(method: &str, message: &str, reason: &str, error: Option<&JsValue>) {
    let target = property(&global(), "console");
    let Some(call) = function(&target, method) else {
        return;
    };
    let details = js_sys::Object::new();
    set_property(details.as_ref(), "reason", &JsValue::from_str(reason));
    if let Some(error) = error {
        set_property(
            details.as_ref(),
            "error",
            &JsValue::from_str(&crate::js_string_owned(&property(error, "message"))),
        );
    }
    let _ = call.call2(&target, &JsValue::from_str(message), details.as_ref());
}

fn set_attr(target: &JsValue, name: &str, value: &str) {
    if let Some(call) = function(target, "setAttribute") {
        let _ = call.call2(target, &JsValue::from_str(name), &JsValue::from_str(value));
    }
}

fn append_child(parent: &JsValue, child: &JsValue) {
    if let Some(call) = function(parent, "appendChild") {
        let _ = call.call1(parent, child);
    }
}

fn error_text(error: &JsValue) -> String {
    let message = text(&property(error, "message"));
    if message.is_empty() {
        text(error)
    } else {
        message
    }
}

async fn invoke_command(command: &str, args: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let invoke = invoke_api().ok_or_else(|| JsValue::from_str("Tauri invoke is not available"))?;
    let value = match args {
        Some(args) => invoke.call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)?,
        None => invoke.call1(&JsValue::UNDEFINED, &JsValue::from_str(command))?,
    };
    JsFuture::from(Promise::resolve(&value)).await
}

fn path_status(target_id: &str, message: &str, state: &str) {
    let target = by_id(target_id);
    if !is_present(&target) {
        return;
    }
    let parent = property(&target, "parentElement");
    if !is_present(&parent) {
        return;
    }
    let selector = format!("[data-settings-path-status=\"{target_id}\"]");
    let mut status = function(&parent, "querySelector")
        .and_then(|call| call.call1(&parent, &JsValue::from_str(&selector)).ok())
        .unwrap_or(JsValue::UNDEFINED);
    if !is_present(&status) {
        status = function(&document(), "createElement")
            .and_then(|call| call.call1(&document(), &JsValue::from_str("span")).ok())
            .unwrap_or(JsValue::UNDEFINED);
        if !is_present(&status) {
            return;
        }
        set_property(
            &property(&status, "dataset"),
            "settingsPathStatus",
            &JsValue::from_str(target_id),
        );
        append_child(&parent, &status);
    }
    set_attr(
        &status,
        "role",
        if state == "error" { "alert" } else { "status" },
    );
    set_attr(
        &status,
        "aria-live",
        if state == "error" {
            "assertive"
        } else {
            "polite"
        },
    );
    set_property(
        &property(&status, "dataset"),
        "state",
        &JsValue::from_str(state),
    );
    set_property(&status, "textContent", &JsValue::from_str(message));
    let _ = crate::apply_status_tone(status, JsValue::from_str(state));
}

fn mark_dirty() {
    let save = by_id("settingsSaveSettings");
    if is_present(&save) {
        set_property(&save, "disabled", &JsValue::FALSE);
    }
}

async fn browse_path(target_id: &str) -> Result<bool, JsValue> {
    let target = by_id(target_id);
    let dialog = property(&property(&window(), "__TAURI__"), "dialog");
    let Some(open) = function(&dialog, "open") else {
        path_status(
            target_id,
            "Native directory chooser is unavailable.",
            "error",
        );
        return Ok(false);
    };
    if !is_present(&target) {
        return Ok(false);
    }
    let before = text(&property(&target, "value"));
    let options = Object::new();
    set_property(
        options.as_ref(),
        "title",
        &JsValue::from_str("Choose directory"),
    );
    set_property(options.as_ref(), "directory", &JsValue::TRUE);
    set_property(options.as_ref(), "multiple", &JsValue::FALSE);
    let selected = open.call1(&dialog, options.as_ref())?;
    let selected = JsFuture::from(Promise::resolve(&selected)).await?;
    let path_value = if Array::is_array(&selected) {
        Array::from(&selected).get(0)
    } else {
        selected
    };
    if !is_present(&path_value) || text(&path_value).is_empty() {
        path_status(target_id, "Browse cancelled; path unchanged.", "info");
        return Ok(false);
    }
    let path = text(&path_value);
    let args = Object::new();
    set_property(args.as_ref(), "key", &JsValue::from_str(target_id));
    set_property(args.as_ref(), "path", &JsValue::from_str(&path));
    set_property(args.as_ref(), "createIfMissing", &JsValue::FALSE);
    match invoke_command("settings_validate_custom_path", Some(args.as_ref())).await {
        Ok(report) => {
            let reported = property(&report, "path");
            let final_path = if crate::js_boolean(&reported) {
                text(&reported)
            } else {
                path
            };
            set_property(&target, "value", &JsValue::from_str(&final_path));
            path_status(target_id, "Directory selected.", "success");
            mark_dirty();
            Ok(true)
        }
        Err(error) => {
            set_property(&target, "value", &JsValue::from_str(&before));
            path_status(
                target_id,
                &format!("Browse failed: {}", error_text(&error)),
                "error",
            );
            Ok(false)
        }
    }
}
#[wasm_bindgen(js_name = settingsPathsApply)]
pub fn apply(paths: JsValue, force: bool) {
    apply_paths(&paths, force);
}
#[wasm_bindgen(js_name = settingsPathsLoadDefaults)]
pub async fn load_defaults(reason: String) -> JsValue {
    match invoke_defaults().await {
        Ok(defaults) => {
            let paths = property(&defaults, "paths");
            if is_present(&paths) {
                apply_paths(&paths, reason == "reset-defaults");
                console(
                    "log",
                    "dynamic settings paths loaded from backend owner",
                    &reason,
                    None,
                );
                paths
            } else {
                JsValue::NULL
            }
        }
        Err(error) => {
            console(
                "warn",
                "dynamic settings paths backend load failed",
                &reason,
                Some(&error),
            );
            JsValue::NULL
        }
    }
}

#[wasm_bindgen(js_name = settingsPathsRepairBeforeSave)]
pub async fn repair_before_save() {
    let _ = load_defaults("save-repair".to_owned()).await;
}

#[wasm_bindgen(js_name = settingsPathsBrowse)]
pub async fn browse(target_id: String) -> Result<bool, JsValue> {
    browse_path(&target_id).await
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_user_path_matches_legacy_appdata_pattern() {
        assert!(stale_user_path(
            r"C:\Users\alice\AppData\Roaming\KaspaGateway\data"
        ));
        assert!(stale_user_path("/x/AppData/Roaming/KaspaGateway/logs"));
        assert!(!stale_user_path(r"D:\KaspaGateway\data"));
    }
}
