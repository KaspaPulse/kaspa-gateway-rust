use crate::e2e_native_webdriver::{NativeWebDriverHarness, WebDriverSession};
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEFAULT_WEBDRIVER_PORT: u16 = 4487;
const DEFAULT_ROWS: usize = 1_000_000;
const RESULT_LIMIT: usize = 100;
const QUERY_BUDGET_MS: u64 = 5_000;
const MAX_HEARTBEAT_GAP_MS: f64 = 250.0;
const ADDRESS: &str = "kaspa:qq2avyvncscg5dtsk8u4uwjhlr3799dhaqj8k9y6q5y9hpwfxjy6u00pep7vg";
const COMMON_SEARCH: &str = "qpsender";

#[derive(Debug, Clone)]
struct Args {
    app_binary: PathBuf,
    output_directory: PathBuf,
    data_directory: PathBuf,
    rows: usize,
    webdriver_port: u16,
    window_label: String,
    startup_timeout: Duration,
}

#[derive(Debug, Clone)]
struct HeartbeatObservation {
    value: Value,
    elapsed_ms: f64,
    ticks: u64,
    max_gap_ms: f64,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut app_binary = None;
    let mut output_directory = None;
    let mut data_directory = None;
    let mut rows = DEFAULT_ROWS;
    let mut webdriver_port = DEFAULT_WEBDRIVER_PORT;
    let mut window_label = "main".to_owned();
    let mut startup_timeout = Duration::from_secs(120);

    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--app-binary" => app_binary = Some(PathBuf::from(value)),
            "--output-directory" => output_directory = Some(PathBuf::from(value)),
            "--data-directory" => data_directory = Some(PathBuf::from(value)),
            "--rows" => {
                rows = value
                    .parse::<usize>()
                    .ok()
                    .filter(|rows| (1..=1_000_000).contains(rows))
                    .ok_or_else(|| format!("--rows must be in 1..=1000000; got {value}"))?;
            }
            "--port" => {
                webdriver_port = value
                    .parse::<u16>()
                    .ok()
                    .filter(|port| *port != 0)
                    .ok_or_else(|| format!("invalid non-zero WebDriver port: {value}"))?;
            }
            "--window-label" => {
                if value.trim().is_empty() {
                    return Err("--window-label must not be empty".to_owned());
                }
                window_label = value;
            }
            "--startup-timeout-seconds" => {
                let seconds = value
                    .parse::<u64>()
                    .ok()
                    .filter(|seconds| *seconds > 0 && *seconds <= 300)
                    .ok_or_else(|| format!("invalid startup timeout seconds: {value}"))?;
                startup_timeout = Duration::from_secs(seconds);
            }
            _ => {
                return Err(format!(
                    "unknown Explorer UI responsiveness argument: {flag}"
                ));
            }
        }
    }

    let app_binary = app_binary.ok_or_else(|| "--app-binary <path> is required".to_owned())?;
    let output_directory =
        output_directory.ok_or_else(|| "--output-directory <path> is required".to_owned())?;
    let data_directory =
        data_directory.ok_or_else(|| "--data-directory <path> is required".to_owned())?;
    if !data_directory.is_absolute() {
        return Err("--data-directory must be absolute".to_owned());
    }

    Ok(Args {
        app_binary,
        output_directory,
        data_directory,
        rows,
        webdriver_port,
        window_label,
        startup_timeout,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let rendered = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(path, rendered)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn txid_for(value: u64) -> String {
    format!("{value:08x}{value:056x}")
}

fn verify_isolation(args: &Args) -> Result<(), String> {
    let inherited = std::env::var_os("KASPA_GATEWAY_DATA_DIR")
        .map(PathBuf::from)
        .ok_or("KASPA_GATEWAY_DATA_DIR must be explicitly set for responsiveness qualification")?;
    if inherited != args.data_directory {
        return Err(format!(
            "KASPA_GATEWAY_DATA_DIR mismatch: expected={}; actual={}",
            args.data_directory.display(),
            inherited.display()
        ));
    }
    Ok(())
}

fn seed_large_fixture(args: &Args) -> Result<Value, String> {
    verify_isolation(args)?;
    let database_root = args.data_directory.join("databases");
    let paths = DatabasePaths::new(&database_root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths.clone());
    manager
        .initialize_all()
        .map_err(|error| error.to_string())?;
    drop(
        manager
            .transactions_repository()
            .map_err(|error| error.to_string())?,
    );

    let connection = rusqlite::Connection::open(&paths.transactions_sqlite)
        .map_err(|error| error.to_string())?;
    let existing: i64 = connection
        .query_row("SELECT COUNT(*) FROM transactions", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if existing != 0 {
        return Err(format!(
            "isolated responsiveness database is not empty: existing_rows={existing}"
        ));
    }

    connection
        .execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF;")
        .map_err(|error| error.to_string())?;

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

    let started = Instant::now();
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|error| error.to_string())?;
    connection
        .execute(
            &insert_transactions,
            rusqlite::params![args.rows as i64, ADDRESS],
        )
        .map_err(|error| error.to_string())?;
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
        .map_err(|error| error.to_string())?;
    let seed_ms = started.elapsed().as_millis();

    connection
        .execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
        .map_err(|error| error.to_string())?;

    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM address_transactions WHERE address=?1",
            [ADDRESS],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if count != args.rows as i64 {
        return Err(format!(
            "seeded relation count mismatch: expected={}; actual={count}",
            args.rows
        ));
    }

    let db_bytes = fs::metadata(&paths.transactions_sqlite)
        .map(|metadata| metadata.len())
        .unwrap_or_default();

    Ok(json!({
        "rows": args.rows,
        "address": ADDRESS,
        "seedMs": seed_ms,
        "database": paths.transactions_sqlite,
        "databaseBytes": db_bytes,
        "journalModeRestored": "WAL",
        "synchronousRestored": "NORMAL"
    }))
}

async fn invoke_with_heartbeat(
    session: &WebDriverSession,
    command: &str,
    payload: Value,
) -> Result<HeartbeatObservation, String> {
    let result = session
        .execute_async(
            r#"
const commandName = arguments[0];
const payload = arguments[1];
const done = arguments[arguments.length - 1];
const invoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
if (typeof invoke !== "function") {
  done({ ok: false, error: "Tauri invoke API is unavailable" });
} else {
  const started = performance.now();
  let last = started;
  let maxGapMs = 0;
  let ticks = 0;
  const timer = setInterval(() => {
    const now = performance.now();
    maxGapMs = Math.max(maxGapMs, now - last);
    last = now;
    ticks += 1;
  }, 16);

  Promise.resolve(invoke(commandName, payload))
    .then((value) => {
      const ended = performance.now();
      clearInterval(timer);
      maxGapMs = Math.max(maxGapMs, ended - last);
      done({
        ok: true,
        value,
        elapsedMs: ended - started,
        ticks,
        maxGapMs
      });
    })
    .catch((error) => {
      const ended = performance.now();
      clearInterval(timer);
      maxGapMs = Math.max(maxGapMs, ended - last);
      done({
        ok: false,
        error: error && error.message ? error.message : String(error),
        elapsedMs: ended - started,
        ticks,
        maxGapMs
      });
    });
}
"#,
            vec![json!(command), payload],
        )
        .await?;

    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "{command} failed: {}",
            result
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("unknown Tauri invoke error")
        ));
    }

    let value = result
        .get("value")
        .cloned()
        .ok_or_else(|| format!("{command} returned no value"))?;
    let elapsed_ms = result
        .get("elapsedMs")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{command} missing elapsedMs: {result}"))?;
    let ticks = result
        .get("ticks")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{command} missing ticks: {result}"))?;
    let max_gap_ms = result
        .get("maxGapMs")
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("{command} missing maxGapMs: {result}"))?;

    Ok(HeartbeatObservation {
        value,
        elapsed_ms,
        ticks,
        max_gap_ms,
    })
}

