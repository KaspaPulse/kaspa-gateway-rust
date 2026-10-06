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
    installer: PathBuf,
    output_directory: PathBuf,
    expected_installer_sha256: String,
    expected_installed_sha256: String,
    version: String,
    startup_timeout: Duration,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut installer = None;
    let mut output_directory = None;
    let mut expected_installer_sha256 = None;
    let mut expected_installed_sha256 = None;
    let mut version = None;
    let mut startup_timeout = Duration::from_secs(120);

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--installer" => installer = Some(PathBuf::from(value)),
            "--output-directory" => output_directory = Some(PathBuf::from(value)),
            "--expected-installer-sha256" => expected_installer_sha256 = Some(value),
            "--expected-installed-sha256" => expected_installed_sha256 = Some(value),
            "--version" => version = Some(value),
            "--startup-timeout-seconds" => {
                let seconds = value
                    .parse::<u64>()
                    .ok()
                    .filter(|seconds| *seconds > 0 && *seconds <= 300)
                    .ok_or_else(|| format!("invalid startup timeout seconds: {value}"))?;
                startup_timeout = Duration::from_secs(seconds);
            }
            _ => return Err(format!("unknown uninstall/reinstall argument: {flag}")),
        }
    }

    let output_directory =
        output_directory.ok_or_else(|| "--output-directory <path> is required".to_owned())?;
    if !output_directory.is_absolute() {
        return Err("--output-directory must be absolute".to_owned());
    }

    Ok(Args {
        installer: installer.ok_or_else(|| "--installer <path> is required".to_owned())?,
        output_directory,
        expected_installer_sha256: normalize_sha256(
            &expected_installer_sha256
                .ok_or_else(|| "--expected-installer-sha256 <hex> is required".to_owned())?,
        )?,
        expected_installed_sha256: normalize_sha256(
            &expected_installed_sha256
                .ok_or_else(|| "--expected-installed-sha256 <hex> is required".to_owned())?,
        )?,
        version: version
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "--version <X.Y.Z> is required".to_owned())?,
        startup_timeout,
    })
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

fn find_uninstaller(install_dir: &Path) -> Result<Option<PathBuf>, String> {
    let uninstallers = find_files(install_dir, &|path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| {
                let lower = name.to_ascii_lowercase();
                lower == "uninstall.exe" || (lower.starts_with("unins") && lower.ends_with(".exe"))
            })
    })?;
    match uninstallers.len() {
        0 => Ok(None),
        1 => Ok(Some(uninstallers[0].clone())),
        count => Err(format!(
            "expected at most one NSIS uninstaller; found {count}"
        )),
    }
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

    let result = async {
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
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!(
            "{error}; {phase} process cleanup also failed: {cleanup_error}"
        )),
    }
}

async fn uninstall_and_verify(
    install_dir: &Path,
    data_dir: &Path,
    sentinel_path: &Path,
    sentinel_sha256: &str,
    label: &str,
) -> Result<Value, String> {
    let uninstaller = find_uninstaller(install_dir)?
        .ok_or_else(|| format!("{label} NSIS uninstaller is missing"))?;
    let status = Command::new(&uninstaller)
        .arg("/S")
        .status()
        .map_err(|error| format!("launch {label} NSIS uninstaller: {error}"))?;
    require_success(&format!("{label} silent NSIS uninstall"), status)?;
    sleep(Duration::from_secs(3)).await;

    let remaining = find_files(install_dir, &|path| {
        path.file_name().and_then(|value| value.to_str()) == Some("kaspa-gateway-desktop.exe")
    })?;
    if !remaining.is_empty() {
        return Err(format!(
            "{label} installed executable remains after uninstall"
        ));
    }
    if !data_dir.is_dir() {
        return Err(format!("{label} isolated user-data root was removed"));
    }
    if !sentinel_path.is_file() {
        return Err(format!("{label} isolated user-data sentinel was removed"));
    }
    let sentinel_after = sha256_file(sentinel_path)?;
    if sentinel_after != sentinel_sha256 {
        return Err(format!("{label} isolated user-data sentinel changed"));
    }

    Ok(json!({
        "phase": label,
        "uninstall": "PASS",
        "installedExecutableRemoved": true,
        "isolatedUserDataPreserved": true,
        "sentinelSha256": sentinel_after
    }))
}

