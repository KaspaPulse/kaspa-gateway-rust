from pathlib import Path


def replace_or_done(
    text: str,
    old: str,
    new: str,
    expected: int,
    label: str,
    *,
    optional: bool = False,
) -> str:
    old_count = text.count(old)
    new_count = text.count(new)

    if old_count == expected and new_count == 0:
        print(f"{label}: applying {old_count} repair(s)")
        return text.replace(old, new)

    if old_count == 0 and new_count == expected:
        print(f"{label}: already repaired")
        return text

    if optional and old_count == 0:
        print(f"{label}: source pattern not present; strict Clippy remains authoritative")
        return text

    raise RuntimeError(
        f"{label}: unexpected state: old={old_count}, new={new_count}, expected={expected}"
    )


def delete_once_or_done(text: str, old: str, label: str) -> str:
    count = text.count(old)
    if count == 1:
        print(f"{label}: deleting obsolete code")
        return text.replace(old, "", 1)
    if count == 0:
        print(f"{label}: already deleted")
        return text
    raise RuntimeError(f"{label}: expected at most one match, found {count}")


db = Path("crates/kaspa-gateway-db/src/lib.rs")
text = db.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''            let placeholders = std::iter::repeat("?")
                .take(chunk.len())
                .collect::<Vec<_>>()
                .join(",");''',
    '''            let placeholders = std::iter::repeat_n("?", chunk.len())
                .collect::<Vec<_>>()
                .join(",");''',
    1,
    "manual_repeat_n",
)
text = replace_or_done(
    text,
    '''        if let Some(tx_type) = filter.tx_type {
            if !tx_type.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND tx_type = ?");
                params.push(Box::new(tx_type.to_string()));
            }
        }''',
    '''        if let Some(tx_type) = filter.tx_type
            && !tx_type.eq_ignore_ascii_case("ALL")
        {
            query.push_str(" AND tx_type = ?");
            params.push(Box::new(tx_type.to_string()));
        }''',
    3,
    "tx_type collapsible_if",
)
text = replace_or_done(
    text,
    '''        if let Some(direction) = filter.direction {
            if !direction.eq_ignore_ascii_case("ALL") {
                query.push_str(" AND direction = ?");
                params.push(Box::new(direction.to_string()));
            }
        }''',
    '''        if let Some(direction) = filter.direction
            && !direction.eq_ignore_ascii_case("ALL")
        {
            query.push_str(" AND direction = ?");
            params.push(Box::new(direction.to_string()));
        }''',
    3,
    "direction collapsible_if",
)
text = replace_or_done(
    text,
    '''        if row_matches {
            if !seen.insert(tx.id.clone()) {
                continue;
            }
        }''',
    '''        if row_matches && !seen.insert(tx.id.clone()) {
            continue;
        }''',
    1,
    "row_matches collapsible_if",
    optional=True,
)
db.write_text(text, encoding="utf-8")

cfg = Path("crates/kaspa-gateway-config/src/lib.rs")
text = cfg.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            return Ok(parent.to_path_buf());
        }
    }''',
    '''    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        return Ok(parent.to_path_buf());
    }''',
    1,
    "current_exe collapsible_if",
)
text = replace_or_done(
    text,
    '''                    if let Value::String(text) = child {
                        if text.starts_with("keyring_managed:") {
                            *text = String::new();
                        }
                    }''',
    '''                    if let Value::String(text) = child
                        && text.starts_with("keyring_managed:")
                    {
                        *text = String::new();
                    }''',
    1,
    "keyring collapsible_if",
)
cfg.write_text(text, encoding="utf-8")

rk_owner = Path("crates/kaspa-gateway-rk-node/src/kgw_real_owner_runtime.rs")
text = rk_owner.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''#[allow(dead_code)]

fn ''',
    '''#[allow(dead_code)]
