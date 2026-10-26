//! Rust-owned bootstrap and execution for the pinned workflow linter.
use crate::verified_tool::{self, ArchivePolicy};
#[cfg(test)]
use crate::verified_tool::{safe_archive_path, sha256};
use std::fs;
#[cfg(test)]
use std::io::Write;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

pub const VERSION: &str = "1.7.12";
pub const ARCHIVE_SHA256: &str = "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8";
const ARCHIVE_URL: &str = "https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz";
const MAX_BINARY_BYTES: u64 = 64 * 1024 * 1024;
const PIN: ArchivePolicy<'static> = ArchivePolicy {
    name: "actionlint",
    url: ARCHIVE_URL,
    archive_sha256: ARCHIVE_SHA256,
    max_archive_bytes: 32 * 1024 * 1024,
    max_expanded_bytes: 128 * 1024 * 1024,
    max_binary_bytes: MAX_BINARY_BYTES,
    max_entries: 1024,
};
type Result<T> = std::result::Result<T, String>;

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
                root = Some(PathBuf::from(args.next().ok_or("--root requires a path")?));
            }
            "--archive" if archive.is_none() => {
                archive = Some(PathBuf::from(
                    args.next().ok_or("--archive requires a path")?,
                ));
            }
            "--verify-only" if !verify_only => verify_only = true,
            _ => {
                return Err(format!(
                    "Unknown or duplicate workflow-lint argument: {arg}"
                ));
            }
        }
    }
    Ok(Options {
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        archive,
        verify_only,
    })
}

