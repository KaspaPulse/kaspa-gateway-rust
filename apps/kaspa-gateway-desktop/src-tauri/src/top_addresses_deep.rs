use kaspa_gateway_api::{ApiClientConfig, KaspaApiClient};
use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::BTreeMap;

const TOP_ADDRESSES_CACHE_KEY: &str = "top_addresses.cache.json";
const TOP_ADDRESSES_KNOWN_NAMES_KEY: &str = "top_addresses.known_names.json";
const TOP_ADDRESSES_RATES_KEY: &str = "top_addresses.currency_rates.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAddressesFetchRequest {
    pub limit: usize,
    pub force: bool,
    pub currency: String,
    pub kas_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAddressEntry {
    pub rank: usize,
    pub name: String,
    pub address: String,
    pub balance_sompi: i64,
    pub balance_kas: f64,
    pub currency: String,
    pub balance_currency: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAddressesReport {
    pub source: String,
    pub currency: String,
    pub kas_price: f64,
    pub total: usize,
    pub rows: Vec<TopAddressEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownNameRecord {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyRateRecord {
    pub currency: String,
    pub kas_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAddressesSearchRequest {
    pub query: String,
    pub sort_by: String,
    pub descending: bool,
    pub currency: String,
    pub kas_price: f64,
    pub limit: usize,
}
pub async fn top_addresses_fetch_api_impl(
    request: TopAddressesFetchRequest,
) -> Result<TopAddressesReport, String> {
    validate_currency(&request.currency)?;
    validate_price(request.kas_price)?;

    let limit = request.limit.clamp(1, 5_000);

    let manager = database_manager()?;
    let settings_repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    if !request.force
        && let Some(cached) = settings_repo
            .get(TOP_ADDRESSES_CACHE_KEY)
            .map_err(|error| error.to_string())?
        && let Ok(mut report) = serde_json::from_str::<TopAddressesReport>(&cached)
    {
        apply_currency(&mut report.rows, &request.currency, request.kas_price);
        apply_known_names(&mut report.rows, &load_known_names_map()?);
        sort_rows(&mut report.rows, "balance", true);
        report.rows.truncate(limit);
        rerank(&mut report.rows);

        report.currency = normalize_currency(&request.currency);
        report.kas_price = request.kas_price;
        report.source = "cache".to_string();
        report.total = report.rows.len();

        return Ok(report);
    }

    let api_config = ApiClientConfig::default();
    let endpoint = api_config.endpoints.top_addresses.clone();
    let client = KaspaApiClient::new(api_config).map_err(|error| error.to_string())?;

    let path = if endpoint.contains("limit=") {
        replace_or_append_limit(&endpoint, limit)
    } else {
        format!("{endpoint}?limit={limit}")
    };

    let raw = client.get_json::<Value>(&path).await;

    let mut warnings = Vec::new();

    let mut rows = match raw {
        Ok(value) => parse_api_top_addresses(&value, limit, &request.currency, request.kas_price),
        Err(error) => {
            warnings.push(format!("Top addresses API unavailable: {error}"));
            Vec::new()
        }
    };

    if rows.is_empty() {
        warnings.push(
            "Using local rich list fallback because API did not return usable top addresses."
                .to_string(),
        );
        rows = local_fallback_rows(limit, &request.currency, request.kas_price)?;
    }

    let known_names = load_known_names_map()?;
    apply_known_names(&mut rows, &known_names);
    sort_rows(&mut rows, "balance", true);
    rows.truncate(limit);
    rerank(&mut rows);

    let report = TopAddressesReport {
        source: if warnings.iter().any(|warning| warning.contains("fallback")) {
            "local_fallback".to_string()
        } else {
            "api".to_string()
        },
        currency: normalize_currency(&request.currency),
        kas_price: request.kas_price,
        total: rows.len(),
        rows,
        warnings,
    };

    settings_repo
        .set(
            TOP_ADDRESSES_CACHE_KEY,
            &serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    Ok(report)
}
pub fn top_addresses_search_impl(
    request: TopAddressesSearchRequest,
) -> Result<TopAddressesReport, String> {
    validate_currency(&request.currency)?;
    validate_price(request.kas_price)?;

    let manager = database_manager()?;
    let settings_repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let cached = settings_repo
        .get(TOP_ADDRESSES_CACHE_KEY)
        .map_err(|error| error.to_string())?;

    let mut report = if let Some(cached) = cached {
        serde_json::from_str::<TopAddressesReport>(&cached).map_err(|error| error.to_string())?
    } else {
        TopAddressesReport {
            source: "empty_cache".to_string(),
            currency: normalize_currency(&request.currency),
            kas_price: request.kas_price,
            total: 0,
            rows: Vec::new(),
            warnings: vec!["No cache found. Fetch top addresses first.".to_string()],
        }
    };

    let query = request.query.trim().to_ascii_lowercase();

    if !query.is_empty() {
        report.rows.retain(|row| {
            row.name.to_ascii_lowercase().contains(&query)
                || row.address.to_ascii_lowercase().contains(&query)
                || row.rank.to_string() == query
                || row.balance_kas.to_string().contains(&query)
        });
    }

    let known_names = load_known_names_map()?;
    apply_known_names(&mut report.rows, &known_names);
    apply_currency(&mut report.rows, &request.currency, request.kas_price);
    sort_rows(&mut report.rows, &request.sort_by, request.descending);

    report.rows.truncate(request.limit.clamp(1, 5_000));
    rerank(&mut report.rows);

    report.currency = normalize_currency(&request.currency);
    report.kas_price = request.kas_price;
    report.total = report.rows.len();

    Ok(report)
}
pub fn top_addresses_save_known_names_impl(
    records: Vec<KnownNameRecord>,
) -> Result<String, String> {
    let manager = database_manager()?;
    let settings_repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let mut clean = Vec::new();

    for record in records {
        let address = record.address.trim();
        let name = record.name.trim();

        if address.is_empty() || name.is_empty() {
            continue;
        }

        if KaspaAddress::parse(address).is_err() {
            continue;
        }

        clean.push(KnownNameRecord {
            address: address.to_string(),
            name: sanitize_name(name),
        });
    }

    clean.sort_by(|left, right| left.address.cmp(&right.address));
    clean.dedup_by(|left, right| left.address == right.address);

    settings_repo
        .set(
            TOP_ADDRESSES_KNOWN_NAMES_KEY,
            &serde_json::to_string_pretty(&clean).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    Ok(format!("Saved {} known names.", clean.len()))
}
pub fn top_addresses_load_known_names_impl() -> Result<Vec<KnownNameRecord>, String> {
    let manager = database_manager()?;
    let settings_repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let Some(value) = settings_repo
        .get(TOP_ADDRESSES_KNOWN_NAMES_KEY)
        .map_err(|error| error.to_string())?
    else {
        return Ok(Vec::new());
    };

    let mut records =
        serde_json::from_str::<Vec<KnownNameRecord>>(&value).map_err(|error| error.to_string())?;

    records.retain(|record| {
        !record.address.trim().is_empty()
            && !record.name.trim().is_empty()
            && KaspaAddress::parse(&record.address).is_ok()
    });

    records.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(records)
}
pub fn top_addresses_save_currency_rates_impl(
    records: Vec<CurrencyRateRecord>,
) -> Result<String, String> {
    let manager = database_manager()?;
    let settings_repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let mut clean = Vec::new();

    for record in records {
        validate_currency(&record.currency)?;

        if record.kas_price < 0.0 || !record.kas_price.is_finite() {
            continue;
        }

        clean.push(CurrencyRateRecord {
            currency: normalize_currency(&record.currency),
            kas_price: record.kas_price,
        });
    }

    clean.sort_by(|left, right| left.currency.cmp(&right.currency));
    clean.dedup_by(|left, right| left.currency == right.currency);

    settings_repo
        .set(
            TOP_ADDRESSES_RATES_KEY,
            &serde_json::to_string_pretty(&clean).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    Ok(format!("Saved {} currency rates.", clean.len()))
}
pub fn top_addresses_load_currency_rates_impl() -> Result<Vec<CurrencyRateRecord>, String> {
    let manager = database_manager()?;
    let settings_repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let Some(value) = settings_repo
        .get(TOP_ADDRESSES_RATES_KEY)
        .map_err(|error| error.to_string())?
    else {
        return Ok(default_currency_rates());
    };

    let mut records = serde_json::from_str::<Vec<CurrencyRateRecord>>(&value)
        .map_err(|error| error.to_string())?;

    records.retain(|record| {
        validate_currency(&record.currency).is_ok()
            && record.kas_price >= 0.0
            && record.kas_price.is_finite()
    });

    if records.is_empty() {
        Ok(default_currency_rates())
    } else {
        Ok(records)
    }
}

fn parse_api_top_addresses(
    value: &Value,
    limit: usize,
    currency: &str,
    kas_price: f64,
) -> Vec<TopAddressEntry> {
    let items = extract_rows_array(value);
    let mut rows = Vec::new();

    for (index, item) in items.into_iter().take(limit).enumerate() {
        let Some(address) = first_string(&item, &["address", "kaspaAddress", "kaspa_address"])
        else {
            continue;
        };

        if KaspaAddress::parse(&address).is_err() {
            continue;
        }

        let balance_sompi = first_i64(
            &item,
            &[
                "balance_sompi",
                "balanceSompi",
                "balance",
                "amount",
                "value",
                "utxoBalance",
            ],
        )
        .unwrap_or(0)
        .max(0);

        let balance_kas = balance_sompi as f64 / 100_000_000.0;
        let currency = normalize_currency(currency);

        rows.push(TopAddressEntry {
            rank: index + 1,
            name: first_string(&item, &["name", "label"]).unwrap_or_else(|| "Unknown".to_string()),
            address,
            balance_sompi,
            balance_kas,
            balance_currency: balance_kas * kas_price.max(0.0),
            currency,
            source: "api".to_string(),
        });
    }

    rows
}

fn extract_rows_array(value: &Value) -> Vec<Value> {
    if let Some(items) = value.as_array() {
        return items.clone();
    }

    for key in ["addresses", "items", "results", "data", "rows"] {
        if let Some(items) = value.get(key).and_then(Value::as_array) {
            return items.clone();
        }
    }

    Vec::new()
}

fn local_fallback_rows(
    limit: usize,
    currency: &str,
    kas_price: f64,
) -> Result<Vec<TopAddressEntry>, String> {
    let known_names = load_known_names_map().unwrap_or_default();
    let currency = normalize_currency(currency);

    let fallback = [
        (
            "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g",
            1_000_000_000_000_i64,
            "Known Address",
        ),
        (
            "kaspa:qwerty123456789012345678901234567890123456789012345678901234",
            500_000_000_000_i64,
            "Fallback Address 2",
        ),
        (
            "kaspa:qabcdef1234567890123456789012345678901234567890123456789012",
            250_000_000_000_i64,
            "Fallback Address 3",
        ),
    ];

    let mut rows = Vec::new();

    for (index, (address, balance_sompi, fallback_name)) in fallback.iter().take(limit).enumerate()
    {
        let balance_kas = *balance_sompi as f64 / 100_000_000.0;

        rows.push(TopAddressEntry {
            rank: index + 1,
            name: known_names
                .get(*address)
                .cloned()
                .unwrap_or_else(|| (*fallback_name).to_string()),
            address: (*address).to_string(),
            balance_sompi: *balance_sompi,
            balance_kas,
            currency: currency.clone(),
            balance_currency: balance_kas * kas_price.max(0.0),
            source: "local_fallback".to_string(),
        });
    }

    Ok(rows)
}

fn load_known_names_map() -> Result<BTreeMap<String, String>, String> {
    let records = top_addresses_load_known_names_impl().unwrap_or_default();

    Ok(records
        .into_iter()
        .map(|record| (record.address, record.name))
        .collect())
}

fn apply_known_names(rows: &mut [TopAddressEntry], names: &BTreeMap<String, String>) {
    for row in rows {
        if let Some(name) = names.get(&row.address) {
            row.name = name.clone();
        }
    }
}

fn apply_currency(rows: &mut [TopAddressEntry], currency: &str, kas_price: f64) {
    let currency = normalize_currency(currency);
    let safe_price = kas_price.max(0.0);

    for row in rows {
        row.currency = currency.clone();
        row.balance_currency = row.balance_kas * safe_price;
    }
}

fn sort_rows(rows: &mut [TopAddressEntry], sort_by: &str, descending: bool) {
    rows.sort_by(|left, right| {
        let ordering = match sort_by.trim().to_ascii_lowercase().as_str() {
            "rank" => left.rank.cmp(&right.rank),
            "name" => left
                .name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase()),
            "address" => left.address.cmp(&right.address),
            "currency" | "balance_currency" => left
                .balance_currency
                .partial_cmp(&right.balance_currency)
                .unwrap_or(Ordering::Equal),
            "balance" | "balance_kas" | "balance_sompi" => left
                .balance_kas
                .partial_cmp(&right.balance_kas)
                .unwrap_or(Ordering::Equal),
            _ => left.rank.cmp(&right.rank),
        };

        if descending {
            ordering.reverse()
        } else {
            ordering
        }
    });
}

fn rerank(rows: &mut [TopAddressEntry]) {
    for (index, row) in rows.iter_mut().enumerate() {
        row.rank = index + 1;
    }
}

fn replace_or_append_limit(endpoint: &str, limit: usize) -> String {
    if endpoint.contains("limit=") {
        let mut parts = endpoint.split('?');
        let base = parts.next().unwrap_or(endpoint);
        let query = parts.next().unwrap_or("");

        let mut pairs = Vec::new();
        let mut replaced = false;

        for pair in query.split('&').filter(|pair| !pair.trim().is_empty()) {
            if pair.starts_with("limit=") {
                pairs.push(format!("limit={limit}"));
                replaced = true;
            } else {
                pairs.push(pair.to_string());
            }
        }

        if !replaced {
            pairs.push(format!("limit={limit}"));
        }

        format!("{base}?{}", pairs.join("&"))
    } else if endpoint.contains('?') {
        format!("{endpoint}&limit={limit}")
    } else {
        format!("{endpoint}?limit={limit}")
    }
}

fn first_string(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(item) = value.get(*key) {
            if let Some(text) = item.as_str()
                && !text.trim().is_empty()
            {
                return Some(text.trim().to_string());
            }

            if let Some(number) = item.as_i64() {
                return Some(number.to_string());
            }

            if let Some(number) = item.as_u64() {
                return Some(number.to_string());
            }
        }
    }

    None
}

fn first_i64(value: &Value, keys: &[&str]) -> Option<i64> {
    for key in keys {
        if let Some(item) = value.get(*key) {
            if let Some(number) = item.as_i64() {
                return Some(number);
            }

            if let Some(number) = item.as_u64() {
                return i64::try_from(number).ok();
            }

            if let Some(number) = item.as_f64()
                && number.is_finite()
                && number >= 0.0
            {
                return Some(number.round() as i64);
            }

            if let Some(text) = item.as_str() {
                if let Ok(parsed) = text.parse::<i64>() {
                    return Some(parsed);
                }

                if let Ok(parsed) = text.parse::<f64>()
                    && parsed.is_finite()
                    && parsed >= 0.0
                {
                    return Some(parsed.round() as i64);
                }
            }
        }
    }

    None
}

fn validate_currency(value: &str) -> Result<(), String> {
    let value = value.trim();

    if value.len() < 2 || value.len() > 8 {
        return Err("Currency code length is invalid.".to_string());
    }

    if !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Err("Currency code must contain letters only.".to_string());
    }

    Ok(())
}

fn validate_price(value: f64) -> Result<(), String> {
    if !value.is_finite() || value < 0.0 {
        return Err("KAS price must be finite and non-negative.".to_string());
    }

    Ok(())
}

fn normalize_currency(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn sanitize_name(value: &str) -> String {
    let cleaned = value
        .chars()
        .filter(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, ' ' | '_' | '-' | '.' | ':' | '(' | ')' | '[' | ']')
        })
        .collect::<String>()
        .trim()
        .to_string();

    if cleaned.is_empty() {
        "Unknown".to_string()
    } else {
        cleaned
    }
}

fn default_currency_rates() -> Vec<CurrencyRateRecord> {
    vec![
        CurrencyRateRecord {
            currency: "KAS".to_string(),
            kas_price: 1.0,
        },
        CurrencyRateRecord {
            currency: "USD".to_string(),
            kas_price: 0.0,
        },
        CurrencyRateRecord {
            currency: "SAR".to_string(),
            kas_price: 0.0,
        },
        CurrencyRateRecord {
            currency: "EUR".to_string(),
            kas_price: 0.0,
        },
    ]
}

fn database_manager() -> Result<DatabaseManager, String> {
    let root = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("databases");

    let paths = DatabasePaths::new(root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths);

    manager
        .initialize_all()
        .map_err(|error| error.to_string())?;

    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_api_rows_from_array() {
        let raw = json!([
            {
                "address": "kaspa:qwerty123456789012345678901234567890123456789012345678901234",
                "balance": "100000000"
            }
        ]);

        let rows = parse_api_top_addresses(&raw, 10, "usd", 0.1);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].balance_kas, 1.0);
        assert_eq!(rows[0].balance_currency, 0.1);
    }

    #[test]
    fn search_sort_rerank_helpers_work() {
        let mut rows = vec![
            TopAddressEntry {
                rank: 1,
                name: "B".to_string(),
                address: "kaspa:qb1234567890123456789012345678901234567890123456789012345"
                    .to_string(),
                balance_sompi: 1,
                balance_kas: 1.0,
                currency: "USD".to_string(),
                balance_currency: 1.0,
                source: "test".to_string(),
            },
            TopAddressEntry {
                rank: 2,
                name: "A".to_string(),
                address: "kaspa:qa1234567890123456789012345678901234567890123456789012345"
                    .to_string(),
                balance_sompi: 2,
                balance_kas: 2.0,
                currency: "USD".to_string(),
                balance_currency: 2.0,
                source: "test".to_string(),
            },
        ];

        sort_rows(&mut rows, "name", false);
        rerank(&mut rows);

        assert_eq!(rows[0].name, "A");
        assert_eq!(rows[0].rank, 1);
    }

    #[test]
    fn currency_validation_rejects_bad_input() {
        assert!(validate_currency("USD").is_ok());
        assert!(validate_currency("US1").is_err());
        assert!(validate_price(-1.0).is_err());
    }
}
