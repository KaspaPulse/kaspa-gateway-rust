const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(
  path.join(root, "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js"),
  "utf8",
);
const check = (value, message) => { if (!value) throw new Error(message); };

check(!source.includes("kgwFormatUsd("), "Explorer must not reference removed kgwFormatUsd alias");
check(!source.includes("kgwSummaryDayToSeconds("), "Explorer must not reference removed summary day alias");
check(!source.includes("setInterval(tick, 2000)"), "Explorer polling must not reference undefined tick");
check(source.includes("kgwSummaryFormatUsd("), "Explorer must use imported summary USD formatter");
check(source.includes("kgwDayToEpochSeconds(day, false)"), "Explorer day start must use imported UTC day helper");
check(source.includes("kgwDayToEpochSeconds(day, true)"), "Explorer day end must use imported UTC day helper");
check(source.includes("setInterval(kgwLiveDbPollingTick, 2000)"), "Explorer live DB polling must schedule its local owner");

console.log("kgw_explorer_lint_contract_tests: PASS");
