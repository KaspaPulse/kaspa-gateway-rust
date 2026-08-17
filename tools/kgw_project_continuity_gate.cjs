#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const failures = [];

function readRequired(relativePath) {
  const fullPath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(fullPath) || !fs.statSync(fullPath).isFile()) {
    failures.push(`Missing required continuity file: ${relativePath}`);
    return "";
  }
  const text = fs.readFileSync(fullPath, "utf8");
  if (!text.trim()) {
    failures.push(`Continuity file is empty: ${relativePath}`);
  }
  return text;
}

function requireMatch(text, pattern, description) {
  if (!pattern.test(text)) {
    failures.push(`Missing continuity policy: ${description}`);
  }
}

const agents = readRequired("AGENTS.md");
const state = readRequired("PROJECT_STATE.md");
const plans = readRequired("PLANS.md");
const adrIndex = readRequired("docs/adr/README.md");
const continuityAdr = readRequired("docs/adr/0011-repository-native-project-continuity.md");
const releaseRunbook = readRequired("docs/runbooks/desktop-release.md");
const architectureIndex = readRequired("docs/architecture/README.md");

if (agents) {
  requireMatch(agents, /## Session Start and Continuity/i, "session-start protocol in AGENTS.md");
  requireMatch(agents, /PROJECT_STATE\.md/i, "PROJECT_STATE.md ownership in AGENTS.md");
  requireMatch(agents, /conversation memory[\s\S]*advisory/i, "conversation-memory-is-advisory rule");
  requireMatch(agents, /meaningful state transition/i, "meaningful-state-transition reconciliation rule");
}

if (state) {
  requireMatch(state, /^# PROJECT STATE$/m, "PROJECT_STATE.md title");
  requireMatch(state, /## Desired State/i, "Desired State section");
  requireMatch(state, /## Actual State/i, "Actual State section");
  requireMatch(state, /## Drift/i, "Drift section");
  requireMatch(state, /## NEXT ACTION/i, "precise NEXT ACTION section");
  requireMatch(state, /## Resume Instructions/i, "resume protocol");
  requireMatch(state, /Current HEAD:[^\n]*derive dynamically/i, "dynamic HEAD rule");
  requireMatch(state, /NOT VERIFIED/i, "explicit NOT VERIFIED classification");
}

if (plans) {
  requireMatch(plans, /## Objective/i, "PLANS objective");
  requireMatch(plans, /## Milestones/i, "PLANS milestones");
  requireMatch(plans, /## Progress/i, "PLANS progress");
  requireMatch(plans, /## Completion Criteria/i, "PLANS completion criteria");
}

if (adrIndex) {
  requireMatch(adrIndex, /Proposed[\s\S]*Accepted[\s\S]*Deprecated[\s\S]*Superseded/i, "ADR lifecycle states");
  requireMatch(adrIndex, /0010[\s\S]*0011/i, "ADR numbering continuity/index");
}

if (continuityAdr) {
  requireMatch(continuityAdr, /Status: Accepted/i, "accepted continuity ADR status");
  requireMatch(continuityAdr, /source-of-truth/i, "source-of-truth decision");
}

if (releaseRunbook) {
  requireMatch(releaseRunbook, /## Preconditions/i, "release preconditions");
  requireMatch(releaseRunbook, /## Abort Conditions/i, "release abort conditions");
  requireMatch(releaseRunbook, /## Post-Publication Verification/i, "post-publication verification");
  requireMatch(releaseRunbook, /explicit user authorization/i, "explicit publication authorization gate");
}

if (architectureIndex) {
  requireMatch(architectureIndex, /local-first Rust\/Tauri desktop control plane/i, "architecture identity");
  requireMatch(architectureIndex, /raw runtime log panes/i, "raw-log invariant");
}

const combined = [agents, state, plans, adrIndex, continuityAdr, releaseRunbook, architectureIndex].join("\n");
const forbiddenSecretAssignment = /\b(?:GITHUB_TOKEN|DATABASE_URL|EMAIL_API_KEY|CLOUDFLARE_API_TOKEN|PASSWORD|PRIVATE_KEY)\s*=\s*[^\s`]+/i;
if (forbiddenSecretAssignment.test(combined)) {
  failures.push("Possible secret value assignment found in continuity documentation.");
}

if (failures.length) {
  console.error("KGW project continuity gate FAILED");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("KGW project continuity gate PASSED");
console.log("Canonical continuity files, source-of-truth separation, resume protocol, ADR lifecycle, and release runbook are present.");
