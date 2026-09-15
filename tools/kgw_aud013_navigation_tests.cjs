const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const path = require('node:path');

const repo = path.resolve(__dirname, '..');
const mainPath = path.join(repo, 'apps/kaspa-gateway-desktop/frontend/main.js');
const source = fs.readFileSync(mainPath, 'utf8');
const start = source.indexOf('let kgwShellPendingSavedMainTabR102C = "";');
const end = source.indexOf('async function openTab(tabId, options = {})');
assert.ok(start >= 0 && end > start, 'AUD-013 restore guard block missing');
const guardSource = source.slice(start, end);

function harness(saved = 'kaspa-bridge', active = 'kaspa-node') {
  const timers = [];
  const opens = [];
  const context = {
    KGW_TABS: ['explorer','kaspa-node','kaspa-bridge','analysis','top-addresses','log','settings'].map(id => ({ id })),
    kgwShellReadLastMainTabR101W2: () => saved,
    kgwShellDisplayOwnerActiveButtonR59C: () => ({ dataset: { tab: active } }),
    kgwShellDisplayOwnerTabIdVisibleR59C: () => true,
    kgwMainTabTraceR35C: () => {},
    openTab: (tab, options) => { opens.push({ tab, options }); active = tab; return Promise.resolve(true); },
    window: { location: { hash: `#${active}` }, setTimeout: (fn, delay) => { timers.push({ fn, delay }); return timers.length; } },
    String, Number, Array, Boolean, console
  };
  vm.createContext(context);
  vm.runInContext(guardSource, context);
  const api = vm.runInContext(`({
    schedule: kgwShellScheduleSavedMainTabRestoreR102C,
    record: kgwShellRecordExplicitNavigationR103,
    current: kgwShellExplicitNavigationIsCurrentR103,
    restoreCurrent: kgwShellSavedRestoreTokenCurrentR103
  })`, context);
  return { api, timers, opens, setActive: value => { active = value; context.window.location.hash = `#${value}`; } };
}

{
  const h = harness();
  assert.equal(h.api.schedule('boot-after-open-tab'), true);
  assert.equal(h.timers.length, 5);
  const generation = h.api.record('settings');
  h.setActive('settings');
  for (const timer of h.timers) timer.fn();
  assert.equal(h.opens.length, 0, 'stale bootstrap restore must not override explicit Settings navigation');
  assert.equal(h.api.current(generation), true);
}

{
  const h = harness('kaspa-bridge', 'kaspa-node');
  assert.equal(h.api.schedule('boot-after-open-tab'), true);
  assert.equal(h.timers.length, 5);
  h.timers[0].fn();
  assert.equal(h.opens[0]?.tab, 'kaspa-bridge', 'saved tab restore must still work before explicit navigation');
}
{
  const h = harness();
  const first = h.api.record('settings');
  const second = h.api.record('analysis');
  assert.equal(h.api.current(first), false, 'older explicit navigation must become stale');
  assert.equal(h.api.current(second), true, 'latest explicit navigation must win');
}

{
  const h = harness();
  h.api.record('settings');
  assert.equal(h.api.schedule('ensure-active-no-change'), false, 'display hydration must not schedule saved restore after explicit navigation');
  assert.equal(h.timers.length, 0);
}

assert.match(source, /explicitNavigationGeneration:\s*explicitNavigationGenerationR103/);
assert.match(source, /!kgwShellExplicitNavigationIsCurrentR103\(explicitNavigationGenerationR103\)/);
assert.match(source, /kgwShellRecordExplicitNavigationR103\(button\.dataset\.tab\)/);

console.log('kgw_aud013_navigation_tests: PASS');
