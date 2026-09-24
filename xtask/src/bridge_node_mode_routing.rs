use regex::Regex;
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BRIDGE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const NODE_JS: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const INTEGRATED_RUNTIME: &str =
    "apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs";
const TAURI_LIB: &str = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
const RK_BRIDGE: &str = "crates/kaspa-gateway-rk-bridge/src/lib.rs";

const REQUIRED_FILES: [(&str, &str); 5] = [
    ("bridgeJs", BRIDGE_JS),
    ("nodeJs", NODE_JS),
    ("integratedRuntime", INTEGRATED_RUNTIME),
    ("tauriLib", TAURI_LIB),
    ("rkBridge", RK_BRIDGE),
];

#[derive(Debug, Clone)]
struct Loaded {
    rel: &'static str,
    abs: PathBuf,
    exists: bool,
    text: String,
}

#[derive(Debug, Clone)]
struct Extract {
    name: String,
    start_line: usize,
    end_line: usize,
    text: String,
}

pub struct AuditResult {
    pub output: String,
    pub code: i32,
}

pub fn run_cli(args: &mut impl Iterator<Item = String>) -> Result<AuditResult, String> {
    let repo_root = PathBuf::from(
        args.next()
            .ok_or_else(|| "bridge-node-mode-routing-audit requires <repo-root>".to_owned())?,
    );
    let report_dir = PathBuf::from(
        args.next()
            .ok_or_else(|| "bridge-node-mode-routing-audit requires <report-dir>".to_owned())?,
    );
    if args.next().is_some() {
        return Err(
            "bridge-node-mode-routing-audit takes exactly <repo-root> <report-dir>".to_owned(),
        );
    }
    run(&repo_root, &report_dir)
}

fn read_file(root: &Path, rel: &'static str) -> Loaded {
    let abs = root.join(rel);
    match fs::read_to_string(&abs) {
        Ok(text) => Loaded {
            rel,
            abs,
            exists: true,
            text,
        },
        Err(_) => Loaded {
            rel,
            abs,
            exists: false,
            text: String::new(),
        },
    }
}

fn write_text(report_dir: &Path, relative: &Path, text: &str) -> Result<(), String> {
    let out = report_dir.join(relative);
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&out, text.as_bytes())
        .map_err(|error| format!("failed to write {}: {error}", out.display()))
}

fn line_number_at(text: &str, byte_index: usize) -> usize {
    text[..byte_index]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn find_all_literal(text: &str, needle: &str, limit: usize) -> Vec<Value> {
    text.match_indices(needle)
        .take(limit)
        .map(|(index, matched)| {
            json!({
                "index": index,
                "line": line_number_at(text, index),
                "match": matched,
                "groups": [],
            })
        })
        .collect()
}

fn extract_function_by_name(text: &str, name: &str) -> Option<Extract> {
    let escaped = regex::escape(name);
    let patterns = [
        format!(r"function\s+{escaped}\s*\("),
        format!(r"const\s+{escaped}\s*=\s*\("),
        format!(r"let\s+{escaped}\s*=\s*\("),
        format!(r"async\s+function\s+{escaped}\s*\("),
        format!(r"fn\s+{escaped}\s*\("),
        format!(r"pub\s+fn\s+{escaped}\s*\("),
        format!(r"async\s+fn\s+{escaped}\s*\("),
        format!(r"pub\s+async\s+fn\s+{escaped}\s*\("),
    ];

    let mut start = None;
    for pattern in patterns {
        let regex = Regex::new(&pattern).ok()?;
        if let Some(found) = regex.find(text) {
            start = Some(found.start());
            break;
        }
    }
    let start = start?;
    let brace = text[start..].find('{').map(|offset| start + offset)?;

    let bytes = text.as_bytes();
    let mut depth = 0_i32;
    let mut in_string = false;
    let mut quote = 0_u8;
    let mut escape = false;
    let mut index = brace;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == quote {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'"' | b'\'' | b'`') {
            in_string = true;
            quote = byte;
            index += 1;
            continue;
        }
        if byte == b'{' {
            depth += 1;
        } else if byte == b'}' {
            depth -= 1;
            if depth == 0 {
                let end = index + 1;
                return Some(Extract {
                    name: name.to_owned(),
                    start_line: line_number_at(text, start),
                    end_line: line_number_at(text, end),
                    text: text[start..end].to_owned(),
                });
            }
        }
        index += 1;
    }
    None
}

