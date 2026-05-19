# Embedded Bridge Runtime Command Contract

## Status

Accepted locally.

## Goal

The bridge must never start from hidden hardcoded command values. The selected UI flags for the active Bridge network are the source of truth.

## Canonical mechanism

### External node mode

External mode means the bridge attaches to an already-running KGW-owned node.

Required behavior:

- Mainnet bridge uses the Bridge/Mainnet UI flags.
- Testnet10 bridge uses the Bridge/Testnet10 UI flags.
- Testnet12 bridge uses the Bridge/Testnet12 UI flags.
- The command preview and Start request must resolve from the same UI-state command builder.
- The backend validates the selected network, feature family, and ports, but must not silently rebuild a different command.
- External mode must never emit direct --netsuffix in the bridge command.
- External mode uses --kaspad-address only from the selected UI field.

### In-process bridge mode

In-process mode means the bridge owns the kaspad arguments after the -- separator.

Required behavior:

- Bridge flags appear before --.
- Kaspad flags appear after --.
- Testnet10 may emit -- --testnet --netsuffix=10 only when node-mode=inprocess.
- Testnet12 may emit -- --testnet --netsuffix=12 only when node-mode=inprocess.
- Mainnet must not emit --testnet or --netsuffix.
- Any future appdir, ports, RPC, miner, or metrics values must come from the visible Bridge UI fields or a single manifest/schema owner.

### CPU miner

Compile-time feature:

```text
rkstratum_cpu_miner
```

Runtime UI flags:

```text
--internal-cpu-miner
--internal-cpu-miner-address
--internal-cpu-miner-threads
--internal-cpu-miner-throttle-ms
--internal-cpu-miner-template-poll-ms
```

The UI may show these flags, but starting with them must require a binary built with:

```text
--features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

### Log isolation

Node log tab accepts node-role lines only.
Bridge log tab accepts bridge-role lines only.

Allowed markers:

```text
role=node
role=bridge
[self-worker][node]
[self-worker][bridge]
```

No fallback may copy node controller logs into the bridge log tab.
No fallback may copy bridge worker logs into the node log tab.

## Mainnet node UI rule

Node/Mainnet Runtime settings must not show:

```text
--testnet
--netsuffix
```

These controls are valid only for Node/Testnet10 and Node/Testnet12.
