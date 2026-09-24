use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
};

#[derive(Debug, Clone, Eq, PartialEq)]
struct Options {
    action: String,
    process_id: u32,
    expected_executable: String,
    expected_start_time: i64,
    output_path: PathBuf,
    timeout_seconds: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Identity {
    executable: String,
    start_time: i64,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    let options = parse_options(args)?;
    run(&options)
}

fn parse_options(args: &mut impl Iterator<Item = String>) -> Result<Options, String> {
    let action = args
        .next()
        .ok_or_else(|| "e2e-owned-process requires kill or wait".to_owned())?;
    if !matches!(action.as_str(), "kill" | "wait") {
        return Err(format!("unknown e2e-owned-process action: {action}"));
    }

    let mut process_id = None;
    let mut expected_executable = None;
    let mut expected_start_time = None;
    let mut output_path = None;
    let mut timeout_seconds = 45_u32;

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--process-id" => {
                process_id = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| "--process-id must be a positive integer".to_owned())?,
                );
            }
            "--expected-executable" => expected_executable = Some(value),
            "--expected-start-time" => {
                expected_start_time = Some(
                    value
                        .parse::<i64>()
                        .map_err(|_| "--expected-start-time must be an integer".to_owned())?,
                );
            }
            "--output-path" => output_path = Some(PathBuf::from(value)),
            "--timeout-seconds" => {
                timeout_seconds = value
                    .parse::<u32>()
                    .map_err(|_| "--timeout-seconds must be a positive integer".to_owned())?;
                if timeout_seconds == 0 {
                    return Err("--timeout-seconds must be a positive integer".to_owned());
                }
            }
            _ => return Err(format!("unknown e2e-owned-process option: {flag}")),
        }
    }

    let process_id = process_id.ok_or_else(|| "--process-id is required".to_owned())?;
    if process_id == 0 {
        return Err("--process-id must be a positive integer".to_owned());
    }

    Ok(Options {
        action,
        process_id,
        expected_executable: expected_executable
            .ok_or_else(|| "--expected-executable is required".to_owned())?,
        expected_start_time: expected_start_time
            .ok_or_else(|| "--expected-start-time is required".to_owned())?,
        output_path: output_path.ok_or_else(|| "--output-path is required".to_owned())?,
        timeout_seconds,
    })
}

fn normalize_executable(value: &str) -> String {
    let mut text = value.trim();
    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        text = stripped;
    }
    text.trim_end_matches(['\\', '/']).to_ascii_lowercase()
}

fn same_identity(actual: &Identity, options: &Options) -> bool {
    normalize_executable(&actual.executable) == normalize_executable(&options.expected_executable)
        && (actual.start_time - options.expected_start_time).abs() <= 2
}

fn observed_at() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("failed to format observed_at: {error}"))
}

fn write_evidence(path: &Path, value: &Value) -> Result<String, String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|error| format!("evidence JSON serialization failed: {error}"))?;
    fs::write(path, rendered.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(rendered)
}

fn run(options: &Options) -> Result<String, String> {
    #[cfg(windows)]
    {
        match options.action.as_str() {
            "kill" => kill_exact_owned_process(options),
            "wait" => wait_exact_process_exit(options),
            _ => unreachable!(),
        }
    }

    #[cfg(not(windows))]
    {
        let _ = options;
        Err("e2e-owned-process requires Windows".to_owned())
    }
}

#[cfg(windows)]
struct OwnedHandle(HANDLE);

#[cfg(windows)]
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
fn open_process(process_id: u32, access: u32) -> Result<Option<OwnedHandle>, String> {
    let handle = unsafe { OpenProcess(access, 0, process_id) };
    if handle.is_null() {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(87) {
            return Ok(None);
        }
        return Err(format!("failed to open process {process_id}: {error}"));
    }
    Ok(Some(OwnedHandle(handle)))
}

