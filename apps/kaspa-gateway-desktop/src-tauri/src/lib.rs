/* KGW_RUNTIME_MAIN_WINDOW_ICON_START */
fn kgw_set_runtime_main_window_icon(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;

    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    let icon = tauri::image::Image::new(include_bytes!("../icons/icon.rgba"), 128, 128);
    window.set_icon(icon)?;

    Ok(())
}
/* KGW_RUNTIME_MAIN_WINDOW_ICON_END */

mod address_book;
mod address_commands;
mod analysis_commands;
mod app_logger;
pub mod commands;
mod db_state;
mod db_status_commands;
mod diagnostics;
mod explorer_services;
mod export_commands;
mod export_system;
mod i18n_commands;
mod integrated_runtime_commands;
mod live_metrics;
mod transaction_commands;
mod transaction_routes;

mod config_commands;
mod data_enforcement_commands;
mod migration;
mod network_full;
mod persistent_logs;
mod price_service;
mod python_migration_real;
mod real_reports;
mod release_qa;
mod runtime_commands;
mod security_audit;
mod security_hardening;
mod settings;
mod settings_commands;
mod top_addresses_commands;
mod top_addresses_deep;
mod transaction_analysis;
mod ui_wiring;

/* KGW_UI_TRACE_GATE_DEFAULT_OFF_R20
 * Central UI trace gate.
 * Default: no KGW_BUTTON_TRACE output.
 * Enable for dev diagnostics with:
 *   PowerShell: $env:KGW_UI_TRACE="1"
 * Supported truthy values: 1, true, yes, on, debug.
 */
fn kgw_ui_trace_level_v1() -> Option<&'static str> {
    match std::env::var("KGW_UI_TRACE") {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" | "user" => Some("user"),
            "full" => Some("full"),
            "debug" => Some("debug"),
            _ => None,
        },
        Err(_) => None,
    }
}

#[allow(dead_code)]
fn kgw_ui_trace_date_yyyymmdd_v37() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "unknown-date".to_string();
    };

    let days = (duration.as_secs() / 86_400) as i64;
    let (year, month, day) = kgw_ui_trace_civil_from_days_v37(days);
    format!("{year:04}{month:02}{day:02}")
}

#[allow(dead_code)]
fn kgw_ui_trace_civil_from_days_v37(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    (year as i32, m as u32, d as u32)
}

fn kgw_ui_trace_json_escape_v37(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

fn kgw_ui_trace_file_sink_enabled_v40() -> bool {
    match std::env::var("KGW_UI_TRACE_FILE") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on" | "file"
        ),
        Err(_) => false,
    }
}

fn kgw_ui_trace_log_dir_v37() -> Option<std::path::PathBuf> {
    if let Some(dir) = std::env::var_os("KGW_UI_TRACE_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }

    if !kgw_ui_trace_file_sink_enabled_v40() {
        return None;
    }

    let current_dir = std::env::current_dir().ok()?;
    Some(current_dir.join("dev-traces"))
}

// KGW_TRACE_UNIQUE_SESSION_ZIP_R69F2
use std::io::Write;
// Backend trace file session owner:
// - one unique .log per process/run when KGW_UI_TRACE_FILE=1 and KGW_UI_TRACE_DIR is set
// - console output remains controlled by KGW_UI_TRACE
// - file output remains controlled by KGW_UI_TRACE_FILE/KGW_UI_TRACE_DIR
// - writes session-start once
// - writes session-end on app close finalization
// - zips .log and removes original .log only after zip succeeds
struct KgwTraceSessionR69F2 {
    log_path: std::path::PathBuf,
    zip_path: std::path::PathBuf,
    file: std::sync::Mutex<Option<std::fs::File>>,
    started_unix_ms: u128,
    finalized: std::sync::Mutex<bool>,
}

impl KgwTraceSessionR69F2 {
    fn new(dir: std::path::PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&dir)
            .map_err(|error| format!("create trace dir failed: {error}"))?;

        let now_ms = kgw_trace_unix_ms_r69f2();
        let pid = std::process::id();
        let base_name = format!(
            "kgw-ui-trace-{}-pid{}",
            kgw_trace_timestamp_for_file_r69f2(now_ms),
            pid
        );
        let log_path = dir.join(format!("{base_name}.log"));
        let zip_path = dir.join(format!("{base_name}.zip"));

        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&log_path)
            .map_err(|error| format!("open trace session file failed: {error}"))?;

        writeln!(
            file,
            "{{\"event\":\"session-start\",\"owner\":\"KGW_TRACE_BACKEND_GATE_OWNER\",\"pid\":{},\"unix_ms\":{},\"log_path\":\"{}\"}}",
            pid,
            now_ms,
            kgw_trace_json_escape_r69f2(&log_path.to_string_lossy())
        )
        .map_err(|error| format!("write trace session-start failed: {error}"))?;

        let _ = file.flush();

        Ok(Self {
            log_path,
            zip_path,
            file: std::sync::Mutex::new(Some(file)),
            started_unix_ms: now_ms,
            finalized: std::sync::Mutex::new(false),
        })
    }

    fn write_line(&self, line: &str) {
        if let Ok(mut file_guard) = self.file.lock() {
            if let Some(file) = file_guard.as_mut() {
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
        }
    }

    fn finalize(&self) {
        let mut finalized = match self.finalized.lock() {
            Ok(value) => value,
            Err(_) => return,
        };

        if *finalized {
            return;
        }

        *finalized = true;

        let ended_ms = kgw_trace_unix_ms_r69f2();
        let elapsed_ms = ended_ms.saturating_sub(self.started_unix_ms);

        if let Ok(mut file_guard) = self.file.lock() {
            if let Some(mut file) = file_guard.take() {
                let _ = writeln!(
                    file,
                    "{{\"event\":\"session-end\",\"owner\":\"KGW_TRACE_BACKEND_GATE_OWNER\",\"pid\":{},\"unix_ms\":{},\"elapsed_ms\":{}}}",
                    std::process::id(),
                    ended_ms,
                    elapsed_ms
                );
                let _ = file.flush();
                let _ = file.sync_all();
            }
        }

        match kgw_zip_trace_log_r69f2(&self.log_path, &self.zip_path) {
            Ok(()) => {
                let _ = std::fs::remove_file(&self.log_path);
                eprintln!(
                    "[KGW_TRACE_SESSION][R69F2] zipped={} removed_log={}",
                    self.zip_path.display(),
                    self.log_path.display()
                );
            }
            Err(error) => {
                eprintln!(
                    "[KGW_TRACE_SESSION][R69F2] zip_failed log={} zip={} error={}",
                    self.log_path.display(),
                    self.zip_path.display(),
                    error
                );
            }
        }
    }
}

