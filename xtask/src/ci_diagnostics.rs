//! Native owners for the existing CI diagnostic files and capture contracts.
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

const MAX_VALUE_BYTES: usize = 16 * 1024 * 1024;
const REVIEW_OUTPUTS: [(&str, &str); 3] = [
    ("INVALID_LICENSE_CHANGES", "invalid-license-changes.json"),
    ("VULNERABLE_CHANGES", "vulnerable-changes.json"),
    ("DENIED_CHANGES", "denied-changes.json"),
];
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    DependencyReview,
    CargoDeny,
    CargoMachete,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    pub directory: PathBuf,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Options> {
    let mut args = args.into_iter();
    let mode = match args.next().as_deref() {
        Some("dependency-review") => Mode::DependencyReview,
        Some("capture-cargo-deny") => Mode::CargoDeny,
        Some("capture-cargo-machete") => Mode::CargoMachete,
        _ => {
            return Err(
                "Expected dependency-review, capture-cargo-deny or capture-cargo-machete"
                    .to_owned(),
            );
        }
    };
    let directory = match args.next().as_deref() {
        None => PathBuf::from("."),
        Some("--directory") => PathBuf::from(args.next().ok_or("--directory requires a path")?),
        _ => return Err("Only --directory is supported after the selected operation".to_owned()),
    };
    if args.next().is_some() {
        return Err("Unexpected extra diagnostic arguments".to_owned());
    }
    Ok(Options { mode, directory })
}

fn environment_bytes(name: &str) -> Result<Vec<u8>> {
    let Some(value) = env::var_os(name) else {
        return Ok(Vec::new());
    };
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(value.into_vec())
    }
    #[cfg(not(unix))]
    {
        value
            .into_string()
            .map(String::into_bytes)
            .map_err(|_| format!("Diagnostic environment value {name} is not Unicode"))
    }
}

fn check_value_length(length: usize) -> Result<()> {
    if length > MAX_VALUE_BYTES {
        return Err("Diagnostic value exceeds its size limit".to_owned());
    }
    Ok(())
}

fn ordinary_target(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(format!(
            "Diagnostic target is not a regular file: {}",
            path.display()
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("Inspect {}: {error}", path.display())),
    }
}

fn preserve_review(directory: &Path, mut read: impl FnMut(&str) -> Result<Vec<u8>>) -> Result<()> {
    let mut records = Vec::new();
    for (name, file) in REVIEW_OUTPUTS {
        let mut value = read(name)?;
        check_value_length(value.len())?;
        value.push(b'\n');
        records.push((directory.join(file), value));
    }
    if !directory.is_dir() {
        return Err("Diagnostic directory must already exist".to_owned());
    }
    for (path, _) in &records {
        ordinary_target(path)?;
    }
    for (path, bytes) in records {
        let mut temporary = tempfile::Builder::new()
            .prefix(".kgw-ci-diagnostic-")
            .tempfile_in(directory)
            .map_err(|error| error.to_string())?;
        temporary
            .write_all(&bytes)
            .and_then(|()| temporary.as_file().sync_all())
            .map_err(|error| error.to_string())?;
        ordinary_target(&path)?;
        temporary
            .persist(&path)
            .map_err(|error| format!("Save {}: {}", path.display(), error.error))?;
        if fs::read(&path).map_err(|error| error.to_string())? != bytes {
            return Err(format!("Diagnostic read-back mismatch: {}", path.display()));
        }
    }
    Ok(())
}

fn native_command(mode: Mode) -> Result<(Command, &'static str)> {
    match mode {
        Mode::CargoDeny => {
            let mut command = Command::new("cargo");
            command.args(["deny", "check"]);
            Ok((command, "cargo-deny.log"))
        }
        Mode::CargoMachete => {
            // When spawned from Rust, cargo machete forwards the subcommand name as a
            // positional argument. Invoke the allowlisted binary directly instead.
            let mut command = Command::new("cargo-machete");
            command.arg("--with-metadata");
            Ok((command, "cargo-machete.log"))
        }
        Mode::DependencyReview => {
            Err("Dependency-review diagnostics do not execute a scanner".to_owned())
        }
    }
}

fn process_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
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

