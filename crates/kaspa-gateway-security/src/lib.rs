use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("unsafe process argument rejected: {0}")]
    UnsafeProcessArgument(String),

    #[error("unsafe path rejected: {0}")]
    UnsafePath(String),

    #[error("unsafe URL rejected: {0}")]
    UnsafeUrl(String),

    #[error("unsafe log value rejected")]
    UnsafeLogValue,
}

#[derive(Debug, Clone, Zeroize)]
#[zeroize(drop)]
pub struct RedactedString {
    value: String,
}

impl RedactedString {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn masked(&self) -> String {
        mask_sensitive_value(&self.value)
    }
}

pub fn mask_sensitive_value(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.is_empty() || trimmed.chars().count() <= 10 {
        return "***".to_string();
    }

    let chars = trimmed.chars().collect::<Vec<_>>();
    let start = chars.iter().take(6).collect::<String>();
    let end = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();

    format!("{start}...{end}")
}

pub fn mask_address(input: &str) -> String {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.chars().count() <= 18 {
        return trimmed.to_string();
    }

    let chars = trimmed.chars().collect::<Vec<_>>();
    let start = chars.iter().take(14).collect::<String>();
    let end = chars
        .iter()
        .rev()
        .take(8)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();

    format!("{start}...{end}")
}

pub fn redact_url(input: &str) -> String {
    let sensitive_keys = [
        "key",
        "apikey",
        "api_key",
        "token",
        "secret",
        "auth",
        "password",
        "signature",
        "private",
        "pin",
    ];

    let Some((base, query)) = input.split_once('?') else {
        return input.to_string();
    };

    let redacted_query = query
        .split('&')
        .map(|part| {
            let Some((key, _value)) = part.split_once('=') else {
                return part.to_string();
            };

            let key_lower = key.to_ascii_lowercase();
            let is_sensitive = sensitive_keys
                .iter()
                .any(|sensitive| key_lower.contains(sensitive));

            if is_sensitive {
                format!("{key}=***")
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("&");

    format!("{base}?{redacted_query}")
}

pub fn sanitize_for_logging(value: impl ToString) -> String {
    let mut text = value.to_string();

    if text.len() > 4_096 {
        text.truncate(4_096);
        text.push_str("...[truncated]");
    }

    text = text.replace('\0', "");
    text = text.replace('\n', "\\n");
    text = text.replace('\r', "\\r");
    text = text.replace('\t', "\\t");

    redact_inline_secrets(&text)
}

pub fn redact_inline_secrets(input: &str) -> String {
    let mut output = Vec::new();

    for part in input.split_whitespace() {
        let lower = part.to_ascii_lowercase();

        if lower.contains("apikey=")
            || lower.contains("api_key=")
            || lower.contains("token=")
            || lower.contains("password=")
            || lower.contains("secret=")
        {
            if let Some((key, _)) = part.split_once('=') {
                output.push(format!("{key}=***"));
            } else {
                output.push("***".to_string());
            }
        } else {
            output.push(part.to_string());
        }
    }

    output.join(" ")
}

pub fn sanitize_csv_cell(value: &str) -> String {
    let mut cleaned = value.replace('\0', "");
    cleaned = cleaned.replace('\r', " ");
    cleaned = cleaned.replace('\n', " ");

    if cleaned.starts_with('=')
        || cleaned.starts_with('+')
        || cleaned.starts_with('-')
        || cleaned.starts_with('@')
    {
        cleaned.insert(0, '\'');
    }

    cleaned
}

pub fn validate_process_arg(arg: &str) -> Result<(), SecurityError> {
    if arg.trim().is_empty() {
        return Err(SecurityError::UnsafeProcessArgument(arg.to_string()));
    }

    if arg.contains('\0')
        || arg.contains('\n')
        || arg.contains('\r')
        || arg.contains("&&")
        || arg.contains("||")
        || arg.contains('|')
        || arg.contains(';')
    {
        return Err(SecurityError::UnsafeProcessArgument(arg.to_string()));
    }

    Ok(())
}

pub fn validate_path_text(value: &str) -> Result<(), SecurityError> {
    if value.trim().is_empty() {
        return Err(SecurityError::UnsafePath("empty path".to_string()));
    }

    if value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err(SecurityError::UnsafePath(value.to_string()));
    }

    Ok(())
}

pub fn validate_host(value: &str) -> Result<(), SecurityError> {
    if value.trim().is_empty()
        || value.contains('\0')
        || value.contains('/')
        || value.contains('\\')
        || value.contains(' ')
        || value.contains('\n')
        || value.contains('\r')
    {
        return Err(SecurityError::UnsafeProcessArgument(value.to_string()));
    }

    Ok(())
}

pub fn validate_url_text(value: &str) -> Result<(), SecurityError> {
    let value = value.trim();

    if !(value.starts_with("https://") || value.starts_with("http://")) {
        return Err(SecurityError::UnsafeUrl(value.to_string()));
    }

    if value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err(SecurityError::UnsafeUrl(value.to_string()));
    }

    Ok(())
}

pub fn is_disallowed_windows_launcher(path: &str) -> bool {
    let lower = path.trim().to_ascii_lowercase();

    lower.ends_with("cmd.exe")
        || lower.ends_with("powershell.exe")
        || lower.ends_with("pwsh.exe")
        || lower.ends_with("wscript.exe")
        || lower.ends_with("cscript.exe")
        || lower.ends_with("mshta.exe")
        || lower.ends_with("rundll32.exe")
}

#[derive(Debug, Clone)]
pub struct SafeProcessArg(String);

impl SafeProcessArg {
    pub fn new(arg: impl Into<String>) -> Result<Self, SecurityError> {
        let arg = arg.into();
        validate_process_arg(&arg)?;
        Ok(Self(arg))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

pub fn expose_secret_for_runtime(secret: &SecretString) -> &str {
    secret.expose_secret()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensitive_values_are_masked() {
        let masked = mask_sensitive_value("abcdef1234567890");
        assert_eq!(masked, "abcdef...7890");
    }

    #[test]
    fn url_tokens_are_redacted() {
        let redacted = redact_url("https://example.com/path?token=secret123&safe=value");

        assert!(redacted.contains("token=***"));
        assert!(redacted.contains("safe=value"));
        assert!(!redacted.contains("secret123"));
    }

    #[test]
    fn process_args_reject_newlines_and_shell_joiners() {
        assert!(validate_process_arg("--rpclisten=127.0.0.1:16110\nbad").is_err());
        assert!(validate_process_arg("--safe=value").is_ok());
        assert!(validate_process_arg("ok && bad").is_err());
    }

    #[test]
    fn csv_cells_are_hardened_against_formula_injection() {
        assert_eq!(sanitize_csv_cell("=cmd"), "'=cmd");
        assert_eq!(sanitize_csv_cell("+SUM(A1:A2)"), "'+SUM(A1:A2)");
        assert_eq!(sanitize_csv_cell("normal"), "normal");
    }

    #[test]
    fn unsafe_launchers_are_rejected_by_detector() {
        assert!(is_disallowed_windows_launcher(
            "C:\\Windows\\System32\\cmd.exe"
        ));
        assert!(is_disallowed_windows_launcher("powershell.exe"));
    }
}
