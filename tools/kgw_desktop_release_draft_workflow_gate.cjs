#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");

const workflowPath = ".github/workflows/desktop-release-draft.yml";
const ciPath = ".github/workflows/ci.yml";

const workflow = readFileSync(workflowPath, "utf8");
const ci = readFileSync(ciPath, "utf8");

assert.ok(
  workflow.includes("gh api --paginate --slurp"),
  "draft workflow must use a draft-inclusive paginated release listing",
);
assert.ok(
  workflow.includes('repos/$GITHUB_REPOSITORY/releases?per_page=100'),
  "draft workflow must query List releases when detecting or resolving drafts",
);
assert.ok(
  workflow.includes("id: create-draft"),
  "draft creation step must expose the created release ID",
);
assert.ok(
  workflow.includes("GITHUB_OUTPUT"),
  "draft creation must publish release_id through GITHUB_OUTPUT",
);
assert.ok(
  workflow.includes(
    "DRAFT_RELEASE_ID: ${{ steps.create-draft.outputs.release_id }}",
  ),
  "draft verification must consume the created release ID",
);
assert.ok(
  workflow.includes(
    'gh api "repos/$GITHUB_REPOSITORY/releases/$DRAFT_RELEASE_ID"',
  ),
  "draft verification must read the release object by release ID",
);
assert.ok(
  workflow.includes('.target_commitish "$draft_json"'),
  "draft verification must bind target_commitish to the requested commit",
);
assert.ok(
  !workflow.includes('gh release view "$tag"'),
  "draft preflight must not rely on tag-only release lookup",
);
assert.ok(
  !workflow.includes('repos/$GITHUB_REPOSITORY/releases/tags/$tag'),
  "draft verification must not use the published-release-by-tag endpoint",
);

const tagRefOccurrences = (
  workflow.match(/git\/ref\/tags\/\$tag/gu) ?? []
).length;
assert.equal(
  tagRefOccurrences,
  1,
  "git tag-ref lookup is allowed only in preflight, never in post-create draft verification",
);

assert.match(
  workflow,
  /id: create-draft[\s\S]*release_id=%s[\s\S]*DRAFT_RELEASE_ID: \$\{\{ steps\.create-draft\.outputs\.release_id \}\}[\s\S]*releases\/\$DRAFT_RELEASE_ID/u,
  "draft release ID must flow from creation to ID-based verification",
);
assert.ok(
  ci.includes("Verify desktop release draft workflow contract"),
  "blocking CI must run the desktop release draft workflow contract",
);
assert.ok(
  ci.includes("node tools/kgw_desktop_release_draft_workflow_gate.cjs"),
  "blocking CI must execute the draft workflow contract gate",
);

console.log("DESKTOP RELEASE DRAFT WORKFLOW CONTRACT PASSED");
