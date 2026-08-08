use kaspa_gateway_node::{
    MiningConnectivityMode, NodeBinaryKind, NodeCapabilityManager, NodeEndpoint, NodeManagerConfig,
    infer_binary_kind,
};

#[test]
fn default_capabilities_are_integrated_bridge_ready() {
    let config = NodeManagerConfig::default();
    let capabilities = NodeCapabilityManager::inspect(&config).expect("inspect must work");

    assert!(capabilities.node_available);
    assert!(capabilities.rpc_available);
    assert!(capabilities.mining_endpoint_available);
    assert_eq!(
        capabilities.mining_mode,
        MiningConnectivityMode::IntegratedBridge
    );
    assert_eq!(capabilities.binary_kind, NodeBinaryKind::KgwOwner);
}

#[test]
fn owner_args_reject_shell_operators() {
    for value in ["--rpclisten=127.0.0.1:16110", "--safe=value"] {
        let config = NodeManagerConfig {
            extra_args: vec![value.to_string()],
            ..NodeManagerConfig::default()
        };

        assert!(
            NodeCapabilityManager::build_launch_plan(&config).is_ok(),
            "expected safe argument: {value}"
        );
    }

    for value in [
        "--bad=value && whoami",
        "--bad=value\nnext",
        "--bad=value|more",
    ] {
        let config = NodeManagerConfig {
            extra_args: vec![value.to_string()],
            ..NodeManagerConfig::default()
        };

        assert!(
            NodeCapabilityManager::build_launch_plan(&config).is_err(),
            "expected unsafe argument rejection: {value}"
        );
    }
}

#[test]
fn networks_are_allowlisted_through_public_api() {
    for network in ["mainnet", "testnet-10", "simnet"] {
        let config = NodeManagerConfig {
            network: network.to_string(),
            ..NodeManagerConfig::default()
        };

        assert!(
            NodeCapabilityManager::inspect(&config).is_ok(),
            "expected valid network: {network}"
        );
    }

    let invalid = NodeManagerConfig {
        network: "evilnet".to_string(),
        ..NodeManagerConfig::default()
    };

    assert!(NodeCapabilityManager::inspect(&invalid).is_err());
}

#[test]
fn binary_kind_is_owned_by_kgw() {
    for executable in [
        "C:\\kaspa\\kaspad.exe",
        "C:\\kaspa\\rusty-kaspa.exe",
        "C:\\kaspa\\unknown.exe",
    ] {
        assert_eq!(infer_binary_kind(executable), NodeBinaryKind::KgwOwner);
    }
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
        network: "mainnet".to_string(),
        rpc_endpoint: NodeEndpoint::new("127.0.0.1", 16110, false).expect("endpoint"),
        mining_mode: MiningConnectivityMode::IntegratedBridge,
        mining_endpoint: None,
        extra_args: vec!["--maxinpeers=32".to_string()],
    };

    let plan = NodeCapabilityManager::build_launch_plan(&config).expect("plan must build");

    assert_eq!(plan.network, "mainnet");
    assert_eq!(plan.mining_mode, MiningConnectivityMode::IntegratedBridge);
    assert!(plan.args.iter().any(|arg| arg == "--utxoindex"));
    assert!(
        plan.args
            .iter()
            .any(|arg| arg == "--rpclisten=127.0.0.1:16110")
    );
    assert!(plan.args.iter().any(|arg| arg == "--maxinpeers=32"));
    plan.validate().expect("plan must validate");
}

#[test]
fn testnet_launch_plan_is_normalized() {
    let config = NodeManagerConfig {
        network: "testnet-12".to_string(),
        ..NodeManagerConfig::default()
    };

    let plan = NodeCapabilityManager::build_launch_plan(&config).expect("plan must build");

    assert_eq!(plan.network, "testnet12");
    assert!(plan.args.iter().any(|arg| arg == "--testnet"));
    assert!(plan.args.iter().any(|arg| arg == "--netsuffix=12"));
}

#[test]
fn launch_plan_rejects_unsafe_extra_args() {
    let config = NodeManagerConfig {
        extra_args: vec!["--bad=value && whoami".to_string()],
        ..NodeManagerConfig::default()
    };

    assert!(NodeCapabilityManager::build_launch_plan(&config).is_err());
}
