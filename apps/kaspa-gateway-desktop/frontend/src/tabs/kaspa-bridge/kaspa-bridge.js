function kgwI18nTextR41(key, fallback) {
  try {
    if (window.kgwT && typeof window.kgwT === "function") return window.kgwT(key, fallback);
    if (window.KGW_I18N && typeof window.KGW_I18N.t === "function") return window.KGW_I18N.t(key, fallback);
    if (window.i18n && typeof window.i18n.t === "function") return window.i18n.t(key, fallback);
  } catch (_) {
  }
  return fallback;
}


const BRIDGE_NETWORKS = [
  { key: "mainnet", label: "MAINNET", testnet: false, netsuffix: "", kaspadPort: "16110", stratumPort: ":5555", promPort: ":2112" },
  { key: "testnet10", label: "TESTNET10", testnet: true, netsuffix: "10", kaspadPort: "16210", stratumPort: ":5556", promPort: ":2113" },
  { key: "testnet12", label: "TESTNET12", testnet: true, netsuffix: "12", kaspadPort: "16310", stratumPort: ":5557", promPort: ":2114" }
];

const bridgeInstances = {
  mainnet: [{ id: 1 }],
  testnet10: [{ id: 1 }],
  testnet12: [{ id: 1 }]
};

let activeInstance = {
  mainnet: 1,
  testnet10: 1,
  testnet12: 1
};

function byId(id) {
  return document.getElementById(id);
}

function esc(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function id(net, name) {
  return `bridge-${net}-${name}`;
}





function iid(net, instanceId, name) {
  return `bridge-${net}-i${instanceId}-${name}`;
}

function v(net, name) {
  const el = byId(id(net, name));
  return el ? String(el.value || "").trim() : "";
}

function c(net, name) {
  const el = byId(id(net, name));
  return Boolean(el && el.checked);
}

function iv(net, instanceId, name) {
  const el = byId(iid(net, instanceId, name));
  return el ? String(el.value || "").trim() : "";
}

function ic(net, instanceId, name) {
  const el = byId(iid(net, instanceId, name));
  return Boolean(el && el.checked);
}

function addFlag(lines, net, name, flag) {
  if (c(net, name)) lines.push(flag);
}

function addValue(lines, net, name, flag) {
  const value = v(net, name);
  if (value) lines.push(`${flag}=${value}`);
}

function addBoolValue(lines, net, name, flag) {
  const value = v(net, name);
  if (value && value !== "not set") lines.push(`${flag}=${value}`);
}









function cardInput(net, name, label, value = "", placeholder = "", span = "") {
  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <input id="${id(net, name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function cardSelect(net, name, label, options, value = "", span = "") {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <select id="${id(net, name)}">${opts}</select>
    </div>`;
}

function cardCheck(net, name, label, checked = false, span = "") {
  return `
    <label class="bridge-v7-card check${span ? " " + span : ""}">
      <input id="${id(net, name)}" type="checkbox"${checked ? " checked" : ""}>
      <span>${esc(label)}</span>
    </label>`;
}

function instanceInput(net, instanceId, name, label, value = "", placeholder = "", span = "") {
  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <input id="${iid(net, instanceId, name)}" type="text" value="${esc(value)}" placeholder="${esc(placeholder)}">
    </div>`;
}

function instanceSelect(net, instanceId, name, label, options, value = "", span = "") {
  const opts = options.map((item) => {
    const selected = item === value ? " selected" : "";
    return `<option value="${esc(item)}"${selected}>${esc(item || "not set")}</option>`;
  }).join("");

  return `
    <div class="bridge-v7-card${span ? " " + span : ""}">
      <span>${esc(label)}</span>
      <select id="${iid(net, instanceId, name)}">${opts}</select>
    </div>`;
}

function instanceCheck(net, instanceId, name, label, checked = false, span = "") {
  return `
    <label class="bridge-v7-card check${span ? " " + span : ""}">
      <input id="${iid(net, instanceId, name)}" type="checkbox"${checked ? " checked" : ""}>
      <span>${esc(label)}</span>
    </label>`;
}

function renderRuntime(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardSelect(net.key, "nodeMode", "--node-mode", ["external", "inprocess"], "external")}
      ${net.key === "mainnet" ? "" : cardCheck(net.key, "testnet", "--testnet", net.testnet)}
      ${cardInput(net.key, "config", "--config", "", "config.yaml")}
      ${cardInput(net.key, "appdir", "--appdir", "", "app dir")}
      ${cardInput(net.key, "kaspadAddress", "--kaspad-address", `127.0.0.1:${net.kaspadPort}`)}
      ${cardInput(net.key, "blockWaitTime", "--block-wait-time", "50ms")}
      ${cardInput(net.key, "healthCheckPort", "--health-check-port", "", "optional")}
      ${cardInput(net.key, "webDashboardPort", "--web-dashboard-port", "", ":3030")}
    </div>`;
}

function renderDifficulty(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardInput(net.key, "minShareDiff", "--min-share-diff", "8192")}
      ${cardInput(net.key, "sharesPerMin", "--shares-per-min", "30")}
      ${cardSelect(net.key, "varDiff", "--var-diff", ["true", "false"], "true")}
      ${cardSelect(net.key, "varDiffStats", "--var-diff-stats", ["true", "false"], "true")}
      ${cardSelect(net.key, "pow2Clamp", "--pow2-clamp", ["true", "false"], "true")}
      ${cardInput(net.key, "extranonceSize", "--extranonce-size", "0")}
      ${cardInput(net.key, "coinbaseTagSuffix", "--coinbase-tag-suffix", "", "optional", "span2")}
    </div>`;
}

function renderLogging(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardSelect(net.key, "printStats", "--print-stats", ["true", "false"], "true")}
      ${cardSelect(net.key, "logToFile", "--log-to-file", ["true", "false"], "true")}
      ${cardSelect(net.key, "approxGeoLookup", "--approximate-geo-lookup", ["not set", "true", "false"], "not set", "span2")}
      ${cardInput(net.key, "startupDelay", "Startup delay sec", "0")}
      ${cardCheck(net.key, "startOnLaunch", "Start on launch", false)}
      ${cardCheck(net.key, "autoRestart", "Auto-reconnect", true)}
    </div>`;
}

function renderPorts(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardInput(net.key, "stratumPort", "--stratum-port", net.stratumPort)}
      ${cardInput(net.key, "promPort", "--prom-port", net.promPort)}
      ${cardInput(net.key, "customExePath", "Custom bridge exe", "", "optional", "span2")}
      <div class="bridge-v7-card buttons">
        <button type="button" data-bridge-action="default-url" data-net="${net.key}">Default URL</button>
        <button type="button" data-bridge-action="browse" data-net="${net.key}">Browse</button>
      </div>
    </div>`;
}

function renderCpuMiner(net) {
  return `
    <div class="bridge-v7-grid">
      ${cardCheck(net.key, "internalCpuMiner", "--internal-cpu-miner", false)}
      ${cardInput(net.key, "internalCpuMinerAddress", "--internal-cpu-miner-address", "", "kaspa:...", "span2")}
      ${cardInput(net.key, "internalCpuMinerThreads", "--internal-cpu-miner-threads", "", "threads")}
      ${cardInput(net.key, "internalCpuMinerThrottleMs", "--internal-cpu-miner-throttle-ms", "", "optional")}
      ${cardInput(net.key, "internalCpuMinerTemplatePollMs", "--internal-cpu-miner-template-poll-ms", "", "optional", "span2")}
    </div>`;
}

function renderInstancePanel(net, instanceId) {
  return `
    <section class="bridge-v7-instance-panel" data-net="${net.key}" data-instance-panel="${instanceId}"${activeInstance[net.key] === instanceId ? "" : " hidden"}>
      <div class="bridge-v7-grid">
        <div class="bridge-v7-card span3">
          <span>--instance</span>
          <textarea id="${iid(net.key, instanceId, "instance")}" class="bridge-v7-instance-text" placeholder="Optional instance definition"></textarea>
        </div>
        ${instanceSelect(net.key, instanceId, "instanceLogToFile", "--instance-log-to-file", ["not set", "true", "false"], "not set")}
        ${instanceSelect(net.key, instanceId, "instanceVarDiff", "--instance-var-diff", ["not set", "true", "false"], "not set")}
        ${instanceSelect(net.key, instanceId, "instanceVarDiffStats", "--instance-var-diff-stats", ["not set", "true", "false"], "not set")}
        ${instanceInput(net.key, instanceId, "instanceSharesPerMin", "--instance-shares-per-min", "", "optional")}
        ${instanceSelect(net.key, instanceId, "instancePow2Clamp", "--instance-pow2-clamp", ["not set", "true", "false"], "not set")}
        <div class="bridge-v7-card buttons">
          <button type="button" data-bridge-action="duplicate-instance" data-net="${net.key}" data-instance="${instanceId}">Duplicate</button>
          <button type="button" class="danger" data-bridge-action="remove-instance" data-net="${net.key}" data-instance="${instanceId}">Remove</button>
        </div>
      </div>
    </section>`;
}


