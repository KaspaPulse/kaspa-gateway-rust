//! Native orchestration for the existing pinned TruffleHog and Rust result policy.
use crate::{
    trufflehog_policy,
    verified_tool::{self, ArchivePolicy},
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug)]
pub struct Failure {
    pub code: i32,
    message: String,
}
impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self { code: 1, message }
    }
}
impl From<&str> for Failure {
    fn from(message: &str) -> Self {
        Self::from(message.to_owned())
    }
}
impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::ops::Deref for Failure {
    type Target = str;
    fn deref(&self) -> &str {
        &self.message
    }
}
impl Failure {
    fn scanner(status: std::process::ExitStatus) -> Self {
        let code = match status.code() {
            Some(code) => code,
            None => {
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    128 + status.signal().unwrap_or(1)
                }
                #[cfg(not(unix))]
                {
                    1
                }
            }
        };
        Self {
            code,
            message: format!(
                "Secret scanner failed with {status}; result policy was not allowed to convert this to success"
            ),
        }
    }
}
type Result<T> = std::result::Result<T, Failure>;

pub const VERSION: &str = "3.96.0";
pub const ARCHIVE_SHA256: &str = "7105f1cd6577f058a9e39d0578f1a99c8a1e481e4d3512cd8a09acfe22a0fdc0";
const PIN: ArchivePolicy<'static> = ArchivePolicy {
    name: "trufflehog",
    url: "https://github.com/trufflesecurity/trufflehog/releases/download/v3.96.0/trufflehog_3.96.0_linux_amd64.tar.gz",
    archive_sha256: ARCHIVE_SHA256,
    max_archive_bytes: 64 * 1024 * 1024,
    max_expanded_bytes: 512 * 1024 * 1024,
    max_binary_bytes: 256 * 1024 * 1024,
    max_entries: 1024,
};
const SCAN_ARGUMENTS: [&str; 9] = [
    "--no-update",
    "--no-color",
    "--results=verified,unknown",
    "--fail-on-scan-errors",
    "--json",
    "git",
    "file://.",
    "--branch",
    "HEAD",
];

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub root: PathBuf,
    pub archive: Option<PathBuf>,
    pub verify_only: bool,
}
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Options> {
    let mut args = args.into_iter();
    let mut root = None;
    let mut archive = None;
    let mut verify_only = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" if root.is_none() => {
                root = Some(PathBuf::from(args.next().ok_or("--root requires a path")?))
            }
            "--archive" if archive.is_none() => {
                archive = Some(PathBuf::from(
                    args.next().ok_or("--archive requires a path")?,
                ))
            }
            "--verify-only" if !verify_only => verify_only = true,
            _ => {
                return Err(Failure::from(format!(
                    "Unknown or duplicate secret-scan option: {arg}"
                )));
            }
        }
    }
    Ok(Options {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        archive,
        verify_only,
    })
}
fn scanner_command(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    command.args(SCAN_ARGUMENTS);
    command
}
fn check_version(stdout: &[u8], stderr: &[u8]) -> Result<()> {
    let source = if stdout.is_empty() { stderr } else { stdout };
    let text = std::str::from_utf8(source).map_err(|error| error.to_string())?;
    if text.trim() != format!("trufflehog {VERSION}") {
        return Err(Failure::from(format!(
            "Pinned TruffleHog version mismatch; expected {VERSION}"
        )));
    }
    Ok(())
}
fn require_full_history(value: &str) -> Result<()> {
    if value.trim() != "false" {
        return Err(Failure::from(
            "Secret scan requires a complete Git history, not a shallow checkout".to_owned(),
        ));
    }
    Ok(())
}
fn execute_scan(command: &mut Command, root: &Path, private_directory: &Path) -> Result<String> {
    let mut results = tempfile::Builder::new()
        .prefix("trufflehog-results-")
        .tempfile_in(private_directory)
        .map_err(|error| error.to_string())?;
    command.current_dir(root).stdout(Stdio::from(
        results
            .as_file()
            .try_clone()
            .map_err(|error| error.to_string())?,
    ));
    let status = command
        .status()
        .map_err(|error| format!("Cannot start verified secret scanner: {error}"))?;
    results
        .flush()
        .and_then(|()| results.as_file().sync_all())
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(Failure::scanner(status));
    }
    // Preserve the existing exact policy implementation; do not print raw findings.
    trufflehog_policy::check_file(results.path()).map_err(Failure::from)
}
pub fn run(options: Options) -> Result<()> {
    if !options.verify_only && !(cfg!(target_os = "linux") && cfg!(target_arch = "x86_64")) {
        return Err(Failure::from(
            "Secret scan execution requires Linux x86_64; --verify-only never executes the binary"
                .to_owned(),
        ));
    }
    let root = fs::canonicalize(options.root).map_err(|error| error.to_string())?;
    if !root.join("Cargo.toml").is_file()
        || !root.join(".github/workflows/secret-scan.yml").is_file()
    {
        return Err(Failure::from(
            "Secret scan requires the repository root".to_owned(),
        ));
    }
    let tool = verified_tool::prepare(&root, PIN, options.archive.as_deref())?;
    println!("TRUFFLEHOG_ARCHIVE_SHA256={ARCHIVE_SHA256}");
    println!("TRUFFLEHOG_BINARY_SHA256={}", tool.binary_sha256);
    if options.verify_only {
        println!("TRUFFLEHOG_ARCHIVE=VERIFIED; SCAN=NOT_PERFORMED");
        return Ok(());
    }
    let version = Command::new(&tool.executable)
        .args(["--no-update", "--version"])
        .output()
        .map_err(|error| error.to_string())?;
    if !version.status.success() {
        return Err(Failure::from("TruffleHog version check failed".to_owned()));
    }
    check_version(&version.stdout, &version.stderr)?;
    let full = Command::new("git")
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(&root)
        .output()
        .map_err(|error| error.to_string())?;
    if !full.status.success() {
        return Err(Failure::from(
            "Cannot validate Git history before secret scan".to_owned(),
        ));
    }
    require_full_history(std::str::from_utf8(&full.stdout).map_err(|error| error.to_string())?)?;
    let mut command = scanner_command(&tool.executable);
    println!(
        "{}",
        execute_scan(&mut command, &root, tool._directory.path())?
    );
    println!("SECRET_SCAN=PASS");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_arguments_preserve_history_scope_and_failure_semantics() {
        let command = scanner_command(Path::new("verified/trufflehog"));
        assert_eq!(command.get_program(), "verified/trufflehog");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            SCAN_ARGUMENTS.map(std::ffi::OsStr::new)
        );
        assert!(!SCAN_ARGUMENTS.iter().any(|arg| arg.contains("exclude")
            || arg.contains("only-verified")
            || arg.contains("no-verification")));
    }
    #[test]
    fn version_and_history_are_explicit_fail_closed_inputs() {
        assert!(check_version(b"trufflehog 3.96.0\n", b"").is_ok());
        assert!(check_version(b"", b"trufflehog 3.96.0\n").is_ok());
        for value in [
            b"3.96.0".as_slice(),
            b"trufflehog 3.96.1",
            b"warning\ntrufflehog 3.96.0",
            b"",
        ] {
            assert!(check_version(value, b"").is_err());
        }
        assert!(require_full_history("false\n").is_ok());
        for value in ["true", "", "False", "unknown"] {
            assert!(require_full_history(value).is_err());
        }
    }
    #[test]
    fn cli_cannot_change_the_scanner_or_policy_arguments() {
        assert!(!parse(Vec::new()).unwrap().verify_only);
        for args in [
            vec!["--root"],
            vec!["--skip-policy"],
            vec!["--program", "arbitrary"],
            vec!["--verify-only", "--verify-only"],
        ] {
            assert!(parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn child_fixture_scan() {
        let Ok(mode) = std::env::var("KGW_SECRET_SCAN_TEST_MODE") else {
            return;
        };
        let data = match mode.as_str() {
            "empty" => "",
            "invalid" => "not-json\n",
            "unexpected" => "{\"Raw\":\"SYNTHETIC_VALUE_MUST_NOT_BE_PRINTED\"}\n",
            "failed" => "not-json\n",
            _ => panic!("invalid fixture mode"),
        };
        std::io::stdout().write_all(data.as_bytes()).unwrap();
        std::io::stdout().flush().unwrap();
        std::process::exit(if mode == "failed" { 37 } else { 0 });
    }
    fn child(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "secret_scan::tests::child_fixture_scan",
                "--nocapture",
            ])
            .env("KGW_SECRET_SCAN_TEST_MODE", mode);
        command
    }
    // The standard Rust test harness writes its own test header to stdout. Strip
    // nothing from real scanner output: a dedicated fixture executable below is
    // used by the complete caller-parity test; native unit tests assert rejection.
    #[test]
    fn scanner_failure_is_not_overridden_by_result_parsing() {
        let root = tempfile::tempdir().unwrap();
        let error = execute_scan(&mut child("failed"), root.path(), root.path()).unwrap_err();
        assert!(error.contains("scanner failed"));
        assert_eq!(error.code, 37);
        assert!(!error.contains("invalid TruffleHog JSON"));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn invalid_output_is_rejected_and_temporary_findings_are_removed() {
        let root = tempfile::tempdir().unwrap();
        let error = execute_scan(&mut child("invalid"), root.path(), root.path()).unwrap_err();
        assert!(error.contains("invalid TruffleHog JSON"));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn missing_scanner_does_not_leave_sensitive_result_files() {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new(root.path().join("missing"));
        assert!(execute_scan(&mut command, root.path(), root.path()).is_err());
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn exact_native_scanner_output_reaches_unchanged_policy_without_raw_logging() {
        let root = tempfile::tempdir().unwrap();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/secret_scan_child.rs");
        let binary = root.path().join(format!(
            "scoped-scan-fixture{}",
            std::env::consts::EXE_SUFFIX
        ));
        let compile = Command::new("rustc")
            .args(["--edition=2021", "--crate-name", "kgw_scoped_scan_fixture"])
            .arg(source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "fixture compile: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let private = root.path().join("results");
        fs::create_dir(&private).unwrap();
        for (mode, pass, needle) in [
            ("empty", true, "unexpected=0"),
            ("invalid", false, "invalid TruffleHog JSON"),
            ("scalar", false, "invalid TruffleHog result type"),
            ("unexpected", false, "unexpected finding"),
            ("allowed", true, "accepted=1"),
            ("duplicate", false, "more than once"),
            ("failed", false, "scanner failed"),
        ] {
            let mut command = Command::new(&binary);
            command.arg(mode);
            let result = execute_scan(&mut command, root.path(), &private);
            assert_eq!(result.is_ok(), pass, "{mode}: {result:?}");
            let message = result.unwrap_or_else(|error| error.to_string());
            assert!(message.contains(needle), "{mode}: {message}");
            assert!(!message.contains("SYNTHETIC_VALUE_MUST_NOT_BE_PRINTED"));
            assert_eq!(
                fs::read_dir(&private).unwrap().count(),
                0,
                "temporary result leaked: {mode}"
            );
        }
    }
    #[test]
    fn adopted_secret_workflow_preserves_full_history_and_exact_result_policy() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let source = fs::read_to_string(root.join(".github/workflows/secret-scan.yml")).unwrap();
        let binary = fs::read_to_string(root.join("xtask/src/bin/kgw-secret-scan.rs")).unwrap();
        assert!(
            binary.contains("#[path = \"../trufflehog_policy.rs\"]")
                && binary.contains("mod trufflehog_policy;"),
            "kgw-secret-scan binary must compile and execute the exact TruffleHog policy tests"
        );
        for required in [
            "timeout-minutes: 30",
            "fetch-depth: 0",
            "persist-credentials: false",
            "contents: read",
            "cargo test --locked -p xtask --features secret-scan --bin kgw-secret-scan",
            "cargo run --locked -p xtask --features secret-scan --bin kgw-secret-scan -- --root .",
        ] {
            assert!(
                source.contains(required),
                "Missing required secret-scan contract: {required}"
            );
        }
        for forbidden in [
            "run: |",
            "run: >",
            "shell: bash",
            "sudo ",
            "curl ",
            "mktemp",
            "trap ",
            "continue-on-error",
            "timeout-minutes: 15",
            "name: Validate exact historical false-positive policy",
            "cargo test --locked -p xtask trufflehog_policy::tests::",
            "--verify-only",
            "--archive",
        ] {
            assert!(
                !source.contains(forbidden),
                "Owned script or bypass reintroduced: {forbidden}"
            );
        }
    }
}
