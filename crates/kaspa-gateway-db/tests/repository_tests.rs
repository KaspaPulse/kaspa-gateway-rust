use kaspa_gateway_db::{
    AddressRecord, AppCacheRepository, AppSettingsRepository, DatabaseManager, DatabasePaths,
    TransactionFilter, TransactionRecord, TransactionsRepository,
};
use std::path::PathBuf;
use std::time::Duration;

fn unique_test_dir(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time valid")
        .as_nanos();

    std::env::temp_dir().join(format!("kaspa_gateway_repo_{name}_{stamp}"))
}

fn test_manager(name: &str) -> DatabaseManager {
    let root = unique_test_dir(name);
    let paths = DatabasePaths::new(root).expect("paths must be valid");
    let manager = DatabaseManager::new(paths);
    manager.initialize_all().expect("schemas must initialize");
    manager
}

#[test]
fn addresses_repository_upserts_gets_lists_and_deletes() {
    let manager = test_manager("addresses_repository");
    let repository = manager
        .addresses_repository()
        .expect("addresses repository must open");

    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let record =
        AddressRecord::new(address, "Primary Wallet", "mainnet").expect("record must be valid");

    repository.upsert(&record).expect("upsert must work");

    let saved = repository
        .get(address)
        .expect("get must work")
        .expect("record must exist");

    assert_eq!(saved.address, address);
    assert_eq!(saved.name, "Primary Wallet");
    assert_eq!(repository.count().expect("count"), 1);

    let updated =
        AddressRecord::new(address, "Updated Wallet", "mainnet").expect("record must be valid");

    repository
        .upsert(&updated)
        .expect("second upsert must work");

    let saved = repository
        .get(address)
        .expect("get must work")
        .expect("record must exist");

    assert_eq!(saved.name, "Updated Wallet");

    let list = repository.list().expect("list must work");
    assert_eq!(list.len(), 1);

    assert!(repository.delete(address).expect("delete must work"));
    assert_eq!(repository.count().expect("count"), 0);
}

#[test]
fn settings_repository_sets_gets_and_deletes_values() {
    let manager = test_manager("settings_repository");
    let repository: AppSettingsRepository = manager
        .app_settings_repository()
        .expect("settings repository must open");

    repository
        .set("language", "en")
        .expect("setting must be saved");

    assert_eq!(
        repository.get("language").expect("get must work"),
        Some("en".to_string())
    );

    repository
        .set("language", "ar")
        .expect("setting must be updated");

    assert_eq!(
        repository.get("language").expect("get must work"),
        Some("ar".to_string())
    );

    assert!(repository.delete("language").expect("delete must work"));
    assert_eq!(repository.get("language").expect("get must work"), None);
}

