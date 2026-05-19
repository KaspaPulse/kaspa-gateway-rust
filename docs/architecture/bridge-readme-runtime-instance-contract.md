# Bridge README Runtime Instance Contract

## Status

Accepted locally.

## Source

The bridge README describes the active runtime contract:

- External node mode attaches to an existing kaspad.
- In-process node mode passes kaspad arguments after the -- separator.
- Multiple bridge processes or instances must not share web dashboard ports, Stratum ports, or Prometheus ports.
- Internal CPU miner is compile-time gated by rkstratum_cpu_miner.
- Runtime CPU miner flags are still selected through the UI.

## UI ownership rule

The Bridge tab owns these values through visible fields:

- --config
- --node-mode
- --appdir
- --kaspad-address
- --block-wait-time
- --print-stats
- --log-to-file
- --health-check-port
- --web-dashboard-port
- --var-diff
- --shares-per-min
- --var-diff-stats
- --extranonce-size
- --pow2-clamp
- --coinbase-tag-suffix
- --approximate-geo-lookup
- --stratum-port
- --min-share-diff
- --prom-port
- --instance
- --instance-log-to-file
- --instance-var-diff
- --instance-shares-per-min
- --instance-var-diff-stats
- --instance-pow2-clamp
- --internal-cpu-miner
- --internal-cpu-miner-address
- --internal-cpu-miner-threads
- --internal-cpu-miner-throttle-ms
- --internal-cpu-miner-template-poll-ms

No hidden fixed command path may override the command preview.

## Mode contract

External mode:

- no direct --netsuffix
- --kaspad-address comes from UI
- Stratum, dashboard, Prometheus, VarDiff, CPU miner flags come from UI

In-process mode:

- bridge flags before --
- kaspad flags after --
- testnet10/testnet12 may use -- --testnet --netsuffix=10/12 after --
- mainnet must not use --testnet or --netsuffix

## Instance contract

The + button must add a real instance in the existing Bridge tab owner.
Duplicate and Remove must modify the same bridgeInstances state.
Command Preview and Start must use the same resolved instance state.

## CPU miner

Build with:

```text
--features "official-kaspa-runtime-all rkstratum_cpu_miner"
```

Runtime flags stay UI-driven.
