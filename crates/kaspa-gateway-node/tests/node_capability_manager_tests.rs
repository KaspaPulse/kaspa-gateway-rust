use kaspa_gateway_node::{
    infer_binary_kind, validate_network, validate_process_arg, MiningConnectivityMode,
    NodeBinaryKind, NodeCapabilityManager, NodeEndpoint, NodeManagerConfig,
};

#[test]
fn default_capabilities_are_integrated_bridge_ready() {
    let config = NodeManagerConfig::default();
    let capabilities = NodeCapabilityManager::inspect(&config).expect("inspect must work");

    assert!(!capabilities.node_available);
    assert!(capabilities.rpc_available);
    assert!(capabilities.mining_endpoint_available);
    assert_eq!(
        capabilities.mining_mode,
        MiningConnectivityMode::IntegratedBridge
    );
}

#[test]
fn process_args_reject_shell_operators() {
    assert!(validate_process_arg("--rpclisten=127.0.0.1:16110").is_ok());
    assert!(validate_process_arg("--safe=value").is_ok());
    assert!(validate_process_arg("--bad=value && whoami").is_err());
    assert!(validate_process_arg("--bad=value\nnext").is_err());
    assert!(validate_process_arg("--bad=value|more").is_err());
}

#[test]
fn networks_are_allowlisted() {
    assert!(validate_network("mainnet").is_ok());
    assert!(validate_network("testnet-10").is_ok());
    assert!(validate_network("simnet").is_ok());
    assert!(validate_network("evilnet").is_err());
}

#[test]
fn binary_kind_is_inferred_from_name() {
    assert_eq!(NodeBinaryKind::Kaspad);
    assert_eq!(
        infer_binary_kind("C:\\kaspa\\rusty-kaspa.exe"),
        NodeBinaryKind::RustyKaspa
    );
    assert_eq!(
        infer_binary_kind("C:\\kaspa\\unknown.exe"),
        NodeBinaryKind::Unknown
    );
}

#[test]
fn node_endpoint_rejects_unsafe_hosts() {
    assert!(NodeEndpoint::new("127.0.0.1", 16110, false).is_ok());
    assert!(NodeEndpoint::new("bad host", 16110, false).is_err());
    assert!(NodeEndpoint::new("example.com/path", 16110, false).is_err());
    assert!(NodeEndpoint::new("127.0.0.1", 0, false).is_err());
}

#[test]
fn launch_plan_is_built_without_shell_execution() {
    let config = NodeManagerConfig {
        binary_kind: NodeBinaryKind::Kaspad,
        network: "mainnet".to_string(),
        rpc_endpoint: NodeEndpoint::new("127.0.0.1", 16110, false).expect("endpoint"),
        mining_mode: MiningConnectivityMode::IntegratedBridge,
        mining_endpoint: None,
        extra_args: vec!["--utxoindex".to_string()],
    };

    let plan = NodeCapabilityManager::build_launch_plan(&config).expect("plan must build");

    assert_eq!(plan.binary_kind, NodeBinaryKind::Kaspad);
    assert!(plan.args.iter().any(|arg| arg == "--network=mainnet"));
    assert!(plan
        .args
        .iter()
        .any(|arg| arg == "--rpclisten=127.0.0.1:16110"));
    assert!(plan.args.iter().any(|arg| arg == "--utxoindex"));
}

#[test]
fn launch_plan_rejects_unsafe_extra_args() {
    let config = NodeManagerConfig {
        binary_kind: NodeBinaryKind::Kaspad,
        network: "mainnet".to_string(),
        rpc_endpoint: NodeEndpoint::new("127.0.0.1", 16110, false).expect("endpoint"),
        mining_mode: MiningConnectivityMode::IntegratedBridge,
        mining_endpoint: None,
        extra_args: vec!["--bad=value && whoami".to_string()],
    };

    assert!(NodeCapabilityManager::build_launch_plan(&config).is_err());
}
