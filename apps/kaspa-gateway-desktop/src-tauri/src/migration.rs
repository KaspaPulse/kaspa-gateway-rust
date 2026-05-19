use duckdb::{params, Connection};
use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{AddressRecord, DatabaseManager, DatabasePaths, TransactionRecord};
use serde::Serialize;
use std::path::{Path, PathBuf};

const MAX_ROWS_PER_TABLE: usize = 50_000;

#[derive(Debug, Clone, Serialize)]
pub struct MigrationPreview {
    pub source_path: String,
    pub addresses_db: Option<String>,
    pub transactions_db: Option<String>,
    pub app_data_db: Option<String>,
    pub importable_addresses: usize,
    pub importable_transactions: usize,
    pub importable_settings: usize,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationResult {
    pub source_path: String,
    pub addresses_imported: usize,
    pub transactions_imported: usize,
    pub settings_imported: usize,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn preview_python_migration(source_path: String) -> Result<MigrationPreview, String> {
    let source = normalize_source_path(&source_path)?;
    let candidates = MigrationCandidates::from_source(&source);

    let mut notes = Vec::new();

    let importable_addresses = preview_count(
        candidates.addresses_db.as_deref(),
        count_importable_addresses,
        "Addresses.duckdb was not found.",
        "Addresses preview warning",
        &mut notes,
    );

    let importable_transactions = preview_count(
        candidates.transactions_db.as_deref(),
        count_importable_transactions,
        "Transactions.duckdb was not found.",
        "Transactions preview warning",
        &mut notes,
    );

    let importable_settings = preview_count(
        candidates.app_data_db.as_deref(),
        count_importable_settings,
        "AppData.duckdb was not found.",
        "AppData preview warning",
        &mut notes,
    );

    Ok(MigrationPreview {
        source_path: source.display().to_string(),
        addresses_db: candidates
            .addresses_db
            .as_ref()
            .map(|value| value.display().to_string()),
        transactions_db: candidates
            .transactions_db
            .as_ref()
            .map(|value| value.display().to_string()),
        app_data_db: candidates
            .app_data_db
            .as_ref()
            .map(|value| value.display().to_string()),
        importable_addresses,
        importable_transactions,
        importable_settings,
        notes,
    })
}

#[tauri::command]
pub fn migrate_python_data(source_path: String) -> Result<MigrationResult, String> {
    let source = normalize_source_path(&source_path)?;
    let candidates = MigrationCandidates::from_source(&source);
    let manager = database_manager()?;

    let mut warnings = Vec::new();

    let addresses_imported = import_count(
        candidates.addresses_db.as_deref(),
        |path| import_addresses(path, &manager),
        "Addresses.duckdb was not found.",
        "Address migration warning",
        &mut warnings,
    );

    let transactions_imported = import_count(
        candidates.transactions_db.as_deref(),
        |path| import_transactions(path, &manager),
        "Transactions.duckdb was not found.",
        "Transaction migration warning",
        &mut warnings,
    );

    let settings_imported = import_count(
        candidates.app_data_db.as_deref(),
        |path| import_settings(path, &manager),
        "AppData.duckdb was not found.",
        "Settings migration warning",
        &mut warnings,
    );

    Ok(MigrationResult {
        source_path: source.display().to_string(),
        addresses_imported,
        transactions_imported,
        settings_imported,
        warnings,
    })
}

struct MigrationCandidates {
    addresses_db: Option<PathBuf>,
    transactions_db: Option<PathBuf>,
    app_data_db: Option<PathBuf>,
}

impl MigrationCandidates {
    fn from_source(source: &Path) -> Self {
        if source.is_file() {
            let file_name = source
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            return Self {
                addresses_db: if file_name.contains("address") {
                    Some(source.to_path_buf())
                } else {
                    None
                },
                transactions_db: if file_name.contains("transaction") || file_name.contains("tx") {
                    Some(source.to_path_buf())
                } else {
                    None
                },
                app_data_db: if file_name.contains("app") || file_name.contains("data") {
                    Some(source.to_path_buf())
                } else {
                    None
                },
            };
        }

        Self {
            addresses_db: first_existing(
                source,
                &[
                    "Addresses.duckdb",
                    "addresses.duckdb",
                    "Address.duckdb",
                    "address.duckdb",
                ],
            ),
            transactions_db: first_existing(
                source,
                &[
                    "Transactions.duckdb",
                    "transactions.duckdb",
                    "Tx.duckdb",
                    "tx.duckdb",
                ],
            ),
            app_data_db: first_existing(
                source,
                &[
                    "AppData.duckdb",
                    "app_data.duckdb",
                    "App.duckdb",
                    "app.duckdb",
                    "data.duckdb",
                ],
            ),
        }
    }
}

fn preview_count<F>(
    path: Option<&Path>,
    counter: F,
    missing: &str,
    label: &str,
    notes: &mut Vec<String>,
) -> usize
where
    F: FnOnce(&Path) -> Result<usize, String>,
{
    let Some(path) = path else {
        notes.push(missing.to_string());
        return 0;
    };

    match counter(path) {
        Ok(count) => count,
        Err(error) => {
            notes.push(format!("{label}: {error}"));
            0
        }
    }
}

fn import_count<F>(
    path: Option<&Path>,
    importer: F,
    missing: &str,
    label: &str,
    warnings: &mut Vec<String>,
) -> usize
where
    F: FnOnce(&Path) -> Result<usize, String>,
{
    let Some(path) = path else {
        warnings.push(missing.to_string());
        return 0;
    };

    match importer(path) {
        Ok(count) => count,
        Err(error) => {
            warnings.push(format!("{label}: {error}"));
            0
        }
    }
}

fn first_existing(root: &Path, names: &[&str]) -> Option<PathBuf> {
    names
        .iter()
        .map(|name| root.join(name))
        .find(|candidate| candidate.is_file())
}

fn count_importable_addresses(path: &Path) -> Result<usize, String> {
    count_role_tables(path, "addresses")
}

fn count_importable_transactions(path: &Path) -> Result<usize, String> {
    count_role_tables(path, "transactions")
}

fn count_importable_settings(path: &Path) -> Result<usize, String> {
    count_role_tables(path, "settings")
}

fn count_role_tables(path: &Path, role: &str) -> Result<usize, String> {
    let connection = open_duckdb(path)?;
    let mut count = 0_usize;

    for table in list_tables(&connection)? {
        let columns = list_columns(&connection, &table)?;

        let matches = match role {
            "addresses" => find_column(&columns, ADDRESS_COLUMNS).is_some(),
            "transactions" => {
                find_column(&columns, TXID_COLUMNS).is_some()
                    && find_column(&columns, ADDRESS_COLUMNS).is_some()
            }
            "settings" => {
                find_column(&columns, SETTING_KEY_COLUMNS).is_some()
                    && find_column(&columns, SETTING_VALUE_COLUMNS).is_some()
            }
            _ => false,
        };

        if matches {
            count = count.saturating_add(count_table_rows(&connection, &table)?);
        }
    }

    Ok(count)
}

fn import_addresses(path: &Path, manager: &DatabaseManager) -> Result<usize, String> {
    let connection = open_duckdb(path)?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let mut imported = 0_usize;

    for table in list_tables(&connection)? {
        let columns = list_columns(&connection, &table)?;

        let Some(address_col) = find_column(&columns, ADDRESS_COLUMNS) else {
            continue;
        };

        let name_col = find_column(&columns, NAME_COLUMNS);
        let network_col = find_column(&columns, NETWORK_COLUMNS);

        let select = format!(
            "SELECT {}, {}, {} FROM {} LIMIT {}",
            cast_column_sql(&address_col),
            optional_cast_column_sql(name_col.as_deref(), "'Imported Address'"),
            optional_cast_column_sql(network_col.as_deref(), "'mainnet'"),
            quoted_identifier(&table),
            MAX_ROWS_PER_TABLE
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

        for row in rows {
            let (address, name, network) = row.map_err(|error| error.to_string())?;
            let Some(address) = address.map(|value| value.trim().to_string()) else {
                continue;
            };

            if KaspaAddress::parse(&address).is_err() {
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

            imported += 1;
        }
    }

    Ok(imported)
}

fn import_transactions(path: &Path, manager: &DatabaseManager) -> Result<usize, String> {
    let connection = open_duckdb(path)?;
    let repository = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let mut imported = 0_usize;

    for table in list_tables(&connection)? {
        let columns = list_columns(&connection, &table)?;

        let Some(txid_col) = find_column(&columns, TXID_COLUMNS) else {
            continue;
        };

        let Some(address_col) = find_column(&columns, ADDRESS_COLUMNS) else {
            continue;
        };

        let amount_col = find_column(&columns, AMOUNT_COLUMNS);
        let timestamp_col = find_column(&columns, TIMESTAMP_COLUMNS);
        let direction_col = find_column(&columns, DIRECTION_COLUMNS);
        let tx_type_col = find_column(&columns, TX_TYPE_COLUMNS);
        let counterparty_col = find_column(&columns, COUNTERPARTY_COLUMNS);
        let raw_json_col = find_column(&columns, RAW_JSON_COLUMNS);

        let select = format!(
            "SELECT {}, {}, {}, {}, {}, {}, {}, {} FROM {} LIMIT {}",
            cast_column_sql(&txid_col),
            cast_column_sql(&address_col),
            optional_cast_column_sql(amount_col.as_deref(), "'0'"),
            optional_cast_column_sql(timestamp_col.as_deref(), "'0'"),
            optional_cast_column_sql(direction_col.as_deref(), "'unknown'"),
            optional_cast_column_sql(tx_type_col.as_deref(), "'transfer'"),
            optional_cast_column_sql(counterparty_col.as_deref(), "NULL"),
            optional_cast_column_sql(raw_json_col.as_deref(), "NULL"),
            quoted_identifier(&table),
            MAX_ROWS_PER_TABLE
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

        for row in rows {
            let (txid, address, amount, timestamp, direction, tx_type, counterparty, raw_json) =
                row.map_err(|error| error.to_string())?;

            let Some(txid) = txid.map(|value| value.trim().to_string()) else {
                continue;
            };

            let Some(address) = address.map(|value| value.trim().to_string()) else {
                continue;
            };

            if txid.is_empty() || KaspaAddress::parse(&address).is_err() {
                continue;
            }

            let amount_sompi = parse_i64(amount.as_deref()).unwrap_or(0).max(0);
            let direction = normalize_direction(direction.as_deref().unwrap_or("unknown"));
            let tx_type = clean_text(tx_type.as_deref().unwrap_or("transfer"), "transfer");

            let mut record =
                TransactionRecord::new(txid, address, tx_type, direction, amount_sompi)
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

            imported += 1;
        }
    }

    Ok(imported)
}

fn import_settings(path: &Path, manager: &DatabaseManager) -> Result<usize, String> {
    let connection = open_duckdb(path)?;
    let repository = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let mut imported = 0_usize;

    for table in list_tables(&connection)? {
        let columns = list_columns(&connection, &table)?;

        let Some(key_col) = find_column(&columns, SETTING_KEY_COLUMNS) else {
            continue;
        };

        let Some(value_col) = find_column(&columns, SETTING_VALUE_COLUMNS) else {
            continue;
        };

        let select = format!(
            "SELECT {}, {} FROM {} LIMIT {}",
            cast_column_sql(&key_col),
            cast_column_sql(&value_col),
            quoted_identifier(&table),
            MAX_ROWS_PER_TABLE
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

        for row in rows {
            let (key, value) = row.map_err(|error| error.to_string())?;
            let Some(key) = key.map(|value| sanitize_setting_key(&value)) else {
                continue;
            };

            if key.is_empty() {
                continue;
            }

            let value = value.unwrap_or_default();

            repository
                .set(&format!("python.{key}"), &value)
                .map_err(|error| error.to_string())?;

            imported += 1;
        }
    }

    Ok(imported)
}

fn normalize_source_path(source_path: &str) -> Result<PathBuf, String> {
    validate_path_text(source_path)?;

    let path = PathBuf::from(source_path.trim());

    if !path.exists() {
        return Err("Python migration source path does not exist.".to_string());
    }

    Ok(path)
}

fn open_duckdb(path: &Path) -> Result<Connection, String> {
    if !path.exists() {
        return Err(format!("DuckDB file does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("DuckDB path is not a file: {}", path.display()));
    }

    Connection::open(path).map_err(|error| error.to_string())
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
const SETTING_KEY_COLUMNS: &[&str] = &["key", "name", "setting_key", "config_key", "option"];
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
    fn identifiers_are_quoted() {
        assert_eq!(quoted_identifier("abc"), "\"abc\"");
        assert_eq!(quoted_identifier("a\"b"), "\"a\"\"b\"");
    }

    #[test]
    fn timestamps_are_normalized() {
        assert_eq!(normalize_timestamp_ms(1_700_000_000), 1_700_000_000_000);
        assert_eq!(normalize_timestamp_ms(1_700_000_000_000), 1_700_000_000_000);
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        assert!(validate_path_text("D:\\PythonApp").is_ok());
        assert!(validate_path_text("bad && whoami").is_err());
    }
}
