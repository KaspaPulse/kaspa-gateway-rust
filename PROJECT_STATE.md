# PROJECT STATE

## Metadata

- Last state update: 2026-08-17 14:38 +03:00.
- State author/agent: repository-continuity conformance audit and hardening session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This document is the canonical resumable **summary**, not a substitute for Git, GitHub Actions, GitHub Releases, repository settings, or live runtime evidence.

## Executive Status

- Overall status: **DESKTOP 0.1.1 RELEASE/WORKFLOW ENGINEERING CLOSED; REPOSITORY-NATIVE CONTINUITY SYSTEM ACTIVE; OWNER CREDENTIAL RETIREMENT PENDING**.
- Current objective: preserve the verified continuity/source-of-truth model and resume ordinary maintenance without reopening the completed Desktop `0.1.1` release plan.
- Current engineering blocker: **NONE** for Desktop `0.1.1` release, artifact integrity, provenance, immutability, tag binding, or draft-workflow repair.
- Owner-only security hygiene remains: remove repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the short-lived fine-grained PAT that supplied it. Never expose the credential value in chat, repository files, logs, evidence bundles, or documentation.

## Repository State

- Default integration branch: `main`.
- Current branch: **VERIFY DYNAMICALLY** with `git branch --show-current` in the actual checkout.
- Current HEAD: **VERIFY DYNAMICALLY** with `git rev-parse HEAD`.
- Current remote main: **VERIFY DYNAMICALLY** from Git/GitHub before any decision that depends on it.
- Working tree: **NOT VERIFIED** in the dedicated server checkout from this agent environment.
- Remote tracking state: **NOT VERIFIED** in a local checkout; inspect `git status --short --branch` and remotes when resuming locally.
- Verified code baseline (audit-start evidence): `5be1b59de4e9a6b60e295423a9c5a36399ad2fd9` (`docs: close desktop 0.1.1 release plan (#60)`). This is evidence of what was verified at the start of the continuity audit, not a claim that it remains the current HEAD forever.
- Verified application/workflow implementation baseline: `197bdead257973164931b34cce20c4556820df44` (`fix: verify desktop release drafts by release ID (#59)`).
- State-document commit: derive dynamically from Git when needed; do not copy a self-referential state SHA here.
- Qualified Desktop `0.1.1` source and published tag target: `b911eb44619f8eab706bc2fe786d1c84ac958f1d` (`release: prepare desktop 0.1.1 (#50)`).
- Open-PR audit snapshot at 2026-08-17 14:38 +03:00: Dependabot #51, #52, #53, #54, and #55 only. Query dynamically before acting.
- Ruleset audit snapshot: ID `20627285`, `main-rebootstrap-baseline`, enforcement active, squash-only integration, unresolved review threads required, no bypass actors, and six required PR contexts. Re-query before merge-policy decisions.

## Uncommitted Work

- Dedicated server checkout/worktree: **NOT VERIFIED**. Do not discard or hide local work without inspecting it first.
- Temporary remote operations branch `ops/release-011-recovery-gate-20260817` remains evidence-only and must not be merged as application history.
- The temporary exact-publication workflow is absent from that branch's current workflow directory; the branch retains only ordinary repository workflows.
- Repository secret/PAT retirement is an external GitHub-settings action, not uncommitted repository code.

## Desired State

