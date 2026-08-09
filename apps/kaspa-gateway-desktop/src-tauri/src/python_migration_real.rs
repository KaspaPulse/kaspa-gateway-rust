use duckdb::{Connection, params};
use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{AddressRecord, DatabaseManager, DatabasePaths, TransactionRecord};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealMigrationRequest {
    pub source_path: String,
    pub import_addresses: bool,
    pub import_transactions: bool,
    pub import_settings: bool,
    pub max_rows_per_table: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealMigrationColumnMap {
    pub table_name: String,
    pub role: String,
    pub columns: BTreeMap<String, String>,
    pub confidence: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealMigrationPreview {
    pub source_path: String,
    pub discovered_databases: Vec<String>,
    pub maps: Vec<RealMigrationColumnMap>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealMigrationRunReport {
    pub source_path: String,
    pub addresses_imported: usize,
    pub transactions_imported: usize,
    pub settings_imported: usize,
    pub skipped_rows: usize,
    pub warnings: Vec<String>,
    pub maps: Vec<RealMigrationColumnMap>,
}

#[tauri::command]
pub fn real_python_migration_preview(
    request: RealMigrationRequest,
) -> Result<RealMigrationPreview, String> {
    validate_request(&request)?;

    let source = normalize_source_path(&request.source_path)?;
    let databases = discover_duckdb_files(&source)?;

    let mut maps = Vec::new();
    let mut warnings = Vec::new();

    for db_path in &databases {
        match discover_database_maps(db_path, request.max_rows_per_table) {
            Ok(mut found) => maps.append(&mut found),
            Err(error) => warnings.push(format!("{}: {}", db_path.display(), error)),
        }
    }

    Ok(RealMigrationPreview {
        source_path: source.display().to_string(),
        discovered_databases: databases
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        maps,
        warnings,
    })
}

#[tauri::command]
pub fn real_python_migration_run(
    request: RealMigrationRequest,
) -> Result<RealMigrationRunReport, String> {
    validate_request(&request)?;

    let preview = real_python_migration_preview(request.clone())?;
    let manager = database_manager()?;

    let mut addresses_imported = 0_usize;
    let mut transactions_imported = 0_usize;
    let mut settings_imported = 0_usize;
    let mut skipped_rows = 0_usize;
    let mut warnings = preview.warnings.clone();

    let databases = preview
        .discovered_databases
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    for db_path in &databases {
        let connection = match Connection::open(db_path) {
            Ok(connection) => connection,
            Err(error) => {
                warnings.push(format!("Failed to open {}: {}", db_path.display(), error));
                continue;
            }
        };

        let maps = match discover_database_maps(db_path, request.max_rows_per_table) {
            Ok(maps) => maps,
            Err(error) => {
                warnings.push(format!(
                    "Failed to discover maps for {}: {}",
                    db_path.display(),
                    error
                ));
                continue;
            }
        };

        for map in maps {
            match map.role.as_str() {
                "addresses" if request.import_addresses => {
                    match import_address_table(
                        &connection,
                        &map,
                        &manager,
                        request.max_rows_per_table,
                    ) {
                        Ok(result) => {
                            addresses_imported += result.imported;
                            skipped_rows += result.skipped;
                            warnings.extend(result.warnings);
                        }
                        Err(error) => warnings.push(format!(
                            "Address import failed for {}: {}",
                            map.table_name, error
                        )),
                    }
                }
                "transactions" if request.import_transactions => {
                    match import_transaction_table(
                        &connection,
                        &map,
                        &manager,
                        request.max_rows_per_table,
                    ) {
                        Ok(result) => {
                            transactions_imported += result.imported;
                            skipped_rows += result.skipped;
                            warnings.extend(result.warnings);
                        }
                        Err(error) => warnings.push(format!(
                            "Transaction import failed for {}: {}",
                            map.table_name, error
                        )),
                    }
                }
                "settings" if request.import_settings => {
                    match import_settings_table(
                        &connection,
                        &map,
                        &manager,
                        request.max_rows_per_table,
                    ) {
                        Ok(result) => {
                            settings_imported += result.imported;
                            skipped_rows += result.skipped;
                            warnings.extend(result.warnings);
                        }
                        Err(error) => warnings.push(format!(
                            "Settings import failed for {}: {}",
                            map.table_name, error
                        )),
                    }
                }
                _ => {}
            }
        }
    }

    Ok(RealMigrationRunReport {
        source_path: preview.source_path,
        addresses_imported,
        transactions_imported,
        settings_imported,
        skipped_rows,
        warnings,
        maps: preview.maps,
    })
}

#[derive(Debug, Default)]
struct ImportTableResult {
    imported: usize,
    skipped: usize,
    warnings: Vec<String>,
}

fn discover_database_maps(
    path: &Path,
    max_rows: usize,
) -> Result<Vec<RealMigrationColumnMap>, String> {
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    let tables = list_tables(&connection)?;
    let mut maps = Vec::new();

    for table in tables {
        let columns = list_columns(&connection, &table)?;
        let row_count = count_table_rows(&connection, &table)?.min(max_rows);

        if let Some(map) = map_address_table(&table, &columns, row_count) {
            maps.push(map);
            continue;
        }

        if let Some(map) = map_transaction_table(&table, &columns, row_count) {
            maps.push(map);
            continue;
        }

        if let Some(map) = map_settings_table(&table, &columns, row_count) {
            maps.push(map);
        }
    }

    Ok(maps)
}

fn map_address_table(
    table: &str,
    columns: &[String],
    row_count: usize,
) -> Option<RealMigrationColumnMap> {
    let address = find_column(columns, ADDRESS_COLUMNS)?;

    let mut map = BTreeMap::new();
    map.insert("address".to_string(), address);
    insert_if_found(&mut map, "name", columns, NAME_COLUMNS);
    insert_if_found(&mut map, "network", columns, NETWORK_COLUMNS);

    Some(RealMigrationColumnMap {
        table_name: table.to_string(),
        role: "addresses".to_string(),
        columns: map,
        confidence: "high".to_string(),
        row_count,
    })
}

fn map_transaction_table(
    table: &str,
    columns: &[String],
    row_count: usize,
) -> Option<RealMigrationColumnMap> {
    let txid = find_column(columns, TXID_COLUMNS)?;
    let address = find_column(columns, ADDRESS_COLUMNS)?;

    let mut map = BTreeMap::new();
    map.insert("txid".to_string(), txid);
    map.insert("address".to_string(), address);
    insert_if_found(&mut map, "amount", columns, AMOUNT_COLUMNS);
    insert_if_found(&mut map, "timestamp", columns, TIMESTAMP_COLUMNS);
    insert_if_found(&mut map, "direction", columns, DIRECTION_COLUMNS);
    insert_if_found(&mut map, "tx_type", columns, TX_TYPE_COLUMNS);
    insert_if_found(&mut map, "counterparty", columns, COUNTERPARTY_COLUMNS);
    insert_if_found(&mut map, "raw_json", columns, RAW_JSON_COLUMNS);

    Some(RealMigrationColumnMap {
        table_name: table.to_string(),
        role: "transactions".to_string(),
        columns: map,
        confidence: "high".to_string(),
        row_count,
    })
}

fn map_settings_table(
    table: &str,
    columns: &[String],
    row_count: usize,
) -> Option<RealMigrationColumnMap> {
    let key = find_column(columns, SETTING_KEY_COLUMNS)?;
    let value = find_column(columns, SETTING_VALUE_COLUMNS)?;

    let mut map = BTreeMap::new();
    map.insert("key".to_string(), key);
    map.insert("value".to_string(), value);

    Some(RealMigrationColumnMap {
        table_name: table.to_string(),
        role: "settings".to_string(),
        columns: map,
        confidence: "medium".to_string(),
        row_count,
    })
}

fn import_address_table(
    connection: &Connection,
    map: &RealMigrationColumnMap,
    manager: &DatabaseManager,
    max_rows: usize,
) -> Result<ImportTableResult, String> {
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let address_col = required_mapped_column(map, "address")?;
    let name_col = map.columns.get("name").map(String::as_str);
    let network_col = map.columns.get("network").map(String::as_str);

    let select = format!(
        "SELECT {}, {}, {} FROM {} LIMIT {}",
        cast_column_sql(address_col),
        optional_cast_column_sql(name_col, "'Imported Address'"),
        optional_cast_column_sql(network_col, "'mainnet'"),
        quoted_identifier(&map.table_name),
        max_rows
    );

    let mut statement = connection
        .prepare(&select)
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            let address: Option<String> = row.get(0)?;
            let name: Option<String> = row.get(1)?;
            let network: Option<String> = row.get(2)?;
            Ok((address, name, network))
        })
        .map_err(|error| error.to_string())?;

    let mut result = ImportTableResult::default();

    for row in rows {
        let (address, name, network) = row.map_err(|error| error.to_string())?;
        let Some(address) = address.map(|value| value.trim().to_string()) else {
            result.skipped += 1;
            continue;
        };

        if KaspaAddress::parse(&address).is_err() {
            result.skipped += 1;
            continue;
        }

        let record = AddressRecord::new(
            address,
            clean_text(
                name.as_deref().unwrap_or("Imported Address"),
                "Imported Address",
            ),
            clean_network(network.as_deref().unwrap_or("mainnet")),
        )
        .map_err(|error| error.to_string())?;

        repository
            .upsert(&record)
            .map_err(|error| error.to_string())?;

        result.imported += 1;
    }

    Ok(result)
}

fn import_transaction_table(
    connection: &Connection,
    map: &RealMigrationColumnMap,
    manager: &DatabaseManager,
    max_rows: usize,
) -> Result<ImportTableResult, String> {
    let repository = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let txid_col = required_mapped_column(map, "txid")?;
    let address_col = required_mapped_column(map, "address")?;
    let amount_col = map.columns.get("amount").map(String::as_str);
    let timestamp_col = map.columns.get("timestamp").map(String::as_str);
    let direction_col = map.columns.get("direction").map(String::as_str);
    let tx_type_col = map.columns.get("tx_type").map(String::as_str);
    let counterparty_col = map.columns.get("counterparty").map(String::as_str);
    let raw_json_col = map.columns.get("raw_json").map(String::as_str);

    let select = format!(
        "SELECT {}, {}, {}, {}, {}, {}, {}, {} FROM {} LIMIT {}",
        cast_column_sql(txid_col),
        cast_column_sql(address_col),
        optional_cast_column_sql(amount_col, "'0'"),
        optional_cast_column_sql(timestamp_col, "'0'"),
        optional_cast_column_sql(direction_col, "'unknown'"),
        optional_cast_column_sql(tx_type_col, "'transfer'"),
        optional_cast_column_sql(counterparty_col, "NULL"),
        optional_cast_column_sql(raw_json_col, "NULL"),
        quoted_identifier(&map.table_name),
        max_rows
    );

    let mut statement = connection
        .prepare(&select)
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            let txid: Option<String> = row.get(0)?;
            let address: Option<String> = row.get(1)?;
            let amount: Option<String> = row.get(2)?;
            let timestamp: Option<String> = row.get(3)?;
            let direction: Option<String> = row.get(4)?;
            let tx_type: Option<String> = row.get(5)?;
            let counterparty: Option<String> = row.get(6)?;
            let raw_json: Option<String> = row.get(7)?;
            Ok((
                txid,
                address,
                amount,
                timestamp,
                direction,
                tx_type,
                counterparty,
                raw_json,
            ))
        })
        .map_err(|error| error.to_string())?;

    let mut result = ImportTableResult::default();

    for row in rows {
        let (txid, address, amount, timestamp, direction, tx_type, counterparty, raw_json) =
            row.map_err(|error| error.to_string())?;

        let Some(txid) = txid.map(|value| value.trim().to_string()) else {
            result.skipped += 1;
            continue;
        };

        let Some(address) = address.map(|value| value.trim().to_string()) else {
            result.skipped += 1;
            continue;
        };

        if txid.is_empty() || KaspaAddress::parse(&address).is_err() {
            result.skipped += 1;
            continue;
        }

        let amount_sompi = parse_i64(amount.as_deref()).unwrap_or(0).max(0);
        let direction = normalize_direction(direction.as_deref().unwrap_or("unknown"));
        let tx_type = clean_text(tx_type.as_deref().unwrap_or("transfer"), "transfer");

        let mut record = TransactionRecord::new(txid, address, tx_type, direction, amount_sompi)
            .map_err(|error| error.to_string())?;

        let timestamp_ms = normalize_timestamp_ms(parse_i64(timestamp.as_deref()).unwrap_or(0));
        if timestamp_ms > 0 {
            record.timestamp_ms = timestamp_ms;
        }

        record.counterparty = counterparty
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        record.raw_json = raw_json
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        repository
            .upsert(&record)
            .map_err(|error| error.to_string())?;

        result.imported += 1;
    }

    Ok(result)
}

