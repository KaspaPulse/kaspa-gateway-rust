# Kaspa Gateway Agent Workflow

Kaspa Gateway is a Rust workspace with a Tauri desktop application and a JavaScript frontend for managing Kaspa node and bridge runtimes. All source code, scripts, filenames, identifiers, comments, developer documentation, diagnostics, and commit messages added to this repository must be written in English, except intentional existing UI translations.

## Repository Structure

- `crates/` contains the Rust workspace crates: core, config, security, rk-node, rk-bridge, API, DB, node, observability, and runtime.
- `apps/kaspa-gateway-desktop/src-tauri/` contains the Tauri desktop package, Rust commands, generated Tauri outputs, and Tauri integration tests. The desktop binary is `kaspa-gateway-desktop`.
- `apps/kaspa-gateway-desktop/frontend/` contains the JavaScript frontend, shell, tab modules, templates, CSS, and i18n catalogs.
- `apps/kaspa-gateway-cli/` contains CLI tools and runtime probes.
- `tools/` contains local gates, audits, repair scripts, and smoke tooling. Prefer existing gates before adding new ones.
- `tests/`, crate-level `tests/`, `src-tauri/tests/`, JavaScript gates, `Cargo.toml`, `Cargo.lock`, and Tauri config files define the validation surface.
- `graphify-out/` is generated repository graph data. Keep graph snapshots separate from application changes. Do not commit Graphify cache, cost, or local reflection data.
- `PROJECT_STATE.md` is the canonical current-state handoff. It summarizes verified facts but never replaces Git, CI, release metadata, or runtime evidence.
- `PLANS.md` is the living execution plan for active multi-stage work.
- `docs/adr/` is the canonical location for new architecture/process decision records. Existing historical ADRs under `docs/architecture/` remain valid and are indexed from `docs/adr/README.md`.
- `docs/runbooks/` contains operational procedures. Existing focused operational documents under `docs/operations/` remain authoritative for their stated scope.

## Session Start and Continuity

At the beginning of every new work session:

1. Read `AGENTS.md`.
2. Read `PROJECT_STATE.md`.
3. Inspect the actual repository before making changes. At minimum run `git branch --show-current`, `git rev-parse HEAD`, `git status --short --branch`, and `git log -1 --oneline`; inspect remotes when relevant.
4. If verified reality differs from `PROJECT_STATE.md`, reconcile the state document before relying on the stale claim.
5. Read only the ADRs relevant to the task when it touches an established architectural or process decision.
6. Read the relevant runbook before release publication, rollback/recovery, live-network smoke, or incident work.
7. Read `PLANS.md` when multi-stage work is active.
8. Treat conversation memory, handoff ZIPs, and old chat summaries as advisory context only. Verified repository, CI, release, and runtime state takes precedence.
9. Do not repeat completed work unless fresh verification shows it is incomplete or regressed.
10. Continue from `PROJECT_STATE.md` `NEXT ACTION` unless the user explicitly changes priority.

After every meaningful state transition, reconcile `PROJECT_STATE.md` with verified reality. Record what changed, what was actually validated, any active blocker or drift, and the precise next action. Keep the handoff compact; do not duplicate Git logs, CI history, issue trackers, release bodies, or secrets.

Create or update an ADR only for consequential decisions that are expensive to reverse or define a durable architecture/security/operational boundary. Use a runbook for repeatable operational procedures. Use `PLANS.md` for long-horizon, multi-stage, migration, or high-risk work. Do not create those artifacts for trivial edits.

## Source-of-Truth Hierarchy

Use the source that actually owns each fact:

1. Verified live runtime or deployment/release state for operational facts.
2. Git repository state for code, branches, commits, and worktree state.
3. CI/CD results for builds, tests, security gates, and recorded automation.
4. `PROJECT_STATE.md` as the current verified summary after reconciliation.
5. `AGENTS.md` for durable working rules.
6. Accepted ADRs for durable decisions.
7. Relevant runbooks for operational procedures.
8. `PLANS.md` for the current execution plan.
9. Release notes and historical documentation for history.
10. Conversation memory only as non-authoritative context.

Never convert `NOT VERIFIED` into `PASS`. Push, merge, CI success, release publication, and healthy runtime are distinct states and require distinct evidence.

## Mandatory Graphify Lifecycle

