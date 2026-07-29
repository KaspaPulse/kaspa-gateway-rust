import assert from "node:assert/strict";
import path from "node:path";
import { parseKeyValueLine, pidFromStatus } from "./assertions.mjs";
import { waitUntil } from "./windows.mjs";
import { writeJson, writeText } from "./paths.mjs";

export async function clickTestId(testId) {
  const element = await $(`[data-testid="${testId}"]`);
  await element.waitForDisplayed({ timeout: 30000 });
  await element.scrollIntoView();
  await element.click();
  return element;
}

export async function readByTestId(testId) {
  const element = await $(`[data-testid="${testId}"]`);
  await element.waitForExist({ timeout: 30000 });
  return await browser.execute((selector) => {
    const node = document.querySelector(selector);
    if (!node) return "";
    const tag = String(node.tagName || "").toUpperCase();
    return tag === "TEXTAREA" || tag === "INPUT"
      ? String(node.value || "")
      : String(node.textContent || "");
  }, `[data-testid="${testId}"]`);
}

export async function waitForRawLogOutput(testId, timeoutMs = 60000) {
  return await waitUntil(`raw log output ${testId}`, timeoutMs, 500, async () => {
    const text = await readByTestId(testId);
    const trimmed = String(text || "").trim();
    if (!trimmed) return false;
    if (/waiting for raw/i.test(trimmed) || /no raw/i.test(trimmed) || /placeholder/i.test(trimmed)) return false;
    return text;
  });
}

export async function openNodeSettings(network) {
  await clickTestId("kgw-tab-kaspa-node");
  await clickTestId(`kgw-node-network-${network}`);
  await clickTestId(`kgw-node-settings-${network}`);
}

export async function openNodeLiveMonitor(network) {
  await clickTestId("kgw-tab-kaspa-node");
  await clickTestId(`kgw-node-network-${network}`);
  await clickTestId(`kgw-node-live-monitor-${network}`);
}

export async function openBridgeSettings(network) {
  await clickTestId("kgw-tab-kaspa-bridge");
  await clickTestId(`kgw-bridge-network-${network}`);
  await clickTestId(`kgw-bridge-settings-${network}`);
}

export async function openBridgeLiveMonitor(network) {
  await clickTestId("kgw-tab-kaspa-bridge");
  await clickTestId(`kgw-bridge-network-${network}`);
  await clickTestId(`kgw-bridge-live-monitor-${network}`);
}

export async function readBridgeRuntimeSelection(network) {
  return await browser.execute((net) => {
    const panel = document.querySelector(`[data-testid="kgw-bridge-panel-${net}"]`);
    const active = panel?.querySelector?.('[data-bridge-action="select-instance"].active') ||
      panel?.querySelector?.('[data-bridge-action="select-instance"]');
    const bridgeInstanceId = String(active?.dataset?.instanceId || "1");
    const input = panel?.querySelector?.(`[data-testid="kgw-bridge-instance-field-${net}-${bridgeInstanceId}-instancePort"]`);
    const bridgeLevel = panel?.querySelector?.(`[data-testid="kgw-bridge-field-${net}-stratumPort"]`);
    const rawPort = String(input?.value || input?.placeholder || bridgeLevel?.value || "").trim().replace(/^:/, "");
    const port = Number(rawPort);
    return {
      bridgeInstanceId,
      bridgePort: Number.isInteger(port) && port > 0 ? port : null,
      rawPort,
      bridgeLevelPort: String(bridgeLevel?.value || ""),
    };
  }, network);
}

export async function invoke(command, args = {}, timeoutMs = 60000) {
  await browser.setTimeout({ script: timeoutMs + 5000 });
  const result = await browser.executeAsync((commandName, payload, done) => {
    try {
      const tauriInvoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
      if (typeof tauriInvoke !== "function") {
        done({ ok: false, error: "Tauri invoke API is unavailable" });
        return;
      }

      Promise.resolve(tauriInvoke(commandName, payload))
        .then((value) => done({ ok: true, value }))
        .catch((error) => {
          done({
            ok: false,
            error: error && error.message ? error.message : String(error),
          });
        });
    } catch (error) {
      done({
        ok: false,
        error: error && error.message ? error.message : String(error),
      });
    }
  }, command, args);

  if (!result?.ok) {
    throw new Error(`IPC ${command} failed: ${result?.error || "unknown error"}`);
  }

  return result.value;
}

