use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Record {
    relative_path: String,
    text: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let failures = validate_root(root);
    if failures.is_empty() {
        Ok(
            "KGW project continuity gate PASSED\nCanonical continuity files, active/current state, durable handoff and project memory, dynamic Git-state semantics, regression/security lifecycles, plan/ADR lifecycle, and release runbook are present."
                .to_owned(),
        )
    } else {
        let mut message = String::from("KGW project continuity gate FAILED");
        for failure in failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        Err(message)
    }
}

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern)
        .unwrap_or_else(|error| panic!("invalid continuity regex {pattern:?}: {error}"))
}

fn require_match(failures: &mut Vec<String>, text: &str, pattern: &str, description: &str) {
    if !regex(pattern).is_match(text) {
        failures.push(format!("Missing continuity policy: {description}"));
    }
}

fn forbid_match(failures: &mut Vec<String>, text: &str, pattern: &str, description: &str) {
    if regex(pattern).is_match(text) {
        failures.push(format!("Invalid continuity state: {description}"));
    }
}

fn read_required(root: &Path, relative: &str, failures: &mut Vec<String>) -> String {
    let path = root.join(relative);
    if !path.is_file() {
        failures.push(format!("Missing required continuity file: {relative}"));
        return String::new();
    }
    match fs::read_to_string(&path) {
        Ok(text) => {
            let text = text.replace("\r\n", "\n").replace('\r', "\n");
            if text.trim().is_empty() {
                failures.push(format!("Continuity file is empty: {relative}"));
            }
            text
        }
        Err(_) => {
            failures.push(format!("Missing required continuity file: {relative}"));
            String::new()
        }
    }
}

fn read_markdown_records(
    root: &Path,
    relative_dir: &str,
    failures: &mut Vec<String>,
) -> Vec<Record> {
    let dir = root.join(relative_dir);
    if !dir.is_dir() {
        failures.push(format!(
            "Missing required continuity directory: {relative_dir}"
        ));
        return Vec::new();
    }
    let mut paths: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                    && !matches!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some("README.md" | "TEMPLATE.md")
                    )
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            let relative = format!("{relative_dir}/{name}");
            Record {
                relative_path: relative.clone(),
                text: read_required(root, &relative, failures),
            }
        })
        .collect()
}
fn read_workflow_sources(root: &Path, failures: &mut Vec<String>) -> Vec<Record> {
    let relative_dir = ".github/workflows";
    let dir = root.join(relative_dir);
    if !dir.is_dir() {
        failures.push(format!(
            "Missing required continuity directory: {relative_dir}"
        ));
        return Vec::new();
    }
    let mut paths: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| {
                            ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml")
                        })
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            let relative = format!("{relative_dir}/{name}");
            let text = fs::read_to_string(root.join(&relative)).unwrap_or_default();
            Record {
                relative_path: relative,
                text,
            }
        })
        .collect()
}

