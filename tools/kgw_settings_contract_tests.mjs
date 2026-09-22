import assert from "node:assert/strict";
import test from "node:test";
import {
  NODE_ENDPOINTS, NODE_MANAGED, nodeFieldEnabled, validateNodeForm,
  isHost, isPort, endpoint, listenersOverlap, runtimePresentation,
} from "../apps/kaspa-gateway-desktop/frontend/src/settings-contract.js";

const values = () => ({
  rpcListenEnabled: true, rpcListenHost: "127.0.0.1", rpcListenPort: "16110",
  rpcBorshEnabled: false, rpcBorshHost: "invalid host", rpcBorshPort: "",
  rpcJsonEnabled: false, listenEnabled: false, externalIpEnabled: false,
  connectEnabled: false, addPeerEnabled: false, noLogFiles: true, perfMetrics: true,
  asyncThreads: "16", ramScale: "1", rpcMaxClients: "16",
  outPeers: "8", maxInPeers: "32", logLevel: "info",
});
test("default form uses valid KGW settings; blank optional values are inactive", () => {
  assert.deepEqual(validateNodeForm(values(), {}, "mainnet"), {});
});
for (const [parent, host, port] of NODE_ENDPOINTS.slice(1)) {
  test(parent + " controls both children and validation", () => {
    const v = { ...values(), [parent]: false, [host]: "", [port]: "" };
    assert.equal(nodeFieldEnabled(host, v), false);
    assert.equal(nodeFieldEnabled(port, v), false);
    assert.deepEqual(validateNodeForm(v, {}, "mainnet"), {});
    v[parent] = true;
    assert.equal(nodeFieldEnabled(host, v), true);
    const errors = validateNodeForm(v, {}, "mainnet");
    assert.ok(errors[host]); assert.ok(errors[port]);
    v[host] = "127.0.0.1"; v[port] = "19100";
    assert.deepEqual(validateNodeForm(v, {}, "mainnet"), {});
  });
}
for (const name of Object.keys(NODE_MANAGED)) {
  test(name + " stays managed even when an inclusion option is true", () => {
    assert.equal(nodeFieldEnabled(name, values(), {[name]: true}), false);
  });
}
test("explicit optional empty value must be corrected before Save/Start", () => {
  assert.ok(validateNodeForm(values(), {uaComment: true}, "mainnet").uaComment);
  assert.equal(validateNodeForm({...values(), uaComment: "desktop"}, {uaComment: true}, "mainnet").uaComment, undefined);
});
test("disabled parent suppresses a stored logging/performance/cache draft", () => {
  const v = {...values(), logDir: "", perfMetrics: false, perfMetricsInterval: "", rocksDbPreset: "default"};
  assert.equal(nodeFieldEnabled("logDir", v, {logDir: true}), false);
  assert.equal(nodeFieldEnabled("perfMetricsInterval", v, {perfMetricsInterval: true}), false);
  assert.equal(nodeFieldEnabled("rocksDbCacheSize", v, {rocksDbCacheSize: true}), false);
});
for (const [key, bad] of [["asyncThreads","0"],["ramScale","NaN"],["ramScale","11"],["rpcMaxClients","17"],["outPeers","9"],["maxInPeers","33"],["asyncThreads","1.2"]]) {
  test(key + " rejects " + bad, () => assert.ok(validateNodeForm({...values(), [key]:bad}, {}, "mainnet")[key]));
}
test("IP and hostname validation covers IPv4, IPv6 and invalid input", () => {
  for (const host of ["127.0.0.1", "::1", "[2001:db8::1]", "node.example", "localhost"]) assert.ok(isHost(host), host);
  for (const host of ["300.1.2.3", "127.0.0", "-bad", "bad host", "a/b", "::::"]) assert.equal(isHost(host), false, host);
  assert.equal(endpoint("::1", 16110), "[::1]:16110");
  assert.ok(isPort("65535")); assert.equal(isPort("65536"), false); assert.equal(isPort("0"), false);
});
test("wildcard/localhost aliases conflict but different exact interfaces do not", () => {
  assert.ok(listenersOverlap({host:"0.0.0.0",port:"16110"},{host:"127.0.0.1",port:"16110"}));
  assert.ok(listenersOverlap({host:"localhost",port:16110},{host:"127.0.0.1",port:16110}));
  assert.equal(listenersOverlap({host:"192.168.1.1",port:16110},{host:"192.168.1.2",port:16110}),false);
  assert.ok(validateNodeForm({...values(),listenEnabled:true,listenHost:"0.0.0.0",listenPort:"16110"},{},"mainnet").listenPort);
});
test("RPC stays loopback unless unsafe RPC is selected", () => {
  const v = {...values(),rpcListenHost:"0.0.0.0"};
  assert.ok(validateNodeForm(v,{},"mainnet").rpcListenHost);
  assert.equal(validateNodeForm({...v,unsafeRpc:true},{},"mainnet").rpcListenHost,undefined);
});
test("network profiles are independent and unsynced mining is test-only", () => {
  const main = values(), tn = {...values(),rpcListenPort:"16210",enableUnsyncedMining:true};
  assert.deepEqual(validateNodeForm(tn,{},"testnet10"),{});
  assert.equal(main.rpcListenPort,"16110");
  assert.ok(validateNodeForm(tn,{},"mainnet").enableUnsyncedMining);
});
test("enabled profile never implies a running process or synchronized network", () => {
  const s = runtimePresentation({enabled:true,running:false});
  assert.equal(s.profile,"Enabled"); assert.equal(s.process,"Stopped"); assert.equal(s.network,"Not connected");
  assert.equal(runtimePresentation({running:true}).network,"Synchronization not reported");
  assert.equal(runtimePresentation({running:true,synced:false}).network,"Not synchronized");
  assert.equal(runtimePresentation({error:"exit 1"}).process,"Failed");
});

