# BUG-0001: sysinfo refresh_processes API migration

- Status: VERIFIED
- Date: 2026-08-17
- Category: BUG
- Affected scope: process identity refresh in the desktop/runtime integration path.

## Evidence
Dependabot PR #52 upgraded `sysinfo` from 0.31.4 to 0.39.6 and exposed a compile failure because `System::refresh_processes` gained a `remove_dead_processes` argument. Commit `f3bd629126b8572bda32dd1f8da253d3491430b7` contains the qualified repair.

## Root Cause
Application code relied on the older one-argument `sysinfo` API. The dependency update changed the method contract, so unchanged call sites no longer compiled.

## Fix / Decision
Adapt the existing call minimally from `refresh_processes(ProcessesToUpdate::Some(&[pid]))` to `refresh_processes(ProcessesToUpdate::Some(&[pid]), true)` without unrelated runtime refactoring.

## Verification
The repaired exact PR head passed formatting, Cargo check, strict Clippy, Rust tests, npm audits, and required security/supply-chain contexts; post-merge push CI also passed. Historical run evidence is retained in `PROJECT_STATE.md` and Git history.

## Regression Protection
Blocking CI runs `cargo check --locked --workspace --all-targets`, strict Clippy, Rust tests, and project/runtime contract gates for dependency changes.

## Remaining Risk
Future `sysinfo` API migrations may require new compatibility work; dependency changes must be treated as code changes when compile evidence says so.

## NEXT ACTION
NONE — VERIFIED. Reuse this record if a later `sysinfo` update breaks the same path.

## DO NOT REPEAT
Do not assume Dependabot minor/maintenance labels prove API compatibility; exact-head compilation remains required.
