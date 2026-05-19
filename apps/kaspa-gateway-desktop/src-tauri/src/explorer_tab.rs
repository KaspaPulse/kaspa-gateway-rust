use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths, TransactionRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerTransactionsPageRequest {
    pub address: String,
    pub limit: usize,
    pub offset: usize,
    pub direction_filter: Option<String>,
    pub type_filter: Option<String>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplorerTransactionRow {
    pub txid: String,
    pub address: String,
    pub timestamp_ms: i64,
    pub tx_type: String,
    pub direction: String,
    pub amount_sompi: i64,
    pub amount_kas: f64,
    pub counterparty: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplorerTransactionsPage {
    pub address: String,
    pub total_filtered: usize,
    pub offset: usize,
    pub limit: usize,
    pub page: usize,
    pub total_pages: usize,
    pub rows: Vec<ExplorerTransactionRow>,
}

#[tauri::command]
pub fn explorer_transactions_page(
    request: ExplorerTransactionsPageRequest,
) -> Result<ExplorerTransactionsPage, String> {
    let parsed = KaspaAddress::parse(&request.address).map_err(|error| error.to_string())?;

    if request.limit == 0 || request.limit > 500 {
        return Err("limit must be between 1 and 500.".to_string());
    }

    let manager = database_manager()?;
    let repository = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let records = repository
        .list_for_address(parsed.as_str(), 50_000)
        .map_err(|error| error.to_string())?;

    let direction_filter = request
        .direction_filter
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    let type_filter = request
        .type_filter
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    let query = request
        .query
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    let filtered = records
        .into_iter()
        .filter(|record| {
            if !direction_filter.is_empty()
                && direction_filter != "all"
                && record.direction.to_ascii_lowercase() != direction_filter
            {
                return false;
            }

            if !type_filter.is_empty()
                && type_filter != "all"
                && record.tx_type.to_ascii_lowercase() != type_filter
            {
                return false;
            }

            if !query.is_empty() {
                let haystack = format!(
                    "{} {} {} {}",
                    record.txid,
                    record.address,
                    record.tx_type,
                    record.counterparty.clone().unwrap_or_default()
                )
                .to_ascii_lowercase();

                if !haystack.contains(&query) {
                    return false;
                }
            }

            true
        })
        .collect::<Vec<_>>();

    let total_filtered = filtered.len();
    let total_pages = total_filtered.div_ceil(request.limit).max(1);
    let safe_offset = request.offset.min(total_filtered);
    let page = (safe_offset / request.limit) + 1;

    let rows = filtered
        .into_iter()
        .skip(safe_offset)
        .take(request.limit)
        .map(to_row)
        .collect::<Vec<_>>();

    Ok(ExplorerTransactionsPage {
        address: parsed.as_str().to_string(),
        total_filtered,
        offset: safe_offset,
        limit: request.limit,
        page,
        total_pages,
        rows,
    })
}

fn to_row(record: TransactionRecord) -> ExplorerTransactionRow {
    ExplorerTransactionRow {
        txid: record.txid,
        address: record.address,
        timestamp_ms: record.timestamp_ms,
        tx_type: record.tx_type,
        direction: record.direction,
        amount_sompi: record.amount_sompi,
        amount_kas: record.amount_sompi as f64 / 100_000_000.0,
        counterparty: record.counterparty.unwrap_or_default(),
    }
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
