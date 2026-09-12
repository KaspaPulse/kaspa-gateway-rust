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
Local remediation is complete through `BUG-0010`, Milestone 7 non-Windows validation is green, and the baseline Milestone 8 Windows zero-touch matrix is VERIFIED PASS on exact local HEAD `3414f9ca...`: Mainnet/Testnet10 Node and external Bridge START -> READY -> raw-log/copy -> STOP passed on isolated ports. Desktop `0.1.2` remains local-only and immutable `0.1.1` remains untouched.

## Confirmed Progress
`local` remains the only push target for intermediate work and `origin` push remains disabled. Frontend raw logs/status truth, backend cross-network status responsiveness, and STARTING control semantics are repaired. READY status now retains and reports exact worker/parent identity plus runtime endpoint semantics without changing ownership behavior.

## Current Blocker
NONE. Source/IPC/frontend remediation and close/relaunch reconciliation are locally complete; full local release validation is the next gate, with real Windows validation intentionally pending behind it.

## Last Completed Action
Workspace Rust tests pass, the Desktop E2E-feature cargo check passes, and the full local gate advanced through raw-log checks. `BUG-0006` frontend regression and `kgw_true_raw_log_gate.ps1` now pass after preserving typed child rawText while rejecting only untyped transport envelopes.

## Current Action
Record the successful exact-HEAD Windows zero-touch evidence locally, then continue to the first unverified lifecycle scope: restart, forced crash recovery, application close/relaunch reconciliation, and in-process Bridge mode on `Server`.

## Next Action
Build focused real-Windows lifecycle coverage for restart/crash/relaunch and in-process Bridge mode without repeating the already-passed baseline matrix. No real GitHub push is allowed yet.

## Verification Required
Focused regression per defect; full runtime IPC suite; frontend lifecycle/raw-log gates; workspace tests/security gates; production desktop build/artifact; real Windows lifecycle sequences for mainnet/testnet10 and supported bridge modes; no orphan/stale/false READY; final local audit.

## Completion Criteria
All local release-gate items pass; no known P0/P1 lifecycle/raw-log issue remains; final release commit/artifact/evidence is locally verified; only then one final push to real GitHub followed by exact-commit deployment and production verification.

## DO NOT REPEAT
Do not push intermediate work to GitHub, weaken ownership/READY/stop contracts, replace raw stdout/stderr with diagnostics, kill unrelated Kaspa processes, rewrite accepted runtime topology, or treat mocks/CI as real Windows runtime proof.
