const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css"),
  lib: path.join(repoRoot, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs"),
};

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function saveJson(name, value) {
  fs.mkdirSync(reportDir, { recursive: true });
  fs.writeFileSync(path.join(reportDir, name), JSON.stringify(value, null, 2), "utf8");
}

function count(text, re) {
  const m = text.match(re);
  return m ? m.length : 0;
}

function functionBounds(text, name) {
  const re = new RegExp("function\\s+" + name + "\\s*\\([^)]*\\)\\s*\\{");
  const m = text.match(re);
  if (!m) throw new Error("Function not found: " + name);

  const start = m.index;
  const brace = text.indexOf("{", start);
  let depth = 0;
  let end = brace;

  for (; end < text.length; end++) {
    const ch = text[end];
    if (ch === "{") depth++;
    if (ch === "}") {
      depth--;
      if (depth === 0) {
        end++;
        break;
      }
    }
  }

  return { start, end, body: text.slice(start, end) };
}

function replaceFunction(text, name, replacement) {
  const b = functionBounds(text, name);
  return text.slice(0, b.start) + replacement + text.slice(b.end);
}

function removeBlock(text, start, end) {
  const re = new RegExp("\\n?\\/\\* " + start + " \\*\\/[\\s\\S]*?\\/\\* " + end + " \\*\\/\\n?", "g");
  return text.replace(re, "\n");
}

function insertBeforeFunction(text, name, block) {
  text = removeBlock(text, "KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START", "KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END");
  const idx = text.indexOf("function " + name);
  if (idx < 0) throw new Error("Insertion function not found: " + name);
  return text.slice(0, idx) + block + text.slice(idx);
}

function nodeR11Block() {
  return `
/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */
const kgwNodeSettingsFeedbackLocksR11 = new Map();

function kgwNodeSettingsIsFeedbackLockedR11(net) {
  const lock = kgwNodeSettingsFeedbackLocksR11.get(net);
  return Boolean(lock && lock.until > Date.now());
}

function kgwNodeSettingsForceButtonsDisabledR11(net, reason = "feedback-lock") {
  ["save-settings", "restore-defaults", "set-defaults"].forEach((action) => {
    const buttons = Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`));
    buttons.forEach((button) => {
      button.disabled = true;
      button.setAttribute("aria-disabled", "true");
      button.dataset.kgwSettingsActionDisabled = "1";
      button.classList.add("kgw-settings-action-disabled");
    });
  });

  if (typeof kgwNodeSettingsTraceR4D === "function") {
    kgwNodeSettingsTraceR4D(net, "settings-buttons", "disabled", {
      reason,
      feedbackLocked: true,
      nativeDisabledExpected: true,
    });
  }
}

function kgwNodeSettingsStartFeedbackLockR11(net, action, button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const until = Date.now() + 10000;

  const lock = {
    action,
    label,
    original,
    until,
    timer: null,
    interval: null,
  };

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.dataset.kgwOriginalLabel = target.dataset.kgwOriginalLabel || original;
    target.dataset.kgwFeedbackActive = "1";
    target.dataset.kgwFeedbackUntil = String(until);
    target.textContent = label;
  });

  kgwNodeSettingsFeedbackLocksR11.set(net, lock);
  kgwNodeSettingsForceButtonsDisabledR11(net, "feedback-lock-start-" + action);

  lock.interval = window.setInterval(() => {
    if (!kgwNodeSettingsIsFeedbackLockedR11(net)) return;

    targets.forEach((target) => {
      target.textContent = label;
      target.disabled = true;
      target.setAttribute("aria-disabled", "true");
      target.dataset.kgwSettingsActionDisabled = "1";
      target.classList.add("kgw-settings-action-disabled");
    });
  }, 100);

  lock.timer = window.setTimeout(() => {
    window.clearInterval(lock.interval);

    targets.forEach((target) => {
      target.dataset.kgwFeedbackActive = "0";
      target.dataset.kgwFeedbackUntil = "0";
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });

    kgwNodeSettingsFeedbackLocksR11.delete(net);
    kgwNodeSettingsAcceptBaselineR7C(net, "feedback-lock-end-" + action);
    kgwNodeSettingsForceButtonsDisabledR11(net, "feedback-lock-end-" + action);
  }, 10000);
}
/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

`;
}

function bridgeR11Block() {
  return `
/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START */
const kgwBridgeSettingsFeedbackLocksR11 = new Map();

function kgwBridgeSettingsIsFeedbackLockedR11(net) {
  const lock = kgwBridgeSettingsFeedbackLocksR11.get(net);
  return Boolean(lock && lock.until > Date.now());
}

