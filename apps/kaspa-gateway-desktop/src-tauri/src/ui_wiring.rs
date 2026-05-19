use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FeatureWiringItem {
    pub feature: String,
    pub status: String,
    pub command: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureWiringReport {
    pub total_features: usize,
    pub wired_features: usize,
    pub partial_features: usize,
    pub remaining_features: usize,
    pub items: Vec<FeatureWiringItem>,
}

#[tauri::command]
pub fn feature_wiring_report() -> FeatureWiringReport {
    let items = vec![
        item(
            "Dashboard Runtime",
            "wired",
            "dashboard_report",
            "Runtime, API, node capabilities, and address count.",
        ),
        item(
            "Manage Addresses",
            "wired",
            "list_addresses/add_address/rename_address/delete_address/address_book_*",
            "CRUD, import/export, stats, and DB-backed address book.",
        ),
        item(
            "Explorer Balance",
            "wired",
            "explorer_fetch_balance/explorer_balance",
            "Live balance fetch through Rust API client.",
        ),
        item(
            "Explorer Transactions",
            "wired",
            "explorer_fetch_transactions/explorer_real_transactions/explorer_transactions_page",
            "Pagination, force fetch, cache, filtering, and local DB.",
        ),
        item(
            "Transaction Analysis",
            "wired",
            "analyze_address_flow/analyze_all_transactions/address_direction_breakdown/local_rich_list",
            "Incoming/outgoing/net flow and local rich list.",
        ),
        item(
            "Top Addresses",
            "wired",
            "top_addresses_fetch_api/top_addresses_search/top_addresses_*",
            "API fetch, known names, cached rates, and search.",
        ),
        item(
            "Network Analytics",
            "wired",
            "network_analytics_report/full_network_analytics_report",
            "Structured metric cards and raw JSON.",
        ),
        item(
            "Node Settings",
            "wired",
            "load_node_settings/save_node_settings/node_command_preview/node_inspect",
            "Settings, preview, deep inspect, health, version, and download verification.",
        ),
        item(
            "Node Runtime",
            "partial",
            "node_start/node_stop/node_status/real_node_runtime_report",
            "Supervisor is wired; real process execution still depends on a valid kaspad path.",
        ),
        item(
            "Bridge Runtime",
            "partial",
            "bridge_runtime_plan/bridge_runtime_health/bridge_config_preview/bridge_write_config_yaml",
            "Integrated and external bridge planning is wired; live execution depends on real binaries.",
        ),
        item(
            "Logs",
            "wired",
            "append_log/list_logs/clear_logs/diagnostics_report/persistent_log_*",
            "In-memory logs, persistent JSONL logs, tails, and diagnostics.",
        ),
        item(
            "Settings Migration",
            "wired",
            "settings_deep_*/real_config_*",
            "Display tabs, currencies, languages, API profiles, paths, and Python config import.",
        ),
        item(
            "i18n",
            "wired",
            "i18n_languages/i18n_load_catalog/i18n_save_language/i18n_import_translations",
            "Language catalog loading, persistence, report, and Python translation import.",
        ),
        item(
            "Export",
            "wired",
            "export_real_report/export_data/export_report",
            "CSV/HTML/PDF report export and parity reports.",
        ),
        item(
            "Python Migration",
            "wired",
            "preview_python_migration/migrate_python_data/real_python_migration_preview/real_python_migration_run",
            "DuckDB discovery, schema mapping, and safe import.",
        ),
        item(
            "Security",
            "wired",
            "security_hardening_report/security_secret_scan/security_recovery_manifest",
            "Redaction, validation, recovery snapshots, secret scan, and manifest.",
        ),
        item(
            "Release QA",
            "wired",
            "final_release_qa_report/feature_wiring_report",
            "Release readiness and feature wiring reports.",
        ),
        item(
            "Release Build",
            "partial",
            "tools/build-release.ps1/npm run tauri -- build",
            "Unsigned unless certificate is configured.",
        ),
    ];

    let wired_features = items.iter().filter(|item| item.status == "wired").count();
    let partial_features = items.iter().filter(|item| item.status == "partial").count();
    let remaining_features = items
        .iter()
        .filter(|item| item.status == "remaining")
        .count();

    FeatureWiringReport {
        total_features: items.len(),
        wired_features,
        partial_features,
        remaining_features,
        items,
    }
}

fn item(feature: &str, status: &str, command: &str, notes: &str) -> FeatureWiringItem {
    FeatureWiringItem {
        feature: feature.to_string(),
        status: status.to_string(),
        command: command.to_string(),
        notes: notes.to_string(),
    }
}
