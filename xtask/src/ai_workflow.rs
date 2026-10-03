use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn run(root: &Path) -> Result<String, String> {
    let failures = validate(root);
    if failures.is_empty() {
        Ok(
            "KGW AI workflow gate PASSED\nAGENTS.md, Graphify skill, hooks JSON, graph JSON, and required workflow policies are present."
                .to_owned(),
        )
    } else {
        let mut message = String::from("KGW AI workflow gate FAILED");
        for failure in failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        Err(message)
    }
}

fn validate(root: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let agents = read_required(root, "AGENTS.md", &mut failures);
    let _ = read_required(root, ".codex/skills/graphify/SKILL.md", &mut failures);
    test_json_file(root, ".codex/hooks.json", &mut failures);
    test_json_file(root, "graphify-out/graph.json", &mut failures);

    if !agents.is_empty() {
        let graphify_sections = Regex::new(r"(?im)^##\s+.*graphify.*$")
            .expect("static Graphify heading regex must compile")
            .find_iter(&agents)
            .count();
        if graphify_sections > 1 {
            failures.push("AGENTS.md contains duplicate Graphify sections.".to_owned());
        }

        for (pattern, description) in [
            (
                r"(?is)local-first workflow.*build, test, and run locally",
                "local-first build/test/run rule",
            ),
            (
                r"(?is)display the real native process stdout and stderr",
                "raw stdout and stderr display rule",
            ),
            (
                r"(?is)database directories, ports, runtime state, logs, and operating-system processes isolated by network",
                "network isolation rule",
            ),
            (
                r"(?is)testnet13.*experimental.*disabled by default.*explicit opt-in",
                "testnet13 explicit opt-in rule",
            ),
            (
                r"(?is)definition of done.*formatting.*javascript.*rust.*graphify",
                "definition-of-done test rules",
            ),
            (r"(?is)do not push", "no-push rule"),
        ] {
            require_agents_pattern(&agents, pattern, description, &mut failures);
        }
    }

    failures
}

fn read_required(root: &Path, relative: &str, failures: &mut Vec<String>) -> String {
    let path = root.join(relative);
    match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(_) => {
            failures.push(format!("Missing required file: {relative}"));
            String::new()
        }
    }
}

fn test_json_file(root: &Path, relative: &str, failures: &mut Vec<String>) {
    let text = read_required(root, relative, failures);
    if text.is_empty() {
        return;
    }
    if serde_json::from_str::<Value>(&text).is_err() {
        failures.push(format!("Invalid JSON in {relative}"));
    }
}

fn require_agents_pattern(
    text: &str,
    pattern: &str,
    description: &str,
    failures: &mut Vec<String>,
) {
    let regex = Regex::new(pattern)
        .unwrap_or_else(|error| panic!("invalid static AI workflow regex {pattern:?}: {error}"));
    if !regex.is_match(text) {
        failures.push(format!(
            "AGENTS.md is missing required policy: {description}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn fixture() -> Fixture {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("kgw-ai-workflow-rust-{}-{id}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join(".codex/skills/graphify")).unwrap();
        fs::create_dir_all(root.join("graphify-out")).unwrap();

        let agents = r#"# AGENTS
## Local-first workflow
Build, test, and run locally before external integration.
## Graphify
Use Graphify for broad architecture analysis.
Display the real native process stdout and stderr.
Keep database directories, ports, runtime state, logs, and operating-system processes isolated by network.
Testnet13 remains experimental, disabled by default, and requires explicit opt-in.
## Definition of Done
Formatting, JavaScript, Rust, and Graphify checks are required.
Do not push without authorization.
"#;
        fs::write(root.join("AGENTS.md"), agents).unwrap();
        fs::write(root.join(".codex/skills/graphify/SKILL.md"), "# Graphify\n").unwrap();
        fs::write(root.join(".codex/hooks.json"), "{}\n").unwrap();
        fs::write(root.join("graphify-out/graph.json"), "{}\n").unwrap();

        Fixture { root }
    }

    #[test]
    fn complete_fixture_passes() {
        let fixture = fixture();
        assert!(validate(&fixture.root).is_empty());
    }

    #[test]
    fn missing_hooks_and_graph_match_legacy_failure_shape() {
        let fixture = fixture();
        fs::remove_file(fixture.root.join(".codex/hooks.json")).unwrap();
        fs::remove_file(fixture.root.join("graphify-out/graph.json")).unwrap();

        let failures = validate(&fixture.root);
        assert_eq!(
            failures,
            vec![
                "Missing required file: .codex/hooks.json".to_owned(),
                "Missing required file: graphify-out/graph.json".to_owned(),
            ]
        );
    }

    #[test]
    fn invalid_json_fails_closed() {
        let fixture = fixture();
        fs::write(fixture.root.join(".codex/hooks.json"), "{not-json").unwrap();
        let failures = validate(&fixture.root);
        assert!(
            failures
                .iter()
                .any(|item| { item == "Invalid JSON in .codex/hooks.json" })
        );
    }

    #[test]
    fn duplicate_graphify_section_fails_closed() {
        let fixture = fixture();
        let path = fixture.root.join("AGENTS.md");
        let mut agents = fs::read_to_string(&path).unwrap();
        agents.push_str("\n## Graphify Additional\n");
        fs::write(path, agents).unwrap();
        let failures = validate(&fixture.root);
        assert!(
            failures
                .iter()
                .any(|item| item.contains("duplicate Graphify sections"))
        );
    }

    #[test]
    fn missing_required_agents_policy_fails_closed() {
        let fixture = fixture();
        let path = fixture.root.join("AGENTS.md");
        let agents = fs::read_to_string(&path)
            .unwrap()
            .replace("Do not push without authorization.", "");
        fs::write(path, agents).unwrap();
        let failures = validate(&fixture.root);
        assert!(failures.iter().any(|item| item.contains("no-push rule")));
    }
}
