/*
 * KGW_UI_CLEANUP_NO_BEHAVIOR_CHANGE
 * This file has been cleaned only with behavior-preserving UI hygiene:
 * - trailing whitespace removal
 * - excessive blank-line collapse
 * - ownership/reporting guard
 * No function names, command names, event flow, runtime calls, or rendering logic were intentionally changed.
 */
/*
 * KGW_OWNERSHIP_EXPLORER_FRONTEND_UI_ONLY
 * Explorer frontend owns UI state, rendering, DOM events, and Tauri command calls only.
 * Forbidden: direct transaction HTTP API calls, API pagination ownership,
 * DB persistence orchestration, and runtime fetch/sync ownership.
 */

/*
 * KGW_PHASE7A2_EXPLORER_OWNERSHIP_BLOCKERS_FIXED
 *
 * Ownership cleanup:
 * - Explorer UI must not call external HTTP endpoints directly.
 * - Explorer UI must not own API paging defaults such as runtime paging defaults.
 * - Explorer UI may request data through Tauri commands only.
 */
/*
 * KGW_PHASE7A1_EXPLORER_JS_UI_ONLY_REFINED
 *
 * Explorer frontend ownership rule:
 * - This file owns UI state, rendering, and Tauri command calls only.
 * - It must not implement DB writes, HTTP endpoint construction, direct API paging,
 *   or transaction persistence orchestration.
 * - Transaction fetch/sync orchestration belongs to Rust runtime transaction_sync.rs.
 */

import { openBlockExplorer, exportCsv, exportHtml, exportPdf, kgwExplorerExportNumberV2, kgwExplorerExportNormalizeRawTxV2 } from "./explorer.export.js";
import { parseDateSeconds, kgwDayToEpochSeconds, kgwClean2DayToSeconds } from "./explorer.date.js";
import { kgwSummaryFormatKas, kgwSummaryFormatUsd, kgwClean2Kas, kgwClean2Usd } from "./explorer.formatting.js";
import { kgwClean2SafeText, kgwClean2UsdPrice, kgwSummaryUsdForKas, kgwNormalizeDaySummaries, kgwClean2NormalizeSummaries, kgwSummaryUsdForSummary, kgwLiveCoreAddress, kgwLiveCoreReset, kgwLiveCoreSeedRows, kgwLiveCoreMergeRecords, kgwLiveCoreMergeDays, kgwLiveCoreRows, kgwLiveCoreShouldRender, kgwNormalizeUnifiedResult, kgwDaySummaryRowsFromResult, kgwForceSetTableMessage, kgwForceResetDisplayFiltersToAll, kgwForceSetControlsBusy, kgwInstallForceBusyBlocker, defaultDates, kgwBindFontSpinbox, setTableFontSize, kgwApplyExplorerLocalBusyControls, kgwBuildExplorerListRequest, kgwNormalizeTxTypeFilterValue, kgwNormalizeDirectionFilterValue, kgwClean2Request, kgwExplorerSaveManualAddress, kgwInstallExplorerManualAddressSave, normalizeAddress, isKaspaAddress, kgwCanonicalKaspaAddress, loadKnownAddressNames, saveAddressToDatabase, refreshAddressName, loadSavedAddresses, fetchBalance, explorerAddressDiagnosticsSnapshot } from "./explorer.utils.js";

const explorerState = {
  rows: [],
  filteredRows: [],
  selectedAddress: "",
  busy: false,
  cancelRequested: false
};

function root() {
  return document.getElementById("explorer") || document.querySelector(".explorer-python-root") || document;
}

function qs(selector, scope = root()) {
  return scope.querySelector(selector);
}

function qsa(selector, scope = root()) {
  return Array.from(scope.querySelectorAll(selector));
}

function invokeApi() {
  return (
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_INVOKE__
  );
}


/* KGW_PHASE3C_CANONICAL_EXPLORER_COMMAND_WRAPPERS */
async function kgwInvokeExplorerUnifiedFetch(request) {
  return await invokeCommand("explorer_transactions", { request });
}

async function kgwInvokeExplorerCancelTransactionsR57D4(requestId) {
  return await invokeCommand("explorer_cancel_transactions", { requestId });
}

async function kgwInvokeExplorerGroupedTransactions(request) {
  return await invokeCommand("explorer_list_transactions_grouped_rust", { request });
}

async function kgwInvokeExplorerDaySummaries(request) {
  return await invokeCommand("explorer_transaction_day_summaries_rust", { request });
}

async function invokeCommand(command, args = {}) {
  const invoke = invokeApi();

  if (!invoke) {
    throw new Error("Tauri invoke API is not available.");
  }

  return await invoke(command, args);
}


function setStatus(section, message, state = "info") {
  const cleanMessage = String(message || "");

  const node = qs("#explorerStatus", section);
  if (node) {
    node.hidden = false;
    node.textContent = cleanMessage;
    node.dataset.state = state;
    node.setAttribute("role", state === "error" ? "alert" : "status");
    node.setAttribute("aria-live", state === "error" ? "assertive" : "polite");
  }

  if (typeof window.kgwSetGlobalFetchProgressText === "function") {
    window.kgwSetGlobalFetchProgressText(cleanMessage);
  }

  console.info("[KGW Explorer]", message);
}


/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_EXPLORER_OWNER_START */
/* KGW_CALENDAR_EXISTING_OWNER_REGEX_FIX_R4: fixed regex escaping only; no new owner layer.\n * KGW_CALENDAR_EXISTING_OWNER_DOM_CSS_FIX_R6: body-attached compact popover inside existing owner.\n * KGW_CALENDAR_SCOPED_POPOVER_OWNER_FIX_R7: scope-isolated popovers per tab.\n * KGW_CALENDAR_SINGLE_ACTIVE_POPOVER_FIX_R8: removes stale body-attached popovers before opening current tab calendar.
/* */
/* Explorer calendar/date-control ownership lives in Rust/WASM explorer_calendar.rs. */

/* Explorer local busy-control ownership lives in Rust/WASM explorer_controls.rs. */

function syncActionState(section) {
  const busy = explorerState.busy;
  kgwApplyExplorerLocalBusyControls(section, busy); // KGW_LOCAL_BUSY_CONTROLS

  qs("#explorerFetch", section).disabled = busy;
  qs("#explorerForceFetch", section).disabled = busy;
  qs("#explorerCancel", section).disabled = !busy;

  const hasRows = explorerState.filteredRows.length > 0;

  microscopeLog("SYNC ACTION STATE", {
    busy,
    rows: explorerState.rows?.length || 0,
    filteredRows: explorerState.filteredRows?.length || 0
  });

  for (const selector of ["#explorerExportCsv", "#explorerExportHtml", "#explorerExportPdf"]) {
    const node = qs(selector, section);
    if (node) node.disabled = !hasRows;
  }
}

function extractRowsFromUnifiedResult(result) {
  microscopeApiShape("UNIFIED RESULT RAW SHAPE", result);
  const rows = kgwNormalizeUnifiedResult(result);
  microscopeLog("UNIFIED RESULT NORMALIZED ROWS", {
    rows: rows.length,
    sample: rows.slice(0, 3)
  });
  return rows;
}


/* KGW_TX_R4_GROUPED_COLLAPSED_RENDERER
   Python parity:
   - Group transactions by date.
   - Groups are collapsed by default with +.
   - Clicking the date row toggles + / -.
*/
if (!window.__kgwExplorerExpandedDateGroups) {
  window.__kgwExplorerExpandedDateGroups = new Set();
}








/* KGW_TX_UI_FAST_3B_DAY_SUMMARY_MODE
   Fast path:
   - Load only day summaries from local database.
   - Do not move 38k+ transactions into JS for collapsed view.
   - Fetch one day's transactions only when the user expands that day.
*/
if (!window.__kgwExplorerDayTransactionCache) {
  window.__kgwExplorerDayTransactionCache = new Map();
}

/* KGW_EXPLORER_SAFE_CONTROLS_TRACE_PATCH_R53B3
   Existing Explorer UI trace owner.
   Scope: Explorer fetch, force fetch, filters, address, date/calendar, grouped rows, and safe export button activity.
*/
function kgwExplorerUiTraceR53B3(action, phase, details = {}) {
  try {
    const call = invokeApi();
    if (typeof call !== "function") return Promise.resolve(false);

    return Promise.resolve(call("kgw_frontend_button_trace_v1", {
      scope: "explorer",
      net: "ui",
      action: String(action || "explorer-ui"),
      phase: String(phase || "unknown"),
      details: JSON.stringify({
        patch: "KGW_EXPLORER_SAFE_CONTROLS_TRACE_PATCH_R53B3",
        owner: "explorer-existing-safe-controls-owner",
        action: String(action || "explorer-ui"),
        phase: String(phase || "unknown"),
        details: details && typeof details === "object" ? details : {}
      })
    })).catch(function () {});
  } catch (_) {
    return Promise.resolve(false);
  }
}

/* KGW_FILTER_TRACE_1
   Console tracing for Explorer filters.
   This proves which filter values are sent to Rust and what comes back.
*/
function kgwFilterTrace(label, payload = {}) {
  try {
    console.log(`[KGW Explorer][filter] ${label}`, payload);
  } catch (_) {
    console.log(`[KGW Explorer][filter] ${label}`);
  }
}


/* KGW_FIX_FILTER_DROPDOWN_OPTIONS
   Canonical Explorer filters:
   Type      => ALL / coinbase / transfer
   Direction => ALL / incoming / outgoing
   Address search belongs in #explorerSearch, not in dropdowns.
*/
/* Explorer filter option/state/request ownership lives in Rust/WASM explorer_filters.rs. */

