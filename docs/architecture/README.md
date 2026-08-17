# Architecture Overview

Kaspa Gateway is a local-first Rust/Tauri desktop control plane for official Kaspa node and Stratum bridge runtimes. It owns configuration, process lifecycle, status, and observability; it does not reimplement Kaspa consensus, `kaspad`, or bridge protocol behavior that belongs to the official runtime.

## Durable Boundaries

- `mainnet` and `testnet10` are stable supported networks.
- `testnet12` is experimental and explicit opt-in.
- Runtime ownership remains isolated by role/network with one owner per runtime instance.
- The frontend requests runtime actions across the Tauri IPC boundary; it does not own operating-system process launch.
- Raw runtime log panes display captured official stdout/stderr only. Status or UI management messages must not be mixed into those raw streams.
- Runtime repository/revision ownership is defined by `config/runtime-repository-bindings.json` and must be reviewed before changing official bindings.

## Component Map

- `crates/kaspa-gateway-config/` — configuration contracts and parsing.
- `crates/kaspa-gateway-rk-node/` — Rusty Kaspa node binding/control surface.
- `crates/kaspa-gateway-rk-bridge/` — bridge binding/control surface.
- `crates/kaspa-gateway-runtime/` — runtime ownership/lifecycle support.
- `apps/kaspa-gateway-desktop/src-tauri/` — desktop IPC and process/runtime boundary.
- `apps/kaspa-gateway-desktop/frontend/` — UI adapters, views, and controls.
- `apps/kaspa-gateway-cli/` — CLI probes and utilities.
- `tools/` — repository-native gates, audits, and smoke tooling.

## Architecture Records and Contracts

Canonical new ADRs live in `docs/adr/`; see `docs/adr/README.md` for lifecycle and numbering. The historical ADR below remains valid at its existing path for compatibility:

- `docs/architecture/adr-0010-same-exe-parallel-self-worker-runtime.md` — accepted runtime ownership/process topology decision.

Other current architecture/contract documents in this directory:

- `bridge-readme-runtime-instance-contract.md`
- `embedded-bridge-runtime-command-contract.md`
- `runtime-network-repository-bindings.md`

Do not duplicate these documents inside `PROJECT_STATE.md`. The state document should summarize only what a new session needs to resume safely and link here for architectural detail.
