use serde_json::{Value, json};
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use sysinfo::System;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const E2E_CONFIG: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.e2e.conf.json";
const E2E_LOCK: &str = "e2e/package-lock.json";
const E2E_DIR: &str = "e2e";
const GENERATED_SCHEMAS: &str = "apps/kaspa-gateway-desktop/src-tauri/gen/schemas";
const E2E_JS_SYNTAX_CHECKS: [(&str, &str); 7] = [
    ("wdio.conf.mjs", "node-check-wdio-conf.log"),
    (
        "specs/zero-touch-live-matrix.e2e.js",
        "node-check-zero-touch-matrix.log",
    ),
    ("helpers/assertions.mjs", "node-check-assertions.log"),
    ("helpers/paths.mjs", "node-check-paths.log"),
    ("helpers/tauri-app.mjs", "node-check-tauri-app.log"),
    ("helpers/windows.mjs", "node-check-windows.log"),
    ("helpers/runtime-ports.mjs", "node-check-runtime-ports.log"),
];

#[derive(Debug, Clone)]
struct OwnedProcess {
    pid: u32,
    parent: u32,
    name: String,
    executable: String,
    command_line: String,
    start_time: u64,
}

#[derive(Debug, Clone)]
struct CommandReceipt {
    label: String,
    status: String,
    exit_code: i32,
    log: String,
}

struct StageContext<'a> {
    artifact_root: &'a Path,
    envs: &'a [(OsString, OsString)],
    receipts: Vec<CommandReceipt>,
}

impl<'a> StageContext<'a> {
    fn new(artifact_root: &'a Path, envs: &'a [(OsString, OsString)]) -> Self {
        Self {
            artifact_root,
            envs,
            receipts: Vec::new(),
        }
    }

    fn skipped(&mut self, label: &str) {
        self.receipts.push(CommandReceipt {
            label: label.to_owned(),
            status: "skipped-existing".to_owned(),
            exit_code: 0,
            log: String::new(),
        });
    }

    fn run(
        &mut self,
        stage: &str,
        program: &str,
        args: &[&str],
        cwd: &Path,
        log_name: &str,
        receipt_label: &str,
    ) -> Result<(), String> {
        self.run_with_extra_env(stage, program, args, cwd, (log_name, receipt_label), &[])
    }

    fn run_with_extra_env(
        &mut self,
        stage: &str,
        program: &str,
        args: &[&str],
        cwd: &Path,
        receipt: (&str, &str),
        extra_envs: &[(OsString, OsString)],
    ) -> Result<(), String> {
        let (log_name, receipt_label) = receipt;
        let log = self.artifact_root.join(log_name);
        let mut envs = self.envs.to_vec();
        envs.extend(extra_envs.iter().cloned());
        let code = run_logged(program, args, cwd, &log, &envs)?;
        self.receipts.push(CommandReceipt {
            label: receipt_label.to_owned(),
            status: if code == 0 { "passed" } else { "failed" }.to_owned(),
            exit_code: code,
            log: log_name.to_owned(),
        });
        if code == 0 {
            Ok(())
        } else {
            Err(format!("{stage} failed with exit code {code}"))
        }
    }

    fn run_wdio_fail_closed(
        &mut self,
        stage: &str,
        program: &str,
        args: &[&str],
        cwd: &Path,
        receipt: (&str, &str),
        extra_envs: &[(OsString, OsString)],
    ) -> Result<(), String> {
        let (log_name, receipt_label) = receipt;
        let log = self.artifact_root.join(log_name);
        let mut envs = self.envs.to_vec();
        envs.extend(extra_envs.iter().cloned());
        let code = run_logged(program, args, cwd, &log, &envs)?;
        let accepted_teardown_race =
            code == 1 && wdio_post_success_teardown_race(self.artifact_root, &log)?;
        self.receipts.push(CommandReceipt {
            label: receipt_label.to_owned(),
            status: if code == 0 {
                "passed".to_owned()
            } else if accepted_teardown_race {
                "passed-post-success-teardown-race".to_owned()
            } else {
                "failed".to_owned()
            },
            exit_code: code,
            log: log_name.to_owned(),
        });
        if code == 0 || accepted_teardown_race {
            Ok(())
        } else {
            Err(format!("{stage} failed with exit code {code}"))
        }
    }
}

