

const KGW_LIVE_METRICS_REFRESH_MS = 15000;

let kgwLiveMetricsStarted = false;
let kgwClockTimer = null;
let kgwMetricsTimer = null;

function kgwPad(value) {
  return String(value).padStart(2, "0");
}

function kgwFormatLocalDateTime(epochMs) {
  if (!epochMs) return "غير معروف";

  const date = new Date(Number(epochMs));

  if (Number.isNaN(date.getTime())) {
    return "غير معروف";
  }

  return `${date.getFullYear()}-${kgwPad(date.getMonth() + 1)}-${kgwPad(date.getDate())} ${kgwPad(date.getHours())}:${kgwPad(date.getMinutes())}:${kgwPad(date.getSeconds())}`;
}

function kgwNowDateOnly() {
  const now = new Date();
  return `${now.getFullYear()}-${kgwPad(now.getMonth() + 1)}-${kgwPad(now.getDate())}`;
}

function kgwNowTimeOnly() {
  const now = new Date();
  return `${kgwPad(now.getHours())}:${kgwPad(now.getMinutes())}:${kgwPad(now.getSeconds())}`;
}

function kgwLog(message) {
  try {
    if (window.kgwLog) {
      window.kgwLog("frontend:shell", message);
    } else {
      console.log(`[KGW][header-live-metrics] ${message}`);
    }
  } catch (_) {}
}

function kgwSetClock() {
  const timeEl = document.getElementById("kgwHeaderClockTime");
  const dateEl = document.getElementById("kgwHeaderClockDate");

  if (timeEl) timeEl.textContent = kgwNowTimeOnly();
  if (dateEl) dateEl.textContent = kgwNowDateOnly();
}

async function kgwResolveInvoke() {
  if (window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke;
  }

  if (window.__TAURI__?.tauri?.invoke) {
    return window.__TAURI__.tauri.invoke;
  }

  try {
    const core = await import("@tauri-apps/api/core");
    if (core?.invoke) return core.invoke;
  } catch (_) {}

  try {
    const tauri = await import("@tauri-apps/api/tauri");
    if (tauri?.invoke) return tauri.invoke;
  } catch (_) {}

  throw new Error("Tauri invoke API is not available");
}

function kgwMetricIds(kind) {
  const cap = `${kind[0].toUpperCase()}${kind.slice(1)}`;

  return {
    valueId: `kgwHeader${cap}`,
    boxId: `kgwHeader${cap}Box`
  };
}


/* KGW_LIVE_METRICS_DYNAMIC_VALUE_OWNER_V1 */
function kgwExtractUsdPriceFromMetricValue(value) {
  const match = String(value || "").replace(/,/g, "").match(/([0-9]+(?:\.[0-9]+)?)/);
  const number = Number(match?.[1]);

  return Number.isFinite(number) && number > 0 ? number : 0;
}

function kgwPublishKaspaUsdPrice(value) {
  const priceUsd = kgwExtractUsdPriceFromMetricValue(value);

  if (!priceUsd) return;

  window.__kgwKaspaUsdPrice = priceUsd;
  window.__kgwHeaderPriceUsd = priceUsd;
  window.__kgwLastKasPriceUsd = priceUsd;
  window.__kaspaPriceUsd = priceUsd;
  window.kaspaPriceUsd = priceUsd;
  document.documentElement.dataset.kgwKaspaUsdPrice = String(priceUsd);

  try {
    window.dispatchEvent(new CustomEvent("kgw:kaspa-price-updated", {
      detail: { priceUsd }
    }));
  } catch (_) {}
}

function kgwOwnMetricValueElement(kind, element) {
  if (!element) return;

  element.removeAttribute("data-i18n");
  element.setAttribute("data-kgw-no-i18n", "true");
  element.dataset.kgwLiveMetricValue = kind;
}




/* KGW_HEADER_SELECTED_CURRENCY_CANONICAL_WRITER_R85B
 * Makes the existing live metrics price writer respect #shellCurrencySelect.
 * This prevents the periodic live metrics USD snapshot from overwriting selected currency rendering.
 */
function kgwHeaderTraceR85B(phase, details) {
  try {
    const payload = Object.assign(
      {
        scope: "header",
        owner: "KGW_HEADER_SELECTED_CURRENCY_CANONICAL_WRITER_R85B",
        phase,
      },
      details || {},
    );

    if (typeof globalThis.kgwUiTrace === "function") {
      globalThis.kgwUiTrace(payload);
    }
  } catch (_) {
    // Trace must never break header metrics.
  }
}

