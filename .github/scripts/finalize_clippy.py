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
if rk_owner.exists():
    text = rk_owner.read_text(encoding="utf-8")
    text = replace_or_done(
        text,
        '''#[allow(dead_code)]

fn ''',
        '''#[allow(dead_code)]
fn ''',
        5,
        "empty_line_after_outer_attr",
        optional=True,
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
        optional=True,
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
        optional=True,
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
old_matches = []
new_matches = []
for root in (Path("crates"), Path("apps")):
    if root.exists():
        for path in root.rglob("*.rs"):
            source = path.read_text(encoding="utf-8")
            if node_old in source:
                old_matches.append(path)
            if node_new in source:
                new_matches.append(path)
if len(old_matches) == 1 and not new_matches:
    path = old_matches[0]
    source = path.read_text(encoding="utf-8")
    path.write_text(source.replace(node_old, node_new, 1), encoding="utf-8")
    print(f"node_rpc collapsible_if: repaired {path}")
elif not old_matches and len(new_matches) <= 1:
    print("node_rpc collapsible_if: already repaired or not present")
else:
    raise RuntimeError(f"node_rpc collapsible_if: ambiguous state old={old_matches} new={new_matches}")

lib = Path("apps/kaspa-gateway-desktop/src-tauri/src/lib.rs")
if lib.exists():
    text = lib.read_text(encoding="utf-8")
    text = replace_or_done(
        text,
        '''        let Some(dir) = kgw_ui_trace_log_dir_v37() else {
            return None;
        };''',
        '''        let dir = kgw_ui_trace_log_dir_v37()?;''',
        1,
        "question_mark ui trace directory",
        optional=True,
    )
    lib.write_text(text, encoding="utf-8")