fn ''',
    5,
    "empty_line_after_outer_attr",
)
text = replace_or_done(
    text,
    '''    let mut args = kaspad_lib_mainline::args::Args::default();

    args.appdir = Some(kgw_owner_safe_runtime_appdir(settings.network));
    args.utxoindex = settings.enable_utxo_index;
    args.archival = settings.archival;
    args.yes = true;
    args.disable_upnp = true;
    args.log_level = "INFO".to_string();''',
    '''    let mut args = kaspad_lib_mainline::args::Args {
        appdir: Some(kgw_owner_safe_runtime_appdir(settings.network)),
        utxoindex: settings.enable_utxo_index,
        archival: settings.archival,
        yes: true,
        disable_upnp: true,
        log_level: "INFO".to_string(),
        ..Default::default()
    };''',
    1,
    "mainline field_reassign_with_default",
)
text = replace_or_done(
    text,
    '''    let mut args = kaspad_lib_tn12::args::Args::default();

    args.appdir = Some(kgw_owner_safe_runtime_appdir(settings.network));
    args.utxoindex = settings.enable_utxo_index;
    args.archival = settings.archival;
    args.yes = true;
    args.disable_upnp = true;
    args.log_level = "INFO".to_string();''',
    '''    let mut args = kaspad_lib_tn12::args::Args {
        appdir: Some(kgw_owner_safe_runtime_appdir(settings.network)),
        utxoindex: settings.enable_utxo_index,
        archival: settings.archival,
        yes: true,
        disable_upnp: true,
        log_level: "INFO".to_string(),
        ..Default::default()
    };''',
    1,
    "tn12 field_reassign_with_default",
)
rk_owner.write_text(text, encoding="utf-8")

node_old = '''        if !accumulated.contains_key(&key) {
            if let Some(tx) = transactions.remove(&tx_id) {
                accumulated.insert(key, tx);
            }
        }'''
node_new = '''        if !accumulated.contains_key(&key)
            && let Some(tx) = transactions.remove(&tx_id)
        {
            accumulated.insert(key, tx);
        }'''

old_matches: list[Path] = []
new_matches: list[Path] = []
for root in (Path("crates"), Path("apps")):
    if not root.exists():
        continue
    for path in root.rglob("*.rs"):
        source = path.read_text(encoding="utf-8")
        if node_old in source:
            old_matches.append(path)
        if node_new in source:
            new_matches.append(path)

if len(old_matches) == 1 and not new_matches:
    node_path = old_matches[0]
    node_text = node_path.read_text(encoding="utf-8")
    node_text = replace_or_done(
        node_text,
        node_old,
        node_new,
        1,
        f"node_rpc collapsible_if in {node_path}",
    )
    node_path.write_text(node_text, encoding="utf-8")
    print(f"node_rpc collapsible_if: repaired {node_path}")
elif not old_matches and len(new_matches) == 1:
    print(f"node_rpc collapsible_if: already repaired in {new_matches[0]}")
elif not old_matches and not new_matches:
    print("node_rpc collapsible_if: pattern not present in tracked Rust sources; strict Clippy will validate")
else:
    raise RuntimeError(
        "node_rpc collapsible_if: ambiguous source state: "
        f"old={old_matches}, new={new_matches}"
    )

# Third-layer Rust 1.97 repairs: remove genuinely unused test-only shims,
# reduce boundary argument counts with request/layout structs, and keep JS IPC payloads in sync.
integrated = Path("apps/kaspa-gateway-desktop/src-tauri/src/integrated_runtime_commands.rs")
text = integrated.read_text(encoding="utf-8")
text = delete_once_or_done(
    text,
    '''#[cfg(test)]
pub(crate) fn kgw_raw_log_entry_for_test_v1(
    sequence: u64,
    network: &str,
    runtime_role: &str,
    bridge_instance_id: Option<&str>,
    stream: &str,
    raw_text: &str,
) -> KgwRuntimeRawLogEntryV1 {
    KgwRuntimeRawLogEntryV1 {
        sequence,
        network: network.to_string(),
        source: "self-worker".to_string(),
        runtime_role: runtime_role.to_string(),
        bridge_instance_id: bridge_instance_id.map(str::to_string),
        stream: stream.to_string(),
        received_ms: 1,
        raw_text: raw_text.to_string(),
    }
}

''',
    "unused raw-log entry test helper",
)
text = delete_once_or_done(
    text,
    '''#[cfg(test)]
pub(crate) fn kgw_raw_log_text_from_entries_for_test_v1(
    mut entries: Vec<KgwRuntimeRawLogEntryV1>,
) -> String {
    entries.sort_by_key(|entry| entry.sequence);
    entries
        .into_iter()
        .map(|entry| entry.raw_text)
        .collect::<Vec<_>>()
        .join("\\n")
}

''',
    "unused raw-log text test helper",
)
text = delete_once_or_done(
    text,
    '''#[cfg(test)]
