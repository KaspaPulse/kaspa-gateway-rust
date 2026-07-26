# Runtime Network Repository Bindings

## Current binding

| Network | Family | Repository | Branch | Feature |
|---|---|---|---|---|
| mainnet | mainline | https://github.com/kaspanet/rusty-kaspa.git | stable | official-kaspa-runtime-mainline |
| testnet10 | mainline | https://github.com/kaspanet/rusty-kaspa.git | stable | official-kaspa-runtime-mainline |
| testnet12 | tn12 (experimental) | https://github.com/LiveLaughLove13/rusty-kaspa.git | RKStratumTN12 | official-kaspa-runtime-tn12 |

## Decision

Mainnet and Testnet10 are pinned to the official kaspanet/rusty-kaspa stable v2.0.1 revision.

Testnet12 remains a separate experimental binding, is disabled by default, and requires explicit runtime opt-in.

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

Mainline aliases, used by mainnet and testnet10:

```text
kaspad-lib-mainline
kaspa-core-mainline
kaspa-utils-mainline
```

TN12 aliases, used only by experimental testnet12:

```text
kaspad-lib-tn12
kaspa-core-tn12
kaspa-utils-tn12
```

### Bridge Cargo dependencies

```text
crates/kaspa-gateway-rk-bridge/Cargo.toml
```

Mainline alias, used by mainnet and testnet10:

```text
kaspa-stratum-bridge-mainline
```

TN12 alias, used only by experimental testnet12:

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
mainnet/testnet10 -> stable v2.0.1 / Mainline
testnet12         -> RKStratumTN12 / Tn12 (explicit opt-in)
```

## Permanent audit

Run:

```powershell
node tools\kgw_runtime_repository_binding_audit.cjs
```

Expected result:

```text
network=mainnet;family=mainline;branch=stable;node_repo=https://github.com/kaspanet/rusty-kaspa.git;bridge_repo=https://github.com/kaspanet/rusty-kaspa.git
network=testnet10;family=mainline;branch=stable;node_repo=https://github.com/kaspanet/rusty-kaspa.git;bridge_repo=https://github.com/kaspanet/rusty-kaspa.git
network=testnet12;family=tn12;branch=RKStratumTN12;node_repo=https://github.com/LiveLaughLove13/rusty-kaspa.git;bridge_repo=https://github.com/LiveLaughLove13/rusty-kaspa.git
status=PASS
```
