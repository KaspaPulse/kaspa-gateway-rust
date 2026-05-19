use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct RealFeatureStatus {
    pub tab: String,
    pub feature: String,
    pub status: String,
    pub evidence: String,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealDataEnforcementReport {
    pub ready_for_real_use: bool,
    pub real_features: usize,
    pub partial_features: usize,
    pub missing_features: usize,
    pub statuses: Vec<RealFeatureStatus>,
    pub placeholder_findings: Vec<PlaceholderFinding>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaceholderFinding {
    pub path: String,
    pub line: usize,
    pub term: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealApiProbeRequest {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealApiProbeReport {
    pub base_url: String,
    pub network_url: String,
    pub reachable: bool,
    pub response_preview: String,
    pub message: String,
}

#[tauri::command]
pub fn data_enforcement_report(root_path: String) -> Result<RealDataEnforcementReport, String> {
    validate_path_text(&root_path)?;

    let root = PathBuf::from(root_path.trim());

    if !root.exists() {
        return Err("Root path does not exist.".to_string());
    }

    if !root.is_dir() {
        return Err("Root path is not a directory.".to_string());
    }

    let manager = database_manager()?;
    let user_root = default_user_data_dir().map_err(|error| error.to_string())?;

    let mut statuses = Vec::new();

    let address_count = manager
        .addresses_repository()
        .and_then(|repo| repo.count())
        .unwrap_or(0);

    statuses.push(status(
        "Explorer",
        "Saved address book",
        address_count > 0,
        format!("saved_addresses={address_count}"),
        "Save or import at least one real Kaspa address.",
    ));

    statuses.push(transaction_cache_status(&manager)?);

    let api_probe = probe_kaspa_api("https://api.kaspa.org");
    statuses.push(status(
        "Explorer",
        "Live Kaspa API",
        api_probe.reachable,
        api_probe.message,
        "Verify network access to https://api.kaspa.org and retry balance/transactions.",
    ));

    statuses.push(setting_status(
        &manager,
        "Kaspa Node",
        "Node settings persisted",
        "node.settings.json",
        "Configure kaspad path, network, RPC host/port, then save settings.",
    ));

    statuses.push(setting_status(
        &manager,
        "Kaspa Bridge",
        "Bridge settings persisted",
        "bridge.settings.json",
        "Configure IntegratedBridge / ExternalCompatibilityBridge and save settings.",
    ));

    statuses.push(setting_status(
        &manager,
        "Settings",
        "Deep settings persisted",
        "settings.deep.v1.json",
        "Open Settings and press Save Deep Settings.",
    ));

    statuses.push(setting_status(
        &manager,
        "Configuration",
        "Migrated real config",
        "real_config.migrated.json",
        "Import or save real config once.",
    ));

    let i18n_language = read_setting_value(&manager, "i18n.active_language");
    statuses.push(status(
        "Settings",
        "i18n active language",
        i18n_language.is_some(),
        i18n_language.unwrap_or_else(|| "no active language stored".to_string()),
        "Change language once to persist the selected language.",
    ));

    let logs_dir = user_root.join("logs");
    statuses.push(status(
        "Logs",
        "Persistent logs path",
        logs_dir.exists() && logs_dir.is_dir(),
        logs_dir.display().to_string(),
        "Open Log tab and append/refresh persistent logs.",
    ));

    let exports_dir = user_root.join("exports");
    statuses.push(status(
        "Export",
        "Export output path",
        exports_dir.exists() && exports_dir.is_dir(),
        exports_dir.display().to_string(),
        "Run an export once or create the exports folder.",
    ));

    let top_addresses_ready = read_setting_value(&manager, "top_addresses.cache.json").is_some()
        || read_setting_value(&manager, "top_addresses.known_names.json").is_some();

    statuses.push(status(
        "Top Addresses",
        "Known names/cache ready",
        top_addresses_ready,
        "top_addresses cache/known names check".to_string(),
        "Fetch top addresses or add known names.",
    ));

    let placeholder_findings = scan_placeholder_terms(&root)?;

    let real_features = statuses.iter().filter(|item| item.status == "real").count();
    let partial_features = statuses
        .iter()
        .filter(|item| item.status == "partial")
        .count();
    let missing_features = statuses
        .iter()
        .filter(|item| item.status == "missing")
        .count();

    let ready_for_real_use = missing_features == 0 && placeholder_findings.is_empty();

    Ok(RealDataEnforcementReport {
        ready_for_real_use,
        real_features,
        partial_features,
        missing_features,
        statuses,
        placeholder_findings,
        message: if ready_for_real_use {
            "All checked features look real-ready.".to_string()
        } else {
            "Real data enforcement found incomplete or placeholder-backed areas.".to_string()
        },
    })
}

#[tauri::command]
pub fn api_probe(request: RealApiProbeRequest) -> Result<RealApiProbeReport, String> {
    validate_https_url(&request.base_url)?;
    Ok(probe_kaspa_api(&request.base_url))
}

fn transaction_cache_status(manager: &DatabaseManager) -> Result<RealFeatureStatus, String> {
    let address_repo = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let tx_repo = manager
        .transactions_repository()
        .map_err(|error| error.to_string())?;

    let addresses = address_repo.list().map_err(|error| error.to_string())?;
    let mut total_transactions = 0_usize;

    for address in addresses {
        let rows = tx_repo
            .list_for_address(&address.address, 10_000_usize)
            .map_err(|error| error.to_string())?;

        total_transactions = total_transactions.saturating_add(rows.len());
    }

    Ok(status(
        "Explorer",
        "Cached real transactions",
        total_transactions > 0,
        format!("cached_transactions={total_transactions}"),
        "Use Explorer to fetch transactions from the live API for a saved address.",
    ))
}

fn setting_status(
    manager: &DatabaseManager,
    tab: &str,
    feature: &str,
    key: &str,
    next_action: &str,
) -> RealFeatureStatus {
    let value = read_setting_value(manager, key);

    status(
        tab,
        feature,
        value.is_some(),
        if value.is_some() {
            format!("{key} exists in app settings")
        } else {
            format!("{key} not found")
        },
        next_action,
    )
}

fn probe_kaspa_api(base_url: &str) -> RealApiProbeReport {
    let base = base_url.trim().trim_end_matches('/');
    let url = format!("{base}/info/network");

    let result = ureq::get(&url).timeout(Duration::from_secs(6)).call();

    match result {
        Ok(response) => {
            let text = response
                .into_string()
                .unwrap_or_else(|_| "<non-text response>".to_string());

            let preview = safe_preview(&text, 300);
            let valid_json = serde_json::from_str::<Value>(&text).is_ok();

            RealApiProbeReport {
                base_url: base.to_string(),
                network_url: url,
                reachable: valid_json,
                response_preview: preview,
                message: if valid_json {
                    "Kaspa API is reachable and returned JSON.".to_string()
                } else {
                    "Kaspa API responded but response was not JSON.".to_string()
                },
            }
        }
        Err(error) => RealApiProbeReport {
            base_url: base.to_string(),
            network_url: url,
            reachable: false,
            response_preview: String::new(),
            message: format!("Kaspa API probe failed: {error}"),
        },
    }
}

fn read_setting_value(manager: &DatabaseManager, key: &str) -> Option<String> {
    manager
        .app_settings_repository()
        .ok()
        .and_then(|repo| repo.get(key).ok())
        .flatten()
}

fn status(
    tab: &str,
    feature: &str,
    condition: bool,
    evidence: String,
    next_action: &str,
) -> RealFeatureStatus {
    RealFeatureStatus {
        tab: tab.to_string(),
        feature: feature.to_string(),
        status: if condition {
            "real".to_string()
        } else {
            "missing".to_string()
        },
        evidence,
        next_action: next_action.to_string(),
    }
}

fn scan_placeholder_terms(root: &Path) -> Result<Vec<PlaceholderFinding>, String> {
    let mut findings = Vec::new();

    scan_dir(root, &mut findings)?;

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.term.cmp(&right.term))
    });

    findings.truncate(500);

    Ok(findings)
}

fn scan_dir(path: &Path, findings: &mut Vec<PlaceholderFinding>) -> Result<(), String> {
    let entries = fs::read_dir(path).map_err(|error| error.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        if should_skip_path(file_name) {
            continue;
        }

        let metadata = entry.metadata().map_err(|error| error.to_string())?;

        if metadata.is_dir() {
            scan_dir(&path, findings)?;
        } else if metadata.is_file() && is_text_source_file(&path) && metadata.len() <= 2_000_000 {
            scan_file(&path, findings)?;
        }

        if findings.len() >= 500 {
            break;
        }
    }

    Ok(())
}

fn scan_file(path: &Path, findings: &mut Vec<PlaceholderFinding>) -> Result<(), String> {
    let text = fs::read_to_string(path).unwrap_or_default();

    if text.is_empty() {
        return Ok(());
    }

    let terms = [
        "TODO_FAKE",
        "PLACEHOLDER_DATA",
        "MOCK_DATA",
        "DUMMY_DATA",
        "FAKE_BALANCE",
        "SAMPLE_TRANSACTION",
        "sample data",
        "mocked",
        "placeholder",
    ];

    for (index, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();

        for term in terms {
            if lower.contains(&term.to_ascii_lowercase()) {
                findings.push(PlaceholderFinding {
                    path: path.display().to_string(),
                    line: index + 1,
                    term: term.to_string(),
                    preview: safe_preview(line.trim(), 180),
                });
            }
        }

        if findings.len() >= 500 {
            break;
        }
    }

    Ok(())
}

fn should_skip_path(file_name: &str) -> bool {
    matches!(
        file_name,
        "target"
            | "node_modules"
            | ".git"
            | "dist"
            | "build"
            | ".next"
            | ".vite"
            | "Cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
    )
}

fn is_text_source_file(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };

    matches!(
        extension.to_ascii_lowercase().as_str(),
        "rs" | "ts" | "tsx" | "js" | "jsx" | "json" | "toml" | "yaml" | "yml" | "md" | "txt"
    )
}