pub(crate) fn kgw_worker_node_command_args_for_test_v1(
    network: &str,
    appdir: &str,
) -> Vec<String> {
    kgw_worker_node_command_args(network, appdir)
}

''',
    "unused worker command test helper",
)
text = replace_or_done(
    text,
    '''#[tauri::command]
pub fn kgw_kgw_apply_node_settings_v1(
    network: String,
    node_kind: String,
    bridge_kind: String,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
    runtime_role: Option<String>,
    bridge_active_instance_id: Option<String>,
    bridge_active_instance: Option<String>,
    bridge_active_instance_port: Option<String>,
    bridge_structured_instances: Option<String>,
    experimental_network_opt_in: Option<bool>,
) -> Result<String, String> {
''',
    '''#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KgwApplyNodeSettingsRequestV1 {
    network: String,
    node_kind: String,
    bridge_kind: String,
    node_command_preview: Option<String>,
    bridge_command_preview: Option<String>,
    runtime_role: Option<String>,
    bridge_active_instance_id: Option<String>,
    bridge_active_instance: Option<String>,
    bridge_active_instance_port: Option<String>,
    bridge_structured_instances: Option<String>,
    experimental_network_opt_in: Option<bool>,
}

#[tauri::command]
pub fn kgw_kgw_apply_node_settings_v1(request: KgwApplyNodeSettingsRequestV1) -> Result<String, String> {
    let KgwApplyNodeSettingsRequestV1 {
        network,
        node_kind,
        bridge_kind,
        node_command_preview,
        bridge_command_preview,
        runtime_role,
        bridge_active_instance_id,
        bridge_active_instance,
        bridge_active_instance_port,
        bridge_structured_instances,
        experimental_network_opt_in,
    } = request;
''',
    1,
    "node-settings request boundary",
)
integrated.write_text(text, encoding="utf-8")

lib = Path("apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
text = lib.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''        let Some(dir) = kgw_ui_trace_log_dir_v37() else {
            return None;
        };''',
    '''        let dir = kgw_ui_trace_log_dir_v37()?;''',
    1,
    "question_mark ui trace directory",
)
text = replace_or_done(
    text,
    '''fn kgw_copy_text_to_clipboard_inner_v1<F>(
    network: String,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    text: String,
    metadata_character_count: u64,
    metadata_line_count: u64,
    metadata_sha256: Option<String>,
    writer: F,
) -> Result<String, String>
where
    F: FnOnce(String) -> Result<(), String>,
{
''',
    '''#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KgwClipboardWriteRequestV1 {
    network: String,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    text: String,
    character_count: u64,
    line_count: u64,
    sha256: Option<String>,
}

fn kgw_copy_text_to_clipboard_inner_v1<F>(
    request: KgwClipboardWriteRequestV1,
    writer: F,
) -> Result<String, String>
where
    F: FnOnce(String) -> Result<(), String>,
{
    let KgwClipboardWriteRequestV1 {
        network,
        runtime_role,
        bridge_instance_id,
        text,
        character_count: metadata_character_count,
        line_count: metadata_line_count,
        sha256: metadata_sha256,
    } = request;
''',
    1,
    "clipboard request boundary",
)
text = replace_or_done(
    text,
    '''#[tauri::command]
fn kgw_copy_text_to_clipboard_v1(
    app: tauri::AppHandle,
    network: String,
    runtime_role: Option<String>,
    bridge_instance_id: Option<String>,
    text: String,
    character_count: u64,
    line_count: u64,
    sha256: Option<String>,
) -> Result<String, String> {
    kgw_copy_text_to_clipboard_inner_v1(
        network,
        runtime_role,
        bridge_instance_id,
        text,
        character_count,
        line_count,
        sha256,
        |value| {
''',
    '''#[tauri::command]
fn kgw_copy_text_to_clipboard_v1(
    app: tauri::AppHandle,
    request: KgwClipboardWriteRequestV1,
) -> Result<String, String> {
    kgw_copy_text_to_clipboard_inner_v1(
        request,
        |value| {
''',
    1,
    "clipboard Tauri command boundary",
)
text = replace_or_done(
    text,
    '''        let result = kgw_copy_text_to_clipboard_inner_v1(
            "mainnet".to_string(),
            Some("node".to_string()),
            None,
            text.clone(),
            char_count(&text),
            2,
            Some(sha256.clone()),
            move |value| {''',
    '''        let result = kgw_copy_text_to_clipboard_inner_v1(
            KgwClipboardWriteRequestV1 {
                network: "mainnet".to_string(),
                runtime_role: Some("node".to_string()),
                bridge_instance_id: None,
                text: text.clone(),
                character_count: char_count(&text),
                line_count: 2,
                sha256: Some(sha256.clone()),
            },
            move |value| {''',
    1,
    "clipboard success test request",
)
text = replace_or_done(
    text,
    '''        let error = kgw_copy_text_to_clipboard_inner_v1(
            "testnet10".to_string(),
            Some("node".to_string()),
            None,
            text.clone(),
            char_count(&text),
            1,
            Some(kgw_clipboard_sha256_v1(&text)),
            |_value| Err("native clipboard permission denied".to_string()),''',
    '''        let error = kgw_copy_text_to_clipboard_inner_v1(
            KgwClipboardWriteRequestV1 {
                network: "testnet10".to_string(),
                runtime_role: Some("node".to_string()),
                bridge_instance_id: None,
                text: text.clone(),
                character_count: char_count(&text),
                line_count: 1,
                sha256: Some(kgw_clipboard_sha256_v1(&text)),
            },
            |_value| Err("native clipboard permission denied".to_string()),''',
    1,
    "clipboard failure test request",
)
text = replace_or_done(
    text,
    '''        let error = kgw_copy_text_to_clipboard_inner_v1(
            "mainnet".to_string(),
            Some("node".to_string()),
            None,
            "   \\r\\n  ".to_string(),
            7,
            2,
            None,
            |_value| panic!("empty content must not reach the clipboard writer"),''',
    '''        let error = kgw_copy_text_to_clipboard_inner_v1(
            KgwClipboardWriteRequestV1 {
                network: "mainnet".to_string(),
                runtime_role: Some("node".to_string()),
                bridge_instance_id: None,
                text: "   \\r\\n  ".to_string(),
                character_count: 7,
                line_count: 2,
                sha256: None,
            },
            |_value| panic!("empty content must not reach the clipboard writer"),''',
    1,
    "clipboard empty test request",
)
text = replace_or_done(
    text,
    '''        let result = kgw_copy_text_to_clipboard_inner_v1(
            "mainnet".to_string(),
            Some("bridge".to_string()),
            Some("bridge-a".to_string()),
            text.clone(),
            char_count(&text),
            1,
            Some(sha256.clone()),
            |_value| Ok(()),''',
    '''        let result = kgw_copy_text_to_clipboard_inner_v1(
            KgwClipboardWriteRequestV1 {
                network: "mainnet".to_string(),
                runtime_role: Some("bridge".to_string()),
                bridge_instance_id: Some("bridge-a".to_string()),
                text: text.clone(),
                character_count: char_count(&text),
                line_count: 1,
                sha256: Some(sha256.clone()),
            },
            |_value| Ok(()),''',
    1,
    "clipboard trace test request",
)
lib.write_text(text, encoding="utf-8")