fn capture_command(
    command: &mut Command,
    directory: &Path,
    log_name: &str,
    output: &mut impl Write,
) -> Result<i32> {
    if !directory.is_dir() {
        return Err("Capture directory must already exist".to_owned());
    }
    let path = directory.join(log_name);
    ordinary_target(&path)?;
    let mut log = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|error| format!("Open diagnostic log {}: {error}", path.display()))?;
    // Both inherited handles share the same file position. There is no pipe that
    // can deadlock while the scanner writes, and partial logs survive interruption.
    command
        .current_dir(directory)
        .stdout(Stdio::from(
            log.try_clone().map_err(|error| error.to_string())?,
        ))
        .stderr(Stdio::from(
            log.try_clone().map_err(|error| error.to_string())?,
        ));
    let status = command
        .status()
        .map_err(|error| format!("Start allowlisted scanner: {error}"))?;
    let code = process_code(status);
    let relay = (|| -> io::Result<()> {
        log.sync_all()?;
        log.seek(SeekFrom::Start(0))?;
        io::copy(&mut log, output)?;
        output.flush()
    })();
    if let Err(error) = relay {
        if code == 0 {
            return Err(format!("Diagnostic capture/relay failed: {error}"));
        }
        eprintln!("Diagnostic relay failed; preserving scanner exit {code}: {error}");
    }
    Ok(code)
}

