use kaspa_gateway_runtime::runtime_check_default;
use serde::Serialize;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::State;

const MAX_MEMORY_LOGS: usize = 5_000;
const MAX_FIELD_CHARS: usize = 16_384;

#[derive(Default)]
pub struct LogState {
    entries: Mutex<Vec<DesktopLogEntry>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopLogEntry {
    pub id: u64,
    pub timestamp_ms: i64,
    pub severity: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopDiagnosticsReport {
    pub runtime_lines: Vec<String>,
    pub logs_count: usize,
    pub os: String,
    pub arch: String,
    pub package_version: String,
}

#[tauri::command]
pub fn append_log(
    state: State<'_, LogState>,
    severity: String,
    source: String,
    message: String,
) -> Result<DesktopLogEntry, String> {
    let severity = normalize_severity(&severity)?;
    let source = clean_field("source", &source)?;
    let message = clean_field("message", &message)?;

    let mut guard = state
        .entries
        .lock()
        .map_err(|_| "Failed to lock log state.".to_string())?;

    let next_id = guard
        .last()
        .map(|entry| entry.id.saturating_add(1))
        .unwrap_or(1);

    let entry = DesktopLogEntry {
        id: next_id,
        timestamp_ms: now_ms(),
        severity,
        source,
        message,
    };

    guard.push(entry.clone());

    if guard.len() > MAX_MEMORY_LOGS {
        let overflow = guard.len() - MAX_MEMORY_LOGS;
        guard.drain(0..overflow);
    }

    Ok(entry)
}

#[tauri::command]
pub fn list_logs(
    state: State<'_, LogState>,
    severity: Option<String>,
    query: Option<String>,
) -> Result<Vec<DesktopLogEntry>, String> {
    let guard = state
        .entries
        .lock()
        .map_err(|_| "Failed to lock log state.".to_string())?;

    let severity_filter = normalize_query_severity(severity.as_deref().unwrap_or("ALL"));
    let query_filter = query.unwrap_or_default().trim().to_ascii_lowercase();

    let mut logs = Vec::new();

    for entry in guard.iter() {
        let severity_matches = severity_filter == "ALL" || entry.severity == severity_filter;
        let query_matches = query_filter.is_empty()
            || entry.message.to_ascii_lowercase().contains(&query_filter)
            || entry.source.to_ascii_lowercase().contains(&query_filter)
            || entry.severity.to_ascii_lowercase().contains(&query_filter);

        if severity_matches && query_matches {
            logs.push(entry.clone());
        }
    }

    Ok(logs)
}

#[tauri::command]
pub fn clear_logs(state: State<'_, LogState>) -> Result<String, String> {
    let mut guard = state
        .entries
        .lock()
        .map_err(|_| "Failed to lock log state.".to_string())?;

    guard.clear();

    Ok("Logs cleared.".to_string())
}

#[tauri::command]
pub fn diagnostics_report(state: State<'_, LogState>) -> Result<DesktopDiagnosticsReport, String> {
    let runtime = runtime_check_default().map_err(|error| error.to_string())?;
    let logs_count = state
        .entries
        .lock()
        .map_err(|_| "Failed to lock log state.".to_string())?
        .len();

    Ok(DesktopDiagnosticsReport {
        runtime_lines: runtime.to_lines(),
        logs_count,
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        package_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

fn normalize_severity(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_ascii_uppercase();

    match normalized.as_str() {
        "INFO" | "WARN" | "ERROR" | "DEBUG" | "TRACE" => Ok(normalized),
        _ => Err("Unsupported log severity.".to_string()),
    }
}

fn normalize_query_severity(value: &str) -> String {
    match value.trim().to_ascii_uppercase().as_str() {
        "INFO" | "WARN" | "ERROR" | "DEBUG" | "TRACE" => value.trim().to_ascii_uppercase(),
        _ => "ALL".to_string(),
    }
}

fn clean_field(name: &str, value: &str) -> Result<String, String> {
    let cleaned = value.trim();

    if cleaned.is_empty() {
        return Err(format!("{name} cannot be empty."));
    }

    if cleaned.contains('\0') {
        return Err(format!("{name} cannot contain null bytes."));
    }

    Ok(cleaned
        .replace(['\r', '\n'], " ")
        .chars()
        .take(MAX_FIELD_CHARS)
        .collect())
}

fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_filters_are_normalized() {
        assert_eq!(normalize_severity("info").unwrap(), "INFO");
        assert_eq!(normalize_query_severity("bad"), "ALL");
    }

    #[test]
    fn fields_are_cleaned() {
        assert_eq!(clean_field("message", "a\r\nb").unwrap(), "a  b");
        assert!(clean_field("message", "\0").is_err());
    }
}
