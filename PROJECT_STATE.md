# PROJECT STATE

## Metadata

- Last state update: 2026-09-10 18:45 +03:00.
- State author/agent: Remote Desktop Commander continuity reconciliation session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This document is the canonical resumable summary after reconciliation; Git/GitHub, CI, release metadata, and live runtime evidence remain the owning sources for their facts.

## Executive Status

- Overall status: **DESKTOP 0.1.1 RELEASE BOUNDARY PRESERVED; SEPTEMBER DEPENDENCY MAINTENANCE CLOSED; SAFE DEPENDABOT AUTO-MERGE ACTIVE; OWNER CREDENTIAL RETIREMENT NOT VERIFIED**.
- Current objective: complete authorized external integration of the verified continuity/security/knowledge-management lifecycle through warning closure, protected PR qualification, squash merge, and post-merge verification without reopening completed release work.
- Current engineering blocker: **NONE**.
- Open pull requests at reconciliation time: **NONE**.
- `PLANS.md` is **ACTIVE** for the authorized warning-closure and GitHub integration phase of the 2026-09-10 continuity/security/knowledge-management task.
- Owner-only security hygiene remains external and **NOT VERIFIED**: remove repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the associated short-lived fine-grained PAT. Its value must never be recorded in chat, repository files, logs, evidence bundles, or documentation.

## Repository State

