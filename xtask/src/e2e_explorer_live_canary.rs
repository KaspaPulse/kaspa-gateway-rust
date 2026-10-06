use crate::e2e_native_webdriver::{NativeWebDriverHarness, WebDriverSession};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_WEBDRIVER_PORT: u16 = 4485;
const DEFAULT_ADDRESS: &str = "kaspa:qz0yqq8z3twwgg7lq2mjzg6w4edqys45w2wslz7tym2tc6s84580vvx9zr44g";

#[derive(Debug, Clone)]
struct Args {
    app_binary: PathBuf,
    output_directory: PathBuf,
    data_directory: PathBuf,
    address: String,
    webdriver_port: u16,
    window_label: String,
    startup_timeout: Duration,
}

fn parse_args(args: &mut impl Iterator<Item = String>) -> Result<Args, String> {
    let mut app_binary = None;
    let mut output_directory = None;
    let mut data_directory = None;
    let mut address = DEFAULT_ADDRESS.to_owned();
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
            "--address" => {
                if value.trim().is_empty() {
                    return Err("--address must not be empty".to_owned());
                }
                address = value;
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
            _ => return Err(format!("unknown explorer live canary argument: {flag}")),
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
        address,
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

async fn invoke(
    session: &WebDriverSession,
    command: &str,
    payload: Value,
) -> Result<Value, String> {
    let result = session
        .execute_async(
            r#"
const commandName = arguments[0];
const payload = arguments[1];
const done = arguments[arguments.length - 1];
try {
  const tauriInvoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.invoke;
  if (typeof tauriInvoke !== "function") {
    done({ ok: false, error: "Tauri invoke API is unavailable" });
  } else {
    Promise.resolve(tauriInvoke(commandName, payload))
      .then((value) => done({ ok: true, value }))
      .catch((error) => done({
        ok: false,
        error: error && error.message ? error.message : String(error),
      }));
  }
} catch (error) {
  done({
    ok: false,
    error: error && error.message ? error.message : String(error),
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
    result
        .get("value")
        .cloned()
        .ok_or_else(|| format!("{command} returned no value"))
}

fn assert_address(value: &Value, expected: &str, label: &str) -> Result<(), String> {
    let actual = value
        .get("address")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} is missing address: {value}"))?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!(
            "{label} address mismatch: expected={expected}; actual={actual}"
        ));
    }
    Ok(())
}

fn grouped_rows<'a>(groups: &'a Value, expected: &str) -> Result<Vec<&'a Value>, String> {
    let groups = groups
        .as_array()
        .ok_or_else(|| format!("grouped transaction response is not an array: {groups}"))?;
    let rows = groups
        .iter()
        .flat_map(|group| {
            group
                .get("transactions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .collect::<Vec<_>>();
    if rows.is_empty() || rows.len() > 10 {
        return Err(format!(
            "grouped read must contain 1..=10 real rows; observed {}",
            rows.len()
        ));
    }
    for row in &rows {
        assert_address(row, expected, "explorer_list_transactions_grouped_rust.row")?;
    }
    Ok(rows)
}

fn first_grouped_txid(groups: &Value, expected: &str) -> Result<String, String> {
    let rows = grouped_rows(groups, expected)?;
    rows[0]
        .get("txid")
        .and_then(Value::as_str)
        .filter(|txid| !txid.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("first grouped row is missing txid: {}", rows[0]))
}

fn grouped_contains_exact_txid(
    groups: &Value,
    expected_address: &str,
    expected_txid: &str,
) -> Result<bool, String> {
    Ok(grouped_rows(groups, expected_address)?.iter().any(|row| {
        row.get("txid")
            .and_then(Value::as_str)
            .is_some_and(|txid| txid.eq_ignore_ascii_case(expected_txid))
    }))
}

fn verify_isolation(args: &Args) -> Result<(), String> {
    let inherited = std::env::var_os("KASPA_GATEWAY_DATA_DIR")
        .map(PathBuf::from)
        .ok_or("KASPA_GATEWAY_DATA_DIR must be explicitly set for the live canary")?;
    if inherited != args.data_directory {
        return Err(format!(
            "KASPA_GATEWAY_DATA_DIR mismatch: expected={}; actual={}",
            args.data_directory.display(),
            inherited.display()
        ));
    }
    Ok(())
}

async fn run_native(root: &Path, args: &Args) -> Result<Value, String> {
    verify_isolation(args)?;
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

        let balance = invoke(
            session,
            "explorer_fetch_balance",
            json!({ "address": args.address }),
        )
        .await?;
        assert_address(&balance, &args.address, "explorer_fetch_balance")?;
        if balance
            .get("balance_sompi")
            .and_then(Value::as_u64)
            .is_none()
        {
            return Err(format!("balance_sompi is missing or invalid: {balance}"));
        }

        let sync = invoke(
            session,
            "explorer_transactions",
            json!({
                "request": {
                    "address": args.address,
                    "start_ts": null,
                    "end_ts": null,
                    "force": false,
                    "page_limit": 10,
                    "max_pages": 1,
                    "request_id": "op23-real-explorer-canary",
                    "tx_type": null,
                    "direction": null,
                    "search_query": null
                }
            }),
        )
        .await?;

        let summary = sync
            .get("summary")
            .ok_or_else(|| format!("explorer_transactions summary missing: {sync}"))?;
        assert_address(summary, &args.address, "explorer_transactions.summary")?;
        let pages = summary
            .get("pages")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("summary.pages missing: {summary}"))?;
        if pages > 1 {
            return Err(format!(
                "bounded canary exceeded max_pages=1: pages={pages}"
            ));
        }
        let db_rows_after = sync
            .get("db_rows_after")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("db_rows_after missing: {sync}"))?;
        if db_rows_after == 0 {
            return Err("live canary fetched no persistent Explorer rows".to_owned());
        }

        let grouped = invoke(
            session,
            "explorer_list_transactions_grouped_rust",
            json!({
                "request": {
                    "address": args.address,
                    "start_ts": null,
                    "end_ts": null,
                    "tx_type": null,
                    "direction": null,
                    "search_query": null,
                    "limit": 10,
                    "offset": 0
                }
            }),
        )
        .await?;
        let txid = first_grouped_txid(&grouped, &args.address)?;

        let search = invoke(
            session,
            "explorer_list_transactions_grouped_rust",
            json!({
                "request": {
                    "address": args.address,
                    "start_ts": null,
                    "end_ts": null,
                    "tx_type": null,
                    "direction": null,
                    "search_query": txid,
                    "limit": 10,
                    "offset": 0
                }
            }),
        )
        .await?;
        if !grouped_contains_exact_txid(&search, &args.address, &txid)? {
            return Err(format!(
                "exact transaction search did not return txid={txid}"
            ));
        }

        Ok(json!({
            "passed": true,
            "appBinary": args.app_binary,
            "webdriverPort": args.webdriver_port,
            "windowLabel": args.window_label,
            "dataDirectory": args.data_directory,
            "address": args.address,
            "balance": balance,
            "sync": sync,
            "grouped": grouped,
            "search": search,
            "searchedTxid": txid,
            "fullHistoryClaimed": false,
            "pageLimit": 10,
            "maxPages": 1
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
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to create Explorer live canary runtime: {error}"))?;
    let value = runtime.block_on(run_native(root, &args))?;
    write_json(
        &args
            .output_directory
            .join("explorer-live-canary-result.json"),
        &value,
    )?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to serialize Explorer live canary result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_is_fail_closed_and_defaults_are_bounded() {
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
        assert_eq!(parsed.address, DEFAULT_ADDRESS);
        assert_eq!(parsed.webdriver_port, DEFAULT_WEBDRIVER_PORT);
    }

    #[test]
    fn grouped_validation_requires_real_rows_exact_address_and_txid() {
        let groups = json!([{
            "day": "2026-10-05",
            "transactions": [{
                "txid": "abc123",
                "address": DEFAULT_ADDRESS
            }]
        }]);
        assert_eq!(
            first_grouped_txid(&groups, DEFAULT_ADDRESS).unwrap(),
            "abc123"
        );
        assert!(grouped_contains_exact_txid(&groups, DEFAULT_ADDRESS, "abc123").unwrap());

        let empty = json!([{
            "day": "2026-10-05",
            "transactions": []
        }]);
        assert!(first_grouped_txid(&empty, DEFAULT_ADDRESS).is_err());
    }
}
