# DEC-0002: Windows E2E uses opt-in isolated runtime ports

- Status: VERIFIED
- Date: 2026-09-12
- Category: DECISION / TEST SAFETY
- Affected scope: Windows zero-touch E2E runtime validation only.

## Evidence
The real `Server` host has an unrelated user-managed Kaspa process listening on mainnet ports 16110/16111. The zero-touch matrix previously hard-coded those same ports, so running it unchanged would either fail on collision or tempt an unsafe stop/adoption of a foreign service.

Node and Bridge settings already expose editable RPC/P2P/upstream/instance ports through the real UI. Production defaults therefore do not need to change.

## Root Cause
The E2E validation profile assumed an otherwise idle Windows host and encoded default runtime ports directly in the matrix.
## Fix / Decision
Keep default E2E ports unchanged when no override is supplied. Allow validated environment overrides for mainnet/testnet10 RPC, P2P, and Bridge listener ports. The E2E harness writes those values through the same UI controls a user would edit and dispatches real `input`/`change` events before Start.

The isolated profile must also drive negative cross-network assertions and evidence port capture; no hard-coded default may silently remain in those checks.

## Verification
- `helpers/runtime-ports-smoke.mjs`: PASS for defaults, isolated mainnet values, and invalid-port fail-closed cases.
- E2E JavaScript syntax: PASS.
- E2E ESLint: PASS.
- `npm run check`: PASS.
- Real Windows zero-touch validation remains the next required proof.

## Regression Protection
`e2e/package.json` runs the runtime-port smoke as part of `npm run check`, and the zero-touch matrix derives runtime/evidence/isolation ports from one validated profile.
## Remaining Risk
The isolation profile protects port ownership only; real Windows lifecycle correctness still depends on the zero-touch/lifecycle matrix completing successfully on the packaged candidate.

## NEXT ACTION
Checkpoint the E2E isolation profile to the local bare remote, transfer that exact local Git history to `Server`, and run zero-touch with isolated mainnet ports while proving PID 33436 / 16110 / 16111 remain untouched.

## DO NOT REPEAT
Do not stop, adopt, or reconfigure an unrelated process merely to free default validation ports. Do not change production port defaults to accommodate the test harness.