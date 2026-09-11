# REG-0002: Live-smoke custom endpoint effective-settings drift

- Status: IN PROGRESS
- Date: 2026-09-11
- Category: REGRESSION
- Evidence classification: CONFIRMED
- Affected scope: Windows live-smoke parent/self-worker handoff for custom loopback RPC/P2P endpoints.

## Evidence
The official Windows `testnet10` smoke passed on `Server` with Rusty Kaspa 2.0.1, RPC readiness, 8 peers, parent-loss cleanup, and relaunch reconciliation. A mainnet smoke was then isolated from an unrelated existing `kaspad` service by using loopback RPC `16120` and P2P `16121`; ports `16110/16111` remained owned by the unrelated process.

The isolated mainnet parent exited before RPC readiness. Captured stderr reported: `effective Node settings do not match worker-owned compatibility arguments`. Both isolation ports were released and the unrelated service remained untouched.

## Root Cause
`kgw_validate_live_smoke_parent_settings_v1` accepted custom RPC/P2P endpoints and changed only top-level `NodeSettings.rpc_endpoint` / `p2p_listen`. It did not update `NodeSettings.effective_node.rpc_listen` / `p2p_listen`. The parent later serialized `effective_node` for the self-worker while passing the top-level values as compatibility arguments, so the worker correctly rejected the internally inconsistent pair.
## Fix / Decision
Build the custom smoke endpoint overrides into a cloned `EffectiveNodeSettings` value, then apply it through `NodeSettings::apply_effective_node_settings()`. This reuses the canonical synchronizer that keeps top-level compatibility fields and effective settings identical. Keep the app-directory override separate and preserve all existing loopback/testnet12 safety validation.

## Verification
The focused regression test now uses non-default mainnet endpoints (`127.0.0.1:16120` / `127.0.0.1:16121`) and asserts top-level/effective RPC and P2P equality. The focused test passed after a cold build; the complete `integrated_runtime_ipc_smoke_tests` suite passed 52/52. Clippy/protected CI and repaired Windows mainnet smoke remain required before closure.

## Regression Protection
`live_smoke_parent_accepts_only_valid_stable_network_runtime_settings` now fails if a future live-smoke override updates only compatibility arguments or only effective settings. Existing tests continue to prohibit testnet12 and non-loopback RPC smoke starts.

## Remaining Risk
The code repair is locally test-verified but not yet protected-integrated. Mainnet must be re-tested on the repaired Windows binary using isolated loopback ports so the unrelated service on 16110/16111 remains untouched.

## NEXT ACTION
Finish Clippy and continuity/Graphify qualification, integrate through a protected PR, rebuild the exact merged Windows main, rerun isolated mainnet smoke and default testnet10 smoke, then close durable runtime state.

## DO NOT REPEAT
Do not stop the unrelated `kaspad` service merely to free default mainnet ports, do not weaken the self-worker effective-settings equality check, and do not treat a larger sleep/retry as a fix for this deterministic configuration drift.