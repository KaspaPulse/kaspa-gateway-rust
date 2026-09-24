# ACTIVE TASK

## Status
IN PROGRESS — OWNED PROGRAMMING IMPLEMENTATION 100% RUST MIGRATION

## Objective
Make Rust the only owned programming implementation language in Kaspa Gateway without manipulating GitHub Linguist statistics. Preserve required declarative/platform artifacts, Windows/macOS/Linux support, KSSS governance, supply-chain controls, and production isolation.

## Scope
- Migrate project-owned Python, JavaScript/TypeScript, Shell, PowerShell, and other programming implementation to Rust.
- Keep the existing Rust/Tauri backend and IPC boundary unless evidence proves a change is required.
- Replace the owned JavaScript frontend with Rust/WASM while preserving user-visible contracts and platform support.
- Replace Node/WebdriverIO E2E, KSSS adapters/gates, CI/release/security gates, and OS helpers with Rust-native equivalents.
- Keep YAML/JSON/TOML/Markdown/licenses/manifests/generated/vendor artifacts only when technically required and explicitly classified.
- Maintain a fail-closed CI language guard. New non-Rust implementation must never be auto-baselined.
- No Production, DNS, Cloudflare, live runtime, production credentials, or protected-checkpoint mutation.

## Current Phase
PHASE 8 — non-direct-CI owned tooling migration is IN PROGRESS. i18n and raw-log provenance static gates are Rust and COMMITTED; dynamic effective-bridge, log-ui behavioral tests, and current frontend findings are deferred to the frontend Rust/WASM phase.

