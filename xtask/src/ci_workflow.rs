use std::env;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

const CI_PATH: &str = ".github/workflows/ci.yml";
const CHECKOUT_SHA: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const RUST_SHA: &str = "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c";
const NODE_SHA: &str = "actions/setup-node@820762786026740c76f36085b0efc47a31fe5020";
const MSRV_TEST_ARGS: &[&str] = &[
    "test",
    "--locked",
    "--workspace",
    "--all-targets",
    "--no-run",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    KsssAdoption,
    Msrv,
    Quality,
    NodeCurrent,
}

impl Stage {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "ksss-adoption" => Ok(Self::KsssAdoption),
            "msrv" => Ok(Self::Msrv),
            "quality" => Ok(Self::Quality),
            "node-current" => Ok(Self::NodeCurrent),
            _ => Err(format!("ci-workflow-stage: unsupported stage {value}")),
        }
    }
}

fn current_xtask() -> Result<PathBuf, String> {
    env::current_exe().map_err(|error| format!("ci workflow: resolve current xtask: {error}"))
}

fn status(root: &Path, program: &str, args: &[&str]) -> Result<(), String> {
    println!("CI_RUN {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|error| format!("ci workflow: launch {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ci workflow: {program} failed with {status}"))
    }
}

fn xtask(root: &Path, args: &[&str]) -> Result<(), String> {
    let exe = current_xtask()?;
    let program = exe.to_string_lossy().into_owned();
    status(root, &program, args)
}

fn forbid_retired_release_admin_secret(path: &Path, text: &str) -> Result<(), String> {
    if text.contains("RELEASE_ADMIN_TOKEN") {
        Err(format!(
            "retired GitHub Actions secret RELEASE_ADMIN_TOKEN must not be referenced by workflow: {}",
            path.display()
        ))
    } else {
        Ok(())
    }
}

fn verify_retired_workflow_secrets(root: &Path) -> Result<(), String> {
    let workflows = root.join(".github/workflows");
    for entry in fs::read_dir(&workflows)
        .map_err(|error| format!("ci workflow: read {}: {error}", workflows.display()))?
    {
        let path = entry
            .map_err(|error| format!("ci workflow: read workflow entry: {error}"))?
            .path();
        if !path.is_file()
            || !matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("yml" | "yaml")
            )
        {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("ci workflow: read {}: {error}", path.display()))?;
        forbid_retired_release_admin_secret(&path, &text)?;
    }
    Ok(())
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("ci workflow: launch {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "ci workflow: {program} failed with {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("ci workflow: non-UTF8 output from {program}: {error}"))
}

fn status_to_log(
    root: &Path,
    program: &str,
    args: &[&str],
    stdout_path: &Path,
    stderr_path: Option<&Path>,
) -> Result<ExitStatus, String> {
    let stdout = File::create(stdout_path)
        .map_err(|error| format!("ci workflow: create {}: {error}", stdout_path.display()))?;
    let stderr = match stderr_path {
        Some(path) => Stdio::from(
            File::create(path)
                .map_err(|error| format!("ci workflow: create {}: {error}", path.display()))?,
        ),
        None => Stdio::from(
            stdout
                .try_clone()
                .map_err(|error| format!("ci workflow: clone log handle: {error}"))?,
        ),
    };
    Command::new(program)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::from(stdout))
        .stderr(stderr)
        .status()
        .map_err(|error| format!("ci workflow: launch {program}: {error}"))
}

fn require_success(label: &str, result: ExitStatus) -> Result<(), String> {
    if result.success() {
        Ok(())
    } else {
        Err(format!("ci workflow: {label} failed with {result}"))
    }
}

