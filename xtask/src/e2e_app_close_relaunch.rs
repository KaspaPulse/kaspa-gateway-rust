use crate::e2e_native_webdriver::{NativeWebDriverHarness, WebDriverSession};
use crate::e2e_owned_process::{
    ExactProcessExitGuard, prepare_exact_process_exit_guard, wait_prepared_exact_process_exit,
};
use kaspa_gateway_e2e_wasm::{
    NetworkPorts, is_stopped_owner_status_native, parse_key_value_line_native,
    pid_from_status_native, runtime_port_profile_native,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::time::sleep;

const DEFAULT_WEBDRIVER_PORT: u16 = 4455;
const POLL_FAST: Duration = Duration::from_millis(250);
const POLL_OWNER: Duration = Duration::from_millis(750);

#[derive(Debug)]
struct Args {
    app_binary: PathBuf,
    output_directory: PathBuf,
    webdriver_port: u16,
    window_label: String,
    startup_timeout: Duration,
}

#[derive(Debug, Clone, Serialize)]
struct OwnerStatus {
    status: String,
    pid: u32,
    fields: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
struct ParentIdentity {
    pid: u32,
    start_time: i64,
    executable: String,
}

#[derive(Debug, Clone, Serialize)]
struct UiState {
    start_exists: bool,
    stop_exists: bool,
    start_disabled: bool,
    stop_disabled: bool,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut app_binary = None;
    let mut output_directory = None;
    let mut webdriver_port = DEFAULT_WEBDRIVER_PORT;
    let mut window_label = "main".to_owned();
    let mut startup_timeout = Duration::from_secs(120);

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--app-binary" => app_binary = Some(PathBuf::from(value)),
            "--output-directory" => output_directory = Some(PathBuf::from(value)),
            "--port" => {
                webdriver_port = value
                    .parse::<u16>()
                    .ok()
                    .filter(|port| *port > 0 && *port < u16::MAX)
                    .ok_or_else(|| format!("invalid WebDriver base port: {value}"))?;
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
            _ => return Err(format!("unknown app-close-relaunch argument: {flag}")),
        }
    }

    Ok(Args {
        app_binary: app_binary.ok_or_else(|| "--app-binary <path> is required".to_owned())?,
        output_directory: output_directory
            .ok_or_else(|| "--output-directory <path> is required".to_owned())?,
        webdriver_port,
        window_label,
        startup_timeout,
    })
}

fn fields_map(text: &str) -> BTreeMap<String, String> {
    parse_key_value_line_native(text).into_iter().collect()
}

fn field<'a>(fields: &'a BTreeMap<String, String>, name: &str) -> &'a str {
    fields.get(name).map(String::as_str).unwrap_or("")
}

fn parse_positive_u32(value: &str, label: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{label} is missing or invalid: {value:?}"))
}

fn parse_positive_i64(value: &str, label: &str) -> Result<i64, String> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{label} is missing or invalid: {value:?}"))
}

fn parent_identity(status: &OwnerStatus) -> Result<ParentIdentity, String> {
    Ok(ParentIdentity {
        pid: parse_positive_u32(field(&status.fields, "parent_pid"), "parent_pid")?,
        start_time: parse_positive_i64(
            field(&status.fields, "parent_start_time"),
            "parent_start_time",
        )?,
        executable: {
            let executable = field(&status.fields, "parent_executable").trim();
            if executable.is_empty() {
                return Err("parent_executable is missing".to_owned());
            }
            executable.to_owned()
        },
    })
}

