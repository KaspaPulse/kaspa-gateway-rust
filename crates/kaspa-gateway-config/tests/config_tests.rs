use kaspa_gateway_config::{GatewayConfig, GatewayPaths};

#[test]
fn default_config_is_valid() {
    let config = GatewayConfig::default();
    config.validate().expect("default config must be valid");
}

#[test]
fn gateway_paths_are_derived_from_root() {
    let root = std::path::PathBuf::from("C:\\Users\\Public\\KaspaGatewayTest");
    let paths = GatewayPaths::from_root(&root).expect("paths must be valid");

    assert!(paths.database_dir.ends_with("data"));
    assert!(paths.export_dir.ends_with("exports"));
    assert!(paths.log_dir.ends_with("logs"));
    assert!(paths.backup_dir.ends_with("backups"));
}
