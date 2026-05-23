/*
 * KGW_EXPLORER_EXPORT_NATIVE_SAVE_PHASE_A_V8
 * KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3
 * KGW_EXPLORER_EXPORT_VISIBLE_RESULT_V4C
 * KGW_EXPLORER_EXPORT_SAVED_OPEN_PROMPT_V5B
 *
 * Explorer export owner:
 * - Delegates CSV/HTML writing to Rust export_commands.rs.
 * - Sends runtime traces to Rust kgw_frontend_button_trace_v1 so PowerShell shows button activity.
 * - Keeps PDF intentionally deferred to Phase 2.
 * - Does not use Blob/download.
 * - Does not introduce Save As/Open File.
 */

const KGW_EXPLORER_EXPORT_OWNER_MARKER = "KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3";

function qs(selector, scope = document) {
  return scope ? scope.querySelector(selector) : null;
}

function qsa(selector, scope = document) {
  return scope ? Array.from(scope.querySelectorAll(selector)) : [];
}

function invoke() {
  return window.__TAURI__?.core?.invoke ||
    window.__TAURI__?.tauri?.invoke ||
    window.__TAURI__?.invoke ||
    window.__TAURI_INVOKE__;
}

function currentLocale() {
  return document.documentElement.getAttribute("lang") ||
    window.kgwCurrentLocale ||
    window.localStorage?.getItem?.("kgw.locale") ||
    "en";
}

function safeJson(value) {
  try {
    return JSON.stringify(value);
  } catch (_) {
    return String(value);
  }
}

function exportButtonInfo(button) {
  if (!button) return {};
  const rect = button.getBoundingClientRect?.();
  return {
    id: button.id || "",
    text: String(button.textContent || "").trim(),
    disabled: Boolean(button.disabled),
    ariaDisabled: button.getAttribute?.("aria-disabled") || "",
    hidden: Boolean(button.hidden),
    display: window.getComputedStyle(button).display,
    visibility: window.getComputedStyle(button).visibility,
    width: rect ? Math.round(rect.width) : null,
    height: rect ? Math.round(rect.height) : null
  };
}

function traceExport(phase, details = {}) {
  const payload = {
    marker: KGW_EXPLORER_EXPORT_OWNER_MARKER,
    at: new Date().toISOString(),
    ...details
  };

  console.info("[KGW][export][explorer][trace]", phase, payload);

  const call = invoke();
  if (!call) return Promise.resolve(false);

  try {
    return Promise.resolve(call("kgw_frontend_button_trace_v1", {
      scope: "explorer",
      net: "ui",
      action: "export",
      phase,
      details: safeJson(payload)
    })).catch((error) => {
      console.warn("[KGW][export][explorer][trace-failed]", phase, error);
      return false;
    });
  } catch (error) {
    console.warn("[KGW][export][explorer][trace-failed]", phase, error);
    return Promise.resolve(false);
  }
}

function cleanHeader(value) {
  return String(value || "").replace(/↕/g, "").trim();
}

function setExportStatus(section, message, level = "info") {
  const status =
    qs("#explorerStatus", section) ||
    qs("[data-explorer-status]", section) ||
    qs(".explorer-status", section);

  if (status) {
    status.hidden = false;
    status.removeAttribute("hidden");
    status.textContent = message;
    status.dataset.kgwExplorerExportStatus = level;
  }

  console.info("[KGW][export][explorer]", message);
}

function selectedAddress(section) {
  return String(qs("#explorerAddress", section)?.value || "").trim();
}

function isKaspaAddress(value) {
  return /^kaspa[a-z0-9]*:/i.test(String(value || "").trim());
}

function explorerTable(section) {
  return qs("#explorerTransactionsTable", section) ||
    qs("[data-explorer-transactions-table]", section) ||
    qs("table", section);
}

function isEmptyExplorerRow(row) {
  const text = String(row.join(" ")).trim().toLowerCase();
  if (!text) return true;
  if (text.includes("load an address")) return true;
  if (text.includes("no explorer rows")) return true;
  if (text.includes("enter a valid kaspa address")) return true;
  if (text.includes("table cleared")) return true;
  return false;
}

