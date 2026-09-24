import assert from "node:assert/strict";
import fs from "node:fs";
import { isStoppedOwnerStatus } from "./assertions.mjs";

const wdio = fs.readFileSync(new URL("../wdio.conf.mjs", import.meta.url), "utf8");
assert.ok(wdio.includes('process.env.KGW_E2E_SPEC || "./specs/zero-touch-live-matrix.e2e.js"'));
assert.ok(wdio.includes("specs: [selectedSpec]"), "WDIO must use the selected local spec");


assert.equal(
  isStoppedOwnerStatus("role=node;network=mainnet;pid=4242;running=false;readiness=FAILED"),
  true,
  "terminal status must remain stopped even when PID is retained as crash evidence",
);
assert.equal(
  isStoppedOwnerStatus("role=node;network=mainnet;pid=4242;running=true;readiness=READY"),
  false,
  "running owner status must never be classified as stopped",
);

const windowsHelpers = fs.readFileSync(new URL("./windows.mjs", import.meta.url), "utf8");
const ownedProcessRust = fs.readFileSync(new URL("../../xtask/src/e2e_owned_process.rs", import.meta.url), "utf8");

const killCallStart = windowsHelpers.indexOf("export async function killExactOwnedProcess");
const waitCallStart = windowsHelpers.indexOf("export async function waitForExactProcessExit");
const evidenceCallStart = windowsHelpers.indexOf("export async function captureWindowsEvidence");
assert.ok(killCallStart >= 0 && waitCallStart > killCallStart && evidenceCallStart > waitCallStart);
const killCall = windowsHelpers.slice(killCallStart, waitCallStart);
const waitCall = windowsHelpers.slice(waitCallStart, evidenceCallStart);
assert.ok(killCall.includes('"e2e-owned-process"') && killCall.includes('"kill"'));
assert.ok(waitCall.includes('"e2e-owned-process"') && waitCall.includes('"wait"'));
assert.equal(killCall.includes(".ps1"), false, "exact-owner kill helper must be Rust-owned");
assert.equal(waitCall.includes(".ps1"), false, "exact-owner wait helper must be Rust-owned");

const killRustStart = ownedProcessRust.indexOf("fn kill_exact_owned_process");
const waitRustStart = ownedProcessRust.indexOf("fn wait_exact_process_exit");
assert.ok(killRustStart >= 0 && waitRustStart > killRustStart);
const killRust = ownedProcessRust.slice(killRustStart, waitRustStart);
const executableGuard = killRust.indexOf("owned process executable mismatch");
const startTimeGuard = killRust.indexOf("owned process start-time mismatch");
const forceKill = killRust.indexOf("TerminateProcess");
assert.ok(executableGuard >= 0 && startTimeGuard >= 0 && forceKill >= 0);
assert.ok(executableGuard < forceKill, "executable identity must be checked before force kill");
assert.ok(startTimeGuard < forceKill, "start-time identity must be checked before force kill");

const testModuleStart = ownedProcessRust.indexOf("#[cfg(test)]", waitRustStart);
assert.ok(waitRustStart >= 0 && testModuleStart > waitRustStart);
const waitRust = ownedProcessRust.slice(waitRustStart, testModuleStart);
assert.ok(waitRust.includes("exact_identity_exited"), "close/relaunch evidence must track exact identity exit");
assert.equal(waitRust.includes("TerminateProcess"), false, "close/relaunch wait helper must never kill a process");

const relaunch = fs.readFileSync(new URL("./app-close-relaunch.mjs", import.meta.url), "utf8");
const closeRequest = relaunch.indexOf("firstBrowser.closeWindow()");
const exactExit = relaunch.indexOf("waitForExactProcessExit({");
const newSession = relaunch.indexOf('secondBrowser = await newSession("after-relaunch"');
assert.ok(closeRequest >= 0 && closeRequest < exactExit && exactExit < newSession);
console.log("recovery harness smoke: PASS");
