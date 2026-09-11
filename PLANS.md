# EXECUTION PLAN

## Status

**ACTIVE — POST-MERGE CI REGRESSION REPAIR AND FINAL CLOSURE**

## Objective
Restore protected `main` to fully green after PR #76 by removing the confirmed post-READY self-worker test race, preserving the merged npm/continuity hardening, and completing durable post-merge closure.

## Success Criteria
- Keep production runtime startup safety unchanged.
- Replace wall-clock fixture timing with deterministic parent-observed READY synchronization.
- Preserve the exact npm dependency policy and duplicate project-memory ID guard.
- Pass focused local regression/lint/gate verification.
- Integrate through a new protected exact-head PR with no bypass.
- Verify every post-merge `main` workflow, especially CI, is green.
- Reconcile durable state to no active multi-stage plan.
## Milestones
1. PR #76 continuity/npm integration — COMPLETE; merged as `3f8174c7...`.
2. Detect post-merge `main` CI failure — COMPLETE; run `34516559028` isolated one Rust integration-test race.
3. Root-cause and deterministic test-only ACK repair — COMPLETE.
4. Local stress verification — COMPLETE: cold PASS, 20/20 repeat PASS, full IPC 52/52 PASS, zero test-harness warnings.
5. Finish lint/clippy/continuity/Graphify qualification and checkpoint — IN PROGRESS.
6. Commit, refresh main, push new protected repair PR.
7. Exact-head CI, protected squash merge, post-merge `main` verification.
8. Final durable state reconciliation and inactive-plan sentinel.

## Progress
The prior npm High finding is fixed on `main` and its residual upstream exceptions remain exact through 2026-10-10. The only observed post-merge blocker is the fixed-time post-READY fixture race. `REG-0001` records the recurrence, root cause, deterministic ACK fix, and regression evidence.

## Completion Criteria
All success criteria above are met, no material warning/failure is hidden, protected `main` is green after the repair merge, and repository state can be resumed without conversation context.
## Constraints
No production runtime mutation, no release/source-binding change, no force push, no rebase, no `--admin` merge, no retry-only flake handling, and no unsupported npm major override.

## NEXT ACTION
Complete focused local qualification, checkpoint the verified post-merge repair, then push a new protected PR and follow exact-head CI through post-merge verification.