async function kgwLoadTransactionDaySummariesFromDb(section, address, startTs, endTs) {
  const request = kgwExplorerListRequest(section, address, startTs, endTs);
  request.limit = 10000;

  const result = await kgwInvokeExplorerDaySummaries(request);
  return kgwDaySummaryRowsFromResult(result);
}

async function kgwLoadTransactionsForSingleDayFromDb(section, address, day) {
  const startTs = kgwDayToEpochSeconds(day, false);
  const endTs = kgwDayToEpochSeconds(day, true);

  if (!Number.isFinite(startTs) || !Number.isFinite(endTs)) {
    return [];
  }

  const request = kgwExplorerListRequest(section, address, startTs, endTs);
  request.limit = 1000000;

  const groups = await kgwInvokeExplorerGroupedTransactions(request);
  return extractRowsFromUnifiedResult({ groups });
}




/* KGW_TX_UI_CLEAN_1_DAY_SUMMARY_RENDERER
   Clean Explorer transaction rendering:
   - collapsed table renders day summaries only
   - expanding a day loads only that day's transactions
   - no full 38k-row render in collapsed view
*/
if (!window.__kgwExplorerDayTransactionCache) {
  window.__kgwExplorerDayTransactionCache = new Map();
}

if (!window.__kgwExplorerExpandedDateGroups) {
  window.__kgwExplorerExpandedDateGroups = new Set();
}

/* KGW_FIX_DAY_SUMMARY_USD_TOTALS
   Collapsed day rows must show the total USD value for the day.
   The DB summary has KAS totals; USD is calculated from the current header price.
*/


async function kgwLoadTransactionsForDay(section, address, day) {
  const startTs = kgwDayToEpochSeconds(day, false);
  const endTs = kgwDayToEpochSeconds(day, true);

  if (!Number.isFinite(startTs) || !Number.isFinite(endTs)) {
    kgwFilterTrace("day load invalid range", { day, startTs, endTs });
    return [];
  }

  const request = kgwBuildExplorerListRequest(section, address, startTs, endTs, 1000000);

  kgwFilterTrace("day transactions request", {
    day,
    request
  });

  const started = performance.now();

  const groups = await kgwInvokeExplorerGroupedTransactions(request);
  const rows = extractRowsFromUnifiedResult({ groups });

  kgwFilterTrace("day transactions response", {
    day,
    elapsedMs: Math.round(performance.now() - started),
    groups: Array.isArray(groups) ? groups.length : null,
    rows: rows.length,
    sample: rows.slice(0, 3).map((row) => ({
      txid: row.txid || row.transaction_id || row.transactionId,
      direction: row.direction,
      tx_type: row.tx_type || row.type,
      amount: row.amount_kas ?? row.amountKas ?? row.amount,
      value: row.value_usd ?? row.valueUsd ?? row.value
    }))
  });

  return rows;
}


/* KGW_EXPLORER_USD_VALUE_RUNTIME_PRICE_REPAIR_V1 */
function kgwInstallExplorerPriceRerenderV1() {
  if (window.__kgwExplorerPriceRerenderV1Installed) return;
  window.__kgwExplorerPriceRerenderV1Installed = true;

  window.addEventListener("kgw:kaspa-price-updated", () => {
    try {
      const section = root();
      const rows = Array.isArray(explorerState?.rows) ? explorerState.rows : [];

      if (!section || !rows.some((row) => row?.__kgwDaySummary)) return;

      kgwRenderDaySummaries(section, rows).catch((error) => {
        console.warn("[KGW Explorer] price rerender failed", error);
      });
    } catch (error) {
      console.warn("[KGW Explorer] price rerender failed", error);
    }
  });
}

kgwInstallExplorerPriceRerenderV1();


async function kgwRenderDaySummaries(section, rows, statusText = "") {
  const body = qs("#explorerTransactionsBody", section);

  if (!body) {
    console.error("[KGW Explorer][clean] #explorerTransactionsBody not found");
    return;
  }

  const summaries = kgwNormalizeDaySummaries(rows);

  explorerState.rows = summaries;
  explorerState.filteredRows = summaries.slice();

  body.innerHTML = "";

  if (!summaries.length) {
    body.innerHTML = '<tr><td colspan="6" class="muted" data-i18n="explorer.noTransactionsToDisplay">No transactions to display.</td></tr>';
    syncActionState(section);

    if (statusText) {
      setStatus(section, statusText);
    }

    console.warn("[KGW Explorer][clean] no day summaries to render", { rows });
    return;
  }

  let totalCount = 0;
  let totalNetKas = 0;
  let totalValueUsd = 0;

  for (const summary of summaries) {
    totalCount += Number(summary.count || 0) || 0;
    totalNetKas += Number(summary.net_kas || 0) || 0;
    totalValueUsd += kgwSummaryUsdForSummary(summary);
  }

  const fragment = document.createDocumentFragment();

  const totalTr = document.createElement("tr");
  totalTr.className = "kgw-total-row";
  totalTr.innerHTML =
    "<td data-i18n=\"explorer.total\">Total</td>" +
    `<td>${totalCount.toLocaleString()} transactions</td>` +
    "<td></td>" +
    `<td>${kgwSummaryFormatKas(totalNetKas)}</td>` +
    `<td>${kgwSummaryFormatUsd(totalValueUsd)}</td>` +
    "<td></td>";
  fragment.appendChild(totalTr);

  for (const summary of summaries) {
    const day = summary.day;
    const expanded = window.__kgwExplorerExpandedDateGroups.has(day);
    const sign = expanded ? "-" : "+";

    const dayTr = document.createElement("tr");
    dayTr.className = "kgw-day-group-row";
    dayTr.dataset.kgwDateGroup = day;
    dayTr.innerHTML =
      `<td>${sign} ${kgwClean2SafeText(day)}</td>` +
      `<td>${Number(summary.count || 0).toLocaleString()} transactions</td>` +
      "<td></td>" +
      `<td>${kgwSummaryFormatKas(summary.net_kas)}</td>` +
      `<td>${kgwSummaryFormatUsd(kgwSummaryUsdForSummary(summary))}</td>` +
      "<td></td>";

    fragment.appendChild(dayTr);

    if (!expanded) {
      continue;
    }

    const cachedRows = window.__kgwExplorerDayTransactionCache.get(day);

    if (!cachedRows) {
      const loadingTr = document.createElement("tr");
      loadingTr.className = "kgw-transaction-row";
      loadingTr.innerHTML = `<td colspan="6" class="muted">Loading transactions for ${kgwClean2SafeText(day)}...</td>`;
      fragment.appendChild(loadingTr);
      continue;
    }

    for (const tx of cachedRows) {
      const txid = tx?.txid || tx?.transaction_id || tx?.transactionId || "";
      const amount = Number(tx?.amount_kas ?? tx?.amountKas ?? tx?.amount ?? 0) || 0;
      const usd = Number(
        tx?.value_usd ??
        tx?.valueUsd ??
        tx?.usd_value ??
        tx?.usdValue ??
        tx?.value ??
        kgwSummaryUsdForKas(amount)
      ) || 0;

      const txTr = document.createElement("tr");
      txTr.className = "kgw-transaction-row";
      txTr.dataset.kgwParentDateGroup = day;
      txTr.innerHTML =
        `<td>${kgwClean2SafeText(tx?.datetime || tx?.date_time || tx?.time || tx?.timestamp || "")}</td>` +
        `<td>${kgwClean2SafeText(txid)}</td>` +
        `<td>${kgwClean2SafeText(tx?.direction || "")}</td>` +
        `<td>${kgwSummaryFormatKas(amount)}</td>` +
        `<td>${kgwSummaryFormatUsd(usd)}</td>` +
        `<td>${kgwClean2SafeText(tx?.tx_type || tx?.type || "")}</td>`;

      fragment.appendChild(txTr);
    }
  }

  body.appendChild(fragment);

  body.querySelectorAll("[data-kgw-date-group]").forEach((row) => {
    row.addEventListener("click", async () => {
      const day = row.dataset.kgwDateGroup;

      if (!day) {
        return;
      }

      if (window.__kgwExplorerExpandedDateGroups.has(day)) {
        window.__kgwExplorerExpandedDateGroups.delete(day);
        await kgwRenderDaySummaries(section, explorerState.rows || [], statusText);
        return;
      }

      window.__kgwExplorerExpandedDateGroups.add(day);
      await kgwRenderDaySummaries(section, explorerState.rows || [], statusText);

      if (!window.__kgwExplorerDayTransactionCache.has(day)) {
        try {
          const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", section)?.value);
          const dayRows = await kgwLoadTransactionsForDay(section, address, day);
          window.__kgwExplorerDayTransactionCache.set(day, Array.isArray(dayRows) ? dayRows : []);
        } catch (error) {
          console.error("[KGW Explorer][clean] failed to load day transactions", day, error);
          window.__kgwExplorerDayTransactionCache.set(day, []);
        }

        await kgwRenderDaySummaries(section, explorerState.rows || [], statusText);
      }
    });
  });

  syncActionState(section);

  if (statusText) {
    setStatus(section, statusText);
  }

  console.log("[KGW Explorer][clean] day summaries rendered", {
    days: summaries.length,
    totalCount,
    totalValueUsd
  });
}


/* KGW_FILTER_SINGLE_OWNER_RENDER
   Filter must not route through old client-side row filtering.
   Rust already returns matching day summaries. Render those directly.
*/


/* Explorer single-owner filter value/request construction lives in Rust/WASM explorer_filters.rs. */










/* KGW_DB_SOURCE_OF_TRUTH_TABLE
   Python parity rule:
   The table is rendered from Transactions DB, not from raw network results.
*/








