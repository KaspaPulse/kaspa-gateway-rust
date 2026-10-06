use crate::e2e_native_webdriver::{NativeWebDriverHarness, WebDriverSession};
use serde::Serialize;
use serde_json::{Value, json};
use std::fs;
use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const NODE_RECOVERY_RUNS: usize = 4;
const BRIDGE_RUNS: usize = 10;
const APP_RELAUNCH_RUNS: usize = 10;
const TAB_SWITCHES: usize = 100;

const NODE_START_STOP_MIN: usize = 20;
const NODE_RESTART_MIN: usize = 10;
const BRIDGE_START_STOP_MIN: usize = 20;
const BRIDGE_RECONNECT_MIN: usize = 10;
const APP_RELAUNCH_MIN: usize = 10;

#[derive(Debug)]
struct Args {
    app_binary: PathBuf,
    output_directory: PathBuf,
    webdriver_port_base: u16,
    startup_timeout_seconds: u64,
}

#[derive(Debug, Serialize)]
struct StressReport {
    schema_version: u32,
    passed: bool,
    app_binary: PathBuf,
    node_start_stop_cycles: usize,
    node_restart_cycles: usize,
    bridge_start_stop_cycles: usize,
    bridge_reconnect_cycles: usize,
    app_relaunch_cycles: usize,
    tab_switches: usize,
    node_runs: Vec<Value>,
    bridge_runs: Vec<Value>,
    app_relaunch_runs: Vec<Value>,
    tab_switch_report: Value,
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<String, String> {
    let args = parse_args(args)?;
    fs::create_dir_all(&args.output_directory).map_err(|error| {
        format!(
            "lifecycle-stress: create {}: {error}",
            args.output_directory.display()
        )
    })?;

    let mut node_runs = Vec::with_capacity(NODE_RECOVERY_RUNS);
    for index in 0..NODE_RECOVERY_RUNS {
        let output = args
            .output_directory
            .join(format!("node-recovery-{index:02}"));
        let port = checked_available_port(args.webdriver_port_base, index, 1)?;
        let mut inner = common_args(&args, &output, port);
        inner.extend(["--network".to_owned(), "all".to_owned()]);
        let rendered = crate::e2e_lifecycle_recovery::run_cli(&mut inner.into_iter(), root)
            .map_err(|error| format!("lifecycle-stress: node recovery run {index}: {error}"))?;
        node_runs.push(parse_passed_json(&rendered, "node recovery", index)?);
    }

    let bridge_base = checked_port(args.webdriver_port_base, 20)?;
    let mut bridge_runs = Vec::with_capacity(BRIDGE_RUNS);
    for index in 0..BRIDGE_RUNS {
        let output = args.output_directory.join(format!("bridge-{index:02}"));
        let port = checked_available_port(bridge_base, index, 1)?;
        let mut inner = common_args(&args, &output, port);
        inner.extend(["--network".to_owned(), "all".to_owned()]);
        let rendered = crate::e2e_bridge_inprocess::run_cli(&mut inner.into_iter(), root)
            .map_err(|error| format!("lifecycle-stress: bridge run {index}: {error}"))?;
        bridge_runs.push(parse_passed_json(&rendered, "bridge", index)?);
    }

    let relaunch_base = checked_port(args.webdriver_port_base, 40)?;
    let mut app_relaunch_runs = Vec::with_capacity(APP_RELAUNCH_RUNS);
    for index in 0..APP_RELAUNCH_RUNS {
        let output = args
            .output_directory
            .join(format!("app-relaunch-{index:02}"));
        let port = checked_available_port(relaunch_base, index * 2, 2)?;
        let inner = common_args(&args, &output, port);
        let rendered = crate::e2e_app_close_relaunch::run_cli(&mut inner.into_iter(), root)
            .map_err(|error| format!("lifecycle-stress: app relaunch run {index}: {error}"))?;
        app_relaunch_runs.push(parse_passed_json(&rendered, "app relaunch", index)?);
    }

    let tab_port = checked_available_port(args.webdriver_port_base, 70, 1)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("lifecycle-stress: create tab runtime: {error}"))?;
    let tab_switch_report = runtime.block_on(tab_switch_stress(
        root,
        &args.app_binary,
        tab_port,
        args.startup_timeout_seconds,
    ))?;

