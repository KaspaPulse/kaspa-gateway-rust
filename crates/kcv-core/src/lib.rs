#![forbid(unsafe_code)]

use kaspa_consensus_core::{
    Hash,
    hashing::covenant_id::covenant_id,
    tx::{ScriptPublicKey as KaspaScriptPublicKey, TransactionOutpoint, TransactionOutput},
};
use kcv_types::{ClaimId, ClaimStatus, CovenantId, GenesisOutput, Outpoint, VerifiedClaim};

pub const CONSENSUS_GENESIS_COVENANT_ID_CLAIM: &str = "consensus.genesis.covenant_id";

/// Local verifier admission cap, not a Kaspa consensus rule.
pub const HARD_MAX_AUTHORIZED_OUTPUTS_V1: usize = 4096;
/// Local verifier admission cap, not a Kaspa consensus rule.
pub const HARD_MAX_SCRIPT_BYTES_PER_OUTPUT_V1: usize = 1024 * 1024;
/// Local verifier admission cap, not a Kaspa consensus rule.
pub const HARD_MAX_TOTAL_SCRIPT_BYTES_V1: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreVerificationLimitsV1 {
    pub max_authorized_outputs: usize,
    pub max_script_bytes_per_output: usize,
    pub max_total_script_bytes: usize,
}

impl Default for CoreVerificationLimitsV1 {
    fn default() -> Self {
        Self {
            max_authorized_outputs: HARD_MAX_AUTHORIZED_OUTPUTS_V1,
            max_script_bytes_per_output: HARD_MAX_SCRIPT_BYTES_PER_OUTPUT_V1,
            max_total_script_bytes: HARD_MAX_TOTAL_SCRIPT_BYTES_V1,
        }
    }
}

