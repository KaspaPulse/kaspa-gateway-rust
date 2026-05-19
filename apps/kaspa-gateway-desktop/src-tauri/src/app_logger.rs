use kaspa_gateway_config::default_user_data_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use tracing_subscriber::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAppendRequest {
    pub level: Option<String>,
    pub target: Option<String>,
    pub message: String,
}

struct KgwAppLogLayer;

#[derive(Default)]
struct KgwTracingEventVisitor {
    message: Option<String>,
    fields: Vec<String>,
}

impl tracing::field::Visit for KgwTracingEventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let mut rendered = format!("{value:?}");

        if rendered.len() >= 2 && rendered.starts_with('"') && rendered.ends_with('"') {
            rendered = rendered[1..rendered.len() - 1].to_string();
        }

        if field.name() == "message" {
            self.message = Some(rendered);
        } else {
            self.fields.push(format!("{}={}", field.name(), rendered));
        }
    }
}

impl<S> tracing_subscriber::Layer<S> for KgwAppLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let target = metadata.target();

        // Keep the bridge narrow and safe: only transaction progress from crates.
        if target != "transactions" {
            return;
        }

        let mut visitor = KgwTracingEventVisitor::default();
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_else(|| visitor.fields.join(" "));

        if message.trim().is_empty() {
            return;
        }

        match *metadata.level() {
            tracing::Level::ERROR => log_error(target, &message),
            tracing::Level::WARN => {
                let _ = write_log_line("WARN", target, &message);
            }
            tracing::Level::INFO => log_info(target, &message),
            tracing::Level::DEBUG | tracing::Level::TRACE => {}
        }
    }
}

pub fn init_tracing_bridge() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let subscriber = tracing_subscriber::registry().with(KgwAppLogLayer);

        match tracing::subscriber::set_global_default(subscriber) {
            Ok(()) => log_info(
                "app_logger",
                "Tracing bridge installed for target=transactions.",
            ),
            Err(error) => {
                let _ = write_log_line(
                    "WARN",
                    "app_logger",
                    &format!("Tracing bridge not installed: {error}"),
                );
            }
        }
    });
}

pub fn app_log_dir() -> Result<PathBuf, String> {
    let dir = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("logs");

    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

pub fn app_log_file() -> Result<PathBuf, String> {
    Ok(app_log_dir()?.join("KaspaGateway.log"))
}

pub fn write_log_line(level: &str, target: &str, message: &str) -> Result<(), String> {
    let path = app_log_file()?;

    let line = format!("{}\n", kgw_python_style_log_line(level, target, message));

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;

    file.write_all(line.as_bytes())
        .map_err(|error| error.to_string())
}

fn kgw_should_drop_noisy_info_log(target: &str, message: &str) -> bool {
    let noisy = (target == "live_metrics" && message.starts_with("snapshot built status=ok"))
        || (target == "price" && message.starts_with("kaspa price fetched source="));

    if !noisy {
        return false;
    }

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0);

    static LAST_NOISY_INFO_LOG_MS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::BTreeMap<String, i64>>,
    > = std::sync::OnceLock::new();

    let map = LAST_NOISY_INFO_LOG_MS
        .get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()));

    let key = format!(
        "{target}:{}",
        message
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
    );

    let Ok(mut guard) = map.lock() else {
        return false;
    };

    let last = guard.get(&key).copied().unwrap_or(0);
    let min_interval_ms = 5 * 60 * 1000;

    if now_ms.saturating_sub(last) < min_interval_ms {
        return true;
    }

    guard.insert(key, now_ms);
    false
}

fn kgw_epoch_ms_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn kgw_civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
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

fn kgw_python_log_timestamp(ms: i64) -> String {
    let seconds = ms.div_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);

    let (year, month, day) = kgw_civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

fn kgw_log_worker(target: &str, message: &str) -> &'static str {
    match target {
        "transactions" => "FetchWorker",
        "top_addresses" => "TopAddressesThread",
        "price" => "InitialPriceWorker",
        "live_metrics" => "MetricsWorker",
        "addresses" => "MainThread",
        "settings" => "MainThread",
        "app_logger" => "MainThread",
        "analysis" => "MainThread",
        "frontend" => "MainThread",
        _ if message.contains("fetch") || message.contains("Fetch") => "FetchWorker",
        _ => "MainThread",
    }
}

fn kgw_value_after<'a>(message: &'a str, key: &str) -> Option<&'a str> {
    let start = message.find(key)?;
    let rest = &message[start + key.len()..];
    let rest = rest.strip_prefix('=').unwrap_or(rest);
    Some(
        rest.split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(','),
    )
}

