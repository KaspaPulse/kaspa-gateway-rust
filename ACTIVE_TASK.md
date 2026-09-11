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
Local remediation and lifecycle reconciliation. `BUG-0002`, `BUG-0003`, and `BUG-0004` are committed and checkpointed to the local bare remote; `BUG-0005` ownership observability is locally verified and awaiting its checkpoint commit.

## Confirmed Progress
`local` remains the only push target for intermediate work and `origin` push remains disabled. Frontend raw logs/status truth, backend cross-network status responsiveness, and STARTING control semantics are repaired. READY status now retains and reports exact worker/parent identity plus runtime endpoint semantics without changing ownership behavior.

## Current Blocker
NONE. Real Windows runtime validation remains intentionally pending until local source remediation and relaunch reconciliation are complete.

## Last Completed Action
`BUG-0005` targeted ownership-status regression passed, the full runtime IPC suite passed 53/53, Rust formatting passed, and Graphify was incrementally refreshed/re-queried.

## Current Action
Checkpoint `BUG-0005` locally, then verify application CloseRequested/exit wiring through bounded `shutdown_all` and stale/dead-owner reconciliation across application relaunch.

## Next Action
Close any confirmed close/relaunch lifecycle gap with focused regression protection, then proceed to the remaining local release gates and real Windows lifecycle matrix before any final publication.

## Verification Required
Focused regression per defect; full runtime IPC suite; frontend lifecycle/raw-log gates; workspace tests/security gates; production desktop build/artifact; real Windows lifecycle sequences for mainnet/testnet10 and supported bridge modes; no orphan/stale/false READY; final local audit.

## Completion Criteria
All local release-gate items pass; no known P0/P1 lifecycle/raw-log issue remains; final release commit/artifact/evidence is locally verified; only then one final push to real GitHub followed by exact-commit deployment and production verification.

## DO NOT REPEAT
Do not push intermediate work to GitHub, weaken ownership/READY/stop contracts, replace raw stdout/stderr with diagnostics, kill unrelated Kaspa processes, rewrite accepted runtime topology, or treat mocks/CI as real Windows runtime proof.
