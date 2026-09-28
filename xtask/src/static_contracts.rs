use regex::Regex;
use std::fs;
use std::path::Path;

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("static contract read failed for {relative}: {error}"))
}

fn require_contains(source: &str, needle: &str, message: &str) -> Result<(), String> {
    if source.contains(needle) {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn forbid_contains(source: &str, needle: &str, message: &str) -> Result<(), String> {
    if source.contains(needle) {
        Err(message.to_owned())
    } else {
        Ok(())
    }
}

fn count_regex(source: &str, pattern: &str) -> usize {
    Regex::new(pattern).unwrap().find_iter(source).count()
}

fn analysis_contract(root: &Path) -> Result<(), String> {
    let binding = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js",
    )?;
    let rust_owner = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/analysis_binding.rs",
    )?;
    let settings_owner = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/settings_addresses.rs",
    )?;
    let backend = read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs",
    )?;
    validate_analysis(&binding, &rust_owner, &settings_owner, &backend)
}

fn validate_analysis(
    binding: &str,
    rust_owner: &str,
    settings_owner: &str,
    backend: &str,
) -> Result<(), String> {
    for needle in [
        "wasmAnalysisNormalizeTimeRange(value)",
        "wasmAnalysisInstallBinding()",
        "window.kgwRunRustAnalysis = runRustAnalysis",
    ] {
        require_contains(
            binding,
            needle,
            "generated Analysis adapter contract missing",
        )?;
    }
    for forbidden in ["analysis_report", "get_all_addresses", "raw_sompi"] {
        forbid_contains(
            binding,
            forbidden,
            "generated Analysis adapter must not regain implementation logic",
        )?;
    }
    if ![
        "\"30d\" => \"last_month\"",
        "\"90d\" => \"last_3_months\"",
        "\"1y\" => \"last_year\"",
    ]
    .iter()
    .all(|needle| rust_owner.contains(needle))
    {
        return Err("Rust frontend range aliases missing".to_owned());
    }
    if !rust_owner.contains("kgw:saved-addresses-changed")
        || !settings_owner.contains("fn notify_saved_addresses_changed()")
        || !settings_owner.contains("kgw:saved-addresses-changed")
        || !settings_owner.contains(r#"call1(&window(), "dispatchEvent", &event)"#)
    {
        return Err("saved-address invalidation Rust-owner contract missing".to_owned());
    }
    if !backend.contains("fn select_latest_records")
        || !backend.contains("select_latest_records(&mut records, request.limit)")
    {
        return Err("canonical latest-N selection missing".to_owned());
    }
    if ![
        "\"30d\" => \"last_month\"",
        "\"90d\" => \"last_3_months\"",
        "\"1y\" => \"last_year\"",
    ]
    .iter()
    .all(|needle| backend.contains(needle))
    {
        return Err("backend alias normalization missing".to_owned());
    }
    Ok(())
}
fn explorer_lint_contract(root: &Path) -> Result<(), String> {
    let source = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
    )?;
    validate_explorer_lint(&source)
}

fn validate_explorer_lint(source: &str) -> Result<(), String> {
    for (needle, message) in [
        (
            "kgwFormatUsd(",
            "Explorer must not reference removed kgwFormatUsd alias",
        ),
        (
            "kgwSummaryDayToSeconds(",
            "Explorer must not reference removed summary day alias",
        ),
        (
            "setInterval(tick, 2000)",
            "Explorer polling must not reference undefined tick",
        ),
        (
            "setInterval(kgwLiveDbPollingTick, 2000)",
            "Explorer must not retain retired local DB polling after event-driven live core ownership",
        ),
    ] {
        forbid_contains(source, needle, message)?;
    }
    for (needle, message) in [
        (
            "kgwSummaryFormatUsd(",
            "Explorer must use imported summary USD formatter",
        ),
        (
            "kgwDayToEpochSeconds(day, false)",
            "Explorer day start must use imported UTC day helper",
        ),
        (
            "kgwDayToEpochSeconds(day, true)",
            "Explorer day end must use imported UTC day helper",
        ),
        (
            "await kgwInstallTxLiveCoreListener();",
            "Explorer fetch must install the event-driven live-core owner",
        ),
        (
            "await listen(\"kgw://transactions/page-stored\"",
            "Explorer live core must consume Rust page-stored events",
        ),
    ] {
        require_contains(source, needle, message)?;
    }
    Ok(())
}

