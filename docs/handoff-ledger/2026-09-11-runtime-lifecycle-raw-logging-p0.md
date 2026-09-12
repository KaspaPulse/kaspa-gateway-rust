# CHECKPOINT: P0 RUNTIME LIFECYCLE & RAW LOGGING RELIABILITY

- Timestamp: 2026-09-11
- Task: Runtime lifecycle and raw logging reliability remediation.
- Status: IN PROGRESS — AUDIT / REPRODUCTION.

## LAST CONFIRMED STATE
Repository baseline is historical `main` `b88cc2571cb65ca30c1361ee3aa9b21eb551ea7c`. Task branch is `fix/runtime-lifecycle-raw-logging-reliability-20260911` and tracks only local bare remote `local`.

The real GitHub remote remains fetch-capable but its push URL is locally disabled. `remote.pushDefault=local`.

## COMPLETED / VERIFIED
- Inspected branch, HEAD, working tree, remotes, worktrees, upstream configuration, and alternate clone.
- Rejected the old shallow/grafted alternate clone as a development mirror.
- Created bare local remote from the real repository history and seeded branches/tags.
- Verified local commit → push → fetch → bare-ref SHA → ancestry using an isolated probe, then removed the probe.
- Read `AGENTS.md`, Graphify skill, ADR-0010, and Windows live-smoke runbook.
- Graphify audit identified integrated lifecycle/raw-log paths and the remaining legacy registered command surface.

## EVIDENCE / TESTS
`LOCAL_GIT_WORKFLOW_PROBE=PASS`; local mirror main equals task baseline; `origin` push URL is `local-first-push-disabled://...`; task branch upstream is `local/...`.

## NEXT ACTION
Trace node/bridge frontend action handlers, IPC response semantics, runtime registry/status/log contracts, and reproduce the first actual divergence before any application-code fix.

## DO NOT REPEAT
Do not recreate another mirror, use the shallow alternate clone, push intermediate work to GitHub, weaken accepted same-EXE ownership, or infer bugs from architecture alone without reproduction evidence.

## CHECKPOINT — BUG-0002 LOCAL FIX
Status: IN PROGRESS
Timestamp: 2026-09-11

## COMPLETED / VERIFIED
Reproduced frontend raw-log starvation behind a blocked status poll. Node/Bridge refresh now use independent status/log single-flight control; raw logs remain live during lifecycle transitions; transient status failure maps to Reconciling rather than false STOPPED; action completion schedules reconciliation.

## EVIDENCE / TESTS
`kgw_true_raw_log_frontend_tests.cjs` PASS with blocked-status/live-log, repeated-log, single-status, and false-STOPPED cases for both roles. `kgw_bridge_readiness_frontend_tests.cjs` PASS. `kgw_parallel_self_worker_runtime_gate.cjs` PASS. JS syntax and `git diff --check` PASS.

## NEXT ACTION
Audit backend global lifecycle-lock responsiveness and bridge mode lifecycle; reproduce the next actual divergence before modifying it.

## DO NOT REPEAT
Do not serialize raw log delivery behind status, treat status transport failure as terminal state, or stack 700 ms status polls.

## BACKEND STATUS ISOLATION CHECKPOINT
Status: VERIFIED
Timestamp: 2026-09-11

## LAST CONFIRMED STATE
`BUG-0003` reproduced cross-network status blocking behind a delayed mainnet Start and repaired worker status to fail fast with typed reconciliation evidence instead of waiting on the global registry mutex.

## COMPLETED / VERIFIED
- Targeted backend status-isolation regression PASS.
- Full runtime IPC suite 53/53 PASS.
- Frontend raw-log/readiness gates PASS.

## EVIDENCE / TESTS
Before repair, testnet10 status waited about 1.226s behind mainnet startup. After repair, the targeted test satisfies the 250ms bound and returns `registry_busy=true;runtime_state=reconciling`.

