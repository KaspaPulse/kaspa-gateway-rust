import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import {
  assertDirectRawPayload,
  assertRuntimeLogReport,
  normalizeClipboardText,
  parseKeyValueLine,
  sha256Hex,
} from "../helpers/assertions.mjs";
import {
  artifactRoot,
  caseDir,
  ensureDir,
  repository,
  writeJson,
  writeText,
} from "../helpers/paths.mjs";
import {
  captureWindowsEvidence,
  probeLocalStratum,
  waitUntil,
  waitForPort,
  waitForClipboardShaToFile,
  writeClipboardSentinel,
} from "../helpers/windows.mjs";
import {
  clickTestId,
  installClipboardObserver,
  invoke,
  openBridgeLiveMonitor,
  openBridgeSettings,
  openNodeLiveMonitor,
  openNodeSettings,
  readBridgeRuntimeSelection,
  rawTextFromReport,
  readByTestId,
  saveDomState,
  shutdownAllRuntimeWorkers,
  stopRuntime,
  waitForOwnerStatus,
  waitForRawLogOutput,
  waitForRuntimeLogs,
  waitForStopped,
} from "../helpers/tauri-app.mjs";

const observed = {
  startedAt: new Date().toISOString(),
  pids: [],
  ports: [],
  clipboardCaptures: [],
  cases: [],
  warnings: [],
};

const nodeCases = {
  mainnet: {
    slug: "mainnet-node",
    rpcPort: 16110,
    p2pPort: 16111,
    mustNot: [/testnet10/i, /testnet12/i, /nodes[\\/]+testnet10/i, /nodes[\\/]+testnet12/i],
  },
  testnet10: {
    slug: "testnet10-node",
    rpcPort: 16210,
    p2pPort: 16211,
    mustNot: [/nodes[\\/]+mainnet/i, /mainnet/i, /127\.0\.0\.1:16110/],
  },
};

const bridgeCases = {
  mainnet: {
    slug: "mainnet-bridge",
    nodeRpcPort: 16110,
    bridgePort: 5556,
    mustNot: [/runtimeRole\s*=\s*node/i, /runtime_role\s*=\s*node/i, /node output/i, /testnet10/i, /testnet12/i],
  },
  testnet10: {
    slug: "testnet10-bridge",
    nodeRpcPort: 16210,
    bridgePort: 5656,
    mustNot: [/runtimeRole\s*=\s*node/i, /runtime_role\s*=\s*node/i, /node output/i, /mainnet/i, /127\.0\.0\.1:16110/],
  },
};

function recordPid(network, runtimeRole, pid) {
  if (!observed.pids.some((item) => item.network === network && item.runtimeRole === runtimeRole && item.pid === pid)) {
    observed.pids.push({ network, runtimeRole, pid });
  }
}

function recordPort(network, runtimeRole, port, purpose) {
  if (!observed.ports.some((item) => item.network === network && item.runtimeRole === runtimeRole && item.port === port && item.purpose === purpose)) {
    observed.ports.push({ network, runtimeRole, port, purpose, host: "127.0.0.1" });
  }
}

function parseTauriBackendRecord(line) {
  const marker = "{\"timestamp\"";
  const index = String(line || "").indexOf(marker);
  if (index < 0) return null;
  try {
    const record = JSON.parse(String(line).slice(index));
    if (typeof record.details === "string" && record.details.trim()) {
      try {
        record.parsedDetails = JSON.parse(record.details);
      } catch (_) {
        record.parsedDetails = {};
      }
    } else {
      record.parsedDetails = record.details || {};
    }
    return record;
  } catch (_) {
    return null;
  }
}

async function readWdioRunnerLogLines() {
  const directory = path.join(artifactRoot, "wdio-runner");
  let files;
  try {
    files = await fs.readdir(directory, { withFileTypes: true });
  } catch (_) {
    return [];
  }

  const lines = [];
  for (const file of files) {
    if (!file.isFile() || !file.name.toLowerCase().endsWith(".log")) continue;
    const fullPath = path.join(directory, file.name);
    try {
      const text = await fs.readFile(fullPath, "utf8");
      lines.push(...text.split(/\r?\n/));
    } catch (_) {
      // The runner may still be writing this file; the next poll can read it.
    }
  }
  return lines;
}

