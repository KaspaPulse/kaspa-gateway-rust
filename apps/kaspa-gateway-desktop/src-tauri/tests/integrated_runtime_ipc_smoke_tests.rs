#[path = "../src/integrated_runtime_commands.rs"]
mod integrated_runtime_commands;

use integrated_runtime_commands::{
    KgwStartupControlMessageV1, KgwStopOutcomeMessageV1, KgwValidatedStopOutcomeV1,
    kgw_kgw_apply_node_settings_v1, kgw_kgw_disable_network_v1,
    kgw_kgw_node_bridge_service_plan_v1, kgw_kgw_runtime_clear_logs_v1, kgw_kgw_runtime_logs_v1,
    kgw_runtime_owner_summary_v1, kgw_worker_validate_startup_attestation_v1,
    kgw_worker_validate_stop_outcome_v1,
};
use std::sync::{Mutex, OnceLock};

fn runtime_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn set_runtime_worker_test_env<K, V>(key: K, value: V)
where
    K: AsRef<std::ffi::OsStr>,
    V: AsRef<std::ffi::OsStr>,
{
    // SAFETY: Every environment-mutating test in this integration-test binary holds
    // runtime_test_lock() for the full mutation/use/cleanup sequence. The KGW_* values
    // are read synchronously while planning/spawning child self-workers; parent log-reader
    // threads do not read process environment variables, and cleanup stops workers before
    // removing their test configuration. This preserves the Rust 2024 environment contract.
    unsafe { std::env::set_var(key, value) };
}

fn remove_runtime_worker_test_env<K>(key: K)
where
    K: AsRef<std::ffi::OsStr>,
{
    // SAFETY: Same serialized integration-test invariant as set_runtime_worker_test_env.
    unsafe { std::env::remove_var(key) };
}

struct RuntimeWorkerTestGuard;

impl RuntimeWorkerTestGuard {
    fn new() -> Self {
        let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();
        clear_runtime_worker_test_env();
        Self
    }
}

impl Drop for RuntimeWorkerTestGuard {
    fn drop(&mut self) {
        let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();
        clear_runtime_worker_test_env();
    }
}

fn clear_runtime_worker_test_env() {
    for key in [
        "KGW_TEST_SELF_WORKER_COMMAND",
        "KGW_TEST_SELF_WORKER_FAIL_COMMAND",
        "KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND",
        "KGW_TEST_SELF_WORKER_MISSING_COMMAND",
        "KGW_TEST_SELF_WORKER_READY_DELAY_MS",
        "KGW_TEST_STARTUP_ATTESTATION_TIMEOUT_MS",
        "KGW_TEST_SELF_WORKER_STDOUT",
        "KGW_TEST_SELF_WORKER_STDERR",
        "KGW_TEST_SELF_WORKER_HANG_ON_STOP",
        "KGW_TEST_SELF_WORKER_TIMEOUT_ON_STOP",
        "KGW_TEST_SELF_WORKER_FAIL_ON_STOP",
        "KGW_TEST_SELF_WORKER_BRIDGE_LISTENER_FAIL_ON_STOP",
        "KGW_TEST_SELF_WORKER_OWNED_NODE_STOP_MARKER_PATH",
        "KGW_TEST_PARENT_GRACEFUL_STOP_TIMEOUT_MS",
        "KGW_START_TRACE",
    ] {
        remove_runtime_worker_test_env(key);
    }
    integrated_runtime_commands::kgw_clear_raw_process_log_buffers_for_test_v1();
}

fn assert_contains_all(text: &str, parts: &[&str]) {
    for part in parts {
        assert!(text.contains(part), "expected `{part}` in `{text}`");
    }
}

fn startup_control_files_for_test_worker(role: &str, network: &str) -> Vec<std::path::PathBuf> {
    let expected_prefix = format!("{role}-{network}-{}-", std::process::id());
    let directory = std::env::temp_dir()
        .join("KaspaGateway")
        .join("startup-control");
    let mut paths = std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|name| name.starts_with(&expected_prefix))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn stop_outcome_requires_exact_protocol_role_network_and_pid() {
    let accepted = kgw_worker_validate_stop_outcome_v1(
        KgwStopOutcomeMessageV1 {
            version: 1,
            outcome: "STOPPED".to_string(),
            runtime_role: "node".to_string(),
            network: "mainnet".to_string(),
            worker_pid: 42,
            all_owned_components_terminal: true,
            evidence: Some("official-core-shutdown-and-join".to_string()),
            error: None,
        },
        "node",
        "mainnet",
        42,
    )
    .expect("matching STOPPED outcome must be accepted");
    assert!(matches!(
        accepted,
        KgwValidatedStopOutcomeV1::Stopped(ref evidence)
            if evidence == "official-core-shutdown-and-join"
    ));

    let stopped_without_terminal_proof = kgw_worker_validate_stop_outcome_v1(
        KgwStopOutcomeMessageV1 {
            version: 1,
            outcome: "STOPPED".to_string(),
            runtime_role: "node".to_string(),
            network: "mainnet".to_string(),
            worker_pid: 42,
            all_owned_components_terminal: false,
            evidence: Some("unproven shutdown".to_string()),
            error: None,
        },
        "node",
        "mainnet",
        42,
    );
    assert!(
        stopped_without_terminal_proof
            .unwrap_err()
            .contains("lacks terminal ownership proof")
    );

    let failed_without_error = kgw_worker_validate_stop_outcome_v1(
        KgwStopOutcomeMessageV1 {
            version: 1,
            outcome: "FAILED".to_string(),
            runtime_role: "node".to_string(),
            network: "mainnet".to_string(),
            worker_pid: 42,
            all_owned_components_terminal: true,
            evidence: None,
            error: None,
        },
        "node",
        "mainnet",
        42,
    );
    assert!(
        failed_without_error
            .unwrap_err()
            .contains("FAILED is missing an error")
    );

    for message in [
        KgwStopOutcomeMessageV1 {
            version: 2,
            outcome: "STOPPED".to_string(),
            runtime_role: "node".to_string(),
            network: "mainnet".to_string(),
            worker_pid: 42,
            all_owned_components_terminal: true,
            evidence: Some("wrong version".to_string()),
            error: None,
        },
        KgwStopOutcomeMessageV1 {
            version: 1,
            outcome: "STOPPED".to_string(),
            runtime_role: "bridge".to_string(),
            network: "mainnet".to_string(),
            worker_pid: 42,
            all_owned_components_terminal: true,
            evidence: Some("wrong role".to_string()),
            error: None,
        },
        KgwStopOutcomeMessageV1 {
            version: 1,
            outcome: "STOPPED".to_string(),
            runtime_role: "node".to_string(),
            network: "testnet10".to_string(),
            worker_pid: 42,
            all_owned_components_terminal: true,
            evidence: Some("wrong network".to_string()),
            error: None,
        },
        KgwStopOutcomeMessageV1 {
            version: 1,
            outcome: "STOPPED".to_string(),
            runtime_role: "node".to_string(),
            network: "mainnet".to_string(),
            worker_pid: 43,
            all_owned_components_terminal: true,
            evidence: Some("wrong pid".to_string()),
            error: None,
        },
    ] {
        assert!(kgw_worker_validate_stop_outcome_v1(message, "node", "mainnet", 42).is_err());
    }
}

