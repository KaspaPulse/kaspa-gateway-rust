# Kaspa Gateway

Local-first Rust/Tauri desktop control plane for official Kaspa node and Stratum bridge runtimes.

Kaspa Gateway manages **runtime configuration, process ownership, status, and observability** for `mainnet`, `testnet10`, and experimental opt-in `testnet12`. It does **not** reimplement Kaspa consensus, `kaspad`, or Stratum bridge behavior.

## Download

**Current verified desktop build: `0.1.0`**  
Source: [`91046d16fa20a0b9a2b2b59ec9cac4f1db2594f4`](https://github.com/KaspaPulse/kaspa-gateway-rust/commit/91046d16fa20a0b9a2b2b59ec9cac4f1db2594f4)

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
841daaabd8d2802e4866a6846df5b13b5b75625ffe9fb4c7a72f9915da6c4690  KASPA_GATEWAY_WINDOWS_X64_NSIS_0.1.0_91046d1.exe
ce59f5e42207b5bdba309d1f497ca8af9673e5bf843802c01d68b2f3e3c4492d  KASPA_GATEWAY_MACOS_UNIVERSAL_DMG_0.1.0_91046d1.dmg
6b6df3a3fd81513b28a34315601d3fec722c7d1ab11086782db5a07d80df3aee  KASPA_GATEWAY_MACOS_UNIVERSAL_APP_0.1.0_91046d1.zip
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
| `testnet12` | **Experimental, explicit opt-in** | Dedicated TN12 binding |

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

## Quick start for developers

Requirements: Rust `1.97.1`, Node.js `24`, npm `11.17.0`, Git, and the native prerequisites required by Tauri for your platform.

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
docs/                         Architecture, operations, security and AI workflow docs
tools/                        Repository-native contract and quality gates
.github/workflows/            CI, security and desktop artifact workflows
```

## Documentation

- [`AGENTS.md`](AGENTS.md) — repository engineering and ownership rules
- [`docs/architecture/`](docs/architecture/) — runtime architecture and contracts
- [`docs/operations/`](docs/operations/) — operational documentation
- [`docs/security/`](docs/security/) — security documentation
- [`docs/AI_DEVELOPMENT_WORKFLOW.md`](docs/AI_DEVELOPMENT_WORKFLOW.md) — AI-assisted development workflow
- [`SECURITY.md`](SECURITY.md) — security policy

## Desktop artifacts

Native Windows and macOS packages are built by [`.github/workflows/desktop-artifacts.yml`](.github/workflows/desktop-artifacts.yml) against an explicit commit SHA. The workflow qualifies the installer/application before publishing workflow artifacts.

The current verified dual-platform artifact run is [`31873904088`](https://github.com/KaspaPulse/kaspa-gateway-rust/actions/runs/31873904088).

## License

`MIT OR Apache-2.0` — see [`LICENSE-MIT`](LICENSE-MIT) and [`LICENSE-APACHE`](LICENSE-APACHE).
