# ACTIVE TASK

## Status
IN PROGRESS — KASPA V2.1.0 GITHUB PUBLICATION AND DESKTOP 0.1.3 RELEASE

## Objective
Publish the fully qualified Rusty Kaspa v2.1.0 candidate through protected GitHub integration, exact-head CI, trusted Windows/macOS artifact qualification, provenance plus SPDX SBOM attestations, and an immutable Desktop 0.1.3 release while preserving the protected local qualification checkpoint.

## Scope
- Use the authorized Windows `Server` worktree `feat/kaspa-v2.1.0-runtime-rebaseline-20260922`.
- Protected qualification identity remains `7d670bb00682a7ceec6d40046798cdbe645121d8` / tree `1668bd42e7dc4e0e2fb778638450bdebf3143945`.
- Protected ref `refs/checkpoints/kaspa-v2.1.0-runtime-rebaseline-20260923` is write-once receive protected and must not be changed or deleted.
- Owner authorization now includes integration/rebase, GitHub push/PR/CI fixes, squash merge, exact-main qualification, trusted artifact workflows, provenance, SBOM, and final GitHub Release.
- Reuse BUILD-014, native 4/4, and E2E evidence unless their actual validity predicates change.
- Preserve Testnet13 as experimental explicit opt-in and preserve `FAIL-0004` without force-killing PID 4404.

## Current Phase
Fresh GitHub reconciliation is complete. The candidate was cleanly rebased onto observed main `bb183816e5c315107c64411c1793c89d8ec74e8e` without conflicts. The post-rebase publication head before current uncommitted hardening is `1a464f87926deace3f1d87b7edc013947085b72f`. Product/runtime, Git Cargo.lock blob, build-feature, native-harness, network-config, E2E, and artifact predicates remain unchanged, so prior product qualification is reusable.

## Confirmed Progress
- Local qualification and checkpoint-ref protection are CLOSED / VERIFIED_SUCCESS.
- Protected checkpoint target/tree remain exact; protection receipt SHA-256 is `E78DF5070D227C75BD014149517C37D11F9E60C1A6A48315AA9E4F230DE2AC46`.
- Fresh reconciliation found current main one workflow-only commit ahead of the historical candidate base, with zero product/runtime/Cargo.lock/harness/network overlap.
- Rebase onto exact observed main completed cleanly; protected checkpoint was unchanged.
- Cargo.lock Git blob is identical before/after rebase (`02b33e18a89d2ae2848cc2056aac807952207f7f`); a raw worktree SHA difference was line-ending-only and did not invalidate dependency evidence.
- Current scoped hardening separates MSRV Rust 1.97.1 from stable quality/security Rust 1.98.1 and adds pinned Syft 1.52.0 SPDX 2.3 SBOM generation/attestation/verification to trusted release workflows.
- Affected workflow contract gates, JavaScript syntax, full-SHA external Action audit, and `git diff --check` are PASS.

## Current Blocker
`FAIL-0004` remains `BLOCKED_NON_QUALIFICATION`: PID 4404 is preserved, force-kill is forbidden, and this does not block GitHub publication/release.

## Last Completed Action
The publication candidate was cleanly rebased onto the freshly observed GitHub main without conflicts. Post-rebase validity predicates were reconciled to REUSE after proving Cargo.lock Git blob identity; scoped Rust CI/SBOM hardening was implemented and its affected workflow contract checks passed.

## Current Action
Reconcile continuity for the newly authorized publication/release phase, run affected continuity/Graphify verification, review/stage/commit the scoped hardening, then re-observe GitHub main immediately before publication push.

## Next Action
If fresh main still matches the integrated base, push the publication branch normally, open a PR to `main`, capture exact PR head/tree/base, make the MSRV check required while preserving all existing protections, and drive exact-head CI/review to PASS before squash merge.

## Verification Required
- Project continuity gate and its fail-closed regression tests must pass on the reconciled publication state.
- Graphify must be incrementally refreshed once for changed tooling/docs and queried on the affected workflow-contract path.
- Final staged paths/diff must be reviewed and pass `git diff --cached --check`.
- Immediately before GitHub push, re-observe `main` and remote candidate state; after PR creation require exact-final-head protected CI, reviews/threads, mergeability, and rulesets.
- After merge require exact-main CI, trusted artifact qualification, provenance verification, SPDX SBOM verification, immutable-release gate, and post-publication identity/digest checks.

## Completion Criteria
The task is complete only after the candidate is integrated, pushed, reviewed and squash-merged under rulesets; exact-main CI passes; trusted Windows/macOS Desktop 0.1.3 artifacts pass their smoke/qualification workflows; provenance and SBOM attestations are verified; the existing verified release draft is published under the repository immutable-release policy; the protected local checkpoint remains unchanged; and final continuity evidence is reconciled.

## DO NOT REPEAT
Do not rerun BUILD-014, native 4/4, E2E qualification, or broad product testing while their validity predicates remain unchanged. Do not mutate the protected checkpoint, force-kill PID 4404, enable Testnet13, bypass required checks, rewrite historical receipts, or publish an unverified artifact.