static KGW_TRACE_SESSION_R69F2: std::sync::OnceLock<Option<std::sync::Arc<KgwTraceSessionR69F2>>> =
    std::sync::OnceLock::new();

fn kgw_trace_unix_ms_r69f2() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn kgw_trace_timestamp_for_file_r69f2(unix_ms: u128) -> String {
    format!("{unix_ms}")
}

fn kgw_trace_json_escape_r69f2(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn kgw_trace_session_r69f2() -> Option<std::sync::Arc<KgwTraceSessionR69F2>> {
    KGW_TRACE_SESSION_R69F2
        .get_or_init(|| {
            if !kgw_ui_trace_file_sink_enabled_v40() {
                return None;
            }

            let Some(dir) = kgw_ui_trace_log_dir_v37() else {
                return None;
            };

            match KgwTraceSessionR69F2::new(dir) {
                Ok(session) => Some(std::sync::Arc::new(session)),
                Err(error) => {
                    eprintln!("[KGW_TRACE_SESSION][R69F2] init_failed error={error}");
                    None
                }
            }
        })
        .as_ref()
        .cloned()
}

fn kgw_trace_write_file_line_r69f2(line: &str) {
    if let Some(session) = kgw_trace_session_r69f2() {
        session.write_line(line);
    }
}

fn kgw_trace_finalize_session_r69f2() {
    if let Some(Some(session)) = KGW_TRACE_SESSION_R69F2.get() {
        session.finalize();
    }
}

fn kgw_zip_trace_log_r69f2(
    log_path: &std::path::Path,
    zip_path: &std::path::Path,
) -> Result<(), String> {
    let zip_file =
        std::fs::File::create(zip_path).map_err(|error| format!("create zip failed: {error}"))?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let entry_name = log_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("kgw-ui-trace.log");

    zip.start_file(entry_name, options)
        .map_err(|error| format!("start zip entry failed: {error}"))?;

    let mut log_file =
        std::fs::File::open(log_path).map_err(|error| format!("open log failed: {error}"))?;
    let mut buffer = Vec::new();

    use std::io::Read;
    log_file
        .read_to_end(&mut buffer)
        .map_err(|error| format!("read log failed: {error}"))?;

    zip.write_all(&buffer)
        .map_err(|error| format!("write zip entry failed: {error}"))?;

    zip.finish()
        .map_err(|error| format!("finish zip failed: {error}"))?;

    Ok(())
}

fn kgw_ui_trace_append_file_v37(
    level: &str,
    scope: &str,
    net: &str,
    action: &str,
    phase: &str,
    details: &str,
) {
    let now = format!("{:?}", std::time::SystemTime::now());
    let line = format!(
        "{{\"ts\":\"{}\",\"level\":\"{}\",\"scope\":\"{}\",\"net\":\"{}\",\"action\":\"{}\",\"phase\":\"{}\",\"details\":\"{}\"}}",
        kgw_ui_trace_json_escape_v37(&now),
        kgw_ui_trace_json_escape_v37(level),
        kgw_ui_trace_json_escape_v37(scope),
        kgw_ui_trace_json_escape_v37(net),
        kgw_ui_trace_json_escape_v37(action),
        kgw_ui_trace_json_escape_v37(phase),
        kgw_ui_trace_json_escape_v37(details)
    );

    kgw_trace_write_file_line_r69f2(&line);
}

fn kgw_ui_trace_should_print_v1(level: &str, action: &str, phase: &str, details: &str) -> bool {
    let text = format!("{action} {phase} {details}").to_ascii_lowercase();

    let is_programmatic = text.contains("programmatic")
        || text.contains("input-programmatic")
        || text.contains("change-programmatic")
        || text.contains("programmatic-input")
        || text.contains("programmatic-change");

    let is_bootstrap = text.contains("owner-installed")
        || text.contains("bootstrap")
        || text.contains("probe")
        || text.contains("initial")
        || text.contains("invoke-proxy-installed");

    let is_owner_disabled = phase.contains("disabled")
        || text.contains("v19-disabled")
        || text.contains("\"reason\":\"initial\"");

    match level {
        "debug" => true,
        "full" => !is_bootstrap,
        "user" => !is_programmatic && !is_bootstrap && !is_owner_disabled,
        _ => false,
    }
}

#[tauri::command]
fn kgw_start_trace_frontend_v1(
    stage: String,
    network: String,
    action: String,
    result: String,
    details: Option<String>,
) -> bool {
    integrated_runtime_commands::kgw_start_trace_emit_v1(
        "frontend",
        &stage,
        &network,
        &action,
        &result,
        details.as_deref(),
    );

    integrated_runtime_commands::kgw_start_trace_enabled_v1()
}

#[tauri::command]
fn kgw_frontend_button_trace_v1(
    scope: String,
    net: String,
    action: String,
    phase: String,
    details: String,
) -> bool {
    let Some(trace_level) = kgw_ui_trace_level_v1() else {
        return false;
    };

    if !kgw_ui_trace_should_print_v1(trace_level, &action, &phase, &details) {
        return false;
    }

    println!(
        "[KGW_BUTTON_TRACE] level={} scope={} net={} action={} phase={} details={}",
        trace_level, scope, net, action, phase, details
    );

    kgw_ui_trace_append_file_v37(trace_level, &scope, &net, &action, &phase, &details);

    true
}

fn kgw_clipboard_normalize_network_v1(network: &str) -> Result<String, String> {
    let clean = network.trim().to_ascii_lowercase();
    match clean.as_str() {
        "mainnet" | "testnet10" | "testnet12" => Ok(clean),
        _ => Err(format!(
            "clipboard_write_failed=true;reason=unsupported-network;network={};message=Copy Log received an unsupported network.",
            kgw_clipboard_safe_field_v1(network, "unknown")
        )),
    }
}

fn kgw_clipboard_normalize_runtime_role_v1(runtime_role: Option<&str>) -> String {
    let clean = runtime_role
        .unwrap_or("unknown")
        .trim()
        .to_ascii_lowercase();

    match clean.as_str() {
        "node" | "bridge" => clean,
        _ => "unknown".to_string(),
    }
}

fn kgw_clipboard_normalize_bridge_instance_id_v1(
    bridge_instance_id: Option<&str>,
) -> Option<String> {
    let clean = bridge_instance_id?
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '\0' && *ch != '\r' && *ch != '\n')
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        None
    } else {
        Some(clean.chars().take(128).collect())
    }
}

