# CURRENT STATE

- Verified at: 2026-09-11 at start of P0 Runtime Lifecycle & Raw Logging Reliability remediation.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Current HEAD: **VERIFY DYNAMICALLY**; task baseline is historical `main` `b88cc2571cb65ca30c1361ee3aa9b21eb551ea7c`.
- Current branch: `fix/runtime-lifecycle-raw-logging-reliability-20260911`.
- Current remote main: **VERIFY DYNAMICALLY** before final integration.
- Working tree: **DIRTY** intentionally with task-state/checkpoint setup before the first local commit.
- Local development remote: `/home/kas/kaspa-gateway-dev/local-git/kaspa-gateway-rust.git` (`local`).
- Real GitHub fetch remote: `origin=https://github.com/KaspaPulse/kaspa-gateway-rust.git`.
- Real GitHub push path: **DISABLED LOCALLY** during remediation (`local-first-push-disabled://...`).
- Local workflow probe: **PASS** — local commit, push, fetch, direct bare-ref verification, and ancestry verification succeeded.
- Architecture: accepted same-EXE self-worker runtime (`role:network` ownership); do not rewrite unnecessarily.
- Frontend node/bridge tabs currently invoke `kgw_kgw_*` integrated runtime commands; legacy node/bridge commands remain registered and require audit for interference/drift.
- Real Windows validation host: `Server`.
- Current P0 runtime lifecycle/raw-log correctness: **NOT VERIFIED** until end-to-end reproduction and real Windows lifecycle matrix complete.
- Live unrelated Kaspa service on `Server` is outside this remediation and must not be killed or adopted accidentally.

## NEXT ACTION
Audit and reproduce lifecycle/raw-log divergences before code changes, starting with actual UI handlers and integrated runtime status/log semantics.

## DO NOT REPEAT
Do not push intermediate commits to real GitHub, do not bypass local remote checkpoints, and do not infer runtime health from prior short-smoke evidence.
