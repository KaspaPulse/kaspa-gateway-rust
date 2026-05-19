use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};

const SETTINGS_KEY: &str = "desktop.full_settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopApiProfile {
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopFullSettings {
    pub language: String,
    pub theme: String,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_interval_seconds: u64,
    pub logging_level: String,
    pub displayed_tabs: Vec<String>,
    pub displayed_currencies: Vec<String>,
    pub displayed_languages: Vec<String>,
    pub api_profiles: Vec<DesktopApiProfile>,
    pub active_api_profile: String,
    pub database_path: String,
    pub log_path: String,
    pub export_path: String,
    pub backup_path: String,
}

#[tauri::command]
pub fn load_full_settings() -> Result<DesktopFullSettings, String> {
    let manager = database_manager()?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    match repository
        .get(SETTINGS_KEY)
        .map_err(|error| error.to_string())?
    {
        Some(value) => {
            let settings = serde_json::from_str::<DesktopFullSettings>(&value)
                .map_err(|error| error.to_string())?;
            validate_full_settings(&settings)?;
            Ok(settings)
        }
        None => Ok(default_full_settings()?),
    }
}

#[tauri::command]
pub fn save_full_settings(settings: DesktopFullSettings) -> Result<DesktopFullSettings, String> {
    validate_full_settings(&settings)?;

    let manager = database_manager()?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repository
        .set(
            SETTINGS_KEY,
            &serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

    Ok(settings)
}

#[tauri::command]
pub fn reset_full_settings() -> Result<DesktopFullSettings, String> {
    let settings = default_full_settings()?;
    save_full_settings(settings)
}

pub fn default_full_settings() -> Result<DesktopFullSettings, String> {
    let root = default_user_data_dir().map_err(|error| error.to_string())?;
    let database_path = root.join("databases");
    let log_path = root.join("logs");
    let export_path = root.join("exports");
    let backup_path = root.join("backups");

    Ok(DesktopFullSettings {
        language: "English".to_string(),
        theme: "Dark".to_string(),
        auto_refresh_enabled: false,
        auto_refresh_interval_seconds: 60,
        logging_level: "INFO".to_string(),
        displayed_tabs: vec![
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
            "About".to_string(),
        ],
        displayed_currencies: vec![
            "KAS".to_string(),
            "USD".to_string(),
            "SAR".to_string(),
            "EUR".to_string(),
        ],
        displayed_languages: vec![
            "English".to_string(),
            "Arabic".to_string(),
            "German".to_string(),
            "Spanish".to_string(),
        ],
        api_profiles: vec![DesktopApiProfile {
            name: "default".to_string(),
            base_url: "https://api.kaspa.org".to_string(),
            enabled: true,
        }],
        active_api_profile: "default".to_string(),
        database_path: database_path.display().to_string(),
        log_path: log_path.display().to_string(),
        export_path: export_path.display().to_string(),
        backup_path: backup_path.display().to_string(),
    })
}

fn validate_full_settings(settings: &DesktopFullSettings) -> Result<(), String> {
    validate_choice(
        "language",
        &settings.language,
        &["English", "Arabic", "German", "Spanish"],
    )?;

    validate_choice("theme", &settings.theme, &["Dark", "Light", "System"])?;

    validate_choice(
        "logging_level",
        &settings.logging_level,
        &["TRACE", "DEBUG", "INFO", "WARN", "ERROR"],
    )?;

    if settings.auto_refresh_interval_seconds < 5 || settings.auto_refresh_interval_seconds > 86_400
    {
        return Err("Auto-refresh interval must be between 5 and 86400 seconds.".to_string());
    }

    if settings.displayed_tabs.is_empty() {
        return Err("At least one displayed tab must be enabled.".to_string());
    }

    if settings.displayed_currencies.is_empty() {
        return Err("At least one displayed currency must be enabled.".to_string());
    }

    if settings.displayed_languages.is_empty() {
        return Err("At least one displayed language must be enabled.".to_string());
    }

    if settings.api_profiles.is_empty() {
        return Err("At least one API profile is required.".to_string());
    }

    let mut active_exists = false;
    for profile in &settings.api_profiles {
        validate_non_empty("API profile name", &profile.name)?;
        validate_https_url(&profile.base_url)?;
        if profile.name == settings.active_api_profile {
            active_exists = true;
        }
    }

    if !active_exists {
        return Err("Active API profile must exist in API profiles.".to_string());
    }

    validate_path_text("database_path", &settings.database_path)?;
    validate_path_text("log_path", &settings.log_path)?;
    validate_path_text("export_path", &settings.export_path)?;
    validate_path_text("backup_path", &settings.backup_path)?;

    Ok(())
}

fn validate_choice(name: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if !allowed.contains(&value) {
        return Err(format!("{name} has unsupported value: {value}"));
    }

    Ok(())
}

fn validate_non_empty(name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{name} cannot be empty."));
    }

    if value.contains('\0') {
        return Err(format!("{name} cannot contain null bytes."));
    }

    Ok(())
}

fn validate_https_url(value: &str) -> Result<(), String> {
    validate_non_empty("API URL", value)?;

    if !value.starts_with("https://") {
        return Err("API profile URL must use HTTPS.".to_string());
    }

    if value.contains('\0') || value.contains(' ') || value.contains('\n') || value.contains('\r') {
        return Err("API profile URL contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_path_text(name: &str, value: &str) -> Result<(), String> {
    validate_non_empty(name, value)?;

    if value.contains('"') || value.contains('\n') || value.contains('\r') {
        return Err(format!("{name} contains unsafe characters."));
    }

    Ok(())
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
