# ACTIVE TASK

## Status
IN PROGRESS — P0 RUNTIME LIFECYCLE & RAW LOGGING RELIABILITY

## Objective
Make network and bridge lifecycle trustworthy end-to-end: UI state must reflect real runtime ownership/readiness, Start/Stop/Restart/recovery/reconciliation must be deterministic, and Raw Log panes must contain ordered native stdout/stderr only.

## Scope
- Audit UI → Tauri IPC → supervisor/runtime → managed node/bridge → stdout/stderr → reconciliation → UI.
- Repair network and bridge lifecycle without replacing the accepted same-EXE self-worker architecture.
- Verify ownership identity, readiness, graceful/bounded stop, restart, crash recovery, orphan/stale-state reconciliation, network isolation, bridge attachment/listener readiness, and raw-log provenance/order/identity.
- Use the LOCAL bare Git remote for every intermediate commit/checkpoint; real GitHub push is blocked until the final local release gate passes.
- Validate real Windows runtime on `Server`; mocks alone are insufficient.

## Current Phase
Audit and reproduction. Local-first Git workflow is established and verified; no application-code remediation has been committed yet.

## Confirmed Progress
`local` is a bare repository at `/home/kas/kaspa-gateway-dev/local-git/kaspa-gateway-rust.git` seeded from the real project history. A probe branch/empty commit was pushed, fetched, SHA/history verified, then removed. `origin` fetch still points to GitHub while its push URL is locally disabled.

## Current Blocker
NONE. First task is to locate and reproduce the first real divergence in lifecycle/state/logging.

## Last Completed Action
Verified WORKTREE → LOCAL REMOTE → FETCH/VERIFY → HISTORY using an isolated local probe without any real GitHub push.

## Current Action
Trace actual Start/Stop/Status/Logs/reconciliation paths for node and bridge, compare frontend assumptions with runtime-owned truth, and reproduce the first divergence.

## Next Action
Run focused existing gates/tests and inspect the exact UI action handlers, IPC payloads/responses, worker registry/status/log contracts, and bridge readiness contract. Record each confirmed defect before fixing it.

## Verification Required
Focused regression per defect; full runtime IPC suite; frontend lifecycle/raw-log gates; workspace tests/security gates; production desktop build/artifact; real Windows lifecycle sequences for mainnet/testnet10 and supported bridge modes; no orphan/stale/false READY; final local audit.

## Completion Criteria
All local release-gate items pass; no known P0/P1 lifecycle/raw-log issue remains; final release commit/artifact/evidence is locally verified; only then one final push to real GitHub followed by exact-commit deployment and production verification.

## DO NOT REPEAT
Do not push intermediate work to GitHub, weaken ownership/READY/stop contracts, replace raw stdout/stderr with diagnostics, kill unrelated Kaspa processes, rewrite accepted runtime topology, or treat mocks/CI as real Windows runtime proof.