fn kgw_clipboard_safe_field_v1(value: &str, fallback: &str) -> String {
    let clean = value
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean.chars().take(160).collect()
    }
}

fn kgw_clipboard_safe_error_v1(error: &str) -> String {
    let clean = kgw_clipboard_safe_field_v1(error, "clipboard write failed");
    let lowered = clean.to_ascii_lowercase();
    if lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("private")
        || lowered.contains("mnemonic")
        || lowered.contains("wallet")
        || lowered.contains("address")
    {
        "clipboard write failed with a sensitive native error".to_string()
    } else {
        clean
    }
}

fn kgw_clipboard_character_count_v1(text: &str) -> u64 {
    text.chars().count() as u64
}

fn kgw_clipboard_line_count_v1(text: &str) -> u64 {
    if text.is_empty() {
        0
    } else {
        text.split('\n').count() as u64
    }
}

fn kgw_clipboard_sha256_v1(text: &str) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(text.as_bytes());
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn kgw_clipboard_trace_details_v1(
    character_count: u64,
    line_count: u64,
    sha256: &str,
    implementation: &str,
    extra: serde_json::Value,
) -> String {
    serde_json::json!({
        "characterCount": character_count,
        "lineCount": line_count,
        "sha256": sha256,
        "implementation": implementation,
        "extra": extra,
    })
    .to_string()
}

fn kgw_copy_text_to_clipboard_inner_v1<F>(
    network: String,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    text: String,
    metadata_character_count: u64,
    metadata_line_count: u64,
    metadata_sha256: Option<String>,
    writer: F,
) -> Result<String, String>
where
    F: FnOnce(String) -> Result<(), String>,
{
    let network = kgw_clipboard_normalize_network_v1(&network)?;
    let runtime_role = kgw_clipboard_normalize_runtime_role_v1(runtime_role.as_deref());
    let bridge_instance_id =
        kgw_clipboard_normalize_bridge_instance_id_v1(bridge_instance_id.as_deref());
    let character_count = kgw_clipboard_character_count_v1(&text);
    let line_count = kgw_clipboard_line_count_v1(&text);
    let sha256 = kgw_clipboard_sha256_v1(&text);
    let implementation = "tauri-plugin-clipboard-manager";
    let supplied_sha256 = metadata_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("");

    integrated_runtime_commands::kgw_start_trace_emit_v1(
        "native",
        "native.clipboard_write_entered",
        &network,
        "copy-log",
        "entered",
        Some(&kgw_clipboard_trace_details_v1(
            character_count,
            line_count,
            &sha256,
            implementation,
            serde_json::json!({
                "runtimeRole": runtime_role.clone(),
                "bridgeInstanceId": bridge_instance_id.clone(),
                "metadataCharacterCount": metadata_character_count,
                "metadataLineCount": metadata_line_count,
                "metadataSha256Present": !supplied_sha256.is_empty(),
            }),
        )),
    );

    let fail = |reason: &str, message: String| -> String {
        integrated_runtime_commands::kgw_start_trace_emit_v1(
            "native",
            "native.clipboard_write_failed",
            &network,
            "copy-log",
            "error",
            Some(&kgw_clipboard_trace_details_v1(
                character_count,
                line_count,
                &sha256,
                implementation,
                serde_json::json!({
                    "runtimeRole": runtime_role.clone(),
                    "bridgeInstanceId": bridge_instance_id.clone(),
                    "reason": reason,
                    "safeError": kgw_clipboard_safe_error_v1(&message),
                }),
            )),
        );
        message
    };

    if text.trim().is_empty() {
        return Err(fail(
            "empty-log-buffer",
            format!(
                "clipboard_write_failed=true;network={network};reason=empty-log-buffer;message=Copy Log requires a non-empty raw log buffer."
            ),
        ));
    }

    if character_count != metadata_character_count || line_count != metadata_line_count {
        return Err(fail(
            "metadata-mismatch",
            format!(
                "clipboard_write_failed=true;network={network};reason=metadata-mismatch;actual_characters={character_count};actual_lines={line_count};metadata_characters={metadata_character_count};metadata_lines={metadata_line_count};message=Copy Log metadata did not match the supplied text."
            ),
        ));
    }

    if !supplied_sha256.is_empty() && supplied_sha256 != sha256 {
        return Err(fail(
            "sha256-mismatch",
            format!(
                "clipboard_write_failed=true;network={network};reason=sha256-mismatch;message=Copy Log SHA-256 metadata did not match the supplied text."
            ),
        ));
    }

    match writer(text) {
        Ok(()) => {
            integrated_runtime_commands::kgw_start_trace_emit_v1(
                "native",
                "native.clipboard_write_succeeded",
                &network,
                "copy-log",
                "ok",
                Some(&kgw_clipboard_trace_details_v1(
                    character_count,
                    line_count,
                    &sha256,
                    implementation,
                    serde_json::json!({
                        "runtimeRole": runtime_role.clone(),
                        "bridgeInstanceId": bridge_instance_id.clone(),
                        "confirmed": true,
                    }),
                )),
            );

            Ok(format!(
                "clipboard_write_v1;network={network};characters={character_count};lines={line_count};sha256={sha256};implementation={implementation};copied=true"
            ))
        }
        Err(error) => Err(fail(
            "native-clipboard-error",
            format!(
                "clipboard_write_failed=true;network={network};reason=native-clipboard-error;error={}",
                error
            ),
        )),
    }
}

