use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_db::{DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const LANGUAGE_SETTING_KEY: &str = "i18n.active_language";
const I18N_REPORT_KEY: &str = "i18n.last_report.json";
const MAX_CATALOG_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nLanguageInfo {
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub rtl: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct I18nCatalogReport {
    pub active_language: String,
    pub frontend_i18n_dir: String,
    pub user_i18n_dir: String,
    pub languages: Vec<I18nLanguageInfo>,
    pub catalog_counts: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct I18nImportReport {
    pub source_dir: String,
    pub imported_files: Vec<String>,
    pub skipped_files: Vec<String>,
    pub message: String,
}

#[tauri::command]
pub fn i18n_languages() -> Vec<I18nLanguageInfo> {
    language_infos()
}

#[tauri::command]
pub fn i18n_load_catalog(language_code: String) -> Result<Value, String> {
    let code = normalize_code_checked(&language_code)?;

    let user_path = user_i18n_dir()?.join(format!("{code}.json"));
    let frontend_path = frontend_i18n_dir().join(format!("{code}.json"));
    let fallback_user = user_i18n_dir()?.join("en.json");
    let fallback_frontend = frontend_i18n_dir().join("en.json");

    for path in [
        &user_path,
        &frontend_path,
        &fallback_user,
        &fallback_frontend,
    ] {
        if path.exists() {
            return read_catalog_json(path);
        }
    }

    Ok(Value::Object(Default::default()))
}

#[tauri::command]
pub fn i18n_save_language(language_code: String) -> Result<String, String> {
    let code = normalize_code_checked(&language_code)?;

    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repo.set(LANGUAGE_SETTING_KEY, &code)
        .map_err(|error| error.to_string())?;

    Ok(code)
}

#[tauri::command]
pub fn i18n_get_active_language() -> Result<String, String> {
    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    let value = repo
        .get(LANGUAGE_SETTING_KEY)
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| "en".to_string());

    normalize_code_checked(&value).or_else(|_| Ok("en".to_string()))
}

#[tauri::command]
pub fn i18n_report() -> Result<I18nCatalogReport, String> {
    let mut warnings = Vec::new();
    let mut counts = BTreeMap::new();

    for language in language_infos() {
        match i18n_load_catalog(language.code.clone()) {
            Ok(Value::Object(map)) => {
                counts.insert(language.code, map.len());
            }
            Ok(_) => {
                counts.insert(language.code.clone(), 0);
                warnings.push(format!("Catalog {} is not a JSON object.", language.code));
            }
            Err(error) => {
                counts.insert(language.code.clone(), 0);
                warnings.push(format!("Catalog {} failed: {}", language.code, error));
            }
        }
    }

    let report = I18nCatalogReport {
        active_language: i18n_get_active_language()?,
        frontend_i18n_dir: frontend_i18n_dir().display().to_string(),
        user_i18n_dir: user_i18n_dir()?.display().to_string(),
        languages: language_infos(),
        catalog_counts: counts,
        warnings,
    };

    let manager = database_manager()?;
    let repo = manager
        .app_settings_repository()
        .map_err(|error| error.to_string())?;

    repo.set(
        I18N_REPORT_KEY,
        &serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(report)
}

#[tauri::command]
pub fn i18n_import_translations(source_dir: String) -> Result<I18nImportReport, String> {
    validate_path_text(&source_dir)?;

    let source_root = PathBuf::from(source_dir.trim());

    if !source_root.exists() {
        return Err(format!(
            "Translation source does not exist: {}",
            source_root.display()
        ));
    }

    if !source_root.is_dir() {
        return Err("Translation source must be a directory.".to_string());
    }

    let source = find_translation_source(&source_root).ok_or_else(|| {
        "Could not find Python src/translations directory or JSON catalogs.".to_string()
    })?;

    let target = user_i18n_dir()?;
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;

    let mut imported_files = Vec::new();
    let mut skipped_files = Vec::new();

    for language in language_infos() {
        let candidates = translation_candidates(&source, &language.code);
        let mut imported = false;

        for candidate in candidates {
            if !candidate.exists() {
                continue;
            }

            match read_catalog_json(&candidate) {
                Ok(Value::Object(map)) => {
                    let destination = target.join(format!("{}.json", language.code));
                    let normalized = Value::Object(map);
                    let text = serde_json::to_string_pretty(&normalized)
                        .map_err(|error| error.to_string())?;

                    fs::write(&destination, text).map_err(|error| error.to_string())?;

                    imported_files.push(destination.display().to_string());
                    imported = true;
                    break;
                }
                Ok(_) => {
                    skipped_files.push(format!("{}: not a JSON object", candidate.display()));
                }
                Err(error) => {
                    skipped_files.push(format!("{}: {}", candidate.display(), error));
                }
            }
        }

        if !imported {
            skipped_files.push(format!("{}: no matching catalog found", language.code));
        }
    }

    Ok(I18nImportReport {
        source_dir: source.display().to_string(),
        imported_files,
        skipped_files,
        message: "Python translation import completed.".to_string(),
    })
}

fn language_infos() -> Vec<I18nLanguageInfo> {
    vec![
        lang("en", "English", "English", false),
        lang("ar", "Arabic", "العربية", true),
        lang("de", "German", "Deutsch", false),
        lang("es", "Spanish", "Español", false),
        lang("fr", "French", "Français", false),
        lang("hi", "Hindi", "हिन्दी", false),
        lang("id", "Indonesian", "Bahasa Indonesia", false),
        lang("ja", "Japanese", "日本語", false),
        lang("ko", "Korean", "한국어", false),
        lang("ru", "Russian", "Русский", false),
        lang("tr", "Turkish", "Türkçe", false),
        lang("zh-CN", "Chinese (Simplified)", "简体中文", false),
    ]
}

fn lang(code: &str, name: &str, native_name: &str, rtl: bool) -> I18nLanguageInfo {
    I18nLanguageInfo {
        code: code.to_string(),
        name: name.to_string(),
        native_name: native_name.to_string(),
        rtl,
    }
}

fn normalize_code(value: &str) -> String {
    match value.trim() {
        "English" | "lang_en" | "en_US" | "en-US" => "en".to_string(),
        "Arabic" | "lang_ar" | "ar_SA" | "ar-SA" => "ar".to_string(),
        "German" | "lang_de" | "de_DE" | "de-DE" => "de".to_string(),
        "Spanish" | "lang_es" | "es_ES" | "es-ES" => "es".to_string(),
        "French" | "lang_fr" | "fr_FR" | "fr-FR" => "fr".to_string(),
        "Hindi" | "lang_hi" | "hi_IN" | "hi-IN" => "hi".to_string(),
        "Indonesian" | "lang_id" | "id_ID" | "id-ID" => "id".to_string(),
        "Japanese" | "lang_ja" | "ja_JP" | "ja-JP" => "ja".to_string(),
        "Korean" | "lang_ko" | "ko_KR" | "ko-KR" => "ko".to_string(),
        "Russian" | "lang_ru" | "ru_RU" | "ru-RU" => "ru".to_string(),
        "Turkish" | "lang_tr" | "tr_TR" | "tr-TR" => "tr".to_string(),
        "Chinese (Simplified)" | "lang_zh-CN" | "zh_CN" | "zh-cn" | "zh_Hans" => {
            "zh-CN".to_string()
        }
        other => other.trim().to_string(),
    }
}

fn normalize_code_checked(value: &str) -> Result<String, String> {
    let code = normalize_code(value);

    let allowed = language_infos()
        .into_iter()
        .any(|language| language.code == code || language.name == value);

    if allowed {
        Ok(code)
    } else {
        Err(format!("Unsupported language code: {value}"))
    }
}

fn frontend_i18n_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("frontend")
        .join("i18n")
}

fn user_i18n_dir() -> Result<PathBuf, String> {
    Ok(default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("i18n"))
}

fn read_catalog_json(path: &Path) -> Result<Value, String> {
    validate_catalog_path(path)?;

    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;

    if metadata.len() > MAX_CATALOG_BYTES {
        return Err("Catalog file is too large.".to_string());
    }

    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let value = serde_json::from_str::<Value>(&text).map_err(|error| error.to_string())?;

    Ok(value)
}

fn validate_catalog_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Catalog does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Catalog path is not a file: {}", path.display()));
    }

    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return Err("Catalog file must have .json extension.".to_string());
    };

    if !extension.eq_ignore_ascii_case("json") {
        return Err("Catalog file must have .json extension.".to_string());
    }

    Ok(())
}

