/*
  KGW_ANALYSIS_ADDRESS_SELECTOR_OWNER_V1
  Rust/Tauri Analysis binding.
  UI adapter only.
  Address source: get_all_addresses + manual fallback.
  Analysis owner: Tauri command analysis_report.
*/

const ANALYSIS_STATE = {
  currentAddress: "",
  lastReport: null,
  running: false,
  addressesLoaded: false
};

export function installRustAnalysisBinding() {
  if (window.__kgwRustAnalysisBindingV2) return;
  window.__kgwRustAnalysisBindingV2 = true;

  window.kgwRunRustAnalysis = runRustAnalysis;
  window.kgwRefreshAnalysisAddresses = loadSavedAddresses;
  window.kgwClearRustAnalysis = clearAnalysisView;

  bindWhenReady();
  document.addEventListener("DOMContentLoaded", bindWhenReady, { once: true });
  window.addEventListener("kgw:tab:shown", bindWhenReady);

  logAnalysis("Rust analysis binding installed: analysis_report owner.");
}

function bindWhenReady() {
  const r = root();
  if (!r) return;
  bindControls();
  updateRunState();

  if (!ANALYSIS_STATE.addressesLoaded) {
    loadSavedAddresses().catch((error) => {
      logAnalysisError("Failed to load analysis addresses", error);
      setStatus("Could not load saved addresses. Manual input is still available.", "error");
    });
  }
}

function root() {
  return document.querySelector("#analysis");
}

function q(selector) {
  const r = root();
  return r ? r.querySelector(selector) : null;
}

function invoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
}


function translateI18n(key) {
  const runtime = window.kgwT || window.kgwI18n;
  if (typeof runtime === "function") {
    const value = runtime(key);
    if (value && value !== key) return value;
  }
  return key;
}
function safeJson(value) {
  try { return JSON.stringify(value); } catch { return String(value); }
}

async function appendLog(level, message) {
  const call = invoke();
  if (!call) return;
  try {
    await call("kgw_log_append", {
      request: {
        level,
        target: "analysis",
        message
      }
    });
  } catch {
  }
}

function logAnalysis(message, data) {
  console.log("[KGW Analysis]", message, data || "");
  appendLog("INFO", data ? message + " " + safeJson(data) : message);
}

function logAnalysisError(message, error) {
  const text = message + ": " + (error && error.message ? error.message : String(error));
  console.error("[KGW Analysis]", text, error);
  appendLog("ERROR", text);
}

function bindControls() {
  const run = q("#analysisRun");
  const cancel = q("#analysisCancel");
  const select = q("#analysisAddressSelect");
  const input = q("#analysisAddressInput");

  if (run && run.dataset.kgwAnalysisBound !== "true") {
    run.dataset.kgwAnalysisBound = "true";
    run.addEventListener("click", (event) => {
      event.preventDefault();
      runRustAnalysis().catch((error) => {
        logAnalysisError("Analysis failed", error);
        setStatus(error && error.message ? error.message : String(error), "error");
        ANALYSIS_STATE.running = false;
        updateRunState();
      });
    });
  }

  if (cancel && cancel.dataset.kgwAnalysisBound !== "true") {
    cancel.dataset.kgwAnalysisBound = "true";
    cancel.addEventListener("click", (event) => {
      event.preventDefault();
      ANALYSIS_STATE.running = false;
      setStatus("Analysis cancelled.", "warn");
      updateRunState();
      appendLog("WARN", "Analysis cancelled by user.");
    });
  }

  if (select && select.dataset.kgwAnalysisBound !== "true") {
    select.dataset.kgwAnalysisBound = "true";
    select.addEventListener("change", () => {
      if (String(select.value || "").trim()) {
        const inputNode = q("#analysisAddressInput");
        if (inputNode) inputNode.value = "";
      }
      updateRunState();
    });
  }

  if (input && input.dataset.kgwAnalysisBound !== "true") {
    input.dataset.kgwAnalysisBound = "true";
    input.addEventListener("input", () => {
      if (String(input.value || "").trim()) {
        const selectNode = q("#analysisAddressSelect");
        if (selectNode) selectNode.value = "";
      }
      updateRunState();
    });
  }
}

