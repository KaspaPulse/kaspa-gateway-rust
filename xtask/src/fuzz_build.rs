use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TARGET: &str = "gateway_config_json";

fn cargo_fuzz_command(root: &Path) -> Command {
    let mut command = Command::new("cargo");
    command
        .args([
            "fuzz",
            "build",
            "--sanitizer",
            "address",
            "-O",
            "--debug-assertions",
            TARGET,
        ])
        .env("RUSTUP_TOOLCHAIN", "nightly")
        .current_dir(root);
    for (source, target) in [
        ("KGW_CFL_CFLAGS", "CFLAGS"),
        ("KGW_CFL_CXXFLAGS", "CXXFLAGS"),
    ] {
        if let Some(value) = env::var_os(source).filter(|value| !value.is_empty()) {
            command.env(target, value);
        }
    }
    command
}

pub fn run(root: &Path) -> Result<String, String> {
    let out = env::var_os("OUT")
        .map(PathBuf::from)
        .ok_or_else(|| "fuzz-build: OUT environment variable is required".to_owned())?;

    let status = cargo_fuzz_command(root)
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
    fn cargo_fuzz_build_owns_address_sanitizer_configuration() {
        let command = cargo_fuzz_command(Path::new("fixture-root"));
        assert_eq!(command.get_program(), "cargo");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "fuzz",
                "build",
                "--sanitizer",
                "address",
                "-O",
                "--debug-assertions",
                TARGET,
            ]
            .map(std::ffi::OsStr::new)
            .to_vec()
        );
        assert!(command.get_envs().any(|(key, value)| {
            key == "RUSTUP_TOOLCHAIN" && value == Some(std::ffi::OsStr::new("nightly"))
        }));
    }

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
