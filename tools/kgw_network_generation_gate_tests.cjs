#!/usr/bin/env node
"use strict";

const assert = require("assert");
const { legacyRe, scanText } = require("./kgw_network_generation_gate.cjs");

const legacy = [
  "testnet" + String(12),
  "testnet-" + String(12),
  "testnet " + String(12),
  "tn" + String(12),
  "tn-" + String(12),
  "tn " + String(12),
];

for (const token of legacy) {
  assert.equal(legacyRe.test(token), true, "legacy form must be rejected");
  assert.equal(scanText("fixture.txt", "prefix " + token + " suffix").length, 1);
}

for (const current of ["mainnet", "testnet10", "testnet13", "tn13", "testnet-13"]) {
  assert.equal(legacyRe.test(current), false, "current network must be accepted: " + current);
  assert.equal(scanText("fixture.txt", current).length, 0);
}

console.log("KGW network generation gate regression tests PASS");
