use crate::app_logger;
use crate::price_service;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAddressRow {
    pub rank: usize,
    pub known_name: String,
    pub address: String,

    // Python parity: amount is kept as the original balance value from /addresses/top.
    pub balance: f64,
    pub amount: f64,

    // Calculated totals using live prices.
    pub kas_price_usd: Option<f64>,
    pub total_usd: Option<f64>,
    pub total_sar: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAddressesResponse {
    pub rows: Vec<TopAddressRow>,
    pub source: String,
    pub prices: BTreeMap<String, f64>,
}

fn as_f64(value: Option<&Value>) -> f64 {
    match value {
        Some(Value::Number(number)) => number.as_f64().unwrap_or_default(),
        Some(Value::String(text)) => text.replace(',', "").parse::<f64>().unwrap_or_default(),
        _ => 0.0,
    }
}

fn as_string(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn json_preview(value: &Value) -> String {
    let mut text = value.to_string();
    if text.len() > 900 {
        text.truncate(900);
        text.push_str("...");
    }
    text
}

/// Python parity:
/// if raw_data is list and raw_data[0] has "ranking", use raw_data[0]["ranking"].
/// elif raw_data is list, use raw_data directly.
fn extract_python_top_address_items(value: Value) -> Vec<Value> {
    match value {
        Value::Array(items) => {
            if let Some(Value::Object(first)) = items.first() {
                if let Some(Value::Array(ranking)) = first.get("ranking") {
                    return ranking.clone();
                }
            }

            items
        }
        Value::Object(mut object) => {
            if let Some(Value::Array(ranking)) = object.remove("ranking") {
                ranking
            } else if let Some(Value::Array(addresses)) = object.remove("addresses") {
                addresses
            } else if let Some(Value::Array(items)) = object.remove("items") {
                items
            } else if let Some(Value::Array(results)) = object.remove("results") {
                results
            } else if let Some(Value::Array(data)) = object.remove("data") {
                data
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

fn row_from_value(
    index: usize,
    value: Value,
    names: &HashMap<String, String>,
    prices: &BTreeMap<String, f64>,
) -> Option<TopAddressRow> {
    let Value::Object(object) = value else {
        return None;
    };

    let address = as_string(
        object
            .get("address")
            .or_else(|| object.get("kaspa_address"))
            .or_else(|| object.get("wallet")),
    );

    if address.is_empty() {
        return None;
    }

    // Python parity: item.get("rank", i) + 1
    let rank = object
        .get("rank")
        .or_else(|| object.get("position"))
        .and_then(Value::as_u64)
        .map(|value| value as usize + 1)
        .unwrap_or(index + 1);

    let known_name = names.get(&address).cloned().unwrap_or_default();

    // Python parity: Balance = float(item.get("amount", 0))
    let balance = as_f64(
        object
            .get("amount")
            .or_else(|| object.get("balance"))
            .or_else(|| object.get("balance_kas"))
            .or_else(|| object.get("sompi")),
    );

    let kas_price_usd = prices.get("usd").copied();
    let total_usd = kas_price_usd.map(|price| balance * price);
    let total_sar = prices.get("sar").copied().map(|price| balance * price);

    Some(TopAddressRow {
        rank,
        known_name,
        address,
        balance,
        amount: balance,
        kas_price_usd,
        total_usd,
        total_sar,
    })
}

async fn fetch_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    app_logger::log_info("top_addresses", &format!("requesting {url}"));

    let response = client
        .get(url)
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("request failed for {url}: {error}"))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("failed to read response body from {url}: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "HTTP {status} from {url}: {}",
            text.chars().take(180).collect::<String>()
        ));
    }

    serde_json::from_str::<Value>(&text).map_err(|error| {
        format!(
            "invalid JSON from {url}: {error}; sample={}",
            text.chars().take(180).collect::<String>()
        )
    })
}

async fn fetch_address_names_map(client: &reqwest::Client) -> HashMap<String, String> {
    let candidates = [
        "https://api.kaspa.org/addresses/names",
        "https://api.kaspa.org/address-names",
        "https://api.kaspa.org/info/address-names",
    ];

    for url in candidates {
        match fetch_json(client, url).await {
            Ok(Value::Array(items)) => {
                let mut map = HashMap::new();

                for item in items {
                    if let Value::Object(object) = item {
                        let address = as_string(object.get("address"));
                        let name = as_string(object.get("name"));

                        if !address.is_empty() && !name.is_empty() {
                            map.insert(address, name);
                        }
                    }
                }

                app_logger::log_info(
                    "top_addresses",
                    &format!("address names fetched count={}", map.len()),
                );

                return map;
            }
            Ok(other) => {
                app_logger::log_error(
                    "top_addresses",
                    &format!(
                        "address names unsupported response url={url} preview={}",
                        json_preview(&other)
                    ),
                );
            }
            Err(error) => {
                app_logger::log_error(
                    "top_addresses",
                    &format!("address names fetch failed url={url}: {error}"),
                );
            }
        }
    }

    HashMap::new()
}

async fn fetch_top_addresses_inner(limit: Option<usize>) -> Result<TopAddressesResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .user_agent("KaspaGateway/0.1.0 top-addresses")
        .build()
        .map_err(|error| format!("failed to build top addresses HTTP client: {error}"))?;

    let limit = limit.unwrap_or(10_000).clamp(1, 10_000);

    app_logger::log_info(
        "top_addresses",
        &format!("top addresses fetch requested limit={limit}"),
    );
    let prices = match price_service::get_kaspa_prices().await {
        Ok(snapshot) => {
            app_logger::log_info(
                "top_addresses",
                &format!(
                    "using shared kaspa price source={} currencies={}",
                    snapshot.source,
                    snapshot
                        .prices
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            );
            snapshot.prices
        }
        Err(error) => {
            app_logger::log_error(
                "top_addresses",
                &format!("shared kaspa price failed: {error}"),
            );
            std::collections::BTreeMap::new()
        }
    };
    let names = fetch_address_names_map(&client).await;

    // API/Python parity: /addresses/top works; /addresses/top?limit=100 returns 400.
    let url = "https://api.kaspa.org/addresses/top?limit=1";

    let json = fetch_json(&client, url).await?;
    let preview = json_preview(&json);

    let rows = extract_python_top_address_items(json)
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| row_from_value(index, value, &names, &prices))
        .take(limit)
        .collect::<Vec<_>>();

    if rows.is_empty() {
        let error = format!("empty or unsupported response from {url}; preview={preview}");
        app_logger::log_error("top_addresses", &error);
        return Err(error);
    }

    app_logger::log_info(
        "top_addresses",
        &format!(
            "top addresses fetch ok rows={} names={} prices={} source={}",
            rows.len(),
            names.len(),
            prices.len(),
            url
        ),
    );

    Ok(TopAddressesResponse {
        rows,
        source: url.to_string(),
        prices,
    })
}

#[tauri::command]
pub async fn fetch_top_addresses_rust(
    limit: Option<usize>,
) -> Result<TopAddressesResponse, String> {
    match fetch_top_addresses_inner(limit).await {
        Ok(response) => Ok(response),
        Err(error) => {
            app_logger::log_error(
                "top_addresses",
                &format!("fetch_top_addresses_rust failed: {error}"),
            );
            Err(error)
        }
    }
}

/* KGW_PHASE14_COMPLETE_TOP_ADDRESSES_WRAPPERS */

#[tauri::command]
pub fn top_addresses_load_known_names(
) -> Result<Vec<crate::top_addresses_deep::KnownNameRecord>, String> {
    crate::top_addresses_deep::top_addresses_load_known_names_impl()
}

#[tauri::command]
pub fn top_addresses_load_currency_rates(
) -> Result<Vec<crate::top_addresses_deep::CurrencyRateRecord>, String> {
    crate::top_addresses_deep::top_addresses_load_currency_rates_impl()
}

#[tauri::command]
pub fn top_addresses_save_known_names(
    records: Vec<crate::top_addresses_deep::KnownNameRecord>,
) -> Result<String, String> {
    crate::top_addresses_deep::top_addresses_save_known_names_impl(records)
}

#[tauri::command]
pub fn top_addresses_save_currency_rates(
    records: Vec<crate::top_addresses_deep::CurrencyRateRecord>,
) -> Result<String, String> {
    crate::top_addresses_deep::top_addresses_save_currency_rates_impl(records)
}

#[tauri::command]
pub fn top_addresses_search(
    request: crate::top_addresses_deep::TopAddressesSearchRequest,
) -> Result<crate::top_addresses_deep::TopAddressesReport, String> {
    crate::top_addresses_deep::top_addresses_search_impl(request)
}

#[tauri::command]
pub async fn top_addresses_fetch_api(
    request: crate::top_addresses_deep::TopAddressesFetchRequest,
) -> Result<crate::top_addresses_deep::TopAddressesReport, String> {
    crate::top_addresses_deep::top_addresses_fetch_api_impl(request).await
}
