use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths, TransactionRecord};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealReportFormat {
    Csv,
    Html,
    Pdf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealReportTarget {
    Addresses,
    Transactions,
    FullSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealReportRequest {
    pub target: RealReportTarget,
    pub format: RealReportFormat,
    pub output_path: String,
    pub address_filter: Option<String>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealReportResult {
    pub output_path: String,
    pub rows_exported: usize,
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ReportTable {
    title: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[tauri::command]
pub fn default_real_report_path(filename: String) -> Result<String, String> {
    validate_filename(&filename)?;

    let root = default_user_data_dir().map_err(|error| error.to_string())?;
    let path = root.join("exports").join(filename);

    Ok(path.display().to_string())
}

#[tauri::command]
pub fn export_real_report(request: RealReportRequest) -> Result<RealReportResult, String> {
    validate_output_path(&request.output_path)?;

    let limit = request.limit.clamp(1, 50_000);

    let table = match request.target {
        RealReportTarget::Addresses => addresses_report(limit)?,
        RealReportTarget::Transactions => {
            transactions_report(request.address_filter.clone(), limit)?
        }
        RealReportTarget::FullSummary => full_summary_report(limit)?,
    };

    let output_path = ensure_extension(PathBuf::from(&request.output_path), &request.format);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    match request.format {
        RealReportFormat::Csv => write_csv(&output_path, &table)?,
        RealReportFormat::Html => write_html(&output_path, &table)?,
        RealReportFormat::Pdf => write_pdf(&output_path, &table)?,
    }

    Ok(RealReportResult {
        output_path: output_path.display().to_string(),
        rows_exported: table.rows.len(),
        title: table.title,
        message: "Real report exported successfully.".to_string(),
    })
}

#[tauri::command]
pub fn real_report_preview(
    target: RealReportTarget,
    address_filter: Option<String>,
    limit: usize,
) -> Result<Vec<Vec<String>>, String> {
    let limit = limit.clamp(1, 200);

    let table = match target {
        RealReportTarget::Addresses => addresses_report(limit)?,
        RealReportTarget::Transactions => transactions_report(address_filter, limit)?,
        RealReportTarget::FullSummary => full_summary_report(limit)?,
    };

    let mut rows = Vec::new();
    rows.push(table.headers);
    rows.extend(table.rows);

    Ok(rows)
}

fn addresses_report(limit: usize) -> Result<ReportTable, String> {
    let manager = database_manager()?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let mut records = repository.list().map_err(|error| error.to_string())?;
    records.truncate(limit);

    let rows = records
        .into_iter()
        .map(|record| {
            vec![
                record.name,
                record.address,
                record.network,
                record.created_at_ms.to_string(),
                record.updated_at_ms.to_string(),
            ]
        })
        .collect::<Vec<_>>();

    Ok(ReportTable {
        title: "Saved Addresses Report".to_string(),
        headers: vec![
            "Name".to_string(),
            "Address".to_string(),
            "Network".to_string(),
            "Created At (ms)".to_string(),
            "Updated At (ms)".to_string(),
        ],
        rows,
    })
}

fn transactions_report(
    address_filter: Option<String>,
    limit: usize,
) -> Result<ReportTable, String> {
    let manager = database_manager()?;
    let tx_repository = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let mut transactions = Vec::new();

    if let Some(address) = address_filter
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let parsed = KaspaAddress::parse(address).map_err(|error| error.to_string())?;
        transactions.extend(
            tx_repository
                .list_for_address(parsed.as_str(), limit)
                .map_err(|error| error.to_string())?,
        );
    } else {
        let address_repository = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        let addresses = address_repository
            .list()
            .map_err(|error| error.to_string())?;

        for address in addresses {
            let remaining = limit.saturating_sub(transactions.len());

            if remaining == 0 {
                break;
            }

            let mut rows = tx_repository
                .list_for_address(&address.address, remaining)
                .map_err(|error| error.to_string())?;

            transactions.append(&mut rows);
        }
    }

    transactions.sort_by_key(|right| std::cmp::Reverse(right.timestamp_ms));
    transactions.truncate(limit);

    let rows = transactions
        .into_iter()
        .map(transaction_row)
        .collect::<Vec<_>>();

    Ok(ReportTable {
        title: "Transactions Report".to_string(),
        headers: vec![
            "TxID".to_string(),
            "Address".to_string(),
            "Timestamp (ms)".to_string(),
            "Type".to_string(),
            "Direction".to_string(),
            "Amount Sompi".to_string(),
            "Amount KAS".to_string(),
            "Counterparty".to_string(),
            "Block Height".to_string(),
        ],
        rows,
    })
}

fn full_summary_report(limit: usize) -> Result<ReportTable, String> {
    let manager = database_manager()?;

    let address_repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let tx_repository = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let addresses = address_repository
        .list()
        .map_err(|error| error.to_string())?;

    let mut rows = Vec::new();

    let saved_addresses = addresses.len();
    let total_transactions = tx_repository
        .total_count()
        .map_err(|error| error.to_string())?;

    rows.push(vec![
        "Saved Addresses".to_string(),
        saved_addresses.to_string(),
        "Total saved addresses in the local database.".to_string(),
    ]);

    rows.push(vec![
        "Cached Transactions".to_string(),
        total_transactions.to_string(),
        "Total cached transactions in the local database.".to_string(),
    ]);

    for address in addresses.into_iter().take(limit.saturating_sub(rows.len())) {
        let count = tx_repository
            .count_for_address(&address.address)
            .map_err(|error| error.to_string())?;

        rows.push(vec![
            format!("Address: {}", address.name),
            count.to_string(),
            address.address,
        ]);
    }

    Ok(ReportTable {
        title: "Full Summary Report".to_string(),
        headers: vec![
            "Metric".to_string(),
            "Value".to_string(),
            "Details".to_string(),
        ],
        rows,
    })
}

fn transaction_row(record: TransactionRecord) -> Vec<String> {
    let amount_kas = record.amount_kas();

    vec![
        record.txid,
        record.address,
        record.timestamp_ms.to_string(),
        record.tx_type,
        record.direction,
        record.amount_sompi.to_string(),
        format!("{amount_kas:.8}"),
        record.counterparty.unwrap_or_default(),
        record
            .block_height
            .map(|value| value.to_string())
            .unwrap_or_default(),
    ]
}

fn write_csv(path: &Path, table: &ReportTable) -> Result<(), String> {
    let mut output = String::new();

    output.push_str(
        &table
            .headers
            .iter()
            .map(|header| csv_cell(header))
            .collect::<Vec<_>>()
            .join(","),
    );
    output.push('\n');

    for row in &table.rows {
        output.push_str(
            &row.iter()
                .map(|cell| csv_cell(cell))
                .collect::<Vec<_>>()
                .join(","),
        );
        output.push('\n');
    }

    fs::write(path, output).map_err(|error| error.to_string())
}

fn write_html(path: &Path, table: &ReportTable) -> Result<(), String> {
    let mut output = String::new();

    output.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    output.push_str("<title>");
    output.push_str(&html_escape(&table.title));
    output.push_str("</title>");
    output.push_str(
        r#"<style>
body{font-family:Arial,sans-serif;margin:24px}
table{border-collapse:collapse;width:100%}
th,td{border:1px solid #ccc;padding:6px;text-align:left;font-size:13px}
th{background:#f4f4f4}
caption{font-size:20px;font-weight:bold;margin-bottom:12px}
</style>"#,
    );
    output.push_str("</head><body><table><caption>");
    output.push_str(&html_escape(&table.title));
    output.push_str("</caption><thead><tr>");

    for header in &table.headers {
        output.push_str("<th>");
        output.push_str(&html_escape(header));
        output.push_str("</th>");
    }

    output.push_str("</tr></thead><tbody>");

    for row in &table.rows {
        output.push_str("<tr>");

        for cell in row {
            output.push_str("<td>");
            output.push_str(&html_escape(cell));
            output.push_str("</td>");
        }

        output.push_str("</tr>");
    }

    output.push_str("</tbody></table></body></html>");

    fs::write(path, output).map_err(|error| error.to_string())
}

fn write_pdf(path: &Path, table: &ReportTable) -> Result<(), String> {
    let text_lines = pdf_text_lines(table);
    let content_stream = build_pdf_content_stream(&text_lines);

    let pdf = build_minimal_pdf(&content_stream);
    fs::write(path, pdf).map_err(|error| error.to_string())
}

fn pdf_text_lines(table: &ReportTable) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(table.title.clone());
    lines.push(table.headers.join(" | "));

    for row in table.rows.iter().take(80) {
        lines.push(row.join(" | "));
    }

    if table.rows.len() > 80 {
        lines.push(format!(
            "... truncated preview: {} total rows exported in source table",
            table.rows.len()
        ));
    }

    lines
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

fn ensure_extension(mut path: PathBuf, format: &RealReportFormat) -> PathBuf {
    let extension = match format {
        RealReportFormat::Csv => "csv",
        RealReportFormat::Html => "html",
        RealReportFormat::Pdf => "pdf",
    };

    let needs_extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| !value.eq_ignore_ascii_case(extension))
        .unwrap_or(true);

    if needs_extension {
        path.set_extension(extension);
    }

    path
}

fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.trim().is_empty()
        || filename.contains('\0')
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains('"')
        || filename.contains('\n')
        || filename.contains('\r')
        || filename.contains("..")
        || filename.contains("&&")
        || filename.contains("||")
        || filename.contains('|')
        || filename.contains(';')
    {
        return Err("Filename contains unsafe characters.".to_string());
    }

    Ok(())
}

fn validate_output_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty()
        || path.contains('\0')
        || path.contains('"')
        || path.contains('\n')
        || path.contains('\r')
        || path.contains("&&")
        || path.contains("||")
        || path.contains('|')
        || path.contains(';')
    {
        return Err("Output path contains unsafe characters.".to_string());
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_cell_blocks_formula_injection() {
        assert_eq!(csv_cell("=cmd"), "\"'=cmd\"");
        assert_eq!(csv_cell("normal"), "\"normal\"");
    }

    #[test]
    fn filename_rejects_traversal() {
        assert!(validate_filename("../bad.csv").is_err());
        assert!(validate_filename("report.csv").is_ok());
    }

    #[test]
    fn pdf_builder_outputs_pdf_header() {
        let table = ReportTable {
            title: "Test".to_string(),
            headers: vec!["A".to_string()],
            rows: vec![vec!["B".to_string()]],
        };

        let lines = pdf_text_lines(&table);
        let stream = build_pdf_content_stream(&lines);
        let pdf = build_minimal_pdf(&stream);

        assert!(pdf.starts_with(b"%PDF-1.4"));
    }
}