fn import_settings_table(
    connection: &Connection,
    map: &RealMigrationColumnMap,
    manager: &DatabaseManager,
    max_rows: usize,
) -> Result<ImportTableResult, String> {
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let key_col = required_mapped_column(map, "key")?;
    let value_col = required_mapped_column(map, "value")?;

    let select = format!(
        "SELECT {}, {} FROM {} LIMIT {}",
        cast_column_sql(key_col),
        cast_column_sql(value_col),
        quoted_identifier(&map.table_name),
        max_rows
    );

    let mut statement = connection
        .prepare(&select)
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            let key: Option<String> = row.get(0)?;
            let value: Option<String> = row.get(1)?;
            Ok((key, value))
        })
        .map_err(|error| error.to_string())?;

    let mut result = ImportTableResult::default();

    for row in rows {
        let (key, value) = row.map_err(|error| error.to_string())?;
        let Some(key) = key.map(|value| sanitize_setting_key(&value)) else {
            result.skipped += 1;
            continue;
        };

        if key.is_empty() {
            result.skipped += 1;
            continue;
        }

        repository
            .set(&format!("python.{key}"), &value.unwrap_or_default())
            .map_err(|error| error.to_string())?;

        result.imported += 1;
    }

    Ok(result)
}

fn validate_request(request: &RealMigrationRequest) -> Result<(), String> {
    normalize_source_path(&request.source_path)?;

    if !request.import_addresses && !request.import_transactions && !request.import_settings {
        return Err("At least one import option must be enabled.".to_string());
    }

    if request.max_rows_per_table == 0 || request.max_rows_per_table > 1_000_000 {
        return Err("max_rows_per_table must be between 1 and 1000000.".to_string());
    }

    Ok(())
}

