const fs = require("fs");
const path = require("path");

const repoRoot = process.argv[2];
const reportDir = process.argv[3];

if (!repoRoot || !reportDir) {
  throw new Error("Usage: node kgw_settings_selection_trace_r12h.cjs <repoRoot> <reportDir>");
}

const files = {
  nodeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js"),
  bridgeJs: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js"),
  nodeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.css"),
  bridgeCss: path.join(repoRoot, "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.css")
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

function removeExistingTraceBlock(text) {
  return text.replace(/\/\/\s*KGW_SETTINGS_SELECTION_TRACE_R12H[\s\S]*?\/\/\s*END_KGW_SETTINGS_SELECTION_TRACE_R12H\s*/g, "");
}

function buildTraceBlock(scopeName) {
  return `
// KGW_SETTINGS_SELECTION_TRACE_R12H
(function installKgwSettingsSelectionTraceR12H() {
  const TRACE_OWNER = "KGW_SETTINGS_SELECTION_TRACE_R12H";
  const SCOPE = "${scopeName}";
  const EVENT_LIMIT_PER_SECOND = 80;
  let eventWindowStartedAt = Date.now();
  let eventWindowCount = 0;

  function nowIso() {
    try {
      return new Date().toISOString();
    } catch (_) {
      return "";
    }
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

  function stableStringify(value) {
    try {
      return JSON.stringify(value, Object.keys(value).sort());
    } catch (_) {
      try {
        return JSON.stringify(value);
      } catch (_) {
        return "";
      }
    }
  }

  function hashText(text) {
    let hash = 0;
    const input = String(text || "");
    for (let i = 0; i < input.length; i += 1) {
      hash = ((hash << 5) - hash + input.charCodeAt(i)) | 0;
    }
    return String(hash);
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
        console.debug("[KGW_SETTINGS_SELECTION_TRACE_R12H]", payload);
      }
    } catch (_) {}
  }

  function lower(value) {
    return String(value || "").toLowerCase();
  }

  function isSettingsControl(element) {
    if (!element || !element.tagName) return false;

    const tag = lower(element.tagName);
    if (tag !== "input" && tag !== "select" && tag !== "textarea") return false;

    const type = lower(element.type);
    if (type === "button" || type === "submit" || type === "reset") return false;

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
      text.includes("حفظ") ||
      text.includes("استعادة") ||
      text.includes("افتراض")
    );
  }

  function findTabRoot() {
    const candidates = Array.from(document.querySelectorAll("[data-tab], [id], .tab-pane, main, body"));
    const scopeNeedle = SCOPE === "node" ? "node" : "bridge";

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

    const name = lower(element && (element.name || element.id || ""));
    if (name.includes("testnet12") || name.includes("tn12")) return "testnet12";
    if (name.includes("testnet10") || name.includes("tn10")) return "testnet10";
    if (name.includes("mainnet")) return "mainnet";

    return "unknown";
  }

  function fieldKey(element) {
    if (!element) return "";

    const dataset = element.dataset || {};
    return String(
      dataset.kgwSettingKey ||
      dataset.settingKey ||
      dataset.key ||
      element.name ||
      element.id ||
      element.getAttribute("aria-label") ||
      element.getAttribute("placeholder") ||
      element.className ||
      element.tagName ||
      ""
    ).slice(0, 180);
  }

  function readValue(element) {
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
      classes: String(element.className || "").slice(0, 220)
    };

    if (tag === "input" && (type === "checkbox" || type === "radio")) {
      data.checked = !!element.checked;
      data.value = String(element.value || "");
    } else if (tag === "select") {
      data.value = String(element.value || "");
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
        text: String(button.textContent || "").trim().slice(0, 80),
        action: String(
          (button.dataset && (button.dataset.kgwSettingsAction || button.dataset.action)) ||
          button.getAttribute("data-action") ||
          button.getAttribute("aria-label") ||
          ""
        ).slice(0, 80),
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

  function collectSettingsSnapshot(root, network) {
    const controls = Array.from((root || document).querySelectorAll("input, select, textarea")).filter(isSettingsControl);

    const fields = controls.map(function (element) {
      return readValue(element);
    }).filter(function (item) {
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

  function traceControlEvent(eventName, event) {
    const target = event && event.target;
    if (!isSettingsControl(target)) return;

    const root = findTabRoot();
    const network = guessNetwork(target);
    const beforeValue = readValue(target);
    const beforeSnapshot = collectSettingsSnapshot(root, network);

    invokeTrace("setting-" + eventName + "-before", {
      isTrusted: !!event.isTrusted,
      network,
      field: beforeValue,
      snapshotHash: beforeSnapshot.hash,
      fieldCount: beforeSnapshot.fieldCount,
      actionButtons: listActionButtons(root, network)
    });

    setTimeout(function () {
      const afterValue = readValue(target);
      const afterSnapshot = collectSettingsSnapshot(root, network);

      invokeTrace("setting-" + eventName + "-after", {
        isTrusted: !!event.isTrusted,
        network,
        field: afterValue,
        changed:
          stableStringify(beforeValue) !== stableStringify(afterValue) ||
          beforeSnapshot.hash !== afterSnapshot.hash,
        beforeSnapshotHash: beforeSnapshot.hash,
        afterSnapshotHash: afterSnapshot.hash,
        beforeField: beforeValue,
        afterField: afterValue,
        actionButtons: listActionButtons(root, network)
      });
    }, 0);

    setTimeout(function () {
      const delayedSnapshot = collectSettingsSnapshot(root, network);

      invokeTrace("setting-" + eventName + "-after-250ms", {
        isTrusted: !!event.isTrusted,
        network,
        snapshotHash: delayedSnapshot.hash,
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
    const beforeSnapshot = collectSettingsSnapshot(root, network);

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
      const afterSnapshot = collectSettingsSnapshot(root, network);

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
      const afterSnapshot = collectSettingsSnapshot(root, network);

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
    if (window["__kgwSettingsSelectionTraceR12HInstalled_" + SCOPE]) return;
    window["__kgwSettingsSelectionTraceR12HInstalled_" + SCOPE] = true;

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
      const snapshot = collectSettingsSnapshot(root, "unknown");
      invokeTrace("installed", {
        snapshotHash: snapshot.hash,
        fieldCount: snapshot.fieldCount,
        actionButtons: listActionButtons(root, "unknown")
      });
    }, 0);
  }

  install();
})();
// END_KGW_SETTINGS_SELECTION_TRACE_R12H
`;
}

function patchFile(file, scopeName) {
  const before = read(file);
  let text = removeExistingTraceBlock(before);

  const block = buildTraceBlock(scopeName);
  text = text.trimEnd() + "\n\n" + block + "\n";

  write(file, text);

  return {
    file,
    scopeName,
    changed: before !== text,
    hasTrace: text.includes("KGW_SETTINGS_SELECTION_TRACE_R12H"),
    hasInvoke: text.includes("kgw_frontend_button_trace_v1"),
    hasInputTrace: text.includes("setting-input-before"),
    hasChangeTrace: text.includes("setting-change-before"),
    hasActionTrace: text.includes("action-click-before")
  };
}

function validatePatchedFile(file, label) {
  const text = read(file);
  const errors = [];

  if (!text.includes("KGW_SETTINGS_SELECTION_TRACE_R12H")) {
    errors.push(label + ": missing KGW_SETTINGS_SELECTION_TRACE_R12H");
  }

  if (!text.includes("setting-input-before")) {
    errors.push(label + ": missing setting input trace");
  }

  if (!text.includes("setting-change-before")) {
    errors.push(label + ": missing setting change trace");
  }

  if (!text.includes("action-click-before")) {
    errors.push(label + ": missing action click trace");
  }

  if (!text.includes("kgw_frontend_button_trace_v1")) {
    errors.push(label + ": missing Tauri trace invoke");
  }

  return errors;
}

function main() {
  const beforeAudit = {
    nodeHadTrace: read(files.nodeJs).includes("KGW_SETTINGS_SELECTION_TRACE_R12H"),
    bridgeHadTrace: read(files.bridgeJs).includes("KGW_SETTINGS_SELECTION_TRACE_R12H"),
    nodeHasR12E: read(files.nodeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: read(files.bridgeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    nodeHasNativeDisabled: /button\\.disabled\\s*=\\s*!!disabled/.test(read(files.nodeJs)),
    bridgeHasNativeDisabled: /button\\.disabled\\s*=\\s*!!disabled/.test(read(files.bridgeJs))
  };

  const nodeResult = patchFile(files.nodeJs, "node");
  const bridgeResult = patchFile(files.bridgeJs, "bridge");

  const validationErrors = [
    ...validatePatchedFile(files.nodeJs, "Node"),
    ...validatePatchedFile(files.bridgeJs, "Bridge")
  ];

  const afterAudit = {
    validationErrors,
    nodeResult,
    bridgeResult,
    nodeHasR12E: read(files.nodeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    bridgeHasR12E: read(files.bridgeJs).includes("KGW_SETTINGS_UNIFIED_OWNER_R12E"),
    nodeHasNativeDisabled: /button\\.disabled\\s*=\\s*!!disabled/.test(read(files.nodeJs)),
    bridgeHasNativeDisabled: /button\\.disabled\\s*=\\s*!!disabled/.test(read(files.bridgeJs))
  };

  writeJson("audit-before-r12h.json", beforeAudit);
  writeJson("audit-after-r12h.json", afterAudit);

  if (validationErrors.length) {
    throw new Error("R12H validation failed:\n- " + validationErrors.join("\n- "));
  }

  console.log("# R12H setting-selection trace installed");
  console.log(JSON.stringify(afterAudit, null, 2));
}

main();
