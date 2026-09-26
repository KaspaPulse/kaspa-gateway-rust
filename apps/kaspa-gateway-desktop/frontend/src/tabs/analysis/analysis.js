import {
  analysisCalendarInstall,
  analysisCalendarResetToToday,
  analysisViewApplyFilter,
  analysisViewFilteredRows,
  analysisViewRenderRows,
  analysisViewSetData,
  installRustAnalysisBinding
} from "./analysis-rust-binding.js";

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

function toWesternDigits(value) {
  return String(value ?? "")
    .replace(/[٠-٩]/g, (digit) => String("٠١٢٣٤٥٦٧٨٩".indexOf(digit)))
    .replace(/[۰-۹]/g, (digit) => String("۰۱۲۳۴۵۶۷۸۹".indexOf(digit)));
}

function setText(selector, value) {
  const node = q(selector);
  if (node) node.textContent = String(value ?? "");
}

function applyFilter() {
  return analysisViewApplyFilter();
}

function renderRows() {
  return analysisViewRenderRows();
}

function setAnalysisData(payload = {}) {
  return analysisViewSetData(payload);
}

function kgwAnalysisPythonBindR9() {
  return analysisViewRenderRows();
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
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
}

/* Analysis calendar/date/filter-layout ownership moved to Rust/WASM. */


/* Analysis export ownership moved to Rust/WASM in analysis_binding.rs. */


function bindControls() {
  kgwAnalysisPythonBindR9();

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
      analysisCalendarResetToToday();
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
  analysisCalendarInstall();
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
