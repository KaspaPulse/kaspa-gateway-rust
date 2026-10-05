use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{HWND, LPARAM};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsHungAppWindow,
    IsWindowVisible, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_NULL,
};
#[cfg(windows)]
use windows_sys::core::BOOL;

const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
struct Args {
    installer: PathBuf,
    output_directory: PathBuf,
    expected_installer_sha256: String,
    expected_installed_sha256: String,
    expected_version: String,
    startup_timeout: Duration,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut installer = None;
    let mut output_directory = None;
    let mut expected_installer_sha256 = None;
    let mut expected_installed_sha256 = None;
    let mut expected_version = None;
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
            "--expected-version" => expected_version = Some(value),
            "--startup-timeout-seconds" => {
                let seconds = value
                    .parse::<u64>()
                    .ok()
                    .filter(|seconds| *seconds > 0 && *seconds <= 300)
                    .ok_or_else(|| format!("invalid startup timeout seconds: {value}"))?;
                startup_timeout = Duration::from_secs(seconds);
            }
            _ => return Err(format!("unknown installed NSIS E2E argument: {flag}")),
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
        expected_version: expected_version
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "--expected-version <X.Y.Z> is required".to_owned())?,
        startup_timeout,
    })
}

fn normalize_sha256(value: &str) -> Result<String, String> {
    let value = value.trim().to_ascii_lowercase();
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid SHA-256 hex value: {value:?}"));
    }
    Ok(value)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn ensure_fresh_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        let mut entries =
            fs::read_dir(path).map_err(|error| format!("read {}: {error}", path.display()))?;
        if entries.next().is_some() {
            return Err(format!(
                "installed NSIS E2E output directory must be fresh/empty: {}",
                path.display()
            ));
        }
    } else {
        fs::create_dir_all(path).map_err(|error| format!("create {}: {error}", path.display()))?;
    }
    Ok(())
}

fn find_files(root: &Path, predicate: &impl Fn(&Path) -> bool) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    if !root.exists() {
        return Ok(found);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .map_err(|error| format!("read directory {}: {error}", dir.display()))?
        {
            let entry = entry.map_err(|error| format!("read directory entry: {error}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if predicate(&path) {
                found.push(path);
            }
        }
    }
    Ok(found)
}

fn require_success(label: &str, status: std::process::ExitStatus) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with status {status}"))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeWindowProbe {
    hwnd: usize,
    title: String,
    visible: bool,
    hung: bool,
    responsive: bool,
}

#[cfg(windows)]
struct WindowSearch {
    pid: u32,
    found: Option<NativeWindowProbe>,
}

#[cfg(windows)]
unsafe extern "system" fn enum_window_for_pid(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let search = unsafe { &mut *(lparam as *mut WindowSearch) };
    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if pid != search.pid || unsafe { IsWindowVisible(hwnd) } == 0 {
        return 1;
    }

    let length = unsafe { GetWindowTextLengthW(hwnd) }.max(0) as usize;
    let mut buffer = vec![0u16; length.saturating_add(1)];
    let copied = if buffer.len() > 1 {
        unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) }.max(0) as usize
    } else {
        0
    };
    let title = String::from_utf16_lossy(&buffer[..copied.min(buffer.len())]);
    let hung = unsafe { IsHungAppWindow(hwnd) } != 0;
    let mut message_result = 0usize;
    let responsive = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_NULL,
            0,
            0,
            SMTO_ABORTIFHUNG,
            1000,
            &mut message_result,
        )
    } != 0;

    search.found = Some(NativeWindowProbe {
        hwnd: hwnd as usize,
        title,
        visible: true,
        hung,
        responsive,
    });
    0
}

#[cfg(windows)]
fn native_window_probe(pid: u32) -> Result<Option<NativeWindowProbe>, String> {
    let mut search = WindowSearch { pid, found: None };
    let result = unsafe {
        EnumWindows(
            Some(enum_window_for_pid),
            &mut search as *mut WindowSearch as LPARAM,
        )
    };
    if result == 0 && search.found.is_none() {
        return Err(
            "EnumWindows failed before locating the installed application window".to_owned(),
        );
    }
    Ok(search.found)
}

#[cfg(not(windows))]
fn native_window_probe(_pid: u32) -> Result<Option<NativeWindowProbe>, String> {
    Err("installed NSIS production-native acceptance requires Windows".to_owned())
}

