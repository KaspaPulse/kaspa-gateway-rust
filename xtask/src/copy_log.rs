use std::fs;
use std::path::Path;
use std::process::Command;

const NODE_JS_REL: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const FRONTEND_TEST_REL: &str = "xtask/src/start_button_frontend.rs";
const LIB_RS_REL: &str = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
const CARGO_TOML_REL: &str = "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml";

#[derive(Debug, Clone)]
struct Inputs {
    node: String,
    frontend_test: String,
    lib_rs: String,
    cargo_toml: String,
}

pub fn run(root: &Path) -> Result<String, String> {
    let inputs = Inputs {
        node: read_required(root, NODE_JS_REL)?,
        frontend_test: read_required(root, FRONTEND_TEST_REL)?,
        lib_rs: read_required(root, LIB_RS_REL)?,
        cargo_toml: read_required(root, CARGO_TOML_REL)?,
    };

    let mut failures = validate(&inputs);
    println!("Running: Rust-owned Copy Log frontend regression bridge");
    match crate::start_button_frontend::run(root) {
        Ok(message) => {
            if !message.is_empty() {
                println!("{message}");
            }
        }
        Err(error) => failures.push(error),
    }
    run_command(
        root,
        "targeted Tauri clipboard tests",
        "cargo",
        &["test", "-p", "kaspa-gateway-desktop", "kgw_clipboard"],
        &mut failures,
    );

    if failures.is_empty() {
        Ok("KGW Copy Log gate PASSED".to_owned())
    } else {
        let mut message = String::from("KGW Copy Log gate FAILED");
        for failure in failures {
            message.push_str("\n- ");
            message.push_str(&failure);
        }
        Err(message)
    }
}

fn read_required(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    fs::read_to_string(&path)
        .map_err(|error| format!("Missing required file: {} ({error})", path.display()))
}

fn run_command(root: &Path, label: &str, program: &str, args: &[&str], failures: &mut Vec<String>) {
    println!("Running: {label}");
    match Command::new(program).args(args).current_dir(root).status() {
        Ok(status) if status.success() => {}
        Ok(status) => failures.push(format!(
            "{label} failed with exit code {}",
            status.code().unwrap_or(1)
        )),
        Err(error) => failures.push(format!("{label} failed to start: {error}")),
    }
}

fn extract_between(text: &str, start: &str, end: &str) -> String {
    let Some(start_index) = text.find(start) else {
        return String::new();
    };
    let tail = &text[start_index + start.len()..];
    let Some(end_relative) = tail.find(end) else {
        return String::new();
    };
    text[start_index..start_index + start.len() + end_relative].to_owned()
}

