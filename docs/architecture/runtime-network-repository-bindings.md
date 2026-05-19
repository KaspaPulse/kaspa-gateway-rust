# Runtime Network Repository Bindings

## Current binding

| Network | Family | Repository | Branch | Feature |
|---|---|---|---|---|
| mainnet | mainline | https://github.com/kaspanet/rusty-kaspa.git | master | official-kaspa-runtime-mainline |
| testnet10 | tn12 | https://github.com/LiveLaughLove13/rusty-kaspa.git | RKStratumTN12 | official-kaspa-runtime-tn12 |
| testnet12 | tn12 | https://github.com/LiveLaughLove13/rusty-kaspa.git | RKStratumTN12 | official-kaspa-runtime-tn12 |

## Decision

Mainnet remains bound to kaspanet/rusty-kaspa master.

Testnet10 and Testnet12 are bound to LiveLaughLove13/rusty-kaspa RKStratumTN12 for both node and bridge dependencies.

## Important rule

Repository binding is build-time, not runtime.

Changing this binding requires:

1. Editing Cargo dependency aliases.
2. Keeping Rust network mapping consistent.
3. Running the repository binding audit.
4. Rebuilding the desktop app.

## Real owners

### Node Cargo dependencies

```text
crates/kaspa-gateway-rk-node/Cargo.toml
```

Mainline aliases, used by mainnet:

```text
kaspad-lib-mainline
kaspa-core-mainline
kaspa-utils-mainline
```

TN12 aliases, now used by testnet10 and testnet12:

```text
kaspad-lib-tn12
kaspa-core-tn12
kaspa-utils-tn12
```

### Bridge Cargo dependencies

```text
crates/kaspa-gateway-rk-bridge/Cargo.toml
```

Mainline alias, used by mainnet:

```text
kaspa-stratum-bridge-mainline
```

TN12 alias, now used by testnet10 and testnet12:

```text
kaspa-stratum-bridge-tn12
```

### Runtime network mapping

```text
crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs
crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs
crates/kaspa-gateway-rk-bridge/src/lib.rs
```

Current mapping:

```text
mainnet             -> master / Mainline
testnet10/testnet12 -> RKStratumTN12 / Tn12
```

## Permanent audit

Run:

```powershell
node tools\kgw_runtime_repository_binding_audit.cjs
```

Expected result:

```text
network=mainnet;family=mainline;branch=master;node_repo=https://github.com/kaspanet/rusty-kaspa.git;bridge_repo=https://github.com/kaspanet/rusty-kaspa.git
network=testnet10;family=tn12;branch=RKStratumTN12;node_repo=https://github.com/LiveLaughLove13/rusty-kaspa.git;bridge_repo=https://github.com/LiveLaughLove13/rusty-kaspa.git
network=testnet12;family=tn12;branch=RKStratumTN12;node_repo=https://github.com/LiveLaughLove13/rusty-kaspa.git;bridge_repo=https://github.com/LiveLaughLove13/rusty-kaspa.git
status=PASS
```
