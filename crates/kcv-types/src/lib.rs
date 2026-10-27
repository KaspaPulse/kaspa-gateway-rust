#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CovenantId(pub [u8; 32]);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NetworkId {
    pub network: String,
    pub suffix: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Outpoint {
    pub transaction_id: [u8; 32],
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptPublicKey {
    pub version: u16,
    pub script: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisOutput {
    pub index: u32,
    pub value: u64,
    pub script_public_key: ScriptPublicKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofKind {
    ConsensusGenesis,
    SilPhysicalState,
    ArgentAuthoredState,
    SourceCorrespondence,
    BuildProvenance,
    IndependentReproduction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimStatus {
    ClaimMatch,
    ClaimMismatch,
    MissingEvidence,
    Unsupported,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaimId(String);

impl ClaimId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for ClaimId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for ClaimId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl AsRef<str> for ClaimId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedClaim {
    pub claim_id: ClaimId,
    pub status: ClaimStatus,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub code: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedClaim {
    pub claim_id: ClaimId,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifierIdentity {
    pub name: String,
    pub version: String,
    pub source_commit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterIdentity {
    pub adapter_name: String,
    pub adapter_version: String,
    pub adapter_commit: Option<String>,
    pub adapter_artifact_digest: Option<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClaimEvidenceSource {
    pub adapter_name: String,
    pub upstream_profile_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimEvidenceBinding {
    pub claim_id: ClaimId,
    pub sources: Vec<ClaimEvidenceSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimEvidenceBindingV2 {
    pub claim_id: ClaimId,
    pub sources: Vec<ClaimEvidenceSource>,
    pub reproduction_record_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolchainIdentity {
    pub toolchain_name: String,
    pub toolchain_version: String,
    pub target_triple: String,
    pub toolchain_commit: Option<String>,
    pub toolchain_artifact_digest: Option<[u8; 32]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReproductionRecordV1 {
    pub source_identity: String,
    pub source_digest_sha256: [u8; 32],
    pub build_recipe_identity: String,
    pub build_recipe_digest_sha256: [u8; 32],
    pub toolchain_identity: ToolchainIdentity,
    pub published_artifact_digest_sha256: [u8; 32],
    pub reproduced_artifact_digest_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReproductionRecordIdentityV1 {
    pub record_id: String,
    pub payload: ReproductionRecordV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildProvenanceRecordV1 {
    pub provenance_identity: String,
    pub provenance_digest_sha256: [u8; 32],
    pub source_identity: String,
    pub source_digest_sha256: [u8; 32],
    pub build_recipe_identity: String,
    pub build_recipe_digest_sha256: [u8; 32],
    pub builder_identity: String,
    pub toolchain_identity: ToolchainIdentity,
    pub subject_artifact_digest_sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamProfilePayload {
    pub upstream_kind: String,
    pub upstream_repository: String,
    pub exact_commit: String,
    pub specification_id: String,
    pub specification_maturity: String,
    pub document_digest: [u8; 32],
    pub vector_set_digest: [u8; 32],
    pub artifact_schema_version: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpstreamSpecProfile {
    pub profile_id: String,
    pub payload: UpstreamProfilePayload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeSyncState {
    Synced,
    Unsynced,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IndexCoverage {
    WatchedAddressesOnly,
    LiveFromDaa,
    BackfilledToDaa,
    ArchivalComplete,
    ExternalIndexUnverified,
    ExternalIndexLocallyVerified,
}

impl IndexCoverage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WatchedAddressesOnly => "WATCHED_ADDRESSES_ONLY",
            Self::LiveFromDaa => "LIVE_FROM_DAA",
            Self::BackfilledToDaa => "BACKFILLED_TO_DAA",
            Self::ArchivalComplete => "ARCHIVAL_COMPLETE",
            Self::ExternalIndexUnverified => "EXTERNAL_INDEX_UNVERIFIED",
            Self::ExternalIndexLocallyVerified => "EXTERNAL_INDEX_LOCALLY_VERIFIED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainObservationContext {
    pub network_id: NetworkId,
    pub node_software_identity: String,
    pub node_version_or_commit: String,
    pub sync_state: NodeSyncState,
    pub observation_daa_score: u64,
    pub observation_time_unix_millis: u64,
    pub rpc_method: String,
    pub index_coverage: Option<IndexCoverage>,
    pub observed_outpoint: Outpoint,
    pub observed_covenant_id: CovenantId,
    pub observation_digest: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_status_is_policy_neutral() {
        let allowed = [
            ClaimStatus::ClaimMatch,
            ClaimStatus::ClaimMismatch,
            ClaimStatus::MissingEvidence,
            ClaimStatus::Unsupported,
            ClaimStatus::Indeterminate,
        ];
        assert_eq!(allowed.len(), 5);
    }

    #[test]
    fn covenant_id_is_exactly_32_bytes() {
        let id = CovenantId([7; 32]);
        assert_eq!(id.0.len(), 32);
    }

    #[test]
    fn claim_id_is_an_owned_policy_neutral_identifier() {
        let id = ClaimId::new("consensus.genesis");
        assert_eq!(id.as_str(), "consensus.genesis");
        assert!(!id.is_empty());
        assert_eq!(id.clone().into_string(), "consensus.genesis");
        assert_eq!(id, ClaimId::from(String::from("consensus.genesis")));
    }

    #[test]
    fn index_coverage_preserves_the_frozen_explicit_classifications() {
        let coverage = [
            IndexCoverage::WatchedAddressesOnly,
            IndexCoverage::LiveFromDaa,
            IndexCoverage::BackfilledToDaa,
            IndexCoverage::ArchivalComplete,
            IndexCoverage::ExternalIndexUnverified,
            IndexCoverage::ExternalIndexLocallyVerified,
        ];
        assert_eq!(
            coverage.map(IndexCoverage::as_str),
            [
                "WATCHED_ADDRESSES_ONLY",
                "LIVE_FROM_DAA",
                "BACKFILLED_TO_DAA",
                "ARCHIVAL_COMPLETE",
                "EXTERNAL_INDEX_UNVERIFIED",
                "EXTERNAL_INDEX_LOCALLY_VERIFIED",
            ]
        );
    }

    #[test]
    fn upstream_profile_payload_excludes_profile_id_and_adapter_identity() {
        let payload = UpstreamProfilePayload {
            upstream_kind: "kcc".to_owned(),
            upstream_repository: "kaspanet/kccs".to_owned(),
            exact_commit: "abc123".to_owned(),
            specification_id: "KCC-1".to_owned(),
            specification_maturity: "Last Call".to_owned(),
            document_digest: [1; 32],
            vector_set_digest: [2; 32],
            artifact_schema_version: None,
        };
        let profile = UpstreamSpecProfile {
            profile_id: "sha256:example".to_owned(),
            payload: payload.clone(),
        };
        assert_eq!(profile.payload, payload);
    }
}
