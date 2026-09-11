# ACTIVE TASK

## Status
COMPLETE — VERIFIED AND MERGED

## Objective
Repair the Windows live-smoke custom-endpoint effective-settings drift and complete stable-network short-smoke verification without disturbing unrelated runtime ownership.

## Scope
- Synchronize custom loopback RPC/P2P values through canonical `EffectiveNodeSettings`.
- Preserve stable-network, loopback-only, parent-identity, and self-worker equality checks.
- Protected squash integration only; no admin bypass, force push, or rebase.
- Verify exact merged `main` on Windows `Server` for isolated mainnet and default testnet10.

## Current Phase
No active implementation phase remains. REG-0002 is merged and live-verified; this file records the completed boundary.

## Confirmed Progress
PR #82 merged as `fb16b9a18b7e17621dfb1c280fef7951c8b819a7` after every required exact-head check passed. Post-merge CI, CodeQL/Rust security, Secret Scan, and supply-chain posture all passed on the same SHA.

Exact merged Windows short smoke passed for isolated mainnet on RPC `16120` / P2P `16121` with 8 peers and for default testnet10 on RPC `16210` with 7 peers. Parent-loss cleanup and relaunch reconciliation passed; all smoke-owned ports were released. Unrelated PID `35540` remained unchanged on `16110/16111`.

## Current Blocker
NONE for repository-owned repair work. Full synchronization/production capacity remains NOT VERIFIED because the smoke is intentionally short and the Windows test drive has less than the 640 GB production-disk guideline. Historical fine-grained PAT revocation remains separately NOT VERIFIED.

## Last Completed Action
Verified exact merged Windows mainnet/testnet10 short smoke on `Server`, preserved unrelated runtime ownership, and confirmed post-merge GitHub checks are green.

## Current Action
NONE. The `REG-0002` repair is closed; derive repository reality dynamically before starting new work.

## Next Action
No further action for `REG-0002`. Independently re-review npm exceptions by 2026-10-10; perform a full-sync production-readiness exercise only on hardware/storage meeting its explicit requirements; revoke the historical PAT only if the exact token can be safely identified.

## Verification Required
Completed: focused regression, IPC 52/52, fmt, strict Clippy, continuity/PowerShell/Graphify, PR #82 exact-head checks, post-merge `main`, exact merged Windows mainnet/testnet10 short smoke, cleanup, and unrelated-service preservation.

## Completion Criteria
MET for this task. Short smoke proves startup/RPC/peer connectivity/parent-loss cleanup/relaunch on stable networks; it does not prove full IBD or long-duration production capacity.

## DO NOT REPEAT
Do not kill unrelated PID `35540`, weaken effective-settings equality, reintroduce direct self-worker smoke shortcuts, start testnet12 without explicit experimental scope, or claim full production readiness from short smoke.