use std::fs;
use std::path::Path;

const NODE_PATH: &str = "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-node/kaspa-node.js";
const BRIDGE_PATH: &str =
    "apps/kaspa-gateway-desktop/frontend/src/tabs/kaspa-bridge/kaspa-bridge.js";
const COMMANDS_PATH: &str = "apps/kaspa-gateway-desktop/src-tauri/src/commands.rs";

const FORBIDDEN_CLAIMS: &[&str] = &[
    "startOnLaunch",
    "autoRestart",
    "Auto-start",
    "Start on launch",
    "Auto-reconnect",
    "Startup delay sec",
    "auto_start",
    "auto_restart",
];

const REQUIRED_COMMON: &[&str] = &[
    "Effective settings apply on next Start",
    "restartRequired = running ? \"true\" : \"false\"",
    "Restart required to apply changed effective settings",
];

const NODE_ACTIONS: &[&str] = &["data-node-action=\"start\"", "data-node-action=\"stop\""];

const BRIDGE_ACTIONS: &[&str] = &[
    "data-bridge-action=\"start\"",
    "data-bridge-action=\"stop\"",
];

pub fn run(root: &Path) -> Result<String, String> {
    let node = read(root, NODE_PATH)?;
    let bridge = read(root, BRIDGE_PATH)?;
    let commands = read(root, COMMANDS_PATH)?;
    validate(&node, &bridge, &commands)?;
    Ok("KGW runtime automation claims gate PASSED".to_owned())
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative)).map_err(|error| {
        format!("runtime automation claims gate: failed to read {relative}: {error}")
    })
}

fn validate(node: &str, bridge: &str, commands: &str) -> Result<(), String> {
    let active_product_source = [node, bridge, commands].join("\n");

    for claim in FORBIDDEN_CLAIMS {
        if active_product_source.contains(claim) {
            return Err(format!(
                "unsupported runtime automation claim remains visible or persisted: {claim}"
            ));
        }
    }

    for (label, source) in [("node", node), ("bridge", bridge)] {
        for required in REQUIRED_COMMON {
            if !source.contains(required) {
                return Err(format!(
                    "{label} runtime settings contract is missing required claim: {required}"
                ));
            }
        }
    }

    for required in NODE_ACTIONS {
        if !node.contains(required) {
            return Err(format!("node runtime controls are missing: {required}"));
        }
    }
    for required in BRIDGE_ACTIONS {
        if !bridge.contains(required) {
            return Err(format!("bridge runtime controls are missing: {required}"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_node() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            REQUIRED_COMMON[0],
            REQUIRED_COMMON[1],
            REQUIRED_COMMON[2],
            NODE_ACTIONS[0],
            NODE_ACTIONS[1]
        )
    }

    fn valid_bridge() -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}",
            REQUIRED_COMMON[0],
            REQUIRED_COMMON[1],
            REQUIRED_COMMON[2],
            BRIDGE_ACTIONS[0],
            BRIDGE_ACTIONS[1]
        )
    }

    #[test]
    fn valid_contract_passes() {
        assert!(validate(&valid_node(), &valid_bridge(), "commands").is_ok());
    }

    #[test]
    fn every_unsupported_automation_claim_fails_closed() {
        for claim in FORBIDDEN_CLAIMS {
            let commands = format!("commands\n{claim}");
            let error = validate(&valid_node(), &valid_bridge(), &commands).unwrap_err();
            assert!(error.contains(claim), "{claim}");
        }
    }

    #[test]
    fn missing_common_claim_fails_for_node_and_bridge() {
        for required in REQUIRED_COMMON {
            let node = valid_node().replace(required, "");
            assert!(
                validate(&node, &valid_bridge(), "commands").is_err(),
                "{required}"
            );

            let bridge = valid_bridge().replace(required, "");
            assert!(
                validate(&valid_node(), &bridge, "commands").is_err(),
                "{required}"
            );
        }
    }

    #[test]
    fn missing_start_or_stop_controls_fail_closed() {
        for required in NODE_ACTIONS {
            let node = valid_node().replace(required, "");
            assert!(
                validate(&node, &valid_bridge(), "commands").is_err(),
                "{required}"
            );
        }
        for required in BRIDGE_ACTIONS {
            let bridge = valid_bridge().replace(required, "");
            assert!(
                validate(&valid_node(), &bridge, "commands").is_err(),
                "{required}"
            );
        }
    }
}
