# CURRENT STATE

- Verified at: 2026-09-23 during local-only Kaspa v2.1.0 mainline upgrade qualification and checkpoint reconciliation.
- Repository: `KaspaPulse/kaspa-gateway-rust`.
- Current HEAD: **VERIFY DYNAMICALLY** before any decision.
- Historical task-base observation: `3bcf8d8aaa94bc303aba558788f101edef48b5b2`.
- Current branch: `feat/kaspa-v2.1.0-runtime-rebaseline-20260922`.
- Current remote main: **VERIFY DYNAMICALLY** before any integration decision.
- Historical GitHub-main observation: `bb183816e5c315107c64411c1793c89d8ec74e8e`, one workflow-only commit ahead of the task base with no product-source overlap.
- Working tree: **NOT VERIFIED** until derived dynamically. Before the local checkpoint it is intentionally DIRTY with the reviewed v2.1.0 candidate; after checkpoint it should be CLEAN and the `local` bare-mirror ref should match HEAD.
- Push target for authorized checkpoint work: local bare remote `local`. Real GitHub push/PR/merge/tag/release/deployment/Production actions are **NOT VERIFIED / NOT AUTHORIZED** for this task.
- BUILD-014-A4 artifact SHA-256: `39A7E1D923414677F8510DCEC2B6EACA4F01317D7A1E1868E3F2821BCA12F3A2`.
- Four planned native cases are VERIFIED_SUCCESS: Node Mainnet, Node Testnet10, External Bridge Mainnet, External Bridge Testnet10 CPU-only.
- Testnet10 External Bridge runtime evidence: stable v2.1.0, `rpc_network=testnet-10`, `node_mode=external`, `node_kind=remote`, `bridge_kind=official-external-node`, `listener_count=0`, no Stratum/Prometheus listener, `cpu_enabled=true`, CPU hash samples `1,1,21`.
- All four native task cases are DO-NOT-REPEAT unless product/runtime bytes or applicable runtime policy change.
- E2E CPU-only remediation changed five E2E files. Syntax, npm check, npm lint, deepmerge-security, runtime-port, bridge-locator, recovery-harness, network-generation gate/regressions, diff-check, and Graphify incremental/query are PASS.
- Project continuity gate and continuity regression tests are PASS after current reconciliation.
- Historical build manifest remains preserved at SHA-256 `29E91A73FF43E206143BAFE8323B22438E0B9356310D5528F55D96B4F2469FB1`.
- Post-build E2E delta receipt SHA-256: `74913534B068582900650C10B6C52F5BE3442402C5953D693FCA2C0779ACD8B1`.
- Final checkpoint manifest identity is external and must be regenerated/verified against the exact current Git path set immediately before staging; use the operation journal rather than an older embedded candidate-manifest hash.
- `FAIL-0004` is BLOCKED: idle exact parent PID 4404 does not exit through authorized normal close paths. Runtime workers are zero and task runtime ports are free. Force termination was not authorized or used. Close root cause remains **NOT VERIFIED**.
- Existing parent 4404 must remain preserved; do not start a second desktop parent.

## NEXT ACTION
Derive Git/local-mirror state first. If the checkpoint does not yet exist on `local`, regenerate the final manifest, require exact path/hash equality, stage only those files, verify the cached diff, create one local `[skip ci]` commit, and mirror it only to `local`. If the local ref already matches the checkpoint HEAD and the worktree is clean, stop local execution and carry forward `FAIL-0004` for owner review or a separately authorized safe-close follow-up.

## DO NOT REPEAT
Do not repeat BUILD-014, Node Mainnet, Node Testnet10, External Bridge Mainnet, External Bridge Testnet10, E2E npm/static checks, continuity gates, Graphify refresh, or historical evidence verification unless relevant inputs change. Do not force-kill PID 4404, start a second parent, touch the preserved production lease, enable Testnet13, or perform real GitHub/release/deployment actions.