#[cfg(windows)]
fn query_identity(handle: &OwnedHandle) -> Result<Identity, String> {
    let mut size = 32_768_u32;
    let mut buffer = vec![0_u16; size as usize];
    let ok = unsafe { QueryFullProcessImageNameW(handle.0, 0, buffer.as_mut_ptr(), &mut size) };
    if ok == 0 {
        return Err(format!(
            "failed to query process executable: {}",
            std::io::Error::last_os_error()
        ));
    }
    let executable = String::from_utf16(&buffer[..size as usize])
        .map_err(|error| format!("process executable is not valid UTF-16: {error}"))?;

    let mut creation = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut exit = creation;
    let mut kernel = creation;
    let mut user = creation;
    let ok = unsafe { GetProcessTimes(handle.0, &mut creation, &mut exit, &mut kernel, &mut user) };
    if ok == 0 {
        return Err(format!(
            "failed to query process start time: {}",
            std::io::Error::last_os_error()
        ));
    }

    Ok(Identity {
        executable,
        start_time: filetime_to_unix_seconds(creation)?,
    })
}

#[cfg(windows)]
fn filetime_to_unix_seconds(value: FILETIME) -> Result<i64, String> {
    const TICKS_PER_SECOND: u64 = 10_000_000;
    const WINDOWS_TO_UNIX_EPOCH_SECONDS: u64 = 11_644_473_600;
    let ticks = ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64;
    let seconds = ticks / TICKS_PER_SECOND;
    if seconds < WINDOWS_TO_UNIX_EPOCH_SECONDS {
        return Err("process start time predates Unix epoch".to_owned());
    }
    Ok((seconds - WINDOWS_TO_UNIX_EPOCH_SECONDS) as i64)
}

#[cfg(windows)]
fn query_process_optional(process_id: u32) -> Result<Option<Identity>, String> {
    let Some(handle) = open_process(
        process_id,
        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
    )?
    else {
        return Ok(None);
    };
    query_identity(&handle).map(Some)
}

#[cfg(windows)]
fn kill_exact_owned_process(options: &Options) -> Result<String, String> {
    let Some(handle) = open_process(
        options.process_id,
        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE,
    )?
    else {
        return Err(format!(
            "owned process {} is not running",
            options.process_id
        ));
    };

    let actual = query_identity(&handle)?;
    if normalize_executable(&actual.executable)
        != normalize_executable(&options.expected_executable)
    {
        return Err(format!(
            "owned process executable mismatch for PID {}",
            options.process_id
        ));
    }
    if (actual.start_time - options.expected_start_time).abs() > 2 {
        return Err(format!(
            "owned process start-time mismatch for PID {}",
            options.process_id
        ));
    }

    let mut evidence = json!({
        "process_id": options.process_id,
        "expected_executable": options.expected_executable,
        "actual_executable": actual.executable,
        "expected_start_time": options.expected_start_time,
        "actual_start_time": actual.start_time,
        "killed": false,
        "observed_at": observed_at()?,
    });

    let ok = unsafe { TerminateProcess(handle.0, 1) };
    if ok == 0 {
        return Err(format!(
            "failed to force-kill owned process {}: {}",
            options.process_id,
            std::io::Error::last_os_error()
        ));
    }
    let wait = unsafe { WaitForSingleObject(handle.0, 15_000) };
    if wait == WAIT_TIMEOUT {
        return Err(format!(
            "owned process {} did not terminate after force kill",
            options.process_id
        ));
    }
    if wait != WAIT_OBJECT_0 {
        return Err(format!(
            "failed while waiting for owned process {} termination: wait status {}",
            options.process_id, wait
        ));
    }

    evidence["killed"] = Value::Bool(true);
    evidence["observed_at"] = Value::String(observed_at()?);
    write_evidence(&options.output_path, &evidence)
}

