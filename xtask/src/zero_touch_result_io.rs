//! Atomic JSON writes for result artifacts, not source-file recovery or overrides.
//! A denied replacement returns an error; there is no in-place fallback.
use crate::zero_touch_evidence::EvidenceResult;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;

const MAX_OUTPUT_BYTES: usize = 32 * 1024 * 1024;
const MAX_DEPTH: usize = 64;

fn validate(value: &Value, path: &str, depth: usize) -> EvidenceResult<()> {
    if depth > MAX_DEPTH {
        return Err(format!("JSON-safe nesting exceeds {MAX_DEPTH} at {path}"));
    }
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => Ok(()),
        Value::Number(number) if number.as_i64().is_some() => Ok(()),
        Value::Number(_) => Err(format!("JSON-safe signed integer expected at {path}")),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                validate(item, &format!("{path}[{index}]"), depth + 1)?;
            }
            Ok(())
        }
        Value::Object(items) => {
            for (key, item) in items {
                validate(item, &format!("{path}.{key}"), depth + 1)?;
            }
            Ok(())
        }
    }
}

pub fn atomic_write(path: &Path, value: &Value) -> EvidenceResult<Value> {
    validate(value, "$", 0)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(format!("Result JSON exceeds {MAX_OUTPUT_BYTES} bytes"));
    }
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if path.file_name().is_none() {
        return Err("Result file name is required".to_owned());
    }
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "Cannot create result directory {}: {error}",
            parent.display()
        )
    })?;
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Result target must be a regular file: {}",
                path.display()
            ));
        }
        if metadata.permissions().readonly() {
            return Err(format!("Result target is read-only: {}", path.display()));
        }
    }
    let mut temporary = tempfile::Builder::new()
        .prefix(".kgw-result-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|error| {
            format!(
                "Cannot create temporary result in {}: {error}",
                parent.display()
            )
        })?;
    temporary
        .write_all(&bytes)
        .map_err(|error| format!("Cannot write temporary result: {error}"))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| format!("Cannot flush temporary result: {error}"))?;
    if let Ok(metadata) = fs::metadata(path) {
        temporary
            .as_file()
            .set_permissions(metadata.permissions())
            .map_err(|error| format!("Cannot preserve result permissions: {error}"))?;
    }
    // This same-directory atomic persist never falls back to truncating the
    // target. The owned temporary file is cleaned if persistence is rejected.
    let file = temporary
        .persist(path)
        .map_err(|error| format!("Atomic result replacement failed: {}", error.error))?;
    file.sync_all()
        .map_err(|error| format!("Result was written but durability flush failed: {error}"))?;
    drop(file);
    let actual = fs::read(path)
        .map_err(|error| format!("Result was written but read-back failed: {error}"))?;
    if actual != bytes {
        return Err("Result changed before read-back verification completed".to_owned());
    }
    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| format!("Result written but parent-directory flush failed: {error}"))?;
    Ok(
        json!({"written":true,"path":path,"sha256":format!("{:x}",Sha256::digest(&actual)),"bytes":actual.len()}),
    )
}

pub fn artifact_target(directory: &Path, name: &str) -> EvidenceResult<std::path::PathBuf> {
    // The command exposes only explicit artifact filenames. It cannot target a
    // source file, parent directory, or arbitrary absolute output path.
    if !matches!(
        name,
        "zero-touch-result.json"
            | "zero-touch-script-summary.json"
            | "zero-touch-script-start.json"
    ) {
        return Err("Unsupported result artifact filename".to_owned());
    }
    Ok(directory.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_artifact_and_replacement_preserve_json_types_without_bom() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("result.json");
        atomic_write(
            &path,
            &json!({"version":1,"pid":9007199254740993i64,"enabled":true}),
        )
        .unwrap();
        let bytes = fs::read(&path).unwrap();
        assert!(!bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["pid"].as_i64(), Some(9007199254740993));
        atomic_write(
            &path,
            &json!({"version":2,"text":"Arabic: \u{0645}\u{0631}\u{062d}\u{0628}\u{0627}"}),
        )
        .unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap()["version"],
            2
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
    #[test]
    fn rejected_float_and_unsigned_overflow_preserve_existing_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("result.json");
        fs::write(&path, b"{\"stable\":true}").unwrap();
        for value in [json!({"bad":1.25}), json!({"bad":u64::MAX})] {
            assert!(atomic_write(&path, &value).is_err());
            assert_eq!(fs::read(&path).unwrap(), b"{\"stable\":true}");
        }
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
    #[test]
    fn null_and_empty_collections_remain_exact_types() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("result.json");
        for value in [
            Value::Null,
            json!([]),
            json!({}),
            json!({"nested":[null,false,0,""]}),
        ] {
            atomic_write(&path, &value).unwrap();
            assert_eq!(
                serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap(),
                value
            );
        }
    }
    #[test]
    fn directory_target_and_traversal_names_are_rejected_without_mutation() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("result.json");
        fs::create_dir(&path).unwrap();
        assert!(atomic_write(&path, &json!({})).is_err());
        assert!(path.is_dir());
        for name in ["../source.txt", "Cargo.toml", "C:\\file.json", "other.json"] {
            assert!(artifact_target(directory.path(), name).is_err());
        }
        assert!(artifact_target(directory.path(), "zero-touch-result.json").is_ok());
    }
    #[test]
    fn excessive_json_nesting_is_rejected_before_existing_file_changes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("result.json");
        fs::write(&path, b"stable").unwrap();
        let mut value = Value::Null;
        for _ in 0..66 {
            value = json!([value]);
        }
        assert!(atomic_write(&path, &value).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"stable");
    }
    #[cfg(windows)]
    #[test]
    fn open_reader_without_delete_sharing_blocks_replacement_without_fallback() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("result.json");
        fs::write(&path, b"stable").unwrap();
        let reader = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&path)
            .unwrap();
        assert!(atomic_write(&path, &json!({"replaced":true})).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"stable");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
        drop(reader);
    }
}