// KGW_EXPORT_TEMPLATE_PARITY_EXPLORER_URLS_V1B
function kgwExplorerExportCleanUrlV1B(value) {
  const text = String(value || "").trim();
  if (!text) return "";
  return text.replace(/[\\s'"<>]+$/g, "");
}

function kgwExplorerExportClassifyUrlV1B(url) {
  const clean = kgwExplorerExportCleanUrlV1B(url);
  const lower = clean.toLowerCase();

  if (/\/txs?\//i.test(lower) || /\/transactions?\//i.test(lower)) return "tx";
  if (/\/addresses?\//i.test(lower) || lower.includes("kaspa:")) return "address";
  return "";
}

function kgwExplorerExportUniqueV1B(values) {
  const seen = new Set();
  const out = [];

  for (const value of values) {
    const clean = kgwExplorerExportCleanUrlV1B(value);
    if (!clean || seen.has(clean)) continue;
    seen.add(clean);
    out.push(clean);
  }

  return out;
}

function buildExplorerClientTable(section) {
  const table = explorerTable(section);
  if (!table) {
    throw new Error("Explorer transactions table was not found.");
  }

  const baseHeaders = qsa("thead th", table).map((th) => cleanHeader(th.textContent));

  const rowObjects = qsa("tbody tr", table)
    .map((tr) => {
      const cells = qsa("td", tr);
      const values = cells.map((td) => String(td.textContent || "").trim());
      const urls = kgwExplorerExportUniqueV1B(
        cells.flatMap((td) =>
          qsa("a[href]", td)
            .map((a) => a.getAttribute("href") || a.href || "")
            .filter(Boolean)
        )
      );

      const txUrls = urls.filter((url) => kgwExplorerExportClassifyUrlV1B(url) === "tx");
      const addressUrls = urls.filter((url) => kgwExplorerExportClassifyUrlV1B(url) === "address");

      return {
        values,
        txUrl: txUrls[0] || "",
        addressUrls: addressUrls.join(" | ")
      };
    })
    .filter((row) => row.values.length && row.values.some((cell) => cell !== ""))
    .filter((row) => !isEmptyExplorerRow(row.values));

  const includeTxUrl = rowObjects.some((row) => row.txUrl);
  const includeAddressUrl = rowObjects.some((row) => row.addressUrls);

  const headers = [...baseHeaders];
  if (includeTxUrl) headers.push("Transaction URL");
  if (includeAddressUrl) headers.push("Address URL");

  const rows = rowObjects.map((row) => {
    const out = [...row.values];
    if (includeTxUrl) out.push(row.txUrl);
    if (includeAddressUrl) out.push(row.addressUrls);
    return out;
  });

  traceExport("table-scan", {
    headers: headers.length,
    rows: rows.length,
    includeTxUrl,
    includeAddressUrl
  });

  if (!headers.length || !rows.length) {
    throw new Error("No explorer rows are available for export. Fetch transactions first, then export.");
  }

  return {
    title: "Kaspa Gateway Explorer Transactions",
    subtitle: "Exported from Explorer tab",
    headers,
    rows
  };
}

function kgwExportT(key) {
  const candidates = [
    window.kgwT,
    window.kgwTranslate,
    window.t,
    window.__KGW_I18N__?.t,
    window.kgwI18n?.t,
    window.KGWI18n?.t,
    window.i18n?.t
  ];

  for (const fn of candidates) {
    if (typeof fn !== "function") continue;
    try {
      const value = fn(key);
      if (typeof value === "string" && value && value !== key) return value;
    } catch (_) {
      continue;
    }
  }

  return key;
}

function hideExportResultBox(section) {
  const root = section && section.querySelector ? section : document;
  const box = qs("#explorerExportResult", root) || document.getElementById("explorerExportResult");
  if (box) {
    box.style.display = "none";
    box.textContent = "";
  }
}

function hideExportStatus(section) {
  const root = section && section.querySelector ? section : document;
  const status =
    qs("#explorerStatus", root) ||
    qs("[data-explorer-status]", root) ||
    qs(".explorer-status", root);

  if (status) {
    status.textContent = "";
    status.hidden = true;
    status.setAttribute("hidden", "");
  }
}














function kgwExplorerDialogApiV9() {
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


async function kgwExplorerLoadNativeDialogV8() {
  return kgwExplorerDialogApiV9();
}

function kgwExplorerNativeDialogFilterV8(format) {
  const ext = String(format || "").replace(/^\./, "").toLowerCase();
  return {
    name: ext ? `${ext.toUpperCase()} files` : "Export files",
    extensions: ext ? [ext] : []
  };
}

async function promptExportSavePath(section, format, defaultPath) {
  const dialog = await kgwExplorerLoadNativeDialogV8();
  const selected = await dialog.save({
    title: kgwExportT("explorer.export.modal.saveTitle"),
    defaultPath,
    filters: [kgwExplorerNativeDialogFilterV8(format)]
  });

  await traceExport("native-save-dialog-result-v8", {
    format,
    accepted: Boolean(selected),
    hasPath: Boolean(selected)
  });

  hideExportResultBox(section);
  hideExportStatus(section);

  return selected ? String(selected) : null;
}

async function copyExportPathToClipboard(path) {
  try {
    if (!path || !navigator.clipboard || typeof navigator.clipboard.writeText !== "function") {
      await traceExport("copy-path-skipped", {
        reason: "clipboard-api-unavailable",
        outputPath: path || ""
      });
      return false;
    }

    await navigator.clipboard.writeText(path);
    await traceExport("copy-path-success", {
      outputPath: path
    });
    return true;
  } catch (error) {
    await traceExport("copy-path-error", {
      outputPath: path || "",
      error: error && error.message ? error.message : String(error)
    });
    return false;
  }
}

function ensureExportResultBox(section) {
  const root = section && section.querySelector ? section : document;
  let box = qs("#explorerExportResult", root) || document.getElementById("explorerExportResult");
  if (box) return box;

  box = document.createElement("div");
  box.id = "explorerExportResult";
  box.setAttribute("role", "status");
  box.setAttribute("aria-live", "polite");
  box.style.marginTop = "8px";
  box.style.padding = "8px 10px";
  box.style.border = "1px solid rgba(120, 120, 120, 0.35)";
  box.style.borderRadius = "8px";
  box.style.fontSize = "12px";
  box.style.lineHeight = "1.45";
  box.style.wordBreak = "break-all";
  box.style.whiteSpace = "normal";
  box.style.display = "none";

  const status =
    qs("#explorerStatus", root) ||
    qs("[data-explorer-status]", root) ||
    qs(".explorer-status", root);

  if (status && status.parentNode) {
    status.parentNode.insertBefore(box, status.nextSibling);
    return box;
  }

  const exportCsv = document.getElementById("explorerExportCsv");
  if (exportCsv && exportCsv.parentNode) {
    exportCsv.parentNode.appendChild(box);
    return box;
  }

  document.body.appendChild(box);
  return box;
}

function buildExportResultMessage(format, finalPath, copied) {
  const parts = [];
  parts.push("Export");
  parts.push(" completed");
  parts.push(" (");
  parts.push(String(format).toUpperCase());
  parts.push("): ");
  parts.push(finalPath);
  parts.push(copied ? " | Path copied to clipboard." : " | Copy this path manually.");
  return parts.join("");
}

function buildExportSavedPromptMessage() {
  const parts = [];
  parts.push("تم");
  parts.push(" الحفظ.");
  parts.push(" هل");
  parts.push(" تريد");
  parts.push(" فتح");
  parts.push(" الملف؟");
  return parts.join("");
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


async function openExportedFileAfterPrompt(section, finalPath, format) {
  const accepted = await kgwExportCenteredOpenPromptV10();

  await traceExport("centered-open-file-prompt-v10", {
    format,
    accepted: Boolean(accepted),
    outputPath: finalPath
  });

  if (!accepted) return false;

  const call = invoke();
  if (!call) {
    await traceExport("open-file-error", {
      format,
      outputPath: finalPath,
      error: "tauri invoke unavailable"
    });
    return false;
  }

  try {
    await call("kgw_open_exported_file_v1", { path: finalPath });
    await traceExport("open-file-success", {
      format,
      outputPath: finalPath
    });
    return true;
  } catch (error) {
    await traceExport("open-file-error", {
      format,
      outputPath: finalPath,
      error: error && error.message ? error.message : String(error)
    });
    return false;
  }
}

function showExportResult(section, format, finalPath, copied) {
  hideExportResultBox(section);
  hideExportStatus(section);

  traceExport("native-save-result-v8", {
    format,
    outputPath: finalPath,
    copiedToClipboard: Boolean(copied)
  });

  openExportedFileAfterPrompt(section, finalPath, format);
}

async function runExplorerBackendExport(section, format, sourceButton = null) {
  await traceExport("run-start", {
    format,
    button: exportButtonInfo(sourceButton)
  });

  const call = invoke();
  if (!call) {
    throw new Error("Tauri invoke API is not available.");
  }

  // KGW_EXPORT_RAW_PAYLOAD_PARITY_EXPLORER_ROUTE_V2
  const clientTable = typeof window.__kgwExplorerBuildRawExportTableV2 === "function"
    ? await window.__kgwExplorerBuildRawExportTableV2(section)
    : buildExplorerClientTable(section);
  const address = selectedAddress(section);

  await traceExport("default-path-start", {
    format,
    reportType: "ExplorerTransactions",
    rows: clientTable.rows.length
  });

  const outputPath = await call("export_default_path", {
    reportType: "ExplorerTransactions",
    format
  });

  await traceExport("default-path-done", {
    format,
    outputPath
  });

  const selectedOutputPath = await promptExportSavePath(section, format, outputPath);
  if (!selectedOutputPath) {
    hideExportResultBox(section);
    hideExportStatus(section);
    await traceExport("save-path-cancelled", {
      format,
      outputPath
    });
    return;
  }

  const request = {
    reportType: "ExplorerTransactions",
    format,
    outputPath: selectedOutputPath,
    addressFilter: isKaspaAddress(address) ? address : null,
    timeRange: "all",
    limit: 100000,
    locale: currentLocale(),
    clientTable
  };

  await traceExport("export-report-start", {
    format,
    outputPath: selectedOutputPath,
    rows: clientTable.rows.length,
    locale: request.locale
  });

  const result = await call("export_report", { request });
  const finalPath = result.output_path || result.outputPath || selectedOutputPath;

  await traceExport("success", {
    format,
    outputPath: finalPath,
    rowsExported: result.rows_exported || result.rowsExported || clientTable.rows.length,
    bytesWritten: result.bytes_written || result.bytesWritten || null
  });

  const copied = await copyExportPathToClipboard(finalPath);
  hideExportStatus(section);
  showExportResult(section, format, finalPath, copied);
}

function nearestExplorerSection(node) {
  return node?.closest?.("#explorer") || qs("#explorer") || document;
}

function buttonFormat(button) {
  if (!button) return null;
  if (button.id === "explorerExportCsv") return "csv";
  if (button.id === "explorerExportHtml") return "html";
  if (button.id === "explorerExportPdf") return "pdf";
  return null;
}

function findExportButton(event) {
  const target = event?.target;
  if (!target?.closest) return null;
  return target.closest("#explorerExportCsv, #explorerExportHtml, #explorerExportPdf");
}

async function handleExportButtonClick(event) {
  const button = findExportButton(event);
  if (!button) return;

  const format = buttonFormat(button);
  const section = nearestExplorerSection(button);

  event.preventDefault();
  event.stopPropagation();
  event.stopImmediatePropagation?.();

  await traceExport("r53b3-explorer-export-owner-click", {
    format,
    button: exportButtonInfo(button)
  });

  await traceExport("click", {
    format,
    button: exportButtonInfo(button)
  });

  if (button.disabled || button.getAttribute("aria-disabled") === "true") {
    setExportStatus(section, "Export button is disabled. Fetch transactions first, then export.", "blocked");
    await traceExport("blocked-disabled", {
      format,
      button: exportButtonInfo(button)
    });
    return;
  }

  try {
    await runExplorerBackendExport(section, format, button);
  } catch (error) {
    const message = error?.message || String(error);
    setExportStatus(section, message, "error");
    await traceExport("error", {
      format,
      error: message,
      stack: error?.stack || ""
    });
  }
}

function installExplorerExportClickOwner() {
  if (window.__KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3_INSTALLED) return;
  window.__KGW_EXPLORER_EXPORT_CLICK_TRACE_OWNER_V3_INSTALLED = true;

  document.addEventListener("pointerdown", (event) => {
    const button = findExportButton(event);
    if (!button) return;

    traceExport("pointerdown", {
      format: buttonFormat(button),
      button: exportButtonInfo(button)
    });
  }, true);

  document.addEventListener("click", (event) => {
    const button = findExportButton(event);
    if (!button) return;
    handleExportButtonClick(event);
  }, true);

  traceExport("owner-installed", {
    csvExists: Boolean(document.getElementById("explorerExportCsv")),
    htmlExists: Boolean(document.getElementById("explorerExportHtml")),
    pdfExists: Boolean(document.getElementById("explorerExportPdf"))
  });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", installExplorerExportClickOwner, { once: true });
} else {
  installExplorerExportClickOwner();
}

export function openBlockExplorer(section) {
  const address = selectedAddress(section);
  traceExport("open-block-explorer", {
    hasAddress: isKaspaAddress(address)
  });
  if (!isKaspaAddress(address)) return;
  window.open("https://explorer.kaspa.org/addresses/" + encodeURIComponent(address), "_blank");
}

export function exportCsv(section = document) {
  const button = qs("#explorerExportCsv", section) || document.getElementById("explorerExportCsv");
  runExplorerBackendExport(section, "csv", button).catch(async (error) => {
    const message = error?.message || String(error);
    setExportStatus(section, message, "error");
    await traceExport("error", { format: "csv", error: message, stack: error?.stack || "" });
  });
}

export function exportHtml(section = document) {
  const button = qs("#explorerExportHtml", section) || document.getElementById("explorerExportHtml");
  runExplorerBackendExport(section, "html", button).catch(async (error) => {
    const message = error?.message || String(error);
    setExportStatus(section, message, "error");
    await traceExport("error", { format: "html", error: message, stack: error?.stack || "" });
  });
}

export function exportPdf(section = document) {
  const button = qs("#explorerExportPdf", section) || document.getElementById("explorerExportPdf");
  runExplorerBackendExport(section, "pdf", button).catch(async (error) => {
    const message = error?.message || String(error);
    setExportStatus(section, message, "error");
    await traceExport("error", { format: "pdf", error: message, stack: error?.stack || "" });
  });
}