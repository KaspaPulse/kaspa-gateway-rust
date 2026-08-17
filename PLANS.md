# EXECUTION PLAN

## Objective

Safely complete Desktop `0.1.1` publication from the existing independently verified draft, then repair the draft-verification workflow without recreating artifacts or weakening release evidence.

## Success Criteria

- Existing draft release ID `371168378` is independently verified from downloaded release assets and provenance.
- Publication uses that exact draft and results in an immutable `desktop-v0.1.1` release targeting the qualified source commit.
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
- Do not weaken the immutable-releases administration gate because a credential lacks the required scope.

## Current Baseline

- Last verified live `main`: `f07b29be804ab39a3b372d27683900af55b52684`.
- Qualified release source: `b911eb44619f8eab706bc2fe786d1c84ac958f1d`; current `main` is one continuity commit ahead and the draft must not be silently retargeted.
- Qualified Desktop Artifacts run: `31910163486` — success.
- Existing draft: release ID `371168378`, `draft=true`, `published_at=null`, target `b911eb4...`, six assets.
- Latest public release: `desktop-v0.1.0`, immutable.
- Draft staging run `31911079027`: failed after draft creation due post-create lookup semantics.
- Independent recovery verifier run `32019926018`: asset/checksum/provenance stages passed; only the immutable-releases administration read failed with HTTP 403 for the available GitHub Actions token.

## Risks

- Re-dispatching the draft workflow can create competing state or duplicated assets.
- Publishing without a fresh immutable-releases administration read would waive an explicit release gate.
- Main has moved after qualification; release-source appropriateness is explicitly classified and the verified draft remains pinned to `b911eb4...`.
- Workflow repairs made before release publication can unnecessarily complicate exact-source release reasoning.
- Publication credentials must be able to preserve the old qualified release target under GitHub's current release/workflow permission rules.

## Milestones

### Milestone 1 — Recover and verify the existing draft — COMPLETE

- Goal: prove the six existing draft assets and provenance are intact without mutating the release.
- Result:
  - release ID/target/state checks passed;
  - all six assets downloaded by release asset ID;
  - asset names, sizes, and GitHub SHA-256 digests matched;
  - strict `SHA256SUMS.txt` verification passed;
  - Windows NSIS, macOS DMG, and macOS App ZIP passed preserved Sigstore bundle verification and online GitHub attestation verification;
  - repository, signer workflow, source digest `b911eb4...`, source ref `refs/heads/main`, SLSA v1 predicate, and self-hosted-runner denial were enforced;
  - post-verification release reads confirm the draft remains unpublished and latest public release remains `desktop-v0.1.0`.
- Evidence: GitHub Actions run `32019926018`, job `95357274069`, plus fresh release API reads.

### Milestone 2 — Publication gate — BLOCKED ON ADMINISTRATION READ

- Goal: establish whether publication is safe and explicitly authorized.
- Completed:
  - recovery evidence is PASS;
  - publication authorization is explicitly granted by the repository owner;
  - live `main`, draft identity, and release-source drift are classified.
- Remaining gate:
  - repository immutable-releases administration read must succeed and report `enabled=true` immediately before publication.
- Current blocker:
  - connected GitHub App and GitHub Actions `GITHUB_TOKEN` return HTTP 403 for the immutable-releases endpoint;
  - a safe repository-secret presence probe found no suitable release-administration token/App credential under the standard names checked.
- Exit criteria: a credential with repository Administration read performs the fresh check successfully while the exact draft/source/assets remain unchanged.

### Milestone 3 — Publish and verify `0.1.1` — PENDING

- Goal: publish the existing draft exactly once.
- Changes: release state changes from draft to published; tag/ref becomes real/public.
- Validation:
  - `draft=false` and `published_at` populated;
  - `immutable=true`;
  - tag/ref resolves to `b911eb4...`;
  - six assets intact and digests unchanged;
  - `gh release verify` and `gh release verify-asset` pass where supported;
  - `gh attestation verify` still passes for release artifacts.
- Exit criteria: immutable public `desktop-v0.1.1` is verified and `PROJECT_STATE.md` is reconciled.

### Milestone 4 — Repair draft workflow verification — PENDING AFTER PUBLICATION

- Goal: remove the known draft-semantics bug without broad workflow redesign.
- Changes:
  - capture/discover the draft release ID;
  - verify draft via `/releases/{release_id}` or list selection;
  - verify `draft`, `tag_name`, `target_commitish`, exact asset set, and digests;
  - defer actual git tag/ref identity checks until publication.
- Validation: `actionlint`, a focused release-workflow contract gate if added, and all required PR checks on the exact head.
- Exit criteria: small PR merged by squash after exact-head checks pass.

## Decisions Needed

- Publication content/source decision: **resolved** — use existing draft ID `371168378` at qualified source `b911eb4...`.
- Publication authorization: **granted**.
- Operational dependency: provide/use a credential with repository Administration read and release publication permissions compatible with the existing qualified draft.
- Independent review/merge decisions for Dependabot PRs #51-#55 remain outside this plan.

## Progress

- Source metadata `0.1.1` merged — verified.
- Qualified Windows/macOS artifact build and provenance — verified.
- Existing draft and six release assets — verified by live API metadata.
- Independent draft-byte recovery verification — **PASS**.
- Strict checksum-manifest verification — **PASS**.
- Preserved-bundle and online GitHub attestation verification — **PASS**.
- Publication authorization — **GRANTED**.
- Immutable-releases administration freshness gate — **BLOCKED BY CREDENTIAL SCOPE**.
- Publication — not performed.
- Workflow repair — pending until the release-critical path is safe.

## Discoveries

- GitHub draft releases cannot be safely verified by assuming `releases/tags/{tag}` and a real git tag ref are available before publication.
- The existing draft was created successfully even though the workflow run concluded failure in its later verification step.
- `contents: write` is sufficient for the recovery workflow to read/download the draft assets, but the immutable-releases status endpoint separately requires repository Administration read.
- An older immutable public release is not sufficient evidence that repository immutable releases remain enabled; the setting must be read freshly at the publication gate.

## Blockers

- **Active blocker:** no credential currently exposed to this execution environment can read the repository immutable-releases administration state. This gate must not be downgraded or inferred.
- No artifact-integrity, checksum, or provenance blocker is established.

## Final Validation

After all milestones, re-check GitHub `main`, release identity, ruleset, required checks, public release immutability, asset digests, provenance, and `PROJECT_STATE.md` freshness.

## Completion Criteria

This plan is finished only when the `0.1.1` release is safely published and verified, the workflow verification bug is repaired and exact-head qualified, and the current-state handoff no longer points to these milestones as active work.
