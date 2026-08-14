# Live Kaspa Network Smoke Test

The live smoke test starts the embedded official Rusty Kaspa runtime sequentially
for mainnet and Testnet 10. It does not mine, submit transactions, or wait for a
full initial block download.

## Operating policy

| Network | Default | Runtime | Data directory |
|---|---|---|---|
| mainnet | enabled | official stable v2.0.1 | `%LOCALAPPDATA%\KaspaGateway\nodes\mainnet` |
| testnet10 | enabled | official stable v2.0.1 | `%LOCALAPPDATA%\KaspaGateway\nodes\testnet10` |
| testnet12 | disabled | experimental TN12 build | `%LOCALAPPDATA%\KaspaGateway\nodes\testnet12` |

RPC listeners are bound to loopback. Bridge Stratum listeners may bind to the
LAN only when the operator intentionally exposes the selected port and applies
host firewall rules. External local-node mode is the recommended default for a
mining bridge; in-process mode remains exclusive with the same-network Node tab.

## Run on Windows

From the repository root:

```powershell
pwsh -NoProfile -File .\tools\kgw_live_network_smoke.ps1
```

The script:

1. enforces Rust 1.91 or newer and installs missing Protobuf/LLVM build dependencies with WinGet;
2. checks hardware, disk, and RPC port availability;
3. builds the stable embedded runtime and the read-only gRPC probe;
4. starts mainnet and TN10 one at a time;
5. verifies the reported network, RPC health, peer count, and DAA progression;
6. terminates the task-owned parent, verifies its same-executable worker exits and releases the port, then proves relaunch reconciliation;
7. force-stops the relaunched task-owned parent in a `finally` block and confirms port release;
8. writes logs and `report.json` below `artifacts/live-network-smoke/`.

The smoke launcher uses the desktop's dedicated same-executable parent mode. After RPC readiness it terminates that task-owned parent, proves the worker releases the RPC listener, and relaunches the same network to verify durable ownership reconciliation. Direct `--kgw-self-worker` invocation is intentionally unsupported because production workers require an exact desktop parent identity.

Testnet 12 is never started by this script. Its UI toggle requires an explicit
warning confirmation and the backend independently rejects starts without the
experimental opt-in flag.

## Production readiness

A passing smoke test proves startup, local RPC, peer connectivity, and short-term
chain progress. It does not prove full synchronization or production capacity.
Before production use, complete a full sync on hardware that satisfies the
current official Rusty Kaspa guidance and verify stable peer count, disk growth,
restart recovery, firewall policy, and bridge share telemetry over an extended
observation period.