- Preserve official Kaspa runtime ownership and official runtime bindings; do not reimplement official runtime behavior.
- Preserve the zero-fake-log invariant: raw runtime panes show real official stdout/stderr only.
- Keep `mainnet` and `testnet10` stable/supported; keep `testnet12` experimental and explicit opt-in.
- Keep immutable Desktop `0.1.1` bound to source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` with its verified six-asset release set unchanged.
- Preserve repaired draft-release semantics: draft-inclusive discovery, unique draft resolution by tag/target, numeric release-ID propagation, and ID-based post-create verification.
- Keep `AGENTS.md` stable policy, `PROJECT_STATE.md` current state, ADRs durable decisions, runbooks repeatable operations, and `PLANS.md` active only when a genuine multi-stage plan exists.
- Keep current-HEAD/current-branch/current-working-tree facts dynamic so documentation cannot become stale merely because the documentation itself was committed.

## Actual State

### Repository

- Desktop package/release line remains `0.1.1` for the completed release described below.
- Rust workspace policy is Rust `1.97.1`, edition `2024`.
- Desktop Node engine is `>=24 <27`; blocking CI currently uses Node `24.19.0` and npm `11.17.0`.
- `AGENTS.md`, `PROJECT_STATE.md`, `PLANS.md`, `docs/adr/README.md`, accepted ADR-0011, architecture documentation, release runbook, and repository-native continuity gate are present.
- `PLANS.md` is intentionally in **NO ACTIVE MULTI-STAGE PLAN** state after closure of the Desktop `0.1.1` plan.

### CI

- GitHub Actions push CI run `32025676832` on verified audit-start baseline `5be1b59de4e9a6b60e295423a9c5a36399ad2fd9` completed **success**.
- Its blocking `quality (rust + npm)` job `95374469518` completed **success** on `ubuntu-24.04` and included the project-continuity contract, formatting, Cargo check, strict Clippy, Rust tests, and both npm audits.
- Node.js 26 compatibility job `95374469620` also completed **success** and remains non-blocking.
- The active ruleset requires these six PR contexts: `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, `dependency vulnerability and license review`, `policy + audit + deny + machete`, and `Rust security-extended analysis`.
- Any new PR must qualify its **exact final head**; historical success does not transfer to a moved head.

### Release Distribution

- GitHub Release ID `371168378`, tag `desktop-v0.1.1`, is public with `draft=false`, `prerelease=false`, and `immutable=true`.
- Release target and actual git tag ref resolve to exact qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Published at `2026-08-17T11:01:16Z` (`2026-08-17 14:01:16 +03:00`).
- Verified release asset SHA-256 digests:
  - macOS provenance bundle: `821b58d56bc894d43ee8f1b9962ff9394f09f0476d43ed86843311b567104b0d`;
  - macOS App ZIP: `82195836a67e143bf5b4d085bf42b7491d5332b05ac19cb79980df39aa5a8e9f`;
  - macOS DMG: `ba5150b92860ca316e85e5fa90d27c60f2e15b0a7f0c6beed8ac970eb18b6387`;
  - Windows provenance bundle: `9b6dc872f784f99841ecf25e4151250f7f5688677b51a72ba289cf133ae6dfa8`;
  - Windows NSIS installer: `eff90f265fe782fb186f5d79950a2b0c42ff463ad36fcdf2675526a4d4133082`;
  - `SHA256SUMS.txt`: `c6e4dd07f91b721e37d69b778a640e9af3fc55e25d1f2ad33ca4635ac1087624`.
- Publication gate run `32022865218`, job `95366007733`, passed independent asset/checksum/provenance verification, immutable-release administration gating, publication, release/asset verification, and post-publication build-attestation verification.
- Windows Authenticode is not configured. macOS Developer ID/notarization is not configured; the artifacts are ad-hoc signed. Build provenance must not be represented as OS publisher signing.

### Draft Release Automation

- `.github/workflows/desktop-release-draft.yml` was repaired in PR #59.
- Preflight uses authenticated paginated release listing so existing drafts are visible.
- Creation resolves the draft uniquely by `draft=true`, exact `tag_name`, and exact `target_commitish`, exports the numeric release ID, and post-create verification reads `/releases/{release_id}`.
- Post-create draft verification does not depend on the published-release-by-tag endpoint or on a public git tag/ref.
- `tools/kgw_desktop_release_draft_workflow_gate.cjs` guards those semantics in blocking CI.

### Staging

- Staging deployment/runtime state: **NOT VERIFIED**. No staging platform/runtime metadata was queried in this continuity audit.
- A GitHub draft release is release-distribution state, not proof of a healthy staging runtime.

