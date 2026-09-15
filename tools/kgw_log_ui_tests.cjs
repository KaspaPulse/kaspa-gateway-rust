const assert = require("node:assert/strict");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

global.window = {};
global.document = {
  getElementById: () => null,
  querySelector: () => null,
  documentElement: { lang: "en" }
};
global.localStorage = { getItem: () => null, setItem: () => {} };

(async () => {
  const modulePath = pathToFileURL(path.resolve(__dirname, "../apps/kaspa-gateway-desktop/frontend/src/tabs/log/log.js")).href;
  const mod = await import(modulePath + `?test=${Date.now()}`);
  const { kgwParseLogLevel, kgwCopyTextToClipboard } = mod;

  for (const level of ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"]) {
    assert.equal(kgwParseLogLevel(`2026-09-14 12:34:56 - ${level} - [MainThread] - audit - message`), level);
    assert.equal(kgwParseLogLevel(`time="x" level="${level}" target="audit" msg="message"`), level);
  }
  assert.equal(kgwParseLogLevel("ordinary text contains ERROR but is not a native log record"), "UNKNOWN");
  assert.equal(kgwParseLogLevel("2026-09-14 12:34:56 ERROR message"), "UNKNOWN");

  let apiWrites = 0;
  assert.equal(await kgwCopyTextToClipboard("a", { clipboard: { writeText: async () => { apiWrites += 1; } }, document: null }), true);
  assert.equal(apiWrites, 1);


  function fallbackDocument(result) {
    let removed = false;
    const textarea = { value: "", select() {}, remove() { removed = true; } };
    return {
      body: { appendChild(node) { assert.equal(node, textarea); } },
      createElement(tag) { assert.equal(tag, "textarea"); return textarea; },
      execCommand(command) { assert.equal(command, "copy"); return result; },
      get removed() { return removed; }
    };
  }

  const fallbackOk = fallbackDocument(true);
  assert.equal(await kgwCopyTextToClipboard("b", { clipboard: null, document: fallbackOk }), true);
  assert.equal(fallbackOk.removed, true);

  const rejectedThenFallback = fallbackDocument(true);
  assert.equal(await kgwCopyTextToClipboard("c", {
    clipboard: { writeText: async () => { throw new Error("denied"); } },
    document: rejectedThenFallback
  }), true);

  await assert.rejects(
    kgwCopyTextToClipboard("d", { clipboard: null, document: fallbackDocument(false) }),
    /fallback was rejected/i
  );
  await assert.rejects(
    kgwCopyTextToClipboard("e", { clipboard: null, document: null }),
    /unavailable/i
  );
  await assert.rejects(
    kgwCopyTextToClipboard("f", { clipboard: { writeText: async () => { throw new Error("api denied"); } }, document: null }),
    /api denied/i
  );

  console.log("kgw_log_ui_tests: PASS");
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
