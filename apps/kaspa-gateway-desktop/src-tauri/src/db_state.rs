use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths, TransactionsRepository};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

static DB_OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock() -> &'static Mutex<()> {
    DB_OPERATION_LOCK.get_or_init(|| Mutex::new(()))
}

pub fn database_root() -> Result<PathBuf, String> {
    Ok(default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("databases"))
}

fn build_manager_unlocked() -> Result<DatabaseManager, String> {
    let root = database_root()?;
    let paths = DatabasePaths::new(root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths);

    manager
        .initialize_all()
        .map_err(|error| error.to_string())?;

    Ok(manager)
}

pub fn with_database_manager<T, F>(target: &str, operation: F) -> Result<T, String>
where
    F: FnOnce(&DatabaseManager) -> Result<T, String>,
{
    let _guard = lock()
        .lock()
        .map_err(|_| format!("{target}: database operation lock is poisoned"))?;

    let manager = build_manager_unlocked()
        .map_err(|error| format!("{target}: database manager failed: {error}"))?;

    operation(&manager)
}

pub fn transactions_repository_unlocked() -> Result<TransactionsRepository, String> {
    let root = database_root()?;
    let paths = DatabasePaths::new(root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths);

    manager
        .transactions_repository()
        .map_err(|error| error.to_string())
}

/// Non-queuing entry for restore: an overlapping request must not become a new restore.
pub fn try_with_database_root<T>(
    target: &str,
    operation: impl FnOnce(&Path) -> Result<T, String>,
) -> Result<T, String> {
    let _guard = lock()
        .try_lock()
        .map_err(|error| format!("{target}: database operation is busy or unavailable: {error}"))?;
    let root = database_root()?;
    operation(&root)
}
