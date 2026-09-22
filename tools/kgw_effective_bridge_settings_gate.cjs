#!/usr/bin/env node
const assert = require("assert");
const fs = require("fs");
const vm = require("vm");

const sourcePath = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const source = fs.readFileSync(sourcePath, "utf8");

const values = new Map([
  ["bridge-mainnet-nodeMode", "external"],
  ["bridge-mainnet-kaspadAddress", "127.0.0.1:16110"],
  ["bridge-mainnet-blockWaitTime", "1500ms"],
  ["bridge-mainnet-printStats", "false"],
  ["bridge-mainnet-logToFile", "false"],
  ["bridge-mainnet-healthCheckPort", ""],
  ["bridge-mainnet-webDashboardPort", ""],
  ["bridge-mainnet-varDiff", "true"],
  ["bridge-mainnet-sharesPerMin", "20"],
  ["bridge-mainnet-varDiffStats", "false"],
  ["bridge-mainnet-extranonceSize", "2"],
  ["bridge-mainnet-pow2Clamp", "false"],
  ["bridge-mainnet-coinbaseTagSuffix", "owner-test"],
  ["bridge-mainnet-stratumPort", "5555"],
  ["bridge-mainnet-minShareDiff", "8192"],
  ["bridge-mainnet-promPort", "2112"],
  ["bridge-mainnet-config", ""],
]);

const elements = new Map();
for (const [key, value] of values) {
  elements.set(key, { id: key, value, checked: false, type: "text" });
}

const sandbox = {
  document: {
    querySelector() { return { querySelectorAll() { return [...elements.values()]; } }; },
    getElementById(id) {
      return elements.get(id) || null;
    },
  },
  console,
};
sandbox.window = sandbox;
sandbox.globalThis = sandbox;

const selected = [
  fs.readFileSync("apps/kaspa-gateway-desktop/frontend/src/settings-contract.js", "utf8").replace(/^export /gm, ""),
  source.slice(source.indexOf("function kgwBridgeCommandInlineStateKeyR7("), source.indexOf("function kgwBridgeCommandInlineToggleR7(")),
  source.slice(source.indexOf("function bridgeNodeMode("), source.indexOf("function bridgeHasConfig(")),
  source.slice(source.indexOf("function kgwBridgeR51Panel("), source.indexOf("function kgwBridgeR51Fields(")),
  source.slice(source.indexOf("function kgwBridgeForm("), source.indexOf("function kgwBridgeValidateForm(")),
  source.slice(source.indexOf("const BRIDGE_NETWORKS = ["), source.indexOf("function kgwBridgeSetNetworkEnabled")),
  source.slice(source.indexOf("function byId("), source.indexOf("function esc(")),
  source.slice(source.indexOf("function id("), source.indexOf("function iid(")),
  source.slice(source.indexOf("function v("), source.indexOf("function iv(")),
  source.slice(source.indexOf("function kgwBridgeInstanceCommandStateKeyR13B("), source.indexOf("function kgwBridgeInstanceCommandCheckboxR13B(")),
  source.slice(source.indexOf("function bridgeInstanceSplitParts("), source.indexOf("function bridgeExtractPortsFromTextR5(")),
  source.slice(source.indexOf("function bridgeProfile("), source.indexOf("/* KGW_BRIDGE_NETWORK_PORT_PROFILES_SOFT_POLICY_PATCH_R35B")),
  source.slice(source.indexOf("function bridgeHasConfig("), source.indexOf("function bridgeControl(")),
].join("\n");

vm.runInNewContext(
  `${selected}\nthis.api = { bridgeInstanceParseStructured, bridgeNormalizeInstanceRecord, kgwBridgeEffectiveSettingsV1 };`,
  sandbox,
  { filename: sourcePath },
);

const { api } = sandbox;
sandbox.__kgwBridgeCommandComposerInlineR7 = {mainnet: {coinbaseTagSuffix: true}};

