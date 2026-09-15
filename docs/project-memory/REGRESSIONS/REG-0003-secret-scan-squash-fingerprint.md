# REG-0003: Squash merge created a second exact secret-scan fingerprint

- Status: VERIFIED
- Date: 2026-09-15
- Category: REGRESSION
- Evidence classification: CONFIRMED
- Affected scope: full-history TruffleHog false-positive policy after protected squash merge.

## Evidence
PR #89 passed Secret Scan on its exact synchronized head. The protected squash merge created `main` commit `3f6fb666241135be0f6f5071994bcf4deeb75326`; exact-main Secret Scan then failed with one unverified URI result at `crates/kaspa-gateway-security/src/lib.rs:374`. TruffleHog reported zero verified secrets.

## Root Cause
The fail-closed result policy allowed the original synthetic URI fingerprint only at historical commit `8f209ba516707b11098bd962972da38157346833`. Squash integration legitimately recreated the same synthetic line under a second commit identity, so the exact-main result no longer matched the single allowed tuple.

## Fix / Decision
Keep full-history scanning, `verified,unknown` coverage, and fail-on-scan-errors unchanged. Allow exactly the original and squash-generated historical tuples, each at most once. Any commit/path/line/detector/decoder/verification drift remains a failure.

## Verification
- Policy regression suite: PASS.
- Python syntax compilation: PASS.
- `git diff --check`: PASS.
- Graphify code-only extraction/diagnostics: PASS with the checker and regression-test nodes resolved.

## Regression Protection
`test-check-trufflehog-results.py` now proves both exact historical fingerprints are accepted together, duplicate occurrences are rejected per fingerprint, and any fingerprint drift, verified result, or malformed output still fails closed.

## Remaining Risk
Protected PR and exact-main CI still own integration proof. The exception must not expand to detector-, path-, or verification-wide suppression.

## NEXT ACTION
Integrate this narrow policy correction through a protected PR, require exact-head CI, then require exact-main Secret Scan and all other main checks to pass before any release or deployment decision.

## DO NOT REPEAT
Do not switch to verified-only scanning, exclude the URI detector or affected path, rewrite Git history, or add wildcard/commit-prefix exceptions to make the security gate green.
