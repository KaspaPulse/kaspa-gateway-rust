# CURRENT STATE

- Verified at: 2026-09-11 after protected PR #82 merge, post-merge `main` verification, and exact merged Windows stable-network short smoke.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Current HEAD: **VERIFY DYNAMICALLY**; historical `REG-0002` repair baseline: `fb16b9a18b7e17621dfb1c280fef7951c8b819a7`.
- Current task branch: **NONE AUTHORITATIVE WHILE IDLE**; derive dynamically before new work.
- Current remote main: **VERIFY DYNAMICALLY** before any integration decision; historical REG-0002 runtime-code baseline is `fb16b9a18b7e17621dfb1c280fef7951c8b819a7`.
- Working tree: **VERIFY DYNAMICALLY** before every task; historical exact-merged smoke boundary was CLEAN before this documentation reconciliation.
- `REG-0002`: **VERIFIED/CLOSED**. Custom live-smoke RPC/P2P endpoints now flow through canonical `EffectiveNodeSettings` and remain synchronized with top-level worker compatibility values.
- PR #82: MERGED through protected squash; all exact-head required checks passed.
- Post-merge `main`: CI, CodeQL/Rust security, Secret Scan, and OpenSSF Scorecard/supply-chain = SUCCESS on `fb16b9a...`.
- Windows host: `Server` is the verified live-smoke host used for this task.
- Exact merged mainnet short smoke: PASS on RPC `16120` / P2P `16121`, Rusty Kaspa 2.0.1, 8 peers, parent-loss cleanup and relaunch reconciliation PASS.
- Exact merged testnet10 short smoke: PASS on RPC `16210`, Rusty Kaspa 2.0.1, 7 peers, parent-loss cleanup and relaunch reconciliation PASS.
- Smoke-owned ports `16120/16121/16210`: FREE after completion.
- Unrelated existing service: PID `35540` remained unchanged and continued owning `16110/16111` throughout smoke testing.
- Both short smokes reported `IsSynced=false`; full synchronization/production capacity is **NOT VERIFIED**.
- Windows smoke drive free space was about 570 GB, below the runbook's 640 GB production-disk guideline; do not run/claim full production sync on that drive from this evidence.
- Repository secret `RELEASE_ADMIN_TOKEN`: REMOVED / VERIFIED ABSENT; historical associated PAT remains account-level NOT VERIFIED.
- Npm residual-risk review/expiry remains 2026-10-10 with no broadened exceptions.
- Desktop `0.1.1` immutable release/source boundary remains unchanged.

## NEXT ACTION
No further action for `REG-0002`. Start the next task from dynamically verified Git/GitHub/runtime reality. Re-review npm exceptions by 2026-10-10; perform full-sync production readiness only on a host meeting the documented storage/runtime requirements.

## DO NOT REPEAT
Do not stop PID `35540`, do not reopen the fixed custom-endpoint drift without new evidence, do not weaken self-worker equality/loopback/stable-network constraints, and do not treat short-smoke peer connectivity as proof of full synchronization.