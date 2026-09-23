use regex::Regex;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const FRONTEND: &str = "apps/kaspa-gateway-desktop/frontend";

const CRITICAL_KEYS: &[&str] = &[
    "settings.language",
    "settings.currency",
    "settings.theme",
    "tabs.explorer",
    "tabs.analysis",
    "tabs.log",
    "tabs.settings",
    "status.ready",
    "runtime.ready",
    "common.loading",
    "metrics.price",
    "metrics.hashrate",
    "metrics.difficulty",
    "common.force.fetch",
    "actions.fetch",
    "actions.cancel",
    "actions.export.results",
    "ui.analysis.reset.filter",
    "ui.analysis.save.as.csv",
    "ui.analysis.save.as.html",
    "ui.analysis.save.as.pdf",
    "ui.explorer.select.saved.address",
    "ui.explorer.balance",
    "ui.explorer.date.time.sort",
    "ui.explorer.transaction.id.sort",
    "ui.explorer.direction.sort",
    "ui.explorer.amount.kas.sort",
    "ui.explorer.value.usd.sort",
    "ui.explorer.type.sort",
    "ui.explorer.transaction.table.font.size",
    "explorer.noTransactionsToDisplay",
    "links.donations",
];

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("i18n read failed {}: {error}", path.display()))
}

fn flatten_dictionary(value: &Value) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    flatten_into(value, "", &mut out);
    out
}

fn flatten_into(value: &Value, prefix: &str, out: &mut BTreeMap<String, Value>) {
    let Some(object) = value.as_object() else {
        if !prefix.is_empty() {
            out.insert(prefix.to_owned(), value.clone());
        }
        return;
    };
    for (key, value) in object {
        if key.contains('.') {
            if value.is_object() {
                flatten_into(value, key, out);
            } else {
                out.insert(key.clone(), value.clone());
            }
            continue;
        }
        let next = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        if value.is_object() {
            flatten_into(value, &next, out);
        } else {
            out.insert(next, value.clone());
        }
    }
}
fn language_dictionaries(root: &Path) -> Result<BTreeMap<String, BTreeMap<String, Value>>, String> {
    let dir = root.join(FRONTEND).join("i18n");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .map_err(|error| format!("Missing i18n dir {}: {error}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                && !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("map.json" | "lang_map.json")
                )
        })
        .collect();
    paths.sort();
    let mut dictionaries = BTreeMap::new();
    for path in paths {
        let lang = path.file_stem().unwrap().to_string_lossy().into_owned();
        let value: Value = serde_json::from_str(&read(&path)?)
            .map_err(|error| format!("invalid i18n JSON {}: {error}", path.display()))?;
        dictionaries.insert(lang, flatten_dictionary(&value));
    }
    Ok(dictionaries)
}

fn approved_same_as_english(lang: &str, key: &str, value: &str) -> bool {
    lang == "de" && key == "tabs.explorer" && normalize_text(value) == "Explorer"
}

#[derive(Debug)]
struct LocaleReport {
    missing: Vec<Value>,
    same_as_english: Vec<Value>,
    approved: Vec<Value>,
}

fn locale_report(root: &Path) -> Result<LocaleReport, String> {
    let dictionaries = language_dictionaries(root)?;
    let mut missing = Vec::new();
    let mut same_as_english = Vec::new();
    let mut approved = Vec::new();
    for key in CRITICAL_KEYS {
        let english = dictionaries
            .get("en")
            .and_then(|dict| dict.get(*key))
            .and_then(Value::as_str);
        for (lang, dict) in &dictionaries {
            let value = dict.get(*key).and_then(Value::as_str);
            let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
                missing.push(json!({"lang": lang, "key": key}));
                continue;
            };
            if lang != "en" && english.is_some_and(|en| normalize_text(value) == normalize_text(en))
            {
                if approved_same_as_english(lang, key, value) {
                    approved.push(json!({"lang": lang, "key": key, "value": value}));
                } else {
                    same_as_english.push(json!({"lang": lang, "key": key, "value": value}));
                }
            }
        }
    }
    Ok(LocaleReport {
        missing,
        same_as_english,
        approved,
    })
}

