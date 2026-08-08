use crate::app_logger;
use crate::commands::{DesktopAddressRecord, add_address, delete_address, list_addresses};

#[tauri::command]
pub fn get_all_addresses() -> Result<Vec<DesktopAddressRecord>, String> {
    app_logger::log_info("addresses", "get_all_addresses requested");

    match list_addresses() {
        Ok(addresses) => {
            app_logger::log_info(
                "addresses",
                &format!("get_all_addresses ok count={}", addresses.len()),
            );
            Ok(addresses)
        }
        Err(error) => {
            app_logger::log_error("addresses", &format!("get_all_addresses failed: {error}"));
            Err(error)
        }
    }
}

#[tauri::command]
pub fn save_address(address: String, name: String) -> Result<String, String> {
    app_logger::log_info(
        "addresses",
        &format!("save_address requested address={}", address),
    );

    match add_address(address, name) {
        Ok(message) => {
            app_logger::log_info("addresses", &format!("save_address ok: {message}"));
            Ok(message)
        }
        Err(error) => {
            app_logger::log_error("addresses", &format!("save_address failed: {error}"));
            Err(error)
        }
    }
}

#[tauri::command]
pub fn delete_saved_address(address: String) -> Result<String, String> {
    app_logger::log_info(
        "addresses",
        &format!("delete_saved_address requested address={}", address),
    );

    match delete_address(address) {
        Ok(message) => {
            app_logger::log_info("addresses", &format!("delete_saved_address ok: {message}"));
            Ok(message)
        }
        Err(error) => {
            app_logger::log_error(
                "addresses",
                &format!("delete_saved_address failed: {error}"),
            );
            Err(error)
        }
    }
}
