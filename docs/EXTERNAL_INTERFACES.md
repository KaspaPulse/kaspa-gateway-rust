# Kaspa Gateway External Interfaces

This document describes the user- and operator-accessible input/output interfaces implemented by Kaspa Gateway. It is a reference for the current desktop/CLI contract, not a declaration that internal Tauri commands are a stable public API.

Kaspa Gateway itself does **not** expose a stable KGW-owned public HTTP API. The application manages local files and official Kaspa/Stratum runtimes, and those managed runtimes may expose their own network endpoints according to the selected configuration.

## 1. Desktop GUI

**NAME:** Kaspa Gateway desktop GUI
**PURPOSE:** Configure and operate the local Kaspa Gateway control plane.
**ENTRYPOINT:** `kaspa-gateway-desktop` / installed desktop application.
**INPUTS:** User selections, addresses/search values, runtime settings, paths, network choices, start/stop actions, supported import/export actions.
**OUTPUTS:** Rendered network/data views, runtime ownership/status/readiness, application diagnostics, official Node/Bridge stdout/stderr in raw-log surfaces, and supported exported files.
**DEFAULTS:** `mainnet` and `testnet10` are enabled stable networks; `testnet13` is experimental and disabled by default. Network/runtime defaults are described below.
**AUTHENTICATION_IF_ANY:** No KGW application login. Access is the current operating-system user session.
**NETWORK_EXPOSURE:** The GUI itself is local. Actions in Node/Bridge can create managed runtime listeners; network exposure depends on those runtime settings.
**FAILURE_BEHAVIOR:** Backend/runtime errors are surfaced to status/diagnostic paths; a requested action is not sufficient evidence of runtime success. Application close is prevented when graceful shutdown fails.
**SECURITY_BOUNDARY:** The frontend does not directly own operating-system runtime processes. Process control crosses the typed Tauri/backend boundary, which validates and owns runtime operations.

### Registered GUI surfaces

| Surface | User inputs | Primary outputs / effects |
|---|---|---|
| `explorer` | Address/search and supported explorer actions | Address/network data, saved/local explorer state, supported exports |
| `kaspa-node` | Network, runtime/path/port settings, start/stop and supported node options | Managed node process state, readiness, command/runtime details, raw node stdout/stderr |
| `kaspa-bridge` | Network, Kaspa RPC endpoint, listener/instance settings, start/stop | Managed bridge state/readiness/listeners, port validation, raw bridge stdout/stderr |
| `analysis` | Supported analysis selections over available data | Analysis results derived by the application |
| `top-addresses` | Supported top-addresses view controls | Top-addresses data/results |
| `log` | Log-view controls | Application diagnostics/status logging |
| `settings` | Application/data settings and supported maintenance actions | Persisted configuration/state changes and corresponding result/error feedback |

## 2. Development CLI

**NAME:** `kaspa-gateway` CLI
**PURPOSE:** Repository development/diagnostic CLI for application info, paths, databases, address validation, API URL construction, and node capability/plan inspection.
**ENTRYPOINT:** `kaspa-gateway <subcommand>` from a built `apps/kaspa-gateway-cli` binary.
**INPUTS:** Subcommand plus the documented positional/flag values.
**OUTPUTS:** Human-readable stdout; errors to stderr; database-changing subcommands write under the resolved user-data root.
**DEFAULTS:** `node-plan` defaults to `--network mainnet --rpc-host 127.0.0.1 --rpc-port 16110`.
**AUTHENTICATION_IF_ANY:** None added by the CLI. File access uses the current OS user; constructed runtime/API endpoints retain their own security model.
**NETWORK_EXPOSURE:** The CLI does not open a KGW network server. `node-plan` only builds/previews a launch plan; URL commands construct URLs.
**FAILURE_BEHAVIOR:** Invalid inputs/path resolution/runtime checks print an error and exit non-zero.
**SECURITY_BOUNDARY:** The CLI validates addresses/paths/endpoints through repository crates and must not be treated as a credential store.

