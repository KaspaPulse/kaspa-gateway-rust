use serde_json::Value;
use std::fs;
use std::path::Path;

const WORKFLOW_PATH: &str = ".github/workflows/desktop-artifacts.yml";
const DESKTOP_MANIFEST_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml";
const WINDOWS_CONFIG_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.windows.conf.json";
const MACOS_CONFIG_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.macos.conf.json";
const RUST_TOOLCHAIN_PATH: &str = "rust-toolchain.toml";
const CARGO_CONFIG_PATH: &str = ".cargo/config.toml";
const SECRETS_MARKER: &str = concat!("$", "{{ secrets.");

struct Inputs {
    workflow: String,
    desktop_manifest: String,
    windows_config: Value,
    macos_config: Value,
    rust_toolchain: String,
    cargo_config: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let inputs = Inputs {
        workflow: normalize_newlines(&read(root, WORKFLOW_PATH)?),
        desktop_manifest: read(root, DESKTOP_MANIFEST_PATH)?,
        windows_config: parse_json(&read(root, WINDOWS_CONFIG_PATH)?, WINDOWS_CONFIG_PATH)?,
        macos_config: parse_json(&read(root, MACOS_CONFIG_PATH)?, MACOS_CONFIG_PATH)?,
        rust_toolchain: read(root, RUST_TOOLCHAIN_PATH)?,
        cargo_config: read(root, CARGO_CONFIG_PATH)?,
    };
    validate(&inputs)?;
    Ok("DESKTOP ARTIFACT WORKFLOW CONTRACT PASSED".to_owned())
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative)).map_err(|error| {
        format!("desktop artifact workflow contract: failed to read {relative}: {error}")
    })
}

fn parse_json(text: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|error| {
        format!("desktop artifact workflow contract: invalid JSON {label}: {error}")
    })
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn compact_whitespace(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn quoted_assignment(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim();
        if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
            return Some(rest[1..rest.len() - 1].to_owned());
        }
    }
    None
}

fn workflow_toolchains(workflow: &str) -> Vec<String> {
    workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("toolchain:"))
        .map(str::trim)
        .filter(|value| is_semver_triplet(value))
        .map(ToOwned::to_owned)
        .collect()
}

fn is_semver_triplet(value: &str) -> bool {
    let parts: Vec<_> = value.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn json_string_array(value: &Value, pointer: &str) -> Option<Vec<String>> {
    value.pointer(pointer)?.as_array().map(|items| {
        items
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect()
    })
}

fn require(text: &str, needle: &str, message: &str) -> Result<(), String> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn line_equals(text: &str, expected: &str) -> bool {
    text.lines().any(|line| line.trim() == expected)
}

fn action_uses(workflow: &str) -> Vec<String> {
    workflow
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("uses:")?.trim();
            let token = rest.split_whitespace().next()?;
            Some(token.to_owned())
        })
        .collect()
}

