# AI Development Workflow

This repository uses a local-first workflow for AI-assisted development. The root `AGENTS.md` defines mandatory policy. `PROJECT_STATE.md` is the canonical current-state handoff. The Graphify skill at `.codex/skills/graphify/SKILL.md` defines the exact Graphify commands and must be read completely at the start of every programming task.

## Session Start

1. Read `AGENTS.md`.
2. Read `PROJECT_STATE.md`.
3. Capture the live Git baseline with `git branch --show-current`, `git rev-parse HEAD`, `git status --short --branch`, and `git log -1 --oneline`.
4. If the verified repository, CI, release, or runtime reality differs from `PROJECT_STATE.md`, reconcile the state document before relying on the stale claim.
5. Read only the ADRs relevant to the task.
6. Read the relevant runbook before release publication, rollback/recovery, live-network smoke, or incident work.
7. Read `PLANS.md` when multi-stage work is active.
8. Treat conversation memory and historical handoffs as advisory context only.

## Local-First Lifecycle

1. Read `AGENTS.md`, `PROJECT_STATE.md`, and `.codex/skills/graphify/SKILL.md` as applicable.
2. Query Graphify before broad raw source searches when `graphify-out/graph.json` exists.
3. Inspect only the files needed for the scoped change.
4. Make the smallest correct change.
5. Run targeted checks first, then broader checks only when the risk justifies them.
6. Refresh Graphify locally after source changes.
7. Query the affected flow again after the graph refresh.
8. Review all diffs before staging.
9. Reconcile `PROJECT_STATE.md` after a meaningful state transition; record actual validation, blockers/drift, and a precise `NEXT ACTION`.
10. Commit locally only after relevant checks pass, and never push unless explicitly requested.

## Graphify Query-Before-Source Workflow

Use the existing graph as the first codebase map:

- Use `graphify query "<question>"` for broad architecture.
- Use `graphify query "<question>" --dfs` for execution chains and data flow.
- Use `graphify explain "<concept>"` for a specific node.
- Use `graphify path "<A>" "<B>"` to inspect relationships between two concepts.

If a result is broad or truncated, narrow the query before switching to raw source reads. If Graphify fails, capture the command and real error, then continue only with an explicit note that graph verification failed.

## Safe Incremental Graph Refresh

When `graphify-out/graph.json` exists, prefer the supported incremental update:

```powershell
graphify update .
```

When the graph does not exist, inspect installed help and use code-only extraction so project analysis does not require API tokens:

```powershell
graphify extract . --code-only
```

Do not use `--cargo`, semantic document extraction, document/PDF/image/video extraction, or a remote LLM backend for ordinary code work. Do not commit `graphify-out/cost.json`, cache data, local reflection files, vocabulary, Python interpreter, or temporary extraction files.

## Continuity and Instruction Verification

Before editing workflow or continuity files, verify:

- `AGENTS.md` exists and has a single Graphify policy section.
- `PROJECT_STATE.md` exists and contains a dynamic resume boundary rather than a permanent current-HEAD claim.
- `PLANS.md` is read when its active plan applies.
- `docs/adr/README.md` indexes durable decisions.
- `docs/runbooks/` contains the relevant repeatable operational procedure when one exists.
- `.codex/skills/graphify/SKILL.md` exists.
- `.codex/hooks.json` is valid JSON.
- `.gitattributes` maps `graphify-out/graph.json` to the `graphify` merge driver.
- The repository Git config has a `merge.graphify.driver` entry.
- `graphify hook status` reports installed hooks when supported by the installed CLI.

The local gate `tools/kgw_ai_workflow_gate.ps1` checks the permanent Graphify/instruction surface. The cross-session continuity contract is checked by:

```bash
node tools/kgw_project_continuity_gate.cjs
```

## Test Selection

Use the narrowest test set that covers the change:

- Continuity documentation: run `node --check tools/kgw_project_continuity_gate.cjs` and `node tools/kgw_project_continuity_gate.cjs`.
- Workflow and PowerShell changes: run `pwsh -NoProfile -ExecutionPolicy Bypass -File tools/kgw_ai_workflow_gate.ps1` when PowerShell 7 is available, plus `actionlint` for workflow YAML.
- JavaScript tools: run `node --check <file>` and any relevant `tools/*.cjs` gate.
- Frontend behavior: run syntax checks for touched modules and the existing owner/i18n gates that cover the affected surface.
- Rust behavior: run targeted package or test names first.
- Cross-crate/runtime contracts: run `cargo test --workspace` when the environment can support the full workspace.
- Desktop build changes: confirm the package and binary, then run `cargo build --locked --bin kaspa-gateway-desktop` when relevant.

Do not change application behavior only to satisfy an unrelated test. Do not report a skipped or unavailable check as `PASS`.

## State Transition Rules

Update `PROJECT_STATE.md` when a change materially affects the resume boundary, including a significant merge, release/draft/publication, deployment/rollback, major CI result, blocker, architectural decision, runtime/environment state, or safely interruptible long-running task boundary.

Do not update it for formatting-only changes or every small commit. Do not duplicate Git history or issue-tracker details. Use:

- Git for code history.
- GitHub Actions for CI results.
- GitHub Releases for release state.
- runtime/observability evidence for actual process health.
- ADRs for durable decisions.
- runbooks for procedures.
- `PLANS.md` for active multi-stage work.

## Generated Graph Snapshot Policy

`graphify-out/graph.json`, `graphify-out/GRAPH_REPORT.md`, `graphify-out/graph.html`, `graphify-out/manifest.json`, and tracked Graphify analysis/label files are generated repository data. Keep them in a separate commit from application, documentation, or tooling changes.

Do not delete existing Graphify history. Do not stage generated cache, cost, local reflection, vocabulary, Python interpreter, or temporary extraction files.

## Local Commit Policy

Commits are local by default:

- Commit only after relevant checks pass.
- Keep workflow/tooling changes separate from generated Graphify snapshots.
- Use `[skip ci]` on local-only workflow commits; do not suppress CI on a PR that requires repository qualification.
- Do not stage unrelated pre-existing changes.
- Do not push, create pull requests, or run GitHub Actions unless the user explicitly requests it or the repository task explicitly requires PR-based integration.
