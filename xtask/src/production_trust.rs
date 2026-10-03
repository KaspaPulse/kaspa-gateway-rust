use serde_json::{Value, json};
use std::fs;
use std::path::Path;

const POLICY_PATH: &str = "config/production-trust-requirements.json";
const INTERNAL_MACOS_CONFIG: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.macos.conf.json";
const PRODUCTION_MACOS_CONFIG: &str =
    "apps/kaspa-gateway-desktop/src-tauri/tauri.macos.production.conf.json";
const BUILDER_WORKFLOW: &str = ".github/workflows/desktop-artifacts-builder.yml";
const ARTIFACT_STAGE: &str = "xtask/src/desktop_artifacts_stage.rs";
const RUNBOOK: &str = "docs/security/production-native-signing.md";
const SECRETS_MARKER: &str = concat!("$", "{{ secrets.");

struct Inputs {
    policy: Value,
    internal_macos: Value,
    production_macos: Value,
    builder_workflow: String,
    artifact_stage: String,
    runbook: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let inputs = Inputs {
        policy: read_json(root, POLICY_PATH)?,
        internal_macos: read_json(root, INTERNAL_MACOS_CONFIG)?,
        production_macos: read_json(root, PRODUCTION_MACOS_CONFIG)?,
        builder_workflow: read(root, BUILDER_WORKFLOW)?,
        artifact_stage: read(root, ARTIFACT_STAGE)?,
        runbook: read(root, RUNBOOK)?,
    };
    validate(&inputs)?;
    Ok("PRODUCTION TRUST READINESS CONTRACT PASSED".to_owned())
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("production trust readiness: read {relative}: {error}"))
}

fn read_json(root: &Path, relative: &str) -> Result<Value, String> {
    let text = read(root, relative)?;
    serde_json::from_str(&text)
        .map_err(|error| format!("production trust readiness: invalid {relative}: {error}"))
}

fn expect(value: &Value, pointer: &str, expected: Value) -> Result<(), String> {
    if value.pointer(pointer) == Some(&expected) {
        Ok(())
    } else {
        Err(format!(
            "production trust readiness: {pointer} must equal {expected}"
        ))
    }
}

fn require(text: &str, marker: &str) -> Result<(), String> {
    if text.contains(marker) {
        Ok(())
    } else {
        Err(format!(
            "production trust readiness: required marker missing: {marker}"
        ))
    }
}

fn forbid(text: &str, marker: &str) -> Result<(), String> {
    if text.contains(marker) {
        Err(format!(
            "production trust readiness: forbidden marker present: {marker}"
        ))
    } else {
        Ok(())
    }
}

fn validate_policy(policy: &Value) -> Result<(), String> {
    expect(policy, "/schema_version", json!(1))?;
    expect(
        policy,
        "/artifact_builder/workflow",
        json!(".github/workflows/desktop-artifacts-builder.yml"),
    )?;
    expect(
        policy,
        "/artifact_builder/production_native_signing_separate_boundary",
        json!(true),
    )?;
    expect(
        policy,
        "/artifact_builder/signing_secrets_in_builder",
        json!(false),
    )?;

    expect(
        policy,
        "/windows/signing_provider",
        json!("EXTERNAL_REQUIRED"),
    )?;
    expect(policy, "/windows/file_digest", json!("SHA256"))?;
    expect(policy, "/windows/timestamp_protocol", json!("RFC3161"))?;
    expect(policy, "/windows/timestamp_digest", json!("SHA256"))?;
    expect(
        policy,
        "/windows/code_signing_eku_oid",
        json!("1.3.6.1.5.5.7.3.3"),
    )?;
    expect(
        policy,
        "/windows/verification_requirements",
        json!([
            "AUTHENTICODE_STATUS_VALID",
            "SIGNTOOL_VERIFY_PA_ALL",
            "RFC3161_TIMESTAMP_PRESENT"
        ]),
    )?;

    expect(
        policy,
        "/macos/signing_identity_class",
        json!("Developer ID Application"),
    )?;
    expect(policy, "/macos/hardened_runtime", json!(true))?;
    expect(policy, "/macos/secure_timestamp", json!(true))?;
    expect(policy, "/macos/notarization_tool", json!("notarytool"))?;
    expect(policy, "/macos/staple_required", json!(true))?;
    expect(policy, "/macos/gatekeeper_assessment_required", json!(true))?;
    expect(
        policy,
        "/macos/production_config",
        json!("apps/kaspa-gateway-desktop/src-tauri/tauri.macos.production.conf.json"),
    )?;
    expect(
        policy,
        "/macos/required_signing_environment",
        json!("APPLE_SIGNING_IDENTITY"),
    )?;
    expect(
        policy,
        "/macos/accepted_credential_sets",
        json!([
            ["APPLE_API_ISSUER", "APPLE_API_KEY", "APPLE_API_KEY_PATH"],
            ["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"]
        ]),
    )
}

fn validate_macos_configs(internal: &Value, production: &Value) -> Result<(), String> {
    expect(internal, "/bundle/macOS/signingIdentity", json!("-"))?;
    expect(internal, "/bundle/macOS/hardenedRuntime", json!(false))?;
    expect(production, "/bundle/macOS/signingIdentity", Value::Null)?;
    expect(production, "/bundle/macOS/hardenedRuntime", json!(true))
}