- Default integration branch: `main`.
- Current HEAD: **VERIFY DYNAMICALLY** with `git rev-parse HEAD`; do not embed the state-document commit as a forever-current HEAD.
- Current remote main: **VERIFY DYNAMICALLY** from Git/GitHub before any decision that depends on it; the timestamped reconciliation observation is recorded below.
- Working tree: **CLEAN** at the verified session baseline before this isolated state-document edit; classify it dynamically on every resume.
- State-document commit: derive dynamically from Git when needed; do not copy a self-referential state SHA into this document.
- Verified code baseline (historical reconciliation observation): `9a7b18f76dd6184785a4cf972daa1431ee07138f`.
- Verified continuity-lifecycle implementation commit (historical evidence): `f270d5c811176396df0a6c06ac9ad983cb7f229b`.
- Dedicated checkout: `/home/kas/kaspa-gateway-dev/codex/kaspa-gateway-rust`.
- Origin: `https://github.com/KaspaPulse/kaspa-gateway-rust.git`.
- Verified remote `main`: `9a7b18f76dd6184785a4cf972daa1431ee07138f` (`chore(deps): bump the github-actions group with 4 updates (#67)`).
- The dedicated checkout was fast-forwarded from `b911eb44619f8eab706bc2fe786d1c84ac958f1d` to the verified remote head with no local divergence (`ahead=0`, `behind=0`) before this documentation reconciliation.
- Current task branch: `feature/project-continuity-lifecycle-20260910`, isolated in `/home/kas/kaspa-gateway-dev/codex/worktrees/project-continuity-lifecycle-20260910`; no push or pull request has been performed by this task.
- Working tree was clean before the reconciliation branch was created.
- Open pull request query returned an empty set at reconciliation time.
- PR #51 is `CLOSED` and was not merged; it is no longer an active maintenance item.
- Active ruleset `main-rebootstrap-baseline` (ID `20627285`) remains enforced with linear history, squash-only pull-request integration, resolved review threads, six baseline required contexts, and no bypass actors.
- Active ruleset `kgw-dependency-required-checks-v1` (ID `22426271`) applies to `main`, uses strict up-to-date required checks, adds `Rust address fuzzing` to the protected dependency-maintenance surface, and has no bypass actors.
- Repository variable `KGW_DEPENDABOT_AUTOMERGE` is verified as `enabled`.
- Published Desktop `0.1.1` tag target remains `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.

## Uncommitted Work

- No pre-existing user source-code changes were present when this session began.
- The 2026-09-10 continuity lifecycle task is complete locally on its isolated worktree/branch; no unrelated application/runtime changes were made and external integration was not performed.
- A stale/prunable Git worktree registration under `/tmp/kaspa-gateway-lanes/.../p10-macos-qualification-from-dmg` was observed. It is not part of the active checkout and is not being removed without a separate cleanup reason.
- Repository secret/PAT retirement remains an external GitHub-settings action, not repository code.

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

- Desktop package/release line remains `0.1.1`.
- Rust workspace toolchain is now Rust `1.98.1`; edition remains `2024`.
- Desktop Node engine remains `>=24 <27`.
- `AGENTS.override.md` is absent in the verified current checkout; `AGENTS.md` is the active repository policy.
- `PLANS.md` is **ACTIVE** while the verified local lifecycle implementation is being exact-head qualified and integrated through GitHub.
- The September dependency-maintenance sequence is present in verified `main` history:
  - PR #75: dependency security repair and safe Dependabot auto-merge workflow.
  - PR #71: grouped Cargo minor/patch maintenance.
  - PR #74: Rust toolchain `1.97.1` -> `1.98.1`.
  - PR #72: desktop npm minor/patch maintenance.
  - PR #73: E2E npm minor/patch maintenance.
  - PR #67: GitHub Actions maintenance.
- Current `main` is `9a7b18f76dd6184785a4cf972daa1431ee07138f` and has no open pull requests at this reconciliation boundary.
- `.github/workflows/dependabot-auto-merge.yml` is present. It enables protected squash auto-merge only for verified Dependabot minor/patch updates, excludes dependency sets containing `duckdb`, and does not auto-approve major updates.

### CI

- Exact current `main` head `9a7b18f76dd6184785a4cf972daa1431ee07138f` has successful push runs for `CI`, `Workflow Lint`, `Secret Scan`, `OpenSSF Scorecard`, `Dependency & Supply Chain Security`, and `CodeQL`.
- Current-head push CI run `34114469182` completed **success**.
- Current-head dependency/supply-chain run `34114469226` completed **success**.
- Current-head CodeQL push run `34114469224` completed **success**; scheduled CodeQL run `34450142563` also completed **success**.
- Current-head Secret Scan push run `34114469219` completed **success**; scheduled Secret Scan run `34332233837` also completed **success**.
- Strict dependency ruleset ID `22426271` requires these seven contexts on `main`: `quality (rust + npm)`, `policy + audit + deny + machete`, `dependency vulnerability and license review`, `Rust security-extended analysis`, `TruffleHog verified and unknown secrets`, `actionlint`, and `Rust address fuzzing`.
- Historical success never transfers to a moved pull-request head; future protected merges must satisfy the applicable exact-head rules.

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

- The previous handoff was stale relative to live repository state: it still treated PR #51 as the next maintenance action and did not describe the completed September dependency-maintenance batch.
- Dedicated checkout drift is now reconciled: local `main` was verified clean, fetched, and fast-forwarded to exact remote `main` `9a7b18f76dd6184785a4cf972daa1431ee07138f` with zero divergence before this documentation branch was created.
- The previous Rust `1.97.1` policy claim was stale; `rust-toolchain.toml` now pins `1.98.1`.
- Safe Dependabot auto-merge and the strict seven-check ruleset are newer than the previous handoff and are now represented here.
- Qualified source ↔ published Desktop `0.1.1` release: no drift observed in the tag target re-fetch performed during this reconciliation.
- Staging/runtime drift remains **NOT VERIFIED** because no live node/bridge runtime was started or inspected.
- Owner credential-retirement drift remains **NOT VERIFIED** until an authorized administration surface confirms removal.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane around official Kaspa node and Stratum bridge runtimes. It does not reimplement Kaspa consensus or official runtime behavior. Raw runtime log panes must contain native official stdout/stderr only.

## Important Paths

- `AGENTS.md` — permanent agent/engineering/session-start rules.
- `PROJECT_STATE.md` — canonical current resumable summary.
- `ACTIVE_TASK.md` — current task objective, phase, blocker, verification, next action, and completion criteria.
- `CURRENT_STATE.md` — concise operational handoff for a newly starting session.
- `PLANS.md` — active warning-closure / protected-integration plan for the current task.
- `docs/continuity/PROJECT_CONTINUITY_POLICY.md` — detailed continuity/security-engineering/knowledge-management policy.
- `docs/handoff-ledger/` — durable atomic checkpoints.
- `docs/project-memory/` — stable-ID durable bugs, regressions, security findings, incidents, decisions, and known failures.
- `docs/adr/0011-repository-native-project-continuity.md` — accepted continuity/source-of-truth decision.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `tools/kgw_project_continuity_gate.cjs` — blocking continuity contract.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Repository-native continuity now includes `ACTIVE_TASK.md`, `CURRENT_STATE.md`, atomic handoff checkpoints, permanent project-memory categories/templates/records, and fail-closed regression tests in addition to the pre-existing `PROJECT_STATE.md`/`PLANS.md`/ADR/runbook model.
- The verified local implementation commit for that lifecycle is `f270d5c811176396df0a6c06ac9ad983cb7f229b`; derive current HEAD dynamically on resume.
- Verified current `main` history contains the September maintenance sequence PRs #75, #71, #74, #72, #73, and #67.
- Current remote `main` is `9a7b18f76dd6184785a4cf972daa1431ee07138f`; the dedicated checkout matched it with zero ahead/behind divergence before this local documentation branch was created.
- No open pull requests were present at the reconciliation boundary.
- Protected Dependabot minor/patch auto-merge is present and the repository variable is enabled; major updates and dependency sets containing `duckdb` remain outside unattended auto-merge.
- The current `main` exact-head CI/security workflow set observed in this reconciliation is successful.
- No Desktop `0.1.1` release asset, tag, or source target was changed by this reconciliation session.

## Last Verified Validation

### Project Continuity Contract

- `node --check tools/kgw_project_continuity_gate.cjs`: **PASS**.
- `node --check tools/kgw_project_continuity_gate_tests.cjs`: **PASS**.
- `node tools/kgw_project_continuity_gate.cjs`: **PASS** with active/current state, handoff/project memory, dynamic Git-state, regression/security lifecycle, plan/ADR, and release-runbook checks.
- `node tools/kgw_project_continuity_gate_tests.cjs`: **PASS** with one positive fixture and five fail-closed negative cases.
- Additional temporary negative checks for missing `ACTIVE_TASK.md`, no durable checkpoint, and invalid project-memory status failed as expected and recovered after restoration.
- `git diff --check`: **PASS**.
- Python YAML parse of `.github/workflows/ci.yml`: **PASS**.
- `actionlint 1.7.12 -no-color`: **PASS** after installing the checksum-verified official Linux binary under the user-local tool directory.
- `pwsh 7.6.6 -NoProfile -ExecutionPolicy Bypass -File tools/kgw_ai_workflow_gate.ps1`: **PASS** after installing the checksum-verified official portable PowerShell archive under the user-local tool directory.
- Graphify local integration state: `.codex/hooks.json` is valid local ignored JSON, `post-commit` and `post-checkout` hooks are installed, and the Graphify merge driver is registered.

### Current Main CI

- Exact head: `9a7b18f76dd6184785a4cf972daa1431ee07138f`.
- Push CI run `34114469182`: **success**.
- Workflow Lint run `34114469184`: **success**.
- OpenSSF Scorecard run `34114469188`: **success**.

### Security / Supply Chain

- Dependency & Supply Chain Security run `34114469226`: **success**.
- CodeQL push run `34114469224`: **success**.
- Secret Scan push run `34114469219`: **success**.
- Scheduled CodeQL run `34450142563` on the same head: **success**.
- Scheduled Secret Scan run `34332233837` on the same head: **success**.

### Release / Distribution Verification

- `desktop-v0.1.1` re-fetch: `draft=false`, `prerelease=false`, published target `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Historical immutability/provenance evidence remains in the project record; immutability was not independently re-queried by this session's limited release command.

