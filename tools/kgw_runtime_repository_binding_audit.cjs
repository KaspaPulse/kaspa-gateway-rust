#!/usr/bin/env node
"use strict";

/**
 * KGW_RUNTIME_REPOSITORY_BINDING_AUDIT_WRAPPER_R21C
 *
 * Compatibility wrapper.
 * Canonical implementation lives in:
 *   tools/kgw_runtime_repository_binding_gate.cjs
 */

const cp = require("child_process");

const extraArgs = process.argv.slice(2);
const args = ["tools/kgw_runtime_repository_binding_gate.cjs", "--strict", "--online", ...extraArgs];

const result = cp.spawnSync(process.execPath, args, {
  cwd: process.cwd(),
  stdio: "inherit",
  windowsHide: true
});

process.exit(result.status == null ? 1 : result.status);
