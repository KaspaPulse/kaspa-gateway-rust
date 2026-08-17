# EXECUTION PLAN

## Objective

Close the Desktop `0.1.1` release cycle by preserving the now-published immutable release exactly as qualified, repairing the draft-verification workflow bug that caused the original staging run to fail after successful draft creation, and leaving repository-native continuity in a completed state.

## Success Criteria

- Desktop `0.1.1` remains immutable, latest, and tag-bound to qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` with the six verified release assets unchanged.
- The known draft workflow bug is repaired without broad workflow redesign or release-state mutation.
- Draft verification uses the created release ID or draft-inclusive release-list semantics and does not require a public git tag/ref before publication.
- The repair PR is exact-head qualified by all required ruleset contexts and squash-merged with no unresolved review threads.
- `PROJECT_STATE.md` is reconciled after the workflow repair and the release plan is closed.

## Non-Goals

- No mutation of immutable `desktop-v0.1.0` or `desktop-v0.1.1`.
- No artifact rebuild or replacement `0.1.1` release.
- No reimplementation of Kaspa consensus, node, or Stratum bridge behavior.
- No new runtime ownership subsystem.
- No `testnet12` live smoke without separate explicit authorization.
- No Dependabot #51-#55 integration inside this release-repair plan.

## Constraints

- Published release source/tag binding remains `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Windows `0.1.1` remains unsigned; macOS remains ad-hoc signed and not notarized. Build provenance must not be represented as OS publisher signing.
- Repository changes use PR-based integration and exact-head CI qualification before squash merge.
- Repair the existing draft workflow rather than adding a parallel release-control subsystem.
- The temporary release-publication credential must not be persisted in repository files or logs and should be removed from repository settings after release operations.

## Current Baseline

- Last verified live `main`: `ba53796a5933cdf97b6b810c208050686a8555e6`.
- Qualified/published release source: `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Qualified Desktop Artifacts run: `31910163486` — success.
- Recovery verifier run: `32019926018` — byte/checksum/provenance recovery passed; administration-read limitation was later resolved with a dedicated credential.
- Exact publication gate run: `32022865218`, job `95366007733` — **success**.
- Latest public release: ID `371168378`, tag `desktop-v0.1.1`, `draft=false`, `immutable=true`, published `2026-08-17T11:01:16Z`.
- Tag `refs/tags/desktop-v0.1.1` resolves directly to exact source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Original draft staging run `31911079027` remains the evidence for the post-create lookup semantics bug.

## Risks

- Any attempt to modify immutable `desktop-v0.1.1` would cross the completed release boundary.
- Reintroducing a tag-ref precondition into the draft-only workflow would recreate the original failure mode.
- A broad rewrite could obscure the small root cause and weaken auditability; prefer a minimal semantics repair.
- A workflow-only fix without a focused regression guard may allow the same draft/tag assumption to return later.
- Dependabot changes touching GitHub Actions should not be conflated with this repair until the release workflow is stable.

## Milestones

### Milestone 1 — Recover and verify the existing draft — COMPLETE

- Six draft assets were downloaded by release asset ID and matched exact GitHub metadata digests/sizes.
- Strict `SHA256SUMS.txt` verification passed.
- Windows NSIS, macOS DMG, and macOS App ZIP passed preserved Sigstore bundle and online GitHub attestation verification with repository/workflow/source/SLSA/self-hosted constraints.
- Evidence: run `32019926018`, job `95357274069`, plus fresh release API reads.

### Milestone 2 — Publication gate — COMPLETE

- Owner publication authorization was present.
- Live `main`, ruleset, open PRs, exact release ID/source/assets, and latest public release were rechecked.
- A dedicated short-lived credential successfully read the repository immutable-releases administration setting immediately before publication and required `enabled=true`.
- No release was recreated or retargeted.

### Milestone 3 — Publish and verify `0.1.1` — COMPLETE

- Existing release ID `371168378` was published in place exactly once.
- Result is `draft=false`, `prerelease=false`, `immutable=true`, and latest public release.
- Tag/ref resolves directly to `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- All six asset IDs/names/sizes/digests remained unchanged.
- `gh release verify`, all `gh release verify-asset` checks, and post-publication build attestations passed.
- Evidence: run `32022865218`, job `95366007733`.

### Milestone 4 — Repair draft workflow verification — ACTIVE

- Goal: remove the known draft-semantics bug without broad workflow redesign.
- Required repair:
  - retain exact-input/source/artifact qualification checks already owned by the workflow;
  - capture or discover the created draft release ID, or select it from a draft-inclusive releases list;
  - verify the release object by ID for `draft`, `tag_name`, `target_commitish`, exact asset set, sizes, and digests;
  - do not require `releases/tags/{tag}` or a real git tag/ref before publication;
  - leave public tag/ref identity verification to the separate publication/post-publication path.
- Regression guard: add or extend a focused repository-native contract check if the current test surface does not explicitly enforce these semantics.
- Validation: `actionlint`, focused contract gate, and all six ruleset-required PR contexts on the exact final head.
- Exit criteria: one small repair PR squash-merged, no unresolved review threads, and continuity docs reconciled to completion.

## Decisions Needed

- Desktop `0.1.1` release content/source/publication decisions are **resolved and complete**.
- Workflow repair approach: reuse the existing workflow and fix its lookup/verification semantics; do not add a parallel subsystem.
- Dependabot #51-#55 remain independent future decisions outside this plan.

## Progress

- Source metadata `0.1.1` — **VERIFIED**.
- Qualified Windows/macOS native artifacts — **VERIFIED**.
- Draft byte/checksum recovery — **PASS**.
- Preserved/online build provenance — **PASS**.
- Immutable-releases freshness gate — **PASS**.
- Publication of existing release ID `371168378` — **PASS**.
- Immutable release/tag/assets verification — **PASS**.
- Post-publication build attestation verification — **PASS**.
- Publication-state continuity reconciliation — **IN PROGRESS VIA PR**.
- Draft workflow repair — **NEXT**.
- Release-plan closure — **PENDING WORKFLOW REPAIR MERGE**.

## Discoveries

- GitHub draft releases cannot be safely verified by assuming `releases/tags/{tag}` or a real git tag ref is available before publication.
- The original draft workflow created and uploaded the intended draft successfully before its later verification step failed.
- Release asset recovery can bind directly to release asset IDs and GitHub SHA-256 metadata.
- The immutable-releases administration setting is a distinct publication gate and requires appropriate repository administration scope.
- Once immutable publication succeeds, GitHub release/asset attestations provide a separate release-distribution verification layer in addition to build provenance.

## Blockers

- No release-distribution blocker remains for Desktop `0.1.1`.
- Active engineering item: `.github/workflows/desktop-release-draft.yml` post-create verification semantics.
- No artifact-integrity, checksum, provenance, immutable-release, or tag-binding blocker remains.

## Final Validation

After the workflow-repair merge, re-check live `main`, open PRs, ruleset, required checks, immutable `desktop-v0.1.1` identity/tag/assets, relevant workflow contract, and `PROJECT_STATE.md` freshness. Remove the short-lived publication secret through repository settings after release operations.

## Completion Criteria

This plan is finished only when the draft-verification workflow repair is squash-merged after exact-head required checks, immutable `desktop-v0.1.1` remains unchanged and verified, the short-lived publication credential is removed from repository settings, and the current-state handoff no longer points to Desktop `0.1.1` release milestones as active work.
