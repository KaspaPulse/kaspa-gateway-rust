use kaspa_gateway_db::{DatabaseManager, DatabasePaths, TransactionFilter};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const ADDRESS: &str = "kaspa:qq2avyvncscg5dtsk8u4uwjhlr3799dhaqj8k9y6q5y9hpwfxjy6u00pep7vg";
const COUNTERPARTY: &str = "kaspa:qpsender";

struct CleanupDir(PathBuf);

impl Drop for CleanupDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn unique_root(rows: usize) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kgw_large_data_{}_{}_{}",
        std::process::id(),
        rows,
        stamp
    ))
}

fn txid_for(value: u64) -> String {
    format!("{value:08x}{value:056x}")
}

fn seed_rows(connection: &rusqlite::Connection, rows: usize) {
    const DIGITS: &str = "WITH digits(d) AS (VALUES(0),(1),(2),(3),(4),(5),(6),(7),(8),(9)),
         nums(n) AS (
           SELECT a.d + 10*b.d + 100*c.d + 1000*d.d + 10000*e.d + 100000*f.d
           FROM digits a
           CROSS JOIN digits b
           CROSS JOIN digits c
           CROSS JOIN digits d
           CROSS JOIN digits e
           CROSS JOIN digits f
           LIMIT ?1
         ) ";

    connection
        .execute_batch("BEGIN IMMEDIATE")
        .expect("begin seed");

    let insert_transactions = format!(
        "{DIGITS}
         INSERT INTO transactions(
           txid,address,tx_type,direction,amount_sompi,
           from_address,to_address,counterparty,block_height,
           timestamp_ms,raw_json,created_at_ms,updated_at_ms
         )
         SELECT
           printf('%08x%056x', n, n), ?2, 'transfer',
           CASE WHEN n % 2 = 0 THEN 'incoming' ELSE 'outgoing' END,
           (n % 100000) + 1,
           CASE WHEN n % 2 = 0 THEN 'kaspa:qpsender' ELSE ?2 END,
           CASE WHEN n % 2 = 0 THEN ?2 ELSE 'kaspa:qpreceiver' END,
           CASE WHEN n % 2 = 0 THEN 'kaspa:qpsender' ELSE 'kaspa:qpreceiver' END,
           n,
           1700000000000 + n * 60000,
           '{{}}',
           1700000000000 + n,
           1700000000000 + n
         FROM nums"
    );

    connection
        .execute(
            &insert_transactions,
            rusqlite::params![rows as i64, ADDRESS],
        )
        .expect("seed canonical transactions");

    connection
        .execute_batch(
            "INSERT INTO address_transactions(
               address,txid,tx_type,direction,amount_sompi,
               from_address,to_address,counterparty,created_at_ms,updated_at_ms,timestamp_ms
             )
             SELECT
               address,txid,tx_type,direction,amount_sompi,
               from_address,to_address,counterparty,created_at_ms,updated_at_ms,timestamp_ms
             FROM transactions;
             COMMIT;",
        )
        .expect("seed relations");
}

fn query_plan(connection: &rusqlite::Connection, sql: &str) -> Vec<String> {
    let mut statement = connection.prepare(sql).expect("prepare explain");
    statement
        .query_map([], |row| row.get::<_, String>(3))
        .expect("query plan rows")
        .map(|row| row.expect("plan row"))
        .collect()
}

fn assert_no_full_scan_or_temp_sort(plan: &[String], label: &str) {
    assert!(
        plan.iter().all(|step| !step.contains("SCAN r")),
        "{label} unexpectedly scans address relations: {plan:?}"
    );
    assert!(
        plan.iter()
            .all(|step| !step.contains("USE TEMP B-TREE FOR ORDER BY")),
        "{label} unexpectedly uses temp sort: {plan:?}"
    );
}