#[cfg(windows)]
fn wait_exact_process_exit(options: &Options) -> Result<String, String> {
    let mut evidence = Map::new();
    evidence.insert("process_id".to_owned(), json!(options.process_id));
    evidence.insert(
        "expected_executable".to_owned(),
        json!(options.expected_executable),
    );
    evidence.insert(
        "expected_start_time".to_owned(),
        json!(options.expected_start_time),
    );
    evidence.insert("exact_identity_exited".to_owned(), Value::Bool(false));
    evidence.insert("pid_reused".to_owned(), Value::Bool(false));
    evidence.insert("observed_at".to_owned(), Value::Null);

    let handle = open_process(
        options.process_id,
        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
    )?;

    if let Some(handle) = handle {
        let actual = query_identity(&handle)?;
        if !same_identity(&actual, options) {
            evidence.insert("exact_identity_exited".to_owned(), Value::Bool(true));
            evidence.insert("pid_reused".to_owned(), Value::Bool(true));
            evidence.insert(
                "replacement_executable".to_owned(),
                json!(actual.executable),
            );
            evidence.insert(
                "replacement_start_time".to_owned(),
                json!(actual.start_time),
            );
        } else {
            let timeout_ms = options.timeout_seconds.saturating_mul(1000);
            let wait = unsafe { WaitForSingleObject(handle.0, timeout_ms) };
            if wait == WAIT_TIMEOUT {
                return Err(format!(
                    "exact process identity {} did not exit within {} seconds",
                    options.process_id, options.timeout_seconds
                ));
            }
            if wait != WAIT_OBJECT_0 {
                return Err(format!(
                    "failed while waiting for exact process identity {}: wait status {}",
                    options.process_id, wait
                ));
            }
            evidence.insert("exact_identity_exited".to_owned(), Value::Bool(true));

            if let Some(replacement) = query_process_optional(options.process_id)?
                && !same_identity(&replacement, options)
            {
                evidence.insert("pid_reused".to_owned(), Value::Bool(true));
                evidence.insert(
                    "replacement_executable".to_owned(),
                    json!(replacement.executable),
                );
                evidence.insert(
                    "replacement_start_time".to_owned(),
                    json!(replacement.start_time),
                );
            }
        }
    } else {
        evidence.insert("exact_identity_exited".to_owned(), Value::Bool(true));
    }

    evidence.insert("observed_at".to_owned(), Value::String(observed_at()?));
    write_evidence(&options.output_path, &Value::Object(evidence))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_options() -> Options {
        Options {
            action: "kill".to_owned(),
            process_id: 4242,
            expected_executable: r"C:\Apps\Kaspa\gateway.exe".to_owned(),
            expected_start_time: 1_700_000_000,
            output_path: PathBuf::from("evidence.json"),
            timeout_seconds: 45,
        }
    }

    #[test]
    fn executable_normalization_matches_legacy_contract() {
        assert_eq!(
            normalize_executable(r"\\?\C:\Apps\Kaspa\gateway.exe\"),
            r"c:\apps\kaspa\gateway.exe"
        );
        assert_eq!(
            normalize_executable(r" C:\APPS\KASPA\GATEWAY.EXE "),
            r"c:\apps\kaspa\gateway.exe"
        );
    }

    #[test]
    fn start_time_tolerance_is_two_seconds() {
        let options = base_options();
        for delta in [-2_i64, -1, 0, 1, 2] {
            assert!(same_identity(
                &Identity {
                    executable: options.expected_executable.clone(),
                    start_time: options.expected_start_time + delta,
                },
                &options
            ));
        }
        assert!(!same_identity(
            &Identity {
                executable: options.expected_executable.clone(),
                start_time: options.expected_start_time + 3,
            },
            &options
        ));
    }

    #[test]
    fn executable_mismatch_rejects_identity() {
        let options = base_options();
        assert!(!same_identity(
            &Identity {
                executable: r"C:\Other\gateway.exe".to_owned(),
                start_time: options.expected_start_time,
            },
            &options
        ));
    }

    #[test]
    fn cli_parses_wait_timeout_and_required_identity() {
        let mut args = vec![
            "wait",
            "--process-id",
            "4242",
            "--expected-executable",
            r"C:\Apps\Kaspa\gateway.exe",
            "--expected-start-time",
            "1700000000",
            "--output-path",
            "evidence.json",
            "--timeout-seconds",
            "9",
        ]
        .into_iter()
        .map(ToOwned::to_owned);
        let parsed = parse_options(&mut args).unwrap();
        assert_eq!(parsed.action, "wait");
        assert_eq!(parsed.process_id, 4242);
        assert_eq!(parsed.timeout_seconds, 9);
    }

    #[test]
    fn unknown_action_and_zero_pid_fail_closed() {
        let mut bad_action = vec!["stop".to_owned()].into_iter();
        assert!(parse_options(&mut bad_action).is_err());

        let mut zero_pid = vec![
            "kill",
            "--process-id",
            "0",
            "--expected-executable",
            "x",
            "--expected-start-time",
            "1",
            "--output-path",
            "evidence.json",
        ]
        .into_iter()
        .map(ToOwned::to_owned);
        assert!(parse_options(&mut zero_pid).is_err());
    }
}
