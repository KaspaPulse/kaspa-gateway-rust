mod aud013_navigation_regressions;
mod bridge_node_mode_routing;
mod bridge_readiness_frontend_regressions;
mod ci_workflow;
mod clippy_policy;
mod copy_log;
mod deepmerge_security_smoke;
mod desktop_artifacts;
mod desktop_artifacts_stage;
mod desktop_diagnostic;
mod desktop_release_draft;
mod desktop_release_draft_stage;
mod desktop_version;
mod e2e_app_close_relaunch;
mod e2e_bridge_inprocess;
mod e2e_clipboard;
mod e2e_config_codegen;
mod e2e_explorer_live_canary;
mod e2e_explorer_ui_responsiveness;
mod e2e_installed_nsis;
mod e2e_lifecycle_recovery;
mod e2e_lifecycle_stress;
mod e2e_native_webdriver;
mod e2e_owned_process;
mod e2e_static_smokes;
mod e2e_wasm_codegen;
mod e2e_wasm_codegen_tauri_app;
mod e2e_wasm_codegen_windows;
mod e2e_wasm_codegen_zero_touch_matrix;
mod e2e_windows_evidence;
mod e2e_workspace_checks;
mod effective_bridge_settings_frontend;
mod effective_node_settings;
mod frontend_template_codegen;
mod frontend_wasm_codegen;
mod full_local_gate;
mod fuzz_build;
mod global_owner;
mod i18n_contracts;
#[cfg(feature = "ksss")]
mod ksss;
mod language_policy;
mod live_raw_log_matrix;
mod network_generation;
mod npm_dependency_policy;
mod parallel_self_worker;
mod production_trust;
mod program_unified;
mod raw_log_clipboard_capture;
mod raw_log_provenance;
mod release_admission;
mod restore_latest_frontend_regressions;
mod runtime_automation_claims;
mod runtime_repository_binding;
mod runtime_repository_binding_apply;
mod runtime_trace_owner;
mod security_advisories;
mod settings_contract_regressions;
mod start_button;
mod start_button_frontend;
mod static_contracts;
mod true_raw_log;
mod true_raw_log_frontend;
mod trufflehog_policy;
mod windows_runtime_dependencies;
mod writer_claim;
mod zero_touch_e2e;
mod zero_touch_result_writer;

use language_policy::{Mode, run_language_policy};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct CliError {
    code: i32,
    message: String,
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }

    fn failure(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
        }
    }
}

