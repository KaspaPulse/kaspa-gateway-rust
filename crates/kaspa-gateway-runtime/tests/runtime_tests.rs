use kaspa_gateway_runtime::{AppRuntime, RuntimePaths};

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time valid")
        .as_nanos();

    std::env::temp_dir().join(format!("kaspa_gateway_runtime_{name}_{stamp}"))
}

#[test]
fn runtime_initializes_config_database_api_and_node_layers() {
    let root = unique_test_dir("init");
    let config_path = root.join("config").join("config.json");
    let database_root = root.join("databases");

    let runtime =
        AppRuntime::initialize_from_paths(RuntimePaths::new(&config_path, &database_root))
            .expect("runtime must initialize");

    assert_eq!(runtime.app_info().name, "Kaspa Gateway");
    assert!(config_path.exists());
    assert!(database_root.exists());
    assert!(runtime.database_manager().paths().app_data.exists());
    assert!(runtime.database_manager().paths().addresses.exists());
    assert!(runtime.database_manager().paths().transactions.exists());
}

#[test]
fn runtime_health_report_is_healthy() {
    let root = unique_test_dir("health");
    let config_path = root.join("config").join("config.json");
    let database_root = root.join("databases");

    let runtime =
        AppRuntime::initialize_from_paths(RuntimePaths::new(&config_path, &database_root))
            .expect("runtime must initialize");

    let report = runtime.health_report().expect("health report must work");

    assert!(report.config_loaded);
    assert!(report.database_initialized);
    assert!(report.api_client_ready);
    assert!(report.is_healthy());
    assert!(report
        .api_network_url
        .contains("https://api.kaspa.org/info/network"));
    assert!(report.node_capabilities.contains("integrated-bridge"));
}

#[test]
fn runtime_paths_can_be_constructed_explicitly() {
    let root = unique_test_dir("paths");
    let paths = RuntimePaths::new(root.join("config.json"), root.join("db"));

    assert!(paths.config_path.ends_with("config.json"));
    assert!(paths.database_root.ends_with("db"));
}
