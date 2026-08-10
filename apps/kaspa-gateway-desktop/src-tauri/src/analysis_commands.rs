use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths, TransactionRecord};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDeepRequest {
    pub address: Option<String>,
    pub time_range: String,
    pub limit: usize,
    pub include_all_saved_addresses: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisMetric {
    pub label: String,
    pub value: String,
    pub raw_sompi: Option<i64>,
    pub raw_number: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisBucket {
    pub label: String,
    pub incoming_kas: f64,
    pub outgoing_kas: f64,
    pub net_kas: f64,
    pub transactions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisCounterparty {
    pub counterparty: String,
    pub incoming_kas: f64,
    pub outgoing_kas: f64,
    pub net_kas: f64,
    pub transactions: usize,
    // KGW_ANALYSIS_BACKEND_CHILD_TRANSACTIONS_PATCH_R11C
    // Python-style parent row carries expandable child transactions.
    pub details: Vec<AnalysisTransactionRow>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisDeepReport {
    pub scope: String,
    pub time_range: String,
    pub total_transactions: usize,
    pub metrics: Vec<AnalysisMetric>,
    pub buckets: Vec<AnalysisBucket>,
    pub counterparties: Vec<AnalysisCounterparty>,
    pub rows: Vec<AnalysisTransactionRow>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisTransactionRow {
    pub txid: String,
    pub address: String,
    pub timestamp_ms: i64,
    pub tx_type: String,
    pub direction: String,
    pub amount_kas: f64,
    pub counterparty: String,
    // KGW_ANALYSIS_CHILD_COLUMNS_PYTHON_PARITY_PATCH_R16
    pub block_score: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisGraphNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub total_kas: f64,
    pub degree: usize,
    pub cluster: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisGraphEdge {
    pub source: String,
    pub target: String,
    pub direction: String,
    pub amount_kas: f64,
    pub transactions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisCluster {
    pub cluster_id: usize,
    pub nodes: usize,
    pub edges: usize,
    pub total_kas: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisGraphReport {
    pub nodes: Vec<AnalysisGraphNode>,
    pub edges: Vec<AnalysisGraphEdge>,
    pub clusters: Vec<AnalysisCluster>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisTimeRangeOption {
    pub label: String,
    pub value: String,
}

#[tauri::command]
pub fn analysis_time_range_options() -> Vec<AnalysisTimeRangeOption> {
    vec![
        option("All Time", "all"),
        option("Last 3 Days", "last_3_days"),
        option("Last Week", "last_week"),
        option("Last Month", "last_month"),
        option("Last 3 Months", "last_3_months"),
        option("Last 6 Months", "last_6_months"),
        option("Last Year", "last_year"),
    ]
}

#[tauri::command]
pub fn analysis_report(request: AnalysisDeepRequest) -> Result<AnalysisDeepReport, String> {
    validate_request(&request)?;

    let mut warnings = Vec::new();
    let mut records = load_records(&request, &mut warnings)?;

    apply_time_range_filter(&mut records, &request.time_range)?;
    records.sort_by_key(|record| record.timestamp_ms);

    if request.limit > 0 && records.len() > request.limit {
        let keep_from = records.len().saturating_sub(request.limit);
        records = records.into_iter().skip(keep_from).collect();
        warnings.push(format!(
            "Result limited to latest {} transactions.",
            request.limit
        ));
    }

    let total_transactions = records.len();

    let mut incoming_sompi = 0_i64;
    let mut outgoing_sompi = 0_i64;
    let mut incoming_count = 0_usize;
    let mut outgoing_count = 0_usize;
    let mut max_inflow = 0_i64;
    let mut max_outflow = 0_i64;
    let mut counterparties_set = BTreeSet::new();
    let mut first_tx = 0_i64;
    let mut last_tx = 0_i64;

    let mut bucket_map: BTreeMap<String, BucketAccumulator> = BTreeMap::new();
    let mut cp_map: BTreeMap<String, CounterpartyAccumulator> = BTreeMap::new();

    for record in &records {
        if first_tx == 0 || record.timestamp_ms < first_tx {
            first_tx = record.timestamp_ms;
        }

        if record.timestamp_ms > last_tx {
            last_tx = record.timestamp_ms;
        }

        let amount = record.amount_sompi.max(0);
        let counterparty = clean_counterparty(record.counterparty.as_deref());

        counterparties_set.insert(counterparty.clone());

        let bucket_key = day_bucket_label(record.timestamp_ms);
        let bucket = bucket_map.entry(bucket_key).or_default();
        bucket.transactions += 1;

        let cp = cp_map.entry(counterparty.clone()).or_default();
        cp.transactions += 1;
        // KGW_ANALYSIS_BACKEND_CHILD_TRANSACTIONS_PATCH_R11C
        cp.details.push(AnalysisTransactionRow {
            txid: record.txid.clone(),
            address: record.address.clone(),
            timestamp_ms: record.timestamp_ms,
            tx_type: record.tx_type.clone(),
            direction: record.direction.clone(),
            amount_kas: sompi_to_kas(record.amount_sompi),
            counterparty: record
                .counterparty
                .clone()
                .unwrap_or_else(|| counterparty.clone()),
            // KGW_ANALYSIS_CHILD_COLUMNS_PYTHON_PARITY_PATCH_R16
            block_score: record
                .block_height
                .map(|value| value.to_string())
                .unwrap_or_default(),
        });

        match record.direction.to_ascii_lowercase().as_str() {
            "incoming" => {
                incoming_sompi = incoming_sompi.saturating_add(amount);
                incoming_count += 1;
                max_inflow = max_inflow.max(amount);
                bucket.incoming_sompi = bucket.incoming_sompi.saturating_add(amount);
                cp.incoming_sompi = cp.incoming_sompi.saturating_add(amount);
            }
            "outgoing" => {
                outgoing_sompi = outgoing_sompi.saturating_add(amount);
                outgoing_count += 1;
                max_outflow = max_outflow.max(amount);
                bucket.outgoing_sompi = bucket.outgoing_sompi.saturating_add(amount);
                cp.outgoing_sompi = cp.outgoing_sompi.saturating_add(amount);
            }
            "self" => {
                bucket.incoming_sompi = bucket.incoming_sompi.saturating_add(amount);
                bucket.outgoing_sompi = bucket.outgoing_sompi.saturating_add(amount);
                cp.incoming_sompi = cp.incoming_sompi.saturating_add(amount);
                cp.outgoing_sompi = cp.outgoing_sompi.saturating_add(amount);
            }
            _ => {}
        }
    }

    let net_sompi = incoming_sompi.saturating_sub(outgoing_sompi);

    let avg_inflow = if incoming_count > 0 {
        incoming_sompi as f64 / incoming_count as f64
    } else {
        0.0
    };

    let avg_outflow = if outgoing_count > 0 {
        outgoing_sompi as f64 / outgoing_count as f64
    } else {
        0.0
    };

    let duration_days = if first_tx > 0 && last_tx >= first_tx {
        ((last_tx - first_tx) / 86_400_000) + 1
    } else {
        0
    };

    let metrics = vec![
        metric_sompi("Total Inflow (KAS)", incoming_sompi),
        metric_sompi("Total Outflow (KAS)", outgoing_sompi),
        metric_sompi("Net Flow (KAS)", net_sompi),
        metric_float("Avg Inflow (KAS)", sompi_to_kas_f64(avg_inflow)),
        metric_float("Avg Outflow (KAS)", sompi_to_kas_f64(avg_outflow)),
        metric_number("Total Transactions", total_transactions as f64),
        metric_sompi("Largest Inflow (KAS)", max_inflow),
        metric_sompi("Largest Outflow (KAS)", max_outflow),
        metric_number("Unique Counterparties", counterparties_set.len() as f64),
        metric_number("Duration (Days)", duration_days as f64),
        metric_text("First Transaction", format_timestamp_label(first_tx)),
        metric_text("Last Transaction", format_timestamp_label(last_tx)),
    ];

    let buckets = bucket_map
        .into_iter()
        .map(|(label, bucket)| AnalysisBucket {
            label,
            incoming_kas: sompi_to_kas(bucket.incoming_sompi),
            outgoing_kas: sompi_to_kas(bucket.outgoing_sompi),
            net_kas: sompi_to_kas(bucket.incoming_sompi.saturating_sub(bucket.outgoing_sompi)),
            transactions: bucket.transactions,
        })
        .collect::<Vec<_>>();

    let mut counterparties = cp_map
        .into_iter()
        .map(|(counterparty, cp)| AnalysisCounterparty {
            counterparty,
            incoming_kas: sompi_to_kas(cp.incoming_sompi),
            outgoing_kas: sompi_to_kas(cp.outgoing_sompi),
            net_kas: sompi_to_kas(cp.incoming_sompi.saturating_sub(cp.outgoing_sompi)),
            transactions: cp.transactions,
            // KGW_ANALYSIS_BACKEND_CHILD_TRANSACTIONS_PATCH_R11C
            details: cp.details,
        })
        .collect::<Vec<_>>();

    counterparties.sort_by(|left, right| {
        right.transactions.cmp(&left.transactions).then_with(|| {
            right
                .net_kas
                .partial_cmp(&left.net_kas)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });
    counterparties.truncate(100);

    let rows = records
        .into_iter()
        .rev()
        .take(500)
        .map(|record| AnalysisTransactionRow {
            txid: record.txid,
            address: record.address,
            timestamp_ms: record.timestamp_ms,
            tx_type: record.tx_type,
            direction: record.direction,
            amount_kas: sompi_to_kas(record.amount_sompi),
            counterparty: record.counterparty.unwrap_or_default(),
            // KGW_ANALYSIS_CHILD_COLUMNS_PYTHON_PARITY_PATCH_R16
            block_score: record
                .block_height
                .map(|value| value.to_string())
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    Ok(AnalysisDeepReport {
        scope: if request.include_all_saved_addresses {
            "all_saved_addresses".to_string()
        } else {
            request
                .address
                .clone()
                .unwrap_or_else(|| "selected_address".to_string())
        },
        time_range: request.time_range,
        total_transactions,
        metrics,
        buckets,
        counterparties,
        rows,
        warnings,
    })
}

#[tauri::command]
pub fn analysis_graph_report(request: AnalysisDeepRequest) -> Result<AnalysisGraphReport, String> {
    validate_request(&request)?;

    let mut warnings = Vec::new();
    let mut records = load_records(&request, &mut warnings)?;

    apply_time_range_filter(&mut records, &request.time_range)?;

    if request.limit > 0 && records.len() > request.limit {
        let keep_from = records.len().saturating_sub(request.limit);
        records = records.into_iter().skip(keep_from).collect();
    }

    let mut node_totals: BTreeMap<String, i64> = BTreeMap::new();
    let mut edge_map: BTreeMap<(String, String, String), EdgeAccumulator> = BTreeMap::new();

    for record in &records {
        let counterparty = clean_counterparty(record.counterparty.as_deref());
        let amount = record.amount_sompi.max(0);

        let (source, target, direction) = match record.direction.to_ascii_lowercase().as_str() {
            "incoming" => (
                counterparty.clone(),
                record.address.clone(),
                "incoming".to_string(),
            ),
            "outgoing" => (
                record.address.clone(),
                counterparty.clone(),
                "outgoing".to_string(),
            ),
            "self" => (
                record.address.clone(),
                record.address.clone(),
                "self".to_string(),
            ),
            _ => (
                record.address.clone(),
                counterparty.clone(),
                "unknown".to_string(),
            ),
        };

        *node_totals.entry(source.clone()).or_insert(0) = node_totals
            .get(&source)
            .copied()
            .unwrap_or(0)
            .saturating_add(amount);

        *node_totals.entry(target.clone()).or_insert(0) = node_totals
            .get(&target)
            .copied()
            .unwrap_or(0)
            .saturating_add(amount);

        let edge = edge_map.entry((source, target, direction)).or_default();
        edge.amount_sompi = edge.amount_sompi.saturating_add(amount);
        edge.transactions += 1;
    }

    let edges = edge_map
        .iter()
        .map(|((source, target, direction), edge)| AnalysisGraphEdge {
            source: source.clone(),
            target: target.clone(),
            direction: direction.clone(),
            amount_kas: sompi_to_kas(edge.amount_sompi),
            transactions: edge.transactions,
        })
        .collect::<Vec<_>>();

    let cluster_ids = compute_clusters(&node_totals, &edges);
    let mut degree_map: BTreeMap<String, usize> = BTreeMap::new();

    for edge in &edges {
        *degree_map.entry(edge.source.clone()).or_insert(0) += 1;
        *degree_map.entry(edge.target.clone()).or_insert(0) += 1;
    }

    let nodes = node_totals
        .iter()
        .map(|(id, total_sompi)| AnalysisGraphNode {
            id: id.clone(),
            label: shorten_label(id),
            kind: if is_kaspa_address_like(id) {
                "address".to_string()
            } else {
                "counterparty".to_string()
            },
            total_kas: sompi_to_kas(*total_sompi),
            degree: degree_map.get(id).copied().unwrap_or(0),
            cluster: cluster_ids.get(id).copied().unwrap_or(0),
        })
        .collect::<Vec<_>>();

    let mut cluster_map: BTreeMap<usize, AnalysisCluster> = BTreeMap::new();

    for node in &nodes {
        let cluster = cluster_map.entry(node.cluster).or_insert(AnalysisCluster {
            cluster_id: node.cluster,
            nodes: 0,
            edges: 0,
            total_kas: 0.0,
        });

        cluster.nodes += 1;
        cluster.total_kas += node.total_kas;
    }

    for edge in &edges {
        if let Some(cluster_id) = cluster_ids.get(&edge.source)
            && let Some(cluster) = cluster_map.get_mut(cluster_id)
        {
            cluster.edges += 1;
        }
    }

    let mut clusters = cluster_map.into_values().collect::<Vec<_>>();
    clusters.sort_by(|left, right| {
        right
            .total_kas
            .partial_cmp(&left.total_kas)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if nodes.is_empty() {
        warnings.push("No graph data was available for the selected range.".to_string());
    }

    Ok(AnalysisGraphReport {
        nodes,
        edges,
        clusters,
        warnings,
    })
}

fn load_records(
    request: &AnalysisDeepRequest,
    warnings: &mut Vec<String>,
) -> Result<Vec<TransactionRecord>, String> {
    let manager = database_manager()?;
    let tx_repo = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    if request.include_all_saved_addresses {
        let address_repo = manager
            .addresses_repository()
            .map_err(|error| error.to_string())?;

        let addresses = address_repo.list().map_err(|error| error.to_string())?;
        let mut records = Vec::new();

        for address in addresses {
            let mut rows = tx_repo
                .list_for_address(&address.address, 50_000_usize)
                .map_err(|error| error.to_string())?;

            records.append(&mut rows);
        }

        if records.is_empty() {
            warnings.push("No cached transactions found for saved addresses.".to_string());
        }

        Ok(records)
    } else {
        let address = request.address.clone().ok_or_else(|| {
            "Address is required unless all saved addresses is enabled.".to_string()
        })?;

        let parsed = KaspaAddress::parse(&address).map_err(|error| error.to_string())?;

        let records = tx_repo
            .list_for_address(parsed.as_str(), 50_000_usize)
            .map_err(|error| error.to_string())?;

        if records.is_empty() {
            warnings.push(
                "No cached transactions found for this address. Fetch transactions first in Explorer."
                    .to_string(),
            );
        }

        Ok(records)
    }
}

fn validate_request(request: &AnalysisDeepRequest) -> Result<(), String> {
    if request.limit > 500_000 {
        return Err("limit cannot exceed 500000.".to_string());
    }

    if let Some(address) = &request.address {
        KaspaAddress::parse(address).map_err(|error| error.to_string())?;
    }

    time_range_start_ms(&request.time_range)?;

    Ok(())
}

fn apply_time_range_filter(
    records: &mut Vec<TransactionRecord>,
    time_range: &str,
) -> Result<(), String> {
    if let Some(start_ms) = time_range_start_ms(time_range)? {
        records.retain(|record| record.timestamp_ms >= start_ms);
    }

    Ok(())
}

fn time_range_start_ms(value: &str) -> Result<Option<i64>, String> {
    let now = now_ms();

    let days: i64 = match value {
        "all" | "" => return Ok(None),
        "last_3_days" => 3,
        "last_week" => 7,
        "last_month" => 30,
        "last_3_months" => 90,
        "last_6_months" => 180,
        "last_year" => 365,
        _ => return Err("Unsupported analysis time range.".to_string()),
    };

    Ok(Some(now.saturating_sub(days.saturating_mul(86_400_000))))
}

fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

fn day_bucket_label(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        return "Unknown".to_string();
    }

    let day = timestamp_ms / 86_400_000;
    format!("Day {day}")
}

fn format_timestamp_label(timestamp_ms: i64) -> String {
    if timestamp_ms <= 0 {
        "N/A".to_string()
    } else {
        timestamp_ms.to_string()
    }
}

fn metric_sompi(label: &str, value: i64) -> AnalysisMetric {
    AnalysisMetric {
        label: label.to_string(),
        value: format!("{:.8}", sompi_to_kas(value)),
        raw_sompi: Some(value),
        raw_number: Some(sompi_to_kas(value)),
    }
}

fn metric_float(label: &str, value: f64) -> AnalysisMetric {
    AnalysisMetric {
        label: label.to_string(),
        value: format!("{value:.8}"),
        raw_sompi: None,
        raw_number: Some(value),
    }
}

fn metric_number(label: &str, value: f64) -> AnalysisMetric {
    AnalysisMetric {
        label: label.to_string(),
        value: format!("{value:.0}"),
        raw_sompi: None,
        raw_number: Some(value),
    }
}

fn metric_text(label: &str, value: String) -> AnalysisMetric {
    AnalysisMetric {
        label: label.to_string(),
        value,
        raw_sompi: None,
        raw_number: None,
    }
}

fn sompi_to_kas(value: i64) -> f64 {
    value as f64 / 100_000_000.0
}

fn sompi_to_kas_f64(value: f64) -> f64 {
    value / 100_000_000.0
}

fn clean_counterparty(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Unknown")
        .to_string()
}

fn is_kaspa_address_like(value: &str) -> bool {
    value.starts_with("kaspa:")
        || value.starts_with("kaspatest:")
        || value.starts_with("kaspadev:")
        || value.starts_with("kaspasim:")
}

fn shorten_label(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();

    if chars.len() <= 18 {
        return value.to_string();
    }

    let start = chars.iter().take(10).collect::<String>();
    let end = chars
        .iter()
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();

    format!("{start}...{end}")
}

fn compute_clusters(
    nodes: &BTreeMap<String, i64>,
    edges: &[AnalysisGraphEdge],
) -> BTreeMap<String, usize> {
    let mut adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for node in nodes.keys() {
        adjacency.entry(node.clone()).or_default();
    }

    for edge in edges {
        adjacency
            .entry(edge.source.clone())
            .or_default()
            .insert(edge.target.clone());

        adjacency
            .entry(edge.target.clone())
            .or_default()
            .insert(edge.source.clone());
    }

    let mut visited = BTreeSet::new();
    let mut cluster_ids = BTreeMap::new();
    let mut cluster_id = 0_usize;

    for node in nodes.keys() {
        if visited.contains(node) {
            continue;
        }

        cluster_id += 1;
        let mut queue = VecDeque::new();

        queue.push_back(node.clone());
        visited.insert(node.clone());

        while let Some(current) = queue.pop_front() {
            cluster_ids.insert(current.clone(), cluster_id);

            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if visited.insert(neighbor.clone()) {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    cluster_ids
}

fn option(label: &str, value: &str) -> AnalysisTimeRangeOption {
    AnalysisTimeRangeOption {
        label: label.to_string(),
        value: value.to_string(),
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

#[derive(Debug, Default)]
struct BucketAccumulator {
    incoming_sompi: i64,
    outgoing_sompi: i64,
    transactions: usize,
}

#[derive(Debug, Default)]
struct CounterpartyAccumulator {
    incoming_sompi: i64,
    outgoing_sompi: i64,
    transactions: usize,
    // KGW_ANALYSIS_BACKEND_CHILD_TRANSACTIONS_PATCH_R11C
    details: Vec<AnalysisTransactionRow>,
}

#[derive(Debug, Default)]
struct EdgeAccumulator {
    amount_sompi: i64,
    transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_range_options_include_all() {
        let options = analysis_time_range_options();
        assert!(options.iter().any(|option| option.value == "all"));
    }

    #[test]
    fn unknown_time_range_is_rejected() {
        assert!(time_range_start_ms("bad_range").is_err());
    }

    #[test]
    fn graph_clusters_connect_nodes() {
        let mut nodes = BTreeMap::new();
        nodes.insert("a".to_string(), 1);
        nodes.insert("b".to_string(), 1);

        let edges = vec![AnalysisGraphEdge {
            source: "a".to_string(),
            target: "b".to_string(),
            direction: "incoming".to_string(),
            amount_kas: 1.0,
            transactions: 1,
        }];

        let clusters = compute_clusters(&nodes, &edges);

        assert_eq!(clusters.get("a"), clusters.get("b"));
    }
}