function bridgeNormalizeInstance(raw) {
  const value = String(raw || "").trim();

  if (!value) return "";

  if (value.includes("=")) {
    return value;
  }

  if (/^\d+$/.test(value)) {
    return `port=:${value}`;
  }

  return value;
}

function bridgeReadInstanceField(net, instanceId, fieldName) {
  const el = byId(id(net, `${fieldName}-${instanceId}`));
  if (!el) return "";

  if (el.type === "checkbox") return el.checked ? "true" : "";
  return String(el.value || "").trim();
}

function bridgeInstanceBoolArg(lines, net, instanceId, fieldName, flag) {
  const value = bridgeReadInstanceField(net, instanceId, fieldName);
  if (value === "true" || value === "false") {
    lines.push(`${flag}=${value}`);
  }
}

function bridgeInstanceValueArg(lines, net, instanceId, fieldName, flag) {
  const value = bridgeReadInstanceField(net, instanceId, fieldName);
  if (value) {
    lines.push(`${flag}=${value}`);
  }
}

function bridgeCollectCommandPorts(lines) {
  const ports = [];

  for (const part of lines) {
    const text = String(part || "");
    const values = [];

    const eq = text.match(/=(0\.0\.0\.0:|127\.0\.0\.1:|localhost:|:)?(\d{2,5})(\b|,)/);
    if (eq) values.push(eq[2]);

    const instancePorts = [...text.matchAll(/(?:port|prom_port)=(:|0\.0\.0\.0:|127\.0\.0\.1:|localhost:)?(\d{2,5})/g)];
    for (const match of instancePorts) values.push(match[2]);

    for (const value of values) {
      ports.push(value);
    }
  }

  return ports;
}

function bridgeDuplicatePorts(lines) {
  const ports = bridgeCollectCommandPorts(lines);
  const seen = new Set();
  const dup = new Set();

  for (const port of ports) {
    if (seen.has(port)) dup.add(port);
    seen.add(port);
  }

  return [...dup];
}

function bridgeEnsureInstanceState(net) {
  if (!Array.isArray(bridgeInstances[net])) {
    bridgeInstances[net] = [];
  }

  if (bridgeInstances[net].length === 0) {
    bridgeInstances[net].push({ id: Date.now(), instance: "", instanceLogToFile: "true", instanceVarDiff: "true", instanceSharesPerMin: "20", instanceVarDiffStats: "true", instancePow2Clamp: "true" });
  }
}


function renderInstances(net) {
  bridgeEnsureInstanceState(net);

  return `
    <div class="bridge-v7-instance-tabs">
      ${bridgeInstances[net].map((instance, index) => `
        <button
          type="button"
          class="bridge-v7-instance-tab ${index === 0 ? "active" : ""}"
          data-bridge-action="select-instance"
          data-network="${net}"
          data-instance-id="${instance.id}">
          Instance ${index + 1}
        </button>`).join("")}
      <button
        type="button"
        class="bridge-v7-instance-add"
        data-bridge-action="add-instance"
        data-network="${net}">+</button>
    </div>

    <div class="bridge-v7-instance-stack">
      ${bridgeInstances[net].map((instance, index) => `
        <section
          class="bridge-v7-instance-panel ${index === 0 ? "active" : ""}"
          data-bridge-instance-panel="${instance.id}">
          <label class="bridge-v7-card bridge-v7-wide">
            <span>--instance</span>
            <input
              id="${id(net, `instance-${instance.id}`)}"
              data-bridge-instance-field="instance"
              value="${instance.instance || ""}"
              placeholder="port=:5555,diff=2048,prom_port=:2114" />
          </label>

          <label class="bridge-v7-card">
            <span>--instance-log-to-file</span>
            <select id="${id(net, `instanceLogToFile-${instance.id}`)}" data-bridge-instance-field="instanceLogToFile">
              <option value=""></option>
              <option value="true" ${instance.instanceLogToFile === "true" ? "selected" : ""}>true</option>
              <option value="false" ${instance.instanceLogToFile === "false" ? "selected" : ""}>false</option>
            </select>
          </label>

          <label class="bridge-v7-card">
            <span>--instance-var-diff</span>
            <select id="${id(net, `instanceVarDiff-${instance.id}`)}" data-bridge-instance-field="instanceVarDiff">
              <option value=""></option>
              <option value="true" ${instance.instanceVarDiff === "true" ? "selected" : ""}>true</option>
              <option value="false" ${instance.instanceVarDiff === "false" ? "selected" : ""}>false</option>
            </select>
          </label>

          <label class="bridge-v7-card">
            <span>--instance-var-diff-stats</span>
            <select id="${id(net, `instanceVarDiffStats-${instance.id}`)}" data-bridge-instance-field="instanceVarDiffStats">
              <option value=""></option>
              <option value="true" ${instance.instanceVarDiffStats === "true" ? "selected" : ""}>true</option>
              <option value="false" ${instance.instanceVarDiffStats === "false" ? "selected" : ""}>false</option>
            </select>
          </label>

          <label class="bridge-v7-card">
            <span>--instance-shares-per-min</span>
            <input
              id="${id(net, `instanceSharesPerMin-${instance.id}`)}"
              data-bridge-instance-field="instanceSharesPerMin"
              value="${instance.instanceSharesPerMin || ""}" />
          </label>

          <label class="bridge-v7-card">
            <span>--instance-pow2-clamp</span>
            <select id="${id(net, `instancePow2Clamp-${instance.id}`)}" data-bridge-instance-field="instancePow2Clamp">
              <option value=""></option>
              <option value="true" ${instance.instancePow2Clamp === "true" ? "selected" : ""}>true</option>
              <option value="false" ${instance.instancePow2Clamp === "false" ? "selected" : ""}>false</option>
            </select>
          </label>

          <div class="bridge-v7-instance-actions">
            <button type="button" data-bridge-action="duplicate-instance" data-network="${net}" data-instance-id="${instance.id}">Duplicate</button>
            <button type="button" data-bridge-action="remove-instance" data-network="${net}" data-instance-id="${instance.id}" ${bridgeInstances[net].length <= 1 ? "disabled" : ""}>Remove</button>
          </div>
        </section>
      `).join("")}
    </div>`;
}



function renderSections(net) {
  const sections = [
    ["runtime", "Runtime", renderRuntime(net)],
    ["difficulty", "Difficulty", renderDifficulty(net)],
    ["logging", "Logging", renderLogging(net)],
    ["ports", "Ports / Paths", renderPorts(net)],
    ...(net.key === "mainnet" ? [] : [["cpu", "CPU Miner", renderCpuMiner(net)]]),
    ["instances", "Instances", renderInstances(net)]
  ];

  const tabs = sections.map(([key, label], index) =>
    `<button type="button" class="bridge-v7-section-tab${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-section-tab="${key}">${label}</button>`
  ).join("");

  const panels = sections.map(([key, , body], index) =>
    `<section class="bridge-v7-section${index === 0 ? " active" : ""}" data-net="${net.key}" data-bridge-section-panel="${key}"${index === 0 ? "" : " hidden"}>${body}</section>`
  ).join("");

  return `
    <div class="bridge-v7-section-tabs">${tabs}</div>
    <div class="bridge-v7-sections">${panels}</div>`;
}

