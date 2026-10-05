use crate::e2e_native_webdriver::{NativeWebDriverHarness, WebDriverSession};
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
struct BridgeSelection {
    bridge_instance_id: Option<String>,
    bridge_port: Option<u16>,
    raw_port: String,
    bridge_level_port: String,
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
            _ => return Err(format!("unknown bridge-inprocess argument: {flag}")),
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
    let role_ok = role.is_empty() || role.eq_ignore_ascii_case("bridge");
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

fn assert_bridge_owner(
    status: &OwnerStatus,
    network: &str,
    profile: NetworkPorts,
) -> Result<(), String> {
    let fields = &status.fields;
    if !field(fields, "network").eq_ignore_ascii_case(network) {
        return Err(format!(
            "{network} bridge owner network mismatch: {:?}",
            field(fields, "network")
        ));
    }
    if !field(fields, "role").eq_ignore_ascii_case("bridge") {
        return Err(format!(
            "{network} bridge owner role mismatch: {:?}",
            field(fields, "role")
        ));
    }
    if !field(fields, "running").eq_ignore_ascii_case("true") {
        return Err(format!(
            "{network} bridge owner is not running: {}",
            status.status
        ));
    }
    if !field(fields, "readiness").eq_ignore_ascii_case("READY") {
        return Err(format!(
            "{network} bridge owner is not READY: {}",
            status.status
        ));
    }
    if !field(fields, "node_mode").eq_ignore_ascii_case("inprocess") {
        return Err(format!(
            "{network} bridge node_mode mismatch: {:?}",
            field(fields, "node_mode")
        ));
    }
    if !field(fields, "node_kind").eq_ignore_ascii_case("integrated-inproc") {
        return Err(format!(
            "{network} bridge node_kind mismatch: {:?}",
            field(fields, "node_kind")
        ));
    }
    if !field(fields, "bridge_kind").eq_ignore_ascii_case("official-inprocess-node") {
        return Err(format!(
            "{network} bridge_kind mismatch: {:?}",
            field(fields, "bridge_kind")
        ));
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
    if !profile.external_bridge_listeners
        && (!field(fields, "listener").is_empty()
            || !field(fields, "prometheus_listener").is_empty()
            || field(fields, "listener_count")
                .parse::<u64>()
                .unwrap_or_default()
                != 0)
    {
        return Err(format!(
            "{network} CPU-only bridge unexpectedly reports external listeners: {}",
            status.status
        ));
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
    selector: &str,
    value: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let result = session
            .execute_sync(
                r#"
const selector = arguments[0];
const nextValue = String(arguments[1]);
const node = document.querySelector(selector);
if (!node) return { ok: false, reason: "missing" };
if (node.disabled || node.readOnly) return { ok: false, reason: "not-editable" };
node.value = nextValue;
node.dispatchEvent(new Event("input", { bubbles: true }));
node.dispatchEvent(new Event("change", { bubbles: true }));
return { ok: true, value: String(node.value || "") };
"#,
                vec![json!(selector), json!(value)],
            )
            .await?;
        if result.get("ok").and_then(Value::as_bool) == Some(true) {
            if result.get("value").and_then(Value::as_str) != Some(value) {
                return Err(format!(
                    "{selector} did not retain requested value {value:?}"
                ));
            }
            return Ok(());
        }
        let reason = result
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        if !matches!(reason, "missing" | "not-editable") || Instant::now() >= deadline {
            return Err(format!("unable to set {selector}: {reason}"));
        }
        sleep(POLL_FAST).await;
    }
}

async fn activate_top_tab(
    session: &WebDriverSession,
    test_id: &str,
    tab: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let result = session
            .execute_sync(
                r#"
const wanted = arguments[0];
const node = Array.from(document.querySelectorAll("[data-testid]"))
  .find((item) => item.getAttribute("data-testid") === wanted);
return {
  present: Boolean(node),
  bound: Boolean(node?.dataset?.kgwBound === "true"),
  disabled: Boolean(node?.disabled || node?.getAttribute("aria-disabled") === "true"),
};
"#,
                vec![json!(test_id)],
            )
            .await?;
        let ready = result.get("present").and_then(Value::as_bool) == Some(true)
            && result.get("bound").and_then(Value::as_bool) == Some(true)
            && result.get("disabled").and_then(Value::as_bool) == Some(false);
        if ready {
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "top tab {test_id} did not become bound and enabled: {result}"
            ));
        }
        sleep(POLL_FAST).await;
    }

    click_test_id(session, test_id).await?;
    loop {
        let result = session
            .execute_sync(
                r#"
const expected = arguments[0];
const active = document.querySelector("[data-tab].active")?.getAttribute("data-tab") || "";
const panel = document.getElementById(expected);
return {
  active,
  panelExists: Boolean(panel),
  panelRendered: Boolean(panel && String(panel.innerHTML || "").trim().length > 0),
};
"#,
                vec![json!(tab)],
            )
            .await?;
        let active = result.get("active").and_then(Value::as_str) == Some(tab);
        let rendered = result.get("panelRendered").and_then(Value::as_bool) == Some(true);
        if active && rendered {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "top tab {test_id} did not become active/rendered for {tab}: {result}"
            ));
        }
        sleep(POLL_FAST).await;
    }
}

