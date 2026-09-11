# ACTIVE TASK

## Status
COMPLETE — VERIFIED AND MERGED

## Objective
Close repository-owned credential-retirement and npm residual-risk review without weakening dependency contracts, runtime/release invariants, or branch protection.

## Scope
- Retire the unused repository Actions secret `RELEASE_ADMIN_TOKEN` without handling its value.
- Prevent tracked workflows from silently depending on the retired secret again.
- Re-review the exact E2E npm residual-risk exceptions against current supported upstream versions.
- Preserve the existing 2026-10-10 npm review deadline unless a supported fix removes the exceptions.
- Integrate through protected squash PR flow and verify post-merge `main`.

## Current Phase
No active repository engineering phase remains for this task. PR #80 merged as historical security-hygiene baseline `1e650b6e96873d269f8a1b09c900a31abd7a7eb6`, and its post-merge `main` checks are green.

## Confirmed Progress
`RELEASE_ADMIN_TOKEN` is removed and verified absent by repository secret-name listing. The continuity/security gate rejects future workflow references to that retired name, with a fail-closed regression fixture. The 2026-09-11 npm review found no supported compatible path to remove the existing 3 Low / 2 deprecation exceptions, so their exact identities and 2026-10-10 expiry remain unchanged.

## Current Blocker
NONE for repository-owned work. Historical fine-grained PAT revocation remains account-level **NOT VERIFIED** because no exact token identifier/value is stored or safely exposed here. Live runtime smoke remains **NOT VERIFIED** because the supported smoke is Windows-only and the `KaspaGateway` device is offline.

## Last Completed Action
Verified PR #80 protected merge and successful post-merge `main` checks: quality, npm/continuity gates, Rust security analysis, Secret Scan, and supply-chain posture all passed on `1e650b6e96873d269f8a1b09c900a31abd7a7eb6`; the retired repository secret remained absent after merge.

## Current Action
NONE. This repository-owned security task is closed; derive repository reality dynamically before starting new work.

## Next Action
Review the exact npm exception contract no later than 2026-10-10. Run the official Windows live-network smoke for mainnet/testnet10 when the authorized `KaspaGateway` Windows device is online. Revoke the historical PAT only if an authorized account-level surface identifies that exact token.

## Verification Required
Completed: secret absence, zero workflow references, continuity gate/tests (7 negative cases), npm policy gate/tests, E2E lint/check, actionlint/YAML/PowerShell, Graphify warning-free health, PR #80 exact-head checks, and post-merge `main` checks.

## Completion Criteria
MET for repository-owned work. The repository secret is retired, regression protection is merged, npm review evidence is current and unextended, protected integration is green, and unresolved external items remain explicitly NOT VERIFIED.

## DO NOT REPEAT
Do not recreate `RELEASE_ADMIN_TOKEN`, guess/revoke unrelated PATs, force unsupported npm major overrides, broaden/extend the npm exception contract without evidence, or run the Windows-only live smoke on Linux `kas`.
