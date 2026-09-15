# BUG-0003: Global runtime registry blocks status across networks

- Status: VERIFIED
- Date: 2026-09-11
- Category: BUG / P0 LIFECYCLE
- Evidence classification: CONFIRMED
- Scope: Integrated runtime worker status across network lifecycle transitions.

## Evidence
A deterministic regression delayed mainnet READY by 1200 ms, then requested testnet10 node status while mainnet Start still owned the worker registry. Before the fix, testnet10 status waited about 1.226 seconds instead of returning promptly.

## Root Cause
`kgw_worker_start()` holds the global parallel self-worker registry mutex while waiting for READY. `kgw_worker_status()` previously used blocking `lock()` on the same registry, so an unrelated network status call could not proceed during another network lifecycle mutation.

This violated network-isolation expectations and amplified frontend status/log starvation during transitions.

## Fix / Decision
`kgw_worker_status()` now uses `try_lock()` and fails fast with typed transient reconciliation evidence when lifecycle mutation owns the registry:

- `registry_busy=true`
- `runtime_state=reconciling`

The frontend keeps the last confirmed runtime truth and renders reconciliation instead of fabricating `Stopped`.

## Verification
- Targeted regression `status_poll_does_not_block_behind_other_network_startup`: PASS.
- Full `integrated_runtime_ipc_smoke_tests`: 53/53 PASS.
- `tools/kgw_true_raw_log_frontend_tests.cjs`: PASS.
- `tools/kgw_bridge_readiness_frontend_tests.cjs`: PASS.
- `tools/kgw_parallel_self_worker_runtime_gate.cjs`: PASS.

## Regression Protection
The targeted Rust regression requires cross-network status to return within 250 ms while another network is still STARTING, and requires the typed `registry_busy=true;runtime_state=reconciling` contract rather than blocking or fabricating terminal state.

## Remaining Risk
A Stop requested for the same role/network while Start is still waiting for READY may still wait behind the Start transition. Restart sequencing remains to be audited next.

## NEXT ACTION
Reproduce and repair Stop-during-STARTING and Restart sequencing without weakening exact ownership, READY, or bounded shutdown guarantees.

## DO NOT REPEAT
Do not restore blocking status acquisition on the global worker registry. Do not interpret registry contention as `Stopped`, and do not hide this class of failure by increasing frontend timeouts.
