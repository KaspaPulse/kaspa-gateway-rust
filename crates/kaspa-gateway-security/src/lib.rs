use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use url::Url;
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

const REDACTED_URL_CREDENTIAL: &str = "redacted";
const SENSITIVE_URL_KEYS: [&str; 10] = [
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

fn is_sensitive_url_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    SENSITIVE_URL_KEYS
        .iter()
        .any(|sensitive| key.contains(sensitive))
}

fn redact_url_pairs(value: &str) -> String {
    value
        .split('&')
        .map(|part| match part.split_once('=') {
            Some((key, _)) if is_sensitive_url_key(key) => format!("{key}=***"),
            None if is_sensitive_url_key(part) => "***".to_string(),
            _ => part.to_string(),
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn redact_url_authority_fallback(input: &str) -> String {
    let Some(scheme_end) = input.find("://") else {
        return input.to_string();
    };
    let authority_start = scheme_end + 3;
    let authority_end = input[authority_start..]
        .find(['/', '?', '#'])
        .map(|offset| authority_start + offset)
        .unwrap_or(input.len());
    let authority = &input[authority_start..authority_end];
    let Some(at) = authority.rfind('@') else {
        return input.to_string();
    };
    format!(
        "{}{}@{}{}",
        &input[..authority_start],
        REDACTED_URL_CREDENTIAL,
        &authority[at + 1..],
        &input[authority_end..]
    )
}

fn redact_url_fallback(input: &str) -> String {
    let (before_fragment, fragment) = input
        .split_once('#')
        .map(|(left, right)| (left, Some(right)))
        .unwrap_or((input, None));
    let (base, query) = before_fragment
        .split_once('?')
        .map(|(left, right)| (left, Some(right)))
        .unwrap_or((before_fragment, None));
    let mut output = redact_url_authority_fallback(base);
    if let Some(query) = query {
        output.push('?');
        output.push_str(&redact_url_pairs(query));
    }
    if let Some(fragment) = fragment {
        output.push('#');
        output.push_str(&redact_url_pairs(fragment));
    }
    output
}

pub fn redact_url(input: &str) -> String {
    let Ok(mut parsed) = Url::parse(input) else {
        return redact_url_fallback(input);
    };

    if !parsed.username().is_empty() {
        let _ = parsed.set_username(REDACTED_URL_CREDENTIAL);
    }
    if parsed.password().is_some() {
        let _ = parsed.set_password(Some(REDACTED_URL_CREDENTIAL));
    }
    if let Some(query) = parsed.query() {
        let query = redact_url_pairs(query);
        parsed.set_query(Some(&query));
    }
    if let Some(fragment) = parsed.fragment() {
        let fragment = redact_url_pairs(fragment);
        parsed.set_fragment(Some(&fragment));
    }

    parsed.to_string()
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
    fn url_authority_query_and_fragment_credentials_are_redacted() {
        let cases = [
            "https://user_label:pass_value@example.invalid/path",
            "https://user_label@example.invalid/path",
            "https://user%40label:pass%3Avalue@example.invalid/path",
            "https://[2001:db8::1]/path?token=query_value&safe=ok#password=fragment_value",
            "https://user_label:pass_value@[2001:db8::1]/path?api_key=query_value",
            "https://user_label:pass_value@example.invalid:bad/path?token=query_value",
        ];
        for input in cases {
            let redacted = redact_url(input);
            assert!(!redacted.contains("user_label"));
            assert!(!redacted.contains("pass_value"));
            assert!(!redacted.contains("query_value"));
            assert!(!redacted.contains("fragment_value"));
        }
        let safe = redact_url("https://example.invalid/path?safe=value#section");
        assert!(safe.contains("safe=value"));
        assert!(safe.ends_with("#section"));
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
