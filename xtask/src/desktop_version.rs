use serde_json::Value;
use std::fs;
use std::path::Path;

const PACKAGE_JSON: &str = "apps/kaspa-gateway-desktop/package.json";
const CARGO_TOML: &str = "apps/kaspa-gateway-desktop/src-tauri/Cargo.toml";
const TAURI_CONFIG: &str = "apps/kaspa-gateway-desktop/src-tauri/tauri.conf.json";
const INDEX_HTML: &str = "apps/kaspa-gateway-desktop/frontend/index.html";
const SHELL_RUNTIME_RS: &str = "crates/kaspa-gateway-frontend-wasm/src/shell_runtime.rs";
const DIAGNOSTICS_RS: &str = "apps/kaspa-gateway-desktop/src-tauri/src/diagnostics.rs";
const LIB_RS: &str = "apps/kaspa-gateway-desktop/src-tauri/src/lib.rs";
const LOCALES: &[&str] = &[
    "ar", "de", "en", "es", "fr", "hi", "id", "ja", "ko", "ru", "tr", "zh-CN",
];

struct Inputs {
    package_json: String,
    cargo_toml: String,
    tauri_config: String,
    index_html: String,
    shell_runtime_rs: String,
    diagnostics_rs: String,
    lib_rs: String,
    locales: Vec<(String, String)>,
}

pub fn run(root: &Path) -> Result<String, String> {
    let inputs = Inputs {
        package_json: read(root, PACKAGE_JSON)?,
        cargo_toml: read(root, CARGO_TOML)?,
        tauri_config: read(root, TAURI_CONFIG)?,
        index_html: read(root, INDEX_HTML)?,
        shell_runtime_rs: read(root, SHELL_RUNTIME_RS)?,
        diagnostics_rs: read(root, DIAGNOSTICS_RS)?,
        lib_rs: read(root, LIB_RS)?,
        locales: LOCALES
            .iter()
            .map(|name| {
                let relative = format!("apps/kaspa-gateway-desktop/frontend/i18n/{name}.json");
                read(root, &relative).map(|text| (relative, text))
            })
            .collect::<Result<_, _>>()?,
    };
    let version = validate(&inputs)?;
    Ok(format!(
        "DESKTOP VERSION CONTRACT PASSED version={version} locales={}",
        LOCALES.len()
    ))
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative))
        .map_err(|error| format!("desktop version contract: failed to read {relative}: {error}"))
}

fn parse_json(text: &str, label: &str) -> Result<Value, String> {
    serde_json::from_str(text)
        .map_err(|error| format!("desktop version contract: invalid JSON {label}: {error}"))
}

fn cargo_version(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("version") else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim();
        if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
            return Some(rest[1..rest.len() - 1].to_owned());
        }
    }
    None
}

