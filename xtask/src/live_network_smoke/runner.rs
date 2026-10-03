#[cfg(test)]
use super::Network;
use super::{
    Options, Profile, Result, Snapshot, failed_result, report, round_tenth, successful_result, tcp,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Clone, Copy)]
struct Timing {
    ready: Duration,
    observation: Duration,
    poll: Duration,
    probe: Duration,
    orphan: Duration,
    release: Duration,
}

struct OwnedChild {
    child: Child,
}
impl OwnedChild {
    fn exited(&mut self) -> Result<Option<std::process::ExitStatus>> {
        self.child
            .try_wait()
            .map_err(|error| format!("Observe owned child {}: {error}", self.child.id()))
    }
    fn terminate(&mut self) -> Result<()> {
        if self.exited()?.is_none()
            && let Err(error) = self.child.kill()
            && self.exited()?.is_none()
        {
            return Err(format!("Stop owned child {}: {error}", self.child.id()));
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        while self.exited()?.is_none() {
            if Instant::now() >= deadline {
                return Err(format!(
                    "Owned child {} did not exit within its stop deadline",
                    self.child.id()
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Err(error) = self.terminate() {
            eprintln!("Owned smoke child cleanup failed: {error}");
        }
    }
}

fn hidden(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    #[cfg(not(windows))]
    let _ = command;
}
fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|error| format!("Read {}: {error}", path.display()))?;
    let mut buffer = [0_u8; 65536];
    let mut hash = Sha256::new();
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
fn parent_command(binary: &Path, profile: &Profile) -> Command {
    let mut command = Command::new(binary);
    command
        .arg("--kgw-live-smoke-parent")
        .args(["--network", profile.network.name()])
        .arg("--appdir")
        .arg(&profile.app_dir)
        .args(["--rpc", &format!("127.0.0.1:{}", profile.port)]);
    hidden(&mut command);
    command
}
fn probe_command(binary: &Path, profile: &Profile) -> Command {
    let mut command = Command::new(binary);
    command.args([
        "--rpc",
        &profile.endpoint(),
        "--expect-network",
        profile.network.name(),
    ]);
    hidden(&mut command);
    command
}
fn lifecycle(report_dir: &Path, event: &str, profile: &Profile, detail: Value) -> Result<()> {
    let path = report_dir.join("lifecycle.jsonl");
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    let row = json!({"event":event,"network":profile.network.name(),"rpc_port":profile.port,
        "utc":OffsetDateTime::now_utc().format(&Rfc3339).map_err(|error|error.to_string())?,"detail":detail});
    writeln!(output, "{row}")
        .and_then(|()| output.sync_all())
        .map_err(|error| error.to_string())
}
fn start_parent(
    binary: &Path,
    expected_hash: &str,
    profile: &Profile,
    report_dir: &Path,
    relaunch: bool,
) -> Result<OwnedChild> {
    if hash_file(binary)? != expected_hash {
        return Err("Desktop executable identity changed before launch".to_owned());
    }
    let stdout = if relaunch {
        report_dir.join(format!("{}.relaunch.stdout.log", profile.network.name()))
    } else {
        profile.stdout.clone()
    };
    let stderr = if relaunch {
        report_dir.join(format!("{}.relaunch.stderr.log", profile.network.name()))
    } else {
        profile.stderr.clone()
    };
    let out = File::create(stdout).map_err(|error| error.to_string())?;
    let err = File::create(stderr).map_err(|error| error.to_string())?;
    lifecycle(
        report_dir,
        "START_INTENT",
        profile,
        json!({"executable":binary,"sha256":expected_hash,"relaunch":relaunch}),
    )?;
    let mut command = parent_command(binary, profile);
    command.stdout(Stdio::from(out)).stderr(Stdio::from(err));
    let child = command
        .spawn()
        .map_err(|error| format!("Start exact smoke parent: {error}"))?;
    let owned = OwnedChild { child };
    lifecycle(
        report_dir,
        "STARTED",
        profile,
        json!({"pid":owned.child.id(),"relaunch":relaunch}),
    )?;
    Ok(owned)
}
fn invoke_probe(
    binary: &Path,
    expected_hash: &str,
    profile: &Profile,
    report_dir: &Path,
    timeout: Duration,
) -> Result<Snapshot> {
    if hash_file(binary)? != expected_hash {
        return Err("RPC probe executable identity changed before invocation".to_owned());
    }
    let mut output = tempfile::Builder::new()
        .prefix("probe-")
        .suffix(".log")
        .tempfile_in(report_dir)
        .map_err(|error| error.to_string())?;
    let mut command = probe_command(binary, profile);
    command
        .stdout(Stdio::from(
            output
                .as_file()
                .try_clone()
                .map_err(|error| error.to_string())?,
        ))
        .stderr(Stdio::from(
            output
                .as_file()
                .try_clone()
                .map_err(|error| error.to_string())?,
        ));
    let child = command
        .spawn()
        .map_err(|error| format!("Start exact RPC probe: {error}"))?;
    let mut owned = OwnedChild { child };
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = owned.exited()? {
            break status;
        }
        if Instant::now() >= deadline {
            owned.terminate()?;
            let path = output
                .into_temp_path()
                .keep()
                .map_err(|error| error.to_string())?;
            return Err(format!(
                "RPC probe exceeded its bounded deadline; diagnostic={}",
                path.display()
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    output
        .flush()
        .and_then(|()| output.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    let file = File::open(output.path()).map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("RPC probe output exceeds the bounded evidence limit".to_owned());
    }
    let text = String::from_utf8(bytes)
        .map_err(|error| format!("RPC probe output is not UTF8: {error}"))?;
    if !status.success() {
        let path = output
            .into_temp_path()
            .keep()
            .map_err(|error| error.to_string())?;
        return Err(format!(
            "RPC probe failed: {status}; diagnostic={}; {}",
            path.display(),
            text.chars().take(8192).collect::<String>()
        ));
    }
    let line = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .ok_or("RPC probe returned no JSON result")?;
    let value: Value =
        serde_json::from_str(line).map_err(|error| format!("Invalid RPC probe JSON: {error}"))?;
    Snapshot::parse(&value, profile.network)
}
fn wait_ready(
    parent: &mut OwnedChild,
    probe: &Path,
    probe_hash: &str,
    profile: &Profile,
    report_dir: &Path,
    timing: Timing,
) -> Result<Snapshot> {
    let deadline = Instant::now() + timing.ready;
    let mut last_error = String::new();
    while Instant::now() < deadline {
        if let Some(status) = parent.exited()? {
            return Err(format!(
                "Node process exited before RPC became ready. exit_code={status}"
            ));
        }
        match invoke_probe(
            probe,
            probe_hash,
            profile,
            report_dir,
            timing
                .probe
                .min(deadline.saturating_duration_since(Instant::now())),
        ) {
            Ok(value) => return Ok(value),
            Err(error) => last_error = error,
        }
        std::thread::sleep(
            timing
                .poll
                .min(deadline.saturating_duration_since(Instant::now())),
        );
    }
    Err(format!(
        "RPC did not become ready within {} seconds. last_error={last_error}",
        timing.ready.as_secs()
    ))
}
fn wait_port_release(port: u16, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if !tcp::listening(port)? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "Same-executable worker retained RPC port {port} after its exact parent exited"
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn one_network(
    desktop: &Path,
    probe: &Path,
    desktop_hash: &str,
    probe_hash: &str,
    profile: &Profile,
    report_dir: &Path,
    timing: Timing,
) -> Value {
    let started = Instant::now();
    let mut parent = None;
    let outcome = (|| -> Result<Value> {
        if tcp::listening(profile.port)? {
            return Err(format!(
                "RPC port {} for {} is already in use; no process was stopped",
                profile.port,
                profile.network.name()
            ));
        }
        parent = Some(start_parent(
            desktop,
            desktop_hash,
            profile,
            report_dir,
            false,
        )?);
        let first = wait_ready(
            parent.as_mut().expect("owned parent"),
            probe,
            probe_hash,
            profile,
            report_dir,
            timing,
        )?;
        std::thread::sleep(timing.observation);
        if let Some(status) = parent.as_mut().expect("owned parent").exited()? {
            return Err(format!(
                "Node process exited during the observation window. exit_code={status}"
            ));
        }
        let second = invoke_probe(probe, probe_hash, profile, report_dir, timing.probe)?;
        let mut owned = parent.take().expect("owned parent");
        let pid = owned.child.id();
        lifecycle(
            report_dir,
            "PARENT_STOP_INTENT",
            profile,
            json!({"pid":pid}),
        )?;
        owned.terminate()?;
        drop(owned);
        wait_port_release(profile.port, timing.orphan)?;
        lifecycle(
            report_dir,
            "PARENT_LOSS_PORT_RELEASED",
            profile,
            json!({"pid":pid}),
        )?;
        parent = Some(start_parent(
            desktop,
            desktop_hash,
            profile,
            report_dir,
            true,
        )?);
        let relaunch = wait_ready(
            parent.as_mut().expect("relaunched parent"),
            probe,
            probe_hash,
            profile,
            report_dir,
            timing,
        )?;
        Ok(successful_result(
            profile,
            &first,
            &second,
            &relaunch,
            started.elapsed().as_secs_f64(),
        ))
    })();
    let mut result = match outcome {
        Ok(value) => value,
        Err(error) => {
            let alive = parent
                .as_mut()
                .is_some_and(|owned| owned.exited().ok() == Some(None));
            failed_result(profile, alive, started.elapsed().as_secs_f64(), &error)
        }
    };
    let cleanup = (|| -> Result<()> {
        if let Some(mut owned) = parent.take() {
            lifecycle(
                report_dir,
                "FINAL_STOP_INTENT",
                profile,
                json!({"pid":owned.child.id()}),
            )?;
            owned.terminate()?;
            drop(owned);
        }
        wait_port_release(profile.port, timing.release)
    })();
    if let Err(error) = cleanup {
        result["Success"] = json!(false);
        let original = result.get("Failure").and_then(Value::as_str).unwrap_or("");
        result["Failure"] = json!(format!("{original} Final owned cleanup failed: {error}").trim());
    }
    if let Err(error) = lifecycle(report_dir, "NETWORK_COMPLETE", profile, result.clone()) {
        result["Success"] = json!(false);
        result["Failure"] = json!(format!("Cannot preserve lifecycle evidence: {error}"));
    }
    result
}

fn build_commands() -> [(Vec<&'static str>, &'static str); 2] {
    [
        (
            vec![
                "build",
                "--locked",
                "-p",
                "kaspa-gateway-desktop",
                "--release",
                "--features",
                "official-kaspa-runtime-mainline",
            ],
            "desktop-build.log",
        ),
        (
            vec![
                "build",
                "--locked",
                "-p",
                "kaspa-gateway-cli",
                "--release",
                "--bin",
                "kgw-live-probe",
                "--features",
                "live-network-probe",
            ],
            "probe-build.log",
        ),
    ]
}
fn find_program(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|directory| directory.join(name))
        .find(|path| path.is_file())
}
fn prepared_build_environment() -> Result<Vec<(&'static str, PathBuf)>> {
    let rust = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|error| format!("Rust is required: {error}"))?;
    if !rust.status.success() {
        return Err(
            "rustc --version failed; prepare the repository-pinned toolchain explicitly".to_owned(),
        );
    }
    let version = String::from_utf8(rust.stdout).map_err(|error| error.to_string())?;
    let raw = version
        .split_whitespace()
        .nth(1)
        .ok_or("Cannot parse rustc version")?;
    let numbers = raw
        .split('.')
        .take(3)
        .map(|value| value.split('-').next().unwrap_or("").parse::<u64>())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| "Cannot parse rustc version".to_owned())?;
    if numbers.as_slice() < [1, 97, 1].as_slice() {
        return Err("Repository MSRV1.97.1 or newer is required; no automatic toolchain upgrade is performed".to_owned());
    }
    let protoc=std::env::var_os("PROTOC").map(PathBuf::from).filter(|path|path.is_file()).or_else(||find_program("protoc.exe"))
        .ok_or("Prepare Protobuf/protoc explicitly before a smoke build, or use --skip-build with qualified existing binaries")?;
    let llvm = std::env::var_os("LIBCLANG_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("ProgramFiles").map(|base| PathBuf::from(base).join("LLVM/bin"))
        })
        .ok_or("Prepare LLVM/libclang explicitly before a smoke build")?;
    if !llvm.join("libclang.dll").is_file() {
        return Err(format!(
            "Missing {}; no WinGet or system mutation is performed",
            llvm.join("libclang.dll").display()
        ));
    }
    let mut values = vec![("PROTOC", protoc), ("LIBCLANG_PATH", llvm.clone())];
    if llvm.join("llvm-ar.exe").is_file() {
        values.push(("AR", llvm.join("llvm-ar.exe")));
    }
    Ok(values)
}
fn preflight(root: &Path) -> Result<Value> {
    let mut system = sysinfo::System::new();
    system.refresh_cpu_all();
    system.refresh_memory();
    let cores = system.cpus().len();
    let memory = round_tenth(system.total_memory() as f64 / 1073741824.0);
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let root_text = root.to_string_lossy().to_ascii_lowercase();
    let disk = disks
        .list()
        .iter()
        .filter(|disk| {
            root_text.starts_with(&disk.mount_point().to_string_lossy().to_ascii_lowercase())
        })
        .max_by_key(|disk| disk.mount_point().as_os_str().len())
        .ok_or("Cannot identify the repository's disk for preflight")?;
    let free = round_tenth(disk.available_space() as f64 / 1073741824.0);
    Ok(
        json!({"LogicalCores":cores,"MemoryGB":memory,"FreeDiskGB":free,
        "MeetsProductionCoreMinimum":cores>=8,"MeetsProductionMemoryMinimum":memory>=16.0,"MeetsProductionDiskMinimum":free>=640.0,
        "FullInitialBlockDownloadRequested":false,"MainnetTransactionsOrMiningRequested":false}),
    )
}
fn save_report(path: &Path, value: &Value) -> Result<()> {
    let directory = path.parent().ok_or("Report path has no parent")?;
    let mut temp = tempfile::Builder::new()
        .prefix(".smoke-report-")
        .tempfile_in(directory)
        .map_err(|error| error.to_string())?;
    serde_json::to_writer_pretty(&mut temp, value).map_err(|error| error.to_string())?;
    temp.write_all(b"\n")
        .and_then(|()| temp.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    temp.persist(path).map_err(|error| error.to_string())?;
    let saved: Value = serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    if &saved != value {
        return Err("Smoke report read-back mismatch".to_owned());
    }
    Ok(())
}

pub(super) fn run(options: Options) -> Result<()> {
    let root = fs::canonicalize(&options.repository).map_err(|error| error.to_string())?;
    if !root.join("Cargo.toml").is_file() {
        return Err("The smoke repository has no Cargo.toml".to_owned());
    }
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or("LOCALAPPDATA is required for network-isolated Node directories")?;
    let desktop = root.join("target/release/kaspa-gateway-desktop.exe");
    let probe = root.join("target/release/kgw-live-probe.exe");
    if options.plan_only {
        let plans=options.networks.iter().map(|network|{
            let profile=Profile{network:*network,port:network.port(),app_dir:local.join("KaspaGateway/nodes").join(network.name()),stdout:PathBuf::new(),stderr:PathBuf::new()};
            let command=parent_command(&desktop,&profile);
            json!({"network":network.name(),"rpc":profile.endpoint(),"executable":command.get_program().to_string_lossy(),
                "args":command.get_args().map(|arg|arg.to_string_lossy().into_owned()).collect::<Vec<_>>()})
        }).collect::<Vec<_>>();
        println!(
            "{}",
            json!({"operation":"PLAN_ONLY","runtime_started":false,"networks":plans,
            "ready_timeout_seconds":options.ready_timeout.as_secs(),"observation_seconds":options.observation.as_secs(),
            "skip_build":options.skip_build,"build_commands":build_commands().map(|(args,_)|args),
            "bootstrap_policy":"REQUIRE_EXPLICITLY_PREPARED_PREREQUISITES"})
        );
        return Ok(());
    }
    if !cfg!(windows) {
        return Err("The live embedded-runtime smoke test must run on Windows".to_owned());
    }
    for network in &options.networks {
        if tcp::listening(network.port())? {
            return Err(format!(
                "RPC port {} for {} is already in use; no process or service is stopped",
                network.port(),
                network.name()
            ));
        }
    }
    let preflight = preflight(&root)?;
    if preflight["MeetsProductionDiskMinimum"] == false {
        eprintln!(
            "Less than640GiB is free; do not use this short smoke as proof of full production synchronization capacity."
        );
    }
    let artifact_parent = root.join("artifacts/live-network-smoke");
    fs::create_dir_all(&artifact_parent).map_err(|error| error.to_string())?;
    let report_dir = tempfile::Builder::new()
        .prefix("native-")
        .tempdir_in(&artifact_parent)
        .map_err(|error| error.to_string())?
        .keep();
    println!("SMOKE_ARTIFACT_DIRECTORY={}", report_dir.display());
    if !options.skip_build {
        let environment = prepared_build_environment()?;
        for (arguments, logname) in build_commands() {
            let file = File::create(report_dir.join(logname)).map_err(|error| error.to_string())?;
            let status = Command::new("cargo")
                .args(arguments)
                .current_dir(&root)
                .envs(environment.iter().map(|(key, value)| (*key, value)))
                .stdout(Stdio::from(
                    file.try_clone().map_err(|error| error.to_string())?,
                ))
                .stderr(Stdio::from(file))
                .status()
                .map_err(|error| error.to_string())?;
            if !status.success() {
                return Err(format!(
                    "Smoke build failed: {status}; see {}",
                    report_dir.join(logname).display()
                ));
            }
        }
    }
    for binary in [&desktop, &probe] {
        let metadata = fs::symlink_metadata(binary)
            .map_err(|error| format!("Required executable {}: {error}", binary.display()))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(format!(
                "Smoke executable must be a regular owned file: {}",
                binary.display()
            ));
        }
    }
    let desktop_hash = hash_file(&desktop)?;
    let probe_hash = hash_file(&probe)?;
    let timing = Timing {
        ready: options.ready_timeout,
        observation: options.observation,
        poll: Duration::from_secs(5),
        probe: Duration::from_secs(20),
        orphan: Duration::from_secs(60),
        release: Duration::from_secs(20),
    };
    let mut results = Vec::new();
    for network in &options.networks {
        let profile = Profile {
            network: *network,
            port: network.port(),
            app_dir: local.join("KaspaGateway/nodes").join(network.name()),
            stdout: report_dir.join(format!("{}.stdout.log", network.name())),
            stderr: report_dir.join(format!("{}.stderr.log", network.name())),
        };
        results.push(one_network(
            &desktop,
            &probe,
            &desktop_hash,
            &probe_hash,
            &profile,
            &report_dir,
            timing,
        ));
        let current = report(
            &options.networks,
            preflight.clone(),
            &results,
            &OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|error| error.to_string())?,
        );
        save_report(&report_dir.join("report.json"), &current)?;
    }
    let passed = results.iter().all(|result| result["Success"] == true);
    if !passed {
        return Err(format!(
            "One or more live network smoke tests failed. Review {} and captured logs",
            report_dir.join("report.json").display()
        ));
    }
    println!("LIVE_NETWORK_SMOKE=PASS; FULL_SYNC_OR_PRODUCTION_QUALIFICATION=NOT_CLAIMED");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parent_and_probe_arguments_keep_same_executable_contract_and_path_boundaries() {
        let profile = Profile {
            network: Network::Testnet10,
            port: 16210,
            app_dir: PathBuf::from("folder with spaces/data"),
            stdout: PathBuf::new(),
            stderr: PathBuf::new(),
        };
        let command = parent_command(Path::new("owned/desktop.exe"), &profile);
        let args = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            args,
            vec![
                "--kgw-live-smoke-parent",
                "--network",
                "testnet10",
                "--appdir",
                "folder with spaces/data",
                "--rpc",
                "127.0.0.1:16210"
            ]
        );
        assert!(
            !args
                .iter()
                .any(|arg| arg == "--kgw-self-worker" || arg.contains("testnet13"))
        );
        assert_eq!(
            probe_command(Path::new("owned/probe.exe"), &profile)
                .get_args()
                .map(|value| value.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec![
                "--rpc",
                "grpc://127.0.0.1:16210",
                "--expect-network",
                "testnet10"
            ]
        );
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "Requires explicit external preflight output path"]
    fn explicit_preflight_output() {
        let repository = std::env::var_os("KGW_SMOKE_PREFLIGHT_REPOSITORY")
            .expect("explicit preflight repository");
        let output =
            std::env::var_os("KGW_SMOKE_PREFLIGHT_RESULT").expect("explicit preflight result path");
        let value = preflight(Path::new(&repository)).unwrap();
        fs::write(output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    #[test]
    fn build_profiles_remain_stable_and_locked() {
        let commands = build_commands();
        assert!(commands[0].0.contains(&"--locked"));
        assert!(commands[1].0.contains(&"--locked"));
        assert!(commands[0].0.contains(&"official-kaspa-runtime-mainline"));
        assert!(commands[1].0.contains(&"live-network-probe"));
        assert!(
            !commands
                .iter()
                .flat_map(|(args, _)| args)
                .any(|arg| arg.contains("testnet13"))
        );
    }
    #[test]
    fn report_write_is_utf8_and_roundtrips_exact_u64_values() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("report.json");
        let value = json!({"counter":u64::MAX,"text":"\u{0645}\u{0631}\u{062d}\u{0628}\u{0627}"});
        save_report(&path, &value).unwrap();
        assert!(!fs::read(&path).unwrap().starts_with(&[0xef, 0xbb, 0xbf]));
        save_report(&path, &json!({"new":true})).unwrap();
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }
    #[cfg(windows)]
    #[test]
    fn actual_owned_fixture_lifecycle_preserves_parent_loss_relaunch_and_failures() {
        let root = tempfile::tempdir().unwrap();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/live_smoke_child.rs");
        let binary = root.path().join("scoped-live-smoke-fixture.exe");
        let compile = Command::new("rustc")
            .args(["--edition=2021", "--crate-name", "kgw_scoped_live_fixture"])
            .arg(source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "fixture compiler: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let hash = hash_file(&binary).unwrap();
        let timing = Timing {
            ready: Duration::from_millis(750),
            observation: Duration::from_millis(10),
            poll: Duration::from_millis(10),
            probe: Duration::from_millis(350),
            orphan: Duration::from_millis(750),
            release: Duration::from_millis(750),
        };
        for scenario in [
            "healthy",
            "stagnant",
            "no-peers",
            "wrong-network",
            "early-exit",
            "probe-timeout",
        ] {
            let directory = root.path().join(scenario);
            fs::create_dir(&directory).unwrap();
            let app = directory.join("isolated data");
            fs::create_dir(&app).unwrap();
            fs::write(app.join("fixture-scenario.txt"), scenario).unwrap();
            let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener);
            let profile = Profile {
                network: Network::Testnet10,
                port,
                app_dir: app.clone(),
                stdout: directory.join("node.stdout.log"),
                stderr: directory.join("node.stderr.log"),
            };
            let result = one_network(&binary, &binary, &hash, &hash, &profile, &directory, timing);
            assert!(
                !tcp::listening(port).unwrap(),
                "owned fixture port retained: {scenario}"
            );
            assert!(
                fs::read_to_string(&profile.stderr)
                    .unwrap()
                    .contains("SYNTHETIC_SMOKE_DIAGNOSTIC")
            );
            if ["healthy", "stagnant", "no-peers"].contains(&scenario) {
                assert_eq!(result["RpcReady"], true, "{scenario}: {result}");
                assert_eq!(result["ParentLossCleanup"], true);
                assert_eq!(result["RelaunchAfterParentLoss"], true);
                assert_eq!(result["Success"], scenario != "no-peers");
                let records = fs::read_to_string(app.join("fixture-parent-argv.txt")).unwrap();
                assert_eq!(
                    records.lines().count(),
                    2,
                    "expected exact parent and relaunch only: {scenario}"
                );
                assert!(records.contains("--kgw-live-smoke-parent"));
                assert_eq!(result["ChainProgressObserved"], scenario != "stagnant");
                assert_eq!(result["IsSynced"], false);
                assert_eq!(result["FullSyncRequiredForProduction"], true);
            } else {
                assert_eq!(result["Success"], false, "{scenario}: {result}");
                assert_eq!(result["RpcReady"], false);
                assert!(result.get("Failure").is_some());
            }
        }
        let protected = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = protected.local_addr().unwrap().port();
        let directory = root.path().join("occupied");
        fs::create_dir(&directory).unwrap();
        let profile = Profile {
            network: Network::Testnet10,
            port,
            app_dir: directory.join("data"),
            stdout: directory.join("out.log"),
            stderr: directory.join("err.log"),
        };
        let result = one_network(
            &binary,
            &binary,
            &hash,
            &hash,
            &profile,
            &directory,
            Timing {
                release: Duration::from_millis(10),
                ..timing
            },
        );
        assert_eq!(result["Success"], false);
        assert!(tcp::listening(port).unwrap());
        assert!(
            !profile.stdout.exists(),
            "busy port must not start or terminate any process"
        );
        drop(protected);
        let result = one_network(
            &binary,
            &binary,
            "wrong-identity",
            &hash,
            &profile,
            &directory,
            timing,
        );
        assert_eq!(result["Success"], false);
        assert!(!profile.stdout.exists());
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "Requires explicit isolated external parity request paths"]
    fn explicit_saved_lifecycle_parity_requests() {
        let input = std::env::var_os("KGW_SMOKE_PARITY_REQUESTS").expect("explicit test requests");
        let output = std::env::var_os("KGW_SMOKE_PARITY_RESULTS").expect("explicit test results");
        let requests: Value = serde_json::from_slice(&fs::read(input).unwrap()).unwrap();
        let mut results = Vec::new();
        for case in requests.as_array().unwrap() {
            let path = |name: &str| PathBuf::from(case[name].as_str().unwrap());
            let directory = path("native_directory");
            fs::create_dir_all(&directory).unwrap();
            let network = Network::parse(case["network"].as_str().unwrap()).unwrap();
            let profile = Profile {
                network,
                port: case["port"].as_u64().unwrap().try_into().unwrap(),
                app_dir: path("native_appdir"),
                stdout: directory.join(format!("{}.stdout.log", network.name())),
                stderr: directory.join(format!("{}.stderr.log", network.name())),
            };
            let binary = path("fixture_binary");
            let hash = hash_file(&binary).unwrap();
            let timing = Timing {
                ready: Duration::from_secs(3),
                observation: Duration::ZERO,
                poll: Duration::from_millis(10),
                probe: Duration::from_secs(1),
                orphan: Duration::from_secs(1),
                release: Duration::from_secs(1),
            };
            let result = one_network(&binary, &binary, &hash, &hash, &profile, &directory, timing);
            assert!(!tcp::listening(profile.port).unwrap());
            results.push(json!({"id":case["id"],"result":result}));
        }
        fs::write(output, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
