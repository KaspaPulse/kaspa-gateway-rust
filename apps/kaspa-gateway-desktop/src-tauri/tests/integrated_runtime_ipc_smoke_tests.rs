#[path = "../src/integrated_runtime_commands.rs"]
mod integrated_runtime_commands;

use integrated_runtime_commands::{
    KgwStartupControlMessageV1, KgwStopOutcomeMessageV1, KgwValidatedStopOutcomeV1,
    kgw_kgw_disable_network_v1, kgw_kgw_node_bridge_service_plan_v1, kgw_kgw_runtime_clear_logs_v1,
    kgw_kgw_runtime_logs_v1, kgw_runtime_owner_summary_v1,
    kgw_worker_validate_startup_attestation_v1, kgw_worker_validate_stop_outcome_v1,
};
use std::sync::{Mutex, OnceLock};

#[allow(clippy::too_many_arguments)]
fn kgw_kgw_apply_node_settings_v1(
    network: String,
    node_kind: String,
    bridge_kind: String,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
    runtime_role: Option<String>,
    bridge_active_instance_id: Option<String>,
    bridge_active_instance: Option<String>,
    bridge_active_instance_port: Option<String>,
    bridge_structured_instances: Option<String>,
    experimental_network_opt_in: Option<bool>,
) -> Result<String, String> {
    integrated_runtime_commands::kgw_kgw_apply_node_settings_v1(
        network,
        node_kind,
        bridge_kind,
        node_command_preview,
        bridge_command_preview,
        runtime_role,
        bridge_active_instance_id,
        bridge_active_instance,
        bridge_active_instance_port,
        bridge_structured_instances,
        None,
        experimental_network_opt_in,
    )
}

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
        "KGW_TEST_SELF_WORKER_EXIT_AFTER_READY_MS",
        "KGW_TEST_SELF_WORKER_OWNED_NODE_STOP_MARKER_PATH",
        "KGW_TEST_SELF_WORKER_PARENT_PID",
        "KGW_TEST_SELF_WORKER_PARENT_START_TIME",
        "KGW_TEST_SELF_WORKER_PARENT_EXECUTABLE",
        "KGW_TEST_PARENT_FIXTURE_ROLE",
        "KGW_TEST_PARENT_FIXTURE_NETWORK",
        "KGW_TEST_PARENT_FIXTURE_NODE_KIND",
        "KGW_TEST_PARENT_FIXTURE_BRIDGE_KIND",
        "KGW_TEST_PARENT_FIXTURE_READY_PATH",
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

fn runtime_owner_lease_path(role: &str, network: &str) -> std::path::PathBuf {
    kaspa_gateway_config::default_user_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("KaspaGateway").join("nodes"))
        .join("runtime-ownership")
        .join(format!("{role}-{network}.json"))
}

fn runtime_owner_worker_paths(role: &str, network: &str) -> Vec<std::path::PathBuf> {
    let expected_prefix = format!("{role}-{network}.");
    let directory = runtime_owner_lease_path(role, network)
        .parent()
        .expect("runtime ownership directory")
        .to_path_buf();
    let mut paths = std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|name| {
                    name.starts_with(&expected_prefix) && name.ends_with(".worker.json")
                })
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