#[tauri::command]
fn kgw_copy_text_to_clipboard_v1(
    app: tauri::AppHandle,
    network: String,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    text: String,
    character_count: u64,
    line_count: u64,
    sha256: Option<String>,
) -> Result<String, String> {
    kgw_copy_text_to_clipboard_inner_v1(
        network,
        runtime_role,
        bridge_instance_id,
        text,
        character_count,
        line_count,
        sha256,
        |value| {
            use tauri_plugin_clipboard_manager::ClipboardExt;
            app.clipboard()
                .write_text(value)
                .map_err(|error| error.to_string())
        },
    )
}

#[cfg(test)]
mod kgw_clipboard_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn char_count(text: &str) -> u64 {
        text.chars().count() as u64
    }

    #[test]
    fn clipboard_success_returns_success_after_writer_accepts_text() {
        let copied = Arc::new(Mutex::new(None::<String>));
        let copied_for_writer = Arc::clone(&copied);
        let text = "mainnet raw line 1\r\nmainnet raw line 2".to_string();
        let sha256 = kgw_clipboard_sha256_v1(&text);

        let result = kgw_copy_text_to_clipboard_inner_v1(
            "mainnet".to_string(),
            Some("node".to_string()),
            None,
            text.clone(),
            char_count(&text),
            2,
            Some(sha256.clone()),
            move |value| {
                *copied_for_writer.lock().unwrap() = Some(value);
                Ok(())
            },
        )
        .expect("clipboard success should be returned after writer success");

        assert!(result.contains("clipboard_write_v1"));
        assert!(result.contains("network=mainnet"));
        assert!(result.contains("copied=true"));
        assert!(result.contains(&format!("sha256={sha256}")));
        assert_eq!(copied.lock().unwrap().as_deref(), Some(text.as_str()));
    }

    #[test]
    fn clipboard_failure_propagates_original_native_error() {
        let text = "testnet10 raw line".to_string();
        let error = kgw_copy_text_to_clipboard_inner_v1(
            "testnet10".to_string(),
            Some("node".to_string()),
            None,
            text.clone(),
            char_count(&text),
            1,
            Some(kgw_clipboard_sha256_v1(&text)),
            |_value| Err("native clipboard permission denied".to_string()),
        )
        .expect_err("native clipboard failure must remain an error");

        assert!(error.contains("clipboard_write_failed=true"));
        assert!(error.contains("network=testnet10"));
        assert!(error.contains("native clipboard permission denied"));
    }

    #[test]
    fn empty_clipboard_content_is_rejected_before_writer() {
        let error = kgw_copy_text_to_clipboard_inner_v1(
            "mainnet".to_string(),
            Some("node".to_string()),
            None,
            "   \r\n  ".to_string(),
            7,
            2,
            None,
            |_value| panic!("empty content must not reach the clipboard writer"),
        )
        .expect_err("empty clipboard text must be rejected");

        assert!(error.contains("empty-log-buffer"));
        assert!(error.contains("network=mainnet"));
    }

    #[test]
    fn clipboard_trace_excludes_raw_content_and_records_hash() {
        std::env::set_var("KGW_START_TRACE", "1");
        let _ = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1();

        let text = "mainnet secret raw content should not appear".to_string();
        let sha256 = kgw_clipboard_sha256_v1(&text);
        let result = kgw_copy_text_to_clipboard_inner_v1(
            "mainnet".to_string(),
            Some("bridge".to_string()),
            Some("bridge-a".to_string()),
            text.clone(),
            char_count(&text),
            1,
            Some(sha256.clone()),
            |_value| Ok(()),
        )
        .expect("clipboard trace success should succeed");

        assert!(result.contains("copied=true"));

        let trace = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1().join("\n");
        assert!(trace.contains("native.clipboard_write_entered"));
        assert!(trace.contains("native.clipboard_write_succeeded"));
        assert!(trace.contains(&sha256));
        assert!(trace.contains("\\\"runtimeRole\\\":\\\"bridge\\\""));
        assert!(trace.contains("\\\"bridgeInstanceId\\\":\\\"bridge-a\\\""));
        assert!(!trace.contains("secret raw content"));
        assert!(!trace.contains(&text));

        std::env::remove_var("KGW_START_TRACE");
    }
}