## NEXT ACTION
Audit Stop requested during STARTING and Restart sequencing for the same role/network.

## DO NOT REPEAT
Do not restore blocking status acquisition on the global worker registry or interpret registry contention as `Stopped`.

## STARTING CONTROL CHECKPOINT
Status: VERIFIED
Timestamp: 2026-09-11

## LAST CONFIRMED STATE
`BUG-0004` confirmed the UI offered Stop before READY even though backend graceful-stop ownership is not established until the worker is READY and registered.

## COMPLETED / VERIFIED
- Node Stop is disabled during STARTING/STOPPING and enabled only after READY/running truth.
- Bridge Stop follows the same contract.
- Missing/failed IPC remains Reconciling rather than optimistic Stopped/Startable.

## EVIDENCE / TESTS
Node start-button tests, Bridge readiness tests, and true raw-log frontend tests all PASS after the corrected lifecycle contract.

## NEXT ACTION
Verify Restart after terminal Stop, crash recovery, relaunch reconciliation, and ownership identity surfaces.

## DO NOT REPEAT
Do not advertise pre-READY Stop without an explicit backend cancellation protocol.

## OWNERSHIP OBSERVABILITY CHECKPOINT
Status: VERIFIED
Timestamp: 2026-09-11

## LAST CONFIRMED STATE
`BUG-0005` confirmed that READY status omitted exact worker/parent identity and endpoint evidence even though startup had already verified and owned those values.

## COMPLETED / VERIFIED
- Managed workers retain verified worker and parent PID/start-time/executable identity.
- READY status exposes exact identity plus RPC, Stratum, and explicit-vs-official-default P2P semantics.
- No new process-identity lookup or ownership semantic was introduced.

## EVIDENCE / TESTS
Targeted ownership-status regression PASS (1/1); full runtime IPC suite PASS (53/53); Rust formatting PASS; Graphify incremental refresh/re-query PASS.

## NEXT ACTION
Verify application close/exit wiring through bounded `shutdown_all`, then stale/dead-owner relaunch reconciliation.

## DO NOT REPEAT
Do not reduce ownership evidence to PID-only status or fabricate an explicit P2P endpoint when upstream selects the official default.

## CLOSE / RELAUNCH CONTRACT CHECKPOINT
Status: VERIFIED LOCALLY
Timestamp: 2026-09-11

## LAST CONFIRMED STATE
CloseRequested already prevented immediate window close and called owned `shutdown_all` before success-only `app.exit(0)`. Start holds the worker-registry transition until READY, so shutdown-all requested during STARTING waits rather than returning a false `stopped=0`.

## COMPLETED / VERIFIED
- Added regression protection for shutdown-all requested during delayed STARTING: it waits, then terminally stops the exact worker and removes lease/sidecar.
- Added STARTING parent-loss relaunch coverage: the exact child terminates after parent loss and relaunch reconciliation clears durable stale ownership without false READY.
- Extended the parallel self-worker runtime gate to protect CloseRequested prevent-close, single-flight shutdown, shutdown-all invocation, success-only exit, and retryable failure behavior.

## EVIDENCE / TESTS
Targeted close/startup test PASS; targeted STARTING parent-loss/relaunch test PASS; parallel self-worker runtime gate PASS; complete runtime IPC suite PASS 55/55; Rust formatting PASS; Graphify refresh/re-query PASS.

## NEXT ACTION
Checkpoint this regression protection to the local bare remote, then run the complete applicable local regression/security/build/package/artifact gates before Windows validation.

## DO NOT REPEAT
Do not treat shutdown-all during STARTING as an empty registry success: Start owns the registry lock through READY. Do not rely only on post-READY parent-loss tests for relaunch safety.

## MILESTONE 7 — CLIPPY CHECKPOINT
Status: RESOLVED / VERIFIED
Timestamp: 2026-09-11

