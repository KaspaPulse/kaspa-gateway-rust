# CURRENT STATE

- Verified at: 2026-09-11 during P0 Runtime Lifecycle & Raw Logging Reliability remediation.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Current HEAD: **VERIFY DYNAMICALLY**; latest committed local checkpoint before `BUG-0005` is `1f1abe63890b3488d7ac2b201c30c4b467683538`.
- Current branch: `fix/runtime-lifecycle-raw-logging-reliability-20260911`.
- Current remote main: **VERIFY DYNAMICALLY** before final integration; task baseline observation is `b88cc2571cb65ca30c1361ee3aa9b21eb551ea7c`.
- Working tree: **DIRTY** intentionally while `BUG-0005` ownership-observability checkpoint is being finalized; classify dynamically on resume.
- Local development remote: `/home/kas/kaspa-gateway-dev/local-git/kaspa-gateway-rust.git` (`local`) and current task branch tracks `local/...` only.
- Real GitHub fetch remote: `origin=https://github.com/KaspaPulse/kaspa-gateway-rust.git`.
- Real GitHub push path: **DISABLED LOCALLY** during remediation (`local-first-push-disabled://...`).
- Local checkpoints already verified on `local`: `e2b00ef` task open, `147b57e` BUG-0002, `dc41641` BUG-0003, `1f1abe6` BUG-0004.
- `BUG-0002`: raw logs/status polling decoupled; IPC uncertainty remains Reconciling instead of false STOPPED.
- `BUG-0003`: status no longer blocks behind another network lifecycle transition; registry contention returns reconciliation evidence.
- `BUG-0004`: Stop is not advertised before READY ownership exists.
- `BUG-0005`: READY status now exposes exact worker/parent process identity and endpoint semantics; targeted PASS and full runtime IPC PASS 53/53 before checkpoint commit.
- Existing IPC coverage re-verifies terminal Stop→reacquire, post-READY crash recovery/restart, parent-loss cleanup, shutdown ordering, and mainnet/testnet10 isolation.
- Architecture: accepted same-EXE self-worker runtime (`role:network` ownership); do not rewrite unnecessarily.
- Real Windows validation host: `Server`; mocks/Linux integration tests are not final Windows proof.
- Current P0 runtime lifecycle/raw-log correctness: **NOT VERIFIED** for release until close/relaunch reconciliation, local release gates, packaged Windows lifecycle matrix, and artifact validation complete.
- Live unrelated Kaspa service on `Server` is outside this remediation and must not be killed or adopted accidentally.

## NEXT ACTION
Checkpoint `BUG-0005` to the local bare remote, then verify CloseRequested/exit → bounded `shutdown_all` and stale/dead-owner application relaunch reconciliation.

## DO NOT REPEAT
Do not push intermediate commits to real GitHub, redo already-green restart/crash/isolation tests without a relevant code change, reduce ownership identity to PID-only evidence, or fabricate an explicit P2P endpoint when upstream owns the official default.
