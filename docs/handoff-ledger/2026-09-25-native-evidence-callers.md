# Native evidence caller consolidation checkpoint

Status: PARTIAL TASK / SUMMARY CALLERS VERIFIED / LEGACY RETIREMENT BLOCKED
Timestamp: 2026-09-25T04:04:45.901352+00:00
Task ID: KASPA_GATEWAY_100_PERCENT_RUST_MIGRATION_20260923
Branch: feat/owned-implementation-100-percent-rust-20260923
Observed base: 5977447e5bc9256e1bee62bf6f3a8c613be17cbd; verify actual current Git dynamically.

## LAST CONFIRMED STATE
Both remaining active calls to the PowerShell evidence-summary implementation in `tools/kgw_zero_touch_e2e.ps1` now invoke `Get-KgwNativeEvidenceSummary`, which delegates to the source-built native Rust evidence CLI. It accepts only exit0/1 with a matching boolean pass field; parse/transport errors throw. Its native-error preference is function-local and preserves the caller's preference.

## COMPLETED / VERIFIED
- Ten extracted-caller cases PASS: eight exact report comparisons and two malformed-JSON rejection cases, with both native error preference values and no preference leakage.
- Changed-script parser checks and working diff checks PASS. Rust implementation is unchanged; existing OP068/OP069 native semantic, stable/MSRV and strict Clippy evidence is reused, not rerun.
- An exact active-source usage audit found no outside references to16 superseded helper functions. The reviewed retirement would remove937 lines, reducing the helper from1356 to419 lines while retaining emergency/serialization/PowerShell-version/port support.

## BLOCKED RETIREMENT / PRESERVATION
The one guarded atomic update of `tools/kgw_zero_touch_evidence.ps1` failed with WinError5/Access denied. The original is byte-identical and remains1356 lines. The proposed937-line retirement was NOT applied and is NOT counted as completed migration.

Original backup: `C:/Users/abuha/KaspaGateway-Rust100-20260923/native-evidence-callers-op070/before.zip`.
Review: `retirement-review.json` in the same evidence directory.
Preserved same-directory proposal: C:\Users\abuha\KaspaGateway-Rust100-20260923\repo\tools\kgw_zero_touch_evidence.ps1.op070-040119449844.tmp
Original SHA256: 3d39766467deb92547ebc65e70bd4ff2291afa2568d567279d0cb91d444082e3
Proposal SHA256: f0f1837e9fb0b467a4234c8ed9483189fb2ec0ff0e33c0e7b185883d383b0b2a

No retry, alternative write API, in-place write, permission change, process termination or handle closure was attempted. File use is observed; the precise original WinError5 root cause remains unproven.

## EVIDENCE
Directory: `C:/Users/abuha/KaspaGateway-Rust100-20260923/native-evidence-callers-op070/`.
Receipts: `summary-caller-receipt.json`, `results.json`, `differences.json`, `qualification-receipt.json`, and the existing append-only OPERATION_JOURNAL.md.
No E2E script body, desktop, browser, node/bridge or clipboard operation was executed by these caller tests.

## NEXT ACTION
Preserve the verified summary caller and this checkpoint in a scoped local commit. Legacy function retirement remains blocked pending an authorized supported resolution of the file replacement failure. Continue only independent safe work without bypassing denied operations or discarding older OP066 source. Full migration remains PARTIAL; source debt41/execution debt13/exceptions21 remain and final Graphify/KSSS-result inspection/native/platform qualification is incomplete.

## DO NOT REPEAT
Do not retry the blocked helper replacement or old generator/canonical replacements via alternate APIs. Do not rerun valid native/caller evidence merely because a checkpoint was committed. Do not treat the reviewed proposal as an applied deletion, and do not push/release an unqualified candidate.
