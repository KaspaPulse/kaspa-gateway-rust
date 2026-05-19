# ADR-0010: Same-EXE Parallel Self-Worker Runtime

## Status

Accepted.

## Decision

Kaspa Gateway Rust uses the existing same-executable self-worker mechanism as the official runtime strategy for embedded node and bridge operation.

The desktop shell does not launch an external `kaspad.exe`, `kaspabridge.exe`, shell script, or external binary. Instead, the application starts the current executable with the self-worker arguments:

```text
--kgw-self-worker <role> --network <network>
```

The runtime registry key is:

```text
role:network
```

Required supported workers:

```text
node:mainnet
node:testnet10
node:testnet12
bridge:mainnet
bridge:testnet10
bridge:testnet12
```

## Rationale

This keeps the current working behavior where all networks and all bridges can run together while avoiding unsafe external process ownership.

It also avoids forcing all Rusty Kaspa runtime state into one process. Each self-worker is still the same application executable and uses Kaspa libraries internally, but it receives an isolated process boundary for role/network runtime state, ports, logs, and shutdown.

## Ownership

### Runtime owner

```text
apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs
```

Responsibilities:

- Own the parallel self-worker registry.
- Own start/stop/status/logs for role/network workers.
- Spawn only the current executable as a self-worker.
- Preserve `same_exe=true`.
- Preserve `external_kaspad_exe=false`.
- Preserve `uses_kaspa_libraries=true`.

### Network metadata owner

```text
crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs
```

Responsibilities:

- Own mainnet/testnet10/testnet12 metadata.
- Preserve distinct network ports.
- Preserve the correct source owner:
  - mainnet/testnet10: mainline/master owner.
  - testnet12: tn12 owner.

### Frontend role

Frontend tabs are adapters only.

They may request:

```text
start
stop
status
logs
```

They must not own shell commands, external process launch, or Rusty Kaspa process topology.

## Guard

The permanent guard is:

```text
tools/kgw_parallel_self_worker_runtime_gate.cjs
```

It must pass before runtime topology changes are accepted.

## Non-goals

- No external `kaspad.exe`.
- No external shell launcher.
- No frontend-owned runtime launch logic.
- No replacement of the existing working parallel self-worker mechanism.
