//! Shared bounded, digest-first archive bootstrap for pinned external CI tools.
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy)]
pub struct ArchivePolicy<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub archive_sha256: &'a str,
    pub max_archive_bytes: u64,
    pub max_expanded_bytes: u64,
    pub max_binary_bytes: u64,
    pub max_entries: usize,
}

pub struct VerifiedTool {
    pub _directory: tempfile::TempDir,
    pub executable: PathBuf,
    pub binary_sha256: String,
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn safe_archive_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
        && !path.to_string_lossy().contains('\\')
        && !path.to_string_lossy().contains(':')
}

fn read_archive(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let file = File::open(path).map_err(|error| format!("Read {}: {error}", path.display()))?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("Verified archive must be a regular file".to_owned());
    }
    let mut bytes = Vec::new();
    file.take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > maximum {
        return Err("Verified archive is empty or exceeds the size limit".to_owned());
    }
    Ok(bytes)
}

pub fn extract_verified(
    bytes: &[u8],
    policy: ArchivePolicy<'_>,
    destination: &Path,
) -> Result<String> {
    if bytes.len() as u64 > policy.max_archive_bytes || sha256(bytes) != policy.archive_sha256 {
        return Err("Pinned tool archive SHA256 mismatch or size limit exceeded".to_owned());
    }
    // Account for actual decompressed bytes, not only untrusted header sizes.
    let decoder = GzDecoder::new(bytes).take(policy.max_expanded_bytes + 1);
    let mut archive = tar::Archive::new(decoder);
    let mut selected = None;
    let mut declared = 0_u64;
    for (count, entry) in archive
        .entries()
        .map_err(|error| error.to_string())?
        .enumerate()
    {
        if count >= policy.max_entries {
            return Err("Archive entry limit exceeded".to_owned());
        }
        let mut entry = entry.map_err(|error| error.to_string())?;
        let path = entry
            .path()
            .map_err(|error| error.to_string())?
            .into_owned();
        if !safe_archive_path(&path) {
            return Err(format!("Unsafe archive path: {}", path.display()));
        }
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            return Err("Archive links and special entries are not permitted".to_owned());
        }
        declared = declared
            .checked_add(entry.size())
            .ok_or("Archive size overflow")?;
        if declared > policy.max_expanded_bytes {
            return Err("Expanded archive limit exceeded".to_owned());
        }
        if path == Path::new(policy.name) {
            let size = entry.size();
            if !kind.is_file() || selected.is_some() || size == 0 || size > policy.max_binary_bytes
            {
                return Err("Tool binary must be one nonempty regular root entry".to_owned());
            }
            let mut binary = Vec::new();
            entry
                .by_ref()
                .take(policy.max_binary_bytes + 1)
                .read_to_end(&mut binary)
                .map_err(|error| error.to_string())?;
            if binary.len() as u64 != size {
                return Err("Binary size differs from its archive header".to_owned());
            }
            selected = Some(binary);
        }
    }
    let mut decoder = archive.into_inner();
    io::copy(&mut decoder, &mut io::sink())
        .map_err(|error| format!("Invalid gzip tail: {error}"))?;
    if decoder.limit() == 0 {
        return Err("Actual decompressed archive limit exceeded".to_owned());
    }
    let binary = selected.ok_or("Pinned binary is missing from the verified archive")?;
    let expected_hash = sha256(&binary);
    // No output is created until all archive entries and the gzip trailer validate.
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination)
        .map_err(|error| format!("Create {}: {error}", destination.display()))?;
    file.write_all(&binary)
        .and_then(|()| file.sync_all())
        .map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o755))
            .map_err(|error| error.to_string())?;
    }
    drop(file);
    drop(binary);
    let mut hasher = Sha256::new();
    let mut input = File::open(destination).map_err(|error| error.to_string())?;
    let mut buffer = [0_u8; 65536];
    loop {
        let count = input.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    if format!("{:x}", hasher.finalize()) != expected_hash {
        return Err("Extracted tool read-back mismatch".to_owned());
    }
    Ok(expected_hash)
}

