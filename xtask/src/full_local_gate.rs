use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    repository: Option<PathBuf>,
    reuse_successful_e2e_artifact: Option<PathBuf>,
    commit_on_success: bool,
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<String, String> {
    let options = parse_args(args)?;
    let root = options
        .repository
        .unwrap_or_else(|| default_root.to_path_buf());
    let root = root.canonicalize().map_err(|error| {
        format!(
            "full-local-gate: invalid repository {}: {error}",
            root.display()
        )
    })?;
    run(
        &root,
        options.reuse_successful_e2e_artifact.as_deref(),
        options.commit_on_success,
    )
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repository" => {
                options.repository = Some(PathBuf::from(next_value(args, "--repository")?))
            }
            "--reuse-successful-e2e-artifact" => {
                options.reuse_successful_e2e_artifact = Some(PathBuf::from(next_value(
                    args,
                    "--reuse-successful-e2e-artifact",
                )?));
            }
            "--commit-on-success" => options.commit_on_success = true,
            _ => return Err(format!("full-local-gate: unknown argument {arg}")),
        }
    }
    Ok(options)
}
fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("full-local-gate: missing value for {flag}"))
}

fn command_output(root: &Path, program: &str, args: &[String]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("full-local-gate: failed to launch {program}: {error}"))
}

fn run_checked(root: &Path, label: &str, program: &str, args: &[&str]) -> Result<(), String> {
    println!("Running: {label}");
    let args = args
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>();
    let output = command_output(root, program, &args)?;
    emit_output(&output);
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{label} failed with exit code {}",
            output.status.code().unwrap_or(1)
        ))
    }
}