#[tauri::command]
fn kgw_open_exported_file_v1(path: String) -> Result<(), String> {
    use std::path::PathBuf;
    use std::process::Command;

    let requested = PathBuf::from(&path);
    if !requested.is_absolute() {
        return Err("export path must be absolute".to_string());
    }

    if !requested.exists() {
        return Err(format!(
            "export file does not exist: {}",
            requested.display()
        ));
    }

    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| "LOCALAPPDATA is not available".to_string())?;
    let exports_root = PathBuf::from(local_app_data)
        .join("KaspaGateway")
        .join("exports");

    let exports_root_canonical = std::fs::canonicalize(&exports_root).map_err(|e| {
        format!(
            "failed to canonicalize exports root {}: {e}",
            exports_root.display()
        )
    })?;

    let requested_parent = requested
        .parent()
        .ok_or_else(|| "export file has no parent directory".to_string())?;

    let requested_parent_canonical = std::fs::canonicalize(requested_parent).map_err(|e| {
        format!(
            "failed to canonicalize export parent {}: {e}",
            requested_parent.display()
        )
    })?;

    if !requested_parent_canonical.starts_with(&exports_root_canonical) {
        return Err(format!(
            "refusing to open file outside exports directory: {}",
            requested.display()
        ));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("failed to open exported file: {e}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open exported file: {e}"))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("failed to open exported file: {e}"))?;
    }

    Ok(())
}

