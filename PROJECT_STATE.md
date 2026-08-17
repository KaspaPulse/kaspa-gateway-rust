# PROJECT STATE

## Metadata

- Last state update: 2026-08-17 12:31 +03:00, reconciled against GitHub API state.
- State author/agent: repository continuity reconciliation session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This file is a current summary, not a substitute for Git, GitHub Actions, GitHub Releases, or live runtime evidence.

## Executive Status

- Overall status: **ACTIVE RELEASE PREPARATION**.
- Current objective: independently recover/verify the existing Desktop `0.1.1` draft release, then publish that exact draft only after the release gate passes and explicit publication authorization is given.
- Current blocker: the draft creation workflow has a post-create verification bug; artifact creation and upload already succeeded. Publication remains gated on independent draft recovery verification and a fresh immutable-releases administration check.

## Repository State

- Default integration branch: `main`.
- Current checked-out branch: **VERIFY DYNAMICALLY** with `git branch --show-current`; do not persist a session branch here as permanent truth.
- Working tree: **NOT VERIFIED** in this reconciliation because the agent environment had GitHub API access but could not create a live local clone. Every local session must run `git status --short --branch` before edits.
- Last verified implementation baseline: `b911eb44619f8eab706bc2fe786d1c84ac958f1d` (`release: prepare desktop 0.1.1 (#50)`).
- Initial continuity-reconciliation baseline: `main` was `b911eb44619f8eab706bc2fe786d1c84ac958f1d` on 2026-08-17. Treat this as the verified implementation baseline, not a perpetual current-HEAD claim.
- Current HEAD: derive dynamically with `git rev-parse HEAD`; do not assume the baseline above is forever the current HEAD.
- Initial open-PR snapshot: #51, #52, #53, #54, #55; all were Dependabot update PRs. Current open PRs must be queried dynamically because this list changes independently of state-document commits.
- Active main ruleset: ID `20627285`, `main-rebootstrap-baseline`, enforcement `active`, squash-only PR integration, unresolved review threads required, no bypass actors.

## Uncommitted Work

- Local worktree state: **NOT VERIFIED**.
- Required action before any local mutation: inspect branch, HEAD, and `git status --short --branch`; preserve unrelated user work.

## Desired State

- Keep `mainnet` and `testnet10` as stable supported networks.
- Keep `testnet12` experimental and explicit opt-in.
- Preserve the official runtime ownership model and zero-fake-log invariant.
- Publish Desktop `0.1.1` from the already-qualified source commit `b911eb44619f8eab706bc2fe786d1c84ac958f1d` by using the existing draft release ID `371168378`, after recovery verification passes.
- After safe publication, fix the draft workflow verification logic so draft verification uses release ID/list selection instead of assuming tag lookup/ref availability before publication.
- Maintain repository-native continuity through this file, `AGENTS.md`, relevant ADRs, runbooks, and `PLANS.md`.

## Actual State

### Repository

- Current `main` HEAD: **VERIFY DYNAMICALLY**. The last verified implementation baseline before continuity-only changes was `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Desktop package metadata: `0.1.1`.
- Rust workspace toolchain policy: Rust `1.97.1`, edition `2024`.
- Desktop Node engine: `>=24 <27`; CI blocking lane uses Node `24.19.0` and npm `11.17.0`.
- Open PRs: **VERIFY DYNAMICALLY**. Initial 2026-08-17 snapshot was Dependabot #51-#55.

### CI

- Main baseline `quality (rust + npm)`: **PASS**, GitHub Actions run `31901112999`, job `95051921350`.
- Qualified Desktop Artifacts run: `31910163486`, workflow `Desktop Artifacts`, exact head `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, conclusion **success**.
- Draft staging run: `31911079027`, conclusion **failure** only after the draft and six assets were created; failure classification is the known post-create draft lookup bug.
- Main ruleset requires these PR checks: `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, `dependency vulnerability and license review`, `policy + audit + deny + machete`, and `Rust security-extended analysis`.
- A main-branch commit does not necessarily have every PR-only required check rerun; verify the exact PR head before merge.

### Release Distribution

- Latest published release: ID `371031676`, tag `desktop-v0.1.0`, target `91046d16fa20a0b9a2b2b59ec9cac4f1db2594f4`, `draft=false`, `immutable=true`.
- Existing Desktop `0.1.1` draft: ID `371168378`, `tag_name=desktop-v0.1.1`, target `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, `draft=true`, `published_at=null`, six non-empty assets.
- Draft product digests observed in GitHub metadata match the previously qualified Windows installer, macOS DMG, and macOS App ZIP hashes.
- Windows `0.1.1` artifact: build provenance verified previously; Authenticode is **not configured / unsigned**.
- macOS `0.1.1` artifacts: build provenance verified previously; signing is **ad-hoc**, Developer ID/notarization are not configured.

