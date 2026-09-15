# BUG-0013: AUD-013 — Settings tab deferred restore overrides explicit navigation

- Status: VERIFIED
- Date: 2026-09-14
- Category: BUG
- Audit finding ID: AUD-013
- Severity: MEDIUM
- Affected scope: desktop shell top-level navigation and saved-tab restore lifecycle.

## Evidence
A real WebDriver Settings click reached `openTab(settings)`, `activateTab(settings)`, and a visible Settings panel. Runtime trace then showed `kgwShellScheduleSavedMainTabRestoreR102C("ensure-active-no-change")` dispatching a delay=0 callback while `activeHash=#settings`; that callback reopened the stale saved `kaspa-bridge` tab.

## Root Cause
Saved-tab restore scheduling had no explicit-navigation generation/state. Display-preference hydration could schedule a new restore after a newer navigation intent. The old `activeTabId === pending` guard compared current state only and could not distinguish stale intent. WebDriver clicks are `isTrusted=false`, so `isTrusted` cannot define explicit user navigation for this lifecycle.

## Fix / Decision
Keep startup restoration, but give explicit top-level navigation its own generation. Explicit navigation invalidates pending bootstrap restores, future stale scheduling is suppressed, and an async explicit open activates only if its generation is still the latest. No Settings-specific hardcode was added.

## Verification
`tools/kgw_aud013_navigation_tests.cjs` PASS. Real WebDriver regression PASS on source-bound Desktop binary SHA-256 `2e10c23295951c6a3ef581888f71c30c1965d6b6ea3d1a41b05d51b877c6c2e7`: startup restored saved `kaspa-bridge`; Settings remained the sole active tab/panel after stale restore delays; rapid bridge→Settings left Settings active. File-scoped ESLint count was unchanged before/after the fix (32 errors, 3 warnings).

## Regression Protection
`tools/kgw_aud013_navigation_tests.cjs` guards stale pending restore invalidation, startup restore without interaction, latest explicit generation wins, no post-intent restore scheduling, and source integration. The targeted Desktop WebDriver evidence protects the actual shell path.

## Remaining Risk
NONE known for this defect. Repository-wide preexisting frontend lint debt is tracked separately and is not caused by AUD-013.

## NEXT ACTION
NONE — VERIFIED. Reuse the existing regression evidence unless shell navigation or restore lifecycle source changes.

## DO NOT REPEAT
Do not disable saved-tab restoration globally, special-case Settings, or use `event.isTrusted` as the sole explicit-navigation signal. Do not repeat the old navigation probe unless relevant shell lifecycle source changes.