#[test]
fn runtime_owner_lease_requires_exact_process_identity() {
    let identity =
        integrated_runtime_commands::kgw_process_identity_for_worker_v1(std::process::id())
            .expect("current process identity must be available");
    let appdir = std::env::temp_dir()
        .join("KaspaGateway")
        .join("nodes")
        .join("mainnet")
        .to_string_lossy()
        .to_string();
    let (lease, worker) = integrated_runtime_commands::kgw_runtime_owner_lease_fixture_v1(
        "node",
        "mainnet",
        &appdir,
        "same-exe-self-worker",
        &identity,
        &identity,
    );
    assert!(
        integrated_runtime_commands::kgw_runtime_owner_lease_identity_matches_for_test_v1(
            &lease,
            &worker,
            "node",
            "mainnet",
            &appdir,
            &identity.executable,
            &identity,
        )
    );
    let forged_pid = integrated_runtime_commands::KgwProcessIdentityV1 {
        pid: identity.pid.saturating_add(1),
        ..identity.clone()
    };
    assert!(
        !integrated_runtime_commands::kgw_runtime_owner_lease_identity_matches_for_test_v1(
            &lease,
            &worker,
            "node",
            "mainnet",
            &appdir,
            &identity.executable,
            &forged_pid,
        ),
        "a forged or PID-reused process identity must never authorize ownership"
    );
    let foreign_executable = integrated_runtime_commands::KgwProcessIdentityV1 {
        executable: format!("{}.foreign", identity.executable),
        ..identity.clone()
    };
    assert!(
        !integrated_runtime_commands::kgw_runtime_owner_lease_identity_matches_for_test_v1(
            &lease,
            &worker,
            "node",
            "mainnet",
            &appdir,
            &identity.executable,
            &foreign_executable,
        ),
        "a foreign executable must never authorize ownership"
    );
}

#[test]
fn live_smoke_parent_accepts_only_valid_stable_network_runtime_settings() {
    let appdir = std::env::temp_dir()
        .join("KaspaGateway")
        .join("nodes")
        .join("mainnet")
        .to_string_lossy()
        .to_string();
    let accepted = integrated_runtime_commands::kgw_validate_live_smoke_parent_settings_v1(
        "mainnet",
        &appdir,
        "127.0.0.1:16110",
        Some("127.0.0.1:16111"),
    )
    .expect("valid isolated mainnet smoke settings must pass");
    assert_eq!(accepted.network.as_str(), "mainnet");
    assert!(
        integrated_runtime_commands::kgw_validate_live_smoke_parent_settings_v1(
            "testnet12",
            &appdir,
            "127.0.0.1:16310",
            None,
        )
        .is_err(),
        "the live smoke parent must never start experimental testnet12"
    );
    assert!(
        integrated_runtime_commands::kgw_validate_live_smoke_parent_settings_v1(
            "mainnet",
            &appdir,
            "0.0.0.0:16110",
            None,
        )
        .is_err(),
        "the live smoke parent must keep RPC on loopback"
    );
}

#[test]
fn ready_worker_publishes_and_normal_stop_removes_exact_owner_lease() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    let lease_path = runtime_owner_lease_path("node", "mainnet");
    let _ = std::fs::remove_file(&lease_path);

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
    assert_contains_all(&started, &["parent_bound=true", "reconciliation=none"]);
    let lease: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&lease_path).expect("READY worker must publish owner lease"),
    )
    .expect("owner lease must be typed JSON");
    assert_eq!(lease["version"], 1);
    assert_eq!(lease["runtimeRole"], "node");
    assert_eq!(lease["network"], "mainnet");
    assert_eq!(lease["parentPid"], std::process::id());
    assert!(lease["publishedMs"].as_u64().is_some_and(|value| value > 0));
    let worker_paths = runtime_owner_worker_paths("node", "mainnet");
    assert_eq!(
        worker_paths.len(),
        1,
        "READY worker must publish one identity sidecar"
    );
    let worker: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&worker_paths[0]).expect("READY worker identity must be readable"),
    )
    .expect("worker identity must be typed JSON");
    assert!(worker["workerPid"].as_u64().is_some_and(|pid| pid > 0));

    let stopped = kgw_kgw_disable_network_v1("mainnet".to_string(), Some("node".to_string()))
        .expect("normal test worker Stop should succeed");
    assert_contains_all(&stopped, &["graceful=true", "forced=false"]);
    assert!(
        !lease_path.exists(),
        "terminal exact worker Stop must remove its durable lease"
    );
    assert!(
        runtime_owner_worker_paths("node", "mainnet").is_empty(),
        "terminal exact worker Stop must remove its identity sidecar"
    );
}

