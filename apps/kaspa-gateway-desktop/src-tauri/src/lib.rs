/* KGW_RUNTIME_MAIN_WINDOW_ICON_START */
fn kgw_set_runtime_main_window_icon(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::Manager;

    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    let icon = tauri::image::Image::new(include_bytes!("../icons/icon.rgba"), 128, 128);
    window.set_icon(icon)?;

    Ok(())
}
/* KGW_RUNTIME_MAIN_WINDOW_ICON_END */

mod address_book;
mod address_commands;
mod analysis_commands;
mod app_logger;
pub mod commands;
mod db_state;
mod db_status_commands;
mod diagnostics;
mod explorer_services;
mod export_commands;
mod export_system;
mod i18n_commands;
mod integrated_runtime_commands;
mod live_metrics;
mod transaction_commands;
mod transaction_routes;

mod config_commands;
mod data_enforcement_commands;
mod migration;
mod network_full;
mod persistent_logs;
mod price_service;
mod python_migration_real;
mod real_reports;
mod release_qa;
mod runtime_commands;
mod security_audit;
mod security_hardening;
mod settings;
mod settings_commands;
mod top_addresses_commands;
mod top_addresses_deep;
mod transaction_analysis;
mod ui_wiring;

#[tauri::command]
fn kgw_frontend_button_trace_v1(
    scope: String,
    net: String,
    action: String,
    phase: String,
    details: String,
) -> bool {
    println!(
        "[KGW_BUTTON_TRACE] scope={} net={} action={} phase={} details={}",
        scope, net, action, phase, details
    );
    true
}

