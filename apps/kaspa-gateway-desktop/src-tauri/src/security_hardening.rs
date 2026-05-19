use kaspa_gateway_config::default_user_data_dir;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_SNAPSHOT_ITEMS: usize = 200;
const MAX_COPY_BYTES_PER_FILE: u64 = 100 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct SecurityValidationReport {
    pub safe: bool,
    pub redacted_preview: String,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecoverySnapshotReport {
    pub snapshot_dir: String,
    pub copied_items: Vec<String>,
    pub skipped_items: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityHardeningReport {
    pub process_args_policy: String,
    pub redaction_policy: String,
    pub path_policy: String,
    pub recovery_policy: String,
    pub status: String,
    pub checks: Vec<String>,
}

#[tauri::command]
pub fn security_redact_text(input: String) -> String {
    redact_sensitive_text(&input)
}

#[tauri::command]
pub fn security_validate_process_args(
    executable: String,
    args: Vec<String>,
) -> Result<SecurityValidationReport, String> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if let Err(error) = validate_process_target_path(&executable) {
        errors.push(error);
    }

    if !Path::new(&executable).is_file() {
        warnings.push("Program path does not currently point to a file.".to_string());
    }

    if args.len() > 256 {
        errors.push("Too many process arguments.".to_string());
    }

    for arg in &args {
        if let Err(error) = validate_safe_arg(arg) {
            errors.push(error);
        }
    }

    let preview = format!("{} {}", executable, args.join(" "));
    let redacted_preview = redact_sensitive_text(&preview);

    Ok(SecurityValidationReport {
        safe: errors.is_empty(),
        redacted_preview,
        warnings,
        errors,
    })
}

#[tauri::command]
pub fn security_validate_path(path: String) -> Result<SecurityValidationReport, String> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    if let Err(error) = validate_path_text(&path) {
        errors.push(error);
    }

    let path_buf = PathBuf::from(path.trim());

    if !path_buf.exists() {
        warnings.push("Path does not currently exist.".to_string());
    }

    if is_sensitive_system_root(&path_buf) {
        warnings.push(
            "Path points to a sensitive system root. Avoid destructive operations here."
                .to_string(),
        );
    }

    Ok(SecurityValidationReport {
        safe: errors.is_empty(),
        redacted_preview: redact_sensitive_text(&path),
        warnings,
        errors,
    })
}

#[tauri::command]
pub fn create_recovery_snapshot(paths: Vec<String>) -> Result<RecoverySnapshotReport, String> {
    if paths.is_empty() {
        return Err("At least one path is required for recovery snapshot.".to_string());
    }

    if paths.len() > MAX_SNAPSHOT_ITEMS {
        return Err("Too many snapshot paths requested.".to_string());
    }

    let root = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("recovery_snapshots");

    let snapshot_dir = root.join(format!("snapshot_{}", now_ms()));
    fs::create_dir_all(&snapshot_dir).map_err(|error| error.to_string())?;

    let mut copied_items = Vec::new();
    let mut skipped_items = Vec::new();

    for path_text in paths {
        if let Err(error) = validate_path_text(&path_text) {
            skipped_items.push(format!("{}: {}", redact_sensitive_text(&path_text), error));
            continue;
        }

        let path = PathBuf::from(path_text.trim());

        if !path.exists() {
            skipped_items.push(format!("{}: not found", path.display()));
            continue;
        }

        if is_sensitive_system_root(&path) {
            skipped_items.push(format!("{}: sensitive system root skipped", path.display()));
            continue;
        }

        let Some(file_name) = path.file_name() else {
            skipped_items.push(format!("{}: missing file name", path.display()));
            continue;
        };

        let destination = snapshot_dir.join(file_name);

        let result = if path.is_dir() {
            copy_dir_recursive(&path, &destination)
        } else {
            copy_file_checked(&path, &destination)
        };

        match result {
            Ok(()) => copied_items.push(path.display().to_string()),
            Err(error) => skipped_items.push(format!("{}: {}", path.display(), error)),
        }
    }

    Ok(RecoverySnapshotReport {
        snapshot_dir: snapshot_dir.display().to_string(),
        copied_items,
        skipped_items,
        message: "Recovery snapshot completed.".to_string(),
    })
}

#[tauri::command]
pub fn security_hardening_report() -> SecurityHardeningReport {
    SecurityHardeningReport {
        process_args_policy: "Typed managed binary plus Vec<String> args. No shell string execution."
            .to_string(),
        redaction_policy:
            "Tokens, secrets, API keys, passwords, auth headers, and URL secret parameters are redacted."
                .to_string(),
        path_policy: "Paths reject null bytes, quotes, CR/LF, and shell metacharacters."
            .to_string(),
        recovery_policy:
            "Recovery snapshots copy selected files/directories into the app data recovery folder before risky operations."
                .to_string(),
        status: "Security hardening baseline active.".to_string(),
        checks: vec![
            "Reject shell chaining tokens: &&, ||, ;, |".to_string(),
            "Reject null bytes and newlines in process args.".to_string(),
            "Block script hosts and shell launchers as managed binaries.".to_string(),
            "Redact token/password/secret/api_key/auth query parameters.".to_string(),
            "Limit recovery copy size per file.".to_string(),
            "Use repository writes for DB changes.".to_string(),
        ],
    }
}