function setStatus(message, state) {
  const node = q("#analysisStatus");
  if (!node) return;
  node.textContent = message;
  if (state) node.dataset.state = state;
  else delete node.dataset.state;
}

function selectedAddress() {
  const selectValue = String(q("#analysisAddressSelect")?.value || "").trim();
  if (selectValue) return selectValue;
  return String(q("#analysisAddressInput")?.value || "").trim();
}

function updateRunState() {
  const run = q("#analysisRun");
  const cancel = q("#analysisCancel");
  const address = selectedAddress();
  if (run) run.disabled = ANALYSIS_STATE.running || !address;
  if (cancel) cancel.disabled = !ANALYSIS_STATE.running;
}

function normalizeAddressRows(payload) {
  const source = Array.isArray(payload) ? payload : Array.isArray(payload?.addresses) ? payload.addresses : Array.isArray(payload?.items) ? payload.items : Array.isArray(payload?.rows) ? payload.rows : [];
  const seen = new Set();
  const rows = [];
  for (const item of source) {
    const address = typeof item === "string" ? item : String(item?.address || item?.kaspa_address || item?.wallet_address || item?.value || "").trim();
    if (!address || seen.has(address)) continue;
    seen.add(address);
    const label = typeof item === "string" ? address : String(item?.name || item?.label || item?.address_name || item?.title || address).trim();
    rows.push({ address, label });
  }
  rows.sort((a, b) => a.label.localeCompare(b.label));
  return rows;
}

async function loadSavedAddresses() {
  const select = q("#analysisAddressSelect");
  if (!select) return [];
  const call = invoke();
  if (!call) {
    setStatus("Tauri invoke API is not available. Manual input is available.", "error");
    updateRunState();
    return [];
  }
  const current = select.value;
  setStatus("Loading saved addresses...", "loading");
  const payload = await call("get_all_addresses");
  const rows = normalizeAddressRows(payload);
  select.innerHTML = "";
  const empty = document.createElement("option");
  empty.value = "";
  empty.textContent = translateI18n("ui.explorer.select.saved.address");
  empty.setAttribute("data-i18n", "ui.explorer.select.saved.address");
  select.appendChild(empty);
  for (const row of rows) {
    const option = document.createElement("option");
    option.value = row.address;
    option.textContent = row.label === row.address ? row.address : row.label + " — " + row.address;
    select.appendChild(option);
  }
  if (current && rows.some((row) => row.address === current)) select.value = current;
  ANALYSIS_STATE.addressesLoaded = true;
  setStatus(rows.length ? "Select an address to analyze." : "No saved addresses found. Use manual input.", rows.length ? "ok" : "warn");
  updateRunState();
  logAnalysis("analysis addresses loaded", { count: rows.length });
  return rows;
}

function getTimeRange() {
  const node = q("#analysisTimeRange") || q("[data-analysis-time-range]");
  const value = String(node?.value || "all").trim();
  return value || "all";
}

function formatMetricValue(metric) {
  if (!metric) return "—";
  if (metric.value !== undefined && metric.value !== null && metric.value !== "") return String(metric.value);
  if (metric.raw_number !== undefined && metric.raw_number !== null) return String(metric.raw_number);
  if (metric.raw_sompi !== undefined && metric.raw_sompi !== null) return String(Number(metric.raw_sompi) / 100000000);
  return "—";
}

function metricMap(report) {
  const map = new Map();
  for (const metric of Array.isArray(report?.metrics) ? report.metrics : []) {
    map.set(String(metric.label || "").toLowerCase(), formatMetricValue(metric));
  }
  return map;
}

function pickMetric(map, names) {
  for (const name of names) {
    const key = String(name).toLowerCase();
    if (map.has(key)) return map.get(key);
  }
  return "—";
}