export async function installClipboardObserver() {
  await browser.execute(() => {
    if (window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1?.installed) return;

    const tauriCore = window.__TAURI__?.core || window.__TAURI__;
    if (!tauriCore || typeof tauriCore.invoke !== "function") {
      window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1 = {
        installed: false,
        error: "Tauri invoke API is unavailable",
        events: [],
        nextSequence: 1,
      };
      return;
    }

    const originalInvoke = tauriCore.invoke.bind(tauriCore);
    const state = {
      installed: true,
      events: [],
      nextSequence: 1,
      originalInvoke,
    };

    function parseFields(text) {
      const fields = {};
      String(text || "").split(";").forEach((part) => {
        const index = part.indexOf("=");
        if (index > 0) fields[part.slice(0, index).trim()] = part.slice(index + 1).trim();
      });
      return fields;
    }

    tauriCore.invoke = function kgwZeroTouchObservedInvoke(command, args) {
      const startedAt = Date.now();
      const promise = originalInvoke(command, args);
      if (command !== "kgw_copy_text_to_clipboard_v1") return promise;

      return Promise.resolve(promise)
        .then((value) => {
          const fields = parseFields(value);
          state.events.push({
            sequence: state.nextSequence++,
            status: "ok",
            command,
            network: String(args?.network || ""),
            runtimeRole: String(args?.runtimeRole || ""),
            bridgeInstanceId: String(args?.bridgeInstanceId || ""),
            expectedSha256: String(args?.sha256 || ""),
            resultSha256: String(fields.sha256 || ""),
            result: String(value || ""),
            characterCount: Number(args?.characterCount || 0),
            lineCount: Number(args?.lineCount || 0),
            startedAt,
            completedAt: Date.now(),
          });
          return value;
        })
        .catch((error) => {
          state.events.push({
            sequence: state.nextSequence++,
            status: "error",
            command,
            network: String(args?.network || ""),
            runtimeRole: String(args?.runtimeRole || ""),
            bridgeInstanceId: String(args?.bridgeInstanceId || ""),
            expectedSha256: String(args?.sha256 || ""),
            error: error && error.message ? error.message : String(error),
            characterCount: Number(args?.characterCount || 0),
            lineCount: Number(args?.lineCount || 0),
            startedAt,
            completedAt: Date.now(),
          });
          throw error;
        });
    };

    window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1 = state;
  });

  const installed = await browser.execute(() => Boolean(window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1?.installed));
  assert.equal(installed, true, "clipboard observer was not installed");
}

export async function clearClipboardEvents() {
  await browser.execute(() => {
    if (window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1) {
      window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1.events = [];
    }
  });
}

export async function waitForClipboardEvent({ network, runtimeRole, bridgeInstanceId = "", afterSequence = 0, timeoutMs = 20000 }) {
  return await waitUntil(`native clipboard event ${runtimeRole}/${network}`, timeoutMs, 250, async () => {
    const event = await browser.execute((wanted) => {
      const events = window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1?.events || [];
      return events.find((item) => {
        if (Number(item.sequence || 0) <= Number(wanted.afterSequence || 0)) return false;
        if (item.status !== "ok") return false;
        if (String(item.network || "").toLowerCase() !== wanted.network) return false;
        if (String(item.runtimeRole || "").toLowerCase() !== wanted.runtimeRole) return false;
        if (wanted.bridgeInstanceId && String(item.bridgeInstanceId || "") !== wanted.bridgeInstanceId) return false;
        return true;
      }) || null;
    }, {
      network,
      runtimeRole,
      bridgeInstanceId,
      afterSequence,
    });

    return event || false;
  });
}

export async function clipboardEventCount() {
  return await browser.execute(() => Number(window.__KGW_ZERO_TOUCH_CLIPBOARD_OBSERVER_V1?.events?.length || 0));
}

