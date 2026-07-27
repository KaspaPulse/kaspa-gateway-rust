# Graph Report - kaspa-gateway-rust  (2026-07-28)

## Corpus Check
- 165 files · ~273,234 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 3946 nodes · 10107 edges · 196 communities (177 shown, 19 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 128 edges (avg confidence: 0.71)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c064107f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Result
- KgwNetwork
- kaspa-gateway-config/src/lib.rs
- settings.js
- analysis.js
- export_commands.rs
- commands.rs
- transaction_sync.rs
- kaspa-bridge.js
- Result
- app_logger.rs
- top_addresses_deep.rs
- default_user_data_dir
- integrated_runtime_commands.rs
- settings_commands.rs
- kaspa-gateway-core/src/lib.rs
- official_kaspa_runtime.rs
- python_migration_real.rs
- kaspa-gateway-rk-bridge/src/lib.rs
- migration.rs
- top-addresses.js
- src-tauri/src/lib.rs
- header-live-metrics.js
- kaspa-node.js
- config_commands.rs
- build_kgw_inproc_plan
- analysis_commands.rs
- main.js
- explorer.export.js
- kaspa-gateway-node/src/lib.rs
- byId
- runtime_commands.rs
- explorer.js
- installActions
- real_reports.rs
- kaspa-gateway-security/src/lib.rs
- i18n_commands.rs
- qs
- kgw_i18n_contract_gate.cjs
- bridgePortProfileR35B
- address_book.rs
- data_enforcement_commands.rs
- security_audit.rs
- esc
- explorer_services.rs
- kgw_runtime_repository_binding_gate.cjs
- analysis-rust-binding.js
- microscopeLog
- installActions
- bridgeApplyPortAutofixR37
- kgw_global_owner_gate.cjs
- kgw_program_unified_gate.cjs
- fetchTransactions
- db_status_commands.rs
- security_hardening.rs
- kgw_bridge_node_mode_routing_audit_v1.cjs
- install
- install
- persistent_logs.rs
- boot
- log.js
- install
- extractRowsFromUnifiedResult
- transaction_analysis.rs
- buildCommandLines
- definitions
- properties
- diagnostics.rs
- tauri.conf.json
- package.json
- setLanguage
- definitions
- apply
- kgwBridgeR51ReadSettings
- properties
- kgw_i18n_locale_coverage_gate.cjs
- cardInput
- settings.rs
- kgw_runtime_trace_owner_audit_v20.cjs
- kgwShellSetSelectOptionsVisibleR73
- updateCommand
- network_full.rs
- normalizeDateInputValue
- kgwNodeR51RefreshOne
- kgw_parallel_self_worker_runtime_gate.cjs
- installForNetwork
- permissions
- kgw_runtime_repository_binding_apply.ps1
- shell-logger.js
- invokeCommand
- default.json
- Capability
- webviews
- webviews
- kgwInvokeExplorerDaySummaries
- CapabilityRemote
- CapabilityRemote
- probe
- initKaspaBridgeTab
- release_qa.rs
- kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5
- permissions
- ui_wiring.rs
- unique_test_dir
- kgwStrictNormalizeAllPanels
- core/tab-registry.js
- kgwInstallNodeLogAutoScrollControlsR27
- kgw_live_network_smoke.ps1
- kgw_runtime_repository_binding_audit.cjs
- tabs.js
- Self
- database_root
- explorer_tab.rs
- ADR-0010: Same-EXE Parallel Self-Worker Runtime
- Kaspa Gateway Agent Workflow
- Canonical mechanism
- Runtime Network Repository Bindings
- graphify reference: extra exports and benchmark
- db_foundation_tests.rs
- repository_tests.rs
- AI Development Workflow
- Security Policy
- Bridge README Runtime Instance Contract
- Main features
- KGW Security Baseline
- Desktop UI overview
- Development commands
- Troubleshooting
- Capability
- graphify reference: query, path, explain
- KnownNamesRepository
- Dependency Risk Register
- desktop-schema.json
- windows-schema.json
- AppSettingsRepository
- Live Kaspa Network Smoke Test
- Raw log model
- Node and bridge behavior
- kgw_ai_workflow_gate.ps1
- Kaspa Gateway Desktop
- local
- local
- graphify reference: add a URL and watch a folder
- graphify reference: commit hook and native CLAUDE.md integration
- graphify reference: incremental update and cluster-only
- Architecture overview
- Configuration
- Repository links and runtime bindings
- graphify reference: GitHub clone and cross-repo merge
- graphify reference: transcribe video and audio
- Maintainer rules
- Internationalization
- src/README.md
- extraction-spec.md
- initKaspaBridgeTab
- kgwInstallNodeLogAutoScrollControlsR27
- kgwInvokeExplorerGroupedTransactions
- kgwNodeSmallOwnerTraceR44D
- kgwSettingsTraceButtonDetailsR29B
- kgw_start_button_gate.ps1
- database_paths_from_default_user_dir
- kgw_inproc_node_only.rs
- kaspa-gateway-rk-node/src/lib.rs
- kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5
- kgw_inproc_owner_tests.rs
- AppCacheRepository
- Arc
- AsRef
- HashMap
- Into
- Option
- PathBuf
- Result
- String
- Vec
- VecDeque

## God Nodes (most connected - your core abstractions)
1. `DatabaseManager` - 46 edges
2. `default_user_data_dir()` - 42 edges
3. `qs()` - 41 edges
4. `KaspaApiClient` - 35 edges
5. `KgwNetwork` - 34 edges
6. `installActions()` - 30 edges
7. `microscopeLog()` - 27 edges
8. `TransactionRecord` - 27 edges
9. `Kaspa Gateway Rust` - 27 edges
10. `fetchTransactions()` - 26 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `default_config_path()`  [INFERRED]
  apps/kaspa-gateway-cli/src/main.rs → crates/kaspa-gateway-config/src/lib.rs
- `main()` --calls--> `runtime_check_default()`  [INFERRED]
  apps/kaspa-gateway-cli/src/main.rs → crates/kaspa-gateway-runtime/src/lib.rs
- `database_paths_from_default_user_dir()` --calls--> `default_user_data_dir()`  [INFERRED]
  apps/kaspa-gateway-cli/src/main.rs → crates/kaspa-gateway-config/src/lib.rs
- `address_book_stats()` --calls--> `default_user_data_dir()`  [INFERRED]
  apps/kaspa-gateway-desktop/src-tauri/src/address_book.rs → crates/kaspa-gateway-config/src/lib.rs
- `database_manager()` --calls--> `default_user_data_dir()`  [INFERRED]
  apps/kaspa-gateway-desktop/src-tauri/src/address_book.rs → crates/kaspa-gateway-config/src/lib.rs

## Import Cycles
- 2-file cycle: `crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs -> crates/kaspa-gateway-rk-node/src/kgw_service_controller.rs -> crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs`

## Communities (196 total, 19 thin omitted)

### Community 0 - "Result"
Cohesion: 0.20
Nodes (6): initialize_sqlite_transactions_schema(), Result, Vec, TransactionFilter, TransactionsRepository, validate_non_empty()

### Community 1 - "KgwNetwork"
Cohesion: 0.06
Nodes (57): Args, default_prometheus_for_network(), embedded_fd_budget_fits_windows_default_limit(), embedded_fd_budget_removes_legacy_1024_floor_behavior(), kgw_apply_embedded_fd_limits_mainline(), kgw_apply_embedded_fd_limits_tn12(), kgw_claim_single_inproc_official_core(), kgw_embedded_core_fd_budget() (+49 more)

### Community 2 - "kaspa-gateway-config/src/lib.rs"
Cohesion: 0.17
Nodes (29): rejects_unsafe_output_path(), default_config_for_root(), default_config_is_valid(), default_config_path(), default_user_data_dir(), GatewayConfig, GatewayPaths, get_assets_path() (+21 more)

### Community 3 - "settings.js"
Cohesion: 0.07
Nodes (90): activateInner(), activateOuter(), applyBindings(), applyState(), bindSelectAll(), bindStaticActions(), collectState(), combineUrl() (+82 more)

### Community 4 - "analysis.js"
Cohesion: 0.06
Nodes (88): analysisRows, applyFilter(), bindCalendar(), bindControls(), emptyMessage(), escapeHtml(), filteredRows, initAnalysisTab() (+80 more)

### Community 5 - "export_commands.rs"
Cohesion: 0.08
Nodes (85): address_row(), addresses_report(), analysis_report(), apply_time_range(), average_kas(), build_report_table(), client_report_table(), csv_cell() (+77 more)

### Community 6 - "commands.rs"
Cohesion: 0.10
Nodes (71): delete_saved_address(), get_all_addresses(), Result, String, Vec, save_address(), add_address(), api_network_url() (+63 more)

### Community 7 - "transaction_sync.rs"
Cohesion: 0.06
Nodes (75): AppHandle, explorer_list_transactions_grouped_rust(), explorer_transaction_day_summaries_rust(), ExplorerTransactionDaySummary, Result, String, Vec, sompi_to_kas() (+67 more)

### Community 8 - "kaspa-bridge.js"
Cohesion: 0.04
Nodes (68): activeInstance, applyFontSize(), bridgeBuildUpstreamInstanceArg(), bridgeCollectCommandPorts(), bridgeDuplicatePorts(), bridgeInstanceAppend(), bridgeInstanceBoolArg(), bridgeInstanceCanonicalKey() (+60 more)

### Community 9 - "Result"
Cohesion: 0.07
Nodes (36): AddressBalanceRaw, AddressNameRecord, ApiClientConfig, ApiEndpoints, ApiError, BlockDagInfo, build_api_url(), coingecko_prices_parse() (+28 more)

### Community 10 - "app_logger.rs"
Cohesion: 0.07
Nodes (71): app_log_dir(), app_log_file(), init_tracing_bridge(), kgw_civil_from_days(), kgw_epoch_ms_now(), kgw_log_append(), kgw_log_clear(), kgw_log_file_path() (+63 more)

### Community 11 - "top_addresses_deep.rs"
Cohesion: 0.11
Nodes (63): as_f64(), as_string(), extract_python_top_address_items(), fetch_address_names_map(), fetch_json(), fetch_top_addresses_inner(), fetch_top_addresses_rust(), json_preview() (+55 more)

### Community 12 - "default_user_data_dir"
Cohesion: 0.16
Nodes (36): build_minimal_pdf(), build_pdf_content_stream(), csv_cell(), database_manager(), default_export_path(), default_format(), default_target(), ensure_extension() (+28 more)

### Community 13 - "integrated_runtime_commands.rs"
Cohesion: 0.08
Nodes (75): controller(), kgw_all_parallel_node_bridge_plans_v1(), kgw_apply_bridge_active_instance_runtime_overrides_r110f(), kgw_apply_command_preview_overrides(), kgw_bridge_command_preview_instance_listens_r123(), kgw_bridge_config_path_from_preview_r122(), kgw_bridge_instance_value_r110f(), kgw_bridge_instance_value_r120() (+67 more)

### Community 14 - "settings_commands.rs"
Cohesion: 0.14
Nodes (55): ApiEndpoint, ApiProfileDeep, database_manager(), default_settings(), defaults_are_valid(), find_bool(), find_f64(), find_string() (+47 more)

### Community 15 - "kaspa-gateway-core/src/lib.rs"
Cohesion: 0.07
Nodes (29): app_info_is_available(), AppInfo, format_kas(), format_with_currency(), GatewayError, is_kaspa_address_like(), kaspa_address_accepts_safe_prefixes(), KaspaAddress (+21 more)

### Community 16 - "official_kaspa_runtime.rs"
Cohesion: 0.12
Nodes (15): all_parallel_runtime_plans_v1(), KaspaBridgeRuntimeMode, KaspaNodeRuntimeMode, KaspaRuntimeFamily, KaspaRuntimePlan, KaspaRuntimeServiceEvent, KaspaRuntimeServiceEventKind, KaspaRuntimeSettings (+7 more)

### Community 17 - "python_migration_real.rs"
Cohesion: 0.13
Nodes (50): cast_column_sql(), clean_network(), clean_text(), collect_rows(), count_table_rows(), database_manager(), discover_database_maps(), discover_duckdb_files() (+42 more)

### Community 18 - "kaspa-gateway-rk-bridge/src/lib.rs"
Cohesion: 0.09
Nodes (28): all_parallel_bridge_plans_v1(), bridge_service_event_from_settings_v1(), BridgeInternalCpuMinerSettings, BridgeOwnerRuntimeHandle, BridgeRuntimeError, BridgeRuntimeFamily, BridgeRuntimeMode, BridgeRuntimeNetwork (+20 more)

### Community 19 - "migration.rs"
Cohesion: 0.13
Nodes (46): cast_column_sql(), clean_network(), clean_text(), collect_rows(), count_importable_addresses(), count_importable_settings(), count_importable_transactions(), count_role_tables() (+38 more)

### Community 20 - "top-addresses.js"
Cohesion: 0.10
Nodes (47): applyFilter(), buttonByText(), currentSearchText(), exportCsv(), exportHtml(), exportPdf(), findCsvButton(), findFilterButton() (+39 more)

### Community 21 - "src-tauri/src/lib.rs"
Cohesion: 0.10
Nodes (44): App, kgw_bridge_config_instance_listens_r122(), kgw_bridge_normalize_listen_from_config_r122(), kgw_frontend_button_trace_v1(), kgw_init_bridge_self_worker_raw_tracing_r23(), kgw_open_exported_file_v1(), kgw_run_bridge_self_worker(), kgw_run_node_self_worker() (+36 more)

### Community 22 - "header-live-metrics.js"
Cohesion: 0.09
Nodes (47): bindCurrencySelect(), boot(), findPriceElement(), initHeaderLiveMetrics(), invokeCommand(), kgwApplyEnglishTooltips(), kgwApplySnapshot(), kgwBuildEnglishTooltip() (+39 more)

### Community 23 - "kaspa-node.js"
Cohesion: 0.07
Nodes (40): id(), installDelegatedTabs(), KGW_NODE_R51_LAST_ACTIVITY_NOTICE, KGW_NODE_R51_LAST_LOGS, KGW_NODE_R51_LAST_STATUS, KGW_NODE_R51_TRANSITIONS, kgwFinalIsolatedAdapterInvokeV1(), kgwFinalIsolatedAdapterStartV1() (+32 more)

### Community 24 - "config_commands.rs"
Cohesion: 0.18
Nodes (45): config_default(), config_import_python_config(), config_load(), config_save(), currency_codes_are_normalized(), database_manager(), default_config_is_valid(), default_currencies() (+37 more)

### Community 25 - "build_kgw_inproc_plan"
Cohesion: 0.15
Nodes (20): Result, build_kgw_inproc_plan(), explicit_opt_in_is_blocked_until_owned_dependencies_exist(), KgwInprocError, KgwInprocPlan, KgwInprocRuntimeState, KgwInprocStartRequest, KgwInprocStep (+12 more)

### Community 26 - "analysis_commands.rs"
Cohesion: 0.13
Nodes (42): analysis_graph_report(), analysis_report(), analysis_time_range_options(), AnalysisBucket, AnalysisCluster, AnalysisCounterparty, AnalysisDeepReport, AnalysisDeepRequest (+34 more)

### Community 27 - "main.js"
Cohesion: 0.07
Nodes (35): applyBindings(), applyTheme(), bindShellControls(), ensureCss(), getTauriInvoke(), importTabModule(), initializedTabs, initTab() (+27 more)

### Community 28 - "explorer.export.js"
Cohesion: 0.12
Nodes (41): buildExplorerClientTable(), buttonFormat(), cleanHeader(), copyExportPathToClipboard(), currentLocale(), ensureExportResultBox(), explorerTable(), exportButtonInfo() (+33 more)

### Community 29 - "kaspa-gateway-node/src/lib.rs"
Cohesion: 0.11
Nodes (24): branch_for_network(), infer_binary_kind(), MiningConnectivityMode, NodeBinaryKind, NodeCapabilities, NodeCapabilityManager, NodeEndpoint, NodeError (+16 more)

### Community 30 - "byId"
Cohesion: 0.11
Nodes (31): appendLog(), bridgeAssertNoPortConflictsR5(), buildApplyPayload(), byId(), getTauriInvoke(), invokeBridgeIntegratedRuntime(), invokeWithTimeout(), KGW_BRIDGE_RUNTIME_IN_FLIGHT (+23 more)

### Community 31 - "runtime_commands.rs"
Cohesion: 0.19
Nodes (34): branch_for_network(), bridge_command_preview(), node_command_preview(), normalize_network(), normalized_bridge_kind(), normalized_node_kind(), real_bridge_default_runtime_settings(), real_bridge_runtime_apply_settings() (+26 more)

### Community 32 - "explorer.js"
Cohesion: 0.08
Nodes (36): normalizeDateInputValue(), explorerSaveAddressMemo, explorerState, formatKas(), formatUsd(), kgwClean2Kas(), kgwClean2Usd(), kgwSummaryFormatUsd() (+28 more)

### Community 33 - "installActions"
Cohesion: 0.28
Nodes (17): installActions(), installNetworkTabs(), kgwBridgeOwnedNodeLockStoreR65E(), kgwIsBridgeOwnedNodeLockedR65E(), kgwNodeApplyBridgeOwnedDisplayOnlyR65E(), kgwNodeExplicitTraceR27D(), kgwNodeHydrateBridgeOwnedDisplayOnlyR65H2(), kgwNodeInstallBridgeOwnedDisplayOnlyHydrationR65H2() (+9 more)

### Community 34 - "real_reports.rs"
Cohesion: 0.18
Nodes (33): addresses_report(), build_minimal_pdf(), build_pdf_content_stream(), csv_cell(), database_manager(), default_real_report_path(), ensure_extension(), export_real_report() (+25 more)

### Community 35 - "kaspa-gateway-security/src/lib.rs"
Cohesion: 0.11
Nodes (25): expose_secret_for_runtime(), mask_address(), mask_sensitive_value(), redact_inline_secrets(), redact_url(), RedactedString, Into, Result (+17 more)

### Community 36 - "i18n_commands.rs"
Cohesion: 0.18
Nodes (31): contains_json_file(), database_manager(), find_translation_source(), frontend_i18n_dir(), i18n_get_active_language(), i18n_import_translations(), i18n_languages(), i18n_load_catalog() (+23 more)

### Community 37 - "qs"
Cohesion: 0.15
Nodes (33): applyExplorerFiltersFromDatabase(), applyFilters(), clearExplorerTransactionTable(), kgwSummaryFormatKas(), installEvents(), kgwApplyFilterSingleOwner(), kgwDaySummaryRowsFromResult(), kgwExplorerListRequest() (+25 more)

### Community 38 - "kgw_i18n_contract_gate.cjs"
Cohesion: 0.08
Nodes (27): blockers, dictionaries, dynamicLiterals, extractDynamicLiteralFindings(), extractHtmlRefs(), extractJsRefs(), extractUnboundHtmlText(), frontendRoot (+19 more)

### Community 39 - "bridgePortProfileR35B"
Cohesion: 0.12
Nodes (31): bridgeAddUsedPortR91(), bridgeAllocateInstancePortsR8B(), bridgeAssignMissingInstancePortsR9(), bridgeChooseReplacementPortR37(), bridgeClassifyPortProfileR35B(), bridgeCollectConfiguredPortsR5(), bridgeCollectPortProfileWarningsR35B(), bridgeConfiguredPortRecordsR45() (+23 more)

### Community 40 - "address_book.rs"
Cohesion: 0.22
Nodes (28): address_book_export_csv(), address_book_export_json(), address_book_import_csv(), address_book_import_json(), address_book_stats(), AddressBookImportRecord, AddressBookIoReport, AddressBookIoRequest (+20 more)

### Community 41 - "data_enforcement_commands.rs"
Cohesion: 0.20
Nodes (27): api_probe(), data_enforcement_report(), database_manager(), is_text_source_file(), PlaceholderFinding, probe_kaspa_api(), read_setting_value(), RealApiProbeReport (+19 more)

### Community 42 - "security_audit.rs"
Cohesion: 0.19
Nodes (27): collect_manifest_files(), file_check(), is_text_like(), looks_sensitive(), policy_check(), RecoveryManifestFile, RecoveryManifestReport, RecoveryManifestRequest (+19 more)

### Community 43 - "esc"
Cohesion: 0.13
Nodes (30): bridgeInstanceExamplePlaceholderR47(), bridgeInstancePortPlaceholderR49(), bridgeInstancePromPlaceholderR49(), cardCheck(), cardInput(), cardSelect(), esc(), ic() (+22 more)

### Community 44 - "explorer_services.rs"
Cohesion: 0.20
Nodes (29): api_client(), build_minimal_pdf(), build_pdf_content_stream(), csv_cell(), database_manager(), explorer_balance(), explorer_export(), explorer_saved_addresses() (+21 more)

### Community 45 - "kgw_runtime_repository_binding_gate.cjs"
Cohesion: 0.07
Nodes (24): addFinding(), args, cargoAliases, cp, errors, expectedNetworks, files, findings (+16 more)

### Community 46 - "analysis-rust-binding.js"
Cohesion: 0.19
Nodes (28): ANALYSIS_STATE, appendLog(), bindControls(), bindWhenReady(), clearAnalysisView(), emitAnalysisData(), formatMetricValue(), getTimeRange() (+20 more)

### Community 47 - "microscopeLog"
Cohesion: 0.14
Nodes (29): addressLookupKeys(), defaultDates(), fetchBalance(), initExplorerTab(), kgwApplyPythonLoadAddressStyle(), kgwFormatExplorerBalanceKasV1(), kgwLoadExplorerRowsFromDb(), kgwStartLiveDbPollingDuringFetch() (+21 more)

### Community 48 - "installActions"
Cohesion: 0.18
Nodes (29): addInstance(), bridgeDefaultInstanceRecord(), bridgeEnsureInstanceState(), bridgeInstallAllVisibleInstanceContainerOwnersR11(), bridgeInstallInstanceContainerOwnerR11(), bridgeInstanceNetworkKeyR15(), bridgeNormalizeInstanceRecord(), bridgeReadInstanceState() (+21 more)

### Community 49 - "bridgeApplyPortAutofixR37"
Cohesion: 0.10
Nodes (29): bridgeApplyPortAutofixR37(), bridgeApplyPortConflictStartStateR33(), bridgeAssertNoPortConflictsBeforeStartR33(), bridgeAutofixButtonsR37(), bridgeAutofixChangeKeyR37(), bridgeInstallPortAutofixButtonR37(), bridgeInstanceIdFromOwnerR37(), bridgeInstancePortKindForConflictR37() (+21 more)

### Community 50 - "kgw_global_owner_gate.cjs"
Cohesion: 0.14
Nodes (28): collectFiles(), exists(), fs, isActiveFile(), isListedFile(), isOwnerEvidenceFile(), isOwnerToolFile(), isReferenceFile() (+20 more)

### Community 51 - "kgw_program_unified_gate.cjs"
Cohesion: 0.08
Nodes (21): args, cp, defaultReportDir, exists(), failedRequired, fs, jsonMode, nodeCheck() (+13 more)

### Community 52 - "fetchTransactions"
Cohesion: 0.11
Nodes (34): applyFilter(), parseDateSeconds(), fetchTransactions(), isKaspaAddress(), kgwClean2Body(), kgwClean2LoadDayTransactions(), kgwClean2LoadSummaries(), kgwClean2Log() (+26 more)

### Community 53 - "db_status_commands.rs"
Cohesion: 0.26
Nodes (26): backup_all_databases_with_label(), copy_if_exists(), database_paths_for_root(), db_operation_result(), file_status(), find_existing_file(), kgw_settings_database_backup(), kgw_settings_database_clear_caches() (+18 more)

### Community 54 - "security_hardening.rs"
Cohesion: 0.21
Nodes (22): copy_dir_recursive(), copy_file_checked(), create_recovery_snapshot(), is_disallowed_launcher(), is_sensitive_system_root(), RecoverySnapshotReport, redact_key_value(), redact_sensitive_text() (+14 more)

### Community 55 - "kgw_bridge_node_mode_routing_audit_v1.cjs"
Cohesion: 0.09
Nodes (21): blockers, cp, critical, ensureDir(), evidence, extractAround(), extractFunctionByName(), files (+13 more)

### Community 56 - "install"
Cohesion: 0.12
Nodes (30): actionName(), allButtons(), buttons(), clearFeedback(), currentLanguage(), dirtyMap(), fallbackText(), feedbackMap() (+22 more)

### Community 57 - "install"
Cohesion: 0.16
Nodes (24): actionName(), allButtons(), buttons(), clearFeedback(), currentLanguage(), dirtyMap(), fallbackText(), feedbackMap() (+16 more)

### Community 58 - "persistent_logs.rs"
Cohesion: 0.24
Nodes (21): clean_message(), clean_short_field(), log_path(), normalize_query_severity(), normalize_severity(), now_ms(), persistent_log_append(), persistent_log_clear() (+13 more)

### Community 59 - "boot"
Cohesion: 0.18
Nodes (23): activateTab(), allTabIds(), bindNavigation(), boot(), currentSelectedTabId(), kgwCloseCalendarPopoversForTabLifecycleR11B(), kgwShellCanonicalDefaultTabR63F(), kgwShellCanonicalVisibleTabsR63F() (+15 more)

### Community 60 - "log.js"
Cohesion: 0.21
Nodes (22): applyInitialState(), clearLog(), copyLog(), copyText(), currentUiLanguage(), escapeHtml(), filteredLines, highlightLine() (+14 more)

### Community 61 - "install"
Cohesion: 0.19
Nodes (22): actionButtons(), addressControls(), areFetchButtonsIdle(), beginBusy(), cancelLooksIdle(), endBusy(), filterControls(), hasSelectedAddress() (+14 more)

### Community 62 - "extractRowsFromUnifiedResult"
Cohesion: 0.10
Nodes (24): kgwClean2DayToSeconds(), kgwDayToEpochSeconds(), kgwTransactionDateKey(), kgwTxDayToEpochSeconds(), extractRowsFromUnifiedResult(), kgwApplyExplorerFontSize(), kgwBindFontSpinbox(), kgwBuildExplorerListRequest() (+16 more)

### Community 63 - "transaction_analysis.rs"
Cohesion: 0.24
Nodes (19): address_direction_breakdown(), AddressFlowSummary, analyze_address_flow(), analyze_all_transactions(), database_manager(), GlobalTransactionAnalysis, local_rich_list(), normalize_direction() (+11 more)

### Community 64 - "buildCommandLines"
Cohesion: 0.21
Nodes (20): addBoolValue(), addFlag(), addRawValue(), addValue(), bridgeControl(), bridgeControlCard(), bridgeExternalBasePortR91(), bridgeHasConfig() (+12 more)

### Community 65 - "definitions"
Cohesion: 0.15
Nodes (13): definitions, Number, PermissionEntry, Target, Value, anyOf, description, anyOf (+5 more)

### Community 66 - "properties"
Cohesion: 0.15
Nodes (13): properties, Identifier, default, description, type, description, oneOf, type (+5 more)

### Community 67 - "diagnostics.rs"
Cohesion: 0.27
Nodes (17): append_log(), clean_field(), clear_logs(), DesktopDiagnosticsReport, DesktopLogEntry, diagnostics_report(), list_logs(), LogState (+9 more)

### Community 68 - "tauri.conf.json"
Cohesion: 0.10
Nodes (19): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, frontendDist (+11 more)

### Community 69 - "package.json"
Cohesion: 0.11
Nodes (18): dependencies, @tauri-apps/plugin-dialog, description, devDependencies, @tauri-apps/cli, name, private, scripts (+10 more)

### Community 70 - "setLanguage"
Cohesion: 0.16
Nodes (18): applyDictionary(), bindMissingI18nAttributesR99(), buildKgwI18nReverseIndexR99(), buildKgwI18nRuntimeReverseIndexR102(), currentLanguage(), flatten(), flattenKgwI18nDictionaryR100(), installDynamicKgwI18nObserverR102() (+10 more)

### Community 71 - "definitions"
Cohesion: 0.15
Nodes (13): definitions, Number, PermissionEntry, Target, Value, anyOf, description, anyOf (+5 more)

### Community 72 - "apply"
Cohesion: 0.21
Nodes (17): apply(), defaults(), keys(), kgwShellApplyDisplayOwnerBeforeBootR63F(), kgwShellDisplayOwnerPreferredTabIdR59C(), kgwShellDisplayOwnerPublishR59C(), kgwShellDisplayOwnerResolveTabIdR59C(), kgwShellDisplayOwnerTabButtonsR59C() (+9 more)

### Community 73 - "kgwBridgeR51ReadSettings"
Cohesion: 0.16
Nodes (19): kgwBridgeR51CaptureFactoryDefaults(), kgwBridgeR51Fields(), kgwBridgeR51Keys(), kgwBridgeR51Load(), kgwBridgeR51LoadSavedSettings(), kgwBridgeR51Panel(), kgwBridgeR51ReadCommandOptionsR38C(), kgwBridgeR51ReadInstanceCommandOptionsR38C() (+11 more)

### Community 74 - "properties"
Cohesion: 0.15
Nodes (13): properties, Identifier, default, description, type, description, oneOf, type (+5 more)

### Community 75 - "kgw_i18n_locale_coverage_gate.cjs"
Cohesion: 0.12
Nodes (14): approved, approvedSameAsEnglish, criticalKeys, dictionaries, frontendRoot, fs, i18nDir, isApprovedSameAsEnglish() (+6 more)

### Community 76 - "cardInput"
Cohesion: 0.29
Nodes (15): cardCheck(), cardInput(), cardSelect(), esc(), kgwNodeCommandInlineToggleR7(), kgwNodeNetworkPolicyMessage(), renderDatabase(), renderNetwork() (+7 more)

### Community 77 - "settings.rs"
Cohesion: 0.45
Nodes (15): database_manager(), default_full_settings(), DesktopApiProfile, DesktopFullSettings, load_full_settings(), reset_full_settings(), Result, String (+7 more)

### Community 78 - "kgw_runtime_trace_owner_audit_v20.cjs"
Cohesion: 0.17
Nodes (10): cp, finish(), fs, mkdirp(), path, read(), report, run() (+2 more)

### Community 79 - "kgwShellSetSelectOptionsVisibleR73"
Cohesion: 0.20
Nodes (15): applyTabs(), kgwMainTabTraceR35C(), kgwShellApplyDisplayPreferencesDirectR73(), kgwShellApplyLooseMenuVisibilityR75(), kgwShellApplySelectVisibilityR75(), kgwShellAsArrayR75(), kgwShellCollectSelectsForKindR75(), kgwShellKnownCurrenciesR75() (+7 more)

### Community 80 - "updateCommand"
Cohesion: 0.21
Nodes (15): addFlag(), addHostPort(), addValue(), buildCommandLines(), c(), kgwNodeCommandInlineStateKeyR7(), kgwNodeCommandInlineStateR7(), kgwNodeCommandOptionEnabledR7() (+7 more)

### Community 81 - "network_full.rs"
Cohesion: 0.40
Nodes (14): fetch_optional_json(), find_number_string(), first_number_string(), full_network_analytics_report(), FullNetworkAnalyticsReport, metric_card(), NetworkMetricCard, push_warning_if_missing() (+6 more)

### Community 82 - "normalizeDateInputValue"
Cohesion: 0.17
Nodes (14): AppSettingsRepository, DatabaseManager, DatabasePaths, initialize_addr_schema(), initialize_addresses_schema(), initialize_app_data_schema(), initialize_transactions_schema(), initialize_tx_schema() (+6 more)

### Community 83 - "kgwNodeR51RefreshOne"
Cohesion: 0.18
Nodes (17): getTauriInvoke(), invokeNodeIntegratedRuntime(), invokeWithTimeout(), kgwExtractNodeOwnerFlags(), kgwGuardBlockReasonV3(), kgwLoadNodeOwnerCommandPreview(), kgwNodeAssertStartEvidence(), kgwNodeR51BridgeInprocessLockedV7() (+9 more)

### Community 84 - "kgw_parallel_self_worker_runtime_gate.cjs"
Cohesion: 0.15
Nodes (13): checks, controller, fail(), failed, files, forbidden, forbiddenHits, fs (+5 more)

### Community 85 - "installForNetwork"
Cohesion: 0.23
Nodes (12): applyFontSize(), clampSize(), installAll(), installForNetwork(), logOutput(), makeButton(), readSize(), removeToolbarDuplicates() (+4 more)

### Community 86 - "permissions"
Cohesion: 0.17
Nodes (12): $ref, array, null, description, items, type, uniqueItems, description (+4 more)

### Community 87 - "kgw_runtime_repository_binding_apply.ps1"
Cohesion: 0.30
Nodes (8): Branch-Function(), Bridge-Family-Function(), Cargo-Line(), Get-Family-Groups(), Get-Spec(), Network-Arm(), Node-Family-Function(), Revision-Function()

### Community 88 - "shell-logger.js"
Cohesion: 0.29
Nodes (7): createLogger(), getTauriInvoke(), kgwFatal(), kgwLog(), pushLocal(), safeDetails(), sendToTauriLog()

### Community 89 - "invokeCommand"
Cohesion: 0.21
Nodes (12): invokeCommand(), kgwExplorerIsKaspaAddress(), kgwExplorerManualAddressValue(), kgwExplorerSaveManualAddress(), kgwI18nTextR41(), kgwInstallExplorerManualAddressSave(), kgwInvokeExplorerCancelTransactionsR57D4(), kgwInvokeExplorerDaySummaries() (+4 more)

### Community 90 - "default.json"
Cohesion: 0.18
Nodes (10): description, identifier, permissions, $schema, windows, core:default, dialog:allow-open, dialog:allow-save (+2 more)

### Community 91 - "Capability"
Cohesion: 0.33
Nodes (6): description, required, type, Capability, identifier, permissions

### Community 92 - "webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 93 - "webviews"
Cohesion: 0.20
Nodes (10): type, webviews, windows, items, description, items, type, description (+2 more)

### Community 94 - "kgwInvokeExplorerDaySummaries"
Cohesion: 0.12
Nodes (21): address_record_from_row(), AddressesRepository, AddressRecord, DatabaseKind, DbError, is_kaspa_address_like(), KnownNameRecord, normalize_direction() (+13 more)

### Community 95 - "CapabilityRemote"
Cohesion: 0.22
Nodes (9): description, properties, required, type, CapabilityRemote, urls, urls, description (+1 more)

### Community 96 - "CapabilityRemote"
Cohesion: 0.22
Nodes (9): description, properties, required, type, CapabilityRemote, urls, urls, description (+1 more)

### Community 98 - "probe"
Cohesion: 0.43
Nodes (7): argument_value(), main(), normalized_network(), probe(), Option, Result, String

### Community 100 - "initKaspaBridgeTab"
Cohesion: 0.08
Nodes (24): For /graphify add and --watch, For /graphify query, For the commit hook and native CLAUDE.md integration, For --update and --cluster-only, /graphify, Honesty Rules, Interpreter guard for subcommands, Part A - Structural extraction for code files (+16 more)

### Community 101 - "release_qa.rs"
Cohesion: 0.43
Nodes (7): file_check(), final_release_qa_report(), FinalReleaseQaReport, policy_check(), QaCheck, String, Vec

### Community 102 - "kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5"
Cohesion: 0.07
Nodes (22): assert, assertIncludes(), ClassList, createHarness(), dataKey(), dynamicClickTests(), extractBetween(), flush() (+14 more)

### Community 103 - "permissions"
Cohesion: 0.17
Nodes (12): $ref, array, null, description, items, type, uniqueItems, description (+4 more)

### Community 104 - "ui_wiring.rs"
Cohesion: 0.43
Nodes (6): feature_wiring_report(), FeatureWiringItem, FeatureWiringReport, item(), String, Vec

### Community 106 - "unique_test_dir"
Cohesion: 0.53
Nodes (5): PathBuf, runtime_health_report_is_healthy(), runtime_initializes_config_database_api_and_node_layers(), runtime_paths_can_be_constructed_explicitly(), unique_test_dir()

### Community 107 - "kgwStrictNormalizeAllPanels"
Cohesion: 0.60
Nodes (5): kgwStrictEnsureSinglePanel(), kgwStrictHidePanel(), kgwStrictNormalizeAllPanels(), kgwStrictSetImportant(), kgwStrictShowPanel()

### Community 109 - "kgwInstallNodeLogAutoScrollControlsR27"
Cohesion: 0.11
Nodes (17): Acknowledgements, Current verified desktop release, Git hygiene, GitHub Actions status, Kaspa Gateway Rust, License, Prerequisites, Project status (+9 more)

### Community 111 - "kgw_live_network_smoke.ps1"
Cohesion: 0.60
Nodes (4): Install-WingetPackage(), Invoke-LiveProbe(), Refresh-ProcessPath(), Wait-RpcReady()

### Community 112 - "kgw_runtime_repository_binding_audit.cjs"
Cohesion: 0.40
Nodes (4): args, cp, extraArgs, result

### Community 131 - "database_root"
Cohesion: 0.33
Nodes (11): build_manager_unlocked(), database_root(), lock(), F, Mutex, PathBuf, Result, String (+3 more)

### Community 132 - "explorer_tab.rs"
Cohesion: 0.36
Nodes (10): database_manager(), explorer_transactions_page(), ExplorerTransactionRow, ExplorerTransactionsPage, ExplorerTransactionsPageRequest, Option, Result, String (+2 more)

### Community 133 - "ADR-0010: Same-EXE Parallel Self-Worker Runtime"
Cohesion: 0.18
Nodes (10): ADR-0010: Same-EXE Parallel Self-Worker Runtime, Decision, Frontend role, Guard, Network metadata owner, Non-goals, Ownership, Rationale (+2 more)

### Community 134 - "Kaspa Gateway Agent Workflow"
Cohesion: 0.20
Nodes (9): Definition Of Done, Git Discipline, Kaspa Gateway Agent Workflow, Local-First Workflow, Mandatory Graphify Lifecycle, Network Invariants, Raw Process Log Invariants, Repository Structure (+1 more)

### Community 135 - "Canonical mechanism"
Cohesion: 0.20
Nodes (9): Canonical mechanism, CPU miner, Embedded Bridge Runtime Command Contract, External node mode, Goal, In-process bridge mode, Log isolation, Mainnet node UI rule (+1 more)

### Community 136 - "Runtime Network Repository Bindings"
Cohesion: 0.20
Nodes (9): Bridge Cargo dependencies, Current binding, Decision, Important rule, Node Cargo dependencies, Permanent audit, Real owners, Runtime network mapping (+1 more)

### Community 137 - "graphify reference: extra exports and benchmark"
Cohesion: 0.22
Nodes (8): graphify reference: extra exports and benchmark, Step 6b - Wiki (only if --wiki flag), Step 7 - Neo4j export (only if --neo4j or --neo4j-push flag), Step 7a - FalkorDB export (only if --falkordb or --falkordb-push flag), Step 7b - SVG export (only if --svg flag), Step 7c - GraphML export (only if --graphml flag), Step 7d - MCP server (only if --mcp flag), Step 8 - Token reduction benchmark (only if total_words > 5000)

### Community 138 - "db_foundation_tests.rs"
Cohesion: 0.39
Nodes (7): addresses_schema_can_be_initialized(), app_data_schema_can_be_initialized(), database_paths_are_created_from_root(), manager_initializes_all_database_files(), PathBuf, transactions_schema_can_be_initialized(), unique_test_dir()

### Community 139 - "repository_tests.rs"
Cohesion: 0.39
Nodes (7): addresses_repository_upserts_gets_lists_and_deletes(), cache_repository_respects_expiration(), PathBuf, settings_repository_sets_gets_and_deletes_values(), test_manager(), transactions_repository_upserts_lists_counts_and_deletes(), unique_test_dir()

### Community 140 - "AI Development Workflow"
Cohesion: 0.22
Nodes (8): AI Development Workflow, Codex Instruction Verification, Generated Graph Snapshot Policy, Graphify Query-Before-Source Workflow, Local Commit Policy, Local-First Lifecycle, Safe Incremental Graph Refresh, Test Selection

### Community 141 - "Security Policy"
Cohesion: 0.22
Nodes (8): Current Security Baseline, Dependency Risk Policy, Not Yet Claimed, Release Asset Expectations, Reporting a Vulnerability, Runtime Security Rules, Security Policy, Supported Repository

### Community 142 - "Bridge README Runtime Instance Contract"
Cohesion: 0.25
Nodes (7): Bridge README Runtime Instance Contract, CPU miner, Instance contract, Mode contract, Source, Status, UI ownership rule

### Community 143 - "Main features"
Cohesion: 0.25
Nodes (8): Bridge management, Explorer and analysis, Internationalization, Main features, Multi-network runtime management, Node management, Official Kaspa runtime bindings, Raw log panes

### Community 144 - "KGW Security Baseline"
Cohesion: 0.29
Nodes (6): Before Claiming Hardened Status, KGW Security Baseline, Latest Corrected Baseline, Purpose, Required Local Commands, Runtime Repository Binding Expectations

### Community 145 - "Desktop UI overview"
Cohesion: 0.29
Nodes (7): Analysis tab, Bridge tab, Desktop UI overview, Explorer tab, Header, Node tab, Settings tab

### Community 146 - "Development commands"
Cohesion: 0.29
Nodes (7): Build desktop app, Check desktop backend, Check Rust formatting, Development commands, Format Rust, JavaScript syntax checks, Run desktop dev shell

### Community 147 - "Troubleshooting"
Cohesion: 0.29
Nodes (7): Header clock appears oversized, Language switching issues, Node or bridge remains running after app close, Raw bridge logs are empty, Raw node logs are empty, Release build does not produce installer, Troubleshooting

### Community 148 - "Capability"
Cohesion: 0.33
Nodes (6): description, required, type, Capability, identifier, permissions

### Community 149 - "graphify reference: query, path, explain"
Cohesion: 0.33
Nodes (5): For /graphify explain, For /graphify path, graphify reference: query, path, explain, Step 0 — Constrained query expansion (REQUIRED before traversal), Step 1 — Traversal

### Community 151 - "Dependency Risk Register"
Cohesion: 0.33
Nodes (5): Current Action, Current Classification, Dependency Risk Register, Policy, Required Review Questions

### Community 152 - "desktop-schema.json"
Cohesion: 0.40
Nodes (4): anyOf, description, $schema, title

### Community 153 - "windows-schema.json"
Cohesion: 0.40
Nodes (4): anyOf, description, $schema, title

### Community 154 - "AppSettingsRepository"
Cohesion: 0.14
Nodes (20): AppRuntime, readiness_report_initializes_required_files(), AsRef, Path, PathBuf, Result, Self, String (+12 more)

### Community 155 - "Live Kaspa Network Smoke Test"
Cohesion: 0.40
Nodes (4): Live Kaspa Network Smoke Test, Operating policy, Production readiness, Run on Windows

### Community 156 - "Raw log model"
Cohesion: 0.40
Nodes (5): Auto-scroll, Bridge raw logs, Correct log path, Node raw logs, Raw log model

### Community 157 - "Node and bridge behavior"
Cohesion: 0.40
Nodes (5): Bridge modes, Bridge runtime, Mining policy, Node and bridge behavior, Node runtime

### Community 158 - "kgw_ai_workflow_gate.ps1"
Cohesion: 0.80
Nodes (4): Add-Failure(), Read-RequiredFile(), Require-AgentsPattern(), Test-JsonFile()

### Community 159 - "Kaspa Gateway Desktop"
Cohesion: 0.50
Nodes (3): Commands, Kaspa Gateway Desktop, Security posture

### Community 160 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 161 - "local"
Cohesion: 0.50
Nodes (4): default, description, type, local

### Community 162 - "graphify reference: add a URL and watch a folder"
Cohesion: 0.50
Nodes (3): For /graphify add, For --watch, graphify reference: add a URL and watch a folder

### Community 163 - "graphify reference: commit hook and native CLAUDE.md integration"
Cohesion: 0.50
Nodes (3): For git commit hook, For native CLAUDE.md integration, graphify reference: commit hook and native CLAUDE.md integration

### Community 164 - "graphify reference: incremental update and cluster-only"
Cohesion: 0.50
Nodes (3): For --cluster-only, For --update (incremental re-extraction), graphify reference: incremental update and cluster-only

### Community 165 - "Architecture overview"
Cohesion: 0.50
Nodes (4): Architecture overview, Backend layer, Frontend layer, Runtime crates

### Community 166 - "Configuration"
Cohesion: 0.50
Nodes (4): Configuration, Frontend configuration, Runtime repository bindings, Tauri configuration

### Community 167 - "Repository links and runtime bindings"
Cohesion: 0.50
Nodes (4): Important runtime policy, Main project repository, Repository links and runtime bindings, Runtime source repositories

### Community 170 - "Maintainer rules"
Cohesion: 0.67
Nodes (3): Do, Do not, Maintainer rules

### Community 171 - "Internationalization"
Cohesion: 0.67
Nodes (3): i18n gates, i18n rule, Internationalization

### Community 174 - "initKaspaBridgeTab"
Cohesion: 0.22
Nodes (13): BRIDGE_NETWORKS, bridgeSyncAllModeControls(), initKaspaBridgeTab(), installKgwBridgeR51BottomStyle(), kgwBridgeLogAutoScrollEnabledR27(), kgwBridgeLogAutoScrollKeyR27(), kgwBridgeNetworkPolicyMessage(), kgwBridgeNetworkProfile() (+5 more)

### Community 175 - "kgwInstallNodeLogAutoScrollControlsR27"
Cohesion: 0.20
Nodes (15): appendLog(), byId(), initKaspaNodeTab(), installKgwNodeR51BottomStyle(), kgwI18nTextR41(), kgwInstallNodeLogAutoScrollControlsR27(), kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(), kgwNodeLogAutoScrollEnabledR27() (+7 more)

### Community 176 - "kgwInvokeExplorerGroupedTransactions"
Cohesion: 0.11
Nodes (25): api_settings_to_value(), ApiProfile, ApiSettings, ConfigError, DbFilenames, decrypt_legacy_api_keys(), default_api_profile(), default_api_profile_contains_python_endpoints() (+17 more)

### Community 177 - "kgwNodeSmallOwnerTraceR44D"
Cohesion: 0.26
Nodes (13): kgwNodeR51CaptureFactoryDefaults(), kgwNodeR51Keys(), kgwNodeR51Load(), kgwNodeR51LoadSavedSettings(), kgwNodeR51ReadCommandOptionsR38C(), kgwNodeR51ReadSettings(), kgwNodeR51RefreshAll(), kgwNodeR51RestoreDefaults() (+5 more)

### Community 178 - "kgwSettingsTraceButtonDetailsR29B"
Cohesion: 0.26
Nodes (8): build_official_kaspa_runtime_plan_v1(), KaspaRuntimeError, KaspaRuntimeNetwork, Result, Self, runtime_service_events_from_settings_v1(), validate_listen(), validate_safe_value()

### Community 179 - "kgw_start_button_gate.ps1"
Cohesion: 0.83
Nodes (3): Add-Failure(), Read-RequiredFile(), Run-LocalCommand()

### Community 180 - "database_paths_from_default_user_dir"
Cohesion: 0.36
Nodes (9): Cli, Commands, database_manager_from_default_user_dir(), database_paths_from_default_user_dir(), main(), Box, Error, Result (+1 more)

### Community 181 - "kgw_inproc_node_only.rs"
Cohesion: 0.27
Nodes (5): KgwInProcNodeOnly, KgwInProcNodeOnlyStatus, main_handle_reports_kgw_mechanism_ported(), main_handle_returns_inproc_plan(), Self

### Community 182 - "kaspa-gateway-rk-node/src/lib.rs"
Cohesion: 0.39
Nodes (8): all_parallel_runtime_plans_log_v1(), Option, Result, String, runtime_owner_plan_for_network_v1(), runtime_owner_status_for_network_v1(), runtime_owner_summary_v1(), official_kaspa_runtime_summary_v1()

### Community 183 - "kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5"
Cohesion: 0.33
Nodes (7): kgwNodeApplyRustyKaspaRootOnlyDefaultPathsR5(), kgwNodeBackendInvokeR5(), kgwNodeExtractUserLocalAppDataR5(), kgwNodeIsEmptyOrGeneratedPathR5(), kgwNodeJoinPathR5(), kgwNodeLoadEnvironmentPathHintsR5(), kgwNodeRustyKaspaLocalAppDataRootR5()

### Community 184 - "kgw_inproc_owner_tests.rs"
Cohesion: 0.29
Nodes (5): kgw_inproc_summary_v1(), explicit_opt_in_does_not_start_without_owned_kaspad_dependencies(), kgw_inproc_lifecycle_matches_reference_shape(), kgw_inproc_owner_is_single_canonical_owner(), summary_declares_no_clone_no_source_cache()

## Knowledge Gaps
- **432 isolated node(s):** `KGW_NODE_R51_LAST_STATUS`, `KGW_NODE_R51_LAST_LOGS`, `KGW_NODE_R51_LAST_ACTIVITY_NOTICE`, `KGW_NODE_R51_TRANSITIONS`, `kgwNodeSettingsFeedbackLocksR11` (+427 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **19 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `DatabaseManager` connect `normalizeDateInputValue` to `Result`, `database_root`, `explorer_tab.rs`, `export_commands.rs`, `commands.rs`, `top_addresses_deep.rs`, `default_user_data_dir`, `repository_tests.rs`, `settings_commands.rs`, `python_migration_real.rs`, `migration.rs`, `config_commands.rs`, `analysis_commands.rs`, `AppSettingsRepository`, `real_reports.rs`, `i18n_commands.rs`, `address_book.rs`, `data_enforcement_commands.rs`, `explorer_services.rs`, `database_paths_from_default_user_dir`, `transaction_analysis.rs`, `settings.rs`, `kgwInvokeExplorerDaySummaries`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **Why does `default_user_data_dir()` connect `kaspa-gateway-config/src/lib.rs` to `database_root`, `explorer_tab.rs`, `export_commands.rs`, `commands.rs`, `app_logger.rs`, `top_addresses_deep.rs`, `default_user_data_dir`, `settings_commands.rs`, `python_migration_real.rs`, `migration.rs`, `config_commands.rs`, `analysis_commands.rs`, `AppSettingsRepository`, `real_reports.rs`, `i18n_commands.rs`, `address_book.rs`, `data_enforcement_commands.rs`, `explorer_services.rs`, `database_paths_from_default_user_dir`, `security_hardening.rs`, `persistent_logs.rs`, `transaction_analysis.rs`, `settings.rs`?**
  _High betweenness centrality (0.039) - this node is a cross-community bridge._
- **Why does `AppRuntime` connect `AppSettingsRepository` to `kaspa-gateway-config/src/lib.rs`, `Result`, `kaspa-gateway-core/src/lib.rs`, `normalizeDateInputValue`, `kaspa-gateway-node/src/lib.rs`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **What connects `KGW_NODE_R51_LAST_STATUS`, `KGW_NODE_R51_LAST_LOGS`, `KGW_NODE_R51_LAST_ACTIVITY_NOTICE` to the rest of the system?**
  _432 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `KgwNetwork` be split into smaller, more focused modules?**
  _Cohesion score 0.05604719764011799 - nodes in this community are weakly interconnected._
- **Should `settings.js` be split into smaller, more focused modules?**
  _Cohesion score 0.06545114539504442 - nodes in this community are weakly interconnected._
- **Should `analysis.js` be split into smaller, more focused modules?**
  _Cohesion score 0.057387057387057384 - nodes in this community are weakly interconnected._