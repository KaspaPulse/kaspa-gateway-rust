use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;

const REGISTRY_PATH: &str = "docs/governance/global-owner-registry.json";
const GATE_MARKER: &str = "KGW_CANONICAL_GLOBAL_OWNER_GATE_R3C_REGISTRY_REFINEMENT";

const SCAN_ROOTS: &[&str] = &[
    "tools",
    "apps/kaspa-gateway-desktop/frontend",
    "apps/kaspa-gateway-desktop/src-tauri/src",
    "crates/kaspa-gateway-frontend-wasm/src",
    "crates/kaspa-gateway-rk-bridge/src",
    "crates/kaspa-gateway-rk-node/src",
    "config",
];

const SCAN_EXTS: &[&str] = &[
    ".js", ".css", ".rs", ".json", ".toml", ".md", ".html", ".cjs",
];

const OWNER_AUDIT_TOOL_FILES: &[&str] = &[
    "xtask/src/global_owner.rs",
    "xtask/src/runtime_trace_owner.rs",
    "xtask/src/i18n_contracts.rs",
    "xtask/src/parallel_self_worker.rs",
    "xtask/src/bridge_node_mode_routing.rs",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Owner {
    owner_id: String,
    #[serde(default, rename = "description")]
    _description: String,
    #[serde(default)]
    active_files: Vec<String>,
    #[serde(default)]
    reference_files: Vec<String>,
    #[serde(default)]
    required_markers: Vec<String>,
    #[serde(default)]
    required_files: Vec<String>,
    #[serde(default)]
    forbidden_markers: Vec<String>,
    #[serde(default)]
    responsibilities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OwnerEntry {
    name: String,
    #[serde(flatten)]
    owner: Owner,
}

#[derive(Debug, Clone, Default)]
struct Args {
    strict: bool,
    owner: Option<String>,
    changed_files: Option<Vec<String>>,
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct Found {
    marker: String,
    file: String,
    kind: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct Evidence {
    marker: String,
    file: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ContextLine {
    line: usize,
    text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ForbiddenHit {
    marker: String,
    file: String,
    line: usize,
    context: Vec<ContextLine>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OwnerResult {
    owner_name: String,
    owner_id: String,
    required_missing: Vec<String>,
    required_files_missing: Vec<String>,
    required_found: Vec<Found>,
    reference_evidence: Vec<Evidence>,
    forbidden_hits: Vec<ForbiddenHit>,
    marker_outside_owner_files: Vec<Evidence>,
    active_evidence_files: Vec<String>,
    reference_evidence_files: Vec<String>,
    responsibilities: Vec<String>,
}

#[derive(Debug, Clone)]
struct LiteralHit {
    line: usize,
    context: Vec<ContextLine>,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub output: String,
    pub code: i32,
}

fn normalize_rel(value: &str) -> String {
    let mut value = value.replace('\\', "/").trim().to_owned();
    while let Some(rest) = value.strip_prefix("./") {
        value = rest.to_owned();
    }
    value
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Args {
    let values: Vec<String> = args.collect();
    let mut parsed = Args::default();
    let mut index = 0usize;
    while index < values.len() {
        let value = &values[index];
        match value.as_str() {
            "--strict" => parsed.strict = true,
            "--json" => parsed.json = true,
            "--help" | "-h" => parsed.help = true,
            "--owner" => {
                index += 1;
                parsed.owner = values.get(index).cloned();
            }
            "--changed-files" => {
                let mut files = Vec::new();
                while index + 1 < values.len() && !values[index + 1].starts_with("--") {
                    index += 1;
                    files.push(normalize_rel(&values[index]));
                }
                parsed.changed_files = Some(files);
            }
            _ if value.starts_with("--owner=") => {
                parsed.owner = Some(value["--owner=".len()..].to_owned());
            }
            _ if value.starts_with("--changed-files=") => {
                parsed.changed_files = Some(
                    value["--changed-files=".len()..]
                        .split(',')
                        .map(normalize_rel)
                        .filter(|item| !item.is_empty())
                        .collect(),
                );
            }
            _ => {}
        }
        index += 1;
    }
    parsed
}

fn load_registry(repo_root: &Path) -> Result<Vec<OwnerEntry>, String> {
    let path = repo_root.join(REGISTRY_PATH);
    let bytes =
        fs::read(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn exists(repo_root: &Path, rel: &str) -> bool {
    repo_root.join(normalize_rel(rel)).exists()
}

fn read_text(repo_root: &Path, rel: &str) -> Result<String, String> {
    let path = repo_root.join(normalize_rel(rel));
    fs::read_to_string(&path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn is_listed_file(rel: &str, list: &[String]) -> bool {
    let file = normalize_rel(rel);
    list.iter().any(|entry| {
        let normalized = normalize_rel(entry);
        file == normalized || file.starts_with(&(normalized + "/"))
    })
}

fn is_owner_tool_file(rel: &str) -> bool {
    let rel = normalize_rel(rel);
    OWNER_AUDIT_TOOL_FILES.iter().any(|item| rel == *item)
}

fn walk(repo_root: &Path, root_rel: &str) -> Vec<String> {
    let root = repo_root.join(root_rel);
    let mut out = Vec::new();
    let skip: HashSet<&str> = [
        ".git",
        "node_modules",
        "target",
        "dist",
        "build",
        ".vite",
        "_TEMP",
    ]
    .into_iter()
    .collect();

    fn visit(repo_root: &Path, dir: &Path, skip: &HashSet<&str>, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let rel = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                if skip.contains(name.as_ref())
                    || rel.contains("/target/")
                    || rel.contains("/node_modules/")
                    || rel.contains("/tools/_TEMP/")
                {
                    continue;
                }
                visit(repo_root, &path, skip, out);
            } else if kind.is_file() && SCAN_EXTS.iter().any(|ext| name.ends_with(ext)) {
                out.push(rel);
            }
        }
    }

    if root.exists() {
        visit(repo_root, &root, &skip, &mut out);
    }
    out
}

fn collect_files(repo_root: &Path, args: &Args) -> Vec<String> {
    if let Some(changed) = &args.changed_files
        && !changed.is_empty()
    {
        let mut seen = BTreeSet::new();
        for file in changed {
            let file = normalize_rel(file);
            if exists(repo_root, &file) {
                seen.insert(file);
            }
        }
        return seen.into_iter().collect();
    }

    let mut seen = BTreeSet::new();
    for root in SCAN_ROOTS {
        for file in walk(repo_root, root) {
            seen.insert(file);
        }
    }
    seen.into_iter().collect()
}

fn line_at(text: &str, index: usize) -> usize {
    text[..index].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn lines_around(text: &str, line: usize, radius: usize) -> Vec<ContextLine> {
    let lines: Vec<&str> = text.lines().collect();
    let start = line.saturating_sub(radius + 1);
    let end = usize::min(lines.len(), line + radius);
    lines[start..end]
        .iter()
        .enumerate()
        .map(|(offset, value)| ContextLine {
            line: start + offset + 1,
            text: (*value).to_owned(),
        })
        .collect()
}

fn literal_hits(text: &str, term: &str) -> Vec<LiteralHit> {
    let mut hits = Vec::new();
    let mut pos = 0usize;
    while pos <= text.len() {
        let Some(relative) = text[pos..].find(term) else {
            break;
        };
        let index = pos + relative;
        let line = line_at(text, index);
        hits.push(LiteralHit {
            line,
            context: lines_around(text, line, 4),
        });
        pos = index + usize::max(1, term.len());
    }
    hits
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|item| item == value) {
        values.push(value.to_owned());
    }
}

fn scan_owner(
    owner_name: &str,
    owner: &Owner,
    files: &[String],
    args: &Args,
    repo_root: &Path,
) -> Result<OwnerResult, String> {
    let mut result = OwnerResult {
        owner_name: owner_name.to_owned(),
        owner_id: owner.owner_id.clone(),
        required_missing: Vec::new(),
        required_files_missing: Vec::new(),
        required_found: Vec::new(),
        reference_evidence: Vec::new(),
        forbidden_hits: Vec::new(),
        marker_outside_owner_files: Vec::new(),
        active_evidence_files: Vec::new(),
        reference_evidence_files: Vec::new(),
        responsibilities: owner.responsibilities.clone(),
    };

    let mut evidence_text = String::new();

    for file in files {
        if !exists(repo_root, file) {
            continue;
        }
        let text = read_text(repo_root, file)?;
        let active = is_listed_file(file, &owner.active_files);
        let reference = is_listed_file(file, &owner.reference_files);
        let tool = is_owner_tool_file(file);
        if active || reference {
            if !evidence_text.is_empty() {
                evidence_text.push('\n');
            }
            evidence_text.push_str(&text);
        }

        for marker in &owner.required_markers {
            if !text.contains(marker) {
                continue;
            }
            if active {
                result.required_found.push(Found {
                    marker: marker.clone(),
                    file: file.clone(),
                    kind: "active".to_owned(),
                });
                push_unique(&mut result.active_evidence_files, file);
            } else if reference {
                result.required_found.push(Found {
                    marker: marker.clone(),
                    file: file.clone(),
                    kind: "reference".to_owned(),
                });
                result.reference_evidence.push(Evidence {
                    marker: marker.clone(),
                    file: file.clone(),
                });
                push_unique(&mut result.reference_evidence_files, file);
            } else if !tool {
                result.marker_outside_owner_files.push(Evidence {
                    marker: marker.clone(),
                    file: file.clone(),
                });
            }
        }

        if tool || reference {
            continue;
        }

        for marker in &owner.forbidden_markers {
            for hit in literal_hits(&text, marker) {
                result.forbidden_hits.push(ForbiddenHit {
                    marker: marker.clone(),
                    file: file.clone(),
                    line: hit.line,
                    context: hit.context,
                });
            }
        }
    }

    if args
        .changed_files
        .as_ref()
        .is_none_or(|changed| changed.is_empty())
    {
        for marker in &owner.required_markers {
            if !evidence_text.contains(marker) {
                result.required_missing.push(marker.clone());
            }
        }
        for file in &owner.required_files {
            if !exists(repo_root, file) {
                result.required_files_missing.push(file.clone());
            }
        }
    }

    Ok(result)
}

fn error_required_marker(result: &OwnerResult, marker: &str) -> Value {
    json!({
        "type": "required-marker-missing",
        "ownerName": result.owner_name,
        "ownerId": result.owner_id,
        "marker": marker,
    })
}

fn error_required_file(result: &OwnerResult, file: &str) -> Value {
    json!({
        "type": "required-file-missing",
        "ownerName": result.owner_name,
        "ownerId": result.owner_id,
        "file": file,
    })
}

fn error_outside(result: &OwnerResult, hit: &Evidence) -> Value {
    json!({
        "type": "marker-outside-owner-files",
        "ownerName": result.owner_name,
        "ownerId": result.owner_id,
        "marker": hit.marker,
        "file": hit.file,
    })
}

fn error_forbidden(result: &OwnerResult, hit: &ForbiddenHit) -> Value {
    json!({
        "type": "forbidden-marker",
        "ownerName": result.owner_name,
        "ownerId": result.owner_id,
        "marker": hit.marker,
        "file": hit.file,
        "line": hit.line,
    })
}

fn run_gate(args: &Args, repo_root: &Path, registry: &[OwnerEntry]) -> Result<Value, String> {
    let selected: Vec<(&String, &Owner)> = if let Some(owner) = args.owner.as_deref() {
        registry
            .iter()
            .filter(|entry| entry.name.as_str() == owner)
            .map(|entry| (&entry.name, &entry.owner))
            .collect()
    } else {
        registry
            .iter()
            .map(|entry| (&entry.name, &entry.owner))
            .collect()
    };

    if args.owner.is_some() && selected.is_empty() {
        return Ok(json!({
            "ok": false,
            "fatal": true,
            "errors": [{"type":"unknown-owner","owner":args.owner}],
            "ownerResults": [],
            "summary": {},
        }));
    }

    let files = collect_files(repo_root, args);
    let mut owner_results = Vec::new();
    for (name, owner) in selected {
        owner_results.push(scan_owner(name, owner, &files, args, repo_root)?);
    }

    let mut errors = Vec::new();
    for result in &owner_results {
        for marker in &result.required_missing {
            errors.push(error_required_marker(result, marker));
        }
        for file in &result.required_files_missing {
            errors.push(error_required_file(result, file));
        }
        for hit in &result.marker_outside_owner_files {
            errors.push(error_outside(result, hit));
        }
        for hit in &result.forbidden_hits {
            errors.push(error_forbidden(result, hit));
        }
    }

    let required_missing = errors
        .iter()
        .filter(|error| {
            error.get("type").and_then(Value::as_str) == Some("required-marker-missing")
        })
        .count();
    let required_files_missing = errors
        .iter()
        .filter(|error| error.get("type").and_then(Value::as_str) == Some("required-file-missing"))
        .count();
    let markers_outside = errors
        .iter()
        .filter(|error| {
            error.get("type").and_then(Value::as_str) == Some("marker-outside-owner-files")
        })
        .count();
    let forbidden = errors
        .iter()
        .filter(|error| error.get("type").and_then(Value::as_str) == Some("forbidden-marker"))
        .count();

    Ok(json!({
        "ok": errors.is_empty(),
        "fatal": false,
        "filesScanned": files.len(),
        "strict": args.strict,
        "ownerFilter": args.owner,
        "changedFiles": args.changed_files,
        "errors": errors,
        "ownerResults": owner_results,
        "summary": {
            "ownersChecked": owner_results.len(),
            "errors": errors.len(),
            "requiredMissing": required_missing,
            "requiredFilesMissing": required_files_missing,
            "markersOutsideOwnerFiles": markers_outside,
            "forbiddenMarkers": forbidden,
        }
    }))
}

fn print_help(registry: &[OwnerEntry]) -> String {
    let mut out = String::from(
        "KGW Canonical Global Owner Gate\n\nUsage:\n  cargo run --locked -p xtask -- global-owner-gate\n  cargo run --locked -p xtask -- global-owner-gate --json\n  cargo run --locked -p xtask -- global-owner-gate --strict\n  cargo run --locked -p xtask -- global-owner-gate --owner bridgeInstances --strict\n  cargo run --locked -p xtask -- global-owner-gate --changed-files apps/.../file.js --strict\n\nOwners:\n",
    );
    for entry in registry {
        out.push_str("  ");
        out.push_str(&entry.name);
        out.push_str(" => ");
        out.push_str(&entry.owner.owner_id);
        out.push('\n');
    }
    out.trim_end().to_owned()
}

fn string_array(value: Option<&Value>) -> String {
    let Some(items) = value.and_then(Value::as_array) else {
        return "none".to_owned();
    };
    if items.is_empty() {
        return "none".to_owned();
    }

    items
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_human(result: &Value) -> String {
    let files_scanned = result
        .get("filesScanned")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let summary = result.get("summary").unwrap_or(&Value::Null);
    let owners_checked = summary
        .get("ownersChecked")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let error_count = summary.get("errors").and_then(Value::as_u64).unwrap_or(0);

    let mut out = format!(
        "KGW canonical global owner gate\nmarker: {GATE_MARKER}\nfilesScanned: {files_scanned}\nownersChecked: {owners_checked}\nerrors: {error_count}\n"
    );

    if let Some(owners) = result.get("ownerResults").and_then(Value::as_array) {
        for owner in owners {
            let owner_name = owner
                .get("ownerName")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let owner_id = owner
                .get("ownerId")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let required_found = owner
                .get("requiredFound")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let outside = owner
                .get("markerOutsideOwnerFiles")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let forbidden = owner
                .get("forbiddenHits")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);

            out.push_str(&format!(
                "\n[{owner_name}] {owner_id}\n  requiredFound: {required_found}\n  requiredMissing: {}\n  requiredFilesMissing: {}\n  markerOutsideOwnerFiles: {outside}\n  forbiddenHits: {forbidden}\n  activeEvidenceFiles: {}\n  referenceEvidenceFiles: {}\n",
                string_array(owner.get("requiredMissing")),
                string_array(owner.get("requiredFilesMissing")),
                string_array(owner.get("activeEvidenceFiles")),
                string_array(owner.get("referenceEvidenceFiles")),
            ));
        }
    }

    if let Some(errors) = result.get("errors").and_then(Value::as_array)
        && !errors.is_empty()
    {
        out.push_str("\nOwner gate errors:\n");
        for error in errors.iter().take(100) {
            out.push_str("  - ");
            out.push_str(&serde_json::to_string(error).unwrap_or_else(|_| "{}".to_owned()));
            out.push('\n');
        }

        if errors.len() > 100 {
            out.push_str(&format!("  ... {} more\n", errors.len() - 100));
        }
    }

    out.push('\n');
    if result.get("ok").and_then(Value::as_bool) == Some(true) {
        out.push_str("KGW_OWNER_GATE_PASS");
    } else {
        out.push_str("KGW_OWNER_GATE_CONFLICTS_FOUND");
    }
    out
}

pub fn run_cli(
    args: &mut impl Iterator<Item = String>,
    repo_root: &Path,
) -> Result<RunResult, String> {
    let parsed = parse_args(args);
    let registry = load_registry(repo_root)?;
    if parsed.help {
        return Ok(RunResult {
            output: print_help(&registry),
            code: 0,
        });
    }

    let result = run_gate(&parsed, repo_root, &registry)?;
    let output = if parsed.json {
        serde_json::to_string_pretty(&result)
            .map_err(|error| format!("failed to serialize global owner result: {error}"))?
    } else {
        print_human(&result)
    };
    let code = if parsed.strict && result.get("ok").and_then(Value::as_bool) != Some(true) {
        1
    } else {
        0
    };
    Ok(RunResult { output, code })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rel_matches_legacy_path_rules() {
        assert_eq!(normalize_rel(r".\tools\x.cjs"), "tools/x.cjs");
        assert_eq!(normalize_rel("./apps/a.js"), "apps/a.js");
        assert_eq!(normalize_rel("  config/x.json  "), "config/x.json");
    }

    #[test]
    fn literal_hits_preserve_line_and_context() {
        let text = "a\nb marker\nc\nd\ne\nf\ng marker\nh\n";
        let hits = literal_hits(text, "marker");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line, 2);
        assert_eq!(hits[1].line, 7);
        assert_eq!(hits[0].context[0].line, 1);
    }

    #[test]
    fn registry_keeps_canonical_owner_order() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repository parent");
        let registry = load_registry(root).expect("registry");
        assert_eq!(registry.len(), 23);
        assert_eq!(
            registry.first().map(|entry| entry.name.as_str()),
            Some("settingsButtons")
        );
        assert_eq!(
            registry.last().map(|entry| entry.name.as_str()),
            Some("traceBackendGate")
        );
    }
}