#[test]
fn cache_repository_respects_expiration() {
    let manager = test_manager("cache_repository");
    let repository: AppCacheRepository = manager
        .app_cache_repository()
        .expect("cache repository must open");

    repository
        .set("network", r#"{"name":"mainnet"}"#, None)
        .expect("cache must save");

    assert_eq!(
        repository.get("network").expect("cache get"),
        Some(r#"{"name":"mainnet"}"#.to_string())
    );

    repository
        .set("expired", r#"{"old":true}"#, Some(1))
        .expect("expired cache must save");

    assert_eq!(repository.get("expired").expect("expired get"), None);

    let deleted = repository.delete_expired().expect("delete expired");
    assert!(deleted >= 1);
}

#[test]
fn transactions_repository_upserts_lists_counts_and_deletes() {
    let manager = test_manager("transactions_repository");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");

    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let mut record = TransactionRecord::new("tx001", address, "transfer", "incoming", 100_000_000)
        .expect("transaction must be valid");

    record.timestamp_ms = 1000;
    record.raw_json = Some(r#"{"txid":"tx001"}"#.to_string());

    repository.upsert(&record).expect("upsert must work");

    assert_eq!(
        repository
            .count_for_address(address)
            .expect("count must work"),
        1
    );

    let list = repository
        .list_for_address(address, 10)
        .expect("list must work");

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].txid, "tx001");
    assert_eq!(list[0].amount_sompi, 100_000_000);

    let deleted = repository
        .delete_for_address(address)
        .expect("delete must work");

    assert_eq!(deleted, 1);
    assert_eq!(
        repository
            .count_for_address(address)
            .expect("count must work"),
        0
    );
}

#[test]
fn transactions_repository_preserves_same_txid_for_multiple_addresses() {
    let manager = test_manager("transactions_multi_address");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");

    let address_a = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
    let address_b = "kaspa:qq2avyvncscg5dtsk8u4uwjhlr3799dhaqj8k9y6q5y9hpwfxjy6u00pep7vg";

    let mut for_a = TransactionRecord::new("shared-tx", address_a, "transfer", "outgoing", 42)
        .expect("A relation valid");
    for_a.timestamp_ms = 1_000;
    for_a.raw_json = Some(r#"{"txid":"shared-tx"}"#.to_string());

    let mut for_b = TransactionRecord::new("shared-tx", address_b, "transfer", "incoming", 42)
        .expect("B relation valid");
    for_b.timestamp_ms = 1_000;
    for_b.raw_json = Some(r#"{"txid":"shared-tx"}"#.to_string());

    repository.upsert(&for_a).expect("A upsert");
    repository.upsert(&for_b).expect("B upsert");

    assert_eq!(repository.total_count().expect("canonical count"), 1);
    assert_eq!(repository.count_for_address(address_a).expect("A count"), 1);
    assert_eq!(repository.count_for_address(address_b).expect("B count"), 1);
    assert_eq!(
        repository.list_for_address(address_a, 10).expect("A list")[0].address,
        address_a
    );
    assert_eq!(
        repository.list_for_address(address_b, 10).expect("B list")[0].address,
        address_b
    );

    assert_eq!(
        repository.delete_for_address(address_a).expect("delete A"),
        1
    );
    assert_eq!(repository.count_for_address(address_a).expect("A empty"), 0);
    assert_eq!(
        repository.count_for_address(address_b).expect("B survives"),
        1
    );
    assert_eq!(repository.total_count().expect("canonical retained"), 1);

    assert_eq!(
        repository.delete_for_address(address_b).expect("delete B"),
        1
    );
    assert_eq!(
        repository.total_count().expect("orphan canonical removed"),
        0
    );
}

#[test]
fn transactions_repository_rejects_invalid_batch_without_partial_write() {
    let manager = test_manager("transactions_atomic_validation");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let valid = TransactionRecord::new("atomic-valid", address, "transfer", "incoming", 1)
        .expect("valid record");
    let mut invalid = valid.clone();
    invalid.txid = "atomic-invalid".to_string();
    invalid.amount_sompi = -1;

    assert!(repository.upsert_many(&[valid, invalid]).is_err());
    assert_eq!(repository.count_for_address(address).expect("empty"), 0);
    assert_eq!(repository.total_count().expect("no canonical rows"), 0);
}

#[test]
fn transactions_repository_filters_and_summarizes_through_address_relations() {
    let manager = test_manager("transactions_relation_queries");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let mut record = TransactionRecord::new("tx-filter-001", address, "transfer", "incoming", 123)
        .expect("transaction valid");
    record.timestamp_ms = 86_400_000;
    record.counterparty = Some("counterparty-demo".to_string());
    record.raw_json = Some(r#"{"txid":"tx-filter-001"}"#.to_string());
    repository.upsert(&record).expect("upsert");

    let filter = TransactionFilter {
        address,
        start_ms: Some(0),
        end_ms: Some(172_800_000),
        tx_type: Some("transfer"),
        direction: Some("incoming"),
        search: Some("tx-filter"),
        limit: Some(10),
        offset: None,
    };

    assert_eq!(
        repository
            .count_filtered_for_address(filter)
            .expect("filtered count"),
        1
    );
    let rows = repository
        .filter_for_address(filter)
        .expect("filtered rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].address, address);

    let summaries = repository
        .day_summaries_for_address(TransactionFilter {
            search: None,
            ..filter
        })
        .expect("day summaries");
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].count, 1);
    assert_eq!(summaries[0].incoming_sompi, 123);
}

#[test]
fn transaction_filter_offset_paginates_without_overlap() {
    let manager = test_manager("transactions_filter_offset");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    for index in 0..5_i64 {
        let mut record = TransactionRecord::new(
            format!("tx-page-{index:03}"),
            address,
            "transfer",
            "incoming",
            100 + index,
        )
        .expect("transaction valid");
        record.timestamp_ms = index * 1_000;
        record.raw_json = Some(format!(r#"{{"txid":"tx-page-{index:03}"}}"#));
        repository.upsert(&record).expect("upsert");
    }

    let page = |offset| {
        repository
            .filter_for_address(TransactionFilter {
                address,
                start_ms: None,
                end_ms: None,
                tx_type: None,
                direction: None,
                search: None,
                limit: Some(2),
                offset: Some(offset),
            })
            .expect("page")
    };

    let first = page(0);
    let second = page(2);
    let third = page(4);

    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 2);
    assert_eq!(third.len(), 1);

    let txids = first
        .iter()
        .chain(second.iter())
        .chain(third.iter())
        .map(|row| row.txid.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        txids,
        vec![
            "tx-page-004",
            "tx-page-003",
            "tx-page-002",
            "tx-page-001",
            "tx-page-000",
        ]
    );
}

#[test]
fn sqlite_v1_transaction_rows_are_backfilled_into_address_relations() {
    let root = unique_test_dir("transactions_v1_backfill");
    let paths = DatabasePaths::new(&root).expect("paths");
    paths.ensure_root().expect("root");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    {
        let connection =
            rusqlite::Connection::open(&paths.transactions_sqlite).expect("legacy sqlite open");
        connection
            .execute_batch(
                r#"
                CREATE TABLE transactions(
                    txid TEXT PRIMARY KEY,
                    address TEXT NOT NULL,
                    tx_type TEXT NOT NULL,
                    direction TEXT NOT NULL,
                    amount_sompi INTEGER NOT NULL,
                    from_address TEXT,
                    to_address TEXT,
                    counterparty TEXT,
                    block_height INTEGER,
                    timestamp_ms INTEGER NOT NULL,
                    raw_json TEXT NOT NULL,
                    created_at_ms INTEGER NOT NULL,
                    updated_at_ms INTEGER NOT NULL
                );
                "#,
            )
            .expect("legacy schema");
        connection
            .execute(
                r#"
                INSERT INTO transactions(
                    txid, address, tx_type, direction, amount_sompi,
                    from_address, to_address, counterparty, block_height,
                    timestamp_ms, raw_json, created_at_ms, updated_at_ms
                ) VALUES (?1, ?2, 'transfer', 'incoming', 7, NULL, NULL, NULL, NULL, 1000, '{}', 1, 1)
                "#,
                rusqlite::params!["legacy-tx", address],
            )
            .expect("legacy row");
    }

    let manager = DatabaseManager::new(paths);
    let repository = manager
        .transactions_repository()
        .expect("v2 repository opens and migrates");

    assert_eq!(
        repository
            .count_for_address(address)
            .expect("backfilled count"),
        1
    );
    assert_eq!(
        repository
            .list_for_address(address, 10)
            .expect("backfilled list")[0]
            .txid,
        "legacy-tx"
    );
}

#[test]
fn force_refresh_staging_preserves_known_good_until_atomic_promote() {
    let manager = test_manager("transactions_force_refresh_atomic");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let mut old = TransactionRecord::new("force-old", address, "transfer", "incoming", 10)
        .expect("old valid");
    old.timestamp_ms = 1_000;
    old.raw_json = Some(r#"{"txid":"force-old"}"#.to_string());
    repository.upsert(&old).expect("old upsert");

    repository
        .begin_force_refresh_stage(address)
        .expect("stage begins");

    let mut replacement = TransactionRecord::new("force-new", address, "transfer", "incoming", 20)
        .expect("replacement valid");
    replacement.timestamp_ms = 2_000;
    replacement.raw_json = Some(r#"{"txid":"force-new"}"#.to_string());

    assert_eq!(
        repository
            .stage_force_refresh_many(&[replacement])
            .expect("stage page"),
        1
    );
    assert_eq!(
        repository
            .force_refresh_staged_count(address)
            .expect("staged count"),
        1
    );

    let before = repository
        .list_for_address(address, 10)
        .expect("known-good remains readable before promote");
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].txid, "force-old");

    assert_eq!(
        repository
            .promote_force_refresh(address)
            .expect("atomic promote"),
        1
    );

    let after = repository
        .list_for_address(address, 10)
        .expect("replacement visible");
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].txid, "force-new");
    assert_eq!(
        repository
            .force_refresh_staged_count(address)
            .expect("stage cleared"),
        0
    );
    assert_eq!(repository.total_count().expect("old orphan removed"), 1);
}

#[test]
fn force_refresh_discard_keeps_known_good_data() {
    let manager = test_manager("transactions_force_refresh_discard");
    let repository = manager
        .transactions_repository()
        .expect("transactions repository must open");
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let mut old = TransactionRecord::new("discard-old", address, "transfer", "incoming", 10)
        .expect("old valid");
    old.raw_json = Some(r#"{"txid":"discard-old"}"#.to_string());
    repository.upsert(&old).expect("old upsert");

    repository
        .begin_force_refresh_stage(address)
        .expect("stage begins");

    let mut candidate = TransactionRecord::new("discard-new", address, "transfer", "incoming", 20)
        .expect("candidate valid");
    candidate.raw_json = Some(r#"{"txid":"discard-new"}"#.to_string());

    repository
        .stage_force_refresh_many(&[candidate])
        .expect("stage candidate");
    repository
        .discard_force_refresh_stage(address)
        .expect("discard stage");

    let rows = repository
        .list_for_address(address, 10)
        .expect("known-good remains");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].txid, "discard-old");
    assert_eq!(
        repository.total_count().expect("no staged canonical leak"),
        1
    );
}

#[test]
fn invalid_records_are_rejected() {
    let valid = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
    let bad_checksum = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44q";
    assert!(AddressRecord::new("", "Name", "mainnet").is_err());
    assert!(AddressRecord::new(bad_checksum, "Name", "mainnet").is_err());
    assert!(AddressRecord::new(valid, "Name", "testnet").is_err());
    assert!(AddressRecord::new(valid, "Name", "mainnet").is_ok());
    assert!(TransactionRecord::new("tx", "address", "transfer", "incoming", -1).is_err());
}

#[test]
fn sqlite_newer_schema_version_fails_closed_without_downgrade() {
    let root = unique_test_dir("newer_schema");
    let paths = DatabasePaths::new(&root).expect("paths");
    std::fs::create_dir_all(&root).expect("root");
    let connection =
        rusqlite::Connection::open(&paths.transactions_sqlite).expect("open raw sqlite");
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations(version INTEGER PRIMARY KEY, name TEXT NOT NULL, applied_at_ms INTEGER NOT NULL);
             INSERT INTO schema_migrations(version, name, applied_at_ms) VALUES (999, 'future-schema', 1);",
        )
        .expect("seed future schema");
    drop(connection);

    let manager = DatabaseManager::new(paths.clone());
    let error = match manager.transactions_repository() {
        Ok(_) => panic!("newer schema must fail closed"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("newer than supported"));

    let connection = rusqlite::Connection::open(&paths.transactions_sqlite).expect("reopen raw");
    let version: i64 = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("future version remains");
    assert_eq!(version, 999);
    drop(connection);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn sqlite_busy_is_bounded_and_known_good_survives() {
    let manager = test_manager("sqlite_busy");
    let path = manager.paths().transactions_sqlite.clone();
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let seed_repo = manager.transactions_repository().expect("seed repo");
    let mut seed = TransactionRecord::new("busy-known-good", address, "transfer", "incoming", 1)
        .expect("seed");
    seed.raw_json = Some("{}".to_string());
    seed_repo.upsert(&seed).expect("seed known-good");
    drop(seed_repo);

    let writer = rusqlite::Connection::open(&path).expect("lock writer");
    writer
        .execute_batch("BEGIN IMMEDIATE")
        .expect("writer lock");

    let candidate_connection = rusqlite::Connection::open(&path).expect("candidate connection");
    candidate_connection
        .busy_timeout(Duration::from_millis(75))
        .expect("bounded busy timeout");
    let candidate_repo = TransactionsRepository::new(candidate_connection);
    let mut candidate =
        TransactionRecord::new("busy-candidate", address, "transfer", "incoming", 2)
            .expect("candidate");
    candidate.raw_json = Some("{}".to_string());

    let started = std::time::Instant::now();
    assert!(candidate_repo.upsert(&candidate).is_err());
    assert!(started.elapsed() < Duration::from_secs(1));

    writer.execute_batch("ROLLBACK").expect("release lock");
    candidate_repo
        .upsert(&candidate)
        .expect("write after lock release");
    assert_eq!(candidate_repo.count_for_address(address).expect("count"), 2);

    drop(candidate_repo);
    drop(writer);
    let _ = std::fs::remove_dir_all(manager.paths().root.clone());
}

#[test]
fn sqlite_uncommitted_write_rolls_back_on_reopen_and_integrity_is_ok() {
    let manager = test_manager("sqlite_rollback_reopen");
    let path = manager.paths().transactions_sqlite.clone();
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let repo = manager.transactions_repository().expect("repo");
    let mut known = TransactionRecord::new("rollback-known", address, "transfer", "incoming", 3)
        .expect("known");
    known.raw_json = Some("{}".to_string());
    repo.upsert(&known).expect("seed");
    drop(repo);

    {
        let connection = rusqlite::Connection::open(&path).expect("raw");
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON;
                 BEGIN IMMEDIATE;
                 INSERT INTO transactions(
                    txid,address,tx_type,direction,amount_sompi,from_address,to_address,counterparty,
                    block_height,timestamp_ms,raw_json,created_at_ms,updated_at_ms
                 ) VALUES(
                    'rollback-uncommitted','kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g',
                    'transfer','incoming',9,NULL,NULL,NULL,NULL,9,'{}',9,9
                 );
                 INSERT INTO address_transactions(
                    address,txid,tx_type,direction,amount_sompi,from_address,to_address,counterparty,
                    created_at_ms,updated_at_ms
                 ) VALUES(
                    'kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g',
                    'rollback-uncommitted','transfer','incoming',9,NULL,NULL,NULL,9,9
                 );",
            )
            .expect("uncommitted transaction");
        // Dropping the connection without COMMIT simulates interruption.
    }

    let reopened = manager.transactions_repository().expect("reopen repo");
    assert_eq!(reopened.count_for_address(address).expect("count"), 1);
    let rows = reopened.list_for_address(address, 10).expect("rows");
    assert_eq!(rows[0].txid, "rollback-known");
    drop(reopened);

    let connection = rusqlite::Connection::open(&path).expect("integrity connection");
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity check");
    assert_eq!(integrity, "ok");
    drop(connection);
    let _ = std::fs::remove_dir_all(manager.paths().root.clone());
}

#[test]
fn sqlite_read_only_connection_reads_but_rejects_writes() {
    let manager = test_manager("sqlite_read_only");
    let path = manager.paths().transactions_sqlite.clone();
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
    let repo = manager.transactions_repository().expect("repo");
    let mut known = TransactionRecord::new("read-only-known", address, "transfer", "incoming", 4)
        .expect("known");
    known.raw_json = Some("{}".to_string());
    repo.upsert(&known).expect("seed");
    drop(repo);

    let connection =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("read-only open");
    let read_only_repo = TransactionsRepository::new(connection);
    assert_eq!(
        read_only_repo
            .list_for_address(address, 10)
            .expect("read")
            .len(),
        1
    );

    let mut blocked = TransactionRecord::new("read-only-write", address, "transfer", "incoming", 5)
        .expect("blocked");
    blocked.raw_json = Some("{}".to_string());
    assert!(read_only_repo.upsert(&blocked).is_err());

    drop(read_only_repo);
    let _ = std::fs::remove_dir_all(manager.paths().root.clone());
}

#[test]
fn sqlite_corrupt_database_is_detected_without_silent_reinitialization() {
    let root = unique_test_dir("sqlite_corrupt");
    let paths = DatabasePaths::new(&root).expect("paths");
    std::fs::create_dir_all(&root).expect("root");
    std::fs::write(&paths.transactions_sqlite, b"this is not a sqlite database")
        .expect("write corrupt fixture");

    let manager = DatabaseManager::new(paths.clone());
    assert!(
        manager.transactions_repository().is_err(),
        "corrupt database must fail closed"
    );
    let bytes = std::fs::read(&paths.transactions_sqlite).expect("corrupt bytes remain");
    assert_eq!(bytes, b"this is not a sqlite database");
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn sqlite_foreign_key_violation_is_rejected() {
    let manager = test_manager("sqlite_foreign_key");
    let path = manager.paths().transactions_sqlite.clone();
    let connection = rusqlite::Connection::open(&path).expect("raw");
    connection
        .execute_batch("PRAGMA foreign_keys=ON")
        .expect("foreign keys on");

    let result = connection.execute(
        "INSERT INTO address_transactions(
            address,txid,tx_type,direction,amount_sompi,from_address,to_address,counterparty,
            created_at_ms,updated_at_ms
         ) VALUES(?1,'missing-canonical','transfer','incoming',1,NULL,NULL,NULL,1,1)",
        rusqlite::params!["kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g"],
    );
    assert!(result.is_err(), "orphan relation must violate foreign key");

    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity");
    assert_eq!(integrity, "ok");
    drop(connection);
    let _ = std::fs::remove_dir_all(manager.paths().root.clone());
}

#[test]
fn sqlite_full_error_does_not_corrupt_database() {
    let root = unique_test_dir("sqlite_full");
    std::fs::create_dir_all(&root).expect("root");
    let path = root.join("full.sqlite");
    let connection = rusqlite::Connection::open(&path).expect("open");
    connection
        .execute_batch(
            "PRAGMA page_size=512;
             PRAGMA journal_mode=DELETE;
             CREATE TABLE payloads(id INTEGER PRIMARY KEY, data BLOB);",
        )
        .expect("initialize");
    let current_pages: i64 = connection
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .expect("page count");
    connection
        .pragma_update(None, "max_page_count", current_pages + 1)
        .expect("bound max pages");

    let result = connection.execute("INSERT INTO payloads(data) VALUES(zeroblob(1048576))", []);
    assert!(result.is_err(), "bounded database must report storage full");

    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity after full");
    assert_eq!(integrity, "ok");
    drop(connection);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn sqlite_fts5_trigram_search_stays_consistent_across_mutations() {
    let manager = test_manager("sqlite_fts_consistency");
    let path = manager.paths().transactions_sqlite.clone();
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";
    let repository = manager.transactions_repository().expect("repository");

    let raw = rusqlite::Connection::open(&path).expect("raw");
    let fts5_enabled: i64 = raw
        .query_row(
            "SELECT sqlite_compileoption_used('ENABLE_FTS5')",
            [],
            |row| row.get(0),
        )
        .expect("fts5 compile option");
    assert_eq!(fts5_enabled, 1);
    drop(raw);

    let mut record =
        TransactionRecord::new("fts-consistency-tx", address, "transfer", "incoming", 10)
            .expect("record");
    record.raw_json = Some("{}".to_string());
    record.counterparty = Some("kaspa:qneedlealpha000".to_string());
    repository.upsert(&record).expect("initial upsert");

    let search = |term: &str| {
        repository
            .filter_for_address(TransactionFilter {
                address,
                start_ms: None,
                end_ms: None,
                tx_type: None,
                direction: None,
                search: Some(term),
                limit: Some(100),
                offset: None,
            })
            .expect("substring search")
    };

    assert_eq!(search("needlealpha").len(), 1);
    assert_eq!(search("eed").len(), 1);
    assert_eq!(search("ee").len(), 1);
    assert!(search("needlealpha%").is_empty());

    record.counterparty = Some("kaspa:qneedlebravo000".to_string());
    repository.upsert(&record).expect("update upsert");
    assert!(search("needlealpha").is_empty());
    assert_eq!(search("needlebravo").len(), 1);

    repository
        .begin_force_refresh_stage(address)
        .expect("stage begin");
    let mut replacement =
        TransactionRecord::new("fts-refresh-tx", address, "transfer", "incoming", 20)
            .expect("replacement");
    replacement.raw_json = Some("{}".to_string());
    replacement.counterparty = Some("kaspa:qneedlecharlie000".to_string());
    repository
        .stage_force_refresh_many(&[replacement])
        .expect("stage replacement");
    repository
        .promote_force_refresh(address)
        .expect("promote replacement");

    assert!(search("needlebravo").is_empty());
    assert_eq!(search("needlecharlie").len(), 1);

    repository
        .delete_for_address(address)
        .expect("delete address");
    assert!(search("needlecharlie").is_empty());

    drop(repository);
    let raw = rusqlite::Connection::open(&path).expect("reopen raw");
    let fts_rows: i64 = raw
        .query_row("SELECT COUNT(*) FROM address_transactions_fts", [], |row| {
            row.get(0)
        })
        .expect("fts count");
    assert_eq!(fts_rows, 0);

    drop(raw);
    let _ = std::fs::remove_dir_all(manager.paths().root.clone());
}

#[test]
fn sqlite_fts5_index_rebuilds_from_known_good_relations_when_missing() {
    let manager = test_manager("sqlite_fts_rebuild");
    let path = manager.paths().transactions_sqlite.clone();
    let address = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

    let repository = manager.transactions_repository().expect("repository");
    let mut record = TransactionRecord::new("fts-rebuild-tx", address, "transfer", "incoming", 30)
        .expect("record");
    record.raw_json = Some("{}".to_string());
    record.counterparty = Some("kaspa:qrebuildneedle000".to_string());
    repository.upsert(&record).expect("seed relation");
    drop(repository);

    let raw = rusqlite::Connection::open(&path).expect("raw");
    raw.execute_batch(
        "DROP TRIGGER IF EXISTS address_transactions_fts_ai;
         DROP TRIGGER IF EXISTS address_transactions_fts_ad;
         DROP TRIGGER IF EXISTS address_transactions_fts_au;
         DROP TABLE IF EXISTS address_transactions_fts;",
    )
    .expect("simulate missing search index");
    drop(raw);

    let reopened = manager
        .transactions_repository()
        .expect("reopen rebuilds search index");
    let found = reopened
        .filter_for_address(TransactionFilter {
            address,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: Some("rebuildneedle"),
            limit: Some(100),
            offset: None,
        })
        .expect("search rebuilt index");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].txid, "fts-rebuild-tx");
    drop(reopened);

    let raw = rusqlite::Connection::open(&path).expect("raw verify");
    let relation_rows: i64 = raw
        .query_row("SELECT COUNT(*) FROM address_transactions", [], |row| {
            row.get(0)
        })
        .expect("relation count");
    let fts_rows: i64 = raw
        .query_row("SELECT COUNT(*) FROM address_transactions_fts", [], |row| {
            row.get(0)
        })
        .expect("fts count");
    assert_eq!(fts_rows, relation_rows);

    drop(raw);
    let _ = std::fs::remove_dir_all(manager.paths().root.clone());
}
