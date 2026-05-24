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




// KGW_ANALYSIS_PYTHON_STYLE_LAZY_TABLE_R9
const kgwAnalysisExpandedRowsR9 = new Set();

function kgwAnalysisPythonRowKeyR9(row, index) {
  return [
    index,
    kgwAnalysisRawR3(row.type || ""),
    kgwAnalysisRawR3(row.address || row.counterparty || ""),
    kgwAnalysisRawR3(row.transactionId || row.txid || ""),
    kgwAnalysisRawR3(row.name || row.knownName || "")
  ].join("|");
}

function kgwAnalysisPythonChildRowsR9(row) {
  const candidates = [
    row.transactions,
    row.children,
    row.details,
    row.items,
    row.txList,
    row.tx_list,
    row.rows
  ];

  for (const value of candidates) {
    if (Array.isArray(value)) return value;
  }

  return [];
}

function kgwAnalysisPythonNormalizeChildR9(tx, index) {
  if (!tx || typeof tx !== "object") {
    return {
      datetime: "",
      id: "",
      direction: "",
      amount: "",
      value: "",
      block: "",
      type: ""
    };
  }

  const timestampRaw = tx.timestamp_ms ?? tx.timestampMs ?? tx.timestamp ?? tx.datetime ?? tx.date_time ?? tx.time ?? "";
  let datetime = kgwAnalysisRawR3(timestampRaw);

  const numericTs = Number(timestampRaw);
  if (Number.isFinite(numericTs) && numericTs > 0) {
    const ms = numericTs > 10000000000 ? numericTs : numericTs * 1000;
    const d = new Date(ms);
    if (!Number.isNaN(d.getTime())) {
      datetime = [
        d.getFullYear(),
        "-",
        String(d.getMonth() + 1).padStart(2, "0"),
        "-",
        String(d.getDate()).padStart(2, "0"),
        " ",
        String(d.getHours()).padStart(2, "0"),
        ":",
        String(d.getMinutes()).padStart(2, "0"),
        ":",
        String(d.getSeconds()).padStart(2, "0")
      ].join("");
    }
  }

  const amount =
    tx.amount_kas ??
    tx.amountKas ??
    tx.amount ??
    tx.value ??
    "";

  return {
    datetime,
    id: kgwAnalysisRawR3(tx.txid || tx.transaction_id || tx.transactionId || tx.id || tx.hash || ""),
    direction: kgwAnalysisRawR3(tx.direction || tx.txsDir || ""),
    amount,
    value: tx.value_usd ?? tx.valueUsd ?? "",
    block: tx.block_score ?? tx.blockScore ?? tx.block_height ?? tx.blockHeight ?? "",
    type: kgwAnalysisRawR3(tx.tx_type || tx.txType || tx.type || ""),
    counterparty: kgwAnalysisRawR3(tx.counterparty || tx.address || tx.from_address || tx.to_address || "")
  };
}

function kgwAnalysisPythonRenderChildrenR9(rowKey, row, colspan = 7) {
  const children = kgwAnalysisPythonChildRowsR9(row);

  if (!children.length) {
    return `
      <tr class="kgw-analysis-python-child-row-r9 kgw-analysis-python-tree-empty-r14" data-kgw-analysis-python-child="true" data-row-key="${escapeHtml(rowKey)}">
        <td colspan="${colspan}" class="kgw-analysis-python-empty-r9">
          No transaction rows found for this parent.
        </td>
      </tr>
    `;
  }

  return children.map((tx, childIndex) => {
    const item = kgwAnalysisPythonNormalizeChildR9(tx, childIndex);
    const amountDisplay = kgwAnalysisFlowSignR3(item.amount);
    const valueDisplay = kgwAnalysisValueUsdR16(item.amount, item.value);
    const idDisplay = item.id || item.counterparty || "";
    const directionDisplay = kgwAnalysisDirectionCaseR16(item.direction);
    const typeDisplay = kgwAnalysisTitleCaseR16(item.type);

    return `
      <tr class="kgw-analysis-python-child-row-r9 kgw-analysis-python-tree-child-r14 kgw-analysis-child-columns-r16" data-kgw-analysis-python-child="true" data-row-key="${escapeHtml(rowKey)}">
        <td class="kgw-analysis-tree-date-r14">${escapeHtml(item.datetime || "—")}</td>
        <td class="kgw-analysis-tree-id-r14" title="${escapeHtml(item.id || item.counterparty || "")}">
          <span class="kgw-analysis-mono">${escapeHtml(idDisplay || "—")}</span>
        </td>
        <td class="kgw-analysis-tree-dir-r14">${escapeHtml(directionDisplay)}</td>
        <td class="kgw-analysis-tree-amount-r14">${escapeHtml(amountDisplay || "—")}</td>
        <td class="kgw-analysis-tree-value-r14">${escapeHtml(valueDisplay)}</td>
        <td class="kgw-analysis-tree-block-r14">${escapeHtml(item.block || "—")}</td>
        <td class="kgw-analysis-tree-type-r14">${escapeHtml(typeDisplay)}</td>
      </tr>
    `;
  }).join("");
}

function kgwAnalysisPythonBindR9() {
  const body = q("#analysisRows");
  if (!body || body.dataset.kgwAnalysisPythonLazyR9 === "true") return;

  body.dataset.kgwAnalysisPythonLazyR9 = "true";

  body.addEventListener("click", (event) => {
    const button = event.target && event.target.closest
      ? event.target.closest("[data-kgw-analysis-python-toggle-r9]")
      : null;

    if (!button || !body.contains(button)) return;

    event.preventDefault();

    const rowKey = button.getAttribute("data-row-key") || "";
    if (!rowKey) return;

    if (kgwAnalysisExpandedRowsR9.has(rowKey)) {
      kgwAnalysisExpandedRowsR9.delete(rowKey);
    } else {
      kgwAnalysisExpandedRowsR9.add(rowKey);
    }

    renderRows();
  });
}

function kgwAnalysisPythonResetR9() {
  kgwAnalysisExpandedRowsR9.clear();
}



// KGW_ANALYSIS_PYTHON_TREEVIEW_VISUAL_PATCH_R14
function kgwAnalysisPlainNumberR14(value) {
  const numeric = Number(value);
  if (!Number.isFinite(numeric)) return kgwAnalysisRawR3(value || "");
  return numeric.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2
  });
}


// KGW_ANALYSIS_CHILD_COLUMNS_PYTHON_PARITY_PATCH_R16
function kgwAnalysisCurrentUsdPriceR16() {
  const text = document.body ? String(document.body.textContent || "") : "";
  const matches = [...text.matchAll(/(\d+(?:\.\d+)?)\s*USD/g)];
  if (!matches.length) return null;
  const price = Number(matches[0][1]);
  return Number.isFinite(price) && price > 0 ? price : null;
}

function kgwAnalysisValueUsdR16(amount, explicitValue) {
  const explicit = kgwAnalysisNumberR3(explicitValue);
  if (explicit !== null) return kgwAnalysisFormatUsdR3(explicit);

  const numericAmount = kgwAnalysisNumberR3(amount);
  const price = kgwAnalysisCurrentUsdPriceR16();

  if (numericAmount === null || price === null) return "—";
  return kgwAnalysisFormatUsdR3(Math.abs(numericAmount) * price);
}

function kgwAnalysisTitleCaseR16(value) {
  const raw = kgwAnalysisRawR3(value);
  if (!raw) return "—";
  return raw
    .split(/[\s_-]+/)
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1).toLowerCase())
    .join(" ");
}

