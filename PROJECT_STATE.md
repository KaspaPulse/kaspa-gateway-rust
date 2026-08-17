# PROJECT STATE

## Metadata

- Last state update: 2026-08-17 13:28 +03:00, reconciled against GitHub API, GitHub Actions, and GitHub Releases.
- State author/agent: repository continuity reconciliation session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This file is a current summary, not a substitute for Git, GitHub Actions, GitHub Releases, repository administration settings, or live runtime evidence.

## Executive Status

- Overall status: **ACTIVE RELEASE PREPARATION — RECOVERY VERIFIED / PUBLICATION ADMIN GATE BLOCKED**.
- Desktop `0.1.1` draft recovery verification is complete for release ID `371168378`: all six draft assets were downloaded by release asset ID, matched GitHub SHA-256 metadata, passed strict `SHA256SUMS.txt` verification, and the three product artifacts passed preserved-bundle and online GitHub attestation verification with the required source/workflow/SLSA constraints.
- Publication authorization has been explicitly granted by the repository owner in the active release session, but publication must not occur until the repository immutable-releases administration state is freshly verified.
- Current blocker: available GitHub App / `GITHUB_TOKEN` credentials can read/write release contents but cannot read `GET /repos/KaspaPulse/kaspa-gateway-rust/immutable-releases`; GitHub returns `403 Resource not accessible by integration`. No repository release-administration PAT/App credential was found under the probed standard secret names.

## Repository State

- Default integration branch: `main`.
- Last verified live `main`: `f07b29be804ab39a3b372d27683900af55b52684` (`docs: establish canonical project continuity system (#56)`).
- Qualified Desktop `0.1.1` source: `b911eb44619f8eab706bc2fe786d1c84ac958f1d` (`release: prepare desktop 0.1.1 (#50)`).
- `f07b29b...` is exactly one continuity commit ahead of `b911eb4...`; release source remains intentionally pinned to the qualified `b911eb4...` commit and must not be silently retargeted.
- Current checked-out branch: **VERIFY DYNAMICALLY** with `git branch --show-current` in any actual checkout.
- Current HEAD: derive dynamically with `git rev-parse HEAD`; do not treat the last verified remote `main` above as permanent truth.
- Working tree: **NOT VERIFIED** in the dedicated server workspace from this agent environment; before local mutation inspect `git status --short --branch` and preserve unrelated work.
- Current open PR snapshot: Dependabot #51, #52, #53, #54, and #55 only; query dynamically before decisions.
- Active main ruleset: ID `20627285`, `main-rebootstrap-baseline`, enforcement `active`, squash-only PR integration, unresolved review threads required, no bypass actors.

## Uncommitted Work

- Dedicated server checkout/worktree state: **NOT VERIFIED** from this environment.
- Temporary remote operational evidence branch created for recovery verification: `ops/release-011-recovery-gate-20260817`. It is not an integration branch and must not be merged as application history.
- State reconciliation branch: `agent/reconcile-release-011-recovery`.

## Desired State

- Preserve the official Kaspa runtime ownership model and zero-fake-log invariant.
- Keep `mainnet` and `testnet10` stable and supported; keep `testnet12` experimental and explicit opt-in.
- Publish the existing Desktop `0.1.1` draft release ID `371168378` from qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` only after a fresh successful immutable-releases administration read.
- Never recreate, replace, or retarget the verified draft merely to bypass publication permissions.
- After safe publication and post-publication verification, repair `.github/workflows/desktop-release-draft.yml` so draft discovery/verification uses release ID/list semantics and tag-ref identity is checked only after publication.

## Actual State

### Repository

- Live `main` last verified at `f07b29be804ab39a3b372d27683900af55b52684`.
- Desktop package metadata remains `0.1.1`.
- Rust workspace policy remains Rust `1.97.1`, edition `2024`.
- Desktop Node engine remains `>=24 <27`; blocking CI uses Node `24.19.0` and npm `11.17.0`.
- Dependabot #51-#55 remain outside the release-critical path.

### CI

- Current `main` check snapshot on `f07b29b...`: visible main checks completed successfully, including `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, and `Rust security-extended analysis`.
- Qualified Desktop Artifacts run: `31910163486`, exact head `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, conclusion **success**.
- Original draft staging run: `31911079027`, conclusion **failure** only after draft creation/upload succeeded; the failure is the known post-create draft tag-lookup/ref bug.
- Independent recovery verifier run: `32019926018`, job `95357274069`. Draft identity, six-asset byte verification, strict checksum manifest verification, and provenance verification **PASSED**. The job conclusion is `failure` solely because the immutable-releases administration endpoint returned HTTP 403 for the GitHub Actions token.
- Required PR contexts remain: `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, `dependency vulnerability and license review`, `policy + audit + deny + machete`, and `Rust security-extended analysis`.

