use regex::Regex;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_NETWORKS: [&str; 3] = ["mainnet", "testnet10", "testnet13"];

pub fn run_cli(
    args: &mut impl Iterator<Item = String>,
    default_root: &Path,
) -> Result<String, String> {
    let mut root = default_root.to_path_buf();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--root" => {
                root = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--root requires a value".to_owned())?,
                );
            }
            _ => {
                return Err(format!(
                    "unknown runtime-repository-binding-apply argument: {argument}"
                ));
            }
        }
    }
    apply(&root)?;
    Ok("KGW runtime repository binding apply\nstatus=PASS".to_owned())
}

fn load_manifest(root: &Path) -> Result<Value, String> {
    let path = root.join("config/runtime-repository-bindings.json");
    let bytes = fs::read(&path)
        .map_err(|error| format!("manifest read failed {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("manifest JSON invalid {}: {error}", path.display()))
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("binding field missing or invalid: {key}"))
}

fn strings(value: &Value, key: &str) -> Result<Vec<String>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("binding field missing or invalid: {key}"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("binding array contains non-string: {key}"))
        })
        .collect()
}

fn networks(manifest: &Value) -> Result<&Map<String, Value>, String> {
    manifest
        .get("networks")
        .and_then(Value::as_object)
        .ok_or_else(|| "manifest networks missing or invalid".to_owned())
}

fn binding_by_family<'a>(
    networks: &'a Map<String, Value>,
    family: &str,
) -> Result<&'a Value, String> {
    for network_name in EXPECTED_NETWORKS {
        if let Some(binding) = networks.get(network_name)
            && binding.get("family").and_then(Value::as_str) == Some(family)
        {
            return Ok(binding);
        }
    }
    Err(format!("Missing binding family: {family}"))
}

fn spec(binding: &Value) -> Result<(&'static str, &str), String> {
    if let Some(rev) = binding.get("rev").and_then(Value::as_str)
        && !rev.is_empty()
    {
        return Ok(("rev", rev));
    }
    if let Some(branch) = binding.get("branch").and_then(Value::as_str)
        && !branch.is_empty()
    {
        return Ok(("branch", branch));
    }
    Err("Binding must define branch or rev.".to_owned())
}

fn cargo_line(alias: &str, package: &str, binding: &Value) -> Result<String, String> {
    let repo = string(binding, "repo")?;
    let (key, value) = spec(binding)?;
    Ok(format!(
        r#"{alias} = {{ package = "{package}", git = "{repo}", {key} = "{value}", optional = true }}"#
    ))
}

fn simple_cargo_line(alias: &str, binding: &Value) -> Result<String, String> {
    let repo = string(binding, "repo")?;
    let (key, value) = spec(binding)?;
    Ok(format!(
        r#"{alias} = {{ git = "{repo}", {key} = "{value}" }}"#
    ))
}

fn source_newline(text: &str) -> &'static str {
    if text.contains("\r\n") { "\r\n" } else { "\n" }
}

fn write_if_changed(path: &Path, before: &str, after: &str) -> Result<(), String> {
    if after != before {
        fs::write(path, after.as_bytes())
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    Ok(())
}

fn replace_cargo_line(
    root: &Path,
    relative: &str,
    alias: &str,
    replacement: &str,
) -> Result<(), String> {
    let path = root.join(relative);
    let before = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let pattern = Regex::new(&format!(
        r"^([ \t]*){}[ \t]*=[ \t]*\{{[^\r\n]*\}}[ \t]*$",
        regex::escape(alias)
    ))
    .map_err(|error| format!("invalid Cargo alias regex: {error}"))?;

    let mut matched = false;
    let mut after = String::with_capacity(before.len());
    for segment in before.split_inclusive('\n') {
        let (line, ending) = if let Some(line) = segment.strip_suffix("\r\n") {
            (line, "\r\n")
        } else if let Some(line) = segment.strip_suffix('\n') {
            (line, "\n")
        } else {
            (segment, "")
        };
        if let Some(captures) = pattern.captures(line) {
            if matched {
                return Err(format!("Duplicate Cargo alias: {alias} in {relative}"));
            }
            matched = true;
            after.push_str(captures.get(1).map_or("", |value| value.as_str()));
            after.push_str(replacement);
            after.push_str(ending);
        } else {
            after.push_str(segment);
        }
    }
    if !matched {
        return Err(format!("Missing Cargo alias: {alias} in {relative}"));
    }
    write_if_changed(&path, &before, &after)
}

#[derive(Debug, Clone)]
struct FamilyGroup<'a> {
    family: String,
    binding: &'a Value,
    networks: Vec<String>,
}

fn family_groups<'a>(networks: &'a Map<String, Value>) -> Result<Vec<FamilyGroup<'a>>, String> {
    let mut groups = Vec::new();
    for network_name in EXPECTED_NETWORKS {
        let binding = networks
            .get(network_name)
            .ok_or_else(|| format!("Missing network binding: {network_name}"))?;
        let family = string(binding, "family")?.to_owned();
        if let Some(existing) = groups
            .iter_mut()
            .find(|group: &&mut FamilyGroup<'a>| group.family == family)
        {
            existing.networks.push(network_name.to_owned());
        } else {
            groups.push(FamilyGroup {
                family,
                binding,
                networks: vec![network_name.to_owned()],
            });
        }
    }
    Ok(groups)
}

