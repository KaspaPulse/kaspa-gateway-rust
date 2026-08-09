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
    #[serde(alias = "reportType")]
    pub report_type: String,
    pub format: String,
    #[serde(alias = "outputPath")]
    pub output_path: String,
    #[serde(default, alias = "addressFilter")]
    pub address_filter: Option<String>,
    #[serde(alias = "timeRange")]
    pub time_range: String,
    pub limit: usize,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default, alias = "clientTable")]
    pub client_table: Option<ExportClientTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportClientTable {
    pub title: String,
    pub subtitle: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
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
        "html" => write_html(&output_path, &table, request.locale.as_deref())?,
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
    if let Some(client_table) = request.client_table.as_ref() {
        return client_report_table(client_table);
    }

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

fn client_report_table(client_table: &ExportClientTable) -> Result<ReportTable, String> {
    if client_table.headers.is_empty() {
        return Err("Export table must contain at least one header.".to_string());
    }

    let column_count = client_table.headers.len();

    let rows = client_table
        .rows
        .iter()
        .map(|row| {
            let mut normalized = row
                .iter()
                .map(|cell| cell.replace('\0', "").replace(['\r', '\n'], " "))
                .collect::<Vec<_>>();

            if normalized.len() < column_count {
                normalized.resize(column_count, String::new());
            }

            normalized.truncate(column_count);
            normalized
        })
        .collect::<Vec<_>>();

    Ok(ReportTable {
        title: client_table.title.trim().to_string(),
        subtitle: client_table.subtitle.trim().to_string(),
        headers: client_table.headers.clone(),
        rows,
    })
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

fn kgw_export_csv_metadata_comment_v13(value: &str) -> String {
    let normalized = value.replace(['\r', '\n'], " ").trim().to_string();

    if normalized.is_empty() {
        String::new()
    } else {
        normalized
    }
}

fn kgw_export_csv_generated_on_v13() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => format!("Unix Time: {}", duration.as_secs()),
        Err(_) => "Unix Time: unavailable".to_string(),
    }
}

fn kgw_export_csv_write_comment_v13(output: &mut String, label: Option<&str>, value: &str) {
    let value = kgw_export_csv_metadata_comment_v13(value);
    if value.is_empty() {
        return;
    }

    output.push_str("# ");

    if let Some(label) = label {
        let label = kgw_export_csv_metadata_comment_v13(label);
        if !label.is_empty() {
            output.push_str(&label);
            output.push_str(": ");
        }
    }

    output.push_str(&value);
    output.push('\n');
}

fn write_csv(path: &Path, table: &ReportTable) -> Result<(), String> {
    // KGW_EXPORT_PHASE_B_CSV_TEMPLATE_PARITY_V13
    //
    // Python parity intent:
    // - keep UTF-8 BOM behavior equivalent to Python utf-8-sig
    // - write report metadata comment rows before the table
    // - keep existing csv_cell formula-injection protection
    // - keep this as the single CSV writer owner in export_commands.rs

    let mut output = String::new();

    output.push('\u{feff}');

    kgw_export_csv_write_comment_v13(
        &mut output,
        Some("Kaspa Gateway Version"),
        env!("CARGO_PKG_VERSION"),
    );

    kgw_export_csv_write_comment_v13(&mut output, None, table.title.as_str());

    if !table.subtitle.trim().is_empty() {
        kgw_export_csv_write_comment_v13(&mut output, Some("Details"), table.subtitle.as_str());
    }

    kgw_export_csv_write_comment_v13(
        &mut output,
        Some("Exported On"),
        kgw_export_csv_generated_on_v13().as_str(),
    );

    output.push('\n');

    let header_line = table
        .headers
        .iter()
        .map(|header| csv_cell(header))
        .collect::<Vec<_>>()
        .join(",");

    output.push_str(&header_line);
    output.push('\n');

    for row in &table.rows {
        let row_line = row
            .iter()
            .map(|cell| csv_cell(cell))
            .collect::<Vec<_>>()
            .join(",");

        output.push_str(&row_line);
        output.push('\n');
    }

    std::fs::write(path, output).map_err(|error| error.to_string())
}

fn kgw_export_html_generated_on_v16() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => format!("Unix Time: {}", duration.as_secs()),
        Err(_) => "Unix Time: unavailable".to_string(),
    }
}

fn kgw_export_html_metadata_row_v16(label: &str, value: &str) -> String {
    if value.trim().is_empty() {
        return String::new();
    }

    format!(
        "<div class=\"kgw-report-meta-row\"><span>{}</span><strong>{}</strong></div>",
        html_escape(label),
        html_escape(value)
    )
}

// KGW_EXPORT_TEMPLATE_PARITY_URL_LINKS_V1B
fn is_export_url_v1b(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();

    (lower.starts_with("https://") || lower.starts_with("http://"))
        && !value.chars().any(char::is_whitespace)
}

