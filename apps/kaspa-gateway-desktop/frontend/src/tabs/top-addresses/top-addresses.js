const TOP_ADDRESSES_STATE = {
  installed: false,
  rows: [],
  filteredRows: [],
  prices: {},
  sortColumn: "rank",
  sortDirection: "asc",
  running: false,
  loadedOnce: false,
  lastUpdatedText: "--"
};

function invoke() {
  return window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
}

function root() {
  return (
    document.querySelector("#top-addresses") ||
    document.querySelector("[data-tab='top-addresses']") ||
    document.querySelector("[data-tab-id='top-addresses']") ||
    document.querySelector(".top-addresses-tab")
  );
}

function q(selector) {
  const r = root();
  return r ? r.querySelector(selector) : null;
}

function qa(selector) {
  const r = root();
  return r ? Array.from(r.querySelectorAll(selector)) : [];
}

function buttonByText(pattern) {
  return qa("button").find((button) => {
    const text = [
      button.id,
      button.name,
      button.textContent,
      button.title,
      button.getAttribute("aria-label"),
      button.dataset?.action
    ].join(" ");

    return pattern.test(text);
  });
}

function findRefreshButton() {
  return (
    q("#refreshTopAddresses") ||
    q("#topAddressesRefresh") ||
    buttonByText(/refresh|reload|fetch|update|تحديث|جلب/i)
  );
}

function findFilterButton() {
  return q("#topAddressesFilter") || q("#filterTopAddresses") || buttonByText(/\bfilter\b|فلتر|تصفية/i);
}

function findResetButton() {
  return q("#topAddressesReset") || q("#resetTopAddresses") || buttonByText(/reset filter|reset|إعادة|اعادة|مسح/i);
}

function findCsvButton() {
  return q("#topAddressesExportCsv") || q("#saveTopAddressesCsv") || buttonByText(/save as csv|csv/i);
}

function findHtmlButton() {
  return q("#topAddressesExportHtml") || q("#saveTopAddressesHtml") || buttonByText(/save as html|html/i);
}

function findPdfButton() {
  return q("#topAddressesExportPdf") || q("#saveTopAddressesPdf") || buttonByText(/save as pdf|pdf/i);
}

function findSearchInput() {
  return (
    q("#topAddressesSearch") ||
    q("input[type='search']") ||
    qa("input").find((node) => {
      const meta = [
        node.id,
        node.name,
        node.placeholder,
        node.getAttribute("aria-label")
      ].join(" ").toLowerCase();

      return meta.includes("search by rank") ||
        meta.includes("search") ||
        meta.includes("address") ||
        meta.includes("name");
    })
  );
}

function tableParts() {
  const table = q("table") || q("[data-top-addresses-table]");
  if (!table) return null;

  let thead = table.querySelector("thead");
  let tbody = table.querySelector("tbody");

  if (!thead) {
    thead = document.createElement("thead");
    table.prepend(thead);
  }

  if (!tbody) {
    tbody = document.createElement("tbody");
    table.appendChild(tbody);
  }

  return { table, thead, tbody };
}

function setStatus(message) {
  const node =
    q("#topAddressesStatus") ||
    q("[data-top-addresses-status]") ||
    q(".top-addresses-status");

  if (node) node.textContent = message;
  console.log("[KGW Top Addresses]", message);
}

function setLastUpdated(text) {
  TOP_ADDRESSES_STATE.lastUpdatedText = text || "--";

  const explicit =
    q("#topAddressesLastUpdated") ||
    q("#topAddressesLastUpdatedValue") ||
    q("[data-top-addresses-last-updated]");

  if (explicit) {
    explicit.textContent = TOP_ADDRESSES_STATE.lastUpdatedText;
    return;
  }

  const leaf = qa("*").find((el) => {
    const content = String(el.textContent || "").trim();
    return content.startsWith("Last Updated:") && el.children.length === 0;
  });

  if (leaf) {
    leaf.textContent = `Last Updated: ${TOP_ADDRESSES_STATE.lastUpdatedText}`;
  }
}

function formatDateTime(date = new Date()) {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  const hh = String(date.getHours()).padStart(2, "0");
  const mm = String(date.getMinutes()).padStart(2, "0");
  const ss = String(date.getSeconds()).padStart(2, "0");
  return `${y}-${m}-${d} ${hh}:${mm}:${ss}`;
}

