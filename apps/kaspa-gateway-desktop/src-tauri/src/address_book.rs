use kaspa_gateway_config::default_user_data_dir;
use kaspa_gateway_core::KaspaAddress;
use kaspa_gateway_db::{AddressRecord, DatabaseManager, DatabasePaths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookIoRequest {
    pub path: String,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookImportRecord {
    pub name: Option<String>,
    pub address: String,
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddressBookIoReport {
    pub path: String,
    pub imported: usize,
    pub exported: usize,
    pub skipped: usize,
    pub warnings: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddressBookStats {
    pub count: i64,
    pub default_export_csv_path: String,
    pub default_export_json_path: String,
}

#[tauri::command]
pub fn address_book_stats() -> Result<AddressBookStats, String> {
    let manager = database_manager()?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let count = repository.count().map_err(|error| error.to_string())?;
    let root = default_user_data_dir().map_err(|error| error.to_string())?;

    Ok(AddressBookStats {
        count,
        default_export_csv_path: root
            .join("exports")
            .join("kaspa_gateway_addresses.csv")
            .display()
            .to_string(),
        default_export_json_path: root
            .join("exports")
            .join("kaspa_gateway_addresses.json")
            .display()
            .to_string(),
    })
}

#[tauri::command]
pub fn address_book_export_csv(
    request: AddressBookIoRequest,
) -> Result<AddressBookIoReport, String> {
    let path = safe_output_path(&request.path, "csv")?;
    ensure_parent(&path)?;

    let manager = database_manager()?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let records = repository.list().map_err(|error| error.to_string())?;

    let mut output = String::new();
    output.push_str("name,address,network\n");

    for record in &records {
        output.push_str(&csv_cell(&record.name));
        output.push(',');
        output.push_str(&csv_cell(&record.address));
        output.push(',');
        output.push_str(&csv_cell(&record.network));
        output.push('\n');
    }

    fs::write(&path, output).map_err(|error| error.to_string())?;

    Ok(AddressBookIoReport {
        path: path.display().to_string(),
        imported: 0,
        exported: records.len(),
        skipped: 0,
        warnings: Vec::new(),
        message: "Address book exported to CSV.".to_string(),
    })
}

#[tauri::command]
pub fn address_book_export_json(
    request: AddressBookIoRequest,
) -> Result<AddressBookIoReport, String> {
    let path = safe_output_path(&request.path, "json")?;
    ensure_parent(&path)?;

    let manager = database_manager()?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let records = repository.list().map_err(|error| error.to_string())?;

    let import_records = records
        .iter()
        .map(|record| AddressBookImportRecord {
            name: Some(record.name.clone()),
            address: record.address.clone(),
            network: Some(record.network.clone()),
        })
        .collect::<Vec<_>>();

    let output =
        serde_json::to_string_pretty(&import_records).map_err(|error| error.to_string())?;

    fs::write(&path, output).map_err(|error| error.to_string())?;

    Ok(AddressBookIoReport {
        path: path.display().to_string(),
        imported: 0,
        exported: records.len(),
        skipped: 0,
        warnings: Vec::new(),
        message: "Address book exported to JSON.".to_string(),
    })
}

#[tauri::command]
pub fn address_book_import_csv(
    request: AddressBookIoRequest,
) -> Result<AddressBookIoReport, String> {
    let path = safe_existing_path(&request.path)?;
    let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let default_network = clean_network(request.network.as_deref().unwrap_or("mainnet"));

    let manager = database_manager()?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let mut imported = 0_usize;
    let mut skipped = 0_usize;
    let mut warnings = Vec::new();

    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            skipped += 1;
            continue;
        }

        let parts = split_csv_line(line);

        if index == 0 && looks_like_header(&parts) {
            continue;
        }

        let Some((name, address, network)) = csv_record_to_fields(parts, &default_network) else {
            skipped += 1;
            warnings.push(format!("Skipped row {}: missing address.", index + 1));
            continue;
        };

        match import_one_record(&repository, &name, &address, &network) {
            Ok(()) => imported += 1,
            Err(error) => {
                skipped += 1;
                warnings.push(format!("Skipped row {}: {}", index + 1, error));
            }
        }
    }

    Ok(AddressBookIoReport {
        path: path.display().to_string(),
        imported,
        exported: 0,
        skipped,
        warnings,
        message: "Address book CSV import completed.".to_string(),
    })
}

#[tauri::command]
pub fn address_book_import_json(
    request: AddressBookIoRequest,
) -> Result<AddressBookIoReport, String> {
    let path = safe_existing_path(&request.path)?;
    let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;

    let records: Vec<AddressBookImportRecord> =
        serde_json::from_str(&text).map_err(|error| error.to_string())?;

    let default_network = clean_network(request.network.as_deref().unwrap_or("mainnet"));

    let manager = database_manager()?;
    let repository = manager
        .addresses_repository()
        .map_err(|error| error.to_string())?;

    let mut imported = 0_usize;
    let mut skipped = 0_usize;
    let mut warnings = Vec::new();

    for (index, record) in records.into_iter().enumerate() {
        let name = clean_name(record.name.as_deref().unwrap_or("Imported Address"));
        let network = clean_network(record.network.as_deref().unwrap_or(&default_network));

        match import_one_record(&repository, &name, &record.address, &network) {
            Ok(()) => imported += 1,
            Err(error) => {
                skipped += 1;
                warnings.push(format!("Skipped JSON item {}: {}", index + 1, error));
            }
        }
    }

    Ok(AddressBookIoReport {
        path: path.display().to_string(),
        imported,
        exported: 0,
        skipped,
        warnings,
        message: "Address book JSON import completed.".to_string(),
    })
}

fn import_one_record(
    repository: &kaspa_gateway_db::AddressesRepository,
    name: &str,
    address: &str,
    network: &str,
) -> Result<(), String> {
    let parsed = KaspaAddress::parse(address).map_err(|error| error.to_string())?;

    let record = AddressRecord::new(
        parsed.as_str().to_string(),
        clean_name(name),
        clean_network(network),
    )
    .map_err(|error| error.to_string())?;

    repository
        .upsert(&record)
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn safe_existing_path(value: &str) -> Result<PathBuf, String> {
    validate_path_text(value)?;

    let path = PathBuf::from(value.trim());

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    Ok(path)
}

fn safe_output_path(value: &str, extension: &str) -> Result<PathBuf, String> {
    validate_path_text(value)?;

    let mut path = PathBuf::from(value.trim());

    let needs_extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase() != extension)
        .unwrap_or(true);

    if needs_extension {
        path.set_extension(extension);
    }

    Ok(path)
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    Ok(())
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

fn clean_name(value: &str) -> String {
    let value = value.trim().replace('\0', "");

    if value.is_empty() {
        "Imported Address".to_string()
    } else {
        value
    }
}

fn clean_network(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "mainnet" | "testnet" | "devnet" | "simnet" => normalized,
        "kaspa" => "mainnet".to_string(),
        "kaspatest" => "testnet".to_string(),
        _ => "mainnet".to_string(),
    }
}

fn csv_cell(value: &str) -> String {
    let mut value = value.replace('\0', "");
    value = value.replace('\r', " ");
    value = value.replace('\n', " ");

    if value.starts_with('=')
        || value.starts_with('+')
        || value.starts_with('-')
        || value.starts_with('@')
    {
        value.insert(0, '\'');
    }

    format!("\"{}\"", value.replace('"', "\"\""))
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                cells.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    cells.push(current.trim().to_string());
    cells
}

fn looks_like_header(parts: &[String]) -> bool {
    parts.iter().any(|part| {
        let lower = part.trim().to_ascii_lowercase();
        lower == "address" || lower == "name" || lower == "network"
    })
}

fn csv_record_to_fields(
    parts: Vec<String>,
    default_network: &str,
) -> Option<(String, String, String)> {
    match parts.len() {
        0 => None,
        1 => Some((
            "Imported Address".to_string(),
            parts[0].trim().to_string(),
            default_network.to_string(),
        )),
        2 => {
            let first = parts[0].trim().to_string();
            let second = parts[1].trim().to_string();

            if first.starts_with("kaspa:")
                || first.starts_with("kaspatest:")
                || first.starts_with("kaspadev:")
                || first.starts_with("kaspasim:")
            {
                Some((
                    "Imported Address".to_string(),
                    first,
                    clean_network(&second),
                ))
            } else {
                Some((first, second, default_network.to_string()))
            }
        }
        _ => Some((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
            clean_network(parts[2].trim()),
        )),
    }
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
    fn csv_parser_handles_quoted_commas() {
        let row = split_csv_line("\"My, Address\",\"kaspa:qwerty12345678901234\",mainnet");

        assert_eq!(row[0], "My, Address");
        assert_eq!(row[1], "kaspa:qwerty12345678901234");
        assert_eq!(row[2], "mainnet");
    }

    #[test]
    fn csv_cell_blocks_formula_injection() {
        assert_eq!(csv_cell("=cmd"), "\"'=cmd\"");
        assert_eq!(csv_cell("+SUM(A1:A2)"), "\"'+SUM(A1:A2)\"");
    }

    #[test]
    fn network_is_normalized() {
        assert_eq!(clean_network("kaspa"), "mainnet");
        assert_eq!(clean_network("DEVNET"), "devnet");
        assert_eq!(clean_network("bad"), "mainnet");
    }
}