function kgwHeaderSelectedCurrencyR85B() {
  const select = document.querySelector("#shellCurrencySelect");
  const raw = select && select.value ? String(select.value) : "USD";
  const currency = raw.trim().toUpperCase();
  return currency || "USD";
}

function kgwHeaderPriceSnapshotR85B() {
  const snapshot = globalThis.kgwHeaderLastKaspaPricesR81C;
  return snapshot && typeof snapshot === "object" ? snapshot : {};
}

function kgwHeaderNumericPriceR85B(prices, currency) {
  const lower = String(currency || "USD").toLowerCase();
  const upper = lower.toUpperCase();
  const raw = prices[lower] ?? prices[upper];
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : null;
}

function kgwHeaderFormatCurrencyR85B(currency, value) {
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      maximumFractionDigits: value >= 1 ? 4 : 8,
    }).format(value);
  } catch (_) {
    return currency + " " + String(value);
  }
}

function kgwHeaderSelectedCurrencyMetricValueR85B(metricValue, currentText) {
  const currency = kgwHeaderSelectedCurrencyR85B();
  const prices = kgwHeaderPriceSnapshotR85B();
  const selectedValue =
    currency === "USD"
      ? kgwExtractUsdPriceFromMetricValue(metricValue)
      : kgwHeaderNumericPriceR85B(prices, currency);

  const hasCachedSelectedValue = selectedValue !== null && selectedValue > 0;

  kgwHeaderTraceR85B("r85b-selected-currency-price-writer", {
    currency,
    hasCachedSelectedValue,
    hasR81CRefresh: typeof globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C === "function",
  });

  if (hasCachedSelectedValue) {
    return kgwHeaderFormatCurrencyR85B(currency, selectedValue);
  }

  if (currency !== "USD") {
    if (typeof globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C === "function") {
      globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C("r85b-missing-selected-currency-cache");
    }

    const existing = String(currentText || "").trim();
    if (existing && !/^\$/.test(existing) && !/^USD\b/i.test(existing)) {
      return existing;
    }
  }

  return metricValue;
}

function kgwSetMetric(kind, metric) {
  const ids = kgwMetricIds(kind);
  const valueEl = document.getElementById(ids.valueId);
  const boxEl = document.getElementById(ids.boxId);

  if (!valueEl) return;

  kgwOwnMetricValueElement(kind, valueEl);

  const status = metric?.status || "error";
  const rawValue = metric?.value || (status === "error" ? "Error" : "Loading");
  const value =
    kind === "price" && status === "ok"
      ? kgwHeaderSelectedCurrencyMetricValueR85B(rawValue, valueEl.textContent)
      : rawValue;
  const source = metric?.source || "Rust backend";
  const updatedAt = kgwFormatLocalDateTime(metric?.updated_at_epoch_ms);
  const error = metric?.error || "";

  valueEl.textContent = value;

  if (kind === "price") {
    valueEl.dataset.kgwSelectedCurrency = kgwHeaderSelectedCurrencyR85B();
    valueEl.dataset.kgwPriceWriter = "KGW_HEADER_SELECTED_CURRENCY_CANONICAL_WRITER_R85B";
  }
  valueEl.dataset.liveStatus = status;

  if (kind === "price" && status === "ok") {
    kgwPublishKaspaUsdPrice(rawValue);
  }

  if (boxEl) {
    boxEl.dataset.liveStatus = status;
    boxEl.title = error
      ? `آخر تحديث: ${updatedAt}\nالمصدر: ${source}\nالقيمة: ${value}\nالحالة: ${status}\nالخطأ: ${error}`
      : `آخر تحديث: ${updatedAt}\nالمصدر: ${source}\nالقيمة: ${value}\nالحالة: ${status}`;
  }
}

function kgwSetInvokeError(message) {
  for (const kind of ["price", "hashrate", "difficulty"]) {
    kgwSetMetric(kind, {
      value: "Error",
      status: "error",
      source: "Tauri invoke",
      updated_at_epoch_ms: Date.now(),
      error: message
    });
  }
}

function kgwApplySnapshot(snapshot) {
  kgwSetMetric("price", snapshot?.price);
  kgwSetMetric("hashrate", snapshot?.hashrate);
  kgwSetMetric("difficulty", snapshot?.difficulty);

// KGW: live metrics refresh succeeds silently; errors are still logged.
}