    // Each node recovery run exercises three exact node starts per network:
    // initial start, normal restart, and crash recovery. With both networks this
    // is six complete lifecycle terminations per run and four restart/recovery
    // transitions per run.
    let node_start_stop_cycles = NODE_RECOVERY_RUNS * 6;
    let node_restart_cycles = NODE_RECOVERY_RUNS * 4;

    // Each bridge run covers one complete in-process bridge start/stop per
    // Mainnet and Testnet10. All runs after the first are reconnect cycles.
    let bridge_start_stop_cycles = BRIDGE_RUNS * 2;
    let bridge_reconnect_cycles = BRIDGE_RUNS.saturating_sub(1) * 2;
    let app_relaunch_cycles = APP_RELAUNCH_RUNS;

    let passed = node_start_stop_cycles >= NODE_START_STOP_MIN
        && node_restart_cycles >= NODE_RESTART_MIN
        && bridge_start_stop_cycles >= BRIDGE_START_STOP_MIN
        && bridge_reconnect_cycles >= BRIDGE_RECONNECT_MIN
        && app_relaunch_cycles >= APP_RELAUNCH_MIN
        && tab_switch_report
            .get("switches")
            .and_then(Value::as_u64)
            .is_some_and(|value| value >= TAB_SWITCHES as u64);

    let report = StressReport {
        schema_version: 1,
        passed,
        app_binary: args.app_binary,
        node_start_stop_cycles,
        node_restart_cycles,
        bridge_start_stop_cycles,
        bridge_reconnect_cycles,
        app_relaunch_cycles,
        tab_switches: TAB_SWITCHES,
        node_runs,
        bridge_runs,
        app_relaunch_runs,
        tab_switch_report,
    };
    write_json(
        &args.output_directory.join("lifecycle-stress-result.json"),
        &report,
    )?;
    let rendered = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("lifecycle-stress: serialize result: {error}"))?;
    if passed {
        Ok(rendered)
    } else {
        Err(format!("LIFECYCLE_STRESS=FAIL\n{rendered}"))
    }
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut app_binary = None;
    let mut output_directory = None;
    let mut webdriver_port_base = 4600_u16;
    let mut startup_timeout_seconds = 120_u64;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("lifecycle-stress: {flag} requires a value"))?;
        match flag.as_str() {
            "--app-binary" => app_binary = Some(PathBuf::from(value)),
            "--output-directory" => output_directory = Some(PathBuf::from(value)),
            "--port-base" => {
                webdriver_port_base = value
                    .parse::<u16>()
                    .ok()
                    .filter(|value| (1024..=65000).contains(value))
                    .ok_or_else(|| format!("lifecycle-stress: invalid port base {value}"))?;
            }
            "--startup-timeout-seconds" => {
                startup_timeout_seconds = value
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0 && *value <= 300)
                    .ok_or_else(|| format!("lifecycle-stress: invalid startup timeout {value}"))?;
            }
            _ => return Err(format!("lifecycle-stress: unknown argument {flag}")),
        }
    }
    Ok(Args {
        app_binary: app_binary
            .ok_or_else(|| "lifecycle-stress: --app-binary is required".to_owned())?,
        output_directory: output_directory
            .ok_or_else(|| "lifecycle-stress: --output-directory is required".to_owned())?,
        webdriver_port_base,
        startup_timeout_seconds,
    })
}

