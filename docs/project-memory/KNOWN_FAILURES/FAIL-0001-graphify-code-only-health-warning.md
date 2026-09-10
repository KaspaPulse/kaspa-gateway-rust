# FAIL-0001: Graphify code-only graph health warning

- Status: VERIFIED
- Date: 2026-09-10
- Category: KNOWN FAILURE
- Affected scope: local generated `graphify-out/graph.json`; repository code navigation only.

## Evidence
The original `graphify extract . --code-only --no-cluster` diagnostic exposed raw pre-build edges and reported dangling/collapse candidates. After upgrading Graphify from 0.9.32 to 0.9.57 and rebuilding the normal final graph with `graphify extract . --code-only --force`, `graphify diagnose multigraph --graph graphify-out/graph.json --json` reported 5,017 nodes, 12,834 edges, and zero missing endpoints, dangling endpoints, self-loops, exact duplicates, or directed/undirected same-endpoint collapse candidates.

## Root Cause
CONFIRMED. The earlier warning was produced by diagnosing the `--no-cluster` raw extraction as if it were the final consumer graph. Raw extraction intentionally retains unresolved external/stdlib import/dependency references and parallel relation candidates before the normal build filters or resolves them. Graphify 0.9.57 source explicitly treats edges to absent external/stdlib nodes as expected and drops them during build. Its MultiDiGraph compatibility module also states that opt-in `--multigraph` is a future capability, not a current Kaspa Gateway requirement.

## Fix / Decision
Use the normal post-build Graphify graph for repository health claims. Keep raw `--no-cluster` diagnostics as extractor-development evidence only; do not classify their expected external references as a Kaspa Gateway defect. Graphify was upgraded locally to 0.9.57 and Git hooks were refreshed after the upgrade.

## Verification
The final post-build graph diagnostic is clean: all endpoint/collapse counters are zero. A focused Graphify query resolves the new continuity gate, regression test, `ACTIVE_TASK.md`, `CURRENT_STATE.md`, handoff ledger, and project-memory nodes. The Graphify MultiDiGraph runtime capability probe also passes on Python 3.12.3 / NetworkX 3.6.1.

## Regression Protection
Continue running a normal code-only Graphify refresh plus a post-change query for programming/tooling changes. When raw `--no-cluster` extraction is intentionally inspected, label it as pre-build data and do not conflate expected external references with final graph corruption.

## Remaining Risk
NONE identified for the final generated graph at this verification boundary. Future Graphify or source-topology changes require fresh verification.

## NEXT ACTION
CLOSED. Reopen this stable ID only if a future normal post-build graph diagnostic reports material endpoint/collapse corruption or code navigation demonstrably loses required relationships.

## DO NOT REPEAT
Do not diagnose a raw `--no-cluster` extraction and report its expected unresolved external references as final graph health without first building and checking the normal consumer graph.
