# PROJECT STATE

## Metadata

- Last state update: 2026-08-17 14:27 +03:00, reconciled against live GitHub `main`, GitHub Actions, GitHub Releases, tag refs, open pull requests, and repository ruleset state.
- State author/agent: Desktop `0.1.1` release-cycle closure session.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- This file is a current summary, not a substitute for Git, GitHub Actions, GitHub Releases, repository settings, or live runtime evidence.

## Executive Status

- Overall status: **DESKTOP 0.1.1 RELEASE CYCLE COMPLETE — MANUAL CREDENTIAL RETIREMENT PENDING**.
- Desktop `0.1.1` release ID `371168378` is public, latest, `draft=false`, `prerelease=false`, and `immutable=true`; tag `desktop-v0.1.1` resolves directly to qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Exact publication gate run `32022865218`, job `95366007733`, completed **success** after independent asset/checksum/provenance verification, a fresh immutable-releases administration gate, exact-release publication, immutable release/asset verification, and post-publication build-attestation verification.
- The post-create draft verification bug was repaired in PR #59 and squash-merged as `197bdead257973164931b34cce20c4556820df44`. The repaired workflow now discovers drafts through the draft-inclusive release list, carries the created numeric release ID forward, verifies `/releases/{release_id}`, and does not require a public tag/ref after draft creation.
- Blocking CI now includes `tools/kgw_desktop_release_draft_workflow_gate.cjs`, which guards those draft semantics against regression.
- No Desktop `0.1.1` release, artifact-integrity, provenance, immutability, tag-binding, or workflow-repair blocker remains.
- Remaining owner-only security hygiene: remove repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the short-lived fine-grained PAT that supplied it. Do not expose the token value in chat, repository files, or logs.

## Repository State

- Default integration branch: `main`.
- Last verified live `main`: `197bdead257973164931b34cce20c4556820df44` (`fix: verify desktop release drafts by release ID (#59)`).
- Qualified Desktop `0.1.1` source and published tag target: `b911eb44619f8eab706bc2fe786d1c84ac958f1d` (`release: prepare desktop 0.1.1 (#50)`).
- Current checked-out branch: **VERIFY DYNAMICALLY** with `git branch --show-current` in any actual checkout.
- Current HEAD: derive dynamically with `git rev-parse HEAD`; do not treat the last verified remote `main` above as permanent truth.
- Dedicated server working tree: **NOT VERIFIED** from this agent environment; inspect `git status --short --branch` before local mutation and preserve unrelated work.
- Current open PR snapshot: Dependabot #51, #52, #53, #54, and #55; query dynamically before decisions.
- Active main ruleset: ID `20627285`, `main-rebootstrap-baseline`, enforcement `active`, squash-only PR integration, unresolved review threads required, no bypass actors, and six required PR check contexts.

## Uncommitted Work

- Dedicated server checkout/worktree state: **NOT VERIFIED**.
- Temporary remote operations branch `ops/release-011-recovery-gate-20260817` remains evidence-only and must not be merged as application history.
- The exact publication workflow was retired from that operations branch after successful publication, so it no longer provides a workflow path that can consume the publication credential.
- Repository secret/PAT retirement is an external GitHub-settings action and is not represented as uncommitted repository code.

## Desired State