function renderNetworkPanel(net, index) {
  return `
    <div class="bridge-v7-network-panel${index === 0 ? " active" : ""}" data-bridge-network-panel="${net.key}"${index === 0 ? "" : " hidden"}>
      <div class="bridge-v7-inner-tabs">
        <button type="button" class="bridge-v7-inner-tab active" data-net="${net.key}" data-bridge-inner-tab="settings">Settings</button>
        <button type="button" class="bridge-v7-inner-tab" data-net="${net.key}" data-bridge-inner-tab="log">Log</button>
      </div>

      <div class="bridge-v7-inner-panel active" data-net="${net.key}" data-bridge-inner-panel="settings">
        <section class="bridge-v7-command">
          <div class="bridge-v7-command-title">Command Preview</div>
          <textarea id="${id(net.key, "commandPreview")}" readonly spellcheck="false" wrap="soft"></textarea>
          <button type="button" class="bridge-v7-copy" data-bridge-action="copy-command" data-net="${net.key}" title="Copy command">⧉</button>
        </section>

        <section class="bridge-v7-toolbar">
          <div class="bridge-v7-buttons">
            <button type="button" class="good" data-bridge-action="start" data-net="${net.key}">Start</button>
            <button type="button" data-bridge-action="stop" data-net="${net.key}">Stop</button>
          </div>

          <div class="bridge-v7-status">
            <label><input id="${id(net.key, "launch")}" type="checkbox"> Launch</label>
            <label><input id="${id(net.key, "restart")}" type="checkbox" checked> Restart</label>
          </div>
        </section>

        ${renderSections(net)}

        <div class="settings-bottom-actions bridge-settings-bottom-actions">
        <button type="button" data-bridge-action="save-settings" data-net="${net.key}">Save Settings</button>
        <button type="button" data-bridge-action="restore-defaults" data-net="${net.key}">Restore Defaults</button>
        <button type="button" data-bridge-action="set-defaults" data-net="${net.key}">Set as Defaults</button>
        </div>

      </div>

      <div class="bridge-v7-inner-panel" data-net="${net.key}" data-bridge-inner-panel="log" hidden>
        <div class="bridge-v7-log-toolbar">
          <button type="button" data-bridge-action="copy-log" data-net="${net.key}">Copy Log</button>
          <button type="button" data-bridge-action="clear-log" data-net="${net.key}">Clear Log</button>
        </div>
        <pre id="${id(net.key, "logOutput")}" class="bridge-v7-log"></pre>
      </div>
</div>`;
}


function bridgeReadInstanceState(net, instanceId) {
  const current = bridgeInstances[net].find((instance) => String(instance.id) === String(instanceId)) || {};
  return {
    id: Date.now() + Math.floor(Math.random() * 1000),
    instance: bridgeReadInstanceField(net, instanceId, "instance") || current.instance || "",
    instanceLogToFile: bridgeReadInstanceField(net, instanceId, "instanceLogToFile") || current.instanceLogToFile || "true",
    instanceVarDiff: bridgeReadInstanceField(net, instanceId, "instanceVarDiff") || current.instanceVarDiff || "true",
    instanceSharesPerMin: bridgeReadInstanceField(net, instanceId, "instanceSharesPerMin") || current.instanceSharesPerMin || "20",
    instanceVarDiffStats: bridgeReadInstanceField(net, instanceId, "instanceVarDiffStats") || current.instanceVarDiffStats || "true",
    instancePow2Clamp: bridgeReadInstanceField(net, instanceId, "instancePow2Clamp") || current.instancePow2Clamp || "true"
  };
}

function bridgeRefreshInstances(net) {
  const container = byId(id(net, "instances"));
  if (container) {
    container.innerHTML = renderInstances(net);
  }
  updateCommand(net);
}

function addInstance(net) {
  bridgeEnsureInstanceState(net);
  bridgeInstances[net].push({
    id: Date.now() + Math.floor(Math.random() * 1000),
    instance: "",
    instanceLogToFile: "true",
    instanceVarDiff: "true",
    instanceSharesPerMin: "20",
    instanceVarDiffStats: "true",
    instancePow2Clamp: "true"
  });
  bridgeRefreshInstances(net);
}

function duplicateInstance(net, instanceId) {
  bridgeEnsureInstanceState(net);
  bridgeInstances[net].push(bridgeReadInstanceState(net, instanceId));
  bridgeRefreshInstances(net);
}

function removeInstance(net, instanceId) {
  bridgeEnsureInstanceState(net);
  if (bridgeInstances[net].length <= 1) return;
  bridgeInstances[net] = bridgeInstances[net].filter((instance) => String(instance.id) !== String(instanceId));
  bridgeRefreshInstances(net);
}

function selectInstance(net, instanceId) {
  const root = byId(id(net, "instances"));
  if (!root) return;

  root.querySelectorAll("[data-bridge-instance-panel]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.bridgeInstancePanel === String(instanceId));
  });

  root.querySelectorAll("[data-bridge-action='select-instance']").forEach((button) => {
    button.classList.toggle("active", button.dataset.instanceId === String(instanceId));
  });
}

function renderAllNetworks(root) {
  const host = root.querySelector("#bridgeNetworkPanels");
  if (!host) return;
  host.innerHTML = BRIDGE_NETWORKS.map(renderNetworkPanel).join("");


  setTimeout(kgwInstallBridgeLogAutoScrollControlsR27, 0);
}

function bridgeProfile(net) {
  return BRIDGE_NETWORKS.find((item) => item.key === net);
}

function bridgeNodeMode(net) {
  const value = v(net, "nodeMode");
  return value === "inprocess" ? "inprocess" : "external";
}

function bridgeHasConfig(net) {
  return Boolean(v(net, "config"));
}

function bridgeControl(net, name) {
  return byId(id(net, name));
}

function bridgeControlCard(el) {
  return el ? el.closest(".bridge-v7-card") : null;
}

function bridgeSetDisabled(net, name, disabled, reason = "") {
  const el = bridgeControl(net, name);
  if (!el) return;

  el.disabled = Boolean(disabled);

  const card = bridgeControlCard(el);
  if (card) {
    card.classList.toggle("bridge-v7-mode-disabled", Boolean(disabled));
    card.title = disabled ? reason : "";
  }
}

function bridgeSyncModeControls(net) {
  const profile = bridgeProfile(net);
  if (!profile) return;

  const configMode = bridgeHasConfig(net);
  const nodeMode = bridgeNodeMode(net);
  const internalMinerEnabled = c(net, "internalCpuMiner");

  const explicitBridgeFields = [
    "testnet",
    "nodeMode",
    "appdir",
    "kaspadAddress",
    "blockWaitTime",
    "printStats",
    "logToFile",
    "healthCheckPort",
    "webDashboardPort",
    "varDiff",
    "sharesPerMin",
    "varDiffStats",
    "extranonceSize",
    "pow2Clamp",
    "coinbaseTagSuffix",
    "approxGeoLookup",
    "stratumPort",
    "minShareDiff",
    "promPort",
    "internalCpuMiner",
    "internalCpuMinerAddress",
    "internalCpuMinerThreads",
    "internalCpuMinerThrottleMs",
    "internalCpuMinerTemplatePollMs"
  ];

  for (const name of explicitBridgeFields) {
    bridgeSetDisabled(net, name, configMode, "Config mode is active. Clear --config to edit explicit CLI flags.");
  }

  if (configMode) return;

  const testnetControl = bridgeControl(net, "testnet");
  if (testnetControl) {
    testnetControl.checked = Boolean(profile.testnet);
  }
  bridgeSetDisabled(net, "testnet", true, "Network identity is owned by the selected Mainnet/Testnet tab.");

  if (nodeMode === "external") {
    bridgeSetDisabled(net, "kaspadAddress", false, "");
  } else {
    bridgeSetDisabled(net, "kaspadAddress", true, "In-process mode owns kaspad args after the -- separator.");
  }

  for (const name of [
    "internalCpuMinerAddress",
    "internalCpuMinerThreads",
    "internalCpuMinerThrottleMs",
    "internalCpuMinerTemplatePollMs"
  ]) {
    bridgeSetDisabled(net, name, !internalMinerEnabled, "Enable --internal-cpu-miner first.");
  }
}

function bridgeSyncAllModeControls() {
  BRIDGE_NETWORKS.forEach((item) => bridgeSyncModeControls(item.key));
}

function addRawValue(lines, flag, value) {
  const normalized = String(value || "").trim();
  if (normalized) lines.push(`${flag}=${normalized}`);
}













function bridgeLogLineBelongsToBridge(_line) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  return true;
}


// KGW_BRIDGE_LOG_AUTOSCROLL_CONTROLS_R27_START
function kgwBridgeLogAutoScrollKeyR27(net) {
  return `kgw.bridge.log.autoscroll.${net}`;
}

function kgwBridgeLogAutoScrollEnabledR27(net) {
  try {
    return localStorage.getItem(kgwBridgeLogAutoScrollKeyR27(net)) !== "0";
  } catch (_) {
    return true;
  }
}