- At the start of every programming task, read `.codex/skills/graphify/SKILL.md` completely and follow it as the source of truth for Graphify commands.
- Before broad source analysis, check whether `graphify-out/graph.json` exists and whether the graph appears fresh enough for the task. Dirty generated graph files are not a reason to skip Graphify.
- Query Graphify before broad raw file searches: use `graphify query "<question>"` for scoped context, `graphify explain "<concept>"` for a focused node, and `graphify path "<A>" "<B>"` for relationships.
- Use breadth-first traversal for architecture discovery when supported. Use depth-first traversal for execution chains and data-flow traces when supported.
- If a result is broad or truncated, narrow the query or use supported explain, path, or node-inspection commands before falling back to raw source reads.
- Make the smallest correct source change, then run relevant tests for the touched surface.
- After source changes, run a supported local Graphify refresh. Use incremental update when `graphify-out/graph.json` exists. Use code-only extraction when the graph does not exist. Do not use Graphify Cargo introspection, semantic document extraction, document/PDF/image/video extraction, or any backend that requires an API key unless explicitly requested.
- After updating the graph, query the affected path again and use the graph result to verify that the changed flow is represented.
- Diagnose and report Graphify failures. Do not silently skip Graphify.

## Local-First Workflow

- Build, test, and run locally before remote operations.
- Do not push, create pull requests, run GitHub Actions, or require remote API tokens unless explicitly requested.
- Project testing and code-only Graphify extraction must not require API keys or paid LLM services.
- Do not commit before relevant tests pass.
- Do not stage unrelated user changes. Preserve pre-existing worktree changes and work with them instead of discarding or hiding them.

## Network Invariants

- Mainnet and testnet10 are stable networks that may be enabled.
- Testnet12 is experimental, disabled by default, and requires explicit opt-in before start or runtime ownership.
- Keep database directories, ports, runtime state, logs, and operating-system processes isolated by network.
- Maintain exactly one process owner per network. Do not create duplicate frontend, Tauri, self-worker, bridge, or node owners for the same runtime.
- Keep RPC bound to loopback by default.
- Never build shell command strings for process execution. Use structured process APIs and argument arrays.
- Validate executable paths, arguments, hosts, ports, network names, app/data directories, and output paths before use.

## Raw Process Log Invariants

- Display the real native process stdout and stderr.
- Never invent startup, synchronization, connection, readiness, or success messages.
- Never replace raw logs with summaries. Summaries may be additional UI, not a substitute for raw lines.
- Preserve native line ordering as closely as possible.
- Store network, process source, stream type, and receive time as separate metadata.
- Keep Copy Log, Clear Log, Auto-scroll, and log buffers independent per network.
- Surface the real diagnostic error when process startup fails.
- Do not report Start as successful unless the process actually started or the IPC response provides verifiable evidence.
- Surface spawn errors, permission failures, process crashes, panics, and nonzero exits honestly.

## Tauri Boundary

- The frontend must not start operating-system processes directly.
- Start, stop, settings, status, and log operations must cross a documented and tested Tauri IPC boundary.
- Validate every IPC payload at the Tauri boundary and in the runtime layer that consumes it.
- Preserve complete native errors across the IPC boundary.
- Do not manually edit generated Tauri schema files.
- Restore unrelated generated schema changes only when doing so does not discard intentional user work.

## Definition Of Done

No meaningful task is complete until the requested change is implemented, applicable validation has actually run, failures and skipped checks are disclosed, and `PROJECT_STATE.md` is reconciled when the project state materially changed. Record a durable decision in an ADR when applicable, update the relevant runbook when an operational procedure changed, and verify release/deployment/runtime state when that was part of the task.

- Run formatting and static validation for files you changed.
- Run relevant JavaScript checks and gates, including targeted `node --check` or existing `tools/*.cjs` gates when JavaScript or tooling changes.
- Run relevant targeted Rust tests before broad Rust tests when Rust behavior changes.
- Run `cargo test --workspace` when the change crosses crates or shared runtime contracts and the current environment can support it.
- Run `cargo build --locked --bin kaspa-gateway-desktop` when the desktop target is relevant and the package/binary names are confirmed.
- Run a short local smoke test for affected runtime behavior. Do not run long live-network tests unless explicitly requested.
- Run Graphify update and a post-change Graphify query for affected flows.
- Report every check honestly as passed, failed, or skipped with the exact command.

## Git Discipline

- Never use `git reset --hard`.
- Never discard, hide, or overwrite user changes with `git restore`, `git checkout`, or `git stash` unless the user explicitly asks for that exact operation.
- Keep commits small and scoped.
- Keep generated Graphify snapshots in a separate commit from documentation, tooling, or application changes.
- Use `[skip ci]` for local workflow commits.
- Do not push during local workflow tasks unless explicitly requested.