async function waitForNativeClipboardTrace({ network, runtimeRole, bridgeInstanceId = "", sha256, afterTimestamp, timeoutMs = 30000 }) {
  const wantedSha = String(sha256 || "").toLowerCase();
  return await waitUntil(`native clipboard trace ${runtimeRole}/${network}`, timeoutMs, 500, async () => {
    const lines = await readWdioRunnerLogLines();
    for (const line of lines) {
      if (!line.includes("native.clipboard_write_succeeded")) continue;
      const record = parseTauriBackendRecord(line);
      if (!record) continue;
      if (Number(record.timestamp || 0) < Number(afterTimestamp || 0)) continue;
      if (String(record.source || "").toLowerCase() !== "native") continue;
      if (String(record.stage || "") !== "native.clipboard_write_succeeded") continue;
      if (String(record.network || "").toLowerCase() !== network) continue;
      if (String(record.result || "").toLowerCase() !== "ok") continue;

      const details = record.parsedDetails || {};
      const extra = details.extra || {};
      if (String(extra.runtimeRole || "").toLowerCase() !== runtimeRole) continue;
      if (bridgeInstanceId && String(extra.bridgeInstanceId || "") !== bridgeInstanceId) continue;
      if (wantedSha && String(details.sha256 || "").toLowerCase() !== wantedSha) continue;

      return {
        sequence: null,
        status: "ok",
        command: "kgw_copy_text_to_clipboard_v1",
        network,
        runtimeRole,
        bridgeInstanceId,
        expectedSha256: String(details.sha256 || ""),
        resultSha256: String(details.sha256 || ""),
        resultSource: "native.clipboard_write_succeeded",
        characterCount: Number(details.characterCount || 0),
        lineCount: Number(details.lineCount || 0),
        traceTimestamp: Number(record.timestamp || 0),
        traceDetails: details,
      };
    }
    return false;
  });
}

async function startNodeFromSettings(network, settings) {
  await openNodeSettings(network);
  await clickTestId(`kgw-node-start-${network}`);
  const status = await waitForOwnerStatus({ network, runtimeRole: "node", timeoutMs: 180000 });
  recordPid(network, "node", status.pid);
  await writeJson(path.join(settings.outputDirectory, "node-owner-status.json"), status);
  await waitForPort("127.0.0.1", settings.rpcPort, 180000);
  recordPort(network, "node", settings.rpcPort, "rpc");
  if (settings.p2pPort) {
    await waitForPort("127.0.0.1", settings.p2pPort, 180000);
    recordPort(network, "node", settings.p2pPort, "p2p");
  }
  return status;
}

async function startBridgeFromSettings(network, settings) {
  await openBridgeSettings(network);
  const selection = await readBridgeRuntimeSelection(network);
  const selectedInstancePort = selection.bridgePort || null;
  const bridgePort = settings.bridgePort || selectedInstancePort;
  await writeJson(path.join(settings.outputDirectory, "bridge-runtime-selection.json"), {
    ...selection,
    selectedInstancePort,
    runtimeBridgePort: bridgePort,
  });
  await clickTestId(`kgw-bridge-start-${network}`);
  const status = await waitForOwnerStatus({ network, runtimeRole: "bridge", timeoutMs: 180000 });
  recordPid(network, "bridge", status.pid);
  await writeJson(path.join(settings.outputDirectory, "bridge-owner-status.json"), status);
  await waitForPort("127.0.0.1", bridgePort, 180000);
  recordPort(network, "bridge", bridgePort, "stratum");
  return { ...status, bridgePort, bridgeInstanceId: selection.bridgeInstanceId || "1" };
}

