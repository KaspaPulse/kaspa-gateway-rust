use kaspa_gateway_api::{ApiClientConfig, KaspaApiClient};
use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerBalanceRequest {
    pub address: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplorerBalanceResponse {
    pub address: String,
    pub balance_sompi: u64,
    pub balance_kas: String,
    pub api_url: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerExportRequest {
    pub address: String,
    pub format: String,
    pub rows: Vec<ExplorerTransactionRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerTransactionRow {
    pub timestamp_ms: i64,
    pub time_text: String,
    pub direction: String,
    pub tx_type: String,
    pub amount_sompi: i64,
    pub amount_kas: String,
    pub counterparty: String,
    pub txid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplorerSavedAddress {
    pub name: String,
    pub address: String,
    pub network: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplorerExportResponse {
    pub output_path: String,
    pub rows_exported: usize,
    pub bytes_written: u64,
    pub message: String,
}

#[tauri::command]
pub fn explorer_saved_addresses() -> Result<Vec<ExplorerSavedAddress>, String> {
    let manager = database_manager()?;
    let repo = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let rows = repo.list().map_err(|error| error.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| ExplorerSavedAddress {
            name: row.name,
            address: row.address,
            network: row.network,
        })
        .collect())
}

#[tauri::command]
pub async fn explorer_balance(
    request: ExplorerBalanceRequest,
) -> Result<ExplorerBalanceResponse, String> {
    let parsed = KaspaAddress::parse(&request.address).map_err(|error| error.to_string())?;
    let client = api_client(request.base_url.as_deref())?;

    let api_url = client
        .address_balance_url(parsed.as_str())
        .map_err(|error| error.to_string())?
        .to_string();

    let balance_sompi = client
        .fetch_address_balance_sompi(parsed.as_str())
        .await
        .map_err(|error| error.to_string())?;

    Ok(ExplorerBalanceResponse {
        address: parsed.as_str().to_string(),
        balance_sompi,
        balance_kas: sompi_to_kas(balance_sompi as i64),
        api_url,
        message: "Balance fetched from live Kaspa API.".to_string(),
    })
}

#[tauri::command]
pub fn explorer_export(request: ExplorerExportRequest) -> Result<ExplorerExportResponse, String> {
    let parsed = KaspaAddress::parse(&request.address).map_err(|error| error.to_string())?;
    let format = normalize_export_format(&request.format)?;

    let export_dir = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("exports");

    fs::create_dir_all(&export_dir).map_err(|error| error.to_string())?;

    let safe_address = safe_file_component(parsed.as_str());
    let output_path = export_dir.join(format!("explorer_{safe_address}.{format}"));

    match format.as_str() {
        "csv" => write_csv(&output_path, &request.rows)?,
        "html" => write_html(&output_path, parsed.as_str(), &request.rows)?,
        "pdf" => write_pdf_like(&output_path, parsed.as_str(), &request.rows)?,
        _ => unreachable!(),
    }

    let bytes_written = fs::metadata(&output_path)
        .map_err(|error| error.to_string())?
        .len();

    Ok(ExplorerExportResponse {
        output_path: output_path.display().to_string(),
        rows_exported: request.rows.len(),
        bytes_written,
        message: "Explorer export completed.".to_string(),
    })
}

fn api_client(base_url: Option<&str>) -> Result<KaspaApiClient, String> {
    let config = if let Some(base_url) = base_url.map(str::trim).filter(|value| !value.is_empty()) {
        ApiClientConfig::new(base_url).map_err(|error| error.to_string())?
    } else {
        ApiClientConfig::default()
    };

    KaspaApiClient::new(config).map_err(|error| error.to_string())
}

fn sompi_to_kas(value: i64) -> String {
    format!("{:.8}", value as f64 / 100_000_000.0)
}

fn normalize_export_format(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "csv" => Ok("csv".to_string()),
        "html" | "htm" => Ok("html".to_string()),
        "pdf" => Ok("pdf".to_string()),
        _ => Err("Export format must be csv, html, or pdf.".to_string()),
    }
}

fn write_csv(path: &Path, rows: &[ExplorerTransactionRow]) -> Result<(), String> {
    let mut output = String::new();

    output.push_str(
        "timestamp_ms,time_text,direction,tx_type,amount_sompi,amount_kas,counterparty,txid\n",
    );

    for row in rows {
        output.push_str(
            &[
                csv_cell(&row.timestamp_ms.to_string()),
                csv_cell(&row.time_text),
                csv_cell(&row.direction),
                csv_cell(&row.tx_type),
                csv_cell(&row.amount_sompi.to_string()),
                csv_cell(&row.amount_kas),
                csv_cell(&row.counterparty),
                csv_cell(&row.txid),
            ]
            .join(","),
        );
        output.push('\n');
    }

    fs::write(path, output).map_err(|error| error.to_string())
}

fn write_html(path: &Path, address: &str, rows: &[ExplorerTransactionRow]) -> Result<(), String> {
    let mut output = String::new();

    output.push_str(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Explorer Export</title>",
    );
    output.push_str("<style>body{font-family:Arial,sans-serif;margin:24px}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:6px;font-size:13px;text-align:left}th{background:#f4f4f4}</style>");
    output.push_str("</head><body>");
    output.push_str("<h1>Explorer Export</h1>");
    output.push_str("<p>");
    output.push_str(&html_escape(address));
    output.push_str("</p>");
    output.push_str("<table><thead><tr><th>Time</th><th>Direction</th><th>Type</th><th>Amount KAS</th><th>Counterparty</th><th>TxID</th></tr></thead><tbody>");

    for row in rows {
        output.push_str("<tr>");
        output.push_str(&format!(
            "<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
            html_escape(&row.time_text),
            html_escape(&row.direction),
            html_escape(&row.tx_type),
            html_escape(&row.amount_kas),
            html_escape(&row.counterparty),
            html_escape(&row.txid)
        ));
        output.push_str("</tr>");
    }

    output.push_str("</tbody></table></body></html>");

    fs::write(path, output).map_err(|error| error.to_string())
}

fn write_pdf_like(
    path: &Path,
    address: &str,
    rows: &[ExplorerTransactionRow],
) -> Result<(), String> {
    let mut lines = Vec::new();

    lines.push("Explorer Export".to_string());
    lines.push(address.to_string());
    lines.push("Time | Direction | Type | Amount KAS | Counterparty | TxID".to_string());

    for row in rows.iter().take(80) {
        lines.push(format!(
            "{} | {} | {} | {} | {} | {}",
            row.time_text, row.direction, row.tx_type, row.amount_kas, row.counterparty, row.txid
        ));
    }

    if rows.len() > 80 {
        lines.push(format!("... truncated: {} total rows", rows.len()));
    }

    let stream = build_pdf_content_stream(&lines);
    let pdf = build_minimal_pdf(&stream);

    fs::write(path, pdf).map_err(|error| error.to_string())
}

fn build_pdf_content_stream(lines: &[String]) -> String {
    let mut stream = String::new();

    stream.push_str("BT\n/F1 10 Tf\n50 780 Td\n14 TL\n");

    for line in lines {
        stream.push('(');
        stream.push_str(&pdf_escape(&safe_truncate(line, 120)));
        stream.push_str(") Tj\nT*\n");
    }

    stream.push_str("ET\n");
    stream
}

fn build_minimal_pdf(content_stream: &str) -> Vec<u8> {
    let mut objects = Vec::new();

    objects.push("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string());
    objects.push("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n".to_string());
    objects.push("3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n".to_string());
    objects.push(
        "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n".to_string(),
    );
    objects.push(format!(
        "5 0 obj\n<< /Length {} >>\nstream\n{}endstream\nendobj\n",
        content_stream.len(),
        content_stream
    ));

    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();

    for object in &objects {
        offsets.push(pdf.len());
        pdf.push_str(object);
    }

    let xref_offset = pdf.len();

    pdf.push_str("xref\n0 6\n0000000000 65535 f \n");

    for offset in offsets {
        pdf.push_str(&format!("{offset:010} 00000 n \n"));
    }

    pdf.push_str("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n");
    pdf.push_str(&xref_offset.to_string());
    pdf.push_str("\n%%EOF\n");

    pdf.into_bytes()
}

fn csv_cell(value: &str) -> String {
    let mut value = value.replace('\0', "");
    value = value.replace('\r', " ");
    value = value.replace('\n', " ");

    if value.starts_with('=')
        || value.starts_with('+')
        || value.starts_with('-')
        || value.starts_with('@')
    {
        value.insert(0, '\'');
    }

    format!("\"{}\"", value.replace('"', "\"\""))
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn pdf_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace(['\r', '\n'], " ")
}

fn safe_truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn safe_file_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn database_manager() -> Result<DatabaseManager, String> {
    let root = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("databases");

    let paths = DatabasePaths::new(root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths);

    manager
        .initialize_all()
        .map_err(|error| error.to_string())?;

    Ok(manager)
}