fn common_args(args: &Args, output: &Path, port: u16) -> Vec<String> {
    vec![
        "--app-binary".to_owned(),
        args.app_binary.to_string_lossy().into_owned(),
        "--output-directory".to_owned(),
        output.to_string_lossy().into_owned(),
        "--port".to_owned(),
        port.to_string(),
        "--window-label".to_owned(),
        "main".to_owned(),
        "--startup-timeout-seconds".to_owned(),
        args.startup_timeout_seconds.to_string(),
    ]
}

fn checked_port(base: u16, offset: usize) -> Result<u16, String> {
    let offset: u16 = offset
        .try_into()
        .map_err(|_| "lifecycle-stress: port offset overflow".to_owned())?;
    base.checked_add(offset)
        .filter(|port| *port <= 65534)
        .ok_or_else(|| "lifecycle-stress: WebDriver port range overflow".to_owned())
}

fn span_is_available(start: u16, width: u16) -> bool {
    if width == 0 {
        return false;
    }
    let Some(last) = start.checked_add(width.saturating_sub(1)) else {
        return false;
    };
    if last > 65534 {
        return false;
    }

    let mut listeners = Vec::with_capacity(width as usize);
    for offset in 0..width {
        let Some(port) = start.checked_add(offset) else {
            return false;
        };
        match TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
            Ok(listener) => listeners.push(listener),
            Err(_) => return false,
        }
    }
    true
}

fn checked_available_port(base: u16, offset: usize, width: u16) -> Result<u16, String> {
    let nominal = checked_port(base, offset)?;
    for delta in 0..=128_u16 {
        let Some(candidate) = nominal.checked_add(delta) else {
            break;
        };
        if span_is_available(candidate, width) {
            return Ok(candidate);
        }
    }
    Err(format!(
        "lifecycle-stress: no free WebDriver port span width={width} at/after {nominal}"
    ))
}

fn parse_passed_json(rendered: &str, label: &str, index: usize) -> Result<Value, String> {
    let value: Value = serde_json::from_str(rendered)
        .map_err(|error| format!("lifecycle-stress: {label} run {index} invalid JSON: {error}"))?;
    if value.get("passed").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "lifecycle-stress: {label} run {index} did not report passed=true"
        ));
    }
    Ok(value)
}