function kgwBridgeSettingsForceButtonsDisabledR11(net, reason = "feedback-lock") {
  ["save-settings", "restore-defaults", "set-defaults"].forEach((action) => {
    const buttons = Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`));
    buttons.forEach((button) => {
      button.disabled = true;
      button.setAttribute("aria-disabled", "true");
      button.dataset.kgwSettingsActionDisabled = "1";
      button.classList.add("kgw-settings-action-disabled");
    });
  });

  if (typeof kgwBridgeSettingsTraceR4D === "function") {
    kgwBridgeSettingsTraceR4D(net, "settings-buttons", "disabled", {
      reason,
      feedbackLocked: true,
      nativeDisabledExpected: true,
    });
  }
}

function kgwBridgeSettingsStartFeedbackLockR11(net, action, button, label) {
  const original = button.dataset.kgwOriginalLabel || button.textContent;
  const until = Date.now() + 10000;

  const lock = {
    action,
    label,
    original,
    until,
    timer: null,
    interval: null,
  };

  const targets = action && net
    ? Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`))
    : [button];

  targets.forEach((target) => {
    target.dataset.kgwOriginalLabel = target.dataset.kgwOriginalLabel || original;
    target.dataset.kgwFeedbackActive = "1";
    target.dataset.kgwFeedbackUntil = String(until);
    target.textContent = label;
  });

  kgwBridgeSettingsFeedbackLocksR11.set(net, lock);
  kgwBridgeSettingsForceButtonsDisabledR11(net, "feedback-lock-start-" + action);

  lock.interval = window.setInterval(() => {
    if (!kgwBridgeSettingsIsFeedbackLockedR11(net)) return;

    targets.forEach((target) => {
      target.textContent = label;
      target.disabled = true;
      target.setAttribute("aria-disabled", "true");
      target.dataset.kgwSettingsActionDisabled = "1";
      target.classList.add("kgw-settings-action-disabled");
    });
  }, 100);

  lock.timer = window.setTimeout(() => {
    window.clearInterval(lock.interval);

    targets.forEach((target) => {
      target.dataset.kgwFeedbackActive = "0";
      target.dataset.kgwFeedbackUntil = "0";
      target.textContent = target.dataset.kgwOriginalLabel || original;
    });

    kgwBridgeSettingsFeedbackLocksR11.delete(net);
    kgwBridgeSettingsAcceptBaselineR7C(net, "feedback-lock-end-" + action);
    kgwBridgeSettingsForceButtonsDisabledR11(net, "feedback-lock-end-" + action);
  }, 10000);
}
/* KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_END */

