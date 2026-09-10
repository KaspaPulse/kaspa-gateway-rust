#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const policyPath = path.join(repoRoot, "docs/security/npm-dependency-policy.json");

function sorted(values) {
  return [...values].sort((a, b) => a.localeCompare(b));
}

function sameArray(left, right) {
  return JSON.stringify(sorted(left)) === JSON.stringify(sorted(right));
}

function deprecationKey(item) {
  return `${item.name}@${item.version}`;
}

function extractDeprecations(text) {
  const found = [];
  for (const line of String(text || "").split(/\r?\n/)) {
    const match = line.match(/^npm warn deprecated (.+)@([^:]+):/);
    if (match) found.push({ name: match[1], version: match[2] });
  }
  return found;
}

function viaProfile(via) {
  const viaPackages = [];
  const advisoryUrls = [];
  for (const item of Array.isArray(via) ? via : []) {
    if (typeof item === "string") viaPackages.push(item);
    else if (item && typeof item.url === "string") advisoryUrls.push(item.url);
  }
  return { viaPackages: sorted(viaPackages), advisoryUrls: sorted(advisoryUrls) };
}

function validatePolicySnapshot({ workspacePolicy, audit, lock, installLog, now = new Date() }) {
  const failures = [];
  const allowed = workspacePolicy.allowed_vulnerabilities || [];
  const allowedByName = new Map(allowed.map((item) => [item.name, item]));
  const hasExceptions = allowed.length > 0 || (workspacePolicy.allowed_deprecations || []).length > 0;

  if (hasExceptions) {
    const reviewUntil = workspacePolicy.review_until;
    const expiry = reviewUntil ? new Date(`${reviewUntil}T23:59:59Z`) : null;
    if (!expiry || Number.isNaN(expiry.getTime())) {
      failures.push("Accepted npm dependency risk is missing a valid review_until date.");
    } else if (now.getTime() > expiry.getTime()) {
      failures.push(`Accepted npm dependency risk review expired on ${reviewUntil}.`);
    }
  }

  const packages = lock && lock.packages ? lock.packages : {};
  for (const [lockPath, expectedVersion] of Object.entries(workspacePolicy.required_lock_versions || {})) {
    const actualVersion = packages[lockPath] && packages[lockPath].version;
    if (actualVersion !== expectedVersion) {
      failures.push(`Lockfile drift: ${lockPath} expected ${expectedVersion}, found ${actualVersion || "MISSING"}.`);
    }
  }

  const vulnerabilities = audit && audit.vulnerabilities ? audit.vulnerabilities : {};
  const actualNames = Object.keys(vulnerabilities);
  for (const name of actualNames) {
    const finding = vulnerabilities[name] || {};
    const expected = allowedByName.get(name);
    if (!expected) {
      failures.push(`Unapproved npm vulnerability: ${name} (${finding.severity || "unknown"}).`);
      continue;
    }
    if (finding.severity !== expected.severity) {
      failures.push(`Severity drift for ${name}: expected ${expected.severity}, found ${finding.severity || "unknown"}.`);
    }
    const profile = viaProfile(finding.via);
    if (!sameArray(profile.viaPackages, expected.via_packages || [])) {
      failures.push(`Dependency-path drift for ${name}: via package set changed.`);
    }
    if (!sameArray(profile.advisoryUrls, expected.advisory_urls || [])) {
      failures.push(`Advisory identity drift for ${name}: advisory URL set changed.`);
    }
  }

  for (const expected of allowed) {
    if (!Object.prototype.hasOwnProperty.call(vulnerabilities, expected.name)) {
      failures.push(`Stale npm vulnerability exception: ${expected.name} is no longer reported.`);
    }
  }

  const metadata = audit && audit.metadata && audit.metadata.vulnerabilities;
  if (!metadata) {
    failures.push("npm audit JSON is missing metadata.vulnerabilities.");
  } else {
    for (const severity of ["critical", "high", "moderate"]) {
      if ((metadata[severity] || 0) !== 0) {
        failures.push(`npm audit reports ${metadata[severity]} ${severity} vulnerabilities.`);
      }
    }
    const expectedLow = allowed.filter((item) => item.severity === "low").length;
    if ((metadata.low || 0) !== expectedLow) {
      failures.push(`npm low-vulnerability count drift: expected ${expectedLow}, found ${metadata.low || 0}.`);
    }
  }

  const actualDeprecations = extractDeprecations(installLog);
  const actualDeprecationKeys = sorted(new Set(actualDeprecations.map(deprecationKey)));
  const expectedDeprecationKeys = sorted(
    new Set((workspacePolicy.allowed_deprecations || []).map(deprecationKey)),
  );
  if (!sameArray(actualDeprecationKeys, expectedDeprecationKeys)) {
    failures.push(
      `npm deprecation warning set drift: expected [${expectedDeprecationKeys.join(", ")}], found [${actualDeprecationKeys.join(", ")}].`,
    );
  }

  return failures;
}

