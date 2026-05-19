#[path = "../src/integrated_runtime_commands.rs"]
mod integrated_runtime_commands;

use integrated_runtime_commands::{
    kgw_kgw_apply_node_settings_v1, kgw_kgw_disable_network_v1, kgw_kgw_runtime_logs_v1,
    kgw_runtime_owner_status_v1, kgw_runtime_owner_summary_v1,
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
            "testnet12 uses tn12",
        ],
    );
}

#[test]
fn apply_node_settings_uses_exact_event_path_for_testnet10() {
    let applied = kgw_kgw_apply_node_settings_v1(
        "testnet10".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
    )
    .expect("apply_node_settings should succeed");

    assert_contains_all(
        &applied,
        &[
            "apply_node_settings accepted",
            "network=testnet10",
            "node_kind=integrated-inproc",
            "bridge_kind=official-inprocess-node",
            "branch=master",
        ],
    );

    let status =
        kgw_runtime_owner_status_v1(Some("testnet10".to_string())).expect("status should succeed");

    assert_contains_all(&status, &["network=testnet10", "branch=master"]);

    let logs = kgw_kgw_runtime_logs_v1(Some("testnet10".to_string())).expect("logs should succeed");

    assert_contains_all(&logs, &["StartInternalInProc", "StartBridgeInProcessNode"]);
}

#[test]
fn disable_network_uses_controller_event_path_for_testnet12() {
    let applied = kgw_kgw_apply_node_settings_v1(
        "testnet12".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
    )
    .expect("apply_node_settings should succeed");

    assert_contains_all(&applied, &["network=testnet12", "branch=tn12"]);

    let disabled =
        kgw_kgw_disable_network_v1("testnet12".to_string()).expect("disable should succeed");

    assert_contains_all(
        &disabled,
        &["disable accepted", "network=testnet12", "branch=tn12"],
    );

    let status =
        kgw_runtime_owner_status_v1(Some("testnet12".to_string())).expect("status should succeed");

    assert_contains_all(&status, &["network=testnet12", "branch=tn12"]);
}

#[test]
fn unsupported_network_is_rejected() {
    let error = kgw_kgw_apply_node_settings_v1(
        "badnet".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
    )
    .expect_err("unsupported network must fail");

    assert!(
        error.to_ascii_lowercase().contains("network"),
        "error should mention network: {error}"
    );
}
