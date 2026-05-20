const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_deep_owner_trace_r12i.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js")
};

function read(file) {
  return fs.readFileSync(file, "utf8");
}

function write(file, text) {
  fs.writeFileSync(file, text, "utf8");
}

function writeJson(fileName, data) {
  fs.writeFileSync(path.join(reportDir, fileName), JSON.stringify(data, null, 2) + "\n", "utf8");
}

function findMatchingBrace(text, openIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;

  for (let i = openIndex; i < text.length; i++) {
    const ch = text[i];
    const next = text[i + 1];

    if (lineComment) {
      if (ch === "\n") lineComment = false;
      continue;
    }

    if (blockComment) {
      if (ch === "*" && next === "/") {
        blockComment = false;
        i++;
      }
      continue;
    }

    if (quote) {
      if (escaped) {
        escaped = false;
        continue;
      }
      if (ch === "\\") {
        escaped = true;
        continue;
      }
      if (ch === quote) {
        quote = null;
      }
      continue;
    }

    if (ch === "/" && next === "/") {
      lineComment = true;
      i++;
      continue;
    }

    if (ch === "/" && next === "*") {
      blockComment = true;
      i++;
      continue;
    }

    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      continue;
    }

    if (ch === "{") depth++;
    if (ch === "}") depth--;

    if (depth === 0) return i;
  }

  return -1;
}

function findFunction(text, functionName) {
  const re = new RegExp("function\\s+" + functionName + "\\s*\\([^)]*\\)\\s*\\{");
  const match = text.match(re);
  if (!match || match.index === undefined) return null;

  const start = match.index;
  const open = text.indexOf("{", start);
  const close = findMatchingBrace(text, open);

  if (close < 0) return null;

  return {
    start,
    open,
    end: close + 1,
    source: text.slice(start, close + 1)
  };
}

function removeOldTrace(text) {
  return text.replace(/\/\/\s*KGW_SETTINGS_DEEP_OWNER_TRACE_R12I[\s\S]*?\/\/\s*END_KGW_SETTINGS_DEEP_OWNER_TRACE_R12I\s*/g, "");
}

