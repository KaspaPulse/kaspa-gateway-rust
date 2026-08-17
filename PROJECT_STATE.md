# PROJECT STATE

## Metadata

- Last state update: 2026-08-17 16:47 +03:00.
- State author/agent: maintenance-state reconciliation session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This document is the canonical resumable summary after reconciliation; Git/GitHub, CI, release metadata, and live runtime evidence remain the owning sources for their facts.

## Executive Status

- Overall status: **DESKTOP 0.1.1 RELEASE/WORKFLOW ENGINEERING CLOSED; ORDINARY MAINTENANCE ACTIVE; OWNER CREDENTIAL RETIREMENT PENDING**.
- Current objective: complete ordinary dependency maintenance without reopening the completed Desktop `0.1.1` release plan or changing runtime/release invariants.
- Current engineering blocker: **NONE** for the completed Desktop `0.1.1` release/workflow/continuity work.
- Owner-only security hygiene remains: remove repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the associated short-lived fine-grained PAT. Its value must never be recorded in chat, repository files, logs, evidence bundles, or documentation.

## Repository State

- Default integration branch: `main`.
- Current branch: **VERIFY DYNAMICALLY** with `git branch --show-current` in the actual checkout.
- Current HEAD: **VERIFY DYNAMICALLY** with `git rev-parse HEAD`.
- Current remote main: **VERIFY DYNAMICALLY** from Git/GitHub before any decision that depends on it.
- Working tree: **NOT VERIFIED** in the dedicated server checkout from this agent environment.
- Dedicated server checkout probe during this reconciliation: `/home/kas/kaspa-gateway-dev/codex/kaspa-gateway-rust` was not mounted/present in this execution environment, so local branch/HEAD/status/remotes were not inferred.
- Remote tracking state: **NOT VERIFIED** in a local checkout; inspect `git status --short --branch`, `git log -1 --oneline`, and remotes when resuming in the dedicated checkout.
- Verified code baseline (historical continuity evidence): `363a3f7f959a4ead47a6297c4866218d08b9820c` (`docs: harden project continuity handoff (#61)`).
- Verified application/workflow implementation baseline: `197bdead257973164931b34cce20c4556820df44` (`fix: verify desktop release drafts by release ID (#59)`).
- State-document commit: derive dynamically from Git when needed; do not copy a self-referential state SHA here.
- Qualified Desktop `0.1.1` source and published tag target: `b911eb44619f8eab706bc2fe786d1c84ac958f1d` (`release: prepare desktop 0.1.1 (#50)`).
- Reconciliation observation at 2026-08-17 16:47 +03:00: GitHub `main` resolved to `f3bd629126b8572bda32dd1f8da253d3491430b7` after PR #52; this is a timestamped observation, not a forever-current claim.
- Active ruleset snapshot re-verified during this reconciliation: ID `20627285`, `main-rebootstrap-baseline`, enforcement active, squash-only integration, unresolved review threads required, no bypass, and six required PR contexts.

## Uncommitted Work

- Dedicated server checkout/worktree: **NOT VERIFIED** because it is not mounted in this agent environment. Do not discard or hide local work without inspecting it first.
- Temporary remote operations branch `ops/release-011-recovery-gate-20260817` remains evidence-only and must not be merged as application history.
- Repository secret/PAT retirement is an external GitHub-settings action, not uncommitted repository code.

## Desired State