fn owner_matches_running_ready(fields: &BTreeMap<String, String>, network: &str) -> bool {
    field(fields, "network").eq_ignore_ascii_case(network)
        && field(fields, "role").eq_ignore_ascii_case("node")
        && field(fields, "running").eq_ignore_ascii_case("true")
        && field(fields, "readiness").eq_ignore_ascii_case("READY")
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let rendered = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(path, rendered)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
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
  const tauriInvoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
  if (typeof tauriInvoke !== "function") {
    done({ ok: false, error: "Tauri invoke API is unavailable" });
  } else {
    Promise.resolve(tauriInvoke(commandName, payload))
      .then((value) => done({ ok: true, value }))
      .catch((error) => done({
        ok: false,
        error: error && error.message ? error.message : String(error),
      }));
  }
} catch (error) {
  done({
    ok: false,
    error: error && error.message ? error.message : String(error),
  });
}
"#,
            vec![json!(command), payload],
        )
        .await?;
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "IPC {command} failed: {}",
            result
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
        ));
    }
    Ok(result.get("value").cloned().unwrap_or(Value::Null))
}

async fn click_test_id(session: &WebDriverSession, test_id: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let result = session
            .execute_sync(
                r#"
const wanted = arguments[0];
const node = Array.from(document.querySelectorAll("[data-testid]"))
  .find((item) => item.getAttribute("data-testid") === wanted);
if (!node) return { ok: false, reason: "missing" };
if (node.disabled || node.getAttribute("aria-disabled") === "true") {
  return { ok: false, reason: "disabled" };
}
node.scrollIntoView?.({ block: "center", inline: "center" });
node.click();
return { ok: true };
"#,
                vec![json!(test_id)],
            )
            .await?;
        if result.get("ok").and_then(Value::as_bool) == Some(true) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "unable to click {test_id}: {}",
                result
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ));
        }
        sleep(POLL_FAST).await;
    }
}

async fn set_control_value(
    session: &WebDriverSession,
    test_id: &str,
    value: &str,
) -> Result<(), String> {
    let result = session
        .execute_sync(
            r#"
const wanted = arguments[0];
const nextValue = String(arguments[1]);
const node = Array.from(document.querySelectorAll("[data-testid]"))
  .find((item) => item.getAttribute("data-testid") === wanted);
if (!node) return { ok: false, reason: "missing" };
if (node.disabled || node.readOnly) return { ok: false, reason: "not-editable" };
node.value = nextValue;
node.dispatchEvent(new Event("input", { bubbles: true }));
node.dispatchEvent(new Event("change", { bubbles: true }));
return { ok: true, value: String(node.value || "") };
"#,
            vec![json!(test_id), json!(value)],
        )
        .await?;
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "unable to set {test_id}: {}",
            result
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ));
    }
    if result.get("value").and_then(Value::as_str) != Some(value) {
        return Err(format!("{test_id} did not retain value {value:?}"));
    }
    Ok(())
}

async fn set_control_checked(
    session: &WebDriverSession,
    test_id: &str,
    checked: bool,
) -> Result<(), String> {
    let result = session
        .execute_sync(
            r#"
const wanted = arguments[0];
const nextChecked = Boolean(arguments[1]);
const node = Array.from(document.querySelectorAll("[data-testid]"))
  .find((item) => item.getAttribute("data-testid") === wanted);
if (!node) return { ok: false, reason: "missing" };
if (node.disabled || node.readOnly) return { ok: false, reason: "not-editable" };
node.checked = nextChecked;
node.dispatchEvent(new Event("input", { bubbles: true }));
node.dispatchEvent(new Event("change", { bubbles: true }));
return { ok: true, checked: Boolean(node.checked) };
"#,
            vec![json!(test_id), json!(checked)],
        )
        .await?;
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "unable to set {test_id}: {}",
            result
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        ));
    }
    if result.get("checked").and_then(Value::as_bool) != Some(checked) {
        return Err(format!("{test_id} did not retain checked={checked}"));
    }
    Ok(())
}

async fn open_node_settings(session: &WebDriverSession, network: &str) -> Result<(), String> {
    click_test_id(session, "kgw-tab-kaspa-node").await?;
    click_test_id(session, &format!("kgw-node-network-{network}")).await?;
    click_test_id(session, &format!("kgw-node-settings-{network}")).await
}

