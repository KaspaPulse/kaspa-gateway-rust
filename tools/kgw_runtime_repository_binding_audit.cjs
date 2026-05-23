#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const repo = process.cwd();

function fail(message) {
  console.error("KGW runtime repository binding audit FAILED");
  console.error(message);
  process.exit(1);
}

function read(file) {
  if (!fs.existsSync(file)) fail("Missing file: " + file);
  return fs.readFileSync(file, "utf8");
}

function requireContains(text, needle, label) {
  if (!text.includes(needle)) fail(label + " missing expected text: " + needle);
}

function parseCargoAlias(source, alias) {
  const line = source.split(/\r?\n/).find((item) => item.trim().startsWith(alias + " = {"));
  if (!line) return null;

  const field = (name) => {
    const re = new RegExp(name + '\\s*=\\s*"([^"]+)"');
    const m = re.exec(line);
    return m ? m[1] : "";
  };

  return { alias, git: field("git"), branch: field("branch"), rev: field("rev") };
}

function assertAlias(source, alias, repoUrl, rev, label) {
  const item = parseCargoAlias(source, alias);
  if (!item) fail(label + " missing alias " + alias);
  if (item.git !== repoUrl) fail(label + " expected repo " + repoUrl + " but got " + item.git);
  if (item.rev !== rev) fail(label + " expected rev " + rev + " but got " + item.rev);
}

const files = {
  manifest: path.join(repo, "config", "runtime-repository-bindings.json"),
  nodeCargo: path.join(repo, "crates", "kaspa-gateway-rk-node", "Cargo.toml"),
  bridgeCargo: path.join(repo, "crates", "kaspa-gateway-rk-bridge", "Cargo.toml"),
  serviceController: path.join(repo, "crates", "kaspa-gateway-rk-node", "src", "kgw_service_controller.rs"),
  officialRuntime: path.join(repo, "crates", "kaspa-gateway-rk-node", "src", "official_kaspa_runtime.rs"),
  bridgeRuntime: path.join(repo, "crates", "kaspa-gateway-rk-bridge", "src", "lib.rs")
};

const manifest = JSON.parse(read(files.manifest));
const mainnet = manifest.networks.mainnet;
const testnet10 = manifest.networks.testnet10;
const testnet12 = manifest.networks.testnet12;

if (!/^[0-9a-f]{40}$/i.test(mainnet.rev || "")) fail("mainnet rev missing or invalid");
if (!/^[0-9a-f]{40}$/i.test(testnet10.rev || "")) fail("testnet10 rev missing or invalid");
if (!/^[0-9a-f]{40}$/i.test(testnet12.rev || "")) fail("testnet12 rev missing or invalid");
if (testnet10.rev !== testnet12.rev) fail("testnet10/testnet12 rev mismatch");

const nodeCargo = read(files.nodeCargo);
const bridgeCargo = read(files.bridgeCargo);
const service = read(files.serviceController);
const official = read(files.officialRuntime);
const bridge = read(files.bridgeRuntime);

for (const alias of ["kaspad-lib-mainline", "kaspa-core-mainline", "kaspa-utils-mainline"]) {
  assertAlias(nodeCargo, alias, mainnet.repo, mainnet.rev, "node mainline");
}
for (const alias of ["kaspad-lib-tn12", "kaspa-core-tn12", "kaspa-utils-tn12"]) {
  assertAlias(nodeCargo, alias, testnet10.repo, testnet10.rev, "node tn12");
}
assertAlias(bridgeCargo, "kaspa-stratum-bridge-mainline", mainnet.repo, mainnet.rev, "bridge mainline");
assertAlias(bridgeCargo, "kaspa-stratum-bridge-tn12", testnet10.repo, testnet10.rev, "bridge tn12");

requireContains(service, `Self::Mainnet => "${mainnet.branch}"`, "service mainnet branch");
requireContains(service, `Self::Testnet10 | Self::Testnet12 => "${testnet10.branch}"`, "service testnet branch");
requireContains(service, `Self::Mainnet => "${mainnet.rev}"`, "service mainnet revision");
requireContains(service, `Self::Testnet10 | Self::Testnet12 => "${testnet10.rev}"`, "service testnet revision");

requireContains(official, `Self::Mainnet => "${mainnet.branch}"`, "official mainnet branch");
requireContains(official, `Self::Testnet10 | Self::Testnet12 => "${testnet10.branch}"`, "official testnet branch");
requireContains(official, `Self::Mainnet => "${mainnet.rev}"`, "official mainnet revision");
requireContains(official, `Self::Testnet10 | Self::Testnet12 => "${testnet10.rev}"`, "official testnet revision");
requireContains(official, "Self::Mainnet => KaspaRuntimeFamily::Mainline", "official mainnet family");
requireContains(official, "Self::Testnet10 | Self::Testnet12 => KaspaRuntimeFamily::Tn12", "official tn12 family");

requireContains(bridge, `Self::Mainnet => "${mainnet.branch}"`, "bridge mainnet branch");
requireContains(bridge, `Self::Testnet10 | Self::Testnet12 => "${testnet10.branch}"`, "bridge testnet branch");
requireContains(bridge, `Self::Mainnet => "${mainnet.rev}"`, "bridge mainnet revision");
requireContains(bridge, `Self::Testnet10 | Self::Testnet12 => "${testnet10.rev}"`, "bridge testnet revision");
requireContains(bridge, "Self::Mainnet => BridgeRuntimeFamily::Mainline", "bridge mainnet family");
requireContains(bridge, "Self::Testnet10 | Self::Testnet12 => BridgeRuntimeFamily::Tn12", "bridge tn12 family");

console.log("KGW runtime repository binding audit");
for (const [network, item] of Object.entries(manifest.networks)) {
  console.log([
    "network=" + network,
    "family=" + (item.family || ""),
    "branch=" + (item.branch || ""),
    "rev=" + (item.rev || ""),
    "node_repo=" + (item.repo || ""),
    "bridge_repo=" + (item.repo || ""),
    "feature=" + (item.feature || "")
  ].join(";"));
}
console.log("status=PASS");