### Release Distribution

- Latest public release remains ID `371031676`, tag `desktop-v0.1.0`, target `91046d16fa20a0b9a2b2b59ec9cac4f1db2594f4`, `draft=false`, `immutable=true`.
- Existing Desktop `0.1.1` draft remains ID `371168378`, tag `desktop-v0.1.1`, target `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, `draft=true`, `published_at=null`, six non-empty assets.
- Verified draft asset SHA-256 values:
  - macOS provenance bundle: `821b58d56bc894d43ee8f1b9962ff9394f09f0476d43ed86843311b567104b0d`;
  - macOS App ZIP: `82195836a67e143bf5b4d085bf42b7491d5332b05ac19cb79980df39aa5a8e9f`;
  - macOS DMG: `ba5150b92860ca316e85e5fa90d27c60f2e15b0a7f0c6beed8ac970eb18b6387`;
  - Windows provenance bundle: `9b6dc872f784f99841ecf25e4151250f7f5688677b51a72ba289cf133ae6dfa8`;
  - Windows NSIS installer: `eff90f265fe782fb186f5d79950a2b0c42ff463ad36fcdf2675526a4d4133082`;
  - `SHA256SUMS.txt`: `c6e4dd07f91b721e37d69b778a640e9af3fc55e25d1f2ad33ca4635ac1087624`.
- Strict `SHA256SUMS.txt` verification passed for all five listed product/provenance files.
- Windows installer remains unsigned / Authenticode not configured.
- macOS artifacts remain ad-hoc signed; Developer ID and notarization are not configured.

### Provenance

- Windows NSIS, macOS DMG, and macOS App ZIP each passed verification against the preserved Sigstore bundle and the online GitHub attestation store.
- Verification enforced repository `KaspaPulse/kaspa-gateway-rust`, signer workflow `.github/workflows/desktop-artifacts.yml`, source digest `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, source ref `refs/heads/main`, SLSA provenance v1, and self-hosted-runner denial.

### Staging / Runtime

- GitHub draft release is a release-staging object, not a hosted staging deployment.
- Kaspa Gateway remains a local-first desktop control plane; no production service was started/stopped during this work.
- Live node/bridge runtime state is **NOT VERIFIED** and must not be inferred from CI, mocks, or release metadata.
- `testnet12` live smoke was not run and remains explicit opt-in.

## Drift

- Qualified source ↔ Desktop Artifacts: **NONE**; run `31910163486` is exact-head bound to `b911eb4...`.
- Qualified source ↔ verified draft bytes: **NONE**; all six downloaded draft assets match GitHub digests and the expected checksum manifest.
- Qualified source ↔ provenance: **NONE FOUND**; preserved and online attestations passed the strict identity constraints.
- Current `main` ↔ qualified release source: **PRESENT, CLASSIFIED**; `main` is one continuity commit ahead. The release remains intentionally pinned to `b911eb4...` and must not be retargeted without requalification.
- Desired `0.1.1` ↔ public release: **PENDING**; public latest remains immutable `0.1.0` because the immutable-releases administration gate has not been readable with available credentials.
- Runtime drift: **NOT VERIFIED**.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane around official Kaspa node and Stratum bridge runtimes. It does not reimplement consensus or official runtime behavior. Raw runtime log panes must contain native official stdout/stderr only.

## Important Paths

- `AGENTS.md` — durable workflow rules.
- `PROJECT_STATE.md` — current verified handoff.
- `PLANS.md` — active release plan.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `.github/workflows/desktop-artifacts.yml` — qualified artifact/provenance workflow.
- `.github/workflows/desktop-release-draft.yml` — draft staging workflow with the known post-create verification bug.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Desktop `0.1.1` metadata merged at `b911eb4...`.
- Desktop Artifacts run `31910163486` passed on exact `b911eb4...`.
- Existing draft `371168378` independently recovered and byte-verified by asset ID.
- All draft asset digests and strict checksum-manifest checks passed.
- Preserved Sigstore bundles and online GitHub attestations passed for the three product artifacts.
- Post-verification re-fetch confirms draft `371168378` remains `draft=true`, `published_at=null`; latest public release remains immutable `desktop-v0.1.0`.
- Owner publication authorization is present.