fn count(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

fn assert_contains(failures: &mut Vec<String>, text: &str, needle: &str, message: &str) {
    if !text.contains(needle) {
        failures.push(format!("{message} Missing: {needle}"));
    }
}

fn assert_not_contains(failures: &mut Vec<String>, text: &str, needle: &str, message: &str) {
    if text.contains(needle) {
        failures.push(format!("{message} Forbidden: {needle}"));
    }
}
fn validate(inputs: &Inputs) -> Vec<String> {
    let mut failures = Vec::new();
    let node = &inputs.node;

    let render_start = node.find("function renderNetworkPanel");
    if let Some(render_start) = render_start {
        let render = &node[render_start..];
        let copy_index = render.find(r#"data-node-action="copy-log""#);
        let clear_index = render.find(r#"data-node-action="clear-log""#);
        let settings_panel = extract_between(
            render,
            r#"data-node-inner-panel="settings""#,
            r#"data-node-inner-panel="log""#,
        );
        assert_not_contains(
            &mut failures,
            &settings_panel,
            r#"data-node-action="copy-log""#,
            "Copy Log must not exist in Settings.",
        );

        let log_panel_index =
            copy_index.and_then(|copy| render[..=copy].rfind(r#"data-node-inner-panel="log""#));
        let settings_panel_index = copy_index
            .and_then(|copy| render[..=copy].rfind(r#"data-node-inner-panel="settings""#));
        if copy_index.is_none()
            || log_panel_index.is_none()
            || matches!((settings_panel_index, log_panel_index), (Some(settings), Some(log)) if settings > log)
        {
            failures.push("Copy Log must exist in Live Node Monitor.".to_owned());
        }
        if clear_index.is_none()
            || log_panel_index.is_none()
            || matches!((clear_index, log_panel_index), (Some(clear), Some(log)) if clear < log)
        {
            failures.push("Clear Log must exist in Live Node Monitor.".to_owned());
        }
    } else {
        failures.push("Could not find renderNetworkPanel.".to_owned());
    }

    let markup_count = count(node, r#"data-node-action="copy-log""#);
    if markup_count != 1 {
        failures.push(format!(
            "Copy Log markup must appear exactly once in the node template; found {markup_count}."
        ));
    }

    let handler_count = count(node, r#"action === "copy-log" || action === "clear-log""#);
    if handler_count != 1 {
        failures.push(format!(
            "The delegated Copy Log handler must be attached exactly once; found {handler_count}."
        ));
    }

    let handler = node
        .find("async function kgwNodeHandleLogActionV29")
        .map(|index| &node[index..])
        .unwrap_or_else(|| {
            failures.push("The node log action handler was not found.".to_owned());
            ""
        });
    let copy_block = extract_between(
        handler,
        r#"if (action === "copy-log") {"#,
        r#"if (action === "clear-log") {"#,
    );

    for (needle, message) in [
        (
            "kgwNodeTraceActiveNetworkR1",
            "Copy Log must resolve the active network.",
        ),
        (
            "kgwNodeReadClipboardRawLogBufferV1(copyNetwork)",
            "Copy Log must read the selected active-network raw buffer.",
        ),
        (
            "buffer.isPlaceholder",
            "Copy Log must reject placeholder log text.",
        ),
        (
            "non-empty raw log buffer",
            "Copy Log must reject empty logs with an explicit error.",
        ),
        (
            "await kgwNodeDispatchClipboardWriteV1",
            "Copy Log must await native clipboard success.",
        ),
        (
            "kgwNodeCopyLogFailureV1",
            "Copy Log native failure must remain an error.",
        ),
        (
            "frontend.copy_log_succeeded",
            "Copy Log success trace must exist.",
        ),
    ] {
        assert_contains(&mut failures, &copy_block, needle, message);
    }
    for (needle, message) in [
        (
            "navigator.clipboard.writeText",
            "Copy Log must not use browser clipboard as the primary path.",
        ),
        (
            "document.execCommand",
            "Copy Log must not use execCommand as the primary path.",
        ),
    ] {
        assert_not_contains(&mut failures, &copy_block, needle, message);
    }

    let await_index = copy_block.find("await kgwNodeDispatchClipboardWriteV1");
    let feedback_index = copy_block.find(r#"kgwNodeTranslateRuntimeV29("log.copied""#);
    if await_index.is_none()
        || feedback_index.is_none()
        || matches!((await_index, feedback_index), (Some(awaited), Some(feedback)) if feedback < awaited)
    {
        failures.push(
            "Copied feedback must be displayed only after the awaited native clipboard write."
                .to_owned(),
        );
    }
    for (needle, message) in [
        (
            "frontend.copy_log_click_observed",
            "Copy Log physical click trace stage is required.",
        ),
        (
            "frontend.copy_log_network_resolved",
            "Copy Log network trace stage is required.",
        ),
        (
            "frontend.copy_log_content_prepared",
            "Copy Log content trace stage is required.",
        ),
        (
            "frontend.copy_log_dispatched",
            "Copy Log dispatch trace stage is required.",
        ),
        (
            "frontend.copy_log_failed",
            "Copy Log failure trace stage is required.",
        ),
        (
            "kgw_copy_text_to_clipboard_v1",
            "Copy Log must call the project-owned native clipboard command.",
        ),
        (
            "kgwNodeNormalizeClipboardLineEndingsV1",
            "Copy Log must normalize line endings safely.",
        ),
        (
            "kgwNodeSha256HexV1",
            "Copy Log should include SHA-256 metadata when practical.",
        ),
    ] {
        assert_contains(&mut failures, node, needle, message);
    }
    assert_not_contains(
        &mut failures,
        node,
        "completeClipboardContent",
        "Trace metadata must not expose clipboard content.",
    );

    for (needle, message) in [
        (
            "Copy Log must invoke native clipboard exactly once",
            "Frontend tests must prove exactly one clipboard invoke.",
        ),
        (
            "Copy Log must pass testnet10 when testnet10 is active",
            "Frontend tests must prove active-network isolation.",
        ),
        (
            "Copy Log must not mix mainnet into testnet10",
            "Frontend tests must prevent network mixing.",
        ),
        (
            "Large Copy Log text must not be silently truncated or reordered",
            "Frontend tests must cover large raw logs.",
        ),
        (
            "Copy Log trace must exclude raw clipboard content",
            "Frontend tests must verify trace redaction.",
        ),
    ] {
        assert_contains(&mut failures, &inputs.frontend_test, needle, message);
    }

    for (needle, message) in [
        (
            "kgw_copy_text_to_clipboard_v1",
            "Native clipboard command must be registered.",
        ),
        (
            "native.clipboard_write_entered",
            "Native clipboard entry trace is required.",
        ),
        (
            "native.clipboard_write_succeeded",
            "Native clipboard success trace is required.",
        ),
        (
            "native.clipboard_write_failed",
            "Native clipboard failure trace is required.",
        ),
        (
            "tauri_plugin_clipboard_manager::ClipboardExt",
            "Native clipboard must use the official Tauri clipboard-manager plugin.",
        ),
    ] {
        assert_contains(&mut failures, &inputs.lib_rs, needle, message);
    }
    assert_contains(
        &mut failures,
        &inputs.cargo_toml,
        "tauri-plugin-clipboard-manager",
        "Desktop Cargo.toml must depend on the official clipboard-manager plugin.",
    );

    failures
}
#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Inputs {
        let node = r#"
function renderNetworkPanel() {
  return `
    <div data-node-inner-panel="settings"></div>
    <div data-node-inner-panel="log">
      <button data-node-action="copy-log">Copy</button>
      <button data-node-action="clear-log">Clear</button>
    </div>`;
}
function install() {
  if (action === "copy-log" || action === "clear-log") {}
}
async function kgwNodeHandleLogActionV29() {
  frontend.copy_log_click_observed;
  frontend.copy_log_network_resolved;
  frontend.copy_log_content_prepared;
  frontend.copy_log_dispatched;
  frontend.copy_log_failed;
  kgw_copy_text_to_clipboard_v1;
  kgwNodeNormalizeClipboardLineEndingsV1;
  kgwNodeSha256HexV1;
  if (action === "copy-log") {
    kgwNodeTraceActiveNetworkR1();
    kgwNodeReadClipboardRawLogBufferV1(copyNetwork);
    buffer.isPlaceholder;
    "non-empty raw log buffer";
    await kgwNodeDispatchClipboardWriteV1();
    kgwNodeCopyLogFailureV1();
    frontend.copy_log_succeeded;
    kgwNodeTranslateRuntimeV29("log.copied");
  }
  if (action === "clear-log") {}
}
"#;
        Inputs {
            node: node.to_owned(),
            frontend_test: [
                "Copy Log must invoke native clipboard exactly once",
                "Copy Log must pass testnet10 when testnet10 is active",
                "Copy Log must not mix mainnet into testnet10",
                "Large Copy Log text must not be silently truncated or reordered",
                "Copy Log trace must exclude raw clipboard content",
            ]
            .join("\n"),
            lib_rs: [
                "kgw_copy_text_to_clipboard_v1",
                "native.clipboard_write_entered",
                "native.clipboard_write_succeeded",
                "native.clipboard_write_failed",
                "tauri_plugin_clipboard_manager::ClipboardExt",
            ]
            .join("\n"),
            cargo_toml: "tauri-plugin-clipboard-manager".to_owned(),
        }
    }

    #[test]
    fn complete_static_contract_passes() {
        assert!(validate(&fixture()).is_empty());
    }

    #[test]
    fn copy_log_in_settings_fails_closed() {
        let mut inputs = fixture();
        inputs.node = inputs.node.replace(
            r#"<div data-node-inner-panel="settings"></div>"#,
            r#"<div data-node-inner-panel="settings"><button data-node-action="copy-log">Copy</button></div>"#,
        );
        assert!(
            validate(&inputs)
                .iter()
                .any(|item| item.contains("must not exist in Settings"))
        );
    }

    #[test]
    fn browser_clipboard_path_fails_closed() {
        let mut inputs = fixture();
        inputs.node = inputs.node.replace(
            "await kgwNodeDispatchClipboardWriteV1();",
            "navigator.clipboard.writeText(raw); await kgwNodeDispatchClipboardWriteV1();",
        );
        assert!(
            validate(&inputs)
                .iter()
                .any(|item| item.contains("browser clipboard"))
        );
    }

    #[test]
    fn success_feedback_before_await_fails_closed() {
        let mut inputs = fixture();
        inputs.node = inputs.node.replace(
            r#"await kgwNodeDispatchClipboardWriteV1();
    kgwNodeCopyLogFailureV1();
    frontend.copy_log_succeeded;
    kgwNodeTranslateRuntimeV29("log.copied");"#,
            r#"kgwNodeTranslateRuntimeV29("log.copied");
    await kgwNodeDispatchClipboardWriteV1();
    kgwNodeCopyLogFailureV1();
    frontend.copy_log_succeeded;"#,
        );
        assert!(
            validate(&inputs)
                .iter()
                .any(|item| item.contains("only after"))
        );
    }

    #[test]
    fn missing_native_registration_fails_closed() {
        let mut inputs = fixture();
        inputs.lib_rs = inputs.lib_rs.replace("kgw_copy_text_to_clipboard_v1", "");
        assert!(
            validate(&inputs)
                .iter()
                .any(|item| item.contains("must be registered"))
        );
    }

    #[test]
    fn missing_frontend_regression_marker_fails_closed() {
        let mut inputs = fixture();
        inputs.frontend_test = inputs
            .frontend_test
            .replace("Copy Log trace must exclude raw clipboard content", "");
        assert!(
            validate(&inputs)
                .iter()
                .any(|item| item.contains("trace redaction"))
        );
    }
}
