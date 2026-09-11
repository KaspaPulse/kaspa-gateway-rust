# SEC-0003: RELEASE_ADMIN_TOKEN repository-secret retirement

- Status: VERIFIED
- Date: 2026-09-11
- Category: SECURITY
- Affected scope: GitHub Actions repository secrets, `.github/workflows/`, continuity/security gate, and durable project state.

## Evidence
Repository-wide source inspection found no active workflow or application reference to `RELEASE_ADMIN_TOKEN`; the remaining tracked references were policy/state text describing its retirement. `gh secret list -R KaspaPulse/kaspa-gateway-rust` showed the repository Actions secret before removal. After explicit owner authorization, `gh secret remove RELEASE_ADMIN_TOKEN -R KaspaPulse/kaspa-gateway-rust` completed and a second secret-name listing confirmed that only the unrelated Cloudflare secret names remained.

No secret value was read, printed, stored, or copied. The associated fine-grained PAT itself has no value or identifier in the repository and its account-level revocation remains independently NOT VERIFIED.

## Root Cause
The short-lived release administration credential remained configured at repository scope after the release/recovery workflow no longer referenced it. Configuration persistence outlived code dependency, leaving unnecessary credential exposure surface even though the current workflows had already stopped consuming the secret.

## Fix / Decision
Remove the unused repository Actions secret after proving current tracked workflows/code do not reference it. Permanently forbid future `.github/workflows/*.yml|yaml` references to the retired `RELEASE_ADMIN_TOKEN` name through the existing continuity/security gate. Do not infer or revoke any unrelated PAT from repository metadata.

## Verification
- Pre-removal tracked-source check: PASS; no active repository dependency on the secret name.
- Repository secret-name listing before removal: secret present by name.
- Repository secret removal: PASS.
- Repository secret-name listing after removal: `RELEASE_ADMIN_TOKEN` absent.
- No secret value handling occurred.
- Continuity/security gate regression test injects the retired name into a workflow fixture and must fail closed.

## Regression Protection
`tools/kgw_project_continuity_gate.cjs` scans tracked GitHub Actions workflow YAML and fails if the retired name is referenced. `tools/kgw_project_continuity_gate_tests.cjs` includes a negative fixture proving that behavior. This protects repository code from silently reintroducing dependency on a credential intentionally removed from GitHub settings.

## Remaining Risk
The repository-scoped secret is retired. Account-level revocation/deletion of the historical fine-grained PAT remains NOT VERIFIED because this repository stores neither its value nor an account-level token identifier and the current authorized tooling does not expose a safe revocation target.

## NEXT ACTION
Keep `RELEASE_ADMIN_TOKEN` absent. If an authorized account-level surface later identifies the exact historical fine-grained PAT, revoke/delete that exact token and record verification without recording its value. Otherwise do not guess among account credentials.

## DO NOT REPEAT
Do not recreate `RELEASE_ADMIN_TOKEN`, do not add a workflow reference to it, do not record secret/token values in project memory, and do not revoke an unrelated credential merely to clear the remaining account-level NOT VERIFIED status.
