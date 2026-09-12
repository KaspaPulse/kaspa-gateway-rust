# EXECUTION PLAN

## Status
**ACTIVE — P0 RUNTIME LIFECYCLE & RAW LOGGING RELIABILITY**

## Objective
Rehabilitate network/bridge lifecycle and raw logging end-to-end using the accepted same-EXE ownership architecture, local-first Git development, real Windows runtime validation, and a single final GitHub publication.

## Success Criteria
- Network and bridge state machines are truthful across Start/Ready/Stop/Stopped/Restart/Crash/Recovery/Relaunch/Reconciliation.
- Exact process ownership identity is verified; no orphan, stale READY, false READY, or cross-network ownership.
- Bridge readiness proves node attachment and listener readiness for every supported mode.
- Native stdout/stderr is preserved, ordered, timestamped/identified by metadata, and separated from application diagnostics.
- UI state reconciles to runtime truth after failures, application close, and relaunch.
- Full local tests/security/build/artifact/Windows validation pass with no P0/P1 issue.
- Real GitHub receives exactly one final validated push; deployment uses exactly that commit.

## Milestones
1. Establish and verify local bare remote workflow; block real GitHub push — **COMPLETE**.
2. Audit full UI→IPC→runtime→process→logs→UI execution paths and reproduce defects — **COMPLETE LOCALLY**.
3. Repair network lifecycle defects with regression coverage and local checkpoint commits — **COMPLETE LOCALLY**.
4. Repair bridge lifecycle/readiness/attachment/listener defects with regression coverage — **COMPLETE LOCALLY AT SOURCE/IPC/FRONTEND LEVEL; WINDOWS PROOF REMAINS IN MILESTONE 8**.
5. Repair raw stdout/stderr ordering/provenance/diagnostics separation and frontend rendering — **COMPLETE LOCALLY**.
6. Repair reconciliation/orphan/stale-state/UI truth after crash, close, and relaunch — **COMPLETE LOCALLY**.
7. Run full local regression/security/build/artifact validation — **COMPLETE FOR NON-WINDOWS GATES; WINDOWS ARTIFACT/GUI PROOF CONTINUES IN MILESTONE 8**.
8. Run real Windows lifecycle matrix for mainnet/testnet10 and every supported bridge mode — **IN PROGRESS; BASELINE NODE + EXTERNAL BRIDGE ZERO-TOUCH PASS ON WINDOWS**.
9. Consolidate local commits, independently verify final production artifact/evidence, and pass local release gate — **PENDING**.
10. Perform one final real-GitHub push, deploy exact commit, verify production, and close durable state — **PENDING; REAL GITHUB REMAINS BLOCKED UNTIL FINAL LOCAL GATE**.

## Progress
Local remediation has BUG-0002 through BUG-0010 checkpointed; Milestone 7 non-Windows validation is green and Desktop 0.1.2 is local-only. Exact local HEAD `3414f9ca...` passed Windows zero-touch for Mainnet/Testnet10 Node and external Bridge, including START -> READY -> raw-log/copy -> STOP, isolated ports, real PIDs, and zero validation errors. Remaining Windows scope is restart/crash/relaunch reconciliation plus in-process Bridge mode.

## Completion Criteria
Every local release checklist item is evidenced; no known P0/P1 issue remains; final diff/artifact/evidence match the release commit; post-push remote HEAD and deployed commit match exactly.

## Constraints
No intermediate GitHub pushes, no force push/rebase of remote history, no admin bypass, no unnecessary runtime rewrite, no direct frontend process ownership, no fake logs, no weakening of testnet12 policy or ownership identity.

## NEXT ACTION
Record the successful baseline Windows matrix, then add focused restart/crash/relaunch and in-process Bridge validation on Server. Do not repeat already-passed baseline/non-Windows gates and do not push to real GitHub.
