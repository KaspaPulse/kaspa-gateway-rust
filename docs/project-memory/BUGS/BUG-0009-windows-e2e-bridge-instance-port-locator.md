# BUG-0009: Windows E2E Bridge instance-port locator drift

- Stable ID: `BUG-0009`
- Status: VERIFIED LOCALLY; Windows rerun pending exact checkpoint transfer.
- Area: Windows zero-touch E2E / Bridge UI locator contract

## Evidence
- Exact Windows HEAD `e81883ba813d04700c9b6d4cbfece979c25fb11a` completed Mainnet and Testnet10 Node READY/raw-log/copy/stop paths.
- Mainnet Bridge setup failed before bridge start with `Unable to set [data-testid=kgw-bridge-instance-field-mainnet-1-instancePort]: missing`.
- Captured DOM showed the real input id `bridge-mainnet-instancePort-1` and no matching `data-testid`.

## Root Cause
The E2E helper and matrix assumed a stale `data-testid` contract for Bridge instance ports. The production Bridge UI renders instance ports with the stable DOM id `bridge-${network}-instancePort-${instanceId}`.

## Fix / Decision
- Read Bridge runtime selection from the real instance-port DOM id first, retaining the old test-id only as a compatibility fallback.
- Write isolated Bridge instance ports through `setControlValueById` using the production DOM id.
- Do not change Bridge production defaults or runtime semantics for a validation-only locator defect.

## Verification
- Red `bridge-locator-smoke.mjs` reproduced the missing locator contract.
- After the fix: bridge locator smoke PASS, E2E ESLint PASS, E2E syntax/check PASS.

## Regression Protection
`e2e/helpers/bridge-locator-smoke.mjs` now asserts both the read and write paths use the real instance-port DOM id. The smoke is included in `npm run check`.

## Remaining Risk
Exact Windows zero-touch rerun is still required to prove Mainnet/Testnet10 Bridge start/readiness/raw-log/copy/stop on the corrected locator.

## NEXT ACTION
Checkpoint locally, transfer the exact local commit to `Server`, rerun Windows zero-touch, and continue from the first subsequent divergence only.

## DO NOT REPEAT
Do not add a new production `data-testid` merely to satisfy the harness unless the UI itself needs that contract. Prefer the existing stable production DOM id and keep validation changes isolated.