### CLI subcommands

| Command | Inputs | Output / effect |
|---|---|---|
| `info` | none | Application name/version |
| `architecture` | none | Workspace/architecture summary |
| `config-path` | none | Resolved `config.json` path |
| `runtime-check` | none | Runtime diagnostic report; non-zero on diagnostic setup failure |
| `db-paths` | none | Resolved database root/app-data/address/transaction paths |
| `db-init` | none | Initializes databases under the resolved user-data root |
| `db-add-address <address> <name>` | Kaspa address and display name | Validates and upserts the address record; current command records it as `mainnet` |
| `db-list-addresses` | none | Prints saved address, name, and network records |
| `validate-address <address>` | Kaspa address | Prints masked valid address or a validation error |
| `redact-url <url>` | URL | Prints the repository's redacted URL representation |
| `api-balance-url <address>` | Kaspa address | Builds and prints the balance endpoint URL; it does not itself fetch the balance |
| `api-network-url` | none | Builds and prints the network-info endpoint URL |
| `node-capabilities` | none | Prints detected/default node capability description |
| `node-owner-kind` | none | Prints `kgw-owner` |
| `node-plan [--network <network>] [--rpc-host <host>] [--rpc-port <port>]` | Optional network/RPC values | Validates the endpoint and prints the managed-node command preview |
| `node-owner-check` | none | Prints owner identity plus capability description |

## 3. Configuration and local-data interface

**NAME:** User data root and `config.json`
**PURPOSE:** Persist application configuration and local database state.
**ENTRYPOINT:** Desktop/backend configuration code; CLI `config-path` and `db-paths`.
**INPUTS:** OS environment/path resolution, supported settings, and optional `KASPA_GATEWAY_DATA_DIR`.
**OUTPUTS:** `<user-data-root>/config.json`, `<user-data-root>/databases`, and other application-owned data below the same validated root.
**DEFAULTS:** Windows `%LOCALAPPDATA%\KaspaGateway` with `%APPDATA%` fallback; macOS `$HOME/Library/Application Support/KaspaGateway`; other Unix `$XDG_DATA_HOME/KaspaGateway` or `$HOME/.local/share/KaspaGateway`.
**AUTHENTICATION_IF_ANY:** OS filesystem permissions only.
**NETWORK_EXPOSURE:** None by itself.
**FAILURE_BEHAVIOR:** Missing platform data roots, invalid/unsafe paths, or relative `KASPA_GATEWAY_DATA_DIR` values fail path resolution rather than silently using the unsafe value.
**SECURITY_BOUNDARY:** `KASPA_GATEWAY_DATA_DIR` must be an absolute validated path. Do not store credentials, seed phrases, private keys, or tokens in KGW data files.

### Environment variable

| Name | Meaning | Contract |
|---|---|---|
| `KASPA_GATEWAY_DATA_DIR` | Override the application user-data root | Optional; ignored when empty; must be an absolute validated path when set |

## 4. Managed Kaspa node runtime

**NAME:** Official Kaspa node runtime interface
**PURPOSE:** Start/stop and observe the configured official Kaspa node for a selected network.
**ENTRYPOINT:** Desktop `kaspa-node` surface; CLI `node-plan` provides a development preview of the managed launch plan.
**INPUTS:** Network, RPC endpoint, P2P/listener configuration, data/runtime paths, and supported additional runtime arguments.
**OUTPUTS:** Managed runtime process, ownership/status/readiness, RPC/P2P endpoints according to runtime settings, and official stdout/stderr in the raw node log.
**DEFAULTS:** Desktop network profiles use RPC/P2P ports `16110/16111` (`mainnet`), `16210/16211` (`testnet10`), and `16210/16711` (`testnet13`). CLI `node-plan` defaults to `mainnet`, `127.0.0.1`, `16110`.
**AUTHENTICATION_IF_ANY:** KGW does not add an authentication layer to the official node protocol endpoint; any endpoint authentication/security is owned by the underlying runtime/configuration.
**NETWORK_EXPOSURE:** RPC should remain loopback for local-only use. P2P or other listener exposure depends on the selected runtime settings and bind interface.
**FAILURE_BEHAVIOR:** Spawn/readiness/runtime failures remain failures and are surfaced in status/diagnostic/raw-log evidence; KGW must not fabricate READY/success.
**SECURITY_BOUNDARY:** KGW owns only the exact process/runtime it starts under its ownership model. A PID/port match alone is not authorization to adopt or terminate another process.

