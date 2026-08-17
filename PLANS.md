# EXECUTION PLAN

## Status

**NO ACTIVE MULTI-STAGE PLAN**

`PLANS.md` is an active-work coordination surface, not a historical archive. The most recent Desktop `0.1.1` release/recovery/workflow-repair plan met its repository/release engineering completion criteria and is closed. Current facts belong in `PROJECT_STATE.md`; durable decisions belong in ADRs; operational procedures belong in runbooks; implementation and review history belongs in Git and pull requests; release history belongs in GitHub Releases.

## Usage

Create or replace the active-plan body only when work is genuinely long-horizon, multi-stage, migration-heavy, high-risk, or expected to span sessions. An active plan should define the objective, success criteria, non-goals, constraints, baseline, risks, milestones, decisions needed, progress, discoveries, blockers, final validation, and completion criteria.

When an active plan reaches its completion criteria:

1. Reconcile the resulting current state into `PROJECT_STATE.md`.
2. Record any durable architectural/process decision in the appropriate ADR.
3. Update the relevant runbook if the operational procedure changed.
4. Leave release/version history in GitHub Releases and implementation history in Git/PRs.
5. Return this file to **NO ACTIVE MULTI-STAGE PLAN** rather than preserving a completed execution narrative as active coordination state.

Do not use this file as a duplicate issue tracker, Git log, CI history, release body, or credential store.

## Most Recent Completed Plan

Desktop `0.1.1` release-cycle engineering is closed. The durable evidence and current handoff are intentionally distributed to the systems that own them:

- Current resumable project state: `PROJECT_STATE.md`.
- Continuity/source-of-truth decision: `docs/adr/0011-repository-native-project-continuity.md`.
- Release procedure: `docs/runbooks/desktop-release.md`.
- Draft-release regression guard: `tools/kgw_desktop_release_draft_workflow_gate.cjs`.
- Publication/recovery/workflow-repair implementation history: Git and PRs #58, #59, and #60.
- Published release identity and assets: GitHub Release ID `371168378`, tag `desktop-v0.1.1`.

The previous plan is not reopened merely because later maintenance occurs. Start a new scoped plan if a future task independently meets the multi-stage criteria above.

## Starting a New Plan

Before activating a new plan:

1. Read `AGENTS.md` and `PROJECT_STATE.md`.
2. Verify branch, HEAD, working tree, remotes as relevant, current GitHub `main`, open PRs, required checks/ruleset, and any runtime/release/deployment facts relevant to the task.
3. Reconcile `PROJECT_STATE.md` first if verified reality differs.
4. Reuse existing architecture, ADRs, runbooks, workflows, and components before creating parallel machinery.
5. Define evidence-based exit criteria and keep `NOT VERIFIED` explicit where access or execution is unavailable.
