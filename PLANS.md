# EXECUTION PLAN

## Status

**COMPLETE — REPOSITORY/RELEASE ENGINEERING CLOSED. MANUAL CREDENTIAL RETIREMENT REQUIRED OUTSIDE REPOSITORY HISTORY.**

## Objective

Safely complete Desktop `0.1.1` from the existing qualified draft, independently verify and publish that exact release as immutable, repair the draft-verification workflow bug that caused the original staging run to fail after successful creation, and leave repository-native continuity in a completed and guarded state.

## Success Criteria

- Existing release ID `371168378` is independently recovered and verified from exact release assets and build provenance — **COMPLETE**.
- Publication uses that exact release without recreation/retargeting and results in immutable `desktop-v0.1.1` bound to qualified source `b911eb44619f8eab706bc2fe786d1c84ac958f1d` — **COMPLETE**.
- Post-publication verification proves release/tag/source identity, immutability, asset binding, checksums, release verification, and build provenance — **COMPLETE**.
- Draft verification uses draft-inclusive release discovery, propagates the created numeric release ID, verifies the release object by ID, and does not require a public tag/ref before publication — **COMPLETE**.
- A focused blocking CI regression gate protects those draft semantics — **COMPLETE**.
- The workflow repair is exact-head qualified by all six required ruleset contexts and squash-merged with no unresolved review threads — **COMPLETE**.
- `PROJECT_STATE.md` and this plan are reconciled after the repair and no Desktop `0.1.1` engineering milestone remains active — **COMPLETE via final closure PR**.

## Non-Goals

- No mutation of immutable `desktop-v0.1.0` or `desktop-v0.1.1`.
- No artifact rebuild or replacement `0.1.1` release.
- No reimplementation of Kaspa consensus, node, or Stratum bridge behavior.
- No new runtime ownership subsystem.
- No `testnet12` live smoke without separate explicit authorization.
- No Dependabot #51-#55 integration inside this release plan.

## Constraints

