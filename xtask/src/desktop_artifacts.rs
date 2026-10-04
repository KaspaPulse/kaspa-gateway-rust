use serde_json::Value;
use std::fs;
use std::path::Path;

const WORKFLOW_PATH: &str = ".github/workflows/desktop-artifacts.yml";
const BUILDER_WORKFLOW_PATH: &str = ".github/workflows/desktop-artifacts-builder.yml";
const STAGE_PATH: &str = "xtask/src/desktop_artifacts_stage.rs";
const DESKTOP_MANIFEST_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml";
const WINDOWS_CONFIG_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.windows.conf.json";
const MACOS_CONFIG_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.macos.conf.json";
const RUST_TOOLCHAIN_PATH: &str = "rust-toolchain.toml";
const CARGO_CONFIG_PATH: &str = ".cargo/config.toml";
const SECRETS_MARKER: &str = concat!("$", "{{ secrets.");

const CHECKOUT: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const RUST: &str = "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c";
const CACHE: &str = "Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6";
const NODE: &str = "actions/setup-node@820762786026740c76f36085b0efc47a31fe5020";
const INSTALL: &str = "taiki-e/install-action@83ac0ad63c0167e6f06796fab0fce28db1bf3db0";
const ATTEST: &str = "actions/attest@1e69f48acb82d1966a394da916b4c1698aa569d6";
const UPLOAD: &str = "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a";
struct Inputs {
    workflow: String,
    builder_workflow: String,
    stage_source: String,
    desktop_manifest: String,
    windows_config: Value,
    macos_config: Value,
    rust_toolchain: String,
    cargo_config: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let inputs = Inputs {
        workflow: normalize_newlines(&read(root, WORKFLOW_PATH)?),
        builder_workflow: normalize_newlines(&read(root, BUILDER_WORKFLOW_PATH)?),
        stage_source: normalize_newlines(&read(root, STAGE_PATH)?),
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
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("desktop artifact workflow contract: read {relative}: {error}"))
}
fn parse_json(text: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(text)
        .map_err(|error| format!("desktop artifact workflow contract: invalid {label}: {error}"))
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn compact_whitespace(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn quoted_assignment(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let rest = line
            .trim()
            .strip_prefix(key)?
            .trim_start()
            .strip_prefix('=')?
            .trim();
        (rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"'))
            .then(|| rest[1..rest.len() - 1].to_owned())
    })
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
    let parts = value.split('.').collect::<Vec<_>>();
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

fn require(text: &str, marker: &str) -> Result<(), String> {
    if text.contains(marker) {
        Ok(())
    } else {
        Err(format!(
            "desktop artifact workflow contract: missing {marker}"
        ))
    }
}

fn forbid(text: &str, marker: &str) -> Result<(), String> {
    if text.contains(marker) {
        Err(format!(
            "desktop artifact workflow contract: forbidden executable token {marker}"
        ))
    } else {
        Ok(())
    }
}
fn action_uses(workflow: &str) -> Vec<String> {
    workflow
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("uses:")?.trim();
            Some(rest.split_whitespace().next()?.to_owned())
        })
        .collect()
}

