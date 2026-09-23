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
    let settings = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
    )?;
    let backend = read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs",
    )?;
    validate_analysis(&binding, &settings, &backend)
}

fn validate_analysis(binding: &str, settings: &str, backend: &str) -> Result<(), String> {
    if ![
        "\"30d\": \"last_month\"",
        "\"90d\": \"last_3_months\"",
        "\"1y\": \"last_year\"",
    ]
    .iter()
    .all(|needle| binding.contains(needle))
    {
        return Err("frontend range aliases missing".to_owned());
    }
    if !binding.contains("kgw:saved-addresses-changed")
        || !settings.contains("kgwNotifySavedAddressesChanged")
    {
        return Err("saved-address invalidation contract missing".to_owned());
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
            "setInterval(kgwLiveDbPollingTick, 2000)",
            "Explorer live DB polling must schedule its local owner",
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
    let explorer_css = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.css",
    )?;
    let settings = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
    )?;
    let top_js = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js",
    )?;
    let top_html = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.html",
    )?;
    let top_template = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.template.js",
    )?;
    validate_functional_ui(
        &explorer_js,
        &explorer_css,
        &settings,
        &top_js,
        &top_html,
        &top_template,
    )
}
fn validate_functional_ui(
    explorer_js: &str,
    explorer_css: &str,
    settings: &str,
    top_js: &str,
    top_html: &str,
    top_template: &str,
) -> Result<(), String> {
    forbid_contains(
        explorer_css,
        "#explorer #explorerStatus {\n  display: none !important;",
        "Explorer status must not be permanently hidden",
    )?;
    if count_regex(
        explorer_js,
        r#"setStatus\((?:section|root), "Enter a valid Kaspa address\.", "error"\)"#,
    ) < 4
    {
        return Err("All invalid Explorer actions need visible error status".to_owned());
    }
    require_contains(
        explorer_js,
        r#"node.setAttribute("aria-live", state === "error" ? "assertive" : "polite")"#,
        "Explorer ARIA-live state missing",
    )?;
    require_contains(
        explorer_js,
        r#"invokeCommand("validate_kaspa_address""#,
        "Explorer must use canonical backend address validation",
    )?;
    if count_regex(explorer_js, r"await kgwCanonicalKaspaAddress\(address\)") < 4 {
        return Err(
            "All active Explorer filter/fetch owners must use canonical validation".to_owned(),
        );
    }
    require_contains(
        settings,
        r#"dialog.open({ title: "Choose directory", directory: true, multiple: false })"#,
        "Settings Browse must invoke native directory dialog",
    )?;
    if !settings.contains("settings_validate_custom_path")
        || !settings.contains("Browse cancelled; path unchanged.")
    {
        return Err("Settings Browse selected/cancel contract missing".to_owned());
    }
    if !top_html.contains(r#"id="topAddressesStatus""#)
        || !top_template.contains("topAddressesStatus")
    {
        return Err("Top Addresses status target missing from runtime template".to_owned());
    }
    for state in ["LOADING —", "EMPTY —", "ERROR —", "SUCCESS —"] {
        if !top_js.contains(state) {
            return Err(format!("Top Addresses {state} state missing"));
        }
    }
    Ok(())
}
fn settings_workflow_contract(root: &Path) -> Result<(), String> {
    let settings = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
    )?;
    let backend = read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/settings_commands.rs",
    )?;
    let address_book = read(
        root,
        "apps/kaspa-gateway-desktop/src-tauri/src/address_book.rs",
    )?;
    validate_settings_workflow(&settings, &backend, &address_book)
}

