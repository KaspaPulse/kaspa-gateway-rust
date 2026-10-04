use super::*;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::process::Command;

pub fn hash_text(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
pub fn hash_file(path: &Path) -> EvidenceResult<String> {
    let mut file =
        fs::File::open(path).map_err(|error| format!("Cannot open {}: {error}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 81920];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn git(repository: &Path, args: &[&str]) -> EvidenceResult<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()
        .map_err(|error| format!("Cannot execute read-only git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed with exit code {}",
            args.first().unwrap_or(&""),
            output.status.code().unwrap_or(-1)
        ));
    }
    Ok(output.stdout)
}
pub fn commit(repository: &Path) -> EvidenceResult<String> {
    let output = git(repository, &["rev-parse", "HEAD"])?;
    Ok(String::from_utf8_lossy(&output).trim().to_owned())
}

fn require_source_root(repository: &Path) -> EvidenceResult<()> {
    let supplied = fs::canonicalize(repository)
        .map_err(|error| format!("Cannot resolve source repository root: {error}"))?;
    let output = git(repository, &["rev-parse", "--show-toplevel"])?;
    let reported = String::from_utf8(output)
        .map_err(|error| format!("Invalid Git repository root text: {error}"))?;
    let reported = reported.trim_end_matches(['\r', '\n']);
    let actual = fs::canonicalize(reported)
        .map_err(|error| format!("Cannot resolve Git working-tree root: {error}"))?;
    if supplied != actual {
        return Err(
            "Source identity requires the actual Git working-tree root, not a subdirectory"
                .to_owned(),
        );
    }
    Ok(())
}

fn require_unstaged_context(repository: &Path) -> EvidenceResult<()> {
    let staged = git(
        repository,
        &["diff", "--cached", "--name-only", "-z", "--no-ext-diff"],
    )?;
    if !staged.is_empty() {
        return Err("The v2 source fingerprint requires an empty staged index; staged changes need separate qualification".to_owned());
    }
    Ok(())
}

fn untracked_paths(output: &[u8]) -> EvidenceResult<Vec<String>> {
    if output.is_empty() {
        return Ok(Vec::new());
    }
    let body = output
        .strip_suffix(&[0])
        .ok_or_else(|| "Git untracked path list is not NUL terminated".to_owned())?;
    let mut paths = Vec::new();
    for entry in body.split(|byte| *byte == 0) {
        let path = std::str::from_utf8(entry)
            .map_err(|error| format!("Invalid UTF-8 in Git untracked path: {error}"))?;
        if path.is_empty()
            || path.chars().any(char::is_control)
            || !Path::new(path)
                .components()
                .all(|part| matches!(part, std::path::Component::Normal(_)))
        {
            return Err("Ambiguous or non-relative untracked path cannot be represented by the v2 source fingerprint".to_owned());
        }
        paths.push(path.to_owned());
    }
    paths.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
    if paths.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("Duplicate untracked path in Git source fingerprint input".to_owned());
    }
    Ok(paths)
}

fn untracked_record(repository: &Path, path: &str) -> EvidenceResult<String> {
    let full = repository.join(path);
    let metadata = fs::symlink_metadata(&full)
        .map_err(|error| format!("Cannot inspect untracked source {path}: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Untracked source must be a regular file, not a symbolic link or directory: {path}"
        ));
    }
    let hash = hash_file(&full)?;
    Ok(format!("{path}\t{}\t{hash}\n", metadata.len()))
}