function kgwBridgeSetLogAutoScrollR27(net, enabled) {
  try {
    localStorage.setItem(kgwBridgeLogAutoScrollKeyR27(net), enabled ? "1" : "0");
  } catch (_) {}

  const out = byId(id(net, "logOutput"));
  if (enabled && out) out.scrollTop = out.scrollHeight;
}

function kgwInstallBridgeLogAutoScrollControlsR27() {
  if (typeof document === "undefined") return;
  if (!Array.isArray(BRIDGE_NETWORKS)) return;

  for (const profile of BRIDGE_NETWORKS) {
    const net = profile.key;
    const out = byId(id(net, "logOutput"));
    if (!out) continue;

    const controlId = id(net, "logAutoScrollR27");
    if (byId(controlId)) continue;

    const label = document.createElement("label");
    label.className = "kgw-log-autoscroll-toggle";
    label.setAttribute("data-kgw-log-autoscroll", "bridge");
    label.setAttribute("title", "Keep the log pinned to the newest raw line.");

    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.id = controlId;
    checkbox.checked = kgwBridgeLogAutoScrollEnabledR27(net);
    checkbox.addEventListener("change", () => kgwBridgeSetLogAutoScrollR27(net, checkbox.checked));

    const span = document.createElement("span");
    span.textContent = kgwI18nTextR41("common.autoScroll", "Auto-scroll");

    label.appendChild(checkbox);
    label.appendChild(span);

    const panel = out.closest(".bridge-v7-inner-panel, [data-bridge-inner-panel], [data-inner-panel], [data-bridge-panel], [data-panel]") || out.parentElement;
    const toolbar =
      panel?.querySelector(".bridge-v7-log-toolbar, .bridge-log-toolbar, [data-bridge-log-toolbar]") ||
      out.parentElement?.querySelector(".bridge-v7-log-toolbar, .bridge-log-toolbar, [data-bridge-log-toolbar]");

    if (toolbar) {
      toolbar.appendChild(label);
    } else {
      out.parentElement?.insertBefore(label, out);
    }
  }
}
// KGW_BRIDGE_LOG_AUTOSCROLL_CONTROLS_R27_END

function appendLog(net, message) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  const out = byId(id(net, "logOutput"));
  if (!out) return;

  const rawText = String(message ?? "");
  if (rawText.length === 0) {
    return;
  }

  const previousText = String(out.textContent || "");
  const lines = previousText ? previousText.split("\n") : [];

  for (const rawLine of rawText.split(/\r?\n/)) {
    lines.push(rawLine.trimEnd());
  }

  while (lines.length > 3000) lines.shift();

  out.textContent = lines.join("\n");
  if (kgwBridgeLogAutoScrollEnabledR27(net)) out.scrollTop = out.scrollHeight;
}
function buildCommandLines(net) {
  bridgeSyncModeControls(net);
  bridgeEnsureInstanceState(net);

  const profile = bridgeProfile(net);
  const lines = ["stratum-bridge"];
  const kaspadArgs = [];
  const nodeMode = bridgeNodeMode(net);
  const configValue = v(net, "config");

  if (configValue) {
    addRawValue(lines, "--config", configValue);
    addValue(lines, net, "nodeMode", "--node-mode");
    addValue(lines, net, "webDashboardPort", "--web-dashboard-port");
    return lines;
  }

  if (profile?.testnet) {
    lines.push("--testnet");
  }

  addValue(lines, net, "nodeMode", "--node-mode");
  addValue(lines, net, "appdir", "--appdir");

  if (nodeMode === "external") {
    addValue(lines, net, "kaspadAddress", "--kaspad-address");
  } else if (nodeMode === "inprocess") {
    if (profile?.testnet) {
      kaspadArgs.push("--testnet");
      if (profile.netsuffix) {
        kaspadArgs.push(`--netsuffix=${profile.netsuffix}`);
      }
    }
  }

  addValue(lines, net, "blockWaitTime", "--block-wait-time");
  addValue(lines, net, "printStats", "--print-stats");
  addValue(lines, net, "logToFile", "--log-to-file");
  addValue(lines, net, "healthCheckPort", "--health-check-port");
  addValue(lines, net, "webDashboardPort", "--web-dashboard-port");
  addValue(lines, net, "varDiff", "--var-diff");
  addValue(lines, net, "sharesPerMin", "--shares-per-min");
  addValue(lines, net, "varDiffStats", "--var-diff-stats");
  addValue(lines, net, "extranonceSize", "--extranonce-size");
  addValue(lines, net, "pow2Clamp", "--pow2-clamp");
  addValue(lines, net, "coinbaseTagSuffix", "--coinbase-tag-suffix");
  addBoolValue(lines, net, "approxGeoLookup", "--approximate-geo-lookup");
  addValue(lines, net, "stratumPort", "--stratum-port");
  addValue(lines, net, "minShareDiff", "--min-share-diff");
  addValue(lines, net, "promPort", "--prom-port");

  for (const instance of bridgeInstances[net]) {
    const instanceDefinition = bridgeNormalizeInstance(bridgeReadInstanceField(net, instance.id, "instance") || instance.instance);
    if (instanceDefinition) {
      lines.push(`--instance=${instanceDefinition}`);
    }

    bridgeInstanceBoolArg(lines, net, instance.id, "instanceLogToFile", "--instance-log-to-file");
    bridgeInstanceBoolArg(lines, net, instance.id, "instanceVarDiff", "--instance-var-diff");
    bridgeInstanceValueArg(lines, net, instance.id, "instanceSharesPerMin", "--instance-shares-per-min");
    bridgeInstanceBoolArg(lines, net, instance.id, "instanceVarDiffStats", "--instance-var-diff-stats");
    bridgeInstanceBoolArg(lines, net, instance.id, "instancePow2Clamp", "--instance-pow2-clamp");
  }

  if (c(net, "internalCpuMiner")) {
    if (net.key !== "mainnet") {
      addFlag(lines, net, "internalCpuMiner", "--internal-cpu-miner");
      addValue(lines, net, "internalCpuMinerAddress", "--internal-cpu-miner-address");
      addValue(lines, net, "internalCpuMinerThreads", "--internal-cpu-miner-threads");
      addValue(lines, net, "internalCpuMinerThrottleMs", "--internal-cpu-miner-throttle-ms");
      addValue(lines, net, "internalCpuMinerTemplatePollMs", "--internal-cpu-miner-template-poll-ms");
    }
  }

  if (kaspadArgs.length) {
    lines.push("--", ...kaspadArgs);
  }

  return lines;
}













function kgwExtractBridgeOwnerFlags(result) {
  const raw = stringifyRuntimeResult(result);
  const fields = {};

  for (const part of raw.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;

    const key = part.slice(0, index).trim();
    const value = part.slice(index + 1).trim();
    if (key) fields[key] = value;
  }

  return fields.flags || "";
}

async function kgwLoadBridgeOwnerCommandPreview(net, fallbackText) {
  const invoke = getTauriInvoke();
  if (!invoke) return fallbackText;

  try {
    const result = await invokeWithTimeout(
      invoke,
      KGW_BRIDGE_RUNTIME_FLAGS_OWNER_COMMAND,
      { network: net },
      KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS
    );

    const flags = kgwExtractBridgeOwnerFlags(result);
    return flags ? "stratum-bridge " + flags : fallbackText;
  } catch (_) {
    return fallbackText;
  }
}


function updateCommand(net) {
  const preview = byId(id(net, "commandPreview"));
  if (!preview) return "";

  try {
    bridgeSyncModeControls(net);

    const lines = buildCommandLines(net);
    const duplicatePorts = bridgeDuplicatePorts(lines);
    const first = lines.shift() || "stratum-bridge";
    const text = lines.length ? `${first} ${lines.join(" ")}` : first;

    preview.value = text;
    preview.textContent = text;
    preview.dataset.kgwBridgeCommandOwner = "readme-instance-command-owner";
    preview.dataset.kgwBridgeNetwork = net;

    if (duplicatePorts.length) {
      preview.dataset.kgwBridgeCommandWarning = `duplicate ports: ${duplicatePorts.join(",")}`;
      preview.classList.add("bridge-v7-command-warning");
    } else {
      delete preview.dataset.kgwBridgeCommandWarning;
      preview.classList.remove("bridge-v7-command-warning");
    }

    return text;
  } catch (error) {
    const message = "stratum-bridge # command preview error: " + normalizeRuntimeError(error);
    preview.value = message;
    preview.textContent = message;
    preview.dataset.kgwBridgeCommandOwner = "readme-instance-command-owner-error";
    preview.dataset.kgwBridgeNetwork = net;
    return message;
  }
}









