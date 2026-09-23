# EXECUTION PLAN

## Status
**ACTIVE — GITHUB PUBLICATION / DESKTOP 0.1.3 RELEASE**

## Objective
Publish the protected, qualified v2.1.0 candidate through current-main integration, exact-head protected CI, squash-only merge, exact-main qualification, trusted Windows/macOS artifacts, verified provenance plus SPDX SBOM attestations, and the final Desktop 0.1.3 GitHub Release.

## Success Criteria
- Preserve protected checkpoint `7d670bb00682a7ceec6d40046798cdbe645121d8` / tree `1668bd42e7dc4e0e2fb778638450bdebf3143945`.
- Reuse BUILD-014/native 4-of-4/E2E evidence while their bound predicates stay unchanged.
- Keep `rust-version = "1.97.1"` as MSRV, test MSRV explicitly, and run supported stable quality/security on Rust 1.98.1.
- Keep all external Actions pinned to full commit SHAs.
- Produce trusted Windows/macOS artifacts from final merged main with provenance and SPDX 2.3 SBOM attestations, verify both, then publish Desktop 0.1.3 only after all release gates pass.
- Preserve `FAIL-0004` as a non-qualification blocker; never force-kill PID 4404.

## Milestones
1. Local v2.1.0 qualification — **VERIFIED_SUCCESS / CLOSED**.
2. Protected write-once receive checkpoint — **VERIFIED_SUCCESS / CLOSED**.
3. Fresh GitHub reconciliation — **VERIFIED_SUCCESS**; observed main `bb183816...`, one workflow-only drift commit, PR #95 open/behind, candidate absent remotely.
4. Candidate integration/rebase — **VERIFIED_SUCCESS**; clean rebase, no conflicts, protected checkpoint unchanged.
5. Validity-predicate evaluation — **VERIFIED_SUCCESS / REUSE**; Cargo.lock blob and all product/runtime/native/E2E predicates unchanged.
6. Rust CI + release supply-chain hardening — **IN PROGRESS**; MSRV/Stable split and SPDX SBOM workflow changes implemented locally, affected contract checks PASS.
7. Continuity + Graphify affected verification — **IN PROGRESS**.
8. Publication branch push + PR — **PENDING**.
9. Exact-final-PR-head required CI/reviews/rulesets — **PENDING**.
10. Squash merge + exact-main CI — **PENDING**.
11. Trusted Desktop Artifacts Windows/macOS — **PENDING**.
12. Provenance + SBOM attestation verification — **PENDING**.
13. Desktop 0.1.3 draft/release immutability gate/publication — **PENDING**.
14. Final continuity receipt — **PENDING**.

## Progress
Local qualification, checkpoint protection, fresh GitHub reconciliation, clean rebase, and validity-predicate reuse are complete. Publication hardening is implemented locally and its workflow contracts/full-SHA audit/diff-check are green. Continuity reconciliation is being validated now; publication branch push/PR and every remote release gate remain pending until their exact evidence exists.

## Completion Criteria
Completion requires: publication branch pushed; PR created; exact-final-head required CI including MSRV PASS; review threads resolved; squash merge under current rulesets; exact-main CI PASS; trusted Windows/macOS artifact workflow PASS on final main; provenance and SPDX SBOM attestations generated and independently verified; Desktop 0.1.3 draft uniquely bound to final main and qualified artifact run; immutable-release administration gate PASS; release published and post-publication tag/assets/digests/immutability verified; protected checkpoint preserved; final continuity receipt verified.

## Constraints
No protected-checkpoint mutation; no force-kill PID 4404; no Testnet13 live start; no broad product retest without predicate invalidation; no blind force push; no main protection weakening; no bypass of required CI; no historical receipt rewrite; no unverified release artifact.

## NEXT ACTION
Finish continuity and Graphify verification for current workflow/tooling changes, then commit reviewed scope. Immediately re-observe `main`; if unchanged, push candidate normally, open PR, update required-check governance safely to include the new MSRV context while removing duplicate baseline status-check ownership without reducing protection, then drive exact-head CI to PASS.