fn wdio_post_success_teardown_race(artifact_root: &Path, log_path: &Path) -> Result<bool, String> {
    let log =
        fs::read_to_string(log_path).map_err(|e| format!("read {}: {e}", log_path.display()))?;
    let exact_signature = log.contains("Error [ERR_STREAM_WRITE_AFTER_END]: write after end")
        && log.contains("@wdio/native-core/dist/esm/index.js")
        && log.contains("LogWriter.write")
        && log.contains("@wdio/tauri-service/dist/esm/index.js")
        && log.contains("forwardLog");
    if !exact_signature {
        return Ok(false);
    }

    let report_path = artifact_root.join("json").join("wdio-0-0.json");
    let report_text = fs::read_to_string(&report_path)
        .map_err(|e| format!("read {}: {e}", report_path.display()))?;
    let report: Value = serde_json::from_str(&report_text)
        .map_err(|e| format!("parse {}: {e}", report_path.display()))?;
    Ok(wdio_report_is_exact_full_pass(&report))
}

fn wdio_report_is_exact_full_pass(report: &Value) -> bool {
    let state = match report.get("state") {
        Some(value) => value,
        None => return false,
    };
    if state.get("passed").and_then(Value::as_u64) != Some(5)
        || state.get("failed").and_then(Value::as_u64) != Some(0)
        || state.get("skipped").and_then(Value::as_u64) != Some(0)
    {
        return false;
    }

    let suites = match report.get("suites").and_then(Value::as_array) {
        Some(value) if value.len() == 1 => value,
        _ => return false,
    };
    let tests = match suites[0].get("tests").and_then(Value::as_array) {
        Some(value) if value.len() == 5 => value,
        _ => return false,
    };
    let hooks = match suites[0].get("hooks").and_then(Value::as_array) {
        Some(value) => value,
        None => return false,
    };

    tests
        .iter()
        .chain(hooks.iter())
        .all(|entry| entry.get("state").and_then(Value::as_str) == Some("passed"))
}

struct NativeResultInput<'a> {
    started_at: &'a str,
    exit_code: i32,
    failed_stage: Option<&'a str>,
    executable: &'a Path,
    evidence: &'a Value,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>, root: &Path) -> Result<String, String> {
    if args.next().is_some() {
        return Err(
            "zero-touch-e2e takes no arguments; repository is the current checkout".to_owned(),
        );
    }
    run(root)
}

fn path_dirty(root: &Path, relative: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain", "--", relative])
        .output()
        .map_err(|e| format!("git status {relative}: {e}"))?;
    if !output.status.success() {
        return Err(format!("git status failed for {relative}"));
    }
    Ok(!output.stdout.is_empty())
}