fn safe_preview(value: &str, max_chars: usize) -> String {
    value
        .replace(['\r', '\n'], " ")
        .chars()
        .take(max_chars)
        .collect()
}

fn validate_https_url(value: &str) -> Result<(), String> {
    let value = value.trim();

    if value.is_empty() {
        return Err("URL cannot be empty.".to_string());
    }

    if !value.starts_with("https://") {
        return Err("API base URL must use HTTPS.".to_string());
    }

    if value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err("URL contains unsafe characters.".to_string());
    }

    Ok(())
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
    fn status_maps_condition_to_real_or_missing() {
        let real = status("Tab", "Feature", true, "ok".to_string(), "none");
        let missing = status("Tab", "Feature", false, "missing".to_string(), "fix");

        assert_eq!(real.status, "real");
        assert_eq!(missing.status, "missing");
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        assert!(validate_path_text("C:\\temp\\project").is_ok());
        assert!(validate_path_text("bad && whoami").is_err());
    }

    #[test]
    fn https_url_is_required() {
        assert!(validate_https_url("https://api.kaspa.org").is_ok());
        assert!(validate_https_url("http://api.kaspa.org").is_err());
    }

    #[test]
    fn preview_collapses_newlines() {
        assert_eq!(safe_preview("a\r\nb", 10), "a  b");
    }
}