fn command_result(program: &str, args: &[&str], cwd: &Path) -> Value {
    match Command::new(program).args(args).current_dir(cwd).output() {
        Ok(output) => json!({
            "ok": output.status.success(),
            "status": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "error": "",
        }),
        Err(error) => json!({
            "ok": false,
            "status": Value::Null,
            "stdout": "",
            "stderr": "",
            "error": error.to_string(),
        }),
    }
}

fn add_finding(findings: &mut Vec<Value>, severity: &str, area: &str, message: &str, extra: Value) {
    let mut object = Map::new();
    object.insert("severity".to_owned(), json!(severity));
    object.insert("area".to_owned(), json!(area));
    object.insert("message".to_owned(), json!(message));
    if let Some(extra) = extra.as_object() {
        for (key, value) in extra {
            object.insert(key.clone(), value.clone());
        }
    }
    findings.push(Value::Object(object));
}

fn needle_map(text: &str, needles: &[&str], limit: usize) -> Value {
    let mut map = Map::new();
    for needle in needles {
        map.insert(
            (*needle).to_owned(),
            Value::Array(find_all_literal(text, needle, limit)),
        );
    }
    Value::Object(map)
}

fn save_extract(
    report_dir: &Path,
    label: &str,
    loaded: &Loaded,
    function_name: &str,
    extracted: &mut Map<String, Value>,
) -> Result<String, String> {
    if let Some(extract) = extract_function_by_name(&loaded.text, function_name) {
        write_text(
            report_dir,
            Path::new("extracts").join(format!("{label}.txt")).as_path(),
            &extract.text,
        )?;
        extracted.insert(
            label.to_owned(),
            json!({
                "rel": loaded.rel,
                "name": extract.name,
                "startLine": extract.start_line,
                "endLine": extract.end_line,
            }),
        );
        Ok(extract.text)
    } else {
        extracted.insert(label.to_owned(), Value::Null);
        Ok(String::new())
    }
}

fn bool_signal(pattern: &str, mainline: bool, tn13: bool, run_bridge: bool) -> Value {
    json!({
        "pattern": pattern,
        "mainline": mainline,
        "tn13": tn13,
        "runBridge": run_bridge,
    })
}

