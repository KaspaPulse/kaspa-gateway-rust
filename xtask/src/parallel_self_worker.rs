use regex::Regex;
use std::fs;
use std::path::Path;

const INTEGRATED_PATH: &str =
    "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs";
const CONTROLLER_PATH: &str = "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs";
const LIB_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
const NODE_FRONTEND_PATH: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";

struct Sources {
    integrated: String,
    controller: String,
    lib_rs: String,
    node_frontend: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let sources = Sources {
        integrated: read(root, INTEGRATED_PATH)?,
        controller: read(root, CONTROLLER_PATH)?,
        lib_rs: normalize_newlines(&read(root, LIB_PATH)?),
        node_frontend: read(root, NODE_FRONTEND_PATH)?,
    };
    validate(&sources)?;
    Ok(
        "KGW parallel self-worker runtime gate\nsameExeWorker: true\nexternalKaspadExe: false\nusesKaspaLibraries: true\nregistry: role:network\nroles: node, bridge\nnetworks: mainnet, testnet10, testnet13\nstatus: PASS"
            .to_owned(),
    )
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("parallel self-worker gate: failed to read {relative}: {error}"))
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern)
        .unwrap_or_else(|error| panic!("invalid parallel self-worker regex {pattern:?}: {error}"))
}

fn close_handler(lib_rs: &str) -> Result<&str, String> {
    let start_marker = ".on_window_event(|window, event| {";
    let end_marker = "\n        })\n        .run(context)";
    let start = lib_rs
        .find(start_marker)
        .ok_or_else(|| "close handler missing".to_owned())?;
    let tail = &lib_rs[start + start_marker.len()..];
    let end = tail
        .find(end_marker)
        .ok_or_else(|| "close handler missing".to_owned())?;
    Ok(&tail[..end])
}
fn require(ok: bool, name: &str, failures: &mut Vec<String>) {
    if !ok {
        failures.push(name.to_owned());
    }
}