function kgwAnalysisDirectionCaseR16(value) {
  const raw = kgwAnalysisRawR3(value);
  if (!raw) return "—";
  return raw.slice(0, 1).toUpperCase() + raw.slice(1).toLowerCase();
}

// KGW_ANALYSIS_RESULTS_TABLE_UI_PATCH_R3
function kgwAnalysisRawR3(value) {
  return value === undefined || value === null ? "" : String(value).trim();
}

function kgwAnalysisNumberR3(value) {
  const raw = kgwAnalysisRawR3(value).replace(/,/g, "").replace(/\s+(KAS|USD)$/i, "");
  if (!raw) return null;
  const number = Number(raw);
  return Number.isFinite(number) ? number : null;
}

function kgwAnalysisFormatKasR3(value) {
  const number = kgwAnalysisNumberR3(value);
  if (number === null) return kgwAnalysisRawR3(value);
  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: 3
  }).format(number);
}

function kgwAnalysisFormatCountR3(value) {
  const number = kgwAnalysisNumberR3(value);
  if (number === null) return kgwAnalysisRawR3(value);
  return new Intl.NumberFormat("en-US", {
    maximumFractionDigits: 0
  }).format(number);
}

function kgwAnalysisFormatUsdR3(value) {
  const number = kgwAnalysisNumberR3(value);
  if (number === null) return kgwAnalysisRawR3(value);
  return "$" + new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: 2
  }).format(number);
}

function kgwAnalysisMiddleTrimR3(value, head = 18, tail = 10) {
  const raw = kgwAnalysisRawR3(value);
  if (raw.length <= head + tail + 3) return raw;
  return raw.slice(0, head) + "…" + raw.slice(-tail);
}

function kgwAnalysisDirectionLabelR3(row) {
  const direction = kgwAnalysisRawR3(row.direction || row.txsDir).toUpperCase();
  if (direction === "IN" || direction === "OUT" || direction === "MIXED") return direction;
  return direction && !/^\d+$/.test(direction) ? direction : "—";
}

function kgwAnalysisTxCountR3(row) {
  return row.txCount || row.txs || row.transactions || row.txsDir || "";
}

function kgwAnalysisFlowClassR3(value) {
  const number = kgwAnalysisNumberR3(value);
  if (number === null) return "is-neutral";
  if (number > 0) return "is-positive";
  if (number < 0) return "is-negative";
  return "is-neutral";
}

function kgwAnalysisFlowSignR3(value) {
  const number = kgwAnalysisNumberR3(value);
  if (number === null) return kgwAnalysisRawR3(value);
  const formatted = kgwAnalysisFormatKasR3(number);
  return number > 0 ? "+" + formatted : formatted;
}

function kgwAnalysisTypeLabelR3(value) {
  const raw = kgwAnalysisRawR3(value);
  return raw || "—";
}