fn restore_generated_schemas_if_newly_dirty(
    root: &Path,
    initially_dirty: bool,
) -> Result<(), String> {
    if initially_dirty || !path_dirty(root, GENERATED_SCHEMAS)? {
        return Ok(());
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["restore", "--worktree", "--", GENERATED_SCHEMAS])
        .output()
        .map_err(|e| format!("restore generated Tauri schemas: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "restore generated Tauri schemas failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    if path_dirty(root, GENERATED_SCHEMAS)? {
        return Err("generated Tauri schemas remained dirty after restore".to_owned());
    }
    Ok(())
}

pub(crate) fn run(root: &Path) -> Result<String, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("repository path: {e}"))?;
    let config_path = root.join(E2E_CONFIG);
    let lock_path = root.join(E2E_LOCK);
    if !config_path.is_file() {
        return Err(format!(
            "Missing E2E Tauri config: {}",
            config_path.display()
        ));
    }
    if !lock_path.is_file() {
        return Err(format!(
            "Missing locked E2E dependencies: {}",
            lock_path.display()
        ));
    }

    let schemas_dirty = path_dirty(&root, GENERATED_SCHEMAS)?;
    let pwsh = required_pwsh()?;
    let started_at = now()?;
    let run_id = std::env::var("KGW_ZERO_TOUCH_RUN_ID").unwrap_or_else(|_| {
        format!(
            "{}-{}",
            OffsetDateTime::now_utc().unix_timestamp(),
            std::process::id()
        )
    });
    let artifact_root = std::env::var_os("KGW_ZERO_TOUCH_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("artifacts").join("zero-touch-e2e").join(&run_id));
    let artifact_root = absolute(&artifact_root)?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target").join("kgw-zero-touch-e2e"));
    let target_dir = absolute(&target_dir)?;
    let app_binary = target_dir.join("debug").join("kaspa-gateway-desktop.exe");
    let e2e_dir = root.join(E2E_DIR);

    for dir in [
        artifact_root.clone(),
        artifact_root.join("localappdata"),
        artifact_root.join("appdata"),
        artifact_root.join("backend-traces"),
    ] {
        fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }

    let tauri_config = fs::read_to_string(&config_path)
        .map_err(|e| format!("read {}: {e}", config_path.display()))?;
    let envs = runtime_env(
        &root,
        &artifact_root,
        &run_id,
        &app_binary,
        &target_dir,
        &pwsh,
        &tauri_config,
    );

    let mut summary = json!({
        "repository": root.to_string_lossy(),
        "run_id": run_id,
        "artifact_root": artifact_root.to_string_lossy(),
        "cargo_target_dir": target_dir.to_string_lossy(),
        "app_binary": app_binary.to_string_lossy(),
        "commands": [],
    });
    write_json(
        &artifact_root.join("zero-touch-script-start.json"),
        &summary,
    )?;

    save_snapshot(&root, &artifact_root, "pre-run")?;
    stop_owned_tree(&root, &artifact_root, "pre-run-cleanup")?;

    let nested_cargo_target = target_dir.join("nested-xtask");
    let current_xtask_program = std::env::current_exe()
        .map_err(|error| format!("resolve current xtask executable: {error}"))?
        .to_string_lossy()
        .into_owned();
    let nested_cargo_env = [
        (
            OsString::from("CARGO_TARGET_DIR"),
            nested_cargo_target.as_os_str().to_os_string(),
        ),
        (
            OsString::from("KGW_XTASK_EXE"),
            OsString::from(&current_xtask_program),
        ),
    ];
    let mut stages = StageContext::new(&artifact_root, &envs);
    let mut exit_code = 0_i32;
    let mut failed_stage: Option<String> = None;
    let mut primary_error: Option<String> = None;

    let result = (|| -> Result<Value, String> {
        let wdio_cli = e2e_dir.join("node_modules/@wdio/cli/package.json");
        if wdio_cli.is_file() {
            stages.skipped("npm ci");
        } else {
            stages.run(
                "Install locked E2E dependencies",
                if cfg!(windows) { "npm.cmd" } else { "npm" },
                &["ci"],
                &e2e_dir,
                "npm-ci.log",
                "npm ci",
            )?;
        }
        for (relative, log_name) in E2E_JS_SYNTAX_CHECKS {
            let receipt_label = format!("node --check {relative}");
            stages.run(
                "E2E JavaScript syntax checks",
                "node",
                &["--check", relative],
                &e2e_dir,
                log_name,
                &receipt_label,
            )?;
        }
        stages.run(
            "Deepmerge security smoke",
            &current_xtask_program,
            &["deepmerge-security-smoke"],
            &root,
            "deepmerge-security-smoke.log",
            "xtask deepmerge-security-smoke",
        )?;
        stages.run(
            "E2E static smokes",
            &current_xtask_program,
            &["e2e-static-smokes"],
            &root,
            "e2e-static-smokes.log",
            "xtask e2e-static-smokes",
        )?;
        stages.run(
            "Build desktop E2E binary once",
            "cargo",
            &[
                "build",
                "--locked",
                "-p",
                "kaspa-gateway-desktop",
                "--bin",
                "kaspa-gateway-desktop",
                "--features",
                "e2e-test",
            ],
            &root,
            "cargo-build-e2e.log",
            "cargo build --locked -p kaspa-gateway-desktop --bin kaspa-gateway-desktop --features e2e-test",
        )?;
        if !app_binary.is_file() {
            return Err(format!(
                "Build desktop E2E binary once: binary not created at {}",
                app_binary.display()
            ));
        }
        stages.run_wdio_fail_closed(
            "WebdriverIO zero-touch live matrix",
            if cfg!(windows) { "npm.cmd" } else { "npm" },
            &["run", "e2e"],
            &e2e_dir,
            ("wdio-run.log", "npm run e2e"),
            &nested_cargo_env,
        )?;
        evidence_summary(&root, &artifact_root, &envs)
    })();

    let evidence = match result {
        Ok(value) => {
            if value.get("passed").and_then(Value::as_bool) != Some(true) {
                exit_code = 1;
                failed_stage = value
                    .get("failed_stage")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or_else(|| Some("Zero-touch evidence validation".to_owned()));
                primary_error = Some(format!(
                    "Zero-touch evidence validation failed: {}",
                    validation_errors(&value).join(" | ")
                ));
            }
            Some(value)
        }
        Err(error) => {
            exit_code = 1;
            failed_stage = Some(stage_from_error(&error));
            primary_error = Some(error);
            None
        }
    };

    let cleanup = stop_owned_tree(&root, &artifact_root, "finally-cleanup")
        .and_then(|_| save_snapshot(&root, &artifact_root, "post-cleanup").map(|_| ()));
    if let Err(error) = cleanup {
        if exit_code == 0 {
            exit_code = 1;
            failed_stage = Some("Final process cleanup".to_owned());
        }
        summary["cleanup_error"] = Value::String(error);
    }

    let final_evidence = match evidence {
        Some(value) => value,
        None => match evidence_summary(&root, &artifact_root, &envs) {
            Ok(value) => value,
            Err(error) => json!({"passed": false, "validation_errors": [error]}),
        },
    };
    if exit_code == 0 && final_evidence.get("passed").and_then(Value::as_bool) != Some(true) {
        exit_code = 1;
        failed_stage = final_evidence
            .get("failed_stage")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| Some("Zero-touch evidence validation".to_owned()));
    }

    restore_generated_schemas_if_newly_dirty(&root, schemas_dirty)?;

    let result_input = NativeResultInput {
        started_at: &started_at,
        exit_code,
        failed_stage: failed_stage.as_deref(),
        executable: &app_binary,
        evidence: &final_evidence,
    };
    let result_receipt = write_native_result(&root, &artifact_root, &result_input, &envs)?;

    summary["commands"] = Value::Array(stages.receipts.iter().map(receipt_json).collect());
    summary["evidence_validation"] = final_evidence;
    summary["result_file"] = Value::String(
        artifact_root
            .join("zero-touch-result.json")
            .to_string_lossy()
            .into_owned(),
    );
    summary["passed_stages"] = result_receipt
        .get("passed_stages")
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    summary["failed_stage"] = result_receipt
        .get("failed_stage")
        .cloned()
        .unwrap_or(Value::Null);
    if let Some(error) = primary_error {
        summary["error"] = Value::String(error);
    }
    summary["finished_at"] = Value::String(now()?);
    summary["exit_code"] = json!(exit_code);
    write_json(
        &artifact_root.join("zero-touch-script-summary.json"),
        &summary,
    )?;
    write_report(
        &artifact_root,
        &run_id,
        exit_code,
        &app_binary,
        &target_dir,
        &summary,
    )?;

    if exit_code == 0 {
        Ok(format!(
            "KGW zero-touch E2E PASSED\nARTIFACT_ROOT={}",
            artifact_root.display()
        ))
    } else {
        Err(format!(
            "KGW zero-touch E2E FAILED\nARTIFACT_ROOT={}\nFAILED_STAGE={}",
            artifact_root.display(),
            failed_stage.unwrap_or_else(|| "Zero-touch script".to_owned())
        ))
    }
}

