use crate::app_logger;
use crate::price_service;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetric {
    pub value: String,
    pub status: String,
    pub source: String,
    pub updated_at_epoch_ms: i128,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetricsSnapshot {
    pub price: LiveMetric,
    pub hashrate: LiveMetric,
    pub difficulty: LiveMetric,
    pub status: String,
}

fn now_ms() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i128)
        .unwrap_or_default()
}

fn ok_metric(value: String, source: &str) -> LiveMetric {
    LiveMetric {
        value,
        status: "ok".to_string(),
        source: source.to_string(),
        updated_at_epoch_ms: now_ms(),
        error: None,
    }
}

fn error_metric(source: &str, error: String) -> LiveMetric {
    LiveMetric {
        value: "Error".to_string(),
        status: "error".to_string(),
        source: source.to_string(),
        updated_at_epoch_ms: now_ms(),
        error: Some(error),
    }
}

fn as_f64(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Number(number)) => number.as_f64().unwrap_or_default(),
        Some(Value::String(text)) => text.replace(',', "").parse::<f64>().unwrap_or_default(),
        _ => 0.0,
    }
}

fn trim_number(value: f64, decimals: usize) -> String {
    let mut text = format!("{value:.decimals$}");

    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }

    if text.ends_with('.') {
        text.pop();
    }

    text
}

fn format_hashrate_from_api(raw_hashrate: f64) -> String {
    // api.kaspa.org/info/hashrate returns a value matching TH/s.
    // Example: 389724 TH/s => 389.72 PH/s.
    let ph = raw_hashrate / 1_000.0;

    if ph >= 1_000.0 {
        format!("{} EH/s", trim_number(ph / 1_000.0, 2))
    } else {
        format!("{} PH/s", trim_number(ph, 2))
    }
}

fn format_difficulty(raw: f64) -> String {
    let units = [
        ("E", 1e18_f64),
        ("P", 1e15_f64),
        ("T", 1e12_f64),
        ("G", 1e9_f64),
        ("M", 1e6_f64),
        ("K", 1e3_f64),
    ];

    for (unit, factor) in units {
        if raw.abs() >= factor {
            return format!("{}{}", trim_number(raw / factor, 2), unit);
        }
    }

    trim_number(raw, 2)
}

async fn fetch_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("{url}: request failed: {error}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("{url}: failed reading response: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "{url}: HTTP {status}: {}",
            text.chars().take(160).collect::<String>()
        ));
    }

    serde_json::from_str::<Value>(&text).map_err(|error| {
        format!(
            "{url}: invalid JSON: {error}; sample={}",
            text.chars().take(160).collect::<String>()
        )
    })
}

async fn fetch_price_metric() -> LiveMetric {
    match price_service::get_kaspa_prices().await {
        Ok(snapshot) => {
            let usd = snapshot.prices.get("usd").copied().unwrap_or_default();
            ok_metric(format!("{:.4} USD", usd), &snapshot.source)
        }
        Err(error) => error_metric("price_service", error),
    }
}

async fn fetch_hashrate_metric(client: &reqwest::Client) -> LiveMetric {
    match fetch_json(client, "https://api.kaspa.org/info/hashrate").await {
        Ok(json) => {
            let raw = as_f64(json.get("hashrate"));
            ok_metric(format_hashrate_from_api(raw), "api.kaspa.org/info/hashrate")
        }
        Err(error) => error_metric("api.kaspa.org/info/hashrate", error),
    }
}

async fn fetch_difficulty_metric(client: &reqwest::Client) -> LiveMetric {
    match fetch_json(client, "https://api.kaspa.org/info/blockdag").await {
        Ok(json) => {
            let raw = as_f64(json.get("difficulty"));
            ok_metric(format_difficulty(raw), "api.kaspa.org/info/blockdag")
        }
        Err(error) => error_metric("api.kaspa.org/info/blockdag", error),
    }
}

async fn build_snapshot() -> LiveMetricsSnapshot {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("KaspaGateway/0.1.0 live-metrics")
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            let error = format!("failed to build HTTP client: {error}");

            return LiveMetricsSnapshot {
                price: error_metric("live_metrics", error.clone()),
                hashrate: error_metric("live_metrics", error.clone()),
                difficulty: error_metric("live_metrics", error),
                status: "error".to_string(),
            };
        }
    };

    let price = fetch_price_metric().await;
    let hashrate = fetch_hashrate_metric(&client).await;
    let difficulty = fetch_difficulty_metric(&client).await;

    let status = if price.status == "ok" && hashrate.status == "ok" && difficulty.status == "ok" {
        "ok"
    } else {
        "partial"
    }
    .to_string();

    app_logger::log_info(
        "live_metrics",
        &format!(
            "snapshot built status={} price={} hashrate={} difficulty={}",
            status, price.value, hashrate.value, difficulty.value
        ),
    );

    LiveMetricsSnapshot {
        price,
        hashrate,
        difficulty,
        status,
    }
}

#[tauri::command]
pub async fn kgw_live_metrics_snapshot() -> Result<LiveMetricsSnapshot, String> {
    Ok(build_snapshot().await)
}

#[tauri::command]
pub async fn kgw_live_metrics_refresh_now() -> Result<LiveMetricsSnapshot, String> {
    Ok(build_snapshot().await)
}