### Production / Live Runtime

- Live Kaspa node/bridge runtime state: **NOT VERIFIED**. No local production runtime was started, stopped, or inspected as part of this continuity audit.
- GitHub Release publication proves distribution state only; it is not proof that any node/bridge process is running or healthy.
- `testnet12` live smoke was not run and must remain explicit opt-in.

### External Dependencies

- Official runtime repository bindings remain defined by repository configuration; live external-runtime availability/version state is **NOT VERIFIED** in this audit.
- Owner credential-retirement state is **NOT VERIFIED** because the available GitHub connector does not expose Actions-secret deletion or fine-grained PAT revocation.

## Drift

- Qualified source ↔ published tag: **NONE FOUND** in the latest release/tag verification.
- Qualified source ↔ published release asset digests/provenance: **NONE FOUND** in the release verification evidence.
- Repository automation ↔ desired draft semantics: **NONE FOUND**; the repair and blocking contract are present.
- Canonical handoff ↔ dynamic Git HEAD: designed to avoid duplicate truth; branch/HEAD/working tree are **VERIFY DYNAMICALLY / NOT VERIFIED** rather than static current-state claims.
- Staging/runtime drift: **NOT VERIFIED**.
- Owner credential-retirement drift: **NOT VERIFIED** until the secret and PAT are confirmed removed.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane around official Kaspa node and Stratum bridge runtimes. It does not reimplement Kaspa consensus or official runtime behavior. Raw runtime log panes must contain native official stdout/stderr only. See `docs/architecture/README.md` and relevant ADRs for durable design detail.

## Important Paths

- `AGENTS.md` — permanent agent/engineering/session-start rules.
- `PROJECT_STATE.md` — canonical current resumable summary.
- `PLANS.md` — active multi-stage plan only; currently inactive.
- `docs/adr/README.md` — ADR lifecycle/index.
- `docs/adr/0011-repository-native-project-continuity.md` — accepted continuity/source-of-truth decision.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `docs/operations/live-network-smoke.md` — live-network smoke procedure.
- `tools/kgw_project_continuity_gate.cjs` — blocking continuity contract.
- `tools/kgw_desktop_release_draft_workflow_gate.cjs` — blocking draft-release semantics contract.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Repository-native continuity surfaces and ADR/runbook structure were established and CI-guarded.
- Desktop `0.1.1` artifacts/provenance were qualified from exact source `b911eb4...`.
- Existing release ID `371168378` was recovered, verified, published in place, and made immutable without retargeting or replacement.
- Draft-workflow semantics were repaired and exact-head qualified in PR #59.
- Release-plan state was closed through PR #60.
- The temporary exact-publication workflow was retired from the evidence branch after use.
- This continuity hardening removes static current-main/HEAD claims and retires completed-plan narrative from the active planning surface; its own PR/CI qualification must be verified before relying on it as merged `main` state.

## Last Verified Validation

### Project Continuity Contract

- Command: `node tools/kgw_project_continuity_gate.cjs`
- Result: **PASS** in GitHub Actions push CI run `32025676832`, job `95374469518`, on `ubuntu-24.04`.
- Evidence baseline: `5be1b59de4e9a6b60e295423a9c5a36399ad2fd9`.

### Formatting / Build Static Check

- Command: `cargo fmt --all -- --check`
- Result: **PASS** in job `95374469518`.
- Command: `cargo check --locked --workspace --all-targets`
- Result: **PASS** in job `95374469518`.

### Clippy

- Commands:
  - `cargo clippy --locked --workspace --lib --bins --examples --benches -- -D warnings`
  - `cargo clippy --locked --workspace --tests --exclude kaspa-gateway-desktop -- -D warnings`
  - `cargo clippy --locked -p kaspa-gateway-desktop --tests -- -D warnings -A dead-code`
- Result: **PASS** in job `95374469518`.

### Rust Tests

- Command: `cargo test --locked --workspace --all-targets`
- Result: **PASS** in job `95374469518`.

### JavaScript / npm Validation

