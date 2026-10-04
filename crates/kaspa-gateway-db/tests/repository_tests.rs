use kaspa_gateway_db::{
    AddressRecord, AppCacheRepository, AppSettingsRepository, DatabaseManager, DatabasePaths,
    TransactionFilter, TransactionRecord,
};
use std::path::PathBuf;

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