async fn owner_status(session: &WebDriverSession, network: &str) -> Result<OwnerStatus, String> {
    let value = invoke(
        session,
        "kgw_runtime_owner_status_v1",
        json!({ "network": network, "runtimeRole": "node" }),
    )
    .await?;
    let status = value
        .as_str()
        .ok_or_else(|| format!("owner status node/{network} was not a string: {value}"))?
        .to_owned();
    Ok(OwnerStatus {
        pid: pid_from_status_native(&status).unwrap_or(0),
        fields: fields_map(&status),
        status,
    })
}

async fn wait_for_owner(
    session: &WebDriverSession,
    network: &str,
    timeout_duration: Duration,
) -> Result<OwnerStatus, String> {
    let deadline = Instant::now() + timeout_duration;
    let mut last = String::new();
    loop {
        match owner_status(session, network).await {
            Ok(status) => {
                last.clone_from(&status.status);
                if status.pid > 0 && owner_matches_running_ready(&status.fields, network) {
                    return Ok(status);
                }
            }
            Err(error) => last = error,
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "owner status node/{network} did not become READY: {last}"
            ));
        }
        sleep(POLL_OWNER).await;
    }
}

async fn wait_for_stopped(
    session: &WebDriverSession,
    network: &str,
    timeout_duration: Duration,
) -> Result<OwnerStatus, String> {
    let deadline = Instant::now() + timeout_duration;
    let mut last = String::new();
    loop {
        match owner_status(session, network).await {
            Ok(status) => {
                last.clone_from(&status.status);
                if is_stopped_owner_status_native(&status.status) {
                    return Ok(status);
                }
            }
            Err(error) => last = error,
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "owner status node/{network} did not reconcile to stopped: {last}"
            ));
        }
        sleep(Duration::from_millis(500)).await;
    }
}

async fn wait_port(
    host: &str,
    port: u16,
    open: bool,
    timeout_duration: Duration,
) -> Result<(), String> {
    let address = SocketAddr::new(
        host.parse::<IpAddr>()
            .map_err(|error| format!("invalid TCP host {host}: {error}"))?,
        port,
    );
    let deadline = Instant::now() + timeout_duration;
    loop {
        let reachable = TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_ok();
        if reachable == open {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "TCP {host}:{port} did not become {}",
                if open { "reachable" } else { "free" }
            ));
        }
        sleep(POLL_FAST).await;
    }
}

async fn wait_node_ui(
    session: &WebDriverSession,
    network: &str,
    expected_running: bool,
) -> Result<UiState, String> {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        open_node_settings(session, network).await?;
        let value = session
            .execute_sync(
                r#"
const net = arguments[0];
const start = document.querySelector('[data-testid="kgw-node-start-' + net + '"]');
const stop = document.querySelector('[data-testid="kgw-node-stop-' + net + '"]');
return {
  startExists: Boolean(start),
  stopExists: Boolean(stop),
  startDisabled: Boolean(start?.disabled || start?.getAttribute("aria-disabled") === "true"),
  stopDisabled: Boolean(stop?.disabled || stop?.getAttribute("aria-disabled") === "true"),
};
"#,
                vec![json!(network)],
            )
            .await?;
        let state = UiState {
            start_exists: value
                .get("startExists")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            stop_exists: value
                .get("stopExists")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            start_disabled: value
                .get("startDisabled")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            stop_disabled: value
                .get("stopDisabled")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        };
        let matches = state.start_exists
            && state.stop_exists
            && if expected_running {
                state.start_disabled && !state.stop_disabled
            } else {
                !state.start_disabled && state.stop_disabled
            };
        if matches {
            return Ok(state);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "node UI {network} did not reach running={expected_running}: {state:?}"
            ));
        }
        sleep(Duration::from_millis(500)).await;
    }
}

