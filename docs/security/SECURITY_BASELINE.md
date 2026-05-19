# KGW Security Baseline

## Purpose

This document records the local security baseline for KGW before describing the project as security-reviewed.

## Latest Corrected Baseline

Latest corrected local security gate result:

- Verdict: SECURITY_GATE_CORRECTED_BASELINE_PASSED
- Confirmed RustSec vulnerabilities: 0
- npm moderate-or-higher vulnerabilities: 0
- tracked-file secret scan: no obvious secrets detected
- Tauri capabilities: reviewed
- release executable metadata: reviewed
- Windows GUI subsystem: expected for portable executable

## Required Local Commands

Run from the current clean repository clone:

- node .\tools\kgw_i18n_contract_gate.cjs
- node .\tools\kgw_i18n_locale_coverage_gate.cjs
- node .\tools\kgw_parallel_self_worker_runtime_gate.cjs
- cargo fmt --all -- --check
- cargo check -p kaspa-gateway-desktop --no-default-features --features "official-kaspa-runtime-all rkstratum_cpu_miner"
- cd .\apps\kaspa-gateway-desktop
- npm audit --audit-level=moderate

Optional but recommended:

- cargo audit
- cargo deny check

## Runtime Repository Binding Expectations

- mainnet: https://github.com/kaspanet/rusty-kaspa, branch master
- testnet10: https://github.com/LiveLaughLove13/rusty-kaspa, branch RKStratumTN12
- testnet12: https://github.com/LiveLaughLove13/rusty-kaspa, branch RKStratumTN12

## Before Claiming Hardened Status

Complete all of the following before claiming KGW is hardened:

1. Resolve or explicitly document RustSec warnings.
2. Add and enforce a stable cargo-deny policy.
3. Enable Dependabot alerts and review dependency PRs manually.
4. Enable GitHub secret scanning where available.
5. Reintroduce CI only after workflow scope is safe and billing is stable.
6. Sign Windows installer and portable executable.
7. Review network ports, RPC exposure, and local file writes manually.
8. Run external or independent penetration testing.