function updateAllCommands() {
  bridgeSyncAllModeControls();
  BRIDGE_NETWORKS.forEach((net) => updateCommand(net.key));
}













function installNetworkTabs(root) {
  // KGW_R63_DIRECT_BRIDGE_NETWORK_TAB_SWITCH_OWNER
  const networkTabSelector = "[data-bridge-network-tab]";
  const networkPanelSelector = "[data-bridge-network-panel]";

  function normalizeNetFromElement(element) {
    if (!element) return "";
    return element.dataset.net || element.dataset.bridgeNetworkTab || element.dataset.bridgeNetworkPanel || "";
  }

  function allNetworkTabs() {
    return Array.from(root.querySelectorAll(networkTabSelector));
  }

  function allNetworkPanels() {
    return Array.from(root.querySelectorAll(networkPanelSelector));
  }

  function selectBridgeNetwork(net, reason = "manual") {
    const normalized = String(net || "").trim();

    if (!normalized) return;

    const tabs = allNetworkTabs();
    const panels = allNetworkPanels();

    for (const tab of tabs) {
      const tabNet = normalizeNetFromElement(tab);
      const active = tabNet === normalized;

      tab.classList.toggle("active", active);
      tab.classList.toggle("is-active", active);
      tab.classList.toggle("selected", active);
      tab.setAttribute("aria-selected", active ? "true" : "false");
      tab.dataset.active = active ? "true" : "false";
    }

    for (const panel of panels) {
      const panelNet = normalizeNetFromElement(panel);
      const active = panelNet === normalized;

      panel.hidden = !active;
      panel.classList.toggle("active", active);
      panel.classList.toggle("is-active", active);
      panel.dataset.active = active ? "true" : "false";
      panel.style.display = active ? "" : "none";
    }

    if (typeof updateCommand === "function") {
      updateCommand(normalized);
    }

    if (typeof kgwBridgeR51RefreshOne === "function") {
      window.setTimeout(() => kgwBridgeR51RefreshOne(normalized, "network-tab-" + reason), 50);
      window.setTimeout(() => kgwBridgeR51RefreshOne(normalized, "network-tab-" + reason + "+700ms"), 700);
    }

    if (typeof appendLog === "function") {
    }
  }

  root.addEventListener("click", (event) => {
    const tab = event.target.closest(networkTabSelector);

    if (!tab || !root.contains(tab)) return;

    const net = normalizeNetFromElement(tab);

    if (!net) return;

    event.preventDefault();
    event.stopPropagation();

    selectBridgeNetwork(net, "click");
  }, true);

  const existingActiveTab = allNetworkTabs().find((tab) => {
    return tab.classList.contains("active") ||
      tab.classList.contains("is-active") ||
      tab.getAttribute("aria-selected") === "true" ||
      tab.dataset.active === "true";
  });

  const defaultTab =
    existingActiveTab ||
    allNetworkTabs().find((tab) => normalizeNetFromElement(tab) === "mainnet") ||
    allNetworkTabs()[0];

  if (defaultTab) {
    selectBridgeNetwork(normalizeNetFromElement(defaultTab), "initial");
  }

  window.kgwBridgeSelectNetworkTabR63 = selectBridgeNetwork;
}

function installDelegatedTabs(root) {
  root.addEventListener("click", (event) => {
    const innerTab = event.target.closest("[data-bridge-inner-tab]");
    if (innerTab) {
      const net = innerTab.dataset.net;
      const selected = innerTab.dataset.bridgeInnerTab;
      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      panel.querySelectorAll("[data-bridge-inner-tab]").forEach((item) => {
        item.classList.toggle("active", item === innerTab);
      });

      panel.querySelectorAll("[data-bridge-inner-panel]").forEach((item) => {
        const active = item.dataset.bridgeInnerPanel === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });

      return;
    }

    const sectionTab = event.target.closest("[data-bridge-section-tab]");
    if (sectionTab) {
      const net = sectionTab.dataset.net;
      const selected = sectionTab.dataset.bridgeSectionTab;
      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      panel.querySelectorAll("[data-bridge-section-tab]").forEach((item) => {
        item.classList.toggle("active", item === sectionTab);
      });

      panel.querySelectorAll("[data-bridge-section-panel]").forEach((item) => {
        const active = item.dataset.bridgeSectionPanel === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });

      return;
    }

    const instanceTab = event.target.closest("[data-instance-tab]");
    if (instanceTab) {
      const net = instanceTab.dataset.net;
      const selected = Number(instanceTab.dataset.instanceTab);
      activeInstance[net] = selected;

      const panel = root.querySelector(`[data-bridge-network-panel="${net}"]`);

      panel.querySelectorAll("[data-instance-tab]").forEach((item) => {
        item.classList.toggle("active", Number(item.dataset.instanceTab) === selected);
      });

      panel.querySelectorAll("[data-instance-panel]").forEach((item) => {
        const active = Number(item.dataset.instancePanel) === selected;
        item.classList.toggle("active", active);
        item.hidden = !active;
      });
    }
  });
}

// KGW_BRIDGE_INTEGRATED_RUNTIME_LINKAGE_V1: readable Bridge runtime response + duplicate-click guard.
const KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS = 7000;
const KGW_BRIDGE_RUNTIME_FLAGS_OWNER_COMMAND = "rk_integrated_bridge_runtime_flags_v1";
const KGW_BRIDGE_RUNTIME_IN_FLIGHT = new Set();

function getTauriInvoke() {
  const tauri = window.__TAURI__;
  return tauri?.core?.invoke || tauri?.invoke || window.__TAURI_INVOKE__ || null;
}

function stringifyRuntimeResult(result) {
  if (result == null) return "No response";
  if (typeof result === "string") return result;
  try {
    return JSON.stringify(result);
  } catch {
    return String(result);
  }
}

function normalizeRuntimeError(error) {
  if (error == null) return "Unknown backend error";
  if (typeof error === "string") return error;
  if (error.message) return error.message;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

function parseRuntimeKeyValueResponse(value) {
  const raw = stringifyRuntimeResult(value);
  const text = raw.trim();

  if (!text || !text.includes("=")) {
    return { raw: text, fields: {} };
  }

  const fields = {};
  for (const part of text.split(";")) {
    const index = part.indexOf("=");
    if (index <= 0) continue;

    const key = part.slice(0, index).trim();
    const fieldValue = part.slice(index + 1).trim();
    if (key) fields[key] = fieldValue;
  }

  return { raw: text, fields };
}

function yesNo(value) {
  if (value === true || value === "true") return "yes";
  if (value === false || value === "false") return "no";
  return value == null || value === "" ? "unknown" : String(value);
}

function readableRuntimeSummary(title, result) {
  const parsed = parseRuntimeKeyValueResponse(result);
  const f = parsed.fields;

  if (!Object.keys(f).length) {
    return title + ": " + parsed.raw;
  }

  const blocked = f.start_blocked === "true" || f.start_allowed === "false";
  const lines = [];

  if (blocked) {
    lines.push(title + ": blocked");
  } else if (f.running === "true") {
    lines.push(title + ": running");
  } else if (f.running === "false") {
    lines.push(title + ": stopped");
  } else {
    lines.push(title + ": response");
  }

  if (f.block_reason) lines.push("Reason: " + f.block_reason);
  if (f.network) lines.push("Network: " + f.network);
  if (f.dynamic_preflight_passed) lines.push("Preflight: " + (f.dynamic_preflight_passed === "true" ? "passed" : "failed"));
  if (f.explicit_start_enabled) lines.push("Explicit start enabled: " + yesNo(f.explicit_start_enabled));
  if (f.compile_time_start_enabled) lines.push("Compile-time start enabled: " + yesNo(f.compile_time_start_enabled));
  if (f.runtime_starts_processes) lines.push("Runtime starts processes: " + yesNo(f.runtime_starts_processes));
  if (f.running) lines.push("Running: " + yesNo(f.running));
  if (f.healthy) lines.push("Healthy: " + yesNo(f.healthy));
  if (f.message) lines.push("Message: " + f.message);

  return lines.join("\n");
}

function appendReadableRuntimeResult(_net, _title, _result) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  // Runtime action summaries are UI/status data and must not be written into the raw bridge log pane.
}

