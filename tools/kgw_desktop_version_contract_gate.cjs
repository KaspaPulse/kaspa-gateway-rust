#!/usr/bin/env node

const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");

const packageJson = JSON.parse(
  readFileSync("apps/kaspa-gateway-desktop/package.json", "utf8"),
);
const cargoToml = readFileSync(
  "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml",
  "utf8",
);
const tauriConfig = JSON.parse(
  readFileSync("apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json", "utf8"),
);
const indexHtml = readFileSync(
  "apps/kaspa-gateway-desktop/frontend/index.html",
  "utf8",
);
const mainJs = readFileSync(
  "apps/kaspa-gateway-desktop/frontend/main.js",
  "utf8",
);
const diagnosticsRs = readFileSync(
  "apps/kaspa-gateway-desktop/src-tauri/src/diagnostics.rs",
  "utf8",
);
const libRs = readFileSync(
  "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
  "utf8",
);

const cargoVersionMatch = cargoToml.match(
  /^version\s*=\s*"([^"]+)"\s*$/mu,
);
assert.ok(cargoVersionMatch, "Desktop Cargo.toml must declare package version");
const cargoVersion = cargoVersionMatch[1];
assert.equal(
  packageJson.version,
  cargoVersion,
  "Desktop package.json and Cargo.toml versions must match",
);
assert.equal(
  tauriConfig.version,
  cargoVersion,
  "Desktop tauri.conf.json and Cargo.toml versions must match",
);

assert.match(
  diagnosticsRs,
  /pub fn kgw_app_version_v1\(\) -> String\s*\{\s*env!\("CARGO_PKG_VERSION"\)\.to_string\(\)\s*\}/u,
  "Desktop version IPC must derive from CARGO_PKG_VERSION",
);
assert.ok(
  libRs.includes("diagnostics::kgw_app_version_v1,"),
  "Desktop version IPC must be registered in the Tauri invoke handler",
);
assert.match(
  indexHtml,
  /<h1 id="kgwAppVersionTitle" data-kgw-no-i18n="true">KaspaGateway<\/h1>/u,
  "Desktop header must use an unversioned non-i18n fallback",
);
assert.doesNotMatch(
  indexHtml,
  /KaspaGateway V[0-9]+\.[0-9]+\.[0-9]+/u,
  "Desktop HTML must not hard-code a visible release version",
);
assert.ok(
  mainJs.includes('resolved.invoke("kgw_app_version_v1")'),
  "Desktop shell must request the authoritative package version over Tauri IPC",
);
assert.ok(
  mainJs.includes("await kgwHydrateAppVersionV1();"),
  "Desktop shell boot must hydrate the visible package version",
);
assert.ok(
  mainJs.includes('title.dataset.versionSource = "cargo-pkg-version"'),
  "Desktop shell must mark the authoritative version source",
);

const localeNames = [
  "ar", "de", "en", "es", "fr", "hi",
  "id", "ja", "ko", "ru", "tr", "zh-CN",
];
for (const localeName of localeNames) {
  const localePath = `apps/kaspa-gateway-desktop/frontend/i18n/${localeName}.json`;
  const localeText = readFileSync(localePath, "utf8");
  const locale = JSON.parse(localeText);
  assert.doesNotMatch(
    localeText,
    /KaspaGateway V[0-9]+\.[0-9]+\.[0-9]+/u,
    `${localePath} must not own a hard-coded Desktop release version`,
  );
  assert.ok(
    !Object.hasOwn(locale.ui?.shell?.kaspagateway ?? {}, "v0"),
    `${localePath} must not retain the retired version-only translation key`,
  );
}

console.log(
  `DESKTOP VERSION CONTRACT PASSED version=${cargoVersion} locales=${localeNames.length}`,
);