pub fn run(repo_root: &Path, report_dir: &Path) -> Result<AuditResult, String> {
    fs::create_dir_all(report_dir)
        .map_err(|error| format!("failed to create {}: {error}", report_dir.display()))?;
    fs::create_dir_all(report_dir.join("full-files"))
        .map_err(|error| format!("failed to create full-files: {error}"))?;
    fs::create_dir_all(report_dir.join("extracts"))
        .map_err(|error| format!("failed to create extracts: {error}"))?;

    let bridge_js = read_file(repo_root, BRIDGE_JS);
    let node_js = read_file(repo_root, NODE_JS);
    let integrated_runtime = read_file(repo_root, INTEGRATED_RUNTIME);
    let tauri_lib = read_file(repo_root, TAURI_LIB);
    let rk_bridge = read_file(repo_root, RK_BRIDGE);

    let loaded = [
        &bridge_js,
        &node_js,
        &integrated_runtime,
        &tauri_lib,
        &rk_bridge,
    ];

    for file in loaded {
        if file.exists {
            let name = file.rel.replace(['/', '\\'], "__");
            write_text(
                report_dir,
                Path::new("full-files").join(name).as_path(),
                &file.text,
            )?;
        }
    }

    let mut blockers = Vec::new();
    for (key, rel) in REQUIRED_FILES {
        let exists = match key {
            "bridgeJs" => bridge_js.exists,
            "nodeJs" => node_js.exists,
            "integratedRuntime" => integrated_runtime.exists,
            "tauriLib" => tauri_lib.exists,
            "rkBridge" => rk_bridge.exists,
            _ => false,
        };
        if !exists {
            blockers.push(format!("Missing required file: {rel}"));
        }
    }

    let warnings: Vec<String> = Vec::new();
    let mut findings = Vec::new();
    let mut evidence = Map::new();

    if blockers.is_empty() {
        let frontend_needles = [
            "buildCommandLines",
            "buildApplyPayload",
            "nodeMode",
            "--node-mode",
            "runtimeRole",
            "nodeKind",
            "bridgeKind",
            "official-external-node",
            "official-inprocess-node",
            "invoke(",
        ];
        let rust_needles = [
            "kgw_apply_command_preview_overrides",
            "kgw_worker_start",
            "try_run_kgw_self_worker_from_args",
            "kgw_run_bridge_self_worker",
            "BridgeRuntimeSettings",
            "BridgeRuntimeMode::OfficialExternalNode",
            "BridgeRuntimeMode::OfficialInProcessNode",
            "StartOfficialExternalNode",
            "StartOfficialInProcessNode",
            "--node-mode",
            "node-mode",
            "node_mode",
            "kaspad",
            "rpclisten",
            "utxoindex",
        ];

        evidence.insert(
            "bridgeJsNeedles".to_owned(),
            needle_map(&bridge_js.text, &frontend_needles, 50),
        );
        evidence.insert(
            "tauriLibNeedles".to_owned(),
            needle_map(&tauri_lib.text, &rust_needles, 80),
        );
        evidence.insert(
            "integratedRuntimeNeedles".to_owned(),
            needle_map(&integrated_runtime.text, &rust_needles, 80),
        );
        evidence.insert(
            "rkBridgeNeedles".to_owned(),
            needle_map(&rk_bridge.text, &rust_needles, 80),
        );

        let mut extracted = Map::new();
        let _build_command_lines = save_extract(
            report_dir,
            "bridgeJs_buildCommandLines",
            &bridge_js,
            "buildCommandLines",
            &mut extracted,
        )?;
        let build_apply_payload = save_extract(
            report_dir,
            "bridgeJs_buildApplyPayload",
            &bridge_js,
            "buildApplyPayload",
            &mut extracted,
        )?;
        let _install_actions = save_extract(
            report_dir,
            "bridgeJs_installActions",
            &bridge_js,
            "installActions",
            &mut extracted,
        )?;
        let _apply_overrides = save_extract(
            report_dir,
            "integratedRuntime_applyOverrides",
            &integrated_runtime,
            "kgw_apply_command_preview_overrides",
            &mut extracted,
        )?;
        let _worker_start = save_extract(
            report_dir,
            "integratedRuntime_workerStart",
            &integrated_runtime,
            "kgw_worker_start",
            &mut extracted,
        )?;
        let _try_self = save_extract(
            report_dir,
            "tauriLib_trySelfWorkerArgs",
            &tauri_lib,
            "try_run_kgw_self_worker_from_args",
            &mut extracted,
        )?;
        let run_bridge = save_extract(
            report_dir,
            "tauriLib_runBridgeSelfWorker",
            &tauri_lib,
            "kgw_run_bridge_self_worker",
            &mut extracted,
        )?;
        let _start_official = save_extract(
            report_dir,
            "rkBridge_startOfficialBridgeOwner",
            &rk_bridge,
            "start_official_bridge_owner_thread_v1",
            &mut extracted,
        )?;
        let mainline = save_extract(
            report_dir,
            "rkBridge_startMainlineBridgeOwner",
            &rk_bridge,
            "start_mainline_bridge_owner_thread",
            &mut extracted,
        )?;
        let tn13 = save_extract(
            report_dir,
            "rkBridge_startTn13BridgeOwner",
            &rk_bridge,
            "start_tn13_bridge_owner_thread",
            &mut extracted,
        )?;
        let _event = save_extract(
            report_dir,
            "rkBridge_bridgeEventFromSettings",
            &rk_bridge,
            "bridge_service_event_from_settings_v1",
            &mut extracted,
        )?;
        evidence.insert("extractedFunctions".to_owned(), Value::Object(extracted));

        let has_frontend_node_mode = Regex::new(r"\bnodeMode\b")
            .unwrap()
            .is_match(&bridge_js.text)
            && bridge_js.text.contains("--node-mode");
        let payload_forces_external =
            Regex::new(r#"bridgeKind\s*:\s*["']official-external-node["']"#)
                .unwrap()
                .is_match(&build_apply_payload)
                || Regex::new(r#"nodeKind\s*:\s*["']remote["']"#)
                    .unwrap()
                    .is_match(&build_apply_payload);

        if has_frontend_node_mode {
            add_finding(
                &mut findings,
                "INFO",
                "frontend-preview",
                "Bridge tab appears to build or display --node-mode from a nodeMode setting.",
                json!({"file": BRIDGE_JS}),
            );
        } else {
            add_finding(
                &mut findings,
                "HIGH",
                "frontend-preview",
                "Bridge tab did not show a clear nodeMode / --node-mode preview owner.",
                json!({"file": BRIDGE_JS}),
            );
        }

        if payload_forces_external {
            add_finding(
                &mut findings,
                "CRITICAL",
                "frontend-start-payload",
                "Bridge Start payload appears to force external-node semantics even when preview may show inprocess.",
                json!({"file": BRIDGE_JS}),
            );
        } else {
            add_finding(
                &mut findings,
                "WARN",
                "frontend-start-payload",
                "Bridge Start payload did not clearly force external mode in the extracted buildApplyPayload function; inspect extract manually.",
                json!({"file": BRIDGE_JS}),
            );
        }

        let backend_forces_external = run_bridge
            .contains("mode: kaspa_gateway_rk_bridge::BridgeRuntimeMode::OfficialExternalNode")
            || run_bridge.contains("BridgeRuntimeMode::OfficialExternalNode");
        let backend_parses_node_mode = Regex::new(r"node[-_]mode").unwrap().is_match(&run_bridge)
            || run_bridge.contains("OfficialInProcessNode")
            || run_bridge.contains("--node-mode");

        if backend_forces_external && !backend_parses_node_mode {
            add_finding(
                &mut findings,
                "CRITICAL",
                "tauri-self-worker",
                "kgw_run_bridge_self_worker appears to construct BridgeRuntimeSettings with OfficialExternalNode and does not parse node-mode into OfficialInProcessNode.",
                json!({"file": TAURI_LIB}),
            );
        } else if backend_forces_external && backend_parses_node_mode {
            add_finding(
                &mut findings,
                "HIGH",
                "tauri-self-worker",
                "kgw_run_bridge_self_worker references external mode and node-mode/inprocess markers; inspect whether branching is real or dead.",
                json!({"file": TAURI_LIB}),
            );
        } else {
            add_finding(
                &mut findings,
                "WARN",
                "tauri-self-worker",
                "Could not prove forced external mode from kgw_run_bridge_self_worker extract.",
                json!({"file": TAURI_LIB}),
            );
        }

        let enum_has_inprocess = rk_bridge.text.contains("OfficialInProcessNode")
            && rk_bridge.text.contains("StartOfficialInProcessNode");
        if enum_has_inprocess {
            add_finding(
                &mut findings,
                "INFO",
                "rk-bridge-api",
                "Bridge crate exposes OfficialInProcessNode / StartOfficialInProcessNode names.",
                json!({"file": RK_BRIDGE}),
            );
        } else {
            add_finding(
                &mut findings,
                "HIGH",
                "rk-bridge-api",
                "Bridge crate does not expose clear in-process runtime names.",
                json!({"file": RK_BRIDGE}),
            );
        }

        let empty_inprocess = Regex::new(
            r"if\s+event\.mode\s*==\s*BridgeRuntimeMode::OfficialInProcessNode\s*\{\s*\}",
        )
        .unwrap()
        .find_iter(&rk_bridge.text)
        .map(|matched| line_number_at(&rk_bridge.text, matched.start()))
        .collect::<Vec<_>>();
        if !empty_inprocess.is_empty() {
            add_finding(
                &mut findings,
                "CRITICAL",
                "rk-bridge-implementation",
                "Bridge owner contains empty OfficialInProcessNode branches; in-process mode name exists but appears unimplemented.",
                json!({"file": RK_BRIDGE, "lines": empty_inprocess}),
            );
        }

        let mainline_uses_rpc = Regex::new(r"KaspaApi::new\s*\(\s*event\.kaspa_rpc_endpoint")
            .unwrap()
            .is_match(&mainline);
        let tn13_uses_rpc = Regex::new(r"KaspaApi::new\s*\(\s*event\.kaspa_rpc_endpoint")
            .unwrap()
            .is_match(&tn13);
        if mainline_uses_rpc || tn13_uses_rpc {
            add_finding(
                &mut findings,
                "HIGH",
                "rk-bridge-owner-runtime",
                "Bridge owner startup uses event.kaspa_rpc_endpoint via KaspaApi::new, which is external-node style connectivity.",
                json!({
                    "file": RK_BRIDGE,
                    "mainlineUsesRpcApi": mainline_uses_rpc,
                    "tn13UsesRpcApi": tn13_uses_rpc,
                }),
            );
        }

        let combined_owner = format!("{mainline}\n{tn13}\n{run_bridge}");
        let signal_specs: [(&str, &str, bool); 8] = [
            ("/kaspad/i", "kaspad", true),
            ("/start.*node/i", "start.*node", true),
            ("/spawn/i", "spawn", true),
            ("/Command::new/i", "Command::new", false),
            (
                "/StartOfficialInProcessNode/",
                "StartOfficialInProcessNode",
                false,
            ),
            ("/--utxoindex/", "--utxoindex", false),
            ("/--rpclisten/", "--rpclisten", false),
            ("/appdir/", "appdir", false),
        ];
        let mut signals = Vec::new();
        for (label, pattern, insensitive) in signal_specs {
            let regex = if insensitive {
                Regex::new(&format!("(?i){pattern}")).unwrap()
            } else {
                Regex::new(pattern).unwrap()
            };
            signals.push(bool_signal(
                label,
                regex.is_match(&mainline),
                regex.is_match(&tn13),
                regex.is_match(&run_bridge),
            ));
        }
        evidence.insert(
            "inprocessImplementationSignals".to_owned(),
            Value::Array(signals.clone()),
        );

        let has_spawn_in_owner = Regex::new(r"Command::new|\.spawn\s*\(")
            .unwrap()
            .is_match(&combined_owner);
        let has_kaspad_args_in_owner = Regex::new(r"(?i)--utxoindex|--rpclisten|appdir|kaspad")
            .unwrap()
            .is_match(&combined_owner);

        if !has_spawn_in_owner && !has_kaspad_args_in_owner {
            add_finding(
                &mut findings,
                "CRITICAL",
                "inprocess-missing-runtime",
                "No clear kaspad spawn or kaspad-argument handling was found in the bridge self-worker/owner extracts. This strongly indicates in-process mode is not actually implemented.",
                json!({"files": [TAURI_LIB, RK_BRIDGE]}),
            );
        } else {
            add_finding(
                &mut findings,
                "WARN",
                "inprocess-runtime-signals",
                "Some kaspad/spawn/argument signals exist; inspect extracts to decide whether they are real implementation or unrelated text.",
                json!({"signals": signals}),
            );
        }

        let command_preview_has_separator =
            bridge_js.text.contains("--") && bridge_js.text.contains("rpclisten");
        if command_preview_has_separator {
            add_finding(
                &mut findings,
                "INFO",
                "frontend-command-preview",
                "Bridge command preview appears to include in-process separator/kaspad-argument concepts.",
                json!({"file": BRIDGE_JS}),
            );
        } else {
            add_finding(
                &mut findings,
                "HIGH",
                "frontend-command-preview",
                "Bridge command preview does not clearly include the required in-process '--' separator and kaspad args.",
                json!({"file": BRIDGE_JS}),
            );
        }

        evidence.insert(
            "nextPatchTargets".to_owned(),
            json!([
                {
                    "file": BRIDGE_JS,
                    "requiredChange": "buildApplyPayload must preserve selected nodeMode and bridge runtime mode instead of forcing official-external-node / remote."
                },
                {
                    "file": TAURI_LIB,
                    "requiredChange": "kgw_run_bridge_self_worker must parse selected node-mode / bridge kind into BridgeRuntimeMode::OfficialExternalNode or OfficialInProcessNode."
                },
                {
                    "file": RK_BRIDGE,
                    "requiredChange": "OfficialInProcessNode path must actually start or attach an in-process kaspad runtime and pass kaspad args after the '--' separator, instead of using only event.kaspa_rpc_endpoint."
                }
            ]),
        );

        let verdict = if findings
            .iter()
            .any(|finding| finding["severity"] == "CRITICAL")
        {
            "BRIDGE_INPROCESS_ROUTING_BROKEN_CONFIRMED"
        } else if findings.iter().any(|finding| finding["severity"] == "HIGH") {
            "BRIDGE_INPROCESS_ROUTING_RISK_FOUND"
        } else {
            "BRIDGE_INPROCESS_ROUTING_NOT_PROVEN_BROKEN"
        };
        evidence.insert("verdict".to_owned(), json!(verdict));
    }

    let mut node_checks = Map::new();
    if bridge_js.exists {
        node_checks.insert(
            "bridgeJsSyntax".to_owned(),
            command_result(
                "node",
                &["--check", bridge_js.abs.to_string_lossy().as_ref()],
                repo_root,
            ),
        );
    }
    if node_js.exists {
        node_checks.insert(
            "nodeJsSyntax".to_owned(),
            command_result(
                "node",
                &["--check", node_js.abs.to_string_lossy().as_ref()],
                repo_root,
            ),
        );
    }

    write_text(
        report_dir,
        Path::new("EVIDENCE.json"),
        &(serde_json::to_string_pretty(&Value::Object(evidence.clone()))
            .map_err(|error| format!("failed to serialize evidence: {error}"))?
            + "\n"),
    )?;
    let findings_document = json!({
        "blockers": blockers,
        "warnings": warnings,
        "findings": findings,
        "nodeChecks": Value::Object(node_checks),
    });
    write_text(
        report_dir,
        Path::new("FINDINGS.json"),
        &(serde_json::to_string_pretty(&findings_document)
            .map_err(|error| format!("failed to serialize findings: {error}"))?
            + "\n"),
    )?;

    let blockers = findings_document["blockers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let findings = findings_document["findings"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let critical = findings
        .iter()
        .filter(|finding| finding["severity"] == "CRITICAL")
        .collect::<Vec<_>>();
    let high = findings
        .iter()
        .filter(|finding| finding["severity"] == "HIGH")
        .collect::<Vec<_>>();
    let warn = findings
        .iter()
        .filter(|finding| finding["severity"] == "WARN")
        .collect::<Vec<_>>();
    let info = findings
        .iter()
        .filter(|finding| finding["severity"] == "INFO")
        .collect::<Vec<_>>();

    let report = render_report(
        repo_root, report_dir, &blockers, &critical, &high, &warn, &info,
    );
    let report_name = if blockers.is_empty() {
        "REPORT_SUCCESS.md"
    } else {
        "REPORT_FAILED.md"
    };
    write_text(report_dir, Path::new(report_name), &report)?;

    Ok(AuditResult {
        output: report,
        code: if blockers.is_empty() { 0 } else { 1 },
    })
}

fn finding_text(finding: &Value) -> (&str, &str, Option<&str>, Option<&Vec<Value>>) {
    (
        finding["area"].as_str().unwrap_or("unknown"),
        finding["message"].as_str().unwrap_or(""),
        finding["file"].as_str(),
        finding["lines"].as_array(),
    )
}

fn render_report(
    repo_root: &Path,
    report_dir: &Path,
    blockers: &[Value],
    critical: &[&Value],
    high: &[&Value],
    warn: &[&Value],
    info: &[&Value],
) -> String {
    let mut lines = Vec::new();
    lines.push("# KGW Bridge Node Mode Routing Audit V1".to_owned());
    lines.push(String::new());
    lines.push(format!("- Repository: {}", repo_root.display()));
    lines.push(format!("- Report directory: {}", report_dir.display()));
    lines.push("- Source mutation: no".to_owned());
    lines.push("- Backup created: no, audit only".to_owned());
    lines.push("- Git commit: no".to_owned());
    lines.push("- Git push: no".to_owned());
    lines.push(String::new());
    lines.push("## Verdict".to_owned());
    lines.push(String::new());
    if !blockers.is_empty() {
        lines.push("FAILED: required files are missing.".to_owned());
    } else if !critical.is_empty() {
        lines.push("CONFIRMED: Bridge in-process routing is broken or incomplete.".to_owned());
    } else if !high.is_empty() {
        lines.push(
            "RISK FOUND: Bridge in-process routing has high-risk issues, but no critical proof was found."
                .to_owned(),
        );
    } else {
        lines.push("NOT CONFIRMED: no major routing break was proven by this audit.".to_owned());
    }

    lines.push(String::new());
    lines.push("## Blockers".to_owned());
    lines.push(String::new());
    if blockers.is_empty() {
        lines.push("- None".to_owned());
    } else {
        for blocker in blockers {
            lines.push(format!(
                "- {}",
                blocker.as_str().unwrap_or("unknown blocker")
            ));
        }
    }

    for (heading, entries) in [
        ("## Critical Findings", critical),
        ("## High Findings", high),
        ("## Warnings", warn),
        ("## Informational Findings", info),
    ] {
        lines.push(String::new());
        lines.push(heading.to_owned());
        lines.push(String::new());
        if entries.is_empty() {
            lines.push("- None".to_owned());
        }
        for finding in entries {
            let (area, message, file, finding_lines) = finding_text(finding);
            lines.push(format!("- [{area}] {message}"));
            if let Some(file) = file {
                lines.push(format!("  - File: {file}"));
            }
            if let Some(finding_lines) = finding_lines {
                let rendered = finding_lines
                    .iter()
                    .filter_map(Value::as_u64)
                    .map(|line| line.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                if !rendered.is_empty() {
                    lines.push(format!("  - Lines: {rendered}"));
                }
            }
        }
    }

    lines.push(String::new());
    lines.push("## Extracts Saved".to_owned());
    lines.push(String::new());
    lines.push("- full-files/".to_owned());
    lines.push("- extracts/".to_owned());
    lines.push("- EVIDENCE.json".to_owned());
    lines.push("- FINDINGS.json".to_owned());
    lines.push(String::new());
    lines.push("## Expected Correct Model From dagknight Docs".to_owned());
    lines.push(String::new());
    lines.push(
        "- external mode: stratum-bridge connects to an already running kaspad through RPC."
            .to_owned(),
    );
    lines.push(
        "- inprocess mode: stratum-bridge starts/runs kaspad internally and kaspad args must be passed after the required '--' separator."
            .to_owned(),
    );
    lines.push(
        "- internal CPU miner is a separate flag axis and must not be treated as equivalent to inprocess node mode."
            .to_owned(),
    );
    lines.push(String::new());
    lines.push("## Next Safe Step".to_owned());
    lines.push(String::new());
    if !critical.is_empty() || !high.is_empty() {
        lines.push(
            "Write a patch only after reviewing this report. The patch must preserve selected bridge nodeMode from Bridge tab into Tauri backend, map it into BridgeRuntimeMode, and implement or route true in-process owner behavior without adding duplicate UI/runtime owners."
                .to_owned(),
        );
    } else {
        lines.push(
            "Manually inspect extracts before patching because the audit did not prove the break."
                .to_owned(),
        );
    }
    lines.push(String::new());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "kgw-bridge-node-mode-{label}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn function_extractor_handles_balanced_js_and_rust_bodies() {
        let js = r#"
function buildApplyPayload() {
  const text = "{not-a-brace}";
  return { nodeMode: "external" };
}
function next() {}
"#;
        let extract = extract_function_by_name(js, "buildApplyPayload").unwrap();
        assert!(extract.text.contains("return { nodeMode"));
        assert!(!extract.text.contains("function next"));

        let rust = r#"
pub async fn kgw_run_bridge_self_worker() {
    let mode = "}";
    if true { println!("ok"); }
}
"#;
        let extract = extract_function_by_name(rust, "kgw_run_bridge_self_worker").unwrap();
        assert!(extract.text.contains("println!"));
    }

    #[test]
    fn needle_inventory_records_line_numbers() {
        let found = find_all_literal("a\nnodeMode\nb nodeMode", "nodeMode", 50);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0]["line"], 2);
        assert_eq!(found[1]["line"], 3);
    }

    #[test]
    fn missing_required_files_fail_closed_and_write_failed_report() {
        let root = temp_path("missing-root");
        let report = temp_path("missing-report");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&report);
        fs::create_dir_all(&root).unwrap();

        let result = run(&root, &report).unwrap();
        assert_eq!(result.code, 1);
        assert!(
            result
                .output
                .contains("FAILED: required files are missing.")
        );
        assert!(report.join("REPORT_FAILED.md").is_file());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(report);
    }

    #[test]
    fn risk_report_uses_legacy_verdict_text() {
        let high = json!({
            "severity": "HIGH",
            "area": "tauri-self-worker",
            "message": "risk",
            "file": TAURI_LIB,
        });
        let report = render_report(
            Path::new("repo"),
            Path::new("report"),
            &[],
            &[],
            &[&high],
            &[],
            &[],
        );
        assert!(report.contains(
            "RISK FOUND: Bridge in-process routing has high-risk issues, but no critical proof was found."
        ));
        assert!(report.contains("- [tauri-self-worker] risk"));
    }
}