pub fn source_diff(repository: &Path) -> EvidenceResult<String> {
    // Preserve the existing v2 wire bytes for an unambiguous root/empty-index
    // context. Refuse unsupported contexts rather than silently omitting source.
    require_source_root(repository)?;
    require_unstaged_context(repository)?;
    let diff = git(
        repository,
        &[
            "diff",
            "--binary",
            "--no-ext-diff",
            "--",
            ".",
            ":(exclude)artifacts",
            ":(exclude)e2e/node_modules",
        ],
    )?;
    let output = git(
        repository,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            ".",
            ":(exclude)artifacts",
            ":(exclude)e2e/node_modules",
        ],
    )?;
    let paths = untracked_paths(&output)?;
    let mut digest = Sha256::new();
    digest.update(b"kgw-zero-touch-source-diff-v2\ntracked-diff-bytes\n");
    digest.update(diff);
    digest.update(b"\nuntracked-files\n");
    for path in paths {
        digest.update(untracked_record(repository, &path)?.as_bytes());
    }
    // Catch index changes during collection without rewriting or locking it.
    require_unstaged_context(repository)?;
    Ok(format!("{:x}", digest.finalize()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_hash_is_sha256_over_exact_utf8_bytes() {
        assert_eq!(
            hash_text(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_ne!(hash_text("a\n"), hash_text("a\r\n"));
    }

    fn fixture_git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "fixture git failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn identity_fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fixture_git(
            dir.path(),
            &["init", "--quiet", "--initial-branch=identity-fixture"],
        );
        fixture_git(dir.path(), &["config", "core.quotePath", "true"]);
        fixture_git(dir.path(), &["config", "core.autocrlf", "false"]);
        dir
    }

    #[test]
    fn source_identity_ascii_untracked_keeps_existing_v2_wire_bytes() {
        let dir = identity_fixture();
        let source = "fn example() {}\n";
        fs::write(dir.path().join("source.rs"), source).unwrap();
        let mut expected = Sha256::new();
        expected.update(b"kgw-zero-touch-source-diff-v2\ntracked-diff-bytes\n\nuntracked-files\n");
        expected.update(format!(
            "source.rs\t{}\t{}\n",
            source.len(),
            hash_text(source)
        ));
        assert_eq!(
            source_diff(dir.path()).unwrap(),
            format!("{:x}", expected.finalize())
        );
    }

    #[test]
    fn source_identity_staged_changes_are_not_silently_ignored() {
        let dir = identity_fixture();
        fs::write(dir.path().join("staged.rs"), "fn staged() {}\n").unwrap();
        fixture_git(dir.path(), &["add", "--", "staged.rs"]);
        assert!(
            source_diff(dir.path()).is_err(),
            "v2 context cannot prove a nonempty staged index"
        );
    }

    #[test]
    fn source_identity_unicode_untracked_contents_change_the_fingerprint() {
        let dir = identity_fixture();
        let file = dir.path().join("\u{0645}\u{0644}\u{0641}.rs");
        fs::write(&file, "first\n").unwrap();
        let first = source_diff(dir.path()).unwrap();
        fs::write(&file, "second\n").unwrap();
        assert_ne!(
            source_diff(dir.path()).unwrap(),
            first,
            "quoted Unicode file was silently omitted"
        );
    }

    #[test]
    fn source_identity_subdirectory_cannot_qualify_as_whole_repository() {
        let dir = identity_fixture();
        let child = dir.path().join("nested");
        fs::create_dir(&child).unwrap();
        fs::write(dir.path().join("root-only.rs"), "root source\n").unwrap();
        assert!(
            source_diff(&child).is_err(),
            "a subdirectory must not omit sibling source from identity"
        );
    }

    #[test]
    fn source_identity_nul_parser_preserves_unicode_and_rejects_ambiguous_records() {
        let unicode = "\u{0645}\u{0644}\u{0641}.rs\0";
        assert_eq!(
            untracked_paths(unicode.as_bytes()).unwrap(),
            vec!["\u{0645}\u{0644}\u{0641}.rs"]
        );
        assert_eq!(
            untracked_paths(b"b.rs\0a.rs\0").unwrap(),
            vec!["a.rs", "b.rs"]
        );
        assert!(untracked_paths(b"").unwrap().is_empty());
        for bad in [
            &b"unterminated"[..],
            b"\0",
            b"a.rs\0a.rs\0",
            b"../escape.rs\0",
            b"/absolute.rs\0",
            b"a\tname.rs\0",
            b"a\nname.rs\0",
            b"a\rname.rs\0",
            b"\xff.rs\0",
        ] {
            assert!(
                untracked_paths(bad).is_err(),
                "accepted ambiguous record: {bad:?}"
            );
        }
    }

    #[test]
    fn source_identity_missing_file_and_directory_never_disappear_silently() {
        let dir = tempfile::tempdir().unwrap();
        assert!(untracked_record(dir.path(), "missing.rs").is_err());
        fs::create_dir(dir.path().join("directory.rs")).unwrap();
        assert!(untracked_record(dir.path(), "directory.rs").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn source_identity_untracked_symlink_cannot_hash_external_source() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("target.rs"), "contents").unwrap();
        std::os::unix::fs::symlink(dir.path().join("target.rs"), dir.path().join("link.rs"))
            .unwrap();
        assert!(untracked_record(dir.path(), "link.rs").is_err());
    }
}
