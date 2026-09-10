# Dependency Risk Register

## Current Classification

The corrected security gates find no Critical/High/Moderate npm vulnerability in either npm workspace and no currently blocking RustSec vulnerability. Residual dependency warnings remain explicitly tracked rather than hidden.

Current npm classification (2026-09-10):

- desktop: 0 Critical/High/Moderate/Low npm vulnerabilities and 0 deprecation warnings
- E2E: 0 Critical/High/Moderate; 3 Low npm audit nodes from the single GHSA-73rr-hh4g-fpgx upstream chain
- E2E deprecations: `glob` 10.5.0 and `whatwg-encoding` 3.1.1, both time-bounded to a mandatory review by 2026-10-10
- machine-readable npm exception contract: `docs/security/npm-dependency-policy.json`

Known Rust warning classes:

- unmaintained transitive dependencies
- unsoundness advisories in transitive dependencies
- deprecated GTK3-related bindings
- discontinued async ecosystem packages
- older serialization dependencies

## Policy

Warnings are not treated as immediate release blockers merely because they are transitive or deprecated. Confirmed Critical/High/Moderate npm vulnerabilities are blocking. Low npm findings may be accepted only by exact advisory/path/version contract with an expiry; any new or drifting npm warning fails closed. Rust findings retain their documented reachability/upstream review policy.

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

The current E2E residual npm risk is documented in `SECURITY_ADVISORIES.md`, `SEC-0002`, and `docs/security/npm-dependency-policy.json`. CI must keep that contract fail-closed and force re-review no later than 2026-10-10. Do not broaden exceptions merely to keep CI green.
