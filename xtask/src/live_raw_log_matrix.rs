use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

const DEFAULT_TIMEOUT_SECONDS: u64 = 600;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Options {
    repository: PathBuf,
    timeout_seconds: u64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Stage {
    index: u8,
    name: &'static str,
    slug: &'static str,
    network: &'static str,
    runtime_role: &'static str,
}
const STAGES: [Stage; 3] = [
    Stage {
        index: 1,
        name: "Testnet10 Node",
        slug: "testnet10-node",
        network: "testnet10",
        runtime_role: "node",
    },
    Stage {
        index: 2,
        name: "Mainnet Bridge",
        slug: "mainnet-bridge",
        network: "mainnet",
        runtime_role: "bridge",
    },
    Stage {
        index: 3,
        name: "Testnet10 Bridge",
        slug: "testnet10-bridge",
        network: "testnet10",
        runtime_role: "bridge",
    },
];

pub fn run_cli(
    args: &mut impl Iterator<Item = String>,
    default_repository: &Path,
) -> Result<String, String> {
    let options = parse_options(args, default_repository)?;
    run(&options)
}

fn parse_options(
    args: &mut impl Iterator<Item = String>,
    default_repository: &Path,
) -> Result<Options, String> {
    let mut repository = default_repository.to_path_buf();
    let mut timeout_seconds = DEFAULT_TIMEOUT_SECONDS;
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--repository" => repository = PathBuf::from(value),
            "--timeout-seconds" => {
                timeout_seconds = value
                    .parse::<u64>()
                    .map_err(|_| "--timeout-seconds must be an integer".to_owned())?;
                if timeout_seconds == 0 {
                    return Err("--timeout-seconds must be greater than zero".to_owned());
                }
            }
            _ => return Err(format!("unknown live-raw-log-matrix option: {flag}")),
        }
    }
    Ok(Options {
        repository,
        timeout_seconds,
    })
}

fn run(options: &Options) -> Result<String, String> {
    #[cfg(windows)]
    {
        run_windows(options)
    }
    #[cfg(not(windows))]
    {
        let _ = options;
        Err("live-raw-log-matrix requires Windows".to_owned())
    }
}

#[cfg(windows)]
fn run_windows(options: &Options) -> Result<String, String> {
    let repository = options.repository.canonicalize().map_err(|error| {
        format!(
            "failed to resolve repository {}: {error}",
            options.repository.display()
        )
    })?;
    let run_id = format!("rust-{}", OffsetDateTime::now_utc().unix_timestamp_nanos());
    let run_directory = repository
        .join("artifacts")
        .join("live-raw-log-matrix")
        .join(&run_id);
    let payload_directory = run_directory.join("payloads");
    fs::create_dir_all(&payload_directory)
        .map_err(|error| format!("failed to create {}: {error}", payload_directory.display()))?;

    let stdout_log = run_directory.join("desktop.stdout.log");
    let stderr_log = run_directory.join("desktop.stderr.log");
    let launcher_log = run_directory.join("launcher.log");
    let build_log = run_directory.join("build.log");
    let report_file = run_directory.join("report.json");
    for path in [&stdout_log, &stderr_log, &launcher_log] {
        File::create(path)
            .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
    }

    let executable = repository
        .join("target")
        .join("debug")
        .join("kaspa-gateway-desktop.exe");
    let mut report = json!({
        "run_id": run_id,
        "repository": repository.to_string_lossy(),
        "started_at": OffsetDateTime::now_utc().unix_timestamp(),
        "finished_at": Value::Null,
        "executable": executable.to_string_lossy(),
        "executable_sha256": Value::Null,
        "run_directory": run_directory.to_string_lossy(),
        "logs": {
            "stdout": stdout_log.to_string_lossy(),
            "stderr": stderr_log.to_string_lossy(),
            "launcher": launcher_log.to_string_lossy(),
            "build": build_log.to_string_lossy()
        },
        "stages": [],
        "final_clipboard": Value::Null,
        "warnings": []
    });
    save_report(&report_file, &report)?;
    log_line(
        &launcher_log,
        "INFO",
        &format!("Repository={}", repository.display()),
    )?;
    log_line(
        &launcher_log,
        "INFO",
        &format!("RunDirectory={}", run_directory.display()),
    )?;

    build_if_needed(&repository, &executable, &build_log, &launcher_log)?;
    if !executable.is_file() {
        return Err(format!(
            "debug desktop executable was not found after build: {}",
            executable.display()
        ));
    }
    report["executable_sha256"] = Value::String(file_sha256(&executable)?);
    save_report(&report_file, &report)?;

    let stdout = File::create(&stdout_log)
        .map_err(|error| format!("failed to open {}: {error}", stdout_log.display()))?;
    let stderr = File::create(&stderr_log)
        .map_err(|error| format!("failed to open {}: {error}", stderr_log.display()))?;
    let mut desktop = Command::new(&executable)
        .current_dir(&repository)
        .env("RUST_BACKTRACE", "full")
        .env(
            "RUST_LOG",
            "kaspa_gateway_desktop=trace,kaspa_gateway_runtime=trace,kaspa_gateway_node=trace,info",
        )
        .env("KGW_LOG_LEVEL", "trace")
        .env("KGW_START_TRACE", "1")
        .env(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--enable-logging=stderr --v=1",
        )
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|error| format!("failed to launch desktop executable: {error}"))?;
    log_line(
        &launcher_log,
        "INFO",
        &format!("DesktopPID={}", desktop.id()),
    )?;

    let result = run_stages(
        &repository,
        &payload_directory,
        &stdout_log,
        &stderr_log,
        &launcher_log,
        &report_file,
        &mut report,
        &mut desktop,
        options.timeout_seconds,
    );

    report["final_clipboard"] = crate::e2e_clipboard::read_snapshot_metadata()
        .unwrap_or_else(|error| json!({"error": error, "informational_only": true}));
    report["finished_at"] = Value::from(OffsetDateTime::now_utc().unix_timestamp());
    save_report(&report_file, &report)?;

    if desktop.try_wait().ok().flatten().is_none() {
        let _ = desktop.kill();
        let _ = desktop.wait();
    }

    result?;
    log_line(
        &launcher_log,
        "INFO",
        &format!(
            "Live raw log matrix PASSED. Report={}",
            report_file.display()
        ),
    )?;
    Ok(format!(
        "LIVE_RAW_LOG_MATRIX=PASS\nREPORT={}",
        report_file.display()
    ))
}

