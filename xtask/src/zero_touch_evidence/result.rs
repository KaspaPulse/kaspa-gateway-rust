//! Construct the existing result protocol from explicit inputs and saved evidence.
//! A result object is a receipt, not independent runtime qualification.
use super::*;
use time::OffsetDateTime;

fn now() -> EvidenceResult<String> {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|error| error.to_string())
}
fn required_text(request: &Value, key: &str) -> EvidenceResult<String> {
    let value = property(request, &[key])
        .as_str()
        .ok_or_else(|| format!("{key} must be a string"))?;
    if value.trim().is_empty() {
        return Err(format!("{key} must not be empty"));
    }
    Ok(value.to_owned())
}
fn exit_code(request: &Value, key: &str) -> EvidenceResult<i32> {
    let value =
        integer(property(request, &[key]))?.ok_or_else(|| format!("{key} must be an integer"))?;
    i32::try_from(value).map_err(|_| format!("{key} is outside the signed32-bit range"))
}
fn string_list(value: &Value, null_is_empty_string: bool) -> Vec<String> {
    if value.is_null() && null_is_empty_string {
        vec![String::new()]
    } else {
        sequence(value).into_iter().map(text).collect()
    }
}
fn nonblank_errors(value: &Value) -> Vec<String> {
    sequence(value)
        .into_iter()
        .map(text)
        .filter(|text| !text.trim().is_empty())
        .collect()
}

pub fn build(request: &Value) -> EvidenceResult<Value> {
    build_at(request, &now()?)
}
fn build_at(request: &Value, completed_at: &str) -> EvidenceResult<Value> {
    let repository = PathBuf::from(required_text(request, "repository")?);
    let artifact = absolute(Path::new(&required_text(request, "artifact_directory")?))?;
    let started = required_text(request, "started_at")?;
    let exit = exit_code(request, "exit_code")?;
    let executable = field(request, &["executable_path"]);
    let executable_sha = if !executable.trim().is_empty() && Path::new(&executable).is_file() {
        json!(identity::hash_file(Path::new(&executable))?)
    } else {
        Value::Null
    };
    let supplied = property(request, &["evidence_summary"]);
    let summary =
        if !supplied.is_null() {
            supplied.clone()
        } else {
            stage::summary(&artifact).unwrap_or_else(|error| json!({
            "passed":false,"validation_errors":[error],"warnings":[],"passed_stages":[],
            "failed_stage":null,"clipboard_hashes":[],"process_ids":[],"ports":[],"stages":[]
        }))
        };
    let requested_failure = field(request, &["failed_stage"]);
    let failure = if requested_failure.trim().is_empty() {
        field(&summary, &["failed_stage"])
    } else {
        requested_failure
    };
    let mut errors = nonblank_errors(property(&summary, &["validation_errors"]));
    errors.extend(nonblank_errors(property(
        request,
        &["additional_validation_errors"],
    )));
    // Preserve the existing builder flag contract. Final reuse is independently
    // checked by the evidence validator; arbitrary supplied summaries are not proof.
    let mut result = json!({
        "completed":true,"success":exit==0 && truthy(property(&summary,&["passed"])),
        "exit_code":exit,"started_at":started,"completed_at":completed_at,
        "git_commit":identity::commit(&repository)?,"source_diff_sha256":identity::source_diff(&repository)?,
        "executable_sha256":executable_sha,
        "passed_stages":string_list(property(&summary,&["passed_stages"]),true),
        "failed_stage":failure,"clipboard_hashes":integrity::normalized_hashes(property(&summary,&["clipboard_hashes"])),
        "process_ids":integrity::normalized_pids(property(&summary,&["process_ids"]))?,
        "ports":integrity::normalized_ports(property(&summary,&["ports"]))?,
        "artifact_directory":artifact,"app_binary":executable,"validation_errors":errors
    });
    if truthy(property(request, &["recovered_from_completed_wdio_run"])) {
        let recovery_time = field(request, &["recovery_timestamp"]);
        result["recovered_from_completed_wdio_run"] = Value::Bool(true);
        result["original_failure_stage"] = json!(field(request, &["original_failure_stage"]));
        result["recovery_timestamp"] = json!(if recovery_time.trim().is_empty() {
            completed_at.to_owned()
        } else {
            recovery_time
        });
        result["recovery_evidence_files"] = json!(string_list(
            property(request, &["recovery_evidence_files"]),
            false
        ));
    }
    Ok(result)
}

