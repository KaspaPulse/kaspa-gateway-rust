mod language_policy;

use language_policy::{Mode, run_language_policy};
use std::env;

fn usage() -> &'static str {
    "usage: cargo run -p xtask -- language-policy <check|strict|inventory>"
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(usage().to_owned());
    };
    if command != "language-policy" {
        return Err(format!("unknown xtask command: {command}\n{}", usage()));
    }

    let action = args.next().unwrap_or_else(|| "check".to_owned());
    if args.next().is_some() {
        return Err(format!("too many arguments\n{}", usage()));
    }

    match action.as_str() {
        "check" => run_language_policy(Mode::Migration, false),
        "strict" => run_language_policy(Mode::Strict, false),
        "inventory" => run_language_policy(Mode::Migration, true),
        _ => Err(format!(
            "unknown language-policy action: {action}\n{}",
            usage()
        )),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("LANGUAGE_POLICY_ERROR={error}");
        std::process::exit(2);
    }
}