pub fn run() {
    app_logger::init_tracing_bridge();
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init());

    #[cfg(feature = "e2e-test")]
    let builder = builder
        .plugin(tauri_plugin_wdio_webdriver::init())
        .plugin(tauri_plugin_wdio::init());

    #[cfg(feature = "e2e-test")]
    let context = tauri::generate_context!("tauri.e2e.conf.json");
    #[cfg(not(feature = "e2e-test"))]
    let context = tauri::generate_context!();

    builder
        .manage(commands::DesktopRuntimeState::default())
        .manage(diagnostics::LogState::default())
        .invoke_handler(tauri::generate_handler![
            integrated_runtime_commands::kgw_runtime_owner_summary_v1,
            integrated_runtime_commands::kgw_runtime_owner_status_v1,
            integrated_runtime_commands::kgw_runtime_owner_plan_v1,
            integrated_runtime_commands::kgw_all_parallel_node_bridge_plans_v1,
            integrated_runtime_commands::kgw_kgw_runtime_logs_v1,
            integrated_runtime_commands::kgw_kgw_runtime_clear_logs_v1,
            integrated_runtime_commands::kgw_kgw_real_owner_summary_v1,
            integrated_runtime_commands::kgw_kgw_real_owner_feature_status_v1,
            integrated_runtime_commands::kgw_kgw_apply_node_settings_v1,
            integrated_runtime_commands::kgw_kgw_disable_network_v1,
            integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1,
            integrated_runtime_commands::kgw_kgw_smoke_start_network_v1,
            integrated_runtime_commands::kgw_kgw_smoke_stop_network_v1,
            integrated_runtime_commands::kgw_kgw_node_bridge_service_plan_v1,
            commands::open_external_url,
            transaction_commands::explorer_list_transactions_grouped_rust,
            transaction_commands::explorer_transaction_day_summaries_rust,
            db_status_commands::kgw_settings_database_status,
            db_status_commands::kgw_settings_database_compact,
            db_status_commands::kgw_settings_database_clear_caches,
            db_status_commands::kgw_settings_database_backup,
            db_status_commands::kgw_settings_database_restore_latest,
            db_status_commands::kgw_settings_database_delete,
            price_service::kgw_get_kaspa_prices,
            top_addresses_commands::fetch_top_addresses_rust,
            address_commands::delete_saved_address,
            address_commands::save_address,
            address_commands::get_all_addresses,
            app_logger::kgw_log_file_path,
            app_logger::kgw_log_read,
            app_logger::kgw_log_clear,
            app_logger::kgw_log_append,
            transaction_routes::explorer_transactions,
            transaction_routes::explorer_cancel_transactions,
            transaction_routes::explorer_delete_transactions_for_address,
            live_metrics::kgw_live_metrics_snapshot,
            live_metrics::kgw_live_metrics_refresh_now,
            explorer_services::explorer_export,
            explorer_services::explorer_balance,
            explorer_services::explorer_saved_addresses,
            data_enforcement_commands::api_probe,
            data_enforcement_commands::data_enforcement_report,
            security_audit::security_audit_report,
            security_audit::security_recovery_manifest,
            security_audit::security_secret_scan,
            export_commands::export_report,
            export_commands::export_preview,
            export_commands::export_default_path,
            i18n_commands::i18n_import_translations,
            i18n_commands::i18n_report,
            i18n_commands::i18n_get_active_language,
            i18n_commands::i18n_save_language,
            i18n_commands::i18n_load_catalog,
            i18n_commands::i18n_languages,
            settings_commands::settings_api_endpoint_editor,
            settings_commands::settings_secure_get_masked,
            settings_commands::settings_secure_store,
            settings_commands::settings_validate_custom_path,
            settings_commands::settings_import_config,
            settings_commands::settings_reset,
            settings_commands::settings_save,
            settings_commands::settings_load,
            settings_commands::settings_defaults,
            persistent_logs::persistent_log_stats,
            persistent_logs::persistent_log_clear,
            persistent_logs::persistent_log_tail,
            persistent_logs::persistent_log_list,
            persistent_logs::persistent_log_append,
            top_addresses_commands::top_addresses_load_currency_rates,
            top_addresses_commands::top_addresses_save_currency_rates,
            top_addresses_commands::top_addresses_load_known_names,
            top_addresses_commands::top_addresses_save_known_names,
            top_addresses_commands::top_addresses_search,
            top_addresses_commands::top_addresses_fetch_api,
            analysis_commands::analysis_graph_report,
            analysis_commands::analysis_report,
            analysis_commands::analysis_time_range_options,
            address_book::address_book_import_json,
            address_book::address_book_import_csv,
            address_book::address_book_export_json,
            address_book::address_book_export_csv,
            address_book::address_book_stats,
            release_qa::final_release_qa_report,
            security_hardening::security_hardening_report,
            security_hardening::create_recovery_snapshot,
            security_hardening::security_validate_path,
            security_hardening::security_validate_process_args,
            security_hardening::security_redact_text,
            ui_wiring::feature_wiring_report,
            python_migration_real::real_python_migration_run,
            python_migration_real::real_python_migration_preview,
            real_reports::export_real_report,
            real_reports::real_report_preview,
            real_reports::default_real_report_path,
            network_full::full_network_analytics_report,
            transaction_analysis::address_direction_breakdown,
            transaction_analysis::local_rich_list,
            transaction_analysis::analyze_all_transactions,
            transaction_analysis::analyze_address_flow,
            runtime_commands::tail_process_log,
            runtime_commands::real_bridge_runtime_report,
            runtime_commands::real_node_runtime_report,
            runtime_commands::real_node_default_runtime_settings,
            runtime_commands::real_bridge_default_runtime_settings,
            runtime_commands::real_node_runtime_command_preview,
            runtime_commands::real_bridge_runtime_command_preview,
            runtime_commands::real_node_runtime_apply_settings,
            runtime_commands::real_bridge_runtime_apply_settings,
            config_commands::config_import_python_config,
            config_commands::config_save,
            config_commands::config_load,
            config_commands::config_default,
            commands::desktop_ping,
            commands::app_info,
            commands::runtime_check,
            commands::api_network_url,
            commands::settings_fetch_address_names,
            commands::node_capabilities,
            commands::dashboard_report,
            commands::ui_sections,
            commands::list_addresses,
            commands::add_address,
            commands::delete_address,
            commands::rename_address,
            commands::explorer_fetch_balance,
            commands::network_analytics_report,
            commands::load_node_settings,
            commands::save_node_settings,
            commands::node_command_preview,
            commands::node_start,
            commands::node_stop,
            commands::node_status,
            commands::load_bridge_settings,
            commands::save_bridge_settings,
            commands::bridge_command_preview,
            commands::bridge_start,
            commands::bridge_stop,
            commands::bridge_status,
            diagnostics::append_log,
            diagnostics::list_logs,
            diagnostics::clear_logs,
            diagnostics::diagnostics_report,
            settings::load_full_settings,
            settings::save_full_settings,
            settings::reset_full_settings,
            export_system::default_export_path,
            export_system::export_data,
            migration::preview_python_migration,
            migration::migrate_python_data,
            kgw_start_trace_frontend_v1,
            kgw_frontend_button_trace_v1,
            kgw_copy_text_to_clipboard_v1,
            kgw_open_exported_file_v1,
        ])
        .setup(|app| {
            kgw_set_runtime_main_window_icon(app)?;
            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let _ = crate::integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();

                kgw_trace_finalize_session_r69f2();
            }
        })
        .run(context)
        .expect("error while running Kaspa Gateway desktop app");
}

fn kgw_init_bridge_self_worker_raw_tracing_r23() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                "info,kaspa_stratum_bridge=debug,kaspa_gateway_rk_bridge=debug",
            )
        });

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .with_target(true)
            .with_level(true)
            .with_writer(std::io::stderr)
            .try_init();
    });
}