fn validate_settings_workflow(
    settings: &str,
    backend: &str,
    address_book: &str,
) -> Result<(), String> {
    let placeholder = isolate_optional(settings, "const placeholderActions = [", "];");
    for id in [
        "settingsProfileAdd",
        "settingsProfileRename",
        "settingsProfileDelete",
        "settingsResetSelectedEndpoint",
        "settingsExportAddresses",
        "settingsImportAddresses",
    ] {
        if placeholder.contains(id) {
            return Err(format!("{id} must not remain placeholder-owned"));
        }
    }
    for command in [
        "settings_profile_add",
        "settings_profile_rename",
        "settings_profile_delete",
        "settings_profile_select",
        "settings_reset_selected_endpoint",
        "address_book_export_json",
        "address_book_import_json",
    ] {
        if !settings.contains(command) {
            return Err(format!("frontend missing real command {command}"));
        }
    }
    if !settings.contains("dialog.save") || !settings.contains("dialog.open") {
        return Err("native dialog open/save flows must be present".to_owned());
    }
    require_contains(
        settings,
        "kgwSettingsUpdateEndpointResetAvailability",
        "reset availability must be explicit",
    )?;
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

fn isolate_optional<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let Some(begin) = source.find(start) else {
        return "";
    };
    let tail = &source[begin + start.len()..];
    let Some(length) = tail.find(end) else {
        return "";
    };
    &source[begin..begin + start.len() + length]
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
    let node = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
    )?;
    let bridge = read(
        root,
        "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
    )?;
    validate_programmatic_restore(&node, &bridge)
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

fn validate_programmatic_restore(node: &str, bridge: &str) -> Result<(), String> {
    if !regex_is_match(
        node,
        r"function kgwNodeSettingsWithProgrammaticWriteR9B\(callback\)\s*\{\s*return callback\(\);\s*\}",
    ) {
        return Err("Node programmatic restore call boundary is missing".to_owned());
    }
    let node_restore = function_block(
        node,
        "function kgwNodeR51RestoreDefaults",
        "function kgwNodeR51IsRunning",
    )?;
    for (needle, message) in [
        (
            "kgwNodeSettingsWithProgrammaticWriteR9B(() =>",
            "Node Restore Defaults must use the programmatic boundary",
        ),
        (
            "kgwNodeR51WriteSettings(net, defaults)",
            "Node Restore Defaults must write restored settings",
        ),
        (
            "kgwNodeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, { force: true })",
            "Node Restore Defaults must refresh backend-owned default paths",
        ),
    ] {
        require_contains(node_restore, needle, message)?;
    }

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
        .unwrap()
        .replace(r#""30d": "last_month""#, r#""30d": "wrong""#);
        let settings = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
        )
        .unwrap();
        let backend = read(
            &root,
            "apps/kaspa-gateway-desktop/src-tauri/src/analysis_commands.rs",
        )
        .unwrap();
        assert!(validate_analysis(&binding, &settings, &backend).is_err());
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
        .unwrap()
        .replace(
            r#"invokeCommand("validate_kaspa_address""#,
            r#"invokeCommand("wrong_validation""#,
        );
        let explorer_css = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/explorer/explorer.css",
        )
        .unwrap();
        let settings = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
        )
        .unwrap();
        let top_js = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/top-addresses/top-addresses.js",
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
            validate_functional_ui(
                &explorer_js,
                &explorer_css,
                &settings,
                &top_js,
                &top_html,
                &top_template
            )
            .is_err()
        );
    }

    #[test]
    fn settings_workflow_real_contract_passes() {
        assert!(settings_workflow_contract(&root()).is_ok());
    }

    #[test]
    fn settings_workflow_missing_real_command_fails_closed() {
        let root = root();
        let settings = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/settings/settings.js",
        )
        .unwrap()
        .replace("settings_profile_add", "profile_add_removed");
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
        assert!(validate_settings_workflow(&settings, &backend, &address_book).is_err());
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
    fn programmatic_restore_missing_node_write_fails_closed() {
        let root = root();
        let node = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js",
        )
        .unwrap()
        .replace(
            "kgwNodeR51WriteSettings(net, defaults)",
            "kgwNodeR51WriteSettingsRemoved(net, defaults)",
        );
        let bridge = read(
            &root,
            "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js",
        )
        .unwrap();
        assert!(validate_programmatic_restore(&node, &bridge).is_err());
    }
}
