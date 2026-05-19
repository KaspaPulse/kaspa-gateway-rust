// ============================================================================
// KGW_OWNERSHIP_API_TRANSACTIONS_CLIENT_ONLY
// API layer owns endpoint construction, HTTP request/response handling, and raw API parsing.
// Forbidden: UI state, Tauri IPC ownership, and database persistence orchestration.
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFetchConfig {
    pub base_url: String,
    pub full_transactions_endpoint: String,
    pub page_limit: usize,
    pub max_pages: usize,
}

impl Default for TransactionFetchConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.kaspa.org".to_string(),
            full_transactions_endpoint:
                "/addresses/{kaspaAddress}/full-transactions?limit={limit}&offset={offset}&resolve_previous_outpoints=full"
                    .to_string(),
            page_limit: 500,
            max_pages: 10_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPage {
    pub page: usize,
    pub offset: usize,
    pub transactions: Vec<Value>,
    pub next_before: Option<i64>,
    pub next_after: Option<i64>,
}

pub fn transaction_id(raw: &Value) -> Option<String> {
    raw.get("transaction_id")
        .or_else(|| raw.get("txid"))
        .or_else(|| raw.get("id"))
        .or_else(|| raw.get("hash"))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

pub fn block_time_seconds(raw: &Value) -> i64 {
    let raw_time = raw
        .get("block_time")
        .or_else(|| raw.get("timestamp"))
        .and_then(|value| match value {
            Value::Number(number) => number.as_i64(),
            Value::String(text) => text.parse::<i64>().ok(),
            _ => None,
        })
        .unwrap_or_default();

    if raw_time > 10_000_000_000 {
        raw_time / 1000
    } else {
        raw_time
    }
}

pub fn build_transactions_url(
    config: &TransactionFetchConfig,
    address: &str,
    limit: usize,
    offset: usize,
) -> Result<Url, String> {
    let endpoint = config
        .full_transactions_endpoint
        .replace("{kaspaAddress}", address)
        .replace("{address}", address)
        .replace("{limit}", &limit.to_string())
        .replace("{offset}", &offset.to_string());

    Url::parse(&format!(
        "{}/{}",
        config.base_url.trim_end_matches('/'),
        endpoint.trim_start_matches('/')
    ))
    .map_err(|error| format!("invalid transaction API URL: {error}"))
}

pub async fn fetch_transactions_offset(
    client: &reqwest::Client,
    config: &TransactionFetchConfig,
    address: &str,
    limit: usize,
    offset: usize,
    page: usize,
) -> Result<TransactionPage, String> {
    let url = build_transactions_url(config, address, limit, offset)?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("transaction API request failed: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "transaction API returned HTTP status {}",
            response.status()
        ));
    }

    let transactions = response
        .json::<Vec<Value>>()
        .await
        .map_err(|error| format!("invalid transaction API JSON: {error}"))?;

    Ok(TransactionPage {
        page,
        offset,
        transactions,
        next_before: None,
        next_after: None,
    })
}

pub fn build_transactions_page_accepted_url(
    config: &TransactionFetchConfig,
    address: &str,
    limit: usize,
    before: i64,
    after: i64,
) -> Result<Url, String> {
    let encoded_address = address.replace(":", "%3A");

    let url = format!(
        "{}/addresses/{}/full-transactions-page?limit={}&before={}&after={}&resolve_previous_outpoints=full&acceptance=accepted",
        config.base_url.trim_end_matches('/'),
        encoded_address,
        limit.clamp(1, 500),
        before.max(0),
        after.max(0)
    );

    Url::parse(&url).map_err(|error| format!("invalid accepted transaction page API URL: {error}"))
}

fn parse_i64_header(headers: &reqwest::header::HeaderMap, name: &str) -> Option<i64> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<i64>().ok())
}

pub async fn fetch_transactions_page_accepted(
    client: &reqwest::Client,
    config: &TransactionFetchConfig,
    address: &str,
    limit: usize,
    before: i64,
    after: i64,
    page: usize,
) -> Result<TransactionPage, String> {
    let url = build_transactions_page_accepted_url(config, address, limit, before, after)?;

    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|error| format!("accepted transaction page API request failed: {error}"))?;

    let status = response.status();

    if !status.is_success() {
        let body = response
            .text()
            .await
            .unwrap_or_else(|error| format!("<failed to read error body: {error}>"));

        return Err(format!(
            "accepted transaction page API returned HTTP status {status} before={before} after={after} body={body}"
        ));
    }

    let next_before = parse_i64_header(response.headers(), "X-Next-Page-Before");
    let next_after = parse_i64_header(response.headers(), "X-Next-Page-After");

    let transactions = response
        .json::<Vec<Value>>()
        .await
        .map_err(|error| format!("invalid accepted transaction page API JSON: {error}"))?;

    Ok(TransactionPage {
        page,
        offset: before.max(0) as usize,
        transactions,
        next_before,
        next_after,
    })
}
