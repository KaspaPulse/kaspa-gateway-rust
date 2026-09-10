# FAIL-0001: Graphify code-only graph health warning

- Status: DEFERRED
- Date: 2026-09-10
- Category: KNOWN FAILURE
- Affected scope: local generated `graphify-out/graph.json`; repository code navigation only.

## Evidence
`graphify extract . --code-only --no-cluster` succeeded with 4,636 nodes and 12,718 raw edges. `graphify diagnose multigraph --graph graphify-out/graph.json` reported 0 missing-endpoint edges, 370 dangling-endpoint edges, 10 directed same-endpoint collapse candidates, and 15 undirected candidates. A focused post-change query still resolved `kgw_project_continuity_gate.cjs`, `activeTask`, `currentState`, `handoffRecords`, `memoryRecords`, and `readMarkdownRecords()`.

## Root Cause
NOT YET CONFIRMED. The warning is in generated code-graph topology and is not evidence of an application/runtime defect. The first diagnostic invocation also passed `--extract-path .` incorrectly and failed with `Errno 21`; the corrected command above produced the actual graph-health result.

## Fix / Decision
Do not block the continuity-lifecycle task on this advisory graph-quality warning because the changed tooling is present and queryable. Do not claim the generated graph is fully healthy.

## Verification
The corrected Graphify diagnostic completed and the focused post-change BFS query found the newly added continuity-gate symbols. Application/runtime tests are not inferred from Graphify.

## Regression Protection
Keep Graphify refresh plus post-change query in the repository workflow. When graph diagnostics are used, surface health warnings rather than silently treating graph generation as a full integrity PASS.

## Remaining Risk
Graph traversal may omit or collapse some relationships until the dangling/collapse warning is separately investigated.

## NEXT ACTION
DEFERRED — investigate Graphify dangling/collapse diagnostics only when graph-quality work is prioritized or when a code-navigation question depends on affected edges.

## DO NOT REPEAT
Do not rerun broad repository analysis merely to hide this warning; reuse this evidence unless Graphify/version/source topology changes materially.
