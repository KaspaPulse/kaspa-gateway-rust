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
pub fn source_diff(repository: &Path) -> EvidenceResult<String> {
    // Keep the existing v2 wire fingerprint: unstaged binary diff plus sorted
    // untracked file records. Final qualification must also bind the Git index.
    let diff = git(
        repository,
        &[
            "diff",
            "--binary",
            "--no-ext-diff",
            "--",
            ".",
            ":(exclude)artifacts",
            ":(exclude)graphify-out",
            ":(exclude)e2e/node_modules",
        ],
    )?;
    let output = git(
        repository,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            ".",
            ":(exclude)artifacts",
            ":(exclude)graphify-out",
            ":(exclude)e2e/node_modules",
        ],
    )?;
    let listing =
        String::from_utf8(output).map_err(|error| format!("Invalid Git path text: {error}"))?;
    let mut paths = listing
        .lines()
        .filter(|path| !path.trim().is_empty())
        .collect::<Vec<_>>();
    paths.sort_by(|left, right| left.encode_utf16().cmp(right.encode_utf16()));
    let mut digest = Sha256::new();
    digest.update(b"kgw-zero-touch-source-diff-v2\ntracked-diff-bytes\n");
    digest.update(diff);
    digest.update(b"\nuntracked-files\n");
    for path in paths {
        let full = repository.join(path);
        if !full.is_file() {
            continue;
        }
        let length = fs::metadata(&full)
            .map_err(|error| error.to_string())?
            .len();
        digest.update(format!("{path}\t{length}\t{}\n", hash_file(&full)?).as_bytes());
    }
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
}
