# REG-0001: Post-READY self-worker integration-test race

- Status: VERIFIED
- Date: 2026-09-10
- Category: REGRESSION
- Evidence classification: CONFIRMED
- Affected scope: desktop runtime IPC integration-test fixture only; no production runtime protocol change.

## Evidence
PR #76 merged as `3f8174c7e9e663da81e29eda5cd889de196eec7e`. Its post-merge `main` CI run `34516559028` failed only in `Run Rust tests`, at `post_ready_worker_failure_is_non_running_durable_and_restartable_for_all_roles` for the bridge `official-external-node` fixture.

The child had already emitted READY and then intentionally exited with code 17. The parent nevertheless observed the process as exited during the narrow interval after reading READY but before registering the worker and returning the successful start result.

## Root Cause
The test fixture used a fixed 40 ms sleep after writing READY before its deliberate exit. The parent polls startup attestation every 25 ms and performs additional ownership/log/process bookkeeping before returning from start. Under runner load, the 40 ms timer could expire before the caller observed the READY state. This was a timing race in the test fixture, not a production worker-readiness failure.
## Fix / Decision
Add a test-only READY acknowledgement file. The self-worker still publishes the real startup attestation first, but when the deliberate post-READY-exit fixture is enabled it waits for an ACK from the test caller before starting the existing exit-delay timer. The caller writes that ACK only after `kgw_kgw_apply_node_settings_v1` returns successfully and `kgw_runtime_owner_status_v1` confirms `running=true` and `readiness=READY`.

The ACK environment variable is forwarded only by the `#[cfg(test)]` self-worker command path. Production startup behavior remains unchanged: a real worker that dies before start completion still causes startup to fail.

## Verification
The repaired targeted test passed after a cold build, then passed 20/20 consecutive repetitions under concurrent host load. The complete `integrated_runtime_ipc_smoke_tests` suite passed 52/52. A test-module-scoped `#[allow(dead_code)]` removes ten artificial warnings caused by path-including the complete runtime module; the rerun remained 52/52 with zero Rust warnings.

## Regression Protection
The existing integration test now synchronizes on the semantic boundary it is intended to test instead of wall-clock timing. The ACK is not issued until parent-visible ownership and READY state are both established. Exit code 17 and the restartability assertions remain unchanged, so loss of durable post-READY failure/restart semantics still fails CI.
## Remaining Risk
The repair removes the fixture race without weakening production startup checks. Remaining confidence depends on protected PR CI and the post-merge `main` CI run on GitHub Actions.

## NEXT ACTION
Push this verified repair through a protected PR, require exact-head CI to pass, squash-merge without bypass, then verify the post-merge `main` CI run is green.

## DO NOT REPEAT
Do not fix this recurrence by increasing the 40 ms sleep, retrying CI until it happens to pass, or weakening the parent `try_wait()` startup safety check. Synchronize the test on observable READY ownership instead.