fn validate_root(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();

    let agents = read_required(root, "AGENTS.md", &mut failures);
    let state = read_required(root, "PROJECT_STATE.md", &mut failures);
    let active_task = read_required(root, "ACTIVE_TASK.md", &mut failures);
    let current_state = read_required(root, "CURRENT_STATE.md", &mut failures);
    let plans = read_required(root, "PLANS.md", &mut failures);
    let continuity_policy = read_required(
        root,
        "docs/continuity/PROJECT_CONTINUITY_POLICY.md",
        &mut failures,
    );
    let handoff_ledger = read_required(root, "docs/handoff-ledger/README.md", &mut failures);
    let project_memory = read_required(root, "docs/project-memory/README.md", &mut failures);
    let bug_memory = read_required(root, "docs/project-memory/BUGS/README.md", &mut failures);
    let regression_memory = read_required(
        root,
        "docs/project-memory/REGRESSIONS/README.md",
        &mut failures,
    );
    let security_memory = read_required(
        root,
        "docs/project-memory/SECURITY/README.md",
        &mut failures,
    );
    let incident_memory = read_required(
        root,
        "docs/project-memory/INCIDENTS/README.md",
        &mut failures,
    );
    let decision_memory = read_required(
        root,
        "docs/project-memory/DECISIONS/README.md",
        &mut failures,
    );
    let known_failure_memory = read_required(
        root,
        "docs/project-memory/KNOWN_FAILURES/README.md",
        &mut failures,
    );
    let handoff_template = read_required(root, "docs/handoff-ledger/TEMPLATE.md", &mut failures);
    let memory_template = read_required(root, "docs/project-memory/TEMPLATE.md", &mut failures);
    let handoff_records = read_markdown_records(root, "docs/handoff-ledger", &mut failures);
    let mut memory_records = Vec::new();
    for category in [
        "BUGS",
        "REGRESSIONS",
        "SECURITY",
        "INCIDENTS",
        "DECISIONS",
        "KNOWN_FAILURES",
    ] {
        memory_records.extend(read_markdown_records(
            root,
            &format!("docs/project-memory/{category}"),
            &mut failures,
        ));
    }
    let adr_index = read_required(root, "docs/adr/README.md", &mut failures);
    let continuity_adr = read_required(
        root,
        "docs/adr/0011-repository-native-project-continuity.md",
        &mut failures,
    );
    let release_runbook = read_required(root, "docs/runbooks/desktop-release.md", &mut failures);
    let architecture_index = read_required(root, "docs/architecture/README.md", &mut failures);
    let workflow_sources = read_workflow_sources(root, &mut failures);
    if !agents.is_empty() {
        for (pattern, label) in [
            (
                r"(?i)## Session Start and Continuity",
                "session-start protocol in AGENTS.md",
            ),
            (
                r"(?i)PROJECT_STATE\.md",
                "PROJECT_STATE.md ownership in AGENTS.md",
            ),
            (
                r"(?is)conversation memory[\s\S]*advisory",
                "conversation-memory-is-advisory rule",
            ),
            (
                r"(?i)meaningful state transition",
                "meaningful-state-transition reconciliation rule",
            ),
            (
                r"(?i)PLANS\.md[^\n]*active multi-stage",
                "PLANS.md active-work ownership rule",
            ),
        ] {
            require_match(&mut failures, &agents, pattern, label);
        }
    }

    if !active_task.is_empty() {
        require_match(
            &mut failures,
            &active_task,
            r"(?m)^# ACTIVE TASK$",
            "ACTIVE_TASK.md title",
        );
        for (pattern, label) in [
            (r"(?i)## Status", "active-task status"),
            (r"(?i)## Objective", "active-task objective"),
            (r"(?i)## Scope", "active-task scope"),
            (r"(?i)## Current Phase", "active-task current phase"),
            (
                r"(?i)## Confirmed Progress",
                "active-task confirmed progress",
            ),
            (
                r"(?i)## Current Blocker",
                "active-task blocker classification",
            ),
            (
                r"(?i)## Last Completed Action",
                "active-task last completed action",
            ),
            (r"(?i)## Current Action", "active-task current action"),
            (r"(?i)## Next Action", "active-task next action"),
            (
                r"(?i)## Verification Required",
                "active-task verification contract",
            ),
            (
                r"(?i)## Completion Criteria",
                "active-task completion criteria",
            ),
            (
                r"(?i)## DO NOT REPEAT",
                "active-task do-not-repeat boundary",
            ),
        ] {
            require_match(&mut failures, &active_task, pattern, label);
        }
    }

    if !current_state.is_empty() {
        require_match(
            &mut failures,
            &current_state,
            r"(?m)^# CURRENT STATE$",
            "CURRENT_STATE.md title",
        );
        require_match(
            &mut failures,
            &current_state,
            r"(?i)Current HEAD:[^\n]*(?:VERIFY DYNAMICALLY|derive dynamically)",
            "current-state dynamic HEAD rule",
        );
        require_match(
            &mut failures,
            &current_state,
            r"(?i)Current remote main:[^\n]*VERIFY DYNAMICALLY",
            "current-state dynamic remote-main rule",
        );
        require_match(
            &mut failures,
            &current_state,
            r"(?i)Working tree:[^\n]*(?:CLEAN|DIRTY|NOT VERIFIED)",
            "current-state working-tree classification",
        );
        require_match(
            &mut failures,
            &current_state,
            r"(?i)## NEXT ACTION",
            "current-state next action",
        );
        require_match(
            &mut failures,
            &current_state,
            r"(?i)## DO NOT REPEAT",
            "current-state do-not-repeat",
        );
        require_match(
            &mut failures,
            &current_state,
            r"(?i)NOT VERIFIED",
            "current-state explicit not-verified state",
        );
    }
    if !state.is_empty() {
        for (pattern, label) in [
            (r"(?m)^# PROJECT STATE$", "PROJECT_STATE.md title"),
            (r"(?i)## Desired State", "Desired State section"),
            (r"(?i)## Actual State", "Actual State section"),
            (r"(?i)### CI", "actual CI state section"),
            (r"(?i)### Staging", "staging classification section"),
            (
                r"(?i)### Production / Live Runtime",
                "production/live-runtime classification section",
            ),
            (r"(?i)## Drift", "Drift section"),
            (
                r"(?i)## Last Verified Validation",
                "evidence-based validation section",
            ),
            (r"(?i)## NEXT ACTION", "precise NEXT ACTION section"),
            (r"(?i)## Resume Instructions", "resume protocol"),
            (
                r"(?i)Verified code baseline[^\n]*[0-9a-f]{40}",
                "historical verified code baseline",
            ),
            (
                r"(?i)State-document commit:[^\n]*derive dynamically",
                "dynamic state-document commit rule",
            ),
            (
                r"(?i)Current remote main:[^\n]*VERIFY DYNAMICALLY",
                "dynamic remote-main rule",
            ),
            (
                r"(?i)Current HEAD:[^\n]*(?:VERIFY DYNAMICALLY|derive dynamically)",
                "dynamic HEAD rule",
            ),
            (
                r"(?i)Working tree:[^\n]*(?:CLEAN|DIRTY|NOT VERIFIED)",
                "working-tree classification",
            ),
            (r"(?i)NOT VERIFIED", "explicit NOT VERIFIED classification"),
        ] {
            require_match(&mut failures, &state, pattern, label);
        }
        forbid_match(
            &mut failures,
            &state,
            r"(?im)^(?:-\s*)?(?:Current HEAD|Current remote main|Last verified live main|Live main last verified at):[^\n]*\b[0-9a-f]{40}\b",
            "current HEAD/main labels must not embed a static 40-character SHA; use a historical verified baseline plus dynamic current-state commands",
        );
    }

    if !plans.is_empty() {
        let inactive =
            regex(r"(?is)## Status\s*\n+\s*\*\*NO ACTIVE MULTI-STAGE PLAN\*\*\s*(?:\n|$)")
                .is_match(&plans);
        if inactive {
            for (pattern, label) in [
                (r"(?i)## Usage", "inactive PLANS usage contract"),
                (
                    r"(?i)## Most Recent Completed Plan",
                    "inactive PLANS completed-plan pointer",
                ),
                (
                    r"(?i)PROJECT_STATE\.md",
                    "inactive PLANS pointer to canonical current state",
                ),
                (
                    r"(?i)Git(?:Hub)?(?: Releases)?|Git/PRs",
                    "inactive PLANS pointer to durable history",
                ),
            ] {
                require_match(&mut failures, &plans, pattern, label);
            }
            forbid_match(
                &mut failures,
                &plans,
                r"(?im)^## (?:Objective|Success Criteria|Milestones|Progress|Completion Criteria)$",
                "inactive PLANS.md must not retain an active execution-plan body",
            );
        } else {
            for (pattern, label) in [
                (r"(?i)## Objective", "active PLANS objective"),
                (r"(?i)## Success Criteria", "active PLANS success criteria"),
                (r"(?i)## Milestones", "active PLANS milestones"),
                (r"(?i)## Progress", "active PLANS progress"),
                (
                    r"(?i)## Completion Criteria",
                    "active PLANS completion criteria",
                ),
            ] {
                require_match(&mut failures, &plans, pattern, label);
            }
        }
    }

    if !continuity_policy.is_empty() {
        for (pattern, label) in [
            (
                r"(?i)READ -> RECOVER -> VERIFY -> RECONCILE -> CHECK -> PRIORITIZE -> CONTINUE -> CHANGE -> TEST -> PROTECT AGAINST REGRESSION -> DOCUMENT -> CHECKPOINT -> HANDOFF",
                "full continuity lifecycle",
            ),
            (r"(?i)## Sources of Truth", "source-of-truth policy"),
            (
                r"(?i)Bug -> Reproduce -> Root Cause -> Fix -> Verification -> Regression Protection -> Documentation",
                "important-bug lifecycle",
            ),
            (
                r"(?i)Fix -> Strengthen -> Verify -> Protect",
                "recurring-problem strengthening lifecycle",
            ),
            (r"(?i)OWASP ASVS 5\.0\.0", "OWASP ASVS security reference"),
            (
                r"(?i)NIST Secure Software Development Framework",
                "NIST SSDF security-development reference",
            ),
            (
                r"(?i)Production-impacting and external actions require explicit authorization",
                "explicit external-action authorization rule",
            ),
            (
                r"(?i)## End-of-Task Contract",
                "end-of-task verification contract",
            ),
        ] {
            require_match(&mut failures, &continuity_policy, pattern, label);
        }
    }
    if !handoff_ledger.is_empty() {
        for (pattern, label) in [
            (r"(?i)LAST CONFIRMED STATE", "handoff last confirmed state"),
            (r"(?i)NEXT ACTION", "handoff next action"),
            (r"(?i)DO NOT REPEAT", "handoff do-not-repeat boundary"),
            (r"(?i)Timestamp", "handoff timestamp field"),
            (r"(?i)Test|evidence", "handoff verification/evidence field"),
        ] {
            require_match(&mut failures, &handoff_ledger, pattern, label);
        }
    }

    if !handoff_template.is_empty() {
        for (pattern, label) in [
            (
                r"(?i)LAST CONFIRMED STATE",
                "handoff template last confirmed state",
            ),
            (
                r"(?i)COMPLETED / VERIFIED",
                "handoff template completed/verified",
            ),
            (r"(?i)EVIDENCE / TESTS", "handoff template evidence/tests"),
            (
                r"(?i)BLOCKERS / REMAINING WORK",
                "handoff template blockers/remaining work",
            ),
            (r"(?i)NEXT ACTION", "handoff template next action"),
            (r"(?i)DO NOT REPEAT", "handoff template do-not-repeat"),
        ] {
            require_match(&mut failures, &handoff_template, pattern, label);
        }
    }

    if handoff_records.is_empty() {
        failures.push("No durable task checkpoint exists under docs/handoff-ledger/.".to_owned());
    }
    for record in &handoff_records {
        for (pattern, label) in [
            (r"(?i)Status:", "status"),
            (r"(?i)Timestamp:", "timestamp"),
            (r"(?i)## LAST CONFIRMED STATE", "last confirmed state"),
            (
                r"(?i)## (?:COMPLETED|COMPLETED / VERIFIED)",
                "completed/verified work",
            ),
            (r"(?i)## EVIDENCE", "evidence"),
            (r"(?i)## NEXT ACTION", "next action"),
            (r"(?i)## DO NOT REPEAT", "do-not-repeat"),
        ] {
            require_match(
                &mut failures,
                &record.text,
                pattern,
                &format!("{}: {label}", record.relative_path),
            );
        }
    }

    if !memory_template.is_empty() {
        for (pattern, label) in [
            (r"(?i)Status:", "memory template status"),
            (r"(?i)## Evidence", "memory template evidence"),
            (r"(?i)## Root Cause", "memory template root cause"),
            (r"(?i)## Verification", "memory template verification"),
            (
                r"(?i)## Regression Protection",
                "memory template regression protection",
            ),
            (r"(?i)## NEXT ACTION", "memory template next action"),
            (r"(?i)## DO NOT REPEAT", "memory template do-not-repeat"),
        ] {
            require_match(&mut failures, &memory_template, pattern, label);
        }
    }

    if memory_records.is_empty() {
        failures
            .push("No durable project-memory record exists under docs/project-memory/.".to_owned());
    }
    let memory_status_pattern = r"(?i)Status:\s*(?:OPEN|IN PROGRESS|RESOLVED|VERIFIED|NOT REPRODUCIBLE|DEFERRED|BLOCKED|REQUIRES ACTION|DUPLICATE|FALSE POSITIVE)";
    let memory_id_pattern = r"(?m)^(?:# )?(?:BUG|REG|SEC|INC|DEC|FAIL)-\d{4}:";
    let memory_id_capture = regex(r"(?m)^(?:# )?((?:BUG|REG|SEC|INC|DEC|FAIL)-\d{4}):");
    let mut memory_ids = BTreeMap::new();

    for record in &memory_records {
        require_match(
            &mut failures,
            &record.text,
            memory_id_pattern,
            &format!("{}: stable ID", record.relative_path),
        );
        if let Some(capture) = memory_id_capture.captures(&record.text)
            && let Some(id) = capture.get(1)
        {
            let stable_id = id.as_str().to_ascii_uppercase();
            if let Some(previous) =
                memory_ids.insert(stable_id.clone(), record.relative_path.clone())
            {
                failures.push(format!(
                    "Duplicate project-memory stable ID {stable_id}: {previous} and {}.",
                    record.relative_path
                ));
            }
        }
        for (pattern, label) in [
            (memory_status_pattern, "lifecycle status"),
            (r"(?i)## Evidence", "evidence"),
            (r"(?i)## Root Cause", "root cause"),
            (r"(?i)## Verification", "verification"),
            (r"(?i)## Regression Protection", "regression protection"),
            (r"(?i)## NEXT ACTION", "next action"),
            (r"(?i)## DO NOT REPEAT", "do-not-repeat"),
        ] {
            require_match(
                &mut failures,
                &record.text,
                pattern,
                &format!("{}: {label}", record.relative_path),
            );
        }
    }
    if !project_memory.is_empty() {
        for (pattern, label) in [
            (r"(?i)BUG-NNNN", "bug stable identifier"),
            (r"(?i)REG-NNNN", "regression stable identifier"),
            (r"(?i)SEC-NNNN", "security stable identifier"),
            (r"(?i)INC-NNNN", "incident stable identifier"),
            (r"(?i)DEC-NNNN", "decision stable identifier"),
            (r"(?i)FAIL-NNNN", "known-failure stable identifier"),
            (
                r"(?is)OPEN[\s\S]*IN PROGRESS[\s\S]*RESOLVED[\s\S]*VERIFIED",
                "explicit project-memory statuses",
            ),
        ] {
            require_match(&mut failures, &project_memory, pattern, label);
        }
    }

    for (text, id_pattern, label) in [
        (&bug_memory, r"(?i)BUG-NNNN", "bug memory category"),
        (
            &regression_memory,
            r"(?i)REG-NNNN",
            "regression memory category",
        ),
        (
            &security_memory,
            r"(?i)SEC-NNNN",
            "security memory category",
        ),
        (
            &incident_memory,
            r"(?i)INC-NNNN",
            "incident memory category",
        ),
        (
            &decision_memory,
            r"(?i)DEC-NNNN",
            "decision memory category",
        ),
        (
            &known_failure_memory,
            r"(?i)FAIL-NNNN",
            "known-failure memory category",
        ),
    ] {
        if text.is_empty() {
            continue;
        }
        require_match(
            &mut failures,
            text,
            id_pattern,
            &format!("{label} stable ID"),
        );
        require_match(
            &mut failures,
            text,
            r"(?i)NEXT ACTION",
            &format!("{label} next-action contract"),
        );
        require_match(
            &mut failures,
            text,
            r"(?i)status",
            &format!("{label} explicit status contract"),
        );
    }

    if !adr_index.is_empty() {
        require_match(
            &mut failures,
            &adr_index,
            r"(?is)Proposed[\s\S]*Accepted[\s\S]*Deprecated[\s\S]*Superseded",
            "ADR lifecycle states",
        );
        require_match(
            &mut failures,
            &adr_index,
            r"(?is)0010[\s\S]*0011",
            "ADR numbering continuity/index",
        );
    }

    if !continuity_adr.is_empty() {
        for (pattern, label) in [
            (r"(?i)Status: Accepted", "accepted continuity ADR status"),
            (r"(?i)source-of-truth", "source-of-truth decision"),
            (
                r"(?i)NO ACTIVE MULTI-STAGE PLAN",
                "inactive-plan lifecycle decision",
            ),
            (
                r"(?i)self-stale|static SHA",
                "self-stale/static-current-SHA decision",
            ),
        ] {
            require_match(&mut failures, &continuity_adr, pattern, label);
        }
    }

    if !release_runbook.is_empty() {
        for (pattern, label) in [
            (r"(?i)## Preconditions", "release preconditions"),
            (r"(?i)## Abort Conditions", "release abort conditions"),
            (
                r"(?i)## Post-Publication Verification",
                "post-publication verification",
            ),
            (
                r"(?i)explicit user authorization",
                "explicit publication authorization gate",
            ),
        ] {
            require_match(&mut failures, &release_runbook, pattern, label);
        }
    }

    if !architecture_index.is_empty() {
        require_match(
            &mut failures,
            &architecture_index,
            r"(?i)local-first Rust/Tauri desktop control plane",
            "architecture identity",
        );
        require_match(
            &mut failures,
            &architecture_index,
            r"(?i)raw runtime log panes",
            "raw-log invariant",
        );
    }
    let mut combined_parts = vec![
        agents,
        state,
        active_task,
        current_state,
        plans,
        continuity_policy,
        handoff_ledger,
        project_memory,
        bug_memory,
        regression_memory,
        security_memory,
        incident_memory,
        decision_memory,
        known_failure_memory,
        handoff_template,
        memory_template,
    ];
    combined_parts.extend(handoff_records.iter().map(|record| record.text.clone()));
    combined_parts.extend(memory_records.iter().map(|record| record.text.clone()));
    combined_parts.extend([
        adr_index,
        continuity_adr,
        release_runbook,
        architecture_index,
    ]);
    let combined = combined_parts.join("\n");

    if regex(
        r"(?i)\b(?:GITHUB_TOKEN|RELEASE_ADMIN_TOKEN|DATABASE_URL|EMAIL_API_KEY|CLOUDFLARE_API_TOKEN|PASSWORD|PRIVATE_KEY|CLIENT_SECRET|API_TOKEN)\s*=\s*[^\s\x60]+",
    )
    .is_match(&combined)
    {
        failures.push("Possible secret value assignment found in continuity documentation.".to_owned());
    }

    for workflow in &workflow_sources {
        if workflow.text.contains("RELEASE_ADMIN_TOKEN") {
            failures.push(format!(
                "Retired GitHub Actions secret RELEASE_ADMIN_TOKEN must not be referenced by workflow: {}",
                workflow.relative_path
            ));
        }
    }

    failures
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    struct TempFixture {
        root: PathBuf,
    }

    impl Drop for TempFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn unique_temp_root() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "kgw-continuity-gate-rust-{}-{id}",
            std::process::id()
        ))
    }

    fn copy_recursive(source: &Path, target: &Path) {
        if source.is_dir() {
            fs::create_dir_all(target).unwrap();
            let mut entries: Vec<_> = fs::read_dir(source)
                .unwrap()
                .filter_map(Result::ok)
                .collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                copy_recursive(&entry.path(), &target.join(entry.file_name()));
            }
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(source, target).unwrap();
        }
    }
    fn fixture() -> TempFixture {
        let source_root = repo_root();
        let root = unique_temp_root();
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        for relative in [
            "AGENTS.md",
            "PROJECT_STATE.md",
            "ACTIVE_TASK.md",
            "CURRENT_STATE.md",
            "PLANS.md",
            "docs/continuity",
            "docs/handoff-ledger",
            "docs/project-memory",
            "docs/adr/README.md",
            "docs/adr/0011-repository-native-project-continuity.md",
            "docs/runbooks/desktop-release.md",
            "docs/architecture/README.md",
            ".github/workflows",
        ] {
            copy_recursive(&source_root.join(relative), &root.join(relative));
        }
        TempFixture { root }
    }

    fn assert_pass(root: &Path) {
        let failures = validate_root(root);
        assert!(
            failures.is_empty(),
            "unexpected failures:\n{}",
            failures.join("\n")
        );
    }

    fn assert_failure_contains(root: &Path, expected: &str) {
        let failures = validate_root(root);
        assert!(
            failures.iter().any(|failure| failure.contains(expected)),
            "expected failure containing {expected:?}; got:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn baseline_continuity_fixture_passes() {
        let fixture = fixture();
        assert_pass(&fixture.root);
    }

    #[test]
    fn active_plan_may_mention_future_inactive_sentinel_in_prose() {
        let fixture = fixture();
        let path = fixture.root.join("PLANS.md");
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\nFuture closure returns this file to NO ACTIVE MULTI-STAGE PLAN.\n");
        fs::write(path, text).unwrap();
        assert_pass(&fixture.root);
    }

    #[test]
    fn missing_active_task_fails_closed() {
        let fixture = fixture();
        fs::remove_file(fixture.root.join("ACTIVE_TASK.md")).unwrap();
        assert_failure_contains(
            &fixture.root,
            "Missing required continuity file: ACTIVE_TASK.md",
        );
    }

    #[test]
    fn missing_durable_checkpoint_fails_closed() {
        let fixture = fixture();
        let dir = fixture.root.join("docs/handoff-ledger");
        for entry in fs::read_dir(&dir).unwrap().filter_map(Result::ok) {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                && !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("README.md" | "TEMPLATE.md")
                )
            {
                fs::remove_file(path).unwrap();
            }
        }
        assert_failure_contains(
            &fixture.root,
            "No durable task checkpoint exists under docs/handoff-ledger/.",
        );
    }

    #[test]
    fn invalid_security_memory_status_fails_closed() {
        let fixture = fixture();
        let dir = fixture.root.join("docs/project-memory/SECURITY");
        let path = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                    && path.file_name().and_then(|name| name.to_str()) != Some("README.md")
            })
            .expect("security fixture record missing");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(
            path,
            text.replace("- Status: VERIFIED", "- Status: UNKNOWN"),
        )
        .unwrap();
        assert_failure_contains(&fixture.root, "lifecycle status");
    }
    #[test]
    fn duplicate_project_memory_stable_id_fails_closed() {
        let fixture = fixture();
        let dir = fixture.root.join("docs/project-memory/SECURITY");
        let source = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                    && path.file_name().and_then(|name| name.to_str()) != Some("README.md")
            })
            .expect("security fixture record missing");
        let name = source.file_name().unwrap().to_string_lossy();
        fs::copy(&source, dir.join(format!("DUPLICATE-{name}"))).unwrap();
        assert_failure_contains(&fixture.root, "Duplicate project-memory stable ID");
    }

    #[test]
    fn retired_release_admin_secret_workflow_reference_fails_closed() {
        let fixture = fixture();
        let path = fixture.root.join(".github/workflows/ci.yml");
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\n# negative fixture only\n# RELEASE_ADMIN_TOKEN must remain retired\n");
        fs::write(path, text).unwrap();
        assert_failure_contains(
            &fixture.root,
            "Retired GitHub Actions secret RELEASE_ADMIN_TOKEN must not be referenced by workflow",
        );
    }

    #[test]
    fn missing_regression_memory_category_fails_closed() {
        let fixture = fixture();
        fs::remove_file(
            fixture
                .root
                .join("docs/project-memory/REGRESSIONS/README.md"),
        )
        .unwrap();
        assert_failure_contains(
            &fixture.root,
            "Missing required continuity file: docs/project-memory/REGRESSIONS/README.md",
        );
    }

    #[test]
    fn missing_root_cause_lifecycle_fails_closed() {
        let fixture = fixture();
        let path = fixture
            .root
            .join("docs/continuity/PROJECT_CONTINUITY_POLICY.md");
        let text = fs::read_to_string(&path).unwrap();
        fs::write(
            path,
            text.replace(
                "Bug -> Reproduce -> Root Cause -> Fix -> Verification -> Regression Protection -> Documentation",
                "Bug -> Patch -> Done",
            ),
        )
        .unwrap();
        assert_failure_contains(&fixture.root, "important-bug lifecycle");
    }
}