export async function waitForOwnerStatus({ network, runtimeRole, timeoutMs = 120000 }) {
  return await waitUntil(`owner status ${runtimeRole}/${network}`, timeoutMs, 750, async () => {
    const status = String(await invoke("kgw_runtime_owner_status_v1", { network, runtimeRole }, 30000));
    const pid = pidFromStatus(status);
    const fields = parseKeyValueLine(status);
    if (!pid) return false;
    if (String(fields.network || "").toLowerCase() && String(fields.network || "").toLowerCase() !== network) return false;
    if (String(fields.role || fields.runtime_role || fields.runtimeRole || "").toLowerCase() && String(fields.role || fields.runtime_role || fields.runtimeRole || "").toLowerCase() !== runtimeRole) return false;
    return { status, pid, fields };
  });
}

export async function waitForStopped({ network, runtimeRole, timeoutMs = 30000 }) {
  return await waitUntil(`stopped status ${runtimeRole}/${network}`, timeoutMs, 500, async () => {
    const status = String(await invoke("kgw_runtime_owner_status_v1", { network, runtimeRole }, 30000));
    if (!pidFromStatus(status) && /running=false/i.test(status)) return { status };
    if (/no .*worker status yet|stopped|running=false/i.test(status) && !/pid=\d+/i.test(status)) return { status };
    return false;
  });
}

export async function waitForRuntimeLogs({ network, runtimeRole, bridgeInstanceId = "", minEntries = 1, timeoutMs = 90000 }) {
  return await waitUntil(`runtime logs ${runtimeRole}/${network}`, timeoutMs, 750, async () => {
    const report = await invoke("kgw_kgw_runtime_logs_v1", {
      network,
      runtimeRole,
      bridgeInstanceId,
    }, 30000);
    const entries = Array.isArray(report?.entries) ? report.entries : [];
    if (entries.length < minEntries) return false;
    return report;
  });
}

export function rawTextFromReport(report) {
  const entries = Array.isArray(report?.entries) ? [...report.entries] : [];
  entries.sort((left, right) => Number(left.sequence || 0) - Number(right.sequence || 0));
  return entries.map((entry) => String(entry.rawText ?? entry.raw_text ?? "")).join("\n");
}

export async function saveDomState(outputDirectory, label) {
  const state = await browser.execute(() => {
    const read = (selector) => {
      const node = document.querySelector(selector);
      if (!node) return null;
      return {
        selector,
        hidden: Boolean(node.hidden),
        disabled: Boolean(node.disabled),
        text: String(node.textContent || "").slice(0, 4000),
        value: String(node.value || "").slice(0, 4000),
      };
    };
    return {
      title: document.title,
      htmlDir: document.documentElement?.dir || "",
      language: document.documentElement?.lang || "",
      activeTab: document.querySelector("[data-tab].active")?.getAttribute("data-tab") || "",
      node: ["mainnet", "testnet10", "testnet12"].map((net) => read(`[data-testid="kgw-node-panel-${net}"]`)),
      bridge: ["mainnet", "testnet10", "testnet12"].map((net) => read(`[data-testid="kgw-bridge-panel-${net}"]`)),
    };
  });
  await writeJson(path.join(outputDirectory, `${label}-dom-state.json`), state);
  await writeText(path.join(outputDirectory, `${label}-page-source.html`), await browser.getPageSource());
  await browser.saveScreenshot(path.join(outputDirectory, `${label}-screenshot.png`));
  return state;
}

export async function stopRuntime(network, runtimeRole) {
  try {
    if (runtimeRole === "bridge") {
      await openBridgeSettings(network);
      const stop = await $(`[data-testid="kgw-bridge-stop-${network}"]`);
      if (await stop.isExisting() && await stop.isEnabled()) await stop.click();
    } else {
      await openNodeSettings(network);
      const stop = await $(`[data-testid="kgw-node-stop-${network}"]`);
      if (await stop.isExisting() && await stop.isEnabled()) await stop.click();
    }
  } catch (_) {
    // IPC fallback below preserves zero-touch cleanup when the UI is already torn down.
  }

  try {
    await invoke("kgw_kgw_disable_network_v1", { network, runtimeRole }, 30000);
  } catch (_) {
    // Best-effort cleanup continues with shutdown-all.
  }
}

export async function shutdownAllRuntimeWorkers() {
  try {
    return await invoke("kgw_shutdown_all_runtime_workers_v1", {}, 30000);
  } catch (error) {
    return String(error?.message || error);
  }
}
