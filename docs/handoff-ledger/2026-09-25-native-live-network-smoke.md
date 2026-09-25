# Checkpoint - native live-network smoke orchestration

Timestamp: 2026-09-25T10:40:00Z
Task ID: KASPA_GATEWAY_100_PERCENT_RUST_MIGRATION_20260923
Host: Server
Branch: feat/owned-implementation-100-percent-rust-20260923
Status: OP079 locally qualified component; full migration remains PARTIAL.

## LAST CONFIRMED STATE
The former `tools/kgw_live_network_smoke.ps1` was the only owned live-smoke implementation and had no active caller beyond its runbook and migration-debt manifest.
The exact repository script SHA-256 matched the frozen OP079 oracle: `b286d1992f3cd8039c5dc7d767ebc118869bf5f4ca51fb81639926aa475681e6`.

## COMPLETED AND VERIFIED
`xtask/src/bin/kgw-live-network-smoke.rs` plus `xtask/src/live_network_smoke/` implement the same Windows mainnet/Testnet10 orchestration around the existing same-executable parent and `kgw-live-probe`.
Stable and MSRV tests pass 13/13 with two explicit external-oracle tests ignored by default; scoped Clippy `-D warnings` and binary build pass.
Owned synthetic process coverage proves exact parent start, readiness polling, observation, parent loss, port release, relaunch, final cleanup, busy-port refusal, executable-identity refusal, wrong-network, early-exit and probe-timeout behavior.

Frozen legacy versus native lifecycle comparison passes 4/4 with zero semantic differences after repairing only the parity wrapper to provide the legacy `ExpectedNetwork` profile field.
Only bounded elapsed time and isolated log-parent prefixes are normalized. Success/failure, network, peer counts, counters, parent-loss and relaunch fields remain compared.
Exact frozen legacy preflight assignments and native preflight match on Server with zero differences.
## ADOPTION AND RETIREMENT
The runbook now invokes `cargo run --locked -p xtask --bin kgw-live-network-smoke --`.
The old PowerShell implementation is deleted and its exact source-debt entry is removed.
The native runner deliberately fails closed on missing build prerequisites instead of using WinGet, rewriting PATH, updating Rust, or copying `llvm-ar.exe` to `ar.exe`.
`--plan-only` provides a no-build/no-runtime execution plan; `--skip-build` is only for already-qualified release binaries.
No real node, browser, production data, Testnet13, DNS, Cloudflare, release or deployment operation was performed by OP079 qualification.

## EVIDENCE
Evidence directory: `C:/Users/abuha/KaspaGateway-Rust100-20260923/live-network-smoke-rust-op079/`.
Key receipts: `expected-network-parity-receipt.json`, `preflight-parity-receipt.json`, `qualification-after-preflight-receipt.json`.
Operation journal: `C:/Users/abuha/KaspaGateway-Rust100-20260923/OPERATION_JOURNAL.md`, OP079/OP079R/OP079P/OP079Q/OP079A.

## LIMITATIONS
Synthetic lifecycle parity is not a claim that the current native Kaspa application was launched. Final native/platform qualification remains required before task closeout.
Server has no local Graphify installation/graph, so the approved task analysis clone on `kas` was updated with an exact SHA-bound mirror of only the five OP079 code files plus the old PowerShell deletion. Graphify PASS: 6906 nodes / 18149 edges; all five native source paths are represented and the old PowerShell path has zero nodes. Graphify still reports 33 unrelated inputs with zero AST nodes, so this is a focused component result, not complete-corpus assurance.

## NEXT ACTION
Stage only OP079, rerun language-policy/continuity/diff checks against the staged retirement, commit OP079 locally, then resume OP078 without replaying valid OP079 tests.

## DO NOT REPEAT
Do not modify the frozen legacy oracle, rerun real networks for reassurance, restore the PowerShell launcher, or rerun OP079 parity unless relevant source/fixture/toolchain predicates change.
Do not stage or discard OP066/OP071/OP072/OP078 work as part of the OP079 checkpoint.