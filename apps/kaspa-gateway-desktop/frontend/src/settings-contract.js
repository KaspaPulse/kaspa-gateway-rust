// Shared validation for the existing node and bridge settings owners.
// These are effective configuration rules, not process or network simulation.
export const NODE_ENDPOINTS = [
  ["rpcListenEnabled", "rpcListenHost", "rpcListenPort", "rpcListen", true],
  ["rpcBorshEnabled", "rpcBorshHost", "rpcBorshPort", "rpcListenBorsh", true],
  ["rpcJsonEnabled", "rpcJsonHost", "rpcJsonPort", "rpcListenJson", true],
  ["listenEnabled", "listenHost", "listenPort", "p2pListen", false],
  ["externalIpEnabled", "externalIpHost", "externalIpPort", "externalIp", false],
  ["connectEnabled", "connectHost", "connectPort", "connectPeers", false],
  ["addPeerEnabled", "addPeerHost", "addPeerPort", "addPeers", false],
];
export const NODE_MANAGED = {
  appDir: "Managed by KaspaGateway; isolated for this network.",
  configFile: "Unsupported in managed mode: network and database ownership are managed.",
  overrideParamsFile: "Unsupported in managed mode: network parameters are managed.",
  testnet: "Managed by the selected network.",
  netsuffix: "Managed by the selected network.",
  noGrpc: "Unavailable in managed mode: gRPC is required for startup verification.",
  rpcListenEnabled: "Required in managed mode for startup verification.",
};
export const NODE_REQUIRED = {
  logLevel: "info", asyncThreads: "16", ramScale: "1",
  rpcMaxClients: "16", outPeers: "8", maxInPeers: "32",
};
export const NODE_OPTIONAL = new Set([
  "uaComment", "retentionDays", "maxTrackedAddresses", "perfMetricsInterval",
  "rocksDbPreset", "rocksDbCacheSize", "rocksDbWalDir", "logDir",
]);
export const NODE_DANGEROUS = {
  resetDb: "Deletes this network's database on every Start while enabled.",
  unsafeRpc: "Allows RPC outside loopback; use only on a trusted, protected network.",
  enableUnsyncedMining: "Allows mining before synchronization on a test network.",
};
export function isHost(value) {
  const host = String(value || "").trim().replace(/^\[|\]$/g, "");
  if (!host || host.length > 253 || /[\s/\\@?#]/.test(host)) return false;
  if (host.includes(":")) {
    try { return new URL("http://[" + host + "]/").hostname.startsWith("["); }
    catch { return false; }
  }
  if (/^[\d.]+$/.test(host)) {
    const parts = host.split(".");
    return parts.length === 4 && parts.every(p => /^\d{1,3}$/.test(p) && Number(p) <= 255);
  }
  return host.replace(/\.$/, "").split(".").every(part =>
    /^[a-z\d](?:[a-z\d-]{0,61}[a-z\d])?$/i.test(part));
}
export function isPort(value) {
  return /^\d+$/.test(String(value)) && Number(value) >= 1 && Number(value) <= 65535;
}
export function endpoint(host, port) {
  const plain = String(host).replace(/^\[|\]$/g, "");
  return (plain.includes(":") ? "[" + plain + "]" : plain) + ":" + port;
}
export function splitEndpoint(value) {
  const match = String(value || "").trim().match(/^(.*):(\d+)$/);
  if (!match) return null;
  return { host: match[1].replace(/^\[|\]$/g, ""), port: match[2] };
}
export function isLoopback(host) {
  const normalized = String(host).toLowerCase().replace(/^\[|\]$/g, "");
  return normalized === "localhost" || normalized === "::1" ||
    (/^127\./.test(normalized) && isHost(normalized));
}
export function listenersOverlap(a, b) {
  if (!a || !b || Number(a.port) !== Number(b.port)) return false;
  const norm = h => isLoopback(h) ? "loopback" : String(h).toLowerCase();
  return norm(a.host) === norm(b.host) ||
    [a.host, b.host].some(h => h === "0.0.0.0" || h === "::");
}
export function nodeFieldEnabled(name, values, options = {}) {
  if (Object.hasOwn(NODE_MANAGED, name)) return false;
  const dependency = NODE_ENDPOINTS.find(row => row[1] === name || row[2] === name);
  if (dependency) return values[dependency[0]] === true;
  if (name === "logDir" && values.noLogFiles) return false;
  if (name === "perfMetricsInterval" && !values.perfMetrics) return false;
  if (name === "rocksDbCacheSize" &&
      (options.rocksDbPreset === false || values.rocksDbPreset !== "hdd")) return false;
  if (Object.hasOwn(NODE_REQUIRED, name)) return true;
  return !NODE_OPTIONAL.has(name) || options[name] === true;
}
export function validateNodeForm(values, options, network) {
  const errors = {};
  const text = key => String(values[key] ?? "").trim();
  const active = key => nodeFieldEnabled(key, values, options);
  for (const [parent, hostKey, portKey, , rpc] of NODE_ENDPOINTS) {
    if (!values[parent]) continue;
    if (!isHost(text(hostKey))) errors[hostKey] = "Enter a valid IP address or hostname.";
    if (!isPort(text(portKey))) errors[portKey] = "Enter a whole port from 1 to 65535.";
    if (rpc && !values.unsafeRpc && isHost(text(hostKey)) && !isLoopback(text(hostKey)))
      errors[hostKey] = "RPC must use loopback unless unsafe RPC is explicitly enabled.";
  }
  if (!values.rpcListenEnabled) errors.rpcListenEnabled = "gRPC is required in managed mode.";
  if (values.noGrpc) errors.noGrpc = "gRPC cannot be disabled in managed mode.";
  if (values.connectEnabled && values.addPeerEnabled)
    errors.addPeerHost = "Use either connect-only peers or additional peers, not both.";
  const ranges = {
    asyncThreads: [1, Number.MAX_SAFE_INTEGER, true],
    ramScale: [0.1, 10, false], rpcMaxClients: [1, 16, true],
    outPeers: [0, 8, true], maxInPeers: [0, 32, true],
    maxTrackedAddresses: [0, Number.MAX_SAFE_INTEGER, true],
    retentionDays: [Number.MIN_VALUE, Number.MAX_SAFE_INTEGER, false],
    perfMetricsInterval: [1, Number.MAX_SAFE_INTEGER, true],
    rocksDbCacheSize: [1, Number.MAX_SAFE_INTEGER, true],
  };
  for (const [key, [min, max, integer]] of Object.entries(ranges)) {
    if (!active(key)) continue;
    const raw = text(key), number = Number(raw);
    if (!raw || !Number.isFinite(number) || number < min || number > max ||
        (integer && (!Number.isSafeInteger(number) || !/^\d+$/.test(raw))))
      errors[key] = "Enter " + (integer ? "a whole number" : "a number") +
        (max < Number.MAX_SAFE_INTEGER ? " from " + min + " to " + max : " greater than " + (min === 0 ? "or equal to 0" : "0")) + ".";
  }
  for (const key of NODE_OPTIONAL) {
    if (!active(key)) continue;
    if (!text(key)) errors[key] = "Enter a value or turn this option off.";
    else if (text(key).length > 4096 || /[\u0000-\u001f]/.test(text(key)))
      errors[key] = "Use at most 4096 characters without control characters.";
  }
  for (const key of ["rocksDbWalDir", "logDir"]) {
    if (active(key) && text(key) && !/^(?:[a-z]:[\\/]|\\\\[^\\]+\\[^\\]+|\/)/i.test(text(key)))
      errors[key] = "Enter a full directory path.";
  }
  if (network === "mainnet" && values.enableUnsyncedMining)
    errors.enableUnsyncedMining = "Available only for test networks.";
  const listeners = NODE_ENDPOINTS.slice(0, 4)
    .filter(([parent]) => values[parent])
    .map(([, host, port]) => ({ host: text(host), port: text(port), field: port }));
  for (let i = 0; i < listeners.length; i++)
    for (const other of listeners.slice(i + 1))
      if (listenersOverlap(listeners[i], other))
        errors[other.field] = "This listener conflicts with another enabled listener.";
  return errors;
}
export function renderFieldErrors(root, prefix, errors) {
  if (!root) return;
  root.querySelectorAll("[data-settings-error]").forEach(el => el.remove());
  root.querySelectorAll('[aria-invalid="true"]').forEach(el => {
    el.removeAttribute("aria-invalid");
    if (el.dataset.settingsErrorId) {
      const remaining = (el.getAttribute("aria-describedby") || "").split(" ")
        .filter(id => id !== el.dataset.settingsErrorId).join(" ");
      if (remaining) el.setAttribute("aria-describedby", remaining);
      else el.removeAttribute("aria-describedby");
      delete el.dataset.settingsErrorId;
    }
  });
  for (const [name, message] of Object.entries(errors)) {
    const field = document.getElementById(prefix + name);
    if (!field || !root.contains(field)) continue;
    const error = document.createElement("small");
    error.id = prefix + name + "-error";
    error.dataset.settingsError = name;
    error.className = "kgw-field-error";
    error.textContent = message;
    field.setAttribute("aria-invalid", "true");
    field.dataset.settingsErrorId = error.id;
    field.setAttribute("aria-describedby", [field.getAttribute("aria-describedby"), error.id].filter(Boolean).join(" "));
    field.closest(".node-v6-card, .bridge-v7-card")?.appendChild(error);
  }
}
export function runtimePresentation({role = "Node", enabled = true, running = false, transition = "", error = "", synced}) {
  const process = transition === "starting" ? "Starting" : transition === "stopping" ? "Stopping"
    : error ? "Failed" : running ? "Running" : "Stopped";
  return {
    application: "Ready", profile: enabled ? "Enabled" : "Disabled",
    process, processLabel: role + ": " + process,
    network: !running ? "Not connected" : synced === true ? "Synchronized"
      : synced === false ? "Not synchronized" : "Synchronization not reported",
  };
}

export const BRIDGE_MANAGED = {
  testnet: "Managed by the selected network.",
  appdir: "Managed by KaspaGateway; isolated for this network.",
  inprocessAppdirMirror: "Managed by KaspaGateway; same isolated node directory.",
  inprocessNetworkArgs: "Managed by the selected network.",
  inprocessConfigfile: "Unsupported in managed mode: network and database ownership are managed.",
  inprocessOverrideParamsFile: "Unsupported in managed mode: network parameters are managed.",
  inprocessDevnet: "Unsupported: choose a supported network tab.",
  inprocessSimnet: "Unsupported: choose a supported network tab.",
  healthCheckPort: "Unsupported by the embedded bridge: no health HTTP service is owned.",
  webDashboardPort: "Unsupported by the embedded bridge: no dashboard HTTP service is owned.",
  logToFile: "Unsupported in managed mode; real process logs remain available in Monitor.",
  approxGeoLookup: "Unsupported by the embedded bridge runtime.",
};
export const BRIDGE_REQUIRED = new Set([
  "nodeMode", "kaspadAddress", "stratumPort", "minShareDiff", "blockWaitTime",
  "printStats", "varDiff", "sharesPerMin", "varDiffStats", "extranonceSize",
  "pow2Clamp", "inprocessRpcListen", "inprocessLogLevel", "inprocessRamScale",
  "inprocessOutpeers", "inprocessMaxInpeers", "inprocessAsyncThreads",
  "internalCpuMinerAddress", "internalCpuMinerThreads",
]);
export const BRIDGE_OPTIONAL = new Set([
  "config", "coinbaseTagSuffix", "inprocessRpcListenBorsh", "inprocessRpcListenJson",
  "inprocessListen", "inprocessAddPeer", "inprocessConnect", "inprocessPerfMetricsIntervalSec",
]);
export function bridgeFieldEnabled(name, values, options = {}) {
  if (BRIDGE_MANAGED[name]) return false;
  if (["testnet10", "testnet13"].includes(values.network) &&
      !name.startsWith("inprocess") && !name.startsWith("internalCpuMiner") &&
      !["nodeMode", "kaspadAddress", "appdir", "testnet"].includes(name)) return false;
  const config = options.config === true && Boolean(String(values.config || "").trim());
  if (config && !["config", "nodeMode"].includes(name) && !name.startsWith("inprocess")) return false;
  if (name.startsWith("inprocess") && values.nodeMode !== "inprocess") return false;
  if (name === "kaspadAddress" && values.nodeMode !== "external") return false;
  if (name.startsWith("internalCpuMiner") && name !== "internalCpuMiner" && !values.internalCpuMiner) return false;
  if (name === "inprocessPerfMetricsIntervalSec" && !values.inprocessPerfMetrics) return false;
  if (BRIDGE_REQUIRED.has(name)) return true;
  if (BRIDGE_OPTIONAL.has(name)) return options[name] === true;
  return options[name] !== false;
}
export function validateBridgeForm(values, options, network) {
  values = { ...values, network };
  const errors = {}, text = key => String(values[key] ?? "").trim();
  const active = key => bridgeFieldEnabled(key, values, options);
  for (const key of BRIDGE_OPTIONAL) {
    if (active(key) && !text(key)) errors[key] = "Enter a value or turn this option off.";
  }
  if (active("config") && text("config")) {
    if (!/^(?:[a-z]:[\\/]|\\\\[^\\]+\\[^\\]+|\/)/i.test(text("config")))
      errors.config = "Enter the full path to an existing bridge configuration file.";
  }
  for (const key of ["kaspadAddress", "inprocessRpcListen", "inprocessRpcListenBorsh",
    "inprocessRpcListenJson", "inprocessListen", "inprocessAddPeer", "inprocessConnect"]) {
    if (!active(key)) continue;
    const ep = splitEndpoint(text(key));
    if (!ep || !isHost(ep.host) || !isPort(ep.port)) errors[key] = "Enter a valid host:port (port 1 to 65535).";
    else if (key.startsWith("inprocessRpc") && !values.inprocessUnsafeRpc && !isLoopback(ep.host))
      errors[key] = "RPC must use loopback unless unsafe RPC is explicitly enabled.";
  }
  for (const key of ["stratumPort", "promPort"]) {
    if (!active(key)) continue;
    const raw = text(key), ep = splitEndpoint(raw.includes(":") ? raw : ":" + raw);
    if (!ep || !isPort(ep.port) || (ep.host && !isHost(ep.host)))
      errors[key] = "Enter a listener port from 1 to 65535.";
  }
  const ranges = {
    minShareDiff: [1, 4294967295], sharesPerMin: [1, 4294967295],
    extranonceSize: [0, 8], inprocessOutpeers: [0, 8], inprocessMaxInpeers: [0, 32],
    inprocessPerfMetricsIntervalSec: [1, Number.MAX_SAFE_INTEGER],
    inprocessAsyncThreads: [1, 65535],
    internalCpuMinerThreads: [1, 256],
    internalCpuMinerThrottleMs: [0, 60000],
    internalCpuMinerTemplatePollMs: [1, 60000],
  };
  for (const [key, [min, max]] of Object.entries(ranges)) {
    if (!active(key)) continue;
    if (key.startsWith("internalCpuMiner") && !text(key) && key !== "internalCpuMinerThreads") continue;
    const n = Number(text(key));
    if (!/^\d+$/.test(text(key)) || !Number.isSafeInteger(n) || n < min || n > max)
      errors[key] = "Enter a whole number from " + min + " to " + max + ".";
  }
  if (active("inprocessRamScale") && (!text("inprocessRamScale") ||
      !Number.isFinite(Number(text("inprocessRamScale"))) ||
      Number(text("inprocessRamScale")) < 0.1 || Number(text("inprocessRamScale")) > 10))
    errors.inprocessRamScale = "Enter a number from 0.1 to 10.";
  if (active("blockWaitTime") && !/^[1-9]\d*(ms|s)?$/.test(text("blockWaitTime")))
    errors.blockWaitTime = "Enter a positive duration, for example 50ms or 1s.";
  if (active("inprocessConnect") && active("inprocessAddPeer"))
    errors.inprocessAddPeer = "Use either connect-only peers or additional peers, not both.";
  if (values.internalCpuMiner && !text("internalCpuMinerAddress"))
    errors.internalCpuMinerAddress = "Enter a test-network mining address.";
  if (network === "mainnet" && values.internalCpuMiner)
    errors.internalCpuMiner = "CPU mining is available only for test networks.";
  if (network === "mainnet" && values.inprocessEnableUnsyncedMining)
    errors.inprocessEnableUnsyncedMining = "Available only for test networks.";
  return errors;
}

export function runtimeObservationSummary(fields = {}, running = false, cpuOnly = false) {
  const fresh = running && fields.observation_state === "fresh";
  const rpc = !fresh ? "Unknown" : fields.rpc_ready === "true" ? "Available" : "Unavailable";
  const sync = !fresh || fields.synced === "unknown" ? "Not reported"
    : fields.synced === "true" ? "Synchronized" : fields.synced === "false" ? "Not synchronized" : "Not reported";
  let text = "RPC: " + rpc + " | Sync: " + sync;
  if (fresh && /^\d+$/.test(fields.virtual_daa_score || "")) text += " | DAA: " + fields.virtual_daa_score;
  if (cpuOnly) {
    const enabled = fresh && fields.cpu_enabled === "true";
    const rate = enabled && fields.cpu_hashrate_hs !== "unknown" ? Number(fields.cpu_hashrate_hs) : NaN;
    const activity = !fresh ? "Not reported" : !enabled ? "Disabled"
      : Number.isFinite(rate) && rate > 0 ? "Hashing"
      : fields.cpu_hashes_tried === "0" ? "Waiting for work" : "No recent hashing reported";
    text += " | CPU: " + activity;
    if (enabled) {
      for (const [field, label] of [["cpu_hashes_tried","Hashes"],["cpu_blocks_submitted","Submitted blocks"],["cpu_blocks_confirmed_blue","Confirmed blue blocks"]])
        if (/^\d+$/.test(fields[field] || "")) text += " | " + label + ": " + fields[field];
      if (Number.isFinite(rate) && rate >= 0) text += " | " + rate.toFixed(2) + " H/s";
    }
  }
  if (fresh && fields.observation_error && fields.observation_error !== "none")
    text += " | RPC error: " + fields.observation_error;
  return text;
}

export async function confirmUserAction(message) {
  const invoke = window.__TAURI__?.core?.invoke;
  if (typeof invoke === "function") {
    const answer = await invoke("plugin:dialog|message", {
      title: "KaspaGateway", message, kind: "warning", buttons: "OkCancel",
    });
    return answer === "Ok";
  }
  return (await window.confirm(message)) === true;
}
