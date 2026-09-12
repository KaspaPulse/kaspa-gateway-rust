import assert from "node:assert/strict";
import fs from "node:fs";

const helper = fs.readFileSync(new URL("./tauri-app.mjs", import.meta.url), "utf8");
const matrix = fs.readFileSync(new URL("../specs/zero-touch-live-matrix.e2e.js", import.meta.url), "utf8");

assert.ok(
  helper.includes('`#bridge-${net}-instancePort-${bridgeInstanceId}`'),
  "Bridge runtime selection must read the actual instancePort DOM id",
);
assert.ok(
  /setControlValueById\(\s*`bridge-\$\{network\}-instancePort-\$\{selection\.bridgeInstanceId\}`/.test(matrix),
  "Bridge E2E must write the actual instancePort DOM id",
);
console.log("bridge locator smoke: PASS");
