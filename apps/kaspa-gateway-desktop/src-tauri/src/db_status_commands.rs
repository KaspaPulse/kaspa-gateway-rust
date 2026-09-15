use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize)]
pub struct SettingsDatabaseStatusRow {
    pub file: String,
    pub path: String,
    pub exists: bool,
    pub size_kb: f64,
    pub last_modified: String,
    pub details: String,
}

fn unix_ms_to_text(ms: u128) -> String {
    // Keep this simple and stable for frontend display.
    // JS can format later if needed.
    format!("{ms}")
}

fn find_existing_file(root: &Path, candidates: &[&str]) -> PathBuf {
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();

            for candidate in candidates {
                if name == candidate.to_ascii_lowercase() {
                    return path;
                }
            }
        }
    }

    root.join(candidates[0])
}

fn file_status(
    root: &Path,
    display_name: &str,
    candidates: &[&str],
    details: String,
) -> SettingsDatabaseStatusRow {
    let path = find_existing_file(root, candidates);

    match fs::metadata(&path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| unix_ms_to_text(duration.as_millis()))
                .unwrap_or_else(|| "unknown".to_string());

            SettingsDatabaseStatusRow {
                file: display_name.to_string(),
                path: path.display().to_string(),
                exists: true,
                size_kb: metadata.len() as f64 / 1024.0,
                last_modified: modified,
                details,
            }
        }
        Err(_) => SettingsDatabaseStatusRow {
            file: display_name.to_string(),
            path: path.display().to_string(),
            exists: false,
            size_kb: 0.0,
            last_modified: "missing".to_string(),
            details: "missing".to_string(),
        },
    }
}

#[tauri::command]
pub fn kgw_settings_database_status() -> Result<Vec<SettingsDatabaseStatusRow>, String> {
    crate::db_state::with_database_manager("settings.db_status", database_status_for_manager)
}

fn database_status_for_manager(
    manager: &kaspa_gateway_db::DatabaseManager,
) -> Result<Vec<SettingsDatabaseStatusRow>, String> {
    let root = &manager.paths().root;
    let address_count = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?
        .list()
        .map_err(|error| error.to_string())?
        .len();

    let rows = vec![
        file_status(
            root,
            "Addresses.duckdb",
            &["Addresses.duckdb", "addresses.duckdb"],
            format!("{address_count} addresses"),
        ),
        file_status(
            root,
            "AppData.duckdb",
            &["AppData.duckdb", "appdata.duckdb", "app_data.duckdb"],
            "application settings / app data".to_string(),
        ),
        file_status(
            root,
            "Transactions.duckdb",
            &["Transactions.duckdb", "transactions.duckdb"],
            "transaction store".to_string(),
        ),
    ];

    Ok(rows)
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingsDatabaseOperationResult {
    pub ok: bool,
    pub message: String,
    pub backup_path: Option<String>,
    pub rows: Vec<SettingsDatabaseStatusRow>,
}

fn db_operation_result(
    message: impl Into<String>,
    backup_path: Option<PathBuf>,
) -> Result<SettingsDatabaseOperationResult, String> {
    Ok(SettingsDatabaseOperationResult {
        ok: true,
        message: message.into(),
        backup_path: backup_path.map(|p| p.display().to_string()),
        rows: kgw_settings_database_status()?,
    })
}

fn database_paths_for_root(root: &Path) -> Result<kaspa_gateway_db::DatabasePaths, String> {
    kaspa_gateway_db::DatabasePaths::new(root).map_err(|error| error.to_string())
}

fn backup_all_databases_with_label(label: &str) -> Result<PathBuf, String> {
    crate::db_state::with_database_manager("settings.db.backup", |manager| {
        kaspa_gateway_db::backup_restore::backup_databases(&manager.paths().root, label)
            .map_err(|error| error.to_string())
    })
}

fn parse_database_kind(value: &str) -> Result<kaspa_gateway_db::DatabaseKind, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "appdata" | "app_data" | "app-data" | "appdata.duckdb" => {
            Ok(kaspa_gateway_db::DatabaseKind::AppData)
        }
        "addresses" | "addresses.duckdb" => Ok(kaspa_gateway_db::DatabaseKind::Addresses),
        "transactions" | "transactions.duckdb" | "transactions.sqlite" => {
            Ok(kaspa_gateway_db::DatabaseKind::Transactions)
        }
        other => Err(format!("unknown database kind: {other}")),
    }
}

#[tauri::command]
pub fn kgw_settings_database_compact() -> Result<SettingsDatabaseOperationResult, String> {
    crate::db_state::with_database_manager("settings.db.compact", |manager| {
        manager.compact_all().map_err(|error| error.to_string())
    })?;

    db_operation_result("Database compact completed.", None)
}

#[tauri::command]
pub fn kgw_settings_database_clear_caches() -> Result<SettingsDatabaseOperationResult, String> {
    let removed = crate::db_state::with_database_manager("settings.db.clear_caches", |manager| {
        let cache = manager
            .app_cache_repository()
            .map_err(|error| error.to_string())?;
        cache.clear().map_err(|error| error.to_string())
    })?;

    db_operation_result(format!("Cache rows cleared: {removed}."), None)
}

#[tauri::command]
pub fn kgw_settings_database_backup() -> Result<SettingsDatabaseOperationResult, String> {
    let backup = backup_all_databases_with_label("kgw-db-backup")?;
    db_operation_result("Database backup completed.", Some(backup))
}

#[tauri::command]
pub fn kgw_settings_database_restore_latest() -> Result<SettingsDatabaseOperationResult, String> {
    crate::db_state::try_with_database_root("settings.db.restore", |root| {
        let restored = kaspa_gateway_db::backup_restore::restore_latest(root)
            .map_err(|error| error.to_string())?;
        let manager = kaspa_gateway_db::DatabaseManager::new(database_paths_for_root(root)?);
        let rows = database_status_for_manager(&manager).map_err(|error| {
            format!(
                "Restore data verified, but status refresh failed: {error}; safety backup: {}",
                restored.safety_backup.display()
            )
        })?;
        let mut message = format!(
            "Database restored and verified from: {}. Safety backup: {}",
            restored.restored_from.display(),
            restored.safety_backup.display()
        );
        if let Some(warning) = restored.cleanup_warning {
            message.push_str(&format!(". {warning}"));
        }
        Ok(SettingsDatabaseOperationResult {
            ok: true,
            message,
            backup_path: Some(restored.restored_from.display().to_string()),
            rows,
        })
    })
}

#[tauri::command]
pub fn kgw_settings_database_delete(
    database: String,
) -> Result<SettingsDatabaseOperationResult, String> {
    let kind = parse_database_kind(&database)?;
    let backup = backup_all_databases_with_label("kgw-pre-delete-backup")?;

    crate::db_state::with_database_manager("settings.db.delete", |manager| {
        manager
            .delete_and_reinitialize_database(kind)
            .map_err(|error| error.to_string())
    })?;

    db_operation_result(
        format!("Database deleted and reinitialized: {database}."),
        Some(backup),
    )
}