async fn wait_for_native_window(
    child: &mut Child,
    timeout: Duration,
) -> Result<NativeWindowProbe, String> {
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("inspect installed app: {error}"))?
        {
            return Err(format!(
                "installed application exited before native window readiness: {status}"
            ));
        }

        if let Some(probe) = native_window_probe(child.id())?
            && probe.visible
            && !probe.hung
            && probe.responsive
        {
            return Ok(probe);
        }

        if started.elapsed() >= timeout {
            return Err(format!(
                "installed production application did not expose a visible responsive native window within {} seconds",
                timeout.as_secs()
            ));
        }
        sleep(POLL_INTERVAL).await;
    }
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let rendered = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("serialize {}: {error}", path.display()))?;
    fs::write(path, rendered).map_err(|error| format!("write {}: {error}", path.display()))
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    if !args.installer.is_file() {
        return Err(format!("installer missing: {}", args.installer.display()));
    }
    let installer_hash = sha256_file(&args.installer)?;
    if installer_hash != args.expected_installer_sha256 {
        return Err(format!(
            "installer SHA-256 mismatch: expected={} actual={installer_hash}",
            args.expected_installer_sha256
        ));
    }

    ensure_fresh_directory(&args.output_directory)?;
    let install_dir = args.output_directory.join("installed");
    let data_dir = args.output_directory.join("data");
    fs::create_dir_all(&install_dir)
        .map_err(|error| format!("create install directory: {error}"))?;
    fs::create_dir_all(&data_dir).map_err(|error| format!("create data directory: {error}"))?;

    let install_arg = format!("/D={}", install_dir.display());
    let install_status = Command::new(&args.installer)
        .args(["/S", &install_arg])
        .status()
        .map_err(|error| format!("launch NSIS installer: {error}"))?;
    require_success("silent NSIS install", install_status)?;

    let run_result: Result<(Value, PathBuf, String), String> = async {
        let installed = find_files(&install_dir, &|path| {
            path.file_name().and_then(|value| value.to_str())
                == Some("kaspa-gateway-desktop.exe")
        })?;
        if installed.len() != 1 {
            return Err(format!(
                "expected exactly one installed kaspa-gateway-desktop.exe; found {}",
                installed.len()
            ));
        }
        let installed_binary = installed[0].clone();
        let installed_hash = sha256_file(&installed_binary)?;
        if installed_hash != args.expected_installed_sha256 {
            return Err(format!(
                "installed executable SHA-256 mismatch: expected={} actual={installed_hash}",
                args.expected_installed_sha256
            ));
        }

        let working_directory = installed_binary.parent().unwrap_or(root);
        let mut child = Command::new(&installed_binary)
            .current_dir(working_directory)
            .env("KASPA_GATEWAY_DATA_DIR", &data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("launch installed application: {error}"))?;

        let test_result = async {
            let first_probe = wait_for_native_window(&mut child, args.startup_timeout).await?;
            sleep(Duration::from_secs(2)).await;
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("inspect installed application after readiness: {error}"))?
            {
                return Err(format!(
                    "installed application exited during native readiness observation: {status}"
                ));
            }
            let second_probe = native_window_probe(child.id())?
                .ok_or_else(|| "installed application window disappeared after readiness".to_owned())?;
            if !second_probe.visible || second_probe.hung || !second_probe.responsive {
                return Err(format!(
                    "installed application window failed sustained native responsiveness: {second_probe:?}"
                ));
            }
            let data_entries = fs::read_dir(&data_dir)
                .map_err(|error| format!("read isolated data directory: {error}"))?
                .count();
            if data_entries == 0 {
                return Err(format!(
                    "installed application produced no activity in isolated data directory {}",
                    data_dir.display()
                ));
            }

            Ok(json!({
                "processId": child.id(),
                "firstWindowProbe": first_probe,
                "secondWindowProbe": second_probe,
                "processAliveAfterObservation": true,
                "isolatedDataDirectory": data_dir,
                "isolatedDataTopLevelEntries": data_entries,
                "acceptanceMode": "production-native-win32-window",
                "versionEvidence": {
                    "expectedVersion": args.expected_version,
                    "binding": "exact-installed-payload-sha256-from-qualified-local-production-rc"
                }
            }))
        }
        .await;

        if child
            .try_wait()
            .map_err(|error| format!("inspect installed application after native acceptance: {error}"))?
            .is_none()
        {
            child
                .kill()
                .map_err(|error| format!("stop installed application: {error}"))?;
        }
        let _ = child.wait();

        let verification = test_result?;
        Ok((verification, installed_binary, installed_hash))
    }
    .await;

    let uninstallers = find_files(&install_dir, &|path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| {
                let lower = name.to_ascii_lowercase();
                lower == "uninstall.exe" || (lower.starts_with("unins") && lower.ends_with(".exe"))
            })
    })?;
    let uninstall_result = async {
        if uninstallers.len() != 1 {
            return Err(format!(
                "expected exactly one NSIS uninstaller; found {}",
                uninstallers.len()
            ));
        }
        let uninstall_status = Command::new(&uninstallers[0])
            .arg("/S")
            .status()
            .map_err(|error| format!("launch NSIS uninstaller: {error}"))?;
        require_success("silent NSIS uninstall", uninstall_status)?;
        sleep(Duration::from_secs(3)).await;
        if let Ok((_, installed_binary, _)) = &run_result
            && installed_binary.is_file()
        {
            return Err("installed executable remains after uninstall".to_owned());
        }
        Ok(())
    }
    .await;

    let (verification, _installed_binary, installed_hash) = match (run_result, uninstall_result) {
        (Ok(values), Ok(())) => values,
        (Err(error), Ok(())) => return Err(error),
        (Ok(_), Err(uninstall_error)) => {
            return Err(format!(
                "installed production-native acceptance passed but uninstall cleanup failed: {uninstall_error}"
            ));
        }
        (Err(error), Err(uninstall_error)) => {
            return Err(format!(
                "{error}; uninstall cleanup also failed: {uninstall_error}"
            ));
        }
    };

    Ok(json!({
        "passed": true,
        "installer": args.installer,
        "installerSha256": installer_hash,
        "installDirectory": install_dir,
        "dataDirectory": data_dir,
        "installedBinarySha256": installed_hash,
        "expectedVersion": args.expected_version,
        "acceptanceMode": "production-native-win32-window",
        "verification": verification,
        "uninstall": "PASS"
    }))
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<String, String> {
    let args = parse_args(args)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("create installed NSIS E2E runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args.output_directory.join("installed-nsis-e2e-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("serialize installed NSIS E2E result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_validation_is_strict() {
        assert!(normalize_sha256(&"a".repeat(64)).is_ok());
        assert!(normalize_sha256("abc").is_err());
        assert!(normalize_sha256(&"z".repeat(64)).is_err());
    }

    #[test]
    fn cli_requires_exact_identity_inputs() {
        let mut empty = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut empty).is_err());
    }
}
