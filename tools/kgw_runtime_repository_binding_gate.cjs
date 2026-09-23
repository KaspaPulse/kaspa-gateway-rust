#!/usr/bin/env node
"use strict";

/**
 * KGW_RUNTIME_REPOSITORY_BINDING_UNIFIED_GATE_R21C
 *
 * Read-only unified gate for KGW runtime repository bindings.
 * It validates manifest, Cargo aliases, Rust mappings, Cargo.lock, and optional remote freshness.
 *
 * This gate MUST NOT mutate source files.
 * This gate MUST NOT call the apply script.
 */

const fs = require("fs");
const path = require("path");
const cp = require("child_process");

const args = new Set(process.argv.slice(2));
const jsonMode = args.has("--json");
const strictMode = args.has("--strict");
const onlineMode = args.has("--online") || args.has("--fresh");
const offlineMode = args.has("--offline");
const repoRoot = process.cwd();

function read(rel) {
  return fs.readFileSync(path.join(repoRoot, rel), "utf8");
}

function exists(rel) {
  return fs.existsSync(path.join(repoRoot, rel));
}

function run(command, argv) {
  const result = cp.spawnSync(command, argv, {
    cwd: repoRoot,
    encoding: "utf8",
    windowsHide: true
  });

  return {
    command: [command, ...argv].join(" "),
    status: result.status,
    stdout: result.stdout || "",
    stderr: result.stderr || ""
  };
}

function normalizeRepo(url) {
  return String(url || "").replace(/\.git$/, "").toLowerCase();
}

function addFinding(findings, level, issue, extra = {}) {
  findings.push({ level, issue, ...extra });
}

function parseCargoAlias(source, alias) {
  const line = source.split(/\r?\n/).find((item) => item.trim().startsWith(alias + " = {"));
  if (!line) return null;

  const field = (name) => {
    const re = new RegExp(name + "\\s*=\\s*\"([^\"]+)\"");
    const m = re.exec(line);
    return m ? m[1] : "";
  };

  return {
    alias,
    line,
    git: field("git"),
    branch: field("branch"),
    rev: field("rev"),
    package: field("package"),
    optional: /optional\s*=\s*true/.test(line)
  };
}

function parseLockGitSources(lockText) {
  const hits = [];
  const re = /source\s*=\s*"git\+([^"?]+)\?[^"]*?(?:branch=([^#"]+))?[^"]*#([0-9a-f]{40})"/g;
  let m;
  while ((m = re.exec(lockText)) !== null) {
    const block = lockText.slice(lockText.lastIndexOf("[[package]]", m.index), m.index);
    hits.push({
      name: /^name\s*=\s*"([^"]+)"/m.exec(block)?.[1] || "",
      url: m[1],
      branch: m[2] || "",
      rev: m[3],
      raw: m[0]
    });
  }
  return hits;
}

function lsRemote(repoUrl, branch) {
  const result = run("git", ["ls-remote", repoUrl, "refs/heads/" + branch]);
  const stdout = result.stdout.trim();
  const match = stdout.match(/^([0-9a-f]{40})\s+/i);

  return {
    repo: repoUrl,
    branch,
    ok: result.status === 0 && Boolean(match),
    latestRev: match ? match[1] : "",
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout
  };
}

const files = {
  manifest: "config/runtime-repository-bindings.json",
  nodeCargo: "crates/kaspa-gateway-rk-node/Cargo.toml",
  bridgeCargo: "crates/kaspa-gateway-rk-bridge/Cargo.toml",
  cliCargo: "apps/kaspa-gateway-cli/Cargo.toml",
  coreCargo: "crates/kaspa-gateway-core/Cargo.toml",
  serviceController: "crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs",
  officialRuntime: "crates/kaspa-gateway-rk-node/src/official_kaspa_runtime.rs",
  bridgeRuntime: "crates/kaspa-gateway-rk-bridge/src/lib.rs",
  cargoLock: "Cargo.lock",
  applyScript: "tools/kgw_runtime_repository_binding_apply.ps1"
};