fn normalize_source_path(source_path: &str) -> Result<PathBuf, String> {
    validate_path_text(source_path)?;

    let path = PathBuf::from(source_path.trim());

    if !path.exists() {
        return Err("Python migration source path does not exist.".to_string());
    }

    Ok(path)
}

fn discover_duckdb_files(source: &Path) -> Result<Vec<PathBuf>, String> {
    if source.is_file() {
        if is_duckdb_file(source) {
            return Ok(vec![source.to_path_buf()]);
        }

        return Err("Source file is not a DuckDB database.".to_string());
    }

    if !source.is_dir() {
        return Err("Source path is neither a file nor a directory.".to_string());
    }

    let mut files = Vec::new();
    discover_duckdb_files_inner(source, 0, &mut files)?;

    files.sort();
    files.dedup();

    Ok(files)
}

fn discover_duckdb_files_inner(
    dir: &Path,
    depth: usize,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if depth > 4 {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        if matches!(name, ".git" | "target" | "node_modules" | "__pycache__") {
            continue;
        }

        if path.is_dir() {
            discover_duckdb_files_inner(&path, depth + 1, files)?;
        } else if is_duckdb_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn is_duckdb_file(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "duckdb" | "db"))
        .unwrap_or(false)
}

fn list_tables(connection: &Connection) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT table_name
             FROM information_schema.tables
             WHERE table_schema = 'main'
             ORDER BY table_name",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;

    collect_rows(rows)
}

