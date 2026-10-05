use crate::e2e_native_webdriver::{WebDriverClient, WebDriverSession};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

const DEFAULT_PORT: u16 = 4490;
const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
struct Args {
    installer: PathBuf,
    output_directory: PathBuf,
    expected_installer_sha256: String,
    expected_installed_sha256: String,
    expected_version: String,
    webdriver_port: u16,
    window_label: String,
    startup_timeout: Duration,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut installer = None;
    let mut output_directory = None;
    let mut expected_installer_sha256 = None;
    let mut expected_installed_sha256 = None;
    let mut expected_version = None;
    let mut webdriver_port = DEFAULT_PORT;
    let mut window_label = "main".to_owned();
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
            "--port" => {
                webdriver_port = value
                    .parse::<u16>()
                    .ok()
                    .filter(|port| *port != 0)
                    .ok_or_else(|| format!("invalid non-zero WebDriver port: {value}"))?;
            }
            "--window-label" => {
                if value.trim().is_empty() {
                    return Err("--window-label must not be empty".to_owned());
                }
                window_label = value;
            }
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
        webdriver_port,
        window_label,
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

async fn wait_for_ready(
    driver: &WebDriverClient,
    child: &mut Child,
    timeout: Duration,
) -> Result<(), String> {
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("inspect installed app: {error}"))?
        {
            return Err(format!(
                "installed application exited before WebDriver became ready: {status}"
            ));
        }
        match driver.status_ready().await {
            Ok(true) => return Ok(()),
            Ok(false) | Err(_) if started.elapsed() < timeout => sleep(POLL_INTERVAL).await,
            Ok(false) => {
                return Err(format!(
                    "installed application WebDriver did not become ready within {} seconds",
                    timeout.as_secs()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "installed application WebDriver unreachable after {} seconds: {error}",
                    timeout.as_secs()
                ));
            }
        }
    }
}

async fn invoke(
    session: &WebDriverSession,
    command: &str,
    payload: Value,
) -> Result<Value, String> {
    let result = session
        .execute_async(
            r#"
const commandName = arguments[0];
const payload = arguments[1];
const done = arguments[arguments.length - 1];
try {
  const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
  if (typeof invoke !== "function") {
    done({ ok: false, error: "Tauri invoke API is unavailable" });
  } else {
    Promise.resolve(invoke(commandName, payload))
      .then((value) => done({ ok: true, value }))
      .catch((error) => done({ ok: false, error: error?.message || String(error) }));
  }
} catch (error) {
  done({ ok: false, error: error?.message || String(error) });
}
"#,
            vec![json!(command), payload],
        )
        .await?;
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "{command} failed: {}",
            result
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown Tauri invoke error")
        ));
    }
    Ok(result.get("value").cloned().unwrap_or(Value::Null))
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

    TcpListener::bind(("127.0.0.1", args.webdriver_port)).map_err(|error| {
        format!(
            "WebDriver port {} is not free: {error}",
            args.webdriver_port
        )
    })?;

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

        let mut child = Command::new(&installed_binary)
            .current_dir(root)
            .env("TAURI_WEBDRIVER_PORT", args.webdriver_port.to_string())
            .env("WDIO_EMBEDDED_SERVER", "true")
            .env("KASPA_GATEWAY_DATA_DIR", &data_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("launch installed application: {error}"))?;

        let driver = WebDriverClient::loopback(args.webdriver_port)?;
        let test_result = async {
            wait_for_ready(&driver, &mut child, args.startup_timeout).await?;
            let session = driver.create_session(&args.window_label).await?;
            session.set_timeouts(0, 300_000, 65_000).await?;

            let title = session.title().await?;
            let state = session
                .execute_sync(
                    "return { title: document.title, readyState: document.readyState, href: location.href, tauri: !!(window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke) };",
                    Vec::new(),
                )
                .await?;
            if state.get("readyState").and_then(Value::as_str) != Some("complete") {
                return Err(format!("installed UI readyState is not complete: {state}"));
            }
            if state.get("tauri").and_then(Value::as_bool) != Some(true) {
                return Err(format!("installed UI has no Tauri invoke API: {state}"));
            }

            let ping = invoke(&session, "desktop_ping", json!({})).await?;
            if ping.as_str() != Some("pong") {
                return Err(format!("desktop_ping mismatch: {ping}"));
            }
            let info = invoke(&session, "app_info", json!({})).await?;
            if info.get("version").and_then(Value::as_str) != Some(args.expected_version.as_str()) {
                return Err(format!(
                    "installed app_info version mismatch: expected={} value={info}",
                    args.expected_version
                ));
            }

            session.close().await?;
            Ok(json!({
                "title": title,
                "state": state,
                "desktopPing": ping,
                "appInfo": info
            }))
        }
        .await;

        if child
            .try_wait()
            .map_err(|error| format!("inspect installed application after E2E: {error}"))?
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
                "installed E2E passed but uninstall cleanup failed: {uninstall_error}"
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
        "webdriverPort": args.webdriver_port,
        "windowLabel": args.window_label,
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
