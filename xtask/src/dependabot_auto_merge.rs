use serde_json::Value;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::Path;
use std::process::Command;

const REQUIRED_REPOSITORY: &str = "KaspaPulse/kaspa-gateway-rust";
const MAX_API_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Context {
    repository: String,
    pr_number: u64,
    expected_sha: String,
}

impl Context {
    fn new(repository: &str, pr_number: &str, expected_sha: &str) -> Result<Self, String> {
        if repository != REQUIRED_REPOSITORY {
            return Err("GH_REPO does not match the protected Kaspa Gateway repository".to_owned());
        }
        let pr_number = pr_number
            .parse::<u64>()
            .map_err(|_| "PR_NUMBER must be a positive decimal integer".to_owned())?;
        if pr_number == 0 {
            return Err("PR_NUMBER must be greater than zero".to_owned());
        }
        if expected_sha.len() != 40
            || !expected_sha
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(
                "EXPECTED_SHA must be exactly 40 lowercase hexadecimal characters".to_owned(),
            );
        }
        Ok(Self {
            repository: repository.to_owned(),
            pr_number,
            expected_sha: expected_sha.to_owned(),
        })
    }
}

pub fn from_environment() -> Result<Context, String> {
    let repository = std::env::var("GH_REPO").map_err(|_| "GH_REPO is required".to_owned())?;
    let pr_number = std::env::var("PR_NUMBER").map_err(|_| "PR_NUMBER is required".to_owned())?;
    let expected_sha =
        std::env::var("EXPECTED_SHA").map_err(|_| "EXPECTED_SHA is required".to_owned())?;
    if std::env::var_os("GH_TOKEN").is_none_or(|value| value.is_empty()) {
        return Err("GH_TOKEN is required but is never printed or persisted".to_owned());
    }
    Context::new(&repository, &pr_number, &expected_sha)
}

fn api_args(context: &Context) -> Vec<String> {
    vec![
        "api".to_owned(),
        format!("repos/{}/pulls/{}", context.repository, context.pr_number),
    ]
}

fn merge_args(context: &Context) -> Vec<String> {
    vec![
        "pr".to_owned(),
        "merge".to_owned(),
        context.pr_number.to_string(),
        "--repo".to_owned(),
        context.repository.clone(),
        "--auto".to_owned(),
        "--squash".to_owned(),
        "--match-head-commit".to_owned(),
        context.expected_sha.clone(),
    ]
}

fn exact_string<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str)
}

fn validate_pr(value: &Value, context: &Context) -> Result<(), String> {
    if exact_string(value, "/state") != Some("open") {
        return Err("pull request is not open".to_owned());
    }
    if value.pointer("/draft").and_then(Value::as_bool) != Some(false) {
        return Err("pull request is draft or has invalid draft metadata".to_owned());
    }
    if exact_string(value, "/user/login") != Some("dependabot[bot]")
        || exact_string(value, "/user/type") != Some("Bot")
    {
        return Err("pull request author is not the verified Dependabot bot".to_owned());
    }
    if exact_string(value, "/head/repo/full_name") != Some(context.repository.as_str()) {
        return Err("pull request head repository does not match GH_REPO".to_owned());
    }
    if exact_string(value, "/base/ref") != Some("main") {
        return Err("pull request base is not main".to_owned());
    }
    if exact_string(value, "/head/sha") != Some(context.expected_sha.as_str()) {
        return Err("pull request head SHA does not match EXPECTED_SHA".to_owned());
    }
    Ok(())
}

pub fn run(context: Context) -> Result<(), String> {
    let api = Command::new("gh")
        .args(api_args(&context))
        .output()
        .map_err(|error| format!("start gh api: {error}"))?;
    if !api.status.success() {
        return Err(format!("gh api failed with status {}", api.status));
    }
    if api.stdout.len() > MAX_API_BYTES {
        return Err("gh api response exceeds the bounded JSON limit".to_owned());
    }
    let value: Value = serde_json::from_slice(&api.stdout)
        .map_err(|error| format!("gh api returned invalid pull request JSON: {error}"))?;
    validate_pr(&value, &context)?;

    let status = Command::new("gh")
        .args(merge_args(&context))
        .status()
        .map_err(|error| format!("start gh pr merge: {error}"))?;
    if !status.success() {
        return Err(format!("gh pr merge failed with status {status}"));
    }
    println!(
        "DEPENDABOT_AUTO_MERGE=ENABLED; PR={}; EXPECTED_HEAD_VERIFIED=YES",
        context.pr_number
    );
    Ok(())
}