fn run_scale(rows: usize) {
    let root = unique_root(rows);
    let _cleanup = CleanupDir(root.clone());
    let paths = DatabasePaths::new(&root).expect("paths");
    let manager = DatabaseManager::new(paths.clone());
    manager.initialize_all().expect("initialize schemas");
    drop(
        manager
            .transactions_repository()
            .expect("initialize SQLite transaction schema"),
    );

    let raw = rusqlite::Connection::open(&paths.transactions_sqlite).expect("raw sqlite");

    // Large-data qualification measures the product queries, not SQLite's WAL
    // cost of constructing a synthetic fixture. Storage durability is covered
    // separately by the fault-matrix tests.
    raw.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;")
        .expect("fast deterministic fixture pragmas");

    let seed_started = Instant::now();
    seed_rows(&raw, rows);
    let seed_elapsed = seed_started.elapsed();

    raw.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .expect("restore product journal mode");

    let repository = manager.transactions_repository().expect("repository");

    let count_started = Instant::now();
    let count = repository.count_for_address(ADDRESS).expect("count");
    let count_elapsed = count_started.elapsed();
    assert_eq!(count, rows as i64);

    let list_started = Instant::now();
    let newest = repository
        .list_for_address(ADDRESS, 100)
        .expect("newest rows");
    let list_elapsed = list_started.elapsed();
    assert_eq!(newest.len(), rows.min(100));

    let exact_txid = txid_for(0);
    let exact_started = Instant::now();
    let exact = repository
        .filter_for_address(TransactionFilter {
            address: ADDRESS,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: Some(&exact_txid),
            limit: Some(100),
        })
        .expect("exact txid search");
    let exact_elapsed = exact_started.elapsed();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].txid, exact_txid);

    let prefix = &exact_txid[..8];
    let prefix_started = Instant::now();
    let prefix_rows = repository
        .filter_for_address(TransactionFilter {
            address: ADDRESS,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: Some(prefix),
            limit: Some(100),
        })
        .expect("txid prefix search");
    let prefix_elapsed = prefix_started.elapsed();
    assert_eq!(prefix_rows.len(), 1);
    assert_eq!(prefix_rows[0].txid, exact_txid);

    let address_started = Instant::now();
    let address_rows = repository
        .filter_for_address(TransactionFilter {
            address: ADDRESS,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: Some(COUNTERPARTY),
            limit: Some(100),
        })
        .expect("exact address search");
    let address_elapsed = address_started.elapsed();
    assert!(!address_rows.is_empty());

    let substring_started = Instant::now();
    let substring_rows = repository
        .filter_for_address(TransactionFilter {
            address: ADDRESS,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: Some("kgw-definitely-not-present-fragment"),
            limit: Some(100),
        })
        .expect("free substring miss search");
    let substring_elapsed = substring_started.elapsed();
    assert!(substring_rows.is_empty());

    let common_substring_started = Instant::now();
    let common_substring_rows = repository
        .filter_for_address(TransactionFilter {
            address: ADDRESS,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: Some("qpsender"),
            limit: Some(100),
        })
        .expect("common free substring search");
    let common_substring_elapsed = common_substring_started.elapsed();
    assert_eq!(common_substring_rows.len(), rows.min(100));

    let day_started = Instant::now();
    let days = repository
        .day_summaries_for_address(TransactionFilter {
            address: ADDRESS,
            start_ms: None,
            end_ms: None,
            tx_type: None,
            direction: None,
            search: None,
            limit: Some(10_000),
        })
        .expect("day summaries");
    let day_elapsed = day_started.elapsed();
    assert!(!days.is_empty());

    let plan_list = query_plan(
        &raw,
        "EXPLAIN QUERY PLAN
         SELECT t.txid
         FROM address_transactions r INDEXED BY idx_address_transactions_address_time
         JOIN transactions t ON t.txid=r.txid
         WHERE r.address='kaspa:qq2avyvncscg5dtsk8u4uwjhlr3799dhaqj8k9y6q5y9hpwfxjy6u00pep7vg'
         ORDER BY r.timestamp_ms DESC
         LIMIT 100",
    );

    let plan_exact = query_plan(
        &raw,
        &format!(
            "EXPLAIN QUERY PLAN
             SELECT t.txid
             FROM address_transactions r
             JOIN transactions t ON t.txid=r.txid
             WHERE r.address='{ADDRESS}'
               AND r.txid='{exact_txid}'
             ORDER BY r.timestamp_ms DESC
             LIMIT 100"
        ),
    );

    let plan_substring = query_plan(
        &raw,
        &format!(
            "EXPLAIN QUERY PLAN
             SELECT rowid
             FROM address_transactions_fts
             WHERE address='{ADDRESS}'
               AND address_transactions_fts MATCH '\"kgw-definitely-not-present-fragment\"'"
        ),
    );

    assert_no_full_scan_or_temp_sort(&plan_list, "bounded list");
    assert_no_full_scan_or_temp_sort(&plan_exact, "exact txid");
    assert!(
        plan_list
            .iter()
            .any(|step| step.contains("idx_address_transactions_address_time")),
        "bounded list must use address/time index: {plan_list:?}"
    );
    assert!(
        plan_exact
            .iter()
            .any(|step| step.contains("address_transactions") && step.contains("txid")),
        "exact txid must use address/txid key: {plan_exact:?}"
    );
    assert!(
        plan_substring
            .iter()
            .any(|step| step.contains("VIRTUAL TABLE INDEX")),
        "free substring search must use the FTS5 virtual index: {plan_substring:?}"
    );

    let file_size = std::fs::metadata(&paths.transactions_sqlite)
        .map(|meta| meta.len())
        .unwrap_or(0);

    println!(
        "KGW_LARGE_DATA rows={} seed_ms={} count_ms={} list100_ms={} exact_txid_ms={} prefix_ms={} exact_address_ms={} substring_miss_ms={} substring_common_ms={} day_summary_ms={} days={} db_bytes={} plan_list={:?} plan_exact={:?}",
        rows,
        seed_elapsed.as_millis(),
        count_elapsed.as_millis(),
        list_elapsed.as_millis(),
        exact_elapsed.as_millis(),
        prefix_elapsed.as_millis(),
        address_elapsed.as_millis(),
        substring_elapsed.as_millis(),
        common_substring_elapsed.as_millis(),
        day_elapsed.as_millis(),
        days.len(),
        file_size,
        plan_list,
        plan_exact
    );

    assert!(
        count_elapsed < Duration::from_secs(5),
        "count is not interactive at {rows} rows"
    );
    assert!(
        list_elapsed < Duration::from_secs(5),
        "bounded list is not interactive at {rows} rows"
    );
    assert!(
        exact_elapsed < Duration::from_secs(5),
        "exact txid search is not interactive at {rows} rows"
    );
    assert!(
        prefix_elapsed < Duration::from_secs(5),
        "txid prefix search is not interactive at {rows} rows"
    );
    assert!(
        address_elapsed < Duration::from_secs(5),
        "exact address search is not interactive at {rows} rows"
    );
    assert!(
        substring_elapsed < Duration::from_secs(5),
        "free substring miss is not interactive at {rows} rows"
    );
    assert!(
        common_substring_elapsed < Duration::from_secs(5),
        "common free substring search is not interactive at {rows} rows"
    );
    assert!(
        day_elapsed < Duration::from_secs(10),
        "day summary generation exceeded release budget at {rows} rows"
    );

    drop(repository);
    drop(raw);
}

#[test]
#[ignore = "release-admission large-data qualification; run explicitly"]
fn explorer_large_data_release_admission_matrix() {
    for rows in [1_000_usize, 10_000, 100_000, 1_000_000] {
        run_scale(rows);
    }
}

#[test]
#[ignore = "release-admission 1M focused qualification; run explicitly"]
fn explorer_large_data_1m_focused() {
    run_scale(1_000_000);
}