#[test]
fn failed_stop_outcome_requires_explicit_terminal_ownership_proof() {
    for (terminal, expected_terminal) in [(false, false), (true, true)] {
        let outcome = kgw_worker_validate_stop_outcome_v1(
            KgwStopOutcomeMessageV1 {
                version: 1,
                outcome: "FAILED".to_string(),
                runtime_role: "node".to_string(),
                network: "mainnet".to_string(),
                worker_pid: 42,
                all_owned_components_terminal: terminal,
                evidence: None,
                error: Some("official shutdown fixture failure".to_string()),
            },
            "node",
            "mainnet",
            42,
        )
        .expect("typed FAILED outcome must be validated before classification");

        assert!(matches!(
            outcome,
            KgwValidatedStopOutcomeV1::Failed {
                all_owned_components_terminal,
                ..
            } if all_owned_components_terminal == expected_terminal
        ));
    }
}

#[test]
fn graceful_stop_waits_for_exit_and_drains_final_official_output() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");

    kgw_kgw_apply_node_settings_v1(
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
    .expect("test worker must start");

    let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
        .expect("test worker must stop gracefully");
    assert_contains_all(
        &stopped,
        &[
            "running=false",
            "graceful=true",
            "forced=false",
            "stop_outcome=STOPPED",
        ],
    );

    let logs = kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
        .expect("retained raw logs must remain queryable");
    let raw = logs
        .entries
        .iter()
        .map(|entry| entry.raw_text.as_str())
        .collect::<Vec<_>>();
    assert!(raw.contains(&"test-self-worker final official stdout"));
    assert!(raw.contains(&"test-self-worker final official stderr"));
    assert!(raw.iter().all(|line| {
        !line.contains("STOPPED")
            && !line.contains("graceful=true")
            && !line.contains("forced=true")
    }));
}

#[test]
fn hung_test_worker_uses_truthful_bounded_force_fallback() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_HANG_ON_STOP", "1");
    set_runtime_worker_test_env("KGW_TEST_PARENT_GRACEFUL_STOP_TIMEOUT_MS", "150");

    kgw_kgw_apply_node_settings_v1(
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
    .expect("hung test worker must start");

    let stopped = kgw_kgw_disable_network_v1("testnet10".to_string(), Some("node".to_string()))
        .expect("force fallback must terminate exact test worker");
    assert_contains_all(
        &stopped,
        &[
            "running=false",
            "graceful=false",
            "forced=true",
            "stop_outcome=FORCED",
            "graceful stop timed out",
        ],
    );

    let logs = kgw_kgw_runtime_logs_v1(
        Some("testnet10".to_string()),
        Some("node".to_string()),
        None,
    )
    .expect("forced path raw logs must remain queryable");
    assert!(logs.entries.iter().all(|entry| {
        !entry.raw_text.contains("FORCED") && !entry.raw_text.contains("graceful stop timed out")
    }));
}

