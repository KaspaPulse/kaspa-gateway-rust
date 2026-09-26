use regex::Regex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use sysinfo::System;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const EXPECTED_NODE_PORTS: [u16; 2] = [16110, 16210];

#[derive(Debug, Clone, Eq, PartialEq)]
struct Options {
    repository: PathBuf,
}

#[derive(Debug, Default)]
struct ClipboardState {
    initial: Option<Value>,
    latest: Option<Value>,
    event_captures: Vec<Value>,
}

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
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--repository" => repository = PathBuf::from(value),
            _ => return Err(format!("unknown desktop-diagnostic option: {flag}")),
        }
    }
    Ok(Options { repository })
}

fn run(options: &Options) -> Result<String, String> {
    #[cfg(windows)]
    {
        run_windows(options)
    }
    #[cfg(not(windows))]
    {
        let _ = options;
        Err("desktop-diagnostic requires Windows".to_owned())
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
    let run_id = format!("rust-{}", OffsetDateTime::now_utc().unix_timestamp());
    let diagnostic_root = repository.join("artifacts").join("desktop-diagnostics");
    let diagnostic_directory = diagnostic_root.join(&run_id);
    fs::create_dir_all(&diagnostic_directory).map_err(|error| {
        format!(
            "failed to create diagnostic directory {}: {error}",
            diagnostic_directory.display()
        )
    })?;

    let stdout_log = diagnostic_directory.join("desktop.stdout.log");
    let stderr_log = diagnostic_directory.join("desktop.stderr.log");
    let launcher_log = diagnostic_directory.join("launcher.log");
    let process_log = diagnostic_directory.join("processes.log");
    let port_log = diagnostic_directory.join("ports.log");
    let child_process_log = diagnostic_directory.join("child-process.log");
    let clipboard_log = diagnostic_directory.join("clipboard.log");
    let clipboard_payload_directory = diagnostic_directory.join("clipboard-payloads");
    let windows_event_log = diagnostic_directory.join("windows-events.log");
    let summary_file = diagnostic_directory.join("summary.json");
    let build_log = diagnostic_directory.join("build.log");
    let zip_file = diagnostic_root.join(format!("{run_id}.zip"));
    fs::create_dir_all(&clipboard_payload_directory)
        .map_err(|error| format!("failed to create clipboard payload directory: {error}"))?;
    for path in [
        &stdout_log,
        &stderr_log,
        &launcher_log,
        &process_log,
        &port_log,
        &child_process_log,
        &clipboard_log,
    ] {
        File::create(path)
            .map_err(|error| format!("failed to create {}: {error}", path.display()))?;
    }

    let branch = command_text(&repository, "git", &["branch", "--show-current"])?;
    let commit = command_text(&repository, "git", &["rev-parse", "HEAD"])?;
    let status = command_text(&repository, "git", &["status", "--short"])?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("Repository={}", repository.display()),
    )?;
    diagnostic(&launcher_log, "INFO", &format!("Branch={}", branch.trim()))?;
    diagnostic(&launcher_log, "INFO", &format!("Commit={}", commit.trim()))?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("DiagnosticDirectory={}", diagnostic_directory.display()),
    )?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!(
            "ExpectedNodePorts={}",
            EXPECTED_NODE_PORTS.map(|v| v.to_string()).join(",")
        ),
    )?;
    if status.trim().is_empty() {
        diagnostic(&launcher_log, "INFO", "WorktreeStatus=clean")?;
    } else {
        diagnostic(&launcher_log, "WARN", "WorktreeStatus=dirty")?;
        for line in status.lines() {
            diagnostic(&launcher_log, "WARN", &format!("GitStatus={line}"))?;
        }
    }

    stop_existing_repository_desktops(&repository, &launcher_log)?;
    thread::sleep(Duration::from_secs(2));

    let executable = repository
        .join("target")
        .join("debug")
        .join("kaspa-gateway-desktop.exe");
    if !executable.is_file() {
        diagnostic(
            &launcher_log,
            "INFO",
            "Debug executable was not found. Starting local build.",
        )?;
        run_build(&repository, &build_log)?;
    }
    if !executable.is_file() {
        return Err(format!(
            "desktop executable is missing after build: {}",
            executable.display()
        ));
    }

    let executable_bytes =
        fs::read(&executable).map_err(|error| format!("failed to read executable: {error}"))?;
    let executable_hash = format!("{:x}", Sha256::digest(&executable_bytes));
    let executable_meta =
        fs::metadata(&executable).map_err(|error| format!("failed to stat executable: {error}"))?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("Executable={}", executable.display()),
    )?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("ExecutableSHA256={executable_hash}"),
    )?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("ExecutableLength={}", executable_meta.len()),
    )?;

    let mut clipboard = ClipboardState::default();
    clipboard_snapshot(
        &launcher_log,
        &clipboard_log,
        "before-launch",
        true,
        &mut clipboard,
    )?;

    let stdout_file =
        File::create(&stdout_log).map_err(|error| format!("failed to open stdout log: {error}"))?;
    let stderr_file =
        File::create(&stderr_log).map_err(|error| format!("failed to open stderr log: {error}"))?;
    let started_at = OffsetDateTime::now_utc();
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
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|error| format!("failed to launch desktop executable: {error}"))?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("DesktopPID={}", desktop.id()),
    )?;
    diagnostic(
        &launcher_log,
        "INFO",
        "Use the application now, reproduce the issue if needed, then close the desktop application.",
    )?;

    let mut stdout_count = 0usize;
    let mut stderr_count = 0usize;
    let mut previous_process_snapshot = String::new();
    let mut previous_port_snapshot = String::new();
    let mut poll_counter = 0u64;
    let mut log_context = LogLineContext {
        repository: &repository,
        launcher_log: &launcher_log,
        child_log: &child_process_log,
        clipboard_log: &clipboard_log,
        clipboard_payload_directory: &clipboard_payload_directory,
        clipboard: &mut clipboard,
    };

    let exit_code = loop {
        show_new_log_lines(&stdout_log, &mut stdout_count, "STDOUT", &mut log_context)?;
        show_new_log_lines(&stderr_log, &mut stderr_count, "STDERR", &mut log_context)?;
        write_process_snapshot(
            &repository,
            Some(desktop.id()),
            &process_log,
            &launcher_log,
            &mut previous_process_snapshot,
        )?;
        if poll_counter.is_multiple_of(3) {
            write_port_snapshot(&port_log, &launcher_log, &mut previous_port_snapshot)?;
        }
        poll_counter += 1;

        if let Some(status) = desktop
            .try_wait()
            .map_err(|error| format!("failed to poll desktop process: {error}"))?
        {
            break status.code();
        }
        thread::sleep(Duration::from_secs(1));
    };

    show_new_log_lines(&stdout_log, &mut stdout_count, "STDOUT", &mut log_context)?;
    show_new_log_lines(&stderr_log, &mut stderr_count, "STDERR", &mut log_context)?;
    write_process_snapshot(
        &repository,
        Some(desktop.id()),
        &process_log,
        &launcher_log,
        &mut previous_process_snapshot,
    )?;
    write_port_snapshot(&port_log, &launcher_log, &mut previous_port_snapshot)?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("Desktop process exited. ExitCode={exit_code:?}"),
    )?;

    capture_windows_events(&windows_event_log, &launcher_log)?;
    clipboard_snapshot(
        &launcher_log,
        &clipboard_log,
        "after-application-close",
        false,
        &mut clipboard,
    )?;

    let finished_at = OffsetDateTime::now_utc();
    let child_summary = child_process_summary(&child_process_log)?;
    let clipboard_summary = clipboard_summary(&clipboard, &clipboard_log)?;
    let summary = json!({
        "repository": repository.to_string_lossy(),
        "branch": branch.trim(),
        "commit": commit.trim(),
        "executable": executable.to_string_lossy(),
        "executable_sha256": executable_hash,
        "kgw_start_trace": "1",
        "expected_node_ports": EXPECTED_NODE_PORTS,
        "child_process_evidence": child_summary,
        "clipboard_evidence": clipboard_summary,
        "desktop_pid": desktop.id(),
        "exit_code": exit_code,
        "started_at": format_time(started_at),
        "finished_at": format_time(finished_at),
        "duration_seconds": (finished_at - started_at).as_seconds_f64(),
        "worktree_was_clean": status.trim().is_empty(),
        "logs": {
            "launcher": launcher_log.to_string_lossy(),
            "stdout": stdout_log.to_string_lossy(),
            "stderr": stderr_log.to_string_lossy(),
            "child_process": child_process_log.to_string_lossy(),
            "clipboard": clipboard_log.to_string_lossy(),
            "processes": process_log.to_string_lossy(),
            "ports": port_log.to_string_lossy(),
            "windows_events": windows_event_log.to_string_lossy(),
            "build": build_log.to_string_lossy(),
        }
    });
    fs::write(
        &summary_file,
        serde_json::to_vec_pretty(&summary)
            .map_err(|error| format!("failed to serialize diagnostic summary: {error}"))?,
    )
    .map_err(|error| format!("failed to write {}: {error}", summary_file.display()))?;

    create_zip_archive(&diagnostic_directory, &zip_file)?;
    diagnostic(&launcher_log, "INFO", "Diagnostic capture completed.")?;
    diagnostic(
        &launcher_log,
        "INFO",
        &format!("DiagnosticArchive={}", zip_file.display()),
    )?;
    Ok(format!(
        "KGW desktop diagnostic capture completed\nDiagnosticArchive={}",
        zip_file.display()
    ))
}