fn functional_ui_contract(root: &Path) -> Result<(), String> {
    let explorer_js = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
    )?;
    let explorer_address_rust = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/explorer_addresses.rs",
    )?;
    let explorer_filters_rust = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/explorer_filters.rs",
    )?;
    let explorer_css = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.css",
    )?;
    let settings_paths = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/settings_paths.rs",
    )?;
    let top_js = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js",
    )?;
    let top_rust = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/top_addresses.rs",
    )?;
    let top_html = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.html",
    )?;
    let top_template = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.template.js",
    )?;
    validate_functional_ui(FunctionalUiSources {
        explorer_js: &explorer_js,
        explorer_address_rust: &explorer_address_rust,
        explorer_filters_rust: &explorer_filters_rust,
        explorer_css: &explorer_css,
        settings_paths: &settings_paths,
        top_js: &top_js,
        top_rust: &top_rust,
        top_html: &top_html,
        top_template: &top_template,
    })
}

struct FunctionalUiSources<'a> {
    explorer_js: &'a str,
    explorer_address_rust: &'a str,
    explorer_filters_rust: &'a str,
    explorer_css: &'a str,
    settings_paths: &'a str,
    top_js: &'a str,
    top_rust: &'a str,
    top_html: &'a str,
    top_template: &'a str,
}

fn validate_explorer_filter_ownership(
    explorer_js: &str,
    explorer_filters_rust: &str,
) -> Result<(), String> {
    for export in [
        "#[wasm_bindgen(js_name = explorerEnsureFilterOptions)]",
        "#[wasm_bindgen(js_name = explorerReadFilterState)]",
        "#[wasm_bindgen(js_name = explorerBuildListRequest)]",
        "#[wasm_bindgen(js_name = explorerFilterValue)]",
        "#[wasm_bindgen(js_name = explorerFilterBuildRequest)]",
        "#[wasm_bindgen(js_name = explorerNormalizeTxTypeFilterValue)]",
        "#[wasm_bindgen(js_name = explorerNormalizeDirectionFilterValue)]",
        "#[wasm_bindgen(js_name = explorerRepairFilterSelects)]",
        "#[wasm_bindgen(js_name = explorerInstallFilterSelectRepair)]",
        "#[wasm_bindgen(js_name = explorerClean2Request)]",
    ] {
        require_contains(
            explorer_filters_rust,
            export,
            "Explorer filter/control/request Rust-owner export missing",
        )?;
    }
    for legacy_definition in [
        "function kgwEnsureExplorerFilterOptions(",
        "function kgwReadExplorerFilterState(",
        "function kgwBuildExplorerListRequest(",
        "function kgwFilterValue(",
        "function kgwFilterBuildRequest(",
        "function kgwNormalizeTxTypeFilterValue(",
        "function kgwNormalizeDirectionFilterValue(",
        "function kgwRepairExplorerFilterSelects(",
        "function kgwRepairExplorerFilterSelectsNow(",
        "function kgwClean2Request(",
        "if (!window.__kgwFilterSelectsInitRepairInstalled)",
    ] {
        forbid_contains(
            explorer_js,
            legacy_definition,
            "Explorer filter/control/request implementation must not return to JavaScript",
        )?;
    }
    Ok(())
}

