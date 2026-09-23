# Kaspa Gateway

Local-first Rust/Tauri desktop control plane for official Kaspa node and Stratum bridge runtimes.

Kaspa Gateway manages **runtime configuration, process ownership, status, and observability** for `mainnet`, `testnet10`, and experimental opt-in `testnet13`. It does **not** reimplement Kaspa consensus, `kaspad`, or Stratum bridge behavior.

## Download

**Current published verified desktop build: `0.1.1`**
Source: [`b911eb44619f8eab706bc2fe786d1c84ac958f1d`](https://github.com/KaspaPulse/kaspa-gateway-rust/commit/b911eb44619f8eab706bc2fe786d1c84ac958f1d)

[**Download the verified desktop release**](https://github.com/KaspaPulse/kaspa-gateway-rust/releases/latest)

| Platform | Package | Qualification |
|---|---|---|
| Windows x64 | NSIS `.exe` | Build + installer smoke: **PASS** |
| macOS Intel + Apple Silicon | Universal `.dmg` | Build + DMG/app smoke: **PASS** |
| macOS Intel + Apple Silicon | Universal `.app.zip` | `arm64` + `x86_64` verified |

> Windows is currently unsigned. macOS uses ad-hoc signing and is not notarized. OS trust warnings are therefore expected for this developer release.

<details>
<summary><strong>SHA-256 checksums</strong></summary>

```text
eff90f265fe782fb186f5d79950a2b0c42ff463ad36fcdf2675526a4d4133082  KASPA_GATEWAY_WINDOWS_X64_NSIS_0.1.1_b911eb4.exe
ba5150b92860ca316e85e5fa90d27c60f2e15b0a7f0c6beed8ac970eb18b6387  KASPA_GATEWAY_MACOS_UNIVERSAL_DMG_0.1.1_b911eb4.dmg
82195836a67e143bf5b4d085bf42b7491d5332b05ac19cb79980df39aa5a8e9f  KASPA_GATEWAY_MACOS_UNIVERSAL_APP_0.1.1_b911eb4.zip
```

</details>

## What it does

- Starts and stops the official Kaspa node runtime per network.
- Starts and stops the official Stratum bridge runtime per network.
- Keeps Node and Bridge runtime ownership explicit and isolated.
- Applies effective runtime settings through typed Rust/Tauri paths.
- Shows runtime state, command previews, diagnostics, Explorer, and Analysis views.
- Displays **official runtime stdout/stderr only** in raw Node/Bridge log panes.

## Networks

| Network | Policy | Runtime family |
|---|---|---|
| `mainnet` | Stable / production behavior | Official mainline Rusty Kaspa |
| `testnet10` | Stable supported testnet | Official mainline Rusty Kaspa |
| `testnet13` | **Experimental, explicit opt-in** | Dedicated TN13 binding |

Runtime repository bindings and pinned revisions are defined in [`config/runtime-repository-bindings.json`](config/runtime-repository-bindings.json).

## Runtime safety model

```text
Official Kaspa runtime stdout/stderr
        ↓
Kaspa Gateway runtime owner
        ↓
Raw runtime buffer/event path
        ↓
Node / Bridge raw log pane
```

Raw runtime panes must not contain invented log lines, placeholders, or UI-only management messages. Status belongs in status surfaces; official stdout/stderr belongs in raw logs.

## Feedback and contributing

- Report reproducible bugs and enhancement requests through [GitHub Issues](https://github.com/KaspaPulse/kaspa-gateway-rust/issues).
- Report suspected vulnerabilities privately using [`SECURITY.md`](SECURITY.md).
- Read [`CONTRIBUTING.md`](CONTRIBUTING.md) before proposing code, dependency, test, workflow, or documentation changes.

## Quick start for developers

Requirements: Rust `1.98.1`, Node.js `24`, npm `11.17.0`, Git, and the native prerequisites required by Tauri for your platform.

```bash
git clone https://github.com/KaspaPulse/kaspa-gateway-rust.git
cd kaspa-gateway-rust/apps/kaspa-gateway-desktop
npm ci
npm run tauri -- dev --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

From the repository root:

```bash
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets
```

## Repository layout

```text
apps/kaspa-gateway-desktop/   Tauri desktop application
apps/kaspa-gateway-cli/       CLI application
crates/                       Runtime, config, API, DB, security and domain crates
config/                       Runtime repository bindings
docs/                         Architecture, ADRs, runbooks, operations and security docs
tools/                        Repository-native contract and quality gates
.github/workflows/            CI, security and desktop artifact workflows
```

## Documentation and continuity

- [`AGENTS.md`](AGENTS.md) — durable repository engineering, safety, and agent rules.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — public contribution process, coding standards, and testing expectations.
- [docs/USER_GUIDE.md](docs/USER_GUIDE.md) — installation, first-run, runtime operation, configuration, shutdown, and troubleshooting.
- [docs/EXTERNAL_INTERFACES.md](docs/EXTERNAL_INTERFACES.md) — GUI, CLI, files, managed runtime listeners, defaults, failures, and security boundaries.
- [`PROJECT_STATE.md`](PROJECT_STATE.md) — current verified resume boundary; always reconcile it against live Git/CI/release/runtime state.
- [`PLANS.md`](PLANS.md) — living plan for active multi-stage work.
- [`docs/architecture/`](docs/architecture/) — runtime architecture and contracts.
- [`docs/adr/`](docs/adr/) — canonical index/location for durable decisions; legacy ADR-0010 remains under `docs/architecture/` and is indexed there.
- [`docs/runbooks/`](docs/runbooks/) — repeatable release/operational procedures.
- [`docs/operations/`](docs/operations/) — focused operational documentation, including live-network smoke.
- [`docs/security/`](docs/security/) — security documentation.
- [`docs/AI_DEVELOPMENT_WORKFLOW.md`](docs/AI_DEVELOPMENT_WORKFLOW.md) — AI-assisted development and continuity workflow.
- [`SECURITY.md`](SECURITY.md) — security policy.

## Desktop artifacts

Native Windows and macOS packages are built by [`.github/workflows/desktop-artifacts.yml`](.github/workflows/desktop-artifacts.yml) against an explicit commit SHA. The workflow qualifies the installer/application before release staging or publication.

The current published immutable desktop release is [`desktop-v0.1.1`](https://github.com/KaspaPulse/kaspa-gateway-rust/releases/tag/desktop-v0.1.1), source `b911eb44619f8eab706bc2fe786d1c84ac958f1d`, qualified by Desktop Artifacts run [`31910163486`](https://github.com/KaspaPulse/kaspa-gateway-rust/actions/runs/31910163486). Release assets carry SHA-256 checksums and preserved Sigstore/SLSA build-provenance bundles. Candidate metadata or a successful artifact build does not become a public release until GitHub Releases reports it as published.

## License

`MIT OR Apache-2.0` — see [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
