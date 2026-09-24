use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SOURCE_DEBT_MANIFEST: &str = "config/owned-language-migration-debt.txt";
const EXECUTION_DEBT_MANIFEST: &str = "config/non-rust-execution-migration-debt.txt";
const EXCEPTION_MANIFEST: &str = "config/owned-language-exceptions.txt";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Migration,
    Strict,
}

#[derive(Debug)]
struct ExceptionEntry {
    category: String,
    reason: String,
}

pub fn run_language_policy(mode: Mode, print_inventory: bool) -> Result<(), String> {
    let root = repo_root()?;
    let files = repository_files(&root)?;
    let source_debt = load_path_manifest(&root.join(SOURCE_DEBT_MANIFEST))?;
    let execution_debt = load_path_manifest(&root.join(EXECUTION_DEBT_MANIFEST))?;
    let exceptions = load_exceptions(&root.join(EXCEPTION_MANIFEST))?;
    let mut problems = Vec::new();
    for path in source_debt.intersection(&execution_debt) {
        problems.push(format!("path appears in both debt manifests: {path}"));
    }
    for path in source_debt.iter().chain(execution_debt.iter()) {
        if exceptions.contains_key(path) {
            problems.push(format!(
                "path is both migration debt and an exception: {path}"
            ));
        }
    }

    let mut rust_sources = Vec::new();
    let mut source_candidates = BTreeMap::new();
    let mut execution_candidates = BTreeMap::new();

    for path in &files {
        let absolute = root.join(path);
        if is_rust_source(path) {
            rust_sources.push(path.clone());
        } else if let Some(language) = non_rust_source_language(path, &absolute) {
            source_candidates.insert(path.clone(), language);
        }
        if let Some(reason) = non_rust_execution_reference(path, &absolute) {
            execution_candidates.insert(path.clone(), reason);
        }
    }

    let current_source_debt = intersection_keys(&source_candidates, &source_debt);
    let current_execution_debt = intersection_keys(&execution_candidates, &execution_debt);
    let unapproved_sources = difference_keys(&source_candidates, &source_debt, &exceptions);
    let unapproved_execution = difference_keys(&execution_candidates, &execution_debt, &exceptions);

    for path in source_debt.difference(&source_candidates.keys().cloned().collect()) {
        problems.push(format!("stale source-debt entry: {path}"));
    }
    for path in execution_debt.difference(&execution_candidates.keys().cloned().collect()) {
        problems.push(format!("stale execution-debt entry: {path}"));
    }

    let mut active_exceptions = BTreeMap::new();
    for (path, entry) in &exceptions {
        if source_candidates.contains_key(path) || execution_candidates.contains_key(path) {
            active_exceptions.insert(path.clone(), entry);
        } else {
            problems.push(format!("stale exception entry: {path}"));
        }
    }

    println!("RUST_SOURCE_INVENTORY={}", rust_sources.len());
    println!(
        "NON_RUST_OWNED_PROGRAMMING_SOURCE_COUNT={}",
        current_source_debt.len() + unapproved_sources.len()
    );
    println!(
        "BASELINED_SOURCE_MIGRATION_DEBT={}",
        current_source_debt.len()
    );
    println!("UNAPPROVED_NON_RUST_SOURCE={}", unapproved_sources.len());
    println!(
        "NON_RUST_EXECUTION_REFERENCE_COUNT={}",
        current_execution_debt.len() + unapproved_execution.len()
    );
    println!(
        "BASELINED_EXECUTION_MIGRATION_DEBT={}",
        current_execution_debt.len()
    );
    println!(
        "UNAPPROVED_NON_RUST_EXECUTION_REFERENCE={}",
        unapproved_execution.len()
    );
    println!("TECHNICAL_EXCEPTIONS={}", active_exceptions.len());

    if print_inventory {
        print_source_inventory(
            &source_candidates,
            &source_debt,
            &exceptions,
            "SOURCE_CANDIDATE",
        );
        print_source_inventory(
            &execution_candidates,
            &execution_debt,
            &exceptions,
            "EXECUTION_REFERENCE",
        );
        for (path, entry) in &active_exceptions {
            println!("EXCEPTION={}|{}|{}", entry.category, path, entry.reason);
        }
    }

    for path in &unapproved_sources {
        problems.push(format!("unapproved non-Rust source: {path}"));
    }
    for path in &unapproved_execution {
        problems.push(format!("unapproved non-Rust execution reference: {path}"));
    }

    if mode == Mode::Strict {
        if !current_source_debt.is_empty() {
            problems.push(format!(
                "strict mode requires zero owned non-Rust source debt; {} remain",
                current_source_debt.len()
            ));
        }
        if !current_execution_debt.is_empty() {
            problems.push(format!(
                "strict mode requires zero non-Rust execution debt; {} remain",
                current_execution_debt.len()
            ));
        }
    }

    if problems.is_empty() {
        println!("RUST_POLICY_GUARD=PASS");
        if mode == Mode::Strict {
            println!("OWNED_PROGRAMMING_IMPLEMENTATION=100_PERCENT_RUST");
        } else {
            println!("OWNED_PROGRAMMING_IMPLEMENTATION=MIGRATION_IN_PROGRESS");
        }
        Ok(())
    } else {
        println!("RUST_POLICY_GUARD=FAIL");
        for problem in &problems {
            println!("POLICY_VIOLATION={problem}");
        }
        Err(format!("{} language-policy violation(s)", problems.len()))
    }
}
fn repo_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest has no repository parent".to_owned())
}

