use kaspa_gateway_db::{
    DatabaseManager, DatabasePaths, initialize_addresses_schema, initialize_app_data_schema,
    initialize_transactions_schema, schema_version,
};
use std::path::PathBuf;

fn unique_test_dir(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time valid")
        .as_nanos();

    std::env::temp_dir().join(format!("kaspa_gateway_{name}_{stamp}"))
}

#[test]
fn database_paths_are_created_from_root() {
    let root = unique_test_dir("paths");
    let paths = DatabasePaths::new(&root).expect("paths must be valid");

    assert_eq!(paths.root, root);
    assert!(paths.app_data.ends_with("AppData.duckdb"));
    assert!(paths.addresses.ends_with("Addresses.duckdb"));
    assert!(paths.transactions.ends_with("Transactions.duckdb"));
}

#[test]
fn schema_version_is_stage5_initial_version() {
    assert_eq!(schema_version(), 1);
}

#[test]
fn manager_initializes_all_database_files() {
    let root = unique_test_dir("manager_init");
    let paths = DatabasePaths::new(&root).expect("paths must be valid");
    let manager = DatabaseManager::new(paths.clone());

    manager
        .initialize_all()
        .expect("all schemas must initialize");

    assert!(paths.app_data.exists());
    assert!(paths.addresses.exists());
    assert!(paths.transactions.exists());
}

#[test]
fn app_data_schema_can_be_initialized() {
    let root = unique_test_dir("app_data_schema");
    let paths = DatabasePaths::new(&root).expect("paths must be valid");
    let manager = DatabaseManager::new(paths);

    let connection = manager.open_app_data().expect("app data DB must open");
    initialize_app_data_schema(&connection).expect("schema must initialize");

    connection
        .prepare("SELECT key, value FROM app_settings LIMIT 0")
        .expect("app_settings table must exist");

    connection
        .prepare("SELECT version, name FROM schema_migrations LIMIT 0")
        .expect("schema_migrations table must exist");
}

#[test]
fn addresses_schema_can_be_initialized() {
    let root = unique_test_dir("addresses_schema");
    let paths = DatabasePaths::new(&root).expect("paths must be valid");
    let manager = DatabaseManager::new(paths);

    let connection = manager.open_addresses().expect("addresses DB must open");
    initialize_addresses_schema(&connection).expect("schema must initialize");

    connection
        .prepare("SELECT address, name, network FROM addresses LIMIT 0")
        .expect("addresses table must exist");
}

#[test]
fn transactions_schema_can_be_initialized() {
    let root = unique_test_dir("transactions_schema");
    let paths = DatabasePaths::new(&root).expect("paths must be valid");
    let manager = DatabaseManager::new(paths);

    let connection = manager
        .open_transactions()
        .expect("transactions DB must open");

    initialize_transactions_schema(&connection).expect("schema must initialize");

    connection
        .prepare("SELECT txid, address, amount_sompi FROM transactions LIMIT 0")
        .expect("transactions table must exist");
}