- Preserve official Kaspa runtime ownership and official runtime bindings; do not reimplement official runtime behavior.
- Preserve the zero-fake-log invariant: raw runtime panes show real official stdout/stderr only.
- Keep `mainnet` and `testnet10` stable/supported; keep `testnet12` experimental and explicit opt-in.
- Keep immutable Desktop `0.1.1` bound to source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` with its verified six-asset release set unchanged.
- Preserve repaired draft-release semantics: draft-inclusive discovery, unique draft resolution by tag/target, numeric release-ID propagation, and ID-based post-create verification.
- Keep `AGENTS.md` stable policy, `PROJECT_STATE.md` current state, ADRs durable decisions, runbooks repeatable operations, and `PLANS.md` active only for genuine multi-stage work.
- Exact-head qualify every selected maintenance PR and preserve linear squash-only integration.

## Actual State

### Repository

- Desktop package/release line remains `0.1.1` for the completed release described below.
- Rust workspace policy is Rust `1.97.1`, edition `2024`.
- Desktop Node engine is `>=24 <27`; blocking CI uses Node `24.19.0` and npm `11.17.0`.
- `AGENTS.override.md` is absent on the verified GitHub `main` state; `AGENTS.md` is the active repository agent policy.
- `PLANS.md` remains intentionally in **NO ACTIVE MULTI-STAGE PLAN** state; no new multi-stage plan is required for the current one-at-a-time maintenance sequence.
- Dependabot maintenance transitions completed in this session before this reconciliation:
  - PR #53 (`globals` desktop) squash-merged; post-merge `main` baseline was `21ab0782fa741d0ddca6f23d1b60099b54be7cbd` and push CI passed.
  - PR #54 (`globals` E2E) squash-merged; post-merge `main` baseline was `1499dad88214190f9f6cf4728387e551286c541f` and push CI passed.
  - PR #55 (GitHub Actions group) squash-merged; post-merge `main` baseline was `bc56b8c53aff889421e36ad91154c61da87808a7` and push CI passed.
  - PR #52 (`sysinfo` 0.31.4 -> 0.39.6) initially failed `cargo check` because `System::refresh_processes` gained the `remove_dead_processes` argument. The same PR was repaired with the minimal compatibility adaptation `refresh_processes(..., true)`, requalified on exact final head `ef514ba28bd9cdacbd2ca5c2cfe638357a65a655`, and squash-merged as `f3bd629126b8572bda32dd1f8da253d3491430b7`.
- Open PR audit during this reconciliation found exactly one open PR: Dependabot #51 (`duckdb` and `ureq` grouped update). PRs #52-#55 are no longer open.

### CI

- PR #52 final exact-head CI run `32035617943` completed **success** after the compatibility repair. Its blocking quality job passed formatting, `cargo check`, strict Clippy, Rust tests, desktop npm audit, E2E npm audit, and continuity/runtime/release-workflow contracts.
- PR #52 final exact head also passed the other required ruleset contexts, including `actionlint`, secret scan, dependency review, supply-chain policy, and Rust security-extended analysis.
- Post-merge push CI run `32036339504` on merge commit `f3bd629126b8572bda32dd1f8da253d3491430b7` is **IN PROGRESS** at this state update. At the last read, formatting and `cargo check` had passed and Rust tests were running. Do not promote this run to PASS until GitHub reports completion success.
- The active ruleset requires these six PR contexts: `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, `dependency vulnerability and license review`, `policy + audit + deny + machete`, and `Rust security-extended analysis`.
- Any new or rebased PR must qualify its **exact final head**; historical success does not transfer to a moved head.

### Release Distribution

- GitHub Release ID `371168378`, tag `desktop-v0.1.1`, was re-fetched during this reconciliation and remains public with `draft=false`, `prerelease=false`, and `immutable=true`.
- Release target remains exact qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- The six release assets remain present with the previously verified digests; no release mutation was performed during maintenance.
- Publication gate run `32022865218`, job `95366007733`, remains the historical publication/immutability/provenance evidence.
- Windows Authenticode is not configured. macOS Developer ID/notarization is not configured; build provenance must not be represented as OS publisher signing.

### Draft Release Automation

- `.github/workflows/desktop-release-draft.yml` remains repaired from PR #59.
- Preflight uses draft-inclusive authenticated release listing, creation resolves a unique draft by exact tag/target, and post-create verification uses the numeric release ID.
- `tools/kgw_desktop_release_draft_workflow_gate.cjs` continues to guard those semantics in blocking CI.

### Staging

- Staging deployment/runtime state: **NOT VERIFIED**. No staging runtime was started, stopped, or inspected in this maintenance session.

### Production / Live Runtime