function buildApplyPayload(net, command) {

  if (command === "kgw_kgw_apply_node_settings_v1") {
    const preview = updateCommand(net) || byId(id(net, "commandPreview"))?.value || "";

    return {
      network: net,
      runtimeRole: "bridge",
      nodeKind: "remote",
      bridgeKind: "official-external-node",
      nodeCommandPreview: "",
      bridgeCommandPreview: preview,
    };
  }

  if (
    command === "kgw_kgw_disable_network_v1" ||
    command === "kgw_runtime_owner_status_v1" ||
    command === "kgw_kgw_runtime_logs_v1"
  ) {
    return { network: net, runtimeRole: "bridge" };
  }

  return { network: net };
}
function invokeWithTimeout(invoke, command, args, timeoutMs) {
  let timer = null;

  const timeout = new Promise((_, reject) => {
    timer = window.setTimeout(() => {
      reject(new Error(command + " timed out after " + timeoutMs + "ms"));
    }, timeoutMs);
  });

  return Promise.race([
    invoke(command, args),
    timeout
  ]).finally(() => {
    if (timer != null) window.clearTimeout(timer);
  });
}

async function invokeBridgeIntegratedRuntime(command, net) {
  const invoke = getTauriInvoke();
  if (!invoke) {
    throw new Error("Tauri invoke is unavailable in this window.");
  }

  return await invokeWithTimeout(invoke, command, buildApplyPayload(net, command), KGW_BRIDGE_RUNTIME_INVOKE_TIMEOUT_MS);
}


async function runBridgeIntegratedAction(action, net) {

  const commandByAction = {
    start: "kgw_kgw_apply_node_settings_v1",
    stop: "kgw_kgw_disable_network_v1"
  };

  const command = commandByAction[action];
  if (!command) return false;

  const inFlightKey = net + ":" + action;

  if (KGW_BRIDGE_RUNTIME_IN_FLIGHT.has(inFlightKey)) {
    return true;
  }

  KGW_BRIDGE_RUNTIME_IN_FLIGHT.add(inFlightKey);

  try {
    const preview = updateCommand(net) || byId(id(net, "commandPreview"))?.value || "";

    const result = await invokeBridgeIntegratedRuntime(command, net);
    const raw = stringifyRuntimeResult(result);
    const parsed = parseRuntimeKeyValueResponse(result);
    const fields = parsed.fields || {};

    if (action === "start") {
      const confirmedStarted =
        /parallel-owned-self-worker\s+started/i.test(raw) ||
        /parallel-owned-self-worker\s+already\s+running/i.test(raw) ||
        (/role=bridge/i.test(raw) && /started|running=true|already running/i.test(raw)) ||
        fields.running === "true" ||
        fields.bridge_running === "true" ||
        fields.bridge_owner_active === "true";

      const blocked =
        fields.start_blocked === "true" ||
        fields.start_allowed === "false" ||
        /blocked|not enabled|failed/i.test(raw);

      if (confirmedStarted && !blocked) {

      } else {

      }
    }

    if (action === "stop") {
      const confirmedStopped =
        /parallel-owned-self-worker\s+stopped/i.test(raw) ||
        fields.running === "false" ||
        fields.bridge_running === "false";

      if (confirmedStopped) {
      } else {
      }
    }
  } catch (error) {
  }
finally {
    KGW_BRIDGE_RUNTIME_IN_FLIGHT.delete(inFlightKey);
  }

  return true;
}
/* KGW_R51_DIRECT_BRIDGE_LOG_RUNTIME_SETTINGS_OWNER */
const KGW_BRIDGE_R51_STORAGE_PREFIX = "kgw.bridge.direct.v51.";
const KGW_BRIDGE_R51_LAST_STATUS = {};
const KGW_BRIDGE_R51_LAST_LOGS = {};
const KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE = {};
let KGW_BRIDGE_R51_TIMER = null;

function kgwBridgeR51Keys() {
  return BRIDGE_NETWORKS.map((item) => item.key);
}

function kgwBridgeR51Panel(net) {
  return document.querySelector(`[data-bridge-network-panel="${net}"]`);
}

function kgwBridgeR51Fields(net) {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return [];

  return Array.from(panel.querySelectorAll("input, select, textarea")).filter((field) => {
    if (!field.id || !field.id.startsWith(`bridge-${net}-`)) return false;
    if (field.id.endsWith("-commandPreview")) return false;
    if (field.id.endsWith("-logOutput")) return false;
    if (field.readOnly) return false;
    return true;
  });
}

function kgwBridgeR51ReadSettings(net) {
  const values = {};

  for (const field of kgwBridgeR51Fields(net)) {
    values[field.id] = field.type === "checkbox"
      ? { type: "checkbox", checked: Boolean(field.checked) }
      : { type: "value", value: String(field.value ?? "") };
  }

  return values;
}

function kgwBridgeR51WriteSettings(net, values) {
  if (!values || typeof values !== "object") return;

  for (const field of kgwBridgeR51Fields(net)) {
    const item = values[field.id];
    if (!item) continue;

    if (field.type === "checkbox") {
      field.checked = Boolean(item.checked);
    } else if ("value" in item) {
      field.value = String(item.value ?? "");
    }

    field.dispatchEvent(new Event("input", { bubbles: true }));
    field.dispatchEvent(new Event("change", { bubbles: true }));
  }

  updateCommand(net);
}

function kgwBridgeR51Store(key, value) {
  localStorage.setItem(KGW_BRIDGE_R51_STORAGE_PREFIX + key, JSON.stringify(value));
}

function kgwBridgeR51Load(key) {
  try {
    return JSON.parse(localStorage.getItem(KGW_BRIDGE_R51_STORAGE_PREFIX + key) || "null");
  } catch {
    return null;
  }
}

function kgwBridgeR51CaptureFactoryDefaults() {
  for (const net of kgwBridgeR51Keys()) {
    if (!kgwBridgeR51Load("factory:" + net)) {
      kgwBridgeR51Store("factory:" + net, kgwBridgeR51ReadSettings(net));
    }
  }
}

function kgwBridgeR51LoadSavedSettings() {
  for (const net of kgwBridgeR51Keys()) {
    const saved = kgwBridgeR51Load("saved:" + net);
    if (saved) {
      kgwBridgeR51WriteSettings(net, saved);
    }
  }
}

function kgwBridgeR51SaveSettings(net) {
  kgwBridgeR51Store("saved:" + net, kgwBridgeR51ReadSettings(net));
}

function kgwBridgeR51SetAsDefaults(net) {
  kgwBridgeR51Store("default:" + net, kgwBridgeR51ReadSettings(net));
}

function kgwBridgeR51RestoreDefaults(net) {
  const defaults = kgwBridgeR51Load("default:" + net) || kgwBridgeR51Load("factory:" + net);
  kgwBridgeR51WriteSettings(net, defaults);
}

function kgwBridgeR51IsRunning(text) {
  const value = String(text || "");
  return /running=true/.test(value) || /bridge_running=true/.test(value) || /bridge_owner_active=true/.test(value);
}

function kgwBridgeR51SetRuntimeButtons(net, running) {
  const panel = kgwBridgeR51Panel(net);
  if (!panel) return;

  const start = panel.querySelector(`[data-bridge-action="start"][data-net="${net}"]`);
  const stop = panel.querySelector(`[data-bridge-action="stop"][data-net="${net}"]`);

  if (start) {
    start.disabled = Boolean(running);
    start.style.opacity = running ? "0.45" : "";
    start.style.cursor = running ? "not-allowed" : "";
    start.title = running ? "Bridge is running. Stop it before starting again." : "Start bridge";
  }

  if (stop) {
    stop.disabled = !running;
    stop.style.opacity = running ? "" : "0.45";
    stop.style.cursor = running ? "" : "not-allowed";
    stop.title = running ? "Stop bridge" : "Bridge is not running";
  }
}

function kgwBridgeR51Delta(previous, current) {
  const before = String(previous || "");
  const after = String(current || "");

  if (!after || before === after) return "";
  if (after.startsWith(before)) return after.slice(before.length).trim();

  return after.trim();
}

function kgwBridgeR51MaybeActivityNotice(net, statusText) {
  const now = Date.now();
  const last = KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE[net] || 0;

  if (now - last < 15000) return;

  if (!kgwBridgeR51IsRunning(statusText)) return;

  KGW_BRIDGE_R51_LAST_ACTIVITY_NOTICE[net] = now;

}