## Known Issues / Blockers

- **Publication administration credential — ACTIVE BLOCKER.** Repository immutable-release state must be freshly read before publication. The connected GitHub App and GitHub Actions `GITHUB_TOKEN` receive HTTP 403 because this endpoint requires repository Administration read. A safe probe found no release-admin token/App credential under the standard repository-secret names checked. Do not downgrade this gate or infer current enablement from the older immutable `0.1.0` release.
- **Draft workflow post-create verification bug — KNOWN / NOT YET REPAIRED ON MAIN.** The workflow assumes tag-based draft lookup/ref availability before publication. Repair remains scheduled after the release-critical publication path is safely resolved.
- **Publication credential compatibility risk.** Because current `main` is ahead of the qualified source and includes a workflow-file change, publication credentials must support GitHub's current release-update permission rules without retargeting the draft.
- RustSec managed advisory set remains accepted/managed under current policy; do not claim advisories are eliminated.
- Dependabot #51-#55 remain separate work.

## Risks

- Recreating or retargeting the verified draft would weaken exact-source auditability.
- Publishing without a fresh immutable-releases administration read would waive an explicit release gate.
- Treating Sigstore/SLSA provenance as Windows Authenticode or Apple Developer ID/notarization would misstate trust guarantees.
- Treating CI as live runtime proof would violate runtime safety policy.

## Constraints / Invariants

- Preserve official runtime ownership and official runtime bindings.
- Preserve zero-fake-log policy.
- `mainnet` and `testnet10` remain stable; `testnet12` remains experimental explicit opt-in.
- Do not use destructive Git cleanup or discard user work.
- Use PR-based integration and exact-head qualification for material repository changes.
- Do not mutate immutable `desktop-v0.1.0`.
- Do not create a second `desktop-v0.1.1` release or silently retarget draft `371168378`.
- Never record credentials, tokens, private keys, cookies, or secret values in project state.

## DO NOT CHANGE WITHOUT EXPLICIT REASON

- Runtime ownership model and official runtime bindings.
- Raw log invariant.
- Network stability classification.
- Immutable `desktop-v0.1.0`.
- Existing Desktop `0.1.1` draft ID `371168378` and qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.

## Pending Decisions

- No product/release-content decision remains for recovery: recovery evidence passed and publication is authorized.
- Operational dependency: obtain/use a credential capable of the required Administration-read gate and compliant release publication without retargeting.
- Dependabot #51-#55 remain outside this release plan.

## NEXT ACTION

1. Obtain or use an authenticated GitHub credential with repository **Administration read** for `KaspaPulse/kaspa-gateway-rust`; do not expose or persist the secret value in repository files/logs.
2. Immediately re-check live `main`, open PRs, ruleset, draft ID `371168378`, exact six assets, and `GET /repos/KaspaPulse/kaspa-gateway-rust/immutable-releases`; require `enabled=true`.
3. With the same release state still exact and owner authorization already present, publish **existing draft ID `371168378` only**. Do not create a replacement release and do not retarget from `b911eb4...`.
4. Immediately perform post-publication verification: `draft=false`, `published_at` populated, `immutable=true`, tag/ref resolves to `b911eb4...`, six asset digests unchanged, native immutable release/asset verification where supported, and GitHub attestation verification still passing.
5. Reconcile this file after publication.
6. Only after safe publication, repair `.github/workflows/desktop-release-draft.yml` in a small PR so draft preflight/discovery includes drafts, verification binds to release ID/list selection, and git tag/ref identity is deferred until publication.
7. Qualify the workflow-repair PR on its exact head with all required ruleset checks before squash merge.

## Resume Instructions

1. Read `AGENTS.md` first.
2. Read `PROJECT_STATE.md` second.
3. Inspect actual Git branch, HEAD, working tree, live GitHub `main`, open PRs, checks, ruleset, and releases; do not assume this file is current.
4. If verified reality differs, reconcile this document before relying on stale claims.
5. Read `PLANS.md` because the release plan remains active.
6. Read `docs/runbooks/desktop-release.md` before release publication or recovery actions.
7. Read only directly relevant ADRs for architectural/process decisions.
8. Treat old chats and handoff archives as advisory only.
9. Do not repeat the completed draft-byte/provenance recovery unless fresh evidence shows regression.
10. Continue from `NEXT ACTION` unless the owner explicitly changes priority.
