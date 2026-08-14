#!/usr/bin/env node

const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");

const workflowPath = ".github/workflows/desktop-artifacts.yml";
const windowsConfigPath =
  "apps/kaspa-gateway-desktop/src-tauri/tauri.windows.conf.json";
const macosConfigPath =
  "apps/kaspa-gateway-desktop/src-tauri/tauri.macos.conf.json";

const workflow = readFileSync(workflowPath, "utf8");
const windowsConfig = JSON.parse(readFileSync(windowsConfigPath, "utf8"));
const macosConfig = JSON.parse(readFileSync(macosConfigPath, "utf8"));

assert.deepEqual(windowsConfig.bundle.targets, ["nsis"]);
assert.deepEqual(windowsConfig.bundle.icon, ["icons/icon.ico"]);
assert.deepEqual(macosConfig.bundle.targets, ["dmg"]);
assert.deepEqual(macosConfig.bundle.icon, ["icons/icon.png"]);
assert.equal(macosConfig.bundle.macOS.signingIdentity, "-");
assert.equal(macosConfig.bundle.macOS.hardenedRuntime, false);

const requiredWorkflowFragments = [
  "workflow_dispatch:",
  "commit_sha:",
  "permissions:\n  contents: read",
  "runs-on: windows-2022",
  "runs-on: macos-15-intel",
  "^[0-9a-f]{40}$",
  "npm run tauri:build -- --ci --target x86_64-pc-windows-msvc",
  "npm run tauri:build -- --ci --target universal-apple-darwin",
  "x86_64-pc-windows-msvc",
  "aarch64-apple-darwin,x86_64-apple-darwin",
  "protoc-35.1-win64.zip",
  "5d3ff218d7d91eea95f7569bcb5a98f3030f8996d44151279d9772edcff76082",
  "protoc-35.1-osx-universal_binary.zip",
  "9c27aebb44c537f5627cc13c9c1c6bc0e34ecfefc6e4d79b19764afb8302d95b",
  "WINDOWS_CODE_SIGNING=UNSIGNED_NOT_CONFIGURED",
  "MACOS_CODE_SIGNING=AD_HOC",
  "MACOS_NOTARIZATION=NOT_CONFIGURED",
  "MACOS_ARCHITECTURE_PROOF=UNIVERSAL_ARM64_X86_64",
  "Get-AuthenticodeSignature",
  'ArgumentList @("/S"',
  "KASPA_GATEWAY_DATA_DIR",
  "hdiutil attach",
  "lipo -archs",
  "codesign --verify --deep --strict",
  "ditto -c -k --sequesterRsrc --keepParent",
  "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
];

for (const fragment of requiredWorkflowFragments) {
  assert.ok(
    workflow.includes(fragment),
    `desktop artifact workflow must contain: ${fragment}`,
  );
}

assert.ok(
  !workflow.includes("${{ secrets."),
  "unsigned internal artifact workflow must not consume signing secrets",
);
assert.ok(
  !workflow.includes("arduino/setup-protoc"),
  "workflow must use checksum-verified official protoc archives, not the incompatible GPL action",
);
assert.ok(
  !/\b(?:release|softprops)\/[^\s@]+@/u.test(workflow),
  "artifact workflow must not use a GitHub Release action",
);

const actionUses = [...workflow.matchAll(/^\s*uses:\s*([^\s#]+).*$/gmu)].map(
  (match) => match[1],
);
assert.ok(
  actionUses.length >= 8,
  "expected immutable actions in both native jobs",
);
for (const actionUse of actionUses) {
  assert.match(
    actionUse,
    /^[^@]+@[0-9a-f]{40}$/u,
    `action must be pinned to an immutable full SHA: ${actionUse}`,
  );
}

console.log("DESKTOP ARTIFACT WORKFLOW CONTRACT PASSED");
