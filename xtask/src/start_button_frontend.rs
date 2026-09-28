use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_RENDER_FIXTURE: &str = r##########"
function cardInput(net, name, label, value = "", placeholder = "", span2 = false) {
  return wasmNodeCardInput(
    String(net || ""),
    String(name || ""),
    String(label || ""),
    String(value ?? ""),
    String(placeholder ?? ""),
    Boolean(span2),
    kgwNodeCommandInlineToggleR7(net, name)
  );
}

function cardSelect(net, name, label, options, value = "", span2 = false) {
  return wasmNodeCardSelect(
    String(net || ""),
    String(name || ""),
    String(label || ""),
    Array.from(options || [], (item) => String(item ?? "")),
    String(value ?? ""),
    Boolean(span2),
    kgwNodeCommandInlineToggleR7(net, name)
  );
}

function cardCheck(net, name, label, checked = false, span2 = false) {
  return wasmNodeCardCheck(
    String(net || ""),
    String(name || ""),
    String(label || ""),
    Boolean(checked),
    Boolean(span2)
  );
}





// KGW_NODE_LOG_AUTOSCROLL_CONTROLS_R27 is Rust-owned in node_frontend_helpers.rs.

/* KGW_NODE_RAW_LOG_OWNER_V2 is Rust-owned in node_start_trace.rs. */
function appendLog(net, message) {
  // Raw monitor text is driven by typed runtime log reports. This legacy hook is
  // intentionally inert so UI status strings cannot become fabricated raw lines.
  void net;
  void message;
}

function renderRuntime(net) {
  const networkIdentityControls = net.testnet
    ? `
      ${cardCheck(net.key, "testnet", "--testnet", true)}
      ${cardInput(net.key, "netsuffix", "--netsuffix", net.netsuffix, "required")}`
    : "";

  return `
    <div class="node-v6-grid">
      ${networkIdentityControls}
      ${cardSelect(net.key, "logLevel", "--loglevel", ["off", "error", "warn", "info", "debug", "trace"], "info")}
      ${cardInput(net.key, "asyncThreads", "--async-threads", "16")}
      ${cardInput(net.key, "ramScale", "--ram-scale", "1")}
      ${cardCheck(net.key, "yes", "--yes", true)}
      ${cardCheck(net.key, "noLogFiles", "--nologfiles", true)}
      ${cardCheck(net.key, "sanity", "--sanity", false)}
      ${cardCheck(net.key, "enableUnsyncedMining", "--enable-unsynced-mining", false, true)}
    </div>`;
}


function renderNetwork(net) {
  const p2pPort = net.key === "mainnet" ? "16111" : net.key === "testnet10" ? "16211" : "16711";
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "listenEnabled", "--listen", false)}
      ${cardInput(net.key, "listenHost", "listen host", "0.0.0.0")}
      ${cardInput(net.key, "listenPort", "listen port", p2pPort)}
      ${cardCheck(net.key, "externalIpEnabled", "--externalip", false)}
      ${cardInput(net.key, "externalIpHost", "external host", "", "ip")}
      ${cardInput(net.key, "externalIpPort", "external port", "", "port")}
      ${cardCheck(net.key, "disableUpnp", "--disable-upnp", true)}
      ${cardCheck(net.key, "noDnsSeed", "--nodnsseed", false)}
      ${cardInput(net.key, "uaComment", "--uacomment", "", "comment", true)}
    </div>`;
}

function renderRpc(net) {
  const base = net.key === "mainnet" ? 16110 : net.key === "testnet10" ? 16210 : 16210;
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "rpcListenEnabled", "--rpclisten", true)}
      ${cardInput(net.key, "rpcListenHost", "RPC host", "127.0.0.1")}
      ${cardInput(net.key, "rpcListenPort", "RPC port", String(base))}
      ${cardCheck(net.key, "rpcBorshEnabled", "--rpclisten-borsh", false)}
      ${cardInput(net.key, "rpcBorshHost", "Borsh host", "127.0.0.1")}
      ${cardInput(net.key, "rpcBorshPort", "Borsh port", String(base + 1000))}
      ${cardCheck(net.key, "rpcJsonEnabled", "--rpclisten-json", false)}
      ${cardInput(net.key, "rpcJsonHost", "JSON host", "127.0.0.1")}
      ${cardInput(net.key, "rpcJsonPort", "JSON port", String(base + 2000))}
      ${cardInput(net.key, "rpcMaxClients", "--rpcmaxclients (managed max 16)", "16")}
      ${cardCheck(net.key, "unsafeRpc", "--unsaferpc", false)}
      ${cardCheck(net.key, "noGrpc", "--nogrpc", false)}
    </div>`;
}

function renderPeers(net) {
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "connectEnabled", "--connect", false)}
      ${cardInput(net.key, "connectHost", "connect host", "", "host")}
      ${cardInput(net.key, "connectPort", "connect port", "", "port")}
      ${cardCheck(net.key, "addPeerEnabled", "--addpeer", false)}
      ${cardInput(net.key, "addPeerHost", "peer host", "", "host")}
      ${cardInput(net.key, "addPeerPort", "peer port", "", "port")}
      ${cardInput(net.key, "outPeers", "--outpeers", "8")}
      ${cardInput(net.key, "maxInPeers", "--maxinpeers (managed max 32)", "32")}
    </div>`;
}

function renderDatabase(net) {
  return `
    <div class="node-v6-grid">
      ${cardCheck(net.key, "utxoIndex", "--utxoindex", true)}
      ${cardCheck(net.key, "archival", "--archival", false)}
      ${cardCheck(net.key, "resetDb", "--reset-db", false)}
      ${cardCheck(net.key, "perfMetrics", "--perf-metrics", true)}
      ${cardInput(net.key, "maxTrackedAddresses", "--max-tracked-addresses", "", "0")}
      ${cardInput(net.key, "retentionDays", "--retention-period-days", "", "optional")}
      ${cardInput(net.key, "perfMetricsInterval", "--perf-metrics-interval-sec", "", "optional", true)}
    </div>`;
}

function renderRocksDb(net) {
  return `
    <div class="node-v6-grid">
      ${cardSelect(net.key, "rocksDbPreset", "--rocksdb-preset", ["", "default", "hdd"], "")}
      ${cardInput(net.key, "rocksDbCacheSize", "--rocksdb-cache-size", "", "MB")}
      ${cardInput(net.key, "rocksDbWalDir", "--rocksdb-wal-dir", "", "path", true)}
      ${cardInput(net.key, "overrideParamsFile", "--override-params-file (unsupported: managed network)", "", "not supported", true)}
    </div>`;
}

function renderPaths(net) {
  return `
    <div class="node-v6-grid">
      ${cardInput(net.key, "configFile", "--configfile (unsupported: managed ownership)", "", "not supported")}
      ${cardInput(net.key, "appDir", "--appdir (managed per network)", "", "managed by desktop")}
      ${cardInput(net.key, "logDir", "--logdir", "", "log dir")}
    </div>`;
}

function renderSections(net) {
  const template = document.createElement("template");
  template.innerHTML = [renderRuntime(net), renderNetwork(net), renderRpc(net), renderPeers(net),
    renderDatabase(net), renderRocksDb(net), renderPaths(net)].join("");
  const cards = new Map();
  template.content.querySelectorAll(".node-v6-card").forEach(card => {
    const field = card.querySelector("[id]");
    if (field) cards.set(field.id.slice(("node-" + net.key + "-").length), card.outerHTML);
  });
  const definitions = [
    ["general", "basic", "Basic", "testnet netsuffix utxoIndex yes"],
    ["general", "networking", "Networking", "listenEnabled listenHost listenPort externalIpEnabled externalIpHost externalIpPort disableUpnp noDnsSeed uaComment"],
    ["general", "rpc", "RPC", "rpcListenEnabled rpcListenHost rpcListenPort"],
    ["general", "performance", "Performance", "asyncThreads ramScale outPeers maxInPeers"],
    ["general", "storage", "Storage", "appDir rocksDbPreset rocksDbCacheSize"],
    ["general", "logging", "Logging", "logLevel noLogFiles logDir"],
    ["advanced", "p2p", "P2P", "connectEnabled connectHost connectPort addPeerEnabled addPeerHost addPeerPort"],
    ["advanced", "rpc-advanced", "RPC Advanced", "rpcBorshEnabled rpcBorshHost rpcBorshPort rpcJsonEnabled rpcJsonHost rpcJsonPort rpcMaxClients noGrpc"],
    ["advanced", "database", "Database", "archival maxTrackedAddresses retentionDays rocksDbWalDir configFile sanity"],
    ["advanced", "metrics", "Metrics", "perfMetrics perfMetricsInterval"],
    ["advanced", "experimental", "Experimental", "overrideParamsFile"],
    ["advanced", "dangerous", "Dangerous", "resetDb unsafeRpc enableUnsyncedMining"]
  ];
  const groups = definitions.map(([section, key, label, names]) => {
    const fields = names.split(" ").map(name => { const card = cards.get(name) || ""; cards.delete(name); return card; }).join("");
    const note = key === "dangerous"
      ? '<p class="kgw-danger-warning">Reset DB removes network data. Unsafe RPC can expose privileged methods. Unsynced mining bypasses synchronization. Existing confirmations and network restrictions still apply.</p>'
      : key === "experimental" ? '<p class="kgw-settings-info">' + esc(kgwNodeNetworkPolicyMessage(net.key)) + "</p>" : "";
    return [section, key, label, note + '<div class="kgw-settings-grid">' + fields + "</div>"];
  });
  if (cards.size) throw new Error("Ungrouped node settings: " + [...cards.keys()].join(", "));
  return renderSettingsTabs("node", net.key, groups);
}

/* R101U inner-tab persistence is Rust-owned in node_frontend_helpers.rs. */
function renderNetworkPanel(net, index) {
  /* KGW_NODE_LIVE_MONITOR_TAB_LABEL_ORDER_R101S */
  /* KGW_NODE_LIVE_MONITOR_DEFAULT_LAST_TAB_R101U
   * Settings is no longer the default inner panel.
   * Default is Live Node Monitor unless a valid saved tab exists for this network.
   */
  const activeInnerTab = kgwNodeResolveInnerTabR101U(net.key);
  const logActive = activeInnerTab === "log";
  const settingsActive = activeInnerTab === "settings";

  return `
    <div class="node-v6-network-panel${index === 0 ? " active" : ""}" data-node-network-panel="${net.key}" data-testid="kgw-node-panel-${net.key}"${index === 0 ? "" : " hidden"}>
      <div class="node-v6-inner-tabs">
        <button type="button" class="node-v6-inner-tab${logActive ? " active" : ""}" data-net="${net.key}" data-node-inner-tab="log" data-testid="kgw-node-live-monitor-${net.key}">Live Node Monitor</button>
        <button type="button" class="node-v6-inner-tab${settingsActive ? " active" : ""}" data-net="${net.key}" data-node-inner-tab="settings" data-testid="kgw-node-settings-${net.key}">Settings</button>
      </div>

      <div class="node-v6-inner-panel${settingsActive ? " active" : ""}" data-net="${net.key}" data-node-inner-panel="settings" data-node-settings-panel="${net.key}"${settingsActive ? "" : " hidden"}>
        <div class="kgw-settings-scroll">
        <section class="kgw-network-policy${net.experimental ? " is-experimental" : ""}" data-net="${net.key}" data-testid="kgw-node-policy-${net.key}">
          <div>
            <strong>${net.label}</strong>${net.experimental ? '<span class="kgw-experimental-badge">Experimental - opt-in required</span>' : ""}
            <span>${esc(kgwNodeNetworkPolicyMessage(net.key))}</span>
          </div>
          <div class="kgw-network-policy-controls">
            <span id="${id(net.key, "policyStatus")}" class="kgw-network-policy-status">Stopped</span>
            <label>
              <input type="checkbox" data-node-network-enabled="${net.key}" data-testid="kgw-node-policy-enabled-${net.key}" data-net="${net.key}"${kgwNodeNetworkEnabled(net.key) ? " checked" : ""}>
              Profile enabled
            </label>
          </div>
        </section>

        <section class="node-v6-command kgw-effective-preview">
          <div class="kgw-preview-row">
            <strong>Effective node settings</strong>
            <button type="button" data-settings-preview-toggle aria-expanded="false" aria-controls="${id(net.key, "previewBody")}">Expand</button>
            <button type="button" class="node-v6-copy" data-node-action="copy-command" data-net="${net.key}" title="Copy effective settings">Copy command</button>
            <button type="button" data-node-action="copy-path" data-net="${net.key}">Copy data directory</button>
          </div>
          <p id="${id(net.key, "previewMessage")}" class="kgw-preview-message" role="status" aria-live="polite"></p>
          <div class="kgw-preview-body" id="${id(net.key, "previewBody")}" hidden>
            <p class="kgw-preview-help">The embedded node library consumes these equivalent arguments inside KaspaGateway self-workers.</p>
            <textarea id="${id(net.key, "commandPreview")}" aria-label="Effective node settings preview" readonly spellcheck="false" wrap="soft"></textarea>
            <details class="kgw-arguments"><summary>Argument list</summary><pre id="${id(net.key, "argumentList")}"></pre></details>
          </div>
        </section>

        <section class="node-v6-toolbar">
          <div class="node-v6-buttons">
            <button type="button" class="good" data-node-action="start" data-testid="kgw-node-start-${net.key}" data-net="${net.key}">Start</button>
            <button type="button" data-node-action="stop" data-testid="kgw-node-stop-${net.key}" data-net="${net.key}">Stop</button>
          </div>

          <div class="node-v6-status">
            <span id="${id(net.key, "runtimeStatus")}" class="node-v6-runtime-status-pill" data-state="stopped">Stopped</span>
            <span id="${id(net.key, "runtimeEvidence")}" class="node-v6-runtime-evidence">No process owner</span>
            <span id="${id(net.key, "settingsAuthority")}" class="node-v6-runtime-evidence">Effective settings apply on next Start</span>
          </div>

          <div id="${id(net.key, "runtimeError")}" class="node-v6-runtime-error" role="status" aria-live="polite" hidden></div>
        </section>

        ${renderSections(net)}
        </div>

        <div class="settings-bottom-actions node-settings-bottom-actions">
        <button type="button" data-node-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-node-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-node-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        <p class="kgw-settings-help" data-settings-defaults-context="${net.key}">Restore uses KaspaGateway defaults. Settings apply on the next Start.</p>
        </div>

      </div>

      <div class="node-v6-inner-panel${logActive ? " active" : ""}" data-net="${net.key}" data-node-inner-panel="log" data-testid="kgw-node-live-panel-${net.key}"${logActive ? "" : " hidden"}>
        <p id="${id(net.key, "monitorState")}" class="kgw-monitor-state" role="status">Node: Stopped</p>
        <div class="node-v6-log-toolbar">
          <button type="button" data-node-action="monitor-start" data-net="${net.key}">Start Node</button>
          <span class="node-v6-log-metadata" data-net="${net.key}">Network: ${net.label} | Source: self-worker | Streams: stdout/stderr</span>
          <button type="button" data-node-action="copy-log" data-testid="kgw-node-copy-log-${net.key}" data-net="${net.key}">Copy Log</button>
          <button type="button" data-node-action="clear-log" data-testid="kgw-node-clear-log-${net.key}" data-net="${net.key}">Clear Log</button>
        </div>
        <div id="${id(net.key, "logEmpty")}" class="node-v6-log-empty" data-node-log-empty="${net.key}">Node is stopped. Start the node to view its logs.</div>
        <pre id="${id(net.key, "logOutput")}" class="node-v6-log" data-testid="kgw-node-log-output-${net.key}"></pre>
      </div>