function renderRows() {
  const body = q("#analysisRows");
  if (!body) return;

  applyFilter();
  kgwAnalysisPythonBindR9();

  if (!filteredRows.length) {
    body.innerHTML = `
      <tr class="analysis-empty-row kgw-analysis-python-tree-empty-r14">
        <td colspan="7">${escapeHtml(emptyMessage())}</td>
      </tr>
    `;
    return;
  }

  body.innerHTML = filteredRows.map((row, index) => {
    const rowKey = kgwAnalysisPythonRowKeyR9(row, index);
    const expanded = kgwAnalysisExpandedRowsR9.has(rowKey);
    const childRows = expanded ? kgwAnalysisPythonRenderChildrenR9(rowKey, row, 7) : "";

    const primary = row.name || row.knownName || row.datetime || "Counterparty";
    const isCoinbase = /coinbase|mining/i.test(String(row.address || row.transactionId || primary || row.type || ""));
    const parentLabel = isCoinbase ? "Coinbase / Mining" : primary;
    const addressOrTx = row.address || row.transactionId || "";
    const addressOrTxDisplay = isCoinbase ? "" : kgwAnalysisMiddleTrimR3(addressOrTx, 64, 18);
    const txCount = kgwAnalysisFormatCountR3(kgwAnalysisTxCountR3(row));
    const flowRaw = row.netFlow || row.amount || "";
    const flowDisplay = kgwAnalysisPlainNumberR14(flowRaw);
    const valueDisplay = kgwAnalysisValueUsdR16(flowRaw, row.valueUsd || row.value || "");
    const blockDisplay = kgwAnalysisMiddleTrimR3(row.blockScore || "", 12, 8);
    const typeDisplay = isCoinbase ? "" : kgwAnalysisTypeLabelR3(row.type);
    const plusLabel = expanded ? "⊟" : "⊞";

    return `
      <tr class="kgw-analysis-result-row kgw-analysis-python-parent-row-r9 kgw-analysis-python-tree-parent-r14" data-row-key="${escapeHtml(rowKey)}">
        <td class="kgw-analysis-tree-name-r14" title="${escapeHtml(parentLabel)}">
          <button
            type="button"
            class="kgw-analysis-python-toggle-r9 kgw-analysis-tree-toggle-r14"
            data-kgw-analysis-python-toggle-r9="true"
            data-row-key="${escapeHtml(rowKey)}"
            aria-expanded="${expanded ? "true" : "false"}"
            title="${expanded ? "Hide transactions" : "Show transactions"}"
          >${plusLabel}</button>
          <span class="kgw-analysis-tree-parent-label-r14">${escapeHtml(parentLabel)}</span>
        </td>
        <td class="kgw-analysis-tree-id-r14" title="${escapeHtml(addressOrTx)}">
          <span class="kgw-analysis-mono">${escapeHtml(addressOrTxDisplay)}</span>
        </td>
        <td class="kgw-analysis-tree-txs-r14">${escapeHtml(txCount || "—")}</td>
        <td class="kgw-analysis-tree-amount-r14">${escapeHtml(flowDisplay || "—")}</td>
        <td class="kgw-analysis-tree-value-r14">${escapeHtml(valueDisplay || "")}</td>
        <td class="kgw-analysis-tree-block-r14">${escapeHtml(blockDisplay || "")}</td>
        <td class="kgw-analysis-tree-type-r14">${escapeHtml(typeDisplay || "")}</td>
      </tr>
      ${childRows}
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
  kgwAnalysisPythonResetR9();
  const rows = Array.isArray(payload)
    ? payload
    : Array.isArray(payload.rows)
      ? payload.rows
      : Array.isArray(payload.counterparties)
        ? payload.counterparties
        : [];

  analysisRows = rows.map((row) => ({
    name: row.name ?? row.knownName ?? row.datetime ?? "",
    knownName: row.knownName ?? row.name ?? "",
    datetime: row.datetime ?? "",
    address: row.address ?? row.counterparty ?? "",
    transactionId: row.transactionId ?? row.txid ?? "",
    txCount: row.txCount ?? row.txs ?? row.transactions ?? row.count ?? "",
    txsDir: row.txsDir ?? row.direction ?? "",
    direction: row.direction ?? "",
    netFlow: row.netFlow ?? row.amount ?? row.net_kas ?? "",
    amount: row.amount ?? row.amount_kas ?? row.net_kas ?? "",
    valueUsd: row.valueUsd ?? row.value ?? "",
    blockScore: row.blockScore ?? row.block ?? "",
    type: row.type ?? row.tx_type ?? "",
    details: row.details ?? row.children ?? row.items ?? row.txList ?? row.tx_list ?? row.rows ?? (Array.isArray(row.transactions) ? row.transactions : []),
    transactions: row.details ?? row.children ?? row.items ?? row.txList ?? row.tx_list ?? row.rows ?? (Array.isArray(row.transactions) ? row.transactions : []),
    firstSeen: row.firstSeen ?? row.firstTransaction ?? row.first ?? "",
    lastSeen: row.lastSeen ?? row.lastTransaction ?? row.last ?? ""
  }));

  setSummary(payload.summary || {});
  setFilterControlsEnabled(analysisRows.length > 0);
  kgwAnalysisSetExportButtonsEnabledV1G(analysisRows.length > 0);
  renderRows();

  log().log("analysis rows set", { count: analysisRows.length });
}

/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_ANALYSIS_OWNER_START */
/* KGW_CALENDAR_EXISTING_OWNER_REGEX_FIX_R4: fixed regex escaping only; no new owner layer.\n * KGW_CALENDAR_EXISTING_OWNER_DOM_CSS_FIX_R6: robust DOM contract + body-attached compact popover inside existing owner.\n * KGW_CALENDAR_SCOPED_POPOVER_OWNER_FIX_R7: scope-isolated popovers per tab.\n * KGW_CALENDAR_SINGLE_ACTIVE_POPOVER_FIX_R8: removes stale body-attached popovers before opening current tab calendar.
/* */
function kgwAnalysisPad2(value) {
  return String(value).padStart(2, "0");
}

function kgwAnalysisTodayIso() {
  const now = new Date();
  return `${now.getFullYear()}-${kgwAnalysisPad2(now.getMonth() + 1)}-${kgwAnalysisPad2(now.getDate())}`;
}

function kgwAnalysisIsoFromDate(date) {
  return `${date.getFullYear()}-${kgwAnalysisPad2(date.getMonth() + 1)}-${kgwAnalysisPad2(date.getDate())}`;
}

function kgwAnalysisCleanIso(value, fallbackValue = kgwAnalysisTodayIso()) {
  const clean = toWesternDigits(value || fallbackValue).replace(/[^0-9-]/g, "").slice(0, 10);
  return /^\d{4}-\d{2}-\d{2}$/.test(clean) ? clean : fallbackValue;
}

function kgwAnalysisParseIso(value, fallbackValue = kgwAnalysisTodayIso()) {
  const clean = kgwAnalysisCleanIso(value, fallbackValue);
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(clean);
  if (!match) return kgwAnalysisParseIso(fallbackValue, kgwAnalysisTodayIso());

  const year = Number(match[1]);
  const month = Number(match[2]) - 1;
  const day = Number(match[3]);
  const date = new Date(year, month, day);

  if (date.getFullYear() !== year || date.getMonth() !== month || date.getDate() !== day) {
    return kgwAnalysisParseIso(fallbackValue, kgwAnalysisTodayIso());
  }

  return date;
}

function kgwAnalysisMonthLabel(year, monthIndex) {
  return new Intl.DateTimeFormat("en-US", {
    month: "long",
    year: "numeric",
    calendar: "gregory",
    numberingSystem: "latn"
  }).format(new Date(year, monthIndex, 1));
}


// KGW_ANALYSIS_SAFE_CONTROLS_TRACE_PATCH_R50B
function kgwAnalysisUiTraceR50B(action, phase, details) {
  try {
    const safeAction = String(action || "analysis-ui");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "analysis",
      net: "ui",
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_ANALYSIS_SAFE_CONTROLS_TRACE_PATCH_R50B",
        owner: "analysis-existing-safe-owners",
        action: safeAction,
        phase: safePhase,
        details: safeDetails
      })
    };
    const tauri = window.__TAURI__;
    const invoke = tauri && tauri.core && typeof tauri.core.invoke === "function"
      ? tauri.core.invoke.bind(tauri.core)
      : tauri && typeof tauri.invoke === "function"
        ? tauri.invoke.bind(tauri)
        : window.__TAURI_INVOKE__;
    if (typeof invoke === "function") invoke("kgw_frontend_button_trace_v1", args).catch(function () {});
  } catch (_) {}
}
function kgwAnalysisEscapeSelector(id) {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") return CSS.escape(id);
  return String(id).replace(/["\\]/g, "\\$&");
}

// KGW_ANALYSIS_FILTER_BAR_REBUILD_SINGLE_OWNER_R23
function kgwAnalysisMarkDateTouchedR18(node) {
  if (node) node.dataset.kgwAnalysisUserTouched = "true";
}

function kgwAnalysisBindDateTouchR18() {
  ["analysisFromDate", "analysisFromDateNative", "analysisToDate", "analysisToDateNative"].forEach((id) => {
    const node = q("#" + id);
    if (!node || node.dataset.kgwAnalysisTouchR23 === "true") return;
    node.dataset.kgwAnalysisTouchR23 = "true";
    ["input", "change"].forEach((eventName) => {
      node.addEventListener(eventName, (event) => {
        if (event && event.isTrusted) kgwAnalysisMarkDateTouchedR18(node);
      });
    });
  });
}

function kgwAnalysisEnsureTodayToDateR18(force = false) {
  const range = kgwAnalysisRange();
  if (!range || !range.toInput) return "";

  const today = kgwAnalysisTodayIso();
  const touched = range.toInput.dataset.kgwAnalysisUserTouched === "true"
    || (range.toNative && range.toNative.dataset.kgwAnalysisUserTouched === "true");

  if (force || !touched || !range.toInput.value || range.toInput.value !== today) {
    if (force || !touched) {
      range.toInput.dataset.kgwAnalysisOwnerDefaultR23 = "today";
      if (range.toNative) range.toNative.dataset.kgwAnalysisOwnerDefaultR23 = "today";
      return kgwAnalysisSetDate(range.toInput, range.toNative, today);
    }
  }

  return kgwAnalysisCleanIso(range.toInput.value, today);
}

function kgwAnalysisFilterBarButtonR23(textInput, nativeInput, role) {
  const owner = root();
  if (!owner || !textInput) return null;

  const textId = String(textInput.id || "");
  const nativeId = String(nativeInput && nativeInput.id ? nativeInput.id : "");

  let button =
    (textId ? owner.querySelector(`[data-date-for="${kgwAnalysisEscapeSelector(textId)}"]`) : null)
    || (nativeId ? owner.querySelector(`[data-native-for="${kgwAnalysisEscapeSelector(nativeId)}"]`) : null)
    || textInput.closest(".kgw-analysis-date-field")?.querySelector(".analysis-calendar-btn")
    || textInput.parentElement?.querySelector(".analysis-calendar-btn")
    || null;

  if (!button) {
    button = document.createElement("button");
    button.type = "button";
    button.className = "analysis-calendar-btn kgw-analysis-calendar-created-r23";
    button.textContent = "📅";
    button.setAttribute("aria-label", role === "from" ? "Open from date calendar" : "Open to date calendar");
    button.addEventListener("click", () => {
      if (!nativeInput) {
        textInput.focus();
        return;
      }

      try {
        if (typeof nativeInput.showPicker === "function") {
          nativeInput.showPicker();
        } else {
          nativeInput.focus();
          nativeInput.click();
        }
      } catch {
        nativeInput.focus();
        nativeInput.click();
      }
    });
  }

  button.classList.add("kgw-analysis-filter-calendar-button-r23", `kgw-analysis-filter-${role}-calendar-r23`);
  button.setAttribute("data-kgw-analysis-calendar-role-r23", role);
  button.setAttribute("type", button.getAttribute("type") || "button");

  return button;
}

function kgwAnalysisFilterBarLcaR23(nodes) {
  const filtered = nodes.filter(Boolean);
  if (!filtered.length) return null;

  function ancestors(node) {
    const out = [];
    let current = node;
    while (current) {
      out.push(current);
      current = current.parentElement;
    }
    return out;
  }

  const first = ancestors(filtered[0]);
  return first.find((candidate) => filtered.every((node) => candidate.contains(node))) || null;
}

function kgwAnalysisFilterBarRemoveLabelsR23(node, terms) {
  if (!node) return;

  node.setAttribute("aria-label", terms[0]);
  node.setAttribute("title", terms[0]);

  const owner = root() || document;
  const id = String(node.id || "");

  if (id) {
    owner.querySelectorAll(`label[for="${kgwAnalysisEscapeSelector(id)}"]`).forEach((label) => label.remove());
  }

  const parent = node.parentElement;
  if (parent) {
    for (const child of Array.from(parent.childNodes || [])) {
      if (child.nodeType === Node.TEXT_NODE) {
        let text = child.textContent || "";
        for (const term of terms) {
          text = text.replace(new RegExp("\\b" + term + "\\s*:?", "gi"), "");
        }
        child.textContent = text;
      }
    }
  }

  const prev = node.previousElementSibling;
  if (prev && /^(LABEL|SPAN|SMALL|B)$/i.test(prev.tagName || "")) {
    const text = String(prev.textContent || "").trim().replace(/:$/, "");
    if (terms.some((term) => text.toLowerCase() === term.toLowerCase())) {
      prev.remove();
    }
  }
}

function kgwAnalysisFilterBarMakeDateUnitR23(role, textInput, nativeInput, button) {
  const unit = document.createElement("span");
  unit.className = `kgw-analysis-filter-date-unit-r23 kgw-analysis-filter-date-${role}-r23`;
  unit.setAttribute("data-kgw-analysis-filter-date-unit-r23", role);

  if (textInput) {
    textInput.classList.add("kgw-analysis-filter-date-text-r23");
    textInput.setAttribute("data-kgw-analysis-filter-date-text-r23", role);
    unit.appendChild(textInput);
  }

  if (button) {
    button.classList.add("kgw-analysis-filter-date-button-r23");
    button.setAttribute("data-kgw-analysis-filter-date-button-r23", role);
    unit.appendChild(button);
  }

  if (nativeInput) {
    nativeInput.classList.add("kgw-analysis-filter-date-native-r23");
    nativeInput.setAttribute("data-kgw-analysis-filter-date-native-r23", role);
    unit.appendChild(nativeInput);
  }

  return unit;
}

function kgwAnalysisFilterBarAppendLabelR23(bar, text, className) {
  const label = document.createElement("span");
  label.className = `kgw-analysis-filter-label-r23 ${className}`;
  label.textContent = text;
  bar.appendChild(label);
  return label;
}

function kgwAnalysisFilterBarAppendControlR23(bar, node, className, label) {
  if (!node) return null;
  node.classList.add("kgw-analysis-filter-control-r23", className);
  node.setAttribute("data-kgw-analysis-filter-owner-r23", "true");
  if (label) {
    node.setAttribute("aria-label", label);
    node.setAttribute("title", label);
  }
  bar.appendChild(node);
  return node;
}

function kgwAnalysisNormalizeFilterLayoutR18() {
  const owner = root();
  if (!owner) return;

  const range = kgwAnalysisRange();
  const fromInput = range?.fromInput || q("#analysisFromDate");
  const fromNative = range?.fromNative || q("#analysisFromDateNative");
  const toInput = range?.toInput || q("#analysisToDate");
  const toNative = range?.toNative || q("#analysisToDateNative");

  const type = q("#analysisType");
  const direction = q("#analysisDirection");
  const search = q("#analysisSearch");
  const filter = q("#analysisFilter");
  const reset = q("#analysisResetFilter");

  const fromButton = kgwAnalysisFilterBarButtonR23(fromInput, fromNative, "from");
  const toButton = kgwAnalysisFilterBarButtonR23(toInput, toNative, "to");

  kgwAnalysisFilterBarRemoveLabelsR23(type, ["Type"]);
  kgwAnalysisFilterBarRemoveLabelsR23(direction, ["Direction"]);
  kgwAnalysisFilterBarRemoveLabelsR23(search, ["Search"]);

  const controls = [fromInput, fromButton, toInput, toButton, type, direction, search, filter, reset].filter(Boolean);
  let host = kgwAnalysisFilterBarLcaR23(controls);

  if (!host || host === owner || host === document.body || host === document.documentElement) {
    host = filter?.closest(".analysis-filter-row, .kgw-analysis-filter-calendar-r18, .kgw-analysis-filter-bar-owner-r20, .kgw-analysis-date-calendar-owner-r22")
      || search?.closest(".analysis-filter-row, .kgw-analysis-filter-calendar-r18, .kgw-analysis-filter-bar-owner-r20, .kgw-analysis-date-calendar-owner-r22")
      || filter?.parentElement
      || owner;
  }

  if (host === owner) {
    const fallback = filter?.parentElement || search?.parentElement;
    if (fallback) host = fallback;
  }

  const fromUnit = kgwAnalysisFilterBarMakeDateUnitR23("from", fromInput, fromNative, fromButton);
  const toUnit = kgwAnalysisFilterBarMakeDateUnitR23("to", toInput, toNative, toButton);

  const bar = document.createElement("div");
  bar.className = "kgw-analysis-filter-bar-rebuild-r23";
  bar.setAttribute("data-kgw-analysis-filter-bar-owner-r23", "true");

  kgwAnalysisFilterBarAppendLabelR23(bar, "From:", "kgw-analysis-filter-from-label-r23");
  bar.appendChild(fromUnit);
  kgwAnalysisFilterBarAppendLabelR23(bar, "To:", "kgw-analysis-filter-to-label-r23");
  bar.appendChild(toUnit);
  kgwAnalysisFilterBarAppendControlR23(bar, type, "kgw-analysis-filter-type-r23", "Type");
  kgwAnalysisFilterBarAppendControlR23(bar, direction, "kgw-analysis-filter-direction-r23", "Direction");
  kgwAnalysisFilterBarAppendControlR23(bar, search, "kgw-analysis-filter-search-r23", "Search by Address or Transaction");
  kgwAnalysisFilterBarAppendControlR23(bar, filter, "kgw-analysis-filter-apply-r23", "Filter");
  kgwAnalysisFilterBarAppendControlR23(bar, reset, "kgw-analysis-filter-reset-r23", "Reset Filter");

  if (search) search.placeholder = "Search by Address/Transaction...";

  while (host.firstChild) host.removeChild(host.firstChild);

  host.classList.remove(
    "kgw-analysis-filter-calendar-r18",
    "kgw-analysis-filter-bar-owner-r20",
    "kgw-analysis-date-calendar-owner-r22"
  );
  host.classList.add("kgw-analysis-filter-host-r23");
  host.setAttribute("data-kgw-analysis-filter-host-r23", "true");
  host.appendChild(bar);

  kgwAnalysisEnsureTodayToDateR18(false);
}

function kgwAnalysisSetDate(textInput, nativeInput, value) {
  const clean = kgwAnalysisCleanIso(value);
  if (textInput) {
    textInput.value = clean;
    textInput.dispatchEvent(new Event("input", { bubbles: true }));
    textInput.dispatchEvent(new Event("change", { bubbles: true }));
  }
  if (nativeInput) {
    nativeInput.value = clean;
    nativeInput.dispatchEvent(new Event("change", { bubbles: true }));
  }
  return clean;
}

function kgwAnalysisRange() {
  const fromInput = q("#analysisFromDate");
  const toInput = q("#analysisToDate");
  const fromNative = q("#analysisFromDateNative");
  const toNative = q("#analysisToDateNative");
  if (!fromInput || !toInput) return null;
  return { fromInput, toInput, fromNative, toNative };
}

function kgwAnalysisCloseCalendar(scope = "analysis") {
  document.querySelectorAll(`.kgw-calendar-popover[data-kgw-calendar-scope="${scope}"]`).forEach((node) => node.remove());

  const owner = root() || document;
  owner.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    if (!node.dataset.kgwCalendarScope || node.dataset.kgwCalendarScope === scope) {
      delete node.dataset.kgwCalendarOpen;
      delete node.dataset.kgwCalendarScope;
    }
  });
}

function kgwAnalysisApplyPreset(activeTextId, presetName) {
  const today = kgwAnalysisParseIso(kgwAnalysisTodayIso());
  const range = kgwAnalysisRange();

  let from = kgwAnalysisIsoFromDate(today);
  let to = kgwAnalysisIsoFromDate(today);

  if (presetName === "last7") {
    const start = new Date(today);
    start.setDate(start.getDate() - 6);
    from = kgwAnalysisIsoFromDate(start);
  } else if (presetName === "last30") {
    const start = new Date(today);
    start.setDate(start.getDate() - 29);
    from = kgwAnalysisIsoFromDate(start);
  } else if (presetName === "thisMonth") {
    from = kgwAnalysisIsoFromDate(new Date(today.getFullYear(), today.getMonth(), 1));
  } else if (presetName === "sinceLaunch") {
    from = "2021-11-07";
  }

  if (range) {
    kgwAnalysisSetDate(range.fromInput, range.fromNative, from);
    kgwAnalysisSetDate(range.toInput, range.toNative, to);
  } else {
    const activeInput = q(`#${kgwAnalysisEscapeSelector(activeTextId)}`);
    kgwAnalysisSetDate(activeInput, null, presetName === "today" ? to : from);
  }

  renderRows();
  kgwAnalysisCloseCalendar("analysis");
}

function kgwAnalysisAttachPopover(popover, anchor) {
  document.querySelectorAll(".kgw-calendar-popover").forEach((node) => node.remove());
  document.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    delete node.dataset.kgwCalendarOpen;
    delete node.dataset.kgwCalendarScope;
  });

  popover.dataset.kgwCalendarScope = "analysis";
  popover.classList.add("kgw-calendar-popover-analysis");

  document.body.append(popover);

  const rect = anchor.getBoundingClientRect();
  const width = Math.min(236, Math.max(218, window.innerWidth - 24));
  const heightLimit = Math.min(326, window.innerHeight - 24);
  const left = Math.min(Math.max(12, rect.left), Math.max(12, window.innerWidth - width - 12));
  const preferredTop = rect.bottom + 6;
  const top = preferredTop + heightLimit <= window.innerHeight - 12
    ? preferredTop
    : Math.max(12, rect.top - heightLimit - 6);

  popover.style.position = "fixed";
  popover.style.left = `${Math.round(left)}px`;
  popover.style.top = `${Math.round(top)}px`;
  popover.style.width = `${Math.round(width)}px`;
  popover.style.maxHeight = `${Math.round(heightLimit)}px`;
  popover.style.zIndex = "2147483000";
}