fn row_count(groups: &Value) -> Result<usize, String> {
    let groups = groups
        .as_array()
        .ok_or_else(|| format!("grouped response is not an array: {groups}"))?;
    Ok(groups
        .iter()
        .map(|group| {
            group
                .get("transactions")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or_default()
        })
        .sum())
}

fn assert_responsive(
    label: &str,
    observation: &HeartbeatObservation,
    budget_ms: u64,
) -> Result<(), String> {
    if observation.elapsed_ms > budget_ms as f64 {
        return Err(format!(
            "{label} exceeded query budget: elapsed_ms={:.1}; budget_ms={budget_ms}",
            observation.elapsed_ms
        ));
    }
    if observation.max_gap_ms > MAX_HEARTBEAT_GAP_MS {
        return Err(format!(
            "{label} blocked WebView heartbeat: max_gap_ms={:.1}; budget_ms={MAX_HEARTBEAT_GAP_MS}",
            observation.max_gap_ms
        ));
    }
    Ok(())
}

fn list_request(search_query: Option<&str>) -> Value {
    json!({
        "request": {
            "address": ADDRESS,
            "start_ts": null,
            "end_ts": null,
            "tx_type": null,
            "direction": null,
            "search_query": search_query,
            "limit": RESULT_LIMIT,
            "offset": 0
        }
    })
}