pub fn run() {
    app_logger::init_tracing_bridge();
    tauri::Builder::default()
        .manage(commands::DesktopRuntimeState::default())
        .manage(diagnostics::LogState::default())
        .invoke_handler(tauri::generate_handler![
            integrated_runtime_commands::kgw_runtime_owner_summary_v1,
            integrated_runtime_commands::kgw_runtime_owner_status_v1,
            integrated_runtime_commands::kgw_runtime_owner_plan_v1,
            integrated_runtime_commands::kgw_all_parallel_node_bridge_plans_v1,
            integrated_runtime_commands::kgw_kgw_runtime_logs_v1,
            integrated_runtime_commands::kgw_kgw_real_owner_summary_v1,
            integrated_runtime_commands::kgw_kgw_real_owner_feature_status_v1,
            integrated_runtime_commands::kgw_kgw_apply_node_settings_v1,
            integrated_runtime_commands::kgw_kgw_disable_network_v1,
            integrated_runtime_commands::kgw_kgw_smoke_start_network_v1,
            integrated_runtime_commands::kgw_kgw_smoke_stop_network_v1,
            integrated_runtime_commands::kgw_kgw_node_bridge_service_plan_v1,
            commands::open_external_url,
            transaction_commands::explorer_list_transactions_grouped_rust,
            transaction_commands::explorer_transaction_day_summaries_rust,
            db_status_commands::kgw_settings_database_status,
            db_status_commands::kgw_settings_database_compact,
            db_status_commands::kgw_settings_database_clear_caches,
            db_status_commands::kgw_settings_database_backup,
            db_status_commands::kgw_settings_database_restore_latest,
            db_status_commands::kgw_settings_database_delete,
            price_service::kgw_get_kaspa_prices,
            top_addresses_commands::fetch_top_addresses_rust,
            address_commands::delete_saved_address,
            address_commands::save_address,
            address_commands::get_all_addresses,
            app_logger::kgw_log_file_path,
            app_logger::kgw_log_read,
            app_logger::kgw_log_clear,
            app_logger::kgw_log_append,
            transaction_routes::explorer_transactions,
            transaction_routes::explorer_delete_transactions_for_address,
            live_metrics::kgw_live_metrics_snapshot,
            live_metrics::kgw_live_metrics_refresh_now,
            explorer_services::explorer_export,
            explorer_services::explorer_balance,
            explorer_services::explorer_saved_addresses,
            data_enforcement_commands::api_probe,
            data_enforcement_commands::data_enforcement_report,
            security_audit::security_audit_report,
            security_audit::security_recovery_manifest,
            security_audit::security_secret_scan,
            export_commands::export_report,
            export_commands::export_preview,
            export_commands::export_default_path,
            i18n_commands::i18n_import_translations,
            i18n_commands::i18n_report,
            i18n_commands::i18n_get_active_language,
            i18n_commands::i18n_save_language,
            i18n_commands::i18n_load_catalog,
            i18n_commands::i18n_languages,
            settings_commands::settings_api_endpoint_editor,
            settings_commands::settings_secure_get_masked,
            settings_commands::settings_secure_store,
            settings_commands::settings_validate_custom_path,
            settings_commands::settings_import_config,
            settings_commands::settings_reset,
            settings_commands::settings_save,
            settings_commands::settings_load,
            settings_commands::settings_defaults,
            persistent_logs::persistent_log_stats,
            persistent_logs::persistent_log_clear,
            persistent_logs::persistent_log_tail,
            persistent_logs::persistent_log_list,
            persistent_logs::persistent_log_append,
            top_addresses_commands::top_addresses_load_currency_rates,
            top_addresses_commands::top_addresses_save_currency_rates,
            top_addresses_commands::top_addresses_load_known_names,
            top_addresses_commands::top_addresses_save_known_names,
            top_addresses_commands::top_addresses_search,
            top_addresses_commands::top_addresses_fetch_api,
            analysis_commands::analysis_graph_report,
            analysis_commands::analysis_report,
            analysis_commands::analysis_time_range_options,
            address_book::address_book_import_json,
            address_book::address_book_import_csv,
            address_book::address_book_export_json,
            address_book::address_book_export_csv,
            address_book::address_book_stats,
            release_qa::final_release_qa_report,
            security_hardening::security_hardening_report,
            security_hardening::create_recovery_snapshot,
            security_hardening::security_validate_path,
            security_hardening::security_validate_process_args,
            security_hardening::security_redact_text,
            ui_wiring::feature_wiring_report,
            python_migration_real::real_python_migration_run,
            python_migration_real::real_python_migration_preview,
            real_reports::export_real_report,
            real_reports::real_report_preview,
            real_reports::default_real_report_path,
            network_full::full_network_analytics_report,
            transaction_analysis::address_direction_breakdown,
            transaction_analysis::local_rich_list,
            transaction_analysis::analyze_all_transactions,
            transaction_analysis::analyze_address_flow,
            runtime_commands::tail_process_log,
            runtime_commands::real_bridge_runtime_report,
            runtime_commands::real_node_runtime_report,
            runtime_commands::real_node_default_runtime_settings,
            runtime_commands::real_bridge_default_runtime_settings,
            runtime_commands::real_node_runtime_command_preview,
            runtime_commands::real_bridge_runtime_command_preview,
            runtime_commands::real_node_runtime_apply_settings,
            runtime_commands::real_bridge_runtime_apply_settings,
            config_commands::config_import_python_config,
            config_commands::config_save,
            config_commands::config_load,
            config_commands::config_default,
            commands::desktop_ping,
            commands::app_info,
            commands::runtime_check,
            commands::api_network_url,
            commands::settings_fetch_address_names,
            commands::node_capabilities,
            commands::dashboard_report,
            commands::ui_sections,
            commands::list_addresses,
            commands::add_address,
            commands::delete_address,
            commands::rename_address,
            commands::explorer_fetch_balance,
            commands::network_analytics_report,
            commands::load_node_settings,
            commands::save_node_settings,
            commands::node_command_preview,
            commands::node_start,
            commands::node_stop,
            commands::node_status,
            commands::load_bridge_settings,
            commands::save_bridge_settings,
            commands::bridge_command_preview,
            commands::bridge_start,
            commands::bridge_stop,
            commands::bridge_status,
            diagnostics::append_log,
            diagnostics::list_logs,
            diagnostics::clear_logs,
            diagnostics::diagnostics_report,
            settings::load_full_settings,
            settings::save_full_settings,
            settings::reset_full_settings,
            export_system::default_export_path,
            export_system::export_data,
            migration::preview_python_migration,
            migration::migrate_python_data,
            kgw_frontend_button_trace_v1
        ])
        .setup(|app| {
            kgw_set_runtime_main_window_icon(app)?;
            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let _ = crate::integrated_runtime_commands::kgw_shutdown_all_runtime_workers_v1();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Kaspa Gateway desktop app");
}

fn kgw_init_bridge_self_worker_raw_tracing_r23() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new(
                "info,kaspa_stratum_bridge=debug,kaspa_gateway_rk_bridge=debug",
            )
        });

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .with_target(true)
            .with_level(true)
            .with_writer(std::io::stderr)
            .try_init();
    });
}