const parsed = api.bridgeInstanceParseStructured(
  "port=:5556,prom=:2113,wait=2500ms,extranonce=4,log=false,var_diff=true,shares_per_min=30,var_diff_stats=true,pow2_clamp=true",
);
assert.strictEqual(parsed.instanceBlockWaitTime, "2500ms");
assert.strictEqual(parsed.instanceExtranonceSize, "4");

const structured = {
  activeInstance: "one",
  instances: [
    {
      id: "one",
      instancePort: "5556",
      instanceDiff: "4096",
      instanceProm: "2113",
      instanceLogToFile: "not set",
      instanceBlockWaitTime: "2500ms",
      instanceExtranonceSize: "4",
      instanceVarDiff: "not set",
      instanceSharesPerMin: "30",
      instanceVarDiffStats: "true",
      instancePow2Clamp: "not set",
    },
    {
      id: "two",
      instancePort: "5557",
      instanceDiff: "2048",
      instanceProm: "2114",
      instanceLogToFile: "true",
      instanceVarDiff: "false",
      instanceVarDiffStats: "false",
      instancePow2Clamp: "true",
    },
  ],
};

sandbox.bridgeInstances = {mainnet: structured.instances};
let settings = api.kgwBridgeEffectiveSettingsV1("mainnet", structured);
assert.strictEqual(settings.version, 1);
assert.strictEqual(settings.global.blockWaitTimeMs, 1500);
assert.strictEqual(settings.global.printStats, false);
assert.strictEqual(settings.global.logToFile, false);
assert.strictEqual(settings.global.coinbaseTagSuffix, "owner-test");
assert.strictEqual(settings.instances.length, 2);
assert.strictEqual(settings.instances[0].stratumListen, ":5556");
assert.strictEqual(settings.instances[0].prometheusListen, ":2113");
assert.strictEqual(settings.instances[0].logToFile, null, "not set must preserve official inheritance");
assert.strictEqual(settings.instances[0].varDiff, null, "not set must preserve official inheritance");
assert.strictEqual(settings.instances[0].blockWaitTimeMs, 2500);
assert.strictEqual(settings.instances[0].extranonceSize, 4);
assert.strictEqual(settings.instances[1].logToFile, true);
assert.strictEqual(settings.instances[1].varDiff, false);

sandbox.__kgwBridgeInstanceCommandComposerR13B["mainnet::two::instance"] = false;
sandbox.__kgwBridgeInstanceCommandComposerR13B["mainnet::one::instanceProm"] = false;
settings = api.kgwBridgeEffectiveSettingsV1("mainnet", structured);
assert.strictEqual(settings.instances.length, 1, "excluded instances must not cross IPC");
assert.strictEqual(
  settings.instances[0].prometheusListen,
  ":2112",
  "an excluded first-instance Prometheus override must inherit the bridge-level value",
);

sandbox.__kgwBridgeCommandComposerInlineR7.mainnet.promPort = false;
settings = api.kgwBridgeEffectiveSettingsV1("mainnet", structured);
assert.strictEqual(settings.instances[0].prometheusListen, null, "disabled global and instance Prometheus options must not start a hidden listener");
sandbox.__kgwBridgeCommandComposerInlineR7.mainnet.promPort = true;
elements.set("bridge-mainnet-inprocessRpcListen", {id:"bridge-mainnet-inprocessRpcListen", value:"127.0.0.1:19110", type:"text"});
elements.get("bridge-mainnet-nodeMode").value = "inprocess";
assert.strictEqual(api.kgwBridgeEffectiveSettingsV1("mainnet", structured).global.kaspaRpcEndpoint, "127.0.0.1:19110");
elements.get("bridge-mainnet-nodeMode").value = "external";
elements.get("bridge-mainnet-config").value = "/tmp/official-bridge.yaml";
assert.ok(api.kgwBridgeEffectiveSettingsV1("mainnet", structured), "disabled config draft must not activate config mode");
sandbox.__kgwBridgeCommandComposerInlineR7.mainnet.config = true;
assert.strictEqual(
  api.kgwBridgeEffectiveSettingsV1("mainnet", structured),
  null,
  "config-file mode must not also send structured settings",
);

console.log("KGW effective Bridge settings gate PASSED");
