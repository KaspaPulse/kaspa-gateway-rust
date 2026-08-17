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

function forbidMatch(text, pattern, description) {
  if (pattern.test(text)) {
    failures.push(`Invalid continuity state: ${description}`);
  }
}

const agents = readRequired("AGENTS.md");
const state = readRequired("PROJECT_STATE.md");
const plans = readRequired("PLANS.md");
const adrIndex = readRequired("docs/adr/README.md");
const continuityAdr = readRequired(
  "docs/adr/0011-repository-native-project-continuity.md",
);
const releaseRunbook = readRequired("docs/runbooks/desktop-release.md");
const architectureIndex = readRequired("docs/architecture/README.md");

if (agents) {
  requireMatch(
    agents,
    /## Session Start and Continuity/i,
    "session-start protocol in AGENTS.md",
  );
  requireMatch(
    agents,
    /PROJECT_STATE\.md/i,
    "PROJECT_STATE.md ownership in AGENTS.md",
  );
  requireMatch(
    agents,
    /conversation memory[\s\S]*advisory/i,
    "conversation-memory-is-advisory rule",
  );
  requireMatch(
    agents,
    /meaningful state transition/i,
    "meaningful-state-transition reconciliation rule",
  );
  requireMatch(
    agents,
    /PLANS\.md[^\n]*active multi-stage/i,
    "PLANS.md active-work ownership rule",
  );
}

if (state) {
  requireMatch(state, /^# PROJECT STATE$/m, "PROJECT_STATE.md title");
  requireMatch(state, /## Desired State/i, "Desired State section");
  requireMatch(state, /## Actual State/i, "Actual State section");
  requireMatch(state, /### CI/i, "actual CI state section");
  requireMatch(state, /### Staging/i, "staging classification section");
  requireMatch(
    state,
    /### Production \/ Live Runtime/i,
    "production/live-runtime classification section",
  );
  requireMatch(state, /## Drift/i, "Drift section");
  requireMatch(
    state,
    /## Last Verified Validation/i,
    "evidence-based validation section",
  );
  requireMatch(state, /## NEXT ACTION/i, "precise NEXT ACTION section");
  requireMatch(state, /## Resume Instructions/i, "resume protocol");
  requireMatch(
    state,
    /Verified code baseline[^\n]*[0-9a-f]{40}/i,
    "historical verified code baseline",
  );
  requireMatch(
    state,
    /State-document commit:[^\n]*derive dynamically/i,
    "dynamic state-document commit rule",
  );
  requireMatch(
    state,
    /Current remote main:[^\n]*VERIFY DYNAMICALLY/i,
    "dynamic remote-main rule",
  );
  requireMatch(
    state,
    /Current HEAD:[^\n]*(?:VERIFY DYNAMICALLY|derive dynamically)/i,
    "dynamic HEAD rule",
  );
  requireMatch(
    state,
    /Working tree:[^\n]*(?:CLEAN|DIRTY|NOT VERIFIED)/i,
    "working-tree classification",
  );
  requireMatch(state, /NOT VERIFIED/i, "explicit NOT VERIFIED classification");

  const staticCurrentSha =
    /^(?:-\s*)?(?:Current HEAD|Current remote main|Last verified live main|Live main last verified at):[^\n]*\b[0-9a-f]{40}\b/gim;
  forbidMatch(
    state,
    staticCurrentSha,
    "current HEAD/main labels must not embed a static 40-character SHA; use a historical verified baseline plus dynamic current-state commands",
  );
}

if (plans) {
  const inactivePlan = /NO ACTIVE MULTI-STAGE PLAN/i.test(plans);
  if (inactivePlan) {
    requireMatch(plans, /## Usage/i, "inactive PLANS usage contract");
    requireMatch(
      plans,
      /## Most Recent Completed Plan/i,
      "inactive PLANS completed-plan pointer",
    );
    requireMatch(
      plans,
      /PROJECT_STATE\.md/i,
      "inactive PLANS pointer to canonical current state",
    );
    requireMatch(
      plans,
      /Git(?:Hub)?(?: Releases)?|Git\/PRs/i,
      "inactive PLANS pointer to durable history",
    );
    forbidMatch(
      plans,
      /^## (?:Objective|Success Criteria|Milestones|Progress|Completion Criteria)$/gim,
      "inactive PLANS.md must not retain an active execution-plan body",
    );
  } else {
    requireMatch(plans, /## Objective/i, "active PLANS objective");
    requireMatch(plans, /## Success Criteria/i, "active PLANS success criteria");
    requireMatch(plans, /## Milestones/i, "active PLANS milestones");
    requireMatch(plans, /## Progress/i, "active PLANS progress");
    requireMatch(
      plans,
      /## Completion Criteria/i,
      "active PLANS completion criteria",
    );
  }
}

if (adrIndex) {
  requireMatch(
    adrIndex,
    /Proposed[\s\S]*Accepted[\s\S]*Deprecated[\s\S]*Superseded/i,
    "ADR lifecycle states",
  );
  requireMatch(
    adrIndex,
    /0010[\s\S]*0011/i,
    "ADR numbering continuity/index",
  );
}

if (continuityAdr) {
  requireMatch(
    continuityAdr,
    /Status: Accepted/i,
    "accepted continuity ADR status",
  );
  requireMatch(
    continuityAdr,
    /source-of-truth/i,
    "source-of-truth decision",
  );
  requireMatch(
    continuityAdr,
    /NO ACTIVE MULTI-STAGE PLAN/i,
    "inactive-plan lifecycle decision",
  );
  requireMatch(
    continuityAdr,
    /self-stale|static SHA/i,
    "self-stale/static-current-SHA decision",
  );
}

if (releaseRunbook) {
  requireMatch(releaseRunbook, /## Preconditions/i, "release preconditions");
  requireMatch(
    releaseRunbook,
    /## Abort Conditions/i,
    "release abort conditions",
  );
  requireMatch(
    releaseRunbook,
    /## Post-Publication Verification/i,
    "post-publication verification",
  );
  requireMatch(
    releaseRunbook,
    /explicit user authorization/i,
    "explicit publication authorization gate",
  );
}

if (architectureIndex) {
  requireMatch(
    architectureIndex,
    /local-first Rust\/Tauri desktop control plane/i,
    "architecture identity",
  );
  requireMatch(
    architectureIndex,
    /raw runtime log panes/i,
    "raw-log invariant",
  );
}

const combined = [
  agents,
  state,
  plans,
  adrIndex,
  continuityAdr,
  releaseRunbook,
  architectureIndex,
].join("\n");
const forbiddenSecretAssignment =
  /\b(?:GITHUB_TOKEN|RELEASE_ADMIN_TOKEN|DATABASE_URL|EMAIL_API_KEY|CLOUDFLARE_API_TOKEN|PASSWORD|PRIVATE_KEY|CLIENT_SECRET|API_TOKEN)\s*=\s*[^\s`]+/i;
if (forbiddenSecretAssignment.test(combined)) {
  failures.push(
    "Possible secret value assignment found in continuity documentation.",
  );
}

if (failures.length) {
  console.error("KGW project continuity gate FAILED");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("KGW project continuity gate PASSED");
console.log(
  "Canonical continuity files, dynamic Git-state semantics, plan lifecycle, evidence sections, ADR lifecycle, and release runbook are present.",
);