/* KGW_CLEAR_EXPLORER_TABLE_ON_CONTEXT_CHANGE */
/* KGW_EXPLORER_CANCEL_STATE_OWNER_FIX_R56E
   Existing owner fix:
   clearExplorerTransactionTable is used by normal context changes and by Cancel.
   Normal context changes reset cancelRequested.
   Cancel preserves cancelRequested so fetchTransactions/finally can observe it. */
function clearExplorerTransactionTable(section, reason = "cleared", options = {}) {
  const preserveCancelRequested = Boolean(options && options.preserveCancelRequested);

  explorerState.rows = [];
  explorerState.filteredRows = [];

  if (!preserveCancelRequested) {
    explorerState.cancelRequested = false;
  }

  renderTable(section);
  syncActionState(section);

  if (reason) {
    setStatus(section, reason);
  }
}


/* KGW_DB_LOAD_AFTER_FETCH_ONLY
   database rule:
   Do not poll/read Transactions DB while backend is writing.
   Read DB only after fetch command completes, or from Apply Filter when not busy.
*/
function kgwExplorerListRequest(section, address, startTs, endTs) {
  return {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: qs("#explorerTypeFilter", section)?.value || "ALL",
    direction: qs("#explorerDirectionFilter", section)?.value || "ALL",
    search_query: qs("#explorerSearch", section)?.value || "",
    limit: 1000000
  };
}





/* KGW_TX_R2_LIVE_DB_POLLING
   Python parity:
   While backend fetch is still running, poll local DB and render rows in batches.
   This prevents the table from looking frozen until all pages finish.
*/



/* KGW_TX_R3_LIVE_LOCAL_DATABASE_RENDER
   Python parity:
   While backend fetch is still running, read saved local database transactions and render them.
   The backend keeps fetching pages; the UI does not wait for all pages to finish.
*/



/* KGW_TX_UI_FAST_3D_DIRECT_DAY_SUMMARY_RENDERER
   Render day summaries directly. Do not pass 38k transactions through the generic renderer.
   Expanded day rows are loaded on demand from local database.
*/
async function kgwRenderDaySummariesDirect(section, summaries, statusText = "") {
  const body = qs("#explorerTransactionsBody", section);
  if (!body) return;

  const safeSummaries = Array.isArray(summaries)
    ? summaries.filter((item) => item && (item.__kgwDaySummary === true || item.day || item.date))
    : [];

  explorerState.rows = safeSummaries;
  explorerState.filteredRows = safeSummaries.slice();

  body.innerHTML = "";

  if (!safeSummaries.length) {
    body.innerHTML = `
      <tr>
        <td colspan="6" class="muted" data-i18n="explorer.noTransactionsToDisplay">No transactions to display.</td>
      </tr>
    `;

    syncActionState(section);

    if (statusText) {
      setStatus(section, statusText);
    }

    microscopeWarn("DIRECT DAY SUMMARY RENDER EMPTY", {
      summariesType: typeof summaries,
      isArray: Array.isArray(summaries)
    });

    return;
  }

  const totalCount = safeSummaries.reduce((sum, row) => {
    return sum + (Number(row.count || row.tx_count || row.transactions_count || 0) || 0);
  }, 0);

  const totalNetKas = safeSummaries.reduce((sum, row) => {
    return sum + (Number(row.net_kas ?? row.netKas ?? 0) || 0);
  }, 0);

  const fragment = document.createDocumentFragment();

  const totalTr = document.createElement("tr");
  totalTr.className = "kgw-total-row";
  totalTr.innerHTML = `
    <td data-i18n="explorer.total">Total</td>
    <td>${totalCount.toLocaleString()} transactions</td>
    <td></td>
    <td>${kgwSummaryFormatKas(totalNetKas)}</td>
    <td></td>
    <td></td>
  `;
  fragment.appendChild(totalTr);

  for (const summary of safeSummaries) {
    const day = summary.day || summary.date || "";
    if (!day) continue;

    const count = Number(summary.count || summary.tx_count || summary.transactions_count || 0) || 0;
    const netKas = Number(summary.net_kas ?? summary.netKas ?? 0) || 0;
    const expanded = window.__kgwExplorerExpandedDateGroups.has(day);
    const sign = expanded ? "-" : "+";

    const dayTr = document.createElement("tr");
    dayTr.className = "kgw-day-group-row";
    dayTr.dataset.kgwDateGroup = day;
    dayTr.innerHTML = `
      <td>${sign} ${day}</td>
      <td>${count.toLocaleString()} transactions</td>
      <td></td>
      <td>${kgwSummaryFormatKas(netKas)}</td>
      <td></td>
      <td></td>
    `;

    fragment.appendChild(dayTr);

    if (!expanded) {
      continue;
    }

    const cached = window.__kgwExplorerDayTransactionCache?.get(day);

    if (!cached) {
      const loadingTr = document.createElement("tr");
      loadingTr.className = "kgw-transaction-row";
      loadingTr.innerHTML = `
        <td colspan="6" class="muted">Loading transactions for ${day}...</td>
      `;
      fragment.appendChild(loadingTr);
      continue;
    }

    for (const row of cached) {
      const txid = row?.txid || row?.transaction_id || row?.transactionId || "";

      const txTr = document.createElement("tr");
      txTr.className = "kgw-transaction-row";
      txTr.dataset.kgwParentDateGroup = day;
      txTr.innerHTML = `
        <td>${row?.datetime || row?.date_time || row?.time || row?.timestamp || ""}</td>
        <td>${txid}</td>
        <td>${row?.direction || ""}</td>
        <td>${kgwSummaryFormatKas(Number(row?.amount_kas ?? row?.amountKas ?? row?.amount ?? 0) || 0)}</td>
        <td>${kgwSummaryFormatUsd(Number(row?.value_usd ?? row?.valueUsd ?? row?.usd_value ?? row?.usdValue ?? 0) || 0)}</td>
        <td>${row?.tx_type || row?.type || ""}</td>
      `;

      fragment.appendChild(txTr);
    }
  }

  body.appendChild(fragment);

  qsa("[data-kgw-date-group]", body).forEach((row) => {
    row.addEventListener("click", async () => {
      const day = row.dataset.kgwDateGroup;
      if (!day) return;

      if (window.__kgwExplorerExpandedDateGroups.has(day)) {
        window.__kgwExplorerExpandedDateGroups.delete(day);
        await kgwRenderDaySummariesDirect(section, explorerState.rows || [], statusText);
        return;
      }

      window.__kgwExplorerExpandedDateGroups.add(day);
      await kgwRenderDaySummariesDirect(section, explorerState.rows || [], statusText);

      if (!window.__kgwExplorerDayTransactionCache.has(day)) {
        try {
          const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", section)?.value);
          const dayRows = await kgwLoadTransactionsForSingleDayFromDb(section, address, day);
          window.__kgwExplorerDayTransactionCache.set(day, dayRows);

          microscopeLog("DIRECT DAY TRANSACTIONS LOADED", {
            day,
            rows: dayRows.length
          });
        } catch (error) {
          microscopeWarn("DIRECT DAY TRANSACTIONS LOAD FAILED", {
            day,
            message: error?.message || String(error)
          });

          window.__kgwExplorerDayTransactionCache.set(day, []);
        }

        await kgwRenderDaySummariesDirect(section, explorerState.rows || [], statusText);
      }
    });
  });

  syncActionState(section);

  if (statusText) {
    setStatus(section, statusText);
  }

  microscopeLog("DIRECT DAY SUMMARY RENDER DONE", {
    days: safeSummaries.length,
    totalCount
  });
}

async function kgwLoadAndRenderDaySummaries(section, address, startTs, endTs, statusText = "") {
  window.__kgwExplorerDayTransactionCache = new Map();

  const summaries = await kgwLoadTransactionDaySummariesFromDb(section, address, startTs, endTs);

  explorerState.selectedAddress = address;

  await kgwRenderDaySummariesDirect(
    section,
    summaries,
    statusText || `Displayed ${summaries.length.toLocaleString()} days from local database. Click + to load one day.`
  );

  return summaries;
}


/* KGW_TX_UI_FAST_3E_SINGLE_SUMMARY_RENDER_PATH
   One authoritative Explorer transaction table path:
   - collapsed view renders day summaries only
   - expanded view loads exactly one day from local database
   - no full 38k-row render during normal table display
*/
if (!window.__kgwExplorerDayTransactionCache) {
  window.__kgwExplorerDayTransactionCache = new Map();
}

if (!window.__kgwExplorerExpandedDateGroups) {
  window.__kgwExplorerExpandedDateGroups = new Set();
}









function resetFilters(section) {
  qs("#explorerDirectionFilter", section).value = "ALL";
  qs("#explorerTypeFilter", section).value = "ALL";
  qs("#explorerSearch", section).value = "";
  explorerState.filteredRows = explorerState.rows.slice();
  renderTable(section);
}

/* Explorer table font-size ownership lives in Rust/WASM explorer_controls.rs. */


