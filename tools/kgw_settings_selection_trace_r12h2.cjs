const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_selection_trace_r12h2.cjs <repoRoot> <reportDir>");
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

function removeExistingTraceBlocks(text) {
  return text
    .replace(/\/\/\s*KGW_SETTINGS_SELECTION_TRACE_R12H2[\s\S]*?\/\/\s*END_KGW_SETTINGS_SELECTION_TRACE_R12H2\s*/g, "")
    .replace(/\/\/\s*KGW_SETTINGS_SELECTION_TRACE_R12H[\s\S]*?\/\/\s*END_KGW_SETTINGS_SELECTION_TRACE_R12H\s*/g, "");
}

function buildTraceBlock(scopeName) {
  return `
// KGW_SETTINGS_SELECTION_TRACE_R12H2
(function installKgwSettingsSelectionTraceR12H2() {
  const TRACE_OWNER = "KGW_SETTINGS_SELECTION_TRACE_R12H2";
  const SCOPE = "${scopeName}";
  const EVENT_LIMIT_PER_SECOND = 120;
  let eventWindowStartedAt = Date.now();
  let eventWindowCount = 0;

  // literal validation markers:
  // setting-input-before
  // setting-input-after
  // setting-input-after-250ms
  // setting-change-before
  // setting-change-after
  // setting-change-after-250ms
  // action-click-before
  // action-click-after-250ms
  // action-click-after-1200ms

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

  function invokeTrace(phase, details) {
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
        console.debug("[KGW_SETTINGS_SELECTION_TRACE_R12H2]", payload);
      }
    } catch (_) {}
  }

  function isSettingsControl(element) {
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

  function findTabRoot() {
    const scopeNeedle = SCOPE === "node" ? "node" : "bridge";
    const candidates = Array.from(document.querySelectorAll("[data-tab], [id], .tab-pane, main, body"));

    for (const item of candidates) {
      const id = lower(item.id);
      const cls = lower(item.className);
      const dataTab = lower(item.getAttribute && item.getAttribute("data-tab"));

      if (id.includes(scopeNeedle) || cls.includes(scopeNeedle) || dataTab.includes(scopeNeedle)) {
        return item;
      }
    }

    return document.body;
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
      element.name ||
      element.id ||
      element.getAttribute("aria-label") ||
      element.getAttribute("placeholder") ||
      element.className ||
      element.tagName ||
      ""
    ).slice(0, 180);
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
      className: String(element.className || "").slice(0, 180)
    };

    if (tag === "input" && (type === "checkbox" || type === "radio")) {
      data.checked = !!element.checked;
      data.value = String(element.value || "");
    } else if (tag === "select") {
      data.value = String(element.value || "");
      data.selectedIndex = element.selectedIndex;
      data.selectedText = element.options && element.selectedIndex >= 0
        ? String(element.options[element.selectedIndex].textContent || "").slice(0, 160)
        : "";
    } else {
      data.value = String(element.value || "").slice(0, 220);
    }

    return data;
  }

  function listActionButtons(root, network) {
    const buttons = Array.from((root || document).querySelectorAll("button")).filter(isActionButton);

    return buttons.map(function (button) {
      return {
        text: String(button.textContent || "").trim().slice(0, 90),
        action: String(
          (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
          button.getAttribute("data-action") ||
          button.getAttribute("aria-label") ||
          ""
        ).slice(0, 90),
        network: guessNetwork(button),
        disabled: !!button.disabled,
        ariaDisabled: button.getAttribute("aria-disabled") || "",
        kgwR12eDisabled: button.dataset ? button.dataset.kgwR12eDisabled || "" : "",
        className: String(button.className || "").slice(0, 160)
      };
    }).filter(function (button) {
      return network === "unknown" || button.network === network || button.network === "unknown";
    });
  }

  function collectSnapshot(root, network) {
    const controls = Array.from((root || document).querySelectorAll("input, select, textarea")).filter(isSettingsControl);

    const fields = controls.map(readControl).filter(function (item) {
      return network === "unknown" || item.network === network || item.network === "unknown";
    });

    const compact = fields.map(function (item) {
      return {
        key: item.key,
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
      fields: fields.slice(0, 80)
    };
  }

  function traceControlEvent(kind, event) {
    const target = event && event.target;
    if (!isSettingsControl(target)) return;

    const root = findTabRoot();
    const network = guessNetwork(target);
    const beforeField = readControl(target);
    const beforeSnapshot = collectSnapshot(root, network);

    const beforePhase = kind === "input" ? "setting-input-before" : "setting-change-before";
    const afterPhase = kind === "input" ? "setting-input-after" : "setting-change-after";
    const delayedPhase = kind === "input" ? "setting-input-after-250ms" : "setting-change-after-250ms";

    invokeTrace(beforePhase, {
      isTrusted: !!event.isTrusted,
      network,
      field: beforeField,
      snapshotHash: beforeSnapshot.hash,
      fieldCount: beforeSnapshot.fieldCount,
      actionButtons: listActionButtons(root, network)
    });

    setTimeout(function () {
      const afterField = readControl(target);
      const afterSnapshot = collectSnapshot(root, network);

      invokeTrace(afterPhase, {
        isTrusted: !!event.isTrusted,
        network,
        changed: safeJson(beforeField) !== safeJson(afterField) || beforeSnapshot.hash !== afterSnapshot.hash,
        beforeField,
        afterField,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        fieldCount: afterSnapshot.fieldCount,
        actionButtons: listActionButtons(root, network)
      });
    }, 0);

    setTimeout(function () {
      const delayedSnapshot = collectSnapshot(root, network);

      invokeTrace(delayedPhase, {
        isTrusted: !!event.isTrusted,
        network,
        beforeSnapshotHash: beforeSnapshot.hash,
        delayedSnapshotHash: delayedSnapshot.hash,
        changed: beforeSnapshot.hash !== delayedSnapshot.hash,
        fieldCount: delayedSnapshot.fieldCount,
        actionButtons: listActionButtons(root, network)
      });
    }, 250);
  }

  function traceActionClick(event) {
    const button = event && event.target && event.target.closest ? event.target.closest("button") : null;
    if (!isActionButton(button)) return;

    const root = findTabRoot();
    const network = guessNetwork(button);
    const beforeSnapshot = collectSnapshot(root, network);

    invokeTrace("action-click-before", {
      isTrusted: !!event.isTrusted,
      network,
      button: {
        text: String(button.textContent || "").trim().slice(0, 100),
        action: String(
          (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
          button.getAttribute("data-action") ||
          button.getAttribute("aria-label") ||
          ""
        ).slice(0, 100),
        disabled: !!button.disabled,
        ariaDisabled: button.getAttribute("aria-disabled") || "",
        kgwR12eDisabled: button.dataset ? button.dataset.kgwR12eDisabled || "" : "",
        className: String(button.className || "").slice(0, 180)
      },
      snapshotHash: beforeSnapshot.hash,
      fieldCount: beforeSnapshot.fieldCount,
      actionButtons: listActionButtons(root, network)
    });

    setTimeout(function () {
      const afterSnapshot = collectSnapshot(root, network);
      invokeTrace("action-click-after-250ms", {
        isTrusted: !!event.isTrusted,
        network,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        changed: beforeSnapshot.hash !== afterSnapshot.hash,
        fieldCount: afterSnapshot.fieldCount,
        actionButtons: listActionButtons(root, network)
      });
    }, 250);

    setTimeout(function () {
      const afterSnapshot = collectSnapshot(root, network);
      invokeTrace("action-click-after-1200ms", {
        isTrusted: !!event.isTrusted,
        network,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        changed: beforeSnapshot.hash !== afterSnapshot.hash,
        fieldCount: afterSnapshot.fieldCount,
        actionButtons: listActionButtons(root, network)
      });
    }, 1200);
  }

  function install() {
    const key = "__kgwSettingsSelectionTraceR12H2Installed_" + SCOPE;
    if (window[key]) return;
    window[key] = true;

    document.addEventListener("input", function (event) {
      traceControlEvent("input", event);
    }, true);

    document.addEventListener("change", function (event) {
      traceControlEvent("change", event);
    }, true);

    document.addEventListener("click", function (event) {
      traceActionClick(event);
    }, true);

    setTimeout(function () {
      const root = findTabRoot();
      const snapshot = collectSnapshot(root, "unknown");
      invokeTrace("installed", {
        snapshotHash: snapshot.hash,
        fieldCount: snapshot.fieldCount,
        actionButtons: listActionButtons(root, "unknown")
      });
    }, 0);
  }

  install();
})();
// END_KGW_SETTINGS_SELECTION_TRACE_R12H2
`;
}