async fn configure_and_start_node(
    session: &WebDriverSession,
    network: &str,
    ports: NetworkPorts,
    output_directory: &Path,
) -> Result<OwnerStatus, String> {
    open_node_settings(session, network).await?;
    set_control_value(
        session,
        &format!("kgw-node-field-{network}-rpcListenHost"),
        "127.0.0.1",
    )
    .await?;
    set_control_value(
        session,
        &format!("kgw-node-field-{network}-rpcListenPort"),
        &ports.rpc_port.to_string(),
    )
    .await?;
    set_control_checked(
        session,
        &format!("kgw-node-field-{network}-listenEnabled"),
        true,
    )
    .await?;
    set_control_value(
        session,
        &format!("kgw-node-field-{network}-listenHost"),
        "127.0.0.1",
    )
    .await?;
    set_control_value(
        session,
        &format!("kgw-node-field-{network}-listenPort"),
        &ports.p2p_port.to_string(),
    )
    .await?;
    click_test_id(session, &format!("kgw-node-start-{network}")).await?;

    let status = wait_for_owner(session, network, Duration::from_secs(180)).await?;
    parent_identity(&status)?;
    wait_port("127.0.0.1", ports.rpc_port, true, Duration::from_secs(180)).await?;
    wait_port("127.0.0.1", ports.p2p_port, true, Duration::from_secs(180)).await?;
    wait_node_ui(session, network, true).await?;
    write_json(
        &output_directory.join(format!("{network}-owner-status.json")),
        &status,
    )?;
    Ok(status)
}

async fn stop_node(
    session: &WebDriverSession,
    network: &str,
    ports: NetworkPorts,
    output_directory: &Path,
) -> Result<(), String> {
    open_node_settings(session, network).await?;
    let _ = session
        .execute_sync(
            r#"
const net = arguments[0];
const stop = document.querySelector('[data-testid="kgw-node-stop-' + net + '"]');
if (stop && !stop.disabled && stop.getAttribute("aria-disabled") !== "true") {
  stop.click();
  return true;
}
return false;
"#,
            vec![json!(network)],
        )
        .await?;
    let _ = invoke(
        session,
        "kgw_kgw_disable_network_v1",
        json!({ "network": network, "runtimeRole": "node" }),
    )
    .await;

    let stopped = wait_for_stopped(session, network, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.rpc_port, false, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.p2p_port, false, Duration::from_secs(60)).await?;
    let ui = wait_node_ui(session, network, false).await?;
    write_json(
        &output_directory.join(format!("{network}-stopped.json")),
        &json!({ "stopped": stopped, "ui": ui }),
    )
}

async fn reconcile_after_relaunch(
    session: &WebDriverSession,
    network: &str,
    ports: NetworkPorts,
    output_directory: &Path,
) -> Result<Value, String> {
    let stopped = wait_for_stopped(session, network, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.rpc_port, false, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.p2p_port, false, Duration::from_secs(60)).await?;
    let ui = wait_node_ui(session, network, false).await?;
    let value = json!({ "stopped": stopped, "ui": ui });
    write_json(
        &output_directory.join(format!("{network}-reconciled.json")),
        &value,
    )?;
    Ok(value)
}

async fn cleanup_runtime_workers(session: &WebDriverSession) -> Result<(), String> {
    let _ = invoke(session, "kgw_shutdown_all_runtime_workers_v1", json!({})).await?;
    Ok(())
}

fn wait_exact_parent_exit(
    guard: ExactProcessExitGuard,
    output_path: &Path,
) -> Result<Value, String> {
    let process_id = guard.process_id();
    let rendered = wait_prepared_exact_process_exit(guard, output_path, 60)?;
    let value: Value = serde_json::from_str(&rendered)
        .map_err(|error| format!("failed to parse exact-parent exit evidence: {error}"))?;
    if value.get("exact_identity_exited").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "desktop parent {process_id} did not report exact_identity_exited=true"
        ));
    }
    Ok(value)
}

