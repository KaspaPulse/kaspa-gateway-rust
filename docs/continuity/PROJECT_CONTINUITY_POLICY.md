# Project Continuity, Security Engineering, and Knowledge Management Policy

## Purpose
Kaspa Gateway is treated as one continuous engineering process. Sessions, agents, model changes, crashes, timeouts, and handoffs must not cause verified work to be rediscovered or silently lost.

This policy implements the requested 2026-09-10 continuity lifecycle in repository-native form. Durable handoff records live in `docs/handoff-ledger/` so they are versioned with Kaspa Gateway and governed by ADR-0011.

## Golden Rule
Never lose verified progress. Never repeat completed work without evidence. Never forget a material failure. Never fix a serious bug without regression protection. Never confuse documentation with reality. Never claim verification without evidence. Never leave `NEXT ACTION` ambiguous.

Required lifecycle:

`READ -> RECOVER -> VERIFY -> RECONCILE -> CHECK -> PRIORITIZE -> CONTINUE -> CHANGE -> TEST -> PROTECT AGAINST REGRESSION -> DOCUMENT -> CHECKPOINT -> HANDOFF`

## Sources of Truth
Use the owner of each fact, in this order when applicable:

1. Verified live runtime/deployment/release state for operational facts.
2. Current filesystem and Git state for code/worktree facts.
3. CI/CD evidence for builds, tests, security gates, and automation.
4. `PROJECT_STATE.md` after reconciliation.
5. `ACTIVE_TASK.md` and `CURRENT_STATE.md` for current execution context.
6. Accepted ADRs and relevant runbooks.
7. `docs/handoff-ledger/` and `docs/project-memory/` records.
8. Conversation memory and historical bundles as advisory context only.

Observable reality wins when sources disagree. Reconcile the durable record instead of ignoring the discrepancy.

## Session Start and Recovery
Before changing code, establish the minimum sufficient state: branch, HEAD, working tree, relevant recent commits, current files, task state, and only the runtime/configuration/tests required by the active task.

After interruption, do not restart from zero. Read `PROJECT_STATE.md`, `ACTIVE_TASK.md`, `CURRENT_STATE.md`, and the latest relevant checkpoint; identify the last confirmed action and the first uncertain boundary; verify that boundary; resume from `NEXT ACTION`.

Do not rerun broad audits or full test suites merely because a new session started. Reuse trustworthy evidence unless code/environment changed, verification was inconclusive, a regression appeared, or security revalidation is necessary.

## State Surfaces
- `AGENTS.md`: durable engineering/session policy.
- `PROJECT_STATE.md`: stable high-level current state; it must keep current Git identity dynamic rather than self-stale.
- `ACTIVE_TASK.md`: current task objective, scope, phase, progress, blocker, last/current/next action, verification, and completion criteria.
- `CURRENT_STATE.md`: short operational answer to "what must a new session know right now?".
- `PLANS.md`: active only for genuine multi-stage/high-risk work; return to the inactive sentinel after closure.
- `docs/handoff-ledger/`: concise atomic checkpoints.
- `docs/project-memory/`: durable problem/failure/security records.

## Checkpoints
Create or update a checkpoint after a meaningful investigation, root cause, change, failed or passed verification, security finding, major decision, migration, deployment preparation, or state transition.

A useful checkpoint records timestamp, task, state, completed/verified work, changed files/actions, tests/evidence, root cause when known, decisions, remaining work, blockers, `LAST CONFIRMED STATE`, `NEXT ACTION`, and `DO NOT REPEAT`.

The ledger is a continuity mechanism, not a verbose activity log. Git history is not a substitute for why a decision exists or what must happen next.

## Permanent Problem Memory
Material findings belong under `docs/project-memory/` using stable IDs and explicit status. Categories are `BUGS`, `REGRESSIONS`, `SECURITY`, `INCIDENTS`, `DECISIONS`, and `KNOWN_FAILURES`.

Allowed status vocabulary includes `OPEN`, `IN PROGRESS`, `RESOLVED`, `VERIFIED`, `NOT REPRODUCIBLE`, `DEFERRED`, `BLOCKED`, `REQUIRES ACTION`, `DUPLICATE`, and `FALSE POSITIVE`.

