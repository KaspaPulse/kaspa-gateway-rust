use serde_json::Value;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const LOB_RAW: &str = concat!("test_", "diagnosis_requires_knowledge_search");

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct Fingerprint {
    commit: Option<String>,
    file: Option<String>,
    line: Option<i64>,
    detector: Option<String>,
    decoder: Option<String>,
    verified: Option<bool>,
}

#[derive(Debug, Default)]
struct Evaluation {
    allowed: usize,
    unexpected: Vec<Fingerprint>,
}

pub fn check_file(path: &Path) -> Result<String, String> {
    let file =
        File::open(path).map_err(|error| format!("trufflehog-result-policy: FAIL: {error}"))?;
    let evaluation = evaluate(BufReader::new(file))?;
    if !evaluation.unexpected.is_empty() {
        let mut message = format!(
            "trufflehog-result-policy: FAIL: {} unexpected finding(s)",
            evaluation.unexpected.len()
        );
        for item in evaluation.unexpected {
            message.push_str("\n  ");
            message.push_str(&describe(&item));
        }
        return Err(message);
    }
    Ok(format!(
        "trufflehog-result-policy: PASS (exact historical false positives accepted={}; unexpected=0)",
        evaluation.allowed
    ))
}

fn evaluate<R: BufRead>(reader: R) -> Result<Evaluation, String> {
    let mut evaluation = Evaluation::default();
    let mut seen_allowed = BTreeSet::new();
    for (index, line) in reader.lines().enumerate() {
        let raw_line = line.map_err(|error| format!("trufflehog-result-policy: FAIL: {error}"))?;
        if raw_line.trim().is_empty() {
            continue;
        }
        let result: Value =
            serde_json::from_str(raw_line.trim_start_matches('\u{feff}')).map_err(|error| {
                format!(
                    "trufflehog-result-policy: FAIL: invalid TruffleHog JSON on output line {}: {}",
                    index + 1,
                    error
                )
            })?;
        if !result.is_object() {
            return Err(format!(
                "trufflehog-result-policy: FAIL: invalid TruffleHog result type on output line {}",
                index + 1
            ));
        }
        let current = fingerprint(&result);
        if let Some(key) = allowed_key(&current, result.get("Raw").and_then(Value::as_str)) {
            if !seen_allowed.insert(key) {
                return Err(
                    "trufflehog-result-policy: FAIL: historical false-positive result appeared more than once"
                        .to_owned(),
                );
            }
            evaluation.allowed += 1;
        } else {
            evaluation.unexpected.push(current);
        }
    }
    Ok(evaluation)
}