### Live Runtime Verification

- Result: **NOT VERIFIED**.
- Reason: no live node/bridge runtime was started, stopped, or inspected in this reconciliation session.

### Graphify

- `.codex/skills/graphify/SKILL.md` was read completely before programming changes as required by `AGENTS.md`.
- Local Graphify was upgraded from `0.9.32` to `0.9.57`; Git hooks were refreshed after the upgrade.
- Root-cause investigation proved the earlier `FAIL-0001` counts came from treating a raw `--no-cluster` pre-build extraction as final graph health. Graphify source explicitly drops absent external/stdlib endpoints during normal build as expected behavior.
- `graphify extract . --code-only --force`: **PASS**, producing the normal final graph with 5,017 nodes, 12,834 edges, and 244 communities without API-backed semantic extraction.
- `graphify diagnose multigraph --graph graphify-out/graph.json --json`: **PASS/CLEAN** with zero missing endpoints, dangling endpoints, self-loops, exact duplicate edges, or directed/undirected same-endpoint collapse candidates.
- Graphify MultiDiGraph capability probe: **PASS** on Python 3.12.3 / NetworkX 3.6.1; the installed Graphify source identifies opt-in `--multigraph` as a future capability, not a current project requirement.
- Focused post-change query: **PASS**; it resolves the continuity gate/test, `ACTIVE_TASK.md`, `CURRENT_STATE.md`, handoff ledger, and project-memory nodes.
- `FAIL-0001` is now **VERIFIED/CLOSED** with the corrected root cause and regression guidance; no final-graph health warning remains at this boundary.

