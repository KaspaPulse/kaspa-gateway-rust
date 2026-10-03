use std::fs;
use std::path::Path;

const WORKFLOW_PATH: &str = ".github/workflows/desktop-release-draft.yml";
const CI_PATH: &str = ".github/workflows/ci.yml";
const CHECKOUT_SHA: &str = "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1";
const RUST_TOOLCHAIN_SHA: &str = "dtolnay/rust-toolchain@4360b52568e2003a75bf9bc1d59f33a8e3fc893c";

pub fn run(root: &Path) -> Result<String, String> {
    let workflow = read(root, WORKFLOW_PATH)?;
    let ci = read(root, CI_PATH)?;
    validate(&workflow, &ci)?;
    Ok("DESKTOP RELEASE DRAFT WORKFLOW CONTRACT PASSED".to_owned())
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative)).map_err(|error| {
        format!("desktop release draft workflow contract: failed to read {relative}: {error}")
    })
}

fn require(text: &str, needle: &str, message: &str) -> Result<(), String> {
    if text.contains(needle) {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn forbid(text: &str, needle: &str, message: &str) -> Result<(), String> {
    if text.contains(needle) {
        Err(message.to_owned())
    } else {
        Ok(())
    }
}

fn validate(workflow: &str, ci: &str) -> Result<(), String> {
    for (needle, message) in [
        (
            "workflow_dispatch:",
            "workflow_dispatch must remain the release entry point",
        ),
        ("version:", "version input must remain explicit"),
        ("commit_sha:", "commit_sha input must remain explicit"),
        (
            "artifact_run_id:",
            "artifact_run_id input must remain explicit",
        ),
        ("confirmation:", "confirmation input must remain explicit"),
        (
            "actions: read",
            "workflow requires read access to artifact runs",
        ),
        (
            "attestations: read",
            "workflow requires attestation verification permission",
        ),
        (
            "contents: write",
            "workflow requires draft-release creation permission",
        ),
        (
            CHECKOUT_SHA,
            "checkout must remain pinned to the approved SHA",
        ),
        (
            RUST_TOOLCHAIN_SHA,
            "Rust action must remain pinned to the approved SHA",
        ),
        (
            "toolchain: 1.98.1",
            "workflow must use the canonical Rust toolchain",
        ),
        (
            "ref: ${{ inputs.commit_sha }}",
            "checkout must bind to the requested exact commit",
        ),
        (
            "persist-credentials: false",
            "checkout credentials must not persist",
        ),
        (
            "REQUESTED_VERSION: ${{ inputs.version }}",
            "version input must flow to Rust owner",
        ),
        (
            "REQUESTED_COMMIT_SHA: ${{ inputs.commit_sha }}",
            "commit input must flow to Rust owner",
        ),
        (
            "ARTIFACT_RUN_ID: ${{ inputs.artifact_run_id }}",
            "artifact run input must flow to Rust owner",
        ),
        (
            "RELEASE_CONFIRMATION: ${{ inputs.confirmation }}",
            "confirmation input must flow to Rust owner",
        ),
        (
            "SIGNER_WORKFLOW: ${{ github.repository }}/.github/workflows/desktop-artifacts.yml",
            "signer workflow identity must be explicit",
        ),
        (
            "cargo run --locked -p xtask -- desktop-release-draft-stage",
            "workflow must delegate release orchestration to Rust",
        ),
    ] {
        require(workflow, needle, message)?;
    }
    for (needle, message) in [
        (
            "shell: bash",
            "workflow adapter must not own shell orchestration",
        ),
        ("gh api ", "GitHub API orchestration must be Rust-owned"),
        ("gh release ", "release mutation must be Rust-owned"),
        ("jq ", "JSON policy logic must be Rust-owned"),
        ("python3", "version parsing must be Rust-owned"),
        ("sha256sum", "checksum logic must be Rust-owned"),
        ("grep -F", "smoke validation must be Rust-owned"),
        ("sleep 2", "draft retry policy must be Rust-owned"),
    ] {
        forbid(workflow, needle, message)?;
    }
    require(
        ci,
        "Verify desktop release draft workflow contract",
        "blocking CI must run the desktop release draft workflow contract",
    )?;
    require(
        ci,
        "cargo run --locked -p xtask -- desktop-release-draft-workflow-gate",
        "blocking CI must execute the Rust draft workflow contract gate",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workflow_fixture() -> String {
        [
            "workflow_dispatch:",
            "version:",
            "commit_sha:",
            "artifact_run_id:",
            "confirmation:",
            "actions: read",
            "attestations: read",
            "contents: write",
            CHECKOUT_SHA,
            RUST_TOOLCHAIN_SHA,
            "toolchain: 1.98.1",
            "ref: ${{ inputs.commit_sha }}",
            "persist-credentials: false",
            "REQUESTED_VERSION: ${{ inputs.version }}",
            "REQUESTED_COMMIT_SHA: ${{ inputs.commit_sha }}",
            "ARTIFACT_RUN_ID: ${{ inputs.artifact_run_id }}",
            "RELEASE_CONFIRMATION: ${{ inputs.confirmation }}",
            "SIGNER_WORKFLOW: ${{ github.repository }}/.github/workflows/desktop-artifacts.yml",
            "cargo run --locked -p xtask -- desktop-release-draft-stage",
        ]
        .join("\n")
    }

    fn ci_fixture() -> String {
        "Verify desktop release draft workflow contract\ncargo run --locked -p xtask -- desktop-release-draft-workflow-gate".to_owned()
    }

    #[test]
    fn minimal_adapter_contract_passes() {
        assert!(validate(&workflow_fixture(), &ci_fixture()).is_ok());
    }

    #[test]
    fn shell_owned_release_logic_is_rejected() {
        for forbidden in [
            "shell: bash",
            "gh api ",
            "gh release ",
            "jq ",
            "python3",
            "sha256sum",
        ] {
            let workflow = format!("{}\n{forbidden}", workflow_fixture());
            assert!(validate(&workflow, &ci_fixture()).is_err(), "{forbidden}");
        }
    }

    #[test]
    fn required_binding_is_fail_closed() {
        for marker in [
            "ref: ${{ inputs.commit_sha }}",
            "REQUESTED_COMMIT_SHA: ${{ inputs.commit_sha }}",
            "cargo run --locked -p xtask -- desktop-release-draft-stage",
        ] {
            let workflow = workflow_fixture().replace(marker, "");
            assert!(validate(&workflow, &ci_fixture()).is_err(), "{marker}");
        }
    }
}
