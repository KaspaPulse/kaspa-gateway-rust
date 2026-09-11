# BUG-0005: Runtime status omitted exact ownership identity

- Status: VERIFIED
- Date: 2026-09-11
- Category: BUG / P0 LIFECYCLE / OBSERVABILITY
- Affected scope: Integrated runtime worker registry and READY status reporting.

## Evidence
The durable owner lease and worker sidecar already carried exact process identity, but READY status exposed only the child PID plus network/appdir. Worker start-time/executable, parent identity, and effective runtime endpoint semantics were not available to the UI/operator status surface.

## Root Cause
`KgwParallelSelfWorker` retained only the spawned PID after startup. Identity values already verified during Start were discarded instead of being preserved for status reporting.

## Fix / Decision
Retain the already-verified worker and parent process identities plus RPC/P2P/Stratum settings in the managed worker record. READY status now emits worker PID/start-time/executable, parent PID/start-time/executable, network/appdir, and runtime endpoints without performing a new identity lookup or changing ownership semantics.

When no explicit P2P listener is passed, status reports `p2p=official-default`; it must not fabricate a concrete endpoint selected later by upstream.

## Verification
- Targeted READY ownership-status regression: PASS (1/1).
- Full `integrated_runtime_ipc_smoke_tests`: PASS (53/53).
- `cargo fmt --all -- --check`: PASS.
- Graphify incremental refresh and affected-path query: PASS.

## Regression Protection
The existing READY lease/normal-stop integration test now requires exact worker/parent identity fields and RPC/P2P/Stratum status evidence before terminal Stop removes ownership.

## Remaining Risk
`official-default` intentionally represents an upstream-selected P2P listener when `--listen` is omitted; real packaged Windows runtime evidence is still required before release closure.

## NEXT ACTION
Verify application CloseRequested/exit wiring invokes bounded `shutdown_all`, then verify stale/dead-owner reconciliation across relaunch before real Windows lifecycle validation.

## DO NOT REPEAT
Do not infer exact process ownership from PID alone, re-query mutable process identity when verified startup identity is already owned, or invent a concrete P2P endpoint when upstream owns the default selection.
