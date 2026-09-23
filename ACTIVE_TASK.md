# ACTIVE TASK

## Status
IN PROGRESS — KASPA V2.1.0 MAINLINE UPGRADE LOCAL CHECKPOINT

## Objective
Preserve the fully qualified local Kaspa Gateway v2.1.0 upgrade candidate for stable Mainnet and Testnet10 as a source-bound, interruption-safe local checkpoint, while keeping Testnet13 explicit experimental opt-in and avoiding any remote publication/deployment.

## Scope
- Use only the authorized Windows `Server` worktree `feat/kaspa-v2.1.0-runtime-rebaseline-20260922`.
- Preserve the accepted same-EXE self-worker runtime and official Kaspa runtime bindings.
- Reuse qualified BUILD-014 and native runtime evidence unless product/runtime bytes change.
- Keep Testnet10 Bridge CPU-only: no external ASIC Stratum or Prometheus listener is expected.
- Preserve the qualified E2E CPU-only automation repair without rebuilding the product.
- Commit/checkpoint only to the local bare mirror. No real GitHub push, PR, merge, tag, release, deployment, or Production action is authorized.

## Current Phase
Product/runtime qualification, the planned four-case native matrix, the Testnet10 CPU-only E2E repair, affected npm/static validation, Graphify refresh, and repository continuity verification are complete. The remaining authorized boundary is exact-manifest review and local checkpoint persistence.

## Confirmed Progress
- BUILD-014-A4 is VERIFIED_SUCCESS for artifact SHA-256 `39A7E1D923414677F8510DCEC2B6EACA4F01317D7A1E1868E3F2821BCA12F3A2`.
- Native Node Mainnet and Node Testnet10 are VERIFIED_SUCCESS.
- Native External Bridge Mainnet is VERIFIED_SUCCESS.
- Native External Bridge Testnet10 is VERIFIED_SUCCESS_CPU_ONLY with `listener_count=0`, no ASIC/Prometheus listener, and progressing CPU hash samples `1,1,21`.
- E2E CPU-only remediation touches five E2E files only; targeted syntax, npm check, npm lint, deepmerge security smoke, runtime-port smoke, bridge-locator smoke, recovery-harness smoke, network-generation gate/regressions, diff-check, and Graphify refresh/query are PASS.
- Project continuity gate and continuity regression tests are PASS after current-state reconciliation.
- Historical build manifest and all external evidence receipts remain preserved. Final checkpoint identity is owned by the external operation journal plus the most recent manifest generated immediately before staging; do not rely on an older embedded manifest hash as forever-current.

## Current Blocker
`FAIL-0004`: exact native parent PID 4404 remains idle after `CloseMainWindow` and Tauri normal close attempts. All runtime owner statuses are stopped, child runtime workers are absent, and task runtime ports are free. Tauri `Window.destroy()` is blocked by capability policy and OS force-kill is not authorized. This blocker does not invalidate the four-case native runtime evidence; it blocks only the clean single-parent no-file-trace A/B close-root-cause experiment.

## Last Completed Action
Repository-native continuity reconciliation is verified: project continuity gate PASS, positive plus seven fail-closed regression cases PASS, and `git diff --check` PASS. A source-bound candidate manifest was also verified before this final checkpoint-wording update.

## Current Action
This file is part of the local checkpoint boundary. Before creating or updating that checkpoint, regenerate the final manifest for the exact current Git changed/untracked path set, require zero mismatch and zero path delta, stage only those manifest-bound files, and verify the cached diff.

## Next Action
Derive the checkpoint state dynamically. If the current branch has not yet been persisted to the `local` bare mirror, finish the verified manifest/cached-diff review, create one local `[skip ci]` checkpoint commit, and push only that branch to `local` without force. If the local mirror already resolves to that checkpoint, no further runtime action is authorized while `FAIL-0004` remains; proceed to owner review or wait for a separately authorized safe close capability.

## Verification Required
- Final path/hash manifest immediately before staging with zero mismatches and zero path delta.
- Cached path set exactly equal to the final manifest.
- `git diff --cached --check` and cached diff/file classification.
- Local commit tree identity and local bare-mirror ref identity.
- No additional product build, Cargo tests, native runtime cases, npm checks, or Graphify unless relevant inputs change.

## Completion Criteria
A durable local checkpoint commit exists on the task branch and local bare mirror with source-bound evidence/state reconciliation; all already-qualified product/runtime/E2E evidence remains correctly scoped; `FAIL-0004` remains explicitly BLOCKED rather than misreported as fixed; no real GitHub/release/deployment action occurs.

## DO NOT REPEAT
Do not rerun BUILD-014, the four planned native cases, the already-passed E2E/static/npm checks, continuity tests, or Graphify unless their validity predicates change. Do not force-kill PID 4404, start a second desktop parent, enable Testnet13, weaken Mainnet Stratum assertions, overwrite historical evidence, reset/stash/restore user work, or perform any real GitHub/release/deployment action.