fn usage() -> &'static str {
    "usage:
  cargo run -p xtask -- aud013-navigation-regressions
  cargo run -p xtask -- bridge-node-mode-routing-audit <repo-root> <report-dir>
  cargo run -p xtask -- bridge-readiness-frontend-regressions
  cargo run -p xtask -- language-policy <check|strict|inventory>
  cargo run -p xtask -- live-raw-log-matrix [--repository <path>] [--timeout-seconds <n>]
  cargo run -p xtask -- check-clippy-results <cargo-clippy-jsonl>
  cargo run -p xtask -- ci-workflow-gate
  cargo run -p xtask -- ci-workflow-stage <ksss-adoption|msrv|quality|node-current>
  cargo run -p xtask -- check-security-advisories [--max-age-days N]
  cargo run -p xtask -- check-trufflehog-results <jsonl>
  cargo run -p xtask -- copy-log-gate
  cargo run -p xtask -- desktop-artifacts-workflow-gate
  cargo run -p xtask -- desktop-artifacts-stage <windows|macos|preserve-windows-sbom|preserve-windows-provenance|preserve-macos-sbom|preserve-macos-provenance>
  cargo run -p xtask -- desktop-diagnostic [--repository <path>]
  cargo run -p xtask -- desktop-release-draft-workflow-gate
  cargo run -p xtask -- desktop-release-draft-stage
  cargo run -p xtask -- desktop-version-contract-gate
  cargo run -p xtask -- production-trust-readiness-gate
  cargo run -p xtask -- deepmerge-security-smoke
  cargo run -p xtask -- effective-bridge-settings-gate
  cargo run -p xtask -- writer-claim <acquire|renew|verify|release|status> [--writer-id <id>] [--session-id <id>] [--epoch <n>] [--lease-secs <n>]
  cargo run -p xtask -- effective-node-settings-gate
  cargo run -p xtask -- e2e-clipboard <read|write|preflight> [--value <text>] [--output-path <path>]
  cargo run -p xtask -- e2e-config-codegen <check|write>
  cargo run -p xtask -- e2e-explorer-live-canary --app-binary <path> --output-directory <path> --data-directory <absolute-path> [--address <kaspa-address>] [--port <n>] [--window-label <label>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-explorer-ui-responsiveness --app-binary <path> --output-directory <path> --data-directory <absolute-path> [--rows <1..1000000>] [--port <n>] [--window-label <label>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-installed-nsis --installer <path> --output-directory <absolute-path> --expected-installer-sha256 <hex> --expected-installed-sha256 <hex> --expected-version <X.Y.Z> [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-app-close-relaunch --app-binary <path> --output-directory <path> [--port <n>] [--window-label <label>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-bridge-inprocess --app-binary <path> --output-directory <path> [--network <all|mainnet|testnet10>] [--port <n>] [--window-label <label>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-lifecycle-recovery --app-binary <path> --output-directory <path> [--network <all|mainnet|testnet10>] [--port <n>] [--window-label <label>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-lifecycle-stress --app-binary <path> --output-directory <path> [--port-base <n>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-native-webdriver-smoke --app-binary <path> [--port <n>] [--window-label <label>] [--startup-timeout-seconds <n>]
  cargo run -p xtask -- e2e-owned-process <kill|wait> --process-id <pid> --expected-executable <path> --expected-start-time <unix-seconds> --output-path <path> [--timeout-seconds <n>]
  cargo run -p xtask -- e2e-static-smokes
  cargo run -p xtask -- e2e-wasm-codegen <check|write>
  cargo run -p xtask -- e2e-windows-evidence --repository <path> --output-directory <path> [--ports <csv>] [--desktop-pid <pid>]
  cargo run -p xtask -- e2e-workspace-checks
  cargo run -p xtask -- frontend-template-codegen <check|write>
  cargo run -p xtask -- frontend-wasm-codegen <check|write>
  cargo run -p xtask -- full-local-gate [--repository <path>] [--reuse-successful-e2e-artifact <path>] [--commit-on-success]
  cargo run -p xtask -- release-admission local --version <X.Y.Z> [--repository <path>]
  cargo run -p xtask -- global-owner-gate [--strict] [--json] [--owner <name>] [--changed-files <files...>]
  cargo run -p xtask -- i18n-contract-gate
  cargo run -p xtask -- i18n-locale-coverage-gate
  cargo run -p xtask -- network-generation-gate
  cargo run -p xtask -- npm-dependency-policy-gate --workspace <desktop|e2e> --ci-log <path>
  cargo run -p xtask -- parallel-self-worker-runtime-gate
  cargo run -p xtask -- program-unified-gate [options]
  cargo run -p xtask -- restore-latest-frontend-regressions
  cargo run -p xtask -- raw-log-provenance-gate
  cargo run -p xtask -- runtime-automation-claims-gate
  cargo run -p xtask -- runtime-repository-binding-gate [--strict] [--online|--fresh] [--offline] [--json]
  cargo run -p xtask -- runtime-repository-binding-apply [--root <path>]
  cargo run -p xtask -- runtime-trace-owner-audit <repo-root> <report-dir>
  cargo run -p xtask -- settings-contract-regressions
  cargo run -p xtask -- start-button-gate
  cargo run -p xtask -- static-contract-regressions
  cargo run -p xtask -- true-raw-log-gate
  cargo run -p xtask -- verify-windows-runtime-dependencies --executable <path> [--dumpbin-path <path>] [--report-path <path>]
  cargo run -p xtask -- zero-touch-e2e
  cargo run -p xtask -- zero-touch-result-writer-tests
  cargo run -p xtask -- fuzz-build
  cargo run -p xtask --features ksss -- ksss <check|trust-verify|evaluate|knowledge|release-check|materialize> [options]"
}
fn repo_root() -> Result<PathBuf, CliError> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CliError::failure("xtask manifest has no repository parent"))
}

fn exactly_one(args: &mut impl Iterator<Item = String>, label: &str) -> Result<String, CliError> {
    let value = args
        .next()
        .ok_or_else(|| CliError::usage(format!("missing {label}\n{}", usage())))?;
    if args.next().is_some() {
        return Err(CliError::usage(format!(
            "too many arguments for {label}\n{}",
            usage()
        )));
    }
    Ok(value)
}

fn run() -> Result<(), CliError> {
    let mut args = env::args().skip(1);
    let command = args
        .next()
        .ok_or_else(|| CliError::usage(usage().to_owned()))?;

    match command.as_str() {
        "aud013-navigation-regressions" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "aud013-navigation-regressions takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                aud013_navigation_regressions::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "bridge-readiness-frontend-regressions" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "bridge-readiness-frontend-regressions takes no arguments\n{}",
                    usage()
                )));
            }
            let message = bridge_readiness_frontend_regressions::run(&repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "restore-latest-frontend-regressions" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "restore-latest-frontend-regressions takes no arguments\n{}",
                    usage()
                )));
            }
            let message = restore_latest_frontend_regressions::run(&repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "language-policy" => run_language_policy_command(&mut args),
        "check-clippy-results" => {
            let path = exactly_one(&mut args, "cargo-clippy-jsonl path")?;
            let message = clippy_policy::check_file(Path::new(&path)).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "ci-workflow-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "ci-workflow-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = ci_workflow::run_gate(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "ci-workflow-stage" => {
            let stage = exactly_one(&mut args, "CI workflow stage")?;
            let message =
                ci_workflow::run_stage(&repo_root()?, &stage).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "check-security-advisories" => run_security_advisories(&mut args),
        "check-trufflehog-results" => {
            let path = exactly_one(&mut args, "TruffleHog JSONL path")?;
            let message =
                trufflehog_policy::check_file(Path::new(&path)).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "copy-log-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "copy-log-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = copy_log::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "desktop-artifacts-workflow-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "desktop-artifacts-workflow-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = desktop_artifacts::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "desktop-artifacts-stage" => {
            let stage = exactly_one(&mut args, "desktop artifact stage")?;
            let message = desktop_artifacts_stage::run_stage(&repo_root()?, &stage)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "desktop-diagnostic" => {
            let message =
                desktop_diagnostic::run_cli(&mut args, &repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "desktop-release-draft-workflow-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "desktop-release-draft-workflow-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = desktop_release_draft::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "desktop-release-draft-stage" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "desktop-release-draft-stage takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                desktop_release_draft_stage::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "desktop-version-contract-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "desktop-version-contract-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = desktop_version::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "production-trust-readiness-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "production-trust-readiness-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = production_trust::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "writer-claim" => {
            let rest: Vec<String> = args.collect();
            let message = writer_claim::run(&repo_root()?, &rest).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "effective-bridge-settings-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "effective-bridge-settings-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = effective_bridge_settings_frontend::run(&repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "effective-node-settings-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "effective-node-settings-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = effective_node_settings::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "deepmerge-security-smoke" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "deepmerge-security-smoke takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                deepmerge_security_smoke::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-clipboard" => {
            let message = e2e_clipboard::run_cli(&mut args).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-config-codegen" => {
            let message =
                e2e_config_codegen::run_cli(&mut args, &repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-explorer-live-canary" => {
            let message = e2e_explorer_live_canary::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-explorer-ui-responsiveness" => {
            let message = e2e_explorer_ui_responsiveness::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-installed-nsis" => {
            let message =
                e2e_installed_nsis::run_cli(&mut args, &repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-app-close-relaunch" => {
            let message = e2e_app_close_relaunch::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-bridge-inprocess" => {
            let message = e2e_bridge_inprocess::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-lifecycle-recovery" => {
            let message = e2e_lifecycle_recovery::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-lifecycle-stress" => {
            let message = e2e_lifecycle_stress::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-native-webdriver-smoke" => {
            let message = e2e_native_webdriver::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-owned-process" => {
            let message = e2e_owned_process::run_cli(&mut args).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-static-smokes" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "e2e-static-smokes takes no arguments\n{}",
                    usage()
                )));
            }
            let message = e2e_static_smokes::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-wasm-codegen" => {
            let message =
                e2e_wasm_codegen::run_cli(&mut args, &repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-windows-evidence" => {
            let message = e2e_windows_evidence::run_cli(&mut args).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "e2e-workspace-checks" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "e2e-workspace-checks takes no arguments\n{}",
                    usage()
                )));
            }
            let message = e2e_workspace_checks::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "frontend-template-codegen" => {
            let message = frontend_template_codegen::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "frontend-wasm-codegen" => {
            let message = frontend_wasm_codegen::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "full-local-gate" => {
            let root = repo_root()?;
            let message = full_local_gate::run_cli(&mut args, &root).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "release-admission" => {
            let root = repo_root()?;
            let message =
                release_admission::run_cli(&mut args, &root).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "global-owner-gate" => {
            let result =
                global_owner::run_cli(&mut args, &repo_root()?).map_err(CliError::failure)?;
            if !result.output.is_empty() {
                println!("{}", result.output);
            }
            if result.code == 0 {
                Ok(())
            } else {
                Err(CliError {
                    code: result.code,
                    message: String::new(),
                })
            }
        }
        "i18n-contract-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "i18n-contract-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = i18n_contracts::contract(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "i18n-locale-coverage-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "i18n-locale-coverage-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                i18n_contracts::locale_coverage(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "network-generation-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "network-generation-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = network_generation::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "npm-dependency-policy-gate" => {
            match npm_dependency_policy::run_cli(&mut args, &repo_root()?) {
                Ok(message) => {
                    println!("{message}");
                    Ok(())
                }
                Err(error) if error.code == 2 => Err(CliError::usage(error.message)),
                Err(error) => Err(CliError::failure(error.message)),
            }
        }
        "parallel-self-worker-runtime-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "parallel-self-worker-runtime-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = parallel_self_worker::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "program-unified-gate" => match program_unified::run_cli(&mut args, &repo_root()?) {
            Ok(result) => {
                println!("{}", result.output);
                if result.code == 0 {
                    Ok(())
                } else {
                    Err(CliError {
                        code: result.code,
                        message: String::new(),
                    })
                }
            }
            Err(error) => Err(CliError::usage(format!("{error}\n{}", usage()))),
        },
        "raw-log-provenance-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "raw-log-provenance-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = raw_log_provenance::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "runtime-automation-claims-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "runtime-automation-claims-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                runtime_automation_claims::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "runtime-repository-binding-gate" => {
            let result = runtime_repository_binding::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{}", result.output);
            if result.code == 0 {
                Ok(())
            } else {
                Err(CliError {
                    code: result.code,
                    message: String::new(),
                })
            }
        }
        "runtime-repository-binding-apply" => {
            let message = runtime_repository_binding_apply::run_cli(&mut args, &repo_root()?)
                .map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "runtime-trace-owner-audit" => {
            let result = runtime_trace_owner::run_cli(&mut args).map_err(CliError::failure)?;
            if !result.output.is_empty() {
                println!("{}", result.output);
            }
            if result.code == 0 {
                Ok(())
            } else {
                Err(CliError {
                    code: result.code,
                    message: String::new(),
                })
            }
        }
        "bridge-node-mode-routing-audit" => {
            let result = bridge_node_mode_routing::run_cli(&mut args).map_err(CliError::failure)?;
            if !result.output.is_empty() {
                println!("{}", result.output);
            }
            if result.code == 0 {
                Ok(())
            } else {
                Err(CliError {
                    code: result.code,
                    message: String::new(),
                })
            }
        }
        "settings-contract-regressions" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "settings-contract-regressions takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                settings_contract_regressions::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "start-button-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "start-button-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = start_button::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "static-contract-regressions" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "static-contract-regressions takes no arguments\n{}",
                    usage()
                )));
            }
            let message = static_contracts::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "live-raw-log-matrix" => {
            let root = repo_root()?;
            let message =
                live_raw_log_matrix::run_cli(&mut args, &root).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "true-raw-log-frontend-regressions" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "true-raw-log-frontend-regressions takes no arguments\n{}",
                    usage()
                )));
            }
            let message = true_raw_log_frontend::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "true-raw-log-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "true-raw-log-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = true_raw_log::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "verify-windows-runtime-dependencies" => {
            let message =
                windows_runtime_dependencies::run_cli(&mut args).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "zero-touch-e2e" => {
            let message =
                zero_touch_e2e::run_cli(&mut args, &repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "zero-touch-result-writer-tests" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "zero-touch-result-writer-tests takes no arguments\n{}",
                    usage()
                )));
            }
            let message =
                zero_touch_result_writer::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "fuzz-build" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "fuzz-build takes no arguments\n{}",
                    usage()
                )));
            }
            let message = fuzz_build::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
        "ksss" => run_ksss(&mut args),
        _ => Err(CliError::usage(format!(
            "unknown xtask command: {command}\n{}",
            usage()
        ))),
    }
}

