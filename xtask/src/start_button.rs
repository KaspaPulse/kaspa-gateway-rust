use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const NODE_RENDER_RUST: &str = "crates/kaspa-gateway-frontend-wasm/src/node_frontend_helpers.rs";
const NODE_TAB_RUST: &str = "crates/kaspa-gateway-frontend-wasm/src/node_tab.rs";
#[cfg(test)]
const TEMPLATE_TICK: char = '\u{0060}';

pub fn run(root: &Path) -> Result<String, String> {
    let node_js = fs::read_to_string(root.join(NODE_JS))
        .map_err(|_| format!("Missing required file: {NODE_JS}"))?;
    let node_render_rust = fs::read_to_string(root.join(NODE_RENDER_RUST))
        .map_err(|_| format!("Missing required file: {NODE_RENDER_RUST}"))?;
    let node_tab_rust = fs::read_to_string(root.join(NODE_TAB_RUST))
        .map_err(|_| format!("Missing required file: {NODE_TAB_RUST}"))?;
    let mut failures = production_static_failures(&node_js, &node_render_rust, &node_tab_rust);

    println!("Running: Rust-owned frontend start button regression bridge");
    match crate::start_button_frontend::run(root) {
        Ok(message) => {
            if !message.is_empty() {
                println!("{message}");
            }
        }
        Err(error) => failures.push(error),
    }
    run_local_command(
        root,
        "targeted Tauri IPC regression tests",
        "cargo",
        &[
            "test",
            "--manifest-path",
            "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml",
            "--test",
            "integrated_runtime_ipc_smoke_tests",
        ],
        &mut failures,
    );

    if failures.is_empty() {
        Ok("KGW start button gate PASSED".to_owned())
    } else {
        let mut message = String::from("KGW start button gate FAILED");
        for failure in failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        Err(message)
    }
}

fn run_local_command(
    root: &Path,
    label: &str,
    command: &str,
    args: &[&str],
    failures: &mut Vec<String>,
) {
    println!("Running: {label}");
    match Command::new(command).args(args).current_dir(root).status() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            failures.push(format!("{label} failed with exit code {code}"));
        }
        Err(error) => failures.push(format!("{label} failed to start: {error}")),
    }
}

