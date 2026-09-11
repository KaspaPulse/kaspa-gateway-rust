# CURRENT STATE

- Verified at: 2026-09-11 during Windows live-smoke effective-settings repair.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Current HEAD: **VERIFY DYNAMICALLY**; task baseline is historical merged `main` `fad670eb29ac3b8a2bb3315032403dc22c060f2d`.
- Current task branch: `fix/live-smoke-effective-settings-overrides-20260911`.
- `Server` is the authorized Windows live-smoke device for this task; device `KaspaGateway` is intentionally not used.
- Existing unrelated `kaspad` service on `Server`: ports 16110/16111 owned by PID 35540; must remain untouched.
- Official testnet10 short smoke: **PASS** on baseline binary; Rusty Kaspa 2.0.1, RPC ready, 8 peers, parent-loss cleanup and relaunch reconciliation true; not fully synced.
- Isolated mainnet smoke reproduction: RPC 16120 / P2P 16121; **FAILED BEFORE RPC READY** with effective-settings/compatibility-argument mismatch; isolation ports cleaned and unrelated service preserved.
- Root cause: live-smoke validator changed top-level RPC/P2P only and left `effective_node` at defaults.
- Local fix: custom endpoints flow through `NodeSettings::apply_effective_node_settings()`; appdir remains a separate validated override.
- Regression evidence: focused custom-port test PASS; complete runtime IPC integration suite 52/52 PASS.
- Working tree: **DIRTY intentionally** with the current repair/state record; verify dynamically before commit.
- Current remote main: **VERIFY DYNAMICALLY** before push; historical task baseline recorded above.
- Desktop 0.1.1 release assets/tag/source binding are unchanged.
- Full sync/production capacity remains **NOT VERIFIED**; short smoke intentionally does not wait for IBD completion.

## NEXT ACTION
Finish Clippy/continuity/Graphify qualification, checkpoint and protected-integrate the repair, then rebuild exact merged `main` on `Server` and rerun isolated mainnet plus default testnet10 short smoke.

## DO NOT REPEAT
Do not stop PID 35540 or use its appdir, do not weaken effective-settings equality, do not run testnet12, and do not claim production readiness from short smoke.