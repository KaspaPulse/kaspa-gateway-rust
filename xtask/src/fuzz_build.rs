use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TARGET: &str = "gateway_config_json";

pub fn run(root: &Path) -> Result<String, String> {
    let out = env::var_os("OUT")
        .map(PathBuf::from)
        .ok_or_else(|| "fuzz-build: OUT environment variable is required".to_owned())?;

    let status = Command::new("cargo")
        .args(["fuzz", "build", "-O", "--debug-assertions", TARGET])
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .current_dir(root)
        .status()
        .map_err(|error| format!("fuzz-build: failed to start cargo fuzz: {error}"))?;
    if !status.success() {
        return Err(format!(
            "fuzz-build: cargo fuzz failed with status {status}"
        ));
    }

    let binary = find_fuzz_binary(root)?
        .ok_or_else(|| format!("{TARGET} was built but its release binary was not found"))?;
    fs::create_dir_all(&out)
        .map_err(|error| format!("fuzz-build: failed to create {}: {error}", out.display()))?;
    let destination = out.join(TARGET);
    fs::copy(&binary, &destination).map_err(|error| {
        format!(
            "fuzz-build: failed to copy {} to {}: {error}",
            binary.display(),
            destination.display()
        )
    })?;
    Ok(format!("FUZZ_BINARY={}", binary.display()))
}

fn find_fuzz_binary(root: &Path) -> Result<Option<PathBuf>, String> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("fuzz-build: cannot read {}: {error}", directory.display()))?;
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("fuzz-build: directory entry error: {error}"))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("fuzz-build: file type error: {error}"))?;
            if file_type.is_dir() {
                if should_descend(&path) {
                    stack.push(path);
                }
                continue;
            }
            if file_type.is_file() && is_release_target(&path) && is_executable(&path)? {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}
fn should_descend(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    !matches!(name, ".git" | "node_modules" | "graphify-out")
}

fn is_release_target(path: &Path) -> bool {
    if path.file_name().and_then(|value| value.to_str()) != Some(TARGET) {
        return false;
    }
    path.parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        == Some("release")
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path).map_err(|error| {
        format!(
            "fuzz-build: metadata failed for {}: {error}",
            path.display()
        )
    })?;
    Ok(metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> Result<bool, String> {
    Ok(path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_release_target_name() {
        assert!(is_release_target(Path::new(
            "fuzz/target/x86_64-unknown-linux-gnu/release/gateway_config_json"
        )));
        assert!(!is_release_target(Path::new(
            "fuzz/target/debug/gateway_config_json"
        )));
        assert!(!is_release_target(Path::new("fuzz/target/release/other")));
    }

    #[test]
    fn skips_generated_or_repository_metadata_directories() {
        assert!(!should_descend(Path::new(".git")));
        assert!(!should_descend(Path::new("graphify-out")));
        assert!(should_descend(Path::new("fuzz")));
    }
}
