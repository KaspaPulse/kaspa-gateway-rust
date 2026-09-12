# CURRENT STATE

- Verified at: 2026-09-12 during Milestone 7 local validation for P0 Runtime Lifecycle & Raw Logging Reliability remediation.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Current HEAD: **VERIFY DYNAMICALLY**; latest committed local checkpoint is `8bdaff43c65cfaa9f40de177bff221ac80a6bfcd` (BUG-0006 raw-log boundary).
- Current branch: `fix/runtime-lifecycle-raw-logging-reliability-20260911`.
- Current remote main: **VERIFY DYNAMICALLY** before final integration; task baseline observation is `b88cc2571cb65ca30c1361ee3aa9b21eb551ea7c`.
- Working tree: **DIRTY** intentionally with Windows E2E isolated-port validation harness changes pending checkpoint commit; classify dynamically on resume.
- Local development remote: `/home/kas/kaspa-gateway-dev/local-git/kaspa-gateway-rust.git` (`local`) and current task branch tracks `local/...` only.
- Real GitHub fetch remote: `origin=https://github.com/KaspaPulse/kaspa-gateway-rust.git`.
- Real GitHub push path: **DISABLED LOCALLY** during remediation (`local-first-push-disabled://...`).
- Local checkpoints already verified on `local`: `e2b00ef` task open, `147b57e` BUG-0002, `dc41641` BUG-0003, `1f1abe6` BUG-0004, `b948849` BUG-0005, `2e1c545` close/relaunch protection, `5242804` strict-Clippy cleanup, `8bdaff4` BUG-0006.
- `BUG-0002`: raw logs/status polling decoupled; IPC uncertainty remains Reconciling instead of false STOPPED.
- `BUG-0003`: status no longer blocks behind another network lifecycle transition; registry contention returns reconciliation evidence.
- `BUG-0004`: Stop is not advertised before READY ownership exists.
- `BUG-0005`: READY status exposes exact worker/parent process identity and endpoint semantics; checkpointed locally at `b948849` with targeted PASS and runtime IPC PASS 53/53.
- `BUG-0006`: untyped top-level transport envelopes are explicitly rejected before typed raw-log ingestion, while typed official child `rawText` remains verbatim; frontend regression PASS and true-raw-log gate PASS.
- Release candidate: **Desktop 0.1.2 LOCAL ONLY**; package/Cargo/Tauri/lock metadata are consistent and `cargo check --locked -p kaspa-gateway-desktop` passes. Published immutable Desktop 0.1.1 is unchanged.
- Existing IPC coverage re-verifies terminal Stop→reacquire, post-READY crash recovery/restart, parent-loss cleanup, shutdown ordering, and mainnet/testnet10 isolation.
- Architecture: accepted same-EXE self-worker runtime (`role:network` ownership); do not rewrite unnecessarily.
- Real Windows validation host: `Server`; mocks/Linux integration tests are not final Windows proof.
- Close/relaunch contract: **VERIFIED LOCALLY** — CloseRequested is guarded by prevent-close/single-flight/shutdown-all/success-only exit; shutdown-all during STARTING waits for the owned transition; STARTING parent-loss relaunch reconciliation clears exact stale ownership. Runtime IPC PASS 55/55.
- Current P0 runtime lifecycle/raw-log correctness: **NOT VERIFIED** for release until full local release gates, packaged Windows lifecycle matrix, and artifact validation complete.
- Live unrelated Kaspa service on `Server` is outside this remediation and must not be killed or adopted accidentally.

## NEXT ACTION
Checkpoint the isolated Windows E2E port profile, transfer the exact local branch to `Server`, and run zero-touch with mainnet overrides while proving the unrelated 16110/16111 service remains untouched.

## DO NOT REPEAT
Do not push intermediate commits to real GitHub, redo already-green restart/crash/isolation tests without a relevant code change, reduce ownership identity to PID-only evidence, or fabricate an explicit P2P endpoint when upstream owns the official default.
