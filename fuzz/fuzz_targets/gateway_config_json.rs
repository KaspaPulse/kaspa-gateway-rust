#![no_main]

use kaspa_gateway_config::{GatewayConfig, default_config_for_root};
use libfuzzer_sys::fuzz_target;

fn fuzz_text(data: &[u8], offset: usize, max_len: usize) -> String {
    if offset >= data.len() {
        return String::new();
    }

    let end = data.len().min(offset.saturating_add(max_len));
    String::from_utf8_lossy(&data[offset..end]).into_owned()
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
    let mut bytes = [0_u8; 8];
    for (index, slot) in bytes.iter_mut().enumerate() {
        if let Some(value) = data.get(offset + index) {
            *slot = *value;
        }
    }
    u64::from_le_bytes(bytes)
}

fn u32_at(data: &[u8], offset: usize) -> u32 {
    let mut bytes = [0_u8; 4];
    for (index, slot) in bytes.iter_mut().enumerate() {
        if let Some(value) = data.get(offset + index) {
            *slot = *value;
        }
    }
    u32::from_le_bytes(bytes)
}

fuzz_target!(|data: &[u8]| {
    // Exercise deserialization, validation, and migration/merge invariants using
    // arbitrary user-controlled configuration bytes. These calls do not create
    // files, access the network, or start node/bridge runtimes.
    if let Ok(config) = serde_json::from_slice::<GatewayConfig>(data) {
        let _ = config.validate();

        let merged = GatewayConfig::merge_user_config(config, "/tmp/kaspa-gateway-fuzz");

        if merged.validate().is_ok() {
            let _ = merged.active_api_profile();
        }
    }

    // Random JSON will rarely deserialize into the full schema, so also mutate
    // a known-good in-memory default to continuously exercise validator edges.
    let mut config = default_config_for_root("/tmp/kaspa-gateway-fuzz");

    config.performance.timeout = u64_at(data, 1);
    config.performance.retry_attempts = u32_at(data, 9);
    config.performance.auto_refresh_interval_seconds = u64_at(data, 13);

    if let Some(selector) = data.first() {
        match selector % 6 {
            0 => config.language = fuzz_text(data, 21, 32),
            1 => config.log_level = fuzz_text(data, 21, 32),
            2 => config.selected_currency = fuzz_text(data, 21, 32),
            3 => config.api.active_profile = fuzz_text(data, 21, 64),
            4 => {
                if let Some(profile) = config.api.profiles.get_mut("Default") {
                    profile.base_url = fuzz_text(data, 21, 256);
                    profile.page_limit = u32_at(data, 277);
                }
            }
            _ => {
                if let Some(profile) = config.api.profiles.get_mut("Default") {
                    profile
                        .endpoints
                        .insert(fuzz_text(data, 21, 64), fuzz_text(data, 85, 192));
                }
            }
        }
    }

    let _ = config.validate();
});
