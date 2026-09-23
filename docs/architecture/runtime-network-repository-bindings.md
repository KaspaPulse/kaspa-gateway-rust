# Runtime Network Repository Bindings

## Current binding

| Network | Family | Repository | Branch | Feature |
|---|---|---|---|---|
| mainnet | mainline | https://github.com/kaspanet/rusty-kaspa.git | stable | official-kaspa-runtime-mainline |
| testnet10 | mainline | https://github.com/kaspanet/rusty-kaspa.git | stable | official-kaspa-runtime-mainline |
| testnet13 | tn13 (experimental) | https://github.com/kaspanet/rusty-kaspa.git | dagknight | official-kaspa-runtime-tn13 |

## Decision

Mainnet and Testnet10 are pinned to the official kaspanet/rusty-kaspa stable v2.1.0 revision.

Testnet13 remains a separate experimental binding, is disabled by default, and requires explicit runtime opt-in.

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

TN13 aliases, used only by experimental testnet13:

```text
kaspad-lib-tn13
kaspa-core-tn13
kaspa-utils-tn13
```

### Bridge Cargo dependencies

```text
crates/kaspa-gateway-rk-bridge/Cargo.toml
```

Mainline alias, used by mainnet and testnet10:

```text
kaspa-stratum-bridge-mainline
```

TN13 alias, used only by experimental testnet13:

```text
kaspa-stratum-bridge-tn13
```

### Runtime network mapping

```text
crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs
crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs
crates/kaspa-gateway-rk-bridge/src/lib.rs
```

Current mapping:

```text
mainnet/testnet10 -> stable v2.1.0 / Mainline
testnet13         -> dagknight / Tn13 (explicit opt-in)
```

## Permanent audit

Run:

```text
cargo run --locked -p xtask -- runtime-repository-binding-gate --strict --online
```

Expected result:

```text
KGW_RUNTIME_REPOSITORY_BINDING_GATE_R21C_PASSED
errors: 0
warnings: 0
```