const findings = [];
const requiredFiles = [
  files.manifest,
  files.nodeCargo,
  files.bridgeCargo,
  files.cliCargo,
  files.coreCargo,
  files.serviceController,
  files.officialRuntime,
  files.bridgeRuntime,
  files.applyScript
];

for (const rel of requiredFiles) {
  if (!exists(rel)) addFinding(findings, "error", "missing-required-file", { file: rel });
}

let manifest = {};
let networks = {};

if (!findings.some((x) => x.level === "error")) {
  manifest = JSON.parse(read(files.manifest));
  networks = manifest.networks || {};
}

const expectedNetworks = ["mainnet", "testnet10", "testnet13"];

for (const net of expectedNetworks) {
  const b = networks[net];

  if (!b) {
    addFinding(findings, "error", "missing-network-binding", { net });
    continue;
  }

  if (!b.repo) addFinding(findings, "error", "missing-repo", { net });
  if (!b.branch) addFinding(findings, "error", "missing-branch", { net });
  if (!b.rev || !/^[0-9a-f]{40}$/i.test(b.rev)) addFinding(findings, "error", "missing-or-invalid-rev", { net, rev: b.rev || "" });
  if (!b.family) addFinding(findings, "error", "missing-family", { net });
}

if (networks.mainnet && networks.mainnet.repo !== "https://github.com/kaspanet/rusty-kaspa.git") {
  addFinding(findings, "error", "mainnet-must-use-official-kaspanet-repo", {
    expected: "https://github.com/kaspanet/rusty-kaspa.git",
    actual: networks.mainnet.repo
  });
}

for (const net of ["mainnet", "testnet10"]) {
  if (networks[net] && networks[net].repo !== "https://github.com/kaspanet/rusty-kaspa.git") {
    addFinding(findings, "error", "stable-network-must-use-official-kaspanet-repo", {
      net,
      expected: "https://github.com/kaspanet/rusty-kaspa.git",
      actual: networks[net].repo
    });
  }

  if (networks[net] && networks[net].family !== "mainline") {
    addFinding(findings, "error", "stable-network-family-must-be-mainline", {
      net,
      actual: networks[net].family
    });
  }
}

if (Object.keys(networks).sort().join(",") !== [...expectedNetworks].sort().join(",")) {
  addFinding(findings, "error", "unexpected-network-binding");
}
if (networks.testnet13 && (networks.testnet13.repo !== "https://github.com/kaspanet/rusty-kaspa.git" || networks.testnet13.branch !== "dagknight")) {
  addFinding(findings, "error", "tn13-must-use-official-dagknight");
}
if (networks.testnet13 && networks.testnet13.family !== "tn13") {
  addFinding(findings, "error", "testnet13-family-must-be-tn13", {
    actual: networks.testnet13.family
  });
}

if (networks.mainnet && networks.testnet10) {
  for (const key of ["repo", "branch", "rev", "feature"]) {
    if (networks.mainnet[key] !== networks.testnet10[key]) {
      addFinding(findings, "error", "mainnet-testnet10-stable-binding-mismatch", {
        key,
        mainnet: networks.mainnet[key],
        testnet10: networks.testnet10[key]
      });
    }
  }
}

if (networks.testnet13) {
  if (networks.testnet13.experimental !== true) {
    addFinding(findings, "error", "testnet13-must-be-marked-experimental", {
      actual: networks.testnet13.experimental
    });
  }

  if (networks.testnet13.enabledByDefault !== false) {
    addFinding(findings, "error", "testnet13-must-be-disabled-by-default", {
      actual: networks.testnet13.enabledByDefault
    });
  }
}

const cargoAliases = [];
let lockSources = [];
let remoteChecks = [];