/* KGW_CALENDAR_CLOSE_I18N_FIX_R12: i18n-safe calendar close label; no new calendar owner. */
function kgwAnalysisI18nText(key, fallback) {
  const api = globalThis.kgwI18n || globalThis.KGW_I18N || globalThis.i18n || null;
  const candidates = [
    api && typeof api.t === "function" ? api.t.bind(api) : null,
    typeof globalThis.t === "function" ? globalThis.t.bind(globalThis) : null
  ];

  for (const translate of candidates) {
    if (!translate) continue;
    try {
      const value = translate(key);
      if (typeof value === "string" && value.trim() && value !== key) return value;
    } catch {
      /* keep fallback */
    }
  }

  return fallback;
}

function kgwAnalysisOpenCalendar(textInput, nativeInput, textId) {
  const section = root();
  if (!section || !textInput) return;

  const host = textInput.closest(".kgw-analysis-date-field") || textInput.parentElement || section;
  const wasOpen = host.dataset.kgwCalendarOpen === "1";
  kgwAnalysisCloseCalendar("analysis");
  if (wasOpen) return;

  host.dataset.kgwCalendarOpen = "1";
  host.dataset.kgwCalendarScope = "analysis";

  let activeDate = kgwAnalysisParseIso(textInput.value);
  let displayYear = activeDate.getFullYear();
  let displayMonth = activeDate.getMonth();

  const popover = document.createElement("div");
  popover.className = "kgw-calendar-popover";
  popover.lang = "en-US";
  popover.dir = "ltr";
  popover.setAttribute("role", "dialog");
  popover.setAttribute("aria-label", "Date picker");

  function render() {
    popover.textContent = "";

    const header = document.createElement("div");
    header.className = "kgw-calendar-header";

    const prev = document.createElement("button");
    prev.type = "button";
    prev.className = "kgw-calendar-nav";
    prev.textContent = "‹";
    prev.setAttribute("aria-label", "Previous month");

    const title = document.createElement("div");
    title.className = "kgw-calendar-title";
    title.textContent = kgwAnalysisMonthLabel(displayYear, displayMonth);

    const next = document.createElement("button");
    next.type = "button";
    next.className = "kgw-calendar-nav";
    next.textContent = "›";
    next.setAttribute("aria-label", "Next month");

    prev.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-prev-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || ""),
        year: displayYear,
        month: displayMonth
      });
      displayMonth -= 1;
      if (displayMonth < 0) {
        displayMonth = 11;
        displayYear -= 1;
      }
      render();
    });

    next.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-next-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || ""),
        year: displayYear,
        month: displayMonth
      });
      displayMonth += 1;
      if (displayMonth > 11) {
        displayMonth = 0;
        displayYear += 1;
      }
      render();
    });

    header.append(prev, title, next);

    const presets = document.createElement("div");
    presets.className = "kgw-calendar-presets";

    [
      ["today", "Today"],
      ["last7", "Last 7 Days"],
      ["last30", "Last 30 Days"],
      ["thisMonth", "This Month"],
      ["sinceLaunch", "Since Launch"]
    ].forEach(([value, label]) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "kgw-calendar-preset";
      button.textContent = label;
      button.addEventListener("click", (event) => {
        event.preventDefault();
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-preset-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(textId || ""),
          preset: String(value || ""),
          label: String(label || "")
        });
        kgwAnalysisApplyPreset(textId, value);
      });
      presets.append(button);
    });

    const weekdays = document.createElement("div");
    weekdays.className = "kgw-calendar-weekdays";
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].forEach((day) => {
      const node = document.createElement("span");
      node.textContent = day;
      weekdays.append(node);
    });

    const grid = document.createElement("div");
    grid.className = "kgw-calendar-grid";

    const firstDay = new Date(displayYear, displayMonth, 1);
    const daysInMonth = new Date(displayYear, displayMonth + 1, 0).getDate();
    const offset = firstDay.getDay();

    for (let i = 0; i < offset; i++) {
      const empty = document.createElement("span");
      empty.className = "kgw-calendar-empty";
      grid.append(empty);
    }

    const selectedIso = kgwAnalysisCleanIso(textInput.value);
    const todayIso = kgwAnalysisTodayIso();

    for (let day = 1; day <= daysInMonth; day++) {
      const iso = kgwAnalysisIsoFromDate(new Date(displayYear, displayMonth, day));
      const button = document.createElement("button");
      button.type = "button";
      button.className = "kgw-calendar-day";
      button.textContent = String(day);
      button.dataset.iso = iso;

      if (iso === selectedIso) button.classList.add("is-selected");
      if (iso === todayIso) button.classList.add("is-today");

      button.addEventListener("click", (event) => {
        event.preventDefault();
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-day-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(textId || ""),
          iso: String(iso || "")
        });
        kgwAnalysisSetDate(textInput, nativeInput, iso);
        renderRows();
        kgwAnalysisCloseCalendar("analysis");
      });

      grid.append(button);
    }

    const footer = document.createElement("div");
    footer.className = "kgw-calendar-footer";

    const close = document.createElement("button");
    close.type = "button";
    close.className = "kgw-calendar-close";
    close.textContent = kgwAnalysisI18nText("calendar.close", "Close");
    close.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-close-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || "")
      });
      kgwAnalysisCloseCalendar("analysis");
    });

    footer.append(close);
    popover.append(header, presets, weekdays, grid, footer);
  }

  render();
  kgwAnalysisAttachPopover(popover, textInput);
}

