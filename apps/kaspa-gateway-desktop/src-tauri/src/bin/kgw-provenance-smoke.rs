// This standalone validation binary intentionally path-includes the private
// desktop runtime module so it exercises the same parent/child IPC path without
// exposing production internals from the library API.
#[allow(dead_code)]
#[path = "../integrated_runtime_commands.rs"]
mod integrated_runtime_commands;

use integrated_runtime_commands::{
    KgwRuntimeLogsReportV1, kgw_kgw_apply_node_settings_v1, kgw_kgw_disable_network_v1,
    kgw_kgw_runtime_logs_v1, kgw_runtime_owner_status_v1, kgw_shutdown_all_runtime_workers_v1,
};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SmokeResult {
    network: String,
    runtime_role: String,
    prerequisite_node_start: Option<String>,
    start_result: Option<String>,
    start_error: Option<String>,
    management_diagnostic: Option<String>,
    status: Option<String>,
    raw_report: KgwRuntimeLogsReportV1,
    stop_result: Option<String>,
}

fn argument_value(args: &[String], key: &str) -> Result<String, String> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].clone())
        .ok_or_else(|| format!("missing required argument {key}"))
}

fn main() -> Result<(), String> {
    if kaspa_gateway_desktop_lib::try_run_kgw_self_worker_from_args() {
        return Ok(());
    }

    let args = std::env::args().collect::<Vec<_>>();
    let network = argument_value(&args, "--network")?;
    let runtime_role = argument_value(&args, "--runtime-role")?;
    let rpc = argument_value(&args, "--rpc")?;
    let p2p = argument_value(&args, "--p2p")?;
    let stratum = argument_value(&args, "--stratum")?;
    let appdir = PathBuf::from(argument_value(&args, "--appdir")?);
    if !appdir.is_absolute() {
        return Err("--appdir must be absolute".to_string());
    }

    let _ = kgw_shutdown_all_runtime_workers_v1();

    let prerequisite_node_start = if runtime_role == "bridge" {
        Some(kgw_kgw_apply_node_settings_v1(
            network.clone(),
            "integrated-inproc".to_string(),
            "disable".to_string(),
            Some(format!(
                "kaspad --rpclisten {rpc} --listen {p2p} --appdir {}",
                appdir.display()
            )),
            None,
            Some("node".to_string()),
            None,
            None,
            None,
            None,
            Some(false),
        )?)
    } else {
        None
    };

    let (node_kind, bridge_kind, node_preview, bridge_preview) = if runtime_role == "bridge" {
        (
            "remote",
            "official-external-node",
            None,
            Some(format!(
                "stratum-bridge --kaspa-rpc {rpc} --stratum-listen {stratum} --appdir {} --node-mode external",
                appdir.display()
            )),
        )
    } else {
        (
            "integrated-inproc",
            "disable",
            Some(format!(
                "kaspad --rpclisten {rpc} --listen {p2p} --appdir {}",
                appdir.display()
            )),
            None,
        )
    };

    let occupied_listener = if args.iter().any(|arg| arg == "--occupy-stratum") {
        Some(
            std::net::TcpListener::bind(&stratum)
                .map_err(|error| format!("occupy Stratum listener {stratum} failed: {error}"))?,
        )
    } else {
        None
    };

    let started = kgw_kgw_apply_node_settings_v1(
        network.clone(),
        node_kind.to_string(),
        bridge_kind.to_string(),
        node_preview,
        bridge_preview,
        Some(runtime_role.clone()),
        if runtime_role == "bridge" {
            Some("provenance-smoke".to_string())
        } else {
            None
        },
        None,
        None,
        None,
        Some(false),
    );

    let (start_result, start_error) = match started {
        Ok(value) => (Some(value), None),
        Err(error) => (None, Some(error)),
    };

    let management_diagnostic = if start_result.is_some() {
        kgw_kgw_apply_node_settings_v1(
            network.clone(),
            node_kind.to_string(),
            bridge_kind.to_string(),
            None,
            None,
            Some(runtime_role.clone()),
            None,
            None,
            None,
            None,
            Some(false),
        )
        .err()
    } else {
        None
    };

    std::thread::sleep(std::time::Duration::from_secs(2));
    let status =
        kgw_runtime_owner_status_v1(Some(network.clone()), Some(runtime_role.clone())).ok();
    let raw_report =
        kgw_kgw_runtime_logs_v1(Some(network.clone()), Some(runtime_role.clone()), None)?;
    let stop_result = kgw_kgw_disable_network_v1(network.clone(), Some(runtime_role.clone())).ok();
    drop(occupied_listener);
    if runtime_role == "bridge" {
        let _ = kgw_kgw_disable_network_v1(network.clone(), Some("node".to_string()));
    }

    let result = SmokeResult {
        network,
        runtime_role,
        prerequisite_node_start,
        start_result,
        start_error,
        management_diagnostic,
        status,
        raw_report,
        stop_result,
    };
    let output = serde_json::to_string_pretty(&result).map_err(|error| error.to_string())?;
    println!("{output}");
    Ok(())
}