fn repository_files(root: &Path) -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-co", "--exclude-standard", "-z"])
        .output()
        .map_err(|error| format!("failed to execute git ls-files: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let text = String::from_utf8(output.stdout)
        .map_err(|_| "git ls-files returned a non-UTF-8 repository path".to_owned())?;
    Ok(text
        .split('\0')
        .filter(|value| !value.is_empty())
        .map(normalize_path)
        .collect())
}

fn load_path_manifest(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut paths = BTreeSet::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let normalized = normalize_path(line);
        if !paths.insert(normalized.clone()) {
            return Err(format!(
                "{}:{} duplicates path {normalized}",
                path.display(),
                index + 1
            ));
        }
    }
    Ok(paths)
}

fn load_exceptions(path: &Path) -> Result<BTreeMap<String, ExceptionEntry>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut entries = BTreeMap::new();
    let allowed = [
        "GENERATED",
        "VENDORED_THIRD_PARTY",
        "PLATFORM_REQUIRED_ADAPTER",
    ];

    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, '|');
        let category = parts.next().unwrap_or_default().trim();
        let source_path = parts.next().unwrap_or_default().trim();
        let reason = parts.next().unwrap_or_default().trim();
        if !allowed.contains(&category) || source_path.is_empty() || reason.is_empty() {
            return Err(format!(
                "{}:{} must be CATEGORY|PATH|REASON with an allowed category",
                path.display(),
                index + 1
            ));
        }

        let source_path = normalize_path(source_path);
        if entries
            .insert(
                source_path.clone(),
                ExceptionEntry {
                    category: category.to_owned(),
                    reason: reason.to_owned(),
                },
            )
            .is_some()
        {
            return Err(format!(
                "{}:{} duplicates exception path {source_path}",
                path.display(),
                index + 1
            ));
        }
    }
    Ok(entries)
}

fn is_rust_source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("rs"))
}

