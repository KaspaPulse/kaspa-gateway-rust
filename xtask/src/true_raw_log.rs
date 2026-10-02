use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const CLIPBOARD_CAPTURE: &str = "xtask/src/raw_log_clipboard_capture.rs";
const LIVE_MATRIX: &str = "xtask/src/live_raw_log_matrix.rs";
const NODE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const NODE_RAW_LOG_RS: &str = "crates/kaspa-gateway-frontend-wasm/src/node_start_trace.rs";
const BRIDGE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const BRIDGE_TAB_RS: &str = "crates/kaspa-gateway-frontend-wasm/src/bridge_tab.rs";
const BRIDGE_RAW_LOG_RS: &str = "crates/kaspa-gateway-frontend-wasm/src/bridge_raw_log.rs";
const BRIDGE_START_TRACE_RS: &str = "crates/kaspa-gateway-frontend-wasm/src/bridge_start_trace.rs";
const RUNTIME_RS: &str = "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs";
const ZERO_TOUCH_E2E: &str = "xtask/src/zero_touch_e2e.rs";
const ZERO_TOUCH_EVIDENCE: &str = "tools/kgw_zero_touch_evidence.ps1";
const FULL_LOCAL_GATE: &str = "xtask/src/full_local_gate.rs";
const E2E_WINDOWS_HELPER: &str = "e2e/helpers/windows.mjs";