fn immutable_action_ref(value: &str) -> bool {
    value.rsplit_once('@').is_some_and(|(name, sha)| {
        !name.is_empty()
            && sha.len() == 40
            && sha
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn allowed_action(value: &str) -> bool {
    matches!(
        value,
        CHECKOUT | RUST | CACHE | NODE | INSTALL | ATTEST | UPLOAD
    )
}

fn validate_platform_configuration(inputs: &Inputs) -> Result<(), String> {
    let compact_cargo = compact_whitespace(&inputs.cargo_config);
    require(
        &compact_cargo,
        r#"[target.x86_64-pc-windows-msvc]rustflags=["-C","target-feature=+crt-static"]"#,
    )?;
    let canonical = quoted_assignment(&inputs.rust_toolchain, "channel")
        .ok_or_else(|| "rust-toolchain.toml must declare channel".to_owned())?;
    let toolchains = workflow_toolchains(&inputs.builder_workflow);
    if toolchains.len() != 2 || toolchains.iter().any(|value| value != &canonical) {
        return Err(
            "desktop artifact workflow toolchains must exactly match canonical Rust".to_owned(),
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
    )?;
    if !inputs
        .desktop_manifest
        .lines()
        .any(|line| line.trim() == "runtime-provenance-smoke = []")
    {
        return Err("runtime provenance probe feature must remain disabled by default".to_owned());
    }
    Ok(())
}

fn validate_caller_workflow(workflow: &str) -> Result<(), String> {
    for marker in [
        "workflow_dispatch:",
        "commit_sha:",
        "contents: read",
        "id-token: write",
        "attestations: write",
        "artifact-metadata: write",
        "cancel-in-progress: false",
        "uses: ./.github/workflows/desktop-artifacts-builder.yml",
        "commit_sha: ${{ inputs.commit_sha }}",
    ] {
        require(workflow, marker)?;
    }
    for marker in [
        "run:",
        "shell:",
        "actions/",
        "dtolnay/",
        "Swatinem/",
        "taiki-e/",
        "sigstore/",
    ] {
        forbid(workflow, marker)?;
    }
    if workflow.contains(SECRETS_MARKER) {
        return Err("desktop artifact caller must not consume signing secrets".to_owned());
    }
    Ok(())
}

fn validate_builder_workflow(workflow: &str) -> Result<(), String> {
    for marker in [
        "workflow_call:",
        "commit_sha:",
        "contents: read",
        "id-token: write",
        "attestations: write",
        "artifact-metadata: write",
        "runs-on: windows-2022",
        "runs-on: macos-15-intel",
        "ref: ${{ inputs.commit_sha }}",
        "targets: x86_64-pc-windows-msvc,wasm32-unknown-unknown",
        "targets: aarch64-apple-darwin,x86_64-apple-darwin,wasm32-unknown-unknown",
        "tool: syft@1.52.0,wasm-pack@0.15.0",
        "cargo run --locked -p xtask -- desktop-artifacts-stage windows",
        "cargo run --locked -p xtask -- desktop-artifacts-stage macos",
        "cargo run --locked -p xtask -- desktop-artifacts-stage preserve-windows-sbom",
        "cargo run --locked -p xtask -- desktop-artifacts-stage preserve-windows-provenance",
        "cargo run --locked -p xtask -- desktop-artifacts-stage preserve-macos-sbom",
        "cargo run --locked -p xtask -- desktop-artifacts-stage preserve-macos-provenance",
        "WINDOWS_SBOM.spdx.json",
        "MACOS_SBOM.spdx.json",
        "KaspaGateway-windows-x64-nsis.exe",
        "kaspa-gateway-desktop-windows-x64.exe",
        "KaspaGateway-macos-universal.dmg",
        "KaspaGateway-macos-universal-app.zip",
        "sbom-path:",
    ] {
        require(workflow, marker)?;
    }

    for marker in [
        "run: |",
        "run: >",
        "shell:",
        "npm ci",
        "npm --version",
        "node node_modules",
        "powershell",
        "pwsh",
        "bash ",
        "python ",
        "python3 ",
        "Get-AuthenticodeSignature",
        "hdiutil ",
        "codesign ",
        "lipo ",
        "curl ",
        "curl.exe ",
        "shasum ",
        "set +e",
        "PIPESTATUS",
    ] {
        forbid(workflow, marker)?;
    }
    if workflow.contains(SECRETS_MARKER) {
        return Err(
            "trusted reusable artifact builder must not consume signing secrets".to_owned(),
        );
    }
    let uses = action_uses(workflow);
    if uses.len() != 16 {
        return Err(format!(
            "desktop artifact builder must use exactly 16 pinned actions; found {}",
            uses.len()
        ));
    }
    for action in uses {
        if !immutable_action_ref(&action) || !allowed_action(&action) {
            return Err(format!(
                "desktop artifact builder action is not allowed and immutable: {action}"
            ));
        }
    }
    Ok(())
}
fn validate_rust_owner(stage: &str) -> Result<(), String> {
    for marker in [
        "protoc-{PROTOC_VERSION}-win64.zip",
        "5d3ff218d7d91eea95f7569bcb5a98f3030f8996d44151279d9772edcff76082",
        "protoc-{PROTOC_VERSION}-osx-universal_binary.zip",
        "9c27aebb44c537f5627cc13c9c1c6bc0e34ecfefc6e4d79b19764afb8302d95b",
        "x86_64-pc-windows-msvc",
        "universal-apple-darwin",
        "verify_windows_runtime",
        "windows_signature_status",
        "KASPA_GATEWAY_DATA_DIR",
        "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED",
        "WINDOWS_INSTALLER_SMOKE=PASS",
        "MACOS_NOTARIZATION=NOT_CONFIGURED",
        "MACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64",
        "MACOS_CODESIGN_VERIFY=PASS",
        "hdiutil",
        "lipo",
        "codesign",
        "ditto",
        "SPDX-2.3",
        "syft",
        "SHA256SUMS",
        "WINDOWS_SBOM_ATTESTATION.sigstore.json",
        "WINDOWS_BUILD_PROVENANCE.sigstore.json",
        "MACOS_SBOM_ATTESTATION.sigstore.json",
        "MACOS_BUILD_PROVENANCE.sigstore.json",
    ] {
        require(stage, marker)?;
    }
    Ok(())
}

fn validate(inputs: &Inputs) -> Result<(), String> {
    validate_platform_configuration(inputs)?;
    validate_caller_workflow(&inputs.workflow)?;
    validate_builder_workflow(&inputs.builder_workflow)?;
    validate_rust_owner(&inputs.stage_source)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Inputs {
        let actions = [
            CHECKOUT, RUST, CACHE, NODE, INSTALL, ATTEST, ATTEST, UPLOAD, CHECKOUT, RUST, CACHE,
            NODE, INSTALL, ATTEST, ATTEST, UPLOAD,
        ];
        let workflow = [
            "workflow_dispatch:",
            "commit_sha:",
            "contents: read",
            "id-token: write",
            "attestations: write",
            "artifact-metadata: write",
            "cancel-in-progress: false",
            "uses: ./.github/workflows/desktop-artifacts-builder.yml",
            "commit_sha: ${{ inputs.commit_sha }}",
        ]
        .join("\n");
        let mut builder_workflow = String::from(
            "workflow_call:\ncommit_sha:\ncontents: read\nid-token: write\nattestations: write\nartifact-metadata: write\nruns-on: windows-2022\nruns-on: macos-15-intel\nref: ${{ inputs.commit_sha }}\n",
        );
        builder_workflow.push_str(
            "toolchain: 1.99.0\ntoolchain: 1.99.0\ntargets: x86_64-pc-windows-msvc,wasm32-unknown-unknown\ntargets: aarch64-apple-darwin,x86_64-apple-darwin,wasm32-unknown-unknown\n",
        );
        builder_workflow.push_str(
            "tool: syft@1.52.0,wasm-pack@0.15.0\ncargo run --locked -p xtask -- desktop-artifacts-stage windows\ncargo run --locked -p xtask -- desktop-artifacts-stage macos\n",
        );
        for stage in [
            "preserve-windows-sbom",
            "preserve-windows-provenance",
            "preserve-macos-sbom",
            "preserve-macos-provenance",
        ] {
            builder_workflow.push_str(&format!(
                "cargo run --locked -p xtask -- desktop-artifacts-stage {stage}\n"
            ));
        }
        builder_workflow.push_str(
            "WINDOWS_SBOM.spdx.json\nMACOS_SBOM.spdx.json\nKaspaGateway-windows-x64-nsis.exe\nkaspa-gateway-desktop-windows-x64.exe\nKaspaGateway-macos-universal.dmg\nKaspaGateway-macos-universal-app.zip\nsbom-path:\n",
        );
        for action in actions {
            builder_workflow.push_str(&format!("uses: {action}\n"));
        }
        let stage_source = [
            "protoc-{PROTOC_VERSION}-win64.zip",
            "5d3ff218d7d91eea95f7569bcb5a98f3030f8996d44151279d9772edcff76082",
            "protoc-{PROTOC_VERSION}-osx-universal_binary.zip",
            "9c27aebb44c537f5627cc13c9c1c6bc0e34ecfefc6e4d79b19764afb8302d95b",
            "x86_64-pc-windows-msvc",
            "universal-apple-darwin",
            "verify_windows_runtime",
            "windows_signature_status",
            "KASPA_GATEWAY_DATA_DIR",
            "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED",
            "WINDOWS_INSTALLER_SMOKE=PASS",
            "MACOS_NOTARIZATION=NOT_CONFIGURED",
            "MACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64",
            "MACOS_CODESIGN_VERIFY=PASS",
            "hdiutil",
            "lipo",
            "codesign",
            "ditto",
            "SPDX-2.3",
            "syft",
            "SHA256SUMS",
            "WINDOWS_SBOM_ATTESTATION.sigstore.json",
            "WINDOWS_BUILD_PROVENANCE.sigstore.json",
            "MACOS_SBOM_ATTESTATION.sigstore.json",
            "MACOS_BUILD_PROVENANCE.sigstore.json",
        ]
        .join("\n");
        Inputs {
            workflow,
            builder_workflow,
            stage_source,
            desktop_manifest: "[[bin]]\nname = \"kgw-provenance-smoke\"\npath = \"src/bin/kgw-provenance-smoke.rs\"\nrequired-features = [\"runtime-provenance-smoke\"]\n[features]\nruntime-provenance-smoke = []\n".to_owned(),
            windows_config: serde_json::json!({"bundle":{"targets":["nsis"],"icon":["icons/icon.ico"]}}),
            macos_config: serde_json::json!({"bundle":{"targets":["dmg"],"icon":["icons/icon.png"],"macOS":{"signingIdentity":"-","hardenedRuntime":false}}}),
            rust_toolchain: "channel = \"1.99.0\"\n".to_owned(),
            cargo_config: "[target.x86_64-pc-windows-msvc]\nrustflags = [\"-C\", \"target-feature=+crt-static\"]\n".to_owned(),
        }
    }

    #[test]
    fn complete_contract_passes() {
        assert!(validate(&fixture()).is_ok());
    }

    #[test]
    fn workflow_must_remain_declarative_and_pinned() {
        let mut input = fixture();
        input.workflow.push_str("shell: pwsh\nrun: |\n");
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.builder_workflow =
            input
                .builder_workflow
                .replacen(CHECKOUT, "actions/checkout@v7", 1);
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.builder_workflow = input.builder_workflow.replace("workflow_call:", "");
        assert!(validate(&input).is_err());
    }
    #[test]
    fn rust_owner_contract_is_fail_closed() {
        let mut input = fixture();
        input.stage_source = input.stage_source.replace("verify_windows_runtime", "");
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.stage_source = input.stage_source.replace("SPDX-2.3", "");
        assert!(validate(&input).is_err());
    }

    #[test]
    fn platform_bundle_contracts_remain_exact() {
        let mut input = fixture();
        input.windows_config["bundle"]["targets"] = serde_json::json!(["msi"]);
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.macos_config["bundle"]["macOS"]["hardenedRuntime"] = Value::Bool(true);
        assert!(validate(&input).is_err());
    }

    #[test]
    fn signing_secrets_remain_forbidden() {
        let mut input = fixture();
        input
            .workflow
            .push_str(&format!("{}TOKEN }}", SECRETS_MARKER));
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input
            .builder_workflow
            .push_str(&format!("{}TOKEN }}", SECRETS_MARKER));
        assert!(validate(&input).is_err());
    }
}