fn list_columns(connection: &Connection, table: &str) -> Result<Vec<String>, String> {
    let mut statement = connection
        .prepare(
            "SELECT column_name
             FROM information_schema.columns
             WHERE table_schema = 'main' AND table_name = ?
             ORDER BY ordinal_position",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map(params![table], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?;

    collect_rows(rows)
}

fn count_table_rows(connection: &Connection, table: &str) -> Result<usize, String> {
    let sql = format!("SELECT COUNT(*) FROM {}", quoted_identifier(table));
    let count: i64 = connection
        .query_row(&sql, [], |row| row.get(0))
        .map_err(|error| error.to_string())?;

    Ok(usize::try_from(count.max(0)).unwrap_or(usize::MAX))
}

fn collect_rows<T>(
    rows: duckdb::MappedRows<'_, impl FnMut(&duckdb::Row<'_>) -> duckdb::Result<T>>,
) -> Result<Vec<T>, String> {
    let mut out = Vec::new();

    for row in rows {
        out.push(row.map_err(|error| error.to_string())?);
    }

    Ok(out)
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

const ADDRESS_COLUMNS: &[&str] = &[
    "address",
    "kaspa_address",
    "wallet_address",
    "addr",
    "wallet",
];
const NAME_COLUMNS: &[&str] = &["name", "known_name", "address_name", "label", "title"];
const NETWORK_COLUMNS: &[&str] = &["network", "chain", "net"];
const TXID_COLUMNS: &[&str] = &["txid", "transaction_id", "transaction_hash", "hash", "id"];
const AMOUNT_COLUMNS: &[&str] = &["amount_sompi", "sompi", "amount", "value"];
const TIMESTAMP_COLUMNS: &[&str] = &[
    "timestamp_ms",
    "timestamp",
    "time",
    "block_time",
    "created_at",
];
const DIRECTION_COLUMNS: &[&str] = &["direction", "tx_direction", "flow"];
const TX_TYPE_COLUMNS: &[&str] = &["tx_type", "type", "transaction_type", "kind"];
const COUNTERPARTY_COLUMNS: &[&str] = &["counterparty", "peer", "from_to"];
const RAW_JSON_COLUMNS: &[&str] = &["raw_json", "json", "raw", "payload"];
const SETTING_KEY_COLUMNS: &[&str] = &["key", "setting_key", "name", "config_key", "option"];
const SETTING_VALUE_COLUMNS: &[&str] = &["value", "setting_value", "config_value", "data", "json"];

fn find_column(columns: &[String], candidates: &[&str]) -> Option<String> {
    for candidate in candidates {
        if let Some(found) = columns
            .iter()
            .find(|column| column.eq_ignore_ascii_case(candidate))
        {
            return Some(found.clone());
        }
    }

    None
}

fn insert_if_found(
    map: &mut BTreeMap<String, String>,
    role: &str,
    columns: &[String],
    candidates: &[&str],
) {
    if let Some(column) = find_column(columns, candidates) {
        map.insert(role.to_string(), column);
    }
}

fn required_mapped_column<'a>(
    map: &'a RealMigrationColumnMap,
    key: &str,
) -> Result<&'a str, String> {
    map.columns
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("Required mapped column missing: {key}"))
}