fn verify_log_prefix(before: &[u8], after: &[u8]) -> Result<(), String> {
    if before.is_empty() {
        return Err("first-install application log is empty".to_owned());
    }
    if after.len() <= before.len() || !after.starts_with(before) {
        return Err(
            "reinstall launch did not preserve and extend the first-install application log"
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

async fn best_effort_cleanup(install_dir: &Path) -> Result<Value, String> {
    if let Some(uninstaller) = find_uninstaller(install_dir)? {
        let status = Command::new(&uninstaller)
            .arg("/S")
            .status()
            .map_err(|error| format!("launch cleanup NSIS uninstaller: {error}"))?;
        require_success("cleanup silent NSIS uninstall", status)?;
        sleep(Duration::from_secs(3)).await;
    }
    let remaining = find_files(install_dir, &|path| {
        path.file_name().and_then(|value| value.to_str()) == Some("kaspa-gateway-desktop.exe")
    })?;
    if !remaining.is_empty() {
        return Err("cleanup left installed executable behind".to_owned());
    }
    Ok(json!({
        "cleanup": "PASS",
        "installedExecutableAbsent": true
    }))
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    let installer_hash = exact_hash(
        &args.installer,
        &args.expected_installer_sha256,
        "target installer",
    )?;

    ensure_fresh_directory(&args.output_directory)?;
    let install_dir = args.output_directory.join("installed");
    let data_dir = args.output_directory.join("data");
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("create install directory: {error}"))?;
    fs::create_dir_all(&data_dir).map_err(|error| format!("create data directory: {error}"))?;

    let result: Result<Value, String> = async {
        install_nsis(&args.installer, &install_dir, "first install")?;
        let first_binary = installed_binary(&install_dir)?;
        let first_installed_hash = exact_hash(
            &first_binary,
            &args.expected_installed_sha256,
            "first installed executable",
        )?;
        let first_runtime = launch_and_probe(
            root,
            &first_binary,
            &data_dir,
            args.startup_timeout,
            "first-install",
        )
        .await?;

        let log_path = data_dir.join("logs").join("KaspaGateway.log");
        let first_log = fs::read(&log_path).map_err(|error| {
            format!(
                "first-install application-created log missing/unreadable {}: {error}",
                log_path.display()
            )
        })?;
        if first_log.is_empty() {
            return Err("first-install application-created log is empty".to_owned());
        }
        let first_log_sha256 = sha256_file(&log_path)?;

        let sentinel_path = data_dir.join("uninstall-reinstall-sentinel.json");
        let sentinel = format!(
            "{{\"schema\":1,\"purpose\":\"uninstall-reinstall\",\"version\":\"{}\"}}\n",
            args.version
        );
        fs::write(&sentinel_path, sentinel.as_bytes())
            .map_err(|error| format!("write isolated reinstall sentinel: {error}"))?;
        let sentinel_sha256 = sha256_file(&sentinel_path)?;

        let first_uninstall = uninstall_and_verify(
            &install_dir,
            &data_dir,
            &sentinel_path,
            &sentinel_sha256,
            "first-uninstall",
        )
        .await?;
        if sha256_file(&log_path)? != first_log_sha256 {
            return Err("first uninstall changed application-created user-data log".to_owned());
        }

        install_nsis(&args.installer, &install_dir, "reinstall")?;
        let second_binary = installed_binary(&install_dir)?;
        let second_installed_hash = exact_hash(
            &second_binary,
            &args.expected_installed_sha256,
            "reinstalled executable",
        )?;
        if second_installed_hash != first_installed_hash {
            return Err("same-version reinstall changed installed executable identity".to_owned());
        }
        if sha256_file(&sentinel_path)? != sentinel_sha256 {
            return Err("reinstall changed isolated user-data sentinel".to_owned());
        }
        if sha256_file(&log_path)? != first_log_sha256 {
            return Err("reinstall changed user-data log before second launch".to_owned());
        }

        let second_runtime = launch_and_probe(
            root,
            &second_binary,
            &data_dir,
            args.startup_timeout,
            "reinstall",
        )
        .await?;
        let second_log = fs::read(&log_path)
            .map_err(|error| format!("read reinstall application log: {error}"))?;
        verify_log_prefix(&first_log, &second_log)?;
        let second_log_sha256 = sha256_file(&log_path)?;
        if sha256_file(&sentinel_path)? != sentinel_sha256 {
            return Err("second launch changed isolated user-data sentinel".to_owned());
        }

        let final_uninstall = uninstall_and_verify(
            &install_dir,
            &data_dir,
            &sentinel_path,
            &sentinel_sha256,
            "final-uninstall",
        )
        .await?;

        Ok(json!({
            "passed": true,
            "acceptanceMode": "same-version-uninstall-reinstall-native-runtime",
            "version": args.version,
            "installerSha256": installer_hash,
            "installedBinarySha256": first_installed_hash,
            "installDirectory": install_dir,
            "dataDirectory": data_dir,
            "firstInstall": {
                "runtime": first_runtime,
                "applicationLogSha256": first_log_sha256,
                "applicationLogBytes": first_log.len(),
                "uninstall": first_uninstall
            },
            "reinstall": {
                "runtime": second_runtime,
                "applicationLogSha256": second_log_sha256,
                "applicationLogBytes": second_log.len(),
                "uninstall": final_uninstall
            },
            "dataPreservation": {
                "sentinelSha256": sentinel_sha256,
                "sentinelPreservedAcrossFirstUninstall": true,
                "sentinelPreservedAcrossReinstall": true,
                "sentinelPreservedAcrossFinalUninstall": true,
                "applicationLogPreservedAcrossFirstUninstall": true,
                "applicationLogPreservedAcrossReinstallAsExactPrefix": true
            }
        }))
    }
    .await;

    let cleanup = best_effort_cleanup(&install_dir).await;
    match (result, cleanup) {
        (Ok(mut value), Ok(cleanup)) => {
            if let Some(object) = value.as_object_mut() {
                object.insert("cleanup".to_owned(), cleanup);
            }
            Ok(value)
        }
        (Err(error), Ok(_)) => Err(error),
        (Ok(_), Err(cleanup_error)) => Err(format!(
            "uninstall/reinstall acceptance passed but cleanup failed: {cleanup_error}"
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
        .map_err(|error| format!("create uninstall/reinstall runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args
            .output_directory
            .join("uninstall-reinstall-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("serialize uninstall/reinstall result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_requires_exact_identity_inputs() {
        let mut empty = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut empty).is_err());
    }

    #[test]
    fn log_prefix_requires_preservation_and_extension() {
        assert!(verify_log_prefix(b"first\n", b"first\nsecond\n").is_ok());
        assert!(verify_log_prefix(b"first\n", b"first\n").is_err());
        assert!(verify_log_prefix(b"first\n", b"second\n").is_err());
        assert!(verify_log_prefix(b"", b"second\n").is_err());
    }
}