fn validate_functional_ui(sources: FunctionalUiSources<'_>) -> Result<(), String> {
    let FunctionalUiSources {
        explorer_js,
        explorer_address_rust,
        explorer_filters_rust,
        explorer_css,
        settings_paths,
        top_js,
        top_rust,
        top_html,
        top_template,
    } = sources;
    forbid_contains(
        explorer_css,
        "#explorer #explorerStatus {\n  display: none !important;",
        "Explorer status must not be permanently hidden",
    )?;
    let invalid_address_error_statuses = count_regex(
        explorer_js,
        r#"setStatus\((?:section|root), "Enter a valid Kaspa address\.", "error"\)"#,
    );
    let canonical_address_validations =
        count_regex(explorer_js, r"await kgwCanonicalKaspaAddress\(address\)");
    if canonical_address_validations < 3 {
        return Err(
            "Explorer must retain all active canonical address validation owners".to_owned(),
        );
    }
    if invalid_address_error_statuses != canonical_address_validations {
        return Err("All invalid Explorer actions need visible error status".to_owned());
    }
    require_contains(
        explorer_js,
        r#"node.setAttribute("aria-live", state === "error" ? "assertive" : "polite")"#,
        "Explorer ARIA-live state missing",
    )?;
    require_contains(
        explorer_address_rust,
        r#""validate_kaspa_address""#,
        "Explorer Rust owner must use canonical backend address validation",
    )?;
    require_contains(
        explorer_address_rust,
        "explorerCanonicalKaspaAddress",
        "Explorer canonical validation must remain exported by Rust/WASM",
    )?;
    validate_explorer_filter_ownership(explorer_js, explorer_filters_rust)?;
    for needle in [
        r#"function(&dialog, "open")"#,
        r#""Choose directory""#,
        r#""settings_validate_custom_path""#,
        r#""Browse cancelled; path unchanged.""#,
    ] {
        require_contains(
            settings_paths,
            needle,
            "Settings Browse Rust-owner contract missing",
        )?;
    }
    if !top_html.contains(r#"id="topAddressesStatus""#)
        || !top_template.contains("topAddressesStatus")
    {
        return Err("Top Addresses status target missing from runtime template".to_owned());
    }
    for needle in [
        "@generated",
        "wasmInitTopAddressesTab()",
        "wasmRefreshTopAddresses()",
    ] {
        require_contains(
            top_js,
            needle,
            "Top Addresses JavaScript must remain deterministic Rust/WASM ABI glue",
        )?;
    }
    for forbidden in [
        "fetch_top_addresses_rust",
        "export_default_path",
        "export_report",
        "kgw_open_exported_file_v1",
        "kgw_frontend_button_trace_v1",
        "querySelector(",
        "addEventListener(",
    ] {
        forbid_contains(
            top_js,
            forbidden,
            "Top Addresses implementation must not return to generated JavaScript",
        )?;
    }
    for state in ["LOADING —", "EMPTY —", "ERROR —", "SUCCESS —"] {
        if !top_rust.contains(state) {
            return Err(format!("Top Addresses Rust owner missing {state} state"));
        }
    }
    for command in [
        "fetch_top_addresses_rust",
        "export_default_path",
        "export_report",
        "kgw_open_exported_file_v1",
        "kgw_frontend_button_trace_v1",
        "KGW_TOP_ADDRESSES_SAFE_CONTROLS_TRACE_PATCH_R49D",
        "top-addresses-installButtonHandlers-safe-owner",
    ] {
        if !top_rust.contains(command) {
            return Err(format!(
                "Top Addresses Rust owner missing contract {command}"
            ));
        }
    }
    Ok(())
}
fn settings_workflow_contract(root: &Path) -> Result<(), String> {
    let settings_adapter = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
    )?;
    let settings_ui = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/settings_ui.rs",
    )?;
    let settings_profiles = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/settings_profiles.rs",
    )?;
    let settings_addresses = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/settings_addresses.rs",
    )?;
    let backend = read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/settings_commands.rs",
    )?;
    let address_book = read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/address_book.rs",
    )?;
    validate_settings_workflow(
        &settings_adapter,
        &settings_ui,
        &settings_profiles,
        &settings_addresses,
        &backend,
        &address_book,
    )
}

