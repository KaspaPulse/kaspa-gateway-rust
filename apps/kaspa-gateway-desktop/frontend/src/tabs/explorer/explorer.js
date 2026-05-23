function kgwI18nTextR41(key, fallback) {
  try {
    if (window.kgwT && typeof window.kgwT === "function") return window.kgwT(key, fallback);
    if (window.KGW_I18N && typeof window.KGW_I18N.t === "function") return window.KGW_I18N.t(key, fallback);
    if (window.i18n && typeof window.i18n.t === "function") return window.i18n.t(key, fallback);
  } catch (_) {
  }
  return fallback;
}

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

import { parseHeaderUsdPrice } from "./explorer.header.js";
import { openBlockExplorer, exportCsv, exportHtml, exportPdf } from "./explorer.export.js";
import { normalizeDateInputValue, parseDateSeconds, kgwDayToEpochSeconds, kgwTxDayToEpochSeconds, kgwClean2DayToSeconds, kgwTransactionDateKey } from "./explorer.date.js";
import { formatKas, formatUsd, kgwSummaryFormatKas, kgwSummaryFormatUsd, kgwClean2Kas, kgwClean2Usd } from "./explorer.formatting.js";
import { toEnglishDigits, pick, toNumber, kgwClean2SafeText } from "./explorer.utils.js";

const SOMPI_PER_KAS = 100_000_000;

