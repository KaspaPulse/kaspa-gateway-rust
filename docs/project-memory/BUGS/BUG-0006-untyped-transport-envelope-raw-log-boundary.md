# BUG-0006: Untyped transport envelope lacked an explicit raw-log boundary

- Status: VERIFIED
- Date: 2026-09-12
- Category: BUG / P0 RAW LOG RELIABILITY
- Affected scope: Node/Bridge frontend typed raw-log ingestion and true-raw-log validation gate.

## Evidence
Milestone 7 reached `tools/kgw_true_raw_log_gate.ps1` after the E2E-feature cargo check and earlier raw-log checks passed. The gate failed because Node and Bridge had no explicit transport-envelope rejection helper and the frontend regression did not name sequence ordering.

Existing typed-entry tests also proved an important invariant: official child stdout/stderr may legitimately contain text that resembles an internal transport marker, and that typed `rawText` must remain byte-faithful.

## Root Cause
The frontend already consumed only typed `report.entries`, so legacy top-level transport text did not enter the buffer, but that boundary was implicit. The static gate also conflated an untyped transport envelope with the opaque `rawText` payload of an already-typed child record.
## Fix / Decision
Node and Bridge now reject only untyped top-level legacy transport envelopes before typed entries are accepted. The rejection helper is applied to `legacyTransportText`, never to `entry.rawText`. Typed child records remain opaque and are rendered/copied verbatim even when their official text resembles transport framing.

The true-raw-log gate now protects that distinction instead of requiring content filtering inside typed raw output.

## Verification
- Node and Bridge JavaScript syntax: PASS.
- `tools/kgw_true_raw_log_frontend_tests.cjs`: PASS.
- `tools/kgw_true_raw_log_gate.ps1`: PASS.
- Typed raw-log, child fixture, official sentinel, role/network isolation, and desktop debug build inside the gate: PASS.

## Regression Protection
Frontend tests require untyped Node/Bridge transport wrappers to be rejected before display/copy, require typed transport-looking official literals to remain verbatim, and explicitly assert that sequence ordering is preserved.
## Remaining Risk
Real packaged Windows lifecycle/raw-log validation remains pending under Milestone 8. No typed child output is filtered by content.

## NEXT ACTION
Checkpoint BUG-0006 to the local bare remote, then resume Milestone 7 at the first unrun step after the true-raw-log gate: zero-touch live E2E.

## DO NOT REPEAT
Do not filter typed `entry.rawText` merely because its contents resemble an internal transport marker. Reject only untyped transport framing before the typed-entry boundary.