use kaspa_gateway_db;
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
    let root = crate::db_state::database_root()?;

    let address_count =
        crate::db_state::with_database_manager("settings.db_status.addresses", |manager| {
            let repository = manager
                .addresses_repository()
                .map_err(|error| error.to_string())?;

            let count = repository.list().map_err(|error| error.to_string())?.len();

            Ok(count)
        })
        .unwrap_or(0);

    let rows = vec![
        file_status(
            &root,
            "Addresses.duckdb",
            &["Addresses.duckdb", "addresses.duckdb"],
            format!("{address_count} addresses"),
        ),
        file_status(
            &root,
            "AppData.duckdb",
            &["AppData.duckdb", "appdata.duckdb", "app_data.duckdb"],
            "application settings / app data".to_string(),
        ),
        file_status(
            &root,
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

fn now_ms_string() -> String {
    UNIX_EPOCH
        .elapsed()
        .map(|duration| duration.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn database_paths_for_root(root: &Path) -> Result<kaspa_gateway_db::DatabasePaths, String> {
    kaspa_gateway_db::DatabasePaths::new(root).map_err(|error| error.to_string())
}

fn copy_if_exists(source: &Path, destination_dir: &Path) -> Result<(), String> {
    if !source.exists() {
        return Ok(());
    }

    fs::create_dir_all(destination_dir).map_err(|error| error.to_string())?;

    let file_name = source
        .file_name()
        .ok_or_else(|| format!("invalid database path: {}", source.display()))?;

    fs::copy(source, destination_dir.join(file_name)).map_err(|error| error.to_string())?;
    Ok(())
}

fn backup_all_databases_with_label(label: &str) -> Result<PathBuf, String> {
    let root = crate::db_state::database_root()?;
    let paths = database_paths_for_root(&root)?;
    let backup_dir = root
        .join("backups")
        .join(format!("{}-{}", label, now_ms_string()));

    fs::create_dir_all(&backup_dir).map_err(|error| error.to_string())?;

    for file in [
        &paths.app_data,
        &paths.addresses,
        &paths.transactions,
        &paths.transactions_sqlite,
    ] {
        copy_if_exists(file, &backup_dir)?;
        let wal = PathBuf::from(format!("{}.wal", file.display()));
        copy_if_exists(&wal, &backup_dir)?;
    }

    Ok(backup_dir)
}

fn latest_backup_dir() -> Result<PathBuf, String> {
    let root = crate::db_state::database_root()?;
    let backup_root = root.join("backups");

    let mut dirs = fs::read_dir(&backup_root)
        .map_err(|error| format!("backup directory not found: {error}"))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();

    dirs.sort();

    dirs.pop()
        .ok_or_else(|| "no database backup directories found".to_string())
}

fn restore_backup_dir(backup_dir: &Path) -> Result<(), String> {
    let root = crate::db_state::database_root()?;
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    for entry in fs::read_dir(backup_dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source = entry.path();

        if !source.is_file() {
            continue;
        }

        let Some(file_name) = source.file_name() else {
            continue;
        };

        fs::copy(&source, root.join(file_name)).map_err(|error| error.to_string())?;
    }

    Ok(())
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
    let safety = backup_all_databases_with_label("kgw-pre-restore-backup")?;
    let latest = latest_backup_dir()?;
    restore_backup_dir(&latest)?;
    db_operation_result(
        format!(
            "Database restored from latest backup. Safety backup: {}",
            safety.display()
        ),
        Some(latest),
    )
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
