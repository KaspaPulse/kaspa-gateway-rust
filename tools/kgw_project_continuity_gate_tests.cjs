#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const fixturePaths = [
  "AGENTS.md",
  "PROJECT_STATE.md",
  "ACTIVE_TASK.md",
  "CURRENT_STATE.md",
  "PLANS.md",
  "docs/continuity",
  "docs/handoff-ledger",
  "docs/project-memory",
  "docs/adr/README.md",
  "docs/adr/0011-repository-native-project-continuity.md",
  "docs/runbooks/desktop-release.md",
  "docs/architecture/README.md",
  "tools/kgw_project_continuity_gate.cjs",
  ".github/workflows",
];

function copyFixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "kgw-continuity-gate-"));
  for (const relativePath of fixturePaths) {
    const source = path.join(repoRoot, relativePath);
    const target = path.join(root, relativePath);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.cpSync(source, target, { recursive: true });
  }
  return root;
}

function runGate(root) {
  return spawnSync(process.execPath, [path.join(root, "tools/kgw_project_continuity_gate.cjs")], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env },
  });
}

function output(result) {
  return `${result.stdout || ""}${result.stderr || ""}`;
}

function expectPass(label, mutate = () => {}) {
  const root = copyFixture();
  try {
    mutate(root);
    const result = runGate(root);
    if (result.status !== 0) {
      throw new Error(`${label}: expected PASS\n${output(result)}`);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function expectFail(label, expectedText, mutate) {
  const root = copyFixture();
  try {
    mutate(root);
    const result = runGate(root);
    const combined = output(result);
    if (result.status === 0 || !combined.includes(expectedText)) {
      throw new Error(
        `${label}: expected fail containing ${JSON.stringify(expectedText)}\n${combined}`,
      );
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

expectPass("baseline continuity fixture");

expectPass("active plan may mention future inactive sentinel in prose", (root) => {
  const file = path.join(root, "PLANS.md");
  fs.appendFileSync(file, "\nFuture closure returns this file to NO ACTIVE MULTI-STAGE PLAN.\n");
});

expectFail(
  "missing active task",
  "Missing required continuity file: ACTIVE_TASK.md",
  (root) => fs.rmSync(path.join(root, "ACTIVE_TASK.md")),
);

expectFail(
  "missing durable checkpoint",
  "No durable task checkpoint exists under docs/handoff-ledger/.",
  (root) => {
    const dir = path.join(root, "docs/handoff-ledger");
    for (const name of fs.readdirSync(dir)) {
      if (name.endsWith(".md") && !["README.md", "TEMPLATE.md"].includes(name)) {
        fs.rmSync(path.join(dir, name));
      }
    }
  },
);

expectFail(
  "invalid security-memory status",
  "lifecycle status",
  (root) => {
    const dir = path.join(root, "docs/project-memory/SECURITY");
    const name = fs
      .readdirSync(dir)
      .find((entry) => entry.endsWith(".md") && entry !== "README.md");
    if (!name) throw new Error("security fixture record missing");
    const file = path.join(dir, name);
    const text = fs.readFileSync(file, "utf8");
    fs.writeFileSync(file, text.replace(/- Status: VERIFIED/, "- Status: UNKNOWN"));
  },
);

expectFail(
  "duplicate project-memory stable ID",
  "Duplicate project-memory stable ID",
  (root) => {
    const dir = path.join(root, "docs/project-memory/SECURITY");
    const name = fs
      .readdirSync(dir)
      .find((entry) => entry.endsWith(".md") && entry !== "README.md");
    if (!name) throw new Error("security fixture record missing");
    fs.copyFileSync(path.join(dir, name), path.join(dir, `DUPLICATE-${name}`));
  },
);

expectFail(
  "retired release admin secret workflow reference",
  "Retired GitHub Actions secret RELEASE_ADMIN_TOKEN must not be referenced by workflow",
  (root) => {
    const file = path.join(root, ".github/workflows/ci.yml");
    fs.appendFileSync(
      file,
      "\n# negative fixture only\n# RELEASE_ADMIN_TOKEN must remain retired\n",
    );
  },
);

expectFail(
  "missing regression-memory category",
  "Missing required continuity file: docs/project-memory/REGRESSIONS/README.md",
  (root) => fs.rmSync(path.join(root, "docs/project-memory/REGRESSIONS/README.md")),
);

expectFail(
  "missing root-cause lifecycle",
  "important-bug lifecycle",
  (root) => {
    const file = path.join(root, "docs/continuity/PROJECT_CONTINUITY_POLICY.md");
    const text = fs.readFileSync(file, "utf8");
    fs.writeFileSync(
      file,
      text.replace(
        "Bug -> Reproduce -> Root Cause -> Fix -> Verification -> Regression Protection -> Documentation",
        "Bug -> Patch -> Done",
      ),
    );
  },
);

console.log("KGW project continuity gate regression tests PASSED");
console.log("Positive fixture and seven fail-closed negative cases behaved as required.");
