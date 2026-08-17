# EXECUTION PLAN

## Objective

Safely complete Desktop `0.1.1` release preparation from the existing qualified draft, then repair the draft-verification workflow without recreating artifacts or weakening release evidence.

## Success Criteria

- Existing draft release ID `371168378` is independently verified from downloaded release assets and provenance.
- Publication, if authorized, publishes that exact draft and results in an immutable `desktop-v0.1.1` release targeting the qualified source commit.
- Post-publication verification proves release/tag/source identity, immutable-release verification, asset binding, checksums, and build provenance.
- The draft workflow bug is repaired in a small, reviewed PR after the release-critical path is safe.
- `PROJECT_STATE.md` is reconciled at each meaningful state transition.

## Non-Goals

- No reimplementation of Kaspa consensus, node, or Stratum bridge behavior.
- No new runtime ownership subsystem.
- No `testnet12` live smoke without separate explicit authorization.
- No artifact rebuild or second `0.1.1` draft unless evidence proves the current bytes invalid.
- No dependency-upgrade work from Dependabot PRs #51-#55 inside this plan.

## Constraints

- Release source remains `b911eb44619f8eab706bc2fe786d1c84ac958f1d` unless a reviewed release decision explicitly changes it and requalification is performed.
- Public `desktop-v0.1.0` is immutable and must not be mutated.
- Existing draft ID `371168378` is the only intended `0.1.1` draft.
- Windows is unsigned; macOS is ad-hoc signed and not notarized. Build provenance must not be represented as OS publisher signing.
- Use PR-based integration for repository changes and exact-head CI qualification before merge.

## Current Baseline

- `main` observed: `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Qualified Desktop Artifacts run: `31910163486` — success.
- Existing draft: release ID `371168378`, `draft=true`, target `b911eb4...`, six assets.
- Latest public release: `desktop-v0.1.0`, immutable.
- Draft staging run `31911079027`: failed after draft creation due post-create lookup semantics.

## Risks

- Re-dispatching the draft workflow can create competing state or duplicated assets.
- Publishing before independent verification can make a bad release immutable.
- Main may move because of unrelated Dependabot PRs; release-source appropriateness must be explicitly classified rather than silently retargeted.
- Workflow repairs made before release publication can unnecessarily complicate exact-source release reasoning.

## Milestones

### Milestone 1 — Recover and verify the existing draft

- Goal: prove the six existing draft assets and provenance are intact without mutating the release.
- Changes: none to GitHub release state.
- Validation:
  - release ID/target/state checks;
  - asset ID/name/size/digest checks;
  - local SHA-256 checks;
  - strict checksum manifest verification;
  - Sigstore bundle verification;
  - online GitHub attestation verification.
- Exit criteria: all recovery checks pass and no-publish assertions still hold.

### Milestone 2 — Publication gate

- Goal: establish whether publication is safe and explicitly authorized.
- Changes: none until authorization.
- Validation:
  - current `main`/PR/ruleset freshness;
  - repository immutable-releases administration read succeeds and `enabled=true`;
  - draft ID/source/assets remain exact.
- Exit criteria: recovery evidence is PASS and the user explicitly authorizes publication.

### Milestone 3 — Publish and verify `0.1.1`

- Goal: publish the existing draft exactly once.
- Changes: release state changes from draft to published; tag/ref becomes real/public.
- Validation:
  - `draft=false` and `published_at` populated;
  - `immutable=true`;
  - tag/ref resolves to `b911eb4...`;
  - six assets intact;
  - `gh release verify` and `gh release verify-asset` pass where supported;
  - `gh attestation verify` still passes for release artifacts.
- Exit criteria: immutable public `desktop-v0.1.1` is verified and `PROJECT_STATE.md` is reconciled.

### Milestone 4 — Repair draft workflow verification

- Goal: remove the known draft-semantics bug without broad workflow redesign.
- Changes:
  - capture/discover the draft release ID;
  - verify draft via `/releases/{release_id}` or list selection;
  - verify `draft`, `tag_name`, `target_commitish`, exact asset set, and digests;
  - defer actual git tag/ref identity checks until publication.
- Validation: `actionlint`, relevant workflow contract gates, required PR checks on exact head.
- Exit criteria: small PR merged by squash after exact-head checks pass.

## Decisions Needed

- Explicit publication authorization after Milestones 1 and 2 pass.
- Independent review/merge decisions for Dependabot PRs #51-#55 outside this plan.

## Progress

- Source metadata `0.1.1` merged — verified.
- Qualified Windows/macOS artifact build and provenance — verified.
- Existing draft and six release assets — verified to exist by live GitHub API metadata.
- Independent draft-byte recovery verification — pending.
- Publication — not authorized/not performed.
- Workflow repair — pending until the release-critical path is safe.

## Discoveries

- GitHub draft releases cannot be safely verified by assuming `releases/tags/{tag}` and a real git tag ref are available before publication.
- The existing draft was created successfully even though the workflow run concluded failure in its later verification step.

## Blockers

- No artifact-integrity blocker is currently established.
- Publication remains intentionally blocked until independent recovery verification and explicit authorization.

## Final Validation

After all milestones, re-check GitHub `main`, release identity, ruleset, required checks, public release immutability, asset digests, provenance, and `PROJECT_STATE.md` freshness.

## Completion Criteria

This plan is finished only when the `0.1.1` release state is safely resolved, the workflow verification bug is repaired and qualified, and the current-state handoff no longer points to these milestones as active work.
