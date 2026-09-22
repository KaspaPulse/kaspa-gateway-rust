import assert from "node:assert/strict";

// Read-only contract regression against an already-running native Tauri application.
// Start the owner-approved isolated dev session first. This gate starts no runtime.
const port = Number(process.env.KGW_NATIVE_CDP_PORT || 49377);
assert.ok(Number.isInteger(port) && port > 0 && port <= 65535, "Invalid local CDP port");
const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
const target = targets.find(item => item.type === "page" &&
  (item.url.startsWith("http://tauri.localhost") ||
   item.url.startsWith("http://127.0.0.1:1430/")));
assert.ok(target, "The native KGW WebView must already be running");
const socketUrl = new URL(target.webSocketDebuggerUrl);
assert.ok(["127.0.0.1", "localhost", "[::1]"].includes(socketUrl.hostname),
  "Only a loopback native debug endpoint is allowed");
const ws = new WebSocket(socketUrl);
const pending = new Map();
let nextId = 0;
const results = [];
try {
  await new Promise((resolve, reject) => {
    ws.addEventListener("open", resolve, { once: true });
    ws.addEventListener("error", reject, { once: true });
  });
  ws.addEventListener("message", event => {
    const response = JSON.parse(event.data);
    const waiter = pending.get(response.id);
    if (!waiter) return;
    pending.delete(response.id);
    clearTimeout(waiter.timer);
    response.error ? waiter.reject(new Error(JSON.stringify(response.error))) :
      waiter.resolve(response.result);
  });
  const evaluate = expression => new Promise((resolve, reject) => {
    const id = ++nextId;
    const timer = setTimeout(() => { pending.delete(id); reject(new Error("Native IPC timeout")); }, 15000);
    pending.set(id, { resolve, reject, timer });
    ws.send(JSON.stringify({ id, method: "Runtime.evaluate",
      params: { expression, awaitPromise: true, returnByValue: true } }));
  }).then(response => {
    assert.equal(response.exceptionDetails, undefined, "Native WebView evaluation failed");
    return response.result?.value;
  });
  const preview = payload => evaluate(`(async () => {
    if (!window.__TAURI__?.core?.invoke) throw Error("Native Tauri IPC unavailable");
    try { return { ok: true, value: await window.__TAURI__.core.invoke(
      "kgw_runtime_settings_preview_v1", ${JSON.stringify(payload)}) }; }
    catch (error) { return { ok: false, error: String(error) }; }
  })()`);
  for (const kind of ["node", "bridge"]) {
    for (const role of [null, kind, " " + kind.toUpperCase() + " ", "invalid"]) {
      const payload = { network: "mainnet", runtimeRole: role,
        nodeKind: kind === "node" ? "integrated-inproc" : "remote",
        bridgeKind: kind === "node" ? "disable" : "official-external-node" };
      const result = await preview(payload);
      assert.equal(result.ok, false, `Missing typed ${kind} settings must reject role ${role}`);
      assert.match(result.error, role === "invalid" ? /runtimeRole must be/ :
        kind === "node" ? /effectiveNodeSettings is required/ : /effectiveBridgeSettings is required/);
      results.push({ kind, role, rejected: true, error: result.error });
    }
  }
  const payload = { network: "mainnet", nodeKind: "integrated-inproc",
    bridgeKind: "disable", runtimeRole: " NODE ", effectiveNodeSettings: {
      asyncThreads: 2, ramScale: 0.1, outboundTarget: 0, inboundLimit: 0,
      disableDnsSeeding: true, p2pListen: "127.0.0.1:16111" } };
  const explicit = await preview(payload);
  assert.equal(explicit.ok, true);
  assert.equal(explicit.value.runtimeRole, "node");
  assert.equal(explicit.value.effectiveNodeSettings.asyncThreads, 2);
  const inferred = await preview({ ...payload, runtimeRole: null });
  assert.deepEqual(inferred, explicit, "Inferred and normalized role must resolve the same payload");
  console.log(JSON.stringify({ state: "VERIFIED_SUCCESS", method: "actual-native-read-only-IPC",
    time: new Date().toISOString(), cases: results, positiveCases: 2 }));
} finally {
  for (const waiter of pending.values()) clearTimeout(waiter.timer);
  ws.close();
}
