use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{AddressRecord, DatabaseManager, DatabasePaths, TransactionRecord};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportParityRequest {
    pub report_type: String,
    pub format: String,
    pub output_path: String,
    pub address_filter: Option<String>,
    pub time_range: String,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportParityResult {
    pub report_type: String,
    pub format: String,
    pub output_path: String,
    pub rows_exported: usize,
    pub bytes_written: u64,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ReportTable {
    title: String,
    subtitle: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[tauri::command]
pub fn export_default_path(report_type: String, format: String) -> Result<String, String> {
    let report_type = normalize_report_type(&report_type)?;
    let format = normalize_format(&format)?;

    let root = default_user_data_dir().map_err(|error| error.to_string())?;
    let filename = format!(
        "kaspa_gateway_{}_{}.{}",
        report_type.to_ascii_lowercase(),
        now_file_stamp(),
        format
    );

    Ok(root.join("exports").join(filename).display().to_string())
}

#[tauri::command]
pub fn export_preview(request: ExportParityRequest) -> Result<Vec<Vec<String>>, String> {
    let request = normalize_request(request)?;
    let table = build_report_table(&request)?;

    let mut rows = Vec::new();
    rows.push(table.headers);
    rows.extend(table.rows.into_iter().take(100));

    Ok(rows)
}

#[tauri::command]
pub fn export_report(request: ExportParityRequest) -> Result<ExportParityResult, String> {
    let request = normalize_request(request)?;
    validate_output_path(&request.output_path)?;

    let table = build_report_table(&request)?;
    let output_path = ensure_extension(PathBuf::from(&request.output_path), &request.format);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    match request.format.as_str() {
        "csv" => write_csv(&output_path, &table)?,
        "html" => write_html(&output_path, &table)?,
        "pdf" => write_pdf(&output_path, &table)?,
        _ => return Err("Unsupported export format.".to_string()),
    }

    let bytes_written = fs::metadata(&output_path)
        .map_err(|error| error.to_string())?
        .len();

    Ok(ExportParityResult {
        report_type: request.report_type,
        format: request.format,
        output_path: output_path.display().to_string(),
        rows_exported: table.rows.len(),
        bytes_written,
        message: "Export parity report completed successfully.".to_string(),
    })
}

fn normalize_request(mut request: ExportParityRequest) -> Result<ExportParityRequest, String> {
    request.report_type = normalize_report_type(&request.report_type)?;
    request.format = normalize_format(&request.format)?;
    request.time_range = normalize_time_range(&request.time_range)?;
    request.limit = request.limit.clamp(1, 100_000);

    if let Some(address) = request
        .address_filter
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let parsed = KaspaAddress::parse(address).map_err(|error| error.to_string())?;
        request.address_filter = Some(parsed.as_str().to_string());
    }

    Ok(request)
}

fn build_report_table(request: &ExportParityRequest) -> Result<ReportTable, String> {
    match request.report_type.as_str() {
        "Addresses" => addresses_report(request.limit),
        "Transactions" => transactions_report(
            request.address_filter.clone(),
            &request.time_range,
            request.limit,
        ),
        "Analysis" => analysis_report(
            request.address_filter.clone(),
            &request.time_range,
            request.limit,
        ),
        "Full" => full_report(
            request.address_filter.clone(),
            &request.time_range,
            request.limit,
        ),
        _ => Err("Unsupported report type.".to_string()),
    }
}

fn addresses_report(limit: usize) -> Result<ReportTable, String> {
    let manager = database_manager()?;
    let repo = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let records = repo.list().map_err(|error| error.to_string())?;

    let rows = records
        .into_iter()
        .take(limit)
        .map(address_row)
        .collect::<Vec<_>>();

    Ok(ReportTable {
        title: "Kaspa Gateway Addresses Report".to_string(),
        subtitle: "Address book export compatible with the Python report flow.".to_string(),
        headers: vec![
            "Name".to_string(),
            "Address".to_string(),
            "Network".to_string(),
            "Created At Ms".to_string(),
            "Updated At Ms".to_string(),
        ],
        rows,
    })
}

fn transactions_report(
    address_filter: Option<String>,
    time_range: &str,
    limit: usize,
) -> Result<ReportTable, String> {
    let records = load_transactions(address_filter, time_range, limit)?;
    let rows = records.into_iter().map(transaction_row).collect::<Vec<_>>();

    Ok(ReportTable {
        title: "Kaspa Gateway Transactions Report".to_string(),
        subtitle: format!("Time range: {time_range}"),
        headers: vec![
            "Timestamp Ms".to_string(),
            "Direction".to_string(),
            "Type".to_string(),
            "Amount KAS".to_string(),
            "Amount Sompi".to_string(),
            "Address".to_string(),
            "Counterparty".to_string(),
            "TXID".to_string(),
        ],
        rows,
    })
}

fn analysis_report(
    address_filter: Option<String>,
    time_range: &str,
    limit: usize,
) -> Result<ReportTable, String> {
    let records = load_transactions(address_filter, time_range, limit)?;

    let mut incoming_sompi = 0_i64;
    let mut outgoing_sompi = 0_i64;
    let mut incoming_count = 0_usize;
    let mut outgoing_count = 0_usize;
    let mut largest_inflow = 0_i64;
    let mut largest_outflow = 0_i64;
    let mut counterparty_map: BTreeMap<String, usize> = BTreeMap::new();
    let mut unique_addresses = BTreeSet::new();
    let mut first_tx = 0_i64;
    let mut last_tx = 0_i64;

    for record in &records {
        if first_tx == 0 || record.timestamp_ms < first_tx {
            first_tx = record.timestamp_ms;
        }

        if record.timestamp_ms > last_tx {
            last_tx = record.timestamp_ms;
        }

        unique_addresses.insert(record.address.clone());

        let amount = record.amount_sompi.max(0);
        let counterparty = record
            .counterparty
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Unknown".to_string());

        *counterparty_map.entry(counterparty).or_insert(0) += 1;

        match record.direction.to_ascii_lowercase().as_str() {
            "incoming" => {
                incoming_sompi = incoming_sompi.saturating_add(amount);
                incoming_count += 1;
                largest_inflow = largest_inflow.max(amount);
            }
            "outgoing" => {
                outgoing_sompi = outgoing_sompi.saturating_add(amount);
                outgoing_count += 1;
                largest_outflow = largest_outflow.max(amount);
            }
            _ => {}
        }
    }

    let net_sompi = incoming_sompi.saturating_sub(outgoing_sompi);
    let duration_days = if first_tx > 0 && last_tx >= first_tx {
        ((last_tx - first_tx) / 86_400_000) + 1
    } else {
        0
    };

    let top_counterparty = counterparty_map
        .iter()
        .max_by_key(|(_, count)| **count)
        .map(|(counterparty, count)| format!("{counterparty} ({count})"))
        .unwrap_or_else(|| "N/A".to_string());

    let rows = vec![
        metric_row("Total Transactions", records.len().to_string(), "count"),
        metric_row(
            "Unique Addresses",
            unique_addresses.len().to_string(),
            "count",
        ),
        metric_row("Incoming Transactions", incoming_count.to_string(), "count"),
        metric_row("Outgoing Transactions", outgoing_count.to_string(), "count"),
        metric_row("Total Inflow KAS", sompi_to_kas(incoming_sompi), "kas"),
        metric_row("Total Outflow KAS", sompi_to_kas(outgoing_sompi), "kas"),
        metric_row("Net Flow KAS", sompi_to_kas(net_sompi), "kas"),
        metric_row("Largest Inflow KAS", sompi_to_kas(largest_inflow), "kas"),
        metric_row("Largest Outflow KAS", sompi_to_kas(largest_outflow), "kas"),
        metric_row(
            "Average Inflow KAS",
            average_kas(incoming_sompi, incoming_count),
            "kas",
        ),
        metric_row(
            "Average Outflow KAS",
            average_kas(outgoing_sompi, outgoing_count),
            "kas",
        ),
        metric_row(
            "Unique Counterparties",
            counterparty_map.len().to_string(),
            "count",
        ),
        metric_row("Top Counterparty", top_counterparty, "counterparty"),
        metric_row("Duration Days", duration_days.to_string(), "days"),
        metric_row("First Transaction Ms", first_tx.to_string(), "timestamp_ms"),
        metric_row("Last Transaction Ms", last_tx.to_string(), "timestamp_ms"),
    ];

    Ok(ReportTable {
        title: "Kaspa Gateway Analysis Report".to_string(),
        subtitle: format!("Time range: {time_range}"),
        headers: vec![
            "Metric".to_string(),
            "Value".to_string(),
            "Unit".to_string(),
        ],
        rows,
    })
}

fn full_report(
    address_filter: Option<String>,
    time_range: &str,
    limit: usize,
) -> Result<ReportTable, String> {
    let addresses = load_addresses(limit)?;
    let transactions = load_transactions(address_filter, time_range, limit)?;

    let mut rows = Vec::new();

    rows.push(vec![
        "Summary".to_string(),
        "Saved Addresses".to_string(),
        addresses.len().to_string(),
        String::new(),
    ]);

    rows.push(vec![
        "Summary".to_string(),
        "Transactions".to_string(),
        transactions.len().to_string(),
        format!("Time range: {time_range}"),
    ]);

    for address in addresses {
        rows.push(vec![
            "Address".to_string(),
            address.name,
            address.address,
            address.network,
        ]);
    }

    for record in transactions {
        rows.push(vec![
            "Transaction".to_string(),
            record.timestamp_ms.to_string(),
            sompi_to_kas(record.amount_sompi),
            format!(
                "{} | {} | {}",
                record.direction, record.tx_type, record.txid
            ),
        ]);
    }

    Ok(ReportTable {
        title: "Kaspa Gateway Full Report".to_string(),
        subtitle: "Combined addresses and transactions report.".to_string(),
        headers: vec![
            "Section".to_string(),
            "Field A".to_string(),
            "Field B".to_string(),
            "Field C".to_string(),
        ],
        rows,
    })
}

fn load_addresses(limit: usize) -> Result<Vec<AddressRecord>, String> {
    let manager = database_manager()?;
    let repo = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let mut records = repo.list().map_err(|error| error.to_string())?;
    records.truncate(limit);

    Ok(records)
}

fn load_transactions(
    address_filter: Option<String>,
    time_range: &str,
    limit: usize,
) -> Result<Vec<TransactionRecord>, String> {
    let manager = database_manager()?;
    let tx_repo = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let mut records = Vec::new();

    if let Some(address) = address_filter {
        let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;

        records.extend(
            tx_repo
                .list_for_address(parsed.as_str(), limit)
                .map_err(|error| error.to_string())?,
        );
    } else {
        let address_repo = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        for address in address_repo.list().map_err(|error| error.to_string())? {
            let remaining = limit.saturating_sub(records.len());

            if remaining == 0 {
                break;
            }

            let mut rows = tx_repo
                .list_for_address(&address.address, remaining)
                .map_err(|error| error.to_string())?;

            records.append(&mut rows);
        }
    }

    apply_time_range(&mut records, time_range)?;
    records.sort_by_key(|record| std::cmp::Reverse(record.timestamp_ms));
    records.truncate(limit);

    Ok(records)
}

fn apply_time_range(records: &mut Vec<TransactionRecord>, time_range: &str) -> Result<(), String> {
    let Some(start_ms) = time_range_start_ms(time_range)? else {
        return Ok(());
    };

    records.retain(|record| record.timestamp_ms >= start_ms);
    Ok(())
}

fn time_range_start_ms(value: &str) -> Result<Option<i64>, String> {
    let now = now_ms();

    let days: i64 = match value {
        "all" | "All" | "" => return Ok(None),
        "last_3_days" | "Last3Days" => 3,
        "last_week" | "LastWeek" => 7,
        "last_month" | "LastMonth" => 30,
        "last_3_months" | "Last3Months" => 90,
        "last_6_months" | "Last6Months" => 180,
        "last_year" | "LastYear" => 365,
        _ => return Err("Unsupported time range.".to_string()),
    };

    Ok(Some(now.saturating_sub(days.saturating_mul(86_400_000))))
}

fn address_row(record: AddressRecord) -> Vec<String> {
    vec![
        record.name,
        record.address,
        record.network,
        record.created_at_ms.to_string(),
        record.updated_at_ms.to_string(),
    ]
}

fn transaction_row(record: TransactionRecord) -> Vec<String> {
    vec![
        record.timestamp_ms.to_string(),
        record.direction,
        record.tx_type,
        sompi_to_kas(record.amount_sompi),
        record.amount_sompi.to_string(),
        record.address,
        record.counterparty.unwrap_or_default(),
        record.txid,
    ]
}

fn metric_row(metric: &str, value: String, unit: &str) -> Vec<String> {
    vec![metric.to_string(), value, unit.to_string()]
}

fn write_csv(path: &Path, table: &ReportTable) -> Result<(), String> {
    let mut output = String::new();

    output.push_str("# ");
    output.push_str(&table.title);
    output.push('\n');
    output.push_str("# ");
    output.push_str(&table.subtitle);
    output.push('\n');

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
.subtitle{margin-bottom:14px;color:#555}
</style>"#,
    );
    output.push_str("</head><body><table><caption>");
    output.push_str(&html_escape(&table.title));
    output.push_str("</caption><div class=\"subtitle\">");
    output.push_str(&html_escape(&table.subtitle));
    output.push_str("</div><thead><tr>");

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
    let mut lines = Vec::new();

    lines.push(table.title.clone());
    lines.push(table.subtitle.clone());
    lines.push(table.headers.join(" | "));

    for row in table.rows.iter().take(80) {
        lines.push(row.join(" | "));
    }

    if table.rows.len() > 80 {
        lines.push(format!("... truncated: {} total rows", table.rows.len()));
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

fn ensure_extension(mut path: PathBuf, format: &str) -> PathBuf {
    let needs_extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| !value.eq_ignore_ascii_case(format))
        .unwrap_or(true);

    if needs_extension {
        path.set_extension(format);
    }

    path
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

fn normalize_report_type(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "addresses" | "address" => Ok("Addresses".to_string()),
        "transactions" | "transaction" | "tx" => Ok("Transactions".to_string()),
        "analysis" => Ok("Analysis".to_string()),
        "full" | "summary" | "fullsummary" | "full_summary" => Ok("Full".to_string()),
        _ => Err("Unsupported report type.".to_string()),
    }
}

fn normalize_format(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "csv" => Ok("csv".to_string()),
        "html" | "htm" => Ok("html".to_string()),
        "pdf" => Ok("pdf".to_string()),
        _ => Err("Unsupported export format.".to_string()),
    }
}

fn normalize_time_range(value: &str) -> Result<String, String> {
    match value.trim() {
        "" | "all" | "All" => Ok("all".to_string()),
        "last_3_days" | "Last3Days" => Ok("last_3_days".to_string()),
        "last_week" | "LastWeek" => Ok("last_week".to_string()),
        "last_month" | "LastMonth" => Ok("last_month".to_string()),
        "last_3_months" | "Last3Months" => Ok("last_3_months".to_string()),
        "last_6_months" | "Last6Months" => Ok("last_6_months".to_string()),
        "last_year" | "LastYear" => Ok("last_year".to_string()),
        _ => Err("Unsupported time range.".to_string()),
    }
}

fn csv_cell(value: &str) -> String {
    let mut value = value.replace('\0', "");
    value = value.replace(['\r', '\n'], " ");

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

fn sompi_to_kas(value: i64) -> String {
    format!("{:.8}", value as f64 / 100_000_000.0)
}

fn average_kas(total_sompi: i64, count: usize) -> String {
    if count == 0 {
        "0.00000000".to_string()
    } else {
        format!("{:.8}", total_sompi as f64 / count as f64 / 100_000_000.0)
    }
}

fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn now_file_stamp() -> String {
    now_ms().to_string()
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
    fn normalizes_report_type_and_format() {
        assert_eq!(normalize_report_type("tx").unwrap(), "Transactions");
        assert_eq!(normalize_report_type("summary").unwrap(), "Full");
        assert_eq!(normalize_format("HTML").unwrap(), "html");
    }

    #[test]
    fn rejects_unsafe_output_path() {
        assert!(validate_output_path("C:\\temp\\report.csv").is_ok());
        assert!(validate_output_path("bad && whoami").is_err());
    }

    #[test]
    fn csv_cell_blocks_formula_injection() {
        assert_eq!(csv_cell("=cmd"), "\"'=cmd\"");
    }

    #[test]
    fn pdf_builder_outputs_header() {
        let stream = build_pdf_content_stream(&["Export".to_string()]);
        let pdf = build_minimal_pdf(&stream);

        assert!(pdf.starts_with(b"%PDF-1.4"));
    }
}