function patchFile(file, scopeName) {
  const before = read(file);
  const text = removeExistingTraceBlocks(before).trimEnd() + "\n\n" + buildTraceBlock(scopeName) + "\n";
  write(file, text);

  return {
    file,
    scopeName,
    changed: before !== text,
    hasTrace: text.includes("KGW_SETTINGS_SELECTION_TRACE_R12H2"),
    hasInputBefore: text.includes("setting-input-before"),
    hasInputAfter: text.includes("setting-input-after"),
    hasChangeBefore: text.includes("setting-change-before"),
    hasChangeAfter: text.includes("setting-change-after"),
    hasActionBefore: text.includes("action-click-before"),
    hasInvoke: text.includes("kgw_frontend_button_trace_v1")
  };
}

function validateFile(file, label) {
  const text = read(file);
  const required = [
    "KGW_SETTINGS_SELECTION_TRACE_R12H2",
    "setting-input-before",
    "setting-input-after",
    "setting-input-after-250ms",
    "setting-change-before",
    "setting-change-after",
    "setting-change-after-250ms",
    "action-click-before",
    "action-click-after-250ms",
    "action-click-after-1200ms",
    "kgw_frontend_button_trace_v1"
  ];

  const errors = [];

  for (const item of required) {
    if (!text.includes(item)) {
      errors.push(label + ": missing " + item);
    }
  }

  return errors;
}

function main() {
  const beforeAudit = {
    nodeHadR12H: read(files.nodeJs).includes("KGW_SETTINGS_SELECTION_TRACE_R12H"),
    bridgeHadR12H: read(files.bridgeJs).includes("KGW_SETTINGS_SELECTION_TRACE_R12H"),
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

  writeJson("audit-before-r12h2.json", beforeAudit);
  writeJson("audit-after-r12h2.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R12H2 validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R12H2 setting-selection trace installed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
