// AUD-001 focused tests of the actual Restore UI owner; no DOM/network dependency.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");
const source = fs.readFileSync("apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js", "utf8");
const start = source.indexOf("// AUD-001: one restore flight");
const end = source.indexOf("function kgwInstallSettingsDbMaintenanceActions", start);
assert.ok(start > 0 && end > start);
function harness(invoke, render = async () => true, consent = true) {
  const nodes = new Map();
  const events = [];
  const controls = [{ disabled: false }, { disabled: true }];
  const context = {
    console: { log() {}, error() {} },
    document: {
      getElementById: (id) => nodes.get(id),
      createElement: () => ({ dataset: {}, setAttribute(key, value) { this[key] = value; } }),
      querySelector: () => ({ appendChild(node) { nodes.set(node.id, node); } }),
      querySelectorAll: () => controls,
    },
    KGW_SETTINGS_DB_ACTION_STATE: { restoring: false },
    KGW_SETTINGS_ADDRESS_STATE: { restoreEpoch: 0 },
    confirm: () => consent,
    kgwSettingsDbInvoke: () => invoke,
    kgwSettingsDbStatus: (message) => events.push(message),
    kgwRenderSettingsDatabaseRows: () => events.push("database rows"),
    kgwRenderSettingsAddressRows: async (rows, options) => { events.push("address rows"); assert.equal(options.localOnly, true); return render(rows); },
    kgwClearSettingsAddressFields() {}, kgwSettingsAddressSetStatus() {}, kgwSettingsAddressNow: () => "test-time",
  };
  vm.createContext(context);
  vm.runInContext(source.slice(start, end), context);
  return { context, events, controls, run: () => context.kgwSettingsRestoreLatest(), status: () => nodes.get("settingsRestoreStatus") };
}
const good = { ok: true, backup_path: "audit/OLD", rows: [], message: "Restored verified OLD" };
async function main() {
  let release;
  let calls = 0;
  const pending = new Promise((resolve) => { release = resolve; });
  let finishRender;
  const renderPending = new Promise((resolve) => { finishRender = resolve; });
  const happy = harness((command) => {
    if (command === "get_all_addresses") return Promise.resolve([{ name: "OLD" }]);
    calls += 1; return pending;
  }, () => renderPending);
  const first = happy.run();
  assert.equal(happy.status().dataset.state, "running");
  assert.ok(happy.controls.every((node) => node.disabled));
  assert.equal(await happy.run(), null);
  assert.equal(calls, 1, "repeat request must not queue another restore");
  release(good);
  await new Promise(setImmediate);
  assert.equal(happy.status().dataset.state, "running", "backend success is not UI success before rendering");
  finishRender(true);
  assert.equal((await first).backup_path, good.backup_path);
  assert.equal(happy.status().dataset.state, "success");
  assert.ok(happy.events.indexOf("address rows") < happy.events.indexOf(good.message));
  assert.deepEqual(happy.controls.map((node) => node.disabled), [false, true]);
  assert.equal(happy.context.KGW_SETTINGS_DB_ACTION_STATE.restoring, false);
  for (const mode of ["native-error", "ok-false", "missing-target", "bad-rows", "read-error", "render-error", "missing-invoke"]) {
    const invoke = mode === "missing-invoke" ? null : async (command) => {
      if (command === "get_all_addresses") {
        if (mode === "read-error") throw new Error("fresh read failed");
        return [{ name: "OLD" }];
      }
      if (mode === "native-error") throw new Error("target changed; current data preserved");
      if (mode === "ok-false") return { ...good, ok: false };
      if (mode === "missing-target") return { ...good, backup_path: null };
      if (mode === "bad-rows") return { ...good, rows: null };
      return good;
    };
    const test = harness(invoke, async () => mode !== "render-error");
    assert.equal(await test.run(), null, mode);
    assert.equal(test.status().dataset.state, "error", mode);
    assert.equal(test.status().role, "alert", mode);
    assert.equal(test.context.KGW_SETTINGS_DB_ACTION_STATE.restoring, false, mode);
    assert.deepEqual(test.controls.map((node) => node.disabled), [false, true], mode);
  }
  const cancelled = harness(() => { throw new Error("cancelled request must not invoke"); }, undefined, false);
  assert.equal(await cancelled.run(), null);
  assert.equal(cancelled.status().dataset.state, "cancelled");
  const renderStart = source.indexOf("async function kgwRenderSettingsAddressRows(");
  const renderEnd = source.indexOf("async function kgwRefreshSettingsAddresses", renderStart);
  let releaseOld;
  const rows = { innerHTML: "restored OLD" };
  const state = { restoreEpoch: 0 };
  const stale = { KGW_SETTINGS_ADDRESS_STATE: state, kgwSettingsAddressElements: () => ({ rows }), kgwEnrichSettingsAddressRows: () => new Promise((resolve) => { releaseOld = resolve; }) };
  vm.createContext(stale);
  vm.runInContext(source.slice(renderStart, renderEnd), stale);
  const oldRender = stale.kgwRenderSettingsAddressRows([{ name: "NEW" }]);
  state.restoreEpoch += 1;
  releaseOld([{ name: "NEW" }]);
  assert.equal(await oldRender, false);
  assert.equal(rows.innerHTML, "restored OLD", "pre-restore async refresh must not overwrite restored rows");
  assert.ok(source.includes("await kgwSettingsRestoreLatest();"), "real Restore button must call the guarded owner");
  console.log("AUD-001 frontend regression: PASS (single-flight, ordered success, seven failures, cancellation, stale refresh)");
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