function buildTraceBlock(scopeName) {
  return `
// KGW_SETTINGS_DEEP_OWNER_TRACE_R12I
(function installKgwSettingsDeepOwnerTraceR12I() {
  const TRACE_OWNER = "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I";
  const SCOPE = "${scopeName}";
  const EVENT_LIMIT_PER_SECOND = 300;
  let eventWindowStartedAt = Date.now();
  let eventWindowCount = 0;

  // literal validation markers:
  // phase=install-actions-root
  // phase=setting-input-before
  // phase=setting-input-after
  // phase=setting-change-before
  // phase=setting-change-after
  // phase=action-click-before
  // phase=action-click-after-300ms
  // phase=action-click-after-1500ms

  function nowIso() {
    try { return new Date().toISOString(); } catch (_) { return ""; }
  }

  function shouldTrace() {
    const now = Date.now();
    if (now - eventWindowStartedAt > 1000) {
      eventWindowStartedAt = now;
      eventWindowCount = 0;
    }
    eventWindowCount += 1;
    return eventWindowCount <= EVENT_LIMIT_PER_SECOND;
  }

  function lower(value) {
    return String(value || "").toLowerCase();
  }

  function hashText(text) {
    let hash = 0;
    const input = String(text || "");
    for (let i = 0; i < input.length; i += 1) {
      hash = ((hash << 5) - hash + input.charCodeAt(i)) | 0;
    }
    return String(hash);
  }

  function safeJson(value) {
    try { return JSON.stringify(value); } catch (_) { return ""; }
  }

  function trace(phase, details) {
    if (!shouldTrace()) return;

    try {
      const payload = {
        owner: TRACE_OWNER,
        scope: SCOPE,
        phase,
        details: Object.assign({ at: nowIso() }, details || {})
      };

      if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
        window.__TAURI__.core.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else if (window.__TAURI__ && typeof window.__TAURI__.invoke === "function") {
        window.__TAURI__.invoke("kgw_frontend_button_trace_v1", { payload }).catch(function () {});
      } else {
        console.debug("[KGW_SETTINGS_DEEP_OWNER_TRACE_R12I]", payload);
      }
    } catch (_) {}
  }

  function isControl(element) {
    if (!element || !element.tagName) return false;

    const tag = lower(element.tagName);
    if (tag !== "input" && tag !== "select" && tag !== "textarea") return false;

    const type = lower(element.type);
    if (type === "button" || type === "submit" || type === "reset" || type === "hidden") return false;

    return true;
  }

  function isActionButton(element) {
    if (!element || !element.tagName || lower(element.tagName) !== "button") return false;

    const text = lower(element.textContent);
    const action = lower(
      (element.dataset && (element.dataset.kgwSettingsAction || element.dataset.action)) ||
      element.getAttribute("data-action") ||
      element.getAttribute("aria-label") ||
      ""
    );

    return (
      action.includes("save") ||
      action.includes("restore") ||
      action.includes("default") ||
      text.includes("save settings") ||
      text.includes("restore defaults") ||
      text.includes("set as defaults") ||
      text.includes("saved") ||
      text.includes("restored") ||
      text.includes("حفظ") ||
      text.includes("استعادة") ||
      text.includes("افتراض")
    );
  }

  function guessNetwork(element) {
    let current = element;

    while (current && current !== document) {
      const dataset = current.dataset || {};
      const direct =
        dataset.network ||
        dataset.net ||
        dataset.kgwNetwork ||
        current.getAttribute("data-network") ||
        current.getAttribute("data-net") ||
        current.getAttribute("data-kgw-network");

      if (direct) return String(direct);

      const id = lower(current.id);
      const cls = lower(current.className);

      if (id.includes("testnet12") || cls.includes("testnet12") || id.includes("tn12") || cls.includes("tn12")) return "testnet12";
      if (id.includes("testnet10") || cls.includes("testnet10") || id.includes("tn10") || cls.includes("tn10")) return "testnet10";
      if (id.includes("mainnet") || cls.includes("mainnet")) return "mainnet";

      current = current.parentElement;
    }

    const local = lower((element && (element.name || element.id || element.className)) || "");
    if (local.includes("testnet12") || local.includes("tn12")) return "testnet12";
    if (local.includes("testnet10") || local.includes("tn10")) return "testnet10";
    if (local.includes("mainnet")) return "mainnet";

    return "unknown";
  }

  function fieldKey(element) {
    if (!element) return "";

    const dataset = element.dataset || {};
    return String(
      dataset.kgwSettingKey ||
      dataset.settingKey ||
      dataset.key ||
      dataset.option ||
      dataset.flag ||
      element.name ||
      element.id ||
      element.getAttribute("aria-label") ||
      element.getAttribute("placeholder") ||
      element.className ||
      element.tagName ||
      ""
    ).slice(0, 220);
  }

  function readControl(element) {
    if (!element) return {};

    const tag = lower(element.tagName);
    const type = lower(element.type);

    const data = {
      tag,
      type,
      id: element.id || "",
      name: element.name || "",
      key: fieldKey(element),
      network: guessNetwork(element),
      disabled: !!element.disabled,
      ariaDisabled: element.getAttribute("aria-disabled") || "",
      dataNetwork: element.getAttribute("data-network") || "",
      dataKey: element.getAttribute("data-key") || "",
      dataSettingKey: element.getAttribute("data-setting-key") || "",
      className: String(element.className || "").slice(0, 220)
    };

    if (tag === "input" && (type === "checkbox" || type === "radio")) {
      data.checked = !!element.checked;
      data.defaultChecked = !!element.defaultChecked;
      data.value = String(element.value || "");
    } else if (tag === "select") {
      data.value = String(element.value || "");
      data.defaultValue = String(element.defaultValue || "");
      data.selectedIndex = element.selectedIndex;
      data.selectedText = element.options && element.selectedIndex >= 0
        ? String(element.options[element.selectedIndex].textContent || "").slice(0, 180)
        : "";
    } else {
      data.value = String(element.value || "").slice(0, 260);
      data.defaultValue = String(element.defaultValue || "").slice(0, 260);
    }

    return data;
  }

  function listButtons(root, network) {
    const buttons = Array.from((root || document).querySelectorAll("button")).filter(isActionButton);

    return buttons.map(function (button) {
      return {
        text: String(button.textContent || "").trim().slice(0, 110),
        action: String(
          (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
          button.getAttribute("data-action") ||
          button.getAttribute("aria-label") ||
          ""
        ).slice(0, 110),
        network: guessNetwork(button),
        disabled: !!button.disabled,
        ariaDisabled: button.getAttribute("aria-disabled") || "",
        kgwR12eDisabled: button.dataset ? button.dataset.kgwR12eDisabled || "" : "",
        kgwSettingsActionDisabled: button.dataset ? button.dataset.kgwSettingsActionDisabled || "" : "",
        className: String(button.className || "").slice(0, 200)
      };
    }).filter(function (button) {
      return network === "unknown" || button.network === network || button.network === "unknown";
    });
  }

  function snapshot(root, network) {
    const controls = Array.from((root || document).querySelectorAll("input, select, textarea")).filter(isControl);

    const fields = controls.map(readControl).filter(function (item) {
      return network === "unknown" || item.network === network || item.network === "unknown";
    });

    const compact = fields.map(function (item) {
      return {
        key: item.key,
        id: item.id,
        name: item.name,
        network: item.network,
        type: item.type,
        value: item.value,
        checked: Object.prototype.hasOwnProperty.call(item, "checked") ? item.checked : null
      };
    });

    return {
      network,
      fieldCount: fields.length,
      hash: hashText(JSON.stringify(compact)),
      selected: compact.filter(function (item) {
        return item.checked === true || (item.value !== "" && item.value !== "false" && item.value !== "0");
      }).slice(0, 80),
      fields: fields.slice(0, 120)
    };
  }

  function traceControl(root, kind, event) {
    const target = event && event.target;
    if (!isControl(target)) return;

    const network = guessNetwork(target);
    const beforeField = readControl(target);
    const beforeSnapshot = snapshot(root, network);
    const beforePhase = kind === "input" ? "setting-input-before" : "setting-change-before";
    const afterPhase = kind === "input" ? "setting-input-after" : "setting-change-after";

    trace(beforePhase, {
      isTrusted: !!event.isTrusted,
      network,
      field: beforeField,
      snapshotHash: beforeSnapshot.hash,
      selected: beforeSnapshot.selected,
      actionButtons: listButtons(root, network)
    });

    setTimeout(function () {
      const afterField = readControl(target);
      const afterSnapshot = snapshot(root, network);

      trace(afterPhase, {
        isTrusted: !!event.isTrusted,
        network,
        beforeField,
        afterField,
        fieldChanged: safeJson(beforeField) !== safeJson(afterField),
        snapshotChanged: beforeSnapshot.hash !== afterSnapshot.hash,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        selected: afterSnapshot.selected,
        actionButtons: listButtons(root, network)
      });
    }, 0);

    setTimeout(function () {
      const afterSnapshot = snapshot(root, network);

      trace(afterPhase + "-300ms", {
        isTrusted: !!event.isTrusted,
        network,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        snapshotChanged: beforeSnapshot.hash !== afterSnapshot.hash,
        selected: afterSnapshot.selected,
        actionButtons: listButtons(root, network)
      });
    }, 300);
  }

  function traceAction(root, event) {
    const button = event && event.target && event.target.closest ? event.target.closest("button") : null;
    if (!isActionButton(button)) return;

    const network = guessNetwork(button);
    const beforeSnapshot = snapshot(root, network);

    trace("action-click-before", {
      isTrusted: !!event.isTrusted,
      network,
      button: {
        text: String(button.textContent || "").trim().slice(0, 120),
        action: String(
          (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
          button.getAttribute("data-action") ||
          button.getAttribute("aria-label") ||
          ""
        ).slice(0, 120),
        disabled: !!button.disabled,
        ariaDisabled: button.getAttribute("aria-disabled") || "",
        kgwR12eDisabled: button.dataset ? button.dataset.kgwR12eDisabled || "" : "",
        kgwSettingsActionDisabled: button.dataset ? button.dataset.kgwSettingsActionDisabled || "" : "",
        className: String(button.className || "").slice(0, 220)
      },
      snapshotHash: beforeSnapshot.hash,
      selected: beforeSnapshot.selected,
      actionButtons: listButtons(root, network)
    });

    setTimeout(function () {
      const afterSnapshot = snapshot(root, network);
      trace("action-click-after-300ms", {
        isTrusted: !!event.isTrusted,
        network,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        snapshotChanged: beforeSnapshot.hash !== afterSnapshot.hash,
        selected: afterSnapshot.selected,
        actionButtons: listButtons(root, network)
      });
    }, 300);

    setTimeout(function () {
      const afterSnapshot = snapshot(root, network);
      trace("action-click-after-1500ms", {
        isTrusted: !!event.isTrusted,
        network,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        snapshotChanged: beforeSnapshot.hash !== afterSnapshot.hash,
        selected: afterSnapshot.selected,
        actionButtons: listButtons(root, network)
      });
    }, 1500);
  }

  function install(root) {
    if (!root || root.dataset.kgwSettingsDeepOwnerTraceR12i === "installed") return;
    root.dataset.kgwSettingsDeepOwnerTraceR12i = "installed";

    trace("install-actions-root", {
      rootId: root.id || "",
      rootClass: String(root.className || "").slice(0, 180),
      initialSnapshot: snapshot(root, "unknown"),
      actionButtons: listButtons(root, "unknown")
    });

    root.addEventListener("input", function (event) {
      traceControl(root, "input", event);
    }, true);

    root.addEventListener("change", function (event) {
      traceControl(root, "change", event);
    }, true);

    root.addEventListener("click", function (event) {
      traceAction(root, event);
    }, true);
  }

  window.KGW_SETTINGS_DEEP_OWNER_TRACE_R12I = {
    install,
    snapshot,
    listButtons
  };
})();
// END_KGW_SETTINGS_DEEP_OWNER_TRACE_R12I
`;
}