#[test]
fn child_budget_timeout_classification_is_deterministically_forced() {
    const ITERATIONS: usize = 24;
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_TIMEOUT_ON_STOP", "1");
    set_runtime_worker_test_env("KGW_TEST_PARENT_GRACEFUL_STOP_TIMEOUT_MS", "500");

    for attempt in 1..=ITERATIONS {
        kgw_kgw_apply_node_settings_v1(
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
        .unwrap_or_else(|error| panic!("iteration {attempt} worker must start: {error}"));
        assert!(
            startup_control_files_for_test_worker("node", "mainnet").is_empty(),
            "iteration {attempt} READY control must be consumed before Stop"
        );

        let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
            .unwrap_or_else(|error| panic!("iteration {attempt} must force exact child: {error}"));
        assert_contains_all(
            &stopped,
            &[
                "running=false",
                "graceful=false",
                "forced=true",
                "stop_outcome=FORCED",
                "official shutdown exceeded child budget",
            ],
        );
        assert!(
            !stopped.contains("stop_failed=true"),
            "iteration {attempt} must never schedule-flip to terminal FAILED: {stopped}"
        );
        assert!(
            startup_control_files_for_test_worker("node", "mainnet").is_empty(),
            "iteration {attempt} post-READY timeout must not publish startup FAILED"
        );
    }

    let logs = kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
        .expect("forced failure-path raw logs must remain queryable");
    assert!(logs.entries.iter().all(|entry| {
        !entry.raw_text.contains("FAILED")
            && !entry.raw_text.contains("FORCED")
            && !entry.raw_text.contains("stop_outcome")
    }));
    let raw = logs
        .entries
        .iter()
        .map(|entry| entry.raw_text.as_str())
        .collect::<Vec<_>>();
    assert!(raw.contains(&"test-self-worker final official stdout"));
    assert!(raw.contains(&"test-self-worker final official stderr"));
}

#[test]
fn failed_stop_attestation_with_terminal_exit_removes_owner_truthfully() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_FAIL_ON_STOP", "1");

    kgw_kgw_apply_node_settings_v1(
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
    .expect("failure-path test worker must start");

    let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
        .expect("terminal FAILED outcome must return typed stopped state");
    assert_contains_all(
        &stopped,
        &[
            "running=false",
            "graceful=false",
            "forced=false",
            "stop_failed=true",
            "stop_outcome=FAILED",
            "test official shutdown failure",
        ],
    );

    let status = integrated_runtime_commands::kgw_runtime_owner_status_v1(
        Some("mainnet".to_string()),
        Some("node".to_string()),
    )
    .expect("terminal failed worker must no longer be registered");
    assert_contains_all(&status, &["node_running=false", "bridge_running=false"]);
}

#[test]
fn inprocess_bridge_listener_failure_attempts_owned_node_then_forces_exact_child() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    let marker = std::env::temp_dir().join("KaspaGateway").join(format!(
        "inprocess-bridge-owned-node-stop-{}.txt",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&marker);
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_BRIDGE_LISTENER_FAIL_ON_STOP", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_OWNED_NODE_STOP_MARKER_PATH", &marker);
    set_runtime_worker_test_env("KGW_TEST_PARENT_GRACEFUL_STOP_TIMEOUT_MS", "500");

    kgw_kgw_apply_node_settings_v1(
        "mainnet".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
        None,
        None,
        Some("bridge".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("in-process bridge failure fixture must start");

    let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("bridge".to_string()))
        .expect("unproven in-process bridge failure must force exact child");
    assert_contains_all(
        &stopped,
        &[
            "role=bridge",
            "running=false",
            "graceful=false",
            "forced=true",
            "stop_outcome=FORCED",
            "component:bridge-listener-0",
        ],
    );
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        "owned Node graceful Stop attempted"
    );
    let _ = std::fs::remove_file(marker);
}

#[test]
fn completed_graceful_stop_can_reacquire_the_same_role_and_network() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");

    for attempt in 1..=2 {
        let started = kgw_kgw_apply_node_settings_v1(
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
        .unwrap_or_else(|error| panic!("start attempt {attempt} must reacquire owner: {error}"));
        assert_contains_all(&started, &["runtime_state=running", "readiness=READY"]);

        let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
            .unwrap_or_else(|error| panic!("Stop attempt {attempt} must complete: {error}"));
        assert_contains_all(
            &stopped,
            &["running=false", "graceful=true", "forced=false"],
        );
    }
}

#[test]
fn repeated_stop_is_typed_terminal_and_idempotent() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();

    let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
        .expect("Stop without a live worker must be idempotent");
    assert_contains_all(
        &stopped,
        &[
            "role=node",
            "network=mainnet",
            "running=false",
            "graceful=false",
            "forced=false",
            "already_stopped=true",
        ],
    );
}

#[test]
fn stopping_mainnet_does_not_affect_testnet10() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");

    for network in ["mainnet", "testnet10"] {
        kgw_kgw_apply_node_settings_v1(
            network.to_string(),
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
        .unwrap_or_else(|error| panic!("{network} worker must start: {error}"));
    }

    kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
        .expect("mainnet Stop must complete");
    let testnet10 = integrated_runtime_commands::kgw_runtime_owner_status_v1(
        Some("testnet10".to_string()),
        Some("node".to_string()),
    )
    .expect("testnet10 status must remain queryable");
    assert_contains_all(&testnet10, &["network=testnet10", "running=true"]);
}

#[test]
fn shutdown_all_uses_graceful_bridge_first_order() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");

    for (network, role, node_kind, bridge_kind) in [
        ("mainnet", "node", "integrated-as-daemon", "disable"),
        ("testnet10", "bridge", "remote", "official-external-node"),
    ] {
        kgw_kgw_apply_node_settings_v1(
            network.to_string(),
            node_kind.to_string(),
            bridge_kind.to_string(),
            None,
            None,
            Some(role.to_string()),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_or_else(|error| panic!("{role} worker must start: {error}"));
    }

    let stopped = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1()
        .expect("shutdown-all must complete through graceful Stop");
    assert_eq!(stopped.matches("graceful=true").count(), 2);
    assert_eq!(stopped.matches("forced=false").count(), 2);
    assert!(
        stopped.find("role=bridge").unwrap() < stopped.find("role=node").unwrap(),
        "Bridge owners must stop before Node owners: {stopped}"
    );
}

#[test]
fn timeout_hierarchy_is_strict_and_race_free() {
    const {
        assert!(
            integrated_runtime_commands::KGW_PARENT_GRACEFUL_STOP_TIMEOUT_MS_V1
                > integrated_runtime_commands::KGW_CHILD_OFFICIAL_SHUTDOWN_BUDGET_MS_V1
        );
        assert!(70_000 > integrated_runtime_commands::KGW_PARENT_GRACEFUL_STOP_TIMEOUT_MS_V1);
    }
    let node_js = include_str!("../../frontend/src/tabs/kaspa-node/kaspa-node.js");
    let bridge_js = include_str!("../../frontend/src/tabs/kaspa-bridge/kaspa-bridge.js");
    assert!(node_js.contains("const KGW_NODE_STOP_INVOKE_TIMEOUT_MS = 70000"));
    assert!(bridge_js.contains("const KGW_BRIDGE_STOP_INVOKE_TIMEOUT_MS = 70000"));
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
fn production_self_worker_arguments_match_child_parser() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND");
    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_FAIL_COMMAND");
    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_MISSING_COMMAND");

    let mut settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        "mainnet".to_string(),
        "integrated-as-daemon".to_string(),
        "disable".to_string(),
    )
    .expect("mainnet settings should parse");
    settings.app_dir_name = "D:\\kgw-test\\nodes\\mainnet".to_string();
    settings.p2p_listen = Some("127.0.0.1:26111".to_string());
    settings.rpc_endpoint = "127.0.0.1:16110".to_string();
    settings.enable_utxo_index = true;
    settings.archival = false;

    let args =
        integrated_runtime_commands::kgw_worker_node_command_args_for_test_v1("node", &settings)
            .expect("production node worker args should be generated");

    assert_eq!(
        args,
        vec![
            "--kgw-self-worker",
            "node",
            "--network",
            "mainnet",
            "--appdir",
            "D:\\kgw-test\\nodes\\mainnet",
            "--rpc",
            "127.0.0.1:16110",
            "--listen",
            "127.0.0.1:26111",
            "--startup-control-path",
            &std::env::temp_dir()
                .join("KaspaGateway")
                .join("startup-control")
                .join("kgw-startup-control-args-test.json")
                .to_string_lossy(),
            "--stop-request-path",
            &std::env::temp_dir()
                .join("KaspaGateway")
                .join("stop-control")
                .join("kgw-stop-request-args-test.json")
                .to_string_lossy(),
            "--stop-outcome-path",
            &std::env::temp_dir()
                .join("KaspaGateway")
                .join("stop-control")
                .join("kgw-stop-outcome-args-test.json")
                .to_string_lossy(),
            "--utxoindex",
        ]
    );

    let lib_rs = include_str!("../src/lib.rs");
    for key in [
        "--kgw-self-worker",
        "--network",
        "--appdir",
        "--rpc",
        "--listen",
        "--startup-control-path",
        "--stop-request-path",
        "--stop-outcome-path",
        "--utxoindex",
    ] {
        assert!(
            lib_rs.contains(key),
            "child parser must recognize production argument `{key}`"
        );
    }
}

