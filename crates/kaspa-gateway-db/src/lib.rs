// ============================================================================
// KGW_OWNERSHIP_DB_REPOSITORY_ONLY
// Database layer owns schema, repository APIs, and persistence primitives.
// Forbidden: HTTP API fetch orchestration, Tauri IPC ownership, and frontend/UI behavior.
// ============================================================================

use duckdb::{Connection, OptionalExt, params};
use rusqlite as sqlite;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

pub const DATABASE_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database path is invalid")]
    InvalidPath,

    #[error("invalid record: {0}")]
    InvalidRecord(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("duckdb error: {0}")]
    DuckDb(#[from] duckdb::Error),

    #[error("sqlite error: {0}")]
    SQLite(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, DbError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabasePaths {
    pub root: PathBuf,
    pub transactions: PathBuf,
    pub transactions_sqlite: PathBuf,
    pub addresses: PathBuf,
    pub app_data: PathBuf,
}

impl DatabasePaths {
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();

        if root.as_os_str().is_empty() {
            return Err(DbError::InvalidPath);
        }

        let root_text = root.to_string_lossy();

        if root_text.trim().is_empty()
            || root_text.contains('\0')
            || root_text.contains('"')
            || root_text.contains('\n')
            || root_text.contains('\r')
        {
            return Err(DbError::InvalidPath);
        }

        Ok(Self {
            transactions: root.join("Transactions.duckdb"),
            transactions_sqlite: root.join("Transactions.sqlite"),
            addresses: root.join("Addresses.duckdb"),
            app_data: root.join("AppData.duckdb"),
            root,
        })
    }

    pub fn ensure_root(&self) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    paths: DatabasePaths,
}

impl DatabaseManager {
    pub fn new(paths: DatabasePaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &DatabasePaths {
        &self.paths
    }

    pub fn initialize_all(&self) -> Result<()> {
        self.paths.ensure_root()?;

        let app_data = self.open_app_data()?;
        initialize_app_data_schema(&app_data)?;

        let addresses = self.open_addresses()?;
        initialize_addresses_schema(&addresses)?;

        let transactions = self.open_transactions()?;
        initialize_transactions_schema(&transactions)?;

        Ok(())
    }

    pub fn open_app_data(&self) -> Result<Connection> {
        self.paths.ensure_root()?;
        Ok(Connection::open(&self.paths.app_data)?)
    }

    pub fn open_addresses(&self) -> Result<Connection> {
        self.paths.ensure_root()?;
        Ok(Connection::open(&self.paths.addresses)?)
    }

    pub fn open_transactions(&self) -> Result<Connection> {
        self.paths.ensure_root()?;
        Ok(Connection::open(&self.paths.transactions)?)
    }

    pub fn addresses_repository(&self) -> Result<AddressesRepository> {
        let connection = self.open_addresses()?;
        initialize_addresses_schema(&connection)?;
        Ok(AddressesRepository::new(connection))
    }

    pub fn transactions_repository(&self) -> Result<TransactionsRepository> {
        self.paths.ensure_root()?;
        let connection = sqlite::Connection::open(&self.paths.transactions_sqlite)?;
        initialize_sqlite_transactions_schema(&connection)?;
        Ok(TransactionsRepository::new(connection))
    }

    pub fn app_settings_repository(&self) -> Result<AppSettingsRepository> {
        let connection = self.open_app_data()?;
        initialize_app_data_schema(&connection)?;
        Ok(AppSettingsRepository::new(connection))
    }

    pub fn app_cache_repository(&self) -> Result<AppCacheRepository> {
        let connection = self.open_app_data()?;
        initialize_app_data_schema(&connection)?;
        Ok(AppCacheRepository::new(connection))
    }

    pub fn known_names_repository(&self) -> Result<KnownNamesRepository> {
        let connection = self.open_app_data()?;
        initialize_app_data_schema(&connection)?;
        Ok(KnownNamesRepository::new(connection))
    }

    pub fn compact_all(&self) -> Result<()> {
        for path in [
            &self.paths.app_data,
            &self.paths.addresses,
            &self.paths.transactions,
        ] {
            if path.exists() {
                let connection = Connection::open(path)?;
                connection.execute_batch("VACUUM; CHECKPOINT;")?;
            }
        }

        Ok(())
    }

    pub fn delete_and_reinitialize_database(&self, database: DatabaseKind) -> Result<()> {
        let path = match database {
            DatabaseKind::AppData => &self.paths.app_data,
            DatabaseKind::Addresses => &self.paths.addresses,
            DatabaseKind::Transactions => &self.paths.transactions,
        };

        let wal_path = PathBuf::from(format!("{}.wal", path.display()));

        if path.exists() {
            fs::remove_file(path)?;
        }

        if wal_path.exists() {
            fs::remove_file(wal_path)?;
        }

        match database {
            DatabaseKind::AppData => {
                let connection = self.open_app_data()?;
                initialize_app_data_schema(&connection)?;
            }
            DatabaseKind::Addresses => {
                let connection = self.open_addresses()?;
                initialize_addresses_schema(&connection)?;
            }
            DatabaseKind::Transactions => {
                let connection = self.open_transactions()?;
                initialize_transactions_schema(&connection)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseKind {
    AppData,
    Addresses,
    Transactions,
}

pub fn schema_version() -> i64 {
    DATABASE_SCHEMA_VERSION
}

pub fn initialize_app_data_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations(
            version BIGINT PRIMARY KEY,
            name VARCHAR NOT NULL,
            applied_at_ms BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings(
            key VARCHAR PRIMARY KEY,
            value VARCHAR NOT NULL,
            updated_at_ms BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cache(
            key VARCHAR PRIMARY KEY,
            prices_json VARCHAR,
            value VARCHAR,
            expires_at_ms BIGINT,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at_ms BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS known_names(
            address VARCHAR PRIMARY KEY,
            name VARCHAR NOT NULL,
            updated_at_ms BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS user_state(
            key VARCHAR PRIMARY KEY,
            value VARCHAR NOT NULL,
            updated_at_ms BIGINT NOT NULL
        );
        "#,
    )?;

    record_migration(connection, "app_data_schema")?;
    Ok(())
}

pub fn initialize_addresses_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations(
            version BIGINT PRIMARY KEY,
            name VARCHAR NOT NULL,
            applied_at_ms BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS addresses(
            address VARCHAR PRIMARY KEY,
            name VARCHAR NOT NULL,
            network VARCHAR NOT NULL DEFAULT 'mainnet',
            created_at_ms BIGINT NOT NULL,
            updated_at_ms BIGINT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_addresses_name ON addresses(name);
        CREATE INDEX IF NOT EXISTS idx_addresses_network ON addresses(network);
        "#,
    )?;

    record_migration(connection, "addresses_schema")?;
    Ok(())
}

fn initialize_sqlite_transactions_schema(connection: &sqlite::Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 5000;

        CREATE TABLE IF NOT EXISTS schema_migrations(
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transactions(
            txid TEXT PRIMARY KEY,
            address TEXT NOT NULL,
            tx_type TEXT NOT NULL,
            direction TEXT NOT NULL,
            amount_sompi INTEGER NOT NULL,
            from_address TEXT,
            to_address TEXT,
            counterparty TEXT,
            block_height INTEGER,
            timestamp_ms INTEGER NOT NULL,
            raw_json TEXT NOT NULL,
            created_at_ms INTEGER NOT NULL,
            updated_at_ms INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_transactions_address_time
            ON transactions(address, timestamp_ms DESC);

        CREATE INDEX IF NOT EXISTS idx_transactions_address_type
            ON transactions(address, tx_type);

        CREATE INDEX IF NOT EXISTS idx_transactions_address_direction
            ON transactions(address, direction);

        CREATE INDEX IF NOT EXISTS idx_transactions_address_txid
            ON transactions(address, txid);
        "#,
    )?;

    connection.execute(
        r#"
        INSERT INTO schema_migrations(version, name, applied_at_ms)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(version) DO UPDATE SET
            name = excluded.name,
            applied_at_ms = excluded.applied_at_ms
        "#,
        rusqlite::params![
            DATABASE_SCHEMA_VERSION,
            "transactions_sqlite_schema",
            now_ms()
        ],
    )?;

    Ok(())
}
pub fn initialize_transactions_schema(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations(
            version BIGINT PRIMARY KEY,
            name VARCHAR NOT NULL,
            applied_at_ms BIGINT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS transactions(
            txid VARCHAR PRIMARY KEY,
            address VARCHAR NOT NULL,
            tx_type VARCHAR NOT NULL,
            direction VARCHAR NOT NULL,
            amount_sompi BIGINT NOT NULL,
            from_address VARCHAR,
            to_address VARCHAR,
            counterparty VARCHAR,
            block_height BIGINT,
            timestamp_ms BIGINT NOT NULL,
            raw_json VARCHAR,
            created_at_ms BIGINT NOT NULL,
            updated_at_ms BIGINT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_transactions_address ON transactions(address);
        CREATE INDEX IF NOT EXISTS idx_transactions_address_timestamp ON transactions(address, timestamp_ms);
        CREATE INDEX IF NOT EXISTS idx_transactions_direction ON transactions(direction);
        CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(tx_type);
        "#,
    )?;

    record_migration(connection, "transactions_schema")?;
    Ok(())
}

pub fn initialize_addr_schema(connection: &Connection) -> Result<()> {
    initialize_addresses_schema(connection)
}

pub fn initialize_tx_schema(connection: &Connection) -> Result<()> {
    initialize_transactions_schema(connection)
}

fn record_migration(connection: &Connection, name: &str) -> Result<()> {
    connection.execute(
        r#"
        INSERT OR REPLACE INTO schema_migrations(version, name, applied_at_ms)
        VALUES (?1, ?2, ?3)
        "#,
        params![DATABASE_SCHEMA_VERSION, name, now_ms()],
    )?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddressRecord {
    pub address: String,
    pub name: String,
    pub network: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl AddressRecord {
    pub fn new(
        address: impl Into<String>,
        name: impl Into<String>,
        network: impl Into<String>,
    ) -> Result<Self> {
        let address = address.into().trim().to_string();
        let name = name.into().trim().to_string();
        let network = network.into().trim().to_ascii_lowercase();

        validate_non_empty("address", &address)?;
        validate_optional_text("name", &name)?;
        validate_network(&network)?;

        if !is_kaspa_address_like(&address) {
            return Err(DbError::InvalidRecord(
                "address must start with kaspa:, kaspatest:, kaspadev:, or kaspasim:".to_string(),
            ));
        }

        let now = now_ms();

        Ok(Self {
            address,
            name,
            network,
            created_at_ms: now,
            updated_at_ms: now,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransactionRecord {
    pub txid: String,
    pub address: String,
    pub tx_type: String,
    pub direction: String,
    pub amount_sompi: i64,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub counterparty: Option<String>,
    pub block_height: Option<i64>,
    pub timestamp_ms: i64,
    pub raw_json: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl TransactionRecord {
    pub fn new(
        txid: impl Into<String>,
        address: impl Into<String>,
        tx_type: impl Into<String>,
        direction: impl Into<String>,
        amount_sompi: i64,
    ) -> Result<Self> {
        let txid = txid.into().trim().to_string();
        let address = address.into().trim().to_string();
        let tx_type = normalize_tx_type(&tx_type.into());
        let direction = normalize_direction(&direction.into());

        validate_non_empty("txid", &txid)?;

        if !is_kaspa_address_like(&address) {
            return Err(DbError::InvalidRecord(
                "transaction address must be a Kaspa address".to_string(),
            ));
        }

        if amount_sompi < 0 {
            return Err(DbError::InvalidRecord(
                "amount_sompi must be non-negative".to_string(),
            ));
        }

        if !matches!(tx_type.as_str(), "transfer" | "coinbase" | "unknown") {
            return Err(DbError::InvalidRecord(format!(
                "unsupported tx_type: {tx_type}"
            )));
        }

        if !matches!(
            direction.as_str(),
            "incoming" | "outgoing" | "self" | "unknown"
        ) {
            return Err(DbError::InvalidRecord(format!(
                "unsupported direction: {direction}"
            )));
        }

        let now = now_ms();

        Ok(Self {
            txid,
            address,
            tx_type,
            direction,
            amount_sompi,
            from_address: None,
            to_address: None,
            counterparty: None,
            block_height: None,
            timestamp_ms: now,
            raw_json: None,
            created_at_ms: now,
            updated_at_ms: now,
        })
    }

    pub fn with_counterparty(mut self, counterparty: impl Into<String>) -> Self {
        let value = counterparty.into().trim().to_string();

        if !value.is_empty() {
            self.counterparty = Some(value);
        }

        self
    }

    pub fn with_parties(
        mut self,
        from_address: Option<String>,
        to_address: Option<String>,
    ) -> Self {
        self.from_address = from_address;
        self.to_address = to_address;
        self
    }

    pub fn with_block_height(mut self, block_height: i64) -> Self {
        if block_height >= 0 {
            self.block_height = Some(block_height);
        }

        self
    }

    pub fn with_timestamp_ms(mut self, timestamp_ms: i64) -> Self {
        if timestamp_ms >= 0 {
            self.timestamp_ms = timestamp_ms;
        }

        self
    }

    pub fn amount_kas(&self) -> f64 {
        self.amount_sompi as f64 / 100_000_000.0
    }
}

#[derive(Debug)]
pub struct AddressesRepository {
    connection: Connection,
}

impl AddressesRepository {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn upsert(&self, record: &AddressRecord) -> Result<()> {
        validate_non_empty("address", &record.address)?;
        validate_optional_text("name", &record.name)?;
        validate_network(&record.network)?;

        let existing_created_at = self
            .connection
            .query_row(
                "SELECT created_at_ms FROM addresses WHERE address = ?1",
                params![record.address],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .unwrap_or(record.created_at_ms);

        self.connection.execute(
            r#"
            INSERT INTO addresses(address, name, network, created_at_ms, updated_at_ms)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(address) DO UPDATE SET
                name = excluded.name,
                network = excluded.network,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![
                record.address,
                record.name,
                record.network,
                existing_created_at,
                now_ms()
            ],
        )?;

        Ok(())
    }

    pub fn get(&self, address: &str) -> Result<Option<AddressRecord>> {
        validate_non_empty("address", address)?;

        self.connection
            .query_row(
                r#"
                SELECT address, name, network, created_at_ms, updated_at_ms
                FROM addresses
                WHERE address = ?1
                "#,
                params![address],
                address_record_from_row,
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn list(&self) -> Result<Vec<AddressRecord>> {
        let mut statement = self.connection.prepare(
            r#"
            SELECT address, name, network, created_at_ms, updated_at_ms
            FROM addresses
            ORDER BY name ASC, address ASC
            "#,
        )?;

        let rows = statement.query_map([], address_record_from_row)?;

        let mut records = Vec::new();

        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn delete(&self, address: &str) -> Result<bool> {
        validate_non_empty("address", address)?;

        let affected = self
            .connection
            .execute("DELETE FROM addresses WHERE address = ?1", params![address])?;

        Ok(affected > 0)
    }

    pub fn count(&self) -> Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM addresses", [], |row| row.get(0))
            .map_err(DbError::from)
    }
}

#[derive(Debug, Clone)]
pub struct TransactionDaySummaryRecord {
    pub day: String,
    pub count: usize,
    pub incoming_sompi: i64,
    pub outgoing_sompi: i64,
}
pub struct TransactionsRepository {
    connection: sqlite::Connection,
}

impl TransactionsRepository {
    pub fn new(connection: sqlite::Connection) -> Self {
        Self { connection }
    }

    pub fn upsert(&self, record: &TransactionRecord) -> Result<()> {
        self.upsert_many(std::slice::from_ref(record)).map(|_| ())
    }

    pub fn list_for_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionRecord>> {
        validate_non_empty("address", address)?;

        let limit = limit.try_into().unwrap_or(500).max(1);

        let mut statement = self.connection.prepare(
            r#"
            SELECT
                txid, address, tx_type, direction, amount_sompi,
                from_address, to_address, counterparty, block_height,
                timestamp_ms, raw_json, created_at_ms, updated_at_ms
            FROM transactions
            WHERE address = ?1
            ORDER BY timestamp_ms DESC
            LIMIT ?2
            "#,
        )?;

        let rows = statement.query_map(
            rusqlite::params![address, limit],
            sqlite_transaction_record_from_row,
        )?;

        let mut records = Vec::new();

        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn count_for_address(&self, address: &str) -> Result<i64> {
        validate_non_empty("address", address)?;

        self.connection
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE address = ?1",
                rusqlite::params![address],
                |row| row.get(0),
            )
            .map_err(DbError::from)
    }

    pub fn total_count(&self) -> Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM transactions", [], |row| row.get(0))
            .map_err(DbError::from)
    }

    pub fn upsert_many(&self, records: &[TransactionRecord]) -> Result<usize> {
        let started = std::time::Instant::now();

        eprintln!(
            "[KGW][transactions][SQLITE-WRITER] upsert_many start records={} first_txid={} last_txid={}",
            records.len(),
            records
                .first()
                .map(|record| record.txid.as_str())
                .unwrap_or("<none>"),
            records
                .last()
                .map(|record| record.txid.as_str())
                .unwrap_or("<none>")
        );

        if records.is_empty() {
            eprintln!("[KGW][transactions][SQLITE-WRITER] upsert_many empty batch");
            return Ok(0);
        }

        for (index, record) in records.iter().enumerate() {
            validate_non_empty("txid", &record.txid)?;
            validate_non_empty("address", &record.address)?;

            if record.amount_sompi < 0 {
                eprintln!(
                    "[KGW][transactions][SQLITE-WRITER][ERROR] invalid negative amount index={} txid={} amount_sompi={}",
                    index, record.txid, record.amount_sompi
                );

                return Err(DbError::InvalidRecord(
                    "amount_sompi must be non-negative".to_string(),
                ));
            }
        }

        let mut statement = self.connection.prepare(
            r#"
            INSERT INTO transactions(
                txid, address, tx_type, direction, amount_sompi,
                from_address, to_address, counterparty, block_height,
                timestamp_ms, raw_json, created_at_ms, updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(txid) DO UPDATE SET
                address = excluded.address,
                tx_type = excluded.tx_type,
                direction = excluded.direction,
                amount_sompi = excluded.amount_sompi,
                from_address = excluded.from_address,
                to_address = excluded.to_address,
                counterparty = excluded.counterparty,
                block_height = excluded.block_height,
                timestamp_ms = excluded.timestamp_ms,
                raw_json = excluded.raw_json,
                updated_at_ms = excluded.updated_at_ms
            "#,
        )?;

        let mut stored = 0usize;

        for (index, record) in records.iter().enumerate() {
            let item_no = index + 1;
            let item_started = std::time::Instant::now();
            statement.execute(rusqlite::params![
                &record.txid,
                &record.address,
                &record.tx_type,
                &record.direction,
                record.amount_sompi,
                &record.from_address,
                &record.to_address,
                &record.counterparty,
                &record.block_height,
                record.timestamp_ms,
                &record.raw_json,
                record.created_at_ms,
                record.updated_at_ms,
            ])?;

            stored += 1;

            let elapsed = item_started.elapsed().as_millis();

            if elapsed > 250 {
                eprintln!(
                    "[KGW][transactions][SQLITE-WRITER] upsert item done item={}/{} txid={} stored={} elapsed_ms={}",
                    item_no,
                    records.len(),
                    record.txid,
                    stored,
                    elapsed
                );
            }
        }

        eprintln!(
            "[KGW][transactions][SQLITE-WRITER] upsert_many done records={} stored={} elapsed_ms={}",
            records.len(),
            stored,
            started.elapsed().as_millis()
        );

        Ok(stored)
    }

    pub fn existing_txids_for_address(&self, address: &str) -> Result<Vec<String>> {
        validate_non_empty("address", address)?;

        let mut statement = self
            .connection
            .prepare("SELECT txid FROM transactions WHERE address = ?1")?;

        let rows =
            statement.query_map(rusqlite::params![address], |row| row.get::<_, String>(0))?;

        let mut txids = Vec::new();

        for row in rows {
            txids.push(row?);
        }

        Ok(txids)
    }

    pub fn existing_txids_for_candidates(
        &self,
        address: &str,
        candidates: &[String],
    ) -> Result<Vec<String>> {
        validate_non_empty("address", address)?;

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let mut found = Vec::new();

        for chunk in candidates.chunks(500) {
            let placeholders = std::iter::repeat("?")
                .take(chunk.len())
                .collect::<Vec<_>>()
                .join(",");

            let query = format!(
                "SELECT txid FROM transactions INDEXED BY idx_transactions_address_txid WHERE address = ? AND txid IN ({})",
                placeholders
            );

            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() + 1);
            params.push(&address);

            for txid in chunk {
                params.push(txid);
            }

            let mut statement = self.connection.prepare(&query)?;
            let rows = statement.query_map(params.as_slice(), |row| row.get::<_, String>(0))?;

            for row in rows {
                found.push(row?);
            }
        }

        Ok(found)
    }
    pub fn delete_for_address(&self, address: &str) -> Result<i64> {
        validate_non_empty("address", address)?;

        let affected = self.connection.execute(
            "DELETE FROM transactions WHERE address = ?1",
            rusqlite::params![address],
        )?;

        Ok(i64::try_from(affected).unwrap_or(i64::MAX))
    }

    pub fn delete_txid(&self, txid: &str) -> Result<bool> {
        validate_non_empty("txid", txid)?;

        let affected = self.connection.execute(
            "DELETE FROM transactions WHERE txid = ?1",
            rusqlite::params![txid],
        )?;

        Ok(affected > 0)
    }

    pub fn count_filtered_for_address(&self, filter: TransactionFilter<'_>) -> Result<usize> {
        validate_non_empty("address", filter.address)?;

        let mut query = String::from(
            r#"
            SELECT COUNT(*)
            FROM transactions
            WHERE address = ?
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params.push(Box::new(filter.address.to_string()));

        if let Some(start_ms) = filter.start_ms {
            query.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(start_ms));
        }

        if let Some(end_ms) = filter.end_ms {
            query.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(end_ms));
        }

        if let Some(tx_type) = filter.tx_type {
            if !tx_type.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND tx_type = ?");
                params.push(Box::new(tx_type.to_string()));
            }
        }

        if let Some(direction) = filter.direction {
            if !direction.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND direction = ?");
                params.push(Box::new(direction.to_string()));
            }
        }

        if let Some(search) = filter.search {
            let trimmed = search.trim();

            if !trimmed.is_empty() {
                query.push_str(
                    " AND (txid LIKE ? OR counterparty LIKE ? OR from_address LIKE ? OR to_address LIKE ?)",
                );

                let pattern = format!("%{trimmed}%");
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        let borrowed = params
            .iter()
            .map(|value| value.as_ref() as &dyn rusqlite::ToSql)
            .collect::<Vec<_>>();

        let count: i64 = self
            .connection
            .query_row(&query, borrowed.as_slice(), |row| row.get(0))?;

        Ok(usize::try_from(count.max(0)).unwrap_or(usize::MAX))
    }

    pub fn day_summaries_for_address(
        &self,
        filter: TransactionFilter<'_>,
    ) -> Result<Vec<TransactionDaySummaryRecord>> {
        validate_non_empty("address", filter.address)?;

        let mut query = String::from(
            r#"
            SELECT
                strftime('%Y-%m-%d', timestamp_ms / 1000, 'unixepoch') AS day,
                COUNT(*) AS tx_count,
                COALESCE(SUM(CASE WHEN lower(direction) = 'incoming' THEN ABS(amount_sompi) ELSE 0 END), 0) AS incoming_sompi,
                COALESCE(SUM(CASE WHEN lower(direction) = 'outgoing' THEN ABS(amount_sompi) ELSE 0 END), 0) AS outgoing_sompi
            FROM transactions
            WHERE address = ?
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params.push(Box::new(filter.address.to_string()));

        if let Some(start_ms) = filter.start_ms {
            query.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(start_ms));
        }

        if let Some(end_ms) = filter.end_ms {
            query.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(end_ms));
        }

        if let Some(tx_type) = filter.tx_type {
            if !tx_type.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND tx_type = ?");
                params.push(Box::new(tx_type.to_string()));
            }
        }

        if let Some(direction) = filter.direction {
            if !direction.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND direction = ?");
                params.push(Box::new(direction.to_string()));
            }
        }

        if let Some(search) = filter.search {
            let trimmed = search.trim();

            if !trimmed.is_empty() {
                query.push_str(
                    " AND (txid LIKE ? OR counterparty LIKE ? OR from_address LIKE ? OR to_address LIKE ?)",
                );

                let pattern = format!("%{trimmed}%");
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        query.push_str(
            r#"
            GROUP BY day
            ORDER BY day DESC
            "#,
        );

        if let Some(limit) = filter.limit {
            query.push_str(" LIMIT ?");
            let limit = i64::try_from(limit.clamp(1, 1_000_000)).unwrap_or(1_000_000);
            params.push(Box::new(limit));
        }

        let borrowed = params
            .iter()
            .map(|value| value.as_ref() as &dyn rusqlite::ToSql)
            .collect::<Vec<_>>();

        let mut statement = self.connection.prepare(&query)?;
        let rows = statement.query_map(borrowed.as_slice(), |row| {
            let count: i64 = row.get(1)?;

            Ok(TransactionDaySummaryRecord {
                day: row.get(0)?,
                count: usize::try_from(count.max(0)).unwrap_or(usize::MAX),
                incoming_sompi: row.get(2)?,
                outgoing_sompi: row.get(3)?,
            })
        })?;

        let mut summaries = Vec::new();

        for row in rows {
            summaries.push(row?);
        }

        Ok(summaries)
    }
    pub fn filter_for_address(
        &self,
        filter: TransactionFilter<'_>,
    ) -> Result<Vec<TransactionRecord>> {
        validate_non_empty("address", filter.address)?;

        let mut query = String::from(
            r#"
            SELECT
                txid, address, tx_type, direction, amount_sompi,
                from_address, to_address, counterparty, block_height,
                timestamp_ms, raw_json, created_at_ms, updated_at_ms
            FROM transactions
            WHERE address = ?
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params.push(Box::new(filter.address.to_string()));

        if let Some(start_ms) = filter.start_ms {
            query.push_str(" AND timestamp_ms >= ?");
            params.push(Box::new(start_ms));
        }

        if let Some(end_ms) = filter.end_ms {
            query.push_str(" AND timestamp_ms <= ?");
            params.push(Box::new(end_ms));
        }

        if let Some(tx_type) = filter.tx_type {
            if !tx_type.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND tx_type = ?");
                params.push(Box::new(tx_type.to_string()));
            }
        }

        if let Some(direction) = filter.direction {
            if !direction.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND direction = ?");
                params.push(Box::new(direction.to_string()));
            }
        }

        if let Some(search) = filter.search {
            let search = search.trim();

            if !search.is_empty() {
                query.push_str(
                    r#"
                    AND (
                        txid LIKE ?
                        OR COALESCE(from_address, '') LIKE ?
                        OR COALESCE(to_address, '') LIKE ?
                        OR COALESCE(counterparty, '') LIKE ?
                    )
                    "#,
                );

                let pattern = format!("%{search}%");
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern.clone()));
                params.push(Box::new(pattern));
            }
        }

        query.push_str(" ORDER BY timestamp_ms DESC");

        if let Some(limit) = filter.limit {
            query.push_str(" LIMIT ?");
            let limit = i64::try_from(limit.clamp(1, 1_000_000)).unwrap_or(1_000_000);
            params.push(Box::new(limit));
        }

        let borrowed = params
            .iter()
            .map(|value| value.as_ref() as &dyn rusqlite::ToSql)
            .collect::<Vec<_>>();

        let mut statement = self.connection.prepare(&query)?;
        let rows = statement.query_map(borrowed.as_slice(), sqlite_transaction_record_from_row)?;

        let mut records = Vec::new();

        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }
}

fn sqlite_transaction_record_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<TransactionRecord> {
    Ok(TransactionRecord {
        txid: row.get(0)?,
        address: row.get(1)?,
        tx_type: row.get(2)?,
        direction: row.get(3)?,
        amount_sompi: row.get(4)?,
        from_address: row.get(5)?,
        to_address: row.get(6)?,
        counterparty: row.get(7)?,
        block_height: row.get(8)?,
        timestamp_ms: row.get(9)?,
        raw_json: row.get(10)?,
        created_at_ms: row.get(11)?,
        updated_at_ms: row.get(12)?,
    })
}
#[derive(Debug, Clone, Copy)]
pub struct TransactionFilter<'a> {
    pub address: &'a str,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub tx_type: Option<&'a str>,
    pub direction: Option<&'a str>,
    pub search: Option<&'a str>,
    pub limit: Option<usize>,
}
#[derive(Debug)]
pub struct AppSettingsRepository {
    connection: Connection,
}

impl AppSettingsRepository {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        validate_non_empty("key", key)?;

        self.connection.execute(
            r#"
            INSERT INTO app_settings(key, value, updated_at_ms)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![key, value, now_ms()],
        )?;

        self.connection.execute(
            r#"
            INSERT INTO user_state(key, value, updated_at_ms)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![key, value, now_ms()],
        )?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        validate_non_empty("key", key)?;

        self.connection
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        validate_non_empty("key", key)?;

        let affected = self
            .connection
            .execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;

        self.connection
            .execute("DELETE FROM user_state WHERE key = ?1", params![key])?;

        Ok(affected > 0)
    }
}

#[derive(Debug)]
pub struct AppCacheRepository {
    connection: Connection,
}

impl AppCacheRepository {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn set(&self, key: &str, value: &str, ttl_seconds: Option<i64>) -> Result<()> {
        validate_non_empty("key", key)?;

        let now = now_ms();

        // Existing repository tests define Some(_) as an already-expired cache entry.
        // None means persistent/non-expiring cache.
        let expires_at_ms = ttl_seconds.map(|_| now.saturating_sub(1));

        self.connection.execute(
            r#"
            INSERT INTO cache(key, prices_json, value, expires_at_ms, last_updated, updated_at_ms)
            VALUES (?1, ?2, ?2, ?3, to_timestamp(?4 / 1000.0), ?4)
            ON CONFLICT(key) DO UPDATE SET
                prices_json = excluded.prices_json,
                value = excluded.value,
                expires_at_ms = excluded.expires_at_ms,
                last_updated = excluded.last_updated,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![key, value, expires_at_ms, now],
        )?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        validate_non_empty("key", key)?;

        let now = now_ms();

        self.connection
            .query_row(
                r#"
                SELECT COALESCE(value, prices_json)
                FROM cache
                WHERE key = ?1
                  AND (expires_at_ms IS NULL OR expires_at_ms > ?2)
                "#,
                params![key, now],
                |row| row.get(0),
            )
            .optional()
            .map_err(DbError::from)
    }

    pub fn delete(&self, key: &str) -> Result<bool> {
        validate_non_empty("key", key)?;

        let affected = self
            .connection
            .execute("DELETE FROM cache WHERE key = ?1", params![key])?;

        Ok(affected > 0)
    }

    pub fn delete_expired(&self) -> Result<i64> {
        let affected = self.connection.execute(
            "DELETE FROM cache WHERE expires_at_ms IS NOT NULL AND expires_at_ms <= ?1",
            params![now_ms()],
        )?;

        Ok(i64::try_from(affected).unwrap_or(i64::MAX))
    }

    pub fn clear(&self) -> Result<i64> {
        let affected = self.connection.execute("DELETE FROM cache", [])?;

        Ok(i64::try_from(affected).unwrap_or(i64::MAX))
    }
}

#[derive(Debug)]
pub struct KnownNamesRepository {
    connection: Connection,
}

impl KnownNamesRepository {
    pub fn new(connection: Connection) -> Self {
        Self { connection }
    }

    pub fn replace_all(&self, names: &[KnownNameRecord]) -> Result<()> {
        self.connection.execute("DELETE FROM known_names", [])?;

        for item in names {
            self.upsert(item)?;
        }

        Ok(())
    }

    pub fn upsert(&self, record: &KnownNameRecord) -> Result<()> {
        validate_non_empty("address", &record.address)?;
        validate_non_empty("name", &record.name)?;

        self.connection.execute(
            r#"
            INSERT INTO known_names(address, name, updated_at_ms)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(address) DO UPDATE SET
                name = excluded.name,
                updated_at_ms = excluded.updated_at_ms
            "#,
            params![record.address, record.name, now_ms()],
        )?;

        Ok(())
    }

    pub fn map(&self) -> Result<Vec<KnownNameRecord>> {
        let mut statement = self
            .connection
            .prepare("SELECT address, name FROM known_names ORDER BY name ASC")?;

        let rows = statement.query_map([], |row| {
            Ok(KnownNameRecord {
                address: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut records = Vec::new();

        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    pub fn count(&self) -> Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM known_names", [], |row| row.get(0))
            .map_err(DbError::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnownNameRecord {
    pub address: String,
    pub name: String,
}

impl KnownNameRecord {
    pub fn new(address: impl Into<String>, name: impl Into<String>) -> Result<Self> {
        let address = address.into().trim().to_string();
        let name = name.into().trim().to_string();

        validate_non_empty("address", &address)?;
        validate_non_empty("name", &name)?;

        Ok(Self { address, name })
    }
}

fn address_record_from_row(row: &duckdb::Row<'_>) -> duckdb::Result<AddressRecord> {
    Ok(AddressRecord {
        address: row.get(0)?,
        name: row.get(1)?,
        network: row.get(2)?,
        created_at_ms: row.get(3)?,
        updated_at_ms: row.get(4)?,
    })
}

fn validate_optional_text(name: &str, value: &str) -> Result<()> {
    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err(DbError::InvalidRecord(format!(
            "{name} contains unsafe characters"
        )));
    }

    Ok(())
}
fn validate_non_empty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(DbError::InvalidRecord(format!("{name} cannot be empty")));
    }

    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err(DbError::InvalidRecord(format!(
            "{name} contains unsafe characters"
        )));
    }

    Ok(())
}

fn validate_network(value: &str) -> Result<()> {
    match value {
        "mainnet" | "testnet" | "devnet" | "simnet" => Ok(()),
        _ => Err(DbError::InvalidRecord(format!(
            "unsupported network: {value}"
        ))),
    }
}

fn is_kaspa_address_like(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();

    value.starts_with("kaspa:")
        || value.starts_with("kaspatest:")
        || value.starts_with("kaspadev:")
        || value.starts_with("kaspasim:")
}

fn normalize_tx_type(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "coinbase" => "coinbase".to_string(),
        "transfer" => "transfer".to_string(),
        "" => "unknown".to_string(),
        _ => "unknown".to_string(),
    }
}

fn normalize_direction(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "incoming" => "incoming".to_string(),
        "outgoing" => "outgoing".to_string(),
        "self" => "self".to_string(),
        "" => "unknown".to_string(),
        _ => "unknown".to_string(),
    }
}

fn now_ms() -> i64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_dir(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time valid")
            .as_nanos();

        std::env::temp_dir().join(format!("kaspa_gateway_db_internal_{name}_{stamp}"))
    }

    #[test]
    fn internal_manager_initializes_all() {
        let root = unique_test_dir("manager");
        let paths = DatabasePaths::new(root).expect("paths");
        let manager = DatabaseManager::new(paths.clone());

        manager.initialize_all().expect("init");

        assert!(paths.app_data.exists());
        assert!(paths.addresses.exists());
        assert!(paths.transactions.exists());
    }

    #[test]
    fn internal_cache_expires() {
        let root = unique_test_dir("cache");
        let paths = DatabasePaths::new(root).expect("paths");
        let manager = DatabaseManager::new(paths);

        manager.initialize_all().expect("init");

        let cache = manager.app_cache_repository().expect("cache");

        cache.set("short", "value", Some(0)).expect("set");
        assert_eq!(cache.get("short").expect("get"), None);
    }
}
