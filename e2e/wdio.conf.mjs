import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const repository = process.env.KGW_REPOSITORY || path.resolve(e2eDir, "..");
const artifactRoot = process.env.KGW_ZERO_TOUCH_ARTIFACT_DIR ||
  path.join(repository, "artifacts", "zero-touch-e2e", `wdio-${Date.now()}`);
const appBinaryPath = process.env.KGW_E2E_APP_BINARY ||
  path.join(repository, "target", "debug", "kaspa-gateway-desktop.exe");
const embeddedPort = Number(process.env.TAURI_WEBDRIVER_PORT || process.env.WDIO_EMBEDDED_PORT || "4445");

async function writeFailureArtifact(name, content) {
  const failures = path.join(artifactRoot, "wdio-failures");
  await fs.mkdir(failures, { recursive: true });
  await fs.writeFile(path.join(failures, name), content, "utf8");
}

export const config = {
  runner: "local",
  specs: ["./specs/zero-touch-live-matrix.e2e.js"],
  maxInstances: 1,
  capabilities: [
    {
      browserName: "tauri",
      "tauri:options": {
        application: appBinaryPath
      }
    }
  ],
  services: [
    [
      "@wdio/tauri-service",
      {
        appBinaryPath,
        driverProvider: "embedded",
        embeddedPort,
        captureBackendLogs: true,
        captureFrontendLogs: true,
        backendLogLevel: "trace",
        frontendLogLevel: "debug",
        logDir: path.join(artifactRoot, "wdio-tauri"),
        commandTimeout: 60000,
        startTimeout: 120000,
        statusPollTimeout: 10000
      }
    ]
  ],
  framework: "mocha",
  reporters: [
    "spec",
    [
      "junit",
      {
        outputDir: path.join(artifactRoot, "junit"),
        outputFileFormat: (options) => `wdio-${options.cid}.xml`
      }
    ],
    [
      "json",
      {
        outputDir: path.join(artifactRoot, "json"),
        outputFileFormat: (options) => `wdio-${options.cid}.json`
      }
    ]
  ],
  logLevel: process.env.WDIO_LOG_LEVEL || "info",
  outputDir: path.join(artifactRoot, "wdio-runner"),
  mochaOpts: {
    ui: "bdd",
    timeout: 20 * 60 * 1000
  },
  beforeSession: async () => {
    await fs.mkdir(artifactRoot, { recursive: true });
    await fs.writeFile(
      path.join(artifactRoot, "wdio-session.json"),
      JSON.stringify({ repository, artifactRoot, appBinaryPath, embeddedPort }, null, 2),
      "utf8"
    );
  },
  afterTest: async (test, _context, result) => {
    if (result.passed) return;
    const safeName = String(test.title || "unknown-test").replace(/[^A-Za-z0-9_.-]+/g, "_").slice(0, 120);
    try {
      await browser.saveScreenshot(path.join(artifactRoot, "wdio-failures", `${safeName}.png`));
    } catch (error) {
      await writeFailureArtifact(`${safeName}.screenshot-error.txt`, String(error?.message || error));
    }
    try {
      await writeFailureArtifact(`${safeName}.html`, await browser.getPageSource());
    } catch (error) {
      await writeFailureArtifact(`${safeName}.page-source-error.txt`, String(error?.message || error));
    }
  }
};