#[test]
fn bridge_inprocess_preview_preserves_isolated_node_p2p_listener() {
    let settings = integrated_runtime_commands::kgw_bridge_inprocess_preview_settings_for_test_v1(
        "stratum-bridge --node-mode inprocess --kaspa-rpc 127.0.0.1:36110 --listen 127.0.0.1:36111"
            .to_string(),
    )
    .expect("in-process Bridge preview should parse");

    assert_eq!(settings.rpc_endpoint, "127.0.0.1:36110");
    assert_eq!(settings.p2p_listen.as_deref(), Some("127.0.0.1:36111"));
    assert_eq!(
        settings.bridge_kind,
        kaspa_gateway_rk_node::BridgeNodeKind::OfficialInProcessNode
    );
}

#[test]
fn worker_entrypoint_is_selected_before_desktop_mode() {
    let main_rs = include_str!("../src/main.rs");
    let self_worker_index = main_rs
        .find("try_run_kgw_self_worker_from_args")
        .expect("main must check the self-worker entrypoint");
    let desktop_index = main_rs
        .find("kaspa_gateway_desktop_lib::run")
        .expect("main must run the desktop after self-worker check");

    assert!(
        self_worker_index < desktop_index,
        "self-worker entrypoint must be evaluated before desktop mode"
    );

    let lib_rs = include_str!("../src/lib.rs");
    assert_contains_all(
        lib_rs,
        &[
            "args.iter().any(|arg| arg == \"--kgw-self-worker\")",
            "kgw_run_node_self_worker",
            "std::process::exit(1)",
        ],
    );
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
        Some(
            "{\"secret\":\"abc\",\"wallet\":\"kaspa:abc\",\"completeCommand\":\"--rpc 127.0.0.1:16110\"}",
        ),
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
fn child_trace_line_redacts_sensitive_fields_without_truncating() {
    let line = "startup failure token=abc123 wallet=kaspa:qprv000000000000000000000000000000000000000000000000000000000000 diagnostic_tail=COMPLETE-CHILD-STDERR-END";
    let mirror = integrated_runtime_commands::kgw_worker_format_child_mirror_line_v1(
        "node", "mainnet", None, "stderr", 4242, line,
    );

    assert!(mirror.starts_with("[KGW_CHILD_STDERR] "));
    assert_contains_all(
        &mirror,
        &[
            "\"network\":\"mainnet\"",
            "\"runtimeRole\":\"node\"",
            "\"bridgeInstanceId\":null",
            "\"pid\":4242",
            "COMPLETE-CHILD-STDERR-END",
        ],
    );
    assert!(
        !mirror.contains("abc123") && !mirror.contains("kaspa:qprv"),
        "child mirror line must redact sensitive values: {mirror}"
    );

    let bridge_mirror = integrated_runtime_commands::kgw_worker_format_child_mirror_line_v1(
        "bridge",
        "testnet10",
        Some("bridge-a"),
        "stdout",
        4343,
        "bridge stdout ready",
    );
    assert_contains_all(
        &bridge_mirror,
        &[
            "\"network\":\"testnet10\"",
            "\"runtimeRole\":\"bridge\"",
            "\"bridgeInstanceId\":\"bridge-a\"",
            "\"stream\":\"stdout\"",
        ],
    );
}

#[test]
fn typed_raw_log_entry_preserves_text_and_keeps_metadata_separate() {
    let line = format!(
        "2026-07-28 15:10:50.082+03:00 [INFO ] kaspad path=C:\\Kaspa\\node;equals=value;json={{\"ok\":true}};unicode={}",
        '\u{03a9}'
    );
    let stdout = integrated_runtime_commands::kgw_raw_log_entry_for_test_v1(
        10, "mainnet", "node", None, "stdout", &line,
    );
    let stderr = integrated_runtime_commands::kgw_raw_log_entry_for_test_v1(
        11,
        "mainnet",
        "node",
        None,
        "stderr",
        "stderr raw line ; equals=value",
    );

    assert_eq!(stdout.raw_text, line);
    assert_eq!(stderr.raw_text, "stderr raw line ; equals=value");
    assert_eq!(stdout.network, "mainnet");
    assert_eq!(stdout.runtime_role, "node");
    assert_eq!(stdout.stream, "stdout");
    assert!(
        !stdout.raw_text.contains("kgw_raw_process_log_v1")
            && !stdout.raw_text.contains("source=self-worker")
            && !stdout.raw_text.contains("received_ms="),
        "transport metadata must stay out of raw_text: {stdout:?}"
    );
}

#[test]
fn typed_raw_log_text_is_sorted_by_sequence() {
    let second = integrated_runtime_commands::kgw_raw_log_entry_for_test_v1(
        2,
        "mainnet",
        "node",
        None,
        "stdout",
        "second raw line",
    );
    let first = integrated_runtime_commands::kgw_raw_log_entry_for_test_v1(
        1,
        "mainnet",
        "node",
        None,
        "stderr",
        "first raw line",
    );

    let text =
        integrated_runtime_commands::kgw_raw_log_text_from_entries_for_test_v1(vec![second, first]);

    assert_eq!(text, "first raw line\nsecond raw line");
}

#[test]
fn official_sentinel_stdout_and_stderr_use_the_production_pipe_reader_unchanged() {
    let stdout = "OFFICIAL-SENTINEL-STDOUT kgw_raw_process_log_v1 [KGW_CHILD_STDOUT] {\"eventKind\":\"diagnostic_transport_record\"}";
    let stderr = "OFFICIAL-SENTINEL-STDERR source=self-worker;runtime_role=node;received_ms=7";
    let logs = integrated_runtime_commands::kgw_empty_raw_log_buffer_for_test_v1();

    let stdout_reader = integrated_runtime_commands::kgw_capture_raw_pipe_for_test_v1(
        "node",
        "mainnet",
        "stdout",
        std::io::Cursor::new(format!("{stdout}\n")),
        std::sync::Arc::clone(&logs),
    );
    stdout_reader.join().expect("stdout reader must finish");
    let stderr_reader = integrated_runtime_commands::kgw_capture_raw_pipe_for_test_v1(
        "node",
        "mainnet",
        "stderr",
        std::io::Cursor::new(format!("{stderr}\n")),
        std::sync::Arc::clone(&logs),
    );
    stderr_reader.join().expect("stderr reader must finish");

    let entries = integrated_runtime_commands::kgw_raw_log_buffer_entries_for_test_v1(&logs);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].raw_text, stdout);
    assert_eq!(entries[0].stream, "stdout");
    assert_eq!(entries[0].network, "mainnet");
    assert_eq!(entries[0].runtime_role, "node");
    assert_eq!(entries[1].raw_text, stderr);
    assert_eq!(entries[1].stream, "stderr");
    assert!(entries[0].sequence < entries[1].sequence);
    assert_eq!(
        integrated_runtime_commands::kgw_raw_log_text_from_entries_for_test_v1(entries),
        format!("{stdout}\n{stderr}")
    );
}

