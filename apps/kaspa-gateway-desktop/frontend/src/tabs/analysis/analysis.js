import { installRustAnalysisBinding } from "./analysis-rust-binding.js";

let analysisRows = [];
let filteredRows = [];

function root() {
  return document.getElementById("analysis");
}

function q(selector) {
  const r = root();
  return r ? r.querySelector(selector) : null;
}

function qa(selector) {
  const r = root();
  return r ? Array.from(r.querySelectorAll(selector)) : [];
}

function log() {
  if (typeof window.kgwCreateLogger === "function") {
    return window.kgwCreateLogger("analysis");
  }
  return { log: () => {}, warn: () => {}, error: () => {} };
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function toWesternDigits(value) {
  return String(value ?? "")
    .replace(/[٠-٩]/g, (digit) => String("٠١٢٣٤٥٦٧٨٩".indexOf(digit)))
    .replace(/[۰-۹]/g, (digit) => String("۰۱۲۳۴۵۶۷۸۹".indexOf(digit)));
}

function setText(selector, value) {
  const node = q(selector);
  if (node) node.textContent = value;
}

function setSummary(summary = {}) {
  const values = [
    summary.totalInflow,
    summary.totalOutflow,
    summary.netFlow,
    summary.avgInflow,
    summary.avgOutflow,
    summary.totalTransactions,
    summary.largestInflow,
    summary.largestOutflow,
    summary.uniqueCounterparties,
    summary.firstTransaction,
    summary.lastTransaction,
    summary.durationDays
  ];

  qa(".analysis-metric strong").forEach((node, index) => {
    const value = values[index];
    node.textContent = value === undefined || value === null || value === "" ? "—" : String(value);
    node.title = node.textContent;
  });
}

function rowMatches(row, search, type, direction) {
  const text = [
    row.name,
    row.datetime,
    row.address,
    row.transactionId,
    row.type,
    row.direction,
    row.txsDir,
    row.netFlow,
    row.amount,
    row.blockScore
  ].join(" ").toLowerCase();

  const rowType = String(row.type || "").toUpperCase();
  const rowDirection = String(row.direction || row.txsDir || "").toUpperCase();

  return (!search || text.includes(search)) &&
    (type === "ALL" || rowType === type) &&
    (direction === "ALL" || rowDirection === direction);
}

function applyFilter() {
  const search = String(q("#analysisSearch")?.value || "").trim().toLowerCase();
  const type = String(q("#analysisType")?.value || "ALL").toUpperCase();
  const direction = String(q("#analysisDirection")?.value || "ALL").toUpperCase();

  filteredRows = analysisRows.filter((row) => rowMatches(row, search, type, direction));
}

function emptyMessage() {
  return analysisRows.length
    ? "No rows match the current filter."
    : "Load an address to see analysis.";
}

function renderRows() {
  const body = q("#analysisRows");
  if (!body) return;

  applyFilter();

  if (!filteredRows.length) {
    body.innerHTML = `
      <tr class="analysis-empty-row">
        <td colspan="7">${escapeHtml(emptyMessage())}</td>
      </tr>
    `;
    return;
  }

  body.innerHTML = filteredRows.map((row) => {
    const first = row.name || row.datetime || "";
    const second = row.address || row.transactionId || "";
    const txsDir = row.txsDir || row.direction || "";
    const flow = row.netFlow || row.amount || "";

    return `
      <tr>
        <td title="${escapeHtml(first)}">${escapeHtml(first)}</td>
        <td title="${escapeHtml(second)}">${escapeHtml(second)}</td>
        <td title="${escapeHtml(txsDir)}">${escapeHtml(txsDir)}</td>
        <td title="${escapeHtml(flow)}">${escapeHtml(flow)}</td>
        <td title="${escapeHtml(row.valueUsd || "")}">${escapeHtml(row.valueUsd || "")}</td>
        <td title="${escapeHtml(row.blockScore || "")}">${escapeHtml(row.blockScore || "")}</td>
        <td title="${escapeHtml(row.type || "")}">${escapeHtml(row.type || "")}</td>
      </tr>
    `;
  }).join("");
}

function setFilterControlsEnabled(enabled) {
  [
    "#analysisSearch",
    "#analysisType",
    "#analysisDirection",
    "#analysisFilter",
    "#analysisResetFilter"
  ].forEach((selector) => {
    const node = q(selector);
    if (node) node.disabled = !enabled;
  });
}

function setAnalysisData(payload = {}) {
  const rows = Array.isArray(payload)
    ? payload
    : Array.isArray(payload.rows)
      ? payload.rows
      : Array.isArray(payload.counterparties)
        ? payload.counterparties
        : [];

  analysisRows = rows.map((row) => ({
    name: row.name ?? row.knownName ?? row.datetime ?? "",
    datetime: row.datetime ?? "",
    address: row.address ?? row.counterparty ?? "",
    transactionId: row.transactionId ?? row.txid ?? "",
    txsDir: row.txsDir ?? row.direction ?? "",
    direction: row.direction ?? "",
    netFlow: row.netFlow ?? row.amount ?? row.net_kas ?? "",
    amount: row.amount ?? row.amount_kas ?? row.net_kas ?? "",
    valueUsd: row.valueUsd ?? row.value ?? "",
    blockScore: row.blockScore ?? row.block ?? "",
    type: row.type ?? row.tx_type ?? ""
  }));

  setSummary(payload.summary || {});
  setFilterControlsEnabled(analysisRows.length > 0);
  renderRows();

  log().log("analysis rows set", { count: analysisRows.length });
}

function bindCalendar() {
  qa(".analysis-calendar-btn").forEach((button) => {
    if (button.dataset.bound === "true") return;
    button.dataset.bound = "true";

    button.addEventListener("click", () => {
      const dateFor = button.dataset.dateFor;
      const nativeFor = button.dataset.nativeFor;
      if (!dateFor || !nativeFor || typeof CSS === "undefined" || typeof CSS.escape !== "function") return;

      const textInput = q(`#${CSS.escape(dateFor)}`);
      const nativeInput = q(`#${CSS.escape(nativeFor)}`);
      if (!textInput || !nativeInput) return;

      nativeInput.value = toWesternDigits(textInput.value);

      if (typeof nativeInput.showPicker === "function") {
        nativeInput.showPicker();
      } else {
        nativeInput.focus();
        nativeInput.click();
      }
    });
  });

  qa(".analysis-date-native").forEach((nativeInput) => {
    if (nativeInput.dataset.bound === "true") return;
    nativeInput.dataset.bound = "true";

    nativeInput.addEventListener("change", () => {
      const id = nativeInput.id === "analysisFromDateNative"
        ? "analysisFromDate"
        : "analysisToDate";

      const textInput = q(`#${id}`);
      if (textInput) textInput.value = nativeInput.value;
    });
  });

  qa(".analysis-date-text").forEach((input) => {
    if (input.dataset.boundDigits === "true") return;
    input.dataset.boundDigits = "true";

    input.addEventListener("input", () => {
      input.value = toWesternDigits(input.value);
    });
  });
}

function bindControls() {
  const search = q("#analysisSearch");
  const type = q("#analysisType");
  const direction = q("#analysisDirection");
  const filter = q("#analysisFilter");
  const reset = q("#analysisResetFilter");

  [search, type, direction].forEach((node) => {
    if (!node || node.dataset.bound === "true") return;
    node.dataset.bound = "true";

    const eventName = node.tagName === "INPUT" ? "input" : "change";
    node.addEventListener(eventName, renderRows);
  });

  if (filter && filter.dataset.bound !== "true") {
    filter.dataset.bound = "true";
    filter.addEventListener("click", renderRows);
  }

  if (reset && reset.dataset.bound !== "true") {
    reset.dataset.bound = "true";
    reset.addEventListener("click", () => {
      if (search) search.value = "";
      if (type) type.value = "ALL";
      if (direction) direction.value = "ALL";
      renderRows();
    });
  }
}

function installHook() {
  if (window.__kgwAnalysisRebuildHookInstalled) return;
  window.__kgwAnalysisRebuildHookInstalled = true;

  window.addEventListener("kgw:analysis", (event) => {
    setAnalysisData(event.detail || {});
  });

  window.kgwSetAnalysisData = (payload) => {
    setAnalysisData(payload || {});
  };
}

export async function initAnalysisTab() {
  if (!root()) return;

  bindControls();
  bindCalendar();
  installHook();

  if (root().dataset.analysisInitialized !== "true") {
    root().dataset.analysisInitialized = "true";
    setAnalysisData({ rows: [], summary: {} });
    setText("#analysisStatus", "Load an address to see analysis.");
    log().log("analysis rebuilt owner initialized");
  } else {
    renderRows();
  }
}

installRustAnalysisBinding();
