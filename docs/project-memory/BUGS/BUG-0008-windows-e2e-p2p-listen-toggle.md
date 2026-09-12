# BUG-0008: Windows E2E P2P override did not enable --listen

Status: VERIFIED LOCALLY; Windows rerun pending exact checkpoint transfer.
Stable ID: BUG-0008
Severity: P0 validation blocker exposing real-port collision.
Discovered: 2026-09-12 during Windows zero-touch rerun after BUG-0007.

## Evidence
- Exact Windows validation HEAD: `4f6cf39f033177211f14402efaca1510e97562be`.
- RPC override applied correctly as `16120`.
- Spawn plan omitted `--listen`; Rusty Kaspa fell back to `0.0.0.0:16111`.
- Worker panicked with Windows `AddrInUse` because unrelated PID 33436 already owns `16111`.
## Root Cause
The isolated-port harness changed `listenHost` and `listenPort`, but the Node UI's `listenEnabled` checkbox defaults to false. Therefore `kgwNodeEffectiveNodeSettings()` emitted no `p2pListen` and the official runtime used its default P2P port.

## Fix / Decision
Add a real checkbox helper that dispatches `input` + `change`, and make `startNodeFromSettings()` explicitly enable `listenEnabled` whenever an isolated P2P port is requested. Production defaults remain unchanged.

## Verification
- Red runtime-port smoke failed before the fix.
- Runtime-port smoke PASS after the fix.
- E2E ESLint PASS.
- E2E syntax/deepmerge/runtime-port checks PASS.
## Regression Protection
`runtime-ports-smoke.mjs` now asserts that the zero-touch matrix explicitly enables the `listenEnabled` UI control before applying an isolated P2P host/port.

## Remaining Risk
The exact local checkpoint still requires real Windows zero-touch rerun. Any subsequent lifecycle divergence remains blocking.

## NEXT ACTION
Checkpoint BUG-0008 to the local bare remote, transfer exact HEAD to `Server`, rerun zero-touch with 16120/16121 isolation, and continue from the first subsequent divergence.

## DO NOT REPEAT
Do not change production P2P defaults merely to make E2E coexist with another service; isolation belongs in the validation harness and must use the same UI controls a user would use.