- Published release source/tag binding remains `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Windows `0.1.1` remains unsigned; macOS remains ad-hoc signed and not notarized. Build provenance is not OS publisher signing.
- Repository changes use PR-based integration and exact-head CI qualification before squash merge.
- Existing Node/Bridge/runtime ownership and zero-fake-log invariants remain unchanged.
- The temporary release-publication credential must never be persisted in repository files/logs and should be retired from GitHub settings immediately after release operations.

## Final Baseline

- Live `main` after workflow repair: `197bdead257973164931b34cce20c4556820df44` (`fix: verify desktop release drafts by release ID (#59)`).
- Qualified/published release source: `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Qualified Desktop Artifacts run: `31910163486` — success.
- Independent recovery verifier: run `32019926018`, job `95357274069` — draft byte/checksum/provenance recovery passed.
- Exact publication gate: run `32022865218`, job `95366007733` — **success**.
- Latest public release: ID `371168378`, tag `desktop-v0.1.1`, `draft=false`, `prerelease=false`, `immutable=true`, published `2026-08-17T11:01:16Z`.
- Tag `refs/tags/desktop-v0.1.1` resolves directly to exact source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- Workflow repair PR #59 final head `16426e3062878a7d2ccc271a9bc1aeeb06b7e3cd` passed all six required contexts and was squash-merged as `197bdead257973164931b34cce20c4556820df44`.
- Blocking CI includes `tools/kgw_desktop_release_draft_workflow_gate.cjs`.

## Risks After Closure

- Mutating immutable `desktop-v0.1.1` would violate the completed release boundary.
- Removing or weakening the dedicated draft-workflow contract could allow the repaired published-tag/ref assumption to regress.
- Treating Sigstore/SLSA provenance as Windows Authenticode or Apple Developer ID/notarization would misstate trust guarantees.
- Treating CI or mocks as live runtime proof would violate runtime safety policy.
- Leaving the short-lived publication repository secret or PAT active unnecessarily increases credential exposure; retire both manually in GitHub settings.

## Milestones

### Milestone 1 — Recover and verify the existing draft — COMPLETE

- Six draft assets downloaded by release asset ID and matched exact GitHub metadata sizes/digests.
- Strict `SHA256SUMS.txt` verification passed.
- Windows NSIS, macOS DMG, and macOS App ZIP passed preserved Sigstore bundle and online GitHub attestation verification with repository/workflow/source/SLSA/self-hosted constraints.
- Evidence: run `32019926018`, job `95357274069`, plus release API reads.

### Milestone 2 — Publication gate — COMPLETE

- Owner publication authorization was present.
- Live `main`, ruleset, open PRs, exact release ID/source/assets, and latest public release were rechecked.
- Dedicated short-lived credential successfully read the immutable-releases administration setting immediately before publication and required `enabled=true`.
- No release was recreated or retargeted.

### Milestone 3 — Publish and verify `0.1.1` — COMPLETE

- Existing release ID `371168378` was published in place exactly once.
- Result is `draft=false`, `prerelease=false`, `immutable=true`, and latest public release.
- Tag/ref resolves directly to `b911eb44619f8eab706bc2fe786d1c84ac958f1d`.
- All six asset IDs/names/sizes/digests remained unchanged.
- `gh release verify`, all `gh release verify-asset` checks, and post-publication build attestations passed.
- Evidence: run `32022865218`, job `95366007733`.

### Milestone 4 — Repair draft workflow verification — COMPLETE

- Reused the existing `.github/workflows/desktop-release-draft.yml`; no parallel release subsystem was added.
- Preflight now uses authenticated paginated `List releases` semantics so existing drafts are visible and rejected.
- Creation resolves the new draft uniquely by `draft=true`, exact `tag_name`, and exact `target_commitish`, then exports its numeric release ID.
- Post-create verification reads `/releases/{release_id}` and verifies ID/state/tag/target/asset set/sizes/SHA-256 digests.
- Post-create verification no longer depends on the published-release-by-tag endpoint or a real git tag/ref.
- Added `tools/kgw_desktop_release_draft_workflow_gate.cjs` and invoked it in blocking `quality (rust + npm)` CI.
- Initial repair head exposed one ShellCheck SC2034 warning for an unused retry-loop variable; fixed minimally by using `_` with no behavior change.
- Final PR #59 head `16426e3062878a7d2ccc271a9bc1aeeb06b7e3cd` passed `actionlint`, the new contract gate, all Rust/npm quality gates, TruffleHog, dependency review, policy/audit/deny/machete, and Rust security-extended analysis; no review threads remained.
- PR #59 squash-merged as `197bdead257973164931b34cce20c4556820df44`.

### Milestone 5 — Continuity and operational closure — COMPLETE VIA FINAL PR

- Publication state was reconciled through PR #58.
- Workflow repair state is reconciled through this final closure update.
- The temporary exact-publication workflow was retired from the evidence branch after successful use.
- Desktop `0.1.1` has no remaining repository/release engineering blocker.
- External credential retirement remains immediate owner security hygiene and does not reopen the completed engineering milestones.

## Decisions Resolved

- Desktop `0.1.1` source: exact qualified `b911eb44619f8eab706bc2fe786d1c84ac958f1d` — resolved.
- Release object: existing ID `371168378`, no replacement — resolved.
- Publication: completed and immutable — resolved.
- Draft workflow repair: reuse existing workflow with draft-inclusive discovery + release-ID verification — resolved.
- Regression prevention: dedicated repository-native blocking CI gate — resolved.
- Dependabot #51-#55 remain independent future maintenance decisions outside this completed plan.

## Progress

- Source metadata `0.1.1` — **VERIFIED**.
- Qualified Windows/macOS native artifacts — **VERIFIED**.
- Draft byte/checksum recovery — **PASS**.
- Preserved/online build provenance — **PASS**.
- Immutable-releases freshness gate — **PASS**.
- Publication of existing release ID `371168378` — **PASS**.
- Immutable release/tag/assets verification — **PASS**.
- Post-publication build attestation verification — **PASS**.
- Publication-state continuity reconciliation — **COMPLETE**.
- Draft workflow repair — **COMPLETE / MERGED #59**.
- Dedicated draft semantics regression gate — **COMPLETE / BLOCKING CI**.
- Release-plan repository engineering closure — **COMPLETE**.
- Manual short-lived credential retirement — **OWNER ACTION OUTSIDE REPOSITORY HISTORY**.

## Discoveries

- Draft-only staging must not assume the published-release-by-tag endpoint or a public tag ref exists before publication.
- Authenticated release listing is the correct discovery surface for drafts; binding subsequent verification to the numeric release ID produces auditable object identity.
- The original staging failure occurred after successful draft creation/upload, so recovery/reuse was safer than rebuilding or creating a competing release.
- Immutable-releases administration state is a separate publication gate and must be checked with appropriate repository administration scope.
- Immutable release/asset attestations provide a distribution-verification layer separate from build provenance.
- Graphify policy was read in full, but execution in this agent sandbox was environment-limited; no Graphify PASS was claimed and no generated Graphify artifacts were introduced.

## Blockers

- **None for Desktop `0.1.1` repository/release engineering.**
- Manual security hygiene only: remove repository Actions secret `RELEASE_ADMIN_TOKEN` and revoke/delete the associated short-lived fine-grained PAT in GitHub settings.
- Dependabot #51-#55 are unrelated open maintenance proposals, not release blockers.

## Final Validation

- Live `main` after repair: `197bdead257973164931b34cce20c4556820df44` before this final docs-only closure PR.
- Repaired workflow and dedicated regression gate are present on `main`.
- Release ID `371168378` remains latest, public, immutable, and bound to exact `b911eb4...` source with the verified asset set.
- Active ruleset `20627285` remains squash-only with six required contexts and no bypass actors.
- Final closure PR must itself pass all six required contexts on its exact head before squash merge.

## Completion Criteria

The Desktop `0.1.1` repository/release engineering plan is complete when this final continuity closure PR is exact-head qualified and squash-merged while the immutable release remains unchanged. The only action that remains afterward is owner-side credential retirement in GitHub settings: remove `RELEASE_ADMIN_TOKEN` and revoke/delete its short-lived fine-grained PAT. That security cleanup is external to repository history and must not be represented as an unresolved release-engineering milestone.
