# REG-0002: Live-smoke custom endpoint effective-settings drift

- Status: VERIFIED/CLOSED
- Date: 2026-09-11
- Category: REGRESSION
- Evidence classification: CONFIRMED AND LIVE-VERIFIED
- Affected scope: Windows live-smoke parent/self-worker handoff for custom loopback RPC/P2P endpoints.

## Evidence
Baseline Windows testnet10 short smoke passed on `Server`. A mainnet smoke isolated from unrelated PID `35540` used RPC `16120` / P2P `16121` and deterministically failed before RPC readiness with `effective Node settings do not match worker-owned compatibility arguments`; unrelated ports `16110/16111` remained untouched.

## Root Cause
`kgw_validate_live_smoke_parent_settings_v1` accepted custom RPC/P2P endpoints but changed only top-level `NodeSettings` fields. It left `effective_node.rpc_listen` / `p2p_listen` at defaults. The parent serialized `effective_node` for the self-worker while passing different compatibility arguments, so the worker correctly rejected the inconsistent contract.

## Fix / Decision
Clone the canonical effective settings, apply the custom loopback RPC/P2P values there, and use `NodeSettings::apply_effective_node_settings()`. Keep the app-directory override separate. Preserve loopback-only validation, stable-network scope, parent identity, and worker/effective-settings equality checks.

## Verification
- Focused custom-port regression test: PASS after cold build.
- Full `integrated_runtime_ipc_smoke_tests`: 52/52 PASS.
- `cargo fmt`, diff check, and targeted CI-equivalent strict Clippy: PASS.
- Continuity/PowerShell/Graphify qualification: PASS; Graphify corruption counters all zero.
- PR #82 exact-head required checks: PASS; protected squash merge, no admin bypass.
- Post-merge `main` at `fb16b9a18b7e17621dfb1c280fef7951c8b819a7`: CI, CodeQL/Rust security, Secret Scan, and supply-chain posture SUCCESS.

### Exact merged Windows live verification
- Mainnet isolated smoke on `Server`: PASS, Rusty Kaspa 2.0.1, RPC ready, 8 peers, parent-loss cleanup PASS, relaunch reconciliation PASS.
- Testnet10 official smoke on `Server`: PASS, Rusty Kaspa 2.0.1, RPC ready, 7 peers, parent-loss cleanup PASS, relaunch reconciliation PASS.
- Smoke-owned ports `16120/16121/16210`: FREE after completion.
- Unrelated PID `35540` retained ports `16110/16111` with unchanged process start time throughout.

## Regression Protection
`live_smoke_parent_accepts_only_valid_stable_network_runtime_settings` uses non-default isolated ports and asserts top-level/effective RPC and P2P equality. Existing tests continue to reject testnet12 and non-loopback smoke endpoints.

## Remaining Risk
Short smoke does not prove full initial block download or long-duration production capacity. Both smoke runs reported `IsSynced=false`; the Windows smoke drive had about 570 GB free, below the runbook's 640 GB production-disk guideline.

## NEXT ACTION
No further action for `REG-0002`. Any future full-sync production-readiness exercise must use a host/storage profile that meets the documented requirements.

## DO NOT REPEAT
Do not stop unrelated PID `35540`, bypass the effective-settings equality check, invoke the self-worker directly as a smoke substitute, start testnet12 without explicit experimental scope, or claim full production readiness from short-smoke peer connectivity.