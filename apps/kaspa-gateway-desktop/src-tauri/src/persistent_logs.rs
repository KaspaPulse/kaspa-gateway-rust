use kaspa_gateway_config::default_user_data_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_MESSAGE_CHARS: usize = 16_384;
const MAX_LIST_LIMIT: usize = 20_000;
const MAX_TAIL_BYTES: u64 = 256_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentLogRequest {
    pub severity: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentLogQuery {
    pub severity: String,
    pub query: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentLogEntry {
    pub timestamp_ms: i64,
    pub severity: String,
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistentLogReport {
    pub path: String,
    pub total_lines: usize,
    pub returned_lines: usize,
    pub entries: Vec<PersistentLogEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistentLogTailReport {
    pub path: String,
    pub offset: u64,
    pub next_offset: u64,
    pub entries: Vec<PersistentLogEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistentLogStats {
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
    pub total_lines: usize,
}

#[tauri::command]
pub fn persistent_log_append(request: PersistentLogRequest) -> Result<PersistentLogEntry, String> {
    let severity = normalize_severity(&request.severity);
    let source = clean_short_field("source", &request.source)?;
    let message = clean_message(&request.message)?;

    let entry = PersistentLogEntry {
        timestamp_ms: now_ms(),
        severity,
        source,
        message,
    };

    let path = log_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| error.to_string())?;

    let line = serde_json::to_string(&entry).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())?;

    Ok(entry)
}

#[tauri::command]
pub fn persistent_log_list(query: PersistentLogQuery) -> Result<PersistentLogReport, String> {
    let path = log_path()?;

    if !path.exists() {
        return Ok(PersistentLogReport {
            path: path.display().to_string(),
            total_lines: 0,
            returned_lines: 0,
            entries: Vec::new(),
        });
    }

    let severity = normalize_query_severity(&query.severity);
    let search = query.query.trim().to_ascii_lowercase();
    let limit = query.limit.clamp(1, MAX_LIST_LIMIT);

    let file = fs::File::open(&path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);

    let mut total_lines = 0_usize;
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|error| error.to_string())?;
        total_lines += 1;

        let Ok(entry) = serde_json::from_str::<PersistentLogEntry>(&line) else {
            continue;
        };

        if severity != "ALL" && entry.severity != severity {
            continue;
        }

        if !search.is_empty() {
            let haystack = format!("{} {} {}", entry.severity, entry.source, entry.message)
                .to_ascii_lowercase();

            if !haystack.contains(&search) {
                continue;
            }
        }

        entries.push(entry);
    }

    entries.reverse();
    entries.truncate(limit);

    Ok(PersistentLogReport {
        path: path.display().to_string(),
        total_lines,
        returned_lines: entries.len(),
        entries,
    })
}

#[tauri::command]
pub fn persistent_log_tail(offset: u64, max_bytes: u64) -> Result<PersistentLogTailReport, String> {
    let path = log_path()?;

    if !path.exists() {
        return Ok(PersistentLogTailReport {
            path: path.display().to_string(),
            offset,
            next_offset: offset,
            entries: Vec::new(),
        });
    }

    let mut file = fs::File::open(&path).map_err(|error| error.to_string())?;
    let size = file.metadata().map_err(|error| error.to_string())?.len();
    let safe_offset = offset.min(size);
    let limit = max_bytes.clamp(1, MAX_TAIL_BYTES);
    let to_read = size.saturating_sub(safe_offset).min(limit);

    file.seek(SeekFrom::Start(safe_offset))
        .map_err(|error| error.to_string())?;

    let mut buffer = vec![0_u8; usize::try_from(to_read).unwrap_or(MAX_TAIL_BYTES as usize)];
    let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
    buffer.truncate(read);

    let text = String::from_utf8_lossy(&buffer);
    let entries = text
        .lines()
        .filter_map(|line| serde_json::from_str::<PersistentLogEntry>(line).ok())
        .collect::<Vec<_>>();

    Ok(PersistentLogTailReport {
        path: path.display().to_string(),
        offset: safe_offset,
        next_offset: safe_offset.saturating_add(read as u64),
        entries,
    })
}

#[tauri::command]
pub fn persistent_log_clear() -> Result<String, String> {
    let path = log_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    fs::write(&path, "").map_err(|error| error.to_string())?;

    Ok("Persistent log cleared.".to_string())
}

#[tauri::command]
pub fn persistent_log_stats() -> Result<PersistentLogStats, String> {
    let path = log_path()?;
    let exists = path.exists();

    if !exists {
        return Ok(PersistentLogStats {
            path: path.display().to_string(),
            exists,
            size_bytes: 0,
            total_lines: 0,
        });
    }

    let file = fs::File::open(&path).map_err(|error| error.to_string())?;
    let reader = BufReader::new(file);
    let mut total_lines = 0_usize;

    for line in reader.lines() {
        line.map_err(|error| error.to_string())?;
        total_lines += 1;
    }

    Ok(PersistentLogStats {
        path: path.display().to_string(),
        exists,
        size_bytes: fs::metadata(&path)
            .map_err(|error| error.to_string())?
            .len(),
        total_lines,
    })
}

fn log_path() -> Result<PathBuf, String> {
    let root = default_user_data_dir().map_err(|error| error.to_string())?;
    Ok(root.join("logs").join("kaspa_gateway_persistent.log.jsonl"))
}

fn clean_short_field(name: &str, value: &str) -> Result<String, String> {
    let cleaned = value.trim();

    if cleaned.is_empty()
        || cleaned.contains('\0')
        || cleaned.contains('\n')
        || cleaned.contains('\r')
    {
        return Err(format!("{name} contains unsafe characters."));
    }

    Ok(cleaned.chars().take(256).collect())
}

fn clean_message(value: &str) -> Result<String, String> {
    if value.contains('\0') {
        return Err("Log message contains unsafe characters.".to_string());
    }

    let cleaned = value.replace(['\r', '\n'], " ");
    Ok(cleaned.trim().chars().take(MAX_LOG_MESSAGE_CHARS).collect())
}

fn normalize_severity(value: &str) -> String {
    match value.trim().to_ascii_uppercase().as_str() {
        "TRACE" => "TRACE".to_string(),
        "DEBUG" => "DEBUG".to_string(),
        "WARN" => "WARN".to_string(),
        "ERROR" => "ERROR".to_string(),
        _ => "INFO".to_string(),
    }
}

fn normalize_query_severity(value: &str) -> String {
    match value.trim().to_ascii_uppercase().as_str() {
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" => value.trim().to_ascii_uppercase(),
        _ => "ALL".to_string(),
    }
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
    fn severity_is_normalized() {
        assert_eq!(normalize_severity("warn"), "WARN");
        assert_eq!(normalize_severity("bad"), "INFO");
        assert_eq!(normalize_query_severity("bad"), "ALL");
    }

    #[test]
    fn message_is_cleaned() {
        assert_eq!(clean_message("a\r\nb").unwrap(), "a  b");
        assert!(clean_message("bad\0").is_err());
    }
}