async fn close_window_and_wait(
    mut harness: NativeWebDriverHarness,
    parent: &ParentIdentity,
    output_directory: &Path,
) -> Result<Value, String> {
    let exit_guard =
        prepare_exact_process_exit_guard(parent.pid, &parent.executable, parent.start_time)?;
    let close_result = harness.session().close_window().await;
    let close_record = match &close_result {
        Ok(value) => json!({ "requested": true, "response": value, "error": Value::Null }),
        Err(error) => json!({ "requested": true, "response": Value::Null, "error": error }),
    };
    write_json(&output_directory.join("close-window.json"), &close_record)?;

    let exit_result = wait_exact_parent_exit(
        exit_guard,
        &output_directory.join("desktop-parent-exit.json"),
    );
    match exit_result {
        Ok(value) => {
            drop(harness);
            Ok(value)
        }
        Err(error) => {
            let cleanup = harness.shutdown().await;
            Err(match cleanup {
                Ok(()) => error,
                Err(cleanup_error) => format!(
                    "{error}; failed cleanup after close/relaunch exit failure: {cleanup_error}"
                ),
            })
        }
    }
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    if !args.app_binary.is_file() {
        return Err(format!(
            "app-close-relaunch binary does not exist: {}",
            args.app_binary.display()
        ));
    }
    fs::create_dir_all(&args.output_directory).map_err(|error| {
        format!(
            "failed to create {}: {error}",
            args.output_directory.display()
        )
    })?;

    let profile = runtime_port_profile_native()?;
    let networks = [
        ("mainnet", profile.mainnet),
        ("testnet10", profile.testnet10),
    ];
    let before_dir = args.output_directory.join("before-close");
    let after_dir = args.output_directory.join("after-relaunch");

    let mut first = NativeWebDriverHarness::launch(
        root,
        &args.app_binary,
        args.webdriver_port,
        &args.window_label,
        args.startup_timeout,
    )
    .await?;

    let first_phase = async {
        let session = first.session();
        let mut statuses = BTreeMap::new();
        for (network, ports) in networks {
            let status = configure_and_start_node(session, network, ports, &before_dir).await?;
            statuses.insert(network.to_owned(), status);
        }
        let mainnet_parent = parent_identity(
            statuses
                .get("mainnet")
                .ok_or_else(|| "mainnet owner status missing".to_owned())?,
        )?;
        let testnet_parent = parent_identity(
            statuses
                .get("testnet10")
                .ok_or_else(|| "testnet10 owner status missing".to_owned())?,
        )?;
        if mainnet_parent != testnet_parent {
            return Err(format!(
                "parallel node workers do not share one exact desktop parent: mainnet={mainnet_parent:?} testnet10={testnet_parent:?}"
            ));
        }
        write_json(&before_dir.join("desktop-parent.json"), &mainnet_parent)?;
        Ok::<_, String>(mainnet_parent)
    }
    .await;

    let first_parent = match first_phase {
        Ok(parent) => parent,
        Err(error) => {
            let cleanup = cleanup_runtime_workers(first.session()).await;
            let shutdown = first.shutdown().await;
            return Err(format!(
                "{error}; cleanup={:?}; shutdown={:?}",
                cleanup.err(),
                shutdown.err()
            ));
        }
    };

    close_window_and_wait(first, &first_parent, &before_dir).await?;
    for (_, ports) in networks {
        wait_port("127.0.0.1", ports.rpc_port, false, Duration::from_secs(60)).await?;
        wait_port("127.0.0.1", ports.p2p_port, false, Duration::from_secs(60)).await?;
    }

    let second_port = args
        .webdriver_port
        .checked_add(1)
        .ok_or_else(|| "WebDriver relaunch port overflow".to_owned())?;
    let mut second = NativeWebDriverHarness::launch(
        root,
        &args.app_binary,
        second_port,
        &args.window_label,
        args.startup_timeout,
    )
    .await?;

    let second_phase = async {
        let session = second.session();
        let mut reconciled = BTreeMap::new();
        for (network, ports) in networks {
            let value = reconcile_after_relaunch(session, network, ports, &after_dir).await?;
            reconciled.insert(network.to_owned(), value);
        }
        write_json(&after_dir.join("reconciled.json"), &reconciled)?;

        let mut recovered = BTreeMap::new();
        for (network, ports) in networks {
            let status = configure_and_start_node(session, network, ports, &after_dir).await?;
            recovered.insert(network.to_owned(), status);
        }
        let mainnet_parent = parent_identity(
            recovered
                .get("mainnet")
                .ok_or_else(|| "recovered mainnet owner status missing".to_owned())?,
        )?;
        let testnet_parent = parent_identity(
            recovered
                .get("testnet10")
                .ok_or_else(|| "recovered testnet10 owner status missing".to_owned())?,
        )?;
        if mainnet_parent != testnet_parent {
            return Err(format!(
                "recovered node workers do not share one exact desktop parent: mainnet={mainnet_parent:?} testnet10={testnet_parent:?}"
            ));
        }
        if mainnet_parent == first_parent {
            return Err("relaunch reused the original exact desktop parent identity".to_owned());
        }
        write_json(&after_dir.join("desktop-parent.json"), &mainnet_parent)?;

        for (network, ports) in networks.into_iter().rev() {
            stop_node(session, network, ports, &after_dir).await?;
        }
        Ok::<_, String>(mainnet_parent)
    }
    .await;

    let second_parent = match second_phase {
        Ok(parent) => parent,
        Err(error) => {
            let cleanup = cleanup_runtime_workers(second.session()).await;
            let shutdown = second.shutdown().await;
            return Err(format!(
                "{error}; cleanup={:?}; shutdown={:?}",
                cleanup.err(),
                shutdown.err()
            ));
        }
    };

    close_window_and_wait(second, &second_parent, &after_dir).await?;
    for (_, ports) in networks {
        wait_port("127.0.0.1", ports.rpc_port, false, Duration::from_secs(60)).await?;
        wait_port("127.0.0.1", ports.p2p_port, false, Duration::from_secs(60)).await?;
    }

    Ok(json!({
        "passed": true,
        "appBinary": args.app_binary,
        "webdriverPorts": [args.webdriver_port, second_port],
        "windowLabel": args.window_label,
        "networks": ["mainnet", "testnet10"],
        "firstParent": first_parent,
        "secondParent": second_parent,
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
        .map_err(|error| format!("failed to create app-close-relaunch runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args.output_directory.join("close-relaunch-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to serialize close/relaunch result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_requires_binary_and_output_directory() {
        let mut empty = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut empty).is_err());

        let mut args = vec![
            "--app-binary".to_owned(),
            "app.exe".to_owned(),
            "--output-directory".to_owned(),
            "out".to_owned(),
            "--port".to_owned(),
            "4455".to_owned(),
        ]
        .into_iter();
        let parsed = parse_args(&mut args).unwrap();
        assert_eq!(parsed.webdriver_port, 4455);
    }

    #[test]
    fn parent_identity_requires_complete_exact_identity() {
        let status = OwnerStatus {
            status: "test".to_owned(),
            pid: 42,
            fields: fields_map(
                r"parent_pid=7;parent_start_time=100;parent_executable=C:\gateway.exe",
            ),
        };
        assert_eq!(
            parent_identity(&status).unwrap(),
            ParentIdentity {
                pid: 7,
                start_time: 100,
                executable: r"C:\gateway.exe".to_owned(),
            }
        );

        let missing = OwnerStatus {
            status: "test".to_owned(),
            pid: 42,
            fields: fields_map("parent_pid=7;parent_start_time=100"),
        };
        assert!(parent_identity(&missing).is_err());
    }

    #[test]
    fn owner_wait_filter_rejects_stale_stopped_status() {
        let stale = fields_map("network=mainnet;role=node;running=false;readiness=FAILED;pid=77");
        assert!(!owner_matches_running_ready(&stale, "mainnet"));
        let ready = fields_map("network=mainnet;role=node;running=true;readiness=READY;pid=77");
        assert!(owner_matches_running_ready(&ready, "mainnet"));
    }
}
