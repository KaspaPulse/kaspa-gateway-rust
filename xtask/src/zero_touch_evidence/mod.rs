//! Read-only verification of saved zero-touch evidence. This never runs a desktop,
//! touches the system clipboard, or treats synthetic fixtures as runtime evidence.
mod identity;
mod integrity;
mod reports;
pub mod result;
mod stage;

use serde_json::{Value, json};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub type EvidenceResult<T> = Result<T, String>;
const MAX_EVIDENCE_BYTES: u64 = 32 * 1024 * 1024;

pub fn property<'a>(value: &'a Value, names: &[&str]) -> &'a Value {
    let Some(object) = value.as_object() else {
        return &Value::Null;
    };
    for name in names {
        if let Some((_, found)) = object
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
        {
            return found;
        }
    }
    &Value::Null
}

pub fn text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Number(value) => {
            let raw = value.to_string();
            raw.strip_suffix(".0").unwrap_or(&raw).to_owned()
        }
        Value::Array(items) => items.iter().map(text).collect::<Vec<_>>().join(" "),
        Value::Object(_) => value.to_string(),
    }
}

pub fn field(value: &Value, names: &[&str]) -> String {
    text(property(value, names))
}

pub fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::String(value) => !value.is_empty(),
        Value::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
        Value::Array(items) => match items.len() {
            0 => false,
            1 => truthy(&items[0]),
            _ => true,
        },
        Value::Object(_) => true,
    }
}

pub fn sequence(value: &Value) -> Vec<&Value> {
    match value {
        Value::Null => Vec::new(),
        Value::Array(items) => items.iter().collect(),
        _ => vec![value],
    }
}

pub fn integer(value: &Value) -> EvidenceResult<Option<i64>> {
    if value.is_null() {
        return Ok(None);
    }
    if let Some(value) = value.as_i64() {
        return Ok(Some(value));
    }
    let raw = text(value);
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(None);
    }
    raw.parse::<i64>()
        .map(Some)
        .map_err(|_| "Evidence integer exceeds signed 64-bit range".to_owned())
}

pub fn decimal_i32(value: &Value) -> EvidenceResult<Option<i32>> {
    let raw = text(value);
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(None);
    }
    raw.parse::<i32>()
        .map(Some)
        .map_err(|_| "Evidence integer exceeds signed 32-bit range".to_owned())
}

pub fn absolute(path: &Path) -> EvidenceResult<PathBuf> {
    std::path::absolute(path).map_err(|error| format!("Cannot resolve {}: {error}", path.display()))
}

pub fn read_text(path: &Path) -> EvidenceResult<String> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Cannot inspect {}: {error}", path.display()))?;
    if metadata.len() > MAX_EVIDENCE_BYTES {
        return Err(format!(
            "Evidence file exceeds {MAX_EVIDENCE_BYTES} bytes: {}",
            path.display()
        ));
    }
    let file =
        fs::File::open(path).map_err(|error| format!("Cannot open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_EVIDENCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read {}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_EVIDENCE_BYTES {
        return Err(format!(
            "Evidence file exceeds {MAX_EVIDENCE_BYTES} bytes: {}",
            path.display()
        ));
    }
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        if bytes.len() % 2 != 0 {
            return Err(format!("Invalid UTF-16 byte length in {}", path.display()));
        }
        let little = bytes[0] == 0xff;
        let words = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                if little {
                    u16::from_le_bytes([pair[0], pair[1]])
                } else {
                    u16::from_be_bytes([pair[0], pair[1]])
                }
            })
            .collect::<Vec<_>>();
        return String::from_utf16(&words)
            .map_err(|error| format!("Invalid UTF-16 in {}: {error}", path.display()));
    }
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes);
    String::from_utf8(bytes.to_vec())
        .map_err(|error| format!("Invalid UTF-8 in {}: {error}", path.display()))
}

pub fn read_json(path: &Path) -> EvidenceResult<Value> {
    if !path.is_file() {
        return Ok(Value::Null);
    }
    let text = read_text(path)?;
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&text)
        .map_err(|error| format!("Invalid JSON in {}: {error}", path.display()))
}

pub fn append_errors(destination: &mut Vec<String>, report: &Value, key: &str) {
    destination.extend(sequence(property(report, &[key])).into_iter().map(text));
}

pub fn dispatch(
    command: &str,
    repository: &Path,
    artifact: &Path,
    selector: Option<&str>,
) -> EvidenceResult<Value> {
    match command {
        "stages" => Ok(Value::Array(
            stage::required_stages()?
                .iter()
                .map(stage::Stage::to_json)
                .collect(),
        )),
        "stage" => {
            let stages = stage::required_stages()?;
            let stage = stages
                .iter()
                .find(|item| Some(item.slug) == selector)
                .ok_or_else(|| "stage requires a known --stage slug".to_owned())?;
            stage::stage_evidence(&absolute(artifact)?, stage)
        }
        "summary" => stage::summary(&absolute(artifact)?),
        "wdio" => reports::wdio(&absolute(artifact)?),
        "policy" => reports::policy(&absolute(artifact)?),
        "integrity" => integrity::check(&absolute(repository)?, &absolute(artifact)?),
        "source-hash" => Ok(
            json!({"git_commit": identity::commit(repository)?, "source_diff_sha256": identity::source_diff(repository)?}),
        ),
        "recovery-files" => reports::recovery_files(&absolute(artifact)?),
        _ => Err(format!("Unknown evidence command: {command}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn property_aliases_are_case_insensitive_and_first_present_wins() {
        let value = json!({"RuntimeRole": null, "runtime_role": "bridge"});
        assert!(property(&value, &["runtimeRole", "runtime_role"]).is_null());
        assert_eq!(field(&json!({"PROCESSID": 17}), &["ProcessId"]), "17");
    }
    #[test]
    fn typed_integer_and_string_integer_behavior_is_preserved() {
        assert_eq!(integer(&json!(-7)).unwrap(), Some(-7));
        assert_eq!(integer(&json!("-7")).unwrap(), None);
        assert_eq!(integer(&json!("0007")).unwrap(), Some(7));
        assert_eq!(integer(&Value::Null).unwrap(), None);
        assert!(integer(&json!("9223372036854775808")).is_err());
    }
    #[test]
    fn powershell_boolean_coercion_is_not_js_coercion() {
        assert!(truthy(&json!("false")));
        assert!(!truthy(&json!([])));
        assert!(!truthy(&json!([false])));
        assert!(truthy(&json!([false, false])));
    }
}

#[cfg(test)]
mod regression_tests;
