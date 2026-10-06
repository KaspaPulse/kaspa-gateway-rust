use crate::e2e_installed_nsis::{
    ensure_fresh_directory, find_files, normalize_sha256, require_success, sha256_file,
    wait_for_native_window,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
struct Args {
    baseline_installer: PathBuf,
    target_installer: PathBuf,
    output_directory: PathBuf,
    expected_baseline_installer_sha256: String,
    expected_baseline_installed_sha256: String,
    expected_target_installer_sha256: String,
    expected_target_installed_sha256: String,
    baseline_version: String,
    target_version: String,
    startup_timeout: Duration,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut baseline_installer = None;
    let mut target_installer = None;
    let mut output_directory = None;
    let mut expected_baseline_installer_sha256 = None;
    let mut expected_baseline_installed_sha256 = None;
    let mut expected_target_installer_sha256 = None;
    let mut expected_target_installed_sha256 = None;
    let mut baseline_version = None;
    let mut target_version = None;
    let mut startup_timeout = Duration::from_secs(120);

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--baseline-installer" => baseline_installer = Some(PathBuf::from(value)),
            "--target-installer" => target_installer = Some(PathBuf::from(value)),
            "--output-directory" => output_directory = Some(PathBuf::from(value)),
            "--expected-baseline-installer-sha256" => {
                expected_baseline_installer_sha256 = Some(value)
            }
            "--expected-baseline-installed-sha256" => {
                expected_baseline_installed_sha256 = Some(value)
            }
            "--expected-target-installer-sha256" => expected_target_installer_sha256 = Some(value),
            "--expected-target-installed-sha256" => expected_target_installed_sha256 = Some(value),
            "--baseline-version" => baseline_version = Some(value),
            "--target-version" => target_version = Some(value),
            "--startup-timeout-seconds" => {
                let seconds = value
                    .parse::<u64>()
                    .ok()
                    .filter(|seconds| *seconds > 0 && *seconds <= 300)
                    .ok_or_else(|| format!("invalid startup timeout seconds: {value}"))?;
                startup_timeout = Duration::from_secs(seconds);
            }
            _ => return Err(format!("unknown upgrade acceptance argument: {flag}")),
        }
    }

    let output_directory =
        output_directory.ok_or_else(|| "--output-directory <path> is required".to_owned())?;
    if !output_directory.is_absolute() {
        return Err("--output-directory must be absolute".to_owned());
    }

    Ok(Args {
        baseline_installer: baseline_installer
            .ok_or_else(|| "--baseline-installer <path> is required".to_owned())?,
        target_installer: target_installer
            .ok_or_else(|| "--target-installer <path> is required".to_owned())?,
        output_directory,
        expected_baseline_installer_sha256: normalize_sha256(
            &expected_baseline_installer_sha256.ok_or_else(|| {
                "--expected-baseline-installer-sha256 <hex> is required".to_owned()
            })?,
        )?,
        expected_baseline_installed_sha256: normalize_sha256(
            &expected_baseline_installed_sha256.ok_or_else(|| {
                "--expected-baseline-installed-sha256 <hex> is required".to_owned()
            })?,
        )?,
        expected_target_installer_sha256: normalize_sha256(
            &expected_target_installer_sha256
                .ok_or_else(|| "--expected-target-installer-sha256 <hex> is required".to_owned())?,
        )?,
        expected_target_installed_sha256: normalize_sha256(
            &expected_target_installed_sha256
                .ok_or_else(|| "--expected-target-installed-sha256 <hex> is required".to_owned())?,
        )?,
        baseline_version: nonempty(baseline_version, "--baseline-version")?,
        target_version: nonempty(target_version, "--target-version")?,
        startup_timeout,
    })
}

fn nonempty(value: Option<String>, flag: &str) -> Result<String, String> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{flag} <X.Y.Z> is required"))
}

fn exact_hash(path: &Path, expected: &str, label: &str) -> Result<String, String> {
    if !path.is_file() {
        return Err(format!("{label} missing: {}", path.display()));
    }
    let actual = sha256_file(path)?;
    if actual != expected {
        return Err(format!(
            "{label} SHA-256 mismatch: expected={expected} actual={actual}"
        ));
    }
    Ok(actual)
}

fn install_nsis(installer: &Path, install_dir: &Path, label: &str) -> Result<(), String> {
    let install_arg = format!("/D={}", install_dir.display());
    let status = Command::new(installer)
        .args(["/S", &install_arg])
        .status()
        .map_err(|error| format!("launch {label} NSIS installer: {error}"))?;
    require_success(&format!("{label} silent NSIS install"), status)
}