/* KGW_DB_FILTER_ONLY
   Python parity: filter button reads from DB only.
*/
async function applyExplorerFiltersFromDatabase(section) {
  if (explorerState.busy) {
    setStatus(section, "Fetch is running. Wait until it finishes before applying filters.");
    return;
  }

  let address = normalizeAddress(qs("#explorerAddress", section)?.value);
  address = await kgwCanonicalKaspaAddress(address);

  if (!address) {
    clearExplorerTransactionTable(section, "Enter a valid Kaspa address.");
    setStatus(section, "Enter a valid Kaspa address.", "error");
    return;
  }

  const startTs = parseDateSeconds(qs("#explorerFromDate", section)?.value, false);
  const endTs = parseDateSeconds(qs("#explorerToDate", section)?.value, true);

  setStatus(section, "Loading filtered transactions from database...");

  await kgwLoadAndRenderDaySummaries(section, address, startTs, endTs, "Filter applied. Showing days from local database. Click + to load one day.");
}
function installEvents(section) {
  const addressInput = qs("#explorerAddress", section);
  const fromDateInput = qs("#explorerFromDate", section);
  const toDateInput = qs("#explorerToDate", section);

  qs("#explorerFetch", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-fetch", "r53b3-explorer-fetch-click", {
      trusted: Boolean(event && event.isTrusted),
      addressLength: String(qs("#explorerAddress", section)?.value || "").length,
      fromDate: String(qs("#explorerFromDate", section)?.value || ""),
      toDate: String(qs("#explorerToDate", section)?.value || "")
    });
    return fetchTransactions(section, false);
  };

  qs("#explorerForceFetch", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-fetch", "r53b3-explorer-force-fetch-click", {
      trusted: Boolean(event && event.isTrusted),
      addressLength: String(qs("#explorerAddress", section)?.value || "").length,
      fromDate: String(qs("#explorerFromDate", section)?.value || ""),
      toDate: String(qs("#explorerToDate", section)?.value || "")
    });
    return fetchTransactions(section, true);
  };

  qs("#explorerOpenExplorer", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-open", "r53b3-explorer-open-explorer-click", {
      trusted: Boolean(event && event.isTrusted),
      addressLength: String(qs("#explorerAddress", section)?.value || "").length
    });
    return openBlockExplorer(section);
  };

  if (addressInput) {
    addressInput.oninput = (event) => {
      kgwExplorerUiTraceR53B3("explorer-address", "r53b3-explorer-address-input", {
        trusted: Boolean(event && event.isTrusted),
        valueLength: String(addressInput.value || "").length
      });
      explorerState.selectedAddress = normalizeAddress(addressInput.value);
      clearExplorerTransactionTable(section, "Address changed. Table cleared.");
    };

    addressInput.onchange = (event) => {
      kgwExplorerUiTraceR53B3("explorer-address", "r53b3-explorer-address-change", {
        trusted: Boolean(event && event.isTrusted),
        valueLength: String(addressInput.value || "").length
      });
      explorerState.selectedAddress = normalizeAddress(addressInput.value);
      clearExplorerTransactionTable(section, "Address changed. Table cleared.");
    };
  }

  if (fromDateInput) {
    fromDateInput.onchange = (event) => {
      kgwExplorerUiTraceR53B3("explorer-date", "r53b3-explorer-from-date-change", {
        trusted: Boolean(event && event.isTrusted),
        value: String(fromDateInput.value || "")
      });
      clearExplorerTransactionTable(section, "Date range changed. Table cleared.");
    };
  }

  if (toDateInput) {
    toDateInput.onchange = (event) => {
      kgwExplorerUiTraceR53B3("explorer-date", "r53b3-explorer-to-date-change", {
        trusted: Boolean(event && event.isTrusted),
        value: String(toDateInput.value || "")
      });
      clearExplorerTransactionTable(section, "Date range changed. Table cleared.");
    };
  }

  qs("#explorerCancel", section).onclick = async (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-cancel", "r53b3-explorer-cancel-click", {
      trusted: Boolean(event && event.isTrusted),
      busyBefore: Boolean(explorerState.busy),
      cancelBefore: Boolean(explorerState.cancelRequested)
    });
    explorerState.cancelRequested = true;

    const backendRequestId = String(explorerState.backendCancelRequestId || "");

    if (backendRequestId) {
      kgwExplorerUiTraceR53B3("explorer-cancel", "r57d4-explorer-backend-cancel-invoke", {
        trusted: Boolean(event && event.isTrusted),
        requestIdLength: backendRequestId.length
      });

      try {
        await kgwInvokeExplorerCancelTransactionsR57D4(backendRequestId);
      } catch (error) {
        kgwExplorerUiTraceR53B3("explorer-cancel", "r57d4-explorer-backend-cancel-error", {
          message: String(error && (error.message || error))
        });
      }
    }

    clearExplorerTransactionTable(section, "Cancel requested. Table cleared.", {
      preserveCancelRequested: true
    });
  };

  qs("#explorerApplyFilter", section).onclick = async (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-filter", "r53b3-explorer-apply-filter-click", {
      trusted: Boolean(event && event.isTrusted),
      type: String(qs("#explorerTypeFilter", section)?.value || "ALL"),
      direction: String(qs("#explorerDirectionFilter", section)?.value || "ALL"),
      searchLength: String(qs("#explorerSearch", section)?.value || "").length
    });
    try {
      await applyExplorerFiltersFromDatabase(section);
    } catch (error) {
      microscopeError("DB FILTER FAILED", error, {});
      setStatus(section, error?.message || String(error));
    }
  };

  qs("#explorerResetFilter", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-filter", "r53b3-explorer-reset-filter-click", {
      trusted: Boolean(event && event.isTrusted)
    });
    resetFilters(section);
    clearExplorerTransactionTable(section, "Filters reset. Table cleared.");
  };

  qs("#explorerExportCsv", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-export", "r53b3-explorer-export-csv-click", {
      trusted: Boolean(event && event.isTrusted)
    });
    return exportCsv(section);
  };

  qs("#explorerExportHtml", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-export", "r53b3-explorer-export-html-click", {
      trusted: Boolean(event && event.isTrusted)
    });
    return exportHtml(section);
  };

  qs("#explorerExportPdf", section).onclick = (event) => {
    event?.preventDefault?.();
    kgwExplorerUiTraceR53B3("explorer-export", "r53b3-explorer-export-pdf-click", {
      trusted: Boolean(event && event.isTrusted)
    });
    return exportPdf(section);
  };
}


const EXPLORER_MICROSCOPE_ENABLED = true;

function microscopeNow() {
  try {
    return new Date().toISOString();
  } catch (_) {
    return "";
  }
}

function microscopeSafeJson(value) {
  try {
    return JSON.stringify(value, function (_key, val) {
      if (typeof val === "bigint") return String(val);
      if (val instanceof Error) return { message: val.message, stack: val.stack };
      return val;
    });
  } catch (_) {
    return String(value);
  }
}

function microscopeLog(stage, details = {}) {
  if (!EXPLORER_MICROSCOPE_ENABLED) return;

  const payload = {
    stage,
    at: microscopeNow(),
    ...details
  };

  console.info(`[KGW][microscope][explorer] ${stage} :: ${microscopeSafeJson(payload)}`);
}

function microscopeWarn(stage, details = {}) {
  const payload = {
    stage,
    at: microscopeNow(),
    ...details
  };

  console.warn(`[KGW][microscope][explorer] ${stage} :: ${microscopeSafeJson(payload)}`);
}

function microscopeError(stage, error, details = {}) {
  const payload = {
    stage,
    at: microscopeNow(),
    error: error?.message || String(error),
    stack: error?.stack || "",
    ...details
  };

  console.error(`[KGW][microscope][explorer] ${stage} :: ${microscopeSafeJson(payload)}`);
}

function microscopeElementReport(section) {
  const ids = [
    "explorer",
    "explorerAddress",
    "explorerAddressOptions",
    "explorerBalanceValue",
    "explorerAddressNameValue",
    "explorerBalanceUsdValue",
    "explorerFromDate",
    "explorerToDate",
    "explorerDirectionFilter",
    "explorerTypeFilter",
    "explorerSearch",
    "explorerFetch",
    "explorerForceFetch",
    "explorerOpenExplorer",
    "explorerCancel",
    "explorerTransactionsTable",
    "explorerTransactionsBody",
    "explorerExportControls",
    "explorerExportCsv",
    "explorerExportHtml",
    "explorerExportPdf",
    "explorerTableFontSize",
    "explorerStatus"
  ];

  const report = {};

  for (const id of ids) {
    const node = section?.querySelector?.(`#${id}`) || document.getElementById(id);
    const rect = node?.getBoundingClientRect?.();

    report[id] = {
      exists: Boolean(node),
      tag: node?.tagName || null,
      hidden: Boolean(node?.hidden),
      disabled: Boolean(node?.disabled),
      display: node ? getComputedStyle(node).display : null,
      visibility: node ? getComputedStyle(node).visibility : null,
      width: rect ? Math.round(rect.width) : null,
      height: rect ? Math.round(rect.height) : null,
      textLength: node?.textContent?.length || 0,
      value: "value" in (node || {}) ? node.value : undefined
    };
  }

  microscopeLog("DOM REPORT", report);
  return report;
}

function microscopeLayoutReport(section) {
  const tableZone = section?.querySelector?.(".explorer-table-zone");
  const footer = section?.querySelector?.("#explorerExportControls");
  const load = section?.querySelector?.(".explorer-load-card");
  const filter = section?.querySelector?.(".explorer-filter-card");
  const shell = section?.querySelector?.(".explorer-clean-shell");

  function box(node) {
    const rect = node?.getBoundingClientRect?.();
    if (!rect) return null;
    const style = getComputedStyle(node);
    return {
      x: Math.round(rect.x),
      y: Math.round(rect.y),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
      display: style.display,
      overflow: style.overflow,
      gridTemplateRows: style.gridTemplateRows || ""
    };
  }

  microscopeLog("LAYOUT REPORT", {
    shell: box(shell),
    load: box(load),
    filter: box(filter),
    tableZone: box(tableZone),
    footer: box(footer),
    viewport: {
      width: window.innerWidth,
      height: window.innerHeight
    }
  });
}

function microscopeStateReport(label = "STATE REPORT") {
  const address = normalizeAddress(qs("#explorerAddress", root())?.value);
  const addressState = explorerAddressDiagnosticsSnapshot(address);
  microscopeLog(label, {
    rows: explorerState?.rows?.length ?? null,
    filteredRows: explorerState?.filteredRows?.length ?? null,
    selectedAddress: explorerState?.selectedAddress ?? "",
    busy: explorerState?.busy ?? null,
    addressState
  });
}