const explorerState = {
  rows: [],
  filteredRows: [],
  savedAddresses: [],
  addressNames: new Map(),
  addressNamesLoaded: false,
  selectedAddress: "",
  busy: false,
  fontSize: 11,
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

async function tryInvokeMany(candidates) {
  let lastError = null;

  for (const item of candidates) {
    try {
      return await invokeCommand(item.cmd, item.args || {});
    } catch (error) {
      lastError = error;
    }
  }

  if (lastError) throw lastError;
  return null;
}
function setStatus(section, message) {
  const cleanMessage = String(message || "");

  const node = qs("#explorerStatus", section);
  if (node) node.textContent = cleanMessage;

  if (typeof window.kgwSetGlobalFetchProgressText === "function") {
    window.kgwSetGlobalFetchProgressText(cleanMessage);
  }

  console.info("[KGW Explorer]", message);
}


/* TX_LOAD_ADDRESS_PYTHON_STYLE_OWNER
   One visual owner only. Do not create/move a grid.
   Keep the native datalist attached to #explorerAddress so the dropdown width follows the input.
*/
function kgwApplyPythonLoadAddressStyle(section) {
  const root =
    section ||
    document.querySelector("#explorer") ||
    document.querySelector(".explorer-python-root") ||
    document;

  const input = qs("#explorerAddress", root);
  const balance = qs("#explorerBalanceValue", root);
  const usd = qs("#explorerBalanceUsdValue", root);
  const name = qs("#explorerAddressNameValue", root);

  if (!input || !balance) return;

  const fieldset = input.closest("fieldset");
  if (!fieldset) return;

  fieldset.classList.add("kgw-python-load-address-owner");

  const buttons = [
    qs("#explorerFetch", root),
    qs("#explorerForceFetch", root),
    qs("#explorerOpenExplorer", root),
    qs("#explorerCancel", root)
  ].filter(Boolean);

  for (const button of buttons) {
    button.classList.add("kgw-python-load-address-button");
  }

  balance.classList.add("kgw-python-balance-kas");

  if (usd) {
    usd.classList.add("kgw-python-balance-fiat");
  }

  if (name) {
    name.classList.add("kgw-python-wallet-name");
  }
}

/* KGW_EXPLORER_BALANCE_3_DECIMAL_FORMAT_OWNER_V1 */
function kgwFormatExplorerBalanceKasV1(value) {
  const raw = String(value ?? "").trim();

  if (!raw || raw === "N/A") {
    return "N/A";
  }

  const number = Number(raw.replace(/,/g, "").replace(/\s*KAS\s*$/i, ""));

  if (!Number.isFinite(number)) {
    return raw;
  }

  return `${number.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 3
  })} KAS`;
}


function setBalance(section, value, balanceKas = null) {
  microscopeLog("SET BALANCE", { value, balanceKas, headerPriceUsd: typeof parseHeaderUsdPrice === "function" ? parseHeaderUsdPrice() : null });

  const node = qs("#explorerBalanceValue", section);
  if (node) node.textContent = kgwFormatExplorerBalanceKasV1(balanceKas ?? value);

  const usdNode = qs("#explorerBalanceUsdValue", section);
  if (!usdNode) return;

  const numericBalance =
    Number.isFinite(Number(balanceKas))
      ? Number(balanceKas)
      : Number(String(value || "").replace(/[^0-9.-]/g, ""));

  const priceUsd = parseHeaderUsdPrice();

  if (Number.isFinite(numericBalance) && numericBalance > 0 && priceUsd > 0) {
    usdNode.textContent = `(${formatUsd(numericBalance * priceUsd)} USD)`;
    usdNode.hidden = false;
    usdNode.style.display = "block";
  } else {
    usdNode.textContent = "";
    usdNode.hidden = true;
  }

  kgwApplyPythonLoadAddressStyle(section);
}


function setAddressName(section, name) {
  microscopeLog("SET ADDRESS NAME", { name });

  const node = qs("#explorerAddressNameValue", section);
  if (!node) return;

  const clean = String(name || "").trim();

  node.textContent = clean;
  node.title = clean;
  node.hidden = !clean;
  node.style.display = clean ? "block" : "none";

  const balanceNode = qs("#explorerBalanceValue", section);
  if (balanceNode && clean) {
    balanceNode.title = `Wallet: ${clean}`;
  }

  kgwApplyPythonLoadAddressStyle(section);
}


function isKaspaAddress(value) {
  return /^kaspa(test|dev|sim)?:[a-z0-9]{50,}$/i.test(String(value || "").trim());
}

function normalizeAddress(value) {
  return String(value || "").trim();
}

function addressLookupKeys(address) {
  const clean = normalizeAddress(address);
  const lower = clean.toLowerCase();
  const noPrefix = clean.replace(/^kaspa(test|dev|sim)?:/i, "");
  const noPrefixLower = noPrefix.toLowerCase();

  return Array.from(new Set([
    clean,
    lower,
    noPrefix,
    noPrefixLower,
    noPrefix ? `kaspa:${noPrefix}` : "",
    noPrefixLower ? `kaspa:${noPrefixLower}` : ""
  ].filter(Boolean)));
}

function storeKnownName(address, name) {
  const cleanName = String(name || "").trim();
  if (!cleanName) return;

  for (const key of addressLookupKeys(address)) {
    explorerState.addressNames.set(key, cleanName);
  }
}

async function loadKnownAddressNames() {
  microscopeLog("NAMES LOAD START", {
    alreadyLoaded: explorerState.addressNamesLoaded,
    currentCount: explorerState.addressNames?.size || 0
  });

  if (explorerState.addressNamesLoaded) {
    microscopeLog("NAMES LOAD SKIP CACHE", {
      count: explorerState.addressNames?.size || 0
    });
    return explorerState.addressNames;
  }
  explorerState.addressNamesLoaded = true;

  try {
    const json =
      (await invokeCommand("top_addresses_load_known_names", {}).catch(() => null)) ||
      (await invokeCommand("get_all_addresses", {}).catch(() => null));

    if (!json) return explorerState.addressNames;
    microscopeApiShape("NAMES API RAW SHAPE", json);

    if (json && typeof json === "object" && !Array.isArray(json)) {
      for (const [address, value] of Object.entries(json)) {
        if (typeof value === "string") {
          storeKnownName(address, value);
        } else if (value && typeof value === "object") {
          storeKnownName(address, value.name || value.known_name || value.label || value.display_name || "");
        }
      }
    }

    const list = Array.isArray(json)
      ? json
      : Array.isArray(json?.addresses)
        ? json.addresses
        : Array.isArray(json?.items)
          ? json.items
          : Array.isArray(json?.names)
            ? json.names
            : [];

    for (const item of list) {
      if (Array.isArray(item)) {
        storeKnownName(item[0], item[1]);
        continue;
      }

      if (item && typeof item === "object") {
        storeKnownName(
          item.address || item.Address || item.addr || item.kaspa_address || "",
          item.name || item.known_name || item.KnownName || item.label || item.display_name || ""
        );
      }
    }
  } catch (error) {
    console.warn("[KGW Explorer] Failed to load /addresses/names", error);
  }

  microscopeLog("NAMES LOAD DONE", {
    count: explorerState.addressNames?.size || 0,
    sample: Array.from(explorerState.addressNames?.entries?.() || []).slice(0, 8)
  });

  return explorerState.addressNames;
}


/* TX_EXPLORER_TOP_ADDRESS_NAME_SOURCE_1
   Reuse the same network-backed Top Addresses source for names.
   Explorer first checks saved-address names; if missing, it asks fetch_top_addresses_rust
   and caches known_name by all normalized address lookup keys.
*/
async function loadTopAddressNamesForExplorer() {
  microscopeLog("TOP NAME SOURCE LOAD START", {
    alreadyLoaded: explorerState.topAddressNamesLoaded === true,
    currentCount: explorerState.addressNames?.size || 0
  });

  if (explorerState.topAddressNamesLoaded === true) {
    return explorerState.addressNames;
  }

  explorerState.topAddressNamesLoaded = true;

  try {
    const result = await tryInvokeMany([
      { cmd: "fetch_top_addresses_rust", args: { limit: 10000 } },
      { cmd: "fetch_top_addresses_rust", args: { request: { limit: 10000 } } }
    ]);

    const rows = Array.isArray(result)
      ? result
      : (Array.isArray(result?.rows) ? result.rows : []);

    let stored = 0;

    for (const row of rows) {
      const address = String(row?.address || row?.Address || "").trim();
      const name = String(
        row?.known_name ||
        row?.KnownName ||
        row?.["Known Name"] ||
        row?.name ||
        row?.label ||
        ""
      ).trim();

      if (!address || !name) continue;

      storeKnownName(address, name);
      stored += 1;
    }

    microscopeLog("TOP NAME SOURCE LOAD DONE", {
      rows: rows.length,
      stored,
      totalNames: explorerState.addressNames?.size || 0
    });
  } catch (error) {
    microscopeWarn("TOP NAME SOURCE LOAD FAILED", {
      message: error?.message || String(error)
    });
  }

  return explorerState.addressNames;
}

async function resolveKnownName(address) {
  microscopeLog("RESOLVE NAME START", { address });

  const map = await loadKnownAddressNames();

  for (const key of addressLookupKeys(address)) {
    const value = map.get(key);
    if (value) {
      microscopeLog("RESOLVE NAME HIT SAVED", { address, key, value });
      return value;
    }
  }

  await loadTopAddressNamesForExplorer();

  for (const key of addressLookupKeys(address)) {
    const value = explorerState.addressNames?.get?.(key);
    if (value) {
      microscopeLog("RESOLVE NAME HIT TOP", { address, key, value });
      return value;
    }
  }

  microscopeCheckAddressNameMatch(address);
  microscopeWarn("RESOLVE NAME MISS", { address });
  return "";
}


const explorerSaveAddressMemo = new Map();

function saveAddressMemoKey(address, name) {
  return `${String(address || "").trim()}|${String(name || "").trim()}`;
}

function shouldSkipAddressSave(address, name) {
  const key = saveAddressMemoKey(address, name);
  const now = Date.now();
  const last = explorerSaveAddressMemo.get(key) || 0;

  if (now - last < 10_000) {
    microscopeLog("SAVE ADDRESS SKIPPED DUPLICATE", { address, name });
    return true;
  }

  explorerSaveAddressMemo.set(key, now);
  return false;
}

async function saveAddressToDatabase(address, name = "") {
  const cleanAddress = normalizeAddress(address);
  const cleanName = String(name || "").trim();

  if (!isKaspaAddress(cleanAddress)) return "";

  if (shouldSkipAddressSave(cleanAddress, cleanName)) {
    return "";
  }

  try {
    const result = await invokeCommand("save_address", {
      address: cleanAddress,
      name: cleanName
    });

    microscopeLog("SAVE ADDRESS DONE", {
      address: cleanAddress,
      name: cleanName,
      result
    });

    return result || "";
  } catch (error) {
    microscopeError("SAVE ADDRESS FAILED", error, {
      address: cleanAddress,
      name: cleanName
    });
    throw error;
  }
}


async function refreshAddressName(section, address) {
  microscopeLog("REFRESH NAME START", { address });
  const knownName = await resolveKnownName(address);
  setAddressName(section, knownName);

  if (knownName) {
    await saveAddressToDatabase(address, knownName);
  }

  microscopeLog("REFRESH NAME DONE", { address, knownName });
  return knownName;
}

/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_EXPLORER_OWNER_START */
/* KGW_CALENDAR_EXISTING_OWNER_REGEX_FIX_R4: fixed regex escaping only; no new owner layer.\n * KGW_CALENDAR_EXISTING_OWNER_DOM_CSS_FIX_R6: body-attached compact popover inside existing owner.\n * KGW_CALENDAR_SCOPED_POPOVER_OWNER_FIX_R7: scope-isolated popovers per tab.\n * KGW_CALENDAR_SINGLE_ACTIVE_POPOVER_FIX_R8: removes stale body-attached popovers before opening current tab calendar.
/* */
function kgwCalendarPad2(value) {
  return String(value).padStart(2, "0");
}

function kgwCalendarTodayIso() {
  const now = new Date();
  return `${now.getFullYear()}-${kgwCalendarPad2(now.getMonth() + 1)}-${kgwCalendarPad2(now.getDate())}`;
}

function kgwCalendarIsoFromDate(date) {
  return `${date.getFullYear()}-${kgwCalendarPad2(date.getMonth() + 1)}-${kgwCalendarPad2(date.getDate())}`;
}

function kgwCalendarParseIso(value, fallbackValue) {
  const clean = normalizeDateInputValue(value || fallbackValue || kgwCalendarTodayIso());
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(clean);
  if (!match) return kgwCalendarParseIso(fallbackValue || kgwCalendarTodayIso(), kgwCalendarTodayIso());

  const year = Number(match[1]);
  const month = Number(match[2]) - 1;
  const day = Number(match[3]);
  const date = new Date(year, month, day);

  if (date.getFullYear() !== year || date.getMonth() !== month || date.getDate() !== day) {
    return kgwCalendarParseIso(fallbackValue || kgwCalendarTodayIso(), kgwCalendarTodayIso());
  }

  return date;
}

function kgwCalendarEnglishMonthLabel(year, monthIndex) {
  return new Intl.DateTimeFormat("en-US", {
    month: "long",
    year: "numeric",
    calendar: "gregory",
    numberingSystem: "latn"
  }).format(new Date(year, monthIndex, 1));
}

function kgwCalendarEscapeSelector(id) {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") return CSS.escape(id);
  return String(id).replace(/["\\]/g, "\\$&");
}

function kgwCalendarSetTextNative(textInput, nativeInput, value) {
  const clean = normalizeDateInputValue(value);
  if (textInput) {
    textInput.value = clean;
    textInput.dispatchEvent(new Event("input", { bubbles: true }));
    textInput.dispatchEvent(new Event("change", { bubbles: true }));
  }
  if (nativeInput && /^\d{4}-\d{2}-\d{2}$/.test(clean)) {
    nativeInput.value = clean;
    nativeInput.dispatchEvent(new Event("change", { bubbles: true }));
  }
  return clean;
}

function kgwCalendarResolveRange(section, textId) {
  const pairs = [
    ["explorerFromDate", "explorerToDate"],
    ["analysisFromDate", "analysisToDate"]
  ];

  const pair = pairs.find(([fromId, toId]) => textId === fromId || textId === toId);
  if (!pair) return null;

  const fromInput = section.querySelector(`#${kgwCalendarEscapeSelector(pair[0])}`);
  const toInput = section.querySelector(`#${kgwCalendarEscapeSelector(pair[1])}`);
  const fromNative = section.querySelector(`#${kgwCalendarEscapeSelector(pair[0] + "Native")}`);
  const toNative = section.querySelector(`#${kgwCalendarEscapeSelector(pair[1] + "Native")}`);

  if (!fromInput || !toInput) return null;

  return { fromInput, toInput, fromNative, toNative };
}

function kgwCalendarClose(section, scope = "explorer") {
  document.querySelectorAll(`.kgw-calendar-popover[data-kgw-calendar-scope="${scope}"]`).forEach((node) => node.remove());

  const owner = section || document;
  owner.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    if (!node.dataset.kgwCalendarScope || node.dataset.kgwCalendarScope === scope) {
      delete node.dataset.kgwCalendarOpen;
      delete node.dataset.kgwCalendarScope;
    }
  });
}

function kgwCalendarApplyPreset(section, textId, presetName) {
  const today = kgwCalendarParseIso(kgwCalendarTodayIso());
  const range = kgwCalendarResolveRange(section, textId);

  let from = kgwCalendarIsoFromDate(today);
  let to = kgwCalendarIsoFromDate(today);

  if (presetName === "last7") {
    const start = new Date(today);
    start.setDate(start.getDate() - 6);
    from = kgwCalendarIsoFromDate(start);
  } else if (presetName === "last30") {
    const start = new Date(today);
    start.setDate(start.getDate() - 29);
    from = kgwCalendarIsoFromDate(start);
  } else if (presetName === "thisMonth") {
    from = kgwCalendarIsoFromDate(new Date(today.getFullYear(), today.getMonth(), 1));
  } else if (presetName === "sinceLaunch") {
    from = "2021-11-07";
  }

  if (range) {
    kgwCalendarSetTextNative(range.fromInput, range.fromNative, from);
    kgwCalendarSetTextNative(range.toInput, range.toNative, to);
  } else {
    const activeInput = section.querySelector(`#${kgwCalendarEscapeSelector(textId)}`);
    kgwCalendarSetTextNative(activeInput, null, presetName === "today" ? to : from);
  }

  kgwCalendarClose(section, "explorer");
}

function kgwCalendarAttachPopover(popover, anchor) {
  document.querySelectorAll(".kgw-calendar-popover").forEach((node) => node.remove());
  document.querySelectorAll("[data-kgw-calendar-open='1']").forEach((node) => {
    delete node.dataset.kgwCalendarOpen;
    delete node.dataset.kgwCalendarScope;
  });

  popover.dataset.kgwCalendarScope = "explorer";
  popover.classList.add("kgw-calendar-popover-explorer");

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
function kgwCalendarI18nText(key, fallback) {
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

function kgwCalendarOpen(section, textInput, nativeInput, textId, fallbackValue) {
  if (!section || !textInput) return;

  const host = textInput.closest(".explorer-date-combo, .kgw-analysis-date-field") || textInput.parentElement || section;
  const wasOpen = host.dataset.kgwCalendarOpen === "1";
  kgwCalendarClose(section, "explorer");
  if (wasOpen) return;

  host.dataset.kgwCalendarOpen = "1";
  host.dataset.kgwCalendarScope = "explorer";

  let activeDate = kgwCalendarParseIso(textInput.value, fallbackValue);
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
    title.textContent = kgwCalendarEnglishMonthLabel(displayYear, displayMonth);

    const next = document.createElement("button");
    next.type = "button";
    next.className = "kgw-calendar-nav";
    next.textContent = "›";
    next.setAttribute("aria-label", "Next month");

    prev.addEventListener("click", (event) => {
      event.preventDefault();
      kgwExplorerUiTraceR53B3("explorer-calendar", "r53b3-explorer-calendar-prev-click", {
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
      kgwExplorerUiTraceR53B3("explorer-calendar", "r53b3-explorer-calendar-next-click", {
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
        kgwExplorerUiTraceR53B3("explorer-calendar", "r53b3-explorer-calendar-preset-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(textId || ""),
          preset: String(value || ""),
          label: String(label || "")
        });
        kgwCalendarApplyPreset(section, textId, value);
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

    const selectedIso = normalizeDateInputValue(textInput.value || fallbackValue || kgwCalendarTodayIso());
    const todayIso = kgwCalendarTodayIso();

    for (let day = 1; day <= daysInMonth; day++) {
      const date = new Date(displayYear, displayMonth, day);
      const iso = kgwCalendarIsoFromDate(date);
      const button = document.createElement("button");
      button.type = "button";
      button.className = "kgw-calendar-day";
      button.textContent = String(day);
      button.dataset.iso = iso;

      if (iso === selectedIso) button.classList.add("is-selected");
      if (iso === todayIso) button.classList.add("is-today");

      button.addEventListener("click", (event) => {
        event.preventDefault();
        kgwExplorerUiTraceR53B3("explorer-calendar", "r53b3-explorer-calendar-day-click", {
          trusted: Boolean(event && event.isTrusted),
          textId: String(textId || ""),
          iso: String(iso || "")
        });
        kgwCalendarSetTextNative(textInput, nativeInput, iso);
        kgwCalendarClose(section, "explorer");
      });

      grid.append(button);
    }

    const footer = document.createElement("div");
    footer.className = "kgw-calendar-footer";

    const close = document.createElement("button");
    close.type = "button";
    close.className = "kgw-calendar-close";
    close.textContent = kgwCalendarI18nText("calendar.close", "Close");
    close.addEventListener("click", (event) => {
      event.preventDefault();
      kgwExplorerUiTraceR53B3("explorer-calendar", "r53b3-explorer-calendar-close-click", {
        trusted: Boolean(event && event.isTrusted),
        textId: String(textId || "")
      });
      kgwCalendarClose(section, "explorer");
    });

    footer.append(close);
    popover.append(header, presets, weekdays, grid, footer);
  }

  render();
  kgwCalendarAttachPopover(popover, textInput);
}

function kgwSetEnglishDateValue(textInput, nativeInput, value) {
  const clean = normalizeDateInputValue(value);
  if (textInput) textInput.value = clean;
  if (nativeInput && /^\d{4}-\d{2}-\d{2}$/.test(clean)) nativeInput.value = clean;
  return clean;
}

function kgwBindDateControl(section, textId, nativeId, buttonId, fallbackValue) {
  const textInput = qs(`#${textId}`, section);
  const nativeInput = qs(`#${nativeId}`, section);
  const button = qs(`#${buttonId}`, section);

  if (!textInput || !button) return;

  kgwSetEnglishDateValue(textInput, nativeInput, textInput.value || fallbackValue);

  if (textInput.dataset.kgwDateBound === "1") return;
  textInput.dataset.kgwDateBound = "1";

  textInput.setAttribute("lang", "en-US");
  textInput.setAttribute("dir", "ltr");
  if (nativeInput) {
    nativeInput.setAttribute("lang", "en-US");
    nativeInput.setAttribute("dir", "ltr");
  }

  textInput.addEventListener("input", (event) => {
    kgwExplorerUiTraceR53B3("explorer-date", "r53b3-explorer-date-text-input", {
      trusted: Boolean(event && event.isTrusted),
      textId: String(textId || ""),
      value: String(textInput.value || "")
    });
    const clean = normalizeDateInputValue(textInput.value);
    if (clean !== textInput.value) textInput.value = clean;
    if (nativeInput && /^\d{4}-\d{2}-\d{2}$/.test(clean)) nativeInput.value = clean;
  });

  textInput.addEventListener("blur", (event) => {
    kgwExplorerUiTraceR53B3("explorer-date", "r53b3-explorer-date-text-blur", {
      trusted: Boolean(event && event.isTrusted),
      textId: String(textId || ""),
      value: String(textInput.value || "")
    });
    kgwSetEnglishDateValue(textInput, nativeInput, textInput.value || fallbackValue);
  });

  nativeInput?.addEventListener("change", (event) => {
    kgwExplorerUiTraceR53B3("explorer-date", "r53b3-explorer-date-native-change", {
      trusted: Boolean(event && event.isTrusted),
      textId: String(textId || ""),
      nativeId: String(nativeId || ""),
      value: String(nativeInput.value || "")
    });
    kgwSetEnglishDateValue(textInput, nativeInput, nativeInput.value || fallbackValue);
  });

  button.setAttribute("lang", "en-US");
  button.setAttribute("dir", "ltr");
  button.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    kgwExplorerUiTraceR53B3("explorer-calendar", "r53b3-explorer-calendar-button-click", {
      trusted: Boolean(event && event.isTrusted),
      textId: String(textId || ""),
      nativeId: String(nativeId || ""),
      buttonId: String(buttonId || "")
    });
    kgwSetEnglishDateValue(textInput, nativeInput, textInput.value || fallbackValue);
    kgwCalendarOpen(section, textInput, nativeInput, textId, fallbackValue);
  });
}
/* KGW_CALENDAR_EXISTING_OWNER_REBUILD_R2_EXPLORER_OWNER_END */

function kgwApplyExplorerFontSize(section, rawValue) {
  const clean = toEnglishDigits(rawValue ?? "9").replace(/[^0-9]/g, "").slice(0, 2);
  const value = Number(clean || "9");
  explorerState.fontSize = Math.max(6, Math.min(24, Number.isFinite(value) ? value : 9));
  setTableFontSize(section);
}

function kgwBindFontSpinbox(section) {
  const input = qs("#explorerTableFontSize", section);
  const dec = qs("#explorerTableFontDecrease", section);
  const inc = qs("#explorerTableFontIncrease", section);

  if (!input || input.dataset.kgwFontBound === "1") return;
  input.dataset.kgwFontBound = "1";

  input.addEventListener("input", () => {
    kgwApplyExplorerFontSize(section, input.value);
  });

  input.addEventListener("blur", () => {
    kgwApplyExplorerFontSize(section, input.value);
  });

  dec?.addEventListener("click", () => {
    kgwApplyExplorerFontSize(section, Number(explorerState.fontSize || input.value || 11) - 1);
  });

  inc?.addEventListener("click", () => {
    kgwApplyExplorerFontSize(section, Number(explorerState.fontSize || input.value || 11) + 1);
  });
}

function defaultDates(section) {
  const now = new Date();
  const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;

  kgwBindDateControl(
    section,
    "explorerFromDate",
    "explorerFromDateNative",
    "explorerFromDatePicker",
    "2021-11-07"
  );

  kgwBindDateControl(
    section,
    "explorerToDate",
    "explorerToDateNative",
    "explorerToDatePicker",
    today
  );
}


/* KGW_EXPLORER_LOCAL_BUSY_CONTROLS_POLICY
   Correct fetch busy policy:
   - Only controls inside #explorer are disabled.
   - Cancel remains enabled while fetch is running.
   - Shell/top tab navigation is handled by shell, not here.
*/
function kgwExplorerControlKey(control) {
  return [
    control?.id || "",
    control?.name || "",
    control?.className || "",
    control?.getAttribute?.("aria-label") || "",
    control?.getAttribute?.("title") || "",
    control?.textContent || ""
  ].join(" ").toLowerCase();
}

function kgwIsExplorerBusyAllowedControl(control) {
  if (!control) return false;

  const key = kgwExplorerControlKey(control);

  return (
    control.id === "explorerCancel" ||
    key.includes("cancel") ||
    key.includes("إلغاء") ||
    key.includes("الغاء")
  );
}

function kgwSetControlLocked(control, locked) {
  if (!control) return;

  const canDisable = "disabled" in control;

  if (locked) {
    if (control.dataset.kgwPrevDisabled === undefined) {
      control.dataset.kgwPrevDisabled = canDisable && control.disabled ? "true" : "false";
    }

    if (canDisable) {
      control.disabled = true;
    }

    control.setAttribute("aria-disabled", "true");
    control.classList.add("disabled");
    control.style.pointerEvents = "none";
    control.style.cursor = "not-allowed";
    return;
  }

  const previous = control.dataset.kgwPrevDisabled;

  if (previous !== undefined) {
    if (canDisable) {
      control.disabled = previous === "true";
    }

    delete control.dataset.kgwPrevDisabled;
  } else if (canDisable) {
    control.disabled = false;
  }

  control.removeAttribute("aria-disabled");
  control.classList.remove("disabled");
  control.classList.remove("is-disabled");
  control.style.pointerEvents = "";
  control.style.cursor = "";
}

function kgwApplyExplorerLocalBusyControls(section, busy) {
  const rootNode = section || root();

  if (!rootNode) return;

  const controls = Array.from(
    rootNode.querySelectorAll("button,input,select,textarea,a,[role='button']")
  );

  for (const control of controls) {
    if (kgwIsExplorerBusyAllowedControl(control)) {
      kgwSetControlLocked(control, false);

      if ("disabled" in control) {
        control.disabled = !busy;
      }

      continue;
    }

    kgwSetControlLocked(control, Boolean(busy));
  }

  const previousBusy = Boolean(window.__kgwExplorerFetchBusy);

  window.__kgwExplorerFetchBusy = Boolean(busy);

  if (previousBusy !== Boolean(busy)) {
    window.dispatchEvent(
      new CustomEvent("kgw:explorer-fetch-busy", {
        detail: { busy: Boolean(busy) }
      })
    );
  }

  if (typeof window.kgwApplyShellExplorerFetchBusyPolicy === "function") {
    window.kgwApplyShellExplorerFetchBusyPolicy(Boolean(busy));
  }
}

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

async function loadSavedAddresses(section) {
  const input = qs("#explorerAddress", section);
  const list = qs("#explorerAddressOptions", section);
  const isSavedSelect = input && input.tagName && input.tagName.toLowerCase() === "select";

  if (!input) return;
  if (!isSavedSelect && !list) return;

  try {
    const result = await tryInvokeMany([
      { cmd: "explorer_saved_addresses" },
      { cmd: "get_all_addresses" },
      { cmd: "list_addresses" }
    ]);

    const addresses = Array.isArray(result)
      ? result
      : Array.isArray(result?.addresses)
        ? result.addresses
        : Array.isArray(result?.items)
          ? result.items
          : [];

    explorerState.savedAddresses = addresses;

    if (isSavedSelect) {
      input.innerHTML = '<option value="">Select saved address...</option>';

      const seenSavedAddresses = new Set();

      for (const item of addresses) {
        const address = String(
          item?.address ||
          item?.kaspa_address ||
          item?.kaspaAddress ||
          item?.wallet_address ||
          ""
        ).trim();

        if (!address || !address.startsWith("kaspa:") || seenSavedAddresses.has(address)) continue;

        seenSavedAddresses.add(address);

        const option = document.createElement("option");
        option.value = address;

        const name = String(item?.name || item?.label || item?.alias || "").trim();
        option.textContent = name ? `${name} — ${address}` : address;

        input.appendChild(option);
      }

      return;
    }

    list.innerHTML = "";

    for (const item of addresses) {
      const address = String(item?.address || item?.kaspa_address || item?.kaspaAddress || item?.wallet_address || "").trim();
      const name = String(item.name || item.known_name || item.label || "").trim();

      if (!address) continue;

      const option = document.createElement("option");
      option.value = address;
      if (name) option.label = name;
      list.appendChild(option);

      if (name) storeKnownName(address, name);
    }

        // KGW_EXPLORER_ADDRESS_DROPDOWN_NO_AUTOFILL_V1
    // Keep the address input empty so the existing datalist can show saved addresses.
    // Do not auto-fill the first saved address here.
  } catch (error) {
    console.warn("[KGW Explorer] Failed to load saved addresses", error);
  }
  kgwInstallExplorerManualAddressSave();
}

async function fetchBalance(section, address) {
  microscopeLog("BALANCE FETCH START", { address });
  setBalance(section, "N/A");

  // TX_EXPLORER_REFRESH_NAME_DURING_BALANCE_1:
  // Manual Explorer addresses should resolve names the same way as Top Addresses.
  await refreshAddressName(section, address);

  try {
    const report = await tryInvokeMany([
      { cmd: "explorer_fetch_balance", args: { address } },
      { cmd: "explorer_fetch_balance", args: { request: { address } } },
      { cmd: "explorer_balance", args: { address } },
      { cmd: "explorer_balance", args: { request: { address } } }
    ]);

    microscopeApiShape("BALANCE RAW RESULT", report);

    const balanceKas =
      report?.balance_kas ??
      report?.balanceKas ??
      report?.balance ??
      (Number.isFinite(Number(report?.balance_sompi ?? report?.balanceSompi))
        ? Number(report?.balance_sompi ?? report?.balanceSompi) / SOMPI_PER_KAS
        : null);

    if (balanceKas === null || balanceKas === undefined || !Number.isFinite(Number(balanceKas))) {
      setBalance(section, "N/A");
      return null;
    }

    setBalance(section, `${formatKas(balanceKas)} KAS`, Number(balanceKas));
    microscopeLog("BALANCE FETCH DONE", { address, balanceKas: Number(balanceKas) });
    return balanceKas;
  } catch (error) {
    setBalance(section, "N/A");
    microscopeError("BALANCE FETCH FAILED", error, { address });
    console.warn("[KGW Explorer] balance fetch failed", error);
    return null;
  }
}

function extractRowsFromUnifiedResult(result) {
  microscopeApiShape("UNIFIED RESULT RAW SHAPE", result);

  const rows = [];
  const seen = new Set();
  const priceUsd = parseHeaderUsdPrice();

  function dateTimeFromTimestamp(value) {
    const raw = toNumber(value, 0);
    if (raw <= 0) return "";

    const ms = raw > 10_000_000_000 ? raw : raw * 1000;
    const d = new Date(ms);
    if (Number.isNaN(d.getTime())) return "";

    const pad = (v) => String(v).padStart(2, "0");

    return [
      d.getFullYear(),
      "-",
      pad(d.getMonth() + 1),
      "-",
      pad(d.getDate()),
      " ",
      pad(d.getHours()),
      ":",
      pad(d.getMinutes()),
      ":",
      pad(d.getSeconds())
    ].join("");
  }

  function normalizeTransaction(tx, day = "") {
    if (!tx || typeof tx !== "object") return null;

    const txid = String(pick(
      tx.txid,
      tx.transaction_id,
      tx.transactionId,
      tx.id,
      tx.hash
    ) || "").trim();

    if (!txid) return null;

    const timestampMs = toNumber(pick(
      tx.timestamp_ms,
      tx.timestampMs,
      tx.timestamp,
      tx.block_time,
      tx.blockTime
    ), 0);

    const amountKas = toNumber(
      pick(tx.amount_kas, tx.amountKas),
      Number.NaN
    );

    const amountSompi = toNumber(
      pick(tx.amount_sompi, tx.amountSompi),
      0
    );

    const amount = Number.isFinite(amountKas)
      ? amountKas
      : amountSompi / SOMPI_PER_KAS;

    const datetime = String(pick(
      tx.datetime,
      tx.date_time,
      tx.dateTime,
      dateTimeFromTimestamp(timestampMs)
    ) || "");

    const value = Number.isFinite(toNumber(tx.value, Number.NaN))
      ? toNumber(tx.value, 0)
      : Number.isFinite(Number(priceUsd))
        ? amount * Number(priceUsd)
        : 0;

    return {
      date: String(pick(tx.date, day, datetime.slice(0, 10)) || ""),
      datetime,
      txid,
      direction: String(pick(tx.direction, "unknown") || "unknown"),
      amount,
      value,
      type: String(pick(tx.type, tx.tx_type, tx.txType, "transfer") || "transfer"),
      from_address: pick(tx.from_address, tx.fromAddress, tx.from),
      to_address: pick(tx.to_address, tx.toAddress, tx.to),
      counterparty: pick(tx.counterparty, tx.counterParty),
      block_height: pick(tx.block_height, tx.blockHeight),
      timestamp_ms: timestampMs
    };
  }

  function pushRow(tx, day = "") {
    const row = normalizeTransaction(tx, day);
    if (!row) return;

    if (seen.has(row.txid)) return;
    seen.add(row.txid);

    rows.push(row);
  }

  function consumeGroups(groups) {
    for (const group of Array.isArray(groups) ? groups : []) {
      const day = String(group?.day || group?.date || "");
      const txs = Array.isArray(group?.transactions)
        ? group.transactions
        : Array.isArray(group?.rows)
          ? group.rows
          : [];

      for (const tx of txs) {
        pushRow(tx, day);
      }
    }
  }

  if (Array.isArray(result)) {
    const looksGrouped = result.some((item) => Array.isArray(item?.transactions));
    if (looksGrouped) {
      consumeGroups(result);
    } else {
      for (const tx of result) pushRow(tx);
    }
  } else {
    consumeGroups(result?.groups);
    for (const tx of Array.isArray(result?.rows) ? result.rows : []) pushRow(tx);
    for (const tx of Array.isArray(result?.transactions) ? result.transactions : []) pushRow(tx);
  }

  rows.sort((a, b) => {
    const bt = toNumber(b.timestamp_ms, 0);
    const at = toNumber(a.timestamp_ms, 0);
    if (bt !== at) return bt - at;
    return String(b.datetime || "").localeCompare(String(a.datetime || ""));
  });

  microscopeLog("UNIFIED RESULT NORMALIZED ROWS", {
    rows: rows.length,
    sample: rows.slice(0, 3)
  });

  return rows;
}

function normalizeTransactionRow(tx, day, priceUsd) {
  const amountKas = Number(
    tx.amount_kas ??
    tx.amountKas ??
    tx.amount ??
    (Number(tx.amount_sompi ?? tx.amountSompi ?? 0) / SOMPI_PER_KAS)
  );

  const valueUsd = Number(
    tx.value_usd ??
    tx.valueUsd ??
    (priceUsd ? Math.abs(amountKas) * priceUsd : 0)
  );

  return {
    date: String(tx.date || tx.day || day || "").slice(0, 10),
    datetime: String(tx.datetime || tx.time || tx.timestamp || tx.date || tx.day || day || "").replace("T", " ").slice(0, 19),
    txid: String(tx.txid || tx.id || tx.transaction_id || tx.transactionId || ""),
    direction: String(tx.direction || "unknown"),
    amount: Number.isFinite(amountKas) ? amountKas : 0,
    value: Number.isFinite(valueUsd) ? valueUsd : 0,
    type: String(tx.tx_type || tx.txType || tx.type || "transfer")
  };
}

function applyFilters(section) {
  const direction = String(qs("#explorerDirectionFilter", section)?.value || "ALL");
  const type = String(qs("#explorerTypeFilter", section)?.value || "ALL");
  const search = String(qs("#explorerSearch", section)?.value || "").trim().toLowerCase();

  explorerState.filteredRows = explorerState.rows.filter((row) => {
    if (direction !== "ALL" && row.direction !== direction) return false;
    if (type !== "ALL" && row.type !== type) return false;

    if (search) {
      const haystack = [
        row.datetime,
        row.txid,
        row.direction,
        row.amount,
        row.value,
        row.type
      ].join(" ").toLowerCase();

      if (!haystack.includes(search)) return false;
    }

    return true;
  });
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

function kgwTransactionAmountNumber(row) {
  const raw = row?.amountKas ?? row?.amount_kas ?? row?.amount ?? row?.kas ?? 0;
  const value = Number(String(raw).replace(/,/g, ""));
  return Number.isFinite(value) ? value : 0;
}

function kgwTransactionUsdNumber(row) {
  const raw = row?.valueUsd ?? row?.value_usd ?? row?.value ?? row?.usd ?? 0;
  const value = Number(String(raw).replace(/,/g, ""));
  return Number.isFinite(value) ? value : 0;
}

function kgwFormatGroupNumber(value, digits = 8) {
  const number = Number(value || 0);
  if (!Number.isFinite(number)) return "0";

  return number.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: digits
  });
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

function kgwDaySummaryRowsFromResult(result) {
  const items = Array.isArray(result)
    ? result
    : Array.isArray(result?.days)
      ? result.days
      : Array.isArray(result?.summaries)
        ? result.summaries
        : [];

  return items.map((item) => ({
    __kgwDaySummary: true,
    day: item.day || item.date || "",
    count: Number(item.count || item.tx_count || item.transactions_count || 0) || 0,
    incoming_kas: Number(item.incoming_kas || item.incomingKas || 0) || 0,
    outgoing_kas: Number(item.outgoing_kas || item.outgoingKas || 0) || 0,
    net_kas: Number(item.net_kas || item.netKas || 0) || 0
  })).filter((item) => item.day);
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
function kgwEnsureExplorerFilterOptions(section) {
  const typeEl = qs("#explorerTypeFilter", section);
  const directionEl = qs("#explorerDirectionFilter", section);

  if (typeEl) {
    const previous = String(typeEl.value || "ALL").toLowerCase();

    typeEl.innerHTML = `
      <option value="ALL">ALL</option>
      <option value="coinbase">coinbase</option>
      <option value="transfer">transfer</option>
    `;

    if (["all", "coinbase", "transfer"].includes(previous)) {
      typeEl.value = previous === "all" ? "ALL" : previous;
    } else {
      typeEl.value = "ALL";
    }
  }

  if (directionEl) {
    const previous = String(directionEl.value || "ALL").toLowerCase();

    directionEl.innerHTML = `
      <option value="ALL">ALL</option>
      <option value="incoming">incoming</option>
      <option value="outgoing">outgoing</option>
    `;

    if (["all", "incoming", "outgoing"].includes(previous)) {
      directionEl.value = previous === "all" ? "ALL" : previous;
    } else {
      directionEl.value = "ALL";
    }
  }

  console.log("[KGW Explorer][filter] canonical dropdown options installed", {
    typeOptions: typeEl ? Array.from(typeEl.options).map((option) => option.value) : [],
    directionOptions: directionEl ? Array.from(directionEl.options).map((option) => option.value) : []
  });
}

function kgwReadExplorerFilterState(section) {
  kgwEnsureExplorerFilterOptions(section);
  const typeEl = qs("#explorerTypeFilter", section);
  const directionEl = qs("#explorerDirectionFilter", section);
  const searchEl = qs("#explorerSearch", section);
  const fromEl = qs("#explorerFromDate", section);
  const toEl = qs("#explorerToDate", section);

  const state = {
    typeId: typeEl?.id || "",
    directionId: directionEl?.id || "",
    typeValue: typeEl?.value || "ALL",
    directionValue: directionEl?.value || "ALL",
    searchValue: searchEl?.value || "",
    fromValue: fromEl?.value || "",
    toValue: toEl?.value || ""
  };

  kgwFilterTrace("controls", state);

  return state;
}

function kgwBuildExplorerListRequest(section, address, startTs, endTs, limit = 10000) {
  const filterState = kgwReadExplorerFilterState(section);

  const request = {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: filterState.typeValue || "ALL",
    direction: filterState.directionValue || "ALL",
    search_query: filterState.searchValue || "",
    limit
  };

  kgwFilterTrace("request", request);

  return request;
}

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

function kgwRenderGroupedCollapsedTransactions(section, rows) {
  const body = qs("#explorerTransactionsBody", section);

  if (!body) return;

  const safeRows = Array.isArray(rows) ? rows : [];
  body.innerHTML = "";

  if (!safeRows.length) {
    body.innerHTML = `
      <tr>
        <td colspan="6" class="muted" data-i18n="explorer.noTransactionsToDisplay">No transactions to display.</td>
      </tr>
    `;
    return;
  }

  const summaryMode = safeRows.every((row) => row?.__kgwDaySummary === true);

  if (summaryMode) {
    const totalCount = safeRows.reduce((sum, row) => sum + (Number(row.count) || 0), 0);
    const totalNetKas = safeRows.reduce((sum, row) => sum + (Number(row.net_kas) || 0), 0);

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
    body.appendChild(totalTr);

    const fragment = document.createDocumentFragment();

    for (const summary of safeRows) {
      const day = summary.day;
      const expanded = window.__kgwExplorerExpandedDateGroups.has(day);
      const sign = expanded ? "-" : "+";

      const dayTr = document.createElement("tr");
      dayTr.className = "kgw-day-group-row";
      dayTr.dataset.kgwDateGroup = day;
      dayTr.innerHTML = `
        <td>${sign} ${day}</td>
        <td>${Number(summary.count || 0).toLocaleString()} transactions</td>
        <td></td>
        <td>${kgwSummaryFormatKas(Number(summary.net_kas || 0) || 0)}</td>
        <td></td>
        <td></td>
      `;

      fragment.appendChild(dayTr);

      if (!expanded) continue;

      const cachedRows = window.__kgwExplorerDayTransactionCache.get(day);

      if (!cachedRows) {
        const loadingTr = document.createElement("tr");
        loadingTr.className = "kgw-transaction-row";
        loadingTr.innerHTML = `
          <td colspan="6" class="muted">Loading transactions for ${day}...</td>
        `;
        fragment.appendChild(loadingTr);
        continue;
      }

      for (const row of cachedRows) {
        const txid = row?.txid || row?.transaction_id || row?.transactionId || "";

        const txTr = document.createElement("tr");
        txTr.className = "kgw-transaction-row";
        txTr.dataset.kgwParentDateGroup = day;
        txTr.innerHTML = `
          <td>${row?.datetime || row?.date_time || row?.time || row?.timestamp || ""}</td>
          <td>${txid}</td>
          <td>${row?.direction || ""}</td>
          <td>${kgwSummaryFormatKas(Number(row?.amount_kas ?? row?.amountKas ?? row?.amount ?? 0) || 0)}</td>
          <td>${kgwFormatUsd(Number(row?.value_usd ?? row?.valueUsd ?? row?.usd_value ?? row?.usdValue ?? kgwSummaryUsdForKas(row?.amount_kas ?? row?.amountKas ?? row?.amount ?? 0)) || 0)}</td>
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
          kgwRenderGroupedCollapsedTransactions(section, explorerState.filteredRows || explorerState.rows || []);
          return;
        }

        window.__kgwExplorerExpandedDateGroups.add(day);
        kgwRenderGroupedCollapsedTransactions(section, explorerState.filteredRows || explorerState.rows || []);

        if (!window.__kgwExplorerDayTransactionCache.has(day)) {
          try {
            const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", section)?.value);
            const dayRows = await kgwLoadTransactionsForSingleDayFromDb(section, address, day);

            window.__kgwExplorerDayTransactionCache.set(day, dayRows);

            microscopeLog("DAY TRANSACTIONS LOADED", {
              day,
              rows: dayRows.length
            });
          } catch (error) {
            microscopeWarn("DAY TRANSACTIONS LOAD FAILED", {
              day,
              message: error?.message || String(error)
            });

            window.__kgwExplorerDayTransactionCache.set(day, []);
          }

          kgwRenderGroupedCollapsedTransactions(section, explorerState.filteredRows || explorerState.rows || []);
        }
      });
    });

    return;
  }

  const grouped = new Map();

  for (const row of safeRows) {
    const dayKey = kgwTransactionDateKey(row) || "Unknown date";

    if (!grouped.has(dayKey)) {
      grouped.set(dayKey, {
        key: dayKey,
        rows: [],
        incoming: 0,
        outgoing: 0,
        usd: 0
      });
    }

    const group = grouped.get(dayKey);
    group.rows.push(row);

    const amountKas = Number(row?.amount_kas ?? row?.amountKas ?? row?.amount ?? 0) || 0;
    const usdValue = Number(row?.value_usd ?? row?.valueUsd ?? row?.usd_value ?? row?.usdValue ?? 0) || 0;
    const direction = String(row?.direction || "").toLowerCase();

    if (direction === "outgoing") {
      group.outgoing += Math.abs(amountKas);
    } else {
      group.incoming += Math.abs(amountKas);
    }

    group.usd += Math.abs(usdValue);
  }

  const groups = Array.from(grouped.values()).sort((left, right) => {
    return String(right.key).localeCompare(String(left.key));
  });

  const fragment = document.createDocumentFragment();

  for (const group of groups) {
    const expanded = window.__kgwExplorerExpandedDateGroups.has(group.key);
    const sign = expanded ? "-" : "+";

    const dayTr = document.createElement("tr");
    dayTr.className = "kgw-day-group-row";
    dayTr.dataset.kgwDateGroup = group.key;
    dayTr.innerHTML = `
      <td>${sign} ${group.key}</td>
      <td>${group.rows.length.toLocaleString()} transactions</td>
      <td></td>
      <td>${kgwSummaryFormatKas(group.incoming - group.outgoing)}</td>
      <td>${kgwFormatUsd(group.usd)}</td>
      <td></td>
    `;

    fragment.appendChild(dayTr);

    if (!expanded) continue;

    for (const row of group.rows) {
      const txid = row?.txid || row?.transaction_id || row?.transactionId || "";

      const txTr = document.createElement("tr");
      txTr.className = "kgw-transaction-row";
      txTr.dataset.kgwParentDateGroup = group.key;
      txTr.innerHTML = `
        <td>${row?.datetime || row?.date_time || row?.time || row?.timestamp || ""}</td>
        <td>${txid}</td>
        <td>${row?.direction || ""}</td>
        <td>${kgwSummaryFormatKas(Number(row?.amount_kas ?? row?.amountKas ?? row?.amount ?? 0) || 0)}</td>
        <td>${kgwFormatUsd(Number(row?.value_usd ?? row?.valueUsd ?? row?.usd_value ?? row?.usdValue ?? 0) || 0)}</td>
        <td>${row?.tx_type || row?.type || ""}</td>
      `;

      fragment.appendChild(txTr);
    }
  }

  body.appendChild(fragment);

  qsa("[data-kgw-date-group]", body).forEach((row) => {
    row.addEventListener("click", () => {
      const key = row.dataset.kgwDateGroup;

      if (!key) return;

      if (window.__kgwExplorerExpandedDateGroups.has(key)) {
        window.__kgwExplorerExpandedDateGroups.delete(key);
      } else {
        window.__kgwExplorerExpandedDateGroups.add(key);
      }

      kgwRenderGroupedCollapsedTransactions(
        section,
        explorerState.filteredRows || explorerState.rows || []
      );
    });
  });
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

function kgwSummarySafeText(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

/* KGW_FIX_DAY_SUMMARY_USD_TOTALS
   Collapsed day rows must show the total USD value for the day.
   The DB summary has KAS totals; USD is calculated from the current header price.
*/
function kgwSummaryCurrentUsdPrice() {
  const candidates = [
    typeof parseHeaderUsdPrice === "function" ? parseHeaderUsdPrice() : null,
    window.__kgwHeaderPriceUsd,
    window.__kgwLastKasPriceUsd,
    window.__kaspaPriceUsd,
    window.kaspaPriceUsd
  ];

  for (const candidate of candidates) {
    const number = Number(candidate);

    if (Number.isFinite(number) && number > 0) {
      return number;
    }
  }

  const priceText = document.body?.innerText?.match(/Price\s+([0-9.]+)\s+USD/i)?.[1];
  const parsed = Number(priceText);

  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

function kgwSummaryUsdForKas(valueKas) {
  const priceUsd = kgwSummaryCurrentUsdPrice();
  const kas = Math.abs(Number(valueKas || 0));

  if (!Number.isFinite(kas) || !Number.isFinite(priceUsd) || priceUsd <= 0) {
    return 0;
  }

  return kas * priceUsd;
}

function kgwNormalizeDaySummaries(result) {
  const items = Array.isArray(result)
    ? result
    : Array.isArray(result?.days)
      ? result.days
      : Array.isArray(result?.summaries)
        ? result.summaries
        : [];

  const priceUsd = kgwSummaryCurrentUsdPrice();

  return items
    .map((item) => {
      const incomingKas = Number(item?.incoming_kas ?? item?.incomingKas ?? 0) || 0;
      const outgoingKas = Number(item?.outgoing_kas ?? item?.outgoingKas ?? 0) || 0;
      const netKas = Number(item?.net_kas ?? item?.netKas ?? (incomingKas - outgoingKas)) || 0;

      // USD day total must represent total movement value, not only net.
      const grossKas = Math.abs(incomingKas) + Math.abs(outgoingKas);
      const kasForUsd = grossKas > 0 ? grossKas : Math.abs(netKas);

      const explicitUsd = Number(
        item?.value_usd ??
        item?.valueUsd ??
        item?.usd_value ??
        item?.usdValue ??
        item?.value
      );

      const valueUsd =
        Number.isFinite(explicitUsd) && explicitUsd > 0
          ? explicitUsd
          : priceUsd > 0
            ? kasForUsd * priceUsd
            : 0;

      return {
        __kgwDaySummary: true,
        day: String(item?.day || item?.date || "").slice(0, 10),
        count: Number(item?.count ?? item?.tx_count ?? item?.transactions_count ?? 0) || 0,
        incoming_kas: incomingKas,
        outgoing_kas: outgoingKas,
        net_kas: netKas,
        value_usd: valueUsd
      };
    })
    .filter((item) => item.day);
}

async function kgwLoadTransactionDaySummaries(section, address, startTs, endTs) {
  const request = kgwBuildExplorerListRequest(section, address, startTs, endTs, 10000);

  const started = performance.now();

  const result = await kgwInvokeExplorerDaySummaries(request);
  const rows = kgwNormalizeDaySummaries(result);

  const totalCount = rows.reduce((sum, row) => sum + (Number(row.count) || 0), 0);
  const totalUsd = rows.reduce((sum, row) => sum + (Number(row.value_usd) || 0), 0);

  kgwFilterTrace("day summaries response", {
    elapsedMs: Math.round(performance.now() - started),
    days: rows.length,
    totalCount,
    totalUsd,
    first: rows[0] || null,
    last: rows[rows.length - 1] || null
  });

  return rows;
}

async function kgwLoadTransactionsForDay(section, address, day) {
  const startTs = kgwSummaryDayToSeconds(day, false);
  const endTs = kgwSummaryDayToSeconds(day, true);

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
function kgwSummaryUsdForSummary(summary) {
  const explicit = Number(
    summary?.value_usd ??
    summary?.valueUsd ??
    summary?.usd_value ??
    summary?.usdValue ??
    0
  );

  if (Number.isFinite(explicit) && explicit > 0) {
    return explicit;
  }

  const incomingKas = Number(summary?.incoming_kas ?? summary?.incomingKas ?? 0) || 0;
  const outgoingKas = Number(summary?.outgoing_kas ?? summary?.outgoingKas ?? 0) || 0;
  const netKas = Number(summary?.net_kas ?? summary?.netKas ?? 0) || 0;
  const grossKas = Math.abs(incomingKas) + Math.abs(outgoingKas);
  const kasForUsd = grossKas > 0 ? grossKas : Math.abs(netKas);

  return kgwSummaryUsdForKas(kasForUsd);
}

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
      `<td>${sign} ${kgwSummarySafeText(day)}</td>` +
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
      loadingTr.innerHTML = `<td colspan="6" class="muted">Loading transactions for ${kgwSummarySafeText(day)}...</td>`;
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
        `<td>${kgwSummarySafeText(tx?.datetime || tx?.date_time || tx?.time || tx?.timestamp || "")}</td>` +
        `<td>${kgwSummarySafeText(txid)}</td>` +
        `<td>${kgwSummarySafeText(tx?.direction || "")}</td>` +
        `<td>${kgwSummaryFormatKas(amount)}</td>` +
        `<td>${kgwSummaryFormatUsd(usd)}</td>` +
        `<td>${kgwSummarySafeText(tx?.tx_type || tx?.type || "")}</td>`;

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
function kgwFilterLog(label, payload = {}) {
  try {
    console.log(`[KGW Explorer][filter-owner] ${label}`, payload);
  } catch (_) {
    console.log(`[KGW Explorer][filter-owner] ${label}`);
  }
}

function kgwFilterValue(selector, section, fallback = "ALL") {
  const element = qs(selector, section);
  const value = String(element?.value || fallback).trim();
  return value || fallback;
}

function kgwFilterBuildRequest(section, address, startTs, endTs, limit = 10000) {
  const request = {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: kgwFilterValue("#explorerTypeFilter", section, "ALL"),
    direction: kgwFilterValue("#explorerDirectionFilter", section, "ALL"),
    search_query: String(qs("#explorerSearch", section)?.value || ""),
    limit
  };

  kgwFilterLog("request", request);
  return request;
}

function kgwFilterTbody(section) {
  return qs("#explorerTransactionsBody", section) || document.querySelector("#explorerTransactionsBody");
}

async function kgwApplyFilterSingleOwner(section) {
  const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", section)?.value);
  const startTs = parseDateSeconds(qs("#explorerFromDate", section)?.value, false);
  const endTs = parseDateSeconds(qs("#explorerToDate", section)?.value, true);

  kgwFilterLog("start", {
    address,
    selectedAddress: explorerState.selectedAddress,
    type: kgwFilterValue("#explorerTypeFilter", section, "ALL"),
    direction: kgwFilterValue("#explorerDirectionFilter", section, "ALL"),
    search: String(qs("#explorerSearch", section)?.value || ""),
    startTs,
    endTs
  });

  if (!isKaspaAddress(address)) {
    clearExplorerTransactionTable(section, "Enter a valid Kaspa address.");
    kgwFilterLog("invalid address", { address });
    return;
  }

  setStatus(section, "Applying filter from local database...");

  const request = kgwFilterBuildRequest(section, address, startTs, endTs, 10000);
  const started = performance.now();

  const result = await kgwInvokeExplorerDaySummaries(request);
  const rows = kgwNormalizeDaySummaries(result);

  kgwFilterLog("rust response normalized", {
    elapsedMs: Math.round(performance.now() - started),
    days: rows.length,
    totalTransactions: rows.reduce((sum, row) => sum + (Number(row.count) || 0), 0),
    first: rows[0] || null
  });

  explorerState.selectedAddress = address;
  explorerState.rows = rows;
  explorerState.filteredRows = rows.slice();

  window.__kgwExplorerDayTransactionCache = new Map();
  window.__kgwExplorerExpandedDateGroups = new Set();

  await kgwRenderDaySummaries(
    section,
    rows,
    rows.length
      ? `Filter applied. Showing ${rows.length.toLocaleString()} days from local database. Click + to load one day.`
      : "Filter applied. No matching transaction days found."
  );

  const body = kgwFilterTbody(section);

  kgwFilterLog("render done", {
    tbodyFound: Boolean(body),
    childRows: body?.children?.length ?? null,
    textPreview: body?.innerText?.slice(0, 300) ?? ""
  });
}

function kgwRowsFromAnyGroupedResult(result) {
  if (Array.isArray(result)) {
    return extractRowsFromUnifiedResult({ groups: result });
  }

  return extractRowsFromUnifiedResult(result);
}

async function kgwLoadLocalTransactionsForAddress(section, address, startTs, endTs) {
  const listRequest = {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: qs("#explorerTypeFilter", section)?.value || "ALL",
    direction: qs("#explorerDirectionFilter", section)?.value || "ALL",
    search_query: qs("#explorerSearch", section)?.value || "",
    limit: 1000000
  };

  const result = await kgwInvokeExplorerGroupedTransactions(listRequest);

  return kgwRowsFromAnyGroupedResult(result);
}


/* KGW_DB_SOURCE_OF_TRUTH_TABLE
   Python parity rule:
   The table is rendered from Transactions DB, not from raw network results.
*/
function kgwTransactionListRequestFromUi(section, address, startTs, endTs) {
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

function kgwRowsFromDbGroupedResult(result) {
  if (!result) return [];

  if (Array.isArray(result)) {
    return extractRowsFromUnifiedResult({ groups: result });
  }

  if (Array.isArray(result.groups)) {
    return extractRowsFromUnifiedResult(result);
  }

  return extractRowsFromUnifiedResult(result);
}

async function kgwLoadRowsFromTransactionsDb(section, address, startTs, endTs) {
  const request = kgwTransactionListRequestFromUi(section, address, startTs, endTs);
  const result = await kgwInvokeExplorerGroupedTransactions(request);
  return kgwRowsFromDbGroupedResult(result);
}

function kgwRenderRowsFromDb(section, rows, statusText) {
  explorerState.rows = Array.isArray(rows) ? rows : [];
  explorerState.filteredRows = explorerState.rows.slice();

  renderTable(section);
  syncActionState(section);

  if (statusText) {
    setStatus(section, statusText);
  }
}

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

async function kgwLoadExplorerRowsFromDb(section, address, startTs, endTs) {
  window.__kgwExplorerDayTransactionCache = new Map();

  const summaries = await kgwLoadTransactionDaySummariesFromDb(section, address, startTs, endTs);

  microscopeLog("DAY SUMMARY LIST LOADED", {
    address,
    days: summaries.length
  });

  return summaries;
}

function kgwSetRowsAndRender(section, rows, statusText = "") {
  explorerState.rows = Array.isArray(rows) ? rows : [];
  explorerState.filteredRows = explorerState.rows.slice();

  renderTable(section);
  syncActionState(section);

  if (statusText) {
    setStatus(section, statusText);
  }
}

/* KGW_TX_R2_LIVE_DB_POLLING
   Python parity:
   While backend fetch is still running, poll local DB and render rows in batches.
   This prevents the table from looking frozen until all pages finish.
*/
function kgwStartLiveDbPollingDuringFetch(section, address, startTs, endTs, isForce) {
  let stopped = false;
  let lastRowCount = -1;
  let inFlight = false;

  async function kgwLiveDbPollingTick() {
    if (stopped || inFlight) return;

    inFlight = true;

    try {
      const rows = await kgwLoadExplorerRowsFromDb(section, address, startTs, endTs);

      if (!stopped && Array.isArray(rows) && rows.length !== lastRowCount) {
        lastRowCount = rows.length;

        explorerState.selectedAddress = address;
        explorerState.rows = rows;
        explorerState.filteredRows = rows.slice();

        renderTable(section);

        setStatus(
          section,
          `${isForce ? "Force fetch" : "Fetch"} is running... showing ${rows.length.toLocaleString()} days from local database so far.`
        );
      }
    } catch (error) {
      microscopeWarn("LIVE DB POLL FAILED", {
        message: error?.message || String(error)
      });
    } finally {
      inFlight = false;
    }
  }

  kgwLiveDbPollingTick();

  const timer = window.setInterval(tick, 2000);

  return async function stopLiveDbPolling(finalRefresh = true) {
    stopped = true;
    window.clearInterval(timer);

    if (finalRefresh) {
      try {
        const rows = await kgwLoadExplorerRowsFromDb(section, address, startTs, endTs);

        explorerState.selectedAddress = address;
        explorerState.rows = Array.isArray(rows) ? rows : [];
        explorerState.filteredRows = explorerState.rows.slice();

        renderTable(section);
      } catch (error) {
        microscopeWarn("LIVE DB FINAL REFRESH FAILED", {
          message: error?.message || String(error)
        });
      }
    }
  };
}


/* KGW_TX_R3_LIVE_LOCAL_DATABASE_RENDER
   Python parity:
   While backend fetch is still running, read saved local database transactions and render them.
   The backend keeps fetching pages; the UI does not wait for all pages to finish.
*/
function kgwStartLiveLocalDatabaseRenderDuringFetch(section, address, startTs, endTs, isForce) {
  let stopped = false;
  let inFlight = false;
  let lastRowCount = -1;
  let lastRenderAt = 0;

  async function kgwLiveLocalDatabaseRenderTick(forceRender = false) {
    if (stopped || inFlight) return;

    inFlight = true;

    try {
      const rows = await kgwLoadExplorerRowsFromDb(section, address, startTs, endTs);
      const rowCount = Array.isArray(rows) ? rows.length : 0;
      const now = Date.now();

      if (
        !stopped &&
        (forceRender || rowCount !== lastRowCount || now - lastRenderAt > 5000)
      ) {
        lastRowCount = rowCount;
        lastRenderAt = now;

        explorerState.selectedAddress = address;
        explorerState.rows = Array.isArray(rows) ? rows : [];
        explorerState.filteredRows = explorerState.rows.slice();

        renderTable(section);
        syncActionState(section);

        setStatus(
          section,
          `${isForce ? "Force fetch" : "Fetch"} is running... showing ${rowCount.toLocaleString()} days from local database so far.`
        );
      }
    } catch (error) {
      microscopeWarn("LIVE LOCAL_DATABASE RENDER FAILED", {
        message: error?.message || String(error)
      });
    } finally {
      inFlight = false;
    }
  }

  kgwLiveLocalDatabaseRenderTick(true);

  const timer = window.setInterval(() => kgwLiveLocalDatabaseRenderTick(false), 2000);

  return async function stopLiveLocalDatabaseRender(finalRefresh = true) {
    stopped = true;
    window.clearInterval(timer);

    if (finalRefresh) {
      await kgwLiveLocalDatabaseRenderTick(true);
    }
  };
}


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
        <td>${kgwFormatUsd(Number(row?.value_usd ?? row?.valueUsd ?? row?.usd_value ?? row?.usdValue ?? 0) || 0)}</td>
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

function kgwNormalizeDaySummaryRows(result) {
  const items = Array.isArray(result)
    ? result
    : Array.isArray(result?.days)
      ? result.days
      : Array.isArray(result?.summaries)
        ? result.summaries
        : [];

  return items
    .map((item) => ({
      __kgwDaySummary: true,
      day: String(item?.day || item?.date || "").slice(0, 10),
      count: Number(item?.count ?? item?.tx_count ?? item?.transactions_count ?? 0) || 0,
      incoming_kas: Number(item?.incoming_kas ?? item?.incomingKas ?? 0) || 0,
      outgoing_kas: Number(item?.outgoing_kas ?? item?.outgoingKas ?? 0) || 0,
      net_kas: Number(item?.net_kas ?? item?.netKas ?? 0) || 0
    }))
    .filter((item) => item.day);
}

async function kgwLoadDaySummariesOnly(section, address, startTs, endTs) {
  const request = {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: qs("#explorerTypeFilter", section)?.value || "ALL",
    direction: qs("#explorerDirectionFilter", section)?.value || "ALL",
    search_query: qs("#explorerSearch", section)?.value || "",
    limit: 10000
  };

  const result = await kgwInvokeExplorerDaySummaries(request);
  const rows = kgwNormalizeDaySummaryRows(result);

  microscopeLog("TX-UI-FAST-3E DAY SUMMARIES LOADED", {
    address,
    days: rows.length,
    first: rows[0] || null
  });

  return rows;
}

async function kgwLoadOneDayTransactionsOnly(section, address, day) {
  const startTs = kgwTxDayToEpochSeconds(day, false);
  const endTs = kgwTxDayToEpochSeconds(day, true);

  if (!Number.isFinite(startTs) || !Number.isFinite(endTs)) {
    return [];
  }

  const request = {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: qs("#explorerTypeFilter", section)?.value || "ALL",
    direction: qs("#explorerDirectionFilter", section)?.value || "ALL",
    search_query: qs("#explorerSearch", section)?.value || "",
    limit: 1000000
  };

  const groups = await kgwInvokeExplorerGroupedTransactions(request);
  const rows = extractRowsFromUnifiedResult({ groups });

  microscopeLog("TX-UI-FAST-3E ONE DAY TX LOADED", {
    day,
    rows: rows.length
  });

  return rows;
}

async function kgwRenderDaySummariesOnly(section, rows, statusText = "") {
  const body = qs("#explorerTransactionsBody", section);

  if (!body) {
    microscopeWarn("TX-UI-FAST-3E BODY MISSING", {});
    return;
  }

  const summaries = Array.isArray(rows)
    ? rows.filter((row) => row && row.__kgwDaySummary === true && row.day)
    : [];

  explorerState.rows = summaries;
  explorerState.filteredRows = summaries.slice();

  body.innerHTML = "";

  if (!summaries.length) {
    const tr = document.createElement("tr");
    tr.innerHTML = `<td colspan="6" class="muted" data-i18n="explorer.noTransactionsToDisplay">No transactions to display.</td>`;
    body.appendChild(tr);

    syncActionState(section);

    if (statusText) {
      setStatus(section, statusText);
    }

    microscopeWarn("TX-UI-FAST-3E EMPTY SUMMARY RENDER", {
      inputIsArray: Array.isArray(rows),
      inputLength: Array.isArray(rows) ? rows.length : null
    });

    return;
  }

  const totalCount = summaries.reduce((sum, row) => {
    return sum + (Number(row.count) || 0);
  }, 0);

  const totalNetKas = summaries.reduce((sum, row) => {
    return sum + (Number(row.net_kas) || 0);
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

  for (const summary of summaries) {
    const day = summary.day;
    const expanded = window.__kgwExplorerExpandedDateGroups.has(day);
    const sign = expanded ? "-" : "+";

    const dayTr = document.createElement("tr");
    dayTr.className = "kgw-day-group-row";
    dayTr.dataset.kgwDateGroup = day;
    dayTr.innerHTML = `
      <td>${sign} ${day}</td>
      <td>${Number(summary.count || 0).toLocaleString()} transactions</td>
      <td></td>
      <td>${kgwSummaryFormatKas(Number(summary.net_kas || 0) || 0)}</td>
      <td></td>
      <td></td>
    `;

    fragment.appendChild(dayTr);

    if (!expanded) {
      continue;
    }

    const cachedRows = window.__kgwExplorerDayTransactionCache.get(day);

    if (!cachedRows) {
      const loadingTr = document.createElement("tr");
      loadingTr.className = "kgw-transaction-row";
      loadingTr.innerHTML = `<td colspan="6" class="muted">Loading transactions for ${day}...</td>`;
      fragment.appendChild(loadingTr);
      continue;
    }

    for (const row of cachedRows) {
      const txid = row?.txid || row?.transaction_id || row?.transactionId || "";

      const txTr = document.createElement("tr");
      txTr.className = "kgw-transaction-row";
      txTr.dataset.kgwParentDateGroup = day;
      txTr.innerHTML = `
        <td>${row?.datetime || row?.date_time || row?.time || row?.timestamp || ""}</td>
        <td>${txid}</td>
        <td>${row?.direction || ""}</td>
        <td>${kgwSummaryFormatKas(Number(row?.amount_kas ?? row?.amountKas ?? row?.amount ?? 0) || 0)}</td>
        <td>${kgwFormatUsd(Number(row?.value_usd ?? row?.valueUsd ?? row?.usd_value ?? row?.usdValue ?? 0) || 0)}</td>
        <td>${row?.tx_type || row?.type || ""}</td>
      `;

      fragment.appendChild(txTr);
    }
  }

  body.appendChild(fragment);

  qsa("[data-kgw-date-group]", body).forEach((row) => {
    row.addEventListener("click", async () => {
      const day = row.dataset.kgwDateGroup;

      if (!day) {
        return;
      }

      if (window.__kgwExplorerExpandedDateGroups.has(day)) {
        window.__kgwExplorerExpandedDateGroups.delete(day);
        await kgwRenderDaySummariesOnly(section, explorerState.rows || [], statusText);
        return;
      }

      window.__kgwExplorerExpandedDateGroups.add(day);
      await kgwRenderDaySummariesOnly(section, explorerState.rows || [], statusText);

      if (!window.__kgwExplorerDayTransactionCache.has(day)) {
        try {
          const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", section)?.value);
          const dayRows = await kgwLoadOneDayTransactionsOnly(section, address, day);
          window.__kgwExplorerDayTransactionCache.set(day, dayRows);
        } catch (error) {
          microscopeWarn("TX-UI-FAST-3E ONE DAY LOAD FAILED", {
            day,
            message: error?.message || String(error)
          });

          window.__kgwExplorerDayTransactionCache.set(day, []);
        }

        await kgwRenderDaySummariesOnly(section, explorerState.rows || [], statusText);
      }
    });
  });

  syncActionState(section);

  if (statusText) {
    setStatus(section, statusText);
  }

  microscopeLog("TX-UI-FAST-3E SUMMARY RENDER DONE", {
    days: summaries.length,
    totalCount
  });
}

function resetFilters(section) {
  qs("#explorerDirectionFilter", section).value = "ALL";
  qs("#explorerTypeFilter", section).value = "ALL";
  qs("#explorerSearch", section).value = "";
  explorerState.filteredRows = explorerState.rows.slice();
  renderTable(section);
}

function setTableFontSize(section) {
  const table = qs("#explorerTransactionsTable", section);
  const input = qs("#explorerTableFontSize", section);
  const size = Math.max(6, Math.min(24, Number(explorerState.fontSize || input?.value || 9)));

  if (input) {
    input.value = String(size);
  }

  if (!table) return;

  section.style.setProperty("--explorer-table-font-size", `${size}px`);
  table.style.setProperty("font-size", `${size}px`, "important");

  table.querySelectorAll("th, td").forEach((cell) => {
    cell.style.setProperty("font-size", `${size}px`, "important");
  });

  microscopeLog("TABLE FONT SIZE APPLIED", {
    size,
    cssVariable: section.style.getPropertyValue("--explorer-table-font-size"),
    tableFontSize: getComputedStyle(table).fontSize
  });
}


/* KGW_DB_FILTER_ONLY
   Python parity: filter button reads from DB only.
*/
async function applyExplorerFiltersFromDatabase(section) {
  if (explorerState.busy) {
    setStatus(section, "Fetch is running. Wait until it finishes before applying filters.");
    return;
  }

  const address = normalizeAddress(qs("#explorerAddress", section)?.value);

  if (!isKaspaAddress(address)) {
    clearExplorerTransactionTable(section, "Enter a valid Kaspa address.");
    return;
  }

  const startTs = parseDateSeconds(qs("#explorerFromDate", section)?.value, false);
  const endTs = parseDateSeconds(qs("#explorerToDate", section)?.value, true);

  setStatus(section, "Loading filtered transactions from database...");

  const rows = await kgwLoadAndRenderDaySummaries(section, address, startTs, endTs, "Filter applied. Showing days from local database. Click + to load one day.");
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
  } catch (error) {
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
  microscopeLog(label, {
    rows: explorerState?.rows?.length ?? null,
    filteredRows: explorerState?.filteredRows?.length ?? null,
    savedAddresses: explorerState?.savedAddresses?.length ?? null,
    addressNames: explorerState?.addressNames?.size ?? null,
    addressNamesLoaded: explorerState?.addressNamesLoaded ?? null,
    selectedAddress: explorerState?.selectedAddress ?? "",
    busy: explorerState?.busy ?? null,
    fontSize: explorerState?.fontSize ?? null
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

function microscopeCheckAddressNameMatch(address) {
  const keys = typeof addressLookupKeys === "function" ? addressLookupKeys(address) : [address];
  const matches = [];

  for (const key of keys) {
    const value = explorerState?.addressNames?.get?.(key);
    if (value) matches.push({ key, value });
  }

  microscopeLog("ADDRESS NAME MATCH CHECK", {
    address,
    lookupKeys: keys,
    matches,
    namesLoaded: explorerState?.addressNamesLoaded ?? null,
    namesCount: explorerState?.addressNames?.size ?? null
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

function kgwClean2UsdPrice() {
  const direct = [
    window.__kgwKaspaUsdPrice,
    window.__kgwHeaderPriceUsd,
    window.__kgwLastKasPriceUsd,
    window.__kaspaPriceUsd,
    window.kaspaPriceUsd,
    document.documentElement?.dataset?.kgwKaspaUsdPrice
  ];

  for (const item of direct) {
    const n = Number(item);
    if (Number.isFinite(n) && n > 0) return n;
  }

  const headerText = String(document.getElementById("kgwHeaderPrice")?.textContent || "").replace(/,/g, "");
  const headerMatch = headerText.match(/([0-9]+(?:\.[0-9]+)?)/);
  const headerParsed = Number(headerMatch?.[1]);

  if (Number.isFinite(headerParsed) && headerParsed > 0) {
    return headerParsed;
  }

  const bodyText = document.body?.innerText || "";
  const m = bodyText.replace(/,/g, "").match(/(?:Price\s*)?([0-9]+(?:\.[0-9]+)?)\s*USD/i);
  const parsed = Number(m?.[1]);

  return Number.isFinite(parsed) && parsed > 0 ? parsed : 0;
}

function kgwClean2NormalizeSummaries(result) {
  const items = Array.isArray(result)
    ? result
    : Array.isArray(result?.days)
      ? result.days
      : Array.isArray(result?.summaries)
        ? result.summaries
        : [];

  const price = kgwClean2UsdPrice();

  return items
    .map((item) => {
      const incomingKas = Number(item?.incoming_kas ?? item?.incomingKas ?? 0) || 0;
      const outgoingKas = Number(item?.outgoing_kas ?? item?.outgoingKas ?? 0) || 0;
      const netKas = Number(item?.net_kas ?? item?.netKas ?? (incomingKas - outgoingKas)) || 0;
      const grossKas = Math.abs(incomingKas) + Math.abs(outgoingKas);
      const kasForUsd = grossKas > 0 ? grossKas : Math.abs(netKas);

      const explicitUsd = Number(
        item?.value_usd ??
        item?.valueUsd ??
        item?.usd_value ??
        item?.usdValue ??
        item?.value
      );

      return {
        __kgwDaySummary: true,
        day: String(item?.day || item?.date || "").slice(0, 10),
        count: Number(item?.count ?? item?.tx_count ?? item?.transactions_count ?? 0) || 0,
        incoming_kas: incomingKas,
        outgoing_kas: outgoingKas,
        net_kas: netKas,
        value_usd: Number.isFinite(explicitUsd) && explicitUsd > 0
          ? explicitUsd
          : price > 0
            ? kasForUsd * price
            : 0
      };
    })
    .filter((item) => item.day);
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


/* KGW_FIX_UNKNOWN_FILTER_VALUES
   The DB currently stores:
   tx_type:   ALL / coinbase / transfer
   direction: ALL / incoming / outgoing

   Any UI value such as unknown/address/عنوان/compound is invalid here
   and must be normalized before sending the Rust request.
*/
function kgwNormalizeTxTypeFilterValue(value) {
  const normalized = String(value || "ALL").trim().toLowerCase();

  if (normalized === "all") return "ALL";
  if (normalized === "coinbase") return "coinbase";
  if (normalized === "transfer") return "transfer";

  console.warn("[KGW Explorer][filter] invalid tx_type normalized to ALL", {
    original: value
  });

  return "ALL";
}

function kgwNormalizeDirectionFilterValue(value) {
  const normalized = String(value || "ALL").trim().toLowerCase();

  if (normalized === "all") return "ALL";
  if (normalized === "incoming") return "incoming";
  if (normalized === "outgoing") return "outgoing";

  console.warn("[KGW Explorer][filter] invalid direction normalized to ALL", {
    original: value
  });

  return "ALL";
}

function kgwRepairExplorerFilterSelects(section) {
  const typeEl = qs("#explorerTypeFilter", section);
  const directionEl = qs("#explorerDirectionFilter", section);

  if (typeEl) {
    const current = kgwNormalizeTxTypeFilterValue(typeEl.value);

    typeEl.innerHTML = `
      <option value="ALL">ALL</option>
      <option value="coinbase">coinbase</option>
      <option value="transfer">transfer</option>
    `;

    typeEl.value = current;
  }

  if (directionEl) {
    const current = kgwNormalizeDirectionFilterValue(directionEl.value);

    directionEl.innerHTML = `
      <option value="ALL">ALL</option>
      <option value="incoming">incoming</option>
      <option value="outgoing">outgoing</option>
    `;

    directionEl.value = current;
  }

  console.log("[KGW Explorer][filter] selects repaired", {
    txType: typeEl?.value,
    direction: directionEl?.value
  });
}


/* KGW_FIX_FILTER_SELECTS_INIT_REPAIR
   Repair Explorer filter dropdowns immediately, not only when Fetch/Filter builds a request.
   unknown/address/عنوان/compound must never remain visible in Type/Direction filters.
*/
function kgwRepairExplorerFilterSelectsNow() {
  const roots = [
    document.querySelector("#explorer"),
    document.querySelector(".explorer-python-root")
  ].filter(Boolean);

  if (!roots.length) {
    return;
  }

  for (const root of roots) {
    kgwRepairExplorerFilterSelects(root);
  }
}

if (!window.__kgwFilterSelectsInitRepairInstalled) {
  window.__kgwFilterSelectsInitRepairInstalled = true;

  queueMicrotask(kgwRepairExplorerFilterSelectsNow);

  document.addEventListener("DOMContentLoaded", kgwRepairExplorerFilterSelectsNow, true);

  document.addEventListener(
    "click",
    (event) => {
      const text = String(event.target?.textContent || event.target?.id || "").toLowerCase();

      if (text.includes("explorer") || event.target?.closest?.("#explorer")) {
        setTimeout(kgwRepairExplorerFilterSelectsNow, 0);
      }
    },
    true
  );

  document.addEventListener(
    "change",
    (event) => {
      const id = event.target?.id || "";

      if (id === "explorerTypeFilter" || id === "explorerDirectionFilter") {
        kgwRepairExplorerFilterSelectsNow();
      }
    },
    true
  );

  setTimeout(kgwRepairExplorerFilterSelectsNow, 100);
  setTimeout(kgwRepairExplorerFilterSelectsNow, 500);
  setTimeout(kgwRepairExplorerFilterSelectsNow, 1500);

  console.log("[KGW Explorer][filter] select init repair installed");
}

function kgwClean2Request(section, address, startTs, endTs, limit) {
  const root = kgwClean2Section(section);
  kgwRepairExplorerFilterSelects(root);

  const request = {
    address,
    start_ts: startTs,
    end_ts: endTs,
    tx_type: kgwNormalizeTxTypeFilterValue(qs("#explorerTypeFilter", root)?.value),
    direction: kgwNormalizeDirectionFilterValue(qs("#explorerDirectionFilter", root)?.value),
    search_query: String(qs("#explorerSearch", root)?.value || ""),
    limit
  };

  kgwClean2Log("request", request);
  return request;
}

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
   This function is intentionally exposed to explorer.export.js through window
   to keep the existing export_report route canonical and avoid a new export system.
*/
function kgwExplorerExportStringV2(value) {
  return String(value ?? "").trim();
}

function kgwExplorerExportNumberV2(value, digits = 8) {
  const number = Number(value);
  if (!Number.isFinite(number)) return "";
  return number.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: digits
  });
}

function kgwExplorerExportTxUrlV2(txid) {
  const clean = kgwExplorerExportStringV2(txid);
  return /^[0-9a-f]{32,}$/i.test(clean) ? `https://explorer.kaspa.org/txs/${clean}` : "";
}

function kgwExplorerExportAddressUrlV2(address) {
  const clean = kgwExplorerExportStringV2(address);
  return clean.startsWith("kaspa:") ? `https://explorer.kaspa.org/addresses/${clean}` : "";
}

function kgwExplorerExportJoinAddressesV2(...values) {
  const seen = new Set();
  const out = [];

  for (const value of values.flat()) {
    const clean = kgwExplorerExportStringV2(value);
    if (!clean || seen.has(clean)) continue;
    seen.add(clean);
    out.push(clean);
  }

  return out.join(" | ");
}

function kgwExplorerExportNormalizeRawTxV2(row) {
  const txid = kgwExplorerExportStringV2(row?.txid || row?.transactionId || row?.transaction_id || row?.id || row?.hash);
  const fromAddress = kgwExplorerExportJoinAddressesV2(row?.from_address, row?.fromAddress, row?.from);
  const toAddress = kgwExplorerExportJoinAddressesV2(row?.to_address, row?.toAddress, row?.to);
  const counterparty = kgwExplorerExportStringV2(row?.counterparty || row?.counterParty);
  const timestampMs = Number(row?.timestamp_ms ?? row?.timestampMs ?? row?.timestamp ?? 0) || 0;
  const amount = Number(row?.amount ?? row?.amount_kas ?? row?.amountKas ?? 0) || 0;
  const value = Number(row?.value ?? row?.value_usd ?? row?.valueUsd ?? 0) || 0;

  return {
    datetime: kgwExplorerExportStringV2(row?.datetime || row?.date_time || row?.dateTime || ""),
    txid,
    direction: kgwExplorerExportStringV2(row?.direction || "unknown"),
    fromAddress,
    toAddress,
    counterparty,
    amount,
    blockScore: kgwExplorerExportStringV2(row?.block_score || row?.blockScore || row?.block_height || row?.blockHeight || ""),
    timestampMs,
    type: kgwExplorerExportStringV2(row?.type || row?.tx_type || row?.txType || "transfer"),
    value,
    date: kgwExplorerExportStringV2(row?.date || row?.day || ""),
    transactionUrl: kgwExplorerExportTxUrlV2(txid),
    addressUrl: kgwExplorerExportJoinAddressesV2(
      kgwExplorerExportAddressUrlV2(fromAddress),
      kgwExplorerExportAddressUrlV2(toAddress),
      kgwExplorerExportAddressUrlV2(counterparty)
    )
  };
}

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
   Force fetch UX rules:
   - Force fetch clears the table immediately.
   - Force fetch always fetches ALL accepted transactions, not the current filter.
   - During fetch, controls that can trigger DB/filter/render work are disabled.
   - Cancel stays enabled.
   - While force fetch is running, summaries are live-rendered from local database after delete begins.
*/
function kgwForceUiRoot(section) {
  return (
    section ||
    document.querySelector("#explorer") ||
    document.querySelector(".explorer-python-root")
  );
}

function kgwForceUiBody(section) {
  const root = kgwForceUiRoot(section);
  return (
    qs("#explorerTransactionsBody", root) ||
    root?.querySelector?.("tbody") ||
    document.querySelector("#explorerTransactionsBody")
  );
}

function kgwForceSetTableMessage(section, message) {
  const body = kgwForceUiBody(section);

  if (!body) {
    console.warn("[KGW Explorer][force-ui] tbody not found for message", { message });
    return;
  }

  body.innerHTML = `<tr><td colspan="6" class="muted">${kgwClean2SafeText(message)}</td></tr>`;

  console.log("[KGW Explorer][force-ui] table message", {
    message,
    childRows: body.children.length
  });
}

function kgwForceResetDisplayFiltersToAll(section) {
  const root = kgwForceUiRoot(section);
  const typeEl = qs("#explorerTypeFilter", root);
  const directionEl = qs("#explorerDirectionFilter", root);
  const searchEl = qs("#explorerSearch", root);

  if (typeof kgwRepairExplorerFilterSelects === "function") {
    kgwRepairExplorerFilterSelects(root);
  }

  if (typeEl) typeEl.value = "ALL";
  if (directionEl) directionEl.value = "ALL";
  if (searchEl) searchEl.value = "";

  console.log("[KGW Explorer][force-ui] force filters reset to ALL");
}

function kgwForceSetControlsBusy(section, busy, mode = "normal") {
  const root = kgwForceUiRoot(section);

  if (!root) return;

  root.dataset.kgwFetchBusy = busy ? "true" : "false";
  root.dataset.kgwFetchMode = busy ? mode : "";

  const selectors = [
    "#explorerAddress",
    "#explorerFromDate",
    "#explorerToDate",
    "#explorerTypeFilter",
    "#explorerDirectionFilter",
    "#explorerSearch",
    "#explorerFetchButton",
    "#explorerForceFetchButton",
    "#explorerFilterButton",
    "#explorerResetFilterButton",
    "button",
    "select",
    "input"
  ];

  const controls = new Set();

  for (const selector of selectors) {
    root.querySelectorAll(selector).forEach((element) => controls.add(element));
  }

  for (const element of controls) {
    const text = [
      element.id,
      element.name,
      element.value,
      element.textContent,
      element.getAttribute?.("aria-label"),
      element.dataset?.action,
      element.dataset?.kgwAction
    ].filter(Boolean).join(" ").toLowerCase();

    const isCancel =
      text.includes("cancel") ||
      element.id?.toLowerCase?.().includes("cancel");

    if (busy) {
      if (isCancel) {
        element.disabled = false;
        element.removeAttribute("aria-disabled");
      } else {
        element.disabled = true;
        element.setAttribute("aria-disabled", "true");
      }
    } else {
      element.disabled = false;
      element.removeAttribute("aria-disabled");
    }
  }

  console.log("[KGW Explorer][force-ui] controls busy", {
    busy,
    mode,
    disabledCount: Array.from(controls).filter((x) => x.disabled).length
  });
}

function kgwForceBlockBusyExplorerActions(event) {
  const root =
    document.querySelector("#explorer") ||
    document.querySelector(".explorer-python-root");

  if (!root || root.dataset.kgwFetchBusy !== "true") return;

  const target = event.target?.closest?.(
    "button,input,select,textarea,a,[role='button'],.btn,.button"
  );

  if (!target || !root.contains(target)) return;

  const text = [
    target.id,
    target.name,
    target.value,
    target.textContent,
    target.getAttribute?.("aria-label"),
    target.dataset?.action,
    target.dataset?.kgwAction
  ].filter(Boolean).join(" ").toLowerCase();

  if (text.includes("cancel")) return;

  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation();

  console.warn("[KGW Explorer][force-ui] blocked action while fetch is busy", {
    eventType: event.type,
    id: target.id || "",
    tag: target.tagName,
    text: String(target.textContent || target.value || "").trim()
  });
}

if (!window.__kgwForceUiBusyBlockInstalled) {
  window.__kgwForceUiBusyBlockInstalled = true;

  document.addEventListener("click", kgwForceBlockBusyExplorerActions, true);
  document.addEventListener("change", kgwForceBlockBusyExplorerActions, true);
  document.addEventListener("input", kgwForceBlockBusyExplorerActions, true);
  document.addEventListener("pointerdown", kgwForceBlockBusyExplorerActions, true);
}


/* KGW_TX_LIVE_CORE_1_LISTENER
   Generic live table updates for both normal and force fetch.
   It listens to pages already stored by the existing Rust sync loop.
*/
function kgwLiveCoreRoot() {
  return document.querySelector("#explorer") || document.querySelector(".explorer-python-root");
}

function kgwLiveCoreDayFromMs(timestampMs) {
  const n = Number(timestampMs || 0);
  if (!Number.isFinite(n) || n <= 0) return "";
  return new Date(n).toISOString().slice(0, 10);
}

function kgwLiveCoreReset(address) {
  window.__kgwLiveCoreAddress = address || "";
  window.__kgwLiveCoreDays = new Map();
}

function kgwLiveCoreSeedFromCurrentRows(address) {
  if (!window.__kgwLiveCoreDays) {
    kgwLiveCoreReset(address);
  }

  const rows = Array.isArray(explorerState?.rows) ? explorerState.rows : [];

  for (const row of rows) {
    if (!row?.__kgwDaySummary || !row.day) continue;
    window.__kgwLiveCoreDays.set(row.day, { ...row });
  }
}

function kgwLiveCoreMergeRecords(records) {
  if (!window.__kgwLiveCoreDays) {
    window.__kgwLiveCoreDays = new Map();
  }

  const price = typeof kgwClean2UsdPrice === "function" ? kgwClean2UsdPrice() : 0;

  for (const record of Array.isArray(records) ? records : []) {
    const day = kgwLiveCoreDayFromMs(record?.timestamp_ms ?? record?.timestampMs);
    if (!day) continue;

    const amountSompi = Math.abs(Number(record?.amount_sompi ?? record?.amountSompi ?? 0) || 0);
    const amountKas = amountSompi / 100000000;
    const direction = String(record?.direction || "").toLowerCase();

    const summary = window.__kgwLiveCoreDays.get(day) || {
      __kgwDaySummary: true,
      day,
      count: 0,
      incoming_kas: 0,
      outgoing_kas: 0,
      net_kas: 0,
      value_usd: 0
    };

    summary.count += 1;

    if (direction === "outgoing") {
      summary.outgoing_kas += amountKas;
    } else {
      summary.incoming_kas += amountKas;
    }

    summary.net_kas = summary.incoming_kas - summary.outgoing_kas;
    summary.value_usd = price > 0
      ? (Math.abs(summary.incoming_kas) + Math.abs(summary.outgoing_kas)) * price
      : 0;

    window.__kgwLiveCoreDays.set(day, summary);
  }
}

function kgwLiveCoreRows() {
  return Array.from((window.__kgwLiveCoreDays || new Map()).values())
    .sort((a, b) => String(b.day).localeCompare(String(a.day)));
}


/* TX_SPEED_CORE_1_LIGHT_LIVE_DAYS
   Merge compact day summaries from Rust.
   This avoids sending/rendering hundreds of transaction rows for every live progress event.
*/
function kgwLiveCoreMergeDays(days) {
  if (!window.__kgwLiveCoreDays) {
    window.__kgwLiveCoreDays = new Map();
  }

  const price = typeof kgwClean2UsdPrice === "function" ? kgwClean2UsdPrice() : 0;

  for (const item of Array.isArray(days) ? days : []) {
    const day = String(item?.day || "").slice(0, 10);
    if (!day) continue;

    const incomingSompi = Math.abs(Number(item?.incoming_sompi ?? item?.incomingSompi ?? 0) || 0);
    const outgoingSompi = Math.abs(Number(item?.outgoing_sompi ?? item?.outgoingSompi ?? 0) || 0);
    const incomingKas = incomingSompi / 100000000;
    const outgoingKas = outgoingSompi / 100000000;

    const summary = window.__kgwLiveCoreDays.get(day) || {
      __kgwDaySummary: true,
      day,
      count: 0,
      incoming_kas: 0,
      outgoing_kas: 0,
      net_kas: 0,
      value_usd: 0
    };

    summary.count += Number(item?.count || 0) || 0;
    summary.incoming_kas += incomingKas;
    summary.outgoing_kas += outgoingKas;
    summary.net_kas = summary.incoming_kas - summary.outgoing_kas;
    summary.value_usd = price > 0
      ? (Math.abs(summary.incoming_kas) + Math.abs(summary.outgoing_kas)) * price
      : 0;

    window.__kgwLiveCoreDays.set(day, summary);
  }
}

function kgwLiveCoreShouldRender(payload) {
  const now = Date.now();
  const page = Number(payload?.page || 0);

  if (!window.__kgwLiveCoreLastRenderMs) {
    window.__kgwLiveCoreLastRenderMs = 0;
  }

  if (page <= 1) {
    window.__kgwLiveCoreLastRenderMs = now;
    return true;
  }

  if (page % 2 === 0) {
    window.__kgwLiveCoreLastRenderMs = now;
    return true;
  }

  if (now - window.__kgwLiveCoreLastRenderMs >= 1500) {
    window.__kgwLiveCoreLastRenderMs = now;
    return true;
  }

  return false;
}

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
      window.__kgwLiveCoreAddress ||
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
async function kgwFinalLocalDatabaseRefreshAfterFetch(root, address, startTs, endTs, isForce) {
  const started = performance.now();

  const finalRows = await kgwClean2LoadSummaries(root, address, startTs, endTs);

  window.__kgwLiveCoreDays = new Map();
  window.__kgwLiveCoreAddress = "";

  window.__kgwExplorerDayTransactionCache = new Map();
  window.__kgwExplorerExpandedDateGroups = new Set();

  await kgwClean2RenderSummaries(
    root,
    finalRows,
    isForce
      ? `Force fetch done. Showing ${finalRows.length.toLocaleString()} days from local database. Click + to load one day.`
      : `Fetch done. Showing ${finalRows.length.toLocaleString()} days from local database. Click + to load one day.`
  );

  console.log("[KGW Explorer][final-refresh] local database final render done", {
    force: Boolean(isForce),
    days: finalRows.length,
    totalTransactions: finalRows.reduce((sum, row) => sum + (Number(row.count) || 0), 0),
    elapsedMs: Math.round(performance.now() - started),
    first: finalRows[0] || null,
    last: finalRows[finalRows.length - 1] || null
  });

  return finalRows;
}

async function fetchTransactions(section, forceMode) {
  const root = kgwClean2Section(section);
  const isForce = Boolean(forceMode);
  const address = normalizeAddress(qs("#explorerAddress", root)?.value);

  
  kgwExplorerUiTraceR53B3("explorer-fetch", "r53b3-explorer-fetch-owner-begin", {
    force: Boolean(isForce),
    addressLength: String(address || "").length
  });

  if (!isKaspaAddress(address)) {
    clearExplorerTransactionTable(root, "Enter a valid Kaspa address.");
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

      kgwLiveCoreSeedFromCurrentRows(address);

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
  const address = explorerState.selectedAddress || normalizeAddress(qs("#explorerAddress", root)?.value);

  kgwClean2Log("applyFilter start", {
    address,
    type: String(qs("#explorerTypeFilter", root)?.value || "ALL"),
    direction: String(qs("#explorerDirectionFilter", root)?.value || "ALL"),
    search: String(qs("#explorerSearch", root)?.value || "")
  });

  if (!isKaspaAddress(address)) {
    clearExplorerTransactionTable(root, "Enter a valid Kaspa address.");
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

/* KGW_EXPLORER_MANUAL_ADDRESS_SAVE_OWNER_V1 */
function kgwExplorerManualAddressValue(section = root()) {
  return String(qs("#explorerAddress", section)?.value || "").trim();
}

function kgwExplorerIsKaspaAddress(value) {
  return /^kaspa(test)?:[a-z0-9]{50,}$/i.test(String(value || "").trim());
}

async function kgwExplorerSaveManualAddress(section = root()) {
  const address = kgwExplorerManualAddressValue(section);

  if (!kgwExplorerIsKaspaAddress(address)) {
    setStatus(section, "Invalid Kaspa address.");
    return;
  }

  try {
    await invokeCommand("save_address", { address, name: "" });
    explorerState.addressNamesLoaded = false;
    await loadSavedAddresses(section);
    await refreshAddressName(section, address);
    setStatus(section, "Address saved.");

    if (typeof window.kgwRefreshSettingsAddresses === "function") {
      window.kgwRefreshSettingsAddresses().catch(console.error);
    }
  } catch (error) {
    setStatus(section, `Save address failed: ${error?.message || error}`);
  }
}

function kgwInstallExplorerManualAddressSave() {
  const section = root();
  const input = section.querySelector("#explorerAddress");
  const datalist = section.querySelector("#explorerAddressOptions");
  const dropdown = section.querySelector("#explorerAddressDropdown");

  if (!input || !datalist || !dropdown) return;
  // KGW_EXPLORER_CUSTOM_SAVED_ADDRESS_DROPDOWN_BODY_ANCHOR_R7
  if (dropdown.parentElement !== document.body) {
    document.body.appendChild(dropdown);
  }

  if (input.dataset.kgwExplorerCustomDropdownR5Installed === "1") return;
  input.dataset.kgwExplorerCustomDropdownR5Installed = "1";

  // KGW_EXPLORER_CUSTOM_SAVED_ADDRESS_DROPDOWN_R5
  input.removeAttribute("list");
  input.setAttribute("aria-haspopup", "listbox");
  input.setAttribute("aria-controls", "explorerAddressDropdown");
  input.setAttribute("aria-expanded", "false");

  const readOptions = () => Array.from(datalist.querySelectorAll("option"))
    .map((option) => String(option.value || option.textContent || "").trim())
    .filter(Boolean)
    .filter((value, index, all) => all.indexOf(value) === index);

  const placeDropdown = () => {
    const rect = input.getBoundingClientRect();
    const viewportWidth = window.innerWidth || document.documentElement.clientWidth || rect.right;
    const viewportHeight = window.innerHeight || document.documentElement.clientHeight || rect.bottom;
    const dropdownWidth = Math.max(220, rect.width);
    const left = Math.max(8, Math.min(rect.left, viewportWidth - dropdownWidth - 8));
    const top = Math.max(8, Math.min(rect.bottom + 4, viewportHeight - 64));

    dropdown.style.position = "fixed";
    dropdown.style.left = `${left}px`;
    dropdown.style.top = `${top}px`;
    dropdown.style.width = `${dropdownWidth}px`;
    dropdown.style.transform = "none";
  };

  const closeDropdown = () => {
    dropdown.hidden = true;
    input.setAttribute("aria-expanded", "false");
  };

  const applyAddress = (address) => {
    input.value = address;
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
    closeDropdown();
  };

  const renderDropdown = () => {
    const addresses = readOptions();
    dropdown.replaceChildren();

    if (!addresses.length) {
      const empty = document.createElement("div");
      empty.className = "kgw-explorer-address-dropdown-empty";
      empty.textContent = kgwI18nTextR41("ui.explorer.noSavedAddresses", "No saved addresses");
      dropdown.appendChild(empty);
      return;
    }

    for (const address of addresses) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "kgw-explorer-address-dropdown-option";
      button.setAttribute("role", "option");
      button.textContent = address.length > 54 ? address.slice(0, 54) + "..." : address;
      button.title = address;
      button.addEventListener("mousedown", (event) => event.preventDefault());
      button.addEventListener("click", () => applyAddress(address));
      dropdown.appendChild(button);
    }
  };

  const openDropdown = () => {
    renderDropdown();
    placeDropdown();
    dropdown.hidden = false;
    input.setAttribute("aria-expanded", "true");
  };

  let autosaveTimer = null;
  let lastAutosavedAddress = "";

  const scheduleAutosave = (reason = "input", delay = 350) => {
    window.clearTimeout(autosaveTimer);
    autosaveTimer = window.setTimeout(async () => {
      const address = kgwExplorerManualAddressValue(section);

      if (!kgwExplorerIsKaspaAddress(address)) return;
      if (address === lastAutosavedAddress) return;

      lastAutosavedAddress = address;

      try {
        await kgwExplorerSaveManualAddress(section);
        setStatus(section, reason === "paste" ? "Address pasted and saved." : "Address saved.");
        await loadSavedAddresses(section);
      } catch (error) {
        lastAutosavedAddress = "";
        setStatus(section, `Save address failed: ${error?.message || error}`);
      }
    }, delay);
  };

  input.addEventListener("focus", openDropdown);
  input.addEventListener("click", openDropdown);
  input.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "Enter") openDropdown();
    if (event.key === "Escape") closeDropdown();
  });

  input.addEventListener("paste", () => {
    scheduleAutosave("paste", 40);
  });

  input.addEventListener("input", () => {
    scheduleAutosave("input", 350);
  });

  input.addEventListener("change", () => {
    scheduleAutosave("change", 0);
  });

  input.addEventListener("blur", () => {
    scheduleAutosave("blur", 0);
  });

  window.addEventListener("resize", () => {
    if (!dropdown.hidden) placeDropdown();
  });

  document.addEventListener("mousedown", (event) => {
    if (event.target === input || dropdown.contains(event.target)) return;
    closeDropdown();
  });

  renderDropdown();
}

window.kgwExplorerSaveManualAddress = kgwExplorerSaveManualAddress;
window.kgwInstallExplorerManualAddressSave = kgwInstallExplorerManualAddressSave;
kgwInstallExplorerManualAddressSave();