async fn open_bridge_settings(session: &WebDriverSession, network: &str) -> Result<(), String> {
    activate_top_tab(session, "kgw-tab-kaspa-bridge", "kaspa-bridge").await?;
    if let Err(error) = click_test_id(session, &format!("kgw-bridge-network-{network}")).await {
        let diagnostics = session
            .execute_sync(
                r#"
return {
  activeTab: document.querySelector("[data-tab].active")?.getAttribute("data-tab") || "",
  bridgePanelExists: Boolean(document.getElementById("kaspa-bridge")),
  bridgePanelHtml: String(document.getElementById("kaspa-bridge")?.innerHTML || "").slice(0, 4000),
  bufferedLogs: typeof window.kgwGetBufferedLogs === "function"
    ? window.kgwGetBufferedLogs().slice(-40)
    : [],
};
"#,
                Vec::new(),
            )
            .await
            .unwrap_or_else(|diag_error| json!({ "diagnosticError": diag_error }));
        return Err(format!("{error}; bridge diagnostics={diagnostics}"));
    }
    click_test_id(session, &format!("kgw-bridge-settings-{network}")).await
}

async fn ensure_inprocess_option_enabled(
    session: &WebDriverSession,
    network: &str,
    name: &str,
) -> Result<(), String> {
    let result = session
        .execute_sync(
            r#"
const net = arguments[0];
const optionName = arguments[1];
const selector =
  '[data-bridge-command-option-toggle-r7="' + optionName + '"][data-net="' + net + '"]';
const toggle = document.querySelector(selector);
if (!toggle) return { ok: false, reason: "missing" };
if (!toggle.checked) toggle.click();
return { ok: Boolean(toggle.checked), checked: Boolean(toggle.checked) };
"#,
            vec![json!(network), json!(name)],
        )
        .await?;
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "bridge command option {name}/{network} is not enabled: {result}"
        ));
    }
    Ok(())
}

