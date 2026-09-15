const fs = require("node:fs");
const path = require("node:path");
const root = path.resolve(__dirname, "..");
const binding = fs.readFileSync(path.join(root, "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js"), "utf8");
const settings = fs.readFileSync(path.join(root, "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js"), "utf8");
const backend = fs.readFileSync(path.join(root, "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs"), "utf8");
function check(v, m) { if (!v) throw new Error(m); }
check(binding.includes('"30d": "last_month"') && binding.includes('"90d": "last_3_months"') && binding.includes('"1y": "last_year"'), "frontend range aliases missing");
check(binding.includes('kgw:saved-addresses-changed') && settings.includes('kgwNotifySavedAddressesChanged'), "saved-address invalidation contract missing");
check(backend.includes("fn select_latest_records") && backend.includes("select_latest_records(&mut records, request.limit)"), "canonical latest-N selection missing");
check(backend.includes('"30d" => "last_month"') && backend.includes('"90d" => "last_3_months"') && backend.includes('"1y" => "last_year"'), "backend alias normalization missing");
console.log("kgw_analysis_contract_tests: PASS");
