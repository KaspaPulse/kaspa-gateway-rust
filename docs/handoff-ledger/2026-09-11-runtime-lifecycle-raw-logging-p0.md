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