function microscopeApiShape(label, value) {
  if (Array.isArray(value)) {
    microscopeLog(label, {
      kind: "array",
      length: value.length,
      first: value[0] || null
    });
    return;
  }

  if (value && typeof value === "object") {
    const keys = Object.keys(value);
    microscopeLog(label, {
      kind: "object",
      keys,
      groupsLength: Array.isArray(value.groups) ? value.groups.length : null,
      rowsLength: Array.isArray(value.rows) ? value.rows.length : null,
      transactionsLength: Array.isArray(value.transactions) ? value.transactions.length : null,
      firstGroupKeys: Array.isArray(value.groups) && value.groups[0] ? Object.keys(value.groups[0]) : null,
      firstGroup: Array.isArray(value.groups) && value.groups[0] ? value.groups[0] : null
    });
    return;
  }

  microscopeLog(label, {
    kind: typeof value,
    value
  });
}




export async function initExplorerTab() {
  const section = root();

  microscopeLog("INIT START", {
    sectionId: section?.id || null,
    sectionClass: section?.className || null,
    sectionTextLength: section?.textContent?.length || 0
  });
  microscopeElementReport(section);
  microscopeLayoutReport(section);
  microscopeStateReport("INIT STATE BEFORE");

  if (section.dataset.kgwExplorerCleanInit === "1") {
    syncActionState(section);
    return;
  }

  section.dataset.kgwExplorerCleanInit = "1";

  defaultDates(section);
  installEvents(section);
  setTableFontSize(section);
  kgwBindFontSpinbox(section);
  syncActionState(section);

  await Promise.allSettled([
    loadKnownAddressNames(),
    loadSavedAddresses(section)
  ]);

  const address = normalizeAddress(qs("#explorerAddress", section)?.value);

  if (isKaspaAddress(address)) {
    await Promise.allSettled([
      fetchBalance(section, address),
      refreshAddressName(section, address)
    ]);
  }

  // TX_LOAD_ADDRESS_LAYOUT_2_INIT_CALL
renderTable(section);
  microscopeElementReport(section);
  microscopeLayoutReport(section);
  microscopeStateReport("INIT STATE AFTER");
  setStatus(section, "Ready");
}


/* KGW_TX_UI_CLEAN_2_SINGLE_OWNER_CORE
   Final owner for Explorer transaction table:
   - Fetch renders day summaries only.
   - Filter renders day summaries only.
   - Expanding + loads one day only.
   - Old renderTable/fetchTransactions/applyFilter declarations were removed above.
*/
function kgwClean2Log(label, payload = {}) {
  try {
    console.log(`[KGW Explorer][clean2] ${label}`, payload);
  } catch (_) {
    console.log(`[KGW Explorer][clean2] ${label}`);
  }
}

function kgwClean2Section(section) {
  return (
    section ||
    document.querySelector("#explorer") ||
    document.querySelector(".explorer-python-root")
  );
}

function kgwClean2Body(section) {
  const root = kgwClean2Section(section);
  return (
    qs("#explorerTransactionsBody", root) ||
    root?.querySelector?.("tbody") ||
    document.querySelector("#explorerTransactionsBody")
  );
}


/* Explorer filter normalization/repair/init and Clean2 request ownership live in Rust/WASM via explorer.utils.js. */

async function kgwClean2LoadSummaries(section, address, startTs, endTs) {
  const request = kgwClean2Request(section, address, startTs, endTs, 10000);
  const started = performance.now();
  const result = await kgwInvokeExplorerDaySummaries(request);
  const rows = kgwClean2NormalizeSummaries(result);

  kgwClean2Log("summaries loaded", {
    elapsedMs: Math.round(performance.now() - started),
    days: rows.length,
    totalTransactions: rows.reduce((sum, row) => sum + (Number(row.count) || 0), 0),
    first: rows[0] || null
  });

  return rows;
}

async function kgwClean2LoadDayTransactions(section, address, day) {
  const startTs = kgwClean2DayToSeconds(day, false);
  const endTs = kgwClean2DayToSeconds(day, true);

  if (!Number.isFinite(startTs) || !Number.isFinite(endTs)) {
    return [];
  }

  const request = kgwClean2Request(section, address, startTs, endTs, 1000000);
  const started = performance.now();
  const groups = await kgwInvokeExplorerGroupedTransactions(request);
  const rows = extractRowsFromUnifiedResult({ groups });

  kgwClean2Log("day transactions loaded", {
    day,
    elapsedMs: Math.round(performance.now() - started),
    rows: rows.length
  });

  return rows;
}

/* KGW_EXPORT_RAW_PAYLOAD_PARITY_EXPLORER_V2
   Export must use raw transaction rows, not the visible day-summary table.
   Pure raw-transaction normalization/number formatting is Rust-owned in explorer_export.rs
   and reached through deterministic explorer.export.js ABI glue.
   Async day loading/table assembly remains here until its own migration boundary.
*/
async function kgwExplorerBuildRawExportTableV2(section) {
  const root = kgwClean2Section(section);
  const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", root)?.value);
  const visibleRows = Array.isArray(explorerState?.filteredRows) ? explorerState.filteredRows : [];

  if (!isKaspaAddress(address)) {
    throw new Error("Enter a valid Kaspa address before exporting raw transactions.");
  }

  if (!visibleRows.length) {
    throw new Error("No explorer rows are available for export. Fetch transactions first, then export.");
  }

  const rawRows = [];
  const seenTxids = new Set();
  const summaryDays = visibleRows
    .filter((row) => row?.__kgwDaySummary && row.day)
    .map((row) => String(row.day).slice(0, 10))
    .filter(Boolean);

  if (summaryDays.length) {
    for (const day of summaryDays) {
      const dayRows = await kgwClean2LoadDayTransactions(root, address, day);

      for (const row of Array.isArray(dayRows) ? dayRows : []) {
        const normalized = kgwExplorerExportNormalizeRawTxV2(row);
        if (!normalized.txid || seenTxids.has(normalized.txid)) continue;
        seenTxids.add(normalized.txid);
        rawRows.push(normalized);
      }
    }
  } else {
    for (const row of visibleRows) {
      const normalized = kgwExplorerExportNormalizeRawTxV2(row);
      if (!normalized.txid || seenTxids.has(normalized.txid)) continue;
      seenTxids.add(normalized.txid);
      rawRows.push(normalized);
    }
  }

  rawRows.sort((a, b) => {
    if (b.timestampMs !== a.timestampMs) return b.timestampMs - a.timestampMs;
    return String(b.datetime || "").localeCompare(String(a.datetime || ""));
  });

  if (!rawRows.length) {
    throw new Error("Explorer raw transaction export found no transaction rows. Expand/fetch data first, then export.");
  }

  const rows = rawRows.map((row) => [
    row.datetime,
    row.txid,
    row.direction,
    row.fromAddress,
    row.toAddress,
    kgwExplorerExportNumberV2(row.amount, 8),
    row.blockScore,
    String(row.timestampMs || ""),
    row.type,
    kgwExplorerExportNumberV2(row.value, 2),
    row.date,
    row.transactionUrl,
    row.addressUrl
  ]);

  window.__KGW_EXPLORER_RAW_EXPORT_LAST_V2 = {
    at: new Date().toISOString(),
    days: summaryDays.length,
    rows: rows.length,
    address
  };

  return {
    title: "Kaspa Gateway Explorer Transactions",
    subtitle: `Address: ${address} | Raw transactions: ${rows.length}`,
    headers: [
      "Date/Time",
      "Transaction ID",
      "Direction",
      "From Address(es)",
      "To Address(es)",
      "Amount (KAS)",
      "Block Score",
      "timestamp",
      "Type:",
      "Value (USD)",
      "date",
      "Transaction URL",
      "Address URL"
    ],
    rows
  };
}

window.__kgwExplorerBuildRawExportTableV2 = kgwExplorerBuildRawExportTableV2;


