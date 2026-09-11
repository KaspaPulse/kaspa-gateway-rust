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

function readMarkdownRecords(relativeDir) {
  const fullDir = path.join(repoRoot, relativeDir);
  if (!fs.existsSync(fullDir) || !fs.statSync(fullDir).isDirectory()) {
    failures.push(`Missing required continuity directory: ${relativeDir}`);
    return [];
  }
  return fs
    .readdirSync(fullDir, { withFileTypes: true })
    .filter(
      (entry) =>
        entry.isFile() &&
        entry.name.endsWith(".md") &&
        !["README.md", "TEMPLATE.md"].includes(entry.name),
    )
    .map((entry) => {
      const relativePath = path.join(relativeDir, entry.name);
      return { relativePath, text: readRequired(relativePath) };
    });
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

function readWorkflowSources() {
  const relativeDir = ".github/workflows";
  const fullDir = path.join(repoRoot, relativeDir);
  if (!fs.existsSync(fullDir) || !fs.statSync(fullDir).isDirectory()) {
    failures.push(`Missing required continuity directory: ${relativeDir}`);
    return [];
  }
  return fs
    .readdirSync(fullDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && /\.ya?ml$/i.test(entry.name))
    .map((entry) => {
      const relativePath = path.join(relativeDir, entry.name);
      return { relativePath, text: fs.readFileSync(path.join(repoRoot, relativePath), "utf8") };
    });
}

const agents = readRequired("AGENTS.md");
const state = readRequired("PROJECT_STATE.md");
const activeTask = readRequired("ACTIVE_TASK.md");
const currentState = readRequired("CURRENT_STATE.md");
const plans = readRequired("PLANS.md");
const continuityPolicy = readRequired(
  "docs/continuity/PROJECT_CONTINUITY_POLICY.md",
);
const handoffLedger = readRequired("docs/handoff-ledger/README.md");
const projectMemory = readRequired("docs/project-memory/README.md");
const bugMemory = readRequired("docs/project-memory/BUGS/README.md");
const regressionMemory = readRequired(
  "docs/project-memory/REGRESSIONS/README.md",
);
const securityMemory = readRequired("docs/project-memory/SECURITY/README.md");
const incidentMemory = readRequired("docs/project-memory/INCIDENTS/README.md");
const decisionMemory = readRequired("docs/project-memory/DECISIONS/README.md");
const knownFailureMemory = readRequired(
  "docs/project-memory/KNOWN_FAILURES/README.md",
);
const handoffTemplate = readRequired("docs/handoff-ledger/TEMPLATE.md");
const memoryTemplate = readRequired("docs/project-memory/TEMPLATE.md");
const handoffRecords = readMarkdownRecords("docs/handoff-ledger");
const memoryRecords = [
  ...readMarkdownRecords("docs/project-memory/BUGS"),
  ...readMarkdownRecords("docs/project-memory/REGRESSIONS"),
  ...readMarkdownRecords("docs/project-memory/SECURITY"),
  ...readMarkdownRecords("docs/project-memory/INCIDENTS"),
  ...readMarkdownRecords("docs/project-memory/DECISIONS"),
  ...readMarkdownRecords("docs/project-memory/KNOWN_FAILURES"),
];
const adrIndex = readRequired("docs/adr/README.md");
const continuityAdr = readRequired(
  "docs/adr/0011-repository-native-project-continuity.md",
);
const releaseRunbook = readRequired("docs/runbooks/desktop-release.md");
const architectureIndex = readRequired("docs/architecture/README.md");
const workflowSources = readWorkflowSources();

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

if (activeTask) {
  requireMatch(activeTask, /^# ACTIVE TASK$/m, "ACTIVE_TASK.md title");
  for (const [pattern, label] of [
    [/## Status/i, "active-task status"],
    [/## Objective/i, "active-task objective"],
    [/## Scope/i, "active-task scope"],
    [/## Current Phase/i, "active-task current phase"],
    [/## Confirmed Progress/i, "active-task confirmed progress"],
    [/## Current Blocker/i, "active-task blocker classification"],
    [/## Last Completed Action/i, "active-task last completed action"],
    [/## Current Action/i, "active-task current action"],
    [/## Next Action/i, "active-task next action"],
    [/## Verification Required/i, "active-task verification contract"],
    [/## Completion Criteria/i, "active-task completion criteria"],
    [/## DO NOT REPEAT/i, "active-task do-not-repeat boundary"],
  ]) requireMatch(activeTask, pattern, label);
}

if (currentState) {
  requireMatch(currentState, /^# CURRENT STATE$/m, "CURRENT_STATE.md title");
  requireMatch(
    currentState,
    /Current HEAD:[^\n]*(?:VERIFY DYNAMICALLY|derive dynamically)/i,
    "current-state dynamic HEAD rule",
  );
  requireMatch(
    currentState,
    /Current remote main:[^\n]*VERIFY DYNAMICALLY/i,
    "current-state dynamic remote-main rule",
  );
  requireMatch(
    currentState,
    /Working tree:[^\n]*(?:CLEAN|DIRTY|NOT VERIFIED)/i,
    "current-state working-tree classification",
  );
  requireMatch(currentState, /## NEXT ACTION/i, "current-state next action");
  requireMatch(currentState, /## DO NOT REPEAT/i, "current-state do-not-repeat");
  requireMatch(currentState, /NOT VERIFIED/i, "current-state explicit not-verified state");
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
  const inactivePlan = /## Status\s*\n+\s*\*\*NO ACTIVE MULTI-STAGE PLAN\*\*\s*(?:\n|$)/i.test(plans);
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

if (continuityPolicy) {
  requireMatch(
    continuityPolicy,
    /READ -> RECOVER -> VERIFY -> RECONCILE -> CHECK -> PRIORITIZE -> CONTINUE -> CHANGE -> TEST -> PROTECT AGAINST REGRESSION -> DOCUMENT -> CHECKPOINT -> HANDOFF/i,
    "full continuity lifecycle",
  );
  requireMatch(continuityPolicy, /## Sources of Truth/i, "source-of-truth policy");
  requireMatch(
    continuityPolicy,
    /Bug -> Reproduce -> Root Cause -> Fix -> Verification -> Regression Protection -> Documentation/i,
    "important-bug lifecycle",
  );
  requireMatch(
    continuityPolicy,
    /Fix -> Strengthen -> Verify -> Protect/i,
    "recurring-problem strengthening lifecycle",
  );
  requireMatch(continuityPolicy, /OWASP ASVS 5\.0\.0/i, "OWASP ASVS security reference");
  requireMatch(
    continuityPolicy,
    /NIST Secure Software Development Framework/i,
    "NIST SSDF security-development reference",
  );
  requireMatch(
    continuityPolicy,
    /Production-impacting and external actions require explicit authorization/i,
    "explicit external-action authorization rule",
  );
  requireMatch(continuityPolicy, /## End-of-Task Contract/i, "end-of-task verification contract");
}

if (handoffLedger) {
  requireMatch(handoffLedger, /LAST CONFIRMED STATE/i, "handoff last confirmed state");
  requireMatch(handoffLedger, /NEXT ACTION/i, "handoff next action");
  requireMatch(handoffLedger, /DO NOT REPEAT/i, "handoff do-not-repeat boundary");
  requireMatch(handoffLedger, /Timestamp/i, "handoff timestamp field");
  requireMatch(handoffLedger, /Test|evidence/i, "handoff verification/evidence field");
}

if (handoffTemplate) {
  for (const [pattern, label] of [
    [/LAST CONFIRMED STATE/i, "handoff template last confirmed state"],
    [/COMPLETED \/ VERIFIED/i, "handoff template completed/verified"],
    [/EVIDENCE \/ TESTS/i, "handoff template evidence/tests"],
    [/BLOCKERS \/ REMAINING WORK/i, "handoff template blockers/remaining work"],
    [/NEXT ACTION/i, "handoff template next action"],
    [/DO NOT REPEAT/i, "handoff template do-not-repeat"],
  ]) requireMatch(handoffTemplate, pattern, label);
}

if (!handoffRecords.length) {
  failures.push("No durable task checkpoint exists under docs/handoff-ledger/.");
}
for (const record of handoffRecords) {
  for (const [pattern, label] of [
    [/Status:/i, "status"],
    [/Timestamp:/i, "timestamp"],
    [/## LAST CONFIRMED STATE/i, "last confirmed state"],
    [/## (?:COMPLETED|COMPLETED \/ VERIFIED)/i, "completed/verified work"],
    [/## EVIDENCE/i, "evidence"],
    [/## NEXT ACTION/i, "next action"],
    [/## DO NOT REPEAT/i, "do-not-repeat"],
  ]) requireMatch(record.text, pattern, `${record.relativePath}: ${label}`);
}

if (memoryTemplate) {
  for (const [pattern, label] of [
    [/Status:/i, "memory template status"],
    [/## Evidence/i, "memory template evidence"],
    [/## Root Cause/i, "memory template root cause"],
    [/## Verification/i, "memory template verification"],
    [/## Regression Protection/i, "memory template regression protection"],
    [/## NEXT ACTION/i, "memory template next action"],
    [/## DO NOT REPEAT/i, "memory template do-not-repeat"],
  ]) requireMatch(memoryTemplate, pattern, label);
}

if (!memoryRecords.length) {
  failures.push("No durable project-memory record exists under docs/project-memory/.");
}
const memoryStatusPattern =
  /Status:\s*(?:OPEN|IN PROGRESS|RESOLVED|VERIFIED|NOT REPRODUCIBLE|DEFERRED|BLOCKED|REQUIRES ACTION|DUPLICATE|FALSE POSITIVE)/i;
const memoryIdPattern = /^(?:# )?(?:BUG|REG|SEC|INC|DEC|FAIL)-\d{4}:/m;
const memoryIds = new Map();
for (const record of memoryRecords) {
  requireMatch(record.text, memoryIdPattern, `${record.relativePath}: stable ID`);
  const idMatch = record.text.match(/(?:^# )?((?:BUG|REG|SEC|INC|DEC|FAIL)-\d{4}):/m);
  if (idMatch) {
    const stableId = idMatch[1].toUpperCase();
    const previous = memoryIds.get(stableId);
    if (previous) {
      failures.push(`Duplicate project-memory stable ID ${stableId}: ${previous} and ${record.relativePath}.`);
    } else {
      memoryIds.set(stableId, record.relativePath);
    }
  }
  requireMatch(record.text, memoryStatusPattern, `${record.relativePath}: lifecycle status`);
  requireMatch(record.text, /## Evidence/i, `${record.relativePath}: evidence`);
  requireMatch(record.text, /## Root Cause/i, `${record.relativePath}: root cause`);
  requireMatch(record.text, /## Verification/i, `${record.relativePath}: verification`);
  requireMatch(
    record.text,
    /## Regression Protection/i,
    `${record.relativePath}: regression protection`,
  );
  requireMatch(record.text, /## NEXT ACTION/i, `${record.relativePath}: next action`);
  requireMatch(record.text, /## DO NOT REPEAT/i, `${record.relativePath}: do-not-repeat`);
}

if (projectMemory) {
  requireMatch(projectMemory, /BUG-NNNN/i, "bug stable identifier");
  requireMatch(projectMemory, /REG-NNNN/i, "regression stable identifier");
  requireMatch(projectMemory, /SEC-NNNN/i, "security stable identifier");
  requireMatch(projectMemory, /INC-NNNN/i, "incident stable identifier");
  requireMatch(projectMemory, /DEC-NNNN/i, "decision stable identifier");
  requireMatch(projectMemory, /FAIL-NNNN/i, "known-failure stable identifier");
  requireMatch(projectMemory, /OPEN[\s\S]*IN PROGRESS[\s\S]*RESOLVED[\s\S]*VERIFIED/i, "explicit project-memory statuses");
}

for (const [text, idPattern, label] of [
  [bugMemory, /BUG-NNNN/i, "bug memory category"],
  [regressionMemory, /REG-NNNN/i, "regression memory category"],
  [securityMemory, /SEC-NNNN/i, "security memory category"],
  [incidentMemory, /INC-NNNN/i, "incident memory category"],
  [decisionMemory, /DEC-NNNN/i, "decision memory category"],
  [knownFailureMemory, /FAIL-NNNN/i, "known-failure memory category"],
]) {
  if (!text) continue;
  requireMatch(text, idPattern, `${label} stable ID`);
  requireMatch(text, /NEXT ACTION/i, `${label} next-action contract`);
  requireMatch(text, /status/i, `${label} explicit status contract`);
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
  activeTask,
  currentState,
  plans,
  continuityPolicy,
  handoffLedger,
  projectMemory,
  bugMemory,
  regressionMemory,
  securityMemory,
  incidentMemory,
  decisionMemory,
  knownFailureMemory,
  handoffTemplate,
  memoryTemplate,
  ...handoffRecords.map((record) => record.text),
  ...memoryRecords.map((record) => record.text),
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

for (const workflow of workflowSources) {
  if (/\bRELEASE_ADMIN_TOKEN\b/.test(workflow.text)) {
    failures.push(
      `Retired GitHub Actions secret RELEASE_ADMIN_TOKEN must not be referenced by workflow: ${workflow.relativePath}`,
    );
  }
}

if (failures.length) {
  console.error("KGW project continuity gate FAILED");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("KGW project continuity gate PASSED");
console.log(
  "Canonical continuity files, active/current state, durable handoff and project memory, dynamic Git-state semantics, regression/security lifecycles, plan/ADR lifecycle, and release runbook are present.",
);
