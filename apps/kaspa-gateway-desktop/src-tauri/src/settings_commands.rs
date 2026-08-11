use kaspa_gateway_config::{
    SUPPORTED_CURRENCIES, SUPPORTED_LANGUAGES, SUPPORTED_TABS, default_api_profile,
    default_user_data_dir,
};
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const SETTINGS_KEY: &str = "settings.deep.v1.json";
const SECURE_STORE_PREFIX: &str = "secure.setting.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProfileDeep {
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub timeout_seconds: u64,
    pub retry_attempts: u8,
    pub backoff_factor: f64,
    pub auto_refresh_interval_seconds: u64,
    pub page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDeep {
    pub language: String,
    pub theme: String,
    pub logging_level: String,
    pub auto_refresh_enabled: bool,
    pub displayed_tabs: Vec<String>,
    pub displayed_currencies: Vec<String>,
    pub displayed_languages: Vec<String>,
    pub active_api_profile: String,
    pub api_profiles: Vec<ApiProfileDeep>,
    pub paths: BTreeMap<String, String>,
    pub performance: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfigMigrationRequest {
    pub source_path: String,
    pub save_after_import: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingsConfigMigrationReport {
    pub source_path: String,
    pub imported: bool,
    pub settings: SettingsDeep,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureSettingRequest {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecureSettingReport {
    pub key: String,
    pub stored: bool,
    pub masked_value: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SafePathReport {
    pub key: String,
    pub path: String,
    pub exists: bool,
    pub created: bool,
    pub message: String,
}

#[tauri::command]
pub fn settings_defaults() -> Result<SettingsDeep, String> {
    default_settings()
}

#[tauri::command]
pub fn settings_load() -> Result<SettingsDeep, String> {
    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let Some(value) = repo.get(SETTINGS_KEY).map_err(|error| error.to_string())? else {
        return default_settings();
    };

    let mut settings =
        serde_json::from_str::<SettingsDeep>(&value).map_err(|error| error.to_string())?;

    normalize_loaded_settings(&mut settings)?;
    validate_settings(&settings)?;

    Ok(settings)
}

#[tauri::command]
pub fn settings_save(settings: SettingsDeep) -> Result<SettingsDeep, String> {
    let mut settings = settings;
    normalize_loaded_settings(&mut settings)?;
    validate_settings(&settings)?;

    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repo.set(
        SETTINGS_KEY,
        &serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(settings)
}

#[tauri::command]
pub fn settings_reset() -> Result<SettingsDeep, String> {
    let settings = default_settings()?;
    settings_save(settings)
}

#[tauri::command]
pub fn settings_import_config(
    request: SettingsConfigMigrationRequest,
) -> Result<SettingsConfigMigrationReport, String> {
    let source = safe_existing_file(&request.source_path)?;
    let text = fs::read_to_string(&source).map_err(|error| error.to_string())?;
    let json: Value = serde_json::from_str(&text).map_err(|error| error.to_string())?;

    let mut settings = default_settings()?;
    let mut warnings = Vec::new();

    if let Some(language) = find_string(&json, &["language", "display.language", "ui.language"]) {
        settings.language = normalize_language(&language);
    }

    if let Some(theme) = find_string(&json, &["theme", "display.theme", "ui.theme"]) {
        settings.theme = normalize_theme(&theme);
    }

    if let Some(level) = find_string(&json, &["logging_level", "log_level", "logging.level"]) {
        settings.logging_level = normalize_logging_level(&level);
    }

    if let Some(enabled) = find_bool(
        &json,
        &[
            "auto_refresh_enabled",
            "auto_refresh",
            "display.auto_refresh",
        ],
    ) {
        settings.auto_refresh_enabled = enabled;
    }

    if let Some(tabs) =
        find_string_array(&json, &["displayed_tabs", "display.displayed_tabs", "tabs"])
    {
        settings.displayed_tabs = normalize_tabs(tabs);
    }

    if let Some(currencies) = find_string_array(
        &json,
        &[
            "displayed_currencies",
            "supported_currencies",
            "display.supported_currencies",
        ],
    ) {
        settings.displayed_currencies = normalize_codes(currencies);
    }

    if let Some(languages) = find_string_array(
        &json,
        &[
            "displayed_languages",
            "supported_languages",
            "display.supported_languages",
        ],
    ) {
        settings.displayed_languages = normalize_languages(languages);
    }

    if let Some(timeout) = find_u64(
        &json,
        &["timeout_seconds", "timeout", "performance.timeout"],
    ) {
        settings.performance.timeout_seconds = timeout.clamp(1, 300);
    }

    if let Some(retry) = find_u64(&json, &["retry_attempts", "performance.retry_attempts"]) {
        settings.performance.retry_attempts = u8::try_from(retry.clamp(1, 10)).unwrap_or(3);
    }

    if let Some(backoff) = find_f64(&json, &["backoff_factor", "performance.backoff_factor"]) {
        settings.performance.backoff_factor = backoff.clamp(0.1, 10.0);
    }

    if let Some(interval) = find_u64(
        &json,
        &[
            "auto_refresh_interval_seconds",
            "auto_refresh_interval",
            "display.auto_refresh_interval",
        ],
    ) {
        settings.performance.auto_refresh_interval_seconds = interval.clamp(5, 86_400);
    }

    if let Some(page_size) = find_u64(&json, &["page_size", "performance.page_size"]) {
        settings.performance.page_size = usize::try_from(page_size.clamp(1, 10_000)).unwrap_or(500);
    }

    migrate_paths(&json, &mut settings.paths, &mut warnings);

    if let Some(api_profiles) = json
        .get("api_profiles")
        .or_else(|| json.pointer("/api/profiles"))
    {
        match parse_api_profiles(api_profiles) {
            Ok(profiles) if !profiles.is_empty() => {
                settings.active_api_profile = profiles[0].name.clone();
                settings.api_profiles = profiles;
            }
            Ok(_) => warnings.push("No valid API profiles found in config.".to_string()),
            Err(error) => warnings.push(format!("API profile migration warning: {error}")),
        }
    } else if let Some(api) = json.get("api") {
        match parse_single_api_profile(api, "Default") {
            Ok(profile) => {
                settings.active_api_profile = profile.name.clone();
                settings.api_profiles = vec![profile];
            }
            Err(error) => warnings.push(format!("Legacy API migration warning: {error}")),
        }
    }

    normalize_loaded_settings(&mut settings)?;
    validate_settings(&settings)?;

    let imported = if request.save_after_import {
        settings_save(settings.clone())?;
        true
    } else {
        false
    };

    Ok(SettingsConfigMigrationReport {
        source_path: source.display().to_string(),
        imported,
        settings,
        warnings,
    })
}

#[tauri::command]
pub fn settings_validate_custom_path(
    key: String,
    path: String,
    create_if_missing: bool,
) -> Result<SafePathReport, String> {
    validate_safe_key(&key)?;
    validate_path_text(&path)?;

    let path_buf = PathBuf::from(path.trim());
    let existed = path_buf.exists();
    let mut created = false;

    if !existed && create_if_missing {
        fs::create_dir_all(&path_buf).map_err(|error| error.to_string())?;
        created = true;
    }

    Ok(SafePathReport {
        key,
        path: path_buf.display().to_string(),
        exists: path_buf.exists(),
        created,
        message: if existed {
            "Path exists.".to_string()
        } else if created {
            "Path created.".to_string()
        } else {
            "Path does not exist.".to_string()
        },
    })
}

#[tauri::command]
pub fn settings_secure_store(request: SecureSettingRequest) -> Result<SecureSettingReport, String> {
    validate_safe_key(&request.key)?;
    validate_secure_value(&request.value)?;

    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let storage_key = format!("{SECURE_STORE_PREFIX}{}", request.key);
    let encoded = hex_encode(request.value.as_bytes());

    repo.set(&storage_key, &encoded)
        .map_err(|error| error.to_string())?;

    Ok(SecureSettingReport {
        key: request.key,
        stored: true,
        masked_value: "********".to_string(),
        message: "Secure setting stored in local app database using masked UI display.".to_string(),
    })
}

#[tauri::command]
pub fn settings_secure_get_masked(key: String) -> Result<SecureSettingReport, String> {
    validate_safe_key(&key)?;

    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let storage_key = format!("{SECURE_STORE_PREFIX}{key}");
    let stored = repo
        .get(&storage_key)
        .map_err(|error| error.to_string())?
        .is_some();

    Ok(SecureSettingReport {
        key,
        stored,
        masked_value: if stored {
            "********".to_string()
        } else {
            String::new()
        },
        message: if stored {
            "Secure setting exists.".to_string()
        } else {
            "Secure setting not found.".to_string()
        },
    })
}

#[tauri::command]
pub fn settings_api_endpoint_editor(
    profile_name: String,
    endpoints: Vec<ApiEndpoint>,
) -> Result<SettingsDeep, String> {
    validate_safe_key(&profile_name)?;

    let mut settings = settings_load()?;

    let Some(profile) = settings
        .api_profiles
        .iter_mut()
        .find(|profile| profile.name == profile_name)
    else {
        return Err("API profile was not found.".to_string());
    };

    for endpoint in &endpoints {
        validate_api_endpoint(endpoint)?;
    }

    profile.endpoints = endpoints;
    settings_save(settings)
}

fn default_settings() -> Result<SettingsDeep, String> {
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

    let default_profile = default_api_profile();

    Ok(SettingsDeep {
        language: "English".to_string(),
        theme: "Dark".to_string(),
        logging_level: "INFO".to_string(),
        auto_refresh_enabled: false,
        displayed_tabs: SUPPORTED_TABS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        displayed_currencies: SUPPORTED_CURRENCIES
            .iter()
            .map(|value| value.to_ascii_uppercase())
            .collect(),
        displayed_languages: SUPPORTED_LANGUAGES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        active_api_profile: "Default".to_string(),
        api_profiles: vec![ApiProfileDeep {
            name: "Default".to_string(),
            base_url: default_profile.base_url,
            enabled: true,
            endpoints: default_profile
                .endpoints
                .into_iter()
                .map(|(name, path)| ApiEndpoint {
                    name,
                    path,
                    enabled: true,
                })
                .collect(),
        }],
        paths,
        performance: PerformanceSettings {
            timeout_seconds: 30,
            retry_attempts: 5,
            backoff_factor: 0.5,
            auto_refresh_interval_seconds: 60,
            page_size: 500,
        },
    })
}

fn normalize_loaded_settings(settings: &mut SettingsDeep) -> Result<(), String> {
    settings.language = normalize_language(&settings.language);
    settings.theme = normalize_theme(&settings.theme);
    settings.logging_level = normalize_logging_level(&settings.logging_level);
    settings.displayed_tabs = normalize_tabs(settings.displayed_tabs.clone());
    settings.displayed_currencies = normalize_codes(settings.displayed_currencies.clone());
    settings.displayed_languages = normalize_languages(settings.displayed_languages.clone());

    if settings.active_api_profile.trim().is_empty() {
        settings.active_api_profile = "Default".to_string();
    }

    for profile in &mut settings.api_profiles {
        profile.name = sanitize_label(&profile.name, "Default");
        profile.base_url = profile.base_url.trim().trim_end_matches('/').to_string();

        for endpoint in &mut profile.endpoints {
            endpoint.name = sanitize_key(&endpoint.name);
            endpoint.path = endpoint.path.trim().to_string();
        }
    }

    if settings.api_profiles.is_empty() {
        settings.api_profiles = default_settings()?.api_profiles;
    }

    if !settings
        .api_profiles
        .iter()
        .any(|profile| profile.name == settings.active_api_profile)
    {
        settings.active_api_profile = settings
            .api_profiles
            .first()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| "Default".to_string());
    }

    Ok(())
}

fn validate_settings(settings: &SettingsDeep) -> Result<(), String> {
    if settings.language.trim().is_empty() {
        return Err("Language cannot be empty.".to_string());
    }

    if settings.theme.trim().is_empty() {
        return Err("Theme cannot be empty.".to_string());
    }

    validate_logging_level(&settings.logging_level)?;

    if settings.displayed_tabs.is_empty() {
        return Err("At least one tab must be displayed.".to_string());
    }

    if settings.displayed_currencies.is_empty() {
        return Err("At least one currency must be displayed.".to_string());
    }

    if settings.displayed_languages.is_empty() {
        return Err("At least one language must be displayed.".to_string());
    }

    if settings.performance.timeout_seconds == 0 || settings.performance.timeout_seconds > 300 {
        return Err("Timeout must be between 1 and 300 seconds.".to_string());
    }

    if settings.performance.retry_attempts == 0 || settings.performance.retry_attempts > 10 {
        return Err("Retry attempts must be between 1 and 10.".to_string());
    }

    if !settings.performance.backoff_factor.is_finite()
        || settings.performance.backoff_factor < 0.1
        || settings.performance.backoff_factor > 10.0
    {
        return Err("Backoff factor must be between 0.1 and 10.0.".to_string());
    }

    if settings.performance.auto_refresh_interval_seconds < 5
        || settings.performance.auto_refresh_interval_seconds > 86_400
    {
        return Err("Auto refresh interval must be between 5 seconds and 24 hours.".to_string());
    }

    if settings.performance.page_size == 0 || settings.performance.page_size > 10_000 {
        return Err("Page size must be between 1 and 10000.".to_string());
    }

    for (key, path) in &settings.paths {
        validate_safe_key(key)?;
        validate_path_text(path)?;
    }

    if settings.api_profiles.is_empty() {
        return Err("At least one API profile is required.".to_string());
    }

    let mut active_found = false;

    for profile in &settings.api_profiles {
        validate_safe_key(&profile.name)?;
        validate_base_url(&profile.base_url)?;

        if profile.name == settings.active_api_profile {
            active_found = true;
        }

        if profile.endpoints.is_empty() {
            return Err(format!("API profile {} has no endpoints.", profile.name));
        }

        for endpoint in &profile.endpoints {
            validate_api_endpoint(endpoint)?;
        }
    }

    if !active_found {
        return Err("Active API profile does not exist.".to_string());
    }

    Ok(())
}

fn validate_api_endpoint(endpoint: &ApiEndpoint) -> Result<(), String> {
    validate_safe_key(&endpoint.name)?;

    if endpoint.path.trim().is_empty() {
        return Err("API endpoint path cannot be empty.".to_string());
    }

    if endpoint.path.starts_with("http://")
        || endpoint.path.starts_with("https://")
        || endpoint.path.starts_with("//")
    {
        return Err("API endpoint path must be relative.".to_string());
    }

    if endpoint.path.contains('\0')
        || endpoint.path.contains('"')
        || endpoint.path.contains('\n')
        || endpoint.path.contains('\r')
        || endpoint.path.contains('\\')
    {
        return Err("API endpoint path contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_base_url(value: &str) -> Result<(), String> {
    let value = value.trim();

    if !(value.starts_with("https://") || value.starts_with("http://")) {
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

fn validate_logging_level(value: &str) -> Result<(), String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" => Ok(()),
        _ => Err("Invalid logging level.".to_string()),
    }
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

fn validate_secure_value(value: &str) -> Result<(), String> {
    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err("Secure value contains unsafe characters.".to_string());
    }

    if value.len() > 16_384 {
        return Err("Secure value is too large.".to_string());
    }

    Ok(())
}

fn safe_existing_file(value: &str) -> Result<PathBuf, String> {
    validate_path_text(value)?;

    let path = PathBuf::from(value.trim());

    if !path.exists() {
        return Err("Source config file does not exist.".to_string());
    }

    if !path.is_file() {
        return Err("Source config path is not a file.".to_string());
    }

    Ok(path)
}

fn parse_api_profiles(value: &Value) -> Result<Vec<ApiProfileDeep>, String> {
    if let Some(array) = value.as_array() {
        let mut profiles = Vec::new();

        for item in array {
            let name = find_string(item, &["name"]).unwrap_or_else(|| "Default".to_string());
            profiles.push(parse_single_api_profile(item, &name)?);
        }

        return Ok(profiles);
    }

    if let Some(object) = value.as_object() {
        let mut profiles = Vec::new();

        for (name, item) in object {
            profiles.push(parse_single_api_profile(item, name)?);
        }

        return Ok(profiles);
    }

    Err("API profiles must be an array or object.".to_string())
}

fn parse_single_api_profile(value: &Value, fallback_name: &str) -> Result<ApiProfileDeep, String> {
    let name = find_string(value, &["name"]).unwrap_or_else(|| fallback_name.to_string());
    let base_url = find_string(value, &["base_url", "baseUrl"])
        .unwrap_or_else(|| "https://api.kaspa.org".to_string());
    let enabled = find_bool(value, &["enabled"]).unwrap_or(true);

    let mut endpoints = Vec::new();

    if let Some(endpoint_value) = value.get("endpoints") {
        if let Some(object) = endpoint_value.as_object() {
            for (name, path) in object {
                if let Some(path) = path.as_str() {
                    endpoints.push(ApiEndpoint {
                        name: sanitize_key(name),
                        path: path.to_string(),
                        enabled: true,
                    });
                }
            }
        } else if let Some(array) = endpoint_value.as_array() {
            for item in array {
                if let Some(name) = find_string(item, &["name"])
                    && let Some(path) = find_string(item, &["path", "endpoint"])
                {
                    endpoints.push(ApiEndpoint {
                        name: sanitize_key(&name),
                        path,
                        enabled: find_bool(item, &["enabled"]).unwrap_or(true),
                    });
                }
            }
        }
    }

    if endpoints.is_empty() {
        endpoints = default_settings()?
            .api_profiles
            .into_iter()
            .next()
            .map(|profile| profile.endpoints)
            .unwrap_or_default();
    }

    let profile = ApiProfileDeep {
        name: sanitize_label(&name, fallback_name),
        base_url: base_url.trim().trim_end_matches('/').to_string(),
        enabled,
        endpoints,
    };

    validate_base_url(&profile.base_url)?;

    for endpoint in &profile.endpoints {
        validate_api_endpoint(endpoint)?;
    }

    Ok(profile)
}

fn migrate_paths(json: &Value, paths: &mut BTreeMap<String, String>, warnings: &mut Vec<String>) {
    let Some(paths_value) = json.get("paths") else {
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
            paths.insert(clean_key, path.to_string());
        }
    }
}

fn find_string(value: &Value, paths: &[&str]) -> Option<String> {
    for path in paths {
        if let Some(found) = find_value(value, path)
            && let Some(text) = found.as_str()
            && !text.trim().is_empty()
        {
            return Some(text.trim().to_string());
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

            if let Some(value) = found.as_i64()
                && value >= 0
            {
                return u64::try_from(value).ok();
            }

            if let Some(text) = found.as_str()
                && let Ok(parsed) = text.parse::<u64>()
            {
                return Some(parsed);
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

            if let Some(text) = found.as_str()
                && let Ok(parsed) = text.parse::<f64>()
            {
                return Some(parsed);
            }
        }
    }

    None
}

fn find_string_array(value: &Value, paths: &[&str]) -> Option<Vec<String>> {
    for path in paths {
        if let Some(found) = find_value(value, path)
            && let Some(array) = found.as_array()
        {
            let items = array
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect::<Vec<_>>();

            if !items.is_empty() {
                return Some(items);
            }
        }
    }

    None
}

fn find_value<'a>(value: &'a Value, dotted_path: &str) -> Option<&'a Value> {
    if let Some(value) = value.get(dotted_path) {
        return Some(value);
    }

    let mut current = value;

    for part in dotted_path.split('.') {
        current = current.get(part)?;
    }

    Some(current)
}

fn normalize_language(value: &str) -> String {
    let trimmed = value.trim();

    match trimmed.to_ascii_lowercase().as_str() {
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
        _ => sanitize_label(trimmed, "English"),
    }
}

fn normalize_theme(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "light" => "Light".to_string(),
        "dark" => "Dark".to_string(),
        "system" => "System".to_string(),
        "superhero" => "Dark".to_string(),
        _ => "Dark".to_string(),
    }
}

fn normalize_logging_level(value: &str) -> String {
    match value.trim().to_ascii_uppercase().as_str() {
        "TRACE" => "TRACE".to_string(),
        "DEBUG" => "DEBUG".to_string(),
        "WARN" => "WARN".to_string(),
        "ERROR" => "ERROR".to_string(),
        _ => "INFO".to_string(),
    }
}

fn normalize_tabs(values: Vec<String>) -> Vec<String> {
    let mut out = values
        .into_iter()
        .map(|value| sanitize_label(&value, "Explorer"))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    if out.is_empty() {
        out = SUPPORTED_TABS
            .iter()
            .map(|value| (*value).to_string())
            .collect();
    }

    out.sort();
    out.dedup();
    out
}

fn normalize_codes(values: Vec<String>) -> Vec<String> {
    let mut out = values
        .into_iter()
        .map(|value| value.trim().to_ascii_uppercase())
        .filter(|value| {
            value.len() >= 2 && value.len() <= 8 && value.chars().all(|ch| ch.is_ascii_alphabetic())
        })
        .collect::<Vec<_>>();

    if out.is_empty() {
        out = SUPPORTED_CURRENCIES
            .iter()
            .map(|value| value.to_ascii_uppercase())
            .collect();
    }

    out.sort();
    out.dedup();
    out
}

fn normalize_languages(values: Vec<String>) -> Vec<String> {
    let mut out = values
        .into_iter()
        .map(|value| normalize_language(&value))
        .collect::<Vec<_>>();

    if out.is_empty() {
        out = SUPPORTED_LANGUAGES
            .iter()
            .map(|value| (*value).to_string())
            .collect();
    }

    out.sort();
    out.dedup();
    out
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

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }

    out
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
    fn defaults_are_valid() {
        let settings = default_settings().expect("settings");
        validate_settings(&settings).expect("valid settings");
        assert_eq!(settings.active_api_profile, "Default");
    }

    #[test]
    fn import_helpers_find_dotted_values() {
        let value = json!({
            "performance": {
                "timeout": 42
            },
            "display": {
                "supported_currencies": ["usd", "sar"]
            }
        });

        assert_eq!(find_u64(&value, &["performance.timeout"]), Some(42));
        assert_eq!(
            find_string_array(&value, &["display.supported_currencies"]).unwrap(),
            vec!["usd".to_string(), "sar".to_string()]
        );
    }

    #[test]
    fn unsafe_keys_are_rejected() {
        assert!(validate_safe_key("api.token").is_ok());
        assert!(validate_safe_key("api token").is_err());
        assert!(validate_safe_key("../bad").is_err());
    }

    #[test]
    fn endpoint_paths_must_be_relative() {
        let endpoint = ApiEndpoint {
            name: "balance".to_string(),
            path: "https://evil.example".to_string(),
            enabled: true,
        };

        assert!(validate_api_endpoint(&endpoint).is_err());
    }

    #[test]
    fn secure_values_are_encoded_as_hex() {
        assert_eq!(hex_encode(b"abc"), "616263");
    }
}
