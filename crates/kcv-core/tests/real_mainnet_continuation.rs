//! Authenticated-source _structural_ continuation fixture for R26, NOT a script
//! execution or a claim of independent UTXO-set consensus validation.
//! Public Kaspa REST and kascov indexer both show accepted parent 4ad9...:1
//! consumed as input zero by child 15c9... and a renewed c106... output zero.
use kaspa_consensus_core::{
    Hash,
    tx::{CovenantBinding, ScriptPublicKey, TransactionOutpoint, TransactionOutput},
};
use kcv_core::{
    ContinuationStructureErrorV1 as Error, ContinuationStructureEvidenceV1 as Evidence,
    validate_continuation_structure_v1,
};

const CID: &str = "c106688901a4dee5da12e98ccc6895e2798bad7d3061c823124e282c6e63b9a1";
const PARENT: &str = "4ad9dfd3c92059af21691410802f3ba4cf3568d4b9e78a083970767812fd2b95";
const CHILD: &str = "15c9b3eaea1e7ebd75714c662eeb107b0300193d4c1692409ce4ad119da074f9";

fn hash32(raw: &str) -> Hash {
    Hash::from_bytes(hex::decode(raw).unwrap().try_into().unwrap())
}

fn fixture() -> (
    TransactionOutpoint,
    TransactionOutpoint,
    TransactionOutput,
    Hash,
    TransactionOutput,
) {
    let covenant = hash32(CID);
    let prevout = TransactionOutpoint::new(hash32(PARENT), 1);
    let previous_output = TransactionOutput::with_covenant(
        42_169_893_362,
        ScriptPublicKey::from_vec(
            0,
            hex::decode("aa20237a19aa2838d4773a48deae64348307055933a3a409299007bca9eb48661b8787")
                .unwrap(),
        ),
        Some(CovenantBinding::new(1, covenant)),
    );
    let new_output = TransactionOutput::with_covenant(
        9_775_041_314,
        ScriptPublicKey::from_vec(
            0,
            hex::decode("aa20eb83e9921a817ac46e9d430a37a8b4f31bae8d2e843a84ce0edfeb9228d7bfe687")
                .unwrap(),
        ),
        Some(CovenantBinding::new(0, covenant)),
    );
    (prevout, prevout, previous_output, covenant, new_output)
}

fn check(
    expected_parent_outpoint: &TransactionOutpoint,
    consumed_prevout: &TransactionOutpoint,
    parent_output: &TransactionOutput,
    spent_utxo_covenant_id: Option<Hash>,
    child_output: &TransactionOutput,
    authorizing_input_index: u16,
    input_shape: (usize, u16),
) -> Result<(), Error> {
    validate_continuation_structure_v1(Evidence {
        parent_outpoint: expected_parent_outpoint,
        child_authorizing_prevout: consumed_prevout,
        parent_output,
        spent_utxo_covenant_id,
        child_output,
        child_authorizing_input_index: authorizing_input_index,
        child_input_count: input_shape.0,
        child_transaction_version: input_shape.1,
    })
}

#[test]
fn canonical_kcv_structurally_matches_real_accepted_mainnet_continuation() {
    assert_eq!(CHILD.len(), 64);
    let (prev, consumed, parent, id, next) = fixture();
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &next, 0, (6, 1)),
        Ok(())
    );
}

#[test]
fn canonical_kcv_continuation_rejects_spent_outpoint_or_utxo_identity_tampering() {
    let (prev, _, parent, id, next) = fixture();
    let bad_outpoint = TransactionOutpoint::new(hash32(PARENT), 2);
    assert_eq!(
        check(&prev, &bad_outpoint, &parent, Some(id), &next, 0, (6, 1)),
        Err(Error::ConsumedOutpointMismatch)
    );
    assert_eq!(
        check(&prev, &prev, &parent, None, &next, 0, (6, 1)),
        Err(Error::MissingSpentUtxoCovenant)
    );
    assert_eq!(
        check(&prev, &prev, &parent, Some(hash32(CHILD)), &next, 0, (6, 1)),
        Err(Error::SpentUtxoCovenantMismatch)
    );
    let mut missing_parent = parent.clone();
    missing_parent.covenant = None;
    assert_eq!(
        check(&prev, &prev, &missing_parent, Some(id), &next, 0, (6, 1)),
        Err(Error::MissingParentCovenant)
    );
}

#[test]
fn canonical_kcv_continuation_rejects_child_binding_and_authorizing_input_tampering() {
    let (prev, consumed, parent, id, next) = fixture();
    let mut bad_id = next.clone();
    bad_id.covenant = Some(CovenantBinding::new(0, hash32(CHILD)));
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &bad_id, 0, (6, 1)),
        Err(Error::ChildCovenantMismatch)
    );
    let mut bad_auth = next.clone();
    bad_auth.covenant = Some(CovenantBinding::new(1, id));
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &bad_auth, 0, (6, 1)),
        Err(Error::ChildAuthorizingInputMismatch)
    );
    let mut no_binding = next.clone();
    no_binding.covenant = None;
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &no_binding, 0, (6, 1)),
        Err(Error::MissingChildCovenant)
    );
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &next, 6, (6, 1)),
        Err(Error::InvalidAuthorizingInput)
    );
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &next, 0, (6, 0)),
        Err(Error::UnsupportedTransactionVersion)
    );
}

#[test]
fn canonical_kcv_continuation_does_not_invent_script_or_state_validity() {
    let (prev, consumed, parent, id, mut next) = fixture();
    // Consensus continuation binds an identity; only the actual covenant program
    // can decide if these changed value/state/script bytes are a legal transition.
    next.value += 1;
    next.script_public_key = ScriptPublicKey::from_vec(0, vec![0x51]);
    assert_eq!(
        check(&prev, &consumed, &parent, Some(id), &next, 0, (6, 1)),
        Ok(())
    );
}