fn diagnostic(path: &Path, level: &str, message: &str) -> Result<(), String> {
    let line = format!(
        "[{}][{level}] {message}",
        format_time(OffsetDateTime::now_utc())
    );
    println!("{line}");
    append_line(path, &line)
}

fn append_line(path: &Path, line: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    writeln!(file, "{line}")
        .map_err(|error| format!("failed to append {}: {error}", path.display()))
}

fn format_time(value: OffsetDateTime) -> String {
    value
        .format(&Rfc3339)
        .unwrap_or_else(|_| value.unix_timestamp().to_string())
}

fn command_text(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_build(repository: &Path, build_log: &Path) -> Result<(), String> {
    let stdout = File::create(build_log)
        .map_err(|error| format!("failed to create {}: {error}", build_log.display()))?;
    let stderr = stdout
        .try_clone()
        .map_err(|error| format!("failed to clone build log handle: {error}"))?;
    let status = Command::new("cargo")
        .args(["build", "--locked", "--bin", "kaspa-gateway-desktop"])
        .current_dir(repository)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .status()
        .map_err(|error| format!("failed to launch cargo build: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "desktop build failed with status {status}; BuildLog={}",
            build_log.display()
        ))
    }
}

#[cfg(windows)]
struct LogLineContext<'a> {
    repository: &'a Path,
    launcher_log: &'a Path,
    child_log: &'a Path,
    clipboard_log: &'a Path,
    clipboard_payload_directory: &'a Path,
    clipboard: &'a mut ClipboardState,
}

