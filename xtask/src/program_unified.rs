use serde_json::{Value, json};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub struct RunResult {
    pub code: i32,
    pub output: String,
}

#[derive(Debug, Clone)]
struct Options {
    repo_root: PathBuf,
    report_dir: PathBuf,
    strict: bool,
    json_mode: bool,
    offline_only: bool,
    skip_online: bool,
    skip_trace: bool,
    skip_runtime: bool,
    skip_node_bridge: bool,
}
#[derive(Debug, Clone)]
struct Step {
    name: String,
    command: String,
    args: Vec<String>,
    cwd: PathBuf,
    required: bool,
}

#[derive(Debug, Clone)]
struct StepResult {
    name: String,
    code: i32,
    required: bool,
    log: PathBuf,
    error: String,
}

impl StepResult {
    fn ok(&self) -> bool {
        self.code == 0
    }
}

fn now_iso() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| format!("unified gate timestamp failed: {error}"))
}
fn default_report_dir(root: &Path) -> Result<PathBuf, String> {
    let stamp = now_iso()?.replace([':', '.'], "-");
    Ok(root
        .join("reports")
        .join(format!("kgw_program_unified_gate_{stamp}")))
}

fn parse_options(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<Options, String> {
    let mut repo_root = default_root.to_path_buf();
    let mut report_dir: Option<PathBuf> = None;
    let mut json_mode = false;
    let mut offline_only = false;
    let mut skip_online = false;
    let mut skip_trace = false;
    let mut skip_runtime = false;
    let mut skip_node_bridge = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--strict" => {}
            "--json" => json_mode = true,
            "--offline-only" => offline_only = true,
            "--skip-online" => skip_online = true,
            "--skip-trace" => skip_trace = true,
            "--skip-runtime" => skip_runtime = true,
            "--skip-node-bridge" => skip_node_bridge = true,
            "--repo-root" => {
                repo_root = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--repo-root requires a value".to_owned())?,
                );
            }
            "--report-dir" => {
                report_dir = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--report-dir requires a value".to_owned())?,
                ));
            }
            _ => return Err(format!("unknown program-unified-gate argument: {arg}")),
        }
    }

    let report_dir = match report_dir {
        Some(path) => path,
        None => default_report_dir(&repo_root)?,
    };
    Ok(Options {
        repo_root,
        report_dir,
        strict: true,
        json_mode,
        offline_only,
        skip_online: skip_online || offline_only,
        skip_trace,
        skip_runtime,
        skip_node_bridge,
    })
}

fn syntax_step(root: &Path, relative: &str) -> Step {
    Step {
        name: format!("node_check_{}", relative.replace(['/', '\\'], "_")),
        command: "node".to_owned(),
        args: vec!["--check".to_owned(), relative.to_owned()],
        cwd: root.to_path_buf(),
        required: true,
    }
}

fn cargo_step(root: &Path, name: &str, xtask_args: &[&str]) -> Step {
    let mut args = vec![
        "run".to_owned(),
        "--locked".to_owned(),
        "-p".to_owned(),
        "xtask".to_owned(),
        "--".to_owned(),
    ];
    args.extend(xtask_args.iter().map(|value| (*value).to_owned()));
    Step {
        name: name.to_owned(),
        command: "cargo".to_owned(),
        args,
        cwd: root.to_path_buf(),
        required: true,
    }
}

fn node_step(root: &Path, name: &str, script: &str, args: &[String]) -> Step {
    let mut all = vec![script.to_owned()];
    all.extend(args.iter().cloned());
    Step {
        name: name.to_owned(),
        command: "node".to_owned(),
        args: all,
        cwd: root.to_path_buf(),
        required: true,
    }
}

