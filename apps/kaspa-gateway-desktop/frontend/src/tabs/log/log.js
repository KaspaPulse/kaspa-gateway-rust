const LOG_STORAGE_KEY = "kgw-log-viewer-state";

let logLines = [];
let filteredLines = [];
let pollTimer = null;
let copyStatusTimer = null;

function root() {
  return document.getElementById("log") || document;
}

function qs(selector) {
  return root().querySelector(selector) || document.querySelector(selector);
}

function currentUiLanguage() {
  const selector =
    document.querySelector("#languageSelect") ||
    document.querySelector("#language") ||
    document.querySelector("select[name='language']");

  const selectedText = String(selector?.selectedOptions?.[0]?.textContent || "").trim().toLowerCase();
  const selectedValue = String(selector?.value || "").trim().toLowerCase();
  const htmlLang = String(document.documentElement.lang || "").trim().toLowerCase();

  const signal = [selectedValue, selectedText, htmlLang].join(" ");

  if (
    signal === "ar" ||
    signal.includes(" ar ") ||
    signal.includes("arabic") ||
    signal.includes("العربية")
  ) {
    return "ar";
  }

  return "en";
}

function copyText(key) {
  const lang = currentUiLanguage();

  const messages = {
    copiedButton: {
      en: "Copied",
      ar: "تم النسخ"
    },
    copiedStatus: {
      en: "Copied to clipboard.",
      ar: "تم النسخ إلى الحافظة."
    },
    copyFailedButton: {
      en: "Copy failed",
      ar: "فشل النسخ"
    },
    copyFailedStatus: {
      en: "Copy failed. Clipboard access is unavailable or was rejected.",
      ar: "فشل النسخ. الوصول إلى الحافظة غير متاح أو تم رفضه."
    }
  };

  return messages[key]?.[lang] || messages[key]?.en || "";
}

function invoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
}

function loadState() {
  try {
    return JSON.parse(localStorage.getItem(LOG_STORAGE_KEY) || "{}");
  } catch {
    return {};
  }
}

function saveState() {
  const state = {
    severity: qs("#logSeverity")?.value || "ALL",
    search: qs("#logSearch")?.value || "",
    autoScroll: Boolean(qs("#logAutoScroll")?.checked),
    fontSize: qs("#logFontSize")?.value || "9"
  };

  localStorage.setItem(LOG_STORAGE_KEY, JSON.stringify(state));
}

export function kgwParseLogLevel(line) {
  const text = String(line || "");
  const structured = text.match(/(?:^|\s)level="(TRACE|DEBUG|INFO|WARN|ERROR)"(?:\s|$)/i);
  if (structured) return structured[1].toUpperCase();

  const native = text.match(/^\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}(?:[.,]\d+)?\s+-\s+(TRACE|DEBUG|INFO|WARN|ERROR)\s+-\s+\[[^\]]+\]\s+-\s+/i);
  return native ? native[1].toUpperCase() : "UNKNOWN";
}

function severityRank(level) {
  return {
    TRACE: 0,
    DEBUG: 1,
    INFO: 2,
    WARN: 3,
    ERROR: 4
  }[String(level || "INFO").toUpperCase()] ?? 2;
}

