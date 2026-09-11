# BUG-0002: Runtime poll starvation and false STOPPED UI

- Status: IN PROGRESS
- Date: 2026-09-11
- Category: BUG / P0 LIFECYCLE
- Evidence classification: CONFIRMED
- Scope: Node and Bridge frontend lifecycle polling and raw-log refresh.

## Evidence
Existing lifecycle/raw-log/backend IPC suites were green, but a new deterministic frontend regression reproduced the real gap: when `kgw_runtime_owner_status_v1` remained pending, `kgw_kgw_runtime_logs_v1` was never requested, so startup raw output stayed blank. Repeated 700 ms refreshes could overlap, and transient status failures forced the UI to `Stopped` without terminal ownership evidence.

## Root Cause
Each Node/Bridge refresh awaited status before raw logs. Start/Stop hold the backend worker-registry mutex through readiness/terminal attestation, so status can block during exactly the period where raw output matters most. Refresh timers fired every 700 ms without single-flight control. Catch paths treated status transport failure as proof of `Stopped`.

## Fix / Decision
Use independent single-flight status and raw-log fetches per role/network. Fetch raw logs independently of status, skip status polling during a locally owned Start/Stop transition, retain truthful uncertainty as `Reconciling`, and schedule reconciliation after action completion/failure. Never fabricate STOPPED/RUNNING from a transport error.

## Verification
`tools/kgw_true_raw_log_frontend_tests.cjs` now proves raw output arrives while status is blocked, status remains single-flight, log polling continues while status remains blocked, and transient status failure never fabricates STOPPED for both Node and Bridge. Existing bridge readiness and parallel-self-worker gates pass.

## Regression Protection
The raw-log frontend regression test must fail if status once again serializes raw-log delivery, if overlapping status polls multiply, if log polling freezes behind a blocked status call, or if status transport failure maps to STOPPED.

## Remaining Risk
Real Windows UI/runtime validation is still required. Backend status lock scope remains globally serialized during lifecycle transitions; frontend no longer lies or starves raw logs, but cross-network lifecycle responsiveness must be validated and may require a backend transition-status improvement.

## NEXT ACTION
Audit backend transition/status concurrency and real bridge lifecycle readiness/stop/restart behavior; reproduce the next divergence before modifying it.

## DO NOT REPEAT
Do not fix this by increasing timeouts or poll intervals alone. Do not reintroduce status-first raw-log fetching or optimistic RUNNING/STOPPED state after transport failure.