import {BRIDGE_MANAGED, bridgeFieldEnabled, validateBridgeForm} from "../apps/kaspa-gateway-desktop/frontend/src/settings-contract.js";
const bridge = () => ({
  nodeMode:"external", kaspadAddress:"127.0.0.1:16110", stratumPort:":5555", promPort:":2112",
  minShareDiff:"8192", sharesPerMin:"30", extranonceSize:"0", blockWaitTime:"50ms",
  inprocessRpcListen:"127.0.0.1:16110", inprocessRamScale:"1", inprocessAsyncThreads:"16",
  inprocessOutpeers:"8", inprocessMaxInpeers:"32",
});
test("Bridge defaults validate without starting an owner", () => {
  assert.deepEqual(validateBridgeForm(bridge(),{}, "mainnet"),{});
});
test("Bridge external mode excludes all stored in-process values", () => {
  const v={...bridge(),inprocessRpcListen:"bad",inprocessRamScale:"bad"};
  assert.equal(bridgeFieldEnabled("inprocessRpcListen",v,{}),false);
  assert.deepEqual(validateBridgeForm(v,{},"mainnet"),{});
});
test("Bridge in-process mode validates its node endpoint and CPU count", () => {
  assert.deepEqual(validateBridgeForm({...bridge(),nodeMode:"inprocess"},{}, "mainnet"),{});
  const e=validateBridgeForm({...bridge(),nodeMode:"inprocess",inprocessRpcListen:"",inprocessAsyncThreads:"0"},{}, "mainnet");
  assert.ok(e.inprocessRpcListen); assert.ok(e.inprocessAsyncThreads);
});
for (const field of ["kaspadAddress","stratumPort","minShareDiff","blockWaitTime"]) {
  test("Bridge required " + field + " rejects an empty value", () => {
    assert.ok(validateBridgeForm({...bridge(),[field]:""},{}, "mainnet")[field]);
  });
}
test("Bridge optional arguments require values only when included", () => {
  assert.ok(validateBridgeForm(bridge(),{coinbaseTagSuffix:true},"mainnet").coinbaseTagSuffix);
  assert.deepEqual(validateBridgeForm(bridge(),{coinbaseTagSuffix:false},"mainnet"),{});
});
test("Bridge config mode suppresses conflicting inline values but validates its path", () => {
  const v={...bridge(),config:"C:\\KGW\\bridge.yaml",kaspadAddress:""};
  assert.equal(bridgeFieldEnabled("kaspadAddress",v,{config:true}),false);
  assert.deepEqual(validateBridgeForm(v,{config:true},"mainnet"),{});
  assert.ok(validateBridgeForm({...v,config:"relative.yaml"},{config:true},"mainnet").config);
});
test("Bridge managed values never become editable effective overrides", () => {
  for(const key of Object.keys(BRIDGE_MANAGED)) assert.equal(bridgeFieldEnabled(key,bridge(),{[key]:true}),false,key);
});
test("Bridge in-process RPC remains loopback and peer modes remain exclusive", () => {
  const v={...bridge(),nodeMode:"inprocess",inprocessRpcListen:"0.0.0.0:16110",inprocessConnect:"127.0.0.1:16111",inprocessAddPeer:"127.0.0.1:16112"};
  const e=validateBridgeForm(v,{inprocessConnect:true,inprocessAddPeer:true},"mainnet");
  assert.ok(e.inprocessRpcListen); assert.ok(e.inprocessAddPeer);
});
test("Bridge disabled metrics and miner parents suppress invalid child drafts", () => {
  const v={...bridge(),nodeMode:"inprocess",inprocessPerfMetrics:false,inprocessPerfMetricsIntervalSec:"",internalCpuMiner:false,internalCpuMinerThreads:"bad"};
  assert.deepEqual(validateBridgeForm(v,{inprocessPerfMetricsIntervalSec:true},"testnet10"),{});
});


