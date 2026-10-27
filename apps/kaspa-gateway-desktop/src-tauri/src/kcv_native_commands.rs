//! Caller-supplied KIP-20 consensus Genesis verification, no network or KSAFE parsing.
//! This endpoint never asserts verified chain acquisition or a global safe badge.
use kcv_types::{ClaimStatus, CovenantId, GenesisOutput, Outpoint, ScriptPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KcvNativeOutputV1 {
    pub index: u32,
    pub value: u64,
    pub script_public_key_version: u16,
    pub script_hex: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KcvNativeGenesisRequestV1 {
    pub expected_covenant_id: String,
    pub authorizing_transaction_id: String,
    pub authorizing_index: u32,
    pub authorized_outputs: Vec<KcvNativeOutputV1>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KcvNativeGenesisResponseV1 {
    pub claim: &'static str,
    pub actual_covenant_id: String,
    pub expected_covenant_id: String,
    pub evidence_source: &'static str,
    pub canonical_evidence_status: &'static str,
    pub global_safe_badge: bool,
}

fn decode_limited_hex(raw: &str, limit: usize) -> Result<Vec<u8>, String> {
    if raw.len() > limit.saturating_mul(2) || !raw.len().is_multiple_of(2) || !raw.is_ascii() {
        return Err("hex string exceeds limit or is not even-length ASCII".into());
    }
    raw.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|part| {
            let s = std::str::from_utf8(part).map_err(|_| "non-ASCII hex")?;
            u8::from_str_radix(s, 16).map_err(|_| "invalid hexadecimal")
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(str::to_owned)
}

fn digest32(name: &str, raw: &str) -> Result<[u8; 32], String> {
    let bytes = decode_limited_hex(raw, 32)?;
    bytes
        .try_into()
        .map_err(|_| format!("{name} must contain exactly 32 bytes"))
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut out, "{byte:02x}").expect("String formatting cannot fail");
    }
    out
}

pub fn verify_native_genesis(
    input: KcvNativeGenesisRequestV1,
) -> Result<KcvNativeGenesisResponseV1, String> {
    if input.authorized_outputs.is_empty()
        || input.authorized_outputs.len() > kcv_core::HARD_MAX_AUTHORIZED_OUTPUTS_V1
    {
        return Err("authorized output count is missing or out of bounds".into());
    }
    let expected = CovenantId(digest32(
        "expected covenant ID",
        &input.expected_covenant_id,
    )?);
    let authorizing_outpoint = Outpoint {
        transaction_id: digest32(
            "authorizing transaction ID",
            &input.authorizing_transaction_id,
        )?,
        index: input.authorizing_index,
    };
    let mut outputs = Vec::with_capacity(input.authorized_outputs.len());
    let mut total_script_bytes = 0usize;
    for output in input.authorized_outputs {
        let script = decode_limited_hex(
            &output.script_hex,
            kcv_core::HARD_MAX_SCRIPT_BYTES_PER_OUTPUT_V1,
        )?;
        total_script_bytes = total_script_bytes
            .checked_add(script.len())
            .ok_or("total script bytes overflow")?;
        if total_script_bytes > kcv_core::HARD_MAX_TOTAL_SCRIPT_BYTES_V1 {
            return Err("total script bytes exceed KCV limit".into());
        }
        outputs.push(GenesisOutput {
            index: output.index,
            value: output.value,
            script_public_key: ScriptPublicKey {
                version: output.script_public_key_version,
                script,
            },
        });
    }
    let verified = kcv_core::verify_covenant_id(expected, authorizing_outpoint, &outputs)
        .map_err(|error| format!("official KCV consensus rejected input: {error:?}"))?;
    let claim = match verified.claim.status {
        ClaimStatus::ClaimMatch => "CLAIM_MATCH",
        ClaimStatus::ClaimMismatch => "CLAIM_MISMATCH",
        ClaimStatus::MissingEvidence => "MISSING_EVIDENCE",
        ClaimStatus::Unsupported => "UNSUPPORTED",
        ClaimStatus::Indeterminate => "INDETERMINATE",
    };
    Ok(KcvNativeGenesisResponseV1 {
        claim,
        actual_covenant_id: to_hex(&verified.actual.0),
        expected_covenant_id: to_hex(&verified.expected.0),
        evidence_source: "CALLER_SUPPLIED_UNVERIFIED",
        canonical_evidence_status: "KSAFE_SCHEMA_UNVERIFIED",
        global_safe_badge: false,
    })
}

#[tauri::command]
pub fn kgw_kcv_verify_native_genesis_v1(
    input: KcvNativeGenesisRequestV1,
) -> Result<KcvNativeGenesisResponseV1, String> {
    verify_native_genesis(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_and_unbounded_payloads_fail_closed() {
        let invalid = KcvNativeGenesisRequestV1 {
            expected_covenant_id: "z".repeat(64),
            authorizing_transaction_id: "a".repeat(64),
            authorizing_index: 1,
            authorized_outputs: vec![KcvNativeOutputV1 {
                index: 0,
                value: 1,
                script_public_key_version: 0,
                script_hex: "aa".to_owned(),
            }],
        };
        assert!(verify_native_genesis(invalid).is_err());
    }

    fn authentic_mainnet_input() -> KcvNativeGenesisRequestV1 {
        KcvNativeGenesisRequestV1 {
            expected_covenant_id:
                "f817e29db006b747489493ac168f386afcae82a741cc133002996e41e89a5752".to_owned(),
            authorizing_transaction_id:
                "018219ebca34717b10203db57d71ba3b61c3b265a909ccb248bc01c0457b13c8".to_owned(),
            authorizing_index: 1,
            authorized_outputs: vec![
                KcvNativeOutputV1 {
                    index: 0,
                    value: 30_000_000,
                    script_public_key_version: 0,
                    script_hex:
                        "aa2063bb08c6f101c5a01b4898342f1e36bbc2ed3674e77349b1513711dba55ba04f87"
                            .to_owned(),
                },
                KcvNativeOutputV1 {
                    index: 1,
                    value: 30_000_000,
                    script_public_key_version: 0,
                    script_hex:
                        "aa206204c471522d0d57ed941a78096cf03b4826cb4f0f86f6660c68ac692d5f19c687"
                            .to_owned(),
                },
            ],
        }
    }

    #[test]
    fn registered_rust_handler_computes_authentic_mainnet_and_does_not_claim_global_safety() {
        let result = kgw_kcv_verify_native_genesis_v1(authentic_mainnet_input())
            .expect("authentic R23 Mainnet fixture must compute");
        assert_eq!(result.claim, "CLAIM_MATCH");
        assert_eq!(
            result.actual_covenant_id,
            "f817e29db006b747489493ac168f386afcae82a741cc133002996e41e89a5752"
        );
        assert_eq!(result.evidence_source, "CALLER_SUPPLIED_UNVERIFIED");
        assert_eq!(result.canonical_evidence_status, "KSAFE_SCHEMA_UNVERIFIED");
        assert!(!result.global_safe_badge);
        let serialized = serde_json::to_value(&result).expect("serialize Tauri response");
        assert_eq!(serialized["globalSafeBadge"], false);
        assert_eq!(serialized["actualCovenantId"], result.actual_covenant_id);
    }

    #[test]
    fn registered_rust_handler_rejects_claim_after_amount_and_outpoint_tampering() {
        let mut changed = authentic_mainnet_input();
        changed.authorized_outputs[0].value += 1;
        let result = kgw_kcv_verify_native_genesis_v1(changed).expect("valid mutated transaction");
        assert_eq!(result.claim, "CLAIM_MISMATCH");
        assert!(!result.global_safe_badge);

        let mut changed = authentic_mainnet_input();
        changed.authorizing_index += 1;
        let result = kgw_kcv_verify_native_genesis_v1(changed).expect("valid mutated outpoint");
        assert_eq!(result.claim, "CLAIM_MISMATCH");
    }

    #[test]
    fn registered_rust_handler_rejects_malformed_or_oversized_script() {
        let mut changed = authentic_mainnet_input();
        changed.authorized_outputs[0].script_hex = "a".to_owned();
        assert!(kgw_kcv_verify_native_genesis_v1(changed).is_err());

        let mut changed = authentic_mainnet_input();
        changed.authorized_outputs[0].script_hex =
            "aa".repeat(kcv_core::HARD_MAX_SCRIPT_BYTES_PER_OUTPUT_V1 + 1);
        assert!(kgw_kcv_verify_native_genesis_v1(changed).is_err());
    }
}