#[test]
fn child_stdout_and_stderr_fixtures_survive_unchanged() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    let stdout_line = format!(
        "stdout fixture;equals=value;json={{\"line\":\"stdout\"}};unicode={};path=C:\\Kaspa\\kaspad.exe",
        '\u{03a9}'
    );
    let stderr_line = format!(
        "stderr fixture;equals=value;json={{\"line\":\"stderr\"}};unicode={};path=C:\\Kaspa\\stderr.log",
        '\u{03a9}'
    );
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDOUT", &stdout_line);
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDERR", &stderr_line);

    kgw_kgw_apply_node_settings_v1(
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
    .expect("custom fixture worker should start");

    let logs = kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
        .expect("typed raw logs should be readable");

    assert!(
        logs.entries
            .iter()
            .any(|entry| entry.stream == "stdout" && entry.raw_text == stdout_line),
        "captured stdout line must survive unchanged: {logs:?}"
    );
    assert!(
        logs.entries
            .iter()
            .any(|entry| entry.stream == "stderr" && entry.raw_text == stderr_line),
        "captured stderr line must survive unchanged: {logs:?}"
    );
    assert!(
        logs.entries.iter().all(|entry| {
            !entry.raw_text.contains("kgw_raw_process_log_v1")
                && !entry.raw_text.contains("network=mainnet")
                && !entry.raw_text.contains("source=self-worker")
                && !entry.raw_text.contains("runtime_role=node")
                && !entry.raw_text.contains("received_ms=")
        }),
        "transport metadata must never be copied into raw_text: {logs:?}"
    );
}

