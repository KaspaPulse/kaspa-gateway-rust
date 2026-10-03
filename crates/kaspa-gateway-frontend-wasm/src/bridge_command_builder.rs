use crate::{bridge_command_options, bridge_frontend_helpers, bridge_instance_settings};
use js_sys::{Array, Reflect};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandProfile {
    net: String,
    testnet: bool,
    netsuffix: String,
    kaspad_port: String,
}

fn property(target: &JsValue, name: &str) -> JsValue {
    if target.is_null() || target.is_undefined() {
        return JsValue::UNDEFINED;
    }
    Reflect::get(target, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
}

fn profile_from_js(net: &str, value: &JsValue) -> CommandProfile {
    CommandProfile {
        net: net.to_owned(),
        testnet: crate::js_boolean(&property(value, "testnet")),
        netsuffix: crate::js_string_owned(&property(value, "netsuffix")),
        kaspad_port: crate::js_string_owned(&property(value, "kaspadPort")),
    }
}
fn push_value(lines: &mut Vec<String>, flag: &str, value: &str, enabled: bool) {
    if !enabled {
        return;
    }
    let clean = value.trim();
    if !clean.is_empty() {
        lines.push(format!("{flag}={clean}"));
    }
}

fn push_bool_value(lines: &mut Vec<String>, flag: &str, value: &str) {
    let clean = value.trim();
    if !clean.is_empty() && clean != "not set" {
        lines.push(format!("{flag}={clean}"));
    }
}

fn push_flag(lines: &mut Vec<String>, flag: &str, checked: bool, enabled: bool) {
    if checked && enabled {
        lines.push(flag.to_owned());
    }
}

fn push_kaspad_value(lines: &mut Vec<String>, flag: &str, value: &str, enabled: bool) {
    push_value(lines, flag, value, enabled);
}
fn build_command_lines_core<FV, FC, FI>(
    profile: &CommandProfile,
    node_mode: &str,
    config_value: &str,
    value: &FV,
    checked: &FC,
    include: &FI,
    instance_args: &[String],
) -> Vec<String>
where
    FV: Fn(&str) -> String,
    FC: Fn(&str) -> bool,
    FI: Fn(&str) -> bool,
{
    let mut lines = vec!["stratum-bridge".to_owned()];
    let mut kaspad_args = Vec::new();

    if !config_value.trim().is_empty() {
        push_value(&mut lines, "--config", config_value, true);
        push_value(
            &mut lines,
            "--node-mode",
            &value("nodeMode"),
            include("nodeMode"),
        );
        push_value(
            &mut lines,
            "--web-dashboard-port",
            &value("webDashboardPort"),
            include("webDashboardPort"),
        );
        return lines;
    }

    if profile.testnet {
        lines.push("--testnet".to_owned());
    }

    push_value(
        &mut lines,
        "--node-mode",
        &value("nodeMode"),
        include("nodeMode"),
    );
    push_value(&mut lines, "--appdir", &value("appdir"), include("appdir"));

    if node_mode == "external" {
        push_value(
            &mut lines,
            "--kaspad-address",
            &value("kaspadAddress"),
            include("kaspadAddress"),
        );
    } else if node_mode == "inprocess" {
        if profile.testnet {
            kaspad_args.push("--testnet".to_owned());
            if !profile.netsuffix.is_empty() {
                kaspad_args.push(format!("--netsuffix={}", profile.netsuffix));
            }
        }

        let rpc_listen = {
            let explicit = value("inprocessRpcListen");
            if explicit.trim().is_empty() {
                format!("127.0.0.1:{}", profile.kaspad_port)
            } else {
                explicit
            }
        };
        push_kaspad_value(
            &mut kaspad_args,
            "--rpclisten",
            &rpc_listen,
            include("inprocessRpcListen"),
        );
        push_kaspad_value(
            &mut kaspad_args,
            "--rpclisten-borsh",
            &value("inprocessRpcListenBorsh"),
            include("inprocessRpcListenBorsh"),
        );
        push_kaspad_value(
            &mut kaspad_args,
            "--rpclisten-json",
            &value("inprocessRpcListenJson"),
            include("inprocessRpcListenJson"),
        );
        for (name, flag) in [
            ("inprocessUnsafeRpc", "--unsaferpc"),
            ("inprocessUtxoIndex", "--utxoindex"),
            ("inprocessArchival", "--archival"),
        ] {
            push_flag(&mut kaspad_args, flag, checked(name), include(name));
        }
        for (name, flag) in [
            ("inprocessListen", "--listen"),
            ("inprocessAddPeer", "--addpeer"),
            ("inprocessConnect", "--connect"),
        ] {
            push_kaspad_value(&mut kaspad_args, flag, &value(name), include(name));
        }
        push_flag(
            &mut kaspad_args,
            "--disable-upnp",
            checked("inprocessDisableUpnp"),
            include("inprocessDisableUpnp"),
        );
        for (name, flag) in [
            ("inprocessMaxInpeers", "--maxinpeers"),
            ("inprocessOutpeers", "--outpeers"),
        ] {
            push_kaspad_value(&mut kaspad_args, flag, &value(name), include(name));
        }
        push_flag(
            &mut kaspad_args,
            "--perf-metrics",
            checked("inprocessPerfMetrics"),
            include("inprocessPerfMetrics"),
        );
        for (name, flag) in [
            (
                "inprocessPerfMetricsIntervalSec",
                "--perf-metrics-interval-sec",
            ),
            ("inprocessLogLevel", "--loglevel"),
            ("inprocessRamScale", "--ram-scale"),
            ("inprocessConfigfile", "--configfile"),
        ] {
            push_kaspad_value(&mut kaspad_args, flag, &value(name), include(name));
        }
        push_flag(
            &mut kaspad_args,
            "--yes",
            checked("inprocessYes"),
            include("inprocessYes"),
        );
        if profile.net != "mainnet" {
            push_kaspad_value(
                &mut kaspad_args,
                "--override-params-file",
                &value("inprocessOverrideParamsFile"),
                include("inprocessOverrideParamsFile"),
            );
            for (name, flag) in [
                ("inprocessDevnet", "--devnet"),
                ("inprocessSimnet", "--simnet"),
                ("inprocessEnableUnsyncedMining", "--enable-unsynced-mining"),
            ] {
                push_flag(&mut kaspad_args, flag, checked(name), include(name));
            }
        }
    }

    for (name, flag) in [
        ("blockWaitTime", "--block-wait-time"),
        ("printStats", "--print-stats"),
        ("logToFile", "--log-to-file"),
        ("healthCheckPort", "--health-check-port"),
        ("webDashboardPort", "--web-dashboard-port"),
        ("varDiff", "--var-diff"),
        ("sharesPerMin", "--shares-per-min"),
        ("varDiffStats", "--var-diff-stats"),
        ("extranonceSize", "--extranonce-size"),
        ("pow2Clamp", "--pow2-clamp"),
        ("coinbaseTagSuffix", "--coinbase-tag-suffix"),
    ] {
        push_value(&mut lines, flag, &value(name), include(name));
    }
    push_bool_value(
        &mut lines,
        "--approximate-geo-lookup",
        &value("approxGeoLookup"),
    );
    for (name, flag) in [
        ("stratumPort", "--stratum-port"),
        ("minShareDiff", "--min-share-diff"),
        ("promPort", "--prom-port"),
    ] {
        push_value(&mut lines, flag, &value(name), include(name));
    }

    lines.extend(instance_args.iter().cloned());

    if checked("internalCpuMiner") && profile.net != "mainnet" {
        push_flag(
            &mut lines,
            "--internal-cpu-miner",
            checked("internalCpuMiner"),
            include("internalCpuMiner"),
        );
        for (name, flag) in [
            ("internalCpuMinerAddress", "--internal-cpu-miner-address"),
            ("internalCpuMinerThreads", "--internal-cpu-miner-threads"),
            (
                "internalCpuMinerThrottleMs",
                "--internal-cpu-miner-throttle-ms",
            ),
            (
                "internalCpuMinerTemplatePollMs",
                "--internal-cpu-miner-template-poll-ms",
            ),
        ] {
            push_value(&mut lines, flag, &value(name), include(name));
        }
    }

    if !kaspad_args.is_empty() {
        lines.push("--".to_owned());
        lines.extend(kaspad_args);
    }

    lines
}

fn collect_instance_args(net: &str, instances: &Array) -> Vec<String> {
    let mut output = Vec::new();
    for instance in instances.iter() {
        let definition = bridge_instance_settings::bridge_build_upstream_instance_arg(
            net.to_owned(),
            instance.clone(),
        );
        if definition.is_empty() {
            continue;
        }
        let instance_id = property(&instance, "id");
        if bridge_command_options::bridge_instance_command_should_include_r13b(
            net.to_owned(),
            instance_id,
            "instance".to_owned(),
            instance,
        ) {
            output.push(format!("--instance={definition}"));
        }
    }
    output
}

#[wasm_bindgen(js_name = bridgeBuildCommandLines)]
pub fn bridge_build_command_lines(net: String, instances: Array) -> Array {
    let profile_value = bridge_frontend_helpers::bridge_network_profile(net.clone());
    let profile = profile_from_js(&net, &profile_value);
    let node_mode_value = bridge_frontend_helpers::bridge_value(net.clone(), "nodeMode".to_owned());
    let node_mode = if node_mode_value == "inprocess" {
        "inprocess"
    } else {
        "external"
    };
    let config_candidate = bridge_frontend_helpers::bridge_value(net.clone(), "config".to_owned());
    let config_value = if bridge_command_options::bridge_command_option_enabled_r7(
        net.clone(),
        "config".to_owned(),
    ) && !config_candidate.is_empty()
    {
        config_candidate
    } else {
        String::new()
    };

    let value = |name: &str| bridge_frontend_helpers::bridge_value(net.clone(), name.to_owned());
    let checked =
        |name: &str| bridge_frontend_helpers::bridge_checked(net.clone(), name.to_owned());
    let include = |name: &str| {
        bridge_command_options::bridge_command_should_include_r7(net.clone(), name.to_owned())
    };
    let instance_args = collect_instance_args(&net, &instances);
    let lines = build_command_lines_core(
        &profile,
        node_mode,
        &config_value,
        &value,
        &checked,
        &include,
        &instance_args,
    );

    let output = Array::new();
    for line in lines {
        output.push(&JsValue::from_str(&line));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn profile(net: &str, testnet: bool) -> CommandProfile {
        CommandProfile {
            net: net.to_owned(),
            testnet,
            netsuffix: if testnet {
                "10".to_owned()
            } else {
                String::new()
            },
            kaspad_port: if testnet {
                "16210".to_owned()
            } else {
                "16110".to_owned()
            },
        }
    }

    #[test]
    fn config_mode_short_circuits_like_legacy_builder() {
        let values = HashMap::from([
            ("nodeMode", "external"),
            ("webDashboardPort", "8080"),
            ("appdir", "ignored"),
        ]);
        let value = |name: &str| values.get(name).copied().unwrap_or("").to_owned();
        let checked = |_name: &str| false;
        let include = |_name: &str| true;
        let lines = build_command_lines_core(
            &profile("testnet10", true),
            "external",
            "C:/bridge.toml",
            &value,
            &checked,
            &include,
            &[],
        );
        assert_eq!(
            lines,
            vec![
                "stratum-bridge",
                "--config=C:/bridge.toml",
                "--node-mode=external",
                "--web-dashboard-port=8080",
            ]
        );
    }

    #[test]
    fn inprocess_testnet_preserves_separator_and_kaspad_order() {
        let values = HashMap::from([
            ("nodeMode", "inprocess"),
            ("appdir", "C:/kgw"),
            ("inprocessListen", "0.0.0.0:16211"),
            ("blockWaitTime", "1500"),
            ("approxGeoLookup", "true"),
        ]);
        let value = |name: &str| values.get(name).copied().unwrap_or("").to_owned();
        let checked_names = HashSet::from(["inprocessUnsafeRpc"]);
        let checked = |name: &str| checked_names.contains(name);
        let include = |_name: &str| true;
        let lines = build_command_lines_core(
            &profile("testnet10", true),
            "inprocess",
            "",
            &value,
            &checked,
            &include,
            &["--instance=port=17000".to_owned()],
        );
        assert_eq!(lines[0], "stratum-bridge");
        assert!(lines.contains(&"--testnet".to_owned()));
        assert!(lines.contains(&"--instance=port=17000".to_owned()));
        let separator = lines.iter().position(|line| line == "--").unwrap();
        assert_eq!(lines[separator + 1], "--testnet");
        assert_eq!(lines[separator + 2], "--netsuffix=10");
        assert_eq!(lines[separator + 3], "--rpclisten=127.0.0.1:16210");
        assert!(lines[separator + 4..].contains(&"--unsaferpc".to_owned()));
        assert!(lines[separator + 4..].contains(&"--listen=0.0.0.0:16211".to_owned()));
    }

    #[test]
    fn disabled_optional_value_is_not_emitted() {
        let values = HashMap::from([
            ("nodeMode", "external"),
            ("appdir", "C:/kgw"),
            ("kaspadAddress", "127.0.0.1:16110"),
            ("promPort", "9090"),
        ]);
        let value = |name: &str| values.get(name).copied().unwrap_or("").to_owned();
        let checked = |_name: &str| false;
        let include = |name: &str| name != "promPort";
        let lines = build_command_lines_core(
            &profile("mainnet", false),
            "external",
            "",
            &value,
            &checked,
            &include,
            &[],
        );
        assert!(lines.contains(&"--node-mode=external".to_owned()));
        assert!(lines.contains(&"--appdir=C:/kgw".to_owned()));
        assert!(lines.contains(&"--kaspad-address=127.0.0.1:16110".to_owned()));
        assert!(!lines.iter().any(|line| line.starts_with("--prom-port=")));
    }
}
