# DEC-0001: Protected Dependabot auto-merge scope

- Status: VERIFIED
- Date: 2026-09-07
- Category: DECISION
- Affected scope: `.github/workflows/dependabot-auto-merge.yml` and protected `main` integration.

## Evidence
PR #75 added the managed workflow and a strict no-bypass required-check ruleset. Current repository state verifies `KGW_DEPENDABOT_AUTOMERGE=enabled` and the workflow is present on `main`.

## Root Cause
Routine dependency maintenance had accumulated while failed security checks and mixed-risk grouped updates made unconditional unattended merging unsafe.

## Fix / Decision
Allow protected squash auto-merge only for authentic Dependabot semver minor/patch PRs after repository protections are satisfied. Exclude major updates and any dependency set containing `duckdb` from unattended auto-merge.

## Verification
The September maintenance batch was merged only after exact-head checks passed. Final `main` CI/security workflows passed on `9a7b18f76dd6184785a4cf972daa1431ee07138f`.

## Regression Protection
The workflow validates actor/repository/head identity and uses expected-head protected merge semantics. Repository rulesets require security/quality contexts and expose no bypass actors.

## Remaining Risk
A future minor/patch update can still be behaviorally risky; required checks and review of excluded/high-risk dependencies remain mandatory safeguards.

## NEXT ACTION
NONE — VERIFIED. Re-evaluate this scope if dependency risk or branch-protection policy materially changes.

## DO NOT REPEAT
Do not broaden unattended auto-merge to majors or `duckdb` merely for convenience, and do not weaken required checks to unblock automation.
