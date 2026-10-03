# Checkpoint - native saved-evidence source identity hardening

Timestamp: 2026-09-25T06:35:00Z
Task ID: KASPA_GATEWAY_100_PERCENT_RUST_MIGRATION_20260923
Host: Server
Branch: feat/owned-implementation-100-percent-rust-20260923
Observed base HEAD: 9f36b11608faa313b83dea3298f5215766c023d9
Status: PARTIAL task; this is an independently verified native qualification-path repair.

## LAST CONFIRMED STATE
The adopted native evidence/result paths call zero_touch_evidence::identity::source_diff. Its legacy-compatible v2 implementation previously hashed only unstaged changes and skipped C-quoted untracked paths that did not resolve as files. A repository subdirectory was accepted even though sibling source was omitted.

## COMPLETED / VERIFIED
Four new isolated Git regressions were executed before the fix. ASCII v2 compatibility passed; staged-index exclusion, Unicode untracked-content sensitivity and working-tree-root enforcement failed. No real project index, commit or remote was mutated by the fixtures.

The shared Rust identity path now requires the actual working-tree root and an empty staged index before and after collection. It reads NUL-delimited untracked names, preserves Unicode and UTF-16 ordinal ordering, rejects ambiguous/control/non-relative/duplicate names, and rejects missing or non-regular untracked files instead of silently dropping them. Existing unambiguous ASCII v2 wire bytes are unchanged.

Six Windows source-identity regressions pass on stable and MSRV. Strict Clippy on all xtask targets with KSSS features passes. The rebuilt native source-hash command matches the old PowerShell hash on the actual current root/empty-index context and rejects the actual xtask subdirectory with exit2. MSRV all-target feature check and diff check pass. A Unix symlink regression is present but was not executed on Windows.

## COMPATIBILITY / INVALIDATION
This deliberately rejects previously unsafe staged/subdirectory contexts. It is not a claim of legacy behavior parity for those inputs. Legacy artifacts with omitted Unicode source may fail current identity validation and must not inherit a PASS. Source/artifact checks that depend on the changed identity implementation require proportional requalification; unchanged UI, KSSS central runtime and OP066 WASM behavior are not rerun merely because this fix exists.

No process/runtime/clipboard operations, dependency upgrades, source-language reclassification, protected-file replacement or remote mutation occurred. This repairs code already used by native callers rather than adding an unused alternate path.

## EVIDENCE
Existing operation journal: C:/Users/abuha/KaspaGateway-Rust100-20260923/OPERATION_JOURNAL.md, OP074.
Evidence directory: C:/Users/abuha/KaspaGateway-Rust100-20260923/source-identity-regressions-op074/.
Read regression-first.log for the proved failures; fixed-regressions-receipt.json for stable/MSRV/Clippy; cli-receipt.json and cli-qualification-receipt.json for the actual caller and ASCII v2 compatibility. Source before-state is preserved in identity.before.rs.

## NEXT ACTION
Recover actual Git and matching receipts. Preserve older OP066 and blocked/unadopted OP071/OP072 source. Continue independent final-qualification repair, including verification of existing Start/Stop gate findings and frontend regression loader compatibility. Do not treat old failure summaries as proof of current root causes. Final browser/native/cross-platform/security/strict-language and governed remote integration remain required.

## DO NOT REPEAT
Do not replay six passing source-identity tests without source/toolchain/fixture invalidation. Do not apply old artifact PASS results across this changed fingerprint guard without checking their source/index/path predicates. Do not reroute blocked emergency/preview/translation/graph/KSSS-result requests, alter protected file permissions or force their replacement. The task is not closed and no remote candidate is qualified.
