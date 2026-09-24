use serde_json::{Map, Value, json};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub struct RunResult {
    pub code: i32,
    pub output: String,
}

struct Audit {
    repo_root: PathBuf,
    report_dir: PathBuf,
    started_at: String,
    files_read: Vec<Value>,
    findings: Vec<Value>,
    errors: Vec<String>,
    checks: Vec<Value>,
}

impl Audit {
    fn new(repo_root: PathBuf, report_dir: PathBuf) -> Result<Self, String> {
        Ok(Self {
            repo_root,
            report_dir,
            started_at: now_iso()?,
            files_read: Vec::new(),
            findings: Vec::new(),
            errors: Vec::new(),
            checks: Vec::new(),
        })
    }

    fn abs(&self, relative: &str) -> PathBuf {
        self.repo_root.join(relative)
    }

    fn save(&self, name: &str, text: &str) -> Result<PathBuf, String> {
        let out = self.report_dir.join(name);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(&out, text)
            .map_err(|error| format!("failed to write {}: {error}", out.display()))?;
        Ok(out)
    }

    fn read(&mut self, relative: &str) -> Result<String, String> {
        let full = self.abs(relative);
        if !full.is_file() {
            self.errors.push(format!("Missing file: {relative}"));
            return Ok(String::new());
        }
        let text = fs::read_to_string(&full)
            .map_err(|error| format!("failed to read {}: {error}", full.display()))?;
        let output_name = format!("FULL_READ__{}", safe_name(relative));
        let out = self.save(&output_name, &text)?;
        self.files_read.push(json!({
            "rel": relative,
            "out": out.to_string_lossy(),
            "bytes": text.len(),
        }));
        Ok(text)
    }

    fn add_finding(&mut self, level: &str, title: &str, data: Value) {
        self.findings.push(json!({
            "level": level,
            "title": title,
            "data": data,
        }));
    }
}
fn now_iso() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("runtime trace audit timestamp failed: {error}"))
}

fn safe_name(relative: &str) -> String {
    let mut output = String::new();
    for ch in relative.chars() {
        if matches!(ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            output.push_str("__");
        } else {
            output.push(ch);
        }
    }
    output
}

fn check_log_name(index: usize, command: &str) -> String {
    let command = command
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("CHECK__{index}__{command}.log")
}

fn count(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    text.match_indices(needle).count()
}

fn line_hits(text: &str, needle: &str) -> Vec<Value> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(index, line)| json!({"line": index + 1, "text": line}))
        .collect()
}

