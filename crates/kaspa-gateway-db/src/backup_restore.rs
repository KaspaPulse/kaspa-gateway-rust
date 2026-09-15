//! AUD-001: pinned backup selection and native, compensating database restore.
use crate::{DATABASE_SCHEMA_VERSION, DbError, Result};
use duckdb::Connection;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const DATABASES: [&str; 4] = [
    "AppData.duckdb",
    "Addresses.duckdb",
    "Transactions.duckdb",
    "Transactions.sqlite",
];
static ACTIVE_ROOTS: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
fn invalid(message: impl Into<String>) -> DbError {
    DbError::InvalidRecord(message.into())
}
fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
fn identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
fn path_sql(path: &Path) -> Result<String> {
    let value = path
        .to_str()
        .ok_or_else(|| invalid("database path is not UTF-8"))?;
    if value.contains('\0') {
        return Err(invalid("database path contains NUL"));
    }
    Ok(sql_string(value))
}
struct RootGuard(PathBuf);
impl Drop for RootGuard {
    fn drop(&mut self) {
        if let Ok(mut roots) = ACTIVE_ROOTS.get_or_init(Default::default).lock() {
            roots.remove(&self.0);
        }
    }
}
fn enter(root: &Path) -> Result<RootGuard> {
    let root = root.canonicalize()?;
    let mut roots = ACTIVE_ROOTS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| invalid("database restore ownership lock poisoned"))?;
    if !roots.insert(root.clone()) {
        return Err(invalid("database backup/restore already in progress"));
    }
    Ok(RootGuard(root))
}
fn work_directory(root: &Path) -> Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| invalid(e.to_string()))?
        .as_nanos();
    let path = root.join(format!(".kgw-restore-work-{}-{stamp}", std::process::id()));
    fs::create_dir(&path)?;
    Ok(path)
}
fn database_names(root: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for (index, name) in DATABASES.iter().enumerate() {
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_file() => names.push((*name).to_owned()),
            Ok(_) => {
                return Err(invalid(format!(
                    "not a regular database file: {}",
                    path.display()
                )));
            }
            Err(error) if index == 3 && error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(invalid(format!(
                    "required database {}: {error}",
                    path.display()
                )));
            }
        }
    }
    Ok(names)
}
fn tables(name: &str) -> &'static [&'static str] {
    match name {
        "AppData.duckdb" => &[
            "app_settings",
            "cache",
            "known_names",
            "schema_migrations",
            "user_state",
        ],
        "Addresses.duckdb" => &["addresses", "schema_migrations"],
        _ => &["schema_migrations", "transactions"],
    }
}
fn native_backup(source: &Path, destination: &Path, name: &str) -> Result<()> {
    if name.ends_with(".sqlite") {
        let connection = rusqlite::Connection::open_with_flags(
            source,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        connection.execute("VACUUM INTO ?1", [destination.to_string_lossy().as_ref()])?;
    } else {
        let connection = Connection::open_in_memory()?;
        connection.execute_batch(&format!("ATTACH {} AS aud_live (READ_ONLY); ATTACH {} AS aud_snapshot; COPY FROM DATABASE aud_live TO aud_snapshot; DETACH aud_snapshot; DETACH aud_live;", path_sql(source)?, path_sql(destination)?))?;
    }
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(destination)?
        .sync_all()?;
    Ok(())
}
fn duck_signature(connection: &Connection, alias: &str, name: &str) -> Result<Vec<String>> {
    let mut statement = connection.prepare(&format!("SELECT table_name FROM information_schema.tables WHERE table_catalog={} AND table_schema='main' AND table_type='BASE TABLE' ORDER BY table_name", sql_string(alias)))?;
    let actual = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if actual != tables(name) {
        return Err(invalid(format!("incompatible table set in {name}")));
    }
    let version: i64 = connection.query_row(
        &format!("SELECT COALESCE(MAX(version),0) FROM {alias}.main.schema_migrations"),
        [],
        |row| row.get(0),
    )?;
    if version != DATABASE_SCHEMA_VERSION {
        return Err(invalid(format!(
            "incompatible schema version in {name}: {version}"
        )));
    }
    let mut statement = connection.prepare(&format!("SELECT table_name || ':' || column_name || ':' || data_type || ':' || is_nullable FROM information_schema.columns WHERE table_catalog={} AND table_schema='main' ORDER BY table_name, ordinal_position", sql_string(alias)))?;
    Ok(statement
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?)
}
fn duck_pair(live: &Path, snapshot: &Path) -> Result<Connection> {
    let connection = Connection::open_in_memory()?;
    connection.execute_batch(&format!(
        "ATTACH {} AS aud_live; ATTACH {} AS aud_snapshot (READ_ONLY);",
        path_sql(live)?,
        path_sql(snapshot)?
    ))?;
    Ok(connection)
}
fn sqlite_pair(live: &Path, snapshot: &Path) -> Result<rusqlite::Connection> {
    let connection =
        rusqlite::Connection::open_with_flags(live, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    connection.busy_timeout(std::time::Duration::from_secs(5))?;
    connection.execute(
        "ATTACH DATABASE ?1 AS aud_snapshot",
        [snapshot.to_string_lossy().as_ref()],
    )?;
    Ok(connection)
}
fn sqlite_signature(connection: &rusqlite::Connection, alias: &str) -> Result<Vec<String>> {
    let integrity: String =
        connection.query_row(&format!("PRAGMA {alias}.integrity_check"), [], |row| {
            row.get(0)
        })?;
    if integrity != "ok" {
        return Err(invalid(format!(
            "SQLite integrity check failed: {integrity}"
        )));
    }
    let version: i64 = connection.query_row(
        &format!("SELECT COALESCE(MAX(version),0) FROM {alias}.schema_migrations"),
        [],
        |row| row.get(0),
    )?;
    if version != DATABASE_SCHEMA_VERSION {
        return Err(invalid("incompatible SQLite schema version"));
    }
    let mut statement = connection.prepare(&format!("SELECT name FROM {alias}.sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"))?;
    let actual = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if actual != tables("Transactions.sqlite") {
        return Err(invalid("incompatible SQLite table set"));
    }
    let mut signature = Vec::new();
    for table in tables("Transactions.sqlite") {
        let mut statement =
            connection.prepare(&format!("PRAGMA {alias}.table_info({})", identifier(table)))?;
        for row in statement.query_map([], |row| {
            Ok(format!(
                "{}:{}:{}:{}:{:?}",
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<String>>(4)?
            ))
        })? {
            signature.push(format!("{table}:{}", row?));
        }
    }
    Ok(signature)
}
fn difference_sql(alias: &str, table: &str, all: bool) -> String {
    let table = identifier(table);
    let op = if all { "EXCEPT ALL" } else { "EXCEPT" };
    format!(
        "SELECT COUNT(*) FROM (SELECT * FROM (SELECT * FROM {alias}.{table} {op} SELECT * FROM aud_snapshot.{table}) UNION ALL SELECT * FROM (SELECT * FROM aud_snapshot.{table} {op} SELECT * FROM {alias}.{table}))"
    )
}
#[derive(Clone, Copy, PartialEq)]
enum Action {
    Validate,
    Replace,
    Verify,
}
fn operate(live: &Path, snapshot: &Path, name: &str, action: Action) -> Result<()> {
    if name.ends_with(".sqlite") {
        let connection = sqlite_pair(live, snapshot)?;
        if sqlite_signature(&connection, "main")? != sqlite_signature(&connection, "aud_snapshot")?
        {
            return Err(invalid("incompatible SQLite columns"));
        }
        if action == Action::Validate {
            return Ok(());
        }
        if action == Action::Replace {
            connection.execute_batch("BEGIN IMMEDIATE;")?;
        }
        let result = (|| -> Result<()> {
            for table in tables(name) {
                if action == Action::Replace {
                    connection.execute_batch(&format!("DELETE FROM main.{0}; INSERT INTO main.{0} SELECT * FROM aud_snapshot.{0};", identifier(table)))?;
                }
                let count: i64 =
                    connection
                        .query_row(&difference_sql("main", table, false), [], |row| row.get(0))?;
                if count != 0 {
                    return Err(invalid(format!("restored data mismatch: {name}/{table}")));
                }
            }
            if action == Action::Replace {
                connection.execute_batch("COMMIT;")?;
            }
            Ok(())
        })();
        if result.is_err() && action == Action::Replace {
            let _ = connection.execute_batch("ROLLBACK;");
        }
        result
    } else {
        let connection = duck_pair(live, snapshot)?;
        if duck_signature(&connection, "aud_live", name)?
            != duck_signature(&connection, "aud_snapshot", name)?
        {
            return Err(invalid(format!("incompatible columns in {name}")));
        }
        if action == Action::Replace {
            connection.execute_batch("BEGIN TRANSACTION;")?;
        }
        let result = (|| -> Result<()> {
            for table in tables(name) {
                if action == Action::Validate {
                    let _: i64 = connection.query_row(
                        &format!(
                            "SELECT COUNT(*) FROM (SELECT DISTINCT * FROM aud_snapshot.{})",
                            identifier(table)
                        ),
                        [],
                        |row| row.get(0),
                    )?;
                    continue;
                }
                if action == Action::Replace {
                    connection.execute_batch(&format!("DELETE FROM aud_live.{0}; INSERT INTO aud_live.{0} SELECT * FROM aud_snapshot.{0};", identifier(table)))?;
                }
                let count: i64 =
                    connection.query_row(&difference_sql("aud_live", table, true), [], |row| {
                        row.get(0)
                    })?;
                if count != 0 {
                    return Err(invalid(format!("restored data mismatch: {name}/{table}")));
                }
            }
            if action == Action::Replace {
                connection.execute_batch("COMMIT;")?;
            }
            Ok(())
        })();
        if result.is_err() && action == Action::Replace {
            let _ = connection.execute_batch("ROLLBACK;");
        }
        result
    }
}
fn backup_inner(root: &Path, label: &str) -> Result<PathBuf> {
    let names = database_names(root)?;
    let temporary = work_directory(root)?;
    let snapshot = temporary.join("snapshot");
    fs::create_dir(&snapshot)?;
    for name in &names {
        native_backup(&root.join(name), &snapshot.join(name), name)?;
        operate(&root.join(name), &snapshot.join(name), name, Action::Verify)?;
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| invalid(e.to_string()))?
        .as_millis();
    let backup_root = root.join("backups");
    fs::create_dir_all(&backup_root)?;
    let destination = backup_root.join(format!("{label}-{stamp}"));
    if destination.exists() {
        return Err(invalid(
            "backup destination already exists; nothing overwritten",
        ));
    }
    fs::rename(&snapshot, &destination)?;
    fs::remove_dir(&temporary)?;
    Ok(destination)
}
/// Call under the application's database-operation ownership boundary.
pub fn backup_databases(root: &Path, label: &str) -> Result<PathBuf> {
    if !matches!(label, "kgw-db-backup" | "kgw-pre-delete-backup") {
        return Err(invalid("unsupported backup label"));
    }
    let _guard = enter(root)?;
    backup_inner(root, label)
}
fn latest_backup(root: &Path) -> Result<PathBuf> {
    let mut directories = Vec::new();
    for entry in fs::read_dir(root.join("backups"))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            directories.push(entry.path());
        }
    }
    // Preserve the established ordering and historical safety-backup eligibility.
    directories.sort();
    directories
        .pop()
        .ok_or_else(|| invalid("no database backup directories found"))
}
fn same_bytes(left: &Path, right: &Path) -> Result<bool> {
    let mut left = File::open(left)?;
    let mut right = File::open(right)?;
    let mut remaining = left.metadata()?.len();
    if remaining != right.metadata()?.len() {
        return Ok(false);
    }
    let (mut a, mut b) = ([0_u8; 65536], [0_u8; 65536]);
    while remaining > 0 {
        let size = remaining.min(a.len() as u64) as usize;
        left.read_exact(&mut a[..size])?;
        right.read_exact(&mut b[..size])?;
        if a[..size] != b[..size] {
            return Ok(false);
        }
        remaining -= size as u64;
    }
    Ok(true)
}
fn inventory(directory: &Path) -> Result<Vec<(String, u64, SystemTime)>> {
    if !fs::symlink_metadata(directory)?.file_type().is_dir() {
        return Err(invalid("backup directory identity changed"));
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .to_str()
            .ok_or_else(|| invalid("invalid backup filename"))?
            .to_owned();
        let allowed = DATABASES.iter().any(|db| {
            name == *db
                || name == format!("{db}.wal")
                || (db.ends_with(".sqlite")
                    && (name == format!("{db}-wal") || name == format!("{db}-shm")))
        });
        let metadata = fs::symlink_metadata(entry.path())?;
        if !allowed || !metadata.file_type().is_file() {
            return Err(invalid(format!("unexpected backup member: {name}")));
        }
        entries.push((name, metadata.len(), metadata.modified()?));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}
struct PinnedBackup {
    path: PathBuf,
    work: PathBuf,
    original_bytes: PathBuf,
    source: PathBuf,
    inventory: Vec<(String, u64, SystemTime)>,
}
impl PinnedBackup {
    fn unchanged(&self) -> Result<()> {
        if self.path.canonicalize()? != self.path || inventory(&self.path)? != self.inventory {
            return Err(invalid(
                "selected backup changed; restore aborted without reselection",
            ));
        }
        for (name, _, _) in &self.inventory {
            if !same_bytes(&self.path.join(name), &self.original_bytes.join(name))? {
                return Err(invalid(
                    "selected backup content changed; restore aborted without reselection",
                ));
            }
        }
        Ok(())
    }
    fn capture(root: &Path, selected: PathBuf) -> Result<Self> {
        let path = selected.canonicalize()?;
        let before = inventory(&path)?;
        let work = work_directory(root)?;
        let original_bytes = work.join("pinned-bytes");
        let source = work.join("restore-source");
        fs::create_dir(&original_bytes)?;
        fs::create_dir(&source)?;
        // These are closed backup files, never files from an open live database.
        for (name, _, _) in &before {
            fs::copy(path.join(name), original_bytes.join(name))?;
            fs::copy(original_bytes.join(name), source.join(name))?;
        }
        let pinned = Self {
            path,
            work,
            original_bytes,
            source,
            inventory: before,
        };
        pinned.unchanged()?;
        Ok(pinned)
    }
}
#[derive(Debug)]
pub struct RestoreOutcome {
    pub restored_from: PathBuf,
    pub safety_backup: PathBuf,
    pub cleanup_warning: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Pinned,
    BeforeSafety,
    SafetyCreated,
    Applied(usize),
}
/// Restores one pinned pre-attempt snapshot. No fallback selection is performed.
pub fn restore_latest(root: &Path) -> Result<RestoreOutcome> {
    restore_with_hook(root, |_| Ok(()))
}
fn restore_with_hook(
    root: &Path,
    mut hook: impl FnMut(Phase) -> Result<()>,
) -> Result<RestoreOutcome> {
    let _guard = enter(root)?;
    let selected = latest_backup(root)?;
    let pinned = PinnedBackup::capture(root, selected)?;
    let names = database_names(root)?;
    if database_names(&pinned.source)? != names {
        return Err(invalid(
            "backup database set is incompatible with the current database set",
        ));
    }
    hook(Phase::Pinned)?;
    pinned.unchanged()?;
    for name in &names {
        operate(
            &root.join(name),
            &pinned.source.join(name),
            name,
            Action::Validate,
        )?;
    }
    hook(Phase::BeforeSafety)?;
    let safety = backup_inner(root, "kgw-pre-restore-backup")?;
    hook(Phase::SafetyCreated)?;
    pinned.unchanged()?;
    let mut attempted = Vec::new();
    let result = (|| -> Result<()> {
        for (index, name) in names.iter().enumerate() {
            attempted.push(name);
            operate(
                &root.join(name),
                &pinned.source.join(name),
                name,
                Action::Replace,
            )?;
            hook(Phase::Applied(index))?;
        }
        for name in &names {
            operate(
                &root.join(name),
                &pinned.source.join(name),
                name,
                Action::Verify,
            )?;
        }
        pinned.unchanged()?;
        Ok(())
    })();
    if let Err(error) = result {
        let mut recovery_errors = Vec::new();
        for name in attempted.into_iter().rev() {
            if let Err(recovery) =
                operate(&root.join(name), &safety.join(name), name, Action::Replace)
            {
                recovery_errors.push(format!("{name}: {recovery}"));
            }
        }
        for name in &names {
            if let Err(recovery) =
                operate(&root.join(name), &safety.join(name), name, Action::Verify)
            {
                recovery_errors.push(format!("verification {name}: {recovery}"));
            }
        }
        let recovery = if recovery_errors.is_empty() {
            "current data recovered and verified".to_owned()
        } else {
            format!("RECOVERY FAILED: {}", recovery_errors.join("; "))
        };
        return Err(invalid(format!(
            "Restore failed: {error}; {recovery}; retained safety backup: {}; diagnostic workspace: {}",
            safety.display(),
            pinned.work.display()
        )));
    }
    let cleanup_warning = fs::remove_dir_all(&pinned.work).err().map(|error| {
        format!("restored and verified, but temporary workspace cleanup failed: {error}")
    });
    Ok(RestoreOutcome {
        restored_from: pinned.path,
        safety_backup: safety,
        cleanup_warning,
    })
}

#[cfg(test)]
#[path = "backup_restore_tests.rs"]
mod tests;
