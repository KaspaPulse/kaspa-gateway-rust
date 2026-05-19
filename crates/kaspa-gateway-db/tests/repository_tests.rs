use kaspa_gateway_db::{
    AddressRecord, AppCacheRepository, AppSettingsRepository, DatabaseManager, DatabasePaths,
    TransactionRecord,
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
fn invalid_records_are_rejected() {
    assert!(AddressRecord::new("", "Name", "mainnet").is_err());
    assert!(TransactionRecord::new("tx", "address", "transfer", "incoming", -1).is_err());
}
