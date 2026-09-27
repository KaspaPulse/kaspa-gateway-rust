use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CLIPBOARD_CAPTURE: &str = "xtask/src/raw_log_clipboard_capture.rs";
const LIVE_MATRIX: &str = "xtask/src/live_raw_log_matrix.rs";
const NODE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const BRIDGE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const RUNTIME_RS: &str = "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs";
const ZERO_TOUCH_E2E: &str = "tools/kgw_zero_touch_e2e.ps1";
const ZERO_TOUCH_EVIDENCE: &str = "tools/kgw_zero_touch_evidence.ps1";
const FULL_LOCAL_GATE: &str = "xtask/src/full_local_gate.rs";
const E2E_WINDOWS_HELPER: &str = "e2e/helpers/windows.mjs";

#[derive(Debug)]
struct Sources {
    node: String,
    bridge: String,
    runtime: String,
    clipboard_capture: String,
    live_matrix: String,
    zero_touch: String,
    zero_touch_evidence: String,
    full_local_gate: String,
    e2e_windows_helper: String,
}

#[derive(Debug)]
struct ProcessResult {
    code: i32,
    text: String,
}

#[derive(Debug, Default)]
struct GateContext {
    failures: Vec<String>,
    isolated_target: Option<PathBuf>,
}

pub fn run(root: &Path) -> Result<String, String> {
    let sources = load_sources(root)?;
    let mut ctx = GateContext::default();
    evaluate_static(&sources, &mut ctx.failures);

    run_clipboard_self_test(root, &mut ctx.failures);
    run_powershell_parser_checks(root, &mut ctx.failures);

    run_checked(
        root,
        "Node frontend syntax",
        "node",
        &["--check", NODE_JS],
        &mut ctx.failures,
    );
    run_checked(
        root,
        "Bridge frontend syntax",
        "node",
        &["--check", BRIDGE_JS],
        &mut ctx.failures,
    );
    println!("Running: True raw log frontend Rust owner");
    if let Err(error) = crate::true_raw_log_frontend::run(root) {
        ctx.failures
            .push(format!("True raw log frontend Rust owner failed: {error}"));
    }

    run_cargo_checked(
        root,
        "Rust typed raw log tests",
        &[
            "test",
            "-p",
            "kaspa-gateway-desktop",
            "typed_raw_log",
            "--test",
            "integrated_runtime_ipc_smoke_tests",
            "--",
            "--nocapture",
        ],
        &mut ctx,
    );
    run_cargo_checked(
        root,
        "Rust child raw log fixture test",
        &[
            "test",
            "-p",
            "kaspa-gateway-desktop",
            "child_stdout_and_stderr_fixtures_survive_unchanged",
            "--test",
            "integrated_runtime_ipc_smoke_tests",
            "--",
            "--nocapture",
        ],
        &mut ctx,
    );
    run_cargo_checked(
        root,
        "Rust official raw log sentinel test",
        &[
            "test",
            "-p",
            "kaspa-gateway-desktop",
            "official_sentinel_stdout_and_stderr_use_the_production_pipe_reader_unchanged",
            "--test",
            "integrated_runtime_ipc_smoke_tests",
            "--",
            "--nocapture",
        ],
        &mut ctx,
    );
    run_cargo_checked(
        root,
        "Rust network and role isolation test",
        &[
            "test",
            "-p",
            "kaspa-gateway-desktop",
            "raw_log_buffers_are_isolated_by_network_and_role_with_process_wide_bridge_output",
            "--test",
            "integrated_runtime_ipc_smoke_tests",
            "--",
            "--nocapture",
        ],
        &mut ctx,
    );
    run_cargo_checked(
        root,
        "Desktop debug build",
        &["build", "--locked", "--bin", "kaspa-gateway-desktop"],
        &mut ctx,
    );

    if ctx.failures.is_empty() {
        let mut message = String::from("KGW true raw log gate PASSED");
        if let Some(target) = ctx.isolated_target {
            message.push_str("\nUsed isolated CARGO_TARGET_DIR=");
            message.push_str(&target.to_string_lossy());
        }
        Ok(message)
    } else {
        let mut message = String::from("KGW true raw log gate FAILED");
        for failure in ctx.failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        Err(message)
    }
}