function injectIntoInstallActions(text) {
  const installActions = findFunction(text, "installActions");
  if (!installActions) {
    throw new Error("installActions(root) function not found");
  }

  let body = installActions.source;

  const oldCallPattern = /if\s*\(\s*window\.KGW_SETTINGS_DEEP_OWNER_TRACE_R12I[\s\S]*?KGW_SETTINGS_DEEP_OWNER_TRACE_R12I\.install\(root\);\s*\}\s*/g;
  body = body.replace(oldCallPattern, "");

  const injection = `{
  if (window.KGW_SETTINGS_DEEP_OWNER_TRACE_R12I && typeof window.KGW_SETTINGS_DEEP_OWNER_TRACE_R12I.install === "function") {
    window.KGW_SETTINGS_DEEP_OWNER_TRACE_R12I.install(root);
  }`;

  body = body.replace("{", injection);

  return text.slice(0, installActions.start) + body + text.slice(installActions.end);
}

function patchFile(file, scopeName) {
  const before = read(file);
  let text = removeOldTrace(before);
  text = buildTraceBlock(scopeName) + "\n" + text;
  text = injectIntoInstallActions(text);

  write(file, text);

  return {
    file,
    scopeName,
    changed: before !== text,
    hasDeepTrace: text.includes("KGW_SETTINGS_DEEP_OWNER_TRACE_R12I"),
    hasInstallActionsCall: text.includes("KGW_SETTINGS_DEEP_OWNER_TRACE_R12I.install(root);"),
    hasInputTrace: text.includes("setting-input-before"),
    hasChangeTrace: text.includes("setting-change-before"),
    hasActionTrace: text.includes("action-click-before"),
    hasR12E: text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")
  };
}