fn build_steps(options: &Options) -> Vec<Step> {
    let root = &options.repo_root;
    let mut steps = Vec::new();

    let mut syntax_targets = Vec::new();
    if root.join("tools/kgw_program_unified_gate.cjs").is_file() {
        syntax_targets.push("tools/kgw_program_unified_gate.cjs");
    }
    syntax_targets.extend([
        "tools/kgw_global_owner_gate.cjs",
        "tools/kgw_bridge_node_mode_routing_audit_v1.cjs",
        "tools/kgw_runtime_trace_owner_audit_v20.cjs",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
    ]);
    for target in syntax_targets {
        steps.push(syntax_step(root, target));
    }

    steps.push(node_step(
        root,
        "global_owner_gate_strict",
        "tools/kgw_global_owner_gate.cjs",
        &["--strict".to_owned()],
    ));
    steps.push(cargo_step(
        root,
        "i18n_contract_gate",
        &["i18n-contract-gate"],
    ));
    steps.push(cargo_step(
        root,
        "i18n_locale_coverage_gate",
        &["i18n-locale-coverage-gate"],
    ));
    steps.push(cargo_step(
        root,
        "runtime_repository_binding_gate_offline",
        &[
            "runtime-repository-binding-gate",
            "--strict",
            "--offline",
            "--json",
        ],
    ));

    if !options.skip_online {
        steps.push(cargo_step(
            root,
            "runtime_repository_binding_gate_online_latest",
            &["runtime-repository-binding-gate", "--strict", "--json"],
        ));
    }
    if !options.skip_node_bridge {
        let report = options.report_dir.join("bridge_node_mode_routing_audit");
        steps.push(node_step(
            root,
            "bridge_node_mode_routing_audit",
            "tools/kgw_bridge_node_mode_routing_audit_v1.cjs",
            &[
                root.to_string_lossy().into_owned(),
                report.to_string_lossy().into_owned(),
            ],
        ));
    }

    if !options.skip_trace {
        let report = options.report_dir.join("runtime_trace_owner_audit_v20");
        steps.push(node_step(
            root,
            "runtime_trace_owner_audit_v20",
            "tools/kgw_runtime_trace_owner_audit_v20.cjs",
            &[
                root.to_string_lossy().into_owned(),
                report.to_string_lossy().into_owned(),
            ],
        ));
    }
    if !options.skip_runtime {
        steps.push(cargo_step(
            root,
            "parallel_self_worker_runtime_gate",
            &["parallel-self-worker-runtime-gate"],
        ));
        steps.push(cargo_step(
            root,
            "raw_log_provenance_gate",
            &["raw-log-provenance-gate"],
        ));
    }

    steps.push(Step {
        name: "git_status_short".to_owned(),
        command: "git".to_owned(),
        args: vec!["status".to_owned(), "--short".to_owned()],
        cwd: root.to_path_buf(),
        required: false,
    });
    steps.push(Step {
        name: "git_diff_tools".to_owned(),
        command: "git".to_owned(),
        args: vec!["diff".to_owned(), "--".to_owned(), "tools".to_owned()],
        cwd: root.to_path_buf(),
        required: false,
    });
    steps
}

fn safe_log_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn command_display(command: &str, args: &[String]) -> String {
    std::iter::once(command)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn run_step(step: &Step, report_dir: &Path) -> Result<StepResult, String> {
    let log = report_dir.join(format!("{}.log", safe_log_name(&step.name)));
    let started = now_iso().unwrap_or_else(|_| "<time-error>".to_owned());
    let output = Command::new(&step.command)
        .args(&step.args)
        .current_dir(&step.cwd)
        .env("KGW_UNIFIED_GATE", "1")
        .output();

    let (code, stdout, stderr, error) = match output {
        Ok(output) => (
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
            String::new(),
        ),
        Err(error) => (1, String::new(), String::new(), error.to_string()),
    };
    let ended = now_iso().unwrap_or_else(|_| "<time-error>".to_owned());

    let body = format!(
        "# {}\nstarted={}\nended={}\ncwd={}\nrequired={}\ncommand={}\ncode={}\nerror={}\n\n--- STDOUT ---\n{}\n\n--- STDERR ---\n{}",
        step.name,
        started,
        ended,
        step.cwd.display(),
        step.required,
        command_display(&step.command, &step.args),
        code,
        error,
        stdout,
        stderr
    );
    fs::write(&log, body).map_err(|error| {
        format!(
            "failed to write unified step log {}: {error}",
            log.display()
        )
    })?;

    Ok(StepResult {
        name: step.name.clone(),
        code,
        required: step.required,
        log,
        error,
    })
}
fn normalized_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Prefix(prefix) => {
                Some(prefix.as_os_str().to_string_lossy().to_ascii_lowercase())
            }
            Component::RootDir => Some("/".to_owned()),
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            Component::ParentDir => Some("..".to_owned()),
            Component::CurDir => None,
        })
        .collect()
}

