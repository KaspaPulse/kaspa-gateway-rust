use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const REAL_CONFIG_KEY: &str = "real_config.migrated.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealApiProfile {
    pub name: String,
    pub base_url: String,
    pub endpoints: BTreeMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealConfig {
    pub version: u32,
    pub language: String,
    pub theme: String,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_seconds: u64,
    pub retry_attempts: u8,
    pub timeout_seconds: u64,
    pub backoff_factor: f64,
    pub displayed_tabs: Vec<String>,
    pub displayed_currencies: Vec<String>,
    pub displayed_languages: Vec<String>,
    pub active_api_profile: String,
    pub api_profiles: Vec<RealApiProfile>,
    pub paths: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealConfigMigrationReport {
    pub loaded_from: String,
    pub saved_key: String,
    pub config: RealConfig,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn config_default() -> Result<RealConfig, String> {
    default_real_config()
}

#[tauri::command]
pub fn config_load() -> Result<RealConfig, String> {
    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let Some(value) = repo
        .get(REAL_CONFIG_KEY)
        .map_err(|error| error.to_string())?
    else {
        return default_real_config();
    };

    let mut config =
        serde_json::from_str::<RealConfig>(&value).map_err(|error| error.to_string())?;
    normalize_config(&mut config)?;
    validate_config(&config)?;

    Ok(config)
}

#[tauri::command]
pub fn config_save(config: RealConfig) -> Result<RealConfigMigrationReport, String> {
    let mut config = config;
    normalize_config(&mut config)?;
    validate_config(&config)?;

    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repo.set(
        REAL_CONFIG_KEY,
        &serde_json::to_string_pretty(&config).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(RealConfigMigrationReport {
        loaded_from: "ui".to_string(),
        saved_key: REAL_CONFIG_KEY.to_string(),
        config,
        warnings: Vec::new(),
    })
}

#[tauri::command]
pub fn config_import_python_config(path: String) -> Result<RealConfigMigrationReport, String> {
    let path = safe_existing_file(&path)?;

    let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let json: Value = serde_json::from_str(&text)
        .map_err(|error| format!("Python config must be JSON for this importer stage: {error}"))?;

    let mut config = default_real_config()?;
    let mut warnings = Vec::new();

    if let Some(language) = find_string(&json, &["display.language", "ui.language", "language"]) {
        config.language = normalize_language(&language);
    }

    if let Some(theme) = find_string(&json, &["display.theme", "ui.theme", "theme"]) {
        config.theme = normalize_theme(&theme);
    }

    if let Some(enabled) = find_bool(
        &json,
        &[
            "display.auto_refresh",
            "display.auto_refresh_enabled",
            "auto_refresh_enabled",
            "auto_refresh",
        ],
    ) {
        config.auto_refresh_enabled = enabled;
    }

    if let Some(seconds) = find_u64(
        &json,
        &[
            "display.auto_refresh_seconds",
            "display.auto_refresh_interval",
            "auto_refresh_seconds",
            "auto_refresh_interval_seconds",
        ],
    ) {
        config.auto_refresh_seconds = seconds.clamp(5, 86_400);
    }

    if let Some(retry) = find_u64(&json, &["performance.retry_attempts", "retry_attempts"]) {
        config.retry_attempts = u8::try_from(retry.clamp(1, 10)).unwrap_or(3);
    }

    if let Some(timeout) = find_u64(
        &json,
        &[
            "performance.timeout",
            "performance.timeout_seconds",
            "timeout_seconds",
        ],
    ) {
        config.timeout_seconds = timeout.clamp(1, 300);
    }

    if let Some(backoff) = find_f64(&json, &["performance.backoff_factor", "backoff_factor"]) {
        config.backoff_factor = backoff.clamp(0.1, 10.0);
    }

    if let Some(currencies) = find_string_array(
        &json,
        &[
            "display.supported_currencies",
            "displayed_currencies",
            "supported_currencies",
        ],
    ) {
        config.displayed_currencies = normalize_currency_codes(currencies);
    }

    if let Some(languages) = find_string_array(
        &json,
        &[
            "display.supported_languages",
            "displayed_languages",
            "supported_languages",
        ],
    ) {
        config.displayed_languages = normalize_languages(languages);
    }

    if let Some(tabs) =
        find_string_array(&json, &["display.displayed_tabs", "displayed_tabs", "tabs"])
    {
        config.displayed_tabs = normalize_tabs(tabs);
    }

    migrate_paths(&json, &mut config.paths, &mut warnings);

    if let Some(profiles) = json
        .pointer("/api/profiles")
        .or_else(|| json.get("api_profiles"))
    {
        match parse_api_profiles(profiles) {
            Ok(imported_profiles) if !imported_profiles.is_empty() => {
                config.active_api_profile = imported_profiles[0].name.clone();
                config.api_profiles = imported_profiles;
            }
            Ok(_) => warnings
                .push("No valid Python API profiles found; default profile retained.".to_string()),
            Err(error) => warnings.push(format!("API profile migration warning: {error}")),
        }
    } else if let Some(api) = json.get("api") {
        match parse_single_profile(api, "Default") {
            Ok(profile) => {
                config.active_api_profile = profile.name.clone();
                config.api_profiles = vec![profile];
            }
            Err(error) => warnings.push(format!("Legacy API migration warning: {error}")),
        }
    } else {
        warnings.push("No Python API profiles found; default profile retained.".to_string());
    }

    normalize_config(&mut config)?;
    validate_config(&config)?;

    let saved = config_save(config.clone())?;

    Ok(RealConfigMigrationReport {
        loaded_from: path.display().to_string(),
        saved_key: saved.saved_key,
        config,
        warnings,
    })
}

fn default_real_config() -> Result<RealConfig, String> {
    let root = default_user_data_dir().map_err(|error| error.to_string())?;

    let mut paths = BTreeMap::new();
    paths.insert(
        "database".to_string(),
        root.join("databases").display().to_string(),
    );
    paths.insert("logs".to_string(), root.join("logs").display().to_string());
    paths.insert(
        "exports".to_string(),
        root.join("exports").display().to_string(),
    );
    paths.insert(
        "backups".to_string(),
        root.join("backups").display().to_string(),
    );
    paths.insert("node".to_string(), root.join("node").display().to_string());
    paths.insert(
        "bridge".to_string(),
        root.join("bridge").display().to_string(),
    );

    Ok(RealConfig {
        version: 1,
        language: "English".to_string(),
        theme: "Dark".to_string(),
        auto_refresh_enabled: false,
        auto_refresh_seconds: 60,
        retry_attempts: 5,
        timeout_seconds: 30,
        backoff_factor: 0.5,
        displayed_tabs: default_tabs(),
        displayed_currencies: default_currencies(),
        displayed_languages: default_languages(),
        active_api_profile: "Default".to_string(),
        api_profiles: vec![RealApiProfile {
            name: "Default".to_string(),
            base_url: "https://api.kaspa.org".to_string(),
            endpoints: default_endpoints(),
            enabled: true,
        }],
        paths,
    })
}

fn default_endpoints() -> BTreeMap<String, String> {
    let mut endpoints = BTreeMap::new();

    endpoints.insert(
        "balance".to_string(),
        "/addresses/{kaspaAddress}/balance".to_string(),
    );
    endpoints.insert(
        "full_transactions".to_string(),
        "/addresses/{kaspaAddress}/full-transactions?limit={limit}&offset={offset}&resolve_previous_outpoints=full".to_string(),
    );
    endpoints.insert("network".to_string(), "/info/network".to_string());
    endpoints.insert("blockdag".to_string(), "/info/blockdag".to_string());
    endpoints.insert(
        "blockreward".to_string(),
        "/info/blockreward?stringOnly=false".to_string(),
    );
    endpoints.insert("coinsupply".to_string(), "/info/coinsupply".to_string());
    endpoints.insert("halving".to_string(), "/info/halving".to_string());
    endpoints.insert("hashrate".to_string(), "/info/hashrate".to_string());
    endpoints.insert("max_hashrate".to_string(), "/info/hashrate/max".to_string());
    endpoints.insert(
        "top_addresses".to_string(),
        "/addresses/top?limit=1".to_string(),
    );
    endpoints.insert("address_names".to_string(), "/addresses/names".to_string());
    endpoints.insert("kaspad".to_string(), "/info/kaspad".to_string());

    endpoints
}

fn normalize_config(config: &mut RealConfig) -> Result<(), String> {
    if config.version == 0 {
        config.version = 1;
    }

    config.language = normalize_language(&config.language);
    config.theme = normalize_theme(&config.theme);
    config.auto_refresh_seconds = config.auto_refresh_seconds.clamp(5, 86_400);
    config.retry_attempts = config.retry_attempts.clamp(1, 10);
    config.timeout_seconds = config.timeout_seconds.clamp(1, 300);
    config.backoff_factor = config.backoff_factor.clamp(0.1, 10.0);
    config.displayed_tabs = normalize_tabs(config.displayed_tabs.clone());
    config.displayed_currencies = normalize_currency_codes(config.displayed_currencies.clone());
    config.displayed_languages = normalize_languages(config.displayed_languages.clone());

    if config.api_profiles.is_empty() {
        config.api_profiles = default_real_config()?.api_profiles;
    }

    for profile in &mut config.api_profiles {
        profile.name = sanitize_label(&profile.name, "Default");
        profile.base_url = profile.base_url.trim().trim_end_matches('/').to_string();

        if profile.endpoints.is_empty() {
            profile.endpoints = default_endpoints();
        }

        let mut clean_endpoints = BTreeMap::new();

        for (key, value) in &profile.endpoints {
            let clean_key = sanitize_key(key);
            let clean_value = value.trim().to_string();

            if !clean_key.is_empty() {
                clean_endpoints.insert(clean_key, clean_value);
            }
        }

        profile.endpoints = clean_endpoints;
    }

    if config.active_api_profile.trim().is_empty()
        || !config
            .api_profiles
            .iter()
            .any(|profile| profile.name == config.active_api_profile)
    {
        config.active_api_profile = config
            .api_profiles
            .first()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| "Default".to_string());
    }

    let mut clean_paths = BTreeMap::new();

    for (key, path) in &config.paths {
        let clean_key = sanitize_key(key);

        if !clean_key.is_empty() {
            clean_paths.insert(clean_key, path.trim().to_string());
        }
    }

    config.paths = clean_paths;

    Ok(())
}

fn validate_config(config: &RealConfig) -> Result<(), String> {
    if config.language.trim().is_empty() {
        return Err("Language cannot be empty.".to_string());
    }

    if config.theme.trim().is_empty() {
        return Err("Theme cannot be empty.".to_string());
    }

    if config.auto_refresh_seconds < 5 || config.auto_refresh_seconds > 86_400 {
        return Err("Auto refresh seconds must be between 5 and 86400.".to_string());
    }

    if config.retry_attempts == 0 || config.retry_attempts > 10 {
        return Err("Retry attempts must be between 1 and 10.".to_string());
    }

    if config.timeout_seconds == 0 || config.timeout_seconds > 300 {
        return Err("Timeout seconds must be between 1 and 300.".to_string());
    }

    if !config.backoff_factor.is_finite()
        || config.backoff_factor < 0.1
        || config.backoff_factor > 10.0
    {
        return Err("Backoff factor must be between 0.1 and 10.0.".to_string());
    }

    if config.displayed_tabs.is_empty() {
        return Err("At least one displayed tab is required.".to_string());
    }

    if config.displayed_currencies.is_empty() {
        return Err("At least one displayed currency is required.".to_string());
    }

    if config.displayed_languages.is_empty() {
        return Err("At least one displayed language is required.".to_string());
    }

    if config.api_profiles.is_empty() {
        return Err("At least one API profile is required.".to_string());
    }

    let mut active_found = false;

    for profile in &config.api_profiles {
        validate_safe_key(&profile.name)?;
        validate_base_url(&profile.base_url)?;

        if profile.name == config.active_api_profile {
            active_found = true;
        }

        if profile.endpoints.is_empty() {
            return Err(format!("API profile {} has no endpoints.", profile.name));
        }

        for (name, path) in &profile.endpoints {
            validate_safe_key(name)?;
            validate_endpoint_path(path)?;
        }
    }

    if !active_found {
        return Err("Active API profile does not exist.".to_string());
    }

    for (key, path) in &config.paths {
        validate_safe_key(key)?;
        validate_path_text(path)?;
    }

    Ok(())
}

fn parse_api_profiles(value: &Value) -> Result<Vec<RealApiProfile>, String> {
    if let Some(object) = value.as_object() {
        let mut profiles = Vec::new();

        for (name, profile_value) in object {
            profiles.push(parse_single_profile(profile_value, name)?);
        }

        return Ok(profiles);
    }

    if let Some(array) = value.as_array() {
        let mut profiles = Vec::new();

        for profile_value in array {
            let name =
                find_string(profile_value, &["name"]).unwrap_or_else(|| "Default".to_string());
            profiles.push(parse_single_profile(profile_value, &name)?);
        }

        return Ok(profiles);
    }

    Err("API profiles must be an object or array.".to_string())
}

fn parse_single_profile(value: &Value, fallback_name: &str) -> Result<RealApiProfile, String> {
    let name = find_string(value, &["name"]).unwrap_or_else(|| fallback_name.to_string());
    let base_url = find_string(value, &["base_url", "baseUrl"])
        .unwrap_or_else(|| "https://api.kaspa.org".to_string());
    let enabled = find_bool(value, &["enabled"]).unwrap_or(true);

    let mut endpoints = BTreeMap::new();

    if let Some(endpoint_value) = value.get("endpoints") {
        if let Some(object) = endpoint_value.as_object() {
            for (key, endpoint) in object {
                if let Some(endpoint) = endpoint.as_str() {
                    endpoints.insert(sanitize_key(key), endpoint.trim().to_string());
                }
            }
        } else if let Some(array) = endpoint_value.as_array() {
            for item in array {
                let Some(name) = find_string(item, &["name"]) else {
                    continue;
                };

                let Some(path) = find_string(item, &["path", "endpoint"]) else {
                    continue;
                };

                endpoints.insert(sanitize_key(&name), path);
            }
        }
    }

    if endpoints.is_empty() {
        endpoints = default_endpoints();
    }

    let profile = RealApiProfile {
        name: sanitize_label(&name, fallback_name),
        base_url: base_url.trim().trim_end_matches('/').to_string(),
        endpoints,
        enabled,
    };

    validate_base_url(&profile.base_url)?;

    for (name, path) in &profile.endpoints {
        validate_safe_key(name)?;
        validate_endpoint_path(path)?;
    }

    Ok(profile)
}

fn migrate_paths(json: &Value, paths: &mut BTreeMap<String, String>, warnings: &mut Vec<String>) {
    let Some(paths_value) = json.get("paths").or_else(|| json.pointer("/storage/paths")) else {
        return;
    };

    let Some(object) = paths_value.as_object() else {
        warnings.push("paths exists but is not an object.".to_string());
        return;
    };

    for (key, value) in object {
        let Some(path) = value.as_str() else {
            continue;
        };

        let clean_key = sanitize_key(key);

        if validate_safe_key(&clean_key).is_ok() && validate_path_text(path).is_ok() {
            paths.insert(clean_key, path.trim().to_string());
        }
    }
}

fn safe_existing_file(value: &str) -> Result<PathBuf, String> {
    validate_path_text(value)?;

    let path = PathBuf::from(value.trim());

    if !path.exists() {
        return Err("Config path does not exist.".to_string());
    }

    if !path.is_file() {
        return Err("Config path is not a file.".to_string());
    }

    Ok(path)
}

fn validate_path_text(value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err("Path contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_safe_key(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("Key cannot be empty.".to_string());
    }

    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err("Key contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_base_url(value: &str) -> Result<(), String> {
    let value = value.trim();

    if !value.starts_with("https://") && !value.starts_with("http://") {
        return Err("Base URL must start with http:// or https://.".to_string());
    }

    if value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains('\\')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err("Base URL contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_endpoint_path(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("Endpoint path cannot be empty.".to_string());
    }

    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("//") {
        return Err("Endpoint path must be relative.".to_string());
    }

    if value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains('\\')
    {
        return Err("Endpoint path contains unsafe characters.".to_string());
    }

    Ok(())
}

fn find_value<'a>(value: &'a Value, dotted_path: &str) -> Option<&'a Value> {
    if let Some(found) = value.get(dotted_path) {
        return Some(found);
    }

    let mut current = value;

    for part in dotted_path.split('.') {
        current = current.get(part)?;
    }

    Some(current)
}

fn find_string(value: &Value, paths: &[&str]) -> Option<String> {
    for path in paths {
        if let Some(found) = find_value(value, path) {
            if let Some(text) = found.as_str() {
                if !text.trim().is_empty() {
                    return Some(text.trim().to_string());
                }
            }
        }
    }

    None
}

fn find_bool(value: &Value, paths: &[&str]) -> Option<bool> {
    for path in paths {
        if let Some(found) = find_value(value, path) {
            if let Some(value) = found.as_bool() {
                return Some(value);
            }

            if let Some(text) = found.as_str() {
                match text.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => return Some(true),
                    "false" | "0" | "no" | "off" => return Some(false),
                    _ => {}
                }
            }
        }
    }

    None
}

