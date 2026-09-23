mod clippy_policy;
mod fuzz_build;
#[cfg(feature = "ksss")]
mod ksss;
mod language_policy;
mod network_generation;
mod security_advisories;
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
  cargo run -p xtask -- network-generation-gate
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
