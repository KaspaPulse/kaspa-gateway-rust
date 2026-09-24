use regex::Regex;
use std::fs;
use std::path::Path;

const INTEGRATED: &str = "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs";
const LIB: &str = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
const NODE_OWNER: &str = "crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs";
const BRIDGE_OWNER: &str = "crates/kaspa-gateway-rk-bridge/src/lib.rs";
const SMOKE: &str = "apps/kaspa-gateway-desktop/src-tauri/src/bin/kgw-provenance-smoke.rs";
const NODE_FRONTEND: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const BRIDGE_FRONTEND: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";

#[derive(Debug, Clone, Eq, PartialEq)]
struct Check {
    name: String,
    ok: bool,
}

struct Sources {
    integrated: String,
    lib: String,
    node_owner: String,
    bridge_owner: String,
    smoke: String,
    node_frontend: String,
    bridge_frontend: String,
    true_raw_gate: String,
    full_local_gate: String,
}
pub fn run(root: &Path) -> Result<String, String> {
    let sources = load_sources(root)?;
    let checks = evaluate(&sources)?;
    let failed: Vec<_> = checks.iter().filter(|check| !check.ok).collect();
    if !failed.is_empty() {
        let mut message = String::from("KGW raw log provenance gate FAILED");
        for check in failed {
            message.push_str("\n- ");
            message.push_str(&check.name);
        }
        return Err(message);
    }

    let mut message = String::from("KGW raw log provenance gate");
    for check in checks {
        message.push_str("\nPASS ");
        message.push_str(&check.name);
    }
    message.push_str("\nrawProducersUnknown: 0\nproductionTextFiltering: false\nstatus: PASS");
    Ok(message)
}

fn load_sources(root: &Path) -> Result<Sources, String> {
    Ok(Sources {
        integrated: read(root, INTEGRATED)?,
        lib: read(root, LIB)?,
        node_owner: read(root, NODE_OWNER)?,
        bridge_owner: read(root, BRIDGE_OWNER)?,
        smoke: read(root, SMOKE)?,
        node_frontend: read(root, NODE_FRONTEND)?,
        bridge_frontend: read(root, BRIDGE_FRONTEND)?,
        true_raw_gate: read(root, "tools/kgw_true_raw_log_gate.ps1")?,
        full_local_gate: read(root, "tools/kgw_full_local_gate.ps1")?,
    })
}
fn read(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("raw-log provenance: failed to read {relative}: {error}"))?;
    Ok(text.replace("\r\n", "\n").replace('\r', "\n"))
}

fn count(source: &str, literal: &str) -> usize {
    source.match_indices(literal).count()
}

fn function_body(source: &str, name: &str) -> Result<String, String> {
    let rust_marker = format!("fn {name}");
    let js_marker = format!("function {name}");
    let start = source
        .find(&rust_marker)
        .or_else(|| source.find(&js_marker))
        .ok_or_else(|| format!("Missing function: {name}"))?;
    let open = source[start..]
        .find('{')
        .map(|offset| start + offset)
        .ok_or_else(|| format!("Missing function body: {name}"))?;

    let bytes = source.as_bytes();
    let mut depth = 0_i64;
    let mut index = open;
    let mut quote: Option<u8> = None;
    let mut line_comment = false;
    let mut block_comment = false;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied().unwrap_or_default();

        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            index += 1;
            continue;
        }
        if block_comment {
            if byte == b'*' && next == b'/' {
                block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(active_quote) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && next == b'/' {
            line_comment = true;
            index += 2;
            continue;
        }
        if byte == b'/' && next == b'*' {
            block_comment = true;
            index += 2;
            continue;
        }
        if matches!(byte, b'"' | b'\'' | b'`') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if byte == b'{' {
            depth += 1;
        } else if byte == b'}' {
            depth -= 1;
            if depth == 0 {
                return Ok(source[start..=index].to_owned());
            }
        }
        index += 1;
    }
    Err(format!("Unterminated function: {name}"))
}

fn check(checks: &mut Vec<Check>, name: impl Into<String>, ok: bool) {
    checks.push(Check {
        name: name.into(),
        ok,
    });
}