#[cfg(test)]
fn extract_verified(bytes: &[u8], expected: &str, destination: &Path) -> Result<String> {
    verified_tool::extract_verified(
        bytes,
        ArchivePolicy {
            archive_sha256: expected,
            ..PIN
        },
        destination,
    )
}
#[cfg(test)]
fn download_arguments(destination: &Path) -> Vec<std::ffi::OsString> {
    verified_tool::download_arguments(destination, PIN)
}
fn check_version(output: &str) -> Result<()> {
    if output.lines().next().map(str::trim) != Some(VERSION) {
        return Err(format!(
            "Pinned actionlint version mismatch; expected {VERSION}"
        ));
    }
    Ok(())
}
pub fn run(options: Options) -> Result<()> {
    if !options.verify_only && !(cfg!(target_os = "linux") && cfg!(target_arch = "x86_64")) {
        return Err("Workflow lint execution requires Linux x86_64; --verify-only never executes the binary".to_owned());
    }
    let root = fs::canonicalize(&options.root).map_err(|error| error.to_string())?;
    if !root.join("Cargo.toml").is_file() || !root.join(".github/workflows").is_dir() {
        return Err("Workflow lint requires the repository root".to_owned());
    }
    let tool = verified_tool::prepare(&root, PIN, options.archive.as_deref())?;
    println!("ACTIONLINT_ARCHIVE_SHA256={ARCHIVE_SHA256}");
    println!("ACTIONLINT_BINARY_SHA256={}", tool.binary_sha256);
    if options.verify_only {
        println!("ACTIONLINT_ARCHIVE=VERIFIED; EXECUTION=NOT_PERFORMED");
        return Ok(());
    }
    let version = Command::new(&tool.executable)
        .arg("-version")
        .output()
        .map_err(|error| error.to_string())?;
    if !version.status.success() {
        return Err("Actionlint version probe failed".to_owned());
    }
    check_version(std::str::from_utf8(&version.stdout).map_err(|error| error.to_string())?)?;
    println!("ACTIONLINT_VERSION={VERSION}");
    let shellcheck = Command::new("shellcheck")
        .arg("--version")
        .output()
        .map_err(|error| format!("shellcheck is required for full workflow lint: {error}"))?;
    if !shellcheck.status.success() {
        return Err("shellcheck version check failed".to_owned());
    }
    let status = Command::new(&tool.executable)
        .arg("-color")
        .current_dir(&root)
        .status()
        .map_err(|error| format!("Cannot run pinned actionlint: {error}"))?;
    if !status.success() {
        return Err(format!("Workflow lint failed: {status}"));
    }
    println!("WORKFLOW_LINT=PASS");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};

    fn fixture(entries: &[(&str, &[u8], tar::EntryType)]) -> Vec<u8> {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut archive = tar::Builder::new(encoder);
        for (name, bytes, kind) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o755);
            header.set_entry_type(*kind);
            header.set_cksum();
            archive.append_data(&mut header, name, *bytes).unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap()
    }
    fn extract(bytes: &[u8]) -> Result<String> {
        let root = tempfile::tempdir().unwrap();
        extract_verified(bytes, &sha256(bytes), &root.path().join("actionlint"))
    }
    #[test]
    fn archive_digest_is_checked_before_parsing_or_output() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("actionlint");
        let error = extract_verified(b"not an archive", ARCHIVE_SHA256, &target).unwrap_err();
        assert!(error.contains("SHA256 mismatch"));
        assert!(!target.exists());
    }
    #[test]
    fn extracts_only_the_exact_verified_root_binary() {
        let bytes = fixture(&[
            ("README.md", b"notes", tar::EntryType::Regular),
            ("actionlint", b"pinned fixture", tar::EntryType::Regular),
        ]);
        assert_eq!(extract(&bytes).unwrap(), sha256(b"pinned fixture"));
    }
    #[test]
    fn missing_duplicate_empty_and_nested_binaries_are_rejected() {
        for entries in [
            vec![("README", b"notes".as_slice(), tar::EntryType::Regular)],
            vec![
                ("actionlint", b"a".as_slice(), tar::EntryType::Regular),
                ("actionlint", b"b".as_slice(), tar::EntryType::Regular),
            ],
            vec![("actionlint", b"".as_slice(), tar::EntryType::Regular)],
            vec![(
                "nested/actionlint",
                b"a".as_slice(),
                tar::EntryType::Regular,
            )],
        ] {
            assert!(extract(&fixture(&entries)).is_err());
        }
    }
    #[test]
    fn archive_links_are_rejected_without_extraction() {
        assert!(extract(&fixture(&[("actionlint", b"", tar::EntryType::Symlink)])).is_err());
        assert!(extract(&fixture(&[("actionlint", b"", tar::EntryType::Link)])).is_err());
    }
    #[test]
    fn archive_paths_cannot_escape_or_use_windows_separators() {
        for value in [
            "../actionlint",
            "/actionlint",
            "C:/actionlint",
            "a/../../actionlint",
            "a\\actionlint",
        ] {
            assert!(!safe_archive_path(Path::new(value)), "{value}");
        }
        assert!(safe_archive_path(Path::new("docs/README.md")));
    }
    #[test]
    fn destination_is_never_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("actionlint");
        fs::write(&target, b"preserve").unwrap();
        let bytes = fixture(&[("actionlint", b"new", tar::EntryType::Regular)]);
        assert!(extract_verified(&bytes, &sha256(&bytes), &target).is_err());
        assert_eq!(fs::read(&target).unwrap(), b"preserve");
    }
    #[test]
    fn malformed_archive_is_not_accepted_with_a_matching_digest() {
        assert!(extract(b"not gzip").is_err());
    }
    #[test]
    fn version_mismatch_cannot_reach_workflow_execution() {
        assert!(check_version("1.7.12\ninstalled by fixture\n").is_ok());
        for output in ["", "1.7.11", "v1.7.12", "1.7.120", "error\n1.7.12"] {
            assert!(check_version(output).is_err());
        }
    }
    #[test]
    fn download_uses_exact_https_pin_and_structured_arguments() {
        let target = Path::new("folder with spaces/archive.tar.gz");
        let args = download_arguments(target);
        assert_eq!(args[0], "--disable");
        assert_eq!(args[args.len() - 2], target.as_os_str());
        assert_eq!(args.last().unwrap(), ARCHIVE_URL);
        assert!(args.iter().any(|arg| arg == "--proto-redir"));
        assert!(!args.iter().any(|arg| arg == "-k" || arg == "--insecure"));
    }
    #[test]
    fn cli_rejects_unknown_and_duplicate_arguments() {
        assert_eq!(parse(Vec::new()).unwrap().root, Path::new("."));
        for args in [
            vec!["--root"],
            vec!["--url", "https://example.invalid"],
            vec!["--root", ".", "--root", "."],
            vec!["--verify-only", "--verify-only"],
        ] {
            assert!(parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn adopted_workflow_delegates_only_to_native_rust_entrypoints() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let source = fs::read_to_string(root.join(".github/workflows/workflow-lint.yml")).unwrap();
        for required in [
            "name: Workflow Lint",
            "contents: read",
            "persist-credentials: false",
            "name: actionlint",
            "runs-on: ubuntu-24.04",
            "timeout-minutes: 30",
            "cancel-in-progress: true",
            "\"xtask/**\"",
            "workflow_dispatch:",
            "cargo test --locked -p xtask --features workflow-lint --bin kgw-workflow-lint",
            "cargo run --locked -p xtask --features workflow-lint --bin kgw-workflow-lint -- --root .",
        ] {
            assert!(
                source.contains(required),
                "Missing workflow contract: {required}"
            );
        }
        for forbidden in [
            "run: |",
            "run: >",
            "shell: bash",
            "sudo ",
            "curl ",
            "sha256sum",
            "tar -",
            "find ",
            "continue-on-error",
            "--verify-only",
            "--archive",
        ] {
            assert!(
                !source.contains(forbidden),
                "Owned script or bypass reintroduced: {forbidden}"
            );
        }
    }

    #[test]
    fn rejected_late_archive_entry_never_leaves_an_executable() {
        let bytes = fixture(&[
            ("actionlint", b"verified fixture", tar::EntryType::Regular),
            ("unwanted-link", b"", tar::EntryType::Symlink),
        ]);
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("actionlint");
        assert!(extract_verified(&bytes, &sha256(&bytes), &target).is_err());
        assert!(!target.exists());
    }

    #[test]
    fn oversized_binary_header_is_rejected_without_allocating_its_payload() {
        let mut header = tar::Header::new_gnu();
        header.set_path("actionlint").unwrap();
        header.set_size(MAX_BINARY_BYTES + 1);
        header.set_mode(0o755);
        header.set_entry_type(tar::EntryType::Regular);
        header.set_cksum();
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(header.as_bytes()).unwrap();
        let bytes = encoder.finish().unwrap();
        assert!(
            extract(&bytes)
                .unwrap_err()
                .contains("nonempty regular root entry")
        );
    }
}