pub fn try_run_kgw_self_worker_from_args() -> bool {
    let args: Vec<String> = std::env::args().collect();

    if !args.iter().any(|arg| arg == "--kgw-self-worker") {
        return false;
    }

    let role =
        kgw_self_worker_arg_value(&args, "--kgw-self-worker").unwrap_or_else(|| "node".to_string());
    let network =
        kgw_self_worker_arg_value(&args, "--network").unwrap_or_else(|| "mainnet".to_string());
    let appdir =
        kgw_self_worker_arg_value(&args, "--appdir").unwrap_or_else(kgw_self_worker_default_appdir);
    let rpc = kgw_self_worker_arg_value(&args, "--rpc")
        .unwrap_or_else(|| kgw_self_worker_default_rpc(&network).to_string());
    let stratum = kgw_self_worker_arg_value(&args, "--stratum")
        .unwrap_or_else(|| kgw_self_worker_default_stratum(&network).to_string());
    let utxoindex = args.iter().any(|arg| arg == "--utxoindex");
    let archival = args.iter().any(|arg| arg == "--archival");

    let result = match role.trim().to_ascii_lowercase().as_str() {
        "node" => kgw_run_node_self_worker(&network, &appdir, &rpc, utxoindex, archival),
        "bridge" => {
            kgw_init_bridge_self_worker_raw_tracing_r23();
            kgw_run_bridge_self_worker(&network, &rpc, &stratum, &args)
        }
        other => Err(format!("unsupported self-worker role: {other}")),
    };

    if let Err(_error) = result {
        std::process::exit(1);
    }

    true
}

fn kgw_self_worker_arg_value(args: &[String], key: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == key)
        .map(|window| window[1].clone())
}

fn kgw_self_worker_default_appdir() -> String {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        std::path::PathBuf::from(local_app_data)
            .join("rusty-kaspa")
            .to_string_lossy()
            .to_string()
    } else {
        std::env::temp_dir()
            .join("rusty-kaspa")
            .to_string_lossy()
            .to_string()
    }
}

fn kgw_self_worker_default_rpc(network: &str) -> &'static str {
    match network.trim().to_ascii_lowercase().as_str() {
        "testnet10" | "testnet" => "127.0.0.1:16210",
        "testnet12" | "tn12" => "127.0.0.1:16310",
        _ => "127.0.0.1:16110",
    }
}

fn kgw_self_worker_default_stratum(network: &str) -> &'static str {
    match network.trim().to_ascii_lowercase().as_str() {
        "testnet10" | "testnet" => "0.0.0.0:15555",
        "testnet12" | "tn12" => "0.0.0.0:25555",
        _ => "0.0.0.0:5555",
    }
}

fn kgw_run_node_self_worker(
    network: &str,
    appdir: &str,
    rpc: &str,
    utxoindex: bool,
    archival: bool,
) -> Result<(), String> {
    let mut settings = kaspa_gateway_rk_node::NodeSettings::from_strings(
        network.to_string(),
        "integrated-inproc".to_string(),
        "disable".to_string(),
    )
    .map_err(|error| error.to_string())?;

    settings.app_dir_name = appdir.to_string();
    settings.rpc_endpoint = rpc.to_string();
    settings.enable_utxo_index = utxoindex;
    settings.archival = archival;

    let runtime = kaspa_gateway_rk_node::KgwRealOwnerRuntime::new();
    let _status = runtime
        .start_node_owner_session(&settings)
        .map_err(|error| error.to_string())?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));

        match runtime.status(settings.network) {
            Ok(_status) => {}
            Err(_error) => {}
        }
    }
}

fn kgw_run_bridge_self_worker(
    network: &str,
    rpc: &str,
    stratum: &str,
    args: &[String],
) -> Result<(), String> {
    let bridge_cpu_miner_enabled = args.iter().any(|arg| arg == "--internal-cpu-miner");
    let bridge_cpu_miner_address = kgw_self_worker_arg_value(args, "--internal-cpu-miner-address");
    let bridge_cpu_miner_threads = kgw_self_worker_arg_value(args, "--internal-cpu-miner-threads")
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|value| *value > 0);
    let bridge_cpu_miner_throttle_ms =
        kgw_self_worker_arg_value(args, "--internal-cpu-miner-throttle-ms")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0);
    let bridge_cpu_miner_template_poll_ms =
        kgw_self_worker_arg_value(args, "--internal-cpu-miner-template-poll-ms")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0);

    let settings = kaspa_gateway_rk_bridge::BridgeRuntimeSettings {
        network: network.to_string(),
        mode: kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialExternalNode,
        stratum_listen: Some(stratum.to_string()),
        prometheus_listen: None,
        kaspa_rpc_endpoint: Some(rpc.to_string()),
        internal_cpu_miner: kaspa_gateway_rk_bridge::BridgeInternalCpuMinerSettings {
            enabled: bridge_cpu_miner_enabled,
            address: bridge_cpu_miner_address.clone(),
            threads: bridge_cpu_miner_threads,
            throttle_ms: bridge_cpu_miner_throttle_ms,
            template_poll_ms: bridge_cpu_miner_template_poll_ms,
        },
        explicit_runtime_opt_in: true,
    };

    let event = kaspa_gateway_rk_bridge::bridge_service_event_from_settings_v1(settings)
        .map_err(|error| error.to_string())?;

    let handle = kaspa_gateway_rk_bridge::start_official_bridge_owner_thread_v1(event)
        .map_err(|error| error.to_string())?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));

        if handle.is_finished() {
            return Ok(());
        }
    }
}
