# BUG-0032: Shutdown after pruning-chain download exceeds the child budget

- Status: RESOLVED_LOCAL_QUALIFIED.
- Severity: High; official graceful shutdown fails in a newly observed IBD phase.
- Feature/location: Kaspa Node > Testnet10 > Stop after public pruning-chain download.

## Expected and actual
The owned official runtime should shut down within the existing45s budget, release its database/listeners and report the actual result with a responsive UI.
RUN138/139 used one real TN10 worker23640, official2.0.1 on pinned98a4ccd, two CPUs, two async threads, ramScale0.1 and mining off. It validated the pruning proof and downloaded163265headers (trusted stream166249items). Stop at2026-09-21T07:18:37Z exceeded45000ms and used exact-child forced termination. The UI truthfully showed FORCED, stayed responsive(max33.3068ms), and the task settings were restored. No server memory-floor abort.

## Failure mechanism
RUN149 reproduced the same post-chain failure on exact worker432. Code-identity-qualified MAP145 resolved both the active and stopping hot thread to kaspa_pow::matrix::Matrix::compute_rank -> State::new -> calc_block_level -> PruningProofManager::apply_proof. AsyncRuntime shutdown waits for its Tokio blocking pool; the official core joins that runtime. Unoptimized dev PoW matrix computation prolongs this blocking work beyond the existing shutdown budget. This later phase was absent from the earlier conversion/validation regression.
BUG0029's conversion and validation cancellation096/100 remain valid scoped passes. This case occurs after chain download and was previously unqualified.

## Evidence and components
C:/KGW-AUD/v013-v012-host-repro-20260916/ui-ux-20260920/native-public-139-testnet10.json and -shutdown.log; dev16/resources.jsonl; canonical CURRENT.md.
The observer's COMPLETED_P2P_OBSERVATION / exit0 means collection/cleanup completed, not graceful PASS; gracefulClassification is FAILED_GRACEFUL.
Related components: official Rusty Kaspa consensus/IBD shutdown, kgw_real_owner_runtime core join and integrated_runtime_commands self-worker45s contract.

## Next action and protection
FIX156 adds only kaspa-pow opt-level2 to the existing dev profile. Debug assertions, official dependency bytes/revisions, runtime/IPC/settings ownership and45s deadline remain unchanged. BUILD161 and a source/artifact-bound actual post-chain Stop check are required before closing this finding. RUN149 reproduced only the failed phase using the same isolated database/artifact, with bounded exact-worker noninvasive/nonsuspending stack capture. MAP145 .text/base identity was verified before resolving symbols. Keep original logs and unsuccessful diagnostic attempts.
Do not extend shutdown deadlines, patch Cargo caches, change accepted ownership, infer full sync/mining, or publish a release. Fix the proved cause, then verify this exact phase and related invalidated behavior.
SERVICE_STATUS=OPEN; LEARNING_STATUS=OPEN.

## Fresh-build failed regression165
BUILD161 produced actual0.1.3 artifact53D3803838FA49E6751DD3E4194DEDB26E2865EEABEB06BAF77C58946F7D8CFF. RUN165 reached163265chain headers but again exceeded45s/coreJoined=false/forced=true, maxUI99.104ms; settings restored, observerexit1. The PoW-only optimization is insufficient. Capture165 failed before attach due a corrupt external PowerShell prefix; no165stack exists. New map173 all PE sections/base match the fresh artifact; new ASCII capture177 parses without error. Next177captures the still-failed phase under unchanged limits before another fix hypothesis. No complete-runtime claim.

## Recovered diagnostic177/185
RUN177=FAILED_GRACEFUL on the same53D380 artifact, forced=true/coreJoined=false/maxUI146.2996ms, original task settings restored. Captures177 succeeded noninvasively without suspending. MAP173 all9PE sections/base match;185 resolved207addresses. Active thread21988 is in compute_rank/populate_reachability_and_headers/apply_proof; the same stopping thread performs bincode hash-vector serialization. AsyncRuntime waits for blocking-pool shutdown and Core waits for join. The PoW-only optimization is insufficient; inspect the remaining unoptimized consensus proof/header-store path before a narrow next fix. Preserve45s budget and all failed evidence.

## Closure: FIX248 and RUN254
The earlier requirement to complete every official Node-core shutdown inside a fixed 45s/60s/180s wall-clock budget was disproved by repeated source-bound native evidence. In the affected post-pruning-chain phase, the official shutdown signal is accepted immediately and async-runtime services stop quickly, but Rusty Kaspa can still be completing a non-cancellable pruning-proof blocking task before the official owner thread can join. A KGW wall-clock cutoff therefore converted a slow-but-progressing official shutdown into exact-child forced termination and could interrupt database-mutating work.