fn relative_path(from: &Path, to: &Path) -> String {
    let from_parts = normalized_components(from);
    let to_parts = normalized_components(to);
    let mut common = 0;
    while common < from_parts.len()
        && common < to_parts.len()
        && from_parts[common].eq_ignore_ascii_case(&to_parts[common])
    {
        common += 1;
    }
    if common == 0 {
        return to.to_string_lossy().replace('\\', "/");
    }
    let mut parts = Vec::new();
    parts.extend(std::iter::repeat_n(
        "..".to_owned(),
        from_parts.len() - common,
    ));
    parts.extend(to_parts[common..].iter().cloned());
    if parts.is_empty() {
        ".".to_owned()
    } else {
        parts.join("/")
    }
}

fn result_json(root: &Path, result: &StepResult) -> Value {
    json!({
        "name": result.name,
        "ok": result.ok(),
        "code": result.code,
        "required": result.required,
        "log": relative_path(root, &result.log),
        "error": result.error,
    })
}

fn summary_json(
    options: &Options,
    started_at: &str,
    finished_at: &str,
    results: &[StepResult],
) -> Value {
    let failed: Vec<_> = results
        .iter()
        .filter(|result| result.required && !result.ok())
        .collect();

    json!({
        "verdict": if failed.is_empty() {
            "KGW_PROGRAM_UNIFIED_GATE_PASS"
        } else {
            "KGW_PROGRAM_UNIFIED_GATE_FAIL"
        },
        "ok": failed.is_empty(),
        "strict": options.strict,
        "repoRoot": options.repo_root.to_string_lossy(),
        "reportDir": options.report_dir.to_string_lossy(),
        "startedAt": started_at,
        "finishedAt": finished_at,

        "options": {
            "offlineOnly": options.offline_only,
            "skipOnline": options.skip_online,
            "skipTrace": options.skip_trace,
            "skipRuntime": options.skip_runtime,
            "skipNodeBridge": options.skip_node_bridge,
            "jsonMode": options.json_mode,
        },
        "resultCount": results.len(),
        "failedRequiredCount": failed.len(),
        "failedRequired": failed
            .iter()
            .map(|result| json!({
                "name": result.name,
                "code": result.code,
                "log": relative_path(&options.repo_root, &result.log),
                "error": result.error,
            }))
            .collect::<Vec<_>>(),
        "results": results
            .iter()
            .map(|result| result_json(&options.repo_root, result))
            .collect::<Vec<_>>(),
    })
}

fn summary_markdown(options: &Options, summary: &Value) -> String {
    let verdict = summary
        .get("verdict")
        .and_then(Value::as_str)
        .unwrap_or("KGW_PROGRAM_UNIFIED_GATE_FAIL");
    let mut out = format!(
        "# KGW Program Unified Gate\n\nVerdict: {verdict}\n\nRepository: {}\n\nReport directory: {}\n\n## Results\n\n",
        options.repo_root.display(),
        options.report_dir.display()
    );

    if let Some(results) = summary.get("results").and_then(Value::as_array) {
        for result in results {
            let ok = result.get("ok").and_then(Value::as_bool).unwrap_or(false);
            let name = result.get("name").and_then(Value::as_str).unwrap_or("");
            let code = result.get("code").and_then(Value::as_i64).unwrap_or(1);
            let log = result.get("log").and_then(Value::as_str).unwrap_or("");
            let error = result.get("error").and_then(Value::as_str).unwrap_or("");
            out.push_str(&format!(
                "- {} {name} — code {code}",
                if ok { "PASS" } else { "FAIL" }
            ));
            if !log.is_empty() {
                out.push_str(&format!(" — {log}"));
            }

            if !error.is_empty() {
                out.push_str(&format!(" — {error}"));
            }
            out.push('\n');
        }
    }

    let failed_count = summary
        .get("failedRequiredCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if failed_count > 0 {
        out.push_str("\n## Failed required gates\n\n");
        if let Some(failed) = summary.get("failedRequired").and_then(Value::as_array) {
            for item in failed {
                let name = item.get("name").and_then(Value::as_str).unwrap_or("");
                let code = item.get("code").and_then(Value::as_i64).unwrap_or(1);
                let log = item.get("log").and_then(Value::as_str).unwrap_or("");
                out.push_str(&format!("- {name}: code {code}, log {log}\n"));
            }
        }
    }
    out
}