fn quoted_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn cast_column_sql(column: &str) -> String {
    format!("CAST({} AS VARCHAR)", quoted_identifier(column))
}

fn optional_cast_column_sql(column: Option<&str>, fallback: &str) -> String {
    column
        .map(cast_column_sql)
        .unwrap_or_else(|| fallback.to_string())
}

fn validate_path_text(value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err("Path contains unsafe characters.".to_string());
    }

    Ok(())
}

fn parse_i64(value: Option<&str>) -> Option<i64> {
    let value = value?.trim();

    value.parse::<i64>().ok().or_else(|| {
        value
            .parse::<f64>()
            .ok()
            .map(|number| number.round() as i64)
    })
}

fn normalize_timestamp_ms(value: i64) -> i64 {
    if value <= 0 {
        0
    } else if value < 10_000_000_000 {
        value.saturating_mul(1000)
    } else {
        value
    }
}

fn normalize_direction(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "in" | "incoming" | "receive" | "received" => "incoming".to_string(),
        "out" | "outgoing" | "send" | "sent" => "outgoing".to_string(),
        "self" => "self".to_string(),
        _ => "unknown".to_string(),
    }
}

fn clean_network(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "mainnet" | "kaspa" => "mainnet".to_string(),
        "testnet" | "testnet-10" | "kaspatest" => "testnet".to_string(),
        "testnet-11" => "testnet-11".to_string(),
        "simnet" => "simnet".to_string(),
        "devnet" => "devnet".to_string(),
        _ => "mainnet".to_string(),
    }
}

fn clean_text(value: &str, fallback: &str) -> String {
    let clean = value.replace('\0', "").trim().to_string();

    if clean.is_empty() {
        fallback.to_string()
    } else {
        clean
    }
}

fn sanitize_setting_key(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-' || *ch == '.')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_address_columns() {
        let columns = vec!["id".to_string(), "wallet_address".to_string()];
        let map = map_address_table("addresses", &columns, 5).expect("map");
        assert_eq!(map.role, "addresses");
        assert_eq!(map.columns.get("address").unwrap(), "wallet_address");
    }

    #[test]
    fn discovers_duckdb_extensions() {
        assert!(is_duckdb_file(Path::new("data.duckdb")));
        assert!(is_duckdb_file(Path::new("data.db")));
        assert!(!is_duckdb_file(Path::new("data.txt")));
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        assert!(validate_path_text("D:\\PythonApp").is_ok());
        assert!(validate_path_text("bad && whoami").is_err());
    }
}