async fn activate_top_tab(
    session: &WebDriverSession,
    test_id: &str,
    tab: &str,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let state = session
            .execute_sync(
                r#"
const wanted = arguments[0];
const node = document.querySelector('[data-testid="' + wanted + '"]');
return {
  present: Boolean(node),
  bound: Boolean(node?.dataset?.kgwBound === "true"),
  disabled: Boolean(node?.disabled || node?.getAttribute("aria-disabled") === "true"),
};
"#,
                vec![json!(test_id)],
            )
            .await?;
        let ready = state.get("present").and_then(Value::as_bool) == Some(true)
            && state.get("bound").and_then(Value::as_bool) == Some(true)
            && state.get("disabled").and_then(Value::as_bool) == Some(false);
        if ready {
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "lifecycle-stress: tab {test_id} did not become bound/enabled: {state}"
            ));
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let clicked = session
        .execute_sync(
            r#"
const wanted = arguments[0];
const node = document.querySelector('[data-testid="' + wanted + '"]');
if (!node) return false;
node.click();
return true;
"#,
            vec![json!(test_id)],
        )
        .await?;
    if clicked.as_bool() != Some(true) {
        return Err(format!(
            "lifecycle-stress: tab {test_id} click was not dispatched"
        ));
    }

    loop {
        let state = session
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
        let active = state.get("active").and_then(Value::as_str) == Some(tab);
        let rendered = state.get("panelRendered").and_then(Value::as_bool) == Some(true);
        if active && rendered {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "lifecycle-stress: tab {test_id} did not become active/rendered for {tab}: {state}"
            ));
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn tab_switch_stress(
    root: &Path,
    app_binary: &Path,
    port: u16,
    startup_timeout_seconds: u64,
) -> Result<Value, String> {
    let mut harness = NativeWebDriverHarness::launch(
        root,
        app_binary,
        port,
        "main",
        Duration::from_secs(startup_timeout_seconds),
    )
    .await?;
    let result = async {
        let session = harness.session();
        for index in 0..TAB_SWITCHES {
            let (test_id, tab) = if index % 2 == 0 {
                ("kgw-tab-kaspa-node", "kaspa-node")
            } else {
                ("kgw-tab-kaspa-bridge", "kaspa-bridge")
            };
            activate_top_tab(session, test_id, tab)
                .await
                .map_err(|error| format!("lifecycle-stress: tab switch {index}: {error}"))?;
        }
        Ok(json!({"passed":true,"switches":TAB_SWITCHES}))
    }
    .await;
    let shutdown = harness.shutdown().await;
    match (result, shutdown) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(format!("lifecycle-stress: tab harness shutdown: {error}")),
        (Err(error), Err(shutdown_error)) => Err(format!(
            "{error}; lifecycle-stress tab harness shutdown also failed: {shutdown_error}"
        )),
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("lifecycle-stress: create {}: {error}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("lifecycle-stress: serialize {}: {error}", path.display()))?;
    fs::write(path, bytes)
        .map_err(|error| format!("lifecycle-stress: write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_counts_meet_release_contract() {
        let node_runs = std::hint::black_box(NODE_RECOVERY_RUNS);
        let bridge_runs = std::hint::black_box(BRIDGE_RUNS);
        let relaunch_runs = std::hint::black_box(APP_RELAUNCH_RUNS);
        let tab_switches = std::hint::black_box(TAB_SWITCHES);
        assert!(node_runs * 6 >= NODE_START_STOP_MIN);
        assert!(node_runs * 4 >= NODE_RESTART_MIN);
        assert!(bridge_runs * 2 >= BRIDGE_START_STOP_MIN);
        assert!(bridge_runs.saturating_sub(1) * 2 >= BRIDGE_RECONNECT_MIN);
        assert!(relaunch_runs >= APP_RELAUNCH_MIN);
        assert!(tab_switches >= 100);
    }

    #[test]
    fn parser_is_fail_closed() {
        let mut missing = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut missing).is_err());

        let mut valid = vec![
            "--app-binary".to_owned(),
            "app.exe".to_owned(),
            "--output-directory".to_owned(),
            "out".to_owned(),
            "--port-base".to_owned(),
            "4600".to_owned(),
        ]
        .into_iter();
        let args = parse_args(&mut valid).expect("valid args");
        assert_eq!(args.webdriver_port_base, 4600);
    }

    #[test]
    fn port_allocator_skips_occupied_listener() {
        let (occupied, _listener) = (43000_u16..44000)
            .find_map(|port| {
                TcpListener::bind((Ipv4Addr::LOCALHOST, port))
                    .ok()
                    .map(|listener| (port, listener))
            })
            .expect("test requires one free loopback port");
        assert!(!span_is_available(occupied, 1));
        let selected = checked_available_port(occupied, 0, 1).expect("find replacement port");
        assert_ne!(selected, occupied);
        assert!(span_is_available(selected, 1));
    }

    #[test]
    fn port_allocator_reserves_relaunch_pair() {
        let (occupied, _listener) = (44000_u16..45000)
            .find_map(|port| {
                TcpListener::bind((Ipv4Addr::LOCALHOST, port))
                    .ok()
                    .map(|listener| (port, listener))
            })
            .expect("test requires one free loopback port");
        let selected = checked_available_port(occupied, 0, 2).expect("find replacement pair");
        assert_ne!(selected, occupied);
        assert!(span_is_available(selected, 2));
    }

    #[test]
    fn port_ranges_are_bounded() {
        assert_eq!(checked_port(4600, 70).unwrap(), 4670);
        assert!(checked_port(65500, 70).is_err());
    }
}
