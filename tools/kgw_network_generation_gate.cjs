#!/usr/bin/env node
"use strict";

const cp = require("child_process");
const fs = require("fs");
const path = require("path");

const repoRoot = process.cwd();
const legacyRe = new RegExp("(?:testnet|tn)[ _-]?" + String(12), "i");

function gitTrackedFiles() {
  const out = cp.execFileSync("git", ["ls-files", "-z"], {
    cwd: repoRoot,
    encoding: "buffer",
    windowsHide: true,
  });
  return out.toString("utf8").split("\0").filter(Boolean);
}

function isBinary(buffer) {
  return buffer.includes(0);
}

function scanText(rel, text) {
  const findings = [];
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i += 1) {
    if (legacyRe.test(lines[i])) {
      findings.push({ path: rel, line: i + 1, text: lines[i].trim().slice(0, 240) });
    }
  }
  return findings;
}

function scanRepository() {
  const findings = [];
  for (const rel of gitTrackedFiles()) {
    if (legacyRe.test(rel)) {
      findings.push({ path: rel, line: 0, text: "legacy network token in tracked path" });
    }
    const full = path.join(repoRoot, rel);
    let buffer;
    try {
      buffer = fs.readFileSync(full);
    } catch {
      continue;
    }
    if (isBinary(buffer)) continue;
    findings.push(...scanText(rel, buffer.toString("utf8")));
  }
  return findings;
}

function main() {
  const findings = scanRepository();
  if (findings.length) {
    console.error("KGW_NETWORK_GENERATION_GATE_FAILED");
    for (const finding of findings) {
      console.error(JSON.stringify(finding));
    }
    process.exit(1);
  }
  console.log("KGW_NETWORK_GENERATION_GATE_PASSED");
}

if (require.main === module) main();

module.exports = { legacyRe, scanText };
