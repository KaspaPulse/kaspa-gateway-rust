from pathlib import Path


def replace_exact(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise RuntimeError(f"{label}: expected {expected} matches, found {count}")
    return text.replace(old, new)


db = Path("crates/kaspa-gateway-db/src/lib.rs")
text = db.read_text(encoding="utf-8")
text = replace_exact(
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
text = replace_exact(
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
text = replace_exact(
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
text = replace_exact(
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
)
db.write_text(text, encoding="utf-8")

cfg = Path("crates/kaspa-gateway-config/src/lib.rs")
text = cfg.read_text(encoding="utf-8")
text = replace_exact(
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
text = replace_exact(
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
node_matches: list[Path] = []
for root in (Path("crates"), Path("apps")):
    if not root.exists():
        continue
    for path in root.rglob("*.rs"):
        source = path.read_text(encoding="utf-8")
        if node_old in source:
            node_matches.append(path)

if len(node_matches) != 1:
    raise RuntimeError(
        "node_rpc collapsible_if: expected exactly one source match, "
        f"found {len(node_matches)}: {node_matches}"
    )

node_path = node_matches[0]
node_text = node_path.read_text(encoding="utf-8")
node_text = replace_exact(
    node_text,
    node_old,
    node_new,
    1,
    f"node_rpc collapsible_if in {node_path}",
)
node_path.write_text(node_text, encoding="utf-8")
print(f"Patched remaining node_rpc Clippy finding in {node_path}")
