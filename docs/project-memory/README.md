# Permanent Project Memory

Material engineering knowledge that must survive sessions belongs here. Use stable identifiers and explicit status so a future engineer can distinguish active findings from resolved history.

## Categories
- `BUGS/` — reproducible defects (`BUG-NNNN`).
- `REGRESSIONS/` — previously working behavior that returned or degraded (`REG-NNNN`).
- `SECURITY/` — security findings with evidence classification (`SEC-NNNN`).
- `INCIDENTS/` — material operational/security incidents (`INC-NNNN`).
- `DECISIONS/` — lightweight decision records (`DEC-NNNN`); consequential architecture/process decisions belong in `docs/adr/` instead.
- `KNOWN_FAILURES/` — understood failure modes that remain relevant (`FAIL-NNNN`).

## Status
Use one of: `OPEN`, `IN PROGRESS`, `RESOLVED`, `VERIFIED`, `NOT REPRODUCIBLE`, `DEFERRED`, `BLOCKED`, `REQUIRES ACTION`, `DUPLICATE`, `FALSE POSITIVE`.

## Minimum Record
Each material record includes ID, status, date, affected scope, evidence, root cause when known, action/fix, verification, regression protection, remaining risk, `NEXT ACTION`, and `DO NOT REPEAT` when relevant.

Use `TEMPLATE.md` for new material records and keep category-specific guidance in each category `README.md`.

Never store credentials or secret values here.