fn emitter_pattern() -> Regex {
    Regex::new(
        r"(?:println|eprintln|print|eprint)!|(?:tracing|log)::(?:trace|debug|info|warn|error)!|\b(?:trace|debug|info|warn|error)!",
    )
    .unwrap()
}

fn owner_emitter_pattern() -> Regex {
    Regex::new(
        r"(?:println|eprintln|print|eprint)!|(?:tracing|log)::(?:trace|debug|info|warn|error)!",
    )
    .unwrap()
}

fn before_test_module(source: &str) -> &str {
    source
        .split_once("#[cfg(test)]\nmod tests")
        .map_or(source, |(before, _)| before)
}

fn evaluate(s: &Sources) -> Result<Vec<Check>, String> {
    let mut checks = Vec::new();

    check(
        &mut checks,
        "one production raw entry construction call",
        count(&s.integrated, "kgw_worker_raw_log_entry_v1(") == 2,
    );
    check(
        &mut checks,
        "one production raw buffer insertion call",
        count(&s.integrated, "kgw_worker_push_raw_log(") == 2,
    );
    check(
        &mut checks,
        "one underlying raw push_back",
        count(&s.integrated, "guard.push_back(entry);") == 1,
    );
    let pipe_reader = function_body(&s.integrated, "kgw_worker_spawn_reader")?;
    check(
        &mut checks,
        "raw construction is inside child pipe reader",
        pipe_reader.contains("kgw_worker_raw_log_entry_v1"),
    );
    check(
        &mut checks,
        "raw insertion is inside child pipe reader",
        pipe_reader.contains("kgw_worker_push_raw_log"),
    );
    check(
        &mut checks,
        "raw constructor accepts no Bridge listener attribution",
        !function_body(&s.integrated, "kgw_worker_raw_log_entry_v1")?
            .contains("_bridge_instance_id"),
    );
    check(
        &mut checks,
        "stdout and stderr are OS pipes",
        count(&s.integrated, ".stdout(Stdio::piped())") >= 1
            && count(&s.integrated, ".stderr(Stdio::piped())") >= 1,
    );

    let spawn_order = Regex::new(
        r"kgw_worker_replace_raw_log_buffer_v1\(&role, &network\)\?;\s*let mut child = match command\.spawn\(\)",
    )
    .unwrap();
    check(
        &mut checks,
        "raw buffer is registered immediately before spawn",
        spawn_order.is_match(&s.integrated),
    );
    let test_capture = Regex::new(
        r"(?s)#\[cfg\(test\)\]\s*#\[allow\(dead_code\)\].*?fn kgw_capture_raw_pipe_for_test_v1",
    )
    .unwrap();
    check(
        &mut checks,
        "test raw capture helper is cfg(test) scoped",
        test_capture.is_match(&s.integrated),
    );

    let sentinel = "official_sentinel_stdout_and_stderr_use_the_production_pipe_reader_unchanged";
    check(
        &mut checks,
        "official sentinel test is wired into the true raw log gate",
        s.true_raw_gate.contains(sentinel),
    );
    check(
        &mut checks,
        "official sentinel test is wired into the full local gate",
        s.full_local_gate.contains(sentinel),
    );

    let emitter = emitter_pattern();
    for name in [
        "try_run_kgw_self_worker_from_args",
        "kgw_run_node_self_worker",
        "kgw_run_bridge_self_worker",
    ] {
        check(
            &mut checks,
            format!("no KGW emitter in production self-worker function {name}"),
            !emitter.is_match(&function_body(&s.lib, name)?),
        );
    }
    let bridge_writer = function_body(&s.lib, "kgw_init_bridge_self_worker_raw_tracing_r23")?;
    check(
        &mut checks,
        "Bridge writer plumbing emits no KGW record",
        !emitter.is_match(&bridge_writer) && bridge_writer.contains("with_writer(std::io::stderr)"),
    );

    let owner_emitter = owner_emitter_pattern();
    check(
        &mut checks,
        "no KGW node owner log emitter",
        !owner_emitter.is_match(before_test_module(&s.node_owner)),
    );
    check(
        &mut checks,
        "no KGW Bridge owner log emitter",
        !owner_emitter.is_match(before_test_module(&s.bridge_owner)),
    );

    let fixture_markers = [
        "#[cfg(test)]\n#[test]\nfn kgw_test_self_worker_hold",
        "#[cfg(test)]\n#[test]\nfn kgw_test_self_worker_fail",
        "#[cfg(test)]\n#[test]\nfn kgw_test_self_worker_delayed_fail",
    ];
    check(
        &mut checks,
        "test fixture emitters are cfg(test) scoped",
        fixture_markers
            .iter()
            .all(|marker| s.integrated.contains(marker)),
    );
    let node_normalizer = function_body(&s.node_frontend, "kgwNodeNormalizeRawLogEntryV1")?;
    check(
        &mut checks,
        "Node frontend has no raw content blacklist",
        !s.node_frontend.contains("RawLogTextHasTransportWrapper")
            && !node_normalizer.contains("rawTextValue).includes"),
    );

    let bridge_normalizer = function_body(&s.bridge_frontend, "kgwBridgeNormalizeRawLogEntryV1")?;
    check(
        &mut checks,
        "Bridge frontend has no raw content blacklist",
        !s.bridge_frontend.contains("RawLogTextHasTransportWrapper")
            && !bridge_normalizer.contains("rawTextValue).includes"),
    );

    let append_mutation = Regex::new(r"(textContent|records\.|\.push\(|\.set\()").unwrap();
    check(
        &mut checks,
        "Node legacy appendLog is inert",
        !append_mutation.is_match(&function_body(&s.node_frontend, "appendLog")?),
    );
    check(
        &mut checks,
        "Bridge legacy appendLog is inert",
        !append_mutation.is_match(&function_body(&s.bridge_frontend, "appendLog")?),
    );
    let bridge_key = function_body(&s.bridge_frontend, "kgwBridgeRawLogBufferKeyV1")?;
    check(
        &mut checks,
        "Bridge raw buffer key is process-level",
        bridge_key.contains("void instanceId") && !bridge_key.contains("String(instanceId"),
    );
    check(
        &mut checks,
        "Bridge raw normalizer ignores UI listener selection",
        bridge_normalizer.contains("void expectedInstanceId"),
    );

    let self_worker = function_body(&s.lib, "try_run_kgw_self_worker_from_args")?;
    check(
        &mut checks,
        "startup control is a typed file side channel",
        s.lib.contains("KgwStartupControlMessageV1")
            && s.lib.contains("kgw_write_startup_control_v1")
            && !self_worker.contains("println!"),
    );

    let stop = s.smoke.find("let stop_result = kgw_kgw_disable_network_v1");
    let last_print = s.smoke.rfind("println!");
    check(
        &mut checks,
        "live smoke parent output is emitted after child shutdown",
        stop.zip(last_print)
            .is_some_and(|(left, right)| left < right),
    );

    Ok(checks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn function_body_ignores_braces_inside_strings_and_comments() {
        let source = r#"
function sample() {
  const value = "{ not structural }";
  // }
  /* { } */
  if (true) { return value; }
}
function after() { return 1; }
"#;
        let body = function_body(source, "sample").unwrap();
        assert!(body.contains("if (true) { return value; }"));
        assert!(!body.contains("function after"));
    }

    #[test]
    fn missing_function_fails_closed() {
        assert_eq!(
            function_body("fn present() {}", "missing").unwrap_err(),
            "Missing function: missing"
        );
    }
    #[test]
    fn emitter_patterns_detect_production_emitters() {
        let full = emitter_pattern();
        let owner = owner_emitter_pattern();
        for value in [
            "println!(\"x\")",
            "eprintln!(\"x\")",
            "tracing::warn!(\"x\")",
            "log::error!(\"x\")",
        ] {
            assert!(full.is_match(value), "{value}");
            assert!(owner.is_match(value), "{value}");
        }
        assert!(!full.is_match("with_writer(std::io::stderr)"));
    }

    #[test]
    fn current_repository_matches_known_transport_filter_debt() {
        let sources = load_sources(&repo_root()).unwrap();
        let checks = evaluate(&sources).unwrap();
        let failed: Vec<_> = checks
            .iter()
            .filter(|check| !check.ok)
            .map(|check| check.name.as_str())
            .collect();
        assert_eq!(
            failed,
            vec![
                "Node frontend has no raw content blacklist",
                "Bridge frontend has no raw content blacklist",
            ]
        );
        assert_eq!(checks.len(), 26);
    }
}
