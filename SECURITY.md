# Security Policy

## Supported Repository

This project is maintained at:

- Repository: KaspaPulse/kaspa-gateway-rust
- Public release tag: desktop-v0.1.0-20260519-165127
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

The project currently claims a repository and release security baseline, not formal hardening certification.

Required repository quality and security checks include:

- Rust 1.97.1 formatting with `cargo fmt --all -- --check`
- Locked workspace compilation with `cargo check --locked --workspace --all-targets`
- Clippy across the workspace with warnings denied
- Rust workspace tests
- Node.js 24 dependency installation from lockfiles
- ESLint 10 checks for the desktop frontend and E2E packages
- JavaScript syntax checks used by the project
- `npm audit` at the configured severity threshold for both npm packages
- CodeQL analysis for Rust
- Full-history TruffleHog secret scanning
- Pull-request dependency vulnerability and license review
- RustSec auditing with `cargo audit`
- Dependency policy enforcement with `cargo deny`
- Unused Rust dependency analysis with `cargo machete`
- GitHub Actions workflow validation with `actionlint`
- Dependabot update monitoring for Cargo, npm, GitHub Actions, and the Rust toolchain
- Third-party GitHub Actions pinned to immutable commit SHAs
- Project-specific KGW contract, locale, runtime, and release checks where applicable
- Release executable metadata review and Windows GUI subsystem verification for portable release artifacts

These automated checks reduce preventable regressions and dependency risk. They do not replace focused security review, threat modeling, penetration testing, or release-specific validation.

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

Dependency and license exceptions must remain narrow, documented, and limited to reviewed cases. Permissive-license allow-list entries do not waive vulnerability checks.

## Release Asset Expectations

Release assets must include:

- Windows installer
- Timestamped installer copy
- Portable executable
- INSTALL.txt
- SHA256SUMS.txt

Code signing is recommended for future releases. Unsigned Windows executables may trigger SmartScreen warnings even when the build is clean.

## Not Yet Claimed

This project does not yet claim:

- Formal penetration-test certification
- Third-party security audit completion
- Cryptographic reproducible build certification
- Full hardening against all local malware or compromised dependencies
- Signed Windows binaries

Acceptable public wording:

- The release passed the KGW repository and release security baseline checks applicable to that build.

Do not claim:

- Fully hardened.
- Impossible to hack.
- Penetration-tested.
- Certified secure.
