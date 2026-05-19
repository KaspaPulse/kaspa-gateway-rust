use kaspa_gateway_node::{MiningConnectivityMode, NodeCapabilities, NodeEndpoint};

#[test]
fn default_node_capabilities_are_integrated_bridge_ready() {
    let caps = NodeCapabilities::default();

    assert!(!caps.node_available);
    assert!(!caps.rpc_available);
    assert!(!caps.mining_endpoint_available);

    match caps.mining_mode {
        MiningConnectivityMode::IntegratedBridge => {}
        _ => panic!("default mining mode must be IntegratedBridge"),
    }
}

#[test]
fn node_endpoint_formats_display_address() {
    let endpoint = NodeEndpoint::new("127.0.0.1", 16110, false).expect("endpoint valid");
    assert_eq!(endpoint.display_address(), "ws://127.0.0.1:16110");
}