function shouldShow(line) {
  const severity = qs("#logSeverity")?.value || "ALL";
  const search = String(qs("#logSearch")?.value || "").trim().toLowerCase();
  const level = kgwParseLogLevel(line);

  if (severity !== "ALL" && (level === "UNKNOWN" || severityRank(level) < severityRank(severity))) {
    return false;
  }

  if (search && !String(line).toLowerCase().includes(search)) {
    return false;
  }

  return true;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function highlightLine(line) {
  const safe = escapeHtml(line);

  return safe
    .replace(/time=&quot;([^&]*)&quot;/g, '<span class="log-time">time="$1"</span>')
    .replace(/level=&quot;([^&]*)&quot;/g, '<span class="log-level">level="$1"</span>')
    .replace(/target=&quot;([^&]*)&quot;/g, '<span class="log-target">target="$1"</span>')
    .replace(/msg=&quot;([^&]*)&quot;/g, '<span class="log-message">msg="$1"</span>');
}

function render() {
  const output = qs("#logOutput");
  if (!output) return;

  filteredLines = logLines.filter(shouldShow);
  output.innerHTML = filteredLines.map(highlightLine).join("\n");

  const fontSize = qs("#logFontSize")?.value || "9";
  output.style.fontSize = `${fontSize}px`;

  if (qs("#logAutoScroll")?.checked) {
    output.scrollTop = output.scrollHeight;
  }

  saveState();
}

async function refreshLog() {
  const call = invoke();
  const output = qs("#logOutput");

  if (!call) {
    if (output) output.textContent = (window.kgwT ? window.kgwT("log.tauriInvokeApiUnavailable") : "Tauri invoke API is not available.");
    return;
  }

  try {
    logLines = await call("kgw_log_read", { maxLines: 5000 });
    render();
  } catch (error) {
    if (output) output.textContent = `Failed to read log: ${error?.message || error}`;
  }
}

async function clearLog() {
  const call = invoke();
  if (!call) return;

  await call("kgw_log_clear");
  await refreshLog();
}

function showCopyStatus(message, success = true) {
  let node = qs("#logCopyStatus");

  if (!node) {
    node = document.createElement("span");
    node.id = "logCopyStatus";
    node.className = "log-copy-status";
    node.setAttribute("role", "status");
    node.setAttribute("aria-live", "polite");

    const host = qs("#logCopy")?.parentElement || root();
    host.appendChild(node);
  }

  node.hidden = false;
  node.textContent = message;

  const button = qs("#logCopy");
  const oldText = button?.dataset.originalText || button?.textContent || "";

  if (button) {
    if (!button.dataset.originalText) button.dataset.originalText = oldText;
    button.textContent = copyText(success ? "copiedButton" : "copyFailedButton");
  }

  clearTimeout(copyStatusTimer);
  copyStatusTimer = setTimeout(() => {
    node.textContent = "";
    node.hidden = true;

    if (button?.dataset.originalText) {
      button.textContent = button.dataset.originalText;
    }
  }, 2200);
}

export async function kgwCopyTextToClipboard(text, env = {}) {
  const clipboard = Object.prototype.hasOwnProperty.call(env, "clipboard")
    ? env.clipboard
    : globalThis.navigator?.clipboard;
  let apiError = null;

  if (clipboard && typeof clipboard.writeText === "function") {
    try {
      await clipboard.writeText(String(text));
      return true;
    } catch (error) {
      apiError = error;
    }
  }

  const doc = Object.prototype.hasOwnProperty.call(env, "document") ? env.document : globalThis.document;
  if (!doc || typeof doc.createElement !== "function" || typeof doc.execCommand !== "function" || !doc.body) {
    throw apiError || new Error("Clipboard copy is unavailable.");
  }

  const textarea = doc.createElement("textarea");
  textarea.value = String(text);
  doc.body.appendChild(textarea);
  try {
    textarea.select();
    const copied = doc.execCommand("copy");
    if (copied !== true) throw new Error("Clipboard fallback was rejected.");
    return true;
  } finally {
    textarea.remove();
  }
}

async function copyLog() {
  const text = filteredLines.join("\n");
  try {
    await kgwCopyTextToClipboard(text);
    showCopyStatus(copyText("copiedStatus"), true);
    return true;
  } catch (error) {
    showCopyStatus(copyText("copyFailedStatus"), false);
    throw error;
  }
}

function applyInitialState() {
  const state = loadState();

  if (qs("#logSeverity") && state.severity) qs("#logSeverity").value = state.severity;
  if (qs("#logSearch") && state.search) qs("#logSearch").value = state.search;
  if (qs("#logAutoScroll")) qs("#logAutoScroll").checked = state.autoScroll !== false;
  if (qs("#logFontSize") && state.fontSize) qs("#logFontSize").value = state.fontSize;
}


// KGW_LOG_UI_TRACE_PATCH_R49B2
function kgwLogUiTraceR49B2(action, phase, details) {
  try {
    const call = invoke();
    if (typeof call !== "function") return;
    const safeAction = String(action || "log-ui");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    call("kgw_frontend_button_trace_v1", {
      scope: "log",
      net: "ui",
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_LOG_UI_TRACE_PATCH_R49B2",
        owner: "log-existing-initLogTab-owner",
        action: safeAction,
        phase: safePhase,
        details: safeDetails
      })
    }).catch(function () {});
  } catch (_) { /* Best-effort secondary operation; primary behavior is preserved. */ }
}
export function initLogTab() {
  applyInitialState();

  qs("#logSeverity")?.addEventListener("change", (event) => {
    kgwLogUiTraceR49B2("log-filter", "r49b2-log-severity-change", {
      trusted: Boolean(event && event.isTrusted),
      value: String(qs("#logSeverity")?.value || "")
    });
    render();
  });
  qs("#logSearch")?.addEventListener("input", (event) => {
    kgwLogUiTraceR49B2("log-filter", "r49b2-log-search-input", {
      trusted: Boolean(event && event.isTrusted),
      valueLength: String(qs("#logSearch")?.value || "").length
    });
    render();
  });
  qs("#logAutoScroll")?.addEventListener("change", (event) => {
    kgwLogUiTraceR49B2("log-option", "r49b2-log-autoscroll-change", {
      trusted: Boolean(event && event.isTrusted),
      checked: Boolean(qs("#logAutoScroll")?.checked)
    });
    saveState();
  });
  qs("#logFontSize")?.addEventListener("change", (event) => {
    kgwLogUiTraceR49B2("log-option", "r49b2-log-font-size-change", {
      trusted: Boolean(event && event.isTrusted),
      value: String(qs("#logFontSize")?.value || "")
    });
    render();
  });
  qs("#logClear")?.addEventListener("click", (event) => {
    kgwLogUiTraceR49B2("log-action", "r49b2-log-clear-click", {
      trusted: Boolean(event && event.isTrusted),
      visibleLines: filteredLines.length
    });
    clearLog().catch(console.error);
  });
  qs("#logCopy")?.addEventListener("click", (event) => {
    kgwLogUiTraceR49B2("log-action", "r49b2-log-copy-click", {
      trusted: Boolean(event && event.isTrusted),
      visibleLines: filteredLines.length
    });
    copyLog().catch(console.error);
  });

  if (pollTimer) clearInterval(pollTimer);

  refreshLog();
  pollTimer = setInterval(refreshLog, 1200);
}

window.initLogTab = initLogTab;