fn validate_settings_workflow(
    settings_adapter: &str,
    settings_ui: &str,
    settings_profiles: &str,
    settings_addresses: &str,
    backend: &str,
    address_book: &str,
) -> Result<(), String> {
    for needle in [
        "@generated",
        "wasmSettingsUiInitTab()",
        "wasmAddressesInstallAll()",
    ] {
        require_contains(
            settings_adapter,
            needle,
            "generated Settings adapter wiring contract missing",
        )?;
    }
    for forbidden in [
        "settings_profile_add",
        "settings_profile_rename",
        "settings_profile_delete",
        "settings_profile_select",
        "settings_reset_selected_endpoint",
        "address_book_export_json",
        "address_book_import_json",
        "dialog.save",
        "dialog.open",
        "placeholderActions",
    ] {
        forbid_contains(
            settings_adapter,
            forbidden,
            "generated Settings adapter must not regain workflow implementation",
        )?;
    }
    for needle in [
        "crate::settings_profiles::install();",
        "crate::settings_paths::browse(target_id).await",
        "crate::settings_addresses::install_io()",
    ] {
        require_contains(settings_ui, needle, "Settings Rust UI owner wiring missing")?;
    }
    for command in [
        "settings_profile_add",
        "settings_profile_rename",
        "settings_profile_delete",
        "settings_profile_select",
        "settings_reset_selected_endpoint",
    ] {
        require_contains(
            settings_profiles,
            command,
            "Settings profile Rust-owner command missing",
        )?;
    }
    for needle in [
        "fn update_reset_availability(",
        r#""settingsResetSelectedEndpoint""#,
        r#""settings_reset_selected_endpoint""#,
    ] {
        require_contains(
            settings_profiles,
            needle,
            "Settings endpoint-reset Rust-owner contract missing",
        )?;
    }
    for command in ["address_book_export_json", "address_book_import_json"] {
        require_contains(
            settings_addresses,
            command,
            "Settings address IO Rust-owner command missing",
        )?;
    }
    for needle in [
        r#"function(&dialog, "save")"#,
        r#"function(&dialog, "open")"#,
    ] {
        require_contains(
            settings_addresses,
            needle,
            "Settings address IO native dialog contract missing",
        )?;
    }
    for function in [
        "settings_profile_add",
        "settings_profile_rename",
        "settings_profile_delete",
        "settings_profile_select",
        "settings_reset_selected_endpoint",
    ] {
        if !backend.contains(&format!("pub fn {function}")) {
            return Err(format!("backend missing {function}"));
        }
    }
    require_contains(
        address_book,
        "import will not overwrite it",
        "address import must preserve conflicting saved data",
    )
}

fn aud010_contract(root: &Path) -> Result<(), String> {
    let source = normalize_newlines(&read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs",
    )?);
    validate_aud010(&source)
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}

