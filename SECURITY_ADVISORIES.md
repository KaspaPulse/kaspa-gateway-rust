# Security Advisories Review Log

This file records intentionally accepted or currently unavoidable RustSec findings in Kaspa Gateway Rust.

## Policy

- Never suppress a RustSec finding without documenting the dependency path, reason, and removal condition.
- Prefer upgrading or removing the vulnerable dependency before adding an exception.
- Do not replace the official stable Kaspa SDK with an unvalidated floating branch solely to silence an advisory.
- Remove an exception as soon as the upstream dependency path permits a compatible fixed version.
- Keep `unmaintained` and `unsound` informational advisories visible in `cargo audit`.
- CI rejects ignored RustSec IDs that are not documented here and rejects stale review dates. cargo-deny also warns when an ignore is not encountered in its active dependency graph; that warning is reviewed rather than promoted to an error because cargo-audit scans the broader Cargo.lock set.

Last automated review: **2026-08-15**

## Current managed findings

### RUSTSEC-2026-0235 — `rkyv` 0.7.x

Status: temporary upstream/transitive exception.

The resolved path is through `workflow-core` / Rusty Kaspa. The fixed `rkyv` line is 0.8.17+, which is a breaking major-version transition from the 0.7 API used by the upstream stack and cannot be safely forced from this application repository.

Action: remove this exception when the stable Rusty Kaspa/workflow dependency stack adopts a compatible fixed `rkyv` line.

## Tracked informational soundness findings

These findings are deliberately **not ignored**. They remain visible in `cargo audit` as informational `unsound` warnings.

### RUSTSEC-2021-0145 — `atty` 0.2.14

Status: upstream/transitive informational soundness finding.

The current Windows dependency graph reaches `atty` through `hexplay` and the official Rusty Kaspa/workflow stack. RustSec reports a potential unaligned read on Windows and no patched `atty` release.

Action: track removal/replacement in the upstream dependency path. Do not fork or replace the official Kaspa runtime solely to silence the informational advisory.

### RUSTSEC-2024-0429 — `glib` 0.18.5

Status: Linux-only Tauri/GTK3 transitive informational soundness finding.

The current Linux desktop graph reaches `glib` 0.18.5 through the Tauri/WebKitGTK/GTK3 stack. RustSec fixes this soundness issue in `glib` 0.20.0 and later. The 2026-08-15 target reconciliation did not place this affected 0.18.5 path in the Windows or macOS target graphs.

Action: track the supported Tauri/Linux platform migration path and remove the affected GTK3/glib line when that can be done without bypassing Tauri runtime ownership or platform compatibility.

## Findings remediated

### RUSTSEC-2026-0194 and RUSTSEC-2026-0195 — `quick-xml`

Status: remediated; previous exceptions retired on 2026-08-15.

`Cargo.lock` now resolves `quick-xml` to 0.41.0. Both advisories are fixed in `quick-xml` 0.41.0 and later, and neither advisory appears in the current `cargo audit` result or the current Scorecard RustSec set.

The stale ignore entries are therefore removed from both cargo-audit and cargo-deny.

### Findings remediated during the 2026-08-08 baseline

The security baseline initially detected seven RustSec vulnerabilities. Dependency refreshes and a direct `printpdf` upgrade eliminated the vulnerable `crossbeam-epoch`, `lopdf`, `quinn-proto`, and `ruint` paths. `printpdf` was upgraded from 0.9.1 to 0.12.5 so the PDF stack no longer resolves vulnerable `lopdf` 0.39.x.

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
