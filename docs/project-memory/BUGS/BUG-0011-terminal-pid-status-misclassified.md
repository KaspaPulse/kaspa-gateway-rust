# BUG-0011: Terminal PID status misclassified as live ownership

- Stable ID: `BUG-0011`
- Status: IN PROGRESS
- Area: Windows lifecycle recovery E2E / owner-status interpretation.

## Evidence
- Exact local HEAD `f7a82e78663460054c7f0d12e67273a43fd0bee3` ran focused recovery on Windows `Server`.
- Exact-owner force kill succeeded for Mainnet PID `34372` and Testnet10 PID `33984` after executable + start-time verification.
- Runtime owner status then reported `running=false;readiness=FAILED` with `runtime terminated unexpectedly after READY`.
- The status deliberately retained PID/start-time/executable as terminal crash evidence.
- `waitForStopped()` timed out because it rejected any status that still contained `pid=<n>`.

## Root Cause
The E2E helper conflated retained terminal process identity with live process ownership. Production runtime reconciliation was already correct; the harness classification was wrong.

## Verification
- Added `isStoppedOwnerStatus()` as the single terminal-state predicate.
- E2E lint/check, recovery harness smoke, `git diff --check`, and Graphify refresh/query PASS locally.
- Focused Windows rerun is still required before changing this record to VERIFIED.

## Regression Protection
- Explicit `running=false` is terminal even when PID identity remains present.
- `running=true` can never classify as stopped.
- `waitForStopped()` preserves terminal PID and parsed fields as evidence.

## NEXT ACTION
Checkpoint to local mirror, transfer exact HEAD to `Server`, and rerun only `lifecycle-recovery.e2e.js`.

## DO NOT REPEAT
Do not remove terminal PID/start-time/executable from runtime status merely to satisfy a test; those fields are durable crash evidence, not liveness.