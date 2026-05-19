

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


function kgwSetMetric(kind, metric) {
  const ids = kgwMetricIds(kind);
  const valueEl = document.getElementById(ids.valueId);
  const boxEl = document.getElementById(ids.boxId);

  if (!valueEl) return;

  kgwOwnMetricValueElement(kind, valueEl);

  const status = metric?.status || "error";
  const value = metric?.value || (status === "error" ? "Error" : "Loading");
  const source = metric?.source || "Rust backend";
  const updatedAt = kgwFormatLocalDateTime(metric?.updated_at_epoch_ms);
  const error = metric?.error || "";

  valueEl.textContent = value;
  valueEl.dataset.liveStatus = status;

  if (kind === "price" && status === "ok") {
    kgwPublishKaspaUsdPrice(value);
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

export function initHeaderLiveMetrics() {
  if (kgwLiveMetricsStarted) return;

  kgwLiveMetricsStarted = true;

  kgwSetClock();
  kgwRefreshHeaderMetrics(true);

  if (kgwClockTimer) window.clearInterval(kgwClockTimer);
  if (kgwMetricsTimer) window.clearInterval(kgwMetricsTimer);

  kgwClockTimer = window.setInterval(kgwSetClock, 1000);
  kgwMetricsTimer = window.setInterval(() => kgwRefreshHeaderMetrics(false), KGW_LIVE_METRICS_REFRESH_MS);

  kgwLog("header live metrics invoke module installed :: 15s refresh");
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initHeaderLiveMetrics, { once: true });
} else {
  initHeaderLiveMetrics();
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

