# CHECKPOINT: LIVE-SMOKE-EFFECTIVE-SETTINGS-REPAIR-2026-09-11

- Status: IN PROGRESS — LOCALLY QUALIFIED BEFORE PROTECTED INTEGRATION
- Timestamp: 2026-09-11T15:20:10+00:00
- Task branch: `fix/live-smoke-effective-settings-overrides-20260911`.
- Starting merged baseline: `fad670eb29ac3b8a2bb3315032403dc22c060f2d`.
- Windows test host: `Server` (`aa0bce57-97a0-4c83-8b78-5be7b88109d7`).

## LAST CONFIRMED STATE
Official baseline testnet10 short smoke passed on Windows with Rusty Kaspa 2.0.1, RPC ready, 8 peers, parent-loss cleanup, and relaunch reconciliation. It did not complete IBD and is not full-production evidence.

An isolated mainnet smoke using loopback RPC `16120` and P2P `16121` failed before RPC readiness with `effective Node settings do not match worker-owned compatibility arguments`. The unrelated existing service PID `35540` on ports `16110/16111` remained untouched; both isolated ports were released.

## COMPLETED / VERIFIED

### Root Cause / Fix
`kgw_validate_live_smoke_parent_settings_v1` accepted custom loopback RPC/P2P values but wrote only top-level `NodeSettings` fields. The self-worker consumes the serialized `effective_node` contract and truthfully rejected the mismatch.

The repair clones the existing effective settings, applies custom RPC/P2P values there, and passes them through canonical `NodeSettings::apply_effective_node_settings()`. The appdir remains a separately validated override. No equality check, parent-identity check, stable-network restriction, or loopback restriction was weakened.

## EVIDENCE / TESTS
- Focused custom-port regression test: PASS.
- Full `integrated_runtime_ipc_smoke_tests`: 52/52 PASS.
- `cargo fmt`: PASS.
- `git diff --check`: PASS.
- Targeted CI-equivalent strict Clippy for production desktop and desktop tests: PASS.

## NEXT ACTION
Run continuity gate/tests, PowerShell AI gate, Graphify update/diagnose/query, and final state review. If all pass, commit without CI skip, re-fetch `origin/main`, push, open protected PR, exact-head qualify, squash-merge, and verify post-merge `main`.

After protected merge, update the isolated Windows clone to the exact merged SHA, rebuild the desktop release binary, rerun isolated mainnet smoke on `16120/16121`, then rerun default testnet10 smoke. Confirm smoke-owned ports/processes are clean and PID `35540` still owns the unrelated `16110/16111` service.

## COMPLETION CRITERIA
Protected repair merged green; post-merge `main` green; exact merged Windows binary passes both stable-network short smokes; unrelated service preserved; durable state distinguishes short smoke from full synchronization/production capacity.

## DO NOT REPEAT
Do not kill PID `35540` or repurpose its appdir, do not bypass the effective-settings equality check, do not invoke the self-worker directly as a substitute for the parent contract, do not start testnet12, and do not claim production readiness from a short smoke.
## FINAL PROTECTED INTEGRATION / LIVE VERIFICATION
- PR #82 merged through protected squash at `fb16b9a18b7e17621dfb1c280fef7951c8b819a7`; no admin bypass.
- Exact-head checks passed: quality, policy/audit/deny/machete, dependency review, Rust security-extended, Secret Scan, actionlint, Rust address fuzzing, and CodeQL.
- Post-merge `main` runs: CI `34618666371`, CodeQL `34618666115`, Secret Scan `34618666198`, OpenSSF Scorecard `34618666120` — SUCCESS.
- Exact merged Windows mainnet isolated smoke: PASS, Rusty Kaspa 2.0.1, 8 peers, parent-loss cleanup and relaunch PASS.
- Exact merged Windows testnet10 official smoke: PASS, Rusty Kaspa 2.0.1, 7 peers, parent-loss cleanup and relaunch PASS.
- Smoke ports `16120/16121/16210` released; unrelated PID `35540` remained unchanged on `16110/16111`.
- Both short smokes remained unsynced; full IBD/production capacity is not proven.

## FINAL STATE
Repository-owned `REG-0002` work is VERIFIED/CLOSED. No further task-specific engineering action remains. Full-sync production-readiness remains a separate exercise requiring documented hardware/storage capacity.