fn validate_aud010(source: &str) -> Result<(), String> {
    const GUEST_PATH: &str = "../../../../e2e/node_modules/@wdio/tauri-plugin/dist-js/index.js";
    for (needle, message) in [
        (
            "#[cfg(feature = \"e2e-test\")]\n    let builder = {",
            "AUD-010 guest seam must remain e2e-test-only",
        ),
        (
            GUEST_PATH,
            "seam must embed the locked official WDIO guest bundle",
        ),
        (
            ".rfind(\"\\nexport {\")",
            "seam must strip only the terminal ESM export block",
        ),
        (
            "initPromise = init();",
            "guest bundle must explicitly initialize during Tauri invoke initialization",
        ),
        (
            "builder.append_invoke_initialization_script(&initialization_script)",
            "guest bundle must initialize before app scripts",
        ),
        (
            "const facade = Object.create(originalCore)",
            "seam must preserve the original Core through a facade",
        ),
        (
            "Object.defineProperty(facade, 'invoke'",
            "facade invoke must be independently configurable",
        ),
        (
            "window.__wdio_original_core__ = originalCore",
            "seam must preserve the exact original Core",
        ),
        (
            "tauri.core = facade",
            "e2e global Core must switch to the mutable facade",
        ),
        (
            "setupInvokeInterception();",
            "official guest interceptor must own the facade invoke path",
        ),
        (
            "window.__wdio_mutable_core_facade__ = true",
            "seam must expose deterministic readiness marker",
        ),
        ("attempt >= 100", "facade wait must be bounded"),
        (
            "installMutableCoreFacade(attempt + 1), 50",
            "facade wait cadence must remain deterministic",
        ),
        (
            "window.__wdio_mutable_core_facade_error__",
            "facade exhaustion must fail closed with a marker",
        ),
    ] {
        require_contains(source, needle, message)?;
    }

    forbid_contains(
        source,
        "__wdio_internal_invoke_fallback__",
        "ineffective immutable internal-invoke fallback must not remain",
    )?;

    if source
        .matches("@wdio/tauri-plugin/dist-js/index.js")
        .count()
        != 1
    {
        return Err("exactly one guest bundle owner is allowed".to_owned());
    }
    if source
        .matches("append_invoke_initialization_script")
        .count()
        != 1
    {
        return Err("exactly one AUD-010 initialization seam is allowed".to_owned());
    }
    if count_regex(source, r"on_page_load\(\|webview, _payload\|") != 0 {
        return Err("late page-load seam must not remain".to_owned());
    }
    Ok(())
}

fn settings_programmatic_restore_contract(root: &Path) -> Result<(), String> {
    let node = read(root, "crates/kaspa-gateway-frontend-wasm/src/node_tab.rs")?;
    let bridge = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
    )?;
    let node_owner = read(
        root,
        "crates/kaspa-gateway-frontend-wasm/src/node_frontend_helpers.rs",
    )?;
    validate_programmatic_restore(&node, &bridge, &node_owner)
}

fn function_block<'a>(source: &'a str, start: &str, next: &str) -> Result<&'a str, String> {
    let begin = source
        .find(start)
        .ok_or_else(|| format!("unable to isolate {start}"))?;
    let end = source[begin + start.len()..]
        .find(next)
        .map(|offset| begin + start.len() + offset)
        .ok_or_else(|| format!("unable to isolate {start}"))?;
    if end <= begin {
        return Err(format!("unable to isolate {start}"));
    }
    Ok(&source[begin..end])
}

fn validate_programmatic_restore(node: &str, bridge: &str, node_owner: &str) -> Result<(), String> {
    for (needle, message) in [
        (
            "fn restore_defaults(net: &str) -> JsValue",
            "Node Restore Defaults Rust owner is missing",
        ),
        (
            "node_r51_restore_defaults_action(net.to_owned())",
            "Node Restore Defaults must delegate restored-settings application to the Rust helper",
        ),
        (
            "node_apply_root_default_path(",
            "Node Restore Defaults must refresh backend-owned default paths",
        ),
        (
            "\"restore-defaults\" => {",
            "Node action dispatcher must retain Restore Defaults routing",
        ),
        (
            "let _ = restore_defaults(&net);",
            "Node Restore Defaults action must call the Rust restore owner",
        ),
    ] {
        require_contains(node, needle, message)?;
    }
    require_contains(
        node_owner,
        "let write_result = r51_write_settings(net, &defaults);",
        "Rust/WASM Node Restore Defaults owner must apply restored settings",
    )?;

    if !regex_is_match(
        bridge,
        r"function kgwBridgeSettingsWithProgrammaticWriteR9B\(callback\)\s*\{\s*return callback\(\);\s*\}",
    ) {
        return Err("Bridge programmatic restore call boundary is missing".to_owned());
    }
    let bridge_restore = function_block(
        bridge,
        "function kgwBridgeR51RestoreDefaults",
        "function kgwBridgeR51IsRunning",
    )?;
    for (needle, message) in [
        (
            "kgwBridgeSettingsWithProgrammaticWriteR9B(() =>",
            "Bridge Restore Defaults must use the programmatic boundary",
        ),
        (
            "kgwBridgeR51WriteSettings(net, defaults)",
            "Bridge Restore Defaults must write restored settings",
        ),
        (
            "kgwBridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true })",
            "Bridge Restore Defaults must refresh backend-owned default paths",
        ),
    ] {
        require_contains(bridge_restore, needle, message)?;
    }
    Ok(())
}