function validateFile(file, label) {
  const text = read(file);

  const required = [
    "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I",
    "KGW_SETTINGS_DEEP_OWNER_TRACE_R12I.install(root);",
    "setting-input-before",
    "setting-input-after",
    "setting-change-before",
    "setting-change-after",
    "action-click-before",
    "action-click-after-300ms",
    "action-click-after-1500ms",
    "install-actions-root",
    "kgw_frontend_button_trace_v1"
  ];

  const errors = [];

  for (const item of required) {
    if (!text.includes(item)) {
      errors.push(label + ": missing " + item);
    }
  }

  if (!text.includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")) {
    errors.push(label + ": R12E owner missing");
  }

  return errors;
}

function main() {
  const beforeAudit = {
    nodeHadR12I: read(files.nodeJs).includes("KGW_SETTINGS_DEEP_OWNER_TRACE_R12I"),
    bridgeHadR12I: read(files.bridgeJs).includes("KGW_SETTINGS_DEEP_OWNER_TRACE_R12I"),
    nodeHadR12H2: read(files.nodeJs).includes("KGW_SETTINGS_SELECTION_TRACE_R12H2"),
    bridgeHadR12H2: read(files.bridgeJs).includes("KGW_SETTINGS_SELECTION_TRACE_R12H2"),
    nodeHasR12E: read(files.nodeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: read(files.bridgeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")
  };

  const nodeResult = patchFile(files.nodeJs, "node");
  const bridgeResult = patchFile(files.bridgeJs, "bridge");

  const validationErrors = [
    ...validateFile(files.nodeJs, "Node"),
    ...validateFile(files.bridgeJs, "Bridge")
  ];

  const afterAudit = {
    validationErrors,
    nodeResult,
    bridgeResult,
    nodeHasR12E: read(files.nodeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: read(files.bridgeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E")
  };

  writeJson("audit-before-r12i.json", beforeAudit);
  writeJson("audit-after-r12i.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R12I validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R12I deep owner trace installed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
