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

function downloadText(filename, content, type = "text/plain;charset=utf-8") {
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function exportCsv() {
  const rows = TOP_ADDRESSES_STATE.filteredRows || [];
  const lines = [
    ["Rank", "Known Name", "Address", "Balance (KAS)", "Value (USD)"].join(",")
  ];

  for (const row of rows) {
    lines.push([
      row.rank,
      escapeCsv(row.known_name),
      escapeCsv(row.address),
      row.balance,
      row.value_usd
    ].join(","));
  }

  downloadText("top-addresses.csv", lines.join("\r\n"), "text/csv;charset=utf-8");
}

function exportHtml() {
  const rows = TOP_ADDRESSES_STATE.filteredRows || [];
  const bodyRows = rows.map((row) => `
    <tr>
      <td>${row.rank}</td>
      <td>${escapeHtml(row.known_name)}</td>
      <td>${escapeHtml(row.address)}</td>
      <td>${formatNumber(row.balance, 2)}</td>
      <td>${formatNumber(row.value_usd, 2)} USD</td>
    </tr>
  `).join("");

  const html = `<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>Top Addresses</title>
<style>
body { font-family: Arial, sans-serif; padding: 20px; }
table { width: 100%; border-collapse: collapse; }
th, td { border: 1px solid #999; padding: 8px; text-align: left; }
th { background: #e6eef8; }
</style>
</head>
<body>
<h2>Top Addresses</h2>
<p>Last Updated: ${escapeHtml(TOP_ADDRESSES_STATE.lastUpdatedText)}</p>
<table>
<thead>
<tr>
<th>Rank</th>
<th>Known Name</th>
<th>Address</th>
<th>Balance (KAS)</th>
<th>Value (USD)</th>
</tr>
</thead>
<tbody>
${bodyRows}
</tbody>
</table>
</body>
</html>`;

  downloadText("top-addresses.html", html, "text/html;charset=utf-8");
}

function exportPdf() {
  window.print();
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