fn emit_output(output: &Output) {
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
fn path_dirty(root: &Path, relative: &str) -> Result<bool, String> {
    let output = command_output(
        root,
        "git",
        &[
            "status".to_owned(),
            "--porcelain".to_owned(),
            "--".to_owned(),
            relative.to_owned(),
        ],
    )?;
    if !output.status.success() {
        emit_output(&output);
        return Err(format!("git status failed for {relative}"));
    }
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn powershell_parser_checks(root: &Path) -> Result<(), String> {
    let script = r#"$e=$null; [void][System.Management.Automation.Language.Parser]::ParseFile($args[0],[ref]$null,[ref]$e); if($e -and $e.Count){$e | ForEach-Object { [Console]::Error.WriteLine($_.Message) }; exit 1}"#;
    for relative in [
        "tools/kgw_zero_touch_e2e.ps1",
        "tools/kgw_zero_touch_evidence.ps1",
        "e2e/helpers/kgw_windows_clipboard.ps1",
    ] {
        let absolute = root.join(relative);
        if !absolute.is_file() {
            return Err(format!(
                "Missing PowerShell script for parser check: {relative}"
            ));
        }
        let args = vec![
            "-NoLogo".to_owned(),
            "-NoProfile".to_owned(),
            "-Command".to_owned(),
            script.to_owned(),
            absolute.to_string_lossy().into_owned(),
        ];
        let output = command_output(root, "pwsh", &args)?;
        emit_output(&output);
        if !output.status.success() {
            return Err(format!("PowerShell parser check failed for {relative}"));
        }
        println!("PowerShell parser PASS: {relative}");
    }
    Ok(())
}
fn validate_reused_e2e_artifact(root: &Path, artifact: &Path) -> Result<(), String> {
    let artifact = artifact
        .canonicalize()
        .map_err(|error| format!("invalid E2E artifact {}: {error}", artifact.display()))?;
    let manifest = root.join("Cargo.toml");
    let args = vec![
        "run".to_owned(),
        "--manifest-path".to_owned(),
        manifest.to_string_lossy().into_owned(),
        "--locked".to_owned(),
        "-p".to_owned(),
        "xtask".to_owned(),
        "--bin".to_owned(),
        "kgw-zero-touch-evidence".to_owned(),
        "--".to_owned(),
        "integrity".to_owned(),
        "--repository".to_owned(),
        root.to_string_lossy().into_owned(),
        "--artifact-directory".to_owned(),
        artifact.to_string_lossy().into_owned(),
    ];
    let output = command_output(root, "cargo", &args)?;
    emit_output(&output);
    let code = output.status.code().unwrap_or(1);
    if !matches!(code, 0 | 1) {
        return Err(format!(
            "Rust E2E evidence validation could not complete; exit code {code}"
        ));
    }
    let result: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Rust E2E evidence validation returned invalid JSON: {error}"))?;
    let passed = result
        .get("passed")
        .and_then(Value::as_bool)
        .ok_or_else(|| "Rust E2E evidence validation omitted boolean passed".to_owned())?;
    if (code == 0) != passed {
        return Err(
            "Rust E2E evidence validation returned inconsistent pass/exit result".to_owned(),
        );
    }
    if !passed {
        let errors = result
            .get("errors")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        return Err(format!(
            "Rejected E2E artifact reuse: {}",
            errors
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }
    println!(
        "Reusing verified zero-touch E2E artifact: {}",
        artifact.display()
    );
    Ok(())
}
fn graphify_refresh(root: &Path) -> Result<(), String> {
    let graph = root.join("graphify-out").join("graph.json");
    if graph.is_file() {
        run_checked(
            root,
            "Graphify code-only incremental refresh",
            "graphify",
            &["update", "."],
        )?;
    } else {
        run_checked(
            root,
            "Graphify code-only extraction",
            "graphify",
            &["extract", ".", "--code-only"],
        )?;
    }
    run_checked(
        root,
        "Graphify post-change query",
        "graphify",
        &[
            "query",
            "How does the zero-touch WebdriverIO E2E suite verify raw node and bridge logs and clipboard hashes?",
            "--budget",
            "4000",
        ],
    )
}

fn restore_generated_schemas_if_newly_dirty(
    root: &Path,
    initially_dirty: bool,
) -> Result<(), String> {
    let relative = "apps/kaspa-gateway-desktop/src-tauri/gen/schemas";
    if initially_dirty || !path_dirty(root, relative)? {
        return Ok(());
    }
    run_checked(
        root,
        "Restore generated Tauri schemas produced by local validation",
        "git",
        &["restore", "--worktree", "--", relative],
    )
}

fn run_live_zero_touch(root: &Path) -> Result<(), String> {
    let script = root.join("tools/kgw_zero_touch_e2e.ps1");
    let repository = root.to_string_lossy().into_owned();
    let script = script.to_string_lossy().into_owned();
    let args = [
        "-NoLogo",
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
        script.as_str(),
        "-Repository",
        repository.as_str(),
    ];
    run_checked(root, "Zero-touch live E2E suite", "pwsh", &args)
}
fn run(root: &Path, reuse: Option<&Path>, commit_on_success: bool) -> Result<String, String> {
    powershell_parser_checks(root)?;
    let schemas_dirty = path_dirty(root, "apps/kaspa-gateway-desktop/src-tauri/gen/schemas")?;

    run_checked(
        root,
        "Rust formatting check",
        "cargo",
        &["fmt", "--all", "--", "--check"],
    )?;
    run_checked(
        root,
        "Desktop E2E feature cargo check",
        "cargo",
        &[
            "check",
            "--locked",
            "-p",
            "kaspa-gateway-desktop",
            "--features",
            "e2e-test",
        ],
    )?;
    for (label, filter) in [
        ("Rust typed raw log tests", "typed_raw_log"),
        (
            "Rust child raw log fixture test",
            "child_stdout_and_stderr_fixtures_survive_unchanged",
        ),
        (
            "Rust official raw log sentinel test",
            "official_sentinel_stdout_and_stderr_use_the_production_pipe_reader_unchanged",
        ),
        (
            "Rust raw log isolation test",
            "raw_log_buffers_are_isolated_by_network_and_role_with_process_wide_bridge_output",
        ),
    ] {
        run_checked(
            root,
            label,
            "cargo",
            &[
                "test",
                "-p",
                "kaspa-gateway-desktop",
                filter,
                "--test",
                "integrated_runtime_ipc_smoke_tests",
                "--",
                "--nocapture",
            ],
        )?;
    }
    run_checked(
        root,
        "Node frontend syntax",
        "node",
        &[
            "--check",
            "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
        ],
    )?;
    run_checked(
        root,
        "Bridge frontend syntax",
        "node",
        &[
            "--check",
            "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        ],
    )?;
    run_checked(
        root,
        "True raw log frontend Rust owner",
        "cargo",
        &[
            "run",
            "--locked",
            "-p",
            "xtask",
            "--",
            "true-raw-log-frontend-regressions",
        ],
    )?;
    run_checked(
        &root.join("e2e"),
        "E2E workspace checks",
        "npm",
        &["run", "check"],
    )?;
    run_checked(
        root,
        "Zero-touch result writer tests",
        "cargo",
        &[
            "run",
            "--locked",
            "-p",
            "xtask",
            "--",
            "zero-touch-result-writer-tests",
        ],
    )?;
    run_checked(
        root,
        "True raw log gate",
        "cargo",
        &["run", "--locked", "-p", "xtask", "--", "true-raw-log-gate"],
    )?;

    if let Some(artifact) = reuse {
        validate_reused_e2e_artifact(root, artifact)?;
    } else {
        run_live_zero_touch(root)?;
    }

    graphify_refresh(root)?;
    restore_generated_schemas_if_newly_dirty(root, schemas_dirty)?;
    run_checked(
        root,
        "Git diff whitespace check",
        "git",
        &["diff", "--check"],
    )?;
    if commit_on_success {
        commit_scoped_changes(root)?;
    }
    Ok("KGW full local gate passed. No push was performed.".to_owned())
}
fn commit_scoped_changes(root: &Path) -> Result<(), String> {
    let paths = [
        ".gitignore",
        "Cargo.lock",
        "apps/kaspa-gateway-desktop/frontend/index.html",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.template.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.template.js",
        "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml",
        "apps/kaspa-gateway-desktop/src-tauri/build.rs",
        "apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json",
        "apps/kaspa-gateway-desktop/src-tauri/tauri.e2e.conf.json",
        "apps/kaspa-gateway-desktop/src-tauri/capabilities-e2e",
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        "apps/kaspa-gateway-desktop/src-tauri/tests/integrated_runtime_ipc_smoke_tests.rs",
        "xtask/src/live_raw_log_matrix.rs",
        "xtask/src/raw_log_clipboard_capture.rs",
        "xtask/src/true_raw_log.rs",
        "xtask/src/true_raw_log_frontend.rs",
        "xtask/src/main.rs",
        "tools/kgw_zero_touch_e2e.ps1",
        "tools/kgw_zero_touch_evidence.ps1",
        "e2e/package.json",
        "e2e/package-lock.json",
        "e2e/wdio.conf.mjs",
        "e2e/capabilities",
        "e2e/helpers",
        "e2e/specs",
    ];
    let mut args = vec!["add".to_owned(), "--".to_owned()];
    args.extend(paths.iter().map(|path| (*path).to_owned()));
    let output = command_output(root, "git", &args)?;
    emit_output(&output);
    if !output.status.success() {
        return Err("git add failed".to_owned());
    }
    run_checked(
        root,
        "Commit scoped full-local changes",
        "git",
        &[
            "commit",
            "-m",
            "fix: automate and verify true raw node and bridge logs",
        ],
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_accepts_all_supported_options() {
        let mut args = vec![
            "--repository".to_owned(),
            "repo".to_owned(),
            "--reuse-successful-e2e-artifact".to_owned(),
            "artifact".to_owned(),
            "--commit-on-success".to_owned(),
        ]
        .into_iter();
        let parsed = parse_args(&mut args).unwrap();
        assert_eq!(parsed.repository, Some(PathBuf::from("repo")));
        assert_eq!(
            parsed.reuse_successful_e2e_artifact,
            Some(PathBuf::from("artifact"))
        );
        assert!(parsed.commit_on_success);
    }

    #[test]
    fn parser_rejects_unknown_arguments() {
        let mut args = vec!["--surprise".to_owned()].into_iter();
        assert!(
            parse_args(&mut args)
                .unwrap_err()
                .contains("unknown argument")
        );
    }

    #[test]
    fn parser_requires_flag_values() {
        let mut args = vec!["--repository".to_owned()].into_iter();
        assert!(parse_args(&mut args).unwrap_err().contains("missing value"));
    }
}