// KGW_EXPORT_HTML_PDF_READABLE_LONG_CELLS_V5
fn kgw_middle_ellipsis_v5(value: &str, head: usize, tail: usize) -> String {
    let clean = value
        .replace(['\r', '\n', '\t'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let chars = clean.chars().collect::<Vec<_>>();

    if chars.len() <= head + tail + 3 {
        return clean;
    }

    let prefix = chars.iter().take(head).collect::<String>();
    let suffix = chars
        .iter()
        .rev()
        .take(tail)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();

    format!("{prefix}...{suffix}")
}

fn kgw_compact_url_v5(value: &str) -> String {
    let clean = value.trim();

    if clean.contains(" | ") {
        return clean
            .split(" | ")
            .map(|part| kgw_compact_url_v5(part.trim()))
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" | ");
    }

    let lower = clean.to_ascii_lowercase();

    if lower.contains("/txs/")
        && let Some(txid) = clean.rsplit('/').next()
    {
        return format!(
            "explorer.kaspa.org/txs/{}",
            kgw_middle_ellipsis_v5(txid, 10, 8)
        );
    }

    if lower.contains("/addresses/")
        && let Some(address) = clean.rsplit('/').next()
    {
        return format!(
            "explorer.kaspa.org/addresses/{}",
            kgw_middle_ellipsis_v5(address, 14, 8)
        );
    }

    kgw_middle_ellipsis_v5(clean, 26, 14)
}

fn kgw_compact_address_v5(value: &str) -> String {
    let clean = value.trim();

    if clean.contains(" | ") {
        return clean
            .split(" | ")
            .map(|part| kgw_compact_address_v5(part.trim()))
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" | ");
    }

    if clean.starts_with("kaspa:") {
        return kgw_middle_ellipsis_v5(clean, 14, 8);
    }

    if clean.len() >= 48 && clean.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return kgw_middle_ellipsis_v5(clean, 10, 8);
    }

    clean.to_string()
}

fn kgw_display_cell_v5(header: &str, value: &str) -> String {
    let clean = value.trim();
    let lower = header.to_ascii_lowercase();

    if clean.is_empty() {
        return String::new();
    }

    if lower.contains("url") {
        return kgw_compact_url_v5(clean);
    }

    if lower.contains("transaction id") || lower == "txid" || lower.contains("tx id") {
        return kgw_middle_ellipsis_v5(clean, 10, 8);
    }

    if lower.contains("address") {
        return kgw_compact_address_v5(clean);
    }

    if lower.contains("timestamp") && clean.len() > 12 {
        return kgw_middle_ellipsis_v5(clean, 8, 4);
    }

    clean.to_string()
}

fn kgw_html_attr_escape_v5(value: &str) -> String {
    html_escape(value).replace('"', "&quot;")
}

