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
