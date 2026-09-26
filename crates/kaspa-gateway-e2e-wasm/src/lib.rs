mod artifact_paths;
mod assertions;

pub use assertions::{
    is_stopped_owner_status_native, parse_key_value_line_native, pid_from_status_native,
};

use js_sys::{Error, JsString, Object, Reflect};
use wasm_bindgen::prelude::*;

const MAINNET_RPC_DEFAULT: u16 = 16110;
const MAINNET_P2P_DEFAULT: u16 = 16111;
const MAINNET_BRIDGE_DEFAULT: u16 = 5556;
const TESTNET10_RPC_DEFAULT: u16 = 16210;
const TESTNET10_P2P_DEFAULT: u16 = 16211;
const TESTNET10_BRIDGE_DEFAULT: u16 = 5656;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = String)]
    fn js_string(value: &JsValue) -> JsString;

    #[wasm_bindgen(js_name = Number)]
    fn js_number(value: &JsValue) -> f64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkPorts {
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub bridge_port: u16,
    pub external_bridge_listeners: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimePortProfile {
    pub mainnet: NetworkPorts,
    pub testnet10: NetworkPorts,
}

fn validate_port_number(value: f64, name: &str, raw: &str) -> Result<u16, String> {
    if !value.is_finite() || value.fract() != 0.0 || !(1024.0..=65535.0).contains(&value) {
        return Err(format!(
            "{name} must be an integer TCP port in 1024..65535; got {raw}"
        ));
    }
    Ok(value as u16)
}

fn profile_with(
    mut read: impl FnMut(&str, u16) -> Result<u16, String>,
) -> Result<RuntimePortProfile, String> {
    Ok(RuntimePortProfile {
        mainnet: NetworkPorts {
            rpc_port: read("KGW_E2E_MAINNET_RPC_PORT", MAINNET_RPC_DEFAULT)?,
            p2p_port: read("KGW_E2E_MAINNET_P2P_PORT", MAINNET_P2P_DEFAULT)?,
            bridge_port: read("KGW_E2E_MAINNET_BRIDGE_PORT", MAINNET_BRIDGE_DEFAULT)?,
            external_bridge_listeners: true,
        },
        testnet10: NetworkPorts {
            rpc_port: read("KGW_E2E_TESTNET10_RPC_PORT", TESTNET10_RPC_DEFAULT)?,
            p2p_port: read("KGW_E2E_TESTNET10_P2P_PORT", TESTNET10_P2P_DEFAULT)?,
            bridge_port: read("KGW_E2E_TESTNET10_BRIDGE_PORT", TESTNET10_BRIDGE_DEFAULT)?,
            external_bridge_listeners: false,
        },
    })
}

pub fn runtime_port_profile_native() -> Result<RuntimePortProfile, String> {
    profile_with(|name, fallback| {
        let raw = match std::env::var(name) {
            Ok(value) if !value.trim().is_empty() => value.trim().to_owned(),
            Ok(_) | Err(std::env::VarError::NotPresent) => return Ok(fallback),
            Err(error) => return Err(format!("failed to read {name}: {error}")),
        };
        let value = raw.parse::<f64>().unwrap_or(f64::NAN);
        validate_port_number(value, name, &raw)
    })
}

fn env_port(env: &JsValue, name: &str, fallback: u16) -> Result<u16, String> {
    let value = Reflect::get(env, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED);
    if value.is_null() || value.is_undefined() {
        return Ok(fallback);
    }

    let raw = String::from(js_string(&value)).trim().to_owned();
    if raw.is_empty() {
        return Ok(fallback);
    }

    validate_port_number(js_number(&JsValue::from_str(&raw)), name, &raw)
}

fn set(object: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    Reflect::set(object, &JsValue::from_str(key), value).map(|_| ())
}

fn network_object(profile: NetworkPorts) -> Result<Object, JsValue> {
    let object = Object::new();
    set(
        &object,
        "rpcPort",
        &JsValue::from_f64(profile.rpc_port as f64),
    )?;
    set(
        &object,
        "p2pPort",
        &JsValue::from_f64(profile.p2p_port as f64),
    )?;
    set(
        &object,
        "bridgePort",
        &JsValue::from_f64(profile.bridge_port as f64),
    )?;
    set(
        &object,
        "externalBridgeListeners",
        &JsValue::from_bool(profile.external_bridge_listeners),
    )?;
    Ok(object)
}

#[wasm_bindgen(js_name = runtimePortProfile)]
pub fn runtime_port_profile(env: JsValue) -> Result<JsValue, JsValue> {
    let env = if env.is_null() || env.is_undefined() {
        Object::new().into()
    } else {
        env
    };

    let profile = profile_with(|name, fallback| env_port(&env, name, fallback))
        .map_err(|message| JsValue::from(Error::new(&message)))?;

    let mainnet = network_object(profile.mainnet)?;
    let testnet10 = network_object(profile.testnet10)?;
    let result = Object::new();
    set(&result, "mainnet", &mainnet.into())?;
    set(&result, "testnet10", &testnet10.into())?;
    Ok(result.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> RuntimePortProfile {
        profile_with(|_, fallback| Ok(fallback)).unwrap()
    }

    #[test]
    fn defaults_match_legacy_profile() {
        assert_eq!(
            defaults(),
            RuntimePortProfile {
                mainnet: NetworkPorts {
                    rpc_port: 16110,
                    p2p_port: 16111,
                    bridge_port: 5556,
                    external_bridge_listeners: true,
                },
                testnet10: NetworkPorts {
                    rpc_port: 16210,
                    p2p_port: 16211,
                    bridge_port: 5656,
                    external_bridge_listeners: false,
                },
            }
        );
    }

    #[test]
    fn overrides_are_applied_per_key() {
        let profile = profile_with(|name, fallback| match name {
            "KGW_E2E_MAINNET_RPC_PORT" => Ok(16120),
            "KGW_E2E_MAINNET_P2P_PORT" => Ok(16121),
            "KGW_E2E_MAINNET_BRIDGE_PORT" => Ok(5566),
            _ => Ok(fallback),
        })
        .unwrap();

        assert_eq!(profile.mainnet.rpc_port, 16120);
        assert_eq!(profile.mainnet.p2p_port, 16121);
        assert_eq!(profile.mainnet.bridge_port, 5566);
        assert!(profile.mainnet.external_bridge_listeners);
        assert!(!profile.testnet10.external_bridge_listeners);
    }

    #[test]
    fn invalid_ports_fail_closed_with_legacy_message() {
        assert_eq!(
            validate_port_number(80.0, "KGW_E2E_MAINNET_RPC_PORT", "80").unwrap_err(),
            "KGW_E2E_MAINNET_RPC_PORT must be an integer TCP port in 1024..65535; got 80"
        );
        assert_eq!(
            validate_port_number(f64::NAN, "KGW_E2E_MAINNET_RPC_PORT", "abc").unwrap_err(),
            "KGW_E2E_MAINNET_RPC_PORT must be an integer TCP port in 1024..65535; got abc"
        );
        assert!(validate_port_number(65535.0, "PORT", "65535").is_ok());
        assert!(validate_port_number(65536.0, "PORT", "65536").is_err());
        assert!(validate_port_number(1024.5, "PORT", "1024.5").is_err());
    }
}
