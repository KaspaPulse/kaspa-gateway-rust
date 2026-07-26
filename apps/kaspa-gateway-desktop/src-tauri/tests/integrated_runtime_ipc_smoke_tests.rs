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
fn apply_node_settings_starts_owned_worker_for_testnet10() {
    let applied = kgw_kgw_apply_node_settings_v1(
        "testnet10".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
    )
    .expect("apply_node_settings should succeed");

    let status_result =
        kgw_runtime_owner_status_v1(Some("testnet10".to_string()), Some("node".to_string()));
    let logs_result =
        kgw_kgw_runtime_logs_v1(Some("testnet10".to_string()), Some("node".to_string()));

    let disabled_result =
        kgw_kgw_disable_network_v1("testnet10".to_string(), Some("node".to_string()));

    let status = status_result.expect("status should succeed");
    let _logs = logs_result.expect("logs should succeed");
    let disabled = disabled_result.expect("cleanup should succeed");

    assert_contains_all(
        &applied,
        &[
            "parallel-owned-self-worker started",
            "role=node",
            "network=testnet10",
            "same_exe=true",
            "external_kaspad_exe=false",
            "uses_kaspa_libraries=true",
            "same_db_path=true",
            "exclusive_node_owner_per_network=true",
        ],
    );
    assert!(!status.trim().is_empty());
    assert!(!disabled.trim().is_empty());
}

#[test]
fn disable_network_stops_owned_worker_for_testnet12() {
    let applied = kgw_kgw_apply_node_settings_v1(
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
    )
    .expect("apply_node_settings should succeed");

    let disabled = kgw_kgw_disable_network_v1("testnet12".to_string(), Some("node".to_string()))
        .expect("disable should succeed");

    assert_contains_all(
        &applied,
        &[
            "parallel-owned-self-worker started",
            "role=node",
            "network=testnet12",
            "same_exe=true",
            "external_kaspad_exe=false",
            "uses_kaspa_libraries=true",
        ],
    );
    assert!(!disabled.trim().is_empty());
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
    )
    .expect_err("unsupported network must fail");

    assert!(
        error.to_ascii_lowercase().contains("network"),
        "error should mention network: {error}"
    );
}
