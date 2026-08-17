# Desktop Release Runbook

## Scope

This runbook covers qualification, recovery verification, publication, and post-publication verification for Kaspa Gateway Desktop GitHub Releases. It does not start/stop Kaspa node or bridge runtimes and does not authorize `testnet12` live smoke.

The active `0.1.1` release-specific state belongs in `PROJECT_STATE.md`/`PLANS.md`; this runbook is the durable procedure.

## Preconditions

Before any release mutation:

1. Read `AGENTS.md`, `PROJECT_STATE.md`, and `PLANS.md`.
2. Verify `main`, open PRs, the main ruleset, exact release source SHA, qualified Desktop Artifacts run, current latest public release, and any existing draft release.
3. Verify the exact artifact build run is successful on the intended source SHA.
4. Confirm Windows/macOS qualification evidence and provenance exist.
5. Confirm `testnet12` live smoke is not being inferred or silently run.
6. Ensure the intended release version matches the desktop package metadata.
7. Ensure no competing draft/release for the same version exists.

If any source, draft, artifact, or policy identity is ambiguous, **abort mutation and reconcile first**.

## Required Access

- GitHub repository read access for releases, Actions, attestations, rulesets, and commits.
- Release mutation permission only for the explicit publication/staging action being performed.
- Repository administration read access for immutable-releases settings immediately before publication.
- No secret values belong in this runbook or evidence archive.

## Safety Rules

- Never mutate an existing immutable published release.
- Never create a second draft when a valid draft for the intended version already exists.
- Never retarget a qualified draft to a different source SHA without requalification and explicit review.
- Never represent Sigstore/SLSA build provenance as Windows Authenticode or Apple Developer ID/notarization.
- Never treat a successful build, upload, or mock as live runtime proof.
- Publication is a separate high-impact mutation and requires explicit authorization after the verification gate passes.

## Stage a New Draft

Use `.github/workflows/desktop-release-draft.yml` only when no intended draft already exists and the workflow's current implementation is known to be safe for the repository state.

Required inputs must bind the version, exact commit SHA, and exact qualified Desktop Artifacts run. The workflow must create **draft only** and must not publish.

After creation, discover/capture the release ID. While the release is still a draft, verify it by release ID or by listing releases and selecting the exact tuple:

- `draft == true`
- expected `tag_name`
- expected `target_commitish`

Do **not** require `releases/tags/{tag}` or a real `git/ref/tags/{tag}` to exist before publication.

## Recover and Verify an Existing Draft

This is the preferred path whenever an intended draft already exists.

1. Fetch the draft directly by release ID.
2. Verify `draft=true`, `published_at=null`, expected version/tag name, expected target SHA, and expected exact asset set.
3. Download release assets by release asset ID rather than assuming the draft tag URL behaves like a published release.
4. For every asset:
   - compute SHA-256 locally;
   - compare with GitHub `asset.digest`;
   - reject zero-length/unexpected/duplicate assets.
5. Verify `SHA256SUMS.txt` strictly against the product assets.
6. Verify each product artifact against its preserved Sigstore bundle and against the online GitHub attestation store.
7. Enforce the intended repository, signer workflow, source digest, source ref `refs/heads/main`, SLSA v1 predicate, and denial of self-hosted runners.
8. Re-fetch the draft and latest public release to prove verification caused no publication or release mutation.

### Abort Conditions

Abort and do not publish if any of these occur:

- draft source SHA differs from the qualified source;
- asset set differs from the expected set;
- local digest differs from GitHub metadata or checksum manifest;
- bundle/online provenance verification fails;
- another public/draft release creates version ambiguity;
- repository immutable-release policy cannot be read immediately before publication;
- the user has not explicitly authorized publication.

## Publication Gate

Immediately before publication:

1. Re-check current `main` and open PRs.
2. If `main` moved after artifact qualification, explicitly classify whether the already-qualified source SHA remains appropriate; never silently retarget.
3. Re-check the exact draft release ID/source/assets.
4. Re-check repository immutable-releases administration state and require it to be enabled.
5. Require explicit user authorization for publication.

## Publish

Publish the **existing verified draft**. Do not create a replacement release.

No unrelated repository, runtime, or service mutation belongs in the publication operation.

## Post-Publication Verification

Immediately after publication verify:

- `draft=false`;
- `published_at` is populated;
- `immutable=true`;
- expected tag name exists;
- actual git tag/ref resolves to the qualified source SHA;
- release target/source identity is unchanged;
- exact expected asset set and digests remain intact;
- latest public release identity is the newly published version only after success;
- native immutable-release verification (`gh release verify`) passes when supported;
- downloaded release asset binding (`gh release verify-asset`) passes when supported;
- build provenance verification (`gh attestation verify`) still passes.

Reconcile `PROJECT_STATE.md` immediately after this state transition.

## Rollback / Recovery Model

Published immutable releases are not edited in place as a rollback mechanism. If a published release is materially defective:

1. Preserve the immutable release/evidence.
2. Stop further promotion/distribution actions under maintainer control where applicable.
3. Document the defect and impact.
4. Prepare a corrected patch release from a reviewed source revision.
5. Re-run artifact qualification/provenance and publish the corrected version through the same gates.

Do not delete/rewrite history merely to make the previous release appear clean.

## Expected Signals

- Exact source SHA is stable through qualification and release verification.
- Product asset hashes match both GitHub metadata and checksum manifest.
- Sigstore/SLSA verification reports the expected repository/workflow/source identity.
- A draft remains unpublished until the explicit publication step.
- A published release becomes immutable under the repository policy.

## Failure Modes

- Draft API/tag semantics differ from published release semantics.
- Artifact retention expires before draft recovery; the release draft assets may still be valid and must be independently verified instead of blindly rebuilding.
- `main` moves after qualification; this requires explicit source-appropriateness classification.
- OS-native signing is absent even though build provenance is valid; report both trust layers accurately.

## Related Procedures

- Live-network smoke: `docs/operations/live-network-smoke.md`.
- Current release state and exact IDs: `PROJECT_STATE.md`.
- Active release milestones: `PLANS.md`.