async function kgwClean2RenderSummaries(section, rows, statusText = "") {
  const root = kgwClean2Section(section);
  const body = kgwClean2Body(root);

  if (!body) {
    console.error("[KGW Explorer][clean2] tbody not found");
    return;
  }

  const summaries = kgwClean2NormalizeSummaries(rows);

  explorerState.rows = summaries;
  explorerState.filteredRows = summaries.slice();

  body.innerHTML = "";

  if (!summaries.length) {
    body.innerHTML = '<tr><td colspan="6" class="muted" data-i18n="explorer.noTransactionsToDisplay">No transactions to display.</td></tr>';
    if (statusText) setStatus(root, statusText);
    syncActionState(root);

    kgwClean2Log("render empty", {
      tbodyFound: true,
      childRows: body.children.length
    });
    return;
  }

  let totalCount = 0;
  let totalNetKas = 0;
  let totalUsd = 0;

  for (const summary of summaries) {
    totalCount += Number(summary.count || 0) || 0;
    totalNetKas += Number(summary.net_kas || 0) || 0;
    totalUsd += Number(summary.value_usd || 0) || 0;
  }

  const fragment = document.createDocumentFragment();

  const totalTr = document.createElement("tr");
  totalTr.className = "kgw-total-row";
  totalTr.innerHTML =
    "<td data-i18n=\"explorer.total\">Total</td>" +
    `<td>${totalCount.toLocaleString()} transactions</td>` +
    "<td></td>" +
    `<td>${kgwClean2Kas(totalNetKas)}</td>` +
    `<td>${kgwClean2Usd(totalUsd)}</td>` +
    "<td></td>";
  fragment.appendChild(totalTr);

  for (const summary of summaries) {
    const day = summary.day;
    const expanded = window.__kgwExplorerExpandedDateGroups?.has(day) === true;
    const sign = expanded ? "-" : "+";

    const dayTr = document.createElement("tr");
    dayTr.className = "kgw-day-group-row";
    dayTr.dataset.kgwDateGroup = day;
    dayTr.innerHTML =
      `<td>${sign} ${kgwClean2SafeText(day)}</td>` +
      `<td>${Number(summary.count || 0).toLocaleString()} transactions</td>` +
      "<td></td>" +
      `<td>${kgwClean2Kas(summary.net_kas)}</td>` +
      `<td>${kgwClean2Usd(summary.value_usd)}</td>` +
      "<td></td>";

    fragment.appendChild(dayTr);

    if (!expanded) continue;

    const cachedRows = window.__kgwExplorerDayTransactionCache?.get(day);

    if (!cachedRows) {
      const loadingTr = document.createElement("tr");
      loadingTr.className = "kgw-transaction-row";
      loadingTr.innerHTML = `<td colspan="6" class="muted">Loading transactions for ${kgwClean2SafeText(day)}...</td>`;
      fragment.appendChild(loadingTr);
      continue;
    }

    for (const tx of cachedRows) {
      const txid = tx?.txid || tx?.transaction_id || tx?.transactionId || "";
      const amount = Number(tx?.amount_kas ?? tx?.amountKas ?? tx?.amount ?? 0) || 0;
      const usd = Number(
        tx?.value_usd ??
        tx?.valueUsd ??
        tx?.usd_value ??
        tx?.usdValue ??
        tx?.value ??
        Math.abs(amount) * kgwClean2UsdPrice()
      ) || 0;

      const txTr = document.createElement("tr");
      txTr.className = "kgw-transaction-row";
      txTr.dataset.kgwParentDateGroup = day;
      txTr.innerHTML =
        `<td>${kgwClean2SafeText(tx?.datetime || tx?.date_time || tx?.time || tx?.timestamp || "")}</td>` +
        `<td>${kgwClean2SafeText(txid)}</td>` +
        `<td>${kgwClean2SafeText(tx?.direction || "")}</td>` +
        `<td>${kgwClean2Kas(amount)}</td>` +
        `<td>${kgwClean2Usd(usd)}</td>` +
        `<td>${kgwClean2SafeText(tx?.tx_type || tx?.type || "")}</td>`;

      fragment.appendChild(txTr);
    }
  }

  body.appendChild(fragment);

  body.querySelectorAll("[data-kgw-date-group]").forEach((row) => {
    row.addEventListener("click", async () => {
      const day = row.dataset.kgwDateGroup;
      if (!day) return;

      if (!window.__kgwExplorerExpandedDateGroups) {
        window.__kgwExplorerExpandedDateGroups = new Set();
      }

      if (!window.__kgwExplorerDayTransactionCache) {
        window.__kgwExplorerDayTransactionCache = new Map();
      }

      if (window.__kgwExplorerExpandedDateGroups.has(day)) {
        window.__kgwExplorerExpandedDateGroups.delete(day);
        await kgwClean2RenderSummaries(root, explorerState.rows || [], statusText);
        return;
      }

      window.__kgwExplorerExpandedDateGroups.add(day);
      await kgwClean2RenderSummaries(root, explorerState.rows || [], statusText);

      if (!window.__kgwExplorerDayTransactionCache.has(day)) {
        const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", root)?.value);
        const dayRows = await kgwClean2LoadDayTransactions(root, address, day);
        window.__kgwExplorerDayTransactionCache.set(day, Array.isArray(dayRows) ? dayRows : []);
        await kgwClean2RenderSummaries(root, explorerState.rows || [], statusText);
      }
    });
  });

  if (statusText) setStatus(root, statusText);
  syncActionState(root);

  kgwClean2Log("render done", {
    days: summaries.length,
    childRows: body.children.length,
    textPreview: body.innerText.slice(0, 260)
  });
}

function renderTable(section) {
  /*
   * KGW_PHASE3E_CANONICAL_RENDER_TABLE
   *
   * renderTable is kept as the compatibility entry point, but rendering is now
   * delegated to the canonical Clean2 grouped-summary renderer.
   * Old grouped renderers remain in the file for rollback/parity review, but
   * renderTable must not call more than one renderer.
   */
  const root = section || document.querySelector("#explorer");
  const rows = Array.isArray(explorerState?.filteredRows)
    ? explorerState.filteredRows
    : Array.isArray(explorerState?.rows)
      ? explorerState.rows
      : [];

  void kgwClean2RenderSummaries(root, rows, "");
}


/* KGW_TX_FORCE_UI_LOCK_1
   Explorer force-fetch table messaging, filter reset, busy controls, and capture blocking
   are Rust-owned by explorer_force_ui.rs and exposed through generated explorer.utils.js.
*/
kgwInstallForceBusyBlocker();

/* KGW_TX_LIVE_CORE_1_LISTENER
   Generic live table updates for both normal and force fetch.
   It listens to pages already stored by the existing Rust sync loop.
*/
function kgwLiveCoreRoot() {
  return document.querySelector("#explorer") || document.querySelector(".explorer-python-root");
}

/* Explorer live-core state, aggregation, ordering, and render throttling are Rust-owned via explorer.utils.js. */

async function kgwInstallTxLiveCoreListener() {
  if (window.__kgwTxLiveCoreListenerInstalled) return;

  const listen = window.__TAURI__?.event?.listen;

  if (typeof listen !== "function") {
    console.warn("[KGW Explorer][live-core] Tauri event listen API not found");
    return;
  }

  window.__kgwTxLiveCoreListenerInstalled = true;

  await listen("kgw://transactions/page-stored", async (event) => {
    const payload = event?.payload || {};
    const root = kgwLiveCoreRoot();

    if (!root || root.dataset.kgwFetchBusy !== "true") {
      return;
    }

    const activeAddress =
      kgwLiveCoreAddress() ||
      normalizeAddress(qs("#explorerAddress", root)?.value);

    if (payload.address && activeAddress && payload.address !== activeAddress) {
      return;
    }

    if (payload.phase !== "page_stored") {
      return;
    }

    if (Array.isArray(payload.days) && payload.days.length) {
      kgwLiveCoreMergeDays(payload.days);
    } else {
      kgwLiveCoreMergeRecords(payload.records);
    }

    const rows = kgwLiveCoreRows();

    if (!kgwLiveCoreShouldRender(payload)) {
      setStatus(
        root,
        `Fetch is running... page ${payload.page}, stored ${Number(payload.stored_total || 0).toLocaleString()} transactions.`
      );

      return;
    }

    await kgwClean2RenderSummaries(
      root,
      rows,
      `Fetch is running... page ${payload.page}, stored ${Number(payload.stored_total || 0).toLocaleString()} transactions.`
    );

    kgwForceSetControlsBusy(root, true, payload.mode === "force" ? "force" : "normal");

    console.log("[KGW Explorer][live-core] rendered page", {
      mode: payload.mode,
      page: payload.page,
      pageStored: payload.page_stored,
      storedTotal: payload.stored_total,
      compactDays: Array.isArray(payload.days) ? payload.days.length : 0,
      records: Array.isArray(payload.records) ? payload.records.length : 0,
      days: rows.length
    });
  });

  console.log("[KGW Explorer][live-core] listener installed");
}


/* KGW_TX_FORCE_FINAL_LOCAL_DATABASE_REFRESH
   Live-core is only a temporary view while pages are arriving.
   The official final table must always be loaded from local database after explorer_transactions returns.
*/


