import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  cleanupWdioSession,
  createTauriCapabilities,
  startWdioSession,
} from "@wdio/tauri-service";
import { parseKeyValueLine, pidFromStatus } from "./assertions.mjs";
import { runtimePortProfile } from "./runtime-ports.mjs";
import {
  waitForExactProcessExit,
  waitForPort,
  waitForPortFree,
  waitUntil,
} from "./windows.mjs";
import { ensureDir, writeJson } from "./paths.mjs";

const helperDir = path.dirname(fileURLToPath(import.meta.url));
const e2eDir = path.resolve(helperDir, "..");
const repository = process.env.KGW_REPOSITORY || path.resolve(e2eDir, "..");
const artifactRoot = process.env.KGW_ZERO_TOUCH_ARTIFACT_DIR ||
  path.join(repository, "artifacts", "close-relaunch-e2e", `run-${Date.now()}`);
const appBinaryPath = process.env.KGW_E2E_APP_BINARY ||
  path.join(repository, "target", "kgw-zero-touch-e2e", "debug", "kaspa-gateway-desktop.exe");
const ports = runtimePortProfile();
const networks = ["mainnet", "testnet10"];
const embeddedBasePort = Number(process.env.KGW_CLOSE_RELAUNCH_EMBEDDED_PORT || "4455");

function sessionCapabilities(embeddedPort, logDir) {
  const capabilities = createTauriCapabilities(appBinaryPath, {
    driverProvider: "embedded",
    commandTimeout: 60000,
    startTimeout: 120000,
    logLevel: process.env.WDIO_LOG_LEVEL || "info",
  });
  capabilities["wdio:tauriServiceOptions"] = {
    ...(capabilities["wdio:tauriServiceOptions"] || {}),
    embeddedPort,
    captureBackendLogs: true,
    captureFrontendLogs: true,
    backendLogLevel: "trace",
    frontendLogLevel: "debug",
    logDir,
    statusPollTimeout: 10000,
  };
  return capabilities;
}

async function newSession(label, embeddedPort) {
  const logDir = await ensureDir(path.join(artifactRoot, label, "tauri-service"));
  return await startWdioSession(sessionCapabilities(embeddedPort, logDir), { rootDir: e2eDir });
}
async function clickTestId(browser, testId) {
  const element = await browser.$(`[data-testid="${testId}"]`);
  await element.waitForDisplayed({ timeout: 30000 });
  await element.scrollIntoView();
  await element.click();
}

async function setControlValue(browser, testId, value) {
  const result = await browser.execute((selector, nextValue) => {
    const node = document.querySelector(selector);
    if (!node) return { ok: false, reason: "missing" };
    if (node.disabled || node.readOnly) return { ok: false, reason: "not-editable" };
    node.value = String(nextValue);
    node.dispatchEvent(new Event("input", { bubbles: true }));
    node.dispatchEvent(new Event("change", { bubbles: true }));
    return { ok: true, value: String(node.value || "") };
  }, `[data-testid="${testId}"]`, String(value));
  assert.equal(result?.ok, true, `Unable to set ${testId}: ${result?.reason || "unknown"}`);
}

async function setControlChecked(browser, testId, checked) {
  const result = await browser.execute((selector, nextChecked) => {
    const node = document.querySelector(selector);
    if (!node) return { ok: false, reason: "missing" };
    if (node.disabled || node.readOnly) return { ok: false, reason: "not-editable" };
    node.checked = Boolean(nextChecked);
    node.dispatchEvent(new Event("input", { bubbles: true }));
    node.dispatchEvent(new Event("change", { bubbles: true }));
    return { ok: true, checked: Boolean(node.checked) };
  }, `[data-testid="${testId}"]`, Boolean(checked));
  assert.equal(result?.ok, true, `Unable to set ${testId}: ${result?.reason || "unknown"}`);
  assert.equal(result.checked, Boolean(checked), `Checkbox ${testId} did not retain requested state`);
}

async function invoke(browser, command, args = {}, timeoutMs = 60000) {
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
        .catch((error) => done({ ok: false, error: error?.message || String(error) }));
    } catch (error) {
      done({ ok: false, error: error?.message || String(error) });
    }
  }, command, args);
  if (!result?.ok) {
    throw new Error(`IPC ${command} failed: ${result?.error || "unknown error"}`);
  }
  return result.value;
}

async function ownerStatus(browser, network) {
  const status = String(await invoke(browser, "kgw_runtime_owner_status_v1", {
    network,
    runtimeRole: "node",
  }, 30000));
  return { status, pid: pidFromStatus(status), fields: parseKeyValueLine(status) };
}

