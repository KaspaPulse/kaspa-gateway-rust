use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const APP_VERSION: &str = "1.0.0";
pub const APP_NAME: &str = "KaspaGateway";

pub const SUPPORTED_CURRENCIES: &[&str] = &[
    "usd", "sar", "eur", "gbp", "chf", "aud", "cad", "jpy", "krw", "rub", "cny", "try", "inr",
    "idr", "hkd", "sgd", "brl",
];

pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "en", "ar", "ru", "tr", "de", "es", "fr", "hi", "ja", "ko", "zh-CN", "id",
];

pub const SUPPORTED_TABS: &[&str] = &[
    "Explorer",
    "Kaspa Node",
    "Kaspa Bridge",
    "Analysis",
    "Top Addresses",
    "Log",
];

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not resolve user data directory")]
    UserDataDirectoryUnavailable,

    #[error("unsafe path rejected: {0}")]
    UnsafePath(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayPaths {
    pub data_root: PathBuf,
    pub database_dir: PathBuf,
    pub export_dir: PathBuf,
    pub log_dir: PathBuf,
    pub backup_dir: PathBuf,
}

impl GatewayPaths {
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        validate_user_path(&root)?;

        Ok(Self {
            database_dir: root.join("data"),
            export_dir: root.join("exports"),
            log_dir: root.join("logs"),
            backup_dir: root.join("backups"),
            data_root: root,
        })
    }

    pub fn ensure_all(&self) -> Result<()> {
        fs::create_dir_all(&self.data_root)?;
        fs::create_dir_all(&self.database_dir)?;
        fs::create_dir_all(&self.export_dir)?;
        fs::create_dir_all(&self.log_dir)?;
        fs::create_dir_all(&self.backup_dir)?;
        Ok(())
    }

    pub fn to_python_paths(&self) -> PathSettings {
        PathSettings {
            database: self.database_dir.clone(),
            export: self.export_dir.clone(),
            log: self.log_dir.clone(),
            backup: self.backup_dir.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathSettings {
    pub database: PathBuf,
    pub export: PathBuf,
    pub log: PathBuf,
    pub backup: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DbFilenames {
    pub transactions: String,
    pub addresses: String,
    pub app_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceSettings {
    pub timeout: u64,
    pub retry_attempts: u32,
    pub backoff_factor: f64,
    pub max_workers: usize,
    pub max_pages: usize,
    pub page_delay: f64,
    pub price_cache_hours: f64,
    pub network_cache_hours: f64,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplorerLinks {
    pub address: String,
    pub transaction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalApiSettings {
    pub coingecko: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiProfile {
    pub base_url: String,
    pub page_limit: u32,
    pub endpoints: BTreeMap<String, String>,
    pub explorer: ExplorerLinks,
    pub external: ExternalApiSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiSettings {
    pub active_profile: String,
    pub profiles: BTreeMap<String, ApiProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkSettings {
    pub donation: String,
    pub twitter: String,
    pub github: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisplaySettings {
    pub supported_currencies: Vec<String>,
    pub displayed_languages: Vec<String>,
    pub displayed_currencies: Vec<String>,
    pub displayed_tabs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KaspaBridgeSettings {
    pub enable_bridge_2: bool,

    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GatewayConfig {
    pub version: String,
    pub language: String,
    pub log_level: String,
    pub theme: String,
    pub selected_currency: String,
    pub table_font_size: u32,
    pub analysis_font_size: u32,
    pub check_for_updates: bool,
    pub autostart_on_windows: bool,
    pub paths: PathSettings,
    pub db_filenames: DbFilenames,
    pub performance: PerformanceSettings,
    pub api: ApiSettings,
    pub links: LinkSettings,
    pub display: DisplaySettings,

    #[serde(default)]
    pub kaspa_node: BTreeMap<String, Value>,

    #[serde(default = "default_kaspa_bridge_settings")]
    pub kaspa_bridge: KaspaBridgeSettings,

    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        let root = default_user_data_dir().unwrap_or_else(|_| PathBuf::from(".").join("user_data"));

        default_config_for_root(root)
    }
}

impl GatewayConfig {
    pub fn validate(&self) -> Result<()> {
        if self.version.trim().is_empty() {
            return Err(ConfigError::InvalidConfig(
                "version cannot be empty".to_string(),
            ));
        }

        validate_language(&self.language)?;
        validate_log_level(&self.log_level)?;

        if !SUPPORTED_CURRENCIES
            .iter()
            .any(|currency| currency.eq_ignore_ascii_case(&self.selected_currency))
        {
            return Err(ConfigError::InvalidConfig(format!(
                "unsupported selected currency: {}",
                self.selected_currency
            )));
        }

        validate_user_path(&self.paths.database)?;
        validate_user_path(&self.paths.export)?;
        validate_user_path(&self.paths.log)?;
        validate_user_path(&self.paths.backup)?;

        if self.performance.timeout == 0 {
            return Err(ConfigError::InvalidConfig(
                "timeout must be greater than zero".to_string(),
            ));
        }

        if self.performance.retry_attempts == 0 {
            return Err(ConfigError::InvalidConfig(
                "retry_attempts must be greater than zero".to_string(),
            ));
        }

        if self.performance.auto_refresh_interval_seconds < 15 {
            return Err(ConfigError::InvalidConfig(
                "auto refresh interval must be at least 15 seconds".to_string(),
            ));
        }

        if self.api.profiles.is_empty() {
            return Err(ConfigError::InvalidConfig(
                "at least one API profile is required".to_string(),
            ));
        }

        if !self.api.profiles.contains_key(&self.api.active_profile) {
            return Err(ConfigError::InvalidConfig(
                "active API profile does not exist".to_string(),
            ));
        }

        for (name, profile) in &self.api.profiles {
            validate_api_profile_name(name)?;
            profile.validate()?;
        }

        Ok(())
    }

    pub fn active_api_profile(&self) -> &ApiProfile {
        self.api
            .profiles
            .get(&self.api.active_profile)
            .or_else(|| self.api.profiles.get("Default"))
            .expect("GatewayConfig::validate guarantees at least one active profile")
    }

    pub fn active_api_profile_mut(&mut self) -> Option<&mut ApiProfile> {
        let active = self.api.active_profile.clone();
        self.api.profiles.get_mut(&active)
    }

    pub fn ensure_runtime_version(&mut self) {
        self.version = APP_VERSION.to_string();
    }

    pub fn merge_user_config(mut user: GatewayConfig, root: impl AsRef<Path>) -> Self {
        let defaults = default_config_for_root(root);

        if user.version.trim().is_empty() {
            user.version = defaults.version;
        }

        if user.language.trim().is_empty() {
            user.language = defaults.language;
        }

        if user.log_level.trim().is_empty() {
            user.log_level = defaults.log_level;
        }

        if user.theme.trim().is_empty() {
            user.theme = defaults.theme;
        }

        if user.selected_currency.trim().is_empty() {
            user.selected_currency = defaults.selected_currency;
        }

        if user.display.supported_currencies.is_empty() {
            user.display.supported_currencies = defaults.display.supported_currencies;
        }

        if user.display.displayed_languages.is_empty() {
            user.display.displayed_languages = defaults.display.displayed_languages;
        }

        if user.display.displayed_currencies.is_empty() {
            user.display.displayed_currencies = defaults.display.displayed_currencies;
        }

        if user.display.displayed_tabs.is_empty() {
            user.display.displayed_tabs = defaults.display.displayed_tabs;
        }

        for (name, profile) in defaults.api.profiles {
            user.api.profiles.entry(name).or_insert(profile);
        }

        if !user.api.profiles.contains_key(&user.api.active_profile) {
            user.api.active_profile = "Default".to_string();
        }

        user.version = APP_VERSION.to_string();
        user
    }
}

impl ApiProfile {
    pub fn validate(&self) -> Result<()> {
        validate_base_url(&self.base_url)?;

        if self.page_limit == 0 || self.page_limit > 10_000 {
            return Err(ConfigError::InvalidConfig(
                "page_limit must be between 1 and 10000".to_string(),
            ));
        }

        if self.endpoints.is_empty() {
            return Err(ConfigError::InvalidConfig(
                "API profile must contain endpoints".to_string(),
            ));
        }

        for (key, endpoint) in &self.endpoints {
            validate_config_key(key)?;
            validate_endpoint_path(endpoint)?;
        }

        validate_url_template(&self.explorer.address)?;
        validate_url_template(&self.explorer.transaction)?;
        validate_url_template(&self.external.coingecko)?;

        Ok(())
    }

    pub fn endpoint_url(&self, key: &str) -> Option<String> {
        self.endpoints
            .get(key)
            .map(|endpoint| format!("{}{}", self.base_url.trim_end_matches('/'), endpoint))
    }
}

pub fn default_user_data_dir() -> Result<PathBuf> {
    let base = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("APPDATA"))
        .ok_or(ConfigError::UserDataDirectoryUnavailable)?;

    Ok(PathBuf::from(base).join(APP_NAME))
}

pub fn get_user_data_root(custom_path: Option<impl AsRef<Path>>) -> Result<PathBuf> {
    let default_root = default_user_data_dir()?;

    let path = match custom_path {
        Some(custom) => {
            let candidate = custom.as_ref();

            if candidate.as_os_str().is_empty() {
                default_root
            } else {
                validate_user_path(candidate)?;

                if candidate.is_absolute() {
                    candidate.to_path_buf()
                } else {
                    default_root.join(candidate)
                }
            }
        }
        None => default_root,
    };

    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn default_config_path() -> Result<PathBuf> {
    Ok(default_user_data_dir()?.join("config.json"))
}

pub fn get_project_root() -> Result<PathBuf> {
    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        return Ok(parent.to_path_buf());
    }

    env::current_dir().map_err(ConfigError::Io)
}

pub fn get_assets_path(relative_path: impl AsRef<Path>) -> Result<PathBuf> {
    Ok(get_project_root()?.join("assets").join(relative_path))
}

pub fn validate_user_path(path: &Path) -> Result<()> {
    let path_string = path.to_string_lossy();

    if path_string.trim().is_empty() {
        return Err(ConfigError::UnsafePath("empty path".to_string()));
    }

    if path_string.contains('"')
        || path_string.contains('\0')
        || path_string.contains('\n')
        || path_string.contains('\r')
    {
        return Err(ConfigError::UnsafePath(path_string.to_string()));
    }

    Ok(())
}

pub fn load_or_create_config(path: impl AsRef<Path>) -> Result<GatewayConfig> {
    let path = path.as_ref();

    let root = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    GatewayPaths::from_root(&root)?.ensure_all()?;

    if !path.exists() {
        let mut config = default_config_for_root(&root);
        config.ensure_runtime_version();
        save_config(path, &config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(path)?;

    let mut value: Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(_) => {
            let mut config = default_config_for_root(&root);
            config.ensure_runtime_version();
            save_config(path, &config)?;
            return Ok(config);
        }
    };

    migrate_config_value(&mut value);

    let user_config: GatewayConfig = match serde_json::from_value(value) {
        Ok(config) => config,
        Err(_) => {
            let mut config = default_config_for_root(&root);
            config.ensure_runtime_version();
            save_config(path, &config)?;
            return Ok(config);
        }
    };

    let mut final_config = GatewayConfig::merge_user_config(user_config, &root);
    final_config.validate()?;

    if final_config.version != APP_VERSION {
        final_config.ensure_runtime_version();
        save_config(path, &final_config)?;
    }

    Ok(final_config)
}

pub fn load_default_config() -> Result<GatewayConfig> {
    load_or_create_config(default_config_path()?)
}

pub fn save_config(path: impl AsRef<Path>, config: &GatewayConfig) -> Result<()> {
    let path = path.as_ref();
    let mut config = config.clone();
    config.ensure_runtime_version();
    config.validate()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    GatewayPaths::from_root(path.parent().unwrap_or_else(|| Path::new(".")))?.ensure_all()?;

    let temp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(&config)?;

    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, path)?;

    Ok(())
}

pub fn get_active_api_config(config: &GatewayConfig) -> &ApiProfile {
    config.active_api_profile()
}

pub fn default_config_for_root(root: impl AsRef<Path>) -> GatewayConfig {
    let paths = GatewayPaths::from_root(root.as_ref())
        .unwrap_or_else(|_| GatewayPaths {
            data_root: root.as_ref().to_path_buf(),
            database_dir: root.as_ref().join("data"),
            export_dir: root.as_ref().join("exports"),
            log_dir: root.as_ref().join("logs"),
            backup_dir: root.as_ref().join("backups"),
        })
        .to_python_paths();

    GatewayConfig {
        version: APP_VERSION.to_string(),
        language: "en".to_string(),
        log_level: "INFO".to_string(),
        theme: "superhero".to_string(),
        selected_currency: "USD".to_string(),
        table_font_size: 9,
        analysis_font_size: 9,
        check_for_updates: true,
        autostart_on_windows: false,
        paths,
        db_filenames: DbFilenames {
            transactions: "Transactions.duckdb".to_string(),
            addresses: "Addresses.duckdb".to_string(),
            app_data: "AppData.duckdb".to_string(),
        },
        performance: PerformanceSettings {
            timeout: 30,
            retry_attempts: 5,
            backoff_factor: 0.5,
            max_workers: 10,
            max_pages: 10_000,
            page_delay: 0.05,
            price_cache_hours: 0.25,
            network_cache_hours: 0.25,
            auto_refresh_enabled: false,
            auto_refresh_interval_seconds: 60,
        },
        api: ApiSettings {
            active_profile: "Default".to_string(),
            profiles: BTreeMap::from([("Default".to_string(), default_api_profile())]),
        },
        links: LinkSettings {
            donation: "https://explorer.kaspa.org/addresses/kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g".to_string(),
            twitter: "https://x.com/KaspaPulse".to_string(),
            github: "https://github.com/KaspaPulse/KaspaGateway".to_string(),
        },
        display: DisplaySettings {
            supported_currencies: SUPPORTED_CURRENCIES.iter().map(|v| (*v).to_string()).collect(),
            displayed_languages: SUPPORTED_LANGUAGES.iter().map(|v| (*v).to_string()).collect(),
            displayed_currencies: SUPPORTED_CURRENCIES.iter().map(|v| (*v).to_string()).collect(),
            displayed_tabs: SUPPORTED_TABS.iter().map(|v| (*v).to_string()).collect(),
        },
        kaspa_node: BTreeMap::new(),
        kaspa_bridge: default_kaspa_bridge_settings(),
        extra: BTreeMap::new(),
    }
}

pub fn default_api_profile() -> ApiProfile {
    ApiProfile {
        base_url: "https://api.kaspa.org".to_string(),
        page_limit: 500,
        endpoints: BTreeMap::from([
            ("balance".to_string(), "/addresses/{kaspaAddress}/balance".to_string()),
            (
                "full_transactions".to_string(),
                "/addresses/{kaspaAddress}/full-transactions?limit={limit}&offset={offset}&resolve_previous_outpoints=full".to_string(),
            ),
            ("top_addresses".to_string(), "/addresses/top?limit=1".to_string()),
            ("address_names".to_string(), "/addresses/names".to_string()),
            ("blockdag_info".to_string(), "/info/blockdag".to_string()),
            ("blockreward".to_string(), "/info/blockreward?stringOnly=false".to_string()),
            ("coinsupply".to_string(), "/info/coinsupply".to_string()),
            ("halving".to_string(), "/info/halving".to_string()),
            ("hashrate".to_string(), "/info/hashrate".to_string()),
            ("max_hashrate".to_string(), "/info/hashrate/max".to_string()),
            ("network".to_string(), "/info/network".to_string()),
            ("kaspad".to_string(), "/info/kaspad".to_string()),
        ]),
        explorer: ExplorerLinks {
            address: "https://explorer.kaspa.org/addresses/{kaspaAddress}".to_string(),
            transaction: "https://explorer.kaspa.org/txs/{txid}".to_string(),
        },
        external: ExternalApiSettings {
            coingecko: "https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies={supported_currencies}".to_string(),
            api_key: String::new(),
        },
    }
}

fn default_kaspa_bridge_settings() -> KaspaBridgeSettings {
    KaspaBridgeSettings {
        enable_bridge_2: false,
        extra: BTreeMap::new(),
    }
}

fn migrate_config_value(value: &mut Value) {
    decrypt_legacy_api_keys(value);

    let Some(root) = value.as_object_mut() else {
        return;
    };

    if !root.contains_key("api") {
        root.insert(
            "api".to_string(),
            api_settings_to_value(default_api_settings()),
        );
        return;
    }

    let Some(api) = root.get_mut("api") else {
        return;
    };

    let Some(api_obj) = api.as_object_mut() else {
        *api = api_settings_to_value(default_api_settings());
        return;
    };

    if !api_obj.contains_key("profiles") {
        let old_api = Value::Object(api_obj.clone());
        let migrated_profile = migrate_old_api_profile(&old_api);

        let mut profiles = Map::new();
        profiles.insert(
            "Default".to_string(),
            serde_json::to_value(migrated_profile).unwrap(),
        );

        let mut new_api = Map::new();
        new_api.insert(
            "active_profile".to_string(),
            Value::String("Default".to_string()),
        );
        new_api.insert("profiles".to_string(), Value::Object(profiles));

        *api = Value::Object(new_api);
    }
}

fn decrypt_legacy_api_keys(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                if key == "api_key" {
                    if let Value::String(text) = child
                        && text.starts_with("keyring_managed:")
                    {
                        *text = String::new();
                    }
                } else {
                    decrypt_legacy_api_keys(child);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                decrypt_legacy_api_keys(item);
            }
        }
        _ => {}
    }
}

fn migrate_old_api_profile(old_api: &Value) -> ApiProfile {
    let mut profile = default_api_profile();

    if let Some(base_url) = old_api.get("base_url").and_then(Value::as_str) {
        profile.base_url = base_url.to_string();
    }

    if let Some(page_limit) = old_api.get("page_limit").and_then(Value::as_u64) {
        profile.page_limit = u32::try_from(page_limit).unwrap_or(profile.page_limit);
    }

    if let Some(endpoints) = old_api.get("endpoints").and_then(Value::as_object) {
        for (key, value) in endpoints {
            if let Some(endpoint) = value.as_str() {
                profile.endpoints.insert(key.clone(), endpoint.to_string());
            }
        }
    }

    if let Some(explorer) = old_api.get("explorer").and_then(Value::as_object) {
        if let Some(address) = explorer.get("address").and_then(Value::as_str) {
            profile.explorer.address = address.to_string();
        }
        if let Some(transaction) = explorer.get("transaction").and_then(Value::as_str) {
            profile.explorer.transaction = transaction.to_string();
        }
    }

    if let Some(external) = old_api.get("external").and_then(Value::as_object) {
        if let Some(coingecko) = external.get("coingecko").and_then(Value::as_str) {
            profile.external.coingecko = coingecko.to_string();
        }
        if let Some(api_key) = external.get("api_key").and_then(Value::as_str) {
            profile.external.api_key = if api_key.starts_with("keyring_managed:") {
                String::new()
            } else {
                api_key.to_string()
            };
        }
    }

    profile
}

fn default_api_settings() -> ApiSettings {
    ApiSettings {
        active_profile: "Default".to_string(),
        profiles: BTreeMap::from([("Default".to_string(), default_api_profile())]),
    }
}

fn api_settings_to_value(settings: ApiSettings) -> Value {
    serde_json::to_value(settings).unwrap_or(Value::Object(Map::new()))
}

fn validate_language(value: &str) -> Result<()> {
    if SUPPORTED_LANGUAGES.iter().any(|lang| lang == &value) {
        Ok(())
    } else {
        Err(ConfigError::InvalidConfig(format!(
            "unsupported language: {value}"
        )))
    }
}

fn validate_log_level(value: &str) -> Result<()> {
    match value.trim().to_ascii_uppercase().as_str() {
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" => Ok(()),
        _ => Err(ConfigError::InvalidConfig(format!(
            "unsupported log level: {value}"
        ))),
    }
}

fn validate_api_profile_name(value: &str) -> Result<()> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(ConfigError::InvalidConfig(
            "API profile name cannot be empty".to_string(),
        ));
    }

    if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(ConfigError::InvalidConfig(
            "API profile name contains unsafe characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_config_key(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ConfigError::InvalidConfig(
            "configuration key cannot be empty".to_string(),
        ));
    }

    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(ConfigError::InvalidConfig(format!(
            "configuration key contains unsafe characters: {value}"
        )));
    }

    Ok(())
}

fn validate_base_url(value: &str) -> Result<()> {
    let value = value.trim();

    if !(value.starts_with("https://") || value.starts_with("http://")) {
        return Err(ConfigError::InvalidConfig(format!(
            "base_url must start with http:// or https://: {value}"
        )));
    }

    if value.contains('\0') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        return Err(ConfigError::InvalidConfig(
            "base_url contains unsafe characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_endpoint_path(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ConfigError::InvalidConfig(
            "endpoint path cannot be empty".to_string(),
        ));
    }

    if value.contains('\0') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        return Err(ConfigError::InvalidConfig(
            "endpoint path contains unsafe characters".to_string(),
        ));
    }

    Ok(())
}

fn validate_url_template(value: &str) -> Result<()> {
    if !(value.starts_with("https://") || value.starts_with("http://")) {
        return Err(ConfigError::InvalidConfig(format!(
            "URL template must start with http:// or https://: {value}"
        )));
    }

    if value.contains('\0') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        return Err(ConfigError::InvalidConfig(
            "URL template contains unsafe characters".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = GatewayConfig::default();
        config.validate().expect("default config must be valid");
    }

    #[test]
    fn default_api_profile_contains_python_endpoints() {
        let profile = default_api_profile();

        assert_eq!(profile.base_url, "https://api.kaspa.org");
        assert!(profile.endpoints.contains_key("balance"));
        assert!(profile.endpoints.contains_key("full_transactions"));
        assert!(profile.endpoints.contains_key("top_addresses"));
        assert!(profile.endpoints.contains_key("network"));
        assert!(profile.external.coingecko.contains("coingecko.com"));
    }

    #[test]
    fn old_python_api_config_migrates_to_profiles() {
        let mut value = serde_json::json!({
            "api": {
                "base_url": "https://example.invalid",
                "page_limit": 250,
                "endpoints": {
                    "balance": "/addresses/{kaspaAddress}/balance"
                },
                "explorer": {
                    "address": "https://explorer.example/addresses/{kaspaAddress}",
                    "transaction": "https://explorer.example/txs/{txid}"
                },
                "external": {
                    "coingecko": "https://api.example/price",
                    "api_key": "keyring_managed:service:user"
                }
            }
        });

        migrate_config_value(&mut value);

        let api = value.get("api").expect("api");
        assert_eq!(
            api.get("active_profile").and_then(Value::as_str),
            Some("Default")
        );

        let profile = api
            .get("profiles")
            .and_then(|v| v.get("Default"))
            .expect("Default profile");

        assert_eq!(
            profile.get("base_url").and_then(Value::as_str),
            Some("https://example.invalid")
        );

        assert_eq!(
            profile
                .get("external")
                .and_then(|v| v.get("api_key"))
                .and_then(Value::as_str),
            Some("")
        );
    }
}
