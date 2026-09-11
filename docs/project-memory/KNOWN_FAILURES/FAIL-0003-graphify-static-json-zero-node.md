# FAIL-0003: Graphify static JSON zero-node warning

- Status: VERIFIED
- Date: 2026-09-11
- Category: KNOWN_FAILURE
- Affected scope: local Graphify code-only extraction only; no application/runtime behavior.

## Evidence
`graphify update .` on the security-hygiene branch reported 19 accepted source files that produced zero nodes. Reproducing extraction with Graphify 0.9.57 identified the set as static JSON data/config: 14 desktop i18n JSON files plus generated Tauri schema data, runtime repository binding data, the npm dependency policy JSON, and the E2E capability JSON. Package lockfiles were not part of the emitted warning set.

## Root Cause
Graphify's JSON extractor accepts these tracked JSON files as source candidates, but their structures intentionally contain no code/package nodes for the code graph. They were therefore retried on every update under Graphify issue class #1666 even though the files are valid and are not intended to contribute AST/code topology.

## Fix / Decision
Extend `.graphifyignore` only for the exact static JSON data/config paths that produce zero nodes. Do not broadly ignore JSON, JavaScript, Rust, workflow YAML, Cargo manifests, or other graph-bearing sources. Repository-native validators remain authoritative for the ignored configuration files.

## Verification
A second `graphify update .` completed with no warning. `graphify diagnose multigraph` remained clean with zero missing endpoints, dangling endpoints, self-loops, exact duplicate edges, and same-endpoint collapse candidates. The final graph remained populated (5,141 nodes / 12,839 edges at this checkpoint).

## Regression Protection
`.graphifyignore` now documents the exact data/config exclusions. Any future zero-node warning must be investigated before adding another pattern; the rule is to prove the file is intentionally non-graph-bearing rather than silence extractor errors generically.

## Remaining Risk
Graphify may classify future new static JSON files as source and surface the same warning. This is informational unless the file should have contributed code topology; investigate each new path individually.

## NEXT ACTION
No action for the currently identified files. Reopen this stable ID only if Graphify reports new persistent zero-node paths or an ignored path becomes graph-bearing source code.

## DO NOT REPEAT
Do not add broad `*.json` or directory-wide ignores without proving all affected files are non-graph-bearing. Do not treat a clean diagnostic as permission to suppress parser errors in real source files.