</div>`;
}


"##########;

const NODE_BRIDGE: &str = r########"#!/usr/bin/env node
const assert = require("assert");
const { webcrypto } = require("crypto");
const fs = require("fs");
const path = require("path");
const vm = require("vm");

const repo = process.cwd();
const nodeJsPath = path.join(
  repo,
  "apps",
  "kaspa-gateway-desktop",
  "frontend",
  "src",
  "tabs",
  "kaspa-node",
  "kaspa-node.js",
);
const source = fs.readFileSync(nodeJsPath, "utf8");
const tauriConfigPath = path.join(
  repo,
  "apps",
  "kaspa-gateway-desktop",
  "src-tauri",
  "tauri.conf.json",
);
const rustNodeHelpersPath = path.join(
  repo,
  "crates",
  "kaspa-gateway-frontend-wasm",
  "src",
  "node_frontend_helpers.rs",
);
const rustNodeHelpersSource = fs.readFileSync(rustNodeHelpersPath, "utf8");
const renderFixturePath = process.env.KGW_NODE_RENDER_FIXTURE;
if (!renderFixturePath) throw new Error("KGW_NODE_RENDER_FIXTURE is required");
const renderFixture = fs.readFileSync(renderFixturePath, "utf8");

function fail(message) {
  console.error("KGW start button frontend tests FAILED");
  console.error(message);
  process.exit(1);
}

function assertIncludes(text, needle, message) {
  assert.ok(text.includes(needle), message + " (`" + needle + "` missing)");
}

function extractBetween(text, start, end) {
  const startIndex = text.indexOf(start);
  assert.ok(startIndex >= 0, "missing start marker: " + start);
  const endIndex = text.indexOf(end, startIndex + start.length);
  assert.ok(endIndex > startIndex, "missing end marker: " + end);
  return text.slice(startIndex, endIndex);
}

function staticPlacementTests() {
  assert.ok(
    source.includes("nodeRenderNetworkPanelsHtml as wasmNodeRenderNetworkPanelsHtml")
      && source.includes("host.innerHTML = wasmNodeRenderNetworkPanelsHtml();"),
    "Production Node panel rendering must delegate to the Rust/WASM renderer",
  );
  for (const forbidden of [
    "function cardInput(",
    "function cardSelect(",
    "function cardCheck(",
    "function renderRuntime(",
    "function renderNetwork(",
    "function renderRpc(",
    "function renderPeers(",
    "function renderDatabase(",
    "function renderRocksDb(",
    "function renderPaths(",
    "function renderSections(",
    "function renderNetworkPanel(",
  ]) {
    assert.ok(!source.includes(forbidden), "Hand-maintained Node renderer must stay absent: " + forbidden);
  }

  const renderStart = rustNodeHelpersSource.indexOf("fn render_node_network_panel(");
  assert.ok(renderStart >= 0, "missing Rust render_node_network_panel owner");
  const renderSource = rustNodeHelpersSource.slice(renderStart);
  const settingsPanel = extractBetween(
    renderSource,
    'data-node-inner-panel="settings"',
    'data-node-inner-panel="log"',
  );
  const logPanel = extractBetween(renderSource, 'data-node-inner-panel="log"', '</div>"#,');

  assertIncludes(settingsPanel, 'data-node-action="start"', "Settings must own Start");
  assertIncludes(settingsPanel, 'data-node-action="stop"', "Settings must own Stop");
  assertIncludes(settingsPanel, "data-node-network-enabled", "Settings must own network enable");
  assertIncludes(settingsPanel, "kgw-network-policy", "Settings must own network policy");
  assertIncludes(settingsPanel, "{runtime_error}", "Settings must expose the Rust runtime-error placeholder");
  assertIncludes(settingsPanel, "{runtime_status}", "Settings must expose the Rust runtime-status placeholder");
  assertIncludes(rustNodeHelpersSource, 'runtime_error = id("runtimeError")', "Rust renderer must bind runtimeError to the shared ID contract");
  assertIncludes(rustNodeHelpersSource, 'runtime_status = id("runtimeStatus")', "Rust renderer must bind runtimeStatus to the shared ID contract");

  assert.ok(!logPanel.includes('data-node-action="start"'), "Live Node Monitor must not contain Start");
  assert.ok(!logPanel.includes('data-node-action="stop"'), "Live Node Monitor must not contain Stop");
  assert.ok(!logPanel.includes("data-node-network-enabled"), "Live Node Monitor must not contain network enable");
  assert.ok(!logPanel.includes("kgw-network-policy"), "Live Node Monitor must not contain network policy");
  assertIncludes(logPanel, 'data-node-action="copy-log"', "Live Node Monitor must contain Copy Log");
  assertIncludes(logPanel, 'data-node-action="clear-log"', "Live Node Monitor must contain Clear Log");
  assertIncludes(logPanel, "node-v6-log-metadata", "Live Node Monitor must contain stream/source metadata");

  const startMatches = renderSource.match(/<button[^>]+data-node-action="start"/g) || [];
  const stopMatches = renderSource.match(/<button[^>]+data-node-action="stop"/g) || [];
  assert.strictEqual(startMatches.length, 1, "Rust renderer Start control markup must not be duplicated");
  assert.strictEqual(stopMatches.length, 1, "Rust renderer Stop control markup must not be duplicated");
  assert.ok(!/<button[^>]+\s+id\s*=[^>]+data-node-action="start"/.test(renderSource), "Start control must not use duplicate generated IDs");
  assert.ok(!/<button[^>]+data-node-action="start"[^>]+\s+id\s*=/.test(renderSource), "Start control must not use duplicate generated IDs");
  assert.ok(!/<button[^>]+\s+id\s*=[^>]+data-node-action="stop"/.test(renderSource), "Stop control must not use duplicate generated IDs");
  assert.ok(!/<button[^>]+data-node-action="stop"[^>]+\s+id\s*=/.test(renderSource), "Stop control must not use duplicate generated IDs");

  assert.ok(!/appendLog\([^)]*initialized/i.test(source), "Synthetic initialized text must not be inserted into raw logs");
  assert.ok(!/appendLog\([^)]*node settings saved/i.test(source), "Settings success text must not be inserted into raw logs");
  assert.ok(!/appendLog\([^)]*node .* response/i.test(source), "Synthetic start response text must not be inserted into raw logs");
  assert.ok(
    source.includes("nodeApplyRuntimeLogReport as kgwNodeApplyRuntimeLogReportV1")
      && source.includes("nodeHandleLogAction as kgwNodeHandleLogActionV29")
      && source.includes("nodeCopyLogFailure as kgwNodeCopyLogFailureV1")
      && !source.includes("function kgwNodeApplyRuntimeLogReportV1(")
      && !source.includes("function kgwNodeHandleLogActionV29(")
      && !source.includes("function kgwNodeCopyLogFailureV1("),
    "Node raw-log and log-action adapters must bind directly to the Rust/WASM owner without JavaScript wrappers",
  );
  assert.ok(
    source.includes("const KGW_NODE_RUNTIME_INVOKE_TIMEOUT_MS = 110000"),
    "Node Start must wait through the backend readiness window",
  );
  assert.ok(
    source.includes("const KGW_NODE_STOP_INVOKE_TIMEOUT_MS = 0"),
    "Node Stop must defer terminality to the async backend without a JS wall-clock cutoff",
  );
  assert.ok(source.includes('action === "start" ? "Starting" : "Stopping"'), "Stop click must enter Stopping before backend completion");
  assert.ok(source.includes('evidence.fields.running === "false"'), "Stopped requires terminal backend liveness evidence");
  assert.ok(source.includes("Stop required FORCED termination."), "forced Stop must be visible outside raw logs");
  assert.ok(!/appendLog\([^\n]*(FORCED|graceful|stop_outcome|Stopping)/i.test(source), "Stop control diagnostics must remain outside raw Node logs");
  assert.ok(
    source.includes("nodeRuntimeIsRunning as kgwNodeR51IsRunning") && !source.includes("function kgwNodeR51IsRunning("),
    "Node status polling must delegate READY/running classification to the Rust/WASM owner",
  );
  assert.ok(
    source.includes("nodeRuntimeErrorFromStatus as kgwNodeRuntimeErrorFromStatus")
      && !source.includes("function kgwNodeRuntimeErrorFromStatus(")
      && source.includes("kgwNodeRuntimeErrorFromStatus(status)"),
    "Node status polling must surface typed post-READY runtime failures through the Rust/WASM owner",
  );
  assert.ok(
    source.includes("nodeRuntimeEvidence as kgwNodeRuntimeEvidence")
      && !source.includes("function kgwNodeRuntimeEvidence(")
      && source.includes("nodeAssertStartEvidence as kgwNodeAssertStartEvidence")
      && !source.includes("function kgwNodeAssertStartEvidence("),
    "Node runtime evidence and Start attestation must be delegated to the Rust/WASM owner",
  );
  assert.ok(
    source.includes('kgwI18nTextR41("runtime.failed", "Failed")')
      && source.includes("nodeI18nText as kgwI18nTextR41"),
    "Node post-READY failure must remain visible outside raw logs",
  );
  assert.ok(
    source.includes("nodeCommandInlineState as kgwNodeCommandInlineStateR7")
      && !source.includes("nodeCommandInlineToggle as kgwNodeCommandInlineToggleR7")
      && source.includes("nodeRefreshInlineCommandToggles as kgwNodeRefreshInlineCommandTogglesR7")
      && source.includes("nodeToggleCommandOptionAndUpdate as wasmNodeToggleCommandOptionAndUpdate")
      && !source.includes("nodeToggleCommandOption as wasmNodeToggleCommandOption")
      && source.includes("nodeRenderNetworkPanelsHtml as wasmNodeRenderNetworkPanelsHtml")
      && rustNodeHelpersSource.includes("fn command_inline_toggle_html(")
      && !source.includes("function kgwNodeCommandInlineStateR7(")
      && !source.includes("function kgwNodeCommandInlineToggleR7(")
      && !source.includes("function kgwNodeRefreshInlineCommandTogglesR7(")
      && !source.includes("nodeCommandShouldInclude as wasmNodeCommandShouldInclude")
      && !source.includes("function kgwNodeCommandShouldIncludeR7("),
    "Active Node command-composer adapters must bind directly to Rust/WASM, renderer-only inline-toggle ownership must remain in Rust, and retired JS wrappers/imports must stay absent",
  );
  assert.ok(
    !source.includes("window.__kgwNodeCommandComposerInlineR7 = window.__kgwNodeCommandComposerInlineR7 || {}"),
    "Node command-composer state initialization must not remain implemented in hand-maintained JS",
  );
  assert.ok(
    !source.includes("NODE_ENDPOINTS.some(row => row[1] === name || row[2] === name)"),
    "Node command-composer schema policy must not remain implemented in hand-maintained JS",
  );
  assert.ok(
    source.includes("nodeR51Keys as kgwNodeR51Keys")
      && !source.includes("function kgwNodeR51Keys(")
      && source.includes("nodeR51Panel as kgwNodeR51Panel")
      && !source.includes("function kgwNodeR51Panel(")
      && source.includes("nodeR51Fields as kgwNodeR51Fields")
      && !source.includes("function kgwNodeR51Fields(")
      && source.includes("nodeR51ReadSettingsTracked as kgwNodeR51ReadSettings")
      && !source.includes("function kgwNodeR51ReadSettings(")
      && source.includes("nodeR51Load as kgwNodeR51Load")
      && !source.includes("function kgwNodeR51Load(")
      && source.includes("nodeR51CaptureFactoryDefaults as kgwNodeR51CaptureFactoryDefaults")
      && !source.includes("function kgwNodeR51CaptureFactoryDefaults(")
      && source.includes("nodeR51LoadSavedSettings as wasmNodeR51LoadSavedSettings"),
    "Node R51 persistence core must bind directly to the Rust/WASM owner",
  );
  assert.ok(
    !source.includes("nodeCommandOptionsKey as wasmNodeCommandOptionsKey")
      && !source.includes("KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C")
      && source.includes("nodeR51SaveSettings as kgwNodeR51SaveSettings")
      && !source.includes("function kgwNodeR51SaveSettings(")
      && source.includes("nodeR51SetAsDefaults as kgwNodeR51SetAsDefaults")
      && !source.includes("function kgwNodeR51SetAsDefaults(")
      && !source.includes("function kgwNodeR51WriteSettings(")
      && !source.includes("function kgwNodeR51Store("),
    "Node R51 read/load/save/set-defaults must use direct Rust/WASM bindings",
  );
  assert.ok(
    source.includes("nodeR51RestoreDefaultsAction as wasmNodeR51RestoreDefaultsAction")
      && /function kgwNodeR51RestoreDefaults\(net\)[\s\S]*?wasmNodeR51RestoreDefaultsAction/.test(source),
    "Node R51 Restore Defaults compatibility wrapper must remain delegated to Rust/WASM",
  );
  assert.ok(
    !source.includes("const commandOptions = values && values[KGW_NODE_R51_COMMAND_OPTIONS_KEY_R38C];")
      && !source.includes("state[String(name)] = Boolean(enabled) && (!NODE_OPTIONAL.has(name)")
      && !source.includes('const KGW_NODE_R51_STORAGE_PREFIX = "kgw.node.direct.v51.";')
      && !source.includes("localStorage.setItem(KGW_NODE_R51_STORAGE_PREFIX")
      && !source.includes('kgwNodeR51Store("saved:" + net, values)')
      && !source.includes('kgwNodeR51Store("default:" + net, values)')
      && !source.includes('const defaults = kgwNodeR51Load("default:" + net) || kgwNodeR51Load("factory:" + net)'),
    "Node R51 persistence implementation must not return to hand-maintained JS",
  );
  assert.ok(
    source.includes("nodeEffectiveNodeSettings as kgwNodeEffectiveNodeSettings")
      && source.includes("nodeValidateForm as kgwNodeValidateForm")
      && !source.includes("nodeRequireValidSettings as kgwNodeRequireValidSettings")
      && !source.includes("function kgwNodeEffectiveNodeSettings(")
      && !source.includes("function kgwNodeValidateForm(")
      && !source.includes("function kgwNodeRequireValidSettings(")
      && rustNodeHelpersSource.includes("fn node_validate_form_inner(")
      && rustNodeHelpersSource.includes("node_require_valid_settings"),
    "Node effective settings and validation must bind directly to the Rust/WASM owner",
  );
  assert.ok(
    source.includes("nodeRuntimeArgs as nodeRuntimeArgs")
      && !source.includes("function nodeRuntimeArgs("),
    "Node runtime args must delegate to the Rust/WASM owner",
  );
  assert.ok(
    source.includes("nodeSyncDependencies as wasmNodeSyncDependencies")
      && source.includes("nodePreviewMessage as kgwNodePreviewMessage")
      && source.includes("nodePanelStartFromMonitor as panelStartFromMonitor")
      && !source.includes("function kgwNodeForm(")
      && !source.includes("function kgwNodeSyncDependencies(")
      && !source.includes("function kgwNodePreviewMessage(")
      && !source.includes("function panelStartFromMonitor(")
      && rustNodeHelpersSource.includes("fn node_sync_dependencies_inner(")
      && rustNodeHelpersSource.includes("fn panel_start_from_monitor_inner("),
    "Node form/dependency/preview/monitor-start ownership must remain in Rust/WASM",
  );
  assert.ok(
    source.includes("nodeExplicitTrace as kgwNodeExplicitTraceR27D")
      && source.includes("nodeExplicitOwnerTrace as kgwNodeExplicitOwnerTraceR27D")
      && !source.includes("function kgwNodeExplicitTraceR27D(")
      && !source.includes("function kgwNodeExplicitOwnerTraceR27D(")
      && source.includes('kgwNodeExplicitOwnerTraceR27D(net, "settings-scope", "r27d-scoped-update"'),
    "Both Node explicit-trace variants must bind directly to their Rust/WASM owners",
  );
  for (const retired of [
    "function kgwNodeEffectiveNumber(",
    "function kgwNodeEffectiveEndpoint(",
    "const rpcBase = net ===",
    "asyncThreads: kgwNodeEffectiveNumber",
    'nodeKind: "integrated-as-daemon"',
  ]) {
    assert.ok(!source.includes(retired), "Retired Node effective-settings implementation must stay out of hand-maintained JS: " + retired);
  }
}

class ClassList {
  constructor(owner) {
    this.owner = owner;
    this.items = new Set();
  }

  add(...items) {
    for (const item of items) this.items.add(String(item));
    this.sync();
  }

  remove(...items) {
    for (const item of items) this.items.delete(String(item));
    this.sync();
  }

  contains(item) {
    return this.items.has(String(item));
  }

  toggle(item, force) {
    const key = String(item);
    const enabled = force === undefined ? !this.items.has(key) : Boolean(force);
    if (enabled) this.items.add(key);
    else this.items.delete(key);
    this.sync();
    return enabled;
  }

  setFromString(value) {
    this.items = new Set(String(value || "").split(/\s+/).filter(Boolean));
    this.sync();
  }

  sync() {
    this.owner.attributes.class = Array.from(this.items).join(" ");
  }

  toString() {
    return Array.from(this.items).join(" ");
  }
}

function dataKey(attr) {
  return attr
    .slice("data-".length)
    .replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
}

class TestElement {
  constructor(tagName, document) {
    this.tagName = String(tagName || "").toUpperCase();
    this.ownerDocument = document;
    this.parentElement = null;
    this.children = [];
    this.attributes = {};
    this.dataset = {};
    this.classList = new ClassList(this);
    this.style = { setProperty(name, value) { this[name] = value; } };
    this.listeners = new Map();
    this.hidden = false;
    this.disabled = false;
    this.readOnly = false;
    this.checked = false;
    this.value = "";
    this.textContent = "";
    this.id = "";
    this.type = "";
    if (this.tagName === "TEMPLATE") this.content = this;
  }

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }

  append(...nodes) {
    for (const node of nodes) this.appendChild(node);
  }

  remove() {
    if (!this.parentElement) return;
    this.parentElement.children = this.parentElement.children.filter((child) => child !== this);
    this.parentElement = null;
  }

  contains(node) {
    if (node === this) return true;
    return walk(this).includes(node);
  }

  setAttribute(name, value) {
    const key = String(name);
    const stringValue = String(value);
    this.attributes[key] = stringValue;
    if (key === "id") this.id = stringValue;
    if (key === "class") this.classList.setFromString(stringValue);
    if (key === "type") this.type = stringValue;
    if (key === "value") this.value = stringValue;
    if (key === "hidden") this.hidden = true;
    if (key === "disabled") this.disabled = true;
    if (key === "readonly") this.readOnly = true;
    if (key === "checked") this.checked = true;
    if (key.startsWith("data-")) this.dataset[dataKey(key)] = stringValue;
  }

  getAttribute(name) {
    return this.attributes[String(name)] ?? null;
  }

  addEventListener(type, handler, options = false) {
    const list = this.listeners.get(type) || [];
    list.push({ handler, capture: options === true || Boolean(options && options.capture) });
    this.listeners.set(type, list);
  }

  dispatchEvent(event) {
    event.target = event.target || this;
    event.currentTarget = null;
    event.defaultPrevented = false;
    event.cancelBubble = false;
    event.immediateStopped = false;
    event.preventDefault = () => { event.defaultPrevented = true; };
    event.stopPropagation = () => { event.cancelBubble = true; };
    event.stopImmediatePropagation = () => {
      event.cancelBubble = true;
      event.immediateStopped = true;
    };

    const path = [];
    let current = this;
    while (current) {
      path.unshift(current);
      current = current.parentElement;
    }

    for (const node of path) {
      const listeners = (node.listeners.get(event.type) || []).filter((item) => item.capture);
      for (const listener of listeners) {
        event.currentTarget = node;
        listener.handler(event);
        if (event.immediateStopped) return !event.defaultPrevented;
      }
      if (event.cancelBubble) return !event.defaultPrevented;
    }

    for (const node of path.reverse()) {
      const listeners = (node.listeners.get(event.type) || []).filter((item) => !item.capture);
      for (const listener of listeners) {
        event.currentTarget = node;
        listener.handler(event);
        if (event.immediateStopped) return !event.defaultPrevented;
      }
      if (event.cancelBubble) return !event.defaultPrevented;
    }

    return !event.defaultPrevented;
  }

  click() {
    if (this.disabled) return false;
    return this.dispatchEvent({ type: "click", isTrusted: true });
  }

  matches(selector) {
    return matchesSelector(this, selector);
  }

  closest(selector) {
    let current = this;
    while (current) {
      if (current.matches(selector)) return current;
      current = current.parentElement;
    }
    return null;
  }

  querySelectorAll(selector) {
    return querySelectorAll(this, selector);
  }

  querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }

  get outerHTML() {
    const tag = String(this.tagName || "div").toLowerCase();
    const attrs = Object.entries(this.attributes)
      .map(([name, value]) => value === "" ? name : name + '="' + String(value).replace(/"/g, "&quot;") + '"')
      .join(" ");
    const opening = "<" + tag + (attrs ? " " + attrs : "") + ">";
    if (["input", "br", "hr", "img", "meta", "link"].includes(tag)) return opening;
    const body = (this.textContent || "") + this.children.map(child => child.outerHTML || "").join("");
    return opening + body + "</" + tag + ">";
  }

  set innerHTML(value) {
    this.children = [];
    parseHtmlInto(this, String(value || ""));
  }

  get innerHTML() {
    return "";
  }
}

class TestDocument extends TestElement {
  constructor() {
    super("#document", null);
    this.ownerDocument = this;
    this.readyState = "complete";
    this.head = new TestElement("head", this);
    this.body = new TestElement("body", this);
    this.appendChild(this.head);
    this.appendChild(this.body);
  }

  createElement(tagName) {
    return new TestElement(tagName, this);
  }

  getElementById(id) {
    return walk(this).find((item) => item.id === id) || null;
  }
}

function walk(root) {
  const out = [];
  for (const child of root.children || []) {
    out.push(child);
    out.push(...walk(child));
  }
  return out;
}

function parseHtmlInto(parent, html) {
  const stack = [parent];
  const tokens = html.match(/<\/?[^>]+>|[^<]+/g) || [];
  const voidTags = new Set(["INPUT", "BR", "HR", "IMG", "META", "LINK"]);

  for (const token of tokens) {
    if (token.startsWith("</")) {
      if (stack.length > 1) stack.pop();
      continue;
    }

    if (token.startsWith("<")) {
      const tagMatch = token.match(/^<\s*([A-Za-z0-9-]+)/);
      if (!tagMatch) continue;
      const element = parent.ownerDocument.createElement(tagMatch[1]);
      const attrText = token.replace(/^<\s*[A-Za-z0-9-]+/, "").replace(/\/?\s*>$/, "");
      const attrRegex = /([:@A-Za-z0-9_-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+)))?/g;
      let attrMatch;
      while ((attrMatch = attrRegex.exec(attrText))) {
        const name = attrMatch[1];
        const value = attrMatch[2] ?? attrMatch[3] ?? attrMatch[4] ?? "";
        element.setAttribute(name, value);
      }
      stack[stack.length - 1].appendChild(element);
      if (!voidTags.has(element.tagName) && !token.endsWith("/>")) stack.push(element);
      continue;
    }

    const text = token.replace(/\s+/g, " ").trim();
    if (text) {
      const current = stack[stack.length - 1];
      current.textContent = (current.textContent + " " + text).trim();
    }
  }
}

function querySelectorAll(root, selector) {
  const selectors = String(selector).split(",").map((item) => item.trim()).filter(Boolean);
  const nodes = walk(root);
  return nodes.filter((node) => selectors.some((part) => matchesDescendantSelector(node, part)));
}

function matchesDescendantSelector(node, selector) {
  const parts = selector.split(/\s+/).filter(Boolean);
  if (parts.length === 0) return false;
  if (!matchesSelector(node, parts[parts.length - 1])) return false;

  let current = node.parentElement;
  for (let index = parts.length - 2; index >= 0; index -= 1) {
    while (current && !matchesSelector(current, parts[index])) {
      current = current.parentElement;
    }
    if (!current) return false;
    current = current.parentElement;
  }

  return true;
}

function matchesSelector(node, selector) {
  const alternatives = String(selector).split(",").map((item) => item.trim()).filter(Boolean);
  if (alternatives.length > 1) return alternatives.some((item) => matchesSelector(node, item));

  let rest = alternatives[0] || "";
  const tagMatch = rest.match(/^[A-Za-z0-9_-]+/);
  if (tagMatch) {
    if (node.tagName !== tagMatch[0].toUpperCase()) return false;
    rest = rest.slice(tagMatch[0].length);
  }

  const idMatches = [...rest.matchAll(/#([A-Za-z0-9_-]+)/g)];
  for (const match of idMatches) {
    if (node.id !== match[1]) return false;
  }

  const classMatches = [...rest.matchAll(/\.([A-Za-z0-9_-]+)/g)];
  for (const match of classMatches) {
    if (!node.classList.contains(match[1])) return false;
  }

  const attrMatches = [...rest.matchAll(/\[([^\]=~\^\$\*\|]+)(?:=(?:"([^"]*)"|'([^']*)'|([^\]]+)))?\]/g)];
  for (const match of attrMatches) {
    const attr = match[1].trim();
    const expected = match[2] ?? match[3] ?? (match[4] ? match[4].replace(/^['"]|['"]$/g, "") : undefined);
    const actual = node.getAttribute(attr);
    if (expected === undefined) {
      if (actual === null) return false;
    } else if (actual !== expected) {
      return false;
    }
  }

  return true;
}

function createHarness(options = {}) {
  const document = new TestDocument();
  const storage = new Map();
  const timers = [];
  const window = {
    document,
    listeners: new Map(),
    localStorage: {
      getItem(key) { return storage.has(String(key)) ? storage.get(String(key)) : null; },
      setItem(key, value) { storage.set(String(key), String(value)); },
      removeItem(key) { storage.delete(String(key)); },
    },
    setTimeout(callback) {
      timers.push(callback);
      return timers.length;
    },
    clearTimeout() {},
    setInterval() { return 1; },
    clearInterval() {},
    queueMicrotask(callback) { Promise.resolve().then(callback); },
    confirm() { return false; },
    navigator: { clipboard: { writeText: async () => {} } },
    crypto: webcrypto,
    TextEncoder,
    console: { ...console, debug() {} },
    CustomEvent: class CustomEvent {
      constructor(type, options) {
        this.type = type;
        this.detail = options && options.detail;
      }
    },
    Event: class Event {
      constructor(type, options) {
        this.type = type;
        this.bubbles = Boolean(options && options.bubbles);
      }
    },
  };
  if (options.tauri) {
    window.__TAURI__ = options.tauri;
  }
  window.addEventListener = (type, handler) => {
    const list = window.listeners.get(type) || [];
    list.push(handler);
    window.listeners.set(type, list);
  };
  window.dispatchEvent = (event) => {
    for (const handler of window.listeners.get(event.type) || []) handler(event);
    return true;
  };
  window.window = window;
  document.defaultView = window;

  const root = document.createElement("section");
  root.setAttribute("id", "kaspa-node");
  root.setAttribute("class", "page node-v6-root");
  root.setAttribute("data-kgw-tab", "kaspa-node");
  const shell = document.createElement("div");
  shell.setAttribute("class", "node-v6-shell");
  const tabs = document.createElement("div");
  tabs.setAttribute("class", "node-v6-network-tabs");
  for (const [net, label, active] of [
    ["mainnet", "Mainnet", true],
    ["testnet10", "Testnet 10", false],
    ["testnet13", "Testnet 13 آ· Experimental", false],
  ]) {
    const button = document.createElement("button");
    button.setAttribute("type", "button");
    button.setAttribute("class", "node-v6-network-tab" + (active ? " active" : ""));
    button.setAttribute("data-node-network-tab", net);
    button.setAttribute("data-net", net);
    button.textContent = label;
    tabs.appendChild(button);
  }
  const panels = document.createElement("div");
  panels.setAttribute("id", "nodeNetworkPanels");
  panels.setAttribute("class", "node-v6-network-panels");
  shell.appendChild(tabs);
  shell.appendChild(panels);
  root.appendChild(shell);
  document.body.appendChild(root);

  const sandbox = {
    window,
    document,
    console: window.console,
    navigator: window.navigator,
    localStorage: window.localStorage,
    setTimeout: window.setTimeout,
    clearTimeout: window.clearTimeout,
    setInterval: window.setInterval,
    clearInterval: window.clearInterval,
    queueMicrotask: window.queueMicrotask,
    crypto: window.crypto,
    TextEncoder: window.TextEncoder,
    CustomEvent: window.CustomEvent,
    Event: window.Event,
  };
  sandbox.globalThis = sandbox;

  const importPrelude = `
