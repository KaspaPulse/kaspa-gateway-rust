# ADR-0011: Repository-Native Project Continuity and Source-of-Truth Model

- Status: Accepted
- Date: 2026-08-17
- Owners: Kaspa Gateway maintainers
- Supersedes: none
- Superseded by: none

## Context

Kaspa Gateway work spans repository changes, GitHub Actions qualification, immutable desktop releases, runtime safety invariants, and long-lived tasks that can cross multiple ChatGPT/Codex sessions. Conversation memory and external handoff bundles are useful context but are not durable, independently verifiable project state.

A single oversized handoff document would also create duplicate truth: Git already owns code history, Actions owns CI results, GitHub Releases owns public/draft release state, repository administration owns rules/settings, and real local runtimes own process health.

A second failure mode is **self-stale state**: if a handoff labels a literal SHA as the forever-current `main` or `HEAD`, committing the handoff itself can immediately make that claim obsolete. Similarly, leaving a completed execution narrative in `PLANS.md` makes a historical plan compete with the current handoff.

## Decision

Use the repository as the durable coordination surface with separated responsibilities:

- `AGENTS.md` — stable agent/engineering policy and session-start protocol.
- `PROJECT_STATE.md` — small current-state handoff, reconciled after meaningful state transitions.
- `PLANS.md` — living execution plan only while genuine multi-stage work is active; otherwise it carries an explicit `NO ACTIVE MULTI-STAGE PLAN` sentinel.
- `docs/adr/` — append-only durable decisions with lifecycle metadata.
- `docs/runbooks/` — repeatable operational procedures.
- Git/GitHub Actions/GitHub Releases/repository settings/live runtime state remain the authoritative systems for the facts they own.

`PROJECT_STATE.md` may record a **verified code/implementation baseline** with a historical SHA when that SHA is actual evidence. It must not represent a static SHA as forever-current `HEAD` or current remote `main`. Current branch, HEAD, remote main, and working-tree state are verified dynamically at session start. The state-document commit itself is derived from Git when needed rather than copied into the document.

When a multi-stage plan reaches its completion criteria, reconcile durable outcomes into `PROJECT_STATE.md`, ADRs, runbooks, release records, and Git/PR history as appropriate, then return `PLANS.md` to the inactive sentinel. Completed execution narratives must not remain an active coordination surface.

Conversation memory, old chat summaries, exported ZIPs, screenshots, and historical reports are advisory only. When they conflict with verified repository/CI/release/runtime reality, verified reality wins and the repository state summary is reconciled.

## Alternatives Considered

### Conversation memory as the primary handoff

Rejected because it is not guaranteed to exist in a new session, is difficult to audit, and can become stale relative to GitHub/runtime state.

### One monolithic `HANDOFF.md`

Rejected because it mixes stable rules, transient state, execution plans, decisions, and historical evidence, increasing duplication and stale claims.

### Static current-HEAD/current-main SHAs in the handoff

Rejected because a documentation commit can invalidate its own claim immediately. Historical verified baselines remain useful; current Git identity must be derived dynamically.

### Preserve completed plans indefinitely in `PLANS.md`

Rejected because Git, pull requests, releases, ADRs, and postmortems already own history. An inactive planning surface should say that no plan is active and point to authoritative history rather than duplicate it.

### Generate current state entirely from CI

Rejected because CI cannot authoritatively observe every local runtime/worktree/release-authorization fact, and human/agent next-action context still needs a concise repository-native handoff.

## Consequences

### Positive

- New sessions have a deterministic resume order.
- Transient state is separated from durable policy/decisions/history.
- Verified sources remain authoritative instead of Markdown becoming a competing database.
- Release/runtime claims can remain explicit as `NOT VERIFIED` when evidence is unavailable.
- Documentation commits do not automatically invalidate current-HEAD claims because current Git identity is dynamic.
- Long work can resume from a precise `NEXT ACTION`, while finished plans stop appearing active.

### Negative / Trade-offs

- `PROJECT_STATE.md` requires disciplined reconciliation after meaningful state transitions.
- Some facts intentionally remain dynamic and must be re-queried rather than copied permanently.
- A new multi-stage task must deliberately reactivate `PLANS.md` instead of appending indefinitely to an old plan.
- Maintaining ADR/runbook discipline adds small documentation overhead for consequential changes.

## Validation / Evidence

- `tools/kgw_project_continuity_gate.cjs` validates the permanent continuity surface in blocking CI.
- The existing required `quality (rust + npm)` workflow runs the continuity gate; no separate required status-check context is introduced for continuity documentation.
- The gate requires dynamic current-HEAD/current-main semantics, explicit working-tree classification, Desired/Actual/Drift separation, `Last Verified Validation`, and either an active execution plan contract or the inactive `NO ACTIVE MULTI-STAGE PLAN` contract.
- The gate rejects static 40-character SHAs on labels that claim to be current `HEAD`/remote `main`, and checks continuity documentation for likely secret-value assignments.

## Related Files

- `AGENTS.md`
- `PROJECT_STATE.md`
- `PLANS.md`
- `docs/architecture/README.md`
- `docs/adr/README.md`
- `docs/runbooks/desktop-release.md`
- `tools/kgw_project_continuity_gate.cjs`
- `.github/workflows/ci.yml`

## Related Issues / PRs / Commits

- Continuity implementation and later hardening are discoverable from Git history and PRs. Do not add a self-referential current-HEAD SHA here.

## Change History

- 2026-08-17 — Accepted initial repository-native continuity model.
- 2026-08-17 — Hardened the model against self-stale current-SHA claims and defined the inactive-plan lifecycle for `PLANS.md`.
