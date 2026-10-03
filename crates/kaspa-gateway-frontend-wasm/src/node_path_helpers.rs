use js_sys::Object;
use wasm_bindgen::prelude::*;

const LOCAL_APP_DATA_PLACEHOLDER: &str = "%LOCALAPPDATA%";

fn join_path_text(root: &str, child: &str) -> String {
    let base = root.trim_end_matches(['\\', '/']);
    if base.is_empty() {
        String::new()
    } else {
        format!("{base}\\{child}")
    }
}

fn windows_user_appdata_parts(value: &str) -> Option<(String, Vec<String>)> {
    let normalized = value.replace('/', "\\");
    let bytes = normalized.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || !bytes[0].is_ascii_alphabetic() {
        return None;
    }
    let drive = normalized[..2].to_owned();
    let rest = normalized[2..].trim_start_matches('\\');
    let parts = rest.split('\\').map(str::to_owned).collect::<Vec<_>>();
    Some((drive, parts))
}
fn local_app_data_from_text(value: &str) -> Option<String> {
    let (drive, parts) = windows_user_appdata_parts(value)?;
    if parts.len() < 4
        || !parts[0].eq_ignore_ascii_case("Users")
        || parts[1].is_empty()
        || !parts[2].eq_ignore_ascii_case("AppData")
        || !(parts[3].eq_ignore_ascii_case("Local") || parts[3].eq_ignore_ascii_case("Roaming"))
    {
        return None;
    }
    Some(format!(
        "{drive}\\{}\\{}\\{}\\Local",
        parts[0], parts[1], parts[2]
    ))
}

fn extract_local_app_data_from_values(values: &[String]) -> String {
    values
        .iter()
        .find_map(|value| local_app_data_from_text(value))
        .unwrap_or_else(|| LOCAL_APP_DATA_PLACEHOLDER.to_owned())
}

fn generated_path_text(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() {
        return true;
    }
    let normalized = text.replace('/', "\\");
    let lower = normalized.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("%localappdata%\\") {
        return rest == "rusty-kaspa" || rest.starts_with("rusty-kaspa\\");
    }

    let Some((_drive, parts)) = windows_user_appdata_parts(&normalized) else {
        return false;
    };
    parts.len() >= 5
        && parts[0].eq_ignore_ascii_case("Users")
        && !parts[1].is_empty()
        && parts[2].eq_ignore_ascii_case("AppData")
        && (parts[3].eq_ignore_ascii_case("Local") || parts[3].eq_ignore_ascii_case("Roaming"))
        && (parts[4].eq_ignore_ascii_case("rusty-kaspa")
            || parts[4].eq_ignore_ascii_case("KaspaGateway"))
}

#[wasm_bindgen(js_name = nodeJoinPath)]
pub fn node_join_path(root: String, child: String) -> String {
    join_path_text(&root, &child)
}

#[wasm_bindgen(js_name = nodeExtractUserLocalAppData)]
pub fn node_extract_user_local_app_data(paths: JsValue) -> String {
    if !paths.is_object() || paths.is_null() {
        return LOCAL_APP_DATA_PLACEHOLDER.to_owned();
    }
    let values = Object::values(&Object::from(paths))
        .iter()
        .map(|value| crate::js_string_owned(&value))
        .collect::<Vec<_>>();
    extract_local_app_data_from_values(&values)
}

#[wasm_bindgen(js_name = nodeRustyKaspaLocalAppDataRoot)]
pub fn node_rusty_kaspa_local_app_data_root(paths: JsValue, net: String) -> String {
    let base = node_extract_user_local_app_data(paths);
    let network = if net.trim().is_empty() {
        "mainnet"
    } else {
        net.trim()
    };
    let app_root = join_path_text(&base, "KaspaGateway");
    let nodes_root = join_path_text(&app_root, "nodes");
    join_path_text(&nodes_root, network)
}

#[wasm_bindgen(js_name = nodeIsEmptyOrGeneratedPath)]
pub fn node_is_empty_or_generated_path(value: String) -> bool {
    generated_path_text(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_path_matches_legacy_windows_shape() {
        assert_eq!(join_path_text(r"C:\Users\a\", "child"), r"C:\Users\a\child");
        assert_eq!(join_path_text("C:/Users/a/", "child"), "C:/Users/a\\child");
        assert_eq!(join_path_text("", "child"), "");
    }

    #[test]
    fn local_app_data_is_extracted_from_local_or_roaming_paths() {
        let values = vec![
            r"D:\misc".to_owned(),
            r"C:\Users\abuha\AppData\Roaming\KaspaGateway\x".to_owned(),
        ];
        assert_eq!(
            extract_local_app_data_from_values(&values),
            r"C:\Users\abuha\AppData\Local"
        );
        assert_eq!(
            extract_local_app_data_from_values(&["not-a-user-path".to_owned()]),
            LOCAL_APP_DATA_PLACEHOLDER
        );
    }

    #[test]
    fn generated_path_classification_matches_legacy_scope() {
        for value in [
            "",
            r"C:\Users\abuha\AppData\Local\rusty-kaspa",
            r"C:\Users\abuha\AppData\Roaming\KaspaGateway\nodes\mainnet",
            r"%LOCALAPPDATA%\rusty-kaspa\data",
        ] {
            assert!(generated_path_text(value), "{value}");
        }
        for value in [
            r"C:\custom\kaspa",
            r"%LOCALAPPDATA%\KaspaGateway",
            r"C:\Users\abuha\AppData\Local\OtherApp",
        ] {
            assert!(!generated_path_text(value), "{value}");
        }
    }

    #[test]
    fn network_root_uses_mainnet_default_contract() {
        let local = r"C:\Users\abuha\AppData\Local";
        let app = join_path_text(local, "KaspaGateway");
        let nodes = join_path_text(&app, "nodes");
        assert_eq!(
            join_path_text(&nodes, "mainnet"),
            r"C:\Users\abuha\AppData\Local\KaspaGateway\nodes\mainnet"
        );
    }
}