pub fn locale_coverage(root: &Path) -> Result<String, String> {
    let report = locale_report(root)?;
    if !report.missing.is_empty() || !report.same_as_english.is_empty() {
        return Err(format!(
            "KGW i18n locale coverage gate FAILED\n{}",
            serde_json::to_string_pretty(&json!({
                "missing": report.missing,
                "sameAsEnglish": report.same_as_english,
                "approvedSameAsEnglish": report.approved,
            }))
            .unwrap()
        ));
    }
    Ok(format!(
        "KGW i18n locale coverage gate\ncriticalKeys: {}\nmissing: 0\nsameAsEnglish: 0\napprovedSameAsEnglish: {}",
        CRITICAL_KEYS.len(),
        report.approved.len()
    ))
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct HtmlFinding {
    file: String,
    tag: String,
    text: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LiteralFinding {
    file: String,
    kind: String,
    literal: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct QuoteRisk {
    file: String,
    line: usize,
    text: String,
}

#[derive(Debug)]
struct ContractReport {
    refs: BTreeSet<String>,
    missing_refs: Vec<Value>,
    unbound_html_text: Vec<HtmlFinding>,
    dynamic_literals: Vec<LiteralFinding>,
    quote_risks: Vec<QuoteRisk>,
    runtime_findings: Vec<String>,
}

fn list_files_recursive(root: &Path, extension: &str) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<_> = fs::read_dir(&dir)
            .map_err(|error| format!("i18n directory read failed {}: {error}", dir.display()))?
            .filter_map(Result::ok)
            .collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(extension))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn relative_frontend_path(frontend: &Path, path: &Path) -> String {
    path.strip_prefix(frontend)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn strip_blocks(html: &str) -> String {
    let mut clean = html.to_owned();
    for pattern in [
        r"(?is)<script\b.*?</script>",
        r"(?is)<style\b.*?</style>",
        r"(?is)<svg\b.*?</svg>",
        r"(?is)<template\b.*?</template>",
    ] {
        clean = Regex::new(pattern)
            .unwrap()
            .replace_all(&clean, "")
            .into_owned();
    }
    clean
}

fn is_structural_html_fragment(value: &str) -> bool {
    let text = normalize_text(value);
    if text.is_empty() {
        return true;
    }
    let structural_start =
        Regex::new(r"(?i)^<\s*(div|span|pre|tr|td|th|option|tbody|thead|table)\b").unwrap();
    let structural_end =
        Regex::new(r"(?i)^<\s*/\s*(div|span|pre|tr|td|th|option|tbody|thead|table)\s*>").unwrap();
    structural_start.is_match(&text)
        || structural_end.is_match(&text)
        || Regex::new(r"^<[^>]+=$").unwrap().is_match(&text)
        || Regex::new(r"^<[^>]+\s*$").unwrap().is_match(&text)
}

fn is_likely_user_text(value: &str) -> bool {
    let text = normalize_text(value);
    if text.is_empty() || is_structural_html_fragment(&text) || text.len() > 220 {
        return false;
    }
    if matches!(text.as_str(), "N/A" | "—" | "×" | "i") {
        return false;
    }
    if Regex::new(r"^[0-9.,:%()\-+]+$").unwrap().is_match(&text)
        || Regex::new(r"^\{\{.*\}\}$").unwrap().is_match(&text)
        || Regex::new(r"(?i)^https?://").unwrap().is_match(&text)
    {
        return false;
    }
    for forbidden in [
        "${",
        "=>",
        "querySelector",
        "addEventListener",
        "pointer-events:",
        "z-index:",
        "window.kgwT",
    ] {
        if text.contains(forbidden) {
            return false;
        }
    }
    Regex::new(r"[A-Za-z\x{0600}-\x{06FF}]")
        .unwrap()
        .is_match(&text)
}

fn extract_html_refs(html: &str) -> Vec<String> {
    let clean = strip_blocks(html);
    let re = Regex::new(
        r#"\b(?:data-i18n|data-i18n-title|data-i18n-placeholder|data-i18n-aria-label)=["']([^"']+)["']"#,
    )
    .unwrap();
    re.captures_iter(&clean)
        .filter_map(|capture| capture.get(1))
        .map(|value| normalize_text(value.as_str()))
        .filter(|value| !value.is_empty())
        .collect()
}

fn extract_js_refs(js: &str) -> Vec<String> {
    let patterns = [
        r#"\bkgwT\s*\(\s*["']([^"']+)["']\s*\)"#,
        r#"\bkgwI18n\s*\(\s*["']([^"']+)["']\s*\)"#,
        r#"\bt\s*\(\s*["']([^"']+)["']\s*\)"#,
        r#"\bdataset\.i18n\s*=\s*["']([^"']+)["']"#,
        r#"\bsetAttribute\(\s*["']data-i18n["']\s*,\s*["']([^"']+)["']\s*\)"#,
        r#"\bdata-i18n=["']([^"']+)["']"#,
        r#"\bdata-i18n=\\"([^"\\]+)\\""#,
    ];
    let mut refs = Vec::new();
    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        for capture in re.captures_iter(js) {
            let Some(found) = capture.get(1) else {
                continue;
            };
            let key = normalize_text(found.as_str());
            if !key.is_empty() && !key.contains(' ') {
                refs.push(key);
            }
        }
    }
    refs
}

fn extract_unbound_html_text(html: &str, relative: &str) -> Vec<HtmlFinding> {
    let clean = strip_blocks(html);
    let re = Regex::new(
        r#"(?is)<([a-zA-Z][a-zA-Z0-9:-]*)([^>]*)>\s*([^<>{}]*[A-Za-z\x{0600}-\x{06FF}][^<>{}]*)\s*</([a-zA-Z][a-zA-Z0-9:-]*)>"#,
    )
    .unwrap();
    let mut findings = Vec::new();
    for capture in re.captures_iter(&clean) {
        let tag = capture.get(1).unwrap().as_str().to_ascii_lowercase();
        let close = capture.get(4).unwrap().as_str().to_ascii_lowercase();
        if tag != close || matches!(tag.as_str(), "script" | "style" | "svg" | "template") {
            continue;
        }
        let attrs = capture.get(2).map_or("", |value| value.as_str());
        let text = normalize_text(capture.get(3).map_or("", |value| value.as_str()));
        if !is_likely_user_text(&text)
            || attrs.contains("data-i18n")
            || attrs.contains("data-kgw-no-i18n")
        {
            continue;
        }
        findings.push(HtmlFinding {
            file: relative.to_owned(),
            tag,
            text,
        });
    }
    findings
}

fn extract_dynamic_literal_findings(js: &str, relative: &str) -> Vec<LiteralFinding> {
    let patterns = [
        (
            "textContent",
            r#"\.(?:textContent|innerText)\s*=\s*["'\x60]([^"'\x60]*[A-Za-z\x{0600}-\x{06FF}][^"'\x60]*)["'\x60]"#,
        ),
        (
            "innerHTML",
            r#"\.innerHTML\s*=\s*["'\x60]([^"'\x60]*[A-Za-z\x{0600}-\x{06FF}][^"'\x60]*)["'\x60]"#,
        ),
        (
            "insertAdjacentHTML",
            r#"\.insertAdjacentHTML\s*\(\s*["'\x60][^"'\x60]*["'\x60]\s*,\s*["'\x60]([^"'\x60]*[A-Za-z\x{0600}-\x{06FF}][^"'\x60]*)["'\x60]\s*\)"#,
        ),
        (
            "newOption",
            r#"new\s+Option\s*\(\s*["'\x60]([^"'\x60]*[A-Za-z\x{0600}-\x{06FF}][^"'\x60]*)["'\x60]"#,
        ),
    ];
    let mut findings = Vec::new();
    for (kind, pattern) in patterns {
        let re = Regex::new(pattern).unwrap();
        for capture in re.captures_iter(js) {
            let literal = normalize_text(capture.get(1).map_or("", |value| value.as_str()));
            if !is_likely_user_text(&literal) || literal.contains("data-i18n=") {
                continue;
            }
            findings.push(LiteralFinding {
                file: relative.to_owned(),
                kind: kind.to_owned(),
                literal,
            });
        }
    }
    findings
}

fn extract_quote_risks(js: &str, relative: &str) -> Vec<QuoteRisk> {
    let mut findings = Vec::new();
    for (index, line) in js.lines().enumerate() {
        let Some(data_index) = line.find("data-i18n=\"") else {
            continue;
        };
        if line.contains('\x60') {
            continue;
        }
        let quote_count_before = line[..data_index]
            .bytes()
            .filter(|byte| *byte == b'"')
            .count();
        if quote_count_before % 2 == 1 {
            findings.push(QuoteRisk {
                file: relative.to_owned(),
                line: index + 1,
                text: line.trim().to_owned(),
            });
        }
    }
    findings
}

fn contract_report(root: &Path) -> Result<ContractReport, String> {
    let frontend = root.join(FRONTEND);
    let i18n_dir = frontend.join("i18n");
    if !frontend.is_dir() {
        return Err(format!("Missing frontend root: {}", frontend.display()));
    }
    if !i18n_dir.is_dir() {
        return Err(format!("Missing i18n dir: {}", i18n_dir.display()));
    }

    let dictionaries = language_dictionaries(root)?;
    let html_files = list_files_recursive(&frontend, "html")?;
    let js_files = list_files_recursive(&frontend, "js")?;

    let mut refs = BTreeSet::new();
    let mut unbound_html_text = Vec::new();
    let mut dynamic_literals = Vec::new();
    let mut quote_risks = Vec::new();

    for path in html_files {
        let relative = relative_frontend_path(&frontend, &path);
        let html = read(&path)?;
        refs.extend(extract_html_refs(&html));
        unbound_html_text.extend(extract_unbound_html_text(&html, &relative));
    }
    for path in js_files {
        let relative = relative_frontend_path(&frontend, &path);
        let js = read(&path)?;
        refs.extend(extract_js_refs(&js));
        dynamic_literals.extend(extract_dynamic_literal_findings(&js, &relative));
        quote_risks.extend(extract_quote_risks(&js, &relative));
    }

    let mut missing_refs = Vec::new();
    for (lang, dictionary) in &dictionaries {
        for key in &refs {
            let valid = dictionary
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty());
            if !valid {
                missing_refs.push(json!({"lang": lang, "key": key}));
            }
        }
    }

    let main_js = read(&frontend.join("main.js"))?;
    let mut runtime_findings = Vec::new();
    for marker in [
        "KGW_R99_CANONICAL_I18N_BIND_APPLY_HELPER",
        "KGW_R100_FLATTEN_I18N_DICTIONARY",
        "KGW_R102_DYNAMIC_DOM_I18N_REAPPLY",
        "KGW_R107_CANONICAL_TRANSLATION_RUNTIME_API",
        "window.kgwT = function kgwTranslateRuntimeR107",
        "window.kgwI18n = window.kgwT",
        "window.__kgwI18nDictR107 = dict;",
        "kgw:tab-opened",
        "tab-opened-after-mount",
        "flattenKgwI18nDictionaryR100",
        "bindMissingI18nAttributesR99",
        "updateDynamicKgwI18nRuntimeR102",
    ] {
        if !main_js.contains(marker) {
            runtime_findings.push(format!("missing-main-marker:{marker}"));
        }
    }

    Ok(ContractReport {
        refs,
        missing_refs,
        unbound_html_text,
        dynamic_literals,
        quote_risks,
        runtime_findings,
    })
}

fn report_json(report: &ContractReport) -> Value {
    json!({
        "missingRefs": report.missing_refs,
        "unboundHtmlText": report
            .unbound_html_text
            .iter()
            .map(|item| json!({"file": item.file, "tag": item.tag, "text": item.text}))
            .collect::<Vec<_>>(),
        "dynamicLiterals": report
            .dynamic_literals
            .iter()
            .map(|item| json!({"file": item.file, "kind": item.kind, "literal": item.literal}))
            .collect::<Vec<_>>(),
        "quoteRisks": report
            .quote_risks
            .iter()
            .map(|item| json!({"file": item.file, "line": item.line, "text": item.text}))
            .collect::<Vec<_>>(),
        "runtimeFindings": report.runtime_findings,
    })
}

pub fn contract(root: &Path) -> Result<String, String> {
    let report = contract_report(root)?;
    let summary = format!(
        "KGW i18n contract gate\nrefs: {}\nmissingRefs: {}\nunboundHtmlText: {}\ndynamicLiterals: {}\nquoteRisks: {}\nruntimeFindings: {}",
        report.refs.len(),
        report.missing_refs.len(),
        report.unbound_html_text.len(),
        report.dynamic_literals.len(),
        report.quote_risks.len(),
        report.runtime_findings.len()
    );

    let mut blockers = Vec::new();
    for (name, count) in [
        ("missingRefs", report.missing_refs.len()),
        ("unboundHtmlText", report.unbound_html_text.len()),
        ("dynamicLiterals", report.dynamic_literals.len()),
        ("quoteRisks", report.quote_risks.len()),
        ("runtimeFindings", report.runtime_findings.len()),
    ] {
        if count > 0 {
            blockers.push(format!("{name}={count}"));
        }
    }

    if blockers.is_empty() {
        Ok(summary)
    } else {
        Err(format!(
            "{summary}\n{}\ni18n contract gate failed: {}",
            serde_json::to_string_pretty(&report_json(&report)).unwrap(),
            blockers.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flatten_dictionary_preserves_nested_and_dotted_keys() {
        let value = json!({
            "settings": {"language": "Language"},
            "tabs.explorer": "Explorer",
            "ui": {"analysis.reset.filter": "Reset"}
        });
        let flat = flatten_dictionary(&value);
        assert_eq!(
            flat.get("settings.language").and_then(Value::as_str),
            Some("Language")
        );
        assert_eq!(
            flat.get("tabs.explorer").and_then(Value::as_str),
            Some("Explorer")
        );
        assert_eq!(
            flat.get("analysis.reset.filter").and_then(Value::as_str),
            Some("Reset")
        );
    }

    #[test]
    fn approved_same_as_english_is_narrow() {
        assert!(approved_same_as_english("de", "tabs.explorer", "Explorer"));
        assert!(!approved_same_as_english("fr", "tabs.explorer", "Explorer"));
        assert!(!approved_same_as_english("de", "tabs.settings", "Explorer"));
    }

    #[test]
    fn html_refs_ignore_script_blocks_and_capture_attributes() {
        let html = r#"
            <span data-i18n="tabs.settings">Settings</span>
            <input data-i18n-placeholder='common.loading'>
            <script><span data-i18n="ignored.script">x</span></script>
        "#;
        let refs = extract_html_refs(html);
        assert_eq!(refs, vec!["tabs.settings", "common.loading"]);
    }

    #[test]
    fn unbound_html_text_is_fail_closed_but_respects_opt_outs() {
        let html = r#"
            <span>Application:</span>
            <div data-i18n="status.ready">Ready.</div>
            <p data-kgw-no-i18n>Brand Name</p>
        "#;
        let findings = extract_unbound_html_text(html, "index.html");
        assert_eq!(
            findings,
            vec![HtmlFinding {
                file: "index.html".to_owned(),
                tag: "span".to_owned(),
                text: "Application:".to_owned(),
            }]
        );
    }

    #[test]
    fn javascript_reference_extractors_cover_supported_forms() {
        let js = r#"
            kgwT("tabs.settings");
            kgwI18n('status.ready');
            t("common.loading");
            node.dataset.i18n = "actions.fetch";
            node.setAttribute("data-i18n", "actions.cancel");
            const html = '<span data-i18n="tabs.analysis"></span>';
            const escaped = "data-i18n=\"tabs.log\"";
        "#;
        let refs: BTreeSet<_> = extract_js_refs(js).into_iter().collect();
        for expected in [
            "tabs.settings",
            "status.ready",
            "common.loading",
            "actions.fetch",
            "actions.cancel",
            "tabs.analysis",
            "tabs.log",
        ] {
            assert!(refs.contains(expected), "{expected}");
        }
    }

    #[test]
    fn dynamic_literal_extractor_matches_user_visible_assignments_only() {
        let js = r#"
            title.textContent = "Reconciling";
            node.innerText = "Node:";
            panel.innerHTML = "<span>Restore uses</span>";
            other.textContent = "https://example.invalid";
            code.textContent = "window.kgwT";
        "#;
        let findings = extract_dynamic_literal_findings(js, "fixture.js");
        let literals: Vec<_> = findings.iter().map(|item| item.literal.as_str()).collect();
        assert!(literals.contains(&"Reconciling"));
        assert!(literals.contains(&"Node:"));
        assert!(!literals.contains(&"<span>Restore uses</span>"));
        assert!(!literals.contains(&"https://example.invalid"));
        assert!(!literals.contains(&"window.kgwT"));
    }

    #[test]
    fn quote_risk_detects_double_quote_embedding_without_template_literal() {
        let risky = r#"const html = "<span data-i18n="tabs.settings">";"#;
        let safe = "const html = \x60<span data-i18n=\"tabs.settings\">\x60;";
        assert_eq!(extract_quote_risks(risky, "a.js").len(), 1);
        assert!(extract_quote_risks(safe, "b.js").is_empty());
    }

    #[test]
    fn normalize_text_matches_legacy_whitespace_collapse() {
        assert_eq!(normalize_text("  Hello \n\t world  "), "Hello world");
    }
}