fn load_sources(root: &Path) -> Result<Sources, String> {
    Ok(Sources {
        node: read(root, NODE_JS)?,
        bridge: read(root, BRIDGE_JS)?,
        runtime: read(root, RUNTIME_RS)?,
        clipboard_capture: read(root, CLIPBOARD_CAPTURE)?,
        live_matrix: read(root, LIVE_MATRIX)?,
        zero_touch: read(root, ZERO_TOUCH_E2E)?,
        zero_touch_evidence: read(root, ZERO_TOUCH_EVIDENCE)?,
        full_local_gate: read(root, FULL_LOCAL_GATE)?,
        e2e_windows_helper: read(root, E2E_WINDOWS_HELPER)?,
    })
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("true-raw-log: failed to read {relative}: {error}"))
}

fn require(failures: &mut Vec<String>, source: &str, needle: &str, message: &str) {
    if !source.contains(needle) {
        failures.push(format!("{message} Missing: {needle}"));
    }
}

fn forbid(failures: &mut Vec<String>, source: &str, needle: &str, message: &str) {
    if source.contains(needle) {
        failures.push(format!("{message} Forbidden: {needle}"));
    }
}

fn evaluate_static(s: &Sources, failures: &mut Vec<String>) {
    forbid(
        failures,
        &s.runtime,
        "kgw_raw_process_log_v1",
        "Rust runtime source must not serialize the old raw-process envelope.",
    );

    let old_append = Regex::new(r"(?i)appendLog\s*\([^)]*kgw_raw_process_log_v1")
        .expect("valid old envelope regex");
    if old_append.is_match(&format!("{}\n{}", s.node, s.bridge)) {
        failures.push(
            "Frontend code must not append the old transport envelope into a UI buffer.".to_owned(),
        );
    }
    for (needle, message) in [
        (
            "KgwRuntimeRawLogEntryV1",
            "Rust must carry typed raw log entries.",
        ),
        (
            "sequence",
            "Rust raw log entries must include a monotonic sequence.",
        ),
        (
            "raw_text",
            "Rust raw log entries must keep raw child text separate.",
        ),
    ] {
        require(failures, &s.runtime, needle, message);
    }

    for (source, needle, message) in [
        (
            &s.node,
            "kgwNodeApplyRuntimeLogReportV1",
            "Node UI must consume typed raw log reports.",
        ),
        (
            &s.bridge,
            "kgwBridgeApplyRuntimeLogReportV1",
            "Bridge UI must consume typed raw log reports.",
        ),
        (
            &s.node,
            "kgwNodeRawLogTextHasTransportWrapperV1",
            "Node UI must reject transport wrapper text before display or copy.",
        ),
        (
            &s.bridge,
            "kgwBridgeRawLogTextHasTransportWrapperV1",
            "Bridge UI must reject transport wrapper text before display or copy.",
        ),
        (
            &s.node,
            "kgwNodeRawLogTextHasTransportWrapperV1(legacyTransportText)",
            "Node UI must apply transport rejection only at the untyped report boundary.",
        ),
        (
            &s.bridge,
            "kgwBridgeRawLogTextHasTransportWrapperV1(legacyTransportText)",
            "Bridge UI must apply transport rejection only at the untyped report boundary.",
        ),
        (
            &s.node,
            r#"runtimeRole: metadata.runtimeRole || "node""#,
            "Node Copy Log must carry runtime role metadata to native clipboard traces.",
        ),
        (
            &s.bridge,
            r#"runtimeRole: metadata.runtimeRole || "bridge""#,
            "Bridge Copy Log must carry runtime role metadata to native clipboard traces.",
        ),
        (
            &s.bridge,
            r#"bridgeInstanceId: metadata.bridgeInstanceId || """#,
            "Bridge Copy Log must carry bridge instance metadata to native clipboard traces.",
        ),
    ] {
        require(failures, source, needle, message);
    }

    for (needle, message) in [
        (
            "capture_from_clipboard",
            "Rust tooling must capture clipboard payloads at event time.",
        ),
        (
            "expected_sha256",
            "Rust clipboard capture metadata must record expected SHA256.",
        ),
        (
            "actual_sha256",
            "Rust clipboard capture metadata must record actual SHA256.",
        ),
        (
            "sha256_match",
            "Rust clipboard capture must compare expected and actual SHA256.",
        ),
    ] {
        require(failures, &s.clipboard_capture, needle, message);
    }

    for (needle, message) in [
        (
            r#"name: "Testnet10 Node""#,
            "Live matrix must guide Testnet10 Node first.",
        ),
        (
            r#"name: "Mainnet Bridge""#,
            "Live matrix must guide Mainnet Bridge second.",
        ),
        (
            r#"name: "Testnet10 Bridge""#,
            "Live matrix must guide Testnet10 Bridge third.",
        ),
        (
            "write_text_after_preflight",
            "Live matrix must set a clipboard sentinel through the fail-closed Rust preflight.",
        ),
        (
            "wait_for_stage_capture",
            "Live matrix must capture Copy Log at event time before advancing.",
        ),
    ] {
        require(failures, &s.live_matrix, needle, message);
    }
    for (needle, message) in [
        (
            "(Get-Command pwsh -ErrorAction Stop).Source",
            "Zero-touch launcher must resolve the PowerShell 7 executable.",
        ),
        (
            "KGW_REQUIRED_PWSH_PATH",
            "Zero-touch launcher must pass the resolved PowerShell 7 executable to child helpers.",
        ),
    ] {
        require(failures, &s.zero_touch, needle, message);
    }

    for (needle, message) in [
        (
            "Write-KgwZeroTouchJsonFile",
            "Zero-touch evidence helper must provide the shared strict JSON writer.",
        ),
        (
            "Write-KgwZeroTouchEmergencyJsonFile",
            "Zero-touch evidence helper must provide the emergency fallback writer.",
        ),
    ] {
        require(failures, &s.zero_touch_evidence, needle, message);
    }

    require(
        failures,
        &s.full_local_gate,
        "powershell_parser_checks",
        "Full local gate must validate remaining PowerShell helpers through Rust-owned orchestration.",
    );
    require(
        failures,
        &s.full_local_gate,
        "zero-touch-result-writer-tests",
        "Full local gate must invoke the Rust zero-touch result-writer tests.",
    );
    forbid(
        failures,
        &s.full_local_gate,
        r#"-FilePath "powershell""#,
        "Full local gate must not launch Windows PowerShell.",
    );
    require(
        failures,
        &s.full_local_gate,
        "true-raw-log-gate",
        "Full local gate must delegate true raw log verification to Rust.",
    );
    require(
        failures,
        &s.e2e_windows_helper,
        "KGW_REQUIRED_PWSH_PATH",
        "WDIO helper must inherit the resolved PowerShell 7 executable.",
    );
}