`;
}

function patchNode(text) {
  text = insertBeforeFunction(text, "kgwNodeR51ReadSettings", nodeR11Block());

  text = replaceFunction(text, "kgwNodeSetSettingsActionEnabledR2", `function kgwNodeSetSettingsActionEnabledR2(net, action, enabled) {
  const locked = kgwNodeSettingsIsFeedbackLockedR11(net);
  const finalEnabled = locked ? false : Boolean(enabled);
  const buttons = Array.from(document.querySelectorAll(\`[data-node-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = !finalEnabled;
    button.setAttribute("aria-disabled", finalEnabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = finalEnabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !finalEnabled);
  });
}`);

  text = replaceFunction(text, "kgwNodeFlashSettingsActionButtonR2", `function kgwNodeFlashSettingsActionButtonR2(button, label) {
  const action = button.dataset.nodeAction;
  const net = button.dataset.net || "";

  if (action && net) {
    kgwNodeSettingsStartFeedbackLockR11(net, action, button, label);
    return;
  }

  const original = button.dataset.kgwOriginalLabel || button.textContent;
  button.dataset.kgwOriginalLabel = original;
  button.textContent = label;

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR11);
  button.__kgwSettingsFeedbackTimerR11 = window.setTimeout(() => {
    button.textContent = button.dataset.kgwOriginalLabel || original;
  }, 10000);
}`);

  const install = functionBounds(text, "installActions");
  let body = install.body;

  body = body.replace(
    `    if (kgwNodeSettingsIsProgrammaticWriteR9B()) return;
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "input");`,
    `    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (kgwNodeSettingsIsProgrammaticWriteR9B() || kgwNodeSettingsIsFeedbackLockedR11(net)) return;
    if (net) kgwNodeSettingsChangedR7C(net, "input");`
  );

  body = body.replace(
    `    if (kgwNodeSettingsIsProgrammaticWriteR9B()) return;
    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (net) kgwNodeSettingsChangedR7C(net, "change");`,
    `    const net = kgwNodeSettingsNetFromEventR7C(event);
    if (kgwNodeSettingsIsProgrammaticWriteR9B() || kgwNodeSettingsIsFeedbackLockedR11(net)) return;
    if (net) kgwNodeSettingsChangedR7C(net, "change");`
  );

  text = text.slice(0, install.start) + body + text.slice(install.end);

  text = replaceFunction(text, "kgwNodeSettingsAfterActionR7C", `function kgwNodeSettingsAfterActionR7C(net, action) {
  kgwNodeSettingsWithProgrammaticWriteR9B(() => {
    kgwNodeSettingsAcceptBaselineR7C(net, "after-" + action + "-immediate-r11");
  });
}`);

  return text;
}

function patchBridge(text) {
  text = insertBeforeFunction(text, "kgwBridgeR51ReadSettings", bridgeR11Block());

  text = replaceFunction(text, "kgwBridgeSetSettingsActionEnabledR2", `function kgwBridgeSetSettingsActionEnabledR2(net, action, enabled) {
  const locked = kgwBridgeSettingsIsFeedbackLockedR11(net);
  const finalEnabled = locked ? false : Boolean(enabled);
  const buttons = Array.from(document.querySelectorAll(\`[data-bridge-action="\${action}"][data-net="\${net}"]\`));

  buttons.forEach((button) => {
    button.disabled = !finalEnabled;
    button.setAttribute("aria-disabled", finalEnabled ? "false" : "true");
    button.dataset.kgwSettingsActionDisabled = finalEnabled ? "0" : "1";
    button.classList.toggle("kgw-settings-action-disabled", !finalEnabled);
  });
}`);

  text = replaceFunction(text, "kgwBridgeFlashSettingsActionButtonR2", `function kgwBridgeFlashSettingsActionButtonR2(button, label) {
  const action = button.dataset.bridgeAction;
  const net = button.dataset.net || button.dataset.network || "";

  if (action && net) {
    kgwBridgeSettingsStartFeedbackLockR11(net, action, button, label);
    return;
  }

  const original = button.dataset.kgwOriginalLabel || button.textContent;
  button.dataset.kgwOriginalLabel = original;
  button.textContent = label;

  window.clearTimeout(button.__kgwSettingsFeedbackTimerR11);
  button.__kgwSettingsFeedbackTimerR11 = window.setTimeout(() => {
    button.textContent = button.dataset.kgwOriginalLabel || original;
  }, 10000);
}`);

  const install = functionBounds(text, "installActions");
  let body = install.body;

  body = body.replace(
    `    if (kgwBridgeSettingsIsProgrammaticWriteR9B()) return;
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "input");`,
    `    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (kgwBridgeSettingsIsProgrammaticWriteR9B() || kgwBridgeSettingsIsFeedbackLockedR11(net)) return;
    if (net) kgwBridgeSettingsChangedR7C(net, "input");`
  );

  body = body.replace(
    `    if (kgwBridgeSettingsIsProgrammaticWriteR9B()) return;
    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (net) kgwBridgeSettingsChangedR7C(net, "change");`,
    `    const net = kgwBridgeSettingsNetFromEventR7C(event, root);
    if (kgwBridgeSettingsIsProgrammaticWriteR9B() || kgwBridgeSettingsIsFeedbackLockedR11(net)) return;
    if (net) kgwBridgeSettingsChangedR7C(net, "change");`
  );

  text = text.slice(0, install.start) + body + text.slice(install.end);

  text = replaceFunction(text, "kgwBridgeSettingsAfterActionR7C", `function kgwBridgeSettingsAfterActionR7C(net, action) {
  kgwBridgeSettingsWithProgrammaticWriteR9B(() => {
    kgwBridgeSettingsAcceptBaselineR7C(net, "after-" + action + "-immediate-r11");
  });
}`);

  return text;
}

function auditAfter() {
  const node = read(files.nodeJs);
  const bridge = read(files.bridgeJs);
  const lib = read(files.lib);

  const nodeInstall = functionBounds(node, "installActions").body;
  const bridgeInstall = functionBounds(bridge, "installActions").body;

  return {
    node: {
      r11Markers: count(node, /KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START/g),
      feedbackLockMap: count(node, /kgwNodeSettingsFeedbackLocksR11/g),
      forceDisabled: count(node, /kgwNodeSettingsForceButtonsDisabledR11/g),
      keepAlive100ms: count(node, /setInterval\(\(\) =>/g),
      feedbackTimer10000: count(node, /10000/g),
      setEnabledUsesLock: functionBounds(node, "kgwNodeSetSettingsActionEnabledR2").body.includes("kgwNodeSettingsIsFeedbackLockedR11"),
      flashStartsLock: functionBounds(node, "kgwNodeFlashSettingsActionButtonR2").body.includes("kgwNodeSettingsStartFeedbackLockR11"),
      inputGuardUsesLock: nodeInstall.includes("kgwNodeSettingsIsFeedbackLockedR11(net)"),
      afterActionNo1000Timer: !functionBounds(node, "kgwNodeSettingsAfterActionR7C").body.includes("1000"),
      nativeDisabledByEnabled: functionBounds(node, "kgwNodeSetSettingsActionEnabledR2").body.includes("button.disabled = !finalEnabled"),
    },
    bridge: {
      r11Markers: count(bridge, /KGW_SETTINGS_FEEDBACK_LOCK_OWNER_R11_START/g),
      feedbackLockMap: count(bridge, /kgwBridgeSettingsFeedbackLocksR11/g),
      forceDisabled: count(bridge, /kgwBridgeSettingsForceButtonsDisabledR11/g),
      keepAlive100ms: count(bridge, /setInterval\(\(\) =>/g),
      feedbackTimer10000: count(bridge, /10000/g),
      setEnabledUsesLock: functionBounds(bridge, "kgwBridgeSetSettingsActionEnabledR2").body.includes("kgwBridgeSettingsIsFeedbackLockedR11"),
      flashStartsLock: functionBounds(bridge, "kgwBridgeFlashSettingsActionButtonR2").body.includes("kgwBridgeSettingsStartFeedbackLockR11"),
      inputGuardUsesLock: bridgeInstall.includes("kgwBridgeSettingsIsFeedbackLockedR11(net)"),
      afterActionNo1000Timer: !functionBounds(bridge, "kgwBridgeSettingsAfterActionR7C").body.includes("1000"),
      nativeDisabledByEnabled: functionBounds(bridge, "kgwBridgeSetSettingsActionEnabledR2").body.includes("button.disabled = !finalEnabled"),
    },
    rust: {
      traceCommand: count(lib, /fn\s+kgw_frontend_button_trace_v1\s*\(/g),
      tracePrint: count(lib, /\[KGW_BUTTON_TRACE\]/g),
      handlerRefs: count(lib, /kgw_frontend_button_trace_v1/g),
    },
  };
}

saveJson("audit-before-r11.json", {
  nodeLength: read(files.nodeJs).length,
  bridgeLength: read(files.bridgeJs).length,
});

const beforeNode = read(files.nodeJs);
const beforeBridge = read(files.bridgeJs);

const afterNode = patchNode(beforeNode);
const afterBridge = patchBridge(beforeBridge);

write(files.nodeJs, afterNode);
write(files.bridgeJs, afterBridge);

saveJson("patch-changes-r11.json", {
  nodeChanged: beforeNode !== afterNode,
  bridgeChanged: beforeBridge !== afterBridge,
});

const after = auditAfter();
saveJson("audit-after-r11.json", after);

const failures = [];

for (const [scope, data] of Object.entries({ node: after.node, bridge: after.bridge })) {
  if (data.r11Markers !== 1) failures.push(scope + " R11 marker must exist exactly once.");
  if (data.feedbackLockMap < 1) failures.push(scope + " feedback lock map missing.");
  if (data.forceDisabled < 1) failures.push(scope + " force-disabled helper missing.");
  if (data.keepAlive100ms < 1) failures.push(scope + " 100ms feedback keep-alive missing.");
  if (data.feedbackTimer10000 < 1) failures.push(scope + " 10s feedback timer missing.");
  if (!data.setEnabledUsesLock) failures.push(scope + " set-enabled does not respect feedback lock.");
  if (!data.flashStartsLock) failures.push(scope + " flash does not start feedback lock.");
  if (!data.inputGuardUsesLock) failures.push(scope + " input/change guard does not respect feedback lock.");
  if (!data.afterActionNo1000Timer) failures.push(scope + " after-action still has delayed 1000ms timer.");
  if (!data.nativeDisabledByEnabled) failures.push(scope + " native disabled by finalEnabled missing.");
}

if (after.rust.traceCommand !== 1) failures.push("Rust trace command must exist exactly once.");
if (after.rust.tracePrint !== 1) failures.push("Rust trace print marker must exist exactly once.");
if (after.rust.handlerRefs < 2) failures.push("Rust trace command must be registered.");

if (failures.length) {
  throw new Error("R11 validation failed:\n- " + failures.join("\n- "));
}

console.log("# KGW R11 patch complete");
console.log(JSON.stringify(after, null, 2));