fn install_tauri_linux_prerequisites(root: &Path) -> Result<(), String> {
    status(root, "sudo", &["apt-get", "update"])?;
    status(
        root,
        "sudo",
        &[
            "apt-get",
            "install",
            "--yes",
            "--no-install-recommends",
            "libwebkit2gtk-4.1-dev",
            "build-essential",
            "curl",
            "wget",
            "file",
            "libxdo-dev",
            "libssl-dev",
            "libayatana-appindicator3-dev",
            "librsvg2-dev",
            "protobuf-compiler",
        ],
    )
}

fn configure_data_root() -> Result<PathBuf, String> {
    let root = env::var_os("KASPA_GATEWAY_DATA_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "ci workflow: KASPA_GATEWAY_DATA_DIR is required".to_owned())?;
    if root.exists() {
        fs::remove_dir_all(&root)
            .map_err(|error| format!("ci workflow: remove {}: {error}", root.display()))?;
    }
    fs::create_dir_all(&root)
        .map_err(|error| format!("ci workflow: create {}: {error}", root.display()))?;
    Ok(root)
}

fn verify_npm(root: &Path, expected: &str) -> Result<(), String> {
    let actual = output(root, "npm", &["--version"])?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "ci workflow: npm version {actual} != expected {expected}"
        ))
    }
}

fn npm_ci(workspace: &Path, log: &Path) -> Result<(), String> {
    let result = status_to_log(workspace, "npm", &["ci", "--ignore-scripts"], log, None)?;
    require_success("npm ci --ignore-scripts", result)
}

fn run_ksss(root: &Path) -> Result<(), String> {
    let temp = env::var_os("RUNNER_TEMP")
        .map(PathBuf::from)
        .ok_or_else(|| "ci workflow: RUNNER_TEMP is required".to_owned())?;
    let trust = temp.join("kgw-ksss-trust.json");
    let check = temp.join("kgw-ksss-check.json");
    let evaluation = temp.join("kgw-ksss-evaluation.json");
    let base = env::var("BASE_SHA").unwrap_or_default();
    xtask(
        root,
        &[
            "ksss",
            "trust-verify",
            "--require-cryptographic",
            "--cosign",
            "cosign",
            "--output",
            &trust.to_string_lossy(),
        ],
    )?;
    xtask(
        root,
        &["ksss", "check", "--output", &check.to_string_lossy()],
    )?;
    status(
        root,
        "cargo",
        &[
            "test",
            "--locked",
            "-p",
            "xtask",
            "--features",
            "ksss",
            "ksss::tests",
        ],
    )?;
    xtask(
        root,
        &[
            "ksss",
            "evaluate",
            "--base",
            &base,
            "--output",
            &evaluation.to_string_lossy(),
        ],
    )
}