#[test]
fn raw_log_buffers_are_isolated_by_network_and_role_with_process_wide_bridge_output() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");

    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDOUT", "mainnet node stdout only");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDERR", "mainnet node stderr only");
    kgw_kgw_apply_node_settings_v1(
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
    .expect("mainnet node fixture should start");

    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDOUT", "testnet10 node stdout only");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDERR", "testnet10 node stderr only");
    kgw_kgw_apply_node_settings_v1(
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
    .expect("testnet10 node fixture should start");

    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDOUT", "mainnet bridge stdout only");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_STDERR", "mainnet bridge stderr only");
    kgw_kgw_apply_node_settings_v1(
        "mainnet".to_string(),
        "remote".to_string(),
        "official-external-node".to_string(),
        None,
        None,
        Some("bridge".to_string()),
        Some("bridge-a".to_string()),
        Some("port=5556".to_string()),
        Some("5556".to_string()),
        None,
        None,
    )
    .expect("mainnet bridge fixture should start");

    set_runtime_worker_test_env(
        "KGW_TEST_SELF_WORKER_STDOUT",
        "testnet10 bridge stdout only",
    );
    set_runtime_worker_test_env(
        "KGW_TEST_SELF_WORKER_STDERR",
        "testnet10 bridge stderr only",
    );
    kgw_kgw_apply_node_settings_v1(
        "testnet10".to_string(),
        "remote".to_string(),
        "official-external-node".to_string(),
        None,
        None,
        Some("bridge".to_string()),
        Some("bridge-b".to_string()),
        Some("port=6556".to_string()),
        Some("6556".to_string()),
        None,
        None,
    )
    .expect("testnet10 bridge fixture should start");

    let mainnet_node =
        kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
            .expect("mainnet node logs should be readable");
    let testnet10_node = kgw_kgw_runtime_logs_v1(
        Some("testnet10".to_string()),
        Some("node".to_string()),
        None,
    )
    .expect("testnet10 node logs should be readable");
    let mainnet_bridge = kgw_kgw_runtime_logs_v1(
        Some("mainnet".to_string()),
        Some("bridge".to_string()),
        Some("bridge-a".to_string()),
    )
    .expect("mainnet bridge logs should be readable");
    let wrong_bridge_instance = kgw_kgw_runtime_logs_v1(
        Some("mainnet".to_string()),
        Some("bridge".to_string()),
        Some("bridge-b".to_string()),
    )
    .expect("wrong bridge instance logs should be readable");
    let testnet10_bridge = kgw_kgw_runtime_logs_v1(
        Some("testnet10".to_string()),
        Some("bridge".to_string()),
        Some("bridge-b".to_string()),
    )
    .expect("testnet10 bridge logs should be readable");
    let wrong_testnet10_bridge_instance = kgw_kgw_runtime_logs_v1(
        Some("testnet10".to_string()),
        Some("bridge".to_string()),
        Some("bridge-a".to_string()),
    )
    .expect("wrong testnet10 bridge instance logs should be readable");

    assert!(
        mainnet_node
            .entries
            .iter()
            .all(|entry| entry.network == "mainnet" && entry.runtime_role == "node"),
        "mainnet node entries must not mix roles or networks: {mainnet_node:?}"
    );
    assert!(
        testnet10_node
            .entries
            .iter()
            .all(|entry| entry.network == "testnet10" && entry.runtime_role == "node"),
        "testnet10 node entries must not mix mainnet records: {testnet10_node:?}"
    );
    assert!(
        mainnet_bridge.entries.iter().all(|entry| {
            entry.network == "mainnet"
                && entry.runtime_role == "bridge"
                && entry.bridge_instance_id.is_none()
        }),
        "bridge entries must stay scoped to role/network without invented listener attribution: {mainnet_bridge:?}"
    );
    assert!(
        mainnet_node
            .entries
            .iter()
            .any(|entry| entry.raw_text == "mainnet node stdout only")
            && !mainnet_node
                .entries
                .iter()
                .any(|entry| entry.raw_text.contains("bridge stdout")),
        "node logs must not contain bridge records: {mainnet_node:?}"
    );
    assert!(
        testnet10_node
            .entries
            .iter()
            .any(|entry| entry.raw_text == "testnet10 node stdout only")
            && !testnet10_node
                .entries
                .iter()
                .any(|entry| entry.raw_text.contains("mainnet node")),
        "testnet10 logs must not contain mainnet records: {testnet10_node:?}"
    );
    assert!(
        mainnet_bridge
            .entries
            .iter()
            .any(|entry| entry.raw_text == "mainnet bridge stdout only"),
        "bridge logs must contain actual bridge child output: {mainnet_bridge:?}"
    );
    assert!(
        testnet10_bridge.entries.iter().all(|entry| {
            entry.network == "testnet10"
                && entry.runtime_role == "bridge"
                && entry.bridge_instance_id.is_none()
        }),
        "testnet10 bridge entries must stay scoped to network and role: {testnet10_bridge:?}"
    );
    assert!(
        testnet10_bridge
            .entries
            .iter()
            .any(|entry| entry.raw_text == "testnet10 bridge stdout only")
            && !testnet10_bridge
                .entries
                .iter()
                .any(|entry| entry.raw_text.contains("mainnet bridge")),
        "testnet10 bridge logs must not contain mainnet bridge records: {testnet10_bridge:?}"
    );
    assert_eq!(
        wrong_bridge_instance
            .entries
            .iter()
            .map(|entry| &entry.raw_text)
            .collect::<Vec<_>>(),
        mainnet_bridge
            .entries
            .iter()
            .map(|entry| &entry.raw_text)
            .collect::<Vec<_>>(),
        "Bridge logs are process-wide and must not be filtered by an unprovable listener ID"
    );
    assert_eq!(
        wrong_testnet10_bridge_instance
            .entries
            .iter()
            .map(|entry| &entry.raw_text)
            .collect::<Vec<_>>(),
        testnet10_bridge
            .entries
            .iter()
            .map(|entry| &entry.raw_text)
            .collect::<Vec<_>>(),
        "Testnet10 Bridge logs are process-wide"
    );

    kgw_kgw_runtime_clear_logs_v1(
        Some("testnet10".to_string()),
        Some("node".to_string()),
        None,
    )
    .expect("testnet10 node clear should succeed");

    let cleared_testnet10 = kgw_kgw_runtime_logs_v1(
        Some("testnet10".to_string()),
        Some("node".to_string()),
        None,
    )
    .expect("cleared testnet10 node logs should be readable");
    let remaining_mainnet =
        kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
            .expect("mainnet node logs should remain readable");

    assert!(
        cleared_testnet10.entries.is_empty(),
        "selected buffer should clear"
    );
    assert!(
        remaining_mainnet
            .entries
            .iter()
            .any(|entry| entry.raw_text == "mainnet node stdout only"),
        "clearing testnet10 node must not clear mainnet node: {remaining_mainnet:?}"
    );

    kgw_kgw_runtime_clear_logs_v1(
        Some("mainnet".to_string()),
        Some("bridge".to_string()),
        Some("bridge-a".to_string()),
    )
    .expect("mainnet bridge clear should succeed");

    let cleared_bridge = kgw_kgw_runtime_logs_v1(
        Some("mainnet".to_string()),
        Some("bridge".to_string()),
        Some("bridge-a".to_string()),
    )
    .expect("cleared bridge logs should be readable");
    let remaining_node =
        kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
            .expect("mainnet node logs should remain readable after bridge clear");

    assert!(
        cleared_bridge.entries.is_empty(),
        "selected bridge buffer should clear"
    );
    assert!(
        remaining_node
            .entries
            .iter()
            .any(|entry| entry.raw_text == "mainnet node stdout only"),
        "clearing bridge logs must not clear node logs: {remaining_node:?}"
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
fn mainnet_and_testnet10_test_workers_stay_alive_through_startup_verification() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();

    for network in ["mainnet", "testnet10"] {
        let started = kgw_kgw_apply_node_settings_v1(
            network.to_string(),
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
        .unwrap_or_else(|error| panic!("{network} test worker should start: {error}"));

        assert_contains_all(
            &started,
            &[
                "parallel-owned-self-worker started",
                &format!("network={network}"),
                "pid=",
                "runtime_state=running",
                "same_exe=true",
                "external_kaspad_exe=false",
            ],
        );

        let status = kgw_runtime_owner_summary_for_test(network);
        assert_contains_all(
            &status,
            &[
                "parallel-owned-self-worker status",
                &format!("network={network}"),
                "role=node",
                "running=true",
            ],
        );

        let stopped = kgw_kgw_disable_network_v1(network.to_string(), Some("node".to_string()))
            .unwrap_or_else(|error| panic!("{network} test worker should stop: {error}"));
        assert_contains_all(
            &stopped,
            &[
                "parallel-owned-self-worker stopped",
                &format!("network={network}"),
                "pid=",
            ],
        );
    }

    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND");
}

fn kgw_runtime_owner_summary_for_test(network: &str) -> String {
    integrated_runtime_commands::kgw_runtime_owner_status_v1(
        Some(network.to_string()),
        Some("node".to_string()),
    )
    .unwrap_or_else(|error| panic!("{network} status should be readable: {error}"))
}

#[test]
fn duplicate_owner_for_one_network_is_rejected() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
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
        kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
            .expect("mainnet logs should be readable");

    assert!(
        mainnet_logs
            .entries
            .iter()
            .all(|entry| entry.network == "mainnet" && entry.runtime_role == "node"),
        "mainnet node log entries must keep typed network and role metadata: {mainnet_logs:?}"
    );
    assert!(
        mainnet_logs
            .entries
            .iter()
            .any(|entry| entry.stream == "stdout")
            && mainnet_logs
                .entries
                .iter()
                .any(|entry| entry.stream == "stderr"),
        "mainnet node logs must include typed stdout and stderr entries: {mainnet_logs:?}"
    );
    assert!(
        mainnet_logs.entries.iter().any(|entry| {
            entry.stream == "stdout"
                && entry.raw_text == "test-self-worker stdout role=node network=mainnet"
        }) && mainnet_logs.entries.iter().any(|entry| {
            entry.stream == "stderr"
                && entry.raw_text == "test-self-worker stderr role=node network=mainnet"
        }),
        "raw process text must preserve child stdout and stderr exactly: {mainnet_logs:?}"
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
    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND");
}

#[test]
fn node_start_trace_uses_same_exe_mode_not_external() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_START_TRACE", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    let _ = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1();
    let _ = integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();

    let started = kgw_kgw_apply_node_settings_v1(
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
    .expect("test worker start should succeed");
    assert_contains_all(
        &started,
        &[
            "parallel-owned-self-worker started",
            "network=mainnet",
            "node_mode=same-exe-self-worker",
        ],
    );

    let trace_text = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1().join("\n");
    assert_contains_all(
        &trace_text,
        &[
            "\"stage\":\"native.spawn_plan_created\"",
            "\\\"nodeMode\\\":\\\"same-exe-self-worker\\\"",
            "\\\"externalKaspadExe\\\":false",
        ],
    );
    assert!(
        !trace_text.contains("\\\"runtimeRole\\\":\\\"node\\\",\\\"nodeMode\\\":\\\"external\\\"")
            && !trace_text.contains("\"runtimeRole\":\"node\",\"nodeMode\":\"external\""),
        "node starts must not be traced as external mode: {trace_text}"
    );

    let _ = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()));
    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND");
    remove_runtime_worker_test_env("KGW_START_TRACE");
}