async function waitForOwner(browser, network, timeoutMs = 180000) {
  return await waitUntil(`owner status node/${network}`, timeoutMs, 750, async () => {
    const status = await ownerStatus(browser, network);
    if (!status.pid) return false;
    const fields = status.fields || {};
    if (String(fields.network || "").toLowerCase() !== network) return false;
    if (String(fields.role || "").toLowerCase() !== "node") return false;
    if (String(fields.readiness || "").toUpperCase() !== "READY") return false;
    return status;
  });
}
async function openNodeSettings(browser, network) {
  await clickTestId(browser, "kgw-tab-kaspa-node");
  await clickTestId(browser, `kgw-node-network-${network}`);
  await clickTestId(browser, `kgw-node-settings-${network}`);
}

async function configureAndStartNode(browser, network, label) {
  const profile = ports[network];
  assert.ok(profile, `missing runtime port profile for ${network}`);
  await openNodeSettings(browser, network);
  await setControlValue(browser, `kgw-node-field-${network}-rpcListenHost`, "127.0.0.1");
  await setControlValue(browser, `kgw-node-field-${network}-rpcListenPort`, String(profile.rpcPort));
  await setControlChecked(browser, `kgw-node-field-${network}-listenEnabled`, true);
  await setControlValue(browser, `kgw-node-field-${network}-listenHost`, "127.0.0.1");
  await setControlValue(browser, `kgw-node-field-${network}-listenPort`, String(profile.p2pPort));
  await clickTestId(browser, `kgw-node-start-${network}`);

  const status = await waitForOwner(browser, network);
  await waitForPort("127.0.0.1", profile.rpcPort, 180000);
  await waitForPort("127.0.0.1", profile.p2pPort, 180000);
  await writeJson(path.join(artifactRoot, label, `${network}-owner-status.json`), status);
  return status;
}
async function waitForNodeUi(browser, network, expectedRunning) {
  return await waitUntil(`node UI ${network} running=${expectedRunning}`, 45000, 500, async () => {
    await openNodeSettings(browser, network);
    const state = await browser.execute((net) => {
      const start = document.querySelector(`[data-testid="kgw-node-start-${net}"]`);
      const stop = document.querySelector(`[data-testid="kgw-node-stop-${net}"]`);
      return {
        startExists: Boolean(start),
        stopExists: Boolean(stop),
        startDisabled: Boolean(start?.disabled || start?.getAttribute("aria-disabled") === "true"),
        stopDisabled: Boolean(stop?.disabled || stop?.getAttribute("aria-disabled") === "true"),
      };
    }, network);
    if (!state.startExists || !state.stopExists) return false;
    if (expectedRunning && state.startDisabled && !state.stopDisabled) return state;
    if (!expectedRunning && !state.startDisabled && state.stopDisabled) return state;
    return false;
  });
}

async function waitForNodeStopped(browser, network) {
  return await waitUntil(`stopped owner status node/${network}`, 60000, 500, async () => {
    const status = await ownerStatus(browser, network);
    if (!status.pid && /running=false|stopped|no .*worker status yet/i.test(status.status)) return status;
    return false;
  });
}
async function stopNode(browser, network, label) {
  const profile = ports[network];
  await openNodeSettings(browser, network);
  const stop = await browser.$(`[data-testid="kgw-node-stop-${network}"]`);
  if (await stop.isExisting() && await stop.isEnabled()) await stop.click();
  try {
    await invoke(browser, "kgw_kgw_disable_network_v1", { network, runtimeRole: "node" }, 30000);
  } catch (_) {
    // UI stop may already have completed the exact same operation.
  }
  const stopped = await waitForNodeStopped(browser, network);
  await waitForPortFree("127.0.0.1", profile.rpcPort, 60000);
  await waitForPortFree("127.0.0.1", profile.p2pPort, 60000);
  const ui = await waitForNodeUi(browser, network, false);
  await writeJson(path.join(artifactRoot, label, `${network}-stopped.json`), { stopped, ui });
}