function bindCalendar() {
  const pairs = [
    {
      textId: "analysisFromDate",
      nativeId: "analysisFromDateNative",
      role: "from"
    },
    {
      textId: "analysisToDate",
      nativeId: "analysisToDateNative",
      role: "to"
    }
  ];

  function resolveButton(textInput, textId, role, index) {
    const escapedTextId = kgwAnalysisEscapeSelector(textId);
    const host = textInput?.closest(".kgw-analysis-date-field") || textInput?.parentElement || root();

    return q(`[data-date-for="${escapedTextId}"]`)
      || q(`[data-native-for="${kgwAnalysisEscapeSelector(role === "from" ? "analysisFromDateNative" : "analysisToDateNative")}"]`)
      || host?.querySelector(".analysis-calendar-btn")
      || host?.querySelector("button")
      || qa(".analysis-calendar-btn")[index]
      || null;
  }

  function bindPair(pair, index) {
    const textInput = q(`#${kgwAnalysisEscapeSelector(pair.textId)}`);
    const nativeInput = q(`#${kgwAnalysisEscapeSelector(pair.nativeId)}`);
    if (!textInput) return;

    textInput.setAttribute("lang", "en-US");
    textInput.setAttribute("dir", "ltr");

    if (nativeInput) {
      nativeInput.setAttribute("lang", "en-US");
      nativeInput.setAttribute("dir", "ltr");
    }

    const button = resolveButton(textInput, pair.textId, pair.role, index);
    if (button && button.dataset.kgwCalendarBound !== "true") {
      button.dataset.kgwCalendarBound = "true";
      button.dataset.dateFor = pair.textId;
      button.dataset.nativeFor = pair.nativeId;
      button.classList.add("analysis-calendar-btn");
      button.setAttribute("lang", "en-US");
      button.setAttribute("dir", "ltr");
      button.setAttribute("type", "button");

      button.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-calendar-button-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          nativeId: String(pair.nativeId || ""),
          role: String(pair.role || "")
        });
        kgwAnalysisSetDate(textInput, nativeInput, textInput.value || kgwAnalysisTodayIso());
        kgwAnalysisOpenCalendar(textInput, nativeInput, pair.textId);
      });
    }

    if (nativeInput && nativeInput.dataset.kgwCalendarNativeBound !== "true") {
      nativeInput.dataset.kgwCalendarNativeBound = "true";
      nativeInput.addEventListener("change", (event) => {
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-native-date-change", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          nativeId: String(pair.nativeId || ""),
          value: String(nativeInput.value || "")
        });
        textInput.value = kgwAnalysisCleanIso(nativeInput.value, textInput.value);
        renderRows();
      });
    }

    if (textInput.dataset.kgwCalendarTextBound !== "true") {
      textInput.dataset.kgwCalendarTextBound = "true";
      textInput.addEventListener("input", (event) => {
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-text-date-input", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          value: String(textInput.value || "")
        });
        textInput.value = kgwAnalysisCleanIso(textInput.value, textInput.value);
      });
      textInput.addEventListener("change", (event) => {
        kgwAnalysisUiTraceR50B("analysis-calendar", "r50b-analysis-text-date-change", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(pair.textId || ""),
          value: String(textInput.value || "")
        });
        renderRows();
      });
    }
  }

  pairs.forEach(bindPair);
}
/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_ANALYSIS_OWNER_END */