#[test]
fn success_response_contains_process_start_evidence_and_stream_logs() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
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

    let logs = kgw_kgw_runtime_logs_v1(
        Some("testnet10".to_string()),
        Some("node".to_string()),
        None,
    )
    .expect("logs should be readable");

    assert!(
        logs.entries.iter().any(|entry| entry.stream == "stdout"
            && entry.raw_text == "test-self-worker stdout role=node network=testnet10"),
        "stdout raw_text must survive unchanged: {logs:?}"
    );
    assert!(
        logs.entries.iter().any(|entry| entry.stream == "stderr"
            && entry.raw_text == "test-self-worker stderr role=node network=testnet10"),
        "stderr raw_text must survive unchanged: {logs:?}"
    );
    assert!(
        logs.entries.iter().all(|entry| {
            entry.network == "testnet10"
                && entry.runtime_role == "node"
                && entry.source == "self-worker"
                && !entry.raw_text.contains("kgw_raw_process_log_v1")
                && !entry.raw_text.contains("source=self-worker")
                && !entry.raw_text.contains("runtime_role=")
                && !entry.raw_text.contains("received_ms=")
        }),
        "typed metadata must stay outside raw_text: {logs:?}"
    );

    let _ = kgw_kgw_disable_network_v1("testnet10".to_string(), Some("node".to_string()));
    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND");
}

#[test]
fn early_self_worker_exit_preserves_complete_safe_stderr_and_returns_error() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    let complete_tail = "COMPLETE-SELF-WORKER-STDERR-END";
    let child_stderr = format!(
        "forced startup failure token=abc123 wallet=kaspa:qprv000000000000000000000000000000000000000000000000000000000000 diagnostic_tail={complete_tail}"
    );
    set_runtime_worker_test_env("KGW_START_TRACE", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_FAIL_COMMAND", &child_stderr);
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
    .expect_err("early child exit must remain an error");

    assert_contains_all(
        &error,
        &[
            "self-worker exited before role readiness",
            "role=node",
            "network=mainnet",
            "exit_code=1",
            complete_tail,
        ],
    );
    assert!(
        !error.contains("parallel-owned-self-worker started"),
        "startup success must not be returned after an early child exit: {error}"
    );
    assert!(
        !error.contains("abc123") && !error.contains("kaspa:qprv"),
        "startup error must expose only safe stderr: {error}"
    );

    let raw = kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
        .expect("pre-failure official stream must remain queryable");
    assert!(
        raw.entries
            .iter()
            .any(|entry| entry.stream == "stderr" && entry.raw_text == child_stderr),
        "genuine child stderr before startup failure must remain raw and unmodified: {raw:?}"
    );
    assert!(
        raw.entries.iter().all(|entry| !entry
            .raw_text
            .contains("self-worker exited before role readiness")),
        "typed terminal wrapper error must stay outside raw entries: {raw:?}"
    );

    let trace_text = integrated_runtime_commands::kgw_start_trace_test_take_lines_v1().join("\n");
    assert_contains_all(
        &trace_text,
        &[
            "[KGW_CHILD_STDOUT]",
            "[KGW_CHILD_STDERR]",
            "\"network\":\"mainnet\"",
            "\"runtimeRole\":\"node\"",
            "\"stage\":\"native.startup_response_returned\"",
            "\"result\":\"error\"",
            complete_tail,
        ],
    );
    assert!(
        !trace_text.contains("abc123") && !trace_text.contains("kaspa:qprv"),
        "trace output must redact sensitive child fields: {trace_text}"
    );

    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_FAIL_COMMAND");
    remove_runtime_worker_test_env("KGW_START_TRACE");
}