async function fetchTransactions(section, forceMode) {
  const root = kgwClean2Section(section);
  const isForce = Boolean(forceMode);
  let address = normalizeAddress(qs("#explorerAddress", root)?.value);

  
  kgwExplorerUiTraceR53B3("explorer-fetch", "r53b3-explorer-fetch-owner-begin", {
    force: Boolean(isForce),
    addressLength: String(address || "").length
  });

  address = await kgwCanonicalKaspaAddress(address);
  if (!address) {
    clearExplorerTransactionTable(root, "Enter a valid Kaspa address.");
    setStatus(root, "Enter a valid Kaspa address.", "error");
    return;
  }

  const startTs = parseDateSeconds(qs("#explorerFromDate", root)?.value, false);
  const endTs = parseDateSeconds(qs("#explorerToDate", root)?.value, true);

  explorerState.selectedAddress = address;
  explorerState.busy = true;
  explorerState.cancelRequested = false;

  const backendRequestIdR57D4 = `explorer-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  explorerState.backendCancelRequestId = backendRequestIdR57D4;

  const stopIfExplorerCancelRequestedR56E = (stage) => {
    if (!explorerState.cancelRequested) {
      return false;
    }

    kgwExplorerUiTraceR53B3("explorer-fetch", "r56e-explorer-fetch-cancel-observed", {
      force: Boolean(isForce),
      stage: String(stage || "")
    });

    clearExplorerTransactionTable(root, "Fetch cancelled. Table cleared.", {
      preserveCancelRequested: true
    });

    return true;
  };

  if (typeof window.kgwSetGlobalFetchBusy === "function") {
    window.kgwSetGlobalFetchBusy(true, {
      text: isForce
        ? "Force fetch is starting..."
        : "Fetch is starting..."
    });
  }

  window.__kgwExplorerDayTransactionCache = new Map();
  window.__kgwExplorerExpandedDateGroups = new Set();

  await kgwInstallTxLiveCoreListener();

  kgwLiveCoreReset(address);

  kgwForceSetControlsBusy(root, true, isForce ? "force" : "normal");
  syncActionState(root);
  try {
    if (isForce) {
      kgwForceResetDisplayFiltersToAll(root);

      kgwForceSetTableMessage(
        root,
        "Force fetch is running... deleting old transactions, then running the normal accepted-transactions fetch."
      );

      setStatus(root, "Force fetch is running... deleting old local database transactions.");
    } else {
      setStatus(root, "Loading days from local database...");

      const firstRows = await kgwClean2LoadSummaries(root, address, startTs, endTs);

      if (stopIfExplorerCancelRequestedR56E("after-local-summary-load")) {
        return;
      }

      await kgwClean2RenderSummaries(
        root,
        firstRows,
        firstRows.length
          ? `${firstRows.length.toLocaleString()} days loaded from local database. Fetch is updating...`
          : "No saved transaction days found yet. Fetch is running..."
      );

      kgwLiveCoreSeedRows(address, explorerState.rows);

      if (stopIfExplorerCancelRequestedR56E("after-initial-render")) {
        return;
      }

      kgwForceSetControlsBusy(root, true, "normal");
    }

    await Promise.allSettled([
      fetchBalance(root, address),
      refreshAddressName(root, address),
      saveAddressToDatabase(address, "")
    ]);

    if (stopIfExplorerCancelRequestedR56E("after-address-side-effects")) {
      return;
    }

    const request = {
      address,
      force: isForce,
      start_ts: startTs,
      end_ts: endTs,
      request_id: backendRequestIdR57D4,
      // Force fetch must rebuild the address database completely.
      // Do not let current display filters limit what is fetched/stored.
      tx_type: isForce ? "ALL" : kgwNormalizeTxTypeFilterValue(qs("#explorerTypeFilter", root)?.value),
      direction: isForce ? "ALL" : kgwNormalizeDirectionFilterValue(qs("#explorerDirectionFilter", root)?.value),
      search_query: isForce ? "" : String(qs("#explorerSearch", root)?.value || "")
    };

    console.log("[KGW Explorer][force-ui] fetch request", request);

    const result = await kgwInvokeExplorerUnifiedFetch(request);
    kgwClean2Log("fetch result ignored for table", result);

    if (stopIfExplorerCancelRequestedR56E("after-unified-fetch")) {
      return;
    }

    const finalRows = await kgwClean2LoadSummaries(root, address, startTs, endTs);

    if (stopIfExplorerCancelRequestedR56E("after-final-summary-load")) {
      return;
    }

    await kgwClean2RenderSummaries(
      root,
      finalRows,
      isForce
        ? `Force fetch done. Showing ${finalRows.length.toLocaleString()} days from local database. Click + to load one day.`
        : `Fetch done. Showing ${finalRows.length.toLocaleString()} days from local database. Click + to load one day.`
    );
  } catch (error) {
    kgwExplorerUiTraceR53B3("explorer-fetch", "r53b3-explorer-fetch-owner-error", {
      force: Boolean(isForce),
      message: error?.message || String(error)
    });
    console.error("[KGW Explorer][force-ui] fetch failed", error);
    try {
      const rows = await kgwClean2LoadSummaries(root, address, startTs, endTs);

      await kgwClean2RenderSummaries(
        root,
        rows,
        `${isForce ? "Force fetch" : "Fetch"} failed. Showing ${rows.length.toLocaleString()} days currently in local database.`
      );
    } catch (_) {
      clearExplorerTransactionTable(root, error?.message || String(error));
    }
  } finally {
        if (explorerState.backendCancelRequestId === backendRequestIdR57D4) {
      explorerState.backendCancelRequestId = "";
    }

kgwExplorerUiTraceR53B3("explorer-fetch", "r53b3-explorer-fetch-owner-finally", {
      force: Boolean(isForce),
      cancelRequested: Boolean(explorerState.cancelRequested)
    });
    explorerState.busy = false;

    if (typeof window.kgwSetGlobalFetchBusy === "function") {
      window.kgwSetGlobalFetchBusy(false, { text: "Ready" });
    }

    kgwForceSetControlsBusy(root, false, "");
    syncActionState(root);
  }
}

async function applyFilter(section) {
  const root = kgwClean2Section(section);
  let address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", root)?.value);

  kgwClean2Log("applyFilter start", {
    address,
    type: String(qs("#explorerTypeFilter", root)?.value || "ALL"),
    direction: String(qs("#explorerDirectionFilter", root)?.value || "ALL"),
    search: String(qs("#explorerSearch", root)?.value || "")
  });

  address = await kgwCanonicalKaspaAddress(address);
  if (!address) {
    clearExplorerTransactionTable(root, "Enter a valid Kaspa address.");
    setStatus(root, "Enter a valid Kaspa address.", "error");
    return;
  }

  const startTs = parseDateSeconds(qs("#explorerFromDate", root)?.value, false);
  const endTs = parseDateSeconds(qs("#explorerToDate", root)?.value, true);

  try {
    setStatus(root, "Applying filter from local database...");

    window.__kgwExplorerDayTransactionCache = new Map();
    window.__kgwExplorerExpandedDateGroups = new Set();

    const rows = await kgwClean2LoadSummaries(root, address, startTs, endTs);

    await kgwClean2RenderSummaries(
      root,
      rows,
      rows.length
        ? `Filter applied. Showing ${rows.length.toLocaleString()} days from local database. Click + to load one day.`
        : "Filter applied. No matching transaction days found."
    );
  } catch (error) {
    console.error("[KGW Explorer][clean2] applyFilter failed", error);
    clearExplorerTransactionTable(root, error?.message || String(error));
  }
}

/* KGW_TX_UI_CLEAN_2_FILTER_CAPTURE
   Capture Filter only. Fetch remains normal.
*/
if (!window.__kgwClean2FilterCaptureInstalled) {
  window.__kgwClean2FilterCaptureInstalled = true;

  document.addEventListener(
    "click",
    (event) => {
      const target = event.target?.closest?.("button,input[type='button'],input[type='submit'],[role='button'],.btn,.button");
      if (!target) return;

      const root =
        document.querySelector("#explorer") ||
        document.querySelector(".explorer-python-root");

      if (root && !root.contains(target)) return;

      const text = [
        target.id,
        target.name,
        target.value,
        target.textContent,
        target.getAttribute?.("aria-label"),
        target.dataset?.action,
        target.dataset?.kgwAction
      ].filter(Boolean).join(" ").toLowerCase();

      if (!text.includes("filter") || text.includes("reset")) return;

      event.preventDefault();
      event.stopPropagation();
      event.stopImmediatePropagation();

      kgwClean2Log("captured filter click", {
        id: target.id || "",
        value: target.value || "",
        text: String(target.textContent || "").trim()
      });

      void applyFilter(root);
    },
    true
  );
}




/* KGW_EXPLORER_FILTER_BUSY_LOCK_OWNER_V1
   Owns only Explorer filter availability:
   - enabled after a real saved address is selected
   - locked during Fetch / Force Fetch
   - unlocked after the fetch UI settles
   This intentionally does not own runtime fetch logic or transaction rendering. */
(function installKgwExplorerFilterBusyLockOwnerV1() {
  if (window.__kgwExplorerFilterBusyLockOwnerV1Installed) return;
  window.__kgwExplorerFilterBusyLockOwnerV1Installed = true;

  const state = {
    busy: false,
    busyStartedAt: 0,
    lastMutationAt: 0,
    unlockTimer: null,
    pollTimer: null,
  };

  function scope() {
    return document.getElementById("explorer") ||
      document.querySelector(".explorer-python-root") ||
      document.querySelector("[data-tab-panel='explorer']") ||
      document;
  }

  function normalize(value) {
    return String(value || "").replace(/\s+/g, " ").trim().toLowerCase();
  }

  function textOf(element) {
    return normalize(element?.textContent || element?.value || element?.getAttribute?.("aria-label") || "");
  }

  function isExplorerVisible() {
    const root = scope();

    if (!root || root === document) return true;

    const rect = root.getBoundingClientRect?.();

    if (!rect) return true;

    return rect.width > 0 && rect.height > 0;
  }

  function isAddressControl(element) {
    if (!element) return false;

    const id = normalize(element.id);
    const name = normalize(element.name);
    const label = normalize(element.getAttribute?.("aria-label"));
    const placeholder = normalize(element.getAttribute?.("placeholder"));

    if (id.includes("address") || name.includes("address") || label.includes("address") || placeholder.includes("address")) {
      return true;
    }

    if (element.tagName === "SELECT") {
      const optionText = Array.from(element.options || [])
        .slice(0, 25)
        .map((option) => String(option.value || "") + " " + String(option.textContent || ""))
        .join(" ")
        .toLowerCase();

      if (optionText.includes("kaspa:") || optionText.includes("saved address") || optionText.includes("select saved")) {
        return true;
      }
    }

    return false;
  }

  function addressControls() {
    const root = scope();
    const selectors = [
      "select",
      "input",
      "#explorerAddressSelect",
      "#explorerSavedAddressSelect",
      "#savedAddressSelect",
      "[data-role='address-select']"
    ];

    return Array.from(root.querySelectorAll(selectors.join(",")))
      .filter((element, index, array) => array.indexOf(element) === index)
      .filter(isAddressControl);
  }

  function hasSelectedAddress() {
    for (const element of addressControls()) {
      const value = String(element.value || "").trim();

      if (value.startsWith("kaspa:") || value.includes("kaspa:")) {
        return true;
      }

      if (element.tagName === "SELECT") {
        const selected = element.options?.[element.selectedIndex];
        const text = String(selected?.textContent || "").trim();

        if (text.startsWith("kaspa:") || text.includes("kaspa:")) {
          return true;
        }
      }
    }

    const root = scope();
    const visibleText = String(root.textContent || "");

    return /kaspa:[a-z0-9]{20,}/i.test(visibleText);
  }

  function isActionButton(element) {
    if (!element || element.tagName !== "BUTTON") return false;

    const id = normalize(element.id);
    const text = textOf(element);
    const i18n = normalize(element.dataset?.i18n);

    if (id.includes("fetch")) return true;
    if (id.includes("forcefetch")) return true;
    if (id.includes("cancel")) return true;
    if (id.includes("openexplorer")) return true;
    if (i18n.includes("fetch")) return true;
    if (i18n.includes("cancel")) return true;
    if (text === "fetch" || text === "force fetch" || text === "cancel" || text === "explorer") return true;

    return false;
  }

  function isFilterControl(element) {
    if (!element) return false;

    const root = scope();
    if (root !== document && !root.contains(element)) return false;

    if (isAddressControl(element)) return false;

    const id = normalize(element.id);
    const name = normalize(element.name);
    const placeholder = normalize(element.getAttribute?.("placeholder"));
    const i18n = normalize(element.dataset?.i18n);
    const text = textOf(element);
    const tag = element.tagName;

    if (isActionButton(element)) return false;

    if (tag === "INPUT" && element.type === "date") return true;
    if (tag === "INPUT" && (element.type === "search" || placeholder.includes("search"))) return true;

    if (tag === "SELECT") {
      if (id.includes("language") || id.includes("currency") || id.includes("theme")) return false;
      if (name.includes("language") || name.includes("currency") || name.includes("theme")) return false;
      return true;
    }

    if (tag === "BUTTON") {
      if (id.includes("filter") || id.includes("reset")) return true;
      if (i18n.includes("filter") || i18n.includes("reset")) return true;
      if (text === "filter" || text === "reset filter") return true;
    }

    return false;
  }

  function filterControls() {
    const root = scope();
    const selectors = [
      "input[type='date']",
      "input[type='search']",
      "input[placeholder*='Search']",
      "input[placeholder*='Address']",
      "input[placeholder*='Transaction']",
      "select",
      "button",
      "#explorerFilter",
      "#explorerResetFilter"
    ];

    return Array.from(root.querySelectorAll(selectors.join(",")))
      .filter((element, index, array) => array.indexOf(element) === index)
      .filter(isFilterControl);
  }

  function actionButtons() {
    const root = scope();

    return {
      fetch: root.querySelector("#explorerFetch") ||
        Array.from(root.querySelectorAll("button")).find((button) => textOf(button) === "fetch"),
      forceFetch: root.querySelector("#explorerForceFetch") ||
        Array.from(root.querySelectorAll("button")).find((button) => textOf(button).includes("force fetch")),
      cancel: root.querySelector("#explorerCancel") ||
        Array.from(root.querySelectorAll("button")).find((button) => textOf(button) === "cancel"),
    };
  }

  function setFilterAvailability(enabled, reason) {
    const controls = filterControls();

    for (const element of controls) {
      element.disabled = !enabled;
      element.setAttribute("aria-disabled", enabled ? "false" : "true");
      element.dataset.kgwExplorerFilterLifecycle = reason;
    }

    const root = scope();

    if (root && root !== document && root.dataset) {
      root.dataset.kgwExplorerFiltersEnabled = enabled ? "true" : "false";
      root.dataset.kgwExplorerFiltersReason = reason;
    }

    document.documentElement.dataset.kgwExplorerFiltersEnabled = enabled ? "true" : "false";
    document.documentElement.dataset.kgwExplorerFiltersReason = reason;
  }

  function refreshFilterAvailability(reason = "refresh") {
    if (state.busy) {
      setFilterAvailability(false, "busy:" + reason);
      return;
    }

    setFilterAvailability(hasSelectedAddress(), hasSelectedAddress() ? "address-selected:" + reason : "no-address:" + reason);
  }

  function areFetchButtonsIdle() {
    const buttons = actionButtons();
    const fetchIdle = !buttons.fetch || buttons.fetch.disabled === false;
    const forceIdle = !buttons.forceFetch || buttons.forceFetch.disabled === false;

    return fetchIdle && forceIdle;
  }

  function cancelLooksIdle() {
    const buttons = actionButtons();

    if (!buttons.cancel) return true;

    const style = window.getComputedStyle(buttons.cancel);
    const hidden = style.display === "none" || style.visibility === "hidden" || buttons.cancel.offsetParent === null;
    const disabled = buttons.cancel.disabled || buttons.cancel.getAttribute("aria-disabled") === "true";

    return hidden || disabled;
  }

  function stopPolling() {
    if (state.pollTimer) {
      window.clearInterval(state.pollTimer);
      state.pollTimer = null;
    }

    if (state.unlockTimer) {
      window.clearTimeout(state.unlockTimer);
      state.unlockTimer = null;
    }
  }

  function endBusy(reason = "complete") {
    state.busy = false;
    stopPolling();
    refreshFilterAvailability("fetch-" + reason);
  }

  function beginBusy(reason = "fetch") {
    state.busy = true;
    state.busyStartedAt = Date.now();
    state.lastMutationAt = Date.now();
    setFilterAvailability(false, reason);

    stopPolling();

    state.pollTimer = window.setInterval(() => {
      if (!state.busy) return;

      const elapsed = Date.now() - state.busyStartedAt;
      const quietMs = Date.now() - state.lastMutationAt;

      if (elapsed > 1800 && quietMs > 900 && areFetchButtonsIdle() && cancelLooksIdle()) {
        endBusy("buttons-idle");
        return;
      }

      if (elapsed > 90000 && areFetchButtonsIdle()) {
        endBusy("watchdog");
      }
    }, 500);

    state.unlockTimer = window.setTimeout(() => {
      if (state.busy && areFetchButtonsIdle()) {
        endBusy("max-timeout");
      }
    }, 180000);
  }

  function isFetchClickTarget(target) {
    const button = target?.closest?.("button");

    if (!button) return false;

    const id = normalize(button.id);
    const text = textOf(button);
    const i18n = normalize(button.dataset?.i18n);

    if (id === "explorerfetch" || id === "explorerforcefetch") return true;
    if (i18n.includes("fetch")) return true;
    if (text === "fetch" || text === "force fetch") return true;

    return false;
  }

  function isCancelClickTarget(target) {
    const button = target?.closest?.("button");

    if (!button) return false;

    const id = normalize(button.id);
    const text = textOf(button);
    const i18n = normalize(button.dataset?.i18n);

    return id === "explorercancel" || text === "cancel" || i18n.includes("cancel");
  }

  function installInvokeCompletionHook() {
    /* KGW_EXPLORER_READONLY_TAURI_INVOKE_SAFE_V1
       Tauri's invoke object can be read-only. Do not monkey-patch it.
       Fetch lifecycle is tracked through button state, DOM mutations,
       cancel clicks, custom events, and watchdog timers instead. */
    const tauriCore = window.__TAURI__ && (window.__TAURI__.core || window.__TAURI__.tauri);

    if (!tauriCore || typeof tauriCore.invoke !== "function") {
      return;
    }

    document.documentElement.dataset.kgwExplorerInvokeReadonlySafeV1 = "true";
  }

  function install() {
    installInvokeCompletionHook();

    document.addEventListener("change", (event) => {
      if (isAddressControl(event.target)) {
        window.setTimeout(() => refreshFilterAvailability("address-change"), 0);
      }
    }, true);

    document.addEventListener("input", (event) => {
      if (isAddressControl(event.target)) {
        window.setTimeout(() => refreshFilterAvailability("address-input"), 0);
      }
    }, true);

    document.addEventListener("click", (event) => {
      if (!isExplorerVisible()) return;

      if (isFetchClickTarget(event.target)) {
        beginBusy("fetch-click");
        return;
      }

      if (isCancelClickTarget(event.target)) {
        window.setTimeout(() => endBusy("cancel-click"), 150);
      }
    }, true);

    for (const eventName of [
      "kgw:explorer-fetch-complete",
      "kgw:explorer-fetch-failed",
      "kgw:explorer-fetch-cancelled",
      "kgw:transactions-loaded",
      "kgw:tab-opened",
      "kgw:tab-opened-after-mount"
    ]) {
      window.addEventListener(eventName, () => {
        if (eventName.includes("fetch") || eventName.includes("transactions")) {
          window.setTimeout(() => endBusy(eventName), 150);
        } else {
          window.setTimeout(() => refreshFilterAvailability(eventName), 150);
        }
      });
    }

    const observer = new MutationObserver(() => {
      if (state.busy) {
        state.lastMutationAt = Date.now();
      } else {
        window.setTimeout(() => refreshFilterAvailability("dom-mutation"), 80);
      }
    });

    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["disabled", "aria-disabled", "style", "class", "value"]
    });

    window.setInterval(() => {
      installInvokeCompletionHook();

      if (!state.busy) {
        refreshFilterAvailability("periodic");
      }
    }, 1000);

    window.kgwRefreshExplorerFilterAvailabilityV1 = refreshFilterAvailability;
    window.kgwSetExplorerFilterBusyV1 = function kgwSetExplorerFilterBusyV1(value, reason = "manual") {
      if (value) {
        beginBusy(reason);
      } else {
        endBusy(reason);
      }
    };

    refreshFilterAvailability("install");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", install, { once: true });
  } else {
    install();
  }
})();

/* KGW_EXPLORER_ADDRESS_DIAGNOSTICS_V1
 * Rust/WASM owns address/name/balance behavior.
 * JavaScript exposes diagnostics only; no business callbacks remain.
 */
window.__kgwExplorerAddressDiagnostics = {
  log(label, details) {
    microscopeLog(label, details);
  },
  warn(label, details) {
    microscopeWarn(label, details);
  },
  error(label, details) {
    const message = details?.message || String(details || "unknown error");
    microscopeError(label, new Error(message), details);
  },
  apiShape(label, details) {
    microscopeApiShape(label, details);
  }
};

window.kgwExplorerSaveManualAddress = (section = root()) => kgwExplorerSaveManualAddress(section);
window.kgwInstallExplorerManualAddressSave = () => kgwInstallExplorerManualAddressSave();
kgwInstallExplorerManualAddressSave();
