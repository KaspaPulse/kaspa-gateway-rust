# BUG-0010: Zero-touch evidence required-port profile drift

- Stable ID: `BUG-0010`
- Status: VERIFIED LOCALLY; Windows rerun pending exact checkpoint transfer.
- Area: Windows zero-touch evidence validation

## Evidence
- Windows result for exact HEAD `e81883ba813d04700c9b6d4cbfece979c25fb11a` captured Mainnet Node RPC/P2P on isolated `16120/16121` but validation still reported missing `16110/16111`.
- `tools/kgw_zero_touch_evidence.ps1` hard-coded RequiredPorts for all required stages.
- A red PowerShell regression reproduced six mismatches for isolated Mainnet/Testnet10 node and bridge ports.

## Root Cause
The E2E runtime matrix gained opt-in isolated port environment variables, but the evidence validator retained fixed default ports. Runtime evidence and post-run validation therefore described different contracts.

## Fix / Decision
- Evidence required stages now read the same `KGW_E2E_*_PORT` process environment variables as the E2E runtime profile.
- Defaults remain unchanged when overrides are absent.
- Invalid overrides fail closed unless they are integer TCP ports in 1024..65535.

## Verification
`tools/kgw_zero_touch_result_writer_tests.ps1` failed before the fix on all six isolated-port assertions and PASSED after the fix while also preserving default-port assertions.

## Regression Protection
The result-writer test now verifies both default and isolated required-port profiles and restores process environment values after the test.

## Remaining Risk
Exact Windows rerun is required to prove the final evidence result accepts isolated Node/Bridge ports while all runtime cases complete.

## NEXT ACTION
Checkpoint locally, transfer exact HEAD to `Server`, rerun Windows zero-touch, and continue from the first subsequent real divergence only.

## DO NOT REPEAT
Do not hard-code validation ports separately from the runtime E2E profile. Keep the evidence validator and execution profile on the same environment contract.