fn process_output(
    root: &Path,
    program: &str,
    args: &[&str],
    target_dir: Option<&Path>,
) -> Result<ProcessResult, String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    if let Some(target) = target_dir {
        command.env("CARGO_TARGET_DIR", target);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to launch {program}: {error}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.stderr.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    Ok(ProcessResult {
        code: output.status.code().unwrap_or(1),
        text,
    })
}

fn run_checked(root: &Path, label: &str, program: &str, args: &[&str], failures: &mut Vec<String>) {
    println!("Running: {label}");
    match process_output(root, program, args, None) {
        Ok(result) => {
            if !result.text.trim().is_empty() {
                print!("{}", result.text);
                if !result.text.ends_with('\n') {
                    println!();
                }
            }
            if result.code != 0 {
                failures.push(format!("{label} failed with exit code {}", result.code));
            }
        }
        Err(error) => failures.push(format!("{label} failed to launch: {error}")),
    }
}
fn librocksdb_artifact_inconsistency(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("librocksdb-sys")
        && [
            "inconsistent",
            "missing",
            "not found",
            "could not find",
            "failed to read",
            "no such file",
            "access is denied",
            "lnk1104",
            "corrupt",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn isolated_target_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "kgw-true-raw-log-target-{}-{stamp}",
        std::process::id()
    ))
}

