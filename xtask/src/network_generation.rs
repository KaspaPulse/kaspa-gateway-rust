use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Finding {
    path: String,
    line: usize,
    text: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let findings = scan_repository(root)?;
    if findings.is_empty() {
        return Ok("KGW_NETWORK_GENERATION_GATE_PASSED".to_owned());
    }

    let mut message = String::from("KGW_NETWORK_GENERATION_GATE_FAILED");
    for finding in findings {
        let payload = json!({
            "path": finding.path,
            "line": finding.line,
            "text": finding.text,
        });
        message.push('\n');
        message.push_str(&payload.to_string());
    }
    Err(message)
}

fn git_tracked_files(root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|error| format!("KGW_NETWORK_GENERATION_GATE_GIT_FAILED:{error}"))?;
    if !output.status.success() {
        return Err(format!(
            "KGW_NETWORK_GENERATION_GATE_GIT_FAILED:{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|item| !item.is_empty())
        .map(|item| {
            String::from_utf8(item.to_vec())
                .map_err(|_| "KGW_NETWORK_GENERATION_GATE_NON_UTF8_PATH".to_owned())
        })
        .collect()
}

fn scan_repository(root: &Path) -> Result<Vec<Finding>, String> {
    let mut findings = Vec::new();
    for relative in git_tracked_files(root)? {
        if contains_legacy_network(&relative) {
            findings.push(Finding {
                path: relative.clone(),
                line: 0,
                text: "legacy network token in tracked path".to_owned(),
            });
        }

        let full = root.join(&relative);
        let Ok(buffer) = fs::read(full) else {
            continue;
        };
        if is_binary(&buffer) {
            continue;
        }
        let text = String::from_utf8_lossy(&buffer);
        findings.extend(scan_text(&relative, &text));
    }
    Ok(findings)
}

fn is_binary(buffer: &[u8]) -> bool {
    buffer.contains(&0)
}

fn scan_text(path: &str, text: &str) -> Vec<Finding> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| contains_legacy_network(line))
        .map(|(index, line)| Finding {
            path: path.to_owned(),
            line: index + 1,
            text: line.trim().chars().take(240).collect(),
        })
        .collect()
}

fn contains_legacy_network(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();

    for (index, _) in lower
        .match_indices("testnet")
        .chain(lower.match_indices("tn"))
    {
        let prefix_len = if lower[index..].starts_with("testnet") {
            "testnet".len()
        } else {
            "tn".len()
        };
        let mut cursor = index + prefix_len;
        if cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'_' | b'-') {
            cursor += 1;
        }
        if bytes.get(cursor) == Some(&b'1') && bytes.get(cursor + 1) == Some(&b'2') {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_forms() -> Vec<String> {
        let generation = 12;
        [
            format!("testnet{generation}"),
            format!("testnet-{generation}"),
            format!("testnet {generation}"),
            format!("testnet_{generation}"),
            format!("tn{generation}"),
            format!("tn-{generation}"),
            format!("tn {generation}"),
            format!("tn_{generation}"),
        ]
        .into()
    }

    #[test]
    fn rejects_all_legacy_forms_case_insensitively() {
        for token in legacy_forms() {
            assert!(contains_legacy_network(&token), "{token}");
            assert!(
                contains_legacy_network(&token.to_ascii_uppercase()),
                "{token}"
            );
            let findings = scan_text("fixture.txt", &format!("prefix {token} suffix"));
            assert_eq!(findings.len(), 1, "{token}");
            assert_eq!(findings[0].line, 1);
        }
    }

    #[test]
    fn accepts_current_network_forms() {
        for current in ["mainnet", "testnet10", "testnet13", "tn13", "testnet-13"] {
            assert!(!contains_legacy_network(current), "{current}");
            assert!(scan_text("fixture.txt", current).is_empty(), "{current}");
        }
    }

    #[test]
    fn reports_one_finding_per_matching_line() {
        let legacy = format!("tn{}", 12);
        let text = format!("ok\n  first {legacy} hit  \nagain {legacy} and {legacy}\n");
        let findings = scan_text("fixture.txt", &text);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].line, 2);
        assert_eq!(findings[0].text, format!("first {legacy} hit"));
        assert_eq!(findings[1].line, 3);
    }

    #[test]
    fn finding_text_is_trimmed_and_capped() {
        let legacy = format!("testnet-{}", 12);
        let long = format!("  {legacy} {}  ", "x".repeat(400));
        let finding = scan_text("fixture.txt", &long).remove(0);
        assert_eq!(finding.text.chars().count(), 240);
        assert!(finding.text.starts_with(&legacy));
    }

    #[test]
    fn nul_byte_marks_binary_content() {
        assert!(is_binary(b"text\0binary"));
        assert!(!is_binary(b"plain text"));
    }

    #[test]
    fn preserves_legacy_regex_prefix_semantics() {
        let generation = 12;
        assert!(contains_legacy_network(&format!("testnet{generation}0")));
        assert!(contains_legacy_network(&format!("tn {generation}0")));
        assert!(!contains_legacy_network("tn 21"));
    }
}