#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
fn run_stages(
    repository: &Path,
    payload_directory: &Path,
    stdout_log: &Path,
    stderr_log: &Path,
    launcher_log: &Path,
    report_file: &Path,
    report: &mut Value,
    desktop: &mut Child,
    timeout_seconds: u64,
) -> Result<(), String> {
    let mut stdout_lines = 0_usize;
    let mut stderr_lines = 0_usize;

    for stage in STAGES {
        println!("\nPHYSICAL STAGE: {}", stage.name);
        println!("1. Use the desktop app to select {}.", stage.name);
        println!("2. Start the runtime from Settings only, then open its Live Monitor.");
        println!("3. Wait until native child stdout/stderr is visible.");
        println!("4. Click Copy Log while this Rust matrix is watching.");

        let sentinel = format!(
            "KGW_LIVE_RAW_LOG_MATRIX_SENTINEL|run={}|stage={}|{}",
            report["run_id"].as_str().unwrap_or("unknown"),
            stage.index,
            unique_suffix()
        );
        let preflight = crate::e2e_clipboard::ensure_text_only_write_safe()?;
        crate::e2e_clipboard::write_text_after_preflight(&sentinel)?;
        log_line(
            launcher_log,
            "INFO",
            &format!("Clipboard sentinel set after fail-closed preflight: {preflight}"),
        )?;

        println!(
            "Press Enter to start watching for {}, then perform the steps above.",
            stage.name
        );
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .map_err(|error| format!("failed to read stage confirmation: {error}"))?;

        let capture = wait_for_stage_capture(
            repository,
            payload_directory,
            stdout_log,
            stderr_log,
            launcher_log,
            desktop,
            stage,
            timeout_seconds,
            &mut stdout_lines,
            &mut stderr_lines,
        )?;
        let stage_result = validate_stage_capture(stage, &capture);
        report
            .get_mut("stages")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| "report stages must be an array".to_owned())?
            .push(stage_result.clone());
        save_report(report_file, report)?;

        if !stage_result["passed"].as_bool().unwrap_or(false) {
            return Err(format!(
                "{} validation failed. Report={}",
                stage.name,
                report_file.display()
            ));
        }
    }
    Ok(())
}

