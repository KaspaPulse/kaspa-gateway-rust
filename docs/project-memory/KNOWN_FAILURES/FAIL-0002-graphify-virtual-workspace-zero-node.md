# FAIL-0002: Graphify virtual Cargo workspace zero-node retry warning

- Status: VERIFIED
- Date: 2026-09-11
- Category: KNOWN FAILURE
- Affected scope: Graphify local code-graph update only; root virtual `Cargo.toml`.

## Evidence
Graphify 0.9.57 repeatedly warned that the repository root `Cargo.toml` produced zero nodes. The final graph itself remained clean: zero missing/dangling endpoints, self-loops, exact duplicates, or same-endpoint collapse candidates.

## Root Cause
CONFIRMED. The root manifest is a Cargo virtual workspace and has no `[package]`. Graphify's own `manifest_ingest.py` intentionally returns no package node for such virtual workspace roots. Its zero-node retry path still warns on that intentional empty result (#1666).

## Fix / Decision
Add root-only `/Cargo.toml` to `.graphifyignore`. Do not ignore nested crate manifests; `crates/*/Cargo.toml` remain available to Graphify.
## Verification
A fresh `graphify update .` completed with no warnings after the root-only ignore. The graph diagnostic remained clean at 5,128 nodes / 12,907 edges, and ten nested `crates/*/Cargo.toml` manifests remain present in the repository.

## Regression Protection
Keep the ignore root-anchored. If the root manifest later gains a real `[package]`, remove the ignore and verify Graphify emits the package node without reintroducing warnings.

## Remaining Risk
NONE identified for the current virtual-workspace layout.

## NEXT ACTION
CLOSED. Reopen only if the root Cargo manifest becomes a real package or Graphify changes virtual-workspace manifest behavior.

## DO NOT REPEAT
Do not ignore every `Cargo.toml`, fabricate a root package node, or report the intentional virtual-workspace zero-node result as code-graph corruption.