# Kaspa Gateway User Guide

Kaspa Gateway is a local-first desktop control plane for the official Kaspa node and Stratum bridge runtimes. It manages configuration, process ownership, status, and observability; it does not replace Kaspa consensus, `kaspad`, or the bridge runtime.

This guide covers the published desktop application and the user-facing workflows that exist in the repository. Developer build instructions remain in the repository [README](../README.md).

## 1. Obtain and verify a release

Use the repository's [GitHub Releases](https://github.com/KaspaPulse/kaspa-gateway-rust/releases) page. The README identifies the current published verified build; local candidate metadata or a successful CI artifact is not itself a published release.

Published desktop release sets include the product artifacts, `SHA256SUMS.txt`, and preserved Sigstore/SLSA build-provenance bundles. Verify the checksum before installation. Examples:

```powershell
Get-FileHash .\KASPA_GATEWAY_WINDOWS_X64_NSIS_*.exe -Algorithm SHA256
```

```bash
shasum -a 256 KASPA_GATEWAY_MACOS_UNIVERSAL_DMG_*.dmg
```

Compare the result with `SHA256SUMS.txt` from the same release. If GitHub CLI is available, release provenance can also be checked with `gh attestation verify` against `KaspaPulse/kaspa-gateway-rust`; see the [desktop release runbook](runbooks/desktop-release.md) for the repository's exact provenance contract.

### Windows x64

Download the NSIS `.exe` from the published release and run it as the current user. The current developer release is not Authenticode-signed, so Windows may show an OS trust warning. Review the release source/checksum rather than disabling Windows security controls globally.

### macOS Universal

Use the published Universal `.dmg` or `.app.zip`. The current developer release uses ad-hoc signing and is not Apple-notarized, so macOS trust warnings are expected. Review the release source/checksum rather than disabling Gatekeeper globally.

## 2. First-run workflow

1. Start Kaspa Gateway.
2. Choose a network in the Node or Bridge surface. `mainnet` and `testnet10` are the stable supported networks. `testnet13` is experimental and requires explicit opt-in.
3. In **Kaspa Node**, review the runtime/data location and network ports before starting the node.
4. Start the node and wait for the runtime status/readiness surface to report the real managed-runtime state. Do not infer success only from a button click.
5. In **Kaspa Bridge**, confirm the target Kaspa RPC endpoint and listener ports. Start the bridge only after the intended node/RPC endpoint is available.
6. Use the Node/Bridge raw-log panes for official runtime stdout/stderr and the application Log surface for application diagnostics.
7. Use Explorer, Analysis, Top Addresses, and Settings as needed for local data and network/API-backed views.

## 3. Main desktop surfaces

The desktop registers these seven user-facing tabs:

| Surface | Primary use |
|---|---|
| Explorer | Inspect addresses and network data, manage address-oriented explorer workflows, and export supported results. |
| Kaspa Node | Configure, start, stop, and inspect the managed Kaspa node runtime for the selected network. |
| Kaspa Bridge | Configure, start, stop, and inspect the managed Stratum bridge runtime and its listeners. |
| Analysis | Run the application's supported analysis views over available local/network data. |
| Top Addresses | View the supported top-addresses data surface. |
| Log | View application diagnostics/status logging. Node/Bridge raw panes remain reserved for official runtime stdout/stderr. |
| Settings | Manage application settings and supported data/configuration operations. |

The complete external-interface contract, including the CLI, local files, runtime listeners, defaults, and security boundaries, is documented in [External Interfaces](EXTERNAL_INTERFACES.md).

## 4. Networks and default port profiles

Defaults are profiles, not permission to expose a service publicly. Review the active settings before start, and use unused ports when another process already owns a port.

### Node defaults

| Network | Policy | RPC port | P2P listen port |
|---|---|---:|---:|
| `mainnet` | Stable | `16110` | `16111` |
| `testnet10` | Stable supported testnet | `16210` | `16211` |
| `testnet13` | Experimental opt-in | `16210` | `16711` |

The CLI `node-plan` defaults to network `mainnet`, RPC host `127.0.0.1`, and RPC port `16110`.

### Bridge defaults

| Network | Kaspa RPC port | Stratum listen | Prometheus listen | Dashboard port |
|---|---:|---:|---:|---:|
| `mainnet` | `16110` | `:5555` | `:2112` | `3030` |
| `testnet10` | `16210` | `:5655` | `:2212` | `3130` |
| `testnet13` | `16210` | `:5755` | `:2312` | `3230` |

A listener such as `:5555` is a runtime listen setting, not a loopback-only promise. Restrict listener interfaces and host firewall rules to the access actually required.

## 5. Configuration and local data

Kaspa Gateway resolves one user-data root and stores `config.json` directly under that root. The database root is `<user-data-root>/databases`.