fn validate(s: &Sources) -> Result<(), String> {
    let mut failures = Vec::new();
    let close = close_handler(&s.lib_rs).unwrap_or_default();

    require(
        s.integrated.contains("KGW_PARALLEL_SELF_WORKERS")
            && s.integrated.contains("OnceLock")
            && s.integrated.contains("HashMap"),
        "parallel registry",
        &mut failures,
    );
    require(
        s.integrated.contains("std::env::current_exe")
            && s.integrated.contains("Command::new")
            && s.integrated.contains("--kgw-self-worker"),
        "same exe spawn",
        &mut failures,
    );
    require(
        s.integrated.contains("--desktop-parent-pid")
            && s.integrated.contains("--desktop-parent-start-time")
            && s.integrated.contains("--desktop-parent-executable"),
        "exact parent identity",
        &mut failures,
    );
    require(
        s.lib_rs.contains("kgw_wait_for_stop_or_parent_loss_v1") && s.lib_rs.contains("ParentLost"),
        "parent loss shutdown",
        &mut failures,
    );
    require(
        s.integrated.contains("runtime-ownership")
            && s.integrated.contains("default_user_data_dir")
            && s.integrated.contains("create_new(true)")
            && s.integrated.contains("kgw_runtime_owner_worker_path_v1")
            && s.integrated
                .contains("kgw_runtime_owner_reconcile_stale_lease_v1"),
        "durable owner lease",
        &mut failures,
    );
    require(
        s.integrated.contains("--network"),
        "network arg",
        &mut failures,
    );
    require(
        s.integrated.contains("role")
            && s.integrated.contains("network")
            && s.integrated.contains(':'),
        "role network key",
        &mut failures,
    );
    for (name, marker) in [
        ("worker status string", "parallel-owned-self-worker"),
        ("same exe marker", "same_exe"),
        ("no external kaspad exe marker", "external_kaspad_exe"),
        ("uses kaspa libraries marker", "uses_kaspa_libraries"),
    ] {
        require(s.integrated.contains(marker), name, &mut failures);
    }
    require(
        regex("(?i)mainnet").is_match(&s.controller),
        "mainnet",
        &mut failures,
    );
    require(
        regex("(?i)testnet10").is_match(&s.controller),
        "testnet10",
        &mut failures,
    );
    require(
        regex("(?i)testnet13").is_match(&s.controller),
        "testnet13",
        &mut failures,
    );
    require(
        regex("(?i)stable|mainline").is_match(&s.controller),
        "stable mainline owner",
        &mut failures,
    );
    require(
        regex("(?i)tn13").is_match(&s.controller),
        "tn13 owner",
        &mut failures,
    );
    require(
        s.controller.contains("16110") && s.controller.contains("16210"),
        "distinct rpc ports",
        &mut failures,
    );
    require(
        s.lib_rs.contains("integrated_runtime_commands"),
        "tauri module registered",
        &mut failures,
    );

    require(
        close.contains("WindowEvent::CloseRequested") && close.contains("api.prevent_close()"),
        "close request intercepted",
        &mut failures,
    );
    require(
        close.contains("KGW_CLOSE_SHUTDOWN_STARTED") && close.contains("swap(true"),
        "close shutdown is single-flight",
        &mut failures,
    );
    require(
        close.contains("kgw_shutdown_all_runtime_workers_v1"),
        "close uses owned shutdown-all",
        &mut failures,
    );
    require(
        regex(r"(?s)Ok\(_\) => \{.*?app\.exit\(0\)").is_match(close),
        "close exits only after shutdown success",
        &mut failures,
    );
    require(
        regex(r"(?s)Err\(error\) => \{.*?KGW_CLOSE_SHUTDOWN_STARTED.*?store\(false")
            .is_match(close),
        "close failure stays open and retryable",
        &mut failures,
    );

    if !failures.is_empty() {
        return Err(format!(
            "KGW parallel self-worker runtime gate FAILED\n{}",
            failures
                .into_iter()
                .map(|name| format!("- {name}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    let forbidden_external =
        regex(r"(?i)Command::new\([^\n]*(kaspad|kaspabridge|stratum-bridge)\.exe")
            .is_match(&s.integrated);
    let forbidden_shell =
        regex(r"(?i)Command::new\([^\n]*(cmd|powershell|pwsh|sh)\b").is_match(&s.integrated);
    let frontend_shell = regex(r"(?i)shell|cmd\.exe|powershell|pwsh").is_match(&s.node_frontend);

    let mut forbidden = Vec::new();
    if forbidden_external {
        forbidden.push("external kaspad exe launch");
    }
    if forbidden_shell {
        forbidden.push("shell launch");
    }
    if frontend_shell {
        forbidden.push("frontend shell ownership");
    }
    if !forbidden.is_empty() {
        return Err(format!(
            "KGW parallel self-worker runtime gate FAILED\nForbidden runtime ownership detected:\n{}",
            forbidden
                .into_iter()
                .map(|name| format!("- {name}"))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn sources() -> Sources {
        let root = root();
        Sources {
            integrated: read(&root, INTEGRATED_PATH).unwrap(),
            controller: read(&root, CONTROLLER_PATH).unwrap(),
            lib_rs: normalize_newlines(&read(&root, LIB_PATH).unwrap()),
            node_frontend: read(&root, NODE_FRONTEND_PATH).unwrap(),
        }
    }

    #[test]
    fn real_repository_contract_passes() {
        assert!(validate(&sources()).is_ok());
    }

    #[test]
    fn missing_registry_fails_closed() {
        let mut s = sources();
        s.integrated = s
            .integrated
            .replace("KGW_PARALLEL_SELF_WORKERS", "REMOVED_REGISTRY");
        assert!(validate(&s).is_err());
    }
    #[test]
    fn close_handler_single_flight_drift_fails_closed() {
        let mut s = sources();
        s.lib_rs = s.lib_rs.replace("swap(true", "swap(false");
        assert!(validate(&s).is_err());
    }

    #[test]
    fn external_executable_launch_fails_closed() {
        let mut s = sources();
        s.integrated
            .push_str("\nfn forbidden_fixture() { let _ = Command::new(\"kaspad.exe\"); }\n");
        assert!(validate(&s).is_err());
    }

    #[test]
    fn frontend_shell_ownership_fails_closed() {
        let mut s = sources();
        s.node_frontend
            .push_str("\n// powershell ownership fixture\n");
        assert!(validate(&s).is_err());
    }
}