/* KGW_EXPORT_PHASE1_FRONTEND_WIRING_V1G_START */
function kgwAnalysisInvokeExportV1G() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
}

function kgwAnalysisLocaleV1G() {
  return document.documentElement.getAttribute("lang") ||
    window.kgwCurrentLocale ||
    window.localStorage?.getItem?.("kgw.locale") ||
    "en";
}

function kgwAnalysisSetExportStatusV1G(message) {
  setText("#analysisStatus", message);
  log().log("analysis export", { message });
}

function kgwAnalysisSetExportButtonsEnabledV1G(enabled) {
  ["#analysisExportCsv", "#analysisExportHtml", "#analysisExportPdf"].forEach((selector) => {
    const button = q(selector);
    if (button) {
      button.disabled = !enabled;
      if (selector === "#analysisExportPdf") button.title = "";
    }
  });
}

// KGW_EXPORT_TEMPLATE_PARITY_ANALYSIS_URLS_V1B
function kgwAnalysisCleanExportIdV1B(value) {
  return String(value || "").trim().replace(/[\\s'"<>]+$/g, "");
}

function kgwAnalysisAddressUrlV1B(address) {
  const clean = kgwAnalysisCleanExportIdV1B(address);
  return clean.startsWith("kaspa:") ? `https://explorer.kaspa.org/addresses/${clean}` : "";
}

function kgwAnalysisTxUrlV1B(txid) {
  const clean = kgwAnalysisCleanExportIdV1B(txid);
  return /^[0-9a-f]{32,}$/i.test(clean) ? `https://explorer.kaspa.org/txs/${clean}` : "";
}

// KGW_EXPORT_RAW_PAYLOAD_PARITY_ANALYSIS_V2
function kgwAnalysisCleanExportIdV2(value) {
  return String(value || "").trim().replace(/[\\s'"<>]+$/g, "");
}

function kgwAnalysisAddressUrlV2(address) {
  const clean = kgwAnalysisCleanExportIdV2(address);
  return clean.startsWith("kaspa:") ? `https://explorer.kaspa.org/addresses/${clean}` : "";
}

function kgwAnalysisTxUrlV2(txid) {
  const clean = kgwAnalysisCleanExportIdV2(txid);
  return /^[0-9a-f]{32,}$/i.test(clean) ? `https://explorer.kaspa.org/txs/${clean}` : "";
}

function kgwAnalysisGroupRowsV2(rows) {
  const groups = new Map();

  for (const row of rows) {
    const key = row.address || row.counterparty || row.name || "Unknown Counterparty";
    if (!groups.has(key)) {
      groups.set(key, {
        key,
        name: row.name || "",
        address: row.address || row.counterparty || "",
        rows: [],
        totalKas: 0,
        totalUsd: 0
      });
    }

    const group = groups.get(key);
    const kas = Number(String(row.amount || row.netFlow || "0").replace(/,/g, ""));
    const usd = Number(String(row.valueUsd || "0").replace(/,/g, ""));

    if (Number.isFinite(kas)) group.totalKas += kas;
    if (Number.isFinite(usd)) group.totalUsd += usd;

    group.rows.push(row);
  }

  return Array.from(groups.values()).sort((a, b) =>
    String(a.name || a.address || a.key).localeCompare(String(b.name || b.address || b.key))
  );
}

function kgwAnalysisClientTableV1G() {
  applyFilter();

  const groups = kgwAnalysisGroupRowsV2(filteredRows);

  const headers = [
    "Counterparty",
    "Known Name",
    "Counterparty Address",
    "Counterparty URL",
    "Date/Time",
    "Transaction ID",
    "Transaction URL",
    "Direction",
    "Amount (KAS)",
    "Value (USD)",
    "Block Score",
    "Type"
  ];

  const rows = [];

  for (const group of groups) {
    rows.push([
      "Counterparty",
      group.name || "",
      group.address || group.key || "",
      kgwAnalysisAddressUrlV2(group.address || group.key || ""),
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      ""
    ]);

    for (const row of group.rows) {
      const address = row.address || row.counterparty || group.address || "";
      const txid = row.transactionId || row.txid || "";

      rows.push([
        "",
        row.name || group.name || "",
        address,
        kgwAnalysisAddressUrlV2(address),
        row.datetime || "",
        txid,
        kgwAnalysisTxUrlV2(txid),
        row.direction || row.txsDir || "",
        row.amount || row.netFlow || "",
        row.valueUsd || "",
        row.blockScore || "",
        row.type || ""
      ]);
    }

    rows.push([
      "Total",
      group.name || "",
      group.address || group.key || "",
      kgwAnalysisAddressUrlV2(group.address || group.key || ""),
      "",
      "",
      "",
      "",
      Number.isFinite(group.totalKas) ? String(group.totalKas) : "",
      Number.isFinite(group.totalUsd) ? String(group.totalUsd) : "",
      "",
      ""
    ]);

    rows.push(["", "", "", "", "", "", "", "", "", "", "", ""]);
  }

  if (!rows.length) throw new Error("No analysis rows are available for export.");

  return {
    title: "Kaspa Gateway Analysis Report",
    subtitle: `Counterparty groups: ${groups.length} | Rows: ${filteredRows.length}`,
    headers,
    rows
  };
}


/* KGW_EXPORT_NATIVE_SAVE_PHASE_A_V8_START */


function kgwAnalysisDialogApiV9() {
  const tauriKeys = Object.keys(window.__TAURI__ || {});
  const dialog = window.__TAURI__?.dialog;

  if (!dialog || typeof dialog.save !== "function" || typeof dialog.ask !== "function") {
    throw new Error(
      "Tauri global dialog API is not available. Expected window.__TAURI__.dialog.save/ask. Available window.__TAURI__ keys: " +
      tauriKeys.join(",")
    );
  }

  return dialog;
}


async function kgwAnalysisLoadNativeDialogV8() {
  return kgwAnalysisDialogApiV9();
}

function kgwAnalysisNativeDialogFilterV8(format) {
  const ext = String(format || "").replace(/^\./, "").toLowerCase();
  return {
    name: ext ? `${ext.toUpperCase()} files` : "Export files",
    extensions: ext ? [ext] : []
  };
}

async function kgwAnalysisNativeSavePathV8(format, defaultPath) {
  const dialog = await kgwAnalysisLoadNativeDialogV8();
  const selected = await dialog.save({
    title: "Save export",
    defaultPath,
    filters: [kgwAnalysisNativeDialogFilterV8(format)]
  });
  return selected ? String(selected) : null;
}


/* KGW_EXPORT_CENTERED_OPEN_PROMPT_V10_START */
function kgwExportCenteredOpenPromptTextV10() {
  const lang = String(document.documentElement?.lang || localStorage.getItem("kgw.language") || "en").toLowerCase();

  if (lang.startsWith("ar")) {
    return {
      title: "تم الحفظ",
      message: "تم حفظ الملف بنجاح. هل تريد فتحه الآن؟",
      open: "فتح",
      cancel: "إلغاء",
      dir: "rtl"
    };
  }

  if (lang.startsWith("de")) {
    return {
      title: "Gespeichert",
      message: "Die Datei wurde gespeichert. Möchten Sie sie jetzt öffnen?",
      open: "Öffnen",
      cancel: "Abbrechen",
      dir: "ltr"
    };
  }

  if (lang.startsWith("es")) {
    return {
      title: "Guardado",
      message: "El archivo se ha guardado. ¿Quieres abrirlo ahora?",
      open: "Abrir",
      cancel: "Cancelar",
      dir: "ltr"
    };
  }

  if (lang.startsWith("fr")) {
    return {
      title: "Enregistré",
      message: "Le fichier a été enregistré. Voulez-vous l’ouvrir maintenant ?",
      open: "Ouvrir",
      cancel: "Annuler",
      dir: "ltr"
    };
  }

  return {
    title: "Saved",
    message: "The file was saved successfully. Do you want to open it now?",
    open: "Open",
    cancel: "Cancel",
    dir: "ltr"
  };
}

function kgwExportEnsureCenteredOpenPromptStyleV10() {
  const styleId = "kgw-export-centered-open-prompt-v10-style";
  if (document.getElementById(styleId)) return;

  const style = document.createElement("style");
  style.id = styleId;
  style.textContent = `
    .kgw-export-open-prompt-v10-backdrop {
      position: fixed;
      inset: 0;
      z-index: 2147483000;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 24px;
      background: rgba(0, 0, 0, 0.28);
      box-sizing: border-box;
    }

    .kgw-export-open-prompt-v10-card {
      width: min(440px, calc(100vw - 48px));
      min-height: 132px;
      border-radius: 14px;
      border: 1px solid rgba(124, 171, 255, 0.28);
      background: #101827;
      color: #f8fbff;
      box-shadow: 0 22px 70px rgba(0, 0, 0, 0.45);
      overflow: hidden;
      font-family: inherit;
    }

    .kgw-export-open-prompt-v10-card[dir="rtl"] {
      text-align: right;
    }

    .kgw-export-open-prompt-v10-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 15px 18px 10px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.08);
      font-weight: 800;
      letter-spacing: 0.01em;
    }

    .kgw-export-open-prompt-v10-close {
      width: 30px;
      height: 30px;
      border: 0;
      border-radius: 10px;
      background: transparent;
      color: #d7e7ff;
      font-size: 22px;
      line-height: 1;
      cursor: pointer;
    }

    .kgw-export-open-prompt-v10-close:hover {
      background: rgba(255, 255, 255, 0.10);
    }

    .kgw-export-open-prompt-v10-body {
      padding: 18px;
      color: #e7eefb;
      font-size: 14px;
      line-height: 1.6;
    }

    .kgw-export-open-prompt-v10-actions {
      display: flex;
      justify-content: flex-end;
      gap: 10px;
      padding: 0 18px 18px;
    }

    .kgw-export-open-prompt-v10-card[dir="rtl"] .kgw-export-open-prompt-v10-actions {
      justify-content: flex-start;
    }

    .kgw-export-open-prompt-v10-button {
      min-width: 94px;
      height: 34px;
      border-radius: 10px;
      border: 1px solid rgba(124, 171, 255, 0.26);
      background: rgba(255, 255, 255, 0.08);
      color: #f8fbff;
      font-weight: 700;
      cursor: pointer;
    }

    .kgw-export-open-prompt-v10-button:hover {
      background: rgba(255, 255, 255, 0.13);
    }

    .kgw-export-open-prompt-v10-primary {
      border-color: rgba(112, 180, 255, 0.65);
      background: #4f88d9;
      color: #ffffff;
    }

    .kgw-export-open-prompt-v10-primary:hover {
      background: #5b96ee;
    }
  `;

  document.head.appendChild(style);
}

function kgwExportCenteredOpenPromptV10() {
  kgwExportEnsureCenteredOpenPromptStyleV10();

  const labels = kgwExportCenteredOpenPromptTextV10();

  return new Promise((resolve) => {
    const previousActive = document.activeElement instanceof HTMLElement ? document.activeElement : null;

    const backdrop = document.createElement("div");
    backdrop.className = "kgw-export-open-prompt-v10-backdrop";
    backdrop.setAttribute("role", "presentation");

    const card = document.createElement("div");
    card.className = "kgw-export-open-prompt-v10-card";
    card.setAttribute("role", "dialog");
    card.setAttribute("aria-modal", "true");
    card.setAttribute("aria-labelledby", "kgw-export-open-prompt-v10-title");
    card.setAttribute("dir", labels.dir);

    const header = document.createElement("div");
    header.className = "kgw-export-open-prompt-v10-header";

    const title = document.createElement("div");
    title.id = "kgw-export-open-prompt-v10-title";
    title.textContent = labels.title;

    const close = document.createElement("button");
    close.type = "button";
    close.className = "kgw-export-open-prompt-v10-close";
    close.setAttribute("aria-label", labels.cancel);
    close.textContent = "×";

    const body = document.createElement("div");
    body.className = "kgw-export-open-prompt-v10-body";
    body.textContent = labels.message;

    const actions = document.createElement("div");
    actions.className = "kgw-export-open-prompt-v10-actions";

    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "kgw-export-open-prompt-v10-button";
    cancel.textContent = labels.cancel;

    const open = document.createElement("button");
    open.type = "button";
    open.className = "kgw-export-open-prompt-v10-button kgw-export-open-prompt-v10-primary";
    open.textContent = labels.open;

    let resolved = false;

    function cleanup(value) {
      if (resolved) return;
      resolved = true;
      document.removeEventListener("keydown", onKeyDown, true);
      backdrop.remove();
      if (previousActive && typeof previousActive.focus === "function") {
        try { previousActive.focus(); } catch (_) {}
      }
      resolve(Boolean(value));
    }

    function onKeyDown(event) {
      if (event.key === "Escape") {
        event.preventDefault();
        cleanup(false);
        return;
      }

      if (event.key === "Enter") {
        event.preventDefault();
        cleanup(true);
      }
    }

    close.addEventListener("click", () => cleanup(false));
    cancel.addEventListener("click", () => cleanup(false));
    open.addEventListener("click", () => cleanup(true));

    backdrop.addEventListener("mousedown", (event) => {
      if (event.target === backdrop) cleanup(false);
    });

    header.append(title, close);
    actions.append(cancel, open);
    card.append(header, body, actions);
    backdrop.append(card);
    document.body.appendChild(backdrop);

    document.addEventListener("keydown", onKeyDown, true);
    setTimeout(() => open.focus(), 0);
  });
}
/* KGW_EXPORT_CENTERED_OPEN_PROMPT_V10_END */


async function kgwAnalysisNativeAskOpenV8(finalPath) {
  const accepted = await kgwExportCenteredOpenPromptV10();
  if (!accepted) return false;

  const call = kgwAnalysisInvokeExportV1G();
  if (!call) throw new Error("Tauri invoke API is not available.");

  await call("kgw_open_exported_file_v1", { path: finalPath });
  return true;
}

/* KGW_EXPORT_NATIVE_SAVE_PHASE_A_V8_END */


async function kgwAnalysisExportBackendV1G(format) {
  const call = kgwAnalysisInvokeExportV1G();
  if (!call) throw new Error("Tauri invoke API is not available.");

  const outputPath = await call("export_default_path", {
    reportType: "Analysis",
    format
  });

  const selectedOutputPath = await kgwAnalysisNativeSavePathV8(format, outputPath);
  if (!selectedOutputPath) {
    kgwAnalysisSetExportStatusV1G("Export cancelled.");
    return;
  }

  const address = String(q("#analysisAddressSelect")?.value || q("#analysisAddressInput")?.value || "").trim();

  const result = await call("export_report", {
    request: {
      reportType: "Analysis",
      format,
      outputPath: selectedOutputPath,
      addressFilter: address || null,
      timeRange: String(q("#analysisTimeRange")?.value || "all"),
      limit: 100000,
      locale: kgwAnalysisLocaleV1G(),
      clientTable: kgwAnalysisClientTableV1G()
    }
  });

  const finalPath = result.output_path || result.outputPath || selectedOutputPath;
  kgwAnalysisSetExportStatusV1G(`Export completed: ${finalPath}`);
  await kgwAnalysisNativeAskOpenV8(finalPath);
}

function kgwAnalysisBindExportButtonsV1G() {
  const bindings = [
    ["#analysisExportCsv", "csv"],
    ["#analysisExportHtml", "html"],
    ["#analysisExportPdf", "pdf"]
  ];

  for (const [selector, format] of bindings) {
    const button = q(selector);
    if (!button || button.dataset.kgwExportPhase1 === "true") continue;

    button.dataset.kgwExportPhase1 = "true";
    button.addEventListener("click", (event) => {
      event.preventDefault();
      kgwAnalysisUiTraceR50B("analysis-export", "r50b-analysis-export-click", {
        trusted: Boolean(event && event.isTrusted),
        format: String(format || ""),
        selector: String(selector || ""),
        id: String(button.id || ""),
        text: String(button.textContent || "").trim()
      });
      kgwAnalysisExportBackendV1G(format).catch((error) => {
        kgwAnalysisSetExportStatusV1G(error?.message || String(error));
      });
    });
  }
}
/* KGW_EXPORT_PHASE1_FRONTEND_WIRING_V1G_END */


function bindControls() {
  kgwAnalysisBindExportButtonsV1G();
  kgwAnalysisPythonBindR9();
  // KGW_ANALYSIS_FILTER_CALENDAR_PATCH_R18
  kgwAnalysisNormalizeFilterLayoutR18();

  const search = q("#analysisSearch");
  const type = q("#analysisType");
  const direction = q("#analysisDirection");
  const filter = q("#analysisFilter");
  const reset = q("#analysisResetFilter");

  [search, type, direction].forEach((node) => {
    if (!node || node.dataset.bound === "true") return;
    node.dataset.bound = "true";

    const eventName = node.tagName === "INPUT" ? "input" : "change";
    node.addEventListener(eventName, (event) => {
      kgwAnalysisUiTraceR50B("analysis-filter", "r50b-analysis-filter-change", {
        trusted: Boolean(event && event.isTrusted),
        element: "node",
        id: String(node.id || ""),
        tag: String(node.tagName || ""),
        eventName: String(eventName || ""),
        value: String(node.value || "")
      });
      renderRows();
    });
  });

  if (filter && filter.dataset.bound !== "true") {
    filter.dataset.bound = "true";
    filter.addEventListener("click", (event) => {
      kgwAnalysisUiTraceR50B("analysis-filter", "r50b-analysis-filter-click", {
        trusted: Boolean(event && event.isTrusted),
        id: String(filter.id || ""),
        text: String(filter.textContent || "").trim()
      });
      renderRows();
    });
  }

  if (reset && reset.dataset.bound !== "true") {
    reset.dataset.bound = "true";
    reset.addEventListener("click", (event) => {
      kgwAnalysisUiTraceR50B("analysis-filter", "r50b-analysis-reset-filter-click", {
        trusted: Boolean(event && event.isTrusted),
        id: String(reset.id || ""),
        text: String(reset.textContent || "").trim()
      });
      if (search) search.value = "";
      if (type) type.value = "ALL";
      if (direction) direction.value = "ALL";
      // KGW_ANALYSIS_FILTER_CALENDAR_PATCH_R18
      kgwAnalysisEnsureTodayToDateR18(true);
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
  // KGW_ANALYSIS_FILTER_CALENDAR_PATCH_R18
  kgwAnalysisNormalizeFilterLayoutR18();
  kgwAnalysisBindDateTouchR18();
  kgwAnalysisEnsureTodayToDateR18(false);
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
