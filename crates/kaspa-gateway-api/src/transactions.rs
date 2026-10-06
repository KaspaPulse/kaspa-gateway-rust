// ============================================================================
// KGW_OWNERSHIP_API_TRANSACTIONS_CLIENT_ONLY
// API layer owns endpoint construction, HTTP request/response handling, and raw API parsing.
// Forbidden: UI state, Tauri IPC ownership, and database persistence orchestration.
// ============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFetchConfig {
    pub base_url: String,
    pub full_transactions_endpoint: String,
    pub page_limit: usize,
    pub max_pages: usize,
    pub max_retries: usize,
    pub retry_base_delay_ms: u64,
    pub retry_max_delay_ms: u64,
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
            max_retries: 3,
            retry_base_delay_ms: 250,
            retry_max_delay_ms: 10_000,
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

fn retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504)
}

fn retry_after_ms(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(|seconds| seconds.saturating_mul(1_000))
}

fn retry_backoff_ms(
    config: &TransactionFetchConfig,
    attempt: usize,
    address: &str,
    before: i64,
    after: i64,
) -> u64 {
    let max_delay = config
        .retry_max_delay_ms
        .max(config.retry_base_delay_ms)
        .max(1);
    let multiplier = 1_u64
        .checked_shl(u32::try_from(attempt.min(16)).unwrap_or(16))
        .unwrap_or(u64::MAX);
    let window = config
        .retry_base_delay_ms
        .max(1)
        .saturating_mul(multiplier)
        .min(max_delay);

    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    seed ^= before as u64;
    seed = seed.rotate_left(17) ^ after as u64;
    for byte in address.as_bytes() {
        seed = seed
            .wrapping_mul(1_099_511_628_211)
            .wrapping_add(u64::from(*byte));
    }

    1 + seed % window
}