- Live Kaspa node/bridge runtime state: **NOT VERIFIED**. Dependency CI and release metadata are not runtime-health evidence.
- No claim of mainnet/testnet runtime success is made from #52 tests.
- `testnet12` live smoke was not run and remains explicit opt-in.

### External Dependencies

- Official runtime repository bindings remain defined by repository configuration; live external-runtime availability/version state is **NOT VERIFIED** in this maintenance audit.
- Owner credential-retirement state remains **NOT VERIFIED** because the available GitHub connector exposes no authorized secret-deletion or fine-grained PAT-revocation action.

## Drift

- Repository handoff drift found and reconciled in progress: the prior state said Dependabot #51-#55 remained open, but live GitHub shows #52-#55 merged and only #51 open.
- Current remote-main observation advanced beyond the historical continuity baseline; current values remain dynamically verified rather than embedded as permanent labels.
- Qualified source ↔ immutable published Desktop `0.1.1` release: **NONE FOUND** in the latest release re-fetch.
- Repository ruleset ↔ documented six-context/squash-only policy: **NONE FOUND** in the latest ruleset re-fetch.
- Local dedicated checkout/worktree drift: **NOT VERIFIED** because the checkout is not accessible from this execution environment.
- Staging/runtime drift: **NOT VERIFIED**.
- Owner credential-retirement drift: **NOT VERIFIED** until an authorized administration surface confirms removal.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane around official Kaspa node and Stratum bridge runtimes. It does not reimplement Kaspa consensus or official runtime behavior. Raw runtime log panes must contain native official stdout/stderr only.

## Important Paths

- `AGENTS.md` — permanent agent/engineering/session-start rules.
- `PROJECT_STATE.md` — canonical current resumable summary.
- `PLANS.md` — active multi-stage plan only; currently inactive.
- `docs/adr/0011-repository-native-project-continuity.md` — accepted continuity/source-of-truth decision.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `tools/kgw_project_continuity_gate.cjs` — blocking continuity contract.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Repository-native continuity surfaces and Desktop `0.1.1` release/workflow engineering remain complete.
- Dependabot PRs #53, #54, and #55 were independently rebased, exact-head qualified, squash-merged, and followed by successful push CI on their resulting `main` commits.
- Dependabot PR #52 exposed a real upstream API migration, was repaired minimally on the same PR, exact-head qualified with `cargo check`, strict Clippy, Rust tests, npm audits, security/supply-chain gates, and then squash-merged.
- No Desktop `0.1.1` release asset, tag, source binding, or immutable release metadata was changed by this maintenance sequence.

## Last Verified Validation

### Project Continuity Contract

- Command represented by CI: `node tools/kgw_project_continuity_gate.cjs`.
- Result: **PASS** on PR #52 exact final head in CI run `32035617943`.
- Post-merge run `32036339504` is still **IN PROGRESS** at this update.

### Formatting / Build Static Check

- `cargo fmt --all -- --check`: **PASS** on PR #52 exact final head and already PASS in the in-progress post-merge run.
- `cargo check --locked --workspace --all-targets`: **PASS** on PR #52 exact final head and already PASS in the in-progress post-merge run.

### Clippy

- Strict repository Clippy lanes: **PASS** on PR #52 exact final head.

### Rust Tests

- `cargo test --locked --workspace --all-targets`: **PASS** on PR #52 exact final head.
- Post-merge run `32036339504`: **IN PROGRESS** at this update.

### JavaScript / npm Validation

- Desktop JavaScript lint, E2E lint/syntax checks, desktop npm audit, E2E npm audit, and Node.js 26 compatibility: **PASS** on PR #52 exact final head.

### Release / Distribution Verification

- Release ID `371168378` re-fetch: **PASS** for public immutable `desktop-v0.1.1` identity and unchanged qualified target.

### Live Runtime Verification

- Result: **NOT VERIFIED**.
- Reason: no live node/bridge runtime was executed or inspected in this maintenance session.

### Graphify

- Result: **NOT VERIFIED / ENVIRONMENT-LIMITED** in this agent environment.
- Reason: the dedicated server checkout is not mounted here and no local Graphify execution surface is available. No Graphify PASS is claimed.