export = Path("apps/kaspa-gateway-desktop/src-tauri/src/export_commands.rs")
text = export.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''fn kgw_pdf_draw_table_row_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    headers: &[String],
    row: &[String],
    widths: &[f32],
    x0: f32,
    y: f32,
    row_h: f32,
) {
    let mut x = x0;''',
    '''#[derive(Debug, Clone, Copy)]
struct KgwPdfTableRowLayoutV29<'a> {
    widths: &'a [f32],
    x0: f32,
    y: f32,
    row_h: f32,
}

fn kgw_pdf_draw_table_row_v29(
    ops: &mut Vec<printpdf::Op>,
    font: &printpdf::PdfFontHandle,
    headers: &[String],
    row: &[String],
    layout: KgwPdfTableRowLayoutV29<'_>,
) {
    let KgwPdfTableRowLayoutV29 {
        widths,
        x0,
        y,
        row_h,
    } = layout;
    let mut x = x0;''',
    1,
    "PDF row layout boundary",
)
text = replace_or_done(
    text,
    '''            kgw_pdf_draw_table_row_v29(&mut ops, &font, &table.headers, &row, &widths, x0, y, 8.0);''',
    '''            kgw_pdf_draw_table_row_v29(
                &mut ops,
                &font,
                &table.headers,
                &row,
                KgwPdfTableRowLayoutV29 {
                    widths: &widths,
                    x0,
                    y,
                    row_h: 8.0,
                },
            );''',
    1,
    "PDF empty-row layout call",
)
text = replace_or_done(
    text,
    '''                    kgw_pdf_draw_table_row_v29(
                        &mut ops,
                        &font,
                        &table.headers,
                        row,
                        &widths,
                        x0,
                        y,
                        compact_h,
                    );''',
    '''                    kgw_pdf_draw_table_row_v29(
                        &mut ops,
                        &font,
                        &table.headers,
                        row,
                        KgwPdfTableRowLayoutV29 {
                            widths: &widths,
                            x0,
                            y,
                            row_h: compact_h,
                        },
                    );''',
    1,
    "PDF regular-row layout call",
)
export.write_text(text, encoding="utf-8")

node_ui = Path("apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js")
text = node_ui.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''    return {
      network: net,
      nodeKind: "integrated-as-daemon",
      bridgeKind: "disable",
      nodeCommandPreview: preview,
      bridgeCommandPreview: "",
      runtimeRole: "node",
      experimentalNetworkOptIn: net === "testnet12" && kgwNodeNetworkEnabled(net),
    };''',
    '''    return {
      request: {
        network: net,
        nodeKind: "integrated-as-daemon",
        bridgeKind: "disable",
        nodeCommandPreview: preview,
        bridgeCommandPreview: "",
        runtimeRole: "node",
        experimentalNetworkOptIn: net === "testnet12" && kgwNodeNetworkEnabled(net),
      }
    };''',
    1,
    "node settings IPC request",
)
text = replace_or_done(
    text,
    '''    {
      network: net,
      runtimeRole: metadata.runtimeRole || "node",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      text,
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || ""
    },''',
    '''    {
      request: {
        network: net,
        runtimeRole: metadata.runtimeRole || "node",
        bridgeInstanceId: metadata.bridgeInstanceId || "",
        text,
        characterCount: metadata.characterCount,
        lineCount: metadata.lineCount,
        sha256: metadata.sha256 || ""
      }
    },''',
    1,
    "node clipboard IPC request",
)
node_ui.write_text(text, encoding="utf-8")

