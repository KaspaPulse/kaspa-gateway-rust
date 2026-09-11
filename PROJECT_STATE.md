# PROJECT STATE

## Metadata

- Last state update: 2026-09-11 after PR #78 protected merge, successful post-merge verification, and canonical-checkout reconciliation.
- State author/agent: Remote Desktop Commander continuity reconciliation session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This document is the canonical resumable summary after reconciliation; Git/GitHub, CI, release metadata, and live runtime evidence remain the owning sources for their facts.

## Executive Status

- Overall status: **DESKTOP 0.1.1 RELEASE BOUNDARY PRESERVED; SEPTEMBER DEPENDENCY MAINTENANCE CLOSED; SAFE DEPENDABOT AUTO-MERGE ACTIVE; OWNER CREDENTIAL RETIREMENT NOT VERIFIED**.
- Current objective: **COMPLETE** — repository-native continuity/security hardening, post-merge Rust race repair, and durable closure are integrated. No active task is implied by this document.
- Current engineering blocker: **NONE for this completed task**. Residual E2E npm Low/deprecation risk remains explicit and review-bound through 2026-10-10.
- PR #76 **MERGED** as `3f8174c7e9e663da81e29eda5cd889de196eec7e`; PR #77 **MERGED** as `99b5a751e21bf6d11d6cad1ac3884e3b5f23a9e5`; PR #78 **MERGED** as historical closure baseline `50ad815b3a7569c576d7625900462734961cbc69`.
- `PLANS.md` is **NO ACTIVE MULTI-STAGE PLAN** after successful protected integration and post-merge verification.
- Owner-only security hygiene remains external and **NOT VERIFIED**: remove repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the associated short-lived fine-grained PAT. Its value must never be recorded in chat, repository files, logs, evidence bundles, or documentation.

## Repository State

