#[path = "../src/integrated_runtime_commands.rs"]
mod integrated_runtime_commands;

use integrated_runtime_commands::{
    kgw_kgw_apply_node_settings_v1, kgw_kgw_node_bridge_service_plan_v1,
    kgw_runtime_owner_summary_v1,
};

fn assert_contains_all(text: &str, parts: &[&str]) {
    for part in parts {
        assert!(text.contains(part), "expected `{part}` in `{text}`");
    }
}

#[test]
fn exact_kgw_controller_summary_mentions_event_flow() {
    let summary = kgw_runtime_owner_summary_v1().expect("summary should succeed");

    assert_contains_all(
        &summary,
        &[
            "NodeSettings",
            "KaspadServiceEvents::from_node_settings",
            "controller event loop",
            "testnet12 uses the opt-in experimental tn12 runtime",
        ],
    );
}

#[test]
fn testnet10_uses_the_official_stable_runtime_family() {
    let plan = kgw_kgw_node_bridge_service_plan_v1(
        "testnet10".to_string(),
        "integrated-as-daemon".to_string(),
        "disabled".to_string(),
        true,
    )
    .expect("testnet10 plan should succeed");

    assert_contains_all(
        &plan,
        &[
            "network=testnet10",
            "family=official-stable-v2.0.1",
            "branch=stable",
        ],
    );
}

#[test]
fn testnet12_requires_explicit_experimental_opt_in() {
    let error = kgw_kgw_apply_node_settings_v1(
        "testnet12".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("testnet12 must be blocked without explicit opt-in");

    assert_contains_all(
        &error,
        &[
            "network=testnet12",
            "start_blocked=true",
            "experimental-network-opt-in-required",
        ],
    );
}

#[test]
fn unsupported_network_is_rejected() {
    let error = kgw_kgw_apply_node_settings_v1(
        "badnet".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("unsupported network must fail");

    assert!(
        error.to_ascii_lowercase().contains("network"),
        "error should mention network: {error}"
    );
}