fn html_cell_with_header_v5(header: &str, value: &str) -> String {
    let display = kgw_display_cell_v5(header, value);
    let title = kgw_html_attr_escape_v5(value);

    if is_export_url_v1b(value.trim()) {
        let href = kgw_html_attr_escape_v5(value.trim());
        return format!(
            "<a href=\"{href}\" target=\"_blank\" rel=\"noopener noreferrer\" title=\"{title}\">{}</a>",
            html_escape(&display)
        );
    }

    if value.contains(" | ") {
        let parts = value
            .split(" | ")
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();

        if !parts.is_empty() && parts.iter().all(|part| is_export_url_v1b(part)) {
            return parts
                .iter()
                .map(|part| {
                    let display_part = kgw_compact_url_v5(part);
                    let href = kgw_html_attr_escape_v5(part);
                    let title = kgw_html_attr_escape_v5(part);

                    format!(
                        "<a href=\"{href}\" target=\"_blank\" rel=\"noopener noreferrer\" title=\"{title}\">{}</a>",
                        html_escape(&display_part)
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ");
        }
    }

    if display == value {
        html_escape(value)
    } else {
        format!(
            "<span title=\"{title}\" class=\"kgw-export-compact-cell\">{}</span>",
            html_escape(&display)
        )
    }
}

// KGW_EXPORT_COMPACT_REPORT_LAYOUT_V6
fn kgw_header_index_v6(headers: &[String], candidates: &[&str]) -> Option<usize> {
    headers.iter().position(|header| {
        let lower = header.to_ascii_lowercase();

        candidates.iter().any(|candidate| {
            let candidate_lower = candidate.to_ascii_lowercase();
            lower == candidate_lower || lower.contains(&candidate_lower)
        })
    })
}

fn kgw_row_cell_v6(row: &[String], index: Option<usize>) -> String {
    index
        .and_then(|idx| row.get(idx))
        .map(|value| value.trim().to_string())
        .unwrap_or_default()
}

fn kgw_short_date_time_v6(value: &str) -> String {
    let clean = value.trim();

    if clean.len() >= 16 {
        return clean[..16].to_string();
    }

    clean.to_string()
}

fn kgw_pdf_compact_table_v6(table: &ReportTable) -> ReportTable {
    let headers = &table.headers;

    let txid_idx = kgw_header_index_v6(headers, &["transaction id", "txid", "tx id"]);
    let datetime_idx = kgw_header_index_v6(headers, &["date/time", "datetime", "time"]);
    let direction_idx = kgw_header_index_v6(headers, &["direction"]);
    let from_idx = kgw_header_index_v6(headers, &["from address"]);
    let to_idx = kgw_header_index_v6(headers, &["to address"]);
    let amount_idx = kgw_header_index_v6(headers, &["amount"]);
    let value_idx = kgw_header_index_v6(headers, &["value"]);
    let block_idx = kgw_header_index_v6(headers, &["block score", "block"]);
    let type_idx = kgw_header_index_v6(headers, &["type"]);

    let looks_like_explorer =
        txid_idx.is_some() && datetime_idx.is_some() && amount_idx.is_some() && headers.len() >= 10;

    if !looks_like_explorer {
        return table.clone();
    }

    let rows = table
        .rows
        .iter()
        .map(|row| {
            let txid = kgw_row_cell_v6(row, txid_idx);
            let from = kgw_row_cell_v6(row, from_idx);
            let to = kgw_row_cell_v6(row, to_idx);

            let reference = if !txid.is_empty() {
                kgw_middle_ellipsis_v5(&txid, 8, 6)
            } else {
                String::new()
            };

            let parties = match (from.trim().is_empty(), to.trim().is_empty()) {
                (true, true) => String::new(),
                (false, true) => kgw_compact_address_v5(&from),
                (true, false) => kgw_compact_address_v5(&to),
                (false, false) => format!(
                    "{} -> {}",
                    kgw_compact_address_v5(&from),
                    kgw_compact_address_v5(&to)
                ),
            };

            vec![
                kgw_short_date_time_v6(&kgw_row_cell_v6(row, datetime_idx)),
                kgw_row_cell_v6(row, direction_idx),
                kgw_row_cell_v6(row, amount_idx),
                kgw_row_cell_v6(row, value_idx),
                kgw_row_cell_v6(row, type_idx),
                kgw_row_cell_v6(row, block_idx),
                reference,
                parties,
            ]
        })
        .collect::<Vec<_>>();

    ReportTable {
        title: table.title.clone(),
        subtitle: format!(
            "{} | PDF compact layout; CSV/HTML keep full raw export values.",
            table.subtitle
        ),
        headers: vec![
            "Date/Time".to_string(),
            "Dir".to_string(),
            "Amount".to_string(),
            "Value".to_string(),
            "Type".to_string(),
            "Block".to_string(),
            "Tx".to_string(),
            "Parties".to_string(),
        ],
        rows,
    }
}

fn kgw_pdf_compact_font_size_v6(headers: &[String]) -> f32 {
    if headers.len() <= 8 { 5.05 } else { 4.75 }
}

fn kgw_pdf_compact_header_font_size_v6(headers: &[String]) -> f32 {
    if headers.len() <= 8 { 5.65 } else { 5.25 }
}

fn write_html(path: &Path, table: &ReportTable, locale: Option<&str>) -> Result<(), String> {
    // KGW_EXPORT_PHASE_C_HTML_TEMPLATE_PARITY_V16C
    //
    // Python parity intent:
    // - keep one canonical HTML writer owner in export_commands.rs
    // - keep the existing call contract: locale: Option<&str>
    // - render a complete full HTML document
    // - preserve lang/dir for RTL Arabic
    // - include report metadata, CSS, escaped table values, and footer
    // - keep frontend Native Save As and CSV behavior unchanged

    let locale = locale.unwrap_or("en").trim();
    let lang = if locale.is_empty() { "en" } else { locale };
    let is_rtl = lang.to_ascii_lowercase().starts_with("ar");
    let dir = if is_rtl { "rtl" } else { "ltr" };
    let align = if is_rtl { "right" } else { "left" };
    let opposite_align = if is_rtl { "left" } else { "right" };

    let mut metadata = String::new();

    metadata.push_str(&kgw_export_html_metadata_row_v16(
        "Kaspa Gateway Version",
        env!("CARGO_PKG_VERSION"),
    ));

    metadata.push_str(&kgw_export_html_metadata_row_v16("Report", &table.title));

    if !table.subtitle.trim().is_empty() {
        metadata.push_str(&kgw_export_html_metadata_row_v16(
            "Details",
            &table.subtitle,
        ));
    }

    let generated_on = kgw_export_html_generated_on_v16();
    metadata.push_str(&kgw_export_html_metadata_row_v16(
        "Exported On",
        &generated_on,
    ));

    let row_count = table.rows.len().to_string();
    metadata.push_str(&kgw_export_html_metadata_row_v16("Rows", &row_count));

    let headers = table
        .headers
        .iter()
        .map(|header| format!("<th>{}</th>", html_escape(header)))
        .collect::<Vec<_>>()
        .join("");

    let rows = table
        .rows
        .iter()
        .map(|row| {
            let cells = row
                .iter()
                .enumerate()
                .map(|(index, cell)| {
                    let header = table.headers.get(index).map(String::as_str).unwrap_or("");
                    format!("<td>{}</td>", html_cell_with_header_v5(header, cell))
                })
                .collect::<Vec<_>>()
                .join("");

            format!("<tr>{cells}</tr>")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r#"<!doctype html>
<html lang="{lang}" dir="{dir}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title}</title>
  <style>
    :root {{
      color-scheme: light dark;
      --kgw-bg: #f5f7fb;
      --kgw-card: #ffffff;
      --kgw-text: #172033;
      --kgw-muted: #64748b;
      --kgw-border: #dbe3ef;
      --kgw-header: #10233f;
      --kgw-header-text: #ffffff;
      --kgw-row: #f8fbff;
      --kgw-link: #1d5fd1;
    }}

    * {{
      box-sizing: border-box;
    }}

    body {{
      margin: 0;
      padding: 28px;
      background: var(--kgw-bg);
      color: var(--kgw-text);
      font-family: "Segoe UI", Tahoma, Arial, sans-serif;
      line-height: 1.55;
      text-align: {align};
    }}

    .kgw-report {{
      max-width: 1180px;
      margin: 0 auto;
      background: var(--kgw-card);
      border: 1px solid var(--kgw-border);
      border-radius: 14px;
      box-shadow: 0 18px 50px rgba(15, 23, 42, 0.10);
      overflow: hidden;
    }}

    .kgw-report-header {{
      padding: 28px 30px 22px;
      background: linear-gradient(135deg, #10233f, #214b82);
      color: var(--kgw-header-text);
    }}

    .kgw-report-title {{
      margin: 0;
      font-size: 26px;
      font-weight: 800;
      letter-spacing: 0.01em;
    }}

    .kgw-report-subtitle {{
      margin: 8px 0 0;
      color: rgba(255,255,255,0.84);
      font-size: 14px;
    }}

    .kgw-report-meta {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(230px, 1fr));
      gap: 10px;
      padding: 20px 30px;
      border-bottom: 1px solid var(--kgw-border);
      background: #f8fbff;
    }}

    .kgw-report-meta-row {{
      min-height: 46px;
      padding: 10px 12px;
      border: 1px solid var(--kgw-border);
      border-radius: 10px;
      background: #ffffff;
    }}

    .kgw-report-meta-row span {{
      display: block;
      margin-bottom: 3px;
      color: var(--kgw-muted);
      font-size: 12px;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }}

    .kgw-report-meta-row strong {{
      display: block;
      color: var(--kgw-text);
      font-size: 14px;
      overflow-wrap: anywhere;
    }}

    .kgw-table-wrap {{
      width: 100%;
      overflow-x: auto;
      padding: 22px 30px 30px;
    }}

    table {{
      width: 100%;
      border-collapse: collapse;
      border: 1px solid var(--kgw-border);
      background: #ffffff;
      font-size: 13px;
    }}

    thead th {{
      background: var(--kgw-header);
      color: var(--kgw-header-text);
      text-align: {align};
      padding: 11px 12px;
      border: 1px solid rgba(255,255,255,0.12);
      white-space: nowrap;
    }}

    tbody td {{
      padding: 10px 12px;
      border: 1px solid var(--kgw-border);
      vertical-align: top;
      overflow-wrap: anywhere;
    }}

    tbody tr:nth-child(even) {{
      background: var(--kgw-row);
    }}

    a {{
      color: var(--kgw-link);
      text-decoration: none;
    }}

    a:hover {{
      text-decoration: underline;
    }}

    .kgw-report-footer {{
      display: flex;
      justify-content: space-between;
      gap: 12px;
      padding: 16px 30px 20px;
      border-top: 1px solid var(--kgw-border);
      color: var(--kgw-muted);
      font-size: 12px;
      text-align: {align};
    }}

    .kgw-report-footer .kgw-footer-side {{
      text-align: {opposite_align};
    }}

    @media print {{
      body {{
        padding: 0;
        background: #ffffff;
      }}

      .kgw-report {{
        box-shadow: none;
        border: 0;
        border-radius: 0;
      }}
    }}
  </style>
</head>
<body>
  <main class="kgw-report">
    <header class="kgw-report-header">
      <h1 class="kgw-report-title">{title}</h1>
      <p class="kgw-report-subtitle">{subtitle}</p>
    </header>

    <section class="kgw-report-meta" aria-label="Report metadata">
      {metadata}
    </section>

    <section class="kgw-table-wrap">
      <table>
        <thead>
          <tr>{headers}</tr>
        </thead>
        <tbody>
          {rows}
        </tbody>
      </table>
    </section>

    <footer class="kgw-report-footer">
      <span>Generated by Kaspa Gateway</span>
      <span class="kgw-footer-side">Rows: {row_count}</span>
    </footer>
  </main>
</body>
</html>
"#,
        lang = html_escape(lang),
        dir = dir,
        align = align,
        opposite_align = opposite_align,
        title = html_escape(&table.title),
        subtitle = html_escape(&table.subtitle),
        metadata = metadata,
        headers = headers,
        rows = rows,
        row_count = table.rows.len(),
    );

    std::fs::write(path, html).map_err(|error| error.to_string())
}

fn kgw_pdf_color_v29(r: f32, g: f32, b: f32) -> printpdf::Color {
    printpdf::Color::Rgb(printpdf::Rgb::new(r, g, b, None))
}

fn kgw_pdf_point_v29(x: f32, y: f32) -> printpdf::Point {
    printpdf::Point::new(printpdf::Mm(x), printpdf::Mm(y))
}

fn kgw_pdf_draw_line_v29(ops: &mut Vec<printpdf::Op>, x1: f32, y1: f32, x2: f32, y2: f32) {
    ops.push(printpdf::Op::SetOutlineColor {
        col: kgw_pdf_color_v29(0.70, 0.70, 0.70),
    });
    ops.push(printpdf::Op::SetOutlineThickness {
        pt: printpdf::Pt(0.35),
    });
    ops.push(printpdf::Op::DrawLine {
        line: printpdf::Line {
            points: vec![
                printpdf::LinePoint {
                    p: kgw_pdf_point_v29(x1, y1),
                    bezier: false,
                },
                printpdf::LinePoint {
                    p: kgw_pdf_point_v29(x2, y2),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
}

fn kgw_pdf_draw_rect_border_v29(ops: &mut Vec<printpdf::Op>, x: f32, y: f32, w: f32, h: f32) {
    kgw_pdf_draw_line_v29(ops, x, y, x + w, y);
    kgw_pdf_draw_line_v29(ops, x + w, y, x + w, y - h);
    kgw_pdf_draw_line_v29(ops, x + w, y - h, x, y - h);
    kgw_pdf_draw_line_v29(ops, x, y - h, x, y);
}

fn kgw_pdf_text_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    text: &str,
    x: f32,
    y: f32,
    size: f32,
) {
    ops.push(printpdf::Op::StartTextSection);
    ops.push(printpdf::Op::SetTextCursor {
        pos: kgw_pdf_point_v29(x, y),
    });
    ops.push(printpdf::Op::SetFont {
        font: font.clone(),
        size: printpdf::Pt(size),
    });
    ops.push(printpdf::Op::SetLineHeight {
        lh: printpdf::Pt(size + 2.0),
    });
    ops.push(printpdf::Op::ShowText {
        items: vec![printpdf::TextItem::Text(text.to_string())],
    });
    ops.push(printpdf::Op::EndTextSection);
}

fn kgw_pdf_clean_text_v29(value: &str) -> String {
    value
        .replace(['\r', '\n', '\t'], " ")
        .replace(['—', '–', '•'], "-")
        .replace('…', "...")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn kgw_pdf_column_weight_v30(header: &str) -> f32 {
    let lower = header.to_ascii_lowercase();

    if lower == "parties" {
        2.35
    } else if lower == "tx" {
        1.30
    } else if lower == "date/time" {
        1.05
    } else if lower == "dir" {
        0.55
    } else if lower == "amount" || lower == "value" {
        0.80
    } else if lower == "type" {
        0.65
    } else if lower == "block" {
        0.75
    } else if lower.contains("url") {
        2.20
    } else if lower.contains("address") {
        1.85
    } else if lower.contains("transaction") || lower.contains("txid") || lower == "txid" {
        1.65
    } else if lower.contains("date") || lower.contains("time") {
        1.05
    } else if lower.contains("amount") || lower.contains("balance") || lower.contains("value") {
        0.95
    } else if lower.contains("rank") {
        0.45
    } else if lower.contains("direction") || lower.contains("type") {
        0.65
    } else {
        1.00
    }
}

fn kgw_pdf_column_widths_v29(headers: &[String]) -> Vec<f32> {
    const TABLE_WIDTH_MM: f32 = 269.0;

    if headers.is_empty() {
        return vec![TABLE_WIDTH_MM];
    }

    let weights = headers
        .iter()
        .map(|header| kgw_pdf_column_weight_v30(header))
        .collect::<Vec<_>>();

    let total_weight = weights.iter().sum::<f32>().max(1.0);
    let min_width = if headers.len() >= 10 { 8.0 } else { 12.0 };

    let mut widths = weights
        .iter()
        .map(|weight| ((weight / total_weight) * TABLE_WIDTH_MM).max(min_width))
        .collect::<Vec<_>>();

    let sum = widths.iter().sum::<f32>();

    if sum > TABLE_WIDTH_MM {
        let scale = TABLE_WIDTH_MM / sum;
        for width in &mut widths {
            *width = (*width * scale).max(6.0);
        }
    }

    widths
}

fn kgw_pdf_chars_for_width_v30(width: f32, font_size: f32) -> usize {
    ((width / (font_size * 0.36)).floor() as usize).max(4)
}

fn kgw_pdf_chunk_word_v30(word: &str, max_chars: usize) -> Vec<String> {
    let chars = word.chars().collect::<Vec<_>>();

    if chars.len() <= max_chars {
        return vec![word.to_string()];
    }

    chars
        .chunks(max_chars.max(1))
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
}

fn kgw_pdf_wrap_segment_v30(segment: &str, max_chars: usize) -> Vec<String> {
    let clean = kgw_pdf_clean_text_v29(segment);

    if clean.is_empty() {
        return vec![String::new()];
    }

    if !clean.contains(' ') {
        return kgw_pdf_chunk_word_v30(&clean, max_chars);
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in clean.split_whitespace() {
        let word_len = word.chars().count();

        if word_len > max_chars {
            if !current.is_empty() {
                lines.push(current);
                current = String::new();
            }

            lines.extend(kgw_pdf_chunk_word_v30(word, max_chars));
            continue;
        }

        let next_len = if current.is_empty() {
            word_len
        } else {
            current.chars().count() + 1 + word_len
        };

        if next_len > max_chars && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }

            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn kgw_pdf_wrap_text_v30(value: &str, width: f32, font_size: f32) -> Vec<String> {
    let max_chars = kgw_pdf_chars_for_width_v30(width, font_size);
    let clean = kgw_pdf_clean_text_v29(value);

    if clean.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();

    for segment in clean.split(" | ") {
        let segment_lines = kgw_pdf_wrap_segment_v30(segment.trim(), max_chars);

        if !lines.is_empty() && !segment.trim().is_empty() {
            lines.push("|".to_string());
        }

        lines.extend(segment_lines);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn kgw_pdf_row_height_v30(row: &[String], widths: &[f32], font_size: f32) -> f32 {
    let mut max_lines = 1usize;

    for (index, width) in widths.iter().enumerate() {
        let cell = row.get(index).map(String::as_str).unwrap_or("");
        let lines = kgw_pdf_wrap_text_v30(cell, *width - 3.0, font_size);
        max_lines = max_lines.max(lines.len().min(2));
    }

    let line_gap = font_size * 0.42 + 1.05;
    ((max_lines as f32) * line_gap + 3.0).max(6.2)
}

fn kgw_pdf_draw_table_header_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    headers: &[String],
    widths: &[f32],
    x0: f32,
    y: f32,
    row_h: f32,
) {
    let mut x = x0;
    let font_size = kgw_pdf_compact_header_font_size_v6(headers);

    for (index, width) in widths.iter().enumerate() {
        let header = headers.get(index).cloned().unwrap_or_default();
        let mut lines = kgw_pdf_wrap_text_v30(&header, *width - 3.0, font_size);

        if lines.len() > 2 {
            lines.truncate(2);
        }

        kgw_pdf_draw_rect_border_v29(ops, x, y, *width, row_h);

        for (line_index, line) in lines.iter().enumerate() {
            let text_y = y - 4.0 - (line_index as f32 * 3.1);

            if text_y > y - row_h + 1.5 {
                kgw_pdf_text_v29(ops, font, line, x + 1.3, text_y, font_size);
            }
        }

        x += *width;
    }
}

#[allow(clippy::too_many_arguments)] // Explicit PDF layout inputs keep row rendering call sites auditable.
fn kgw_pdf_draw_table_row_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    headers: &[String],
    row: &[String],
    widths: &[f32],
    x0: f32,
    y: f32,
    row_h: f32,
) {
    let mut x = x0;
    let font_size = kgw_pdf_compact_font_size_v6(headers);

    for (index, width) in widths.iter().enumerate() {
        let header = headers.get(index).map(String::as_str).unwrap_or("");
        let cell = row.get(index).cloned().unwrap_or_default();
        let display_cell = kgw_display_cell_v5(header, &cell);
        let mut lines = kgw_pdf_wrap_text_v30(&display_cell, *width - 3.0, font_size);

        if lines.len() > 2 {
            lines.truncate(2);

            if let Some(last) = lines.last_mut() {
                *last = kgw_middle_ellipsis_v5(last, 14, 5);
            }
        }

        kgw_pdf_draw_rect_border_v29(ops, x, y, *width, row_h);

        for (line_index, line) in lines.iter().enumerate() {
            let text_y = y - 3.8 - (line_index as f32 * 3.0);

            if text_y > y - row_h + 1.3 {
                kgw_pdf_text_v29(ops, font, line, x + 1.2, text_y, font_size);
            }
        }

        x += *width;
    }
}

fn kgw_pdf_draw_metadata_line_v30(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    label: &str,
    value: &str,
    x: f32,
    y: f32,
) {
    let text = if value.trim().is_empty() {
        label.to_string()
    } else {
        format!("{label}: {value}")
    };

    kgw_pdf_text_v29(ops, font, &kgw_pdf_clean_text_v29(&text), x, y, 7.4);
}

fn kgw_pdf_draw_page_header_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    table: &ReportTable,
    page_no: usize,
) {
    kgw_pdf_text_v29(
        ops,
        font,
        &kgw_pdf_clean_text_v29(&table.title),
        14.0,
        195.0,
        15.0,
    );

    if page_no == 1 {
        if !table.subtitle.trim().is_empty() {
            let subtitle_lines = kgw_pdf_wrap_text_v30(&table.subtitle, 250.0, 7.6);

            for (index, line) in subtitle_lines.iter().take(3).enumerate() {
                kgw_pdf_text_v29(ops, font, line, 14.0, 186.0 - (index as f32 * 4.8), 7.6);
            }
        }

        kgw_pdf_draw_metadata_line_v30(
            ops,
            font,
            "Kaspa Gateway Version",
            env!("CARGO_PKG_VERSION"),
            14.0,
            169.0,
        );

        kgw_pdf_draw_metadata_line_v30(ops, font, "Report", &table.title, 14.0, 164.2);

        kgw_pdf_draw_metadata_line_v30(
            ops,
            font,
            "Exported On",
            &kgw_export_html_generated_on_v16(),
            14.0,
            159.4,
        );

        kgw_pdf_draw_metadata_line_v30(
            ops,
            font,
            "Rows",
            &table.rows.len().to_string(),
            150.0,
            169.0,
        );

        kgw_pdf_draw_metadata_line_v30(
            ops,
            font,
            "Columns",
            &table.headers.len().to_string(),
            150.0,
            164.2,
        );
    }
}

fn kgw_pdf_draw_page_footer_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    page_no: usize,
    total_pages: usize,
) {
    kgw_pdf_text_v29(
        ops,
        font,
        &format!("Generated by Kaspa Gateway    Page {page_no} / {total_pages}"),
        14.0,
        10.0,
        7.8,
    );
}

fn kgw_pdf_paginate_rows_v30(
    rows: &[Vec<String>],
    widths: &[f32],
    first_page_table_y: f32,
    next_page_table_y: f32,
    bottom_y: f32,
    header_h: f32,
) -> Vec<Vec<(usize, f32)>> {
    if rows.is_empty() {
        return vec![Vec::new()];
    }

    let mut pages = Vec::new();
    let mut index = 0usize;
    let mut first = true;

    while index < rows.len() {
        let table_y = if first {
            first_page_table_y
        } else {
            next_page_table_y
        };

        let available = (table_y - bottom_y - header_h).max(12.0);
        let mut used = 0.0f32;
        let mut page_rows = Vec::new();

        while index < rows.len() {
            let row_h = kgw_pdf_row_height_v30(&rows[index], widths, 5.55).max(8.0);
            let safe_h = row_h.min(available.max(8.0));

            if !page_rows.is_empty() && used + safe_h > available {
                break;
            }

            page_rows.push((index, safe_h));
            used += safe_h;
            index += 1;
        }

        if page_rows.is_empty() {
            page_rows.push((index, available.max(8.0)));
            index += 1;
        }

        pages.push(page_rows);
        first = false;
    }

    pages
}

fn kgw_pdf_build_pages_v29(table: &ReportTable) -> Vec<printpdf::PdfPage> {
    // KGW_EXPORT_COMPACT_REPORT_LAYOUT_BUILD_PAGES_V6
    //
    // PDF is a compact, print-oriented report. CSV remains the raw data export,
    // and HTML remains the full interactive report. This prevents hundreds of
    // unreadable PDF pages when rows contain long txids, addresses, and URLs.

    let table = kgw_pdf_compact_table_v6(table);
    let font = printpdf::PdfFontHandle::Builtin(printpdf::BuiltinFont::Helvetica);
    let widths = kgw_pdf_column_widths_v29(&table.headers);

    let x0: f32 = 14.0;
    let first_page_table_y: f32 = 151.0;
    let next_page_table_y: f32 = 184.0;
    let bottom_y: f32 = 18.0;
    let header_font = kgw_pdf_compact_header_font_size_v6(&table.headers);
    let row_font = kgw_pdf_compact_font_size_v6(&table.headers);
    let header_h = kgw_pdf_row_height_v30(&table.headers, &widths, header_font).max(7.2);

    let paged_rows = kgw_pdf_paginate_rows_v30(
        &table.rows,
        &widths,
        first_page_table_y,
        next_page_table_y,
        bottom_y,
        header_h,
    );

    let total_pages = paged_rows.len().max(1);
    let mut pages = Vec::new();

    for (page_index, rows) in paged_rows.iter().enumerate() {
        let page_no = page_index + 1;
        let mut ops = Vec::new();

        kgw_pdf_draw_page_header_v29(&mut ops, &font, &table, page_no);

        let table_y = if page_no == 1 {
            first_page_table_y
        } else {
            next_page_table_y
        };

        kgw_pdf_draw_table_header_v29(
            &mut ops,
            &font,
            &table.headers,
            &widths,
            x0,
            table_y,
            header_h,
        );

        let mut y = table_y - header_h;

        if table.rows.is_empty() {
            let row = vec!["No rows available".to_string()];
            kgw_pdf_draw_table_row_v29(&mut ops, &font, &table.headers, &row, &widths, x0, y, 8.0);
        } else {
            for (row_index, row_h) in rows {
                if let Some(row) = table.rows.get(*row_index) {
                    let compact_h = kgw_pdf_row_height_v30(row, &widths, row_font)
                        .min(*row_h)
                        .max(6.2);
                    kgw_pdf_draw_table_row_v29(
                        &mut ops,
                        &font,
                        &table.headers,
                        row,
                        &widths,
                        x0,
                        y,
                        compact_h,
                    );
                    y -= compact_h;
                }
            }
        }

        kgw_pdf_draw_page_footer_v29(&mut ops, &font, page_no, total_pages);

        pages.push(printpdf::PdfPage::new(
            printpdf::Mm(297.0),
            printpdf::Mm(210.0),
            ops,
        ));
    }

    pages
}

fn write_pdf(path: &Path, table: &ReportTable) -> Result<(), String> {
    // KGW_EXPORT_PDF_HTML_PARITY_WRITE_PDF_V2
    //
    // PDF output intentionally mirrors the HTML report model:
    // same ReportTable, same headers/rows order, visible URL values,
    // metadata header, repeated table headers, and page footer.
    //
    // No alternate export route is introduced.

    let mut doc = printpdf::PdfDocument::new(&table.title);
    let pages = kgw_pdf_build_pages_v29(table);

    let mut warnings = Vec::new();
    let pdf_bytes = doc
        .with_pages(pages)
        .save(&printpdf::PdfSaveOptions::default(), &mut warnings);

    std::fs::write(path, pdf_bytes).map_err(|error| error.to_string())
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

    let root = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("exports");

    fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let candidate = PathBuf::from(path);

    if !candidate.is_absolute() {
        return Err("Output path must be absolute.".to_string());
    }

    let Some(parent) = candidate.parent() else {
        return Err("Output path must include a parent directory.".to_string());
    };

    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let parent = parent.canonicalize().map_err(|error| error.to_string())?;

    if !parent.starts_with(&root) {
        return Err(
            "Output path must stay inside the Kaspa Gateway exports directory.".to_string(),
        );
    }

    Ok(())
}

fn normalize_report_type(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "addresses" | "address" => Ok("Addresses".to_string()),
        "transactions" | "transaction" | "tx" => Ok("Transactions".to_string()),
        "analysis" => Ok("Analysis".to_string()),
        "full" | "summary" | "fullsummary" | "full_summary" => Ok("Full".to_string()),
        "explorertransactions" | "explorer_transactions" | "explorer" => {
            Ok("ExplorerTransactions".to_string())
        }
        "topaddresses" | "top_addresses" | "top" => Ok("TopAddresses".to_string()),
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
        let valid_path = default_user_data_dir()
            .expect("user data directory")
            .join("exports")
            .join("report.csv");

        assert!(validate_output_path(valid_path.to_string_lossy().as_ref()).is_ok());
        assert!(validate_output_path("C:\\temp\\report.csv").is_err());
        assert!(validate_output_path("bad && whoami").is_err());
    }

    #[test]
    fn csv_cell_blocks_formula_injection() {
        assert_eq!(csv_cell("=cmd"), "\"'=cmd\"");
    }
}