pub fn run(options: Options) -> Result<i32> {
    let directory = fs::canonicalize(options.directory).map_err(|error| error.to_string())?;
    match options.mode {
        Mode::DependencyReview => {
            preserve_review(&directory, environment_bytes)?;
            println!("DEPENDENCY_REVIEW_DIAGNOSTICS=PRESERVED; FILES=3");
            Ok(0)
        }
        mode => {
            let (mut command, log) = native_command(mode)?;
            capture_command(&mut command, &directory, log, &mut io::stdout().lock())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_names_are_exact_and_missing_values_still_write_newlines() {
        let root = tempfile::tempdir().unwrap();
        let mut seen = Vec::new();
        preserve_review(root.path(), |name| {
            seen.push(name.to_owned());
            Ok(Vec::new())
        })
        .unwrap();
        assert_eq!(seen, REVIEW_OUTPUTS.map(|(name, _)| name.to_owned()));
        for (_, name) in REVIEW_OUTPUTS {
            assert_eq!(fs::read(root.path().join(name)).unwrap(), b"\n");
        }
    }
    #[test]
    fn review_preserves_raw_bytes_including_quotes_crlf_and_unicode() {
        let root = tempfile::tempdir().unwrap();
        let value =
            "{\"text\":\"\u{0645}\u{0631}\u{062d}\u{0628}\u{0627}\"}\r\n'$(do-not-execute)'\n"
                .as_bytes();
        preserve_review(root.path(), |_| Ok(value.to_vec())).unwrap();
        for (_, name) in REVIEW_OUTPUTS {
            let mut expected = value.to_vec();
            expected.push(b'\n');
            assert_eq!(fs::read(root.path().join(name)).unwrap(), expected);
        }
    }
    #[test]
    fn review_replaces_only_three_named_files_and_leaves_no_temporary_files() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("unrelated.txt"), b"preserved").unwrap();
        for (_, name) in REVIEW_OUTPUTS {
            fs::write(root.path().join(name), b"old").unwrap();
        }
        preserve_review(root.path(), |_| Ok(b"new".to_vec())).unwrap();
        assert_eq!(
            fs::read(root.path().join("unrelated.txt")).unwrap(),
            b"preserved"
        );
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 4);
    }
    #[test]
    fn review_preflights_all_destinations_before_mutating_any_output() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join(REVIEW_OUTPUTS[0].1), b"old").unwrap();
        fs::create_dir(root.path().join(REVIEW_OUTPUTS[1].1)).unwrap();
        assert!(preserve_review(root.path(), |_| Ok(b"new".to_vec())).is_err());
        assert_eq!(
            fs::read(root.path().join(REVIEW_OUTPUTS[0].1)).unwrap(),
            b"old"
        );
    }
    #[test]
    fn diagnostic_limits_and_reader_errors_fail_before_writing() {
        assert!(check_value_length(MAX_VALUE_BYTES).is_ok());
        assert!(check_value_length(MAX_VALUE_BYTES + 1).is_err());
        let root = tempfile::tempdir().unwrap();
        assert!(preserve_review(root.path(), |_| Err("fixture input error".to_owned())).is_err());
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn scanner_command_surface_is_exact_and_not_freeform() {
        for (mode, program, expected, name) in [
            (
                Mode::CargoDeny,
                "cargo",
                vec!["deny", "check"],
                "cargo-deny.log",
            ),
            (
                Mode::CargoMachete,
                "cargo-machete",
                vec!["--with-metadata"],
                "cargo-machete.log",
            ),
        ] {
            let (command, log) = native_command(mode).unwrap();
            assert_eq!(command.get_program(), program);
            assert_eq!(
                command.get_args().collect::<Vec<_>>(),
                expected
                    .into_iter()
                    .map(std::ffi::OsStr::new)
                    .collect::<Vec<_>>()
            );
            assert_eq!(log, name);
        }
        assert!(native_command(Mode::DependencyReview).is_err());
    }
    #[test]
    fn cli_rejects_extra_programs_and_duplicate_directory_options() {
        assert_eq!(
            parse(["dependency-review".to_owned()]).unwrap().mode,
            Mode::DependencyReview
        );
        for args in [
            vec!["execute", "anything"],
            vec!["capture-cargo-deny", "--program", "cmd"],
            vec!["dependency-review", "--directory"],
            vec!["dependency-review", "--directory", ".", "--directory", "."],
        ] {
            assert!(parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn child_fixture_output() {
        let Ok(value) = env::var("KGW_CI_DIAGNOSTIC_TEST_CHILD_EXIT") else {
            return;
        };
        let code: i32 = value.parse().unwrap();
        println!("KGW_SCOPED_CHILD_STDOUT");
        eprintln!("KGW_SCOPED_CHILD_STDERR");
        io::stdout().flush().unwrap();
        io::stderr().flush().unwrap();
        std::process::exit(code);
    }
    fn fixture_command(code: i32) -> Command {
        let mut command = Command::new(env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "ci_diagnostics::tests::child_fixture_output",
                "--nocapture",
            ])
            .env("KGW_CI_DIAGNOSTIC_TEST_CHILD_EXIT", code.to_string());
        command
    }
    #[test]
    fn actual_child_stdout_stderr_and_exit_codes_are_preserved() {
        for code in [0, 7, 37] {
            let root = tempfile::tempdir().unwrap();
            let mut console = Vec::new();
            let status = capture_command(
                &mut fixture_command(code),
                root.path(),
                "fixture.log",
                &mut console,
            )
            .unwrap();
            assert_eq!(status, code);
            let saved = fs::read(root.path().join("fixture.log")).unwrap();
            assert_eq!(console, saved);
            let text = String::from_utf8(saved).unwrap();
            assert!(text.contains("KGW_SCOPED_CHILD_STDOUT"));
            assert!(text.contains("KGW_SCOPED_CHILD_STDERR"));
        }
    }
    struct BrokenOutput;
    impl Write for BrokenOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("fixture relay failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn relay_failure_never_replaces_an_original_nonzero_scanner_exit() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            capture_command(
                &mut fixture_command(37),
                root.path(),
                "failed.log",
                &mut BrokenOutput
            )
            .unwrap(),
            37
        );
        assert!(
            capture_command(
                &mut fixture_command(0),
                root.path(),
                "success.log",
                &mut BrokenOutput
            )
            .is_err()
        );
    }
    #[test]
    fn absent_scanner_is_a_failure_with_a_preserved_log() {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new(root.path().join("does-not-exist"));
        assert!(
            capture_command(&mut command, root.path(), "missing.log", &mut Vec::new()).is_err()
        );
        assert!(root.path().join("missing.log").is_file());
    }
    #[cfg(unix)]
    #[test]
    fn review_rejects_symlink_targets_without_following_them() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        fs::write(outside.path(), b"preserve").unwrap();
        symlink(outside.path(), root.path().join(REVIEW_OUTPUTS[0].1)).unwrap();
        assert!(preserve_review(root.path(), |_| Ok(b"new".to_vec())).is_err());
        assert_eq!(fs::read(outside.path()).unwrap(), b"preserve");
    }
    #[test]
    fn adopted_workflows_keep_diagnostics_and_delegate_only_to_rust() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let review =
            fs::read_to_string(root.join(".github/workflows/dependency-review.yml")).unwrap();
        for required in [
            "fail-on-severity: moderate",
            "fail-on-scopes: runtime, development, unknown",
            "failure() && steps.review.outcome == 'failure'",
            "retention-days: 3",
            "cargo run --locked -p xtask --bin kgw-ci-diagnostics -- dependency-review --directory .",
        ] {
            assert!(
                review.contains(required),
                "Missing review contract: {required}"
            );
        }
        for (variable, filename) in REVIEW_OUTPUTS {
            assert!(review.contains(variable));
            assert!(review.contains(filename));
        }
        let security = fs::read_to_string(root.join(".github/workflows/security.yml")).unwrap();
        for required in [
            "name: policy + audit + deny + machete",
            "contents: read",
            "persist-credentials: false",
            "cargo audit",
            "cargo tree --locked -d",
            "cargo test --locked -p xtask --bin kgw-ci-diagnostics",
            "capture-cargo-deny --directory .",
            "capture-cargo-machete --directory .",
            "cargo-deny.log",
            "cargo-machete.log",
            "fallback: none",
            "failure() && steps.cargo-deny.outcome == 'failure'",
            "failure() && steps.cargo-machete.outcome == 'failure'",
        ] {
            assert!(
                security.contains(required),
                "Missing security contract: {required}"
            );
        }
        for source in [&review, &security] {
            for forbidden in [
                "run: |",
                "run: >",
                "shell: bash",
                "PIPESTATUS",
                "continue-on-error",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "Owned script or bypass reintroduced: {forbidden}"
                );
            }
        }
    }
}