fn installed_binary(install_dir: &Path) -> Result<PathBuf, String> {
    let installed = find_files(install_dir, &|path| {
        path.file_name().and_then(|value| value.to_str()) == Some("kaspa-gateway-desktop.exe")
    })?;
    if installed.len() != 1 {
        return Err(format!(
            "expected exactly one installed kaspa-gateway-desktop.exe; found {}",
            installed.len()
        ));
    }
    Ok(installed[0].clone())
}

fn uninstaller(install_dir: &Path) -> Result<PathBuf, String> {
    let uninstallers = find_files(install_dir, &|path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| {
                let lower = name.to_ascii_lowercase();
                lower == "uninstall.exe" || (lower.starts_with("unins") && lower.ends_with(".exe"))
            })
    })?;
    if uninstallers.len() != 1 {
        return Err(format!(
            "expected exactly one NSIS uninstaller; found {}",
            uninstallers.len()
        ));
    }
    Ok(uninstallers[0].clone())
}

async fn stop_child(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|error| format!("inspect installed application: {error}"))?
        .is_none()
    {
        child
            .kill()
            .map_err(|error| format!("stop installed application: {error}"))?;
    }
    child
        .wait()
        .map_err(|error| format!("reap installed application: {error}"))?;
    Ok(())
}

async fn launch_and_probe(
    root: &Path,
    binary: &Path,
    data_dir: &Path,
    timeout: Duration,
    phase: &str,
) -> Result<Value, String> {
    let working_directory = binary.parent().unwrap_or(root);
    let mut child = Command::new(binary)
        .current_dir(working_directory)
        .env("KASPA_GATEWAY_DATA_DIR", data_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("launch {phase} installed application: {error}"))?;

    let test_result = async {
        let first_probe = wait_for_native_window(&mut child, timeout).await?;
        sleep(Duration::from_secs(2)).await;
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("inspect {phase} app after readiness: {error}"))?
        {
            return Err(format!(
                "{phase} installed application exited during observation: {status}"
            ));
        }
        let second_probe = wait_for_native_window(&mut child, Duration::from_secs(5)).await?;
        Ok(json!({
            "phase": phase,
            "processId": child.id(),
            "firstWindowProbe": first_probe,
            "secondWindowProbe": second_probe,
            "processAliveAfterObservation": true
        }))
    }
    .await;

    let cleanup = stop_child(&mut child).await;
    match (test_result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!(
            "{error}; {phase} process cleanup also failed: {cleanup_error}"
        )),
    }
}