FIX248 removes automatic time-based forced fallback only for self-workers that own the official Node core (Node and Bridge in-process). Those owners now wait for the real typed terminal shutdown result and child exit; ordinary workers retain the bounded 45s child / 55s parent fallback contract. Node and Bridge Stop remain asynchronous through Tauri, keep the UI in Stopping while the backend owns reconciliation, and no longer apply a JavaScript Stop wall-clock cutoff. Exact ownership, typed STOPPED/FAILED validation, parent identity checks, and forced fallback for bounded non-Node-core paths remain intact.

Verification is source/artifact bound. Targeted static/syntax/Settings checks passed; targeted Rust tests passed 5/5; Graphify current-flow refresh/query passed; KSSS contract checks passed 24/24. BUILD253 produced artifact CF2DA44593CC69B2CA05600E822B0C87797B40BF013556A2B13EEFA96CDD417E. RUN254 reached fresh AFTER_PRUNING_CHAIN_DOWNLOAD on Testnet10, then Stop completed naturally: forcedStop=false, coreJoined=true, gracefulClassification=VERIFIED_GRACEFUL, driver exit_code=0, settings restored exactly, max Stop UI latency 177.3001ms. Shutdown signal began 2026-09-22 00:49:27.201+03:00; async-runtime worker stopped at 00:49:27.767; official owner emitted "... core is shut down" at 00:53:47.247. No forced termination occurred.

Regression protection: official Node-core ownership maps to no automatic shutdown deadline; standard workers remain bounded; explicit timeout unit coverage still exercises the forced-fallback contract; frontend static assertions require Stop to defer terminality to the async backend; supervised runtime-failure and normal-stop contracts remain covered. Failed RUN165/177/227/234/244 evidence remains valid historical learning and must not be relabeled as passes.

Scope boundary: this closes BUG-0032 for the reproduced Testnet10 late-pruning Stop phase on the locally qualified 0.1.3 candidate. It does not qualify full synchronization, accepted mining, TN13 public operation, packaging, clean-machine installation, cross-platform behavior, remote CI, release, deployment, or production.

SERVICE_STATUS=RESOLVED; LEARNING_STATUS=CLOSED.


## Evidence

The final locally qualified FIX248 source remains byte-identical in the 25CC external-delivery candidate. Governed dependency re-resolution produced Cargo.lock blob `6031ba5c58f250a2808a1cb2e0be428da6d8cbee` while preserving PR #94 direct dependency intent and the qualified runtime revisions `98a4ccd8d200853787f227bd4536ac540cf34957` / `ad45e241e6688a14901fd24dd8dc33c5c9a33f40`.

The replacement native artifact for the changed dependency baseline is `FAECC4FFD052B4BC482C362EAA6F55418F313647B34C5EEB3A36190958AE4E45`. The one invalidated Testnet10 phase was requalified at `AFTER_PRUNING_CHAIN_DOWNLOAD` with `forcedStop=false`, `coreJoined=true`, `VERIFIED_GRACEFUL`, no runtime/cleanup error, and exact profile restoration.

## Root Cause

The proved defect was KGW imposing an outer wall-clock cutoff on a same-EXE worker that owns the official Node core. Rusty Kaspa had accepted shutdown and could still be completing a non-cancellable pruning-proof blocking task before the core join completed. The old cutoff therefore converted slow-but-progressing official shutdown into exact-child forced termination.

## Verification

FIX248 source qualification passed, including the targeted Rust set 5/5. The original BUILD253/RUN254 evidence closed the reproduced late-pruning phase for its source/artifact baseline. After the 25CC dependency baseline changed, dependency qualification passed again and a new artifact was built. Because its SHA differed from BUILD253, only the affected Testnet10 `AFTER_PRUNING_CHAIN_DOWNLOAD` phase was replayed; it passed with natural terminal shutdown, `forcedStop=false`, `coreJoined=true`, `gracefulClassification=VERIFIED_GRACEFUL`, and responsive Stop UI.

## Regression Protection

Official Node-core owners have no automatic shutdown wall-clock deadline; standard workers retain bounded fallback. Typed terminal outcomes, exact parent/child identity checks, explicit timeout coverage for bounded paths, and asynchronous frontend Stop ownership remain enforced. Runtime evidence is reused only while its source/artifact/environment predicates remain valid.

## NEXT ACTION

BUG-0032 itself remains closed. Continue only the external-delivery/release lifecycle on the exact qualified delivery content. Do not reopen BUG-0032 merely because CI, merge, artifacts, or release preparation remain pending.

## DO NOT REPEAT

Do not repeat RUN254 or the 25CC affected-phase runtime replay, the qualified native build, targeted FIX248 Rust tests, Graphify, frontend qualification, or KSSS unless a relevant validity predicate changes. Preserve failed historical runs as learning evidence and never relabel them as passes.