#[derive(Debug)]
struct Sources {
    node: String,
    node_raw_log: String,
    bridge: String,
    bridge_tab: String,
    bridge_raw_log: String,
    bridge_start_trace: String,
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
        node_raw_log: read(root, NODE_RAW_LOG_RS)?,
        bridge: read(root, BRIDGE_JS)?,
        bridge_tab: read(root, BRIDGE_TAB_RS)?,
        bridge_raw_log: read(root, BRIDGE_RAW_LOG_RS)?,
        bridge_start_trace: read(root, BRIDGE_START_TRACE_RS)?,
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
            &s.node_raw_log,
            "#[wasm_bindgen(js_name = nodeApplyRuntimeLogReport)]",
            "Node typed raw-log report ownership must remain in Rust/WASM.",
        ),
        (
            &s.bridge_raw_log,
            "#[wasm_bindgen(js_name = bridgeApplyRuntimeLogReport)]",
            "Bridge typed raw-log report application must remain Rust/WASM-owned.",
        ),
        (
            &s.bridge_tab,
            "bridge_render_raw_log_buffer(",
            "Bridge tab Rust owner must render raw-log buffers directly.",
        ),
        (
            &s.bridge_raw_log,
            "#[wasm_bindgen(js_name = bridgeClearRawLogBuffer)]",
            "Bridge raw-log clear ownership must remain exported by Rust/WASM.",
        ),
        (
            &s.bridge_raw_log,
            "fn transport_wrapper_text(value: &str) -> bool",
            "Bridge transport-wrapper rejection must remain Rust-owned before display or copy.",
        ),
        (
            &s.node_raw_log,
            "if raw_log_transport_wrapper_text(&legacy)",
            "Node Rust owner must apply transport rejection only at the untyped report boundary.",
        ),
        (
            &s.bridge_raw_log,
            "if transport_wrapper_text(&legacy_transport_text(report))",
            "Bridge Rust owner must apply transport rejection only at the untyped report boundary.",
        ),
        (
            &s.node_raw_log,
            r#"metadata_text(metadata, "runtimeRole", "node")"#,
            "Node Rust Copy Log owner must carry runtime role metadata to native clipboard traces.",
        ),
        (
            &s.bridge_start_trace,
            r#"metadata_text(metadata, "runtimeRole", "bridge")"#,
            "Bridge Rust Copy Log owner must carry runtime role metadata to native clipboard traces.",
        ),
        (
            &s.bridge_start_trace,
            r#"metadata_text(metadata, "bridgeInstanceId", "")"#,
            "Bridge Rust Copy Log owner must carry bridge instance metadata to native clipboard traces.",
        ),
    ] {
        require(failures, source, needle, message);
    }

    for (needle, message) in [
        (
            "kgwNodeApplyRuntimeLogReportV1",
            "Node typed raw-log application must remain Rust-owned without a JavaScript compatibility wrapper.",
        ),
        (
            "kgwNodeRawLogTextHasTransportWrapperV1",
            "Node transport-wrapper rejection must be Rust-owned without a JavaScript compatibility wrapper.",
        ),
        (
            "kgwNodeNormalizeRawLogEntryV1",
            "Node raw-log normalization must be Rust-owned without a JavaScript compatibility wrapper.",
        ),
        (
            "kgwNodeLegacyTransportReportTextV1",
            "Node legacy transport extraction must be Rust-owned without a JavaScript compatibility wrapper.",
        ),
        (
            "kgwNodeVisibleRawLogTextV1",
            "Node visible raw-log text must be Rust-owned without a JavaScript compatibility wrapper.",
        ),
        (
            "kgwNodeRenderRawLogBufferV1",
            "Node raw-log rendering must be Rust-owned without a JavaScript compatibility wrapper.",
        ),
    ] {
        forbid(failures, &s.node, needle, message);
    }

    forbid(
        failures,
        &s.bridge,
        "kgwBridgeRawLogTextHasTransportWrapperV1",
        "Bridge transport-wrapper rejection must remain Rust-owned without a JavaScript compatibility wrapper.",
    );
    for (needle, message) in [
        (
            "function kgwBridgeRenderRawLogBufferV1(",
            "Bridge raw-log rendering must remain Rust-owned without a JavaScript passthrough wrapper.",
        ),
        (
            "function kgwBridgeApplyRuntimeLogReportV1(",
            "Bridge typed raw-log report application must remain Rust-owned without a JavaScript passthrough wrapper.",
        ),
        (
            "function kgwBridgeClearRawLogBufferV1(",
            "Bridge raw-log clearing must remain Rust-owned without a JavaScript passthrough wrapper.",
        ),
    ] {
        forbid(failures, &s.bridge, needle, message);
    }
    forbid(
        failures,
        &s.bridge_start_trace,
        "\"clearRawLogBuffer\"",
        "Bridge start-trace must not depend on the retired JavaScript clearRawLogBuffer callback.",
    );
    require(
        failures,
        &s.bridge_start_trace,
        "crate::bridge_raw_log::bridge_clear_raw_log_buffer(",
        "Bridge start-trace must call the Rust raw-log clear owner directly.",
    );
    for (needle, message) in [
        (
            "function kgwBridgeTranslateRuntimeV29(",
            "Bridge log-action translation must remain Rust-owned without a JavaScript helper.",
        ),
        (
            "translateRuntime:",
            "Bridge log-action translation must not return as a JavaScript callback seam.",
        ),
        (
            "smallOwnerTrace:",
            "Bridge log-action tracing must not return as a JavaScript callback seam.",
        ),
    ] {
        forbid(failures, &s.bridge, needle, message);
    }
    forbid(
        failures,
        &s.bridge_start_trace,
        "\"translateRuntime\"",
        "Bridge start-trace must not depend on the retired translateRuntime callback.",
    );
    forbid(
        failures,
        &s.bridge_start_trace,
        "\"smallOwnerTrace\"",
        "Bridge start-trace must not depend on the retired smallOwnerTrace callback.",
    );
    require(
        failures,
        &s.bridge_start_trace,
        "for name in [\"kgwT\", \"kgwI18n\", \"__kgwT\"]",
        "Bridge start-trace must own runtime translation lookup in Rust/WASM.",
    );
    require(
        failures,
        &s.bridge_start_trace,
        "crate::bridge_frontend_helpers::bridge_small_owner_trace_r44d(",
        "Bridge start-trace must call the Rust small-owner trace owner directly.",
    );

    for (needle, message) in [
        (
            "function kgwBridgeDispatchRuntimeLogClearV1(",
            "Bridge Clear Log runtime dispatch must not remain JavaScript-owned.",
        ),
        (
            "dispatchRuntimeLogClear:",
            "Bridge Clear Log must not retain the JavaScript callback seam.",
        ),
        (
            "kgw_kgw_runtime_clear_logs_v1",
            "Bridge Clear Log runtime command ownership must not remain in JavaScript.",
        ),
    ] {
        forbid(failures, &s.bridge, needle, message);
    }
    forbid(
        failures,
        &s.bridge_start_trace,
        "\"dispatchRuntimeLogClear\"",
        "Bridge start-trace must not depend on the retired dispatchRuntimeLogClear callback.",
    );
    for (needle, message) in [
        (
            "fn active_runtime_bridge_instance_id(",
            "Bridge start-trace must derive Clear Log bridgeInstanceId in Rust.",
        ),
        (
            "property(deps, \"activeInstance\")",
            "Bridge start-trace must read the active Bridge instance from Rust-owned deps.",
        ),
        (
            "spawn_local(async move",
            "Bridge Clear Log runtime dispatch must remain fire-and-forget in Rust.",
        ),
        (
            "\"kgw_kgw_runtime_clear_logs_v1\"",
            "Bridge Clear Log runtime command must be Rust-owned.",
        ),
        (
            "invoke_runtime_command_impl(",
            "Bridge Clear Log runtime dispatch must use the existing Rust runtime transport.",
        ),
    ] {
        require(failures, &s.bridge_start_trace, needle, message);
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
            "required_pwsh",
            "Rust zero-touch runner must resolve and validate the PowerShell 7 platform adapter.",
        ),
        (
            "KGW_REQUIRED_PWSH_PATH",
            "Rust zero-touch runner must pass the resolved PowerShell 7 executable to child helpers.",
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
        "zero_touch_e2e::run",
        "Full-local integration must invoke the Rust zero-touch runner.",
    );
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
        "e2e-clipboard",
        "WDIO Windows helper must delegate clipboard operations to Rust e2e-clipboard.",
    );
    forbid(
        failures,
        &s.e2e_windows_helper,
        ".ps1",
        "WDIO Windows helper must not invoke PowerShell clipboard helpers.",
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
    let (relative, label) = (ZERO_TOUCH_EVIDENCE, "Zero-touch evidence helper");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> Sources {
        Sources {
            node: String::new(),
            node_raw_log: [
                "#[wasm_bindgen(js_name = nodeApplyRuntimeLogReport)]",
                "if raw_log_transport_wrapper_text(&legacy)",
                r#"metadata_text(metadata, "runtimeRole", "node")"#,
            ]
            .join("\n"),
            bridge: String::new(),
            bridge_tab: ["bridge_render_raw_log_buffer("].join("\n"),
            bridge_raw_log: [
                "#[wasm_bindgen(js_name = bridgeApplyRuntimeLogReport)]",
                "#[wasm_bindgen(js_name = bridgeClearRawLogBuffer)]",
                "fn transport_wrapper_text(value: &str) -> bool",
                "if transport_wrapper_text(&legacy_transport_text(report))",
            ]
            .join("\n"),
            bridge_start_trace: [
                r#"metadata_text(metadata, "runtimeRole", "bridge")"#,
                r#"metadata_text(metadata, "bridgeInstanceId", "")"#,
                "crate::bridge_raw_log::bridge_clear_raw_log_buffer(",
                r#"for name in ["kgwT", "kgwI18n", "__kgwT"]"#,
                "crate::bridge_frontend_helpers::bridge_small_owner_trace_r44d(",
                "fn active_runtime_bridge_instance_id(",
                r#"property(deps, "activeInstance")"#,
                "spawn_local(async move",
                r#""kgw_kgw_runtime_clear_logs_v1""#,
                "invoke_runtime_command_impl(",
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
            zero_touch: ["required_pwsh", "KGW_REQUIRED_PWSH_PATH"].join("\n"),
            zero_touch_evidence: [
                "Write-KgwZeroTouchJsonFile",
                "Write-KgwZeroTouchEmergencyJsonFile",
            ]
            .join("\n"),
            full_local_gate: [
                "zero_touch_e2e::run",
                "powershell_parser_checks",
                "true-raw-log-gate",
                "zero-touch-result-writer-tests",
            ]
            .join("\n"),
            e2e_windows_helper: "e2e-clipboard".to_owned(),
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