fn verify_log_prefix(before: &[u8], after: &[u8]) -> Result<(), String> {
    if before.is_empty() {
        return Err("baseline application log is empty".to_owned());
    }
    if after.len() < before.len() || !after.starts_with(before) {
        return Err(
            "target launch did not preserve baseline application log as an exact byte prefix"
                .to_owned(),
        );
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let rendered = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    fs::write(path, rendered).map_err(|error| format!("write {}: {error}", path.display()))
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    let baseline_installer_hash = exact_hash(
        &args.baseline_installer,
        &args.expected_baseline_installer_sha256,
        "baseline installer",
    )?;
    let target_installer_hash = exact_hash(
        &args.target_installer,
        &args.expected_target_installer_sha256,
        "target installer",
    )?;

    ensure_fresh_directory(&args.output_directory)?;
    let install_dir = args.output_directory.join("installed");
    let data_dir = args.output_directory.join("data");
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("create install directory: {error}"))?;
    fs::create_dir_all(&data_dir).map_err(|error| format!("create data directory: {error}"))?;

    let upgrade_result: Result<Value, String> = async {
        install_nsis(&args.baseline_installer, &install_dir, "baseline")?;
        let baseline_binary = installed_binary(&install_dir)?;
        let baseline_installed_hash = exact_hash(
            &baseline_binary,
            &args.expected_baseline_installed_sha256,
            "baseline installed executable",
        )?;
        let baseline_runtime = launch_and_probe(
            root,
            &baseline_binary,
            &data_dir,
            args.startup_timeout,
            "baseline",
        )
        .await?;

        let log_path = data_dir.join("logs").join("KaspaGateway.log");
        let baseline_log = fs::read(&log_path).map_err(|error| {
            format!(
                "baseline application-created log missing/unreadable {}: {error}",
                log_path.display()
            )
        })?;
        if baseline_log.is_empty() {
            return Err("baseline application-created log is empty".to_owned());
        }
        let baseline_log_sha256 = sha256_file(&log_path)?;

        let sentinel_path = data_dir.join("upgrade-preservation-sentinel.json");
        let sentinel = format!(
            "{{\"schema\":1,\"purpose\":\"upgrade-preservation\",\"from\":\"{}\",\"to\":\"{}\"}}\n",
            args.baseline_version, args.target_version
        );
        fs::write(&sentinel_path, sentinel.as_bytes())
            .map_err(|error| format!("write isolated upgrade sentinel: {error}"))?;
        let sentinel_sha256_before = sha256_file(&sentinel_path)?;

        install_nsis(&args.target_installer, &install_dir, "target upgrade")?;
        let target_binary = installed_binary(&install_dir)?;
        let target_installed_hash = exact_hash(
            &target_binary,
            &args.expected_target_installed_sha256,
            "target installed executable",
        )?;
        if target_installed_hash == baseline_installed_hash {
            return Err("upgrade did not replace the installed executable identity".to_owned());
        }

        let sentinel_sha256_after_install = sha256_file(&sentinel_path)?;
        if sentinel_sha256_after_install != sentinel_sha256_before {
            return Err("isolated user-data sentinel changed during in-place upgrade".to_owned());
        }

        let target_runtime = launch_and_probe(
            root,
            &target_binary,
            &data_dir,
            args.startup_timeout,
            "target",
        )
        .await?;

        let target_log = fs::read(&log_path).map_err(|error| {
            format!(
                "target application log missing/unreadable {}: {error}",
                log_path.display()
            )
        })?;
        verify_log_prefix(&baseline_log, &target_log)?;
        let target_log_sha256 = sha256_file(&log_path)?;
        let sentinel_sha256_after_run = sha256_file(&sentinel_path)?;
        if sentinel_sha256_after_run != sentinel_sha256_before {
            return Err("isolated user-data sentinel changed during target runtime".to_owned());
        }

        Ok(json!({
            "passed": true,
            "acceptanceMode": "in-place-nsis-upgrade-native-runtime",
            "baselineVersion": args.baseline_version,
            "targetVersion": args.target_version,
            "installDirectory": install_dir,
            "dataDirectory": data_dir,
            "baseline": {
                "installerSha256": baseline_installer_hash,
                "installedBinarySha256": baseline_installed_hash,
                "runtime": baseline_runtime,
                "applicationLogSha256": baseline_log_sha256,
                "applicationLogBytes": baseline_log.len()
            },
            "target": {
                "installerSha256": target_installer_hash,
                "installedBinarySha256": target_installed_hash,
                "runtime": target_runtime,
                "applicationLogSha256": target_log_sha256,
                "applicationLogBytes": target_log.len()
            },
            "dataPreservation": {
                "applicationCreatedLogPreservedAsExactPrefix": true,
                "sentinelSha256Before": sentinel_sha256_before,
                "sentinelSha256AfterInstall": sentinel_sha256_after_install,
                "sentinelSha256AfterRun": sentinel_sha256_after_run,
                "sameIsolatedDataRoot": true
            }
        }))
    }
    .await;

    let cleanup_result = async {
        let uninstaller = uninstaller(&install_dir)?;
        let status = Command::new(&uninstaller)
            .arg("/S")
            .status()
            .map_err(|error| format!("launch target NSIS uninstaller: {error}"))?;
        require_success("target silent NSIS uninstall", status)?;
        sleep(Duration::from_secs(3)).await;
        let remaining = find_files(&install_dir, &|path| {
            path.file_name().and_then(|value| value.to_str()) == Some("kaspa-gateway-desktop.exe")
        })?;
        if !remaining.is_empty() {
            return Err("installed target executable remains after uninstall".to_owned());
        }
        let sentinel = data_dir.join("upgrade-preservation-sentinel.json");
        if !sentinel.is_file() {
            return Err("isolated user-data sentinel was removed by uninstall".to_owned());
        }
        Ok(json!({
            "uninstall": "PASS",
            "installedExecutableRemoved": true,
            "isolatedUserDataPreservedAfterUninstall": true
        }))
    }
    .await;

    match (upgrade_result, cleanup_result) {
        (Ok(mut value), Ok(cleanup)) => {
            if let Some(object) = value.as_object_mut() {
                object.insert("cleanup".to_owned(), cleanup);
            }
            Ok(value)
        }
        (Err(error), Ok(_)) => Err(error),
        (Ok(_), Err(cleanup_error)) => Err(format!(
            "upgrade acceptance passed but cleanup failed: {cleanup_error}"
        )),
        (Err(error), Err(cleanup_error)) => {
            Err(format!("{error}; cleanup also failed: {cleanup_error}"))
        }
    }
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<String, String> {
    let args = parse_args(args)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("create upgrade acceptance runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args.output_directory.join("upgrade-acceptance-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("serialize upgrade acceptance result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_requires_all_exact_identity_inputs() {
        let mut empty = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut empty).is_err());
    }

    #[test]
    fn log_prefix_validation_rejects_replacement_or_truncation() {
        assert!(verify_log_prefix(b"baseline\n", b"baseline\ntarget\n").is_ok());
        assert!(verify_log_prefix(b"baseline\n", b"target\n").is_err());
        assert!(verify_log_prefix(b"baseline\n", b"base").is_err());
        assert!(verify_log_prefix(b"", b"target").is_err());
    }
}