Default user-data roots are derived from the operating system:

| Platform | Default base |
|---|---|
| Windows | `%LOCALAPPDATA%\KaspaGateway` (falling back to `%APPDATA%\KaspaGateway`) |
| macOS | `$HOME/Library/Application Support/KaspaGateway` |
| Other Unix development environments | `$XDG_DATA_HOME/KaspaGateway`, or `$HOME/.local/share/KaspaGateway` when `XDG_DATA_HOME` is not set |

`KASPA_GATEWAY_DATA_DIR` can override the user-data root when it is set to a non-empty **absolute** path before the process starts. Relative override paths are rejected.

For source/developer CLI builds, resolve the paths instead of guessing them:

```text
kaspa-gateway config-path
kaspa-gateway db-paths
```

Do not place seed phrases, private keys, passwords, access tokens, or other secrets in the application data directory or user-entered address/search fields.

## 6. Node workflow

1. Select the intended network. Keep `testnet13` disabled unless you explicitly intend to use the experimental network.
2. Review the effective data path, RPC/P2P settings, and any additional supported runtime arguments.
3. Check that configured ports are not already owned by an unrelated process.
4. Start the node from the Node surface.
5. Wait for backend/runtime status and readiness. The UI is designed to reflect runtime ownership rather than optimistic local state.
6. Inspect the raw runtime log for the official node stdout/stderr when diagnosing startup or runtime behavior.
7. Stop the node from the owning Node controls. Do not terminate unrelated Kaspa processes by PID merely because they use familiar ports.

## 7. Stratum bridge workflow

1. Select the same intended network as the node/RPC endpoint.
2. Confirm the Kaspa RPC endpoint and the Stratum/Prometheus/dashboard listener settings.
3. Resolve port conflicts before start; the Bridge surface includes port validation/auto-fix support for its configured listeners.
4. Start the bridge and wait for its actual runtime/listener readiness state.
5. Use the bridge raw-log pane for official bridge stdout/stderr.
6. Stop bridge instances through their owning controls before changing conflicting listener settings.

Network listeners can expose services beyond the local machine. Bind only to the required interfaces and apply host/network firewall policy appropriate to the environment.

## 8. Logs and status

Kaspa Gateway separates runtime logs from application diagnostics:

- **Node/Bridge raw log panes:** official managed-runtime stdout/stderr only. They must not be replaced by invented startup/success text.
- **Status/readiness surfaces:** process ownership, lifecycle, readiness, and failure state.
- **Log tab/application diagnostics:** application-level events and diagnostics.

When reporting a problem, preserve the relevant status and log context, but remove credentials, private data, or secrets before sharing it.

## 9. Safe shutdown

Closing the main desktop window triggers a managed shutdown of KGW-owned runtime workers. The application prevents the window from closing while that shutdown is in progress. If shutdown fails, the application keeps the window open and reports the failure instead of claiming a clean exit.

For a controlled shutdown:

1. Stop active Bridge work first when practical.
2. Stop the managed Node runtime.
3. Confirm the status surfaces no longer show owned running work.
4. Close the application.

Do not kill an unknown or unrelated Kaspa process to make the UI appear stopped.

## 10. Common troubleshooting

### Start does not become ready

- Read the runtime status/readiness surface and the corresponding raw log.
- Confirm the configured executable/runtime is available.
- Check for port conflicts and invalid data/configuration paths.
- Confirm the selected network is enabled; `testnet13` requires explicit opt-in.
- For source/developer CLI builds, run `kaspa-gateway runtime-check` for the repository's runtime diagnostic report.

### Port conflict

- Identify which process owns the port before changing or terminating anything.
- Choose an unused port, or use the Bridge surface's supported port-fix workflow.
- Do not assume a process is KGW-owned only because it uses `16110`, `16111`, or another default port.

### Configuration path is unexpected

- Run `kaspa-gateway config-path` in a CLI build.
- Check whether `KASPA_GATEWAY_DATA_DIR` is set.
- The override must be an absolute path.

### The window does not close

A failed managed-runtime shutdown intentionally prevents application exit. Read the reported shutdown error, stop the affected KGW-owned runtime through its normal controls, and retry close. Do not force-kill unrelated processes.

## 11. Security and support

- Kaspa Gateway is not a wallet and must not store wallet seed phrases or private keys.
- Prefer loopback for local-only RPC use. Expose RPC, Stratum, metrics, or dashboards only when required and protected by the surrounding host/network controls.
- Treat exported files and logs as potentially sensitive because they can contain addresses, paths, and operational metadata.
- Report reproducible bugs through [GitHub Issues](https://github.com/KaspaPulse/kaspa-gateway-rust/issues).
- Report suspected vulnerabilities privately using the process in [SECURITY.md](../SECURITY.md).