## FAILURE EVIDENCE
The first full Rust quality pass reached desktop test Clippy and failed only on `clippy::redundant_closure` in the new shutdown-all-during-STARTING regression. Earlier `cargo fmt` and `cargo check --locked --workspace --all-targets` had completed successfully.

## FIX / VERIFICATION
Replaced the redundant zero-argument closure with the shutdown-all function pointer. `cargo fmt --all -- --check` PASS; desktop test Clippy with `-D warnings -A dead-code` PASS; the affected shutdown-all regression PASS; Graphify refresh/re-query PASS.

## NEXT ACTION
Continue Milestone 7 from the first unrun gate. Do not repeat the already-passed `cargo check --workspace --all-targets` or unrelated Clippy stages unless a later code change makes them relevant.

## DO NOT REPEAT
Do not hide the initial Clippy failure, and do not rerun the complete expensive Rust quality sequence merely to re-prove stages that were unaffected by this test-only cleanup.

## BUG-0006 RAW-LOG BOUNDARY CHECKPOINT
Status: VERIFIED LOCALLY
Timestamp: 2026-09-12

## LAST CONFIRMED STATE
Milestone 7 advanced through the Desktop E2E-feature cargo check and reached the true-raw-log gate. The gate exposed a missing explicit frontend boundary between legacy untyped transport envelopes and typed child raw-log records.

## COMPLETED / VERIFIED
- Node and Bridge reject untyped top-level legacy transport envelopes before typed raw-log ingestion.
- Typed `entry.rawText` remains opaque/verbatim even when official stdout/stderr resembles internal transport framing.
- Sequence ordering remains monotonic by typed record sequence.

## EVIDENCE / TESTS
Workspace Rust tests PASS; E2E-feature cargo check PASS; frontend raw-log regression PASS; `kgw_true_raw_log_gate.ps1` PASS after the boundary repair.

## NEXT ACTION
Checkpoint BUG-0006 to the local bare remote, then resume the full-local gate at zero-touch live E2E. Do not repeat the already-passed E2E-feature build or earlier raw-log stages.

## DO NOT REPEAT
Do not conflate typed child payload content with an untyped transport envelope, and do not restart Milestone 7 from its beginning after this checkpoint.
## WINDOWS E2E ISOLATION PROFILE CHECKPOINT
Status: VERIFIED LOCALLY
Timestamp: 2026-09-12

## LAST CONFIRMED STATE
Milestone 7 non-Windows gates are green and Desktop 0.1.2 is the local-only candidate. `Server` has an unrelated live mainnet service on 16110/16111, so default-port zero-touch execution would not be safe.

## COMPLETED / VERIFIED
- Added validated opt-in E2E runtime port profile with unchanged defaults.
- Zero-touch writes isolated RPC/P2P/upstream/Bridge ports through real UI controls before Start.
- Cross-network negative assertions and evidence capture use the same runtime profile.

## EVIDENCE / TESTS
Runtime-port smoke PASS; JS syntax PASS; E2E lint PASS; E2E `npm run check` PASS.

## NEXT ACTION
Checkpoint locally, transfer exact local history to `Server`, and run zero-touch with mainnet RPC/P2P overrides while preserving the unrelated 16110/16111 service.

## DO NOT REPEAT
Do not stop or adopt the unrelated service to satisfy test defaults, and do not change production defaults for validation convenience.
## BUG-0007 WINDOWS IPC SCHEMA DRIFT CHECKPOINT
Status: VERIFIED LOCALLY; exact Windows rerun pending checkpoint transfer.

Evidence:
- `Server` zero-touch on HEAD `583ac620...` reached the real Mainnet Start click.
- `kgw_kgw_apply_node_settings_v1` rejected `effectiveNodeSettings` before spawn: unknown field `rocksDbCacheSize`.
- Root cause: serde acronym casing produced `rocksdb*` keys while frontend contract uses `rocksDb*`.
- No validation runtime port opened; unrelated PID 33436 on 16110/16111 was preserved.

