#!/usr/bin/env node
"use strict";

const {
  validatePolicySnapshot,
} = require("./kgw_npm_dependency_policy_gate.cjs");

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

const basePolicy = {
  review_until: "2026-10-10",
  allowed_vulnerabilities: [
    {
      name: "diff",
      severity: "low",
      via_packages: [],
      advisory_urls: ["https://github.com/advisories/GHSA-73rr-hh4g-fpgx"],
    },
    {
      name: "mocha",
      severity: "low",
      via_packages: ["diff"],
      advisory_urls: [],
    },
    {
      name: "@wdio/mocha-framework",
      severity: "low",
      via_packages: ["mocha"],
      advisory_urls: [],
    },
  ],
  allowed_deprecations: [
    { name: "glob", version: "10.5.0" },
    { name: "whatwg-encoding", version: "3.1.1" },
  ],
  required_lock_versions: {
    "node_modules/@wdio/mocha-framework": "9.31.7",
    "node_modules/mocha": "11.8.0",
    "node_modules/mocha/node_modules/diff": "7.0.0",
    "node_modules/js-yaml": "4.3.2",
    "node_modules/glob": "10.5.0",
    "node_modules/whatwg-encoding": "3.1.1",
  },
};

const baseAudit = {
  vulnerabilities: {
    diff: {
      severity: "low",
      via: [{ url: "https://github.com/advisories/GHSA-73rr-hh4g-fpgx" }],
    },
    mocha: { severity: "low", via: ["diff"] },
    "@wdio/mocha-framework": { severity: "low", via: ["mocha"] },
  },
  metadata: {
    vulnerabilities: { info: 0, low: 3, moderate: 0, high: 0, critical: 0, total: 3 },
  },
};

const baseLock = {
  packages: {
    "node_modules/@wdio/mocha-framework": { version: "9.31.7" },
    "node_modules/mocha": { version: "11.8.0" },
    "node_modules/mocha/node_modules/diff": { version: "7.0.0" },
    "node_modules/js-yaml": { version: "4.3.2" },
    "node_modules/glob": { version: "10.5.0" },
    "node_modules/whatwg-encoding": { version: "3.1.1" },
  },
};

const baseLog = [
  "npm warn deprecated glob@10.5.0: upstream constraint",
  "npm warn deprecated whatwg-encoding@3.1.1: upstream constraint",
].join("\n");

function validate(overrides = {}) {
  return validatePolicySnapshot({
    workspacePolicy: overrides.policy || clone(basePolicy),
    audit: overrides.audit || clone(baseAudit),
    lock: overrides.lock || clone(baseLock),
    installLog: Object.prototype.hasOwnProperty.call(overrides, "log") ? overrides.log : baseLog,
    now: overrides.now || new Date("2026-09-10T17:00:00Z"),
  });
}

function expectPass(label, overrides = {}) {
  const failures = validate(overrides);
  if (failures.length !== 0) {
    throw new Error(`${label}: expected PASS\n${failures.join("\n")}`);
  }
}

function expectFail(label, expectedText, mutate) {
  const policy = clone(basePolicy);
  const audit = clone(baseAudit);
  const lock = clone(baseLock);
  const state = { policy, audit, lock, log: baseLog, now: new Date("2026-09-10T17:00:00Z") };
  mutate(state);
  const failures = validate(state);
  if (!failures.some((failure) => failure.includes(expectedText))) {
    throw new Error(`${label}: expected failure containing ${JSON.stringify(expectedText)}\n${failures.join("\n")}`);
  }
}

expectPass("baseline accepted-risk snapshot");

expectFail("new high vulnerability", "Unapproved npm vulnerability: injected-high", ({ audit }) => {
  audit.vulnerabilities["injected-high"] = { severity: "high", via: [] };
  audit.metadata.vulnerabilities.high = 1;
  audit.metadata.vulnerabilities.total += 1;
});

expectFail("new low vulnerability", "Unapproved npm vulnerability: injected-low", ({ audit }) => {
  audit.vulnerabilities["injected-low"] = { severity: "low", via: [] };
  audit.metadata.vulnerabilities.low = 4;
  audit.metadata.vulnerabilities.total = 4;
});

expectFail("expired exception review", "review expired", (state) => {
  state.now = new Date("2026-10-11T00:00:00Z");
});

expectFail("new deprecation warning", "deprecation warning set drift", (state) => {
  state.log += "\nnpm warn deprecated surprise-package@1.0.0: new warning";
});

expectFail("lockfile drift", "node_modules/js-yaml expected 4.3.2", ({ lock }) => {
  lock.packages["node_modules/js-yaml"].version = "4.3.1";
});

expectFail("stale vulnerability exception", "Stale npm vulnerability exception: diff", ({ audit }) => {
  delete audit.vulnerabilities.diff;
  audit.metadata.vulnerabilities.low = 2;
  audit.metadata.vulnerabilities.total = 2;
});

console.log("KGW npm dependency policy gate regression tests PASSED");
console.log("Positive snapshot and six fail-closed negative cases behaved as required.");