pub fn try_run_kgw_self_worker_from_args() -> bool {
    let args: Vec<String> = std::env::args().collect();

    if !args.iter().any(|arg| arg == "--kgw-self-worker") {
        return false;
    }

    let role =
        kgw_self_worker_arg_value(&args, "--kgw-self-worker").unwrap_or_else(|| "node".to_string());
    let role_key = role.trim().to_ascii_lowercase();
    let network =
        kgw_self_worker_arg_value(&args, "--network").unwrap_or_else(|| "mainnet".to_string());
    let appdir = kgw_self_worker_arg_value(&args, "--appdir")
        .unwrap_or_else(|| kgw_self_worker_default_appdir(&network));
    let rpc = kgw_self_worker_arg_value(&args, "--rpc")
        .unwrap_or_else(|| kgw_self_worker_default_rpc(&network).to_string());
    let stratum = kgw_self_worker_arg_value(&args, "--stratum")
        .unwrap_or_else(|| kgw_self_worker_default_stratum(&network).to_string());
    let utxoindex = args.iter().any(|arg| arg == "--utxoindex");
    let archival = args.iter().any(|arg| arg == "--archival");

    let bridge_node_mode = if role_key == "bridge" {
        kgw_self_worker_arg_value(&args, "--node-mode")
            .unwrap_or_else(|| "external".to_string())
            .trim()
            .to_ascii_lowercase()
    } else {
        String::new()
    };

    let bridge_owns_inprocess_node = matches!(
        bridge_node_mode.as_str(),
        "inprocess" | "inproc" | "official-inprocess-node" | "inprocess-node"
    );

    let result = match role_key.as_str() {
        "node" => kgw_run_node_self_worker(&network, &appdir, &rpc, utxoindex, archival),
        "bridge" => {
            // KGW_BRIDGE_INPROCESS_SETLOGGERERROR_V14
            // External bridge keeps the bridge tracing subscriber so raw bridge logs remain visible.
            // In-process bridge must not initialize a second process-global tracing/log subscriber
            // before embedded kaspad starts, otherwise rusty-kaspa can panic with SetLoggerError.
            if bridge_owns_inprocess_node {
                eprintln!(
                    "[KGW][bridge-self-worker][{}] inprocess mode detected; skipping bridge tracing subscriber before embedded kaspad logger initialization",
                    network
                );
            } else {
                kgw_init_bridge_self_worker_raw_tracing_r23();
            }

            kgw_run_bridge_self_worker(&network, &appdir, &rpc, &stratum, &args)
        }
        other => Err(format!("unsupported self-worker role: {other}")),
    };

    if let Err(error) = result {
        eprintln!(
            "[KGW][self-worker][{}][{}] failed: {}",
            role_key, network, error
        );
        std::process::exit(1);
    }

    true
}

fn kgw_self_worker_arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].clone())
}

fn kgw_self_worker_arg_values(args: &[String], key: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == key)
        .map(|window| window[1].clone())
        .collect()
}

fn kgw_bridge_normalize_listen_from_config_r122(value: &str) -> Option<String> {
    let clean = value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string();

    if clean.is_empty() {
        return None;
    }

    if clean.contains(':')
        && clean
            .rsplit(':')
            .next()
            .unwrap_or("")
            .parse::<u16>()
            .is_ok()
    {
        return Some(clean);
    }

    let port_text = clean.trim_start_matches(':').trim();

    let Ok(port) = port_text.parse::<u16>() else {
        return None;
    };

    if port == 0 {
        return None;
    }

    Some(format!("0.0.0.0:{port}"))
}

fn kgw_bridge_config_instance_listens_r122(config_path: &str) -> Result<Vec<String>, String> {
    let path = std::path::PathBuf::from(config_path.trim());

    if !path.is_absolute() {
        return Err(format!(
            "bridge config path must be absolute: {config_path}"
        ));
    }

    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("read bridge config failed {}: {error}", path.display()))?;

    let mut listens = Vec::<String>::new();

    for raw_line in text.lines() {
        let mut line = raw_line.trim();

        if let Some((before_comment, _)) = line.split_once('#') {
            line = before_comment.trim();
        }

        if line.is_empty() {
            continue;
        }

        let lower = line.to_ascii_lowercase();

        let maybe_value = if lower.starts_with("port")
            || lower.starts_with("stratum_port")
            || lower.starts_with("stratum-listen")
            || lower.starts_with("stratum_listen")
            || lower.starts_with("listen")
        {
            line.split_once('=')
                .or_else(|| line.split_once(':'))
                .map(|(_, value)| value.trim().trim_end_matches(',').trim().to_string())
        } else if let Some(index) = lower.find("port=") {
            Some(
                line[index + "port=".len()..]
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            )
        } else if let Some(index) = lower.find("port:") {
            Some(
                line[index + "port:".len()..]
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
            )
        } else {
            None
        };

        if let Some(value) = maybe_value {
            if let Some(listen) = kgw_bridge_normalize_listen_from_config_r122(&value) {
                if !listens.iter().any(|existing| existing == &listen) {
                    listens.push(listen);
                }
            }
        }
    }

    if listens.is_empty() {
        return Err(format!(
            "bridge config did not contain any instance listen ports: {}",
            path.display()
        ));
    }

    Ok(listens)
}

fn kgw_self_worker_default_appdir(network: &str) -> String {
    let network = match network.trim().to_ascii_lowercase().as_str() {
        "testnet" | "testnet10" => "testnet10",
        "testnet12" | "tn12" => "testnet12",
        _ => "mainnet",
    };

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("KaspaGateway")
            .join("nodes")
            .join(network)
            .to_string_lossy()
            .to_string()
    } else {
        std::env::temp_dir()
            .join("KaspaGateway")
            .join("nodes")
            .join(network)
            .to_string_lossy()
            .to_string()
    }
}

fn kgw_self_worker_default_rpc(network: &str) -> &'static str {
    match network.trim().to_ascii_lowercase().as_str() {
        "testnet10" | "testnet" => "127.0.0.1:16210",
        "testnet12" | "tn12" => "127.0.0.1:16310",
        _ => "127.0.0.1:16110",
    }
}

fn kgw_self_worker_default_stratum(network: &str) -> &'static str {
    match network.trim().to_ascii_lowercase().as_str() {
        "testnet10" | "testnet" => "0.0.0.0:15555",
        "testnet12" | "tn12" => "0.0.0.0:25555",
        _ => "0.0.0.0:5555",
    }
}