fn non_rust_source_language(path: &str, absolute: &Path) -> Option<String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let language = match extension.as_str() {
        "py" | "pyw" => Some("Python"),
        "js" | "mjs" | "cjs" | "jsx" => Some("JavaScript"),
        "ts" | "tsx" | "mts" | "cts" => Some("TypeScript"),
        "sh" | "bash" | "zsh" | "fish" | "command" => Some("Shell"),
        "ps1" | "psm1" => Some("PowerShell"),
        "bat" | "cmd" => Some("WindowsBatch"),
        "go" => Some("Go"),
        "c" | "h" => Some("C"),
        "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx" => Some("C++"),
        "rb" => Some("Ruby"),
        "pl" | "pm" => Some("Perl"),
        "php" => Some("PHP"),
        "lua" => Some("Lua"),
        "java" => Some("Java"),
        "kt" | "kts" => Some("Kotlin"),
        "scala" | "sc" => Some("Scala"),
        "cs" => Some("CSharp"),
        "fs" | "fsx" => Some("FSharp"),
        "swift" => Some("Swift"),
        "dart" => Some("Dart"),
        "zig" => Some("Zig"),
        "nim" | "nims" => Some("Nim"),
        "ex" | "exs" => Some("Elixir"),
        "erl" | "hrl" => Some("Erlang"),
        "clj" | "cljs" | "cljc" => Some("Clojure"),
        "groovy" | "gradle" => Some("Groovy"),
        "r" => Some("R"),
        "jl" => Some("Julia"),
        "hs" | "lhs" => Some("Haskell"),
        "ml" | "mli" => Some("OCaml"),
        "asm" | "s" => Some("Assembly"),
        "v" | "vsh" => Some("V"),
        "sol" => Some("Solidity"),
        "move" => Some("Move"),
        "wat" => Some("WebAssemblyText"),
        _ => None,
    };
    if let Some(language) = language {
        return Some(language.to_owned());
    }

    shebang_language(absolute)
}

fn shebang_language(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let first_line = bytes.split(|byte| *byte == b'\n').next()?;
    let first_line = String::from_utf8_lossy(first_line).to_ascii_lowercase();
    if !first_line.starts_with("#!") {
        return None;
    }
    let interpreters = [
        ("python", "Python"),
        ("node", "JavaScript"),
        ("deno", "JavaScript"),
        ("bash", "Shell"),
        ("/sh", "Shell"),
        ("zsh", "Shell"),
        ("fish", "Shell"),
        ("pwsh", "PowerShell"),
        ("powershell", "PowerShell"),
        ("ruby", "Ruby"),
        ("perl", "Perl"),
        ("php", "PHP"),
        ("lua", "Lua"),
        ("julia", "Julia"),
        ("rscript", "R"),
    ];
    interpreters
        .iter()
        .find(|(needle, _)| first_line.contains(needle))
        .map(|(_, language)| (*language).to_owned())
}

fn non_rust_execution_reference(path: &str, absolute: &Path) -> Option<String> {
    let lower_path = path.to_ascii_lowercase();
    let targeted = lower_path.starts_with(".github/workflows/")
        || lower_path == ".github/dependabot.yml"
        || lower_path.ends_with("/package.json")
        || lower_path.ends_with("/package-lock.json")
        || lower_path.ends_with("tauri.conf.json")
        || lower_path.ends_with(".html");
    if !targeted {
        return None;
    }

    let content = fs::read_to_string(absolute).ok()?.to_ascii_lowercase();
    non_rust_execution_content(&lower_path, &content)
}

fn non_rust_execution_content(lower_path: &str, content: &str) -> Option<String> {
    if lower_path.ends_with("/package.json") || lower_path.ends_with("/package-lock.json") {
        return Some("Node package manifest or lockfile".to_owned());
    }
    if lower_path.ends_with(".html") {
        return content
            .contains("<script")
            .then(|| "HTML script execution/reference".to_owned());
    }
    if lower_path == ".github/dependabot.yml" && content.contains("package-ecosystem: \"npm\"") {
        return Some("npm dependency automation".to_owned());
    }
    if lower_path.starts_with(".github/workflows/")
        && (content.contains("run: |")
            || content.contains("run: >")
            || content.contains("shell: bash")
            || content.contains("shell: sh")
            || content.contains("shell: pwsh")
            || content.contains("shell: powershell"))
    {
        return Some("GitHub Actions workflow embeds non-Rust shell/program logic".to_owned());
    }

    let tokens = [
        "python ",
        "python3 ",
        "node ",
        "npm ",
        "pnpm ",
        "yarn ",
        "powershell",
        "pwsh ",
        "bash ",
        ".py",
        ".js",
        ".mjs",
        ".cjs",
        ".ps1",
        ".sh",
        "setup-node",
    ];
    if tokens.iter().any(|token| content.contains(token)) {
        return Some("declarative file invokes or wires non-Rust implementation".to_owned());
    }
    None
}

fn normalize_path(path: &str) -> String {
    let normalized = path.trim().replace('\\', "/");
    normalized.trim_start_matches("./").to_owned()
}