async function kgwRefreshHeaderMetrics(force) {
  try {
    const invoke = await kgwResolveInvoke();
    const snapshot = force
      ? await invoke("kgw_live_metrics_refresh_now")
      : await invoke("kgw_live_metrics_snapshot");

    kgwApplySnapshot(snapshot);
  } catch (error) {
    const message = error?.message || String(error);
    kgwSetInvokeError(message);
    kgwLog(`live metrics invoke failed :: ${message}`);
  }
}


/* KGW_HEADER_PRICE_INIT_BINDING_OWNER_R83B
 * Bridges the selected-currency price owner into the canonical header live metrics init path.
 * This preserves the existing header-live-metrics owner instead of adding another loader.
 */
function kgwHeaderPriceTraceR83B(phase, details) {
  try {
    const payload = Object.assign(
      {
        scope: "header",
        owner: "KGW_HEADER_PRICE_INIT_BINDING_OWNER_R83B",
        phase,
      },
      details || {},
    );

    if (typeof globalThis.kgwUiTrace === "function") {
      globalThis.kgwUiTrace(payload);
    }
  } catch (_) {
    // Trace must never break header metrics.
  }
}

function kgwHeaderEnsureSelectedCurrencyPriceOwnerR83B(reason) {
  const select = document.querySelector("#shellCurrencySelect");
  const priceEl =
    document.querySelector("#kgwHeaderPrice") ||
    document.querySelector("[data-kgw-metric='price']") ||
    document.querySelector("[data-metric='price']") ||
    document.querySelector("[data-header-metric='price']");

  kgwHeaderPriceTraceR83B("r83b-header-price-owner-ensure", {
    reason: reason || "unknown",
    hasSelect: !!select,
    hasPriceElement: !!priceEl,
    hasR81CRefresh: typeof globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C === "function",
  });

  if (typeof globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C === "function") {
    globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C(reason || "header-init-r83b");
    return true;
  }

  return false;
}

function kgwHeaderBindCurrencySelectFallbackR83B() {
  const select = document.querySelector("#shellCurrencySelect");

  if (!select) {
    kgwHeaderPriceTraceR83B("r83b-currency-select-missing", {});
    return false;
  }

  if (select.dataset.kgwHeaderPriceInitBindingOwnerR83B === "1") {
    return true;
  }

  select.dataset.kgwHeaderPriceInitBindingOwnerR83B = "1";
  select.addEventListener("change", () => {
    kgwHeaderEnsureSelectedCurrencyPriceOwnerR83B("shell-currency-change-r83b");
  });

  kgwHeaderPriceTraceR83B("r83b-currency-select-bound", {
    optionCount: select.options ? select.options.length : 0,
    selectedCurrency: select.value || "",
  });

  return true;
}

function kgwHeaderBootSelectedCurrencyPriceR83B(reason) {
  kgwHeaderBindCurrencySelectFallbackR83B();
  kgwHeaderEnsureSelectedCurrencyPriceOwnerR83B(reason || "boot-r83b");
}

export function initHeaderLiveMetrics() {
  if (kgwLiveMetricsStarted) return;

  kgwLiveMetricsStarted = true;

  kgwSetClock();
  kgwRefreshHeaderMetrics(true);
  kgwHeaderBootSelectedCurrencyPriceR83B("initHeaderLiveMetrics-r83b");

  if (kgwClockTimer) window.clearInterval(kgwClockTimer);
  if (kgwMetricsTimer) window.clearInterval(kgwMetricsTimer);

  kgwClockTimer = window.setInterval(kgwSetClock, 1000);
  kgwMetricsTimer = window.setInterval(() => kgwRefreshHeaderMetrics(false), KGW_LIVE_METRICS_REFRESH_MS);

  kgwLog("header live metrics invoke module installed :: 15s refresh");
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    initHeaderLiveMetrics();
    kgwHeaderBootSelectedCurrencyPriceR83B("DOMContentLoaded-r83b");
  }, { once: true });
} else {
  initHeaderLiveMetrics();
  kgwHeaderBootSelectedCurrencyPriceR83B("module-loaded-r83b");
}

/* KGW REAL HEADER CLOCK + ENGLISH TOOLTIPS START */
const KGW_HEADER_CLOCK_TICK_MS = 1000;

function kgwPad2(value) {
  return String(value).padStart(2, "0");
}

function kgwFormatEnglishNow(date = new Date()) {
  const year = date.getFullYear();
  const month = kgwPad2(date.getMonth() + 1);
  const day = kgwPad2(date.getDate());
  const hours = kgwPad2(date.getHours());
  const minutes = kgwPad2(date.getMinutes());
  const seconds = kgwPad2(date.getSeconds());

  return {
    time: `${hours}:${minutes}:${seconds}`,
    date: `${year}-${month}-${day}`,
    stamp: `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`
  };
}