fn regex_is_match(source: &str, pattern: &str) -> bool {
    Regex::new(pattern)
        .expect("static contract regex must compile")
        .is_match(source)
}

pub fn run(root: &Path) -> Result<String, String> {
    analysis_contract(root)?;
    explorer_lint_contract(root)?;
    functional_ui_contract(root)?;
    settings_workflow_contract(root)?;
    aud010_contract(root)?;
    settings_programmatic_restore_contract(root)?;
    Ok(
        "KGW static contract regressions PASSED: analysis, explorer-lint, functional-ui, settings-workflow, AUD-010, programmatic-restore"
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn analysis_real_contract_passes() {
        assert!(analysis_contract(&root()).is_ok());
    }

    #[test]
    fn analysis_alias_drift_fails_closed() {
        let root = root();
        let binding = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js",
        )
        .unwrap();
        let rust_owner = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/analysis_binding.rs",
        )
        .unwrap()
        .replace(r#""30d" => "last_month""#, r#""30d" => "wrong""#);
        let settings_owner = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/settings_addresses.rs",
        )
        .unwrap();
        let backend = read(
            &root,
            "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs",
        )
        .unwrap();
        assert!(validate_analysis(&binding, &rust_owner, &settings_owner, &backend).is_err());
    }

    #[test]
    fn analysis_saved_address_owner_drift_fails_closed() {
        let root = root();
        let binding = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/analysis/analysis-rust-binding.js",
        )
        .unwrap();
        let rust_owner = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/analysis_binding.rs",
        )
        .unwrap();
        let settings_owner = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/settings_addresses.rs",
        )
        .unwrap()
        .replace("kgw:saved-addresses-changed", "kgw:saved-addresses-removed");
        let backend = read(
            &root,
            "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs",
        )
        .unwrap();
        assert!(validate_analysis(&binding, &rust_owner, &settings_owner, &backend).is_err());
    }

    #[test]
    fn explorer_lint_real_contract_passes() {
        assert!(explorer_lint_contract(&root()).is_ok());
    }

    #[test]
    fn explorer_lint_missing_formatter_fails_closed() {
        let root = root();
        let source = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
        )
        .unwrap()
        .replace("kgwSummaryFormatUsd(", "kgwRemovedSummaryFormatter(");
        assert!(validate_explorer_lint(&source).is_err());
    }

    #[test]
    fn functional_ui_real_contract_passes() {
        assert!(functional_ui_contract(&root()).is_ok());
    }

    #[test]
    fn functional_ui_missing_backend_validation_fails_closed() {
        let root = root();
        let explorer_js = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
        )
        .unwrap();
        let explorer_address_rust = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/explorer_addresses.rs",
        )
        .unwrap()
        .replace(r#""validate_kaspa_address""#, r#""wrong_validation""#);
        let explorer_filters_rust = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/explorer_filters.rs",
        )
        .unwrap();
        let explorer_css = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.css",
        )
        .unwrap();
        let settings_paths = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/settings_paths.rs",
        )
        .unwrap();
        let top_js = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js",
        )
        .unwrap();
        let top_rust = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/top_addresses.rs",
        )
        .unwrap();
        let top_html = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.html",
        )
        .unwrap();
        let top_template = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.template.js",
        )
        .unwrap();
        assert!(
            validate_functional_ui(FunctionalUiSources {
                explorer_js: &explorer_js,
                explorer_address_rust: &explorer_address_rust,
                explorer_filters_rust: &explorer_filters_rust,
                explorer_css: &explorer_css,
                settings_paths: &settings_paths,
                top_js: &top_js,
                top_rust: &top_rust,
                top_html: &top_html,
                top_template: &top_template,
            })
            .is_err()
        );
    }

    #[test]
    fn explorer_filter_owner_missing_export_fails_closed() {
        let root = root();
        let explorer_js = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
        )
        .unwrap();
        let explorer_filters_rust = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/explorer_filters.rs",
        )
        .unwrap()
        .replace(
            "#[wasm_bindgen(js_name = explorerFilterBuildRequest)]",
            "#[wasm_bindgen(js_name = explorerFilterBuildRequestRemoved)]",
        );
        assert!(validate_explorer_filter_ownership(&explorer_js, &explorer_filters_rust).is_err());
    }

    #[test]
    fn explorer_filter_owner_legacy_js_definition_fails_closed() {
        let root = root();
        let explorer_js = format!(
            "{}\nfunction kgwFilterBuildRequest() {{}}\n",
            read(
                &root,
                "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.js",
            )
            .unwrap()
        );
        let explorer_filters_rust = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/explorer_filters.rs",
        )
        .unwrap();
        assert!(validate_explorer_filter_ownership(&explorer_js, &explorer_filters_rust).is_err());
    }

    #[test]
    fn settings_workflow_real_contract_passes() {
        assert!(settings_workflow_contract(&root()).is_ok());
    }

    #[test]
    fn settings_workflow_missing_real_command_fails_closed() {
        let root = root();
        let settings_adapter = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
        )
        .unwrap();
        let settings_ui = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/settings_ui.rs",
        )
        .unwrap();
        let settings_profiles = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/settings_profiles.rs",
        )
        .unwrap()
        .replace("settings_profile_add", "profile_add_removed");
        let settings_addresses = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/settings_addresses.rs",
        )
        .unwrap();
        let backend = read(
            &root,
            "apps/kaspa-gateway-desktop/src-tauri/src/settings_commands.rs",
        )
        .unwrap();
        let address_book = read(
            &root,
            "apps/kaspa-gateway-desktop/src-tauri/src/address_book.rs",
        )
        .unwrap();
        assert!(
            validate_settings_workflow(
                &settings_adapter,
                &settings_ui,
                &settings_profiles,
                &settings_addresses,
                &backend,
                &address_book,
            )
            .is_err()
        );
    }

    #[test]
    fn aud010_real_contract_passes() {
        assert!(aud010_contract(&root()).is_ok());
    }

    #[test]
    fn aud010_readiness_marker_drift_fails_closed() {
        let root = root();
        let source = normalize_newlines(
            &read(&root, "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
                .unwrap()
                .replace(
                    "window.__wdio_mutable_core_facade__ = true",
                    "window.__wdio_mutable_core_facade__ = false",
                ),
        );
        assert!(validate_aud010(&source).is_err());
    }

    #[test]
    fn programmatic_restore_real_contract_passes() {
        assert!(settings_programmatic_restore_contract(&root()).is_ok());
    }

    #[test]
    fn programmatic_restore_missing_node_rust_write_fails_closed() {
        let root = root();
        let node = read(&root, "crates/kaspa-gateway-frontend-wasm/src/node_tab.rs").unwrap();
        let bridge = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        )
        .unwrap();
        let node_owner = read(
            &root,
            "crates/kaspa-gateway-frontend-wasm/src/node_frontend_helpers.rs",
        )
        .unwrap()
        .replace(
            "let write_result = r51_write_settings(net, &defaults);",
            "let write_result = r51_write_settings_removed(net, &defaults);",
        );
        assert!(validate_programmatic_restore(&node, &bridge, &node_owner).is_err());
    }
}