impl CoreVerificationLimitsV1 {
    pub fn validate(self) -> Result<(), CoreVerificationError> {
        validate_limit(
            "max_authorized_outputs",
            self.max_authorized_outputs,
            HARD_MAX_AUTHORIZED_OUTPUTS_V1,
        )?;
        validate_limit(
            "max_script_bytes_per_output",
            self.max_script_bytes_per_output,
            HARD_MAX_SCRIPT_BYTES_PER_OUTPUT_V1,
        )?;
        validate_limit(
            "max_total_script_bytes",
            self.max_total_script_bytes,
            HARD_MAX_TOTAL_SCRIPT_BYTES_V1,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreVerificationError {
    InvalidLimit {
        field: &'static str,
        value: usize,
    },
    EmptyAuthorizedOutputs,
    TooManyAuthorizedOutputs {
        actual: usize,
        limit: usize,
    },
    ScriptPublicKeyTooLarge {
        output_index: u32,
        actual: usize,
        limit: usize,
    },
    TotalScriptBytesOverflow,
    TotalScriptBytesExceeded {
        actual: usize,
        limit: usize,
    },
    OutputIndicesNotStrictlyIncreasing {
        previous: u32,
        current: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CovenantIdVerification {
    pub expected: CovenantId,
    pub actual: CovenantId,
    pub claim: VerifiedClaim,
}

pub fn compute_covenant_id(
    authorizing_outpoint: Outpoint,
    authorized_outputs: &[GenesisOutput],
) -> Result<CovenantId, CoreVerificationError> {
    compute_covenant_id_with_limits(
        authorizing_outpoint,
        authorized_outputs,
        CoreVerificationLimitsV1::default(),
    )
}

pub fn compute_covenant_id_with_limits(
    authorizing_outpoint: Outpoint,
    authorized_outputs: &[GenesisOutput],
    limits: CoreVerificationLimitsV1,
) -> Result<CovenantId, CoreVerificationError> {
    validate_authorized_outputs(authorized_outputs, limits)?;

    let outpoint = TransactionOutpoint::new(
        Hash::from_bytes(authorizing_outpoint.transaction_id),
        authorizing_outpoint.index,
    );
    let kaspa_outputs = authorized_outputs
        .iter()
        .map(|output| {
            TransactionOutput::new(
                output.value,
                KaspaScriptPublicKey::from_vec(
                    output.script_public_key.version,
                    output.script_public_key.script.clone(),
                ),
            )
        })
        .collect::<Vec<_>>();

    let digest = covenant_id(
        outpoint,
        authorized_outputs
            .iter()
            .zip(kaspa_outputs.iter())
            .map(|(output, kaspa_output)| (output.index, kaspa_output)),
    );

    Ok(CovenantId(digest.as_bytes()))
}

pub fn verify_covenant_id(
    expected: CovenantId,
    authorizing_outpoint: Outpoint,
    authorized_outputs: &[GenesisOutput],
) -> Result<CovenantIdVerification, CoreVerificationError> {
    verify_covenant_id_with_limits(
        expected,
        authorizing_outpoint,
        authorized_outputs,
        CoreVerificationLimitsV1::default(),
    )
}

pub fn verify_covenant_id_with_limits(
    expected: CovenantId,
    authorizing_outpoint: Outpoint,
    authorized_outputs: &[GenesisOutput],
    limits: CoreVerificationLimitsV1,
) -> Result<CovenantIdVerification, CoreVerificationError> {
    let actual = compute_covenant_id_with_limits(authorizing_outpoint, authorized_outputs, limits)?;
    let status = if actual == expected {
        ClaimStatus::ClaimMatch
    } else {
        ClaimStatus::ClaimMismatch
    };

    Ok(CovenantIdVerification {
        expected,
        actual,
        claim: VerifiedClaim {
            claim_id: ClaimId::from(CONSENSUS_GENESIS_COVENANT_ID_CLAIM),
            status,
            detail: None,
        },
    })
}

/// A narrow, caller-evidence KIP-20 continuation structure gate.
///
/// This cross-checks the already-accepted parent output, the UTXO being spent,
/// the actual child input outpoint, and the child output covenant binding using
/// official rusty-kaspa consensus types. It DOES NOT authenticate a remote
/// API, execute signatures/scripts, prove UTXO-set membership, or assert that
/// the new output state is permitted by the contract. No global safety badge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuationStructureErrorV1 {
    UnsupportedTransactionVersion,
    InvalidAuthorizingInput,
    ConsumedOutpointMismatch,
    MissingParentCovenant,
    MissingSpentUtxoCovenant,
    SpentUtxoCovenantMismatch,
    MissingChildCovenant,
    ChildAuthorizingInputMismatch,
    ChildCovenantMismatch,
}

/// All caller-supplied, unauthenticated structural evidence required by KIP-20.
#[derive(Clone, Copy, Debug)]
pub struct ContinuationStructureEvidenceV1<'a> {
    pub parent_outpoint: &'a TransactionOutpoint,
    pub child_authorizing_prevout: &'a TransactionOutpoint,
    pub parent_output: &'a TransactionOutput,
    pub spent_utxo_covenant_id: Option<Hash>,
    pub child_output: &'a TransactionOutput,
    pub child_authorizing_input_index: u16,
    pub child_input_count: usize,
    pub child_transaction_version: u16,
}

pub fn validate_continuation_structure_v1(
    evidence: ContinuationStructureEvidenceV1<'_>,
) -> Result<(), ContinuationStructureErrorV1> {
    let ContinuationStructureEvidenceV1 {
        parent_outpoint,
        child_authorizing_prevout,
        parent_output,
        spent_utxo_covenant_id,
        child_output,
        child_authorizing_input_index,
        child_input_count,
        child_transaction_version,
    } = evidence;
    if child_transaction_version < 1 {
        return Err(ContinuationStructureErrorV1::UnsupportedTransactionVersion);
    }
    if usize::from(child_authorizing_input_index) >= child_input_count {
        return Err(ContinuationStructureErrorV1::InvalidAuthorizingInput);
    }
    if parent_outpoint != child_authorizing_prevout {
        return Err(ContinuationStructureErrorV1::ConsumedOutpointMismatch);
    }
    let Some(parent_binding) = parent_output.covenant else {
        return Err(ContinuationStructureErrorV1::MissingParentCovenant);
    };
    let Some(spent_id) = spent_utxo_covenant_id else {
        return Err(ContinuationStructureErrorV1::MissingSpentUtxoCovenant);
    };
    if parent_binding.covenant_id != spent_id {
        return Err(ContinuationStructureErrorV1::SpentUtxoCovenantMismatch);
    }
    let Some(child_binding) = child_output.covenant else {
        return Err(ContinuationStructureErrorV1::MissingChildCovenant);
    };
    if child_binding.authorizing_input != child_authorizing_input_index {
        return Err(ContinuationStructureErrorV1::ChildAuthorizingInputMismatch);
    }
    if child_binding.covenant_id != spent_id {
        return Err(ContinuationStructureErrorV1::ChildCovenantMismatch);
    }
    Ok(())
}

fn validate_limit(
    field: &'static str,
    value: usize,
    hard_max: usize,
) -> Result<(), CoreVerificationError> {
    if value == 0 || value > hard_max {
        Err(CoreVerificationError::InvalidLimit { field, value })
    } else {
        Ok(())
    }
}

fn validate_authorized_outputs(
    outputs: &[GenesisOutput],
    limits: CoreVerificationLimitsV1,
) -> Result<(), CoreVerificationError> {
    limits.validate()?;

    if outputs.is_empty() {
        return Err(CoreVerificationError::EmptyAuthorizedOutputs);
    }
    if outputs.len() > limits.max_authorized_outputs {
        return Err(CoreVerificationError::TooManyAuthorizedOutputs {
            actual: outputs.len(),
            limit: limits.max_authorized_outputs,
        });
    }

    let mut total_script_bytes = 0usize;
    for output in outputs {
        let script_bytes = output.script_public_key.script.len();
        if script_bytes > limits.max_script_bytes_per_output {
            return Err(CoreVerificationError::ScriptPublicKeyTooLarge {
                output_index: output.index,
                actual: script_bytes,
                limit: limits.max_script_bytes_per_output,
            });
        }
        total_script_bytes = total_script_bytes
            .checked_add(script_bytes)
            .ok_or(CoreVerificationError::TotalScriptBytesOverflow)?;
        if total_script_bytes > limits.max_total_script_bytes {
            return Err(CoreVerificationError::TotalScriptBytesExceeded {
                actual: total_script_bytes,
                limit: limits.max_total_script_bytes,
            });
        }
    }

    for pair in outputs.windows(2) {
        if pair[0].index >= pair[1].index {
            return Err(CoreVerificationError::OutputIndicesNotStrictlyIncreasing {
                previous: pair[0].index,
                current: pair[1].index,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kcv_types::ScriptPublicKey;

    const GOLDEN_ID: CovenantId = CovenantId([
        0xed, 0xfa, 0x06, 0x2a, 0x92, 0xae, 0x19, 0x1b, 0xc2, 0x54, 0xf9, 0x51, 0x8f, 0xe9, 0xa4,
        0x6a, 0x65, 0x1b, 0x2e, 0x63, 0x4b, 0x75, 0xe2, 0x72, 0xdd, 0xad, 0x5b, 0xf7, 0x02, 0xe4,
        0xf0, 0x3b,
    ]);

    fn fixture() -> (Outpoint, Vec<GenesisOutput>) {
        (
            Outpoint {
                transaction_id: [0x11; 32],
                index: 3,
            },
            vec![
                GenesisOutput {
                    index: 0,
                    value: 100,
                    script_public_key: ScriptPublicKey {
                        version: 0,
                        script: vec![0x51],
                    },
                },
                GenesisOutput {
                    index: 2,
                    value: 200,
                    script_public_key: ScriptPublicKey {
                        version: 1,
                        script: vec![0xaa, 0xbb],
                    },
                },
            ],
        )
    }

    #[test]
    fn fixed_vector_matches_exact_upstream_implementation() {
        let (outpoint, outputs) = fixture();
        assert_eq!(compute_covenant_id(outpoint, &outputs), Ok(GOLDEN_ID));
    }

    #[test]
    fn verification_is_claim_specific_and_policy_neutral() {
        let (outpoint, outputs) = fixture();
        let matched = verify_covenant_id(GOLDEN_ID, outpoint, &outputs).unwrap();
        assert_eq!(matched.actual, GOLDEN_ID);
        assert_eq!(matched.claim.status, ClaimStatus::ClaimMatch);
        assert_eq!(
            matched.claim.claim_id.as_str(),
            CONSENSUS_GENESIS_COVENANT_ID_CLAIM
        );

        let mismatched = verify_covenant_id(CovenantId([0; 32]), outpoint, &outputs).unwrap();
        assert_eq!(mismatched.claim.status, ClaimStatus::ClaimMismatch);
    }

    #[test]
    fn every_committed_preimage_field_changes_the_id() {
        let (outpoint, outputs) = fixture();
        let baseline = compute_covenant_id(outpoint, &outputs).unwrap();

        let mut changed_outpoint_hash = outpoint;
        changed_outpoint_hash.transaction_id[0] ^= 1;
        assert_ne!(
            compute_covenant_id(changed_outpoint_hash, &outputs).unwrap(),
            baseline
        );

        let mut changed_outpoint_index = outpoint;
        changed_outpoint_index.index += 1;
        assert_ne!(
            compute_covenant_id(changed_outpoint_index, &outputs).unwrap(),
            baseline
        );

        let mut changed_count = outputs.clone();
        changed_count.pop();
        assert_ne!(
            compute_covenant_id(outpoint, &changed_count).unwrap(),
            baseline
        );

        let mut changed_output_index = outputs.clone();
        changed_output_index[0].index = 1;
        assert_ne!(
            compute_covenant_id(outpoint, &changed_output_index).unwrap(),
            baseline
        );

        let mut changed_value = outputs.clone();
        changed_value[0].value += 1;
        assert_ne!(
            compute_covenant_id(outpoint, &changed_value).unwrap(),
            baseline
        );

        let mut changed_script_version = outputs.clone();
        changed_script_version[0].script_public_key.version += 1;
        assert_ne!(
            compute_covenant_id(outpoint, &changed_script_version).unwrap(),
            baseline
        );

        let mut changed_script = outputs.clone();
        changed_script[0].script_public_key.script.push(0x00);
        assert_ne!(
            compute_covenant_id(outpoint, &changed_script).unwrap(),
            baseline
        );
    }

    #[test]
    fn empty_outputs_fail_closed() {
        let (outpoint, _) = fixture();
        assert_eq!(
            compute_covenant_id(outpoint, &[]),
            Err(CoreVerificationError::EmptyAuthorizedOutputs)
        );
    }

    #[test]
    fn output_indices_must_be_strictly_increasing() {
        let (outpoint, mut outputs) = fixture();
        outputs[1].index = outputs[0].index;
        assert_eq!(
            compute_covenant_id(outpoint, &outputs),
            Err(CoreVerificationError::OutputIndicesNotStrictlyIncreasing {
                previous: 0,
                current: 0,
            })
        );

        outputs[0].index = 3;
        outputs[1].index = 2;
        assert_eq!(
            compute_covenant_id(outpoint, &outputs),
            Err(CoreVerificationError::OutputIndicesNotStrictlyIncreasing {
                previous: 3,
                current: 2,
            })
        );
    }

    #[test]
    fn output_count_limit_fails_closed_before_hashing() {
        let (outpoint, outputs) = fixture();
        let limits = CoreVerificationLimitsV1 {
            max_authorized_outputs: 1,
            ..CoreVerificationLimitsV1::default()
        };

        assert_eq!(
            compute_covenant_id_with_limits(outpoint, &outputs, limits),
            Err(CoreVerificationError::TooManyAuthorizedOutputs {
                actual: 2,
                limit: 1,
            })
        );
    }

    #[test]
    fn per_output_script_limit_fails_closed_before_hashing() {
        let (outpoint, outputs) = fixture();
        let limits = CoreVerificationLimitsV1 {
            max_script_bytes_per_output: 1,
            ..CoreVerificationLimitsV1::default()
        };

        assert_eq!(
            compute_covenant_id_with_limits(outpoint, &outputs, limits),
            Err(CoreVerificationError::ScriptPublicKeyTooLarge {
                output_index: 2,
                actual: 2,
                limit: 1,
            })
        );
    }

    #[test]
    fn total_script_limit_fails_closed_before_hashing() {
        let (outpoint, outputs) = fixture();
        let limits = CoreVerificationLimitsV1 {
            max_total_script_bytes: 2,
            ..CoreVerificationLimitsV1::default()
        };

        assert_eq!(
            compute_covenant_id_with_limits(outpoint, &outputs, limits),
            Err(CoreVerificationError::TotalScriptBytesExceeded {
                actual: 3,
                limit: 2,
            })
        );
    }

    #[test]
    fn caller_cannot_disable_or_raise_resource_limits() {
        for limits in [
            CoreVerificationLimitsV1 {
                max_authorized_outputs: 0,
                ..CoreVerificationLimitsV1::default()
            },
            CoreVerificationLimitsV1 {
                max_authorized_outputs: HARD_MAX_AUTHORIZED_OUTPUTS_V1 + 1,
                ..CoreVerificationLimitsV1::default()
            },
            CoreVerificationLimitsV1 {
                max_script_bytes_per_output: HARD_MAX_SCRIPT_BYTES_PER_OUTPUT_V1 + 1,
                ..CoreVerificationLimitsV1::default()
            },
            CoreVerificationLimitsV1 {
                max_total_script_bytes: HARD_MAX_TOTAL_SCRIPT_BYTES_V1 + 1,
                ..CoreVerificationLimitsV1::default()
            },
        ] {
            assert!(matches!(
                limits.validate(),
                Err(CoreVerificationError::InvalidLimit { .. })
            ));
        }
    }
}
