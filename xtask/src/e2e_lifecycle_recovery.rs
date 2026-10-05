use crate::e2e_native_webdriver::{NativeWebDriverHarness, WebDriverSession};
use crate::e2e_owned_process::kill_exact_owned_process_checked;
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

const DEFAULT_WEBDRIVER_PORT: u16 = 4445;
const POLL_FAST: Duration = Duration::from_millis(250);
const POLL_OWNER: Duration = Duration::from_millis(750);

#[derive(Debug)]
struct Args {
    app_binary: PathBuf,
    output_directory: PathBuf,
    webdriver_port: u16,
    window_label: String,
    startup_timeout: Duration,
    networks: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
struct OwnerStatus {
    status: String,
    pid: u32,
    fields: BTreeMap<String, String>,
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
    let mut networks = vec!["mainnet", "testnet10"];

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
            "--network" => {
                networks = match value.as_str() {
                    "all" => vec!["mainnet", "testnet10"],
                    "mainnet" => vec!["mainnet"],
                    "testnet10" => vec!["testnet10"],
                    _ => {
                        return Err(format!(
                            "--network must be all, mainnet, or testnet10; got {value}"
                        ));
                    }
                };
            }
            _ => return Err(format!("unknown lifecycle-recovery argument: {flag}")),
        }
    }

    Ok(Args {
        app_binary: app_binary.ok_or_else(|| "--app-binary <path> is required".to_owned())?,
        output_directory: output_directory
            .ok_or_else(|| "--output-directory <path> is required".to_owned())?,
        webdriver_port,
        window_label,
        startup_timeout,
        networks,
    })
}

fn fields_map(text: &str) -> BTreeMap<String, String> {
    parse_key_value_line_native(text).into_iter().collect()
}

fn field<'a>(fields: &'a BTreeMap<String, String>, name: &str) -> &'a str {
    fields.get(name).map(String::as_str).unwrap_or("")
}

fn owner_matches_running_ready(fields: &BTreeMap<String, String>, network: &str) -> bool {
    let network_ok = field(fields, "network").is_empty()
        || field(fields, "network").eq_ignore_ascii_case(network);
    let role = if field(fields, "role").is_empty() {
        field(fields, "runtime_role")
    } else {
        field(fields, "role")
    };
    let role_ok = role.is_empty() || role.eq_ignore_ascii_case("node");
    network_ok
        && role_ok
        && field(fields, "running").eq_ignore_ascii_case("true")
        && field(fields, "readiness").eq_ignore_ascii_case("READY")
}

fn parse_positive_i64(value: &str, label: &str) -> Result<i64, String> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{label} is missing or invalid: {value:?}"))
}

fn assert_exact_owner(status: &OwnerStatus, network: &str) -> Result<(), String> {
    let fields = &status.fields;
    if !field(fields, "network").eq_ignore_ascii_case(network) {
        return Err(format!(
            "{network} owner network mismatch: {:?}",
            field(fields, "network")
        ));
    }
    if !field(fields, "role").eq_ignore_ascii_case("node") {
        return Err(format!(
            "{network} owner role mismatch: {:?}",
            field(fields, "role")
        ));
    }
    if !field(fields, "running").eq_ignore_ascii_case("true") {
        return Err(format!("{network} owner is not running: {}", status.status));
    }
    if !field(fields, "readiness").eq_ignore_ascii_case("READY") {
        return Err(format!("{network} owner is not READY: {}", status.status));
    }
    let worker_pid = field(fields, "worker_pid")
        .parse::<u32>()
        .map_err(|_| format!("{network} worker_pid missing: {}", status.status))?;
    if worker_pid != status.pid {
        return Err(format!(
            "{network} worker_pid {worker_pid} != status pid {}",
            status.pid
        ));
    }
    parse_positive_i64(field(fields, "worker_start_time"), "worker_start_time")?;
    if field(fields, "worker_executable").trim().is_empty() {
        return Err(format!("{network} worker_executable is missing"));
    }
    parse_positive_i64(field(fields, "parent_pid"), "parent_pid")?;
    parse_positive_i64(field(fields, "parent_start_time"), "parent_start_time")?;
    if field(fields, "parent_executable").trim().is_empty() {
        return Err(format!("{network} parent_executable is missing"));
    }
    Ok(())
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
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let result = session
            .execute_sync(
                r#"
const wanted = arguments[0];
const nextValue = String(arguments[1]);
const node = Array.from(document.querySelectorAll("[data-testid]"))
  .find((item) => item.getAttribute("data-testid") === wanted);
if (!node) return { ok: false, reason: "missing" };
if (node.disabled || node.readOnly) {
  return {
    ok: false,
    reason: "not-editable",
    disabled: Boolean(node.disabled),
    readOnly: Boolean(node.readOnly),
    title: String(node.title || ""),
    ariaReadOnly: String(node.getAttribute("aria-readonly") || ""),
    dataset: { ...node.dataset },
  };
}
node.value = nextValue;
node.dispatchEvent(new Event("input", { bubbles: true }));
node.dispatchEvent(new Event("change", { bubbles: true }));
return { ok: true, value: String(node.value || "") };
"#,
                vec![json!(test_id), json!(value)],
            )
            .await?;
        if result.get("ok").and_then(Value::as_bool) == Some(true) {
            if result.get("value").and_then(Value::as_str) != Some(value) {
                return Err(format!(
                    "{test_id} did not retain requested value {value:?}"
                ));
            }
            return Ok(());
        }
        let reason = result
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        if !matches!(reason, "missing" | "not-editable") || Instant::now() >= deadline {
            return Err(format!("unable to set {test_id}: {reason}; state={result}"));
        }
        sleep(POLL_FAST).await;
    }
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