#[test]
fn delayed_ready_keeps_start_non_running_until_attestation() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_READY_DELAY_MS", "1100");

    let started_at = std::time::Instant::now();
    let started = kgw_kgw_apply_node_settings_v1(
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
    .expect("delayed READY should eventually start");

    assert!(
        started_at.elapsed() >= std::time::Duration::from_millis(1000),
        "Start must not return Running before READY"
    );
    assert_contains_all(
        &started,
        &[
            "runtime_state=running",
            "readiness=READY",
            "readiness_evidence=test-role-ready",
        ],
    );
}

#[test]
fn delayed_failed_after_old_liveness_window_leaves_no_false_owner() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env(
        "KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND",
        "delayed RPC readiness failure after 750ms",
    );

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
    .expect_err("delayed FAILED must reject Start");

    assert_contains_all(
        &error,
        &[
            "role startup failed",
            "role=node",
            "network=mainnet",
            "delayed RPC readiness failure after 750ms",
        ],
    );
    let status = kgw_runtime_owner_summary_for_test("mainnet");
    assert!(
        !status.contains("running=true") && !status.contains("readiness=READY"),
        "failed startup must leave no false owner status: {status}"
    );
}

fn javascript_timeout_constant(source: &str, name: &str) -> u64 {
    let marker = format!("const {name} = ");
    source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("missing JavaScript timeout constant {name}"))
        .1
        .split_once(';')
        .unwrap_or_else(|| panic!("unterminated JavaScript timeout constant {name}"))
        .0
        .trim()
        .parse()
        .unwrap_or_else(|error| panic!("invalid JavaScript timeout constant {name}: {error}"))
}

#[test]
fn startup_timeout_hierarchy_is_strict_for_node_and_bridge() {
    let bridge_child = kaspa_gateway_rk_bridge::KGW_BRIDGE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS;
    let bridge_parent =
        integrated_runtime_commands::KGW_BRIDGE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1;
    let bridge_ui = javascript_timeout_constant(
        include_str!("../../frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
        "KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS",
    );
    assert!(bridge_parent > bridge_child);
    assert!(bridge_ui > bridge_parent);

    let node_child = integrated_runtime_commands::KGW_NODE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS_V1;
    let node_parent =
        integrated_runtime_commands::KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1;
    let node_ui = javascript_timeout_constant(
        include_str!("../../frontend/src/tabs/kaspa-node/kaspa-node.js"),
        "KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS",
    );
    assert!(node_parent > node_child);
    assert!(node_ui > node_parent);
}

#[test]
fn bridge_failed_attestation_leaves_no_false_owner_registered() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env(
        "KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND",
        "foreign accepting listener cannot attest KGW ownership",
    );

    let error = kgw_kgw_apply_node_settings_v1(
        "mainnet".to_string(),
        "remote".to_string(),
        "official-external-node".to_string(),
        None,
        None,
        Some("bridge".to_string()),
        None,
        None,
        None,
        None,
        None,
    )
    .expect_err("typed Bridge FAILED must reject Start");

    assert_contains_all(
        &error,
        &[
            "role startup failed",
            "role=bridge",
            "network=mainnet",
            "foreign accepting listener cannot attest KGW ownership",
        ],
    );
    let status = integrated_runtime_commands::kgw_runtime_owner_status_v1(
        Some("mainnet".to_string()),
        Some("bridge".to_string()),
    )
    .expect("Bridge owner status must remain queryable after startup failure");
    assert!(
        !status.contains("running=true") && !status.contains("readiness=READY"),
        "failed Bridge startup must leave no false owner status: {status}"
    );
}

#[test]
fn ready_attestation_requires_nonempty_evidence() {
    let error = kgw_worker_validate_startup_attestation_v1(
        KgwStartupControlMessageV1 {
            version: 1,
            outcome: "READY".to_string(),
            runtime_role: "node".to_string(),
            network: "mainnet".to_string(),
            evidence: None,
            error: None,
        },
        "node",
        "mainnet",
    )
    .expect_err("READY without role evidence must be rejected");

    assert_contains_all(
        &error,
        &["READY is missing evidence", "role=node", "network=mainnet"],
    );
}

#[test]
fn spawn_failure_remains_an_error() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    let missing = std::env::temp_dir().join("kgw-missing-self-worker-command.exe");
    set_runtime_worker_test_env("KGW_START_TRACE", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_MISSING_COMMAND", &missing);
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
    assert_eq!(
        integrated_runtime_commands::kgw_raw_process_log_buffer_count_for_test_v1(),
        1,
        "spawn failure may create only an empty retained pipe-owned buffer"
    );
    let raw = kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some("node".to_string()), None)
        .expect("spawn-failure raw report must remain queryable");
    assert!(
        raw.entries.is_empty(),
        "typed spawn failure must never be synthesized into raw entries: {raw:?}"
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

    remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_MISSING_COMMAND");
    remove_runtime_worker_test_env("KGW_START_TRACE");
}