fn find_u64(value: &Value, paths: &[&str]) -> Option<u64> {
    for path in paths {
        if let Some(found) = find_value(value, path) {
            if let Some(value) = found.as_u64() {
                return Some(value);
            }

            if let Some(value) = found.as_i64() {
                if value >= 0 {
                    return u64::try_from(value).ok();
                }
            }

            if let Some(text) = found.as_str() {
                if let Ok(parsed) = text.parse::<u64>() {
                    return Some(parsed);
                }
            }
        }
    }

    None
}

fn find_f64(value: &Value, paths: &[&str]) -> Option<f64> {
    for path in paths {
        if let Some(found) = find_value(value, path) {
            if let Some(value) = found.as_f64() {
                return Some(value);
            }

            if let Some(text) = found.as_str() {
                if let Ok(parsed) = text.parse::<f64>() {
                    return Some(parsed);
                }
            }
        }
    }

    None
}

fn find_string_array(value: &Value, paths: &[&str]) -> Option<Vec<String>> {
    for path in paths {
        if let Some(found) = find_value(value, path) {
            if let Some(array) = found.as_array() {
                let values = array
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();

                if !values.is_empty() {
                    return Some(values);
                }
            }
        }
    }

    None
}

fn normalize_language(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "en" | "english" => "English".to_string(),
        "ar" | "arabic" | "العربية" => "Arabic".to_string(),
        "ru" | "russian" => "Russian".to_string(),
        "tr" | "turkish" => "Turkish".to_string(),
        "de" | "german" => "German".to_string(),
        "es" | "spanish" => "Spanish".to_string(),
        "fr" | "french" => "French".to_string(),
        "hi" | "hindi" => "Hindi".to_string(),
        "ja" | "japanese" => "Japanese".to_string(),
        "ko" | "korean" => "Korean".to_string(),
        "zh-cn" | "chinese" => "Chinese".to_string(),
        "id" | "indonesian" => "Indonesian".to_string(),
        _ => sanitize_label(value, "English"),
    }
}