const applyStatusTone = (node, state) => { if (node) node.dataset.state = String(state || "").toLowerCase(); };
const renderStatusSummary = (node, text) => { if (node) node.textContent = String(text || ""); };
const NODE_ENDPOINTS = [["rpcListenEnabled","rpcListenHost","rpcListenPort","rpcListen",true],["rpcBorshEnabled","rpcBorshHost","rpcBorshPort","rpcListenBorsh",true],["rpcJsonEnabled","rpcJsonHost","rpcJsonPort","rpcListenJson",true],["listenEnabled","listenHost","listenPort","p2pListen",false],["externalIpEnabled","externalIpHost","externalIpPort","externalIp",false],["connectEnabled","connectHost","connectPort","connectPeers",false],["addPeerEnabled","addPeerHost","addPeerPort","addPeers",false]];
const NODE_MANAGED = { appDir:"Managed", configFile:"Unsupported", overrideParamsFile:"Unsupported", testnet:"Managed", netsuffix:"Managed", noGrpc:"Managed", rpcListenEnabled:"Managed" };
const NODE_REQUIRED = { logLevel:"info", asyncThreads:"16", ramScale:"1", rpcMaxClients:"16", outPeers:"8", maxInPeers:"32" };
const NODE_OPTIONAL = new Set(["uaComment","retentionDays","maxTrackedAddresses","perfMetricsInterval","rocksDbPreset","rocksDbCacheSize","rocksDbWalDir","logDir"]);
const NODE_DANGEROUS = { resetDb:"Deletes database", unsafeRpc:"Unsafe RPC", enableUnsyncedMining:"Unsynced mining" };
const nodeFieldEnabled = () => true;
const validateNodeForm = () => ({});
const renderFieldErrors = () => {};
const endpoint = (host, port) => String(host) + ":" + String(port);
const runtimePresentation = ({enabled, running, transition, error}) => ({ process: error ? "Failed" : transition === "starting" ? "Starting" : transition === "stopping" ? "Stopping" : running ? "Running" : "Stopped", processLabel: error ? "Node: Failed" : transition === "starting" ? "Node: Starting" : transition === "stopping" ? "Node: Stopping" : running ? "Node: Running" : "Node: Stopped", profile: enabled ? "Enabled" : "Disabled" });
const runtimeObservationSummary = () => "RPC/synchronization/mining: unknown";
const confirmUserAction = async () => true;
const renderSettingsTabs = (_scope, _key, groups) => groups.map(group => group[3]).join("");
const installSettingsLayout = () => {};
const decorateSettingsFields = () => {};
const revealSettingsField = () => {};
const setSettingFieldState = () => {};
const wasmNodeI18nText = (_key, fallback) => String(fallback ?? "");
const wasmNodeBackendInvoke = (command, payload = {}) => {
  const tauri = window.__TAURI__;
  const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
    ? tauri.core.invoke
    : tauri && tauri.tauri && typeof tauri.tauri.invoke === "function"
      ? tauri.tauri.invoke
      : window.__TAURI_INVOKE__;
  if (typeof invoke !== "function") return Promise.reject(new Error("Tauri invoke is not available"));
  return invoke(command, payload);
};
const wasmNodeIsolatedCompatInvoke = async (command, payload) => {
  try { return await wasmNodeBackendInvoke(command, payload); }
  catch (_) { return null; }
};
const wasmNodeSuperMegaIsolatedAdapterStatusPreviewV1 = (network) =>
  wasmNodeIsolatedCompatInvoke("rk_isolated_adapter_status_preview_v1", { network });
const wasmNodeFinalIsolatedAdapterStartV1 = (network, appDirName) =>
  wasmNodeIsolatedCompatInvoke("rk_final_isolated_adapter_start_v1", { network, appDirName });
const wasmNodeFinalIsolatedAdapterStatusV1 = (network) =>
  wasmNodeIsolatedCompatInvoke("rk_final_isolated_adapter_status_v1", { network });