- Preserve the official Kaspa runtime ownership model and zero-fake-log invariant.
- Keep `mainnet` and `testnet10` stable and supported; keep `testnet12` experimental and explicit opt-in.
- Keep immutable Desktop `0.1.1` bound to source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` with the verified six-asset release set unchanged.
- Preserve the repaired draft semantics: draft-inclusive list discovery, unique draft resolution by tag/target, numeric release-ID propagation, and ID-based post-create verification.
- Keep public tag/ref identity verification in publication/post-publication handling rather than draft-only staging.
- Continue normal maintenance through small PRs and exact-head required checks; dependency updates remain independent work.

## Actual State

### Repository

- Live `main` last verified at `197bdead257973164931b34cce20c4556820df44`.
- Desktop package metadata remains `0.1.1`.
- Rust workspace policy remains Rust `1.97.1`, edition `2024`.
- Desktop Node engine remains `>=24 <27`; blocking CI uses Node `24.19.0` and npm `11.17.0`.
- Dependabot #51-#55 remain open proposals outside the completed Desktop `0.1.1` release cycle.

### CI / Release Qualification

- Qualified Desktop Artifacts run `31910163486` succeeded on exact source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Independent recovery verifier run `32019926018`, job `95357274069`, proved draft identity, six-asset byte integrity, strict checksum verification, and build provenance before publication.
- Exact publication gate run `32022865218`, job `95366007733`, completed **success** through publication and all post-publication verification steps.
- Workflow repair PR #59 final head `16426e3062878a7d2ccc271a9bc1aeeb06b7e3cd` passed all six required ruleset contexts before squash merge: `quality (rust + npm)`, `actionlint`, `TruffleHog verified and unknown secrets`, `dependency vulnerability and license review`, `policy + audit + deny + machete`, and `Rust security-extended analysis`.
- The blocking `quality (rust + npm)` lane on that exact head passed desktop/E2E lint, existing settings/runtime/artifact gates, the new desktop release-draft workflow contract, project continuity, formatting, `cargo check`, strict Clippy, Rust tests, and both npm audits.
- Graphify policy was read in full. Graphify execution was **environment-limited** in this agent sandbox because the dedicated server checkout was not mounted, Graphify was unavailable, and sandbox DNS could not clone the public repository. No Graphify PASS is claimed and no stale/generated Graphify artifacts were added.

### Draft Release Automation

- `.github/workflows/desktop-release-draft.yml` is repaired on `main`.
- Preflight uses authenticated paginated `List releases` semantics to detect an existing release including drafts while retaining the public git-tag collision check.
- The create step resolves the new draft uniquely by `draft=true`, exact `tag_name`, and exact `target_commitish`, validates a numeric release ID, and exports it through `GITHUB_OUTPUT`.
- Post-create verification reads `repos/$GITHUB_REPOSITORY/releases/$DRAFT_RELEASE_ID` and verifies the release ID, draft state, null `published_at`, prerelease state, exact tag/target, asset count/names/sizes, and SHA-256 digests.
- Post-create draft verification no longer calls the published-release-by-tag endpoint and no longer requires a real git tag/ref.
- `tools/kgw_desktop_release_draft_workflow_gate.cjs` enforces these semantics in blocking CI.

### Release Distribution

- Latest public release: ID `371168378`, tag `desktop-v0.1.1`, target `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, `draft=false`, `prerelease=false`, `immutable=true`.
- Published at `2026-08-17T11:01:16Z` (`2026-08-17 14:01:16 +03:00`).
- Git ref `refs/tags/desktop-v0.1.1` resolves directly to commit `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Six published assets remain unchanged from the independently verified draft. Key product digests remain Windows NSIS `eff90f265fe782fb186f5d79950a2b0c42ff463ad36fcdf2675526a4d4133082`, macOS DMG `ba5150b92860ca316e85e5fa90d27c60f2e15b0a7f0c6beed8ac970eb18b6387`, and macOS App ZIP `82195836a67e143bf5b4d085bf42b7491d5332b05ac19cb79980df39aa5a8e9f`.
- `gh release verify`, all release-asset verifications, and post-publication build attestations passed in the publication gate.
- Windows installer remains unsigned / Authenticode not configured.
- macOS artifacts remain ad-hoc signed; Developer ID and notarization are not configured.

### Runtime

- Kaspa Gateway remains a local-first desktop control plane; no production service or unrelated runtime was started/stopped during this release-cycle work.
- Live node/bridge runtime state is **NOT VERIFIED** and must not be inferred from CI, mocks, release metadata, or artifact publication.
- `testnet12` live smoke was not run and remains experimental explicit opt-in.

## Drift

- Qualified source ↔ published tag: **NONE**; `desktop-v0.1.1` resolves to exact `b911eb4...`.
- Qualified source ↔ published release assets/provenance: **NONE FOUND**; publication and post-publication verification passed.
- Desired `0.1.1` ↔ public release: **NONE**; release is latest and immutable.
- Repository automation ↔ desired draft semantics: **NONE**; PR #59 repaired the prior drift and blocking CI now guards it.
- Repository continuity ↔ live release/workflow state: **NONE after this reconciliation**.
- Runtime drift: **NOT VERIFIED**.

## Current Architecture

Kaspa Gateway is a local-first Rust/Tauri desktop control plane around official Kaspa node and Stratum bridge runtimes. It does not reimplement Kaspa consensus or official runtime behavior. Raw runtime log panes must contain native official stdout/stderr only.

## Important Paths

- `AGENTS.md` — durable workflow rules.
- `PROJECT_STATE.md` — current verified handoff.
- `PLANS.md` — completed Desktop `0.1.1` release-cycle plan and evidence.
- `docs/runbooks/desktop-release.md` — release qualification/recovery/publication procedure.
- `.github/workflows/desktop-artifacts.yml` — qualified artifact/provenance workflow.
- `.github/workflows/desktop-release-draft.yml` — repaired draft staging workflow.
- `tools/kgw_desktop_release_draft_workflow_gate.cjs` — blocking regression guard for draft semantics.
- `config/runtime-repository-bindings.json` — official runtime bindings.

## Completed and Verified

- Desktop `0.1.1` metadata/source qualified at `b911eb4...`.
- Windows/macOS native artifacts and build provenance qualified by run `31910163486`.
- Draft `371168378` independently recovered and byte/checksum/provenance verified.
- Fresh immutable-releases administration gate passed immediately before publication.
- Existing release ID `371168378` was published in place; no duplicate or retargeted release was created.
- `desktop-v0.1.1` is latest, immutable, and tag-bound to exact `b911eb4...`.
- Immutable release/asset verification and post-publication build provenance passed in run `32022865218`.
- Publication-state continuity PR #58 was exact-head qualified and squash-merged.
- Draft-workflow repair PR #59 was exact-head qualified and squash-merged as `197bdead...`.
- The repaired semantics are covered by a dedicated blocking CI contract gate.
- Temporary exact-publication workflow on the evidence branch was retired after successful use.

## Known Issues / Blockers

- **No active Desktop `0.1.1` release or workflow-repair blocker remains.**
- Manual credential retirement remains immediate security hygiene: delete repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the associated short-lived fine-grained PAT in GitHub settings.
- RustSec managed advisory set remains accepted/managed under current policy; do not claim advisories are eliminated.
- Dependabot #51-#55 remain separate dependency proposals and require independent review.

## Risks

- Attempting to mutate immutable `desktop-v0.1.1` would violate the completed release boundary.
- Reintroducing published-tag/ref assumptions into draft-only staging would recreate the repaired failure mode; blocking CI now guards against this.
- Treating Sigstore/SLSA provenance as Windows Authenticode or Apple Developer ID/notarization would misstate trust guarantees.
- Treating CI as live runtime proof would violate runtime safety policy.
- Leaving the short-lived publication PAT or repository secret active unnecessarily increases credential exposure; retire both immediately in GitHub settings.

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
- Draft release-ID verification semantics without a reviewed replacement that preserves draft-inclusive discovery and regression coverage.

## Pending Decisions

- No Desktop `0.1.1` release-content, source, publication, recovery, or workflow-repair decision remains.
- Dependabot #51-#55 are independent future maintenance decisions outside the completed release plan.

## NEXT ACTION

1. Owner security cleanup outside repository history: remove repository Actions secret `RELEASE_ADMIN_TOKEN`, then revoke/delete the associated short-lived fine-grained PAT in GitHub account settings. Never paste its value into chat or repository content.
2. Resume ordinary repository maintenance from live `main`; verify freshness before every decision.
3. Review Dependabot #51-#55 independently when desired; do not conflate them with the completed Desktop `0.1.1` release cycle.
4. For any future desktop release, use the repaired draft workflow plus the separate immutable publication gate/runbook and exact-head evidence model.

## Resume Instructions

1. Read `AGENTS.md` first.
2. Read `PROJECT_STATE.md` second.
3. Inspect actual Git branch, HEAD, working tree, live GitHub `main`, open PRs, checks, ruleset, and releases; do not assume this file is current forever.
4. If verified reality differs, reconcile this document before relying on stale claims.
5. Read `PLANS.md` for completed Desktop `0.1.1` evidence; create a new scoped plan for future multi-stage work rather than reopening the completed plan casually.
6. Read `docs/runbooks/desktop-release.md` before future release recovery/publication operations; immutable `0.1.1` must not be mutated.
7. Read only directly relevant ADRs for architectural/process decisions.
8. Treat old chats and handoff archives as advisory only.
9. Do not repeat completed Desktop `0.1.1` recovery/publication/workflow-repair work unless fresh evidence shows regression.
10. Continue with normal maintenance or the owner-selected next task; no active Desktop `0.1.1` engineering milestone remains.
