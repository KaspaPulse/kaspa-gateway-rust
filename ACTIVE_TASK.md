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
PHASE 10 — frontend Rust/WASM migration is IN PROGRESS after substantial tooling migration. Seven template modules are deterministic Rust-generated wrappers, twelve dead frontend JavaScript scaffolds are retired, and Explorer utilities/date/formatting now run from the shared Rust/WASM crate with deterministic generated ABI/glue. Behavioral frontend/E2E JavaScript outside proven generated seams remains migration debt and is not claimed Rust.

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
- Current language guard after complete frontend status Rust/WASM migration: Rust source 112; owned non-Rust source debt 46; execution debt 13; unapproved 0/0; technical exceptions 16; PASS.
- Network-generation Node/CJS family is committed as `5494f580c9426155c5a848289595175f02d3d7d7`.
- Runtime-automation-claims gate is ported to Rust `xtask`; legacy gate PASS before deletion, Rust gate PASS, four regressions PASS on stable and MSRV, strict Clippy/FMT/check PASS, and focused Graphify post-change refresh/query PASS.
- Effective-node-settings gate is ported to Rust `xtask` and committed as `8798af0557384c83cbb8c1b075678a7a01266647`; legacy gate PASS before deletion, Rust gate PASS, five regressions PASS on stable and MSRV, strict Clippy/FMT/check PASS, language guard PASS, and focused Graphify PASS.
- Desktop-version contract gate is ported to Rust `xtask` and committed as `52dcccbc65cff9c23bd6fadf1f9c03de5484ab23`; legacy/Rust outputs match at version 0.1.3/locales 12, seven regressions PASS on stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Desktop-release-draft workflow contract gate is ported to Rust `xtask` and committed as `0031541d5fe833ce7cb8fdd8b265fe5b95657ae7`; legacy gate PASS, Rust real gate PASS, eight regressions PASS on stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Desktop-artifacts workflow contract gate is ported to Rust `xtask` and committed as `efc5885d9d1959271a91c49d1d27ef776452a180`; legacy/Rust real gates PASS, seven regressions PASS stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- npm dependency policy gate/tests are ported to Rust `xtask` and committed as `478ff1642d6016bc65aca53c3fbf20c132b21164`; legacy regression suite PASS, live desktop/E2E policy reference PASS, Rust real gates PASS, eight regressions PASS stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Runtime-repository-binding canonical gate/tests and legacy audit wrappers are ported/retired in commit `a296932742847b232844603ab5d38e1417fae9f1`; the mutating apply path is now Rust and committed as `59e8748a63b29fa44f003ffaf423745e11e4fd5a` after exact drifted-fixture legacy/Rust byte parity and Rust idempotence. Apply regressions 3/3, gate regressions 11/11, strict-offline real gate, stable/MSRV/KSSS-feature checks, Clippy/FMT/language guard, and focused Graphify all PASS.
- E2E exact-owned-process kill/wait helpers are ported from PowerShell to Rust and committed as `ec46991d6e80478038dcb439952cd058394189f8`; legacy/Rust kill parity and wait parity PASS on dedicated synthetic processes, deliberate executable mismatch fails closed without killing, recovery-harness smoke PASS after JS caller migration, language guard PASS, and focused Graphify PASS at 6368 nodes / 16599 edges.
- Project-continuity gate/tests are ported to Rust and committed as `d23d656838397d36c4b0ebc18d96631b8210155a`; the real Rust gate PASS, 9/9 positive/fail-closed regressions PASS on stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Six standalone static contract regressions (analysis, Explorer lint, functional UI, Settings workflow, AUD-010 Tauri seam, and programmatic restore) are consolidated in Rust `xtask` and committed as `cce6059c6efb9f0bc37e22ad4303c6edd7179895`; legacy six PASS before retirement, Rust aggregate PASS, 12/12 regressions PASS stable/MSRV, Clippy/FMT/check PASS, and focused Graphify PASS.
- Parallel-self-worker runtime contract gate is ported to Rust and committed as `dd1dbe88562bc4a22f173b53c7a6fd7f35014376`; the legacy CRLF-sensitive extractor was corrected before retirement, both legacy/Rust real gates PASS, 5/5 regressions PASS stable/MSRV, and focused Graphify PASS.
- i18n locale/full-contract gates are ported to Rust and committed as `9c53fa70a9cb6397e92efe68e2607da9033ea6e5`; locale remains PASS at 32/0/0/1. After template HTML became first-class declarative source, full-contract current truth is refs=266/missing=0/unbound=9/dynamic=6/quote=0/runtime=0: seven additional unbound findings are latent template text now statically visible, not runtime drift.
- Raw-log provenance static gate is ported to Rust and committed as `9c084dc63fca128ae5b2e621dde1204e17d795e8`; CRLF-only legacy false negative was corrected before parity, legacy/Rust both preserve exactly two true frontend filter findings, 4/4 Rust regressions PASS stable/MSRV, Clippy/FMT/MSRV check/language guard/runtime-owner checks PASS. `kgw_log_ui_tests.cjs` remains because it executes live frontend JS behavior.
- Program-unified orchestration/reporting is ported to Rust and committed as `a353ed52cf06f9383265a47e23d309e012181a11`; pre-retirement legacy/Rust semantic parity matched 14 steps and three required failures, post-retirement Rust has 13 steps with the same three required failures, 4/4 stable+MSRV regressions PASS, Clippy/FMT/MSRV check PASS, language guard PASS, and focused Graphify PASS at 6216 nodes / 16117 edges.
- Runtime-trace-owner audit is ported to Rust and committed as `c22503d6e96d08f58e1e4fe5795819e876c72814`; legacy/Rust report parity PASS, post-switch Rust audit PASS, runtime-trace regressions 4/4 and program-unified regressions 5/5 PASS on stable/MSRV, Clippy/FMT/MSRV check/language guard PASS, and focused Graphify PASS at 6241 nodes / 16219 edges.
- Windows PE runtime-dependency verifier is ported from PowerShell to Rust and committed as `9d1885dc5f66be60a36585ca189f15e8ae1417ef`; legacy/Rust same-PE SHA/import/runtime/passed parity PASS, 4/4 verifier and 7/7 desktop-artifacts regressions PASS stable/MSRV, workflow fail-closed contract PASS, Clippy/FMT/MSRV/KSSS-feature checks PASS, and focused Graphify PASS at 6265 nodes / 16288 edges.
- AI workflow gate is ported from PowerShell to Rust and committed as `7b1d869bd0e79003f276ef8fec960dc5cea0750c`; legacy/Rust exact current-failure parity is preserved (missing `.codex/hooks.json` and `graphify-out/graph.json`), 5/5 regressions PASS stable/MSRV, stable/MSRV/KSSS-feature checks and Clippy/FMT PASS, active docs invoke Rust, and focused Graphify PASS at 6279 nodes / 16324 edges.
- Start-button orchestration is ported from PowerShell to Rust and committed as `2e4a3e1dd4ce94a18486f49a30f170d7b08e239c`; legacy/Rust real-gate parity preserves exactly three current failures (duplicate Start ID, duplicate Stop ID, behavioral CJS SyntaxError), Tauri IPC remains 56/56 PASS, 6/6 Rust regressions PASS stable/MSRV, stable/MSRV/KSSS-feature checks, Clippy/FMT/language guard PASS, and focused Graphify PASS at 6290 nodes / 16349 edges.
- Copy Log orchestration is ported from PowerShell to Rust and committed as `729c99aec7c4cdf8fee6727a77ed385f54c29e4c`; legacy/Rust real-gate parity preserves exactly one current failure (behavioral CJS SyntaxError), Tauri clipboard tests remain 4/4 PASS, 6/6 Rust regressions PASS stable/MSRV, stable/MSRV/KSSS-feature checks, Clippy/FMT/language guard PASS, and focused Graphify PASS at 6304 nodes / 16390 edges.
- `kgw_effective_bridge_settings_gate.cjs` remains intentionally deferred because it executes live frontend JavaScript via Node `vm`; replacing it now without a JS engine would weaken coverage, while adding an engine only for transitional tooling would increase supply-chain surface.
- Frontend template source/codegen migration is committed as `eae06aa2fc6aa2c857a7381f9340d974530bbc2e`; seven `.template.html` sources have exact original export SHA/byte parity, seven `.template.js` files are Rust-generated and CI-checked, and focused Graphify PASS at 6415 nodes / 16723 edges.
- Twelve unreachable frontend JS scaffold/compatibility modules were removed rather than rewritten and committed as `65eb328b5e86b6d74a174685d6842f1948c33afa`; repo-wide path/symbol audit found no active references, shell/registry syntax and language guard PASS, and Graphify confirms all twelve nodes absent.
- First active frontend Rust/WASM seam is committed as `c480079f35d1cf41932b1f737209e32090d048d5`: `explorer.utils.js` is deterministic generated ABI glue only; implementation is owned by `kaspa-gateway-frontend-wasm`. Legacy/Rust/generated-adapter behavior matrices are byte-identical SHA-256, native tests 3/3 PASS, stable/MSRV wasm32 checks PASS, codegen+CI+language guard+Clippy/FMT/lint/CSP checks PASS, and focused Graphify PASS at 6432 nodes / 16819 edges.
- Explorer date/formatting are committed as `3e73e1b40d550ffec8a1432e4d60697c4c9ad7b1`; legacy/Rust nodejs matrix SHA parity is exact, generated adapters pass full browser matrix parity under production-equivalent CSP at `Asia/Riyadh` / `en-US`, native 3/3 plus stable/MSRV wasm32 checks PASS, codegen/guard/lint/static contracts PASS, and focused Graphify PASS at 6470 nodes / 16933 edges.
- Frontend `status.js` is fully migrated to Rust/WASM and committed as `1105ae86efee4469476159bcdff5c1e5990e73a3`; generated adapter preserves `statusTone`, `applyStatusTone`, and `renderStatusSummary` with exact browser semantic parity (canonical SHA-256 `0724bb52be828798d4dff420f832b99fe1415731eea5d3965a005b52a8efc4af`, 0 structural diffs), Rust tests 5/5 PASS, stable/MSRV wasm32 checks PASS, codegen/lint/language guard PASS, and focused Graphify PASS at 6572 nodes / 17345 edges.
- Frontend tab registry is declarative JSON with deterministic Rust generation committed as `4037bd2b1f5f8d2b2643f5dfdbf1a366b18de4b8`; normalized runtime JS source SHA remains exactly `d880d93421e75deb871cf40bf1b1f462880133ec92e5fa5555994c3049f13680`, codegen 4/4 tests/check PASS, language guard/lint/static contracts/Clippy/FMT PASS, and focused Graphify PASS at 6475 nodes / 16948 edges.
- True raw-log orchestration is ported from PowerShell to Rust and committed as `b9b7282846c349b7c8decec1d16144b750ca5ad3`; the legacy wrapper defect (redirected-pipe deadlock) is preserved diagnostically, Rust real gate completes with only the known frontend CJS failure, raw-log Rust tests/build PASS, affected callsite/provenance qualification PASS, and focused Graphify PASS at 6518 nodes / 17083 edges.
- Zero-touch result-writer tests are ported from PowerShell to Rust and committed as `eeb2a3e0070da4f029abb558c1830f3b320cf2b8`; legacy test PASS, Rust integration runner PASS against the real evidence helper, Rust unit tests 2/2 PASS stable/MSRV, true-raw-log 5/5 PASS, full-local parser/stable/MSRV/KSSS/Clippy/FMT/language guard PASS, and focused Graphify PASS at 6544 nodes / 17190 edges.
- Bridge node-mode routing audit/reporting is ported from CJS to Rust and committed as `d16ae5b0f4e8824e0d1871bdce2269279dd57769`; legacy/Rust findings/verdict/extract-name/node-syntax semantics match exactly, Rust regressions 4/4 and program-unified regressions 6/6 PASS, runtime global-owner scan returns ok=true/errors=0 after CJS retirement, stable/MSRV/KSSS/Clippy/FMT/language guard PASS, and focused Graphify PASS at 6549 nodes / 17276 edges.
- E2E bridge-locator + recovery-harness static smokes are ported from MJS to Rust and committed as `39768dffbc2a912987a915b53f86f4a0c3529386`; legacy smokes PASS, Rust regressions 5/5 stable/MSRV, E2E npm check PASS with unchanged behavioral deepmerge/runtime-port smokes, KSSS/Clippy/FMT/language guard PASS, and focused Graphify PASS at 6541 nodes / 17218 edges.
- Native Rust E2E clipboard capability is checkpointed as `e9f1b48aa892d9c7e186b379d88b75be605469fb`; read-only legacy/Rust metadata parity is exact and 5/5 tests stable/MSRV plus KSSS/Clippy/FMT pass. It is NOT adopted by `windows.mjs` and does NOT retire `kgw_windows_clipboard.ps1` because real write parity is blocked while the user's clipboard contains non-text/OLE/enterprise formats that a text-only restore could destroy.
- Native/runtime/release evidence whose predicates exclude frontend Explorer utility behavior remains reusable. Frontend/app-boot/E2E evidence touching the changed module graph is now invalidated for final closure and must be requalified proportionally before the task can be declared complete.

