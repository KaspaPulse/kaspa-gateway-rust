mod clippy_policy;
mod desktop_artifacts;
mod desktop_release_draft;
mod desktop_version;
mod effective_node_settings;
mod fuzz_build;
mod i18n_contracts;
#[cfg(feature = "ksss")]
mod ksss;
mod language_policy;
mod network_generation;
mod npm_dependency_policy;
mod parallel_self_worker;
mod program_unified;
mod project_continuity;
mod raw_log_provenance;
mod runtime_automation_claims;
mod runtime_repository_binding;
mod security_advisories;
mod static_contracts;
mod trufflehog_policy;

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
  cargo run -p xtask -- language-policy <check|strict|inventory>
  cargo run -p xtask -- check-clippy-results <cargo-clippy-jsonl>
  cargo run -p xtask -- check-security-advisories [--max-age-days N]
  cargo run -p xtask -- check-trufflehog-results <jsonl>
  cargo run -p xtask -- desktop-artifacts-workflow-gate
  cargo run -p xtask -- desktop-release-draft-workflow-gate
  cargo run -p xtask -- desktop-version-contract-gate
  cargo run -p xtask -- effective-node-settings-gate
  cargo run -p xtask -- i18n-contract-gate
  cargo run -p xtask -- i18n-locale-coverage-gate
  cargo run -p xtask -- network-generation-gate
  cargo run -p xtask -- npm-dependency-policy-gate --workspace <desktop|e2e> --ci-log <path>
  cargo run -p xtask -- parallel-self-worker-runtime-gate
  cargo run -p xtask -- program-unified-gate [options]
  cargo run -p xtask -- project-continuity-gate
  cargo run -p xtask -- raw-log-provenance-gate
  cargo run -p xtask -- runtime-automation-claims-gate
  cargo run -p xtask -- runtime-repository-binding-gate [--strict] [--online|--fresh] [--offline] [--json]
  cargo run -p xtask -- static-contract-regressions
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
        "language-policy" => run_language_policy_command(&mut args),
        "check-clippy-results" => {
            let path = exactly_one(&mut args, "cargo-clippy-jsonl path")?;
            let message = clippy_policy::check_file(Path::new(&path)).map_err(CliError::failure)?;
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
        "project-continuity-gate" => {
            if args.next().is_some() {
                return Err(CliError::usage(format!(
                    "project-continuity-gate takes no arguments\n{}",
                    usage()
                )));
            }
            let message = project_continuity::run(&repo_root()?).map_err(CliError::failure)?;
            println!("{message}");
            Ok(())
        }
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
