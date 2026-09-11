# ACTIVE TASK

## Status
IN PROGRESS — POST-MERGE CI REGRESSION REPAIR

## Objective
Restore fully green protected `main` after PR #76 by fixing the confirmed post-READY self-worker integration-test race, preserving the merged continuity/npm hardening, and closing the task only after protected repair integration and post-merge verification.

## Scope
- Preserve PR #76 continuity and npm dependency-policy implementation.
- Repair only the confirmed Rust integration-test synchronization race; do not weaken production startup safety.
- Preserve exact, expiring npm upstream exceptions and duplicate stable-ID protection.
- Push this repair through a new protected PR, squash-merge without bypass, and verify post-merge `main`.
## Current Phase
PR #76 is merged as `3f8174c7e9e663da81e29eda5cd889de196eec7e`. Five observed post-merge workflows passed, but CI run `34516559028` failed only in `Run Rust tests` on `post_ready_worker_failure_is_non_running_durable_and_restartable_for_all_roles`. Root cause is confirmed and the deterministic test-only ACK repair is locally verified.

## Confirmed Progress
The prior E2E High npm finding is fixed on `main`; npm policy remains fail-closed and time-bounded. For the Rust regression, the repaired targeted test passed after a cold build, passed 20/20 consecutive repetitions under host load, and the full runtime IPC integration suite passed 52/52 with zero test-harness Rust warnings.

## Current Blocker
NONE locally. Protected remote qualification of the new repair head is still required.

## Last Completed Action
Added semantic READY acknowledgement to the deliberate post-READY exit fixture, preserved the real parent `try_wait()` startup safety check, added `REG-0001`, and verified the complete 52-test runtime IPC suite.
## Current Action
Finish local lint/clippy/continuity/Graphify qualification, commit this post-merge repair without CI skip, refresh `origin/main`, then push a new repair PR.

## Next Action
Run protected exact-head CI on the repair PR, root-cause any failure without retry-only behavior or bypass, squash-merge after green, and verify all post-merge `main` workflows including CI.

## Verification Required
Completed: targeted cold test PASS; 20/20 repeat PASS; full runtime IPC suite 52/52 PASS; test-harness Rust warnings = 0. Still required before push: focused clippy, continuity/npm policy gates, actionlint/YAML/PowerShell, `git diff --check`, and Graphify refresh/query. Remote exact-head and post-merge CI remain mandatory.

## Completion Criteria
Repair PR merged through protected squash; post-merge `main` CI/security workflows green; npm policy exceptions remain exact and unexpired; durable state/checkpoints are reconciled to the final repository truth.

## DO NOT REPEAT
Do not rerun PR #76, increase the 40 ms timer as a flaky-test workaround, weaken the parent startup `try_wait()` safety check, force unsupported npm dependency overrides, or use `--admin`/protection bypass.