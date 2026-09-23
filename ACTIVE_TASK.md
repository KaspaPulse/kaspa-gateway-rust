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
PHASE 6 — Node/CJS repository-gate migration is IN PROGRESS. The first independent family, network-generation gate + regression tests, is VERIFIED_LOCAL_PENDING_COMMIT.

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
- Current language guard after the network-generation port: Rust source 84; owned non-Rust source debt 109; execution debt 14; unapproved 0/0; technical exception 1; PASS.
- Network-generation Node/CJS family is ported to Rust `xtask`; legacy Node gate/tests passed before deletion, Rust gate PASS, six Rust regressions PASS on stable and MSRV, strict Clippy/FMT/check PASS, and focused Graphify post-change refresh/query PASS.
- Previous Desktop runtime/native/E2E/release evidence remains reusable because application runtime/product source is untouched.

## Current Blocker
No local engineering blocker. Local `actionlint`, `cargo-audit`, `cargo-deny`, and `cargo-machete` are unavailable, so workflow/supply-chain qualification for the changed dependency/workflow surface remains NOT VERIFIED until exact-head GitHub CI.

## Last Completed Action
Verified the first Node/CJS migration family locally: `kgw_network_generation_gate.cjs` and its CJS tests are superseded by Rust `xtask`, with debt reduced 111→109 and all affected local checks green.

## Current Action
Reconcile continuity for the verified network-generation Rust port, review/stage its exact scope, and create one local checkpoint commit.

## Next Action
After the network-generation checkpoint commit, select the next smallest independent Node/CJS gate family and repeat parity-first Rust migration without replaying valid KSSS/network-generation evidence.

## Verification Required
- `cargo run --locked -p xtask -- language-policy check` = PASS with Rust 84 / source debt 109 / execution debt 14 / exception 1 / zero unapproved.
- Network-generation Rust gate = PASS; network-generation regressions = 6/6 PASS on stable and MSRV 1.97.1; focused Graphify update/query = PASS.
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
