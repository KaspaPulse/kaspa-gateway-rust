import assert from "node:assert/strict";
import fs from "node:fs";

const wdio = fs.readFileSync(new URL("../wdio.conf.mjs", import.meta.url), "utf8");
assert.ok(wdio.includes('process.env.KGW_E2E_SPEC || "./specs/zero-touch-live-matrix.e2e.js"'));
assert.ok(wdio.includes("specs: [selectedSpec]"), "WDIO must use the selected local spec");

const killScript = fs.readFileSync(new URL("./kgw_kill_exact_owned_process.ps1", import.meta.url), "utf8");
const executableGuard = killScript.indexOf("owned process executable mismatch");
const startTimeGuard = killScript.indexOf("owned process start-time mismatch");
const forceKill = killScript.indexOf("Stop-Process -Id $ProcessId -Force");
assert.ok(executableGuard >= 0 && startTimeGuard >= 0 && forceKill >= 0);
assert.ok(executableGuard < forceKill, "executable identity must be checked before force kill");
assert.ok(startTimeGuard < forceKill, "start-time identity must be checked before force kill");

const waitScript = fs.readFileSync(new URL("./kgw_wait_exact_process_exit.ps1", import.meta.url), "utf8");
assert.ok(waitScript.includes("exact_identity_exited"), "close/relaunch evidence must track exact identity exit");
assert.equal(waitScript.includes("Stop-Process"), false, "close/relaunch wait helper must never kill a process");

const relaunch = fs.readFileSync(new URL("./app-close-relaunch.mjs", import.meta.url), "utf8");
const closeRequest = relaunch.indexOf("firstBrowser.closeWindow()");
const exactExit = relaunch.indexOf("waitForExactProcessExit({");
const newSession = relaunch.indexOf('secondBrowser = await newSession("after-relaunch"');
assert.ok(closeRequest >= 0 && closeRequest < exactExit && exactExit < newSession);
console.log("recovery harness smoke: PASS");