fn compact_whitespace(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn contains_hardcoded_visible_version(text: &str) -> bool {
    let needle = "KaspaGateway V";
    let mut start = 0;
    while let Some(offset) = text[start..].find(needle) {
        let tail = &text[start + offset + needle.len()..];
        if semver_prefix(tail) {
            return true;
        }
        start += offset + needle.len();
    }
    false
}

fn semver_prefix(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    for part in 0..3 {
        let begin = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor == begin {
            return false;
        }
        if part < 2 {
            if bytes.get(cursor) != Some(&b'.') {
                return false;
            }
            cursor += 1;
        }
    }
    true
}

fn nested_has_v0(locale: &Value) -> bool {
    locale
        .pointer("/ui/shell/kaspagateway")
        .and_then(Value::as_object)
        .is_some_and(|object| object.contains_key("v0"))
}

fn require_contains(source: &str, needle: &str, message: &str) -> Result<(), String> {
    if source.contains(needle) {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate(inputs: &Inputs) -> Result<String, String> {
    let package = parse_json(&inputs.package_json, PACKAGE_JSON)?;
    let package_version = package
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| "Desktop package.json must declare string version".to_owned())?;
    let cargo_version = cargo_version(&inputs.cargo_toml)
        .ok_or_else(|| "Desktop Cargo.toml must declare package version".to_owned())?;
    let tauri = parse_json(&inputs.tauri_config, TAURI_CONFIG)?;
    let tauri_version = tauri
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| "Desktop tauri.conf.json must declare string version".to_owned())?;

    if package_version != cargo_version {
        return Err("Desktop package.json and Cargo.toml versions must match".to_owned());
    }
    if tauri_version != cargo_version {
        return Err("Desktop tauri.conf.json and Cargo.toml versions must match".to_owned());
    }

    let diagnostics = compact_whitespace(&inputs.diagnostics_rs);
    require_contains(
        &diagnostics,
        r#"pubfnkgw_app_version_v1()->String{env!("CARGO_PKG_VERSION").to_string()}"#,
        "Desktop version IPC must derive from CARGO_PKG_VERSION",
    )?;
    require_contains(
        &inputs.lib_rs,
        "diagnostics::kgw_app_version_v1,",
        "Desktop version IPC must be registered in the Tauri invoke handler",
    )?;
    require_contains(
        &inputs.index_html,
        r#"<h1 id="kgwAppVersionTitle" data-kgw-no-i18n="true">KaspaGateway</h1>"#,
        "Desktop header must use an unversioned non-i18n fallback",
    )?;
    if contains_hardcoded_visible_version(&inputs.index_html) {
        return Err("Desktop HTML must not hard-code a visible release version".to_owned());
    }
    for (needle, message) in [
        (
            r#"invoke("kgw_app_version_v1""#,
            "Desktop Rust shell must request the authoritative package version over Tauri IPC",
        ),
        (
            "hydrate_version().await;",
            "Desktop Rust shell boot must hydrate the visible package version",
        ),
        (
            r#""versionSource""#,
            "Desktop Rust shell must mark the authoritative version source",
        ),
        (
            r#""cargo-pkg-version""#,
            "Desktop Rust shell must identify CARGO package version provenance",
        ),
    ] {
        require_contains(&inputs.shell_runtime_rs, needle, message)?;
    }

    for (path, text) in &inputs.locales {
        if contains_hardcoded_visible_version(text) {
            return Err(format!(
                "{path} must not own a hard-coded Desktop release version"
            ));
        }
        let locale = parse_json(text, path)?;
        if nested_has_v0(&locale) {
            return Err(format!(
                "{path} must not retain the retired version-only translation key"
            ));
        }
    }
    Ok(cargo_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Inputs {
        Inputs {
            package_json: r#"{"version":"0.1.3"}"#.to_owned(),
            cargo_toml: "version = \"0.1.3\"\n".to_owned(),
            tauri_config: r#"{"version":"0.1.3"}"#.to_owned(),
            index_html: r#"<h1 id="kgwAppVersionTitle" data-kgw-no-i18n="true">KaspaGateway</h1>"#
                .to_owned(),
            shell_runtime_rs: r#"invoke("kgw_app_version_v1", args).await;
hydrate_version().await;
set(&dataset(&title), "versionSource", &JsValue::from_str("cargo-pkg-version"));"#
                .to_owned(),
            diagnostics_rs:
                r#"pub fn kgw_app_version_v1() -> String { env!("CARGO_PKG_VERSION").to_string() }"#
                    .to_owned(),
            lib_rs: "diagnostics::kgw_app_version_v1,".to_owned(),
            locales: LOCALES
                .iter()
                .map(|name| {
                    (
                        format!("i18n/{name}.json"),
                        r#"{"ui":{"shell":{"kaspagateway":{}}}}"#.to_owned(),
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn complete_contract_passes() {
        assert_eq!(validate(&fixture()).unwrap(), "0.1.3");
    }

    #[test]
    fn all_three_versions_must_match() {
        let mut input = fixture();
        input.package_json = r#"{"version":"0.1.4"}"#.to_owned();
        assert!(
            validate(&input)
                .unwrap_err()
                .contains("versions must match")
        );

        let mut input = fixture();
        input.tauri_config = r#"{"version":"0.1.2"}"#.to_owned();
        assert!(
            validate(&input)
                .unwrap_err()
                .contains("versions must match")
        );
    }

    #[test]
    fn authoritative_ipc_contract_is_required() {
        let mut input = fixture();
        input.diagnostics_rs.clear();
        assert!(validate(&input).unwrap_err().contains("CARGO_PKG_VERSION"));

        let mut input = fixture();
        input.lib_rs.clear();
        assert!(validate(&input).unwrap_err().contains("invoke handler"));
    }

    #[test]
    fn hardcoded_visible_versions_are_rejected() {
        let mut input = fixture();
        input
            .index_html
            .push_str("<span>KaspaGateway V0.1.3</span>");
        assert!(validate(&input).is_err());

        let mut input = fixture();
        input.locales[0].1 = r#"{"label":"KaspaGateway V10.20.30"}"#.to_owned();
        assert!(validate(&input).is_err());
    }

    #[test]
    fn retired_locale_version_key_is_rejected() {
        let mut input = fixture();
        input.locales[0].1 = r#"{"ui":{"shell":{"kaspagateway":{"v0":"retired"}}}}"#.to_owned();
        assert!(validate(&input).unwrap_err().contains("retired"));
    }

    #[test]
    fn shell_hydration_markers_are_required() {
        for marker in [
            r#"invoke("kgw_app_version_v1""#,
            "hydrate_version().await;",
            r#""versionSource""#,
            r#""cargo-pkg-version""#,
        ] {
            let mut input = fixture();
            input.shell_runtime_rs = input.shell_runtime_rs.replace(marker, "");
            assert!(validate(&input).is_err(), "{marker}");
        }
    }

    #[test]
    fn semver_detector_preserves_prefix_behavior() {
        assert!(contains_hardcoded_visible_version("KaspaGateway V1.2.3"));
        assert!(contains_hardcoded_visible_version(
            "KaspaGateway V1.2.3-beta"
        ));
        assert!(!contains_hardcoded_visible_version("KaspaGateway"));
        assert!(!contains_hardcoded_visible_version("kaspagateway V1.2.3"));
    }
}