bridge_ui = Path("apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js")
text = bridge_ui.read_text(encoding="utf-8")
text = replace_or_done(
    text,
    '''    return {
      network: net,
      runtimeRole: "bridge",
      nodeKind: nodeMode === "inprocess" ? "integrated-inproc" : "remote",
      bridgeKind: nodeMode === "inprocess" ? "official-inprocess-node" : "official-external-node",
      nodeCommandPreview: "",
      bridgeCommandPreview: preview,
      bridgeActiveInstanceId,
      bridgeActiveInstance,
      bridgeActiveInstancePort,
      bridgeStructuredInstances: JSON.stringify(structuredInstances || {}),
      experimentalNetworkOptIn: net === "testnet12" && kgwBridgeNetworkEnabled(net),
    };''',
    '''    return {
      request: {
        network: net,
        runtimeRole: "bridge",
        nodeKind: nodeMode === "inprocess" ? "integrated-inproc" : "remote",
        bridgeKind: nodeMode === "inprocess" ? "official-inprocess-node" : "official-external-node",
        nodeCommandPreview: "",
        bridgeCommandPreview: preview,
        bridgeActiveInstanceId,
        bridgeActiveInstance,
        bridgeActiveInstancePort,
        bridgeStructuredInstances: JSON.stringify(structuredInstances || {}),
        experimentalNetworkOptIn: net === "testnet12" && kgwBridgeNetworkEnabled(net),
      }
    };''',
    1,
    "bridge settings IPC request",
)
text = replace_or_done(
    text,
    '''    {
      network: net,
      runtimeRole: metadata.runtimeRole || "bridge",
      bridgeInstanceId: metadata.bridgeInstanceId || "",
      text,
      characterCount: metadata.characterCount,
      lineCount: metadata.lineCount,
      sha256: metadata.sha256 || ""
    },''',
    '''    {
      request: {
        network: net,
        runtimeRole: metadata.runtimeRole || "bridge",
        bridgeInstanceId: metadata.bridgeInstanceId || "",
        text,
        characterCount: metadata.characterCount,
        lineCount: metadata.lineCount,
        sha256: metadata.sha256 || ""
      }
    },''',
    1,
    "bridge clipboard IPC request",
)
bridge_ui.write_text(text, encoding="utf-8")
