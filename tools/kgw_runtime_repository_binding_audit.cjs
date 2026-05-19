#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const repo = process.cwd();

const EXPECTED = {
  kaspanetRepo: "https://github.com/kaspanet/rusty-kaspa.git",
  rkRepo: "https://github.com/LiveLaughLove13/rusty-kaspa.git",
  rkBranch: "RKStratumTN12"
};

const files = {
  nodeCargo: path.join(repo, "crates", "kaspa-gateway-rk-node", "Cargo.toml"),
  bridgeCargo: path.join(repo, "crates", "kaspa-gateway-rk-bridge", "Cargo.toml"),
  serviceController: path.join(repo, "crates", "kaspa-gateway-rk-node", "src", "kgw_service_controller.rs"),
  officialRuntime: path.join(repo, "crates", "kaspa-gateway-rk-node", "src", "official_kaspa_runtime.rs"),
  bridgeRuntime: path.join(repo, "crates", "kaspa-gateway-rk-bridge", "src", "lib.rs")
};

function fail(message) {
  console.error("KGW runtime repository binding audit FAILED");
  console.error(message);
  process.exit(1);
}

function read(file) {
  if (!fs.existsSync(file)) fail("Missing file: " + file);
  return fs.readFileSync(file, "utf8");
}

function parseCargoAlias(source, alias) {
  const line = source
    .split(/\r?\n/)
    .find((item) => item.trim().startsWith(alias + " = {"));

  if (!line) return null;

  const bodyStart = line.indexOf("{");
  const bodyEnd = line.lastIndexOf("}");
  const body = bodyStart >= 0 && bodyEnd > bodyStart
    ? line.slice(bodyStart + 1, bodyEnd)
    : line;

  function field(name) {
    const token = name + " = \"";
    const start = body.indexOf(token);
    if (start < 0) return "";
    const valueStart = start + token.length;
    const valueEnd = body.indexOf("\"", valueStart);
    return valueEnd < 0 ? "" : body.slice(valueStart, valueEnd);
  }

  return {
    alias,
    package: field("package"),
    git: field("git"),
    branch: field("branch"),
    optional: body.includes("optional = true")
  };
}

function unique(values) {
  return Array.from(new Set(values.filter(Boolean)));
}

function assertAliases(source, aliases, expectedRepo, expectedBranch, label) {
  const entries = aliases.map((alias) => parseCargoAlias(source, alias));

  const missing = entries.filter((entry) => !entry);
  if (missing.length) fail("Missing aliases for " + label);

  const repos = unique(entries.map((entry) => entry.git));
  const branches = unique(entries.map((entry) => entry.branch));

  if (repos.length !== 1 || repos[0] !== expectedRepo) {
    fail(label + " expected repo " + expectedRepo + " but found " + repos.join(", "));
  }

  if (branches.length !== 1 || branches[0] !== expectedBranch) {
    fail(label + " expected branch " + expectedBranch + " but found " + branches.join(", "));
  }

  return {
    repo: repos[0],
    branch: branches[0],
    packages: entries.map((entry) => entry.package),
    aliases: entries.map((entry) => entry.alias)
  };
}

const nodeCargo = read(files.nodeCargo);
const bridgeCargo = read(files.bridgeCargo);
const serviceController = read(files.serviceController);
const officialRuntime = read(files.officialRuntime);
const bridgeRuntime = read(files.bridgeRuntime);

const mainlineNode = assertAliases(
  nodeCargo,
  ["kaspad-lib-mainline", "kaspa-core-mainline", "kaspa-utils-mainline"],
  EXPECTED.kaspanetRepo,
  "master",
  "node/mainline"
);

const testnetNode = assertAliases(
  nodeCargo,
  ["kaspad-lib-tn12", "kaspa-core-tn12", "kaspa-utils-tn12"],
  EXPECTED.rkRepo,
  EXPECTED.rkBranch,
  "node/testnets"
);

const mainlineBridge = assertAliases(
  bridgeCargo,
  ["kaspa-stratum-bridge-mainline"],
  EXPECTED.kaspanetRepo,
  "master",
  "bridge/mainline"
);

const testnetBridge = assertAliases(
  bridgeCargo,
  ["kaspa-stratum-bridge-tn12"],
  EXPECTED.rkRepo,
  EXPECTED.rkBranch,
  "bridge/testnets"
);

const mappingChecks = [
  ["service_controller mainnet -> master", serviceController.includes('Self::Mainnet => "master"')],
  ["service_controller testnet10/testnet12 -> RKStratumTN12", serviceController.includes('Self::Testnet10 | Self::Testnet12 => "RKStratumTN12"')],
  ["official_runtime mainnet -> master", officialRuntime.includes('Self::Mainnet => "master"')],
  ["official_runtime testnet10/testnet12 -> RKStratumTN12", officialRuntime.includes('Self::Testnet10 | Self::Testnet12 => "RKStratumTN12"')],
  ["official_runtime mainnet -> Mainline family", officialRuntime.includes("Self::Mainnet => KaspaRuntimeFamily::Mainline")],
  ["official_runtime testnet10/testnet12 -> Tn12 family", officialRuntime.includes("Self::Testnet10 | Self::Testnet12 => KaspaRuntimeFamily::Tn12")],
  ["bridge_runtime mainnet -> master", bridgeRuntime.includes('Self::Mainnet => "master"')],
  ["bridge_runtime testnet10/testnet12 -> RKStratumTN12", bridgeRuntime.includes('Self::Testnet10 | Self::Testnet12 => "RKStratumTN12"')],
  ["bridge_runtime mainnet -> Mainline family", bridgeRuntime.includes("Self::Mainnet => BridgeRuntimeFamily::Mainline")],
  ["bridge_runtime testnet10/testnet12 -> Tn12 family", bridgeRuntime.includes("Self::Testnet10 | Self::Testnet12 => BridgeRuntimeFamily::Tn12")]
];

const failedMapping = mappingChecks.filter(([, ok]) => !ok);

if (failedMapping.length) {
  fail("Network mapping mismatch:\n" + failedMapping.map(([name]) => "- " + name).join("\n"));
}

const networkBindings = [
  {
    network: "mainnet",
    family: "mainline",
    branch: "master",
    nodeRepo: mainlineNode.repo,
    bridgeRepo: mainlineBridge.repo,
    nodePackages: mainlineNode.packages,
    bridgePackages: mainlineBridge.packages,
    feature: "official-kaspa-runtime-mainline"
  },
  {
    network: "testnet10",
    family: "tn12",
    branch: EXPECTED.rkBranch,
    nodeRepo: testnetNode.repo,
    bridgeRepo: testnetBridge.repo,
    nodePackages: testnetNode.packages,
    bridgePackages: testnetBridge.packages,
    feature: "official-kaspa-runtime-tn12"
  },
  {
    network: "testnet12",
    family: "tn12",
    branch: EXPECTED.rkBranch,
    nodeRepo: testnetNode.repo,
    bridgeRepo: testnetBridge.repo,
    nodePackages: testnetNode.packages,
    bridgePackages: testnetBridge.packages,
    feature: "official-kaspa-runtime-tn12"
  }
];

console.log("KGW runtime repository binding audit");

for (const item of networkBindings) {
  console.log([
    "network=" + item.network,
    "family=" + item.family,
    "branch=" + item.branch,
    "node_repo=" + item.nodeRepo,
    "bridge_repo=" + item.bridgeRepo,
    "feature=" + item.feature,
    "node_packages=" + item.nodePackages.join(","),
    "bridge_packages=" + item.bridgePackages.join(",")
  ].join(";"));
}

console.log("status=PASS");
