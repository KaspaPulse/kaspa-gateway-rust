# AI Development Workflow

This repository uses a local-first workflow for AI-assisted development. The root `AGENTS.md` defines mandatory policy. The Graphify skill at `.codex/skills/graphify/SKILL.md` defines the exact Graphify commands and must be read completely at the start of every programming task.

## Local-First Lifecycle

1. Capture the baseline with `git status --short`, the current branch, and the current commit.
2. Read `AGENTS.md` and `.codex/skills/graphify/SKILL.md`.
3. Query Graphify before broad raw source searches when `graphify-out/graph.json` exists.
4. Inspect only the files needed for the scoped change.
5. Make the smallest correct change.
6. Run targeted checks first, then broader checks only when the risk justifies them.
7. Refresh Graphify locally after source changes.
8. Query the affected flow again after the graph refresh.
9. Review all diffs before staging.
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

Do not use `--cargo`, semantic document extraction, document/PDF/image/video extraction, or a remote LLM backend for ordinary code work. Do not commit `graphify-out/cost.json`, cache data, local reflection files, or other transient Graphify working files.

## Codex Instruction Verification

Before editing workflow files, verify:

- `AGENTS.md` exists and has a single Graphify policy section.
- `.codex/skills/graphify/SKILL.md` exists.
- `.codex/hooks.json` is valid JSON.
- `.gitattributes` maps `graphify-out/graph.json` to the `graphify` merge driver.
- The repository Git config has a `merge.graphify.driver` entry.
- `graphify hook status` reports installed hooks when supported by the installed CLI.

The local gate `tools/kgw_ai_workflow_gate.ps1` checks the permanent instruction surface.

## Test Selection

Use the narrowest test set that covers the change:

- Workflow and PowerShell changes: run `powershell -NoProfile -ExecutionPolicy Bypass -File tools/kgw_ai_workflow_gate.ps1`.
- JavaScript tools: run `node --check <file>` and any relevant `tools/*.cjs` gate.
- Frontend behavior: run syntax checks for touched modules and the existing owner/i18n gates that cover the affected surface.
- Rust behavior: run targeted package or test names first.
- Cross-crate/runtime contracts: run `cargo test --workspace` when the environment can support the full workspace.
- Desktop build changes: confirm the package and binary, then run `cargo build --locked --bin kaspa-gateway-desktop` when relevant.

Do not change application behavior only to satisfy an unrelated test.

## Generated Graph Snapshot Policy

`graphify-out/graph.json`, `graphify-out/GRAPH_REPORT.md`, `graphify-out/graph.html`, `graphify-out/manifest.json`, and tracked Graphify analysis/label files are generated repository data. Keep them in a separate commit from application, documentation, or tooling changes.

Do not delete existing Graphify history. Do not stage generated cache, cost, local reflection, vocabulary, Python interpreter, or temporary extraction files.

## Local Commit Policy

Commits are local by default:

- Commit only after relevant checks pass.
- Keep workflow/tooling changes separate from generated Graphify snapshots.
- Use `[skip ci]` on local workflow commits.
- Do not stage unrelated pre-existing changes.
- Do not push, create pull requests, or run GitHub Actions unless the user explicitly requests it.

