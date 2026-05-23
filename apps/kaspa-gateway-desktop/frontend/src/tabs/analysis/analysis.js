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