#[cfg(windows)]
#[allow(clippy::too_many_arguments)]
fn wait_for_stage_capture(
    repository: &Path,
    payload_directory: &Path,
    stdout_log: &Path,
    stderr_log: &Path,
    launcher_log: &Path,
    desktop: &mut Child,
    stage: Stage,
    timeout_seconds: u64,
    stdout_lines: &mut usize,
    stderr_lines: &mut usize,
) -> Result<Value, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let mut captures = Vec::new();

    while Instant::now() < deadline {
        if let Some(status) = desktop
            .try_wait()
            .map_err(|error| format!("failed to inspect desktop process: {error}"))?
        {
            return Err(format!(
                "desktop process exited before {} Copy Log was captured. status={status}",
                stage.name
            ));
        }

        for (path, count, label) in [
            (stdout_log, &mut *stdout_lines, "STDOUT"),
            (stderr_log, &mut *stderr_lines, "STDERR"),
        ] {
            for line in read_new_lines(path, count)? {
                if line.starts_with("[KGW_CHILD_STD") || is_clipboard_trace(&line) {
                    log_line(launcher_log, label, &line)?;
                }
                if !is_clipboard_success_trace(&line) {
                    continue;
                }

                let capture = capture_from_clipboard(repository, payload_directory, &line, stage)?;
                let matches = capture_matches_stage(stage, &capture);
                captures.push(capture.clone());
                if matches {
                    return Ok(capture);
                }
            }
        }
        thread::sleep(Duration::from_millis(250));
    }

    let summary = captures
        .iter()
        .map(|value| {
            format!(
                "{}/{}/{}/valid={}",
                value["event_stage"].as_str().unwrap_or("unknown"),
                value["network"].as_str().unwrap_or("unknown"),
                value["runtime_role"].as_str().unwrap_or("unknown"),
                value["valid"].as_bool().unwrap_or(false)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Err(format!(
        "timed out waiting for {} Copy Log event. Captures seen: {summary}",
        stage.name
    ))
}

fn is_clipboard_trace(line: &str) -> bool {
    line.starts_with("[KGW_START_TRACE]")
        && (line.contains("copy_log_") || line.contains("clipboard_write_"))
}

fn is_clipboard_success_trace(line: &str) -> bool {
    line.starts_with("[KGW_START_TRACE]")
        && (line.contains("native.clipboard_write_succeeded")
            || line.contains("frontend.copy_log_succeeded"))
}

fn capture_matches_stage(stage: Stage, capture: &Value) -> bool {
    capture["valid"].as_bool().unwrap_or(false)
        && capture["network"].as_str() == Some(stage.network)
        && capture["runtime_role"].as_str() == Some(stage.runtime_role)
        && (stage.runtime_role != "bridge"
            || capture["bridge_instance_id"]
                .as_str()
                .is_some_and(|value| !value.trim().is_empty()))
}

#[cfg(windows)]
fn validate_stage_capture(stage: Stage, capture: &Value) -> Value {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if !capture["valid"].as_bool().unwrap_or(false)
        && let Some(values) = capture["errors"].as_array()
    {
        errors.extend(values.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    if capture["network"].as_str() != Some(stage.network) {
        errors.push(format!(
            "capture network did not match expected {}",
            stage.network
        ));
    }
    if capture["runtime_role"].as_str() != Some(stage.runtime_role) {
        errors.push(format!(
            "capture runtime role did not match expected {}",
            stage.runtime_role
        ));
    }
    if stage.runtime_role == "bridge"
        && !capture["bridge_instance_id"]
            .as_str()
            .is_some_and(|value| !value.trim().is_empty())
    {
        errors.push("bridge capture did not include bridge instance identity".to_owned());
    }
    if let Some(values) = capture["acceptance_errors"].as_array() {
        errors.extend(values.iter().filter_map(Value::as_str).map(str::to_owned));
    }
    if let Some(values) = capture["acceptance_warnings"].as_array() {
        warnings.extend(values.iter().filter_map(Value::as_str).map(str::to_owned));
    }

    json!({
        "stage": stage.name,
        "network": stage.network,
        "runtime_role": stage.runtime_role,
        "bridge_instance_id": capture["bridge_instance_id"].clone(),
        "passed": errors.is_empty(),
        "errors": errors,
        "warnings": warnings,
        "capture": capture,
        "stage_payload_file": capture["payload_file"].clone()
    })
}

#[cfg(windows)]
fn capture_from_clipboard(
    _repository: &Path,
    payload_directory: &Path,
    trace_line: &str,
    stage: Stage,
) -> Result<Value, String> {
    crate::raw_log_clipboard_capture::capture_with_acceptance_from_clipboard(
        trace_line,
        payload_directory,
        &format!("matrix-{}", stage.slug),
        stage.network,
        stage.runtime_role,
    )
}

fn read_new_lines(path: &Path, line_count: &mut usize) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    if lines.len() <= *line_count {
        return Ok(Vec::new());
    }
    let output = lines[*line_count..].to_vec();
    *line_count = lines.len();
    Ok(output)
}

fn build_if_needed(
    repository: &Path,
    executable: &Path,
    build_log: &Path,
    launcher_log: &Path,
) -> Result<(), String> {
    if executable.is_file() {
        log_line(
            launcher_log,
            "INFO",
            "Debug desktop executable exists; source freshness remains a qualification prerequisite for live execution.",
        )?;
        return Ok(());
    }

    let first = cargo_build(repository, None)?;
    fs::write(build_log, &first.1)
        .map_err(|error| format!("failed to write {}: {error}", build_log.display()))?;
    if first.0 {
        return Ok(());
    }
    if !librocksdb_artifact_inconsistency(&first.1) {
        return Err(format!(
            "desktop build failed. Build log: {}",
            build_log.display()
        ));
    }

    let isolated = repository
        .join("artifacts")
        .join("cargo-targets")
        .join(format!("live-raw-log-matrix-{}", unique_suffix()));
    fs::create_dir_all(&isolated)
        .map_err(|error| format!("failed to create {}: {error}", isolated.display()))?;
    let retry = cargo_build(repository, Some(&isolated))?;
    append_text(build_log, &retry.1)?;
    if retry.0 {
        Ok(())
    } else {
        Err(format!(
            "desktop build failed after isolated target retry. Build log: {}",
            build_log.display()
        ))
    }
}
fn cargo_build(repository: &Path, target: Option<&Path>) -> Result<(bool, String), String> {
    let mut command = Command::new("cargo");
    command
        .current_dir(repository)
        .args(["build", "--locked", "--bin", "kaspa-gateway-desktop"]);
    if let Some(target) = target {
        command.env("CARGO_TARGET_DIR", target);
    }
    let output = command
        .output()
        .map_err(|error| format!("failed to launch cargo build: {error}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok((output.status.success(), text))
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

fn save_report(path: &Path, report: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("failed to serialize report: {error}"))?;
    fs::write(path, bytes).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn log_line(path: &Path, level: &str, message: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    writeln!(
        file,
        "[{}][{}] {}",
        OffsetDateTime::now_utc().unix_timestamp_nanos(),
        level,
        message
    )
    .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn append_text(path: &Path, text: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("failed to append {}: {error}", path.display()))
}

fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn unique_suffix() -> String {
    format!(
        "{}-{}",
        std::process::id(),
        OffsetDateTime::now_utc().unix_timestamp_nanos()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_order_preserves_physical_matrix_contract() {
        assert_eq!(
            STAGES.map(|stage| (stage.name, stage.network, stage.runtime_role)),
            [
                ("Testnet10 Node", "testnet10", "node"),
                ("Mainnet Bridge", "mainnet", "bridge"),
                ("Testnet10 Bridge", "testnet10", "bridge"),
            ]
        );
    }

    #[test]
    fn clipboard_success_detection_is_fail_closed() {
        assert!(is_clipboard_success_trace(
            "[KGW_START_TRACE] native.clipboard_write_succeeded"
        ));
        assert!(is_clipboard_success_trace(
            "[KGW_START_TRACE] frontend.copy_log_succeeded"
        ));
        assert!(!is_clipboard_success_trace(
            "[KGW_START_TRACE] native.clipboard_write_entered"
        ));
        assert!(!is_clipboard_success_trace("frontend.copy_log_succeeded"));
    }
    #[test]
    fn bridge_capture_requires_instance_identity() {
        let stage = STAGES[1];
        let missing = json!({
            "valid": true,
            "network": "mainnet",
            "runtime_role": "bridge",
            "bridge_instance_id": ""
        });
        assert!(!capture_matches_stage(stage, &missing));

        let present = json!({
            "valid": true,
            "network": "mainnet",
            "runtime_role": "bridge",
            "bridge_instance_id": "bridge-1"
        });
        assert!(capture_matches_stage(stage, &present));
    }

    #[test]
    fn parser_defaults_are_stable() {
        let mut args = Vec::<String>::new().into_iter();
        let root = Path::new("repo");
        let options = parse_options(&mut args, root).expect("defaults");
        assert_eq!(options.repository, root);
        assert_eq!(options.timeout_seconds, 600);
    }
}