fn run_cargo_checked(root: &Path, label: &str, args: &[&str], ctx: &mut GateContext) {
    println!("Running: {label}");
    let first = match process_output(root, "cargo", args, ctx.isolated_target.as_deref()) {
        Ok(result) => result,
        Err(error) => {
            ctx.failures
                .push(format!("{label} failed to launch cargo: {error}"));
            return;
        }
    };
    if !first.text.trim().is_empty() {
        print!("{}", first.text);
        if !first.text.ends_with('\n') {
            println!();
        }
    }
    if first.code == 0 {
        return;
    }

    if ctx.isolated_target.is_none() && librocksdb_artifact_inconsistency(&first.text) {
        let target = isolated_target_dir();
        if let Err(error) = fs::create_dir_all(&target) {
            ctx.failures.push(format!(
                "{label} detected inconsistent librocksdb-sys artifacts but failed to create isolated target {}: {error}",
                target.display()
            ));
            return;
        }
        println!(
            "librocksdb-sys artifacts look inconsistent; retrying with isolated CARGO_TARGET_DIR={}",
            target.display()
        );
        let retry = match process_output(root, "cargo", args, Some(&target)) {
            Ok(result) => result,
            Err(error) => {
                ctx.failures
                    .push(format!("{label} failed to launch cargo retry: {error}"));
                return;
            }
        };
        if !retry.text.trim().is_empty() {
            print!("{}", retry.text);
            if !retry.text.ends_with('\n') {
                println!();
            }
        }
        ctx.isolated_target = Some(target);
        if retry.code != 0 {
            ctx.failures.push(format!(
                "{label} failed with exit code {} after isolated CARGO_TARGET_DIR retry",
                retry.code
            ));
        }
        return;
    }

    ctx.failures
        .push(format!("{label} failed with exit code {}", first.code));
}

fn pwsh_command(root: &Path, script: &str) -> Result<ProcessResult, String> {
    process_output(
        root,
        "pwsh",
        &["-NoLogo", "-NoProfile", "-Command", script],
        None,
    )
}

fn run_clipboard_self_test(_root: &Path, failures: &mut Vec<String>) {
    match crate::raw_log_clipboard_capture::self_test() {
        Ok(message) => println!("{message}"),
        Err(error) => failures.push(format!(
            "Event-time clipboard capture deterministic self-test failed: {error}"
        )),
    }
}