for (const network of ["testnet10", "testnet13"]) {
  test(network + " excludes stale ASIC settings from restored CPU-only profiles", () => {
    const v = {...bridge(), network, stratumPort:"bad", promPort:"bad", minShareDiff:"bad", internalCpuMiner:false};
    for (const field of ["stratumPort", "promPort", "minShareDiff", "config"])
      assert.equal(bridgeFieldEnabled(field, v, {[field]:true}), false, field);
    assert.deepEqual(validateBridgeForm(v, {}, network), {});
  });
  test(network + " requires enabled CPU miner address and bounded threads", () => {
    const v = {...bridge(), internalCpuMiner:true, internalCpuMinerAddress:"", internalCpuMinerThreads:"0"};
    const e = validateBridgeForm(v, {}, network);
    assert.ok(e.internalCpuMinerAddress); assert.ok(e.internalCpuMinerThreads);
    const valid = {...v, internalCpuMinerAddress:"kaspatest:checked-by-backend", internalCpuMinerThreads:"1"};
    assert.deepEqual(validateBridgeForm(valid, {}, network), {});
    assert.ok(validateBridgeForm({...valid, internalCpuMinerThreads:"257"}, {}, network).internalCpuMinerThreads);
  });
  test(network + " accepts blank optional CPU pacing but rejects invalid active values", () => {
    const v = {...bridge(), internalCpuMiner:true, internalCpuMinerAddress:"kaspatest:checked-by-backend", internalCpuMinerThreads:"1"};
    assert.deepEqual(validateBridgeForm({...v, internalCpuMinerThrottleMs:"", internalCpuMinerTemplatePollMs:""}, {}, network), {});
    assert.ok(validateBridgeForm({...v, internalCpuMinerTemplatePollMs:"0"}, {}, network).internalCpuMinerTemplatePollMs);
    assert.ok(validateBridgeForm({...v, internalCpuMinerThrottleMs:"60001"}, {}, network).internalCpuMinerThrottleMs);
  });
}

import {runtimeObservationSummary} from "../apps/kaspa-gateway-desktop/frontend/src/settings-contract.js";
test("startup readiness never becomes live synchronization or mining evidence", () => {
  const text = runtimeObservationSummary({readiness:"READY"}, true, true);
  assert.match(text, /RPC: Unknown/); assert.match(text, /Sync: Not reported/); assert.match(text, /CPU: Not reported/);
});
test("observed hashing and confirmed-blue counts retain their actual meaning", () => {
  const fields = {observation_state:"fresh",rpc_ready:"true",synced:"false",cpu_enabled:"true",cpu_hashes_tried:"9007199254740993",cpu_hashrate_hs:"12.5",cpu_blocks_submitted:"7",cpu_blocks_confirmed_blue:"2"};
  const text = runtimeObservationSummary(fields, true, true);
  assert.match(text, /Sync: Not synchronized/); assert.match(text, /CPU: Hashing/);
  assert.match(text, /Hashes: 9007199254740993/); assert.match(text, /Submitted blocks: 7/);
  assert.match(text, /Confirmed blue blocks: 2/); assert.match(text, /12.50 H\/s/);
  assert.doesNotMatch(runtimeObservationSummary(fields, false, true), /Hashing|9007199254740993|12.50/);
});
test("RPC errors and unavailable observations cannot retain positive live states", () => {
  assert.match(runtimeObservationSummary({observation_state:"fresh",rpc_ready:"false",synced:"unknown",observation_error:"connection refused"},true), /RPC: Unavailable.*Sync: Not reported.*connection refused/);
  assert.match(runtimeObservationSummary({observation_state:"unavailable",synced:"true",cpu_enabled:"true",cpu_hashrate_hs:"9"},true,true), /Sync: Not reported.*CPU: Not reported/);
});