async fn wait_for_inprocess_controls(
    session: &WebDriverSession,
    network: &str,
) -> Result<Value, String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let result = session
            .execute_sync(
                r#"
const net = arguments[0];
const section = document.querySelector(
  '[data-bridge-inprocess-node-settings="' + net + '"]'
);
const rpc = document.querySelector('#bridge-' + net + '-inprocessRpcListen');
const p2p = document.querySelector('#bridge-' + net + '-inprocessListen');
return {
  active: section?.dataset?.kgwInprocessNodeActive === "true",
  rpcPresent: Boolean(rpc),
  p2pPresent: Boolean(p2p),
  rpcDisabled: Boolean(rpc?.disabled),
  p2pDisabled: Boolean(p2p?.disabled),
  rpc: String(rpc?.value || ""),
  p2p: String(p2p?.value || ""),
};
"#,
                vec![json!(network)],
            )
            .await?;
        let ready = result.get("active").and_then(Value::as_bool) == Some(true)
            && result.get("rpcPresent").and_then(Value::as_bool) == Some(true)
            && result.get("p2pPresent").and_then(Value::as_bool) == Some(true)
            && result.get("rpcDisabled").and_then(Value::as_bool) == Some(false)
            && result.get("p2pDisabled").and_then(Value::as_bool) == Some(false);
        if ready {
            return Ok(result);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "in-process bridge controls {network} did not become editable: {result}"
            ));
        }
        sleep(POLL_FAST).await;
    }
}