function kgwEnsureHeaderClockDom() {
  const root = document.getElementById("kgwHeaderClock");
  if (!root) {
    return null;
  }

  let timeEl = document.getElementById("kgwHeaderClockTime");
  let dateEl = document.getElementById("kgwHeaderClockDate");

  if (!timeEl || !dateEl) {
    root.innerHTML = '<div id="kgwHeaderClockTime"></div><div id="kgwHeaderClockDate"></div>';
    timeEl = document.getElementById("kgwHeaderClockTime");
    dateEl = document.getElementById("kgwHeaderClockDate");
  }

  root.hidden = false;
  root.style.display = "flex";
  root.style.visibility = "visible";
  root.style.opacity = "1";

  return { root, timeEl, dateEl };
}

function kgwRenderHeaderClock() {
  const clock = kgwEnsureHeaderClockDom();
  if (!clock) {
    return;
  }

  const now = kgwFormatEnglishNow(new Date());
  clock.timeEl.textContent = now.time;
  clock.dateEl.textContent = now.date;
  clock.root.title = `Local time: ${now.stamp}`;
}

let kgwHeaderClockTimer = null;

function kgwInstallHeaderClock() {
  kgwRenderHeaderClock();

  if (kgwHeaderClockTimer) {
    clearInterval(kgwHeaderClockTimer);
  }

  kgwHeaderClockTimer = setInterval(() => {
    kgwRenderHeaderClock();
  }, KGW_HEADER_CLOCK_TICK_MS);
}

function kgwFindMetricElement(metricKey) {
  const candidates = [
    `#kgwHeader${metricKey}Value`,
    `#kgwHeader${metricKey}`,
    `[data-kgw-metric-key="${metricKey.toLowerCase()}"]`,
    `[data-kgw-metric="${metricKey.toLowerCase()}"]`,
    `[data-kgw-header-metric="${metricKey.toLowerCase()}"]`
  ];

  for (const selector of candidates) {
    const el = document.querySelector(selector);
    if (el) {
      return el;
    }
  }

  return null;
}

function kgwReadMetricMeta(el) {
  const textValue = (el?.textContent || "N/A").trim();
  const source =
    (el?.dataset?.kgwSource || el?.dataset?.source || "Unknown").trim();

  const updatedAtRaw =
    (el?.dataset?.kgwUpdatedAt ||
     el?.dataset?.updatedAt ||
     el?.dataset?.lastUpdated ||
     "").trim();

  const updatedAt = updatedAtRaw || kgwFormatEnglishNow(new Date()).stamp;

  return {
    value: textValue || "N/A",
    source: source || "Unknown",
    updatedAt: updatedAt || "Unknown"
  };
}

function kgwBuildEnglishTooltip(label, meta) {
  return [
    `${label}`,
    `Value: ${meta.value}`,
    `Last update: ${meta.updatedAt}`,
    `Source: ${meta.source}`
  ].join("\n");
}

function kgwApplyEnglishTooltips() {
  const defs = [
    { key: "Price", label: "Price" },
    { key: "Hashrate", label: "Hashrate" },
    { key: "Difficulty", label: "Difficulty" }
  ];

  for (const def of defs) {
    const el = kgwFindMetricElement(def.key);
    if (!el) {
      continue;
    }

    const meta = kgwReadMetricMeta(el);
    el.title = kgwBuildEnglishTooltip(def.label, meta);
  }
}

function kgwRefreshClockAndEnglishTooltips() {
  kgwRenderHeaderClock();
  kgwApplyEnglishTooltips();
}

document.addEventListener("DOMContentLoaded", () => {
  kgwInstallHeaderClock();
  kgwApplyEnglishTooltips();
});

window.addEventListener("load", () => {
  kgwInstallHeaderClock();
  kgwApplyEnglishTooltips();
});

setInterval(() => {
  kgwRefreshClockAndEnglishTooltips();
}, 15000);
/* KGW REAL HEADER CLOCK + ENGLISH TOOLTIPS END */

/* KGW_HEADER_PRICE_SELECTED_CURRENCY_OWNER_R81C
 * Keeps the header KAS price synchronized with the actual shell currency select.
 * This owner is intentionally limited to header-live-metrics.js and does not touch Settings.
 */
