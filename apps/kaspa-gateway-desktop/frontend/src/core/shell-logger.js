const LOG_STORAGE_KEY = "kgw-frontend-log-buffer";
const MAX_BUFFER = 1500;

function safeDetails(details) {
  try {
    if (details instanceof Error) {
      return details.stack || details.message || String(details);
    }

    if (typeof details === "object" && details !== null) {
      return JSON.stringify(details);
    }

    return String(details ?? "");
  } catch (_) {
    return String(details ?? "");
  }
}

function pushLocal(entry) {
  try {
    window.__KGW_FRONTEND_LOGS = window.__KGW_FRONTEND_LOGS || [];
    window.__KGW_FRONTEND_LOGS.push(entry);

    while (window.__KGW_FRONTEND_LOGS.length > MAX_BUFFER) {
      window.__KGW_FRONTEND_LOGS.shift();
    }
  } catch (_) {}

  try {
    const existing = JSON.parse(localStorage.getItem(LOG_STORAGE_KEY) || "[]");
    existing.push(entry);

    while (existing.length > MAX_BUFFER) {
      existing.shift();
    }

    localStorage.setItem(LOG_STORAGE_KEY, JSON.stringify(existing));
  } catch (_) {}
}

function getTauriInvoke() {
  return (
    window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI_INVOKE__ ||
    null
  );
}

function sendToTauriLog(entry) {
  try {
    const invoke = getTauriInvoke();
    if (!invoke) return;

    const details = entry.details ? ` :: ${entry.details}` : "";
    const target = entry.source ? `frontend:${entry.source}` : "frontend";

    invoke("kgw_log_append", {
      request: {
        level: String(entry.level || "log").toUpperCase(),
        target,
        message: `${entry.message}${details}`
      }
    }).catch(() => {});
  } catch (_) {}
}

export function kgwLog(level, message, details = "", source = "frontend") {
  const entry = {
    time: new Date().toISOString(),
    level: String(level || "log"),
    source,
    message: String(message || ""),
    details: safeDetails(details),
    href: location.href
  };

  const line = `[KGW-FRONTEND][${entry.level}][${entry.source}] ${entry.time} - ${entry.message}${entry.details ? " :: " + entry.details : ""}`;

  try {
    if (entry.level === "error") console.error(line);
    else if (entry.level === "warn") console.warn(line);
    else console.log(line);
  } catch (_) {}

  pushLocal(entry);
  sendToTauriLog(entry);

  return entry;
}

export function createLogger(source) {
  return {
    log: (message, details = "") => kgwLog("log", message, details, source),
    warn: (message, details = "") => kgwLog("warn", message, details, source),
    error: (message, details = "") => kgwLog("error", message, details, source)
  };
}

export function kgwFatal(error, source = "shell") {
  kgwLog("error", "fatal", error, source);

  let panel = document.getElementById("kgwFatalPanel");

  if (!panel) {
    panel = document.createElement("pre");
    panel.id = "kgwFatalPanel";
    panel.style.cssText = [
      "position:fixed",
      "right:12px",
      "bottom:12px",
      "width:min(720px, calc(100vw - 24px))",
      "max-height:260px",
      "overflow:auto",
      "z-index:999999",
      "padding:10px",
      "margin:0",
      "border:1px solid rgba(248,113,113,.65)",
      "border-radius:8px",
      "background:#111827",
      "color:#fca5a5",
      "font:12px Consolas, monospace",
      "white-space:pre-wrap",
      "box-shadow:0 12px 28px rgba(0,0,0,.42)"
    ].join(";");

    document.body.appendChild(panel);
  }

  panel.textContent =
    ((window.kgwT ? window.kgwT("runtime.shellErrorNavigationPreserved") : "Shell/runtime error. Navigation was preserved.") + "\n\n") +
    String(error && error.stack || error);
}

export function getBufferedLogs() {
  try {
    return JSON.parse(localStorage.getItem(LOG_STORAGE_KEY) || "[]");
  } catch (_) {
    return [];
  }
}

export function clearBufferedLogs() {
  try {
    localStorage.removeItem(LOG_STORAGE_KEY);
  } catch (_) {}

  try {
    window.__KGW_FRONTEND_LOGS = [];
  } catch (_) {}
}

if (!window.__KGW_LOGGER_INSTALLED) {
  window.__KGW_LOGGER_INSTALLED = true;

  window.kgwLog = kgwLog;
  window.kgwFatal = kgwFatal;
  window.kgwCreateLogger = createLogger;
  window.kgwGetBufferedLogs = getBufferedLogs;
  window.kgwClearBufferedLogs = clearBufferedLogs;

  window.addEventListener("error", (event) => {
    kgwLog("error", "window error", {
      message: event.message,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      error: event.error && (event.error.stack || event.error.message)
    }, "window");
  });

  window.addEventListener("unhandledrejection", (event) => {
    kgwLog("error", "unhandled rejection", event.reason && (event.reason.stack || event.reason.message || event.reason), "promise");
  });

  kgwLog("log", "logger installed", "", "logger");
}

/* KGW clear log on startup runner */
async function kgwClearLogOnStartupOnce() {
  if (window.__kgwClearLogOnStartupDone) return;
  window.__kgwClearLogOnStartupDone = true;

  try {
    if (localStorage.getItem("kgw.clearLogOnStartup") !== "1") return;

    const call =
      window.__TAURI__?.core?.invoke ||
      window.__TAURI__?.tauri?.invoke ||
      window.__TAURI_INVOKE__;

    if (!call) return;

    await call("kgw_log_clear");
    console.info("[KGW] log cleared on startup by user setting");
  } catch (error) {
    console.warn("[KGW] failed to clear log on startup", error);
  }
}

kgwClearLogOnStartupOnce();