## Current Blocker
Two independent blockers are explicit: full-local wrapper retirement is BLOCKED because no current zero-touch E2E artifact passes `Test-KgwZeroTouchResultIntegrity`; Rust clipboard caller adoption is BLOCKED because current user clipboard formats include OLE/enterprise non-text data and safe real write parity cannot mutate/restore them losslessly. Independent work continues. Preserved frontend findings remain: i18n 9 unbound HTML + 6 dynamic literals, raw-log provenance Node/Bridge transport-wrapper filters, duplicate Start/Stop IDs, and shared behavioral CJS SyntaxError. Generated template/WASM JS artifacts are exceptions only where Rust deterministically owns generation and CI rejects drift. Local `actionlint`, `cargo-audit`, `cargo-deny`, and `cargo-machete` remain unavailable until exact-head CI.

## Last Completed Action
Committed the verified complete status Rust/WASM migration as `1105ae86efee4469476159bcdff5c1e5990e73a3`, tree `e227446efe27ee5f9c0e2fbcc965a0a09535b47c`; source debt is 46, execution debt is 13, exceptions are 16, and the worktree was clean immediately after commit. The earlier Rust clipboard capability remains checkpointed but not adopted.

## Current Action
Reconcile continuity to the complete status Rust/WASM boundary. Full-local and clipboard adoption remain independently blocked; continue safe implementation migration work.