function parseArgs(argv) {
  const args = { workspace: null, ciLog: null };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--workspace") args.workspace = argv[++i];
    else if (argv[i] === "--ci-log") args.ciLog = argv[++i];
    else throw new Error(`Unknown argument: ${argv[i]}`);
  }
  if (!args.workspace) throw new Error("--workspace is required");
  if (!args.ciLog) throw new Error("--ci-log is required");
  return args;
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function main() {
  let args;
  try {
    args = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(`KGW npm dependency policy gate FAILED: ${error.message}`);
    process.exit(2);
  }

  const policy = readJson(policyPath);
  if (policy.schema_version !== 1) {
    console.error(`KGW npm dependency policy gate FAILED: unsupported schema ${policy.schema_version}`);
    process.exit(2);
  }
  const workspacePolicy = policy.workspaces && policy.workspaces[args.workspace];
  if (!workspacePolicy) {
    console.error(`KGW npm dependency policy gate FAILED: unknown workspace ${args.workspace}`);
    process.exit(2);
  }

  const workspaceDir = path.join(repoRoot, workspacePolicy.path);
  const audit = spawnSync("npm", ["audit", "--json", "--audit-level=low"], {
    cwd: workspaceDir,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  });
  const auditOutputPath = path.join(workspaceDir, "npm-audit.json");
  fs.writeFileSync(auditOutputPath, audit.stdout || "", "utf8");
  if (![0, 1].includes(audit.status)) {
    console.error(`KGW npm dependency policy gate FAILED: npm audit operational status ${audit.status}.`);
    if (audit.stderr) console.error(audit.stderr.trim());
    process.exit(1);
  }

  let auditJson;
  try {
    auditJson = JSON.parse(audit.stdout || "");
  } catch (error) {
    console.error(`KGW npm dependency policy gate FAILED: invalid npm audit JSON: ${error.message}`);
    process.exit(1);
  }

  const lock = readJson(path.join(workspaceDir, "package-lock.json"));
  const installLogPath = path.resolve(repoRoot, args.ciLog);
  if (!fs.existsSync(installLogPath)) {
    console.error(`KGW npm dependency policy gate FAILED: install log missing: ${args.ciLog}`);
    process.exit(1);
  }
  const installLog = fs.readFileSync(installLogPath, "utf8");
  const failures = validatePolicySnapshot({ workspacePolicy, audit: auditJson, lock, installLog });
  if (failures.length > 0) {
    console.error(`KGW npm dependency policy gate FAILED for ${args.workspace}:`);
    for (const failure of failures) console.error(`- ${failure}`);
    process.exit(1);
  }

  fs.rmSync(auditOutputPath, { force: true });
  const counts = auditJson.metadata.vulnerabilities;
  console.log(
    `KGW npm dependency policy gate PASSED: workspace=${args.workspace}; critical=${counts.critical || 0}; high=${counts.high || 0}; moderate=${counts.moderate || 0}; accepted_low=${counts.low || 0}; accepted_deprecations=${extractDeprecations(installLog).length}; review_until=${workspacePolicy.review_until || "NONE"}`,
  );
}

module.exports = { extractDeprecations, validatePolicySnapshot };
if (require.main === module) main();