fn find_translation_source(path: &Path) -> Option<PathBuf> {
    for candidate in [
        path.join("src").join("translations"),
        path.join("translations"),
        path.join("src").join("i18n"),
        path.join("i18n"),
    ] {
        if candidate.is_dir() && contains_json_file(&candidate) {
            return Some(candidate);
        }
    }

    if path.is_dir() && contains_json_file(path) {
        return Some(path.to_path_buf());
    }

    None
}

fn contains_json_file(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|value| value.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        })
}

fn translation_candidates(source: &Path, code: &str) -> Vec<PathBuf> {
    let underscore = code.replace('-', "_");
    let lowercase = code.to_ascii_lowercase();

    vec![
        source.join(format!("{code}.json")),
        source.join(format!("{underscore}.json")),
        source.join(format!("{lowercase}.json")),
        source.join(format!("lang_{code}.json")),
        source.join(format!("lang_{underscore}.json")),
        source.join(format!("translation_{code}.json")),
        source.join(format!("translations_{code}.json")),
    ]
}

fn validate_path_text(value: &str) -> Result<(), String> {
    if value.trim().is_empty()
        || value.contains('\0')
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains("&&")
        || value.contains("||")
        || value.contains('|')
        || value.contains(';')
    {
        return Err("Path contains unsafe characters.".to_string());
    }

    Ok(())
}

fn database_manager() -> Result<DatabaseManager, String> {
    let root = default_user_data_dir()
        .map_err(|error| error.to_string())?
        .join("databases");

    let paths = DatabasePaths::new(root).map_err(|error| error.to_string())?;
    let manager = DatabaseManager::new(paths);

    manager
        .initialize_all()
        .map_err(|error| error.to_string())?;

    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_codes_are_normalized() {
        assert_eq!(normalize_code("English"), "en");
        assert_eq!(normalize_code("zh_CN"), "zh-CN");
        assert!(normalize_code_checked("ar").is_ok());
    }

    #[test]
    fn unsupported_language_is_rejected() {
        assert!(normalize_code_checked("xx").is_err());
    }

    #[test]
    fn translation_candidates_include_python_names() {
        let source = PathBuf::from("translations");
        let candidates = translation_candidates(&source, "zh-CN");

        assert!(candidates.iter().any(|path| path.ends_with("zh-CN.json")));
        assert!(candidates.iter().any(|path| path.ends_with("zh_CN.json")));
        assert!(candidates
            .iter()
            .any(|path| path.ends_with("lang_zh-CN.json")));
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        assert!(validate_path_text("C:\\temp\\translations").is_ok());
        assert!(validate_path_text("bad && whoami").is_err());
    }
}