function formatNumber(value, digits = 2) {
  const num = Number(value);
  if (!Number.isFinite(num)) return "0.00";
  return num.toLocaleString(undefined, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits
  });
}

function usdPrice() {
  const num = Number(TOP_ADDRESSES_STATE.prices?.usd || 0);
  return Number.isFinite(num) ? num : 0;
}

function normalizeRow(row, index) {
  const rank = Number(row.rank ?? row.Rank ?? index + 1) || (index + 1);
  const knownName = row.known_name || row.KnownName || row["Known Name"] || "";
  const address = row.address || row.Address || "";
  const balance = Number(row.balance ?? row.amount ?? row.Balance ?? 0) || 0;

  let valueUsd = Number(row.total_usd ?? row.value_usd ?? row.ValueUsd ?? 0);

  if (!Number.isFinite(valueUsd) || valueUsd <= 0) {
    const price = Number(row.kas_price_usd ?? usdPrice()) || 0;
    valueUsd = balance * price;
  }

  return { rank, known_name: knownName, address, balance, value_usd: valueUsd };
}

function currentSearchText() {
  return String(findSearchInput()?.value || "").trim().toLowerCase();
}

function applyFilter() {
  const search = currentSearchText();

  let rows = TOP_ADDRESSES_STATE.rows.map(normalizeRow);

  if (search) {
    rows = rows.filter((row) =>
      [row.rank, row.known_name, row.address, row.balance, row.value_usd]
        .join(" ")
        .toLowerCase()
        .includes(search)
    );
  }

  const column = TOP_ADDRESSES_STATE.sortColumn;
  const direction = TOP_ADDRESSES_STATE.sortDirection === "desc" ? -1 : 1;

  rows.sort((a, b) => {
    const av = a[column];
    const bv = b[column];

    if (typeof av === "number" && typeof bv === "number") {
      return (av - bv) * direction;
    }

    return String(av || "").localeCompare(String(bv || "")) * direction;
  });

  TOP_ADDRESSES_STATE.filteredRows = rows;
  renderTable();
}

function toggleSort(column) {
  if (TOP_ADDRESSES_STATE.sortColumn === column) {
    TOP_ADDRESSES_STATE.sortDirection =
      TOP_ADDRESSES_STATE.sortDirection === "asc" ? "desc" : "asc";
  } else {
    TOP_ADDRESSES_STATE.sortColumn = column;
    TOP_ADDRESSES_STATE.sortDirection = column === "rank" ? "asc" : "desc";
  }

  applyFilter();
}

function renderHeader(thead) {
  thead.innerHTML = "";

  const tr = document.createElement("tr");

  const columns = [
    ["Rank", "rank"],
    ["Known Name", "known_name"],
    ["Address", "address"],
    ["Balance (KAS)", "balance"],
    ["Value (USD)", "value_usd"]
  ];

  for (const [label, key] of columns) {
    const th = document.createElement("th");
    th.textContent = `${label} ↕`;
    th.style.cursor = "pointer";
    th.addEventListener("click", () => toggleSort(key));
    tr.appendChild(th);
  }

  thead.appendChild(tr);
}

function renderTable() {
  const parts = tableParts();
  if (!parts) return;

  const { thead, tbody } = parts;

  renderHeader(thead);
  tbody.innerHTML = "";

  const rows = TOP_ADDRESSES_STATE.filteredRows || [];

  if (!rows.length) {
    const tr = document.createElement("tr");
    const td = document.createElement("td");
    td.colSpan = 5;
    td.textContent = (window.kgwT ? window.kgwT("topAddresses.noTopAddressesLoaded") : "No top addresses loaded.");
    tr.appendChild(td);
    tbody.appendChild(tr);
    return;
  }

  for (const row of rows) {
    const tr = document.createElement("tr");

    const cells = [
      row.rank,
      row.known_name,
      row.address,
      formatNumber(row.balance, 2),
      `${formatNumber(row.value_usd, 2)} USD`
    ];

    for (const value of cells) {
      const td = document.createElement("td");
      td.textContent = String(value ?? "");
      tr.appendChild(td);
    }

    tbody.appendChild(tr);
  }
}

function escapeCsv(value) {
  return `"${String(value ?? "").replace(/"/g, '""')}"`;
}