fn run_logged(
    program: &str,
    args: &[&str],
    cwd: &Path,
    log: &Path,
    envs: &[(OsString, OsString)],
) -> Result<i32, String> {
    if let Some(parent) = log.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().cloned())
        .output()
        .map_err(|e| format!("failed to launch {program}: {e}"))?;
    let mut bytes = output.stdout;
    bytes.extend_from_slice(&output.stderr);
    fs::write(log, &bytes).map_err(|e| format!("write {}: {e}", log.display()))?;
    if !bytes.is_empty() {
        print!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(output.status.code().unwrap_or(1))
}

fn evidence_summary(
    root: &Path,
    artifact: &Path,
    envs: &[(OsString, OsString)],
) -> Result<Value, String> {
    let manifest = root.join("Cargo.toml").to_string_lossy().into_owned();
    let artifact = artifact.to_string_lossy().into_owned();
    let output = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            &manifest,
            "--locked",
            "-p",
            "xtask",
            "--bin",
            "kgw-zero-touch-evidence",
            "--",
            "summary",
            "--artifact-directory",
            &artifact,
        ])
        .current_dir(root)
        .envs(envs.iter().cloned())
        .output()
        .map_err(|e| format!("Native evidence summary could not start: {e}"))?;
    let code = output.status.code().unwrap_or(2);
    if !matches!(code, 0 | 1) {
        return Err(format!(
            "Native evidence summary could not complete; exit code {code}."
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Native evidence summary JSON: {e}"))?;
    let passed = value
        .get("passed")
        .and_then(Value::as_bool)
        .ok_or_else(|| "Native evidence summary missing boolean passed".to_owned())?;
    if (code == 0) != passed {
        return Err(
            "Native evidence summary returned an inconsistent pass/exit result.".to_owned(),
        );
    }
    Ok(value)
}

fn write_native_result(
    root: &Path,
    artifact: &Path,
    input: &NativeResultInput<'_>,
    envs: &[(OsString, OsString)],
) -> Result<Value, String> {
    let request = json!({
        "repository": root,
        "artifact_directory": artifact,
        "started_at": input.started_at,
        "exit_code": input.exit_code,
        "failed_stage": input.failed_stage,
        "executable_path": input.executable,
        "evidence_summary": input.evidence,
    });
    let manifest = root.join("Cargo.toml").to_string_lossy().into_owned();
    let mut child = Command::new("cargo")
        .args([
            "run",
            "--manifest-path",
            &manifest,
            "--locked",
            "-p",
            "xtask",
            "--bin",
            "kgw-zero-touch-result",
            "--",
            "build-write",
            "--request",
            "-",
        ])
        .current_dir(root)
        .envs(envs.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Native Rust result construction/writing could not start: {e}"))?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "result writer stdin unavailable".to_owned())?
        .write_all(
            serde_json::to_string(&request)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| format!("result writer stdin: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("result writer wait: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "Native Rust result construction/writing failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Native Rust result receipt JSON: {e}"))?;
    if value.get("completed").and_then(Value::as_bool) != Some(true) {
        return Err(
            "Native Rust result writer did not return a completed result receipt.".to_owned(),
        );
    }
    Ok(value)
}

fn runtime_env(
    root: &Path,
    artifact: &Path,
    run_id: &str,
    app: &Path,
    target: &Path,
    pwsh: &Path,
    tauri_config: &str,
) -> Vec<(OsString, OsString)> {
    [
        ("KGW_REPOSITORY", root.as_os_str()),
        ("KGW_ZERO_TOUCH_ARTIFACT_DIR", artifact.as_os_str()),
        ("KGW_E2E_RUN_ID", OsString::from(run_id).as_os_str()),
        ("KGW_E2E_APP_BINARY", app.as_os_str()),
        ("KGW_UI_TRACE_FILE", OsString::from("1").as_os_str()),
        (
            "KGW_UI_TRACE_DIR",
            artifact.join("backend-traces").as_os_str(),
        ),
        ("KGW_START_TRACE", OsString::from("1").as_os_str()),
        ("KGW_REQUIRED_PWSH_PATH", pwsh.as_os_str()),
        ("TAURI_CONFIG", OsString::from(tauri_config).as_os_str()),
        ("LOCALAPPDATA", artifact.join("localappdata").as_os_str()),
        ("APPDATA", artifact.join("appdata").as_os_str()),
        ("CARGO_TARGET_DIR", target.as_os_str()),
    ]
    .into_iter()
    .map(|(k, v)| (OsString::from(k), v.to_os_string()))
    .collect()
}

fn required_pwsh() -> Result<PathBuf, String> {
    let version = Command::new("pwsh")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-Command",
            "$PSVersionTable.PSVersion.Major",
        ])
        .output()
        .map_err(|e| format!("PowerShell 7 is required: {e}"))?;
    if !version.status.success()
        || String::from_utf8_lossy(&version.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(0)
            < 7
    {
        return Err("Kaspa Gateway zero-touch E2E requires PowerShell 7 or later.".to_owned());
    }
    let found = Command::new("where.exe")
        .arg("pwsh.exe")
        .output()
        .map_err(|e| format!("locate pwsh.exe: {e}"))?;
    String::from_utf8_lossy(&found.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "PowerShell 7 executable path could not be resolved.".to_owned())
}

fn owned_processes(repository: &Path, artifact: &Path) -> Vec<OwnedProcess> {
    let system = System::new_all();
    let repo = repository.to_string_lossy().to_ascii_lowercase();
    let artifacts = artifact.to_string_lossy().to_ascii_lowercase();
    let current = sysinfo::Pid::from_u32(std::process::id());
    let mut protected = HashSet::new();
    let mut cursor = Some(current);
    while let Some(pid) = cursor {
        if !protected.insert(pid.as_u32()) {
            break;
        }
        cursor = system.process(pid).and_then(|p| p.parent());
    }
    let mut all = HashMap::<u32, OwnedProcess>::new();
    let mut children = HashMap::<u32, Vec<u32>>::new();
    let mut roots = HashSet::<u32>::new();
    for (pid, process) in system.processes() {
        let id = pid.as_u32();
        let parent = process.parent().map(sysinfo::Pid::as_u32).unwrap_or(0);
        let executable = process
            .exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let command_line = process
            .cmd()
            .iter()
            .map(|v| v.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        let name = process.name().to_string_lossy().into_owned();
        if parent != 0 {
            children.entry(parent).or_default().push(id);
        }
        let exe_lower = executable.to_ascii_lowercase();
        let cmd_lower = command_line.to_ascii_lowercase();
        let name_lower = name.to_ascii_lowercase();
        let in_repo = cmd_lower.contains(&repo);
        let in_artifact = cmd_lower.contains(&artifacts);
        let path_in_repo = exe_lower.starts_with(&repo);
        if (path_in_repo
            && (name_lower == "kaspa-gateway-desktop.exe"
                || cmd_lower.contains("--kgw-self-worker")))
            || (in_repo && cmd_lower.contains("--kgw-self-worker"))
            || ((in_repo || in_artifact)
                && matches!(
                    name_lower.as_str(),
                    "node.exe"
                        | "npm.exe"
                        | "npm.cmd"
                        | "wdio.exe"
                        | "wdio.cmd"
                        | "webdriverio.exe"
                        | "webdriverio.cmd"
                ))
        {
            roots.insert(id);
        }
        all.insert(
            id,
            OwnedProcess {
                pid: id,
                parent,
                name,
                executable,
                command_line,
                start_time: process.start_time(),
            },
        );
    }
    let mut wanted = HashSet::new();
    let mut queue: VecDeque<u32> = roots.into_iter().collect();
    while let Some(pid) = queue.pop_front() {
        if !wanted.insert(pid) {
            continue;
        }
        if let Some(ids) = children.get(&pid) {
            queue.extend(ids.iter().copied());
        }
    }
    let mut out: Vec<_> = wanted
        .into_iter()
        .filter(|pid| !protected.contains(pid))
        .filter_map(|pid| all.remove(&pid))
        .collect();
    out.sort_by_key(|p| (p.parent, p.pid));
    out
}

fn save_snapshot(root: &Path, artifact: &Path, name: &str) -> Result<Vec<OwnedProcess>, String> {
    let values = owned_processes(root, artifact);
    let json = Value::Array(
        values
            .iter()
            .map(|p| {
                json!({
                    "ProcessId": p.pid,
                    "ParentProcessId": p.parent,
                    "Name": p.name,
                    "ExecutablePath": p.executable,
                    "CommandLine": p.command_line,
                    "CreationDate": p.start_time,
                })
            })
            .collect(),
    );
    write_json(&artifact.join(format!("{name}-process-tree.json")), &json)?;
    Ok(values)
}

fn stop_owned_tree(root: &Path, artifact: &Path, phase: &str) -> Result<(), String> {
    let mut values = save_snapshot(root, artifact, phase)?;
    values.sort_by_key(|p| std::cmp::Reverse(p.pid));
    for process in values {
        if process.executable.is_empty() {
            continue;
        }
        let evidence = artifact.join(format!("{phase}-kill-{}.json", process.pid));
        if let Err(error) = crate::e2e_owned_process::kill_exact_owned_process_checked(
            process.pid,
            &process.executable,
            process.start_time as i64,
            &evidence,
        ) {
            eprintln!(
                "Warning: failed to stop repository-owned process {}: {error}",
                process.pid
            );
        }
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| format!("write {}: {e}", path.display()))
}

fn receipt_json(value: &CommandReceipt) -> Value {
    json!({"label": value.label, "status": value.status, "exit_code": value.exit_code, "log": value.log})
}

fn write_report(
    artifact: &Path,
    run_id: &str,
    code: i32,
    app: &Path,
    target: &Path,
    summary: &Value,
) -> Result<(), String> {
    let mut text = format!(
        "# Kaspa Gateway Zero-Touch E2E Script Report\n\nArtifact root: {}\nRun ID: {run_id}\nExit code: {code}\nApp binary: {}\nCargo target dir: {}",
        artifact.display(),
        app.display(),
        target.display()
    );
    if let Some(error) = summary.get("error").and_then(Value::as_str) {
        text.push_str("\n\nError: ");
        text.push_str(error);
    }
    fs::write(artifact.join("zero-touch-script-report.md"), text)
        .map_err(|e| format!("write zero-touch report: {e}"))
}

fn validation_errors(value: &Value) -> Vec<String> {
    value
        .get("validation_errors")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn stage_from_error(error: &str) -> String {
    error
        .split(':')
        .next()
        .unwrap_or("Zero-touch script")
        .to_owned()
}

fn absolute(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|p| p.join(path))
            .map_err(|e| e.to_string())
    }
}

fn now() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_error_preserves_legacy_stage_prefix() {
        assert_eq!(
            stage_from_error("E2E JavaScript syntax checks: failed"),
            "E2E JavaScript syntax checks"
        );
    }

    #[test]
    fn validation_errors_are_typed_strings() {
        let value = json!({"validation_errors": ["one", "two"]});
        assert_eq!(validation_errors(&value), vec!["one", "two"]);
    }

    #[test]
    fn wdio_full_pass_predicate_is_fail_closed() {
        let passing = json!({
            "state": {"passed": 5, "failed": 0, "skipped": 0},
            "suites": [{
                "tests": [
                    {"state": "passed"},
                    {"state": "passed"},
                    {"state": "passed"},
                    {"state": "passed"},
                    {"state": "passed"}
                ],
                "hooks": [
                    {"state": "passed"},
                    {"state": "passed"}
                ]
            }]
        });
        assert!(wdio_report_is_exact_full_pass(&passing));

        let mut failed = passing.clone();
        failed["state"]["failed"] = json!(1);
        assert!(!wdio_report_is_exact_full_pass(&failed));

        let mut skipped = passing.clone();
        skipped["state"]["skipped"] = json!(1);
        assert!(!wdio_report_is_exact_full_pass(&skipped));

        let mut bad_hook = passing.clone();
        bad_hook["suites"][0]["hooks"][1]["state"] = json!("failed");
        assert!(!wdio_report_is_exact_full_pass(&bad_hook));

        let mut short = passing;
        short["suites"][0]["tests"]
            .as_array_mut()
            .expect("tests array")
            .pop();
        assert!(!wdio_report_is_exact_full_pass(&short));
    }

    #[test]
    fn wdio_teardown_signature_requires_full_report_and_exact_stack() {
        let temp = tempfile::tempdir().expect("tempdir");
        let json_dir = temp.path().join("json");
        fs::create_dir_all(&json_dir).expect("json dir");
        let report = json!({
            "state": {"passed": 5, "failed": 0, "skipped": 0},
            "suites": [{
                "tests": [
                    {"state": "passed"},
                    {"state": "passed"},
                    {"state": "passed"},
                    {"state": "passed"},
                    {"state": "passed"}
                ],
                "hooks": [{"state": "passed"}, {"state": "passed"}]
            }]
        });
        fs::write(
            json_dir.join("wdio-0-0.json"),
            serde_json::to_vec(&report).expect("report json"),
        )
        .expect("write report");
        let log_path = temp.path().join("wdio-run.log");
        fs::write(
            &log_path,
            "Error [ERR_STREAM_WRITE_AFTER_END]: write after end\n             at LogWriter.write (.../@wdio/native-core/dist/esm/index.js:1095:24)\n             at forwardLog (.../@wdio/tauri-service/dist/esm/index.js:647:16)\n",
        )
        .expect("write log");
        assert!(wdio_post_success_teardown_race(temp.path(), &log_path).expect("classify"));

        fs::write(
            &log_path,
            "Error [ERR_STREAM_WRITE_AFTER_END]: write after end\n",
        )
        .expect("write incomplete log");
        assert!(!wdio_post_success_teardown_race(temp.path(), &log_path).expect("reject"));
    }

    #[test]
    fn runtime_contract_keeps_required_environment_names() {
        let source = include_str!("zero_touch_e2e.rs");
        for key in [
            "KGW_REPOSITORY",
            "KGW_ZERO_TOUCH_ARTIFACT_DIR",
            "KGW_E2E_RUN_ID",
            "KGW_E2E_APP_BINARY",
            "KGW_UI_TRACE_FILE",
            "KGW_UI_TRACE_DIR",
            "KGW_START_TRACE",
            "KGW_REQUIRED_PWSH_PATH",
            "TAURI_CONFIG",
            "LOCALAPPDATA",
            "APPDATA",
            "CARGO_TARGET_DIR",
        ] {
            assert!(source.contains(key), "missing {key}");
        }
    }

    #[test]
    fn generated_schemas_restore_only_when_run_made_them_dirty() {
        let temp = tempfile::tempdir().expect("tempdir");
        let schema_dir = temp.path().join(GENERATED_SCHEMAS);
        fs::create_dir_all(&schema_dir).expect("schema dir");
        let schema = schema_dir.join("schema.json");
        fs::write(&schema, b"original\n").expect("schema");
        for args in [
            vec!["init", "--quiet"],
            vec!["add", "--", GENERATED_SCHEMAS],
            vec![
                "-c",
                "user.name=KGW Test",
                "-c",
                "user.email=kgw-test@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "fixture",
            ],
        ] {
            let status = Command::new("git")
                .args(args)
                .current_dir(temp.path())
                .status()
                .expect("git");
            assert!(status.success());
        }

        fs::write(&schema, b"generated\n").expect("generated schema");
        assert!(path_dirty(temp.path(), GENERATED_SCHEMAS).expect("dirty"));
        restore_generated_schemas_if_newly_dirty(temp.path(), false).expect("restore");
        assert_eq!(
            fs::read_to_string(&schema).expect("read").trim_end(),
            "original"
        );
        assert!(!path_dirty(temp.path(), GENERATED_SCHEMAS).expect("clean"));

        fs::write(&schema, b"preexisting\n").expect("preexisting schema");
        restore_generated_schemas_if_newly_dirty(temp.path(), true).expect("preserve");
        assert_eq!(fs::read(&schema).expect("read"), b"preexisting\n");
        assert!(path_dirty(temp.path(), GENERATED_SCHEMAS).expect("still dirty"));
    }

    #[test]
    fn live_command_plan_preserves_checks_without_nested_cargo_self_rebuild() {
        let source = include_str!("zero_touch_e2e.rs");
        let runtime = source.split("#[cfg(test)]").next().expect("runtime source");
        for command in [
            "npm ci",
            "npm run e2e",
            "e2e-test",
            "std::env::current_exe()",
            "KGW_XTASK_EXE",
            "deepmerge-security-smoke",
            "e2e-static-smokes",
        ] {
            assert!(runtime.contains(command), "missing {command}");
        }
        for (relative, _) in E2E_JS_SYNTAX_CHECKS {
            assert!(
                runtime.contains(relative),
                "missing JS syntax check for {relative}"
            );
        }
        assert!(
            !runtime.contains("npm run check"),
            "zero-touch runner must not re-enter Cargo through npm run check"
        );
    }
}