pub fn download_arguments(
    destination: &Path,
    policy: ArchivePolicy<'_>,
) -> Vec<std::ffi::OsString> {
    [
        "--disable",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--tlsv1.2",
        "--fail",
        "--location",
        "--silent",
        "--show-error",
        "--connect-timeout",
        "10",
        "--max-time",
        "90",
        "--max-filesize",
    ]
    .into_iter()
    .map(Into::into)
    .chain([
        policy.max_archive_bytes.to_string().into(),
        "--output".into(),
        destination.as_os_str().to_owned(),
        policy.url.into(),
    ])
    .collect()
}

pub fn prepare(
    root: &Path,
    policy: ArchivePolicy<'_>,
    supplied_archive: Option<&Path>,
) -> Result<VerifiedTool> {
    if !policy.url.starts_with("https://github.com/")
        || policy.archive_sha256.len() != 64
        || !policy
            .archive_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || policy.name.is_empty()
        || !policy
            .name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("Invalid pinned tool policy".to_owned());
    }
    let parent = root.join("target/kgw-verified-tools");
    fs::create_dir_all(&parent).map_err(|error| error.to_string())?;
    let directory = tempfile::Builder::new()
        .prefix(policy.name)
        .tempdir_in(&parent)
        .map_err(|error| error.to_string())?;
    let downloaded = directory.path().join("archive.tar.gz");
    let path = match supplied_archive {
        Some(path) => path,
        None => {
            let status = Command::new("curl")
                .args(download_arguments(&downloaded, policy))
                .status()
                .map_err(|error| format!("Cannot start HTTPS downloader: {error}"))?;
            if !status.success() {
                return Err(format!("Pinned tool HTTPS download failed: {status}"));
            }
            &downloaded
        }
    };
    let bytes = read_archive(path, policy.max_archive_bytes)?;
    let executable = directory.path().join(policy.name);
    let binary_sha256 = extract_verified(&bytes, policy, &executable)?;
    Ok(VerifiedTool {
        _directory: directory,
        executable,
        binary_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::{Compression, write::GzEncoder};

    fn sample() -> Vec<u8> {
        let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
        let mut header = tar::Header::new_gnu();
        header.set_size(7);
        header.set_mode(0o755);
        header.set_cksum();
        archive
            .append_data(&mut header, "fixture-tool", b"fixture".as_slice())
            .unwrap();
        archive.into_inner().unwrap().finish().unwrap()
    }
    fn policy(hash: &str) -> ArchivePolicy<'_> {
        ArchivePolicy {
            name: "fixture-tool",
            url: "https://github.com/example/tool",
            archive_sha256: hash,
            max_archive_bytes: 32768,
            max_expanded_bytes: 65536,
            max_binary_bytes: 1024,
            max_entries: 20,
        }
    }
    #[test]
    fn corrupt_gzip_trailer_is_rejected_before_binary_output() {
        let mut bytes = sample();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("tool");
        assert!(extract_verified(&bytes, policy(&sha256(&bytes)), &target).is_err());
        assert!(!target.exists());
    }
    #[test]
    fn actual_tar_expansion_cap_includes_headers_and_padding() {
        let bytes = sample();
        let hash = sha256(&bytes);
        let mut pin = policy(&hash);
        pin.max_expanded_bytes = 600;
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("tool");
        assert!(extract_verified(&bytes, pin, &target).is_err());
        assert!(!target.exists());
    }
    #[test]
    fn supplied_pin_cannot_create_unsafe_output_names() {
        let root = tempfile::tempdir().unwrap();
        let mut pin = policy("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        pin.name = "../unsafe";
        assert!(prepare(root.path(), pin, None).is_err());
        assert!(!root.path().join("target").exists());
    }
}