fn normalize_theme(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "light" => "Light".to_string(),
        "dark" | "superhero" => "Dark".to_string(),
        "system" => "System".to_string(),
        _ => "Dark".to_string(),
    }
}

fn normalize_tabs(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| sanitize_label(&value, "Explorer"))
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();

    if values.is_empty() {
        values = default_tabs();
    }

    values.sort();
    values.dedup();
    values
}

fn normalize_currency_codes(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| {
            value.len() >= 2 && value.len() <= 8 && value.chars().all(|ch| ch.is_ascii_alphabetic())
        })
        .collect::<Vec<_>>();

    if values.is_empty() {
        values = default_currencies();
    }

    values.sort();
    values.dedup();
    values
}

fn normalize_languages(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| normalize_language(&value))
        .collect::<Vec<_>>();

    if values.is_empty() {
        values = default_languages();
    }

    values.sort();
    values.dedup();
    values
}

fn sanitize_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-' || *ch == '.')
        .collect::<String>()
        .trim()
        .to_string()
}

fn sanitize_label(value: &str, fallback: &str) -> String {
    let clean = value
        .chars()
        .filter(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, ' ' | '_' | '-' | '.' | ':' | '(' | ')' | '[' | ']')
        })
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean
    }
}

fn default_tabs() -> Vec<String> {
    vec![
        "Dashboard".to_string(),
        "Kaspa Node".to_string(),
        "Kaspa Bridge".to_string(),
        "Explorer".to_string(),
        "Top Addresses".to_string(),
        "Manage Addresses".to_string(),
        "Analytics".to_string(),
        "Settings".to_string(),
        "Logs".to_string(),
        "Export".to_string(),
    ]
}