#[cfg(test)]
fn verify_workflow(root: &Path) -> Result<(), String> {
    let path = root.join(".github/workflows/dependabot-auto-merge.yml");
    let workflow =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    for required in [
        "github.repository == 'KaspaPulse/kaspa-gateway-rust'",
        "github.event.pull_request.user.login == 'dependabot[bot]'",
        "github.event.pull_request.head.repo.full_name == github.repository",
        "!github.event.pull_request.draft",
        "version-update:semver-patch",
        "version-update:semver-minor",
        "!contains(steps.metadata.outputs.dependency-names, 'duckdb')",
        "cargo run --locked -p xtask --bin kgw-dependabot-auto-merge",
    ] {
        if !workflow.contains(required) {
            return Err(format!(
                "Dependabot workflow lost required contract: {required}"
            ));
        }
    }
    for forbidden in ["shell: bash", "jq -e", "gh api", "gh pr merge"] {
        if workflow.contains(forbidden) {
            return Err(format!(
                "Dependabot workflow retains superseded shell orchestration: {forbidden}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn context() -> Context {
        Context::new(
            REQUIRED_REPOSITORY,
            "123",
            "0123456789abcdef0123456789abcdef01234567",
        )
        .unwrap()
    }

    fn valid_pr() -> Value {
        json!({
            "state": "open",
            "draft": false,
            "user": {"login": "dependabot[bot]", "type": "Bot"},
            "head": {"repo": {"full_name": REQUIRED_REPOSITORY}, "sha": context().expected_sha},
            "base": {"ref": "main"}
        })
    }

    #[test]
    fn accepts_only_the_exact_protected_pull_request_identity() {
        validate_pr(&valid_pr(), &context()).unwrap();
        let mut cases = Vec::new();
        let mut value = valid_pr();
        value["state"] = json!("closed");
        cases.push(value);
        let mut value = valid_pr();
        value["draft"] = json!(true);
        cases.push(value);
        let mut value = valid_pr();
        value["user"]["login"] = json!("someone");
        cases.push(value);
        let mut value = valid_pr();
        value["user"]["type"] = json!("User");
        cases.push(value);
        let mut value = valid_pr();
        value["head"]["repo"]["full_name"] = json!("fork/repo");
        cases.push(value);
        let mut value = valid_pr();
        value["base"]["ref"] = json!("other");
        cases.push(value);
        let mut value = valid_pr();
        value["head"]["sha"] = json!("ffffffffffffffffffffffffffffffffffffffff");
        cases.push(value);
        for value in cases {
            assert!(validate_pr(&value, &context()).is_err(), "{value}");
        }
    }

    #[test]
    fn context_is_fail_closed() {
        assert!(Context::new("other/repo", "1", &context().expected_sha).is_err());
        assert!(Context::new(REQUIRED_REPOSITORY, "0", &context().expected_sha).is_err());
        assert!(Context::new(REQUIRED_REPOSITORY, "abc", &context().expected_sha).is_err());
        assert!(Context::new(REQUIRED_REPOSITORY, "1", "ABCDEF").is_err());
        assert!(
            Context::new(
                REQUIRED_REPOSITORY,
                "1",
                "0123456789ABCDEF0123456789abcdef01234567"
            )
            .is_err()
        );
    }

    #[test]
    fn gh_arguments_preserve_existing_api_and_protected_merge_semantics() {
        let context = context();
        assert_eq!(
            api_args(&context),
            vec!["api", "repos/KaspaPulse/kaspa-gateway-rust/pulls/123"]
        );
        assert_eq!(
            merge_args(&context),
            vec![
                "pr",
                "merge",
                "123",
                "--repo",
                REQUIRED_REPOSITORY,
                "--auto",
                "--squash",
                "--match-head-commit",
                "0123456789abcdef0123456789abcdef01234567"
            ]
        );
    }

    #[test]
    fn malformed_or_incomplete_pr_json_is_rejected() {
        for value in [json!(null), json!({}), json!({"state":"open"})] {
            assert!(validate_pr(&value, &context()).is_err());
        }
    }

    #[test]
    fn current_workflow_delegates_only_to_native_rust_owner() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        verify_workflow(root).unwrap();
    }
}