## Confirmed Progress
- GitHub baseline was reconciled to `aaf2c635672c0fd35a5705579610be8de188b031` / tree `0d19e16d115dc093a3f47967ec57b0cc3e81bfa1`.
- Phase 1 foundation is committed locally as `33461f6511c69b457c5f3dd069b54322d9a236a0` / tree `e355550c6fa311fdfb4dd54a8cd29c4b45f0c583`.
- The strengthened guard now classifies inline GitHub Actions shell logic; one omitted pre-existing workflow debt path was reconciled only after proving exact baseline blob identity.
- Five owned Python CI/security policy scripts were ported to Rust `xtask`, parity-tested, and removed.
- ClusterFuzzLite build logic moved to Rust `xtask`; required `.clusterfuzzlite/build.sh` is now a six-line platform adapter only.
- Generic tooling guard baseline before KSSS was Rust 81 / source debt 116 / execution debt 14 / exception 1.
- The KGW-owned KSSS adapter is now Rust under `xtask`; five superseded KSSS Python adapter/gate files are removed only after parity.
- Signed central KSSS Python runtime remains unchanged inside the verified archive; its SHA-256 is still `38309d2ab8fa30096d99940f855e88173faa182e60db33f2a96b2d3408507430`.
- Rust KSSS rejection contracts are 24/24 PASS; the complete xtask suite is 47/47 PASS on stable and 47/47 PASS on MSRV 1.97.1.
- Old Python and new Rust adapters produced identical semantic JSON for check/evaluate/knowledge/release-check/structural trust, and cryptographic trust parity PASS with verified Cosign v3.0.6.
- Stable check/Clippy `-D warnings`/FMT PASS; MSRV KSSS check/test PASS.
- Current language guard after the raw-log provenance migration: Rust source 96; owned non-Rust source debt 86; execution debt 14; unapproved 0/0; technical exception 1; PASS.
- Network-generation Node/CJS family is committed as `5494f580c9426155c5a848289595175f02d3d7d7`.
- Runtime-automation-claims gate is ported to Rust `xtask`; legacy gate PASS before deletion, Rust gate PASS, four regressions PASS on stable and MSRV, strict Clippy/FMT/check PASS, and focused Graphify post-change refresh/query PASS.
- Effective-node-settings gate is ported to Rust `xtask` and committed as `8798af0557384c83cbb8c1b075678a7a01266647`; legacy gate PASS before deletion, Rust gate PASS, five regressions PASS on stable and MSRV, strict Clippy/FMT/check PASS, language guard PASS, and focused Graphify PASS.
- Desktop-version contract gate is ported to Rust `xtask` and committed as `52dcccbc65cff9c23bd6fadf1f9c03de5484ab23`; legacy/Rust outputs match at version 0.1.3/locales 12, seven regressions PASS on stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Desktop-release-draft workflow contract gate is ported to Rust `xtask` and committed as `0031541d5fe833ce7cb8fdd8b265fe5b95657ae7`; legacy gate PASS, Rust real gate PASS, eight regressions PASS on stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Desktop-artifacts workflow contract gate is ported to Rust `xtask` and committed as `efc5885d9d1959271a91c49d1d27ef776452a180`; legacy/Rust real gates PASS, seven regressions PASS stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- npm dependency policy gate/tests are ported to Rust `xtask` and committed as `478ff1642d6016bc65aca53c3fbf20c132b21164`; legacy regression suite PASS, live desktop/E2E policy reference PASS, Rust real gates PASS, eight regressions PASS stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Runtime-repository-binding canonical gate/tests and legacy audit wrappers are ported/retired in commit `a296932742847b232844603ab5d38e1417fae9f1`; legacy/Rust offline+online PASS, 11 regressions PASS stable/MSRV, Clippy/FMT/check PASS, focused Graphify PASS, and the mutating `apply.ps1` remains unchanged for the later PowerShell phase.
- Project-continuity gate/tests are ported to Rust and committed as `d23d656838397d36c4b0ebc18d96631b8210155a`; the real Rust gate PASS, 9/9 positive/fail-closed regressions PASS on stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Six standalone static contract regressions (analysis, Explorer lint, functional UI, Settings workflow, AUD-010 Tauri seam, and programmatic restore) are consolidated in Rust `xtask` and committed as `cce6059c6efb9f0bc37e22ad4303c6edd7179895`; legacy six PASS before retirement, Rust aggregate PASS, 12/12 regressions PASS stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Parallel-self-worker runtime contract gate is ported to Rust and committed as `dd1dbe88562bc4a22f173b53c7a6fd7f35014376`; the legacy CRLF-sensitive extractor was corrected before retirement, both legacy/Rust real gates PASS, 5/5 regressions PASS stable/MSRV, and focused Graphify PASS.
- i18n locale/full-contract gates are ported to Rust and committed as `9c53fa70a9cb6397e92efe68e2607da9033ea6e5`; locale parity PASS at 32/0/0/1, full-contract parity preserves the current FAIL truth at refs=266/missing=0/unbound=2/dynamic=6/quote=0/runtime=0, eight Rust regressions PASS stable/MSRV, Clippy/MSRV check/language guard PASS. Focused Graphify for this batch is NOT VERIFIED / TOOL_BLOCKED only because safe Server→kas file transfer is unavailable.
- Raw-log provenance static gate is ported to Rust and committed as `9c084dc63fca128ae5b2e621dde1204e17d795e8`; CRLF-only legacy false negative was corrected before parity, legacy/Rust both preserve exactly two true frontend filter findings, 4/4 Rust regressions PASS stable/MSRV, Clippy/FMT/MSRV check/language guard/runtime-owner checks PASS. `kgw_log_ui_tests.cjs` remains because it executes live frontend JS behavior.
- `kgw_effective_bridge_settings_gate.cjs` remains intentionally deferred because it executes live frontend JavaScript via Node `vm`; replacing it now without a JS engine would weaken coverage, while adding an engine only for transitional tooling would increase supply-chain surface.
- Previous Desktop runtime/native/E2E/release evidence remains reusable because application runtime/product source is untouched.

## Current Blocker
No local engineering blocker for the static tooling lane. Preserved product findings for frontend migration: i18n has 2 unbound HTML + 6 dynamic literals; raw-log provenance has Node/Bridge transport-wrapper filters. `kgw_log_ui_tests.cjs` and dynamic effective-bridge remain behavioral JS coverage. Local `actionlint`, `cargo-audit`, `cargo-deny`, and `cargo-machete` remain unavailable until exact-head CI.