fn default_currencies() -> Vec<String> {
    vec![
        "USD".to_string(),
        "SAR".to_string(),
        "EUR".to_string(),
        "GBP".to_string(),
    ]
}

fn default_languages() -> Vec<String> {
    vec![
        "English".to_string(),
        "Arabic".to_string(),
        "Russian".to_string(),
        "Turkish".to_string(),
        "German".to_string(),
        "Spanish".to_string(),
        "French".to_string(),
        "Hindi".to_string(),
        "Japanese".to_string(),
        "Korean".to_string(),
        "Chinese".to_string(),
        "Indonesian".to_string(),
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
    fn default_config_is_valid() {
        let config = default_real_config().expect("config");
        validate_config(&config).expect("valid config");
        assert_eq!(config.active_api_profile, "Default");
    }

    #[test]
    fn dotted_path_lookup_works() {
        let value = json!({
            "display": {
                "language": "Arabic",
                "supported_currencies": ["usd", "sar"]
            }
        });

        assert_eq!(
            find_string(&value, &["display.language"]).unwrap(),
            "Arabic"
        );
        assert_eq!(
            find_string_array(&value, &["display.supported_currencies"]).unwrap(),
            vec!["usd".to_string(), "sar".to_string()]
        );
    }

    #[test]
    fn endpoint_path_rejects_absolute_urls() {
        assert!(validate_endpoint_path("/info/network").is_ok());
        assert!(validate_endpoint_path("https://evil.example").is_err());
    }

    #[test]
    fn currency_codes_are_normalized() {
        let values = normalize_currency_codes(vec![
            "usd".to_string(),
            "SAR".to_string(),
            "bad1".to_string(),
        ]);
        assert_eq!(values, vec!["SAR".to_string(), "USD".to_string()]);
    }
}
