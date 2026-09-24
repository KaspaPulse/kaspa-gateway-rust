use regex::Regex;
use std::fs;
use std::path::Path;

const TAURI_APP: &str = "e2e/helpers/tauri-app.mjs";
const ZERO_TOUCH_MATRIX: &str = "e2e/specs/zero-touch-live-matrix.e2e.js";
const WDIO_CONF: &str = "e2e/wdio.conf.mjs";
const WINDOWS_HELPERS: &str = "e2e/helpers/windows.mjs";
const OWNED_PROCESS_RUST: &str = "xtask/src/e2e_owned_process.rs";
const RELAUNCH: &str = "e2e/helpers/app-close-relaunch.mjs";

#[derive(Debug)]
struct Sources {
    tauri_app: String,
    zero_touch_matrix: String,
    wdio: String,
    windows_helpers: String,
    owned_process_rust: String,
    relaunch: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let sources = load_sources(root)?;
    let failures = evaluate(&sources);
    if failures.is_empty() {
        Ok("KGW E2E static smokes PASSED".to_owned())
    } else {
        let mut message = String::from("KGW E2E static smokes FAILED");
        for failure in failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        Err(message)
    }
}

fn load_sources(root: &Path) -> Result<Sources, String> {
    Ok(Sources {
        tauri_app: read(root, TAURI_APP)?,
        zero_touch_matrix: read(root, ZERO_TOUCH_MATRIX)?,
        wdio: read(root, WDIO_CONF)?,
        windows_helpers: read(root, WINDOWS_HELPERS)?,
        owned_process_rust: read(root, OWNED_PROCESS_RUST)?,
        relaunch: read(root, RELAUNCH)?,
    })
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("e2e-static-smokes: failed to read {relative}: {error}"))
}

fn require(failures: &mut Vec<String>, source: &str, needle: &str, message: &str) {
    if !source.contains(needle) {
        failures.push(format!("{message} Missing: {needle}"));
    }
}

fn require_order(
    failures: &mut Vec<String>,
    source: &str,
    first: &str,
    second: &str,
    message: &str,
) {
    let first_index = source.find(first);
    let second_index = source.find(second);
    if first_index.is_none()
        || second_index.is_none()
        || first_index.is_some_and(|left| second_index.is_some_and(|right| left >= right))
    {
        failures.push(message.to_owned());
    }
}

fn parse_key_value_line(text: &str) -> Vec<(&str, &str)> {
    text.split(';')
        .filter_map(|part| {
            let index = part.find('=')?;
            if index == 0 {
                return None;
            }
            Some((part[..index].trim(), part[index + 1..].trim()))
        })
        .collect()
}

fn field<'a>(fields: &'a [(&str, &str)], name: &str) -> &'a str {
    fields
        .iter()
        .find_map(|(key, value)| (*key == name).then_some(*value))
        .unwrap_or("")
}

fn is_stopped_owner_status(status: &str) -> bool {
    let fields = parse_key_value_line(status);
    let running = field(&fields, "running");
    if running.eq_ignore_ascii_case("false") {
        return true;
    }
    let lower = status.to_ascii_lowercase();
    (Regex::new(r"no .*worker status yet")
        .expect("valid stopped-owner regex")
        .is_match(&lower)
        || lower.contains("stopped"))
        && !running.eq_ignore_ascii_case("true")
}