function summaryFromReport(report) {
  const map = metricMap(report);
  return {
    totalInflow: pickMetric(map, ["Total Inflow (KAS)"]),
    totalOutflow: pickMetric(map, ["Total Outflow (KAS)"]),
    netFlow: pickMetric(map, ["Net Flow (KAS)"]),
    avgInflow: pickMetric(map, ["Avg Inflow (KAS)"]),
    avgOutflow: pickMetric(map, ["Avg Outflow (KAS)"]),
    totalTransactions: pickMetric(map, ["Total Transactions"]) !== "—" ? pickMetric(map, ["Total Transactions"]) : String(report?.total_transactions || 0),
    largestInflow: pickMetric(map, ["Largest Inflow (KAS)"]),
    largestOutflow: pickMetric(map, ["Largest Outflow (KAS)"]),
    uniqueCounterparties: pickMetric(map, ["Unique Counterparties"]),
    firstTransaction: pickMetric(map, ["First Transaction"]),
    lastTransaction: pickMetric(map, ["Last Transaction"]),
    durationDays: pickMetric(map, ["Duration (Days)"])
  };
}

function rowsFromReport(report) {
  const counterparties = Array.isArray(report?.counterparties) ? report.counterparties : [];
  if (counterparties.length) {
    return counterparties.map((row) => ({
      name: "Counterparty",
      datetime: "",
      address: row.counterparty || "",
      transactionId: row.counterparty || "",
      txsDir: String(row.transactions || 0),
      direction: "COUNTERPARTY",
      netFlow: String(row.net_kas ?? ""),
      amount: String(row.net_kas ?? ""),
      valueUsd: "",
      blockScore: "",
      type: "COUNTERPARTY"
    }));
  }
  const rows = Array.isArray(report?.rows) ? report.rows : [];
  return rows.map((row) => ({
    name: row.timestamp_ms ? new Date(Number(row.timestamp_ms)).toLocaleString() : "Transaction",
    datetime: row.timestamp_ms ? new Date(Number(row.timestamp_ms)).toLocaleString() : "",
    address: row.counterparty || row.address || "",
    transactionId: row.txid || "",
    txsDir: row.direction || "",
    direction: row.direction || "",
    netFlow: String(row.amount_kas ?? ""),
    amount: String(row.amount_kas ?? ""),
    valueUsd: "",
    blockScore: "",
    type: row.tx_type || "TX"
  }));
}

function emitAnalysisData(report) {
  const payload = {
    summary: summaryFromReport(report),
    rows: rowsFromReport(report),
    counterparties: rowsFromReport(report),
    rawReport: report
  };
  window.dispatchEvent(new CustomEvent("kgw:analysis", { detail: payload }));
  if (typeof window.kgwSetAnalysisData === "function") window.kgwSetAnalysisData(payload);
}

function setExportEnabled(enabled) {
  ["#analysisExportCsv", "#analysisExportHtml", "#analysisExportPdf"].forEach((selector) => {
    const node = q(selector);
    if (node) node.disabled = !enabled;
  });
}

function clearAnalysisView() {
  ANALYSIS_STATE.lastReport = null;
  setExportEnabled(false);
  window.dispatchEvent(new CustomEvent("kgw:analysis", { detail: { rows: [], summary: {} } }));
}

async function runRustAnalysis() {
  if (ANALYSIS_STATE.running) return;
  const call = invoke();
  if (!call) {
    setStatus("Tauri invoke API is not available.", "error");
    return;
  }
  const address = selectedAddress();
  if (!address) {
    setStatus("Select or enter a Kaspa address before analysis.", "error");
    updateRunState();
    return;
  }
  ANALYSIS_STATE.running = true;
  ANALYSIS_STATE.currentAddress = address;
  setExportEnabled(false);
  updateRunState();
  clearAnalysisView();
  setStatus("Running analysis_report...", "loading");
  logAnalysis("analysis_report requested", { address, time_range: getTimeRange() });
  try {
    const report = await call("analysis_report", {
      request: {
        address,
        time_range: getTimeRange(),
        limit: 5000,
        include_all_saved_addresses: false
      }
    });
    if (!ANALYSIS_STATE.running) {
      setStatus("Analysis cancelled.", "warn");
      return;
    }
    ANALYSIS_STATE.lastReport = report;
    emitAnalysisData(report);
    setExportEnabled(true);
    const total = report?.total_transactions || 0;
    setStatus("Analysis complete. Transactions: " + total, "ok");
    logAnalysis("analysis_report complete", { address, total_transactions: total });
  } catch (error) {
    logAnalysisError("analysis_report failed", error);
    setStatus(error && error.message ? error.message : String(error), "error");
    setExportEnabled(false);
  } finally {
    ANALYSIS_STATE.running = false;
    updateRunState();
  }
}
