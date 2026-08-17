# PROJECT STATE

## Metadata

- Last state update: 2026-08-17 14:02 +03:00, reconciled against GitHub API, GitHub Actions, GitHub Releases, and repository ruleset state.
- State author/agent: release publication and continuity reconciliation session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This file is a current summary, not a substitute for Git, GitHub Actions, GitHub Releases, repository administration settings, or live runtime evidence.

## Executive Status

- Overall status: **DESKTOP 0.1.1 PUBLISHED AND IMMUTABLE — WORKFLOW REPAIR PENDING**.
- Desktop `0.1.1` release ID `371168378` was published in place from the previously verified draft; no replacement release was created and the qualified source was not retargeted.
- Publication gate run `32022865218`, job `95366007733`, completed **success** after rechecking live repository state, all six artifact bytes/checksums, preserved and online build provenance, the repository immutable-releases administration setting, release identity, immutable release attestation, per-asset release verification, and post-publication build attestations.
- Public release `desktop-v0.1.1` is now `draft=false`, `prerelease=false`, `immutable=true`, targets `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, and is the latest public release.
- Remaining release-plan work: repair the known post-create draft verification bug in `.github/workflows/desktop-release-draft.yml`, exact-head qualify the repair PR, merge by squash, then close the active release plan.

## Repository State

- Default integration branch: `main`.
- Last verified live `main`: `ba53796a5933cdf97b6b810c208050686a8555e6` (`docs: reconcile desktop 0.1.1 recovery state (#57)`).
- Qualified Desktop `0.1.1` source and published tag target: `b911eb44619f8eab706bc2fe786d1c84ac958f1d` (`release: prepare desktop 0.1.1 (#50)`).
- Current checked-out branch: **VERIFY DYNAMICALLY** with `git branch --show-current` in any actual checkout.
- Current HEAD: derive dynamically with `git rev-parse HEAD`; do not treat the last verified remote `main` above as permanent truth.
- Working tree in the dedicated server workspace: **NOT VERIFIED** from this agent environment; inspect `git status --short --branch` before local mutation and preserve unrelated work.
- Current open PR snapshot: Dependabot #51, #52, #53, #54, and #55; query dynamically before decisions.
- Active main ruleset: ID `20627285`, `main-rebootstrap-baseline`, enforcement `active`, squash-only PR integration, unresolved review threads required, no bypass actors, and six required PR check contexts.

## Uncommitted Work

- Dedicated server checkout/worktree state: **NOT VERIFIED**.
- Temporary remote operations branch `ops/release-011-recovery-gate-20260817` contains release-recovery/publication evidence and must not be merged as application history.
- The short-lived repository Actions secret used for the publication gate must be removed after release operations; never record its value in repository state or logs.

## Desired State

- Preserve the official Kaspa runtime ownership model and zero-fake-log invariant.
- Keep `mainnet` and `testnet10` stable and supported; keep `testnet12` experimental and explicit opt-in.
- Keep Desktop `0.1.1` immutable and bound to qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` with its six verified release assets unchanged.
- Repair `.github/workflows/desktop-release-draft.yml` so draft creation/verification binds to the created release ID or draft-inclusive list selection rather than assuming tag lookup/ref availability before publication.
- Defer git tag/ref identity checks to publication/post-publication verification, where the public tag actually exists.
- Complete the workflow repair through a small PR, exact-head required checks, squash merge, and final continuity reconciliation.

## Actual State

### Repository

- Live `main` last verified at `ba53796a5933cdf97b6b810c208050686a8555e6`.
- Desktop package metadata remains `0.1.1`.
- Rust workspace policy remains Rust `1.97.1`, edition `2024`.
- Desktop Node engine remains `>=24 <27`; blocking CI uses Node `24.19.0` and npm `11.17.0`.
- Dependabot #51-#55 remain outside the release-critical workflow-repair task.

### CI / Release Qualification

- Qualified Desktop Artifacts run `31910163486` succeeded on exact source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Original draft staging run `31911079027` failed only after draft creation/upload succeeded; the failure is the known post-create draft lookup/tag-ref semantics bug.
- Independent recovery verifier run `32019926018`, job `95357274069`, proved draft identity, six-asset byte integrity, strict checksum verification, and build provenance before publication.
- Exact publication gate run `32022865218`, job `95366007733`, completed **success** through all ten job steps.
- Required PR contexts remain `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, `dependency vulnerability and license review`, `policy + audit + deny + machete`, and `Rust security-extended analysis`.

### Release Distribution

- Latest public release: ID `371168378`, tag `desktop-v0.1.1`, target `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, `draft=false`, `prerelease=false`, `immutable=true`.
- Published at: `2026-08-17T11:01:16Z` (`2026-08-17 14:01:16 +03:00`).
- Git ref `refs/tags/desktop-v0.1.1` resolves directly to commit `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Six published assets are unchanged from the independently verified draft:
  - macOS provenance bundle SHA-256 `821b58d56bc894d43ee8f1b9962ff9394f09f0476d43ed86843311b567104b0d`;
  - macOS App ZIP SHA-256 `82195836a67e143bf5b4d085bf42b7491d5332b05ac19cb79980df39aa5a8e9f`;
  - macOS DMG SHA-256 `ba5150b92860ca316e85e5fa90d27c60f2e15b0a7f0c6beed8ac970eb18b6387`;
  - Windows provenance bundle SHA-256 `9b6dc872f784f99841ecf25e4151250f7f5688677b51a72ba289cf133ae6dfa8`;
  - Windows NSIS installer SHA-256 `eff90f265fe782fb186f5d79950a2b0c42ff463ad36fcdf2675526a4d4133082`;
  - `SHA256SUMS.txt` SHA-256 `c6e4dd07f91b721e37d69b778a640e9af3fc55e25d1f2ad33ca4635ac1087624`.
- `gh release verify` and `gh release verify-asset` passed during the publication gate for the immutable release and downloaded assets.
- Windows installer remains unsigned / Authenticode not configured.
- macOS artifacts remain ad-hoc signed; Developer ID and notarization are not configured.

### Provenance

- Windows NSIS, macOS DMG, and macOS App ZIP passed preserved Sigstore bundle verification and online GitHub attestation verification both before and after publication.
- Verification enforced repository `KaspaPulse/kaspa-gateway-rust`, signer workflow `.github/workflows/desktop-artifacts.yml`, source digest `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, source ref `refs/heads/main`, SLSA provenance v1, and self-hosted-runner denial.

### Runtime

- Kaspa Gateway remains a local-first desktop control plane; no production service or unrelated runtime was started/stopped during release publication.
- Live node/bridge runtime state is **NOT VERIFIED** and must not be inferred from CI, mocks, release metadata, or artifact publication.
- `testnet12` live smoke was not run and remains experimental explicit opt-in.

## Drift

- Qualified source ↔ published tag: **NONE**; `desktop-v0.1.1` resolves to exact `b911eb4...`.
- Qualified source ↔ published release assets: **NONE FOUND**; all six asset IDs/sizes/digests remained exact through publication verification.
- Qualified source ↔ provenance: **NONE FOUND**; pre- and post-publication build attestation checks passed.
- Desired `0.1.1` ↔ public release: **NONE**; release is published, latest, and immutable.
- Repository automation ↔ desired draft semantics: **DRIFT PRESENT**; `.github/workflows/desktop-release-draft.yml` still contains the known post-create verification bug and is the active repair target.
- Runtime drift: **NOT VERIFIED**.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane around official Kaspa node and Stratum bridge runtimes. It does not reimplement consensus or official runtime behavior. Raw runtime log panes must contain native official stdout/stderr only.

## Important Paths

- `AGENTS.md` — durable workflow rules.
- `PROJECT_STATE.md` — current verified handoff.
- `PLANS.md` — active release/workflow-repair plan.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `.github/workflows/desktop-artifacts.yml` — qualified artifact/provenance workflow.
- `.github/workflows/desktop-release-draft.yml` — active workflow-repair target.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Desktop `0.1.1` metadata and release source qualified at `b911eb4...`.
- Windows/macOS native artifacts and build provenance qualified by run `31910163486`.
- Existing draft `371168378` independently recovered and byte/provenance verified.
- Fresh immutable-releases administration gate passed immediately before publication.
- Existing release ID `371168378` was published in place; no duplicate or retargeted release was created.
- `desktop-v0.1.1` is latest, immutable, and tag-bound to exact `b911eb4...`.
- Immutable release attestation, per-asset release verification, and post-publication build provenance all passed in run `32022865218`.

## Known Issues / Blockers

- **Draft workflow post-create verification bug — ACTIVE REPAIR ITEM.** `.github/workflows/desktop-release-draft.yml` assumes draft lookup/ref behavior that is not valid before publication. Repair must use release ID or draft-inclusive list semantics and defer tag-ref identity until publication.
- RustSec managed advisory set remains accepted/managed under current policy; do not claim advisories are eliminated.
- Dependabot #51-#55 remain separate dependency proposals.

## Risks

- Attempting to mutate immutable `desktop-v0.1.1` would violate the completed release boundary.
- Reintroducing tag-ref checks into a draft-only staging workflow would recreate the known failure mode.
- Treating Sigstore/SLSA provenance as Windows Authenticode or Apple Developer ID/notarization would misstate trust guarantees.
- Treating CI as live runtime proof would violate runtime safety policy.
- Leaving the short-lived publication credential configured longer than necessary increases credential exposure surface; remove it after release operations.

## Constraints / Invariants

- Preserve official runtime ownership and official runtime bindings.
- Preserve zero-fake-log policy.
- `mainnet` and `testnet10` remain stable; `testnet12` remains experimental explicit opt-in.
- Do not use destructive Git cleanup or discard user work.
- Use PR-based integration and exact-head qualification for material repository changes.
- Do not mutate immutable `desktop-v0.1.0` or `desktop-v0.1.1`.
- Never record credentials, tokens, private keys, cookies, or secret values in project state.

## DO NOT CHANGE WITHOUT EXPLICIT REASON

- Runtime ownership model and official runtime bindings.
- Raw log invariant.
- Network stability classification.
- Immutable `desktop-v0.1.0` and `desktop-v0.1.1` releases.
- Published Desktop `0.1.1` source/tag binding to `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.

## Pending Decisions

- No release-content/source/publication decision remains for Desktop `0.1.1`; publication is complete and immutable.
- Dependabot #51-#55 remain outside the current workflow-repair plan and require independent review.

## NEXT ACTION

1. Reinspect current `.github/workflows/desktop-release-draft.yml` and related repository-native gates on fresh `main`.
2. Implement the smallest repair that captures/discovers the created draft by release ID or draft-inclusive list selection, verifies draft/tag/target/assets against that release object, and removes any requirement for a real git tag/ref before publication.
3. Add or extend a focused repository-native contract gate only if needed to prevent regression of these draft semantics.
4. Open one PR for the repair; do not create a competing PR.
5. Run all six required ruleset contexts on the exact final head and classify any baseline/upstream failures precisely.
6. Squash-merge only if the exact head remains unchanged, all required checks pass, and no unresolved review threads remain.
7. Reconcile `PROJECT_STATE.md` and `PLANS.md` after the repair merge and close the Desktop `0.1.1` release plan.
8. Remove the short-lived publication repository secret through GitHub repository settings after release operations; do not expose its value.

## Resume Instructions

1. Read `AGENTS.md` first.
2. Read `PROJECT_STATE.md` second.
3. Inspect actual Git branch, HEAD, working tree, live GitHub `main`, open PRs, checks, ruleset, and releases; do not assume this file is current.
4. If verified reality differs, reconcile this document before relying on stale claims.
5. Read `PLANS.md` because the workflow-repair milestone remains active.
6. Read `docs/runbooks/desktop-release.md` only if further release operations are required; immutable `0.1.1` must not be mutated.
7. Read only directly relevant ADRs for architectural/process decisions.
8. Treat old chats and handoff archives as advisory only.
9. Do not repeat completed release recovery/publication verification unless fresh evidence shows regression.
10. Continue from `NEXT ACTION` unless the owner explicitly changes priority.