async function copyAndVerify({ outputDirectory, network, runtimeRole, bridgeInstanceId = "", testId, visibleTestId, mustMatch = [], mustNotMatch = [] }) {
  const sentinel = `kgw-zero-touch-${runtimeRole}-${network}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  const sentinelSha = sha256Hex(sentinel);
  await writeClipboardSentinel(sentinel);
  await waitForRawLogOutput(visibleTestId, 60000);
  const visibleBefore = await readByTestId(visibleTestId);
  const normalizedVisible = normalizeClipboardText(visibleBefore);

  const copyStartedAt = Date.now() - 1000;
  await clickTestId(testId);
  const clipboardPath = path.join(outputDirectory, "clipboard.raw.txt");
  const clipboard = await waitForClipboardShaToFile({
    outputPath: clipboardPath,
    rejectedSha: sentinelSha,
    timeoutMs: 30000,
  });
  const event = await waitForNativeClipboardTrace({
    network,
    runtimeRole,
    bridgeInstanceId,
    sha256: clipboard.sha256,
    afterTimestamp: copyStartedAt,
    timeoutMs: 30000,
  });
  await writeJson(path.join(outputDirectory, "clipboard-capture.json"), { event, clipboard });

  assert.notEqual(clipboard.sha256, sentinelSha, "clipboard still contains the sentinel");
  assert.equal(event.expectedSha256, event.resultSha256, "expected and observed clipboard SHA-256 differ");
  assert.equal(clipboard.sha256, event.resultSha256, "Windows clipboard SHA-256 differs from observed result");
  assert.equal(clipboard.line_count, event.lineCount, "Windows clipboard line count differs from event");

  const rawPayload = await readByTestId(visibleTestId);
  const normalizedRawPayload = normalizeClipboardText(rawPayload);
  const copied = await fs.readFile(clipboardPath, "utf8");
  const normalizedCopied = normalizeClipboardText(copied);
  assert.equal(sha256Hex(normalizedCopied), clipboard.sha256, "clipboard.raw.txt SHA-256 differs from captured clipboard metadata");
  assert.ok(
    normalizedRawPayload.includes(normalizedCopied) || normalizedCopied.includes(normalizedVisible),
    `${runtimeRole}/${network} clipboard raw log is not represented in the visible raw log buffer`,
  );
  assertDirectRawPayload({
    text: rawPayload,
    label: `${runtimeRole}/${network} visible raw log`,
    mustMatch,
    mustNotMatch,
  });

  assertDirectRawPayload({
    text: copied,
    label: `${runtimeRole}/${network} clipboard raw log`,
    mustMatch,
    mustNotMatch,
  });

  observed.clipboardCaptures.push({
    network,
    runtimeRole,
    bridgeInstanceId,
    sha256: clipboard.sha256,
    outputPath: clipboardPath,
    characterCount: clipboard.character_count,
    lineCount: clipboard.line_count,
  });

  return { event, clipboard, clipboardPath };
}

async function saveRuntimeLogs({ outputDirectory, network, runtimeRole, bridgeInstanceId = "" }) {
  const report = await waitForRuntimeLogs({
    network,
    runtimeRole,
    bridgeInstanceId,
    timeoutMs: 90000,
  });
  assertRuntimeLogReport(report, { network, role: runtimeRole, bridgeInstanceId });
  await writeJson(path.join(outputDirectory, "runtime-logs.json"), report);
  await writeText(path.join(outputDirectory, "runtime-raw-lines.txt"), rawTextFromReport(report));
  return report;
}

async function writeCaseEvidence(slug, ports) {
  const outputDirectory = caseDir(slug);
  await ensureDir(outputDirectory);
  await saveDomState(outputDirectory, "current");
  await captureWindowsEvidence({
    repository,
    outputDirectory,
    ports,
    desktopPid: process.env.KGW_DESKTOP_PID || "",
  });
  return outputDirectory;
}

async function exerciseNode(network) {
  const config = nodeCases[network];
  const outputDirectory = await ensureDir(caseDir(config.slug));
  observed.cases.push({ slug: config.slug, network, runtimeRole: "node", outputDirectory });

  try {
    const status = await startNodeFromSettings(network, {
      outputDirectory,
      rpcPort: config.rpcPort,
      p2pPort: config.p2pPort,
    });
    assert.match(status.status, new RegExp(`network=${network}`, "i"), "status must identify the selected network");
    assert.match(status.status, /role=node/i, "status must identify node role");

    await openNodeLiveMonitor(network);
    const report = await saveRuntimeLogs({ outputDirectory, network, runtimeRole: "node" });
    const rawText = rawTextFromReport(report);
    await writeText(path.join(outputDirectory, "runtime-raw-lines-before-copy.txt"), rawText);

    await copyAndVerify({
      outputDirectory,
      network,
      runtimeRole: "node",
      testId: `kgw-node-copy-log-${network}`,
      visibleTestId: `kgw-node-log-output-${network}`,
      mustMatch: [/./],
      mustNotMatch: config.mustNot,
    });

    if (network === "testnet10") {
      assert.doesNotMatch(rawText, /nodes[\\/]+mainnet/i, "Testnet10 node raw output must not contain Mainnet data paths");
      if (/nodes[\\/]+/i.test(rawText)) {
        assert.match(rawText, /nodes[\\/]+testnet10/i, "Testnet10 node raw output paths must stay in the testnet10 directory");
      }
    }
  } finally {
    await writeCaseEvidence(config.slug, [config.rpcPort, config.p2pPort]).catch((error) => {
      observed.warnings.push(`node ${network} evidence capture failed: ${error.message || error}`);
    });
    await stopRuntime(network, "node");
    await waitForStopped({ network, runtimeRole: "node", timeoutMs: 45000 }).catch((error) => {
      observed.warnings.push(`node ${network} stop status did not settle: ${error.message || error}`);
    });
  }
}

async function exerciseBridge(network) {
  const config = bridgeCases[network];
  const outputDirectory = await ensureDir(caseDir(config.slug));
  observed.cases.push({ slug: config.slug, network, runtimeRole: "bridge", outputDirectory });

  try {
    await startNodeFromSettings(network, {
      outputDirectory,
      rpcPort: config.nodeRpcPort,
      p2pPort: network === "testnet10" ? 16211 : 16111,
    });

    const status = await startBridgeFromSettings(network, {
      outputDirectory,
      bridgePort: config.bridgePort,
    });
    assert.match(status.status, new RegExp(`network=${network}`, "i"), "bridge status must identify the selected network");
    assert.match(status.status, /role=bridge/i, "bridge status must identify bridge role");
    assert.match(status.status, /node_mode=external/i, "bridge must run against the separately owned node");

    await openBridgeLiveMonitor(network);

    let report = null;
    try {
      report = await waitForRuntimeLogs({ network, runtimeRole: "bridge", timeoutMs: 15000 });
    } catch (_) {
      await probeLocalStratum({
        host: "127.0.0.1",
        port: status.bridgePort,
        outputDirectory,
        label: `${network}-bridge`,
      });
      report = await waitForRuntimeLogs({ network, runtimeRole: "bridge", timeoutMs: 90000 });
    }

    assertRuntimeLogReport(report, { network, role: "bridge" });
    await writeJson(path.join(outputDirectory, "runtime-logs.json"), report);
    const rawText = rawTextFromReport(report);
    await writeText(path.join(outputDirectory, "runtime-raw-lines-before-copy.txt"), rawText);
    assertDirectRawPayload({
      text: rawText,
      label: `${network} bridge runtime raw lines`,
      mustMatch: [/./],
      mustNotMatch: config.mustNot,
    });

    await copyAndVerify({
      outputDirectory,
      network,
      runtimeRole: "bridge",
      bridgeInstanceId: status.bridgeInstanceId,
      testId: `kgw-bridge-copy-log-${network}`,
      visibleTestId: `kgw-bridge-log-output-${network}`,
      mustMatch: [/./],
      mustNotMatch: [
        ...config.mustNot,
        /generated bridge status summary/i,
        /bridge status summary/i,
        /placeholder/i,
      ],
    });
  } finally {
    await writeCaseEvidence(config.slug, [config.nodeRpcPort, config.bridgePort, config.bridgePort + 1, network === "testnet10" ? 16211 : 16111]).catch((error) => {
      observed.warnings.push(`bridge ${network} evidence capture failed: ${error.message || error}`);
    });
    await stopRuntime(network, "bridge");
    await waitForStopped({ network, runtimeRole: "bridge", timeoutMs: 45000 }).catch((error) => {
      observed.warnings.push(`bridge ${network} stop status did not settle: ${error.message || error}`);
    });
    await stopRuntime(network, "node");
    await waitForStopped({ network, runtimeRole: "node", timeoutMs: 45000 }).catch((error) => {
      observed.warnings.push(`node ${network} stop status did not settle after bridge case: ${error.message || error}`);
    });
  }
}

async function exerciseTestnet12Policy() {
  const slug = "testnet12-policy";
  const outputDirectory = await ensureDir(caseDir(slug));
  observed.cases.push({ slug, network: "testnet12", runtimeRole: "policy", outputDirectory });
  await writeClipboardSentinel(`kgw-zero-touch-policy-testnet12-${Date.now()}`);

  await openNodeSettings("testnet12");
  const nodeEnabled = await browser.execute(() => {
    const checkbox = document.querySelector('[data-testid="kgw-node-policy-enabled-testnet12"]');
    return Boolean(checkbox?.checked);
  });
  assert.equal(nodeEnabled, false, "Testnet12 node must be disabled by default");

  await openBridgeSettings("testnet12");
  const bridgeEnabled = await browser.execute(() => {
    const checkbox = document.querySelector('[data-testid="kgw-bridge-policy-enabled-testnet12"]');
    return Boolean(checkbox?.checked);
  });
  assert.equal(bridgeEnabled, false, "Testnet12 bridge must be disabled by default");

  const startBlocked = await invoke("kgw_kgw_apply_node_settings_v1", {
    network: "testnet12",
    nodeKind: "integrated-as-daemon",
    bridgeKind: "disable",
    nodeCommandPreview: "",
    bridgeCommandPreview: "",
    runtimeRole: "node",
    experimentalNetworkOptIn: false,
  }).then(
    (value) => ({ ok: true, value }),
    (error) => ({ ok: false, error: String(error?.message || error) }),
  );

  assert.equal(startBlocked.ok, false, "Testnet12 start without opt-in must be blocked");
  assert.match(startBlocked.error, /experimental|opt-in|disabled|policy/i, "Testnet12 block reason must mention policy");

  const status = String(await invoke("kgw_runtime_owner_status_v1", {
    network: "testnet12",
    runtimeRole: "node",
  }));
  const fields = parseKeyValueLine(status);
  assert.doesNotMatch(status, /pid=\d+/i, "zero-touch policy case must not launch Testnet12");
  assert.notEqual(fields.running, "true", "Testnet12 must not be running after policy check");

  await writeJson(path.join(outputDirectory, "testnet12-policy.json"), {
    nodeEnabled,
    bridgeEnabled,
    startBlocked,
    status,
  });
  await writeCaseEvidence(slug, [16310, 16311, 5755]);
}

describe("Kaspa Gateway zero-touch live raw-log E2E", function () {
  this.timeout(30 * 60 * 1000);

  before(async () => {
    await ensureDir(artifactRoot);
    await browser.setWindowSize(1280, 860);
    await installClipboardObserver();
    await shutdownAllRuntimeWorkers();
  });

  after(async () => {
    const shutdownResult = await shutdownAllRuntimeWorkers();
    observed.finishedAt = new Date().toISOString();
    observed.shutdownResult = shutdownResult;
    await writeJson(path.join(artifactRoot, "zero-touch-observations.json"), observed);
    const reportLines = [
      "# Kaspa Gateway Zero-Touch E2E Report",
      "",
      `Artifact root: ${artifactRoot}`,
      `Started: ${observed.startedAt}`,
      `Finished: ${observed.finishedAt}`,
      "",
      "## Live PIDs",
      ...observed.pids.map((item) => `- ${item.runtimeRole}/${item.network}: ${item.pid}`),
      "",
      "## Ports",
      ...observed.ports.map((item) => `- ${item.runtimeRole}/${item.network} ${item.purpose}: ${item.host}:${item.port}`),
      "",
      "## Clipboard Captures",
      ...observed.clipboardCaptures.map((item) => `- ${item.runtimeRole}/${item.network}: ${item.sha256} (${item.outputPath})`),
      "",
      "## Warnings",
      ...(observed.warnings.length ? observed.warnings.map((warning) => `- ${warning}`) : ["- None"]),
    ];
    await writeText(path.join(artifactRoot, "zero-touch-report.md"), reportLines.join("\n"));
  });

  it("Mainnet Node regression copies only real child raw lines", async () => {
    await exerciseNode("mainnet");
  });

  it("Testnet10 Node copies isolated Testnet10 child raw lines", async () => {
    await exerciseNode("testnet10");
  });

  it("Mainnet Bridge copies only bridge child raw lines", async () => {
    await exerciseBridge("mainnet");
  });

  it("Testnet10 Bridge copies isolated Testnet10 bridge child raw lines", async () => {
    await exerciseBridge("testnet10");
  });

  it("Testnet12 stays disabled by default and policy blocks zero-touch launch", async () => {
    await exerciseTestnet12Policy();
  });
});
