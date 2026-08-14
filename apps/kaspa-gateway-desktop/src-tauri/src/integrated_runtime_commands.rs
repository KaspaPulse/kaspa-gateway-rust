use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, ProcessesToUpdate, System};

static KGW_CONTROLLER: OnceLock<Arc<kaspa_gateway_rk_node::KgwServiceController>> = OnceLock::new();
static KGW_RAW_PROCESS_LOG_SEQUENCE_V1: AtomicU64 = AtomicU64::new(1);
const KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1: usize = 4096;
pub(crate) const KGW_NODE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS_V1: u64 = 90_000;
pub(crate) const KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1: u64 = 100_000;
pub(crate) const KGW_BRIDGE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1: u64 =
    kaspa_gateway_rk_bridge::KGW_BRIDGE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS + 9_000;
pub(crate) const KGW_CHILD_OFFICIAL_SHUTDOWN_BUDGET_MS_V1: u64 = 45_000;
pub(crate) const KGW_PARENT_GRACEFUL_STOP_TIMEOUT_MS_V1: u64 = 55_000;
const KGW_RUNTIME_OWNER_LEASE_VERSION_V1: u8 = 1;
const KGW_RUNTIME_OWNER_RECONCILIATION_TIMEOUT_MS_V1: u64 = 50_000;
const KGW_RUNTIME_OWNER_STARTING_GRACE_MS_V1: u64 = 10_000;
const _: () = assert!(
    KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1
        > KGW_NODE_CHILD_STARTUP_CONTRACT_TIMEOUT_MS_V1
);
const _: () =
    assert!(KGW_PARENT_GRACEFUL_STOP_TIMEOUT_MS_V1 > KGW_CHILD_OFFICIAL_SHUTDOWN_BUDGET_MS_V1);

fn controller() -> &'static Arc<kaspa_gateway_rk_node::KgwServiceController> {
    KGW_CONTROLLER.get_or_init(kaspa_gateway_rk_node::KgwServiceController::spawn)
}

#[derive(Debug)]
struct KgwParallelSelfWorker {
    role: String,
    network: String,
    appdir: String,

