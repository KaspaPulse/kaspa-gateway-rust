# Kaspa Gateway Rust

> A Rust/Tauri desktop application for running, controlling, and observing Kaspa node and Stratum bridge runtimes across `mainnet`, `testnet10`, and `testnet12` from a single local desktop UI.

Kaspa Gateway Rust is a local-first desktop control plane for Kaspa infrastructure. It is designed to be a **UI, settings manager, argument builder, process/runtime owner, status monitor, and raw stdout/stderr log viewer** for official Kaspa runtimes. It is **not** a replacement implementation of Kaspa consensus, `kaspad`, or `kaspa-stratum-bridge`.

The core principle is simple:

```text
Official Kaspa runtime emits stdout/stderr
        ↓
KGW backend runtime owner captures the raw line
        ↓
KGW stores and streams the line
        ↓
Node/Bridge log panes display the raw official line
```

No invented log messages, no placeholder runtime lines, and no UI-only fake status lines should be mixed into raw runtime log panes.

---

## Table of contents

- [<!-- KGW_CURRENT_RELEASE_START -->
## Current verified desktop release

Current desktop version:

`0.1.0`

Current release tag prepared by this workflow:

`desktop-v0.1.0-20260523-053016`

Runtime bindings are pinned to explicit git revisions, not floating branches:

| Network | Runtime family | Repository | Branch | Pinned revision |
|---|---:|---|---|---|
| mainnet | mainline | https://github.com/kaspanet/rusty-kaspa.git | stable | `cfafeb4c093fa37a303f1b9f19c58f986b870ce3` |
| testnet10 | mainline | https://github.com/kaspanet/rusty-kaspa.git | stable | `cfafeb4c093fa37a303f1b9f19c58f986b870ce3` |
| testnet12 | tn12 (experimental) | https://github.com/LiveLaughLove13/rusty-kaspa.git | RKStratumTN12 | `eeb351ee911e2df906d21203dec8db3a195c6b33` |

Release validation includes:

- runtime repository binding audit
- global owner gate
- i18n contract gate
- i18n locale coverage gate
- JavaScript syntax checks
- cargo fmt check
- cargo check
- Tauri NSIS installer build

Release artifacts are published as GitHub Release assets and are not committed to Git.
<!-- KGW_CURRENT_RELEASE_END -->

Project status](#project-status)
- [Repository links and runtime bindings](#repository-links-and-runtime-bindings)
- [What the application does](#what-the-application-does)
- [Main features](#main-features)
- [Architecture overview](#architecture-overview)
- [Runtime ownership model](#runtime-ownership-model)
- [Node and bridge behavior](#node-and-bridge-behavior)
- [Raw log model](#raw-log-model)
- [Desktop UI overview](#desktop-ui-overview)
- [Internationalization](#internationalization)
- [Workspace layout](#workspace-layout)
- [Prerequisites](#prerequisites)
- [Quick start](#quick-start)
- [Development commands](#development-commands)
- [Quality gates](#quality-gates)
- [Release build](#release-build)
- [Configuration](#configuration)
- [Security posture](#security-posture)
- [Troubleshooting](#troubleshooting)
- [Maintainer rules](#maintainer-rules)
- [License](#license)

---

## Project status

Current application version:

```text
0.1.0
```

Current desktop product name:

```text
Kaspa Gateway
```

Current Rust workspace target:

```text
kaspa-gateway-desktop
```

Current recommended feature set for full local development and release builds:

```text
official-kaspa-runtime-all rkstratum_cpu_miner
```

The project currently includes:

- A Tauri v2 desktop shell.
- A static frontend UI served locally by Tauri.
- Rust backend commands for runtime control, settings, diagnostics, address book, explorer, and analysis workflows.
- Embedded official Rusty Kaspa runtime bindings through Cargo git dependencies.
- Multi-network Node and Bridge tabs for `mainnet`, `testnet10`, and `testnet12`.
- Raw official node and bridge log panes.
- Internationalized UI with gate-based translation validation.
- Windows NSIS installer support through Tauri bundling.

---

## Repository links and runtime bindings

### Main project repository

```text
https://github.com/KaspaPulse/kaspa-gateway-rust.git
```

### Runtime source repositories

Kaspa Gateway Rust uses official or explicitly configured Rusty Kaspa runtime sources. These bindings are treated as build-time ownership contracts and are centralized in:

```text
config/runtime-repository-bindings.json
```

| Network | Runtime family | Repository | Branch | Purpose |
|---|---:|---|---|---|
| `mainnet` | `mainline` | `https://github.com/kaspanet/rusty-kaspa.git` | `stable` | Official stable mainnet node and Stratum bridge runtime |
| `testnet10` | `mainline` | `https://github.com/kaspanet/rusty-kaspa.git` | `stable` | Official stable TN10 node and Stratum bridge runtime |
| `testnet12` | `tn12` | `https://github.com/LiveLaughLove13/rusty-kaspa.git` | `RKStratumTN12` | Experimental opt-in TN12 runtime |

### Important runtime policy

Kaspa Gateway Rust should not reimplement Kaspa node or bridge behavior. It should:

- Select the correct official runtime binding.
- Build the correct command/argument plan.
- Start and stop the selected runtime owner.
- Capture official stdout/stderr.
- Display official raw logs.
- Expose settings and status without duplicating Kaspa runtime logic.

---

## What the application does

Kaspa Gateway Rust is a local desktop control panel for Kaspa node and Stratum bridge operations. It helps a user:

- Configure node runtime options per network.
- Configure bridge runtime options per network.
- Start or stop each network independently.
- View raw official node logs.
- View raw official bridge logs.
- Inspect runtime status and command previews.
- Explore addresses and transactions.
- Run analysis views over explorer data.
- Manage address book data locally.
- Switch UI language and currency display.
- Build Windows installer and portable runner artifacts.

The application is intentionally designed as a **control and observability layer**, not as a consensus fork, wallet, miner pool, or hosted service.

---

## Main features

### Multi-network runtime management

Supported network slots:

```text
mainnet
testnet10
testnet12
```

Each network is handled as an independent runtime slot. Node and Bridge tabs must always treat all three networks consistently.

### Official Kaspa runtime bindings

The runtime layer binds to Rusty Kaspa crates through Cargo features and git dependencies:

- `official-kaspa-runtime-mainline`
- `official-kaspa-runtime-tn12`
- `official-kaspa-runtime-all`
- `rkstratum_cpu_miner`

### Node management

The Node tab is responsible for:

- Per-network settings.
- Start/Stop controls.
- Runtime status.
- Command preview.
- Raw official `kaspad` stdout/stderr log display.
- Per-network log behavior and auto-scroll.

### Bridge management

The Bridge tab is responsible for:

- Per-network bridge settings.
- Start/Stop controls.
- Stratum listen configuration.
- Node RPC attachment configuration.
- Bridge mode selection where supported.
- CPU miner options for test networks where compiled and enabled.
- Raw official `kaspa_stratum_bridge` stdout/stderr log display.
- Per-network log behavior and auto-scroll.

### Raw log panes

Node and Bridge log panes are intended to show official runtime stdout/stderr only.

Examples of valid raw bridge log sources:

```text
kaspa_stratum_bridge::stratum_server
kaspa_stratum_bridge::stratum_listener
kaspa_stratum_bridge::kaspaapi
kaspa_stratum_bridge::rkstratum_cpu_miner
```

Examples of lines that should not be injected into raw panes:

```text
Live refresh active; waiting for new lines
No data to display
Bridge initialized
Saved settings loaded
Frontend heartbeat
```

Status and UI messages belong in status controls, not raw runtime log panes.

### Explorer and analysis

The frontend includes Explorer and Analysis tabs for address-oriented workflows, including:

- Address lookup.
- Transaction display.
- Address book interaction.
- Export/import paths.
- Analysis summaries and tables.
- Counterparty-oriented analysis views.

### Internationalization

The frontend includes locale files for:

```text
ar
de
en
es
fr
hi
id
ja
ko
ru
tr
zh-CN
```

The project includes gate scripts to protect translation coverage and runtime translation contracts.

---

## Architecture overview

High-level architecture:

```text
┌────────────────────────────────────────────────────────────┐
│                    Tauri desktop window                    │
│  Static frontend: HTML, CSS, JS, i18n JSON                  │
└──────────────────────────────┬─────────────────────────────┘
                               │ Tauri invoke/events
┌──────────────────────────────▼─────────────────────────────┐
│                 Rust Tauri backend commands                 │
│  settings, runtime control, logs, explorer, diagnostics      │
└──────────────────────────────┬─────────────────────────────┘
                               │ typed Rust APIs
┌──────────────────────────────▼─────────────────────────────┐
│                 KGW runtime owner layer                     │
│  per-network node/bridge ownership, start/stop/status/logs   │
└──────────────────────────────┬─────────────────────────────┘
                               │ Cargo feature/runtime binding
┌──────────────────────────────▼─────────────────────────────┐
│            Official Rusty Kaspa node / bridge crates        │
│  kaspad, kaspa-core, kaspa-utils, kaspa-stratum-bridge       │
└────────────────────────────────────────────────────────────┘
```

### Frontend layer

The frontend is a static Tauri frontend located under:

```text
apps/kaspa-gateway-desktop/frontend
```

Important frontend owners include:

```text
apps/kaspa-gateway-desktop/frontend/index.html
apps/kaspa-gateway-desktop/frontend/main.js
apps/kaspa-gateway-desktop/frontend/styles.css
apps/kaspa-gateway-desktop/frontend/src/core/header-live-metrics.js
apps/kaspa-gateway-desktop/frontend/src/core/shell-logger.js
apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js
apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js
apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js
apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis.js
apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js
```

### Backend layer

The Tauri backend is located under:

```text
apps/kaspa-gateway-desktop/src-tauri
```

Important backend owners include:

```text
apps/kaspa-gateway-desktop/src-tauri/src/lib.rs
apps/kaspa-gateway-desktop/src-tauri/src/main.rs
apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs
apps/kaspa-gateway-desktop/src-tauri/src/settings_commands.rs
apps/kaspa-gateway-desktop/src-tauri/src/commands.rs
apps/kaspa-gateway-desktop/src-tauri/src/address_commands.rs
apps/kaspa-gateway-desktop/src-tauri/src/explorer_tab.rs
apps/kaspa-gateway-desktop/src-tauri/src/diagnostics.rs
```

### Runtime crates

Runtime integration is split into focused crates:

```text
crates/kaspa-gateway-rk-node
crates/kaspa-gateway-rk-bridge
crates/kaspa-gateway-runtime
```

Other domain crates include:

```text
crates/kaspa-gateway-core
crates/kaspa-gateway-config
crates/kaspa-gateway-security
crates/kaspa-gateway-api
crates/kaspa-gateway-db
crates/kaspa-gateway-node
crates/kaspa-gateway-observability
```

---

## Runtime ownership model

Kaspa Gateway Rust uses an owner-first model.

The goal is to avoid:

- Duplicate runtime start logic.
- Duplicate UI layers.
- CSS hiding workarounds.
- Fake raw log lines.
- Blind search-and-replace changes.
- New orchestration layers when an existing owner already exists.

The preferred model is:

```text
one responsible file/function for each behavior
        ↓
minimal patch to that owner only
        ↓
backup + report
        ↓
syntax/build/gate verification
        ↓
commit only after passing checks
```

### Runtime ownership rules

1. Node tab changes must handle `mainnet`, `testnet10`, and `testnet12`.
2. Bridge tab changes must handle `mainnet`, `testnet10`, and `testnet12`.
3. Status/control messages must stay separate from raw logs.
4. Raw log panes must show official stdout/stderr only.
5. Command previews must reflect the selected settings and flags.
6. Runtime code should use typed Rust APIs and explicit Tauri commands.
7. Release builds should use the same verified runtime path as development builds.

---

## Node and bridge behavior

### Node runtime

The node runtime is bound through `kaspa-gateway-rk-node` and official Rusty Kaspa node crates.

Main responsibilities:

- Build the network-specific runtime plan.
- Resolve the correct runtime family and branch.
- Start the selected node runtime owner.
- Stop the selected node runtime owner.
- Capture raw official node stdout/stderr.
- Publish status without polluting raw logs.

### Bridge runtime

The bridge runtime is bound through `kaspa-gateway-rk-bridge` and `kaspa-stratum-bridge`.

Main responsibilities:

- Build the network-specific bridge plan.
- Attach to the selected node RPC endpoint.
- Start the official bridge runtime.
- Expose Stratum listener behavior through official bridge code.
- Capture official bridge stdout/stderr.
- Publish status without polluting raw logs.

### Bridge modes

The bridge runtime model includes support for modes such as:

```text
official-external-node
official-inprocess-node
```

The exact available behavior depends on compiled features and the selected runtime family.

### Mining policy

Mainnet mining should be treated as external ASIC/Stratum mining. Internal CPU miner behavior is for supported test networks only and requires the `rkstratum_cpu_miner` feature.

Recommended full-feature development command:

```powershell
npm run tauri -- dev --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

Recommended release build feature set:

```powershell
npm run tauri -- build --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

---

## Raw log model

Raw logs are a core feature of this project.

### Correct log path

```text
Official runtime stdout/stderr line
        ↓
backend reader / buffer
        ↓
runtime log command or event
        ↓
frontend log pane
```

### Node raw logs

Node raw logs should show official `kaspad` output only.

The frontend must not wrap raw lines with artificial prefixes such as:

```text
Node logs [poll]
frontend timestamp wrapper
placeholder refresh message
```

### Bridge raw logs

Bridge raw logs should show official `kaspa_stratum_bridge` output only.

For detailed bridge logs, the runtime path should preserve official tracing output such as:

```text
RUST_LOG=info,kaspa_stratum_bridge=debug
```

The backend may configure tracing for the bridge self-worker, but it must not invent raw log lines.

### Auto-scroll

Node and Bridge log panes include per-network auto-scroll controls. Auto-scroll should affect only scroll behavior, not log content.

---

## Desktop UI overview

The desktop UI includes these primary areas:

### Header

The header includes:

- Product title.
- Live network metrics.
- Clock/date display.
- Language selector.
- Currency selector.

Clock ownership is intentionally split:

```text
index.html                 owns clock DOM
header-live-metrics.js     updates clock text
styles.css                 owns clock sizing/layout
explorer.css               must not own header clock sizing
```

### Node tab

The Node tab includes:

- Per-network cards or panels.
- Runtime settings.
- Start/Stop controls.
- Status and command preview.
- Raw logs.
- Log tools such as clear/copy and auto-scroll.

### Bridge tab

The Bridge tab includes:

- Per-network bridge settings.
- Runtime mode selection.
- RPC endpoint options.
- Stratum listen options.
- Optional testnet CPU miner settings.
- Start/Stop controls.
- Status and command preview.
- Raw logs.
- Log tools such as clear/copy and auto-scroll.

### Explorer tab

The Explorer tab includes address and transaction workflows, local address book features, filtering, export/import, and network data display.

### Analysis tab

The Analysis tab includes address analysis, transaction summaries, counterparty analysis, and time-window filtering.

### Settings tab

The Settings tab is the correct owner for persistent configuration controls such as saving settings, restoring defaults, and runtime defaults.

---

## Internationalization

Translations live in:

```text
apps/kaspa-gateway-desktop/frontend/i18n
```

Supported locale files currently include:

```text
ar.json
de.json
en.json
es.json
fr.json
hi.json
id.json
ja.json
ko.json
ru.json
tr.json
zh-CN.json
```

Language metadata files:

```text
lang_map.json
map.json
```

### i18n gates

Run the contract gate:

```powershell
node ".\tools\kgw_i18n_contract_gate.cjs"
```

Run the locale coverage gate:

```powershell
node ".\tools\kgw_i18n_locale_coverage_gate.cjs"
```

The gates are intended to catch:

- Missing translation references.
- Unbound visible HTML text.
- User-visible dynamic JS literals.
- Locale coverage issues.
- Critical key regressions.

### i18n rule

Any user-visible string in HTML or JS should be bound through the i18n system unless it is intentionally technical/runtime data such as a raw log line, network name, URL, command preview, or machine-readable identifier.

---

## Workspace layout

```text
.
├── .github/                             # CODEOWNERS, Dependabot, and CI/security workflows
├── apps/
│   ├── kaspa-gateway-cli/              # CLI application
│   └── kaspa-gateway-desktop/          # Tauri desktop application
│       ├── frontend/                   # Static frontend UI
│       └── src-tauri/                  # Tauri backend
├── config/
│   └── runtime-repository-bindings.json
├── crates/
│   ├── kaspa-gateway-api/
│   ├── kaspa-gateway-config/
│   ├── kaspa-gateway-core/
│   ├── kaspa-gateway-db/
│   ├── kaspa-gateway-node/
│   ├── kaspa-gateway-observability/
│   ├── kaspa-gateway-rk-bridge/
│   ├── kaspa-gateway-rk-node/
│   ├── kaspa-gateway-runtime/
│   └── kaspa-gateway-security/
├── docs/
├── tools/                              # Durable local gate scripts only
├── Cargo.toml                          # Rust workspace
└── README.md
```

---

## Prerequisites

Recommended development environment:

- Windows 10/11 for desktop packaging and NSIS installer builds.
- PowerShell 7+.
- Rust 1.97.1 from `rust-toolchain.toml`.
- Node.js 24 LTS and its bundled npm 11.17.0. Node.js 26 Current is checked as a non-blocking compatibility target.
- Tauri CLI dependency installed through project npm dependencies.
- Git.
- Network access to the configured Rusty Kaspa git repositories.

Install frontend/Tauri dependencies:

```powershell
cd "D:\kaspa-gateway-rust\apps\kaspa-gateway-desktop"
npm ci
```

---

## Quick start

Clone the repository:

```powershell
git clone https://github.com/KaspaPulse/kaspa-gateway-rust.git
cd kaspa-gateway-rust
```

Install desktop dependencies:

```powershell
cd apps\kaspa-gateway-desktop
npm ci
```

Run the desktop application in development mode:

```powershell
npm run tauri -- dev --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

From the repository root, run a Rust check:

```powershell
cargo check -p kaspa-gateway-desktop --no-default-features --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

---

## Development commands

### Format Rust

```powershell
cargo fmt --all
```

### Check Rust formatting

```powershell
cargo fmt --all -- --check
```

### Check desktop backend

```powershell
cargo check -p kaspa-gateway-desktop --no-default-features --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

### Run desktop dev shell

```powershell
cd "apps\kaspa-gateway-desktop"
npm run tauri -- dev --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

### Build desktop app

```powershell
cd "apps\kaspa-gateway-desktop"
npm run tauri -- build --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

### JavaScript syntax checks

```powershell
node --check ".\apps\kaspa-gateway-desktop\frontend\src\core\header-live-metrics.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\analysis\analysis.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\explorer\explorer.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\kaspa-node\kaspa-node.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\kaspa-bridge\kaspa-bridge.js"
```

---


## GitHub Actions status

The repository tracks least-privilege GitHub Actions workflows for CI, workflow linting, CodeQL, secret scanning, dependency review, Rust supply-chain policy, and OpenSSF Scorecard reporting. Third-party actions are pinned to immutable commit SHAs.

Node.js 24 LTS is the blocking JavaScript baseline. Node.js 26 Current runs as a non-blocking compatibility check until it becomes the supported LTS baseline. Dependency installation uses committed lockfiles through `npm ci`.

## Quality gates

Recommended pre-push safety gate:

```powershell
$ErrorActionPreference = "Stop"

cd "D:\kaspa-gateway-rust"

git status --short

node ".\tools\kgw_i18n_contract_gate.cjs"
node ".\tools\kgw_i18n_locale_coverage_gate.cjs"

npm --prefix ".\apps\kaspa-gateway-desktop" ci --ignore-scripts
npm --prefix ".\apps\kaspa-gateway-desktop" run lint
npm --prefix ".\e2e" ci --ignore-scripts
npm --prefix ".\e2e" run lint
npm --prefix ".\e2e" run check

node --check ".\apps\kaspa-gateway-desktop\frontend\src\core\header-live-metrics.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\analysis\analysis.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\explorer\explorer.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\kaspa-node\kaspa-node.js"
node --check ".\apps\kaspa-gateway-desktop\frontend\src\tabs\kaspa-bridge\kaspa-bridge.js"

cargo fmt --all -- --check

cargo check --locked --workspace --all-targets
cargo test --locked --workspace --all-targets

git status --short
```

A clean pre-push state means:

```text
- i18n gates pass
- npm lockfile installs, ESLint, syntax checks, and audits pass
- JavaScript syntax checks pass
- Rust fmt check passes
- locked Rust check and tests pass
- git status is clean
```

---

## Release build

Tauri is configured to bundle an NSIS installer for Windows.

Recommended direct build command:

```powershell
cd "D:\kaspa-gateway-rust\apps\kaspa-gateway-desktop"
npm run tauri -- build --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

Expected build outputs after a successful Windows build:

```text
target\release\bundle\nsis\Kaspa Gateway_0.1.0_x64-setup.exe
target\release\kaspa-gateway-desktop.exe
```

A release artifact folder can include:

```text
INSTALL.txt
Kaspa Gateway_0.1.0_x64-setup.exe
Kaspa Gateway_0.1.0_x64-setup-HHMMSS.exe
kaspa-gateway-desktop.exe
SHA256SUMS.txt
```

Generated release artifacts should stay out of Git. The repository ignores generated artifacts and build outputs.

---

## Configuration

### Runtime repository bindings

Runtime source selection is centralized in:

```text
config/runtime-repository-bindings.json
```

This file defines:

- Network name.
- Runtime family.
- Git repository.
- Branch.
- Feature flag.
- Node package aliases.
- Bridge package aliases.
- Release pinning policy.

### Tauri configuration

Desktop configuration lives in:

```text
apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json
```

Important values include:

```text
productName: Kaspa Gateway
version: 0.1.0
identifier: com.kaspapulse.kaspagateway
bundle target: nsis
```

### Frontend configuration

Frontend shell and tab ownership lives under:

```text
apps/kaspa-gateway-desktop/frontend
```

Important top-level files:

```text
index.html
main.js
styles.css
```

---

## Security posture

Kaspa Gateway Rust follows a local-first and least-privilege security model.

Security principles:

- No plaintext secrets in configuration files.
- Avoid shell command strings for user-controlled input.
- Prefer typed Rust APIs over unstructured command execution.
- Keep frontend access limited to explicit Tauri commands.
- Keep process/runtime ownership isolated.
- Keep generated artifacts out of Git.
- Avoid unsafe Rust unless formally justified.
- Do not log secrets.
- Keep raw runtime logs separate from UI status messages.

The Tauri application uses a Content Security Policy that restricts script, style, image, font, object, frame, and IPC behavior for the local desktop shell.

---

## Troubleshooting

### Raw node logs are empty

Check:

1. The selected network node is actually running.
2. The correct network tab is selected.
3. The backend runtime owner is capturing stdout/stderr.
4. The frontend log pane is polling/receiving the correct runtime log command.
5. No UI placeholder code is writing fake log lines instead of raw runtime output.

### Raw bridge logs are empty

Check:

1. The bridge runtime is started for the selected network.
2. The bridge can connect to the selected node RPC endpoint.
3. The Stratum listener is configured correctly.
4. The bridge self-worker configures tracing/log output correctly.
5. `RUST_LOG=info,kaspa_stratum_bridge=debug` behavior is preserved where detailed bridge logs are required.
6. The raw log pane is not filtering official runtime lines.

### Node or bridge remains running after app close

The desktop app should stop runtime workers on window close. If ports remain open after closing the app, inspect the close handler and runtime shutdown owner in:

```text
apps/kaspa-gateway-desktop/src-tauri/src/lib.rs
apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs
```

### Header clock appears oversized

Clock ownership should remain:

```text
index.html                 DOM
header-live-metrics.js     text updates
styles.css                 size/layout
explorer.css               no ownership
```

Do not add another clamp layer. Fix the existing owner.

### Language switching issues

Run:

```powershell
node ".\tools\kgw_i18n_contract_gate.cjs"
node ".\tools\kgw_i18n_locale_coverage_gate.cjs"
```

Also verify that visible HTML has `data-i18n` bindings and dynamic visible strings use the translation helper.

### Release build does not produce installer

Check:

```text
apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json
```

The bundle target should include:

```text
nsis
```

Then run:

```powershell
cd "D:\kaspa-gateway-rust\apps\kaspa-gateway-desktop"
npm run tauri -- build --features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

---

## Maintainer rules

This project is large enough that changes must be owner-driven.

Before changing behavior:

1. Identify the real owner file/function.
2. Confirm whether a second owner or old layer already exists.
3. Patch the existing owner only.
4. Avoid adding duplicate UI/runtime layers.
5. Avoid CSS hiding workarounds.
6. Backup before mutation.
7. Write a report on success and failure.
8. Restore on failure.
9. Run syntax/build/gate checks.
10. Commit only after passing checks.

### Do not

- Do not push without an explicit user request.
- Do not add fake raw log lines.
- Do not filter raw logs as a substitute for fixing the pipeline.
- Do not hard-code only `mainnet` when a feature must handle all networks.
- Do not add new layers when an existing owner already exists.
- Do not commit generated release artifacts.
- Do not commit large audit backup/report directories.

### Do

- Keep raw runtime logs official.
- Keep UI status separate from raw logs.
- Keep settings ownership clear.
- Keep runtime bindings explicit.
- Keep i18n gates passing.
- Keep Git status clean before release or push.

---

## Git hygiene

Generated folders should stay ignored:

```text
.kgw-release-artifacts/
.kgw-audit-backups/
.kgw-audit-reports/
target/
apps/kaspa-gateway-desktop/src-tauri/target/
apps/kaspa-gateway-desktop/node_modules/
apps/kaspa-gateway-desktop/dist/
apps/kaspa-gateway-desktop/.vite/
*.log
```

Before committing:

```powershell
git status --short
```

Before pushing:

```powershell
git log --oneline -5
git status --short
```

---

## License

This workspace declares:

```text
MIT OR Apache-2.0
```

Review individual upstream Rusty Kaspa dependencies and runtime repositories for their respective license terms.

---

## Acknowledgements

Kaspa Gateway Rust depends on the work of the Kaspa and Rusty Kaspa ecosystem.

Runtime source repositories:

```text
https://github.com/kaspanet/rusty-kaspa
https://github.com/LiveLaughLove13/rusty-kaspa/tree/RKStratumTN12
```

Desktop framework:

```text
https://tauri.app/
```