### Staging

- No hosted staging deployment is defined by the verified repository state.
- The GitHub draft release is a release-staging object, not evidence of a running staging environment.

### Production / Runtime

- Kaspa Gateway is a local-first desktop control plane, not a centrally deployed hosted service in the verified repository state.
- Public release availability is represented by GitHub Releases; end-user runtime health on installed desktops is not centrally observable here.
- Live node/bridge runtime state: **NOT VERIFIED** and must never be inferred from mocks, CI, or release metadata.
- No production service or unrelated runtime was started/stopped during this reconciliation.

## Drift

- Repository ↔ qualified Desktop Artifacts: **NONE for release source `b911eb4...`**; run `31910163486` is tied to that exact SHA.
- Repository ↔ latest published release: **PRESENT, EXPECTED**; `main` contains `0.1.1` release metadata while the latest public immutable release remains `0.1.0`.
- Desired `0.1.1` ↔ existing draft: **NO KNOWN METADATA DRIFT**; draft ID `371168378` targets the qualified `b911eb4...` source and contains the intended six assets.
- Desired `0.1.1` ↔ public release: **PENDING** until the existing draft is independently verified and explicitly published.
- Runtime drift: **NOT VERIFIED** because no live local runtime was inspected.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane. It owns configuration, process lifecycle, status, and observability around official Kaspa node and Stratum bridge runtimes; it does not reimplement Kaspa consensus or official runtime behavior. See `docs/architecture/README.md` and the relevant ADRs before changing runtime ownership.

## Important Paths

- `AGENTS.md` — durable agent/workflow rules.
- `PROJECT_STATE.md` — canonical current-state handoff.
- `PLANS.md` — active multi-stage execution plan.
- `docs/architecture/` — current architecture/contracts and legacy ADR-0010.
- `docs/adr/` — canonical location/index for new ADRs.
- `docs/runbooks/` — repeatable operational procedures.
- `docs/operations/live-network-smoke.md` — existing live-network smoke procedure.
- `.github/workflows/desktop-artifacts.yml` — qualified desktop artifact build/provenance workflow.
- `.github/workflows/desktop-release-draft.yml` — draft staging workflow; contains the known post-create verification bug.
- `config/runtime-repository-bindings.json` — official runtime repository/revision bindings.

## Services / Components

- Desktop Tauri shell and frontend.
- Per-network node runtime ownership.
- Per-network Stratum bridge runtime ownership.
- CLI/runtime probes.
- CI/security/release automation in GitHub Actions.

## Runtime / Environment

- `mainnet`: stable.
- `testnet10`: stable supported testnet.
- `testnet12`: experimental; explicit opt-in only.
- RPC must remain loopback-bound by default.
- Network data directories, ports, processes, runtime state, and raw logs must remain isolated by network.
- Exact local runtime ports and process state must be verified from active configuration/runtime state; they are intentionally not duplicated as static claims here.

## Completed and Verified

- Desktop metadata `0.1.1` is merged at baseline `b911eb4...`.
- Desktop Artifacts run `31910163486` completed successfully on exact `b911eb4...`.
- Existing draft release `371168378` still exists as a draft targeting `b911eb4...` with six assets.
- Latest public release `desktop-v0.1.0` remains immutable.
- Main ruleset `20627285` remains active with squash-only integration and the six required PR check contexts.

## Last Verified Validation

### Build / Tests

- Command source: `.github/workflows/ci.yml` blocking `quality (rust + npm)` lane.
- Result on implementation baseline `b911eb4...`: **PASS** via run `31901112999`, job `95051921350`.
- This evidence includes repository-native npm lint/gates, Rust formatting/check/Clippy/tests, and npm audits as defined by that workflow revision.

### Desktop Artifact Qualification

