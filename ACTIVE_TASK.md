# ACTIVE TASK

## Status
IN PROGRESS — LIVE-SMOKE EFFECTIVE-SETTINGS CONSISTENCY REPAIR

## Objective
Repair the confirmed custom-endpoint drift in the Windows live-smoke parent/self-worker contract, integrate the fix through protected GitHub flow, and finish mainnet/testnet10 short live verification on `Server` without disturbing the unrelated mainnet service already using ports 16110/16111.

## Scope
- Synchronize custom loopback RPC/P2P smoke endpoints through the canonical `EffectiveNodeSettings` path.
- Extend the existing live-smoke regression test with non-default isolated ports.
- Preserve testnet12 prohibition, loopback-only validation, parent identity, and effective-settings equality checks.
- Never stop/kill the unrelated `kaspad` service solely to obtain default ports.
- Protected squash integration only; no admin bypass, force push, or rebase.

## Current Phase
`REG-0002` is reproduced and locally repaired. The focused regression test passed after a cold build and the full runtime IPC integration suite passed 52/52. Targeted strict Clippy qualification is PASS.

## Confirmed Progress
On Windows `Server`, official testnet10 live smoke passed with Rusty Kaspa 2.0.1, RPC ready, 8 peers, parent-loss cleanup, and relaunch reconciliation. Isolated mainnet smoke on RPC 16120 / P2P 16121 reproduced deterministic effective-settings drift while the unrelated service on 16110/16111 remained untouched and both isolation ports were cleaned up.
## Current Blocker
NONE locally. The remaining work is protected CI plus Windows rebuild/retest on the merged repair.

## Last Completed Action
Validated the root cause, changed the validator to use `apply_effective_node_settings`, added RPC/P2P synchronization assertions, passed the focused regression test, and passed all 52 runtime IPC integration tests.

## Current Action
Finish continuity/Graphify/PowerShell qualification and checkpoint this repair without changing unrelated runtime/release policy.

## Next Action
Commit without CI skip, push protected PR, exact-head qualify and squash-merge, then rebuild the exact merged Windows `main` and rerun isolated mainnet plus default testnet10 smoke.

## Verification Required
Focused regression PASS; full IPC 52/52 PASS; fmt/diff check PASS; strict Clippy; continuity gate/tests; PowerShell/YAML/actionlint where applicable; Graphify health/query; exact-head PR checks; post-merge `main`; repaired Windows smoke with port/service preservation.

## Completion Criteria
Protected repair merged green; post-merge `main` green; isolated mainnet and default testnet10 short smoke pass on the exact merged Windows binary; smoke-owned ports/processes clean up; unrelated PID/service remains untouched; durable state records short-smoke limits without claiming full synchronization/production capacity.

## DO NOT REPEAT
Do not kill the unrelated mainnet service, bypass the worker equality check, use unsupported direct self-worker invocation, broaden live-smoke network scope to testnet12, or classify short smoke as full production readiness.