fn redact_sensitive_text(input: &str) -> String {
    let mut output = input.to_string();

    for key in [
        "token",
        "api_key",
        "apikey",
        "secret",
        "password",
        "passwd",
        "auth",
        "authorization",
        "access_token",
        "refresh_token",
        "private_key",
    ] {
        output = redact_key_value(&output, key);
    }

    output
}

fn redact_key_value(input: &str, key: &str) -> String {
    let mut output = String::new();

    for (index, part) in input.split('&').enumerate() {
        if index > 0 {
            output.push('&');
        }

        let lower = part.to_ascii_lowercase();

        if lower.starts_with(&format!("{key}="))
            || lower.contains(&format!("?{key}="))
            || lower.contains(&format!("&{key}="))
            || lower.contains(&format!("{key}:"))
        {
            if let Some((prefix, _)) = part.split_once('=') {
                output.push_str(prefix);
                output.push_str("=***");
            } else if let Some((prefix, _)) = part.split_once(':') {
                output.push_str(prefix);
                output.push_str(":***");
            } else {
                output.push_str("***");
            }
        } else {
            output.push_str(part);
        }
    }

    output
}

fn validate_process_target_path(path: &str) -> Result<(), String> {
    validate_path_text(path)?;

    if is_disallowed_launcher(path) {
        return Err(
            "Shell, script host, or script file is not allowed as a program target.".to_string(),
        );
    }

    Ok(())
}

fn validate_path_text(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Path cannot be empty.".to_string());
    }

    if path.contains('\0')
        || path.contains('"')
        || path.contains('\n')
        || path.contains('\r')
        || path.contains("&&")
        || path.contains("||")
        || path.contains('|')
        || path.contains(';')
    {
        return Err("Path contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_safe_arg(arg: &str) -> Result<(), String> {
    if arg.trim().is_empty() {
        return Err("Argument cannot be empty.".to_string());
    }

    if arg.contains('\0')
        || arg.contains('\n')
        || arg.contains('\r')
        || arg.contains("&&")
        || arg.contains("||")
        || arg.contains('|')
        || arg.contains(';')
    {
        return Err(format!("Unsafe process argument rejected: {arg}"));
    }

    Ok(())
}

fn is_disallowed_launcher(path: &str) -> bool {
    let lower = path.trim().to_ascii_lowercase();

    lower.ends_with("cmd.exe")
        || lower.ends_with("powershell.exe")
        || lower.ends_with("pwsh.exe")
        || lower.ends_with("wscript.exe")
        || lower.ends_with("cscript.exe")
        || lower.ends_with("mshta.exe")
        || lower.ends_with("rundll32.exe")
        || lower.ends_with(".bat")
        || lower.ends_with(".cmd")
        || lower.ends_with(".ps1")
        || lower.ends_with(".vbs")
        || lower.ends_with(".js")
}

fn is_sensitive_system_root(path: &Path) -> bool {
    let text = path
        .display()
        .to_string()
        .replace('\\', "/")
        .to_ascii_lowercase();

    matches!(
        text.as_str(),
        "c:/"
            | "c:/windows"
            | "c:/windows/system32"
            | "c:/program files"
            | "c:/program files (x86)"
    )
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;

    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if should_skip_copy_path(&source_path) {
            continue;
        }

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            copy_file_checked(&source_path, &destination_path)?;
        }
    }

    Ok(())
}

fn copy_file_checked(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;

    if metadata.len() > MAX_COPY_BYTES_PER_FILE {
        return Err("File is too large for recovery snapshot.".to_string());
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    fs::copy(source, destination)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn should_skip_copy_path(path: &Path) -> bool {
    let text = path
        .display()
        .to_string()
        .replace('\\', "/")
        .to_ascii_lowercase();

    text.contains("/target/")
        || text.contains("/node_modules/")
        || text.contains("/.git/")
        || text.contains("/recovery_snapshots/")
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_common_secret_keys() {
        assert_eq!(security_redact_text("token=abc".to_string()), "token=***");
        assert_eq!(
            security_redact_text("x=1&api_key=abc".to_string()),
            "x=1&api_key=***"
        );
    }

    #[test]
    fn unsafe_args_are_rejected() {
        assert!(validate_safe_arg("--safe").is_ok());
        assert!(validate_safe_arg("x && whoami").is_err());
    }

    #[test]
    fn launchers_are_blocked() {
        assert!(validate_process_target_path("C:\\Windows\\System32\\cmd.exe").is_err());
    }
}
