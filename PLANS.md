# EXECUTION PLAN

## Status
**ACTIVE — LOCAL CHECKPOINT BOUNDARY; FAIL-0004 DEFERRED**

## Objective
Persist the fully qualified local v2.1.0 upgrade candidate with source-bound evidence for official stable Mainnet/Testnet10 runtimes, truthful Testnet10 CPU-only Bridge automation, durable continuity state, and one local checkpoint. Remote integration/publication is outside the current authorization.

## Success Criteria
- Official stable Mainnet and Testnet10 runtime bindings remain on Rusty Kaspa v2.1.0; Testnet13 stays separate experimental opt-in.
- BUILD-014 artifact identity is preserved and qualified without redundant rebuilds.
- Planned native matrix passes for Node Mainnet, Node Testnet10, External Bridge Mainnet, and CPU-only External Bridge Testnet10.
- Testnet10 automation does not fabricate Bridge instances, Stratum, or Prometheus listeners.
- E2E/static/npm/Graphify and continuity checks for the current candidate pass.
- Final repository state, handoff, known blocker, and source manifest are durable and resumable.
- One reviewed checkpoint commit is mirrored only to the local bare remote.

## Milestones
1. Rebaseline official Kaspa stable runtime inputs to v2.1.0 and preserve Testnet13 separation — **COMPLETE LOCALLY**.
2. Run affected compile/tests/static gates and repair only invalidated surfaces — **COMPLETE / REUSED VALID EVIDENCE**.
3. Produce BUILD-014-A4 exact desktop artifact — **VERIFIED_SUCCESS**.
4. Qualify planned native runtime matrix on Windows `Server` — **VERIFIED_SUCCESS 4/4**.
5. Reconcile Testnet10 Bridge automation to CPU-only semantics — **VERIFIED_SUCCESS**.
6. Restore exact E2E dependencies and close affected npm/static checks — **VERIFIED_SUCCESS**.
7. Refresh Graphify and produce post-build E2E/source-bound evidence — **VERIFIED_SUCCESS**.
8. Reconcile continuity state and `FAIL-0004`; continuity gate/regressions — **VERIFIED_SUCCESS**.
9. Final manifest/cached-diff review and local checkpoint persistence — **READY; DERIVE COMPLETION DYNAMICALLY FROM GIT/LOCAL MIRROR**.
10. Real GitHub push/PR/merge/tag/release/deployment/Production — **NOT AUTHORIZED IN CURRENT TASK**.

## Progress
The product artifact and all four planned native cases are qualified. Testnet10 External Bridge is correctly CPU-only with no external ASIC listener and observed CPU hash progression. The five-file E2E harness delta is statically green and Graphify-refreshed. Continuity gate and regression tests are green. The remaining authorized action is the source-bound local checkpoint. The only runtime-side blocker is `FAIL-0004`: the exact idle desktop parent does not terminate through normal close paths, while runtime workers and task ports are already clean.

## Completion Criteria
Immediately before checkpoint persistence, a final manifest must exactly equal the current Git changed/untracked path set and all hashes must match. The cached diff must equal that manifest and pass `git diff --cached --check`. The resulting commit tree and `local` bare-mirror ref must match. `FAIL-0004` remains documented and unbypassed. No remote publication is part of completion.

## Constraints
No force-kill, no duplicate desktop parent, no Testnet13 runtime start, no product rebuild or native-matrix replay without invalidation, no reset/stash/restore of existing work, no weakening of runtime/evidence assertions, and no real GitHub/release/deployment operation.

## NEXT ACTION
Derive the branch/local-mirror state. If not yet checkpointed, generate the final manifest from the exact current Git path set, stage/review only those files, create one local `[skip ci]` checkpoint commit, and push only to `local` without force. If already checkpointed and clean, stop; the next unresolved item is `FAIL-0004`, which requires a separately authorized safe close capability or natural parent exit before any no-file-trace A/B.