if (!findings.some((x) => x.level === "error")) {
  const nodeCargo = read(files.nodeCargo);
  const bridgeCargo = read(files.bridgeCargo);
  const cliCargo = read(files.cliCargo);
  const coreCargo = read(files.coreCargo);
  const applyScript = read(files.applyScript);

  // Check every runtime alias; checking only kaspad/bridge misses RPC type drift.
  const aliases = [];
  for (const [family, network] of [["mainline", "mainnet"], ["tn13", "testnet13"]]) {
    for (const [alias, pkg] of [
      ["kaspad-lib", "kaspad"], ["kaspa-core", "kaspa-core"],
      ["kaspa-grpc-client", "kaspa-grpc-client"], ["kaspa-rpc-core", "kaspa-rpc-core"],
      ["kaspa-utils", "kaspa-utils"], ["kaspa-wrpc-server", "kaspa-wrpc-server"]
    ]) aliases.push(["node", alias + "-" + family, pkg, network, nodeCargo]);
    for (const pkg of ["kaspa-grpc-client", "kaspa-rpc-core", "kaspa-stratum-bridge"])
      aliases.push(["bridge", pkg + "-" + family, pkg, network, bridgeCargo]);
  }
  aliases.push(["cli", "kaspa-grpc-client-live", "kaspa-grpc-client", "mainnet", cliCargo]);
  aliases.push(["cli", "kaspa-rpc-core-live", "kaspa-rpc-core", "mainnet", cliCargo]);

  for (const [scope, alias, expectedPackage, network, source] of aliases) {
    const parsed = parseCargoAlias(source, alias);
    cargoAliases.push({ scope, alias, expectedPackage, network, parsed });

    if (!parsed) {
      addFinding(findings, "error", "missing-cargo-alias", { scope, alias });
      continue;
    }

    const b = networks[network];
    if (!b) continue;

    if (parsed.package !== expectedPackage) {
      addFinding(findings, "error", "cargo-package-mismatch", { scope, alias, expected: expectedPackage, actual: parsed.package });
    }

    if (parsed.git !== b.repo) {
      addFinding(findings, "error", "cargo-repo-mismatch", { scope, alias, expected: b.repo, actual: parsed.git });
    }

    if (parsed.rev !== b.rev) {
      addFinding(findings, "error", "cargo-rev-mismatch", { scope, alias, expected: b.rev, actual: parsed.rev });
    }

    if (parsed.branch) {
      addFinding(findings, "error", "cargo-branch-field-forbidden-use-rev-only", { scope, alias, branch: parsed.branch });
    }

    if (!parsed.optional) {
      addFinding(findings, "error", "cargo-alias-must-be-optional", { scope, alias });
    }
  }

  const coreAddress = parseCargoAlias(coreCargo, "kaspa-addresses");
  cargoAliases.push({
    scope: "core",
    alias: "kaspa-addresses",
    expectedPackage: "kaspa-addresses",
    network: "mainnet",
    parsed: coreAddress
  });
  if (!coreAddress) {
    addFinding(findings, "error", "missing-cargo-alias", { scope: "core", alias: "kaspa-addresses" });
  } else if (networks.mainnet) {
    if (coreAddress.git !== networks.mainnet.repo) {
      addFinding(findings, "error", "cargo-repo-mismatch", {
        scope: "core", alias: "kaspa-addresses", expected: networks.mainnet.repo, actual: coreAddress.git
      });
    }
    if (coreAddress.rev !== networks.mainnet.rev) {
      addFinding(findings, "error", "cargo-rev-mismatch", {
        scope: "core", alias: "kaspa-addresses", expected: networks.mainnet.rev, actual: coreAddress.rev
      });
    }
    if (coreAddress.branch) {
      addFinding(findings, "error", "cargo-branch-field-forbidden-use-rev-only", {
        scope: "core", alias: "kaspa-addresses", branch: coreAddress.branch
      });
    }
  }

  const applyRequirements = [
    "nodeAliases",
    "packages.node",
    "bridgeAliases",
    "packages.bridge",
    "kaspa-grpc-client-live",
    "kaspa-rpc-core-live",
    "kaspa-addresses",
    "testnet13",
    "Get-BindingByFamily \"tn13\""
  ];
  for (const needle of applyRequirements) {
    if (!applyScript.includes(needle)) {
      addFinding(findings, "error", "apply-script-missing-manifest-driven-coverage", { needle });
    }
  }

  const serviceController = read(files.serviceController);
  const officialRuntime = read(files.officialRuntime);
  const bridgeRuntime = read(files.bridgeRuntime);

  function requireContains(fileLabel, source, needle, description) {
    if (!source.includes(needle)) {
      addFinding(findings, "error", "rust-mapping-missing", { fileLabel, description, needle });
    }
  }

  if (networks.mainnet && networks.testnet10) {
    requireContains("kgw_service_controller.rs", serviceController, 'Self::Mainnet | Self::Testnet10 => "' + networks.mainnet.branch + '"', "stable service branch");
    requireContains("kgw_service_controller.rs", serviceController, 'Self::Mainnet | Self::Testnet10 => "' + networks.mainnet.rev + '"', "stable service rev");
    requireContains("official_kaspa_runtime.rs", officialRuntime, 'Self::Mainnet | Self::Testnet10 => "' + networks.mainnet.branch + '"', "stable node branch");
    requireContains("official_kaspa_runtime.rs", officialRuntime, 'Self::Mainnet | Self::Testnet10 => "' + networks.mainnet.rev + '"', "stable node rev");
    requireContains("src/lib.rs bridge runtime", bridgeRuntime, 'Self::Mainnet | Self::Testnet10 => "' + networks.mainnet.branch + '"', "stable bridge branch");
    requireContains("src/lib.rs bridge runtime", bridgeRuntime, 'Self::Mainnet | Self::Testnet10 => "' + networks.mainnet.rev + '"', "stable bridge rev");
    requireContains("official_kaspa_runtime.rs", officialRuntime, "Self::Mainnet | Self::Testnet10 => KaspaRuntimeFamily::Mainline", "stable node family");
    requireContains("src/lib.rs bridge runtime", bridgeRuntime, "Self::Mainnet | Self::Testnet10 => BridgeRuntimeFamily::Mainline", "stable bridge family");
  }

  if (networks.testnet13) {
    requireContains("kgw_service_controller.rs", serviceController, 'Self::Testnet13 => "' + networks.testnet13.branch + '"', "testnet13 service branch");
    requireContains("kgw_service_controller.rs", serviceController, 'Self::Testnet13 => "' + networks.testnet13.rev + '"', "testnet13 service rev");
    requireContains("official_kaspa_runtime.rs", officialRuntime, 'Self::Testnet13 => "' + networks.testnet13.branch + '"', "testnet13 node branch");
    requireContains("official_kaspa_runtime.rs", officialRuntime, 'Self::Testnet13 => "' + networks.testnet13.rev + '"', "testnet13 node rev");
    requireContains("src/lib.rs bridge runtime", bridgeRuntime, 'Self::Testnet13 => "' + networks.testnet13.branch + '"', "testnet13 bridge branch");
    requireContains("src/lib.rs bridge runtime", bridgeRuntime, 'Self::Testnet13 => "' + networks.testnet13.rev + '"', "testnet13 bridge rev");
    requireContains("official_kaspa_runtime.rs", officialRuntime, "Self::Testnet13 => KaspaRuntimeFamily::Tn13", "testnet13 node family");
    requireContains("src/lib.rs bridge runtime", bridgeRuntime, "Self::Testnet13 => BridgeRuntimeFamily::Tn13", "testnet13 bridge family");
  }

  lockSources = exists(files.cargoLock) ? parseLockGitSources(read(files.cargoLock)) : [];
  if (!exists(files.cargoLock)) addFinding(findings, "error", "cargo-lock-required");
  for (const item of cargoAliases) {
    const binding = networks[item.network];
    if (!binding) continue;
    if (!lockSources.some((source) => source.name === item.expectedPackage &&
        normalizeRepo(source.url) === normalizeRepo(binding.repo) &&
        source.rev === binding.rev)) {
      addFinding(findings, "error", "cargo-lock-runtime-package-drift", {
        alias: item.alias, package: item.expectedPackage, expectedRev: binding.rev
      });
    }
  }
  for (const source of lockSources) {
    if (/rusty-kaspa/i.test(source.url) &&
        (normalizeRepo(source.url) !== "https://github.com/kaspanet/rusty-kaspa" ||
         ![networks.mainnet.rev, networks.testnet13.rev].includes(source.rev))) {
      addFinding(findings, "error", "cargo-lock-unapproved-runtime-source", { source });
    }
  }

  if (exists(files.cargoLock)) {
    for (const net of expectedNetworks) {
      const b = networks[net];
      if (!b || !b.rev) continue;

      const matches = lockSources.filter((item) => normalizeRepo(item.url) === normalizeRepo(b.repo));
      const hasRev = matches.some((item) => item.rev.toLowerCase() === b.rev.toLowerCase());

      if (!matches.length) {
        addFinding(findings, "warn", "cargo-lock-no-git-source-for-repo", { net, repo: b.repo });
      } else if (!hasRev) {
        addFinding(findings, "error", "cargo-lock-does-not-resolve-manifest-rev", {
          net,
          expectedRev: b.rev,
          lockRevs: Array.from(new Set(matches.map((m) => m.rev)))
        });
      }
    }
  }

  if (onlineMode && !offlineMode) {
    const groups = [];
    const seen = new Set();

    for (const [net, b] of Object.entries(networks)) {
      if (!b || !b.repo || !b.branch) continue;
      const key = b.repo + "#" + b.branch;
      if (seen.has(key)) continue;
      seen.add(key);
      groups.push({
        repo: b.repo,
        branch: b.branch,
        networks: Object.keys(networks).filter((n) => networks[n].repo === b.repo && networks[n].branch === b.branch)
      });
    }

    for (const group of groups) {
      const remote = lsRemote(group.repo, group.branch);
      const expectedRevs = Array.from(new Set(group.networks.map((n) => networks[n].rev)));

      const check = Object.assign({}, group, remote, {
        expectedRevs,
        fresh: remote.ok && expectedRevs.length === 1 && expectedRevs[0].toLowerCase() === remote.latestRev.toLowerCase()
      });

      remoteChecks.push(check);

      if (!check.ok) {
        addFinding(findings, strictMode ? "error" : "warn", "remote-ls-remote-failed", {
          repo: group.repo,
          branch: group.branch,
          status: check.status,
          stderr: check.stderr
        });
      } else if (!check.fresh) {
        addFinding(findings, "error", "manifest-rev-is-not-latest-remote-branch-head", {
          repo: group.repo,
          branch: group.branch,
          networks: group.networks,
          manifestRevs: expectedRevs,
          latestRev: check.latestRev
        });
      }
    }
  }
}

const errors = findings.filter((x) => x.level === "error");
const warnings = findings.filter((x) => x.level === "warn");

const output = {
  ok: errors.length === 0,
  verdict: errors.length === 0 ? "KGW_RUNTIME_REPOSITORY_BINDING_GATE_R21C_PASSED" : "KGW_RUNTIME_REPOSITORY_BINDING_GATE_R21C_FAILED",
  mode: {
    strict: strictMode,
    online: onlineMode && !offlineMode,
    offline: offlineMode
  },
  files,
  networks,
  cargoAliases,
  lockSources,
  remoteChecks,
  findings,
  errorCount: errors.length,
  warningCount: warnings.length
};

if (jsonMode) {
  console.log(JSON.stringify(output, null, 2));
} else {
  console.log(output.verdict);
  console.log("errors:", output.errorCount);
  console.log("warnings:", output.warningCount);
  for (const finding of findings) {
    console.log(finding.level.toUpperCase() + ": " + finding.issue + " " + JSON.stringify(finding));
  }
}

process.exit(errors.length === 0 ? 0 : 1);
