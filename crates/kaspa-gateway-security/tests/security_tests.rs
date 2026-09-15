use kaspa_gateway_security::{mask_sensitive_value, redact_url, validate_process_arg};

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
fn url_userinfo_and_sensitive_components_are_redacted() {
    let redacted = redact_url(
        "https://user_label:pass_value@[2001:db8::1]/path?password=query_value#token=fragment_value",
    );
    assert!(!redacted.contains("user_label"));
    assert!(!redacted.contains("pass_value"));
    assert!(!redacted.contains("query_value"));
    assert!(!redacted.contains("fragment_value"));
    assert!(redacted.contains("redacted"));
}

#[test]
fn process_args_reject_newlines() {
    let result = validate_process_arg("--rpclisten=127.0.0.1:16110\nbad");
    assert!(result.is_err());
}
