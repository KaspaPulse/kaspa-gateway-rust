use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const ALLOWED: [(&str, i64, &str); 4] = [
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        4079,
        "clippy::collapsible_if",
    ),
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
        1841,
        "clippy::too_many_arguments",
    ),
    (
        "crates/kaspa-gateway-rk-bridge/src/observation.rs",
        30,
        "dead_code",
    ),
    (
        "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
        3848,
        "dead_code",
    ),
];

#[derive(Debug, Clone, Eq, PartialEq)]
struct Fingerprint {
    path: String,
    line: i64,
    code: String,
}
#[derive(Debug, Default)]
struct Evaluation {
    accepted: usize,
    unexpected: Vec<String>,
}

pub fn check_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| format!("clippy-result-policy: FAIL: {error}"))?;
    let evaluation = evaluate(BufReader::new(file))?;
    if !evaluation.unexpected.is_empty() {
        let mut message = format!(
            "clippy-result-policy: FAIL: {} unexpected diagnostic(s)",
            evaluation.unexpected.len()
        );
        for item in evaluation.unexpected {
            message.push_str("\n  ");
            message.push_str(&item);
        }
        return Err(message);
    }
    Ok(format!(
        "clippy-result-policy: PASS (reviewed diagnostics accepted={}; unexpected=0)",
        evaluation.accepted
    ))
}

fn evaluate<R: BufRead>(reader: R) -> Result<Evaluation, String> {
    let mut evaluation = Evaluation::default();
    for (index, line) in reader.lines().enumerate() {
        let raw = line.map_err(|error| {
            format!("clippy-result-policy: FAIL: failed reading input: {error}")
        })?;
        if raw.trim().is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(&raw).map_err(|error| {
            format!(
                "clippy-result-policy: FAIL: invalid JSON on output line {}: {error}",
                index + 1
            )
        })?;
        if record.get("reason").and_then(Value::as_str) != Some("compiler-message") {
            continue;
        }
        let message = record.get("message").unwrap_or(&Value::Null);
        let level = message
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if level != "warning" && level != "error" {
            continue;
        }

        let fingerprint = primary_fingerprint(message);
        let rendered = message
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let allowed = ALLOWED.iter().any(|(path, line, code)| {
            fingerprint.path == *path && fingerprint.line == *line && fingerprint.code == *code
        });
        if allowed && level == "warning" {
            evaluation.accepted += 1;
        } else {
            evaluation.unexpected.push(format!(
                "path={} line={} code={} message={level}: {rendered}",
                fingerprint.path, fingerprint.line, fingerprint.code
            ));
        }
    }
    Ok(evaluation)
}

fn primary_fingerprint(message: &Value) -> Fingerprint {
    let code = message
        .get("code")
        .and_then(|value| value.get("code"))
        .and_then(Value::as_str)
        .unwrap_or("<no-code>")
        .replace('-', "_");
    let primary = message
        .get("spans")
        .and_then(Value::as_array)
        .and_then(|spans| {
            spans.iter().find(|span| {
                span.get("is_primary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
        });
    let Some(primary) = primary else {
        return Fingerprint {
            path: "<unclassified>".to_owned(),
            line: -1,
            code,
        };
    };

    let raw_path = primary
        .get("file_name")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    Fingerprint {
        path: normalize_posix_path(raw_path),
        line: primary
            .get("line_start")
            .and_then(Value::as_i64)
            .unwrap_or(-1),
        code,
    }
}

fn normalize_posix_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            value => parts.push(value),
        }
    }
    parts.join("/")
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    fn diagnostic(path: &str, line: i64, code: &str, level: &str) -> String {
        json!({
            "reason": "compiler-message",
            "message": {
                "level": level,
                "message": "fixture",
                "code": {"code": code, "explanation": null},
                "spans": [{
                    "file_name": path,
                    "line_start": line,
                    "line_end": line,
                    "column_start": 1,
                    "column_end": 2,
                    "is_primary": true
                }]
            }
        })
        .to_string()
    }

    fn result(lines: &[String]) -> Result<Evaluation, String> {
        let mut input = lines.join("\n");
        input.push('\n');
        evaluate(Cursor::new(input))
    }

    #[test]
    fn accepts_clean_and_exact_reviewed_diagnostics() {
        assert!(result(&[]).unwrap().unexpected.is_empty());
        let cases = [
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
                4079,
                "clippy::collapsible_if",
                "warning",
            ),
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
                1841,
                "clippy::too_many_arguments",
                "warning",
            ),
            diagnostic(
                "crates/kaspa-gateway-rk-bridge/src/observation.rs",
                30,
                "dead_code",
                "warning",
            ),
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
                3848,
                "dead_code",
                "warning",
            ),
        ];
        let evaluation = result(&cases).unwrap();
        assert_eq!(evaluation.accepted, 4);
        assert!(evaluation.unexpected.is_empty());
    }

    #[test]
    fn normalizes_lexical_parent_alias() {
        let line = diagnostic(
            "apps/kaspa-gateway-desktop/src-tauri/tests/../src/integrated_runtime_commands.rs",
            4079,
            "clippy::collapsible_if",
            "warning",
        );
        assert!(result(&[line]).unwrap().unexpected.is_empty());
    }

    #[test]
    fn drift_and_errors_fail_closed() {
        let cases = [
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
                4080,
                "clippy::collapsible_if",
                "warning",
            ),
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/other.rs",
                4079,
                "clippy::collapsible_if",
                "warning",
            ),
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs",
                4079,
                "clippy::needless_borrow",
                "warning",
            ),
            diagnostic(
                "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
                1,
                "E0308",
                "error",
            ),
        ];
        for case in cases {
            assert_eq!(result(&[case]).unwrap().unexpected.len(), 1);
        }
    }

    #[test]
    fn duplicate_exact_reviewed_warning_is_allowed() {
        let line = diagnostic(
            "crates/kaspa-gateway-rk-bridge/src/observation.rs",
            30,
            "dead_code",
            "warning",
        );
        let evaluation = result(&[line.clone(), line]).unwrap();
        assert_eq!(evaluation.accepted, 2);
        assert!(evaluation.unexpected.is_empty());
    }

    #[test]
    fn invalid_json_fails_closed() {
        assert!(evaluate(Cursor::new("not-json\n")).is_err());
    }
}
