use super::*;
use crate::{AddressRecord, DatabaseManager, DatabasePaths, TransactionRecord};
use std::sync::atomic::{AtomicU64, Ordering};
const ADDRESS: &str = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);
fn fixture(label: &str) -> PathBuf {
    let root = loop {
        let sequence = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate = std::env::temp_dir().join(format!(
            "kgw_aud001_{}_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            sequence
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("failed to create AUD-001 fixture directory: {error}"),
        }
    };
    seed(&root, label);
    root
}
fn seed(root: &Path, label: &str) {
    let manager = DatabaseManager::new(DatabasePaths::new(root).unwrap());
    manager.initialize_all().unwrap();
    manager
        .addresses_repository()
        .unwrap()
        .upsert(&AddressRecord::new(ADDRESS, label, "mainnet").unwrap())
        .unwrap();
    manager
        .open_app_data()
        .unwrap()
        .execute(
            "INSERT OR REPLACE INTO app_settings VALUES ('AUD001', ?, 1)",
            [label],
        )
        .unwrap();
    let amount = if label == "OLD" { 111_i64 } else { 999_i64 };
    manager.open_transactions().unwrap().execute("INSERT OR REPLACE INTO transactions (txid,address,tx_type,direction,amount_sompi,timestamp_ms,created_at_ms,updated_at_ms) VALUES ('AUD001',?,'transfer','incoming',?,1,1,1)", duckdb::params![ADDRESS, amount]).unwrap();
    let mut transaction =
        TransactionRecord::new("AUD001", ADDRESS, "transfer", "incoming", amount).unwrap();
    transaction.raw_json = Some("{}".to_owned());
    manager
        .transactions_repository()
        .unwrap()
        .upsert(&transaction)
        .unwrap();
}
fn values(root: &Path) -> Vec<String> {
    let mut result = Vec::new();
    for (name, query) in [
        (
            "AppData.duckdb",
            "SELECT value FROM source.app_settings WHERE key='AUD001'",
        ),
        ("Addresses.duckdb", "SELECT name FROM source.addresses"),
        (
            "Transactions.duckdb",
            "SELECT CAST(amount_sompi AS VARCHAR) FROM source.transactions",
        ),
    ] {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(&format!(
                "ATTACH {} AS source (READ_ONLY)",
                path_sql(&root.join(name)).unwrap()
            ))
            .unwrap();
        result.push(connection.query_row(query, [], |row| row.get(0)).unwrap());
    }
    let sqlite = rusqlite::Connection::open_with_flags(
        root.join("Transactions.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    result.push(
        sqlite
            .query_row(
                "SELECT CAST(amount_sompi AS TEXT) FROM transactions",
                [],
                |row| row.get(0),
            )
            .unwrap(),
    );
    result
}
fn bytes(root: &Path) -> Vec<Vec<u8>> {
    DATABASES
        .iter()
        .map(|name| fs::read(root.join(name)).unwrap())
        .collect()
}
fn prepared() -> (PathBuf, PathBuf) {
    let root = fixture("OLD");
    let old = backup_databases(&root, "kgw-db-backup").unwrap();
    seed(&root, "NEW");
    (root, old)
}
#[test]
fn aud001_old_new_restore_reopen_and_preserve_both_snapshots() {
    let (root, old) = prepared();
    let original = bytes(&old);
    let outcome = restore_latest(&root).unwrap();
    assert_eq!(outcome.restored_from, old.canonicalize().unwrap());
    assert_ne!(outcome.restored_from, outcome.safety_backup);
    assert_eq!(values(&root), ["OLD", "OLD", "111", "111"]);
    assert_eq!(values(&outcome.safety_backup), ["NEW", "NEW", "999", "999"]);
    assert_eq!(bytes(&old), original);
    let safety = bytes(&outcome.safety_backup);
    assert_eq!(values(&root), values(&old)); // Independent reopened engine connections.
    assert_eq!(bytes(&outcome.safety_backup), safety);
}
#[test]
fn aud001_no_previous_backup_never_creates_current_attempt_target() {
    let root = fixture("NEW");
    assert!(restore_latest(&root).is_err());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
    assert!(!root.join("backups").exists());
}
#[test]
fn aud001_corrupt_selected_snapshot_preserves_new_data() {
    let (root, old) = prepared();
    fs::write(
        old.join("Addresses.duckdb"),
        b"AUD001 intentionally corrupt test backup",
    )
    .unwrap();
    assert!(restore_latest(&root).is_err());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
    assert_eq!(fs::read_dir(root.join("backups")).unwrap().count(), 1);
}
#[test]
fn aud001_incompatible_selected_schema_is_rejected() {
    let (root, old) = prepared();
    Connection::open(old.join("AppData.duckdb"))
        .unwrap()
        .execute_batch("UPDATE schema_migrations SET version=999")
        .unwrap();
    assert!(restore_latest(&root).is_err());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
}
#[test]
fn aud001_disappeared_target_is_not_reselected() {
    let (root, old) = prepared();
    let error = restore_with_hook(&root, |phase| {
        if phase == Phase::Pinned {
            fs::rename(&old, root.join("removed-test-snapshot"))?;
        }
        Ok(())
    })
    .unwrap_err();
    assert!(!error.to_string().is_empty());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
    assert_eq!(fs::read_dir(root.join("backups")).unwrap().count(), 0);
}
#[test]
fn aud001_changed_target_after_safety_keeps_new_and_safety() {
    let (root, old) = prepared();
    let result = restore_with_hook(&root, |phase| {
        if phase == Phase::SafetyCreated {
            fs::write(old.join("Addresses.duckdb"), b"changed after pin")?;
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
    let safety = latest_backup(&root).unwrap();
    assert_ne!(safety, old);
    assert_eq!(values(&safety), values(&root));
}
#[test]
fn aud001_safety_snapshot_creation_failure_preserves_current_data() {
    let (root, _) = prepared();
    let result = restore_with_hook(&root, |phase| {
        if phase == Phase::BeforeSafety {
            fs::rename(root.join("backups"), root.join("test-retained-backups"))?;
            fs::write(
                root.join("backups"),
                b"test-owned obstruction to safety-directory creation",
            )?;
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
}
#[test]
fn aud001_partial_native_restore_failure_recovers_new_without_success() {
    let (root, old) = prepared();
    let original = bytes(&old);
    let error = restore_with_hook(&root, |phase| {
        if phase == Phase::Applied(1) {
            return Err(invalid("controlled failure after two committed databases"));
        }
        Ok(())
    })
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("current data recovered and verified"),
        "{error}"
    );
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
    assert_eq!(bytes(&old), original);
    assert_eq!(values(&latest_backup(&root).unwrap()), values(&root));
}
#[test]
fn aud001_historical_safety_snapshot_remains_eligible() {
    let root = fixture("OLD");
    let old = backup_inner(&root, "kgw-pre-restore-backup").unwrap();
    seed(&root, "NEW");
    let result = restore_latest(&root).unwrap();
    assert_eq!(result.restored_from, old.canonicalize().unwrap());
    assert_eq!(values(&root), ["OLD", "OLD", "111", "111"]);
}
#[test]
fn aud001_incomplete_snapshot_is_rejected_without_partial_restore() {
    let (root, old) = prepared();
    fs::rename(
        old.join("Transactions.sqlite"),
        root.join("test-retained-transactions.sqlite"),
    )
    .unwrap();
    assert!(restore_latest(&root).is_err());
    assert_eq!(values(&root), ["NEW", "NEW", "999", "999"]);
}
#[test]
fn aud001_overlapping_attempt_is_rejected_not_queued() {
    let (root, _) = prepared();
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let worker_root = root.clone();
    let worker = std::thread::spawn(move || {
        restore_with_hook(&worker_root, |phase| {
            if phase == Phase::Pinned {
                ready_tx.send(()).unwrap();
                release_rx
                    .recv_timeout(std::time::Duration::from_secs(30))
                    .unwrap();
            }
            Ok(())
        })
    });
    ready_rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .unwrap();
    let rejected = restore_latest(&root).unwrap_err();
    assert!(rejected.to_string().contains("already in progress"));
    release_tx.send(()).unwrap();
    worker.join().unwrap().unwrap();
    assert_eq!(values(&root), ["OLD", "OLD", "111", "111"]);
}

#[test]
#[ignore = "requires an explicitly marked synthetic AUD-001 GUI evidence directory"]
fn aud001_read_only_gui_snapshot_probe() {
    let scope = PathBuf::from(std::env::var("KGW_AUD001_PROBE_SCOPE").expect("synthetic scope"))
        .canonicalize()
        .unwrap();
    assert_eq!(
        fs::read_to_string(scope.join("AUD001_SYNTHETIC")).unwrap(),
        "functional-surface-ee92134/AUD001"
    );
    for (key, expected) in [
        ("CURRENT", "AUD001_OLD"),
        ("OLD", "AUD001_OLD"),
        ("SAFETY", "AUD001_NEW"),
    ] {
        let root = PathBuf::from(
            std::env::var(format!("KGW_AUD001_PROBE_{key}")).expect("explicit snapshot path"),
        )
        .canonicalize()
        .unwrap();
        assert!(
            root.starts_with(&scope),
            "probe cannot leave its synthetic scope"
        );
        let before = inventory(&root).ok();
        for name in database_names(&root).unwrap() {
            if name.ends_with(".sqlite") {
                let connection = rusqlite::Connection::open_with_flags(
                    root.join(&name),
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )
                .unwrap();
                sqlite_signature(&connection, "main").unwrap();
            } else {
                let connection = Connection::open_in_memory().unwrap();
                connection
                    .execute_batch(&format!(
                        "ATTACH {} AS source (READ_ONLY)",
                        path_sql(&root.join(&name)).unwrap()
                    ))
                    .unwrap();
                duck_signature(&connection, "source", &name).unwrap();
                if name == "Addresses.duckdb" {
                    let label: String = connection
                        .query_row("SELECT name FROM source.addresses", [], |row| row.get(0))
                        .unwrap();
                    assert_eq!(label, expected);
                }
            }
        }
        if let Some(before) = before {
            assert_eq!(inventory(&root).unwrap(), before);
        }
        println!("AUD001_NATIVE_PROBE_{key}={expected};SCHEMA_AND_INTEGRITY=PASS");
    }
}