#[cfg(feature = "ksss")]
fn run_ksss(args: &mut impl Iterator<Item = String>) -> Result<(), CliError> {
    ksss::run_cli(args, &repo_root()?).map_err(CliError::failure)
}

#[cfg(not(feature = "ksss"))]
fn run_ksss(_args: &mut impl Iterator<Item = String>) -> Result<(), CliError> {
    Err(CliError::usage(
        "KSSS commands require --features ksss\n".to_owned() + usage(),
    ))
}

fn run_language_policy_command(args: &mut impl Iterator<Item = String>) -> Result<(), CliError> {
    let action = args.next().unwrap_or_else(|| "check".to_owned());
    if args.next().is_some() {
        return Err(CliError::usage(format!(
            "too many language-policy arguments\n{}",
            usage()
        )));
    }
    let result = match action.as_str() {
        "check" => run_language_policy(Mode::Migration, false),
        "strict" => run_language_policy(Mode::Strict, false),
        "inventory" => run_language_policy(Mode::Migration, true),
        _ => {
            return Err(CliError::usage(format!(
                "unknown language-policy action: {action}\n{}",
                usage()
            )));
        }
    };
    result.map_err(CliError::failure)
}
fn run_security_advisories(args: &mut impl Iterator<Item = String>) -> Result<(), CliError> {
    let mut max_age_days = 45_i64;
    if let Some(flag) = args.next() {
        if flag != "--max-age-days" {
            return Err(CliError::usage(format!(
                "unknown check-security-advisories argument: {flag}\n{}",
                usage()
            )));
        }
        let value = args.next().ok_or_else(|| {
            CliError::usage(format!("--max-age-days requires a value\n{}", usage()))
        })?;
        max_age_days = value.parse().map_err(|error| {
            CliError::usage(format!("invalid --max-age-days value {value}: {error}"))
        })?;
    }
    if args.next().is_some() {
        return Err(CliError::usage(format!(
            "too many check-security-advisories arguments\n{}",
            usage()
        )));
    }

    let message =
        security_advisories::check(&repo_root()?, max_age_days).map_err(CliError::failure)?;
    println!("{message}");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error.message);
        std::process::exit(error.code);
    }
}
