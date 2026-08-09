use crate::app_logger;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaspaPriceSnapshot {
    pub prices: BTreeMap<String, f64>,
    pub source: String,
    pub fetched_at_ms: i128,
}

#[derive(Debug, Clone)]
struct CachedPrice {
    snapshot: KaspaPriceSnapshot,
    inserted_at: Instant,
}

static PRICE_CACHE: OnceLock<Mutex<Option<CachedPrice>>> = OnceLock::new();

fn cache() -> &'static Mutex<Option<CachedPrice>> {
    PRICE_CACHE.get_or_init(|| Mutex::new(None))
}

fn now_ms() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i128)
        .unwrap_or_default()
}

fn number_at_path(value: &Value, path: &[&str]) -> Option<f64> {
    let mut current = value;

    for key in path {
        current = current.get(*key)?;
    }

    current
        .as_f64()
        .or_else(|| current.as_str()?.replace(',', "").parse::<f64>().ok())
}

fn find_number_by_key_name(value: &Value, names: &[&str]) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.replace(',', "").parse::<f64>().ok(),
        Value::Array(items) => {
            for item in items {
                if let Some(number) = find_number_by_key_name(item, names) {
                    return Some(number);
                }
            }

            None
        }
        Value::Object(map) => {
            for (key, child) in map {
                let lower = key.to_ascii_lowercase();

                if names
                    .iter()
                    .any(|name| lower.contains(&name.to_ascii_lowercase()))
                    && let Some(number) = child
                        .as_f64()
                        .or_else(|| child.as_str()?.replace(',', "").parse::<f64>().ok())
                {
                    return Some(number);
                }

                if let Some(number) = find_number_by_key_name(child, names) {
                    return Some(number);
                }
            }

            None
        }
        _ => None,
    }
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
        .map_err(|error| format!("{url}: failed reading response body: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "{url}: HTTP {status}: {}",
            text.chars().take(180).collect::<String>()
        ));
    }

    serde_json::from_str::<Value>(&text).map_err(|error| {
        format!(
            "{url}: invalid JSON: {error}; sample={}",
            text.chars().take(180).collect::<String>()
        )
    })
}

async fn fetch_kaspa_prices_uncached() -> Result<KaspaPriceSnapshot, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("KaspaGateway/0.1.0 price-service")
        .build()
        .map_err(|error| format!("failed to build price HTTP client: {error}"))?;

    let sources = [
        (
            "CoinGecko simple price",
            "https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd,sar,eur,gbp,chf,aud,cad,jpy,krw,rub,cny,try,inr,idr,hkd,sgd,brl",
        ),
        ("Kaspa API price", "https://api.kaspa.org/info/price"),
    ];

    let mut errors = Vec::new();

    for (source, url) in sources {
        match fetch_json(&client, url).await {
            Ok(json) => {
                let mut prices = BTreeMap::<String, f64>::new();

                if source == "CoinGecko simple price" {
                    if let Some(usd) = number_at_path(&json, &["kaspa", "usd"]) {
                        prices.insert("usd".to_string(), usd);
                    }
                    if let Some(sar) = number_at_path(&json, &["kaspa", "sar"]) {
                        prices.insert("sar".to_string(), sar);
                    }

                    // KGW_R81C_PRICE_SERVICE_EXTRA_DISPLAY_CURRENCIES
                    for code in [
                        "eur", "gbp", "chf", "aud", "cad", "jpy", "krw", "rub", "cny", "try",
                        "inr", "idr", "hkd", "sgd", "brl",
                    ] {
                        if let Some(value) = number_at_path(&json, &["kaspa", code]) {
                            prices.insert(code.to_string(), value);
                        }
                    }
                } else if let Some(usd) = find_number_by_key_name(&json, &["price", "usd"]) {
                    prices.insert("usd".to_string(), usd);
                }

                prices.retain(|_, price| price.is_finite() && *price >= 0.0);

                if !prices.is_empty() {
                    app_logger::log_info(
                        "price",
                        &format!(
                            "kaspa price fetched source={} values={}",
                            source,
                            prices
                                .iter()
                                .map(|(code, price)| {
                                    format!("{}={:.8}", code.to_uppercase(), price)
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    );

                    return Ok(KaspaPriceSnapshot {
                        prices,
                        source: source.to_string(),
                        fetched_at_ms: now_ms(),
                    });
                }

                errors.push(format!("{source}: no usable prices"));
            }
            Err(error) => errors.push(error),
        }
    }

    Err(errors.join(" | "))
}

pub async fn get_kaspa_prices() -> Result<KaspaPriceSnapshot, String> {
    {
        let guard = cache()
            .lock()
            .map_err(|_| "price cache lock poisoned".to_string())?;

        if let Some(cached) = guard.as_ref()
            && cached.inserted_at.elapsed() < Duration::from_secs(60)
        {
            return Ok(cached.snapshot.clone());
        }
    }

    let snapshot = fetch_kaspa_prices_uncached().await?;

    {
        let mut guard = cache()
            .lock()
            .map_err(|_| "price cache lock poisoned".to_string())?;

        *guard = Some(CachedPrice {
            snapshot: snapshot.clone(),
            inserted_at: Instant::now(),
        });
    }

    Ok(snapshot)
}

#[tauri::command]
pub async fn kgw_get_kaspa_prices() -> Result<KaspaPriceSnapshot, String> {
    get_kaspa_prices().await
}