fn immutable_action_ref(value: &str) -> bool {
    let Some((name, sha)) = value.rsplit_once('@') else {
        return false;
    };
    !name.is_empty()
        && sha.len() == 40
        && sha
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn has_release_action(workflow: &str) -> bool {
    action_uses(workflow).iter().any(|value| {
        (value.starts_with("release/") || value.starts_with("softprops/")) && value.contains('@')
    })
}

fn validate(inputs: &Inputs) -> Result<(), String> {
    let compact_cargo = compact_whitespace(&inputs.cargo_config);
    require(
        &compact_cargo,
        r#"[target.x86_64-pc-windows-msvc]rustflags=["-C","target-feature=+crt-static"]"#,
        "Windows desktop builds must statically link the MSVC runtime",
    )?;

    let canonical = quoted_assignment(&inputs.rust_toolchain, "channel")
        .ok_or_else(|| "rust-toolchain.toml must declare a channel".to_owned())?;
    let toolchains = workflow_toolchains(&inputs.workflow);
    if toolchains.len() != 2 {
        return Err(
            "desktop artifact workflow must declare exactly one Rust toolchain per native job"
                .to_owned(),
        );
    }
    if toolchains.iter().any(|version| version != &canonical) {
        return Err(
            "desktop artifact workflow Rust toolchains must match rust-toolchain.toml".to_owned(),
        );
    }

    if json_string_array(&inputs.windows_config, "/bundle/targets") != Some(vec!["nsis".to_owned()])
    {
        return Err("Windows bundle targets must equal [nsis]".to_owned());
    }
    if json_string_array(&inputs.windows_config, "/bundle/icon")
        != Some(vec!["icons/icon.ico".to_owned()])
    {
        return Err("Windows bundle icon must equal icons/icon.ico".to_owned());
    }
    if json_string_array(&inputs.macos_config, "/bundle/targets") != Some(vec!["dmg".to_owned()]) {
        return Err("macOS bundle targets must equal [dmg]".to_owned());
    }
    if json_string_array(&inputs.macos_config, "/bundle/icon")
        != Some(vec!["icons/icon.png".to_owned()])
    {
        return Err("macOS bundle icon must equal icons/icon.png".to_owned());
    }
    if inputs
        .macos_config
        .pointer("/bundle/macOS/signingIdentity")
        .and_then(Value::as_str)
        != Some("-")
    {
        return Err("macOS signingIdentity must remain ad-hoc '-'".to_owned());
    }
    if inputs
        .macos_config
        .pointer("/bundle/macOS/hardenedRuntime")
        .and_then(Value::as_bool)
        != Some(false)
    {
        return Err("macOS hardenedRuntime must remain false".to_owned());
    }

    let compact_manifest = compact_whitespace(&inputs.desktop_manifest);
    require(
        &compact_manifest,
        r#"[[bin]]name="kgw-provenance-smoke"path="src/bin/kgw-provenance-smoke.rs"required-features=["runtime-provenance-smoke"]"#,
        "the runtime-only provenance probe must be feature-gated out of desktop packages",
    )?;
    if !line_equals(&inputs.desktop_manifest, "runtime-provenance-smoke = []") {
        return Err(
            "the runtime provenance probe feature must remain explicit and disabled by default"
                .to_owned(),
        );
    }

    for fragment in [
        "workflow_dispatch:",
        "commit_sha:",
        "permissions:\n  contents: read",
        "runs-on: windows-2022",
        "runs-on: macos-15-intel",
        "^[0-9a-f]{40}$",
        "npm run tauri:build -- --ci --target x86_64-pc-windows-msvc",
        "npm run tauri:build -- --ci --target universal-apple-darwin",
        "x86_64-pc-windows-msvc",
        "aarch64-apple-darwin,x86_64-apple-darwin",
        "protoc-35.1-win64.zip",
        "5d3ff218d7d91eea95f7569bcb5a98f3030f8996d44151279d9772edcff76082",
        "protoc-35.1-osx-universal_binary.zip",
        "9c27aebb44c537f5627cc13c9c1c6bc0e34ecfefc6e4d79b19764afb8302d95b",
        "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED",
        "MACOS_NOTARIZATION=NOT_CONFIGURED",
        "MACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64",
        "Get-AuthenticodeSignature",
        "cargo run --locked -p xtask -- verify-windows-runtime-dependencies --executable $rawExe",
        "Raw PE runtime dependency verification failed with exit code",
        "cargo run --locked -p xtask -- verify-windows-runtime-dependencies --executable $installedExecutables[0].FullName",
        "Installed PE runtime dependency verification failed with exit code",
        r#"ArgumentList @("/S""#,
        "KASPA_GATEWAY_DATA_DIR",
        "hdiutil attach",
        "lipo -archs",
        "codesign --verify --deep --strict",
        "ditto -c -k --sequesterRsrc --keepParent",
        "taiki-e/install-action@9114bf4d891761788c546334fd37538eae1bf8b3",
        "tool: syft@1.52.0",
        "WINDOWS_SBOM.spdx.json",
        "MACOS_SBOM.spdx.json",
        "WINDOWS_SBOM_ATTESTATION.sigstore.json",
        "MACOS_SBOM_ATTESTATION.sigstore.json",
        "sbom-path:",
        "SPDX-2.3",
        "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    ] {
        require(
            &inputs.workflow,
            fragment,
            &format!("desktop artifact workflow must contain: {fragment}"),
        )?;
    }

    if !line_equals(&inputs.workflow, "MACOS_CODE_SIGNING=$macos_code_signing") {
        return Err(
            "desktop artifact workflow must record macos_code_signing value in the smoke report"
                .to_owned(),
        );
    }
    let signing_modes = [
        "AD_HOC",
        "DEVELOPER_ID",
        "DEVELOPER_ID_NOTARIZED",
        "UNSIGNED",
    ];
    if !signing_modes.iter().any(|mode| {
        inputs
            .workflow
            .contains(&format!("macos_code_signing=\"{mode}\""))
    }) {
        return Err(
            "desktop artifact workflow must normalize supported macOS signing modes".to_owned(),
        );
    }

    if inputs.workflow.contains(SECRETS_MARKER) {
        return Err(
            "unsigned internal artifact workflow must not consume signing secrets".to_owned(),
        );
    }
    if inputs.workflow.contains("arduino/setup-protoc") {
        return Err(
            "workflow must use checksum-verified official protoc archives, not the incompatible GPL action"
                .to_owned(),
        );
    }
    if has_release_action(&inputs.workflow) {
        return Err("artifact workflow must not use a GitHub Release action".to_owned());
    }

    let uses = action_uses(&inputs.workflow);
    if uses.len() < 8 {
        return Err("expected immutable actions in both native jobs".to_owned());
    }
    for action in uses {
        if !immutable_action_ref(&action) {
            return Err(format!(
                "action must be pinned to an immutable full SHA: {action}"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Inputs {
        let pinned = "actions/example@0123456789abcdef0123456789abcdef01234567";
        let mut workflow = vec![
            "workflow_dispatch:".to_owned(),
            "commit_sha:".to_owned(),
            "permissions:\n  contents: read".to_owned(),
            "runs-on: windows-2022".to_owned(),
            "runs-on: macos-15-intel".to_owned(),
            "^[0-9a-f]{40}$".to_owned(),
            "toolchain: 1.98.1".to_owned(),
            "toolchain: 1.98.1".to_owned(),
            "npm run tauri:build -- --ci --target x86_64-pc-windows-msvc".to_owned(),
            "npm run tauri:build -- --ci --target universal-apple-darwin".to_owned(),
            "x86_64-pc-windows-msvc".to_owned(),
            "aarch64-apple-darwin,x86_64-apple-darwin".to_owned(),
            "protoc-35.1-win64.zip".to_owned(),
            "5d3ff218d7d91eea95f7569bcb5a98f3030f8996d44151279d9772edcff76082".to_owned(),
            "protoc-35.1-osx-universal_binary.zip".to_owned(),
            "9c27aebb44c537f5627cc13c9c1c6bc0e34ecfefc6e4d79b19764afb8302d95b".to_owned(),
            "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED".to_owned(),
            "MACOS_NOTARIZATION=NOT_CONFIGURED".to_owned(),
            "MACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64".to_owned(),
            "Get-AuthenticodeSignature".to_owned(),
            "cargo run --locked -p xtask -- verify-windows-runtime-dependencies --executable $rawExe".to_owned(),
            "Raw PE runtime dependency verification failed with exit code".to_owned(),
            "cargo run --locked -p xtask -- verify-windows-runtime-dependencies --executable $installedExecutables[0].FullName".to_owned(),
            "Installed PE runtime dependency verification failed with exit code".to_owned(),
            r#"ArgumentList @("/S""#.to_owned(),
            "KASPA_GATEWAY_DATA_DIR".to_owned(),
            "hdiutil attach".to_owned(),
            "lipo -archs".to_owned(),
            "codesign --verify --deep --strict".to_owned(),
            "ditto -c -k --sequesterRsrc --keepParent".to_owned(),
            "taiki-e/install-action@9114bf4d891761788c546334fd37538eae1bf8b3".to_owned(),
            "tool: syft@1.52.0".to_owned(),
            "WINDOWS_SBOM.spdx.json".to_owned(),
            "MACOS_SBOM.spdx.json".to_owned(),
            "WINDOWS_SBOM_ATTESTATION.sigstore.json".to_owned(),
            "MACOS_SBOM_ATTESTATION.sigstore.json".to_owned(),
            "sbom-path:".to_owned(),
            "SPDX-2.3".to_owned(),
            "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a".to_owned(),
            "MACOS_CODE_SIGNING=$macos_code_signing".to_owned(),
            "macos_code_signing=\"AD_HOC\"".to_owned(),
        ];
        for _ in 0..8 {
            workflow.push(format!("uses: {pinned}"));
        }
        Inputs {
            workflow: workflow.join("\n"),
            desktop_manifest: "[[bin]]\nname = \"kgw-provenance-smoke\"\npath = \"src/bin/kgw-provenance-smoke.rs\"\nrequired-features = [\"runtime-provenance-smoke\"]\n[features]\nruntime-provenance-smoke = []\n".to_owned(),
            windows_config: serde_json::json!({"bundle":{"targets":["nsis"],"icon":["icons/icon.ico"]}}),
            macos_config: serde_json::json!({"bundle":{"targets":["dmg"],"icon":["icons/icon.png"],"macOS":{"signingIdentity":"-","hardenedRuntime":false}}}),
            rust_toolchain: "channel = \"1.98.1\"\n".to_owned(),
            cargo_config: "[target.x86_64-pc-windows-msvc]\nrustflags = [\"-C\", \"target-feature=+crt-static\"]\n".to_owned(),
        }
    }

    #[test]
    fn complete_contract_passes() {
        assert!(validate(&fixture()).is_ok());
    }

    #[test]
    fn native_toolchains_must_match_canonical() {
        let mut input = fixture();
        input.workflow = input
            .workflow
            .replacen("toolchain: 1.98.1", "toolchain: 1.97.1", 1);
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.workflow = input.workflow.replacen("toolchain: 1.98.1\n", "", 1);
        assert!(validate(&input).is_err());
    }

    #[test]
    fn bundle_contracts_are_exact() {
        let mut input = fixture();
        input.windows_config["bundle"]["targets"] = serde_json::json!(["msi"]);
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.macos_config["bundle"]["macOS"]["hardenedRuntime"] = Value::Bool(true);
        assert!(validate(&input).is_err());
    }

    #[test]
    fn provenance_probe_must_remain_feature_gated() {
        let mut input = fixture();
        input.desktop_manifest = input.desktop_manifest.replace(
            "required-features = [\"runtime-provenance-smoke\"]",
            "required-features = []",
        );
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.desktop_manifest = input
            .desktop_manifest
            .replace("runtime-provenance-smoke = []", "");
        assert!(validate(&input).is_err());
    }

    #[test]
    fn workflow_fragments_fail_closed() {
        for marker in [
            "protoc-35.1-win64.zip",
            "WINDOWS_SBOM.spdx.json",
            "MACOS_SBOM_ATTESTATION.sigstore.json",
            "Get-AuthenticodeSignature",
            "lipo -archs",
        ] {
            let mut input = fixture();
            input.workflow = input.workflow.replace(marker, "");
            assert!(validate(&input).is_err(), "{marker}");
        }
    }

    #[test]
    fn secrets_and_release_actions_are_forbidden() {
        let mut input = fixture();
        input
            .workflow
            .push_str(&format!("\n{}TOKEN }}", SECRETS_MARKER));
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.workflow.push_str(
            "\nuses: softprops/action-gh-release@0123456789abcdef0123456789abcdef01234567",
        );
        assert!(validate(&input).is_err());
    }

    #[test]
    fn every_action_must_be_immutable_sha() {
        let mut input = fixture();
        input.workflow = input.workflow.replacen(
            "actions/example@0123456789abcdef0123456789abcdef01234567",
            "actions/example@v1",
            1,
        );
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.workflow = input
            .workflow
            .lines()
            .filter(|line| !line.starts_with("uses:"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(validate(&input).is_err());
    }
}