pub fn run_cli(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<RunResult, String> {
    let options = parse_options(args, default_root)?;
    fs::create_dir_all(&options.report_dir).map_err(|error| {
        format!(
            "failed to create unified report directory {}: {error}",
            options.report_dir.display()
        )
    })?;

    let started_at = now_iso()?;
    let mut results = Vec::new();
    for step in build_steps(&options) {
        results.push(run_step(&step, &options.report_dir)?);
    }
    let finished_at = now_iso()?;
    let summary = summary_json(&options, &started_at, &finished_at, &results);
    let mut json_text = serde_json::to_string_pretty(&summary)
        .map_err(|error| format!("failed to serialize unified summary: {error}"))?;
    json_text.push('\n');
    let markdown = summary_markdown(&options, &summary);
    let json_path = options
        .report_dir
        .join("kgw_program_unified_gate.summary.json");
    let md_path = options
        .report_dir
        .join("kgw_program_unified_gate.summary.md");
    fs::write(&json_path, &json_text)
        .map_err(|error| format!("failed to write {}: {error}", json_path.display()))?;
    fs::write(&md_path, &markdown)
        .map_err(|error| format!("failed to write {}: {error}", md_path.display()))?;

    let ok = summary.get("ok").and_then(Value::as_bool).unwrap_or(false);
    Ok(RunResult {
        code: if ok { 0 } else { 1 },
        output: if options.json_mode {
            json_text.trim_end().to_owned()
        } else {
            markdown
        },
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    fn options_for_test() -> Options {
        Options {
            repo_root: PathBuf::from("C:/kgw/fake-repo"),
            report_dir: PathBuf::from("C:/kgw/reports/test"),
            strict: true,
            json_mode: false,
            offline_only: true,
            skip_online: true,
            skip_trace: true,
            skip_runtime: false,
            skip_node_bridge: true,
        }
    }

    #[test]
    fn offline_plan_without_legacy_self_is_stable() {
        let options = options_for_test();
        let names: Vec<_> = build_steps(&options)
            .into_iter()
            .map(|step| step.name)
            .collect();
        assert_eq!(names.len(), 13);
        assert_eq!(names[0], "node_check_tools_kgw_global_owner_gate.cjs");
        assert!(names.contains(&"runtime_repository_binding_gate_offline".to_owned()));
        assert!(names.contains(&"parallel_self_worker_runtime_gate".to_owned()));
        assert!(names.contains(&"raw_log_provenance_gate".to_owned()));
        assert_eq!(names[names.len() - 2], "git_status_short");
        assert_eq!(names[names.len() - 1], "git_diff_tools");
    }

    #[test]
    fn offline_only_implies_skip_online_and_strict() {
        let mut args = vec![
            "--offline-only".to_owned(),
            "--skip-trace".to_owned(),
            "--skip-node-bridge".to_owned(),
            "--report-dir".to_owned(),
            "C:/kgw/report".to_owned(),
        ]
        .into_iter();
        let options = parse_options(&mut args, Path::new("C:/kgw/repo")).unwrap();
        assert!(options.strict);
        assert!(options.offline_only);
        assert!(options.skip_online);
        assert!(options.skip_trace);
        assert!(options.skip_node_bridge);
        assert_eq!(options.report_dir, PathBuf::from("C:/kgw/report"));
    }

    #[test]
    fn unknown_argument_fails_closed() {
        let mut args = vec!["--surprise".to_owned()].into_iter();
        let error = parse_options(&mut args, Path::new("C:/kgw/repo")).unwrap_err();
        assert!(error.contains("unknown program-unified-gate argument"));
    }

    #[test]
    fn log_name_sanitization_matches_legacy_shape() {
        assert_eq!(safe_log_name("node/check tools:x"), "node_check_tools_x");
    }
}
