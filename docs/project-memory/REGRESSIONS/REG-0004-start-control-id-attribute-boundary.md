# REG-0004: Start-control ID attribute boundary

Status: VERIFIED
Service status: RESOLVED for the Rust static gate only.
Learning status: CLOSED with regression-first protection.
Task ID: KASPA_GATEWAY_100_PERCENT_RUST_MIGRATION_20260923
Date: 2026-09-25

## Root Cause
The adopted Rust Start/Stop static gate matched id= inside data-testid/custom-id names and missed real id attributes with whitespace before equals. Existing tests covered unspaced real IDs but omitted attribute-name boundaries.

Three new tests ran against unchanged production regexes and failed for the predicted defects. The repair requires attribute whitespace before id and accepts spacing before equals in all four existing patterns. No UI source, test ID, process ownership, IPC or raw-log behavior changed.

## Regression Protection
The new tests retain attribute-name and spacing variants plus the actual source fixture.

## Verification
Nine focused Rust tests pass on stable and MSRV 1.97.1. They cover sixteen non-ID variants, twelve real-ID position/spacing variants, actual current Node markup, and retained duplicate-control, panel-order and synthetic-log rejections. Strict all-target Clippy with KSSS features and diff checks pass.

## Evidence
C:/Users/abuha/KaspaGateway-Rust100-20260923/start-control-regressions-op075/.
Read regression-first.log, fixed-qualification-receipt.json and qualified-source-manifest.json. The existing task journal records OP075.

## Remaining boundaries
The behavioral frontend loader inspection was tool-blocked before execution; its prior failure is not repaired here. Post-change Graphify remains NOT_VERIFIED under the existing blocked transfer boundary. No browser, full application, full Start gate or cross-platform runtime PASS is claimed. The overall migration is PARTIAL.

## NEXT ACTION
Continue independent migration and required qualification. Treat the behavioral loader and post-change graph as separate unresolved dependencies; do not treat the static fix as full Start-gate PASS.

## DO NOT REPEAT
Do not remove test IDs or weaken true-ID, raw-log, panel-order or duplicate-control checks. Reuse fixed source-bound tests only when their inputs match.
