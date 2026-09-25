//! Rust-only, read-only inspection of saved zero-touch qualification evidence.

use std::path::PathBuf;
use xtask::zero_touch_evidence::{EvidenceResult, dispatch};

const USAGE: &str = "kgw-zero-touch-evidence <stages|stage|summary|wdio|policy|integrity|source-hash|recovery-files> [--repository PATH] [--artifact-directory PATH] [--stage SLUG]";

fn run() -> EvidenceResult<bool> {
    let mut arguments = std::env::args().skip(1);
    let command = arguments.next().ok_or_else(|| USAGE.to_owned())?;
    if command == "--help" || command == "-h" {
        println!(
            "{USAGE}\nReads saved evidence only; it never starts a runtime or accesses the clipboard."
        );
        return Ok(true);
    }
    let mut repository = None;
    let mut artifact = None;
    let mut stage = None;
    while let Some(option) = arguments.next() {
        let target = match option.as_str() {
            "--repository" => &mut repository,
            "--artifact-directory" => &mut artifact,
            "--stage" => &mut stage,
            _ => return Err(format!("Unknown argument: {option}\n{USAGE}")),
        };
        if target.is_some() {
            return Err(format!("Duplicate argument: {option}"));
        }
        *target = Some(
            arguments
                .next()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("{option} requires a value"))?,
        );
    }
    if !matches!(command.as_str(), "stages" | "source-hash") && artifact.is_none() {
        return Err("--artifact-directory is required".to_owned());
    }
    if matches!(command.as_str(), "integrity" | "source-hash") && repository.is_none() {
        return Err("--repository is required".to_owned());
    }
    let repository = repository
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let artifact = artifact
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let report = dispatch(&command, &repository, &artifact, stage.as_deref())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
    );
    Ok(report
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true))
}

fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
