# Security Policy

## Supported Repository

This project is maintained at:

- Repository: `KaspaPulse/kaspa-gateway-rust`
- Public release tag: `desktop-v0.1.0-20260519-165127`
- Desktop target: Windows x64
- Runtime model: KGW same-exe parallel self-worker runtime

## Reporting a Vulnerability

Please report suspected security issues privately.

Preferred contact:

- GitHub Security Advisories, if enabled for this repository
- Email: kaspapulse@gmail.com

Please include:

- A clear description of the issue
- Reproduction steps
- Affected commit, release asset, or configuration
- Whether the issue requires local access, network access, miner input, RPC exposure, malicious dependency input, or user interaction
- Logs, screenshots, or proof-of-concept details if available

Do not open public issues for active vulnerabilities until a fix or mitigation is available.

## Current Security Baseline

The project currently claims only a local security baseline, not formal hardening certification.

Required baseline checks:

- `kgw_i18n_contract_gate.cjs`
- `kgw_i18n_locale_coverage_gate.cjs`
- `kgw_parallel_self_worker_runtime_gate.cjs`
- `cargo fmt --all -- --check`
- `cargo check -p kaspa-gateway-desktop --no-default-features --features "official-kaspa-runtime-all rkstratum_cpu_miner"`
- `npm audit --audit-level=moderate`
- tracked source secret-pattern scan
- release executable metadata review
- Windows GUI subsystem verification for the portable executable

## Runtime Security Rules

KGW must remain a UI/settings/argument-builder/start-stop-status/raw-log viewer around selected Kaspa runtime bindings.

KGW must not:

- Store wallet seed phrases or private keys
- Request administrator rights for normal operation
- Hide generated runtime processes from the user
- Replace official Kaspa node or bridge behavior with undocumented KGW behavior
- Print fabricated runtime log lines in raw log panes
- Add duplicate runtime control layers that bypass existing owners

## Dependency Risk Policy

RustSec warnings are tracked separately from confirmed vulnerabilities.

A dependency warning does not automatically prove that KGW is exploitable, but it must remain visible until one of these is true:

- Upstream dependency is updated
- Affected code path is proven unreachable
- Dependency is removed
- A documented exception is accepted with justification

## Release Asset Expectations

Release assets must include:

- Windows installer
- Timestamped installer copy
- Portable executable
- `INSTALL.txt`
- `SHA256SUMS.txt`

Code signing is recommended for future releases. Unsigned Windows executables may trigger SmartScreen warnings even when the build is clean.

## Not Yet Claimed

This project does not yet claim:

- Formal penetration-test certification
- Third-party security audit completion
- Cryptographic reproducible build certification
- Full hardening against all local malware or compromised dependencies
- Signed Windows binaries

Acceptable public wording:

- The release passed KGW local security baseline checks.

Do not claim:

- Fully hardened.
- Impossible to hack.
- Penetration-tested.
- Certified secure.
