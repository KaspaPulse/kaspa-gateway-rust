# BUG-0004: UI advertised Stop before startup ownership was READY

- Status: VERIFIED
- Date: 2026-09-11
- Category: BUG / P0 LIFECYCLE
- Evidence classification: CONFIRMED
- Scope: Node and Bridge frontend lifecycle controls.

## Evidence
Node and Bridge button-state code enabled Stop while the local transition was `starting`. The backend does not publish a fully managed worker into the runtime registry until READY, so a Stop request during that window cannot use the normal owned graceful-stop contract immediately.

The frontend therefore advertised a lifecycle action that the backend did not own yet.

## Root Cause
UI button semantics were ahead of backend lifecycle semantics: `stopEnabled` included the `starting` state even though pre-READY cancellation is not part of the accepted same-EXE ownership contract.

## Fix / Decision
Stop remains disabled while STARTING or STOPPING. It becomes available only after backend READY/running truth is confirmed. Startup failure or missing IPC remains `Reconciling` until status proves a terminal state.

## Verification
- `tools/kgw_start_button_frontend_tests.cjs`: PASS.
- `tools/kgw_bridge_readiness_frontend_tests.cjs`: PASS.
- `tools/kgw_true_raw_log_frontend_tests.cjs`: PASS.

## Regression Protection
Node and Bridge frontend tests now require Stop to remain disabled during STARTING and require missing/failed IPC to preserve `Reconciling` rather than restoring an optimistic startable/stopped state.

## Remaining Risk
Restart behavior after a confirmed terminal Stop and crash/relaunch reconciliation still require explicit local and Windows validation.

## NEXT ACTION
Audit and verify Restart (Stop → terminal proof → Start → READY), post-READY crash recovery, application close/relaunch reconciliation, and exact ownership identity surfaces.

## DO NOT REPEAT
Do not expose Stop during STARTING unless a future backend change introduces an explicit owned pre-READY cancellation contract with exact process identity and regression coverage.