## Known Issues / Blockers

- No external repository engineering blocker or open pull request was found at the continuation boundary. The continuity lifecycle implementation remains locally verified; warning closure is complete and GitHub integration is **IN PROGRESS — AUTHORIZED**.
- Owner security hygiene remains external: retire `RELEASE_ADMIN_TOKEN` and the associated short-lived fine-grained PAT; status remains **NOT VERIFIED** until an authorized administration surface confirms removal.
- Live Kaspa node/bridge runtime health remains **NOT VERIFIED** in this session.
- Major Dependabot updates and dependency groups containing `duckdb` are intentionally excluded from unattended auto-merge and require explicit compatibility review.

## Risks

- Mutating immutable `desktop-v0.1.1` would violate the completed release boundary.
- Treating Dependabot auto-merge as a protection bypass would be incorrect; required checks, branch rules, and exact-head freshness remain authoritative.
- A future major or `duckdb` update can contain breaking API changes and must not be force-fit into the routine auto-merge path.
- Treating CI, mocks, or release publication as live runtime proof would violate runtime safety policy.
- Reintroducing static forever-current HEAD claims without re-verification would recreate handoff drift.

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

- No repository maintenance pull request is currently awaiting a decision.
- No Desktop `0.1.1` release-content/source/publication/recovery/workflow-repair decision remains.
- A future major Dependabot update or any dependency group containing `duckdb` requires explicit compatibility review before integration.
- Owner credential retirement remains owner-only/administration-only and stays **NOT VERIFIED** unless an authorized tool confirms it.

## NEXT ACTION

1. Rerun all available targeted local gates on the warning-closed branch, including Node syntax, continuity gate/regression tests, `actionlint`, PowerShell AI workflow gate, YAML parse, Graphify final diagnostic/query, and `git diff --check`.
2. Commit the warning-closure/state reconciliation without a `[skip ci]` token so pull-request workflows are not suppressed.
3. Push `feature/project-continuity-lifecycle-20260910`, create a pull request against current `main`, and verify the exact pushed head.
4. Follow every required protected check to completion; root-cause and repair any failure on the same branch, preserving regression protection.
5. Squash-merge only after exact-head checks and review-thread requirements are satisfied, then verify post-merge `main` CI/security workflows and reconcile durable state/checkpoint records.

## Resume Instructions

1. Read `AGENTS.md` first and `AGENTS.override.md` if present.
2. Read `PROJECT_STATE.md`, then `ACTIVE_TASK.md` and `CURRENT_STATE.md`.
3. Read the latest relevant `docs/handoff-ledger/` checkpoint and recover `LAST CONFIRMED STATE`, `NEXT ACTION`, and `DO NOT REPEAT`.
4. Inspect actual local Git branch, HEAD, working tree, latest commit, remotes, and only task-relevant GitHub/runtime/release facts.
5. If verified reality differs from a durable state surface, reconcile it before relying on the stale claim.
6. Read only ADRs/runbooks relevant to the active task; read `PLANS.md` when it contains an active multi-stage plan.
7. Treat conversation memory, old chats, exported handoff archives, and screenshots as advisory only.
8. Do not repeat completed work unless fresh evidence proves it incomplete, untrustworthy, security-sensitive, or regressed.
9. Continue from the latest verified `NEXT ACTION` unless the owner explicitly changes priority.