    node_mode: String,
    child: Child,
    spawned_pid: u32,
    stop_request_path: std::path::PathBuf,
    stop_outcome_path: std::path::PathBuf,
    reader_handles: Vec<std::thread::JoinHandle<()>>,
    // Keeps the registry-owned buffer alive with the runtime owner; insertion
    // remains exclusive to `kgw_worker_spawn_reader`.
    _raw_logs: KgwRawProcessLogBufferV1,
    started_ms: u128,
    exit_logged: bool,
    readiness_evidence: String,
    terminal_error: Option<String>,
    lease_path: std::path::PathBuf,
    lease_worker_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KgwRuntimeOwnerLeaseV1 {
    version: u8,
    runtime_role: String,
    network: String,
    appdir: String,
    node_mode: String,
    parent_pid: u32,
    parent_start_time: u64,
    parent_executable: String,
    published_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KgwRuntimeOwnerWorkerV1 {
    version: u8,
    runtime_role: String,
    network: String,
    parent_pid: u32,
    parent_start_time: u64,
    parent_executable: String,
    worker_pid: u32,
    worker_start_time: u64,
    worker_executable: String,
    published_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KgwProcessIdentityV1 {
    pub(crate) pid: u32,
    pub(crate) start_time: u64,
    pub(crate) executable: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KgwParentLifetimeEventV1 {
    StopRequested,
    ParentLost,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KgwStartupControlMessageV1 {
    pub(crate) version: u8,
    pub(crate) outcome: String,
    pub(crate) runtime_role: String,
    pub(crate) network: String,
    pub(crate) evidence: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KgwStopRequestMessageV1<'a> {
    version: u8,
    command: &'a str,
    runtime_role: &'a str,
    network: &'a str,
    worker_pid: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KgwStopOutcomeMessageV1 {
    pub(crate) version: u8,
    pub(crate) outcome: String,
    pub(crate) runtime_role: String,
    pub(crate) network: String,
    pub(crate) worker_pid: u32,
    pub(crate) all_owned_components_terminal: bool,
    pub(crate) evidence: Option<String>,
    pub(crate) error: Option<String>,
}

#[derive(Debug)]
pub(crate) enum KgwValidatedStopOutcomeV1 {
    Stopped(String),
    Failed {
        error: String,
        all_owned_components_terminal: bool,
    },
}

#[derive(Debug)]
struct KgwWorkerStopResultV1 {
    line: String,
    graceful: bool,
    forced: bool,
    stop_failed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwRuntimeRawLogEntryV1 {
    pub sequence: u64,
    pub network: String,
    pub source: String,
    pub runtime_role: String,
    pub bridge_instance_id: Option<String>,
    pub stream: String,
    pub received_ms: u64,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwRuntimeDiagnosticRecordV1 {
    pub diagnostic_event: String,
    pub network: String,
    pub source: String,
    pub runtime_role: String,
    pub bridge_instance_id: Option<String>,
    pub received_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwRuntimeLogsReportV1 {
    pub version: String,
    pub network: Option<String>,
    pub runtime_role: Option<String>,
    pub bridge_instance_id: Option<String>,
    pub source: String,
    pub buffer_limit: usize,
    pub entries: Vec<KgwRuntimeRawLogEntryV1>,
    pub diagnostics: Vec<KgwRuntimeDiagnosticRecordV1>,
}

static KGW_PARALLEL_SELF_WORKERS: OnceLock<Mutex<HashMap<String, KgwParallelSelfWorker>>> =
    OnceLock::new();
type KgwRawProcessLogBufferV1 = Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>;
static KGW_RAW_PROCESS_LOG_BUFFERS_V1: OnceLock<Mutex<HashMap<String, KgwRawProcessLogBufferV1>>> =
    OnceLock::new();
static KGW_RUNTIME_DIAGNOSTICS_V1: OnceLock<Mutex<HashMap<String, KgwRuntimeDiagnosticRecordV1>>> =
    OnceLock::new();

fn kgw_parallel_self_workers() -> &'static Mutex<HashMap<String, KgwParallelSelfWorker>> {
    KGW_PARALLEL_SELF_WORKERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn kgw_raw_process_log_buffers_v1() -> &'static Mutex<HashMap<String, KgwRawProcessLogBufferV1>> {
    KGW_RAW_PROCESS_LOG_BUFFERS_V1.get_or_init(|| Mutex::new(HashMap::new()))
}

fn kgw_runtime_diagnostics_v1() -> &'static Mutex<HashMap<String, KgwRuntimeDiagnosticRecordV1>> {
    KGW_RUNTIME_DIAGNOSTICS_V1.get_or_init(|| Mutex::new(HashMap::new()))
}

fn kgw_runtime_terminal_diagnostic_v1(
    role: &str,
    network: &str,
    message: impl Into<String>,
) -> KgwRuntimeDiagnosticRecordV1 {
    KgwRuntimeDiagnosticRecordV1 {
        diagnostic_event: "kgw_runtime_terminal_failure_v1".to_string(),
        network: network.to_string(),
        source: "self-worker-supervision".to_string(),
        runtime_role: role.to_string(),
        bridge_instance_id: None,
        received_ms: kgw_worker_now_ms_u64(),
        message: message.into(),
    }
}

fn kgw_runtime_record_terminal_diagnostic_v1(
    role: &str,
    network: &str,
    message: impl Into<String>,
) {
    if let Ok(mut diagnostics) = kgw_runtime_diagnostics_v1().lock() {
        diagnostics.insert(
            kgw_worker_key(role, network),
            kgw_runtime_terminal_diagnostic_v1(role, network, message),
        );
    }
}

fn kgw_runtime_clear_terminal_diagnostic_v1(role: &str, network: &str) {
    if let Ok(mut diagnostics) = kgw_runtime_diagnostics_v1().lock() {
        diagnostics.remove(&kgw_worker_key(role, network));
    }
}

#[cfg(test)]
fn kgw_worker_uses_test_command() -> bool {
    std::env::var_os("KGW_TEST_SELF_WORKER_COMMAND").is_some()
        || std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_COMMAND").is_some()
        || std::env::var_os("KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND").is_some()
        || std::env::var_os("KGW_TEST_SELF_WORKER_MISSING_COMMAND").is_some()
}

#[cfg(not(test))]
fn kgw_worker_uses_test_command() -> bool {
    false
}

fn kgw_worker_now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn kgw_worker_now_ms_u64() -> u64 {
    u64::try_from(kgw_worker_now_ms()).unwrap_or(u64::MAX)
}

fn kgw_runtime_owner_lease_directory_v1() -> std::path::PathBuf {
    if let Ok(root) = kaspa_gateway_config::default_user_data_dir() {
        root.join("runtime-ownership")
    } else {
        kgw_safe_runtime_appdir_root().join(".runtime-ownership")
    }
}

fn kgw_runtime_owner_lease_path_v1(role: &str, network: &str) -> std::path::PathBuf {
    kgw_runtime_owner_lease_directory_v1().join(format!("{role}-{network}.json"))
}

fn kgw_runtime_owner_worker_path_v1(
    lease_path: &std::path::Path,
    parent: &KgwProcessIdentityV1,
) -> std::path::PathBuf {
    let stem = lease_path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("runtime-owner");
    lease_path.with_file_name(format!(
        "{stem}.{}-{}.worker.json",
        parent.pid, parent.start_time
    ))
}

pub(crate) fn kgw_process_identity_for_worker_v1(pid: u32) -> Result<KgwProcessIdentityV1, String> {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]));
    let process = system
        .process(pid)
        .ok_or_else(|| format!("process identity unavailable;pid={}", pid.as_u32()))?;
    let executable = process
        .exe()
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| format!("process executable unavailable;pid={}", pid.as_u32()))?;
    let canonical = executable.canonicalize().map_err(|error| {
        format!(
            "canonicalize process executable failed;pid={};path={};error={error}",
            pid.as_u32(),
            executable.display()
        )
    })?;

    Ok(KgwProcessIdentityV1 {
        pid: pid.as_u32(),
        start_time: process.start_time(),
        executable: canonical.to_string_lossy().to_string(),
    })
}

fn kgw_runtime_owner_read_lease_v1(
    path: &std::path::Path,
) -> Result<KgwRuntimeOwnerLeaseV1, String> {
    let payload = std::fs::read(path).map_err(|error| {
        format!(
            "read runtime owner lease failed {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&payload).map_err(|error| {
        format!(
            "parse runtime owner lease failed {}: {error}",
            path.display()
        )
    })
}

fn kgw_runtime_owner_read_worker_v1(
    path: &std::path::Path,
) -> Result<KgwRuntimeOwnerWorkerV1, String> {
    let payload = std::fs::read(path).map_err(|error| {
        format!(
            "read runtime owner worker identity failed {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&payload).map_err(|error| {
        format!(
            "parse runtime owner worker identity failed {}: {error}",
            path.display()
        )
    })
}

fn kgw_runtime_owner_remove_lease_v1(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

fn kgw_runtime_owner_remove_files_v1(lease_path: &std::path::Path, worker_path: &std::path::Path) {
    let _ = std::fs::remove_file(worker_path);
    kgw_runtime_owner_remove_lease_v1(lease_path);
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
fn kgw_runtime_owner_lease_matches_identity_v1(
    lease: &KgwRuntimeOwnerLeaseV1,
    worker: &KgwRuntimeOwnerWorkerV1,
    role: &str,
    network: &str,
    appdir: &str,
    current_executable: &str,
    worker_identity: &KgwProcessIdentityV1,
) -> bool {
    lease.version == KGW_RUNTIME_OWNER_LEASE_VERSION_V1
        && lease.runtime_role == role
        && lease.network == network
        && lease.appdir == appdir
        && worker.version == KGW_RUNTIME_OWNER_LEASE_VERSION_V1
        && worker.runtime_role == role
        && worker.network == network
        && worker.parent_pid == lease.parent_pid
        && worker.parent_start_time == lease.parent_start_time
        && worker.parent_executable == lease.parent_executable
        && worker.worker_pid == worker_identity.pid
        && worker.worker_start_time == worker_identity.start_time
        && worker.worker_executable == worker_identity.executable
        && worker.worker_executable == current_executable
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
pub(crate) fn kgw_runtime_owner_lease_identity_matches_for_test_v1(
    lease: &KgwRuntimeOwnerLeaseV1,
    worker: &KgwRuntimeOwnerWorkerV1,
    role: &str,
    network: &str,
    appdir: &str,
    current_executable: &str,
    worker_identity: &KgwProcessIdentityV1,
) -> bool {
    kgw_runtime_owner_lease_matches_identity_v1(
        lease,
        worker,
        role,
        network,
        appdir,
        current_executable,
        worker_identity,
    )
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
pub(crate) fn kgw_runtime_owner_lease_fixture_v1(
    role: &str,
    network: &str,
    appdir: &str,
    node_mode: &str,
    parent_identity: &KgwProcessIdentityV1,
    worker_identity: &KgwProcessIdentityV1,
) -> (KgwRuntimeOwnerLeaseV1, KgwRuntimeOwnerWorkerV1) {
    let lease = KgwRuntimeOwnerLeaseV1 {
        version: KGW_RUNTIME_OWNER_LEASE_VERSION_V1,
        runtime_role: role.to_string(),
        network: network.to_string(),
        appdir: appdir.to_string(),
        node_mode: node_mode.to_string(),
        parent_pid: parent_identity.pid,
        parent_start_time: parent_identity.start_time,
        parent_executable: parent_identity.executable.clone(),
        published_ms: kgw_worker_now_ms(),
    };
    let worker = KgwRuntimeOwnerWorkerV1 {
        version: KGW_RUNTIME_OWNER_LEASE_VERSION_V1,
        runtime_role: role.to_string(),
        network: network.to_string(),
        parent_pid: parent_identity.pid,
        parent_start_time: parent_identity.start_time,
        parent_executable: parent_identity.executable.clone(),
        worker_pid: worker_identity.pid,
        worker_start_time: worker_identity.start_time,
        worker_executable: worker_identity.executable.clone(),
        published_ms: kgw_worker_now_ms(),
    };
    (lease, worker)
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
pub(crate) fn kgw_runtime_owner_reconcile_for_test_v1(
    role: &str,
    network: &str,
    appdir: &str,
) -> Result<Option<String>, String> {
    kgw_runtime_owner_reconcile_stale_lease_v1(role, network, appdir)
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
pub(crate) fn kgw_runtime_owner_reserve_for_test_v1(
    path: &std::path::Path,
    lease: &KgwRuntimeOwnerLeaseV1,
) -> Result<(), String> {
    kgw_runtime_owner_reserve_lease_v1(path, lease)
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
pub(crate) fn kgw_runtime_owner_publish_worker_for_test_v1(
    lease_path: &std::path::Path,
    parent: &KgwProcessIdentityV1,
    worker: &KgwProcessIdentityV1,
) -> Result<std::path::PathBuf, String> {
    kgw_runtime_owner_update_spawned_lease_v1(lease_path, parent, worker)
}

#[cfg(test)]
#[allow(dead_code)] // Exercised by the path-included integration-test target.
pub(crate) fn kgw_runtime_owner_worker_path_for_test_v1(
    lease_path: &std::path::Path,
    parent: &KgwProcessIdentityV1,
) -> std::path::PathBuf {
    kgw_runtime_owner_worker_path_v1(lease_path, parent)
}

fn kgw_runtime_owner_reconcile_stale_lease_v1(
    role: &str,
    network: &str,
    _appdir: &str,
) -> Result<Option<String>, String> {
    let path = kgw_runtime_owner_lease_path_v1(role, network);
    kgw_runtime_owner_prepare_directory_v1(&path)?;
    match std::fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            return Err(format!(
                "start_blocked=true;start_allowed=false;block_reason=runtime-owner-lease-untrusted;runtime_role={role};network={network};message=Ownership metadata is not a trusted regular file. No process was signaled."
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "inspect runtime owner lease failed {}: {error}",
                path.display()
            ));
        }
    }
    let lease = kgw_runtime_owner_read_lease_v1(&path)?;
    let worker_path = kgw_runtime_owner_worker_path_v1(
        &path,
        &KgwProcessIdentityV1 {
            pid: lease.parent_pid,
            start_time: lease.parent_start_time,
            executable: lease.parent_executable.clone(),
        },
    );
    let current_executable = std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|error| format!("resolve current executable for lease failed: {error}"))?
        .to_string_lossy()
        .to_string();
    if lease.version != KGW_RUNTIME_OWNER_LEASE_VERSION_V1
        || lease.runtime_role != role
        || lease.network != network
        || lease.parent_executable != current_executable
    {
        return Err(format!(
            "start_blocked=true;start_allowed=false;block_reason=runtime-owner-lease-identity-mismatch;runtime_role={role};network={network};message=Stale ownership metadata does not match the requested exact owner. No process was signaled."
        ));
    }

    let parent_alive = kgw_process_identity_for_worker_v1(lease.parent_pid).is_ok_and(|identity| {
        identity.start_time == lease.parent_start_time
            && identity.executable == lease.parent_executable
            && identity.executable == current_executable
    });
    if parent_alive {
        return Err(format!(
            "start_blocked=true;start_allowed=false;block_reason=runtime-owner-lease-active;runtime_role={role};network={network};parent_pid={};message=Another exact desktop owner is active.",
            lease.parent_pid
        ));
    }

    let deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(KGW_RUNTIME_OWNER_RECONCILIATION_TIMEOUT_MS_V1);
    loop {
        let observed = match kgw_runtime_owner_read_lease_v1(&path) {
            Ok(observed) => observed,
            Err(_error) if !path.exists() => {
                return Ok(Some(format!(
                    "reconciliation=orphan-terminated;role={role};network={network}"
                )));
            }
            Err(error) => return Err(error),
        };
        if observed.parent_pid != lease.parent_pid
            || observed.parent_start_time != lease.parent_start_time
            || observed.parent_executable != lease.parent_executable
        {
            return Err(format!(
                "start_blocked=true;start_allowed=false;block_reason=runtime-owner-lease-changed;runtime_role={role};network={network};message=Ownership metadata changed during reconciliation. No process was signaled."
            ));
        }

        let worker_metadata = match std::fs::symlink_metadata(&worker_path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(format!(
                    "start_blocked=true;start_allowed=false;block_reason=runtime-owner-worker-untrusted;runtime_role={role};network={network};message=Worker ownership metadata is not a trusted regular file. No process was signaled."
                ));
            }
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => {
                return Err(format!(
                    "inspect runtime owner worker identity failed {}: {error}",
                    worker_path.display()
                ));
            }
        };
        if worker_metadata.is_none() {
            let starting_deadline_ms = observed
                .published_ms
                .saturating_add(u128::from(KGW_RUNTIME_OWNER_STARTING_GRACE_MS_V1));
            if kgw_worker_now_ms() >= starting_deadline_ms {
                kgw_runtime_owner_remove_files_v1(&path, &worker_path);
                return Ok(Some(format!(
                    "reconciliation=removed-abandoned-start;role={role};network={network};parent_pid={}",
                    observed.parent_pid
                )));
            }
        } else {
            let worker = kgw_runtime_owner_read_worker_v1(&worker_path)?;
            if worker.version != KGW_RUNTIME_OWNER_LEASE_VERSION_V1
                || worker.runtime_role != role
                || worker.network != network
                || worker.parent_pid != observed.parent_pid
                || worker.parent_start_time != observed.parent_start_time
                || worker.parent_executable != observed.parent_executable
                || worker.worker_pid == 0
                || worker.worker_executable != current_executable
            {
                return Err(format!(
                    "start_blocked=true;start_allowed=false;block_reason=runtime-owner-worker-identity-mismatch;runtime_role={role};network={network};message=Worker ownership metadata does not match the immutable reservation. No process was signaled."
                ));
            }
            let exact_worker_alive = kgw_process_identity_for_worker_v1(worker.worker_pid)
                .is_ok_and(|identity| {
                    identity.start_time == worker.worker_start_time
                        && identity.executable == worker.worker_executable
                        && identity.executable == current_executable
                });
            if !exact_worker_alive {
                kgw_runtime_owner_remove_files_v1(&path, &worker_path);
                return Ok(Some(format!(
                    "reconciliation=removed-terminal-lease;role={role};network={network};worker_pid={}",
                    worker.worker_pid
                )));
            }
        }

        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "start_blocked=true;start_allowed=false;block_reason=orphan-terminality-unproven;runtime_role={role};network={network};message=Exact prior worker did not terminate within the reconciliation window. No unrelated process was targeted."
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn kgw_runtime_owner_prepare_directory_v1(lease_path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = lease_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create runtime owner lease directory failed {}: {error}",
                parent.display()
            )
        })?;
        let metadata = std::fs::symlink_metadata(parent).map_err(|error| {
            format!(
                "inspect runtime owner lease directory failed {}: {error}",
                parent.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "runtime owner lease directory is not a trusted directory: {}",
                parent.display()
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).map_err(
                |error| {
                    format!(
                        "restrict runtime owner lease directory failed {}: {error}",
                        parent.display()
                    )
                },
            )?;
        }
    }
    Ok(())
}

struct KgwRuntimeOwnerReservationGuardV1 {
    lease_path: std::path::PathBuf,
    worker_path: Option<std::path::PathBuf>,
    armed: bool,
}

impl KgwRuntimeOwnerReservationGuardV1 {
    fn new(path: std::path::PathBuf) -> Self {
        Self {
            lease_path: path,
            worker_path: None,
            armed: true,
        }
    }

    fn set_worker_path(&mut self, path: std::path::PathBuf) {
        self.worker_path = Some(path);
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for KgwRuntimeOwnerReservationGuardV1 {
    fn drop(&mut self) {
        if self.armed {
            if let Some(worker_path) = &self.worker_path {
                kgw_runtime_owner_remove_files_v1(&self.lease_path, worker_path);
            } else {
                kgw_runtime_owner_remove_lease_v1(&self.lease_path);
            }
        }
    }
}

fn kgw_runtime_owner_reserve_lease_v1(
    lease_path: &std::path::Path,
    lease: &KgwRuntimeOwnerLeaseV1,
) -> Result<(), String> {
    kgw_runtime_owner_prepare_directory_v1(lease_path)?;
    let payload = serde_json::to_vec(lease)
        .map_err(|error| format!("serialize runtime owner lease failed: {error}"))?;
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = match options.open(lease_path) {
        Ok(file) => file,
        Err(error) => {
            return Err(format!(
                "reserve runtime owner lease failed {}: {error}",
                lease_path.display()
            ));
        }
    };
    if let Err(error) = file.write_all(&payload).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(lease_path);
        return Err(format!(
            "write runtime owner lease reservation failed {}: {error}",
            lease_path.display()
        ));
    }
    Ok(())
}

fn kgw_runtime_owner_update_spawned_lease_v1(
    lease_path: &std::path::Path,
    expected_parent: &KgwProcessIdentityV1,
    worker: &KgwProcessIdentityV1,
) -> Result<std::path::PathBuf, String> {
    let file_metadata = std::fs::symlink_metadata(lease_path).map_err(|error| {
        format!(
            "inspect runtime owner reservation failed {}: {error}",
            lease_path.display()
        )
    })?;
    if file_metadata.file_type().is_symlink() || !file_metadata.is_file() {
        return Err("runtime owner reservation is not a trusted regular file".to_string());
    }
    let payload = std::fs::read(lease_path)
        .map_err(|error| format!("read spawned runtime owner lease failed: {error}"))?;
    let lease: KgwRuntimeOwnerLeaseV1 = serde_json::from_slice(&payload)
        .map_err(|error| format!("parse spawned runtime owner lease failed: {error}"))?;
    if lease.parent_pid != expected_parent.pid
        || lease.parent_start_time != expected_parent.start_time
        || lease.parent_executable != expected_parent.executable
    {
        return Err(
            "runtime owner reservation identity changed before spawn publication".to_string(),
        );
    }
    let published = KgwRuntimeOwnerWorkerV1 {
        version: KGW_RUNTIME_OWNER_LEASE_VERSION_V1,
        runtime_role: lease.runtime_role,
        network: lease.network,
        parent_pid: lease.parent_pid,
        parent_start_time: lease.parent_start_time,
        parent_executable: lease.parent_executable,
        worker_pid: worker.pid,
        worker_start_time: worker.start_time,
        worker_executable: worker.executable.clone(),
        published_ms: kgw_worker_now_ms(),
    };
    let payload = serde_json::to_vec(&published)
        .map_err(|error| format!("serialize spawned runtime owner identity failed: {error}"))?;
    let worker_path = kgw_runtime_owner_worker_path_v1(lease_path, expected_parent);
    static WORKER_IDENTITY_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let temporary = worker_path.with_extension(format!(
        "{}-{}-{}.writing",
        std::process::id(),
        kgw_worker_now_ms(),
        WORKER_IDENTITY_SEQUENCE.fetch_add(1, Ordering::SeqCst)
    ));
    {
        let mut options = std::fs::OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary).map_err(|error| {
            format!(
                "create runtime owner worker identity failed {}: {error}",
                temporary.display()
            )
        })?;
        file.write_all(&payload).map_err(|error| {
            format!(
                "write runtime owner worker identity failed {}: {error}",
                temporary.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "sync runtime owner worker identity failed {}: {error}",
                temporary.display()
            )
        })?;
    }
    #[cfg(unix)]
    let publish_result = std::fs::hard_link(&temporary, &worker_path);
    #[cfg(windows)]
    let publish_result = std::fs::rename(&temporary, &worker_path);
    #[cfg(not(any(unix, windows)))]
    let publish_result = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "atomic worker identity publication is unsupported on this platform",
    ));
    if let Err(error) = publish_result {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!(
            "publish runtime owner worker identity failed {}: {error}",
            worker_path.display()
        ));
    }
    let _ = std::fs::remove_file(&temporary);
    Ok(worker_path)
}

fn kgw_worker_next_raw_log_sequence_v1() -> u64 {
    KGW_RAW_PROCESS_LOG_SEQUENCE_V1.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
thread_local! {
    static KGW_START_TRACE_TEST_ENABLED_V1: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) struct KgwStartTraceTestGuardV1 {
    previous: bool,
}

#[cfg(test)]
impl Drop for KgwStartTraceTestGuardV1 {
    fn drop(&mut self) {
        KGW_START_TRACE_TEST_ENABLED_V1.with(|enabled| enabled.set(self.previous));
    }
}

#[cfg(test)]
pub(crate) fn kgw_start_trace_test_enable_v1() -> KgwStartTraceTestGuardV1 {
    let previous = KGW_START_TRACE_TEST_ENABLED_V1.with(|enabled| {
        let previous = enabled.get();
        enabled.set(true);
        previous
    });

    KgwStartTraceTestGuardV1 { previous }
}

pub(crate) fn kgw_start_trace_enabled_v1() -> bool {
    #[cfg(test)]
    if KGW_START_TRACE_TEST_ENABLED_V1.with(|enabled| enabled.get()) {
        return true;
    }

    match std::env::var("KGW_START_TRACE") {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn kgw_start_trace_json_escape_v1(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 16);

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }

    out
}

fn kgw_start_trace_clean_field_v1(value: &str, fallback: &str) -> String {
    let clean = value
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        return fallback.to_string();
    }

    clean.chars().take(160).collect()
}

fn kgw_start_trace_safe_details_v1(value: &str) -> String {
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("private")
        || lowered.contains("mnemonic")
        || lowered.contains("wallet")
        || lowered.contains("address")
    {
        return "{\"redacted\":true,\"reason\":\"sensitive-field\"}".to_string();
    }

    let clean = value
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();

    clean.chars().take(1200).collect()
}

pub(crate) fn kgw_start_trace_format_v1(
    source: &str,
    stage: &str,
    network: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
) -> String {
    let source = kgw_start_trace_clean_field_v1(source, "native");
    let stage = kgw_start_trace_clean_field_v1(stage, "unknown");
    let network = kgw_start_trace_clean_field_v1(network, "unknown");
    let action = kgw_start_trace_clean_field_v1(action, "unknown");
    let result = kgw_start_trace_clean_field_v1(result, "unknown");
    let details = details.map(kgw_start_trace_safe_details_v1);

    let mut line = format!(
        "[KGW_START_TRACE] {{\"timestamp\":{},\"source\":\"{}\",\"stage\":\"{}\",\"network\":\"{}\",\"action\":\"{}\",\"result\":\"{}\"",
        kgw_worker_now_ms(),
        kgw_start_trace_json_escape_v1(&source),
        kgw_start_trace_json_escape_v1(&stage),
        kgw_start_trace_json_escape_v1(&network),
        kgw_start_trace_json_escape_v1(&action),
        kgw_start_trace_json_escape_v1(&result)
    );

    if let Some(details) = details {
        line.push_str(",\"details\":\"");
        line.push_str(&kgw_start_trace_json_escape_v1(&details));
        line.push('"');
    }

    line.push('}');
    line
}

#[cfg(test)]
static KGW_START_TRACE_TEST_LINES_V1: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn kgw_start_trace_test_take_lines_v1() -> Vec<String> {
    let sink = KGW_START_TRACE_TEST_LINES_V1.get_or_init(|| Mutex::new(Vec::new()));
    match sink.lock() {
        Ok(mut guard) => std::mem::take(&mut *guard),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn kgw_start_trace_emit_v1(
    source: &str,
    stage: &str,
    network: &str,
    action: &str,
    result: &str,
    details: Option<&str>,
) {
    if !kgw_start_trace_enabled_v1() {
        return;
    }

    let line = kgw_start_trace_format_v1(source, stage, network, action, result, details);
    eprintln!("{line}");

    #[cfg(test)]
    {
        let sink = KGW_START_TRACE_TEST_LINES_V1.get_or_init(|| Mutex::new(Vec::new()));
        if let Ok(mut guard) = sink.lock() {
            guard.push(line);
        }
    }
}

fn kgw_worker_key(role: &str, network: &str) -> String {
    format!(
        "{}:{}",
        role.trim().to_ascii_lowercase(),
        network.trim().to_ascii_lowercase()
    )
}

fn kgw_worker_replace_raw_log_buffer_v1(
    role: &str,
    network: &str,
) -> Result<KgwRawProcessLogBufferV1, String> {
    let buffer = Arc::new(Mutex::new(VecDeque::new()));
    let mut buffers = kgw_raw_process_log_buffers_v1()
        .lock()
        .map_err(|_| "raw process log buffer registry lock failed".to_string())?;
    buffers.insert(kgw_worker_key(role, network), Arc::clone(&buffer));
    Ok(buffer)
}

fn kgw_worker_clean_optional_identifier_v1(value: Option<String>) -> Option<String> {
    let clean = value?
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '\0' && *ch != '\r' && *ch != '\n')
        .collect::<String>()
        .trim()
        .to_string();

    if clean.is_empty() {
        None
    } else {
        Some(clean.chars().take(128).collect())
    }
}

fn kgw_worker_push_raw_log(
    logs: &Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>,
    entry: KgwRuntimeRawLogEntryV1,
) {
    if let Ok(mut guard) = logs.lock() {
        if guard.len() >= KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1 {
            guard.pop_front();
        }

        guard.push_back(entry);
    }
}

fn kgw_worker_raw_log_entry_v1(
    role: &str,
    network: &str,
    stream: &str,
    line: String,
) -> KgwRuntimeRawLogEntryV1 {
    KgwRuntimeRawLogEntryV1 {
        sequence: kgw_worker_next_raw_log_sequence_v1(),
        network: network.to_string(),
        source: "self-worker".to_string(),
        runtime_role: role.to_string(),
        // The official Bridge library exposes process-wide output and no
        // structural listener attribution. Keep this field unset rather than
        // inventing per-instance provenance from the active UI selection.
        bridge_instance_id: None,
        stream: stream.to_string(),
        received_ms: kgw_worker_now_ms_u64(),
        raw_text: line,
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests; unused in the library test target.
pub(crate) fn kgw_raw_log_entry_for_test_v1(
    sequence: u64,
    network: &str,
    runtime_role: &str,
    bridge_instance_id: Option<&str>,
    stream: &str,
    raw_text: &str,
) -> KgwRuntimeRawLogEntryV1 {
    KgwRuntimeRawLogEntryV1 {
        sequence,
        network: network.to_string(),
        source: "self-worker".to_string(),
        runtime_role: runtime_role.to_string(),
        bridge_instance_id: bridge_instance_id.map(str::to_string),
        stream: stream.to_string(),
        received_ms: 1,
        raw_text: raw_text.to_string(),
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests; unused in the library test target.
pub(crate) fn kgw_raw_log_text_from_entries_for_test_v1(
    mut entries: Vec<KgwRuntimeRawLogEntryV1>,
) -> String {
    entries.sort_by_key(|entry| entry.sequence);
    entries
        .into_iter()
        .map(|entry| entry.raw_text)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn kgw_worker_safe_child_line_v1(line: &str) -> String {
    let clean = line
        .chars()
        .map(|ch| {
            if ch == '\r' || ch == '\n' || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();

    let mut redacted = Vec::new();
    let mut redact_next = false;

    for token in clean.split_whitespace() {
        let lower = token.to_ascii_lowercase();

        if redact_next {
            redacted.push("[redacted-sensitive-value]".to_string());
            redact_next = false;
            continue;
        }

        if matches!(
            lower.as_str(),
            "--internal-cpu-miner-address"
                | "--wallet"
                | "--wallet-address"
                | "--private-key"
                | "--token"
                | "--secret"
                | "--mnemonic"
        ) {
            redacted.push(token.to_string());
            redact_next = true;
            continue;
        }

        if lower.starts_with("kaspa:")
            || lower.starts_with("kaspatest:")
            || lower.starts_with("kaspadev:")
            || lower.starts_with("kaspareg:")
        {
            redacted.push("[redacted-wallet-address]".to_string());
            continue;
        }

        if let Some((key, _value)) = token.split_once('=') {
            let key_lower = key.to_ascii_lowercase();
            if key_lower.contains("token")
                || key_lower.contains("secret")
                || key_lower.contains("private")
                || key_lower.contains("mnemonic")
                || key_lower == "wallet"
                || key_lower == "wallet_address"
                || key_lower == "wallet-address"
                || key_lower == "internal_cpu_miner_address"
                || key_lower == "internal-cpu-miner-address"
            {
                redacted.push(format!("{key}=[redacted-sensitive-value]"));
                continue;
            }
        }

        redacted.push(token.to_string());
    }

    if redacted.is_empty() {
        String::new()
    } else {
        redacted.join(" ")
    }
}

pub(crate) fn kgw_worker_format_child_mirror_line_v1(
    role: &str,
    network: &str,
    bridge_instance_id: Option<&str>,
    stream: &str,
    child_pid: u32,
    line: &str,
) -> String {
    let prefix = if stream == "stdout" {
        "[KGW_CHILD_STDOUT]"
    } else {
        "[KGW_CHILD_STDERR]"
    };
    let safe_line = kgw_worker_safe_child_line_v1(line);
    let bridge_instance_json = bridge_instance_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("\"{}\"", kgw_start_trace_json_escape_v1(value.trim())))
        .unwrap_or_else(|| "null".to_string());

    format!(
        "{} {{\"timestamp\":{},\"eventKind\":\"diagnostic_transport_record\",\"network\":\"{}\",\"runtimeRole\":\"{}\",\"bridgeInstanceId\":{},\"pid\":{},\"stage\":\"diagnostic_transport.child.{}\",\"stream\":\"{}\",\"action\":\"start\",\"result\":\"line\",\"safeText\":\"{}\"}}",
        prefix,
        kgw_worker_now_ms(),
        kgw_start_trace_json_escape_v1(network),
        kgw_start_trace_json_escape_v1(role),
        bridge_instance_json,
        child_pid,
        kgw_start_trace_json_escape_v1(stream),
        kgw_start_trace_json_escape_v1(stream),
        kgw_start_trace_json_escape_v1(&safe_line)
    )
}

fn kgw_start_trace_capture_test_line_v1(_line: &str) {
    #[cfg(test)]
    {
        let sink = KGW_START_TRACE_TEST_LINES_V1.get_or_init(|| Mutex::new(Vec::new()));
        if let Ok(mut guard) = sink.lock() {
            guard.push(_line.to_string());
        }
    }
}

fn kgw_worker_argument_names_for_trace_v1(command: &Command) -> Vec<String> {
    let mut names = Vec::new();
    let mut previous_flag: Option<String> = None;

    for arg in command.get_args() {
        let text = arg.to_string_lossy().to_string();

        if text.starts_with("--") {
            previous_flag = Some(text.clone());
            names.push(text);
            continue;
        }

        let placeholder = match previous_flag.as_deref() {
            Some("--kgw-self-worker") => "<runtime-role>",
            Some("--network") => "<network>",
            Some("--appdir") => "<database-directory>",
            Some("--rpc") => "<rpc-endpoint>",
            Some("--listen") => "<p2p-listen>",
            Some("--stratum") => "<stratum-endpoint>",
            Some("--node-mode") => "<node-mode>",
            Some("--bridge-config") => "<bridge-config-path>",
            Some("--bridge-instance-listen") => "<bridge-instance-listen>",
            Some("--internal-cpu-miner-address") => "<redacted-wallet-address>",
            Some("--internal-cpu-miner-threads") => "<thread-count>",
            Some("--internal-cpu-miner-throttle-ms") => "<throttle-ms>",
            Some("--internal-cpu-miner-template-poll-ms") => "<template-poll-ms>",
            Some("--desktop-parent-pid") => "<parent-pid>",
            Some("--desktop-parent-start-time") => "<parent-start-time>",
            Some("--desktop-parent-executable") => "<parent-executable>",
            Some("--exact") => "<test-name>",
            Some(flag) if flag.starts_with("--") => "<argument>",
            _ => "<argument>",
        };

        names.push(placeholder.to_string());
        previous_flag = None;
    }

    names
}

pub(crate) fn kgw_validate_live_smoke_parent_settings_v1(
    network: &str,
    appdir: &str,
    rpc: &str,
    p2p_listen: Option<&str>,
) -> Result<kaspa_gateway_rk_node::NodeSettings, String> {
    let mut settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        network.to_string(),
        "integrated-as-daemon".to_string(),
        "disable".to_string(),
    )
    .map_err(|error| error.to_string())?;
    if settings.network == kaspa_gateway_rk_node::KgwNetwork::Testnet12 {
        return Err("live smoke parent does not start experimental testnet12".to_string());
    }
    settings.app_dir_name = appdir.to_string();
    settings.rpc_endpoint = rpc.to_string();
    settings.p2p_listen = p2p_listen.map(str::to_string);
    let appdir_path = std::path::Path::new(appdir);
    if !appdir_path.is_absolute()
        || appdir.trim().is_empty()
        || appdir.contains('\0')
        || appdir.contains('\n')
        || appdir.contains('\r')
    {
        return Err("live smoke app directory must be a safe absolute path".to_string());
    }
    let validate_loopback_endpoint = |label: &str, value: &str| -> Result<(), String> {
        let (host, port) = value
            .rsplit_once(':')
            .ok_or_else(|| format!("{label} must contain host and port"))?;
        if !matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
            return Err(format!("{label} must bind to loopback"));
        }
        let port = port
            .parse::<u16>()
            .map_err(|error| format!("{label} port is invalid: {error}"))?;
        if port == 0 {
            return Err(format!("{label} port cannot be zero"));
        }
        Ok(())
    };
    validate_loopback_endpoint("live smoke RPC", rpc)?;
    if let Some(listen) = p2p_listen {
        validate_loopback_endpoint("live smoke P2P", listen)?;
    }
    Ok(settings)
}

fn kgw_worker_endpoint_port_v1(endpoint: &str) -> Option<u16> {
    endpoint
        .rsplit(':')
        .next()
        .and_then(|value| value.trim().parse::<u16>().ok())
}

fn kgw_worker_node_mode_for_trace_v1(role: &str, bridge_node_mode: &str) -> String {
    if role == "node" {
        "same-exe-self-worker".to_string()
    } else {
        bridge_node_mode.to_string()
    }
}

fn kgw_worker_spawn_plan_details_v1(
    role: &str,
    network: &str,
    node_mode: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
    command: &Command,
    uses_test_command: bool,
    has_bridge_config_path: bool,
) -> String {
    let executable_path = command.get_program().to_string_lossy().to_string();
    let working_directory = command
        .get_current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|path| path.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    serde_json::json!({
        "runtimeRole": role,
        "network": network,
        "nodeMode": node_mode,
        "sameExecutable": true,
        "externalKaspadExe": false,
        "usesTestCommand": uses_test_command,
        "hasBridgeConfigPath": has_bridge_config_path,
        "executablePath": executable_path,
        "workingDirectory": working_directory,
        "argumentNames": kgw_worker_argument_names_for_trace_v1(command),
        "databaseDirectory": settings.app_dir_name,
        "rpcPort": kgw_worker_endpoint_port_v1(&settings.rpc_endpoint),
        "stratumPort": kgw_worker_endpoint_port_v1(&settings.stratum_listen),
    })
    .to_string()
}

fn kgw_worker_apply_current_dir_v1(command: &mut Command) {
    if let Ok(current_dir) = std::env::current_dir() {
        command.current_dir(current_dir);
    }
}

fn kgw_worker_startup_control_path_v1(role: &str, network: &str) -> std::path::PathBuf {
    static STARTUP_CONTROL_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let nonce = kgw_worker_now_ms();
    let sequence = STARTUP_CONTROL_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir()
        .join("KaspaGateway")
        .join("startup-control")
        .join(format!(
            "{}-{}-{}-{nonce}-{sequence}.json",
            role,
            network,
            std::process::id()
        ))
}

fn kgw_worker_stop_control_paths_v1(
    role: &str,
    network: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    static STOP_CONTROL_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let nonce = kgw_worker_now_ms();
    let sequence = STOP_CONTROL_SEQUENCE.fetch_add(1, Ordering::SeqCst);
    let stem = format!(
        "{}-{}-{}-{nonce}-{sequence}",
        role,
        network,
        std::process::id()
    );
    let parent = std::env::temp_dir()
        .join("KaspaGateway")
        .join("stop-control");
    (
        parent.join(format!("{stem}-request.json")),
        parent.join(format!("{stem}-outcome.json")),
    )
}

fn kgw_worker_atomic_write_json_v1<T: Serialize>(
    path: &std::path::Path,
    value: &T,
) -> Result<(), String> {
    let payload = serde_json::to_vec(value)
        .map_err(|error| format!("serialize control message failed: {error}"))?;
    static CONTROL_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let temporary = path.with_extension(format!(
        "{}-{}-{}.writing",
        std::process::id(),
        kgw_worker_now_ms(),
        CONTROL_WRITE_SEQUENCE.fetch_add(1, Ordering::SeqCst)
    ));

    {
        let mut options = std::fs::OpenOptions::new();
        options.create_new(true).write(true).append(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary).map_err(|error| {
            format!(
                "create control message failed {}: {error}",
                temporary.display()
            )
        })?;
        file.write_all(&payload).map_err(|error| {
            format!(
                "write control message failed {}: {error}",
                temporary.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "sync control message failed {}: {error}",
                temporary.display()
            )
        })?;
    }

    std::fs::rename(&temporary, path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        format!("publish control message failed {}: {error}", path.display())
    })
}

fn kgw_worker_control_directory_v1(path: &std::path::Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("control path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "create runtime control directory failed {}: {error}",
            parent.display()
        )
    })?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|error| {
        format!(
            "inspect runtime control directory failed {}: {error}",
            parent.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "runtime control directory is not a trusted directory: {}",
            parent.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).map_err(
            |error| {
                format!(
                    "restrict runtime control directory failed {}: {error}",
                    parent.display()
                )
            },
        )?;
    }
    Ok(())
}

fn kgw_worker_remove_stop_control_v1(worker: &KgwParallelSelfWorker) {
    for path in [&worker.stop_request_path, &worker.stop_outcome_path] {
        let _ = std::fs::remove_file(path);
    }
}

fn kgw_worker_remove_stop_request_v1(worker: &KgwParallelSelfWorker) {
    let _ = std::fs::remove_file(&worker.stop_request_path);
}

fn kgw_worker_finalize_ownership_v1(worker: &mut KgwParallelSelfWorker) {
    kgw_runtime_owner_remove_files_v1(&worker.lease_path, &worker.lease_worker_path);
    kgw_worker_remove_stop_control_v1(worker);
}

fn kgw_worker_read_stop_outcome_v1(
    path: &std::path::Path,
) -> Result<KgwStopOutcomeMessageV1, String> {
    let payload = std::fs::read(path)
        .map_err(|error| format!("read stop outcome failed {}: {error}", path.display()))?;
    serde_json::from_slice(&payload)
        .map_err(|error| format!("parse stop outcome failed {}: {error}", path.display()))
}

pub(crate) fn kgw_worker_validate_stop_outcome_v1(
    message: KgwStopOutcomeMessageV1,
    role: &str,
    network: &str,
    worker_pid: u32,
) -> Result<KgwValidatedStopOutcomeV1, String> {
    if message.version != 1 {
        return Err(format!(
            "stop attestation protocol version mismatch;expected=1;actual={}",
            message.version
        ));
    }
    if message.runtime_role != role
        || message.network != network
        || message.worker_pid != worker_pid
    {
        return Err(format!(
            "stop attestation identity mismatch;expected_role={role};actual_role={};expected_network={network};actual_network={};expected_pid={worker_pid};actual_pid={}",
            message.runtime_role, message.network, message.worker_pid
        ));
    }

    match message.outcome.as_str() {
        "STOPPED" => {
            if !message.all_owned_components_terminal {
                return Err(format!(
                    "stop attestation STOPPED lacks terminal ownership proof;role={role};network={network};pid={worker_pid}"
                ));
            }
            message
                .evidence
                .filter(|evidence| !evidence.trim().is_empty())
                .map(KgwValidatedStopOutcomeV1::Stopped)
                .ok_or_else(|| {
                    format!(
                        "stop attestation STOPPED is missing evidence;role={role};network={network};pid={worker_pid}"
                    )
                })
        }
        "FAILED" => {
            let error = message
                .error
                .filter(|error| !error.trim().is_empty())
                .ok_or_else(|| {
                    format!(
                        "stop attestation FAILED is missing an error;role={role};network={network};pid={worker_pid}"
                    )
                })?;
            Ok(KgwValidatedStopOutcomeV1::Failed {
                error: format!(
                    "official graceful shutdown failed;role={role};network={network};pid={worker_pid};error={error}"
                ),
                all_owned_components_terminal: message.all_owned_components_terminal,
            })
        }
        other => Err(format!(
            "stop attestation outcome is invalid;role={role};network={network};pid={worker_pid};outcome={other}"
        )),
    }
}

fn kgw_worker_read_startup_control_v1(
    path: &std::path::Path,
) -> Result<KgwStartupControlMessageV1, String> {
    let payload = std::fs::read(path)
        .map_err(|error| format!("read startup control failed {}: {error}", path.display()))?;
    serde_json::from_slice(&payload)
        .map_err(|error| format!("parse startup control failed {}: {error}", path.display()))
}

pub(crate) fn kgw_worker_validate_startup_attestation_v1(
    message: KgwStartupControlMessageV1,
    role: &str,
    network: &str,
) -> Result<String, String> {
    if message.version != 1 {
        return Err(format!(
            "startup attestation protocol version mismatch;expected=1;actual={}",
            message.version
        ));
    }

    if message.runtime_role != role || message.network != network {
        return Err(format!(
            "startup attestation identity mismatch;expected_role={};actual_role={};expected_network={};actual_network={}",
            role, message.runtime_role, network, message.network
        ));
    }

    match message.outcome.as_str() {
        "READY" => message
            .evidence
            .filter(|evidence| !evidence.trim().is_empty())
            .ok_or_else(|| {
                format!(
                    "startup attestation READY is missing evidence;role={};network={}",
                    role, network
                )
            }),
        "FAILED" => Err(format!(
            "role startup failed;role={};network={};error={}",
            role,
            network,
            message
                .error
                .unwrap_or_else(|| "self-worker reported FAILED without an error".to_string())
        )),
        other => Err(format!(
            "startup attestation outcome is invalid;role={};network={};outcome={other}",
            role, network
        )),
    }
}

fn kgw_worker_remove_startup_control_v1(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

fn kgw_worker_wait_for_startup_attestation_v1(
    child: &mut Child,
    path: &std::path::Path,
    role: &str,
    network: &str,
    logs: &Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>,
) -> Result<String, String> {
    let timeout_ms = std::env::var("KGW_TEST_STARTUP_ATTESTATION_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(match role {
            "bridge" => KGW_BRIDGE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1,
            _ => KGW_NODE_PARENT_STARTUP_ATTESTATION_TIMEOUT_MS_V1,
        });
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

    loop {
        if path.is_file() {
            let message = kgw_worker_read_startup_control_v1(path);
            kgw_worker_remove_startup_control_v1(path);
            return kgw_worker_validate_startup_attestation_v1(message?, role, network);
        }

        if let Some(exit_status) = child.try_wait().map_err(|error| error.to_string())? {
            kgw_worker_remove_startup_control_v1(path);
            let captured = logs
                .lock()
                .map(|guard| {
                    guard
                        .iter()
                        .map(|entry| kgw_worker_safe_child_line_v1(&entry.raw_text))
                        .collect::<Vec<_>>()
                        .join(" | ")
                })
                .unwrap_or_else(|_| "worker log lock failed".to_string());
            let exit_code = exit_status
                .code()
                .map_or_else(|| "unknown".to_string(), |code| code.to_string());
            return Err(format!(
                "self-worker exited before role readiness;role={};network={};status={};exit_code={};logs={}",
                role, network, exit_status, exit_code, captured
            ));
        }

        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            kgw_worker_remove_startup_control_v1(path);
            return Err(format!(
                "role startup attestation timed out;role={};network={};timeout_ms={timeout_ms}",
                role, network
            ));
        }

        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn kgw_worker_spawn_reader<R>(
    role: String,
    network: String,
    bridge_instance_id: Option<String>,
    stream: String,
    child_pid: u32,
    reader: R,
    logs: Arc<Mutex<VecDeque<KgwRuntimeRawLogEntryV1>>>,
) -> std::thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let buffered = BufReader::new(reader);

        for line in buffered.lines() {
            match line {
                Ok(line) => {
                    if kgw_start_trace_enabled_v1() {
                        let mirror = kgw_worker_format_child_mirror_line_v1(
                            &role,
                            &network,
                            bridge_instance_id.as_deref(),
                            &stream,
                            child_pid,
                            &line,
                        );
                        eprintln!("{mirror}");
                        kgw_start_trace_capture_test_line_v1(&mirror);
                    }

                    let entry = kgw_worker_raw_log_entry_v1(&role, &network, &stream, line);
                    kgw_worker_push_raw_log(&logs, entry);
                }
                Err(_error) => {
                    kgw_start_trace_emit_v1(
                        "native",
                        "native.diagnostic_transport_reader_error",
                        &network,
                        "read-child-line",
                        "error",
                        Some(&format!(
                            "{{\"eventKind\":\"diagnostic_transport_record\",\"runtimeRole\":\"{}\",\"stream\":\"{}\",\"pid\":{},\"error\":\"{}\"}}",
                            kgw_start_trace_json_escape_v1(&role),
                            kgw_start_trace_json_escape_v1(&stream),
                            child_pid,
                            kgw_start_trace_json_escape_v1(&_error.to_string())
                        )),
                    );
                    break;
                }
            }
        }
    })
}

fn kgw_worker_join_readers_v1(reader_handles: Vec<std::thread::JoinHandle<()>>) {
    for reader_handle in reader_handles {
        let _ = reader_handle.join();
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_capture_raw_pipe_for_test_v1<R>(
    role: &str,
    network: &str,
    stream: &str,
    reader: R,
    logs: KgwRawProcessLogBufferV1,
) -> std::thread::JoinHandle<()>
where
    R: std::io::Read + Send + 'static,
{
    kgw_worker_spawn_reader(
        role.to_string(),
        network.to_string(),
        None,
        stream.to_string(),
        1,
        reader,
        logs,
    )
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_empty_raw_log_buffer_for_test_v1() -> KgwRawProcessLogBufferV1 {
    Arc::new(Mutex::new(VecDeque::new()))
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_raw_log_buffer_entries_for_test_v1(
    logs: &KgwRawProcessLogBufferV1,
) -> Vec<KgwRuntimeRawLogEntryV1> {
    logs.lock()
        .map(|entries| entries.iter().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_clear_raw_process_log_buffers_for_test_v1() {
    if let Ok(mut buffers) = kgw_raw_process_log_buffers_v1().lock() {
        buffers.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn kgw_worker_command(
    role: &str,
    network: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
    startup_control_path: &std::path::Path,
    stop_request_path: &std::path::Path,
    stop_outcome_path: &std::path::Path,
    parent_identity: &KgwProcessIdentityV1,
) -> Result<Command, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(exe);
    kgw_worker_apply_current_dir_v1(&mut command);

    #[cfg(test)]
    if let Some(path) = std::env::var_os("KGW_TEST_SELF_WORKER_MISSING_COMMAND") {
        let mut missing = Command::new(path);
        kgw_worker_apply_current_dir_v1(&mut missing);
        missing
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(missing);
    }

    #[cfg(test)]
    if let Some(fail_line) = std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_COMMAND") {
        command
            .arg("--exact")
            .arg("integrated_runtime_commands::kgw_test_self_worker_fail")
            .arg("--nocapture")
            .env("KGW_TEST_SELF_WORKER_FAIL_CHILD", "1")
            .env("KGW_TEST_SELF_WORKER_FAIL_STDERR", fail_line)
            .env("KGW_TEST_SELF_WORKER_ROLE", role)
            .env("KGW_TEST_SELF_WORKER_NETWORK", network)
            .env("KGW_TEST_SELF_WORKER_CONTROL_PATH", startup_control_path)
            .env("KGW_TEST_SELF_WORKER_STOP_REQUEST_PATH", stop_request_path)
            .env("KGW_TEST_SELF_WORKER_STOP_OUTCOME_PATH", stop_outcome_path)
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_PID",
                parent_identity.pid.to_string(),
            )
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_START_TIME",
                parent_identity.start_time.to_string(),
            )
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_EXECUTABLE",
                &parent_identity.executable,
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(command);
    }

    #[cfg(test)]
    if let Some(fail_line) = std::env::var_os("KGW_TEST_SELF_WORKER_DELAYED_FAIL_COMMAND") {
        command
            .arg("--exact")
            .arg("integrated_runtime_commands::kgw_test_self_worker_delayed_fail")
            .arg("--nocapture")
            .env("KGW_TEST_SELF_WORKER_DELAYED_FAIL_CHILD", "1")
            .env("KGW_TEST_SELF_WORKER_DELAYED_FAIL_STDERR", fail_line)
            .env("KGW_TEST_SELF_WORKER_ROLE", role)
            .env("KGW_TEST_SELF_WORKER_NETWORK", network)
            .env("KGW_TEST_SELF_WORKER_CONTROL_PATH", startup_control_path)
            .env("KGW_TEST_SELF_WORKER_STOP_REQUEST_PATH", stop_request_path)
            .env("KGW_TEST_SELF_WORKER_STOP_OUTCOME_PATH", stop_outcome_path)
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_PID",
                parent_identity.pid.to_string(),
            )
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_START_TIME",
                parent_identity.start_time.to_string(),
            )
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_EXECUTABLE",
                &parent_identity.executable,
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        return Ok(command);
    }

    #[cfg(test)]
    if std::env::var_os("KGW_TEST_SELF_WORKER_COMMAND").is_some() {
        command
            .arg("--exact")
            .arg("integrated_runtime_commands::kgw_test_self_worker_hold")
            .arg("--nocapture")
            .env("KGW_TEST_SELF_WORKER_CHILD", "1")
            .env("KGW_TEST_SELF_WORKER_ROLE", role)
            .env("KGW_TEST_SELF_WORKER_NETWORK", network)
            .env("KGW_TEST_SELF_WORKER_CONTROL_PATH", startup_control_path)
            .env("KGW_TEST_SELF_WORKER_STOP_REQUEST_PATH", stop_request_path)
            .env("KGW_TEST_SELF_WORKER_STOP_OUTCOME_PATH", stop_outcome_path)
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_PID",
                parent_identity.pid.to_string(),
            )
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_START_TIME",
                parent_identity.start_time.to_string(),
            )
            .env(
                "KGW_TEST_SELF_WORKER_PARENT_EXECUTABLE",
                &parent_identity.executable,
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        for key in [
            "KGW_TEST_SELF_WORKER_READY_DELAY_MS",
            "KGW_TEST_SELF_WORKER_STDOUT",
            "KGW_TEST_SELF_WORKER_STDERR",
            "KGW_TEST_SELF_WORKER_HANG_ON_STOP",
            "KGW_TEST_SELF_WORKER_TIMEOUT_ON_STOP",
            "KGW_TEST_SELF_WORKER_FAIL_ON_STOP",
            "KGW_TEST_SELF_WORKER_BRIDGE_LISTENER_FAIL_ON_STOP",
            "KGW_TEST_SELF_WORKER_EXIT_AFTER_READY_MS",
            "KGW_TEST_SELF_WORKER_OWNED_NODE_STOP_MARKER_PATH",
        ] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        return Ok(command);
    }

    command
        .arg("--kgw-self-worker")
        .arg(role)
        .arg("--network")
        .arg(network)
        .arg("--appdir")
        .arg(&settings.app_dir_name)
        .arg("--rpc")
        .arg(&settings.rpc_endpoint)
        .args(
            settings
                .p2p_listen
                .as_deref()
                .into_iter()
                .flat_map(|listen| ["--listen", listen]),
        )
        .arg("--startup-control-path")
        .arg(startup_control_path)
        .arg("--stop-request-path")
        .arg(stop_request_path)
        .arg("--stop-outcome-path")
        .arg(stop_outcome_path)
        .arg("--desktop-parent-pid")
        .arg(parent_identity.pid.to_string())
        .arg("--desktop-parent-start-time")
        .arg(parent_identity.start_time.to_string())
        .arg("--desktop-parent-executable")
        .arg(&parent_identity.executable)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    Ok(command)
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests; unused in the library test target.
pub(crate) fn kgw_worker_node_command_args_for_test_v1(
    role: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
) -> Result<Vec<String>, String> {
    let normalized_role = role.trim().to_ascii_lowercase();
    let network = settings.network.as_str().to_string();
    let control_path = std::env::temp_dir()
        .join("KaspaGateway")
        .join("startup-control")
        .join("kgw-startup-control-args-test.json");
    let stop_request_path = std::env::temp_dir()
        .join("KaspaGateway")
        .join("stop-control")
        .join("kgw-stop-request-args-test.json");
    let stop_outcome_path = std::env::temp_dir()
        .join("KaspaGateway")
        .join("stop-control")
        .join("kgw-stop-outcome-args-test.json");
    let parent_identity = kgw_process_identity_for_worker_v1(std::process::id())?;
    let mut command = kgw_worker_command(
        &normalized_role,
        &network,
        settings,
        &control_path,
        &stop_request_path,
        &stop_outcome_path,
        &parent_identity,
    )?;

    if normalized_role == "node" {
        if settings.enable_utxo_index && !kgw_worker_uses_test_command() {
            command.arg("--utxoindex");
        }

        if settings.archival && !kgw_worker_uses_test_command() {
            command.arg("--archival");
        }
    }

    Ok(command
        .get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect())
}

fn kgw_worker_role_from_request(
    runtime_role: Option<&str>,
    settings: &kaspa_gateway_rk_node::NodeSettings,
) -> String {
    if let Some(role) = runtime_role {
        let role = role.trim().to_ascii_lowercase();
        if role == "node" || role == "bridge" {
            return role;
        }
    }

    if settings.bridge_kind != kaspa_gateway_rk_node::BridgeNodeKind::Disable
        && settings.node_kind == kaspa_gateway_rk_node::KaspadNodeKind::Remote
    {
        return "bridge".to_string();
    }

    "node".to_string()
}

fn kgw_validate_network_start_policy(
    network: kaspa_gateway_rk_node::KgwNetwork,
    experimental_network_opt_in: Option<bool>,
) -> Result<(), String> {
    if network == kaspa_gateway_rk_node::KgwNetwork::Testnet12
        && experimental_network_opt_in != Some(true)
    {
        return Err(
            "start_blocked=true;start_allowed=false;network=testnet12;block_reason=experimental-network-opt-in-required;message=Testnet 12 is experimental and disabled by default. Enable it explicitly in this network tab before starting."
                .to_string(),
        );
    }

    Ok(())
}

fn kgw_worker_start(
    role: &str,
    settings: &kaspa_gateway_rk_node::NodeSettings,
    bridge_instance_id: Option<String>,
    bridge_structured_instances: Option<String>,
    bridge_config_path: Option<String>,
    bridge_instance_listens_override: Option<Vec<String>>,
) -> Result<String, String> {
    let network = settings.network.as_str().to_string();
    let role = role.trim().to_ascii_lowercase();
    let key = kgw_worker_key(&role, &network);
    let bridge_node_mode =
        if settings.bridge_kind == kaspa_gateway_rk_node::BridgeNodeKind::OfficialInProcessNode {
            "inprocess"
        } else {
            "external"
        };
    let trace_node_mode = kgw_worker_node_mode_for_trace_v1(&role, bridge_node_mode);
    // Listener selection is management metadata only. Upstream Bridge logging
    // is process-wide and offers no structural record-to-listener attribution.
    let _ = bridge_instance_id;
    let has_bridge_config_path = bridge_config_path
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let mut reconciliation_evidence = Vec::new();
    if let Some(evidence) =
        kgw_runtime_owner_reconcile_stale_lease_v1(&role, &network, &settings.app_dir_name)?
    {
        reconciliation_evidence.push(evidence);
    }
    if role == "bridge" && bridge_node_mode == "inprocess" {
        if let Some(evidence) =
            kgw_runtime_owner_reconcile_stale_lease_v1("node", &network, &settings.app_dir_name)?
        {
            reconciliation_evidence.push(evidence);
        }
    } else if role == "node" {
        let bridge_lease_path = kgw_runtime_owner_lease_path_v1("bridge", &network);
        if bridge_lease_path.is_file()
            && kgw_runtime_owner_read_lease_v1(&bridge_lease_path)
                .is_ok_and(|lease| lease.node_mode == "inprocess")
            && let Some(evidence) = kgw_runtime_owner_reconcile_stale_lease_v1(
                "bridge",
                &network,
                &settings.app_dir_name,
            )?
        {
            reconciliation_evidence.push(evidence);
        }
    }

    kgw_start_trace_emit_v1(
        "native",
        "native.runtime_owner_check_started",
        &network,
        "start",
        "checking",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\"}}",
            role, trace_node_mode
        )),
    );

    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    if role == "bridge" && bridge_node_mode == "inprocess" {
        let node_key = kgw_worker_key("node", &network);
        let mut remove_stale_node = false;

        if let Some(existing) = workers.get_mut(&node_key) {
            let running = existing
                .child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_none();

            if running {
                kgw_start_trace_emit_v1(
                    "native",
                    "native.runtime_owner_checked",
                    &network,
                    "start",
                    "blocked",
                    Some(&format!(
                        "{{\"runtimeRole\":\"bridge\",\"blockReason\":\"node-tab-owner-running\",\"existingPid\":{}}}",
                        existing.child.id()
                    )),
                );
                return Err(format!(
                    "start_blocked=true;start_allowed=false;runtime_role=bridge;network={};node_mode=inprocess;block_reason=node-tab-owner-running;message=Cannot start bridge in-process because the same-network node is already running. Stop the node first or use external bridge mode.;node_pid={};appdir={}",
                    network,
                    existing.child.id(),
                    existing.appdir
                ));
            }

            let _ = existing.child.wait();
            remove_stale_node = true;
        }

        if remove_stale_node && let Some(mut stale) = workers.remove(&node_key) {
            kgw_worker_finalize_ownership_v1(&mut stale);
        }
    }

    if role == "node" {
        let bridge_key = kgw_worker_key("bridge", &network);
        let mut remove_stale_bridge = false;

        if let Some(existing) = workers.get_mut(&bridge_key) {
            let running = existing
                .child
                .try_wait()
                .map_err(|error| error.to_string())?
                .is_none();

            if running && existing.node_mode == "inprocess" {
                kgw_start_trace_emit_v1(
                    "native",
                    "native.runtime_owner_checked",
                    &network,
                    "start",
                    "blocked",
                    Some(&format!(
                        "{{\"runtimeRole\":\"node\",\"blockReason\":\"bridge-inprocess-owner-running\",\"existingPid\":{}}}",
                        existing.child.id()
                    )),
                );
                return Err(format!(
                    "start_blocked=true;start_allowed=false;runtime_role=node;network={};node_mode=inprocess;block_reason=bridge-inprocess-owner-running;message=Cannot start node because bridge in-process owns this network. Stop the bridge first.;bridge_pid={};appdir={}",
                    network,
                    existing.child.id(),
                    existing.appdir
                ));
            }

            if !running {
                let _ = existing.child.wait();
                remove_stale_bridge = true;
            }
        }

        if remove_stale_bridge && let Some(mut stale) = workers.remove(&bridge_key) {
            kgw_worker_finalize_ownership_v1(&mut stale);
        }
    }

    if let Some(existing) = workers.get_mut(&key) {
        if existing
            .child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_none()
        {
            kgw_start_trace_emit_v1(
                "native",
                "native.runtime_owner_checked",
                &network,
                "start",
                "blocked",
                Some(&format!(
                    "{{\"runtimeRole\":\"{}\",\"blockReason\":\"duplicate-owner\",\"existingPid\":{},\"nodeMode\":\"{}\"}}",
                    existing.role,
                    existing.child.id(),
                    existing.node_mode
                )),
            );
            return Err(format!(
                "start_blocked=true;start_allowed=false;block_reason=duplicate-owner;runtime_role={};network={};pid={};appdir={};node_mode={};readiness=READY;readiness_evidence={};message=Process owner already exists for this network and role.",
                existing.role,
                existing.network,
                existing.child.id(),
                existing.appdir,
                existing.node_mode,
                existing.readiness_evidence
            ));
        }

        let _ = existing.child.wait();
        if let Some(mut stale) = workers.remove(&key) {
            kgw_worker_finalize_ownership_v1(&mut stale);
        }
    }

    kgw_start_trace_emit_v1(
        "native",
        "native.runtime_owner_checked",
        &network,
        "start",
        "available",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\"}}",
            role, trace_node_mode
        )),
    );
    kgw_runtime_clear_terminal_diagnostic_v1(&role, &network);

    let startup_control_path = kgw_worker_startup_control_path_v1(&role, &network);
    let (stop_request_path, stop_outcome_path) = kgw_worker_stop_control_paths_v1(&role, &network);
    let lease_path = kgw_runtime_owner_lease_path_v1(&role, &network);
    let parent_identity = kgw_process_identity_for_worker_v1(std::process::id())?;
    let starting_lease = KgwRuntimeOwnerLeaseV1 {
        version: KGW_RUNTIME_OWNER_LEASE_VERSION_V1,
        runtime_role: role.clone(),
        network: network.clone(),
        appdir: settings.app_dir_name.clone(),
        node_mode: if role == "bridge" {
            bridge_node_mode.to_string()
        } else {
            trace_node_mode.clone()
        },
        parent_pid: parent_identity.pid,
        parent_start_time: parent_identity.start_time,
        parent_executable: parent_identity.executable.clone(),
        published_ms: kgw_worker_now_ms(),
    };
    kgw_runtime_owner_reserve_lease_v1(&lease_path, &starting_lease)?;
    let mut lease_guard = KgwRuntimeOwnerReservationGuardV1::new(lease_path.clone());
    kgw_worker_control_directory_v1(&startup_control_path)?;
    kgw_worker_remove_startup_control_v1(&startup_control_path);
    kgw_worker_control_directory_v1(&stop_request_path)?;
    for path in [&stop_request_path, &stop_outcome_path] {
        let _ = std::fs::remove_file(path);
    }
    let mut command = kgw_worker_command(
        &role,
        &network,
        settings,
        &startup_control_path,
        &stop_request_path,
        &stop_outcome_path,
        &parent_identity,
    )?;

    if role == "bridge" {
        // KGW_BRIDGE_NORMAL_LOG_DEFAULT_R130
        // Normal bridge runs must not default to verbose protocol DEBUG logs.
        // Developers can still opt in explicitly by setting RUST_LOG before launch.
        let bridge_rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        command.env("RUST_LOG", bridge_rust_log);
    }

    let test_command = kgw_worker_uses_test_command();

    if role == "node" {
        if settings.enable_utxo_index && !test_command {
            command.arg("--utxoindex");
        }

        if settings.archival && !test_command {
            command.arg("--archival");
        }
    } else if !test_command {
        command.arg("--stratum").arg(&settings.stratum_listen);
        command.arg("--node-mode").arg(bridge_node_mode);

        // KGW_BRIDGE_DUAL_CLI_CONFIG_REAL_RUNNER_R122
        // Config route wins over UI/CLI instances to avoid mixing two instance sources.
        if let Some(config_path) = bridge_config_path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            command.arg("--bridge-config").arg(config_path);
        } else {
            let bridge_instance_listens = bridge_instance_listens_override.unwrap_or_else(|| {
                kgw_bridge_structured_instance_listens_r120(bridge_structured_instances.as_deref())
            });

            for listen in &bridge_instance_listens {
                command.arg("--bridge-instance-listen").arg(listen);
            }
        }

        if bridge_node_mode == "inprocess" {
            if settings.enable_utxo_index {
                command.arg("--utxoindex");
            }

            if settings.archival {
                command.arg("--archival");
            }
        }

        if settings.bridge_internal_cpu_miner {
            command.arg("--internal-cpu-miner");

            if let Some(address) = &settings.bridge_internal_cpu_miner_address {
                command.arg("--internal-cpu-miner-address").arg(address);
            }

            if let Some(threads) = settings.bridge_internal_cpu_miner_threads {
                command
                    .arg("--internal-cpu-miner-threads")
                    .arg(threads.to_string());
            }

            if let Some(throttle_ms) = settings.bridge_internal_cpu_miner_throttle_ms {
                command
                    .arg("--internal-cpu-miner-throttle-ms")
                    .arg(throttle_ms.to_string());
            }

            if let Some(template_poll_ms) = settings.bridge_internal_cpu_miner_template_poll_ms {
                command
                    .arg("--internal-cpu-miner-template-poll-ms")
                    .arg(template_poll_ms.to_string());
            }
        }
    }

    let spawn_plan_details = kgw_worker_spawn_plan_details_v1(
        &role,
        &network,
        &trace_node_mode,
        settings,
        &command,
        test_command,
        has_bridge_config_path,
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.spawn_plan_created",
        &network,
        "start",
        "ok",
        Some(&spawn_plan_details),
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.spawn_attempted",
        &network,
        "start",
        "attempting",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\"}}",
            role, trace_node_mode
        )),
    );

    // Raw buffers are registered immediately before spawn and retained
    // independently of live child ownership. Genuine official output emitted
    // before startup failure or process exit remains queryable through logs IPC.
    let logs = kgw_worker_replace_raw_log_buffer_v1(&role, &network)?;
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            kgw_start_trace_emit_v1(
                "native",
                "native.spawn_failed",
                &network,
                "start",
                "error",
                Some(&format!(
                    "{{\"runtimeRole\":\"{}\",\"errorKind\":\"{:?}\"}}",
                    role,
                    error.kind()
                )),
            );
            return Err(format!(
                "spawn_failed=true;runtime_role={};network={};source=self-worker;error={}",
                role, network, error
            ));
        }
    };
    let pid = child.id();
    let worker_identity = match kgw_process_identity_for_worker_v1(pid) {
        Ok(identity) => identity,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "spawn_failed=true;runtime_role={role};network={network};source=self-worker;error=exact child identity unavailable after spawn: {error}"
            ));
        }
    };
    if worker_identity.executable != parent_identity.executable {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "spawn_failed=true;runtime_role={role};network={network};source=self-worker;error=same-executable identity mismatch;parent_executable={};worker_executable={}",
            parent_identity.executable, worker_identity.executable
        ));
    }
    kgw_start_trace_emit_v1(
        "native",
        "native.spawn_succeeded",
        &network,
        "start",
        "ok",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{}}}",
            role, trace_node_mode, pid
        )),
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.child_pid_recorded",
        &network,
        "start",
        "ok",
        Some(&format!(
            "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{}}}",
            role, trace_node_mode, pid
        )),
    );

    let mut reader_handles = Vec::with_capacity(2);
    if let Some(stdout) = child.stdout.take() {
        reader_handles.push(kgw_worker_spawn_reader(
            role.clone(),
            network.clone(),
            None,
            "stdout".to_string(),
            pid,
            stdout,
            Arc::clone(&logs),
        ));
        kgw_start_trace_emit_v1(
            "native",
            "native.stdout_reader_attached",
            &network,
            "start",
            "attached",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    } else {
        kgw_start_trace_emit_v1(
            "native",
            "native.stdout_reader_attached",
            &network,
            "start",
            "missing",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    }

    if let Some(stderr) = child.stderr.take() {
        reader_handles.push(kgw_worker_spawn_reader(
            role.clone(),
            network.clone(),
            None,
            "stderr".to_string(),
            pid,
            stderr,
            Arc::clone(&logs),
        ));
        kgw_start_trace_emit_v1(
            "native",
            "native.stderr_reader_attached",
            &network,
            "start",
            "attached",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    } else {
        kgw_start_trace_emit_v1(
            "native",
            "native.stderr_reader_attached",
            &network,
            "start",
            "missing",
            Some(&format!("{{\"runtimeRole\":\"{}\",\"pid\":{}}}", role, pid)),
        );
    }

    // Publish exact durable ownership only after both native output readers are
    // attached, but before waiting for READY. This covers the complete startup
    // window without delaying collection of early child stderr/stdout.
    let lease_worker_path = match kgw_runtime_owner_update_spawned_lease_v1(
        &lease_path,
        &parent_identity,
        &worker_identity,
    ) {
        Ok(path) => path,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            kgw_worker_join_readers_v1(reader_handles);
            return Err(format!(
                "runtime owner lease publish failed;role={role};network={network};pid={pid};error={error}"
            ));
        }
    };
    lease_guard.set_worker_path(lease_worker_path.clone());

    let readiness_evidence = match kgw_worker_wait_for_startup_attestation_v1(
        &mut child,
        &startup_control_path,
        &role,
        &network,
        &logs,
    ) {
        Ok(evidence) => evidence,
        Err(error) => {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
                let _ = child.wait();
            }
            kgw_worker_join_readers_v1(reader_handles);
            kgw_worker_remove_startup_control_v1(&startup_control_path);
            kgw_start_trace_emit_v1(
                "native",
                "native.startup_response_returned",
                &network,
                "start",
                "error",
                Some(&format!(
                    "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{},\"reason\":\"role-startup-attestation-failed\",\"error\":\"{}\"}}",
                    role,
                    trace_node_mode,
                    pid,
                    kgw_start_trace_json_escape_v1(&error)
                )),
            );
            return Err(error);
        }
    };

    if let Some(exit_status) = child.try_wait().map_err(|error| error.to_string())? {
        kgw_worker_join_readers_v1(reader_handles);
        kgw_start_trace_emit_v1(
            "native",
            "native.startup_response_returned",
            &network,
            "start",
            "error",
            Some(&format!(
                "{{\"runtimeRole\":\"{}\",\"nodeMode\":\"{}\",\"pid\":{},\"reason\":\"self-worker-exited-during-startup\",\"status\":\"{}\"}}",
                role, trace_node_mode, pid, exit_status
            )),
        );
        let captured = logs
            .lock()
            .map(|guard| {
                guard
                    .iter()
                    .map(|entry| kgw_worker_safe_child_line_v1(&entry.raw_text))
                    .collect::<Vec<_>>()
                    .join(" | ")
            })
            .unwrap_or_else(|_| "worker log lock failed".to_string());

        // `ExitStatus` display text differs by platform. Keep it for diagnostics,
        // but expose a stable numeric field for callers and cross-platform tests.
        let exit_code = exit_status
            .code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string());

        return Err(format!(
            "self-worker exited during startup;role={};network={};pid={};status={};exit_code={};logs={}",
            role, network, pid, exit_status, exit_code, captured
        ));
    }

    let stored_node_mode = if role == "bridge" {
        bridge_node_mode.to_string()
    } else {
        trace_node_mode.clone()
    };

    lease_guard.disarm();

    workers.insert(
        key,
        KgwParallelSelfWorker {
            role: role.clone(),
            network: network.clone(),
            appdir: settings.app_dir_name.clone(),
            node_mode: stored_node_mode.clone(),
            child,
            spawned_pid: pid,
            stop_request_path,
            stop_outcome_path,
            reader_handles,
            _raw_logs: logs,
            started_ms: kgw_worker_now_ms(),
            exit_logged: false,
            readiness_evidence: readiness_evidence.clone(),
            terminal_error: None,
            lease_path,
            lease_worker_path,
        },
    );

    Ok(format!(
        "parallel-owned-self-worker started;role={};network={};pid={};owner=self-worker;runtime_state=running;readiness=READY;readiness_evidence={};same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;appdir={};rpc={};stratum={};node_mode={};same_db_path=true;exclusive_node_owner_per_network=true;parent_bound=true;reconciliation={}",
        role,
        network,
        pid,
        readiness_evidence,
        settings.app_dir_name,
        settings.rpc_endpoint,
        settings.stratum_listen,
        stored_node_mode,
        if reconciliation_evidence.is_empty() {
            "none".to_string()
        } else {
            reconciliation_evidence.join("|")
        }
    ))
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_raw_process_log_buffer_count_for_test_v1() -> usize {
    kgw_raw_process_log_buffers_v1()
        .lock()
        .map(|buffers| buffers.len())
        .unwrap_or_default()
}