fn evaluate(s: &Sources) -> Vec<String> {
    let mut failures = Vec::new();

    require(
        &mut failures,
        &s.tauri_app,
        r#"`#bridge-${net}-instancePort-${bridgeInstanceId}`"#,
        "Bridge runtime selection must read the actual instancePort DOM id.",
    );
    let bridge_write = Regex::new(
        r#"setControlValueById\(\s*`bridge-\$\{network\}-instancePort-\$\{selection\.bridgeInstanceId\}`"#,
    )
    .expect("valid bridge locator regex");
    if !bridge_write.is_match(&s.zero_touch_matrix) {
        failures.push("Bridge E2E must write the actual instancePort DOM id.".to_owned());
    }

    require(
        &mut failures,
        &s.wdio,
        r#"process.env.KGW_E2E_SPEC || "./specs/zero-touch-live-matrix.e2e.js""#,
        "WDIO must preserve the selected local spec fallback.",
    );
    require(
        &mut failures,
        &s.wdio,
        "specs: [selectedSpec]",
        "WDIO must use the selected local spec.",
    );

    if !is_stopped_owner_status("role=node;network=mainnet;pid=4242;running=false;readiness=FAILED")
    {
        failures.push(
            "Terminal owner status must remain stopped even when PID is retained as crash evidence."
                .to_owned(),
        );
    }
    if is_stopped_owner_status("role=node;network=mainnet;pid=4242;running=true;readiness=READY") {
        failures.push("Running owner status must never be classified as stopped.".to_owned());
    }

    let kill_start = s
        .windows_helpers
        .find("export async function killExactOwnedProcess");
    let wait_start = s
        .windows_helpers
        .find("export async function waitForExactProcessExit");
    let evidence_start = s
        .windows_helpers
        .find("export async function captureWindowsEvidence");
    match (kill_start, wait_start, evidence_start) {
        (Some(kill), Some(wait), Some(evidence)) if kill < wait && wait < evidence => {
            let kill_call = &s.windows_helpers[kill..wait];
            let wait_call = &s.windows_helpers[wait..evidence];
            for (source, needle, message) in [
                (
                    kill_call,
                    r#""e2e-owned-process""#,
                    "exact-owner kill helper must call Rust xtask command.",
                ),
                (
                    kill_call,
                    r#""kill""#,
                    "exact-owner kill helper must select Rust kill action.",
                ),
                (
                    wait_call,
                    r#""e2e-owned-process""#,
                    "exact-owner wait helper must call Rust xtask command.",
                ),
                (
                    wait_call,
                    r#""wait""#,
                    "exact-owner wait helper must select Rust wait action.",
                ),
            ] {
                require(&mut failures, source, needle, message);
            }
            if kill_call.contains(".ps1") {
                failures.push("exact-owner kill helper must be Rust-owned.".to_owned());
            }
            if wait_call.contains(".ps1") {
                failures.push("exact-owner wait helper must be Rust-owned.".to_owned());
            }
        }
        _ => failures
            .push("kill/wait/evidence E2E helper ordering contract is incomplete.".to_owned()),
    }

    let kill_rust_start = s.owned_process_rust.find("fn kill_exact_owned_process");
    let wait_rust_start = s.owned_process_rust.find("fn wait_exact_process_exit");
    match (kill_rust_start, wait_rust_start) {
        (Some(kill), Some(wait)) if kill < wait => {
            let kill_rust = &s.owned_process_rust[kill..wait];
            require_order(
                &mut failures,
                kill_rust,
                "owned process executable mismatch",
                "TerminateProcess",
                "Executable identity must be checked before force kill.",
            );
            require_order(
                &mut failures,
                kill_rust,
                "owned process start-time mismatch",
                "TerminateProcess",
                "Start-time identity must be checked before force kill.",
            );

            if let Some(test_module) = s.owned_process_rust[wait..]
                .find("#[cfg(test)]")
                .map(|relative| wait + relative)
            {
                let wait_rust = &s.owned_process_rust[wait..test_module];
                require(
                    &mut failures,
                    wait_rust,
                    "exact_identity_exited",
                    "Close/relaunch evidence must track exact identity exit.",
                );
                if wait_rust.contains("TerminateProcess") {
                    failures
                        .push("Close/relaunch wait helper must never kill a process.".to_owned());
                }
            } else {
                failures.push("Could not isolate wait helper before Rust test module.".to_owned());
            }
        }
        _ => failures
            .push("Rust exact-owner kill/wait functions are missing or reordered.".to_owned()),
    }

    require_order(
        &mut failures,
        &s.relaunch,
        "firstBrowser.closeWindow()",
        "waitForExactProcessExit({",
        "Relaunch flow must request close before waiting for exact identity exit.",
    );
    require_order(
        &mut failures,
        &s.relaunch,
        "waitForExactProcessExit({",
        r#"secondBrowser = await newSession("after-relaunch""#,
        "Relaunch flow must wait for exact identity exit before creating a new session.",
    );

    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Sources {
        Sources {
            tauri_app: r#"`#bridge-${net}-instancePort-${bridgeInstanceId}`"#.to_owned(),
            zero_touch_matrix:
                "setControlValueById(`bridge-${network}-instancePort-${selection.bridgeInstanceId}`, 1234)"
                    .to_owned(),
            wdio: [
                r#"const selectedSpec = process.env.KGW_E2E_SPEC || "./specs/zero-touch-live-matrix.e2e.js";"#,
                "specs: [selectedSpec]",
            ]
            .join("\n"),
            windows_helpers: [
                "export async function killExactOwnedProcess() {",
                r#"  "e2e-owned-process"; "kill";"#,
                "}",
                "export async function waitForExactProcessExit() {",
                r#"  "e2e-owned-process"; "wait";"#,
                "}",
                "export async function captureWindowsEvidence() {}",
            ]
            .join("\n"),
            owned_process_rust: [
                "fn kill_exact_owned_process() {",
                r#"  "owned process executable mismatch";"#,
                r#"  "owned process start-time mismatch";"#,
                "  TerminateProcess();",
                "}",
                "fn wait_exact_process_exit() {",
                "  exact_identity_exited();",
                "}",
                "#[cfg(test)]",
            ]
            .join("\n"),
            relaunch: [
                "firstBrowser.closeWindow();",
                "waitForExactProcessExit({});",
                r#"secondBrowser = await newSession("after-relaunch");"#,
            ]
            .join("\n"),
        }
    }

    #[test]
    fn complete_contract_passes() {
        let failures = evaluate(&fixture());
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn stopped_owner_semantics_match_js_contract() {
        assert!(is_stopped_owner_status(
            "role=node;network=mainnet;pid=4242;running=false;readiness=FAILED"
        ));
        assert!(!is_stopped_owner_status(
            "role=node;network=mainnet;pid=4242;running=true;readiness=READY"
        ));
        assert!(is_stopped_owner_status("no node worker status yet"));
        assert!(is_stopped_owner_status("role=node;running=;state=stopped"));
    }

    #[test]
    fn fabricated_bridge_locator_fails_closed() {
        let mut sources = fixture();
        sources.tauri_app = r#"`#bridge-${net}-instancePort-1`"#.to_owned();
        let failures = evaluate(&sources);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("actual instancePort DOM id"))
        );
    }

    #[test]
    fn kill_before_identity_guard_fails_closed() {
        let mut sources = fixture();
        sources.owned_process_rust = sources.owned_process_rust.replace(
            r#"  "owned process executable mismatch";
  "owned process start-time mismatch";
  TerminateProcess();"#,
            r#"  TerminateProcess();
  "owned process executable mismatch";
  "owned process start-time mismatch";"#,
        );
        let failures = evaluate(&sources);
        assert!(
            failures
                .iter()
                .any(|failure| failure.contains("Executable identity"))
        );
    }

    #[test]
    fn current_repository_static_smokes_pass() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let sources = load_sources(root).unwrap();
        let failures = evaluate(&sources);
        assert!(failures.is_empty(), "{failures:#?}");
    }
}