async fn wait_for_owner_status(
    session: &WebDriverSession,
    network: &str,
    timeout_duration: Duration,
) -> Result<OwnerStatus, String> {
    let deadline = Instant::now() + timeout_duration;
    let mut last = String::new();
    loop {
        match invoke(
            session,
            "kgw_runtime_owner_status_v1",
            json!({ "network": network, "runtimeRole": "node" }),
        )
        .await
        {
            Ok(Value::String(status)) => {
                last.clone_from(&status);
                if let Some(pid) = pid_from_status_native(&status) {
                    let fields = fields_map(&status);
                    if owner_matches_running_ready(&fields, network) {
                        return Ok(OwnerStatus {
                            status,
                            pid,
                            fields,
                        });
                    }
                }
            }
            Ok(value) => last = format!("non-string owner status: {value}"),
            Err(error) => last = error,
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "owner status node/{network} did not become running: {last}"
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
        match invoke(
            session,
            "kgw_runtime_owner_status_v1",
            json!({ "network": network, "runtimeRole": "node" }),
        )
        .await
        {
            Ok(Value::String(status)) => {
                last.clone_from(&status);
                if is_stopped_owner_status_native(&status) {
                    return Ok(OwnerStatus {
                        pid: pid_from_status_native(&status).unwrap_or(0),
                        fields: fields_map(&status),
                        status,
                    });
                }
            }
            Ok(value) => last = format!("non-string stopped status: {value}"),
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
    label: &str,
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

    let status = wait_for_owner_status(session, network, Duration::from_secs(180)).await?;
    assert_exact_owner(&status, network)?;
    wait_port("127.0.0.1", ports.rpc_port, true, Duration::from_secs(180)).await?;
    wait_port("127.0.0.1", ports.p2p_port, true, Duration::from_secs(180)).await?;
    write_json(
        &output_directory.join(format!("{label}-owner-status.json")),
        &status,
    )?;
    Ok(status)
}

async fn stop_runtime(session: &WebDriverSession, network: &str) {
    let _ = open_node_settings(session, network).await;
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
        .await;
    let _ = invoke(
        session,
        "kgw_kgw_disable_network_v1",
        json!({ "network": network, "runtimeRole": "node" }),
    )
    .await;
}

async fn stop_and_verify(
    session: &WebDriverSession,
    network: &str,
    ports: NetworkPorts,
    output_directory: &Path,
    label: &str,
) -> Result<(), String> {
    stop_runtime(session, network).await;
    let stopped = wait_for_stopped(session, network, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.rpc_port, false, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.p2p_port, false, Duration::from_secs(60)).await?;
    let ui = wait_node_ui(session, network, false).await?;
    write_json(
        &output_directory.join(format!("{label}-stopped.json")),
        &json!({ "stopped": stopped, "ui": ui }),
    )
}

async fn shutdown_all_runtime_workers(session: &WebDriverSession) -> Result<Value, String> {
    invoke(session, "kgw_shutdown_all_runtime_workers_v1", json!({})).await
}

async fn exercise_recovery(
    session: &WebDriverSession,
    network: &str,
    ports: NetworkPorts,
    output_root: &Path,
) -> Result<Value, String> {
    let output_directory = output_root.join(format!("{network}-node-recovery"));
    fs::create_dir_all(&output_directory)
        .map_err(|error| format!("failed to create {}: {error}", output_directory.display()))?;

    let first =
        configure_and_start_node(session, network, ports, &output_directory, "start-1").await?;
    wait_node_ui(session, network, true).await?;
    stop_and_verify(session, network, ports, &output_directory, "normal-stop").await?;

    let second =
        configure_and_start_node(session, network, ports, &output_directory, "restart-2").await?;
    if second.pid == first.pid {
        return Err(format!(
            "{network} restart reused worker PID {}",
            second.pid
        ));
    }
    wait_node_ui(session, network, true).await?;

    let worker_executable = field(&second.fields, "worker_executable");
    let worker_start_time = parse_positive_i64(
        field(&second.fields, "worker_start_time"),
        "worker_start_time",
    )?;
    kill_exact_owned_process_checked(
        second.pid,
        worker_executable,
        worker_start_time,
        &output_directory.join("forced-crash.json"),
    )?;

    let reconciled = wait_for_stopped(session, network, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.rpc_port, false, Duration::from_secs(60)).await?;
    wait_port("127.0.0.1", ports.p2p_port, false, Duration::from_secs(60)).await?;
    let ui_after_crash = wait_node_ui(session, network, false).await?;
    write_json(
        &output_directory.join("crash-reconciled.json"),
        &json!({
            "killedPid": second.pid,
            "reconciled": reconciled,
            "ui": ui_after_crash,
        }),
    )?;

    let third =
        configure_and_start_node(session, network, ports, &output_directory, "recovery-3").await?;
    if third.pid == second.pid {
        return Err(format!(
            "{network} crash recovery reused worker PID {}",
            third.pid
        ));
    }
    wait_node_ui(session, network, true).await?;
    stop_and_verify(session, network, ports, &output_directory, "final-stop").await?;

    Ok(json!({
        "network": network,
        "passed": true,
        "firstPid": first.pid,
        "secondPid": second.pid,
        "thirdPid": third.pid,
        "rpcPort": ports.rpc_port,
        "p2pPort": ports.p2p_port,
    }))
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    fs::create_dir_all(&args.output_directory).map_err(|error| {
        format!(
            "failed to create output directory {}: {error}",
            args.output_directory.display()
        )
    })?;
    let profile = runtime_port_profile_native()?;
    let mut harness = NativeWebDriverHarness::launch(
        root,
        &args.app_binary,
        args.webdriver_port,
        &args.window_label,
        args.startup_timeout,
    )
    .await?;

    let result = async {
        let session = harness.session();
        let preflight_cleanup = shutdown_all_runtime_workers(session).await?;
        write_json(
            &args.output_directory.join("preflight-shutdown-all.json"),
            &preflight_cleanup,
        )?;
        let mut cases = Vec::new();
        for network in &args.networks {
            let ports = match *network {
                "mainnet" => profile.mainnet,
                "testnet10" => profile.testnet10,
                _ => unreachable!(),
            };
            let case_result =
                exercise_recovery(session, network, ports, &args.output_directory).await;
            let cleanup = shutdown_all_runtime_workers(session).await;
            match (case_result, cleanup) {
                (Ok(value), Ok(cleanup_value)) => {
                    write_json(
                        &args
                            .output_directory
                            .join(format!("{network}-shutdown-all.json")),
                        &cleanup_value,
                    )?;
                    cases.push(value);
                }
                (Err(error), Ok(_)) => return Err(error),
                (Ok(_), Err(error)) => {
                    return Err(format!("{network} cleanup failed: {error}"));
                }
                (Err(error), Err(cleanup_error)) => {
                    return Err(format!(
                        "{error}; {network} cleanup also failed: {cleanup_error}"
                    ));
                }
            }
        }
        Ok(json!({
            "passed": true,
            "appBinary": args.app_binary,
            "webdriverPort": args.webdriver_port,
            "windowLabel": args.window_label,
            "cases": cases,
        }))
    }
    .await;

    let shutdown = harness.shutdown().await;
    match (result, shutdown) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(shutdown_error)) => Err(format!(
            "{error}; app/session shutdown failed: {shutdown_error}"
        )),
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
        .map_err(|error| format!("failed to create lifecycle E2E runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args.output_directory.join("lifecycle-recovery-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to serialize lifecycle result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_parser_and_assertion_accept_exact_truth() {
        let status_text = "pid=42;network=mainnet;role=node;running=true;readiness=READY;worker_pid=42;worker_start_time=100;worker_executable=C:\\node.exe;parent_pid=7;parent_start_time=50;parent_executable=C:\\gateway.exe";
        let status = OwnerStatus {
            status: status_text.to_owned(),
            pid: 42,
            fields: fields_map(status_text),
        };
        assert_exact_owner(&status, "mainnet").unwrap();
    }

    #[test]
    fn owner_assertion_rejects_wrong_pid() {
        let status_text = "pid=42;network=mainnet;role=node;running=true;readiness=READY;worker_pid=43;worker_start_time=100;worker_executable=C:\\node.exe;parent_pid=7;parent_start_time=50;parent_executable=C:\\gateway.exe";
        let status = OwnerStatus {
            status: status_text.to_owned(),
            pid: 42,
            fields: fields_map(status_text),
        };
        assert!(assert_exact_owner(&status, "mainnet").is_err());
    }

    #[test]
    fn owner_wait_filter_rejects_stale_stopped_pid() {
        let stale = fields_map(
            "pid=17056;network=mainnet;role=node;running=false;readiness=FAILED;worker_pid=17056",
        );
        assert!(!owner_matches_running_ready(&stale, "mainnet"));

        let ready = fields_map(
            "pid=18000;network=mainnet;role=node;running=true;readiness=READY;worker_pid=18000",
        );
        assert!(owner_matches_running_ready(&ready, "mainnet"));
    }

    #[test]
    fn cli_requires_binary_and_output_directory() {
        let mut empty = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut empty).is_err());

        let mut args = vec![
            "--app-binary".to_owned(),
            "app.exe".to_owned(),
            "--output-directory".to_owned(),
            "out".to_owned(),
            "--network".to_owned(),
            "testnet10".to_owned(),
        ]
        .into_iter();
        let parsed = parse_args(&mut args).unwrap();
        assert_eq!(parsed.networks, vec!["testnet10"]);
    }
}
