use std::fs;
use std::path::Path;

const WORKFLOW_PATH: &str = ".github/workflows/desktop-release-draft.yml";
const CI_PATH: &str = ".github/workflows/ci.yml";

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

fn collapse_whitespace(text: &str) -> String {
    let mut output = String::new();
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !output.is_empty();
        } else {
            if pending_space {
                output.push(' ');
            }
            output.push(ch);
            pending_space = false;
        }
    }
    output
}

fn count_occurrences(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

fn require_order(text: &str, needles: &[&str], message: &str) -> Result<(), String> {
    let mut cursor = 0;
    for needle in needles {
        let Some(offset) = text[cursor..].find(needle) else {
            return Err(message.to_owned());
        };
        cursor += offset + needle.len();
    }
    Ok(())
}
fn validate(workflow: &str, ci: &str) -> Result<(), String> {
    for (needle, message) in [
        (
            "gh api --paginate --slurp",
            "draft workflow must use a draft-inclusive paginated release listing",
        ),
        (
            r#"repos/$GITHUB_REPOSITORY/releases?per_page=100"#,
            "draft workflow must query List releases when detecting or resolving drafts",
        ),
        (
            "id: create-draft",
            "draft creation step must expose the created release ID",
        ),
        (
            "GITHUB_OUTPUT",
            "draft creation must publish release_id through GITHUB_OUTPUT",
        ),
        (
            "DRAFT_RELEASE_ID: ${{ steps.create-draft.outputs.release_id }}",
            "draft verification must consume the created release ID",
        ),
        (
            r#"gh api "repos/$GITHUB_REPOSITORY/releases/$DRAFT_RELEASE_ID""#,
            "draft verification must read the release object by release ID",
        ),
        (
            r#".target_commitish "$draft_json""#,
            "draft verification must bind target_commitish to the requested commit",
        ),
        (
            "KASPA_GATEWAY_WINDOWS_X64_RAW_*.exe",
            "draft release checksum manifest must cover the raw Windows executable",
        ),
    ] {
        require(workflow, needle, message)?;
    }

    for (needle, message) in [
        (
            r#"gh release view "$tag""#,
            "draft preflight must not rely on tag-only release lookup",
        ),
        (
            r#"repos/$GITHUB_REPOSITORY/releases/tags/$tag"#,
            "draft verification must not use the published-release-by-tag endpoint",
        ),
    ] {
        forbid(workflow, needle, message)?;
    }

    let normalized = collapse_whitespace(workflow);
    require(
        &normalized,
        r#"cp "$root/windows/kaspa-gateway-desktop-windows-x64.exe" "$stage/KASPA_GATEWAY_WINDOWS_X64_RAW_${REQUESTED_VERSION}_${short}.exe""#,
        "draft assembly must publish the qualified raw Windows executable",
    )?;

    for fragment in [
        "WINDOWS_SBOM.spdx.json",
        "MACOS_SBOM.spdx.json",
        "WINDOWS_SBOM_ATTESTATION.sigstore.json",
        "MACOS_SBOM_ATTESTATION.sigstore.json",
        "https://spdx.dev/Document/v2.3",
        "verify_sbom",
        "KASPA_GATEWAY_WINDOWS_SBOM_",
        "KASPA_GATEWAY_MACOS_SBOM_",
    ] {
        require(
            workflow,
            fragment,
            &format!("draft release workflow must preserve and verify SBOM evidence: {fragment}"),
        )?;
    }

    if count_occurrences(workflow, "git/ref/tags/$tag") != 1 {
        return Err(
            "git tag-ref lookup is allowed only in preflight, never in post-create draft verification"
                .to_owned(),
        );
    }

    require_order(
        workflow,
        &[
            "id: create-draft",
            "release_id=%s",
            "DRAFT_RELEASE_ID: ${{ steps.create-draft.outputs.release_id }}",
            "releases/$DRAFT_RELEASE_ID",
        ],
        "draft release ID must flow from creation to ID-based verification",
    )?;

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
            "gh api --paginate --slurp",
            r#"repos/$GITHUB_REPOSITORY/releases?per_page=100"#,
            "git/ref/tags/$tag",
            "id: create-draft",
            "echo release_id=%s >> $GITHUB_OUTPUT",
            "DRAFT_RELEASE_ID: ${{ steps.create-draft.outputs.release_id }}",
            r#"gh api "repos/$GITHUB_REPOSITORY/releases/$DRAFT_RELEASE_ID""#,
            r#".target_commitish "$draft_json""#,
            r#"cp "$root/windows/kaspa-gateway-desktop-windows-x64.exe" "$stage/KASPA_GATEWAY_WINDOWS_X64_RAW_${REQUESTED_VERSION}_${short}.exe""#,
            "KASPA_GATEWAY_WINDOWS_X64_RAW_*.exe",
            "WINDOWS_SBOM.spdx.json",
            "MACOS_SBOM.spdx.json",
            "WINDOWS_SBOM_ATTESTATION.sigstore.json",
            "MACOS_SBOM_ATTESTATION.sigstore.json",
            "https://spdx.dev/Document/v2.3",
            "verify_sbom",
            "KASPA_GATEWAY_WINDOWS_SBOM_",
            "KASPA_GATEWAY_MACOS_SBOM_",
        ]
        .join("\n")
    }

    fn ci_fixture() -> String {
        "Verify desktop release draft workflow contract\n\
cargo run --locked -p xtask -- desktop-release-draft-workflow-gate"
            .to_owned()
    }

    #[test]
    fn complete_contract_passes() {
        assert!(validate(&workflow_fixture(), &ci_fixture()).is_ok());
    }

    #[test]
    fn draft_listing_and_id_flow_are_required() {
        for marker in [
            "gh api --paginate --slurp",
            r#"repos/$GITHUB_REPOSITORY/releases?per_page=100"#,
            "id: create-draft",
            "release_id=%s",
            "DRAFT_RELEASE_ID: ${{ steps.create-draft.outputs.release_id }}",
            "releases/$DRAFT_RELEASE_ID",
        ] {
            let workflow = workflow_fixture().replace(marker, "");
            assert!(validate(&workflow, &ci_fixture()).is_err(), "{marker}");
        }
    }

    #[test]
    fn tag_only_release_lookup_is_forbidden() {
        for forbidden in [
            r#"gh release view "$tag""#,
            r#"repos/$GITHUB_REPOSITORY/releases/tags/$tag"#,
        ] {
            let workflow = format!("{}\n{forbidden}", workflow_fixture());
            assert!(validate(&workflow, &ci_fixture()).is_err(), "{forbidden}");
        }
    }

    #[test]
    fn tag_ref_lookup_must_occur_exactly_once() {
        let none = workflow_fixture().replace("git/ref/tags/$tag", "");
        assert!(validate(&none, &ci_fixture()).is_err());

        let twice = format!("{}\ngit/ref/tags/$tag", workflow_fixture());
        assert!(validate(&twice, &ci_fixture()).is_err());
    }
    #[test]
    fn raw_windows_and_sbom_evidence_are_required() {
        for marker in [
            "KASPA_GATEWAY_WINDOWS_X64_RAW_*.exe",
            "WINDOWS_SBOM.spdx.json",
            "MACOS_SBOM.spdx.json",
            "WINDOWS_SBOM_ATTESTATION.sigstore.json",
            "MACOS_SBOM_ATTESTATION.sigstore.json",
            "https://spdx.dev/Document/v2.3",
            "verify_sbom",
        ] {
            let workflow = workflow_fixture().replace(marker, "");
            assert!(validate(&workflow, &ci_fixture()).is_err(), "{marker}");
        }
    }

    #[test]
    fn raw_windows_copy_allows_whitespace_only_drift() {
        let workflow = workflow_fixture().replace(
            r#"cp "$root/windows/kaspa-gateway-desktop-windows-x64.exe" "$stage/KASPA_GATEWAY_WINDOWS_X64_RAW_${REQUESTED_VERSION}_${short}.exe""#,
            "cp   \"$root/windows/kaspa-gateway-desktop-windows-x64.exe\"\n    \"$stage/KASPA_GATEWAY_WINDOWS_X64_RAW_${REQUESTED_VERSION}_${short}.exe\"",
        );
        assert!(validate(&workflow, &ci_fixture()).is_ok());
    }

    #[test]
    fn blocking_ci_must_use_rust_gate() {
        assert!(
            validate(
                &workflow_fixture(),
                "Verify desktop release draft workflow contract"
            )
            .is_err()
        );
        assert!(
            validate(
                &workflow_fixture(),
                "Verify desktop release draft workflow contract\nnode old-gate.cjs"
            )
            .is_err()
        );
    }

    #[test]
    fn release_id_order_is_fail_closed() {
        let workflow = workflow_fixture().replace(
            "id: create-draft\necho release_id=%s >> $GITHUB_OUTPUT",
            "echo release_id=%s >> $GITHUB_OUTPUT\nid: create-draft",
        );
        assert!(validate(&workflow, &ci_fixture()).is_err());
    }
}