- Desktop JavaScript lint, E2E lint/syntax checks, desktop npm audit, and E2E npm audit: **PASS** in job `95374469518`.
- Node.js 26 compatibility job `95374469620`: **PASS**, non-blocking.

### Release / Distribution Verification

- Method: GitHub Release API/tag-ref verification plus publication-gate release/asset/attestation checks.
- Result: **PASS** for immutable Desktop `0.1.1` distribution identity as described above.

### Live Runtime Verification

- Result: **NOT VERIFIED**.
- Reason: this audit did not execute or inspect live node/bridge runtimes; CI and release metadata are not runtime-health evidence.

### Graphify

- Result: **NOT VERIFIED / ENVIRONMENT-LIMITED** in this agent environment.
- Reason: the dedicated server checkout is not mounted here, `graphify` is not installed locally, and no committed `graphify-out/graph.json` exists on the verified repository baseline. The Graphify skill was read; no Graphify PASS is claimed.

## Known Issues / Blockers

- No active Desktop `0.1.1` repository/release engineering blocker remains.
- Owner security hygiene: remove `RELEASE_ADMIN_TOKEN` from repository Actions secrets and revoke/delete the associated short-lived fine-grained PAT; status remains **NOT VERIFIED** until owner-side confirmation or an authorized administration tool can verify removal.
- RustSec managed advisory set remains governed by the repository's current accepted policy; do not claim all advisories are eliminated.
- Dependabot #51-#55 remain independent maintenance proposals, not release blockers.

## Risks

- Mutating immutable `desktop-v0.1.1` would violate the completed release boundary.
- Reintroducing published-tag/ref assumptions into draft-only staging would recreate the repaired failure mode.
- Recording a static SHA as forever-current `main`/HEAD would recreate self-stale handoff drift after the next documentation commit.
- Keeping completed execution-plan history in `PLANS.md` as active state would duplicate Git/PR/release history and confuse new sessions.
- Treating Sigstore/SLSA provenance as Authenticode or Apple Developer ID/notarization would misstate trust guarantees.
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

- No Desktop `0.1.1` release-content/source/publication/recovery/workflow-repair decision remains.
- Dependabot #51-#55 require independent future review if selected for maintenance.

## NEXT ACTION

1. Owner security cleanup: in repository Actions secrets, remove `RELEASE_ADMIN_TOKEN`; then revoke/delete the associated short-lived fine-grained PAT in GitHub account settings. Do not paste or record its value.
2. Verify the cleanup from an authorized administration surface if available; otherwise keep it explicitly `NOT VERIFIED`.
3. For the next engineering task, start from fresh verified Git/GitHub state and ordinary maintenance priorities rather than reopening Desktop `0.1.1` release work.
4. If a future task is multi-stage/high-risk, activate a new scoped `PLANS.md`; otherwise leave it at **NO ACTIVE MULTI-STAGE PLAN**.
5. Review Dependabot #51-#55 independently when desired and exact-head qualify any selected PR before merge.

## Resume Instructions

Resume protocol for a new ChatGPT/Codex session:

1. Read `AGENTS.md` first.
2. Read `PROJECT_STATE.md` second.
3. Inspect actual Git branch, HEAD, working tree, remote tracking, live GitHub `main`, open PRs, required checks/ruleset, releases, and any runtime/deployment facts relevant to the task. Do not assume this file is current forever.
4. If verified reality differs from this summary, reconcile `PROJECT_STATE.md` before relying on the stale claim.
5. Read only ADRs relevant to the task.
6. Read the relevant runbook before release, deployment, rollback/recovery, live-network smoke, or incident actions.
7. Read `PLANS.md` only when its status says an active multi-stage plan exists; otherwise do not treat completed-plan history as active work.
8. Treat conversation memory, old chats, exported handoff archives, and screenshots as advisory only.
9. Do not repeat completed work unless fresh evidence proves it incomplete or regressed.
10. Continue from `NEXT ACTION` unless the owner explicitly changes priority.
