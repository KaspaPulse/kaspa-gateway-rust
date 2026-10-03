use std::fs;
use std::path::Path;

const WDIO_RELATIVE: &str = "e2e/wdio.conf.mjs";

fn expected_source() -> &'static str {
    r##"import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const repository = process.env.KGW_REPOSITORY || path.resolve(e2eDir, "..");
const artifactRoot = process.env.KGW_ZERO_TOUCH_ARTIFACT_DIR ||
  path.join(repository, "artifacts", "zero-touch-e2e", `wdio-${Date.now()}`);
const appBinaryPath = process.env.KGW_E2E_APP_BINARY ||
  path.join(repository, "target", "debug", "kaspa-gateway-desktop.exe");
const embeddedPort = Number(process.env.TAURI_WEBDRIVER_PORT || process.env.WDIO_EMBEDDED_PORT || "4445");
const selectedSpec = process.env.KGW_E2E_SPEC || "./specs/zero-touch-live-matrix.e2e.js";

async function writeFailureArtifact(name, content) {
  const failures = path.join(artifactRoot, "wdio-failures");
  await fs.mkdir(failures, { recursive: true });
  await fs.writeFile(path.join(failures, name), content, "utf8");
}

export const config = {
  runner: "local",
  specs: [selectedSpec],
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
      JSON.stringify({ repository, artifactRoot, appBinaryPath, embeddedPort, selectedSpec }, null, 2),
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
"##
}

fn normalized(text: &str) -> String {
    text.replace("\r\n", "\n")
}

pub fn run_cli(args: &mut impl Iterator<Item = String>, root: &Path) -> Result<String, String> {
    let action = args.next().unwrap_or_else(|| "check".to_owned());
    if args.next().is_some() {
        return Err("e2e-config-codegen accepts exactly one action: check or write".to_owned());
    }
    match action.as_str() {
        "check" => check(root),
        "write" => write(root),
        _ => Err(format!(
            "unknown e2e-config-codegen action: {action}; expected check or write"
        )),
    }
}

fn check(root: &Path) -> Result<String, String> {
    let path = root.join(WDIO_RELATIVE);
    let actual = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    if normalized(&actual) != expected_source() {
        return Err(format!("E2E_CONFIG_CODEGEN=FAIL\nDRIFT={WDIO_RELATIVE}"));
    }
    Ok("E2E_CONFIG_CODEGEN=PASS\nARTIFACT_COUNT=1".to_owned())
}

fn write(root: &Path) -> Result<String, String> {
    let path = root.join(WDIO_RELATIVE);
    let current = fs::read_to_string(&path).unwrap_or_default();
    let changed = normalized(&current) != expected_source();
    if changed {
        fs::write(&path, expected_source().as_bytes())
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    }
    Ok(format!(
        "E2E_CONFIG_CODEGEN=WRITE_PASS\nARTIFACT_COUNT=1\nCHANGED={}",
        usize::from(changed)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wdio_config_is_tool_glue_with_stable_safety_contracts() {
        let source = expected_source();
        for needle in [
            "runner: \"local\"",
            "maxInstances: 1",
            "driverProvider: \"embedded\"",
            "commandTimeout: 60000",
            "startTimeout: 120000",
            "statusPollTimeout: 10000",
            "specs: [selectedSpec]",
            "beforeSession: async () =>",
            "afterTest: async (test, _context, result) =>",
        ] {
            assert!(source.contains(needle), "missing WDIO contract: {needle}");
        }
    }

    #[test]
    fn current_tracked_config_matches_rust_authority() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let actual = fs::read_to_string(root.join(WDIO_RELATIVE)).unwrap();
        assert_eq!(normalized(&actual), expected_source());
    }
}