## Known Issues / Blockers

- Owner security hygiene remains external: retire `RELEASE_ADMIN_TOKEN` and the associated short-lived fine-grained PAT; status remains **NOT VERIFIED** until an authorized administration surface confirms removal.
- Open maintenance PR #51 groups `duckdb` 1.4.2 -> 1.10505.0 and `ureq` 3.3.0 -> 3.4.0. Upstream release notes explicitly mark DuckDB API changes as breaking; this PR must not be treated as a routine lockfile-only update.
- RustSec managed advisory policy remains governed by repository configuration; do not claim all advisories are eliminated.

## Risks

- Mutating immutable `desktop-v0.1.1` would violate the completed release boundary.
- Merging #51 without exact-head build/tests could import a breaking DuckDB API migration despite the grouped Dependabot label.
- Recording a static SHA as forever-current `main`/HEAD would recreate self-stale handoff drift.
- Treating CI, mocks, or release publication as live runtime proof would violate runtime safety policy.

## Constraints / Invariants

- Preserve official runtime ownership and official runtime bindings.
- Preserve zero-fake-log policy.
- `mainnet` and `testnet10` remain stable; `testnet12` remains experimental explicit opt-in.
- Do not use destructive Git cleanup or discard user work.
- Use PR-based integration and exact-head qualification for material repository changes.
- Do not mutate immutable `desktop-v0.1.0` or `desktop-v0.1.1`.
- Never record secret values, private keys, passwords, cookies, or access tokens in project state.

## DO NOT CHANGE WITHOUT EXPLICIT REASON

- Runtime ownership model and official runtime bindings.
- Raw runtime-log invariant.
- Network stability classification.
- Immutable `desktop-v0.1.0` and `desktop-v0.1.1` releases.
- Published Desktop `0.1.1` source/tag binding to `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Draft release-ID verification semantics without a reviewed replacement that preserves draft-inclusive discovery and regression coverage.
- Continuity source-of-truth hierarchy without a superseding ADR.

## Pending Decisions

- No Desktop `0.1.1` release-content/source/publication/recovery/workflow-repair or continuity-hardening decision remains.
- PR #51 requires independent maintenance review because the DuckDB update has documented breaking API changes.

## NEXT ACTION

1. Wait for post-merge push CI run `32036339504` on merge commit `f3bd629126b8572bda32dd1f8da253d3491430b7` to complete; classify the exact result without waiver.
2. Finalize and merge this `PROJECT_STATE.md` reconciliation only after the current `main` CI result is known and the state reflects it accurately.
3. Re-fetch live `main` and PR #51, rebase #51 onto the resulting current `main`, and treat every earlier #51 check as stale.
4. Review #51 as a potentially breaking DuckDB migration, not a routine minor/patch update. Require exact-head formatting, Cargo check, strict Clippy, Rust tests, npm audits, and all six ruleset contexts; add only the smallest API adaptation if evidence proves one is needed.
5. Squash-merge #51 only if its exact final head is fully qualified and has no unresolved review threads; then verify post-merge push CI.
6. Reconcile `PROJECT_STATE.md` again after the final maintenance transition.
7. Owner credential retirement remains owner-only/administration-only and stays `NOT VERIFIED` unless an authorized tool becomes available.

## Resume Instructions

1. Read `AGENTS.md` first and `AGENTS.override.md` if present.
2. Read `PROJECT_STATE.md` second.
3. Inspect actual local Git branch, HEAD, working tree, latest commit, remote tracking, live GitHub `main`, open PRs, required checks/ruleset, releases, and task-relevant runtime/deployment facts.
4. If verified reality differs from this summary, reconcile `PROJECT_STATE.md` before relying on stale claims.
5. Read only ADRs/runbooks relevant to the active task.
6. Read `PLANS.md` only when it contains an active multi-stage plan.
7. Treat conversation memory, old chats, exported handoff archives, and screenshots as advisory only.
8. Do not repeat completed work unless fresh evidence proves it incomplete or regressed.
9. Continue from `NEXT ACTION` unless the owner explicitly changes priority.