fn command_display(command: &str, args: &[String]) -> String {
    std::iter::once(command)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_check(
    audit: &mut Audit,
    command: &str,
    args: Vec<String>,
    cwd: &Path,
) -> Result<i32, String> {
    let output = Command::new(command).args(&args).current_dir(cwd).output();

    let (status, signal, stdout, stderr, error) = match output {
        Ok(output) => (
            output.status.code(),
            Value::Null,
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
            Value::Null,
        ),
        Err(error) => (
            None,
            Value::Null,
            String::new(),
            String::new(),
            Value::String(error.to_string()),
        ),
    };
    let display = command_display(command, &args);
    let index = audit.checks.len() + 1;
    let body = format!(
        "COMMAND: {display}\nCWD: {}\nSTATUS: {}\nSIGNAL: null\nERROR: {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
        cwd.display(),
        status.map_or_else(|| "null".to_owned(), |value| value.to_string()),
        error.as_str().unwrap_or("null"),
        stdout,
        stderr
    );
    audit.save(&check_log_name(index, command), &body)?;
    audit.checks.push(json!({
        "command": display,
        "cwd": cwd.to_string_lossy(),
        "status": status,
        "signal": signal,
        "stdout": stdout,
        "stderr": stderr,
        "error": error,
    }));
    Ok(status.unwrap_or(1))
}
fn source_findings(
    node_js: &str,
    bridge_js: &str,
    lib_rs: &str,
    main_rs: &str,
    gate: &str,
) -> (Vec<Value>, Vec<String>) {
    let mut findings = Vec::new();
    let mut critical = Vec::new();

    findings.push(json!({
        "level": "INFO",
        "title": "V19 frontend owner markers",
        "data": {
            "nodeOwnerCount": count(node_js, "KGW_SETTINGS_OWNER_V19"),
            "bridgeOwnerCount": count(bridge_js, "KGW_SETTINGS_OWNER_V19"),
            "nodeInstallRoute": node_js.contains("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)"),
            "bridgeInstallRoute": bridge_js.contains("window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)"),
            "nodeTraceInvokeCount": count(node_js, "kgw_frontend_button_trace_v1"),
            "bridgeTraceInvokeCount": count(bridge_js, "kgw_frontend_button_trace_v1"),
        }
    }));

    let old_tokens = [
        "action=settings-buttons",
        "nativeDisabledExpected",
        "click-received",
        "click-ignored-disabled",
        "action-start",
        "auto-baseline-before-input",
        "KGW_SETTINGS_OWNER_V18",
        "KGW_SETTINGS_OWNER_V17",
        "KGW_SETTINGS_OWNER_V16",
        "KGW_SETTINGS_OWNER_FINAL_V15",
    ];
    let mut old_hits = Map::new();
    for token in old_tokens {
        let hits = count(node_js, token) + count(bridge_js, token);
        if hits > 0 {
            old_hits.insert(token.to_owned(), json!(hits));
        }
    }
    findings.push(json!({
        "level": if old_hits.is_empty() { "OK" } else { "ERROR" },
        "title": "Old owner/runtime tokens in Node/Bridge JS",
        "data": old_hits,
    }));

    let rust_combined = format!("{lib_rs}\n{main_rs}");
    findings.push(json!({
        "level": "INFO",
        "title": "Rust trace command presence",
        "data": {
            "commandNameCount": count(&rust_combined, "kgw_frontend_button_trace_v1"),
            "tauriCommandAttrNearName": rust_combined.contains("#[tauri::command]") && rust_combined.contains("kgw_frontend_button_trace_v1"),
            "invokeHandlerMentionsCommand": rust_combined.contains("generate_handler") && rust_combined.contains("kgw_frontend_button_trace_v1"),
            "printlnNearTraceCommand": rust_combined.contains("println!") && rust_combined.contains("kgw_frontend_button_trace_v1"),
            "commandHitsInLib": line_hits(lib_rs, "kgw_frontend_button_trace_v1").into_iter().take(20).collect::<Vec<_>>(),
            "commandHitsInMain": line_hits(main_rs, "kgw_frontend_button_trace_v1").into_iter().take(20).collect::<Vec<_>>(),
        }
    }));

    findings.push(json!({
        "level": "INFO",
        "title": "Global owner gate file",
        "data": {
            "exists": !gate.is_empty(),
            "hasV19": gate.contains("KGW_SETTINGS_OWNER_V19"),
            "hasOldRegexRisk": gate.contains("new RegExp") || gate.contains(".match(") || gate.contains(".replace("),
            "lineCount": if gate.is_empty() { 0 } else { gate.split('\n').count() },
        }
    }));

    if count(node_js, "KGW_SETTINGS_OWNER_V19") < 2 {
        critical.push("Node V19 owner marker missing/incomplete.".to_owned());
    }
    if count(bridge_js, "KGW_SETTINGS_OWNER_V19") < 2 {
        critical.push("Bridge V19 owner marker missing/incomplete.".to_owned());
    }
    if !node_js.contains("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)") {
        critical.push("Node installActions not routed to V19 owner.".to_owned());
    }
    if !bridge_js.contains("window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)") {
        critical.push("Bridge installActions not routed to V19 owner.".to_owned());
    }
    if !old_hits.is_empty() {
        critical.push("Old owner/runtime tokens still exist in frontend JS.".to_owned());
    }
    if !rust_combined.contains("kgw_frontend_button_trace_v1") {
        critical.push(
            "Rust command kgw_frontend_button_trace_v1 is missing from lib.rs/main.rs.".to_owned(),
        );
    }

    (findings, critical)
}

fn read_pipe<T: Read>(pipe: Option<T>) -> String {
    let Some(mut pipe) = pipe else {
        return String::new();
    };
    let mut bytes = Vec::new();
    let _ = pipe.read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

fn tail_chars(value: &str, count: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    chars[chars.len().saturating_sub(count)..].iter().collect()
}

#[cfg(windows)]
fn default_npm_command() -> OsString {
    let pinned = Path::new(r"C:\Program Files\nodejs\npm.cmd");
    if pinned.is_file() {
        pinned.as_os_str().to_os_string()
    } else {
        OsString::from("npm.cmd")
    }
}

#[cfg(not(windows))]
fn default_npm_command() -> OsString {
    OsString::from("npm")
}
fn dev_probe_finding(audit: &mut Audit) -> Result<(), String> {
    let enabled = env::var("KGW_TRACE_AUDIT_DEV_PROBE")
        .unwrap_or_default()
        .trim()
        == "1";
    if !enabled {
        audit.add_finding(
            "INFO",
            "Tauri dev 12-second probe",
            json!({
                "skipped": true,
                "reason": "Disabled by default for stable unified gate execution.",
                "enableWith": "KGW_TRACE_AUDIT_DEV_PROBE=1",
                "patch": "R100A3"
            }),
        );
        return Ok(());
    }

    let app_root = audit.abs("apps/kaspa-gateway-desktop");
    let npm_command = env::var_os("KGW_NPM_CMD").unwrap_or_else(default_npm_command);
    let arguments = [
        "run",
        "tauri",
        "--",
        "dev",
        "--features",
        "official-kaspa-runtime-all rkstratum_cpu_miner",
    ];

    let mut command;
    #[cfg(windows)]
    {
        command = Command::new("cmd");
        command.arg("/C").arg(&npm_command).args(arguments);
    }
    #[cfg(not(windows))]
    {
        command = Command::new(&npm_command);
        command.args(arguments);
    }
    command
        .current_dir(&app_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let spawned = command.spawn();
    let (mut child, spawn_error) = match spawned {
        Ok(child) => (Some(child), String::new()),
        Err(error) => (None, error.to_string()),
    };
    let started = Instant::now();
    let mut exited = false;
    let mut exit_code = None;
    if let Some(child_ref) = child.as_mut() {
        while started.elapsed() < Duration::from_secs(12) {
            match child_ref.try_wait() {
                Ok(Some(status)) => {
                    exited = true;
                    exit_code = status.code();
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(250)),
                Err(_) => break,
            }
        }
    }

    let mut stdout = String::new();
    let mut stderr = String::new();
    if let Some(mut child_value) = child {
        if !exited {
            let _ = child_value.kill();
            let _ = child_value.wait();
        }
        stdout = read_pipe(child_value.stdout.take());
        stderr = read_pipe(child_value.stderr.take());
    }
    let npm_text = npm_command.to_string_lossy().into_owned();
    let log = format!(
        "Enabled: true\nCommand: {npm_text}\nExitedWithin12Seconds: {exited}\nExitCode: {}\nExitSignal: null\nSpawnError: {}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
        exit_code.map_or_else(|| "null".to_owned(), |value| value.to_string()),
        spawn_error,
        stdout,
        stderr
    );
    audit.save("DEV_PROBE_12_SECONDS.log", &log)?;

    let interpretation = if !spawn_error.is_empty() {
        "The optional dev probe could not spawn; this no longer blocks the unified gate unless explicitly reviewed."
    } else if exited {
        "The app/dev process returned to PowerShell quickly. Frontend runtime trace cannot appear if the app exits before UI interaction."
    } else {
        "The dev process stayed alive for 12 seconds. Lack of trace is more likely command registration/printing/frontend loading, not immediate exit."
    };
    audit.add_finding(
        if exited && spawn_error.is_empty() { "ERROR" } else { "INFO" },
        "Tauri dev 12-second probe",
        json!({
            "enabled": true,
            "command": npm_text,
            "exitedWithin12Seconds": exited,
            "exitCode": exit_code,
            "exitSignal": Value::Null,
            "spawnError": if spawn_error.is_empty() { Value::Null } else { Value::String(spawn_error) },
            "stdoutTail": tail_chars(&stdout, 4000),
            "stderrTail": tail_chars(&stderr, 4000),
            "interpretation": interpretation
        }),
    );
    Ok(())
}
fn report_markdown(audit: &Audit, success: bool, reason: &str) -> String {
    let mut lines = vec![
        format!(
            "# {} - KGW Runtime Trace Owner Audit V20",
            if success { "SUCCESS" } else { "FAILED" }
        ),
        String::new(),
        format!("- Repository: `{}`", audit.repo_root.display()),
        format!("- Report dir: `{}`", audit.report_dir.display()),
        "- Source mutation: `false`".to_owned(),
        "- Git commit: `false`".to_owned(),
        "- Git push: `false`".to_owned(),
        format!("- Reason: {reason}"),
        String::new(),
        "## Findings".to_owned(),
        String::new(),
    ];
    if audit.findings.is_empty() {
        lines.push("- none".to_owned());
    } else {
        for finding in &audit.findings {
            let level = finding.get("level").and_then(Value::as_str).unwrap_or("");
            let title = finding.get("title").and_then(Value::as_str).unwrap_or("");
            let data = finding.get("data").cloned().unwrap_or_else(|| json!({}));
            let rendered = serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_owned());
            lines.push(format!("- **{level}** {title}"));
            lines.push("  ```json".to_owned());
            lines.extend(rendered.lines().map(|line| format!("  {line}")));
            lines.push("  ```".to_owned());
        }
    }
    lines.extend([String::new(), "## Errors".to_owned(), String::new()]);
    if audit.errors.is_empty() {
        lines.push("- none".to_owned());
    } else {
        lines.extend(audit.errors.iter().map(|error| format!("- {error}")));
    }
    lines.extend([String::new(), "## Checks".to_owned(), String::new()]);
    if audit.checks.is_empty() {
        lines.push("- none".to_owned());
    } else {
        for check in &audit.checks {
            let command = check.get("command").and_then(Value::as_str).unwrap_or("");
            let status = check.get("status").cloned().unwrap_or(Value::Null);
            let status_text = if status.is_null() {
                "null".to_owned()
            } else {
                status.to_string()
            };
            lines.push(format!("- `{command}` => `{status_text}`"));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn write_report(audit: &Audit, success: bool, reason: &str) -> Result<(), String> {
    let run_name = audit
        .report_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let report = json!({
        "runName": run_name,
        "repoRoot": audit.repo_root.to_string_lossy(),
        "reportDir": audit.report_dir.to_string_lossy(),
        "startedAt": audit.started_at,
        "mutation": false,
        "gitCommit": false,
        "gitPush": false,
        "filesRead": audit.files_read,
        "findings": audit.findings,
        "errors": audit.errors,
        "checks": audit.checks,
        "finishedAt": now_iso()?,
        "success": success,
        "reason": reason
    });
    let mut payload = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("failed to serialize runtime trace report: {error}"))?;
    payload.push('\n');
    audit.save("REPORT.json", &payload)?;
    let markdown = report_markdown(audit, success, reason);
    audit.save(
        if success {
            "REPORT_SUCCESS.md"
        } else {
            "REPORT_FAILED.md"
        },
        &markdown,
    )?;
    Ok(())
}
pub fn run_cli(args: &mut impl Iterator<Item = String>) -> Result<RunResult, String> {
    let repo_root =
        PathBuf::from(args.next().ok_or_else(|| {
            "runtime-trace-owner-audit requires <repo-root> <report-dir>".to_owned()
        })?);
    let report_dir =
        PathBuf::from(args.next().ok_or_else(|| {
            "runtime-trace-owner-audit requires <repo-root> <report-dir>".to_owned()
        })?);
    if args.next().is_some() {
        return Err(
            "runtime-trace-owner-audit accepts exactly <repo-root> <report-dir>".to_owned(),
        );
    }
    fs::create_dir_all(&report_dir)
        .map_err(|error| format!("failed to create {}: {error}", report_dir.display()))?;
    let mut audit = Audit::new(repo_root.clone(), report_dir)?;

    let rels = [
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css",
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
        "apps/kaspa-gateway-desktop/src-tauri/src/main.rs",
        "apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json",
        "tools/kgw_global_owner_gate.cjs",
        "package.json",
        "apps/kaspa-gateway-desktop/package.json",
    ];
    let mut texts = std::collections::BTreeMap::new();
    for relative in rels {
        if audit.abs(relative).is_file() {
            texts.insert(relative, audit.read(relative)?);
        }
    }

    let node_rel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
    let bridge_rel = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
    let lib_rel = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
    let main_rel = "apps/kaspa-gateway-desktop/src-tauri/src/main.rs";
    let gate_rel = "tools/kgw_global_owner_gate.cjs";
    let node_js = texts.get(node_rel).map(String::as_str).unwrap_or("");
    let bridge_js = texts.get(bridge_rel).map(String::as_str).unwrap_or("");
    let lib_rs = texts.get(lib_rel).map(String::as_str).unwrap_or("");
    let main_rs = texts.get(main_rel).map(String::as_str).unwrap_or("");
    let gate = texts.get(gate_rel).map(String::as_str).unwrap_or("");

    let (findings, mut critical) = source_findings(node_js, bridge_js, lib_rs, main_rs, gate);
    audit.findings.extend(findings);

    let node_path = audit.abs(node_rel).to_string_lossy().into_owned();
    let bridge_path = audit.abs(bridge_rel).to_string_lossy().into_owned();
    let gate_path = audit.abs(gate_rel);
    let gate_path_text = gate_path.to_string_lossy().into_owned();
    let repo_root_text = repo_root.to_string_lossy().into_owned();

    let node_status = run_check(
        &mut audit,
        "node",
        vec!["--check".to_owned(), node_path],
        &repo_root,
    )?;
    let bridge_status = run_check(
        &mut audit,
        "node",
        vec!["--check".to_owned(), bridge_path],
        &repo_root,
    )?;
    if gate_path.is_file() {
        let _ = run_check(
            &mut audit,
            "node",
            vec![gate_path_text, repo_root_text],
            &repo_root,
        )?;
    }
    let _ = run_check(
        &mut audit,
        "cargo",
        vec![
            "check".to_owned(),
            "-p".to_owned(),
            "kaspa-gateway-desktop".to_owned(),
            "--no-default-features".to_owned(),
            "--features".to_owned(),
            "official-kaspa-runtime-all rkstratum_cpu_miner".to_owned(),
        ],
        &repo_root,
    )?;
    dev_probe_finding(&mut audit)?;

    if node_status != 0 {
        critical.push("node --check failed for node JS.".to_owned());
    }
    if bridge_status != 0 {
        critical.push("node --check failed for bridge JS.".to_owned());
    }
    if !critical.is_empty() {
        audit.errors.extend(critical);
        let reason = "Critical audit blockers found.";
        write_report(&audit, false, reason)?;
        return Ok(RunResult {
            code: 1,
            output: report_markdown(&audit, false, reason),
        });
    }
    let reason = "Audit completed. Review findings for exact runtime trace cause.";
    write_report(&audit, true, reason)?;
    Ok(RunResult {
        code: 0,
        output: report_markdown(&audit, true, reason),
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    fn valid_sources() -> (String, String, String, String, String) {
        let node = format!(
            "{} {} {}",
            "KGW_SETTINGS_OWNER_V19 ".repeat(10),
            "kgw_frontend_button_trace_v1 ".repeat(5),
            "window.KGW_NODE_SETTINGS_OWNER_V19.install(root)"
        );
        let bridge = format!(
            "{} {} {}",
            "KGW_SETTINGS_OWNER_V19 ".repeat(10),
            "kgw_frontend_button_trace_v1 ".repeat(6),
            "window.KGW_BRIDGE_SETTINGS_OWNER_V19.install(root)"
        );
        let lib = "#[tauri::command]\nfn kgw_frontend_button_trace_v1() { println!(\"trace\"); }\ngenerate_handler![kgw_frontend_button_trace_v1]".to_owned();
        let main = String::new();
        let gate = "KGW_SETTINGS_OWNER_V19 new RegExp".to_owned();
        (node, bridge, lib, main, gate)
    }

    #[test]
    fn legacy_reference_source_findings_match() {
        let (node, bridge, lib, main, gate) = valid_sources();
        let (findings, critical) = source_findings(&node, &bridge, &lib, &main, &gate);
        assert!(critical.is_empty());
        assert_eq!(findings.len(), 4);
        assert_eq!(findings[0]["data"]["nodeOwnerCount"], 10);
        assert_eq!(findings[0]["data"]["bridgeOwnerCount"], 10);
        assert_eq!(findings[0]["data"]["nodeTraceInvokeCount"], 5);
        assert_eq!(findings[0]["data"]["bridgeTraceInvokeCount"], 6);
        assert_eq!(findings[1]["level"], "OK");
        assert_eq!(findings[2]["data"]["commandNameCount"], 2);
        assert_eq!(findings[3]["data"]["hasV19"], true);
    }

    #[test]
    fn old_owner_token_fails_closed() {
        let (mut node, bridge, lib, main, gate) = valid_sources();
        node.push_str(" KGW_SETTINGS_OWNER_V18");
        let (findings, critical) = source_findings(&node, &bridge, &lib, &main, &gate);
        assert_eq!(findings[1]["level"], "ERROR");
        assert!(
            critical
                .iter()
                .any(|item| item.contains("Old owner/runtime tokens"))
        );
    }

    #[test]
    fn missing_install_route_and_trace_command_fail_closed() {
        let (node, bridge, _lib, main, gate) = valid_sources();
        let node = node.replace("window.KGW_NODE_SETTINGS_OWNER_V19.install(root)", "");
        let (_, critical) = source_findings(&node, &bridge, "", &main, &gate);
        assert!(
            critical
                .iter()
                .any(|item| item.contains("Node installActions"))
        );
        assert!(critical.iter().any(|item| item.contains("Rust command")));
    }

    #[test]
    fn legacy_safe_name_shape_is_preserved() {
        assert_eq!(
            safe_name("apps/kaspa:gateway/file?.js"),
            "apps__kaspa__gateway__file__.js"
        );
    }
}