fn run_msrv(root: &Path) -> Result<(), String> {
    let rustc = output(root, "rustc", &["--version"])?;
    if !rustc.starts_with("rustc 1.97.1 ") {
        return Err(format!(
            "ci workflow: MSRV stage requires rustc 1.97.1, observed {rustc}"
        ));
    }
    install_tauri_linux_prerequisites(root)?;
    let _data_root = configure_data_root()?;
    status(
        root,
        "cargo",
        &["check", "--locked", "--workspace", "--all-targets"],
    )?;
    status(
        root,
        "cargo",
        &[
            "check",
            "--locked",
            "-p",
            "kaspa-gateway-frontend-wasm",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )?;
    status(
        root,
        "cargo",
        &[
            "check",
            "--locked",
            "-p",
            "kaspa-gateway-e2e-wasm",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )?;
    status(root, "cargo", MSRV_TEST_ARGS)
}

fn run_clippy_phase(
    root: &Path,
    cargo_args: &[&str],
    json_log: &str,
    stderr_log: &str,
    policy_log: &str,
) -> Result<bool, String> {
    let cargo_status = status_to_log(
        root,
        "cargo",
        cargo_args,
        &root.join(json_log),
        Some(&root.join(stderr_log)),
    )?;
    let exe = current_xtask()?;
    let program = exe.to_string_lossy().into_owned();
    let policy_status = status_to_log(
        root,
        &program,
        &["check-clippy-results", json_log],
        &root.join(policy_log),
        None,
    )?;
    Ok(cargo_status.success() && policy_status.success())
}

fn run_quality(root: &Path) -> Result<(), String> {
    verify_retired_workflow_secrets(root)?;
    install_tauri_linux_prerequisites(root)?;
    status(
        root,
        "cargo",
        &[
            "check",
            "--locked",
            "-p",
            "kaspa-gateway-frontend-wasm",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )?;
    status(
        root,
        "cargo",
        &[
            "check",
            "--locked",
            "-p",
            "kaspa-gateway-e2e-wasm",
            "--target",
            "wasm32-unknown-unknown",
        ],
    )?;
    xtask(root, &["frontend-template-codegen", "check"])?;
    xtask(root, &["frontend-wasm-codegen", "check"])?;
    xtask(root, &["e2e-wasm-codegen", "check"])?;
    xtask(root, &["e2e-config-codegen", "check"])?;
    xtask(root, &["language-policy", "check"])?;
    xtask(root, &["settings-contract-regressions"])?;
    verify_npm(root, "11.17.0")?;
    let _data_root = configure_data_root()?;

    let desktop = root.join("apps/kaspa-gateway-desktop");
    let e2e = root.join("e2e");
    npm_ci(&desktop, &desktop.join("npm-ci.log"))?;
    npm_ci(&e2e, &e2e.join("npm-ci.log"))?;
    xtask(
        root,
        &[
            "npm-dependency-policy-gate",
            "--workspace",
            "desktop",
            "--ci-log",
            "apps/kaspa-gateway-desktop/npm-ci.log",
        ],
    )?;
    xtask(
        root,
        &[
            "npm-dependency-policy-gate",
            "--workspace",
            "e2e",
            "--ci-log",
            "e2e/npm-ci.log",
        ],
    )?;
    status(
        root,
        "cargo",
        &[
            "test",
            "--locked",
            "-p",
            "xtask",
            "npm_dependency_policy::tests",
        ],
    )?;
    status(
        &desktop,
        "node",
        &["node_modules/eslint/bin/eslint.js", "frontend/**/*.js"],
    )?;

    for args in [
        &["effective-node-settings-gate"][..],
        &["effective-bridge-settings-gate"][..],
        &["network-generation-gate"][..],
        &["runtime-repository-binding-gate", "--offline", "--strict"][..],
        &["runtime-automation-claims-gate"][..],
        &["desktop-version-contract-gate"][..],
        &["desktop-artifacts-workflow-gate"][..],
        &["desktop-release-draft-workflow-gate"][..],
        &["production-trust-readiness-gate"][..],
        &["e2e-workspace-checks"][..],
    ] {
        xtask(root, args)?;
    }
    for filter in [
        "network_generation::tests",
        "runtime_repository_binding::tests",
    ] {
        status(root, "cargo", &["test", "--locked", "-p", "xtask", filter])?;
    }
    status(root, "cargo", &["fmt", "--all", "--", "--check"])?;

    let cargo_check = status_to_log(
        root,
        "cargo",
        &["check", "--locked", "--workspace", "--all-targets"],
        &root.join("cargo-check.log"),
        None,
    )?;
    require_success("cargo check", cargo_check)?;

    let mut clippy_ok = true;
    clippy_ok &= run_clippy_phase(
        root,
        &[
            "clippy",
            "--locked",
            "--workspace",
            "--lib",
            "--bins",
            "--examples",
            "--benches",
            "--message-format=json",
            "--",
            "-D",
            "warnings",
            "--cap-lints",
            "warn",
        ],
        "clippy-production.jsonl",
        "clippy-production.stderr.log",
        "clippy-production-policy.log",
    )?;
    clippy_ok &= run_clippy_phase(
        root,
        &[
            "clippy",
            "--locked",
            "--workspace",
            "--tests",
            "--exclude",
            "kaspa-gateway-desktop",
            "--message-format=json",
            "--",
            "-D",
            "warnings",
            "--cap-lints",
            "warn",
        ],
        "clippy-workspace-tests.jsonl",
        "clippy-workspace-tests.stderr.log",
        "clippy-workspace-tests-policy.log",
    )?;
    clippy_ok &= run_clippy_phase(
        root,
        &[
            "clippy",
            "--locked",
            "-p",
            "kaspa-gateway-desktop",
            "--tests",
            "--message-format=json",
            "--",
            "-D",
            "warnings",
            "-A",
            "dead-code",
            "--cap-lints",
            "warn",
        ],
        "clippy-desktop-tests.jsonl",
        "clippy-desktop-tests.stderr.log",
        "clippy-desktop-tests-policy.log",
    )?;
    if !clippy_ok {
        return Err("ci workflow: one or more Clippy policy phases failed".to_owned());
    }

    let tests = status_to_log(
        root,
        "cargo",
        &["test", "--locked", "--workspace", "--all-targets"],
        &root.join("rust-tests.log"),
        None,
    )?;
    require_success("workspace Rust tests", tests)
}

fn run_node_current(root: &Path) -> Result<(), String> {
    verify_npm(root, "11.19.0")?;
    let desktop = root.join("apps/kaspa-gateway-desktop");
    let e2e = root.join("e2e");
    status(&desktop, "npm", &["ci", "--ignore-scripts"])?;
    status(&e2e, "npm", &["ci", "--ignore-scripts"])?;
    status(
        &desktop,
        "node",
        &["node_modules/eslint/bin/eslint.js", "frontend/**/*.js"],
    )?;
    status(&e2e, "node", &["node_modules/eslint/bin/eslint.js", "."])?;
    for file in [
        "wdio.conf.mjs",
        "specs/zero-touch-live-matrix.e2e.js",
        "helpers/assertions.mjs",
        "helpers/paths.mjs",
        "helpers/tauri-app.mjs",
        "helpers/windows.mjs",
        "helpers/runtime-ports.mjs",
    ] {
        status(&e2e, "node", &["--check", file])?;
    }
    Ok(())
}

pub fn run_stage(root: &Path, value: &str) -> Result<String, String> {
    let stage = Stage::parse(value)?;
    match stage {
        Stage::KsssAdoption => run_ksss(root)?,
        Stage::Msrv => run_msrv(root)?,
        Stage::Quality => run_quality(root)?,
        Stage::NodeCurrent => run_node_current(root)?,
    }
    Ok(format!("CI_WORKFLOW_STAGE={} PASS", value))
}

fn require(text: &str, marker: &str) -> Result<(), String> {
    if text.contains(marker) {
        Ok(())
    } else {
        Err(format!("ci workflow contract: missing {marker}"))
    }
}

fn forbid(text: &str, marker: &str) -> Result<(), String> {
    if text.contains(marker) {
        Err(format!(
            "ci workflow contract: executable logic must be Rust-owned; found {marker}"
        ))
    } else {
        Ok(())
    }
}

fn require_job_marker(text: &str, job: &str, next_job: &str, marker: &str) -> Result<(), String> {
    let start = format!("  {job}:");
    let end = format!("\n  {next_job}:");
    let (_, after_start) = text
        .split_once(&start)
        .ok_or_else(|| format!("ci workflow contract: missing job {job}"))?;
    let (block, _) = after_start
        .split_once(&end)
        .ok_or_else(|| format!("ci workflow contract: missing job boundary {next_job}"))?;
    require(block, marker)
        .map_err(|_| format!("ci workflow contract: job {job} is missing required marker {marker}"))
}

fn require_last_job_marker(text: &str, job: &str, marker: &str) -> Result<(), String> {
    let start = format!("  {job}:");
    let (_, block) = text
        .split_once(&start)
        .ok_or_else(|| format!("ci workflow contract: missing job {job}"))?;
    require(block, marker)
        .map_err(|_| format!("ci workflow contract: job {job} is missing required marker {marker}"))
}

pub fn run_gate(root: &Path) -> Result<String, String> {
    let workflow = fs::read_to_string(root.join(CI_PATH))
        .map_err(|error| format!("ci workflow contract: read {CI_PATH}: {error}"))?;
    for marker in [
        "ksss-adoption:",
        "msrv:",
        "quality:",
        "node-current-compatibility:",
        CHECKOUT_SHA,
        RUST_SHA,
        NODE_SHA,
        "RUSTUP_TOOLCHAIN: 1.97.1",
        "cargo run --locked -p xtask --features ksss -- ci-workflow-stage ksss-adoption",
        "cargo run --locked -p xtask -- ci-workflow-stage msrv",
        "cargo run --locked -p xtask -- ci-workflow-stage quality",
        "cargo run --locked -p xtask -- ci-workflow-stage node-current",
    ] {
        require(&workflow, marker)?;
    }
    require_job_marker(&workflow, "ksss-adoption", "msrv", "timeout-minutes: 30")?;
    require_last_job_marker(
        &workflow,
        "node-current-compatibility",
        "timeout-minutes: 30",
    )?;
    for marker in [
        "run: |",
        "run: >",
        "shell: bash",
        "shell: sh",
        "shell: pwsh",
        "shell: powershell",
        "sudo apt-get",
        "npm ci",
        "npm --version",
        "cargo check --locked",
        "cargo test --locked",
        "cargo clippy",
        "node --check",
        "node node_modules",
        "set +e",
        "PIPESTATUS",
    ] {
        forbid(&workflow, marker)?;
    }
    Ok("CI WORKFLOW RUST OWNERSHIP CONTRACT PASSED".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_parser_is_fail_closed() {
        assert_eq!(Stage::parse("quality"), Ok(Stage::Quality));
        assert!(Stage::parse("unknown").is_err());
    }

    #[test]
    fn contract_rejects_shell_execution_tokens() {
        for marker in [
            "run: |",
            "run: >",
            "shell: bash",
            "shell: sh",
            "shell: pwsh",
            "shell: powershell",
            "sudo apt-get",
            "npm ci",
            "cargo clippy",
            "PIPESTATUS",
        ] {
            assert!(forbid(marker, marker).is_err());
        }
    }

    #[test]
    fn contract_accepts_rust_stage_invocations() {
        for marker in [
            "cargo run --locked -p xtask -- ci-workflow-stage msrv",
            "cargo run --locked -p xtask -- ci-workflow-stage quality",
        ] {
            assert!(require(marker, marker).is_ok());
        }
    }

    #[test]
    fn ksss_job_requires_bounded_cold_build_timeout() {
        let workflow =
            "jobs:\n  ksss-adoption:\n    timeout-minutes: 30\n  msrv:\n    timeout-minutes: 60\n";
        assert!(
            require_job_marker(workflow, "ksss-adoption", "msrv", "timeout-minutes: 30").is_ok()
        );
        assert!(
            require_job_marker(workflow, "ksss-adoption", "msrv", "timeout-minutes: 10").is_err()
        );
    }

    #[test]
    fn msrv_compiles_workspace_test_targets_without_execution() {
        assert_eq!(
            MSRV_TEST_ARGS,
            &[
                "test",
                "--locked",
                "--workspace",
                "--all-targets",
                "--no-run",
            ]
        );
    }

    #[test]
    fn retired_release_admin_secret_reference_fails_closed() {
        let path = Path::new(".github/workflows/example.yml");
        assert!(forbid_retired_release_admin_secret(path, "name: safe").is_ok());
        assert!(forbid_retired_release_admin_secret(path, "# RELEASE_ADMIN_TOKEN").is_err());
    }
}