pub fn failure(request: &Value) -> EvidenceResult<Value> {
    failure_at(request, &now()?)
}
fn failure_at(request: &Value, completed_at: &str) -> EvidenceResult<Value> {
    let _repository = required_text(request, "repository")?;
    let artifact = absolute(Path::new(&required_text(request, "artifact_directory")?))?;
    let started = required_text(request, "started_at")?;
    let exit = exit_code(request, "original_exit_code")?;
    let source = property(request, &["writer_error"]);
    let exception = if source.is_object() {
        json!({"message":field(source,&["message"]),"type":field(source,&["type"]),"stack":field(source,&["stack"])})
    } else {
        json!({"message":text(source),"type":if source.is_null(){""}else{"RustJsonValue"},"stack":""})
    };
    let mut errors = nonblank_errors(property(request, &["validation_errors"]));
    errors.push(format!(
        "Failed to write full zero-touch result: {}",
        field(&exception, &["message"])
    ));
    let original = field(request, &["original_failed_stage"]);
    Ok(json!({
        "completed":true,"success":false,"exit_code":exit,"original_exit_code":exit,
        "started_at":started,"completed_at":completed_at,"git_commit":null,"source_diff_sha256":null,"executable_sha256":null,
        "passed_stages":[],"failed_stage":"Zero-touch result writing",
        "original_failure_stage":if original.trim().is_empty(){Value::Null}else{json!(original)},
        "result_writer_error":exception,"clipboard_hashes":[],"process_ids":[],"ports":[],
        "artifact_directory":artifact,"app_binary":field(request,&["executable_path"]),"validation_errors":errors
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallback_preserves_original_failure_without_claiming_success() {
        let request = json!({"repository":".","artifact_directory":".","started_at":"2026-09-25T00:00:00Z",
            "original_exit_code":37,"original_failed_stage":"WebdriverIO zero-touch live matrix","executable_path":"",
            "writer_error":{"message":"synthetic error","type":"FixtureError","stack":"fixture stack"},"validation_errors":["old error"," ",null]});
        let result = failure_at(&request, "2026-09-25T00:00:01Z").unwrap();
        assert_eq!(result["success"], false);
        assert_eq!(result["exit_code"], 37);
        assert_eq!(result["original_exit_code"], 37);
        assert_eq!(result["failed_stage"], "Zero-touch result writing");
        assert_eq!(result["result_writer_error"]["type"], "FixtureError");
        assert_eq!(result["validation_errors"].as_array().unwrap().len(), 2);
        assert!(result["git_commit"].is_null());
    }
    #[test]
    fn fallback_blank_original_stage_remains_null() {
        let result=failure_at(&json!({"repository":".","artifact_directory":".","started_at":"x","original_exit_code":0,"original_failed_stage":"  ","writer_error":null}),"now").unwrap();
        assert!(result["original_failure_stage"].is_null());
        assert_eq!(result["success"], false);
        assert_eq!(result["original_exit_code"], 0);
        assert_eq!(result["result_writer_error"]["message"], "");
    }
    #[test]
    fn protocol_rejects_out_of_range_exit_codes_and_missing_required_paths() {
        assert!(exit_code(&json!({"exit":2147483648i64}), "exit").is_err());
        assert!(exit_code(&json!({"exit":"bad"}), "exit").is_err());
        assert!(required_text(&json!({"path":" "}), "path").is_err());
    }
    #[test]
    fn explicit_empty_and_null_stage_lists_keep_protocol_shape() {
        assert_eq!(string_list(&Value::Null, true), vec![String::new()]);
        assert!(string_list(&json!([]), true).is_empty());
        assert_eq!(
            string_list(&json!(["Node", null]), true),
            vec!["Node".to_owned(), String::new()]
        );
    }
}