#[cfg(unix)]
#[test]
fn runtime_owner_reservation_rejects_symlink_and_uses_restrictive_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let identity =
        integrated_runtime_commands::kgw_process_identity_for_worker_v1(std::process::id())
            .expect("current process identity must be available");
    let appdir = std::env::temp_dir()
        .join("KaspaGateway")
        .join("nodes")
        .join("mainnet")
        .to_string_lossy()
        .to_string();
    let (lease, _worker) = integrated_runtime_commands::kgw_runtime_owner_lease_fixture_v1(
        "node",
        "mainnet",
        &appdir,
        "same-exe-self-worker",
        &identity,
        &identity,
    );

    let base = std::env::temp_dir().join(format!(
        "kgw-owner-permissions-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).expect("permission fixture directory");
    let lease_path = base.join("node-mainnet.json");
    integrated_runtime_commands::kgw_runtime_owner_reserve_for_test_v1(&lease_path, &lease)
        .expect("regular owner reservation must succeed");
    assert_eq!(
        std::fs::metadata(&base)
            .expect("directory metadata")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::metadata(&lease_path)
            .expect("lease metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let worker_path = integrated_runtime_commands::kgw_runtime_owner_publish_worker_for_test_v1(
        &lease_path,
        &identity,
        &identity,
    )
    .expect("worker identity publication must succeed");
    assert_eq!(
        std::fs::metadata(&worker_path)
            .expect("worker identity metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );

    let collision_lease_path = base.join("bridge-mainnet.json");
    integrated_runtime_commands::kgw_runtime_owner_reserve_for_test_v1(
        &collision_lease_path,
        &lease,
    )
    .expect("collision reservation must succeed");
    let collision_worker_path =
        integrated_runtime_commands::kgw_runtime_owner_worker_path_for_test_v1(
            &collision_lease_path,
            &identity,
        );
    std::fs::write(&collision_worker_path, "foreign").expect("foreign worker identity target");
    let collision_error =
        integrated_runtime_commands::kgw_runtime_owner_publish_worker_for_test_v1(
            &collision_lease_path,
            &identity,
            &identity,
        )
        .expect_err("worker identity publication must refuse an existing target");
    assert!(collision_error.contains("publish runtime owner worker identity failed"));
    assert_eq!(
        std::fs::read_to_string(&collision_worker_path)
            .expect("foreign worker identity remains readable"),
        "foreign"
    );
    std::fs::remove_file(collision_worker_path).expect("remove collision target");
    std::fs::remove_file(collision_lease_path).expect("remove collision reservation");

    let foreign_path = base.join("foreign.json");
    std::fs::write(&foreign_path, "foreign").expect("foreign target");
    let symlink_path = base.join("symlink.json");
    std::os::unix::fs::symlink(&foreign_path, &symlink_path).expect("hostile symlink fixture");
    let error =
        integrated_runtime_commands::kgw_runtime_owner_reserve_for_test_v1(&symlink_path, &lease)
            .expect_err("create-new owner reservation must reject a symlink target");
    assert!(error.contains("reserve runtime owner lease failed"));
    assert_eq!(
        std::fs::read_to_string(&foreign_path).expect("foreign target remains readable"),
        "foreign"
    );
    std::fs::remove_file(symlink_path).expect("remove symlink fixture");
    std::fs::remove_file(foreign_path).expect("remove foreign fixture");
    std::fs::remove_file(worker_path).expect("remove worker identity fixture");
    std::fs::remove_file(lease_path).expect("remove lease fixture");
    std::fs::remove_dir(base).expect("remove permission fixture directory");
}

fn wait_for_file(path: &std::path::Path, timeout: std::time::Duration) {
    let deadline = std::time::Instant::now() + timeout;
    while !path.is_file() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {}",
            path.display()
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[test]
fn nested_parent_fixture() {
    if std::env::var_os("KGW_TEST_NESTED_PARENT_CHILD").is_none() {
        return;
    }
    let role = std::env::var("KGW_TEST_PARENT_FIXTURE_ROLE").expect("fixture role");
    let network = std::env::var("KGW_TEST_PARENT_FIXTURE_NETWORK").expect("fixture network");
    let node_kind = std::env::var("KGW_TEST_PARENT_FIXTURE_NODE_KIND").expect("fixture node kind");
    let bridge_kind =
        std::env::var("KGW_TEST_PARENT_FIXTURE_BRIDGE_KIND").expect("fixture bridge kind");
    let ready_path = std::path::PathBuf::from(
        std::env::var_os("KGW_TEST_PARENT_FIXTURE_READY_PATH").expect("fixture ready path"),
    );
    let started = kgw_kgw_apply_node_settings_v1(
        network,
        node_kind,
        bridge_kind,
        None,
        None,
        Some(role),
        None,
        None,
        None,
        None,
        None,
    )
    .expect("nested parent fixture must start its exact child");
    std::fs::write(ready_path, started).expect("nested parent fixture must report READY");
    loop {
        std::thread::park_timeout(std::time::Duration::from_secs(60));
    }
}

fn prove_parent_loss_cleanup(role: &str, node_kind: &str, bridge_kind: &str) {
    let lease_path = runtime_owner_lease_path(role, "mainnet");
    let _ = std::fs::remove_file(&lease_path);
    for path in runtime_owner_worker_paths(role, "mainnet") {
        let _ = std::fs::remove_file(path);
    }
    let ready_path = std::env::temp_dir()
        .join("KaspaGateway")
        .join(format!("parent-loss-{role}-{}.ready", std::process::id()));
    let _ = std::fs::remove_file(&ready_path);
    let mut parent = std::process::Command::new(
        std::env::current_exe().expect("test executable path must be available"),
    )
    .arg("--exact")
    .arg("nested_parent_fixture")
    .arg("--nocapture")
    .env("KGW_TEST_NESTED_PARENT_CHILD", "1")
    .env("KGW_TEST_SELF_WORKER_COMMAND", "1")
    .env("KGW_TEST_PARENT_FIXTURE_ROLE", role)
    .env("KGW_TEST_PARENT_FIXTURE_NETWORK", "mainnet")
    .env("KGW_TEST_PARENT_FIXTURE_NODE_KIND", node_kind)
    .env("KGW_TEST_PARENT_FIXTURE_BRIDGE_KIND", bridge_kind)
    .env("KGW_TEST_PARENT_FIXTURE_READY_PATH", &ready_path)
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .spawn()
    .expect("nested parent fixture must spawn");

    wait_for_file(&ready_path, std::time::Duration::from_secs(10));
    wait_for_file(&lease_path, std::time::Duration::from_secs(10));
    let lease_bytes = std::fs::read(&lease_path).expect("nested worker lease must be readable");
    let lease: serde_json::Value =
        serde_json::from_slice(&lease_bytes).expect("nested worker lease must be typed JSON");
    assert_eq!(lease["parentPid"].as_u64(), Some(u64::from(parent.id())));
    let worker_paths = runtime_owner_worker_paths(role, "mainnet");
    assert_eq!(
        worker_paths.len(),
        1,
        "nested worker must publish one identity sidecar"
    );
    let worker_bytes =
        std::fs::read(&worker_paths[0]).expect("nested worker identity must be readable");
    let worker: serde_json::Value =
        serde_json::from_slice(&worker_bytes).expect("nested worker identity must be typed JSON");
    let worker_pid = worker["workerPid"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .expect("nested worker PID must be present");
    let worker_identity =
        integrated_runtime_commands::kgw_process_identity_for_worker_v1(worker_pid)
            .expect("nested exact child must be alive after READY");
    assert_ne!(
        worker_pid,
        parent.id(),
        "nested fixture must prove a distinct exact worker"
    );

    parent
        .kill()
        .expect("task-owned nested parent must terminate");
    parent
        .wait()
        .expect("task-owned nested parent must be reaped");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let same_worker_alive =
            integrated_runtime_commands::kgw_process_identity_for_worker_v1(worker_pid)
                .is_ok_and(|identity| identity == worker_identity);
        if !same_worker_alive {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "READY {role} worker survived its exact desktop parent"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(
        lease_path.exists(),
        "parent-loss worker must leave durable metadata for relaunch reconciliation"
    );
    assert!(
        worker_paths[0].exists(),
        "parent-loss worker must not mutate its ownership metadata"
    );
    assert_eq!(
        std::fs::read(&lease_path).expect("stale immutable reservation remains readable"),
        lease_bytes,
        "parent-loss worker must leave the immutable reservation byte-for-byte unchanged"
    );
    assert_eq!(
        std::fs::read(&worker_paths[0]).expect("stale worker identity remains readable"),
        worker_bytes,
        "parent-loss worker must leave the identity sidecar byte-for-byte unchanged"
    );
    let appdir = std::env::temp_dir()
        .join("KaspaGateway")
        .join("nodes")
        .join("mainnet")
        .to_string_lossy()
        .to_string();
    let reconciliation = integrated_runtime_commands::kgw_runtime_owner_reconcile_for_test_v1(
        role, "mainnet", &appdir,
    )
    .expect("relaunch reconciliation after parent loss must succeed");
    assert!(
        reconciliation.as_deref().is_none_or(|evidence| {
            evidence.contains("removed-terminal-lease") || evidence.contains("orphan-terminated")
        }),
        "relaunch must durably reconcile the exact terminal orphan lease: {reconciliation:?}"
    );
    assert!(
        !lease_path.exists(),
        "relaunch reconciliation must clear the lease"
    );
    let _ = std::fs::remove_file(ready_path);
}

#[test]
fn ready_node_external_bridge_and_inprocess_bridge_exit_after_exact_parent_loss() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    for (role, node_kind, bridge_kind) in [
        ("node", "integrated-as-daemon", "disable"),
        ("bridge", "remote", "official-external-node"),
        ("bridge", "integrated-inproc", "official-inprocess-node"),
    ] {
        prove_parent_loss_cleanup(role, node_kind, bridge_kind);
    }
}

#[test]
fn post_ready_worker_failure_is_non_running_durable_and_restartable_for_all_roles() {
    let _guard = runtime_test_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _runtime_guard = RuntimeWorkerTestGuard::new();
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_COMMAND", "1");
    set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_EXIT_AFTER_READY_MS", "40");

    for (role, node_kind, bridge_kind) in [
        ("node", "integrated-as-daemon", "disable"),
        ("bridge", "remote", "official-external-node"),
        ("bridge", "integrated-inproc", "official-inprocess-node"),
    ] {
        let started = kgw_kgw_apply_node_settings_v1(
            "mainnet".to_string(),
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
        .unwrap_or_else(|error| panic!("{role}/{bridge_kind} fixture must reach READY: {error}"));
        assert_contains_all(&started, &["runtime_state=running", "readiness=READY"]);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let status = loop {
            let status = integrated_runtime_commands::kgw_runtime_owner_status_v1(
                Some("mainnet".to_string()),
                Some(role.to_string()),
            )
            .expect("post-READY terminal status must be readable");
            if status.contains("running=false") {
                break status;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "{role}/{bridge_kind} remained falsely Running: {status}"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert_contains_all(
            &status,
            &[
                "running=false",
                "readiness=FAILED",
                "runtime_error=runtime terminated unexpectedly after READY",
                "exit_status:exit status: 17",
            ],
        );

        let logs =
            kgw_kgw_runtime_logs_v1(Some("mainnet".to_string()), Some(role.to_string()), None)
                .expect("post-READY failure logs must be readable");
        assert!(logs.diagnostics.iter().any(|record| {
            record.diagnostic_event == "kgw_runtime_terminal_failure_v1"
                && record
                    .message
                    .contains("terminated unexpectedly after READY")
        }));
        assert!(logs.entries.iter().all(|entry| {
            !entry.raw_text.contains("kgw_runtime_terminal_failure_v1")
                && !entry.raw_text.contains("runtime_error=")
        }));

        remove_runtime_worker_test_env("KGW_TEST_SELF_WORKER_EXIT_AFTER_READY_MS");
        let restarted = kgw_kgw_apply_node_settings_v1(
            "mainnet".to_string(),
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
        .unwrap_or_else(|error| {
            panic!("{role}/{bridge_kind} must restart after terminal failure: {error}")
        });
        assert_contains_all(&restarted, &["runtime_state=running", "readiness=READY"]);
        kgw_kgw_disable_network_v1("mainnet".to_string(), Some(role.to_string()))
            .expect("restarted fixture must Stop cleanly");
        set_runtime_worker_test_env("KGW_TEST_SELF_WORKER_EXIT_AFTER_READY_MS", "40");
    }
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
        "effectiveNodeSettings",
        "experimentalNetworkOptIn",
    ] {
        assert!(
            node_js.contains(field),
            "frontend start payload must contain `{field}`"
        );
    }
}

#[test]
fn typed_effective_node_settings_are_validated_and_keep_backend_owned_paths() {
    let effective = kaspa_gateway_rk_node::EffectiveNodeSettings {
        log_level: "trace".to_string(),
        async_threads: 4,
        ram_scale: 0.5,
        rpc_listen: "127.0.0.1:26110".to_string(),
        rpc_listen_borsh: Some("127.0.0.1:27110".to_string()),
        rpc_listen_json: Some("127.0.0.1:28110".to_string()),
        rpc_max_clients: 16,
        p2p_listen: Some("127.0.0.1:26111".to_string()),
        outbound_target: 8,
        inbound_limit: 32,
        ..kaspa_gateway_rk_node::EffectiveNodeSettings::default()
    };
    let settings = integrated_runtime_commands::kgw_apply_effective_node_settings_for_test_v1(
        "mainnet",
        effective.clone(),
    )
    .expect("typed payload must apply");
    assert_eq!(settings.effective_node, effective);
    assert_eq!(settings.rpc_endpoint, "127.0.0.1:26110");
    assert_eq!(settings.p2p_listen.as_deref(), Some("127.0.0.1:26111"));
    assert!(settings.app_dir_name.ends_with("mainnet"));

    let mut invalid = effective;
    invalid.rpc_max_clients = 17;
    let error = integrated_runtime_commands::kgw_apply_effective_node_settings_for_test_v1(
        "mainnet", invalid,
    )
    .expect_err("embedded owner limit must be enforced at the runtime boundary");
    assert!(error.contains("rpcMaxClients must be between 1 and 16"));

    let public_wrpc = kaspa_gateway_rk_node::EffectiveNodeSettings {
        rpc_listen_borsh: Some("0.0.0.0:17110".to_string()),
        ..Default::default()
    };
    let error = integrated_runtime_commands::kgw_apply_effective_node_settings_for_test_v1(
        "mainnet",
        public_wrpc,
    )
    .expect_err("every RPC transport must remain loopback without explicit unsafe RPC");
    assert!(error.contains("rpcListenBorsh must remain loopback"));
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
            "--effective-node-settings-path",
            &std::env::temp_dir()
                .join("KaspaGateway")
                .join("effective-node-settings")
                .join("kgw-effective-node-settings-args-test.json")
                .to_string_lossy(),
            "--desktop-parent-pid",
            &std::process::id().to_string(),
            "--desktop-parent-start-time",
            &integrated_runtime_commands::kgw_process_identity_for_worker_v1(std::process::id())
                .expect("current parent identity")
                .start_time
                .to_string(),
            "--desktop-parent-executable",
            &integrated_runtime_commands::kgw_process_identity_for_worker_v1(std::process::id())
                .expect("current parent identity")
                .executable,
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
        "--effective-node-settings-path",
        "--desktop-parent-pid",
        "--desktop-parent-start-time",
        "--desktop-parent-executable",
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

    assert_contains_all(&duplicate, &["start_blocked=true", "network=mainnet"]);
    assert!(
        duplicate.contains("block_reason=duplicate-owner")
            || duplicate.contains("block_reason=runtime-owner-lease-active"),
        "duplicate Start must be refused by in-memory or durable exact ownership: {duplicate}"
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