const wasmNodeFinalIsolatedAdapterStopV1 = (network) =>
  wasmNodeIsolatedCompatInvoke("rk_final_isolated_adapter_stop_v1", { network });
const wasmNodeV66RuntimeFeaturePolicyV1 = (network) =>
  wasmNodeIsolatedCompatInvoke("rk_v66_runtime_feature_policy_v1", { network });
const wasmNodeV66IsolatedAdapterStartV1 = (network, appDirName) =>
  wasmNodeIsolatedCompatInvoke("rk_v66_isolated_adapter_start_v1", { network, appDirName });
const wasmNodeV66IsolatedAdapterStatusV1 = (network) =>
  wasmNodeIsolatedCompatInvoke("rk_v66_isolated_adapter_status_v1", { network });
const wasmNodeV66IsolatedAdapterStopV1 = (network) =>
  wasmNodeIsolatedCompatInvoke("rk_v66_isolated_adapter_stop_v1", { network });
const wasmNodeV67StartRuntime = wasmNodeV66IsolatedAdapterStartV1;
const wasmNodeV67StatusRuntime = wasmNodeV66IsolatedAdapterStatusV1;
const wasmNodeV67StopRuntime = wasmNodeV66IsolatedAdapterStopV1;
const wasmNodeV67RuntimeFeaturePolicy = wasmNodeV66RuntimeFeaturePolicyV1;
const wasmNodeNetworkProfiles = () => [
  { key: "mainnet", label: "Mainnet", testnet: false, netsuffix: "", enabledByDefault: true, runtime: "Official Rusty Kaspa" },
  { key: "testnet10", label: "Testnet 10", testnet: true, netsuffix: "10", enabledByDefault: true, runtime: "Official Rusty Kaspa" },
  { key: "testnet13", label: "Testnet 13", testnet: true, netsuffix: "13", enabledByDefault: false, experimental: true, runtime: "DAGKnight - Experimental" },
];
const wasmNodeNetworkPolicyKey = (net) => "kgw.node.network.enabled." + String(net || "unknown");
const wasmNodeNetworkProfile = (net) => wasmNodeNetworkProfiles().find((item) => item.key === String(net || "")) || null;
const wasmNodeNetworkEnabled = (net) => {
  const stored = localStorage.getItem(wasmNodeNetworkPolicyKey(net));
  if (stored === "1") return true;
  if (stored === "0") return false;
  const profile = wasmNodeNetworkProfile(net);
  return profile ? profile.enabledByDefault !== false : false;
};
const wasmNodeSetNetworkEnabled = (net, enabled) => localStorage.setItem(wasmNodeNetworkPolicyKey(net), enabled ? "1" : "0");
const wasmNodeNetworkPolicyMessage = (net) => {
  const profile = wasmNodeNetworkProfile(net);
  if (!profile) return "";
  if (profile.experimental) {
    return "Experimental network. Disabled by default and requires explicit opt-in."
      + (String(net) === "testnet13" ? " This Testnet13 build has no DNS seeders. For public sync, set a trusted Testnet13 peer in Connect or Add Peer." : "");
  }
  return String(profile.runtime) + ". RPC remains loopback-only and data is isolated per network.";
};
const wasmNodeById = (id) => document.getElementById(String(id || ""));
const wasmNodeEscapeHtml = (value) => String(value ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");
const wasmNodeElementId = (net, name) => "node-" + String(net || "") + "-" + String(name || "");
const wasmNodeValue = (net, name) => {
  const element = wasmNodeById(wasmNodeElementId(net, name));
  return element ? String(element.value || "").trim() : "";
};
const wasmNodeChecked = (net, name) => {
  const element = wasmNodeById(wasmNodeElementId(net, name));
  return Boolean(element && element.checked);
};
const wasmNodeCommandInlineStateKey = (net) => String(net || "mainnet");
const wasmNodeCommandInlineState = (net) => {
  const key = wasmNodeCommandInlineStateKey(net);
  window.__kgwNodeCommandComposerInlineR7 = window.__kgwNodeCommandComposerInlineR7 || {};
  window.__kgwNodeCommandComposerInlineR7[key] = window.__kgwNodeCommandComposerInlineR7[key] || {};
  return window.__kgwNodeCommandComposerInlineR7[key];
};
const wasmNodeCommandOptionEnabled = (net, name) => {
  if (Object.hasOwn(NODE_REQUIRED, name)) return true;
  const state = wasmNodeCommandInlineState(net);
  if (NODE_OPTIONAL.has(name)) return state[String(name)] === true;
  return state[String(name)] !== false;
};
const wasmNodeCommandShouldInclude = (net, name) => wasmNodeCommandOptionEnabled(net, name);
const wasmNodeCommandInlineToggle = (net, name) => {
  if (NODE_MANAGED[name] || Object.hasOwn(NODE_REQUIRED, name)
      || NODE_ENDPOINTS.some((row) => row[1] === name || row[2] === name)) return "";
  const enabled = wasmNodeCommandOptionEnabled(net, name);
  return '<input type="checkbox" class="kgw-command-option-checkbox-r9" data-node-command-option-toggle-r7="'
    + wasmNodeEscapeHtml(name) + '" data-net="' + wasmNodeEscapeHtml(net) + '" '
    + (enabled ? "checked" : "") + ' aria-label="Use ' + wasmNodeEscapeHtml(name)
    + '" title="Enable this optional setting">';
};
const wasmNodeRefreshInlineCommandToggles = (net) => {
  for (const el of document.querySelectorAll("[data-node-command-option-toggle-r7]")) {
    if (String(el.dataset.net || "") !== String(net || "")) continue;
    const name = String(el.dataset.nodeCommandOptionToggleR7 || "");
    const enabled = wasmNodeCommandOptionEnabled(net, name);
    el.checked = enabled;
    el.setAttribute("aria-label", enabled ? "Included in command" : "Excluded from command");
    el.setAttribute("title", enabled ? "Included in command" : "Excluded from command");
    el.classList.toggle("is-on", enabled);
    el.classList.toggle("is-off", !enabled);
  }
};
const wasmNodeToggleCommandOption = (net, name) => {
  const state = wasmNodeCommandInlineState(net);
  const key = String(name || "");
  state[key] = state[key] === false;
  wasmNodeRefreshInlineCommandToggles(net);
  return state[key];
};
const wasmNodeEffectiveForm = (net) => new Proxy({}, {
  get(_target, name) {
    if (typeof name !== "string") return undefined;
    const field = document.getElementById("node-" + String(net || "") + "-" + name);
    if (!field) return undefined;
    return field.type === "checkbox" ? Boolean(field.checked) : String(field.value ?? "");
  },
});
const wasmNodeEffectiveNumber = (net, name, fallback, integer = false) => {
  const values = wasmNodeEffectiveForm(net);
  if (!nodeFieldEnabled(name, values, wasmNodeCommandInlineState(net))) return fallback;
  const raw = String(values[name] ?? "").trim();
  if (!raw) return fallback;
  const number = Number(raw);
  if (!Number.isFinite(number) || (integer && !Number.isInteger(number))) {
    throw new Error(String(name) + " must be " + (integer ? "an integer" : "a finite number") + ".");
  }
  return number;
};
const wasmNodeEffectiveEndpoint = (net, enabledName, hostName, portName) => {
  const values = wasmNodeEffectiveForm(net);
  if (!values[enabledName]) return null;
  const host = String(values[hostName] ?? "").trim();
  const port = String(values[portName] ?? "").trim();
  if (!host && !port) throw new Error(String(enabledName) + " requires a host and port.");
  if (!host || !/^\\d+$/.test(port) || Number(port) < 1 || Number(port) > 65535) {
    throw new Error(String(enabledName) + " requires a host and a port between 1 and 65535.");
  }
  return endpoint(host, port);
};
const wasmNodeEffectiveNodeSettings = (net) => {
  const values = wasmNodeEffectiveForm(net);
  const options = wasmNodeCommandInlineState(net);
  const errors = validateNodeForm(values, options, net);
  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
  const profile = wasmNodeNetworkProfile(net);
  const grpc = wasmNodeEffectiveEndpoint(net, "rpcListenEnabled", "rpcListenHost", "rpcListenPort");
  if (!grpc) throw new Error("The managed desktop owner requires gRPC RPC to remain enabled.");
  if (values.configFile) throw new Error("--configfile is not supported by the managed desktop owner because network and database ownership must remain authoritative.");
  if (values.overrideParamsFile) throw new Error("--override-params-file is not supported because the desktop owns the selected network identity.");
  if (values.noLogFiles && nodeFieldEnabled("logDir", values, options) && values.logDir) {
    throw new Error("--logdir and --nologfiles cannot be used together.");
  }
  const connect = wasmNodeEffectiveEndpoint(net, "connectEnabled", "connectHost", "connectPort");
  const addPeer = wasmNodeEffectiveEndpoint(net, "addPeerEnabled", "addPeerHost", "addPeerPort");
  const logLevel = wasmNodeCommandShouldInclude(net, "logLevel") ? String(values.logLevel || "info") : "info";
  const ua = wasmNodeCommandShouldInclude(net, "uaComment") ? String(values.uaComment || "") : "";
  return {
    logLevel,
    asyncThreads: wasmNodeEffectiveNumber(net, "asyncThreads", 16, true),
    ramScale: wasmNodeEffectiveNumber(net, "ramScale", 1, false),
    yes: Boolean(values.yes),
    noLogFiles: Boolean(values.noLogFiles),
    sanity: Boolean(values.sanity),
    enableUnsyncedMining: Boolean(values.enableUnsyncedMining) && Boolean(profile && profile.testnet),
    p2pListen: wasmNodeEffectiveEndpoint(net, "listenEnabled", "listenHost", "listenPort"),
    externalIp: wasmNodeEffectiveEndpoint(net, "externalIpEnabled", "externalIpHost", "externalIpPort"),
    disableUpnp: Boolean(values.disableUpnp),
    disableDnsSeeding: Boolean(values.noDnsSeed),
    userAgentComments: ua ? [ua] : [],
    rpcListen: grpc,
    rpcListenBorsh: wasmNodeEffectiveEndpoint(net, "rpcBorshEnabled", "rpcBorshHost", "rpcBorshPort"),
    rpcListenJson: wasmNodeEffectiveEndpoint(net, "rpcJsonEnabled", "rpcJsonHost", "rpcJsonPort"),
    rpcMaxClients: wasmNodeEffectiveNumber(net, "rpcMaxClients", 16, true),
    unsafeRpc: Boolean(values.unsafeRpc),
    disableGrpc: Boolean(values.noGrpc),
    connectPeers: connect ? [connect] : [],
    addPeers: addPeer ? [addPeer] : [],
    outboundTarget: wasmNodeEffectiveNumber(net, "outPeers", 8, true),
    inboundLimit: wasmNodeEffectiveNumber(net, "maxInPeers", 32, true),
    utxoIndex: Boolean(values.utxoIndex),
    archival: Boolean(values.archival),
    resetDb: Boolean(values.resetDb),
    perfMetrics: Boolean(values.perfMetrics),
    maxTrackedAddresses: wasmNodeEffectiveNumber(net, "maxTrackedAddresses", 0, true),
    retentionPeriodDays: wasmNodeCommandShouldInclude(net, "retentionDays") && values.retentionDays
      ? wasmNodeEffectiveNumber(net, "retentionDays", null, false) : null,
    perfMetricsIntervalSec: wasmNodeEffectiveNumber(net, "perfMetricsInterval", 10, true),
    rocksDbPreset: wasmNodeCommandShouldInclude(net, "rocksDbPreset") ? values.rocksDbPreset || null : null,
    rocksDbCacheSize: nodeFieldEnabled("rocksDbCacheSize", values, options) && values.rocksDbCacheSize
      ? wasmNodeEffectiveNumber(net, "rocksDbCacheSize", null, true) : null,
    rocksDbWalDir: wasmNodeCommandShouldInclude(net, "rocksDbWalDir") ? values.rocksDbWalDir || null : null,
    overrideParamsFile: null,
    logDir: nodeFieldEnabled("logDir", values, options) ? values.logDir || null : null,
  };
};
const kgwNodeEffectiveNodeSettings = wasmNodeEffectiveNodeSettings;
const kgwNodeValidateForm = (net, _focus = false) =>
  validateNodeForm(wasmNodeEffectiveForm(net), wasmNodeCommandInlineState(net), net);
const wasmNodePreviewMessage = (net, message, error = false) => {
  const el = document.getElementById("node-" + String(net || "") + "-previewMessage");
  if (!el) return false;
  el.textContent = String(message || "");
  el.classList.toggle("kgw-field-error", Boolean(error));
  applyStatusTone(el, error ? "error" : String(message || "").startsWith("Validating") ? "validating" : "verified");
  return true;
};
const wasmNodeSyncDependencies = (net, locked = false) => {
  const values = wasmNodeEffectiveForm(net);
  const options = wasmNodeCommandInlineState(net);
  const panel = wasmNodeR51Panel(net);
  if (!panel) return false;
  for (const [name] of Object.entries(values)) {
    const field = document.getElementById("node-" + String(net || "") + "-" + name);
    if (!field) continue;
    const managed = NODE_MANAGED[name];
    const experimentalOnly = name === "enableUnsyncedMining" && net === "mainnet";
    const active = nodeFieldEnabled(name, values, options) && !experimentalOnly;
    field.disabled = Boolean(locked || (!active && name !== "appDir"));
    field.readOnly = Boolean(locked || managed);
    field.title = locked ? "Stop the bridge that owns this node to edit settings."
      : managed || (experimentalOnly ? "Available only on test networks." : !active ? "Enable the parent option to use this value." : field.value || "");
    const card = field.closest(".node-v6-card");
    if (card) card.classList.toggle("kgw-field-inactive", !active);
    const state = managed ? (/unsupported/i.test(managed) ? "Unsupported" : "Managed")
      : NODE_DANGEROUS[name] ? "Dangerous"
      : Object.hasOwn(NODE_REQUIRED, name) ? (String(values[name]) === NODE_REQUIRED[name] ? "KGW default" : "Custom value")
      : experimentalOnly ? "Test networks only"
      : !active ? "Not active" : "";
    setSettingFieldState(field, state);
  }
  for (const toggle of panel.querySelectorAll("[data-node-command-option-toggle-r7]")) {
    const name = toggle.dataset.nodeCommandOptionToggleR7;
    toggle.disabled = Boolean(locked || (name === "logDir" && values.noLogFiles)
      || (name === "perfMetricsInterval" && !values.perfMetrics)
      || (name === "rocksDbCacheSize" && (!options.rocksDbPreset || values.rocksDbPreset !== "hdd")));
  }
  decorateSettingsFields(panel);
  return true;
};
const wasmNodePanelStartFromMonitor = (net) => {
  const panel = wasmNodeR51Panel(net);
  const start = panel && panel.querySelector('[data-node-action="start"]');
  if (!start) return false;
  if (start.disabled) {
    const settings = panel.querySelector('[data-node-inner-tab="settings"]');
    if (settings) settings.click();
    wasmNodePreviewMessage(net, start.title || "Check the profile and settings before starting.", true);
  } else {
    start.click();
  }
  return true;
};
const kgwNodePreviewMessage = wasmNodePreviewMessage;
const panelStartFromMonitor = wasmNodePanelStartFromMonitor;
const kgwNodeRequireValidSettings = (net) => {
  const errors = kgwNodeValidateForm(net, true);
  if (Object.keys(errors).length) throw new Error(Object.values(errors)[0]);
  kgwNodeEffectiveNodeSettings(net);
};
const wasmNodeRuntimeArgs = (net, command) => {
  if (command === "kgw_kgw_apply_node_settings_v1") {
    const preview = document.getElementById("node-" + String(net || "") + "-commandPreview");
    return {
      network: net,
      nodeKind: "integrated-as-daemon",
      bridgeKind: "disable",
      nodeCommandPreview: preview ? String(preview.value || "") : "",
      bridgeCommandPreview: "",
      effectiveNodeSettings: wasmNodeEffectiveNodeSettings(net),
      runtimeRole: "node",
      experimentalNetworkOptIn: String(net) === "testnet13" && wasmNodeNetworkEnabled(net),
    };
  }
  if (["kgw_kgw_disable_network_v1", "kgw_runtime_owner_status_v1", "kgw_kgw_runtime_logs_v1"].includes(command)) {
    return { network: net, runtimeRole: "node" };
  }
  return { network: net };
};
const wasmNodeCommandOptionsKey = () => "__kgwNodeCommandOptionsR38C";
const wasmNodeReadCommandOptions = (net) => {
  const state = {};
  const root = document.getElementById("kaspa-node");
  if (!root) return state;
  for (const item of root.querySelectorAll("[data-node-command-option-toggle-r7]")) {
    if (String(item.dataset.net || "") !== String(net || "")) continue;
    const name = String(item.dataset.nodeCommandOptionToggleR7 || "");
    if (name) state[name] = Boolean(item.checked);
  }
  return state;
};
const wasmNodeApplyCommandOptions = (net, values) => {
  const result = { applied: false, count: 0 };
  const commandOptions = values && values[wasmNodeCommandOptionsKey()];
  if (!commandOptions || typeof commandOptions !== "object") return result;
  const state = wasmNodeCommandInlineState(net);
  for (const [name, enabled] of Object.entries(commandOptions)) {
    state[String(name)] = Boolean(enabled)
      && (!NODE_OPTIONAL.has(name) || Boolean(wasmNodeValue(net, name)));
    result.count += 1;
  }
  wasmNodeRefreshInlineCommandToggles(net);
  result.applied = true;
  return result;
};
const wasmNodeR51Keys = () => wasmNodeNetworkProfiles().map((item) => item.key);
const wasmNodeR51Panel = (net) =>
  document.querySelector('[data-node-network-panel="' + String(net || "") + '"]');
const wasmNodeR51Fields = (net) => {
  const panel = wasmNodeR51Panel(net);
  if (!panel) return [];
  const prefix = "node-" + String(net || "") + "-";
  return Array.from(panel.querySelectorAll("input, select, textarea")).filter((field) => {
    if (!field.id || !field.id.startsWith(prefix)) return false;
    if (field.id.endsWith("-commandPreview") || field.id.endsWith("-logOutput")) return false;
    return !field.closest(".node-v6-log-toolbar");
  });
};
const wasmNodeR51ReadSettings = (net) => {
  const values = {};
  values[wasmNodeCommandOptionsKey()] = wasmNodeReadCommandOptions(net);
  const prefix = "node-" + String(net || "") + "-";
  for (const field of wasmNodeR51Fields(net)) {
    const name = String(field.id || "").slice(prefix.length);
    if (!field.id || NODE_MANAGED[name]) continue;
    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }
  return values;
};
const wasmNodeR51WriteSettings = (net, values) => {
  const result = { applied: false, commandOptionsApplied: false, commandOptionsCount: 0 };
  if (!values || typeof values !== "object") return result;
  const prefix = "node-" + String(net || "") + "-";
  for (const field of wasmNodeR51Fields(net)) {
    const name = String(field.id || "").slice(prefix.length);
    if (!field.id || NODE_MANAGED[name]) continue;
    const item = values[field.id];
    if (!item) continue;
    if (field.type === "checkbox") field.checked = Boolean(item.checked);
    else if (Object.hasOwn(item, "value")) field.value = String(item.value ?? "");
    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
  }
  const command = wasmNodeApplyCommandOptions(net, values);
  result.commandOptionsApplied = Boolean(command && command.applied);
  result.commandOptionsCount = Number(command && command.count || 0);
  result.applied = true;
  return result;
};
const wasmNodeR51StoragePrefix = "kgw.node.direct.v51.";
const wasmNodeR51Store = (key, value) =>
  localStorage.setItem(wasmNodeR51StoragePrefix + String(key || ""), JSON.stringify(value));
const wasmNodeR51Load = (key) => {
  try { return JSON.parse(localStorage.getItem(wasmNodeR51StoragePrefix + String(key || "")) || "null"); }
  catch { return null; }
};
const wasmNodeR51CaptureFactoryDefaults = () => {
  for (const net of wasmNodeR51Keys()) {
    if (!wasmNodeR51Load("factory:" + net)) {
      wasmNodeR51Store("factory:" + net, wasmNodeR51ReadSettings(net));
    }
  }
};
const wasmNodeR51LoadSavedSettings = () => {
  const applied = [];
  for (const net of wasmNodeR51Keys()) {
    const saved = wasmNodeR51Load("saved:" + net);
    if (!saved) continue;
    const result = wasmNodeR51WriteSettings(net, saved);
    applied.push({
      net,
      commandOptionsApplied: Boolean(result.commandOptionsApplied),
      commandOptionsCount: Number(result.commandOptionsCount || 0),
    });
  }
  return applied;
};
const wasmNodeR51Summary = (values) => {
  const source = values && typeof values === "object" ? values : {};
  const structured = source.__kgwBridgeStructuredInstancesR26B;
  return {
    keyCount: Object.keys(source).length,
    checkboxCount: Object.values(source).filter((item) => item && item.type === "checkbox").length,
    valueCount: Object.values(source).filter((item) => item && item.type === "value").length,
    structuredInstanceCount: structured && Array.isArray(structured.instances) ? structured.instances.length : 0,
    hasActiveStructuredInstance: Boolean(source.__kgwBridgeActiveInstanceR26B),
  };
};
const wasmNodeR51PersistAction = (net, kind) => {
  const values = wasmNodeR51ReadSettings(net);
  const storageKey = kind + ":" + String(net || "");
  wasmNodeR51Store(storageKey, values);
  const persisted = wasmNodeR51Load(storageKey);
  return { ...wasmNodeR51Summary(values), storageKey, persisted: Boolean(persisted), persistedKeyCount: persisted && typeof persisted === "object" ? Object.keys(persisted).length : 0 };
};
const wasmNodeR51SaveSettingsAction = (net) => wasmNodeR51PersistAction(net, "saved");
const wasmNodeR51SetDefaultsAction = (net) => wasmNodeR51PersistAction(net, "default");
const kgwNodeR51ReadSettings = (net) => wasmNodeR51ReadSettings(net);
const kgwNodeR51Load = (key) => wasmNodeR51Load(key);
const kgwNodeR51SaveSettings = (net) => {
  kgwNodeRequireValidSettings(net);
  return wasmNodeR51SaveSettingsAction(net);
};
const kgwNodeR51SetAsDefaults = (net) => {
  kgwNodeRequireValidSettings(net);
  return wasmNodeR51SetDefaultsAction(net);
};
const wasmNodeR51RestoreDefaultsAction = (net) => {
  const defaults = wasmNodeR51Load("default:" + net) || wasmNodeR51Load("factory:" + net);
  return {
    hasDefaults: Boolean(defaults),
    defaultKeyCount: defaults && typeof defaults === "object" ? Object.keys(defaults).length : 0,
    writeResult: wasmNodeR51WriteSettings(net, defaults),
  };
};
const wasmNodeCardInput = (net, name, label, value = "", placeholder = "", span2 = false, toggle = "") =>
  '\\n    <div class="node-v6-card' + (span2 ? ' span2' : '') + '">'
  + '\\n      <span class="kgw-command-option-title-row-r8e">'
  + '\\n        ' + toggle
  + '\\n        <label for="' + wasmNodeElementId(net, name) + '" class="kgw-command-option-title-text-r8e">' + wasmNodeEscapeHtml(label) + '</label>'
  + '\\n      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->'
  + '\\n      <input id="' + wasmNodeElementId(net, name) + '" data-testid="kgw-node-field-' + wasmNodeEscapeHtml(net) + '-' + wasmNodeEscapeHtml(name) + '" type="text" value="' + wasmNodeEscapeHtml(value) + '" placeholder="' + wasmNodeEscapeHtml(placeholder) + '">'
  + '\\n    </div>';
const wasmNodeCardSelect = (net, name, label, options, value = "", span2 = false, toggle = "") => {
  const opts = Array.from(options || [], (item) => String(item ?? "")).map((item) => {
    const selected = item === String(value ?? "") ? ' selected' : '';
    return '<option value="' + wasmNodeEscapeHtml(item) + '"' + selected + '>' + wasmNodeEscapeHtml(item || "not set") + '</option>';
  }).join('');
  return '\\n    <div class="node-v6-card' + (span2 ? ' span2' : '') + '">'
    + '\\n      <span class="kgw-command-option-title-row-r8e">'
    + '\\n        ' + toggle
    + '\\n        <label for="' + wasmNodeElementId(net, name) + '" class="kgw-command-option-title-text-r8e">' + wasmNodeEscapeHtml(label) + '</label>'
    + '\\n      </span> <!-- KGW_NODE_COMMAND_COMPOSER_INLINE_SWITCH_LAYOUT_R8E -->'
    + '\\n      <select id="' + wasmNodeElementId(net, name) + '" data-testid="kgw-node-field-' + wasmNodeEscapeHtml(net) + '-' + wasmNodeEscapeHtml(name) + '">' + opts + '</select>'
    + '\\n    </div>';
};
const wasmNodeCardCheck = (net, name, label, checked = false, span2 = false) =>
  '\\n    <label class="node-v6-card check' + (span2 ? ' span2' : '') + '">'
  + '\\n      <input id="' + wasmNodeElementId(net, name) + '" data-testid="kgw-node-field-' + wasmNodeEscapeHtml(net) + '-' + wasmNodeEscapeHtml(name) + '" type="checkbox"' + (checked ? ' checked' : '') + '>'
  + '\\n      <span>' + wasmNodeEscapeHtml(label) + '</span>'
  + '\\n    </label>';
const wasmNodeNormalizeInnerTab = (value) => value === "settings" || value === "log" ? value : "log";
const wasmNodeResolveInnerTab = (net) => {
  try { return wasmNodeNormalizeInnerTab(localStorage.getItem("kgw.node.innerTab." + String(net || "unknown"))); }
  catch { return "log"; }
};
const wasmNodeSaveInnerTab = (net, selected) => {
  const normalized = wasmNodeNormalizeInnerTab(selected);
  try { localStorage.setItem("kgw.node.innerTab." + String(net || "unknown"), normalized); } catch {}
  return normalized;
};
const wasmNodeNormalizeNetwork = (value) => {
  const normalized = String(value ?? "").trim();
  return ["mainnet", "testnet10", "testnet13"].includes(normalized) ? normalized : "";
};
const wasmNodeReadLastNetwork = () => {
  try { return wasmNodeNormalizeNetwork(localStorage.getItem("kgw.node.lastNetwork")); } catch { return ""; }
};
const wasmNodeSaveLastNetwork = (net) => {
  const normalized = wasmNodeNormalizeNetwork(net);
  if (!normalized) return "";
  try { localStorage.setItem("kgw.node.lastNetwork", normalized); } catch {}
  return normalized;
};
const wasmNodeStringifyRuntimeResult = (result) => {
  if (result == null) return "No response";
  if (typeof result === "string") return result;
  try { return JSON.stringify(result); } catch { return String(result); }
};
const wasmNodeNormalizeRuntimeError = (error) => {
  if (error == null) return "Unknown backend error";
  if (typeof error === "string") return error;
  if (error.message) return error.message;
  try { return JSON.stringify(error); } catch { return String(error); }
};
const wasmNodeParseRuntimeFields = (result) => {
  const raw = wasmNodeStringifyRuntimeResult(result);
  const fields = {};
  for (const part of raw.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;
    const key = part.slice(0, index).trim();
    const value = part.slice(index + 1).trim();
    if (key) fields[key] = value;
  }
  return fields;
};
const wasmNodeRuntimeEvidence = (result) => {
  const text = wasmNodeStringifyRuntimeResult(result);
  const fields = wasmNodeParseRuntimeFields(text);
  const pid = String(fields.pid || "").trim();
  return {
    text,
    fields,
    pid,
    owner: fields.owner || fields.source || "self-worker",
    role: fields.role || fields.runtime_role || fields.runtimeRole || "node",
    state: fields.runtime_state || fields.runtimeState || (pid ? "running" : ""),
  };
};
const wasmNodeAssertStartEvidence = (net, result) => {
  const evidence = wasmNodeRuntimeEvidence(result);
  const responseNetwork = String(evidence.fields.network || "").trim();
  if (/start_blocked=true|start_allowed=false/i.test(evidence.text)) throw new Error(evidence.text);
  if (responseNetwork && responseNetwork !== String(net || "")) throw new Error("Backend start response used the wrong network: " + evidence.text);
  if (!/^[0-9]+$/.test(evidence.pid)) throw new Error("Backend start response did not include process ID evidence: " + evidence.text);
  if (String(evidence.fields.readiness || "").toUpperCase() !== "READY") throw new Error("Backend Start did not provide role readiness evidence: " + evidence.text);
  return evidence;
};
const wasmNodeRuntimeIsRunning = (text) => {
  const value = String(text || "");
  return /readiness=READY/i.test(value)
    && (/running=true/.test(value) || /node_running=true/.test(value) || /official_core_running=true/.test(value));
};
const wasmNodeRuntimeErrorFromStatus = (text) => {
  const fields = wasmNodeParseRuntimeFields(text);
  const error = String(fields.runtime_error || fields.runtimeError || "").trim();
  return error && error.toLowerCase() !== "none" ? error : "";
};
const wasmNodeLogAutoScrollEnabled = (net) =>
  localStorage.getItem("kgw.node.log.autoscroll." + String(net || "")) !== "0";
const wasmNodeSetLogAutoScroll = (net, enabled) => {
  localStorage.setItem("kgw.node.log.autoscroll." + String(net || ""), enabled ? "1" : "0");
  const out = document.getElementById("node-" + String(net || "") + "-logOutput");
  if (enabled && out) out.scrollTop = out.scrollHeight;
};
const wasmNodeClearRawLogBuffer = (net, _role = "node") => {
  const key = String(net || "");
  const out = document.getElementById("node-" + key + "-logOutput");
  if (out) out.textContent = "";
  const empty = document.getElementById("node-" + key + "-logEmpty");
  if (empty) empty.hidden = false;
  return Boolean(out);
};
const wasmNodeInstallLogAutoScrollControls = () => {
  for (const profile of wasmNodeNetworkProfiles()) {
    const net = profile.key;
    const out = document.getElementById("node-" + net + "-logOutput");
    if (!out) continue;
    const controlId = "node-" + net + "-logAutoScrollR27";
    if (document.getElementById(controlId)) continue;
    const label = document.createElement("label");
    label.className = "kgw-log-autoscroll-toggle";
    label.setAttribute("data-kgw-log-autoscroll", "node");
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.id = controlId;
    checkbox.checked = wasmNodeLogAutoScrollEnabled(net);
    checkbox.addEventListener("change", () => wasmNodeSetLogAutoScroll(net, checkbox.checked));
    const span = document.createElement("span");
    span.textContent = "Auto-scroll";
    label.appendChild(checkbox);
    label.appendChild(span);
    const panel = out.closest('[data-node-inner-panel="log"]') || out.parentElement;
    const toolbar = panel && panel.querySelector(".node-v6-log-toolbar");
    if (toolbar) toolbar.appendChild(label);
  }
};
const wasmNodeClipboardCharacterCount = (text) => Array.from(String(text ?? "")).length;
const wasmNodeClipboardLineCount = (text) => {
  const value = String(text ?? "");
  return value ? value.split("\\n").length : 0;
};
const wasmNodeNormalizeClipboardLineEndings = (text) =>
  String(text ?? "").replace(/\\r\\n/g, "\\n").replace(/\\r/g, "\\n").replace(/\\n/g, "\\r\\n");
const wasmNodeClipboardSafeError = (error) => {
  const text = String(error && error.message ? error.message : error || "clipboard write failed")
    .replace(/[\\r\\n\\t]+/g, " ")
    .trim();
  if (/(secret|token|private|mnemonic|wallet|address)/i.test(text)) {
    return "clipboard write failed with a sensitive error";
  }
  return (text || "clipboard write failed").slice(0, 360);
};
const wasmNodeClipboardPlaceholderText = (net) => {
  const key = String(net || "");
  const labels = { mainnet: "Mainnet", testnet10: "Testnet 10", testnet13: "Testnet 13" };
  return String(labels[key] || key) + " log is empty.";
};
const wasmNodeCopyLogStatus = (net, message, state = "info") => {
  const out = document.getElementById("node-" + net + "-logOutput");
  const panel = out && out.closest && out.closest('[data-node-inner-panel="log"]');
  const toolbar = panel && panel.querySelector(".node-v6-log-toolbar");
  if (!toolbar) return false;
  let status = toolbar.querySelector('.kgw-copy-log-status-v1[data-net="' + net + '"]');
  if (!status) {
    status = document.createElement("span");
    status.setAttribute("class", "kgw-copy-log-status-v1");
    status.dataset.net = net;
    status.setAttribute("data-net", net);
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    toolbar.appendChild(status);
  }
  status.textContent = String(message || "");
  status.dataset.state = String(state || "info");
  status.hidden = !status.textContent;
  return true;
};
const wasmNodeCopyLogFailure = (net, button, error, details = {}) => {
  const safeError = wasmNodeClipboardSafeError(error);
  wasmNodeCopyLogStatus(net, safeError, "error");
  if (button) button.textContent = "Copy failed";
  wasmNodeStartTraceFrontend("frontend.copy_log_failed", {
    network: net,
    action: "copy-log",
    result: "error",
    details: { ...(details || {}), safeError, userFeedbackDisplayed: true },
  });
  return false;
};
const wasmNodeDispatchClipboardWrite = async (net, text, metadata = {}) => {
  const resolved = wasmNodeResolvePublicTauriInvoke();
  if (typeof resolved.invoke !== "function") {
    throw new Error("Tauri invoke API is not available. Expected window.__TAURI__.core.invoke from Tauri 2 with withGlobalTauri enabled.");
  }
  wasmNodeStartTraceFrontend("frontend.copy_log_dispatched", {
    network: net,
    action: "copy-log",
    result: "dispatched",
    details: {
      commandName: "kgw_copy_text_to_clipboard_v1",
      implementation: "native-tauri-command",
      runtimeRole: metadata.runtimeRole || "node",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || "",
      payloadFieldCount: 7,
    },
  });
  return await resolved.invoke("kgw_copy_text_to_clipboard_v1", {
    network: net,
    runtimeRole: metadata.runtimeRole || "node",
    bridgeInstanceId: metadata.bridgeInstanceId || "",
    text,
    characterCount: metadata.characterCount,
    lineCount: metadata.lineCount,
    sha256: metadata.sha256 || "",
  });
};
const wasmNodeHandleCopyLog = async (net, button) => {
  const copyNetwork = String(net || "").trim();
  const root = document.getElementById("kaspa-node");
  const activeNetwork = wasmNodeTraceActiveNetwork(root);
  const belongsToLiveNodeMonitor = Boolean(button && button.closest && button.closest('[data-node-inner-panel="log"]'));
  if (!copyNetwork) {
    return wasmNodeCopyLogFailure(net, button, "Copy Log could not resolve the active network.", {
      reason: "missing-network", activeNetwork, belongsToLiveNodeMonitor,
    });
  }
  wasmNodeStartTraceFrontend("frontend.copy_log_network_resolved", {
    network: copyNetwork,
    action: "copy-log",
    result: activeNetwork && activeNetwork !== copyNetwork ? "error" : "ok",
    details: { activeNetwork, buttonNetwork: copyNetwork, belongsToLiveNodeMonitor },
  });
  if (activeNetwork && activeNetwork !== copyNetwork) {
    return wasmNodeCopyLogFailure(copyNetwork, button, "Copy Log network mismatch; active network changed before copy started.", {
      reason: "network-mismatch", activeNetwork, buttonNetwork: copyNetwork, belongsToLiveNodeMonitor,
    });
  }
  if (button && button.dataset && button.dataset.kgwCopyLogInFlightV1 === "1") {
    return wasmNodeCopyLogFailure(copyNetwork, button, "Copy Log is already in progress for this network.", {
      reason: "duplicate-copy", activeNetwork, belongsToLiveNodeMonitor,
    });
  }
  const originalDisabled = Boolean(button && button.disabled);
  if (button) {
    button.dataset.kgwCopyLogInFlightV1 = "1";
    button.disabled = true;
  }
  let ok = false;
  try {
    const out = document.getElementById("node-" + copyNetwork + "-logOutput");
    const tag = String(out && out.tagName || "").toUpperCase();
    const rawText = String(out ? ((tag === "TEXTAREA" || tag === "INPUT") ? out.value : out.textContent) : "");
    const normalizedText = wasmNodeNormalizeClipboardLineEndings(rawText);
    const isPlaceholder = rawText.trim() === wasmNodeClipboardPlaceholderText(copyNetwork).trim();
    const characterCount = wasmNodeClipboardCharacterCount(normalizedText);
    const lineCount = wasmNodeClipboardLineCount(normalizedText);
    if (!out || isPlaceholder || !normalizedText.trim()) {
      wasmNodeStartTraceFrontend("frontend.copy_log_content_prepared", {
        network: copyNetwork, action: "copy-log", result: "error",
        details: { rawLogBufferSelected: Boolean(out), placeholderRejected: Boolean(isPlaceholder), runtimeRole: "node", bridgeInstanceId: "", characterCount, lineCount, sha256: "" },
      });
      throw new Error("Copy Log requires a non-empty raw log buffer for " + copyNetwork + ".");
    }
    const bytes = new TextEncoder().encode(normalizedText);
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    const sha256 = Array.from(new Uint8Array(digest)).map((value) => value.toString(16).padStart(2, "0")).join("");
    const metadata = { runtimeRole: "node", bridgeInstanceId: "", characterCount, lineCount, sha256 };
    wasmNodeStartTraceFrontend("frontend.copy_log_content_prepared", {
      network: copyNetwork, action: "copy-log", result: "ok",
      details: { rawLogBufferSelected: true, placeholderRejected: false, ...metadata },
    });
    await wasmNodeDispatchClipboardWrite(copyNetwork, normalizedText, metadata);
    if (button) button.textContent = "Copied";
    wasmNodeCopyLogStatus(copyNetwork, "Copied", "ok");
    wasmNodeStartTraceFrontend("frontend.copy_log_succeeded", {
      network: copyNetwork, action: "copy-log", result: "ok",
      details: { ...metadata, userFeedbackDisplayed: true },
    });
    ok = true;
  } catch (error) {
    wasmNodeCopyLogFailure(copyNetwork, button, error, { activeNetwork, belongsToLiveNodeMonitor });
  } finally {
    if (button) {
      button.disabled = originalDisabled;
      delete button.dataset.kgwCopyLogInFlightV1;
    }
  }
  return ok;
};
const wasmNodeHandleLogAction = async (action, net, button) => {
  const normalizedAction = String(action || "");
  const network = String(net || "");
  const out = document.getElementById("node-" + network + "-logOutput");
  if (!out && normalizedAction !== "copy-log") return false;
  if (normalizedAction === "copy-log") return await wasmNodeHandleCopyLog(network, button);
  if (normalizedAction === "clear-log") {
    wasmNodeClearRawLogBuffer(network, "node");
    Promise.resolve(kgwNodeDispatchRuntimeLogClearV1(network, "node")).catch(() => {});
    if (button) button.textContent = "Deleted";
  }
  return true;
};
const kgwNodeHandleLogActionV29 = wasmNodeHandleLogAction;
const wasmNodeStartTraceTauriShape = (adapterName = "") => {
  const tauri = window.__TAURI__;
  const keys = (value) => value && typeof value === "object" ? Object.keys(value).sort().slice(0, 24) : [];
  return { adapter: String(adapterName || "missing"), hasGlobalTauri: Boolean(tauri), globalKeys: keys(tauri), coreKeys: keys(tauri && tauri.core), tauriKeys: keys(tauri && tauri.tauri), hasCoreInvoke: typeof (tauri && tauri.core && tauri.core.invoke) === "function", hasTauriInvoke: typeof (tauri && tauri.tauri && tauri.tauri.invoke) === "function", hasRootInvoke: typeof (tauri && tauri.invoke) === "function", expectedConfiguredGlobal: "window.__TAURI__.core.invoke" };
};
const wasmNodeResolvePublicTauriInvoke = () => {
  const tauri = window.__TAURI__;
  for (const [adapter, owner, invoke] of [["window.__TAURI__.core.invoke", tauri && tauri.core, tauri && tauri.core && tauri.core.invoke], ["window.__TAURI__.tauri.invoke", tauri && tauri.tauri, tauri && tauri.tauri && tauri.tauri.invoke], ["window.__TAURI__.invoke", tauri, tauri && tauri.invoke]]) {
    if (typeof invoke === "function") return { adapter, invoke: invoke.bind(owner), shape: wasmNodeStartTraceTauriShape(adapter) };
  }
  return { adapter: "missing", invoke: null, shape: wasmNodeStartTraceTauriShape("missing") };
};
const wasmNodeStartTraceFrontend = (stage, options = {}) => {
  const resolved = wasmNodeResolvePublicTauriInvoke();
  if (typeof resolved.invoke !== "function") return false;
  const blocked = /(secret|token|private|mnemonic|wallet|address|commandPreview|completeCommand|arguments|appDir|path|rpcEndpoint|stratum)/i;
  const safeText = (value, fallback = "") => String(value ?? "").replace(/[\\r\\n\\t]+/g, " ").trim().slice(0, 220) || fallback;
  const sanitize = (value) => Object.fromEntries(Object.entries(value && typeof value === "object" ? value : {}).map(([key, item]) => [key, blocked.test(key) ? "[redacted]" : Array.isArray(item) ? item.slice(0,24).map(v => safeText(v)) : item && typeof item === "object" ? sanitize(item) : typeof item === "boolean" || typeof item === "number" ? item : safeText(item)]));
  const details = sanitize({ ...(options.details && typeof options.details === "object" ? options.details : {}), invokeAdapter: resolved.adapter });
  Promise.resolve(resolved.invoke("kgw_start_trace_frontend_v1", { stage: safeText(stage, "frontend.unknown"), network: safeText(options.network || options.net, "unknown"), action: safeText(options.action, "unknown"), result: safeText(options.result, "observed"), details: JSON.stringify(details) })).catch(() => {});
  return true;
};
const kgwResolvePublicTauriInvokeR1 = wasmNodeResolvePublicTauriInvoke;
const kgwStartTraceFrontendR1 = wasmNodeStartTraceFrontend;
const kgwNodeDispatchClipboardWriteV1 = wasmNodeDispatchClipboardWrite;
const kgwNodeCopyLogFailureV1 = wasmNodeCopyLogFailure;
const kgwNodeClearRawLogBufferV1 = wasmNodeClearRawLogBuffer;
const kgwNodeDispatchRuntimeLogClearV1 = async (net, role = "node") => {
  const resolved = wasmNodeResolvePublicTauriInvoke();
  if (typeof resolved.invoke !== "function") return null;
  return await resolved.invoke("kgw_kgw_runtime_clear_logs_v1", {
    network: String(net || ""),
    runtimeRole: String(role || "node"),
  });
};
const wasmNodeTraceActiveNetwork = (root = document.getElementById("kaspa-node")) => {
  if (!root) return "";
  const panel = Array.from(root.querySelectorAll("[data-node-network-panel]")).find((item) =>
    !item.hidden && (item.classList.contains("active") || item.dataset.active === "true"));
  if (panel && panel.dataset.nodeNetworkPanel) return panel.dataset.nodeNetworkPanel;
  const tab = Array.from(root.querySelectorAll("[data-node-network-tab]")).find((item) =>
    item.classList.contains("active") || item.getAttribute("aria-selected") === "true" || item.dataset.active === "true");
  return tab && tab.dataset.nodeNetworkTab || "";
};
const wasmNodeTraceNetworkFromElement = (element, root = document.getElementById("kaspa-node")) => {
  const carrier = element && element.closest &&
    element.closest("[data-net], [data-network], [data-node-network-panel], [data-node-inner-panel]");
  const raw = [
    element && element.dataset && element.dataset.net,
    element && element.dataset && element.dataset.network,
    carrier && carrier.dataset && carrier.dataset.net,
    carrier && carrier.dataset && carrier.dataset.network,
    carrier && carrier.dataset && carrier.dataset.nodeNetworkPanel,
    carrier && carrier.id,
    carrier && carrier.className,
  ].filter(Boolean).join(" ").toLowerCase();
  if (raw.includes("testnet13") || raw.includes("tn13")) return "testnet13";
  if (raw.includes("testnet10") || raw.includes("tn10")) return "testnet10";
  if (raw.includes("mainnet")) return "mainnet";
  return wasmNodeTraceActiveNetwork(root) || "mainnet";
};
const wasmNodeTraceStartButtonState = (net) => {
  const panel = document.querySelector('[data-node-network-panel="' + net + '"]');
  const start = panel && panel.querySelector('[data-node-action="start"][data-net="' + net + '"]');
  const stop = panel && panel.querySelector('[data-node-action="stop"][data-net="' + net + '"]');
  return { startRendered: Boolean(start), startDisabled: Boolean(start && start.disabled), stopRendered: Boolean(stop), stopDisabled: Boolean(stop && stop.disabled) };
};
const wasmNodeInstallStartTraceDocumentClickObserver = (root) => {
  if (window.__kgwStartTraceDocumentClickObserverR1 === true) return false;
  window.__kgwStartTraceDocumentClickObserverR1 = true;
  document.addEventListener("click", (event) => {
    const button = event.target && event.target.closest && event.target.closest("[data-node-action]");
    const nodeRoot = root || document.getElementById("kaspa-node");
    if (!button || !nodeRoot || !nodeRoot.contains(button)) return;
    const action = String(button.dataset.nodeAction || "").trim();
    const network = wasmNodeTraceNetworkFromElement(button, nodeRoot);
    const activeNetwork = wasmNodeTraceActiveNetwork(nodeRoot);
    const belongsToSettings = Boolean(button.closest('[data-node-inner-panel="settings"]'));
    const belongsToLiveNodeMonitor = Boolean(button.closest('[data-node-inner-panel="log"]'));
    if (action === "copy-log") {
      wasmNodeStartTraceFrontend("frontend.copy_log_click_observed", {
        network,
        action: "copy-log",
        result: "observed",
        details: { trusted: Boolean(event && event.isTrusted), belongsToSettings, belongsToLiveNodeMonitor, selectedNetwork: activeNetwork, buttonDisabled: Boolean(button.disabled), inFlight: button.dataset.kgwCopyLogInFlightV1 === "1" },
      });
      return;
    }
    if (action !== "start" && action !== "stop") return;
    wasmNodeStartTraceFrontend("frontend.capture_click_observed", {
      network,
      action,
      result: "observed",
      details: { trusted: Boolean(event && event.isTrusted), belongsToSettings, belongsToLiveNodeMonitor, selectedNetwork: activeNetwork, buttonDisabled: Boolean(button.disabled), buttonAction: action },
    });
  }, true);
  return true;
};
const wasmNodeTraceRenderedStartControls = (root) => {
  for (const net of ["mainnet", "testnet10", "testnet13"]) {
    const settingsPanel = root && root.querySelector('[data-node-network-panel="' + net + '"] [data-node-inner-panel="settings"]');
    const start = settingsPanel && settingsPanel.querySelector('[data-node-action="start"][data-net="' + net + '"]');
    wasmNodeStartTraceFrontend("frontend.settings_subtab_rendered", {
      network: net,
      action: "render",
      result: settingsPanel ? "ok" : "missing",
      details: { belongsToSettings: Boolean(settingsPanel), selectedNetwork: wasmNodeTraceActiveNetwork(root) },
    });
    wasmNodeStartTraceFrontend("frontend.start_control_rendered", {
      network: net,
      action: "start",
      result: start ? "ok" : "missing",
      details: { belongsToSettings: Boolean(start && settingsPanel && settingsPanel.contains(start)), startDisabled: Boolean(start && start.disabled) },
    });
  }
  return true;
};
const wasmNodeRuntimeActionForCommand = (command) =>
  command === "kgw_kgw_apply_node_settings_v1" ? "start" :
    command === "kgw_kgw_disable_network_v1" ? "stop" : "runtime";
const wasmNodeSmallOwnerTrace = (net, action, phase, details) => {
  const safeNet = String(net || "unknown");
  const safeAction = String(action || "small-owner");
  const safePhase = String(phase || "unknown");
  const tauri = window.__TAURI__;
  const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
    ? tauri.core.invoke.bind(tauri.core)
    : tauri && typeof tauri.invoke === "function"
      ? tauri.invoke.bind(tauri)
      : window.__TAURI_INVOKE__;
  if (typeof invoke !== "function") return false;
  Promise.resolve(invoke("kgw_frontend_button_trace_v1", {
    scope: "node",
    net: safeNet,
    action: safeAction,
    phase: safePhase,
    details: JSON.stringify({ patch: "KGW_SMALL_NODE_BRIDGE_TRACE_PATCH_R44D", existingOwner: "node-small-owner-functions", network: safeNet, action: safeAction, phase: safePhase, details: details && typeof details === "object" ? details : {} }),
  })).catch(() => {});
  return true;
};
const wasmNodeExplicitTrace = (net, action, phase, details) => {
  const safeNet = String(net || "unknown");
  const safeAction = String(action || "internal-navigation");
  const safePhase = String(phase || "unknown");
  const tauri = window.__TAURI__;
  const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
    ? tauri.core.invoke.bind(tauri.core)
    : tauri && typeof tauri.invoke === "function"
      ? tauri.invoke.bind(tauri)
      : window.__TAURI_INVOKE__;
  if (typeof invoke !== "function") return false;
  Promise.resolve(invoke("kgw_frontend_button_trace_v1", {
    scope: "node",
    net: safeNet,
    action: safeAction,
    phase: safePhase,
    details: JSON.stringify({ patch: "KGW_NODE_EXPLICIT_TRACE_HELPER_VISIBILITY_R45F", owner: "node-module-visible-explicit-trace-helper", network: safeNet, action: safeAction, phase: safePhase, details: details && typeof details === "object" ? details : {} }),
  })).catch(() => {});
  return true;
};
const wasmNodeExplicitOwnerTrace = (net, action, phase, details) => {
  const safeNet = String(net || "unknown");
  const safeAction = String(action || "unknown");
  const safePhase = String(phase || "unknown");
  const tauri = window.__TAURI__;
  const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
    ? tauri.core.invoke.bind(tauri.core)
    : null;
  if (typeof invoke !== "function") return false;
  Promise.resolve(invoke("kgw_frontend_button_trace_v1", {
    scope: "node",
    net: safeNet,
    action: safeAction,
    phase: safePhase,
    details: JSON.stringify({ patch: "KGW_EXPLICIT_TRACE_EXACT_ANCHOR_PATCH_R27D", owner: "node-existing-owner", network: safeNet, action: safeAction, phase: safePhase, details: details && typeof details === "object" ? details : {} }),
  })).catch(() => {});
  return true;
};
const __kgwNodePreviewSequences = {};
const wasmNodePreviewSequence = (net) =>
  Number(__kgwNodePreviewSequences[String(net || "")] || 0);
const wasmNodeUpdateCommand = (net, _locked = false) => {
  const key = String(net || "");
  const next = wasmNodePreviewSequence(key) + 1;
  __kgwNodePreviewSequences[key] = next;
  const preview = document.getElementById("node-" + key + "-commandPreview");
  if (preview) {
    preview.value = "";
    preview.dataset.effectiveSettingsAuthority = "validating";
  }
  return next;
};
const wasmNodePreparePreview = async (net, effective) => {
  const key = String(net || "");
  const preview = document.getElementById("node-" + key + "-commandPreview");
  const appDir = document.getElementById("node-" + key + "-appDir");
  return {
    command: String(preview && preview.value ? preview.value : "kaspad --managed-preview"),
    arguments: [],
    appDir: String(appDir && appDir.value ? appDir.value : ""),
    availableCpuThreads: "16",
    effectiveNodeSettings: effective,
  };
};
const wasmNodeApplyRootDefaultPath = async (net, locked = false) => {
  const key = String(net || "");
  wasmNodeUpdateCommand(key, Boolean(locked));
  const appDir = document.getElementById("node-" + key + "-appDir");
  return { appDir: String(appDir && appDir.value ? appDir.value : "") };
};
const wasmNodeToggleCommandOptionAndUpdate = (net, name, locked = false) => {
  const enabled = wasmNodeToggleCommandOption(net, name);
  wasmNodeUpdateCommand(net, Boolean(locked));
  return enabled;
};
const wasmNodeInstallNetworkTabs = (root, callbacks = {}) => {
  if (!root) return false;
  const tabs = Array.from(root.querySelectorAll("[data-node-network-tab]"));
  const panels = Array.from(root.querySelectorAll("[data-node-network-panel]"));
  const select = (value, persist = true) => {
    const selected = wasmNodeNormalizeNetwork(value) || "mainnet";
    if (persist) wasmNodeSaveLastNetwork(selected);
    for (const tab of tabs) {
      const active = String(tab.dataset.nodeNetworkTab || "") === selected;
      tab.classList.toggle("active", active);
      tab.setAttribute("aria-selected", active ? "true" : "false");
    }
    for (const panel of panels) {
      const active = String(panel.dataset.nodeNetworkPanel || "") === selected;
      panel.classList.toggle("active", active);
      panel.hidden = !active;
      panel.dataset.active = active ? "true" : "false";
    }
    return selected;
  };
  for (const tab of tabs) {
    tab.addEventListener("click", (event) => {
      event.preventDefault();
      select(tab.dataset.nodeNetworkTab, true);
    });
  }
  const active = tabs.find((tab) => tab.classList.contains("active"));
  select(wasmNodeReadLastNetwork() || (active && active.dataset.nodeNetworkTab) || "mainnet", false);
  if (callbacks && typeof callbacks.hydrate === "function") callbacks.hydrate("network-tabs-installed");
  return true;
};
const wasmNodeInstallDelegatedTabs = (root) => {
  if (!root) return false;
  root.addEventListener("click", (event) => {
    const target = event && event.target;
    if (!target || typeof target.closest !== "function") return;
    const inner = target.closest("[data-node-inner-tab]");
    if (inner) {
      const net = String(inner.dataset.net || "");
      const selected = wasmNodeSaveInnerTab(net, inner.dataset.nodeInnerTab);
      const panel = root.querySelector('[data-node-network-panel="' + net + '"]');
      if (!panel) return;
      for (const item of panel.querySelectorAll("[data-node-inner-tab]")) {
        item.classList.toggle("active", item === inner);
      }
      for (const item of panel.querySelectorAll("[data-node-inner-panel]")) {
        const active = String(item.dataset.nodeInnerPanel || "") === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      }
      return;
    }
    const section = target.closest("[data-node-section-tab]");
    if (!section) return;
    const net = String(section.dataset.net || "");
    const selected = String(section.dataset.nodeSectionTab || "");
    const panel = root.querySelector('[data-node-network-panel="' + net + '"]');
    if (!panel) return;
    for (const item of panel.querySelectorAll("[data-node-section-tab]")) {
      const active = item === section;
      item.classList.toggle("active", active);
      item.setAttribute("aria-selected", active ? "true" : "false");
    }
    for (const item of panel.querySelectorAll("[data-node-section-panel]")) {
      const active = String(item.dataset.nodeSectionPanel || "") === selected;
      item.classList.toggle("active", active);
      item.hidden = !active;
    }
  });
  return true;
};
const kgwNodeTraceStartButtonStateR1 = wasmNodeTraceStartButtonState;
const kgwNodeInstallStartTraceDocumentClickObserverR1 = wasmNodeInstallStartTraceDocumentClickObserver;
const kgwNodeTraceRenderedStartControlsR1 = wasmNodeTraceRenderedStartControls;
const kgwNodeRuntimeActionForCommandR1 = wasmNodeRuntimeActionForCommand;
const kgwNodeSmallOwnerTraceR44D = wasmNodeSmallOwnerTrace;
const kgwNodeExplicitTraceR27D = wasmNodeExplicitTrace;
const kgwNodeExplicitOwnerTraceR27D = wasmNodeExplicitOwnerTrace;
const kgwI18nTextR41 = wasmNodeI18nText;
const kgwNodeNetworkProfile = wasmNodeNetworkProfile;
const kgwNodeNetworkEnabled = wasmNodeNetworkEnabled;
const kgwNodeSetNetworkEnabled = wasmNodeSetNetworkEnabled;
const kgwNodeNetworkPolicyMessage = wasmNodeNetworkPolicyMessage;
const byId = wasmNodeById;
const esc = wasmNodeEscapeHtml;
const id = wasmNodeElementId;
const v = wasmNodeValue;
const c = wasmNodeChecked;
const kgwNodeCommandInlineStateR7 = wasmNodeCommandInlineState;
const kgwNodeCommandInlineToggleR7 = wasmNodeCommandInlineToggle;
const kgwNodeRefreshInlineCommandTogglesR7 = wasmNodeRefreshInlineCommandToggles;
const kgwInstallNodeLogAutoScrollControlsR27 = wasmNodeInstallLogAutoScrollControls;
const kgwNodeNormalizeNetworkR101W2 = wasmNodeNormalizeNetwork;
const kgwNodeReadLastNetworkR101W2 = wasmNodeReadLastNetwork;
const kgwNodeSaveLastNetworkR101W2 = wasmNodeSaveLastNetwork;
const stringifyRuntimeResult = wasmNodeStringifyRuntimeResult;
const normalizeRuntimeError = wasmNodeNormalizeRuntimeError;
const parseRuntimeFields = wasmNodeParseRuntimeFields;
const kgwNodeRuntimeEvidence = wasmNodeRuntimeEvidence;
const kgwNodeR51CaptureFactoryDefaults = wasmNodeR51CaptureFactoryDefaults;
const kgwNodeR51IsRunning = wasmNodeRuntimeIsRunning;
const kgwNodeRuntimeErrorFromStatus = wasmNodeRuntimeErrorFromStatus;
const kgwNodeTraceActiveNetworkR1 = wasmNodeTraceActiveNetwork;
const kgwNodeBackendInvokeR5 = wasmNodeBackendInvoke;
const kgwNodeResolveInnerTabR101U = wasmNodeResolveInnerTab;
const kgwNodeSaveInnerTabR101U = wasmNodeSaveInnerTab;
const kgwNodeAssertStartEvidence = wasmNodeAssertStartEvidence;
const nodeRuntimeArgs = wasmNodeRuntimeArgs;
const kgwNodeR51Keys = wasmNodeR51Keys;
const kgwNodeR51Fields = wasmNodeR51Fields;
const kgwNodeR51Panel = wasmNodeR51Panel;
`;
  const executable = importPrelude
    + renderFixture
    + "\nconst wasmNodeRenderNetworkPanelsHtml = () => wasmNodeNetworkProfiles().map(renderNetworkPanel).join(\"\");\n"
    + source
    .replace(/^import[\s\S]*?from\s+["'][^"']+["'];\s*/gm, "")
    .replace(/^import\s+["'][^"']+["'];\s*/gm, "")
    .replace(/^await\s+initNodeRust\(\);\s*/gm, "")
    .replace(/export\s+async\s+function\s+initKaspaNodeTab/, "async function initKaspaNodeTab")
    .replace(/export\s*\{[^}]+\}\s*;?/g, "")
    .replace(/export\s+default\s+initKaspaNodeTab\s*;/, "")
    + "\nwindow.__kgwStartButtonTest = { initKaspaNodeTab, kgwResolvePublicTauriInvokeR1, kgwNodeR51SetRuntimeButtons, KGW_NODE_R51_TRANSITIONS };\n";
  vm.runInNewContext(executable, sandbox, { filename: nodeJsPath });

  return { window, document, root };
}

async function flush() {
  await Promise.resolve();
  await Promise.resolve();
  await new Promise((resolve) => setImmediate(resolve));
  await new Promise((resolve) => setImmediate(resolve));
  await new Promise((resolve) => setTimeout(resolve, 25));
}

function startCalls(calls) {
  return calls.filter((call) => call.command === "kgw_kgw_apply_node_settings_v1");
}

function traceCalls(calls) {
  return calls.filter((call) => call.command === "kgw_start_trace_frontend_v1");
}

function traceStages(calls) {
  return traceCalls(calls).map((call) => call.payload.stage);
}

function copyCalls(calls) {
  return calls.filter((call) => call.command === "kgw_copy_text_to_clipboard_v1");
}

function parsedTraceDetails(call) {
  return JSON.parse(call.payload.details || "{}");
}

async function dynamicClickTests() {
  const { window, document, root } = createHarness();
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();

  const mainnetLogPanel = root.querySelector('[data-node-network-panel="mainnet"] [data-node-inner-panel="log"]');
  const mainnetSettingsPanel = root.querySelector('[data-node-network-panel="mainnet"] [data-node-inner-panel="settings"]');
  assert.ok(mainnetLogPanel, "mainnet log panel must render");
  assert.ok(mainnetSettingsPanel, "mainnet settings panel must render");
  assert.strictEqual(mainnetLogPanel.querySelectorAll('[data-node-action="start"]').length, 0, "rendered Live Node Monitor must not contain Start");
  assert.strictEqual(mainnetLogPanel.querySelectorAll('[data-node-action="stop"]').length, 0, "rendered Live Node Monitor must not contain Stop");
  assert.strictEqual(mainnetSettingsPanel.querySelectorAll('[data-node-action="start"]').length, 1, "rendered Settings must contain one Start");
  assert.strictEqual(mainnetSettingsPanel.querySelectorAll('[data-node-action="stop"]').length, 1, "rendered Settings must contain one Stop");

  const calls = [];
  window.__TAURI__ = {
    core: {
      invoke: async (command, payload) => {
        calls.push({ command, payload });
        if (command !== "kgw_kgw_apply_node_settings_v1") return "ignored";
        return "parallel-owned-self-worker started;role=node;network=" + payload.network + ";pid=4242;owner=self-worker;runtime_state=running;readiness=READY";
      },
    },
  };

  calls.length = 0;
  const mainnetStartReady = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  mainnetStartReady.disabled = false;
  mainnetStartReady.click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 1, "mainnet Start click must invoke exactly once");
  assert.strictEqual(startCalls(calls)[0].payload.network, "mainnet");
  assert.strictEqual(startCalls(calls)[0].payload.nodeKind, "integrated-as-daemon");
  assert.strictEqual(startCalls(calls)[0].payload.bridgeKind, "disable");
  assert.strictEqual(startCalls(calls)[0].payload.runtimeRole, "node");
  assert.ok(traceStages(calls).includes("frontend.capture_click_observed"), "capture observer must trace the physical-style Start click");
  const capture = traceCalls(calls).find((call) => call.payload.stage === "frontend.capture_click_observed");
  assert.strictEqual(capture.payload.network, "mainnet");
  assert.strictEqual(capture.payload.action, "start");
  assert.strictEqual(parsedTraceDetails(capture).belongsToSettings, true, "capture trace must report Settings ownership");

  root.querySelector('[data-node-network-tab="testnet10"]').click();
  calls.length = 0;
  const testnet10StartReady = root.querySelector('[data-node-action="start"][data-net="testnet10"]');
  testnet10StartReady.disabled = false;
  testnet10StartReady.click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 1, "testnet10 Start click must invoke exactly once after tab switch");
  assert.strictEqual(startCalls(calls)[0].payload.network, "testnet10");

  calls.length = 0;
  const testnet13Start = root.querySelector('[data-node-action="start"][data-net="testnet13"]');
  testnet13Start.click();
  await flush();
  assert.strictEqual(startCalls(calls).length, 0, "testnet13 Start must not invoke while opt-in is disabled");

  window.__TAURI__.core.invoke = async (command, payload) => {
    calls.push({ command, payload });
    if (command === "kgw_start_trace_frontend_v1" || command === "kgw_frontend_button_trace_v1") return true;
    if (command === "kgw_kgw_apply_node_settings_v1") {
      throw new Error("spawn_failed=true;runtime_role=node;network=mainnet;source=self-worker;error=Access is denied.");
    }
    if (command === "kgw_runtime_owner_status_v1") {
      throw new Error("runtime status unavailable after rejected Start");
    }
    return true;
  };
  const mainnetStart = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  mainnetStart.disabled = false;
  calls.length = 0;
  mainnetStart.click();
  await flush();
  const errorNode = document.getElementById("node-mainnet-runtimeError");
  const statusNode = document.getElementById("node-mainnet-runtimeStatus");
  const evidenceNode = document.getElementById("node-mainnet-runtimeEvidence");
  assert.strictEqual(startCalls(calls).length, 1, "failed Start click must still invoke exactly once");
  assert.strictEqual(mainnetStart.disabled, true, "failed Start must remain non-startable until backend reconciliation proves terminal state");
  assert.ok(traceStages(calls).includes("frontend.invoke_rejected"), "failed Start must trace invoke rejection");
  assert.ok(traceStages(calls).includes("frontend.button_state_restored_after_failure"), "failed Start must trace button restoration");
  assert.strictEqual(statusNode.textContent, "Reconciling", "failed Start must expose Reconciling until backend status is known");
  assert.ok(
    errorNode.textContent.includes("Access is denied."),
    "failed Start must expose original error, actual text: " + errorNode.textContent + "; status=" + statusNode.textContent + "; evidence=" + evidenceNode.textContent,
  );

  const rawLog = document.getElementById("node-mainnet-logOutput").textContent;
  assert.ok(!/initialized|parallel-owned-self-worker started|KGW node start response/i.test(rawLog), "raw log must not contain synthetic startup success text");
}

async function startupStopAvailabilityTests() {
  const { window, root } = createHarness();
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();
  const api = window.__kgwStartButtonTest;
  const stop = root.querySelector('[data-node-action="stop"][data-net="mainnet"]');
  api.KGW_NODE_R51_TRANSITIONS.mainnet = "starting";
  api.kgwNodeR51SetRuntimeButtons("mainnet", false, false);
  assert.strictEqual(stop.disabled, true, "Node Stop must remain disabled until READY; pre-READY cancellation is not an owned backend contract");
  api.KGW_NODE_R51_TRANSITIONS.mainnet = "";
  api.kgwNodeR51SetRuntimeButtons("mainnet", true, false);
  assert.strictEqual(stop.disabled, false, "Node Stop must become available after READY/running truth is confirmed");
}

async function stopTruthfulnessTests() {
  const calls = [];
  let resolveStop;
  let stopResponse = null;
  const { window, document, root } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          if (command === "kgw_kgw_disable_network_v1") {
            if (stopResponse !== null) return stopResponse;
            return await new Promise((resolve) => { resolveStop = resolve; });
          }
          if (command === "kgw_runtime_owner_status_v1") {
            return "parallel-owned-self-worker status;role=node;network=" + payload.network + ";running=false";
          }
          if (command === "kgw_kgw_runtime_logs_v1") return { entries: [] };
          return true;
        },
      },
    },
  });
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();

  const start = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  const stop = root.querySelector('[data-node-action="stop"][data-net="mainnet"]');
  start.disabled = true;
  stop.disabled = false;
  stop.click();
  await flush();
  assert.strictEqual(document.getElementById("node-mainnet-runtimeStatus").textContent, "Stopping");
  assert.strictEqual(stop.disabled, true, "duplicate Stop must remain disabled while backend Stop is pending");

  resolveStop("parallel-owned-self-worker stopped;role=node;network=mainnet;running=false;graceful=true;forced=false");
  await flush();
  assert.strictEqual(document.getElementById("node-mainnet-runtimeStatus").textContent, "Stopped");
  assert.strictEqual(document.getElementById("node-mainnet-runtimeEvidence").textContent, "Graceful official shutdown confirmed");

  stopResponse = "parallel-owned-self-worker stopped;role=node;network=mainnet;running=false;graceful=false;forced=true;reason=graceful stop timed out";
  start.disabled = true;
  stop.disabled = false;
  stop.click();
  await flush();
  assert.ok(document.getElementById("node-mainnet-runtimeError").textContent.includes("FORCED"));
  assert.strictEqual(document.getElementById("node-mainnet-runtimeEvidence").textContent, "FORCED termination confirmed");

  stopResponse = "parallel-owned-self-worker stopped with graceful failure;role=node;network=mainnet;running=false;graceful=false;forced=false;stop_failed=true;stop_outcome=FAILED;reason=official shutdown failed";
  start.disabled = true;
  stop.disabled = false;
  stop.click();
  await flush();
  assert.ok(document.getElementById("node-mainnet-runtimeError").textContent.includes("Official graceful shutdown failed"));
  assert.strictEqual(document.getElementById("node-mainnet-runtimeEvidence").textContent, "Worker exited after graceful shutdown failure");
}

function configuredTauriInvokeResolverTests() {
  const tauriConfig = JSON.parse(fs.readFileSync(tauriConfigPath, "utf8"));
  assert.strictEqual(tauriConfig.app.withGlobalTauri, true, "Tauri config must expose the supported global API");

  const calls = [];
  const { window } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          return true;
        },
      },
    },
  });

  const resolved = window.__kgwStartButtonTest.kgwResolvePublicTauriInvokeR1();
  assert.strictEqual(resolved.adapter, "window.__TAURI__.core.invoke", "Tauri 2 global core invoke must win");
  assert.strictEqual(typeof resolved.invoke, "function", "resolver must return a public invoke function");
  assert.strictEqual(resolved.shape.hasCoreInvoke, true, "resolver shape must detect core.invoke");
  assert.strictEqual(resolved.shape.expectedConfiguredGlobal, "window.__TAURI__.core.invoke");
}

async function missingInvokeApiVisibleErrorTest() {
  const { window, document, root } = createHarness();
  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();

  const start = root.querySelector('[data-node-action="start"][data-net="mainnet"]');
  start.disabled = false;
  start.click();
  await flush();

  const errorNode = document.getElementById("node-mainnet-runtimeError");
  assert.ok(errorNode.textContent.includes("Tauri invoke API is not available"), "missing invoke API must be visible in the UI");
  assert.strictEqual(start.disabled, true, "missing invoke API must keep Start disabled until IPC/runtime truth can be reconciled");
  assert.strictEqual(document.getElementById("node-mainnet-runtimeStatus").textContent, "Reconciling", "missing invoke API must expose Reconciling rather than fabricate Stopped");
}

async function tracePayloadSafetyTests() {
  const calls = [];
  const { window, root } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          if (command === "kgw_kgw_apply_node_settings_v1") {
            return "parallel-owned-self-worker started;role=node;network=" + payload.network + ";pid=4242;owner=self-worker;runtime_state=running;readiness=READY";
          }
          return true;
        },
      },
    },
  });

  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();
  calls.length = 0;
  root.querySelector('[data-node-action="start"][data-net="mainnet"]').click();
  await flush();

  assert.strictEqual(startCalls(calls).length, 1, "trace safety run must invoke Start exactly once");
  const serializedTrace = JSON.stringify(traceCalls(calls).map((call) => call.payload));
  assert.ok(!/secret|token|private|mnemonic|wallet/i.test(serializedTrace), "trace payload must exclude secret-like fields");
  assert.ok(!/nodeCommandPreview|bridgeCommandPreview|--rpc|--appdir|--stratum/i.test(serializedTrace), "trace payload must not expose command arguments or preview fields");
}

async function copyLogFrontendTests() {
  const calls = [];
  let copyReject = null;
  let copyPending = false;
  let resolvePendingCopy = null;
  let navigatorWriteCount = 0;
  const { window, document, root } = createHarness({
    tauri: {
      core: {
        invoke: async (command, payload) => {
          calls.push({ command, payload });
          if (command === "kgw_start_trace_frontend_v1") return true;
          if (command === "kgw_copy_text_to_clipboard_v1") {
            if (copyReject) throw copyReject;
            if (copyPending) {
              return await new Promise((resolve) => {
                resolvePendingCopy = resolve;
              });
            }
            return "clipboard_write_v1;network=" + payload.network + ";characters=" + payload.characterCount + ";lines=" + payload.lineCount + ";sha256=" + payload.sha256 + ";copied=true";
          }
          if (command === "kgw_runtime_owner_status_v1") return "parallel-owned-self-worker status;role=node;network=" + payload.network + ";running=false";
          if (command === "kgw_kgw_runtime_logs_v1") return "";
          return true;
        },
      },
    },
  });

  window.navigator.clipboard.writeText = async () => {
    navigatorWriteCount += 1;
    throw new Error("browser clipboard must not be used for Copy Log");
  };

  await window.__kgwStartButtonTest.initKaspaNodeTab(root);
  await flush();
  calls.length = 0;

  const mainnetLog = document.getElementById("node-mainnet-logOutput");
  const mainnetCopy = root.querySelector('[data-node-action="copy-log"][data-net="mainnet"]');
  mainnetLog.textContent = "mainnet raw line 1\nmainnet raw line 2";
  mainnetCopy.click();
  await flush();

  assert.strictEqual(copyCalls(calls).length, 1, "Copy Log must invoke native clipboard exactly once");
  const copied = copyCalls(calls)[0].payload;
  assert.strictEqual(copied.network, "mainnet", "Copy Log must pass the active network");
  assert.strictEqual(copied.text, "mainnet raw line 1\r\nmainnet raw line 2", "Copy Log must preserve multiline raw text order with Windows line endings");
  assert.strictEqual(copied.lineCount, 2, "Copy Log must pass line count metadata");
  assert.strictEqual(copied.characterCount, Array.from(copied.text).length, "Copy Log must pass character count metadata");
  assert.strictEqual(String(copied.sha256 || "").length, 64, "Copy Log should pass SHA-256 metadata when Web Crypto is available");
  assert.strictEqual(navigatorWriteCount, 0, "Copy Log must not use navigator.clipboard as the primary path");
  assert.ok(traceStages(calls).includes("frontend.copy_log_click_observed"), "Copy Log physical-style click must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_network_resolved"), "Copy Log network resolution must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_content_prepared"), "Copy Log content metadata must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_dispatched"), "Copy Log native dispatch must be traced");
  assert.ok(traceStages(calls).includes("frontend.copy_log_succeeded"), "Copy Log success feedback must be traced after native success");
  assert.ok(!JSON.stringify(traceCalls(calls).map((call) => call.payload)).includes("mainnet raw line"), "Copy Log trace must exclude raw clipboard content");
  assert.strictEqual(mainnetLog.textContent, "mainnet raw line 1\nmainnet raw line 2", "Copy Log must not insert synthetic content into raw logs");

  calls.length = 0;
  copyReject = new Error("native clipboard failure");
  mainnetCopy.textContent = "Copy Log";
  mainnetCopy.disabled = false;
  mainnetCopy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 1, "Clipboard failure must still call native clipboard once");
  assert.ok(traceStages(calls).includes("frontend.copy_log_failed"), "Clipboard failure must be traced");
  assert.strictEqual(mainnetCopy.disabled, false, "Clipboard failure must restore the button state");
  assert.ok(/copy failed/i.test(mainnetCopy.textContent), "Clipboard failure must not display Copied feedback");
  assert.ok(root.querySelector('.kgw-copy-log-status-v1[data-net="mainnet"]').textContent.includes("native clipboard failure"), "Clipboard failure must be visible");
  copyReject = null;

  calls.length = 0;
  mainnetLog.textContent = "Mainnet log is empty.";
  mainnetCopy.textContent = "Copy Log";
  mainnetCopy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 0, "Empty or placeholder logs must not invoke native clipboard");
  assert.ok(traceStages(calls).includes("frontend.copy_log_failed"), "Empty Copy Log rejection must be traced");

  calls.length = 0;
  mainnetLog.textContent = "duplicate guard line";
  copyPending = true;
  resolvePendingCopy = null;
  mainnetCopy.disabled = false;
  mainnetCopy.textContent = "Copy Log";
  mainnetCopy.click();
  mainnetCopy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 1, "Duplicate Copy Log clicks must not invoke native clipboard twice");
  assert.strictEqual(typeof resolvePendingCopy, "function", "pending Copy Log test must hold the native clipboard promise");
  resolvePendingCopy("clipboard_write_v1;network=mainnet;characters=20;lines=1;copied=true");
  copyPending = false;
  await flush();

  const testnet10Tab = root.querySelector('[data-node-network-tab="testnet10"]');
  testnet10Tab.click();
  await flush();
  const testnet10Log = document.getElementById("node-testnet10-logOutput");
  const testnet10Copy = root.querySelector('[data-node-action="copy-log"][data-net="testnet10"]');
  mainnetLog.textContent = "mainnet must not be copied";
  testnet10Log.textContent = "testnet10 raw line 1\ntestnet10 raw line 2";
  calls.length = 0;
  testnet10Copy.click();
  await flush();
  assert.strictEqual(copyCalls(calls).length, 1, "testnet10 Copy Log must invoke native clipboard once");
  assert.strictEqual(copyCalls(calls)[0].payload.network, "testnet10", "Copy Log must pass testnet10 when testnet10 is active");
  assert.ok(copyCalls(calls)[0].payload.text.includes("testnet10 raw line 1"), "Copy Log must copy the active network buffer");
  assert.ok(!copyCalls(calls)[0].payload.text.includes("mainnet must not be copied"), "Copy Log must not mix mainnet into testnet10");

  calls.length = 0;
  testnet10Log.textContent = "testnet10 clear target";
  mainnetLog.textContent = "mainnet should remain after clear";
  root.querySelector('[data-node-action="clear-log"][data-net="testnet10"]').click();
  await flush();
  assert.strictEqual(testnet10Log.textContent, "", "Clear Log must affect the active network log");
  assert.strictEqual(mainnetLog.textContent, "mainnet should remain after clear", "Clear Log must not clear another network log");

  const largeLines = Array.from({ length: 1600 }, (_, index) => "large raw line " + index);
  testnet10Log.textContent = largeLines.join("\n");
  calls.length = 0;
  testnet10Copy.textContent = "Copy Log";
  testnet10Copy.click();
  await flush();
  const largeCopy = copyCalls(calls)[0].payload.text;
  assert.strictEqual(largeCopy, largeLines.join("\r\n"), "Large Copy Log text must not be silently truncated or reordered");
}

(async () => {
  try {
    staticPlacementTests();
    configuredTauriInvokeResolverTests();
    await dynamicClickTests();
    await startupStopAvailabilityTests();
    await stopTruthfulnessTests();
    await missingInvokeApiVisibleErrorTest();
    await tracePayloadSafetyTests();
    await copyLogFrontendTests();
    console.log("KGW start button and Copy Log frontend tests PASSED");
  } catch (error) {
    fail(error && error.stack ? error.stack : String(error));
  }
})();
"########;

pub fn run(root: &Path) -> Result<String, String> {
    let temp = tempfile::Builder::new()
        .prefix(".kgw-start-button-frontend-")
        .tempdir()
        .map_err(|error| format!("failed to create start-button tempdir: {error}"))?;
    let bridge = temp.path().join("bridge.cjs");
    let render_fixture = temp.path().join("node-render-fixture.js");
    fs::write(&bridge, NODE_BRIDGE.as_bytes())
        .map_err(|error| format!("failed to write start-button Node bridge: {error}"))?;
    fs::write(&render_fixture, NODE_RENDER_FIXTURE.as_bytes())
        .map_err(|error| format!("failed to write Node render fixture: {error}"))?;
    let output = Command::new("node")
        .arg(&bridge)
        .env("KGW_NODE_RENDER_FIXTURE", &render_fixture)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to launch start-button Node bridge: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "start-button frontend bridge failed with {}\nstdout={}\nstderr={}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_frontend_bridge_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let message = run(root).unwrap();
        assert!(message.contains("PASSED"), "{message}");
    }

    #[test]
    fn generated_bridge_is_interop_only_and_loads_real_frontend() {
        assert!(NODE_BRIDGE.contains("apps"));
        assert!(NODE_BRIDGE.contains("kaspa-node.js"));
        assert!(NODE_BRIDGE.contains("vm.runInNewContext"));
        assert!(NODE_BRIDGE.contains("KGW start button and Copy Log frontend tests PASSED"));
    }
}