fn run_powershell_parser_checks(root: &Path, failures: &mut Vec<String>) {
    for (relative, label) in [
        (ZERO_TOUCH_E2E, "Zero-touch E2E launcher"),
        (ZERO_TOUCH_EVIDENCE, "Zero-touch evidence helper"),
        (
            "e2e/helpers/kgw_windows_clipboard.ps1",
            "E2E clipboard helper",
        ),
    ] {
        let escaped = relative.replace('\'', "''");
        let script = format!(
            r#"$errors=$null; [void][System.Management.Automation.Language.Parser]::ParseFile((Resolve-Path -LiteralPath '{escaped}').Path,[ref]$null,[ref]$errors); if($errors -and $errors.Count -gt 0){{foreach($e in $errors){{Write-Error ('{label} syntax error: ' + $e.Message)}}; exit 1}}"#
        );
        match pwsh_command(root, &script) {
            Ok(result) if result.code == 0 => {}
            Ok(result) => failures.push(format!(
                "{label} syntax check failed with exit code {}: {}",
                result.code,
                result.text.trim()
            )),
            Err(error) => failures.push(format!("{label} syntax check failed to launch: {error}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> Sources {
        Sources {
            node: [
                "kgwNodeApplyRuntimeLogReportV1",
                "kgwNodeRawLogTextHasTransportWrapperV1",
                "kgwNodeRawLogTextHasTransportWrapperV1(legacyTransportText)",
                r#"runtimeRole: metadata.runtimeRole || "node""#,
            ]
            .join("\n"),
            bridge: [
                "kgwBridgeApplyRuntimeLogReportV1",
                "kgwBridgeRawLogTextHasTransportWrapperV1",
                "kgwBridgeRawLogTextHasTransportWrapperV1(legacyTransportText)",
                r#"runtimeRole: metadata.runtimeRole || "bridge""#,
                r#"bridgeInstanceId: metadata.bridgeInstanceId || """#,
            ]
            .join("\n"),
            runtime: ["KgwRuntimeRawLogEntryV1", "sequence", "raw_text"].join("\n"),
            clipboard_capture: [
                "capture_from_clipboard",
                "expected_sha256",
                "actual_sha256",
                "sha256_match",
            ]
            .join("\n"),
            live_matrix: [
                r#"name: "Testnet10 Node""#,
                r#"name: "Mainnet Bridge""#,
                r#"name: "Testnet10 Bridge""#,
                "write_text_after_preflight",
                "wait_for_stage_capture",
            ]
            .join("\n"),
            zero_touch: [
                "(Get-Command pwsh -ErrorAction Stop).Source",
                "KGW_REQUIRED_PWSH_PATH",
            ]
            .join("\n"),
            zero_touch_evidence: [
                "Write-KgwZeroTouchJsonFile",
                "Write-KgwZeroTouchEmergencyJsonFile",
            ]
            .join("\n"),
            full_local_gate: [
                "powershell_parser_checks",
                "true-raw-log-gate",
                "zero-touch-result-writer-tests",
            ]
            .join("\n"),
            e2e_windows_helper: "KGW_REQUIRED_PWSH_PATH".to_owned(),
        }
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn complete_static_contract_passes() {
        let mut failures = Vec::new();
        evaluate_static(&sources(), &mut failures);
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn current_repository_static_contract_passes() {
        let sources = load_sources(&repo_root()).unwrap();
        let mut failures = Vec::new();
        evaluate_static(&sources, &mut failures);
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn old_transport_envelope_append_fails_closed() {
        let mut fixture = sources();
        fixture
            .node
            .push_str("\nappendLog(x, kgw_raw_process_log_v1(payload));");
        let mut failures = Vec::new();
        evaluate_static(&fixture, &mut failures);
        assert!(
            failures
                .iter()
                .any(|item| item.contains("old transport envelope"))
        );
    }

    #[test]
    fn missing_clipboard_hash_contract_fails_closed() {
        let mut fixture = sources();
        fixture.clipboard_capture = fixture
            .clipboard_capture
            .replace("sha256_match", "sha_match_removed");
        let mut failures = Vec::new();
        evaluate_static(&fixture, &mut failures);
        assert!(
            failures
                .iter()
                .any(|item| item.contains("compare expected and actual SHA256"))
        );
    }

    #[test]
    fn detects_librocksdb_artifact_inconsistency() {
        assert!(librocksdb_artifact_inconsistency(
            "librocksdb-sys: failed to read artifact, file not found"
        ));
        assert!(librocksdb_artifact_inconsistency(
            "LNK1104 access is denied for librocksdb-sys output"
        ));
        assert!(!librocksdb_artifact_inconsistency(
            "librocksdb-sys compiled successfully"
        ));
        assert!(!librocksdb_artifact_inconsistency(
            "unrelated file not found"
        ));
    }
}
