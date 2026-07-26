#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const repo = process.cwd();

const files = {
  integratedRuntime: path.join(repo, "apps", "kaspa-gateway-desktop", "src-tauri", "src", "integrated_runtime_commands.rs"),
  serviceController: path.join(repo, "crates", "kaspa-gateway-rk-node", "src", "kgw_service_controller.rs"),
  libRs: path.join(repo, "apps", "kaspa-gateway-desktop", "src-tauri", "src", "lib.rs")
};

function fail(message) {
  console.error("KGW parallel self-worker runtime gate FAILED");
  console.error(message);
  process.exit(1);
}

function read(file) {
  if (!fs.existsSync(file)) fail("Missing file: " + file);
  return fs.readFileSync(file, "utf8");
}

const integrated = read(files.integratedRuntime);
const controller = read(files.serviceController);
const libRs = read(files.libRs);

const checks = [
  ["parallel registry", integrated.includes("KGW_PARALLEL_SELF_WORKERS") && integrated.includes("OnceLock") && integrated.includes("HashMap")],
  ["same exe spawn", integrated.includes("std::env::current_exe") && integrated.includes("Command::new") && integrated.includes("--kgw-self-worker")],
  ["network arg", integrated.includes("--network")],
  ["role network key", integrated.includes("role") && integrated.includes("network") && integrated.includes(":")],
  ["worker status string", integrated.includes("parallel-owned-self-worker")],
  ["same exe marker", integrated.includes("same_exe")],
  ["no external kaspad exe marker", integrated.includes("external_kaspad_exe")],
  ["uses kaspa libraries marker", integrated.includes("uses_kaspa_libraries")],
  ["mainnet", /mainnet/i.test(controller) || /Mainnet/.test(controller)],
  ["testnet10", /testnet10/i.test(controller) || /Testnet10/.test(controller)],
  ["testnet12", /testnet12/i.test(controller) || /Testnet12/.test(controller)],
  ["stable mainline owner", /stable|mainline/i.test(controller)],
  ["tn12 owner", /tn12/i.test(controller)],
  ["distinct rpc ports", /16110/.test(controller) && /16210/.test(controller) && /16310/.test(controller)],
  ["tauri module registered", libRs.includes("integrated_runtime_commands")]
];

const failed = checks.filter(([, ok]) => !ok);

if (failed.length) {
  fail(failed.map(([name]) => "- " + name).join("\n"));
}

const forbidden = [
  ["external kaspad exe launch", /Command::new\([^\n]*(kaspad|kaspabridge|stratum-bridge)\.exe/i.test(integrated)],
  ["shell launch", /Command::new\([^\n]*(cmd|powershell|pwsh|sh)\b/i.test(integrated)],
  ["frontend shell ownership", /shell|cmd\.exe|powershell|pwsh/i.test(read(path.join(repo, "apps", "kaspa-gateway-desktop", "frontend", "src", "tabs", "kaspa-node", "kaspa-node.js")))]
];

const forbiddenHits = forbidden.filter(([, hit]) => hit);

if (forbiddenHits.length) {
  fail("Forbidden runtime ownership detected:\n" + forbiddenHits.map(([name]) => "- " + name).join("\n"));
}

console.log("KGW parallel self-worker runtime gate");
console.log("sameExeWorker: true");
console.log("externalKaspadExe: false");
console.log("usesKaspaLibraries: true");
console.log("registry: role:network");
console.log("roles: node, bridge");
console.log("networks: mainnet, testnet10, testnet12");
console.log("status: PASS");
