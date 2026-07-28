#[path = "../src/integrated_runtime_commands.rs"]
mod integrated_runtime_commands;

use integrated_runtime_commands::{
    kgw_kgw_apply_node_settings_v1, kgw_kgw_disable_network_v1,
    kgw_kgw_node_bridge_service_plan_v1, kgw_kgw_runtime_logs_v1, kgw_runtime_owner_summary_v1,
};
use std::sync::{Mutex, OnceLock};

fn runtime_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

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

#[test]
fn start_command_is_registered_and_payload_matches_frontend() {
    let lib_rs = include_str!("../src/lib.rs");
    let node_js = include_str!("../../frontend/src/tabs/kaspa-node/kaspa-node.js");

    assert!(
        lib_rs.contains("integrated_runtime_commands::kgw_kgw_apply_node_settings_v1"),
        "start command must be registered in tauri generate_handler"
    );
    assert!(
        lib_rs.contains("integrated_runtime_commands::kgw_kgw_disable_network_v1"),
        "stop command must be registered in tauri generate_handler"
    );
    assert!(
        lib_rs.contains("kgw_start_trace_frontend_v1"),
        "start trace frontend command must be registered in tauri generate_handler"
    );

    for field in [
        "network",
        "nodeKind",
        "bridgeKind",
        "nodeCommandPreview",
        "bridgeCommandPreview",
        "runtimeRole",
        "experimentalNetworkOptIn",
    ] {
        assert!(
            node_js.contains(field),
            "frontend start payload must contain `{field}`"
        );
    }
}

#[test]
fn start_trace_marker_format_is_registered_and_safe() {
    let lib_rs = include_str!("../src/lib.rs");
    assert!(
        lib_rs.contains("kgw_start_trace_frontend_v1"),
        "native trace command must be registered"
    );

    let marker = integrated_runtime_commands::kgw_start_trace_format_v1(
        "native",
        "native.tauri_command_entered",
        "mainnet",
        "start",
        "entered",
        Some("{\"secret\":\"abc\",\"wallet\":\"kaspa:abc\",\"completeCommand\":\"--rpc 127.0.0.1:16110\"}"),
    );

    assert!(marker.starts_with("[KGW_START_TRACE] "));
    assert_contains_all(
        &marker,
        &[
            "\"timestamp\":",
            "\"stage\":\"native.tauri_command_entered\"",
            "\"network\":\"mainnet\"",
            "\"action\":\"start\"",
            "\"result\":\"entered\"",
            "\\\"redacted\\\":true",
        ],
    );
    assert!(
        !marker.contains("kaspa:abc") && !marker.contains("--rpc") && !marker.contains("abc"),
        "trace marker must not expose sensitive fields: {marker}"
    );
}

#[test]
fn mainnet_and_testnet10_normalize_to_supported_networks() {
    assert_eq!(
        kaspa_gateway_rk_node::KgwNetwork::parse("mainnet")
            .unwrap()
            .as_str(),
        "mainnet"
    );
    assert_eq!(
        kaspa_gateway_rk_node::KgwNetwork::parse("testnet")
            .unwrap()
            .as_str(),
        "testnet10"
    );
    assert_eq!(
        kaspa_gateway_rk_node::KgwNetwork::parse("testnet10")
            .unwrap()
            .as_str(),
        "testnet10"
    );
}

#[test]
fn duplicate_owner_for_one_network_is_rejected() {
    let _guard = runtime_test_lock().lock().unwrap();
    std::env::set_var("KGW_TEST_SELF_WORKER_COMMAND", "1");
    let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();

    let first = kgw_kgw_apply_node_settings_v1(
        "mainnet".to_string(),
        "integrated-as-daemon".to_string(),
        "disable".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("first test worker start should succeed");

    assert_contains_all(
        &first,
        &[
            "parallel-owned-self-worker started",
            "network=mainnet",
            "pid=",
            "owner=self-worker",
            "runtime_state=running",
        ],
    );

    std::thread::sleep(std::time::Duration::from_millis(250));

    let mainnet_logs =
        kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()))
            .expect("mainnet logs should be readable");

    assert_contains_all(
        &mainnet_logs,
        &[
            "kgw_raw_process_log_v1",
            "network=mainnet",
            "source=self-worker",
            "runtime_role=node",
            "stream=stdout",
            "stream=stderr",
        ],
    );

    let duplicate = kgw_kgw_apply_node_settings_v1(
        "mainnet".to_string(),
        "integrated-as-daemon".to_string(),
        "disable".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("duplicate test worker start must be rejected");

    assert_contains_all(
        &duplicate,
        &[
            "start_blocked=true",
            "block_reason=duplicate-owner",
            "network=mainnet",
        ],
    );

    let _ = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()));
    std::env::remove_var("KGW_TEST_SELF_WORKER_COMMAND");
}

#[test]
fn success_response_contains_process_start_evidence_and_stream_logs() {
    let _guard = runtime_test_lock().lock().unwrap();
    std::env::set_var("KGW_TEST_SELF_WORKER_COMMAND", "1");
    let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();

    let started = kgw_kgw_apply_node_settings_v1(
        "testnet10".to_string(),
        "integrated-as-daemon".to_string(),
        "disable".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("test worker start should succeed");

    assert_contains_all(
        &started,
        &[
            "parallel-owned-self-worker started",
            "network=testnet10",
            "pid=",
            "same_exe=true",
            "external_kaspad_exe=false",
            "runtime_state=running",
        ],
    );

    std::thread::sleep(std::time::Duration::from_millis(250));

    let logs = kgw_kgw_runtime_logs_v1(Some("testnet10".to_string()), Some("node".to_string()))
        .expect("logs should be readable");

    assert_contains_all(
        &logs,
        &[
            "kgw_raw_process_log_v1",
            "network=testnet10",
            "source=self-worker",
            "runtime_role=node",
            "stream=stdout",
            "stream=stderr",
            "test-self-worker stdout",
            "test-self-worker stderr",
        ],
    );

    let _ = kgw_kgw_disable_network_v1("testnet10".to_string(), Some("node".to_string()));
    std::env::remove_var("KGW_TEST_SELF_WORKER_COMMAND");
}

#[test]
fn spawn_failure_remains_an_error() {
    let _guard = runtime_test_lock().lock().unwrap();
    let missing = std::env::temp_dir().join("kgw-missing-self-worker-command.exe");
    std::env::set_var("KGW_START_TRACE", "1");
    std::env::set_var("KGW_TEST_SELF_WORKER_MISSING_COMMAND", &missing);
    let _ = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1();
    let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();

    let error = kgw_kgw_apply_node_settings_v1(
        "mainnet".to_string(),
        "integrated-as-daemon".to_string(),
        "disable".to_string(),
        None,
        None,
        Some("node".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("missing test command must surface spawn failure");

    assert_contains_all(
        &error,
        &["spawn_failed=true", "runtime_role=node", "network=mainnet"],
    );

    let trace_lines = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1();
    let trace_text = trace_lines.join("\n");
    assert_contains_all(
        &trace_text,
        &[
            "[KGW_START_TRACE]",
            "\"stage\":\"native.tauri_command_entered\"",
            "\"stage\":\"native.spawn_failed\"",
            "\"stage\":\"native.startup_response_returned\"",
            "\"network\":\"mainnet\"",
            "\"action\":\"start\"",
        ],
    );

    std::env::remove_var("KGW_TEST_SELF_WORKER_MISSING_COMMAND");
    std::env::remove_var("KGW_START_TRACE");
}