async fn wait_before_retry(
    config: &TransactionFetchConfig,
    attempt: usize,
    address: &str,
    before: i64,
    after: i64,
    retry_after: Option<u64>,
) -> Result<(), String> {
    let max_delay = config
        .retry_max_delay_ms
        .max(config.retry_base_delay_ms)
        .max(1);

    let delay_ms = if let Some(retry_after) = retry_after {
        if retry_after > max_delay {
            return Err(format!(
                "transaction API Retry-After {}ms exceeds retry budget {}ms",
                retry_after, max_delay
            ));
        }
        retry_after.max(1)
    } else {
        retry_backoff_ms(config, attempt, address, before, after)
    };

    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    Ok(())
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

    for attempt in 0..=config.max_retries.min(8) {
        let response = match client
            .get(url.clone())
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                let retryable = error.is_timeout() || error.is_connect();

                if retryable && attempt < config.max_retries.min(8) {
                    wait_before_retry(config, attempt, address, before, after, None).await?;
                    continue;
                }

                return Err(format!(
                    "accepted transaction page API request failed attempt={} retryable={} before={} after={} error={}",
                    attempt + 1,
                    retryable,
                    before,
                    after,
                    error
                ));
            }
        };

        let status = response.status();

        if !status.is_success() {
            let retry_after = retry_after_ms(response.headers());
            let retryable = retryable_status(status);
            let body = response
                .text()
                .await
                .unwrap_or_else(|error| format!("<failed to read error body: {error}>"));

            if retryable && attempt < config.max_retries.min(8) {
                wait_before_retry(config, attempt, address, before, after, retry_after).await?;
                continue;
            }

            return Err(format!(
                "accepted transaction page API returned HTTP status {status} attempt={} retryable={} before={before} after={after} body={body}",
                attempt + 1,
                retryable
            ));
        }

        let next_before = parse_i64_header(response.headers(), "X-Next-Page-Before");
        let next_after = parse_i64_header(response.headers(), "X-Next-Page-After");

        let transactions = response
            .json::<Vec<Value>>()
            .await
            .map_err(|error| format!("invalid accepted transaction page API JSON: {error}"))?;

        return Ok(TransactionPage {
            page,
            offset: before.max(0) as usize,
            transactions,
            next_before,
            next_after,
        });
    }

    Err("accepted transaction page retry loop exhausted unexpectedly".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    fn scripted_http_server(
        responses: Vec<String>,
    ) -> (String, Arc<AtomicUsize>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let address = listener.local_addr().expect("server address");
        let count = Arc::new(AtomicUsize::new(0));
        let count_for_thread = Arc::clone(&count);

        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept test request");
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut request = [0_u8; 8192];
                let _ = stream.read(&mut request);
                count_for_thread.fetch_add(1, Ordering::SeqCst);
                stream
                    .write_all(response.as_bytes())
                    .expect("write scripted response");
                let _ = stream.flush();
            }
        });

        (format!("http://{address}"), count, handle)
    }

    fn response(status: &str, headers: &[(&str, &str)], body: &str) -> String {
        let mut value = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        );
        for (name, header_value) in headers {
            value.push_str(name);
            value.push_str(": ");
            value.push_str(header_value);
            value.push_str("\r\n");
        }
        value.push_str("\r\n");
        value.push_str(body);
        value
    }

    fn local_config(base_url: String) -> TransactionFetchConfig {
        TransactionFetchConfig {
            base_url,
            max_retries: 2,
            retry_base_delay_ms: 1,
            retry_max_delay_ms: 5,
            ..TransactionFetchConfig::default()
        }
    }

    #[test]
    fn retry_policy_only_retries_transient_statuses() {
        for status in [429_u16, 500, 502, 503, 504] {
            assert!(retryable_status(
                reqwest::StatusCode::from_u16(status).expect("valid status")
            ));
        }

        for status in [400_u16, 401, 403, 404, 409, 422, 501] {
            assert!(!retryable_status(
                reqwest::StatusCode::from_u16(status).expect("valid status")
            ));
        }
    }

    #[test]
    fn retry_after_and_backoff_are_bounded() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::RETRY_AFTER,
            reqwest::header::HeaderValue::from_static("3"),
        );
        assert_eq!(retry_after_ms(&headers), Some(3_000));

        let config = TransactionFetchConfig {
            retry_base_delay_ms: 10,
            retry_max_delay_ms: 100,
            ..TransactionFetchConfig::default()
        };

        for attempt in 0..8 {
            let delay = retry_backoff_ms(&config, attempt, "kaspa:test", 42, 0);
            assert!((1..=100).contains(&delay));
        }
    }

    #[tokio::test]
    async fn accepted_page_retries_429_then_parses_cursor() {
        let responses = vec![
            response(
                "429 Too Many Requests",
                &[("Retry-After", "0")],
                "rate limited",
            ),
            response(
                "200 OK",
                &[("X-Next-Page-Before", "123"), ("X-Next-Page-After", "456")],
                "[]",
            ),
        ];
        let (base_url, count, server) = scripted_http_server(responses);
        let config = local_config(base_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");

        let page = fetch_transactions_page_accepted(&client, &config, "kaspa:qptest", 25, 0, 0, 0)
            .await
            .expect("retry should recover");

        server.join().expect("server thread");
        assert_eq!(count.load(Ordering::SeqCst), 2);
        assert_eq!(page.next_before, Some(123));
        assert_eq!(page.next_after, Some(456));
        assert!(page.transactions.is_empty());
    }

    #[tokio::test]
    async fn accepted_page_fails_fast_on_non_retryable_status() {
        let (base_url, count, server) =
            scripted_http_server(vec![response("404 Not Found", &[], "missing")]);
        let config = local_config(base_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");

        let error = fetch_transactions_page_accepted(&client, &config, "kaspa:qptest", 25, 0, 0, 0)
            .await
            .expect_err("404 must fail without retry");

        server.join().expect("server thread");
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(error.contains("404"));
        assert!(error.contains("retryable=false"));
    }

    #[tokio::test]
    async fn accepted_page_rejects_malformed_json() {
        let (base_url, count, server) =
            scripted_http_server(vec![response("200 OK", &[], "{not-json")]);
        let config = local_config(base_url);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");

        let error = fetch_transactions_page_accepted(&client, &config, "kaspa:qptest", 25, 0, 0, 0)
            .await
            .expect_err("malformed JSON must fail");

        server.join().expect("server thread");
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(error.contains("invalid accepted transaction page API JSON"));
    }

    #[tokio::test]
    async fn retry_after_over_budget_fails_closed_without_sleeping_seconds() {
        let (base_url, count, server) = scripted_http_server(vec![response(
            "429 Too Many Requests",
            &[("Retry-After", "1")],
            "slow down",
        )]);
        let mut config = local_config(base_url);
        config.retry_max_delay_ms = 5;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("client");

        let error = fetch_transactions_page_accepted(&client, &config, "kaspa:qptest", 25, 0, 0, 0)
            .await
            .expect_err("Retry-After above budget must fail closed");

        server.join().expect("server thread");
        assert_eq!(count.load(Ordering::SeqCst), 1);
        assert!(error.contains("exceeds retry budget"));
    }
}
