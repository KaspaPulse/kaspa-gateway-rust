import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { promisify } from "node:util";
import { helperScript, writeJson } from "./paths.mjs";

const execFileAsync = promisify(execFile);

function resolvePowerShellExecutable() {
  const executable = process.env.KGW_REQUIRED_PWSH_PATH || process.env.KGW_PWSH_PATH || "pwsh.exe";
  if (/(^|[\\/])powershell(?:\.exe)?$/i.test(executable)) {
    throw new Error("Zero-touch E2E requires PowerShell 7 or later; Windows PowerShell is not supported.");
  }
  return executable;
}

export async function runPowerShell(script, args = [], options = {}) {
  const psArgs = [
    "-NoLogo",
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    script,
    ...args,
  ];
  try {
    const result = await execFileAsync(resolvePowerShellExecutable(), psArgs, {
      windowsHide: true,
      maxBuffer: options.maxBuffer || 16 * 1024 * 1024,
      timeout: options.timeout || 60000,
      cwd: options.cwd,
      env: { ...process.env, ...(options.env || {}) },
    });
    const stdout = String(result.stdout || "").trim();
    return stdout ? JSON.parse(stdout) : {};
  } catch (error) {
    const stdout = String(error?.stdout || "").trim();
    const stderr = String(error?.stderr || "").trim();
    const message = [
      error?.message || "PowerShell command failed",
      stdout ? `stdout=${stdout}` : "",
      stderr ? `stderr=${stderr}` : "",
    ].filter(Boolean).join("\n");
    throw new Error(message);
  }
}

export async function writeClipboardSentinel(value) {
  return await runPowerShell(helperScript("kgw_windows_clipboard.ps1"), [
    "-Mode",
    "write",
    "-Value",
    String(value),
  ]);
}

export async function readClipboardToFile(outputPath) {
  return await runPowerShell(helperScript("kgw_windows_clipboard.ps1"), [
    "-Mode",
    "read",
    "-OutputPath",
    outputPath,
  ]);
}

export async function waitForClipboardShaToFile({ outputPath, expectedSha = "", rejectedSha = "", timeoutMs = 30000 }) {
  const wanted = String(expectedSha || "").toLowerCase();
  const rejected = String(rejectedSha || "").toLowerCase();

  return await waitUntil("Windows clipboard content", timeoutMs, 500, async () => {
    const capture = await readClipboardToFile(outputPath);
    const actual = String(capture?.sha256 || "").toLowerCase();
    if (!actual) return false;
    if (rejected && actual === rejected) return false;
    if (wanted && actual !== wanted) return false;
    return capture;
  });
}

export async function captureWindowsEvidence({ repository, outputDirectory, ports = [], desktopPid = "" }) {
  return await runPowerShell(helperScript("kgw_windows_evidence.ps1"), [
    "-Repository",
    repository,
    "-OutputDirectory",
    outputDirectory,
    "-Ports",
    ports.join(","),
    "-DesktopPid",
    String(desktopPid || ""),
  ], { timeout: 60000 });
}

export async function waitUntil(label, timeoutMs, intervalMs, probe) {
  const started = Date.now();
  let lastError = null;

  while (Date.now() - started < timeoutMs) {
    try {
      const value = await probe();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }

    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }

  const suffix = lastError ? ` Last error: ${lastError.message || lastError}` : "";
  throw new Error(`${label} timed out after ${timeoutMs}ms.${suffix}`);
}

export async function waitForPort(host, port, timeoutMs = 120000) {
  return await waitUntil(`port ${host}:${port}`, timeoutMs, 500, async () => {
    return await new Promise((resolve) => {
      const socket = net.createConnection({ host, port });
      let settled = false;

      const finish = (value) => {
        if (settled) return;
        settled = true;
        socket.destroy();
        resolve(value);
      };

      socket.setTimeout(1500);
      socket.once("connect", () => finish({ host, port, connected: true, observedAt: new Date().toISOString() }));
      socket.once("timeout", () => finish(false));
      socket.once("error", () => finish(false));
    });
  });
}

export async function probeLocalStratum({ host = "127.0.0.1", port, outputDirectory, label }) {
  const payload = JSON.stringify({
    id: 1,
    method: "mining.subscribe",
    params: [`kgw-zero-touch-e2e-${label}`],
  }) + "\n";
  const evidence = {
    host,
    port,
    label,
    payload,
    connected: false,
    response: "",
    error: "",
    startedAt: new Date().toISOString(),
  };

  await new Promise((resolve) => {
    const socket = net.createConnection({ host, port });
    const chunks = [];
    let settled = false;

    const finish = (error) => {
      if (settled) return;
      settled = true;
      evidence.endedAt = new Date().toISOString();
      if (error) evidence.error = String(error?.message || error);
      evidence.response = Buffer.concat(chunks).toString("utf8");
      socket.destroy();
      resolve();
    };

    socket.setTimeout(2500);
    socket.once("connect", () => {
      evidence.connected = true;
      socket.write(payload, "utf8");
    });
    socket.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    socket.once("timeout", () => finish());
    socket.once("error", (error) => finish(error));
    socket.once("close", () => finish());
  });

  await fs.mkdir(outputDirectory, { recursive: true });
  await writeJson(path.join(outputDirectory, "stratum-probe.json"), evidence);
  return evidence;
}
