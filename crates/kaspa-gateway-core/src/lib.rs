use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GatewayError>;

pub const KASPA_MAINNET_PREFIX: &str = "kaspa:";
pub const KASPA_TESTNET_PREFIX: &str = "kaspatest:";
pub const KASPA_DEVNET_PREFIX: &str = "kaspadev:";
pub const KASPA_SIMNET_PREFIX: &str = "kaspasim:";

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("security error: {0}")]
    Security(String),

    #[error("api error: {0}")]
    Api(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("node error: {0}")]
    Node(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

impl AppInfo {
    pub fn new() -> Self {
        Self {
            name: "Kaspa Gateway".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct KaspaAddress(String);

impl KaspaAddress {
    pub fn parse(input: impl AsRef<str>) -> Result<Self> {
        let raw = input.as_ref().trim();

        if raw.is_empty() {
            return Err(GatewayError::Validation(
                "Kaspa address cannot be empty".to_string(),
            ));
        }

        if raw.chars().any(char::is_whitespace) {
            return Err(GatewayError::Validation(
                "Kaspa address cannot contain whitespace".to_string(),
            ));
        }

        let allowed_prefixes = [
            KASPA_MAINNET_PREFIX,
            KASPA_TESTNET_PREFIX,
            KASPA_DEVNET_PREFIX,
            KASPA_SIMNET_PREFIX,
        ];

        if !allowed_prefixes
            .iter()
            .any(|prefix| raw.starts_with(prefix))
        {
            return Err(GatewayError::Validation(
                "Kaspa address must start with kaspa:, kaspatest:, kaspadev:, or kaspasim:"
                    .to_string(),
            ));
        }

        let (_, payload) = raw
            .split_once(':')
            .ok_or_else(|| GatewayError::Validation("Kaspa address is missing ':'".to_string()))?;

        if payload.len() < 20 {
            return Err(GatewayError::Validation(
                "Kaspa address payload is too short".to_string(),
            ));
        }

        if payload.len() > 512 {
            return Err(GatewayError::Validation(
                "Kaspa address payload is too long".to_string(),
            ));
        }

        if !payload
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
        {
            return Err(GatewayError::Validation(
                "Kaspa address payload contains invalid characters".to_string(),
            ));
        }

        Ok(Self(raw.to_string()))
    }

    pub fn unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn masked(&self) -> String {
        if self.0.len() <= 18 {
            return "***".to_string();
        }

        let start = &self.0[..12];
        let end = &self.0[self.0.len() - 6..];
        format!("{start}...{end}")
    }

    pub fn network(&self) -> KaspaNetwork {
        if self.0.starts_with(KASPA_TESTNET_PREFIX) {
            KaspaNetwork::Testnet
        } else if self.0.starts_with(KASPA_DEVNET_PREFIX) {
            KaspaNetwork::Devnet
        } else if self.0.starts_with(KASPA_SIMNET_PREFIX) {
            KaspaNetwork::Simnet
        } else {
            KaspaNetwork::Mainnet
        }
    }

    pub fn payload(&self) -> &str {
        self.0
            .split_once(':')
            .map(|(_, payload)| payload)
            .unwrap_or(self.0.as_str())
    }
}

impl fmt::Display for KaspaAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for KaspaAddress {
    type Error = GatewayError;

    fn try_from(value: &str) -> Result<Self> {
        Self::parse(value)
    }
}

impl TryFrom<String> for KaspaAddress {
    type Error = GatewayError;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum KaspaNetwork {
    Mainnet,
    Testnet,
    Devnet,
    Simnet,
}

impl fmt::Display for KaspaNetwork {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mainnet => formatter.write_str("mainnet"),
            Self::Testnet => formatter.write_str("testnet"),
            Self::Devnet => formatter.write_str("devnet"),
            Self::Simnet => formatter.write_str("simnet"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneyAmount {
    pub amount: f64,
    pub currency: String,
}

impl MoneyAmount {
    pub const SOMPI_PER_KAS: i64 = 100_000_000;

    pub fn new(amount: f64, currency: impl Into<String>) -> Result<Self> {
        if !amount.is_finite() || amount < 0.0 {
            return Err(GatewayError::Validation(
                "money amount must be finite and non-negative".to_string(),
            ));
        }

        let currency = currency.into().trim().to_uppercase();

        if currency.len() < 3 || currency.len() > 8 {
            return Err(GatewayError::Validation(
                "currency code length is invalid".to_string(),
            ));
        }

        if !currency
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
        {
            return Err(GatewayError::Validation(
                "currency code contains invalid characters".to_string(),
            ));
        }

        Ok(Self { amount, currency })
    }

    pub fn kas(amount: f64) -> Result<Self> {
        Self::new(amount, "KAS")
    }

    pub fn from_kas(amount: f64) -> Result<Self> {
        Self::kas(amount)
    }

    pub fn from_sompi(sompi: i64) -> Result<Self> {
        if sompi < 0 {
            return Err(GatewayError::Validation(
                "sompi amount must be non-negative".to_string(),
            ));
        }

        Self::kas(sompi_to_kas(sompi))
    }

    pub fn zero(currency: impl Into<String>) -> Result<Self> {
        Self::new(0.0, currency)
    }

    pub fn as_kas(&self) -> Option<f64> {
        if self.currency == "KAS" {
            Some(self.amount)
        } else {
            None
        }
    }

    pub fn as_sompi(&self) -> Option<i64> {
        self.as_kas().map(kas_to_sompi)
    }

    pub fn formatted(&self) -> String {
        format!("{:.8} {}", self.amount, self.currency)
    }
}

impl fmt::Display for MoneyAmount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.formatted())
    }
}

pub fn normalize_address(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub fn validate_kaspa_address(value: &str) -> Result<()> {
    KaspaAddress::parse(value).map(|_| ())
}

pub fn is_kaspa_address_like(value: &str) -> bool {
    let normalized = normalize_address(value);

    normalized.starts_with(KASPA_MAINNET_PREFIX)
        || normalized.starts_with(KASPA_TESTNET_PREFIX)
        || normalized.starts_with(KASPA_DEVNET_PREFIX)
        || normalized.starts_with(KASPA_SIMNET_PREFIX)
}

pub fn mask_address(value: &str) -> String {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.len() <= 18 {
        return "***".to_string();
    }

    let start = &trimmed[..12];
    let end = &trimmed[trimmed.len() - 6..];
    format!("{start}...{end}")
}

pub fn sanitize_input_string(value: &str) -> String {
    value
        .chars()
        .filter(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(
                    ch,
                    ' ' | '_' | '-' | '.' | ':' | '/' | '\\' | '(' | ')' | '[' | ']'
                )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn sanitize_label(value: &str, fallback: &str) -> String {
    let sanitized = sanitize_input_string(value);

    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

pub fn sanitize_filename(value: &str, fallback: &str) -> String {
    let sanitized = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-' | '.' | '(' | ')'))
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();

    if sanitized.is_empty() {
        fallback.to_string()
    } else {
        sanitized
    }
}

pub fn sompi_to_kas(value: i64) -> f64 {
    value as f64 / 100_000_000.0
}

pub fn kas_to_sompi(value: f64) -> i64 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }

    (value * 100_000_000.0).round() as i64
}

pub fn format_kas(value: f64) -> String {
    format!("{value:.8}")
}

pub fn format_with_currency(value: f64, currency: &str) -> String {
    format!("{value:.4} {}", currency.trim().to_ascii_uppercase())
}

pub fn contains_unsafe_text(value: &str) -> bool {
    value.contains('\0')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains('\t')
        || value.contains('<')
        || value.contains('>')
        || value.contains('"')
        || value.contains('\'')
        || value.contains(';')
        || value.contains("--")
        || value.contains("/*")
        || value.contains("*/")
}

pub fn safe_truncate(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let mut output = String::new();

    for ch in value.chars().take(max_chars) {
        output.push(ch);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_is_available() {
        let info = AppInfo::default();

        assert_eq!(info.name, "Kaspa Gateway");
        assert!(!info.version.trim().is_empty());
    }

    #[test]
    fn kaspa_address_accepts_safe_prefixes() {
        let address = KaspaAddress::parse("kaspa:qwerty12345678901234").expect("address");

        assert_eq!(address.network(), KaspaNetwork::Mainnet);
        assert_eq!(address.as_str(), "kaspa:qwerty12345678901234");
    }

    #[test]
    fn kaspa_address_rejects_injection_text() {
        assert!(KaspaAddress::parse("kaspa:qxy' OR 1=1; --").is_err());
        assert!(KaspaAddress::parse("kaspa:<script>alert(1)</script>").is_err());
    }

    #[test]
    fn money_amount_contract_matches_existing_tests() {
        assert!(MoneyAmount::new(-1.0, "USD").is_err());

        let amount = MoneyAmount::new(1.25, "usd").expect("money");
        assert_eq!(amount.amount, 1.25);
        assert_eq!(amount.currency, "USD");
    }

    #[test]
    fn sompi_conversion_matches_kas_units() {
        assert_eq!(sompi_to_kas(100_000_000), 1.0);
        assert_eq!(kas_to_sompi(1.0), 100_000_000);
    }
}