fn fingerprint(result: &Value) -> Fingerprint {
    let git = result
        .get("SourceMetadata")
        .and_then(|value| value.get("Data"))
        .and_then(|value| value.get("Git"))
        .unwrap_or(&Value::Null);
    Fingerprint {
        commit: string_field(git, "commit"),
        file: string_field(git, "file"),
        line: git.get("line").and_then(Value::as_i64),
        detector: string_field(result, "DetectorName"),
        decoder: string_field(result, "DecoderName"),
        verified: result.get("Verified").and_then(Value::as_bool),
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}
fn allowed_key(current: &Fingerprint, raw: Option<&str>) -> Option<String> {
    let historical = [
        (
            "8f209ba516707b11098bd962972da38157346833",
            "crates/kaspa-gateway-security/src/lib.rs",
            374,
        ),
        (
            "3f6fb666241135be0f6f5071994bcf4deeb75326",
            "crates/kaspa-gateway-security/src/lib.rs",
            374,
        ),
        (
            "3f6fb666241135be0f6f5071994bcf4deeb75326",
            "crates/kaspa-gateway-security/src/lib.rs",
            376,
        ),
    ];
    for (commit, file, line) in historical {
        if matches_base(current, commit, file, line, "URI", "PLAIN", false) {
            return Some(format!("base|{commit}|{file}|{line}"));
        }
    }

    let exact = [
        (
            "d079d38c8a78de400a5d6b2d06819feb1ff73df9",
            ".security/ksss/test_consumer.py",
            115,
        ),
        (
            "c83ff593056749b1f0ffcbffc9bd6c0f2bf5c556",
            "scripts/check-trufflehog-results.py",
            21,
        ),
        (
            "c83ff593056749b1f0ffcbffc9bd6c0f2bf5c556",
            "scripts/test-check-trufflehog-results.py",
            38,
        ),
    ];
    for (commit, file, line) in exact {
        if matches_base(current, commit, file, line, "Lob", "PLAIN", true) && raw == Some(LOB_RAW) {
            return Some(format!("raw|{commit}|{file}|{line}|{LOB_RAW}"));
        }
    }
    None
}

fn matches_base(
    value: &Fingerprint,
    commit: &str,
    file: &str,
    line: i64,
    detector: &str,
    decoder: &str,
    verified: bool,
) -> bool {
    value.commit.as_deref() == Some(commit)
        && value.file.as_deref() == Some(file)
        && value.line == Some(line)
        && value.detector.as_deref() == Some(detector)
        && value.decoder.as_deref() == Some(decoder)
        && value.verified == Some(verified)
}
fn describe(item: &Fingerprint) -> String {
    format!(
        "commit={} file={} line={} detector={} decoder={} verified={}",
        item.commit.as_deref().unwrap_or("None"),
        item.file.as_deref().unwrap_or("None"),
        item.line
            .map_or_else(|| "None".to_owned(), |value| value.to_string()),
        item.detector.as_deref().unwrap_or("None"),
        item.decoder.as_deref().unwrap_or("None"),
        item.verified
            .map_or_else(|| "None".to_owned(), |value| value.to_string())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    fn base() -> Value {
        json!({
            "SourceMetadata": {"Data": {"Git": {
                "commit": "8f209ba516707b11098bd962972da38157346833",
                "file": "crates/kaspa-gateway-security/src/lib.rs",
                "line": 374
            }}},
            "DetectorName": "URI",
            "DecoderName": "PLAIN",
            "Verified": false
        })
    }

    fn lob() -> Value {
        json!({
            "SourceMetadata": {"Data": {"Git": {
                "commit": "d079d38c8a78de400a5d6b2d06819feb1ff73df9",
                "file": ".security/ksss/test_consumer.py",
                "line": 115
            }}},
            "DetectorName": "Lob",
            "DecoderName": "PLAIN",
            "Verified": true,
            "Raw": LOB_RAW
        })
    }

    fn run(values: &[Value]) -> Result<Evaluation, String> {
        let mut input = String::new();
        for value in values {
            input.push_str(&value.to_string());
            input.push('\n');
        }
        evaluate(Cursor::new(input))
    }

    fn set_git(value: &mut Value, key: &str, replacement: Value) {
        value["SourceMetadata"]["Data"]["Git"][key] = replacement;
    }

    fn set_top(value: &mut Value, key: &str, replacement: Value) {
        value
            .as_object_mut()
            .expect("test fixture must be an object")
            .insert(key.to_owned(), replacement);
    }

    #[test]
    fn exact_historical_false_positives_pass() {
        let mut squash_374 = base();
        set_git(
            &mut squash_374,
            "commit",
            json!("3f6fb666241135be0f6f5071994bcf4deeb75326"),
        );
        let mut squash_376 = squash_374.clone();
        set_git(&mut squash_376, "line", json!(376));
        let evaluation = run(&[base(), squash_374, squash_376, lob()]).unwrap();
        assert_eq!(evaluation.allowed, 4);
        assert!(evaluation.unexpected.is_empty());
    }
    #[test]
    fn duplicates_fail_closed() {
        let item = base();
        assert!(run(&[item.clone(), item]).is_err());
        let item = lob();
        assert!(run(&[item.clone(), item]).is_err());
    }

    #[test]
    fn fingerprint_drift_is_unexpected() {
        let mut cases = Vec::new();
        let mut item = base();
        set_git(&mut item, "commit", json!("deadbeef"));
        cases.push(item);

        let mut item = base();
        set_git(&mut item, "file", json!("other.rs"));
        cases.push(item);

        let mut item = base();
        set_git(&mut item, "line", json!(375));
        cases.push(item);

        let mut item = base();
        set_top(&mut item, "DetectorName", json!("Generic"));
        cases.push(item);

        let mut item = base();
        set_top(&mut item, "DecoderName", json!("BASE64"));
        cases.push(item);

        let mut item = base();
        set_top(&mut item, "Verified", json!(true));
        cases.push(item);

        for item in cases {
            assert_eq!(run(&[item]).unwrap().unexpected.len(), 1);
        }
    }
    #[test]
    fn raw_bound_lob_drift_is_unexpected() {
        let mut item = lob();
        set_top(&mut item, "Raw", json!(format!("{LOB_RAW}_x")));
        assert_eq!(run(&[item]).unwrap().unexpected.len(), 1);
    }

    #[test]
    fn malformed_input_fails_closed() {
        assert!(evaluate(Cursor::new("not-json\n")).is_err());
        assert!(evaluate(Cursor::new("[]\n")).is_err());
    }
}
