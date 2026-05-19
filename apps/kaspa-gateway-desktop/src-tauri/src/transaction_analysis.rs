use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths, TransactionRecord};
use serde::Serialize;
use std::collections::BTreeMap;

const MAX_ANALYSIS_ROWS_PER_ADDRESS: usize = 100_000;
const MAX_RICH_LIST_LIMIT: usize = 1_000;

#[derive(Debug, Clone, Serialize)]
pub struct AddressFlowSummary {
    pub address: String,
    pub total_transactions: i64,
    pub incoming_count: i64,
    pub outgoing_count: i64,
    pub incoming_sompi: i64,
    pub outgoing_sompi: i64,
    pub net_sompi: i64,
    pub incoming_kas: f64,
    pub outgoing_kas: f64,
    pub net_kas: f64,
    pub largest_incoming_sompi: i64,
    pub largest_outgoing_sompi: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RichListEntry {
    pub rank: usize,
    pub address: String,
    pub total_transactions: i64,
    pub net_sompi: i64,
    pub net_kas: f64,
    pub incoming_sompi: i64,
    pub outgoing_sompi: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalTransactionAnalysis {
    pub total_addresses: usize,
    pub total_transactions: i64,
    pub total_incoming_sompi: i64,
    pub total_outgoing_sompi: i64,
    pub total_net_sompi: i64,
    pub total_incoming_kas: f64,
    pub total_outgoing_kas: f64,
    pub total_net_kas: f64,
    pub top_addresses: Vec<RichListEntry>,
}

#[tauri::command]
pub fn analyze_address_flow(address: String) -> Result<AddressFlowSummary, String> {
    let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;

    let manager = database_manager()?;
    let repo = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let records = repo
        .list_for_address(parsed.as_str(), MAX_ANALYSIS_ROWS_PER_ADDRESS)
        .map_err(|error| error.to_string())?;

    Ok(summarize_address_flow(parsed.as_str(), records))
}

#[tauri::command]
pub fn analyze_all_transactions(limit: usize) -> Result<GlobalTransactionAnalysis, String> {
    let manager = database_manager()?;

    let addresses_repo = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let tx_repo = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let addresses = addresses_repo.list().map_err(|error| error.to_string())?;

    let mut total_transactions = 0_i64;
    let mut total_incoming_sompi = 0_i64;
    let mut total_outgoing_sompi = 0_i64;
    let mut entries = Vec::new();

    for address_record in &addresses {
        let parsed = match KaspaAddress::parse(&address_record.address) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        let records = tx_repo
            .list_for_address(parsed.as_str(), MAX_ANALYSIS_ROWS_PER_ADDRESS)
            .map_err(|error| error.to_string())?;

        let summary = summarize_address_flow(parsed.as_str(), records);

        total_transactions = total_transactions.saturating_add(summary.total_transactions);
        total_incoming_sompi = total_incoming_sompi.saturating_add(summary.incoming_sompi);
        total_outgoing_sompi = total_outgoing_sompi.saturating_add(summary.outgoing_sompi);

        entries.push(RichListEntry {
            rank: 0,
            address: summary.address,
            total_transactions: summary.total_transactions,
            net_sompi: summary.net_sompi,
            net_kas: summary.net_kas,
            incoming_sompi: summary.incoming_sompi,
            outgoing_sompi: summary.outgoing_sompi,
        });
    }

    sort_and_rank(&mut entries);

    let top_addresses = entries
        .into_iter()
        .take(limit.clamp(1, MAX_RICH_LIST_LIMIT))
        .collect::<Vec<_>>();

    let total_net_sompi = total_incoming_sompi.saturating_sub(total_outgoing_sompi);

    Ok(GlobalTransactionAnalysis {
        total_addresses: addresses.len(),
        total_transactions,
        total_incoming_sompi,
        total_outgoing_sompi,
        total_net_sompi,
        total_incoming_kas: sompi_to_kas(total_incoming_sompi),
        total_outgoing_kas: sompi_to_kas(total_outgoing_sompi),
        total_net_kas: sompi_to_kas(total_net_sompi),
        top_addresses,
    })
}

#[tauri::command]
pub fn local_rich_list(query: Option<String>, limit: usize) -> Result<Vec<RichListEntry>, String> {
    let analysis = analyze_all_transactions(MAX_RICH_LIST_LIMIT)?;
    let query = query.unwrap_or_default().trim().to_ascii_lowercase();

    let mut rows = analysis.top_addresses;

    if !query.is_empty() {
        rows.retain(|entry| {
            entry.address.to_ascii_lowercase().contains(&query)
                || entry.total_transactions.to_string().contains(&query)
                || entry.net_sompi.to_string().contains(&query)
                || format!("{:.8}", entry.net_kas).contains(&query)
        });
    }

    rows.truncate(limit.clamp(1, MAX_RICH_LIST_LIMIT));
    rerank(&mut rows);

    Ok(rows)
}

#[tauri::command]
pub fn address_direction_breakdown(address: String) -> Result<BTreeMap<String, i64>, String> {
    let summary = analyze_address_flow(address)?;

    let mut map = BTreeMap::new();
    map.insert("incoming_count".to_string(), summary.incoming_count);
    map.insert("outgoing_count".to_string(), summary.outgoing_count);
    map.insert("total_transactions".to_string(), summary.total_transactions);
    map.insert("incoming_sompi".to_string(), summary.incoming_sompi);
    map.insert("outgoing_sompi".to_string(), summary.outgoing_sompi);
    map.insert("net_sompi".to_string(), summary.net_sompi);
    map.insert(
        "largest_incoming_sompi".to_string(),
        summary.largest_incoming_sompi,
    );
    map.insert(
        "largest_outgoing_sompi".to_string(),
        summary.largest_outgoing_sompi,
    );

    Ok(map)
}

fn summarize_address_flow(address: &str, records: Vec<TransactionRecord>) -> AddressFlowSummary {
    let mut total_transactions = 0_i64;
    let mut incoming_count = 0_i64;
    let mut outgoing_count = 0_i64;
    let mut incoming_sompi = 0_i64;
    let mut outgoing_sompi = 0_i64;
    let mut largest_incoming_sompi = 0_i64;
    let mut largest_outgoing_sompi = 0_i64;

    for record in records {
        total_transactions = total_transactions.saturating_add(1);
        let amount = record.amount_sompi.max(0);

        match normalize_direction(&record.direction).as_str() {
            "incoming" => {
                incoming_count = incoming_count.saturating_add(1);
                incoming_sompi = incoming_sompi.saturating_add(amount);
                largest_incoming_sompi = largest_incoming_sompi.max(amount);
            }
            "outgoing" => {
                outgoing_count = outgoing_count.saturating_add(1);
                outgoing_sompi = outgoing_sompi.saturating_add(amount);
                largest_outgoing_sompi = largest_outgoing_sompi.max(amount);
            }
            "self" => {
                incoming_count = incoming_count.saturating_add(1);
                outgoing_count = outgoing_count.saturating_add(1);
                incoming_sompi = incoming_sompi.saturating_add(amount);
                outgoing_sompi = outgoing_sompi.saturating_add(amount);
                largest_incoming_sompi = largest_incoming_sompi.max(amount);
                largest_outgoing_sompi = largest_outgoing_sompi.max(amount);
            }
            _ => {}
        }
    }

    let net_sompi = incoming_sompi.saturating_sub(outgoing_sompi);

    AddressFlowSummary {
        address: address.to_string(),
        total_transactions,
        incoming_count,
        outgoing_count,
        incoming_sompi,
        outgoing_sompi,
        net_sompi,
        incoming_kas: sompi_to_kas(incoming_sompi),
        outgoing_kas: sompi_to_kas(outgoing_sompi),
        net_kas: sompi_to_kas(net_sompi),
        largest_incoming_sompi,
        largest_outgoing_sompi,
    }
}

fn sort_and_rank(entries: &mut [RichListEntry]) {
    entries.sort_by(|left, right| {
        right
            .net_sompi
            .cmp(&left.net_sompi)
            .then_with(|| right.total_transactions.cmp(&left.total_transactions))
            .then_with(|| left.address.cmp(&right.address))
    });

    rerank(entries);
}

fn rerank(entries: &mut [RichListEntry]) {
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.rank = index + 1;
    }
}

fn normalize_direction(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "incoming" | "in" | "receive" | "received" => "incoming".to_string(),
        "outgoing" | "out" | "send" | "sent" => "outgoing".to_string(),
        "self" => "self".to_string(),
        _ => "unknown".to_string(),
    }
}

fn sompi_to_kas(value: i64) -> f64 {
    value as f64 / 100_000_000.0
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
    fn direction_is_normalized() {
        assert_eq!(normalize_direction("in"), "incoming");
        assert_eq!(normalize_direction("sent"), "outgoing");
        assert_eq!(normalize_direction("other"), "unknown");
    }

    #[test]
    fn kas_conversion_matches_sompi_units() {
        assert_eq!(sompi_to_kas(100_000_000), 1.0);
    }

    #[test]
    fn ranking_orders_by_net_sompi() {
        let mut entries = vec![
            RichListEntry {
                rank: 0,
                address: "kaspa:qb".to_string(),
                total_transactions: 1,
                net_sompi: 5,
                net_kas: sompi_to_kas(5),
                incoming_sompi: 5,
                outgoing_sompi: 0,
            },
            RichListEntry {
                rank: 0,
                address: "kaspa:qa".to_string(),
                total_transactions: 1,
                net_sompi: 10,
                net_kas: sompi_to_kas(10),
                incoming_sompi: 10,
                outgoing_sompi: 0,
            },
        ];

        sort_and_rank(&mut entries);

        assert_eq!(entries[0].address, "kaspa:qa");
        assert_eq!(entries[0].rank, 1);
    }
}
