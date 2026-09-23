# EXECUTION PLAN

## Status
ACTIVE — OWNED PROGRAMMING IMPLEMENTATION 100% RUST MIGRATION

## Objective
Replace all project-owned programming implementation outside Rust with Rust while preserving necessary declarative/platform artifacts, KSSS governance, and Windows/macOS/Linux functionality.

## Success Criteria
- Owned non-Rust programming implementation source count reaches zero.
- Non-Rust execution wiring is removed except proven platform-required thin adapters with no project logic.
- `cargo run --locked -p xtask -- language-policy strict` passes.
- Rust build/check/test/Clippy/FMT/MSRV/security and required CI pass.
- Desktop support remains valid on Windows, macOS, and Linux.
- No Production, DNS, Cloudflare, live runtime, credential, or protected-checkpoint mutation occurs.

## Milestones
1. Recovery/reconciliation and comprehensive baseline inventory — **VERIFIED_SUCCESS**.
2. Isolated Server branch + durable continuity — **VERIFIED_SUCCESS**.
3. Rust `xtask` + fail-closed language policy/inventory + CI enforcement — **VERIFIED_LOCAL / COMMITTED** at `33461f6511c69b457c5f3dd069b54322d9a236a0`.
4. Python/Shell security and CI helper migration — **IN PROGRESS**: five generic Python policy/test scripts removed after Rust parity; ClusterFuzz build logic moved to Rust; KSSS Python remains.
5. Node/CJS repository gate migration to Rust — **PENDING**.
6. PowerShell helper migration to Rust with Windows behavior preserved — **PENDING**.
7. WebdriverIO/Node E2E replacement with Rust-native desktop/WebDriver harness — **PENDING**.
8. JavaScript frontend replacement with Rust/WASM while preserving Tauri IPC/UI contracts — **PENDING**.
9. Remove Node/Python/PowerShell/Shell implementation dependencies and update workflows/configuration — **PENDING**.
10. Zero-debt strict guard + cross-platform/security/MSRV qualification — **PENDING**.
11. GitHub PR exact-head CI, squash merge, exact-main qualification, durable closeout — **PENDING**.

## Progress
The guard now reports Rust source inventory 81, owned non-Rust source debt 116, execution-wiring debt 14, unapproved debt 0, and one platform-required thin adapter exception. Rust tooling tests are 23/23 PASS on both stable and MSRV; MSRV check, stable Clippy/FMT, advisory policy, and ClusterFuzz adapter syntax pass. Workflow lint/integrated CI remains remote qualification work.

## Completion Criteria
The plan closes only when strict language policy proves zero owned non-Rust implementation debt, all affected/final checks pass on supported platforms, KSSS/supply-chain controls remain intact, protected checkpoint is unchanged, final PR is squash-merged under repository rules, exact-main CI passes, and durable closeout records final SHA/tree/inventories/results.

## Constraints
Keep Tauri/Rust backend boundaries unless evidence requires change. Prefer Tauri-supported Rust/WASM frontend with generated output clearly classified. Keep dependencies minimal/workspace-inherited. Workflow YAML stays declarative and should invoke Rust binaries instead of embedding owned scripting logic. Never auto-baseline new debt.

## NEXT ACTION
Finish and commit the current generic Python/Shell tooling batch, then start KSSS Python migration as an isolated security-governance batch.