Fix/verification:
- Explicit serde rename + legacy alias for RocksDB preset/cache/WAL fields.
- Focused red→green serde regression PASS.
- Effective Node settings contract gate PASS.
- Runtime IPC suite PASS 56/56.

NEXT ACTION: local checkpoint → exact bundle transfer → rerun Windows zero-touch from Mainnet Node.
DO NOT REPEAT: do not weaken `deny_unknown_fields` or bypass typed effective settings via preview text.

## BUG-0008 WINDOWS ISOLATED P2P CHECKPOINT
Status: VERIFIED LOCALLY; exact Windows rerun pending checkpoint transfer.

Evidence:
- BUG-0007 payload passed Tauri and reached self-worker spawn on `Server`.
- RPC override `16120` applied, but spawn omitted `--listen` and Rusty Kaspa fell back to occupied `16111`.
- Child panic reported Windows AddrInUse; unrelated PID 33436 remained the owner of 16110/16111.

Fix/verification:
- E2E now enables the real `listenEnabled` checkbox before setting isolated P2P host/port.
- Red wiring smoke reproduced the omission before the fix.
- Runtime-port smoke PASS; E2E lint/check PASS.

NEXT ACTION: local checkpoint → exact bundle transfer → rerun Windows zero-touch.
DO NOT REPEAT: do not mutate production P2P defaults to accommodate validation isolation.

## BUG-0009 WINDOWS BRIDGE LOCATOR CHECKPOINT

Status: LOCAL FIX VERIFIED; exact Windows rerun pending.

COMPLETED / VERIFIED
- Windows `e81883b...` proved Mainnet/Testnet10 Node START -> READY -> raw log -> clipboard copy -> STOP with isolated ports.
- Mainnet Bridge failed before runtime start because the E2E harness targeted a nonexistent instance-port `data-testid`.
- DOM evidence identified the actual production id `bridge-mainnet-instancePort-1`.
- E2E read/write paths now use the real id, with compatibility fallback only on read.
- Bridge locator smoke, E2E lint, and E2E check PASS locally.

EVIDENCE / TESTS
- `e2e/helpers/bridge-locator-smoke.mjs`: red before fix, PASS after fix.
- `npm run lint`: PASS.
- `npm run check`: PASS.
- Windows artifact root from failing run: `C:\KGW-Local-Validation\artifacts\zero-touch-e81883b`.

NEXT ACTION
- Commit/push this checkpoint to local mirror only, transfer exact HEAD to `Server`, rerun Windows zero-touch, then continue from the next real divergence.

DO NOT REPEAT
- Do not rerun completed non-Windows gates; do not touch real GitHub; do not alter production Bridge runtime merely to accommodate a stale E2E locator.

## BUG-0010 ZERO-TOUCH EVIDENCE PORT PROFILE CHECKPOINT

Status: LOCAL FIX VERIFIED; exact Windows rerun pending.

COMPLETED / VERIFIED
- Exact Windows `e81883b...` result proved isolated Mainnet Node ports 16120/16121 but evidence validation still demanded 16110/16111.
- Red PowerShell regression reproduced all six Node/Bridge RequiredPorts mismatches under env overrides.
- Evidence required stages now consume the same `KGW_E2E_*_PORT` process environment contract as the E2E runtime matrix.
- Defaults remain the production/default validation ports when env overrides are absent.
- Invalid env port values fail closed.

EVIDENCE / TESTS
- `tools/kgw_zero_touch_result_writer_tests.ps1`: FAIL before fix on six isolated-port assertions; PASS after fix.

NEXT ACTION
- Commit/push to local mirror only, transfer exact HEAD to `Server`, rerun Windows zero-touch once to validate BUG-0009 and BUG-0010 together.

DO NOT REPEAT
- Do not maintain a second hard-coded evidence port table that can drift from the E2E execution profile.