fn intersection_keys<T>(
    candidates: &BTreeMap<String, T>,
    allowed: &BTreeSet<String>,
) -> BTreeSet<String> {
    candidates
        .keys()
        .filter(|path| allowed.contains(*path))
        .cloned()
        .collect()
}

fn difference_keys<T>(
    candidates: &BTreeMap<String, T>,
    debt: &BTreeSet<String>,
    exceptions: &BTreeMap<String, ExceptionEntry>,
) -> BTreeSet<String> {
    candidates
        .keys()
        .filter(|path| !debt.contains(*path) && !exceptions.contains_key(*path))
        .cloned()
        .collect()
}

fn print_source_inventory<T: std::fmt::Display>(
    candidates: &BTreeMap<String, T>,
    debt: &BTreeSet<String>,
    exceptions: &BTreeMap<String, ExceptionEntry>,
    prefix: &str,
) {
    for (path, kind) in candidates {
        let classification = if debt.contains(path) {
            "MIGRATION_DEBT"
        } else if exceptions.contains_key(path) {
            "EXCEPTION"
        } else {
            "UNAPPROVED"
        };
        println!("{prefix}={classification}|{kind}|{path}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classifies_common_owned_languages() {
        let root = Path::new("does-not-need-to-exist");
        assert_eq!(
            non_rust_source_language("tool.py", root).as_deref(),
            Some("Python")
        );
        assert_eq!(
            non_rust_source_language("ui.mjs", root).as_deref(),
            Some("JavaScript")
        );
        assert_eq!(
            non_rust_source_language("helper.ps1", root).as_deref(),
            Some("PowerShell")
        );
        assert_eq!(non_rust_source_language("src/lib.rs", root), None);
        assert_eq!(non_rust_source_language("workflow.yml", root), None);
    }

    #[test]
    fn normalizes_repository_paths() {
        assert_eq!(normalize_path(".\\tools\\gate.py"), "tools/gate.py");
        assert_eq!(normalize_path("./tools/gate.py"), "tools/gate.py");
    }

    #[test]
    fn strict_and_migration_modes_are_distinct() {
        assert_ne!(Mode::Migration, Mode::Strict);
    }

    #[test]
    fn exact_debt_membership_is_fail_closed() {
        let candidates = BTreeMap::from([
            ("old.py".to_owned(), "Python".to_owned()),
            ("new.js".to_owned(), "JavaScript".to_owned()),
        ]);
        let debt = BTreeSet::from(["old.py".to_owned()]);
        let exceptions = BTreeMap::new();
        let unapproved = difference_keys(&candidates, &debt, &exceptions);
        assert_eq!(unapproved, BTreeSet::from(["new.js".to_owned()]));
    }

    #[test]
    fn rust_source_detection_is_case_insensitive() {
        assert!(is_rust_source("src/lib.rs"));
        assert!(is_rust_source("src/GENERATED.RS"));
        assert!(!is_rust_source("src/lib.js"));
    }

    #[test]
    fn workflow_multiline_shell_logic_is_execution_debt() {
        assert!(
            non_rust_execution_content(
                ".github/workflows/ci.yml",
                "steps:\n  - shell: bash\n    run: |\n      cargo test\n      echo done\n"
            )
            .is_some()
        );
        assert!(
            non_rust_execution_content(
                ".github/workflows/windows.yml",
                "steps:\n  - shell: pwsh\n    run: |\n      cargo test\n"
            )
            .is_some()
        );
    }

    #[test]
    fn simple_rust_only_workflow_command_is_not_script_debt() {
        assert!(
            non_rust_execution_content(
                ".github/workflows/rust.yml",
                "steps:\n  - run: cargo test --locked --workspace\n"
            )
            .is_none()
        );
    }

    #[test]
    fn declarative_html_with_user_visible_node_text_is_not_execution_debt() {
        assert!(
            non_rust_execution_content(
                "frontend/kaspa-node.template.html",
                "<section><span>Node status</span><div>node ready</div></section>"
            )
            .is_none()
        );
    }

    #[test]
    fn html_script_tag_is_execution_debt() {
        assert!(
            non_rust_execution_content(
                "frontend/template.html",
                "<section>safe</section><script>node do-work.js</script>"
            )
            .is_some()
        );
    }
}