async function kgwBridgeR51RefreshOne(net, reason = "live") {
  try {
    const status = stringifyRuntimeResult(await invokeBridgeIntegratedRuntime("kgw_runtime_owner_status_v1", net));
    kgwBridgeR51SetRuntimeButtons(net, kgwBridgeR51IsRunning(status));

    if (KGW_BRIDGE_R51_LAST_STATUS[net] !== status) {
      KGW_BRIDGE_R51_LAST_STATUS[net] = status;
      
    }

    kgwBridgeR51MaybeActivityNotice(net, status);
  } catch (error) {
    kgwBridgeR51SetRuntimeButtons(net, false);
  }

  try {
    const logs = stringifyRuntimeResult(await invokeBridgeIntegratedRuntime("kgw_kgw_runtime_logs_v1", net));
    const delta = kgwBridgeR51Delta(KGW_BRIDGE_R51_LAST_LOGS[net], logs);

    if (delta) {
      KGW_BRIDGE_R51_LAST_LOGS[net] = logs;
      appendLog(net, delta);
    }
  } catch {
    // Runtime may not be ready yet.
  }
}

function kgwBridgeR51RefreshAll(reason = "live") {
  for (const net of kgwBridgeR51Keys()) {
    kgwBridgeR51RefreshOne(net, reason);
  }
}

function kgwBridgeR51StartLiveRefresh() {
  if (KGW_BRIDGE_R51_TIMER != null) {
    clearInterval(KGW_BRIDGE_R51_TIMER);
  }

  kgwBridgeR51RefreshAll("initial");

  KGW_BRIDGE_R51_TIMER = setInterval(() => {
    kgwBridgeR51RefreshAll("poll");
  }, 700);
}

function installKgwBridgeR51BottomStyle() {
  if (document.getElementById("kgw-bridge-r51-bottom-style")) return;

  const style = document.createElement("style");
  style.id = "kgw-bridge-r51-bottom-style";
  style.textContent = `
    [data-bridge-network-panel] {
      position: relative;
      min-height: 680px;
      padding-bottom: 48px;
    }

    .bridge-settings-bottom-actions {
      position: absolute;
      right: 12px;
      bottom: 8px;
      display: flex;
      align-items: center;
      justify-content: flex-end;
      gap: 8px;
      z-index: 2;
    }

    .bridge-settings-bottom-actions button {
      min-width: 112px;
      height: 28px;
      padding: 4px 10px;
      border: 1px solid rgba(148, 163, 184, 0.55);
      background: rgba(80, 80, 80, 0.9);
      color: #fff;
      cursor: pointer;
      font-size: 12px;
      line-height: 1;
    }
  `;
  document.head.appendChild(style);
}



/* KGW_BRIDGE_ACTION_AND_LOG_FEEDBACK_OWNER_V1 */
function kgwBridgeCurrentVisibleNetwork(root) {
  const activePanel = root.querySelector("[data-bridge-network-panel].active, [data-bridge-network-panel].is-active, [data-bridge-network-panel][data-active='true']");
  if (activePanel?.dataset?.bridgeNetworkPanel) return activePanel.dataset.bridgeNetworkPanel;

  const activeTab = root.querySelector("[data-bridge-network-tab].active, [data-bridge-network-tab].is-active, [data-bridge-network-tab][aria-selected='true'], [data-bridge-network-tab][data-active='true']");
  if (activeTab?.dataset?.bridgeNetworkTab) return activeTab.dataset.bridgeNetworkTab;
  if (activeTab?.dataset?.net) return activeTab.dataset.net;

  return "mainnet";
}


/* KGW_BRIDGE_LOG_FEEDBACK_I18N_OWNER_V1 */

function kgwBridgeTranslateRuntime(key, fallback) {
  const runtime = window.kgwT || window.kgwI18n || window.__kgwT;
  if (typeof runtime === "function") {
    try {
      const value = runtime(key);
      if (value && value !== key) return value;
    } catch {
      // Translation fallback must never break button feedback.
    }
  }

  const dict =
    window.__kgwI18nDictR107 ||
    window.__kgwI18nDict ||
    window.kgwI18nDict ||
    window.__KGW_I18N_DICT__;

  if (dict && typeof dict === "object") {
    const flat = dict[key];
    if (typeof flat === "string" && flat.trim()) return flat;

    let node = dict;
    for (const part of String(key).split(".")) {
      if (!node || typeof node !== "object") {
        node = null;
        break;
      }
      node = node[part];
    }

    if (typeof node === "string" && node.trim()) return node;
  }

  return fallback || key;
}

function kgwFlashLogActionButton(button, doneLabel) {
  if (!button) return;

  if (!button.dataset.kgwOriginalLabel) {
    button.dataset.kgwOriginalLabel = String(button.textContent || "").trim();
  }

  button.dataset.kgwDoneLabel = doneLabel;
  button.classList.add("kgw-log-action-feedback");

  window.clearTimeout(button.__kgwLogActionFeedbackTimer);
  button.__kgwLogActionFeedbackTimer = window.setTimeout(() => {
    button.classList.remove("kgw-log-action-feedback");
    delete button.dataset.kgwDoneLabel;
  }, 1400);
}

function kgwBridgeLogCpuMinerDiagnostic(_net) {
  // KGW_BRIDGE_RAW_NO_FILTER_R20
  // Diagnostics are not raw bridge stdout/stderr.
}

function installActions(root) {
  root.addEventListener("input", (event) => {
    const field = event.target?.dataset?.bridgeInstanceField;
    if (field) {
      const panel = event.target.closest("[data-bridge-instance-panel]");
      const instanceId = panel?.dataset?.bridgeInstancePanel;
      const net = event.target.id?.split("-")?.[1];

      if (net && instanceId) {
        const instance = bridgeInstances[net]?.find((item) => String(item.id) === String(instanceId));
        if (instance) {
          instance[field] = event.target.value;
        }
      }
    }

    bridgeSyncAllModeControls();
    updateAllCommands();
  });

  root.addEventListener("change", (event) => {
    const field = event.target?.dataset?.bridgeInstanceField;
    if (field) {
      const panel = event.target.closest("[data-bridge-instance-panel]");
      const instanceId = panel?.dataset?.bridgeInstancePanel;
      const net = event.target.id?.split("-")?.[1];

      if (net && instanceId) {
        const instance = bridgeInstances[net]?.find((item) => String(item.id) === String(instanceId));
        if (instance) {
          instance[field] = event.target.value;
        }
      }
    }

    bridgeSyncAllModeControls();
    updateAllCommands();
  });

  root.addEventListener("click", async (event) => {
    const button = event.target.closest("[data-bridge-action]");
    if (!button || !root.contains(button)) return;

    const action = button.dataset.bridgeAction;
    const net = button.dataset.net || button.dataset.network || kgwBridgeCurrentVisibleNetwork(root);
    const instanceId = button.dataset.instanceId;

    if (!net) {
      return;
    }

    if (action === "add-instance") {
      addInstance(net);
      return;
    }

    if (action === "duplicate-instance") {
      duplicateInstance(net, instanceId);
      return;
    }

    if (action === "remove-instance") {
      removeInstance(net, instanceId);
      return;
    }

    if (action === "select-instance") {
      selectInstance(net, instanceId);
      return;
    }

    if (action === "start" || action === "stop") {
      if (button.dataset.kgwBusy === "1") {
        return;
      }

      button.dataset.kgwBusy = "1";
      button.disabled = true;

      try {
        if (action === "start") {
          kgwBridgeLogCpuMinerDiagnostic(net);
        }

        await runBridgeIntegratedAction(action, net);
      } finally {
        delete button.dataset.kgwBusy;
        button.disabled = false;

        if (typeof kgwBridgeR51RefreshOne === "function") {
          await kgwBridgeR51RefreshOne(net, action + "-final");
          window.setTimeout(() => kgwBridgeR51RefreshOne(net, action + "+700ms"), 700);
          window.setTimeout(() => kgwBridgeR51RefreshOne(net, action + "+1800ms"), 1800);
        }
      }

      return;
    }

    if (action === "save-settings") {
      kgwBridgeR51SaveSettings(net);
      return;
    }

    if (action === "restore-defaults") {
      kgwBridgeR51RestoreDefaults(net);
      return;
    }

    if (action === "set-defaults") {
      kgwBridgeR51SetAsDefaults(net);
      return;
    }

    if (action === "copy-command") {
      try {
        const text = updateCommand(net) || byId(id(net, "commandPreview"))?.value || "";
        await navigator.clipboard?.writeText(text);
        kgwFlashLogActionButton(button, kgwBridgeTranslateRuntime("log.copied", "Copied"));
      } catch (error) {
      }
      return;
    }

    if (action === "copy-log") {
      try {
        const text = byId(id(net, "logOutput"))?.textContent || "";
        await navigator.clipboard?.writeText(text);
        kgwFlashLogActionButton(button, kgwBridgeTranslateRuntime("log.copied", "Copied"));
      } catch (error) {
      }
      return;
    }

    if (action === "clear-log") {
      const out = byId(id(net, "logOutput"));
      if (out) out.textContent = "";
      KGW_BRIDGE_R51_LAST_LOGS[net] = "";
      kgwFlashLogActionButton(button, kgwBridgeTranslateRuntime("log.deleted", "Deleted"));
      return;
    }
  });
}