## 5. Managed Stratum bridge runtime

**NAME:** Official Stratum bridge runtime interface
**PURPOSE:** Connect miners/bridge consumers to the selected Kaspa node endpoint using the supported bridge runtime.
**ENTRYPOINT:** Desktop `kaspa-bridge` surface.
**INPUTS:** Network, Kaspa RPC endpoint, bridge instance/listener settings, Stratum/Prometheus/dashboard ports, and supported bridge runtime options.
**OUTPUTS:** Managed bridge process(es), readiness/listener status, network listeners, and official stdout/stderr in the raw bridge log.
**DEFAULTS:** See the profile table below; `testnet13` is experimental and disabled by default.
**AUTHENTICATION_IF_ANY:** KGW does not add a separate authentication layer to bridge listeners; protocol/runtime security is delegated to the bridge/runtime configuration.
**NETWORK_EXPOSURE:** Listener values can expose services beyond loopback. Restrict bind interfaces/firewalls to the access required.
**FAILURE_BEHAVIOR:** Port conflicts, invalid settings, spawn failures, or listener/readiness failures are surfaced as failures; the UI includes supported port-conflict validation/repair helpers.
**SECURITY_BOUNDARY:** Bridge process ownership is explicit per managed network/instance; do not reuse listener ownership evidence to control an unrelated process.

| Network | Kaspa RPC | Stratum | Prometheus | Dashboard | Default policy |
|---|---:|---:|---:|---:|---|
| `mainnet` | `16110` | `:5555` | `:2112` | `3030` | enabled/stable |
| `testnet10` | `16210` | `:5655` | `:2212` | `3130` | enabled/stable |
| `testnet13` | `16210` | `:5755` | `:2312` | `3230` | disabled/experimental |

These are defaults only. The desktop accepts other valid unused ports according to its validation rules.

## 6. Internal Tauri IPC boundary (not a public API)

**NAME:** Desktop frontend-to-Rust Tauri IPC
**PURPOSE:** Carry typed desktop actions such as settings, start/stop/status/log operations from the local webview frontend to the Rust backend.
**ENTRYPOINT:** Internal desktop webview/Tauri invoke/event paths.
**INPUTS:** Application-internal typed payloads.
**OUTPUTS:** Application-internal results/events/errors consumed by the desktop UI.
**DEFAULTS:** Internal implementation contract; no public compatibility guarantee is made here.
**AUTHENTICATION_IF_ANY:** Not an Internet authentication surface; access is bounded by the local desktop application process/webview model.
**NETWORK_EXPOSURE:** None as a KGW HTTP/RPC server.
**FAILURE_BEHAVIOR:** Backend validation/process errors must propagate as errors; the frontend must not convert an IPC request into unverified success.
**SECURITY_BOUNDARY:** This boundary exists specifically so the frontend does not spawn/control OS processes directly. It must not be presented to external consumers as a supported public API.

## 7. Failure and exposure rules shared by all interfaces

- Status, readiness, and runtime logs are evidence from the managed backend/runtime; UI intent alone is not success.
- Raw Node/Bridge log panes contain official runtime stdout/stderr, not fabricated application messages.
- Keep local-only RPC on loopback when remote access is unnecessary.
- Treat Stratum, metrics, dashboards, P2P listeners, and any explicitly non-loopback binds as network exposure that requires host/network policy.
- Do not put secrets in addresses, URLs, logs, exports, or configuration files.
- Report suspected vulnerabilities privately using [SECURITY.md](../SECURITY.md).