## Next Action
Continue the next independent implementation migration; ESLint flat-config classification is complete and guarded narrowly. Revisit full-local only after a reusable E2E artifact exists and clipboard adoption only in an isolated/safe clipboard context.

## Verification Required
- `cargo run --locked -p xtask -- language-policy check` = PASS with Rust 112 / source debt 46 / execution debt 13 / exceptions 16 / zero unapproved.
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
- Program-unified Rust runner = verified parity before/after legacy retirement; targeted regressions = 5/5 PASS stable/MSRV after runtime-trace callsite migration; Clippy/FMT/MSRV check = PASS; focused Graphify = PASS.
- Runtime-trace-owner Rust audit = legacy/Rust report parity PASS; post-switch audit PASS; targeted regressions 4/4 PASS stable/MSRV; Clippy/FMT/MSRV check = PASS; focused Graphify = PASS.
- Windows runtime-dependency Rust verifier = same-PE legacy/Rust SHA/import/runtime/passed parity PASS; parser regressions 4/4 and desktop-artifacts regressions 7/7 PASS stable/MSRV; workflow fail-closed contract, Clippy/FMT/MSRV/KSSS-feature checks, language guard, and focused Graphify = PASS.
- AI workflow Rust gate = exact legacy current-failure parity; regressions 5/5 PASS stable/MSRV; stable/MSRV/KSSS-feature checks, Clippy/FMT, language guard, active-doc migration, and focused Graphify = PASS.
- Start-button Rust gate = exact legacy current-failure parity; Tauri IPC = 56/56 PASS; Rust regressions 6/6 PASS stable/MSRV; stable/MSRV/KSSS-feature checks, Clippy/FMT, language guard, and focused Graphify = PASS.
- Copy Log Rust gate = exact legacy current-failure parity; Tauri clipboard tests = 4/4 PASS; Rust regressions 6/6 PASS stable/MSRV; stable/MSRV/KSSS-feature checks, Clippy/FMT, language guard, and focused Graphify = PASS.
- Frontend template Rust codegen = seven exported HTML strings exact baseline SHA/bytes/JS length; codegen check PASS; codegen tests 2/2 and language-policy tests 9/9 PASS stable/MSRV; stable/MSRV/KSSS checks, Clippy/FMT, language guard, CI codegen enforcement, and focused Graphify PASS.
- Frontend Rust/WASM Explorer utilities = exact legacy/Rust/generated-adapter behavior matrix SHA parity; native Rust tests 3/3 PASS; stable/MSRV wasm32 checks PASS; frontend-wasm-codegen check + 2/2 regressions PASS; generated adapter/glue syntax and desktop lint PASS; production/E2E CSP uses only `wasm-unsafe-eval` with same-origin fetch; focused Graphify PASS at 6432 nodes / 16819 edges.
- Explorer date/formatting Rust/WASM = exact legacy/Rust nodejs matrix parity plus full browser generated-adapter matrix parity; native 3/3 and stable/MSRV wasm32 checks PASS; codegen/language guard/syntax/lint/static contracts/diff-check PASS; focused Graphify PASS at 6470 nodes / 16933 edges.
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