async fn read_bridge_runtime_selection(
    session: &WebDriverSession,
    network: &str,
) -> Result<BridgeSelection, String> {
    let result = session
        .execute_sync(
            r#"
const net = arguments[0];
const panel = document.querySelector('[data-testid="kgw-bridge-panel-' + net + '"]');
const active =
  panel?.querySelector?.('[data-bridge-action="select-instance"].active') ||
  panel?.querySelector?.('[data-bridge-action="select-instance"]');
const bridgeInstanceId =
  active?.dataset?.instanceId ? String(active.dataset.instanceId) : null;
const input = bridgeInstanceId
  ? panel?.querySelector?.('#bridge-' + net + '-instancePort-' + bridgeInstanceId) ||
    panel?.querySelector?.(
      '[data-testid="kgw-bridge-instance-field-' +
      net + '-' + bridgeInstanceId + '-instancePort"]'
    )
  : null;
const bridgeLevel =
  panel?.querySelector?.('[data-testid="kgw-bridge-field-' + net + '-stratumPort"]');
const rawPort = String(input?.value || input?.placeholder || "")
  .trim()
  .replace(/^:/, "");
const port = Number(rawPort);
return {
  bridgeInstanceId,
  bridgePort: Number.isInteger(port) && port > 0 ? port : null,
  rawPort,
  bridgeLevelPort: String(bridgeLevel?.value || ""),
};
"#,
            vec![json!(network)],
        )
        .await?;

    Ok(BridgeSelection {
        bridge_instance_id: result
            .get("bridgeInstanceId")
            .and_then(Value::as_str)
            .map(str::to_owned),
        bridge_port: result
            .get("bridgePort")
            .and_then(Value::as_u64)
            .and_then(|value| u16::try_from(value).ok()),
        raw_port: result
            .get("rawPort")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        bridge_level_port: result
            .get("bridgeLevelPort")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
    })
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
            json!({ "network": network, "runtimeRole": "bridge" }),
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
                "owner status bridge/{network} did not become READY: {last}"
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
            json!({ "network": network, "runtimeRole": "bridge" }),
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
                "owner status bridge/{network} did not reconcile to stopped: {last}"
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

async fn stop_runtime(session: &WebDriverSession, network: &str) {
    let _ = open_bridge_settings(session, network).await;
    let _ = session
        .execute_sync(
            r#"
const net = arguments[0];
const stop = document.querySelector('[data-testid="kgw-bridge-stop-' + net + '"]');
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
        json!({ "network": network, "runtimeRole": "bridge" }),
    )
    .await;
}

async fn shutdown_all_runtime_workers(session: &WebDriverSession) -> Result<Value, String> {
    invoke(session, "kgw_shutdown_all_runtime_workers_v1", json!({})).await
}

async fn configure_and_start_inprocess_bridge(
    session: &WebDriverSession,
    network: &str,
    profile: NetworkPorts,
    output_directory: &Path,
) -> Result<Value, String> {
    open_bridge_settings(session, network).await?;
    set_control_value(
        session,
        &format!("[data-testid=\"kgw-bridge-field-{network}-nodeMode\"]"),
        "inprocess",
    )
    .await?;

    ensure_inprocess_option_enabled(session, network, "inprocessListen").await?;
    let controls = wait_for_inprocess_controls(session, network).await?;
    set_control_value(
        session,
        &format!("#bridge-{network}-inprocessRpcListen"),
        &format!("127.0.0.1:{}", profile.rpc_port),
    )
    .await?;
    set_control_value(
        session,
        &format!("#bridge-{network}-inprocessListen"),
        &format!("127.0.0.1:{}", profile.p2p_port),
    )
    .await?;

    let mut selection = read_bridge_runtime_selection(session, network).await?;
    if profile.external_bridge_listeners {
        let instance_id = selection
            .bridge_instance_id
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{network} listener-enabled Bridge instance id is missing"))?;
        set_control_value(
            session,
            &format!("#bridge-{network}-instancePort-{instance_id}"),
            &profile.bridge_port.to_string(),
        )
        .await?;
        selection = read_bridge_runtime_selection(session, network).await?;
        if selection.bridge_port != Some(profile.bridge_port) {
            return Err(format!(
                "{network} bridge instance port did not retain isolated value {}; selection={selection:?}",
                profile.bridge_port
            ));
        }
    } else if selection.bridge_instance_id.is_some() || selection.bridge_port.is_some() {
        return Err(format!(
            "{network} CPU-only Bridge exposed ASIC instance/listener: {selection:?}"
        ));
    }

    click_test_id(session, &format!("kgw-bridge-start-{network}")).await?;
    let status = wait_for_owner_status(session, network, Duration::from_secs(180)).await?;
    assert_bridge_owner(&status, network, profile)?;

    wait_port(
        "127.0.0.1",
        profile.rpc_port,
        true,
        Duration::from_secs(180),
    )
    .await?;
    wait_port(
        "127.0.0.1",
        profile.p2p_port,
        true,
        Duration::from_secs(180),
    )
    .await?;
    if profile.external_bridge_listeners {
        wait_port(
            "127.0.0.1",
            profile.bridge_port,
            true,
            Duration::from_secs(180),
        )
        .await?;
    }

    write_json(&output_directory.join("owner-status.json"), &status)?;
    write_json(
        &output_directory.join("ports-ready.json"),
        &json!({
            "rpc": profile.rpc_port,
            "p2p": profile.p2p_port,
            "stratum": if profile.external_bridge_listeners {
                Some(profile.bridge_port)
            } else {
                None
            },
            "selection": selection,
            "inprocessControls": controls,
        }),
    )?;

    Ok(json!({
        "owner": status,
        "selection": selection,
        "rpcPort": profile.rpc_port,
        "p2pPort": profile.p2p_port,
        "bridgePort": if profile.external_bridge_listeners {
            Some(profile.bridge_port)
        } else {
            None
        },
    }))
}

async fn stop_and_verify_inprocess_bridge(
    session: &WebDriverSession,
    network: &str,
    profile: NetworkPorts,
    output_directory: &Path,
) -> Result<Value, String> {
    stop_runtime(session, network).await;
    let stopped = wait_for_stopped(session, network, Duration::from_secs(60)).await?;
    if profile.external_bridge_listeners {
        wait_port(
            "127.0.0.1",
            profile.bridge_port,
            false,
            Duration::from_secs(60),
        )
        .await?;
    }
    wait_port(
        "127.0.0.1",
        profile.rpc_port,
        false,
        Duration::from_secs(60),
    )
    .await?;
    wait_port(
        "127.0.0.1",
        profile.p2p_port,
        false,
        Duration::from_secs(60),
    )
    .await?;
    let value = json!({
        "stopped": stopped,
        "portsFree": {
            "rpc": profile.rpc_port,
            "p2p": profile.p2p_port,
            "stratum": if profile.external_bridge_listeners {
                Some(profile.bridge_port)
            } else {
                None
            }
        }
    });
    write_json(&output_directory.join("stopped.json"), &value)?;
    Ok(value)
}

async fn exercise_inprocess_bridge(
    session: &WebDriverSession,
    network: &str,
    profile: NetworkPorts,
    output_root: &Path,
) -> Result<Value, String> {
    let output_directory = output_root.join(format!("{network}-bridge-inprocess"));
    fs::create_dir_all(&output_directory)
        .map_err(|error| format!("failed to create {}: {error}", output_directory.display()))?;

    let start =
        configure_and_start_inprocess_bridge(session, network, profile, &output_directory).await;
    let stop = stop_and_verify_inprocess_bridge(session, network, profile, &output_directory).await;

    match (start, stop) {
        (Ok(start), Ok(stop)) => Ok(json!({
            "network": network,
            "passed": true,
            "start": start,
            "stop": stop,
        })),
        (Err(error), Ok(_)) => Err(error),
        (Ok(_), Err(error)) => Err(format!("{network} stop verification failed: {error}")),
        (Err(error), Err(stop_error)) => Err(format!(
            "{error}; {network} stop verification also failed: {stop_error}"
        )),
    }
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    fs::create_dir_all(&args.output_directory).map_err(|error| {
        format!(
            "failed to create output directory {}: {error}",
            args.output_directory.display()
        )
    })?;
    let ports = runtime_port_profile_native()?;
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
        let mut cases = Vec::new();
        for network in &args.networks {
            let profile = match *network {
                "mainnet" => ports.mainnet,
                "testnet10" => ports.testnet10,
                _ => unreachable!(),
            };
            let case_result =
                exercise_inprocess_bridge(session, network, profile, &args.output_directory).await;
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
        .map_err(|error| format!("failed to create bridge-inprocess E2E runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args.output_directory.join("bridge-inprocess-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to serialize bridge-inprocess result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_parser_and_assertion_accept_exact_bridge_truth() {
        let status_text = "pid=42;network=mainnet;role=bridge;running=true;readiness=READY;node_mode=inprocess;node_kind=integrated-inproc;bridge_kind=official-inprocess-node;worker_pid=42;worker_start_time=100;worker_executable=C:\\bridge.exe;listener=127.0.0.1:5556;listener_count=1";
        let status = OwnerStatus {
            status: status_text.to_owned(),
            pid: 42,
            fields: fields_map(status_text),
        };
        let profile = NetworkPorts {
            rpc_port: 16110,
            p2p_port: 16111,
            bridge_port: 5556,
            external_bridge_listeners: true,
        };
        assert_bridge_owner(&status, "mainnet", profile).unwrap();
    }
    #[test]
    fn cpu_only_owner_rejects_external_listener() {
        let status_text = "pid=42;network=testnet10;role=bridge;running=true;readiness=READY;node_mode=inprocess;node_kind=integrated-inproc;bridge_kind=official-inprocess-node;worker_pid=42;worker_start_time=100;worker_executable=C:\\bridge.exe;listener=127.0.0.1:5656;listener_count=1";
        let status = OwnerStatus {
            status: status_text.to_owned(),
            pid: 42,
            fields: fields_map(status_text),
        };
        let profile = NetworkPorts {
            rpc_port: 16210,
            p2p_port: 16211,
            bridge_port: 5656,
            external_bridge_listeners: false,
        };
        assert!(assert_bridge_owner(&status, "testnet10", profile).is_err());
    }

    #[test]
    fn owner_wait_filter_rejects_stale_stopped_bridge() {
        let stale = fields_map(
            "pid=17056;network=mainnet;role=bridge;running=false;readiness=FAILED;worker_pid=17056",
        );
        assert!(!owner_matches_running_ready(&stale, "mainnet"));
        let ready = fields_map(
            "pid=18000;network=mainnet;role=bridge;running=true;readiness=READY;worker_pid=18000",
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
