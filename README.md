# Kaspa Gateway

Kaspa Gateway is a local-first Rust/Tauri desktop control plane for the official Kaspa node and Stratum bridge runtimes. It owns configuration, process lifecycle, status, observability, local data workflows, and desktop orchestration; it does **not** replace Kaspa consensus, `kaspad`, or the bridge runtime.

## Rust ownership

The repository enforces a strict language policy for project-owned programming logic:

```text
NON_RUST_OWNED_PROGRAMMING_SOURCE_COUNT=0
NON_RUST_EXECUTION_REFERENCE_COUNT=0
RUST_POLICY_GUARD=PASS
OWNED_PROGRAMMING_IMPLEMENTATION=100_PERCENT_RUST
```

The desktop WebView still requires HTML/CSS and deterministic JavaScript/WASM adapters. Those adapters are generated from Rust owners or are tool/platform-required configuration; CI rejects manual drift. Derived WASM binaries are built from source and are not stored as source artifacts.

## Downloads

Use the latest verified public desktop release:

https://github.com/KaspaPulse/kaspa-gateway-rust/releases/latest

Desktop release targets:

| Platform | Package |
|---|---|
| Windows x64 | NSIS `.exe` |
| macOS Intel + Apple Silicon | Universal `.dmg` and `.app.zip` |

Release sets include checksums and preserved Sigstore/SLSA provenance evidence. Windows Authenticode and Apple Developer ID/notarization are separate trust layers and are reported exactly as configured for each release.

## Networks

- **mainnet** — stable
- **testnet10** — stable supported testnet
- **testnet13** — experimental, explicit opt-in

See [docs/USER_GUIDE.md](docs/USER_GUIDE.md) for the current runtime, port, configuration, and user workflow details.

## Architecture

The workspace uses Rust 2024 edition with canonical build/release toolchain Rust 1.99.0 and MSRV Rust 1.97.1.

Major ownership surfaces include:

- Rust/Tauri desktop backend
- Rust runtime/process ownership and lifecycle
- Rust node and bridge integration
- Rust/WASM frontend implementation
- Rust/WASM E2E implementation
- Rust-owned CI/release orchestration through `xtask`
- deterministic generated WebView and test adapters
- pinned, declarative GitHub Actions adapters

Architecture and runtime contracts are documented in [docs/architecture/README.md](docs/architecture/README.md).

## Build and verify from source

Prerequisites vary by platform. The repository CI is the authoritative build/qualification contract.

Basic Rust verification:

```bash
cargo check --locked --workspace --all-targets
cargo test --locked --workspace
cargo run --locked -p xtask -- language-policy strict
```

Frontend/E2E generated-contract verification requires the pinned WASM toolchain used by CI, including `wasm-pack 0.15.0` and the `wasm32-unknown-unknown` Rust target.

Desktop npm dependencies are lockfile-managed under `apps/kaspa-gateway-desktop/`; E2E dependencies are lockfile-managed under `e2e/`.

## Security and supply chain

The repository uses, among other controls:

- Rust formatting, locked compilation, tests, and Clippy with warnings denied
- RustSec auditing, `cargo deny`, and unused-dependency analysis
- npm audit and project-specific dependency policy gates
- GitHub dependency review
- CodeQL for Rust
- TruffleHog secret scanning
- ClusterFuzzLite
- pinned GitHub Actions SHAs
- workflow contract validation
- deterministic generated-code checks
- release checksums, SBOMs, and provenance attestations

See [SECURITY.md](SECURITY.md) and [docs/security/SECURITY_BASELINE.md](docs/security/SECURITY_BASELINE.md).

## Documentation

- [User Guide](docs/USER_GUIDE.md)
- [External Interfaces](docs/EXTERNAL_INTERFACES.md)
- [Architecture](docs/architecture/README.md)
- [Security Baseline](docs/security/SECURITY_BASELINE.md)
- [Desktop Release Runbook](docs/runbooks/desktop-release.md)
- [Contributing](CONTRIBUTING.md)

## Repository hygiene

Generated build directories, local validation artifacts, and derived WASM binaries are intentionally excluded from source control. Product/release artifacts are produced by CI from exact source commits rather than committed as binary source artifacts.

## License

Licensed under either of:

- Apache License 2.0 — see [LICENSE-APACHE](LICENSE-APACHE)
- MIT License — see [LICENSE-MIT](LICENSE-MIT)