Important bugs follow: `Bug -> Reproduce -> Root Cause -> Fix -> Verification -> Regression Protection -> Documentation`.

A returning problem is not a new bug by default. Locate the previous record/fix/verification, determine why the protection failed, then `Fix -> Strengthen -> Verify -> Protect`.

## Root Cause and Regression Protection
For significant failures, record what failed, where, why, why existing protections missed it, contributing conditions, similar affected paths, and recurrence risk.

A symptom disappearing is insufficient. A successful fix addresses both the immediate failure and the mechanism that allowed it.

Every important bug that can reasonably be protected must leave a durable guard: automated/integration/security test, validation, assertion/type constraint, static analysis, CI check, configuration validation, monitoring/health check, runtime guard, documentation rule, or architectural constraint.

Tests are permanent project memory: they validate current behavior and preserve historical knowledge of failures.

## Security Engineering
Security is continuous engineering, not a one-time audit. Evaluate security when design, code, dependencies, authentication/authorization, APIs, databases, configuration, deployment, or infrastructure change.

For applicable web/application controls, use OWASP ASVS 5.0.0 as a primary verification reference and follow secure-development principles consistent with the NIST Secure Software Development Framework. Checklist review alone is not compliance evidence; verify implementation.

Relevant areas include authentication/session management, authorization/access control, input validation/output encoding, cryptography/secrets, API/file security, SSRF/injection/XSS/CSRF, security headers/CORS/rate limiting, logging/error handling, secure configuration, dependency security, and data protection.

Security findings must distinguish `CONFIRMED`, `LIKELY`, `POTENTIAL`, `NOT REPRODUCIBLE`, or `FALSE POSITIVE`. Confirmed vulnerabilities record affected component, attack precondition, impact, root cause, reproduction evidence, fix, verification, and regression protection.

Never record secret values, passwords, private keys, cookies, or access tokens in project memory, checkpoints, evidence, or documentation.

## Dependency and External Change Discipline
Dependencies are part of the security boundary. Review purpose, security impact, compatibility, relevant tests, and regressions. Do not blindly upgrade or ignore updates.

Continuously account for automatic PRs, dependency/security updates, CI-generated changes, advisories, failed checks, vulnerabilities, runtime/configuration errors, and regressions when relevant to the active task.

Do not merge or push externally generated changes merely because automation created them. Repository-authorized protected auto-merge remains governed by its own exact-head checks and exclusions.

## Engineering Change Discipline
Prefer small, traceable changes. Avoid unrelated refactors, mass formatting, broad rewrites, and changes to unrelated systems while fixing one issue.

After each meaningful change, run the smallest relevant verification, confirm intended behavior, check obvious regressions, record the result, and checkpoint. Expand validation proportionally when the change crosses boundaries.

Never hide failed commands/tests/deployments/migrations. Record what failed, why when known, relevance, whether it blocks, and the next action. Never report success over a failed verification.

## Local Work, Git, and External Actions
Protect local work. Do not reset, discard, hide, clean, or overwrite user changes without explicit authorization or a clearly safe reversible path.

Git is code-state memory, not complete project memory. Complete continuity is `Git + Project Memory + Tests + Evidence`.

Production-impacting and external actions require explicit authorization when applicable: production deployment, DNS/Nginx changes, destructive DB operations/migrations, GitHub push/force-push, repository/branch deletion, or irreversible infrastructure changes. Local development and verification come first.

## Priority
Prioritize approximately: critical security/active exploitation; data loss/corruption; authentication/authorization; production-breaking failures; critical regressions; major functional bugs; reliability; dependency/security maintenance; performance; UX; cleanup. Adjust by actual impact and task scope.

## Architecture Decisions and Runbooks
Use ADRs for consequential, durable, expensive-to-reverse decisions. Use runbooks for repeatable operational procedures. Read only those relevant to the task. Do not create process artifacts for trivial edits.

## End-of-Task Contract
Before declaring completion, confirm the intended change exists, relevant tests passed, result is verified, no known critical regression was introduced, important findings/security implications are documented, state surfaces are reconciled, the task/plan is closed or advanced, `NEXT ACTION` is clear, and a durable checkpoint exists.

The objective is not only to complete tasks. It is to make the project remember failures and decisions, preserve progress, protect previous fixes, and resume reliably from the last verified state.