#[cfg(test)]
fn static_failures(node_js: &str) -> Vec<String> {
    let mut failures = Vec::new();

    let render_start = node_js.find("function renderNetworkPanel");
    let render = render_start.map(|index| &node_js[index..]).unwrap_or("");
    let settings_index = render.find(r#"data-node-inner-panel="settings""#);
    let log_index = render.find(r#"data-node-inner-panel="log""#);
    let end_marker = format!("</div>{TEMPLATE_TICK};");
    let render_end = log_index.and_then(|index| {
        render[index..]
            .find(&end_marker)
            .map(|offset| index + offset)
    });

    match (render_start, settings_index, log_index, render_end) {
        (Some(_), Some(settings), Some(log), Some(end)) if settings < log && log <= end => {
            let settings_block = &render[settings..log];
            let log_block = &render[log..end];

            if !settings_block.contains(r#"data-node-action="start""#) {
                failures.push("Settings panel is missing Start.".to_owned());
            }
            if !settings_block.contains(r#"data-node-action="stop""#) {
                failures.push("Settings panel is missing Stop.".to_owned());
            }
            if !settings_block.contains("data-node-network-enabled") {
                failures.push("Settings panel is missing network enable control.".to_owned());
            }
            if log_block.contains(r#"data-node-action="start""#) {
                failures.push("Live Node Monitor contains Start.".to_owned());
            }
            if log_block.contains(r#"data-node-action="stop""#) {
                failures.push("Live Node Monitor contains Stop.".to_owned());
            }
            if log_block.contains("data-node-network-enabled") {
                failures.push("Live Node Monitor contains network enable control.".to_owned());
            }
        }
        _ => failures
            .push("Could not verify Settings and Live Node Monitor panel ordering.".to_owned()),
    }

    let start_count = Regex::new(r#"<button[^>]+data-node-action="start""#)
        .unwrap()
        .find_iter(node_js)
        .count();
    let stop_count = Regex::new(r#"<button[^>]+data-node-action="stop""#)
        .unwrap()
        .find_iter(node_js)
        .count();
    if start_count != 1 {
        failures.push(format!(
            "Expected exactly one Start control template, found {start_count}."
        ));
    }
    if stop_count != 1 {
        failures.push(format!(
            "Expected exactly one Stop control template, found {stop_count}."
        ));
    }

    // Match the actual id attribute, not data-testid/custom-id suffixes.
    let start_id_before =
        Regex::new(r#"<button[^>]*\sid\s*=[^>]+data-node-action="start""#).unwrap();
    let start_id_after =
        Regex::new(r#"<button[^>]+data-node-action="start"[^>]*\sid\s*="#).unwrap();
    if start_id_before.is_match(node_js) || start_id_after.is_match(node_js) {
        failures.push("Start control uses an ID that can duplicate across networks.".to_owned());
    }

    let stop_id_before = Regex::new(r#"<button[^>]*\sid\s*=[^>]+data-node-action="stop""#).unwrap();
    let stop_id_after = Regex::new(r#"<button[^>]+data-node-action="stop"[^>]*\sid\s*="#).unwrap();
    if stop_id_before.is_match(node_js) || stop_id_after.is_match(node_js) {
        failures.push("Stop control uses an ID that can duplicate across networks.".to_owned());
    }

    for pattern in [
        r"appendLog\([^)]*initialized",
        r"appendLog\([^)]*start response",
        r"appendLog\([^)]*started successfully",
        r"appendLog\([^)]*synchronized",
        r"appendLog\([^)]*connected",
        r"appendLog\([^)]*Node settings saved successfully",
        r"appendLog\([^)]*Node defaults restored successfully",
    ] {
        let regex = Regex::new(pattern).expect("static start-button regex must compile");
        if regex.is_match(node_js) {
            failures.push(format!(
                "Synthetic raw-log message detected by pattern: {pattern}"
            ));
        }
    }

    failures
}

fn production_static_failures(
    node_js: &str,
    node_render_rust: &str,
    node_tab_rust: &str,
) -> Vec<String> {
    let mut failures = Vec::new();

    let adapter_enters_rust_owner = node_js.contains("nodeInitKaspaNodeTab")
        && node_js.contains("return nodeInitKaspaNodeTab(root);");
    let rust_tab_owns_panel_install = node_tab_rust
        .contains("fn render_all_networks(root: &JsValue) -> bool")
        && node_tab_rust
            .contains("crate::node_frontend_helpers::node_render_network_panels_html()")
        && node_tab_rust.contains(r#"set(&host, "innerHTML", &JsValue::from_str(&html));"#)
        && node_tab_rust.contains("render_all_networks(&node_root);");

    if !adapter_enters_rust_owner || !rust_tab_owns_panel_install {
        failures.push(
            "Node network panels must be rendered through the Rust/WASM ownership seam.".to_owned(),
        );
    }

    for forbidden in [
        "function cardInput(",
        "function cardSelect(",
        "function cardCheck(",
        "function renderRuntime(",
        "function renderNetwork(",
        "function renderRpc(",
        "function renderPeers(",
        "function renderDatabase(",
        "function renderRocksDb(",
        "function renderPaths(",
        "function renderSections(",
        "function renderNetworkPanel(",
    ] {
        if node_js.contains(forbidden) {
            failures.push(format!(
                "Hand-maintained Node rendering owner remains in JavaScript: {forbidden}"
            ));
        }
    }

    let render_start = node_render_rust.find("fn render_node_network_panel(");
    let render_end = render_start.and_then(|start| {
        node_render_rust[start..]
            .find("#[wasm_bindgen(js_name = nodeRenderNetworkPanelsHtml)]")
            .map(|offset| start + offset)
    });

    if let (Some(start), Some(end)) = (render_start, render_end) {
        let render = &node_render_rust[start..end];
        let settings_index = render.find(r#"data-node-inner-panel="settings""#);
        let log_index = render.find(r#"data-node-inner-panel="log""#);
        match (settings_index, log_index) {
            (Some(settings), Some(log)) if settings < log => {
                let settings_block = &render[settings..log];
                let log_block = &render[log..];
                if !settings_block.contains(r#"data-node-action="start""#) {
                    failures.push("Settings panel is missing Start.".to_owned());
                }
                if !settings_block.contains(r#"data-node-action="stop""#) {
                    failures.push("Settings panel is missing Stop.".to_owned());
                }
                if !settings_block.contains("data-node-network-enabled") {
                    failures.push("Settings panel is missing network enable control.".to_owned());
                }
                if log_block.contains(r#"data-node-action="start""#) {
                    failures.push("Live Node Monitor contains Start.".to_owned());
                }
                if log_block.contains(r#"data-node-action="stop""#) {
                    failures.push("Live Node Monitor contains Stop.".to_owned());
                }
                if log_block.contains("data-node-network-enabled") {
                    failures.push("Live Node Monitor contains network enable control.".to_owned());
                }
            }
            _ => failures.push(
                "Could not verify Rust-owned Settings and Live Node Monitor panel ordering."
                    .to_owned(),
            ),
        }

        let start_count = Regex::new(r#"<button[^>]+data-node-action="start""#)
            .unwrap()
            .find_iter(render)
            .count();
        let stop_count = Regex::new(r#"<button[^>]+data-node-action="stop""#)
            .unwrap()
            .find_iter(render)
            .count();
        if start_count != 1 {
            failures.push(format!(
                "Expected exactly one Rust-owned Start control template, found {start_count}."
            ));
        }
        if stop_count != 1 {
            failures.push(format!(
                "Expected exactly one Rust-owned Stop control template, found {stop_count}."
            ));
        }

        let start_id_before =
            Regex::new(r#"<button[^>]*\sid\s*=[^>]+data-node-action="start""#).unwrap();
        let start_id_after =
            Regex::new(r#"<button[^>]+data-node-action="start"[^>]*\sid\s*="#).unwrap();
        if start_id_before.is_match(render) || start_id_after.is_match(render) {
            failures
                .push("Start control uses an ID that can duplicate across networks.".to_owned());
        }
        let stop_id_before =
            Regex::new(r#"<button[^>]*\sid\s*=[^>]+data-node-action="stop""#).unwrap();
        let stop_id_after =
            Regex::new(r#"<button[^>]+data-node-action="stop"[^>]*\sid\s*="#).unwrap();
        if stop_id_before.is_match(render) || stop_id_after.is_match(render) {
            failures.push("Stop control uses an ID that can duplicate across networks.".to_owned());
        }
    } else {
        failures.push("Could not locate authoritative Rust Node panel renderer.".to_owned());
    }

    for pattern in [
        r"appendLog\([^)]*initialized",
        r"appendLog\([^)]*start response",
        r"appendLog\([^)]*started successfully",
        r"appendLog\([^)]*synchronized",
        r"appendLog\([^)]*connected",
        r"appendLog\([^)]*Node settings saved successfully",
        r"appendLog\([^)]*Node defaults restored successfully",
    ] {
        let regex = Regex::new(pattern).expect("production start-button regex must compile");
        if regex.is_match(node_js) {
            failures.push(format!(
                "Synthetic raw-log message detected by pattern: {pattern}"
            ));
        }
    }

    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_fixture() -> String {
        let tick = TEMPLATE_TICK;
        format!(
            "function renderNetworkPanel() {{\n  return {tick}<div data-node-inner-panel=\"settings\">\n    <button data-node-action=\"start\">Start</button>\n    <button data-node-action=\"stop\">Stop</button>\n    <input data-node-network-enabled>\n  </div>\n  <div data-node-inner-panel=\"log\">\n    <pre>raw logs</pre>\n  </div>{tick};\n}}\n"
        )
    }

    #[test]
    fn valid_layout_passes_static_contract() {
        assert!(static_failures(&valid_fixture()).is_empty());
    }

    #[test]
    fn duplicate_start_id_fails_closed() {
        let source = valid_fixture().replace(
            r#"data-node-action="start""#,
            r#"id="start-mainnet" data-node-action="start""#,
        );
        assert!(
            static_failures(&source)
                .iter()
                .any(|item| item.contains("Start control uses an ID"))
        );
    }

    #[test]
    fn duplicate_stop_id_fails_closed() {
        let source = valid_fixture().replace(
            r#"data-node-action="stop""#,
            r#"data-node-action="stop" id="stop-mainnet""#,
        );
        assert!(
            static_failures(&source)
                .iter()
                .any(|item| item.contains("Stop control uses an ID"))
        );
    }

    #[test]
    fn control_in_log_panel_fails_closed() {
        let source = valid_fixture().replace(
            "<pre>raw logs</pre>",
            r#"<button data-node-action="start">bad</button><pre>raw logs</pre>"#,
        );
        let failures = static_failures(&source);
        assert!(
            failures
                .iter()
                .any(|item| item == "Live Node Monitor contains Start.")
        );
        assert!(
            failures
                .iter()
                .any(|item| item.contains("Expected exactly one Start control template"))
        );
    }

    #[test]
    fn synthetic_log_message_fails_closed() {
        let source = format!(
            "{}\nfunction bad() {{ appendLog('Node defaults restored successfully'); }}",
            valid_fixture()
        );
        assert!(
            static_failures(&source)
                .iter()
                .any(|item| item.contains("Synthetic raw-log message"))
        );
    }

    #[test]
    fn missing_panel_order_fails_closed() {
        let source = format!(
            "function renderNetworkPanel() {{ return {TEMPLATE_TICK}<div></div>{TEMPLATE_TICK}; }}"
        );
        assert!(
            static_failures(&source)
                .iter()
                .any(|item| item.contains("Could not verify"))
        );
    }
    #[test]
    fn id_attribute_boundary_test_and_custom_names_are_not_dom_ids() {
        for action in ["start", "stop"] {
            for name in ["data-testid", "custom-id", "invalidid", "data-id"] {
                for before in [true, false] {
                    let control = format!("data-node-action=\"{action}\"");
                    let attribute = format!("{name}=\"{action}-test\"");
                    let replacement = if before {
                        format!("{attribute} {control}")
                    } else {
                        format!("{control} {attribute}")
                    };
                    let source = valid_fixture().replace(&control, &replacement);
                    let failures = static_failures(&source);
                    assert!(
                        failures.is_empty(),
                        "{action}/{name}/{before}: {failures:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn id_attribute_boundary_real_ids_with_spacing_still_fail_closed() {
        for action in ["start", "stop"] {
            for attribute in ["id=\"fixed\"", "id = \"fixed\"", "id\t=\t'fixed'"] {
                for before in [true, false] {
                    let control = format!("data-node-action=\"{action}\"");
                    let replacement = if before {
                        format!("{attribute} {control}")
                    } else {
                        format!("{control} {attribute}")
                    };
                    let source = valid_fixture().replace(&control, &replacement);
                    let failures = static_failures(&source);
                    assert!(
                        failures
                            .iter()
                            .any(|item| item.contains("control uses an ID")),
                        "{action}/{attribute}/{before}: {failures:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn id_attribute_boundary_current_node_markup_has_no_static_violation() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let source = fs::read_to_string(root.join(NODE_JS)).unwrap();
        let renderer = fs::read_to_string(root.join(NODE_RENDER_RUST)).unwrap();
        let node_tab = fs::read_to_string(root.join(NODE_TAB_RUST)).unwrap();
        let failures = production_static_failures(&source, &renderer, &node_tab);
        assert!(
            failures.is_empty(),
            "current Node/Rust source: {failures:?}"
        );
    }
}
