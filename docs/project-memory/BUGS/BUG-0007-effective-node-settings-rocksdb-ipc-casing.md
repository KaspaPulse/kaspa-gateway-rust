# BUG-0007: EffectiveNodeSettings RocksDB IPC casing drift

Status: VERIFIED LOCALLY; Windows revalidation pending exact checkpoint transfer.
Stable ID: BUG-0007
Severity: P0 runtime-start blocker.
Discovered: 2026-09-12 during real Windows zero-touch validation on `Server`.

## Evidence
- Exact validation HEAD: `583ac6204609f6eb80504fe797eb828a2ff2ea53`.
- Mainnet Start button was clicked and `kgw_kgw_apply_node_settings_v1` was dispatched.
- Tauri rejected `effectiveNodeSettings` before runtime spawn with `unknown field rocksDbCacheSize`.
- No validation runtime ports opened; unrelated service on `16110/16111` remained untouched.
## Root Cause
`EffectiveNodeSettings` used `#[serde(rename_all = "camelCase")]`. For Rust fields named `rocksdb_*`, serde generated `rocksdbPreset`, `rocksdbCacheSize`, and `rocksdbWalDir`, while both Node and Bridge frontend payloads intentionally use `rocksDbPreset`, `rocksDbCacheSize`, and `rocksDbWalDir`.

## Fix / Decision
Add explicit serde renames for the three RocksDB fields to the frontend contract names and aliases for the legacy Rust-derived spellings. Preserve all other typed payload and runtime behavior.

## Verification
- Red regression reproduced the mismatch before the fix.
- Focused serde regression PASS after the fix.
- `kgw_effective_node_settings_gate.cjs` PASS with explicit contract guards.
- Complete desktop IPC suite PASS: 56/56.
## Regression Protection
The IPC integration test now asserts the exact frontend-facing RocksDB keys, forbids emitting the legacy spellings, and verifies legacy aliases remain accepted. The static effective-node-settings gate also checks the serde rename/alias contract.

## Remaining Risk
Real Windows zero-touch must be rerun on the exact local checkpoint. Any subsequent schema mismatch remains blocking and must be fixed locally before continuing.

## NEXT ACTION
Checkpoint BUG-0007 to the local bare remote, transfer that exact commit to `Server`, and resume Windows zero-touch validation from the failed Mainnet Node case.

## DO NOT REPEAT
Do not rename frontend RocksDB controls to match serde's acronym guess, do not remove `deny_unknown_fields`, and do not bypass the typed `effectiveNodeSettings` contract with command-preview parsing.
