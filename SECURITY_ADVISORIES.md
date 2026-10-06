# Security Advisories Review Log

This file records intentionally accepted or currently unavoidable RustSec and npm dependency findings in Kaspa Gateway Rust.

## Policy

- Never suppress a RustSec finding without documenting the dependency path, reason, and removal condition.
- Prefer upgrading or removing the vulnerable dependency before adding an exception.
- Do not replace the official stable Kaspa SDK with an unvalidated floating branch solely to silence an advisory.
- Remove an exception as soon as the upstream dependency path permits a compatible fixed version.
- Keep `unmaintained` and `unsound` informational advisories visible in `cargo audit`.
- CI rejects ignored RustSec IDs that are not documented here and rejects stale review dates. cargo-deny also warns when an ignore is not encountered in its active dependency graph; that warning is reviewed rather than promoted to an error because cargo-audit scans the broader Cargo.lock set.
- npm exceptions must be exact, machine-readable, time-bounded, and fail closed on advisory identity, dependency-path, severity, lockfile, or deprecation drift.
- Do not use unsupported major-version npm overrides solely to silence a warning when the owning upstream package does not support that dependency line.

Last automated review: **2026-10-03**

Last npm dependency-policy review: **2026-10-06**; next mandatory review: **2026-10-10**.

### 2026-10-03 Rust dependency review evidence

Fresh local review on the exact Stage B worktree ran `cargo audit` and `cargo deny check`; both exited 0. `cargo audit` reported no unignored vulnerability failure and kept 16 allowed informational warnings visible. The observed warning IDs were RUSTSEC-2025-0052, RUSTSEC-2024-0375, RUSTSEC-2025-0141, RUSTSEC-2024-0388, RUSTSEC-2024-0384, RUSTSEC-2020-0168, RUSTSEC-2024-0436, RUSTSEC-2024-0370, RUSTSEC-2025-0081, RUSTSEC-2025-0075, RUSTSEC-2025-0080, RUSTSEC-2025-0100, RUSTSEC-2025-0098, RUSTSEC-2021-0145, RUSTSEC-2026-0306, and RUSTSEC-2024-0429. These remain visible rather than being silently suppressed. `cargo deny check` also exited 0; its warnings remain reviewable policy output rather than hidden failures.

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

### RUSTSEC-2026-0306 — `faster-hex` 0.9.0

Status: transitive informational soundness finding observed in the 2026-10-03 review.

RustSec reports that the `hex_decode_unchecked` AVX2 path can read past the source buffer. The finding remains visible in `cargo audit` and is not added to the ignored vulnerability set. The current repository review does not claim that every downstream call exercises the affected unchecked AVX2 path.

Action: track the owning upstream dependency path and upgrade/remove `faster-hex` as soon as the supported dependency graph permits it. Do not silence the finding merely to obtain a clean warning count.


## npm accepted temporary findings

### GHSA-73rr-hh4g-fpgx — `diff` 7.0.0 via Mocha 11.8.0

Status: temporary upstream/transitive Low-severity exception, review required by 2026-10-10.

The supported E2E path is `@wdio/mocha-framework` 9.31.7 -> Mocha 11.8.0 -> `diff` 7.0.0. npm audit represents the single root advisory as three Low vulnerability nodes (`diff`, `mocha`, and `@wdio/mocha-framework`). WebdriverIO 9.31.7 is the current reviewed 9.31.x line and declares Mocha `^11.8.0`; that Mocha line declares `diff ^7.0.0`, for which no patched compatible 7.x release is available at this review boundary.

Action: keep the exact exception in `docs/security/npm-dependency-policy.json`, fail on any drift, and remove it as soon as WebdriverIO/Mocha provide a supported patched path. Do not force `diff` 8/9 into Mocha 11 outside its declared range merely to silence npm audit.

## npm deprecated transitive packages under watch

The E2E install currently emits exactly two accepted upstream deprecation warnings: `glob` 10.5.0 and `whatwg-encoding` 3.1.1. The reviewed WebdriverIO 9.31.7 stack still declares `glob ^10.2.2`, while WebdriverIO 9.31.7 reaches Cheerio 1.2.0 -> `encoding-sniffer` 0.2.1 -> `whatwg-encoding` 3.1.1. The policy gate rejects any additional deprecation and expires these exceptions on 2026-10-10.

Action: remove each deprecation exception immediately when the supported upstream dependency graph no longer emits it.

## npm findings remediated

### GHSA-r4xh-jqrq-34v2 — `smol-toml`

Status: remediated on 2026-10-06.

The final local release security precheck detected `smol-toml` 1.7.1 as a new Moderate-severity transitive finding through `@wdio/native-utils` 2.5.0. The owning package declares `smol-toml ^1.6.0`, so pinning the compatible fixed release 1.9.0 in the E2E overrides removes the Moderate finding without changing WebdriverIO or product runtime semantics. The dependency policy now requires `node_modules/smol-toml` to remain exactly 1.9.0 and will fail closed on lockfile regression.

### GHSA-2883-xcg3-v3hh — `js-yaml`

Status: remediated on 2026-09-10.

PR #76 CI exposed `js-yaml` 4.3.1 as a High-severity E2E transitive finding through WebdriverIO/Mocha. Updating the supported WebdriverIO 9.31 line to exact 9.31.7 resolves `js-yaml` to 4.3.2, after which npm audit reports 0 Critical, 0 High, and 0 Moderate findings. The same update removes the previous `glob` 8 and `inflight` deprecation warnings.

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
01b532e8b553523216471682649693af92f0fd16  # v2.1.0
```

The experimental Testnet 13 runtime remains separately pinned to the official kaspanet/rusty-kaspa dagknight branch revision and must not be silently substituted for the stable mainline source.

## Review checklist

Before release and whenever the Kaspa SDK changes:

```bash
cargo run --locked -p xtask -- check-security-advisories --max-age-days 45
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
cargo audit
cargo deny check
cargo machete
cargo tree --locked -d
```