## Last Completed Action
Committed the raw-log provenance Rust gate as `9c084dc63fca128ae5b2e621dde1204e17d795e8`, tree `cb16827e96aa8456ecd17250bc4593600681a55a`; source debt is 86 and the worktree was clean immediately after commit.

## Current Action
Reconcile continuity to the committed raw-log provenance boundary, then inventory remaining non-direct-CI Node/MJS tools and select the smallest static family that can move to Rust without executing frontend JS.

## Next Action
Port the next smallest static non-direct-CI tooling family with parity-first validation. Keep behavioral JS gates/tests for the frontend Rust/WASM phase rather than replacing them with weaker static checks.

## Verification Required
- `cargo run --locked -p xtask -- language-policy check` = PASS with Rust 96 / source debt 86 / execution debt 14 / exception 1 / zero unapproved.
- Network-generation evidence remains reusable from commit `5494f58...`.
- Runtime-automation Rust gate = PASS; regressions = 4/4 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Effective-node-settings Rust gate = PASS; regressions = 5/5 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Desktop-version Rust gate = PASS; regressions = 7/7 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Desktop-release-draft Rust gate = PASS; regressions = 8/8 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Desktop-artifacts Rust gate = PASS; regressions = 7/7 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- npm dependency policy Rust gate = PASS for desktop/E2E live audit; regressions = 8/8 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Runtime-repository-binding Rust gate = PASS offline/online; regressions = 11/11 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Project-continuity Rust gate = PASS; regressions = 9/9 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Static-contract Rust aggregate = PASS; regressions = 12/12 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- Parallel-self-worker Rust gate = PASS; regressions = 5/5 PASS on stable and MSRV 1.97.1; Clippy/FMT/check = PASS; focused Graphify update/query = PASS.
- i18n locale Rust gate = PASS at 32 critical keys / 0 missing / 0 same-as-English / 1 approved; full i18n Rust gate matches legacy expected FAIL at 266/0/2/6/0/0 with identical findings; Rust regressions = 8/8 PASS stable/MSRV; Clippy/MSRV check PASS; focused Graphify NOT VERIFIED / TOOL_BLOCKED for this batch.
- Raw-log provenance Rust gate matches legacy corrected expected FAIL with exactly two transport-wrapper findings; regressions = 4/4 PASS stable/MSRV; Clippy/FMT/MSRV check PASS; runtime-owner strict PASS; focused Graphify NOT VERIFIED / TOOL_BLOCKED for this batch.
- `cargo test --locked -p xtask --features ksss` = 47/47 PASS on stable.
- Rust 1.97.1 `cargo check/test --locked -p xtask --features ksss` = PASS / 47/47.
- Stable `cargo clippy --locked -p xtask --all-targets --features ksss -- -D warnings` = PASS.
- `cargo fmt --all -- --check` = PASS.
- KSSS old/new semantic parity = PASS for check/evaluate/knowledge/release-check/trust, including cryptographic Sigstore verification.
- Signed KSSS runtime/trust evidence bytes = unchanged.
- Project continuity gate and regression tests must PASS after this state update.
- `git diff --check` must PASS.
- Workflow lint, cargo-audit, cargo-deny, cargo-machete, and integrated GitHub security jobs remain required in exact-head CI.

## Completion Criteria
- `OWNED_PROGRAMMING_IMPLEMENTATION=100_PERCENT_RUST`.
- Owned non-Rust programming source debt = 0.
- Non-Rust execution wiring debt = 0 except narrowly proven platform-required adapters.
- Strict Rust language policy guard = PASS.
- FMT, build/check, tests, Clippy, MSRV, dependency/security checks, secret scan, workflow lint, and required CI = PASS.
- Windows, macOS, and Linux support preserved.
- KSSS governance preserved.
- Production mutation = NO.
- Final branch/PR follows exact-head CI → squash merge → exact-main qualification → durable closeout.

## DO NOT REPEAT
Do not rerun Desktop 0.1.3 runtime/native/E2E/release qualification without predicate invalidation. Do not touch older worktrees or the protected historical checkpoint. Do not restore deleted Python helpers, weaken exact fingerprint policies, or hide debt with Linguist/automatic baselining.
