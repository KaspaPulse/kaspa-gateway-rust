use reqwest::{Client, Method};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

const DEFAULT_PORT: u16 = 4445;
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 60;
const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone, Debug)]
pub(crate) struct WebDriverClient {
    http: Client,
    base_url: String,
}

impl WebDriverClient {
    pub(crate) fn loopback(port: u16) -> Result<Self, String> {
        if port == 0 {
            return Err("WebDriver port must be non-zero".to_owned());
        }
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(65))
            .build()
            .map_err(|error| format!("failed to build WebDriver HTTP client: {error}"))?;
        Ok(Self {
            http,
            base_url: format!("http://127.0.0.1:{port}"),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    async fn request_value(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, String> {
        let url = self.endpoint(path);
        let mut request = self.http.request(method, &url);
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .map_err(|error| format!("WebDriver request failed for {url}: {error}"))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| format!("failed to read WebDriver response for {url}: {error}"))?;
        let payload = if text.trim().is_empty() {
            json!({ "value": null })
        } else {
            serde_json::from_str::<Value>(&text).map_err(|error| {
                format!("invalid WebDriver JSON from {url}: {error}; body={text}")
            })?
        };
        if !status.is_success() {
            return Err(format!(
                "WebDriver request failed: method/status={status}; url={url}; body={payload}"
            ));
        }
        if let Some(error) = payload
            .pointer("/value/error")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        {
            let message = payload
                .pointer("/value/message")
                .and_then(Value::as_str)
                .unwrap_or("");
            return Err(format!("WebDriver error {error} from {url}: {message}"));
        }
        payload
            .get("value")
            .cloned()
            .ok_or_else(|| format!("WebDriver response from {url} is missing value: {payload}"))
    }

    pub(crate) async fn status_ready(&self) -> Result<bool, String> {
        Ok(self
            .request_value(Method::GET, "/status", None)
            .await?
            .get("ready")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    pub(crate) async fn create_session(
        &self,
        window_label: &str,
    ) -> Result<WebDriverSession, String> {
        let value = self
            .request_value(
                Method::POST,
                "/session",
                Some(&session_request(window_label)),
            )
            .await?;
        let session_id = value
            .get("sessionId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("WebDriver create-session response missing sessionId: {value}"))?
            .to_owned();
        Ok(WebDriverSession {
            driver: self.clone(),
            session_id,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WebDriverSession {
    driver: WebDriverClient,
    session_id: String,
}

impl WebDriverSession {
    fn path(&self, suffix: &str) -> String {
        format!("/session/{}{}", self.session_id, suffix)
    }

    pub(crate) async fn set_timeouts(
        &self,
        implicit_ms: u64,
        page_load_ms: u64,
        script_ms: u64,
    ) -> Result<(), String> {
        self.driver
            .request_value(
                Method::POST,
                &self.path("/timeouts"),
                Some(&json!({
                    "implicit": implicit_ms,
                    "pageLoad": page_load_ms,
                    "script": script_ms,
                })),
            )
            .await?;
        Ok(())
    }

    pub(crate) async fn title(&self) -> Result<String, String> {
        self.driver
            .request_value(Method::GET, &self.path("/title"), None)
            .await?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "WebDriver title response is not a string".to_owned())
    }

    pub(crate) async fn source(&self) -> Result<String, String> {
        self.driver
            .request_value(Method::GET, &self.path("/source"), None)
            .await?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "WebDriver source response is not a string".to_owned())
    }

    pub(crate) async fn execute_sync(
        &self,
        script: &str,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        self.driver
            .request_value(
                Method::POST,
                &self.path("/execute/sync"),
                Some(&json!({ "script": script, "args": args })),
            )
            .await
    }

    #[allow(dead_code)]
    pub(crate) async fn execute_async(
        &self,
        script: &str,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        self.driver
            .request_value(
                Method::POST,
                &self.path("/execute/async"),
                Some(&json!({ "script": script, "args": args })),
            )
            .await
    }

    pub(crate) async fn close(&self) -> Result<(), String> {
        self.driver
            .request_value(Method::DELETE, &self.path(""), None)
            .await?;
        Ok(())
    }
}

#[derive(Debug)]
struct EmbeddedApp {
    child: Child,
}

impl EmbeddedApp {
    fn spawn(root: &Path, app_binary: &Path, port: u16) -> Result<Self, String> {
        if !app_binary.is_file() {
            return Err(format!(
                "E2E app binary does not exist: {}",
                app_binary.display()
            ));
        }
        let child = Command::new(app_binary)
            .current_dir(root)
            .env("TAURI_WEBDRIVER_PORT", port.to_string())
            .env("WDIO_EMBEDDED_SERVER", "true")
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| {
                format!(
                    "failed to spawn E2E app binary {}: {error}",
                    app_binary.display()
                )
            })?;
        Ok(Self { child })
    }

    fn exit_status(&mut self) -> Result<Option<std::process::ExitStatus>, String> {
        self.child
            .try_wait()
            .map_err(|error| format!("failed to query E2E app status: {error}"))
    }

    fn terminate(&mut self) -> Result<(), String> {
        if self.exit_status()?.is_some() {
            return Ok(());
        }
        self.child
            .kill()
            .map_err(|error| format!("failed to terminate owned E2E app: {error}"))?;
        self.child
            .wait()
            .map_err(|error| format!("failed to reap owned E2E app: {error}"))?;
        Ok(())
    }
}

#[derive(Debug)]
struct SmokeArgs {
    app_binary: PathBuf,
    port: u16,
    window_label: String,
    startup_timeout: Duration,
}

fn session_request(window_label: &str) -> Value {
    json!({
        "capabilities": {
            "alwaysMatch": {
                "wdio:tauriServiceOptions": {
                    "windowLabel": window_label
                }
            },
            "firstMatch": [{}]
        }
    })
}

fn parse_smoke_args(args: &mut impl Iterator<Item = String>) -> Result<SmokeArgs, String> {
    let mut app_binary = None;
    let mut port = DEFAULT_PORT;
    let mut window_label = "main".to_owned();
    let mut startup_timeout = Duration::from_secs(DEFAULT_STARTUP_TIMEOUT_SECS);

    while let Some(arg) = args.next() {
        let value = match arg.as_str() {
            "--app-binary" | "--port" | "--window-label" | "--startup-timeout-seconds" => args
                .next()
                .ok_or_else(|| format!("missing value for {arg}"))?,
            other => {
                return Err(format!(
                    "unknown e2e-native-webdriver-smoke argument: {other}"
                ));
            }
        };
        match arg.as_str() {
            "--app-binary" => app_binary = Some(PathBuf::from(value)),
            "--port" => {
                port = value
                    .parse::<u16>()
                    .ok()
                    .filter(|port| *port != 0)
                    .ok_or_else(|| format!("invalid non-zero WebDriver port: {value}"))?;
            }
            "--window-label" => {
                if value.trim().is_empty() {
                    return Err("window label must not be empty".to_owned());
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
            _ => unreachable!(),
        }
    }

    Ok(SmokeArgs {
        app_binary: app_binary
            .ok_or_else(|| "e2e-native-webdriver-smoke requires --app-binary <path>".to_owned())?,
        port,
        window_label,
        startup_timeout,
    })
}

async fn wait_for_ready(
    driver: &WebDriverClient,
    app: &mut EmbeddedApp,
    timeout: Duration,
) -> Result<(), String> {
    let started = Instant::now();
    loop {
        if let Some(status) = app.exit_status()? {
            return Err(format!(
                "E2E app exited before WebDriver became ready: {status}"
            ));
        }
        match driver.status_ready().await {
            Ok(true) => return Ok(()),
            Ok(false) | Err(_) if started.elapsed() < timeout => sleep(POLL_INTERVAL).await,
            Ok(false) => {
                return Err(format!(
                    "WebDriver did not report ready within {} seconds",
                    timeout.as_secs()
                ));
            }
            Err(error) => {
                return Err(format!(
                    "WebDriver did not become reachable within {} seconds: {error}",
                    timeout.as_secs()
                ));
            }
        }
    }
}

async fn smoke(root: &Path, args: SmokeArgs) -> Result<Value, String> {
    let driver = WebDriverClient::loopback(args.port)?;
    let mut app = EmbeddedApp::spawn(root, &args.app_binary, args.port)?;
    let result = async {
        wait_for_ready(&driver, &mut app, args.startup_timeout).await?;
        let session = driver.create_session(&args.window_label).await?;
        session.set_timeouts(0, 300_000, 65_000).await?;
        let title = session.title().await?;
        let state = session
            .execute_sync(
                "return { title: document.title, readyState: document.readyState, href: location.href };",
                Vec::new(),
            )
            .await?;
        let source = session.source().await?;
        session.close().await?;
        Ok(json!({
            "passed": true,
            "port": args.port,
            "windowLabel": args.window_label,
            "title": title,
            "state": state,
            "sourceBytes": source.len(),
        }))
    }
    .await;
    let cleanup = app.terminate();
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error),
        (Err(error), Err(cleanup_error)) => {
            Err(format!("{error}; cleanup failure: {cleanup_error}"))
        }
    }
}

pub(crate) fn run_cli(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<String, String> {
    let args = parse_smoke_args(args)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to create E2E WebDriver runtime: {error}"))?;
    let value = runtime.block_on(smoke(root, args))?;
    serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to serialize E2E WebDriver smoke result: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_request_targets_named_window() {
        let value = session_request("main");
        assert_eq!(
            value.pointer("/capabilities/alwaysMatch/wdio:tauriServiceOptions/windowLabel"),
            Some(&json!("main"))
        );
        assert_eq!(
            value.pointer("/capabilities/firstMatch/0"),
            Some(&json!({}))
        );
    }

    #[test]
    fn loopback_client_rejects_zero_port() {
        assert!(WebDriverClient::loopback(0).is_err());
        let client = WebDriverClient::loopback(4445).unwrap();
        assert_eq!(client.base_url, "http://127.0.0.1:4445");
    }

    #[test]
    fn smoke_args_are_fail_closed() {
        let mut missing = Vec::<String>::new().into_iter();
        assert!(parse_smoke_args(&mut missing).is_err());

        let mut bad_port = vec![
            "--app-binary".to_owned(),
            "app.exe".to_owned(),
            "--port".to_owned(),
            "0".to_owned(),
        ]
        .into_iter();
        assert!(parse_smoke_args(&mut bad_port).is_err());
    }
}
