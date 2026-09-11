# SEC-0002: E2E npm dependency policy and upstream residual risk

- Status: DEFERRED
- Date: 2026-09-10
- Category: SECURITY
- Affected scope: `e2e/package.json`, `e2e/package-lock.json`, `.github/workflows/ci.yml`, `tools/kgw_npm_dependency_policy_gate.cjs`, and `docs/security/npm-dependency-policy.json`.

## Evidence
PR #76 failed the blocking `Audit E2E npm tree` step because `@wdio/mocha-framework` 9.31.5 resolved Mocha 10.8.2 and `js-yaml` 4.3.1, which npm audit reports as High severity under GHSA-2883-xcg3-v3hh. A Node 24.19.0 / npm 11.17.0 local reproduction matched CI. Updating the supported WebdriverIO 9.31 line to 9.31.7 resolves `js-yaml` to 4.3.2 and removes the High finding. The resulting audit has 0 Critical, 0 High, 0 Moderate, and 3 Low vulnerability nodes, all from the single GHSA-73rr-hh4g-fpgx chain through Mocha 11.8.0 and `diff` 7.0.0. The install log also has two remaining upstream deprecation warnings: `glob` 10.5.0 and `whatwg-encoding` 3.1.1.

## Root Cause
The E2E dependency set was exactly pinned to WebdriverIO 9.31.5 while its transitive Mocha tree still resolved a newly vulnerable `js-yaml` release. CI enforced only Moderate-or-higher npm audit findings and performed that check after the expensive Rust lane. Deprecation warnings were visible in logs but had no fail-closed drift policy. The latest supported WebdriverIO 9.31.7 line removes the High finding and the older `glob` 8 / `inflight` warnings, but upstream still constrains Mocha to 11.8.0 with `diff ^7.0.0`, and current WebdriverIO/Cheerio paths still contain `glob` 10.5.0 and `whatwg-encoding` 3.1.1.

## Fix / Decision
Upgrade `@wdio/cli`, `@wdio/local-runner`, `@wdio/mocha-framework`, and `webdriverio` to exact 9.31.7 pins. Add a repository-native npm policy gate that evaluates Low severity too, verifies exact advisory identities and dependency paths, checks the lockfile versions, rejects any new deprecation warning, rejects stale exceptions, and expires the accepted residual risk on 2026-10-10. Run that policy immediately after npm installation in blocking CI. Do not force unsupported major overrides merely to hide warnings.

## Verification
`npm ci --ignore-scripts` succeeds in both npm workspaces. The desktop policy gate reports 0 Critical/High/Moderate/Low and 0 deprecations. The E2E policy gate reports 0 Critical, 0 High, 0 Moderate, 3 explicitly accepted Low nodes, and exactly 2 accepted deprecations. E2E `npm run lint` and `npm run check` pass. The policy-gate regression suite passes one positive fixture and six fail-closed negative cases covering new High, new Low, expired review, new deprecation, lock drift, and stale exception behavior.

## Regression Protection
Blocking CI captures npm install logs and executes `tools/kgw_npm_dependency_policy_gate.cjs` for both desktop and E2E before Rust compilation. `docs/security/npm-dependency-policy.json` is the machine-readable exception contract. Any unapproved vulnerability, severity/path/advisory drift, deprecation-set drift, lockfile drift, stale exception, or expired review date fails the gate. `tools/kgw_npm_dependency_policy_gate_tests.cjs` exercises the fail-closed behavior.

## Upstream Review — 2026-09-11
A fresh registry review confirmed there is still no supported compatible removal path. Latest `@wdio/mocha-framework`, `@wdio/cli`, `@wdio/local-runner`, and `webdriverio` remain 9.31.7. `@wdio/mocha-framework` still declares Mocha `^11.8.0`; Mocha 11.8.0 still declares `diff ^7.0.0`, while the non-vulnerable current lines are Mocha 12.0.0 and `diff` 9.0.0 outside WebdriverIO's supported dependency range. Latest `@wdio/config` still declares `glob ^10.2.2`, and latest Cheerio 1.2.0 still declares `encoding-sniffer ^0.2.1`, which retains `whatwg-encoding` 3.1.1.

A clean `npm ci --ignore-scripts` followed by `npm audit --json` reproduced exactly 0 Critical, 0 High, 0 Moderate, and 3 Low nodes (`@wdio/mocha-framework`, `mocha`, `diff`), plus exactly the two already-accepted deprecation warnings. The review date is advanced to 2026-09-11, but the mandatory expiry remains **2026-10-10**; no exception was broadened or extended.

## Remaining Risk
One Low-severity upstream advisory remains represented as three npm audit vulnerability nodes: GHSA-73rr-hh4g-fpgx through `@wdio/mocha-framework` 9.31.7 -> Mocha 11.8.0 -> `diff` 7.0.0. Two deprecated transitive packages remain upstream constrained: `glob` 10.5.0 and `whatwg-encoding` 3.1.1. These are explicitly time-bounded and cannot expand silently.

## NEXT ACTION
Review upstream WebdriverIO/Mocha/Cheerio dependency releases no later than 2026-10-10. Remove each exception immediately when a supported compatible path eliminates it; the CI gate intentionally fails after the review deadline.

## DO NOT REPEAT
Do not downgrade WebdriverIO or add unsupported major-version overrides solely to make npm output quiet. Do not weaken the audit threshold or add broad advisory/deprecation ignores; update the exact machine-readable policy only with evidence and an expiry.
