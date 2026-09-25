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
6. Node/CJS repository/tooling migration to Rust — **IN PROGRESS / STATIC LANE ADVANCED**; all direct-CI static gate families, six standalone static regressions, parallel-self-worker, i18n static gates `9c53fa7`, raw-log provenance `9c084dc`, program-unified orchestration `a353ed5`, and runtime-trace-owner audit `c22503d` are COMMITTED. Dynamic effective-bridge, log-ui behavior, current i18n findings, and raw transport-wrapper findings are deferred to frontend migration.
7. Windows/PowerShell helper migration to Rust with Windows behavior preserved — **IN PROGRESS**; Windows runtime-dependency verifier `9d1885d`, AI workflow gate `7b1d869`, Start-button orchestration `2e4a3e1`, Copy Log orchestration `729c99a`, runtime-repository-binding apply `59e8748`, E2E exact-owned-process helpers `ec46991`, and zero-touch Windows evidence capture `f4ec63b` are committed after parity/fail-closed preservation.
8. WebdriverIO/Node E2E replacement with Rust-native desktop/WebDriver harness — **PENDING / FOUNDATION ACTIVE**; runtime-port and assertion helper implementation now live in `kaspa-gateway-e2e-wasm`, the standalone runtime-port JavaScript smoke has been retired into Rust static checks, while behavioral specs/WebdriverIO orchestration remain migration debt.
9. JavaScript frontend replacement with Rust/WASM while preserving Tauri IPC/UI contracts — **IN PROGRESS / ACTIVE RUST-WASM SEAM ESTABLISHED**; seven template modules are Rust-generated wrappers over exact-byte declarative HTML at `eae06aa`, twelve dead JS scaffolds were removed at `65eb328`, Explorer utils/date/formatting/status/header-price behavior now has Rust/WASM ownership with deterministic generated ABI/glue.
10. Remove Node/Python/PowerShell/Shell implementation dependencies and update workflows/configuration — **PENDING**.
11. Zero-debt strict guard + cross-platform/security/MSRV qualification — **PENDING**.
12. GitHub PR exact-head CI, squash merge, exact-main qualification, durable closeout — **PENDING**.

## Progress
OP069 adds native result/failure construction and atomic JSON artifact writing. The actual primary E2E result path now delegates to Rust:34 builder cases,20 writer cases,four extracted caller cases,35 stable/MSRV tests,strict Clippy/MSRV feature checks and Unicode roundtrip PASS. Legacy emergency handling and the remaining PowerShell helper remain; no full E2E/native application run occurred. Detailed checkpoint: docs/handoff-ledger/2026-09-25-rust-native-result-writer.md.

OP068 implemented the native Rust saved-evidence validator and adopted it in the full-local artifact-reuse function. Native/PowerShell comparison73 cases PASS (67 exact JSON and6 invalid-input rejection),25 stable/MSRV regressions PASS, strict Clippy/package feature compilation and six extracted caller cases PASS. Graphify preparation and KSSS-result inspection are TOOL_BLOCKED; no full component/final application qualification is inferred. The legacy result-writing library remains active, so owned source debt stays41.

OP066 added an uncommitted Rust E2E artifact-path capability: actual Node/WASM and isolated filesystem parity 557/557 with zero differences; stable/MSRV, strict Clippy, existing codegen, E2E check/lint and focused Graphify PASS. The original paths.mjs remains active because its generator-source atomic update failed. Commit attempt OP066I was TOOL_BLOCKED before execution. No source-debt retirement or full migration success is claimed.

The guard now reports Rust source inventory 130, owned non-Rust source debt 41, execution-wiring debt 13, unapproved debt 0/0, and twenty-one technical exceptions. Explorer `utils/date/formatting`, the original `statusTone`/`applyStatusTone`/`renderStatusSummary` behavior, and Explorer header USD price parsing retain their historical exact-browser-parity receipts. Their current shared module graph still requires final requalification. The new settings runtime presentation/observation exports have 582-case Node/WASM parity and 49 unchanged settings tests PASS, but browser verification is NOT VERIFIED / TOOL_BLOCKED. E2E `runtime-ports.mjs` and `assertions.mjs` are deterministic generated Node ABI adapters over `kaspa-gateway-e2e-wasm`; assertions preserve an exact legacy/external/tracked matrix SHA-256 `40287354a84163529e9d32c21d68b5aa232eb824f190323f6ddac71c50facc27` across 17 cases, and the former `runtime-ports-smoke.mjs` contracts now live in Rust `e2e_static_smokes` with 6/6 regressions PASS. True raw-log orchestration, zero-touch result-writer tests, zero-touch Windows process/TCP evidence capture, bridge-locator/recovery-harness static E2E smokes, and Bridge node-mode routing audit/reporting live in Rust. A Windows-native Rust clipboard helper candidate exists with exact read-only metadata parity, but caller adoption/debt retirement is blocked until write parity can run in a safe isolated clipboard context; the existing PowerShell helper remains authoritative. Full-local wrapper retirement is also deferred until a current reusable E2E artifact exists. i18n locale remains PASS; full i18n current truth is 9 unbound HTML + 6 dynamic literals. Frontend/app-boot/E2E predicates touching the changed module graph require final requalification. Workflow lint and cargo audit/deny/machete remain exact-head CI qualification work.

## Completion Criteria
The plan closes only when strict language policy proves zero owned non-Rust implementation debt, all affected/final checks pass on supported platforms, KSSS/supply-chain controls remain intact, protected checkpoint is unchanged, final PR is squash-merged under repository rules, exact-main CI passes, and durable closeout records final SHA/tree/inventories/results.

## Constraints
Keep Tauri/Rust backend boundaries unless evidence requires change. Prefer Tauri-supported Rust/WASM frontend with generated output clearly classified. Keep dependencies minimal/workspace-inherited. Workflow YAML stays declarative and should invoke Rust binaries instead of embedding owned scripting logic. Never auto-baseline new debt.

## Continuity Recovery
- Canonical state recovery: **PARTIAL / ATOMIC_REPLACE_BLOCKED**. OP063 recovered and compared all five proposals and backups. Only CURRENT_STATE.md and PLANS.md are selected for this update; PROJECT_STATE.md, ACTIVE_TASK.md and the task handoff remain older than source5403092. The saved PROJECT_STATE.md.op062.tmp is preserved. A guarded same-API replacement failed again with WinError5. Restart Manager identifies a Desktop Commander reader, but DELETE-only access probes pass, so the historical root cause is not established. No process, permission, ownership or security setting was changed.

## NEXT ACTION
Recover actual Git and the latest OP069 journal/checkpoint; preserve prior OP066 dirty work. Complete the scoped component durability checkpoint, then consolidate remaining evidence-summary callers and retire superseded helper functions only after equivalent qualification and safe source writes. Graphify/KSSS inspection and earlier canonical/generator/browser/clipboard blockers remain; do not reroute denied operations. Full migration is PARTIAL and publication NOT_QUALIFIED.
