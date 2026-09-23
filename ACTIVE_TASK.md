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
PHASE 2 — burn down bounded non-Rust tooling debt after the committed Rust language-policy foundation.

## Confirmed Progress
- GitHub baseline was reconciled to `aaf2c635672c0fd35a5705579610be8de188b031` / tree `0d19e16d115dc093a3f47967ec57b0cc3e81bfa1`.
- Phase 1 foundation is committed locally as `33461f6511c69b457c5f3dd069b54322d9a236a0` / tree `e355550c6fa311fdfb4dd54a8cd29c4b45f0c583`.
- The strengthened guard now classifies inline GitHub Actions shell logic; one omitted pre-existing workflow debt path was reconciled only after proving exact baseline blob identity.
- Five owned Python CI/security policy scripts were ported to Rust `xtask`, parity-tested, and removed.
- ClusterFuzzLite build logic moved to Rust `xtask`; required `.clusterfuzzlite/build.sh` is now a six-line platform adapter only.
- Current guard: Rust source inventory 81; owned non-Rust source debt 116; execution-wiring debt 14; unapproved source 0; unapproved execution 0; technical exceptions 1; migration guard PASS.
- Rust replacement tests: 23/23 PASS on stable and 23/23 PASS on MSRV 1.97.1.
- MSRV 1.97.1 `cargo check --locked -p xtask` PASS; stable Clippy `-D warnings` PASS; workspace FMT PASS.
- Advisory policy Rust command PASS and matches the prior Python result. ClusterFuzz adapter `bash -n` PASS.
- Previous Desktop runtime/native/E2E/release evidence remains reusable because this batch changes repository tooling/workflows only, not application runtime/product source.

## Current Blocker
No local engineering blocker for this batch. Local `actionlint` is unavailable, so changed workflow syntax/lint and end-to-end TruffleHog integration remain NOT VERIFIED until GitHub CI or equivalent validated tooling runs.

## Last Completed Action
Strengthened the language guard for workflow shell blocks, reconciled the single proven baseline omission, and qualified the Rust tooling replacement on stable/MSRV/Clippy/FMT.

## Current Action
Reconcile continuity to the verified batch state, run continuity/diff/affected validation, then checkpoint the batch in a local commit.

## Next Action
After the batch commit, inspect and migrate the KSSS Python consumer/gate surface as a separate security-sensitive batch without weakening signed-runtime/trust/rollback semantics.

## Verification Required
- `cargo run --locked -p xtask -- language-policy check` = PASS with 116 source debt / 14 execution debt / 1 exception / zero unapproved.
- `cargo test --locked -p xtask` = 23/23 PASS.
- Rust 1.97.1 check/test for `xtask` = PASS.
- Stable `cargo clippy --locked -p xtask --all-targets -- -D warnings` = PASS.
- `cargo fmt --all -- --check` = PASS.
- `bash -n .clusterfuzzlite/build.sh` = PASS.
- Project continuity gate and regression tests = PASS after this state update.
- `git diff --check` = PASS.
- Changed workflow lint and integrated secret-scan/security jobs remain required in exact-head CI.

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