(function kgwHeaderPriceSelectedCurrencyOwnerR81C() {
  const OWNER = "KGW_HEADER_PRICE_SELECTED_CURRENCY_OWNER_R81C";
  if (globalThis[OWNER]) return;
  globalThis[OWNER] = true;

  const DISPLAY_CURRENCIES = ["usd","sar","eur","gbp","chf","aud","cad","jpy","krw","rub","cny","try","inr","idr","hkd","sgd","brl"];

  function trace(phase, details) {
    try {
      const payload = Object.assign(
        {
          scope: "header",
          owner: OWNER,
          phase,
        },
        details || {},
      );
      if (typeof globalThis.kgwUiTrace === "function") {
        globalThis.kgwUiTrace(payload);
      }
    } catch (_) {
      // Trace must never break runtime UI.
    }
  }

  function shellCurrencySelect() {
    return document.querySelector("#shellCurrencySelect");
  }

  function selectedCurrency() {
    const select = shellCurrencySelect();
    const raw = select && select.value ? String(select.value) : "USD";
    const normalized = raw.trim().toUpperCase();
    return normalized || "USD";
  }

  function findPriceElement() {
    return (
      document.querySelector("#kgwHeaderPrice") ||
      document.querySelector("[data-kgw-metric='price']") ||
      document.querySelector("[data-metric='price']") ||
      document.querySelector("[data-header-metric='price']")
    );
  }

  function invokeCommand(command, payload) {
    const tauri = globalThis.__TAURI__;
    const invoke =
      (tauri && tauri.core && tauri.core.invoke) ||
      (tauri && tauri.tauri && tauri.tauri.invoke) ||
      globalThis.__TAURI_INVOKE__;

    if (typeof invoke !== "function") {
      throw new Error("Tauri invoke is not available for " + command);
    }

    return invoke(command, payload);
  }

  function normalizePrices(result) {
    if (!result || typeof result !== "object") return {};
    if (result.prices && typeof result.prices === "object") return result.prices;
    return result;
  }

  function numericPrice(prices, currency) {
    const lower = String(currency || "USD").toLowerCase();
    const upper = lower.toUpperCase();
    const direct = prices[lower] ?? prices[upper];
    const value = Number(direct);
    return Number.isFinite(value) && value > 0 ? value : null;
  }

  function renderSelectedCurrencyPrice(prices, reason) {
    const currency = selectedCurrency();
    const value = numericPrice(prices, currency);
    const el = findPriceElement();

    trace("r81c-price-selected-currency", {
      currency,
      hasElement: !!el,
      hasValue: value !== null,
      reason: reason || "unknown",
    });

    if (!el || value === null) return false;

    const formatted = new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      maximumFractionDigits: value >= 1 ? 4 : 8,
    }).format(value);

    el.textContent = formatted;
    el.setAttribute("data-kgw-selected-currency", currency);
    el.setAttribute("data-kgw-price-source", "kgw_get_kaspa_prices");
    return true;
  }

  async function refreshSelectedCurrencyPrice(reason) {
    const currency = selectedCurrency();

    trace("r81c-currency-price-refresh", {
      currency,
      reason: reason || "manual",
    });

    try {
      const result = await invokeCommand("kgw_get_kaspa_prices");
      const prices = normalizePrices(result);
      globalThis.kgwHeaderLastKaspaPricesR81C = prices;
      renderSelectedCurrencyPrice(prices, reason || "refresh");
    } catch (error) {
      trace("r81c-currency-price-refresh-error", {
        currency,
        message: error && error.message ? error.message : String(error),
      });
    }
  }

  function bindCurrencySelect() {
    const select = shellCurrencySelect();

    if (!select) {
      trace("r81c-currency-select-missing", {});
      return false;
    }

    if (select.dataset.kgwHeaderPriceCurrencyOwnerR81C === "1") {
      return true;
    }

    select.dataset.kgwHeaderPriceCurrencyOwnerR81C = "1";

    select.addEventListener("change", () => {
      refreshSelectedCurrencyPrice("shell-currency-change");
    });

    trace("r81c-currency-select-bound", {
      optionCount: select.options ? select.options.length : 0,
      selectedCurrency: selectedCurrency(),
      displayCurrencies: DISPLAY_CURRENCIES,
    });

    return true;
  }

  function boot() {
    bindCurrencySelect();
    refreshSelectedCurrencyPrice("boot");
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot, { once: true });
  } else {
    boot();
  }

  globalThis.kgwHeaderRefreshSelectedCurrencyPriceR81C = refreshSelectedCurrencyPrice;
})();