fn validate_builder_and_stage(builder: &str, stage: &str) -> Result<(), String> {
    for marker in [
        "workflow_call:",
        "id-token: write",
        "attestations: write",
        "artifact-metadata: write",
        "cargo run --locked -p xtask -- desktop-artifacts-stage windows",
        "cargo run --locked -p xtask -- desktop-artifacts-stage macos",
    ] {
        require(builder, marker)?;
    }
    forbid(builder, SECRETS_MARKER)?;
    for marker in [
        "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED",
        "MACOS_NOTARIZATION=NOT_CONFIGURED",
        "expected unsigned internal installer",
    ] {
        require(stage, marker)?;
    }
    Ok(())
}

fn validate_runbook(runbook: &str) -> Result<(), String> {
    for marker in [
        "SHA-256",
        "RFC 3161",
        "1.3.6.1.5.5.7.3.3",
        "Developer ID Application",
        "APPLE_SIGNING_IDENTITY",
        "APPLE_API_ISSUER",
        "APPLE_API_KEY",
        "APPLE_API_KEY_PATH",
        "APPLE_ID",
        "APPLE_PASSWORD",
        "APPLE_TEAM_ID",
        "Hardened Runtime",
        "notarytool",
        "stapled and validated",
        "Gatekeeper assessment passes",
        "does not claim that production credentials exist",
    ] {
        require(runbook, marker)?;
    }
    Ok(())
}

fn validate(inputs: &Inputs) -> Result<(), String> {
    validate_policy(&inputs.policy)?;
    validate_macos_configs(&inputs.internal_macos, &inputs.production_macos)?;
    validate_builder_and_stage(&inputs.builder_workflow, &inputs.artifact_stage)?;
    validate_runbook(&inputs.runbook)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Inputs {
        Inputs {
            policy: json!({
                "schema_version": 1,
                "artifact_builder": {
                    "workflow": ".github/workflows/desktop-artifacts-builder.yml",
                    "production_native_signing_separate_boundary": true,
                    "signing_secrets_in_builder": false
                },
                "windows": {
                    "signing_provider": "EXTERNAL_REQUIRED",
                    "file_digest": "SHA256",
                    "timestamp_protocol": "RFC3161",
                    "timestamp_digest": "SHA256",
                    "code_signing_eku_oid": "1.3.6.1.5.5.7.3.3",
                    "verification_requirements": [
                        "AUTHENTICODE_STATUS_VALID",
                        "SIGNTOOL_VERIFY_PA_ALL",
                        "RFC3161_TIMESTAMP_PRESENT"
                    ]
                },
                "macos": {
                    "signing_identity_class": "Developer ID Application",
                    "hardened_runtime": true,
                    "secure_timestamp": true,
                    "notarization_tool": "notarytool",
                    "staple_required": true,
                    "gatekeeper_assessment_required": true,
                    "production_config": "apps/kaspa-gateway-desktop/src-tauri/tauri.macos.production.conf.json",
                    "accepted_credential_sets": [
                        ["APPLE_API_ISSUER", "APPLE_API_KEY", "APPLE_API_KEY_PATH"],
                        ["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"]
                    ],
                    "required_signing_environment": "APPLE_SIGNING_IDENTITY"
                }
            }),
            internal_macos: json!({
                "bundle": {
                    "macOS": {
                        "signingIdentity": "-",
                        "hardenedRuntime": false
                    }
                }
            }),
            production_macos: json!({
                "bundle": {
                    "macOS": {
                        "signingIdentity": null,
                        "hardenedRuntime": true
                    }
                }
            }),
            builder_workflow: "workflow_call:\nid-token: write\nattestations: write\nartifact-metadata: write\ncargo run --locked -p xtask -- desktop-artifacts-stage windows\ncargo run --locked -p xtask -- desktop-artifacts-stage macos\n".to_owned(),
            artifact_stage: "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED\nMACOS_NOTARIZATION=NOT_CONFIGURED\nexpected unsigned internal installer\n".to_owned(),
            runbook: "SHA-256 RFC 3161 1.3.6.1.5.5.7.3.3 Developer ID Application APPLE_SIGNING_IDENTITY APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_PATH APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID Hardened Runtime notarytool stapled and validated Gatekeeper assessment passes does not claim that production credentials exist".to_owned(),
        }
    }

    #[test]
    fn complete_readiness_contract_passes() {
        assert!(validate(&fixture()).is_ok());
    }

    #[test]
    fn macos_production_contract_is_fail_closed() {
        let mut input = fixture();
        input.production_macos["bundle"]["macOS"]["hardenedRuntime"] = json!(false);
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.production_macos["bundle"]["macOS"]["signingIdentity"] = json!("-");
        assert!(validate(&input).is_err());
    }

    #[test]
    fn signing_secrets_cannot_enter_trusted_builder() {
        let mut input = fixture();
        input
            .builder_workflow
            .push_str(&format!("{}APPLE_PASSWORD }}", SECRETS_MARKER));
        assert!(validate(&input).is_err());
    }

    #[test]
    fn cryptographic_policy_cannot_weaken_silently() {
        let mut input = fixture();
        input.policy["windows"]["file_digest"] = json!("SHA1");
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.policy["windows"]["timestamp_protocol"] = json!("LEGACY");
        assert!(validate(&input).is_err());
    }
}