- Default integration branch: `main`.
- Current HEAD: **VERIFY DYNAMICALLY** with `git rev-parse HEAD`; do not embed the state-document commit as a forever-current HEAD.
- Current remote main: **VERIFY DYNAMICALLY** from Git/GitHub before any decision that depends on it; the timestamped reconciliation observation is recorded below.
- Working tree: **VERIFY DYNAMICALLY** before every task; the canonical `main` checkout was CLEAN at the historical closure verification boundary.
- State-document commit: derive dynamically from Git when needed; do not copy a self-referential state SHA into this document.
- Verified code baseline (historical reconciliation observation): `9a7b18f76dd6184785a4cf972daa1431ee07138f`.
- Verified continuity-lifecycle implementation commit (historical evidence): `f270d5c811176396df0a6c06ac9ad983cb7f229b`.
- Dedicated checkout: `/home/kas/kaspa-gateway-dev/codex/kaspa-gateway-rust`.
- Origin: `https://github.com/KaspaPulse/kaspa-gateway-rust.git`.
- Verified final repair baseline (historical evidence): `99b5a751e21bf6d11d6cad1ac3884e3b5f23a9e5` (`fix(ci): eliminate post-ready worker race (#77)`).
- Verified closure baseline (historical evidence): `50ad815b3a7569c576d7625900462734961cbc69` (`docs: finalize continuity repair closure (#78)`).
- The dedicated checkout was fast-forwarded from `b911eb44619f8eab706bc2fe786d1c84ac958f1d` to the verified remote head with no local divergence (`ahead=0`, `behind=0`) before this documentation reconciliation.
- Current task branch: **NONE AUTHORITATIVE WHILE IDLE**; derive the actual branch dynamically before new work. Historical closure branches were merged and their remote refs were deleted automatically.
- Working tree was clean before the reconciliation branch was created.
- Open pull request query returned an empty set at reconciliation time.
- PR #51 is `CLOSED` and was not merged; it is no longer an active maintenance item.
- Active ruleset `main-rebootstrap-baseline` (ID `20627285`) remains enforced with linear history, squash-only pull-request integration, resolved review threads, six baseline required contexts, and no bypass actors.
- Active ruleset `kgw-dependency-required-checks-v1` (ID `22426271`) applies to `main`, uses strict up-to-date required checks, adds `Rust address fuzzing` to the protected dependency-maintenance surface, and has no bypass actors.
- Repository variable `KGW_DEPENDABOT_AUTOMERGE` is verified as `enabled`.
- Published Desktop `0.1.1` tag target remains `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.

## Uncommitted Work

- No pre-existing user source-code changes were present when this session began.
- The continuity/npm implementation is merged through PR #76, the deterministic post-merge race repair through PR #77, and durable closure through PR #78. No task-specific uncommitted work is expected; verify dynamically before new work.
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
- `PLANS.md` is **NO ACTIVE MULTI-STAGE PLAN**; the continuity/npm integration and post-merge race repair are complete.
- The September dependency-maintenance sequence is present in verified `main` history:
  - PR #75: dependency security repair and safe Dependabot auto-merge workflow.
  - PR #71: grouped Cargo minor/patch maintenance.
  - PR #74: Rust toolchain `1.97.1` -> `1.98.1`.
  - PR #72: desktop npm minor/patch maintenance.
  - PR #73: E2E npm minor/patch maintenance.
  - PR #67: GitHub Actions maintenance.
- Historical fully green pre-PR-#76 baseline is `9a7b18f76dd6184785a4cf972daa1431ee07138f`; PR #76 merged as `3f8174c7e9e663da81e29eda5cd889de196eec7e`; PR #77 merged as final repair baseline `99b5a751e21bf6d11d6cad1ac3884e3b5f23a9e5`.
- `.github/workflows/dependabot-auto-merge.yml` is present. It enables protected squash auto-merge only for verified Dependabot minor/patch updates, excludes dependency sets containing `duckdb`, and does not auto-approve major updates.

### CI

- PR #76 merged as `3f8174c7e9e663da81e29eda5cd889de196eec7e` after its repaired exact head satisfied protected merge requirements.
- Its first post-merge `main` CI run `34516559028` exposed `REG-0001`, a test-fixture timing race in `post_ready_worker_failure_is_non_running_durable_and_restartable_for_all_roles`.
- PR #77 exact head `871b37668f8b535827ba7e82c3027396f31c22bc` passed all protected checks: `quality (rust + npm)`, supply-chain policy, dependency review, Rust security-extended analysis, Secret Scan, actionlint, and Rust address fuzzing.
- PR #77 merged through protected squash as `99b5a751e21bf6d11d6cad1ac3884e3b5f23a9e5`; no admin bypass was used.
- Post-merge `main` runs on `99b5a751...` all succeeded: CI `34560099528`, Workflow Lint `34560099503`, Secret Scan `34560099512`, OpenSSF Scorecard `34560099501`, and CodeQL `34560099492`.
- `REG-0001` root cause and deterministic test-only READY ACK repair are verified; production startup exit detection was not weakened.
- Regression evidence: targeted cold PASS; 20/20 consecutive PASS; complete runtime IPC suite 52/52 PASS; Rust test warnings = 0.
- Npm policy from PR #76 remains active: 0 Critical/High/Moderate findings; exact Low/deprecation exceptions expire 2026-10-10 and fail closed on drift.

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
- `PLANS.md` — inactive plan sentinel; start a new body only for a genuine future multi-stage task.
- `docs/continuity/PROJECT_CONTINUITY_POLICY.md` — detailed continuity/security-engineering/knowledge-management policy.
- `docs/handoff-ledger/` — durable atomic checkpoints.
- `docs/project-memory/` — stable-ID durable bugs, regressions, security findings, incidents, decisions, and known failures.
- `docs/adr/0011-repository-native-project-continuity.md` — accepted continuity/source-of-truth decision.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `tools/kgw_project_continuity_gate.cjs` — blocking continuity contract.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Repository-native continuity includes `ACTIVE_TASK.md`, `CURRENT_STATE.md`, atomic handoff checkpoints, permanent project-memory categories/templates/records, and fail-closed regression tests.
- PR #76 integrated the continuity lifecycle, E2E npm security repair, exact residual-risk policy, and npm policy regression tests.
- PR #77 integrated the deterministic post-READY test synchronization repair plus continuity hardening for duplicate stable IDs and plan-sentinel classification.
- PR #77 exact-head required checks all passed before protected squash merge.
- Final repair baseline `99b5a751e21bf6d11d6cad1ac3884e3b5f23a9e5` has successful post-merge CI, Workflow Lint, Secret Scan, OpenSSF Scorecard, and CodeQL evidence.
- Protected Dependabot minor/patch auto-merge remains present; major updates and dependency groups containing `duckdb` remain outside unattended auto-merge.
- No Desktop `0.1.1` release asset, tag, source target, production runtime, staging runtime, or credential was changed by this task.

## Last Verified Validation

### Project Continuity Contract

- `node --check tools/kgw_project_continuity_gate.cjs`: **PASS**.
- `node --check tools/kgw_project_continuity_gate_tests.cjs`: **PASS**.
- `node tools/kgw_project_continuity_gate.cjs`: **PASS** with active/current state, handoff/project memory, dynamic Git-state, regression/security lifecycle, plan/ADR, and release-runbook checks.
- `node tools/kgw_project_continuity_gate_tests.cjs`: **PASS** with one positive fixture and six fail-closed negative cases.
- Additional temporary negative checks for missing `ACTIVE_TASK.md`, no durable checkpoint, and invalid project-memory status failed as expected and recovered after restoration.
- `git diff --check`: **PASS**.
- Python YAML parse of `.github/workflows/ci.yml`: **PASS**.
- `actionlint 1.7.12 -no-color`: **PASS** after installing the checksum-verified official Linux binary under the user-local tool directory.
- `pwsh 7.6.6 -NoProfile -ExecutionPolicy Bypass -File tools/kgw_ai_workflow_gate.ps1`: **PASS** after installing the checksum-verified official portable PowerShell archive under the user-local tool directory.
- Graphify local integration state: `.codex/hooks.json` is valid local ignored JSON, `post-commit` and `post-checkout` hooks are installed, and the Graphify merge driver is registered.

### PR #76 npm repair / warning policy

- `node --check tools/kgw_npm_dependency_policy_gate.cjs`: **PASS**.
- `node --check tools/kgw_npm_dependency_policy_gate_tests.cjs`: **PASS**.
- Desktop npm policy gate: **PASS** with 0 Critical/High/Moderate/Low and 0 deprecations.
- E2E npm policy gate: **PASS** with 0 Critical/High/Moderate, exactly 3 accepted Low nodes from GHSA-73rr-hh4g-fpgx, and exactly 2 accepted deprecations.
- npm policy regression suite: **PASS** with one positive snapshot and six fail-closed negative cases (new High, new Low, expired review, new deprecation, lock drift, stale exception).
- E2E `npm run lint`: **PASS**.
- E2E `npm run check`: **PASS** including the deepmerge security compatibility smoke.
- Policy execution was moved immediately after npm installation in blocking CI so dependency failures stop before expensive Rust compilation.

### Post-Merge Rust Regression Repair

- `REG-0001` records CI run `34516559028`, the fixed-time 40 ms post-READY fixture race, deterministic test-only ACK repair, and no-sleep-increase/no-safety-weakening boundary.
- Targeted test after cold build: **PASS**.
- Consecutive stress repetitions: **20/20 PASS**.
- Complete `integrated_runtime_ipc_smoke_tests`: **52/52 PASS**.
- Test-harness Rust warnings after module-scoped suppression: **0**; production targets remain warning-strict.
- PR #77 exact-head `quality (rust + npm)`: **PASS**; protected merge completed.
- Post-merge `main` CI run `34560099528`: **success**.

### Current Main CI

- Current remote/main identity must still be **VERIFY DYNAMICALLY** before future decisions; historical verified final repair baseline is `99b5a751e21bf6d11d6cad1ac3884e3b5f23a9e5`.
- Push CI run `34560099528`: **success**.
- Workflow Lint run `34560099503`: **success**.
- Secret Scan run `34560099512`: **success**.
- OpenSSF Scorecard run `34560099501`: **success**.
- CodeQL run `34560099492`: **success**.

### Security / Supply Chain

- PR #77 exact-head supply-chain policy/audit/deny/machete check: **success**.
- PR #77 dependency vulnerability/license review: **success**.
- PR #77 Rust security-extended analysis and Secret Scan: **success**.
- Residual E2E npm Low/deprecation exceptions remain exact, documented in `SEC-0002`, fail closed on drift, and expire on 2026-10-10.

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
- `graphify extract . --code-only --force`: **PASS**, producing the normal final graph with 5,044 nodes, 12,875 edges, and 243 communities without API-backed semantic extraction.
- `graphify diagnose multigraph --graph graphify-out/graph.json --json`: **PASS/CLEAN** with zero missing endpoints, dangling endpoints, self-loops, exact duplicate edges, or directed/undirected same-endpoint collapse candidates.
- Graphify MultiDiGraph capability probe: **PASS** on Python 3.12.3 / NetworkX 3.6.1; the installed Graphify source identifies opt-in `--multigraph` as a future capability, not a current project requirement.
- Focused post-change query: **PASS**; it resolves the new npm dependency policy gate/test and the existing continuity/security graph surfaces.
- `FAIL-0001` is **VERIFIED/CLOSED** with the corrected raw-vs-final graph root cause.
- `FAIL-0002` is **VERIFIED/CLOSED**: the root `Cargo.toml` is a virtual workspace manifest that Graphify intentionally maps to zero package nodes; root-only `.graphifyignore` now prevents the zero-node retry warning while nested crate manifests remain included.
- Final Graphify update after the root-only ignore emitted no warnings; graph health remained clean at 5,128 nodes / 12,907 edges with every endpoint/duplicate/collapse counter zero.

## Known Issues / Blockers

- No local engineering blocker remains. PR #76 and PR #77 are merged, and post-merge `main` CI/security workflows are green on the final repair baseline.
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

- PR #76/#77 require no further engineering decision; protected integration and post-merge verification are complete.
- No Desktop `0.1.1` release-content/source/publication/recovery/workflow-repair decision remains.
- A future major Dependabot update or any dependency group containing `duckdb` requires explicit compatibility review before integration.
- Owner credential retirement remains owner-only/administration-only and stays **NOT VERIFIED** unless an authorized tool confirms it.

## NEXT ACTION

1. No further action is required for the completed continuity/post-merge-repair task.
2. Start the next owner-requested task from verified repository reality rather than reopening PR #76/#77.
3. Review the existing E2E npm residual-risk policy no later than 2026-10-10 and remove exceptions immediately when supported upstream fixes exist.
4. Owner credential retirement remains a separate administration task and **NOT VERIFIED** here.

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
