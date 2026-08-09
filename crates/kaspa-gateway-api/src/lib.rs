use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use url::Url;

pub const DEFAULT_KASPA_API_BASE_URL: &str = "https://api.kaspa.org";
pub const DEFAULT_COINGECKO_URL_TEMPLATE: &str =
    "https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies={supported_currencies}";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("invalid API configuration: {0}")]
    InvalidConfig(String),

    #[error("request failed: {0}")]
    RequestFailed(String),

    #[error("response parse failed: {0}")]
    ResponseParseFailed(String),

    #[error("invalid response data: {0}")]
    InvalidResponseData(String),
}

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub base_url: Url,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub backoff_base: Duration,
    pub endpoints: ApiEndpoints,
    pub coingecko_url_template: String,
    pub supported_currencies: Vec<String>,
}

impl ApiClientConfig {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())
            .map_err(|error| ApiError::InvalidUrl(error.to_string()))?;

        validate_base_url(&base_url)?;

        Ok(Self {
            base_url,
            timeout: Duration::from_secs(30),
            retry_attempts: 5,
            backoff_base: Duration::from_millis(500),
            endpoints: ApiEndpoints::default(),
            coingecko_url_template: DEFAULT_COINGECKO_URL_TEMPLATE.to_string(),
            supported_currencies: vec![
                "usd".to_string(),
                "sar".to_string(),
                "eur".to_string(),
                "gbp".to_string(),
            ],
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self> {
        if timeout.is_zero() {
            return Err(ApiError::InvalidConfig(
                "timeout must be greater than zero".to_string(),
            ));
        }

        self.timeout = timeout;
        Ok(self)
    }

    pub fn with_retry_attempts(mut self, attempts: u32) -> Result<Self> {
        if attempts == 0 {
            return Err(ApiError::InvalidConfig(
                "retry_attempts must be greater than zero".to_string(),
            ));
        }

        self.retry_attempts = attempts;
        Ok(self)
    }

    pub fn with_backoff_base(mut self, backoff: Duration) -> Result<Self> {
        if backoff.is_zero() {
            return Err(ApiError::InvalidConfig(
                "backoff_base must be greater than zero".to_string(),
            ));
        }

        self.backoff_base = backoff;
        Ok(self)
    }

    pub fn with_supported_currencies(mut self, currencies: Vec<String>) -> Result<Self> {
        if currencies.is_empty() {
            return Err(ApiError::InvalidConfig(
                "supported currencies cannot be empty".to_string(),
            ));
        }

        for currency in &currencies {
            validate_currency(currency)?;
        }

        self.supported_currencies = currencies
            .into_iter()
            .map(|currency| currency.to_ascii_lowercase())
            .collect();

        Ok(self)
    }
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self::new(DEFAULT_KASPA_API_BASE_URL).expect("default Kaspa API URL must be valid")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiEndpoints {
    pub balance: String,
    pub full_transactions: String,
    pub top_addresses: String,
    pub address_names: String,
    pub blockdag_info: String,
    pub blockreward: String,
    pub coinsupply: String,
    pub halving: String,
    pub hashrate: String,
    pub max_hashrate: String,
    pub network: String,
    pub kaspad: String,
}

impl Default for ApiEndpoints {
    fn default() -> Self {
        Self {
            balance: "/addresses/{kaspaAddress}/balance".to_string(),
            full_transactions: "/addresses/{kaspaAddress}/full-transactions?limit={limit}&offset={offset}&resolve_previous_outpoints=full".to_string(),
            top_addresses: "/addresses/top?limit=1".to_string(),
            address_names: "/addresses/names".to_string(),
            blockdag_info: "/info/blockdag".to_string(),
            blockreward: "/info/blockreward?stringOnly=false".to_string(),
            coinsupply: "/info/coinsupply".to_string(),
            halving: "/info/halving".to_string(),
            hashrate: "/info/hashrate".to_string(),
            max_hashrate: "/info/hashrate/max".to_string(),
            network: "/info/network".to_string(),
            kaspad: "/info/kaspad".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KaspaApiClient {
    config: ApiClientConfig,
    client: Client,
}

impl KaspaApiClient {
    pub fn new(config: ApiClientConfig) -> Result<Self> {
        validate_base_url(&config.base_url)?;

        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent("KaspaGateway-Rust/0.1.0")
            .https_only(true)
            .build()
            .map_err(|error| ApiError::RequestFailed(error.to_string()))?;

        Ok(Self { config, client })
    }

    pub fn default_client() -> Result<Self> {
        Self::new(ApiClientConfig::default())
    }

    pub fn config(&self) -> &ApiClientConfig {
        &self.config
    }

    pub fn build_url(&self, path: &str) -> Result<Url> {
        build_api_url(&self.config.base_url, path)
    }

    pub fn address_balance_url(&self, address: &str) -> Result<Url> {
        validate_address_for_url(address)?;

        let path = self
            .config
            .endpoints
            .balance
            .replace("{kaspaAddress}", address);

        self.build_url(&path)
    }

    pub fn address_full_transactions_url(
        &self,
        address: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Url> {
        validate_address_for_url(address)?;

        if limit == 0 || limit > 10_000 {
            return Err(ApiError::InvalidConfig(
                "transaction limit must be between 1 and 10000".to_string(),
            ));
        }

        let path = self
            .config
            .endpoints
            .full_transactions
            .replace("{kaspaAddress}", address)
            .replace("{limit}", &limit.to_string())
            .replace("{offset}", &offset.to_string());

        self.build_url(&path)
    }

    pub fn network_info_url(&self) -> Result<Url> {
        self.build_url(&self.config.endpoints.network)
    }

    pub fn blockdag_info_url(&self) -> Result<Url> {
        self.build_url(&self.config.endpoints.blockdag_info)
    }

    pub fn hashrate_url(&self) -> Result<Url> {
        self.build_url(&self.config.endpoints.hashrate)
    }

    pub fn top_addresses_url(&self) -> Result<Url> {
        self.build_url(&self.config.endpoints.top_addresses)
    }

    pub fn address_names_url(&self) -> Result<Url> {
        self.build_url(&self.config.endpoints.address_names)
    }

    pub fn coingecko_prices_url(&self) -> Result<Url> {
        let currencies = self.config.supported_currencies.join(",");

        let url = self
            .config
            .coingecko_url_template
            .replace("{supported_currencies}", &currencies);

        let parsed = Url::parse(&url).map_err(|error| ApiError::InvalidUrl(error.to_string()))?;
        validate_base_url(&parsed)?;
        Ok(parsed)
    }

    pub async fn get_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        self.get_json_url(url).await
    }

    pub async fn get_json_url<T>(&self, url: Url) -> Result<T>
    where
        T: DeserializeOwned,
    {
        validate_base_url(&url)?;

        let mut last_error: Option<ApiError> = None;

        for attempt in 1..=self.config.retry_attempts {
            let response = self.client.get(url.clone()).send().await;

            match response {
                Ok(response) => match response.error_for_status() {
                    Ok(success_response) => {
                        return success_response
                            .json::<T>()
                            .await
                            .map_err(|error| ApiError::ResponseParseFailed(error.to_string()));
                    }
                    Err(error) => {
                        last_error = Some(ApiError::RequestFailed(error.to_string()));
                    }
                },
                Err(error) => {
                    last_error = Some(ApiError::RequestFailed(error.to_string()));
                }
            }

            if attempt < self.config.retry_attempts {
                let multiplier = attempt.max(1);
                sleep(self.config.backoff_base * multiplier).await;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ApiError::RequestFailed("request failed without a captured error".to_string())
        }))
    }

    pub async fn fetch_address_balance_raw(&self, address: &str) -> Result<AddressBalanceRaw> {
        let url = self.address_balance_url(address)?;
        self.get_json_url(url).await
    }

    pub async fn fetch_address_balance_sompi(&self, address: &str) -> Result<u64> {
        self.fetch_address_balance_raw(address)
            .await?
            .balance_as_sompi()
    }

    pub async fn fetch_address_balance_kas(&self, address: &str) -> Result<f64> {
        self.fetch_address_balance_raw(address)
            .await?
            .balance_as_kas()
    }

    pub async fn fetch_full_transactions(
        &self,
        address: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Value> {
        let url = self.address_full_transactions_url(address, limit, offset)?;
        self.get_json_url(url).await
    }

    pub async fn fetch_network_info(&self) -> Result<NetworkInfo> {
        self.get_json(&self.config.endpoints.network).await
    }

    pub async fn fetch_blockdag_info(&self) -> Result<BlockDagInfo> {
        self.get_json(&self.config.endpoints.blockdag_info).await
    }

    pub async fn fetch_hashrate_info(&self) -> Result<HashrateInfo> {
        self.get_json(&self.config.endpoints.hashrate).await
    }

    pub async fn fetch_top_addresses(&self) -> Result<Value> {
        self.get_json(&self.config.endpoints.top_addresses).await
    }

    pub async fn fetch_address_names(&self) -> Result<Vec<AddressNameRecord>> {
        self.get_json(&self.config.endpoints.address_names).await
    }

    pub async fn fetch_kaspa_info(&self) -> Result<KaspaInfoBundle> {
        let network = self
            .get_json::<Value>(&self.config.endpoints.network)
            .await
            .ok();
        let kaspad = self
            .get_json::<Value>(&self.config.endpoints.kaspad)
            .await
            .ok();
        let blockdag = self
            .get_json::<Value>(&self.config.endpoints.blockdag_info)
            .await
            .ok();
        let coinsupply = self
            .get_json::<Value>(&self.config.endpoints.coinsupply)
            .await
            .ok();
        let halving = self
            .get_json::<Value>(&self.config.endpoints.halving)
            .await
            .ok();
        let hashrate = self
            .get_json::<Value>(&self.config.endpoints.hashrate)
            .await
            .ok();
        let blockreward = self
            .get_json::<Value>(&self.config.endpoints.blockreward)
            .await
            .ok();
        let maxhashrate = self
            .get_json::<Value>(&self.config.endpoints.max_hashrate)
            .await
            .ok();

        Ok(KaspaInfoBundle {
            network,
            kaspad,
            blockdag,
            coinsupply,
            halving,
            hashrate,
            blockreward,
            maxhashrate,
        })
    }

    pub async fn fetch_network_stats(&self) -> Result<NetworkStats> {
        let hashrate = self.fetch_hashrate_info().await.ok();
        let network = self.fetch_network_info().await.ok();

        Ok(NetworkStats {
            hashrate: hashrate
                .and_then(|value| value.hashrate)
                .map(|value| value / 1000.0),
            difficulty: network.and_then(|value| value.difficulty),
        })
    }

    pub async fn fetch_latest_release_info(&self, api_url: &str) -> Result<Value> {
        let url = Url::parse(api_url).map_err(|error| ApiError::InvalidUrl(error.to_string()))?;
        validate_base_url(&url)?;
        self.get_json_url(url).await
    }

    pub async fn get_kaspa_prices(&self) -> Result<BTreeMap<String, f64>> {
        let url = self.coingecko_prices_url()?;
        let raw: Value = self.get_json_url(url).await?;

        parse_coingecko_prices(&raw)
    }
}

pub fn build_api_url(base_url: &Url, path: &str) -> Result<Url> {
    validate_base_url(base_url)?;

    if path.trim().is_empty() {
        return Err(ApiError::InvalidUrl("path cannot be empty".to_string()));
    }

    if path.contains('\0')
        || path.contains('\n')
        || path.contains('\r')
        || path.contains('"')
        || path.contains('\\')
    {
        return Err(ApiError::InvalidUrl(
            "path contains unsafe characters".to_string(),
        ));
    }

    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("//") {
        return Err(ApiError::InvalidUrl(
            "absolute URL paths are not allowed".to_string(),
        ));
    }

    let normalized = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    base_url
        .join(&normalized)
        .map_err(|error| ApiError::InvalidUrl(error.to_string()))
}

pub fn validate_address_for_url(address: &str) -> Result<()> {
    let address = address.trim();

    if address.is_empty() {
        return Err(ApiError::InvalidUrl("address cannot be empty".to_string()));
    }

    if address.chars().any(char::is_whitespace) {
        return Err(ApiError::InvalidUrl(
            "address cannot contain whitespace".to_string(),
        ));
    }

    if address.contains('\0')
        || address.contains('/')
        || address.contains('?')
        || address.contains('#')
        || address.contains('&')
        || address.contains('=')
        || address.contains('\\')
        || address.contains('"')
        || address.contains('\'')
        || address.contains('<')
        || address.contains('>')
    {
        return Err(ApiError::InvalidUrl(
            "address contains unsafe URL characters".to_string(),
        ));
    }

    if !(address.starts_with("kaspa:")
        || address.starts_with("kaspatest:")
        || address.starts_with("kaspadev:")
        || address.starts_with("kaspasim:"))
    {
        return Err(ApiError::InvalidUrl(
            "address must start with a Kaspa prefix".to_string(),
        ));
    }

    Ok(())
}

fn validate_base_url(url: &Url) -> Result<()> {
    if url.scheme() != "https" {
        return Err(ApiError::InvalidConfig(
            "only HTTPS API URLs are allowed".to_string(),
        ));
    }

    if url.host_str().is_none() {
        return Err(ApiError::InvalidConfig(
            "API URL must include a host".to_string(),
        ));
    }

    Ok(())
}

fn validate_currency(value: &str) -> Result<()> {
    let value = value.trim();

    if value.len() < 2 || value.len() > 8 {
        return Err(ApiError::InvalidConfig(format!(
            "invalid currency code length: {value}"
        )));
    }

    if !value.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return Err(ApiError::InvalidConfig(format!(
            "invalid currency code: {value}"
        )));
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBalanceRaw {
    pub balance: Value,
}

impl AddressBalanceRaw {
    pub fn balance_as_sompi(&self) -> Result<u64> {
        match &self.balance {
            Value::Number(number) => number.as_u64().ok_or_else(|| {
                ApiError::InvalidResponseData("balance number must be unsigned".to_string())
            }),
            Value::String(value) => value.parse::<u64>().map_err(|error| {
                ApiError::InvalidResponseData(format!("balance string is invalid: {error}"))
            }),
            _ => Err(ApiError::InvalidResponseData(
                "balance must be a number or string".to_string(),
            )),
        }
    }

    pub fn balance_as_kas(&self) -> Result<f64> {
        Ok(self.balance_as_sompi()? as f64 / 100_000_000.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub name: Option<String>,
    pub network: Option<String>,
    pub difficulty: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDagInfo {
    pub block_count: Option<u64>,
    pub header_count: Option<u64>,
    pub tip_hashes: Option<Vec<String>>,
    pub virtual_daa_score: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashrateInfo {
    pub hashrate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressNameRecord {
    pub address: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub hashrate: Option<f64>,
    pub difficulty: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KaspaInfoBundle {
    pub network: Option<Value>,
    pub kaspad: Option<Value>,
    pub blockdag: Option<Value>,
    pub coinsupply: Option<Value>,
    pub halving: Option<Value>,
    pub hashrate: Option<Value>,
    pub blockreward: Option<Value>,
    pub maxhashrate: Option<Value>,
}

pub fn parse_coingecko_prices(raw: &Value) -> Result<BTreeMap<String, f64>> {
    let kaspa = raw.get("kaspa").and_then(Value::as_object).ok_or_else(|| {
        ApiError::InvalidResponseData(
            "Unexpected data format from CoinGecko: missing kaspa object".to_string(),
        )
    })?;

    let mut prices = BTreeMap::new();

    for (currency, value) in kaspa {
        let price = value.as_f64().ok_or_else(|| {
            ApiError::InvalidResponseData(format!("CoinGecko price for {currency} is not numeric"))
        })?;

        if price < 0.0 || !price.is_finite() {
            return Err(ApiError::InvalidResponseData(format!(
                "CoinGecko price for {currency} is invalid"
            )));
        }

        prices.insert(currency.to_ascii_lowercase(), price);
    }

    if prices.is_empty() {
        return Err(ApiError::InvalidResponseData(
            "CoinGecko returned no prices".to_string(),
        ));
    }

    Ok(prices)
}

pub fn sanitize_url_for_logging(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return "[Failed to sanitize URL]".to_string();
    };

    let sensitive_keys = [
        "key",
        "apikey",
        "api_key",
        "token",
        "secret",
        "auth",
        "password",
        "signature",
        "private",
        "pin",
    ];

    let query_pairs = parsed
        .query_pairs()
        .map(|(key, value)| {
            let lower = key.to_ascii_lowercase();
            let is_sensitive = sensitive_keys
                .iter()
                .any(|sensitive| lower.contains(sensitive));

            if is_sensitive {
                (key.to_string(), "***REDACTED***".to_string())
            } else {
                (key.to_string(), value.to_string())
            }
        })
        .collect::<Vec<_>>();

    parsed.set_query(None);

    if !query_pairs.is_empty() {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());

        for (key, value) in query_pairs {
            serializer.append_pair(&key, &value);
        }

        parsed.set_query(Some(&serializer.finish()));
    }

    parsed.to_string()
}

pub fn sanitize_data_for_logging(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut clean = Map::new();

            for (key, child) in map {
                let lower = key.to_ascii_lowercase();

                if [
                    "key",
                    "apikey",
                    "api_key",
                    "token",
                    "secret",
                    "auth",
                    "password",
                    "signature",
                    "private",
                    "pin",
                ]
                .iter()
                .any(|needle| lower.contains(needle))
                {
                    clean.insert(key.clone(), Value::String("***REDACTED***".to_string()));
                } else {
                    clean.insert(key.clone(), sanitize_data_for_logging(child));
                }
            }

            Value::Object(clean)
        }
        Value::Array(items) => Value::Array(items.iter().map(sanitize_data_for_logging).collect()),
        Value::String(text) => Value::String(text.replace(['\n', '\r'], " ")),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_config_is_https() {
        assert_eq!(ApiClientConfig::default().base_url.scheme(), "https");
    }

    #[test]
    fn coingecko_prices_parse() {
        let raw = json!({
            "kaspa": {
                "usd": 0.12,
                "sar": 0.45
            }
        });

        let prices = parse_coingecko_prices(&raw).expect("prices");
        assert_eq!(prices.get("usd"), Some(&0.12));
        assert_eq!(prices.get("sar"), Some(&0.45));
    }

    #[test]
    fn url_sanitizer_redacts_secrets() {
        let sanitized = sanitize_url_for_logging("https://example.com?a=1&token=secret");

        assert!(sanitized.contains("token=***REDACTED***"));
        assert!(!sanitized.contains("secret"));
    }
}

pub mod transactions;