fn kgw_worker_stop(network: &str, runtime_role: Option<&str>) -> Result<Option<String>, String> {
    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let wanted_role = runtime_role.map(|role| role.trim().to_ascii_lowercase());
    let wanted_network = network.trim().to_ascii_lowercase();

    let mut keys = workers
        .iter()
        .filter_map(|(key, worker)| {
            let role_match = wanted_role
                .as_ref()
                .map(|role| role == &worker.role)
                .unwrap_or(true);

            if worker.network == wanted_network && role_match {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    keys.sort_by_key(|key| {
        workers
            .get(key)
            .map(|worker| if worker.role == "bridge" { 0 } else { 1 })
            .unwrap_or(2)
    });

    if keys.is_empty() {
        return Ok(None);
    }

    let mut lines = Vec::new();

    for key in keys {
        let worker = workers
            .get_mut(&key)
            .ok_or_else(|| format!("parallel self-worker disappeared during Stop;key={key}"))?;
        let result = kgw_worker_stop_one_v1(worker);
        let terminal = worker
            .child
            .try_wait()
            .map_err(|error| format!("verify stopped child failed;key={key};error={error}"))?
            .is_some();

        match result {
            Ok(result) if terminal => {
                let reader_handles = std::mem::take(&mut worker.reader_handles);
                kgw_worker_join_readers_v1(reader_handles);
                let mut worker = workers
                    .remove(&key)
                    .ok_or_else(|| format!("remove terminal self-worker failed;key={key}"))?;
                kgw_worker_finalize_ownership_v1(&mut worker);
                let stop_failed = result.stop_failed;
                lines.push(result.line);
                if stop_failed {
                    break;
                }
            }
            Ok(result) => {
                return Err(format!(
                    "stop failed;role={};network={};pid={};graceful={};forced={};reason=worker remained alive after terminal Stop path",
                    worker.role, worker.network, worker.spawned_pid, result.graceful, result.forced
                ));
            }
            Err(error) => return Err(error),
        }
    }

    Ok(Some(lines.join("\n")))
}

#[tauri::command]
pub fn kgw_shutdown_all_runtime_workers_v1() -> Result<String, String> {
    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let mut keys = workers.keys().cloned().collect::<Vec<_>>();
    keys.sort_by_key(|key| {
        workers
            .get(key)
            .map(|worker| if worker.role == "bridge" { 0 } else { 1 })
            .unwrap_or(2)
    });

    if keys.is_empty() {
        return Ok("parallel-owned-self-worker shutdown-all;stopped=0".to_string());
    }

    let mut stopped = Vec::new();

    for key in keys {
        let result = {
            let worker = workers.get_mut(&key).ok_or_else(|| {
                format!("parallel self-worker disappeared during shutdown-all;key={key}")
            })?;
            match kgw_worker_stop_one_v1(worker) {
                Ok(result) => result,
                Err(error) => {
                    stopped.push(format!(
                        "parallel-owned-self-worker shutdown-all FAILED;role={};network={};pid={};error={}",
                        worker.role,
                        worker.network,
                        worker.spawned_pid,
                        kgw_worker_stop_field_v1(&error)
                    ));
                    let terminal = worker
                        .child
                        .try_wait()
                        .map_err(|wait_error| {
                            format!(
                                "verify failed shutdown-all child failed;key={key};error={wait_error}"
                            )
                        })?
                        .is_some();
                    if terminal {
                        let reader_handles = std::mem::take(&mut worker.reader_handles);
                        kgw_worker_join_readers_v1(reader_handles);
                        let mut terminal_worker = workers.remove(&key).ok_or_else(|| {
                            format!("remove failed terminal shutdown-all worker failed;key={key}")
                        })?;
                        kgw_worker_finalize_ownership_v1(&mut terminal_worker);
                    } else {
                        return Err(stopped.join("\n"));
                    }
                    continue;
                }
            }
        };

        let terminal = workers
            .get_mut(&key)
            .ok_or_else(|| {
                format!("parallel self-worker disappeared during shutdown-all;key={key}")
            })?
            .child
            .try_wait()
            .map_err(|error| format!("verify shutdown-all child failed;key={key};error={error}"))?
            .is_some();
        if !terminal {
            let worker = workers.get(&key).ok_or_else(|| {
                format!("parallel self-worker disappeared during shutdown-all;key={key}")
            })?;
            stopped.push(format!(
                "parallel-owned-self-worker shutdown-all FAILED;role={};network={};pid={};error=worker remained alive",
                worker.role, worker.network, worker.spawned_pid
            ));
            continue;
        }

        let reader_handles = {
            let worker = workers.get_mut(&key).ok_or_else(|| {
                format!("parallel self-worker disappeared during shutdown-all;key={key}")
            })?;
            std::mem::take(&mut worker.reader_handles)
        };
        kgw_worker_join_readers_v1(reader_handles);
        let mut worker = workers
            .remove(&key)
            .ok_or_else(|| format!("remove terminal shutdown-all worker failed;key={key}"))?;
        kgw_worker_finalize_ownership_v1(&mut worker);
        stopped.push(if result.stop_failed {
            format!(
                "parallel-owned-self-worker shutdown-all FAILED;{}",
                result.line
            )
        } else {
            format!("parallel-owned-self-worker shutdown-all;{}", result.line)
        });
    }

    let output = stopped.join("\n");
    if output.contains("shutdown-all FAILED") {
        Err(output)
    } else {
        Ok(output)
    }
}

fn kgw_worker_stop_one_v1(
    worker: &mut KgwParallelSelfWorker,
) -> Result<KgwWorkerStopResultV1, String> {
    let pid = worker.child.id();
    if pid != worker.spawned_pid {
        return Err(format!(
            "force termination blocked by worker identity mismatch;role={};network={};spawned_pid={};child_pid={pid}",
            worker.role, worker.network, worker.spawned_pid
        ));
    }

    if let Some(exit_status) = worker
        .child
        .try_wait()
        .map_err(|error| format!("inspect worker before Stop failed: {error}"))?
    {
        return Ok(KgwWorkerStopResultV1 {
            line: format!(
                "parallel-owned-self-worker already stopped;role={};network={};pid={pid};running=false;graceful=false;forced=false;already_stopped=true;exit_status={exit_status};appdir={};node_mode={}",
                worker.role, worker.network, worker.appdir, worker.node_mode
            ),
            graceful: false,
            forced: false,
            stop_failed: false,
        });
    }

    let _ = std::fs::remove_file(&worker.stop_request_path);
    let _ = std::fs::remove_file(&worker.stop_outcome_path);
    let request = KgwStopRequestMessageV1 {
        version: 1,
        command: "STOP",
        runtime_role: &worker.role,
        network: &worker.network,
        worker_pid: pid,
    };
    let mut graceful_failure =
        kgw_worker_atomic_write_json_v1(&worker.stop_request_path, &request).err();
    let timeout_ms = std::env::var("KGW_TEST_PARENT_GRACEFUL_STOP_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(KGW_PARENT_GRACEFUL_STOP_TIMEOUT_MS_V1);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    let mut evidence = None;
    let mut attested = false;
    let mut failed_terminal_proof = false;
    let mut terminal_control_outcome_received = false;

    while (!terminal_control_outcome_received || attested || failed_terminal_proof)
        && std::time::Instant::now() < deadline
    {
        if worker.stop_outcome_path.is_file() {
            let outcome =
                kgw_worker_read_stop_outcome_v1(&worker.stop_outcome_path).and_then(|message| {
                    kgw_worker_validate_stop_outcome_v1(message, &worker.role, &worker.network, pid)
                });
            let _ = std::fs::remove_file(&worker.stop_outcome_path);
            match outcome {
                Ok(KgwValidatedStopOutcomeV1::Stopped(outcome_evidence)) => {
                    evidence = Some(outcome_evidence);
                    attested = true;
                    terminal_control_outcome_received = true;
                }
                Ok(KgwValidatedStopOutcomeV1::Failed {
                    error,
                    all_owned_components_terminal,
                }) => {
                    terminal_control_outcome_received = true;
                    failed_terminal_proof = all_owned_components_terminal;
                    graceful_failure = Some(error);
                }
                Err(error) => {
                    terminal_control_outcome_received = true;
                    graceful_failure = Some(error);
                }
            }
        }

        if let Some(exit_status) = worker
            .child
            .try_wait()
            .map_err(|error| format!("wait for graceful worker exit failed: {error}"))?
        {
            if !terminal_control_outcome_received && worker.stop_outcome_path.is_file() {
                match kgw_worker_read_stop_outcome_v1(&worker.stop_outcome_path).and_then(
                    |message| {
                        kgw_worker_validate_stop_outcome_v1(
                            message,
                            &worker.role,
                            &worker.network,
                            pid,
                        )
                    },
                ) {
                    Ok(KgwValidatedStopOutcomeV1::Stopped(outcome_evidence)) => {
                        evidence = Some(outcome_evidence);
                        attested = true;
                    }
                    Ok(KgwValidatedStopOutcomeV1::Failed {
                        error,
                        all_owned_components_terminal,
                    }) => {
                        failed_terminal_proof = all_owned_components_terminal;
                        graceful_failure = Some(error);
                    }
                    Err(error) => {
                        graceful_failure = Some(error);
                    }
                }
                let _ = std::fs::remove_file(&worker.stop_outcome_path);
            }
            if attested && exit_status.success() {
                return Ok(KgwWorkerStopResultV1 {
                    line: format!(
                        "parallel-owned-self-worker stopped;role={};network={};pid={pid};running=false;graceful=true;forced=false;stop_outcome=STOPPED;exit_status={exit_status};evidence={};appdir={};node_mode={}",
                        worker.role,
                        worker.network,
                        evidence.as_deref().unwrap_or("official-shutdown-joined"),
                        worker.appdir,
                        worker.node_mode
                    ),
                    graceful: true,
                    forced: false,
                    stop_failed: false,
                });
            }

            if graceful_failure.is_none() {
                graceful_failure = Some(format!(
                    "worker exited without valid STOPPED attestation;status={exit_status};attested={attested}"
                ));
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(25));
    }

    if graceful_failure.is_none() {
        graceful_failure = Some(format!(
            "graceful stop timed out;timeout_ms={timeout_ms};attested={attested}"
        ));
    }
    let reason = graceful_failure.unwrap_or_else(|| "unknown graceful Stop failure".to_string());

    if failed_terminal_proof
        && let Some(exit_status) = worker
            .child
            .try_wait()
            .map_err(|error| format!("inspect worker before force fallback failed: {error}"))?
    {
        return Ok(KgwWorkerStopResultV1 {
            line: format!(
                "parallel-owned-self-worker stopped with graceful failure;role={};network={};pid={pid};running=false;graceful=false;forced=false;stop_failed=true;stop_outcome=FAILED;exit_status={exit_status};reason={};appdir={};node_mode={}",
                worker.role,
                worker.network,
                kgw_worker_stop_field_v1(&reason),
                worker.appdir,
                worker.node_mode
            ),
            graceful: false,
            forced: false,
            stop_failed: true,
        });
    }

    // Force fallback is restricted to the exact Child handle and exact PID that
    // were recorded at spawn. This never performs PID discovery or broad kill.
    if worker.child.id() != worker.spawned_pid {
        return Err(format!(
            "force termination blocked by worker identity mismatch;role={};network={};spawned_pid={};child_pid={};reason={reason}",
            worker.role,
            worker.network,
            worker.spawned_pid,
            worker.child.id()
        ));
    }
    worker.child.kill().map_err(|error| {
        format!(
            "forced termination failed;role={};network={};pid={pid};graceful=false;forced=false;reason={reason};error={error}",
            worker.role, worker.network
        )
    })?;
    let exit_status = worker.child.wait().map_err(|error| {
        format!(
            "wait after forced termination failed;role={};network={};pid={pid};graceful=false;forced=true;reason={reason};error={error}",
            worker.role, worker.network
        )
    })?;

    Ok(KgwWorkerStopResultV1 {
        line: format!(
            "parallel-owned-self-worker stopped;role={};network={};pid={pid};running=false;graceful=false;forced=true;stop_outcome=FORCED;exit_status={exit_status};reason={};appdir={};node_mode={}",
            worker.role,
            worker.network,
            kgw_worker_stop_field_v1(&reason),
            worker.appdir,
            worker.node_mode
        ),
        graceful: false,
        forced: true,
        stop_failed: false,
    })
}

fn kgw_worker_stop_field_v1(value: &str) -> String {
    kgw_worker_safe_child_line_v1(value)
        .replace(';', ",")
        .replace('=', ":")
}

fn kgw_worker_status(
    network: Option<&str>,
    runtime_role: Option<&str>,
) -> Result<Option<String>, String> {
    let mut workers = kgw_parallel_self_workers()
        .lock()
        .map_err(|_| "parallel self-worker lock failed".to_string())?;

    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());

    let mut lines = Vec::new();

    for worker in workers.values_mut() {
        if let Some(ref value) = wanted_network
            && &worker.network != value
        {
            continue;
        }

        if let Some(ref value) = wanted_role
            && &worker.role != value
        {
            continue;
        }

        let exit_status = worker.child.try_wait().map_err(|error| error.to_string())?;
        let running = exit_status.is_none();

        if let Some(status) = exit_status
            && worker.terminal_error.is_none()
        {
            let typed_error = if worker.stop_outcome_path.is_file() {
                kgw_worker_read_stop_outcome_v1(&worker.stop_outcome_path)
                    .and_then(|message| {
                        kgw_worker_validate_stop_outcome_v1(
                            message,
                            &worker.role,
                            &worker.network,
                            worker.spawned_pid,
                        )
                    })
                    .ok()
                    .and_then(|outcome| match outcome {
                        KgwValidatedStopOutcomeV1::Failed { error, .. } => Some(error),
                        KgwValidatedStopOutcomeV1::Stopped(_) => None,
                    })
            } else {
                None
            };
            worker.terminal_error = Some(typed_error.unwrap_or_else(|| {
                format!(
                    "runtime terminated unexpectedly after READY;role={};network={};pid={};exit_status={status}",
                    worker.role, worker.network, worker.spawned_pid
                )
            }));
        }

        if let Some(error) = worker.terminal_error.as_deref() {
            kgw_runtime_record_terminal_diagnostic_v1(&worker.role, &worker.network, error);
        }
        if !running {
            kgw_runtime_owner_remove_files_v1(&worker.lease_path, &worker.lease_worker_path);
            kgw_worker_remove_stop_request_v1(worker);
        }

        if let Some(status) = exit_status
            && !worker.exit_logged
        {
            kgw_start_trace_emit_v1(
                "native",
                "native.diagnostic_transport_process_exit",
                &worker.network,
                "status",
                "observed",
                Some(&format!(
                    "{{\"eventKind\":\"diagnostic_transport_record\",\"runtimeRole\":\"{}\",\"pid\":{},\"status\":\"{}\"}}",
                    kgw_start_trace_json_escape_v1(&worker.role),
                    worker.child.id(),
                    kgw_start_trace_json_escape_v1(&status.to_string())
                )),
            );
            worker.exit_logged = true;
        }

        lines.push(format!(
            "parallel-owned-self-worker status;role={};network={};pid={};running={};readiness={};readiness_evidence={};runtime_error={};same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;appdir={};started_ms={};node_mode={}",
            worker.role,
            worker.network,
            worker.child.id(),
            running,
            if running {
                "READY"
            } else {
                "FAILED"
            },
            if running {
                worker.readiness_evidence.as_str()
            } else {
                "none"
            },
            worker
                .terminal_error
                .as_deref()
                .map(kgw_worker_stop_field_v1)
                .unwrap_or_else(|| "none".to_string()),
            worker.appdir,
            worker.started_ms,
            worker.node_mode
        ));
    }

    if lines.is_empty() {
        Ok(None)
    } else {
        Ok(Some(lines.join("\n")))
    }
}

fn kgw_worker_logs(
    network: Option<&str>,
    runtime_role: Option<&str>,
    _bridge_instance_id: Option<&str>,
) -> Result<Option<KgwRuntimeLogsReportV1>, String> {
    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());

    let buffers = kgw_raw_process_log_buffers_v1()
        .lock()
        .map_err(|_| "raw process log buffer registry lock failed".to_string())?;
    let mut matched_buffer = false;
    let mut lines = Vec::new();

    for (key, logs) in buffers.iter() {
        let (role, network) = key.split_once(':').unwrap_or(("", ""));
        if wanted_network
            .as_deref()
            .is_some_and(|value| network != value)
            || wanted_role.as_deref().is_some_and(|value| role != value)
        {
            continue;
        }
        matched_buffer = true;
        if let Ok(logs) = logs.lock() {
            lines.extend(logs.iter().cloned());
        }
    }

    if !matched_buffer {
        return Ok(None);
    }

    lines.sort_by_key(|entry| entry.sequence);

    let diagnostics = kgw_runtime_diagnostics_v1()
        .lock()
        .map_err(|_| "runtime diagnostic registry lock failed".to_string())?
        .values()
        .filter(|record| {
            wanted_network
                .as_deref()
                .is_none_or(|value| record.network == value)
                && wanted_role
                    .as_deref()
                    .is_none_or(|value| record.runtime_role == value)
        })
        .cloned()
        .collect::<Vec<_>>();

    Ok(Some(KgwRuntimeLogsReportV1 {
        version: "kgw_runtime_logs_v1".to_string(),
        network: wanted_network,
        runtime_role: wanted_role,
        // IPC compatibility field only. Official Bridge records are emitted by
        // a process-wide logger and cannot be attributed to one listener.
        bridge_instance_id: None,
        source: "self-worker".to_string(),
        buffer_limit: KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1,
        entries: lines,
        diagnostics,
    }))
}

fn kgw_worker_clear_logs(
    network: Option<&str>,
    runtime_role: Option<&str>,
    _bridge_instance_id: Option<&str>,
) -> Result<bool, String> {
    let buffers = kgw_raw_process_log_buffers_v1()
        .lock()
        .map_err(|_| "raw process log buffer registry lock failed".to_string())?;

    let wanted_network = network.map(|value| value.trim().to_ascii_lowercase());
    let wanted_role = runtime_role.map(|value| value.trim().to_ascii_lowercase());
    let mut matched_buffer = false;

    for (key, logs) in buffers.iter() {
        let (role, network) = key.split_once(':').unwrap_or(("", ""));
        if wanted_network
            .as_deref()
            .is_some_and(|value| network != value)
            || wanted_role.as_deref().is_some_and(|value| role != value)
        {
            continue;
        }

        matched_buffer = true;

        if let Ok(mut logs) = logs.lock() {
            logs.clear();
        }
    }

    Ok(matched_buffer)
}

fn kgw_empty_runtime_logs_report_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    source: &str,
    diagnostics: Vec<KgwRuntimeDiagnosticRecordV1>,
) -> KgwRuntimeLogsReportV1 {
    KgwRuntimeLogsReportV1 {
        version: "kgw_runtime_logs_v1".to_string(),
        network,
        runtime_role,
        bridge_instance_id,
        source: source.to_string(),
        buffer_limit: KGW_RAW_PROCESS_LOG_BUFFER_LIMIT_V1,
        entries: Vec::new(),
        diagnostics,
    }
}

fn kgw_inprocess_diagnostic_logs_report_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    controller_logs: String,
) -> KgwRuntimeLogsReportV1 {
    let line_count = controller_logs.lines().count();
    let diagnostics = if line_count == 0 {
        Vec::new()
    } else {
        vec![KgwRuntimeDiagnosticRecordV1 {
            diagnostic_event: "kgw_diagnostic_transport_record_v1".to_string(),
            network: network.clone().unwrap_or_else(|| "unknown".to_string()),
            source: "in-process-controller".to_string(),
            runtime_role: runtime_role
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            bridge_instance_id: bridge_instance_id.clone(),
            received_ms: kgw_worker_now_ms_u64(),
            message: format!(
                "Controller diagnostics are available separately; no child stdout/stderr raw output is available for this component. diagnosticLineCount={line_count}"
            ),
        }]
    };

    kgw_empty_runtime_logs_report_v1(
        network,
        runtime_role,
        bridge_instance_id,
        "in-process-controller",
        diagnostics,
    )
}
fn parse_network(
    network: Option<String>,
) -> Result<Option<kaspa_gateway_rk_node::KgwNetwork>, String> {
    match network {
        Some(value) => kaspa_gateway_rk_node::KgwNetwork::parse(&value)
            .map(Some)
            .map_err(|error| error.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn kgw_runtime_owner_summary_v1() -> Result<String, String> {
    Ok(kaspa_gateway_rk_node::exact_kgw_service_controller_summary_v1().to_string())
}

#[tauri::command]
pub fn kgw_runtime_owner_status_v1(
    network: Option<String>,
    runtime_role: Option<String>,
) -> Result<String, String> {
    if let Some(worker_status) = kgw_worker_status(network.as_deref(), runtime_role.as_deref())? {
        return Ok(worker_status);
    }

    let requested_role = runtime_role
        .as_deref()
        .map(|role| role.trim().to_ascii_lowercase());

    if requested_role.as_deref() == Some("bridge") {
        let network_label = network
            .as_deref()
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "all".to_string());

        return Ok(format!(
            "parallel-owned-self-worker status;role=bridge;network={};running=false;same_exe=true;external_kaspad_exe=false;uses_kaspa_libraries=true;node_mode=none;message=no bridge worker status yet",
            network_label
        ));
    }

    let network = parse_network(network)?;
    controller()
        .status(network)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_kgw_runtime_logs_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
) -> Result<KgwRuntimeLogsReportV1, String> {
    if let Some(worker_logs) = kgw_worker_logs(
        network.as_deref(),
        runtime_role.as_deref(),
        bridge_instance_id.as_deref(),
    )? {
        return Ok(worker_logs);
    }

    let bridge_instance_id = kgw_worker_clean_optional_identifier_v1(bridge_instance_id);
    let runtime_role = runtime_role
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let network_label = network
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let parsed_network = parse_network(network)?;
    let controller_logs = controller()
        .logs(parsed_network)
        .map_err(|error| error.to_string())?;

    Ok(kgw_inprocess_diagnostic_logs_report_v1(
        network_label,
        runtime_role,
        bridge_instance_id,
        controller_logs,
    ))
}

#[tauri::command]
pub fn kgw_kgw_runtime_clear_logs_v1(
    network: Option<String>,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
) -> Result<KgwRuntimeLogsReportV1, String> {
    let _ = kgw_worker_clear_logs(
        network.as_deref(),
        runtime_role.as_deref(),
        bridge_instance_id.as_deref(),
    )?;

    let bridge_instance_id = kgw_worker_clean_optional_identifier_v1(bridge_instance_id);
    let runtime_role = runtime_role
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let network_label = network
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    Ok(kgw_empty_runtime_logs_report_v1(
        network_label,
        runtime_role,
        bridge_instance_id,
        "self-worker",
        Vec::new(),
    ))
}

fn kgw_safe_runtime_appdir_root() -> std::path::PathBuf {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("KaspaGateway")
            .join("nodes")
    } else {
        std::env::temp_dir().join("KaspaGateway").join("nodes")
    }
}

fn kgw_network_runtime_appdir(network: kaspa_gateway_rk_node::KgwNetwork) -> String {
    kgw_safe_runtime_appdir_root()
        .join(network.as_str())
        .to_string_lossy()
        .to_string()
}

fn kgw_safe_runtime_appdir(value: String) -> String {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return kgw_safe_runtime_appdir_root()
            .join("default")
            .to_string_lossy()
            .to_string();
    }

    let input = std::path::PathBuf::from(trimmed);

    if input.is_absolute() {
        return input.to_string_lossy().to_string();
    }

    let safe_name = trimmed.replace(['\\', '/', ':'], "_");

    kgw_safe_runtime_appdir_root()
        .join(safe_name)
        .to_string_lossy()
        .to_string()
}
fn kgw_command_preview_find_cli_value(command_preview: &str, flag: &str) -> Option<String> {
    let mut parts = command_preview.split_whitespace().peekable();

    while let Some(part) = parts.next() {
        if let Some(value) = part.strip_prefix(&format!("{flag}=")) {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }

        if part == flag
            && let Some(value) = parts.peek()
        {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn kgw_command_preview_has_cli_flag(command_preview: &str, flag: &str) -> bool {
    command_preview
        .split_whitespace()
        .any(|part| part == flag || part.starts_with(&format!("{flag}=")))
}

fn kgw_bridge_preview_parse_u16(value: Option<String>) -> Option<u16> {
    value
        .and_then(|item| item.trim().parse::<u16>().ok())
        .filter(|item| *item > 0)
}

fn kgw_bridge_preview_parse_u64(value: Option<String>) -> Option<u64> {
    value
        .and_then(|item| item.trim().parse::<u64>().ok())
        .filter(|item| *item > 0)
}
fn kgw_command_preview_normalize_listen(value: String) -> String {
    let trimmed = value.trim().to_string();

    if trimmed.starts_with(':') {
        format!("0.0.0.0{}", trimmed)
    } else {
        trimmed
    }
}

// KGW_BRIDGE_DUAL_CLI_CONFIG_REAL_RUNNER_R122
// KGW_BRIDGE_PREVIEW_ALL_INSTANCES_R123
fn kgw_bridge_preview_instance_clause_listen_r123(instance_clause: &str) -> Option<String> {
    let clean = instance_clause.trim();

    if clean.is_empty() {
        return None;
    }

    for raw_part in clean.split(',') {
        let part = raw_part.trim();
        let Some((raw_key, raw_value)) = part.split_once('=').or_else(|| part.split_once(':'))
        else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_start_matches('-')
            .replace('-', "_")
            .to_ascii_lowercase();

        if !matches!(
            key.as_str(),
            "port" | "stratum" | "stratum_port" | "stratum_listen" | "listen"
        ) {
            continue;
        }

        let value = raw_value.trim().trim_matches('"').trim_matches('\'');

        if let Some(port) = kgw_bridge_normalize_instance_port_r110f(value) {
            return Some(kgw_command_preview_normalize_listen(port));
        }

        let tail = value.rsplit(':').next().unwrap_or(value).trim();
        if let Some(port) = kgw_bridge_normalize_instance_port_r110f(tail) {
            return Some(kgw_command_preview_normalize_listen(port));
        }
    }

    None
}

// KGW_BRIDGE_PREVIEW_ALL_INSTANCES_R123
fn kgw_bridge_command_preview_instance_listens_r123(command_preview: Option<&str>) -> Vec<String> {
    let Some(preview) = command_preview
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Vec::new();
    };

    let parts = preview.split_whitespace().collect::<Vec<_>>();
    let mut listens = Vec::<String>::new();
    let mut index = 0usize;

    while index < parts.len() {
        let part = parts[index];

        let value = if let Some(value) = part.strip_prefix("--instance=") {
            Some(value)
        } else if part == "--instance" {
            index += 1;
            parts.get(index).copied()
        } else {
            None
        };

        if let Some(instance_clause) = value
            && let Some(listen) = kgw_bridge_preview_instance_clause_listen_r123(instance_clause)
            && !listens.iter().any(|existing| existing == &listen)
        {
            listens.push(listen);
        }

        index += 1;
    }

    listens
}

fn kgw_bridge_config_path_from_preview_r122(command_preview: Option<&str>) -> Option<String> {
    let preview = command_preview?.trim();

    if preview.is_empty() {
        return None;
    }

    kgw_command_preview_find_cli_value(preview, "--config")
        .or_else(|| kgw_command_preview_find_cli_value(preview, "--bridge-config"))
        .or_else(|| kgw_command_preview_find_cli_value(preview, "--config-path"))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn kgw_apply_command_preview_overrides(
    settings: &mut kaspa_gateway_rk_node::NodeSettings,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
) {
    if let Some(node_preview) = node_command_preview {
        if let Some(value) = kgw_command_preview_find_cli_value(&node_preview, "--listen") {
            settings.p2p_listen = Some(kgw_command_preview_normalize_listen(value));
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&node_preview, "--rpclisten") {
            settings.rpc_endpoint = kgw_command_preview_normalize_listen(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&node_preview, "--appdir") {
            settings.app_dir_name = kgw_safe_runtime_appdir(value);
        }

        settings.enable_utxo_index = kgw_command_preview_has_cli_flag(&node_preview, "--utxoindex");

        settings.archival = kgw_command_preview_has_cli_flag(&node_preview, "--archival");
    }

    if let Some(bridge_preview) = bridge_command_preview {
        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--kaspad-address")
            .or_else(|| kgw_command_preview_find_cli_value(&bridge_preview, "--kaspa-rpc"))
        {
            settings.rpc_endpoint = kgw_command_preview_normalize_listen(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--stratum-listen")
            .or_else(|| kgw_command_preview_find_cli_value(&bridge_preview, "--stratum-port"))
        {
            settings.stratum_listen = kgw_command_preview_normalize_listen(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--appdir")
            && !value.trim().is_empty()
        {
            settings.app_dir_name = kgw_safe_runtime_appdir(value);
        }

        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--listen") {
            settings.p2p_listen = Some(kgw_command_preview_normalize_listen(value));
        }

        // KGW_BRIDGE_INPROCESS_SAME_DB_OWNER_V7_INTEGRATED
        if let Some(value) = kgw_command_preview_find_cli_value(&bridge_preview, "--node-mode") {
            match value.trim().to_ascii_lowercase().as_str() {
                "inprocess" | "inproc" | "official-inprocess-node" => {
                    settings.node_kind = kaspa_gateway_rk_node::KaspadNodeKind::IntegratedInProc;
                    settings.bridge_kind =
                        kaspa_gateway_rk_node::BridgeNodeKind::OfficialInProcessNode;
                    settings.enable_utxo_index =
                        kgw_command_preview_has_cli_flag(&bridge_preview, "--utxoindex");
                    settings.archival =
                        kgw_command_preview_has_cli_flag(&bridge_preview, "--archival");
                }
                "external" | "external-node" | "official-external-node" => {
                    settings.node_kind = kaspa_gateway_rk_node::KaspadNodeKind::Remote;
                    settings.bridge_kind =
                        kaspa_gateway_rk_node::BridgeNodeKind::OfficialExternalNode;
                }
                _ => {}
            }
        }

        settings.bridge_internal_cpu_miner =
            kgw_command_preview_has_cli_flag(&bridge_preview, "--internal-cpu-miner");
        settings.bridge_internal_cpu_miner_address =
            kgw_command_preview_find_cli_value(&bridge_preview, "--internal-cpu-miner-address");
        settings.bridge_internal_cpu_miner_threads = kgw_bridge_preview_parse_u16(
            kgw_command_preview_find_cli_value(&bridge_preview, "--internal-cpu-miner-threads"),
        );
        settings.bridge_internal_cpu_miner_throttle_ms = kgw_bridge_preview_parse_u64(
            kgw_command_preview_find_cli_value(&bridge_preview, "--internal-cpu-miner-throttle-ms"),
        );
        settings.bridge_internal_cpu_miner_template_poll_ms =
            kgw_bridge_preview_parse_u64(kgw_command_preview_find_cli_value(
                &bridge_preview,
                "--internal-cpu-miner-template-poll-ms",
            ));
    }
}

#[cfg(test)]
#[allow(dead_code)] // Used by path-included integration tests.
pub(crate) fn kgw_bridge_inprocess_preview_settings_for_test_v1(
    bridge_preview: String,
) -> Result<kaspa_gateway_rk_node::NodeSettings, String> {
    let mut settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        "mainnet".to_string(),
        "integrated-inproc".to_string(),
        "official-inprocess-node".to_string(),
    )
    .map_err(|error| error.to_string())?;
    kgw_apply_command_preview_overrides(&mut settings, None, Some(bridge_preview));
    Ok(settings)
}

// KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
fn kgw_bridge_instance_value_r110f(serialized: &str, wanted_key: &str) -> Option<String> {
    let wanted = wanted_key.trim().to_ascii_lowercase();

    for raw_part in serialized.split(',') {
        let part = raw_part.trim();
        let Some((raw_key, raw_value)) = part.split_once('=') else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_start_matches('-')
            .replace('-', "_")
            .to_ascii_lowercase();
        let value = raw_value.trim();

        if value.is_empty() {
            continue;
        }

        let matched = match wanted.as_str() {
            "port" => matches!(
                key.as_str(),
                "port" | "stratum" | "stratum_port" | "stratum_listen" | "listen"
            ),
            _ => key == wanted,
        };

        if matched {
            return Some(value.to_string());
        }
    }

    None
}

// KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
fn kgw_bridge_normalize_instance_port_r110f(value: &str) -> Option<String> {
    let clean = value.trim().trim_start_matches(':').trim();

    if clean.is_empty() {
        return None;
    }

    if !clean.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let Ok(port) = clean.parse::<u16>() else {
        return None;
    };

    if port == 0 {
        return None;
    }

    Some(format!(":{port}"))
}

// KGW_BRIDGE_ACTIVE_INSTANCE_RUNTIME_CONTRACT_R110F
// KGW_BRIDGE_DUAL_CLI_REAL_RUNNER_R120
fn kgw_bridge_normalize_instance_listen_r120(value: &str) -> Option<String> {
    let clean = value.trim().trim_matches('"').trim_matches('\'');

    if clean.is_empty() {
        return None;
    }

    if let Some(port) = kgw_bridge_normalize_instance_port_r110f(clean) {
        return Some(port);
    }

    let tail = clean.rsplit(':').next().unwrap_or(clean).trim();
    kgw_bridge_normalize_instance_port_r110f(tail)
}

// KGW_BRIDGE_DUAL_CLI_REAL_RUNNER_R120
fn kgw_bridge_instance_value_r120(serialized: &str, wanted_key: &str) -> Option<String> {
    let wanted = wanted_key.trim().replace('-', "_").to_ascii_lowercase();

    let normalized = serialized
        .replace(['{', '}', '[', ']', '"'], "")
        .replace("\\r", ",")
        .replace("\\n", ",");

    for raw_part in normalized.split([',', ';']) {
        let part = raw_part.trim();
        let Some((raw_key, raw_value)) = part.split_once('=').or_else(|| part.split_once(':'))
        else {
            continue;
        };

        let key = raw_key
            .trim()
            .trim_start_matches('-')
            .replace('-', "_")
            .to_ascii_lowercase();

        let value = raw_value.trim().trim_matches('\'');

        if value.is_empty() {
            continue;
        }

        let matched = match wanted.as_str() {
            "port" => matches!(
                key.as_str(),
                "port"
                    | "stratum"
                    | "stratum_port"
                    | "stratumport"
                    | "stratum_listen"
                    | "stratumlisten"
                    | "listen"
            ),
            _ => key == wanted,
        };

        if matched {
            return Some(value.to_string());
        }
    }

    None
}

// KGW_BRIDGE_DUAL_CLI_REAL_RUNNER_R120
fn kgw_bridge_structured_instance_listens_r120(serialized: Option<&str>) -> Vec<String> {
    let Some(raw) = serialized.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };

    let mut listens = Vec::<String>::new();

    for object_like in raw.split('{').skip(1) {
        let chunk = object_like.split('}').next().unwrap_or(object_like);

        if let Some(listen) = kgw_bridge_instance_value_r120(chunk, "port")
            .as_deref()
            .and_then(kgw_bridge_normalize_instance_listen_r120)
            && !listens.iter().any(|existing| existing == &listen)
        {
            listens.push(listen);
        }
    }

    if listens.is_empty()
        && let Some(listen) = kgw_bridge_instance_value_r120(raw, "port")
            .as_deref()
            .and_then(kgw_bridge_normalize_instance_listen_r120)
    {
        listens.push(listen);
    }

    listens
}

fn kgw_apply_bridge_active_instance_runtime_overrides_r110f(
    settings: &mut kaspa_gateway_rk_node::NodeSettings,
    bridge_active_instance_id: Option<String>,
    bridge_active_instance: Option<String>,
    bridge_active_instance_port: Option<String>,
    bridge_structured_instances: Option<String>,
) {
    let _ = bridge_active_instance_id;

    let structured_ports =
        kgw_bridge_structured_instance_listens_r120(bridge_structured_instances.as_deref());

    let explicit_port = bridge_active_instance_port
        .as_deref()
        .and_then(kgw_bridge_normalize_instance_port_r110f);

    let serialized_port = match bridge_active_instance.as_deref() {
        Some(serialized) => kgw_bridge_instance_value_r110f(serialized, "port")
            .as_deref()
            .and_then(kgw_bridge_normalize_instance_port_r110f),
        None => None,
    };

    if let Some(port) = explicit_port
        .or(serialized_port)
        .or_else(|| structured_ports.first().cloned())
    {
        settings.stratum_listen = kgw_command_preview_normalize_listen(port);
    }
}

#[allow(clippy::too_many_arguments)] // Stable Tauri IPC contract; grouping would break frontend argument names.
#[tauri::command]
pub fn kgw_kgw_apply_node_settings_v1(
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
    kgw_start_trace_emit_v1(
        "native",
        "native.tauri_command_entered",
        &network,
        "start",
        "entered",
        Some("{\"commandName\":\"kgw_kgw_apply_node_settings_v1\"}"),
    );
    kgw_start_trace_emit_v1(
        "native",
        "native.payload_received_and_validated",
        &network,
        "start",
        "received",
        Some(&format!(
            "{{\"hasNodeCommandPreview\":{},\"hasBridgeCommandPreview\":{},\"runtimeRole\":\"{}\",\"hasBridgeActiveInstance\":{},\"hasBridgeStructuredInstances\":{},\"experimentalOptIn\":{}}}",
            node_command_preview
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            bridge_command_preview
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            runtime_role.as_deref().unwrap_or(""),
            bridge_active_instance
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            bridge_structured_instances
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
            experimental_network_opt_in == Some(true)
        )),
    );

    let mut settings =
        match kaspa_gateway_rk_node::NodeSettings::from_strings(network, node_kind, bridge_kind) {
            Ok(settings) => {
                kgw_start_trace_emit_v1(
                    "native",
                    "native.payload_received_and_validated",
                    settings.network.as_str(),
                    "start",
                    "validated",
                    Some("{\"schema\":\"node-settings-v1\"}"),
                );
                settings
            }
            Err(error) => {
                let error_text = error.to_string();
                kgw_start_trace_emit_v1(
                    "native",
                    "native.payload_received_and_validated",
                    "unknown",
                    "start",
                    "error",
                    Some("{\"schema\":\"node-settings-v1\"}"),
                );
                kgw_start_trace_emit_v1(
                    "native",
                    "native.startup_response_returned",
                    "unknown",
                    "start",
                    "error",
                    Some("{\"reason\":\"payload-validation-failed\"}"),
                );
                return Err(error_text);
            }
        };

    kgw_start_trace_emit_v1(
        "native",
        "native.network_normalized",
        settings.network.as_str(),
        "start",
        "ok",
        Some(&format!(
            "{{\"normalizedNetwork\":\"{}\"}}",
            settings.network.as_str()
        )),
    );

    let experimental_network = settings.network == kaspa_gateway_rk_node::KgwNetwork::Testnet12;
    let experimental_allowed = !experimental_network || experimental_network_opt_in == Some(true);
    kgw_start_trace_emit_v1(
        "native",
        "native.experimental_opt_in_evaluated",
        settings.network.as_str(),
        "start",
        if experimental_allowed {
            "allowed"
        } else {
            "blocked"
        },
        Some(&format!(
            "{{\"experimentalNetwork\":{},\"explicitOptIn\":{}}}",
            experimental_network,
            experimental_network_opt_in == Some(true)
        )),
    );

    if let Err(error) =
        kgw_validate_network_start_policy(settings.network, experimental_network_opt_in)
    {
        kgw_start_trace_emit_v1(
            "native",
            "native.startup_response_returned",
            settings.network.as_str(),
            "start",
            "error",
            Some("{\"reason\":\"experimental-network-policy\"}"),
        );
        return Err(error);
    }

    let bridge_config_path_for_worker =
        kgw_bridge_config_path_from_preview_r122(bridge_command_preview.as_deref());

    let bridge_instance_listens_from_preview =
        kgw_bridge_command_preview_instance_listens_r123(bridge_command_preview.as_deref());

    let bridge_instance_listens_override = if bridge_instance_listens_from_preview.is_empty() {
        None
    } else {
        Some(bridge_instance_listens_from_preview)
    };

    kgw_apply_command_preview_overrides(
        &mut settings,
        node_command_preview,
        bridge_command_preview,
    );
    settings.app_dir_name = kgw_network_runtime_appdir(settings.network);

    let bridge_active_instance_id_for_worker = bridge_active_instance_id.clone();
    let bridge_structured_instances_for_worker = bridge_structured_instances.clone();

    kgw_apply_bridge_active_instance_runtime_overrides_r110f(
        &mut settings,
        bridge_active_instance_id,
        bridge_active_instance,
        bridge_active_instance_port,
        bridge_structured_instances,
    );

    let role = kgw_worker_role_from_request(runtime_role.as_deref(), &settings);

    let result = kgw_worker_start(
        &role,
        &settings,
        bridge_active_instance_id_for_worker,
        bridge_structured_instances_for_worker,
        bridge_config_path_for_worker,
        bridge_instance_listens_override,
    );

    kgw_start_trace_emit_v1(
        "native",
        "native.startup_response_returned",
        settings.network.as_str(),
        "start",
        if result.is_ok() { "ok" } else { "error" },
        Some(&format!("{{\"runtimeRole\":\"{}\"}}", role)),
    );

    result
}

#[tauri::command]
pub fn kgw_kgw_disable_network_v1(
    network: String,
    runtime_role: Option<String>,
) -> Result<String, String> {
    if let Some(stopped) = kgw_worker_stop(&network, runtime_role.as_deref())? {
        return Ok(stopped);
    }

    let network =
        kaspa_gateway_rk_node::KgwNetwork::parse(&network).map_err(|error| error.to_string())?;
    let role = runtime_role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("all")
        .to_ascii_lowercase();
    Ok(format!(
        "parallel-owned-self-worker already stopped;role={role};network={};running=false;graceful=false;forced=false;already_stopped=true;evidence=no-live-same-exe-worker",
        network.as_str()
    ))
}

#[tauri::command]
pub fn kgw_runtime_owner_plan_v1(network: Option<String>) -> Result<String, String> {
    let network = network.unwrap_or_else(|| "mainnet".to_string());

    kaspa_gateway_rk_node::runtime_owner_plan_for_network_v1(Some(network))
        .map(|plan| plan.to_log_line())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_all_parallel_node_bridge_plans_v1() -> Result<String, String> {
    kaspa_gateway_rk_node::all_parallel_runtime_plans_log_v1().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn kgw_kgw_node_bridge_service_plan_v1(
    network: String,
    node_mode: String,
    bridge_mode: String,
    explicit_runtime_opt_in: bool,
) -> Result<String, String> {
    let node_mode = kaspa_gateway_rk_node::KaspaNodeRuntimeMode::parse(&node_mode)
        .map_err(|error| error.to_string())?;

    let bridge_mode = kaspa_gateway_rk_node::KaspaBridgeRuntimeMode::parse(&bridge_mode)
        .map_err(|error| error.to_string())?;

    let settings = kaspa_gateway_rk_node::KaspaRuntimeSettings {
        network,
        node_mode,
        bridge_mode,
        explicit_runtime_opt_in,
        ..kaspa_gateway_rk_node::KaspaRuntimeSettings::default()
    };

    let plan = kaspa_gateway_rk_node::build_official_kaspa_runtime_plan_v1(settings)
        .map_err(|error| error.to_string())?;

    Ok(plan.to_log_line())
}

#[tauri::command]
pub fn kgw_kgw_real_owner_summary_v1() -> Result<String, String> {
    Ok(kaspa_gateway_rk_node::real_owner_runtime_summary_v1().to_string())
}

#[tauri::command]
pub fn kgw_kgw_real_owner_feature_status_v1(network: String) -> Result<String, String> {
    let network =
        kaspa_gateway_rk_node::KgwNetwork::parse(&network).map_err(|error| error.to_string())?;

    let feature_status = kaspa_gateway_rk_node::KgwRuntimeFeatureStatus::for_network(network);

    Ok(feature_status.to_log_line())
}

#[tauri::command]
pub fn kgw_kgw_smoke_start_network_v1(network: String, bridge: bool) -> Result<String, String> {
    let bridge_kind = if bridge {
        "official-inprocess-node"
    } else {
        "disable"
    };

    let settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        network.clone(),
        "integrated-inproc".to_string(),
        bridge_kind.to_string(),
    )
    .map_err(|error| error.to_string())?;

    let accepted = controller()
        .apply_node_settings(settings)
        .map_err(|error| error.to_string())?;

    let status = controller()
        .status(Some(
            kaspa_gateway_rk_node::KgwNetwork::parse(&network)
                .map_err(|error| error.to_string())?,
        ))
        .map_err(|error| error.to_string())?;

    let logs = controller()
        .logs(Some(
            kaspa_gateway_rk_node::KgwNetwork::parse(&network)
                .map_err(|error| error.to_string())?,
        ))
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "smoke_start_network accepted\n{}\nstatus={}\nlogs=\n{}",
        accepted, status, logs
    ))
}

pub(crate) fn kgw_live_smoke_parent_start_v1(
    network: String,
    appdir: String,
    rpc: String,
    p2p_listen: Option<String>,
) -> Result<String, String> {
    let settings =
        kgw_validate_live_smoke_parent_settings_v1(&network, &appdir, &rpc, p2p_listen.as_deref())?;
    kgw_worker_start("node", &settings, None, None, None, None)
}

#[tauri::command]
pub fn kgw_kgw_smoke_stop_network_v1(network: String) -> Result<String, String> {
    let parsed_network =
        kaspa_gateway_rk_node::KgwNetwork::parse(&network).map_err(|error| error.to_string())?;

    let accepted = if let Some(stopped) = kgw_worker_stop(&network, None)? {
        stopped
    } else {
        controller()
            .disable_network(parsed_network)
            .map_err(|error| error.to_string())?
    };

    let status = controller()
        .status(Some(parsed_network))
        .map_err(|error| error.to_string())?;

    let logs = controller()
        .logs(Some(parsed_network))
        .map_err(|error| error.to_string())?;

    Ok(format!(
        "smoke_stop_network accepted\n{}\nstatus={}\nlogs=\n{}",
        accepted, status, logs
    ))
}

#[cfg(test)]
#[test]
fn kgw_test_self_worker_hold() {
    if std::env::var_os("KGW_TEST_SELF_WORKER_CHILD").is_none() {
        return;
    }

    let role = std::env::var("KGW_TEST_SELF_WORKER_ROLE").unwrap_or_else(|_| "node".to_string());
    let network =
        std::env::var("KGW_TEST_SELF_WORKER_NETWORK").unwrap_or_else(|_| "mainnet".to_string());

    let stdout = std::env::var("KGW_TEST_SELF_WORKER_STDOUT")
        .unwrap_or_else(|_| format!("test-self-worker stdout role={role} network={network}"));
    let stderr = std::env::var("KGW_TEST_SELF_WORKER_STDERR")
        .unwrap_or_else(|_| format!("test-self-worker stderr role={role} network={network}"));

    println!("{stdout}");
    eprintln!("{stderr}");

    let control_path = std::env::var("KGW_TEST_SELF_WORKER_CONTROL_PATH")
        .expect("test self-worker control path must be set");
    let stop_request_path = std::env::var("KGW_TEST_SELF_WORKER_STOP_REQUEST_PATH")
        .expect("test self-worker stop request path must be set");
    let stop_outcome_path = std::env::var("KGW_TEST_SELF_WORKER_STOP_OUTCOME_PATH")
        .expect("test self-worker stop outcome path must be set");
    let parent_identity = KgwProcessIdentityV1 {
        pid: std::env::var("KGW_TEST_SELF_WORKER_PARENT_PID")
            .expect("test self-worker parent PID must be set")
            .parse()
            .expect("test self-worker parent PID must be valid"),
        start_time: std::env::var("KGW_TEST_SELF_WORKER_PARENT_START_TIME")
            .expect("test self-worker parent start time must be set")
            .parse()
            .expect("test self-worker parent start time must be valid"),
        executable: std::env::var("KGW_TEST_SELF_WORKER_PARENT_EXECUTABLE")
            .expect("test self-worker parent executable must be set"),
    };
    let delay_ms = std::env::var("KGW_TEST_SELF_WORKER_READY_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_default();
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    let evidence = format!(
        "{{\"version\":1,\"outcome\":\"READY\",\"runtimeRole\":\"{}\",\"network\":\"{}\",\"evidence\":\"test-role-ready\",\"error\":null}}",
        role, network
    );
    let evidence: serde_json::Value =
        serde_json::from_str(&evidence).expect("test READY must be typed JSON");
    kgw_worker_atomic_write_json_v1(std::path::Path::new(&control_path), &evidence)
        .expect("test self-worker must publish READY");
    if let Some(exit_after_ready_ms) = std::env::var("KGW_TEST_SELF_WORKER_EXIT_AFTER_READY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
    {
        std::thread::sleep(std::time::Duration::from_millis(exit_after_ready_ms));
        eprintln!("test-self-worker official runtime terminated after READY");
        std::process::exit(17);
    }

    loop {
        let parent_alive =
            kgw_process_identity_for_worker_v1(parent_identity.pid).is_ok_and(|identity| {
                identity.start_time == parent_identity.start_time
                    && identity.executable == parent_identity.executable
            });
        if !parent_alive {
            println!("test-self-worker parent lost; official shutdown joined");
            eprintln!("test-self-worker parent lost stderr");
            return;
        }
        if std::path::Path::new(&stop_request_path).is_file() {
            let request = std::fs::read_to_string(&stop_request_path)
                .expect("test self-worker must read Stop request");
            let request: serde_json::Value =
                serde_json::from_str(&request).expect("test Stop request must be typed JSON");
            assert_eq!(request["version"], 1);
            assert_eq!(request["command"], "STOP");
            assert_eq!(request["runtimeRole"], role);
            assert_eq!(request["network"], network);
            assert_eq!(
                request["workerPid"].as_u64(),
                Some(u64::from(std::process::id()))
            );
            let _ = std::fs::remove_file(&stop_request_path);
            if std::env::var_os("KGW_TEST_SELF_WORKER_HANG_ON_STOP").is_none() {
                println!("test-self-worker final official stdout");
                eprintln!("test-self-worker final official stderr");
                if std::env::var_os("KGW_TEST_SELF_WORKER_TIMEOUT_ON_STOP").is_some() {
                    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::sync_channel(1);
                    let _shutdown_thread = std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(60));
                        let _ = shutdown_tx.send(());
                    });
                    assert!(matches!(
                        shutdown_rx.recv_timeout(std::time::Duration::from_millis(5)),
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                    ));
                    let outcome = serde_json::json!({
                        "version": 1,
                        "outcome": "FAILED",
                        "runtimeRole": role,
                        "network": network,
                        "workerPid": std::process::id(),
                        "allOwnedComponentsTerminal": false,
                        "evidence": null,
                        "error": "official shutdown exceeded child budget;timeout_ms=5",
                    });
                    std::fs::write(&stop_outcome_path, outcome.to_string())
                        .expect("test self-worker must publish timeout FAILED");
                    loop {
                        std::thread::park_timeout(std::time::Duration::from_secs(60));
                    }
                }
                if std::env::var_os("KGW_TEST_SELF_WORKER_BRIDGE_LISTENER_FAIL_ON_STOP").is_some() {
                    let marker_path = std::env::var_os(
                        "KGW_TEST_SELF_WORKER_OWNED_NODE_STOP_MARKER_PATH",
                    )
                    .expect("in-process bridge fixture must provide owned Node Stop marker path");
                    std::fs::write(marker_path, "owned Node graceful Stop attempted")
                        .expect("in-process bridge fixture must mark owned Node Stop attempt");
                    let outcome = serde_json::json!({
                        "version": 1,
                        "outcome": "FAILED",
                        "runtimeRole": role,
                        "network": network,
                        "workerPid": std::process::id(),
                        "allOwnedComponentsTerminal": false,
                        "evidence": null,
                        "error": "component=bridge-listener-0;listener terminality not proven after join failure",
                    });
                    std::fs::write(&stop_outcome_path, outcome.to_string())
                        .expect("in-process bridge fixture must publish nonterminal FAILED");
                    loop {
                        std::thread::park_timeout(std::time::Duration::from_secs(60));
                    }
                }
                let fail_on_stop = std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_ON_STOP").is_some();
                let outcome = serde_json::json!({
                    "version": 1,
                    "outcome": if fail_on_stop { "FAILED" } else { "STOPPED" },
                    "runtimeRole": role,
                    "network": network,
                    "workerPid": std::process::id(),
                    "allOwnedComponentsTerminal": true,
                    "evidence": if fail_on_stop { None } else { Some("test-official-shutdown-joined") },
                    "error": if fail_on_stop { Some("test official shutdown failure") } else { None },
                });
                std::fs::write(stop_outcome_path, outcome.to_string())
                    .expect("test self-worker must publish STOPPED");
                return;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(test)]
#[test]
fn kgw_test_self_worker_fail() {
    if std::env::var_os("KGW_TEST_SELF_WORKER_FAIL_CHILD").is_none() {
        return;
    }

    let role = std::env::var("KGW_TEST_SELF_WORKER_ROLE").unwrap_or_else(|_| "node".to_string());
    let network =
        std::env::var("KGW_TEST_SELF_WORKER_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    let line = std::env::var("KGW_TEST_SELF_WORKER_FAIL_STDERR").unwrap_or_else(|_| {
        format!("test-self-worker forced failure role={role} network={network}")
    });

    println!("test-self-worker failing stdout role={role} network={network}");
    eprintln!("{line}");
    std::process::exit(1);
}

#[cfg(test)]
#[test]
fn kgw_test_self_worker_delayed_fail() {
    if std::env::var_os("KGW_TEST_SELF_WORKER_DELAYED_FAIL_CHILD").is_none() {
        return;
    }

    let role = std::env::var("KGW_TEST_SELF_WORKER_ROLE").unwrap_or_else(|_| "node".to_string());
    let network =
        std::env::var("KGW_TEST_SELF_WORKER_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    let line = std::env::var("KGW_TEST_SELF_WORKER_DELAYED_FAIL_STDERR")
        .unwrap_or_else(|_| "delayed startup failure".to_string());
    let control_path = std::env::var("KGW_TEST_SELF_WORKER_CONTROL_PATH")
        .expect("test self-worker control path must be set");

    std::thread::sleep(std::time::Duration::from_millis(1_100));
    let message = serde_json::json!({
        "version": 1,
        "outcome": "FAILED",
        "runtimeRole": role,
        "network": network,
        "evidence": null,
        "error": line,
    });
    kgw_worker_atomic_write_json_v1(std::path::Path::new(&control_path), &message)
        .expect("test self-worker must publish FAILED");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
