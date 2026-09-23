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
4. Generic Python/Shell security and CI helper migration — **VERIFIED_LOCAL / COMMITTED** at `d5f274dcc6d9a423dd9783d21605591efdd05e65`; five generic Python scripts removed after Rust parity and ClusterFuzz build logic moved to Rust.
5. KSSS Python consumer/gate migration to Rust — **VERIFIED_LOCAL / COMMITTED** at `55727c4eb53d34a2cd91c8e857850d543ec177e4`; five owned Python files retired after 24-contract and command/crypto parity, with signed central runtime bytes unchanged.
6. Node/CJS repository gate migration to Rust — **IN PROGRESS**; network-generation is COMMITTED at `5494f580c9426155c5a848289595175f02d3d7d7`, runtime-automation-claims is COMMITTED at `02c75b87060da96efe8b843deefccdf8363ff459`, effective-node-settings is COMMITTED at `8798af0557384c83cbb8c1b075678a7a01266647`, and effective-bridge-settings is next.
7. PowerShell helper migration to Rust with Windows behavior preserved — **PENDING**.
8. WebdriverIO/Node E2E replacement with Rust-native desktop/WebDriver harness — **PENDING**.
9. JavaScript frontend replacement with Rust/WASM while preserving Tauri IPC/UI contracts — **PENDING**.
10. Remove Node/Python/PowerShell/Shell implementation dependencies and update workflows/configuration — **PENDING**.
11. Zero-debt strict guard + cross-platform/security/MSRV qualification — **PENDING**.
12. GitHub PR exact-head CI, squash merge, exact-main qualification, durable closeout — **PENDING**.

## Progress
The guard now reports Rust source inventory 86, owned non-Rust source debt 107, execution-wiring debt 14, unapproved debt 0/0, and one platform-required thin adapter exception. KSSS/network-generation/runtime-automation evidence remains reusable. Effective-node-settings is ported to Rust with legacy parity, 5/5 stable+MSRV regressions, Clippy/FMT/check PASS, language guard PASS, and focused Graphify PASS. Workflow lint and cargo audit/deny/machete remain exact-head CI qualification work.

## Completion Criteria
The plan closes only when strict language policy proves zero owned non-Rust implementation debt, all affected/final checks pass on supported platforms, KSSS/supply-chain controls remain intact, protected checkpoint is unchanged, final PR is squash-merged under repository rules, exact-main CI passes, and durable closeout records final SHA/tree/inventories/results.

## Constraints
Keep Tauri/Rust backend boundaries unless evidence requires change. Prefer Tauri-supported Rust/WASM frontend with generated output clearly classified. Keep dependencies minimal/workspace-inherited. Workflow YAML stays declarative and should invoke Rust binaries instead of embedding owned scripting logic. Never auto-baseline new debt.

## NEXT ACTION
Port effective-bridge-settings gate to Rust with parity-first validation, then continue to the next independent Node/CJS gate family. Reuse KSSS/network-generation/runtime-automation/effective-node evidence while their predicates remain unchanged.
