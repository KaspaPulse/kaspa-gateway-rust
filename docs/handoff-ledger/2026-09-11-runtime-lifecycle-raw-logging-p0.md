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
