# Security Advisories Review Log

This file records intentionally accepted or currently unavoidable RustSec findings in Kaspa Gateway Rust.

## Policy

- Never suppress a RustSec finding without documenting the dependency path, reason, and removal condition.
- Prefer upgrading or removing the vulnerable dependency before adding an exception.
- Do not replace the official stable Kaspa SDK with an unvalidated floating branch solely to silence an advisory.
- Remove an exception as soon as the upstream dependency path permits a compatible fixed version.
- CI rejects ignored RustSec IDs that are not documented here and rejects a review date older than 45 days.

Last automated review: **2026-08-08**

## Current managed findings

### RUSTSEC-2026-0194 and RUSTSEC-2026-0195 — `quick-xml` 0.39.x

Status: temporary upstream/transitive exception.

The resolved dependency path is owned by the official `rusty-kaspa` v2.0.1 dependency graph (including wallet/RPC components). The advisories are fixed in `quick-xml` 0.41+, but forcing that semver-incompatible line locally would bypass upstream compatibility constraints.

The mainline Rusty Kaspa dependencies in this repository are pinned to commit `cfafeb4c093fa37a303f1b9f19c58f986b870ce3`, which is exactly the official `v2.0.1` release tag.

Action: remove these exceptions as soon as a stable Rusty Kaspa release resolves the dependency to a fixed compatible `quick-xml` version. Re-review immediately whenever the Kaspa SDK pin changes.

### RUSTSEC-2026-0235 — `rkyv` 0.7.x

Status: temporary upstream/transitive exception.

The resolved path is through `workflow-core` / Rusty Kaspa. The fixed `rkyv` line is 0.8.17+, which is a breaking major-version transition from the 0.7 API used by the upstream stack and cannot be safely forced from this application repository.

Action: remove this exception when the stable Rusty Kaspa/workflow dependency stack adopts a compatible fixed `rkyv` line.

## Findings remediated during the 2026-08-08 baseline

The security baseline initially detected seven RustSec vulnerabilities. Dependency refreshes and a direct `printpdf` upgrade eliminated the vulnerable `crossbeam-epoch`, `lopdf`, `quinn-proto`, and `ruint` paths. `printpdf` was upgraded from 0.9.1 to 0.12.5 so the PDF stack no longer resolves vulnerable `lopdf` 0.39.x.

The three findings above remain visible, explicitly managed, and time-bounded rather than represented as a zero-advisory state.

## Git dependency policy

Production Git dependencies must use immutable release tags or commit revisions from reviewed upstream sources. Floating production branches are not accepted when an immutable stable release exists.

Approved mainline Kaspa source:

```text
https://github.com/kaspanet/rusty-kaspa.git
```

Current approved mainline revision:

```text
cfafeb4c093fa37a303f1b9f19c58f986b870ce3  # v2.0.1
```

The Testnet-12 compatibility fork remains separately pinned to its reviewed immutable revision and must not be silently substituted for the official mainline source.

## Review checklist

Before release and whenever the Kaspa SDK changes:

```bash
python3 scripts/check-security-advisories.py --max-age-days 45
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
cargo audit
cargo deny check
cargo machete
cargo tree --locked -d
```