function escapeHtml(text) {
  return String(text || "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function kgwTopLocaleV1G() {
  return document.documentElement.getAttribute("lang") ||
    window.kgwCurrentLocale ||
    window.localStorage?.getItem?.("kgw.locale") ||
    "en";
}

// KGW_EXPORT_TEMPLATE_PARITY_TOP_URLS_V1B
function kgwTopQueryV1B(selector) {
  return document.querySelector(selector);
}

function kgwTopAddressUrlV1B(address) {
  const clean = String(address || "").trim().replace(/[\\s'"<>]+$/g, "");
  return clean.startsWith("kaspa:") ? `https://explorer.kaspa.org/addresses/${clean}` : "";
}

function kgwTopSelectedCurrencyV1B() {
  const candidates = [
    kgwTopQueryV1B("#topAddressesCurrency")?.value,
    kgwTopQueryV1B("[data-top-addresses-currency]")?.value,
    kgwTopQueryV1B("[name='topAddressesCurrency']")?.value,
    window.kgwSelectedCurrency,
    window.KGW_SELECTED_CURRENCY,
    localStorage.getItem("kgw.currency"),
    localStorage.getItem("kgw.selectedCurrency")
  ];

  for (const value of candidates) {
    const text = String(value || "").trim().toUpperCase();
    if (/^[A-Z]{3}$/.test(text)) return text;
  }

  return "USD";
}

function kgwTopValueForCurrencyV1B(row, currency) {
  const lower = String(currency || "USD").toLowerCase();
  const upper = String(currency || "USD").toUpperCase();

  const candidates = [
    row[`value_${lower}`],
    row[`value_${upper}`],
    row[`value${upper}`],
    row.value,
    row.value_usd,
    row.valueUSD
  ];

  for (const value of candidates) {
    if (value !== undefined && value !== null && String(value).trim() !== "") return String(value);
  }

  return "";
}

function kgwTopClientTableV1G() {
  const rows = TOP_ADDRESSES_STATE.filteredRows || [];
  const currency = kgwTopSelectedCurrencyV1B();

  if (!rows.length) throw new Error("No top address rows are available for export.");

  return {
    title: "Kaspa Gateway Top Addresses",
    subtitle: `Last Updated: ${TOP_ADDRESSES_STATE.lastUpdatedText || "--"} | Currency: ${currency}`,
    headers: ["Rank", "Known Name", "Address", "Address URL", "Balance (KAS)", `Value (${currency})`],
    rows: rows.map((row) => [
      String(row.rank ?? ""),
      String(row.known_name ?? row.knownName ?? ""),
      String(row.address ?? ""),
      kgwTopAddressUrlV1B(row.address ?? ""),
      String(row.balance ?? ""),
      kgwTopValueForCurrencyV1B(row, currency)
    ])
  };
}


/* KGW_EXPORT_NATIVE_SAVE_PHASE_A_V8_START */


function kgwTopDialogApiV9() {
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


async function kgwTopLoadNativeDialogV8() {
  return kgwTopDialogApiV9();
}

function kgwTopNativeDialogFilterV8(format) {
  const ext = String(format || "").replace(/^\./, "").toLowerCase();
  return {
    name: ext ? `${ext.toUpperCase()} files` : "Export files",
    extensions: ext ? [ext] : []
  };
}

async function kgwTopNativeSavePathV8(format, defaultPath) {
  const dialog = await kgwTopLoadNativeDialogV8();
  const selected = await dialog.save({
    title: "Save export",
    defaultPath,
    filters: [kgwTopNativeDialogFilterV8(format)]
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


async function kgwTopNativeAskOpenV8(finalPath) {
  const accepted = await kgwExportCenteredOpenPromptV10();
  if (!accepted) return false;

  const call = invoke();
  if (!call) throw new Error("Tauri invoke API is not available.");

  await call("kgw_open_exported_file_v1", { path: finalPath });
  return true;
}

/* KGW_EXPORT_NATIVE_SAVE_PHASE_A_V8_END */


async function kgwTopExportBackendV1G(format) {
  const call = invoke();
  if (!call) throw new Error("Tauri invoke API is not available.");

  const outputPath = await call("export_default_path", {
    reportType: "TopAddresses",
    format
  });

  const selectedOutputPath = await kgwTopNativeSavePathV8(format, outputPath);
  if (!selectedOutputPath) {
    setStatus("Export cancelled.");
    return;
  }

  const result = await call("export_report", {
    request: {
      reportType: "TopAddresses",
      format,
      outputPath: selectedOutputPath,
      addressFilter: null,
      timeRange: "all",
      limit: 100000,
      locale: kgwTopLocaleV1G(),
      clientTable: kgwTopClientTableV1G()
    }
  });

  const finalPath = result.output_path || result.outputPath || selectedOutputPath;
  setStatus(`Export completed: ${finalPath}`);
  await kgwTopNativeAskOpenV8(finalPath);
}

function exportCsv() {
  kgwTopExportBackendV1G("csv").catch((error) => setStatus(error?.message || String(error)));
}

function exportHtml() {
  kgwTopExportBackendV1G("html").catch((error) => setStatus(error?.message || String(error)));
}

function exportPdf() {
  kgwTopExportBackendV1G("pdf").catch((error) => setStatus(error?.message || String(error)));
}


async function refreshTopAddresses() {
  if (TOP_ADDRESSES_STATE.running) {
    setStatus("Top addresses fetch is already running...");
    return;
  }

  const call = invoke();
  if (!call) {
    setStatus("Tauri invoke API is not available.");
    return;
  }

  TOP_ADDRESSES_STATE.running = true;
  setStatus("Refreshing top addresses...");

  try {
    const response = await call("fetch_top_addresses_rust", { limit: 10000 });

    TOP_ADDRESSES_STATE.rows = Array.isArray(response?.rows) ? response.rows : [];
    TOP_ADDRESSES_STATE.prices = response?.prices || {};
    TOP_ADDRESSES_STATE.loadedOnce = true;

    setLastUpdated(formatDateTime(new Date()));
    applyFilter();

    setStatus(`Loaded ${TOP_ADDRESSES_STATE.rows.length} top addresses.`);
  } catch (error) {
    console.error(error);
    setStatus(error?.message || String(error));
  } finally {
    TOP_ADDRESSES_STATE.running = false;
  }
}


// KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49D
function kgwTopAddressesUiTraceR49D(action, phase, details) {
  try {
    const safeAction = String(action || "top-addresses-ui");
    const safePhase = String(phase || "unknown");
    const safeDetails = details && typeof details === "object" ? details : {};
    const args = {
      scope: "top-addresses",
      net: "ui",
      action: safeAction,
      phase: safePhase,
      details: JSON.stringify({
        patch: "KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49D",
        owner: "top-addresses-installButtonHandlers-safe-owner",
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
    if (typeof invoke === "function") {
      invoke("kgw_frontend_button_trace_v1", args).catch(function () {});
    }
  } catch (_) {}
}
function installButtonHandlers() {
  const bindings = [
    [findRefreshButton(), "refresh", refreshTopAddresses],
    [findFilterButton(), "filter", applyFilter],
    [findResetButton(), "reset", () => {
      const search = findSearchInput();
      if (search) search.value = "";
      applyFilter();
    }],
    [findCsvButton(), "csv", exportCsv],
    [findHtmlButton(), "html", exportHtml],
    [findPdfButton(), "pdf", exportPdf]
  ];

  for (const [button, key, handler] of bindings) {
    if (button && button.dataset.kgwTopHandler !== key) {
      button.dataset.kgwTopHandler = key;
      button.addEventListener("click", (event) => {
      kgwTopAddressesUiTraceR49D("top-addresses-click", "r49d-top-addresses-click", {
        trusted: Boolean(event && event.isTrusted),
        element: "button",
        id: String(button.id || ""),
        text: String(button.textContent || "").trim(),
        dataset: JSON.stringify(button.dataset || {})
      });
        event.preventDefault();
        handler();
      });
    }
  }

  const search = findSearchInput();
  if (search && !search.dataset.kgwTopSearch) {
    search.dataset.kgwTopSearch = "1";
    search.addEventListener("input", applyFilter);
    search.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        event.preventDefault();
        applyFilter();
      }
    });
  }
}

export function initTopAddressesTab() {
  installButtonHandlers();

  if (TOP_ADDRESSES_STATE.loadedOnce) {
    applyFilter();
    return;
  }

  refreshTopAddresses();
}

window.initTopAddressesTab = initTopAddressesTab;
window.refreshTopAddresses = refreshTopAddresses;
