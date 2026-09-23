# CURRENT STATE

- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Active task: Rusty Kaspa v2.1.0 candidate publication and Desktop 0.1.3 release.
- Current branch: `feat/kaspa-v2.1.0-runtime-rebaseline-20260922`.
- Current HEAD: **VERIFY DYNAMICALLY** before every publication/integration decision; historical post-rebase committed observation before current hardening was `1a464f87926deace3f1d87b7edc013947085b72f`.
- Current remote main: **VERIFY DYNAMICALLY** immediately before publication/integration mutation; historical fresh observation for this phase was `bb183816e5c315107c64411c1793c89d8ec74e8e`.
- Working tree: **DIRTY** with intentional publication workflow/tooling/continuity changes; exact path set must be reviewed before staging.
- Some future GitHub/CI/release states are **NOT VERIFIED** until their owning operation runs; do not promote planned states to PASS.
- Protected qualified checkpoint: `7d670bb00682a7ceec6d40046798cdbe645121d8`, tree `1668bd42e7dc4e0e2fb778638450bdebf3143945`.
- Protected ref: `refs/checkpoints/kaspa-v2.1.0-runtime-rebaseline-20260923`; write-once receive protection VERIFIED_SUCCESS. Do not update/delete it.
- Fresh GitHub reconciliation observed `main=bb183816e5c315107c64411c1793c89d8ec74e8e`, tree `9f0c7e8219005132eb8684b3129a1ec5f77d0478`, with PR #95 as the only open PR and no remote candidate branch.
- Candidate integration: clean rebase onto that exact main completed with no conflicts. Post-rebase committed publication head before current hardening edits: `1a464f87926deace3f1d87b7edc013947085b72f`.
- Product/runtime hash set, Cargo.lock Git blob, build-feature set, native harness, network config, E2E relevant set, and qualified artifact remain unchanged after rebase. BUILD-014, native 4/4, and E2E qualification are REUSE.
- Current uncommitted scope is publication hardening/continuity only: MSRV 1.97.1 gate; stable quality/security Rust 1.98.1; trusted SPDX 2.3 SBOM generation/attestation/verification; release workflow contracts; continuity.
- Current affected validation PASS: artifact/release workflow contract gates, both gate syntax checks, full external Action SHA pinning audit, and `git diff --check`.
- Owner authorization includes GitHub publication/integration, CI repair, squash merge, trusted artifacts, provenance, SBOM, and final release. Do not request stage-by-stage reauthorization.
- `FAIL-0004=BLOCKED_NON_QUALIFICATION`; PID 4404 remains preserved. Force-kill and repeated close experiments remain forbidden.

## NEXT ACTION
Complete continuity/Graphify verification for the current scoped edits. Stage/review and commit only the intended publication-hardening/continuity paths. Re-observe GitHub main immediately before push; if the base is unchanged, push the publication branch normally, open the PR, capture exact-head identity, update required-check governance without weakening protection, and drive exact-head CI/review to PASS.

## DO NOT REPEAT
Do not rerun qualified build/native/E2E evidence unless its validity predicates change. Do not modify the protected checkpoint, rewrite historical receipts, force-push `main`, bypass checks, publish unverified artifacts, enable Testnet13, or force-kill PID 4404.
