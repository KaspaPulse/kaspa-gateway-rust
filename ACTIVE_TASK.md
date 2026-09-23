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
PHASE 7 — direct-CI static Node/CJS gate migration is VERIFIED_LOCAL / COMMITTED. The only remaining direct-CI Node gate is dynamic effective-bridge behavior, intentionally deferred to the frontend Rust/WASM phase. Continue with non-direct-CI owned tooling migration.

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
- Current language guard after the parallel-self-worker Rust migration: Rust source 94; owned non-Rust source debt 89; execution debt 14; unapproved 0/0; technical exception 1; PASS.
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
- `kgw_effective_bridge_settings_gate.cjs` remains intentionally deferred because it executes live frontend JavaScript via Node `vm`; replacing it now without a JS engine would weaken coverage, while adding an engine only for transitional tooling would increase supply-chain surface.
- Previous Desktop runtime/native/E2E/release evidence remains reusable because application runtime/product source is untouched.

## Current Blocker
No local engineering blocker. Local `actionlint`, `cargo-audit`, `cargo-deny`, and `cargo-machete` are unavailable, so workflow/supply-chain qualification for the changed dependency/workflow surface remains NOT VERIFIED until exact-head GitHub CI.

## Last Completed Action
Committed the verified parallel-self-worker Rust gate as `dd1dbe88562bc4a22f173b53c7a6fd7f35014376`, tree `4b58ca2f27865580001bb7a9852001772b8ac614`; source debt is 89 and the worktree was clean immediately after commit.

## Current Action
Reconcile continuity to the committed parallel-self-worker boundary, then port the i18n locale-coverage + i18n contract static gate family to Rust. Their current CJS failures/successes must be captured explicitly before retirement.

## Next Action
Port the i18n static gate family with parity-first validation, preserving all reference extraction, dictionary flattening, locale coverage, unbound-text/dynamic-literal/quote-risk, and runtime-marker checks. Keep dynamic effective-bridge coverage until frontend Rust/WASM replacement.

## Verification Required
- `cargo run --locked -p xtask -- language-policy check` = PASS with Rust 94 / source debt 89 / execution debt 14 / exception 1 / zero unapproved.
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
