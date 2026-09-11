# EXECUTION PLAN

## Status

**ACTIVE — CREDENTIAL RETIREMENT AND NPM RESIDUAL-RISK REVIEW**

## Objective
Close remaining repository-owned security hygiene without weakening supported dependency contracts or branch protection.

## Success Criteria
- `RELEASE_ADMIN_TOKEN` repository Actions secret is absent and verified.
- No tracked GitHub Actions workflow can reference the retired secret without failing the continuity/security gate.
- A durable `SEC-0003` record captures removal evidence without any secret value.
- `SEC-0002` and the machine-readable npm policy show a fresh 2026-09-11 review using current upstream versions.
- Existing npm exceptions are removed only if a supported path exists; otherwise their identities and 2026-10-10 expiry remain unchanged.
- Local gates, exact-head PR checks, protected squash merge, and post-merge `main` checks pass.

## Milestones
1. Reconcile current `main`, open task state, verify secret usage and npm upstream reality — COMPLETE.
2. Remove unused repository Actions secret and verify absence — COMPLETE.
3. Add durable retired-secret regression protection and security memory — IN PROGRESS.
4. Refresh npm review evidence without changing unsupported dependency boundaries.
5. Run local qualification and Graphify.
6. Commit/push/open protected PR and exact-head qualify.
7. Protected squash merge, post-merge `main` verification, final state reconciliation.

## Progress
Repository reality is reconciled on `main`. The unused `RELEASE_ADMIN_TOKEN` repository Actions secret was proven unreferenced, removed, and verified absent. Current npm registry review confirms no supported compatible path yet removes the exact residual Low/deprecation findings; the existing 2026-10-10 expiry remains unchanged. Durable `SEC-0003` evidence and workflow-reference regression protection are being added through the existing continuity gate.

## Completion Criteria
All success criteria above are met; the retired secret remains absent and cannot be reintroduced by tracked workflows unnoticed; npm exceptions remain exact and unextended; local and remote protected checks pass; post-merge `main` is verified; project state/checkpoint records reflect final reality.
