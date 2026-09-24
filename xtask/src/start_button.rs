use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const TEMPLATE_TICK: char = '\u{0060}';

pub fn run(root: &Path) -> Result<String, String> {
    let node_js = fs::read_to_string(root.join(NODE_JS))
        .map_err(|_| format!("Missing required file: {NODE_JS}"))?;
    let mut failures = static_failures(&node_js);

    run_local_command(
        root,
        "frontend regression syntax",
        "node",
        &["--check", "tools/kgw_start_button_frontend_tests.cjs"],
        &mut failures,
    );
    run_local_command(
        root,
        "frontend start button regression tests",
        "node",
        &["tools/kgw_start_button_frontend_tests.cjs"],
        &mut failures,
    );
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

    let start_id_before = Regex::new(r#"<button[^>]+id=[^>]+data-node-action="start""#).unwrap();
    let start_id_after = Regex::new(r#"<button[^>]+data-node-action="start"[^>]+id="#).unwrap();
    if start_id_before.is_match(node_js) || start_id_after.is_match(node_js) {
        failures.push("Start control uses an ID that can duplicate across networks.".to_owned());
    }

    let stop_id_before = Regex::new(r#"<button[^>]+id=[^>]+data-node-action="stop""#).unwrap();
    let stop_id_after = Regex::new(r#"<button[^>]+data-node-action="stop"[^>]+id="#).unwrap();
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
}