fn kgw_pretty_log_message(target: &str, message: &str) -> String {
    if target == "transactions" && message.starts_with("unified fetch queued") {
        let start_ts = kgw_value_after(message, "start_ts").unwrap_or("-");
        let end_ts = kgw_value_after(message, "end_ts").unwrap_or("-");
        let force = kgw_value_after(message, "force").unwrap_or("-");
        let max_pages = kgw_value_after(message, "max_pages").unwrap_or("-");

        return format!(
            "FETCH LOOP STARTED. StartTS: {start_ts}, EndTS: {end_ts}, Force: {force}, MaxPages: {max_pages}"
        );
    }

    if target == "transactions" && message.starts_with("unified fetch started") {
        return "Fetching from network...".to_string();
    }

    if target == "transactions" && message.starts_with("unified grouped list started") {
        return "Building grouped transaction table...".to_string();
    }

    if target == "transactions" && message.starts_with("unified fetch ok") {
        let stored = kgw_value_after(message, "stored").unwrap_or("-");
        let fetched = kgw_value_after(message, "fetched").unwrap_or("-");
        let accepted = kgw_value_after(message, "accepted").unwrap_or("-");
        let pages = kgw_value_after(message, "pages").unwrap_or("-");
        let local_loaded = kgw_value_after(message, "local_loaded").unwrap_or("-");
        let deleted_before_fetch = kgw_value_after(message, "deleted_before_fetch").unwrap_or("-");
        let stopped_existing = kgw_value_after(message, "stopped_existing").unwrap_or("-");
        let stopped_start = kgw_value_after(message, "stopped_start").unwrap_or("-");
        let stop_reason = kgw_value_after(message, "stop_reason").unwrap_or("-");
        let groups = kgw_value_after(message, "groups").unwrap_or("-");
        let rows = kgw_value_after(message, "rows").unwrap_or("-");
        let elapsed = kgw_value_after(message, "elapsed_ms").unwrap_or("-");

        return format!(
            "Fetch done ({elapsed} ms), stored={stored}, fetched={fetched}, accepted={accepted}, pages={pages}, local={local_loaded}, deleted={deleted_before_fetch}, stopped_existing={stopped_existing}, stopped_start={stopped_start}, stop={stop_reason}, groups={groups}, rows={rows}"
        );
    }

    if target == "transactions" && message.starts_with("unified fetch failed") {
        return format!("Fetch failed: {message}");
    }

    if target == "top_addresses"
        && message.contains("requesting https://api.kaspa.org/addresses/names")
    {
        return "Fetching address names map...".to_string();
    }

    if target == "top_addresses"
        && message.contains("requesting https://api.kaspa.org/addresses/top")
    {
        return "Fetching top addresses ranking...".to_string();
    }

    if target == "top_addresses" && message.starts_with("address names fetched") {
        return message.replace("address names fetched", "Address names fetched");
    }

    if target == "top_addresses" && message.starts_with("top addresses fetch ok") {
        return message.replace("top addresses fetch ok", "Top addresses fetch done");
    }

    if target == "price" && message.starts_with("kaspa price fetched") {
        return message.replace(
            "kaspa price fetched source=",
            "Successfully fetched prices: source=",
        );
    }

    if target == "live_metrics" && message.starts_with("snapshot built status=ok") {
        return message.replace("snapshot built status=ok", "Network metrics updated:");
    }

    if target == "addresses"
        && message.starts_with("save_address skipped existing saved address without new name")
    {
        return "Address already saved; no name update needed.".to_string();
    }

    if target == "addresses" && message.starts_with("save_address frontend ok") {
        return "Address saved successfully.".to_string();
    }

    if target == "addresses" && message.starts_with("get_all_addresses ok") {
        return message.replace("get_all_addresses ok", "Saved addresses loaded");
    }

    if target == "addresses" && message.starts_with("saved addresses refreshed") {
        return message.replace("saved addresses refreshed", "Address dropdown refreshed");
    }

    message.to_string()
}

fn kgw_python_style_log_line(level: &str, target: &str, message: &str) -> String {
    let timestamp = kgw_python_log_timestamp(kgw_epoch_ms_now());
    let worker = kgw_log_worker(target, message);
    let pretty = kgw_pretty_log_message(target, message);

    format!("{timestamp} - {level} - [{worker}] - {target} - {pretty}")
}

pub fn log_info(target: &str, message: &str) {
    if kgw_should_drop_noisy_info_log(target, message) {
        return;
    }
    let _ = write_log_line("INFO", target, message);
}

pub fn log_error(target: &str, message: &str) {
    let _ = write_log_line("ERROR", target, message);
}

#[tauri::command]
pub fn kgw_log_file_path() -> Result<String, String> {
    Ok(app_log_file()?.display().to_string())
}

#[tauri::command]
pub fn kgw_log_append(request: LogAppendRequest) -> Result<(), String> {
    let level = request.level.unwrap_or_else(|| "INFO".to_string());
    let target = request.target.unwrap_or_else(|| "frontend".to_string());

    write_log_line(&level, &target, &request.message)
}

#[tauri::command]
pub fn kgw_log_read(max_lines: Option<usize>) -> Result<Vec<String>, String> {
    let path = app_log_file()?;

    if !path.exists() {
        write_log_line("INFO", "app_logger", "Log file initialized.")?;
    }

    let mut text = String::new();

    OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|error| error.to_string())?
        .read_to_string(&mut text)
        .map_err(|error| error.to_string())?;

    let limit = max_lines.unwrap_or(2_000).clamp(100, 50_000);
    let lines = text
        .lines()
        .rev()
        .take(limit)
        .map(str::to_string)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>();

    Ok(lines)
}

#[tauri::command]
pub fn kgw_log_clear() -> Result<(), String> {
    let path = app_log_file()?;

    fs::write(&path, "").map_err(|error| error.to_string())?;
    write_log_line("INFO", "app_logger", "Log cleared.")
}