#[cfg(windows)]
fn stop_existing_repository_desktops(repository: &Path, launcher_log: &Path) -> Result<(), String> {
    let repository_lower = repository.to_string_lossy().to_ascii_lowercase();
    let system = System::new_all();
    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy();
        let executable = process
            .exe()
            .map(|path| path.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if name.eq_ignore_ascii_case("kaspa-gateway-desktop.exe")
            && executable.starts_with(&repository_lower)
        {
            diagnostic(
                launcher_log,
                "WARN",
                &format!(
                    "Stopping existing repository-owned process PID={}",
                    pid.as_u32()
                ),
            )?;
            if !process.kill() {
                return Err(format!(
                    "failed to stop existing repository-owned process PID={}",
                    pid.as_u32()
                ));
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
fn show_new_log_lines(
    path: &Path,
    line_count: &mut usize,
    label: &str,
    context: &mut LogLineContext<'_>,
) -> Result<(), String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= *line_count {
        return Ok(());
    }
    for line in &lines[*line_count..] {
        if line.starts_with("[KGW_CHILD_STDERR]") || line.starts_with("[KGW_CHILD_STDOUT]") {
            append_line(context.child_log, line)?;
            println!("[{label}] {line}");
            continue;
        }
        if line.starts_with("[KGW_START_TRACE]") {
            if line.contains("native.child_pid_recorded")
                || line.contains("self-worker-exited-during-startup")
            {
                append_line(context.child_log, line)?;
            }
            if is_clipboard_trace(line) {
                append_line(context.clipboard_log, line)?;
                if line.contains("copy_log_succeeded") || line.contains("clipboard_write_succeeded")
                {
                    println!("[{label}] {line}");
                    match clipboard_event_capture(
                        context.repository,
                        context.clipboard_payload_directory,
                        line,
                    ) {
                        Ok(capture) => {
                            append_line(
                                context.clipboard_log,
                                &serde_json::to_string(&capture).map_err(|error| {
                                    format!("clipboard capture serialization failed: {error}")
                                })?,
                            )?;
                            context.clipboard.event_captures.push(capture);
                        }
                        Err(error) => {
                            diagnostic(
                                context.launcher_log,
                                "ERROR",
                                &format!(
                                    "ClipboardEventCapture failed: {}",
                                    safe_diagnostic_text(&error)
                                ),
                            )?;
                        }
                    }
                    continue;
                }
                if line.contains("copy_log_failed") || line.contains("clipboard_write_failed") {
                    clipboard_snapshot(
                        context.launcher_log,
                        context.clipboard_log,
                        "after-clipboard-failure-trace",
                        false,
                        context.clipboard,
                    )?;
                    continue;
                }
                if line.contains("copy_log_dispatched") || line.contains("clipboard_write_entered")
                {
                    clipboard_snapshot(
                        context.launcher_log,
                        context.clipboard_log,
                        "after-clipboard-dispatch-trace",
                        false,
                        context.clipboard,
                    )?;
                }
                continue;
            }
            if is_runtime_polling_trace(line) {
                continue;
            }
            println!("[{label}] {line}");
            continue;
        }
        println!("[{label}] {line}");
    }
    *line_count = lines.len();
    Ok(())
}

fn is_runtime_polling_trace(line: &str) -> bool {
    line.starts_with("[KGW_START_TRACE]")
        && (line.contains("kgw_runtime_owner_status_v1")
            || line.contains("kgw_kgw_runtime_logs_v1"))
}

fn is_clipboard_trace(line: &str) -> bool {
    line.contains("copy_log_") || line.contains("clipboard_write_")
}

fn safe_diagnostic_text(value: &str) -> String {
    let clean = value.replace(['\r', '\n', '\t'], " ").trim().to_owned();
    let lower = clean.to_ascii_lowercase();
    if [
        "secret", "token", "private", "mnemonic", "wallet", "address",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return "redacted-sensitive-value".to_owned();
    }
    clean.chars().take(220).collect()
}

#[cfg(windows)]
fn clipboard_snapshot(
    launcher_log: &Path,
    clipboard_log: &Path,
    reason: &str,
    initial: bool,
    clipboard: &mut ClipboardState,
) -> Result<(), String> {
    let timestamp = format_time(OffsetDateTime::now_utc());
    let snapshot = match crate::e2e_clipboard::read_snapshot_metadata() {
        Ok(metadata) => json!({
            "timestamp": timestamp,
            "reason": reason,
            "available": true,
            "character_count": metadata["character_count"],
            "line_count": metadata["line_count"],
            "sha256": metadata["sha256"],
        }),
        Err(error) => json!({
            "timestamp": timestamp,
            "reason": reason,
            "available": false,
            "character_count": Value::Null,
            "line_count": Value::Null,
            "sha256": Value::Null,
            "error": safe_diagnostic_text(&error),
        }),
    };
    if initial {
        clipboard.initial = Some(snapshot.clone());
    }
    clipboard.latest = Some(snapshot.clone());
    append_line(
        clipboard_log,
        &serde_json::to_string(&snapshot)
            .map_err(|error| format!("clipboard snapshot serialization failed: {error}"))?,
    )?;
    let changed = clipboard
        .initial
        .as_ref()
        .and_then(|value| value["sha256"].as_str())
        .zip(snapshot["sha256"].as_str())
        .is_some_and(|(left, right)| left != right);
    diagnostic(
        launcher_log,
        "INFO",
        &format!(
            "ClipboardSnapshot reason={reason} available={} characters={} lines={} sha256={} changed_from_launch={changed}",
            snapshot["available"],
            snapshot["character_count"],
            snapshot["line_count"],
            snapshot["sha256"]
        ),
    )
}

#[cfg(windows)]
fn clipboard_event_capture(
    repository: &Path,
    output_directory: &Path,
    trace_line: &str,
) -> Result<Value, String> {
    let helper = repository
        .join("tools")
        .join("kgw_raw_log_clipboard_capture.ps1");
    let script = "$ErrorActionPreference='Stop'; . $env:KGW_CLIPBOARD_HELPER; $r=New-KgwRawLogClipboardCaptureFromClipboardV1 -TraceLine $env:KGW_TRACE_LINE -OutputDirectory $env:KGW_CAPTURE_DIR -Reason 'clipboard-success-trace'; $r | ConvertTo-Json -Depth 6 -Compress";
    let output = Command::new("pwsh")
        .args(["-NoLogo", "-NoProfile", "-Command", script])
        .env("KGW_CLIPBOARD_HELPER", &helper)
        .env("KGW_TRACE_LINE", trace_line)
        .env("KGW_CAPTURE_DIR", output_directory)
        .current_dir(repository)
        .output()
        .map_err(|error| format!("failed to launch clipboard helper: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "clipboard helper failed: {}",
            safe_diagnostic_text(&String::from_utf8_lossy(&output.stderr))
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("clipboard helper returned invalid JSON: {error}"))
}

#[cfg(windows)]
fn write_process_snapshot(
    repository: &Path,
    desktop_pid: Option<u32>,
    process_log: &Path,
    launcher_log: &Path,
    previous_snapshot: &mut String,
) -> Result<(), String> {
    let values = crate::e2e_windows_evidence::capture_process_values(repository, desktop_pid)?;
    let summarized: Vec<Value> = values
        .iter()
        .map(|value| {
            let command_line = value["CommandLine"].as_str().unwrap_or_default();
            json!({
                "ProcessId": value["ProcessId"],
                "ParentProcessId": value["ParentProcessId"],
                "Name": value["Name"],
                "ExecutablePath": value["ExecutablePath"],
                "CommandLineSummary": command_line_summary(command_line),
            })
        })
        .collect();
    let snapshot = serde_json::to_string(&summarized)
        .map_err(|error| format!("process snapshot serialization failed: {error}"))?;
    if snapshot == *previous_snapshot {
        return Ok(());
    }
    append_line(
        process_log,
        &format!("[{}]", format_time(OffsetDateTime::now_utc())),
    )?;
    if summarized.is_empty() {
        append_line(process_log, "No Kaspa Gateway processes detected.")?;
        diagnostic(
            launcher_log,
            "PROCESS",
            "No Kaspa Gateway processes detected.",
        )?;
    } else {
        append_line(
            process_log,
            &serde_json::to_string_pretty(&summarized)
                .map_err(|error| format!("process snapshot serialization failed: {error}"))?,
        )?;
        for value in &summarized {
            diagnostic(
                launcher_log,
                "PROCESS",
                &format!(
                    "PID={} ParentPID={} Name={} CommandLineSummary={}",
                    value["ProcessId"],
                    value["ParentProcessId"],
                    value["Name"],
                    value["CommandLineSummary"]
                ),
            )?;
        }
    }
    *previous_snapshot = snapshot;
    Ok(())
}

fn command_line_summary(command_line: &str) -> String {
    if command_line.trim().is_empty() {
        return "unavailable".to_owned();
    }
    format!(
        "length={};desktop={};self_worker={};webview={}",
        command_line.chars().count(),
        command_line.contains("kaspa-gateway-desktop.exe"),
        command_line.contains("--kgw-self-worker"),
        command_line.contains("embedded-browser-webview")
            || command_line.contains("msedgewebview2")
    )
}

#[cfg(windows)]
fn write_port_snapshot(
    port_log: &Path,
    launcher_log: &Path,
    previous_snapshot: &mut String,
) -> Result<(), String> {
    let values = crate::e2e_windows_evidence::capture_port_values(&EXPECTED_NODE_PORTS)?;
    let snapshot = serde_json::to_string(&values)
        .map_err(|error| format!("port snapshot serialization failed: {error}"))?;
    if snapshot == *previous_snapshot {
        return Ok(());
    }
    append_line(
        port_log,
        &format!("[{}]", format_time(OffsetDateTime::now_utc())),
    )?;
    if values.is_empty() {
        append_line(port_log, "No matching TCP connections detected.")?;
        diagnostic(
            launcher_log,
            "PORT",
            "No matching TCP connections detected.",
        )?;
    } else {
        append_line(
            port_log,
            &serde_json::to_string_pretty(&values)
                .map_err(|error| format!("port snapshot serialization failed: {error}"))?,
        )?;
        for value in &values {
            diagnostic(
                launcher_log,
                "PORT",
                &format!(
                    "PID={} State={} Local={}:{} Remote={}:{}",
                    value["OwningProcess"],
                    value["State"],
                    value["LocalAddress"],
                    value["LocalPort"],
                    value["RemoteAddress"],
                    value["RemotePort"]
                ),
            )?;
        }
    }
    *previous_snapshot = snapshot;
    Ok(())
}

fn child_process_summary(path: &Path) -> Result<Value, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let pid_regex = Regex::new(r#"\"?pid\"?\s*:\s*(?<pid>[0-9]+)"#)
        .map_err(|error| format!("failed to compile child PID regex: {error}"))?;
    let mut pids = BTreeSet::new();
    let mut exit_markers = Vec::new();
    for line in text.lines() {
        for capture in pid_regex.captures_iter(line) {
            if let Some(pid) = capture.name("pid") {
                pids.insert(pid.as_str().to_owned());
            }
        }
        if line.contains("self-worker-exited-during-startup") {
            exit_markers.push(line.to_owned());
        }
    }
    Ok(json!({
        "line_count": text.lines().count(),
        "pids": pids,
        "exit_markers": exit_markers,
    }))
}

fn clipboard_summary(state: &ClipboardState, clipboard_log: &Path) -> Result<Value, String> {
    let line_count = match fs::read_to_string(clipboard_log) {
        Ok(text) => text.lines().count(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => {
            return Err(format!(
                "failed to read {}: {error}",
                clipboard_log.display()
            ));
        }
    };
    let successful = state
        .event_captures
        .iter()
        .filter(|value| value["valid"].as_bool() == Some(true))
        .count();
    let failed = state.event_captures.len().saturating_sub(successful);
    let changed = state
        .initial
        .as_ref()
        .and_then(|value| value["sha256"].as_str())
        .zip(
            state
                .latest
                .as_ref()
                .and_then(|value| value["sha256"].as_str()),
        )
        .is_some_and(|(left, right)| left != right);
    Ok(json!({
        "log": clipboard_log.to_string_lossy(),
        "line_count": line_count,
        "initial": state.initial,
        "latest": state.latest,
        "event_captures": state.event_captures,
        "successful_event_capture_count": successful,
        "failed_event_capture_count": failed,
        "changed_from_launch": changed,
    }))
}

fn capture_windows_events(path: &Path, launcher_log: &Path) -> Result<(), String> {
    let output = Command::new("wevtutil")
        .args(["qe", "Application", "/rd:true", "/f:text", "/c:200"])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            let selected: Vec<&str> = text
                .lines()
                .filter(|line| {
                    let lower = line.to_ascii_lowercase();
                    lower.contains("kaspa-gateway")
                        || lower.contains("webview2")
                        || lower.contains("application error")
                        || lower.contains("windows error reporting")
                })
                .collect();
            if selected.is_empty() {
                fs::write(path, "No matching Windows application events were found.\n")
                    .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
                diagnostic(
                    launcher_log,
                    "INFO",
                    "No matching Windows application events were found.",
                )
            } else {
                fs::write(path, selected.join("\n"))
                    .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
                diagnostic(
                    launcher_log,
                    "INFO",
                    &format!(
                        "Captured {} matching Windows application event lines.",
                        selected.len()
                    ),
                )
            }
        }
        Ok(output) => {
            let message = safe_diagnostic_text(&String::from_utf8_lossy(&output.stderr));
            fs::write(
                path,
                format!("Windows event collection failed: {message}\n"),
            )
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
            diagnostic(
                launcher_log,
                "WARN",
                &format!("Windows event collection failed: {message}"),
            )
        }
        Err(error) => {
            let message = safe_diagnostic_text(&error.to_string());
            fs::write(
                path,
                format!("Windows event collection failed: {message}\n"),
            )
            .map_err(|write_error| format!("failed to write {}: {write_error}", path.display()))?;
            diagnostic(
                launcher_log,
                "WARN",
                &format!("Windows event collection failed: {message}"),
            )
        }
    }
}

fn create_zip_archive(directory: &Path, zip_file: &Path) -> Result<(), String> {
    if zip_file.exists() {
        fs::remove_file(zip_file)
            .map_err(|error| format!("failed to remove {}: {error}", zip_file.display()))?;
    }
    let status = Command::new("tar")
        .args(["-a", "-c", "-f"])
        .arg(zip_file)
        .arg("-C")
        .arg(directory)
        .arg(".")
        .status()
        .map_err(|error| format!("failed to launch Windows tar for ZIP creation: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ZIP archive creation failed with status {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_defaults_repository_and_accepts_override() {
        let root = Path::new(r"C:\repo");
        let mut none = Vec::<String>::new().into_iter();
        assert_eq!(parse_options(&mut none, root).unwrap().repository, root);
        let mut args = vec!["--repository".to_owned(), r"D:\kgw".to_owned()].into_iter();
        assert_eq!(
            parse_options(&mut args, root).unwrap().repository,
            PathBuf::from(r"D:\kgw")
        );
    }

    #[test]
    fn trace_classification_matches_legacy_contract() {
        assert!(is_runtime_polling_trace(
            "[KGW_START_TRACE] invoke kgw_runtime_owner_status_v1"
        ));
        assert!(!is_runtime_polling_trace(
            "[KGW_START_TRACE] native.child_pid_recorded"
        ));
        assert!(is_clipboard_trace("[KGW_START_TRACE] copy_log_succeeded"));
        assert!(is_clipboard_trace(
            "[KGW_START_TRACE] clipboard_write_entered"
        ));
    }

    #[test]
    fn command_line_summary_never_emits_raw_arguments() {
        let line = r#"C:\repo\kaspa-gateway-desktop.exe --kgw-self-worker secret-value"#;
        let summary = command_line_summary(line);
        assert!(summary.contains("desktop=true"));
        assert!(summary.contains("self_worker=true"));
        assert!(!summary.contains("secret-value"));
    }

    #[test]
    fn sensitive_diagnostics_are_redacted_and_long_text_is_bounded() {
        assert_eq!(
            safe_diagnostic_text("telegram token=abc"),
            "redacted-sensitive-value"
        );
        assert_eq!(safe_diagnostic_text(&"x".repeat(300)).len(), 220);
    }

    #[test]
    fn child_summary_extracts_pids_and_exit_markers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("child.log");
        fs::write(
            &path,
            concat!(
                "[KGW_CHILD_STDOUT] {\"pid\":123}\n",
                "[KGW_START_TRACE] {\"pid\":456,\"stage\":\"self-worker-exited-during-startup\"}\n"
            ),
        )
        .unwrap();
        let value = child_process_summary(&path).unwrap();
        assert_eq!(value["line_count"], 2);
        assert_eq!(value["pids"].as_array().unwrap().len(), 2);
        assert_eq!(value["exit_markers"].as_array().unwrap().len(), 1);
    }
}
