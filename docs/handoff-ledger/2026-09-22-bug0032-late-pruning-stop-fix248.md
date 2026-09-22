# BUG-0032 late-pruning graceful Stop closure

Task: BUG-0032 post-pruning-chain shutdown.
Status: COMPLETE_LOCAL_QUALIFIED.
Device/worktree: Server / C:/KGW-Local-Validation/kaspa-gateway-rust-v0.1.3-p0-20260916.
Branch: fix/desktop-v0.1.3-p0-runtime-ui-recovery-20260916.
Source base HEAD during qualification: 1dd4c62601610c5899495f971dffcf9dcd14e8ff.
No GitHub push, PR, tag, release, deployment, or production action is authorized or performed.

## Last confirmed state
FIX248 removes automatic time-based forced termination only for same-EXE workers that own the official Node core. Their Stop path waits for the real typed terminal shutdown result and child exit. Standard workers retain the bounded 45s child / 55s parent fallback and exact-child ownership checks. Node/Bridge Stop remains asynchronous through Tauri and has no JavaScript wall-clock cutoff; UI transition state remains Stopping until backend terminality.

Local source qualification is PASS: affected static/syntax/Settings gate, targeted Rust 5/5, Graphify current-flow refresh/query, KSSS contract checks 24/24, and git diff-check. Historical failed attempts remain evidence and are not relabeled as passes.

BUILD253 produced:
- Artifact SHA256: CF2DA44593CC69B2CA05600E822B0C87797B40BF013556A2B13EEFA96CDD417E.
- Native parent PID 24528 / start 2026-09-21T21:25:50.2189915Z during the qualified session.
- Build completed without resource-floor abort; build rerun is unnecessary while inputs remain valid.

RUN254 is the decisive affected-phase qualification:
- Network: Testnet10.
- Target: fresh AFTER_PRUNING_CHAIN_DOWNLOAD.
- Driver PID 20664; exact self-worker PID 25096 under the qualified parent.
- Receipt state: VERIFIED_FAILED_PHASE_REGRESSION.
- forcedStop=false.
- coreJoined=true.
- gracefulClassification=VERIFIED_GRACEFUL.
- error=null; cleanupError=null.
- settings restored exactly.
- max Stop UI latency: 177.3001ms.
- driver exit_code=0.
- Shutdown signal: 2026-09-22 00:49:27.201+03:00.
- async-runtime worker stopped: 00:49:27.767+03:00.
- official owner terminal evidence: "... core is shut down" at 00:53:47.247+03:00.
- No automatic forced fallback occurred.

CLOSE255 is VERIFIED_SUCCESS: exact dev25 parent closed via CloseMainWindow; BUILD253 scheduled task is Ready with LastTaskResult=0; no application, Cargo/rustc, CDP, RPC, P2P, or Stratum owner remains.

## Root cause and durable protection
The failure was not that the official shutdown signal was ignored. Rusty Kaspa accepted the signal and shut down async services promptly, but an already-running pruning-proof blocking task could outlive arbitrary KGW wall-clock budgets before the official owner thread could join. The old KGW wrapper interpreted that slow-but-progressing join as nonterminal failure and forced the exact child, potentially interrupting database-mutating work.

Protection:
- official Node-core owners have no automatic shutdown wall-clock deadline;
- ordinary non-Node-core workers retain bounded 45s/55s fallback;
- typed STOPPED/FAILED outcome and terminality checks remain mandatory;
- exact parent/child identity checks remain mandatory;
- explicit timeout tests still cover forced fallback where it is permitted;
- frontend Stop uses backend terminality and keeps the UI responsive/Stopping;
- successful evidence is reused only while source/artifact/environment predicates remain valid.

## Claim boundary
BUG-0032 is resolved for the reproduced Testnet10 late-pruning Stop phase on this local 0.1.3 candidate. This is not full-runtime or release qualification. Full synchronization, accepted mining, TN13 public/live/mining, packaging, clean-machine installation, cross-platform behavior, remote CI, release, deployment, and production remain NOT VERIFIED.

SERVICE_STATUS=RESOLVED.
LEARNING_STATUS=CLOSED.

## NEXT ACTION
The scoped local BUG-0032 checkpoint commit now contains this closure evidence and the qualified candidate; verify the actual HEAD/tree with Git rather than self-embedded prose. No further BUG-0032 source, build, or runtime action is required unless a relevant validity predicate changes or a new defect is observed. Do not repeat RUN254, BUILD253, or the qualified source tests without such invalidation.


Timestamp: 2026-09-22T14:08:00Z.

## COMPLETED / VERIFIED

BUG-0032 remains CLOSED_LOCAL_QUALIFIED. External-delivery rebaseline on GitHub main `25cc011943805aefa7045d34c21a5d67134dd640` resolved the dependency graph without rolling back PR #94. The current affected product/runtime source bytes remain identical to the locally qualified FIX248 source.

A new native artifact was built after the governed lock resolution:
- SHA256: `FAECC4FFD052B4BC482C362EAA6F55418F313647B34C5EEB3A36190958AE4E45`.
- Dependency qualification: PASS.
- Targeted FIX248 Rust tests: PASS 5/5.
- The one invalidated runtime phase was requalified on Testnet10 at `AFTER_PRUNING_CHAIN_DOWNLOAD`.
- Result: `forcedStop=false`, `coreJoined=true`, `gracefulClassification=VERIFIED_GRACEFUL`, `proofReceived=true`.
- Runtime cleanup and exact original settings restoration completed successfully.
- No full runtime matrix, live TN13, release, deployment, or production claim is implied.

## EVIDENCE / TESTS

Primary evidence remains the canonical external audit/checkpoint surface under
`C:/KGW-Local-Validation/audits/desktop-v0.1.3-p0-runtime-ui-20260916`.

Current external-delivery evidence includes:
- `BUG0032_EXTERNAL_DELIVERY_25CC_LOCK_RESOLUTION_FINAL.json`.
- Final staged candidate tree before delivery commit: `e7e4ae86b37fb86afeee9f2b87218c087fd11696`.
- Resolved Cargo.lock blob: `6031ba5c58f250a2808a1cb2e0be428da6d8cbee`.
- Final 25CC manifest SHA256: `ff33d2972bc0362d8519578749a9e17d1f0d3ca960c388477deaecbd90786cbd`.
- Affected-phase runtime receipt SHA256: `BA53A11B24FE74E90FBCD2E61CD66B899481777532F9B37DCA5D28C51B8706D2`.

## DO NOT REPEAT

Do not repeat RUN254, the 25CC affected-phase replay, dependency qualification, targeted FIX248 Rust tests, frontend qualification, Graphify, KSSS, or the native build while their source/artifact/environment validity predicates remain unchanged. Reuse the evidence above and rerun only an invalidated surface.
