use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

const OFFICIAL_REPO: &str = "https://github.com/kaspanet/rusty-kaspa.git";
const EXPECTED_NETWORKS: [&str; 3] = ["mainnet", "testnet10", "testnet13"];

#[derive(Clone, Copy, Debug, Default)]
struct Options {
    json: bool,
    strict: bool,
    online: bool,
    offline: bool,
}

#[derive(Debug, Clone)]
struct Finding {
    level: &'static str,
    issue: &'static str,
    extra: Map<String, Value>,
}

impl Finding {
    fn to_value(&self) -> Value {
        let mut value = self.extra.clone();
        value.insert("level".to_owned(), Value::String(self.level.to_owned()));
        value.insert("issue".to_owned(), Value::String(self.issue.to_owned()));
        Value::Object(value)
    }
}

#[derive(Debug, Clone)]
struct CargoAlias {
    alias: String,
    line: String,
    git: String,
    branch: String,
    rev: String,
    package: String,
    optional: bool,
}

#[derive(Debug, Clone)]
struct LockSource {
    name: String,
    url: String,
    branch: String,
    rev: String,
    raw: String,
}
pub struct GateInvocation {
    pub code: i32,
    pub output: String,
}

pub fn run_cli(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<GateInvocation, String> {
    let mut options = Options::default();
    for argument in args {
        match argument.as_str() {
            "--json" => options.json = true,
            "--strict" => options.strict = true,
            "--online" | "--fresh" => options.online = true,
            "--offline" => options.offline = true,
            _ => {}
        }
    }

    let output = evaluate(root, options)?;
    let code = if output.get("ok").and_then(Value::as_bool) == Some(true) {
        0
    } else {
        1
    };
    Ok(GateInvocation {
        code,
        output: render(&output, options.json)?,
    })
}

fn files() -> Value {
    json!({
        "manifest": "config/runtime-repository-bindings.json",
        "nodeCargo": "crates/kaspa-gateway-rk-node/Cargo.toml",
        "bridgeCargo": "crates/kaspa-gateway-rk-bridge/Cargo.toml",
        "cliCargo": "apps/kaspa-gateway-cli/Cargo.toml",
        "coreCargo": "crates/kaspa-gateway-core/Cargo.toml",
        "serviceController": "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
        "officialRuntime": "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs",
        "bridgeRuntime": "crates/kaspa-gateway-rk-bridge/src/lib.rs",
        "cargoLock": "Cargo.lock",
        "applyScript": "tools/kgw_runtime_repository_binding_apply.ps1"
    })
}

fn required_files() -> [&'static str; 9] {
    [
        "config/runtime-repository-bindings.json",
        "crates/kaspa-gateway-rk-node/Cargo.toml",
        "crates/kaspa-gateway-rk-bridge/Cargo.toml",
        "apps/kaspa-gateway-cli/Cargo.toml",
        "crates/kaspa-gateway-core/Cargo.toml",
        "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
        "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs",
        "crates/kaspa-gateway-rk-bridge/src/lib.rs",
        "tools/kgw_runtime_repository_binding_apply.ps1",
    ]
}
fn add(
    findings: &mut Vec<Finding>,
    level: &'static str,
    issue: &'static str,
    extra: impl IntoIterator<Item = (&'static str, Value)>,
) {
    findings.push(Finding {
        level,
        issue,
        extra: extra
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    });
}

fn has_errors(findings: &[Finding]) -> bool {
    findings.iter().any(|finding| finding.level == "error")
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("failed to read {relative}: {error}"))
}

fn load_manifest(root: &Path) -> Result<Value, String> {
    let bytes = fs::read(root.join("config/runtime-repository-bindings.json"))
        .map_err(|error| format!("manifest read failed: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("manifest JSON invalid: {error}"))
}

fn string<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn network<'a>(networks: &'a Map<String, Value>, name: &str) -> Option<&'a Value> {
    networks.get(name)
}

fn quoted_field(line: &str, name: &str) -> String {
    let Some(start) = line.find(name) else {
        return String::new();
    };
    let after_name = &line[start + name.len()..];
    let Some(eq) = after_name.find('=') else {
        return String::new();
    };
    let value = after_name[eq + 1..].trim_start();
    let Some(value) = value.strip_prefix('"') else {
        return String::new();
    };
    value
        .find('"')
        .map(|end| value[..end].to_owned())
        .unwrap_or_default()
}

fn bool_field(line: &str, name: &str) -> bool {
    let Some(start) = line.find(name) else {
        return false;
    };
    let after_name = &line[start + name.len()..];
    let Some(eq) = after_name.find('=') else {
        return false;
    };
    after_name[eq + 1..].trim_start().starts_with("true")
}
fn parse_cargo_alias(source: &str, alias: &str) -> Option<CargoAlias> {
    let prefix = format!("{alias} = {{");
    let line = source
        .lines()
        .find(|line| line.trim_start().starts_with(&prefix))?
        .trim()
        .to_owned();
    Some(CargoAlias {
        alias: alias.to_owned(),
        git: quoted_field(&line, "git"),
        branch: quoted_field(&line, "branch"),
        rev: quoted_field(&line, "rev"),
        package: quoted_field(&line, "package"),
        optional: bool_field(&line, "optional"),
        line,
    })
}

fn alias_json(parsed: &CargoAlias) -> Value {
    json!({
        "alias": parsed.alias,
        "line": parsed.line,
        "git": parsed.git,
        "branch": parsed.branch,
        "rev": parsed.rev,
        "package": parsed.package,
        "optional": parsed.optional
    })
}

fn normalize_repo(url: &str) -> String {
    url.strip_suffix(".git").unwrap_or(url).to_ascii_lowercase()
}

fn parse_lock_git_sources(lock_text: &str) -> Vec<LockSource> {
    let mut result = Vec::new();
    for block in lock_text.split("[[package]]").skip(1) {
        let name = block
            .lines()
            .find_map(|line| {
                let line = line.trim();
                line.strip_prefix("name = \"")
                    .and_then(|rest| rest.strip_suffix('"'))
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_default();
        let Some(source_line) = block
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with("source = \"git+"))
        else {
            continue;
        };
        let Some(source) = source_line
            .strip_prefix("source = \"")
            .and_then(|rest| rest.strip_suffix('"'))
        else {
            continue;
        };
        let Some(git_source) = source.strip_prefix("git+") else {
            continue;
        };
        let Some((base, rev)) = git_source.rsplit_once('#') else {
            continue;
        };
        if rev.len() != 40 || !rev.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            continue;
        }
        let (url, query) = base.split_once('?').unwrap_or((base, ""));
        let branch = query
            .split('&')
            .find_map(|pair| pair.strip_prefix("branch="))
            .unwrap_or_default()
            .to_owned();
        result.push(LockSource {
            name: name.clone(),
            url: url.to_owned(),
            branch,
            rev: rev.to_owned(),
            raw: source_line.to_owned(),
        });
    }
    result
}
fn lock_source_json(source: &LockSource) -> Value {
    json!({
        "name": source.name,
        "url": source.url,
        "branch": source.branch,
        "rev": source.rev,
        "raw": source.raw
    })
}

fn cargo_alias_row(
    scope: &str,
    alias: &str,
    expected_package: &str,
    network_name: &str,
    parsed: Option<&CargoAlias>,
) -> Value {
    json!({
        "scope": scope,
        "alias": alias,
        "expectedPackage": expected_package,
        "network": network_name,
        "parsed": parsed.map(alias_json).unwrap_or(Value::Null)
    })
}

fn require_contains(
    findings: &mut Vec<Finding>,
    file_label: &str,
    source: &str,
    needle: String,
    description: &str,
) {
    if !source.contains(&needle) {
        add(
            findings,
            "error",
            "rust-mapping-missing",
            [
                ("fileLabel", json!(file_label)),
                ("description", json!(description)),
                ("needle", json!(needle)),
            ],
        );
    }
}

fn remote_head(repo: &str, branch: &str) -> Value {
    let output = Command::new("git")
        .args(["ls-remote", repo, &format!("refs/heads/{branch}")])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let latest = stdout
                .split_whitespace()
                .next()
                .filter(|value| value.len() == 40)
                .unwrap_or_default()
                .to_owned();
            json!({
                "ok": output.status.success() && !latest.is_empty(),
                "latestRev": latest,
                "status": output.status.code(),
                "stderr": stderr,
                "stdout": stdout
            })
        }
        Err(error) => json!({
            "ok": false,
            "latestRev": "",
            "status": Value::Null,
            "stderr": error.to_string(),
            "stdout": ""
        }),
    }
}
fn evaluate(root: &Path, options: Options) -> Result<Value, String> {
    let mut findings = Vec::new();
    let file_map = files();

    for relative in required_files() {
        if !root.join(relative).is_file() {
            add(
                &mut findings,
                "error",
                "missing-required-file",
                [("file", json!(relative))],
            );
        }
    }

    let mut manifest = json!({});
    let mut networks = Map::new();
    if !has_errors(&findings) {
        manifest = load_manifest(root)?;
        networks = manifest
            .get("networks")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
    }

    for name in EXPECTED_NETWORKS {
        let Some(binding) = network(&networks, name) else {
            add(
                &mut findings,
                "error",
                "missing-network-binding",
                [("net", json!(name))],
            );
            continue;
        };
        if string(binding, "repo").is_empty() {
            add(
                &mut findings,
                "error",
                "missing-repo",
                [("net", json!(name))],
            );
        }
        if string(binding, "branch").is_empty() {
            add(
                &mut findings,
                "error",
                "missing-branch",
                [("net", json!(name))],
            );
        }
        let rev = string(binding, "rev");
        if rev.len() != 40 || !rev.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            add(
                &mut findings,
                "error",
                "missing-or-invalid-rev",
                [("net", json!(name)), ("rev", json!(rev))],
            );
        }
        if string(binding, "family").is_empty() {
            add(
                &mut findings,
                "error",
                "missing-family",
                [("net", json!(name))],
            );
        }
    }

    if let Some(mainnet) = network(&networks, "mainnet")
        && string(mainnet, "repo") != OFFICIAL_REPO
    {
        add(
            &mut findings,
            "error",
            "mainnet-must-use-official-kaspanet-repo",
            [
                ("expected", json!(OFFICIAL_REPO)),
                ("actual", json!(string(mainnet, "repo"))),
            ],
        );
    }
    for name in ["mainnet", "testnet10"] {
        if let Some(binding) = network(&networks, name) {
            if string(binding, "repo") != OFFICIAL_REPO {
                add(
                    &mut findings,
                    "error",
                    "stable-network-must-use-official-kaspanet-repo",
                    [
                        ("net", json!(name)),
                        ("expected", json!(OFFICIAL_REPO)),
                        ("actual", json!(string(binding, "repo"))),
                    ],
                );
            }
            if string(binding, "family") != "mainline" {
                add(
                    &mut findings,
                    "error",
                    "stable-network-family-must-be-mainline",
                    [
                        ("net", json!(name)),
                        ("actual", json!(string(binding, "family"))),
                    ],
                );
            }
        }
    }

    let actual_networks: BTreeSet<String> = networks.keys().cloned().collect();
    let expected_networks: BTreeSet<String> = EXPECTED_NETWORKS
        .into_iter()
        .map(ToOwned::to_owned)
        .collect();
    if actual_networks != expected_networks {
        add(
            &mut findings,
            "error",
            "unexpected-network-binding",
            std::iter::empty(),
        );
    }

    if let Some(tn13) = network(&networks, "testnet13") {
        if string(tn13, "repo") != OFFICIAL_REPO || string(tn13, "branch") != "dagknight" {
            add(
                &mut findings,
                "error",
                "tn13-must-use-official-dagknight",
                std::iter::empty(),
            );
        }
        if string(tn13, "family") != "tn13" {
            add(
                &mut findings,
                "error",
                "testnet13-family-must-be-tn13",
                [("actual", json!(string(tn13, "family")))],
            );
        }
        if tn13.get("experimental") != Some(&Value::Bool(true)) {
            add(
                &mut findings,
                "error",
                "testnet13-must-be-marked-experimental",
                [(
                    "actual",
                    tn13.get("experimental").cloned().unwrap_or(Value::Null),
                )],
            );
        }
        if tn13.get("enabledByDefault") != Some(&Value::Bool(false)) {
            add(
                &mut findings,
                "error",
                "testnet13-must-be-disabled-by-default",
                [(
                    "actual",
                    tn13.get("enabledByDefault").cloned().unwrap_or(Value::Null),
                )],
            );
        }
    }
    if let (Some(mainnet), Some(tn10)) = (
        network(&networks, "mainnet"),
        network(&networks, "testnet10"),
    ) {
        for key in ["repo", "branch", "rev", "feature"] {
            if mainnet.get(key) != tn10.get(key) {
                add(
                    &mut findings,
                    "error",
                    "mainnet-testnet10-stable-binding-mismatch",
                    [
                        ("key", json!(key)),
                        ("mainnet", mainnet.get(key).cloned().unwrap_or(Value::Null)),
                        ("testnet10", tn10.get(key).cloned().unwrap_or(Value::Null)),
                    ],
                );
            }
        }
    }

    let mut cargo_aliases = Vec::new();
    let mut lock_sources = Vec::new();
    let mut remote_checks = Vec::new();

    if !has_errors(&findings) {
        let node_cargo = read(root, "crates/kaspa-gateway-rk-node/Cargo.toml")?;
        let bridge_cargo = read(root, "crates/kaspa-gateway-rk-bridge/Cargo.toml")?;
        let cli_cargo = read(root, "apps/kaspa-gateway-cli/Cargo.toml")?;
        let core_cargo = read(root, "crates/kaspa-gateway-core/Cargo.toml")?;
        let apply_script = read(root, "tools/kgw_runtime_repository_binding_apply.ps1")?;

        let mut aliases: Vec<(&str, String, &str, &str, &str)> = Vec::new();
        for (family, network_name) in [("mainline", "mainnet"), ("tn13", "testnet13")] {
            for (alias, package) in [
                ("kaspad-lib", "kaspad"),
                ("kaspa-core", "kaspa-core"),
                ("kaspa-grpc-client", "kaspa-grpc-client"),
                ("kaspa-rpc-core", "kaspa-rpc-core"),
                ("kaspa-utils", "kaspa-utils"),
                ("kaspa-wrpc-server", "kaspa-wrpc-server"),
            ] {
                aliases.push((
                    "node",
                    format!("{alias}-{family}"),
                    package,
                    network_name,
                    node_cargo.as_str(),
                ));
            }
            for package in [
                "kaspa-grpc-client",
                "kaspa-rpc-core",
                "kaspa-stratum-bridge",
            ] {
                aliases.push((
                    "bridge",
                    format!("{package}-{family}"),
                    package,
                    network_name,
                    bridge_cargo.as_str(),
                ));
            }
        }
        aliases.push((
            "cli",
            "kaspa-grpc-client-live".to_owned(),
            "kaspa-grpc-client",
            "mainnet",
            cli_cargo.as_str(),
        ));
        aliases.push((
            "cli",
            "kaspa-rpc-core-live".to_owned(),
            "kaspa-rpc-core",
            "mainnet",
            cli_cargo.as_str(),
        ));

        for (scope, alias, expected_package, network_name, source) in aliases {
            let parsed = parse_cargo_alias(source, &alias);
            cargo_aliases.push(cargo_alias_row(
                scope,
                &alias,
                expected_package,
                network_name,
                parsed.as_ref(),
            ));

            let Some(parsed) = parsed else {
                add(
                    &mut findings,
                    "error",
                    "missing-cargo-alias",
                    [("scope", json!(scope)), ("alias", json!(alias))],
                );
                continue;
            };
            let Some(binding) = network(&networks, network_name) else {
                continue;
            };
            if parsed.package != expected_package {
                add(
                    &mut findings,
                    "error",
                    "cargo-package-mismatch",
                    [
                        ("scope", json!(scope)),
                        ("alias", json!(alias)),
                        ("expected", json!(expected_package)),
                        ("actual", json!(parsed.package)),
                    ],
                );
            }
            if parsed.git != string(binding, "repo") {
                add(
                    &mut findings,
                    "error",
                    "cargo-repo-mismatch",
                    [
                        ("scope", json!(scope)),
                        ("alias", json!(alias)),
                        ("expected", json!(string(binding, "repo"))),
                        ("actual", json!(parsed.git)),
                    ],
                );
            }
            if parsed.rev != string(binding, "rev") {
                add(
                    &mut findings,
                    "error",
                    "cargo-rev-mismatch",
                    [
                        ("scope", json!(scope)),
                        ("alias", json!(alias)),
                        ("expected", json!(string(binding, "rev"))),
                        ("actual", json!(parsed.rev)),
                    ],
                );
            }
            if !parsed.branch.is_empty() {
                add(
                    &mut findings,
                    "error",
                    "cargo-branch-field-forbidden-use-rev-only",
                    [
                        ("scope", json!(scope)),
                        ("alias", json!(alias)),
                        ("branch", json!(parsed.branch)),
                    ],
                );
            }
            if !parsed.optional {
                add(
                    &mut findings,
                    "error",
                    "cargo-alias-must-be-optional",
                    [("scope", json!(scope)), ("alias", json!(alias))],
                );
            }
        }
        let core_alias = parse_cargo_alias(&core_cargo, "kaspa-addresses");
        cargo_aliases.push(cargo_alias_row(
            "core",
            "kaspa-addresses",
            "kaspa-addresses",
            "mainnet",
            core_alias.as_ref(),
        ));
        if let Some(parsed) = core_alias {
            if let Some(mainnet) = network(&networks, "mainnet") {
                if parsed.git != string(mainnet, "repo") {
                    add(
                        &mut findings,
                        "error",
                        "cargo-repo-mismatch",
                        [
                            ("scope", json!("core")),
                            ("alias", json!("kaspa-addresses")),
                            ("expected", json!(string(mainnet, "repo"))),
                            ("actual", json!(parsed.git)),
                        ],
                    );
                }
                if parsed.rev != string(mainnet, "rev") {
                    add(
                        &mut findings,
                        "error",
                        "cargo-rev-mismatch",
                        [
                            ("scope", json!("core")),
                            ("alias", json!("kaspa-addresses")),
                            ("expected", json!(string(mainnet, "rev"))),
                            ("actual", json!(parsed.rev)),
                        ],
                    );
                }
                if !parsed.branch.is_empty() {
                    add(
                        &mut findings,
                        "error",
                        "cargo-branch-field-forbidden-use-rev-only",
                        [
                            ("scope", json!("core")),
                            ("alias", json!("kaspa-addresses")),
                            ("branch", json!(parsed.branch)),
                        ],
                    );
                }
            }
        } else {
            add(
                &mut findings,
                "error",
                "missing-cargo-alias",
                [
                    ("scope", json!("core")),
                    ("alias", json!("kaspa-addresses")),
                ],
            );
        }

        for needle in [
            "nodeAliases",
            "packages.node",
            "bridgeAliases",
            "packages.bridge",
            "kaspa-grpc-client-live",
            "kaspa-rpc-core-live",
            "kaspa-addresses",
            "testnet13",
            r#"Get-BindingByFamily "tn13""#,
        ] {
            if !apply_script.contains(needle) {
                add(
                    &mut findings,
                    "error",
                    "apply-script-missing-manifest-driven-coverage",
                    [("needle", json!(needle))],
                );
            }
        }
        let service_controller = read(
            root,
            "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
        )?;
        let official_runtime = read(
            root,
            "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs",
        )?;
        let bridge_runtime = read(root, "crates/kaspa-gateway-rk-bridge/src/lib.rs")?;

        if let (Some(mainnet), Some(_tn10)) = (
            network(&networks, "mainnet"),
            network(&networks, "testnet10"),
        ) {
            let branch = string(mainnet, "branch");
            let rev = string(mainnet, "rev");
            for (label, source, needle, description) in [
                (
                    "kgw_service_controller.rs",
                    service_controller.as_str(),
                    format!(r#"Self::Mainnet | Self::Testnet10 => "{branch}""#),
                    "stable service branch",
                ),
                (
                    "kgw_service_controller.rs",
                    service_controller.as_str(),
                    format!(r#"Self::Mainnet | Self::Testnet10 => "{rev}""#),
                    "stable service rev",
                ),
                (
                    "official_kaspa_runtime.rs",
                    official_runtime.as_str(),
                    format!(r#"Self::Mainnet | Self::Testnet10 => "{branch}""#),
                    "stable node branch",
                ),
                (
                    "official_kaspa_runtime.rs",
                    official_runtime.as_str(),
                    format!(r#"Self::Mainnet | Self::Testnet10 => "{rev}""#),
                    "stable node rev",
                ),
                (
                    "src/lib.rs bridge runtime",
                    bridge_runtime.as_str(),
                    format!(r#"Self::Mainnet | Self::Testnet10 => "{branch}""#),
                    "stable bridge branch",
                ),
                (
                    "src/lib.rs bridge runtime",
                    bridge_runtime.as_str(),
                    format!(r#"Self::Mainnet | Self::Testnet10 => "{rev}""#),
                    "stable bridge rev",
                ),
                (
                    "official_kaspa_runtime.rs",
                    official_runtime.as_str(),
                    "Self::Mainnet | Self::Testnet10 => KaspaRuntimeFamily::Mainline".to_owned(),
                    "stable node family",
                ),
                (
                    "src/lib.rs bridge runtime",
                    bridge_runtime.as_str(),
                    "Self::Mainnet | Self::Testnet10 => BridgeRuntimeFamily::Mainline".to_owned(),
                    "stable bridge family",
                ),
            ] {
                require_contains(&mut findings, label, source, needle, description);
            }
        }
        if let Some(tn13) = network(&networks, "testnet13") {
            let branch = string(tn13, "branch");
            let rev = string(tn13, "rev");
            for (label, source, needle, description) in [
                (
                    "kgw_service_controller.rs",
                    service_controller.as_str(),
                    format!(r#"Self::Testnet13 => "{branch}""#),
                    "testnet13 service branch",
                ),
                (
                    "kgw_service_controller.rs",
                    service_controller.as_str(),
                    format!(r#"Self::Testnet13 => "{rev}""#),
                    "testnet13 service rev",
                ),
                (
                    "official_kaspa_runtime.rs",
                    official_runtime.as_str(),
                    format!(r#"Self::Testnet13 => "{branch}""#),
                    "testnet13 node branch",
                ),
                (
                    "official_kaspa_runtime.rs",
                    official_runtime.as_str(),
                    format!(r#"Self::Testnet13 => "{rev}""#),
                    "testnet13 node rev",
                ),
                (
                    "src/lib.rs bridge runtime",
                    bridge_runtime.as_str(),
                    format!(r#"Self::Testnet13 => "{branch}""#),
                    "testnet13 bridge branch",
                ),
                (
                    "src/lib.rs bridge runtime",
                    bridge_runtime.as_str(),
                    format!(r#"Self::Testnet13 => "{rev}""#),
                    "testnet13 bridge rev",
                ),
                (
                    "official_kaspa_runtime.rs",
                    official_runtime.as_str(),
                    "Self::Testnet13 => KaspaRuntimeFamily::Tn13".to_owned(),
                    "testnet13 node family",
                ),
                (
                    "src/lib.rs bridge runtime",
                    bridge_runtime.as_str(),
                    "Self::Testnet13 => BridgeRuntimeFamily::Tn13".to_owned(),
                    "testnet13 bridge family",
                ),
            ] {
                require_contains(&mut findings, label, source, needle, description);
            }
        }

        let lock_path = root.join("Cargo.lock");
        if lock_path.is_file() {
            lock_sources = parse_lock_git_sources(&read(root, "Cargo.lock")?);
        } else {
            add(
                &mut findings,
                "error",
                "cargo-lock-required",
                std::iter::empty(),
            );
        }
        for row in &cargo_aliases {
            let Some(network_name) = row.get("network").and_then(Value::as_str) else {
                continue;
            };
            let Some(binding) = network(&networks, network_name) else {
                continue;
            };
            let alias = row.get("alias").and_then(Value::as_str).unwrap_or_default();
            let package = row
                .get("expectedPackage")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let expected_rev = string(binding, "rev");
            let repo_url = string(binding, "repo");
            if !lock_sources.iter().any(|source| {
                source.name == package
                    && normalize_repo(&source.url) == normalize_repo(repo_url)
                    && source.rev == expected_rev
            }) {
                add(
                    &mut findings,
                    "error",
                    "cargo-lock-runtime-package-drift",
                    [
                        ("alias", json!(alias)),
                        ("package", json!(package)),
                        ("expectedRev", json!(expected_rev)),
                    ],
                );
            }
        }

        let approved_revs = [
            network(&networks, "mainnet").map(|value| string(value, "rev")),
            network(&networks, "testnet13").map(|value| string(value, "rev")),
        ];
        for source in &lock_sources {
            if source.url.to_ascii_lowercase().contains("rusty-kaspa")
                && (normalize_repo(&source.url)
                    != normalize_repo("https://github.com/kaspanet/rusty-kaspa")
                    || !approved_revs.iter().flatten().any(|rev| *rev == source.rev))
            {
                add(
                    &mut findings,
                    "error",
                    "cargo-lock-unapproved-runtime-source",
                    [("source", lock_source_json(source))],
                );
            }
        }

        if lock_path.is_file() {
            for network_name in EXPECTED_NETWORKS {
                let Some(binding) = network(&networks, network_name) else {
                    continue;
                };
                let repo_url = string(binding, "repo");
                let expected_rev = string(binding, "rev");
                let matches: Vec<&LockSource> = lock_sources
                    .iter()
                    .filter(|source| normalize_repo(&source.url) == normalize_repo(repo_url))
                    .collect();
                let has_rev = matches
                    .iter()
                    .any(|source| source.rev.eq_ignore_ascii_case(expected_rev));
                if matches.is_empty() {
                    add(
                        &mut findings,
                        "warn",
                        "cargo-lock-no-git-source-for-repo",
                        [("net", json!(network_name)), ("repo", json!(repo_url))],
                    );
                } else if !has_rev {
                    let lock_revs: BTreeSet<String> =
                        matches.iter().map(|source| source.rev.clone()).collect();
                    add(
                        &mut findings,
                        "error",
                        "cargo-lock-does-not-resolve-manifest-rev",
                        [
                            ("net", json!(network_name)),
                            ("expectedRev", json!(expected_rev)),
                            ("lockRevs", json!(lock_revs)),
                        ],
                    );
                }
            }
        }
        if options.online && !options.offline {
            let mut groups: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
            for (name, binding) in &networks {
                let repo_url = string(binding, "repo");
                let branch = string(binding, "branch");
                if repo_url.is_empty() || branch.is_empty() {
                    continue;
                }
                groups
                    .entry((repo_url.to_owned(), branch.to_owned()))
                    .or_default()
                    .push(name.clone());
            }

            for ((repo_url, branch), mut network_names) in groups {
                network_names.sort();
                let remote = remote_head(&repo_url, &branch);
                let mut expected_revs: Vec<String> = network_names
                    .iter()
                    .filter_map(|name| network(&networks, name))
                    .map(|binding| string(binding, "rev").to_owned())
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
                expected_revs.sort();
                let ok = remote.get("ok").and_then(Value::as_bool).unwrap_or(false);
                let latest_rev = remote
                    .get("latestRev")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let fresh = ok
                    && expected_revs.len() == 1
                    && expected_revs[0].eq_ignore_ascii_case(latest_rev);

                let check = json!({
                    "repo": repo_url,
                    "branch": branch,
                    "networks": network_names,
                    "ok": ok,
                    "latestRev": latest_rev,
                    "status": remote.get("status").cloned().unwrap_or(Value::Null),
                    "stderr": remote.get("stderr").cloned().unwrap_or(Value::String(String::new())),
                    "stdout": remote.get("stdout").cloned().unwrap_or(Value::String(String::new())),
                    "expectedRevs": expected_revs,
                    "fresh": fresh
                });

                if !ok {
                    add(
                        &mut findings,
                        if options.strict { "error" } else { "warn" },
                        "remote-ls-remote-failed",
                        [
                            ("repo", json!(repo_url)),
                            ("branch", json!(branch)),
                            (
                                "status",
                                remote.get("status").cloned().unwrap_or(Value::Null),
                            ),
                            (
                                "stderr",
                                remote
                                    .get("stderr")
                                    .cloned()
                                    .unwrap_or(Value::String(String::new())),
                            ),
                        ],
                    );
                } else if !fresh {
                    add(
                        &mut findings,
                        "error",
                        "manifest-rev-is-not-latest-remote-branch-head",
                        [
                            ("repo", json!(repo_url)),
                            ("branch", json!(branch)),
                            ("networks", check["networks"].clone()),
                            ("manifestRevs", check["expectedRevs"].clone()),
                            ("latestRev", json!(latest_rev)),
                        ],
                    );
                }
                remote_checks.push(check);
            }
        }
    }
    let error_count = findings.iter().filter(|item| item.level == "error").count();
    let warning_count = findings.iter().filter(|item| item.level == "warn").count();

    Ok(json!({
        "ok": error_count == 0,
        "verdict": if error_count == 0 {
            "KGW_RUNTIME_REPOSITORY_BINDING_GATE_R21C_PASSED"
        } else {
            "KGW_RUNTIME_REPOSITORY_BINDING_GATE_R21C_FAILED"
        },
        "mode": {
            "strict": options.strict,
            "online": options.online && !options.offline,
            "offline": options.offline
        },
        "files": file_map,
        "networks": Value::Object(networks),
        "cargoAliases": cargo_aliases,
        "lockSources": lock_sources.iter().map(lock_source_json).collect::<Vec<_>>(),
        "remoteChecks": remote_checks,
        "findings": findings.iter().map(Finding::to_value).collect::<Vec<_>>(),
        "errorCount": error_count,
        "warningCount": warning_count
    }))
}

fn render(output: &Value, json_mode: bool) -> Result<String, String> {
    if json_mode {
        return serde_json::to_string_pretty(output)
            .map_err(|error| format!("binding gate JSON serialization failed: {error}"));
    }

    let mut text = String::new();
    text.push_str(
        output
            .get("verdict")
            .and_then(Value::as_str)
            .unwrap_or("KGW_RUNTIME_REPOSITORY_BINDING_GATE_R21C_FAILED"),
    );
    text.push_str(&format!(
        "\nerrors: {}\nwarnings: {}",
        output
            .get("errorCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        output
            .get("warningCount")
            .and_then(Value::as_u64)
            .unwrap_or(0)
    ));
    if let Some(findings) = output.get("findings").and_then(Value::as_array) {
        for finding in findings {
            let level = finding
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_uppercase();
            let issue = finding
                .get("issue")
                .and_then(Value::as_str)
                .unwrap_or_default();
            text.push_str(&format!("\n{level}: {issue} {finding}"));
        }
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    const FIXTURE_FILES: &[&str] = &[
        "config/runtime-repository-bindings.json",
        "crates/kaspa-gateway-rk-node/Cargo.toml",
        "crates/kaspa-gateway-rk-bridge/Cargo.toml",
        "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
        "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs",
        "crates/kaspa-gateway-rk-bridge/src/lib.rs",
        "Cargo.lock",
        "apps/kaspa-gateway-cli/Cargo.toml",
        "crates/kaspa-gateway-core/Cargo.toml",
        "tools/kgw_runtime_repository_binding_apply.ps1",
    ];

    struct Fixture {
        root: std::path::PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn repository_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn fixture() -> Fixture {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("kgw-binding-gate-rust-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source_root = repository_root();
        for relative in FIXTURE_FILES {
            let source = source_root.join(relative);
            let target = root.join(relative);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(source, target).unwrap();
        }
        Fixture { root }
    }

    fn offline_strict(root: &Path) -> Value {
        evaluate(
            root,
            Options {
                strict: true,
                offline: true,
                ..Options::default()
            },
        )
        .unwrap()
    }

    fn issues(output: &Value) -> BTreeSet<String> {
        output
            .get("findings")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|finding| finding.get("issue").and_then(Value::as_str))
            .map(ToOwned::to_owned)
            .collect()
    }

    fn mutate_text(root: &Path, relative: &str, mutate: impl FnOnce(String) -> String) {
        let path = root.join(relative);
        let before = fs::read_to_string(&path).unwrap();
        fs::write(path, mutate(before)).unwrap();
    }
    fn replace_alias_field(line: &str, field: &str, value: &str) -> String {
        let Some(start) = line.find(field) else {
            return line.to_owned();
        };
        let suffix = &line[start + field.len()..];
        let Some(eq) = suffix.find('=') else {
            return line.to_owned();
        };
        let value_start = start + field.len() + eq + 1;
        let tail = &line[value_start..];
        let Some(open_rel) = tail.find('"') else {
            return line.to_owned();
        };
        let open = value_start + open_rel;
        let Some(close_rel) = line[open + 1..].find('"') else {
            return line.to_owned();
        };
        let close = open + 1 + close_rel;
        format!(r#"{}"{value}"{}"#, &line[..open], &line[close + 1..])
    }

    fn mutate_alias_field(root: &Path, relative: &str, alias: &str, field: &str, value: &str) {
        mutate_text(root, relative, |text| {
            let prefix = format!("{alias} = {{");
            text.lines()
                .map(|line| {
                    if line.trim_start().starts_with(&prefix) {
                        replace_alias_field(line, field, value)
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join(
                    "
",
                )
                + "
"
        });
    }

    fn remove_alias(root: &Path, relative: &str, alias: &str) {
        mutate_text(root, relative, |text| {
            let prefix = format!("{alias} = {{");
            text.lines()
                .filter(|line| !line.trim_start().starts_with(&prefix))
                .collect::<Vec<_>>()
                .join(
                    "
",
                )
                + "
"
        });
    }

    fn reject(root: &Path, expected_issue: &str) {
        let output = offline_strict(root);
        assert_eq!(output.get("ok"), Some(&Value::Bool(false)));
        assert!(
            issues(&output).contains(expected_issue),
            "expected issue {expected_issue}; findings={}",
            output["findings"]
        );
    }

    #[test]
    fn aligned_official_source_passes() {
        let fixture = fixture();
        let output = offline_strict(&fixture.root);
        assert_eq!(output.get("ok"), Some(&Value::Bool(true)));
        assert_eq!(output["errorCount"], json!(0));
    }

    #[test]
    fn rpc_alias_revision_drift_is_rejected() {
        let fixture = fixture();
        mutate_alias_field(
            &fixture.root,
            "crates/kaspa-gateway-rk-node/Cargo.toml",
            "kaspa-grpc-client-tn13",
            "rev",
            &"0".repeat(40),
        );
        reject(&fixture.root, "cargo-rev-mismatch");
    }
    #[test]
    fn missing_node_core_alias_is_rejected() {
        let fixture = fixture();
        remove_alias(
            &fixture.root,
            "crates/kaspa-gateway-rk-node/Cargo.toml",
            "kaspa-core-mainline",
        );
        reject(&fixture.root, "missing-cargo-alias");
    }

    #[test]
    fn bridge_source_drift_is_rejected() {
        let fixture = fixture();
        mutate_alias_field(
            &fixture.root,
            "crates/kaspa-gateway-rk-bridge/Cargo.toml",
            "kaspa-grpc-client-mainline",
            "git",
            "https://example.invalid/fork.git",
        );
        reject(&fixture.root, "cargo-repo-mismatch");
    }

    #[test]
    fn cli_revision_drift_is_rejected() {
        let fixture = fixture();
        mutate_alias_field(
            &fixture.root,
            "apps/kaspa-gateway-cli/Cargo.toml",
            "kaspa-rpc-core-live",
            "rev",
            &"0".repeat(40),
        );
        reject(&fixture.root, "cargo-rev-mismatch");
    }

    #[test]
    fn core_revision_drift_is_rejected() {
        let fixture = fixture();
        mutate_alias_field(
            &fixture.root,
            "crates/kaspa-gateway-core/Cargo.toml",
            "kaspa-addresses",
            "rev",
            &"0".repeat(40),
        );
        reject(&fixture.root, "cargo-rev-mismatch");
    }

    #[test]
    fn apply_tool_coverage_drift_is_rejected() {
        let fixture = fixture();
        mutate_text(
            &fixture.root,
            "tools/kgw_runtime_repository_binding_apply.ps1",
            |text| text.replace("bridgeAliases", "bridgeAliasList"),
        );
        reject(
            &fixture.root,
            "apply-script-missing-manifest-driven-coverage",
        );
    }

    #[test]
    fn missing_locked_rpc_package_is_rejected() {
        let fixture = fixture();
        mutate_text(&fixture.root, "Cargo.lock", |text| {
            let mut output = String::new();
            for (index, block) in text.split("[[package]]").enumerate() {
                if index == 0 {
                    output.push_str(block);
                    continue;
                }
                let is_target =
                    block.contains(r#"name = "kaspa-rpc-core""#) && block.contains("ad45e241");
                if !is_target {
                    output.push_str("[[package]]");
                    output.push_str(block);
                }
            }
            output
        });
        reject(&fixture.root, "cargo-lock-runtime-package-drift");
    }
    #[test]
    fn missing_lockfile_is_rejected() {
        let fixture = fixture();
        fs::remove_file(fixture.root.join("Cargo.lock")).unwrap();
        reject(&fixture.root, "cargo-lock-required");
    }

    #[test]
    fn unapproved_experimental_source_is_rejected() {
        let fixture = fixture();
        let path = fixture.root.join("config/runtime-repository-bindings.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["networks"]["testnet13"]["repo"] = json!("https://example.invalid/fork.git");
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        reject(&fixture.root, "tn13-must-use-official-dagknight");
    }

    #[test]
    fn extra_network_binding_is_rejected() {
        let fixture = fixture();
        let path = fixture.root.join("config/runtime-repository-bindings.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["networks"]["unsupported"] = json!({});
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        reject(&fixture.root, "unexpected-network-binding");
    }
}
