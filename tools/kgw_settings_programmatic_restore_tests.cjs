const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const check = (value, message) => { if (!value) throw new Error(message); };
const functionBlock = (source, start, next) => {
  const begin = source.indexOf(start);
  const end = source.indexOf(next, begin + start.length);
  check(begin >= 0 && end > begin, `unable to isolate ${start}`);
  return source.slice(begin, end);
};

const node = read("apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js");
check(
  /function kgwNodeSettingsWithProgrammaticWriteR9B\(callback\)\s*\{\s*return callback\(\);\s*\}/.test(node),
  "Node programmatic restore call boundary is missing",
);
const nodeRestore = functionBlock(node, "function kgwNodeR51RestoreDefaults", "function kgwNodeR51IsRunning");
check(nodeRestore.includes("kgwNodeSettingsWithProgrammaticWriteR9B(() =>"), "Node Restore Defaults must use the programmatic boundary");
check(nodeRestore.includes("kgwNodeR51WriteSettings(net, defaults)"), "Node Restore Defaults must write restored settings");
check(nodeRestore.includes("kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true })"), "Node Restore Defaults must refresh backend-owned default paths");

const bridge = read("apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js");
check(
  /function kgwBridgeSettingsWithProgrammaticWriteR9B\(callback\)\s*\{\s*return callback\(\);\s*\}/.test(bridge),
  "Bridge programmatic restore call boundary is missing",
);
const bridgeRestore = functionBlock(bridge, "function kgwBridgeR51RestoreDefaults", "function kgwBridgeR51IsRunning");
check(bridgeRestore.includes("kgwBridgeSettingsWithProgrammaticWriteR9B(() =>"), "Bridge Restore Defaults must use the programmatic boundary");
check(bridgeRestore.includes("kgwBridgeR51WriteSettings(net, defaults)"), "Bridge Restore Defaults must write restored settings");
check(bridgeRestore.includes("kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true })"), "Bridge Restore Defaults must refresh backend-owned default paths");

console.log("kgw_settings_programmatic_restore_tests: PASS");