function exactParentIdentity(status) {
  const fields = status?.fields || {};
  const pid = Number(fields.parent_pid || 0);
  const startTime = Number(fields.parent_start_time || 0);
  const executable = String(fields.parent_executable || "").trim();
  assert.ok(pid > 0, "parent PID is missing");
  assert.ok(startTime > 0, "parent start time is missing");
  assert.ok(executable, "parent executable is missing");
  return { pid, startTime, executable };
}
async function main() {
  await fs.access(appBinaryPath);
  await ensureDir(artifactRoot);
  const result = {
    repository,
    appBinaryPath,
    artifactRoot,
    networks,
    passed: false,
    warnings: [],
    startedAt: new Date().toISOString(),
  };
  let firstBrowser = null;
  let secondBrowser = null;
  let firstParent;
  let secondParent;

  try {
    firstBrowser = await newSession("before-close", embeddedBasePort);
    const before = {};
    for (const network of networks) {
      before[network] = await configureAndStartNode(firstBrowser, network, "before-close");
      await waitForNodeUi(firstBrowser, network, true);
    }
    const mainnetParent = exactParentIdentity(before.mainnet);
    const testnetParent = exactParentIdentity(before.testnet10);
    assert.deepEqual(testnetParent, mainnetParent, "parallel node workers must share one exact desktop parent");
    firstParent = mainnetParent;
    await writeJson(path.join(artifactRoot, "before-close", "desktop-parent.json"), firstParent);
    let closeError = "";
    try {
      await firstBrowser.closeWindow();
    } catch (error) {
      closeError = error?.message || String(error);
    }
    await writeJson(path.join(artifactRoot, "before-close", "close-window.json"), {
      requested: true,
      closeError,
      requestedAt: new Date().toISOString(),
    });

    const parentExit = await waitForExactProcessExit({
      pid: firstParent.pid,
      expectedExecutable: firstParent.executable,
      expectedStartTime: firstParent.startTime,
      outputPath: path.join(artifactRoot, "before-close", "desktop-parent-exit.json"),
      timeoutSeconds: 60,
    });
    assert.equal(Boolean(parentExit?.exact_identity_exited), true, "desktop parent did not exit after CloseRequested");
    for (const network of networks) {
      await waitForPortFree("127.0.0.1", ports[network].rpcPort, 60000);
      await waitForPortFree("127.0.0.1", ports[network].p2pPort, 60000);
    }
    try {
      await cleanupWdioSession(firstBrowser);
    } catch (error) {
      result.warnings.push(`first standalone cleanup: ${error?.message || error}`);
    }
    firstBrowser = null;

    secondBrowser = await newSession("after-relaunch", embeddedBasePort + 1);
    const reconciled = {};
    for (const network of networks) {
      const stopped = await waitForNodeStopped(secondBrowser, network);
      const ui = await waitForNodeUi(secondBrowser, network, false);
      await waitForPortFree("127.0.0.1", ports[network].rpcPort, 60000);
      await waitForPortFree("127.0.0.1", ports[network].p2pPort, 60000);
      reconciled[network] = { stopped, ui };
    }
    await writeJson(path.join(artifactRoot, "after-relaunch", "reconciled.json"), reconciled);

    const recovered = {};
    for (const network of networks) {
      recovered[network] = await configureAndStartNode(secondBrowser, network, "after-relaunch");
      await waitForNodeUi(secondBrowser, network, true);
    }
    const recoveredMainnetParent = exactParentIdentity(recovered.mainnet);
    const recoveredTestnetParent = exactParentIdentity(recovered.testnet10);
    assert.deepEqual(
      recoveredTestnetParent,
      recoveredMainnetParent,
      "recovered parallel node workers must share one exact desktop parent",
    );
    secondParent = recoveredMainnetParent;
    assert.notDeepEqual(secondParent, firstParent, "relaunch must create a new exact desktop parent identity");
    await writeJson(path.join(artifactRoot, "after-relaunch", "desktop-parent.json"), secondParent);

    for (const network of [...networks].reverse()) {
      await stopNode(secondBrowser, network, "after-relaunch");
    }
    let finalCloseError = "";
    try {
      await secondBrowser.closeWindow();
    } catch (error) {
      finalCloseError = error?.message || String(error);
    }
    await writeJson(path.join(artifactRoot, "after-relaunch", "close-window.json"), {
      requested: true,
      closeError: finalCloseError,
      requestedAt: new Date().toISOString(),
    });
    await waitForExactProcessExit({
      pid: secondParent.pid,
      expectedExecutable: secondParent.executable,
      expectedStartTime: secondParent.startTime,
      outputPath: path.join(artifactRoot, "after-relaunch", "desktop-parent-exit.json"),
      timeoutSeconds: 60,
    });
    result.passed = true;
    result.finishedAt = new Date().toISOString();
    await writeJson(path.join(artifactRoot, "close-relaunch-result.json"), result);
  } catch (error) {
    result.error = error?.stack || error?.message || String(error);
    result.finishedAt = new Date().toISOString();
    await writeJson(path.join(artifactRoot, "close-relaunch-result.json"), result).catch(() => {});
    throw error;
  } finally {
    for (const browser of [secondBrowser, firstBrowser]) {
      if (!browser) continue;
      try {
        await invoke(browser, "kgw_shutdown_all_runtime_workers_v1", {}, 30000);
      } catch (_) {
        // Session may already be gone after a native close.
      }
      try {
        await cleanupWdioSession(browser);
      } catch (error) {
        result.warnings.push(`standalone cleanup: ${error?.message || error}`);
      }
    }
    await writeJson(path.join(artifactRoot, "close-relaunch-result.json"), result).catch(() => {});
  }
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exitCode = 1;
});