- Workflow: `.github/workflows/desktop-artifacts.yml`.
- Run: `31910163486`.
- Source: exact `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Result: **PASS / success** for Windows x64 NSIS and macOS universal artifact jobs.
- `testnet12` live smoke: **NOT RUN** because TN12 is experimental and requires explicit authorization.

### Live Runtime Verification

- Result: **NOT VERIFIED** in this state reconciliation.
- Reason: no actual local node/bridge runtime was started or inspected as part of documentation continuity work.

## Known Issues / Blockers

- Draft workflow post-create verification bug — HIGH for release automation correctness. The workflow attempts tag-based draft lookup before publication; the existing draft itself is not evidence of artifact failure. Next required action: recover/verify release ID `371168378`, publish only after authorization, then repair the workflow in a small PR.
- RustSec managed advisory set — accepted/managed under current policy; do not claim vulnerabilities are eliminated. Do not fork `hexplay`/`atty` or Tauri/Wry GTK dependencies solely to silence advisories without a reviewed upstream path.
- Dependabot PRs #51-#55 — active dependency update proposals; evaluate independently and do not conflate them with release or continuity work.

## Risks

- Recreating the `0.1.1` draft would create competing release state and weaken auditability.
- Advancing/retargeting the release source without requalification would break exact-head release evidence.
- Treating Sigstore/SLSA provenance as Windows Authenticode or Apple Developer ID/notarization would misstate the trust layer.
- Treating CI or mocks as live runtime proof would violate the runtime safety invariant.

## Constraints / Invariants

- Preserve official runtime ownership and official runtime bindings.
- Raw runtime panes contain official stdout/stderr only; no invented or UI-only fake runtime lines.
- `testnet12` remains experimental and explicit opt-in.
- Do not run destructive Git cleanup (`reset --hard`, `clean`, or discard user work).
- Use PR-based integration and exact-head qualification for material changes.
- Do not mutate immutable `desktop-v0.1.0`.
- Do not publish `desktop-v0.1.1` without explicit publication authorization after recovery verification.
- Never record credentials, tokens, private keys, cookies, or secret values in project state or handoff documents.

## DO NOT CHANGE WITHOUT EXPLICIT REASON

- Runtime ownership model and official runtime bindings.
- Raw log invariant.
- Network stability classification (`mainnet`, `testnet10`, experimental opt-in `testnet12`).
- Existing immutable `desktop-v0.1.0` release.
- Existing Desktop `0.1.1` draft release ID `371168378` or its release-source target unless a reviewed release decision requires it.

## Pending Decisions

- Whether and when to publish the already-qualified Desktop `0.1.1` draft after recovery verification.
- How to reconcile Dependabot PRs #51-#55 after the release-critical path is safe.

## NEXT ACTION

1. Run the existing Desktop `0.1.1` draft recovery verifier against release ID `371168378`; do not rerun the draft workflow and do not publish yet.
2. Verify all six downloaded draft assets against GitHub asset digests and `SHA256SUMS.txt`.
3. Verify the Windows installer and macOS DMG/App ZIP against the attached Sigstore bundles and online GitHub attestations, enforcing the exact repository, signer workflow, source digest `b911eb4...`, source ref `refs/heads/main`, SLSA v1 predicate, and self-hosted-runner denial.
4. Reassert `draft=true`, `published_at=null`, and latest public release `desktop-v0.1.0` after verification.
5. If recovery verification passes, re-check immutable releases administration state and current GitHub freshness.
6. Publish the existing draft only if the user explicitly authorizes publication; do not create a second release or retarget the draft.
7. After safe publication, fix `.github/workflows/desktop-release-draft.yml` so draft verification uses release ID/list selection and verifies tag ref only after publication.
8. Reconcile this file after each meaningful release-state transition.

## Resume Instructions

Resume protocol for a new ChatGPT/Codex session:

1. Read `AGENTS.md` first.
2. Read `PROJECT_STATE.md` second.
3. Inspect Git branch, HEAD, and working tree; do not assume this file is current.
4. If verified reality differs from this file, reconcile the document before relying on it.
5. Read only ADRs relevant to the task.
6. Read the relevant runbook before release publication, rollback/recovery, live-network smoke, or incident actions.
7. Read `PLANS.md` when the active work is multi-stage.
8. Treat old conversation memory and external handoff bundles as non-authoritative context.
9. Do not repeat completed work unless verification shows it is incomplete or regressed.
10. Continue from `NEXT ACTION` unless the user explicitly changes the goal.
