use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const E2E_DIR: &str = "e2e";
const SYNTAX_CHECKS: [&str; 7] = [
    "wdio.conf.mjs",
    "specs/zero-touch-live-matrix.e2e.js",
    "helpers/assertions.mjs",
    "helpers/paths.mjs",
    "helpers/tauri-app.mjs",
    "helpers/windows.mjs",
    "helpers/runtime-ports.mjs",
];
const ESLINT_CLI: &str = "node_modules/eslint/bin/eslint.js";
const WDIO_CLI: &str = "node_modules/@wdio/cli/bin/wdio.js";

fn run_output(cwd: &Path, program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to launch {program}: {error}"))
}

fn run_checked(cwd: &Path, label: &str, program: &str, args: &[&str]) -> Result<(), String> {
    let output = run_output(cwd, program, args)?;
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{label} failed with exit code {}",
            output.status.code().unwrap_or(1)
        ))
    }
}

fn require_file(path: &Path, label: &str) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{label} missing: {}", path.display()))
    }
}

pub(crate) fn wdio_cli(root: &Path) -> Result<PathBuf, String> {
    let path = root.join(E2E_DIR).join(WDIO_CLI);
    require_file(&path, "WebdriverIO CLI")?;
    Ok(path)
}

pub(crate) fn run(root: &Path) -> Result<String, String> {
    let e2e = root.join(E2E_DIR);
    require_file(&e2e.join(ESLINT_CLI), "ESLint CLI")?;
    require_file(&e2e.join(WDIO_CLI), "WebdriverIO CLI")?;

    run_checked(&e2e, "E2E ESLint", "node", &[ESLINT_CLI, "."])?;
    for relative in SYNTAX_CHECKS {
        run_checked(
            &e2e,
            &format!("node --check {relative}"),
            "node",
            &["--check", relative],
        )?;
    }

    let current_exe = std::env::current_exe()
        .map_err(|error| format!("resolve current xtask executable: {error}"))?;
    let current_exe = current_exe.to_string_lossy().into_owned();
    run_checked(
        root,
        "deepmerge-security-smoke",
        &current_exe,
        &["deepmerge-security-smoke"],
    )?;
    run_checked(
        root,
        "e2e-static-smokes",
        &current_exe,
        &["e2e-static-smokes"],
    )?;

    Ok("KGW E2E workspace Rust owner PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_entrypoints_are_direct_node_files_not_npm_scripts() {
        assert_eq!(ESLINT_CLI, "node_modules/eslint/bin/eslint.js");
        assert_eq!(WDIO_CLI, "node_modules/@wdio/cli/bin/wdio.js");
        assert_eq!(SYNTAX_CHECKS.len(), 7);
        assert!(SYNTAX_CHECKS.contains(&"wdio.conf.mjs"));
        assert!(SYNTAX_CHECKS.contains(&"specs/zero-touch-live-matrix.e2e.js"));
    }
}
