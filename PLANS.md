# EXECUTION PLAN

## Status

**ACTIVE — LIVE-SMOKE EFFECTIVE-SETTINGS CONSISTENCY REPAIR**

## Objective
Repair custom-endpoint live-smoke consistency and complete safe Windows short-smoke verification for stable mainnet/testnet10 without disturbing unrelated runtime ownership.

## Success Criteria
- Custom loopback RPC/P2P smoke endpoints remain identical in top-level and `effective_node` settings.
- Existing regression test proves non-default isolated ports and existing stable-network/loopback restrictions.
- Focused and full IPC tests plus strict lint/gates pass.
- Protected exact-head PR merges without bypass and post-merge `main` is green.
- Exact merged Windows binary passes isolated mainnet smoke and default testnet10 smoke.
- Existing service on 16110/16111 is untouched; smoke ports/processes are released after each run.
- Final state explicitly distinguishes short smoke from full synchronization/production readiness.

## Milestones
1. Verify repository/runtime reality and Windows test host — COMPLETE.
2. Run default testnet10 smoke — COMPLETE / PASS.
3. Reproduce isolated mainnet failure without touching existing service — COMPLETE.
4. Root-cause effective-settings drift and implement canonical synchronization repair — COMPLETE.
5. Local regression/stress/lint/continuity/Graphify qualification — IN PROGRESS.
6. Protected commit/PR/exact-head integration and post-merge verification.
7. Rebuild merged Windows binary; rerun isolated mainnet and default testnet10 smoke.
8. Clean smoke artifacts from working tree, reconcile durable state, return plan to inactive sentinel.
## Progress
The official testnet10 smoke passed on Windows `Server`. A mainnet smoke using isolated loopback ports reproduced a deterministic parent/self-worker settings mismatch while preserving the unrelated service. The validator is repaired through the canonical effective-settings synchronizer; focused regression and all 52 IPC integration tests pass.

## Completion Criteria
All success criteria above are met, no material warning/failure is hidden, the unrelated service remains unaffected, live-smoke evidence is recorded with its limitations, and repository state can be resumed without conversation context.

## Constraints
No force push, rebase, admin bypass, direct self-worker invocation, testnet12 live start, unrelated-service termination, immutable Desktop release mutation, or claim of full sync from short smoke.

## NEXT ACTION
Finish local qualification/checkpoint, protected-integrate the repair, then rebuild and live-verify exact merged `main` on Windows before final closure.