export async function initKaspaBridgeTab(root) {

const bridgeRoot = root || document.getElementById("kaspa-bridge");
  if (!bridgeRoot || bridgeRoot.dataset.kgwBridgeV7Ready === "true") return;

  bridgeRoot.dataset.kgwBridgeV7Ready = "true";

  renderAllNetworks(bridgeRoot);
  kgwBridgeR51CaptureFactoryDefaults();
  kgwBridgeR51LoadSavedSettings();
  bridgeSyncAllModeControls();
  installNetworkTabs(bridgeRoot);
  installDelegatedTabs(bridgeRoot);
  installActions(bridgeRoot);
  bridgeSyncAllModeControls();
  updateAllCommands();
  window.setTimeout(updateAllCommands, 0);
  window.setTimeout(updateAllCommands, 150);
  bridgeSyncAllModeControls();
  updateAllCommands();
  installKgwBridgeR51BottomStyle();
  kgwBridgeR51StartLiveRefresh();


  setTimeout(kgwInstallBridgeLogAutoScrollControlsR27, 0);
}

export default initKaspaBridgeTab;

if (typeof window !== "undefined") {
  window.initKaspaBridgeTab = initKaspaBridgeTab;
}

(function installKgwLogFontSizeControlsV1() {
  "use strict";

  const KGW_MARKER = "KGW_LOG_FONT_SIZE_CONTROLS_V1";
  const KIND = "bridge";
  const MIN_SIZE = 10;
  const MAX_SIZE = 18;
  const DEFAULT_SIZE = 12;

  const NETWORKS = ["mainnet", "testnet10", "testnet12"];

  function storageKey(network) {
    return "kgw." + KIND + ".log.fontSize." + network;
  }

  function clampFontSize(value) {
    const parsed = Number.parseInt(String(value || ""), 10);
    if (!Number.isFinite(parsed)) return DEFAULT_SIZE;
    return Math.max(MIN_SIZE, Math.min(MAX_SIZE, parsed));
  }

  function readSize(network) {
    try {
      return clampFontSize(window.localStorage.getItem(storageKey(network)));
    } catch (_) {
      return DEFAULT_SIZE;
    }
  }

  function writeSize(network, size) {
    const finalSize = clampFontSize(size);
    try {
      window.localStorage.setItem(storageKey(network), String(finalSize));
    } catch (_) {}
    return finalSize;
  }

  function networkFromElement(element) {
    let current = element;
    while (current && current !== document.documentElement) {
      const candidates = [
        current.dataset && current.dataset.network,
        current.dataset && current.dataset.kgwNetwork,
        current.getAttribute && current.getAttribute("data-network"),
        current.getAttribute && current.getAttribute("data-kgw-network"),
        current.id,
        current.className
      ];

      const joined = candidates
        .filter(Boolean)
        .map((value) => String(value).toLowerCase())
        .join(" ");

      for (const network of NETWORKS) {
        if (joined.includes(network.toLowerCase())) return network;
      }

      current = current.parentElement;
    }

    return "mainnet";
  }

  function looksLikeLogPane(element) {
    if (!element || element.dataset.kgwLogFontSizePane === "1") return false;

    const text = [
      element.id || "",
      element.className || "",
      element.getAttribute && element.getAttribute("aria-label") || "",
      element.getAttribute && element.getAttribute("data-role") || ""
    ].join(" ").toLowerCase();

    if (!text.includes("log")) return false;

    const tag = String(element.tagName || "").toLowerCase();
    const allowedTag = tag === "pre" || tag === "code" || tag === "textarea" || tag === "div";
    if (!allowedTag) return false;

    const rect = element.getBoundingClientRect ? element.getBoundingClientRect() : null;
    if (rect && rect.height > 0 && rect.height < 40) return false;

    return true;
  }

  function findLogPanes(root) {
    const selector = [
      "pre",
      "code",
      "textarea",
      "[class*='log']",
      "[id*='log']",
      "[data-role*='log']",
      "[aria-label*='log']"
    ].join(",");

    return Array.from(root.querySelectorAll(selector)).filter(looksLikeLogPane);
  }

  function applySizeToPane(pane) {
    const network = networkFromElement(pane);
    const size = readSize(network);
    pane.dataset.kgwLogFontSizePane = "1";
    pane.dataset.kgwLogFontSizeNetwork = network;
    pane.style.setProperty("--kgw-log-font-size", size + "px");
    pane.style.fontSize = "var(--kgw-log-font-size)";
  }

  function applyAllSizes(root) {
    for (const pane of findLogPanes(root || document)) {
      applySizeToPane(pane);
    }
  }

  function findToolbarForPane(pane) {
    let current = pane.parentElement;
    while (current && current !== document.body) {
      const toolbar = current.querySelector(
        ".kgw-log-toolbar, .log-toolbar, [class*='log-toolbar'], [class*='toolbar'], [role='toolbar']"
      );
      if (toolbar) return toolbar;
      current = current.parentElement;
    }
    return null;
  }

  function createButton(label, title) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "kgw-log-font-size-button";
    button.textContent = label;
    button.title = title;
    button.setAttribute("aria-label", title);
    return button;
  }

  function updateControlsForNetwork(network) {
    const size = readSize(network);
    document
      .querySelectorAll(".kgw-log-font-size-controls[data-kind='" + KIND + "'][data-network='" + network + "']")
      .forEach((controls) => {
        const value = controls.querySelector(".kgw-log-font-size-value");
        if (value) value.textContent = size + "px";
      });

    document
      .querySelectorAll("[data-kgw-log-font-size-pane='1'][data-kgw-log-font-size-network='" + network + "']")
      .forEach((pane) => {
        pane.style.setProperty("--kgw-log-font-size", size + "px");
        pane.style.fontSize = "var(--kgw-log-font-size)";
      });
  }

  function installControlsForPane(pane) {
    const toolbar = findToolbarForPane(pane);
    if (!toolbar) return;

    const network = networkFromElement(pane);
    const existing = toolbar.querySelector(
      ".kgw-log-font-size-controls[data-kind='" + KIND + "'][data-network='" + network + "']"
    );
    if (existing) return;

    const controls = document.createElement("div");
    controls.className = "kgw-log-font-size-controls";
    controls.dataset.kind = KIND;
    controls.dataset.network = network;
    controls.dataset.marker = KGW_MARKER;

    const decrease = createButton("A-", "Decrease log font size");
    const value = document.createElement("span");
    value.className = "kgw-log-font-size-value";
    value.textContent = readSize(network) + "px";
    value.title = "Current log font size";

    const increase = createButton("A+", "Increase log font size");
    const reset = createButton("Reset", "Reset log font size");

    decrease.addEventListener("click", () => {
      const size = writeSize(network, readSize(network) - 1);
      updateControlsForNetwork(network);
    });

    increase.addEventListener("click", () => {
      const size = writeSize(network, readSize(network) + 1);
      updateControlsForNetwork(network);
    });

    reset.addEventListener("click", () => {
      const size = writeSize(network, DEFAULT_SIZE);
      updateControlsForNetwork(network);
    });

    controls.append(decrease, value, increase, reset);
    toolbar.appendChild(controls);
    updateControlsForNetwork(network);
  }

  function installAll(root) {
    applyAllSizes(root || document);
    document
      .querySelectorAll("[data-kgw-log-font-size-pane='1']")
      .forEach((pane) => installControlsForPane(pane));
  }

  function scheduleInstall(root) {
    window.requestAnimationFrame(() => installAll(root || document));
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => scheduleInstall(document), { once: true });
  } else {
    scheduleInstall(document);
  }

  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes || []) {
        if (node && node.nodeType === 1) {
          scheduleInstall(node);
        }
      }
    }
  });

  observer.observe(document.documentElement, {
    childList: true,
    subtree: true
  });

  window.addEventListener("storage", (event) => {
    if (!event.key || !event.key.startsWith("kgw." + KIND + ".log.fontSize.")) return;
    const network = event.key.split(".").pop();
    if (NETWORKS.includes(network)) updateControlsForNetwork(network);
  });
})();

