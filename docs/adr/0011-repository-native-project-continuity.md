# ADR-0011: Repository-Native Project Continuity and Source-of-Truth Model

- Status: Accepted
- Date: 2026-08-17
- Owners: Kaspa Gateway maintainers
- Supersedes: none
- Superseded by: none

## Context

Kaspa Gateway work spans repository changes, GitHub Actions qualification, immutable desktop releases, runtime safety invariants, and long-lived release tasks that can cross multiple ChatGPT/Codex sessions. Conversation memory and external handoff bundles are useful context but are not durable, independently verifiable project state.

A single oversized handoff document would also create duplicate truth: Git already owns code history, Actions owns CI results, GitHub Releases owns public/draft release state, and real local runtimes own process health.

## Decision

Use the repository as the durable coordination surface with separated responsibilities:

- `AGENTS.md` — stable agent/engineering policy and session-start protocol.
- `PROJECT_STATE.md` — small current-state handoff, reconciled after meaningful state transitions.
- `PLANS.md` — living execution plan for active multi-stage work.
- `docs/adr/` — append-only durable decisions with lifecycle metadata.
- `docs/runbooks/` — repeatable operational procedures.
- Git/GitHub Actions/GitHub Releases/live runtime state remain the authoritative systems for the facts they own.

`PROJECT_STATE.md` may record a last verified implementation baseline but must not claim a static SHA is forever the current HEAD. Every new local session verifies branch, HEAD, and worktree dynamically before relying on the state summary.

Conversation memory, old chat summaries, exported ZIPs, and historical reports are advisory only. When they conflict with verified repository/CI/release/runtime reality, verified reality wins and the repository state summary is reconciled.

## Alternatives Considered

### Conversation memory as the primary handoff

Rejected because it is not guaranteed to exist in a new session, is difficult to audit, and can become stale relative to GitHub/runtime state.

### One monolithic `HANDOFF.md`

Rejected because it mixes stable rules, transient state, execution plans, decisions, and historical evidence, increasing duplication and stale claims.

### Generate current state entirely from CI

Rejected because CI cannot authoritatively observe every local runtime/worktree/release-authorization fact, and human/agent next-action context still needs a concise repository-native handoff.

## Consequences

### Positive

- New sessions have a deterministic resume order.
- Transient state is separated from durable policy/decisions.
- Verified sources remain authoritative instead of Markdown becoming a competing database.
- Release/runtime claims can remain explicit as `NOT VERIFIED` when evidence is unavailable.
- Long work can resume from a precise `NEXT ACTION` without replaying old chats.

### Negative / Trade-offs

- `PROJECT_STATE.md` requires disciplined reconciliation after meaningful state transitions.
- Some facts intentionally remain dynamic and must be re-queried rather than copied permanently.
- Maintaining ADR/runbook discipline adds small documentation overhead for consequential changes.

## Validation / Evidence

- `tools/kgw_project_continuity_gate.cjs` validates the permanent continuity surface in CI.
- The existing required `quality (rust + npm)` workflow runs the continuity gate; no new required status-check context is introduced.
- `PROJECT_STATE.md` explicitly separates desired state, repository/CI/release/runtime actual state, drift, and `NOT VERIFIED` facts.

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

- The continuity-system PR/merge commit is discoverable from Git history; do not duplicate a static self-referential HEAD here.

## Change History

- 2026-08-17 — Accepted initial repository-native continuity model.