async fn run_native(root: &Path, args: &Args, fixture: Value) -> Result<Value, String> {
    let mut harness = NativeWebDriverHarness::launch(
        root,
        &args.app_binary,
        args.webdriver_port,
        &args.window_label,
        args.startup_timeout,
    )
    .await?;

    let result = async {
        let session = harness.session();
        let baseline = session
            .execute_sync(
                "return { title: document.title, readyState: document.readyState, visible: document.visibilityState };",
                Vec::new(),
            )
            .await?;

        let exact_txid = txid_for(0);
        let exact = invoke_with_heartbeat(
            session,
            "explorer_list_transactions_grouped_rust",
            list_request(Some(&exact_txid)),
        )
        .await?;
        assert_responsive("exact_txid_search", &exact, QUERY_BUDGET_MS)?;
        let exact_rows = row_count(&exact.value)?;
        if exact_rows != 1 {
            return Err(format!(
                "exact txid search expected 1 row; observed {exact_rows}"
            ));
        }

        let common = invoke_with_heartbeat(
            session,
            "explorer_list_transactions_grouped_rust",
            list_request(Some(COMMON_SEARCH)),
        )
        .await?;
        assert_responsive("common_substring_search", &common, QUERY_BUDGET_MS)?;
        let common_rows = row_count(&common.value)?;
        if common_rows != RESULT_LIMIT {
            return Err(format!(
                "common substring search expected bounded {RESULT_LIMIT} rows; observed {common_rows}"
            ));
        }

        let post = session
            .execute_sync(
                "return { title: document.title, readyState: document.readyState, visible: document.visibilityState, bodyBytes: document.body ? document.body.innerHTML.length : 0 };",
                Vec::new(),
            )
            .await?;
        if post.get("readyState").and_then(Value::as_str) != Some("complete") {
            return Err(format!("WebView not complete after stress queries: {post}"));
        }

        Ok(json!({
            "passed": true,
            "fixture": fixture,
            "appBinary": args.app_binary,
            "webdriverPort": args.webdriver_port,
            "windowLabel": args.window_label,
            "baseline": baseline,
            "budgets": {
                "queryMs": QUERY_BUDGET_MS,
                "maxHeartbeatGapMs": MAX_HEARTBEAT_GAP_MS,
                "resultLimit": RESULT_LIMIT
            },
            "exactTxidSearch": {
                "query": exact_txid,
                "rows": exact_rows,
                "elapsedMs": exact.elapsed_ms,
                "heartbeatTicks": exact.ticks,
                "maxHeartbeatGapMs": exact.max_gap_ms
            },
            "commonSubstringSearch": {
                "query": COMMON_SEARCH,
                "rows": common_rows,
                "elapsedMs": common.elapsed_ms,
                "heartbeatTicks": common.ticks,
                "maxHeartbeatGapMs": common.max_gap_ms
            },
            "post": post
        }))
    }
    .await;

    let shutdown = harness.shutdown().await;
    match (result, shutdown) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(shutdown_error)) => Err(format!(
            "{error}; app/session shutdown failed: {shutdown_error}"
        )),
    }
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<String, String> {
    let args = parse_args(args)?;
    verify_isolation(&args)?;
    fs::create_dir_all(&args.output_directory).map_err(|error| {
        format!(
            "failed to create output directory {}: {error}",
            args.output_directory.display()
        )
    })?;
    fs::create_dir_all(&args.data_directory).map_err(|error| {
        format!(
            "failed to create isolated data directory {}: {error}",
            args.data_directory.display()
        )
    })?;

    let fixture = seed_large_fixture(&args)?;
    write_json(
        &args.output_directory.join("explorer-ui-fixture.json"),
        &fixture,
    )?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to create Explorer responsiveness runtime: {error}"))?;
    let result = runtime.block_on(run_native(root, &args, fixture))?;
    write_json(
        &args
            .output_directory
            .join("explorer-ui-responsiveness-result.json"),
        &result,
    )?;
    serde_json::to_string_pretty(&result)
        .map_err(|error| format!("failed to serialize responsiveness result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_is_fail_closed_and_one_million_bounded() {
        let mut missing = Vec::<String>::new().into_iter();
        assert!(parse_args(&mut missing).is_err());

        let absolute = std::env::current_dir().unwrap().join("isolated");
        let mut args = vec![
            "--app-binary".to_owned(),
            "app.exe".to_owned(),
            "--output-directory".to_owned(),
            "out".to_owned(),
            "--data-directory".to_owned(),
            absolute.to_string_lossy().into_owned(),
        ]
        .into_iter();
        let parsed = parse_args(&mut args).unwrap();
        assert_eq!(parsed.rows, 1_000_000);
        assert_eq!(parsed.webdriver_port, DEFAULT_WEBDRIVER_PORT);
    }

    #[test]
    fn responsiveness_budget_rejects_query_or_heartbeat_stall() {
        let value = json!([]);
        let pass = HeartbeatObservation {
            value: value.clone(),
            elapsed_ms: 1_200.0,
            ticks: 50,
            max_gap_ms: 40.0,
        };
        assert!(assert_responsive("pass", &pass, QUERY_BUDGET_MS).is_ok());

        let slow = HeartbeatObservation {
            value: value.clone(),
            elapsed_ms: 5_100.0,
            ticks: 100,
            max_gap_ms: 30.0,
        };
        assert!(assert_responsive("slow", &slow, QUERY_BUDGET_MS).is_err());

        let frozen = HeartbeatObservation {
            value,
            elapsed_ms: 900.0,
            ticks: 0,
            max_gap_ms: 900.0,
        };
        assert!(assert_responsive("frozen", &frozen, QUERY_BUDGET_MS).is_err());
    }
}
