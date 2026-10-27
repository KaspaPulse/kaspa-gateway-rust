//! Genuine accepted Mainnet transaction e088b95ddf249065705c8a06742463fe50bc08bd224957c0e38141a93fdea4bb.
//! Evidence: KGW_RC4_R23_MAINNET_COVENANT_KASPA_REST_20261026.json,
//! cross-checked with KGW_RC4_R23_MAINNET_COVENANT_INDEXER_20261026.json.
//! Computes the claim independently; it does not trust the API covenant_id field.
use kcv_core::{CoreVerificationError, compute_covenant_id, verify_covenant_id};
use kcv_types::{ClaimStatus, CovenantId, GenesisOutput, Outpoint, ScriptPublicKey};

const EXPECTED: &str = "f817e29db006b747489493ac168f386afcae82a741cc133002996e41e89a5752";

fn hash32(hex_text: &str) -> [u8; 32] {
    hex::decode(hex_text)
        .expect("fixture hex")
        .try_into()
        .expect("32 bytes")
}

fn real_mainnet() -> (Outpoint, Vec<GenesisOutput>) {
    let outpoint = Outpoint {
        transaction_id: hash32("018219ebca34717b10203db57d71ba3b61c3b265a909ccb248bc01c0457b13c8"),
        index: 1,
    };
    let outputs = [
        (
            0,
            "aa2063bb08c6f101c5a01b4898342f1e36bbc2ed3674e77349b1513711dba55ba04f87",
        ),
        (
            1,
            "aa206204c471522d0d57ed941a78096cf03b4826cb4f0f86f6660c68ac692d5f19c687",
        ),
    ]
    .into_iter()
    .map(|(index, script)| GenesisOutput {
        index,
        value: 30_000_000,
        script_public_key: ScriptPublicKey {
            version: 0,
            script: hex::decode(script).expect("authentic output script"),
        },
    })
    .collect();
    (outpoint, outputs)
}

#[test]
fn canonical_kcv_computes_genuine_mainnet_genesis_using_official_consensus() {
    let (outpoint, outputs) = real_mainnet();
    let expected = CovenantId(hash32(EXPECTED));
    assert_eq!(compute_covenant_id(outpoint, &outputs).unwrap(), expected);
    let claim = verify_covenant_id(expected, outpoint, &outputs).unwrap();
    assert_eq!(claim.claim.status, ClaimStatus::ClaimMatch);
    assert_eq!(claim.actual, expected);
}

#[test]
fn canonical_kcv_detects_authentic_mainnet_outpoint_and_value_mutations() {
    let (outpoint, mut outputs) = real_mainnet();
    outputs[0].value += 1;
    assert_eq!(
        verify_covenant_id(CovenantId(hash32(EXPECTED)), outpoint, &outputs)
            .unwrap()
            .claim
            .status,
        ClaimStatus::ClaimMismatch
    );
    let (mut outpoint, outputs) = real_mainnet();
    outpoint.index += 1;
    assert_ne!(
        compute_covenant_id(outpoint, &outputs).unwrap(),
        CovenantId(hash32(EXPECTED))
    );
}

#[test]
fn canonical_kcv_rejects_reordering_duplicate_and_missing_authorized_outputs() {
    let (outpoint, mut outputs) = real_mainnet();
    outputs.reverse();
    assert!(matches!(
        compute_covenant_id(outpoint, &outputs),
        Err(CoreVerificationError::OutputIndicesNotStrictlyIncreasing { .. })
    ));
    let (outpoint, mut outputs) = real_mainnet();
    outputs[1].index = outputs[0].index;
    assert!(matches!(
        compute_covenant_id(outpoint, &outputs),
        Err(CoreVerificationError::OutputIndicesNotStrictlyIncreasing { .. })
    ));
    let (outpoint, mut outputs) = real_mainnet();
    outputs.clear();
    assert_eq!(
        compute_covenant_id(outpoint, &outputs),
        Err(CoreVerificationError::EmptyAuthorizedOutputs)
    );
}

#[test]
fn canonical_kcv_detects_script_and_version_mutations() {
    let (outpoint, mut outputs) = real_mainnet();
    outputs[0].script_public_key.script[0] ^= 1;
    assert_ne!(
        compute_covenant_id(outpoint, &outputs).unwrap(),
        CovenantId(hash32(EXPECTED))
    );
    let (outpoint, mut outputs) = real_mainnet();
    outputs[0].script_public_key.version += 1;
    assert_ne!(
        compute_covenant_id(outpoint, &outputs).unwrap(),
        CovenantId(hash32(EXPECTED))
    );
}
