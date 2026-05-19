# Dependency Risk Register

## Current Classification

The corrected local security gate found no confirmed RustSec vulnerabilities, but dependency warnings remain tracked.

Known warning classes:

- unmaintained transitive dependencies
- unsoundness advisories in transitive dependencies
- deprecated GTK3-related bindings
- discontinued async ecosystem packages
- older serialization dependencies

## Policy

These warnings are not treated as immediate release blockers unless they become confirmed vulnerabilities or affect a reachable KGW code path.

They must remain visible and reviewed when:

- Rusty Kaspa bindings are updated
- Tauri dependencies are updated
- Cargo.lock changes
- Release assets are rebuilt

## Required Review Questions

For every warning:

1. Is the crate direct or transitive?
2. Is the affected API reachable from KGW?
3. Is there an upstream update?
4. Is the warning inherited from Rusty Kaspa, Tauri, or KGW?
5. Is a documented exception still justified?

## Current Action

Do not claim formal hardening until these warnings are either resolved or documented with specific package-level exceptions.