fn kgw_run_node_self_worker(
    network: &str,
    appdir: &str,
    rpc: &str,
    utxoindex: bool,
    archival: bool,
) -> Result<(), String> {
    let mut settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        network.to_string(),
        "integrated-inproc".to_string(),
        "disable".to_string(),
    )
    .map_err(|error| error.to_string())?;

    settings.app_dir_name = appdir.to_string();
    settings.rpc_endpoint = rpc.to_string();
    settings.enable_utxo_index = utxoindex;
    settings.archival = archival;

    let runtime = kaspa_gateway_rk_node::KgwRealOwnerRuntime::new();
    let status = runtime
        .start_node_owner_session(&settings)
        .map_err(|error| error.to_string())?;

    if !status.official_core_running {
        return Err(format!(
            "official core did not start;network={};policy={};message={}",
            settings.network.as_str(),
            status.start_policy.as_str(),
            status.last_message
        ));
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));

        match runtime.status(settings.network) {
            Ok(_status) => {}
            Err(_error) => {}
        }
    }
}

fn kgw_run_bridge_self_worker(
    network: &str,
    appdir: &str,
    rpc: &str,
    stratum: &str,
    args: &[String],
) -> Result<(), String> {
    let bridge_node_mode = kgw_self_worker_arg_value(args, "--node-mode")
        .unwrap_or_else(|| "external".to_string())
        .trim()
        .to_ascii_lowercase();

    let bridge_cpu_miner_enabled = args.iter().any(|arg| arg == "--internal-cpu-miner");
    let bridge_cpu_miner_address = kgw_self_worker_arg_value(args, "--internal-cpu-miner-address");
    let bridge_cpu_miner_threads = kgw_self_worker_arg_value(args, "--internal-cpu-miner-threads")
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0);
    let bridge_cpu_miner_throttle_ms =
        kgw_self_worker_arg_value(args, "--internal-cpu-miner-throttle-ms")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0);
    let bridge_cpu_miner_template_poll_ms =
        kgw_self_worker_arg_value(args, "--internal-cpu-miner-template-poll-ms")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0);

    let is_inprocess = matches!(
        bridge_node_mode.as_str(),
        "inprocess" | "inproc" | "official-inprocess-node" | "inprocess-node"
    );

    let mut inprocess_node_runtime: Option<kaspa_gateway_rk_node::KgwRealOwnerRuntime> = None;
    let mut bridge_rpc = rpc.to_string();

    if is_inprocess {
        let mut settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
            network.to_string(),
            "integrated-inproc".to_string(),
            "disable".to_string(),
        )
        .map_err(|error| error.to_string())?;

        settings.app_dir_name = appdir.to_string();
        settings.rpc_endpoint = rpc.to_string();
        settings.enable_utxo_index = args.iter().any(|arg| arg == "--utxoindex");
        settings.archival = args.iter().any(|arg| arg == "--archival");

        let runtime = kaspa_gateway_rk_node::KgwRealOwnerRuntime::new();
        let _status = runtime
            .start_node_owner_session(&settings)
            .map_err(|error| error.to_string())?;

        bridge_rpc = settings.rpc_endpoint.clone();
        inprocess_node_runtime = Some(runtime);
    }

    // KGW_BRIDGE_DUAL_CLI_CONFIG_REAL_RUNNER_R122
    // Unified runtime truth:
    // - config mode opens config-defined ports only
    // - CLI/UI mode opens every repeated --bridge-instance-listen
    // - inprocess starts the embedded node first, then starts bridge listeners against its RPC
    let bridge_config_path = kgw_self_worker_arg_value(args, "--bridge-config");

    let mut instance_listens = if let Some(config_path) = bridge_config_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        kgw_bridge_config_instance_listens_r122(config_path)?
    } else {
        kgw_self_worker_arg_values(args, "--bridge-instance-listen")
    };

    if instance_listens.is_empty() {
        instance_listens.push(stratum.to_string());
    }

    instance_listens.sort();
    instance_listens.dedup();

    eprintln!(
        "KGW_BRIDGE_DUAL_CLI_CONFIG_REAL_RUNNER_R122 launch_source={} node_mode={} network={} instance_count={} listens={}",
        if bridge_config_path.is_some() { "config" } else { "cli_instances" },
        if is_inprocess { "inprocess" } else { "external" },
        network,
        instance_listens.len(),
        instance_listens.join("|")
    );

    let mut handles = Vec::new();

    for instance_listen in instance_listens {
        let settings = kaspa_gateway_rk_bridge::BridgeRuntimeSettings {
            network: network.to_string(),
            mode: if is_inprocess {
                kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialInProcessNode
            } else {
                kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialExternalNode
            },
            stratum_listen: Some(instance_listen),
            prometheus_listen: None,
            kaspa_rpc_endpoint: Some(bridge_rpc.clone()),
            internal_cpu_miner: kaspa_gateway_rk_bridge::BridgeInternalCpuMinerSettings {
                enabled: bridge_cpu_miner_enabled,
                address: bridge_cpu_miner_address.clone(),
                threads: bridge_cpu_miner_threads,
                throttle_ms: bridge_cpu_miner_throttle_ms,
                template_poll_ms: bridge_cpu_miner_template_poll_ms,
            },
            explicit_runtime_opt_in: true,
        };

        let event = kaspa_gateway_rk_bridge::bridge_service_event_from_settings_v1(settings)
            .map_err(|error| error.to_string())?;

        let handle = kaspa_gateway_rk_bridge::start_official_bridge_owner_thread_v1(event)
            .map_err(|error| error.to_string())?;

        handles.push(handle);
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));

        if let Some(runtime) = inprocess_node_runtime.as_ref() {
            let _ = runtime.status(
                kaspa_gateway_rk_node::KgwNetwork::parse(network)
                    .map_err(|error| error.to_string())?,
            );
        }

        if handles.iter().all(|handle| handle.is_finished()) {
            return Ok(());
        }
    }
}
