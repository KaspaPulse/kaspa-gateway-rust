# EXECUTION PLAN

## Status

**ACTIVE — PR #76 NPM SECURITY REPAIR AND EXACT-HEAD INTEGRATION**

## Objective
Complete the already-implemented repository-native continuity/security/knowledge-management lifecycle by repairing the E2E npm security failure discovered in PR #76, preserving fail-closed warning visibility, and integrating through the protected GitHub path.

## Success Criteria
- Preserve the verified continuity implementation and runtime/release invariants.
- Resolve the High E2E npm finding without weakening audit policy.
- Keep unavoidable residual Low/deprecation risk exact, documented, expiring, and fail-closed on drift.
- Push the repaired same branch to PR #76 and qualify the exact new head.
- Required checks pass; any new failure is root-caused and repaired without protection bypass.
- Protected squash merge completes and post-merge `main` is verified.
- Durable state/checkpoint files reflect final reality.

## Milestones
1. Reopen durable task state and verify repository/PR reality — COMPLETE.
2. Close local tooling/Graphify warnings — COMPLETE.
3. Open PR #76 and run first exact-head qualification — COMPLETE; one E2E npm audit failure found.
4. Root-cause and locally qualify npm repair/policy — COMPLETE.
5. Commit/push repaired head to PR #76 — NEXT.
6. Exact-head CI qualification and repairs if needed.
7. Protected squash merge and post-merge verification.
8. Reconcile durable state and return this file to the inactive sentinel.

## Progress
- PR #76 is open against `main`; first remote head `0ec7d01b...` passed all material lanes except `quality (rust + npm)`.
- Root cause was High GHSA-2883-xcg3-v3hh in E2E `js-yaml` 4.3.1 through WebdriverIO/Mocha.
- Local repair uses exact WebdriverIO 9.31.7 pins; `js-yaml` is 4.3.2 and npm audit is 0 Critical / 0 High / 0 Moderate.
- Residual GHSA-73rr-hh4g-fpgx Low chain plus two upstream deprecations are controlled by an exact machine-readable policy expiring 2026-10-10.
- Local gates, regression tests, actionlint, PowerShell AI gate, E2E lint/check, YAML parse, `git diff --check`, and Graphify final health/query all pass.
- Current phase: state reconciliation immediately before non-skip repair commit and push to the same PR.

## Completion Criteria
All success criteria above are met, no material warning is silently ignored, and the repository can resume from a concise final checkpoint without conversation context.

## Constraints
Preserve official runtime ownership, immutable Desktop 0.1.1 release boundaries, protected PR/squash integration, exact-head checks, no force push, no `--admin` bypass, and no unrelated application/runtime changes.

## NEXT ACTION
Create the non-`[skip ci]` npm repair/policy commit, re-fetch `origin/main`, push the same branch to PR #76, then follow exact-head CI to protected merge and post-merge verification.
