/*
 * KGW_EXPLORER_EXPORT_EXTERNAL_LINKS
 *
 * Behavior-preserving extraction from explorer.js.
 * This module owns export/download helpers and external block explorer opening only.
 * It must not own transaction fetch, Tauri IPC, DB orchestration, or core rendering.
 */

import { formatKas, formatUsd } from "./explorer.formatting.js";
export function openBlockExplorer(section) {
  const address = normalizeAddress(qs("#explorerAddress", section)?.value);
  if (!isKaspaAddress(address)) return;

  window.open(`https://explorer.kaspa.org/addresses/${encodeURIComponent(address)}`, "_blank");
}

export function csvEscape(value) {
  return `"${String(value ?? "").replaceAll('"', '""')}"`;
}

export function htmlEscape(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

export function downloadText(filename, content, type) {
  const blob = new Blob([content], { type: type || "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");

  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();

  URL.revokeObjectURL(url);
}

export function exportCsv() {
  const rows = explorerState.filteredRows;
  const lines = [
    ["Date/Time", "Transaction ID", "Direction", "Amount (KAS)", "Value (USD)", "Type"].map(csvEscape).join(",")
  ];

  for (const row of rows) {
    lines.push([
      row.datetime,
      row.txid,
      row.direction,
      row.amount,
      row.value,
      row.type
    ].map(csvEscape).join(","));
  }

  downloadText("explorer-transactions.csv", lines.join("\r\n"), "text/csv;charset=utf-8");
}

export function exportHtml() {
  const rows = explorerState.filteredRows;

  const body = rows.map((row) => `
    <tr>
      <td>${htmlEscape(row.datetime)}</td>
      <td>${htmlEscape(row.txid)}</td>
      <td>${htmlEscape(row.direction)}</td>
      <td>${htmlEscape(formatKas(row.amount))}</td>
      <td>${htmlEscape(formatUsd(row.value))}</td>
      <td>${htmlEscape(row.type)}</td>
    </tr>
  `).join("");

  downloadText("explorer-transactions.html", `<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>Kaspa Gateway Explorer Transactions</title>
<style>
body { font-family: Arial, sans-serif; padding: 20px; }
table { width: 100%; border-collapse: collapse; }
th, td { border: 1px solid #999; padding: 6px; text-align: left; }
th { background: #e6eef8; }
</style>
</head>
<body>
<h2>Kaspa Gateway Explorer Transactions</h2>
<table>
<thead>
<tr>
<th>Date/Time</th>
<th>Transaction ID</th>
<th>Direction</th>
<th>Amount (KAS)</th>
<th>Value (USD)</th>
<th>Type</th>
</tr>
</thead>
<tbody>
${body}
</tbody>
</table>
</body>
</html>`, "text/html;charset=utf-8");
}
