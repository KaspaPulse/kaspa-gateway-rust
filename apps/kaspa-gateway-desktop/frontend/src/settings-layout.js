// Presentation-only Settings V2 owner. Never reads/writes setting values or calls IPC.
const escapeHtml = value => String(value).replace(/[&<>"']/g, c =>
  ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

export const SETTING_HELP_I18N_KEYS = Object.freeze({
  appDir: "common.tooltip.appdir",
  appdir: "common.tooltip.appdir",
  inprocessAppdirMirror: "common.tooltip.appdir",
  archival: "common.tooltip.archival",
  inprocessArchival: "common.tooltip.archival",
  asyncThreads: "common.tooltip.async.threads",
  inprocessAsyncThreads: "common.tooltip.async.threads",
  configFile: "common.tooltip.configfile",
  config: "common.tooltip.configfile",
  inprocessConfigfile: "common.tooltip.configfile",
  connectEnabled: "common.tooltip.connect",
  connectHost: "common.tooltip.connect",
  connectPort: "common.tooltip.connect",
  inprocessConnect: "common.tooltip.connect",
  disableUpnp: "common.tooltip.disable.upnp",
  inprocessDisableUpnp: "common.tooltip.disable.upnp",
  enableUnsyncedMining: "common.tooltip.enable.unsynced.mining",
  inprocessEnableUnsyncedMining: "common.tooltip.enable.unsynced.mining",
  externalIpEnabled: "common.tooltip.externalip",
  externalIpHost: "common.tooltip.externalip",
  externalIpPort: "common.tooltip.externalip",
  healthCheckPort: "common.tooltip.hcp",
  listenEnabled: "common.tooltip.listen",
  listenHost: "common.tooltip.listen",
  listenPort: "common.tooltip.listen",
  inprocessListen: "common.tooltip.listen",
  maxInPeers: "common.tooltip.maxinpeers",
  inprocessMaxInpeers: "common.tooltip.maxinpeers",
  netsuffix: "common.tooltip.netsuffix",
  noDnsSeed: "common.tooltip.nodnsseed",
  outPeers: "common.tooltip.outpeers",
  inprocessOutpeers: "common.tooltip.outpeers",
  ramScale: "common.tooltip.ram.scale",
  inprocessRamScale: "common.tooltip.ram.scale",
  resetDb: "common.tooltip.reset.db",
  retentionDays: "common.tooltip.retention.period.days",
  sanity: "common.tooltip.sanity",
  uaComment: "common.tooltip.uacomment",
  yes: "common.tooltip.yes",
  inprocessYes: "common.tooltip.yes",
  rpcListenEnabled: "node.tooltip.rpclisten",
  rpcListenHost: "node.tooltip.rpclisten",
  rpcListenPort: "node.tooltip.rpclisten",
  inprocessRpcListen: "node.tooltip.rpclisten",
  rpcMaxClients: "node.tooltip.rpcmaxclients",
  unsafeRpc: "node.tooltip.unsaferpc",
  inprocessUnsafeRpc: "node.tooltip.unsaferpc",
  utxoIndex: "node.tooltip.utxoindex",
  inprocessUtxoIndex: "node.tooltip.utxoindex",
  noGrpc: "node.tooltip.nogrpc",
  stratumPort: "common.tooltip.ks.stratum",
  promPort: "common.tooltip.ks.prom",
  minShareDiff: "common.tooltip.ks.mindiff",
  sharesPerMin: "common.tooltip.ks.sharespermin",
  varDiff: "common.tooltip.ks.vardiff",
  varDiffStats: "common.tooltip.ks.vardiffstats",
  pow2Clamp: "common.tooltip.ks.pow2clamp",
  extranonceSize: "common.tooltip.ks.extranonce",
  printStats: "common.tooltip.ks.stats",
  instanceDiff: "common.tooltip.ks.mindiff",
  instanceSharesPerMin: "common.tooltip.ks.sharespermin",
  instanceVarDiff: "common.tooltip.ks.vardiff",
  instanceVarDiffStats: "common.tooltip.ks.vardiffstats",
  instancePow2Clamp: "common.tooltip.ks.pow2clamp",
  settingsRetryAttempts: "common.tooltip.retry.attempts",
  settingsBackoffFactor: "common.tooltip.backoff.factor",
  settingsMaxWorkers: "common.tooltip.max.workers",
  settingsMaxPages: "common.tooltip.max.pages",
  settingsPageDelay: "common.tooltip.page.delay",
  settingsNetworkCacheHours: "common.tooltip.network.cache.hours"
});

export const GLOBAL_SETTING_HELP_IDS = Object.freeze([
  "settingsCheckUpdates", "settingsStartWindows", "settingsLoggingLevel",
  "settingsDatabasePath", "settingsExportPath", "settingsLogPath", "settingsBackupPath",
  "settingsApiProfile", "settingsApiKey", "settingsApiDescription", "settingsApiBase", "settingsApiPath",
  "settingsApiTimeout", "settingsRetryAttempts", "settingsBackoffFactor", "settingsMaxWorkers",
  "settingsMaxPages", "settingsPageDelay", "settingsPriceCacheHours", "settingsNetworkCacheHours",
  "settingsEnableAutoRefresh", "settingsRefreshInterval", "settingsAddressName", "settingsAddressValue"
]);

const HELP_FALLBACKS = Object.freeze({
  nodeMode: "Chooses whether the Bridge uses an independent Node or owns an embedded in-process Node. Change it only when the ownership model is intentional.",
  blockWaitTime: "Controls how long the Bridge waits for block-related work before retrying. Keep the default unless the runtime environment requires a deliberate adjustment.",
  logLevel: "Controls runtime log verbosity. Higher verbosity is useful for diagnosis but produces more output.",
  inprocessLogLevel: "Controls the embedded Node log verbosity. Use higher verbosity only when extra diagnostics are needed.",
  noLogFiles: "Disables file logging while keeping the live application log available.",
  perfMetrics: "Enables periodic performance metrics collection for the Node.",
  inprocessPerfMetrics: "Enables periodic performance metrics for the embedded Node.",
  perfMetricsInterval: "Controls how often performance metrics are sampled.",
  inprocessPerfMetricsIntervalSec: "Controls how often embedded Node performance metrics are sampled.",
  rocksDbPreset: "Selects the RocksDB tuning preset. Use a non-default preset only when it matches the storage hardware.",
  rocksDbCacheSize: "Sets the RocksDB cache budget. Larger values trade memory for database cache capacity.",
  rocksDbWalDir: "Overrides the RocksDB write-ahead-log directory. Use an absolute path on storage you intentionally manage.",
  maxTrackedAddresses: "Limits how many addresses may be tracked for UTXO-change notifications.",
  overrideParamsFile: "Provides advanced network-parameter overrides. Managed networks may reject this setting; use only for controlled testing.",
  inprocessOverrideParamsFile: "Provides advanced network-parameter overrides for the embedded Node. Use only for controlled testing.",
  inprocessDevnet: "Requests devnet behavior. KaspaGateway managed network tabs do not support this mode.",
  inprocessSimnet: "Requests simnet behavior. KaspaGateway managed network tabs do not support this mode.",
  rpcBorshEnabled: "Enables the Borsh RPC listener when a separate Borsh endpoint is intentionally required.",
  rpcBorshHost: "Host/interface for the optional Borsh RPC listener.",
  rpcBorshPort: "Port for the optional Borsh RPC listener.",
  rpcJsonEnabled: "Enables the JSON RPC listener when a separate JSON endpoint is intentionally required.",
  rpcJsonHost: "Host/interface for the optional JSON RPC listener.",
  rpcJsonPort: "Port for the optional JSON RPC listener.",
  addPeerEnabled: "Enables an explicit peer to add while retaining normal peer discovery.",
  addPeerHost: "Host of the explicit peer to add.",
  addPeerPort: "P2P port of the explicit peer to add.",
  inprocessAddPeer: "Adds an explicit peer to the embedded Node while retaining its normal peer policy.",
  internalCpuMiner: "Enables the embedded CPU-only miner for supported test networks. It does not enable an ASIC listener.",
  internalCpuMinerAddress: "Reward address used by the embedded CPU miner. Use an address for the selected test network.",
  internalCpuMinerThreads: "Number of CPU threads assigned to the embedded testnet miner.",
  internalCpuMinerThrottleMs: "Delay between CPU-mining work loops. Increase it to reduce CPU pressure.",
  internalCpuMinerTemplatePollMs: "How often the CPU miner requests a fresh mining template.",
  kaspadAddress: "RPC endpoint used by the Bridge when connecting to an independent Node.",
  coinbaseTagSuffix: "Optional suffix added to the mining coinbase tag.",
  logToFile: "Controls file logging for this runtime. Managed mode may keep this disabled while live logs remain available.",
  approxGeoLookup: "Controls approximate geographic lookup when the embedded runtime supports it.",
  instance: "Enables this Bridge instance and its instance-specific overrides.",
  instancePort: "Stratum port override for this Bridge instance.",
  instanceProm: "Prometheus port override for this Bridge instance.",
  instanceLogToFile: "Instance file logging is managed by KaspaGateway and may be unavailable.",
  settingsCheckUpdates: "Controls whether KaspaGateway checks for available updates at startup.",
  settingsStartWindows: "Controls whether the installed KaspaGateway application starts with Windows.",
  settingsLoggingLevel: "Controls application log verbosity. Higher levels provide more diagnostics and more output.",
  settingsDatabasePath: "Directory used for application database files. Leave it empty to use the managed application-data location.",
  settingsExportPath: "Default directory offered when exporting data.",
  settingsLogPath: "Directory used for application log files when file logging is enabled.",
  settingsBackupPath: "Directory used for database backups and restore points.",
  settingsApiProfile: "Selects the saved API endpoint profile used by the application.",
  settingsApiKey: "Identifier of the selected API endpoint entry.",
  settingsApiDescription: "Human-readable description of the selected API endpoint.",
  settingsApiBase: "Base URL used to compose the selected API endpoint.",
  settingsApiPath: "Path appended to the endpoint base URL.",
  settingsApiTimeout: "Maximum time to wait for an API request before it is treated as timed out.",
  settingsPriceCacheHours: "Hours to keep price data before requesting a fresh value.",
  settingsEnableAutoRefresh: "Enables periodic refresh for supported application data.",
  settingsRefreshInterval: "Seconds between automatic refresh attempts.",
  settingsAddressName: "Friendly local name stored with a saved Kaspa address.",
  settingsAddressValue: "Kaspa address stored in the local managed address list."
});

function translateHelp(key, fallback) {
  if (!key) return fallback;
  try {
    const apis = [window?.kgwI18n, window?.KGWI18n, window?.KGW_I18N, window?.i18n];
    for (const api of apis) {
      if (!api) continue;
      for (const method of ["t", "translate", "get"]) {
        if (typeof api[method] !== "function") continue;
        const value = api[method](key, fallback);
        if (typeof value === "string" && value.trim() && value !== key) return value.trim();
      }
    }
  } catch (_) { /* Fall through to a source-controlled fallback. */ }
  return fallback;
}

function copyAttributes(from, to) {
  for (const attr of [...from.attributes]) to.setAttribute(attr.name, attr.value);
}

function normalizeSettingName(field) {
  if (!field) return "";
  if (field.dataset.bridgeInstanceField) return field.dataset.bridgeInstanceField;
  return String(field.id || "")
    .replace(/^(?:node|bridge)-(?:mainnet|testnet10|testnet13)-/, "")
    .replace(/-\d+$/, "");
}

export function settingFieldKind(field) {
  const name = normalizeSettingName(field);
  if (field?.type === "checkbox") return "boolean";
  if (field?.matches?.("[data-bridge-instance-preview]")) return "preview";
  if (field?.tagName === "SELECT") return /log/i.test(name) ? "log-level" : "enum";
  if (/port$|instanceProm$/i.test(name)) return "port";
  if (/ramScale$/i.test(name)) return "ram";
  if (/threads$|peers$|netsuffix$|extranonceSize$|maxTrackedAddresses$/i.test(name)) return "integer";
  if (/host$|kaspadAddress$|RpcListen(Borsh|Json)?$/i.test(name)) return "host";
  if (/Time$|Ms$|Interval(Sec)?$|Days$|Hours$/i.test(name)) return "duration";
  if (/dir|file|path|config$/i.test(name)) return "path";
  if (/Address$|Comment$|Suffix$|NetworkArgs$|AddPeer$|Connect$/i.test(name)) return "long";
  return "number";
}

function recommendedSpan(kind) {
  if (kind === "preview") return "full";
  if (kind === "path") return "3";
  if (kind === "long") return "2";
  return "1";
}

function genericHelp(name, label, kind) {
  const clean = String(label || name || "This setting").replace(/\s+/g, " ").trim();
  const subject = clean || "This setting";
  if (kind === "boolean") return subject + " toggles this behavior. Change it only when the feature is intentionally required.";
  if (kind === "port") return subject + " sets a network port. Change it only to resolve a port conflict or to use an intentionally custom endpoint.";
  if (kind === "host") return subject + " sets the host or interface used by this endpoint. Keep loopback/private binding unless broader access is intentionally required.";
  if (kind === "path") return subject + " selects a filesystem location. Use an absolute path that the current user can access.";
  if (kind === "duration") return subject + " controls a timing interval. Keep the default unless a measured workload requires a deliberate change.";
  if (kind === "integer" || kind === "number" || kind === "ram") return subject + " controls a numeric runtime limit or tuning value. Change it only when the workload requires a deliberate override.";
  if (kind === "enum" || kind === "log-level") return subject + " selects one of the supported runtime modes. Use the default unless another mode is intentionally required.";
  return subject + " configures this runtime option. Change it only when a non-default value is intentionally required.";
}

function helpDescriptor(field, labelText) {
  const name = normalizeSettingName(field);
  const kind = settingFieldKind(field);
  const key = SETTING_HELP_I18N_KEYS[name] || SETTING_HELP_I18N_KEYS[field?.id] || "";
  const fallback = HELP_FALLBACKS[name] || HELP_FALLBACKS[field?.id] || genericHelp(name, labelText, kind);
  return { name, kind, key, text: translateHelp(key, fallback) || fallback };
}

function convertToLabel(node, field) {
  if (!node || !field?.id) return node;
  if (node.tagName === "LABEL") {
    if (!node.htmlFor) node.htmlFor = field.id;
    return node;
  }
  const label = document.createElement("label");
  copyAttributes(node, label);
  label.htmlFor = field.id;
  label.innerHTML = node.innerHTML;
  node.replaceWith(label);
  return label;
}

function replaceCardLabelElement(card) {
  if (card.tagName !== "LABEL") return card;
  const replacement = document.createElement("div");
  copyAttributes(card, replacement);
  while (card.firstChild) replacement.appendChild(card.firstChild);
  card.replaceWith(replacement);
  return replacement;
}

function ensureAtomicStructure(initialCard, field) {
  let card = replaceCardLabelElement(initialCard);
  card.classList.add("kgw-setting-field");
  card.dataset.settingKind = settingFieldKind(field);
  card.dataset.settingSpan = recommendedSpan(card.dataset.settingKind);
  card.dataset.settingLayout = field.type === "checkbox" ? "check" :
    ["path", "long", "preview"].includes(card.dataset.settingKind) ? "wide" : "compact";

  if (card.dataset.kgwAtomicSetting !== "v2") {
    const titleRow = card.querySelector(":scope > .kgw-command-option-title-row-r8e");
    let label = titleRow?.querySelector(".kgw-command-option-title-text-r8e") || null;
    if (!label) {
      label = [...card.children].find(node =>
        node !== field && node.tagName === "SPAN" && !node.classList.contains("kgw-setting-state")) || null;
    }
    label = convertToLabel(label, field);

    const header = document.createElement("div");
    header.className = "kgw-setting-field-header";
    if (titleRow) header.appendChild(titleRow);
    else if (label) header.appendChild(label);

    const control = document.createElement("div");
    control.className = "kgw-setting-control";
    control.appendChild(field);

    const state = document.createElement("div");
    state.className = "kgw-setting-state";
    state.setAttribute("aria-live", "polite");
    state.textContent = card.dataset.kgwSettingState || "";

    card.prepend(header);
    header.insertAdjacentElement("afterend", control);
    control.insertAdjacentElement("afterend", state);
    card.dataset.kgwAtomicSetting = "v2";
  }

  const header = card.querySelector(":scope > .kgw-setting-field-header");
  const label = header?.querySelector("label[for]") || header?.querySelector(".kgw-command-option-title-text-r8e");
  const state = card.querySelector(":scope > .kgw-setting-state");
  if (state && state.textContent !== (card.dataset.kgwSettingState || "")) state.textContent = card.dataset.kgwSettingState || "";
  return { card, header, label, state };
}

function ensureHelp(card, header, label, field) {
  if (!header || !field?.id) return;
  const labelText = String(label?.textContent || field.id).replace(/\s+/g, " ").trim();
  const descriptor = helpDescriptor(field, labelText);
  const helpId = field.id + "-help";

  let trigger = header.querySelector(":scope > .kgw-field-help-trigger");
  if (!trigger) {
    trigger = document.createElement("button");
    trigger.type = "button";
    trigger.className = "kgw-field-help-trigger";
    trigger.textContent = "?";
    trigger.setAttribute("aria-expanded", "false");
    header.appendChild(trigger);
  }
  trigger.setAttribute("aria-controls", helpId);
  trigger.setAttribute("aria-label", "Help: " + labelText);
  trigger.dataset.helpKey = descriptor.key;

  let popover = card.querySelector(":scope > .kgw-field-help-popover");
  if (!popover) {
    popover = document.createElement("div");
    popover.className = "kgw-field-help-popover";
    popover.hidden = true;
    popover.setAttribute("role", "tooltip");
    card.appendChild(popover);
  }
  popover.id = helpId;
  popover.dataset.helpKey = descriptor.key;
  popover.textContent = descriptor.text;

  const described = new Set((field.getAttribute("aria-describedby") || "").split(/\s+/).filter(Boolean));
  described.add(helpId);
  field.setAttribute("aria-describedby", [...described].join(" "));

  if (["port", "host", "integer", "ram", "duration", "path", "long", "number", "preview"].includes(descriptor.kind)) {
    field.dir = "ltr";
  }
}

function closeHelp(root, except = null) {
  root?.querySelectorAll(".kgw-field-help-trigger[aria-expanded=\"true\"]").forEach(button => {
    if (button === except) return;
    button.setAttribute("aria-expanded", "false");
    const panel = root.querySelector("#" + CSS.escape(button.getAttribute("aria-controls") || ""));
    if (panel) panel.hidden = true;
  });
}

function openHelp(root, button) {
  closeHelp(root, button);
  button.setAttribute("aria-expanded", "true");
  const panel = root.querySelector("#" + CSS.escape(button.getAttribute("aria-controls") || ""));
  if (panel) panel.hidden = false;
}

function installHelpBehavior(root) {
  if (!root || root.dataset.kgwSettingsHelpInstalled === "v2") return;
  root.dataset.kgwSettingsHelpInstalled = "v2";
  root.addEventListener("click", event => {
    const button = event.target.closest?.(".kgw-field-help-trigger");
    if (!button || !root.contains(button)) {
      closeHelp(root);
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    openHelp(root, button);
  });
  root.addEventListener("focusin", event => {
    const button = event.target.closest?.(".kgw-field-help-trigger");
    if (button && root.contains(button)) openHelp(root, button);
  });
  root.addEventListener("keydown", event => {
    const trigger = event.target.closest?.(".kgw-field-help-trigger");
    if (trigger && root.contains(trigger) && (event.key === "Enter" || event.key === " ")) {
      event.preventDefault();
      event.stopPropagation();
      openHelp(root, trigger);
      return;
    }
    if (event.key !== "Escape") return;
    const button = root.querySelector(".kgw-field-help-trigger[aria-expanded=\"true\"]");
    if (!button) return;
    event.preventDefault();
    closeHelp(root);
    button.focus();
  });
}

export function renderSettingsTabs(scope, net, groups) {
  const prefix = scope === "node" ? "node-v6" : "bridge-v7";
  const sections = ["general", "advanced"];
  const tabs = sections.map((section, index) =>
    '<button type="button" role="tab" class="' + prefix + '-section-tab' + (index ? "" : " active") +
    '" data-net="' + net + '" data-' + scope + '-section-tab="' + section +
    '" aria-selected="' + !index + '">' + (index ? "Advanced" : "General") + "</button>").join("");
  const panels = sections.map((section, index) => {
    const entries = groups.filter(group => group[0] === section);
    const key = scope + "-" + net + "-" + section;
    const nav = entries.map(([, name, label], i) =>
      '<button type="button" role="tab" id="' + key + "-" + name + '-tab" data-settings-tab="' + name +
      '" aria-controls="' + key + "-" + name + '-panel" aria-selected="' + !i + '" tabindex="' +
      (i ? "-1" : "0") + '">' + escapeHtml(label) + "</button>").join("");
    const body = entries.map(([, name, label, html], i) =>
      '<section role="tabpanel" id="' + key + "-" + name + '-panel" aria-labelledby="' + key + "-" + name +
      '-tab" data-settings-panel="' + name + '"' + (i ? " hidden" : "") + ">" + html + "</section>").join("");
    return '<section class="' + prefix + '-section' + (index ? "" : " active") + '" data-net="' + net +
      '" data-' + scope + '-section-panel="' + section + '"' + (index ? " hidden" : "") +
      '><div data-settings-tab-group><div class="kgw-settings-subtabs" role="tablist" aria-label="' +
      escapeHtml(section + " settings") + '">' + nav + "</div>" + body + "</div></section>";
  }).join("");
  return '<div class="' + prefix + '-section-tabs" role="tablist" aria-label="Settings level">' + tabs +
    '</div><div class="' + prefix + '-sections kgw-settings-sections">' + panels + "</div>";
}

export function decorateSettingsFields(root) {
  root?.querySelectorAll(".node-v6-card, .bridge-v7-card").forEach(initialCard => {
    const field = initialCard.querySelector("input[id], select[id], textarea[id], input[data-bridge-instance-preview]");
    if (!field) return;
    const { card, header, label } = ensureAtomicStructure(initialCard, field);
    if (field.matches("[data-bridge-instance-preview][readonly]")) {
      card.dataset.settingExempt = "generated-readonly-preview";
      field.dir = "ltr";
      return;
    }
    ensureHelp(card, header, label, field);
    const cleanLabel = header?.querySelector(".kgw-command-option-title-text-r8e");
    if (cleanLabel && !cleanLabel.hasAttribute("data-i18n")) {
      if (!cleanLabel.title) cleanLabel.title = cleanLabel.textContent;
      cleanLabel.textContent = cleanLabel.textContent.replace(/ \((?:managed|unsupported)[^)]*\)/g, "");
    }
  });
  installHelpBehavior(root);
}

export function setSettingFieldState(field, message = "") {
  const card = field?.closest(".kgw-setting-field, .node-v6-card, .bridge-v7-card");
  if (!card) return;
  card.dataset.kgwSettingState = String(message || "");
  const state = card.querySelector(":scope > .kgw-setting-state");
  if (state) state.textContent = card.dataset.kgwSettingState;
}

function selectInternalTab(button) {
  const group = button?.closest("[data-settings-tab-group]");
  if (!group) return;
  group.querySelectorAll(":scope > .kgw-settings-subtabs > [data-settings-tab]").forEach(tab => {
    tab.setAttribute("aria-selected", String(tab === button));
    tab.tabIndex = tab === button ? 0 : -1;
  });
  group.querySelectorAll(":scope > [data-settings-panel]").forEach(panel => {
    panel.hidden = panel.dataset.settingsPanel !== button.dataset.settingsTab;
  });
}

export function revealSettingsField(field) {
  const internal = field?.closest("[data-settings-panel]");
  const group = internal?.parentElement;
  if (internal && group) selectInternalTab(group.querySelector('[data-settings-tab="' + internal.dataset.settingsPanel + '"]'));
  const nested = field?.closest("[data-bridge-inprocess-node-panel]");
  if (nested) nested.closest("[data-bridge-inprocess-node-settings]")
    ?.querySelector('[data-bridge-inprocess-node-tab="' + nested.dataset.bridgeInprocessNodePanel + '"]')?.click();
}

export function installSettingsLayout(root) {
  decorateSettingsFields(root);
  if (root.dataset.settingsLayoutInstalled) return;
  root.dataset.settingsLayoutInstalled = "true";
  root.addEventListener("click", event => {
    const tab = event.target.closest("[data-settings-tab]");
    if (tab && root.contains(tab)) selectInternalTab(tab);
    const button = event.target.closest("[data-settings-preview-toggle]");
    if (button) {
      const preview = button.closest(".kgw-effective-preview");
      const body = preview.querySelector(".kgw-preview-body");
      body.hidden = !body.hidden;
      button.setAttribute("aria-expanded", String(!body.hidden));
      button.textContent = body.hidden ? "Expand" : "Collapse";
    }
  });
  root.addEventListener("keydown", event => {
    const tab = event.target.closest("[data-settings-tab]");
    if (!tab || !["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    const tabs = [...tab.parentElement.querySelectorAll("[data-settings-tab]")];
    const index = tabs.indexOf(tab);
    const next = event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 :
      (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
    event.preventDefault();
    selectInternalTab(tabs[next]);
    tabs[next].focus();
  });
  window.addEventListener("kgw:language-applied", () => decorateSettingsFields(root));
}

function globalHelpDescriptor(field, labelText) {
  const key = SETTING_HELP_I18N_KEYS[field.id] || "";
  const fallback = HELP_FALLBACKS[field.id] || genericHelp(field.id, labelText, settingFieldKind(field));
  return { key, text: translateHelp(key, fallback) || fallback };
}

function decorateGlobalField(root, field, label) {
  if (!field || !label || field.readOnly || !GLOBAL_SETTING_HELP_IDS.includes(field.id)) return;
  const labelText = label.tagName === "LEGEND"
    ? [...label.childNodes].filter(node => node.nodeType === Node.TEXT_NODE).map(node => node.textContent).join("").trim()
    : String(label.textContent || "").trim();
  let wrapper;
  if (label.tagName === "LEGEND") {
    wrapper = label;
    wrapper.classList.add("kgw-global-field-header");
    if (!wrapper.id) wrapper.id = field.id + "-label";
    if (!field.getAttribute("aria-labelledby")) field.setAttribute("aria-labelledby", wrapper.id);
  } else {
    wrapper = label.parentElement?.classList.contains("kgw-global-field-header") ? label.parentElement : null;
    if (!wrapper) {
      wrapper = document.createElement("span");
      wrapper.className = "kgw-global-field-header";
      label.replaceWith(wrapper);
      wrapper.appendChild(label);
    }
  }
  const descriptor = globalHelpDescriptor(field, labelText);
  const helpId = field.id + "-help";
  let trigger = wrapper.querySelector(":scope > .kgw-field-help-trigger");
  if (!trigger) {
    trigger = document.createElement("button");
    trigger.type = "button";
    trigger.className = "kgw-field-help-trigger";
    trigger.textContent = "?";
    trigger.setAttribute("aria-expanded", "false");
    wrapper.appendChild(trigger);
  }
  trigger.setAttribute("aria-controls", helpId);
  trigger.setAttribute("aria-label", "Help: " + (labelText || field.id));
  trigger.dataset.helpKey = descriptor.key;
  let popover = wrapper.querySelector(":scope > .kgw-field-help-popover");
  if (!popover) {
    popover = document.createElement("span");
    popover.className = "kgw-field-help-popover";
    popover.setAttribute("role", "tooltip");
    popover.hidden = true;
    wrapper.appendChild(popover);
  }
  popover.id = helpId;
  popover.dataset.helpKey = descriptor.key;
  popover.textContent = descriptor.text;
  const described = new Set((field.getAttribute("aria-describedby") || "").split(/\s+/).filter(Boolean));
  described.add(helpId);
  field.setAttribute("aria-describedby", [...described].join(" "));
}

export function decorateGlobalSettings(root) {
  if (!root) return;
  for (const id of GLOBAL_SETTING_HELP_IDS) {
    const field = root.querySelector("#" + CSS.escape(id));
    if (!field || field.readOnly) continue;
    let label = root.querySelector('label[for="' + CSS.escape(id) + '"]');
    if (!label) {
      const containing = field.closest("label");
      if (containing) label = containing;
    }
    if (!label) label = field.closest("fieldset")?.querySelector(":scope > legend") || null;
    decorateGlobalField(root, field, label);
  }
  root.querySelectorAll(".info-dot").forEach(dot => dot.remove());
  installHelpBehavior(root);
}

export function installGlobalSettingsLayout(root) {
  decorateGlobalSettings(root);
  if (!root || root.dataset.kgwGlobalSettingsLayoutInstalled === "v2") return;
  root.dataset.kgwGlobalSettingsLayoutInstalled = "v2";
  window.addEventListener("kgw:language-applied", () => decorateGlobalSettings(root));
}