fn network_arm(networks: &[String], value: &str) -> Result<String, String> {
    let mut variants = Vec::new();
    for network in networks {
        variants.push(match network.as_str() {
            "mainnet" => "Self::Mainnet",
            "testnet10" => "Self::Testnet10",
            "testnet13" => "Self::Testnet13",
            other => return Err(format!("Unsupported network: {other}")),
        });
    }
    Ok(format!("            {} => {value},", variants.join(" | ")))
}

fn string_function(
    groups: &[FamilyGroup<'_>],
    signature: &str,
    field: &str,
) -> Result<String, String> {
    let mut lines = vec![
        format!("    {signature} {{"),
        "        match self {".to_owned(),
    ];
    for group in groups {
        let value = string(group.binding, field)?;
        lines.push(network_arm(&group.networks, &format!(r#""{value}""#))?);
    }
    lines.push("        }".to_owned());
    lines.push("    }".to_owned());
    Ok(lines.join("\n"))
}

fn family_function(
    groups: &[FamilyGroup<'_>],
    signature: &str,
    enum_prefix: &str,
) -> Result<String, String> {
    let mut lines = vec![
        format!("    {signature} {{"),
        "        match self {".to_owned(),
    ];
    for group in groups {
        let variant = match group.family.as_str() {
            "mainline" => "Mainline",
            "tn13" => "Tn13",
            other => return Err(format!("Unsupported {enum_prefix} family: {other}")),
        };
        lines.push(network_arm(
            &group.networks,
            &format!("{enum_prefix}::{variant}"),
        )?);
    }
    lines.push("        }".to_owned());
    lines.push("    }".to_owned());
    Ok(lines.join("\n"))
}

fn replace_rust_function(
    root: &Path,
    relative: &str,
    signature: &str,
    replacement: &str,
) -> Result<(), String> {
    let path = root.join(relative);
    let before = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let start = before
        .find(signature)
        .ok_or_else(|| format!("Missing Rust function signature: {signature} in {relative}"))?;
    let line_start = before[..start].rfind('\n').map_or(0, |index| index + 1);
    let brace_start = before[start..]
        .find('{')
        .map(|index| start + index)
        .ok_or_else(|| format!("Missing Rust function body: {signature} in {relative}"))?;

    let mut depth = 0_i64;
    let mut end = None;
    for (relative_index, ch) in before[brace_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(brace_start + relative_index);
                    break;
                }
            }
            _ => {}
        }
    }
    let end =
        end.ok_or_else(|| format!("Could not find Rust function end: {signature} in {relative}"))?;
    let newline = source_newline(&before);
    let stable_replacement = replacement.replace("\r\n", "\n").replace('\n', newline);
    let after = format!(
        "{}{}{}",
        &before[..line_start],
        stable_replacement,
        &before[end + 1..]
    );
    write_if_changed(&path, &before, &after)
}

fn apply(root: &Path) -> Result<(), String> {
    let manifest = load_manifest(root)?;
    let networks = networks(&manifest)?;
    let mainline = binding_by_family(networks, "mainline")?;

    let node_cargo = "crates/kaspa-gateway-rk-node/Cargo.toml";
    let bridge_cargo = "crates/kaspa-gateway-rk-bridge/Cargo.toml";
    let cli_cargo = "apps/kaspa-gateway-cli/Cargo.toml";
    let core_cargo = "crates/kaspa-gateway-core/Cargo.toml";

    for family in ["mainline", "tn13"] {
        let binding = binding_by_family(networks, family)?;
        let aliases = strings(binding, "nodeAliases")?;
        let packages = binding
            .pointer("/packages/node")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("Node packages missing for family: {family}"))?;
        if aliases.len() != packages.len() {
            return Err(format!(
                "Node alias/package count mismatch for family: {family}"
            ));
        }
        for (alias, package) in aliases.iter().zip(packages) {
            let package = package
                .as_str()
                .ok_or_else(|| format!("Node package must be string for family: {family}"))?;
            let line = cargo_line(alias, package, binding)?;
            replace_cargo_line(root, node_cargo, alias, &line)?;
        }

        let aliases = strings(binding, "bridgeAliases")?;
        let packages = binding
            .pointer("/packages/bridge")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("Bridge packages missing for family: {family}"))?;
        if aliases.len() != packages.len() {
            return Err(format!(
                "Bridge alias/package count mismatch for family: {family}"
            ));
        }
        for (alias, package) in aliases.iter().zip(packages) {
            let package = package
                .as_str()
                .ok_or_else(|| format!("Bridge package must be string for family: {family}"))?;
            let line = cargo_line(alias, package, binding)?;
            replace_cargo_line(root, bridge_cargo, alias, &line)?;
        }
    }

    for (alias, package) in [
        ("kaspa-grpc-client-live", "kaspa-grpc-client"),
        ("kaspa-rpc-core-live", "kaspa-rpc-core"),
    ] {
        let line = cargo_line(alias, package, mainline)?;
        replace_cargo_line(root, cli_cargo, alias, &line)?;
    }
    let core_line = simple_cargo_line("kaspa-addresses", mainline)?;
    replace_cargo_line(root, core_cargo, "kaspa-addresses", &core_line)?;

    let groups = family_groups(networks)?;
    let branch = string_function(&groups, "pub fn branch(self) -> &'static str", "branch")?;
    let revision = string_function(&groups, "pub fn revision(self) -> &'static str", "rev")?;

    let service_controller = "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs";
    replace_rust_function(
        root,
        service_controller,
        "pub fn branch(self) -> &'static str",
        &branch,
    )?;
    replace_rust_function(
        root,
        service_controller,
        "pub fn revision(self) -> &'static str",
        &revision,
    )?;

    let official_runtime = "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs";
    replace_rust_function(
        root,
        official_runtime,
        "pub fn branch(self) -> &'static str",
        &branch,
    )?;
    replace_rust_function(
        root,
        official_runtime,
        "pub fn revision(self) -> &'static str",
        &revision,
    )?;
    let node_family = family_function(
        &groups,
        "pub fn family(self) -> KaspaRuntimeFamily",
        "KaspaRuntimeFamily",
    )?;
    replace_rust_function(
        root,
        official_runtime,
        "pub fn family(self) -> KaspaRuntimeFamily",
        &node_family,
    )?;

    let bridge_runtime = "crates/kaspa-gateway-rk-bridge/src/lib.rs";
    replace_rust_function(
        root,
        bridge_runtime,
        "pub fn branch(self) -> &'static str",
        &branch,
    )?;
    replace_rust_function(
        root,
        bridge_runtime,
        "pub fn revision(self) -> &'static str",
        &revision,
    )?;
    let bridge_family = family_function(
        &groups,
        "pub fn family(self) -> BridgeRuntimeFamily",
        "BridgeRuntimeFamily",
    )?;
    replace_rust_function(
        root,
        bridge_runtime,
        "pub fn family(self) -> BridgeRuntimeFamily",
        &bridge_family,
    )?;

    Ok(())
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
        "apps/kaspa-gateway-cli/Cargo.toml",
        "crates/kaspa-gateway-core/Cargo.toml",
        "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
        "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs",
        "crates/kaspa-gateway-rk-bridge/src/lib.rs",
    ];

    struct Fixture {
        root: PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn fixture() -> Fixture {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "kgw-binding-apply-rust-{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        for relative in FIXTURE_FILES {
            let source = repository_root().join(relative);
            let target = root.join(relative);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::copy(source, target).unwrap();
        }
        Fixture { root }
    }

    fn mutate(root: &Path, relative: &str, from: &str, to: &str) {
        let path = root.join(relative);
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains(from),
            "fixture source missing drift target: {from}"
        );
        fs::write(path, text.replacen(from, to, 1)).unwrap();
    }

    fn bytes(root: &Path, relative: &str) -> Vec<u8> {
        fs::read(root.join(relative)).unwrap()
    }

    #[test]
    fn repairs_manifest_driven_drift_and_is_idempotent() {
        let fixture = fixture();
        mutate(
            &fixture.root,
            "crates/kaspa-gateway-rk-node/Cargo.toml",
            "rev = \"01b532e8b553523216471682649693af92f0fd16\"",
            "rev = \"0000000000000000000000000000000000000000\"",
        );
        mutate(
            &fixture.root,
            "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
            "Self::Mainnet | Self::Testnet10 => \"stable\"",
            "Self::Mainnet | Self::Testnet10 => \"drifted\"",
        );

        apply(&fixture.root).unwrap();

        for relative in FIXTURE_FILES {
            assert_eq!(
                bytes(&fixture.root, relative),
                bytes(&repository_root(), relative),
                "post-apply mismatch for {relative}"
            );
        }

        let once: Vec<_> = FIXTURE_FILES
            .iter()
            .map(|relative| bytes(&fixture.root, relative))
            .collect();
        apply(&fixture.root).unwrap();
        let twice: Vec<_> = FIXTURE_FILES
            .iter()
            .map(|relative| bytes(&fixture.root, relative))
            .collect();
        assert_eq!(once, twice);
    }

    #[test]
    fn missing_alias_fails_closed() {
        let fixture = fixture();
        let path = fixture.root.join("crates/kaspa-gateway-rk-node/Cargo.toml");
        let text = fs::read_to_string(&path).unwrap();
        let filtered = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("kaspad-lib-mainline = {"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(path, filtered).unwrap();
        let error = apply(&fixture.root).unwrap_err();
        assert!(error.contains("Missing Cargo alias: kaspad-lib-mainline"));
    }

    #[test]
    fn unknown_cli_argument_fails_closed() {
        let mut args = vec!["--unknown".to_owned()].into_iter();
        assert!(run_cli(&mut args, &repository_root()).is_err());
    }
}
